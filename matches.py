# -*- coding: utf-8 -*-
"""
matches.py
==========
Match-Historie: "Letzte Matches" und "Alle Matches" je Spiel - in einer
EIGENEN Datei neben ``mem.json`` (mem.json wird ständig neu geschrieben, die
Historie kann Tausende Einträge haben)::

    matches.pgzlamamatches

Aufbau (kompaktes JSON, eine Zeile - Rohdaten, keine Datei zum Bearbeiten)::

    {"format": "pygamez.matches", "v": 1, "app": "PyGameZ",
     "updated": "2026-11-26 14:03",
     "profiles": {
       "p1": {
         "crossy": {"n": 57, "w": 0, "l": 0, "d": 0, "best": 212, "sum": 3120,
                    "time": 4210, "first": "2026-11-20 18:02",
                    "list": [["2026-11-26 14:03", "daily", 57, null, 41, 4,
                              {"day": 329, "c": 3, "ch": "chicken"}], ...]}}}}

Ein Eintrag ist ein Array mit genau sieben Feldern::

    [zeit "YYYY-MM-DD HH:MM", modus, punkte, ergebnis, dauer_s, flags, info]

* ``ergebnis``: 1 Sieg, 0 Niederlage, 2 Remis, None = unbekannt (Arcade).
* ``flags``   : Bitmaske, siehe ``F_*`` (Rekord, abgebrochen, Tagesmodus,
  Archiv-Tag).
* ``info``    : kleines dict mit Spiel-Details (nur Ganzzahlen/Text/bool) oder
  None - die Anzeige kennt die Schlüssel aus ``INFO_KEYS``.

Die Liste je Spiel ist auf ``CAP`` Einträge gedeckelt (die ältesten fallen
heraus), die Summen ``n/w/l/d/best/sum/time`` laufen unbegrenzt weiter.
Gleitkommazahlen kommen bewusst nicht vor (Python, JS und Rust schreiben sie
nicht überall gleich) - die Dauer ist in ganzen Sekunden abgerundet.

Gegenstücke: ``web/js/core/matches.js`` (localStorage, Schlüssel
``matches``) und ``rust/crates/pgz-core/src/matches.rs`` (schreibt dieselbe
Datei bytegleich).

Aufgerufen wird ``record`` zentral aus ``main.py`` (Game Over / Verlassen)
bzw. aus ``Game.log_match()`` für Spiele, deren Runden ohne Game Over
enden (Chip-Spiele, Geometry Dash, Labyrinth). Geschrieben wird gedrosselt
(``maybe_flush``) und sofort beim Verlassen/Beenden/Profilwechsel.
"""

import json
import os
import time

import store

FORMAT = "pygamez.matches"
VERSION = 1
EXT = ".pgzlamamatches"
FILENAME = "matches" + EXT

# Gespeicherte Matches je Spiel (Desktop; die Web-Fassung hat dieselbe Zahl,
# pygamez.com eine eigene, im Dashboard einstellbare).
CAP = 250

# Ergebnis eines Matches
WIN, LOSS, DRAW = 1, 0, 2

# Flags (Bitmaske)
F_RECORD = 1       # neuer Highscore
F_ABORT = 2        # verlassen, bevor die Runde zu Ende war
F_DAILY = 4        # Tagesmodus (Tagesstrecke, Tagesrätsel, Tageswort)
F_ARCHIVE = 8      # vergangener Tag aus dem Tages-Archiv

# Bekannte Info-Schlüssel (Anzeige-Reihenfolge) -> Text-Schlüssel
INFO_KEYS = (
    ("day", "match.info.day"),       # Tag (day_index) des Tagesmodus
    ("bet", "match.info.bet"),       # Einsatz (Chips)
    ("win", "match.info.win"),       # Auszahlung (Chips)
    ("num", "match.info.num"),       # gezogene Zahl (Roulette)
    ("cr", "match.info.cr"),         # Absturzpunkt x100 (Lama Crash)
    ("mi", "match.info.mi"),         # Minen (Lama Mines)
    ("op", "match.info.op"),         # aufgedeckte Felder (Lama Mines)
    ("rw", "match.info.rw"),         # Reihen (Lama Plinko)
    ("x", "match.info.x"),           # Multiplikator x100
    ("c", "match.info.c"),           # gesammelte Münzen
    ("ch", "match.info.ch"),         # Figur
    ("vr", "match.info.vr"),         # Variante (Sudoku)
    ("lv", "match.info.lv"),         # Level / Bahn / Stufe / Schwierigkeit
    ("ln", "match.info.ln"),         # erreichte Spur (Lama Crossing)
    ("st", "match.info.st"),         # Sterne
    ("tm", "match.info.tm"),         # Zeit in Sekunden (Rätsel)
    ("g", "match.info.g"),           # Versuche (Wordle)
    ("wd", "match.info.wd"),         # gelöste Wörter (Wordle Endlos)
    ("len", "match.info.len"),       # Wortlänge (Wordle)
    ("vs", "match.info.vs"),         # Gegner (Profilname / KI)
)

# Schreib-Drosselung (Plinko & Co. können viele Runden je Sekunde liefern).
_FLUSH_EVERY = 5.0

# Kurze Runden, die man verlässt, ohne etwas erreicht zu haben, zählen nicht.
MIN_ABORT_SECONDS = 5

_doc = None
_dirty = False
_last_flush = 0.0

_TOTALS = (("n", 0), ("w", 0), ("l", 0), ("d", 0), ("best", None),
           ("sum", 0), ("time", 0), ("first", ""))
_MAX_INT = 2 ** 53 - 1


def _path():
    return store.path_for(FILENAME)


def _now_str():
    return time.strftime("%Y-%m-%d %H:%M")


# ---------------------------------------------------------------------------
#  Bereinigen
# ---------------------------------------------------------------------------

def _is_int(v):
    return isinstance(v, int) and not isinstance(v, bool)


def _clamp_int(v, lo=-_MAX_INT, hi=_MAX_INT, default=0):
    if not _is_int(v):
        return default
    return max(lo, min(hi, v))


def _clean_info(raw):
    """Nur flache Werte (int, str bis 40 Zeichen, bool) - sonst None."""
    if not isinstance(raw, dict):
        return None
    out = {}
    for key, value in raw.items():
        if not isinstance(key, str) or len(key) > 8:
            continue
        if isinstance(value, bool):
            out[key] = value
        elif _is_int(value):
            out[key] = _clamp_int(value)
        elif isinstance(value, str):
            out[key] = value[:40]
    return out or None


def _clean_entry(raw):
    if not isinstance(raw, list) or len(raw) < 6:
        return None
    stamp, mode, score, result, dur, flags = raw[:6]
    info = raw[6] if len(raw) > 6 else None
    if not isinstance(stamp, str):
        return None
    if result not in (WIN, LOSS, DRAW) or isinstance(result, bool):
        result = None
    return [stamp[:16], str(mode)[:24] if isinstance(mode, str) else "",
            _clamp_int(score), result, _clamp_int(dur, 0),
            _clamp_int(flags, 0, 255), _clean_info(info)]


def _blank_game():
    return {key: default for key, default in _TOTALS} | {"list": []}


def _clean_game(raw):
    game = _blank_game()
    if not isinstance(raw, dict):
        return game
    for key, default in _TOTALS:
        value = raw.get(key, default)
        if key == "first":
            game[key] = value[:16] if isinstance(value, str) else ""
        elif key == "best":
            game[key] = _clamp_int(value) if _is_int(value) else None
        else:
            game[key] = _clamp_int(value, 0)
    items = raw.get("list")
    if isinstance(items, list):
        for item in items:
            entry = _clean_entry(item)
            if entry is not None:
                game["list"].append(entry)
        game["list"] = game["list"][-CAP:]
    return game


def _blank_doc():
    return {"format": FORMAT, "v": VERSION, "app": "PyGameZ",
            "updated": _now_str(), "profiles": {}}


def _read_file(path):
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, dict) else None
    except (OSError, json.JSONDecodeError, ValueError):
        return None


def _load():
    """Lädt die Datei einmalig (fehlt sie, ist die Historie eben leer)."""
    global _doc
    if _doc is not None:
        return _doc
    path = _path()
    raw = _read_file(path)
    if raw is None:
        raw = _read_file(path + ".bak")
    doc = _blank_doc()
    if isinstance(raw, dict) and raw.get("format") == FORMAT \
            and isinstance(raw.get("profiles"), dict):
        for pid, prof in raw["profiles"].items():
            if not isinstance(prof, dict):
                continue
            doc["profiles"][str(pid)] = {str(k): _clean_game(v)
                                         for k, v in prof.items()}
    _doc = doc
    return _doc


def _profile(pid=None):
    doc = _load()
    pid = pid or store.active_id()
    prof = doc["profiles"].get(pid)
    if prof is None:
        prof = {}
        doc["profiles"][pid] = prof
    return prof


def _game(key, pid=None, create=True):
    prof = _profile(pid)
    game = prof.get(key)
    if game is None and create:
        game = _blank_game()
        prof[key] = game
    return game


# ---------------------------------------------------------------------------
#  Schreiben
# ---------------------------------------------------------------------------

def _write(path, obj):
    """Kompakt + atomar (wie replay._write_raw), Fehler werden geschluckt."""
    tmp = path + ".tmp"
    try:
        text = json.dumps(obj, ensure_ascii=False, separators=(",", ":"))
        with open(tmp, "w", encoding="utf-8") as f:
            f.write(text)
            f.flush()
            try:
                os.fsync(f.fileno())
            except OSError:
                pass
        if os.path.exists(path):
            try:
                os.replace(path, path + ".bak")
            except OSError:
                pass
        os.replace(tmp, path)
        return True
    except (OSError, TypeError, ValueError):
        try:
            if os.path.exists(tmp):
                os.remove(tmp)
        except OSError:
            pass
        return False


def flush():
    """Schreibt ungespeicherte Matches sofort."""
    global _dirty, _last_flush
    if not _dirty or _doc is None:
        return
    out = {"format": FORMAT, "v": VERSION, "app": "PyGameZ",
           "updated": _now_str(), "profiles": {}}
    for pid, prof in _doc["profiles"].items():
        games = {key: game for key, game in prof.items() if game.get("n")}
        if games:
            out["profiles"][pid] = games
    _write(_path(), out)
    _dirty = False
    _last_flush = time.monotonic()


def maybe_flush():
    """Gedrosselt schreiben (höchstens alle _FLUSH_EVERY Sekunden)."""
    if _dirty and time.monotonic() - _last_flush >= _FLUSH_EVERY:
        flush()


def reset_cache():
    """Profilwechsel: Offenes wegschreiben (die Datei enthält alle Profile)."""
    flush()


store.on_switch(reset_cache)


# ---------------------------------------------------------------------------
#  Aufzeichnen
# ---------------------------------------------------------------------------

def record(key, mode="", score=0, result=None, duration=0, flags=0, info=None,
           pid=None, replace=False):
    """Hält ein Match fest (neueste zuletzt).

    ``replace=True`` ersetzt das zuletzt gespeicherte Match dieses Spiels
    (Rückgängig nach Game Over: dieselbe Partie lief weiter). Gibt den
    gespeicherten Eintrag zurück.
    """
    global _dirty
    game = _game(str(key), pid)
    entry = _clean_entry([_now_str(), mode or "", score if _is_int(score) else
                          int(score or 0), result, int(max(0, duration or 0)),
                          flags, info])
    if replace and game["list"]:
        # Dieselbe Partie lief weiter: altes Match raus, Spielzeit addieren.
        old = game["list"].pop()
        _sub_totals(game, old)
        entry[4] = _clamp_int(entry[4] + old[4], 0)
    _add_totals(game, entry)
    game["list"].append(entry)
    if len(game["list"]) > CAP:
        del game["list"][:len(game["list"]) - CAP]
    _dirty = True
    return entry


def _add_totals(game, entry):
    _, _, score, result, dur, _, _ = entry
    game["n"] = min(_MAX_INT, game["n"] + 1)
    if result == WIN:
        game["w"] += 1
    elif result == LOSS:
        game["l"] += 1
    elif result == DRAW:
        game["d"] += 1
    if game["best"] is None or score > game["best"]:
        game["best"] = score
    game["sum"] = _clamp_int(game["sum"] + score)
    game["time"] = _clamp_int(game["time"] + dur, 0)
    if not game["first"]:
        game["first"] = entry[0]


def _sub_totals(game, entry):
    """Nimmt ein Match aus den Summen (der Bestwert bleibt - er war erreicht)."""
    _, _, score, result, dur, _, _ = entry
    game["n"] = max(0, game["n"] - 1)
    if result == WIN:
        game["w"] = max(0, game["w"] - 1)
    elif result == LOSS:
        game["l"] = max(0, game["l"] - 1)
    elif result == DRAW:
        game["d"] = max(0, game["d"] - 1)
    game["sum"] = _clamp_int(game["sum"] - score)
    game["time"] = max(0, game["time"] - dur)


# ---------------------------------------------------------------------------
#  Abfragen (für den Matches-Screen)
# ---------------------------------------------------------------------------

def entries(key, pid=None):
    """Alle gespeicherten Matches eines Spiels, NEUESTE ZUERST (Kopien)."""
    game = _game(str(key), pid, create=False)
    if not game:
        return []
    return [list(e) for e in reversed(game["list"])]


def recent(key, n=10, pid=None):
    """Die letzten n Matches, neueste zuerst."""
    return entries(key, pid)[:n]


def totals(key, pid=None):
    """Summen eines Spiels: n, w, l, d, best, sum, time, first, stored."""
    game = _game(str(key), pid, create=False)
    out = {k: d for k, d in _TOTALS}
    out["stored"] = 0
    if game:
        for k, _ in _TOTALS:
            out[k] = game[k]
        out["stored"] = len(game["list"])
    return out


def modes_of(key, pid=None):
    """Modi, die in den gespeicherten Matches vorkommen (erste Sichtung zuerst)."""
    seen = []
    game = _game(str(key), pid, create=False)
    for e in (game or {}).get("list", ()):
        if e[1] not in seen:
            seen.append(e[1])
    return seen


def drop_profile(pid):
    """Historie eines gelöschten Profils entfernen."""
    global _dirty
    doc = _load()
    if doc["profiles"].pop(pid, None) is not None:
        _dirty = True
        flush()


def clear_game(key, pid=None):
    """Historie eines Spiels leeren (Knopf im Matches-Screen)."""
    global _dirty
    prof = _profile(pid)
    if prof.pop(str(key), None) is not None:
        _dirty = True
        flush()


def format_duration(seconds):
    """Sekunden -> '1:05:09' / '3:07' / '0:42'."""
    seconds = max(0, int(seconds or 0))
    h, rest = divmod(seconds, 3600)
    m, s = divmod(rest, 60)
    if h:
        return "%d:%02d:%02d" % (h, m, s)
    return "%d:%02d" % (m, s)
