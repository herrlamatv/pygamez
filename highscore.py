# -*- coding: utf-8 -*-
"""
highscore.py
============
Lädt und speichert Highscores in einer lokalen JSON-Datei.
Die Datei liegt neben diesem Modul ("highscores.json").
"""

import json
import os

# Pfad zur JSON-Datei (immer relativ zu dieser Datei, egal von wo gestartet wird)
_HS_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "highscores.json")


def load_highscores():
    """Liest die Highscores. Gibt ein dict {spiel_key: score} zurück."""
    if not os.path.exists(_HS_PATH):
        return {}
    try:
        with open(_HS_PATH, "r", encoding="utf-8") as f:
            data = json.load(f)
            # Nur saubere int-Werte übernehmen
            return {str(k): int(v) for k, v in data.items()}
    except (json.JSONDecodeError, ValueError, OSError):
        # Beschädigte Datei -> mit leeren Highscores weiterarbeiten
        return {}


def save_highscores(scores):
    """Schreibt das dict {spiel_key: score} in die JSON-Datei."""
    try:
        with open(_HS_PATH, "w", encoding="utf-8") as f:
            json.dump(scores, f, indent=2, ensure_ascii=False)
    except OSError:
        # Wenn das Speichern fehlschlägt (z.B. keine Schreibrechte),
        # soll das Spiel trotzdem weiterlaufen.
        pass


def update_highscore(key, score):
    """
    Aktualisiert den Highscore für 'key', falls 'score' besser ist.
    Gibt (neuer_highscore, ist_neuer_rekord) zurück.
    """
    scores = load_highscores()
    old = scores.get(key, 0)
    if score > old:
        scores[key] = score
        save_highscores(scores)
        return score, True
    return old, False
