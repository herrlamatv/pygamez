# -*- coding: utf-8 -*-
"""
lamaplinko_logic.py
===================
Regeln von Lama Plinko - ohne pygame, damit Web (``lamaplinko_logic.js``) und
Rust (``lamaplinko_logic.rs``) dieselben Tabellen 1:1 übernehmen.

Prinzip: Ein goldenes Wollknäuel fällt durch ``rows`` Reihen von Stiften und
springt an jedem Stift nach links oder rechts (je 50 %). Das Fach unten ist
die Zahl der Rechts-Sprünge (0 .. rows); jedes Fach hat einen festen
Multiplikator. Wahrscheinlichkeit von Fach i: C(rows, i) / 2^rows.

Die 27 Tabellen (3 Risiken x 9 Reihenzahlen) stehen als **Ganzzahlen in
Hundertsteln** fest im Code, erzeugt EINMAL mit ``devtools/gen_plinko_tables.py``
(Verfahren dort: Rand geometrisch eingepasst, Form c * (E/c) ** (x ** p) mit
p für genau 97 %, Rundung auf schöne Werte, gieriger Feinabgleich).
Eigenschaften (geprüft von ``tests/audit_chiplogic.py``): symmetrisch, von der
Mitte nach außen nicht fallend, exakte Quote zwischen 96,5 % und 97,5 %
(alle 96,9-97,1 %), Rand bei 16 Reihen x16 / x110 / x1000.

Der Pfad einer Kugel kommt aus dem Fairness-Zufall
(``fairness.round_rand(KEY, server, client, nonce)``): ``rows`` Würfe
``random() < 0.5`` = rechts. Jede Kugel ist eine eigene Fairness-Runde.
"""

from fractions import Fraction

import fairness

KEY = "lamaplinko"

ROWS = tuple(range(8, 17))
DEFAULT_ROWS = 12
RISKS = ("low", "medium", "high")
DEFAULT_RISK = "medium"
# Einsatz je Kugel (Mindesteinsatz = erste Stufe, siehe lamabank.MIN_BET)
BETS = (1, 2, 5, 10, 20, 50, 100, 200, 500, 1000)
# höchstens so viele Kugeln gleichzeitig unterwegs
MAX_BALLS = 50
# Länge des Verlaufs
HISTORY = 12
# Erfolge: "Kante" (äußerstes Fach) zählt ab so vielen Reihen
EDGE_ROWS = 12
# "Regen": so viele Kugeln gleichzeitig unterwegs
RAIN_BALLS = 25

# Multiplikator x100 je Fach 0 .. rows (erzeugt mit devtools/gen_plinko_tables.py)
TABLES = {
    "low": {
        8: (530, 200, 120, 80, 70, 80, 120, 200, 530),
        9: (590, 260, 130, 100, 70, 70, 100, 130, 260, 590),
        10: (640, 300, 160, 110, 80, 70, 80, 110, 160, 300, 640),
        11: (770, 370, 200, 130, 90, 70, 70, 90, 130, 200, 370, 770),
        12: (880, 440, 250, 140, 100, 80, 70, 80, 100, 140, 250, 440, 880),
        13: (950, 480, 290, 160, 120, 90, 70, 70, 90, 120, 160, 290, 480, 950),
        14: (1200, 640, 340, 200, 120, 100, 80, 70, 80, 100, 120, 200, 340, 640, 1200),
        15: (1400, 700, 410, 230, 160, 120, 80, 70, 70, 80, 120, 160, 230, 410, 700, 1400),
        16: (1600, 860, 490, 280, 180, 130, 100, 70, 70, 70, 100, 130, 180, 280, 490, 860, 1600),
    },
    "medium": {
        8: (1300, 340, 130, 60, 40, 60, 130, 340, 1300),
        9: (1800, 470, 160, 80, 50, 50, 80, 160, 470, 1800),
        10: (2100, 670, 250, 100, 60, 40, 60, 100, 250, 670, 2100),
        11: (3000, 880, 340, 140, 80, 40, 40, 80, 140, 340, 880, 3000),
        12: (3700, 1200, 460, 180, 90, 60, 40, 60, 90, 180, 460, 1200, 3700),
        13: (5000, 1600, 620, 250, 130, 70, 40, 40, 70, 130, 250, 620, 1600, 5000),
        14: (6300, 2200, 820, 340, 160, 90, 50, 40, 50, 90, 160, 340, 820, 2200, 6300),
        15: (8400, 3000, 1100, 460, 200, 110, 70, 40, 40, 70, 110, 200, 460, 1100, 3000, 8400),
        16: (11000, 3800, 1400, 620, 280, 140, 80, 50, 40, 50, 80, 140, 280, 620, 1400, 3800, 11000),
    },
    "high": {
        8: (3200, 470, 110, 30, 20, 30, 110, 470, 3200),
        9: (4700, 770, 180, 50, 20, 20, 50, 180, 770, 4700),
        10: (7100, 1200, 270, 80, 30, 20, 30, 80, 270, 1200, 7100),
        11: (12000, 2000, 420, 120, 40, 20, 20, 40, 120, 420, 2000, 12000),
        12: (18000, 2900, 660, 180, 60, 30, 20, 30, 60, 180, 660, 2900, 18000),
        13: (27000, 5000, 1000, 270, 90, 40, 20, 20, 40, 90, 270, 1000, 5000, 27000),
        14: (40000, 7500, 1600, 410, 130, 50, 30, 20, 30, 50, 130, 410, 1600, 7500, 40000),
        15: (66000, 12000, 2500, 620, 190, 70, 40, 20, 20, 40, 70, 190, 620, 2500, 12000, 66000),
        16: (100000, 19000, 3800, 970, 280, 90, 50, 30, 20, 30, 50, 90, 280, 970, 3800, 19000, 100000),
    },
}


def table(rows, risk):
    """Multiplikatoren x100 aller Fächer 0 .. rows."""
    return TABLES[risk][rows]


def mult100(rows, risk, slot):
    """Multiplikator x100 eines Fachs."""
    return TABLES[risk][rows][slot]


def payout(bet, rows, risk, slot):
    """Auszahlung einer Kugel (abgerundet auf ganze Chips)."""
    return bet * mult100(rows, risk, slot) // 100


def payout_carry(bet, rows, risk, slot, carry):
    """(Auszahlung, neuer Übertrag) mit Rest-Hundertstel-Übertrag.

    ``bet * m100 + carry`` in Hundertsteln; ganze Chips werden ausgezahlt, der
    Rest (0..99) bleibt für die nächste Kugel (Extra-Schlüssel "carry"). So
    bringen auch Einsätze 1/2/5 auf lange Sicht genau die Tabellen-Quote
    (``payout`` allein rundet je Kugel ab: Einsatz 1 x0.4 = 0 Chips).
    """
    total = bet * mult100(rows, risk, slot) + carry
    return total // 100, total % 100


def clean_carry(raw):
    """Gespeicherter Übertrag: ganze Zahl 0..99, sonst 0."""
    return raw if _is_int(raw) and 0 <= raw <= 99 else 0


def is_edge(rows, slot):
    """Äußerstes Fach (ganz links oder ganz rechts)?"""
    return slot == 0 or slot == rows


def drop_path(rand, rows):
    """Pfad einer Kugel: je Reihe 1 = rechts (random() < 0.5), sonst 0 = links."""
    return [1 if rand.random() < 0.5 else 0 for _ in range(rows)]


def slot_of(path):
    """Fach = Zahl der Rechts-Sprünge."""
    return sum(path)


def path_from_seed(rows, server, client, nonce):
    """Pfad einer Kugel aus ihren Fairness-Werten (zum Nachrechnen)."""
    return drop_path(fairness.round_rand(KEY, server, client, nonce), rows)


def comb(n, k):
    """Binomialkoeffizient C(n, k) (exakt, n <= 16)."""
    if k < 0 or k > n:
        return 0
    k = min(k, n - k)
    r = 1
    for i in range(1, k + 1):
        r = r * (n - k + i) // i
    return r


def prob(rows, slot):
    """Wahrscheinlichkeit eines Fachs (exakt): C(rows, slot) / 2^rows."""
    return Fraction(comb(rows, slot), 2 ** rows)


def expected_return(rows, risk):
    """Auszahlungsquote einer Tabelle (exakt, Anteil am Einsatz)."""
    return sum((prob(rows, i) * Fraction(m, 100) for i, m in enumerate(TABLES[risk][rows])),
               Fraction(0))


def fmt_prob(rows, slot, sep="."):
    """Wahrscheinlichkeit in Prozent als Text, kaufmännisch gerundet:
    ab 1 % zwei Nachkommastellen ('19.64'), darunter vier ('0.0015')."""
    num, den = comb(rows, slot), 2 ** rows
    if num * 100 >= den:
        v = (num * 20000 + den) // (den * 2)
        return f"{v // 100}{sep}{v % 100:02d}"
    v = (num * 2000000 + den) // (den * 2)
    return f"{v // 10000}{sep}{v % 10000:04d}"


def fmt_mult(m100, sep="."):
    """Multiplikator x100 als Text mit zwei Stellen: 20 -> '0.20'."""
    return f"{m100 // 100}{sep}{m100 % 100:02d}"


def fmt_short(m100, sep="."):
    """Multiplikator x100 kurz (für die Fächer): 20 -> '0.2', 1100 -> '11',
    125 -> '1.25'."""
    if m100 % 100 == 0:
        return str(m100 // 100)
    if m100 % 10 == 0:
        return f"{m100 // 100}{sep}{m100 % 100 // 10}"
    return fmt_mult(m100, sep)


def _is_int(v):
    return isinstance(v, int) and not isinstance(v, bool)


def blank_hits():
    """Leere Trefferzähler: {"8": [0]*9, ..., "16": [0]*17}."""
    return {str(r): [0] * (r + 1) for r in ROWS}


def clean_hits(raw):
    """Trefferzähler je Reihenzahl prüfen (Schlüssel "8".."16", je rows+1 Zahlen).

    Immer alle neun Schlüssel; eine Liste mit falscher Länge wird ganz auf 0
    gesetzt, einzelne ungültige Werte (kein int, bool, < 0) werden 0.
    """
    out = blank_hits()
    if not isinstance(raw, dict):
        return out
    for r in ROWS:
        v = raw.get(str(r))
        if isinstance(v, list) and len(v) == r + 1:
            out[str(r)] = [x if _is_int(x) and 0 <= x <= fairness.MAX_NONCE else 0 for x in v]
    return out


def add_hit(hits, rows, slot):
    """Einen Treffer zählen (hits aus ``clean_hits``)."""
    hits[str(rows)][slot] += 1
    return hits


def clean_history(raw):
    """Verlauf der letzten Kugeln: Liste von Multiplikatoren x100."""
    if not isinstance(raw, list):
        return []
    out = [v for v in raw if _is_int(v) and 0 <= v <= 1000000]
    return out[-HISTORY:]
