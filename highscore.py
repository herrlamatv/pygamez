# -*- coding: utf-8 -*-
"""
highscore.py
============
Lädt und speichert Highscores. Sie liegen im Abschnitt ``highscores`` der
gemeinsamen Datei ``mem.json`` (siehe store.py) - zusammen mit den übrigen
gespeicherten Daten (z.B. der Sprache).
"""

import store

# Name des Abschnitts in mem.json.
_SECTION = "highscores"


def load_highscores():
    """Liest die Highscores. Gibt ein dict {spiel_key: score} zurück."""
    try:
        return {str(k): int(v) for k, v in store.load_section(_SECTION).items()}
    except (ValueError, TypeError):
        # Beschädigte Werte -> mit leeren Highscores weiterarbeiten
        return {}


def save_highscores(scores):
    """Schreibt das dict {spiel_key: score} in den highscores-Abschnitt."""
    store.save_section(_SECTION, scores)


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
