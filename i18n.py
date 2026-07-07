# -*- coding: utf-8 -*-
"""
i18n.py
=======
Kleine Übersetzungs-Engine (Internationalisierung) für die Spielesammlung.

- Die Sprach-Strings liegen als flache JSON-Dateien im Ordner ``lang/``
  (``de.json``, ``en.json``). Jeder String hat einen stabilen Platzhalter-
  Schlüssel, z.B. ``"menu.singleplayer"``.
- Die gewählte Sprache wird in ``mem.json`` (neben diesem Modul) gespeichert,
  z.B. ``{"lang": "en"}``. Beim nächsten Start wird sie automatisch geladen.
- ``t(key, **kwargs)`` liefert den übersetzten Text. Platzhalter im String
  werden per ``str.format`` gefüllt, z.B.  t("snake.score", score=12).
- Fehlt ein Schlüssel in der aktiven Sprache, wird auf Deutsch (DEFAULT_LANG)
  zurückgegriffen; fehlt er auch dort, wird der Schlüssel selbst angezeigt.

Verwendung::

    import i18n
    i18n.init()                 # einmalig beim Programmstart
    i18n.t("app.quit")          # -> "Beenden" / "Quit"
    i18n.set_language("en")     # Sprache wechseln (und in mem.json speichern)
"""

import json
import os

import store

_DIR = os.path.dirname(os.path.abspath(__file__))
_LANG_DIR = os.path.join(_DIR, "lang")

# Name des Abschnitts in mem.json, in dem Oberflaechen-Einstellungen (Sprache)
# liegen. Die Highscores stehen im Abschnitt "highscores" derselben Datei.
_MEM_SECTION = "mem"

# Standardsprache = Fallback, wenn ein Schlüssel fehlt oder nichts gewählt ist.
DEFAULT_LANG = "de"

# Verfügbare Sprachen: (Code, Anzeigename). Reihenfolge = Anzeige im Auswahl-Screen.
AVAILABLE = [
    ("de", "Deutsch"),
    ("en", "English"),
    ("fr", "Français"),
]
_CODES = {code for code, _ in AVAILABLE}

_current = DEFAULT_LANG
_strings = {}        # Strings der aktiven Sprache
_fallback = {}       # Strings der Standardsprache


# ----- Laden/Speichern ---------------------------------------------------

def _load_lang_file(code):
    """Liest lang/<code>.json und gibt ein dict zurück (leer bei Fehler)."""
    path = os.path.join(_LANG_DIR, f"{code}.json")
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
            return data if isinstance(data, dict) else {}
    except (OSError, json.JSONDecodeError, ValueError):
        return {}


def load_memory():
    """Liest den 'mem'-Abschnitt aus mem.json (immer ein dict)."""
    return store.load_section(_MEM_SECTION)


def save_memory(data):
    """Schreibt den 'mem'-Abschnitt zurück (Rest der Datei bleibt erhalten)."""
    store.save_section(_MEM_SECTION, data)


# ----- Öffentliche API ---------------------------------------------------

def has_language():
    """True, wenn in mem.json bereits eine gültige Sprache gespeichert ist."""
    return load_memory().get("lang") in _CODES


def get_language():
    """Aktiver Sprachcode, z.B. 'de' oder 'en'."""
    return _current


def set_language(code, persist=True):
    """Setzt die aktive Sprache (und speichert sie in mem.json, wenn persist)."""
    global _current, _strings
    if code not in _CODES:
        code = DEFAULT_LANG
    _current = code
    _strings = _load_lang_file(code)
    if persist:
        mem = load_memory()
        mem["lang"] = code
        save_memory(mem)


def init():
    """Lädt Fallback + gespeicherte Sprache. Einmalig beim Programmstart."""
    global _fallback
    _fallback = _load_lang_file(DEFAULT_LANG)
    code = load_memory().get("lang")
    set_language(code if code in _CODES else DEFAULT_LANG, persist=False)


def t(key, **kwargs):
    """Übersetzt 'key' in die aktive Sprache; füllt Platzhalter via str.format."""
    s = _strings.get(key)
    if s is None:
        s = _fallback.get(key, key)
    if kwargs and isinstance(s, str):
        try:
            return s.format(**kwargs)
        except (KeyError, IndexError, ValueError):
            return s
    return s
