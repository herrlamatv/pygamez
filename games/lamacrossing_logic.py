# -*- coding: utf-8 -*-
"""
lamacrossing_logic.py
=====================
Regeln von Lama Crossing - ohne pygame, damit Web (``lamacrossing_logic.js``)
und Rust (``lamacrossing_logic.rs``) dieselben Tabellen 1:1 übernehmen.

Prinzip: Das Lama überquert Schritt für Schritt eine Schlucht, einen Fluss
oder einen Eissee. Jeder Schritt hält mit Wahrscheinlichkeit ``1 - p`` und
hebt den Multiplikator; ein Fehltritt kostet den Einsatz. Auszahlen geht nach
jedem geschafften Schritt.

Multiplikatoren stehen als **Ganzzahlen in Hundertsteln** fest in den
Tabellen (keine Gleitkomma-Rechnung zur Laufzeit, in allen drei Sprachen
bitgleich). Erzeugt wurden sie einmal mit

    m_k = floor(0,97 / (1 - p)^k * 100)

Jede Strategie "bei Schritt k auszahlen" hat damit den Erwartungswert
``(1 - p)^k * m_k / 100`` ≈ 0,97 (Auszahlungsquote ~97 %, siehe
``expected_return``).
"""

DIFFS = ("easy", "medium", "hard", "extreme")
WORLDS = ("bridge", "river", "ice")

# Risiko je Schritt als Bruch (Zähler, Nenner) - zum Ziehen wird daraus EINE
# Gleitkommazahl (in Python, JS und Rust dieselbe).
RISK = {
    "easy": (1, 12),
    "medium": (1, 7),
    "hard": (1, 4),
    "extreme": (2, 5),
}

# Multiplikator (x100) nach k geschafften Schritten, k = 1 .. Anzahl Schritte.
TABLES = {
    "easy": (105, 115, 125, 137, 149, 163, 178, 194, 212, 231, 252, 275, 300,
             327, 357, 390, 425, 464, 506, 552),
    "medium": (113, 132, 154, 179, 209, 244, 285, 332, 388, 453, 528, 616,
               719, 839, 979, 1142),
    "hard": (129, 172, 229, 306, 408, 545, 726, 968, 1291, 1722, 2296, 3062,
             4082),
    "extreme": (161, 269, 449, 748, 1247, 2079, 3465, 5775, 9625, 16042),
}

# Einsatzstufen (Mindesteinsatz = erste Stufe, siehe lamabank.MIN_BET)
BETS = (10, 20, 50, 100, 200, 500, 1000, 2500, 5000, 10000)

# Ab diesem Multiplikator (x100) gibt es den Erfolg "Nervenstark"
BRAVE_MULT = 1000


def steps(diff):
    """Anzahl der Schritte bis zum anderen Ufer."""
    return len(TABLES[diff])


def risk(diff):
    """Fehltritt-Wahrscheinlichkeit je Schritt als Gleitkommazahl."""
    num, den = RISK[diff]
    return num / den


def mult(diff, k):
    """Multiplikator (x100) nach k geschafften Schritten (k = 0 -> 100)."""
    if k <= 0:
        return 100
    table = TABLES[diff]
    return table[min(k, len(table)) - 1]


def payout(bet, diff, k):
    """Auszahlung bei k geschafften Schritten (abgerundet auf ganze Chips)."""
    return bet * mult(diff, k) // 100


def roll(rng, diff):
    """Ein Schritt: True = hält, False = Fehltritt (genau ein random()-Aufruf)."""
    return not rng.random() < risk(diff)


def next_break(rng, diff, k):
    """Nach dem Auszahlen bei Schritt k: wo wäre es schiefgegangen?

    Zieht die restlichen Schritte k+1 .. Ende (bricht beim ersten Fehltritt
    ab) und liefert dessen Nummer - oder None, wenn alles gehalten hätte.
    """
    for i in range(k + 1, steps(diff) + 1):
        if not roll(rng, diff):
            return i
    return None


def risk_percent(diff):
    """Risiko je Schritt in ganzen Prozent (gerundet, für die Anzeige)."""
    num, den = RISK[diff]
    return (num * 200 + den) // (den * 2)


def fmt_mult(m100, sep="."):
    """Multiplikator x100 als Text: 235 -> '2.35' (Trennzeichen je Sprache)."""
    return f"{m100 // 100}{sep}{m100 % 100:02d}"


def expected_return(diff, k):
    """Erwartungswert (Anteil am Einsatz) der Strategie 'bei k auszahlen'."""
    num, den = RISK[diff]
    hold = 1.0
    for _ in range(k):
        hold *= (den - num) / den
    return hold * mult(diff, k) / 100.0


def clean_round(raw):
    """Gespeicherte, noch laufende Runde prüfen (aus lamabank-Extra).

    Gültig: {"bet": >0, "diff": DIFFS, "step": 1 .. Schritte-1}. Sonst None.
    """
    if not isinstance(raw, dict):
        return None
    bet, diff, step = raw.get("bet"), raw.get("diff"), raw.get("step")
    if diff not in DIFFS:
        return None
    for v in (bet, step):
        if isinstance(v, bool) or not isinstance(v, int):
            return None
    if bet <= 0 or not 1 <= step < steps(diff):
        return None
    return {"bet": bet, "diff": diff, "step": step}


def clean_history(raw):
    """Verlauf der letzten Runden: Liste von Multiplikatoren x100 (0 = verloren)."""
    if not isinstance(raw, list):
        return []
    out = [v for v in raw if isinstance(v, int) and not isinstance(v, bool) and v >= 0]
    return out[-HISTORY:]


HISTORY = 12
