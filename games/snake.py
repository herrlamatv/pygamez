# -*- coding: utf-8 -*-
"""
snake.py
========
Snake - komplett ueberarbeitete Deluxe-Version mit Spielmodi und Boost.

Neu
---
- BOOST: Solange die Boost-Taste (Einzelspieler: Leertaste/Shift) gedrueckt
  gehalten wird, laeuft der Turbo. Die Schlange bewegt sich doppelt so schnell
  und verbraucht dabei Ausdauer (Anzeige als Balken). Ist die Ausdauer leer,
  schaltet der Boost automatisch ab; sie laedt sich mit der Zeit wieder auf.
  Goldaepfel fuellen sie sofort ganz auf.
- SPIELMODI (im Setup waehlbar):
    * Klassisch    - der Klassiker.
    * Speed-Rush   - wird mit jedem Apfel schneller.
    * Hindernisse  - feste Bloecke im Spielfeld, die toedlich sind.
    * Portale      - Teleporter-Paare: rein ins eine, raus aus dem anderen.
    * Zeitangriff  - 60 Sekunden, so viele Aepfel wie moeglich.
- Goldaepfel: erscheinen zeitweise, bringen viele Punkte und fuellen den Boost.
- Weiterhin: Waende-durchgehen, Bonus-Aepfel, Mehrspieler (2 Schlangen),
  Prestige (Einzelspieler) - siehe prestige.py.
- Neue Optik: abgerundete Schlange mit Augen, Boost-Glow, Partikel,
  ueberarbeiteter Setup-Screen und HUD.

Steuerung
---------
- Bewegung: die in den Optionen belegten Tasten (Standard P1 = WASD, P2 = Pfeile).
- Boost:  P1 = Leertaste / Shift-links,  P2 = Enter / Shift-rechts.
- Prestige (Einzelspieler): P.   Pause-los; Enter/Leertaste startet nach Game Over neu.
"""

import math
import random
import pygame

import highscore
import prestige
import settings as settings_mod
from game_base import Game, InputEvent

CELL = 20                       # Kantenlaenge einer Rasterzelle in Pixeln
BASE_INTERVAL = 0.12            # Sekunden pro Schritt (Normaltempo)
MIN_INTERVAL = 0.055           # schnellstes Tempo (Speed-Rush)
MIN_LENGTH = 3                  # so kurz darf eine Schlange durch Prestige max. werden

# Boost / Ausdauer
STAMINA_MAX = 1.0
STAMINA_REGEN = 0.22            # Aufladung pro Sekunde (wenn nicht geboostet)
BOOST_DRAIN = 0.045            # Verbrauch pro zusaetzlichem Boost-Schritt
BOOST_MIN_START = 0.15         # so viel Ausdauer braucht man mindestens zum Starten

# Boost-Tasten (keine Richtungstasten, damit es nicht kollidiert)
BOOST_KEYS_P1 = ("space", "Shift_L")
BOOST_KEYS_P2 = ("Return", "Shift_R", "KP_Enter")

GOLDEN_LIFETIME = 6.0          # Sekunden, die ein Goldapfel liegen bleibt
GOLDEN_CHANCE = 0.20           # Chance, nach einem normalen Apfel einen Goldapfel zu setzen
TIMED_SECONDS = 60.0

# Spielzustaende
SETUP, PLAY = "setup", "play"

COL_BG = (15, 15, 25)
COL_GRID = (25, 25, 40)
COL_FOOD = (240, 90, 90)
COL_GOLD = (255, 205, 70)
COL_TEXT = (230, 230, 230)
COL_DIM = (150, 158, 176)
COL_BTN = (44, 50, 66)
COL_BTN_ON = (60, 120, 80)
COL_WLS_ON = (90, 230, 130)     # WLS gruen = durch die Waende gehen ist AN
COL_WLS_OFF = (105, 105, 120)   # WLS grau  = feste Waende
COL_MULT = (255, 210, 90)       # Multiplikator / Prestige (gold)
COL_WALL = (70, 78, 98)         # Hindernis-Bloecke
COL_ACCENT = (90, 160, 240)

# Farben je Schlange: (Koerper, Kopf)
SNAKE_COLORS = [
    ((80, 220, 120), (150, 255, 180)),   # Spieler 1 (gruen)
    ((90, 160, 240), (160, 205, 255)),   # Spieler 2 (blau)
]
# Boost-Glow-Farbe je Schlange
BOOST_GLOW = [(150, 255, 190), (170, 210, 255)]

# Portalfarben (Paare)
PORTAL_COLORS = [(255, 140, 60), (180, 120, 255), (90, 220, 220)]

# Spielmodi
MODES = [
    dict(key="classic", name="Klassisch",   desc="Der Snake-Klassiker."),
    dict(key="speed",   name="Speed-Rush",  desc="Wird mit jedem Apfel schneller."),
    dict(key="walls",   name="Hindernisse", desc="Feste Bloecke im Feld - toedlich!"),
    dict(key="portal",  name="Portale",     desc="Teleporter: rein und woanders raus."),
    dict(key="timed",   name="Zeitangriff", desc="60s - so viele Aepfel wie moeglich."),
]
MODE_KEYS = [m["key"] for m in MODES]


class _Snake:
    """Zustand einer einzelnen Schlange (Koerper: Kopf am Listenende)."""

    def __init__(self, body, direction, player):
        self.body = list(body)
        self.direction = direction
        self.next_direction = direction
        self.player = player            # "p1" / "p2"
        self.alive = True
        self.score = 0
        self.apples = 0                 # eigene Aepfel (Mehrspieler / Zeitangriff)
        self.grow = 0                   # ausstehende Wachstums-Bloecke
        self.stamina = STAMINA_MAX      # Boost-Ausdauer (0..1)
        self.boost_on = False           # Boost gerade aktiv?


class SnakeGame(Game):
    name = "Snake"
    highscore_key = "snake"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        self.winner = None
        self._effects_done = False

        self.cols = self.width // CELL
        self.rows = self.height // CELL

        snk = self.settings.get("snake", {}) if isinstance(self.settings, dict) else {}
        self.wrap = bool(snk.get("wrap", False))
        self.bonus = bool(snk.get("bonus_apple", False))
        mode_key = snk.get("mode", "classic")
        self.mode_index = MODE_KEYS.index(mode_key) if mode_key in MODE_KEYS else 0

        self._small = pygame.font.SysFont("consolas", 16)
        self._tiny = pygame.font.SysFont("consolas", 13)
        self.highscore = highscore.load_highscores().get(self.highscore_key, 0)

        self.particles = []
        self.anim_t = 0.0

        self._build_setup_layout()
        self._reset_run_stats()
        self._new_board()
        self.state = SETUP

    @property
    def mode_key(self):
        return MODES[self.mode_index]["key"]

    def _reset_run_stats(self):
        self.apples_total = 0
        self.apples_bank = 0
        self.prestige = 0
        self.speed_apples = 0

    def _new_board(self):
        """Baut die Schlange(n), das Modus-Layout und das erste Futter."""
        cy = self.rows // 2
        self.snakes = []
        if self.multiplayer:
            self.snakes.append(_Snake(
                [(2, cy - 3), (3, cy - 3), (4, cy - 3)], (1, 0), "p1"))
            self.snakes.append(_Snake(
                [(2, cy + 3), (3, cy + 3), (4, cy + 3)], (1, 0), "p2"))
        else:
            cx = self.cols // 2
            self.snakes.append(_Snake(
                [(cx - 2, cy), (cx - 1, cy), (cx, cy)], (1, 0), "p1"))

        self._timer = 0.0
        self.interval = BASE_INTERVAL
        self._build_mode_layout()
        self._place_food()

    def _start_play(self):
        self.score = 0
        self.game_over = False
        self.winner = None
        self._effects_done = False
        self.particles = []
        self._reset_run_stats()
        self._new_board()
        self.state = PLAY

    # ----- Modus-Layout (Hindernisse / Portale / Zeit) ------------------
    def _build_mode_layout(self):
        self.obstacles = set()
        self.portals = {}
        self.portal_pairs = []
        self.golden = None
        self.golden_timer = 0.0
        self.time_left = TIMED_SECONDS

        # Zellen, die frei bleiben muessen (Schlangen + Startbahn nach rechts)
        tabu = set()
        for sn in self.snakes:
            for (x, y) in sn.body:
                for ddx in range(-1, 8):        # Bahn nach rechts frei halten
                    tabu.add((x + ddx, y))
                tabu.add((x, y - 1))
                tabu.add((x, y + 1))

        if self.mode_key == "walls":
            self._make_obstacles(tabu)
        elif self.mode_key == "portal":
            self._make_portals(tabu)

    def _make_obstacles(self, tabu):
        anzahl = int(self.cols * self.rows * 0.05)
        versuche = 0
        while len(self.obstacles) < anzahl and versuche < anzahl * 30:
            versuche += 1
            # kleine Cluster (1-3 Bloecke) fuer interessantere Formen
            bx = random.randint(1, self.cols - 2)
            by = random.randint(2, self.rows - 2)
            cluster = [(bx, by)]
            if random.random() < 0.5:
                cluster.append((bx + random.choice((-1, 1)), by))
            if random.random() < 0.4:
                cluster.append((bx, by + random.choice((-1, 1))))
            if all(c not in tabu and 0 < c[0] < self.cols - 1
                   and 1 < c[1] < self.rows - 1 for c in cluster):
                self.obstacles.update(cluster)

    def _make_portals(self, tabu):
        paare = 2 if self.cols * self.rows > 500 else 1
        for k in range(paare):
            enden = []
            versuche = 0
            while len(enden) < 2 and versuche < 300:
                versuche += 1
                p = (random.randint(1, self.cols - 2), random.randint(2, self.rows - 2))
                if p in tabu or p in self.portals or p in [e for e in enden]:
                    continue
                if enden and abs(enden[0][0] - p[0]) + abs(enden[0][1] - p[1]) < 6:
                    continue
                enden.append(p)
                tabu.add(p)
            if len(enden) == 2:
                a, b = enden
                self.portals[a] = b
                self.portals[b] = a
                self.portal_pairs.append((a, b, PORTAL_COLORS[k % len(PORTAL_COLORS)]))

    def _blocked_cells(self):
        belegt = set(self.obstacles) | set(self.portals.keys())
        for sn in self.snakes:
            belegt |= set(sn.body)
        if self.golden:
            belegt.add(self.golden)
        return belegt

    def _place_food(self):
        belegt = self._blocked_cells()
        frei = [(x, y) for x in range(self.cols) for y in range(self.rows)
                if (x, y) not in belegt]
        self.food = random.choice(frei) if frei else None

    def _place_golden(self):
        belegt = self._blocked_cells()
        if self.food:
            belegt.add(self.food)
        frei = [(x, y) for x in range(self.cols) for y in range(self.rows)
                if (x, y) not in belegt]
        if frei:
            self.golden = random.choice(frei)
            self.golden_timer = GOLDEN_LIFETIME

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(420, self.width - 60)

        # Modus-Auswahl: Pfeile + Panel
        self.mode_panel = pygame.Rect(cx - bw // 2, 118, bw, 62)
        self.mode_left = pygame.Rect(self.mode_panel.left, 118, 40, 62)
        self.mode_right = pygame.Rect(self.mode_panel.right - 40, 118, 40, 62)

        bh, gap = 44, 14
        y0 = 196
        self.wrap_rect = pygame.Rect(cx - bw // 2, y0, bw, bh)
        self.bonus_rect = pygame.Rect(cx - bw // 2, y0 + (bh + gap), bw, bh)
        self.start_rect = pygame.Rect(cx - 95, y0 + 2 * (bh + gap) + 8, 190, 50)

    def _save_snake_setting(self, key, value):
        if isinstance(self.settings, dict):
            snk = self.settings.setdefault("snake", {})
            snk[key] = value
            settings_mod.save_settings(self.settings)

    def _toggle_setting(self, key):
        neu = not getattr(self, "wrap" if key == "wrap" else "bonus")
        if key == "wrap":
            self.wrap = neu
        else:
            self.bonus = neu
        self._save_snake_setting("wrap" if key == "wrap" else "bonus_apple", neu)
        self.play_sound("select")

    def _cycle_mode(self, step):
        self.mode_index = (self.mode_index + step) % len(MODES)
        self._save_snake_setting("mode", self.mode_key)
        self.play_sound("click")

    def _handle_setup_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Left", "a", "A"):
                self._cycle_mode(-1)
            elif event.key in ("Right", "d", "D"):
                self._cycle_mode(+1)
            elif event.key in ("1", "2", "3", "4", "5"):
                idx = int(event.key) - 1
                if idx < len(MODES):
                    self.mode_index = idx
                    self._save_snake_setting("mode", self.mode_key)
                    self.play_sound("click")
            elif event.key in ("w", "W"):
                self._toggle_setting("wrap")
            elif event.key in ("b", "B"):
                self._toggle_setting("bonus")
            elif event.key in ("Return", "space"):
                self.play_sound("click")
                self._start_play()
        elif event.kind == InputEvent.MOUSEDOWN:
            p = event.pos
            if self.mode_left.collidepoint(p):
                self._cycle_mode(-1)
            elif self.mode_right.collidepoint(p):
                self._cycle_mode(+1)
            elif self.mode_panel.collidepoint(p):
                self._cycle_mode(+1)
            elif self.wrap_rect.collidepoint(p):
                self._toggle_setting("wrap")
            elif self.bonus_rect.collidepoint(p):
                self._toggle_setting("bonus")
            elif self.start_rect.collidepoint(p):
                self.play_sound("click")
                self._start_play()

    # ===================================================== Eingabe (Spiel)
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup_event(event)
            return

        # Boost beenden, sobald die Taste losgelassen wird (gedrueckt-halten-Logik)
        if event.kind == InputEvent.KEYUP:
            if self.multiplayer:
                if event.key in BOOST_KEYS_P1:
                    self._set_boost(self.snakes[0], False)
                elif event.key in BOOST_KEYS_P2:
                    self._set_boost(self.snakes[1], False)
            else:
                if event.key in BOOST_KEYS_P1 or event.key in BOOST_KEYS_P2:
                    self._set_boost(self.snakes[0], False)
            return

        if event.kind != InputEvent.KEYDOWN:
            return

        if self.game_over:
            if event.key in ("Return", "space"):
                self._start_play()
            return

        # Boost aktivieren, solange die Taste gehalten wird
        if self.multiplayer:
            if event.key in BOOST_KEYS_P1:
                self._set_boost(self.snakes[0], True); return
            if event.key in BOOST_KEYS_P2:
                self._set_boost(self.snakes[1], True); return
        else:
            if event.key in BOOST_KEYS_P1 or event.key in BOOST_KEYS_P2:
                self._set_boost(self.snakes[0], True); return

        # Prestige (nur Einzelspieler)
        if event.key in ("p", "P"):
            self._try_prestige()
            return

        if self.multiplayer:
            self._turn(self.snakes[0], event.key, "p1")
            self._turn(self.snakes[1], event.key, "p2")
        else:
            self._turn(self.snakes[0], event.key, None)

    def _set_boost(self, sn, on):
        if not sn.alive:
            return
        if on:
            # Nur neu starten, wenn genug Ausdauer da ist
            if not sn.boost_on and sn.stamina >= BOOST_MIN_START:
                sn.boost_on = True
                self.play_sound("rotate")
        else:
            sn.boost_on = False

    def _turn(self, sn, key, player):
        dx, dy = sn.direction
        if self.is_action(key, "up", player) and dy == 0:
            sn.next_direction = (0, -1)
        elif self.is_action(key, "down", player) and dy == 0:
            sn.next_direction = (0, 1)
        elif self.is_action(key, "left", player) and dx == 0:
            sn.next_direction = (-1, 0)
        elif self.is_action(key, "right", player) and dx == 0:
            sn.next_direction = (1, 0)

    # ----- Prestige -----------------------------------------------------
    def _can_prestige(self):
        if self.multiplayer:
            return None, False
        req = prestige.next_requirement(self.prestige)
        if req is None:
            return None, False
        sn = self.snakes[0]
        genug_aepfel = self.apples_bank >= req["apples"]
        genug_laenge = len(sn.body) - req["length"] >= MIN_LENGTH
        return req, (genug_aepfel and genug_laenge)

    def _try_prestige(self):
        if self.game_over:
            return
        req, ok = self._can_prestige()
        if not ok:
            return
        self.apples_bank -= req["apples"]
        sn = self.snakes[0]
        del sn.body[:req["length"]]
        sn.grow = 0
        self.prestige += 1
        self.play_sound("select")
        self.rumble(120)

    # ===================================================== Spiellogik
    def update(self, dt):
        self.anim_t += dt
        self._update_particles(dt)
        if self.state != PLAY or self.game_over:
            return

        # Goldapfel-Lebensdauer
        if self.golden is not None:
            self.golden_timer -= dt
            if self.golden_timer <= 0:
                self.golden = None

        # Zeitangriff-Countdown
        if self.mode_key == "timed":
            self.time_left -= dt
            if self.time_left <= 0:
                self.time_left = 0.0
                self._finish_timed()
                return

        # Ausdauer regenerieren (fuer nicht aktiv boostende Schlangen)
        for sn in self.snakes:
            if sn.alive and not sn.boost_on:
                sn.stamina = min(STAMINA_MAX, sn.stamina + STAMINA_REGEN * dt)

        # Schritte abarbeiten
        self._timer += dt
        guard = 0
        while self._timer >= self.interval and not self.game_over:
            self._timer -= self.interval
            self._tick()
            guard += 1
            if guard > 8:
                self._timer = 0.0
                break

    def _tick(self):
        alive = [i for i, sn in enumerate(self.snakes) if sn.alive]
        # Unterschritt 0: alle bewegen sich einmal
        self._advance(alive)
        if self.game_over:
            return
        # Unterschritt 1: Booster bewegen sich ein zweites Mal (= doppeltes Tempo)
        booster = [i for i in alive
                   if self.snakes[i].alive and self.snakes[i].boost_on
                   and self.snakes[i].stamina > 0]
        if booster:
            self._advance(booster)
            for i in booster:
                sn = self.snakes[i]
                sn.stamina = max(0.0, sn.stamina - BOOST_DRAIN)
                if sn.boost_on:
                    self._spawn_particles(sn.body[-1], BOOST_GLOW[i % 2], 2)
                if sn.stamina <= 0:
                    sn.boost_on = False

    def _advance(self, movers):
        """Bewegt die Schlangen in 'movers' um eine Zelle (mit voller Kollision)."""
        new_heads = {}
        for i in movers:
            sn = self.snakes[i]
            if not sn.alive:
                continue
            sn.direction = sn.next_direction
            hx, hy = sn.body[-1]
            nx, ny = hx + sn.direction[0], hy + sn.direction[1]
            if (nx, ny) in self.portals:                 # Teleporter
                nx, ny = self.portals[(nx, ny)]
                nx += sn.direction[0]
                ny += sn.direction[1]
            if self.wrap:
                nx %= self.cols
                ny %= self.rows
            new_heads[i] = (nx, ny)

        tot = set()

        # Wandkollision (nur bei festen Waenden)
        if not self.wrap:
            for i, (nx, ny) in new_heads.items():
                if nx < 0 or nx >= self.cols or ny < 0 or ny >= self.rows:
                    tot.add(i)
        # Hindernisse (immer toedlich)
        for i, kopf in new_heads.items():
            if kopf in self.obstacles:
                tot.add(i)
        # Kopf-an-Kopf
        for i in new_heads:
            for j in new_heads:
                if i < j and new_heads[i] == new_heads[j]:
                    tot.add(i)
                    tot.add(j)
        # Koerperkollision
        belegt = set()
        for i, sn in enumerate(self.snakes):
            if not sn.alive:
                continue
            if i in movers:
                waechst = (new_heads.get(i) == self.food) \
                    or (self.golden is not None and new_heads.get(i) == self.golden) \
                    or (sn.grow > 0)
                koerper = sn.body if waechst else sn.body[1:]
            else:
                koerper = sn.body
            belegt |= set(koerper)
        for i, kopf in new_heads.items():
            if kopf in belegt:
                tot.add(i)

        # Bewegung anwenden
        ate = ate_gold = False
        for i in movers:
            sn = self.snakes[i]
            if not sn.alive:
                continue
            if i in tot:
                sn.alive = False
                self._spawn_particles(sn.body[-1], SNAKE_COLORS[i % 2][1], 12)
                continue
            kopf = new_heads[i]
            sn.body.append(kopf)
            if kopf == self.food:
                self._eat_food(sn)
                ate = True
            elif self.golden is not None and kopf == self.golden:
                self._eat_golden(sn)
                ate_gold = True
            if sn.grow > 0:
                sn.grow -= 1
            else:
                sn.body.pop(0)

        if ate:
            self.play_sound("eat")
            self._place_food()
            if self.golden is None and random.random() < GOLDEN_CHANCE:
                self._place_golden()
        if ate_gold:
            self.golden = None

        self._check_end(tot)

    def _eat_food(self, sn):
        gain = random.randint(1, 2) if self.bonus else 1
        sn.grow += gain * prestige.blocks_per_apple(self.prestige)
        sn.stamina = min(STAMINA_MAX, sn.stamina + 0.12)
        sn.apples += gain
        if self.multiplayer:
            sn.score += gain * 10
        else:
            self.apples_total += gain
            self.apples_bank += gain
            sn.score += gain * 10 * prestige.score_multiplier(self.prestige)
        self.speed_apples += gain
        if self.mode_key == "speed":
            self.interval = max(MIN_INTERVAL, BASE_INTERVAL - self.speed_apples * 0.0025)

    def _eat_golden(self, sn):
        sn.grow += 2 * prestige.blocks_per_apple(self.prestige)
        sn.stamina = STAMINA_MAX
        bonus = 3
        sn.apples += bonus
        if self.multiplayer:
            sn.score += 50
        else:
            self.apples_total += bonus
            self.apples_bank += bonus
            sn.score += 50 * prestige.score_multiplier(self.prestige)
        self._spawn_particles(sn.body[-1], COL_GOLD, 16)
        self.play_sound("point")
        self.rumble(80)

    def _check_end(self, tot):
        if self.multiplayer:
            self.score = max(sn.score for sn in self.snakes)
            lebende = [i for i, sn in enumerate(self.snakes) if sn.alive]
            if len(lebende) <= 1 and (tot or len(lebende) < len(self.snakes)):
                self.game_over = True
                if len(lebende) == 1:
                    self.winner = lebende[0]
                else:
                    self.winner = self._winner_by_score()
                self._end_effects()
        else:
            self.score = self.snakes[0].score
            if not self.snakes[0].alive:
                self.game_over = True
                self._end_effects()

    def _winner_by_score(self):
        s0, s1 = self.snakes[0].score, self.snakes[1].score
        return 0 if s0 > s1 else (1 if s1 > s0 else None)

    def _finish_timed(self):
        """Zeitangriff abgelaufen: beenden und Gewinner bestimmen."""
        self.game_over = True
        if self.multiplayer:
            a0, a1 = self.snakes[0].apples, self.snakes[1].apples
            self.winner = 0 if a0 > a1 else (1 if a1 > a0 else None)
            self.score = max(sn.score for sn in self.snakes)
        else:
            self.score = self.snakes[0].score
        self._end_effects()

    def _end_effects(self):
        if self._effects_done:
            return
        self._effects_done = True
        self.highscore = max(self.highscore, self.score)
        self.play_sound("gameover")
        self.rumble(200)

    # ----- Partikel -----------------------------------------------------
    def _spawn_particles(self, cell, color, n):
        cx = cell[0] * CELL + CELL / 2
        cy = cell[1] * CELL + CELL / 2
        for _ in range(n):
            ang = random.uniform(0, math.tau)
            spd = random.uniform(30, 130)
            self.particles.append([cx, cy, math.cos(ang) * spd, math.sin(ang) * spd,
                                   random.uniform(0.25, 0.55), color])

    def _update_particles(self, dt):
        rest = []
        for p in self.particles:
            p[0] += p[2] * dt
            p[1] += p[3] * dt
            p[4] -= dt
            if p[4] > 0:
                rest.append(p)
        self.particles = rest

    # ===================================================== Zeichnen
    def draw(self):
        if self.state == SETUP:
            self._draw_setup()
            return

        s = self.surface
        s.fill(COL_BG)

        # Gitter
        for x in range(0, self.cols * CELL, CELL):
            pygame.draw.line(s, COL_GRID, (x, 0), (x, self.rows * CELL))
        for y in range(0, self.rows * CELL, CELL):
            pygame.draw.line(s, COL_GRID, (0, y), (self.cols * CELL, y))

        # Hindernisse
        for (x, y) in self.obstacles:
            pygame.draw.rect(s, COL_WALL, (x * CELL, y * CELL, CELL, CELL))
            pygame.draw.rect(s, (95, 104, 128),
                             (x * CELL, y * CELL, CELL, CELL), 1)

        # Portale
        for (a, b, col) in self.portal_pairs:
            for (px, py) in (a, b):
                cx, cy = px * CELL + CELL // 2, py * CELL + CELL // 2
                r = CELL // 2 - 1 + int(1.5 * math.sin(self.anim_t * 6))
                pygame.draw.circle(s, col, (cx, cy), r, 3)
                pygame.draw.circle(s, tuple(c // 2 for c in col), (cx, cy), max(2, r - 4))

        # Futter
        if self.food:
            fx, fy = self.food
            pygame.draw.rect(s, COL_FOOD,
                             (fx * CELL + 2, fy * CELL + 2, CELL - 4, CELL - 4),
                             border_radius=6)
        # Goldapfel (pulsiert + blinkt wenn er gleich verschwindet)
        if self.golden is not None:
            gx, gy = self.golden
            blink = self.golden_timer > 1.5 or int(self.anim_t * 8) % 2 == 0
            if blink:
                cx, cy = gx * CELL + CELL // 2, gy * CELL + CELL // 2
                r = CELL // 2 - 1 + int(1.5 * math.sin(self.anim_t * 8))
                pygame.draw.circle(s, COL_GOLD, (cx, cy), r)
                pygame.draw.circle(s, (255, 245, 200), (cx - 2, cy - 2), 2)

        # Partikel
        for p in self.particles:
            a = max(0, min(255, int(255 * (p[4] / 0.55))))
            surf = pygame.Surface((6, 6), pygame.SRCALPHA)
            pygame.draw.circle(surf, (*p[5], a), (3, 3), 3)
            s.blit(surf, (p[0] - 3, p[1] - 3))

        # Schlangen
        for idx, sn in enumerate(self.snakes):
            self._draw_snake(s, sn, idx)

        self._draw_hud()

        if self.game_over:
            self._draw_game_over()

    def _draw_snake(self, s, sn, idx):
        koerper, kopf = SNAKE_COLORS[idx % len(SNAKE_COLORS)]
        if not sn.alive:
            koerper = tuple(c // 2 for c in koerper)
            kopf = koerper

        n = len(sn.body)
        # Boost-Glow um den Kopf
        if sn.alive and sn.boost_on and sn.stamina > 0:
            hx, hy = sn.body[-1]
            glow = pygame.Surface((CELL * 3, CELL * 3), pygame.SRCALPHA)
            pulse = 120 + int(60 * math.sin(self.anim_t * 14))
            pygame.draw.circle(glow, (*BOOST_GLOW[idx % 2], pulse),
                               (CELL * 3 // 2, CELL * 3 // 2), CELL)
            s.blit(glow, (hx * CELL + CELL // 2 - CELL * 3 // 2,
                          hy * CELL + CELL // 2 - CELL * 3 // 2))

        for i, (x, y) in enumerate(sn.body):
            is_head = (i == n - 1)
            if is_head:
                farbe = kopf
            else:
                # sanfter Verlauf vom Kopf (hell) zum Schwanz (dunkler)
                t = i / max(1, n - 1)
                farbe = tuple(int(k + (c - k) * (1 - t * 0.5))
                              for k, c in zip(kopf, koerper))
            pygame.draw.rect(s, farbe,
                             (x * CELL + 1, y * CELL + 1, CELL - 2, CELL - 2),
                             border_radius=5)

        # Augen auf dem Kopf
        if sn.body:
            self._draw_eyes(s, sn)

    def _draw_eyes(self, s, sn):
        hx, hy = sn.body[-1]
        dx, dy = sn.direction
        cx, cy = hx * CELL + CELL / 2, hy * CELL + CELL / 2
        px, py = -dy, dx                       # senkrecht zur Blickrichtung
        for sign in (-1, 1):
            ex = cx + dx * 3 + px * sign * 4
            ey = cy + dy * 3 + py * sign * 4
            pygame.draw.circle(s, (250, 250, 250), (int(ex), int(ey)), 3)
            pygame.draw.circle(s, (20, 20, 30),
                               (int(ex + dx * 1.5), int(ey + dy * 1.5)), 1)

    # ----- HUD ----------------------------------------------------------
    def _draw_hud(self):
        s = self.surface

        # WLS-Anzeige (durch die Waende?)
        wls_col = COL_WLS_ON if self.wrap else COL_WLS_OFF
        wls = self.font.render("WLS", True, wls_col)
        s.blit(wls, wls.get_rect(midtop=(self.width // 2, 6)))
        mode_name = MODES[self.mode_index]["name"]
        mimg = self._small.render(mode_name, True, COL_ACCENT)
        s.blit(mimg, mimg.get_rect(midtop=(self.width // 2, 30)))

        # Zeitangriff-Uhr
        if self.mode_key == "timed":
            col = COL_FOOD if self.time_left <= 10 else COL_TEXT
            timg = self.font.render(f"{self.time_left:04.1f}s", True, col)
            s.blit(timg, timg.get_rect(midtop=(self.width // 2, 50)))

        if self.multiplayer:
            p1 = self.font.render(f"P1: {self.snakes[0].score}", True, SNAKE_COLORS[0][1])
            p2 = self.font.render(f"P2: {self.snakes[1].score}", True, SNAKE_COLORS[1][1])
            s.blit(p1, (10, 8))
            s.blit(p2, (self.width - p2.get_width() - 10, 8))
            self._draw_boost_bar(10, self.height - 22, 150, self.snakes[0], 0)
            self._draw_boost_bar(self.width - 160, self.height - 22, 150,
                                 self.snakes[1], 1)
            return

        # Einzelspieler
        s.blit(self.font.render(f"Punkte: {self.score}", True, COL_TEXT), (10, 8))
        s.blit(self._small.render(
            f"Aepfel: {self.apples_total}   Bank: {self.apples_bank}",
            True, COL_FOOD), (10, 34))
        if self.prestige > 0:
            blocks = prestige.blocks_per_apple(self.prestige)
            info = self._small.render(
                f"Prestige {prestige.roman(self.prestige)}   {blocks} Bloecke/Apfel",
                True, COL_MULT)
            s.blit(info, (10, 54))

        best = self._small.render(f"Best: {self.highscore}", True, COL_DIM)
        s.blit(best, (self.width - best.get_width() - 10, 10))

        self._draw_boost_bar(10, self.height - 22, 180, self.snakes[0], 0)

        if not self.game_over and self.mode_key != "timed":
            self._draw_next_prestige()

    def _draw_boost_bar(self, x, y, w, sn, idx):
        s = self.surface
        pygame.draw.rect(s, (30, 34, 46), (x, y, w, 14), border_radius=5)
        fw = int((w - 2) * max(0.0, min(1.0, sn.stamina)))
        if sn.boost_on and sn.stamina > 0:
            col = (255, 200, 80)          # aktiv = gold
        elif sn.stamina >= BOOST_MIN_START:
            col = BOOST_GLOW[idx % 2]     # bereit
        else:
            col = (120, 90, 90)           # zu leer zum Starten
        pygame.draw.rect(s, col, (x + 1, y + 1, fw, 12), border_radius=5)
        lab = self._tiny.render("BOOST", True, (15, 15, 20))
        s.blit(lab, (x + 6, y))

    def _draw_next_prestige(self):
        s = self.surface
        req, ok = self._can_prestige()
        if req is None:
            txt = f"MAX PRESTIGE {prestige.roman(self.prestige)} erreicht"
            col = COL_MULT
        else:
            txt = (f"Prestige {req['roman']}:  {req['apples']} Aepfel   "
                   f"-{req['length']} Laenge")
            if ok:
                txt += "   ->  P druecken!"
                col = COL_MULT
            else:
                col = COL_DIM
        img = self._small.render(txt, True, col)
        s.blit(img, img.get_rect(midbottom=(self.width // 2, self.height - 8)))

    def _draw_game_over(self):
        ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        ov.fill((0, 0, 0, 150))
        self.surface.blit(ov, (0, 0))

        if self.multiplayer:
            if self.winner is None:
                text, farbe = "UNENTSCHIEDEN", COL_TEXT
            else:
                text = f"SPIELER {self.winner + 1} GEWINNT"
                farbe = SNAKE_COLORS[self.winner][1]
            self.draw_center_text(text, self.big_font, farbe, -30)
            a0, a1 = self.snakes[0].apples, self.snakes[1].apples
            self.draw_center_text(f"Aepfel  P1: {a0}   P2: {a1}", self.font, COL_DIM, 14)
            self.draw_center_text("Enter = Neustart", self.font, COL_TEXT, 48)
            return

        titel = "ZEIT ABGELAUFEN" if (self.mode_key == "timed" and self.snakes[0].alive) \
            else "GAME OVER"
        self.draw_center_text(titel, self.big_font, COL_FOOD, -60)
        self.draw_center_text(f"Aepfel eingesammelt: {self.apples_total}",
                              self.font, COL_FOOD, -14)
        if self.prestige > 0:
            self.draw_center_text(f"Prestige {prestige.roman(self.prestige)}",
                                  self.font, COL_MULT, 16)
        self.draw_center_text("Enter = Neustart", self.font, COL_TEXT, 46)

    # ----- Setup zeichnen ----------------------------------------------
    def _draw_setup(self):
        s = self.surface
        s.fill(COL_BG)

        title = self.big_font.render("SNAKE", True, COL_TEXT)
        s.blit(title, title.get_rect(center=(self.width // 2, 52)))
        modus = "Mehrspieler" if self.multiplayer else "Einzelspieler"
        sub = self._small.render(f"{modus}   -   Highscore: {self.highscore}",
                                 True, COL_DIM)
        s.blit(sub, sub.get_rect(center=(self.width // 2, 88)))

        # Modus-Auswahl
        m = MODES[self.mode_index]
        pygame.draw.rect(s, (38, 44, 60), self.mode_panel, border_radius=10)
        pygame.draw.rect(s, COL_ACCENT, self.mode_panel, 2, border_radius=10)
        name = self.font.render(m["name"], True, COL_TEXT)
        s.blit(name, name.get_rect(center=(self.mode_panel.centerx,
                                           self.mode_panel.top + 20)))
        desc = self._small.render(m["desc"], True, COL_DIM)
        s.blit(desc, desc.get_rect(center=(self.mode_panel.centerx,
                                           self.mode_panel.top + 44)))
        for r, sym in ((self.mode_left, "<"), (self.mode_right, ">")):
            arr = self.big_font.render(sym, True, COL_ACCENT)
            s.blit(arr, arr.get_rect(center=r.center))
        dots = " ".join("*" if i == self.mode_index else "." for i in range(len(MODES)))
        d = self._tiny.render(dots, True, COL_DIM)
        s.blit(d, d.get_rect(center=(self.width // 2, self.mode_panel.bottom + 10)))

        self._draw_setup_toggle(self.wrap_rect, "Waende: durchgehen", self.wrap)
        self._draw_setup_toggle(self.bonus_rect, "Bonus-Aepfel (1-2)", self.bonus)

        pygame.draw.rect(s, COL_BTN_ON, self.start_rect, border_radius=10)
        st = self.font.render("START", True, COL_TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))

        hint = self._small.render(
            "Pfeile/1-5 = Modus   W = Waende   B = Bonus   Enter = Start",
            True, COL_DIM)
        s.blit(hint, hint.get_rect(center=(self.width // 2, self.height - 34)))
        boost = self._tiny.render(
            "Boost im Spiel:  P1 = Leertaste/Shift,  P2 = Enter/Shift-rechts",
            True, (120, 200, 150))
        s.blit(boost, boost.get_rect(center=(self.width // 2, self.height - 14)))

    def _draw_setup_toggle(self, rect, label, an):
        s = self.surface
        pygame.draw.rect(s, COL_BTN_ON if an else COL_BTN, rect, border_radius=8)
        pygame.draw.rect(s, COL_DIM, rect, 1, border_radius=8)
        lab = self.font.render(label, True, COL_TEXT)
        s.blit(lab, (rect.x + 16, rect.centery - lab.get_height() // 2))
        wert = "AN" if an else "AUS"
        col = COL_WLS_ON if an else COL_DIM
        img = self.font.render(f"< {wert} >", True, col)
        s.blit(img, (rect.right - img.get_width() - 16,
                     rect.centery - img.get_height() // 2))
