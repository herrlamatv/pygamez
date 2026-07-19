# -*- coding: utf-8 -*-
"""
tanks.py
=========
Panzer-Duell - 2D-Arena-Duell (1 Spieler gegen KI oder 2 Spieler lokal).

- Panzer drehen und fahren (P1: WASD + Leertaste, P2: laut Tastenbelegung;
  im Einzelspieler steuern beide Belegungen den eigenen Panzer);
  Schüsse prallen EINMAL von Wänden ab (Ricochet) und verschwinden beim
  zweiten Wandkontakt. Auch der eigene Schuss ist gefährlich (kurze
  Schonfrist nach dem Abfeuern).
- Runden: Wer zuerst 5 Runden gewinnt, gewinnt das Match.
- 4 Arenen (offen/Kreuz/Säulen/Labyrinth), wählbar oder zufällige Rotation.
- Power-Ups: Schnellfeuer, Schild (1 Treffer), Dreifach-Schuss.
- KI mit drei Stärken: easy wandert und streut, medium verfolgt und weicht
  aus, hard nutzt Vorhalt UND Ricochet-Schüsse über Wandspiegelung.
- Punkte: Matchsieg gegen die KI = (Stufe+1)*100 + (5-KI-Runden)*20,
  kumulativ über Rematches; Mehrspieler wird nicht gewertet.
"""

import math
import random

import pygame

import settings as settings_mod
import ui
from game_base import Game, InputEvent, LocalizedName
from i18n import t

# Identitätsfarben des Spiels (bewusst fest, unabhängig vom Theme):
# Arena-Boden/-Wände und die beiden Panzerfarben.
COL_FLOOR = (24, 28, 38)
COL_WALL = (72, 80, 100)
COL_WALL_EDGE = (100, 110, 134)
COL_P1 = (120, 180, 90)       # Grün-oliv
COL_P2 = (200, 120, 70)       # Rost
COL_BULLET = (240, 240, 250)
POWER_COLS = {"rapid": (245, 205, 90), "shield": (110, 190, 255),
              "triple": (230, 120, 200)}

DIFFS = ["easy", "medium", "hard"]
ARENAS = ["random", "open", "cross", "pillars", "maze"]
WALL_PRESETS = {
    "open": [],
    "cross": [(0.35, 0.475, 0.30, 0.05), (0.475, 0.35, 0.05, 0.30)],
    "pillars": [(0.18, 0.18, 0.14, 0.14), (0.68, 0.18, 0.14, 0.14),
                (0.18, 0.68, 0.14, 0.14), (0.68, 0.68, 0.14, 0.14)],
    "maze": [(0.0, 0.32, 0.42, 0.045), (0.58, 0.32, 0.42, 0.045),
             (0.30, 0.62, 0.045, 0.38), (0.655, 0.0, 0.045, 0.38)],
}

ROUNDS_TO_WIN = 5
BULLET_SPEED = 320.0
BULLET_LIFE = 6.0
MAX_BULLETS = 4
COOLDOWN = 0.5
GRACE = 0.15                 # Eigentreffer-Schonfrist nach dem Abfeuern

SETUP, COUNTDOWN, PLAY, ROUND_END, MATCH_END = \
    "setup", "countdown", "play", "round_end", "match_end"


def _rgba(color, alpha):
    """Palette-Farbe (RGB) mit einem Alpha-Wert zu RGBA kombinieren."""
    return (color[0], color[1], color[2], alpha)


class TankDuelGame(Game):
    name = LocalizedName("Tank Duel", de="Panzer-Duell", fr="Duel de chars",
                         es="Duelo de tanques", pt="Duelo de tanques")
    highscore_key = "tanks"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        tk = self.settings.get("tanks", {}) if isinstance(self.settings, dict) else {}
        self.diff = max(0, min(2, int(tk.get("difficulty", 1))))
        self.arena_sel = max(-1, min(3, int(tk.get("arena", -1))))

        self._make_fonts()
        self.rounds = [0, 0]
        self._arena_cycle = []
        self._build_setup_layout()
        self.state = SETUP

    def _make_fonts(self):
        """Theme-Schriften, Größen aus der Fensterhöhe abgeleitet."""
        self._small = ui.font(max(13, min(22, self.height // 30)))
        self._tiny = ui.font(max(11, min(18, self.height // 38)))
        self._huge = ui.font(max(26, self.height // 11), bold=True)

    def on_surface_changed(self):
        self._make_fonts()
        self._build_setup_layout()
        if self.state != SETUP:
            self._layout_arena()

    def _dim_surface(self):
        """Abdunkelnde Vollbild-Fläche (gecacht, nur bei Größenwechsel neu)."""
        key = (self.width, self.height)
        if getattr(self, "_dim_key", None) != key:
            self._dim_key = key
            self._dim = pygame.Surface(key, pygame.SRCALPHA)
            self._dim.fill((8, 10, 16, 150))
        return self._dim

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(380, self.width - 60)
        y0 = int(self.height * 0.30)
        rows = []
        if not self.multiplayer:
            rows.append("diff")
        rows.append("arena")
        self.setup_rows = rows
        self.row_rects = [pygame.Rect(cx - bw // 2, y0 + i * 58, bw, 46)
                          for i in range(len(rows))]
        self.start_rect = pygame.Rect(cx - 95, y0 + len(rows) * 58 + 16,
                                      190, 46)
        self.sel_row = 0

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("tanks", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _adjust_row(self, row, direction):
        if self.setup_rows[row] == "diff":
            self.diff = (self.diff + direction) % 3
            self._save_setting("difficulty", self.diff)
        else:
            self.arena_sel = (self.arena_sel + 1 + direction) % 5 - 1
            self._save_setting("arena", self.arena_sel)
        self.play_sound("select")

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("Up", "w", "W"):
                self.sel_row = (self.sel_row - 1) % len(self.setup_rows)
                self.play_sound("move")
            elif k in ("Down", "s", "S"):
                self.sel_row = (self.sel_row + 1) % len(self.setup_rows)
                self.play_sound("move")
            elif k in ("Left", "a", "A"):
                self._adjust_row(self.sel_row, -1)
            elif k in ("Right", "d", "D"):
                self._adjust_row(self.sel_row, +1)
            elif k in ("Return", "space"):
                self._start_match()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, r in enumerate(self.row_rects):
                if r.collidepoint(event.pos):
                    self.sel_row = i
                    self._adjust_row(i, +1)
                    return
            if self.start_rect.collidepoint(event.pos):
                self._start_match()

    # ===================================================== Match / Runde
    def _start_match(self):
        self.rounds = [0, 0]
        self.game_over = False
        self._arena_cycle = [k for k in ("open", "cross", "pillars", "maze")]
        random.shuffle(self._arena_cycle)
        self._start_round()
        self.play_sound("click")

    def _layout_arena(self):
        self.hud_h = max(34, int(self.height * 0.075))
        self.arena = pygame.Rect(8, self.hud_h, self.width - 16,
                                 self.height - self.hud_h - 8)
        self.sc = min(self.width / 640.0, self.height / 480.0)
        self.walls = [self.arena.copy()]      # Index 0 = Außenrand (Sonderfall)
        self.inner_walls = []
        for fx, fy, fw, fh in WALL_PRESETS[self.arena_key]:
            r = pygame.Rect(self.arena.x + int(fx * self.arena.w),
                            self.arena.y + int(fy * self.arena.h),
                            max(14, int(fw * self.arena.w)),
                            max(14, int(fh * self.arena.h)))
            self.inner_walls.append(r)

    def _start_round(self):
        if self.arena_sel >= 0:
            self.arena_key = ARENAS[self.arena_sel + 1]
        else:
            self.arena_key = self._arena_cycle[
                (self.rounds[0] + self.rounds[1]) % 4]
        self._layout_arena()
        inset_x = int(self.arena.w * 0.15)
        inset_y = int(self.arena.h * 0.15)
        self.tanks = [
            dict(x=float(self.arena.x + inset_x),
                 y=float(self.arena.y + inset_y), ang=math.radians(45),
                 cooldown=0.0, shield=False, rapid=0.0, triple=0,
                 alive=True, keys=set()),
            dict(x=float(self.arena.right - inset_x),
                 y=float(self.arena.bottom - inset_y),
                 ang=math.radians(225), cooldown=0.0, shield=False,
                 rapid=0.0, triple=0, alive=True, keys=set()),
        ]
        self.bullets = []       # dict(x,y,vx,vy,owner,bounces,age)
        self.powerups = []      # dict(x,y,kind,age)
        self.power_timer = random.uniform(8, 14)
        self.count_t = 2.4
        self.banner_t = 0.0
        self.round_winner = None
        self._ai_timer = 0.0
        self._ai_move = (0, 0)  # (throttle, turn)
        self.state = COUNTDOWN

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.state == MATCH_END:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self.game_over = False
                    self._start_match()
                elif event.key in ("s", "S"):
                    self.game_over = False
                    self.state = SETUP
                    self.play_sound("click")
            elif event.kind == InputEvent.MOUSEDOWN:
                self.game_over = False
                self._start_match()
            return
        if event.kind == InputEvent.KEYDOWN:
            self._set_key(event.key, True)
        elif event.kind == InputEvent.KEYUP:
            self._set_key(event.key, False)

    def _set_key(self, key, down):
        """Gehaltene Aktionen je Spieler puffern; Feuern sofort auslösen.

        Im Einzelspieler steuern BEIDE Tastenbelegungen (P1 und P2) den
        eigenen Panzer - wie in der Basisklasse als Konvention beschrieben.
        """
        if self.multiplayer:
            mapping = (("p1", 0), ("p2", 1))
        else:
            mapping = (("p1", 0), ("p2", 0))
        for p, i in mapping:
            for act in ("up", "down", "left", "right"):
                if self.key_for(p, act) == key:
                    if down:
                        self.tanks[i]["keys"].add(act)
                    else:
                        self.tanks[i]["keys"].discard(act)
            if self.key_for(p, "action") == key and down \
                    and self.state == PLAY:
                self._fire(self.tanks[i], i)

    # ===================================================== Physik
    def _move_tank(self, tk, dt, throttle, turn):
        tk["ang"] += math.radians(180) * turn * dt
        speed = (140 if throttle > 0 else 100) * self.sc * throttle
        if speed:
            tk["x"] += math.cos(tk["ang"]) * speed * dt
            tk["y"] += math.sin(tk["ang"]) * speed * dt
        self._collide_walls(tk)

    def _tank_r(self):
        return 13 * self.sc

    def _collide_walls(self, tk):
        r = self._tank_r()
        tk["x"] = max(self.arena.x + r, min(self.arena.right - r, tk["x"]))
        tk["y"] = max(self.arena.y + r, min(self.arena.bottom - r, tk["y"]))
        for w in self.inner_walls:
            nearest_x = max(w.left, min(w.right, tk["x"]))
            nearest_y = max(w.top, min(w.bottom, tk["y"]))
            dx, dy = tk["x"] - nearest_x, tk["y"] - nearest_y
            d2 = dx * dx + dy * dy
            if d2 < r * r:
                d = math.sqrt(d2) or 0.001
                push = (r - d)
                tk["x"] += dx / d * push
                tk["y"] += dy / d * push

    def _fire(self, tk, owner):
        if tk["cooldown"] > 0 or not tk["alive"]:
            return
        live = sum(1 for b in self.bullets if b["owner"] == owner)
        if live >= MAX_BULLETS:
            return
        tk["cooldown"] = 0.2 if tk["rapid"] > 0 else COOLDOWN
        angles = [0.0]
        if tk["triple"] > 0:
            angles = [-math.radians(12), 0.0, math.radians(12)]
            tk["triple"] -= 1
        r = self._tank_r()
        for da in angles:
            a = tk["ang"] + da
            self.bullets.append(dict(
                x=tk["x"] + math.cos(a) * (r + 6),
                y=tk["y"] + math.sin(a) * (r + 6),
                vx=math.cos(a) * BULLET_SPEED * self.sc,
                vy=math.sin(a) * BULLET_SPEED * self.sc,
                owner=owner, bounces=0, age=0.0))
        self.play_sound("shoot")

    def _update_bullets(self, dt):
        alive = []
        for b in self.bullets:
            b["age"] += dt
            if b["age"] > BULLET_LIFE:
                continue
            b["x"] += b["vx"] * dt
            b["y"] += b["vy"] * dt
            # Außenrand
            bounced = False
            if b["x"] < self.arena.x or b["x"] > self.arena.right:
                b["vx"] = -b["vx"]
                b["x"] = max(self.arena.x,
                             min(self.arena.right, b["x"]))
                bounced = True
            if b["y"] < self.arena.y or b["y"] > self.arena.bottom:
                b["vy"] = -b["vy"]
                b["y"] = max(self.arena.y,
                             min(self.arena.bottom, b["y"]))
                bounced = True
            # Innenwände: Achse mit kleinerer Eindringtiefe spiegeln
            if not bounced:
                for w in self.inner_walls:
                    if w.collidepoint(b["x"], b["y"]):
                        pen_x = min(b["x"] - w.left, w.right - b["x"])
                        pen_y = min(b["y"] - w.top, w.bottom - b["y"])
                        if pen_x < pen_y:
                            b["vx"] = -b["vx"]
                            b["x"] += math.copysign(pen_x + 1, b["vx"])
                        else:
                            b["vy"] = -b["vy"]
                            b["y"] += math.copysign(pen_y + 1, b["vy"])
                        bounced = True
                        break
            if bounced:
                b["bounces"] += 1
                if b["bounces"] >= 2:
                    continue
                self.play_sound("bounce")
            # Treffer?
            hit = False
            for i, tk in enumerate(self.tanks):
                if not tk["alive"]:
                    continue
                if i == b["owner"] and b["age"] < GRACE:
                    continue
                if math.hypot(b["x"] - tk["x"],
                              b["y"] - tk["y"]) < self._tank_r() + 3.5:
                    hit = True
                    if tk["shield"]:
                        tk["shield"] = False
                        self.play_sound("bounce")
                    else:
                        tk["alive"] = False
                        self._round_over(1 - i)
                    break
            if not hit:
                alive.append(b)
        self.bullets = alive

    def _update_powerups(self, dt):
        for p in self.powerups:
            p["age"] += dt
        self.powerups = [p for p in self.powerups if p["age"] < 12.0]
        self.power_timer -= dt
        if self.power_timer <= 0 and len(self.powerups) < 2:
            self.power_timer = random.uniform(8, 14)
            for _ in range(30):
                x = random.uniform(self.arena.x + 60, self.arena.right - 60)
                y = random.uniform(self.arena.y + 60, self.arena.bottom - 60)
                if any(w.inflate(120, 120).collidepoint(x, y)
                       for w in self.inner_walls):
                    continue
                if any(math.hypot(x - tk["x"], y - tk["y"]) < 60
                       for tk in self.tanks):
                    continue
                self.powerups.append(dict(
                    x=x, y=y, age=0.0,
                    kind=random.choice(("rapid", "shield", "triple"))))
                break
        # Aufnehmen
        for p in self.powerups[:]:
            for tk in self.tanks:
                if not tk["alive"]:
                    continue
                if math.hypot(p["x"] - tk["x"],
                              p["y"] - tk["y"]) < self._tank_r() + 12:
                    if p["kind"] == "rapid":
                        tk["rapid"] = 8.0
                    elif p["kind"] == "shield":
                        tk["shield"] = True
                    else:
                        tk["triple"] = 5
                    self.powerups.remove(p)
                    self.play_sound("powerup")
                    break

    # ===================================================== KI
    def _clear(self, a, b):
        """Sichtlinie: Segment a->b kreuzt keine Innenwand."""
        for w in self.inner_walls:
            if w.clipline(a, b):
                return False
        return True

    def _ai_update(self, dt):
        ai = self.tanks[1]
        pl = self.tanks[0]
        if not ai["alive"] or not pl["alive"]:
            return
        intervals = (0.6, 0.35, 0.2)
        errors = (14.0, 7.0, 3.0)
        self._ai_timer -= dt
        if self._ai_timer <= 0:
            self._ai_timer = intervals[self.diff]
            self._ai_think(ai, pl, errors[self.diff])
        throttle, turn = self._ai_move
        self._move_tank(ai, dt, throttle, turn)
        # Feuern, wenn grob ausgerichtet
        want = getattr(self, "_ai_aim", None)
        if want is not None:
            diff_a = (want - ai["ang"] + math.pi) % math.tau - math.pi
            if abs(diff_a) < math.radians(8):
                self._fire(ai, 1)

    def _ai_think(self, ai, pl, err_deg):
        a_pos = (ai["x"], ai["y"])
        p_pos = (pl["x"], pl["y"])
        dist = math.hypot(p_pos[0] - a_pos[0], p_pos[1] - a_pos[1])
        aim = None
        if self._clear(a_pos, p_pos):
            tx, ty = p_pos
            if self.diff == 2:      # Vorhalt auf hard
                # grobe Zielbewegung aus gehaltenen Tasten ableiten
                spd = 140 * self.sc if "up" in pl["keys"] else 0
                tx += math.cos(pl["ang"]) * spd * dist / (BULLET_SPEED * self.sc)
                ty += math.sin(pl["ang"]) * spd * dist / (BULLET_SPEED * self.sc)
            aim = math.atan2(ty - a_pos[1], tx - a_pos[0])
        elif self.diff == 2:
            aim = self._ricochet_aim(a_pos, p_pos)
        if aim is not None:
            aim += math.radians(random.uniform(-err_deg, err_deg))
        self._ai_aim = aim

        # Bewegung
        dodge = None
        if self.diff >= 1:
            for b in self.bullets:
                if b["owner"] == 1:
                    continue
                d = math.hypot(b["x"] - a_pos[0], b["y"] - a_pos[1])
                if d < 120 * self.sc:
                    dodge = math.atan2(a_pos[1] - b["y"],
                                       a_pos[0] - b["x"]) + math.pi / 2
                    break
        if dodge is not None:
            target_ang = dodge
            throttle = 1.0
        elif self.diff == 0:
            wp = getattr(self, "_ai_wp", None)
            if wp is None or math.hypot(wp[0] - a_pos[0],
                                        wp[1] - a_pos[1]) < 40:
                wp = (random.uniform(self.arena.x + 40, self.arena.right - 40),
                      random.uniform(self.arena.y + 40, self.arena.bottom - 40))
                self._ai_wp = wp
            target_ang = math.atan2(wp[1] - a_pos[1], wp[0] - a_pos[0])
            throttle = 1.0
        elif self.diff == 1:
            target_ang = math.atan2(p_pos[1] - a_pos[1], p_pos[0] - a_pos[0])
            throttle = 1.0 if dist > 350 * self.sc else \
                (-1.0 if dist < 200 * self.sc else 0.0)
        else:
            orbit = math.atan2(p_pos[1] - a_pos[1], p_pos[0] - a_pos[0]) \
                + math.pi / 2
            target_ang = orbit
            throttle = 1.0
        # Richtung Ziel ausrichten: wenn Zielwinkel (fürs Schießen) existiert,
        # hat der Vorrang beim Drehen.
        steer_to = self._ai_aim if self._ai_aim is not None else target_ang
        diff_a = (steer_to - ai["ang"] + math.pi) % math.tau - math.pi
        turn = max(-1.0, min(1.0, diff_a * 3.0))
        self._ai_move = (throttle, turn)

    def _ricochet_aim(self, a, p):
        """Ricochet über Spiegelung des Ziels an Wand-Ebenen (auch Rand)."""
        planes = []
        planes.append(("x", self.arena.x))
        planes.append(("x", self.arena.right))
        planes.append(("y", self.arena.y))
        planes.append(("y", self.arena.bottom))
        for w in self.inner_walls:
            planes += [("x", w.left), ("x", w.right),
                       ("y", w.top), ("y", w.bottom)]
        random.shuffle(planes)
        for axis, v in planes:
            if axis == "x":
                mirror = (2 * v - p[0], p[1])
            else:
                mirror = (p[0], 2 * v - p[1])
            # Reflexionspunkt auf der Ebene
            dx = mirror[0] - a[0]
            dy = mirror[1] - a[1]
            if axis == "x":
                if dx == 0:
                    continue
                tt = (v - a[0]) / dx
            else:
                if dy == 0:
                    continue
                tt = (v - a[1]) / dy
            if not (0.05 < tt < 0.95):
                continue
            hit = (a[0] + dx * tt, a[1] + dy * tt)
            if self._clear(a, hit) and self._clear(hit, p):
                return math.atan2(dy, dx)
        return None

    # ===================================================== Runden-Ende
    def _round_over(self, winner):
        self.rounds[winner] += 1
        self.round_winner = winner
        self.play_sound("explode")
        if self.rounds[winner] >= ROUNDS_TO_WIN:
            self._match_over(winner)
        else:
            self.banner_t = 1.8
            self.state = ROUND_END
            self.play_sound("point")

    def _match_over(self, winner):
        self.state = MATCH_END
        if not self.multiplayer and winner == 0:
            self.score += (self.diff + 1) * 100 \
                + (ROUNDS_TO_WIN - self.rounds[1]) * 20
            self.play_sound("win")
        elif self.multiplayer:
            self.play_sound("win")
        else:
            self.play_sound("gameover")
        self.game_over = True

    # ===================================================== Update
    def update(self, dt):
        if self.state == COUNTDOWN:
            self.count_t -= dt
            if self.count_t <= 0:
                self.state = PLAY
                self.play_sound("level")
            return
        if self.state == ROUND_END:
            self.banner_t -= dt
            if self.banner_t <= 0:
                self._start_round()
            return
        if self.state != PLAY:
            return

        for i, tk in enumerate(self.tanks):
            tk["cooldown"] = max(0.0, tk["cooldown"] - dt)
            tk["rapid"] = max(0.0, tk["rapid"] - dt)
        # Spieler-Steuerung
        players = 2 if self.multiplayer else 1
        for i in range(players):
            tk = self.tanks[i]
            throttle = (1.0 if "up" in tk["keys"] else 0.0) \
                - (1.0 if "down" in tk["keys"] else 0.0)
            turn = (1.0 if "right" in tk["keys"] else 0.0) \
                - (1.0 if "left" in tk["keys"] else 0.0)
            self._move_tank(tk, dt, throttle, turn)
        if not self.multiplayer:
            self._ai_update(dt)
        # Panzer-Panzer sanft auseinanderdrücken
        a, b = self.tanks
        d = math.hypot(a["x"] - b["x"], a["y"] - b["y"])
        min_d = 2 * self._tank_r()
        if 0 < d < min_d:
            push = (min_d - d) / 2
            nx, ny = (a["x"] - b["x"]) / d, (a["y"] - b["y"]) / d
            a["x"] += nx * push
            a["y"] += ny * push
            b["x"] -= nx * push
            b["y"] -= ny * push
        self._update_bullets(dt)
        self._update_powerups(dt)

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        if self.state == SETUP:
            self._draw_setup(s)
            return
        pygame.draw.rect(s, COL_FLOOR, self.arena)
        pygame.draw.rect(s, COL_WALL_EDGE, self.arena, 3)
        for w in self.inner_walls:
            pygame.draw.rect(s, COL_WALL, w)
            pygame.draw.rect(s, COL_WALL_EDGE, w, 2)
        for p in self.powerups:
            self._draw_powerup(s, p)
        for b in self.bullets:
            pygame.draw.circle(s, COL_BULLET,
                               (int(b["x"]), int(b["y"])), max(2, int(3.5 * self.sc)))
        for i, tk in enumerate(self.tanks):
            if tk["alive"]:
                self._draw_tank(s, tk, COL_P1 if i == 0 else COL_P2)
        self._draw_hud(s)
        if self.state == COUNTDOWN:
            n = max(1, int(math.ceil(self.count_t / 0.8)))
            img = self._huge.render(str(n), True, self.accent)
            s.blit(img, img.get_rect(center=(self.width // 2,
                                             self.height // 2)))
        elif self.state == ROUND_END:
            self._draw_banner(s, t("tank.round_win",
                                   n=self.round_winner + 1))
        elif self.state == MATCH_END:
            self._draw_match_end(s)

    def _draw_tank(self, s, tk, col):
        r = self._tank_r()
        x, y, a = tk["x"], tk["y"], tk["ang"]
        # Körper (gedrehtes Rechteck über Polygon)
        c, sn = math.cos(a), math.sin(a)
        pts = []
        for px, py in ((-r, -r * 0.75), (r, -r * 0.75),
                       (r, r * 0.75), (-r, r * 0.75)):
            pts.append((x + px * c - py * sn, y + px * sn + py * c))
        pygame.draw.polygon(s, col, pts)
        pygame.draw.polygon(s, tuple(int(v * 0.6) for v in col), pts, 2)
        # Ketten-Andeutung
        pygame.draw.circle(s, tuple(int(v * 0.7) for v in col),
                           (int(x), int(y)), int(r * 0.55))
        # Rohr
        pygame.draw.line(s, (230, 232, 240), (x, y),
                         (x + c * r * 1.5, y + sn * r * 1.5),
                         max(3, int(4 * self.sc)))
        # Schild-Ring
        if tk["shield"]:
            pygame.draw.circle(s, POWER_COLS["shield"], (int(x), int(y)),
                               int(r * 1.5), 2)
        if tk["rapid"] > 0:
            pygame.draw.circle(s, POWER_COLS["rapid"], (int(x), int(y)),
                               int(r * 1.3), 1)

    def _draw_powerup(self, s, p):
        col = POWER_COLS[p["kind"]]
        x, y = int(p["x"]), int(p["y"])
        blink = p["age"] > 9.0 and int(p["age"] * 6) % 2 == 0
        if blink:
            return
        pygame.draw.circle(s, col, (x, y), 11)
        pygame.draw.circle(s, (20, 24, 34), (x, y), 11, 2)
        sym = {"rapid": ">>", "shield": "O", "triple": "3x"}[p["kind"]]
        img = self._tiny.render(sym, True, (20, 24, 34))
        s.blit(img, img.get_rect(center=(x, y)))

    def _draw_hud(self, s):
        pygame.draw.rect(s, ui.PANEL, (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h),
                         (self.width, self.hud_h))
        cy = self.hud_h // 2
        pr = max(3, self.hud_h // 9)      # Runden-Punkte ("Pips") als Kreise
        gap = 2 * pr + 6
        for i, col in ((0, COL_P1), (1, COL_P2)):
            name = t("common.player1") if i == 0 else \
                (t("common.player2") if self.multiplayer else t("common.ai"))
            img = self._small.render(name, True, col)
            if i == 0:
                s.blit(img, img.get_rect(midleft=(12, cy)))
                x0 = 12 + img.get_width() + 12 + pr
                for k in range(ROUNDS_TO_WIN):
                    x = x0 + k * gap
                    if k < self.rounds[i]:
                        pygame.draw.circle(s, col, (x, cy), pr)
                    else:
                        pygame.draw.circle(s, ui.BORDER_LIGHT, (x, cy), pr, 1)
            else:
                s.blit(img, img.get_rect(midright=(self.width - 12, cy)))
                x0 = self.width - 12 - img.get_width() - 12 - pr
                for k in range(ROUNDS_TO_WIN):
                    x = x0 - k * gap
                    if k < self.rounds[i]:
                        pygame.draw.circle(s, col, (x, cy), pr)
                    else:
                        pygame.draw.circle(s, ui.BORDER_LIGHT, (x, cy), pr, 1)
        mid = self._small.render(t("tank.first_to", n=ROUNDS_TO_WIN), True,
                                 ui.TEXT_DIM)
        s.blit(mid, mid.get_rect(center=(self.width // 2, cy)))

    def _draw_banner(self, s, text):
        img = self._huge.render(text, True, self.accent)
        band_h = img.get_height() + 28
        y = self.height // 2 - band_h // 2
        ov = pygame.Surface((self.width, band_h), pygame.SRCALPHA)
        ov.fill(_rgba(ui.PANEL, 225))
        s.blit(ov, (0, y))
        pygame.draw.line(s, self.accent, (0, y), (self.width, y), 2)
        pygame.draw.line(s, self.accent, (0, y + band_h - 1),
                         (self.width, y + band_h - 1), 2)
        s.blit(img, img.get_rect(center=(self.width // 2, y + band_h // 2)))

    def _draw_match_end(self, s):
        s.blit(self._dim_surface(), (0, 0))
        cx, cy = self.width // 2, self.height // 2
        winner = 0 if self.rounds[0] >= ROUNDS_TO_WIN else 1
        if self.multiplayer:
            head = self._huge.render(t("common.player_wins", n=winner + 1),
                                     True, COL_P1 if winner == 0 else COL_P2)
        else:
            key = "tank.match_win" if winner == 0 else "common.game_over"
            head = self._huge.render(t(key), True,
                                     COL_P1 if winner == 0 else ui.RED)
        rows = [head,
                self.font.render(f"{self.rounds[0]} : {self.rounds[1]}",
                                 True, ui.TEXT)]
        if not self.multiplayer:
            rows.append(self._small.render(
                t("common.points", score=self.score), True, ui.TEXT_DIM))
        rows.append(self._small.render(t("tank.rematch"), True, ui.TEXT_DIM))
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
    def _draw_setup(self, s):
        cx = self.width // 2
        title = self._huge.render(self.name.upper(), True, self.accent)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.13))))
        sub = self._small.render(t("tank.subtitle"), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(self.height * 0.20))))
        for i, r in enumerate(self.row_rects):
            on = (i == self.sel_row)
            pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, r,
                             border_radius=10)
            pygame.draw.rect(s, self.accent if on else ui.BORDER, r,
                             2 if on else 1, border_radius=10)
            if self.setup_rows[i] == "diff":
                label = t("tank.difficulty")
                value = t("tank.diff." + DIFFS[self.diff])
            else:
                label = t("tank.arena")
                value = t("tank.arena." + ARENAS[self.arena_sel + 1])
            img = self._small.render(label, True,
                                     ui.TEXT if on else ui.TEXT_DIM)
            s.blit(img, img.get_rect(midleft=(r.x + 16, r.centery)))
            img = self._small.render("< " + value + " >", True, self.accent)
            s.blit(img, img.get_rect(midright=(r.right - 16, r.centery)))
        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=10)
        pygame.draw.rect(s, self.accent, self.start_rect, 2, border_radius=10)
        st = self.font.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        hint = self._tiny.render(t("tank.setup_hint"), True, ui.TEXT_FAINT)
        s.blit(hint, hint.get_rect(center=(cx, self.height - 30)))
        ctrl = self._tiny.render(t("tank.controls_hint"), True,
                                 ui.mix(self.accent, ui.TEXT, 0.45))
        s.blit(ctrl, ctrl.get_rect(center=(cx, self.height - 12)))
