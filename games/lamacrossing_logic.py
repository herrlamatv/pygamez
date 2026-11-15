# -*- coding: utf-8 -*-
"""
lamacrossing_logic.py
=====================
Regeln von Lama Crossing - ohne pygame, damit Web (``lamacrossing_logic.js``)
und Rust (``lamacrossing_logic.rs``) dieselben Tabellen 1:1 übernehmen.

Prinzip: Das Lama überquert Spur für Spur eine Straße, eine Schlucht, einen
Fluss oder einen Eissee. Jede Spur hält mit Wahrscheinlichkeit ``1 - p`` und
hebt den Multiplikator; ein Fehltritt kostet den Einsatz. Dazu kommen sichtbare
Hindernisse (Autos, Felsbrocken, Treibholz, Eisblöcke), denen man per Timing
ausweichen muss (``TRAFFIC``, ``hit_test``). Auszahlen geht nach jeder
geschafften Spur.

Multiplikatoren stehen als **Ganzzahlen in Hundertsteln** fest in den
Tabellen (keine Gleitkomma-Rechnung zur Laufzeit, in allen drei Sprachen
bitgleich). Erzeugt wurden sie einmal mit

    m_k = floor(0,97 / (1 - p)^k * 100)

Jede Strategie "bei Schritt k auszahlen" hat damit den Erwartungswert
``(1 - p)^k * m_k / 100`` ≈ 0,97 (Auszahlungsquote ~97 %, siehe
``expected_return``).
"""

DIFFS = ("easy", "medium", "hard", "extreme")
WORLDS = ("road", "bridge", "river", "ice")

# Risiko je Schritt als Bruch (Zähler, Nenner) - zum Ziehen wird daraus EINE
# Gleitkommazahl (in Python, JS und Rust dieselbe).
RISK = {
    "easy": (1, 12),
    "medium": (1, 6),
    "hard": (3, 10),
    "extreme": (1, 2),
}

# Multiplikator (x100) nach k geschafften Schritten, k = 1 .. Anzahl Schritte.
TABLES = {
    "easy": (105, 115, 125, 137, 149, 163, 178, 194, 212, 231, 252, 275, 300,
             327, 357, 390, 425, 464, 506, 552, 603, 657),
    "medium": (116, 139, 167, 201, 241, 289, 347, 417, 500, 600, 720, 864,
               1037, 1245, 1494, 1793, 2152, 2582, 3098, 3718),
    "hard": (138, 197, 282, 403, 577, 824, 1177, 1682, 2403, 3433, 4905, 7008,
             10011, 14302, 20431, 29187, 41697, 59567),
    "extreme": (194, 388, 776, 1552, 3104, 6208, 12416, 24832, 49664, 99328,
                198656, 397312, 794624, 1589248, 3178496),
}

# Verkehr je Stufe: (v_lo, v_hi) Tempo der Hindernisse in px/s bei 800x600
# (wird mit k skaliert), (gap_lo, gap_hi) Abstand zwischen zwei Hindernissen
# einer Spur in Sekunden. Jede Spur hat ein festes Tempo (kein Überholen).
TRAFFIC = {
    "easy": (240, 340, 1.8, 3.2),
    "medium": (300, 420, 1.4, 2.8),
    "hard": (360, 500, 1.1, 2.3),
    "extreme": (440, 620, 0.9, 1.9),
}

# Tempo-Faktor je Welt: Treibholz und Eisblöcke kommen vom Horizont her
# (kurzer Weg, daher langsamer), Autos und Felsbrocken von oben.
SPEED_F = {"road": 1.0, "bridge": 1.0, "river": 0.4, "ice": 0.4}

# Raser bei einem Zufalls-Fehltritt auf der Straße: v_hi * RASER_F
RASER_F = 1.6

# Einsatzstufen (Mindesteinsatz = erste Stufe, siehe lamabank.MIN_BET)
BETS = (10, 20, 50, 100, 200, 500, 1000, 2500, 5000, 10000)

# Ab diesem Multiplikator (x100) gibt es den Erfolg "Nervenstark"
BRAVE_MULT = 1000
# "Drahtseilakt": auf Extrem bei mindestens x100 ausgezahlt
EXTREME_MULT = 10000


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


def hit_test(obs_y, obs_len, target_y, reach):
    """Trifft ein Hindernis (Mitte obs_y, senkrechte Länge obs_len) das Lama,
    dessen Mitte auf target_y liegt (reach = halbe Trefferhöhe des Lamas)?"""
    return abs(obs_y - target_y) < obs_len / 2 + reach


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
