# -*- coding: utf-8 -*-
"""
prestige.py
===========
Prestige-Stufen für Snake. Hier stehen ALLE erreichbaren Prestige-Level -
diese Datei kann man frei erweitern und die Werte anpassen.

Jeder Eintrag in PRESTIGE_LEVELS beschreibt, was es kostet, DIESE Stufe zu
ERREICHEN (ausgehend von der vorherigen):

    roman   : Anzeige-Name (römische Zahl), z.B. "I", "II", "III", ...
    apples  : so viele Äpfel braucht man dafür - und verliert sie dabei
    length  : so viele Körper-Segmente verliert die Schlange dabei

PRESTIGE_LEVELS[0] = Weg zu "Prestige I" (Level 1),
PRESTIGE_LEVELS[1] = Weg zu "Prestige II" (Level 2), usw.

Der NUTZEN einer Stufe hängt am erreichten Level:
    Wachstum pro Apfel    = 1 + Level  Blöcke   (Level 0 = 1, Level 9 = 10)
    Punkte-Multiplikator  = 2 ** Level            (jede Stufe verdoppelt die Punkte)

Neue Stufen einfach unten anhängen (Level XI, XII, ...).
"""

# (roman, apples, length) je Stufe. Werte nach Wunsch anpassen.
PRESTIGE_LEVELS = [
    {"roman": "I",    "apples": 20,     "length": 10},
    {"roman": "II",   "apples": 50,     "length": 25},
    {"roman": "III",  "apples": 200,    "length": 63},
    {"roman": "IV",   "apples": 500,    "length": 158},
    {"roman": "V",    "apples": 1200,   "length": 395},
    {"roman": "VI",   "apples": 3000,   "length": 988},
    {"roman": "VII",  "apples": 7500,   "length": 2470},
    {"roman": "VIII", "apples": 18000,  "length": 6175},
    {"roman": "IX",   "apples": 45000,  "length": 15438},
    {"roman": "X",    "apples": 100000, "length": 38594},
]

MAX_PRESTIGE = len(PRESTIGE_LEVELS)


def blocks_per_apple(level):
    """Wie viele Körper-Blöcke ein Apfel bei diesem Prestige-Level bringt."""
    return 1 + max(0, int(level))


def score_multiplier(level):
    """Punkte-Multiplikator bei diesem Prestige-Level (verdoppelt sich je Stufe)."""
    return 2 ** max(0, int(level))


def roman(level):
    """Römische Anzeige für ein bereits erreichtes Level (Level 0 -> '')."""
    if level <= 0:
        return ""
    return PRESTIGE_LEVELS[min(level, MAX_PRESTIGE) - 1]["roman"]


def next_requirement(level):
    """
    Anforderung, um von 'level' auf die nächste Stufe zu kommen.
    Gibt ein dict {roman, apples, length} zurück - oder None, wenn 'level'
    bereits die höchste Stufe ist.
    """
    if level >= MAX_PRESTIGE:
        return None
    return PRESTIGE_LEVELS[level]
