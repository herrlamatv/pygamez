# -*- coding: utf-8 -*-
"""
fairness.py
===========
Nachprüfbarer Zufall ("Provably Fair") für die Chip-Spiele Lama Mines und
Lama Plinko - ohne pygame, Gegenstücke: ``web/js/games/fairness.js`` und
``rust/crates/pgz-core/src/fairness.rs``.

Prinzip:

* Vor jeder Runde steht ein geheimer **Server-Seed** fest (32 Hex-Zeichen).
  Angezeigt wird nur sein **Prüfcode** ``sha256_hex(server)``.
* Der Spieler hat einen **Client-Seed** (8 Zeichen, jederzeit neu wählbar)
  und es zählt eine **Nonce** (Rundennummer) mit.
* Der Rundenzufall ist ``seedrand.Rand(seedrand.seed_from(spiel, server,
  client, nonce))`` - in Python, JS und Rust bitgleich.
* Nach der Runde wird der Server-Seed gezeigt: Wer ``sha256(server)`` mit dem
  vorab gezeigten Prüfcode vergleicht und das Ergebnis aus den drei Werten
  nachrechnet, sieht, dass nichts nachträglich geändert wurde.

Ablauf in einem Spiel (``state`` liegt im lamabank-Extra unter "fair")::

    entry = fairness.begin(state)        # {"n", "s", "c", "h"} dieser Runde
    fairness.advance(state, self.rng)    # nächster Prüfcode steht schon fest
    rand = fairness.entry_rand(KEY, entry)
    ... Runde mit rand ausspielen ...
    log = fairness.record(log, entry, ergebnis)   # letzte 12 im Fenster F

Neue Seeds kommen bewusst aus Pythons ``random.Random`` des Spiels
(``getrandbits``/``randrange``): Die Rust-Fassung (``py::Random``) zieht mit
demselben Startwert exakt dieselben Seeds - so bleiben die Vergleichsläufe
Python = Rust gleich. Im Browser kommen Seeds aus ``crypto.getRandomValues``.
"""

import hashlib

import seedrand

# Zeichen des Client-Seeds (ohne verwechselbare l/o/0/1), genau 32 Stück
CLIENT_ALPHABET = "abcdefghijkmnpqrstuvwxyz23456789"
# Länge eines neuen Client-Seeds und erlaubte Länge eines gespeicherten
CLIENT_LEN = 8
CLIENT_MAX = 16
# Einträge im Fairness-Fenster (letzte Runden bzw. Kugeln)
LOG = 12
# größte Nonce (bleibt in JS-Zahlen exakt)
MAX_NONCE = 2 ** 53 - 1

_HEX = "0123456789abcdef"


def sha256_hex(text):
    """SHA-256 eines Textes (UTF-8) als 64 kleine Hex-Zeichen."""
    return hashlib.sha256(str(text).encode("utf-8")).hexdigest()


def commit(server):
    """Prüfcode eines Server-Seeds (wird VOR der Runde gezeigt)."""
    return sha256_hex(server)


def new_server_seed(rng):
    """Neuer Server-Seed: 32 Hex-Zeichen aus zwei getrandbits(64)."""
    return "%016x%016x" % (rng.getrandbits(64), rng.getrandbits(64))


def new_client_seed(rng):
    """Neuer Client-Seed: 8 Zeichen aus CLIENT_ALPHABET (je randrange(32))."""
    return "".join(CLIENT_ALPHABET[rng.randrange(32)] for _ in range(CLIENT_LEN))


def _is_hex(v, n):
    return isinstance(v, str) and len(v) == n and all(ch in _HEX for ch in v)


def _is_int(v):
    return isinstance(v, int) and not isinstance(v, bool)


def valid_server(v):
    """Server-Seed gültig? (genau 32 kleine Hex-Zeichen)"""
    return _is_hex(v, 32)


def valid_client(v):
    """Client-Seed gültig? (1-16 Zeichen aus CLIENT_ALPHABET)"""
    return (isinstance(v, str) and 1 <= len(v) <= CLIENT_MAX
            and all(ch in CLIENT_ALPHABET for ch in v))


def valid_nonce(v):
    """Nonce gültig? (ganze Zahl 0 .. MAX_NONCE, kein bool)"""
    return _is_int(v) and 0 <= v <= MAX_NONCE


def valid_hash(v):
    """Prüfcode gültig? (genau 64 kleine Hex-Zeichen)"""
    return _is_hex(v, 64)


def blank_state(rng):
    """Frischer Zustand: neuer Server- und Client-Seed, Nonce 0."""
    server = new_server_seed(rng)
    client = new_client_seed(rng)
    return {"server": server, "client": client, "nonce": 0}


def clean_state(raw, rng):
    """Gespeicherten Zustand prüfen - Feld für Feld.

    Ungültiger Server-Seed -> neuer Seed, ungültiger Client-Seed -> neuer
    Seed (in dieser Reihenfolge gezogen), ungültige Nonce -> 0. Ist ``raw``
    gar kein dict, entsteht ein frischer Zustand (wie ``blank_state``).
    """
    if not isinstance(raw, dict):
        raw = {}
    server = raw.get("server")
    if not valid_server(server):
        server = new_server_seed(rng)
    client = raw.get("client")
    if not valid_client(client):
        client = new_client_seed(rng)
    nonce = raw.get("nonce")
    if not valid_nonce(nonce):
        nonce = 0
    return {"server": server, "client": client, "nonce": nonce}


def round_rand(game, server, client, nonce):
    """Zufall einer Runde: seedrand.Rand aus (Spiel, Server, Client, Nonce)."""
    return seedrand.Rand(seedrand.seed_from(game, server, client, nonce))


def entry_rand(game, entry):
    """Zufall der Runde zu einem Eintrag aus ``begin``/dem Fairness-Log."""
    return round_rand(game, entry["s"], entry["c"], entry["n"])


def begin(state):
    """Runde beginnt: Eintrag {"n", "s", "c", "h"} mit den Werten dieser Runde.

    "s" (der Server-Seed) darf erst NACH der Runde gezeigt werden, vorher nur
    "h". Direkt danach ``advance`` aufrufen.
    """
    return {"n": state["nonce"], "s": state["server"], "c": state["client"],
            "h": commit(state["server"])}


def advance(state, rng):
    """Nächste Runde vorbereiten: Nonce + 1, neuer Server-Seed (Client bleibt)."""
    state["nonce"] = min(MAX_NONCE, state["nonce"] + 1)
    state["server"] = new_server_seed(rng)
    return state


def new_client(state, rng):
    """Neuer Client-Seed (Knopf im Fenster F; nicht während einer Runde)."""
    state["client"] = new_client_seed(rng)
    return state


def _plain(v, depth=0):
    """Nur JSON-Grundtypen (None, bool, int, str, list, dict mit str-Schlüsseln).

    Gleitkommazahlen sind bewusst NICHT erlaubt (Python, JS und Rust schreiben
    sie nicht überall gleich), ganze Zahlen nur bis 2^53-1. Tiefe höchstens 4.
    """
    if v is None or isinstance(v, (bool, str)):
        return True
    if isinstance(v, int):
        return -MAX_NONCE <= v <= MAX_NONCE
    if depth >= 4:
        return False
    if isinstance(v, list):
        return all(_plain(x, depth + 1) for x in v)
    if isinstance(v, dict):
        return all(isinstance(k, str) and _plain(x, depth + 1) for k, x in v.items())
    return False


def make_entry(entry, outcome):
    """Log-Eintrag {"n", "s", "c", "h", "o"} aus ``begin``-Eintrag + Ergebnis."""
    return {"n": entry["n"], "s": entry["s"], "c": entry["c"], "h": entry["h"],
            "o": outcome if _plain(outcome) else None}


def record(log, entry, outcome):
    """Fertige Runde ins Log (neueste zuletzt, höchstens LOG Einträge)."""
    log.append(make_entry(entry, outcome))
    del log[:-LOG]
    return log


def clean_log(raw):
    """Gespeichertes Fairness-Log prüfen: Liste von höchstens LOG Einträgen.

    Ein Eintrag braucht gültige "n", "s", "c", "h" (sonst fällt er weg); "o"
    (spiel-eigenes Ergebnis) bleibt nur aus JSON-Grundtypen, sonst None.
    Ob der Prüfcode stimmt, sagt ``verify`` - ein falscher bleibt sichtbar (✗).
    """
    if not isinstance(raw, list):
        return []
    out = []
    for e in raw:
        if not isinstance(e, dict):
            continue
        n, s, c, h = e.get("n"), e.get("s"), e.get("c"), e.get("h")
        if not (valid_nonce(n) and valid_server(s) and valid_client(c) and valid_hash(h)):
            continue
        o = e.get("o")
        out.append({"n": n, "s": s, "c": c, "h": h, "o": o if _plain(o) else None})
    return out[-LOG:]


def verify(entry):
    """Stimmt der Prüfcode des Eintrags mit sha256(Server-Seed) überein?"""
    if not isinstance(entry, dict):
        return False
    s, h = entry.get("s"), entry.get("h")
    return valid_server(s) and valid_hash(h) and commit(s) == h


def short(h, n=12):
    """Gekürzter Prüfcode/Seed für die Anzeige: erste n Zeichen + '…'."""
    h = str(h)
    return h if len(h) <= n else h[:n] + "…"
