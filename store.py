# -*- coding: utf-8 -*-
"""
store.py
========
Zentraler Zugriff auf die einzige Speicherdatei ``mem.json``.

``mem.json`` besteht aus benannten Abschnitten (Sections). Aktuell:

    {
      "mem":        { "lang": "de" },      # Einstellungen der Oberflaeche (Sprache ...)
      "highscores": { "snake": 660, ... }  # Bestwerte je Spiel
    }

Jedes Modul liest/schreibt nur seinen eigenen Abschnitt ueber
``load_section``/``save_section`` - dabei bleibt der Rest der Datei erhalten
(gelesen -> geaendert -> komplett zurueckgeschrieben). So landen Sprache und
Highscores gemeinsam in EINER Datei.

Alte Formate (getrennte ``highscores.json`` bzw. flaches ``mem.json`` mit
``lang`` auf oberster Ebene) werden beim ersten Laden automatisch in die neue
Struktur uebernommen.
"""

import json
import os
import sys

# In einer PyInstaller-.exe (sys.frozen) zeigt __file__ in den temporaeren
# Entpack-Ordner, der beim Beenden verschwindet - dann neben der .exe speichern.
if getattr(sys, "frozen", False):
    _DIR = os.path.dirname(sys.executable)
else:
    _DIR = os.path.dirname(os.path.abspath(__file__))
_PATH = os.path.join(_DIR, "mem.json")
# Frueherer, separater Speicherort der Highscores (nur noch fuer die Migration).
_LEGACY_HS_PATH = os.path.join(_DIR, "highscores.json")


def _read_raw():
    """Liest mem.json roh (immer ein dict, leer bei Fehler/fehlender Datei)."""
    try:
        with open(_PATH, "r", encoding="utf-8") as f:
            data = json.load(f)
            return data if isinstance(data, dict) else {}
    except (OSError, json.JSONDecodeError, ValueError):
        return {}


def _read_legacy_highscores():
    """Liest eine evtl. noch vorhandene alte highscores.json (leer bei Fehler)."""
    try:
        with open(_LEGACY_HS_PATH, "r", encoding="utf-8") as f:
            data = json.load(f)
            return data if isinstance(data, dict) else {}
    except (OSError, json.JSONDecodeError, ValueError):
        return {}


def _normalize(data):
    """Bringt alte Formate auf die neue {mem, highscores}-Struktur.

    Gibt (normalisiertes_dict, wurde_geaendert) zurueck.
    """
    changed = False

    # Abschnitt "mem": frueher lagen Schluessel wie "lang" direkt oben.
    if not isinstance(data.get("mem"), dict):
        mem = {}
        for key in ("lang",):
            if key in data:
                mem[key] = data.pop(key)
        data["mem"] = mem
        changed = True

    # Abschnitt "highscores": ggf. aus alter highscores.json uebernehmen.
    if not isinstance(data.get("highscores"), dict):
        data["highscores"] = _read_legacy_highscores()
        changed = True

    return data, changed


def load():
    """Liest mem.json als vollstaendiges dict (mit Abschnitten) und migriert bei Bedarf."""
    data, changed = _normalize(_read_raw())
    if changed:
        save(data)
    return data


def save(data):
    """Schreibt das vollstaendige dict nach mem.json (Fehler werden ignoriert)."""
    try:
        with open(_PATH, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2, ensure_ascii=False)
    except OSError:
        # Schlaegt das Speichern fehl (z.B. keine Schreibrechte), laeuft das
        # Spiel trotzdem weiter - nur ohne Persistenz.
        pass


def load_section(name):
    """Liefert den Abschnitt 'name' als dict (leer, wenn nicht vorhanden)."""
    section = load().get(name)
    return section if isinstance(section, dict) else {}


def save_section(name, section):
    """Ersetzt den Abschnitt 'name' und schreibt die ganze Datei zurueck."""
    data = load()
    data[name] = section
    save(data)
