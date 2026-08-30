# -*- coding: utf-8 -*-
"""
bowling.py
==========
Bowling - zehn Frames mit vollständiger Strike-/Spare-Wertung, echter
Pin-Physik und perspektivischer Bahnansicht, allein oder zu zweit lokal.

Ein Wurf läuft in vier kurzen Schritten, die je mit der Aktionstaste
(Standard: Leertaste) oder einem Klick festgelegt werden:

    1. Position : Standpunkt an der Foullinie
    2. Ziel     : Wurfwinkel
    3. Effet    : Drall nach links/rechts (der Hook greift erst hinten,
                  wenn das Öl auf der Bahn ausläuft)
    4. Kraft    : Wurfgeschwindigkeit

Jeder Regler pendelt von allein - wer lieber genau zielt, stellt ihn mit den
Links-/Rechts-Tasten selbst ein (das Pendeln hält dann an).

Die zehn Pins werden als Kreise mit Masse simuliert: der Ball stößt sie an,
sie stoßen sich gegenseitig um - Strikes über die Pocket sind also echtes
Ergebnis der Physik, kein Zufall. Auf *Pro* pendeln die Regler schneller und
die Bahn streut minimal stärker.

Punkte (Highscore) = Spielsumme nach offiziellen Regeln (maximal 300).
"""

import math
import random

import pygame

import replay
import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent
from i18n import t

# ------------------------------------------------- Identitätsfarben (Bahn)
COL_LANE = (206, 164, 104)
COL_LANE_D = (186, 144, 88)
COL_BOARD = (196, 152, 96)
COL_GUTTER = (46, 50, 62)
COL_GUTTER_D = (32, 35, 45)
COL_FOUL = (208, 72, 72)
COL_ARROW = (150, 108, 62)
COL_DECK = (232, 228, 220)
COL_BACK = (28, 32, 46)
COL_PIN = (246, 246, 242)
COL_PIN_D = (198, 198, 192)
COL_PIN_RED = (216, 60, 60)
COL_BALL = (58, 74, 176)
COL_BALL_HI = (120, 140, 236)
COL_MARK = (250, 224, 120)

# --------------------------------------------------------- Bahn / Physik
# Alle Maße in Zoll - so stimmen die Verhältnisse mit echten Bahnen überein.
LW = 41.5                     # Bahnbreite
GUT = 9.0                     # Rinne je Seite
PIN_Y = 720.0                 # Kopfpin, gemessen ab Foullinie
ROW_DY = 10.392               # Reihenabstand (12 * cos 30°)
PIN_DX = 6.0                  # halber Abstand benachbarter Pins
BALL_R = 4.25
PIN_R = 2.4
PIN_M = 3.5
BALL_M = 16.0
OIL_END = 430.0               # ab hier greift der Hook
HOOK = 300.0                  # seitliche Beschleunigung bei vollem Effet
PIN_FRIC = 3.4
BALL_FRIC = 0.10
REST = 0.55
DOWN_DIST = 1.6               # so weit verschoben gilt ein Pin als gefallen
MAX_ROLL_TIME = 7.0
PIT_Y = 810.0

SETUP, PLAY, OVER = "setup", "play", "over"
DIFFS = ["easy", "normal", "pro"]
STEPS = ["pos", "aim", "spin", "power"]
# Pendelgeschwindigkeit der vier Regler je Schwierigkeit
SWING = {"easy": 0.72, "normal": 1.05, "pro": 1.45}
SCATTER = {"easy": 0.20, "normal": 0.45, "pro": 0.80}

# Standard-Aufstellung: (Pin-Nummer, x, y) - Kopfpin zuerst
PIN_SETUP = [
    (1, 0.0, PIN_Y),
    (2, -PIN_DX, PIN_Y + ROW_DY), (3, PIN_DX, PIN_Y + ROW_DY),
    (4, -2 * PIN_DX, PIN_Y + 2 * ROW_DY), (5, 0.0, PIN_Y + 2 * ROW_DY),
    (6, 2 * PIN_DX, PIN_Y + 2 * ROW_DY),
    (7, -3 * PIN_DX, PIN_Y + 3 * ROW_DY), (8, -PIN_DX, PIN_Y + 3 * ROW_DY),
    (9, PIN_DX, PIN_Y + 3 * ROW_DY), (10, 3 * PIN_DX, PIN_Y + 3 * ROW_DY),
]


class _Pin:
    __slots__ = ("num", "x", "y", "hx", "hy", "vx", "vy", "down", "spin")

    def __init__(self, num, x, y):
        self.num = num
        self.x = self.hx = x
        self.y = self.hy = y
        self.vx = self.vy = 0.0
        self.down = False
        self.spin = 0.0

    def moved(self):
        return math.hypot(self.x - self.hx, self.y - self.hy)


def score_frames(rolls):
    """Offizielle Wertung; liefert je Frame die laufende Summe (None = offen)."""
    out = []
    total = 0
    i = 0
    for _ in range(10):
        if i >= len(rolls):
            out.append(None)
            continue
        if rolls[i] == 10:                                    # Strike
            if i + 2 < len(rolls):
                total += 10 + rolls[i + 1] + rolls[i + 2]
                out.append(total)
            else:
                out.append(None)
            i += 1
        elif i + 1 < len(rolls):
            if rolls[i] + rolls[i + 1] == 10:                 # Spare
                if i + 2 < len(rolls):
                    total += 10 + rolls[i + 2]
                    out.append(total)
                else:
                    out.append(None)
            else:
                total += rolls[i] + rolls[i + 1]
                out.append(total)
            i += 2
        else:
            out.append(None)
            i += 1
    return out


def total_score(rolls):
    """Aktuelle Gesamtsumme (letzte gewertete Frame-Summe)."""
    vals = [v for v in score_frames(rolls) if v is not None]
    return vals[-1] if vals else 0


class BowlingGame(Game):
    name = "Bowling"
    highscore_key = "bowling"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        gs = self.settings.get("bowling", {}) if isinstance(self.settings, dict) else {}
        self.diff = gs.get("difficulty", "normal")
        if self.diff not in DIFFS:
            self.diff = "normal"
        self.guide = bool(gs.get("guide", True))
        # Replay: Aufnahme der laufenden Partie (rec) und die fertige
        # Wiederholung der letzten Partie (replay, siehe replay.py).
        self.rec = None
        self.replay = None
        self.replay_request = None
        self._rep = None
        self._rec_pins = {}

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
        self._card = ui.font(max(10, h // 44), mono=True)
        self._huge = ui.font(max(24, h // 13), bold=True)

    def on_surface_changed(self):
        self._build_fonts()
        self._layout()
        self._build_setup_layout()
        self._over_cache = None

    def _layout(self):
        """Perspektive: Kamera hinter der Foullinie, Fluchtpunkt über der Bahn."""
        self.hud_h = 40
        self.card_h = max(34, self._card.get_height() * 2 + 12)
        top = self.hud_h + self.card_h + 6
        near_y = self.height - 12
        far_y = top + 18
        self.cx = self.width / 2.0
        self.depth = 340.0
        # Fluchtpunkt aus Nah-/Fernkante ableiten
        k = (PIT_Y + self.depth) / self.depth
        self.hor = (k * far_y - near_y) / (k - 1.0)
        self.ky = (near_y - self.hor) * self.depth
        # Querskalierung: die Bahn samt Rinnen füllt vorn ~86 % der Breite
        half = (LW / 2 + GUT)
        self.kx = (self.width * 0.43) * self.depth / half

    # ------------------------------------------------------- Speicherstand
    def _load_best(self):
        data = store.load_section("bowling")
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
        if score > self.best.get(self.diff, 0):
            self.best[self.diff] = int(score)
            store.save_section("bowling", {"best": self.best})

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("bowling", {})[key] = value
            settings_mod.save_settings(self.settings)

    # ------------------------------------------------------- Partie
    def _new_game(self):
        self.players = 2 if self.multiplayer else 1
        self.rolls = [[] for _ in range(self.players)]
        self.frame = [0] * self.players
        self.player = 0
        self.winner = None
        self.strike_run = 0
        self.score = 0
        self.game_over = False
        self.msg = None
        self.msg_t = 0.0
        self._rec_new()
        self._rack(full=True)
        self._new_delivery()

    def _rack(self, full=True):
        if full:
            self.pins = [_Pin(n, x, y) for (n, x, y) in PIN_SETUP]
        else:
            self.pins = [p for p in self.pins if not p.down]
            for p in self.pins:
                p.x, p.y = p.hx, p.hy
                p.vx = p.vy = 0.0
                p.spin = 0.0

    def _new_delivery(self):
        self.step = 0
        self.t_osc = 0.0
        self.manual = False
        self.pos = 0.0            # -1 .. 1 (Standpunkt quer zur Bahn)
        self.aim = 0.0            # -1 .. 1 (Winkel)
        self.spin = 0.0           # -1 .. 1 (Effet)
        self.power = 0.5          # 0 .. 1
        self.ball = None
        self.roll_t = 0.0
        self.pins_before = sum(1 for p in self.pins if not p.down)
        self.result_key = None

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

        self.diff_rects = row(y0, 3)
        self.guide_rects = row(y0 + 88, 2)
        self.start_rect = pygame.Rect(cx - 95, y0 + 156, 190, 46)

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3"):
                self.diff = DIFFS[int(k) - 1]
                self._save_setting("difficulty", self.diff)
                self.play_sound("click")
            elif k in ("g", "G"):
                self._toggle_guide()
            elif k in ("Return", "space"):
                self._start_play()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.diff_rects):
                if rc.collidepoint(event.pos):
                    self.diff = DIFFS[i]
                    self._save_setting("difficulty", self.diff)
                    self.play_sound("click")
                    return
            for i, rc in enumerate(self.guide_rects):
                if rc.collidepoint(event.pos):
                    if self.guide != (i == 0):
                        self._toggle_guide()
                    return
            if self.start_rect.collidepoint(event.pos):
                self._start_play()

    def _toggle_guide(self):
        self.guide = not self.guide
        self._save_setting("guide", self.guide)
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
        if event.kind == InputEvent.KEYDOWN and event.key in ("g", "G"):
            self._toggle_guide()
            return
        if self.state == OVER:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._restart()
                elif event.key in ("s", "S"):
                    self.state = SETUP
                    self.game_over = False
                    self.play_sound("click")
                elif event.key in ("p", "P") and self.replay is not None:
                    self._open_replay()
            elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
                self._restart()
            return
        if self.state != PLAY or self.ball is not None:
            return
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k == "Left" or self.is_action(k, "left"):
                self._adjust(-0.06)
            elif k == "Right" or self.is_action(k, "right"):
                self._adjust(0.06)
            elif k in ("space", "Return") or self.is_action(k, "action"):
                self._confirm_step()
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            self._confirm_step()

    def _adjust(self, d):
        """Regler von Hand verstellen (stoppt das Pendeln)."""
        self.manual = True
        name = STEPS[self.step]
        lo = 0.0 if name == "power" else -1.0
        val = max(lo, min(1.0, getattr(self, name) + d))
        setattr(self, name, val)
        self.play_sound("move")

    def _confirm_step(self):
        self.play_sound("click")
        self.step += 1
        self.manual = False
        self.t_osc = 0.0
        if self.step >= len(STEPS):
            self._throw()

    def _throw(self):
        scatter = SCATTER[self.diff]
        x0 = self.pos * (LW / 2 - BALL_R - 1.0)
        ang = math.radians(self.aim * 4.2 + random.uniform(-scatter, scatter) * 0.6)
        speed = 240.0 + 210.0 * self.power
        self.ball = {"x": x0, "y": 0.0,
                     "vx": math.sin(ang) * speed,
                     "vy": math.cos(ang) * speed,
                     "spin": self.spin + random.uniform(-scatter, scatter) * 0.10,
                     "gutter": False, "roll": 0.0}
        self.roll_t = 0.0
        if self.rec:
            pins = [[p.num, round(p.x, 1), round(p.y, 1)] for p in self.pins]
            self._rec_pins = {i: (pn[1], pn[2], 0) for i, pn in enumerate(pins)}
            self.rec.scene(player=self.player, frame=self.frame[self.player],
                           pins=pins, pos=round(self.pos, 3),
                           aim=round(self.aim, 3), spin=round(self.spin, 3),
                           power=round(self.power, 3))
        self.play_sound("shoot")
        self.rumble(80)

    # ===================================================== Update / Physik
    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None
        if self.state != PLAY:
            return
        if self.ball is None:
            if not self.manual:
                self.t_osc += dt * SWING[self.diff]
                name = STEPS[self.step]
                w = math.sin(self.t_osc * 2.4)
                setattr(self, name, abs(w) if name == "power" else w)
            return
        self.roll_t += dt
        self._step_ball(dt)
        self._step_pins(dt)
        if self.rec:
            self.rec.tick(dt, self._rec_sample)
        if self._roll_done():
            self._finish_roll()

    def _step_ball(self, dt):
        b = self.ball
        speed = math.hypot(b["vx"], b["vy"]) + 1.0
        steps = max(2, min(24, int(speed * dt / PIN_R) + 2))
        h = dt / steps
        for _ in range(steps):
            if not b["gutter"]:
                if b["y"] > OIL_END:
                    # Der Hook greift, sobald das Öl ausläuft
                    grip = min(1.0, (b["y"] - OIL_END) / 180.0)
                    b["vx"] += b["spin"] * HOOK * grip * h
            f = max(0.0, 1.0 - BALL_FRIC * h)
            b["vx"] *= f
            b["vy"] *= f
            b["x"] += b["vx"] * h
            b["y"] += b["vy"] * h
            b["roll"] += b["vy"] * h / 24.0
            limit = LW / 2 - BALL_R
            if not b["gutter"] and abs(b["x"]) > limit:
                b["gutter"] = True
                b["vx"] = 0.0
                b["spin"] = 0.0
                b["x"] = math.copysign(LW / 2 + GUT / 2 - BALL_R * 0.4, b["x"])
                self.play_sound("hit")
            if not b["gutter"]:
                for p in self.pins:
                    self._hit_pin(b, p)

    def _hit_pin(self, b, p):
        dx, dy = p.x - b["x"], p.y - b["y"]
        d = math.hypot(dx, dy)
        rad = BALL_R + PIN_R
        if d >= rad or d < 1e-9:
            return
        ux, uy = dx / d, dy / d
        overlap = rad - d
        p.x += ux * overlap
        p.y += uy * overlap
        rvx, rvy = p.vx - b["vx"], p.vy - b["vy"]
        vn = rvx * ux + rvy * uy
        if vn > 0:
            return
        j = -(1 + REST) * vn / (1 / PIN_M + 1 / BALL_M)
        p.vx += j * ux / PIN_M
        p.vy += j * uy / PIN_M
        b["vx"] -= j * ux / BALL_M
        b["vy"] -= j * uy / BALL_M
        p.spin += random.uniform(-3.0, 3.0)
        if not p.down:
            self.play_sound("lock")

    def _step_pins(self, dt):
        steps = 3
        h = dt / steps
        for _ in range(steps):
            for p in self.pins:
                if p.vx == 0.0 and p.vy == 0.0:
                    continue
                f = max(0.0, 1.0 - PIN_FRIC * h)
                p.vx *= f
                p.vy *= f
                p.x += p.vx * h
                p.y += p.vy * h
                if abs(p.vx) + abs(p.vy) < 1.2:
                    p.vx = p.vy = 0.0
            for i in range(len(self.pins)):
                for k in range(i + 1, len(self.pins)):
                    self._pin_pin(self.pins[i], self.pins[k])
            for p in self.pins:
                if not p.down and p.moved() > DOWN_DIST:
                    p.down = True

    @staticmethod
    def _pin_pin(a, b):
        dx, dy = b.x - a.x, b.y - a.y
        d = math.hypot(dx, dy)
        rad = PIN_R * 2
        if d >= rad or d < 1e-9:
            return
        ux, uy = dx / d, dy / d
        push = (rad - d) / 2
        a.x -= ux * push
        a.y -= uy * push
        b.x += ux * push
        b.y += uy * push
        vn = (b.vx - a.vx) * ux + (b.vy - a.vy) * uy
        if vn > 0:
            return
        j = -(1 + REST) * vn / (2 / PIN_M)
        a.vx -= j * ux / PIN_M
        a.vy -= j * uy / PIN_M
        b.vx += j * ux / PIN_M
        b.vy += j * uy / PIN_M

    def _roll_done(self):
        b = self.ball
        if b["y"] > PIT_Y:
            return all(p.vx == 0.0 and p.vy == 0.0 for p in self.pins)
        return self.roll_t > MAX_ROLL_TIME

    # ===================================================== Wurf auswerten
    def _finish_roll(self):
        knocked = sum(1 for p in self.pins if p.down)
        standing = self.pins_before - knocked
        self.rolls[self.player].append(knocked)
        rolls = self.rolls[self.player]
        frame = self.frame[self.player]
        strike = (knocked == 10 and self.pins_before == 10)
        spare = (not strike and standing == 0)

        if strike:
            self.strike_run += 1
            self.result_key = "bowl.strike"
            self.play_sound("win")
            if self.strike_run >= 3:
                self.ach_event("bowl_turkey")
        elif spare:
            self.strike_run = 0
            self.result_key = "bowl.spare"
            self.play_sound("level")
        else:
            self.strike_run = 0
            self.result_key = "bowl.pins" if knocked else "bowl.miss"
            self.play_sound("point" if knocked else "hit")
        self._say(t(self.result_key, n=knocked), 1.8)
        if self.rec:
            self.rec.close(self._rec_sample, knocked=knocked,
                           result=self.result_key)

        done, refill = self._frame_state(frame, rolls, strike, spare)
        self.ball = None
        if done:
            self.frame[self.player] += 1
            if self.frame[self.player] >= 10:
                self._next_player_or_end()
                return
            self._rack(full=True)
            if self.players > 1:
                self._switch_player()
            self._new_delivery()
        else:
            self._rack(full=refill)
            self._new_delivery()
        self._sync_score()

    def _frame_state(self, frame, rolls, strike, spare):
        """(frame_fertig, neue_volle_aufstellung) nach dem aktuellen Wurf."""
        in_frame = self._rolls_in_frame(rolls, frame)
        if frame < 9:
            if strike:
                return True, True
            return (in_frame >= 2), False
        # Zehntes Frame: Bonuswürfe nach Strike/Spare
        if in_frame == 1:
            return False, strike
        if in_frame == 2:
            first, second = rolls[-2], rolls[-1]
            if first == 10:
                # Nach dem Strike gibt es zwei Bonuswürfe; neu aufgestellt
                # wird nur, wenn auch der zweite Ball alle Pins geräumt hat.
                return False, second == 10
            if first + second == 10:
                return False, True
            return True, True
        return True, True

    @staticmethod
    def _rolls_in_frame(rolls, frame):
        """Zählt die Würfe, die im angegebenen Frame liegen."""
        i = 0
        for f in range(10):
            start = i
            if i >= len(rolls):
                return 0
            if f == 9:
                return len(rolls) - start
            if rolls[i] == 10:
                i += 1
            else:
                i += 2
            if f == frame:
                return min(len(rolls), i) - start
        return 0

    def _switch_player(self):
        for i in range(1, self.players + 1):
            nxt = (self.player + i) % self.players
            if self.frame[nxt] < 10:
                self.player = nxt
                return

    def _next_player_or_end(self):
        if all(f >= 10 for f in self.frame):
            self._end_game()
            return
        self._switch_player()
        self._rack(full=True)
        self._new_delivery()
        self._sync_score()

    def _end_game(self):
        final = total_score(self.rolls[0])
        self.score = final
        self._save_best(final)
        if final >= 200:
            self.ach_event("bowl_200", final)
        if self.multiplayer:
            s0, s1 = total_score(self.rolls[0]), total_score(self.rolls[1])
            self.winner = 0 if s0 > s1 else (1 if s1 > s0 else None)
        else:
            self.report_result(final >= 100)
        self._rec_finish(final)
        self.state = OVER
        self.game_over = True
        self.play_sound("gameover")

    def _restart(self):
        self._new_game()
        self.state = PLAY
        self.play_sound("click")

    def _step_name(self):
        """Name des aktiven Reglers.

        Nach dem letzten Wurf einer Partie steht ``step`` auf len(STEPS)
        (der Wurf ist gefallen, ein neuer Anlauf kommt nicht mehr) - der
        Wert wird deshalb gedeckelt.
        """
        return STEPS[min(self.step, len(STEPS) - 1)]

    def _sync_score(self):
        self.score = total_score(self.rolls[0])

    def _say(self, text, secs):
        self.msg = text
        self.msg_t = secs

    # ===================================================== Projektion
    def _project(self, x, y, z=0.0):
        inv = 1.0 / (y + self.depth)
        s = self.kx * inv
        return (self.cx + x * s, self.hor + self.ky * inv - z * s, s)

    # ===================================================== Replay-Aufnahme
    # Aufgezeichnet wird je Wurf die tatsächliche Bahn von Ball und Pins
    # (siehe replay.py). Pins stehen nur dann in einem Sample, wenn sie sich
    # bewegt haben - die ersten zwei Sekunden eines Wurfs kosten so nur drei
    # Zahlen je Bild.

    def _rec_new(self):
        """Startet die Aufzeichnung der Partie (falls Replays an sind)."""
        self.replay = None
        self._rec_pins = {}
        self.rec = replay.recorder("bowling", self.settings, meta={
            "diff": self.diff, "players": self.players})

    def _rec_sample(self):
        """Ein Sample: Ball (x, y, Drehung) + die bewegten Pins als Deltas."""
        b = self.ball
        if b is None:
            return None
        out = [round(b["x"], 1), round(b["y"], 1), round(b["roll"], 2)]
        for i, p in enumerate(self.pins):
            state = (round(p.x, 1), round(p.y, 1), 1 if p.down else 0)
            if self._rec_pins.get(i) != state:
                self._rec_pins[i] = state
                out.append(i)
                out.extend(state)
        return out

    def _rec_finish(self, final):
        """Partie-Ende: die Aufnahme als self.replay bereitlegen."""
        if not self.rec:
            return
        self.replay = self.rec.result(
            title=t("bowl.diff." + self.diff),
            sub=t("bowl.final", n=final), score=final)
        self.rec = None

    def _open_replay(self):
        """Partie-Ende: die Wiederholung ansehen (Taste P).

        Den Screen öffnet main.py - das Spiel legt nur den Wunsch ab.
        """
        if self.replay is not None:
            self.replay_request = self.replay
            self.play_sound("click")

    # ===================================================== Replay-Wiedergabe
    # Der Replay-Screen (replayview.py) baut eine ganz normale Spielinstanz
    # und fährt sie über diese drei Methoden durch die Aufnahme - so
    # zeichnet die Wiederholung mit demselben Code wie das Spiel selbst.

    def replay_begin(self, rep):
        """Schaltet diese Instanz auf reine Wiedergabe um."""
        self.rec = None
        self.replay = None
        self.replay_request = None
        self._rep = rep
        self._rep_at = None
        meta = rep.get("meta") or {}
        if meta.get("diff") in DIFFS:
            self.diff = meta["diff"]
        self.players = max(1, min(2, int(meta.get("players", 1) or 1)))
        self.multiplayer = self.players > 1
        self.state = PLAY
        self.game_over = False
        self.msg = None
        self.msg_t = 0.0
        self.manual = True
        self.step = len(STEPS) - 1
        self.replay_seek(0, 0)

    def replay_seek(self, index, frame):
        """Setzt Pins, Ball und Scorecard auf Szene 'index', Sample 'frame'."""
        scenes = self._rep.get("scenes", [])
        if not scenes:
            return
        index = max(0, min(len(scenes) - 1, index))
        sc = scenes[index]
        frames = sc.get("f") or []
        n = max(1, len(frames))
        frame = max(0, min(n - 1, frame))
        last = (frame >= n - 1)

        # Scorecard: alle abgeschlossenen Würfe bis hierher.
        self.rolls = [[] for _ in range(self.players)]
        self.frame = [0] * self.players
        for prev in scenes[:index] + ([sc] if last else []):
            p = min(self.players - 1, max(0, prev.get("player", 0)))
            if prev.get("knocked") is not None:
                self.rolls[p].append(int(prev["knocked"]))
        for prev in scenes[:index + 1]:
            p = min(self.players - 1, max(0, prev.get("player", 0)))
            self.frame[p] = int(prev.get("frame", 0))
        self.player = min(self.players - 1, max(0, sc.get("player", 0)))
        self.pos = float(sc.get("pos", 0.0))
        self.aim = float(sc.get("aim", 0.0))
        self.spin = float(sc.get("spin", 0.0))
        self.power = float(sc.get("power", 0.5))
        self.result_key = sc.get("result")

        # Pins: bei einem Sprung neu aufstellen, sonst die Deltas fortschreiben.
        if (self._rep_at is None or self._rep_at[0] != index
                or frame < self._rep_at[1]):
            self.pins = [_Pin(int(p[0]), float(p[1]), float(p[2]))
                         for p in sc.get("pins", [])]
            start = 0
        else:
            start = self._rep_at[1] + 1
        for k in range(start, frame + 1):
            fr = frames[k]
            for j in range(3, len(fr) - 3, 4):
                i = int(fr[j])
                if 0 <= i < len(self.pins):
                    p = self.pins[i]
                    p.x, p.y = float(fr[j + 1]), float(fr[j + 2])
                    p.down = bool(fr[j + 3])
        fr = frames[frame]
        self.ball = {"x": float(fr[0]), "y": float(fr[1]), "vx": 0.0, "vy": 0.0,
                     "spin": 0.0, "gutter": False, "roll": float(fr[2])}
        self.msg = (t(sc["result"], n=sc.get("knocked", 0))
                    if (last and sc.get("result")) else None)
        self._rep_at = (index, frame)

    def replay_draw(self, aiming=False, banner=False):
        """Zeichnet den aktuellen Replay-Stand (ohne Menü-Overlay)."""
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        ball = self.ball
        if aiming:
            # Vorlauf: Standpunkt, Ziellinie und Kraft wie beim Wurf selbst.
            self.ball = None
        self._draw_lane(s)
        self._draw_pins(s)
        if self.ball is not None:
            self._draw_ball(s)
        else:
            self._draw_controls(s)
        self._draw_hud(s)
        self.ball = ball
        self._draw_card(s)

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        if self.state == SETUP:
            self._draw_setup(s)
            return
        self._draw_lane(s)
        self._draw_pins(s)
        if self.ball is not None:
            self._draw_ball(s)
        elif self.state == PLAY:
            self._draw_controls(s)
        self._draw_hud(s)
        self._draw_card(s)
        if self.state == OVER:
            self._draw_over(s)

    def _draw_lane(self, s):
        # Rückwand
        far = self._project(0, PIT_Y)
        pygame.draw.rect(s, COL_BACK, (0, self.hud_h + self.card_h + 4,
                                       self.width, far[1] - self.hud_h
                                       - self.card_h))
        # Rinnen + Bahn als Trapeze
        def quad(x0, x1, col):
            a = self._project(x0, 0.0)
            b = self._project(x1, 0.0)
            c = self._project(x1, PIT_Y)
            d = self._project(x0, PIT_Y)
            pygame.draw.polygon(s, col, [(a[0], a[1]), (b[0], b[1]),
                                         (c[0], c[1]), (d[0], d[1])])

        quad(-LW / 2 - GUT, -LW / 2, COL_GUTTER)
        quad(LW / 2, LW / 2 + GUT, COL_GUTTER)
        quad(-LW / 2, LW / 2, COL_LANE)
        # Bretter
        for i in range(1, 10):
            x = -LW / 2 + LW * i / 10.0
            a = self._project(x, 0.0)
            b = self._project(x, PIT_Y)
            pygame.draw.line(s, COL_BOARD, (a[0], a[1]), (b[0], b[1]), 1)
        # Pin-Deck
        a = self._project(-LW / 2, PIN_Y - 26)
        b = self._project(LW / 2, PIN_Y - 26)
        c = self._project(LW / 2, PIT_Y)
        d = self._project(-LW / 2, PIT_Y)
        pygame.draw.polygon(s, COL_DECK, [(a[0], a[1]), (b[0], b[1]),
                                          (c[0], c[1]), (d[0], d[1])])
        # Pfeile
        for i in range(-3, 4):
            x = i * 5.0
            y = 180.0 + abs(i) * 22.0
            p0 = self._project(x, y)
            p1 = self._project(x - 1.6, y - 16)
            p2 = self._project(x + 1.6, y - 16)
            pygame.draw.polygon(s, COL_ARROW, [(p0[0], p0[1]), (p1[0], p1[1]),
                                               (p2[0], p2[1])])
        # Foullinie
        fa = self._project(-LW / 2 - GUT, 0.0)
        fb = self._project(LW / 2 + GUT, 0.0)
        pygame.draw.line(s, COL_FOUL, (fa[0], fa[1]), (fb[0], fb[1]), 3)
        pygame.draw.line(s, COL_GUTTER_D, (fa[0], fa[1] + 3), (fb[0], fb[1] + 3), 1)

    def _draw_pins(self, s):
        for p in sorted(self.pins, key=lambda q: -q.y):
            if p.down and p.vx == 0.0 and p.vy == 0.0 and p.moved() > 40:
                continue                       # aus dem Bild gerutscht
            px, py, sc = self._project(p.x, p.y)
            r = max(2, int(PIN_R * sc))
            if p.down:
                # liegender Pin: flache Ellipse in Fallrichtung
                w = max(3, int(PIN_R * 2.6 * sc))
                h = max(2, int(PIN_R * 1.3 * sc))
                pygame.draw.ellipse(s, COL_PIN_D, (px - w // 2, py - h // 2, w, h))
                pygame.draw.ellipse(s, COL_PIN_RED,
                                    (px - w // 2, py - h // 2, w, max(1, h // 3)))
                continue
            top = py - PIN_R * 6.0 * sc
            body = [(px - r, py), (px - r * 0.75, py - r * 2.2),
                    (px - r * 0.42, py - r * 3.4), (px - r * 0.62, py - r * 4.6),
                    (px, py - r * 5.6), (px + r * 0.62, py - r * 4.6),
                    (px + r * 0.42, py - r * 3.4), (px + r * 0.75, py - r * 2.2),
                    (px + r, py)]
            pygame.draw.polygon(s, COL_PIN, body)
            pygame.draw.polygon(s, COL_PIN_D, body, 1)
            if r >= 3:
                pygame.draw.line(s, COL_PIN_RED, (px - r * 0.55, py - r * 3.0),
                                 (px + r * 0.55, py - r * 3.0), max(1, r // 2))
            if top < 0:
                continue

    def _draw_ball(self, s):
        b = self.ball
        px, py, sc = self._project(b["x"], b["y"], BALL_R)
        r = max(3, int(BALL_R * sc))
        sh = self._project(b["x"], b["y"])
        pygame.draw.ellipse(s, (120, 96, 60),
                            (sh[0] - r, sh[1] - r * 0.4, r * 2, r * 0.8))
        pygame.draw.circle(s, COL_BALL, (int(px), int(py)), r)
        pygame.draw.circle(s, COL_BALL_HI, (int(px - r * 0.35), int(py - r * 0.35)),
                           max(1, int(r * 0.32)))
        # Fingerlöcher rotieren mit dem Ball
        if r >= 6:
            for k in range(3):
                a = b["roll"] + k * 2.1
                hx = px + math.cos(a) * r * 0.45
                hy = py + math.sin(a) * r * 0.45 * 0.6
                pygame.draw.circle(s, (18, 22, 60), (int(hx), int(hy)),
                                   max(1, int(r * 0.14)))

    def _draw_controls(self, s):
        """Zielhilfe und der aktive Regler unten am Bild."""
        x0 = self.pos * (LW / 2 - BALL_R - 1.0)
        p0 = self._project(x0, 0.0)
        # Ball an der Foullinie
        r = max(3, int(BALL_R * p0[2]))
        pygame.draw.circle(s, COL_BALL, (int(p0[0]), int(p0[1] - r)), r)
        if self.guide and self.step >= 1:
            ang = math.radians(self.aim * 4.2)
            pts = []
            x, y = x0, 0.0
            vx, vy = math.sin(ang), math.cos(ang)
            spin = self.spin if self.step >= 2 else 0.0
            for i in range(46):
                step = PIN_Y / 46.0
                if y > OIL_END:
                    vx += spin * 0.055 * min(1.0, (y - OIL_END) / 180.0)
                x += vx * step
                y += vy * step
                pr = self._project(max(-LW, min(LW, x)), y)
                pts.append((pr[0], pr[1]))
            for i in range(0, len(pts) - 1, 2):
                pygame.draw.line(s, COL_MARK, pts[i], pts[i + 1], 2)
        # Reglerbalken
        name = self._step_name()
        val = getattr(self, name)
        bw = min(240, self.width - 60)
        bx = self.width // 2 - bw // 2
        by = self.height - 26
        pygame.draw.rect(s, ui.PANEL, (bx - 6, by - 16, bw + 12, 34),
                         border_radius=8)
        pygame.draw.rect(s, ui.BTN, (bx, by, bw, 10), border_radius=5)
        if name == "power":
            pygame.draw.rect(s, self.accent, (bx, by, int(bw * val), 10),
                             border_radius=5)
        else:
            mid = bx + bw // 2
            pygame.draw.line(s, ui.BORDER, (mid, by - 3), (mid, by + 13), 1)
            mx = mid + int(val * bw / 2)
            pygame.draw.rect(s, self.accent, (mx - 4, by - 3, 8, 16),
                             border_radius=3)
        lbl = self._tiny.render(t("bowl.step." + name), True, ui.TEXT)
        s.blit(lbl, lbl.get_rect(midbottom=(self.width // 2, by - 4)))

    def _draw_hud(self, s):
        pygame.draw.rect(s, ui.PANEL, (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h), (self.width, self.hud_h))
        cy = self.hud_h // 2
        left = self._tiny.render(
            t("bowl.frame", n=min(10, self.frame[self.player] + 1)), True,
            ui.TEXT_DIM)
        s.blit(left, left.get_rect(midleft=(12, cy)))
        if self.msg:
            mid, col = self.msg, ui.GOLD
        elif self.multiplayer:
            mid = t("common.player1" if self.player == 0 else "common.player2")
            col = self.accent
        else:
            mid, col = t("bowl.step." + self._step_name()) if self.ball is None \
                else t("bowl.rolling"), ui.TEXT_DIM
        m = self._small.render(mid, True, col)
        s.blit(m, m.get_rect(center=(self.width // 2, cy)))
        tot = self._small.render(str(total_score(self.rolls[self.player])),
                                 True, self.accent)
        s.blit(tot, tot.get_rect(midright=(self.width - 12, cy)))

    def _draw_card(self, s):
        """Scorecard: zehn Frames mit Würfen und laufender Summe."""
        y = self.hud_h + 2
        h = self.card_h - 4
        pygame.draw.rect(s, ui.PANEL, (4, y, self.width - 8, h), border_radius=6)
        pygame.draw.rect(s, ui.BORDER, (4, y, self.width - 8, h), 1, border_radius=6)
        rolls = self.rolls[self.player]
        sums = score_frames(rolls)
        cells = self._frame_cells(rolls)
        fw = (self.width - 16) / 10.0
        for f in range(10):
            fx = 8 + f * fw
            if f:
                pygame.draw.line(s, ui.BORDER, (fx, y + 2), (fx, y + h - 2), 1)
            active = (f == self.frame[self.player])
            marks = " ".join(cells[f])
            im = self._card.render(marks, True, ui.TEXT if active else ui.TEXT_DIM)
            s.blit(im, im.get_rect(midtop=(fx + fw / 2, y + 3)))
            val = sums[f]
            if val is not None:
                v = self._card.render(str(val), True,
                                      self.accent if active else ui.TEXT_DIM)
                s.blit(v, v.get_rect(midbottom=(fx + fw / 2, y + h - 3)))

    @staticmethod
    def _frame_cells(rolls):
        """Wurf-Symbole je Frame: X, /, -, Zahl."""
        cells = [[] for _ in range(10)]
        i = 0
        for f in range(10):
            if i >= len(rolls):
                break
            if f < 9:
                if rolls[i] == 10:
                    cells[f] = ["X"]
                    i += 1
                    continue
                a = rolls[i]
                cells[f] = ["-" if a == 0 else str(a)]
                i += 1
                if i < len(rolls):
                    b = rolls[i]
                    cells[f].append("/" if a + b == 10 else ("-" if b == 0 else str(b)))
                    i += 1
            else:
                prev = None
                while i < len(rolls) and len(cells[f]) < 3:
                    v = rolls[i]
                    if v == 10:
                        cells[f].append("X")
                    elif prev is not None and prev + v == 10 and prev != 10:
                        cells[f].append("/")
                    else:
                        cells[f].append("-" if v == 0 else str(v))
                    prev = v
                    i += 1
        return cells

    def _draw_over(self, s):
        h = 118
        if self._over_cache is None or self._over_cache.get_width() != self.width:
            ov = pygame.Surface((self.width, h), pygame.SRCALPHA)
            ov.fill((14, 12, 10, 214))
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
            head = self._huge.render(t("bowl.game_done"), True, self.accent)
        s.blit(head, head.get_rect(center=(cx, y + 32)))
        sub = self._small.render(
            t("bowl.final", n=total_score(self.rolls[0])), True, ui.TEXT)
        s.blit(sub, sub.get_rect(center=(cx, y + 66)))
        best = self.best.get(self.diff)
        if best:
            b = self._tiny.render(t("bowl.best", n=best), True, ui.GOLD)
            s.blit(b, b.get_rect(center=(cx, y + 88)))
        hint_txt = t("bowl.new_round")
        if self.replay is not None:
            hint_txt += "  ·  " + t("bowl.replay_hint")
        hint = self._tiny.render(hint_txt, True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, y + 106)))

    def _draw_setup(self, s):
        cx = self.width // 2
        title = self._huge.render(t("bowl.title"), True, self.accent)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.13))))
        sub = self._small.render(t("bowl.subtitle"), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(self.height * 0.20))))

        def label(rects, txt):
            im = self._tiny.render(txt, True, ui.TEXT_DIM)
            s.blit(im, im.get_rect(midbottom=(cx, rects[0].top - 4)))

        label(self.diff_rects, t("bowl.lbl_diff"))
        for i, rc in enumerate(self.diff_rects):
            self._btn(s, rc, t("bowl.diff." + DIFFS[i]), self.diff == DIFFS[i])
        label(self.guide_rects, t("bowl.lbl_guide"))
        for i, rc in enumerate(self.guide_rects):
            self._btn(s, rc, t("common.on") if i == 0 else t("common.off"),
                      self.guide == (i == 0))
        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=9)
        pygame.draw.rect(s, self.accent, self.start_rect, 2, border_radius=9)
        st = self.font.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        best = self.best.get(self.diff)
        if best:
            b = self._tiny.render(t("bowl.best", n=best), True, ui.GOLD)
            s.blit(b, b.get_rect(center=(cx, self.start_rect.bottom + 22)))
        hint = self._tiny.render(t("bowl.setup_hint"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, self.height - 14)))

    def _btn(self, s, rc, text, on):
        pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=8)
        pygame.draw.rect(s, self.accent if on else ui.BORDER, rc,
                         2 if on else 1, border_radius=8)
        im = self._small.render(text, True, ui.TEXT if on else ui.TEXT_DIM)
        s.blit(im, im.get_rect(center=rc.center))
