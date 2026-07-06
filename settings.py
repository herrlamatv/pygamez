# -*- coding: utf-8 -*-
"""
settings.py
===========
Globale, dauerhaft gespeicherte Einstellungen (analog zu highscore.py).

Gespeichert wird in "settings.json" neben diesem Modul:
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

DEFAULTS = {
    "sound": True,
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
    "controls": DEFAULT_CONTROLS,
}


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
        pg = data.get("pong")
        if isinstance(pg, dict):
            for k in ("hold_p1", "hold_p2"):
                if isinstance(pg.get(k), bool):
                    out["pong"][k] = pg[k]
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


def save_settings(data):
    """Schreibt die Einstellungen als JSON. Fehler werden bewusst ignoriert."""
    try:
        with open(_PATH, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
    except OSError:
        pass


def apply_preset(settings, index):
    """Setzt die controls aus PRESETS[index] in das übergebene settings-dict."""
    if 0 <= index < len(PRESETS):
        settings["controls"] = _clone(PRESETS[index][1])
