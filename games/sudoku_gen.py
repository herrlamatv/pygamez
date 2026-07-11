# -*- coding: utf-8 -*-
"""
sudoku_gen.py
=============
Deterministischer Sudoku-Generator + Löser (OHNE pygame-Import, damit die
Generierung headless testbar bleibt und sudoku.py schlank ist).

Level-System
------------
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

Alle Bretter sind flache Listen mit 81 Einträgen (Index = zeile*9+spalte,
0 = leer, 1..9 = Ziffer).
"""

import random

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
