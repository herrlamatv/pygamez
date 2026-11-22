# -*- coding: utf-8 -*-
"""
lamacrash_logic.py
==================
Regeln von Lama Crash - ohne pygame, damit Web (``lamacrash_logic.js``) und
Rust (``lamacrash_logic.rs``) sie 1:1 übernehmen. Alle Multiplikatoren sind
**Ganzzahlen in Hundertsteln** (m100: 235 = ×2,35); bis auf ``world_blend``
(Zeichenhilfe) kommt keine Gleitkommazahl vor.

Prinzip: Ein Lama steigt im Ballon auf, der Multiplikator wächst mit der Zeit.
Wer aussteigt, bekommt Einsatz × aktuellen Multiplikator; platzt der Ballon
vorher, ist der noch fliegende Einsatz weg.

**Platzpunkt** aus dem Fairness-Zufall der Runde
(``fairness.round_rand(KEY, server, client, nonce)``, mulberry32)::

    r        = rand.next_u32()                       # 0 .. 2^32-1
    crash100 = min(MAX, max(100, 97 * 2^32 // (2^32 - r)))

Damit gilt für jedes Ziel 101 <= T <= MAX exakt
``P(crash100 >= T) = (97 * 2^32 // T) / 2^32`` - jede Ausstiegs-Strategie
bringt 96,997 bis 97,000 % zurück (``expected_return``); rund 3,96 % der
Runden platzen schon bei ×1,00. Größter Zwischenwert 97 * 2^32 < 2^53
(exakt mit JS-Zahlen; in JS ``97 * 4294967296`` schreiben, nicht ``<<``).

**Steigkurve** (ein Takt = 1/60 s)::

    mu_0 = 1_000_000;   mu_(t+1) = mu_t + mu_t // 1000;   m100(t) = mu_t // 10_000

×2 nach 694 Takten (11,6 s), ×10 nach 2305 (38,4 s), ×100 nach 4608
(76,8 s), ×100.000 nach MOON_TICK = 11520 Takten (192 s). Größtes mu ~1,0e11.

**Takt-Regel** (verbindlich für alle Fassungen, ``tick``)::

    t += 1;  m = mult_at(t)
    1. jeder noch fliegende Teil mit Auto-Ziel a: a <= m und a <= crash100
       -> genau a auszahlen
    2. crash100 >= MAX und m >= MAX  -> Mondlandung: alle fliegenden Teile
       bekommen MAX, Rundenende "moon"
    3. sonst m > crash100            -> Platzen, Rundenende "pop"

Im Flug gilt also immer m <= crash100: Wer von Hand bei m aussteigt, gewinnt
unter genau derselben Bedingung wie ein Auto-Ziel m. Aussteigen geht erst ab
MIN_CASH (×1,01). Jeder Einsatz darf einmal **halbiert** werden: ``b // 2``
steigt aus, ``b - b // 2`` fliegt weiter.

Gespeicherte Runde (Schlüsselreihenfolge fest)::

    {"n", "s", "c", "h",           # Fairness-Werte der Runde (h = sha256(s))
     "t",                          # Takt
     "bets": [{"b": Einsatz, "a": Auto-Ziel (0 = aus),
               "hm": Ausstieg der Hälfte (0 = nicht geteilt),
               "xm": Ausstieg des Rests bzw. des ganzen Einsatzes (0 = fliegt),
               "p": bisher ausgezahlt, "man": Bits 1/2 = Hälfte/Rest von Hand}]}

Fairness-Ergebnis ``{"x": crash100, "t": Endtakt, "b": [[b, hm, xm, p], ...]}``.
"""

from bisect import bisect_left
from fractions import Fraction

import fairness

KEY = "lamacrash"

# Auszahlungsquote in Prozent
HOUSE = 97
# Höchster Multiplikator x100 (x100.000 = Mondlandung)
MAX_MULT100 = 10_000_000
# Kleinster Ausstieg (x1,01) - auch kleinstes Auto-Ziel
MIN_CASH = 101
# Einsatzstufen je Einsatz (Mindesteinsatz = erste Stufe, siehe lamabank.MIN_BET)
BETS = (10, 20, 50, 100, 200, 500, 1000, 2500, 5000, 10000)
# höchstens so viele Einsätze in einer Runde
MAX_BETS = 2
# Länge des Verlaufs (Platzpunkte)
HISTORY = 12

# Steigkurve
TPS = 60                 # Takte je Sekunde (Normal-Tempo)
MU0 = 1_000_000          # mu bei x1,00
GROWTH = 1000            # je Takt wächst mu um mu // GROWTH (~0,1 %)
MU_UNIT = 10_000         # m100 = mu // MU_UNIT
MOON_TICK = 11520        # erster Takt mit m100 >= MAX_MULT100
SPEED_TURBO = 3          # Takte je 1/60 s im Turbo
SPEED_FF = 16            # Takte je 1/60 s beim Vorspulen (alle ausgestiegen)

# Auto-Ziele (Leiter für ←/→ und das Mausrad)
TARGETS = (101, 102, 105, 110, 115, 120, 125, 130, 140, 150, 160, 175, 200, 225, 250,
           300, 350, 400, 500, 600, 750, 1000, 1250, 1500, 2000, 2500, 3000, 4000,
           5000, 7500, 10000, 15000, 20000, 25000, 50000, 75000, 100000, 250000,
           500000, 1000000, 2500000, 5000000, 10000000)
DEFAULT_TARGET = 200

# Höhenwelten: Wiese, Wolken, Gewitter, Nordlicht, Weltall, Mond
WORLDS = ("meadow", "clouds", "storm", "aurora", "space", "moon")
WORLD_AT = (100, 150, 300, 1000, 10000, MAX_MULT100)
# Überblendung zur nächsten Welt: so viele Takte vor ihrer Schwelle
BLEND_TICKS = 90

# Erfolge: "Polarlicht-Pilot" ab x10, "Weltraumlama" ab x1000 ausgestiegen;
# "Haarscharf": von Hand ab x2 ausgestiegen und höchstens 5 % unter dem Platzpunkt
BRAVE_MULT = 1000
SPACE_MULT = 100000
CLOSE_MIN = 200
CLOSE_PCT = 95

# Statistik: Verteilung der Platzpunkte (Untergrenzen der Fächer)
BUCKETS = (100, 150, 200, 500, 1000, 10000, 100000)

# Kurve: sichtbare Zeit mindestens 8 s
CHART_MIN_TICKS = 8 * TPS
_Y_STEPS = (10, 20, 25, 50, 100, 200, 250, 500, 1000, 2000, 2500, 5000, 10000, 20000,
            25000, 50000, 100000, 200000, 250000, 500000, 1000000, 2000000, 2500000, 5000000)
_X_STEPS = (1, 2, 5, 10, 15, 20, 30, 60, 120)        # Sekunden

TWO32 = 4294967296

# ===================================================== Steigkurve
_MU = []         # mu je Takt 0 .. MOON_TICK
_M = []          # m100 je Takt (gedeckelt bei MAX_MULT100)


def _table():
    """Baut die Kurve einmal auf (11521 Werte, schrittweise wie oben)."""
    if not _MU:
        mu = MU0
        for _ in range(MOON_TICK + 1):
            _MU.append(mu)
            _M.append(min(MAX_MULT100, mu // MU_UNIT))
            mu += mu // GROWTH
    return _M


def mu_at(tick):
    """mu zum Takt (ungedeckelt; Takt auf 0 .. MOON_TICK begrenzt)."""
    _table()
    return _MU[max(0, min(MOON_TICK, int(tick)))]


def mult_at(tick):
    """Multiplikator x100 zum Takt (100 bei Takt 0, höchstens MAX_MULT100)."""
    return _table()[max(0, min(MOON_TICK, int(tick)))]


def tick_at(m100):
    """Erster Takt mit mult_at(t) >= m100 (<= 100 -> 0, > MAX -> MOON_TICK).

    Binärsuche über die Kurve (in JS/Rust von Hand: lo/hi wie bisect_left)."""
    m = _table()
    return min(MOON_TICK, bisect_left(m, m100))


# ===================================================== Platzpunkt
def crash_from_u32(r):
    """Platzpunkt x100 aus einer 32-Bit-Zufallszahl r (0 .. 2^32-1)."""
    v = (HOUSE * TWO32) // (TWO32 - r)
    return min(MAX_MULT100, max(100, v))


def crash_point(server, client, nonce):
    """Platzpunkt einer Runde aus ihren Fairness-Werten (zum Nachrechnen)."""
    return crash_from_u32(fairness.round_rand(KEY, server, client, nonce).next_u32())


def pop_tick(crash100):
    """Takt, an dem die Runde endet: Mond -> MOON_TICK, sonst der erste Takt mit
    mult_at(t) > crash100. Nur für Tests und ``clean_round`` - das Spiel liest
    crash100 ausschließlich in ``tick`` und beim Rundenabschluss."""
    if crash100 >= MAX_MULT100:
        return MOON_TICK
    return tick_at(crash100 + 1)


def win_count(t100):
    """Anzahl der 2^32 Zufallswerte, bei denen crash100 >= t100 (101 <= t100)."""
    t100 = max(MIN_CASH, min(MAX_MULT100, int(t100)))
    return (HOUSE * TWO32) // t100


def expected_return(t100):
    """Erwartete Auszahlung (Anteil am Einsatz, exakt) beim Ausstieg bei t100."""
    t100 = max(MIN_CASH, min(MAX_MULT100, int(t100)))
    return Fraction(t100, 100) * Fraction(win_count(t100), TWO32)


def fmt_chance(t100, sep="."):
    """Chance, t100 zu erreichen, in Prozent als Text (kaufmännisch gerundet):
    ab 1 % zwei Nachkommastellen ('48.50'), darunter vier ('0.0010')."""
    num, den = win_count(t100), TWO32
    if num * 100 >= den:
        v = (num * 20000 + den) // (den * 2)
        return f"{v // 100}{sep}{v % 100:02d}"
    v = (num * 2000000 + den) // (den * 2)
    return f"{v // 10000}{sep}{v % 10000:04d}"


# ===================================================== Auszahlung
def payout_carry(stake, m100, carry):
    """(Auszahlung, neuer Übertrag) mit Rest-Hundertstel-Übertrag wie bei Mines:
    ``stake * m100 + carry`` in Hundertsteln, ganze Chips werden ausgezahlt,
    der Rest (0..99) bleibt für die nächste Auszahlung."""
    total = stake * m100 + carry
    return total // 100, total % 100


def clean_carry(raw):
    """Gespeicherter Übertrag: ganze Zahl 0..99, sonst 0."""
    return raw if _is_int(raw) and 0 <= raw <= 99 else 0


def half_stake(b):
    """Anteil, der beim Halbieren aussteigt."""
    return b // 2


def rest_stake(b):
    """Anteil, der nach dem Halbieren weiterfliegt."""
    return b - b // 2


def riding(bet):
    """Noch fliegender Einsatz eines Teil-Einsatzes (0 = ganz ausgestiegen)."""
    if bet["xm"]:
        return 0
    return rest_stake(bet["b"]) if bet["hm"] else bet["b"]


def flying(rnd):
    """Fliegt in der Runde noch Geld mit?"""
    return any(riding(b) > 0 for b in rnd["bets"])


def stake_total(rnd):
    """Gesamteinsatz der Runde."""
    total = 0
    for b in rnd["bets"]:
        total += b["b"]
    return total


def paid_total(rnd):
    """Bisher ausgezahlt (alle Teile)."""
    total = 0
    for b in rnd["bets"]:
        total += b["p"]
    return total


def target_ok(t100):
    """Gültiges Auto-Ziel? (ganze Zahl MIN_CASH .. MAX_MULT100)"""
    return _is_int(t100) and MIN_CASH <= t100 <= MAX_MULT100


def step_target(t100, direction):
    """Nächstes Auto-Ziel der Leiter in Richtung direction (+1/-1)."""
    t100 = max(MIN_CASH, min(MAX_MULT100, int(t100)))
    if direction > 0:
        for v in TARGETS:
            if v > t100:
                return v
        return TARGETS[-1]
    for v in reversed(TARGETS):
        if v < t100:
            return v
    return TARGETS[0]


def new_bet(b, a):
    """Teil-Einsatz zu Rundenbeginn (a = Auto-Ziel oder 0)."""
    return {"b": b, "a": a, "hm": 0, "xm": 0, "p": 0, "man": 0}


def new_round(entry, pairs):
    """Neue Runde aus dem Fairness-Eintrag (``fairness.begin``) und den Einsätzen
    pairs = [(Einsatz, Auto-Ziel oder 0), ...] (1 .. MAX_BETS)."""
    return {"n": entry["n"], "s": entry["s"], "c": entry["c"], "h": entry["h"], "t": 0,
            "bets": [new_bet(b, a) for b, a in pairs]}


def _pay(bet, m100, carry):
    """Den noch fliegenden Teil eines Einsatzes bei m100 auszahlen."""
    win, carry = payout_carry(riding(bet), m100, carry)
    bet["xm"] = m100
    bet["p"] += win
    return win, carry


def tick(rnd, crash100, carry):
    """Einen Takt weiter (Takt-Regel oben). Liefert (Ereignisse, Übertrag, Ende):
    Ereignisse = [(Index, "auto"|"moon", m100, Gewinn), ...] in Auszahlungs-
    reihenfolge, Ende = None (fliegt weiter) / "pop" / "moon"."""
    rnd["t"] += 1
    m = mult_at(rnd["t"])
    events = []
    for i, bet in enumerate(rnd["bets"]):
        a = bet["a"]
        if a and riding(bet) > 0 and a <= m and a <= crash100:
            win, carry = _pay(bet, a, carry)
            events.append((i, "auto", a, win))
    if crash100 >= MAX_MULT100 and m >= MAX_MULT100:
        for i, bet in enumerate(rnd["bets"]):
            if riding(bet) > 0:
                win, carry = _pay(bet, MAX_MULT100, carry)
                events.append((i, "moon", MAX_MULT100, win))
        return events, carry, "moon"
    if m > crash100:
        return events, carry, "pop"
    return events, carry, None


def cash_out(rnd, i, half, carry):
    """Von Hand aussteigen (ganz oder Hälfte) beim aktuellen Takt.

    Liefert (Gewinn, m100, neuer Übertrag) oder None (nicht erlaubt: unter
    MIN_CASH, nichts mehr im Flug, Hälfte schon genommen). Das Spiel ruft das nur,
    solange die Runde fliegt - dann ist mult_at(t) <= crash100 sicher."""
    if not 0 <= i < len(rnd["bets"]):
        return None
    bet = rnd["bets"][i]
    m = mult_at(rnd["t"])
    if m < MIN_CASH or riding(bet) <= 0:
        return None
    if half:
        if bet["hm"]:
            return None
        win, carry = payout_carry(half_stake(bet["b"]), m, carry)
        bet["hm"] = m
        bet["man"] |= 1
    else:
        win, carry = payout_carry(riding(bet), m, carry)
        bet["xm"] = m
        bet["man"] |= 2
    bet["p"] += win
    return win, m, carry


def best_cash(rnd):
    """Höchster Ausstieg der Runde (m100, 0 = keiner)."""
    best = 0
    for b in rnd["bets"]:
        best = max(best, b["hm"], b["xm"])
    return best


def close_call(rnd, crash100):
    """"Haarscharf": ein Ausstieg VON HAND ab CLOSE_MIN und höchstens 5 % unter dem
    Platzpunkt (ganzzahlig: m * 100 >= CLOSE_PCT * crash100)."""
    for b in rnd["bets"]:
        for bit, m in ((1, b["hm"]), (2, b["xm"])):
            if b["man"] & bit and m >= CLOSE_MIN and m * 100 >= CLOSE_PCT * crash100:
                return True
    return False


def outcome(rnd, crash100):
    """Ergebnis für das Fairness-Log (nur ganze Zahlen, Tiefe 3)."""
    return {"x": crash100, "t": rnd["t"],
            "b": [[b["b"], b["hm"], b["xm"], b["p"]] for b in rnd["bets"]]}


def check_outcome(o, crash100):
    """Passt ein Log-Ergebnis zum nachgerechneten Platzpunkt? (Platzpunkt gleich,
    jeder Ausstieg 0 oder MIN_CASH .. crash100)"""
    if not isinstance(o, dict) or o.get("x") != crash100:
        return False
    bets = o.get("b")
    if not isinstance(bets, list) or not 1 <= len(bets) <= MAX_BETS:
        return False
    for row in bets:
        if not isinstance(row, list) or len(row) != 4 or not all(_is_int(v) for v in row):
            return False
        for m in (row[1], row[2]):
            if m and not MIN_CASH <= m <= crash100:
                return False
    return True


# ===================================================== Höhenwelten / Kurve
def world_of(m100):
    """Welt (0 = Wiese .. 5 = Mond) zum Multiplikator."""
    w = 0
    for i, lo in enumerate(WORLD_AT):
        if m100 >= lo:
            w = i
    return w


_WORLD_T = []


def world_ticks():
    """Takt, an dem jede Welt beginnt (Wiese = 0) - einmal berechnet."""
    if not _WORLD_T:
        _WORLD_T.extend(tick_at(v) for v in WORLD_AT)
    return tuple(_WORLD_T)


def world_blend(tick_f):
    """(Welt, Anteil 0..255 der nächsten Welt) zum (gebrochenen) Takt - die
    Überblendung beginnt BLEND_TICKS vor der nächsten Schwelle. Nur fürs Zeichnen;
    die Mond-Welt (5) erreicht die Kurve nie von selbst (Weltall blendet nicht über)."""
    starts = world_ticks()
    w = 0
    for i in range(1, 5):
        if tick_f >= starts[i]:
            w = i
    if w >= 4:
        return 4, 0
    nxt = starts[w + 1]
    left = nxt - tick_f
    if left >= BLEND_TICKS:
        return w, 0
    return w, max(0, min(255, int((BLEND_TICKS - left) * 255 / BLEND_TICKS)))


def chart_span(tick):
    """Achsen der Kurve (wachsen mit): (x_max in Takten, y_max in m100)."""
    x_max = max(CHART_MIN_TICKS, tick + tick // 5)
    m = mult_at(tick)
    y_max = max(200, m + (m - 100) // 4)
    return x_max, y_max


def chart_ylines(y_max):
    """Waagerechte Hilfslinien (m100-Werte über x1,00, höchstens 4)."""
    span = max(1, y_max - 100)
    step = _Y_STEPS[-1]
    for s in _Y_STEPS:
        if span // s <= 4:
            step = s
            break
    first = (100 // step + 1) * step
    return [v for v in range(first, y_max + 1, step)][:4]


def chart_xlines(x_max):
    """Senkrechte Hilfslinien als Takte (ganze Sekunden, höchstens 5)."""
    step = _X_STEPS[-1]
    for s in _X_STEPS:
        if x_max // (s * TPS) <= 5:
            step = s
            break
    return [v * step * TPS for v in range(1, x_max // (step * TPS) + 1)][:5]


# ===================================================== Statistik
def bucket_of(crash100):
    """Fach der Verteilung (0 .. len(BUCKETS)-1)."""
    b = 0
    for i, lo in enumerate(BUCKETS):
        if crash100 >= lo:
            b = i
    return b


def bucket_prob(i):
    """Erwartete Wahrscheinlichkeit eines Fachs (exakt)."""
    lo = BUCKETS[i]
    p_lo = Fraction(1) if lo <= 100 else Fraction(win_count(lo), TWO32)
    if i + 1 >= len(BUCKETS):
        return p_lo
    return p_lo - Fraction(win_count(BUCKETS[i + 1]), TWO32)


_X_COUNTERS = ("cash", "half", "auto", "inst", "moon", "close", "top")


def blank_x():
    """Leere Crash-Zähler für ``stats["x"]``."""
    out = {"hits": [0] * len(BUCKETS)}
    for k in _X_COUNTERS:
        out[k] = 0
    return out


def clean_x(raw):
    """Crash-Zähler prüfen: hits = Liste mit len(BUCKETS) Zahlen >= 0, sonst Nullen;
    jeder Zähler ganze Zahl >= 0, sonst 0."""
    out = blank_x()
    if not isinstance(raw, dict):
        return out
    hits = raw.get("hits")
    if isinstance(hits, list) and len(hits) == len(BUCKETS):
        out["hits"] = [v if _is_int(v) and 0 <= v <= fairness.MAX_NONCE else 0 for v in hits]
    for k in _X_COUNTERS:
        v = raw.get(k)
        if _is_int(v) and 0 <= v <= fairness.MAX_NONCE:
            out[k] = v
    return out


def record_x(x, rnd, crash100, end):
    """Eine fertige Runde in die Crash-Zähler eintragen."""
    x["hits"][bucket_of(crash100)] += 1
    for b in rnd["bets"]:
        if b["hm"]:
            x["cash"] += 1
            x["half"] += 1
        if b["xm"]:
            x["cash"] += 1
            if not b["man"] & 2:
                x["auto"] += 1
    if crash100 < MIN_CASH:
        x["inst"] += 1
    if end == "moon":
        x["moon"] += 1
    if close_call(rnd, crash100):
        x["close"] += 1
    x["top"] = max(x["top"], crash100)
    return x


# ===================================================== Prüfen / Laden
def _is_int(v):
    return isinstance(v, int) and not isinstance(v, bool)


def clean_history(raw):
    """Verlauf der letzten Runden: Liste von Platzpunkten x100."""
    if not isinstance(raw, list):
        return []
    out = [v for v in raw if _is_int(v) and 100 <= v <= MAX_MULT100]
    return out[-HISTORY:]


def _clean_bet(raw, m, crash100):
    if not isinstance(raw, dict):
        return None
    b, a, hm, xm, p, man = (raw.get(k) for k in ("b", "a", "hm", "xm", "p", "man"))
    if not all(_is_int(v) for v in (b, a, hm, xm, p, man)):
        return None
    if b not in BETS or not (a == 0 or target_ok(a)) or not 0 <= man <= 3 or p < 0:
        return None
    for v in (hm, xm):
        if v and not MIN_CASH <= v <= min(m, crash100):
            return None
    if hm and xm and xm < hm:
        return None
    if (man & 1 and not hm) or (man & 2 and not xm):
        return None
    if p > b * MAX_MULT100 // 100 + 1:
        return None
    bet = {"b": b, "a": a, "hm": hm, "xm": xm, "p": p, "man": man}
    # ein schon erreichtes Auto-Ziel muss ausgezahlt sein
    if a and riding(bet) > 0 and a <= m and a <= crash100:
        return None
    return bet


def clean_round(raw):
    """Gespeicherte, noch offene Runde prüfen (aus lamabank-Extra "round").

    Gültig: Fairness-Werte mit h = sha256(s), Takt 0 .. MOON_TICK vor dem
    Rundenende (Platzpunkt aus den Seeds neu berechnet), 1 .. MAX_BETS Einsätze
    mit Ausstiegen höchstens min(mult_at(t), crash100). Eine Runde ohne
    fliegende Teile ist gültig (das Spiel schließt sie beim Laden ab). Sonst None.
    """
    if not isinstance(raw, dict):
        return None
    n, s, c, h, t = (raw.get(k) for k in ("n", "s", "c", "h", "t"))
    if not (fairness.valid_nonce(n) and fairness.valid_server(s) and fairness.valid_client(c)
            and fairness.valid_hash(h) and fairness.commit(s) == h):
        return None
    if not _is_int(t) or not 0 <= t <= MOON_TICK:
        return None
    crash100 = crash_point(s, c, n)
    m = mult_at(t)
    if crash100 >= MAX_MULT100:
        if t >= MOON_TICK:
            return None
    elif m > crash100:
        return None
    raw_bets = raw.get("bets")
    if not isinstance(raw_bets, list) or not 1 <= len(raw_bets) <= MAX_BETS:
        return None
    bets = []
    for rb in raw_bets:
        bet = _clean_bet(rb, m, crash100)
        if bet is None:
            return None
        bets.append(bet)
    return {"n": n, "s": s, "c": c, "h": h, "t": t, "bets": bets}


# ===================================================== Formatierung
def fmt_mult(m100, sep="."):
    """Multiplikator x100 als Text mit zwei Stellen: 235 -> '2.35'."""
    return f"{m100 // 100}{sep}{m100 % 100:02d}"


def fmt_short(m100, sep="."):
    """Kurzform für Pillen/Leiste: x1,23 · x12,3 · x123 · x12,3k · x100k (ohne x)."""
    if m100 < 1000:
        return f"{m100 // 100}{sep}{m100 % 100:02d}"
    if m100 < 10000:
        return f"{m100 // 100}{sep}{(m100 // 10) % 10}"
    if m100 < 100000:
        return str(m100 // 100)
    if m100 < 1000000:
        return f"{m100 // 100000}{sep}{(m100 // 10000) % 10}k"
    return f"{m100 // 100000}k"
