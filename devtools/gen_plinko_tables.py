# -*- coding: utf-8 -*-
"""
gen_plinko_tables.py
====================
Erzeugt EINMAL die 27 Multiplikator-Tabellen (x100) von Lama Plinko und gibt
sie als Python-Quelltext aus (zum Einfügen in ``games/lamaplinko_logic.py``).
Die Tabellen stehen danach fest im Code - dieses Skript läuft im Spiel nie.

    python devtools/gen_plinko_tables.py            # Tabellen + Quoten zeigen
    python devtools/gen_plinko_tables.py --check    # mit lamaplinko_logic vergleichen

Verfahren je Risiko r und Reihenzahl n (8 .. 16), Fach i = 0 .. n:

1. **Rand** E(n): geometrisch zwischen E(8) und E(16) eingepasst
   (Niedrig 5 -> 16, Mittel 12 -> 110, Hoch 30 -> 1000) und auf eine
   "schöne" Zahl gerundet.
2. **Mitte** c: Niedrig x0.7, Mittel x0.4, Hoch x0.2.
3. **Form**: Abstand zur Mitte x = |i - n/2| / (n/2) in [0, 1],
   Zielwert t(x) = c * (E/c) ** (x ** p). Der Exponent p wird per
   Bisektion so gewählt, dass die Quote sum C(n,i)/2^n * t_i genau 97 % ist.
4. **Runden** auf die schöne Leiter: unter x10 eine Nachkommastelle
   (x100-Werte durch 10 teilbar), x10-x99 ganze Zahlen, ab x100 volle Zehner.
5. **Feinabgleich** (gierig, deterministisch): Ein oder zwei symmetrische
   Fächerpaare gehen je eine Leiterstufe hoch oder runter (die Mitte bleibt
   fest, der Rand nur bei 16 Reihen), solange die Tabelle von der Mitte
   nach außen nicht fällt. Genommen wird der Schritt mit den kleinsten Kosten
   ``1000 * max(0, |Q - 0,97| - 0,0015) + 10 * |Q - 0,97| + Formabweichung``
   (Formabweichung = Summe der quadrierten log-Abstände zum Zielwert,
   gewichtet mit der Fach-Wahrscheinlichkeit + 1/n), bis keiner mehr hilft.

Ergebnis: symmetrisch, nach außen nicht fallend, exakte Quote (Fraction)
zwischen 96,5 % und 97,5 % - geprüft von ``tests/audit_chiplogic.py``.
"""

import math
import os
import sys
from fractions import Fraction

ROWS = tuple(range(8, 17))
RISKS = ("low", "medium", "high")
EDGE = {"low": (5, 16), "medium": (12, 110), "high": (30, 1000)}
CENTER = {"low": 0.7, "medium": 0.4, "high": 0.2}
TARGET = 0.97


def ladder():
    """Alle schönen Werte x100, aufsteigend."""
    out = list(range(10, 1000, 10))          # x0.1 .. x9.9
    out += list(range(1000, 10000, 100))     # x10 .. x99
    out += list(range(10000, 200001, 1000))  # x100 .. x2000
    return out


LADDER = ladder()


def nice(v100):
    """Nächster schöner Wert (log-Abstand) zu einem x100-Wert."""
    return min(LADDER, key=lambda m: (abs(math.log(m / v100)), m))


def probs(n):
    return [math.comb(n, i) / 2 ** n for i in range(n + 1)]


def rtp_exact(n, table):
    return sum(Fraction(math.comb(n, i), 2 ** n) * Fraction(m, 100) for i, m in enumerate(table))


def edge(risk, n):
    e8, e16 = EDGE[risk]
    return nice(100 * e8 * (e16 / e8) ** ((n - 8) / 8))


def shape(n, c100, e100, p):
    half = n / 2
    return [c100 * (e100 / c100) ** ((abs(i - half) / half) ** p) for i in range(n + 1)]


def solve_p(n, c100, e100):
    pr = probs(n)
    lo, hi = 0.05, 20.0
    for _ in range(200):
        mid = (lo + hi) / 2
        q = sum(a * b for a, b in zip(pr, shape(n, c100, e100, mid))) / 100
        # größeres p -> flachere Mitte -> kleinere Quote
        if q > TARGET:
            lo = mid
        else:
            hi = mid
    return (lo + hi) / 2


def monotone(n, table):
    half = n // 2
    if any(table[i] != table[n - i] for i in range(n + 1)):
        return False
    # von der Mitte nach außen nicht fallend
    right = table[half:] if n % 2 == 0 else table[half + 1:]
    return all(a <= b for a, b in zip(right, right[1:]))


def cost(n, table, target, pr):
    q = float(rtp_exact(n, table))
    err = abs(q - TARGET)
    dev = sum((pr[i] + 1 / n) * math.log(table[i] / target[i]) ** 2 for i in range(n + 1))
    return 1000 * max(0.0, err - 0.0015) + 10 * err + dev


def moves(n, table, pairs):
    """Alle Tabellen, die aus einem oder zwei Leiterschritten entstehen."""
    single = []
    for i in pairs:
        idx = LADDER.index(table[i])
        for step in (-1, 1):
            if 0 <= idx + step < len(LADDER):
                single.append((i, step))
    out = []
    for a in range(len(single)):
        for b in range(a, len(single) + 1):
            t2 = list(table)
            for i, step in [single[a]] + ([single[b]] if b < len(single) else []):
                t2[i] = t2[n - i] = LADDER[LADDER.index(t2[i]) + step]                     if 0 <= LADDER.index(t2[i]) + step < len(LADDER) else t2[i]
            if t2 != table and monotone(n, t2):
                out.append(t2)
    return out


def build(risk, n):
    c100 = CENTER[risk] * 100
    e100 = edge(risk, n)
    p = solve_p(n, c100, e100)
    target = shape(n, c100, e100, p)
    table = [nice(v) for v in target]
    table[0] = table[n] = e100
    pr = probs(n)
    # Paare (i, n-i) ohne die Mitte; der Rand bleibt nur bei 16 Reihen fest
    pairs = list(range(0 if n < 16 else 1, (n - 1) // 2 if n % 2 else n // 2))
    best = cost(n, table, target, pr)
    while True:
        cand = None
        for t2 in moves(n, table, pairs):
            c = cost(n, t2, target, pr)
            if c < best - 1e-12 and (cand is None or c < cand[0]):
                cand = (c, t2)
        if cand is None:
            break
        best, table = cand
    return tuple(table), p


def main():
    tables = {}
    print("# Quoten (exakt) und Formexponent p:")
    for risk in RISKS:
        tables[risk] = {}
        for n in ROWS:
            t, p = build(risk, n)
            tables[risk][n] = t
            q = rtp_exact(n, t)
            ok = monotone(n, list(t)) and Fraction(965, 1000) <= q <= Fraction(975, 1000)
            print("#   %-6s %2d  Quote %.4f %%  p=%.3f  Rand x%s  %s"
                  % (risk, n, float(q) * 100, p, t[0] / 100, "ok" if ok else "FEHLER"))
    print()
    print("TABLES = {")
    for risk in RISKS:
        print('    "%s": {' % risk)
        for n in ROWS:
            print("        %d: %s," % (n, "(" + ", ".join(str(v) for v in tables[risk][n]) + ")"))
        print("    },")
    print("}")
    if "--check" in sys.argv:
        import importlib.util
        repo = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
        sys.path.insert(0, repo)
        # direkt laden (games/__init__.py zöge alle Spiele samt pygame nach)
        spec = importlib.util.spec_from_file_location(
            "lamaplinko_logic", os.path.join(repo, "games", "lamaplinko_logic.py"))
        L = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(L)
        same = all(L.TABLES[r][n] == tables[r][n] for r in RISKS for n in ROWS)
        print("\n# lamaplinko_logic.TABLES %s" % ("stimmt überein" if same else "WEICHT AB"))
        return 0 if same else 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
