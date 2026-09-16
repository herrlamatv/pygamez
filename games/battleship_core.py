# -*- coding: utf-8 -*-
"""
battleship_core.py
==================
Regeln und KI für Battleship / Schiffe versenken - ganz ohne pygame, damit
Spiel und Audit-Test dieselbe Logik benutzen (der Web-Port in
web/js/games/battleship.js folgt ihr Zeile für Zeile).

- Brett 10x10, Flotte 5/4/3/3/2 (FLEET, Namen über SHIP_KEYS).
- Sea: ein Brett mit Schiffen und den Schüssen, die darauf gefallen sind.
- Platzierungsregel: Schiffe liegen gerade im Brett und überlappen nicht.
  Mit touch=False dürfen sie sich auch nicht berühren - weder an der Seite
  noch über Eck.
- ShotAI: drei Stärken.
    easy   - schießt zufällig, setzt nach einem Treffer nur ab und zu nach.
    medium - Jagen/Zielen: zufällig suchen, nach Treffern die Nachbarn
             abklopfen und Trefferlinien verlängern.
    hard   - Wahrscheinlichkeitskarte über ALLE noch legalen Lagen der
             übrigen Schiffe (inkl. Berühr-Regel); beim Suchen nur auf
             Paritätsfeldern des kleinsten Restschiffs.
  Die KI sieht nur, was auch ein Mensch weiß: Wasser, Treffer und die
  Lage versenkter Schiffe (die im Spiel enthüllt werden).

Felder werden intern als Index 0..99 (= r * 10 + c) geführt; die Lagen der
Schiffe sind als Bitmasken vorberechnet, das hält die schwere KI auch in
Python weit unter einer Millisekunde pro Schuss.
"""

import random

N = 10
FLEET = (5, 4, 3, 3, 2)
SHIP_KEYS = ("carrier", "battleship", "cruiser", "submarine", "destroyer")

UNKNOWN, MISS, HIT = 0, 1, 2

EASY, MEDIUM, HARD = 0, 1, 2


def cells_of(r, c, size, horiz):
    """Felder (r, c) eines Schiffs ab seinem ersten Feld (links bzw. oben)."""
    if horiz:
        return [(r, c + i) for i in range(size)]
    return [(r + i, c) for i in range(size)]


def in_board(r, c):
    return 0 <= r < N and 0 <= c < N


def _neighbors8(r, c):
    for dr in (-1, 0, 1):
        for dc in (-1, 0, 1):
            if (dr or dc) and in_board(r + dr, c + dc):
                yield r + dr, c + dc


def _build_placements():
    """Alle Lagen je Schiffslänge: (maske, feld-indizes, randmaske).

    Die Randmaske umfasst die 8er-Nachbarschaft des Schiffs ohne das Schiff
    selbst - gebraucht für die Berühr-Regel.
    """
    table = {}
    for size in sorted(set(FLEET)):
        lst = []
        for horiz in (True, False):
            for r in range(N if horiz else N - size + 1):
                for c in range(N - size + 1 if horiz else N):
                    cells = cells_of(r, c, size, horiz)
                    mask = 0
                    idx = []
                    for (rr, cc) in cells:
                        mask |= 1 << (rr * N + cc)
                        idx.append(rr * N + cc)
                    halo = 0
                    for (rr, cc) in cells:
                        for (nr, nc) in _neighbors8(rr, cc):
                            halo |= 1 << (nr * N + nc)
                    halo &= ~mask
                    lst.append((mask, tuple(idx), halo))
        table[size] = lst
    return table


PLACEMENTS = _build_placements()

# 8er-Nachbarschaft und Diagonal-Nachbarn je Feld als Maske
HALO8 = [0] * (N * N)
DIAG = [0] * (N * N)
for _r in range(N):
    for _c in range(N):
        for (_nr, _nc) in _neighbors8(_r, _c):
            HALO8[_r * N + _c] |= 1 << (_nr * N + _nc)
            if _nr != _r and _nc != _c:
                DIAG[_r * N + _c] |= 1 << (_nr * N + _nc)


# ===================================================================== Brett
class Ship:
    """Ein Schiff der Flotte. idx = Position in FLEET (bestimmt Name/Länge)."""

    __slots__ = ("idx", "size", "r", "c", "horiz", "hits")

    def __init__(self, idx, r, c, horiz):
        self.idx = idx
        self.size = FLEET[idx]
        self.r = r
        self.c = c
        self.horiz = horiz
        self.hits = set()

    @property
    def cells(self):
        return cells_of(self.r, self.c, self.size, self.horiz)

    @property
    def sunk(self):
        return len(self.hits) >= self.size


class Sea:
    """Ein Spielbrett: eigene Schiffe + alle Schüsse, die darauf fielen."""

    def __init__(self):
        self.ships = []
        self.shots = [[UNKNOWN] * N for _ in range(N)]

    # ----- Aufstellen ----------------------------------------------------
    def ship_at(self, r, c):
        for s in self.ships:
            if (r, c) in s.cells:
                return s
        return None

    def ship_by_idx(self, idx):
        for s in self.ships:
            if s.idx == idx:
                return s
        return None

    def can_place(self, idx, r, c, horiz, touch=True, ignore=None):
        """Darf Schiff 'idx' ab Feld (r, c) so liegen? 'ignore' = gerade gehaltenes Schiff."""
        cells = cells_of(r, c, FLEET[idx], horiz)
        if not all(in_board(rr, cc) for (rr, cc) in cells):
            return False
        mine = set(cells)
        for s in self.ships:
            if s is ignore or s.idx == idx:
                continue
            for (rr, cc) in s.cells:
                if (rr, cc) in mine:
                    return False
                if not touch:
                    for (nr, nc) in _neighbors8(rr, cc):
                        if (nr, nc) in mine:
                            return False
        return True

    def place(self, idx, r, c, horiz):
        """Setzt Schiff 'idx' (ersetzt eine frühere Lage desselben Schiffs)."""
        old = self.ship_by_idx(idx)
        if old is not None:
            self.ships.remove(old)
        ship = Ship(idx, r, c, horiz)
        self.ships.append(ship)
        return ship

    def remove(self, ship):
        if ship in self.ships:
            self.ships.remove(ship)

    def clear(self):
        self.ships = []

    def complete(self):
        return len(self.ships) == len(FLEET)

    def randomize(self, touch=True, rng=None, keep=()):
        """Stellt die Flotte zufällig (legal) auf; Schiffe in 'keep' bleiben liegen."""
        rng = rng or random
        fixed = [s for s in self.ships if s.idx in keep]
        for _attempt in range(200):
            self.ships = list(fixed)
            ok = True
            for idx in sorted(range(len(FLEET)), key=lambda i: -FLEET[i]):
                if self.ship_by_idx(idx) is not None:
                    continue
                spots = [(r, c, h) for h in (True, False)
                         for r in range(N) for c in range(N)
                         if self.can_place(idx, r, c, h, touch)]
                if not spots:
                    ok = False
                    break
                r, c, h = rng.choice(spots)
                self.place(idx, r, c, h)
            if ok:
                return True
            fixed = []            # festgefahren -> ganz neu würfeln
        return False

    def valid_layout(self, touch=True):
        """Prüft eine komplette Aufstellung gegen alle Regeln."""
        if sorted(s.idx for s in self.ships) != list(range(len(FLEET))):
            return False
        for s in self.ships:
            if not self.can_place(s.idx, s.r, s.c, s.horiz, touch, ignore=s):
                return False
        return True

    # ----- Schießen ------------------------------------------------------
    def fire(self, r, c):
        """Schuss auf (r, c): ("miss"|"hit"|"sunk", schiff) oder (None, None)."""
        if not in_board(r, c) or self.shots[r][c] != UNKNOWN:
            return None, None
        ship = self.ship_at(r, c)
        if ship is None:
            self.shots[r][c] = MISS
            return "miss", None
        self.shots[r][c] = HIT
        ship.hits.add((r, c))
        return ("sunk" if ship.sunk else "hit"), ship

    def all_sunk(self):
        return bool(self.ships) and all(s.sunk for s in self.ships)

    def ships_left(self):
        return sum(1 for s in self.ships if not s.sunk)

    def untouched(self):
        return sum(1 for row in self.shots for v in row if v == UNKNOWN)

    def knowledge(self):
        """Was ein Schütze über dieses Brett weiß.

        (grid, sunk, remaining): grid = 100 Werte UNKNOWN/MISS/HIT,
        sunk = Feld-Indizes je versenktem Schiff, remaining = Längen der
        noch schwimmenden Schiffe. Lage und Schaden unversenkter Schiffe
        stehen NICHT darin (außer als Treffer im grid).
        """
        grid = [self.shots[i // N][i % N] for i in range(N * N)]
        sunk = [[r * N + c for (r, c) in s.cells] for s in self.ships if s.sunk]
        remaining = [s.size for s in self.ships if not s.sunk]
        return grid, sunk, remaining


# ======================================================================= KI
class ShotAI:
    """Wählt Schüsse auf ein gegnerisches Brett (nur über Sea.knowledge())."""

    def __init__(self, level=MEDIUM, touch=True, rng=None):
        self.level = level
        self.touch = touch
        self.rng = rng or random.Random()
        self.parity = self.rng.randrange(6)   # Versatz des Paritätsgitters

    def choose(self, sea):
        grid, sunk, remaining = sea.knowledge()
        return self.choose_from(grid, sunk, remaining)

    def choose_from(self, grid, sunk, remaining):
        free = [i for i in range(N * N) if grid[i] == UNKNOWN]
        if not free:
            return None
        if self.level == HARD:
            i = self._hard(grid, sunk, remaining, free)
        elif self.level == MEDIUM:
            i = self._medium(grid, sunk, free)
        else:
            i = self._easy(grid, sunk, free)
        return divmod(i, N)

    # ----- Hilfen --------------------------------------------------------
    @staticmethod
    def _masks(grid, sunk):
        miss = hit = sunk_m = 0
        for i, v in enumerate(grid):
            if v == MISS:
                miss |= 1 << i
            elif v == HIT:
                hit |= 1 << i
        for cells in sunk:
            for i in cells:
                sunk_m |= 1 << i
        return miss, hit & ~sunk_m, sunk_m

    def _known_empty(self, sunk_m, open_hits):
        """Felder, die nach der Berühr-Regel sicher leer sind."""
        if self.touch:
            return 0
        empty = 0
        m = sunk_m
        while m:
            low = m & -m
            empty |= HALO8[low.bit_length() - 1]
            m ^= low
        m = open_hits
        while m:
            low = m & -m
            empty |= DIAG[low.bit_length() - 1]
            m ^= low
        return empty

    @staticmethod
    def _bits(mask):
        out = []
        while mask:
            low = mask & -mask
            out.append(low.bit_length() - 1)
            mask ^= low
        return out

    def _neighbors4(self, open_hits, free_set):
        out = []
        for i in self._bits(open_hits):
            r, c = divmod(i, N)
            for dr, dc in ((-1, 0), (1, 0), (0, -1), (0, 1)):
                rr, cc = r + dr, c + dc
                if in_board(rr, cc) and rr * N + cc in free_set:
                    out.append(rr * N + cc)
        return out

    # ----- Leicht: zufällig ----------------------------------------------
    def _easy(self, grid, sunk, free):
        _miss, open_hits, _sunk_m = self._masks(grid, sunk)
        if open_hits and self.rng.random() < 0.35:
            near = self._neighbors4(open_hits, set(free))
            if near:
                return self.rng.choice(near)
        return self.rng.choice(free)

    # ----- Mittel: jagen und zielen --------------------------------------
    def _medium(self, grid, sunk, free):
        _miss, open_hits, sunk_m = self._masks(grid, sunk)
        empty = self._known_empty(sunk_m, open_hits)
        free_set = {i for i in free if not (empty >> i) & 1} or set(free)
        if open_hits:
            hits = set(self._bits(open_hits))
            ends = []
            for i in hits:
                r, c = divmod(i, N)
                for dr, dc in ((0, 1), (1, 0)):
                    if (r + dr) * N + (c + dc) not in hits or not in_board(r + dr, c + dc):
                        continue
                    # Linie in beide Richtungen bis zum Ende verfolgen
                    for sgn in (1, -1):
                        rr, cc = r, c
                        while in_board(rr, cc) and rr * N + cc in hits:
                            rr += dr * sgn
                            cc += dc * sgn
                        if in_board(rr, cc) and rr * N + cc in free_set:
                            ends.append(rr * N + cc)
            if ends:
                return self.rng.choice(ends)
            near = self._neighbors4(open_hits, free_set)
            if near:
                return self.rng.choice(near)
        return self.rng.choice(sorted(free_set))

    # ----- Schwer: Wahrscheinlichkeitskarte ------------------------------
    def density(self, grid, sunk, remaining):
        """Gewichte je Feld über alle legalen Lagen der Restschiffe."""
        miss, open_hits, sunk_m = self._masks(grid, sunk)
        blocked = miss | sunk_m | self._known_empty(sunk_m, open_hits)
        touching = open_hits | sunk_m
        dens = [0] * (N * N)
        target = open_hits != 0
        for size in remaining:
            for mask, cells, halo in PLACEMENTS[size]:
                if mask & blocked:
                    continue
                if not self.touch and halo & touching:
                    continue
                if target:
                    k = (mask & open_hits).bit_count()
                    if not k:
                        continue
                    w = 8 ** k
                else:
                    w = 1
                for i in cells:
                    dens[i] += w
        return dens, target

    def _hard(self, grid, sunk, remaining, free):
        dens, target = self.density(grid, sunk, remaining)
        cand = [i for i in free if dens[i] > 0]
        if not cand and target:
            # Treffer lassen sich nicht erklären (sollte nicht vorkommen) ->
            # wie beim Suchen weitermachen.
            dens, target = self.density([MISS if v == HIT else v for v in grid],
                                        sunk, remaining)
            cand = [i for i in free if dens[i] > 0]
        if not cand:
            return self.rng.choice(free)
        if not target and remaining:
            step = min(remaining)
            par = [i for i in cand
                   if (i // N + i % N + self.parity) % step == 0]
            if par:
                cand = par
        best = max(dens[i] for i in cand)
        return self.rng.choice([i for i in cand if dens[i] == best])


def play_out(level, touch=True, rng=None, sea=None):
    """Lässt die KI ein komplettes Brett abräumen; liefert die Schusszahl.

    Für den Audit-Test: jede Stärke muss jede Flotte in <= 100 Schüssen
    versenken und darf kein Feld doppelt beschießen.
    """
    rng = rng or random.Random()
    if sea is None:
        sea = Sea()
        sea.randomize(touch, rng)
    ai = ShotAI(level, touch, rng)
    shots = 0
    while not sea.all_sunk():
        pick = ai.choose(sea)
        if pick is None:
            break
        res, _ship = sea.fire(*pick)
        if res is None:
            return 10 ** 6          # doppelter Schuss = Fehler
        shots += 1
    return shots
