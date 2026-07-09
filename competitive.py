# -*- coding: utf-8 -*-
"""
competitive.py
==============
Feineinstellungen für den **Competitive-Modus** von Snake. Diese Datei sammelt
alle Kennzahlen an einem Ort und lässt sich frei anpassen (analog zu
``prestige.py``).

Der Competitive-Modus ist ein eigenständiger Endlos-Modus mit drei ineinander
greifenden Aufstiegs-/Zufalls-Mechaniken:

1. LEVEL ("Prestige")
   Man startet mit genau EINEM Apfel auf dem Feld - am Anfang bekommt man also
   nicht mehr Äpfel. Je mehr Äpfel man insgesamt sammelt, desto höher das Level;
   jede Stufe legt einen weiteren Apfel gleichzeitig aufs Feld (bis ``MAX_APPLES``)
   und erhöht den Punkte-Multiplikator. So bekommt man mit der Zeit "mehr Äpfel".
   Die einzelnen Level (1..15) stehen in ``games/levels/snake-comp.json`` und
   lassen sich dort ohne Code-Änderung erweitern.

2. BLAUER APFEL -> SLOT-MACHINE
   Ein blauer Apfel öffnet einen Spielautomaten. Eingesetzt wird die Größe
   (Länge) der Schlange. Drei Walzen drehen; das Ergebnis ist ein Multiplikator,
   der (a) den eingesetzten Teil der Länge vervielfacht oder verkleinert und
   (b) für kurze Zeit zusätzliche Äpfel spawnen lässt.

3. LILA APFEL
   Setzt 50 % der Länge aufs Spiel: dieser Teil wird mit einem zufälligen Faktor
   x0.5 .. x2.0 multipliziert (bzw. geteilt). Insgesamt schrumpft man so auf bis
   zu 75 % oder wächst auf bis zu 150 % der Länge.
"""

import json
import os
import random

# ===================================================== Level / Aufstieg
# Die Stufen (Level 1..N) stehen in einer eigenen Datei, damit man neue Level
# ohne Code-Änderung anhängen kann:  games/levels/snake-comp.json
# Jede Stufe hat: threshold (kumulierte Äpfel ab dieser Stufe), apples (wie viele
# gleichzeitig aufs Feld) und multiplier (Punkte-Faktor). Level 0 (Start) wird
# durch base_apples/base_multiplier bestimmt.
_LEVELS_FILE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                            "games", "levels", "snake-comp.json")

# Eingebaute Standardwerte (Spiegel der JSON) - greifen nur, falls die Datei
# fehlt oder fehlerhaft ist, damit der Modus nie ganz ausfällt.
_DEFAULT_BASE_APPLES = 1
_DEFAULT_BASE_MULT = 1
_DEFAULT_ROWS = [                       # (threshold, apples, multiplier)
    (4, 2, 2), (10, 3, 3), (18, 4, 4), (30, 5, 5), (46, 6, 6),
    (68, 7, 7), (96, 8, 8), (132, 9, 9), (176, 10, 10), (230, 11, 11),
    (296, 12, 12), (376, 13, 13), (472, 14, 14), (586, 15, 15), (720, 16, 16),
]


def _load_levels():
    """Liest die Level-Definition aus snake-comp.json.

    Rückgabe: (base_apples, base_mult, rows) mit rows als Liste von
    (threshold, apples, multiplier), aufsteigend nach threshold sortiert.
    Bei jedem Fehler (Datei fehlt/kaputt/leer) kommen die Standardwerte zurück.
    """
    base_apples, base_mult, rows = _DEFAULT_BASE_APPLES, _DEFAULT_BASE_MULT, _DEFAULT_ROWS
    try:
        with open(_LEVELS_FILE, "r", encoding="utf-8") as f:
            data = json.load(f)
        parsed = [(int(d["threshold"]), int(d["apples"]), int(d["multiplier"]))
                  for d in (data.get("levels") or [])]
        if parsed:                       # nur übernehmen, wenn wirklich Level drinstehen
            base_apples = int(data.get("base_apples", base_apples))
            base_mult = int(data.get("base_multiplier", base_mult))
            rows = sorted(parsed, key=lambda r: r[0])   # Reihenfolge in der Datei egal
    except Exception:
        pass                             # Datei fehlt/kaputt -> Standardwerte
    return base_apples, base_mult, rows


BASE_APPLES, BASE_MULT, _ROWS = _load_levels()
# Aus den Zeilen abgeleitete Nachschlage-Tabellen (Index = Level, 0..N):
LEVEL_STEPS = [r[0] for r in _ROWS]                     # Schwellen der Level 1..N
_APPLES_TAB = [BASE_APPLES] + [r[1] for r in _ROWS]     # Äpfel je Level (Index 0 = Start)
_MULT_TAB = [BASE_MULT] + [r[2] for r in _ROWS]         # Multiplikator je Level
MAX_LEVEL = len(LEVEL_STEPS)
MAX_APPLES = max(_APPLES_TAB)           # größte gleichzeitig liegende Apfelzahl


def level_for_apples(total):
    """Aktuelles Level für die insgesamt gesammelten Äpfel."""
    lvl = 0
    for need in LEVEL_STEPS:
        if total >= need:
            lvl += 1
        else:
            break
    return lvl


def apples_on_field(level):
    """Gleichzeitig liegende Äpfel bei diesem Level (aus der JSON-Tabelle)."""
    return _APPLES_TAB[max(0, min(level, MAX_LEVEL))]


def score_multiplier(level):
    """Punkte-Multiplikator im Competitive-Modus (aus der JSON-Tabelle)."""
    return _MULT_TAB[max(0, min(level, MAX_LEVEL))]


def next_step(total):
    """(erreicht, benötigt) Äpfel innerhalb der aktuellen Stufe.

    Gibt ``None`` zurück, wenn bereits die Maximalstufe erreicht ist.
    """
    lvl = level_for_apples(total)
    if lvl >= MAX_LEVEL:
        return None
    prev = LEVEL_STEPS[lvl - 1] if lvl > 0 else 0
    return (total - prev, LEVEL_STEPS[lvl] - prev)


# ===================================================== Slot-Machine (blau)
# Symbole: Schlüssel -> Farbe. Gezeichnet werden sie als kleine Icons (snake.py).
SLOT_SYMBOLS = {
    "seven":  (255, 210, 90),
    "gem":    (110, 220, 235),
    "bell":   (255, 190, 70),
    "apple":  (240, 90, 90),
    "cherry": (235, 110, 160),
}

# Walzenband: seltenere Symbole = wertvoller. Die Häufigkeit steuert die Chance.
REEL = (["cherry"] * 5 + ["apple"] * 4 + ["bell"] * 3 + ["gem"] * 2 + ["seven"] * 1)

# Multiplikator bei drei gleichen Symbolen (Jackpot) ...
_TRIPLE = {"seven": 6.0, "gem": 4.0, "bell": 3.0, "apple": 2.5, "cherry": 2.0}
PAIR_MULT = 1.5                     # ... bei zwei gleichen ...
MISS_MULT = 0.5                     # ... bei nichts gleich (Einsatz halbiert).


def spin_reels():
    """Zieht drei Walzen-Symbole."""
    return [random.choice(REEL) for _ in range(3)]


def slot_outcome(reels):
    """Wertet die Walzen aus und liefert (multiplikator, ergebnis-schlüssel)."""
    a, b, c = reels
    if a == b == c:
        return _TRIPLE[a], "jackpot"
    if a == b or b == c or a == c:
        return PAIR_MULT, "pair"
    return MISS_MULT, "miss"


# ===================================================== Lila Apfel
PURPLE_MIN, PURPLE_MAX = 0.5, 2.0


def purple_factor():
    """Zufälliger Faktor x0.5 .. x2.0 für den lila Apfel."""
    return round(random.uniform(PURPLE_MIN, PURPLE_MAX), 2)
