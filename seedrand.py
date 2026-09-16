# -*- coding: utf-8 -*-
"""
seedrand.py
===========
Ein kleiner Zufallsgenerator, der in Python UND im Browser exakt dieselben
Zahlen liefert (Gegenstück: ``web/js/games/seedrand.js``).

Pythons ``random`` und JavaScripts ``Math.random`` erzeugen aus demselben Seed
völlig verschiedene Folgen - ein "Level 12" wäre am PC ein anderes Rätsel als
im Browser. Für neue Inhalte (Tagesstrecke, Tageswort, Tages-Sudoku,
Sudoku-Varianten, Crossy-Road-Strecken) nutzen beide Versionen deshalb diesen
Generator: mulberry32, reine 32-Bit-Ganzzahl-Arithmetik, also auf jeder
Plattform bitgenau gleich. Auch ``randint``/``shuffle``/``choice`` sind hier
selbst geschrieben, damit sie die Zufallszahlen in derselben Reihenfolge
verbrauchen wie die JS-Fassung.

    rng = seedrand.Rand(seedrand.seed_from("crossy", 42))
    rng.randint(1, 6); rng.choice(liste); rng.shuffle(liste)

    seedrand.daily_seed("wordle")   # gleicher Seed für alle an einem Tag

Bestehende Sudoku-Level bleiben bewusst beim alten Generator - sonst wären
alle bereits abgehakten Level plötzlich andere Rätsel.
"""

import time

_M32 = 0xFFFFFFFF


def _imul(a, b):
    """32-Bit-Multiplikation wie JavaScripts Math.imul (vorzeichenlos)."""
    return (a * b) & _M32


class Rand:
    """mulberry32 - deterministisch, schnell, identisch zu seedrand.js."""

    __slots__ = ("state",)

    def __init__(self, seed=0):
        self.state = int(seed) & _M32

    def next_u32(self):
        """Nächste Zufallszahl als vorzeichenlose 32-Bit-Ganzzahl."""
        self.state = (self.state + 0x6D2B79F5) & _M32
        t = self.state
        t = _imul(t ^ (t >> 15), t | 1)
        t = ((t + _imul(t ^ (t >> 7), t | 61)) & _M32) ^ t
        return (t ^ (t >> 14)) & _M32

    def random(self):
        """Gleitkommazahl in [0, 1)."""
        return self.next_u32() / 4294967296.0

    def randint(self, a, b):
        """Ganzzahl in [a, b] (beide Grenzen eingeschlossen)."""
        return a + int(self.random() * (b - a + 1))

    def uniform(self, a, b):
        """Gleitkommazahl in [a, b)."""
        return a + (b - a) * self.random()

    def chance(self, p):
        """True mit Wahrscheinlichkeit p."""
        return self.random() < p

    def choice(self, seq):
        """Zufälliges Element einer (nicht leeren) Sequenz."""
        return seq[int(self.random() * len(seq))]

    def shuffle(self, items):
        """Mischt die Liste an Ort und Stelle (Fisher-Yates von hinten)."""
        for i in range(len(items) - 1, 0, -1):
            j = int(self.random() * (i + 1))
            items[i], items[j] = items[j], items[i]
        return items

    def weighted(self, weights):
        """Index gemäß Gewichten (Liste nicht-negativer Zahlen)."""
        # Bewusst eine schlichte Schleife statt sum(): Python rechnet sum() über
        # Gleitkommazahlen seit 3.12 mit Fehlerkompensation, JavaScript nicht -
        # bei krummen Gewichten könnten beide sonst minimal verschieden ziehen.
        total = 0.0
        for w in weights:
            total += w
        r = self.random() * total
        acc = 0.0
        for i, w in enumerate(weights):
            acc += w
            if r < acc:
                return i
        return len(weights) - 1


def seed_from(*parts):
    """32-Bit-Seed aus beliebigen Teilen (FNV-1a über den Text).

    Die Teile werden mit "|" verbunden; nur ASCII verwenden (die JS-Fassung
    rechnet mit Zeichencodes, bei ASCII identisch zu UTF-8-Bytes).
    """
    h = 0x811C9DC5
    for ch in "|".join(str(p) for p in parts):
        h ^= ord(ch) & 0xFF
        h = _imul(h, 0x01000193)
    return h


def today_str(t=None):
    """Heutiges lokales Datum als 'YYYY-MM-DD'."""
    return time.strftime("%Y-%m-%d", time.localtime(t))


def daily_seed(game, date=None):
    """Seed des Tages für ein Spiel - an einem Datum für alle gleich."""
    return seed_from("daily", game, date or today_str())


def day_index(date=None):
    """Fortlaufende Tagesnummer (Tage seit 2026-01-01) für ein Datum."""
    import datetime
    d = datetime.date.fromisoformat(date or today_str())
    return (d - datetime.date(2026, 1, 1)).days
