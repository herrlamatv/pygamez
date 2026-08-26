# -*- coding: utf-8 -*-
"""
minigolf.py
===========
Minigolf - 360 Bahnen in 40 Kursen, allein oder zu zweit lokal.

Kurse (im Setup wählbar, wird gespeichert):
  - Classic : 9 handgebaute, freundliche Bahnen (Par 2-4), sanfter Einstieg.
  - Pro     : 9 handgebaute Bahnen mit Inselgrün, Doppelmühle und Wanderblöcken.
  - Tour    : 38 seed-erzeugte Kurse zu je 9 Bahnen (342 Bahnen) mit steigender
              Schwierigkeit - Kurs 7, Bahn 3 sieht überall gleich aus, gespeichert
              werden muss dafür nichts (siehe minigolf_gen.py).
  - Random  : 9 zufällig gezogene Bahnen aus allen Kursen, zufällig gespiegelt.

Die Physik läuft - wie beim Billard - in Teilschritten mit Reibung, damit
nichts ruckt und schnelle Bälle nicht durch Banden tunneln. Untergründe
bremsen unterschiedlich (Grün/Sand), Rampen beschleunigen, Wasser kostet einen
Strafschlag, Gummipuffer geben Tempo zurück, Windmühlen und Wanderblöcke
verlangen Timing.

Steuerung: Maus bewegt die Ziellinie, linke Maustaste gedrückt halten lädt die
Schlagstärke, Loslassen schlägt. R bricht einen geladenen Schlag ab, ohne zu
putten. Alternativ Pfeile links/rechts zielen, hoch/runter Stärke, Leertaste
schlägt. G blendet die Ziellinie um, P schaltet das Aufnehmen um.

Punkte (Highscore) = Summe der Bahnpunkte; je Bahn gibt es mehr Punkte, je
weiter unter Par gespielt wird (Hole-in-One extra).
"""

import math
import random

import pygame

import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent
from i18n import t

from . import minigolf_gen as gen
from .minigolf_gen import CW, CH, BORDER, HOLES_PER_ROUND, make_hole as _hole

# ------------------------------------------------- Identitätsfarben (Platz)
COL_GREEN = (46, 132, 74)
COL_GREEN_D = (38, 112, 62)
COL_FRINGE = (62, 152, 88)
COL_WALL = (122, 84, 52)
COL_WALL_HI = (156, 112, 72)
COL_SAND = (214, 190, 132)
COL_SAND_D = (190, 164, 108)
COL_WATER = (52, 122, 196)
COL_WATER_D = (36, 92, 158)
COL_SLOPE = (60, 150, 92)
COL_BUMPER = (222, 84, 108)
COL_BUMPER_HI = (246, 140, 160)
COL_MILL = (186, 190, 198)
COL_MILL_D = (128, 134, 146)
COL_BALL = (248, 248, 244)
COL_CUP = (16, 22, 18)
COL_FLAG = (226, 72, 72)
COL_AIM = (245, 245, 210)

# ------------------------------------------------------------- Platz / Physik
# CW/CH/BORDER kommen aus minigolf_gen.py (dort steht die einzige Definition).
BR = gen.BALL_R              # Ballradius
CUP_R = 3.0                  # Lochradius
ARM_W = 1.5                  # halbe Breite eines Mühlenflügels

FRIC_GREEN = 1.15            # Rollreibung je Sekunde
FRIC_SAND = 4.60
WALL_E = 0.72                # Bandenrestitution
BUMP_E = 1.18                # Gummipuffer geben Tempo zurück
STOP_EPS = 2.2               # darunter gilt der Ball als still
MAX_SPEED = 205.0            # maximale Schlaggeschwindigkeit
CAPTURE_SPEED = 74.0         # darüber springt der Ball über das Loch
MAX_SHOT_TIME = 14.0
MAX_STROKES = 8              # danach wird die Bahn mit Höchstwert beendet

SETUP, PLAY, HOLE_DONE, OVER = "setup", "play", "holedone", "over"
COURSES = ["classic", "pro", "tour", "random"]


# --------------------------------------------------------- Kurs 1: Classic
HOLES_CLASSIC = [
    # 1 - gerade Bahn mit Trichter
    _hole(2, (50, 140), (50, 26),
          walls=[(20, 60, 14, 8), (66, 60, 14, 8)]),
    # 2 - Dogleg nach rechts
    _hole(3, (24, 140), (76, 30),
          walls=[(38, 40, 10, 74), (60, 96, 26, 8)],
          sand=[(58, 118, 26, 14)]),
    # 3 - Mittelblock mit Sandgürtel
    _hole(3, (50, 142), (50, 22),
          walls=[(38, 66, 24, 20)],
          sand=[(14, 62, 20, 28), (66, 62, 20, 28)]),
    # 4 - Teich links, schmale Passage rechts
    _hole(3, (26, 142), (76, 26),
          water=[(12, 56, 44, 44)],
          walls=[(66, 74, 8, 50)]),
    # 5 - S-Kurve
    _hole(4, (20, 144), (80, 24),
          walls=[(30, 108, 62, 8), (8, 66, 62, 8), (30, 30, 46, 8)]),
    # 6 - Gummipuffer-Feld
    _hole(3, (50, 142), (50, 22),
          bumpers=[(30, 96, 6), (70, 96, 6), (50, 66, 7), (30, 44, 5),
                   (70, 44, 5)]),
    # 7 - Windmühle
    _hole(4, (50, 144), (50, 22),
          walls=[(6, 84, 30, 8), (64, 84, 30, 8)],
          mills=[(50, 88, 13, 2, 1.5)]),
    # 8 - Steigung mit Sandfang
    _hole(3, (50, 144), (50, 20),
          slopes=[(20, 50, 60, 56, 0.0, 34.0)],
          sand=[(20, 112, 24, 16), (56, 112, 24, 16)]),
    # 9 - Wanderblock vor dem Grün
    _hole(4, (26, 142), (74, 26),
          walls=[(44, 34, 8, 46), (44, 106, 8, 40)],
          movers=[(20, 80, 24, 8, 40, 0, 26)],
          bumpers=[(78, 96, 6)]),
]

# ------------------------------------------------------------ Kurs 2: Pro
HOLES_PRO = [
    # 10 - Inselgrün mit schmalem Hals
    _hole(3, (50, 144), (50, 36),
          water=[(10, 14, 80, 12), (10, 26, 26, 22), (64, 26, 26, 22),
                 (10, 48, 34, 12), (56, 48, 34, 12)],
          walls=[(26, 62, 14, 6), (60, 62, 14, 6)],
          slopes=[(20, 72, 30, 40, 11.0, -8.0),
                  (50, 72, 30, 40, -11.0, -8.0)]),
    # 11 - Doppelmühle im Korridor
    _hole(4, (50, 146), (50, 18),
          walls=[(28, 20, 6, 100), (66, 20, 6, 100)],
          mills=[(50, 106, 15, 2, -1.8), (50, 56, 15, 2, 2.2)]),
    # 12 - Puffer-Tunnel
    _hole(4, (18, 142), (82, 26),
          walls=[(34, 100, 8, 46), (58, 46, 8, 46)],
          bumpers=[(50, 122, 7), (24, 74, 6), (76, 74, 6), (50, 34, 6)],
          sand=[(60, 116, 26, 20)]),
    # 13 - Zickzack-Labyrinth
    _hole(5, (14, 146), (86, 20),
          walls=[(24, 118, 70, 7), (6, 92, 70, 7), (24, 66, 70, 7),
                 (6, 40, 70, 7)]),
    # 14 - Wanderschleusen
    _hole(4, (50, 146), (50, 18),
          walls=[(6, 108, 34, 7), (60, 108, 34, 7),
                 (6, 56, 34, 7), (60, 56, 34, 7)],
          movers=[(40, 108, 12, 7, 8, 0, 14), (48, 56, 12, 7, -8, 0, 12)]),
    # 15 - Sandwüste mit Rampe
    _hole(5, (20, 146), (80, 18),
          sand=[(8, 92, 84, 34)],
          slopes=[(20, 30, 60, 54, 0.0, 26.0)],
          walls=[(46, 128, 8, 22)],
          bumpers=[(20, 60, 6), (80, 60, 6)]),
    # 16 - Teichquerung über den Steg
    _hole(4, (50, 146), (50, 20),
          water=[(8, 56, 28, 52), (64, 56, 28, 52)],
          bumpers=[(24, 34, 6), (76, 34, 6)],
          sand=[(40, 116, 20, 14)]),
    # 17 - Kreuzmühle
    _hole(4, (18, 144), (82, 24),
          walls=[(6, 96, 26, 8), (68, 96, 26, 8)],
          mills=[(50, 100, 14, 3, 1.1)],
          sand=[(66, 118, 24, 18)]),
    # 18 - Finale: Tore, Wasser und Rampe
    _hole(5, (50, 148), (50, 18),
          water=[(8, 96, 30, 30), (62, 96, 30, 30)],
          walls=[(38, 92, 8, 6), (54, 92, 8, 6)],
          slopes=[(24, 40, 52, 44, 0.0, 30.0)],
          movers=[(24, 62, 22, 8, 30, 0, 24)],
          bumpers=[(50, 30, 7)]),
]


class MiniGolfGame(Game):
    name = "Minigolf"
    highscore_key = "minigolf"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        gs = self.settings.get("minigolf", {}) if isinstance(self.settings, dict) else {}
        self.course = gs.get("course", "classic")
        if self.course not in COURSES:
            self.course = "classic"
        self.guide = bool(gs.get("guide", True))
        # Aufnehmen: nach MAX_STROKES Schlägen ist die Bahn vorbei. Standard an,
        # im Setup abschaltbar - dann wird bis zum Einlochen weitergespielt.
        self.pickup = bool(gs.get("pickup", True))
        self.tour = max(1, min(gen.TOUR_COURSES, int(gs.get("tour", 1) or 1)))
        self._tour_par = gen.course_par(self.tour)
        self.winner = None

        self._build_fonts()
        self._layout()
        self._build_setup_layout()
        self._over_cache = None
        self.best = self._load_best()
        self._new_round()
        self.state = SETUP

    def _build_fonts(self):
        h = self.height
        self._small = ui.font(max(14, h // 32))
        self._tiny = ui.font(max(11, h // 42))
        self._card = ui.font(max(11, h // 46), mono=True)
        self._huge = ui.font(max(24, h // 13), bold=True)

    def on_surface_changed(self):
        self._build_fonts()
        self._layout()
        self._build_setup_layout()
        self._over_cache = None

    def _layout(self):
        """Maßstab und Nullpunkt des Platzes aus der Spielfläche ableiten."""
        self.hud_h = 44
        avail_h = self.height - self.hud_h - 12
        card_w = 132 if self.width >= 560 else 108
        self.scale = max(1.2, min((self.width - card_w - 40) / CW,
                                  avail_h / CH))
        self.ox = (self.width - card_w - 12) / 2.0 - CW * self.scale / 2.0
        self.oy = self.hud_h + (avail_h - CH * self.scale) / 2.0 + 6
        self.card_x = self.width - card_w - 6
        self.card_w = card_w

    # ------------------------------------------------------- Speicherstand
    def _load_best(self):
        """Bestwerte je Kurs: {"classic": schlaege, ...} (kleiner = besser)."""
        data = store.load_section("minigolf")
        best = data.get("best") if isinstance(data, dict) else None
        out = {}
        if isinstance(best, dict):
            for k, v in best.items():
                try:
                    out[str(k)] = int(v)
                except (TypeError, ValueError):
                    continue
        return out

    def _best_key(self):
        """Schlüssel des Bestwerts: je Tour-Kurs ein eigener Eintrag."""
        return "tour%d" % self.tour if self.course == "tour" else self.course

    def _save_best(self, strokes):
        key = self._best_key()
        old = self.best.get(key)
        if old is None or strokes < old:
            self.best[key] = strokes
            store.save_section("minigolf", {"best": self.best})

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("minigolf", {})[key] = value
            settings_mod.save_settings(self.settings)

    # ------------------------------------------------------- Runde / Bahn
    def _round_holes(self):
        """Die 9 Bahnen der Runde (Random: gezogen und zufällig gespiegelt)."""
        if self.course == "classic":
            return [dict(h) for h in HOLES_CLASSIC]
        if self.course == "pro":
            return [dict(h) for h in HOLES_PRO]
        if self.course == "tour":
            return gen.course_holes(self.tour)
        # Random: aus handgebauten UND erzeugten Bahnen ziehen
        pool = list(HOLES_CLASSIC + HOLES_PRO)
        for _ in range(HOLES_PER_ROUND):
            pool.append(gen.generate(random.randint(1, gen.TOUR_COURSES),
                                     random.randrange(HOLES_PER_ROUND)))
        picked = random.sample(pool, HOLES_PER_ROUND)
        picked.sort(key=lambda h: h["par"])
        return [self._mirror(h, random.random() < 0.5) for h in picked]

    @staticmethod
    def _mirror(hole, flip):
        """Spiegelt eine Bahn an der Mittelachse (für den Zufallskurs)."""
        if not flip:
            return dict(hole)

        def mx(x, w=0.0):
            return CW - x - w

        out = dict(hole)
        out["tee"] = (mx(hole["tee"][0]), hole["tee"][1])
        out["cup"] = (mx(hole["cup"][0]), hole["cup"][1])
        for key in ("walls", "sand", "water"):
            out[key] = [(mx(x, w), y, w, h) for (x, y, w, h) in hole[key]]
        out["slopes"] = [(mx(x, w), y, w, h, -ax, ay)
                         for (x, y, w, h, ax, ay) in hole["slopes"]]
        out["bumpers"] = [(mx(x), y, r) for (x, y, r) in hole["bumpers"]]
        out["movers"] = [(mx(x, w), y, w, h, -dx, dy, sp)
                         for (x, y, w, h, dx, dy, sp) in hole["movers"]]
        out["mills"] = [(mx(x), y, ln, arms, -sp)
                        for (x, y, ln, arms, sp) in hole["mills"]]
        return out

    def _new_round(self):
        self.holes = self._round_holes()
        self.players = 2 if self.multiplayer else 1
        self.cards = [[0] * len(self.holes) for _ in range(self.players)]
        self.points = [0] * self.players
        self.hole_idx = 0
        self.player = 0
        self.winner = None
        self.score = 0
        self.game_over = False
        self._start_hole()

    def _start_hole(self):
        h = self.holes[self.hole_idx]
        self.hole = h
        self.par = h["par"]
        self.strokes = 0
        self.cup = (float(h["cup"][0]), float(h["cup"][1]))
        self.bx, self.by = float(h["tee"][0]), float(h["tee"][1])
        self.vx = self.vy = 0.0
        self.safe = (self.bx, self.by)
        self.phase = "aim"
        self.power = 0.35
        self.charging = False
        self.shot_time = 0.0
        self.mill_a = 0.0
        self.move_t = 0.0
        self.trail = []
        self.msg = None
        self.msg_t = 0.0
        self.result_key = None
        self.result_pts = 0
        self._aim_at_cup()

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        """Fünf Blöcke: Kurs, Tour-Kurs, Ziellinie, Aufnehmen, Start.

        Die Buttonhöhe folgt der Auflösung, damit auch 480x360 alles zeigt.
        """
        cx = self.width // 2
        bw = min(370, self.width - 50)
        gap = 8
        top = int(self.height * 0.25)
        bottom = self.height - 24
        bh = max(28, min(42, int((bottom - top - 88) / 5)))
        step = bh + 20          # 20 px lassen Platz für die Beschriftung

        def row(y, n, h=None):
            h = bh if h is None else h
            cw = (bw - gap * (n - 1)) / n
            return [pygame.Rect(int(cx - bw / 2 + i * (cw + gap)), y,
                                int(cw), h) for i in range(n)]

        y = top
        self.course_rects = row(y, 4)
        y += step
        # Tour-Kurs: Pfeil links, Anzeige, Pfeil rechts
        self.tour_rects = [pygame.Rect(int(cx - bw / 2), y, 40, bh),
                           pygame.Rect(int(cx - bw / 2 + 46), y, int(bw - 92), bh),
                           pygame.Rect(int(cx + bw / 2 - 40), y, 40, bh)]
        y += step
        self.guide_rects = row(y, 2)
        y += step
        self.pickup_rects = row(y, 2)
        y += step + 4
        self.start_rect = pygame.Rect(cx - 95, y, 190, bh + 4)

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3", "4"):
                self.course = COURSES[int(k) - 1]
                self._save_setting("course", self.course)
                self.play_sound("click")
            elif k == "Left" or self.is_action(k, "left"):
                self._step_tour(-1)
            elif k == "Right" or self.is_action(k, "right"):
                self._step_tour(1)
            elif k in ("g", "G"):
                self._toggle_guide()
            elif k in ("p", "P"):
                self._toggle_pickup()
            elif k in ("Return", "space"):
                self._start_play()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.course_rects):
                if rc.collidepoint(event.pos):
                    self.course = COURSES[i]
                    self._save_setting("course", self.course)
                    self.play_sound("click")
                    return
            if self.course == "tour":
                if self.tour_rects[0].collidepoint(event.pos):
                    self._step_tour(-1)
                    return
                if self.tour_rects[2].collidepoint(event.pos):
                    self._step_tour(1)
                    return
            for i, rc in enumerate(self.guide_rects):
                if rc.collidepoint(event.pos):
                    if self.guide != (i == 0):
                        self._toggle_guide()
                    return
            for i, rc in enumerate(self.pickup_rects):
                if rc.collidepoint(event.pos):
                    if self.pickup != (i == 0):
                        self._toggle_pickup()
                    return
            if self.start_rect.collidepoint(event.pos):
                self._start_play()

    def _step_tour(self, d):
        """Blättert durch die Tour-Kurse (nur wirksam, wenn Tour gewählt ist)."""
        if self.course != "tour":
            return
        self.tour = (self.tour - 1 + d) % gen.TOUR_COURSES + 1
        self._tour_par = gen.course_par(self.tour)
        self._save_setting("tour", self.tour)
        self.play_sound("move")

    def _toggle_guide(self):
        self.guide = not self.guide
        self._save_setting("guide", self.guide)
        self.play_sound("select")

    def _toggle_pickup(self):
        self.pickup = not self.pickup
        self._save_setting("pickup", self.pickup)
        self.play_sound("select")

    def _start_play(self):
        self._new_round()
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
        if event.kind == InputEvent.KEYDOWN and event.key in ("p", "P"):
            self._toggle_pickup()
            return
        if self.state == HOLE_DONE:
            if (event.kind == InputEvent.KEYDOWN
                    and event.key in ("Return", "space")) \
                    or (event.kind == InputEvent.MOUSEDOWN and event.button == 1):
                self._advance()
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
        if self.state != PLAY or self.phase != "aim":
            return
        if event.kind == InputEvent.MOUSEMOVE:
            mx, my = self._unproject(*event.pos)
            if abs(mx - self.bx) > 0.4 or abs(my - self.by) > 0.4:
                self.aim = math.atan2(my - self.by, mx - self.bx)
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            self.charging = True
            self.power = 0.05
        elif event.kind == InputEvent.MOUSEUP and event.button == 1:
            if self.charging:
                self.charging = False
                self._strike()
        elif event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k == "Left" or self.is_action(k, "left"):
                self.aim -= math.radians(2.5)
            elif k == "Right" or self.is_action(k, "right"):
                self.aim += math.radians(2.5)
            elif k == "Up" or self.is_action(k, "up"):
                self.power = min(1.0, self.power + 0.05)
            elif k == "Down" or self.is_action(k, "down"):
                self.power = max(0.05, self.power - 0.05)
            elif k in ("r", "R"):
                self._cancel_shot()
            elif k in ("space", "Return"):
                self._strike()

    def _cancel_shot(self):
        """Bricht einen geladenen Schlag ab (Taste R).

        Wer die Maustaste hält und es sich anders überlegt, drückt R: der Ball
        bleibt liegen, der Schlag zählt nicht. Nach dem Loslassen lässt sich
        ganz normal neu aufladen.
        """
        if not self.charging:
            return
        self.charging = False
        self.power = 0.35
        self.msg = t("golf.cancel")
        self.msg_t = 1.4
        self.play_sound("click")

    def _strike(self):
        sp = MAX_SPEED * self.power
        self.vx = math.cos(self.aim) * sp
        self.vy = math.sin(self.aim) * sp
        self.strokes += 1
        self.phase = "rolling"
        self.shot_time = 0.0
        self.safe = (self.bx, self.by)
        self.trail = []
        self.play_sound("shoot")
        self.rumble(50)

    # ===================================================== Projektion
    def _project(self, x, y):
        return (self.ox + x * self.scale, self.oy + y * self.scale)

    def _unproject(self, sx, sy):
        return ((sx - self.ox) / self.scale, (sy - self.oy) / self.scale)

    def _rect_px(self, r):
        px, py = self._project(r[0], r[1])
        return pygame.Rect(int(px), int(py), max(1, int(r[2] * self.scale)),
                           max(1, int(r[3] * self.scale)))

    # ===================================================== Update / Physik
    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None
        if self.state != PLAY:
            return
        self.mill_a += dt
        self.move_t += dt
        if self.phase == "aim":
            if self.charging:
                self.power = min(1.0, self.power + dt * 0.80)
        elif self.phase == "rolling":
            self._physics(dt)
            self.shot_time += dt
            if self.shot_time > MAX_SHOT_TIME:
                self.vx = self.vy = 0.0
            if self.phase == "rolling" and self.vx == 0.0 and self.vy == 0.0:
                self._after_shot()

    def _mover_rect(self, m):
        """Aktuelles Rechteck eines Wanderblocks (pendelt zwischen den Enden)."""
        x, y, w, h, dx, dy, speed = m
        if speed <= 0 or (dx == 0 and dy == 0):
            return (x, y, w, h), (0.0, 0.0)
        span = math.hypot(dx, dy)
        period = 2 * span / speed
        ph = (self.move_t % period) / period
        f = ph * 2 if ph < 0.5 else (1 - ph) * 2
        sign = 1.0 if ph < 0.5 else -1.0
        return ((x + dx * f, y + dy * f, w, h),
                (dx / span * speed * sign, dy / span * speed * sign))

    def _physics(self, dt):
        speed = math.hypot(self.vx, self.vy)
        steps = max(2, min(24, int(speed * dt / BR) + 2))
        h = dt / steps
        for _ in range(steps):
            fr = FRIC_SAND if self._terrain() == "sand" else FRIC_GREEN
            # Rampen beschleunigen, solange der Ball darauf liegt
            for (x, y, w, hh, ax, ay) in self.hole["slopes"]:
                if x <= self.bx <= x + w and y <= self.by <= y + hh:
                    self.vx += ax * h
                    self.vy += ay * h
            f = max(0.0, 1.0 - fr * h)
            self.vx *= f
            self.vy *= f
            self.bx += self.vx * h
            self.by += self.vy * h
            self._collide_bounds()
            for r in self.hole["walls"]:
                self._collide_rect(r)
            for m in self.hole["movers"]:
                rect, vel = self._mover_rect(m)
                self._collide_rect(rect, vel)
            for b in self.hole["bumpers"]:
                self._collide_bumper(b)
            for mill in self.hole["mills"]:
                self._collide_mill(mill)
            if self._check_cup() or self._check_water():
                return
            if self.vx * self.vx + self.vy * self.vy < STOP_EPS * STOP_EPS:
                self.vx = self.vy = 0.0
                return
        self.trail.append((self.bx, self.by))
        if len(self.trail) > 26:
            del self.trail[0]

    def _terrain(self):
        for (x, y, w, h) in self.hole["sand"]:
            if x <= self.bx <= x + w and y <= self.by <= y + h:
                return "sand"
        return "green"

    def _collide_bounds(self):
        lo = BORDER + BR
        hix, hiy = CW - BORDER - BR, CH - BORDER - BR
        if self.bx < lo:
            self.bx, self.vx = lo, abs(self.vx) * WALL_E
            self._thud()
        elif self.bx > hix:
            self.bx, self.vx = hix, -abs(self.vx) * WALL_E
            self._thud()
        if self.by < lo:
            self.by, self.vy = lo, abs(self.vy) * WALL_E
            self._thud()
        elif self.by > hiy:
            self.by, self.vy = hiy, -abs(self.vy) * WALL_E
            self._thud()

    def _thud(self):
        if abs(self.vx) + abs(self.vy) > 45:
            self.play_sound("bounce")

    def _collide_rect(self, r, vel=(0.0, 0.0)):
        """Kreis gegen Rechteck: nächster Punkt, herausschieben, reflektieren."""
        x, y, w, h = r[0], r[1], r[2], r[3]
        nx = max(x, min(self.bx, x + w))
        ny = max(y, min(self.by, y + h))
        dx, dy = self.bx - nx, self.by - ny
        d2 = dx * dx + dy * dy
        if d2 >= BR * BR:
            return
        if d2 > 1e-9:
            d = math.sqrt(d2)
            ux, uy, push = dx / d, dy / d, BR - d
        else:
            # Mittelpunkt im Rechteck -> über die kürzeste Achse hinausschieben
            left, right = self.bx - x, x + w - self.bx
            top, bottom = self.by - y, y + h - self.by
            m = min(left, right, top, bottom)
            if m == left:
                ux, uy, push = -1.0, 0.0, left + BR
            elif m == right:
                ux, uy, push = 1.0, 0.0, right + BR
            elif m == top:
                ux, uy, push = 0.0, -1.0, top + BR
            else:
                ux, uy, push = 0.0, 1.0, bottom + BR
        self.bx += ux * push
        self.by += uy * push
        rvx, rvy = self.vx - vel[0], self.vy - vel[1]
        dot = rvx * ux + rvy * uy
        if dot < 0:
            rvx -= (1 + WALL_E) * dot * ux
            rvy -= (1 + WALL_E) * dot * uy
            self.vx = rvx + vel[0] * 1.4
            self.vy = rvy + vel[1] * 1.4
            self._thud()

    def _collide_bumper(self, b):
        x, y, r = b
        dx, dy = self.bx - x, self.by - y
        d = math.hypot(dx, dy)
        rad = r + BR
        if d >= rad or d < 1e-9:
            return
        ux, uy = dx / d, dy / d
        self.bx, self.by = x + ux * rad, y + uy * rad
        dot = self.vx * ux + self.vy * uy
        if dot < 0:
            self.vx -= (1 + BUMP_E) * dot * ux
            self.vy -= (1 + BUMP_E) * dot * uy
            self.play_sound("bounce")

    def _collide_mill(self, mill):
        x, y, length, arms, speed = mill
        base = self.mill_a * speed
        for i in range(int(arms)):
            a = base + i * (2 * math.pi / int(arms))
            ex, ey = x + math.cos(a) * length, y + math.sin(a) * length
            px, py = self._closest_on_seg(x, y, ex, ey, self.bx, self.by)
            dx, dy = self.bx - px, self.by - py
            d = math.hypot(dx, dy)
            rad = BR + ARM_W
            if d >= rad:
                continue
            if d < 1e-9:
                dx, dy, d = 0.0, -1.0, 1.0
            ux, uy = dx / d, dy / d
            self.bx, self.by = px + ux * rad, py + uy * rad
            # Umfangsgeschwindigkeit des Flügels am Kontaktpunkt
            rvx, rvy = -(py - y) * speed, (px - x) * speed
            relx, rely = self.vx - rvx, self.vy - rvy
            dot = relx * ux + rely * uy
            if dot < 0:
                relx -= 1.8 * dot * ux
                rely -= 1.8 * dot * uy
            self.vx, self.vy = relx + rvx, rely + rvy
            self.play_sound("hit")
            return

    @staticmethod
    def _closest_on_seg(x1, y1, x2, y2, px, py):
        dx, dy = x2 - x1, y2 - y1
        l2 = dx * dx + dy * dy
        if l2 < 1e-9:
            return x1, y1
        f = max(0.0, min(1.0, ((px - x1) * dx + (py - y1) * dy) / l2))
        return x1 + f * dx, y1 + f * dy

    def _check_cup(self):
        dx, dy = self.cup[0] - self.bx, self.cup[1] - self.by
        d = math.hypot(dx, dy)
        sp = math.hypot(self.vx, self.vy)
        if d < CUP_R + BR and sp < CAPTURE_SPEED * 1.6:
            # Sog Richtung Loch - fühlt sich an wie eine echte Lochkante
            pull = 26.0 * (1.0 - d / (CUP_R + BR)) / max(d, 0.4)
            self.vx += dx * pull
            self.vy += dy * pull
        if d < CUP_R * 0.75 and sp < CAPTURE_SPEED:
            self.vx = self.vy = 0.0
            self.bx, self.by = self.cup
            self._holed()
            return True
        return False

    def _check_water(self):
        for (x, y, w, h) in self.hole["water"]:
            if x <= self.bx <= x + w and y <= self.by <= y + h:
                self.vx = self.vy = 0.0
                self.bx, self.by = self.safe
                self.strokes += 1
                self.msg = t("golf.penalty")
                self.msg_t = 2.0
                self.play_sound("hit")
                if self.pickup and self.strokes >= MAX_STROKES:
                    self._finish_hole(holed=False)
                else:
                    self.phase = "aim"
                    self._aim_at_cup()
                return True
        return False

    def _aim_at_cup(self):
        self.aim = math.atan2(self.cup[1] - self.by, self.cup[0] - self.bx)
        self.power = 0.35
        self.charging = False

    def _after_shot(self):
        if self.pickup and self.strokes >= MAX_STROKES:
            self.msg = t("golf.max_strokes")
            self.msg_t = 2.4
            self._finish_hole(holed=False)
            return
        self.phase = "aim"
        self._aim_at_cup()

    # ===================================================== Bahn abschließen
    def _holed(self):
        self.play_sound("win" if self.strokes == 1 else "point")
        self.rumble(90)
        px, py = self._project(*self.cup)
        ui.spawn_burst(px, py, self.accent, n=16)
        self._finish_hole(holed=True)

    def _finish_hole(self, holed):
        self.cards[self.player][self.hole_idx] = self.strokes
        if holed:
            pts = max(100, 600 + (self.par - self.strokes) * 300)
            if self.strokes == 1:
                pts += 500
                self.ach_event("golf_ace")
        else:
            pts = 100
        self.points[self.player] += pts
        self.result_pts = pts
        self.result_key = self._result_key(self.strokes, self.par, holed)
        if self.player == 0:
            self.score = self.points[0]
        self.phase = "done"
        self.state = HOLE_DONE

    @staticmethod
    def _result_key(strokes, par, holed):
        if not holed:
            return "golf.res.max"
        if strokes == 1:
            return "golf.res.ace"
        d = strokes - par
        if d <= -2:
            return "golf.res.eagle"
        if d == -1:
            return "golf.res.birdie"
        if d == 0:
            return "golf.res.par"
        if d == 1:
            return "golf.res.bogey"
        if d == 2:
            return "golf.res.double"
        return "golf.res.over"

    def _advance(self):
        """Nächster Spieler bzw. nächste Bahn (oder Rundenende)."""
        self.play_sound("click")
        if self.player + 1 < self.players:
            self.player += 1
            self.state = PLAY
            self._start_hole()
            return
        self.player = 0
        if self.hole_idx + 1 < len(self.holes):
            self.hole_idx += 1
            self.state = PLAY
            self._start_hole()
            return
        self._end_round()

    def _end_round(self):
        total = sum(self.cards[0])
        par_total = sum(h["par"] for h in self.holes)
        self._save_best(total)
        if total < par_total:
            self.ach_event("golf_under_par")
        if self.multiplayer:
            t0, t1 = sum(self.cards[0]), sum(self.cards[1])
            self.winner = 0 if t0 < t1 else (1 if t1 < t0 else None)
        else:
            self.winner = None
            self.report_result(total <= par_total)
        self.score = self.points[0]
        self.state = OVER
        self.game_over = True
        self.play_sound("win")

    def _restart(self):
        self._new_round()
        self.state = PLAY
        self.play_sound("click")

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        if self.state == SETUP:
            self._draw_setup(s)
            return
        self._draw_course(s)
        self._draw_ball(s)
        if self.state == PLAY and self.phase == "aim":
            self._draw_aim(s)
        self._draw_hud(s)
        self._draw_card(s)
        if self.state == HOLE_DONE:
            self._draw_hole_done(s)
        elif self.state == OVER:
            self._draw_over(s)

    def _draw_course(self, s):
        board = pygame.Rect(int(self.ox), int(self.oy),
                            int(CW * self.scale), int(CH * self.scale))
        pygame.draw.rect(s, COL_FRINGE, board.inflate(10, 10), border_radius=10)
        pygame.draw.rect(s, COL_GREEN, board, border_radius=6)
        # Mähstreifen
        stripe = max(6, int(10 * self.scale))
        for i in range(0, board.h, stripe * 2):
            pygame.draw.rect(s, COL_GREEN_D,
                             (board.x, board.y + i, board.w,
                              min(stripe, board.h - i)))
        # Banden
        b = max(2, int(BORDER * self.scale))
        for r in (pygame.Rect(board.x, board.y, board.w, b),
                  pygame.Rect(board.x, board.bottom - b, board.w, b),
                  pygame.Rect(board.x, board.y, b, board.h),
                  pygame.Rect(board.right - b, board.y, b, board.h)):
            pygame.draw.rect(s, COL_WALL, r)
            pygame.draw.rect(s, COL_WALL_HI, r, 1)
        for (x, y, w, h, ax, ay) in self.hole["slopes"]:
            self._draw_slope(s, (x, y, w, h), ax, ay)
        for r in self.hole["sand"]:
            rc = self._rect_px(r)
            pygame.draw.rect(s, COL_SAND, rc, border_radius=6)
            pygame.draw.rect(s, COL_SAND_D, rc, 2, border_radius=6)
        for r in self.hole["water"]:
            self._draw_water(s, r)
        self._draw_cup(s)
        for r in self.hole["walls"]:
            self._draw_wall(s, self._rect_px(r))
        for m in self.hole["movers"]:
            rect, _ = self._mover_rect(m)
            self._draw_wall(s, self._rect_px(rect), mover=True)
        for (x, y, r) in self.hole["bumpers"]:
            px, py = self._project(x, y)
            rr = max(3, int(r * self.scale))
            pygame.draw.circle(s, COL_BUMPER, (int(px), int(py)), rr)
            pygame.draw.circle(s, COL_BUMPER_HI, (int(px), int(py)),
                               max(2, rr - 3), 2)
        for mill in self.hole["mills"]:
            self._draw_mill(s, mill)

    def _draw_slope(self, s, r, ax, ay):
        rc = self._rect_px(r)
        pygame.draw.rect(s, COL_SLOPE, rc, border_radius=5)
        ang = math.atan2(ay, ax)
        step = max(16, int(14 * self.scale))
        col = ui.mix(COL_SLOPE, (255, 255, 255), 0.22)
        dx, dy = math.cos(ang) * 6, math.sin(ang) * 6
        for yy in range(rc.y + step // 2, rc.bottom - 2, step):
            for xx in range(rc.x + step // 2, rc.right - 2, step):
                pygame.draw.line(s, col, (xx - dx, yy - dy), (xx + dx, yy + dy), 2)
                pygame.draw.circle(s, col, (int(xx + dx), int(yy + dy)), 2)

    def _draw_water(self, s, r):
        rc = self._rect_px(r)
        pygame.draw.rect(s, COL_WATER, rc, border_radius=7)
        tsec = pygame.time.get_ticks() / 1000.0
        for i in range(3):
            yy = rc.y + int(rc.h * (0.25 + 0.25 * i)) + int(
                math.sin(tsec * 1.4 + i) * 3)
            if rc.y < yy < rc.bottom:
                pygame.draw.line(s, COL_WATER_D, (rc.x + 6, yy),
                                 (rc.right - 6, yy), 2)
        pygame.draw.rect(s, COL_WATER_D, rc, 2, border_radius=7)

    @staticmethod
    def _draw_wall(s, rc, mover=False):
        pygame.draw.rect(s, COL_WALL, rc, border_radius=3)
        pygame.draw.rect(s, COL_WALL_HI, rc, 2, border_radius=3)
        if mover:
            pygame.draw.line(s, COL_WALL_HI, (rc.x + 4, rc.centery),
                             (rc.right - 4, rc.centery), 1)

    def _draw_mill(self, s, mill):
        x, y, length, arms, speed = mill
        px, py = self._project(x, y)
        base = self.mill_a * speed
        w = max(3, int(ARM_W * 2 * self.scale))
        for i in range(int(arms)):
            a = base + i * (2 * math.pi / int(arms))
            ex, ey = self._project(x + math.cos(a) * length,
                                   y + math.sin(a) * length)
            pygame.draw.line(s, COL_MILL, (px, py), (ex, ey), w)
            pygame.draw.line(s, COL_MILL_D, (px, py), (ex, ey), 1)
        pygame.draw.circle(s, COL_MILL_D, (int(px), int(py)),
                           max(3, int(2.4 * self.scale)))

    def _draw_cup(self, s):
        px, py = self._project(*self.cup)
        r = max(3, int(CUP_R * self.scale))
        pygame.draw.circle(s, (24, 60, 36), (int(px), int(py)), r + 2)
        pygame.draw.circle(s, COL_CUP, (int(px), int(py)), r)
        top = py - max(16, int(18 * self.scale))
        pygame.draw.line(s, (238, 238, 232), (px, py), (px, top), 2)
        wave = math.sin(pygame.time.get_ticks() / 260.0) * 2
        pygame.draw.polygon(s, COL_FLAG, [(px, top), (px + 14, top + 5 + wave),
                                          (px, top + 11)])

    def _draw_ball(self, s):
        px, py = self._project(self.bx, self.by)
        r = max(2, int(BR * self.scale))
        for i, (tx, ty) in enumerate(self.trail):
            a = (i + 1) / (len(self.trail) + 1)
            tpx, tpy = self._project(tx, ty)
            pygame.draw.circle(s, ui.mix(COL_GREEN, COL_BALL, a * 0.35),
                               (int(tpx), int(tpy)), max(1, int(r * 0.6)))
        pygame.draw.circle(s, (18, 46, 28), (int(px + 2), int(py + 2)), r)
        pygame.draw.circle(s, COL_BALL, (int(px), int(py)), r)
        pygame.draw.circle(s, (206, 210, 200), (int(px), int(py)), r, 1)
        if r >= 5:
            pygame.draw.circle(s, (255, 255, 255),
                               (int(px - r // 3), int(py - r // 3)),
                               max(1, r // 3))

    def _draw_aim(self, s):
        px, py = self._project(self.bx, self.by)
        ox, oy = math.cos(self.aim), math.sin(self.aim)
        if self.guide:
            steps = max(6, int((30 + 60 * self.power) * self.scale / 9))
            for i in range(0, steps, 2):
                pygame.draw.line(s, COL_AIM,
                                 (px + ox * (i * 9 + 6), py + oy * (i * 9 + 6)),
                                 (px + ox * (i * 9 + 12), py + oy * (i * 9 + 12)),
                                 2)
        # Schläger hinter dem Ball
        bx1 = px - ox * (10 + self.power * 26)
        by1 = py - oy * (10 + self.power * 26)
        pygame.draw.line(s, (228, 228, 222), (bx1, by1),
                         (bx1 - ox * 26, by1 - oy * 26), 3)
        pygame.draw.line(s, (150, 154, 160), (bx1, by1),
                         (bx1 - oy * 7, by1 + ox * 7), 5)

    def _draw_hud(self, s):
        pygame.draw.rect(s, ui.PANEL, (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h), (self.width, self.hud_h))
        cy = self.hud_h // 2
        left = self._tiny.render(
            t("golf.hole", n=self.hole_idx + 1, total=len(self.holes))
            + "  ·  " + t("golf.par", n=self.par), True, ui.TEXT_DIM)
        s.blit(left, left.get_rect(midleft=(12, cy)))
        if self.msg:
            mid = self.msg
        elif self.multiplayer:
            mid = t("golf.turn",
                    name=t("common.player1" if self.player == 0 else "common.player2"))
        else:
            mid = t("golf.strokes", n=self.strokes)
        img = self._small.render(mid, True, self.accent)
        s.blit(img, img.get_rect(center=(self.width // 2, cy)))
        mw, mh = 84, 10
        mx = self.width - mw - 14
        pygame.draw.rect(s, ui.BTN, (mx, cy - mh // 2, mw, mh), border_radius=4)
        pygame.draw.rect(s, self.accent,
                         (mx, cy - mh // 2, int(mw * self.power), mh),
                         border_radius=4)

    def _draw_card(self, s):
        """Scorekarte rechts: Bahn, Par und Schläge je Spieler."""
        x, w = self.card_x, self.card_w
        y = self.hud_h + 10
        line = self._card.get_height() + 2
        rect = pygame.Rect(x, y, w, (len(self.holes) + 2) * line + 18)
        if rect.bottom > self.height - 4:
            return
        ui.draw_panel(s, rect, radius=8, shadow=False)
        cy = y + 8
        par_x, me_x, p2_x = x + w - 62, x + w - 38, x + w - 16
        s.blit(self._card.render(t("golf.card_hole"), True, ui.TEXT_DIM),
               (x + 8, cy))
        s.blit(self._card.render(t("golf.card_par"), True, ui.TEXT_DIM), (par_x, cy))
        s.blit(self._card.render("1", True, self.accent), (me_x, cy))
        if self.multiplayer:
            s.blit(self._card.render("2", True, ui.GOLD), (p2_x, cy))
        cy += line + 2
        for i, hole in enumerate(self.holes):
            active = (i == self.hole_idx)
            s.blit(self._card.render(str(i + 1), True,
                                     ui.TEXT if active else ui.TEXT_DIM),
                   (x + 8, cy))
            s.blit(self._card.render(str(hole["par"]), True, ui.TEXT_FAINT),
                   (par_x, cy))
            for p, cx in ((0, me_x), (1, p2_x)):
                if p >= self.players:
                    continue
                v = self.cards[p][i]
                if active and p == self.player and self.state == PLAY:
                    v = self.strokes
                if v:
                    col = ui.GREEN if v < hole["par"] else (
                        ui.RED if v > hole["par"] else ui.TEXT)
                    s.blit(self._card.render(str(v), True, col), (cx, cy))
            cy += line
        pygame.draw.line(s, ui.BORDER, (x + 6, cy + 1), (x + w - 6, cy + 1))
        cy += 5
        s.blit(self._card.render(t("golf.card_sum"), True, ui.TEXT_DIM), (x + 8, cy))
        s.blit(self._card.render(str(sum(h["par"] for h in self.holes)), True,
                                 ui.TEXT_FAINT), (par_x, cy))
        for p, cx in ((0, me_x), (1, p2_x)):
            if p < self.players:
                s.blit(self._card.render(str(sum(self.cards[p])), True,
                                         self.accent if p == 0 else ui.GOLD),
                       (cx, cy))

    def _banner(self, s, h=104):
        if self._over_cache is None or self._over_cache.get_width() != self.width \
                or self._over_cache.get_height() != h:
            ov = pygame.Surface((self.width, h), pygame.SRCALPHA)
            ov.fill((8, 14, 10, 214))
            self._over_cache = ov
        y = self.height // 2 - h // 2
        s.blit(self._over_cache, (0, y))
        pygame.draw.line(s, self.accent, (0, y), (self.width, y))
        pygame.draw.line(s, self.accent, (0, y + h - 1), (self.width, y + h - 1))
        return y

    def _draw_hole_done(self, s):
        y = self._banner(s)
        cx = self.width // 2
        head = self._huge.render(t(self.result_key), True, self.accent)
        s.blit(head, head.get_rect(center=(cx, y + 34)))
        sub = self._small.render(
            t("golf.hole_result", strokes=self.strokes, par=self.par,
              pts=self.result_pts), True, ui.TEXT)
        s.blit(sub, sub.get_rect(center=(cx, y + 66)))
        hint = self._tiny.render(t("golf.next"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, y + 90)))

    def _draw_over(self, s):
        y = self._banner(s, 118)
        cx = self.width // 2
        total = sum(self.cards[0])
        par_total = sum(h["par"] for h in self.holes)
        if self.multiplayer:
            head = self._huge.render(
                t("common.draw") if self.winner is None
                else t("common.player_wins", n=self.winner + 1), True, self.accent)
        else:
            head = self._huge.render(t("golf.round_done"), True, self.accent)
        s.blit(head, head.get_rect(center=(cx, y + 32)))
        d = total - par_total
        sub = self._small.render(
            t("golf.final", strokes=total,
              diff=("%+d" % d) if d else t("golf.even"), pts=self.points[0]),
            True, ui.TEXT)
        s.blit(sub, sub.get_rect(center=(cx, y + 66)))
        best = self.best.get(self._best_key())
        if best:
            b = self._tiny.render(t("golf.best", n=best), True, ui.GOLD)
            s.blit(b, b.get_rect(center=(cx, y + 88)))
        hint = self._tiny.render(t("golf.new_round"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, y + 106)))

    def _draw_setup(self, s):
        cx = self.width // 2
        title = self._huge.render(t("golf.title"), True, self.accent)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.115))))
        sub = self._small.render(t("golf.subtitle"), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(self.height * 0.18))))

        def label(rects, txt):
            im = self._tiny.render(txt, True, ui.TEXT_DIM)
            s.blit(im, im.get_rect(midbottom=(cx, rects[0].top - 4)))

        label(self.course_rects, t("golf.lbl_course"))
        for i, rc in enumerate(self.course_rects):
            self._btn(s, rc, t("golf.course." + COURSES[i]),
                      self.course == COURSES[i])
        self._draw_tour_row(s)
        label(self.guide_rects, t("golf.lbl_guide"))
        for i, rc in enumerate(self.guide_rects):
            self._btn(s, rc, t("common.on") if i == 0 else t("common.off"),
                      self.guide == (i == 0))
        label(self.pickup_rects, t("golf.lbl_pickup"))
        for i, rc in enumerate(self.pickup_rects):
            self._btn(s, rc, t("common.on") if i == 0 else t("common.off"),
                      self.pickup == (i == 0))
        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=9)
        pygame.draw.rect(s, self.accent, self.start_rect, 2, border_radius=9)
        st = self.font.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        best = self.best.get(self._best_key())
        if best:
            b = self._tiny.render(t("golf.best", n=best), True, ui.GOLD)
            s.blit(b, b.get_rect(center=(cx, self.start_rect.bottom + 22)))
        hint = self._tiny.render(t("golf.setup_hint"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, self.height - 14)))

    def _draw_tour_row(self, s):
        """Kurswahl der Tour: Pfeile, Kursnummer und Gesamt-Par."""
        left, mid, right = self.tour_rects
        on = (self.course == "tour")
        im = self._tiny.render(t("golf.lbl_tour"), True,
                               ui.TEXT_DIM if on else ui.TEXT_FAINT)
        s.blit(im, im.get_rect(midbottom=(self.width // 2, left.top - 3)))
        col = self.accent if on else ui.BORDER
        for rc, arrow in ((left, "<"), (right, ">")):
            pygame.draw.rect(s, ui.BTN, rc, border_radius=7)
            pygame.draw.rect(s, col, rc, 1, border_radius=7)
            a = self._small.render(arrow, True, ui.TEXT if on else ui.TEXT_FAINT)
            s.blit(a, a.get_rect(center=rc.center))
        pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, mid, border_radius=7)
        pygame.draw.rect(s, col, mid, 2 if on else 1, border_radius=7)
        txt = t("golf.tour", n=self.tour, total=gen.TOUR_COURSES)
        if on:
            txt += "  ·  " + t("golf.par", n=self._tour_par)
        im = self._small.render(txt, True, ui.TEXT if on else ui.TEXT_FAINT)
        s.blit(im, im.get_rect(center=mid.center))

    def _btn(self, s, rc, text, on):
        pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=8)
        pygame.draw.rect(s, self.accent if on else ui.BORDER, rc,
                         2 if on else 1, border_radius=8)
        im = self._small.render(text, True, ui.TEXT if on else ui.TEXT_DIM)
        s.blit(im, im.get_rect(center=rc.center))
