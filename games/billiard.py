# -*- coding: utf-8 -*-
"""
billiard.py
===========
Billard / Pool - 8-Ball, 9-Ball und ein regelfreier Übungsmodus, gegen die KI
oder zu zweit lokal, in drei frei wählbaren Ansichten.

Ansichten (im Setup UND per Taste V umschaltbar, wird gespeichert):
  - 2D      : klassische Draufsicht von oben.
  - 3D      : feste perspektivische Schrägansicht mit schattierten Kugeln.
  - Frei    : wie 3D, aber die Kamera lässt sich mit der rechten Maustaste
              (oder Q/E) sanft um den Tisch drehen.

Alle Bewegungen sind zeitschritt-basiert und weich abgebremst (Reibung), damit
nichts ruckt oder springt. Die Physik läuft in Teilschritten, um bei schnellen
Kugeln kein "Durchtunneln" zuzulassen.

Steuerung: Maus bewegt das Ziel, linke Maustaste gedrückt halten lädt die
Stoßstärke, Loslassen stößt. Alternativ: Pfeile links/rechts zielen, hoch/runter
Stärke, Leertaste stößt. Bei Ball-in-Hand (nach Foul) die weiße Kugel mit der
Maus platzieren. V = Ansicht, nach Spielende Enter = neue Partie.

Punkte (Highscore) = gewonnene Frames gegen die KI (bzw. versenkte Kugeln im
Übungsmodus).
"""

import math
import random

import pygame

import settings as settings_mod
import ui
from game_base import Game, InputEvent, LocalizedName
from i18n import t

# ------------------------------------------------- Identitätsfarben (Tisch)
# Filz, Banden, Taschen, Queue & Kugeln bleiben bewusst fest - alle
# generischen UI-Farben kommen zur Laufzeit dynamisch aus der ui-Palette.
COL_CLOTH = (26, 112, 72)
COL_CLOTH_D = (20, 92, 58)
COL_RAIL = (74, 48, 30)
COL_RAIL_HI = (104, 70, 44)
COL_POCKET = (10, 12, 10)
COL_CUE = (245, 245, 240)
COL_AIM = (240, 240, 200)
COL_STICK = (208, 168, 96)
COL_STICK_D = (150, 116, 60)

# Kugelfarben nach Nummer (1..15). 8 = schwarz.
BALL_COLORS = {
    1: (232, 190, 40), 2: (40, 84, 180), 3: (200, 48, 48), 4: (120, 60, 160),
    5: (224, 120, 40), 6: (34, 130, 80), 7: (150, 44, 52), 8: (26, 26, 30),
    9: (232, 190, 40), 10: (40, 84, 180), 11: (200, 48, 48), 12: (120, 60, 160),
    13: (224, 120, 40), 14: (34, 130, 80), 15: (150, 44, 52),
}

# ----------------------------------------------------------------- Physik / Tisch
HW, HH = 127.0, 63.5             # halbe Tischmaße (Tisch-Koordinaten, zentriert)
BR = 3.1                         # Kugelradius
POCKET_R = 6.4
FRICTION = 0.62                  # Rollreibung pro Sekunde
WALL_E = 0.90                    # Bandenrestitution
BALL_E = 0.94                    # Kugel-Kugel-Restitution
STOP_EPS = 2.0                   # darunter gilt eine Kugel als still
MAX_SPEED = 470.0                # maximale Stoßgeschwindigkeit
MAX_SHOT_TIME = 12.0

POCKETS = [(-HW, -HH), (0, -HH), (HW, -HH),
           (-HW, HH), (0, HH), (HW, HH)]

SETUP, PLAY, OVER = "setup", "play", "over"
VARIANTS = ["8ball", "9ball", "practice"]
VIEWS = ["2d", "3d", "free"]
DIFFS = ["easy", "medium", "hard"]
AI_ACC = [0.80, 0.91, 0.985]


class _Ball:
    __slots__ = ("x", "y", "vx", "vy", "num", "potted")

    def __init__(self, x, y, num):
        self.x = x
        self.y = y
        self.vx = 0.0
        self.vy = 0.0
        self.num = num
        self.potted = False

    def speed2(self):
        return self.vx * self.vx + self.vy * self.vy


class BilliardGame(Game):
    name = LocalizedName("Billiards", de="Billard", fr="Billard",
                         es="Billar", pt="Bilhar")
    highscore_key = "billiard"
    supports_multiplayer = True
    wants_right_click = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        bs = self.settings.get("billiard", {}) if isinstance(self.settings, dict) else {}
        self.variant = bs.get("variant", "8ball")
        if self.variant not in VARIANTS:
            self.variant = "8ball"
        self.view = bs.get("view", "2d")
        if self.view not in VIEWS:
            self.view = "2d"
        self.diff = max(0, min(2, int(bs.get("difficulty", 1))))

        self._build_fonts()
        self._over_cache = None
        self.wins = [0, 0]
        self.cam_yaw = 0.0
        self.cam_yaw_t = 0.0
        self._dragging_cam = False
        self._build_setup_layout()
        self._setup_camera()
        self._new_rack()
        self.state = SETUP

    def _build_fonts(self):
        """Schriftgrößen aus der aktuellen Auflösung ableiten (Theme-Fonts)."""
        h = self.height
        self._small = ui.font(max(14, h // 32))
        self._tiny = ui.font(max(11, h // 40))
        self._huge = ui.font(max(26, h // 12), bold=True)

    def on_surface_changed(self):
        self._build_fonts()
        self._over_cache = None
        self._build_setup_layout()
        self._setup_camera()

    # ------------------------------------------------ Kamera / Projektion
    def _setup_camera(self):
        self.hud_h = 46
        self.cx = self.width / 2.0
        self.cy = self.hud_h + (self.height - self.hud_h) * 0.5
        # 2D-Maßstab
        self.scale2d = min((self.width - 46) / (2 * HW),
                           (self.height - self.hud_h - 40) / (2 * HH))
        # 3D-Pinhole
        self.focal = self.width * 0.92
        self.cam_R = HW * 2.35
        self.cam_EL = math.radians(42)
        self.cy3d = self.hud_h + (self.height - self.hud_h) * 0.54
        self._basis = self._cam_basis()

    def _cam_basis(self):
        yaw = self.cam_yaw
        ce, se = math.cos(self.cam_EL), math.sin(self.cam_EL)
        cyw, syw = math.cos(yaw), math.sin(yaw)
        cam = (self.cam_R * ce * syw, -self.cam_R * ce * cyw, self.cam_R * se)
        # forward = normalize(target - cam) ; target = origin
        fx, fy, fz = -cam[0], -cam[1], -cam[2]
        fl = math.sqrt(fx * fx + fy * fy + fz * fz) or 1.0
        fx, fy, fz = fx / fl, fy / fl, fz / fl
        # right = normalize(forward x up), up_world=(0,0,1)
        rx, ry, rz = fy * 1 - fz * 0, fz * 0 - fx * 1, fx * 0 - fy * 0
        rl = math.sqrt(rx * rx + ry * ry + rz * rz) or 1.0
        rx, ry, rz = rx / rl, ry / rl, rz / rl
        # up = right x forward
        ux, uy, uz = (ry * fz - rz * fy, rz * fx - rx * fz, rx * fy - ry * fx)
        return cam, (fx, fy, fz), (rx, ry, rz), (ux, uy, uz)

    def _project(self, x, y, z=0.0):
        """(x,y) Tisch-Koordinaten -> (sx, sy, scale, depth)."""
        if self.view == "2d":
            return (self.cx + x * self.scale2d, self.cy + y * self.scale2d,
                    self.scale2d, y)
        cam, f, r, u = self._basis
        relx, rely, relz = x - cam[0], y - cam[1], z - cam[2]
        zc = relx * f[0] + rely * f[1] + relz * f[2]
        if zc < 1.0:
            zc = 1.0
        xc = relx * r[0] + rely * r[1] + relz * r[2]
        yc = relx * u[0] + rely * u[1] + relz * u[2]
        sx = self.cx + self.focal * xc / zc
        sy = self.cy3d - self.focal * yc / zc
        return (sx, sy, self.focal / zc, zc)

    def _unproject(self, sx, sy):
        """Bildschirm -> (x,y) auf der Tischebene z=0."""
        if self.view == "2d":
            return ((sx - self.cx) / self.scale2d, (sy - self.cy) / self.scale2d)
        cam, f, r, u = self._basis
        dxc = (sx - self.cx) / self.focal
        dyc = -(sy - self.cy3d) / self.focal
        # Strahlrichtung in Weltkoordinaten
        dx = dxc * r[0] + dyc * u[0] + f[0]
        dy = dxc * r[1] + dyc * u[1] + f[1]
        dz = dxc * r[2] + dyc * u[2] + f[2]
        if abs(dz) < 1e-6:
            return (0.0, 0.0)
        s = -cam[2] / dz
        return (cam[0] + s * dx, cam[1] + s * dy)

    # ------------------------------------------------ Rack aufbauen
    def _new_rack(self):
        self.balls = []
        self.cue = _Ball(-HW * 0.5, 0.0, 0)
        self.balls.append(self.cue)
        fx = HW * 0.45
        sp = 2 * BR + 0.25
        dx = sp * 0.87
        if self.variant == "9ball":
            nums = [1, 2, 3, 4, 5, 6, 7, 8, 9]
            layout = [(0, 0), (1, -1), (1, 1), (2, -2), (2, 0), (2, 2),
                      (3, -1), (3, 1), (4, 0)]
            order = [1] + random.sample([2, 3, 4, 5, 6, 7, 8], 7) + [9]
            for (row, off), num in zip(
                    [(0, 0), (1, -0.5), (1, 0.5), (2, -1), (2, 0), (2, 1),
                     (3, -0.5), (3, 0.5), (4, 0)], order):
                x = fx + row * dx
                y = off * sp
                self.balls.append(_Ball(x, y, num))
        else:
            # 15er-Dreieck (8-Ball / Übung)
            if self.variant == "8ball":
                others = [n for n in range(1, 16) if n != 8]
                random.shuffle(others)
                # 8 in die Mitte der dritten Reihe
                seq = []
                idx = 0
                for row in range(5):
                    for k in range(row + 1):
                        if row == 2 and k == 1:
                            seq.append(8)
                        else:
                            seq.append(others[idx])
                            idx += 1
            else:
                seq = list(range(1, 16))
                random.shuffle(seq)
            i = 0
            for row in range(5):
                for k in range(row + 1):
                    x = fx + row * dx
                    y = (k - row / 2.0) * sp
                    self.balls.append(_Ball(x, y, seq[i]))
                    i += 1
        self.group = [None, None]
        self.current = 0
        self.ball_in_hand = False
        self.break_done = False
        self.winner = None
        self.phase = "aim"
        self.aim = 0.0
        self.power = 0.35
        self.charging = False
        self.shot_time = 0.0
        self.first_hit = None
        self.cue_potted = False
        self.potted_shot = []
        self.potted_all = []
        self.msg = None
        self.msg_t = 0.0
        self.ai_delay = 0.8

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(380, self.width - 50)
        y0 = int(self.height * 0.26)
        gap = 8

        def row(y, n):
            cw = (bw - gap * (n - 1)) / n
            return [pygame.Rect(int(cx - bw / 2 + i * (cw + gap)), y,
                                int(cw), 42) for i in range(n)]

        self.var_rects = row(y0, 3)
        self.view_rects = row(y0 + 84, 3)
        self.diff_rects = row(y0 + 168, 3)
        self.start_rect = pygame.Rect(cx - 95, y0 + 228, 190, 46)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("billiard", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3"):
                self.variant = VARIANTS[int(k) - 1]
                self._save_setting("variant", self.variant)
                self.play_sound("click")
            elif k in ("v", "V"):
                self._cycle_view()
            elif k in ("d", "D"):
                self.diff = (self.diff + 1) % 3
                self._save_setting("difficulty", self.diff)
                self.play_sound("select")
            elif k in ("Return", "space"):
                self._start_play()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.var_rects):
                if rc.collidepoint(event.pos):
                    self.variant = VARIANTS[i]
                    self._save_setting("variant", self.variant)
                    self.play_sound("click")
                    return
            for i, rc in enumerate(self.view_rects):
                if rc.collidepoint(event.pos):
                    self.view = VIEWS[i]
                    self.cam_yaw = self.cam_yaw_t = 0.0
                    self._save_setting("view", self.view)
                    self.play_sound("select")
                    return
            for i, rc in enumerate(self.diff_rects):
                if rc.collidepoint(event.pos):
                    self.diff = i
                    self._save_setting("difficulty", i)
                    self.play_sound("select")
                    return
            if self.start_rect.collidepoint(event.pos):
                self._start_play()

    def _cycle_view(self):
        self.view = VIEWS[(VIEWS.index(self.view) + 1) % 3]
        self._dragging_cam = False
        if self.view != "free":
            self.cam_yaw_t = 0.0
        self._save_setting("view", self.view)
        self.play_sound("select")

    def _start_play(self):
        self._setup_camera()
        self._new_rack()
        self.state = PLAY
        self.play_sound("click")

    # ===================================================== Eingabe
    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN and event.key in ("v", "V") \
                and self.state == PLAY:
            self._cycle_view()
            return
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.state == OVER:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._restart()
                elif event.key in ("s", "S"):
                    self.state = SETUP
                    self.game_over = False
                    self.play_sound("click")
            elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
                self._restart()
            return
        if self.state != PLAY:
            return
        # Kamera drehen: rechte Maustaste HALTEN und Maus bewegen (nur Frei-
        # Ansicht). Loslassen der rechten Taste beendet das Drehen wieder.
        if self.view == "free":
            if event.kind == InputEvent.MOUSEDOWN and event.button == 3:
                self._dragging_cam = True
                self._last_mx = event.pos[0]
                return
            if event.kind == InputEvent.MOUSEUP and event.button == 3:
                self._dragging_cam = False
                return
            if self._dragging_cam:
                if event.kind == InputEvent.MOUSEMOVE:
                    dx = event.pos[0] - getattr(self, "_last_mx", event.pos[0])
                    self._last_mx = event.pos[0]
                    self.cam_yaw_t += dx * 0.006
                    return
                # Sicherheitsnetz: sollte das Loslassen der rechten Taste einmal
                # ausbleiben, beendet JEDE andere Aktion (Linksklick, Taste) das
                # Drehen - danach normal weiterverarbeiten.
                if event.kind in (InputEvent.MOUSEDOWN, InputEvent.MOUSEUP,
                                  InputEvent.KEYDOWN):
                    self._dragging_cam = False
        elif self._dragging_cam:
            self._dragging_cam = False
        if not self._human_turn():
            return
        if self.phase == "place":
            self._handle_place(event)
            return
        if self.phase != "aim":
            return
        self._handle_aim(event)

    def _handle_aim(self, event):
        if event.kind == InputEvent.MOUSEMOVE:
            mx, my = self._unproject(*event.pos)
            self.aim = math.atan2(my - self.cue.y, mx - self.cue.x)
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            self.charging = True
            self.power = 0.05
        elif event.kind == InputEvent.MOUSEUP and event.button == 1:
            if self.charging:
                self.charging = False
                self._strike()
        elif event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("Left", "a", "A"):
                self.aim -= math.radians(2)
            elif k in ("Right", "d", "D"):
                self.aim += math.radians(2)
            elif k in ("Up", "w", "W"):
                self.power = min(1.0, self.power + 0.05)
            elif k in ("Down", "s", "S"):
                self.power = max(0.05, self.power - 0.05)
            elif k in ("space", "Return"):
                self._strike()

    def _handle_place(self, event):
        if event.kind in (InputEvent.MOUSEMOVE, InputEvent.MOUSEDOWN):
            mx, my = self._unproject(*event.pos)
            mx = max(-HW + BR, min(HW - BR, mx))
            my = max(-HH + BR, min(HH - BR, my))
            if self._place_free(mx, my):
                self.cue.x, self.cue.y = mx, my
            if event.kind == InputEvent.MOUSEDOWN and event.button == 1 \
                    and self._place_free(self.cue.x, self.cue.y):
                self.ball_in_hand = False
                self.phase = "aim"
                self.play_sound("click")

    def _place_free(self, x, y):
        for b in self.balls:
            if b is self.cue or b.potted:
                continue
            if (b.x - x) ** 2 + (b.y - y) ** 2 < (2 * BR) ** 2:
                return False
        return True

    def _human_turn(self):
        return self.multiplayer or self.current == 0

    # ===================================================== Stoß / Physik
    def _strike(self):
        sp = MAX_SPEED * self.power
        self.cue.vx = math.cos(self.aim) * sp
        self.cue.vy = math.sin(self.aim) * sp
        self.phase = "rolling"
        self.shot_time = 0.0
        self.first_hit = None
        self.cue_potted = False
        self.potted_shot = []
        self.play_sound("shoot")
        self.rumble(60)

    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None
        # Kamera weich nachführen
        self.cam_yaw += (self.cam_yaw_t - self.cam_yaw) * min(1.0, dt * 8)
        if self.view != "2d":
            self._basis = self._cam_basis()
        if self.state != PLAY:
            return
        if self.phase == "rolling":
            self._physics(dt)
            self.shot_time += dt
            if self._all_stopped() or self.shot_time > MAX_SHOT_TIME:
                self._resolve_shot()
        elif self.phase == "aim":
            if self.charging:
                self.power = min(1.0, self.power + dt * 0.85)
            if not self.multiplayer and self.current == 1:
                self.ai_delay -= dt
                if self.ai_delay <= 0:
                    self._ai_shoot()

    def _all_stopped(self):
        for b in self.balls:
            if not b.potted and b.speed2() > STOP_EPS * STOP_EPS:
                return False
        return True

    def _physics(self, dt):
        # Teilschritte je nach Höchstgeschwindigkeit (kein Durchtunneln)
        vmax = 0.0
        for b in self.balls:
            if not b.potted:
                vmax = max(vmax, b.speed2())
        vmax = math.sqrt(vmax)
        steps = max(3, min(20, int(vmax * dt / BR) + 2))
        h = dt / steps
        fr = max(0.0, 1.0 - FRICTION * h)
        for _ in range(steps):
            for b in self.balls:
                if b.potted:
                    continue
                b.x += b.vx * h
                b.y += b.vy * h
                b.vx *= fr
                b.vy *= fr
                if b.speed2() < STOP_EPS * STOP_EPS:
                    b.vx = b.vy = 0.0
            self._walls()
            self._collisions()
            self._pockets()

    def _walls(self):
        for b in self.balls:
            if b.potted:
                continue
            if b.x < -HW + BR:
                b.x = -HW + BR
                b.vx = -b.vx * WALL_E
            elif b.x > HW - BR:
                b.x = HW - BR
                b.vx = -b.vx * WALL_E
            if b.y < -HH + BR:
                b.y = -HH + BR
                b.vy = -b.vy * WALL_E
            elif b.y > HH - BR:
                b.y = HH - BR
                b.vy = -b.vy * WALL_E

    def _collisions(self):
        bs = [b for b in self.balls if not b.potted]
        n = len(bs)
        for i in range(n):
            a = bs[i]
            for j in range(i + 1, n):
                c = bs[j]
                dx = c.x - a.x
                dy = c.y - a.y
                d2 = dx * dx + dy * dy
                if d2 >= (2 * BR) ** 2 or d2 <= 1e-9:
                    if d2 <= 1e-9:
                        dx, dy, d2 = 0.01, 0.0, 0.0001
                    else:
                        continue
                dist = math.sqrt(d2)
                nx, ny = dx / dist, dy / dist
                overlap = 2 * BR - dist
                a.x -= nx * overlap / 2
                a.y -= ny * overlap / 2
                c.x += nx * overlap / 2
                c.y += ny * overlap / 2
                rvx = a.vx - c.vx
                rvy = a.vy - c.vy
                vn = rvx * nx + rvy * ny
                if vn > 0:
                    jimp = (1 + BALL_E) * vn / 2
                    a.vx -= jimp * nx
                    a.vy -= jimp * ny
                    c.vx += jimp * nx
                    c.vy += jimp * ny
                    if self.first_hit is None and (a.num == 0 or c.num == 0):
                        self.first_hit = c.num if a.num == 0 else a.num
                        self.play_sound("bounce")

    def _pockets(self):
        for b in self.balls:
            if b.potted:
                continue
            for (px, py) in POCKETS:
                if (b.x - px) ** 2 + (b.y - py) ** 2 <= POCKET_R ** 2:
                    b.potted = True
                    b.vx = b.vy = 0.0
                    if b.num == 0:
                        self.cue_potted = True
                    else:
                        self.potted_shot.append(b.num)
                        self.potted_all.append(b.num)
                    self.play_sound("point")
                    break

    # ===================================================== Zugauflösung
    def _object_balls_left(self, lo=1, hi=15):
        return [b.num for b in self.balls
                if not b.potted and b.num != 0 and lo <= b.num <= hi]

    def _resolve_shot(self):
        self.break_done = True
        if self.variant == "practice":
            self._resolve_practice()
        elif self.variant == "9ball":
            self._resolve_9ball()
        else:
            self._resolve_8ball()
        # Weiße neu einsetzen, falls versenkt
        if self.cue_potted and self.state == PLAY:
            self.cue.potted = False
            self.cue.vx = self.cue.vy = 0.0
            if not self.ball_in_hand:
                self._respot_cue()
        if self.state == PLAY:
            if self.ball_in_hand and not self._human_turn():
                self._ai_place()
            if self.ball_in_hand and self._human_turn():
                self.phase = "place"
            else:
                self.phase = "aim"
                self.power = 0.35
                if not self.multiplayer and self.current == 1:
                    self.ai_delay = 0.7

    def _respot_cue(self):
        for x in [-HW * 0.5, -HW * 0.6, -HW * 0.4, -HW * 0.7, 0]:
            if self._place_free(x, 0):
                self.cue.x, self.cue.y = x, 0
                return
        self.cue.x, self.cue.y = -HW * 0.5, 0

    def _foul(self, msg_key):
        self.msg = t(msg_key)
        self.msg_t = 2.2
        self.ball_in_hand = True
        self.current = 1 - self.current
        self.play_sound("hit")

    def _resolve_practice(self):
        self.score = len(self.potted_all)
        if self.cue_potted:
            self.ball_in_hand = False   # automatisch neu einsetzen
        if not self._object_balls_left():
            self._new_rack_keep_score()

    def _new_rack_keep_score(self):
        sc = self.score
        self._new_rack()
        self.state = PLAY
        self.score = sc
        self.msg = t("bil.reracked")
        self.msg_t = 2.0

    def _resolve_8ball(self):
        cur = self.current
        opp = 1 - cur
        potted = self.potted_shot
        foul = self.cue_potted or self.first_hit is None
        # Falsche zuerst getroffene Kugel = Foul (wenn Gruppe zugewiesen)
        if self.group[cur] is not None and self.first_hit is not None:
            legal_first = self._group_nums(self.group[cur])
            if not self._group_cleared(cur):
                if self.first_hit not in legal_first:
                    foul = True
            else:
                if self.first_hit != 8:
                    foul = True
        # Schwarze versenkt?
        if 8 in potted:
            cleared = self._group_cleared(cur)
            if self.group[cur] is None or not cleared or self.cue_potted:
                self.winner = opp
            else:
                self.winner = cur
            self._end()
            return
        # Gruppen zuweisen (nach dem Break, erste saubere versenkte Kugel)
        obj = [p for p in potted if p != 8]
        if self.group[cur] is None and obj and not foul:
            first = obj[0]
            if first <= 7:
                self.group[cur], self.group[opp] = "solid", "stripe"
            else:
                self.group[cur], self.group[opp] = "stripe", "solid"
        made_own = False
        if self.group[cur] is not None:
            made_own = any(p in self._group_nums(self.group[cur]) for p in obj)
        elif obj:
            made_own = True   # offener Tisch nach Break
        if foul:
            self._foul("bil.foul")
        elif made_own:
            pass              # gleicher Spieler weiter
        else:
            self.current = opp

    def _group_nums(self, group):
        return range(1, 8) if group == "solid" else range(9, 16)

    def _group_cleared(self, player):
        g = self.group[player]
        if g is None:
            return False
        lo, hi = (1, 7) if g == "solid" else (9, 15)
        return not self._object_balls_left(lo, hi)

    def _resolve_9ball(self):
        cur = self.current
        opp = 1 - cur
        left_before = self._lowest_before()
        foul = self.cue_potted or self.first_hit is None
        if self.first_hit is not None and left_before is not None \
                and self.first_hit != left_before:
            foul = True
        if 9 in self.potted_shot:
            if not foul:
                self.winner = cur
                self._end()
                return
            else:
                # 9 wieder einsetzen
                for b in self.balls:
                    if b.num == 9:
                        b.potted = True
                self._respot_9()
        if foul:
            self._foul("bil.foul")
        elif self.potted_shot:
            pass
        else:
            self.current = opp

    def _lowest_before(self):
        # niedrigste Kugel, die vor dem Stoß auf dem Tisch war
        nums = [b.num for b in self.balls if b.num != 0
                and (not b.potted or b.num in self.potted_shot)]
        return min(nums) if nums else None

    def _respot_9(self):
        for b in self.balls:
            if b.num == 9:
                b.potted = False
                b.x, b.y = HW * 0.45, 0.0
                if not self._place_free(b.x, b.y):
                    b.x = HW * 0.55
                if 9 in self.potted_all:
                    self.potted_all.remove(9)

    def _ai_place(self):
        # KI setzt Weiße hinter die niedrigste/eigene Zielkugel bzw. in die Mitte
        target = self._ai_target_ball()
        placed = False
        if target is not None:
            for off in (BR * 5, BR * 8, BR * 12):
                x = target.x - HW * 0.02 - off
                if -HW + BR < x < HW - BR and self._place_free(x, target.y):
                    self.cue.x, self.cue.y = x, target.y
                    placed = True
                    break
        if not placed:
            self._respot_cue()
        self.ball_in_hand = False

    def _end(self):
        self.state = OVER
        if self.winner is not None:
            self.wins[self.winner] += 1
            if not self.multiplayer:
                if self.winner == 0:
                    self.score = self.wins[0]
                    self.play_sound("win")
                    self.report_result(True)
                else:
                    self.play_sound("gameover")
                    self.report_result(False)
            else:
                self.play_sound("win")
        self.game_over = True

    def _restart(self):
        self.game_over = False
        self._new_rack()
        self.state = PLAY
        self.play_sound("click")

    # ===================================================== KI
    def _ai_legal_targets(self):
        if self.variant == "9ball":
            low = self._lowest_before()
            return [b for b in self.balls if not b.potted and b.num == low]
        if self.variant == "practice":
            return [b for b in self.balls if not b.potted and b.num != 0]
        g = self.group[self.current]
        if g is None:
            return [b for b in self.balls if not b.potted and b.num not in (0, 8)]
        if self._group_cleared(self.current):
            return [b for b in self.balls if not b.potted and b.num == 8]
        lo, hi = (1, 7) if g == "solid" else (9, 15)
        return [b for b in self.balls if not b.potted and lo <= b.num <= hi]

    def _ai_target_ball(self):
        ts = self._ai_legal_targets()
        if not ts:
            return None
        return min(ts, key=lambda b: (b.x - self.cue.x) ** 2
                   + (b.y - self.cue.y) ** 2)

    def _ai_shoot(self):
        best = None
        for tb in self._ai_legal_targets():
            for (px, py) in POCKETS:
                dpx, dpy = px - tb.x, py - tb.y
                dl = math.hypot(dpx, dpy) or 1.0
                ghost_x = tb.x - dpx / dl * 2 * BR
                ghost_y = tb.y - dpy / dl * 2 * BR
                aim = math.atan2(ghost_y - self.cue.y, ghost_x - self.cue.x)
                # Schnittwinkel: Cue->Ziel gegen Ziel->Tasche
                a1 = math.atan2(tb.y - self.cue.y, tb.x - self.cue.x)
                cut = abs((aim - a1 + math.pi) % (2 * math.pi) - math.pi)
                if cut > math.radians(80):
                    continue
                if self._blocked(self.cue.x, self.cue.y, ghost_x, ghost_y, tb):
                    continue
                dist = math.hypot(ghost_x - self.cue.x, ghost_y - self.cue.y) \
                    + math.hypot(dpx, dpy)
                score = cut * 60 + dist * 0.2
                if best is None or score < best[0]:
                    best = (score, aim, dist)
        if best is None:
            tb = self._ai_target_ball()
            if tb is None:
                self.aim = random.uniform(0, math.tau)
                self.power = 0.4
            else:
                self.aim = math.atan2(tb.y - self.cue.y, tb.x - self.cue.x)
                self.power = 0.5
        else:
            _, aim, dist = best
            acc = AI_ACC[self.diff]
            self.aim = aim + random.uniform(-1, 1) * (1 - acc) * 0.28
            self.power = max(0.3, min(0.92, 0.32 + dist / (2 * HW) * 0.6))
        self._strike()

    def _blocked(self, x0, y0, x1, y1, ignore):
        dx, dy = x1 - x0, y1 - y0
        seg2 = dx * dx + dy * dy
        if seg2 < 1e-6:
            return False
        for b in self.balls:
            if b is self.cue or b is ignore or b.potted:
                continue
            t_ = ((b.x - x0) * dx + (b.y - y0) * dy) / seg2
            if t_ <= 0.02 or t_ >= 0.98:
                continue
            cx = x0 + t_ * dx
            cy = y0 + t_ * dy
            if (b.x - cx) ** 2 + (b.y - cy) ** 2 < (2 * BR) ** 2:
                return True
        return False

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        if self.state == SETUP:
            self._draw_setup(s)
            return
        if self.view != "2d":
            self._basis = self._cam_basis()
        self._draw_table(s)
        self._draw_balls(s)
        if self.phase in ("aim", "place") and self._human_turn():
            self._draw_aim(s)
        self._draw_hud(s)
        if self.state == OVER:
            self._draw_over(s)

    def _corners(self, e):
        return [(-HW - e, -HH - e), (HW + e, -HH - e),
                (HW + e, HH + e), (-HW - e, HH + e)]

    def _draw_table(self, s):
        rail = [self._project(x, y)[:2] for (x, y) in self._corners(BR * 2.4)]
        bed = [self._project(x, y)[:2] for (x, y) in self._corners(0)]
        pygame.draw.polygon(s, COL_RAIL, rail)
        pygame.draw.polygon(s, COL_RAIL_HI, rail, 3)
        pygame.draw.polygon(s, COL_CLOTH, bed)
        # leichte Feld-Schattierung
        pygame.draw.polygon(s, COL_CLOTH_D, bed, 2)
        for (px, py) in POCKETS:
            sx, sy, sc, _ = self._project(px, py)
            pygame.draw.circle(s, COL_POCKET, (int(sx), int(sy)),
                               max(4, int(POCKET_R * sc)))

    def _draw_balls(self, s):
        order = sorted([b for b in self.balls if not b.potted],
                       key=lambda b: self._project(b.x, b.y)[3], reverse=True)
        z = BR if self.view != "2d" else 0.0
        for b in order:
            # Schatten
            sxs, sys_, scs, _ = self._project(b.x, b.y, 0.0)
            if self.view != "2d":
                sh = pygame.Surface((int(BR * scs * 2.2), int(BR * scs * 1.4)),
                                    pygame.SRCALPHA)
                pygame.draw.ellipse(sh, (0, 0, 0, 90), sh.get_rect())
                s.blit(sh, (sxs - BR * scs * 1.1, sys_ - BR * scs * 0.7))
            sx, sy, sc, _ = self._project(b.x, b.y, z)
            r = max(3, int(BR * sc))
            self._draw_ball(s, b, int(sx), int(sy), r)
        # Weiße beim Platzieren blinkend andeuten
        if self.phase == "place" and self._human_turn():
            sx, sy, sc, _ = self._project(self.cue.x, self.cue.y, z)
            k = 0.5 + 0.5 * math.sin(pygame.time.get_ticks() / 150.0)
            pygame.draw.circle(s, (int(120 + 120 * k),) * 3, (int(sx), int(sy)),
                               int(BR * sc) + 3, 2)

    def _draw_ball(self, s, b, x, y, r):
        if b.num == 0:
            pygame.draw.circle(s, COL_CUE, (x, y), r)
        elif 9 <= b.num <= 15:
            # Halbe: weiße Kugel mit farbigem Streifen (auf Kreis maskiert)
            base = BALL_COLORS.get(b.num, (200, 200, 200))
            d = 2 * r
            surf = pygame.Surface((d, d), pygame.SRCALPHA)
            pygame.draw.circle(surf, (245, 245, 240), (r, r), r)
            band_h = max(2, r)
            pygame.draw.rect(surf, base, (0, r - band_h // 2, d, band_h))
            mask = pygame.Surface((d, d), pygame.SRCALPHA)
            pygame.draw.circle(mask, (255, 255, 255, 255), (r, r), r)
            surf.blit(mask, (0, 0), special_flags=pygame.BLEND_RGBA_MULT)
            s.blit(surf, (x - r, y - r))
        else:
            base = BALL_COLORS.get(b.num, (200, 200, 200))
            pygame.draw.circle(s, base, (x, y), r)
        # Glanzlicht
        pygame.draw.circle(s, (255, 255, 255), (x - r // 3, y - r // 3),
                           max(1, r // 4))
        if b.num and r >= 7:
            col = (20, 20, 20) if b.num != 8 else (235, 235, 235)
            img = self._tiny.render(str(b.num), True, col)
            s.blit(img, img.get_rect(center=(x, y)))
        pygame.draw.circle(s, (10, 14, 10), (x, y), r, 1)

    def _predict(self):
        """Erste getroffene Kugel + Geisterpunkt für die Ziellinie."""
        ox, oy = math.cos(self.aim), math.sin(self.aim)
        best_t = 1e9
        hit = None
        for b in self.balls:
            if b is self.cue or b.potted:
                continue
            relx, rely = b.x - self.cue.x, b.y - self.cue.y
            proj = relx * ox + rely * oy
            if proj <= 0:
                continue
            perp2 = (relx - proj * ox) ** 2 + (rely - proj * oy) ** 2
            if perp2 <= (2 * BR) ** 2:
                back = math.sqrt(max(0.0, (2 * BR) ** 2 - perp2))
                tcontact = proj - back
                if 0 < tcontact < best_t:
                    best_t = tcontact
                    hit = b
        # bis zur Bande, falls keine Kugel
        if hit is None:
            best_t = 0
            for _ in range(1):
                # einfache Distanz bis zur Wand
                tx = ((HW - BR) - self.cue.x) / ox if ox > 0 else \
                     ((-HW + BR) - self.cue.x) / ox if ox < 0 else 1e9
                ty = ((HH - BR) - self.cue.y) / oy if oy > 0 else \
                     ((-HH + BR) - self.cue.y) / oy if oy < 0 else 1e9
                best_t = max(0, min(tx, ty))
        cxp = self.cue.x + ox * best_t
        cyp = self.cue.y + oy * best_t
        return cxp, cyp, hit

    def _draw_aim(self, s):
        if self.phase == "place":
            return
        cxp, cyp, hit = self._predict()
        cs = self._project(self.cue.x, self.cue.y, 0)
        ce = self._project(cxp, cyp, 0)
        pygame.draw.line(s, COL_AIM, (cs[0], cs[1]), (ce[0], ce[1]), 2)
        if hit is not None:
            gx, gy, gsc, _ = self._project(cxp, cyp, 0)
            pygame.draw.circle(s, COL_AIM, (int(gx), int(gy)),
                               max(3, int(BR * gsc)), 1)
            # Zielkugel-Richtung
            tdx, tdy = hit.x - cxp, hit.y - cyp
            tl = math.hypot(tdx, tdy) or 1.0
            ex = hit.x + tdx / tl * 6 * BR
            ey = hit.y + tdy / tl * 6 * BR
            hs = self._project(hit.x, hit.y, 0)
            he = self._project(ex, ey, 0)
            pygame.draw.line(s, (200, 220, 255), (hs[0], hs[1]),
                             (he[0], he[1]), 2)
        # Queue-Stock (entgegengesetzt zur Zielrichtung)
        ox, oy = math.cos(self.aim), math.sin(self.aim)
        pull = 4 + self.power * 16
        bx = self.cue.x - ox * (BR + pull)
        by = self.cue.y - oy * (BR + pull)
        ex = self.cue.x - ox * (BR + pull + 70)
        ey = self.cue.y - oy * (BR + pull + 70)
        bs2 = self._project(bx, by, 0)
        es2 = self._project(ex, ey, 0)
        pygame.draw.line(s, COL_STICK, (bs2[0], bs2[1]), (es2[0], es2[1]), 5)
        pygame.draw.line(s, COL_STICK_D, (bs2[0], bs2[1]), (es2[0], es2[1]), 1)

    def _draw_hud(self, s):
        pygame.draw.rect(s, ui.PANEL, (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h), (self.width, self.hud_h))
        cy = self.hud_h // 2
        # Ansicht + Variante links
        vt = self._tiny.render(t("bil.view." + self.view) + "  ·  "
                               + t("bil.var." + self.variant), True, ui.TEXT_DIM)
        s.blit(vt, vt.get_rect(midleft=(12, cy)))
        # Kraft-Meter rechts
        mw, mh = 90, 10
        mx = self.width - mw - 14
        pygame.draw.rect(s, ui.BTN, (mx, cy - mh // 2, mw, mh),
                         border_radius=4)
        pygame.draw.rect(s, self.accent, (mx, cy - mh // 2, int(mw * self.power),
                                          mh), border_radius=4)
        # Mitte: wer ist dran / Gruppe / Nachricht
        if self.msg:
            mid = self.msg
        elif self.variant == "practice":
            mid = t("bil.potted", n=len(self.potted_all))
        elif self.phase == "place":
            mid = t("bil.place_cue")
        elif not self.multiplayer and self.current == 1:
            mid = t("bil.ai_turn")
        else:
            who = (t("common.player1") if self.current == 0 else t("common.player2")) \
                if self.multiplayer else t("bil.you")
            g = self.group[self.current]
            if g:
                who += " (" + t("bil.grp." + g) + ")"
            mid = who
        img = self._small.render(mid, True, self.accent)
        s.blit(img, img.get_rect(center=(self.width // 2, cy)))

    def _draw_over(self, s):
        # Halbtransparentes Banner-Panel wird gecacht (Software-Rendering).
        if self._over_cache is None or self._over_cache.get_width() != self.width:
            ov = pygame.Surface((self.width, 100), pygame.SRCALPHA)
            ov.fill((8, 12, 10, 210))
            self._over_cache = ov
        y = self.height // 2 - 50
        s.blit(self._over_cache, (0, y))
        pygame.draw.line(s, self.accent, (0, y), (self.width, y))
        pygame.draw.line(s, self.accent, (0, y + 99), (self.width, y + 99))
        cx = self.width // 2
        if self.variant == "practice":
            head = self._huge.render(t("bil.potted", n=len(self.potted_all)),
                                     True, self.accent)
        elif self.multiplayer:
            head = self._huge.render(t("common.player_wins", n=self.winner + 1),
                                     True, self.accent)
        else:
            won = self.winner == 0
            head = self._huge.render(t("bil.win_you") if won else t("bil.win_ai"),
                                     True, self.accent if won else ui.TEXT_DIM)
        s.blit(head, head.get_rect(center=(cx, y + 36)))
        hint = self._tiny.render(t("bil.new_round"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, y + 76)))

    def _draw_setup(self, s):
        cx = self.width // 2
        title = self._huge.render(t("bil.title"), True, self.accent)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.12))))
        sub = self._small.render(t("bil.subtitle"), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(self.height * 0.185))))

        def label(rects, txt):
            im = self._tiny.render(txt, True, ui.TEXT_DIM)
            s.blit(im, im.get_rect(midbottom=(cx, rects[0].top - 4)))

        label(self.var_rects, t("bil.lbl_variant"))
        for i, rc in enumerate(self.var_rects):
            self._btn(s, rc, t("bil.var." + VARIANTS[i]), self.variant == VARIANTS[i])
        label(self.view_rects, t("bil.lbl_view"))
        for i, rc in enumerate(self.view_rects):
            self._btn(s, rc, t("bil.view." + VIEWS[i]), self.view == VIEWS[i])
        label(self.diff_rects, t("bil.lbl_diff"))
        for i, rc in enumerate(self.diff_rects):
            self._btn(s, rc, t("bil.diff." + DIFFS[i]), self.diff == i)
        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=9)
        pygame.draw.rect(s, self.accent, self.start_rect, 2, border_radius=9)
        st = self.font.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        hint = self._tiny.render(t("bil.setup_hint"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, self.height - 14)))

    def _btn(self, s, rc, text, on):
        pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=8)
        pygame.draw.rect(s, self.accent if on else ui.BORDER, rc,
                         2 if on else 1, border_radius=8)
        im = self._small.render(text, True, ui.TEXT if on else ui.TEXT_DIM)
        s.blit(im, im.get_rect(center=rc.center))
