# -*- coding: utf-8 -*-
"""
levels.py
=========
Das Levelsystem der Sammlung: aus **Erfolgspunkten** wird ein Level.

Jeder Erfolg hat je nach Seltenheitsstufe einen Punktwert (siehe
``achievements.RARITY``). Die Summe aller Punkte eines Profils steht in
``stats.pgzlamastats`` (``points``); daraus ergibt sich das Level.

Die Kurve ist bewusst einfach und nachrechenbar::

    Kosten für den Sprung von Level n auf n+1 :  20 + 8 * (n - 1)
    Punkte insgesamt bis Level n              :  4 * (n - 1) * (n + 3)

Level 2 kostet also 20 Punkte, Level 3 weitere 28, Level 10 liegt bei 468
Punkten. Wer alle Erfolge holt (~11.000 Punkte), landet bei ungefähr Level 50.

Dazu gibt es 14 **Ränge** - ein Titel, der alle paar Level wechselt und im
Fortschritt-Screen, in der Sidebar und im Level-Toast erscheint. Die Namen
liegen als ``level.rank<N>`` in den Sprachdateien.
"""

import math

# Grundkosten und Zuwachs je Level (siehe Formel oben).
_BASE = 20
_STEP = 8

# Ränge: (ab Level, i18n-Schlüssel, Farbe). Die Farbe färbt Rangtitel und
# Level-Abzeichen; sie passt in jedes Theme (kräftig, aber nicht grell).
RANKS = (
    (1,  "level.rank1",  "#9aa6bf"),
    (3,  "level.rank2",  "#7fc1a6"),
    (6,  "level.rank3",  "#6ecd8c"),
    (10, "level.rank4",  "#5ad4d4"),
    (14, "level.rank5",  "#589cff"),
    (18, "level.rank6",  "#8f7ef2"),
    (22, "level.rank7",  "#b07fe8"),
    (27, "level.rank8",  "#c86ad8"),
    (32, "level.rank9",  "#e05a7d"),
    (37, "level.rank10", "#e2725b"),
    (42, "level.rank11", "#f0a05a"),
    (47, "level.rank12", "#f5cd64"),
    (55, "level.rank13", "#ffd83b"),
    (65, "level.rank14", "#fff1a8"),
)


def cost(level):
    """Punkte, die der Sprung von 'level' auf 'level'+1 kostet."""
    return _BASE + _STEP * (max(1, int(level)) - 1)


def total_for(level):
    """Punkte, die man insgesamt für 'level' gesammelt haben muss."""
    n = max(1, int(level))
    return 4 * (n - 1) * (n + 3)


def level_for(points):
    """Level zu einer Punktzahl (Level 1 ab 0 Punkten)."""
    p = max(0, int(points))
    # Umkehrung von total_for: n = -1 + sqrt(4 + p/4)
    n = int(-1 + math.sqrt(4 + p / 4.0))
    n = max(1, n)
    # Rundungsfehler abfangen (die Formel ist exakt, float ist es nicht).
    while total_for(n + 1) <= p:
        n += 1
    while n > 1 and total_for(n) > p:
        n -= 1
    return n


def progress(points):
    """(Level, Punkte im Level, Punkte bis zum nächsten Level)."""
    p = max(0, int(points))
    level = level_for(p)
    base = total_for(level)
    return level, p - base, cost(level)


def rank(level):
    """(i18n-Schlüssel, Farbe) des Rangs für dieses Level."""
    key, color = RANKS[0][1], RANKS[0][2]
    for min_level, rank_key, rank_color in RANKS:
        if level >= min_level:
            key, color = rank_key, rank_color
        else:
            break
    return key, color


def rank_title(level):
    """Lokalisierter Rang-Titel für ein Level."""
    from i18n import t
    return t(rank(level)[0])


def rank_color(level):
    """Rang-Farbe als (r, g, b)."""
    value = rank(level)[1].lstrip("#")
    return tuple(int(value[i:i + 2], 16) for i in (0, 2, 4))


def rank_index(level):
    """Nummer des Rangs (0-basiert) - für Vergleiche/Erfolge."""
    idx = 0
    for i, (min_level, _key, _col) in enumerate(RANKS):
        if level >= min_level:
            idx = i
    return idx
