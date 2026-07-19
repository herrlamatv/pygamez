# -*- coding: utf-8 -*-
"""
labyrinth.py
============
3D-Labyrinth - Ego-Raycaster im Wolfenstein-Stil (Einzelspieler).

- 50 seed-generierte Level (maze_gen.py), Fortschritt wird gespeichert.
- Ego-Ansicht: Software-Raycaster (DDA), Mouselook (Pointer-Capture) +
  WASD/Pfeile, [Q]/[E] als Tastatur-Drehung, [M] Minimap.
- Alternativ Top-Down-2D-Ansicht (im Setup umschaltbar, gespeichert):
  klassische Draufsicht ohne Maus-Capture.
- Orbs einsammeln, den (grün pulsierenden) Ausgang finden; Punkte =
  500 + Orbs*100 + Zeitbonus, sitzungskumulativ.
"""

import math
import random

import pygame

import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent, LocalizedName
from games import maze_gen
from i18n import t

LEVELS = 50
MOVE_SPEED = 3.0          # Tiles/s
PLAYER_R = 0.25
TURN_KEY = 90.0           # Grad/s über Q/E
DEG_PER_PX = 0.12
FOG_DIST = 14.0

# 3D-Welt-Palette (bewusst fest - unabhängig vom UI-Theme)
COL_BG = (16, 14, 22)
COL_CEIL_TOP = (30, 34, 52)
COL_CEIL_BOT = (20, 22, 34)
COL_FLOOR = (34, 30, 40)
COL_WALL = (90, 110, 170)
COL_EXIT = (120, 255, 170)
COL_ORB = (120, 200, 255)

SETUP, PLAY, FINISH = "setup", "play", "finish"


class LabyrinthGame(Game):
    name = LocalizedName("3D Maze", de="3D-Labyrinth", fr="Labyrinthe 3D",
                         es="Laberinto 3D", pt="Labirinto 3D")
    highscore_key = "maze"
    supports_multiplayer = False

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        mz = self.settings.get("maze", {}) if isinstance(self.settings, dict) else {}
        self.view = mz.get("view", "ego")
        if self.view not in ("ego", "top"):
            self.view = "ego"
        try:
            self.sens = max(0.5, min(2.0, float(mz.get("sens", 1.0))))
        except (TypeError, ValueError):
            self.sens = 1.0
        try:
            self.cursor = max(1, min(LEVELS, int(mz.get("last_level", 1))))
        except (TypeError, ValueError):
            self.cursor = 1

        self._make_fonts()
        self._ov_cache = {}          # gecachte Vollbild-Abdunklungen
        self.capture_mouse = False
        self.show_map = False
        self._load_solved()
        self.level = 1
        self._build_setup_layout()
        self.state = SETUP

    def on_surface_changed(self):
        self._make_fonts()
        self._ov_cache = {}
        self._build_setup_layout()

    def _make_fonts(self):
        """Schriftgrößen aus der aktuellen Auflösung ableiten (Theme-Schrift)."""
        h = self.height
        self._small = ui.font(max(13, h // 30))
        self._tiny = ui.font(max(11, h // 36))
        self._big = ui.font(max(16, h // 21), bold=True)
        self._huge = ui.font(max(26, h // 11), bold=True)

    def _dim(self, s, rgb, alpha):
        """Vollbild-Abdunklung über eine gecachte Surface - vermeidet den
        SRCALPHA-Fill über die ganze Fläche in jedem Frame."""
        surf = self._ov_cache.get(rgb)
        if surf is None or surf.get_size() != (self.width, self.height):
            surf = pygame.Surface((self.width, self.height))
            surf.fill(rgb)
            self._ov_cache[rgb] = surf
        surf.set_alpha(alpha)
        s.blit(surf, (0, 0))

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("maze", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _load_solved(self):
        data = store.load_section("maze")
        lst = data.get("solved", [])
        if isinstance(lst, list):
            self.solved = sorted({int(v) for v in lst
                                  if isinstance(v, int) and 1 <= v <= LEVELS})
        else:
            self.solved = []

    def _mark_solved(self, n):
        if n not in self.solved:
            self.solved = sorted(self.solved + [n])
            store.save_section("maze", {"solved": self.solved})

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        y0 = int(self.height * 0.22)
        rx = cx + 24
        self.view_rect = pygame.Rect(rx - 60, y0, 220, 40)
        self.sens_minus = pygame.Rect(rx - 60, y0 + 52, 44, 40)
        self.sens_plus = pygame.Rect(rx + 116, y0 + 52, 44, 40)
        self.sens_box = pygame.Rect(rx - 8, y0 + 52, 116, 40)
        top = y0 + 110
        avail_h = self.height - top - 96
        cell = max(18, min((self.width - 80) // 10, avail_h // 5))
        self.lv_cell = cell
        self.lv_x = cx - cell * 5
        self.lv_y = top
        self._lv_font = ui.font(max(9, cell * 2 // 5))
        self.start_rect = pygame.Rect(cx - 95, top + 5 * cell + 12, 190, 44)

    def _level_at(self, pos):
        x, y = pos
        c = (x - self.lv_x) // self.lv_cell
        r = (y - self.lv_y) // self.lv_cell
        if 0 <= c < 10 and 0 <= r < 5:
            return int(r * 10 + c + 1)
        return None

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("v", "V"):
                self._toggle_view()
            elif k in ("Left", "a", "A"):
                self.cursor = (self.cursor - 2) % LEVELS + 1
                self.play_sound("move")
            elif k in ("Right", "d", "D"):
                self.cursor = self.cursor % LEVELS + 1
                self.play_sound("move")
            elif k in ("Up", "w", "W"):
                self.cursor = (self.cursor - 11) % LEVELS + 1
                self.play_sound("move")
            elif k in ("Down", "s", "S"):
                self.cursor = (self.cursor + 9) % LEVELS + 1
                self.play_sound("move")
            elif k in ("Return", "space"):
                self._start_level(self.cursor)
        elif event.kind == InputEvent.MOUSEDOWN:
            if self.view_rect.collidepoint(event.pos):
                self._toggle_view()
                return
            if self.sens_minus.collidepoint(event.pos):
                self.sens = round(max(0.5, self.sens - 0.1), 1)
                self._save_setting("sens", self.sens)
                self.play_sound("select")
                return
            if self.sens_plus.collidepoint(event.pos):
                self.sens = round(min(2.0, self.sens + 0.1), 1)
                self._save_setting("sens", self.sens)
                self.play_sound("select")
                return
            lv = self._level_at(event.pos)
            if lv is not None:
                self._start_level(lv)
                return
            if self.start_rect.collidepoint(event.pos):
                self._start_level(self.cursor)

    def _toggle_view(self):
        self.view = "top" if self.view == "ego" else "ego"
        self._save_setting("view", self.view)
        self.play_sound("select")

    # ===================================================== Level starten
    def _start_level(self, level):
        self.level = max(1, min(LEVELS, level))
        self.cursor = self.level
        self._save_setting("last_level", self.level)
        p = maze_gen.level_params(self.level)
        rng = random.Random(4400 + self.level)
        self.grid = maze_gen.generate(p["cells"], rng)
        self.n = len(self.grid)
        self.exit_pos, dist = maze_gen.far_exit(self.grid)
        self.orbs = [list(o) for o in maze_gen.place_orbs(
            self.grid, dist, self.exit_pos, p["orbs"], rng)]
        self.orbs_total = len(self.orbs)
        self.par = p["par"]
        self.px, self.py = 1.5, 1.5
        self.yaw = 0.0            # 0° = +x
        self.elapsed = 0.0
        self.got = 0
        self.keys = set()
        self.show_map = False
        self.state = PLAY
        self.capture_mouse = (self.view == "ego")
        self.play_sound("level")

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.state == FINISH:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._start_level(min(LEVELS, self.level + 1))
                elif event.key in ("s", "S"):
                    self.state = SETUP
                    self.capture_mouse = False
                    self._build_setup_layout()
            return
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            for act in ("up", "down", "left", "right"):
                if self.is_action(k, act) or k == act.capitalize():
                    self.keys.add(act)
            if k in ("q", "Q"):
                self.keys.add("turn_l")
            elif k in ("e", "E"):
                self.keys.add("turn_r")
            elif k in ("m", "M") and self.view == "ego":
                self.show_map = not self.show_map
                self.play_sound("select")
        elif event.kind == InputEvent.KEYUP:
            k = event.key
            for act in ("up", "down", "left", "right"):
                if self.is_action(k, act) or k == act.capitalize():
                    self.keys.discard(act)
            if k in ("q", "Q"):
                self.keys.discard("turn_l")
            elif k in ("e", "E"):
                self.keys.discard("turn_r")
        elif event.kind == InputEvent.MOUSEREL and self.view == "ego":
            self.yaw += event.rel[0] * DEG_PER_PX * self.sens

    # ===================================================== Update
    def _wall(self, x, y):
        ix, iy = int(x), int(y)
        if 0 <= ix < self.n and 0 <= iy < self.n:
            return self.grid[iy][ix] == 1
        return True

    def _blocked(self, x, y):
        r = PLAYER_R
        return (self._wall(x - r, y - r) or self._wall(x + r, y - r)
                or self._wall(x - r, y + r) or self._wall(x + r, y + r))

    def _move(self, dx, dy, dt):
        L = math.hypot(dx, dy)
        if L < 1e-9:
            return
        dx, dy = dx / L * MOVE_SPEED * dt, dy / L * MOVE_SPEED * dt
        nx = self.px + dx
        if not self._blocked(nx, self.py):
            self.px = nx
        ny = self.py + dy
        if not self._blocked(self.px, ny):
            self.py = ny

    def update(self, dt):
        if self.state != PLAY:
            return
        self.elapsed += dt

        if "turn_l" in self.keys:
            self.yaw -= TURN_KEY * dt
        if "turn_r" in self.keys:
            self.yaw += TURN_KEY * dt

        fwd = ((1 if "up" in self.keys else 0)
               - (1 if "down" in self.keys else 0))
        side = ((1 if "right" in self.keys else 0)
                - (1 if "left" in self.keys else 0))
        if self.view == "ego":
            # Strafe: rechts = +90° zur Blickrichtung
            a = math.radians(self.yaw)
            dx = math.cos(a) * fwd + math.cos(a + math.pi / 2) * side
            dy = math.sin(a) * fwd + math.sin(a + math.pi / 2) * side
            if fwd or side:
                self._move(dx, dy, dt)
        else:
            if fwd or side:
                self._move(side, -fwd, dt)

        # Orbs einsammeln
        for o in self.orbs:
            if math.hypot(self.px - (o[0] + 0.5),
                          self.py - (o[1] + 0.5)) < 0.5:
                self.orbs.remove(o)
                self.got += 1
                self.play_sound("powerup")
                break

        # Ausgang erreicht?
        if (int(self.px), int(self.py)) == self.exit_pos:
            self._finish()

    def _finish(self):
        self.capture_mouse = False
        self.time_bonus = max(0, int((self.par - self.elapsed) * 8))
        self.orb_bonus = self.got * 100
        self.level_score = 500 + self.orb_bonus + self.time_bonus
        self.score += self.level_score
        self._mark_solved(self.level)
        self.state = FINISH
        self.play_sound("win")

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        if self.state == SETUP:
            self._draw_setup(s)
            return
        if self.view == "ego":
            self._draw_ego(s)
            if self.show_map:
                self._draw_minimap(s)
        else:
            self._draw_top(s)
        self._draw_hud(s)
        if self.state == FINISH:
            self._draw_finish(s)

    # ----- Raycaster ----------------------------------------------------
    def _draw_ego(self, s):
        w, h = self.width, self.height
        half = h // 2
        # Decke zweifarbig + Boden
        pygame.draw.rect(s, COL_CEIL_TOP, (0, 0, w, half // 2))
        pygame.draw.rect(s, COL_CEIL_BOT, (0, half // 2, w, half - half // 2))
        pygame.draw.rect(s, COL_FLOOR, (0, half, w, h - half))

        a = math.radians(self.yaw)
        dirx, diry = math.cos(a), math.sin(a)
        planex, planey = -diry * 0.66, dirx * 0.66
        step_px = 2
        ncols = w // step_px + 1
        zbuf = [1e9] * ncols
        tick = pygame.time.get_ticks() / 1000.0
        exit_pulse = 0.75 + 0.25 * math.sin(tick * 4)

        for col in range(ncols):
            x = col * step_px
            camx = 2.0 * x / max(1, w) - 1.0
            rdx = dirx + planex * camx
            rdy = diry + planey * camx
            mapx, mapy = int(self.px), int(self.py)
            ddx = abs(1.0 / rdx) if rdx else 1e30
            ddy = abs(1.0 / rdy) if rdy else 1e30
            if rdx < 0:
                stepx, sdx = -1, (self.px - mapx) * ddx
            else:
                stepx, sdx = 1, (mapx + 1.0 - self.px) * ddx
            if rdy < 0:
                stepy, sdy = -1, (self.py - mapy) * ddy
            else:
                stepy, sdy = 1, (mapy + 1.0 - self.py) * ddy
            side = 0
            hit = False
            for _ in range(4 * self.n):
                if sdx < sdy:
                    sdx += ddx
                    mapx += stepx
                    side = 0
                else:
                    sdy += ddy
                    mapy += stepy
                    side = 1
                if mapx < 0 or mapy < 0 or mapx >= self.n or mapy >= self.n:
                    hit = True
                    break
                if self.grid[mapy][mapx] == 1:
                    hit = True
                    break
            if not hit:
                continue
            perp = (sdx - ddx) if side == 0 else (sdy - ddy)
            perp = max(0.02, perp)
            zbuf[col] = perp
            line_h = int(h / perp)
            y0 = max(0, half - line_h // 2)
            y1 = min(h, half + line_h // 2)
            col_rgb = COL_WALL
            # Wand direkt am Ausgangs-Tile grün einfärben
            if (mapx, mapy) != self.exit_pos:
                ex, ey = self.exit_pos
                if abs(mapx - ex) + abs(mapy - ey) == 1:
                    col_rgb = (int(COL_EXIT[0] * exit_pulse),
                               int(COL_EXIT[1] * exit_pulse),
                               int(COL_EXIT[2] * exit_pulse))
            shade = 0.8 if side == 1 else 1.0
            fog = min(1.0, perp / FOG_DIST)
            r = int((col_rgb[0] * shade) * (1 - fog) + COL_BG[0] * fog)
            g = int((col_rgb[1] * shade) * (1 - fog) + COL_BG[1] * fog)
            b = int((col_rgb[2] * shade) * (1 - fog) + COL_BG[2] * fog)
            pygame.draw.rect(s, (r, g, b), (x, y0, step_px, y1 - y0))

        # Sprites: Orbs + Exit-Glow (nach Distanz sortiert, fern zuerst)
        sprites = [(o[0] + 0.5, o[1] + 0.5, COL_ORB, 0.30) for o in self.orbs]
        sprites.append((self.exit_pos[0] + 0.5, self.exit_pos[1] + 0.5,
                        COL_EXIT, 0.45))
        inv = 1.0 / (planex * diry - dirx * planey or 1e-9)
        order = []
        for sx, sy, colr, size in sprites:
            rx, ry = sx - self.px, sy - self.py
            tx = inv * (diry * rx - dirx * ry)
            ty = inv * (-planey * rx + planex * ry)
            if ty <= 0.1:
                continue
            order.append((ty, tx, colr, size))
        order.sort(key=lambda it: -it[0])
        for ty, tx, colr, size in order:
            sx_px = int((w / 2) * (1 + tx / ty))
            r_px = max(2, int(h * size / ty))
            # Occlusion: 3 zbuf-Samples, sichtbar wenn >= 2 frei
            free = 0
            for off in (-r_px // 2, 0, r_px // 2):
                c = (sx_px + off) // step_px
                if 0 <= c < ncols and zbuf[c] > ty:
                    free += 1
            if free < 2:
                continue
            fog = min(1.0, ty / FOG_DIST)
            cc = (int(colr[0] * (1 - fog) + COL_BG[0] * fog),
                  int(colr[1] * (1 - fog) + COL_BG[1] * fog),
                  int(colr[2] * (1 - fog) + COL_BG[2] * fog))
            cy = half + int(h * 0.12 / ty)
            pulse = 1.0 + 0.12 * math.sin(tick * 5 + tx)
            rr = max(2, int(r_px * pulse) // 2)
            pygame.draw.circle(s, cc, (sx_px, cy), rr)
            pygame.draw.circle(s, (min(255, cc[0] + 60), min(255, cc[1] + 60),
                                   min(255, cc[2] + 60)),
                               (sx_px, cy), rr, max(1, rr // 4))

    def _draw_minimap(self, s):
        ts = max(2, min(6, 180 // self.n))
        size = ts * self.n
        mm = pygame.Surface((size, size), pygame.SRCALPHA)
        mm.fill((10, 10, 16, 185))
        for y in range(self.n):
            for x in range(self.n):
                if self.grid[y][x] == 1:
                    mm.fill((90, 100, 140, 220),
                            (x * ts, y * ts, ts, ts))
        ex, ey = self.exit_pos
        mm.fill((*COL_EXIT, 255), (ex * ts, ey * ts, ts, ts))
        for o in self.orbs:
            mm.fill((*COL_ORB, 255), (o[0] * ts, o[1] * ts, ts, ts))
        px, py = int(self.px * ts), int(self.py * ts)
        pygame.draw.circle(mm, (255, 220, 120), (px, py), max(2, ts // 2))
        a = math.radians(self.yaw)
        pygame.draw.line(mm, (255, 220, 120), (px, py),
                         (px + int(math.cos(a) * ts * 2),
                          py + int(math.sin(a) * ts * 2)), 1)
        s.blit(mm, (self.width - size - 12, 78))

    # ----- Top-Down ------------------------------------------------------
    def _draw_top(self, s):
        s.fill(COL_BG)
        avail = min(self.width - 20, self.height - 56)
        ts = avail // self.n
        tick = pygame.time.get_ticks() / 1000.0
        if ts >= 5:
            ox = (self.width - ts * self.n) // 2
            oy = 44 + (self.height - 44 - ts * self.n) // 2
            x0, y0, x1, y1 = 0, 0, self.n, self.n
        else:
            ts = 22
            ox = self.width // 2 - int(self.px * ts)
            oy = self.height // 2 - int(self.py * ts)
            x0 = max(0, int(self.px) - self.width // (2 * ts) - 2)
            x1 = min(self.n, int(self.px) + self.width // (2 * ts) + 3)
            y0 = max(0, int(self.py) - self.height // (2 * ts) - 2)
            y1 = min(self.n, int(self.py) + self.height // (2 * ts) + 3)
        for y in range(y0, y1):
            for x in range(x0, x1):
                r = pygame.Rect(ox + x * ts, oy + y * ts, ts, ts)
                if self.grid[y][x] == 1:
                    pygame.draw.rect(s, (52, 62, 96), r)
                    pygame.draw.rect(s, (34, 40, 64), r, 1)
        ex, ey = self.exit_pos
        pulse = 0.7 + 0.3 * math.sin(tick * 4)
        er = pygame.Rect(ox + ex * ts, oy + ey * ts, ts, ts)
        pygame.draw.rect(s, (int(COL_EXIT[0] * pulse),
                             int(COL_EXIT[1] * pulse),
                             int(COL_EXIT[2] * pulse)), er)
        for o in self.orbs:
            cx = ox + int((o[0] + 0.5) * ts)
            cy = oy + int((o[1] + 0.5) * ts)
            rr = max(2, int(ts * 0.28 * (1 + 0.15 * math.sin(tick * 5))))
            pygame.draw.circle(s, COL_ORB, (cx, cy), rr)
        # Spieler als Pfeil
        px = ox + self.px * ts
        py = oy + self.py * ts
        pygame.draw.circle(s, (255, 220, 120), (int(px), int(py)),
                           max(3, int(ts * 0.3)))

    # ----- HUD / Overlays -------------------------------------------------
    def _draw_hud(self, s):
        img = self._big.render(t("common.points", score=self.score), True,
                               self.accent)
        s.blit(img, (14, 8))
        lines = [t("maze.level", n=self.level),
                 t("maze.orbs", n=self.got, m=self.orbs_total),
                 t("maze.time", s=int(self.elapsed))]
        y = 10
        for line in lines:
            img = self._small.render(line, True, ui.TEXT_DIM)
            s.blit(img, img.get_rect(topright=(self.width - 14, y)))
            y += 20
        if self.view == "ego" and self.state == PLAY and not self.show_map:
            hint = self._tiny.render(t("maze.map_hint"), True, ui.TEXT_FAINT)
            s.blit(hint, hint.get_rect(
                midbottom=(self.width // 2, self.height - 6)))

    def _draw_finish(self, s):
        self._dim(s, (8, 14, 12), 190)
        cx, cy = self.width // 2, self.height // 2
        head = self._huge.render(t("maze.level_done", n=self.level), True,
                                 COL_EXIT)
        lines = [t("maze.base", n=500),
                 t("maze.orb_bonus", n=self.orb_bonus),
                 t("maze.time_bonus", n=self.time_bonus),
                 t("common.points", score=self.score)]
        imgs = [self.font.render(line, True, ui.TEXT) for line in lines]
        hint = self._small.render(t("maze.next"), True, ui.TEXT_DIM)

        # Panel hinter dem Ergebnis (dynamische ui-Palette)
        top = cy - 70 - head.get_height() // 2 - 22
        bottom = cy - 20 + 30 * len(imgs) + 12 + hint.get_height() // 2 + 22
        pw = min(self.width - 40,
                 max(400, head.get_width() + 80, hint.get_width() + 60,
                     max(i.get_width() for i in imgs) + 60))
        panel = pygame.Rect(cx - pw // 2, top, pw, bottom - top)
        pygame.draw.rect(s, ui.PANEL, panel, border_radius=14)
        pygame.draw.rect(s, ui.BORDER_LIGHT, panel, 1, border_radius=14)

        s.blit(head, head.get_rect(center=(cx, cy - 70)))
        y = cy - 20
        for img in imgs:
            s.blit(img, img.get_rect(center=(cx, y)))
            y += 30
        s.blit(hint, hint.get_rect(center=(cx, y + 12)))

    # ----- Setup ----------------------------------------------------------
    def _draw_setup(self, s):
        ui.draw_background(s, self.width, self.height)
        cx = self.width // 2
        title = self._huge.render(t("maze.title"), True, self.accent)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.09))))
        sub = self._small.render(t("maze.subtitle"), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(self.height * 0.15))))

        lbl = self._small.render(t("maze.view"), True, ui.TEXT_DIM)
        s.blit(lbl, lbl.get_rect(midright=(self.view_rect.x - 16,
                                           self.view_rect.centery)))
        pygame.draw.rect(s, ui.BTN_SEL, self.view_rect, border_radius=8)
        pygame.draw.rect(s, ui.BORDER, self.view_rect, 1,
                         border_radius=8)
        img = self._small.render(
            t("maze.view." + self.view) + "  [V]", True, ui.TEXT)
        s.blit(img, img.get_rect(center=self.view_rect.center))

        lbl = self._small.render(t("maze.sens"), True, ui.TEXT_DIM)
        s.blit(lbl, lbl.get_rect(midright=(self.sens_minus.x - 16,
                                           self.sens_minus.centery)))
        for r, sym in ((self.sens_minus, "-"), (self.sens_plus, "+")):
            pygame.draw.rect(s, ui.BTN, r, border_radius=8)
            pygame.draw.rect(s, ui.BORDER, r, 1, border_radius=8)
            img = self._big.render(sym, True, ui.TEXT)
            s.blit(img, img.get_rect(center=r.center))
        img = self._big.render(f"{self.sens:.1f}x", True, self.accent)
        s.blit(img, img.get_rect(center=self.sens_box.center))

        done_fill = ui.mix(ui.BTN, self.accent, 0.25)
        done_text = ui.mix(self.accent, ui.TEXT, 0.35)
        for n in range(1, LEVELS + 1):
            i = n - 1
            x = self.lv_x + (i % 10) * self.lv_cell
            y = self.lv_y + (i // 10) * self.lv_cell
            cell = pygame.Rect(x + 1, y + 1, self.lv_cell - 2,
                               self.lv_cell - 2)
            done = n in self.solved
            pygame.draw.rect(s, done_fill if done else ui.BTN,
                             cell, border_radius=4)
            if n == self.cursor:
                pygame.draw.rect(s, self.accent, cell, 2, border_radius=4)
            num = self._lv_font.render(
                str(n), True, done_text if done else ui.TEXT_DIM)
            s.blit(num, num.get_rect(center=cell.center))
        prog = self._small.render(
            t("maze.progress", n=len(self.solved), m=LEVELS), True,
            ui.TEXT_DIM)
        s.blit(prog, prog.get_rect(
            center=(cx, self.lv_y + 5 * self.lv_cell + 34)))

        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=10)
        pygame.draw.rect(s, self.accent, self.start_rect, 2, border_radius=10)
        st = self.font.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
