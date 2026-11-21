# -*- coding: utf-8 -*-
"""
chipstats.py
============
Statistik der Chip-Spiele Lama Mines und Lama Plinko (Fenster **S**) - ohne
pygame, Gegenstücke: ``web/js/games/chipstats.js`` und
``rust/crates/pgz-core/src/chipstats.rs``.

Gespeichert wird je Profil im lamabank-Extra des Spiels unter "stats"::

    {"rounds", "wins", "losses", "wagered", "paid", "net",
     "best_win", "best_mult", "curve": [Bilanz nach jeder Runde], "x": {...}}

``x`` gehört dem Spiel (z.B. Plinko-Treffer je Reihenzahl). Eine Runde mit
Auszahlung über dem Einsatz zählt als Gewinn, darunter als Verlust, genau
der Einsatz zurück als keins von beiden. Bei Plinko ist jede Kugel eine
"Runde".
"""

# Länge der Bilanzkurve (letzte Werte)
CURVE = 100
# größte gespeicherte Zahl (bleibt in JS-Zahlen exakt)
MAX_INT = 2 ** 53 - 1

_COUNTERS = ("rounds", "wins", "losses", "wagered", "paid", "net", "best_win", "best_mult")


def blank():
    """Leere Statistik."""
    return {"rounds": 0, "wins": 0, "losses": 0, "wagered": 0, "paid": 0, "net": 0,
            "best_win": 0, "best_mult": 0, "curve": [], "x": {}}


def _is_int(v):
    return isinstance(v, int) and not isinstance(v, bool) and -MAX_INT <= v <= MAX_INT


def _plain(v, depth=0):
    """Nur JSON-Grundtypen (None, bool, int, str, list, dict mit str-Schlüsseln),
    keine Gleitkommazahlen, Tiefe höchstens 4 (wie fairness._plain)."""
    if v is None or isinstance(v, (bool, str)):
        return True
    if isinstance(v, int):
        return -MAX_INT <= v <= MAX_INT
    if depth >= 4:
        return False
    if isinstance(v, list):
        return all(_plain(x, depth + 1) for x in v)
    if isinstance(v, dict):
        return all(isinstance(k, str) and _plain(x, depth + 1) for k, x in v.items())
    return False


def clean(raw):
    """Gespeicherte Statistik prüfen (robust gegen kaputte Dateien).

    Zähler: ganze Zahlen >= 0 (bool zählt nicht), sonst 0; ``net`` wird aus
    ``paid - wagered`` neu berechnet. ``curve``: nur ganze Zahlen, die
    letzten CURVE. ``x``: dict, nur Einträge aus JSON-Grundtypen.
    """
    st = blank()
    if not isinstance(raw, dict):
        return st
    for k in _COUNTERS:
        v = raw.get(k)
        if k != "net" and _is_int(v) and v >= 0:
            st[k] = v
    st["net"] = st["paid"] - st["wagered"]
    curve = raw.get("curve")
    if isinstance(curve, list):
        st["curve"] = [v for v in curve if _is_int(v)][-CURVE:]
    x = raw.get("x")
    if isinstance(x, dict):
        st["x"] = {k: v for k, v in x.items() if isinstance(k, str) and _plain(v, 1)}
    return st


def record(st, wager, paid, mult100):
    """Eine Runde (bzw. Kugel) eintragen: Einsatz, Auszahlung, Multiplikator x100."""
    st["rounds"] += 1
    if paid > wager:
        st["wins"] += 1
    elif paid < wager:
        st["losses"] += 1
    st["wagered"] += wager
    st["paid"] += paid
    st["net"] += paid - wager
    st["best_win"] = max(st["best_win"], paid - wager)
    st["best_mult"] = max(st["best_mult"], mult100)
    st["curve"].append(st["net"])
    del st["curve"][:-CURVE]
    return st


def rtp(st):
    """Tatsächliche Auszahlungsquote (paid / wagered) oder None ohne Einsatz."""
    if st["wagered"] <= 0:
        return None
    return st["paid"] / st["wagered"]


def win_rate(st):
    """Anteil gewonnener Runden (wins / rounds) oder None ohne Runde."""
    if st["rounds"] <= 0:
        return None
    return st["wins"] / st["rounds"]
