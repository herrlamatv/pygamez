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

import random

# ===================================================== Level / Aufstieg
# Kumulierte Äpfel, ab denen die jeweilige Stufe (Level 1..N) erreicht ist.
# 15 Stufen: pro Level ein weiterer Apfel gleichzeitig auf dem Feld
# (Level 15 -> 16 Äpfel) und ein Punkt mehr beim Multiplikator (bis x16).
# Die Abstände wachsen gleichmäßig weiter, damit sich der Aufstieg bis ganz
# oben verdient anfühlt, ohne unmöglich zu werden.
LEVEL_STEPS = [4, 10, 18, 30, 46, 68, 96,
               132, 176, 230, 296, 376, 472, 586, 720]
MAX_LEVEL = len(LEVEL_STEPS)
BASE_APPLES = 1                     # so viele Äpfel liegen zu Beginn (Level 0)
MAX_APPLES = BASE_APPLES + MAX_LEVEL   # Obergrenze durch das Level-System


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
    """Gleichzeitig liegende Äpfel bei diesem Level (1 + Level)."""
    return BASE_APPLES + min(max(0, level), MAX_LEVEL)


def score_multiplier(level):
    """Punkte-Multiplikator im Competitive-Modus (steigt je Stufe: x1..x16)."""
    return 1 + max(0, level)


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
