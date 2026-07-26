# -*- coding: utf-8 -*-
"""
stats.py
========
Persistente Spielerstatistiken für alle Spiele der Sammlung.

Die Daten liegen im Abschnitt ``stats`` der gemeinsamen Datei ``mem.json``
(siehe store.py), ein Eintrag je Spiel (Schlüssel = ``highscore_key``):

    "stats": {
      "snake": { "plays": 12,          # gestartete Partien (inkl. Neustarts)
                 "time": 843.2,        # Spielzeit in Sekunden (ohne Pause)
                 "wins": 3,            # Siege    (nur Spiele, die es melden)
                 "losses": 1,          # Niederlagen (dito)
                 "records": 4,         # gebrochene Highscore-Rekorde
                 "last": "2026-07-26 14:03" },   # letzter Start
      ...
    }

Aufgerufen wird das Modul von der zentralen Schleife in main.py
(Partie-Start, Spielzeit, Rekorde) sowie über ``Game.report_result()`` aus
den Spielen selbst (Sieg/Niederlage). Damit nicht bei jedem Frame die ganze
mem.json geschrieben wird, sammelt das Modul die Änderungen im Speicher und
schreibt sie gedrosselt (``maybe_flush``) bzw. bei wichtigen Ereignissen
(Partie-Start, Rekord, Beenden) sofort.

Alle Funktionen sind fehlertolerant: ohne Schreibrechte läuft das Spiel
weiter - die Statistik ist dann eben nur für die Sitzung.
"""

import time

import store

_SECTION = "stats"

# Spielzeit-Änderungen frühestens alle X Sekunden auf die Platte schreiben.
_FLUSH_EVERY = 20.0

_data = None          # {key: {...}} - In-Memory-Kopie des stats-Abschnitts
_dirty = False        # ungeschriebene Änderungen vorhanden?
_last_flush = 0.0     # time.monotonic() des letzten Schreibvorgangs

# Felder eines Spiel-Eintrags mit Standardwerten (Reihenfolge = JSON-Ausgabe).
_FIELDS = (("plays", 0), ("time", 0.0), ("wins", 0), ("losses", 0),
           ("records", 0), ("last", ""))


def _now_str():
    return time.strftime("%Y-%m-%d %H:%M")


def _load():
    """Lädt den stats-Abschnitt einmalig in den Speicher (mit Bereinigung)."""
    global _data
    if _data is None:
        raw = store.load_section(_SECTION)
        _data = {}
        for key, entry in raw.items():
            if not isinstance(entry, dict):
                continue
            clean = {}
            for field, default in _FIELDS:
                value = entry.get(field, default)
                try:
                    if isinstance(default, int):
                        value = max(0, int(value))
                    elif isinstance(default, float):
                        value = max(0.0, float(value))
                    else:
                        value = str(value)
                except (TypeError, ValueError):
                    value = default
                clean[field] = value
            _data[str(key)] = clean
    return _data


def _entry(key):
    """Liefert den (ggf. neu angelegten) Eintrag für ein Spiel."""
    data = _load()
    entry = data.get(key)
    if entry is None:
        entry = {field: default for field, default in _FIELDS}
        data[key] = entry
    return entry


def _mark_dirty():
    global _dirty
    _dirty = True


def flush(force=True):
    """Schreibt ungespeicherte Änderungen nach mem.json."""
    global _dirty, _last_flush
    if not _dirty:
        return
    data = _load()
    # Spielzeit gerundet speichern (Zehntel reichen, hält die Datei lesbar).
    out = {}
    for key, entry in data.items():
        entry = dict(entry)
        entry["time"] = round(float(entry.get("time", 0.0)), 1)
        out[key] = entry
    store.save_section(_SECTION, out)
    _dirty = False
    _last_flush = time.monotonic()


def maybe_flush():
    """Schreibt gedrosselt (höchstens alle _FLUSH_EVERY Sekunden)."""
    if _dirty and time.monotonic() - _last_flush >= _FLUSH_EVERY:
        flush()


# ----- Ereignisse (von main.py / game_base.py aufgerufen) -----------------

def game_started(key):
    """Eine neue Partie beginnt (Spielstart oder Neustart nach Game Over)."""
    entry = _entry(key)
    entry["plays"] += 1
    entry["last"] = _now_str()
    _mark_dirty()
    flush()


def add_playtime(key, seconds):
    """Akkumuliert aktive Spielzeit (läuft nicht bei Pause/Game Over)."""
    if seconds <= 0:
        return
    _entry(key)["time"] += float(seconds)
    _mark_dirty()


def record_result(key, won):
    """Ein Spiel meldet Sieg (True) oder Niederlage (False) des Spielers."""
    entry = _entry(key)
    entry["wins" if won else "losses"] += 1
    _mark_dirty()
    flush()


def record_broken(key):
    """Ein neuer Highscore-Rekord wurde aufgestellt."""
    _entry(key)["records"] += 1
    _mark_dirty()
    flush()


# ----- Abfragen (für Erfolge und den Statistik-Screen) --------------------

def get(key):
    """Statistik-Eintrag eines Spiels (Kopie; leere Werte, wenn nie gespielt)."""
    data = _load()
    entry = data.get(key)
    if entry is None:
        return {field: default for field, default in _FIELDS}
    return dict(entry)


def all_stats():
    """Alle Einträge als dict {key: eintrag} (Kopie)."""
    return {key: dict(entry) for key, entry in _load().items()}


def totals():
    """Gesamtwerte über alle Spiele (für Erfolge und die Übersichtskarten).

    Liefert ein dict mit:
      plays, time, wins, losses, records : Summen
      distinct                           : Anzahl verschiedener gespielter Spiele
      win_games                          : Anzahl Spiele mit mindestens 1 Sieg
      favorite                           : key mit der meisten Spielzeit (oder None)
    """
    data = _load()
    total = dict(plays=0, time=0.0, wins=0, losses=0, records=0,
                 distinct=0, win_games=0, favorite=None)
    best_time = 0.0
    for key, entry in data.items():
        plays = entry.get("plays", 0)
        total["plays"] += plays
        total["time"] += entry.get("time", 0.0)
        total["wins"] += entry.get("wins", 0)
        total["losses"] += entry.get("losses", 0)
        total["records"] += entry.get("records", 0)
        # "Ausprobiert" heisst: mindestens eine Partie gestartet/gespielt.
        if plays > 0 or entry.get("time", 0.0) > 0:
            total["distinct"] += 1
        if entry.get("wins", 0) > 0:
            total["win_games"] += 1
        if entry.get("time", 0.0) > best_time:
            best_time = entry.get("time", 0.0)
            total["favorite"] = key
    return total


def format_time(seconds):
    """Sekunden -> kompakte Anzeige '3h 24m' / '12m' / '45s' (sprachneutral)."""
    seconds = max(0, int(seconds))
    h, rest = divmod(seconds, 3600)
    m, s = divmod(rest, 60)
    if h:
        return f"{h}h {m:02d}m"
    if m:
        return f"{m}m {s:02d}s"
    return f"{s}s"
