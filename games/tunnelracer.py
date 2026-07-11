# -*- coding: utf-8 -*-
"""
tunnelracer.py
==============
Tunnel Racer - Neon-3D-Röhrenflug (Einzelspieler).

- Zwei Modi (Vorspiel-Screen): ENDLOS (immer schneller, Highscore) und
  LEVEL (30 feste, seed-generierte Strecken mit Ziel; Fortschritt wird
  gespeichert und in der Levelauswahl abgehakt).
- Das Schiff fliegt durch eine kurvige Quadrat-Röhre; Hindernissen
  ausweichen (Balken, Blöcke, Ring-Blenden zum Durchfädeln), Münzen sammeln.
- Steuerung: Tasten (Standard, Pfeile/WASD in beide Achsen) oder Maus
  (Pointer-Capture, direkte Position) - im Setup umschaltbar, gespeichert.
- Motion Blur (0-80 %) wie im Aim Trainer, einstellbar und gespeichert.

3D-Technik von aimtrainer.py übernommen (Kamera-Basis, Projektion,
Near-Clip, Nebel, Billboards) - reine Software, ohne Assets.
"""

import math
import random

import pygame

import settings as settings_mod
import store
from game_base import Game, InputEvent
from i18n import t

NEAR = 0.12
FOV_MUL = 1.04
H = 3.0                    # halbe Röhrenbreite
SHIP_AHEAD = 4.5           # Schiff fliegt so weit vor der Kamera
SHIP_W, SHIP_H = 0.45, 0.35
VIEW_DIST = 120.0
LEVELS = 30

COL_BG_TOP = (8, 6, 24)
COL_BG_BOT = (24, 10, 48)
COL_RING = (90, 240, 255)
COL_RING_HI = (255, 80, 220)
COL_RAIL = (150, 90, 255)
COL_OBST = (200, 40, 120)
COL_OBST_EDGE = (255, 120, 190)
COL_COIN = (255, 210, 80)
COL_SHIP = (53, 226, 255)   # = Sidebar-Farbe #35e2ff
COL_TEXT = (225, 228, 238)
COL_DIM = (150, 158, 178)
COL_BTN = (32, 38, 54)
COL_BTN_ON = (36, 84, 96)
COL_BTN_BORDER = (74, 84, 116)

SETUP, READY, PLAY, CRASH, FINISH = "setup", "ready", "play", "crash", "finish"


def _level_def(i):
    """Streckendaten für Level i (1-basiert)."""
    return dict(
        length=900 + i * 140,
        base=min(55.0, 24.0 + i * 1.1),
        gap=max(11.0, 24.0 - i * 0.45),
        ring_p=min(0.35, 0.08 + i * 0.01),
        amp=min(2.4, 0.5 + i * 0.07),
    )


class TunnelRacerGame(Game):
    name = "Tunnel Racer"
    highscore_key = "tunnel"
    supports_multiplayer = False

    MODES = [("endless", "tun.mode.endless"), ("levels", "tun.mode.levels")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        tn = self.settings.get("tunnel", {}) if isinstance(self.settings, dict) else {}
        self.control = tn.get("control", "keys")
        if self.control not in ("keys", "mouse"):
            self.control = "keys"
        try:
            self.blur = max(0.0, min(0.8, float(tn.get("blur", 0.35))))
        except (TypeError, ValueError):
            self.blur = 0.35
        try:
            self.cursor = max(1, min(LEVELS, int(tn.get("last_level", 1))))
        except (TypeError, ValueError):
            self.cursor = 1

        self._small = pygame.font.SysFont("consolas", 16)
        self._tiny = pygame.font.SysFont("consolas", 13)
        self._big = pygame.font.SysFont("consolas", 22, bold=True)
        self._huge = pygame.font.SysFont("consolas", max(26, self.height // 11),
                                         bold=True)
        self._sky_cache = None
        self._prev_frame = None
        self.capture_mouse = False
        self._load_solved()
        self.level = 1
        self._build_setup_layout()
        self.state = SETUP

    def on_surface_changed(self):
        self._huge = pygame.font.SysFont("consolas", max(26, self.height // 11),
                                         bold=True)
        self._sky_cache = None
        self._prev_frame = None
        self._build_setup_layout()

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("tunnel", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _load_solved(self):
        data = store.load_section("tunnel")
        lst = data.get("solved", [])
        if isinstance(lst, list):
            self.solved = sorted({int(v) for v in lst
                                  if isinstance(v, int) and 1 <= v <= LEVELS})
        else:
            self.solved = []

    def _mark_solved(self, n):
        if n not in self.solved:
            self.solved = sorted(self.solved + [n])
            store.save_section("tunnel", {"solved": self.solved})

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        y0 = int(self.height * 0.24)
        rx = cx + 24
        self.ctrl_rect = pygame.Rect(rx - 60, y0, 220, 40)
        self.blur_minus = pygame.Rect(rx - 60, y0 + 52, 44, 40)
        self.blur_plus = pygame.Rect(rx + 116, y0 + 52, 44, 40)
        self.blur_box = pygame.Rect(rx - 8, y0 + 52, 116, 40)
        if self.mode == "levels":
            top = y0 + 108
            avail_h = self.height - top - 96
            cell = max(20, min((self.width - 80) // 10, avail_h // 3))
            self.lv_cell = cell
            self.lv_x = cx - cell * 5
            self.lv_y = top
            self._lv_font = pygame.font.SysFont("consolas",
                                                max(10, cell * 2 // 5))
            self.start_rect = pygame.Rect(cx - 95, top + 3 * cell + 14,
                                          190, 46)
        else:
            self.start_rect = pygame.Rect(cx - 95, y0 + 124, 190, 46)

    def _level_at(self, pos):
        x, y = pos
        c = (x - self.lv_x) // self.lv_cell
        r = (y - self.lv_y) // self.lv_cell
        if 0 <= c < 10 and 0 <= r < 3:
            return int(r * 10 + c + 1)
        return None

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("c", "C"):
                self._toggle_control()
            elif k in ("b", "B"):
                self.blur = 0.0 if self.blur >= 0.79 \
                    else round(self.blur + 0.1, 1)
                self._save_setting("blur", self.blur)
                self.play_sound("select")
            elif self.mode == "levels" and k in ("Left", "a", "A"):
                self.cursor = (self.cursor - 2) % LEVELS + 1
                self.play_sound("move")
            elif self.mode == "levels" and k in ("Right", "d", "D"):
                self.cursor = self.cursor % LEVELS + 1
                self.play_sound("move")
            elif self.mode == "levels" and k in ("Up", "w", "W"):
                self.cursor = (self.cursor - 11) % LEVELS + 1
                self.play_sound("move")
            elif self.mode == "levels" and k in ("Down", "s", "S"):
                self.cursor = (self.cursor + 9) % LEVELS + 1
                self.play_sound("move")
            elif k in ("Return", "space"):
                self._start_run(self.cursor if self.mode == "levels" else None)
        elif event.kind == InputEvent.MOUSEDOWN:
            if self.ctrl_rect.collidepoint(event.pos):
                self._toggle_control()
                return
            if self.blur_minus.collidepoint(event.pos):
                self.blur = round(max(0.0, self.blur - 0.1), 1)
                self._save_setting("blur", self.blur)
                self.play_sound("select")
                return
            if self.blur_plus.collidepoint(event.pos):
                self.blur = round(min(0.8, self.blur + 0.1), 1)
                self._save_setting("blur", self.blur)
                self.play_sound("select")
                return
            if self.mode == "levels":
                lv = self._level_at(event.pos)
                if lv is not None:
                    self._start_run(lv)
                    return
            if self.start_rect.collidepoint(event.pos):
                self._start_run(self.cursor if self.mode == "levels" else None)

    def _toggle_control(self):
        self.control = "mouse" if self.control == "keys" else "keys"
        self._save_setting("control", self.control)
        self.play_sound("select")

    # ===================================================== Lauf starten
    def _start_run(self, level):
        self.level = level or 1
        if self.mode == "levels":
            self.cursor = self.level
            self._save_setting("last_level", self.level)
            d = _level_def(self.level)
            self.length = d["length"]
            self.base_speed = d["base"]
            self.gap = d["gap"]
            self.ring_p = d["ring_p"]
            self.amp = d["amp"]
            self.rng = random.Random(7700 + self.level)
        else:
            self.length = None
            self.base_speed = 26.0
            self.gap = 26.0
            self.ring_p = 0.10
            self.amp = 0.0        # rampt hoch
            self.rng = random.Random()

        self.z = 0.0
        self.px = 0.0
        self.py = 0.0
        self.vx = 0.0
        self.vy = 0.0
        self.cam_x = 0.0
        self.cam_y = 0.0
        self.coins = 0
        self.run_score = 0
        self.elapsed = 0.0
        self.obstacles = []       # dict(z, rect(cx,cy,hw,hh) im Querschnitt)
        self.coin_list = []       # dict(z, cx, cy, taken)
        self.next_spawn = 40.0
        self.ready_t = 1.2
        self.flash_t = 0.0
        self._prev_frame = None
        self.keys = set()
        self.state = READY
        self.capture_mouse = False
        self.play_sound("level")

    # ===================================================== Strecke
    def _amp_now(self):
        if self.mode == "levels":
            return self.amp
        return min(2.2, self.z / 1500.0 * 2.2)

    def _center(self, z):
        a1 = self._amp_now()
        a2 = 0.4 * a1
        return (a1 * math.sin(z * 0.021) + a2 * math.sin(z * 0.043 + 1.7),
                0.5 * a1 * math.sin(z * 0.017 + 0.9))

    def _gap_now(self):
        if self.mode == "levels":
            return self.gap
        return 26.0 - (26.0 - 11.0) * min(1.0, self.z / 6000.0)

    def _gen_ahead(self):
        """Hindernisse/Münzen bis z+160 erzeugen, Altes hinter uns löschen."""
        horizon = self.z + 160.0
        while self.next_spawn < horizon:
            if self.length is not None and self.next_spawn > self.length - 30:
                break
            zs = self.next_spawn
            kind_roll = self.rng.random()
            ring_p = self.ring_p if self.mode == "levels" else \
                min(0.35, 0.08 + self.z / 20000.0)
            free_cell = None
            if kind_roll < ring_p:
                # Ring mit Loch zum Durchfädeln
                hx = self.rng.uniform(-H + 1.0, H - 1.0)
                hy = self.rng.uniform(-H + 1.0, H - 1.0)
                hole = 1.4
                # vier Rechtecke um das Loch
                self.obstacles += [
                    dict(z=zs, cx=0, cy=(hy - hole / 2 - H) / 2 + 0,
                         hw=H, hh=(hy - hole / 2 + H) / 2),
                    dict(z=zs, cx=0, cy=(hy + hole / 2 + H) / 2,
                         hw=H, hh=(H - hy - hole / 2) / 2),
                    dict(z=zs, cx=(hx - hole / 2 - H) / 2, cy=hy,
                         hw=(hx - hole / 2 + H) / 2, hh=hole / 2),
                    dict(z=zs, cx=(hx + hole / 2 + H) / 2, cy=hy,
                         hw=(H - hx - hole / 2) / 2, hh=hole / 2),
                ]
                free_cell = (hx, hy)
            elif kind_roll < ring_p + 0.30:
                # horizontale Barriere oben oder unten
                top = self.rng.random() < 0.5
                cy = -H + 0.45 * H if top else H - 0.45 * H
                self.obstacles.append(dict(z=zs, cx=0.0, cy=cy,
                                           hw=H, hh=0.45 * H))
                free_cell = (0.0, H * 0.5 if top else -H * 0.5)
            elif kind_roll < ring_p + 0.55:
                # vertikale Halbwand links oder rechts
                left = self.rng.random() < 0.5
                cx = -H / 2 if left else H / 2
                self.obstacles.append(dict(z=zs, cx=cx, cy=0.0,
                                           hw=H / 2, hh=H))
                free_cell = (H / 2 if left else -H / 2, 0.0)
            else:
                # Block in einer von 9 Zellen
                gx = self.rng.randint(-1, 1)
                gy = self.rng.randint(-1, 1)
                self.obstacles.append(dict(z=zs, cx=gx * 2.0, cy=gy * 2.0,
                                           hw=0.6, hh=0.6))
                free_cell = (((gx + 2) % 3 - 1) * 2.0, gy * 2.0)
            # Münzreihe zwischen den Hindernissen
            if self.rng.random() < 0.6 and free_cell is not None:
                n = self.rng.randint(3, 5)
                for i in range(n):
                    self.coin_list.append(dict(
                        z=zs + 6 + i * 4.0, cx=free_cell[0],
                        cy=free_cell[1], taken=False))
            self.next_spawn += self._gap_now() * self.rng.uniform(0.8, 1.2)
        self.obstacles = [o for o in self.obstacles if o["z"] > self.z - 10]
        self.coin_list = [c for c in self.coin_list if c["z"] > self.z - 10]

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.state == CRASH:
            if event.kind == InputEvent.KEYDOWN:
                if self.mode == "endless":
                    if event.key in ("Return", "space"):
                        self.game_over = False
                        self.reset()
                elif event.key in ("r", "R", "Return", "space"):
                    self._start_run(self.level)
                elif event.key in ("s", "S"):
                    self.state = SETUP
                    self.capture_mouse = False
                    self._build_setup_layout()
            return
        if self.state == FINISH:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._start_run(min(LEVELS, self.level + 1))
                elif event.key in ("s", "S"):
                    self.state = SETUP
                    self._build_setup_layout()
            return
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            for act in ("up", "down", "left", "right"):
                if self.is_action(k, act) or k == act.capitalize():
                    self.keys.add(act)
            if k in ("b", "B"):
                self.blur = 0.0 if self.blur >= 0.79 \
                    else round(self.blur + 0.1, 1)
                self._save_setting("blur", self.blur)
        elif event.kind == InputEvent.KEYUP:
            k = event.key
            for act in ("up", "down", "left", "right"):
                if self.is_action(k, act) or k == act.capitalize():
                    self.keys.discard(act)
        elif event.kind == InputEvent.MOUSEREL and self.control == "mouse":
            self.px += event.rel[0] * 0.011 * H
            self.py += event.rel[1] * 0.011 * H

    # ===================================================== Update
    def update(self, dt):
        if self.state == READY:
            self.ready_t -= dt
            if self.ready_t <= 0:
                self.state = PLAY
                self.capture_mouse = (self.control == "mouse")
            return
        if self.state != PLAY or self.game_over:
            return
        self.elapsed += dt

        # Tempo
        if self.mode == "endless":
            v = min(75.0, 26.0 + 10.0 * (self.z / 1000.0))
        else:
            progress = self.z / max(1.0, self.length)
            v = self.base_speed * (1.0 + 0.15 * progress)
        z_prev = self.z
        self.z += v * dt

        # Steuerung
        if self.control == "keys":
            ax = ((1 if "right" in self.keys else 0)
                  - (1 if "left" in self.keys else 0)) * 14.0
            ay = ((1 if "down" in self.keys else 0)
                  - (1 if "up" in self.keys else 0)) * 14.0
            self.vx += ax * dt
            self.vy += ay * dt
            self.vx -= self.vx * min(1.0, 6.0 * dt)
            self.vy -= self.vy * min(1.0, 6.0 * dt)
            self.vx = max(-7.0, min(7.0, self.vx))
            self.vy = max(-7.0, min(7.0, self.vy))
            self.px += self.vx * dt
            self.py += self.vy * dt
        self.px = max(-(H - SHIP_W), min(H - SHIP_W, self.px))
        self.py = max(-(H - SHIP_H), min(H - SHIP_H, self.py))

        # Kamera weich hinter dem Schiff
        k = min(1.0, 6.0 * dt)
        self.cam_x += (self.px * 0.55 - self.cam_x) * k
        self.cam_y += (self.py * 0.55 - self.cam_y) * k

        self._gen_ahead()

        # Kollisionen: über die gesamte z-Strecke des Frames prüfen, damit
        # bei hohem Tempo/niedriger FPS nichts übersprungen wird (Sweep).
        z_lo = z_prev + SHIP_AHEAD - 0.8
        z_hi = self.z + SHIP_AHEAD + 0.8
        for o in self.obstacles:
            if z_lo < o["z"] < z_hi:
                if (abs(self.px - o["cx"]) < SHIP_W + o["hw"]
                        and abs(self.py - o["cy"]) < SHIP_H + o["hh"]):
                    self._crash()
                    return
        for c in self.coin_list:
            if not c["taken"] and z_lo < c["z"] < z_hi:
                if math.hypot(self.px - c["cx"], self.py - c["cy"]) < 0.75:
                    c["taken"] = True
                    self.coins += 1
                    self.play_sound("point")

        # Punkte / Ziel
        if self.mode == "endless":
            self.score = int(self.z * 10 + self.coins * 50)
        elif self.z >= self.length:
            self._finish()

        if self.flash_t > 0:
            self.flash_t -= dt

    def _crash(self):
        self.play_sound("explode")
        self.rumble(220)
        self.flash_t = 0.15
        self.capture_mouse = False
        self.state = CRASH
        if self.mode == "endless":
            self.game_over = True     # Engine speichert den Score

    def _finish(self):
        self.capture_mouse = False
        par = self.length / self.base_speed * 1.25
        self.time_bonus = max(0, int((par - self.elapsed) * 20))
        self.coin_bonus = self.coins * 50
        self.run_score = 1000 + self.coin_bonus + self.time_bonus
        self.score += self.run_score
        self._mark_solved(self.level)
        self.state = FINISH
        self.play_sound("win")

    # ===================================================== 3D-Helfer
    def _tangent(self, z):
        c0 = self._center(z)
        c1 = self._center(z + 8.0)
        yaw = math.atan2(c1[0] - c0[0], 8.0)
        pitch = math.atan2(c1[1] - c0[1], 8.0)
        return yaw, pitch

    def _basis_at(self, z):
        yaw, pitch = self._tangent(z)
        cp = math.cos(pitch)
        f = (math.sin(yaw) * cp, math.sin(pitch), math.cos(yaw) * cp)
        rl = math.hypot(f[0], f[2]) or 1.0
        r = (f[2] / rl, 0.0, -f[0] / rl)
        u = (f[1] * r[2] - f[2] * r[1], f[2] * r[0] - f[0] * r[2],
             f[0] * r[1] - f[1] * r[0])
        return r, u, f

    def _world(self, z, ox, oy):
        """Punkt im Röhren-Querschnitt bei z (Offset ox/oy) -> Weltkoordinaten.

        Vereinfachung: Querschnitt bleibt achsenparallel (kein Roll) - bei den
        sanften Kurven visuell völlig ausreichend.
        """
        cx, cy = self._center(z)
        return (cx + ox, cy + oy, z)

    def _to_cam(self, p):
        r, u, f = self._basis
        dx = p[0] - self.cam[0]
        dy = p[1] - self.cam[1]
        dz = p[2] - self.cam[2]
        return (dx * r[0] + dy * r[1] + dz * r[2],
                dx * u[0] + dy * u[1] + dz * u[2],
                dx * f[0] + dy * f[1] + dz * f[2])

    def _proj(self, c):
        k = self._f / c[2]
        return (self._scx + c[0] * k, self._scy - c[1] * k)

    @staticmethod
    def _clip_seg(a, b):
        da, db = a[2] - NEAR, b[2] - NEAR
        if da < 0 and db < 0:
            return None
        if da < 0 or db < 0:
            tt = da / (da - db)
            m = (a[0] + (b[0] - a[0]) * tt, a[1] + (b[1] - a[1]) * tt, NEAR)
            return (m, b) if da < 0 else (a, m)
        return (a, b)

    def _fog(self, col, depth):
        f = min(1.0, max(0.0, depth / VIEW_DIST))
        return (int(col[0] + (COL_BG_BOT[0] - col[0]) * f),
                int(col[1] + (COL_BG_BOT[1] - col[1]) * f),
                int(col[2] + (COL_BG_BOT[2] - col[2]) * f))

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        self._scx, self._scy = self.width / 2, self.height / 2
        self._f = self.height * FOV_MUL

        if self.state == SETUP:
            self._draw_setup(s)
            return

        cx, cy = self._center(self.z)
        self.cam = (cx + self.cam_x, cy + self.cam_y, self.z)
        self._basis = self._basis_at(self.z)

        self._draw_sky(s)
        self._draw_tunnel(s)
        self._draw_obstacles(s)
        self._draw_coins(s)
        self._draw_ship(s)
        self._apply_blur(s)
        if self.flash_t > 0:
            ov = pygame.Surface((self.width, self.height))
            ov.fill((255, 255, 255))
            ov.set_alpha(int(200 * self.flash_t / 0.15))
            s.blit(ov, (0, 0))
        self._draw_hud(s)
        if self.state == READY:
            img = self._huge.render(t("tun.start_hint"), True, COL_SHIP)
            s.blit(img, img.get_rect(center=(self.width // 2,
                                             int(self.height * 0.42))))
        elif self.state == CRASH:
            self._draw_crash(s)
        elif self.state == FINISH:
            self._draw_finish(s)

    def _draw_sky(self, s):
        key = (self.width, self.height)
        if self._sky_cache is None or self._sky_cache[0] != key:
            surf = pygame.Surface((self.width, self.height))
            for y in range(self.height):
                f = y / max(1, self.height - 1)
                surf.fill((int(COL_BG_TOP[0] + (COL_BG_BOT[0] - COL_BG_TOP[0]) * f),
                           int(COL_BG_TOP[1] + (COL_BG_BOT[1] - COL_BG_TOP[1]) * f),
                           int(COL_BG_TOP[2] + (COL_BG_BOT[2] - COL_BG_TOP[2]) * f)),
                          (0, y, self.width, 1))
            self._sky_cache = (key, surf)
        s.blit(self._sky_cache[1], (0, 0))

    def _ring_pts(self, z):
        return [self._to_cam(self._world(z, ox, oy))
                for ox, oy in ((-H, -H), (H, -H), (H, H), (-H, H))]

    def _draw_tunnel(self, s):
        z0 = math.floor(self.z / 6.0) * 6.0
        prev = None
        rings = []
        for i in range(int(VIEW_DIST / 6) + 1):
            z = z0 + i * 6.0
            pts = self._ring_pts(z)
            rings.append((z, pts))
        # Ecken-Rails (verbinden aufeinanderfolgende Ringe)
        for (z_a, a), (z_b, b) in zip(rings, rings[1:]):
            for k in range(4):
                seg = self._clip_seg(a[k], b[k])
                if seg:
                    depth = (seg[0][2] + seg[1][2]) / 2
                    pa, pb = self._proj(seg[0]), self._proj(seg[1])
                    pygame.draw.line(s, self._fog(COL_RAIL, depth),
                                     (int(pa[0]), int(pa[1])),
                                     (int(pb[0]), int(pb[1])))
        # Ringe
        for z, pts in rings:
            hi = (int(z / 6.0) % 4 == 0)
            col = COL_RING_HI if hi else COL_RING
            for k in range(4):
                seg = self._clip_seg(pts[k], pts[(k + 1) % 4])
                if seg:
                    depth = (seg[0][2] + seg[1][2]) / 2
                    pa, pb = self._proj(seg[0]), self._proj(seg[1])
                    pygame.draw.line(s, self._fog(col, depth),
                                     (int(pa[0]), int(pa[1])),
                                     (int(pb[0]), int(pb[1])),
                                     2 if hi else 1)

    def _draw_obstacles(self, s):
        items = []
        for o in self.obstacles:
            rel = o["z"] - self.z
            if rel < NEAR or rel > VIEW_DIST:
                continue
            corners = [self._to_cam(self._world(
                o["z"], o["cx"] + sx * o["hw"], o["cy"] + sy * o["hh"]))
                for sx, sy in ((-1, -1), (1, -1), (1, 1), (-1, 1))]
            if all(c[2] <= NEAR for c in corners):
                continue
            depth = sum(c[2] for c in corners) / 4
            pts = [self._proj(c) for c in corners]
            items.append((depth, pts))
        items.sort(key=lambda it: -it[0])
        for depth, pts in items:
            ipts = [(int(x), int(y)) for x, y in pts]
            pygame.draw.polygon(s, self._fog(COL_OBST, depth), ipts)
            pygame.draw.polygon(s, self._fog(COL_OBST_EDGE, depth), ipts, 2)

    def _draw_coins(self, s):
        tick = pygame.time.get_ticks() / 1000.0
        for c in self.coin_list:
            if c["taken"]:
                continue
            rel = c["z"] - self.z
            if rel < NEAR or rel > VIEW_DIST:
                continue
            cam = self._to_cam(self._world(c["z"], c["cx"], c["cy"]))
            if cam[2] < NEAR:
                continue
            k = self._f / cam[2]
            x = int(self._scx + cam[0] * k)
            y = int(self._scy - cam[1] * k)
            r = max(2, int(k * 0.35 * (1.0 + 0.15 * math.sin(tick * 4))))
            col = self._fog(COL_COIN, cam[2])
            pygame.draw.circle(s, col, (x, y), r)
            pygame.draw.circle(s, self._fog((255, 240, 180), cam[2]),
                               (x, y), r, max(1, r // 4))

    def _draw_ship(self, s):
        z = self.z + SHIP_AHEAD
        pts3 = [self._world(z, self.px, self.py - SHIP_H),        # Nase oben
                self._world(z - 0.9, self.px - SHIP_W, self.py + SHIP_H),
                self._world(z - 0.5, self.px, self.py + SHIP_H * 0.4),
                self._world(z - 0.9, self.px + SHIP_W, self.py + SHIP_H)]
        cams = [self._to_cam(p) for p in pts3]
        if any(c[2] <= NEAR for c in cams):
            return
        pts = [self._proj(c) for c in cams]
        ipts = [(int(x), int(y)) for x, y in pts]
        pygame.draw.polygon(s, COL_SHIP, ipts)
        pygame.draw.polygon(s, (240, 250, 255), ipts, 2)

    def _apply_blur(self, s):
        if self.blur <= 0.01:
            self._prev_frame = None
            return
        if self._prev_frame is None \
                or self._prev_frame.get_size() != s.get_size():
            self._prev_frame = s.copy()
            return
        self._prev_frame.set_alpha(int(self.blur * 230))
        s.blit(self._prev_frame, (0, 0))
        self._prev_frame = s.copy()

    def _draw_hud(self, s):
        img = self._big.render(t("common.points", score=self.score), True,
                               COL_SHIP)
        s.blit(img, (14, 10))
        right = [t("tun.coins", n=self.coins)]
        if self.mode == "endless":
            v = min(75.0, 26.0 + 10.0 * (self.z / 1000.0))
            right.append(t("tun.distance", n=int(self.z)))
            right.append(t("tun.speed", v=int(v * 3.6)))
        else:
            right.insert(0, t("tun.level", n=self.level))
            pct = int(100 * self.z / max(1, self.length))
            right.append(f"{min(100, pct)}%")
        y = 12
        for line in right:
            img = self._small.render(line, True, COL_DIM)
            s.blit(img, img.get_rect(topright=(self.width - 14, y)))
            y += 22
        # Fortschrittsbalken (Level-Modus)
        if self.mode == "levels" and self.state == PLAY:
            frac = min(1.0, self.z / max(1, self.length))
            pygame.draw.rect(s, (40, 46, 66),
                             (0, self.height - 6, self.width, 6))
            pygame.draw.rect(s, COL_SHIP,
                             (0, self.height - 6,
                              int(self.width * frac), 6))

    def _draw_crash(self, s):
        ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        ov.fill((10, 6, 20, 175))
        s.blit(ov, (0, 0))
        cx, cy = self.width // 2, self.height // 2
        head = self._huge.render(t("tun.crash"), True, (255, 110, 110))
        s.blit(head, head.get_rect(center=(cx, cy - 50)))
        sc = self.font.render(t("common.points", score=self.score), True,
                              COL_TEXT)
        s.blit(sc, sc.get_rect(center=(cx, cy + 2)))
        key = "common.enter_restart" if self.mode == "endless" else "tun.retry"
        hint = self._small.render(t(key), True, COL_DIM)
        s.blit(hint, hint.get_rect(center=(cx, cy + 40)))

    def _draw_finish(self, s):
        ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        ov.fill((6, 12, 24, 185))
        s.blit(ov, (0, 0))
        cx, cy = self.width // 2, self.height // 2
        head = self._huge.render(t("tun.level_done", n=self.level), True,
                                 COL_SHIP)
        s.blit(head, head.get_rect(center=(cx, cy - 70)))
        lines = [t("tun.base", n=1000),
                 t("tun.coin_bonus", n=self.coin_bonus),
                 t("tun.time_bonus", n=self.time_bonus),
                 t("common.points", score=self.score)]
        y = cy - 20
        for line in lines:
            img = self.font.render(line, True, COL_TEXT)
            s.blit(img, img.get_rect(center=(cx, y)))
            y += 30
        hint = self._small.render(t("tun.next"), True, COL_DIM)
        s.blit(hint, hint.get_rect(center=(cx, y + 12)))

    # ----- Setup zeichnen -----------------------------------------------
    def _draw_setup(self, s):
        s.fill(COL_BG_TOP)
        self._draw_sky(s)
        cx = self.width // 2
        title = self._huge.render("TUNNEL RACER", True, COL_SHIP)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.10))))
        mode_lbl = t("tun.mode." + self.mode)
        sub = self._small.render(mode_lbl + "   -   " + t("tun.subtitle"),
                                 True, COL_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(self.height * 0.17))))

        lbl = self._small.render(t("tun.control"), True, (200, 205, 220))
        s.blit(lbl, lbl.get_rect(midright=(self.ctrl_rect.x - 16,
                                           self.ctrl_rect.centery)))
        pygame.draw.rect(s, COL_BTN_ON, self.ctrl_rect, border_radius=8)
        pygame.draw.rect(s, COL_BTN_BORDER, self.ctrl_rect, 1,
                         border_radius=8)
        img = self._small.render(
            t("tun.control." + self.control) + "  [C]", True, COL_TEXT)
        s.blit(img, img.get_rect(center=self.ctrl_rect.center))

        lbl = self._small.render(t("tun.blur"), True, (200, 205, 220))
        s.blit(lbl, lbl.get_rect(midright=(self.blur_minus.x - 16,
                                           self.blur_minus.centery)))
        for r, sym in ((self.blur_minus, "-"), (self.blur_plus, "+")):
            pygame.draw.rect(s, COL_BTN, r, border_radius=8)
            pygame.draw.rect(s, COL_BTN_BORDER, r, 1, border_radius=8)
            img = self._big.render(sym, True, COL_TEXT)
            s.blit(img, img.get_rect(center=r.center))
        blur_lbl = t("common.off") if self.blur <= 0 \
            else f"{int(self.blur * 100)}%"
        img = self._big.render(blur_lbl, True, COL_SHIP)
        s.blit(img, img.get_rect(center=self.blur_box.center))

        if self.mode == "levels":
            for n in range(1, LEVELS + 1):
                i = n - 1
                x = self.lv_x + (i % 10) * self.lv_cell
                y = self.lv_y + (i // 10) * self.lv_cell
                cell = pygame.Rect(x + 1, y + 1, self.lv_cell - 2,
                                   self.lv_cell - 2)
                done = n in self.solved
                pygame.draw.rect(s, (26, 56, 52) if done else (30, 36, 52),
                                 cell, border_radius=4)
                if n == self.cursor:
                    pygame.draw.rect(s, COL_SHIP, cell, 2, border_radius=4)
                num = self._lv_font.render(str(n), True,
                                           (110, 220, 190) if done else COL_DIM)
                s.blit(num, num.get_rect(center=cell.center))
            prog = self._small.render(
                t("tun.progress", n=len(self.solved), m=LEVELS), True,
                COL_DIM)
            s.blit(prog, prog.get_rect(
                center=(cx, self.lv_y + 3 * self.lv_cell + 40)))

        pygame.draw.rect(s, COL_BTN_ON, self.start_rect, border_radius=10)
        pygame.draw.rect(s, COL_SHIP, self.start_rect, 2, border_radius=10)
        st = self.font.render(t("common.start"), True, COL_TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
