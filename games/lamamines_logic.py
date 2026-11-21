# -*- coding: utf-8 -*-
"""
lamamines_logic.py
==================
Regeln von Lama Mines - ohne pygame, damit Web (``lamamines_logic.js``) und
Rust (``lamamines_logic.rs``) sie 1:1 übernehmen.

Prinzip: Auf einem Feld mit N = grid x grid Plätzen liegen M Minen, der Rest
sind Edelsteine. Jeder aufgedeckte Edelstein hebt den Multiplikator, eine
Mine kostet den Einsatz. Auszahlen geht ab dem ersten Edelstein.

Multiplikator (x100) nach k Edelsteinen - ganzzahlig, schrittweise, in allen
drei Sprachen bitgleich::

    raw(j)  = HOUSE * C(N, M) // C(N - j, M)
    m_0     = 100
    m_j     = max(raw(j), m_(j-1) + 1)          für j = 1 .. k
    m100(k) = min(MAX_MULT100, m_k)

Die Untergrenze ``m_(j-1) + 1`` sorgt dafür, dass schon der erste Edelstein
mindestens x1.01 bringt und jeder weitere den Multiplikator hebt (nach der
reinen Formel brächte z.B. 6x6 mit 1 Mine nur x0.99).

Die Chance, k Edelsteine hintereinander zu finden, ist C(N-k, M) / C(N, M);
jede Strategie "nach k Edelsteinen auszahlen" bringt damit 96-97 % (HOUSE)
des Einsatzes zurück (``expected_return``). Ausnahmen: Stufen, die die
Untergrenze anhebt, liegen leicht darüber - das sind nur drei: 6x6/1 Mine
k=1 (98,19 %), 7x7/1 Mine k=1 (Höchstwert 1,01 * 48/49 = 98,94 %) und
7x7/1 Mine k=2 (97,84 %); geprüft mit Grenze 99 % in
``tests/audit_chiplogic.py``. Gedeckelte Stufen (x100.000) liegen darunter:
bei vollem Abräumen ab 5x5 mit 6, 6x6 mit 5 und 7x7 mit 4 Minen (4x4 nie).
Größter Wert: C(49, 24) * 97 < 2^53, also exakt mit JS-Zahlen und Rust u64.

Das Minenfeld entsteht aus dem Fairness-Zufall der Runde
(``fairness.round_rand(KEY, server, client, nonce)``): Liste 0 .. N-1 mit
``seedrand.Rand.shuffle`` mischen, die ersten M Plätze sind Minen.
Plätze sind zeilenweise nummeriert: Platz = Zeile * grid + Spalte.
"""

from fractions import Fraction

import fairness

KEY = "lamamines"

# Feldgrößen (Tasten 1-4) und Standard
GRIDS = (4, 5, 6, 7)
DEFAULT_GRID = 5
DEFAULT_MINES = 3
# Einsatzstufen (Mindesteinsatz = erste Stufe, siehe lamabank.MIN_BET)
BETS = (10, 20, 50, 100, 200, 500, 1000, 2500, 5000, 10000)
# Auszahlungsquote in Prozent
HOUSE = 97
# Höchster Multiplikator x100 (x100.000) - wer ihn erreicht, wird ausgezahlt
MAX_MULT100 = 10_000_000
# Länge des Verlaufs
HISTORY = 12

# Schnellwahl der Minenzahl je Feldgröße
MINE_PRESETS = {
    4: (1, 2, 3, 5, 8, 12),
    5: (1, 3, 5, 10, 15, 24),
    6: (1, 3, 6, 12, 20, 35),
    7: (1, 3, 7, 14, 24, 48),
}

# Erfolge: "Mutig" ab x10, "Legende" ab x1000 ausgezahlt
BRAVE_MULT = 1000
LEGEND_MULT = 100000


def cells(grid):
    """Anzahl der Plätze."""
    return grid * grid


def max_mines(grid):
    """Höchste Minenzahl (mindestens ein Edelstein)."""
    return cells(grid) - 1


def gems(grid, mines):
    """Anzahl der Edelsteine."""
    return cells(grid) - mines


def clamp_mines(grid, mines):
    """Minenzahl auf 1 .. N-1 begrenzen."""
    return max(1, min(max_mines(grid), int(mines)))


def comb(n, k):
    """Binomialkoeffizient C(n, k), exakt und schrittweise.

    Rechenweg wie in JS/Rust: r = r * (n - k + i) // i mit k = min(k, n - k).
    Jedes Zwischenergebnis ist höchstens k * C(n, k) < 2^53 (für n <= 49).
    """
    if k < 0 or k > n:
        return 0
    k = min(k, n - k)
    r = 1
    for i in range(1, k + 1):
        r = r * (n - k + i) // i
    return r


def raw_mult100(grid, mines, j):
    """Reine Formel HOUSE * C(N, M) // C(N - j, M) (ohne Untergrenze und Deckel)."""
    n = cells(grid)
    return HOUSE * comb(n, mines) // comb(n - j, mines)


def mult100(grid, mines, k):
    """Multiplikator x100 nach k Edelsteinen (k <= 0 -> 100).

    Schrittweise: m_j = max(raw(j), m_(j-1) + 1), gedeckelt bei MAX_MULT100;
    k wird auf N-M begrenzt.
    """
    if k <= 0:
        return 100
    n = cells(grid)
    k = min(k, n - mines)
    top = HOUSE * comb(n, mines)
    m = 100
    for j in range(1, k + 1):
        m = max(top // comb(n - j, mines), m + 1)
        if m >= MAX_MULT100:
            return MAX_MULT100
    return m


def payout(bet, grid, mines, k):
    """Auszahlung nach k Edelsteinen (abgerundet auf ganze Chips)."""
    return bet * mult100(grid, mines, k) // 100


def payout_carry(bet, grid, mines, k, carry):
    """(Auszahlung, neuer Übertrag) mit Rest-Hundertstel-Übertrag wie bei Plinko.

    ``bet * m100 + carry`` in Hundertsteln; ganze Chips werden ausgezahlt, der
    Rest (0..99) bleibt für die nächste Auszahlung. Ohne Übertrag verliert
    ein kleiner Einsatz durch das Abrunden (Einsatz 10 bei x1.09 -> 10 Chips:
    Quote nur 88,9 %).
    """
    total = bet * mult100(grid, mines, k) + carry
    return total // 100, total % 100


def clean_carry(raw):
    """Gespeicherter Übertrag: ganze Zahl 0..99, sonst 0."""
    return raw if _is_int(raw) and 0 <= raw <= 99 else 0


def next_mult100(grid, mines, k):
    """Multiplikator x100 nach dem nächsten Edelstein oder None (alle gefunden)."""
    if k >= gems(grid, mines):
        return None
    return mult100(grid, mines, k + 1)


def safe_chance(grid, mines, k):
    """Chance, dass das nächste Feld sicher ist: (Zähler, Nenner) = (N-M-k, N-k)."""
    n = cells(grid)
    return (max(0, n - mines - k), max(1, n - k))


def safe_percent(grid, mines, k):
    """Chance auf ein sicheres Feld in ZEHNTEL-Prozent, kaufmännisch gerundet
    (845 = 84,5 %). Anzeige mit ``fmt_percent``."""
    num, den = safe_chance(grid, mines, k)
    return (num * 2000 + den) // (den * 2)


def all_safe_percent(grid, mines, k):
    """Chance, dass k bestimmte Felder ALLE sicher sind (Auto-Markierungen), in
    Zehntel-Prozent, kaufmännisch gerundet: C(N-k, M) / C(N, M).
    k <= 0 -> 1000, k > N-M -> 0. (JS rechnet mit BigInt, Rust mit i128.)"""
    n = cells(grid)
    if k <= 0:
        return 1000
    if k > n - mines:
        return 0
    num, den = comb(n - k, mines), comb(n, mines)
    return (num * 2000 + den) // (den * 2)


def fmt_percent(p10, sep="."):
    """Zehntel-Prozent als Text: 845 -> '84.5' (Trennzeichen je Sprache)."""
    return f"{p10 // 10}{sep}{p10 % 10}"


def is_max(grid, mines, k):
    """Höchstgewinn erreicht (Deckel MAX_MULT100) -> automatisch auszahlen."""
    return mult100(grid, mines, k) >= MAX_MULT100


def is_clear(grid, mines, k):
    """Alle Edelsteine gefunden -> automatisch auszahlen."""
    return k >= gems(grid, mines)


def ladder(grid, mines, k, count=5):
    """Die nächsten ``count`` Stufen als Liste (Schritt, Multiplikator x100)."""
    out = []
    for j in range(k + 1, min(gems(grid, mines), k + count) + 1):
        out.append((j, mult100(grid, mines, j)))
    return out


def make_field(rand, grid, mines):
    """Minenplätze (sortiert): Liste 0..N-1 mischen, die ersten M sind Minen."""
    order = list(range(cells(grid)))
    rand.shuffle(order)
    return sorted(order[:mines])


def field_from_seed(grid, mines, server, client, nonce):
    """Minenplätze einer Runde aus ihren Fairness-Werten (zum Nachrechnen)."""
    return make_field(fairness.round_rand(KEY, server, client, nonce), grid, mines)


def fmt_mult(m100, sep="."):
    """Multiplikator x100 als Text: 235 -> '2.35' (Trennzeichen je Sprache)."""
    return f"{m100 // 100}{sep}{m100 % 100:02d}"


def expected_return(grid, mines, k):
    """Erwartungswert (Anteil am Einsatz, exakt) der Strategie 'nach k auszahlen'."""
    n = cells(grid)
    k = max(0, min(k, n - mines))
    return Fraction(comb(n - k, mines), comb(n, mines)) * Fraction(mult100(grid, mines, k), 100)


def _is_int(v):
    return isinstance(v, int) and not isinstance(v, bool)


def clean_round(raw):
    """Gespeicherte, noch laufende Runde prüfen (aus lamabank-Extra "round").

    Gültig: {"bet": > 0, "grid": GRIDS, "mines": 1..N-1, "n", "s", "c", "h"
    (Fairness-Werte der Runde, h = sha256(s)), "open": [aufgedeckte Plätze in
    Klick-Reihenfolge]} - eindeutig, alle sicher laut neu berechnetem Feld und
    weniger als N-M (sonst wäre schon ausgezahlt). Sonst None.
    """
    if not isinstance(raw, dict):
        return None
    bet, grid, mines = raw.get("bet"), raw.get("grid"), raw.get("mines")
    n, s, c, h = raw.get("n"), raw.get("s"), raw.get("c"), raw.get("h")
    opened = raw.get("open")
    if not (_is_int(bet) and _is_int(grid) and _is_int(mines)) or bet <= 0 or grid not in GRIDS:
        return None
    if not 1 <= mines <= max_mines(grid):
        return None
    if not (fairness.valid_nonce(n) and fairness.valid_server(s) and fairness.valid_client(c)
            and fairness.valid_hash(h) and fairness.commit(s) == h):
        return None
    if not isinstance(opened, list) or len(opened) >= gems(grid, mines):
        return None
    if not all(_is_int(v) and 0 <= v < cells(grid) for v in opened) or len(set(opened)) != len(opened):
        return None
    field = set(field_from_seed(grid, mines, s, c, n))
    if any(v in field for v in opened):
        return None
    return {"bet": bet, "grid": grid, "mines": mines, "n": n, "s": s, "c": c, "h": h,
            "open": list(opened)}


def clean_history(raw):
    """Verlauf der letzten Runden: Liste von Multiplikatoren x100 (0 = verloren)."""
    if not isinstance(raw, list):
        return []
    out = [v for v in raw if _is_int(v) and 0 <= v <= MAX_MULT100]
    return out[-HISTORY:]


def clean_marks(raw):
    """Auto-Markierungen je Feldgröße: {"4": [...], "5": [...], "6": [...], "7": [...]}.

    Immer alle vier Schlüssel; je Liste eindeutige gültige Plätze, sortiert,
    höchstens N-1 Stück.
    """
    if not isinstance(raw, dict):
        raw = {}
    out = {}
    for grid in GRIDS:
        v = raw.get(str(grid))
        marks = []
        if isinstance(v, list):
            for c in v:
                if _is_int(c) and 0 <= c < cells(grid) and c not in marks:
                    marks.append(c)
        out[str(grid)] = sorted(marks)[:max_mines(grid)]
    return out
