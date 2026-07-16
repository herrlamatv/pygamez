# -*- coding: utf-8 -*-
"""
blockjump.py
============
Block Jump - ein 3D-Jump'n'Run im Minecraft-Stil.

- Voll in 3D (Software-Renderer, gleiche Pipeline wie Snakes 3D-Modus): eine
  Welt aus Wuerfel-Bloecken, auf die man springt. Distanz-Nebel, Himmels-Verlauf
  und optionaler Motion-Blur sorgen fuer den "hochwertigen" Look.
- Blocktypen: Gras/Erde/Stein/Holz (fest, zum Draufspringen), **Leitern**
  (kletterbar), **Zaeune** (kleine Bloecke - blockieren, aber ueberspringbar),
  **Sprungbloecke** (katapultieren nach oben), ein **Ziel** und schwebende **Coins**.
- Kamera **standardmaessig 1st-Person wie Minecraft** (Mouselook mit Pointer-
  Capture); **V** schaltet auf eine 3rd-Person-Verfolgerkamera um.
- Steuerung: **WASD/Pfeile** laufen (relativ zur Blickrichtung), **Leertaste**
  springen, **Maus** umsehen. An Leitern klettert **W/S** hoch/runter.
  **V** Ansicht, **B** Motion-Blur, **C** Maus fangen/frei, **+/-** Empfindlichkeit.
- Seed-generierte Parkour-Level werden pro Level schwerer; das Ziel erreichen gibt
  Punkte + Zeitbonus, ein Absturz kostet ein Leben (Start mit 3). Bei 0 Leben ist
  Schluss - die Punktesumme ist der Highscore.
"""

import math
import random

import pygame

import ui
import settings as settings_mod
from game_base import Game, InputEvent
from i18n import t

# ---------------------------------------------------------------------------
#  Renderer-Konstanten (aus dem 3D-Modus von snake.py uebernommen)
# ---------------------------------------------------------------------------
NEAR = 0.12
FOV_MUL = 1.0
FOG_START = 16.0
FOG_END = 46.0
DEG_PER_PX = 0.12                 # Grad Drehung je Maus-Pixel bei sens = 1.0
PITCH_CLAMP = math.radians(84)

COL_SKY_TOP = (108, 164, 232)
COL_SKY_HOR = (196, 222, 244)
COL_FOG = (206, 224, 240)

# ---------------------------------------------------------------------------
#  Physik
# ---------------------------------------------------------------------------
GRAVITY = 20.0
JUMP_VEL = 7.7
MOVE_SPEED = 4.7
CLIMB_SPEED = 3.3
SPRING_VEL = 12.6
EYE_H = 1.62
PLAYER_H = 1.7
HALF_W = 0.3
VY_MIN, VY_MAX = -16.0, 14.5
EPS = 1e-4

# ---------------------------------------------------------------------------
#  Blocktypen + Farben (Oberseite, Seiten)
# ---------------------------------------------------------------------------
EMPTY, GRASS, DIRT, STONE, PLANK, LADDER, FENCE, SPRING, GOAL = range(9)
SOLID = {GRASS, DIRT, STONE, PLANK, FENCE, SPRING, GOAL}

BLOCK_COLS = {
    GRASS: ((104, 184, 84), (124, 94, 58)),
    DIRT:  ((132, 98, 66), (120, 88, 58)),
    STONE: ((140, 146, 154), (120, 126, 134)),
    PLANK: ((182, 142, 88), (156, 118, 70)),
    GOAL:  ((130, 240, 188), (72, 190, 148)),
}
COL_LADDER = (176, 126, 66)
COL_FENCE = (156, 116, 70)
COL_SPRING_BASE = (86, 92, 118)
COL_SPRING_PAD = (112, 236, 200)
COL_COIN = (245, 210, 96)
COL_PLAYER_BODY = (74, 118, 210)
COL_PLAYER_HEAD = (235, 196, 158)

READY, PLAY, CLEAR, GAMEOVER = "ready", "play", "clear", "gameover"

SEED_BASE = {"easy": 1000, "normal": 2000, "hard": 3000}


def _dir_from(yaw, pitch):
    cp = math.cos(pitch)
    return (math.sin(yaw) * cp, math.sin(pitch), math.cos(yaw) * cp)


class BlockJumpGame(Game):
    name = "Block Jump"
    highscore_key = "blockjump"
    supports_multiplayer = False

    MODES = [("easy", "blj.mode.easy"), ("normal", "blj.mode.normal"),
             ("hard", "blj.mode.hard")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.accent = ui.game_color(type(self).__name__)
        mode = self.mode if self.mode in SEED_BASE else "normal"
        self.mode = mode
        cfg = self.settings.get("blockjump", {}) if isinstance(self.settings, dict) else {}
        self.blur = max(0.0, min(0.8, float(cfg.get("blur", 0.35))))
        self.view = cfg.get("view", "first")
        if self.view not in ("first", "third"):
            self.view = "first"
        self.sens = max(0.4, min(2.5, float(cfg.get("sens", 1.0))))
        # Maus-Richtung: Standard normal (Maus rechts -> Blick rechts). Wer die
        # frühere/klassische Belegung will, kann invertiert einstellen (Taste I).
        self.invert = bool(cfg.get("mouse_invert", False))
        self._make_fonts()
        self._sky_cache = None
        self._prev_frame = None
        self.anim = 0.0
        self.held = set()
        self.capture_mouse = False
        self._new_run()

    def _make_fonts(self):
        self._hud = ui.font(20, bold=True)
        self._small = ui.font(15)
        self._huge = ui.font(max(30, self.height // 11), bold=True)
        self._mid = ui.font(22, bold=True)

    def on_surface_changed(self):
        self._make_fonts()
        self._sky_cache = None
        self._prev_frame = None

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("blockjump", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _new_run(self):
        self.lives = 3
        self.score = 0
        self.game_over = False
        self.level = 1
        self.yaw = 0.0
        self.pitch = 0.0
        self._build_level(self.level)
        self.state = READY

    # ===================================================== Level-Generierung
    def _build_level(self, level):
        rng = random.Random(SEED_BASE.get(self.mode, 2000) + level * 7919)
        self.world = {}
        self.coins = []
        hard = self.mode == "hard"
        easy = self.mode == "easy"

        n_plat = 7 + level * 2 + (2 if hard else 0)
        self._plat_count = n_plat
        cx, cy, cz = 0, 0, 0
        self._min_y = 0
        self._pad(cx, cz, cy, 1, 1, GRASS)
        self.spawn = (cx + 0.5, cy + 1.0, cz + 0.5)

        feats = (["jump", "jump", "ladder", "spring", "fence"]
                 if not easy else ["jump", "jump", "jump", "ladder", "spring"])
        for i in range(n_plat):
            last = (i == n_plat - 1)
            feat = "jump" if last else rng.choice(feats)
            hw = 1
            hd = 1 if not hard else rng.choice([0, 1, 1])
            top_type = rng.choice([GRASS, GRASS, STONE, PLANK])

            if feat == "ladder":
                climb = rng.choice([3, 4] if easy else [3, 4, 5])
                # Leiter-Saeule mit Stuetzwand dahinter
                for yy in range(cy + 1, cy + climb + 1):
                    self.world[(cx, yy, cz + 1)] = LADDER
                    self.world[(cx, yy, cz + 2)] = STONE
                cy = cy + climb
                cz = cz + 2
                self._pad(cx, cz, cy, hw, hd, top_type)
                self.coins.append((cx + 0.5, cy - climb / 2.0 + 1.4, cz - 1.5))

            elif feat == "spring":
                # erhoehter Sprungblock auf dem aktuellen Pad ...
                self.world[(cx, cy + 1, cz)] = SPRING
                # ... naechstes Pad deutlich hoeher (per Katapult erreichbar)
                lift = rng.choice([3, 4])
                dz = rng.choice([2, 3])
                cy = cy + lift
                cz = cz + dz
                self._pad(cx, cz, cy, hw, hd, top_type)
                self.coins.append((cx + 0.5, cy - lift / 2.0 + 2.0, cz - dz / 2.0))

            else:  # jump / fence
                dz = rng.choice([2, 3] if easy else [3, 4])
                dxr = [-1, 0, 1] if easy else [-2, -1, 0, 1, 2]
                dx = rng.choice(dxr)
                dyr = [-1, 0] if easy else [-2, -1, 0, 1]
                dy = rng.choice(dyr)
                mx, my, mz = cx, cy, cz
                cx = cx + dx
                cy = max(self._min_y - 3, cy + dy)
                cz = cz + dz
                self._pad(cx, cz, cy, hw, hd, top_type)
                # Coin ueber der Luecke
                self.coins.append(((mx + cx) / 2.0 + 0.5,
                                   (my + cy) / 2.0 + 1.7,
                                   (mz + cz) / 2.0 + 0.5))
                if feat == "fence" and not last:
                    self.world[(cx, cy + 1, cz)] = FENCE
            self._min_y = min(self._min_y, cy)

        # Ziel-Beacon auf dem letzten Pad
        self.world[(cx, cy + 1, cz)] = GOAL
        self.goal = (cx + 0.5, cy + 1.0, cz + 0.5)
        self.death_y = self._min_y - 7.0

        # Spieler setzen
        self.px, self.py, self.pz = self.spawn
        self.vx = self.vy = self.vz = 0.0
        self.on_ground = True
        self.on_ladder = False
        self.checkpoint = self.spawn
        self.level_time = 0.0
        self.coins_level = 0
        self._clear_t = 0.0
        self._cam_pos = (self.px, self.py + EYE_H, self.pz)
        self._cam_look = (self.px, self.py + EYE_H, self.pz + 1.0)

    def _pad(self, cx, cz, y, hw, hd, typ):
        for x in range(cx - hw, cx + hw + 1):
            for z in range(cz - hd, cz + hd + 1):
                self.world[(x, y, z)] = typ

    # ===================================================== Blockabfragen
    def _is_solid(self, x, y, z):
        return self.world.get((int(x), int(y), int(z)), EMPTY) in SOLID

    def _cell(self, x, y, z):
        return self.world.get((int(x), int(y), int(z)), EMPTY)

    # ===================================================== Eingabe
    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if self.state in (READY, GAMEOVER):
                if k in ("Return", "space"):
                    self._start_or_restart()
                return
            self.held.add(k)
            if k in ("space", "Up") and k == "space":
                self._jump()
            elif k in ("v", "V"):
                self._toggle_view()
            elif k in ("b", "B"):
                self._cycle_blur()
            elif k in ("c", "C"):
                self.capture_mouse = not self.capture_mouse
            elif k in ("i", "I"):
                self._toggle_invert()
            elif k in ("plus", "KP_Add", "equal"):
                self._change_sens(0.1)
            elif k in ("minus", "KP_Subtract"):
                self._change_sens(-0.1)
        elif event.kind == InputEvent.KEYUP:
            self.held.discard(event.key)
        elif event.kind == InputEvent.MOUSEDOWN:
            if self.state in (READY, GAMEOVER):
                self._start_or_restart()
            elif self.state == PLAY and not self.capture_mouse:
                self.capture_mouse = True
        elif event.kind == InputEvent.MOUSEREL and self.state == PLAY:
            self._apply_look(event.rel)

    def _start_or_restart(self):
        if self.state == GAMEOVER:
            self.game_over = False
            self._new_run()
        self.held.clear()
        self.state = PLAY
        self.level_time = 0.0
        self.capture_mouse = True
        self.play_sound("click")

    def _apply_look(self, rel):
        k = math.radians(DEG_PER_PX) * self.sens
        # s = -1 -> normal (Maus rechts = Blick rechts, Maus hoch = Blick hoch);
        # s = +1 -> invertiert (beide Achsen umgekehrt).
        s = 1.0 if self.invert else -1.0
        self.yaw = (self.yaw + rel[0] * k * s) % math.tau
        self.pitch = max(-PITCH_CLAMP, min(PITCH_CLAMP, self.pitch + rel[1] * k * s))

    def _toggle_view(self):
        self.view = "third" if self.view == "first" else "first"
        self._save_setting("view", self.view)

    def _cycle_blur(self):
        self.blur = 0.0 if self.blur >= 0.79 else round(self.blur + 0.2, 2)
        self._save_setting("blur", self.blur)

    def _toggle_invert(self):
        self.invert = not self.invert
        self._save_setting("mouse_invert", self.invert)
        self.play_sound("click")

    def _change_sens(self, d):
        self.sens = round(max(0.4, min(2.5, self.sens + d)), 1)
        self._save_setting("sens", self.sens)

    def _jump(self):
        if self.state != PLAY:
            return
        if self.on_ground:
            self.vy = JUMP_VEL
            self.on_ground = False
            self.play_sound("click")
        elif self.on_ladder:
            self.vy = JUMP_VEL * 0.85
            self.on_ladder = False
            f = _dir_from(self.yaw, 0.0)
            self.vx = -f[0] * MOVE_SPEED
            self.vz = -f[2] * MOVE_SPEED
            self.play_sound("click")

    # ===================================================== Update / Physik
    def update(self, dt):
        self.anim += dt
        if self.state == CLEAR:
            self._clear_t -= dt
            if self._clear_t <= 0:
                self.level += 1
                self._build_level(self.level)
                self.state = PLAY
            return
        if self.state != PLAY:
            return
        self.level_time += dt
        self._physics(min(dt, 0.05))

    def _held_axis(self, pos, neg):
        p = 1 if self.held & pos else 0
        n = 1 if self.held & neg else 0
        return p - n

    def _physics(self, dt):
        UP = {"w", "W", "Up"}
        DOWN = {"s", "S", "Down"}
        RIGHT = {"d", "D", "Right"}
        LEFT = {"a", "A", "Left"}
        fwd = self._held_axis(UP, DOWN)
        strafe = self._held_axis(RIGHT, LEFT)

        self.on_ladder = self._in_ladder()
        f = _dir_from(self.yaw, 0.0)
        fx, fz = f[0], f[2]
        rx, rz = -fz, fx                       # Kamera-Rechtsvektor (Bildschirm-rechts)

        if self.on_ladder:
            self.vy = CLIMB_SPEED * fwd
            mvx = rx * strafe
            mvz = rz * strafe
            ln = math.hypot(mvx, mvz)
            if ln > 1e-6:
                self.vx = mvx / ln * MOVE_SPEED * 0.7
                self.vz = mvz / ln * MOVE_SPEED * 0.7
            else:
                self.vx = self.vz = 0.0
        else:
            mvx = fx * fwd + rx * strafe
            mvz = fz * fwd + rz * strafe
            ln = math.hypot(mvx, mvz)
            if ln > 1e-6:
                self.vx = mvx / ln * MOVE_SPEED
                self.vz = mvz / ln * MOVE_SPEED
            else:
                self.vx = self.vz = 0.0
            self.vy -= GRAVITY * dt
        self.vy = max(VY_MIN, min(VY_MAX, self.vy))

        self.on_ground = False
        self._land_cells = []
        self._move_axis(0, self.vx * dt)
        self._move_axis(2, self.vz * dt)
        self._move_axis(1, self.vy * dt)

        if self.on_ground:
            bounced = False
            for c in self._land_cells:
                if self.world.get(c) == SPRING:
                    self.vy = SPRING_VEL
                    self.on_ground = False
                    bounced = True
                    self.play_sound("move")
                    break
            if not bounced:
                self.checkpoint = (self.px, self.py, self.pz)

        self._collect_coins()
        if self._at_goal():
            self._level_clear()
            return
        if self.py < self.death_y:
            self._die()

    def _aabb(self):
        return ((self.px - HALF_W, self.py, self.pz - HALF_W),
                (self.px + HALF_W, self.py + PLAYER_H, self.pz + HALF_W))

    def _move_axis(self, axis, delta):
        if axis == 0:
            self.px += delta
        elif axis == 1:
            self.py += delta
        else:
            self.pz += delta
        mn, mx = self._aabb()
        xr = range(int(math.floor(mn[0] + EPS)), int(math.floor(mx[0] - EPS)) + 1)
        yr = range(int(math.floor(mn[1] + EPS)), int(math.floor(mx[1] - EPS)) + 1)
        zr = range(int(math.floor(mn[2] + EPS)), int(math.floor(mx[2] - EPS)) + 1)
        hits = [(x, y, z) for x in xr for y in yr for z in zr if self._is_solid(x, y, z)]
        if not hits:
            return
        if axis == 0:
            if delta > 0:
                self.px = min(h[0] for h in hits) - HALF_W - EPS
            elif delta < 0:
                self.px = max(h[0] for h in hits) + 1 + HALF_W + EPS
            self.vx = 0.0
        elif axis == 2:
            if delta > 0:
                self.pz = min(h[2] for h in hits) - HALF_W - EPS
            elif delta < 0:
                self.pz = max(h[2] for h in hits) + 1 + HALF_W + EPS
            self.vz = 0.0
        else:
            if delta > 0:                        # Kopf stoesst an die Decke
                self.py = min(h[1] for h in hits) - PLAYER_H - EPS
                self.vy = 0.0
            elif delta < 0:                      # Landung auf dem Boden
                top = max(h[1] for h in hits)
                self.py = top + 1 + EPS
                self.vy = 0.0
                self.on_ground = True
                self._land_cells = [h for h in hits if h[1] == top]

    def _in_ladder(self):
        mn, mx = self._aabb()
        for x in range(int(math.floor(mn[0] + EPS)), int(math.floor(mx[0] - EPS)) + 1):
            for y in range(int(math.floor(mn[1] + EPS)), int(math.floor(mx[1] - EPS)) + 1):
                for z in range(int(math.floor(mn[2] + EPS)), int(math.floor(mx[2] - EPS)) + 1):
                    if self._cell(x, y, z) == LADDER:
                        return True
        return False

    def _collect_coins(self):
        remaining = []
        for (cx, cy, cz) in self.coins:
            if (abs(self.px - cx) < 0.85 and abs(self.pz - cz) < 0.85
                    and abs((self.py + 0.9) - cy) < 1.2):
                self.score += 50
                self.coins_level += 1
                self.play_sound("click")
            else:
                remaining.append((cx, cy, cz))
        self.coins = remaining

    def _at_goal(self):
        gx, gy, gz = self.goal
        return (abs(self.px - gx) < 1.3 and abs(self.pz - gz) < 1.3
                and self.py > gy - 1.6)

    def _level_clear(self):
        par = self._plat_count * 3.2
        bonus = max(0, int((par - self.level_time) * 15))
        self.score += 1000 + bonus
        self._last_bonus = bonus
        self.state = CLEAR
        self._clear_t = 1.8
        self.play_sound("win")

    def _die(self):
        self.lives -= 1
        self.play_sound("gameover")
        if self.lives <= 0:
            self.state = GAMEOVER
            self.game_over = True
            self.capture_mouse = False
        else:
            self.px, self.py, self.pz = self.checkpoint
            self.vx = self.vy = self.vz = 0.0
            self.on_ground = True

    # ===================================================== Kamera
    def _update_cam(self):
        head = (self.px, self.py + EYE_H, self.pz)
        f = _dir_from(self.yaw, self.pitch)
        if self.view == "first":
            self._cam_pos = head
            self._cam_look = (head[0] + f[0], head[1] + f[1], head[2] + f[2])
        else:
            look_h = self.py + 1.1
            target = (self.px, look_h, self.pz)
            dist = 4.4
            want = (self.px - f[0] * dist, look_h - f[1] * dist + 0.5,
                    self.pz - f[2] * dist)
            self._cam_pos = self._cam_collide(target, want)
            self._cam_look = target

    def _cam_collide(self, a, b):
        dx, dy, dz = b[0] - a[0], b[1] - a[1], b[2] - a[2]
        steps = 7
        for i in range(1, steps + 1):
            tt = i / steps
            p = (a[0] + dx * tt, a[1] + dy * tt, a[2] + dz * tt)
            if self._is_solid(math.floor(p[0]), math.floor(p[1]), math.floor(p[2])):
                tt = max(0.0, (i - 1) / steps) * 0.92
                return (a[0] + dx * tt, a[1] + dy * tt, a[2] + dz * tt)
        return b

    # ===================================================== 3D-Renderer
    def _view_basis(self):
        cx, cy, cz = self._cam_pos
        fx = self._cam_look[0] - cx
        fy = self._cam_look[1] - cy
        fz = self._cam_look[2] - cz
        fl = math.sqrt(fx * fx + fy * fy + fz * fz) or 1.0
        f = (fx / fl, fy / fl, fz / fl)
        rl = math.hypot(f[2], f[0]) or 1.0
        r = (-f[2] / rl, 0.0, f[0] / rl)
        u = (r[1] * f[2] - r[2] * f[1],
             r[2] * f[0] - r[0] * f[2],
             r[0] * f[1] - r[1] * f[0])
        return r, u, f

    def _to_cam(self, p):
        r, u, f = self._basis
        dx = p[0] - self._cam_pos[0]
        dy = p[1] - self._cam_pos[1]
        dz = p[2] - self._cam_pos[2]
        return (dx * r[0] + dz * r[2],
                dx * u[0] + dy * u[1] + dz * u[2],
                dx * f[0] + dy * f[1] + dz * f[2])

    def _proj(self, c):
        k = self._f / c[2]
        return (self._scx + c[0] * k, self._scy - c[1] * k)

    @staticmethod
    def _clip_near(pts):
        out = []
        n = len(pts)
        for i in range(n):
            a, b = pts[i], pts[(i + 1) % n]
            a_in, b_in = a[2] >= NEAR, b[2] >= NEAR
            if a_in:
                out.append(a)
            if a_in != b_in:
                tt = (NEAR - a[2]) / (b[2] - a[2])
                out.append((a[0] + (b[0] - a[0]) * tt,
                            a[1] + (b[1] - a[1]) * tt, NEAR))
        return out

    @staticmethod
    def _shade(col, k):
        return (min(255, int(col[0] * k)), min(255, int(col[1] * k)),
                min(255, int(col[2] * k)))

    @staticmethod
    def _fog(col, depth):
        t = (depth - FOG_START) / (FOG_END - FOG_START)
        if t <= 0:
            return col
        t = min(1.0, t)
        return (int(col[0] + (COL_FOG[0] - col[0]) * t),
                int(col[1] + (COL_FOG[1] - col[1]) * t),
                int(col[2] + (COL_FOG[2] - col[2]) * t))

    def _add_poly(self, items, world_pts, color, shade=1.0, outline=None):
        cs = [self._to_cam(p) for p in world_pts]
        if all(c[2] < NEAR for c in cs):
            return
        cs = self._clip_near(cs)
        if len(cs) < 3:
            return
        depth = sum(c[2] for c in cs) / len(cs)
        if depth > FOG_END + 6:
            return
        pts = [self._proj(c) for c in cs]
        if (all(p[0] < -40 for p in pts) or all(p[0] > self.width + 40 for p in pts)
                or all(p[1] < -40 for p in pts) or all(p[1] > self.height + 40 for p in pts)):
            return
        col = self._fog(self._shade(color, shade), depth)
        items.append((depth, pts, col, outline if depth < FOG_START + 6 else None))

    def _cull(self, cx, cy, cz):
        """Grobes Frustum-Cull anhand des Wuerfelzentrums (spart Flaechen-Arbeit)."""
        c = self._to_cam((cx, cy, cz))
        if c[2] < -1.7 or c[2] > FOG_END + 2:
            return True
        if abs(c[0]) > c[2] * 1.9 + 3.0:
            return True
        return False

    def _add_prism(self, items, x0, x1, y0, y1, z0, z1, top, side, outline=None):
        px, py, pz = self._cam_pos
        if py > y1:
            self._add_poly(items, ((x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1)),
                           top, 1.0, outline)
        elif py < y0:
            self._add_poly(items, ((x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1)),
                           side, 0.42, outline)
        if px < x0:
            self._add_poly(items, ((x0, y0, z0), (x0, y0, z1), (x0, y1, z1), (x0, y1, z0)),
                           side, 0.60, outline)
        elif px > x1:
            self._add_poly(items, ((x1, y0, z0), (x1, y0, z1), (x1, y1, z1), (x1, y1, z0)),
                           side, 0.60, outline)
        if pz < z0:
            self._add_poly(items, ((x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0)),
                           side, 0.76, outline)
        elif pz > z1:
            self._add_poly(items, ((x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)),
                           side, 0.76, outline)

    def _add_cube(self, items, bx, by, bz, top, side, outline):
        """Voxel bei (bx,by,bz); nur sichtbare Flaechen, deren Nachbar leer ist."""
        px, py, pz = self._cam_pos
        x0, x1 = bx, bx + 1
        y0, y1 = by, by + 1
        z0, z1 = bz, bz + 1
        if py > y1 and not self._is_solid(bx, by + 1, bz):
            self._add_poly(items, ((x0, y1, z0), (x1, y1, z0), (x1, y1, z1), (x0, y1, z1)),
                           top, 1.0, outline)
        elif py < y0 and not self._is_solid(bx, by - 1, bz):
            self._add_poly(items, ((x0, y0, z0), (x1, y0, z0), (x1, y0, z1), (x0, y0, z1)),
                           side, 0.40, outline)
        if px < x0 and not self._is_solid(bx - 1, by, bz):
            self._add_poly(items, ((x0, y0, z0), (x0, y0, z1), (x0, y1, z1), (x0, y1, z0)),
                           side, 0.60, outline)
        elif px > x1 and not self._is_solid(bx + 1, by, bz):
            self._add_poly(items, ((x1, y0, z0), (x1, y0, z1), (x1, y1, z1), (x1, y1, z0)),
                           side, 0.60, outline)
        if pz < z0 and not self._is_solid(bx, by, bz - 1):
            self._add_poly(items, ((x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0)),
                           side, 0.76, outline)
        elif pz > z1 and not self._is_solid(bx, by, bz + 1):
            self._add_poly(items, ((x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)),
                           side, 0.76, outline)

    def _add_octa(self, items, center, r, color):
        cx, cy, cz = center
        top = (cx, cy + r, cz)
        bot = (cx, cy - r, cz)
        a0 = self.anim * 2.4
        eq = [(cx + math.cos(a0 + i * math.pi / 2) * r, cy,
               cz + math.sin(a0 + i * math.pi / 2) * r) for i in range(4)]
        px, py, pz = self._cam_pos
        lx, ly, lz = 0.42, -0.82, 0.39
        for i in range(4):
            for tri in ((top, eq[i], eq[(i + 1) % 4]),
                        (bot, eq[(i + 1) % 4], eq[i])):
                a, b, c = tri
                ux, uy, uz = b[0] - a[0], b[1] - a[1], b[2] - a[2]
                vx, vy, vz = c[0] - a[0], c[1] - a[1], c[2] - a[2]
                nx = uy * vz - uz * vy
                ny = uz * vx - ux * vz
                nz = ux * vy - uy * vx
                fcx, fcy, fcz = (a[0] + b[0] + c[0]) / 3, (a[1] + b[1] + c[1]) / 3, (a[2] + b[2] + c[2]) / 3
                if nx * (fcx - cx) + ny * (fcy - cy) + nz * (fcz - cz) < 0:
                    nx, ny, nz = -nx, -ny, -nz
                if nx * (fcx - px) + ny * (fcy - py) + nz * (fcz - pz) >= 0:
                    continue
                nl = math.sqrt(nx * nx + ny * ny + nz * nz) or 1.0
                lit = max(0.0, -(nx * lx + ny * ly + nz * lz) / nl)
                self._add_poly(items, tri, color, 0.55 + 0.5 * lit)

    def _add_block(self, items, pos, typ):
        bx, by, bz = pos
        if not self._cull(bx + 0.5, by + 0.5, bz + 0.5):
            if typ in BLOCK_COLS:
                top, side = BLOCK_COLS[typ]
                if typ == GOAL:
                    p = 0.5 + 0.5 * math.sin(self.anim * 4)
                    top = ui.mix(top, (255, 255, 255), 0.3 * p)
                self._add_cube(items, bx, by, bz, top, side, tuple(c // 3 for c in side))
            elif typ == LADDER:
                self._add_prism(items, bx + 0.12, bx + 0.88, by, by + 1, bz + 0.05, bz + 0.28,
                                COL_LADDER, tuple(int(c * 0.8) for c in COL_LADDER),
                                (60, 40, 20))
                for ry in range(3):
                    yy = by + 0.2 + ry * 0.3
                    self._add_prism(items, bx + 0.1, bx + 0.9, yy, yy + 0.08,
                                    bz + 0.02, bz + 0.3, (210, 170, 110), (150, 110, 60))
            elif typ == FENCE:
                self._add_prism(items, bx + 0.36, bx + 0.64, by, by + 1, bz + 0.36, bz + 0.64,
                                COL_FENCE, tuple(int(c * 0.8) for c in COL_FENCE),
                                (60, 40, 20))
                self._add_prism(items, bx + 0.05, bx + 0.95, by + 0.55, by + 0.72,
                                bz + 0.42, bz + 0.58, COL_FENCE, (120, 90, 54))
                self._add_prism(items, bx + 0.42, bx + 0.58, by + 0.55, by + 0.72,
                                bz + 0.05, bz + 0.95, COL_FENCE, (120, 90, 54))
            elif typ == SPRING:
                self._add_prism(items, bx + 0.15, bx + 0.85, by, by + 0.55,
                                bz + 0.15, bz + 0.85, COL_SPRING_BASE, (60, 64, 84))
                p = 0.5 + 0.5 * math.sin(self.anim * 6)
                pad = ui.mix(COL_SPRING_PAD, (255, 255, 255), 0.25 * p)
                self._add_prism(items, bx + 0.08, bx + 0.92, by + 0.55, by + 0.78,
                                bz + 0.08, bz + 0.92, pad, tuple(int(c * 0.8) for c in pad),
                                (40, 120, 100))

    def _add_player(self, items):
        px, py, pz = self.px, self.py, self.pz
        self._add_prism(items, px - 0.26, px + 0.26, py + 0.02, py + 1.12,
                        pz - 0.2, pz + 0.2, COL_PLAYER_BODY,
                        tuple(int(c * 0.8) for c in COL_PLAYER_BODY), (20, 30, 70))
        self._add_prism(items, px - 0.23, px + 0.23, py + 1.12, py + 1.56,
                        pz - 0.23, pz + 0.23, COL_PLAYER_HEAD,
                        tuple(int(c * 0.85) for c in COL_PLAYER_HEAD), (90, 60, 40))

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        self.capture_mouse = (self.state == PLAY and not getattr(self, "paused", False))
        self._update_cam()
        self._scx = self.width / 2
        self._scy = self.height * 0.5
        self._f = self.height * FOV_MUL
        self._basis = self._view_basis()

        self._draw_sky(s)

        items = []
        for pos, typ in self.world.items():
            self._add_block(items, pos, typ)
        for coin in self.coins:
            r = 0.28 + 0.05 * math.sin(self.anim * 5 + coin[0] + coin[2])
            self._add_octa(items, (coin[0], coin[1] + 0.1 * math.sin(self.anim * 2 + coin[0]),
                                   coin[2]), r, COL_COIN)
        gx, gy, gz = self.goal
        self._add_octa(items, (gx, gy + 1.2 + 0.12 * math.sin(self.anim * 2), gz),
                       0.34, (150, 245, 200))
        if self.view == "third":
            self._add_player(items)

        items.sort(key=lambda it: -it[0])
        for _, pts, col, outline in items:
            pygame.draw.polygon(s, col, pts)
            if outline:
                pygame.draw.polygon(s, outline, pts, 1)

        self._apply_blur(s)
        if self.view == "first" and self.state == PLAY:
            self._draw_crosshair(s)
        self._draw_hud(s)
        if self.state == READY:
            self._banner(s, t("blj.ready"), self.accent, t("blj.controls"))
        elif self.state == CLEAR:
            self._banner(s, t("blj.clear", n=self.level),
                         (120, 235, 170),
                         t("blj.clear_sub", pts=1000 + getattr(self, "_last_bonus", 0)))
        elif self.state == GAMEOVER:
            self._banner(s, t("blj.gameover"), (232, 96, 96),
                         t("common.points", score=self.score) + "   ·   "
                         + t("common.enter_restart"))

    def _draw_sky(self, s):
        if self._sky_cache is None or self._sky_cache[0] != (self.width, self.height):
            surf = pygame.Surface((self.width, self.height))
            hor = int(self.height * 0.52)
            haze = int(self.height * 0.66)
            for y in range(self.height):
                if y < hor:
                    tt = y / max(1, hor)
                    c = [int(a + (b - a) * tt) for a, b in zip(COL_SKY_TOP, COL_SKY_HOR)]
                elif y < haze:
                    tt = (y - hor) / max(1, haze - hor)
                    c = [int(a + (b - a) * tt) for a, b in zip(COL_SKY_HOR, COL_FOG)]
                else:
                    c = COL_FOG
                pygame.draw.line(surf, c, (0, y), (self.width, y))
            self._sky_cache = ((self.width, self.height), surf)
        s.blit(self._sky_cache[1], (0, 0))

    def _apply_blur(self, s):
        if self.blur <= 0.01:
            self._prev_frame = None
            return
        if self._prev_frame is None or self._prev_frame.get_size() != s.get_size():
            self._prev_frame = s.copy()
            return
        self._prev_frame.set_alpha(int(self.blur * 230))
        s.blit(self._prev_frame, (0, 0))
        self._prev_frame = s.copy()

    def _draw_crosshair(self, s):
        cx, cy = self.width // 2, self.height // 2
        col = (20, 24, 30)
        pygame.draw.line(s, col, (cx - 9, cy), (cx - 3, cy), 2)
        pygame.draw.line(s, col, (cx + 3, cy), (cx + 9, cy), 2)
        pygame.draw.line(s, col, (cx, cy - 9), (cx, cy - 3), 2)
        pygame.draw.line(s, col, (cx, cy + 3), (cx, cy + 9), 2)
        pygame.draw.circle(s, col, (cx, cy), 2)

    def _draw_hud(self, s):
        chip = pygame.Surface((self.width, 34), pygame.SRCALPHA)
        chip.fill((12, 16, 24, 120))
        s.blit(chip, (0, 0))
        img = self._hud.render(t("blj.level", n=self.level), True, self.accent)
        s.blit(img, img.get_rect(midleft=(14, 17)))
        img = self._small.render(t("common.points", score=self.score), True, ui.TEXT)
        s.blit(img, img.get_rect(center=(self.width // 2, 17)))
        img = self._small.render(
            t("blj.coins", n=self.coins_level) + "   " + t("blj.lives", n=self.lives),
            True, ui.GOLD)
        s.blit(img, img.get_rect(midright=(self.width - 14, 17)))
        if self.state == PLAY:
            mdir = t("common.dir_inverted") if self.invert else t("common.dir_normal")
            hint = t("blj.hud_hint",
                     view=(t("blj.view_1p") if self.view == "first" else t("blj.view_3p")),
                     dir=mdir)
            img = self._small.render(hint, True, ui.TEXT_FAINT)
            s.blit(img, img.get_rect(midbottom=(self.width // 2, self.height - 8)))

    def _banner(self, s, title, color, sub):
        w = min(self.width - 40, 560)
        h = 118
        rc = pygame.Rect((self.width - w) // 2, (self.height - h) // 2, w, h)
        panel = pygame.Surface((w, h), pygame.SRCALPHA)
        panel.fill((14, 18, 26, 232))
        s.blit(panel, rc.topleft)
        pygame.draw.rect(s, color, rc, 2, border_radius=14)
        img = self._huge.render(title, True, color)
        s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 42)))
        img = self._small.render(sub, True, ui.TEXT)
        s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 84)))
