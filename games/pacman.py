# -*- coding: utf-8 -*-
"""
pacman.py
=========
Pac-Man - ein möglichst originalgetreuer Klon (Einzelspieler).

Features
--------
- Klassisches 28x31-Labyrinth mit Pillen, 4 Power-Pillen, Tunnel-Warp und
  Geisterhaus in der Mitte.
- Vier Geister mit den ORIGINAL-Verhaltensweisen (Ziel-Kachel-KI):
    * Blinky (rot)   - jagt Pac-Man direkt.
    * Pinky (rosa)   - zielt 4 Kacheln vor Pac-Man (Hinterhalt).
    * Inky (cyan)    - nutzt einen Vektor von Blinky über Pac-Man hinaus.
    * Clyde (orange) - jagt aus der Ferne, weicht aus der Nähe in seine Ecke.
- Scatter/Chase-Phasen im Wechsel; bei jedem Moduswechsel drehen die Geister um.
- POWER-PILLE -> Geister werden blau (Frightened), sind essbar und drehen um;
  Kette 200/400/800/1600 Punkte, danach kehren die Augen ins Haus zurück.
- Geisterhaus mit gestaffelter Freigabe (Pinky/Inky/Clyde), Augen kehren heim.
- Tunnel an den Seiten (Warp), Geister sind im Tunnel langsamer.
- FRÜCHTE erscheinen zeitweise unter dem Haus und geben Bonuspunkte (je Level).
- 3 Leben, Extraleben bei 10.000 Punkten, Level-System (wird schneller,
  Frightened-Zeit sinkt), Death-Animation, READY-/GAME-OVER-Screens.
- Setup: Schwierigkeit (Normal/Schwer/Extrem) - Geistertempo & Frightened-Zeit.
- Optik: neon-blaues Labyrinth, animierter Pac-Mund, Geister mit Augen/Füßen,
  pulsierende Power-Pillen, Punkte-Popups.

Steuerung: Pfeile oder WASD.  Enter = nach Game Over neu,  S = Setup.
"""

import math
import random
import pygame

import highscore
import settings as settings_mod
from game_base import Game, InputEvent
from i18n import t

# ------------------------------------------------------------------ Labyrinth
# '#' Wand, '.' Pille, 'o' Power-Pille, ' ' frei, '=' Geisterhaus-Tür,
# 't' Tunnel (frei, kein Punkt).  28 Spalten x 31 Zeilen.
MAZE = [
    "############################",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#o####.#####.##.#####.####o#",
    "#.####.#####.##.#####.####.#",
    "#..........................#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#......##....##....##......#",
    "######.#####.##.#####.######",
    "######.#####.##.#####.######",
    "######.##..........##.######",
    "######.##.###==###.##.######",
    "######.##.#      #.##.######",
    "tttttt....#      #....tttttt",
    "######.##.#      #.##.######",
    "######.##.########.##.######",
    "######.##..........##.######",
    "######.##.########.##.######",
    "######.##.########.##.######",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#.####.#####.##.#####.####.#",
    "#o..##.......  .......##..o#",
    "###.##.##.########.##.##.###",
    "###.##.##.########.##.##.###",
    "#......##....##....##......#",
    "#.##########.##.##########.#",
    "#.##########.##.##########.#",
    "#..........................#",
    "############################",
]
COLS, ROWS = 28, 31

# Kacheln, die beim Aufbau von Pillen befreit werden (Geister-Startbereich).
_CLEAR_TILES = {(13, 11), (14, 11)}

# Startpositionen
PAC_START = (13, 23)
DOOR_EXIT = (13, 11)                 # Kachel direkt über der Tür (Aus-/Eingang)
GHOST_HOMES = {"blinky": (13, 11), "pinky": (13, 14),
               "inky": (11, 14), "clyde": (16, 14)}
GHOST_SCATTER = {"blinky": (25, 0), "pinky": (2, 0),
                 "inky": (27, 30), "clyde": (0, 30)}
GHOST_COLORS = {"blinky": (255, 70, 70), "pinky": (255, 165, 220),
                "inky": (70, 220, 240), "clyde": (255, 175, 70)}
GHOST_RELEASE = {"blinky": -1.0, "pinky": 2.0, "inky": 7.0, "clyde": 13.0}

# Richtungen + klassische Vorrang-Reihenfolge (hoch, links, runter, rechts)
UP, LEFT, DOWN, RIGHT = (0, -1), (-1, 0), (0, 1), (1, 0)
ORDER = (UP, LEFT, DOWN, RIGHT)

# Scatter/Chase-Zeitplan (Sekunden, Modus)
SCHEDULE = [(7, "scatter"), (20, "chase"), (7, "scatter"), (20, "chase"),
            (5, "scatter"), (20, "chase"), (5, "scatter"), (1e9, "chase")]

# Tempo in Kacheln/Sekunde
SPD_PAC = 8.4
SPD_GHOST = 7.4
SPD_FRIGHT = 4.8
SPD_EYES = 16.0
SPD_HOUSE = 3.6
SPD_TUNNEL = 4.4

FRUIT_TABLE = [
    ("KIRSCHE", 100, (235, 60, 60)),
    ("ERDBEERE", 300, (235, 80, 110)),
    ("ORANGE", 500, (245, 160, 50)),
    ("APFEL", 700, (220, 50, 50)),
    ("MELONE", 1000, (120, 220, 120)),
    ("GALAXIAN", 2000, (120, 180, 255)),
    ("GLOCKE", 3000, (245, 220, 80)),
    ("SCHLÜSSEL", 5000, (200, 210, 230)),
]

DIFFS = [
    dict(key="normal", gspeed=1.0, fright=6.0),
    dict(key="hard", gspeed=1.07, fright=4.5),
    dict(key="extreme", gspeed=1.13, fright=3.5),
]

# Farben
COL_BG = (0, 0, 0)
COL_WALL = (36, 46, 190)
COL_WALL_HI = (80, 110, 255)
COL_DOOR = (255, 180, 210)
COL_PILL = (250, 220, 170)
COL_POWER = (255, 200, 120)
COL_PAC = (255, 235, 50)
COL_TEXT = (240, 240, 245)
COL_DIM = (150, 158, 176)
COL_FRIGHT = (36, 40, 210)
COL_FRIGHT_END = (245, 245, 255)
COL_ACCENT = (255, 225, 90)
COL_BTN = (44, 50, 66)
COL_BTN_ON = (60, 90, 150)

SETUP, READY, PLAY, DYING, LEVELCLEAR, GAMEOVER = \
    "setup", "ready", "play", "dying", "levelclear", "gameover"


def _add(a, b):
    return (a[0] + b[0], a[1] + b[1])


def _opp(d):
    return (-d[0], -d[1])


def _dist2(a, b):
    return (a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2


class _Mover:
    """Gemeinsamer Zustand von Pac-Man und Geistern (Kachel + Bruchteil)."""

    def __init__(self, tile, direction):
        self.tile = tile
        self.dir = direction
        self.frac = 0.0            # Fortschritt 0..1 zur nächsten Kachel


class _Ghost(_Mover):
    def __init__(self, name):
        super().__init__(GHOST_HOMES[name], LEFT)
        self.name = name
        self.color = GHOST_COLORS[name]
        self.home = GHOST_HOMES[name]
        self.scatter = GHOST_SCATTER[name]
        self.release_at = GHOST_RELEASE[name]
        self.state = "maze" if name == "blinky" else "house"
        self.frightened = False
        self.bob_t = random.uniform(0, math.tau)


class PacmanGame(Game):
    name = "Pac-Man"
    highscore_key = "pacman"
    supports_multiplayer = False

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        pm = self.settings.get("pacman", {}) if isinstance(self.settings, dict) else {}
        self.diff = max(0, min(2, int(pm.get("difficulty", 0))))

        self._layout()
        self.highscore = highscore.load_highscores().get(self.highscore_key, 0)
        self.anim_t = 0.0
        self.popups = []
        self.fruit_history = []

        self._build_setup_layout()
        self._new_game()
        self.state = SETUP

    def _layout(self):
        """Zellgröße/Ursprung an die aktuelle Auflösung anpassen (zentriert)."""
        cell = min(self.width // COLS, (self.height) // (ROWS + 5))
        self.CELL = max(6, cell)
        self.maze_w = COLS * self.CELL
        self.maze_h = ROWS * self.CELL
        self.ox = (self.width - self.maze_w) // 2
        top = int(2.2 * self.CELL)
        self.oy = top + max(0, (self.height - top - self.maze_h - 2 * self.CELL) // 2)

        c = self.CELL
        self._font = pygame.font.SysFont("consolas", max(11, int(c * 1.15)), bold=True)
        self._small = pygame.font.SysFont("consolas", max(10, int(c * 0.95)))
        self._tiny = pygame.font.SysFont("consolas", max(9, int(c * 0.8)))
        self._big = pygame.font.SysFont("consolas", max(20, int(c * 2.0)), bold=True)

    # ----- Level / Positionen -------------------------------------------
    def _new_game(self):
        self.level = 1
        self.score = 0
        self.lives = 3
        self.extra_awarded = False
        self.game_over = False
        self.fruit_history = []
        self._build_level()
        self._reset_positions(2.2)

    def _build_level(self):
        """Baut Wände + Pillen für das aktuelle Level neu auf."""
        self.grid = [list(row) for row in MAZE]
        self.pellets = [[0] * COLS for _ in range(ROWS)]
        total = 0
        for r in range(ROWS):
            for c in range(COLS):
                ch = self.grid[r][c]
                if (c, r) in _CLEAR_TILES:
                    continue
                if ch == ".":
                    self.pellets[r][c] = 1
                    total += 1
                elif ch == "o":
                    self.pellets[r][c] = 2
                    total += 1
        self.dots_total = total
        self.dots_eaten = 0
        self.fruit = None
        self._fruit_thresholds = [70, 170]

    def _reset_positions(self, ready_time):
        self.pac = _Mover(PAC_START, LEFT)
        self.pac.want = LEFT
        self.pac.moving = False
        self.pac.mouth = 0.2
        self.ghosts = [_Ghost(n) for n in ("blinky", "pinky", "inky", "clyde")]
        self.play_timer = 0.0
        self.phase = 0
        self.mode_timer = SCHEDULE[0][0]
        self.fright_timer = 0.0
        self.ghost_combo = 0
        self.freeze = 0.0
        self.state = READY
        self.ready_timer = ready_time
        self.dying_timer = 0.0

    @property
    def global_mode(self):
        return SCHEDULE[self.phase][1]

    def _fright_time(self):
        return max(1.0, DIFFS[self.diff]["fright"] - (self.level - 1) * 0.4)

    def _level_factor(self):
        return min(1.28, 1.0 + (self.level - 1) * 0.035)

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(420, self.width - 60)
        self.diff_panel = pygame.Rect(cx - bw // 2, 150, bw, 60)
        self.diff_left = pygame.Rect(self.diff_panel.left, 150, 42, 60)
        self.diff_right = pygame.Rect(self.diff_panel.right - 42, 150, 42, 60)
        self.start_rect = pygame.Rect(cx - 95, 238, 190, 52)

    def _save(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("pacman", {})[key] = value
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
    def _dir_from_key(self, key):
        if self.is_action(key, "up") or key == "Up":
            return UP
        if self.is_action(key, "down") or key == "Down":
            return DOWN
        if self.is_action(key, "left") or key == "Left":
            return LEFT
        if self.is_action(key, "right") or key == "Right":
            return RIGHT
        return None

    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if event.kind != InputEvent.KEYDOWN:
            return
        if self.state == GAMEOVER:
            if event.key in ("Return", "space"):
                self._new_game()
            elif event.key in ("s", "S"):
                self.state = SETUP
                self.play_sound("click")
            return
        d = self._dir_from_key(event.key)
        if d and self.state in (READY, PLAY):
            self.pac.want = d
            if d == _opp(self.pac.dir):     # sofortiges Umkehren fühlt sich besser an
                self._reverse(self.pac)

    # ===================================================== Wände / Wege
    def _wrapt(self, tile):
        return (tile[0] % COLS, tile[1])

    def _pac_open(self, tile):
        c, r = tile
        if r < 0 or r >= ROWS:
            return False
        ch = self.grid[r][c % COLS]
        return ch not in ("#", "=")

    def _ghost_open(self, g, tile):
        c, r = tile
        if r < 0 or r >= ROWS:
            return False
        ch = self.grid[r][c % COLS]
        if ch == "#":
            return False
        if ch == "=":
            return g.state in ("leaving", "eyes", "entering")
        return True

    # ===================================================== Spiellogik
    def update(self, dt):
        self.anim_t += dt
        self._update_popups(dt)
        if self.state == SETUP or self.state == GAMEOVER:
            return
        if self.state == READY:
            self.ready_timer -= dt
            self._anim_pac(dt, force_move=False)
            if self.ready_timer <= 0:
                self.state = PLAY
            return
        if self.state == LEVELCLEAR:
            self.levelclear_timer -= dt
            if self.levelclear_timer <= 0:
                self.level += 1
                self._build_level()
                self._reset_positions(1.6)
            return
        if self.state == DYING:
            self.dying_timer -= dt
            if self.dying_timer <= 0:
                self._after_death()
            return
        if self.freeze > 0:
            self.freeze -= dt
            return

        self.play_timer += dt
        self._update_modes(dt)
        self._release_ghosts()

        self._move_pac(dt)
        self._anim_pac(dt, force_move=self.pac.moving)
        for g in self.ghosts:
            if g.state == "house":
                g.bob_t += dt * 3.0
            else:
                self._move_ghost(g, dt)

        self._collisions()
        self._update_fruit(dt)
        self._check_extra_life()
        if self.dots_eaten >= self.dots_total:
            self.state = LEVELCLEAR
            self.levelclear_timer = 1.8
            self.play_sound("win")

    def _update_modes(self, dt):
        if self.fright_timer > 0:
            self.fright_timer -= dt
            if self.fright_timer <= 0:
                for g in self.ghosts:
                    if g.state == "maze":
                        g.frightened = False
            return
        self.mode_timer -= dt
        if self.mode_timer <= 0 and self.phase < len(SCHEDULE) - 1:
            self.phase += 1
            self.mode_timer = SCHEDULE[self.phase][0]
            for g in self.ghosts:      # Moduswechsel -> Geister drehen um
                if g.state == "maze" and not g.frightened:
                    self._reverse(g)

    def _release_ghosts(self):
        for g in self.ghosts:
            if g.state == "house" and self.play_timer >= g.release_at:
                g.state = "leaving"
                g.tile = g.home
                g.frac = 0.0
                g.dir = UP

    # ----- Bewegung ------------------------------------------------------
    def _reverse(self, e):
        """Kehrt die Bewegungsrichtung um (auch mitten in einer Kachel)."""
        if e.frac > 1e-6:
            e.tile = self._wrapt(_add(e.tile, e.dir))
            e.frac = 1.0 - e.frac
        e.dir = _opp(e.dir)

    def _pac_speed(self):
        return SPD_PAC * self._level_factor()

    def _move_pac(self, dt):
        p = self.pac
        dist = self._pac_speed() * dt
        while dist > 1e-9:
            if p.frac <= 1e-9:
                if p.want and self._pac_open(_add(p.tile, p.want)):
                    p.dir = p.want
                if not self._pac_open(_add(p.tile, p.dir)):
                    p.moving = False
                    break
                p.moving = True
            step = min(dist, 1.0 - p.frac)
            p.frac += step
            dist -= step
            if p.frac >= 1.0 - 1e-9:
                p.tile = self._wrapt(_add(p.tile, p.dir))
                p.frac = 0.0
                self._pac_eat(p.tile)

    def _pac_eat(self, tile):
        c, r = tile
        v = self.pellets[r][c]
        if v == 1:
            self.pellets[r][c] = 0
            self.score += 10
            self.dots_eaten += 1
            if self.dots_eaten % 2 == 0:
                self.play_sound("eat")
        elif v == 2:
            self.pellets[r][c] = 0
            self.score += 50
            self.dots_eaten += 1
            self._frighten()
            self.play_sound("powerup")
        if self.fruit and tile in (self.fruit["a"], self.fruit["b"]):
            self._eat_fruit()

    def _anim_pac(self, dt, force_move):
        if force_move:
            self.pac.mouth = abs(math.sin(self.anim_t * 11.0)) * 0.92
        else:
            self.pac.mouth = 0.18

    def _ghost_speed(self, g):
        if g.state in ("eyes", "entering"):
            return SPD_EYES
        if g.state in ("house", "leaving"):
            return SPD_HOUSE
        if g.frightened:
            return SPD_FRIGHT
        c, r = g.tile
        if self.grid[r][c % COLS] == "t":
            return SPD_TUNNEL * self._level_factor()
        return SPD_GHOST * self._level_factor() * DIFFS[self.diff]["gspeed"]

    def _move_ghost(self, g, dt):
        dist = self._ghost_speed(g) * dt
        while dist > 1e-9:
            if g.frac <= 1e-9:
                self._ghost_decide(g)
            step = min(dist, 1.0 - g.frac)
            g.frac += step
            dist -= step
            if g.frac >= 1.0 - 1e-9:
                g.tile = self._wrapt(_add(g.tile, g.dir))
                g.frac = 0.0
                self._ghost_arrive(g)

    def _ghost_decide(self, g):
        if g.frightened and g.state == "maze":
            opts = [d for d in ORDER if d != _opp(g.dir)
                    and self._ghost_open(g, _add(g.tile, d))]
            g.dir = random.choice(opts) if opts else _opp(g.dir)
            return
        target = self._ghost_target(g)
        best, bd = None, 1e18
        for d in ORDER:
            if d == _opp(g.dir):
                continue
            nt = _add(g.tile, d)
            if not self._ghost_open(g, nt):
                continue
            dd = _dist2(self._wrapt(nt), target)
            if dd < bd:
                bd, best = dd, d
        g.dir = best if best is not None else _opp(g.dir)

    def _ghost_target(self, g):
        if g.state in ("leaving", "eyes"):
            return DOOR_EXIT
        if g.state == "entering":
            return g.home
        if self.global_mode == "scatter":
            return g.scatter
        return self._chase_target(g)

    def _chase_target(self, g):
        pac = self.pac
        if g.name == "blinky":
            return pac.tile
        if g.name == "pinky":
            return _add(pac.tile, (pac.dir[0] * 4, pac.dir[1] * 4))
        if g.name == "inky":
            p2 = _add(pac.tile, (pac.dir[0] * 2, pac.dir[1] * 2))
            bl = self.ghosts[0].tile
            return (2 * p2[0] - bl[0], 2 * p2[1] - bl[1])
        # clyde: aus der Ferne jagen, aus der Nähe ausweichen
        if _dist2(g.tile, pac.tile) >= 64:
            return pac.tile
        return g.scatter

    def _ghost_arrive(self, g):
        if g.state == "leaving" and g.tile[1] == 11 and g.tile[0] in (13, 14):
            g.state = "maze"
            g.frightened = self.fright_timer > 0
        elif g.state == "eyes" and g.tile[1] == 11 and g.tile[0] in (13, 14):
            g.state = "entering"
        elif g.state == "entering" and g.tile == g.home:
            g.state = "house"
            g.dir = UP
            g.release_at = self.play_timer + 1.0

    def _frighten(self):
        self.fright_timer = self._fright_time()
        self.ghost_combo = 0
        for g in self.ghosts:
            if g.state == "maze":
                g.frightened = True
                self._reverse(g)

    # ----- Kollisionen / Tod --------------------------------------------
    def _pix(self, e, bob=0.0):
        c, r = e.tile
        x = self.ox + (c + 0.5 + e.dir[0] * e.frac) * self.CELL
        y = self.oy + (r + 0.5 + e.dir[1] * e.frac + bob) * self.CELL
        return x, y

    def _collisions(self):
        px, py = self._pix(self.pac)
        for g in self.ghosts:
            if g.state != "maze":
                continue
            gx, gy = self._pix(g)
            if math.hypot(px - gx, py - gy) < 0.55 * self.CELL:
                if g.frightened:
                    self._eat_ghost(g, gx, gy)
                else:
                    self._pac_die()
                    return

    def _eat_ghost(self, g, gx, gy):
        pts = 200 * (2 ** self.ghost_combo)
        self.ghost_combo = min(self.ghost_combo + 1, 3)
        self.score += pts
        g.state = "eyes"
        g.frightened = False
        self._popup(gx, gy, str(pts), (120, 230, 255))
        self.freeze = 0.5
        self.play_sound("point")
        self.rumble(80)

    def _pac_die(self):
        self.state = DYING
        self.dying_timer = 1.7
        self.play_sound("explode")
        self.rumble(300)

    def _after_death(self):
        self.lives -= 1
        if self.lives <= 0:
            self.state = GAMEOVER
            self.game_over = True
            self.highscore = max(self.highscore, self.score)
            self.play_sound("gameover")
        else:
            self._reset_positions(1.8)

    def _check_extra_life(self):
        if not self.extra_awarded and self.score >= 10000:
            self.extra_awarded = True
            self.lives += 1
            self.play_sound("level")

    # ----- Früchte -------------------------------------------------------
    def _update_fruit(self, dt):
        if self.fruit is not None:
            self.fruit["t"] -= dt
            if self.fruit["t"] <= 0:
                self.fruit = None
        elif self._fruit_thresholds and self.dots_eaten >= self._fruit_thresholds[0]:
            self._fruit_thresholds.pop(0)
            name, pts, col = FRUIT_TABLE[min(self.level - 1, len(FRUIT_TABLE) - 1)]
            self.fruit = dict(name=name, pts=pts, col=col, t=9.5,
                              a=(13, 17), b=(14, 17))

    def _eat_fruit(self):
        f = self.fruit
        self.score += f["pts"]
        gx = self.ox + 13.5 * self.CELL
        gy = self.oy + 17.5 * self.CELL
        self._popup(gx, gy, str(f["pts"]), f["col"])
        self.fruit_history.append(f["col"])
        self.fruit = None
        self.play_sound("point")

    # ----- Popups --------------------------------------------------------
    def _popup(self, x, y, text, col):
        self.popups.append(dict(x=x, y=y, text=text, col=col, t=1.4, t0=1.4))

    def _update_popups(self, dt):
        for p in self.popups:
            p["t"] -= dt
            p["y"] -= dt * 12
        self.popups = [p for p in self.popups if p["t"] > 0]

    # ===================================================== Zeichnen
    def draw(self):
        if self.state == SETUP:
            self._draw_setup()
            return
        s = self.surface
        s.fill(COL_BG)
        flash = self.state == LEVELCLEAR and int(self.anim_t * 6) % 2 == 0
        self._draw_maze(s, flash)
        if not flash:
            self._draw_pellets(s)
        self._draw_fruit(s)
        if self.state != DYING:
            for g in self.ghosts:
                self._draw_ghost(s, g)
        self._draw_pac(s)
        for p in self.popups:
            a = max(0, min(255, int(255 * p["t"] / p["t0"])))
            img = self._tiny.render(p["text"], True, p["col"])
            img.set_alpha(a)
            s.blit(img, img.get_rect(center=(int(p["x"]), int(p["y"]))))
        self._draw_hud(s)
        self._draw_overlays(s)

    # ----- Labyrinth -----------------------------------------------------
    def _draw_maze(self, s, flash):
        c = self.CELL
        wall = COL_WALL_HI if flash else COL_WALL
        rad = max(2, int(c * 0.42))
        for r in range(ROWS):
            row = self.grid[r]
            for col in range(COLS):
                ch = row[col]
                if ch == "#":
                    rx, ry = self.ox + col * c, self.oy + r * c
                    pygame.draw.rect(s, wall, (rx, ry, c, c), border_radius=rad)
                elif ch == "=":
                    rx, ry = self.ox + col * c, self.oy + r * c
                    pygame.draw.rect(s, COL_DOOR, (rx, ry + c // 2 - 2, c, 4))

    def _draw_pellets(self, s):
        c = self.CELL
        pr = max(1, int(c * 0.12))
        power_on = int(self.anim_t * 6) % 2 == 0
        for r in range(ROWS):
            for col in range(COLS):
                v = self.pellets[r][col]
                if v == 1:
                    cx = self.ox + col * c + c // 2
                    cy = self.oy + r * c + c // 2
                    pygame.draw.circle(s, COL_PILL, (cx, cy), pr)
                elif v == 2 and power_on:
                    cx = self.ox + col * c + c // 2
                    cy = self.oy + r * c + c // 2
                    pygame.draw.circle(s, COL_POWER, (cx, cy), max(3, int(c * 0.34)))

    def _draw_fruit(self, s):
        if self.fruit is None:
            return
        cx = self.ox + 13.5 * self.CELL
        cy = self.oy + 17.5 * self.CELL
        self._fruit_icon(s, cx, cy, self.CELL * 0.7, self.fruit["col"])

    def _fruit_icon(self, s, cx, cy, r, col):
        pygame.draw.circle(s, col, (int(cx), int(cy + r * 0.15)), int(r * 0.6))
        pygame.draw.line(s, (90, 200, 90), (int(cx), int(cy - r * 0.4)),
                         (int(cx + r * 0.35), int(cy - r * 0.7)), 2)
        pygame.draw.circle(s, (255, 255, 255),
                           (int(cx - r * 0.2), int(cy)), max(1, int(r * 0.12)))

    # ----- Pac-Man -------------------------------------------------------
    def _draw_pac(self, s):
        r = self.CELL * 0.46
        px, py = self._pix(self.pac)
        if self.state == DYING:
            prog = 1.0 - self.dying_timer / 1.7
            mouth = 0.15 + prog * (math.pi - 0.15)
            if prog >= 0.98:
                return
        else:
            mouth = self.pac.mouth
        ang = math.atan2(self.pac.dir[1], self.pac.dir[0])
        pygame.draw.circle(s, COL_PAC, (int(px), int(py)), int(r))
        if mouth > 0.03:
            pts = [(px, py)]
            steps = 12
            for i in range(steps + 1):
                a = ang - mouth + (2 * mouth) * i / steps
                pts.append((px + math.cos(a) * r * 1.25,
                            py + math.sin(a) * r * 1.25))
            pygame.draw.polygon(s, COL_BG, pts)

    # ----- Geister -------------------------------------------------------
    def _draw_ghost(self, s, g):
        r = self.CELL * 0.46
        bob = 0.0
        if g.state == "house":
            bob = 0.12 * math.sin(g.bob_t)
        px, py = self._pix(g, bob)
        eyes_only = g.state in ("eyes", "entering")

        if not eyes_only:
            col = g.color
            if g.frightened:
                ending = self.fright_timer < min(2.2, self._fright_time() * 0.4)
                if ending and int(self.anim_t * 8) % 2 == 0:
                    col = COL_FRIGHT_END
                else:
                    col = COL_FRIGHT
            self._ghost_body(s, px, py, r, col)

        if g.frightened and not eyes_only:
            self._fright_face(s, px, py, r)
        else:
            self._ghost_eyes(s, px, py, r, g.dir)

    def _ghost_body(self, s, cx, cy, r, col):
        pygame.draw.circle(s, col, (int(cx), int(cy - r * 0.12)), int(r))
        pygame.draw.rect(s, col, (int(cx - r), int(cy - r * 0.12),
                                  int(2 * r), int(r * 1.05)))
        foot_y = cy - r * 0.12 + r * 1.05
        n = 4
        fw = 2 * r / n
        for i in range(n):
            x0 = cx - r + i * fw
            pygame.draw.polygon(s, COL_BG, [(x0, foot_y),
                                            (x0 + fw / 2, foot_y - r * 0.42),
                                            (x0 + fw, foot_y)])

    def _ghost_eyes(self, s, cx, cy, r, d):
        for sx in (-1, 1):
            ex = cx + sx * r * 0.42
            ey = cy - r * 0.18
            pygame.draw.circle(s, (255, 255, 255), (int(ex), int(ey)), int(r * 0.34))
            pupil = (ex + d[0] * r * 0.16, ey + d[1] * r * 0.16)
            pygame.draw.circle(s, (40, 50, 190),
                               (int(pupil[0]), int(pupil[1])), int(r * 0.17))

    def _fright_face(self, s, cx, cy, r):
        for sx in (-1, 1):
            pygame.draw.circle(s, (255, 220, 230),
                               (int(cx + sx * r * 0.38), int(cy - r * 0.12)),
                               max(1, int(r * 0.14)))
        y = cy + r * 0.4
        pts = []
        for i in range(7):
            x = cx - r * 0.6 + (r * 1.2) * i / 6
            pts.append((x, y + (r * 0.14 if i % 2 else -r * 0.14)))
        pygame.draw.lines(s, (255, 220, 230), False, pts, 2)

    # ----- HUD / Overlays -----------------------------------------------
    def _draw_hud(self, s):
        self.highscore = max(self.highscore, self.score)
        score = self._font.render(str(self.score), True, COL_TEXT)
        lab = self._tiny.render(t("pac.1up"), True, COL_ACCENT)
        s.blit(lab, (10, 4))
        s.blit(score, (10, 4 + lab.get_height()))
        hlab = self._tiny.render(t("pac.high"), True, COL_ACCENT)
        himg = self._font.render(str(self.highscore), True, COL_TEXT)
        s.blit(hlab, hlab.get_rect(midtop=(self.width // 2, 4)))
        s.blit(himg, himg.get_rect(midtop=(self.width // 2, 4 + hlab.get_height())))
        lv = self._small.render(t("pac.level", n=self.level), True, COL_DIM)
        s.blit(lv, lv.get_rect(topright=(self.width - 10, 6)))

        # Leben (unten links) + gesammelte Früchte (unten rechts)
        y = self.height - int(self.CELL * 1.2)
        for i in range(max(0, self.lives - 1)):
            self._life_icon(s, 16 + i * int(self.CELL * 1.4), y, self.CELL * 0.5)
        for i, col in enumerate(self.fruit_history[-6:]):
            self._fruit_icon(s, self.width - 16 - i * int(self.CELL * 1.3), y,
                             self.CELL * 0.6, col)

    def _life_icon(self, s, x, y, r):
        pygame.draw.circle(s, COL_PAC, (int(x), int(y)), int(r))
        pts = [(x, y)]
        for i in range(7):
            a = math.pi - 0.5 + 1.0 * i / 6
            pts.append((x + math.cos(a) * r * 1.3, y + math.sin(a) * r * 1.3))
        pygame.draw.polygon(s, COL_BG, pts)

    def _draw_overlays(self, s):
        cx = self.ox + self.maze_w // 2
        cy = self.oy + int(17.2 * self.CELL)
        if self.state == READY:
            img = self._font.render(t("pac.ready"), True, COL_ACCENT)
            s.blit(img, img.get_rect(center=(cx, cy)))
        elif self.state == GAMEOVER:
            img = self._big.render(t("common.game_over"), True, (255, 90, 90))
            s.blit(img, img.get_rect(center=(cx, cy)))
            hint = self._small.render(t("pac.restart_hint"), True, COL_TEXT)
            s.blit(hint, hint.get_rect(center=(cx, cy + int(self.CELL * 2.2))))

    # ----- Setup zeichnen -----------------------------------------------
    def _draw_setup(self):
        s = self.surface
        s.fill(COL_BG)
        title = self._big.render("PAC-MAN", True, COL_ACCENT)
        s.blit(title, title.get_rect(center=(self.width // 2, 70)))
        sub = self._small.render(t("snake.singleplayer"), True, COL_DIM)
        s.blit(sub, sub.get_rect(center=(self.width // 2, 110)))

        d = DIFFS[self.diff]
        pygame.draw.rect(s, (30, 36, 58), self.diff_panel, border_radius=10)
        pygame.draw.rect(s, COL_BTN_ON, self.diff_panel, 2, border_radius=10)
        name = self._font.render(
            t("pac.difficulty") + ":  " + t("pac.diff." + d["key"]), True, COL_TEXT)
        s.blit(name, name.get_rect(center=(self.diff_panel.centerx,
                                           self.diff_panel.top + 22)))
        note = self._tiny.render(t("pac.diff_note"), True, COL_DIM)
        s.blit(note, note.get_rect(center=(self.diff_panel.centerx,
                                           self.diff_panel.top + 44)))
        for rect, sym in ((self.diff_left, "<"), (self.diff_right, ">")):
            arr = self._big.render(sym, True, COL_ACCENT)
            s.blit(arr, arr.get_rect(center=rect.center))

        pygame.draw.rect(s, COL_BTN_ON, self.start_rect, border_radius=10)
        st = self._font.render(t("common.start"), True, COL_TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))

        hint = self._small.render(t("pac.setup_hint"), True, COL_DIM)
        s.blit(hint, hint.get_rect(center=(self.width // 2, self.height - 34)))
        ctrl = self._tiny.render(t("pac.controls_hint"), True, (120, 200, 150))
        s.blit(ctrl, ctrl.get_rect(center=(self.width // 2, self.height - 14)))
