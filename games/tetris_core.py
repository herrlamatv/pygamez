# -*- coding: utf-8 -*-
"""
tetris_core.py
==============
Das Regelwerk von Tetris - ohne Grafik, ohne Tastatur, ohne pygame.

Genutzt von tetris.py (Spiel), tetris_ai.py (KI) und dem Audit; die Datei
web/js/games/tetris_core.js ist die 1:1-Fassung für den Browser (gleiche
Zahlen, gleiche Steinfolge dank seedrand).

Regeln nach der modernen Tetris-Guideline:
- Spielfeld 10 x 20, darüber 20 verborgene Pufferzeilen. Zeile 0 ist die
  OBERSTE Pufferzeile, sichtbar sind die Zeilen BUFFER .. ROWS-1.
- Jede Zeile ist eine 10-Bit-Maske (Bit x = Spalte x belegt) - das macht
  Kollisionen und die KI-Bewertung schnell. Die Farben (Steinart je Zelle)
  liegen parallel in ``cells``.
- 7-Bag-Zufall über seedrand: zwei Felder mit demselben Seed bekommen exakt
  dieselbe Steinfolge (Versus) - auch im Browser.
- SRS: vier Drehzustände (0, R, 2, L), echte Kick-Tabellen für J/L/S/T/Z und
  eine eigene für das I; Spawn mittig über dem Feld (Zeilen 21/22), der Stein
  rutscht sofort eine Zeile herunter.
- Lock Delay 0,5 s mit höchstens 15 Resets je Tiefe ("Extended Placement").
- Wertung: T-Spin (voll/Mini, 3-Ecken-Regel + TST-Kick), Back-to-Back x1,5,
  Combo, Perfect Clear; Guideline-Gravitation; Angriffstabelle für Versus mit
  Aufrechnen gegen eingehenden Müll.
- Game Over: Block Out (Spawn belegt), Lock Out (Stein sperrt komplett über
  dem sichtbaren Feld), Top Out (Müll schiebt Blöcke aus dem Puffer).
"""

import seedrand

COLS = 10
VISIBLE = 20
BUFFER = 20                       # verborgene Zeilen über dem sichtbaren Feld
ROWS = VISIBLE + BUFFER
FULL = (1 << COLS) - 1
KINDS = "IJLOSTZ"

SPAWN_X = 3                       # linke Kante der Dreh-Box
SPAWN_Y = BUFFER - 2              # Box-Oberkante -> Steine in Zeile 21/22

LOCK_DELAY = 0.5                  # Sekunden bis zum Einrasten am Boden
MAX_RESETS = 15                   # Bewegungen/Drehungen, die ihn verlängern
SOFT_FACTOR = 20                  # Soft Drop = 20-fache Fallgeschwindigkeit
SOFT_MIN_RATE = 20.0              # ... aber mindestens 20 Zeilen pro Sekunde
GARBAGE_DELAY = 0.5               # Müll steigt frühestens nach 0,5 s auf
GARBAGE_CAP = 8                   # höchstens 8 Müllzeilen je gesetztem Stein

# ----- Formen ---------------------------------------------------------------
# Zustand 0 (Spawn-Lage) in der Dreh-Box, y wächst nach UNTEN. Die übrigen
# Zustände entstehen durch Rechtsdrehung um die Box-Mitte: (x, y) -> (n-1-y, x).
_BASE = {
    "I": ((0, 1), (1, 1), (2, 1), (3, 1)),
    "J": ((0, 0), (0, 1), (1, 1), (2, 1)),
    "L": ((2, 0), (0, 1), (1, 1), (2, 1)),
    "O": ((1, 0), (2, 0), (1, 1), (2, 1)),
    "S": ((1, 0), (2, 0), (0, 1), (1, 1)),
    "T": ((1, 0), (0, 1), (1, 1), (2, 1)),
    "Z": ((0, 0), (1, 0), (1, 1), (2, 1)),
}
_BOX = {"I": 4}                   # alle anderen: 3x3


class Shape:
    """Vorberechnete Daten eines Steins in einem Drehzustand."""

    __slots__ = ("cells", "minx", "maxx", "miny", "maxy", "rows", "bottom")

    def __init__(self, cells):
        self.cells = tuple(sorted(cells, key=lambda c: (c[1], c[0])))
        xs = [c[0] for c in cells]
        ys = [c[1] for c in cells]
        self.minx, self.maxx = min(xs), max(xs)
        self.miny, self.maxy = min(ys), max(ys)
        masks = {}
        for cx, cy in cells:
            masks[cy] = masks.get(cy, 0) | (1 << cx)
        # (dy, Maske bei x = 0) je belegter Box-Zeile
        self.rows = tuple(sorted(masks.items()))
        # unterste belegte Box-Zeile je Box-Spalte (für schnelles Fallenlassen)
        bottom = {}
        for cx, cy in cells:
            bottom[cx] = max(bottom.get(cx, -1), cy)
        self.bottom = tuple(sorted(bottom.items()))


def _build_shapes():
    shapes = {}
    for kind, base in _BASE.items():
        if kind == "O":
            # Das O dreht sich sichtbar nicht - alle vier Zustände gleich.
            shapes[kind] = [Shape(base)] * 4
            continue
        n = _BOX.get(kind, 3)
        states = [tuple(base)]
        for _ in range(3):
            states.append(tuple((n - 1 - y, x) for x, y in states[-1]))
        shapes[kind] = [Shape(s) for s in states]
    return shapes


SHAPES = _build_shapes()

# ----- SRS-Kicks --------------------------------------------------------------
# Werte wie im Tetris-Wiki: (dx, dy) mit dy nach OBEN positiv. Beim Anwenden
# wird dy gespiegelt, weil unser Feld nach unten zählt. Test 5 (Index 4) ist der
# "TST-/Fin-Kick" (1 zur Seite, 2 in der Höhe), der einen T-Spin immer zum
# vollen T-Spin macht.
KICKS_JLSTZ = {
    (0, 1): ((0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)),
    (1, 0): ((0, 0), (1, 0), (1, -1), (0, 2), (1, 2)),
    (1, 2): ((0, 0), (1, 0), (1, -1), (0, 2), (1, 2)),
    (2, 1): ((0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)),
    (2, 3): ((0, 0), (1, 0), (1, 1), (0, -2), (1, -2)),
    (3, 2): ((0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)),
    (3, 0): ((0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)),
    (0, 3): ((0, 0), (1, 0), (1, 1), (0, -2), (1, -2)),
}
KICKS_I = {
    (0, 1): ((0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)),
    (1, 0): ((0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)),
    (1, 2): ((0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)),
    (2, 1): ((0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)),
    (2, 3): ((0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)),
    (3, 2): ((0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)),
    (3, 0): ((0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)),
    (0, 3): ((0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)),
}

# T-Spin: Ecken der 3x3-Box als Index (0 = oben links, 1 = oben rechts,
# 2 = unten rechts, 3 = unten links). "Vorne" = die Seite, in die die Nase zeigt.
T_FRONT = {0: (0, 1), 1: (1, 2), 2: (2, 3), 3: (3, 0)}

# ----- Wertung (Guideline) ----------------------------------------------------
# Punkte je (Spin-Art, Zeilen) - mal Level. Spin-Art: "" / "mini" / "full".
SCORE_TABLE = {
    ("", 0): 0, ("", 1): 100, ("", 2): 300, ("", 3): 500, ("", 4): 800,
    ("mini", 0): 100, ("mini", 1): 200, ("mini", 2): 400,
    ("full", 0): 400, ("full", 1): 800, ("full", 2): 1200, ("full", 3): 1600,
}
PC_POINTS = {1: 800, 2: 1200, 3: 1800, 4: 2000}
PC_POINTS_B2B_TETRIS = 3200
COMBO_POINTS = 50

# Angriff (Müllzeilen) je (Spin-Art, Zeilen); dazu +1 für Back-to-Back,
# die Combo-Tabelle und +10 für ein Perfect Clear.
ATTACK_TABLE = {
    ("", 0): 0, ("", 1): 0, ("", 2): 1, ("", 3): 2, ("", 4): 4,
    ("mini", 0): 0, ("mini", 1): 0, ("mini", 2): 1,
    ("full", 0): 0, ("full", 1): 2, ("full", 2): 4, ("full", 3): 6,
}
COMBO_ATTACK = (0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 4, 5)   # Index = Combo-Zähler
B2B_ATTACK = 1
PC_ATTACK = 10


def gravity_interval(level):
    """Sekunden je Zeile Fall (Guideline-Formel); ab Level 20 praktisch 20G."""
    lv = max(1, min(int(level), 20))
    base = 0.8 - (lv - 1) * 0.007
    # Wiederholte Multiplikation statt ** - rechnet in Python und JavaScript
    # bitgenau gleich (pow() darf sich je Laufzeit um ein ulp unterscheiden).
    value = 1.0
    for _ in range(lv - 1):
        value *= base
    return value


def soft_interval(level):
    """Sekunden je Zeile beim Soft Drop."""
    g = gravity_interval(level)
    return min(g / SOFT_FACTOR, 1.0 / SOFT_MIN_RATE)


def collides(rows, kind, rot, x, y):
    """True, wenn der Stein an (x, y) mit Wand, Boden oder Blöcken kollidiert."""
    sh = SHAPES[kind][rot]
    if (x + sh.minx < 0 or x + sh.maxx >= COLS
            or y + sh.maxy >= ROWS or y + sh.miny < 0):
        return True
    if x >= 0:
        for dy, m in sh.rows:
            if rows[y + dy] & (m << x):
                return True
    else:
        for dy, m in sh.rows:
            if rows[y + dy] & (m >> -x):
                return True
    return False


def try_rotate(rows, kind, rot, x, y, d):
    """SRS-Drehung um d (+1 rechts, -1 links).

    Liefert (neuer_zustand, x, y, kick_index) oder None, wenn kein Kick passt.
    """
    nrot = (rot + d) % 4
    if kind == "O":
        return nrot, x, y, 0
    table = KICKS_I if kind == "I" else KICKS_JLSTZ
    for i, (kx, ky) in enumerate(table[(rot, nrot)]):
        nx, ny = x + kx, y - ky
        if not collides(rows, kind, nrot, nx, ny):
            return nrot, nx, ny, i
    return None


def drop_y(rows, kind, rot, x, y):
    """Tiefste erreichbare y-Lage beim geraden Fallen ab (x, y)."""
    while not collides(rows, kind, rot, x, y + 1):
        y += 1
    return y


class PieceQueue:
    """7-Bag-Steinfolge (seedrand -> identisch in Python und im Browser)."""

    def __init__(self, seed):
        self.rng = seedrand.Rand(seed)
        self.items = []

    def _fill(self, n):
        while len(self.items) < n:
            bag = list(KINDS)
            self.rng.shuffle(bag)
            self.items.extend(bag)

    def pop(self):
        self._fill(8)
        return self.items.pop(0)

    def peek(self, n):
        self._fill(max(n, 8))
        return self.items[:n]


class Board:
    """Ein Spielfeld mit aktivem Stein, Hold, Wertung und Müll-Warteschlange.

    Die Oberfläche liest ``events`` (und leert die Liste): ("lock", info),
    ("level", n), ("garbage", zeilen), ("hold", art), ("dead", grund).
    """

    def __init__(self, seed, level=1, fixed_level=False, garbage_seed=None):
        self.rows = [0] * ROWS
        self.cells = [[None] * COLS for _ in range(ROWS)]
        self.queue = PieceQueue(seed)
        self._grng = seedrand.Rand(seed ^ 0x5F3759DF if garbage_seed is None
                                   else garbage_seed)
        self.hold_kind = None
        self.hold_used = False
        self.start_level = max(1, int(level))
        self.level = self.start_level
        self.fixed_level = fixed_level
        self.lines = 0
        self.score = 0
        self.pieces = 0
        self.combo = -1
        self.b2b = False
        self.tetrises = 0
        self.tspins = 0
        self.attack_sent = 0
        self.garbage_received = 0
        self.pending = []                 # [[zeilen, lochspalte, alter], ...]
        self.dead = False
        self.dead_reason = None
        self.events = []
        self.version = 0                  # steigt bei jeder Feldänderung
        self.piece_id = 0
        self.active = False
        self.kind = None
        self.rot = self.x = self.y = 0
        self.spawn()

    # ----- Stein erzeugen / Zustand -------------------------------------
    def spawn(self, kind=None):
        """Nächsten Stein (oder 'kind' aus dem Hold) oben ins Feld setzen."""
        self.kind = kind if kind is not None else self.queue.pop()
        self.rot, self.x, self.y = 0, SPAWN_X, SPAWN_Y
        self.piece_id += 1
        self.last_rot = False
        self.last_kick = -1
        self.lock_timer = 0.0
        self.lock_resets = 0
        self.fall_acc = 0.0
        if collides(self.rows, self.kind, 0, self.x, self.y):
            self.active = False
            self._die("blockout")
            return False
        self.active = True
        if not collides(self.rows, self.kind, 0, self.x, self.y + 1):
            self.y += 1
        self.lowest = self.y
        return True

    def _die(self, reason):
        if not self.dead:
            self.dead = True
            self.dead_reason = reason
            self.active = False
            self.events.append(("dead", reason))

    def cells_of(self, kind=None, rot=None, x=None, y=None):
        kind = self.kind if kind is None else kind
        rot = self.rot if rot is None else rot
        x = self.x if x is None else x
        y = self.y if y is None else y
        return [(x + cx, y + cy) for cx, cy in SHAPES[kind][rot].cells]

    def ghost_y(self):
        return drop_y(self.rows, self.kind, self.rot, self.x, self.y)

    def grounded(self):
        return collides(self.rows, self.kind, self.rot, self.x, self.y + 1)

    def pending_lines(self):
        return sum(p[0] for p in self.pending)

    def stack_height(self):
        """Höhe des Stapels in Zeilen (0 = leeres Feld)."""
        for i, r in enumerate(self.rows):
            if r:
                return ROWS - i
        return 0

    # ----- Eingaben -------------------------------------------------------
    def move(self, dx):
        if not self.active:
            return False
        if collides(self.rows, self.kind, self.rot, self.x + dx, self.y):
            return False
        self.x += dx
        self.last_rot = False
        self._manipulated()
        return True

    def rotate(self, d):
        if not self.active:
            return False
        res = try_rotate(self.rows, self.kind, self.rot, self.x, self.y, d)
        if res is None:
            return False
        self.rot, self.x, self.y, self.last_kick = res
        self.last_rot = True
        self._manipulated()
        return True

    def _manipulated(self):
        """Lock-Delay-Reset nach erfolgreichem Verschieben/Drehen."""
        if self.y > self.lowest:
            self.lowest = self.y
            self.lock_resets = 0
        on_ground = self.grounded()
        if on_ground or self.lock_timer > 0:
            if self.lock_resets < MAX_RESETS:
                self.lock_resets += 1
                self.lock_timer = 0.0
            elif on_ground:
                # Resets aufgebraucht: beim nächsten Tick sofort einrasten.
                self.lock_timer = LOCK_DELAY

    def soft_step(self):
        """Eine Zeile Soft Drop (1 Punkt)."""
        if not self.active or self.grounded():
            return False
        self.y += 1
        self.score += 1
        self.last_rot = False
        self.fall_acc = 0.0
        if self.y > self.lowest:
            self.lowest = self.y
            self.lock_resets = 0
        return True

    def hard_drop(self):
        """Fallen lassen und sofort einrasten (2 Punkte je Zeile)."""
        if not self.active:
            return None
        gy = self.ghost_y()
        d = gy - self.y
        if d > 0:
            self.y = gy
            self.last_rot = False
        self.score += 2 * d
        return self._lock(drop=d)

    def hold(self):
        """Aktuellen Stein in den Hold legen (einmal je Stein)."""
        if not self.active or self.hold_used:
            return False
        cur = self.kind
        swap = self.hold_kind
        self.hold_kind = cur
        self.spawn(swap)
        self.hold_used = True
        self.events.append(("hold", cur))
        return True

    # ----- Zeit -------------------------------------------------------------
    def tick(self, dt, soft=False):
        """Schwerkraft, Soft Drop und Lock Delay für dt Sekunden."""
        for p in self.pending:
            p[2] += dt
        if not self.active:
            return
        interval = soft_interval(self.level) if soft else gravity_interval(self.level)
        if not self.grounded():
            self.fall_acc += dt / interval
            n = int(self.fall_acc)
            if n > 0:
                self.fall_acc -= n
                moved = 0
                while n > 0 and not self.grounded():
                    self.y += 1
                    n -= 1
                    moved += 1
                if moved:
                    self.last_rot = False
                    if soft:
                        self.score += moved
                    if self.y > self.lowest:
                        self.lowest = self.y
                        self.lock_resets = 0
        if self.grounded():
            self.fall_acc = 0.0
            self.lock_timer += dt
            if self.lock_timer >= LOCK_DELAY:
                self._lock()
        else:
            self.lock_timer = 0.0

    # ----- Einrasten & Wertung ---------------------------------------------
    def occupied(self, x, y):
        if x < 0 or x >= COLS or y < 0 or y >= ROWS:
            return True
        return bool(self.rows[y] >> x & 1)

    def spin_type(self):
        """None / "mini" / "full" für die aktuelle Lage (vor dem Einrasten)."""
        if self.kind != "T" or not self.last_rot:
            return None
        cx, cy = self.x + 1, self.y + 1
        occ = (self.occupied(cx - 1, cy - 1), self.occupied(cx + 1, cy - 1),
               self.occupied(cx + 1, cy + 1), self.occupied(cx - 1, cy + 1))
        if sum(occ) < 3:
            return None
        a, b = T_FRONT[self.rot]
        if (occ[a] and occ[b]) or self.last_kick == 4:
            return "full"
        return "mini"

    def _lock(self, drop=0):
        kind = self.kind
        spin = self.spin_type()
        cells = self.cells_of()
        for px, py in cells:
            self.rows[py] |= 1 << px
            self.cells[py][px] = kind
        self.version += 1
        self.pieces += 1
        self.hold_used = False
        self.active = False
        if all(py < BUFFER for _px, py in cells):
            self._die("lockout")
            return None

        full = sorted({py for _px, py in cells if self.rows[py] == FULL})
        lines = len(full)
        cleared = [list(self.cells[r]) for r in full]
        if lines:
            keep = [i for i in range(ROWS) if self.rows[i] != FULL]
            self.rows = [0] * lines + [self.rows[i] for i in keep]
            self.cells = ([[None] * COLS for _ in range(lines)]
                          + [self.cells[i] for i in keep])
        pc = lines > 0 and not any(self.rows)

        difficult = lines == 4 or (spin is not None and lines > 0)
        b2b_bonus = False
        if lines:
            if difficult:
                b2b_bonus = self.b2b
                self.b2b = True
            else:
                self.b2b = False
            self.combo += 1
        else:
            self.combo = -1

        key = (spin or "", lines)
        level = self.level
        base = SCORE_TABLE.get(key, 0)
        if b2b_bonus:
            base = base * 3 // 2
        points = base * level
        if self.combo > 0:
            points += COMBO_POINTS * self.combo * level
        if pc:
            bonus = (PC_POINTS_B2B_TETRIS if (b2b_bonus and lines == 4)
                     else PC_POINTS[lines])
            points += bonus * level
        self.score += points

        attack = ATTACK_TABLE.get(key, 0)
        if lines:
            if b2b_bonus:
                attack += B2B_ATTACK
            attack += COMBO_ATTACK[min(self.combo, len(COMBO_ATTACK) - 1)]
            if pc:
                attack += PC_ATTACK
        # Aufrechnen: eigener Angriff tilgt zuerst eingehenden Müll.
        cancelled = 0
        sent = attack
        while sent > 0 and self.pending:
            take = min(sent, self.pending[0][0])
            self.pending[0][0] -= take
            sent -= take
            cancelled += take
            if self.pending[0][0] <= 0:
                self.pending.pop(0)
        self.attack_sent += sent

        self.lines += lines
        if lines == 4:
            self.tetrises += 1
        if spin == "full" and lines:
            self.tspins += 1
        level_up = False
        if not self.fixed_level:
            new_level = max(self.start_level, 1 + self.lines // 10)
            if new_level > self.level:
                self.level = new_level
                level_up = True

        info = dict(kind=kind, lines=lines, spin=spin, b2b=b2b_bonus,
                    combo=self.combo, pc=pc, points=points, attack=attack,
                    sent=sent, cancelled=cancelled, rows=full,
                    cleared=cleared, cells=cells, drop=drop)
        self.events.append(("lock", info))
        if level_up:
            self.events.append(("level", self.level))
        if not lines:
            self._raise_garbage()
        if not self.dead:
            self.spawn()
        return info

    # ----- Müll -------------------------------------------------------------
    def receive(self, lines):
        """Eingehende Müllzeilen vormerken (ein Loch je Angriff)."""
        if lines <= 0 or self.dead:
            return
        self.pending.append([int(lines), self._grng.randint(0, COLS - 1), 0.0])

    def _raise_garbage(self):
        total = 0
        overflow = False
        while self.pending and total < GARBAGE_CAP:
            p = self.pending[0]
            if p[2] < GARBAGE_DELAY:
                break
            n = min(p[0], GARBAGE_CAP - total)
            p[0] -= n
            hole = p[1]
            if p[0] <= 0:
                self.pending.pop(0)
            for _ in range(n):
                if self.rows[0]:
                    overflow = True
                self.rows.pop(0)
                self.cells.pop(0)
                self.rows.append(FULL & ~(1 << hole))
                self.cells.append(["G" if c != hole else None
                                   for c in range(COLS)])
            total += n
        if total:
            self.version += 1
            self.garbage_received += total
            self.events.append(("garbage", total))
            if overflow:
                self._die("topout")
