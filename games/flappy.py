# -*- coding: utf-8 -*-
"""
flappy.py
=========
Flappy Bird - Klon mit vielen Extras (Einzelspieler).

Features
--------
- Schwerkraft-Physik: Leertaste / Pfeil hoch / W / Mausklick lässt den Vogel
  flattern; der Vogel neigt sich je nach Steig-/Sinktempo (Rotation).
- Endlose Röhrenpaare mit Lücke; +1 Punkt pro passierter Röhre.
- MÜNZEN (Bonuspunkte) und SCHILD-Power-up (überlebt eine Kollision) erscheinen
  gelegentlich in den Lücken.
- TAG/NACHT-THEMEN: Der Himmel wechselt mit steigender Punktzahl (Farbpaletten),
  mit driftenden Wolken (Parallax) und scrollendem Boden.
- Schwierigkeit (Leicht/Normal/Schwer): Lückengröße, Tempo, Röhrenabstand;
  die Lücke wird mit steigender Punktzahl etwas enger.
- MEDAILLEN nach dem Game Over (Bronze/Silber/Gold/Platin) je nach Punktzahl.
- READY-Screen (Vogel wippt), Crash-Animation mit Kamera-Shake, Partikel,
  Highscore.

Steuerung: Leertaste / Pfeil hoch / W / Klick = Flattern.  Enter = neu, S = Setup.
"""

import math
import random
import pygame

import highscore
import settings as settings_mod
import ui
from game_base import Game, InputEvent
from i18n import t

GRAV = 1550.0
FLAP_V = -430.0
BIRD_X_FRAC = 0.30

PIPE_W_FRAC = 0.13         # Röhrenbreite als Anteil der Breite
CAP_H_FRAC = 0.028         # Höhe der Röhren-"Kappe"

# Schwierigkeit: Lücke (px-Anteil Höhe), Tempo (px/s), Röhrenabstand (px)
DIFFS = [
    dict(key="easy", gap=0.30, speed=150.0, spacing=0.62),
    dict(key="normal", gap=0.25, speed=190.0, spacing=0.55),
    dict(key="hard", gap=0.21, speed=230.0, spacing=0.48),
]

# Tag/Nacht-Paletten: (sky_top, sky_bottom, pipe, pipe_dark, ground, ground_dark)
# Identitätsfarben des Spielfelds - bewusst NICHT aus dem UI-Theme.
THEMES = [
    ((120, 200, 245), (200, 235, 250), (110, 205, 90), (70, 150, 60),
     (222, 200, 120), (180, 155, 90)),
    ((250, 190, 120), (255, 225, 170), (230, 160, 70), (180, 120, 50),
     (210, 180, 120), (170, 140, 90)),
    ((40, 45, 90), (80, 90, 150), (90, 120, 200), (60, 80, 150),
     (60, 60, 100), (40, 40, 75)),
    ((150, 120, 200), (220, 190, 235), (200, 120, 190), (150, 80, 140),
     (200, 170, 200), (160, 130, 165)),
]

MEDALS = [(70, "platin", (225, 235, 245)), (45, "gold", (245, 205, 70)),
          (25, "silber", (200, 205, 215)), (10, "bronze", (205, 140, 85))]

# Weitere Identitätsfarben (Spielobjekte, nicht Theme-abhängig)
COL_COIN = (250, 205, 70)      # Münzen-Gold
COL_SHIELD = (120, 220, 255)   # Schild-Blau

SETUP, READY, PLAY, DYING, GAMEOVER = "setup", "ready", "play", "dying", "gameover"


class FlappyGame(Game):
    name = "Flappy Bird"
    highscore_key = "flappy"
    supports_multiplayer = False

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        fb = self.settings.get("flappy", {}) if isinstance(self.settings, dict) else {}
        self.diff = max(0, min(2, int(fb.get("difficulty", 1))))

        self.highscore = highscore.load_highscores().get(self.highscore_key, 0)
        self.anim_t = 0.0

        self._sky_cache = None
        self._ui_bg_cache = None
        self._dim_cache = None
        self._apply_layout()

        self.clouds = [self._new_cloud(random.uniform(0, self.width))
                       for _ in range(5)]

        self._new_game()
        self.state = SETUP

    def _make_fonts(self):
        """Schriftgrößen aus der Fensterhöhe ableiten (Theme-Schrift)."""
        h = self.height
        self.font = ui.font(max(18, min(26, h // 26)))
        self.big_font = ui.font(max(32, min(52, h // 12)), bold=True)
        self._small = ui.font(max(14, min(20, h // 32)))
        self._tiny = ui.font(max(12, min(16, h // 42)))
        self._huge = ui.font(max(30, h // 9), bold=True)

    def _apply_layout(self):
        """Layout-Größen aus width/height berechnen (auch nach Resize)."""
        self._make_fonts()
        self.ground_h = int(self.height * 0.12)
        self.bird_x = self.width * BIRD_X_FRAC
        self.bird_r = max(9, int(self.height * 0.028))
        self.pipe_w = int(self.width * PIPE_W_FRAC)
        self._build_setup_layout()

    def on_surface_changed(self):
        """Nach Auflösungswechsel: Schriften, Layout und Caches neu aufbauen."""
        self._apply_layout()
        self._sky_cache = None
        self._ui_bg_cache = None
        self._dim_cache = None
        # Vogel im gültigen Bereich halten
        self.bird_y = max(self.bird_r,
                          min(self.bird_y, self.height - self.ground_h - self.bird_r))

    def _new_cloud(self, x=None):
        return dict(x=self.width if x is None else x,
                    y=random.uniform(self.height * 0.08, self.height * 0.5),
                    s=random.uniform(0.6, 1.4),
                    v=random.uniform(12, 30))

    def _new_game(self):
        self.score = 0
        self.coins = 0
        self.game_over = False
        self.bird_y = self.height * 0.45
        self.bird_v = 0.0
        self.bird_a = 0.0
        self.pipes = []
        self.pickups = []          # dicts: x,y,kind('coin'/'shield'),taken
        self.particles = []
        self.shield = False
        self.shake = 0.0
        self.ground_x = 0.0
        self.dist = 0.0
        self.wing_t = 0.0
        self.new_best = False
        self._over_t = 0.0
        self.state = READY

    @property
    def theme(self):
        return THEMES[(self.score // 10) % len(THEMES)]

    def _diff(self):
        return DIFFS[self.diff]

    def _gap(self):
        base = self._diff()["gap"] * self.height
        return max(self.bird_r * 4.5, base - self.score * 1.2)

    def _speed(self):
        return self._diff()["speed"] + self.score * 1.5

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(440, self.width - 60)
        ph = max(56, min(72, int(self.height * 0.14)))
        y0 = max(140, int(self.height * 0.34))
        self.diff_panel = pygame.Rect(cx - bw // 2, y0, bw, ph)
        self.diff_left = pygame.Rect(self.diff_panel.left, y0, 44, ph)
        self.diff_right = pygame.Rect(self.diff_panel.right - 44, y0, 44, ph)
        self.start_rect = pygame.Rect(cx - 95, self.diff_panel.bottom + 26, 190, 52)

    def _save(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("flappy", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _cycle_diff(self, step):
        self.diff = (self.diff + step) % len(DIFFS)
        self._save("difficulty", self.diff)
        self.play_sound("click")

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Left", "a", "A"):
                self._cycle_diff(-1)
            elif event.key in ("Right", "d", "D"):
                self._cycle_diff(+1)
            elif event.key in ("Return", "space"):
                self._new_game()
                self.play_sound("click")
        elif event.kind == InputEvent.MOUSEDOWN:
            p = event.pos
            if self.diff_left.collidepoint(p):
                self._cycle_diff(-1)
            elif self.diff_right.collidepoint(p) or self.diff_panel.collidepoint(p):
                self._cycle_diff(+1)
            elif self.start_rect.collidepoint(p):
                self._new_game()
                self.play_sound("click")

    # ===================================================== Eingabe
    def _is_flap(self, key):
        return (self.is_action(key, "up") or key in ("Up", "space", "w", "W"))

    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.state == GAMEOVER:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._new_game()
                    self.play_sound("click")
                elif event.key in ("s", "S"):
                    self.state = SETUP
                    self.play_sound("click")
            elif event.kind == InputEvent.MOUSEDOWN and \
                    self.anim_t - self._over_t > 0.5:
                # Klick startet neu - aber erst nach kurzer Sperre, damit
                # hektisches Weiterklicken den Ergebnis-Screen nicht überspringt.
                self._new_game()
                self.play_sound("click")
            return
        flap = (event.kind == InputEvent.MOUSEDOWN) or \
               (event.kind == InputEvent.KEYDOWN and self._is_flap(event.key))
        if flap:
            if self.state == READY:
                self.state = PLAY
            if self.state == PLAY:
                self._flap()

    def _flap(self):
        self.bird_v = FLAP_V
        self.wing_t = 0.0
        self.play_sound("move")
        for _ in range(4):
            self.particles.append([self.bird_x - self.bird_r, self.bird_y,
                                   random.uniform(-60, -20), random.uniform(-20, 40),
                                   0.35, (255, 255, 255)])

    # ===================================================== Spiellogik
    def update(self, dt):
        self.anim_t += dt
        self.wing_t += dt
        self._update_clouds(dt)
        self._update_particles(dt)
        if self.shake > 0:
            self.shake = max(0.0, self.shake - dt * 2.0)
        if self.state in (SETUP, GAMEOVER):
            return
        if self.state == READY:
            self.bird_y = self.height * 0.45 + math.sin(self.anim_t * 3) * 12
            return
        if self.state == DYING:
            self.bird_v += GRAV * dt
            self.bird_y += self.bird_v * dt
            self.bird_a = max(-1.4, self.bird_a - dt * 4)
            if self.bird_y >= self.height - self.ground_h - self.bird_r:
                self.bird_y = self.height - self.ground_h - self.bird_r
                self.state = GAMEOVER
                self._finish()
            return

        # PLAY
        self.dist += self._speed() * dt
        self.ground_x = (self.ground_x - self._speed() * dt) % (self.width)
        self.bird_v += GRAV * dt
        self.bird_y += self.bird_v * dt
        self.bird_a = max(-1.3, min(1.4, self.bird_v / 520.0))
        if self.bird_y < self.bird_r:          # Decke: abprallen
            self.bird_y = self.bird_r
            self.bird_v = max(self.bird_v, 40.0)

        self._spawn_pipes()
        self._move_pipes(dt)
        self._collisions()

    def _spawn_pipes(self):
        spacing = self._diff()["spacing"] * self.width
        if not self.pipes or self.pipes[-1]["x"] < self.width - spacing:
            gap = self._gap()
            top = self.height - self.ground_h
            margin = self.height * 0.09
            gy = random.uniform(margin + gap / 2, top - margin - gap / 2)
            pipe = dict(x=float(self.width + self.pipe_w), gy=gy, gap=gap,
                        passed=False)
            self.pipes.append(pipe)
            # gelegentlich Münze oder Schild in die Lücke
            n = len(self.pipes)
            if n % 3 == 0:
                self.pickups.append(dict(x=pipe["x"] + self.pipe_w / 2, y=gy,
                                         kind="coin", taken=False))
            elif n % 7 == 0 and not self.shield:
                self.pickups.append(dict(x=pipe["x"] + self.pipe_w / 2, y=gy,
                                         kind="shield", taken=False))

    def _move_pipes(self, dt):
        v = self._speed() * dt
        for p in self.pipes:
            p["x"] -= v
            if not p["passed"] and p["x"] + self.pipe_w / 2 < self.bird_x:
                p["passed"] = True
                self.score += 1
                self.play_sound("point")
        self.pipes = [p for p in self.pipes if p["x"] + self.pipe_w > -4]
        for pk in self.pickups:
            pk["x"] -= v
        self.pickups = [pk for pk in self.pickups
                        if pk["x"] > -20 and not pk["taken"]]

    def _collisions(self):
        bx, by, r = self.bird_x, self.bird_y, self.bird_r
        # Boden
        if by + r >= self.height - self.ground_h:
            self._hit()
            return
        # Röhren
        for p in self.pipes:
            top_r = pygame.Rect(p["x"], 0, self.pipe_w, p["gy"] - p["gap"] / 2)
            bot_r = pygame.Rect(p["x"], p["gy"] + p["gap"] / 2, self.pipe_w,
                                self.height)
            if self._circle_rect(bx, by, r, top_r) or \
                    self._circle_rect(bx, by, r, bot_r):
                self._hit()
                return
        # Pickups
        for pk in self.pickups:
            if not pk["taken"] and math.hypot(bx - pk["x"], by - pk["y"]) < r + 12:
                pk["taken"] = True
                if pk["kind"] == "coin":
                    self.coins += 1
                    self.score += 2
                    self.play_sound("eat")
                else:
                    self.shield = True
                    self.play_sound("powerup")
                self._sparkle(pk["x"], pk["y"], COL_COIN)

    @staticmethod
    def _circle_rect(cx, cy, r, rect):
        nx = max(rect.left, min(cx, rect.right))
        ny = max(rect.top, min(cy, rect.bottom))
        return (cx - nx) ** 2 + (cy - ny) ** 2 < r * r

    def _hit(self):
        if self.shield:
            self.shield = False
            self.bird_v = FLAP_V * 0.8
            self.shake = 0.4
            self.play_sound("hit")
            self._sparkle(self.bird_x, self.bird_y, COL_SHIELD)
            return
        self.state = DYING
        self.bird_v = FLAP_V * 0.5
        self.shake = 0.7
        self.play_sound("explode")
        self.rumble(250)
        self._sparkle(self.bird_x, self.bird_y, (255, 200, 90), 16)

    def _finish(self):
        self.game_over = True
        self.new_best = self.score > self.highscore
        self.highscore = max(self.highscore, self.score)
        self._over_t = self.anim_t
        self.play_sound("gameover")

    def _medal(self):
        for thr, key, col in MEDALS:
            if self.score >= thr:
                return key, col
        return None, None

    # ----- Effekte -------------------------------------------------------
    def _sparkle(self, x, y, col, n=10):
        for _ in range(n):
            a = random.uniform(0, math.tau)
            sp = random.uniform(40, 180)
            self.particles.append([x, y, math.cos(a) * sp, math.sin(a) * sp,
                                   random.uniform(0.25, 0.55), col])

    def _update_particles(self, dt):
        rest = []
        for p in self.particles:
            p[0] += p[2] * dt
            p[1] += p[3] * dt
            p[3] += 300 * dt
            p[4] -= dt
            if p[4] > 0:
                rest.append(p)
        self.particles = rest

    def _update_clouds(self, dt):
        f = 0.4 if self.state in (SETUP, READY, GAMEOVER) else 1.0
        for c in self.clouds:
            c["x"] -= c["v"] * c["s"] * f * dt
            if c["x"] < -80:
                c.update(self._new_cloud())
                c["x"] = self.width + 60

    # ----- Theme-UI-Helfer ----------------------------------------------
    def _ui_bg(self):
        """Gecachter Theme-Hintergrund für den Setup-Screen."""
        key = (self.width, self.height, ui.BG_TOP, ui.BG_BOTTOM)
        if self._ui_bg_cache is None or self._ui_bg_cache[0] != key:
            surf = pygame.Surface((self.width, self.height))
            ui.draw_background(surf, self.width, self.height)
            self._ui_bg_cache = (key, surf)
        return self._ui_bg_cache[1]

    def _dim(self):
        """Gecachte Abdunkel-Fläche fürs Game-Over (kein per-Frame-Fill)."""
        key = (self.width, self.height)
        if self._dim_cache is None or self._dim_cache[0] != key:
            surf = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
            surf.fill((10, 12, 20, 140))
            self._dim_cache = (key, surf)
        return self._dim_cache[1]

    def _panel(self, s, rect, alpha=235, border=2):
        """Halbtransparentes Theme-Panel mit Akzent-Rahmen."""
        surf = pygame.Surface(rect.size, pygame.SRCALPHA)
        pygame.draw.rect(surf, (*ui.PANEL, alpha), surf.get_rect(),
                         border_radius=12)
        s.blit(surf, rect.topleft)
        pygame.draw.rect(s, self.accent, rect, border, border_radius=12)

    def _chip(self, s, text, font, center, color=None):
        """Kleine Info-Plakette im Theme-Stil (über dem Spielfeld lesbar)."""
        img = font.render(text, True, color or ui.TEXT)
        box = img.get_rect(center=center).inflate(24, 12)
        self._panel(s, box, alpha=200, border=1)
        s.blit(img, img.get_rect(center=box.center))

    # ===================================================== Zeichnen
    def draw(self):
        if self.state == SETUP:
            self._draw_setup()
            return
        s = self.surface
        ox = oy = 0
        if self.shake > 0:
            amp = 8 * self.shake
            ox = random.uniform(-amp, amp)
            oy = random.uniform(-amp, amp)
        self._draw_sky(s)
        self._draw_clouds(s, ox * 0.3, oy * 0.3)
        for p in self.pipes:
            self._draw_pipe(s, p, ox, oy)
        for pk in self.pickups:
            self._draw_pickup(s, pk, ox, oy)
        self._draw_ground(s, ox)
        for p in self.particles:
            a = max(0, min(255, int(255 * p[4] / 0.55)))
            surf = pygame.Surface((5, 5), pygame.SRCALPHA)
            pygame.draw.circle(surf, (*p[5], a), (2, 2), 2)
            s.blit(surf, (p[0] - 2 + ox, p[1] - 2 + oy))
        self._draw_bird(s, ox, oy)
        self._draw_hud(s)

    def _draw_sky(self, s):
        top, bot = self.theme[0], self.theme[1]
        key = (self.width, self.height, top, bot)
        if self._sky_cache is None or self._sky_cache[0] != key:
            surf = pygame.Surface((self.width, self.height))
            for y in range(self.height):
                tt = y / max(1, self.height)
                c = [int(a + (b - a) * tt) for a, b in zip(top, bot)]
                pygame.draw.line(surf, c, (0, y), (self.width, y))
            self._sky_cache = (key, surf)
        s.blit(self._sky_cache[1], (0, 0))

    def _draw_clouds(self, s, ox, oy):
        for c in self.clouds:
            x, y, sc = c["x"] + ox, c["y"] + oy, c["s"]
            col = (255, 255, 255)
            for dx, dy, rr in ((0, 0, 26), (22, 6, 20), (-22, 6, 20), (0, 10, 22)):
                surf = pygame.Surface((int(rr * sc * 2),) * 2, pygame.SRCALPHA)
                pygame.draw.circle(surf, (*col, 150), (int(rr * sc),) * 2,
                                   int(rr * sc))
                s.blit(surf, (x + dx * sc - rr * sc, y + dy * sc - rr * sc))

    def _draw_pipe(self, s, p, ox, oy):
        pipe, dark = self.theme[2], self.theme[3]
        w = self.pipe_w
        cap = int(self.height * CAP_H_FRAC) + 4
        x = p["x"] + ox
        top_h = p["gy"] - p["gap"] / 2
        bot_y = p["gy"] + p["gap"] / 2
        # oberes Rohr
        pygame.draw.rect(s, pipe, (x, oy, w, top_h - cap))
        pygame.draw.rect(s, pipe, (x - 4, top_h - cap + oy, w + 8, cap),
                         border_radius=4)
        pygame.draw.rect(s, dark, (x + w - 8, oy, 6, top_h))
        # unteres Rohr
        gh = self.height - self.ground_h - bot_y
        pygame.draw.rect(s, pipe, (x, bot_y + cap + oy, w, gh))
        pygame.draw.rect(s, pipe, (x - 4, bot_y + oy, w + 8, cap), border_radius=4)
        pygame.draw.rect(s, dark, (x + w - 8, bot_y + oy, 6, gh + cap))
        pygame.draw.rect(s, (255, 255, 255), (x + 4, oy, 4, top_h - cap))

    def _draw_pickup(self, s, pk, ox, oy):
        x, y = int(pk["x"] + ox), int(pk["y"] + oy)
        if pk["kind"] == "coin":
            w = int(10 + 5 * math.sin(self.anim_t * 6))
            pygame.draw.ellipse(s, COL_COIN, (x - w, y - 12, 2 * w, 24))
            pygame.draw.ellipse(s, (255, 240, 160), (x - w + 2, y - 9, 2 * w - 4, 18), 2)
        else:
            surf = pygame.Surface((36, 36), pygame.SRCALPHA)
            pygame.draw.circle(surf, (*COL_SHIELD, 90), (18, 18), 17)
            pygame.draw.circle(surf, COL_SHIELD, (18, 18), 17, 2)
            s.blit(surf, (x - 18, y - 18))
            st = self._tiny.render("S", True, (240, 250, 255))
            s.blit(st, st.get_rect(center=(x, y)))

    def _draw_ground(self, s, ox):
        gcol, gdark = self.theme[4], self.theme[5]
        gy = self.height - self.ground_h
        pygame.draw.rect(s, gcol, (0, gy, self.width, self.ground_h))
        pygame.draw.rect(s, gdark, (0, gy, self.width, 5))
        step = 26
        off = int(self.ground_x) % step
        for x in range(-step, self.width + step, step):
            pygame.draw.line(s, gdark, (x - off, gy + 8),
                             (x - off + 12, self.height), 2)

    def _draw_bird(self, s, ox, oy):
        r = self.bird_r
        wing = math.sin(min(self.wing_t, 0.3) * 30) * r * 0.5 \
            if self.state in (PLAY, READY) else 0
        surf = self._bird_surface(r, wing)
        ang = -math.degrees(self.bird_a)
        rot = pygame.transform.rotozoom(surf, ang, 1.0)
        s.blit(rot, rot.get_rect(center=(self.bird_x + ox, self.bird_y + oy)))
        if self.shield:
            bub = pygame.Surface((int(r * 4), int(r * 4)), pygame.SRCALPHA)
            pygame.draw.circle(bub, (*COL_SHIELD, 70), (int(r * 2),) * 2, int(r * 1.9))
            pygame.draw.circle(bub, (150, 235, 255), (int(r * 2),) * 2,
                               int(r * 1.9), 2)
            s.blit(bub, (self.bird_x + ox - r * 2, self.bird_y + oy - r * 2))

    def _bird_surface(self, r, wing):
        d = int(r * 3)
        surf = pygame.Surface((d, d), pygame.SRCALPHA)
        cx, cy = d // 2, d // 2
        body = (250, 205, 60)
        pygame.draw.polygon(surf, (235, 180, 50),
                            [(cx - r, cy), (cx - r * 1.5, cy - r * 0.5),
                             (cx - r * 1.5, cy + r * 0.5)])
        pygame.draw.ellipse(surf, body, (cx - r, cy - r * 0.85, r * 2, r * 1.7))
        pygame.draw.ellipse(surf, (255, 240, 200),
                            (cx - r * 0.4, cy - r * 0.1, r * 1.1, r * 0.9))
        pygame.draw.ellipse(surf, (230, 180, 60),
                            (cx - r * 0.35, cy - r * 0.15 + wing, r * 0.9, r * 0.6))
        pygame.draw.circle(surf, (255, 255, 255),
                           (int(cx + r * 0.45), int(cy - r * 0.32)), int(r * 0.32))
        pygame.draw.circle(surf, (25, 25, 35),
                           (int(cx + r * 0.55), int(cy - r * 0.32)), int(r * 0.15))
        pygame.draw.polygon(surf, (250, 150, 40),
                            [(cx + r * 0.75, cy - r * 0.1), (cx + r * 1.35, cy),
                             (cx + r * 0.75, cy + r * 0.25)])
        return surf

    # ----- HUD / Overlays -----------------------------------------------
    def _draw_hud(self, s):
        if self.state in (PLAY, DYING):
            # Punktzahl mit Kontur - bleibt auf jedem Himmel lesbar
            img = self._huge.render(str(self.score), True, (245, 245, 250))
            outline = self._huge.render(str(self.score), True, (30, 30, 40))
            r = img.get_rect(center=(self.width // 2, int(self.height * 0.14)))
            for dx, dy in ((-2, 0), (2, 0), (0, -2), (0, 2)):
                s.blit(outline, r.move(dx, dy))
            s.blit(img, r)
        if self.state == READY:
            self._chip(s, t("fb.best", hs=self.highscore), self._small,
                       (self.width // 2, int(self.height * 0.1)), ui.TEXT_DIM)
            hint_col = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.4, 0.0, 1.0))
            self._chip(s, t("fb.tap_hint"), self._small,
                       (self.width // 2, int(self.height * 0.38)), hint_col)
        if self.state == GAMEOVER:
            self._draw_gameover(s)

    def _draw_gameover(self, s):
        s.blit(self._dim(), (0, 0))
        cx = self.width // 2
        img = self._huge.render(t("common.game_over"), True, self.accent)
        s.blit(img, img.get_rect(center=(cx, int(self.height * 0.18))))

        lh = self._small.get_height() + 8
        pw = min(360, self.width - 40)
        ph = lh * 3 + 40
        panel = pygame.Rect(cx - pw // 2, int(self.height * 0.32), pw, ph)
        self._panel(s, panel)
        x, y = panel.x + 20, panel.y + 18
        s.blit(self._small.render(t("common.points", score=self.score),
                                  True, ui.TEXT), (x, y))
        y += lh
        s.blit(self._small.render(t("fb.coins", n=self.coins), True, ui.GOLD),
               (x, y))
        y += lh
        best_col = ui.GOLD if self.new_best else ui.TEXT_DIM
        s.blit(self._small.render(t("fb.best", hs=self.highscore), True, best_col),
               (x, y))

        # Medaille rechts im Panel (Identitätsfarben)
        key, col = self._medal()
        if key:
            mr = max(24, min(36, panel.h // 4))
            mcx, mcy = panel.right - mr - 24, panel.centery - 8
            pygame.draw.circle(s, col, (mcx, mcy), mr)
            pygame.draw.circle(s, tuple(int(c * 0.7) for c in col), (mcx, mcy), mr, 3)
            self._star(s, mcx, mcy, int(mr * 0.55), (255, 255, 255))
            lab = self._tiny.render(t("fb.medal." + key), True, col)
            s.blit(lab, lab.get_rect(center=(mcx, mcy + mr + 12)))

        if self.new_best:
            rec_col = ui.mix(ui.GOLD, ui.TEXT, ui.pulse(3.0, 0.0, 0.5))
            rec = self._small.render(t("trex.new_record"), True, rec_col)
            s.blit(rec, rec.get_rect(center=(cx, panel.top - 18)))

        hint_col = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.4, 0.0, 1.0))
        hint = self._small.render(t("fb.restart_hint"), True, hint_col)
        s.blit(hint, hint.get_rect(center=(cx, panel.bottom + 26)))

    def _star(self, s, cx, cy, r, col):
        pts = []
        for i in range(10):
            rr = r if i % 2 == 0 else r * 0.45
            a = -math.pi / 2 + i * math.pi / 5
            pts.append((cx + math.cos(a) * rr, cy + math.sin(a) * rr))
        pygame.draw.polygon(s, col, pts)

    def _center(self, s, text, font, col, dy=0):
        img = font.render(text, True, col)
        s.blit(img, img.get_rect(center=(self.width // 2, self.height // 2 + int(dy))))

    # ----- Setup zeichnen -----------------------------------------------
    def _draw_setup(self):
        s = self.surface
        s.blit(self._ui_bg(), (0, 0))
        self._draw_clouds(s, 0, 0)
        ui.draw_title(s, self.width, "FLAPPY BIRD",
                      subtitle=t("snake.singleplayer"), accent=self.accent)

        d = DIFFS[self.diff]
        self._panel(s, self.diff_panel)
        name = self.font.render(
            t("fb.difficulty") + ":  " + t("fb.diff." + d["key"]), True, ui.TEXT)
        s.blit(name, name.get_rect(center=(self.diff_panel.centerx,
                                           self.diff_panel.top
                                           + int(self.diff_panel.h * 0.36))))
        note = self._tiny.render(t("fb.diff_note"), True, ui.TEXT_DIM)
        s.blit(note, note.get_rect(center=(self.diff_panel.centerx,
                                           self.diff_panel.top
                                           + int(self.diff_panel.h * 0.72))))
        arr_col = ui.mix(self.accent, ui.TEXT, ui.pulse(3.0, 0.0, 0.3))
        for rect, sym in ((self.diff_left, "<"), (self.diff_right, ">")):
            arr = self.big_font.render(sym, True, arr_col)
            s.blit(arr, arr.get_rect(center=rect.center))

        ui.draw_button(s, self.start_rect, t("common.start"), self.font,
                       selected=True, accent=self.accent)

        ctrl = self._tiny.render(t("fb.controls_hint"), True, ui.GREEN)
        s.blit(ctrl, ctrl.get_rect(center=(self.width // 2,
                                           self.start_rect.bottom + 24)))
        ui.draw_footer(s, self.width, self.height, t("fb.setup_hint"))
