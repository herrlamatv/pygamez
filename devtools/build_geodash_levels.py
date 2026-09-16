# -*- coding: utf-8 -*-
"""
build_geodash_levels.py
=======================
Baut die 8 eingebauten Geometry-Dash-Level, beweist mit dem Solver, dass
jedes davon schaffbar ist (inklusive aller drei Münzen, robust gegen ±1
Schritt Versatz), und schreibt:

    games/levels/geodash.json        die Level (Spiel, Desktop)
    web/js/games/geodash_levels.js   dieselben Level für den Browser
    devtools/geodash_proofs.json     die Lösungen (Umschalt-Schritte + Hash)

Aufruf aus dem Repo-Root:

    python devtools/build_geodash_levels.py              # alles bauen + lösen
    python devtools/build_geodash_levels.py --only 3     # nur Level 3 lösen
    python devtools/build_geodash_levels.py --preview D  # Übersichtsbilder nach D
    python devtools/build_geodash_levels.py --no-solve   # nur Level schreiben
                                                          (vorhandene Lösungen
                                                          werden nur geprüft)

Die Level sind aus Bausteinen zusammengesetzt (Stachelreihen, Treppen,
Pad-Sprünge, Orb-Ketten, Schiffs-Tore, Ball-Wechsel, Wellen-Zickzack ...).
Die Abstände richten sich nach der Sprungweite beim jeweiligen Tempo, so
fallen die Sprünge auf den Takt der Musik.
"""

import json
import os
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, ROOT)
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# Den Kern direkt laden (nicht über das Paket games, das alle Spiele
# samt pygame importieren würde).
sys.path.insert(0, os.path.join(ROOT, "games"))
import geodash_core as core  # noqa: E402
import build_geodash_solver as solver  # noqa: E402

OUT_JSON = os.path.join(ROOT, "games", "levels", "geodash.json")
OUT_JS = os.path.join(ROOT, "web", "js", "games", "geodash_levels.js")
OUT_PROOFS = os.path.join(ROOT, "devtools", "geodash_proofs.json")


# ---------------------------------------------------------------------------
#  Baukasten
# ---------------------------------------------------------------------------

class LB:
    """Level-Baukasten: sammelt Objekte, x/y in Blöcken."""

    def __init__(self):
        self.objs = []

    def o(self, kind, x, y=0, rot=0, *par):
        self.objs.append([kind, int(x), int(y), int(rot)] + [int(p) for p in par])
        return self

    # --- Grundformen -----------------------------------------------------
    def blocks(self, x, y, w=1, h=1):
        for i in range(w):
            for j in range(h):
                self.o("block", x + i, y + j)
        return self

    def spikes(self, x, n=1, y=0, rot=0, small=False):
        for i in range(n):
            self.o("spike_s" if small else "spike", x + i, y, rot)
        return self

    def pit(self, x, w):
        for i in range(w):
            self.o("pit", x + i, 0)
        return self

    def col(self, x, y0, y1):
        """Senkrechte Säule aus Blöcken von y0 bis einschließlich y1."""
        for y in range(y0, y1 + 1):
            self.o("block", x, y)
        return self

    def gate(self, x, gap_lo, gap_hi, top=10, w=1, spikes=True):
        """Tor für Schiff/UFO/Welle: Wand mit Lücke gap_lo..gap_hi (Reihen,
        einschließlich), Stacheln an den Lückenrändern."""
        for i in range(w):
            if gap_lo > 0:
                self.col(x + i, 0, gap_lo - 1)
            if gap_hi < top - 1:
                self.col(x + i, gap_hi + 1, top - 1)
        if spikes:
            for i in range(w):
                if gap_lo > 0:
                    self.o("spike_s", x + i, gap_lo, 0)
                if gap_hi < top - 1:
                    self.o("spike_s", x + i, gap_hi, 2)
        return self

    def funnel(self, x, lo, top=10, w=3, kind=None, *par, speed=None):
        """Zwangs-Durchgang für Flugformen: Wände ober- und unterhalb der drei
        Reihen lo..lo+2 in w Spalten ab x - optional mit einem Portal in der
        Mitte (und einem Tempo-Portal direkt dahinter). So fliegt niemand
        über ein Portal hinweg."""
        if speed:
            w = max(w, 5)
        for i in range(w):
            if lo > 0:
                self.col(x + i, 0, lo - 1)
            if lo + 3 <= top - 1:
                self.col(x + i, lo + 3, top - 1)
        if kind and speed:
            self.o(kind, x + 1, lo + 1, 0, *par)
            self.o(speed, x + 3, lo + 1)
        elif kind:
            self.o(kind, x + w // 2, lo + 1, 0, *par)
        return self

    def roof(self, x, w, row):
        """Durchgehende Blockreihe (Decke, auf der man kopfüber läuft)."""
        return self.blocks(x, row, w, 1)

    def tunnel(self, x, centers, half, top, spikes=False):
        """Wellen-Tunnel: je Spalte offen von centers[i]-half bis
        centers[i]+half (Reihen, einschließlich), sonst Wand."""
        for i, c in enumerate(centers):
            lo, hi = c - half, c + half
            if lo > 0:
                self.col(x + i, 0, lo - 1)
            if hi < top - 1:
                self.col(x + i, hi + 1, top - 1)
        return self

    def color(self, x, target, rgb, dur=8):
        self.o("color", x, 0, 0, target, rgb[0], rgb[1], rgb[2], dur)
        return self

    def colors(self, x, bg, ground, dur=10):
        self.color(x, 0, bg, dur)
        self.color(x, 1, ground, dur)
        return self

    def coin(self, x, y):
        self.o("coin", x, y)
        return self


def zigzag(start, lo, hi, n, run=1):
    """Mittelreihen eines Zickzacks: von start hoch bis hi, runter bis lo ...
    'run' Spalten je Reihe (1 = 45°, 2 = flacher)."""
    out = []
    r, d = start, 1
    while len(out) < n:
        for _ in range(run):
            out.append(r)
        if r >= hi:
            d = -1
        elif r <= lo:
            d = 1
        r += d
    return out[:n]


def level(meta, lb):
    d = dict(meta)
    d["v"] = 1
    d["objects"] = lb.objs
    d = core.normalize_level(d)
    return d


# ---------------------------------------------------------------------------
#  Level 1 - Lama Launch (Leicht, 1 Stern)
# ---------------------------------------------------------------------------

def lvl_lama_launch():
    L = LB()
    # Aufwärmen: einzelne Stacheln im Takt
    L.spikes(18)
    L.spikes(28)
    L.spikes(38, 2)
    L.blocks(48, 0, 4, 1)
    L.spikes(57)
    L.blocks(64, 0, 2, 1).blocks(66, 0, 3, 2)
    L.spikes(76)
    L.spikes(85, 2)
    # Pad auf die Hochebene - oben wartet die erste Münze (nur mit Sprung)
    L.o("pad_y", 94)
    L.blocks(96, 3, 8, 1)
    L.coin(100, 5)
    L.spikes(112)
    L.spikes(121, 2)
    # Orb über der Doppelreihe: wer klickt, fliegt höher - zur zweiten Münze
    L.spikes(131, 2)
    L.o("orb_y", 131, 2)
    L.coin(133, 5)
    L.colors(142, (170, 60, 255), (110, 30, 200))
    # Schiff: sanfte Tore
    L.o("p_ship", 148, 1)
    L.gate(164, 3, 7)
    L.gate(180, 1, 5)
    L.gate(196, 4, 8)
    L.coin(204, 8)
    L.gate(212, 2, 6)
    L.gate(228, 3, 7)
    L.funnel(242, 0, 10, 3, "p_cube")
    L.colors(243, (40, 170, 110), (20, 120, 80))
    # Würfel-Finale
    L.spikes(256)
    L.blocks(264, 0, 3, 1)
    L.blocks(270, 0, 3, 1)
    L.spikes(279, 2)
    L.blocks(288, 0, 2, 1)
    L.spikes(292, 2)
    L.coin(292, 4)
    L.spikes(302)
    L.spikes(311, 2)
    return level({"id": "lama-launch", "name": "Lama Launch", "difficulty": 0,
                  "stars": 1, "speed": 1, "mode": 0, "bg": [40, 110, 255],
                  "ground": [20, 70, 200], "music": "drive", "bpm": 128,
                  "length": 328}, L)


# ---------------------------------------------------------------------------
#  Level 2 - Neon Steps (Leicht, 2 Sterne)
# ---------------------------------------------------------------------------

def lvl_neon_steps():
    L = LB()
    L.spikes(18)
    # Treppe hinauf - wer am Ende abspringt, holt die Münze
    L.blocks(26, 0, 3, 1).blocks(29, 0, 3, 2).blocks(32, 0, 4, 3)
    L.coin(38, 5)
    L.spikes(46)
    L.o("pad_p", 54)
    L.spikes(56)
    L.blocks(64, 0, 6, 1)
    L.spikes(67, 1, 1)
    L.spikes(77, 2)
    # Hochplattform mit Stachel
    L.o("pad_y", 86)
    L.blocks(88, 3, 8, 1)
    L.spikes(92, 1, 4)
    L.coin(98, 6)
    L.spikes(108, 2)
    L.spikes(118, 3)
    L.o("orb_y", 119, 2)
    L.spikes(130)
    L.spikes(131, 1, 0, 0, True)
    L.colors(136, (255, 60, 150), (180, 30, 110))
    L.o("p_ship", 142, 1)
    L.gate(156, 2, 6)
    L.gate(170, 4, 8)
    L.gate(184, 1, 5)
    L.coin(192, 1)
    L.gate(198, 3, 7)
    L.gate(212, 5, 9)
    L.funnel(226, 0, 10, 3, "p_cube")
    L.colors(227, (0, 170, 200), (0, 110, 150))
    L.spikes(238)
    L.blocks(246, 0, 2, 1).blocks(250, 0, 2, 2)
    L.spikes(252, 2)
    L.blocks(254, 0, 3, 1)
    L.spikes(264, 2)
    L.o("pad_p", 273)
    L.spikes(275)
    L.spikes(284, 2)
    L.o("orb_y", 294, 2)
    L.spikes(293, 3)
    L.spikes(305)
    return level({"id": "neon-steps", "name": "Neon Steps", "difficulty": 0,
                  "stars": 2, "speed": 1, "mode": 0, "bg": [0, 170, 200],
                  "ground": [0, 110, 150], "music": "chip", "bpm": 140,
                  "length": 320}, L)


# ---------------------------------------------------------------------------
#  Level 3 - Orbit Garden (Normal, 3 Sterne)
# ---------------------------------------------------------------------------

def lvl_orbit_garden():
    L = LB()
    L.spikes(18, 2)
    L.spikes(28)
    L.blocks(29, 0, 2, 1)
    L.spikes(31)
    # Orb-Kette über dem Stachelbett
    L.spikes(42, 5)
    L.o("orb_y", 43, 2)
    L.coin(45, 5)
    L.spikes(58, 2)
    L.o("pad_p", 67)
    L.spikes(69)
    L.o("orb_p", 73, 2)
    L.spikes(72, 3)
    # Blauer Orb: an die Decke - kopfüber weiter
    L.colors(80, (60, 40, 170), (40, 20, 120))
    L.o("orb_b", 84, 2)
    L.roof(84, 35, 6)
    L.spikes(87, 29)
    L.spikes(96, 1, 5, 2)
    L.spikes(105, 2, 5, 2)
    L.coin(101, 3)
    L.spikes(114, 1, 5, 2)
    L.o("g_norm", 118, 4)
    L.colors(124, (230, 90, 180), (160, 50, 130))
    L.spikes(130, 2)
    # Schiff mit doppeltem Tempo
    L.o("s_fast", 138, 1)
    L.o("p_ship", 144, 1)
    L.gate(160, 2, 6)
    L.gate(180, 5, 9)
    L.gate(200, 1, 5)
    L.gate(220, 4, 8)
    L.coin(229, 9)
    L.gate(238, 2, 6)
    L.funnel(254, 0, 10, 3, "p_cube", speed="s_norm")
    L.colors(259, (80, 60, 220), (50, 40, 160))
    L.spikes(270, 3)
    L.o("pad_p", 280)
    L.spikes(282)
    L.spikes(291, 2)
    L.blocks(300, 0, 3, 1)
    L.spikes(303, 3)
    L.o("orb_y", 304, 3)
    L.blocks(306, 0, 3, 1)
    L.spikes(316, 2)
    return level({"id": "orbit-garden", "name": "Orbit Garden", "difficulty": 1,
                  "stars": 3, "speed": 1, "mode": 0, "bg": [80, 60, 220],
                  "ground": [50, 40, 160], "music": "dream", "bpm": 120,
                  "length": 332}, L)


# ---------------------------------------------------------------------------
#  Level 4 - Gravity Tide (Schwer, 4 Sterne)
# ---------------------------------------------------------------------------

def lvl_gravity_tide():
    L = LB()
    L.spikes(18, 2)
    L.spikes(27, 2)
    # Schwerkraft-Portal: kopfüber unter der Decke
    L.colors(34, (0, 140, 130), (0, 90, 90))
    L.o("g_flip", 40, 1)
    L.roof(36, 41, 7)
    L.spikes(45, 30)
    L.spikes(52, 1, 6, 2)
    L.spikes(61, 2, 6, 2)
    L.coin(66, 4)
    L.spikes(71, 1, 6, 2)
    L.o("g_norm", 76, 5)
    L.spikes(90, 2)
    # Ball im niedrigen Korridor
    L.colors(96, (230, 110, 30), (160, 70, 10))
    L.o("p_ball", 100, 1, 0, 7)
    L.spikes(112, 4)
    L.spikes(124, 4, 6, 2)
    L.coin(130, 5)
    L.spikes(136, 4)
    L.spikes(148, 4, 6, 2)
    L.spikes(160, 3)
    L.spikes(162, 3, 6, 2)
    L.o("pad_b", 172, 0)
    L.spikes(176, 6)
    L.o("pad_b", 188, 6, 2)
    L.spikes(190, 4, 6, 2)
    L.funnel(200, 0, 7, 3, "p_ufo", 9)
    # UFO
    L.colors(201, (120, 50, 200), (80, 30, 140))
    L.gate(214, 2, 5, 9)
    L.gate(228, 4, 7, 9)
    L.coin(236, 8)
    L.gate(244, 1, 4, 9)
    L.gate(258, 3, 6, 9)
    L.funnel(272, 0, 9, 3, "p_cube")
    # Blaues Pad unter die Decke und zurück
    L.colors(273, (0, 140, 130), (0, 90, 90))
    L.spikes(284, 2)
    L.o("pad_b", 294, 0)
    L.roof(292, 25, 6)
    L.spikes(297, 18)
    L.spikes(302, 1, 5, 2)
    L.spikes(309, 2, 5, 2)
    L.o("pad_b", 315, 5, 2)
    L.spikes(326, 2)
    L.spikes(335, 3)
    return level({"id": "gravity-tide", "name": "Gravity Tide", "difficulty": 2,
                  "stars": 4, "speed": 1, "mode": 0, "bg": [0, 140, 130],
                  "ground": [0, 90, 90], "music": "drive", "bpm": 136,
                  "length": 350}, L)


# ---------------------------------------------------------------------------
#  Level 5 - Skyline Rush (Schwer, 5 Sterne)
# ---------------------------------------------------------------------------

def lvl_skyline_rush():
    L = LB()
    L.spikes(22, 2)
    L.pit(32, 4)
    L.blocks(42, 0, 3, 1)
    L.pit(45, 4)
    L.blocks(49, 0, 3, 1)
    L.spikes(60, 3)
    L.o("pad_y", 70)
    L.pit(72, 18)
    L.blocks(74, 2, 12, 1)
    L.spikes(79, 1, 3)
    L.coin(88, 5)
    L.spikes(96, 2)
    # UFO über der Stadt
    L.colors(100, (255, 120, 40), (190, 70, 20))
    L.o("p_ufo", 106, 1)
    L.gate(122, 1, 4)
    L.gate(138, 5, 8)
    L.gate(154, 2, 5)
    L.coin(162, 1)
    L.gate(170, 6, 9)
    L.gate(186, 3, 6)
    L.funnel(200, 0, 10, 3, "p_cube")
    # Kurzer Sprint mit dreifachem Tempo
    L.colors(201, (250, 60, 90), (170, 30, 60))
    L.o("s_vfast", 208, 1)
    L.spikes(222)
    L.spikes(234, 2)
    L.blocks(246, 0, 4, 1)
    L.pit(250, 5)
    L.blocks(255, 0, 4, 1)
    L.spikes(268, 3)
    L.o("s_fast", 280, 1)
    # Schiff mit engeren Toren
    L.colors(284, (255, 120, 40), (190, 70, 20))
    L.o("p_ship", 288, 1)
    L.gate(304, 2, 5)
    L.gate(320, 5, 8)
    L.coin(327, 9)
    L.gate(336, 1, 4)
    L.gate(352, 4, 7)
    L.gate(368, 2, 5)
    L.funnel(382, 0, 10, 3, "p_cube")
    L.colors(383, (250, 60, 90), (170, 30, 60))
    L.spikes(396, 2)
    L.pit(406, 4)
    L.o("orb_y", 417, 2)
    L.spikes(415, 5)
    L.spikes(430, 3)
    return level({"id": "skyline-rush", "name": "Skyline Rush", "difficulty": 2,
                  "stars": 5, "speed": 2, "mode": 0, "bg": [250, 60, 90],
                  "ground": [170, 30, 60], "music": "chip", "bpm": 150,
                  "length": 446}, L)


# ---------------------------------------------------------------------------
#  Level 6 - Crystal Caves (Schwerer, 7 Sterne)
# ---------------------------------------------------------------------------

def lvl_crystal_caves():
    L = LB()
    L.spikes(18, 3)
    L.blocks(28, 0, 3, 1)
    L.spikes(29, 1, 1, 0, True)
    L.spikes(34, 3)
    L.pit(44, 5)
    L.spikes(56, 4)
    L.o("orb_p", 57, 2)
    L.blocks(66, 0, 2, 2)
    L.spikes(68, 3)
    L.o("orb_y", 70, 3)
    L.blocks(72, 0, 2, 2)
    # Blauer Orb über die Grube: kurzer Umweg an der Decke zur Münze
    L.roof(84, 12, 7)
    L.o("orb_b", 81, 2)
    L.pit(82, 16)
    L.spikes(88, 1, 6, 2)
    L.coin(91, 4)
    L.o("orb_b", 96, 5)
    L.colors(104, (20, 160, 200), (10, 100, 140))
    # Ball, doppeltes Tempo, niedriger Korridor
    L.o("s_fast", 108, 1)
    L.o("p_ball", 114, 1, 0, 6)
    L.spikes(126, 3)
    L.spikes(136, 3, 5, 2)
    L.spikes(145, 3)
    L.spikes(153, 2, 5, 2)
    L.spikes(160, 3)
    L.o("orb_b", 170, 2)
    L.spikes(167, 8)
    L.spikes(177, 3, 5, 2)
    L.spikes(186, 3)
    L.funnel(194, 0, 6, 3, "p_ufo", 8, speed="s_norm")
    L.colors(199, (120, 40, 190), (70, 20, 130))
    # UFO durch enge Kristall-Tore
    L.gate(212, 1, 3, 8)
    L.gate(224, 4, 6, 8)
    L.coin(229, 0)
    L.gate(236, 1, 3, 8)
    L.gate(248, 3, 5, 8)
    L.gate(260, 5, 7, 8)
    L.gate(272, 2, 4, 8)
    L.funnel(284, 1, 8, 3, "p_ship", 7, speed="s_fast")
    L.colors(290, (20, 160, 200), (10, 100, 140))
    # Schiff im niedrigen Stollen
    L.gate(304, 1, 3, 7)
    L.gate(318, 4, 6, 7)
    L.gate(332, 2, 4, 7)
    L.coin(338, 6)
    L.gate(346, 4, 6, 7)
    L.gate(360, 1, 3, 7)
    L.funnel(372, 0, 7, 3, "p_cube")
    L.colors(373, (120, 40, 190), (70, 20, 130))
    L.spikes(386, 2)
    L.pit(396, 5)
    L.blocks(401, 0, 3, 1)
    L.spikes(404, 1, 0)
    L.pit(405, 4)
    L.blocks(409, 0, 3, 1)
    L.spikes(420, 3)
    L.o("pad_y", 430)
    L.spikes(432, 4)
    L.spikes(446, 3)
    return level({"id": "crystal-caves", "name": "Crystal Caves", "difficulty": 3,
                  "stars": 7, "speed": 1, "mode": 0, "bg": [120, 40, 190],
                  "ground": [70, 20, 130], "music": "dream", "bpm": 132,
                  "length": 462}, L)


# ---------------------------------------------------------------------------
#  Level 7 - Wave Reactor (Wahnsinnig, 9 Sterne)
# ---------------------------------------------------------------------------

def lvl_wave_reactor():
    L = LB()
    L.spikes(22, 3)
    L.blocks(34, 0, 4, 1)
    L.spikes(38, 3)
    L.blocks(41, 0, 4, 1)
    L.o("orb_y", 52, 2)
    L.spikes(50, 5)
    L.o("pad_p", 62)
    L.spikes(64, 2)
    L.colors(70, (0, 200, 120), (0, 130, 80))
    # Welle: Zickzack-Tunnel
    L.o("p_wave", 76, 1, 0, 8)
    L.tunnel(88, zigzag(3, 2, 5, 36), 2, 8)
    L.coin(125, 6)
    L.gate(130, 5, 7, 8, 1, False)
    L.gate(138, 1, 3, 8, 1, False)
    L.gate(146, 4, 6, 8, 1, False)
    L.gate(154, 1, 3, 8, 1, False)
    L.funnel(164, 1, 8, 3, "p_ship")
    L.colors(165, (255, 190, 0), (190, 120, 0))
    # Schiff mit Stachel-Toren
    L.gate(180, 5, 8, 10)
    L.gate(194, 1, 4, 10)
    L.gate(208, 4, 7, 10)
    L.gate(222, 0, 3, 10)
    L.coin(229, 0)
    L.gate(236, 3, 6, 10)
    L.funnel(250, 2, 10, 3, "p_ufo", 9)
    L.colors(251, (0, 200, 120), (0, 130, 80))
    # UFO
    L.gate(264, 1, 3, 9)
    L.gate(276, 5, 7, 9)
    L.gate(288, 2, 4, 9)
    L.gate(300, 6, 8, 9)
    L.coin(305, 8)
    L.gate(312, 3, 5, 9)
    L.funnel(324, 2, 9, 3, "p_wave", 8, speed="s_vfast")
    L.colors(330, (255, 40, 120), (180, 20, 80))
    # Welle mit dreifachem Tempo
    L.tunnel(340, zigzag(3, 2, 5, 44, 2), 2, 8)
    L.tunnel(384, zigzag(3, 1, 6, 30), 1, 8)
    L.funnel(414, 2, 8, 3, "p_cube")
    L.o("s_fast", 420, 1)
    L.colors(420, (0, 200, 120), (0, 130, 80))
    L.spikes(434, 3)
    L.pit(446, 5)
    L.spikes(458, 2)
    L.spikes(468, 3)
    return level({"id": "wave-reactor", "name": "Wave Reactor", "difficulty": 4,
                  "stars": 9, "speed": 2, "mode": 0, "bg": [0, 200, 120],
                  "ground": [0, 130, 80], "music": "drive", "bpm": 160,
                  "length": 486}, L)


# ---------------------------------------------------------------------------
#  Level 8 - Lama Inferno (Dämon, 10 Sterne)
# ---------------------------------------------------------------------------

def lvl_lama_inferno():
    L = LB()
    L.spikes(26, 3)
    L.blocks(38, 0, 3, 1)
    L.spikes(41, 3)
    L.blocks(44, 0, 3, 2)
    L.pit(50, 4)
    L.spikes(62, 3)
    L.o("orb_y", 72, 2)
    L.spikes(70, 4)
    L.o("orb_p", 78, 3)
    L.spikes(74, 7)
    L.o("orb_b", 88, 2)
    L.roof(86, 18, 7)
    L.pit(84, 20)
    L.spikes(94, 2, 6, 2)
    L.coin(99, 3)
    L.o("orb_b", 104, 5)
    L.spikes(112, 3)
    L.colors(118, (200, 20, 20), (120, 10, 10))
    # Welle, dreifaches Tempo, enger Tunnel
    L.o("p_wave", 124, 1, 0, 7)
    L.tunnel(134, zigzag(2, 2, 4, 30, 2), 2, 7)
    L.tunnel(164, zigzag(3, 1, 5, 32), 1, 7)
    L.funnel(196, 1, 7, 3, "p_ship", 7, speed="s_fast")
    L.colors(202, (255, 90, 0), (170, 50, 0))
    # Schiff durch die Glut
    L.gate(216, 4, 6, 7)
    L.gate(228, 1, 3, 7)
    L.gate(240, 3, 5, 7)
    L.coin(246, 1)
    L.gate(252, 2, 4, 7)
    L.gate(264, 4, 6, 7)
    L.funnel(276, 0, 7, 3, "p_ball", 6, speed="s_vfast")
    L.colors(282, (200, 20, 20), (120, 10, 10))
    # Ball
    L.spikes(292, 4)
    L.spikes(301, 4, 5, 2)
    L.spikes(310, 4)
    L.spikes(319, 4, 5, 2)
    L.spikes(328, 2)
    L.spikes(330, 3, 5, 2)
    L.funnel(340, 0, 6, 3, "p_ufo", 8, speed="s_fast")
    L.colors(346, (255, 90, 0), (170, 50, 0))
    # UFO
    L.gate(358, 4, 6, 8)
    L.gate(370, 1, 3, 8)
    L.gate(382, 5, 7, 8)
    L.coin(388, 7)
    L.gate(394, 2, 4, 8)
    L.funnel(406, 2, 8, 3, "p_wave", 7, speed="s_vfast")
    L.colors(412, (140, 0, 40), (80, 0, 20))
    L.tunnel(420, zigzag(3, 1, 5, 40), 1, 7)
    L.funnel(460, 2, 7, 3, "p_cube")
    L.spikes(474, 3)
    L.blocks(484, 0, 3, 1)
    L.spikes(487, 3)
    L.blocks(490, 0, 3, 2)
    L.pit(493, 6)
    L.spikes(506, 3)
    return level({"id": "lama-inferno", "name": "Lama Inferno", "difficulty": 5,
                  "stars": 10, "speed": 3, "mode": 0, "bg": [140, 0, 40],
                  "ground": [80, 0, 20], "music": "dark", "bpm": 174,
                  "length": 524}, L)


LEVELS = [lvl_lama_launch, lvl_neon_steps, lvl_orbit_garden, lvl_gravity_tide,
          lvl_skyline_rush, lvl_crystal_caves, lvl_wave_reactor,
          lvl_lama_inferno]



# ---------------------------------------------------------------------------
#  Vorschau
# ---------------------------------------------------------------------------

def preview(levels, proofs, folder):
    os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
    import pygame
    pygame.init()
    px = 10
    rows_per_img = 12
    for d in levels:
        lv = core.Level(d)
        per_row = 200
        n_rows = (lv.length + per_row - 1) // per_row
        h_row = rows_per_img * px + 10
        surf = pygame.Surface((per_row * px + 20, n_rows * h_row + 10))
        surf.fill((20, 20, 30))
        path = []
        pr = proofs.get(d["id"])
        if pr:
            s = core.new_state(lv)
            tg = pr["toggles"]
            ti, held, k = 0, False, 0
            while not s.dead and not s.won and k < 200000:
                was = held
                while ti < len(tg) and tg[ti] <= k:
                    held = not held
                    ti += 1
                core.step(s, lv, held, held and not was)
                k += 1
                if k % 4 == 0:
                    path.append((s.x, s.y, s.mode, held))
        cols = {"block": (200, 200, 220), "half": (170, 170, 200),
                "spike": (255, 70, 70), "spike_s": (255, 120, 90),
                "pit": (0, 0, 0), "pad_y": (255, 220, 0), "pad_p": (255, 80, 220),
                "pad_b": (60, 200, 255), "orb_y": (255, 220, 0),
                "orb_p": (255, 80, 220), "orb_b": (60, 200, 255),
                "coin": (255, 200, 60)}
        for r in range(n_rows):
            base_y = 10 + r * h_row + rows_per_img * px
            pygame.draw.line(surf, (80, 80, 120), (10, base_y), (10 + per_row * px, base_y))
        for o in d["objects"]:
            r = o[1] // per_row
            x = 10 + (o[1] - r * per_row) * px
            base_y = 10 + r * h_row + rows_per_img * px
            y = base_y - (o[2] + 1) * px
            k = o[0]
            if k.startswith("p_") or k.startswith("g_") or k.startswith("s_"):
                c = {"p_cube": (80, 255, 80), "p_ship": (255, 100, 200),
                     "p_ball": (255, 120, 40), "p_ufo": (255, 170, 40),
                     "p_wave": (60, 160, 255), "g_norm": (60, 200, 255),
                     "g_flip": (255, 230, 60)}.get(k, (200, 255, 255))
                pygame.draw.rect(surf, c, (x + 2, y - px, px - 4, px * 3), 2)
            elif k == "color":
                pygame.draw.line(surf, (o[5], o[6], o[7]), (x, base_y - rows_per_img * px), (x, base_y), 1)
            elif k.startswith("orb") or k == "coin":
                pygame.draw.circle(surf, cols[k], (x + px // 2, y + px // 2), px // 2 - 1, 0 if k != "coin" else 2)
            elif k.startswith("spike"):
                pts = {0: [(x, y + px), (x + px, y + px), (x + px // 2, y)],
                       2: [(x, y), (x + px, y), (x + px // 2, y + px)],
                       1: [(x, y), (x, y + px), (x + px, y + px // 2)],
                       3: [(x + px, y), (x + px, y + px), (x, y + px // 2)]}[o[3]]
                if k == "spike_s":
                    pts = [(a, b if b == max(p[1] for p in pts) or o[3] != 0 else b + px // 2) for a, b in pts]
                pygame.draw.polygon(surf, cols[k], pts)
            elif k == "pit":
                pygame.draw.rect(surf, (255, 0, 0), (x, base_y - 1, px, 3))
            elif k == "half":
                pygame.draw.rect(surf, cols[k], (x, y + px // 2, px, px // 2))
            else:
                pygame.draw.rect(surf, cols.get(k, (255, 255, 255)), (x, y, px - 1, px - 1))
        for (sx, sy, mode, held) in path:
            bx = sx / core.B
            r = int(bx // per_row)
            x = 10 + (bx - r * per_row) * px + px // 2
            base_y = 10 + r * h_row + rows_per_img * px
            y = base_y - (sy / core.B) * px - px // 2
            surf.set_at((int(x), int(y)), (0, 255, 0) if held else (0, 160, 255))
        pygame.image.save(surf, os.path.join(folder, "gd-%s.png" % d["id"]))


# ---------------------------------------------------------------------------
#  Hauptprogramm
# ---------------------------------------------------------------------------

def main():
    args = sys.argv[1:]
    only = None
    if "--only" in args:
        only = int(args[args.index("--only") + 1])
    levels = [fn() for fn in LEVELS]
    try:
        with open(OUT_PROOFS, "r", encoding="utf-8") as f:
            proofs = json.load(f)
    except (OSError, ValueError):
        proofs = {}
    if "--preview" in args:
        folder = args[args.index("--preview") + 1]
        os.makedirs(folder, exist_ok=True)
        preview(levels if only is None else [levels[only - 1]], proofs, folder)
        return
    ok_all = True
    for i, d in enumerate(levels, 1):
        lv = core.Level(d)
        pr = proofs.get(d["id"])
        solve = "--no-solve" not in args and (only is None or only == i)
        if pr and not solve:
            ok = all(solver.verify(lv, pr["toggles"], sh)[0] for sh in (-1, 0, 1))
            print("Level %d %-16s Lösung %s" % (i, d["id"], "gültig" if ok else "UNGÜLTIG"))
            ok_all &= ok
            continue
        if not solve:
            print("Level %d %-16s keine Lösung" % (i, d["id"]))
            ok_all = False
            continue
        t0 = time.time()
        tg, info = solver.solve(lv, need_coins=True, max_nodes=3000000)
        if tg is None:
            print("Level %d %-16s NICHT GELÖST  %s  (bis x=%.1f)" % (
                i, d["id"], info, info.get("best_x", 0) / core.B))
            ok_all = False
            continue
        oks = [solver.verify(lv, tg, sh)[0] for sh in (-1, 0, 1)]
        s, _ = core.run_toggles(lv, tg)
        proofs[d["id"]] = {"toggles": tg, "steps": s.step,
                           "hash": core.state_hash(s),
                           "content": core.content_hash(d)}
        print("Level %d %-16s gelöst in %.1f s (%d Knoten, %d Schritte = %.1f s, "
              "%d Drücke) ±1: %s" % (i, d["id"], time.time() - t0, info["nodes"],
                                    s.step, s.step / core.HZ, len(tg) // 2, oks))
        ok_all &= all(oks)
    # Schreiben
    payload = {"_info": "Erzeugt von devtools/build_geodash_levels.py - "
                        "nicht von Hand bearbeiten.",
               "v": 1, "levels": levels}
    with open(OUT_JSON, "w", encoding="utf-8", newline="\n") as f:
        f.write(_dump_levels(payload))
    with open(OUT_JS, "w", encoding="utf-8", newline="\n") as f:
        f.write("// Automatisch aus games/levels/geodash.json erzeugt "
                "(devtools/build_geodash_levels.py).\n")
        f.write("(function () {\n  \"use strict\";\n  window.PG.gdLevels = ")
        f.write(json.dumps(levels, separators=(",", ":"), ensure_ascii=False))
        f.write(";\n})();\n")
    with open(OUT_PROOFS, "w", encoding="utf-8", newline="\n") as f:
        json.dump(proofs, f, indent=1)
        f.write("\n")
    print("geschrieben:", OUT_JSON, OUT_JS, OUT_PROOFS)
    sys.exit(0 if ok_all else 1)


def _dump_levels(payload):
    """JSON mit einem Objekt je Zeile - kompakt, aber noch lesbar/diffbar."""
    lines = ["{", '  "_info": %s,' % json.dumps(payload["_info"], ensure_ascii=False),
             '  "v": %d,' % payload["v"], '  "levels": [']
    for li, d in enumerate(payload["levels"]):
        head = {k: v for k, v in d.items() if k != "objects"}
        lines.append("    {")
        for k, v in head.items():
            lines.append("      %s: %s," % (json.dumps(k), json.dumps(v, ensure_ascii=False)))
        lines.append('      "objects": [')
        objs = d["objects"]
        for oi, o in enumerate(objs):
            lines.append("        %s%s" % (json.dumps(o, separators=(",", ":")),
                                           "," if oi < len(objs) - 1 else ""))
        lines.append("      ]")
        lines.append("    }" + ("," if li < len(payload["levels"]) - 1 else ""))
    lines.append("  ]")
    lines.append("}")
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    main()
