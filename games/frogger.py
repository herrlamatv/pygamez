# -*- coding: utf-8 -*-
"""
frogger.py
==========
Frogger - Klassiker mit Extras (Einzelspieler).

Aufbau (13 logische Reihen von oben nach unten):
- Reihe 0     : Ufer mit 5 Ziel-Buchten (alle füllen = Level geschafft)
- Reihen 1-5  : Fluss - Stämme und Schildkröten-Gruppen tragen den Frosch;
                Schildkröten tauchen ab höheren Leveln periodisch ab
- Reihe 6     : Mittelstreifen (sicher)
- Reihen 7-11 : Straße - Autos und Laster in wechselnden Richtungen
- Reihe 12    : Startstreifen (sicher)

Extras: Bonus-Fliege (+200) erscheint zeitweise in einer leeren Bucht,
Krokodile besetzen ab höheren Leveln Buchten (Landung = Tod), Zeitlimit-
Balken je Frosch, 3 Schwierigkeitsgrade (Tempo/Dichte/Zeit), Level werden
schneller. 3 Leben, Extraleben einmalig bei 10 000 Punkten.

Steuerung: Pfeile/WASD = hüpfen, R = neu, S = Setup.
"""

import math
import random

import pygame

import settings as settings_mod
import ui
from game_base import Game, InputEvent
from i18n import t

# ----- Identitätsfarben des Spielfelds (bewusst fest, unabhängig vom Theme):
# Ufer, Fluss und Straße samt Fahrzeugen/Stämmen SIND Frogger. --------------
COL_GRASS = (36, 66, 46)          # Ufer/Mittel-/Startstreifen
COL_GRASS_DARK = (28, 52, 38)
COL_ROAD = (30, 33, 44)
COL_ROAD_LINE = (70, 76, 96)
COL_RIVER = (24, 38, 66)
COL_BAY = (16, 24, 38)            # leere Bucht
COL_BAY_DONE = (56, 120, 80)      # gefüllte Bucht
COL_FROG = (108, 205, 109)        # Froschkörper (helles Grün)
COL_FROG_DARK = (58, 130, 78)
COL_CAR = [(225, 95, 95), (245, 205, 100), (150, 160, 235), (240, 240, 250)]
COL_TRUCK = (150, 158, 178)
COL_LOG = (128, 92, 60)
COL_LOG_DARK = (96, 68, 44)
COL_TURTLE = (90, 160, 120)
COL_TURTLE_DARK = (60, 110, 84)
COL_CROC = (70, 130, 70)
COL_FLY = (245, 205, 100)

# Schwierigkeit: Tempo-Faktor, Lücken (Zellen), Zeit je Frosch (s),
# Krokodile ab Level, tauchende Schildkröten ab Level.
DIFFS = [
    dict(key="easy", speed=0.8, gap=3.5, timer=45, croc_lv=4, dive_lv=3),
    dict(key="normal", speed=1.0, gap=2.75, timer=35, croc_lv=2, dive_lv=2),
    dict(key="hard", speed=1.25, gap=2.0, timer=25, croc_lv=1, dive_lv=1),
]
DIFF_KEYS = [d["key"] for d in DIFFS]

# Lanes: (Art, Basistempo in Zellen/s, Breite in Zellen). Vorzeichen = Richtung.
ROAD_LANES = [("car", 1.6, 1), ("truck", -1.1, 2), ("car", 1.9, 1),
              ("car", -2.4, 1), ("truck", 1.3, 2)]      # Reihen 7..11
RIVER_LANES = [("log", 1.2, 3), ("turtle", -1.5, 3), ("log", 1.9, 4),
               ("turtle", -1.2, 2), ("log", 1.6, 2)]    # Reihen 1..5

EXTRA_LIFE_AT = 10_000
FLY_TIME = 4.0
CROC_TIME = 5.0
DIVE_CYCLE = 10.0        # 6s oben, 1s blinken, 2s unten, 1s auftauchen

SETUP, PLAY = "setup", "play"


def _rgba(color, alpha):
    """Palette-Farbe (RGB) mit einem Alpha-Wert zu RGBA kombinieren."""
    return (color[0], color[1], color[2], alpha)


class FroggerGame(Game):
    name = "Frogger"
    highscore_key = "frogger"
    supports_multiplayer = False

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        fs = self.settings.get("frogger", {}) if isinstance(self.settings, dict) else {}
        key = fs.get("difficulty", "normal")
        self.diff_idx = DIFF_KEYS.index(key) if key in DIFF_KEYS else 1

        self._make_fonts()
        self.anim_t = 0.0

        self._layout()
        self._build_setup_layout()
        self.state = SETUP

    def _make_fonts(self):
        """Theme-Schriften, Größen aus der Fensterhöhe abgeleitet."""
        self._small = ui.font(max(13, min(22, self.height // 30)))
        self._tiny = ui.font(max(11, min(18, self.height // 38)))
        self._huge = ui.font(max(26, self.height // 11), bold=True)

    def _diff(self):
        return DIFFS[self.diff_idx]

    def _layout(self):
        """Spielfeld-Maße aus der aktuellen Auflösung ableiten."""
        self.hud_h = max(30, int(self.height * 0.08))
        self.bar_h = max(8, self.height // 48)
        self.cell = max(16, (self.height - self.hud_h - self.bar_h) // 13)
        self.field_top = self.hud_h

    def on_surface_changed(self):
        self._make_fonts()
        self._layout()
        self._build_setup_layout()
        if self.state == PLAY:
            self._build_lanes()
            self._respawn(full=False)

    def _dim_surface(self):
        """Abdunkelnde Vollbild-Fläche (gecacht, nur bei Größenwechsel neu)."""
        key = (self.width, self.height)
        if getattr(self, "_dim_key", None) != key:
            self._dim_key = key
            self._dim = pygame.Surface(key, pygame.SRCALPHA)
            self._dim.fill((8, 10, 16, 150))
        return self._dim

    def _row_y(self, row):
        return self.field_top + row * self.cell

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(420, self.width - 60)
        y0 = int(self.height * 0.30)
        self.diff_rects = [pygame.Rect(cx - bw // 2, y0 + i * 62, bw, 52)
                           for i in range(3)]
        self.start_rect = pygame.Rect(cx - 95, y0 + 3 * 62 + 14, 190, 46)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("frogger", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("1", "2", "3"):
                self.diff_idx = int(event.key) - 1
                self._save_setting("difficulty", DIFF_KEYS[self.diff_idx])
                self.play_sound("click")
            elif event.key in ("Up", "w", "W"):
                self.diff_idx = (self.diff_idx - 1) % 3
                self._save_setting("difficulty", DIFF_KEYS[self.diff_idx])
                self.play_sound("move")
            elif event.key in ("Down", "s", "S"):
                self.diff_idx = (self.diff_idx + 1) % 3
                self._save_setting("difficulty", DIFF_KEYS[self.diff_idx])
                self.play_sound("move")
            elif event.key in ("Return", "space"):
                self._new_game()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, r in enumerate(self.diff_rects):
                if r.collidepoint(event.pos):
                    self.diff_idx = i
                    self._save_setting("difficulty", DIFF_KEYS[i])
                    self.play_sound("click")
                    return
            if self.start_rect.collidepoint(event.pos):
                self._new_game()

    # ===================================================== Spielaufbau
    def _new_game(self):
        self.score = 0
        self.game_over = False
        self.lives = 3
        self.level = 1
        self.extra_life_given = False
        self.bays_done = [False] * 5
        self.fly = None          # dict(bay=..., t=...)
        self.croc = None
        self.fly_timer = random.uniform(10, 16)
        self.death_t = 0.0
        self.clear_t = 0.0
        self._build_lanes()
        self._respawn(full=True)
        self.state = PLAY
        self.play_sound("click")

    def _speed_mult(self):
        return min(2.5, self._diff()["speed"] * (1 + 0.12 * (self.level - 1)))

    def _build_lanes(self):
        """Erzeugt alle Fahrspuren/Flussbahnen mit gleichmäßig verteilten
        Objekten. Objekte wandern und wrappen über eine gemeinsame Spannweite."""
        self.lanes = []
        d = self._diff()
        mult = self._speed_mult()

        def add_lane(row, kind, base_speed, w_cells):
            speed = base_speed * mult * self.cell
            ew = w_cells * self.cell
            gap = d["gap"] * self.cell * random.uniform(0.8, 1.3)
            margin = ew + self.cell
            n = max(2, int((self.width + 2 * margin) // (ew + gap)) + 1)
            span = n * (ew + gap)
            ents = []
            for i in range(n):
                ents.append(dict(x=i * (ew + gap) - margin, w=ew,
                                 color=random.choice(COL_CAR),
                                 dive=False, phase=random.uniform(0, DIVE_CYCLE)))
            self.lanes.append(dict(row=row, kind=kind, speed=speed,
                                   ents=ents, span=span, margin=margin))

        for i, (kind, sp, w) in enumerate(ROAD_LANES):
            add_lane(7 + i, kind, sp, w)
        for i, (kind, sp, w) in enumerate(RIVER_LANES):
            add_lane(1 + i, kind, sp, w)

        # Tauchende Schildkröten: ab Schwellen-Level jede 2. Gruppe.
        if self.level >= d["dive_lv"]:
            for lane in self.lanes:
                if lane["kind"] == "turtle":
                    for j, e in enumerate(lane["ents"]):
                        e["dive"] = (j % 2 == 0)

    def _respawn(self, full=False):
        self.frog_row = 12
        self.frog_x = self.width / 2
        self.best_row = 12
        self.time_left = float(self._diff()["timer"])
        if full:
            self.anim_t = 0.0

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.game_over:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._new_game()
                elif event.key in ("s", "S"):
                    self.game_over = False
                    self.state = SETUP
                    self.play_sound("click")
            return
        if event.kind != InputEvent.KEYDOWN:
            return
        if event.key in ("r", "R"):
            self._new_game()
            return
        if event.key in ("s", "S") and not self.is_action(event.key, "down"):
            # S gehört standardmäßig zu "runter" (WASD) -> nur als Setup-
            # Taste werten, wenn es nicht als Aktion belegt ist.
            self.state = SETUP
            self.play_sound("click")
            return
        if self.death_t > 0 or self.clear_t > 0:
            return
        if self.is_action(event.key, "up") or event.key == "Up":
            self._hop(0, -1)
        elif self.is_action(event.key, "down") or event.key == "Down":
            self._hop(0, 1)
        elif self.is_action(event.key, "left") or event.key == "Left":
            self._hop(-1, 0)
        elif self.is_action(event.key, "right") or event.key == "Right":
            self._hop(1, 0)

    def _hop(self, dx, dy):
        nx = self.frog_x + dx * self.cell
        nrow = self.frog_row + dy
        if not (0 <= nrow <= 12):
            return
        half = self.cell * 0.5
        if not (half <= nx <= self.width - half):
            return
        self.frog_x = nx
        self.frog_row = nrow
        self.play_sound("move")
        # Punkte für jede neu erreichte (höhere) Reihe in diesem Leben.
        if nrow < self.best_row:
            self.score += 10 * (self.best_row - nrow)
            self.best_row = nrow
        if nrow == 0:
            self._try_home()

    # ===================================================== Spiellogik
    def update(self, dt):
        self.anim_t += dt
        if self.state != PLAY:
            return

        # Objekte bewegen (auch während Todes-/Clear-Anims, wirkt lebendiger)
        for lane in self.lanes:
            for e in lane["ents"]:
                e["x"] += lane["speed"] * dt
                if lane["speed"] > 0 and e["x"] > self.width + lane["margin"]:
                    e["x"] -= lane["span"]
                elif lane["speed"] < 0 and e["x"] + e["w"] < -lane["margin"]:
                    e["x"] += lane["span"]

        # Nach dem Ende läuft nur noch der Verkehr als Kulisse weiter -
        # sonst würden Timer/Fluss den toten Frosch erneut "töten".
        if self.game_over:
            return

        if self.clear_t > 0:
            self.clear_t -= dt
            if self.clear_t <= 0:
                self._next_level()
            return
        if self.death_t > 0:
            self.death_t -= dt
            if self.death_t <= 0:
                self._after_death()
            return

        # Extraleben
        if not self.extra_life_given and self.score >= EXTRA_LIFE_AT:
            self.extra_life_given = True
            self.lives += 1
            self.play_sound("powerup")

        # Fliege / Krokodil in den Buchten
        self._update_bay_extras(dt)

        # Zeitlimit
        self.time_left -= dt
        if self.time_left <= 0:
            self._die()
            return

        # Fluss: mitfahren oder ertrinken
        if 1 <= self.frog_row <= 5:
            carrier = self._carrier_at(self.frog_row, self.frog_x)
            if carrier is None:
                self._die()
                return
            lane = carrier[0]
            self.frog_x += lane["speed"] * dt
            half = self.cell * 0.5
            if self.frog_x < half or self.frog_x > self.width - half:
                self._die()
                return

        # Straße: Kollision
        if 7 <= self.frog_row <= 11:
            if self._vehicle_hit(self.frog_row, self.frog_x):
                self._die()

    def _lane_for_row(self, row):
        for lane in self.lanes:
            if lane["row"] == row:
                return lane
        return None

    def _turtle_state(self, ent):
        """'up' / 'blink' / 'under' / 'rise' je nach Tauch-Zyklus."""
        if not ent["dive"]:
            return "up"
        tt = (self.anim_t + ent["phase"]) % DIVE_CYCLE
        if tt < 6.0:
            return "up"
        if tt < 7.0:
            return "blink"
        if tt < 9.0:
            return "under"
        return "rise"

    def _carrier_at(self, row, x):
        """Stamm/aufgetauchte Schildkröte unter Position x (oder None)."""
        lane = self._lane_for_row(row)
        if lane is None:
            return None
        for e in lane["ents"]:
            if e["x"] <= x <= e["x"] + e["w"]:
                if lane["kind"] == "turtle" and self._turtle_state(e) == "under":
                    return None
                return (lane, e)
        return None

    def _vehicle_hit(self, row, x):
        lane = self._lane_for_row(row)
        if lane is None:
            return False
        half = self.cell * 0.3       # Frosch-Hitbox etwas kleiner als die Zelle
        for e in lane["ents"]:
            if e["x"] < x + half and x - half < e["x"] + e["w"]:
                return True
        return False

    def _bay_index_at(self, x):
        """Bucht, deren Zentrum nah genug an x liegt (oder None)."""
        for i in range(5):
            cx = (i + 0.5) * self.width / 5
            if abs(x - cx) < 0.5 * self.cell:
                return i
        return None

    def _try_home(self):
        bay = self._bay_index_at(self.frog_x)
        if bay is None or self.bays_done[bay]:
            self._die()
            return
        if self.croc is not None and self.croc["bay"] == bay:
            self._die()
            return
        # Geschafft!
        self.bays_done[bay] = True
        bonus = 50 + 10 * int(max(0, self.time_left))
        self.score += bonus
        if self.fly is not None and self.fly["bay"] == bay:
            self.score += 200
            self.fly = None
            self.play_sound("powerup")
        self.play_sound("point")
        if all(self.bays_done):
            self.score += 1000
            self.clear_t = 1.5
            self.ach_event("frog_home")
            self.play_sound("level")
        else:
            self._respawn()

    def _next_level(self):
        self.level += 1
        self.bays_done = [False] * 5
        self.fly = None
        self.croc = None
        self._build_lanes()
        self._respawn()

    def _update_bay_extras(self, dt):
        d = self._diff()
        if self.fly is not None:
            self.fly["t"] -= dt
            if self.fly["t"] <= 0:
                self.fly = None
        if self.croc is not None:
            self.croc["t"] -= dt
            if self.croc["t"] <= 0:
                self.croc = None
        self.fly_timer -= dt
        if self.fly_timer <= 0:
            self.fly_timer = random.uniform(10, 16)
            free = [i for i in range(5) if not self.bays_done[i]
                    and (self.croc is None or self.croc["bay"] != i)
                    and (self.fly is None or self.fly["bay"] != i)]
            if free:
                # Ab dem Krokodil-Level teilen sich Fliege und Krokodil den Takt.
                if self.level >= d["croc_lv"] and self.croc is None \
                        and random.random() < 0.45:
                    self.croc = dict(bay=random.choice(free), t=CROC_TIME)
                elif self.fly is None:
                    self.fly = dict(bay=random.choice(free), t=FLY_TIME)

    def _die(self):
        self.death_t = 1.0
        self.lives -= 1
        self.play_sound("hit")
        self.rumble(160)

    def _after_death(self):
        if self.lives <= 0:
            self.game_over = True
            self.play_sound("gameover")
            self.rumble(250)
        else:
            self._respawn()

    # ===================================================== Zeichnen
    def draw(self):
        if self.state == SETUP:
            self._draw_setup()
            return
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        self._draw_field(s)
        self._draw_entities(s)
        self._draw_bay_extras(s)
        if self.death_t > 0:
            self._draw_death(s)
        elif self.clear_t <= 0:
            self._draw_frog(s, self.frog_x, self._row_y(self.frog_row))
        self._draw_hud(s)
        if self.clear_t > 0:
            self._draw_banner(s, t("frog.level_clear", n=self.level))
        if self.game_over:
            self._draw_gameover(s)

    def _draw_field(self, s):
        c = self.cell
        w = self.width
        # Ufer mit Buchten (Reihe 0)
        y0 = self._row_y(0)
        pygame.draw.rect(s, COL_GRASS, (0, y0, w, c))
        for i in range(5):
            cx = (i + 0.5) * w / 5
            bay = pygame.Rect(int(cx - c * 0.6), y0 + 2, int(c * 1.2), c - 4)
            pygame.draw.rect(s, COL_BAY_DONE if self.bays_done[i] else COL_BAY,
                             bay, border_radius=6)
            if self.bays_done[i]:
                self._draw_frog(s, cx, y0, small=True)
        # Fluss (1-5)
        pygame.draw.rect(s, COL_RIVER, (0, self._row_y(1), w, 5 * c))
        # Mittelstreifen (6) / Start (12)
        pygame.draw.rect(s, COL_GRASS_DARK, (0, self._row_y(6), w, c))
        pygame.draw.rect(s, COL_GRASS_DARK, (0, self._row_y(12), w, c))
        # Straße (7-11) mit Mittellinien
        pygame.draw.rect(s, COL_ROAD, (0, self._row_y(7), w, 5 * c))
        for r in range(8, 12):
            y = self._row_y(r)
            for x in range(0, w, c):
                pygame.draw.rect(s, COL_ROAD_LINE, (x + c // 4, y - 1, c // 2, 2))

    def _draw_entities(self, s):
        c = self.cell
        for lane in self.lanes:
            y = self._row_y(lane["row"])
            for e in lane["ents"]:
                r = pygame.Rect(int(e["x"]), y + 3, int(e["w"]), c - 6)
                if lane["kind"] in ("car", "truck"):
                    col = COL_TRUCK if lane["kind"] == "truck" else e["color"]
                    pygame.draw.rect(s, col, r, border_radius=5)
                    pygame.draw.rect(s, tuple(int(v * 0.6) for v in col), r, 2,
                                     border_radius=5)
                    # Fenster in Fahrtrichtung
                    wx = r.right - c // 3 if lane["speed"] > 0 else r.x + 4
                    pygame.draw.rect(s, (200, 225, 245),
                                     (wx, r.y + 3, c // 4, r.h - 6),
                                     border_radius=3)
                elif lane["kind"] == "log":
                    pygame.draw.rect(s, COL_LOG, r, border_radius=c // 3)
                    pygame.draw.rect(s, COL_LOG_DARK, r, 2, border_radius=c // 3)
                    for lx in range(r.x + c // 2, r.right - c // 3, c):
                        pygame.draw.line(s, COL_LOG_DARK, (lx, r.y + 4),
                                         (lx, r.bottom - 4))
                else:   # turtle
                    st = self._turtle_state(e)
                    if st == "under":
                        continue
                    col = COL_TURTLE
                    if st == "blink" and int(self.anim_t * 8) % 2 == 0:
                        col = COL_TURTLE_DARK
                    if st == "rise":
                        col = COL_TURTLE_DARK
                    n = max(1, int(e["w"]) // c)
                    for i in range(n):
                        cxx = int(e["x"] + (i + 0.5) * c)
                        pygame.draw.circle(s, col, (cxx, y + c // 2), c // 2 - 3)
                        pygame.draw.circle(s, COL_TURTLE_DARK,
                                           (cxx, y + c // 2), c // 2 - 3, 2)

    def _draw_bay_extras(self, s):
        c = self.cell
        y0 = self._row_y(0)
        if self.fly is not None:
            cx = int((self.fly["bay"] + 0.5) * self.width / 5)
            fy = y0 + c // 2
            pygame.draw.circle(s, COL_FLY, (cx, fy), max(3, c // 6))
            off = max(2, c // 8)
            for sx in (-off, off):
                pygame.draw.circle(s, (255, 240, 200), (cx + sx, fy - off),
                                   max(2, c // 9), 1)
        if self.croc is not None:
            cx = int((self.croc["bay"] + 0.5) * self.width / 5)
            r = pygame.Rect(cx - int(c * 0.55), y0 + c // 4, int(c * 1.1), c // 2)
            pygame.draw.rect(s, COL_CROC, r, border_radius=4)
            # Aufgerissenes Maul + Auge
            pygame.draw.polygon(s, (240, 240, 250),
                                [(r.x + 4, r.centery - 2), (r.x + c // 3, r.y + 2),
                                 (r.x + c // 3, r.bottom - 2)])
            pygame.draw.circle(s, (20, 24, 34), (r.right - c // 4, r.y + 4), 2)

    def _draw_frog(self, s, x, y, small=False):
        c = self.cell
        r = c // 2 - (4 if small else 2)
        cx, cy = int(x), int(y + c // 2)
        # Beine
        if not small:
            for sx in (-1, 1):
                pygame.draw.line(s, COL_FROG_DARK, (cx + sx * r, cy),
                                 (cx + sx * (r + 3), cy + r), 3)
        pygame.draw.ellipse(s, COL_FROG, (cx - r, cy - r, 2 * r, 2 * r))
        pygame.draw.ellipse(s, COL_FROG_DARK, (cx - r, cy - r, 2 * r, 2 * r), 2)
        # Augen
        er = max(2, r // 3)
        for sx in (-1, 1):
            pygame.draw.circle(s, (240, 240, 250),
                               (cx + sx * r // 2, cy - r + er // 2), er)
            pygame.draw.circle(s, (20, 24, 34),
                               (cx + sx * r // 2, cy - r + er // 2), max(1, er // 2))

    def _draw_death(self, s):
        """Splat/Platsch-Animation an der Todesposition."""
        c = self.cell
        cx = int(self.frog_x)
        cy = int(self._row_y(self.frog_row) + c // 2)
        f = 1.0 - self.death_t          # 0..1
        n = 8
        for i in range(n):
            a = i / n * math.tau
            d = f * c * 0.9
            px, py = cx + math.cos(a) * d, cy + math.sin(a) * d
            pygame.draw.circle(s, COL_FROG_DARK, (int(px), int(py)),
                               max(1, int(4 * (1 - f))))
        img = self._small.render("+", True, ui.RED)
        s.blit(img, img.get_rect(center=(cx, cy)))

    def _draw_hud(self, s):
        pygame.draw.rect(s, ui.PANEL, (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h),
                         (self.width, self.hud_h))
        cy = self.hud_h // 2
        img = self._small.render(t("common.points", score=self.score), True,
                                 ui.TEXT)
        s.blit(img, img.get_rect(midleft=(12, cy)))
        lvl = self._small.render(t("frog.level", n=self.level), True,
                                 self.accent)
        s.blit(lvl, lvl.get_rect(center=(self.width // 2, cy)))
        # Leben als Mini-Frösche rechts
        for i in range(self.lives):
            fx = self.width - 20 - i * (self.cell // 2 + 8)
            self._draw_frog(s, fx, cy - self.cell // 2, small=True)

        # Zeitbalken unten
        frac = max(0.0, self.time_left / self._diff()["timer"])
        bar = pygame.Rect(0, self.height - self.bar_h, int(self.width * frac),
                          self.bar_h)
        pygame.draw.rect(s, ui.GREEN if frac > 0.25 else ui.RED, bar)

    def _draw_banner(self, s, text):
        img = self._huge.render(text, True, self.accent)
        bg = img.get_rect(center=(self.width // 2, self.height // 2)).inflate(48, 26)
        ov = pygame.Surface(bg.size, pygame.SRCALPHA)
        ov.fill(_rgba(ui.PANEL, 225))
        s.blit(ov, bg)
        pygame.draw.rect(s, self.accent, bg, 2, border_radius=12)
        s.blit(img, img.get_rect(center=bg.center))

    def _draw_gameover(self, s):
        s.blit(self._dim_surface(), (0, 0))
        cx, cy = self.width // 2, self.height // 2
        rows = [
            self._huge.render(t("common.game_over"), True, ui.RED),
            self.font.render(t("common.points", score=self.score), True,
                             ui.TEXT),
            self._small.render(t("frog.retry"), True, ui.TEXT_DIM),
        ]
        gap = 10
        total = sum(r.get_height() for r in rows) + gap * (len(rows) - 1)
        pw = min(self.width - 30,
                 max(340, max(r.get_width() for r in rows) + 64))
        panel = pygame.Rect(0, 0, pw, total + 48)
        panel.center = (cx, cy)
        ov = pygame.Surface(panel.size, pygame.SRCALPHA)
        ov.fill(_rgba(ui.PANEL, 235))
        s.blit(ov, panel.topleft)
        pygame.draw.rect(s, self.accent, panel, 2, border_radius=14)
        yy = panel.y + 24
        for r in rows:
            s.blit(r, r.get_rect(midtop=(cx, yy)))
            yy += r.get_height() + gap

    # ----- Setup zeichnen -----------------------------------------------
    def _draw_setup(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        title = self._huge.render(self.name.upper(), True, self.accent)
        s.blit(title, title.get_rect(center=(self.width // 2,
                                             int(self.height * 0.14))))
        sub = self._small.render(t("frog.subtitle"), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(self.width // 2,
                                         int(self.height * 0.20))))
        for i, r in enumerate(self.diff_rects):
            on = (i == self.diff_idx)
            pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, r,
                             border_radius=10)
            pygame.draw.rect(s, self.accent if on else ui.BORDER, r,
                             2 if on else 1, border_radius=10)
            name = self.font.render(t("frog.diff." + DIFF_KEYS[i]), True,
                                    ui.TEXT if on else ui.TEXT_DIM)
            s.blit(name, name.get_rect(midleft=(r.x + 18, r.y + 18)))
            desc = self._tiny.render(t("frog.diff_desc." + DIFF_KEYS[i]), True,
                                     ui.TEXT_DIM)
            s.blit(desc, desc.get_rect(midleft=(r.x + 18, r.bottom - 15)))
        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=10)
        pygame.draw.rect(s, self.accent, self.start_rect, 2, border_radius=10)
        st = self.font.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        hint = self._tiny.render(t("frog.setup_hint"), True, ui.TEXT_FAINT)
        s.blit(hint, hint.get_rect(center=(self.width // 2, self.height - 30)))
        ctrl = self._tiny.render(t("frog.hint"), True,
                                 ui.mix(self.accent, ui.TEXT, 0.45))
        s.blit(ctrl, ctrl.get_rect(center=(self.width // 2, self.height - 12)))
