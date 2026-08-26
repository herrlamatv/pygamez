# -*- coding: utf-8 -*-
"""
pinball.py
==========
Pinball - ein vollwertiger Flipperautomat mit drei Tischen, Multiball,
Drop-Targets, Rollover-Bahnen, Bonus-Multiplikator, Ball-Save, Nudge und Tilt.

Tische (im Setup wählbar, wird gespeichert):
  - Classic : klassisches Layout, drei Pop-Bumper, ein Target-Bank links.
  - Space   : vier Bumper im Karo, zwei Target-Banks, hohe Punktwerte.
  - Lama    : offener Tisch mit sechs Targets im Bogen und mittigem Saucer.

Ablauf: Ball in der Schussbahn mit gedrückter Aktionstaste (Standard
Leertaste) aufladen und loslassen. Flipper links/rechts über die belegten
Links-/Rechts-Tasten (zusätzlich Shift links/rechts). Mit der Hoch-Taste lässt
sich der Tisch anstoßen (Nudge) - dreimal zu schnell hintereinander und der
Automat geht auf TILT: die Flipper sind bis zum Ballverlust tot.

Punkte: Bumper, Slingshots, Targets und Bahnen zählen mit dem aktuellen
Multiplikator (bis x5). Drei gefangene Bälle im Saucer starten den
**Multiball** samt Jackpot. Der Highscore ist die Punktzahl einer Partie.
"""

import math
import random

import pygame

import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent
from i18n import t

# ------------------------------------------------- Identitätsfarben (Tisch)
COL_TABLE = (24, 26, 40)
COL_TABLE_D = (18, 19, 30)
COL_WALL = (128, 136, 168)
COL_WALL_HI = (176, 186, 220)
COL_BALL = (226, 230, 240)
COL_BALL_D = (140, 146, 164)
COL_FLIP = (238, 196, 74)
COL_FLIP_D = (176, 138, 40)
COL_BUMP = (86, 196, 236)
COL_BUMP_HI = (206, 244, 255)
COL_SLING = (236, 108, 96)
COL_TARGET = (150, 236, 140)
COL_TARGET_OFF = (58, 82, 60)
COL_LANE = (208, 150, 240)
COL_SAUCER = (250, 214, 120)
COL_PLUNGER = (222, 96, 96)

# ------------------------------------------------------------ Tisch / Physik
TW, TH = 100.0, 170.0        # Tischmaße in Tisch-Einheiten
BR = 2.2                     # Ballradius
WALL_R = 1.0                 # halbe Dicke einer Wandstrecke
FLIP_R = 1.6                 # halbe Dicke eines Flippers
GRAV = 118.0                 # Schwerkraft (Einheiten/s²)
DRAG = 0.14                  # Rollwiderstand je Sekunde
WALL_E = 0.52                # Restitution der Banden
SLING_KICK = 118.0           # Zusatzschub der Slingshots
BUMP_KICK = 96.0             # Zusatzschub der Pop-Bumper
MAX_SPEED = 320.0
FLIP_SPEED = 17.0            # Winkelgeschwindigkeit der Flipper (rad/s)
FLIP_LEN = 19.0
DRAIN_Y = 162.0              # darunter ist der Ball verloren
LANE_X = 91.0                # Mitte der Schussbahn
BALL_SAVE = 6.0              # Sekunden Ball-Save nach dem Abschuss
TILT_LIMIT = 3               # so viele Nudges in Folge -> TILT
LANE_LETTERS = "LAMA"

SETUP, PLAY, OVER = "setup", "play", "over"
TABLES = ["classic", "space", "lama"]
BALL_COUNTS = [3, 5]


class _Ball:
    __slots__ = ("x", "y", "vx", "vy", "held", "st", "sx", "sy")

    def __init__(self, x, y, vx=0.0, vy=0.0):
        self.x = x
        self.y = y
        self.vx = vx
        self.vy = vy
        self.held = 0.0        # > 0: liegt im Saucer und wartet
        self.st = 0.0          # Sekunden ohne nennenswerte Bewegung
        self.sx = x            # Position, ab der gemessen wird
        self.sy = y


def _arc(cx, cy, r, a0, a1, n):
    """Kreisbogen als Streckenzug (Winkel in Grad)."""
    pts = []
    for i in range(n + 1):
        a = math.radians(a0 + (a1 - a0) * i / n)
        pts.append((cx + math.cos(a) * r, cy + math.sin(a) * r))
    return [(pts[i][0], pts[i][1], pts[i + 1][0], pts[i + 1][1])
            for i in range(len(pts) - 1)]


def _base_walls():
    """Banden, die jeder Tisch teilt: Bogen, Seiten, Schussbahn, Trichter."""
    walls = list(_arc(50, 52, 46, 180, 360, 22))
    walls += [
        (4, 52, 4, 110),          # linke Wand
        (4, 110, 24, 140),        # linker Trichter (endet am Flipper-Drehpunkt)
        (86, 52, 86, 110),        # rechte Wand (Trennwand zur Schussbahn)
        (86, 110, 76, 140),       # rechter Trichter (endet am Drehpunkt)
        (86, 110, 86, 152),       # Schussbahn links
        (96, 40, 96, 152),        # Schussbahn rechts
        (86, 152, 96, 152),       # Boden der Schussbahn
    ]
    return walls


# Einbahn-Klappe am Ausgang der Schussbahn (wirkt nur vom Spielfeld aus)
GATE = (86.0, 52.0, 77.0, 43.0)


def _table(bumpers, slings, targets, lanes, saucer, extra_walls=()):
    return {"walls": _base_walls() + [tuple(map(float, w)) for w in extra_walls],
            "bumpers": [tuple(map(float, b)) for b in bumpers],
            "slings": [tuple(map(float, s)) for s in slings],
            "targets": [tuple(map(float, r)) for r in targets],
            "lanes": [tuple(map(float, l)) for l in lanes],
            "saucer": tuple(map(float, saucer))}


TABLE_DATA = {
    # Klassiker: drei Bumper im Dreieck, ein Target-Bank links
    "classic": _table(
        bumpers=[(38, 62, 6), (64, 62, 6), (51, 42, 6)],
        slings=[(24, 116, 38, 134), (76, 116, 62, 134)],
        targets=[(12, 66, 7, 6), (12, 76, 7, 6), (12, 86, 7, 6), (12, 96, 7, 6)],
        lanes=[(32, 26, 3), (44, 22, 3), (57, 22, 3), (69, 26, 3)],
        saucer=(50, 88, 4.2),
        extra_walls=[(24, 104, 34, 96), (76, 104, 66, 96)]),
    # Weltraum: vier Bumper im Karo, zwei Banks
    "space": _table(
        bumpers=[(50, 38, 5.5), (34, 58, 5.5), (66, 58, 5.5), (50, 76, 5.5)],
        slings=[(24, 116, 38, 134), (76, 116, 62, 134)],
        targets=[(11, 62, 7, 6), (11, 72, 7, 6), (11, 82, 7, 6),
                 (79, 62, 7, 6), (79, 72, 7, 6), (79, 82, 7, 6)],
        lanes=[(30, 28, 3), (43, 22, 3), (58, 22, 3), (71, 28, 3)],
        saucer=(50, 100, 4.2),
        extra_walls=[(20, 96, 32, 104), (80, 96, 68, 104)]),
    # Lama: offener Tisch, Targets im Bogen, Saucer in der Mitte
    "lama": _table(
        bumpers=[(30, 50, 6.5), (70, 50, 6.5)],
        slings=[(24, 116, 38, 134), (76, 116, 62, 134),
                (34, 92, 44, 104), (66, 92, 56, 104)],
        targets=[(24, 30, 7, 6), (36, 24, 7, 6), (48, 22, 7, 6),
                 (60, 24, 7, 6), (72, 30, 7, 6), (46, 62, 8, 6)],
        lanes=[(16, 60, 3), (16, 72, 3), (84, 60, 3), (84, 72, 3)],
        saucer=(50, 84, 4.6),
        extra_walls=[(16, 96, 26, 104), (84, 96, 74, 104)]),
}


class PinballGame(Game):
    name = "Pinball"
    highscore_key = "pinball"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        gs = self.settings.get("pinball", {}) if isinstance(self.settings, dict) else {}
        self.table_key = gs.get("table", "classic")
        if self.table_key not in TABLES:
            self.table_key = "classic"
        self.ball_count = int(gs.get("balls", 3))
        if self.ball_count not in BALL_COUNTS:
            self.ball_count = 3

        self._build_fonts()
        self._layout()
        self._build_setup_layout()
        self._over_cache = None
        self.best = self._load_best()
        self._new_game()
        self.state = SETUP

    def _build_fonts(self):
        h = self.height
        self._small = ui.font(max(14, h // 32))
        self._tiny = ui.font(max(11, h // 42))
        self._num = ui.font(max(16, h // 26), bold=True)
        self._huge = ui.font(max(24, h // 13), bold=True)

    def on_surface_changed(self):
        self._build_fonts()
        self._layout()
        self._build_setup_layout()
        self._over_cache = None

    def _layout(self):
        self.hud_h = 44
        avail_h = self.height - self.hud_h - 10
        self.scale = max(1.1, min((self.width - 150) / TW, avail_h / TH))
        self.ox = self.width / 2.0 - TW * self.scale / 2.0
        self.oy = self.hud_h + (avail_h - TH * self.scale) / 2.0 + 5
        self.side_x = int(self.ox + TW * self.scale) + 8

    # ------------------------------------------------------- Speicherstand
    def _load_best(self):
        data = store.load_section("pinball")
        best = data.get("best") if isinstance(data, dict) else None
        out = {}
        if isinstance(best, dict):
            for k, v in best.items():
                try:
                    out[str(k)] = int(v)
                except (TypeError, ValueError):
                    continue
        return out

    def _save_best(self, score):
        if score > self.best.get(self.table_key, 0):
            self.best[self.table_key] = int(score)
            store.save_section("pinball", {"best": self.best})

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("pinball", {})[key] = value
            settings_mod.save_settings(self.settings)

    # ------------------------------------------------------- Partie / Ball
    def _new_game(self):
        self.table = TABLE_DATA[self.table_key]
        self.players = 2 if self.multiplayer else 1
        self.scores = [0] * self.players
        self.balls_left = [self.ball_count] * self.players
        self.player = 0
        self.winner = None
        self.score = 0
        self.game_over = False
        self.msg = None
        self.msg_t = 0.0
        self._new_ball()

    def _new_ball(self):
        self.balls = [_Ball(LANE_X, 146.0)]
        self.phase = "launch"
        self.plunger = 0.0
        self.charging = False
        self.flip_l = self.flip_r = 0.0        # 0 = Ruhe, 1 = oben
        self.flip_l_up = self.flip_r_up = False
        self.omega_l = self.omega_r = 0.0
        self.mult = 1
        self.locks = 0
        self.multiball = False
        self.jackpot = False
        self.save_t = 0.0
        self.tilt = False
        self.nudges = 0
        self.nudge_t = 0.0
        self.shake = 0.0
        self.targets_hit = [False] * len(self.table["targets"])
        self.lanes_hit = [False] * len(self.table["lanes"])
        self.bank_clears = 0
        self.flash = {}

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(370, self.width - 50)
        y0 = int(self.height * 0.30)
        gap = 8

        def row(y, n):
            cw = (bw - gap * (n - 1)) / n
            return [pygame.Rect(int(cx - bw / 2 + i * (cw + gap)), y,
                                int(cw), 42) for i in range(n)]

        self.table_rects = row(y0, 3)
        self.ball_rects = row(y0 + 88, 2)
        self.start_rect = pygame.Rect(cx - 95, y0 + 156, 190, 46)

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3"):
                self.table_key = TABLES[int(k) - 1]
                self._save_setting("table", self.table_key)
                self.play_sound("click")
            elif k in ("b", "B"):
                self._cycle_balls()
            elif k in ("Return", "space"):
                self._start_play()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.table_rects):
                if rc.collidepoint(event.pos):
                    self.table_key = TABLES[i]
                    self._save_setting("table", self.table_key)
                    self.play_sound("click")
                    return
            for i, rc in enumerate(self.ball_rects):
                if rc.collidepoint(event.pos):
                    self.ball_count = BALL_COUNTS[i]
                    self._save_setting("balls", self.ball_count)
                    self.play_sound("select")
                    return
            if self.start_rect.collidepoint(event.pos):
                self._start_play()

    def _cycle_balls(self):
        i = (BALL_COUNTS.index(self.ball_count) + 1) % len(BALL_COUNTS)
        self.ball_count = BALL_COUNTS[i]
        self._save_setting("balls", self.ball_count)
        self.play_sound("select")

    def _start_play(self):
        self._new_game()
        self.state = PLAY
        self.play_sound("click")

    # ===================================================== Eingabe
    def handle_event(self, event):
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
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k == "Shift_L" or self.is_action(k, "left") or k == "Left":
                self.flip_l_up = True
            if k == "Shift_R" or self.is_action(k, "right") or k == "Right":
                self.flip_r_up = True
            if self.is_action(k, "up") or k == "Up":
                self._nudge()
            if (self.is_action(k, "action") or k in ("space", "Return")) \
                    and self.phase == "launch":
                self.charging = True
        elif event.kind == InputEvent.KEYUP:
            k = event.key
            if k == "Shift_L" or self.is_action(k, "left") or k == "Left":
                self.flip_l_up = False
            if k == "Shift_R" or self.is_action(k, "right") or k == "Right":
                self.flip_r_up = False
            if (self.is_action(k, "action") or k in ("space", "Return")) \
                    and self.charging:
                self._launch()
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            if self.phase == "launch":
                self.charging = True
            else:
                # Maus: linke Hälfte = linker Flipper, rechte Hälfte = rechter
                if event.pos[0] < self.width // 2:
                    self.flip_l_up = True
                else:
                    self.flip_r_up = True
        elif event.kind == InputEvent.MOUSEUP and event.button == 1:
            if self.charging:
                self._launch()
            self.flip_l_up = self.flip_r_up = False

    def _launch(self):
        self.charging = False
        if self.phase != "launch" or not self.balls:
            return
        b = self.balls[0]
        b.vy = -(95.0 + 175.0 * max(0.10, self.plunger))
        b.vx = -random.uniform(0.0, 4.0)
        self.phase = "play"
        self.save_t = BALL_SAVE
        self.plunger = 0.0
        self.play_sound("shoot")
        self.rumble(70)

    def _nudge(self):
        if self.tilt or self.phase != "play":
            return
        self.nudges += 1
        self.nudge_t = 4.0
        self.shake = 1.0
        for b in self.balls:
            b.vx += random.uniform(-26.0, 26.0)
            b.vy -= 22.0
        self.play_sound("hit")
        if self.nudges > TILT_LIMIT:
            self.tilt = True
            self._say(t("pin.tilt"), 2.6)
            self.play_sound("gameover")

    # ===================================================== Update / Physik
    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None
        for key in list(self.flash):
            self.flash[key] -= dt
            if self.flash[key] <= 0:
                del self.flash[key]
        if self.shake > 0:
            self.shake = max(0.0, self.shake - dt * 3.2)
        if self.nudge_t > 0:
            self.nudge_t -= dt
            if self.nudge_t <= 0:
                self.nudges = 0
        if self.state != PLAY:
            return

        # Flipper weich zur Zielstellung fahren
        for side in ("l", "r"):
            cur = getattr(self, "flip_" + side)
            up = getattr(self, "flip_" + side + "_up") and not self.tilt
            target = 1.0 if up else 0.0
            step = FLIP_SPEED * dt / 2.6
            new = min(target, cur + step) if target > cur else max(target, cur - step)
            setattr(self, "omega_" + side, (new - cur) / max(dt, 1e-4) * 0.95)
            setattr(self, "flip_" + side, new)

        if self.phase == "launch":
            if self.charging:
                self.plunger = min(1.0, self.plunger + dt * 0.85)
            return

        if self.save_t > 0:
            self.save_t = max(0.0, self.save_t - dt)
        for b in list(self.balls):
            if b.held > 0:
                b.held -= dt
                if b.held <= 0:
                    self._eject(b)
                continue
            self._step_ball(b, dt)
            self._unstick(b, dt)
        if not self.balls:
            self._ball_lost()
        elif self._back_in_lane():
            # Zu schwach abgeschossen: der Ball rollt in die Schussbahn
            # zurueck - dann darf noch einmal geladen werden (wie am Automaten).
            b = self.balls[0]
            b.x, b.y, b.vx, b.vy = LANE_X, 146.0, 0.0, 0.0
            self.phase = "launch"
            self.plunger = 0.0
            self.charging = False

    def _unstick(self, b, dt):
        """Sicherheitsnetz: ein liegen gebliebener Ball bekommt einen Stups.

        Auf einer Wandspitze oder in einer engen Ecke kann eine Kugel
        theoretisch balancieren. Bewegt sie sich drei Sekunden lang kaum,
        stupst der Automat sie an - so haengt keine Partie fest.
        """
        if math.hypot(b.x - b.sx, b.y - b.sy) > 1.0:
            b.sx, b.sy, b.st = b.x, b.y, 0.0
            return
        b.st += dt
        if b.st > 3.0:
            b.vx += random.uniform(-45.0, 45.0)
            b.vy -= 25.0
            b.sx, b.sy, b.st = b.x, b.y, 0.0

    def _back_in_lane(self):
        """True, wenn der einzige Ball unten in der Schussbahn liegen bleibt."""
        if self.phase != "play" or len(self.balls) != 1:
            return False
        b = self.balls[0]
        return b.x > 86.0 and b.y > 138.0 and abs(b.vy) < 26.0 and abs(b.vx) < 26.0

    def _step_ball(self, b, dt):
        speed = math.hypot(b.vx, b.vy)
        steps = max(2, min(26, int(speed * dt / BR) + 2))
        h = dt / steps
        for _ in range(steps):
            b.vy += GRAV * h
            f = max(0.0, 1.0 - DRAG * h)
            b.vx *= f
            b.vy *= f
            sp = math.hypot(b.vx, b.vy)
            if sp > MAX_SPEED:
                b.vx *= MAX_SPEED / sp
                b.vy *= MAX_SPEED / sp
            b.x += b.vx * h
            b.y += b.vy * h
            self._collide_walls(b)
            self._collide_flippers(b)
            self._collide_bumpers(b)
            self._collide_slings(b)
            self._collide_targets(b)
            self._check_lanes(b)
            if self._check_saucer(b):
                return
            if b.y > DRAIN_Y:
                self._drain(b)
                return

    # ------------------------------------------------------- Kollisionen
    @staticmethod
    def _closest_on_seg(x1, y1, x2, y2, px, py):
        dx, dy = x2 - x1, y2 - y1
        l2 = dx * dx + dy * dy
        if l2 < 1e-9:
            return x1, y1
        f = max(0.0, min(1.0, ((px - x1) * dx + (py - y1) * dy) / l2))
        return x1 + f * dx, y1 + f * dy

    def _hit_seg(self, b, seg, radius, rest, kick=0.0, vel=(0.0, 0.0)):
        """Kreis gegen Strecke; True bei Treffer."""
        px, py = self._closest_on_seg(seg[0], seg[1], seg[2], seg[3], b.x, b.y)
        dx, dy = b.x - px, b.y - py
        d = math.hypot(dx, dy)
        rad = BR + radius
        if d >= rad:
            return False
        if d < 1e-9:
            dx, dy, d = 0.0, -1.0, 1.0
        ux, uy = dx / d, dy / d
        b.x, b.y = px + ux * rad, py + uy * rad
        rvx, rvy = b.vx - vel[0], b.vy - vel[1]
        dot = rvx * ux + rvy * uy
        if dot < 0:
            rvx -= (1 + rest) * dot * ux
            rvy -= (1 + rest) * dot * uy
        b.vx, b.vy = rvx + vel[0] + ux * kick, rvy + vel[1] + uy * kick
        return True

    def _collide_walls(self, b):
        for seg in self.table["walls"]:
            if self._hit_seg(b, seg, WALL_R, WALL_E):
                if abs(b.vx) + abs(b.vy) > 90:
                    self.play_sound("bounce")
        if b.x < 84.0:                      # Einbahn-Klappe nur vom Feld aus
            self._hit_seg(b, GATE, WALL_R, WALL_E)

    def _flipper_seg(self, side):
        """Aktuelle Strecke (Drehpunkt -> Spitze) eines Flippers."""
        if side == "l":
            pivot, rest, up = (24.0, 140.0), math.radians(26), math.radians(-30)
            a = rest + (up - rest) * self.flip_l
        else:
            pivot, rest, up = (76.0, 140.0), math.radians(154), math.radians(210)
            a = rest + (up - rest) * self.flip_r
        return (pivot[0], pivot[1],
                pivot[0] + math.cos(a) * FLIP_LEN,
                pivot[1] + math.sin(a) * FLIP_LEN)

    def _collide_flippers(self, b):
        for side in ("l", "r"):
            seg = self._flipper_seg(side)
            px, py = self._closest_on_seg(seg[0], seg[1], seg[2], seg[3], b.x, b.y)
            omega = getattr(self, "omega_" + side)
            # Umfangsgeschwindigkeit am Kontaktpunkt (Drehrichtung beachten)
            sign = -1.0 if side == "l" else 1.0
            rx, ry = px - seg[0], py - seg[1]
            w = omega * sign * 3.1
            vel = (-ry * w, rx * w)
            if self._hit_seg(b, seg, FLIP_R, 0.45, vel=vel):
                self.play_sound("bounce")

    def _collide_bumpers(self, b):
        for i, (x, y, r) in enumerate(self.table["bumpers"]):
            dx, dy = b.x - x, b.y - y
            d = math.hypot(dx, dy)
            rad = r + BR
            if d >= rad or d < 1e-9:
                continue
            ux, uy = dx / d, dy / d
            b.x, b.y = x + ux * rad, y + uy * rad
            dot = b.vx * ux + b.vy * uy
            if dot < 0:
                b.vx -= 1.5 * dot * ux
                b.vy -= 1.5 * dot * uy
            b.vx += ux * BUMP_KICK
            b.vy += uy * BUMP_KICK
            self.flash["b%d" % i] = 0.22
            self._award(2500 if self.jackpot else 100,
                        t("pin.jackpot") if self.jackpot else None)
            self.play_sound("point" if self.jackpot else "bounce")

    def _collide_slings(self, b):
        for i, seg in enumerate(self.table["slings"]):
            if self._hit_seg(b, seg, WALL_R * 1.6, 0.35, kick=SLING_KICK):
                self.flash["s%d" % i] = 0.18
                self._award(50)
                self.play_sound("bounce")

    def _collide_targets(self, b):
        for i, (x, y, w, h) in enumerate(self.table["targets"]):
            if self.targets_hit[i]:
                continue
            nx = max(x, min(b.x, x + w))
            ny = max(y, min(b.y, y + h))
            dx, dy = b.x - nx, b.y - ny
            if dx * dx + dy * dy >= BR * BR:
                continue
            d = math.hypot(dx, dy) or 1.0
            ux, uy = dx / d, dy / d
            b.x, b.y = nx + ux * BR, ny + uy * BR
            dot = b.vx * ux + b.vy * uy
            if dot < 0:
                b.vx -= 1.4 * dot * ux
                b.vy -= 1.4 * dot * uy
            self.targets_hit[i] = True
            self._award(250)
            self.play_sound("lock")
            if all(self.targets_hit):
                self.targets_hit = [False] * len(self.targets_hit)
                self.bank_clears += 1
                self.mult = min(5, self.mult + 1)
                self._award(2500, t("pin.bank", n=self.mult))
                self.play_sound("powerup")

    def _check_lanes(self, b):
        for i, (x, y, r) in enumerate(self.table["lanes"]):
            if self.lanes_hit[i]:
                continue
            if (b.x - x) ** 2 + (b.y - y) ** 2 < (r + BR) ** 2:
                self.lanes_hit[i] = True
                self._award(200)
                self.play_sound("eat")
                if all(self.lanes_hit):
                    self.lanes_hit = [False] * len(self.lanes_hit)
                    self.mult = min(5, self.mult + 1)
                    self._award(1000, t("pin.lanes", n=self.mult))
                    self.play_sound("level")

    def _check_saucer(self, b):
        x, y, r = self.table["saucer"]
        if (b.x - x) ** 2 + (b.y - y) ** 2 > r * r:
            return False
        if math.hypot(b.vx, b.vy) > 110:
            return False
        b.x, b.y, b.vx, b.vy = x, y, 0.0, 0.0
        b.held = 1.1
        if self.multiball:
            self._award(5000, t("pin.jackpot"))
            self.play_sound("win")
            return True
        self.locks += 1
        if self.locks >= 3:
            self._start_multiball(b)
        else:
            self._award(1000, t("pin.lock", n=self.locks))
            self.play_sound("powerup")
        return True

    def _start_multiball(self, b):
        self.multiball = True
        self.jackpot = True
        self.locks = 0
        b.held = 1.4
        self._award(5000, t("pin.multiball"))
        self.ach_event("pin_multiball")
        self.play_sound("win")

    def _eject(self, b):
        """Wirft den Ball aus dem Saucer - im Multiball gleich mit Nachschub."""
        b.vx = random.uniform(-40.0, 40.0)
        b.vy = -150.0
        b.held = 0.0
        if self.multiball and len(self.balls) < 3:
            for _ in range(3 - len(self.balls)):
                self.balls.append(_Ball(b.x + random.uniform(-4, 4), b.y,
                                        random.uniform(-60, 60), -130.0))
            self.save_t = max(self.save_t, 4.0)
        self.play_sound("shoot")

    # ------------------------------------------------------- Ballverlust
    def _drain(self, b):
        if b in self.balls:
            self.balls.remove(b)
        if self.balls:
            if len(self.balls) == 1:        # Multiball vorbei
                self.multiball = False
                self.jackpot = False
            self.play_sound("hit")
            return
        if self.save_t > 0 and not self.tilt:
            self.balls = [_Ball(LANE_X, 146.0)]
            self.phase = "launch"
            self.plunger = 0.0
            self._say(t("pin.saved"), 1.8)
            self.play_sound("powerup")
            return
        self.play_sound("gameover")

    def _ball_lost(self):
        """Wird gerufen, wenn kein Ball mehr im Spiel ist."""
        bonus = 250 * self.mult
        self.scores[self.player] += bonus
        self._sync_score()
        self.balls_left[self.player] -= 1
        if max(self.balls_left) <= 0:
            self._end_game()
            return
        # Nächster Spieler mit verbleibenden Bällen
        for i in range(1, self.players + 1):
            nxt = (self.player + i) % self.players
            if self.balls_left[nxt] > 0:
                self.player = nxt
                break
        self._new_ball()
        self._say(t("pin.bonus", n=bonus), 2.2)

    def _end_game(self):
        self._save_best(self.scores[0])
        if self.scores[0] >= 50000:
            self.ach_event("pin_high", self.scores[0])
        if self.multiplayer:
            s0, s1 = self.scores
            self.winner = 0 if s0 > s1 else (1 if s1 > s0 else None)
        else:
            self.report_result(self.scores[0] >= self.best.get(self.table_key, 0))
        self._sync_score()
        self.state = OVER
        self.game_over = True
        self.play_sound("gameover")

    def _restart(self):
        self._new_game()
        self.state = PLAY
        self.play_sound("click")

    # ------------------------------------------------------- Hilfsfunktionen
    def _award(self, points, message=None):
        self.scores[self.player] += points * self.mult
        self._sync_score()
        if message:
            self._say(message, 1.8)

    def _sync_score(self):
        self.score = self.scores[0]

    def _say(self, text, secs):
        self.msg = text
        self.msg_t = secs

    # ===================================================== Zeichnen
    def _project(self, x, y):
        sh = 0.0
        if self.shake > 0:
            sh = math.sin(pygame.time.get_ticks() / 22.0) * 4 * self.shake
        return (self.ox + x * self.scale + sh, self.oy + y * self.scale)

    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        if self.state == SETUP:
            self._draw_setup(s)
            return
        self._draw_table(s)
        self._draw_hud(s)
        self._draw_side(s)
        if self.state == OVER:
            self._draw_over(s)

    def _draw_table(self, s):
        board = pygame.Rect(int(self.ox), int(self.oy),
                            int(TW * self.scale), int(TH * self.scale))
        pygame.draw.rect(s, COL_TABLE, board, border_radius=8)
        pygame.draw.rect(s, COL_TABLE_D, board, 2, border_radius=8)
        # dezenter Lichtkegel oben
        glow = pygame.Surface((board.w, board.h // 2), pygame.SRCALPHA)
        pygame.draw.ellipse(glow, (70, 90, 150, 46), glow.get_rect())
        s.blit(glow, (board.x, board.y))

        for seg in self.table["walls"]:
            a, b = self._project(seg[0], seg[1]), self._project(seg[2], seg[3])
            pygame.draw.line(s, COL_WALL, a, b, max(2, int(WALL_R * 2 * self.scale)))
            pygame.draw.line(s, COL_WALL_HI, a, b, 1)
        ga, gb = self._project(GATE[0], GATE[1]), self._project(GATE[2], GATE[3])
        pygame.draw.line(s, COL_WALL_HI, ga, gb, 1)

        # Saucer
        sx, sy, sr = self.table["saucer"]
        px, py = self._project(sx, sy)
        pygame.draw.circle(s, (12, 14, 22), (int(px), int(py)),
                           max(3, int(sr * self.scale)))
        pygame.draw.circle(s, COL_SAUCER, (int(px), int(py)),
                           max(3, int(sr * self.scale)), 2)

        # Rollover-Bahnen mit Buchstaben
        for i, (x, y, r) in enumerate(self.table["lanes"]):
            px, py = self._project(x, y)
            rr = max(3, int(r * self.scale))
            on = self.lanes_hit[i]
            pygame.draw.circle(s, COL_LANE if on else (58, 44, 72),
                               (int(px), int(py)), rr)
            pygame.draw.circle(s, COL_LANE, (int(px), int(py)), rr, 1)
            ch = LANE_LETTERS[i % len(LANE_LETTERS)]
            img = self._tiny.render(ch, True, (20, 16, 28) if on else COL_LANE)
            s.blit(img, img.get_rect(center=(px, py)))

        # Drop-Targets
        for i, r in enumerate(self.table["targets"]):
            rc = pygame.Rect(self._project(r[0], r[1]),
                             (max(2, int(r[2] * self.scale)),
                              max(2, int(r[3] * self.scale))))
            col = COL_TARGET_OFF if self.targets_hit[i] else COL_TARGET
            pygame.draw.rect(s, col, rc, border_radius=2)
            pygame.draw.rect(s, ui.mix(col, (255, 255, 255), 0.3), rc, 1,
                             border_radius=2)

        # Slingshots
        for i, seg in enumerate(self.table["slings"]):
            a, b = self._project(seg[0], seg[1]), self._project(seg[2], seg[3])
            hot = "s%d" % i in self.flash
            pygame.draw.line(s, (255, 236, 200) if hot else COL_SLING, a, b,
                             max(3, int(WALL_R * 3.4 * self.scale)))

        # Pop-Bumper
        for i, (x, y, r) in enumerate(self.table["bumpers"]):
            px, py = self._project(x, y)
            rr = max(4, int(r * self.scale))
            hot = "b%d" % i in self.flash
            pygame.draw.circle(s, COL_BUMP, (int(px), int(py)), rr)
            pygame.draw.circle(s, (255, 255, 255) if hot else COL_BUMP_HI,
                               (int(px), int(py)), max(2, int(rr * 0.55)))
            pygame.draw.circle(s, COL_BUMP_HI, (int(px), int(py)), rr, 2)

        self._draw_flippers(s)
        self._draw_plunger(s)
        for b in self.balls:
            self._draw_ball(s, b)

    def _draw_flippers(self, s):
        for side in ("l", "r"):
            seg = self._flipper_seg(side)
            a, b = self._project(seg[0], seg[1]), self._project(seg[2], seg[3])
            w = max(4, int(FLIP_R * 2 * self.scale))
            col = COL_FLIP_D if self.tilt else COL_FLIP
            pygame.draw.line(s, col, a, b, w)
            pygame.draw.circle(s, COL_FLIP_D, (int(a[0]), int(a[1])),
                               max(2, w // 2))
            pygame.draw.circle(s, ui.mix(col, (255, 255, 255), 0.4),
                               (int(b[0]), int(b[1])), max(2, w // 3))

    def _draw_plunger(self, s):
        if self.phase != "launch":
            return
        x0, y0 = self._project(LANE_X - 3.4, 150.0)
        x1, _ = self._project(LANE_X + 3.4, 150.0)
        h = max(6, int(16 * self.scale * (0.25 + self.plunger)))
        pygame.draw.rect(s, COL_PLUNGER, (int(x0), int(y0), int(x1 - x0), h),
                         border_radius=3)

    def _draw_ball(self, s, b):
        px, py = self._project(b.x, b.y)
        r = max(2, int(BR * self.scale))
        pygame.draw.circle(s, (8, 10, 16), (int(px + 2), int(py + 2)), r)
        pygame.draw.circle(s, COL_BALL, (int(px), int(py)), r)
        pygame.draw.circle(s, COL_BALL_D, (int(px), int(py)), r, 1)
        if r >= 4:
            pygame.draw.circle(s, (255, 255, 255),
                               (int(px - r // 3), int(py - r // 3)),
                               max(1, r // 3))

    def _draw_hud(self, s):
        pygame.draw.rect(s, ui.PANEL, (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h), (self.width, self.hud_h))
        cy = self.hud_h // 2
        # Punktzahl ohne Tausendertrennung - die schreibt sich je nach Sprache
        # anders (1.200 / 1,200 / 1 200); die uebrigen Spiele zeigen sie ebenso.
        img = self._num.render(str(self.scores[self.player]), True, self.accent)
        s.blit(img, img.get_rect(midleft=(12, cy)))
        if self.msg:
            mid, col = self.msg, ui.GOLD
        elif self.tilt:
            mid, col = t("pin.tilt"), ui.RED
        elif self.phase == "launch":
            mid, col = t("pin.launch"), ui.TEXT_DIM
        elif self.multiplayer:
            mid = t("common.player1" if self.player == 0 else "common.player2")
            col = ui.TEXT_DIM
        else:
            mid, col = t("pin.mult", n=self.mult), ui.TEXT_DIM
        m = self._small.render(mid, True, col)
        s.blit(m, m.get_rect(center=(self.width // 2, cy)))
        balls = self._tiny.render(t("pin.ball", n=self.balls_left[self.player]),
                                  True, ui.TEXT_DIM)
        s.blit(balls, balls.get_rect(midright=(self.width - 12, cy)))

    def _draw_side(self, s):
        """Schmale Info-Spalte rechts: Multiplikator, Ball-Save, Locks."""
        x = self.side_x
        w = self.width - x - 6
        if w < 70:
            return
        y = self.hud_h + 10
        rect = pygame.Rect(x, y, w, 116)
        ui.draw_panel(s, rect, radius=8, shadow=False)
        cy = y + 10
        rows = [(t("pin.lbl_mult"), "x%d" % self.mult, self.accent),
                (t("pin.lbl_lock"), "%d/3" % self.locks, COL_SAUCER),
                (t("pin.lbl_bank"), str(self.bank_clears), COL_TARGET)]
        if self.save_t > 0:
            rows.append((t("pin.lbl_save"), "%.0fs" % self.save_t, ui.GREEN))
        elif self.multiball:
            rows.append((t("pin.lbl_mode"), t("pin.multiball_short"), ui.GOLD))
        for label, value, col in rows:
            s.blit(self._tiny.render(label, True, ui.TEXT_DIM), (x + 8, cy))
            v = self._tiny.render(value, True, col)
            s.blit(v, v.get_rect(topright=(x + w - 8, cy)))
            cy += self._tiny.get_height() + 8
        if self.multiplayer:
            cy += 4
            pygame.draw.line(s, ui.BORDER, (x + 6, cy), (x + w - 6, cy))
            cy += 6
            for i in range(self.players):
                col = self.accent if i == self.player else ui.TEXT_DIM
                s.blit(self._tiny.render("P%d" % (i + 1), True, col), (x + 8, cy))
                v = self._tiny.render(str(self.scores[i]), True, col)
                s.blit(v, v.get_rect(topright=(x + w - 8, cy)))
                cy += self._tiny.get_height() + 4

    def _draw_over(self, s):
        h = 118
        if self._over_cache is None or self._over_cache.get_width() != self.width:
            ov = pygame.Surface((self.width, h), pygame.SRCALPHA)
            ov.fill((10, 12, 20, 214))
            self._over_cache = ov
        y = self.height // 2 - h // 2
        s.blit(self._over_cache, (0, y))
        pygame.draw.line(s, self.accent, (0, y), (self.width, y))
        pygame.draw.line(s, self.accent, (0, y + h - 1), (self.width, y + h - 1))
        cx = self.width // 2
        if self.multiplayer:
            head = self._huge.render(
                t("common.draw") if self.winner is None
                else t("common.player_wins", n=self.winner + 1), True, self.accent)
        else:
            head = self._huge.render(t("common.game_over"), True, self.accent)
        s.blit(head, head.get_rect(center=(cx, y + 32)))
        sub = self._small.render(t("common.points", score=self.scores[0]),
                                 True, ui.TEXT)
        s.blit(sub, sub.get_rect(center=(cx, y + 66)))
        best = self.best.get(self.table_key)
        if best:
            b = self._tiny.render(t("pin.best", n=best), True, ui.GOLD)
            s.blit(b, b.get_rect(center=(cx, y + 88)))
        hint = self._tiny.render(t("pin.new_round"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, y + 106)))

    def _draw_setup(self, s):
        cx = self.width // 2
        title = self._huge.render(t("pin.title"), True, self.accent)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.13))))
        sub = self._small.render(t("pin.subtitle"), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(self.height * 0.20))))

        def label(rects, txt):
            im = self._tiny.render(txt, True, ui.TEXT_DIM)
            s.blit(im, im.get_rect(midbottom=(cx, rects[0].top - 4)))

        label(self.table_rects, t("pin.lbl_table"))
        for i, rc in enumerate(self.table_rects):
            self._btn(s, rc, t("pin.table." + TABLES[i]), self.table_key == TABLES[i])
        label(self.ball_rects, t("pin.lbl_balls"))
        for i, rc in enumerate(self.ball_rects):
            self._btn(s, rc, str(BALL_COUNTS[i]), self.ball_count == BALL_COUNTS[i])
        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=9)
        pygame.draw.rect(s, self.accent, self.start_rect, 2, border_radius=9)
        st = self.font.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        best = self.best.get(self.table_key)
        if best:
            b = self._tiny.render(t("pin.best", n=best), True, ui.GOLD)
            s.blit(b, b.get_rect(center=(cx, self.start_rect.bottom + 22)))
        hint = self._tiny.render(t("pin.setup_hint"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, self.height - 14)))

    def _btn(self, s, rc, text, on):
        pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=8)
        pygame.draw.rect(s, self.accent if on else ui.BORDER, rc,
                         2 if on else 1, border_radius=8)
        im = self._small.render(text, True, ui.TEXT if on else ui.TEXT_DIM)
        s.blit(im, im.get_rect(center=rc.center))
