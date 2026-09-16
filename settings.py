# -*- coding: utf-8 -*-
"""
settings.py
===========
Globale, dauerhaft gespeicherte Einstellungen (analog zu highscore.py).

Gespeichert wird in "settings.json" neben diesem Modul (bei einer mit
pyinstall.bat gebauten .exe: neben der .exe):
- sound     : Soundeffekte an/aus
- volume    : Lautstärke 0.0 .. 1.0
- haptik    : Vibrations-/Rumble-Feedback an/aus (nur mit Gamepad wirksam)
- resolution: [breite, höhe] der logischen Spielfläche, auf die alle Spiele
              zeichnen (wird danach auf das Fenster skaliert). Kleiner = weniger
              Rechenaufwand.
- fps       : Ziel-Bildrate. Weniger FPS = weniger CPU/GPU-Last -> Strom sparen.
- controls  : Tastenbelegung je Spieler und Aktion, z.B.
              {"p1": {"up": "w", "down": "s", ...}, "p2": {...}}

Die Tastennamen sind Tkinter-"keysym"-Strings (z.B. "Up", "Left", "space",
"Return", "w"), genau die Werte, die auch bei InputEvent.key ankommen.
"""

import json
import os
import sys
import time

# In einer PyInstaller-.exe (sys.frozen) zeigt __file__ in den temporären
# Entpack-Ordner, der beim Beenden verschwindet - dann neben der .exe speichern.
if getattr(sys, "frozen", False):
    _PATH = os.path.join(os.path.dirname(sys.executable), "settings.json")
else:
    _PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "settings.json")

# Aktionen, die jedes Spiel kennt (nicht jedes nutzt alle).
ACTIONS = ["up", "down", "left", "right", "action"]

# Standard-Tastenbelegung: Spieler 1 = WASD + Leertaste, Spieler 2 = Pfeile + Enter.
DEFAULT_CONTROLS = {
    "p1": {"up": "w", "down": "s", "left": "a", "right": "d", "action": "space"},
    "p2": {"up": "Up", "down": "Down", "left": "Left", "right": "Right",
           "action": "Return"},
}

# Auswählbare Vorlagen im Options-Menü. Das erste Feld ist ein i18n-Schlüssel
# (wird im Menü über i18n.t(...) übersetzt), das zweite die Tastenbelegung.
PRESETS = [
    ("preset.wasd_arrows", {
        "p1": {"up": "w", "down": "s", "left": "a", "right": "d", "action": "space"},
        "p2": {"up": "Up", "down": "Down", "left": "Left", "right": "Right",
               "action": "Return"},
    }),
    ("preset.wasd_ijkl", {
        "p1": {"up": "w", "down": "s", "left": "a", "right": "d", "action": "space"},
        "p2": {"up": "i", "down": "k", "left": "j", "right": "l", "action": "o"},
    }),
    ("preset.arrows_wasd", {
        "p1": {"up": "Up", "down": "Down", "left": "Left", "right": "Right",
               "action": "Return"},
        "p2": {"up": "w", "down": "s", "left": "a", "right": "d", "action": "space"},
    }),
]

# Auswählbare Auflösungen (logische Spielfläche). Kleiner spart Rechenzeit,
# größer wird schärfer beim Hochskalieren. Erstes Feld = i18n-Schlüssel.
RESOLUTIONS = [
    ("res.480", (480, 360)),
    ("res.640", (640, 480)),
    ("res.800", (800, 600)),
    ("res.960", (960, 720)),
    ("res.1280", (1280, 960)),
]

# Auswählbare Ziel-Bildraten. Weniger FPS = weniger Last = Strom sparen.
FPS_OPTIONS = [15, 30, 45, 60, 120]

# Verfügbare UI-Designs (siehe ui.py): "v42" = UI v4.2 "Midnight Glass"
# (Standard: Mitternachtsblau, Farbwolken, Filmkorn, Glas-Panels), "v41" =
# UI v4.1 (clean + Weltraum-Deko), "v411".."v414" = UI v4.1.1 bis v4.1.4 (v4.1
# mit Zickzack-Muster als Hintergrund - schwarz/anthrazit, blau/dunkelblau,
# v4-Indigo auf Schwarz bzw. v4-Graphit auf Schwarz), "modern" = UI v4
# (komplett ruhig), "classic" = UI v3 (die alte UI mit Sternen/Aurora/Glow),
# "v2" = UI v2 (die allererste ui.py: Navy-Verlauf, Sternenfeld, statische
# Glow-Buttons), "v1" = UI v1 (der flache Look vor dem UI Rework). Wählbar im
# Options-Screen unter "Erscheinungsbild".
THEMES = ("v42", "v41", "v411", "v412", "v413", "v414", "modern", "classic",
          "v2", "v1")

DEFAULTS = {
    # UI-Design: "v42" (Standard, "Midnight Glass"), "v41" (UI v4.1),
    # "v411".."v414" (Muster-Hintergrund), "modern" (UI v4), "classic"
    # (UI v3), "v2" (UI v2) oder "v1" (UI v1).
    "theme": "v42",
    # Standardmäßig AUS (Erststart ist leise; im Willkommens-Screen/den Optionen
    # einschaltbar). Bestehende Installationen behalten ihren gespeicherten Wert.
    "sound": False,
    "volume": 0.6,
    "haptik": False,
    "resolution": [640, 480],
    # auto_resolution: True -> die logische Auflösung folgt der Fenstergröße
    # (beim Öffnen und beim Vergrößern/Verkleinern). False -> feste Auflösung.
    "auto_resolution": False,
    "fps": 60,
    # Snake-spezifische Optionen:
    #   wrap        : durch die Wände gehen und auf der Gegenseite herauskommen
    #   bonus_apple : jeder Apfel zählt zufällig 1 oder 2 (im Schnitt ~1,5)
    "snake": {"wrap": False, "bonus_apple": False},
    # Pong-spezifische Optionen (pro Steuerung):
    #   hold_p1 / hold_p2 : True  -> Schläger bewegt sich nur, solange man die
    #                       Taste drückt (Halten). False -> fährt dauerhaft weiter.
    "pong": {"hold_p1": False, "hold_p2": False},
    # Sudoku-spezifische Optionen:
    #   difficulty : gewählter Schwierigkeitsgrad (0=Leicht .. 3=Experte)
    #   fail_limit : True -> beim 3. Fehler ist die Partie verloren
    #   last_level : zuletzt gewähltes Level je Stufe, z.B. {"0": 12}
    #   variant    : classic / x (Diagonalen) / killer (Käfige) / mini (6x6)
    #   input      : cell (erst Zelle, dann Ziffer) / digit (erst Ziffer, dann Zellen)
    #   colors     : Farbmarker-Leiste im Spiel anzeigen (Zellen einfärben)
    "sudoku": {"difficulty": 0, "fail_limit": True, "last_level": {},
               "variant": "classic", "input": "cell", "colors": False},
    # Frogger: Schwierigkeitsgrad (easy/normal/hard)
    "frogger": {"difficulty": "normal"},
    # Memory: Brettgröße (4x4/6x6/8x6)
    "memory": {"size": "6x6"},
    # Solitär: Klondike zieht 3 Karten (sonst 1); Spider-Farbenzahl (1/2/4)
    "solitaire": {"draw3": False, "spider_suits": 1},
    # Aim Trainer: Arena-Thema (space/neon/range), Maus-Empfindlichkeit,
    # Motion-Blur-Stärke (0.0 = aus .. 0.8 = maximal)
    "aim": {"theme": "space", "sens": 1.0, "blur": 0.0},
    # Vier gewinnt: KI-Stärke (0=Leicht, 1=Mittel, 2=Schwer)
    "connect4": {"difficulty": 1},
    # Reversi (Othello): KI-Stärke (0=Leicht, 1=Mittel, 2=Schwer)
    "reversi": {"difficulty": 1},
    # Panzer-Duell: KI-Stärke (0-2), Arena (-1 = zufällig, 0-3 = Preset)
    "tanks": {"difficulty": 1, "arena": -1},
    # Tunnel Racer: Steuerung (keys/mouse), Motion-Blur, letztes Level
    "tunnel": {"control": "keys", "blur": 0.35, "last_level": 1},
    # Labyrinth: Ansicht (ego/top), Maus-Empfindlichkeit, letztes Level
    "maze": {"view": "ego", "sens": 1.0, "last_level": 1},
    # T-Rex Runner: Schwierigkeit (chill/normal/hardcore), Figur/Skin (0-3),
    # Tag/Nacht-Wechsel an/aus
    "trex": {"difficulty": "normal", "skin": 1, "night": True},
    # Dame: Variante (german/international/checkers), KI-Stärke (0-2)
    "dame": {"variant": "german", "difficulty": 1},
    # Poker: Anzahl KI-Gegner (1-3, nur Texas Hold'em), KI-Stärke (0-2)
    "poker": {"opponents": 2, "difficulty": 1},
    # Schach: KI-Stärke (0=Anfänger .. 5=Meister), Spielerfarbe (white/black),
    # Schachuhr (none/1+0/3+2/5+0/10+5), Chess960-Startaufstellung an/aus,
    # Brett im 2-Spieler-Modus nach jedem Zug drehen an/aus
    "chess": {"difficulty": 2, "color": "white", "clock": "none",
              "chess960": False, "flip": False},
    # Mühle: KI-Stärke (0-2), Fliegen-Regel bei nur noch 3 Steinen an/aus
    "muehle": {"difficulty": 1, "flying": True},
    # Simon: Modus (classic/speed/reverse/duel/mixed), Ton (off/on/mixed),
    # Feldanzahl (4/6/9)
    "simon": {"mode": "classic", "audio": "on", "pads": 4},
    # Billard: Variante (8ball/9ball/practice), Ansicht (2d/3d/free),
    # KI-Stärke (0-2)
    "billiard": {"variant": "8ball", "view": "2d", "difficulty": 1},
    # Block Jump: Kamera (first/third), Motion-Blur (0.0-0.8),
    # Maus-Empfindlichkeit (0.4-2.5), invertierte Maus-Richtung,
    # Texturdetail der Minecraft-Optik (high/low/off)
    "blockjump": {"view": "first", "blur": 0.35, "sens": 1.0,
                  "mouse_invert": False, "textures": "high"},
    # Minigolf: Kurs (classic/pro/tour/random/ugc), Tour-Kursnummer, Ziellinie
    # an/aus, "Autoziel" (Schläger zeigt vor jedem Schlag von selbst zum Loch)
    # an/aus und "Aufnehmen" (Bahn nach 8 Schlägen beenden) an/aus.
    # ugc_map = id der zuletzt einzeln gespielten eigenen Bahn (siehe ugc.py),
    # grid = Rasterfang im Bahn-Editor.
    "minigolf": {"course": "classic", "tour": 1, "guide": True,
                 "autoaim": True, "pickup": True, "ugc_map": "", "grid": True},
    # Pinball: Tisch (classic/space/lama) und Bälle je Partie (3/5)
    "pinball": {"table": "classic", "balls": 3},
    # Bowling: Schwierigkeit (easy/normal/pro) und Zielhilfe an/aus
    "bowling": {"difficulty": "normal", "guide": True},
    # Replays: Aufzeichnung von Minigolf- und Bowling-Runden an/aus (die
    # Aufnahmen selbst liegen in replay.json, siehe replay.py)
    "replay": {"enabled": True},
    # Tetris: DAS (Verzögerung bis zur Dauerbewegung, ms), ARR (Abstand der
    # Wiederholschritte, ms; 0 = sofort an den Rand), Ghost-Stein an/aus,
    # Solo-Variante (marathon/sprint/ultra), KI-Stärke im Versus (0-2),
    # Startlevel im Marathon (1-15)
    "tetris": {"das": 170, "arr": 50, "ghost": True, "solo": "marathon",
               "ai_level": 1, "start_level": 1},
    # 2048: Brettgröße (3-8), Modus (classic/time/endless),
    # Rückgängig (off/limited = 3 pro Partie/unlimited)
    "g2048": {"size": 4, "mode": "classic", "undo": "limited"},
    # Wordle: Wortlänge (4-7), harter Modus, Farbenblind-Palette
    "wordle": {"length": 5, "hard": False, "colorblind": False},
    # Crossy Road: Schatten an/aus, Tag/Nacht-Wechsel an/aus
    # (Figur/Münzen liegen als Fortschritt in mem.json)
    "crossy": {"shadows": True, "daynight": True},
    # Geometry Dash: Level-Musik an/aus, automatische Checkpoints im
    # Übungsmodus an/aus, Fortschrittsbalken an/aus, Rasterfang im Editor,
    # zuletzt gewähltes eingebautes Level (0-7)
    "geodash": {"music": True, "auto_checkpoints": True, "progress_bar": True,
                "grid": True, "last_level": 0},
    # Battleship / Schiffe versenken: KI-Stärke (0-2), Schiffe dürfen sich
    # berühren, Salven-Modus, nach Treffer nochmal schießen
    "battleship": {"difficulty": 1, "touch": True, "salvo": False,
                   "extra_shot": False},
    # Casino: Chipwert am Roulette-Tisch, Einsatz je Gewinnlinie am
    # Spielautomaten, Turbo-Walzen an/aus
    "casino": {"roulette_chip": 5, "line_bet": 1, "turbo": False},
    "controls": DEFAULT_CONTROLS,
}


# Prüfregeln für Spiel-Optionen: {section: {key: regel}}. Statt für jede neue
# Option einen eigenen if-Block in _merge_defaults zu schreiben, genügt hier
# ein Eintrag (plus der Standardwert in DEFAULTS). Nur hier gelistete Schlüssel
# überleben das Laden - alles andere fällt wie bisher still weg.
#   bool                  -> nur True/False
#   (int, lo, hi)         -> Ganzzahl, auf [lo, hi] begrenzt
#   (float, lo, hi)       -> Zahl, auf [lo, hi] begrenzt
#   ("choice", (a, b, …)) -> genau einer der erlaubten Werte
#   (str, maxlen)         -> Text, auf maxlen Zeichen gekürzt
SCHEMAS = {
    "sudoku": {"variant": ("choice", ("classic", "x", "killer", "mini")),
               "input": ("choice", ("cell", "digit")), "colors": bool},
    "chess": {"clock": ("choice", ("none", "1+0", "3+2", "5+0", "10+5")),
              "chess960": bool, "flip": bool},
    "tetris": {"das": (int, 50, 400), "arr": (int, 0, 200), "ghost": bool,
               "solo": ("choice", ("marathon", "sprint", "ultra")),
               "ai_level": (int, 0, 2), "start_level": (int, 1, 15)},
    "g2048": {"size": (int, 3, 8), "mode": ("choice", ("classic", "time", "endless")),
              "undo": ("choice", ("off", "limited", "unlimited"))},
    "wordle": {"length": (int, 4, 7), "hard": bool, "colorblind": bool},
    "crossy": {"shadows": bool, "daynight": bool},
    "geodash": {"music": bool, "auto_checkpoints": bool, "progress_bar": bool,
                "grid": bool, "last_level": (int, 0, 7)},
    "battleship": {"difficulty": (int, 0, 2), "touch": bool, "salvo": bool,
                   "extra_shot": bool},
    "casino": {"roulette_chip": ("choice", (1, 5, 25, 100, 500)),
               "line_bet": ("choice", (1, 2, 5, 10)), "turbo": bool},
}

_INVALID = object()


def _check_rule(rule, value):
    """Prüft 'value' gegen eine Regel aus SCHEMAS (sonst _INVALID)."""
    if rule is bool:
        return value if isinstance(value, bool) else _INVALID
    kind = rule[0]
    if kind is int:
        if isinstance(value, bool) or not isinstance(value, int):
            return _INVALID
        return max(rule[1], min(rule[2], value))
    if kind is float:
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            return _INVALID
        return max(rule[1], min(rule[2], float(value)))
    if kind == "choice":
        # Typ muss passen: bool ist in Python ein int - True darf nicht als 1
        # durchgehen, und "5" nicht als 5.
        ok = any(type(c) is type(value) and c == value for c in rule[1])
        return value if ok else _INVALID
    if kind is str:
        return value[:rule[1]] if isinstance(value, str) else _INVALID
    return _INVALID


def _apply_schemas(out, data):
    """Übernimmt alle in SCHEMAS beschriebenen Optionen aus 'data' nach 'out'."""
    for section, rules in SCHEMAS.items():
        src = data.get(section)
        if not isinstance(src, dict):
            continue
        dst = out.setdefault(section, {})
        for key, rule in rules.items():
            if key in src:
                value = _check_rule(rule, src[key])
                if value is not _INVALID:
                    dst[key] = value


def resolution_index(res):
    """Index der Auflösung in RESOLUTIONS, die (breite, höhe) entspricht (sonst 1)."""
    if isinstance(res, (list, tuple)) and len(res) == 2:
        target = (int(res[0]), int(res[1]))
        for i, (_, wh) in enumerate(RESOLUTIONS):
            if wh == target:
                return i
    return 1  # Standard 640 x 480


def fps_index(fps):
    """Index der FPS in FPS_OPTIONS (sonst der nächstliegende Wert)."""
    if fps in FPS_OPTIONS:
        return FPS_OPTIONS.index(fps)
    return min(range(len(FPS_OPTIONS)), key=lambda i: abs(FPS_OPTIONS[i] - fps))


def _clone(obj):
    """Tiefe Kopie über JSON (nur einfache Typen -> ausreichend hier)."""
    return json.loads(json.dumps(obj))


def _merge_defaults(data):
    """Ergänzt fehlende Schlüssel mit den Standardwerten (robust gegen Alt-Dateien)."""
    out = _clone(DEFAULTS)
    if isinstance(data, dict):
        if data.get("theme") in THEMES:
            out["theme"] = data["theme"]
        for k in ("sound", "haptik", "auto_resolution"):
            if isinstance(data.get(k), bool):
                out[k] = data[k]
        if isinstance(data.get("volume"), (int, float)):
            out["volume"] = max(0.0, min(1.0, float(data["volume"])))
        res = data.get("resolution")
        if (isinstance(res, (list, tuple)) and len(res) == 2
                and all(isinstance(v, int) and v > 0 for v in res)):
            out["resolution"] = [int(res[0]), int(res[1])]
        if isinstance(data.get("fps"), int) and data["fps"] > 0:
            out["fps"] = max(5, min(240, int(data["fps"])))
        snk = data.get("snake")
        if isinstance(snk, dict):
            for k in ("wrap", "bonus_apple"):
                if isinstance(snk.get(k), bool):
                    out["snake"][k] = snk[k]
            # Weitere Snake-Optionen (mode, hardcore, apples, 3D-Kamera, ...)
            # unverändert übernehmen - das Spiel validiert sie beim Lesen
            # selbst. Ohne diese Übernahme gingen sie beim nächsten Speichern
            # der Einstellungen (egal aus welchem Spiel) verloren.
            for k, v in snk.items():
                if k not in ("wrap", "bonus_apple"):
                    out["snake"][k] = v
        pg = data.get("pong")
        if isinstance(pg, dict):
            for k in ("hold_p1", "hold_p2"):
                if isinstance(pg.get(k), bool):
                    out["pong"][k] = pg[k]
        sud = data.get("sudoku")
        if isinstance(sud, dict):
            if isinstance(sud.get("difficulty"), int):
                out["sudoku"]["difficulty"] = max(0, min(3, sud["difficulty"]))
            if isinstance(sud.get("fail_limit"), bool):
                out["sudoku"]["fail_limit"] = sud["fail_limit"]
            ll = sud.get("last_level")
            if isinstance(ll, dict):
                for k, v in ll.items():
                    if k in ("0", "1", "2", "3") and isinstance(v, int):
                        out["sudoku"]["last_level"][k] = max(1, min(100, v))
        fr = data.get("frogger")
        if isinstance(fr, dict) and fr.get("difficulty") in ("easy", "normal",
                                                             "hard"):
            out["frogger"]["difficulty"] = fr["difficulty"]
        mem = data.get("memory")
        if isinstance(mem, dict) and mem.get("size") in ("4x4", "6x6", "8x6"):
            out["memory"]["size"] = mem["size"]
        sol = data.get("solitaire")
        if isinstance(sol, dict):
            if isinstance(sol.get("draw3"), bool):
                out["solitaire"]["draw3"] = sol["draw3"]
            if sol.get("spider_suits") in (1, 2, 4):
                out["solitaire"]["spider_suits"] = sol["spider_suits"]
        aim = data.get("aim")
        if isinstance(aim, dict):
            if aim.get("theme") in ("space", "neon", "range"):
                out["aim"]["theme"] = aim["theme"]
            if isinstance(aim.get("sens"), (int, float)):
                out["aim"]["sens"] = max(0.5, min(2.0, float(aim["sens"])))
            if isinstance(aim.get("blur"), (int, float)):
                out["aim"]["blur"] = max(0.0, min(0.8, float(aim["blur"])))
        c4 = data.get("connect4")
        if isinstance(c4, dict) and isinstance(c4.get("difficulty"), int):
            out["connect4"]["difficulty"] = max(0, min(2, c4["difficulty"]))
        rv = data.get("reversi")
        if isinstance(rv, dict) and isinstance(rv.get("difficulty"), int):
            out["reversi"]["difficulty"] = max(0, min(2, rv["difficulty"]))
        tk_ = data.get("tanks")
        if isinstance(tk_, dict):
            if isinstance(tk_.get("difficulty"), int):
                out["tanks"]["difficulty"] = max(0, min(2, tk_["difficulty"]))
            if isinstance(tk_.get("arena"), int):
                out["tanks"]["arena"] = max(-1, min(3, tk_["arena"]))
        tun = data.get("tunnel")
        if isinstance(tun, dict):
            if tun.get("control") in ("keys", "mouse"):
                out["tunnel"]["control"] = tun["control"]
            if isinstance(tun.get("blur"), (int, float)):
                out["tunnel"]["blur"] = max(0.0, min(0.8, float(tun["blur"])))
            if isinstance(tun.get("last_level"), int):
                out["tunnel"]["last_level"] = max(1, min(30,
                                                         tun["last_level"]))
        mz = data.get("maze")
        if isinstance(mz, dict):
            if mz.get("view") in ("ego", "top"):
                out["maze"]["view"] = mz["view"]
            if isinstance(mz.get("sens"), (int, float)):
                out["maze"]["sens"] = max(0.5, min(2.0, float(mz["sens"])))
            if isinstance(mz.get("last_level"), int):
                out["maze"]["last_level"] = max(1, min(50, mz["last_level"]))
        tx = data.get("trex")
        if isinstance(tx, dict):
            if tx.get("difficulty") in ("chill", "normal", "hardcore"):
                out["trex"]["difficulty"] = tx["difficulty"]
            if isinstance(tx.get("skin"), int):
                out["trex"]["skin"] = max(0, min(3, tx["skin"]))
            if isinstance(tx.get("night"), bool):
                out["trex"]["night"] = tx["night"]
        dm = data.get("dame")
        if isinstance(dm, dict):
            if dm.get("variant") in ("german", "international", "checkers"):
                out["dame"]["variant"] = dm["variant"]
            if isinstance(dm.get("difficulty"), int):
                out["dame"]["difficulty"] = max(0, min(2, dm["difficulty"]))
        pk = data.get("poker")
        if isinstance(pk, dict):
            if isinstance(pk.get("opponents"), int):
                out["poker"]["opponents"] = max(1, min(3, pk["opponents"]))
            if isinstance(pk.get("difficulty"), int):
                out["poker"]["difficulty"] = max(0, min(2, pk["difficulty"]))
        ch = data.get("chess")
        if isinstance(ch, dict):
            if isinstance(ch.get("difficulty"), int):
                out["chess"]["difficulty"] = max(0, min(5, ch["difficulty"]))
            if ch.get("color") in ("white", "black"):
                out["chess"]["color"] = ch["color"]
        mu = data.get("muehle")
        if isinstance(mu, dict):
            if isinstance(mu.get("difficulty"), int):
                out["muehle"]["difficulty"] = max(0, min(2, mu["difficulty"]))
            if isinstance(mu.get("flying"), bool):
                out["muehle"]["flying"] = mu["flying"]
        sim = data.get("simon")
        if isinstance(sim, dict):
            if sim.get("mode") in ("classic", "speed", "reverse", "duel", "mixed"):
                out["simon"]["mode"] = sim["mode"]
            if sim.get("audio") in ("off", "on", "mixed"):
                out["simon"]["audio"] = sim["audio"]
            if sim.get("pads") in (4, 6, 9):
                out["simon"]["pads"] = sim["pads"]
        bil = data.get("billiard")
        if isinstance(bil, dict):
            if bil.get("variant") in ("8ball", "9ball", "practice"):
                out["billiard"]["variant"] = bil["variant"]
            if bil.get("view") in ("2d", "3d", "free"):
                out["billiard"]["view"] = bil["view"]
            if isinstance(bil.get("difficulty"), int):
                out["billiard"]["difficulty"] = max(0, min(2, bil["difficulty"]))
        blj = data.get("blockjump")
        if isinstance(blj, dict):
            if blj.get("view") in ("first", "third"):
                out["blockjump"]["view"] = blj["view"]
            if isinstance(blj.get("blur"), (int, float)):
                out["blockjump"]["blur"] = max(0.0, min(0.8, float(blj["blur"])))
            if isinstance(blj.get("sens"), (int, float)):
                out["blockjump"]["sens"] = max(0.4, min(2.5, float(blj["sens"])))
            if isinstance(blj.get("mouse_invert"), bool):
                out["blockjump"]["mouse_invert"] = blj["mouse_invert"]
            if blj.get("textures") in ("high", "low", "off"):
                out["blockjump"]["textures"] = blj["textures"]
        mg = data.get("minigolf")
        if isinstance(mg, dict):
            if mg.get("course") in ("classic", "pro", "tour", "random", "ugc"):
                out["minigolf"]["course"] = mg["course"]
            if isinstance(mg.get("tour"), int):
                out["minigolf"]["tour"] = max(1, min(38, mg["tour"]))
            if isinstance(mg.get("guide"), bool):
                out["minigolf"]["guide"] = mg["guide"]
            if isinstance(mg.get("autoaim"), bool):
                out["minigolf"]["autoaim"] = mg["autoaim"]
            if isinstance(mg.get("pickup"), bool):
                out["minigolf"]["pickup"] = mg["pickup"]
            # Eigene Bahnen: zuletzt gespielte/bearbeitete id und der
            # Rasterfang des Editors.
            if isinstance(mg.get("ugc_map"), str):
                out["minigolf"]["ugc_map"] = mg["ugc_map"][:32]
            if isinstance(mg.get("grid"), bool):
                out["minigolf"]["grid"] = mg["grid"]
        pin = data.get("pinball")
        if isinstance(pin, dict):
            if pin.get("table") in ("classic", "space", "lama"):
                out["pinball"]["table"] = pin["table"]
            if pin.get("balls") in (3, 5):
                out["pinball"]["balls"] = pin["balls"]
        bow = data.get("bowling")
        if isinstance(bow, dict):
            if bow.get("difficulty") in ("easy", "normal", "pro"):
                out["bowling"]["difficulty"] = bow["difficulty"]
            if isinstance(bow.get("guide"), bool):
                out["bowling"]["guide"] = bow["guide"]
        rpl = data.get("replay")
        if isinstance(rpl, dict) and isinstance(rpl.get("enabled"), bool):
            out["replay"]["enabled"] = rpl["enabled"]
        # Alle Optionen mit Prüfregel in SCHEMAS (neuere Spiele + Erweiterungen).
        _apply_schemas(out, data)
        ctrl = data.get("controls")
        if isinstance(ctrl, dict):
            for player in ("p1", "p2"):
                pdata = ctrl.get(player)
                if isinstance(pdata, dict):
                    for act in ACTIONS:
                        if isinstance(pdata.get(act), str) and pdata[act]:
                            out["controls"][player][act] = pdata[act]
    return out


def load_settings():
    """Liest die Einstellungen (immer ein vollständiges, gültiges dict)."""
    if not os.path.exists(_PATH):
        return _clone(DEFAULTS)
    try:
        with open(_PATH, "r", encoding="utf-8") as f:
            return _merge_defaults(json.load(f))
    except (json.JSONDecodeError, ValueError, OSError):
        return _clone(DEFAULTS)


def _generated_stamp():
    """Liefert den "generated by"-Stempel der Datei.

    Ein bereits vorhandener Stempel (= Anlege-Datum der Datei) bleibt
    erhalten; nur wenn keiner existiert, wird ein neuer erzeugt.
    """
    try:
        with open(_PATH, "r", encoding="utf-8") as f:
            stamp = json.load(f).get("_generated")
            if isinstance(stamp, str) and stamp:
                return stamp
    except (OSError, json.JSONDecodeError, ValueError):
        pass
    return ("generated by PyGameZ at " + time.strftime("%Y-%m-%d %H:%M:%S")
            + " (YYYY-MM-DD HH:MM:SS)")


def save_settings(data):
    """Schreibt die Einstellungen als JSON. Fehler werden bewusst ignoriert.

    "_generated" (Anlege-Datum der Datei) steht immer als erster Eintrag.
    """
    out = {"_generated": _generated_stamp()}
    out.update((k, v) for k, v in data.items() if k != "_generated")
    import store
    store.write_json_atomic(_PATH, out)


def apply_preset(settings, index):
    """Setzt die controls aus PRESETS[index] in das übergebene settings-dict."""
    if 0 <= index < len(PRESETS):
        settings["controls"] = _clone(PRESETS[index][1])
