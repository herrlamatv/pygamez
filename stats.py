# -*- coding: utf-8 -*-
"""
stats.py
========
Persistente Spielerstatistiken - seit dem Statistik-Update in einer EIGENEN
Datei neben ``mem.json``::

    stats.pgzlamastats

Die Datei ist ganz normales JSON (wie die Replay-/Bahn-Dateien mit Kopfzeile),
enthält die Daten ALLER Profile und lässt sich weitergeben bzw. mit der
Web-Version austauschen::

    {"format": "pygamez.stats", "v": 1, "app": "PyGameZ",
     "updated": "2026-09-21 02:14",
     "profiles": {
       "p1": {
         "games":  {"snake": {"plays": 12, "time": 843.2, "wins": 3,
                              "losses": 1, "records": 4, "best": 560,
                              "best_at": "...", "first": "...", "last": "...",
                              "longest": 612.0}},
         "days":   {"2026-09-19": {"t": 1234.5, "p": 12, "w": 3, "a": 2,
                                   "g": {"snake": 800.0}}},
         "hours":  [0, 0, ...],           # 24 Werte: Spielzeit je Tagesstunde
         "series": {"snake": [["2026-09-19 14:03", 560, 1], ...]},  # 1 = Rekord
         "events": [{"t": "...", "k": "ach", "id": "tetris_four", "p": 60},
                    {"t": "...", "k": "rec", "g": "snake", "v": 560},
                    {"t": "...", "k": "lvl", "v": 12},
                    {"t": "...", "k": "duel", "g": "pong", "o": "p2", "w": 1}],
         "duel":   {"p2": {"pong": {"w": 3, "l": 1}}},
         "streak": {"cur": 4, "best": 11, "first_day": "...", "last_day": "..."},
         "points": 1180, "level": 14, "longest": 3705.0}}}

Damit die Datei nicht endlos wächst, sind die Verläufe gedeckelt
(``MAX_EVENTS``/``MAX_SERIES``/``MAX_DAYS``) - die Summen in ``games``,
``days`` und ``hours`` bleiben davon unberührt.

Aufgerufen wird das Modul von der zentralen Schleife in main.py
(Partie-Start, Spielzeit, Rekorde) sowie über ``Game.report_result()`` /
``Game.report_duel()`` aus den Spielen selbst. Damit nicht bei jedem Frame die
ganze Datei geschrieben wird, sammelt das Modul die Änderungen im Speicher und
schreibt sie gedrosselt (``maybe_flush``) bzw. bei wichtigen Ereignissen
(Partie-Start, Rekord, Beenden) sofort.

Alle Funktionen sind fehlertolerant: ohne Schreibrechte läuft das Spiel
weiter - die Statistik ist dann eben nur für die Sitzung.
"""

import json
import os
import time

import store

# Kopfzeile der Datei (wie replay.py/ugc.py: daran erkennt der Import sie).
FORMAT = "pygamez.stats"
VERSION = 1
EXT = ".pgzlamastats"
FILENAME = "stats" + EXT
IMPORT_EXTS = (EXT, ".json")

# Spielzeit-Änderungen frühestens alle X Sekunden auf die Platte schreiben.
_FLUSH_EVERY = 20.0

# Obergrenzen gegen unbegrenztes Wachstum.
MAX_EVENTS = 4000      # Ereignis-Verlauf je Profil
MAX_SERIES = 250       # Punktwerte je Spiel (Rekorde bleiben immer erhalten)
MAX_DAYS = 1100        # gut drei Jahre Kalender

_doc = None           # komplette Datei im Speicher (alle Profile)
_dirty = False        # ungeschriebene Änderungen vorhanden?
_last_flush = 0.0     # time.monotonic() des letzten Schreibvorgangs

# Zwischenspeicher für die Heißpfade (add_playtime läuft in JEDEM Frame).
_day_cache = (None, None, None)   # (profil-id, datum, tages-dict)
_sess_key = None                  # Spiel der laufenden Sitzung
_sess_time = 0.0                  # Spielzeit der laufenden Sitzung

# Felder eines Spiel-Eintrags mit Standardwerten (Reihenfolge = JSON-Ausgabe).
_FIELDS = (("plays", 0), ("time", 0.0), ("wins", 0), ("losses", 0),
           ("records", 0), ("best", 0), ("best_at", ""), ("first", ""),
           ("last", ""), ("longest", 0.0))


def _path():
    return store.path_for(FILENAME)


def _now_str():
    return time.strftime("%Y-%m-%d %H:%M")


def _today_str():
    return time.strftime("%Y-%m-%d")


# ---------------------------------------------------------------------------
#  Laden / Anlegen / Speichern
# ---------------------------------------------------------------------------

def _blank_game():
    return {field: default for field, default in _FIELDS}


def _blank_profile():
    return {"games": {}, "days": {}, "hours": [0.0] * 24, "series": {},
            "events": [], "duel": {},
            "streak": {"cur": 0, "best": 0, "first_day": "", "last_day": ""},
            "points": 0, "level": 1, "longest": 0.0}


def _blank_doc():
    return {"format": FORMAT, "v": VERSION, "app": "PyGameZ",
            "updated": _now_str(), "profiles": {}}


def _num(value, default=0.0, lo=0.0):
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return default
    return max(lo, float(value))


def _clean_game(raw):
    entry = _blank_game()
    if not isinstance(raw, dict):
        return entry
    for field, default in _FIELDS:
        value = raw.get(field, default)
        try:
            if isinstance(default, bool):
                value = bool(value)
            elif isinstance(default, int):
                value = max(0, int(value))
            elif isinstance(default, float):
                value = max(0.0, float(value))
            else:
                value = str(value)
        except (TypeError, ValueError):
            value = default
        entry[field] = value
    return entry


def _clean_profile(raw):
    """Macht aus beliebigem JSON einen benutzbaren Profil-Eintrag."""
    prof = _blank_profile()
    if not isinstance(raw, dict):
        return prof
    games = raw.get("games")
    if isinstance(games, dict):
        for key, entry in games.items():
            prof["games"][str(key)] = _clean_game(entry)
    days = raw.get("days")
    if isinstance(days, dict):
        for day, entry in days.items():
            if not isinstance(entry, dict):
                continue
            clean = {"t": _num(entry.get("t")), "p": int(_num(entry.get("p"))),
                     "w": int(_num(entry.get("w"))), "a": int(_num(entry.get("a"))),
                     "g": {}}
            per_game = entry.get("g")
            if isinstance(per_game, dict):
                for key, value in per_game.items():
                    clean["g"][str(key)] = _num(value)
            prof["days"][str(day)[:10]] = clean
    hours = raw.get("hours")
    if isinstance(hours, list):
        for i, value in enumerate(hours[:24]):
            prof["hours"][i] = _num(value)
    series = raw.get("series")
    if isinstance(series, dict):
        for key, points in series.items():
            if not isinstance(points, list):
                continue
            clean = []
            for item in points:
                if (isinstance(item, list) and len(item) >= 2
                        and isinstance(item[0], str)):
                    clean.append([item[0][:16], int(_num(item[1])),
                                  1 if len(item) > 2 and item[2] else 0])
            if clean:
                prof["series"][str(key)] = clean[-MAX_SERIES:]
    events = raw.get("events")
    if isinstance(events, list):
        for item in events:
            if isinstance(item, dict) and isinstance(item.get("t"), str) \
                    and isinstance(item.get("k"), str):
                prof["events"].append(item)
        prof["events"] = prof["events"][-MAX_EVENTS:]
    duel = raw.get("duel")
    if isinstance(duel, dict):
        for other, games_ in duel.items():
            if not isinstance(games_, dict):
                continue
            slot = {}
            for key, wl in games_.items():
                if isinstance(wl, dict):
                    slot[str(key)] = {"w": int(_num(wl.get("w"))),
                                      "l": int(_num(wl.get("l")))}
            if slot:
                prof["duel"][str(other)] = slot
    streak = raw.get("streak")
    if isinstance(streak, dict):
        prof["streak"] = {"cur": int(_num(streak.get("cur"))),
                          "best": int(_num(streak.get("best"))),
                          "first_day": str(streak.get("first_day") or "")[:10],
                          "last_day": str(streak.get("last_day") or "")[:10]}
    prof["points"] = int(_num(raw.get("points")))
    prof["level"] = max(1, int(_num(raw.get("level"), 1.0)))
    prof["longest"] = _num(raw.get("longest"))
    return prof


def _read_file(path):
    """Liest eine Statistik-Datei (None, wenn sie fehlt/unlesbar ist)."""
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, dict) else None
    except (OSError, json.JSONDecodeError, ValueError):
        return None


def _load():
    """Lädt die Statistik-Datei einmalig in den Speicher (mit Migration)."""
    global _doc, _dirty
    if _doc is not None:
        return _doc
    path = _path()
    raw = _read_file(path)
    if raw is None:
        raw = _read_file(path + ".bak")
    doc = _blank_doc()
    if isinstance(raw, dict) and isinstance(raw.get("profiles"), dict):
        for pid, prof in raw["profiles"].items():
            doc["profiles"][str(pid)] = _clean_profile(prof)
    _doc = doc
    if raw is None:
        # Erster Start mit der neuen Datei: Altbestand aus mem.json holen.
        try:
            _migrate_from_mem()
        except Exception:
            # Die Statistik darf den Start nie verhindern.
            pass
        _dirty = True
        flush()
    return _doc


def _profile(pid=None):
    """Statistik-Eintrag eines Profils (wird bei Bedarf angelegt)."""
    doc = _load()
    pid = pid or store.active_id()
    prof = doc["profiles"].get(pid)
    if prof is None:
        prof = _blank_profile()
        doc["profiles"][pid] = prof
    return prof


def _mark_dirty():
    global _dirty
    _dirty = True


def flush(force=True):
    """Schreibt ungespeicherte Änderungen nach stats.pgzlamastats."""
    global _dirty, _last_flush
    if not _dirty or _doc is None:
        return
    out = {"format": FORMAT, "v": VERSION, "app": "PyGameZ",
           "updated": _now_str(), "profiles": {}}
    for pid, prof in _doc["profiles"].items():
        entry = dict(prof)
        # Zeiten gerundet speichern (Zehntel reichen, hält die Datei lesbar).
        entry["games"] = {}
        for key, game in prof["games"].items():
            game = dict(game)
            game["time"] = round(float(game.get("time", 0.0)), 1)
            game["longest"] = round(float(game.get("longest", 0.0)), 1)
            entry["games"][key] = game
        entry["days"] = {}
        for day, value in prof["days"].items():
            value = dict(value)
            value["t"] = round(float(value.get("t", 0.0)), 1)
            value["g"] = {k: round(float(v), 1) for k, v in value["g"].items()}
            entry["days"][day] = value
        entry["hours"] = [round(float(v), 1) for v in prof["hours"]]
        entry["longest"] = round(float(prof.get("longest", 0.0)), 1)
        out["profiles"][pid] = entry
    store.write_json_atomic(_path(), out)
    _dirty = False
    _last_flush = time.monotonic()


def maybe_flush():
    """Schreibt gedrosselt (höchstens alle _FLUSH_EVERY Sekunden)."""
    if _dirty and time.monotonic() - _last_flush >= _FLUSH_EVERY:
        flush()


def reset_cache():
    """Leert die Profil-Zwischenspeicher (nach einem Profilwechsel)."""
    global _day_cache, _sess_key, _sess_time
    end_session()
    _day_cache = (None, None, None)
    _sess_key, _sess_time = None, 0.0


store.on_switch(reset_cache)


# ---------------------------------------------------------------------------
#  Migration aus der alten mem.json
# ---------------------------------------------------------------------------

def _migrate_from_mem():
    """Übernimmt den alten ``stats``-Abschnitt (und die Erfolgs-Zeitstempel).

    Läuft genau einmal - beim ersten Start ohne stats.pgzlamastats. Der alte
    Abschnitt wird danach aus mem.json entfernt, damit es nur noch EINEN Ort
    für Statistiken gibt.
    """
    data = store.load()
    players = data.get(store._PLAYERS, {})
    touched = False
    for pid in list(players.keys()):
        prof = _profile(pid)
        old = players[pid].get("stats")
        if isinstance(old, dict):
            for key, entry in old.items():
                if not isinstance(entry, dict):
                    continue
                game = _clean_game(entry)
                game["last"] = str(entry.get("last", ""))[:16]
                game["first"] = game["first"] or game["last"]
                prof["games"][str(key)] = game
                day = game["last"][:10]
                if day:
                    slot = _day_entry(prof, day)
                    slot["p"] += max(0, game["plays"])
                    slot["t"] += 0.0
            players[pid].pop("stats", None)
            touched = True
        # Bestwerte als Startpunkt der Punkte-Serien.
        scores = players[pid].get("highscores")
        if isinstance(scores, dict):
            for key, value in scores.items():
                try:
                    value = int(value)
                except (TypeError, ValueError):
                    continue
                game = prof["games"].setdefault(str(key), _blank_game())
                if value > game["best"]:
                    game["best"] = value
                    game["best_at"] = game["last"]
        # Erfolge: Freischalt-Zeitpunkte in den Verlauf übernehmen.
        unlocked = players[pid].get("achievements")
        if isinstance(unlocked, dict):
            items = sorted(((str(v)[:16], str(k)) for k, v in unlocked.items()))
            for stamp, ach_id in items:
                prof["events"].append({"t": stamp, "k": "ach", "id": ach_id,
                                       "p": 0})
                day = stamp[:10]
                if day:
                    _day_entry(prof, day)["a"] += 1
            prof["events"] = prof["events"][-MAX_EVENTS:]
        _recalc_streak(prof)
    if touched:
        store.save(data)


# ---------------------------------------------------------------------------
#  Ereignisse (von main.py / game_base.py aufgerufen)
# ---------------------------------------------------------------------------

def _entry(key, pid=None):
    """Liefert den (ggf. neu angelegten) Eintrag für ein Spiel."""
    prof = _profile(pid)
    entry = prof["games"].get(key)
    if entry is None:
        entry = _blank_game()
        prof["games"][key] = entry
    return entry


def _day_entry(prof, day):
    slot = prof["days"].get(day)
    if slot is None:
        slot = {"t": 0.0, "p": 0, "w": 0, "a": 0, "g": {}}
        prof["days"][day] = slot
        if len(prof["days"]) > MAX_DAYS:
            for old in sorted(prof["days"])[:-MAX_DAYS]:
                prof["days"].pop(old, None)
    return slot


def _today_entry():
    """Tages-Eintrag des aktiven Profils (gecacht - läuft pro Frame)."""
    global _day_cache
    pid = store.active_id()
    day = _today_str()
    if _day_cache[0] == pid and _day_cache[1] == day and _day_cache[2] is not None:
        return _day_cache[2]
    slot = _day_entry(_profile(pid), day)
    _day_cache = (pid, day, slot)
    return slot


def _recalc_streak(prof):
    """Tagesserie (aktuell/längste) aus der Tagesliste neu berechnen."""
    days = sorted(d for d in prof["days"] if len(d) == 10)
    if not days:
        prof["streak"] = {"cur": 0, "best": 0, "first_day": "", "last_day": ""}
        return
    import datetime
    best = cur = 1
    prev = None
    for day in days:
        try:
            date = datetime.date.fromisoformat(day)
        except ValueError:
            continue
        if prev is not None:
            cur = cur + 1 if (date - prev).days == 1 else 1
        prev = date
        best = max(best, cur)
    today = datetime.date.today()
    if prev is not None and (today - prev).days > 1:
        cur = 0
    prof["streak"] = {"cur": cur, "best": best,
                      "first_day": days[0], "last_day": days[-1]}


def game_started(key):
    """Eine neue Partie beginnt (Spielstart oder Neustart nach Game Over)."""
    end_session()
    entry = _entry(key)
    entry["plays"] += 1
    entry["last"] = _now_str()
    if not entry["first"]:
        entry["first"] = entry["last"]
    slot = _today_entry()
    slot["p"] += 1
    _recalc_streak(_profile())
    _mark_dirty()
    flush()


def add_playtime(key, seconds):
    """Akkumuliert aktive Spielzeit (läuft nicht bei Pause/Game Over)."""
    global _sess_key, _sess_time
    if seconds <= 0:
        return
    seconds = float(seconds)
    if _sess_key != key:
        end_session()
        _sess_key = key
    _sess_time += seconds
    _entry(key)["time"] += seconds
    slot = _today_entry()
    slot["t"] += seconds
    slot["g"][key] = slot["g"].get(key, 0.0) + seconds
    prof = _profile()
    prof["hours"][time.localtime().tm_hour] += seconds
    _mark_dirty()


def end_session(pid=None):
    """Schließt die laufende Sitzung ab (längste Sitzung merken)."""
    global _sess_key, _sess_time
    if _sess_key is None or _sess_time <= 0:
        _sess_key, _sess_time = None, 0.0
        return 0.0
    prof = _profile(pid)
    entry = prof["games"].get(_sess_key)
    if entry is not None and _sess_time > entry.get("longest", 0.0):
        entry["longest"] = _sess_time
    if _sess_time > prof.get("longest", 0.0):
        prof["longest"] = _sess_time
    length = _sess_time
    _sess_key, _sess_time = None, 0.0
    _mark_dirty()
    return length


def session_time():
    """Spielzeit der laufenden Sitzung in Sekunden (für Marathon-Erfolge)."""
    return _sess_time


def record_result(key, won):
    """Ein Spiel meldet Sieg (True) oder Niederlage (False) des Spielers."""
    entry = _entry(key)
    entry["wins" if won else "losses"] += 1
    if won:
        _today_entry()["w"] += 1
    _mark_dirty()
    flush()


def record_broken(key):
    """Ein neuer Highscore-Rekord wurde aufgestellt."""
    _entry(key)["records"] += 1
    _mark_dirty()
    flush()


def score_result(key, score, record=False):
    """Ergebnis einer Partie: Punktwert in Serie + Verlauf schreiben.

    Wird bei Game Over aus main.py gerufen (auch ohne Rekord) - daraus
    entstehen der Punkteverlauf je Spiel und die Rekord-Einträge im Verlauf.
    """
    score = int(score or 0)
    prof = _profile()
    entry = _entry(key)
    stamp = _now_str()
    if score > entry["best"]:
        entry["best"] = score
        entry["best_at"] = stamp
    if score > 0 or record:
        series = prof["series"].setdefault(key, [])
        series.append([stamp, score, 1 if record else 0])
        if len(series) > MAX_SERIES:
            # Beim Kappen die Rekorde behalten - sie sind die Geschichte.
            keep = [p for p in series if p[2]]
            rest = [p for p in series if not p[2]][-(MAX_SERIES - len(keep)):]
            prof["series"][key] = sorted(keep + rest, key=lambda p: p[0])
    if record:
        _add_event(prof, {"t": stamp, "k": "rec", "g": key, "v": score})
    _mark_dirty()


def record_duel(key, opponent, won, pid=None):
    """Ergebnis eines lokalen Duells (Profil gegen Profil) festhalten.

    'opponent' ist die Profil-id des Gegners (oder profiles.GUEST),
    'pid' das buchende Profil (Standard: das aktive).
    'won' = True, wenn 'pid' gewonnen hat.
    """
    prof = _profile(pid)
    slot = prof["duel"].setdefault(str(opponent), {}).setdefault(
        key, {"w": 0, "l": 0})
    slot["w" if won else "l"] += 1
    _add_event(prof, {"t": _now_str(), "k": "duel", "g": key,
                      "o": str(opponent), "w": 1 if won else 0})
    _mark_dirty()
    flush()


def duel_total(pid=None):
    """Anzahl aller ausgetragenen Duelle eines Profils."""
    prof = _profile(pid)
    return sum(wl["w"] + wl["l"]
               for games in prof["duel"].values() for wl in games.values())


def _add_event(prof, event):
    prof["events"].append(event)
    if len(prof["events"]) > MAX_EVENTS:
        del prof["events"][:len(prof["events"]) - MAX_EVENTS]


# ----- Erfolgspunkte & Level ---------------------------------------------

def log_ach(ach_id, points, stamp=None):
    """Trägt einen freigeschalteten Erfolg in Verlauf und Tageszähler ein."""
    prof = _profile()
    stamp = stamp or _now_str()
    _add_event(prof, {"t": stamp, "k": "ach", "id": str(ach_id),
                      "p": int(points)})
    day = stamp[:10]
    if day:
        _day_entry(prof, day)["a"] += 1
    _mark_dirty()


def log_level(level, stamp=None):
    """Hält einen Level-Aufstieg im Verlauf fest."""
    prof = _profile()
    _add_event(prof, {"t": stamp or _now_str(), "k": "lvl", "v": int(level)})
    _mark_dirty()
    flush()


def set_points(points, pid=None):
    """Setzt den Punktestand neu (nach dem stillen Abgleich beim Start)."""
    import levels
    prof = _profile(pid)
    prof["points"] = max(0, int(points))
    prof["level"] = levels.level_for(prof["points"])
    _mark_dirty()


def points_of(pid=None):
    return _profile(pid)["points"]


def level_of(pid=None):
    """(Level, Punkte) eines Profils."""
    prof = _profile(pid)
    return prof["level"], prof["points"]


# ---------------------------------------------------------------------------
#  Abfragen (für Erfolge und den Statistik-Screen)
# ---------------------------------------------------------------------------

def get(key, pid=None):
    """Statistik-Eintrag eines Spiels (Kopie; leere Werte, wenn nie gespielt)."""
    entry = _profile(pid)["games"].get(key)
    return dict(entry) if entry else _blank_game()


def all_stats(pid=None):
    """Alle Spiel-Einträge als dict {key: eintrag} (Kopie)."""
    return {key: dict(entry)
            for key, entry in _profile(pid)["games"].items()}


def totals(pid=None):
    """Gesamtwerte über alle Spiele (für Erfolge und die Übersichtskarten).

    Liefert ein dict mit:
      plays, time, wins, losses, records : Summen
      distinct                           : Anzahl verschiedener gespielter Spiele
      win_games                          : Anzahl Spiele mit mindestens 1 Sieg
      favorite                           : key mit der meisten Spielzeit (oder None)
      days, streak, best_streak          : Spieltage und Tagesserien
      duels, points, level, longest      : Duelle, Erfolgspunkte, Level, längste Sitzung
    """
    prof = _profile(pid)
    total = dict(plays=0, time=0.0, wins=0, losses=0, records=0,
                 distinct=0, win_games=0, favorite=None)
    best_time = 0.0
    for key, entry in prof["games"].items():
        plays = entry.get("plays", 0)
        total["plays"] += plays
        total["time"] += entry.get("time", 0.0)
        total["wins"] += entry.get("wins", 0)
        total["losses"] += entry.get("losses", 0)
        total["records"] += entry.get("records", 0)
        # "Ausprobiert" heißt: mindestens eine Partie gestartet/gespielt.
        if plays > 0 or entry.get("time", 0.0) > 0:
            total["distinct"] += 1
        if entry.get("wins", 0) > 0:
            total["win_games"] += 1
        if entry.get("time", 0.0) > best_time:
            best_time = entry.get("time", 0.0)
            total["favorite"] = key
    total["days"] = len([d for d, v in prof["days"].items()
                         if v.get("t", 0) > 0 or v.get("p", 0) > 0])
    total["streak"] = prof["streak"]["cur"]
    total["best_streak"] = prof["streak"]["best"]
    total["duels"] = duel_total(pid)
    total["points"] = prof["points"]
    total["level"] = prof["level"]
    total["longest"] = prof.get("longest", 0.0)
    return total


def days_map(pid=None):
    """{Datum: {t, p, w, a, g}} - Grundlage der Kalender-Heatmap."""
    return {day: dict(entry) for day, entry in _profile(pid)["days"].items()}


def hours_list(pid=None):
    """Spielzeit je Tagesstunde (24 Werte)."""
    return list(_profile(pid)["hours"])


def weekday_list(pid=None):
    """Spielzeit je Wochentag (Montag zuerst)."""
    import datetime
    out = [0.0] * 7
    for day, entry in _profile(pid)["days"].items():
        try:
            out[datetime.date.fromisoformat(day).weekday()] += entry.get("t", 0.0)
        except ValueError:
            continue
    return out


def events(limit=None, kinds=None, pid=None):
    """Ereignis-Verlauf, neueste zuerst."""
    items = _profile(pid)["events"]
    if kinds:
        items = [e for e in items if e.get("k") in kinds]
    items = list(reversed(items))
    return items[:limit] if limit else items


def series_of(key, pid=None):
    """Punkteverlauf eines Spiels [[zeit, punkte, rekord], ...]."""
    return [list(p) for p in _profile(pid)["series"].get(key, ())]


def played_games(pid=None):
    """Spiele mit Punkteverlauf (für die Auswahl im Verlauf-Reiter)."""
    prof = _profile(pid)
    return sorted(prof["series"].keys(),
                  key=lambda k: -len(prof["series"][k]))


def streak(pid=None):
    """{cur, best, first_day, last_day}."""
    return dict(_profile(pid)["streak"])


def duel_table(pid=None):
    """{gegner: {spiel: {w, l}}} des Profils."""
    return {other: {k: dict(v) for k, v in games.items()}
            for other, games in _profile(pid)["duel"].items()}


def profile_ids():
    """Alle Profile, für die Statistiken vorliegen."""
    return list(_load()["profiles"].keys())


def drop_profile(pid):
    """Löscht die Statistik eines Profils (beim Löschen des Profils)."""
    doc = _load()
    if doc["profiles"].pop(pid, None) is not None:
        _mark_dirty()
        flush()


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


# ---------------------------------------------------------------------------
#  Teilen: Datei exportieren / importieren
# ---------------------------------------------------------------------------

def export_to(path):
    """Schreibt die komplette Statistik als .pgzlamastats-Datei."""
    flush()
    doc = _read_file(_path()) or _blank_doc()
    return store.write_json_atomic(path, doc)


def import_from(path, target=None, merge=True):
    """Liest eine .pgzlamastats-Datei ein.

    'target' = Profil-id, in die ALLE enthaltenen Profile einfließen sollen
    (None: jedes Profil landet unter seiner eigenen id). 'merge' addiert auf
    vorhandene Werte, sonst wird ersetzt.

    Gibt (ok, grund) zurück - grund ist "io", "format" oder "".
    """
    raw = _read_file(path)
    if raw is None:
        return False, "io"
    if raw.get("format") != FORMAT or not isinstance(raw.get("profiles"), dict):
        return False, "format"
    doc = _load()
    for pid, prof in raw["profiles"].items():
        clean = _clean_profile(prof)
        dest = target or str(pid)
        if merge and dest in doc["profiles"]:
            _merge_profile(doc["profiles"][dest], clean)
        else:
            doc["profiles"][dest] = clean
    _mark_dirty()
    flush()
    return True, ""


def _merge_profile(dest, src):
    """Addiert die Werte von 'src' auf 'dest' (Import mit Zusammenführen)."""
    for key, entry in src["games"].items():
        slot = dest["games"].setdefault(key, _blank_game())
        for field in ("plays", "wins", "losses", "records"):
            slot[field] += entry.get(field, 0)
        slot["time"] += entry.get("time", 0.0)
        slot["longest"] = max(slot["longest"], entry.get("longest", 0.0))
        if entry.get("best", 0) > slot["best"]:
            slot["best"] = entry["best"]
            slot["best_at"] = entry.get("best_at", "")
        for field in ("first",):
            if entry.get(field) and (not slot[field] or entry[field] < slot[field]):
                slot[field] = entry[field]
        if entry.get("last", "") > slot["last"]:
            slot["last"] = entry["last"]
    for day, entry in src["days"].items():
        slot = _day_entry(dest, day)
        slot["t"] += entry.get("t", 0.0)
        slot["p"] += entry.get("p", 0)
        slot["w"] += entry.get("w", 0)
        slot["a"] += entry.get("a", 0)
        for key, value in entry.get("g", {}).items():
            slot["g"][key] = slot["g"].get(key, 0.0) + value
    for i, value in enumerate(src["hours"]):
        dest["hours"][i] += value
    for key, points in src["series"].items():
        merged = sorted(dest["series"].get(key, []) + points,
                        key=lambda p: p[0])[-MAX_SERIES:]
        dest["series"][key] = merged
    dest["events"] = sorted(dest["events"] + src["events"],
                            key=lambda e: e.get("t", ""))[-MAX_EVENTS:]
    for other, games in src["duel"].items():
        slot = dest["duel"].setdefault(other, {})
        for key, wl in games.items():
            cur = slot.setdefault(key, {"w": 0, "l": 0})
            cur["w"] += wl.get("w", 0)
            cur["l"] += wl.get("l", 0)
    dest["points"] = max(dest["points"], src["points"])
    dest["longest"] = max(dest.get("longest", 0.0), src.get("longest", 0.0))
    _recalc_streak(dest)
    import levels
    dest["level"] = levels.level_for(dest["points"])
