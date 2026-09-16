# -*- coding: utf-8 -*-
"""
casino_logic.py
===============
Spielregeln des Casinos ohne pygame - Roulette-Tisch und Lama-Slot.

Alles hier ist reine Logik (testbar, ohne Grafik) und 1:1 in
``web/js/games/casino_logic.js`` gespiegelt: gleiche Kesselreihenfolge, gleiche
Wettarten, gleiche Walzenstreifen und Gewinntabelle.

ROULETTE (europäisch, 37 Fächer 0-36)
    Einsätze werden als Schlüssel geführt, z.B. "plein:17", "cheval:17-20",
    "trans:1-2-3", "carre:1-2-4-5", "sixain:1-2-3-4-5-6", "column:0",
    "dozen:2", "red", "even", "low". Jede Wette deckt n Zahlen ab und zahlt
    36/n - 1 zu 1 (Plein 35:1, Cheval 17:1, Transversale 11:1, Carré 8:1,
    Sixain 5:1, Kolonne/Dutzend 2:1, einfache Chancen 1:1).

    Der Setztisch liegt waagerecht in "Tisch-Einheiten": die Null links
    (x 0..1), zwölf Zahlenspalten (x 1..13, oben 3/6/9..., unten 1/4/7...),
    die 2:1-Kolonnen rechts (x 13..14); darunter die Dutzende und die sechs
    einfachen Chancen. ``hit_test`` erkennt, ob ein Klick eine Zahl, eine
    Kante (Cheval) oder eine Ecke (Carré/Sixain) trifft.

LAMA-SLOT (5 Walzen x 3 Reihen, 10 Gewinnlinien)
    Lama = Wild (ersetzt alles außer der Münze), Goldmünze = Scatter
    (3+ irgendwo -> Scatter-Gewinn + 10 Freispiele, in denen alle Gewinne
    doppelt zählen; erneut auslösbar). Linien zahlen von links nach rechts ab
    3 gleichen Symbolen. Die Auszahlungsquote wird von ``exact_rtp`` EXAKT
    aus den Walzenstreifen berechnet (Ziel ~96 %).
"""

import itertools

# ============================================================= ROULETTE
# Echte Reihenfolge der Fächer auf dem europäischen Kessel (im Uhrzeigersinn).
WHEEL_ORDER = (0, 32, 15, 19, 4, 21, 2, 25, 17, 34, 6, 27, 13, 36, 11, 30, 8,
               23, 10, 5, 24, 16, 33, 1, 20, 14, 31, 9, 22, 18, 29, 7, 28, 12,
               35, 3, 26)
RED_NUMBERS = frozenset((1, 3, 5, 7, 9, 12, 14, 16, 18, 19, 21, 23, 25, 27,
                         30, 32, 34, 36))
CHIP_VALUES = (1, 5, 25, 100, 500)
HISTORY_LEN = 12

# Tisch-Geometrie in Einheiten (eine Zahlenzelle = 1 x 1)
TABLE_W = 14.0
DOZEN_H = 0.8
OUTSIDE_H = 0.8
TABLE_H = 3.0 + DOZEN_H + OUTSIDE_H
EDGE = 0.26            # so nah an einer Linie zählt ein Klick als Kante/Ecke
OUTSIDE_KEYS = ("low", "even", "red", "black", "odd", "high")

# Auszahlung "zu 1" je Wettart (Gewinn zusätzlich zum Einsatz)
PAYOUT = {"plein": 35, "cheval": 17, "trans": 11, "carre": 8, "sixain": 5,
          "column": 2, "dozen": 2, "red": 1, "black": 1, "even": 1, "odd": 1,
          "low": 1, "high": 1}


def color_of(n):
    """'green' (0), 'red' oder 'black'."""
    if n == 0:
        return "green"
    return "red" if n in RED_NUMBERS else "black"


def num_col(n):
    """Spalte (0..11) einer Zahl 1..36 auf dem waagerechten Tisch."""
    return (n - 1) // 3


def num_row(n):
    """Reihe (0 = oben .. 2 = unten) einer Zahl 1..36."""
    return 2 - (n - 1) % 3


def cell_number(col, row):
    """Zahl in Spalte col (0..11) und Reihe row (0..2)."""
    return 3 * col + 3 - row


def make_key(kind, numbers=None):
    """Kanonischer Wett-Schlüssel, z.B. make_key('cheval', (20, 17))."""
    if numbers is None:
        return kind
    if kind in ("column", "dozen"):
        return f"{kind}:{int(numbers)}"
    return kind + ":" + "-".join(str(n) for n in sorted(numbers))


def bet_kind(key):
    return key.split(":", 1)[0]


def bet_numbers(key):
    """Alle Zahlen, die eine Wette abdeckt (frozenset)."""
    kind, _, arg = key.partition(":")
    if kind == "column":
        r = int(arg)                      # Reihe 0 = obere Kolonne (3, 6, ...)
        return frozenset(cell_number(c, r) for c in range(12))
    if kind == "dozen":
        d = int(arg)
        return frozenset(range(1 + 12 * d, 13 + 12 * d))
    if kind == "red":
        return RED_NUMBERS
    if kind == "black":
        return frozenset(n for n in range(1, 37) if n not in RED_NUMBERS)
    if kind == "even":
        return frozenset(range(2, 37, 2))
    if kind == "odd":
        return frozenset(range(1, 37, 2))
    if kind == "low":
        return frozenset(range(1, 19))
    if kind == "high":
        return frozenset(range(19, 37))
    return frozenset(int(x) for x in arg.split("-"))


def valid_key(key):
    """Prüft einen Wett-Schlüssel (für geladene/wiederholte Einsätze)."""
    try:
        kind = bet_kind(key)
        nums = bet_numbers(key)
    except (ValueError, AttributeError):
        return False
    if kind not in PAYOUT or not nums or not all(0 <= n <= 36 for n in nums):
        return False
    return 36 // len(nums) - 1 == PAYOUT[kind] and make_key_of(key) == key


def make_key_of(key):
    kind, _, arg = key.partition(":")
    if kind in ("column", "dozen"):
        return key if arg in ("0", "1", "2") else None
    if kind in OUTSIDE_KEYS:
        return key if not arg else None
    return make_key(kind, bet_numbers(key))


def payout_multiplier(key):
    """Gewinn zu 1 (z.B. 35 bei Plein)."""
    return PAYOUT[bet_kind(key)]


def settle(bets, number):
    """Rechnet eine Drehung ab.

    bets: {schlüssel: einsatz}. Rückgabe (auszahlung, gewinner-schlüssel):
    auszahlung = Einsatz + Gewinn aller getroffenen Wetten (verlorene
    Einsätze sind schon abgebucht).
    """
    total = 0
    winners = []
    for key, amount in bets.items():
        if amount > 0 and number in bet_numbers(key):
            total += amount * (payout_multiplier(key) + 1)
            winners.append(key)
    return total, winners


def _street(col):
    return make_key("trans", (3 * col + 1, 3 * col + 2, 3 * col + 3))


def hit_test(ux, uy):
    """Welche Wette liegt an Tisch-Position (ux, uy)? Schlüssel oder None."""
    if not (0 <= ux < TABLE_W and 0 <= uy < TABLE_H):
        return None
    if uy < 3.0:
        if ux >= 13.0:                                   # 2:1-Kolonnen
            return make_key("column", int(uy))
        if ux < 1.0 - EDGE:                              # die Null selbst
            return make_key("plein", (0,))
        # Zahlenfeld (inkl. Kante zur Null)
        col = min(11, max(0, int(ux - 1.0)))
        row = min(2, int(uy))
        fx = ux - 1.0 - col
        fy = uy - row
        if ux < 1.0:                                     # rechter Rand der Null
            col, fx = 0, 0.0
        left, right = fx < EDGE, fx > 1.0 - EDGE
        top, bottom = fy < EDGE, fy > 1.0 - EDGE
        n = cell_number(col, row)
        vedge = left or right
        hedge = top or bottom
        if vedge and hedge:
            c1, c2 = (col - 1, col) if left else (col, col + 1)
            if top and row > 0:
                rows = (row - 1, row)
            elif bottom and row < 2:
                rows = (row, row + 1)
            else:
                rows = None                              # Außenlinie
            if rows is not None:
                if c1 < 0:                               # Trio mit der Null
                    return make_key("trans", (0, cell_number(0, rows[0]),
                                              cell_number(0, rows[1])))
                if c2 > 11:
                    return make_key("cheval", (cell_number(col, rows[0]),
                                               cell_number(col, rows[1])))
                return make_key("carre", tuple(cell_number(c, r)
                                               for c in (c1, c2)
                                               for r in rows))
            if c1 < 0:                                   # 0-1-2-3
                return make_key("carre", (0, 1, 2, 3))
            if c2 > 11:
                return _street(col)
            return make_key("sixain", tuple(range(3 * c1 + 1, 3 * c2 + 4)))
        if vedge:
            if left and col == 0:
                return make_key("cheval", (0, n))
            if right and col == 11:
                return make_key("plein", (n,))
            other = cell_number(col - 1 if left else col + 1, row)
            return make_key("cheval", (n, other))
        if hedge:
            if top and row > 0:
                return make_key("cheval", (n, cell_number(col, row - 1)))
            if bottom and row < 2:
                return make_key("cheval", (n, cell_number(col, row + 1)))
            return _street(col)
        return make_key("plein", (n,))
    if not (1.0 <= ux < 13.0):
        return None
    if uy < 3.0 + DOZEN_H:
        return make_key("dozen", min(2, int((ux - 1.0) / 4.0)))
    return OUTSIDE_KEYS[min(5, int((ux - 1.0) / 2.0))]


def anchor(key):
    """Mittelpunkt des Chip-Stapels einer Wette in Tisch-Einheiten."""
    kind = bet_kind(key)
    if kind == "column":
        return 13.5, int(key.split(":")[1]) + 0.5
    if kind == "dozen":
        return 1.0 + 4.0 * int(key.split(":")[1]) + 2.0, 3.0 + DOZEN_H / 2
    if kind in OUTSIDE_KEYS:
        return 1.0 + 2.0 * OUTSIDE_KEYS.index(kind) + 1.0, \
            3.0 + DOZEN_H + OUTSIDE_H / 2
    nums = sorted(bet_numbers(key))
    if kind == "plein":
        n = nums[0]
        if n == 0:
            return 0.5, 1.5
        return num_col(n) + 1.5, num_row(n) + 0.5
    if 0 in nums:
        rest = nums[1:]
        if len(nums) == 4:                               # 0-1-2-3
            return 1.0, 3.0
        rows = [num_row(n) for n in rest]
        if len(rest) == 1:                               # 0-n
            return 1.0, rows[0] + 0.5
        return 1.0, float(max(rows))                     # Trio
    cols = sorted({num_col(n) for n in nums})
    rows = sorted({num_row(n) for n in nums})
    x = cols[0] + 1.5 if len(cols) == 1 else cols[1] + 1.0
    if kind in ("trans", "sixain"):
        return x, 3.0
    y = rows[0] + 0.5 if len(rows) == 1 else float(rows[1])
    return x, y


# ============================================================= LAMA-SLOT
REELS = 5
ROWS = 3
WILD = "lama"
SCATTER = "coin"
SYMBOLS = ("lama", "coin", "seven", "gem", "bell", "clover", "grape",
           "lemon", "cherry")
LINE_BETS = (1, 2, 5, 10)
AUTO_COUNTS = (10, 25)
FREE_SPINS = 10
FREE_MULT = 2

# Gewinn je Linie (x Linien-Einsatz) für 3 / 4 / 5 gleiche von links
PAYS = {
    "lama":   (20, 100, 750),
    "seven":  (8, 30, 120),
    "gem":    (6, 20, 80),
    "bell":   (4, 12, 50),
    "clover": (3, 10, 30),
    "grape":  (2, 5, 20),
    "lemon":  (1, 4, 15),
    "cherry": (1, 4, 12),
}
# Auszahlungsquote für die Gewinntabelle - der Audit-Test prüft, dass sie mit
# exact_rtp() übereinstimmt (die exakte Rechnung dauert zur Laufzeit zu lange).
RTP_TEXT = "96.1"

# Scatter-Gewinn (x Gesamteinsatz) für 3 / 4 / 5 Münzen irgendwo
SCATTER_PAYS = {3: 2, 4: 10, 5: 50}

# Die 10 Gewinnlinien: Reihe (0 oben .. 2 unten) je Walze
LINES = (
    (1, 1, 1, 1, 1),
    (0, 0, 0, 0, 0),
    (2, 2, 2, 2, 2),
    (0, 1, 2, 1, 0),
    (2, 1, 0, 1, 2),
    (0, 0, 1, 2, 2),
    (2, 2, 1, 0, 0),
    (1, 0, 0, 0, 1),
    (1, 2, 2, 2, 1),
    (1, 0, 1, 2, 1),
)

# Walzenstreifen (je 32 Felder, zyklisch). Sichtbar sind drei aufeinander
# folgende Felder ab der Stopp-Position.
STRIPS = (
    ("cherry", "lama", "grape", "bell", "lemon", "gem", "cherry", "clover",
     "seven", "grape", "lemon", "lama", "cherry", "bell", "grape", "coin",
     "lemon", "clover", "gem", "cherry", "lama", "grape", "bell", "lemon",
     "seven", "clover", "cherry", "gem", "grape", "lama", "lemon", "bell"),
    ("lemon", "lama", "cherry", "gem", "grape", "bell", "lama", "clover",
     "cherry", "seven", "lemon", "grape", "lama", "bell", "cherry", "clover",
     "coin", "grape", "gem", "lemon", "lama", "cherry", "bell", "seven",
     "grape", "clover", "lemon", "gem", "lama", "cherry", "grape", "bell"),
    ("grape", "cherry", "lama", "bell", "lemon", "gem", "lama", "grape",
     "clover", "seven", "cherry", "lemon", "lama", "bell", "grape", "gem",
     "coin", "cherry", "clover", "lemon", "lama", "grape", "bell", "seven",
     "cherry", "clover", "lemon", "gem", "lama", "grape", "cherry", "bell"),
    ("cherry", "grape", "lama", "gem", "lemon", "bell", "lama", "cherry",
     "clover", "seven", "grape", "lemon", "lama", "bell", "cherry", "gem",
     "coin", "grape", "clover", "lemon", "lama", "cherry", "bell", "seven",
     "grape", "clover", "lemon", "gem", "lama", "cherry", "grape", "bell"),
    ("lemon", "cherry", "lama", "grape", "bell", "gem", "lemon", "clover",
     "seven", "cherry", "grape", "lama", "lemon", "bell", "cherry", "coin",
     "grape", "clover", "gem", "lemon", "lama", "cherry", "bell", "grape",
     "seven", "clover", "lemon", "gem", "cherry", "lama", "grape", "bell"),
)


def window(stops, strips=STRIPS):
    """Sichtbare Symbole: window[walze][reihe] ab den Stopp-Positionen."""
    out = []
    for r, stop in enumerate(stops):
        strip = strips[r]
        out.append(tuple(strip[(stop + row) % len(strip)]
                         for row in range(ROWS)))
    return out


def random_stops(rng, strips=STRIPS):
    """Zieht eine Stopp-Position je Walze (rng mit randrange)."""
    return [rng.randrange(len(s)) for s in strips]


def line_result(symbols):
    """Wertet die 5 Symbole einer Linie aus -> (symbol, anzahl, multiplikator).

    Links beginnend zählen gleiche Symbole und Lamas. Eine reine Lama-Reihe
    zahlt als Lama; stehen Lamas vor einem anderen Symbol, gilt der höhere
    der beiden Gewinne. Die Münze wird nie ersetzt.
    """
    wild_run = 0
    while wild_run < REELS and symbols[wild_run] == WILD:
        wild_run += 1
    best = (WILD, wild_run, PAYS[WILD][wild_run - 3] if wild_run >= 3 else 0)
    base = next((s for s in symbols if s != WILD), None)
    if base is None or base == SCATTER:
        return best if best[2] > 0 else (None, 0, 0)
    n = 0
    for s in symbols:
        if s == base or s == WILD:
            n += 1
        else:
            break
    mult = PAYS[base][n - 3] if n >= 3 else 0
    if mult > best[2]:
        best = (base, n, mult)
    return best if best[2] > 0 else (None, 0, 0)


def evaluate(win, line_bet, free=False):
    """Wertet ein Walzenbild aus.

    Rückgabe dict: lines = [(linie, symbol, anzahl, gewinn)], scatter
    (Anzahl Münzen), scatter_win, free_spins (neu gewonnene), total,
    jackpot (5 Lamas auf einer Linie), coins = [(walze, reihe), ...].
    """
    mult = FREE_MULT if free else 1
    lines = []
    total = 0
    jackpot = False
    for i, line in enumerate(LINES):
        sym, n, m = line_result([win[r][line[r]] for r in range(REELS)])
        if m > 0:
            amount = m * line_bet * mult
            lines.append((i, sym, n, amount))
            total += amount
            if sym == WILD and n == REELS:
                jackpot = True
    coins = [(r, row) for r in range(REELS) for row in range(ROWS)
             if win[r][row] == SCATTER]
    scatter_win = SCATTER_PAYS.get(len(coins), 0) * line_bet * len(LINES) * mult
    total += scatter_win
    return {"lines": lines, "scatter": len(coins), "coins": coins,
            "scatter_win": scatter_win, "total": total, "jackpot": jackpot,
            "free_spins": FREE_SPINS if len(coins) >= 3 else 0}


def exact_rtp(strips=STRIPS):
    """Auszahlungsquote EXAKT aus den Walzenstreifen.

    - Linien: Die Stopps der Walzen sind unabhängig und gleichverteilt, also
      hat jedes Feld einer Linie genau die Symbolhäufigkeit seines Streifens.
      Alle 9^5 Symbol-Kombinationen werden mit ihrer Wahrscheinlichkeit
      gewichtet (gleicher Erwartungswert für jede der 10 Linien).
    - Scatter: je Walze die Verteilung der Münzen im 3er-Fenster über alle
      Stopps, dann über die Walzen gefaltet.
    - Freispiele: jede Freispiel-Drehung bringt FREE_MULT x Grundwert und
      löst mit derselben Wahrscheinlichkeit 10 weitere aus; erwartete Anzahl
      nach der Wald-Gleichung FREE_SPINS / (1 - FREE_SPINS * p).

    Rückgabe dict mit rtp, line, scatter, trigger, jackpot_line, free_spins.
    """
    probs = []
    for strip in strips:
        counts = {}
        for s in strip:
            counts[s] = counts.get(s, 0) + 1
        probs.append([(s, c / len(strip)) for s, c in sorted(counts.items())])
    line = 0.0
    for combo in itertools.product(*probs):
        p = 1.0
        for _, q in combo:
            p *= q
        mult = line_result([s for s, _ in combo])[2]
        if mult:
            line += p * mult
    dist = [1.0]
    for strip in strips:
        n = len(strip)
        per = [0.0] * (ROWS + 1)
        for stop in range(n):
            k = sum(1 for row in range(ROWS)
                    if strip[(stop + row) % n] == SCATTER)
            per[k] += 1.0 / n
        new = [0.0] * (len(dist) + ROWS)
        for a, pa in enumerate(dist):
            for b, pb in enumerate(per):
                new[a + b] += pa * pb
        dist = new
    scatter = sum(p * SCATTER_PAYS.get(k, 0) for k, p in enumerate(dist))
    trigger = sum(p for k, p in enumerate(dist) if k >= 3)
    base = line + scatter
    exp_free = FREE_SPINS / (1.0 - FREE_SPINS * trigger)
    rtp = base + trigger * exp_free * FREE_MULT * base
    jackpot_line = 1.0
    for strip in strips:
        jackpot_line *= strip.count(WILD) / len(strip)
    return {"rtp": rtp, "line": line, "scatter": scatter, "trigger": trigger,
            "free_spins": exp_free, "jackpot_line": jackpot_line}
