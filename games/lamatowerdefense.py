# -*- coding: utf-8 -*-
"""
lamatowerdefense.py
===================
Tower Defense - Wellen abwehren, Türme bauen und ausbauen (Einzelspieler).

Features
--------
- 4 KARTEN (Wiese, Schlucht, Kreuzung, Spießrutenlauf) mit eigenem Pfad,
  Schwierigkeitsfaktor und Freischaltung über die beste erreichte Welle.
- ENDLOS-WELLEN: jede Karte läuft unbegrenzt, Gegner-Budget und Lebenspunkte
  wachsen pro Welle über Formeln (kein Wellen-Skript) - wie weit kommst du?
- 3 MODI: Klassisch (7 Turmtypen, Hauptmodus), Kompakt (4 Türme, 2 Stufen),
  Maximal (11 Türme, A/B-Spezialisierung, Spezialgegner, Aktiv-Fähigkeiten).
- 11 TURMTYPEN (Pfeil, Kanone, Frost, Scharfschütze, Gift, Tesla, Fahne,
  Mörser, Flak, Laser, Bank) mit 3 Ausbaustufen, Verkauf (70% Erstattung)
  und im Maximal-Modus einer A/B-Verzweigung auf höchster Stufe.
- 11 GEGNERTYPEN inkl. Boss alle 8 Wellen: gepanzert, regenerierend,
  teilend, fliegend (nur Luftabwehr trifft), getarnt (nur bei Spähern
  sichtbar) und heilend.
- ÖKONOMIE: Gold pro Abschuss, Wellen-Bonus, 5% Zinsen in der Bauphase.
- Aktiv-Fähigkeiten im Maximal-Modus: Meteor [Q], Frostnova [W], Goldsegen [E].

Steuerung: Turmkarte anklicken und im Feld platzieren, Rechtsklick bricht ab.
Turm anklicken = Info/Ausbau/Verkauf. Leertaste = Welle starten,
F = Tempo x2, G = Reichweiten zeigen, 1-9 = Turm-Schnellwahl.
"""

import math
import random

import pygame

import store
import ui
from game_base import Game, InputEvent, LocalizedName
from i18n import t

MAPSEL, BUILD, WAVE, GAMEOVER = "mapsel", "build", "wave", "gameover"

GRID_W, GRID_H = 18, 12          # Spielfeld-Raster in Zellen

# Obergrenzen (Performance-Schutz; Spawner/Effekte drosseln sich daran)
MAX_ENEMIES = 90
MAX_SHOTS = 120
MAX_FX = 150
UNITS_PER_WAVE = 140

# Identitätsfarben des Spielfelds (bewusst NICHT aus dem UI-Theme)
COL_GRASS1 = (52, 74, 48)
COL_GRASS2 = (47, 68, 44)
COL_PATH = (122, 100, 70)
COL_PATH_EDGE = (86, 70, 50)
COL_BUILD_OK = (110, 220, 130)
COL_BUILD_BAD = (235, 100, 90)
COL_HP_BG = (40, 30, 30)
COL_HP = (110, 220, 110)
COL_AIR_SHADOW = (25, 30, 22)

# ---------------------------------------------------------------------------
#  Daten-Tabellen. "modes": c = Kompakt, C = Klassisch, M = Maximal.
# ---------------------------------------------------------------------------

MODE_RULES = {
    "compact": dict(letter="c", levels=2, gold=260, lives=20, boss=8,
                    branch=False, abil=False),
    "classic": dict(letter="C", levels=3, gold=220, lives=20, boss=8,
                    branch=False, abil=False),
    "maximal": dict(letter="M", levels=3, gold=220, lives=20, boss=8,
                    branch=True, abil=True),
}

# Türme: cost/dmg/rng (Zellen)/cd (s). Extras:
#   splash (Zellen), slow+slow_t, dot+dot_t (Dauerschaden), chain (Kettenblitz),
#   buff (Aura +Schaden), income (Gold je Welle), lob (Mörser-Bogen),
#   beam (Dauerstrahl), hitscan (Sofort-Treffer), air (trifft Flieger),
#   air_only, detect (enttarnt Getarnte), min_rng, prio ("strong" = meiste HP)
TOWERS = {
    "arrow":   dict(cost=50, dmg=8, rng=2.6, cd=0.55, proj=9.0, modes="cCM",
                    col=(240, 205, 100)),
    "cannon":  dict(cost=90, dmg=18, rng=2.2, cd=1.40, proj=7.0, modes="cCM",
                    col=(205, 125, 85), splash=0.9),
    "frost":   dict(cost=70, dmg=2, rng=2.0, cd=0.80, proj=8.0, modes="cCM",
                    col=(125, 200, 250), slow=0.45, slow_t=1.6, detect=True),
    "sniper":  dict(cost=120, dmg=45, rng=5.5, cd=2.60, modes="cCM",
                    col=(180, 230, 140), hitscan=True, air=True, detect=True,
                    prio="strong", pierce=True),
    "poison":  dict(cost=85, dmg=4, rng=2.2, cd=0.90, proj=8.0, modes="CM",
                    col=(150, 220, 90), dot=12, dot_t=3.0),
    "tesla":   dict(cost=130, dmg=14, rng=2.0, cd=1.10, modes="CM",
                    col=(150, 175, 255), chain=3, air=True),
    "support": dict(cost=100, dmg=0, rng=1.8, cd=0, modes="CM",
                    col=(255, 170, 220), buff=0.20, detect=True),
    "mortar":  dict(cost=140, dmg=30, rng=8.5, cd=3.00, modes="M",
                    col=(190, 190, 170), splash=1.2, min_rng=2.2, lob=True),
    "flak":    dict(cost=95, dmg=22, rng=3.0, cd=0.90, proj=10.0, modes="M",
                    col=(255, 140, 120), air=True, air_only=True),
    "laser":   dict(cost=160, dmg=26, rng=2.8, cd=0, modes="M",
                    col=(255, 95, 95), beam=True, air=True),
    "bank":    dict(cost=110, dmg=0, rng=0, cd=0, modes="M",
                    col=(250, 215, 110), income=12),
}

# A/B-Spezialisierung (nur Maximal, auf höchster Stufe). Wirkung siehe
# _tower_stats() bzw. Kampf-Code; Namen über td.br.<turm>.a / .b.
BRANCH_COST = 1.6      # x Basiskosten

# Gegner: hp/spd (Zellen je s)/bounty (Gold)/score. Extras:
#   armor (flacher Schadensabzug), regen (HP je s), splits (Teilt sich),
#   fly (Luftpfad), stealth (getarnt), heal (heilt Nachbarn), group
#   (spawnt im Pulk), boss + leak (Lebensverlust beim Durchbruch),
#   intro = Welle, ab der der Typ auftaucht. r = Radius in Zellen.
ENEMIES = {
    "runt":     dict(hp=22, spd=1.9, bounty=4, score=10, r=0.26, modes="cCM",
                     intro=1, col=(235, 120, 90)),
    "fast":     dict(hp=14, spd=3.2, bounty=5, score=12, r=0.22, modes="cCM",
                     intro=2, col=(255, 180, 80)),
    "swarm":    dict(hp=8, spd=2.4, bounty=2, score=6, r=0.17, modes="cCM",
                     intro=4, group=6, col=(250, 220, 110)),
    "tank":     dict(hp=95, spd=1.05, bounty=9, score=18, r=0.34, modes="cCM",
                     intro=5, col=(185, 100, 145)),
    "shield":   dict(hp=60, spd=1.5, bounty=8, score=16, r=0.29, modes="CM",
                     intro=6, armor=6, col=(150, 160, 220)),
    "flyer":    dict(hp=40, spd=2.2, bounty=8, score=16, r=0.25, modes="M",
                     intro=7, fly=True, col=(160, 205, 255)),
    "regen":    dict(hp=70, spd=1.4, bounty=8, score=16, r=0.29, modes="CM",
                     intro=9, regen=4, col=(130, 215, 140)),
    "stealth":  dict(hp=55, spd=1.7, bounty=9, score=18, r=0.27, modes="M",
                     intro=10, stealth=True, col=(175, 175, 195)),
    "splitter": dict(hp=48, spd=1.6, bounty=6, score=14, r=0.29, modes="CM",
                     intro=11, splits=2, col=(230, 150, 200)),
    "healer":   dict(hp=65, spd=1.3, bounty=10, score=18, r=0.29, modes="M",
                     intro=12, heal=6, col=(120, 230, 200)),
    "boss":     dict(hp=900, spd=0.85, bounty=60, score=100, r=0.5,
                     modes="cCM", intro=0, boss=True, leak=5,
                     col=(255, 90, 110)),
}

# Karten: Wegpunkte in Zell-Koordinaten (außerhalb 0..17/0..11 = Feldrand),
# diff = Gegner-HP-Faktor (bewusst NICHT das Budget: mehr Gegner brächten
# mehr Gold und machten lange Karten leichter), unlock = beste Welle
# (kartenübergreifend), ab der die Karte spielbar ist.
# builds="ring" = nur direkt neben dem Pfad baubar.
MAPS = [
    dict(id="meadow", diff=1.00, unlock=0,
         path=[(-1, 3), (13, 3), (13, 7), (4, 7), (4, 10), (18, 10)]),
    dict(id="canyon", diff=1.70, unlock=5,
         path=[(-1, 1), (16, 1), (16, 4), (1, 4), (1, 7), (16, 7),
               (16, 10), (-1, 10)]),
    dict(id="crossing", diff=1.85, unlock=10,
         path=[(-1, 6), (8, 6), (8, 2), (14, 2), (14, 9), (8, 9),
               (8, 6), (18, 6)]),
    dict(id="gauntlet", diff=1.20, unlock=15, builds="ring",
         path=[(-1, 5), (8, 5), (8, 8), (18, 8)]),
]


# ----- Endlos-Skalierung ---------------------------------------------------

def _budget(n):
    """Gegner-Budget der Welle n (Punkte, die der Komponist ausgeben darf)."""
    return 45 + 12 * n + 1.6 * n * n


def _hp_mult(n):
    """HP-Faktor der Welle n; ab Welle 25 zusätzlich exponentiell."""
    m = 1.0 + 0.14 * (n - 1) + 0.012 * (n - 1) ** 2
    if n > 25:
        m *= 1.06 ** (n - 25)
    return m


def _bounty_mult(n):
    """Gold-Inflationsbremse: Beute sinkt langsam mit der Wellen-Nummer."""
    return max(0.35, 1.0 - 0.018 * (n - 1))


def _map_hpf(n, diff):
    """Karten-HP-Faktor mit Anlaufkurve: greift erst ab Welle 10 voll."""
    return 1.0 + (diff - 1.0) * min(1.0, n / 10.0)


def _unit_cost(d):
    """Budget-Kosten eines Gegnertyps (aus Basiswerten abgeleitet)."""
    return d["hp"] / 10.0 + d["spd"] * 4.0


class LamaTowerDefenseGame(Game):
    name = LocalizedName("Tower Defense")
    highscore_key = "lamatowerdef"
    supports_multiplayer = False
    wants_right_click = True
    MODES = [
        ("classic", "td.mode.classic"),
        ("compact", "td.mode.compact"),
        ("maximal", "td.mode.maximal"),
    ]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        self.anim_t = 0.0

        self.rules = MODE_RULES.get(self.mode, MODE_RULES["classic"])
        self.mletter = self.rules["letter"]
        self.bar_keys = [k for k, d in TOWERS.items()
                         if self.mletter in d["modes"]]

        saved = store.load_section("lamatowerdef")
        self.best = {k: int(v) for k, v in saved.get("best", {}).items()
                     if isinstance(v, (int, float))}

        # Leere Lauf-Variablen, damit draw() im Kartenwahl-Zustand läuft.
        self.map_idx = 0
        self.towers = {}
        self.enemies = []
        self.shots = []
        self.fx = []
        self.spawn_queue = []
        self.wave = 1
        self.gold = 0
        self.lives = 0
        self.kills = 0
        self.ff = 1
        self.show_ranges = False
        self.sel_card = None       # Turm-Schlüssel im Baumenü (Platzieren)
        self.sel_tower = None      # ausgewählter platzierter Turm (Zelle)
        self.armed = None          # gewappnete Fähigkeit ("meteor")
        self.abil_cd = {"meteor": 0.0, "nova": 0.0, "gold": 0.0}
        self.mapsel_idx = 0
        self.mouse = None
        self.bar_scroll = 0
        self._over_t = 0.0

        self._bg_cache = None
        self._dim_cache = None
        self._layout()
        self.state = MAPSEL

    def _make_fonts(self):
        h = self.height
        self.font = ui.font(max(18, min(26, h // 26)))
        self.big_font = ui.font(max(30, min(50, h // 13)), bold=True)
        self._small = ui.font(max(13, min(19, h // 34)))
        self._tiny = ui.font(max(11, min(15, h // 44)))

    def _layout(self):
        """Alle Masse aus width/height ableiten (auch nach Resize)."""
        self._make_fonts()
        self.hud_h = max(34, self.height // 13)
        self.bar_w = max(148, int(self.width * 0.20))
        avail_w = self.width - self.bar_w
        avail_h = self.height - self.hud_h
        self.cell = max(10, min(avail_w // GRID_W, avail_h // GRID_H))
        self.ox = (avail_w - self.cell * GRID_W) // 2
        self.oy = self.hud_h + (avail_h - self.cell * GRID_H) // 2
        self.field_rect = pygame.Rect(self.ox, self.oy,
                                      self.cell * GRID_W, self.cell * GRID_H)
        self.bar_rect = pygame.Rect(self.width - self.bar_w, 0,
                                    self.bar_w, self.height)
        self._build_bar_layout()
        self._build_mapsel_layout()
        self._setup_path(MAPS[self.map_idx])
        if self.mouse is None:
            self.mouse = self.field_rect.center

    def _build_bar_layout(self):
        """Rechteck-Raster der Seitenleiste (Karten, Buttons)."""
        bx = self.bar_rect.x + 10
        bw = self.bar_w - 20
        y = 12
        self.abil_rects = {}
        if self.rules["abil"]:
            aw = (bw - 12) // 3
            for i, key in enumerate(("meteor", "nova", "gold")):
                self.abil_rects[key] = pygame.Rect(
                    bx + i * (aw + 6), y, aw, max(30, self.cell))
            y += max(30, self.cell) + 10
        # Turmkarten: 2 Spalten
        cw = (bw - 8) // 2
        ch = max(44, int(self.cell * 1.35))
        self.card_rects = []
        for i, key in enumerate(self.bar_keys):
            cx = bx + (i % 2) * (cw + 8)
            cy = y + (i // 2) * (ch + 8)
            self.card_rects.append(pygame.Rect(cx, cy, cw, ch))
        self.cards_top = y
        rows = (len(self.bar_keys) + 1) // 2
        self.cards_h = rows * (ch + 8)
        # Info-Bereich + Start-Button unten
        bh = max(40, int(self.height * 0.07))
        self.start_rect = pygame.Rect(bx, self.height - bh - 12, bw, bh)
        ih = max(120, int(self.height * 0.30))
        self.info_rect = pygame.Rect(bx, self.start_rect.y - ih - 10, bw, ih)
        self.cards_view_h = max(40, self.info_rect.y - y - 8)
        # Buttons im Info-Bereich (Ausbau/Verkauf/Zweige)
        byy = self.info_rect.bottom - 34
        half = (bw - 6) // 2
        self.up_rect = pygame.Rect(bx, byy - 38, bw, 32)
        self.sell_rect = pygame.Rect(bx, byy, bw, 30)
        self.br_a_rect = pygame.Rect(bx, byy - 38, half, 32)
        self.br_b_rect = pygame.Rect(bx + half + 6, byy - 38, half, 32)

    def _build_mapsel_layout(self):
        cw = min(300, (self.width - 80) // 2)
        ch = min(190, (self.height - 170) // 2)
        cx = self.width // 2
        y0 = max(120, int(self.height * 0.24))
        self.map_rects = []
        for i in range(len(MAPS)):
            x = cx - cw - 14 + (i % 2) * (cw + 28)
            y = y0 + (i // 2) * (ch + 22)
            self.map_rects.append(pygame.Rect(x, y, cw, ch))

    def on_surface_changed(self):
        self._layout()
        self._bg_cache = None
        self._dim_cache = None

    # ----- Pfad-Geometrie ------------------------------------------------
    def _setup_path(self, m):
        """Wegpunkte der Karte in Pfad-Polylinie + Zellmengen übersetzen."""
        wps = m["path"]
        self.path_pts = [(c + 0.5, r + 0.5) for c, r in wps]
        self.path_cum = [0.0]
        total = 0.0
        for (x0, y0), (x1, y1) in zip(self.path_pts, self.path_pts[1:]):
            total += abs(x1 - x0) + abs(y1 - y0)
            self.path_cum.append(total)
        self.path_total = total
        # Luftpfad: gerade Linie Start -> Ziel (für Flieger)
        self.fly_a = self.path_pts[0]
        self.fly_b = self.path_pts[-1]
        self.fly_total = math.hypot(self.fly_b[0] - self.fly_a[0],
                                    self.fly_b[1] - self.fly_a[1])
        # Pfad-Zellen (gesperrt fürs Bauen)
        cells = set()
        for (c0, r0), (c1, r1) in zip(wps, wps[1:]):
            if c0 == c1:
                for r in range(min(r0, r1), max(r0, r1) + 1):
                    cells.add((c0, r))
            else:
                for c in range(min(c0, c1), max(c0, c1) + 1):
                    cells.add((c, r0))
        self.path_cells = {(c, r) for c, r in cells
                           if 0 <= c < GRID_W and 0 <= r < GRID_H}
        # Baubare Zellen ("ring" = nur direkte Pfad-Nachbarn)
        if m.get("builds") == "ring":
            ring = set()
            for c, r in self.path_cells:
                for dc in (-1, 0, 1):
                    for dr in (-1, 0, 1):
                        ring.add((c + dc, r + dr))
            self.build_cells = {(c, r) for c, r in ring - self.path_cells
                                if 0 <= c < GRID_W and 0 <= r < GRID_H}
        else:
            self.build_cells = None    # None = überall außer Pfad

    def _pos_at(self, d, fly=False):
        """Position (Zell-Koordinaten) bei Pfad-Distanz d."""
        if fly:
            f = max(0.0, min(1.0, d / max(0.001, self.fly_total)))
            return (self.fly_a[0] + (self.fly_b[0] - self.fly_a[0]) * f,
                    self.fly_a[1] + (self.fly_b[1] - self.fly_a[1]) * f)
        pts, cum = self.path_pts, self.path_cum
        if d <= 0:
            return pts[0]
        if d >= cum[-1]:
            return pts[-1]
        for i in range(1, len(cum)):
            if d <= cum[i]:
                f = (d - cum[i - 1]) / max(0.001, cum[i] - cum[i - 1])
                x0, y0 = pts[i - 1]
                x1, y1 = pts[i]
                return (x0 + (x1 - x0) * f, y0 + (y1 - y0) * f)
        return pts[-1]

    def _px(self, cx, cy):
        """Zell-Koordinaten -> Pixel."""
        return (self.ox + cx * self.cell, self.oy + cy * self.cell)

    # ===================================================== Lauf-Verwaltung
    def _unlocked(self, m):
        top = max(self.best.values()) if self.best else 0
        return m["unlock"] <= top

    def _start_run(self, idx):
        self.map_idx = idx
        self._setup_path(MAPS[idx])
        self._bg_cache = None
        self.towers = {}
        self.enemies = []
        self.shots = []
        self.fx = []
        self.spawn_queue = []
        self.score = 0
        self.kills = 0
        self.wave = 1
        self.gold = self.rules["gold"]
        self.lives = self.rules["lives"]
        self.ff = 1
        self.show_ranges = False
        self.sel_card = None
        self.sel_tower = None
        self.armed = None
        self.abil_cd = {"meteor": 0.0, "nova": 0.0, "gold": 0.0}
        self.bar_scroll = 0
        self.game_over = False
        self.state = BUILD
        self.play_sound("select")

    def _save_best(self):
        cleared = self.wave - 1
        mid = MAPS[self.map_idx]["id"]
        if cleared > self.best.get(mid, 0):
            self.best[mid] = cleared
            store.save_section("lamatowerdef", {"best": self.best})

    # ===================================================== Wellen-Komponist
    def _compose_wave(self, n):
        """Spawn-Liste [(Gegnertyp, Abstand_s), ...] für Welle n bauen."""
        budget = _budget(n)
        gap = max(0.28, 0.9 - 0.012 * n)
        queue = []

        avail = [k for k, d in ENEMIES.items()
                 if self.mletter in d["modes"] and not d.get("boss")
                 and d["intro"] <= n]
        boss_wave = (n % self.rules["boss"] == 0)

        if boss_wave:
            for _ in range(1 + n // 32):
                queue.append(("boss", 1.2))
            budget *= 0.45      # Rest als Eskorte
        elif n % 5 == 0 and n > 4:
            # Themen-Welle: nur ein Typ (rotiert durch die Freischaltungen)
            avail = [avail[(n // 5) % len(avail)]]

        spent = 0.0
        units = len(queue)
        while spent < budget and units < UNITS_PER_WAVE:
            # neuere Typen leicht bevorzugen
            k = random.choice(avail[-3:]) if random.random() < 0.45 \
                else random.choice(avail)
            d = ENEMIES[k]
            group = d.get("group", 1)
            for i in range(group):
                queue.append((k, 0.12 if i else gap))
            spent += _unit_cost(d) * group
            units += group

        # Restbudget (Einheiten-Deckel erreicht) fließt in Bonus-HP.
        hp_bonus = 1.0 + max(0.0, (budget - spent) / max(1.0, budget))
        return queue, hp_bonus

    def _start_wave(self):
        if self.state != BUILD:
            return
        self.spawn_queue, self._hp_bonus = self._compose_wave(self.wave)
        self.spawn_t = 0.5
        self.state = WAVE
        self.play_sound("select")

    def _spawn(self, kind):
        if len(self.enemies) >= MAX_ENEMIES:
            self.spawn_t += 0.4     # warten, bis wieder Platz ist
            self.spawn_queue.insert(0, (kind, 0.0))
            return
        d = ENEMIES[kind]
        hp = (d["hp"] * _hp_mult(self.wave)
              * _map_hpf(self.wave, MAPS[self.map_idx]["diff"])
              * getattr(self, "_hp_bonus", 1.0))
        self.enemies.append(dict(
            kind=kind, hp=hp, maxhp=hp, d=0.0, spd=d["spd"], r=d["r"],
            armor=d.get("armor", 0), regen=d.get("regen", 0),
            fly=d.get("fly", False), stealth=d.get("stealth", False),
            heal=d.get("heal", 0), splits=d.get("splits", 0),
            boss=d.get("boss", False), leak=d.get("leak", 1),
            bounty=d["bounty"], score=d["score"], col=d["col"],
            slow_t=0.0, slow_f=0.0, stun_t=0.0, dot_t=0.0, dot=0.0,
            seen=not d.get("stealth", False)))

    # ===================================================== Turm-Verwaltung
    def _tower_stats(self, tw):
        """Wirkwerte eines Turms aus Basis, Stufe, Zweig und Auren."""
        base = TOWERS[tw["kind"]]
        lvl = tw["level"]
        br = tw.get("branch")
        st = dict(dmg=base.get("dmg", 0) * (1.55 ** (lvl - 1)),
                  rng=base.get("rng", 0) * (1.12 ** (lvl - 1)),
                  cd=base.get("cd", 0) * (0.88 ** (lvl - 1)),
                  splash=base.get("splash", 0),
                  slow=base.get("slow", 0), slow_t=base.get("slow_t", 0),
                  dot=base.get("dot", 0) * (1.55 ** (lvl - 1)),
                  dot_t=base.get("dot_t", 0),
                  chain=base.get("chain", 0),
                  buff=base.get("buff", 0) + 0.05 * (lvl - 1),
                  income=base.get("income", 0) * (1.5 ** (lvl - 1)),
                  air=base.get("air", False),
                  air_only=base.get("air_only", False),
                  targets=1, execute=0.0, stun=0.0, aura_dps=0.0,
                  ramp_max=2.5, ramp_rate=0.35, splits_dot=False,
                  acid=False, ground_frac=0.0, cluster=1,
                  buff_cd=False)
        k = tw["kind"]
        if br == "a":
            if k == "arrow":
                st["targets"] = 2
                st["air"] = True
            elif k == "cannon":
                st["splash"] *= 1.6
                st["stun"] = 0.3
            elif k == "frost":
                st["stun"] = 0.35
            elif k == "sniper":
                st["execute"] = 0.20
            elif k == "poison":
                st["splits_dot"] = True
            elif k == "tesla":
                st["chain"] = 5
            elif k == "support":
                st["buff"] += 0.15
            elif k == "mortar":
                st["cluster"] = 3
            elif k == "flak":
                st["splash"] = 0.9
            elif k == "laser":
                st["targets"] = 3
            elif k == "bank":
                st["income"] *= 1.8
        elif br == "b":
            if k == "arrow":
                st["dmg"] *= 1.85
            elif k == "cannon":
                st["dot"], st["dot_t"] = 10, 2.5
            elif k == "frost":
                st["aura_dps"] = 10
            elif k == "sniper":
                st["cd"] *= 0.55
            elif k == "poison":
                st["acid"] = True
            elif k == "tesla":
                st["dmg"] *= 1.8
            elif k == "support":
                st["buff_cd"] = True
            elif k == "mortar":
                st["dmg"] *= 2.0
            elif k == "flak":
                st["ground_frac"] = 0.6
                st["air_only"] = False
            elif k == "laser":
                st["ramp_max"], st["ramp_rate"] = 3.5, 0.7
            elif k == "bank":
                st["income"] *= 1.2
        # Unterstützungs-Auren einrechnen
        st["dmg"] *= tw.get("buff_dmg", 1.0)
        st["cd"] *= tw.get("buff_cd_f", 1.0)
        return st

    def _refresh_towers(self):
        """Auren neu verteilen und Wirkwerte aller Türme neu berechnen."""
        sups = []
        for tw in self.towers.values():
            if tw["kind"] == "support":
                tw["buff_dmg"] = 1.0
                tw["buff_cd_f"] = 1.0
                tw["st"] = self._tower_stats(tw)
                sups.append(tw)
        for tw in self.towers.values():
            if tw["kind"] == "support":
                continue
            dmg_f, cd_f = 1.0, 1.0
            cx, cy = tw["cell"][0] + 0.5, tw["cell"][1] + 0.5
            for sp in sups:
                sx, sy = sp["cell"][0] + 0.5, sp["cell"][1] + 0.5
                if (cx - sx) ** 2 + (cy - sy) ** 2 <= sp["st"]["rng"] ** 2:
                    dmg_f += sp["st"]["buff"]
                    if sp["st"]["buff_cd"]:
                        cd_f = min(cd_f, 0.85)
            tw["buff_dmg"] = min(1.6, dmg_f)
            tw["buff_cd_f"] = cd_f
            tw["st"] = self._tower_stats(tw)

    def _can_build(self, cell):
        c, r = cell
        if not (0 <= c < GRID_W and 0 <= r < GRID_H):
            return False
        if cell in self.path_cells or cell in self.towers:
            return False
        if self.build_cells is not None and cell not in self.build_cells:
            return False
        return True

    def _place_tower(self, cell):
        base = TOWERS[self.sel_card]
        if self.gold < base["cost"]:
            self.play_sound("hit")
            return
        self.gold -= base["cost"]
        self.towers[cell] = dict(kind=self.sel_card, cell=cell, level=1,
                                 branch=None, invested=base["cost"],
                                 cd_left=0.0, angle=0.0, ramp=1.0,
                                 beam=None, buff_dmg=1.0, buff_cd_f=1.0)
        self._refresh_towers()
        self.play_sound("lock")
        if self.gold < base["cost"]:
            self.sel_card = None     # kann keinen weiteren bezahlen

    def _upgrade_cost(self, tw):
        return int(TOWERS[tw["kind"]]["cost"] * 0.8 * tw["level"])

    def _branch_cost(self, tw):
        return int(TOWERS[tw["kind"]]["cost"] * BRANCH_COST)

    def _sell_value(self, tw):
        return int(tw["invested"] * 0.7)

    def _upgrade_tower(self, tw):
        if tw["level"] >= self.rules["levels"] or tw.get("branch"):
            return
        cost = self._upgrade_cost(tw)
        if self.gold < cost:
            self.play_sound("hit")
            return
        self.gold -= cost
        tw["level"] += 1
        tw["invested"] += cost
        self._refresh_towers()
        self.play_sound("powerup")
        if tw["level"] >= self.rules["levels"] and not self.rules["branch"]:
            self.ach_event("td_maxed")

    def _branch_tower(self, tw, which):
        if (not self.rules["branch"] or tw.get("branch")
                or tw["level"] < self.rules["levels"]):
            return
        cost = self._branch_cost(tw)
        if self.gold < cost:
            self.play_sound("hit")
            return
        self.gold -= cost
        tw["branch"] = which
        tw["invested"] += cost
        if tw["kind"] == "bank" and which == "b":
            self.gold += 200     # Dividende: Sofort-Auszahlung
        self._refresh_towers()
        self.play_sound("level")
        self.ach_event("td_maxed")

    def _sell_tower(self, cell):
        tw = self.towers.pop(cell, None)
        if tw is None:
            return
        self.gold += self._sell_value(tw)
        self.sel_tower = None
        self._refresh_towers()
        self.play_sound("eat")

    # ===================================================== Simulation
    def update(self, dt):
        self.anim_t += dt
        if self.state in (MAPSEL, GAMEOVER):
            self._age_fx(dt)
            return
        for _ in range(self.ff):
            rest = dt
            while rest > 1e-9 and not self.game_over:
                h = min(rest, 0.04)
                self._sim(h)
                rest -= h
        self._age_fx(dt)

    def _sim(self, h):
        # Fähigkeiten-Abklingzeiten
        for k in self.abil_cd:
            self.abil_cd[k] = max(0.0, self.abil_cd[k] - h)

        if self.state == WAVE and self.spawn_queue:
            self.spawn_t -= h
            while self.spawn_queue and self.spawn_t <= 0:
                kind, gap = self.spawn_queue.pop(0)
                self._spawn(kind)
                self.spawn_t += gap

        self._sim_enemies(h)
        self._sim_towers(h)
        self._sim_shots(h)

        if (self.state == WAVE and not self.spawn_queue
                and not self.enemies):
            self._wave_cleared()

    def _sim_enemies(self, h):
        detect = [(tw["cell"][0] + 0.5, tw["cell"][1] + 0.5, tw["st"]["rng"])
                  for tw in self.towers.values()
                  if TOWERS[tw["kind"]].get("detect")]
        aura_frost = [(tw["cell"][0] + 0.5, tw["cell"][1] + 0.5,
                       tw["st"]["rng"], tw["st"]["aura_dps"])
                      for tw in self.towers.values()
                      if tw["st"].get("aura_dps", 0) > 0]
        healers = [e for e in self.enemies if e["heal"] and e["hp"] > 0]

        leaked = False
        for e in self.enemies:
            if e["hp"] <= 0:
                continue
            # Status-Effekte
            if e["dot_t"] > 0:
                e["dot_t"] -= h
                e["hp"] -= e["dot"] * h
                if e["hp"] <= 0:
                    self._kill(e)
                    continue
            if e["regen"]:
                e["hp"] = min(e["maxhp"], e["hp"] + e["regen"] * h)
            ex, ey = self._pos_at(e["d"], e["fly"])
            for ax, ay, rng, dps in aura_frost:
                if (ex - ax) ** 2 + (ey - ay) ** 2 <= rng * rng:
                    e["hp"] -= dps * h
            if e["hp"] <= 0:
                self._kill(e)
                continue
            if e["stealth"]:
                e["seen"] = any((ex - ax) ** 2 + (ey - ay) ** 2 <= rng * rng
                                for ax, ay, rng in detect)
            # Bewegung
            v = e["spd"]
            if e["slow_t"] > 0:
                e["slow_t"] -= h
                v *= (1.0 - e["slow_f"])
            if e["stun_t"] > 0:
                e["stun_t"] -= h
                v = 0.0
            e["d"] += v * h
            total = self.fly_total if e["fly"] else self.path_total
            if e["d"] >= total:
                e["hp"] = 0
                self.lives -= e["leak"]
                leaked = True

        # Heiler-Auren (nach der Bewegung, auf lebende Nachbarn)
        for he in healers:
            if he["hp"] <= 0:
                continue
            hx, hy = self._pos_at(he["d"], he["fly"])
            for e in self.enemies:
                if e is he or e["hp"] <= 0:
                    continue
                ex, ey = self._pos_at(e["d"], e["fly"])
                if (ex - hx) ** 2 + (ey - hy) ** 2 <= 1.5 ** 2:
                    e["hp"] = min(e["maxhp"], e["hp"] + he["heal"] * h)

        self.enemies = [e for e in self.enemies if e["hp"] > 0]
        if leaked:
            self.play_sound("hit")
            self.rumble(150)
            if self.lives <= 0:
                self._finish()

    def _visible(self, e, st):
        """Darf ein Turm mit Werten st den Gegner e anvisieren?"""
        if e["hp"] <= 0 or not e["seen"]:
            return False
        if e["fly"]:
            return st["air"]
        return not st["air_only"] or st["ground_frac"] > 0

    def _find_target(self, tw, st):
        cx, cy = tw["cell"][0] + 0.5, tw["cell"][1] + 0.5
        rng2 = st["rng"] ** 2
        min2 = TOWERS[tw["kind"]].get("min_rng", 0) ** 2
        best, best_v = None, -1.0
        strong = TOWERS[tw["kind"]].get("prio") == "strong"
        for e in self.enemies:
            if not self._visible(e, st):
                continue
            ex, ey = self._pos_at(e["d"], e["fly"])
            d2 = (ex - cx) ** 2 + (ey - cy) ** 2
            if d2 > rng2 or d2 < min2:
                continue
            v = e["hp"] if strong else e["d"] + (1000 if e["boss"] else 0)
            if v > best_v:
                best, best_v = e, v
        return best

    def _sim_towers(self, h):
        for tw in self.towers.values():
            st = tw["st"]
            base = TOWERS[tw["kind"]]
            if base.get("income") or tw["kind"] == "support":
                continue
            cx, cy = tw["cell"][0] + 0.5, tw["cell"][1] + 0.5
            if base.get("beam"):
                self._sim_laser(tw, st, h)
                continue
            tw["cd_left"] -= h
            if tw["cd_left"] > 0:
                continue
            targets = []
            first = self._find_target(tw, st)
            if first is None:
                continue
            targets.append(first)
            if st["targets"] > 1:      # Doppelschuss: zweites Ziel suchen
                for e in self.enemies:
                    if e is first or not self._visible(e, st):
                        continue
                    ex, ey = self._pos_at(e["d"], e["fly"])
                    if (ex - cx) ** 2 + (ey - cy) ** 2 <= st["rng"] ** 2:
                        targets.append(e)
                        if len(targets) >= st["targets"]:
                            break
            for e in targets:
                self._fire(tw, st, e)
            ex, ey = self._pos_at(first["d"], first["fly"])
            tw["angle"] = math.atan2(ey - cy, ex - cx)
            tw["cd_left"] = st["cd"]

    def _sim_laser(self, tw, st, h):
        e = tw.get("beam")
        cx, cy = tw["cell"][0] + 0.5, tw["cell"][1] + 0.5
        if e is not None:
            ex, ey = self._pos_at(e["d"], e["fly"])
            if (e["hp"] <= 0 or not self._visible(e, st)
                    or (ex - cx) ** 2 + (ey - cy) ** 2 > st["rng"] ** 2):
                e = None
                tw["ramp"] = 1.0
        if e is None:
            e = self._find_target(tw, st)
            tw["ramp"] = 1.0
        tw["beam"] = e
        if e is None:
            return
        tw["ramp"] = min(st["ramp_max"], tw["ramp"] + st["ramp_rate"] * h)
        victims = [e]
        if st["targets"] > 1:         # Prisma: Strahl teilt sich
            for o in self.enemies:
                if o is e or not self._visible(o, st):
                    continue
                ox_, oy_ = self._pos_at(o["d"], o["fly"])
                if (ox_ - cx) ** 2 + (oy_ - cy) ** 2 <= st["rng"] ** 2:
                    victims.append(o)
                    if len(victims) >= st["targets"]:
                        break
        for i, v in enumerate(victims):
            frac = 1.0 if i == 0 else 0.5
            self._damage(v, st["dmg"] * tw["ramp"] * h * frac)
        ex, ey = self._pos_at(e["d"], e["fly"])
        tw["angle"] = math.atan2(ey - cy, ex - cx)

    def _fire(self, tw, st, e):
        base = TOWERS[tw["kind"]]
        cx, cy = tw["cell"][0] + 0.5, tw["cell"][1] + 0.5
        ex, ey = self._pos_at(e["d"], e["fly"])
        dmg = st["dmg"]
        if e["fly"] and st["ground_frac"] == 0 and base.get("air_only"):
            pass
        elif e["fly"] is False and st["ground_frac"] > 0:
            dmg *= st["ground_frac"]     # Flak-Zielcomputer: Boden schwächer
        if base.get("hitscan"):
            self._add_fx("tracer", a=(cx, cy), b=(ex, ey),
                         col=base["col"], ttl=0.12)
            if st["execute"] and e["hp"] / e["maxhp"] < st["execute"] \
                    and not e["boss"]:
                e["hp"] = 0
                self._kill(e)
            else:
                self._damage(e, dmg, pierce=base.get("pierce", False))
            self.play_sound("shoot")
            return
        if st["chain"]:
            pts = [(cx, cy)]
            victim, cd = e, dmg
            hit = set()
            for _ in range(st["chain"]):
                vx, vy = self._pos_at(victim["d"], victim["fly"])
                pts.append((vx, vy))
                self._damage(victim, cd)
                hit.add(id(victim))
                cd *= 0.65
                nxt = None
                nd = 1.5 ** 2
                for o in self.enemies:
                    if id(o) in hit or not self._visible(o, st):
                        continue
                    ox_, oy_ = self._pos_at(o["d"], o["fly"])
                    d2 = (ox_ - vx) ** 2 + (oy_ - vy) ** 2
                    if d2 <= nd:
                        nxt, nd = o, d2
                if nxt is None:
                    break
                victim = nxt
            self._add_fx("zigzag", pts=pts, col=base["col"], ttl=0.15)
            return
        if base.get("lob"):
            dist = math.hypot(ex - cx, ey - cy)
            fly_t = max(0.35, min(1.2, dist / 6.0))
            aim = self._pos_at(e["d"] + e["spd"] * fly_t * 0.7, e["fly"])
            n = st["cluster"]
            for i in range(n):
                jx = random.uniform(-0.5, 0.5) if i else 0.0
                jy = random.uniform(-0.5, 0.5) if i else 0.0
                self.shots.append(dict(
                    kind="lob", x=cx, y=cy, tx=aim[0] + jx, ty=aim[1] + jy,
                    t=0.0, dur=fly_t, dmg=dmg / (1.6 if n > 1 else 1.0),
                    splash=st["splash"] * (0.75 if n > 1 else 1.0),
                    stun=st["stun"], col=base["col"]))
            self.play_sound("shoot")
            return
        if len(self.shots) < MAX_SHOTS:
            self.shots.append(dict(
                kind="proj", x=cx, y=cy, tgt=e, lx=ex, ly=ey,
                spd=base.get("proj", 9.0), dmg=dmg, splash=st["splash"],
                slow=st["slow"], slow_t=st["slow_t"],
                dot=st["dot"], dot_t=st["dot_t"], stun=st["stun"],
                acid=st["acid"], splits_dot=st["splits_dot"],
                col=base["col"]))

    def _sim_shots(self, h):
        rest = []
        for s in self.shots:
            if s["kind"] == "lob":
                s["t"] += h
                if s["t"] >= s["dur"]:
                    self._explode((s["tx"], s["ty"]), s["dmg"], s["splash"],
                                  stun=s["stun"])
                else:
                    rest.append(s)
                continue
            e = s["tgt"]
            if e is not None and e["hp"] > 0:
                s["lx"], s["ly"] = self._pos_at(e["d"], e["fly"])
            else:
                s["tgt"] = e = None
            dx, dy = s["lx"] - s["x"], s["ly"] - s["y"]
            dist = math.hypot(dx, dy)
            step = s["spd"] * h
            if dist <= max(step, 0.22):
                if e is not None:
                    self._impact(s, e)
                elif s["splash"]:
                    self._explode((s["lx"], s["ly"]), s["dmg"], s["splash"],
                                  stun=s["stun"])
                continue
            s["x"] += dx / dist * step
            s["y"] += dy / dist * step
            rest.append(s)
        self.shots = rest

    def _impact(self, s, e):
        if s["splash"]:
            self._explode((s["lx"], s["ly"]), s["dmg"], s["splash"],
                          stun=s["stun"])
        else:
            if s["acid"]:
                e["armor"] = 0
            if s["slow"]:
                e["slow_f"] = max(e["slow_f"], s["slow"])
                e["slow_t"] = max(e["slow_t"], s["slow_t"])
            if s["stun"]:
                e["stun_t"] = max(e["stun_t"], s["stun"])
            if s["dot"]:
                e["dot"] = max(e["dot"], s["dot"])
                e["dot_t"] = max(e["dot_t"], s["dot_t"])
                e["plague"] = s["splits_dot"]
            self._damage(e, s["dmg"])

    def _explode(self, pos, dmg, radius, stun=0.0):
        px, py = pos
        for e in list(self.enemies):
            if e["hp"] <= 0 or e["fly"]:
                continue
            ex, ey = self._pos_at(e["d"], e["fly"])
            if (ex - px) ** 2 + (ey - py) ** 2 <= radius * radius:
                if stun:
                    e["stun_t"] = max(e["stun_t"], stun)
                self._damage(e, dmg)
        self._add_fx("ring", pos=pos, r=radius, col=(255, 190, 120), ttl=0.25)

    def _damage(self, e, dmg, pierce=False):
        if e["hp"] <= 0:
            return
        if e["armor"] and not pierce:
            dmg = max(1.0, dmg - e["armor"])
        e["hp"] -= dmg
        if e["hp"] <= 0:
            self._kill(e)

    def _kill(self, e):
        if e.get("_dead"):
            return
        e["_dead"] = True
        e["hp"] = 0
        n = self.wave
        self.gold += max(1, round(e["bounty"] * _bounty_mult(n)))
        self.score += e["score"]
        self.kills += 1
        ex, ey = self._pos_at(e["d"], e["fly"])
        self._add_fx("burst", pos=(ex, ey), col=e["col"], ttl=0.4)
        if e["boss"]:
            self.play_sound("explode")
            self.ach_event("td_boss")
            px, py = self._px(ex, ey)
            ui.spawn_burst(px, py, color=e["col"], n=24)
        if e["splits"]:
            for _ in range(e["splits"]):
                if len(self.enemies) < MAX_ENEMIES:
                    d = ENEMIES["runt"]
                    hp = (d["hp"] * _hp_mult(n) * 0.6
                          * _map_hpf(n, MAPS[self.map_idx]["diff"]))
                    child = dict(
                        kind="runt", hp=hp, maxhp=hp,
                        d=e["d"] - random.uniform(0.0, 0.5),
                        spd=d["spd"], r=d["r"], armor=0, regen=0, fly=False,
                        stealth=False, heal=0, splits=0, boss=False, leak=1,
                        bounty=d["bounty"], score=d["score"], col=d["col"],
                        slow_t=0.0, slow_f=0.0, stun_t=0.0, dot_t=0.0,
                        dot=0.0, seen=True)
                    self.enemies.append(child)
        if e.get("plague") and e["dot"]:
            for o in self.enemies:
                if o is e or o["hp"] <= 0:
                    continue
                ox_, oy_ = self._pos_at(o["d"], o["fly"])
                if (ox_ - ex) ** 2 + (oy_ - ey) ** 2 <= 1.2 ** 2:
                    o["dot"] = max(o["dot"], e["dot"])
                    o["dot_t"] = max(o["dot_t"], e["dot_t"])

    def _wave_cleared(self):
        n = self.wave
        diff = MAPS[self.map_idx]["diff"]
        income = sum(tw["st"]["income"] for tw in self.towers.values()
                     if tw["st"].get("income"))
        self.gold += 20 + 4 * n + int(income)
        self.gold += min(50, int(self.gold * 0.05))     # Zinsen
        self.score += int((20 + 5 * n) * diff)
        self.play_sound("point")
        if n >= 20:
            self.ach_event("td_wave20")
        if n == 10 and self.lives == self.rules["lives"]:
            self.ach_event("td_perfect10")
        self.wave = n + 1
        self._save_best()
        self.state = BUILD

    def _finish(self):
        self._save_best()
        self.state = GAMEOVER
        self.game_over = True
        self._over_t = self.anim_t
        self.play_sound("gameover")
        self.rumble(300)

    # ----- Fähigkeiten ---------------------------------------------------
    ABIL_CDS = {"meteor": 45.0, "nova": 60.0, "gold": 90.0}

    def _use_ability(self, key):
        if not self.rules["abil"] or self.abil_cd[key] > 0:
            return
        if key == "meteor":
            self.armed = "meteor" if self.armed != "meteor" else None
            self.play_sound("click")
            return
        self.abil_cd[key] = self.ABIL_CDS[key]
        if key == "nova":
            for e in self.enemies:
                e["slow_f"] = max(e["slow_f"], 0.6)
                e["slow_t"] = max(e["slow_t"], 3.0)
            self._add_fx("nova", pos=(GRID_W / 2, GRID_H / 2), ttl=0.5)
        elif key == "gold":
            self.gold += 80 + 3 * self.wave
        self.play_sound("level")

    def _cast_meteor(self, cellpos):
        self.armed = None
        self.abil_cd["meteor"] = self.ABIL_CDS["meteor"]
        dmg = 150 + 12 * self.wave
        px, py = cellpos
        for e in list(self.enemies):
            if e["hp"] <= 0:
                continue
            ex, ey = self._pos_at(e["d"], e["fly"])
            if (ex - px) ** 2 + (ey - py) ** 2 <= 1.8 ** 2:
                self._damage(e, dmg, pierce=True)
        self._add_fx("ring", pos=cellpos, r=1.8, col=(255, 150, 80), ttl=0.4)
        self.play_sound("explode")

    # ----- Effekte -------------------------------------------------------
    def _add_fx(self, kind, **kw):
        if len(self.fx) < MAX_FX:
            kw["kind"] = kind
            kw["age"] = 0.0
            self.fx.append(kw)

    def _age_fx(self, dt):
        for f in self.fx:
            f["age"] += dt
        self.fx = [f for f in self.fx if f["age"] < f["ttl"]]

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == MAPSEL:
            self._ev_mapsel(event)
        elif self.state == GAMEOVER:
            self._ev_gameover(event)
        else:
            self._ev_play(event)

    def _ev_mapsel(self, event):
        n = len(MAPS)
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Left", "a"):
                self.mapsel_idx = (self.mapsel_idx - 1) % n
                self.play_sound("move")
            elif event.key in ("Right", "d"):
                self.mapsel_idx = (self.mapsel_idx + 1) % n
                self.play_sound("move")
            elif event.key in ("Up", "w", "Down", "s"):
                self.mapsel_idx = (self.mapsel_idx + 2) % n
                self.play_sound("move")
            elif event.key in ("Return", "space"):
                if self._unlocked(MAPS[self.mapsel_idx]):
                    self._start_run(self.mapsel_idx)
                else:
                    self.play_sound("hit")
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            for i, r in enumerate(self.map_rects):
                if r.collidepoint(event.pos):
                    self.mapsel_idx = i
                    if self._unlocked(MAPS[i]):
                        self._start_run(i)
                    else:
                        self.play_sound("hit")
                    return
        elif event.kind == InputEvent.MOUSEMOVE:
            self.mouse = event.pos

    def _ev_gameover(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Return", "space"):
                self._start_run(self.map_idx)
            elif event.key in ("m", "M"):
                self.game_over = False
                self.state = MAPSEL
                self.play_sound("click")
        elif (event.kind == InputEvent.MOUSEDOWN
                and self.anim_t - self._over_t > 0.5):
            self._start_run(self.map_idx)

    def _cell_at(self, pos):
        if not self.field_rect.collidepoint(pos):
            return None
        return ((pos[0] - self.ox) // self.cell,
                (pos[1] - self.oy) // self.cell)

    def _cellpos_at(self, pos):
        return ((pos[0] - self.ox) / self.cell,
                (pos[1] - self.oy) / self.cell)

    def _ev_play(self, event):
        if event.kind == InputEvent.KEYDOWN:
            self._ev_key(event.key)
        elif event.kind == InputEvent.MOUSEMOVE:
            self.mouse = event.pos
        elif event.kind == InputEvent.WHEEL:
            if self.bar_rect.collidepoint(event.pos or (0, 0)):
                over = max(0, self.cards_h - self.cards_view_h)
                self.bar_scroll = max(0, min(over,
                                             self.bar_scroll - event.delta * 30))
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 3:
            if self.armed or self.sel_card:
                self.armed = None
                self.sel_card = None
            else:
                self.sel_tower = None
            self.play_sound("click")
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            self.mouse = event.pos
            self._ev_click(event.pos)

    def _ev_key(self, key):
        if key in ("space", "Return"):
            self._start_wave()
        elif key in ("f", "F"):
            self.ff = 2 if self.ff == 1 else 1
            self.play_sound("click")
        elif key in ("g", "G"):
            self.show_ranges = not self.show_ranges
        elif key in ("u", "U") and self.sel_tower in self.towers:
            self._upgrade_tower(self.towers[self.sel_tower])
        elif key in ("x", "X"):
            if self.sel_card or self.armed:
                self.sel_card = None
                self.armed = None
            elif self.sel_tower in self.towers:
                self._sell_tower(self.sel_tower)
        elif key in ("q", "Q"):
            self._use_ability("meteor")
        elif key in ("w", "W"):
            self._use_ability("nova")
        elif key in ("e", "E"):
            self._use_ability("gold")
        elif key.isdigit():
            i = (int(key) - 1) % 10
            if 0 <= i < len(self.bar_keys):
                self._select_card(self.bar_keys[i])

    def _select_card(self, key):
        if TOWERS[key]["cost"] > self.gold:
            self.play_sound("hit")
            return
        self.sel_card = key if self.sel_card != key else None
        self.sel_tower = None
        self.armed = None
        self.play_sound("click")

    def _ev_click(self, pos):
        # Seitenleiste
        if self.bar_rect.collidepoint(pos):
            for k, r in self.abil_rects.items():
                if r.collidepoint(pos):
                    self._use_ability(k)
                    return
            if self.start_rect.collidepoint(pos) and self.state == BUILD:
                self._start_wave()
                return
            tw = self.towers.get(self.sel_tower)
            if tw is not None and self.info_rect.collidepoint(pos):
                branchable = (self.rules["branch"] and not tw["branch"]
                              and tw["level"] >= self.rules["levels"])
                if branchable and self.br_a_rect.collidepoint(pos):
                    self._branch_tower(tw, "a")
                    return
                if branchable and self.br_b_rect.collidepoint(pos):
                    self._branch_tower(tw, "b")
                    return
                if (not branchable and self.up_rect.collidepoint(pos)
                        and tw["level"] < self.rules["levels"]
                        and not tw["branch"]):
                    self._upgrade_tower(tw)
                    return
                if self.sell_rect.collidepoint(pos):
                    self._sell_tower(self.sel_tower)
                    return
            for key, r in zip(self.bar_keys, self.card_rects):
                rr = r.move(0, -self.bar_scroll)
                if rr.bottom < self.cards_top or rr.y > self.cards_top + \
                        self.cards_view_h:
                    continue
                if rr.collidepoint(pos):
                    self._select_card(key)
                    return
            return
        # HUD: Tempo-Chip
        if getattr(self, "speed_rect", None) and \
                self.speed_rect.collidepoint(pos):
            self.ff = 2 if self.ff == 1 else 1
            self.play_sound("click")
            return
        # Spielfeld
        if self.armed == "meteor":
            if self.field_rect.collidepoint(pos):
                self._cast_meteor(self._cellpos_at(pos))
            return
        cell = self._cell_at(pos)
        if cell is None:
            return
        if self.sel_card:
            if self._can_build(cell):
                self._place_tower(cell)
            else:
                self.play_sound("hit")
            return
        if cell in self.towers:
            self.sel_tower = cell
            self.play_sound("click")
        else:
            self.sel_tower = None

    # ===================================================== Zeichnen
    def draw(self):
        if self.state == MAPSEL:
            self._draw_mapsel()
            return
        s = self.surface
        s.blit(self._field_bg(), (0, 0))
        self._draw_towers(s)
        self._draw_enemies(s)
        self._draw_shots(s)
        self._draw_fx(s)
        self._draw_ghost(s)
        self._draw_hud(s)
        self._draw_sidebar(s)
        if self.state == GAMEOVER:
            self._draw_gameover(s)

    # ----- Spielfeld-Hintergrund (gecacht) -------------------------------
    def _field_bg(self):
        key = (MAPS[self.map_idx]["id"], self.width, self.height, self.cell)
        if self._bg_cache is not None and self._bg_cache[0] == key:
            return self._bg_cache[1]
        surf = pygame.Surface((self.width, self.height))
        surf.fill(ui.mix(COL_GRASS2, (0, 0, 0), 0.35))
        cs = self.cell
        for r in range(GRID_H):
            for c in range(GRID_W):
                col = COL_GRASS1 if (c + r) % 2 == 0 else COL_GRASS2
                pygame.draw.rect(surf, col,
                                 (self.ox + c * cs, self.oy + r * cs, cs, cs))
        # baubare Zellen der Spießrutenlauf-Karte markieren
        if self.build_cells is not None:
            for c, r in self.build_cells:
                pygame.draw.rect(
                    surf, ui.mix(COL_GRASS1, (255, 255, 255), 0.10),
                    (self.ox + c * cs + 2, self.oy + r * cs + 2,
                     cs - 4, cs - 4), 1)
        # Pfad: breite dunkle Kante + hellerer Kern entlang der Polylinie
        pts = [self._px(x, y) for x, y in self.path_pts]
        for width, col in ((int(cs * 0.86), COL_PATH_EDGE),
                           (int(cs * 0.62), COL_PATH)):
            for a, b in zip(pts, pts[1:]):
                pygame.draw.line(surf, col, a, b, width)
            for p in pts:
                pygame.draw.circle(surf, col, p, width // 2)
        # Start-Pfeil und Ziel-Portal
        sx, sy = pts[0]
        ex, ey = pts[-1]
        pygame.draw.circle(surf, (30, 26, 40), (int(ex), int(ey)),
                           int(cs * 0.42))
        pygame.draw.circle(surf, (120, 90, 200), (int(ex), int(ey)),
                           int(cs * 0.42), 3)
        d = 1 if sx < self.field_rect.centerx else -1
        pygame.draw.polygon(surf, (240, 220, 130),
                            [(sx - d * cs * 0.3, sy - cs * 0.35),
                             (sx + d * cs * 0.45, sy),
                             (sx - d * cs * 0.3, sy + cs * 0.35)])
        pygame.draw.rect(surf, ui.BORDER, self.field_rect, 2)
        self._bg_cache = (key, surf)
        return surf

    # ----- Türme ---------------------------------------------------------
    def _draw_tower(self, s, tw, px, py, ghost=False):
        cs = self.cell
        kind = tw["kind"]
        col = TOWERS[kind]["col"]
        base_r = int(cs * 0.38)
        pygame.draw.circle(s, (38, 42, 50), (px, py), base_r)
        pygame.draw.circle(s, col, (px, py), base_r, 2)
        a = tw.get("angle", 0.0)
        blen = cs * 0.45
        bx, by = px + math.cos(a) * blen, py + math.sin(a) * blen
        if kind == "arrow":
            pygame.draw.line(s, col, (px, py), (bx, by), 3)
        elif kind == "cannon":
            pygame.draw.line(s, col, (px, py), (bx, by), max(4, cs // 6))
            pygame.draw.circle(s, col, (px, py), int(cs * 0.2))
        elif kind == "frost":
            pts = [(px + math.cos(a + i * math.tau / 6) * cs * 0.24,
                    py + math.sin(a + i * math.tau / 6) * cs * 0.24)
                   for i in range(6)]
            pygame.draw.polygon(s, col, pts, 2)
        elif kind == "sniper":
            pygame.draw.line(s, col, (px, py),
                             (px + math.cos(a) * cs * 0.62,
                              py + math.sin(a) * cs * 0.62), 2)
            pygame.draw.circle(s, col, (px, py), int(cs * 0.12))
        elif kind == "poison":
            pygame.draw.circle(s, col, (px, py), int(cs * 0.2))
            pygame.draw.circle(s, ui.mix(col, (0, 0, 0), 0.4),
                               (px, py - int(cs * 0.12)), int(cs * 0.1))
        elif kind == "tesla":
            pygame.draw.circle(s, col, (px, py), int(cs * 0.14))
            for i in range(3):
                aa = self.anim_t * 2 + i * math.tau / 3
                pygame.draw.line(s, col, (px, py),
                                 (px + math.cos(aa) * cs * 0.3,
                                  py + math.sin(aa) * cs * 0.3), 2)
        elif kind == "support":
            rr = int(cs * (0.2 + 0.06 * ui.pulse(2.0)))
            pygame.draw.circle(s, col, (px, py), rr, 2)
        elif kind == "mortar":
            pygame.draw.circle(s, col, (px, py), int(cs * 0.26), 4)
        elif kind == "flak":
            for off in (-0.08, 0.08):
                ox_ = math.cos(a + math.pi / 2) * cs * off
                oy_ = math.sin(a + math.pi / 2) * cs * off
                pygame.draw.line(s, col, (px + ox_, py + oy_),
                                 (bx + ox_, by + oy_), 2)
        elif kind == "laser":
            pts = [(px + math.cos(a) * cs * 0.3, py + math.sin(a) * cs * 0.3),
                   (px + math.cos(a + 2.2) * cs * 0.2,
                    py + math.sin(a + 2.2) * cs * 0.2),
                   (px + math.cos(a - 2.2) * cs * 0.2,
                    py + math.sin(a - 2.2) * cs * 0.2)]
            pygame.draw.polygon(s, col, pts)
        elif kind == "bank":
            rr = int(cs * 0.2)
            pygame.draw.rect(s, col, (px - rr, py - rr, rr * 2, rr * 2))
            pygame.draw.rect(s, ui.mix(col, (0, 0, 0), 0.4),
                             (px - rr, py - rr, rr * 2, rr * 2), 2)
        if ghost:
            return
        # Stufen-Pips + Zweig-Punkt
        for i in range(tw["level"] - 1):
            pygame.draw.rect(s, col, (px - base_r + i * 6, py + base_r - 2,
                                      4, 4))
        if tw.get("branch"):
            bc = (255, 255, 255) if tw["branch"] == "a" else (255, 200, 90)
            pygame.draw.circle(s, bc, (px + base_r - 3, py - base_r + 3), 3)

    def _draw_towers(self, s):
        cs = self.cell
        for cell, tw in self.towers.items():
            px, py = self._px(cell[0] + 0.5, cell[1] + 0.5)
            px, py = int(px), int(py)
            sel = (cell == self.sel_tower)
            if sel or self.show_ranges:
                rng = tw["st"]["rng"]
                if rng:
                    pygame.draw.circle(s, ui.mix(TOWERS[tw["kind"]]["col"],
                                                 (255, 255, 255), 0.2),
                                       (px, py), int(rng * cs), 1)
            if sel:
                pygame.draw.rect(s, self.accent,
                                 (self.ox + cell[0] * cs,
                                  self.oy + cell[1] * cs, cs, cs), 2)
            self._draw_tower(s, tw, px, py)
            if TOWERS[tw["kind"]].get("beam") and tw.get("beam") is not None:
                e = tw["beam"]
                if e["hp"] > 0:
                    ex, ey = self._pos_at(e["d"], e["fly"])
                    epx, epy = self._px(ex, ey)
                    w = 2 + int(tw["ramp"])
                    pygame.draw.line(s, (255, 120, 110), (px, py),
                                     (epx, epy), w)

    # ----- Gegner --------------------------------------------------------
    def _draw_enemies(self, s):
        cs = self.cell
        ground = [e for e in self.enemies if not e["fly"]]
        air = [e for e in self.enemies if e["fly"]]
        for e in ground + air:
            ex, ey = self._pos_at(e["d"], e["fly"])
            px, py = self._px(ex, ey)
            px, py = int(px), int(py)
            r = max(3, int(e["r"] * cs))
            if e["fly"]:
                pygame.draw.ellipse(s, COL_AIR_SHADOW,
                                    (px - r, py + int(cs * 0.28), r * 2,
                                     max(3, r // 2)))
                py -= int(cs * 0.22)
            if e["stealth"] and not e["seen"]:
                pygame.draw.circle(s, ui.mix(COL_GRASS1, (255, 255, 255),
                                             0.18), (px, py), r, 1)
                continue
            col = e["col"]
            if e["slow_t"] > 0:
                col = ui.mix(col, (130, 200, 255), 0.45)
            kind = e["kind"]
            if kind == "tank":
                pygame.draw.rect(s, col, (px - r, py - r, r * 2, r * 2),
                                 border_radius=3)
                pygame.draw.rect(s, ui.mix(col, (0, 0, 0), 0.4),
                                 (px - r, py - r, r * 2, r * 2), 2,
                                 border_radius=3)
            elif kind == "fast":
                pygame.draw.polygon(s, col, [(px + r, py), (px - r, py - r),
                                             (px - r, py + r)])
            elif kind == "boss":
                pygame.draw.circle(s, col, (px, py), r)
                pygame.draw.circle(s, ui.mix(col, (0, 0, 0), 0.45),
                                   (px, py), r, 3)
                for i in range(6):
                    aa = self.anim_t * 1.5 + i * math.tau / 6
                    pygame.draw.line(s, col,
                                     (px + math.cos(aa) * r,
                                      py + math.sin(aa) * r),
                                     (px + math.cos(aa) * (r + cs * 0.18),
                                      py + math.sin(aa) * (r + cs * 0.18)), 2)
            else:
                pygame.draw.circle(s, col, (px, py), r)
                pygame.draw.circle(s, ui.mix(col, (0, 0, 0), 0.4),
                                   (px, py), r, 2)
            if kind == "shield":
                pygame.draw.circle(s, (220, 225, 255), (px, py), r + 2, 1)
            elif kind == "regen":
                pygame.draw.line(s, (240, 255, 240), (px - 3, py),
                                 (px + 3, py), 2)
                pygame.draw.line(s, (240, 255, 240), (px, py - 3),
                                 (px, py + 3), 2)
            elif kind == "healer":
                pygame.draw.circle(s, (230, 255, 245), (px, py), r + 3, 1)
            elif kind == "splitter":
                pygame.draw.circle(s, ui.mix(e["col"], (0, 0, 0), 0.35),
                                   (px - r // 3, py), 2)
                pygame.draw.circle(s, ui.mix(e["col"], (0, 0, 0), 0.35),
                                   (px + r // 3, py), 2)
            if e["dot_t"] > 0:
                pygame.draw.circle(s, (140, 220, 80), (px + r - 1, py - r + 1),
                                   3)
            if e["hp"] < e["maxhp"]:
                bw = r * 2
                frac = max(0.0, e["hp"] / e["maxhp"])
                y0 = py - r - 6
                pygame.draw.rect(s, COL_HP_BG, (px - r, y0, bw, 3))
                pygame.draw.rect(s, ui.mix((220, 80, 70), COL_HP, frac),
                                 (px - r, y0, int(bw * frac), 3))

    # ----- Schüsse / Effekte ---------------------------------------------
    def _draw_shots(self, s):
        cs = self.cell
        for sh in self.shots:
            if sh["kind"] == "lob":
                f = sh["t"] / sh["dur"]
                x = sh["x"] + (sh["tx"] - sh["x"]) * f
                y = sh["y"] + (sh["ty"] - sh["y"]) * f
                px, py = self._px(x, y)
                tx, ty = self._px(sh["tx"], sh["ty"])
                pygame.draw.circle(s, (20, 20, 20), (int(tx), int(ty)), 3)
                arc = math.sin(f * math.pi) * cs * 1.1
                pygame.draw.circle(s, sh["col"], (int(px), int(py - arc)), 4)
            else:
                px, py = self._px(sh["x"], sh["y"])
                pygame.draw.circle(s, sh["col"], (int(px), int(py)), 3)

    def _draw_fx(self, s):
        cs = self.cell
        for f in self.fx:
            fr = f["age"] / f["ttl"]
            if f["kind"] == "tracer":
                a = self._px(*f["a"])
                b = self._px(*f["b"])
                pygame.draw.line(s, f["col"], a, b, 2)
            elif f["kind"] == "zigzag":
                pts = [self._px(x, y) for x, y in f["pts"]]
                for p0, p1 in zip(pts, pts[1:]):
                    mx = (p0[0] + p1[0]) / 2 + random.uniform(-4, 4)
                    my = (p0[1] + p1[1]) / 2 + random.uniform(-4, 4)
                    pygame.draw.lines(s, f["col"], False, [p0, (mx, my), p1],
                                      2)
            elif f["kind"] == "ring":
                px, py = self._px(*f["pos"])
                rr = int(f["r"] * cs * (0.3 + 0.7 * fr))
                pygame.draw.circle(s, f["col"], (int(px), int(py)), rr, 2)
            elif f["kind"] == "burst":
                px, py = self._px(*f["pos"])
                for i in range(5):
                    aa = i * math.tau / 5 + fr * 2
                    d = fr * cs * 0.7
                    pygame.draw.circle(s, f["col"],
                                       (int(px + math.cos(aa) * d),
                                        int(py + math.sin(aa) * d)), 2)
            elif f["kind"] == "nova":
                px, py = self._px(*f["pos"])
                rr = int(fr * self.field_rect.w * 0.7)
                pygame.draw.circle(s, (140, 210, 255), (int(px), int(py)),
                                   max(2, rr), 2)

    def _draw_ghost(self, s):
        """Platzierungs-Vorschau bzw. Meteor-Zielkreis unter der Maus."""
        if self.mouse is None or self.state == GAMEOVER:
            return
        cs = self.cell
        if self.armed == "meteor":
            if self.field_rect.collidepoint(self.mouse):
                px, py = self.mouse
                pygame.draw.circle(s, (255, 150, 80), (px, py),
                                   int(1.8 * cs), 2)
            return
        if not self.sel_card:
            return
        cell = self._cell_at(self.mouse)
        if cell is None:
            return
        ok = self._can_build(cell)
        px, py = self._px(cell[0] + 0.5, cell[1] + 0.5)
        px, py = int(px), int(py)
        col = COL_BUILD_OK if ok else COL_BUILD_BAD
        pygame.draw.rect(s, col, (self.ox + cell[0] * cs,
                                  self.oy + cell[1] * cs, cs, cs), 2)
        rng = TOWERS[self.sel_card].get("rng", 0)
        if rng:
            pygame.draw.circle(s, col, (px, py), int(rng * cs), 1)
        ghost = dict(kind=self.sel_card, level=1, angle=0.0)
        self._draw_tower(s, ghost, px, py, ghost=True)

    # ----- HUD -----------------------------------------------------------
    def _heart(self, s, x, y, r, col):
        pygame.draw.circle(s, col, (x - r // 2, y - r // 3), r // 2 + 1)
        pygame.draw.circle(s, col, (x + r // 2, y - r // 3), r // 2 + 1)
        pygame.draw.polygon(s, col, [(x - r, y - r // 4), (x + r, y - r // 4),
                                     (x, y + r)])

    def _draw_hud(self, s):
        w = self.width - self.bar_w
        pygame.draw.rect(s, ui.PANEL, (0, 0, w, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h - 1),
                         (w, self.hud_h - 1))
        cy = self.hud_h // 2
        x = 14
        self._heart(s, x + 5, cy, 7, (235, 90, 100))
        img = self._small.render(str(max(0, self.lives)), True, ui.TEXT)
        s.blit(img, (x + 18, cy - img.get_height() // 2))
        x += 26 + img.get_width() + 14
        pygame.draw.circle(s, ui.GOLD, (x + 6, cy), 7)
        pygame.draw.circle(s, ui.mix(ui.GOLD, (0, 0, 0), 0.4), (x + 6, cy),
                           7, 2)
        img = self._small.render(str(self.gold), True, ui.GOLD)
        s.blit(img, (x + 18, cy - img.get_height() // 2))
        x += 26 + img.get_width() + 16
        img = self._small.render(t("td.wave", n=self.wave), True,
                                 self.accent)
        s.blit(img, (x, cy - img.get_height() // 2))
        x += img.get_width() + 16
        if self.state == WAVE:
            left = len(self.enemies) + len(self.spawn_queue)
            img = self._tiny.render(t("td.left", n=left), True, ui.TEXT_DIM)
            s.blit(img, (x, cy - img.get_height() // 2))
        # Punkte + Tempo rechtsbündig
        img = self._small.render(str(self.score), True, ui.TEXT)
        chip_w = max(40, self._tiny.size("2x")[0] + 18)
        self.speed_rect = pygame.Rect(w - chip_w - 10, 5, chip_w,
                                      self.hud_h - 10)
        s.blit(img, (self.speed_rect.x - img.get_width() - 14,
                     cy - img.get_height() // 2))
        sel = self.ff > 1
        pygame.draw.rect(s, ui.PANEL_LIGHT if sel else ui.PANEL,
                         self.speed_rect, border_radius=6)
        pygame.draw.rect(s, self.accent if sel else ui.BORDER,
                         self.speed_rect, 1, border_radius=6)
        img = self._tiny.render("%dx" % self.ff, True,
                                self.accent if sel else ui.TEXT_DIM)
        s.blit(img, img.get_rect(center=self.speed_rect.center))

    # ----- Seitenleiste --------------------------------------------------
    def _draw_sidebar(self, s):
        pygame.draw.rect(s, ui.PANEL, self.bar_rect)
        pygame.draw.line(s, ui.BORDER, (self.bar_rect.x, 0),
                         (self.bar_rect.x, self.height))
        # Fähigkeiten (Maximal)
        for key, r in self.abil_rects.items():
            cd = self.abil_cd[key]
            ready = cd <= 0
            armed = (self.armed == key)
            pygame.draw.rect(s, ui.PANEL_LIGHT if ready else ui.PANEL, r,
                             border_radius=6)
            pygame.draw.rect(s, self.accent if armed else ui.BORDER, r,
                             1 + armed, border_radius=6)
            cx, cy = r.center
            if key == "meteor":
                pygame.draw.circle(s, (255, 150, 80), (cx, cy - 2), 5)
                pygame.draw.line(s, (255, 150, 80), (cx + 3, cy - 5),
                                 (cx + 8, cy - 10), 2)
            elif key == "nova":
                pygame.draw.circle(s, (140, 210, 255), (cx, cy - 2), 6, 2)
            else:
                pygame.draw.circle(s, ui.GOLD, (cx, cy - 2), 6)
            if not ready:
                img = self._tiny.render(str(int(cd) + 1), True, ui.TEXT_DIM)
                s.blit(img, img.get_rect(midbottom=(cx, r.bottom - 1)))
        # Turmkarten (mit Scroll-Ausschnitt)
        clip_old = s.get_clip()
        s.set_clip(pygame.Rect(self.bar_rect.x, self.cards_top, self.bar_w,
                               self.cards_view_h))
        hover_key = None
        for key, r in zip(self.bar_keys, self.card_rects):
            rr = r.move(0, -self.bar_scroll)
            base = TOWERS[key]
            afford = self.gold >= base["cost"]
            sel = (self.sel_card == key)
            pygame.draw.rect(s, ui.PANEL_LIGHT if sel else ui.PANEL, rr,
                             border_radius=8)
            pygame.draw.rect(s, self.accent if sel else ui.BORDER, rr,
                             2 if sel else 1, border_radius=8)
            col = base["col"] if afford else ui.mix(base["col"], ui.PANEL,
                                                    0.6)
            ghost = dict(kind=key, level=1, angle=-math.pi / 2)
            self._draw_tower(s, ghost, rr.centerx, rr.centery - 6,
                             ghost=True)
            img = self._tiny.render(str(base["cost"]), True,
                                    ui.GOLD if afford else ui.TEXT_FAINT)
            s.blit(img, img.get_rect(midbottom=(rr.centerx, rr.bottom - 3)))
            if self.mouse and rr.collidepoint(self.mouse):
                hover_key = key
        s.set_clip(clip_old)
        # Info-Bereich
        self._draw_info(s, hover_key)
        # Start-Button / Wellen-Status
        if self.state == BUILD:
            ui.draw_button(s, self.start_rect, t("td.start_wave"),
                           self._small, selected=True, accent=self.accent)
        else:
            pygame.draw.rect(s, ui.PANEL_LIGHT, self.start_rect,
                             border_radius=8)
            pygame.draw.rect(s, ui.BORDER, self.start_rect, 1,
                             border_radius=8)
            img = self._small.render(t("td.wave", n=self.wave), True,
                                     ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=self.start_rect.center))

    def _info_lines(self, key, tw=None):
        base = TOWERS[key]
        st = tw["st"] if tw else None
        dmg = st["dmg"] if st else base.get("dmg", 0)
        rng = st["rng"] if st else base.get("rng", 0)
        cd = st["cd"] if st else base.get("cd", 0)
        lines = []
        if base.get("income"):
            inc = st["income"] if st else base["income"]
            lines.append("+%d/W" % int(inc))
        elif base.get("buff"):
            b = st["buff"] if st else base["buff"]
            lines.append("+%d%%" % int(b * 100))
        else:
            if base.get("beam"):
                lines.append("%d/s" % int(dmg))
            else:
                lines.append("%d  |  %.1fs" % (int(dmg), cd))
        if rng:
            lines.append("R %.1f" % rng)
        return "   ".join(lines)

    def _draw_info(self, s, hover_key):
        r = self.info_rect
        pygame.draw.rect(s, ui.PANEL_LIGHT, r, border_radius=8)
        pygame.draw.rect(s, ui.BORDER, r, 1, border_radius=8)
        x, y = r.x + 10, r.y + 8
        tw = self.towers.get(self.sel_tower)
        if tw is not None:
            base = TOWERS[tw["kind"]]
            img = self._small.render(t("td.tower." + tw["kind"]), True,
                                     base["col"])
            s.blit(img, (x, y))
            y += img.get_height() + 2
            img = self._tiny.render(t("td.level", n=tw["level"]), True,
                                    ui.TEXT_DIM)
            s.blit(img, (x, y))
            y += img.get_height() + 2
            img = self._tiny.render(self._info_lines(tw["kind"], tw), True,
                                    ui.TEXT)
            s.blit(img, (x, y))
            branchable = (self.rules["branch"] and not tw["branch"]
                          and tw["level"] >= self.rules["levels"])
            if branchable:
                for which, rect in (("a", self.br_a_rect),
                                    ("b", self.br_b_rect)):
                    cost = self._branch_cost(tw)
                    ok = self.gold >= cost
                    pygame.draw.rect(s, ui.PANEL, rect, border_radius=6)
                    pygame.draw.rect(s, self.accent if ok else ui.BORDER,
                                     rect, 1, border_radius=6)
                    lab = t("td.br.%s.%s" % (tw["kind"], which))
                    img = self._tiny.render(lab, True,
                                            ui.TEXT if ok else ui.TEXT_FAINT)
                    s.blit(img, img.get_rect(
                        center=(rect.centerx, rect.centery - 6)))
                    img = self._tiny.render(str(cost), True,
                                            ui.GOLD if ok else ui.TEXT_FAINT)
                    s.blit(img, img.get_rect(
                        center=(rect.centerx, rect.centery + 8)))
            elif tw["level"] < self.rules["levels"] and not tw["branch"]:
                cost = self._upgrade_cost(tw)
                ok = self.gold >= cost
                pygame.draw.rect(s, ui.PANEL, self.up_rect, border_radius=6)
                pygame.draw.rect(s, self.accent if ok else ui.BORDER,
                                 self.up_rect, 1, border_radius=6)
                img = self._tiny.render(t("td.upgrade", c=cost), True,
                                        ui.TEXT if ok else ui.TEXT_FAINT)
                s.blit(img, img.get_rect(center=self.up_rect.center))
            else:
                img = self._tiny.render(t("td.max"), True, ui.GOLD)
                s.blit(img, img.get_rect(center=self.up_rect.center))
            pygame.draw.rect(s, ui.PANEL, self.sell_rect, border_radius=6)
            pygame.draw.rect(s, ui.BORDER, self.sell_rect, 1,
                             border_radius=6)
            img = self._tiny.render(t("td.sell", c=self._sell_value(tw)),
                                    True, ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=self.sell_rect.center))
            return
        key = hover_key or self.sel_card
        if key:
            base = TOWERS[key]
            img = self._small.render(t("td.tower." + key), True, base["col"])
            s.blit(img, (x, y))
            y += img.get_height() + 2
            img = self._tiny.render(self._info_lines(key), True, ui.TEXT)
            s.blit(img, (x, y))
            y += img.get_height() + 4
            desc = t("td.tower.%s.desc" % key)
            y = self._wrap_text(s, desc, x, y, r.w - 20, self._tiny,
                                ui.TEXT_DIM)
            if self.sel_card:
                y += 4
                self._wrap_text(s, t("td.place_hint"), x, y, r.w - 20,
                                self._tiny, ui.TEXT_FAINT)
            return
        # Standard: Kurzhilfe
        hint = t("td.hint.build") if self.state == BUILD \
            else t("td.hint.wave")
        self._wrap_text(s, hint, x, y, r.w - 20, self._tiny, ui.TEXT_DIM)

    def _wrap_text(self, s, text, x, y, maxw, fnt, col):
        words = text.split()
        line = ""
        for wd in words:
            test = (line + " " + wd).strip()
            if fnt.size(test)[0] > maxw and line:
                img = fnt.render(line, True, col)
                s.blit(img, (x, y))
                y += img.get_height() + 1
                line = wd
            else:
                line = test
        if line:
            img = fnt.render(line, True, col)
            s.blit(img, (x, y))
            y += img.get_height() + 1
        return y

    # ----- Kartenwahl / Game Over ----------------------------------------
    def _ui_bg(self):
        key = (self.width, self.height, ui.BG_TOP, ui.BG_BOTTOM)
        if self._dim_cache is None or self._dim_cache[0] != key:
            surf = pygame.Surface((self.width, self.height))
            ui.draw_background(surf, self.width, self.height)
            self._dim_cache = (key, surf)
        return self._dim_cache[1]

    def _draw_mapsel(self):
        s = self.surface
        s.blit(self._ui_bg(), (0, 0))
        ui.draw_title(s, self.width, "TOWER DEFENSE",
                      subtitle=t("td.choose_map"), accent=self.accent)
        for i, (m, r) in enumerate(zip(MAPS, self.map_rects)):
            unlocked = self._unlocked(m)
            sel = (i == self.mapsel_idx) or \
                (self.mouse and r.collidepoint(self.mouse))
            ui.draw_panel(s, r, accent_top=self.accent if sel else None)
            if sel:
                pygame.draw.rect(s, self.accent, r, 2, border_radius=10)
            # Pfad-Vorschau
            pad = 18
            pw, ph = r.w - pad * 2, r.h - pad * 2 - 34
            pts = [(r.x + pad + (x / GRID_W) * pw,
                    r.y + pad + (y / GRID_H) * ph)
                   for x, y in self.__class__._preview_pts(m)]
            col = ui.TEXT_DIM if unlocked else ui.TEXT_FAINT
            pygame.draw.lines(s, col, False, pts, 3)
            pygame.draw.circle(s, ui.GREEN if unlocked else ui.TEXT_FAINT,
                               (int(pts[0][0]), int(pts[0][1])), 4)
            pygame.draw.circle(s, self.accent if unlocked else ui.TEXT_FAINT,
                               (int(pts[-1][0]), int(pts[-1][1])), 4)
            name = t("td.map." + m["id"])
            img = self._small.render(name, True,
                                     ui.TEXT if unlocked else ui.TEXT_FAINT)
            s.blit(img, (r.x + 14, r.bottom - 30))
            # Schwierigkeit als Punkte
            for j in range(4):
                cc = self.accent if j <= i else ui.BORDER
                pygame.draw.circle(s, cc, (r.right - 60 + j * 12,
                                           r.bottom - 22), 3)
            if unlocked:
                best = self.best.get(m["id"], 0)
                if best:
                    img = self._tiny.render(t("td.map.best", n=best), True,
                                            ui.GOLD)
                    s.blit(img, (r.x + 14, r.y + 6))
            else:
                img = self._tiny.render(t("td.map.locked", n=m["unlock"]),
                                        True, ui.TEXT_FAINT)
                s.blit(img, (r.x + 14, r.y + 6))
        ui.draw_footer(s, self.width, self.height, t("td.setup_hint"))

    @staticmethod
    def _preview_pts(m):
        pts = [(c + 0.5, r + 0.5) for c, r in m["path"]]
        return [(max(0.0, min(GRID_W, x)), max(0.0, min(GRID_H, y)))
                for x, y in pts]

    def _dim(self):
        surf = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        surf.fill((10, 12, 20, 150))
        return surf

    def _draw_gameover(self, s):
        if not hasattr(self, "_go_dim") or \
                self._go_dim.get_size() != (self.width, self.height):
            self._go_dim = self._dim()
        s.blit(self._go_dim, (0, 0))
        cx = self.width // 2
        img = self.big_font.render(t("common.game_over"), True, self.accent)
        s.blit(img, img.get_rect(center=(cx, int(self.height * 0.2))))
        lh = self._small.get_height() + 8
        pw = min(380, self.width - 40)
        panel = pygame.Rect(cx - pw // 2, int(self.height * 0.34), pw,
                            lh * 4 + 36)
        ui.draw_panel(s, panel, accent_top=self.accent)
        x, y = panel.x + 20, panel.y + 16
        mid = MAPS[self.map_idx]["id"]
        rows = [
            (t("td.gameover.wave", n=self.wave - 1), ui.TEXT),
            (t("common.points", score=self.score), ui.TEXT),
            (t("td.kills", n=self.kills), ui.TEXT_DIM),
            (t("td.map.best", n=self.best.get(mid, 0)), ui.GOLD),
        ]
        for text, col in rows:
            img = self._small.render(text, True, col)
            s.blit(img, (x, y))
            y += lh
        hint_col = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.4, 0.0, 1.0))
        img = self._small.render(t("td.restart_hint"), True, hint_col)
        s.blit(img, img.get_rect(center=(cx, panel.bottom + 26)))
