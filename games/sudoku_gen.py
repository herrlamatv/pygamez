# -*- coding: utf-8 -*-
"""
sudoku_gen.py
=============
Deterministischer Sudoku-Generator + Löser (OHNE pygame-Import, damit die
Generierung headless testbar bleibt und sudoku.py schlank ist).

Level-System (klassische Level)
-------------------------------
Es gibt LEVELS (100) Level je Schwierigkeitsgrad. Level N von Stufe D ist
IMMER dasselbe Puzzle: alle Zufallsentscheidungen (Lösungs-Aufbau und
Loch-Reihenfolge) kommen aus einem einzigen ``random.Random(seed_for(D, N))``.
Der Seed ist ein reiner int (kein hash() auf Tupeln - der wäre nicht über
Prozesse hinweg stabil).

Ablauf von ``generate``:
1. ``_fill``  : volle, gültige Lösung per randomisiertem Backtracking.
2. ``_dig``   : Zellen in zufälliger Reihenfolge leeren; eine Zelle bleibt
                nur leer, wenn das Puzzle EINDEUTIG lösbar bleibt
                (``_count_solutions`` mit Abbruch bei 2 Lösungen).

Diese klassischen Level bleiben BEWUSST beim alten Generator (Python-random):
sonst wären alle schon abgehakten Level plötzlich andere Rätsel.

Varianten (X-Sudoku, Mini 6x6, Killer, Tages-Sudoku)
----------------------------------------------------
Alles Neue arbeitet auf ``Layout``-Objekten: Brettgröße, Blockform und die
Liste aller Einheiten (Zeilen, Spalten, Blöcke, bei X-Sudoku zusätzlich die
zwei Diagonalen). Zufall kommt aus ``seedrand`` - dadurch liefern Desktop und
Browser (``web/js/games/sudoku_gen.js``) bitgenau dieselben Rätsel.

- X-Sudoku und Mini 6x6 entstehen zur Laufzeit (``generate_variant``).
  Leicht/Normal graben nur so lange, wie das Rätsel allein mit "Singles"
  (einzige Ziffer in einer Zelle / einziger Platz in einer Einheit) lösbar
  bleibt; Schwer muss nur eindeutig lösbar sein (weniger Vorgaben), Experte
  gräbt zusätzlich weiter, bis Singles allein nicht mehr reichen.
- Tages-Sudoku (``generate_daily``): klassisches 9x9 aus dem Tages-Seed, die
  Stufe hängt vom Wochentag ab.
- Killer-Sudoku ist vorab erzeugt (``devtools/build_sudoku_killer.py`` ->
  ``games/levels/sudoku-killer.json``), weil der Eindeutigkeitsbeweis mit
  Käfigsummen für die Laufzeit zu langsam ist. ``count_solutions`` kann
  Käfige trotzdem prüfen (Build-Skript und Test nutzen das).

Alle Bretter sind flache Listen mit n*n Einträgen (Index = zeile*n+spalte,
0 = leer, 1..n = Ziffer).
"""

import json
import os
import random

import seedrand

LEVELS = 100

# Ziel-Anzahl an Vorgaben je Schwierigkeitsgrad (Leicht/Normal/Schwer/Experte).
# Der Graber stoppt, sobald das Ziel erreicht ist; bleibt eine Stufe in einem
# Level knapp darüber (Eindeutigkeit!), ist das in Ordnung.
CLUES = (42, 34, 29, 25)

SEED_BASE = 987_654_321

# ----- Index-Tabellen ------------------------------------------------------

ROW_OF = tuple(i // 9 for i in range(81))
COL_OF = tuple(i % 9 for i in range(81))
BOX_OF = tuple((i // 27) * 3 + (i % 9) // 3 for i in range(81))

# 27 Einheiten (9 Zeilen, 9 Spalten, 9 Boxen) mit je 9 Zell-Indizes.
UNITS = ([tuple(r * 9 + c for c in range(9)) for r in range(9)] +
         [tuple(r * 9 + c for r in range(9)) for c in range(9)] +
         [tuple((br * 3 + r) * 9 + bc * 3 + c
                for r in range(3) for c in range(3))
          for br in range(3) for bc in range(3)])

# Die 20 "Peers" jeder Zelle (gleiche Zeile/Spalte/Box, ohne sich selbst).
PEERS = tuple(
    frozenset(j for u in UNITS if i in u for j in u if j != i)
    for i in range(81)
)

_ALL = 0x1FF          # Bitmaske: alle 9 Kandidaten (Bit d-1 = Ziffer d)


def seed_for(diff, level):
    """Stabiler int-Seed für (Schwierigkeitsgrad, Level)."""
    return SEED_BASE + int(diff) * 1_000_000 + int(level)


def _fill(rng):
    """Volle Lösung per randomisiertem Backtracking (Bitmasken-Kandidaten)."""
    board = [0] * 81
    row = [_ALL] * 9      # je Einheit: Bitmaske der noch freien Ziffern
    col = [_ALL] * 9
    box = [_ALL] * 9

    def solve(i):
        if i == 81:
            return True
        r, c, b = ROW_OF[i], COL_OF[i], BOX_OF[i]
        cand = row[r] & col[c] & box[b]
        if not cand:
            return False
        digits = [d for d in range(1, 10) if cand & (1 << (d - 1))]
        rng.shuffle(digits)
        for d in digits:
            m = 1 << (d - 1)
            board[i] = d
            row[r] ^= m
            col[c] ^= m
            box[b] ^= m
            if solve(i + 1):
                return True
            board[i] = 0
            row[r] |= m
            col[c] |= m
            box[b] |= m
        return False

    solve(0)
    return board


def _count_solutions(board, limit=2):
    """Zählt Lösungen (MRV-Heuristik, Abbruch bei ``limit``). Hot Path!"""
    row = [_ALL] * 9
    col = [_ALL] * 9
    box = [_ALL] * 9
    empty = []
    for i in range(81):
        d = board[i]
        if d:
            m = 1 << (d - 1)
            r, c, b = ROW_OF[i], COL_OF[i], BOX_OF[i]
            if not (row[r] & m and col[c] & m and box[b] & m):
                return 0          # Vorgaben widersprechen sich
            row[r] ^= m
            col[c] ^= m
            box[b] ^= m
        else:
            empty.append(i)

    count = 0

    def solve():
        nonlocal count
        # MRV: leere Zelle mit den wenigsten Kandidaten zuerst.
        best_i = -1
        best_cand = 0
        best_n = 10
        for i in empty:
            if board[i]:
                continue
            cand = row[ROW_OF[i]] & col[COL_OF[i]] & box[BOX_OF[i]]
            n = cand.bit_count()
            if n == 0:
                return
            if n < best_n:
                best_i, best_cand, best_n = i, cand, n
                if n == 1:
                    break
        if best_i < 0:            # keine leere Zelle mehr -> Lösung gefunden
            count += 1
            return
        r, c, b = ROW_OF[best_i], COL_OF[best_i], BOX_OF[best_i]
        cand = best_cand
        while cand:
            m = cand & -cand      # niedrigstes gesetztes Bit
            cand ^= m
            board[best_i] = m.bit_length()
            row[r] ^= m
            col[c] ^= m
            box[b] ^= m
            solve()
            board[best_i] = 0
            row[r] |= m
            col[c] |= m
            box[b] |= m
            if count >= limit:
                return

    solve()
    return count


def _dig(solution, target, rng):
    """Leert Zellen der Lösung, solange das Puzzle eindeutig lösbar bleibt.

    Ein einziger, rng-geshuffelter Durchlauf über alle 81 Positionen deckelt
    die Arbeit (max. 81 Eindeutigkeits-Prüfungen) und ist je Seed
    deterministisch. Gestoppt wird, sobald nur noch ``target`` Vorgaben da sind.
    """
    puzzle = list(solution)
    order = list(range(81))
    rng.shuffle(order)
    clues = 81
    for i in order:
        if clues <= target:
            break
        saved = puzzle[i]
        puzzle[i] = 0
        if _count_solutions(list(puzzle)) == 1:
            clues -= 1
        else:
            puzzle[i] = saved
    return puzzle


def generate(diff, level):
    """Erzeugt (puzzle, solution) für Stufe ``diff`` (0..3), Level ``level``.

    Deterministisch: gleicher Aufruf -> identisches Puzzle.
    """
    diff = max(0, min(len(CLUES) - 1, int(diff)))
    rng = random.Random(seed_for(diff, level))
    solution = _fill(rng)
    puzzle = _dig(solution, CLUES[diff], rng)
    return puzzle, solution


# =========================================================================
#  Varianten: allgemeine Einheiten-Listen + seedrand (Desktop == Browser)
# =========================================================================

VARIANTS = ("classic", "x", "killer", "mini")

# Vorgaben-Ziel je Stufe (Leicht/Normal/Schwer/Experte) für die zur Laufzeit
# erzeugten Varianten. "classic" gilt hier nur fürs Tages-Sudoku - die
# klassischen Level nutzen weiter CLUES + den alten Generator oben.
VARIANT_CLUES = {
    "classic": (42, 34, 29, 25),
    "x": (36, 29, 25, 21),
    "mini": (20, 16, 13, 10),
}

# Grab-Regel je Stufe:
#   "singles" - muss allein mit Singles lösbar bleiben (Leicht, Normal)
#   "unique"  - muss nur eindeutig lösbar bleiben (Schwer)
#   "hard"    - wie "unique", gräbt aber nach dem Ziel weiter, solange das
#               Rätsel noch mit Singles aufgeht (Experte braucht mehr)
DIG_RULES = ("singles", "singles", "unique", "hard")

# Tages-Sudoku: Stufe je Wochentag (Montag = 0 ... Sonntag = 6).
DAILY_DIFF = (0, 1, 1, 2, 2, 3, 2)


class Layout:
    """Geometrie eines Sudoku-Typs: Größe, Blockform und alle Einheiten.

    n        : Kantenlänge (9 oder 6), Ziffern 1..n
    box_h/w  : Blockhöhe/-breite (9x9: 3x3, Mini: 2 Zeilen x 3 Spalten)
    units    : Tupel aller Einheiten (Zeilen, Spalten, Blöcke, ggf. Diagonalen)
    units_of : je Zelle die Indizes der Einheiten, in denen sie liegt
    peers    : je Zelle alle Zellen, die keine gleiche Ziffer haben dürfen
    """

    def __init__(self, n, box_h, box_w, diagonals=False):
        self.n = n
        self.size = n * n
        self.box_h = box_h
        self.box_w = box_w
        self.diagonals = diagonals
        self.all = (1 << n) - 1
        rows = [tuple(r * n + c for c in range(n)) for r in range(n)]
        cols = [tuple(r * n + c for r in range(n)) for c in range(n)]
        boxes = [tuple((br * box_h + r) * n + bc * box_w + c
                       for r in range(box_h) for c in range(box_w))
                 for br in range(n // box_h) for bc in range(n // box_w)]
        units = rows + cols + boxes
        if diagonals:
            units.append(tuple(i * n + i for i in range(n)))
            units.append(tuple(i * n + (n - 1 - i) for i in range(n)))
        self.units = tuple(units)
        self.box_of = tuple((i // n) // box_h * (n // box_w) + (i % n) // box_w
                            for i in range(self.size))
        self.units_of = tuple(tuple(k for k, u in enumerate(units) if i in u)
                              for i in range(self.size))
        self.peers = tuple(tuple(sorted({j for k in self.units_of[i]
                                         for j in units[k] if j != i}))
                           for i in range(self.size))

    def on_diagonal(self, i):
        """Liegt Zelle i auf einer der beiden Diagonalen (nur X-Sudoku)?"""
        r, c = divmod(i, self.n)
        return self.diagonals and (r == c or r + c == self.n - 1)


_LAYOUTS = {}


def layout_for(variant):
    """Layout-Objekt je Variante (gecacht)."""
    lay = _LAYOUTS.get(variant)
    if lay is None:
        if variant == "mini":
            lay = Layout(6, 2, 3)
        elif variant == "x":
            lay = Layout(9, 3, 3, diagonals=True)
        else:                              # classic, killer
            lay = Layout(9, 3, 3)
        _LAYOUTS[variant] = lay
    return lay


def fill_grid(lay, rng):
    """Volle Lösung per randomisiertem Backtracking.

    Gewählt wird immer die leere Zelle mit den wenigsten Kandidaten (bei
    Gleichstand die mit dem kleinsten Index) - so verrennt sich die Suche
    auch bei X-Sudoku nicht. rng ist ein ``seedrand.Rand``; die JS-Fassung
    verbraucht die Zufallszahlen in exakt derselben Reihenfolge.
    """
    n, size = lay.n, lay.size
    ALL = lay.all
    board = [0] * size
    umask = [ALL] * len(lay.units)
    uof = lay.units_of

    def solve(left):
        if not left:
            return True
        best_i, best_c, best_n = -1, 0, 99
        for i in range(size):
            if board[i]:
                continue
            c = ALL
            for u in uof[i]:
                c &= umask[u]
            k = c.bit_count()
            if k < best_n:
                best_i, best_c, best_n = i, c, k
                if k <= 1:
                    break
        if not best_c:
            return False
        digits = [d for d in range(1, n + 1) if best_c >> (d - 1) & 1]
        rng.shuffle(digits)
        us = uof[best_i]
        for d in digits:
            m = 1 << (d - 1)
            board[best_i] = d
            for u in us:
                umask[u] ^= m
            if solve(left - 1):
                return True
            for u in us:
                umask[u] ^= m
            board[best_i] = 0
        return False

    solve(size)
    return board


# ----- Killer-Käfige: welche Ziffern können in einem Käfig noch stehen? -------
_COMBO = None


def _combo_table():
    """Tabelle [frei][anzahl][summe] -> Bitmaske möglicher Ziffern.

    Eintrag = Vereinigung aller Teilmengen S der freien Ziffern mit genau
    'anzahl' Elementen und Summe 'summe'. Einmalig aufgebaut (3^9 Paare).
    Index: (frei * 10 + anzahl) * 46 + summe.
    """
    global _COMBO
    if _COMBO is None:
        tab = [0] * (512 * 10 * 46)
        for s in range(512):
            k = s.bit_count()
            total = sum(d + 1 for d in range(9) if s >> d & 1)
            rest = 511 & ~s
            sub = rest
            while True:
                tab[((s | sub) * 10 + k) * 46 + total] |= s
                if sub == 0:
                    break
                sub = (sub - 1) & rest
        _COMBO = tab
    return _COMBO


def cage_digits(free, cells_left, rest):
    """Ziffern (Bitmaske), die in einem Käfig mit 'cells_left' leeren Zellen,
    noch freien Ziffern 'free' und Restsumme 'rest' vorkommen können."""
    if cells_left <= 0 or not 0 <= rest <= 45:
        return 0
    return _combo_table()[(free * 10 + cells_left) * 46 + rest]


def count_solutions(lay, puzzle, limit=2, cages=None, solutions=None):
    """Zählt Lösungen (MRV, Abbruch bei ``limit``) für beliebige Layouts.

    cages     : optionale Killer-Käfige [(summe, (zellen...)), ...]
    solutions : optionale Liste - gefundene Lösungen werden angehängt
    Das Ergebnis ist min(echte Anzahl, limit) - unabhängig von der
    Suchreihenfolge, deshalb darf die JS-Fassung anders suchen.
    """
    size = lay.size
    ALL = lay.all
    board = list(puzzle)
    uof = lay.units_of
    umask = [ALL] * len(lay.units)
    for i in range(size):
        d = board[i]
        if d:
            m = 1 << (d - 1)
            for u in uof[i]:
                if not umask[u] & m:
                    return 0              # Vorgaben widersprechen sich
                umask[u] ^= m

    cage_of = None
    combo = None
    cfree, crem, cleft = [], [], []
    if cages:
        combo = _combo_table()
        cage_of = [-1] * size
        for ci, (total, cells) in enumerate(cages):
            free, rem, left = ALL, int(total), 0
            for i in cells:
                cage_of[i] = ci
                d = board[i]
                if d:
                    m = 1 << (d - 1)
                    if not free & m:
                        return 0
                    free ^= m
                    rem -= d
                else:
                    left += 1
            if rem < 0 or (left == 0 and rem != 0) or rem > 45:
                return 0
            cfree.append(free)
            crem.append(rem)
            cleft.append(left)

    empty = [i for i in range(size) if not board[i]]
    count = 0

    def solve():
        nonlocal count
        best_i, best_c, best_n = -1, 0, 99
        for i in empty:
            if board[i]:
                continue
            c = ALL
            for u in uof[i]:
                c &= umask[u]
            if cage_of is not None:
                ci = cage_of[i]
                if ci >= 0:
                    c &= combo[(cfree[ci] * 10 + cleft[ci]) * 46 + crem[ci]]
            if not c:
                return
            k = c.bit_count()
            if k < best_n:
                best_i, best_c, best_n = i, c, k
                if k == 1:
                    break
        if best_i < 0:                    # keine leere Zelle mehr -> Lösung
            count += 1
            if solutions is not None:
                solutions.append(list(board))
            return
        us = uof[best_i]
        ci = cage_of[best_i] if cage_of is not None else -1
        c = best_c
        while c:
            m = c & -c
            c ^= m
            d = m.bit_length()
            board[best_i] = d
            for u in us:
                umask[u] ^= m
            if ci >= 0:
                cfree[ci] ^= m
                crem[ci] -= d
                cleft[ci] -= 1
            solve()
            board[best_i] = 0
            for u in us:
                umask[u] ^= m
            if ci >= 0:
                cfree[ci] ^= m
                crem[ci] += d
                cleft[ci] += 1
            if count >= limit:
                return

    solve()
    return count


def killer_groups(cages):
    """Summen-Gruppen eines Killer-Sudokus: alle Käfige plus die "45er-Regel".

    Eine Zeile/Spalte/Block enthält 1..9 (Summe 45). Liegen Käfige komplett
    darin, müssen die übrigen Zellen der Einheit zusammen 45 minus deren
    Summen ergeben - eine zusätzliche, unsichtbare Summen-Gruppe.
    Rückgabe: [(summe, (zellen...)), ...], Käfige zuerst.
    """
    groups = [(int(total), tuple(cells)) for total, cells in cages]
    for unit in layout_for("killer").units:
        uset = set(unit)
        inside = [(total, cells) for total, cells in groups[:len(cages)]
                  if uset.issuperset(cells)]
        if not inside:
            continue
        covered = {i for _, cells in inside for i in cells}
        rest = tuple(i for i in unit if i not in covered)
        if rest:
            groups.append((45 - sum(total for total, _ in inside), rest))
    return groups


def count_killer(puzzle, cages, limit=2, solutions=None, budget=0):
    """Zählt Lösungen eines Killer-Sudokus (9x9) - mit Propagation.

    Anders als ``count_solutions`` zieht dieser Löser an jedem Knoten alle
    Folgerungen nach, bevor er verzweigt: Naked/Hidden Singles in Zeilen,
    Spalten und Blöcken, Ziffer-Kombinationen je Summen-Gruppe (Käfige +
    45er-Regel, siehe ``killer_groups``) und Ziffern, die in jeder möglichen
    Kombination einer Gruppe vorkommen müssen. Dadurch sind auch Rätsel ganz
    ohne Vorgaben meist in Sekundenbruchteilen bewiesen.

    budget > 0: höchstens so viele Verzweigungs-Knoten; wird es überschritten,
    kommt -1 zurück ("unbekannt"). Sonst min(echte Anzahl, limit).
    """
    lay = layout_for("killer")
    ALL = lay.all
    combo = _combo_table()
    peers = lay.peers
    units = lay.units
    groups = killer_groups(cages)
    if sorted(i for _, cells in cages for i in cells) != list(range(81)):
        return 0
    gcells = [cells for _, cells in groups]
    groups_of = [[] for _ in range(81)]
    for g, cells in enumerate(gcells):
        for i in cells:
            groups_of[i].append(g)
    groups_of = [tuple(gs) for gs in groups_of]
    board = [0] * 81
    cand = [ALL] * 81
    gfree = [ALL] * len(groups)
    grem = [total for total, _ in groups]
    gleft = [len(cells) for cells in gcells]
    for g, (total, cells) in enumerate(groups):
        allow = combo[(ALL * 10 + len(cells)) * 46 + total] \
            if 0 < total <= 45 else 0
        for i in cells:
            cand[i] &= allow

    def place(st, i, d):
        """Setzt Ziffer d in Zelle i und schränkt Nachbarn/Gruppen ein."""
        bd, cd, fr, rm, lf = st
        m = 1 << (d - 1)
        if not cd[i] & m:
            return False
        bd[i] = d
        cd[i] = 0
        for j in peers[i]:
            if bd[j] == d:
                return False
            cd[j] &= ~m
        for g in groups_of[i]:
            if not fr[g] & m:
                return False
            fr[g] ^= m
            rm[g] -= d
            lf[g] -= 1
            if lf[g]:
                if not 0 < rm[g] <= 45:
                    return False
                allow = combo[(fr[g] * 10 + lf[g]) * 46 + rm[g]]
                if not allow:
                    return False
                for j in gcells[g]:
                    if not bd[j]:
                        cd[j] &= allow
            elif rm[g]:
                return False
        return True

    st0 = (board, cand, gfree, grem, gleft)
    for i, d in enumerate(puzzle):
        if d and not place(st0, i, d):
            return 0

    def hidden(st, cells, need, once, twice):
        """Setzt Ziffern aus 'need', die in 'cells' nur einen Platz haben.
        Rückgabe: None bei Widerspruch, sonst ob etwas gesetzt wurde."""
        bd, cd = st[0], st[1]
        single = once & ~twice & need
        while single:
            m = single & -single
            single ^= m
            for i in cells:
                if not bd[i] and cd[i] & m:
                    if not place(st, i, m.bit_length()):
                        return None
                    break
        return bool(once & ~twice & need)

    def propagate(st):
        bd, cd, fr, rm, lf = st
        while True:
            changed = False
            # Naked Singles
            for i in range(81):
                if bd[i]:
                    continue
                c = cd[i]
                if not c:
                    return False
                if not c & (c - 1):
                    if not place(st, i, c.bit_length()):
                        return False
                    changed = True
            # Hidden Singles in Zeilen/Spalten/Blöcken (alle 9 Ziffern nötig)
            for cells in units:
                once = twice = have = 0
                for i in cells:
                    if bd[i]:
                        have |= 1 << (bd[i] - 1)
                    else:
                        c = cd[i]
                        twice |= once & c
                        once |= c
                need = ALL & ~have
                if need & ~once:
                    return False
                got = hidden(st, cells, need, once, twice)
                if got is None:
                    return False
                changed = changed or got
            # Summen-Gruppen: Ziffern, die in JEDER passenden Kombination
            # vorkommen, müssen hinein - hat so eine nur einen Platz, steht sie.
            for g, cells in enumerate(gcells):
                if lf[g] < 2:
                    continue
                need = _cage_must(fr[g], lf[g], rm[g])
                if not need:
                    continue
                once = twice = 0
                for i in cells:
                    if not bd[i]:
                        c = cd[i]
                        twice |= once & c
                        once |= c
                if need & ~once:
                    return False
                got = hidden(st, cells, need, once, twice)
                if got is None:
                    return False
                changed = changed or got
            if not changed:
                return True

    count = 0
    nodes = 0

    def search(st):
        nonlocal count, nodes
        if not propagate(st):
            return
        bd, cd = st[0], st[1]
        best_i, best_n = -1, 99
        for i in range(81):
            if not bd[i]:
                k = cd[i].bit_count()
                if k < best_n:
                    best_i, best_n = i, k
                    if k == 2:
                        break
        if best_i < 0:
            count += 1
            if solutions is not None:
                solutions.append(list(bd))
            return
        nodes += 1
        if budget and nodes > budget:
            return
        c = cd[best_i]
        while c:
            m = c & -c
            c ^= m
            child = (list(bd), list(cd), list(st[2]), list(st[3]), list(st[4]))
            if place(child, best_i, m.bit_length()):
                search(child)
            if count >= limit or (budget and nodes > budget):
                return

    search(st0)
    if budget and nodes > budget and count < limit:
        return -1
    return count


_MUST = {}


def _cage_must(free, left, rest):
    """Ziffern, die in JEDER Kombination (left Ziffern aus free, Summe rest)
    vorkommen - Ergebnis gecacht."""
    key = (free, left, rest)
    got = _MUST.get(key)
    if got is None:
        got = 511
        any_combo = False
        sub = free
        while sub:
            if sub.bit_count() == left and \
                    sum(d + 1 for d in range(9) if sub >> d & 1) == rest:
                got &= sub
                any_combo = True
            sub = (sub - 1) & free
        got = got if any_combo else 0
        _MUST[key] = got
    return got


def solve_singles(lay, puzzle):
    """Löst nur mit Singles (Naked + Hidden Single). Lösung oder None.

    Kommt das Verfahren bis zum Ende, ist das Rätsel automatisch eindeutig
    lösbar - und ohne Raten machbar (Maßstab für Leicht/Normal).
    """
    size = lay.size
    ALL = lay.all
    board = list(puzzle)
    peers = lay.peers
    cand = [0] * size
    for i in range(size):
        d = board[i]
        if d:
            for j in peers[i]:
                if board[j] == d:
                    return None
            continue
        used = 0
        for j in peers[i]:
            if board[j]:
                used |= 1 << (board[j] - 1)
        cand[i] = ALL & ~used

    left = board.count(0)
    while left:
        placed = 0
        # Naked Singles: nur noch eine Ziffer passt in die Zelle
        for i in range(size):
            if board[i]:
                continue
            c = cand[i]
            if not c:
                return None
            if not c & (c - 1):
                board[i] = c.bit_length()
                cand[i] = 0
                for j in peers[i]:
                    cand[j] &= ~c
                placed += 1
        # Hidden Singles: eine Ziffer hat in einer Einheit nur einen Platz
        for cells in lay.units:
            once = 0
            twice = 0
            have = 0
            for i in cells:
                if board[i]:
                    have |= 1 << (board[i] - 1)
                else:
                    c = cand[i]
                    twice |= once & c
                    once |= c
            if ALL & ~have & ~once:
                return None               # eine Ziffer passt nirgends mehr
            single = once & ~twice & ~have
            while single:
                m = single & -single
                single ^= m
                for i in cells:
                    if not board[i] and cand[i] & m:
                        board[i] = m.bit_length()
                        cand[i] = 0
                        for j in peers[i]:
                            cand[j] &= ~m
                        placed += 1
                        break
        if not placed:
            return None
        left -= placed
    return board


def dig_variant(lay, solution, target, rule, rng):
    """Leert Zellen in rng-Reihenfolge, solange das Rätsel lösbar bleibt.

    rule: "singles" / "unique" / "hard" (siehe DIG_RULES). Ein einziger
    Durchlauf über alle Zellen - die Arbeit ist gedeckelt und je Seed
    deterministisch.
    """
    puzzle = list(solution)
    order = list(range(lay.size))
    rng.shuffle(order)
    clues = lay.size
    easy = True           # geht das Rätsel (noch) allein mit Singles auf?
    for i in order:
        if clues <= target:
            if rule != "hard":
                break
            # Experte: erst aufhören, wenn Singles nicht mehr reichen.
            # (Weniger Vorgaben machen es nie wieder "einfach".)
            if easy:
                easy = solve_singles(lay, puzzle) is not None
            if not easy:
                break
        saved = puzzle[i]
        puzzle[i] = 0
        if rule == "singles":
            ok = solve_singles(lay, puzzle) is not None
        else:
            ok = count_solutions(lay, puzzle) == 1
        if ok:
            clues -= 1
        else:
            puzzle[i] = saved
    return puzzle


def variant_seed(variant, diff, level):
    """seedrand-Seed für Level ``level`` der Stufe ``diff`` einer Variante."""
    return seedrand.seed_from("sudoku", variant, int(diff), int(level))


def generate_variant(variant, diff, level):
    """(puzzle, solution) für X-Sudoku ("x") oder Mini 6x6 ("mini")."""
    lay = layout_for(variant)
    clues = VARIANT_CLUES.get(variant, VARIANT_CLUES["classic"])
    diff = max(0, min(3, int(diff)))
    rng = seedrand.Rand(variant_seed(variant, diff, level))
    solution = fill_grid(lay, rng)
    puzzle = dig_variant(lay, solution, clues[diff], DIG_RULES[diff], rng)
    return puzzle, solution


def daily_diff(date=None):
    """Stufe des Tages-Sudokus (hängt vom Wochentag ab)."""
    import datetime
    day = datetime.date.fromisoformat(date or seedrand.today_str())
    return DAILY_DIFF[day.weekday()]


def generate_daily(date=None):
    """(puzzle, solution, diff) des Tages-Sudokus (klassisch 9x9)."""
    date = date or seedrand.today_str()
    diff = daily_diff(date)
    lay = layout_for("classic")
    rng = seedrand.Rand(seedrand.daily_seed("sudoku", date))
    solution = fill_grid(lay, rng)
    puzzle = dig_variant(lay, solution, VARIANT_CLUES["classic"][diff],
                         DIG_RULES[diff], rng)
    return puzzle, solution, diff


# ----- Killer-Sudoku: vorab erzeugte Level --------------------------------------
KILLER_FILE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                           "levels", "sudoku-killer.json")
_killer = None


def _parse_killer_level(raw):
    """Ein Level aus der JSON prüfen -> (puzzle, solution, cages) oder None."""
    try:
        sol = [int(ch) for ch in raw["s"]]
        puz = [int(ch) for ch in raw["g"]]
        cages = [(int(total), tuple(int(i) for i in cells))
                 for total, cells in raw["c"]]
    except (KeyError, TypeError, ValueError):
        return None
    if len(sol) != 81 or len(puz) != 81 or not cages:
        return None
    if sorted(i for _, cells in cages for i in cells) != list(range(81)):
        return None
    if any(p and p != s for p, s in zip(puz, sol)):
        return None
    if any(sum(sol[i] for i in cells) != total for total, cells in cages):
        return None
    return puz, sol, cages


def load_killer():
    """Liest games/levels/sudoku-killer.json (gecacht): {stufe: [level...]}."""
    global _killer
    if _killer is None:
        out = {d: [] for d in range(4)}
        try:
            with open(KILLER_FILE, "r", encoding="utf-8") as f:
                data = json.load(f)
            for d in range(4):
                for raw in (data.get("levels") or {}).get(str(d), []):
                    lvl = _parse_killer_level(raw)
                    if lvl is not None:
                        out[d].append(lvl)
        except (OSError, ValueError, AttributeError):
            pass                         # Datei fehlt/kaputt -> keine Killer-Level
        _killer = out
    return _killer


def killer_level(diff, level):
    """(puzzle, solution, cages) für Killer-Level ``level`` (1..100) oder None."""
    levels = load_killer().get(max(0, min(3, int(diff))), [])
    if 1 <= int(level) <= len(levels):
        return levels[int(level) - 1]
    return None
