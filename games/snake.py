# -*- coding: utf-8 -*-
"""
snake.py
========
Snake - komplett überarbeitete Deluxe-Version mit Spielmodi und Boost.

Neu
---
- 3D-ANSICHT (Taste V im Setup): Das Spielfeld wird als Echtzeit-3D-Szene
  gerendert (Software-Renderer: Perspektivprojektion + Painter-Algorithmus,
  kein OpenGL nötig). Eine Verfolgerkamera schwebt hinter dem Kopf der
  Schlange, gelenkt wird relativ zur Blickrichtung (links/rechts drehen).
  Mit Distanz-Nebel, Sternenhimmel, Schachbrett-Boden, Banden, rotierenden
  Futter-Kristallen, 3D-Partikeln und Kamera-Shake. In 3D wählbar:
  Klassisch und Hindernisse; die Wände sind dort immer fest.
  Nach dem Game Over umkreist die Kamera langsam die Schlange.
- BOOST: Solange die Boost-Taste (Einzelspieler: Leertaste/Shift) gedrückt
  gehalten wird, läuft der Turbo. Die Schlange bewegt sich doppelt so schnell
  und verbraucht dabei Ausdauer (Anzeige als Balken). Ist die Ausdauer leer,
  schaltet der Boost automatisch ab; sie lädt sich mit der Zeit wieder auf.
  Goldäpfel füllen sie sofort ganz auf. In 3D weitet der Boost das Sichtfeld.
- SPIELMODI (im Setup wählbar):
    * Klassisch    - der Klassiker.
    * Speed-Rush   - wird mit jedem Apfel schneller.
    * Hindernisse  - feste Blöcke im Spielfeld, die tödlich sind.
    * Portale      - Teleporter-Paare: rein ins eine, raus aus dem anderen.
    * Zeitangriff  - 60 Sekunden, so viele Äpfel wie möglich.
- Goldäpfel: erscheinen zeitweise, bringen viele Punkte und füllen den Boost.
- ÄPFEL AUF DER MAP (Setup, Taste F): 1/2/3/5 gleichzeitig liegende Äpfel.
  Mit mehr Äpfeln muss man weniger suchen und kann flotter Punkte machen;
  wird ein Apfel gegessen, wird sofort ein neuer nachgelegt (auf die
  gewählte Anzahl aufgefüllt). In der 2D-Ansicht blenden Äpfel weich mit
  leichtem Überschwingen ein und werden beim Essen sichtbar "weggeknabbert"
  (schrumpfen + Ring + Krümel) statt einfach zu verschwinden.
- Weiterhin: Wände-durchgehen, Bonus-Äpfel, Mehrspieler (2 Schlangen),
  Prestige (Einzelspieler) - siehe prestige.py.
- Neue Optik: abgerundete Schlange mit Augen, Boost-Glow, Partikel,
  überarbeiteter Setup-Screen und HUD.

Steuerung
---------
- Bewegung: die in den Optionen belegten Tasten (Standard P1 = WASD, P2 = Pfeile).
- 3D-Ansicht: links/rechts (bzw. A/D) drehen die Schlange relativ zur Kamera.
- Boost:  P1 = Leertaste / Shift-links,  P2 = Enter / Shift-rechts.
- Prestige (Einzelspieler): P.   Pause-los; Enter/Leertaste startet nach Game Over neu.
"""

import math
import random
import time
import pygame

import competitive
import highscore
import i18n
import ngb
import prestige
import settings as settings_mod
from game_base import Game, InputEvent

CELL = 20                       # Kantenlänge einer Rasterzelle in Pixeln
BASE_INTERVAL = 0.12            # Sekunden pro Schritt (Normaltempo)
MIN_INTERVAL = 0.055           # schnellstes Tempo (Speed-Rush)
MIN_LENGTH = 3                  # so kurz darf eine Schlange durch Prestige max. werden

# Boost / Ausdauer
STAMINA_MAX = 1.0
STAMINA_REGEN = 0.22            # Aufladung pro Sekunde (wenn nicht geboostet)
BOOST_DRAIN = 0.045            # Verbrauch pro zusätzlichem Boost-Schritt
BOOST_MIN_START = 0.15         # so viel Ausdauer braucht man mindestens zum Starten
# HARDCORE (nur Competitive): jeder Boost-Schritt frisst so viele Längen-Blöcke.
HARDCORE_BOOST_LEN_COST = 1

# Boost-Tasten (keine Richtungstasten, damit es nicht kollidiert)
BOOST_KEYS_P1 = ("space", "Shift_L")
BOOST_KEYS_P2 = ("Return", "Shift_R", "KP_Enter")

GOLDEN_LIFETIME = 6.0          # Sekunden, die ein Goldapfel liegen bleibt
GOLDEN_CHANCE = 0.20           # Chance, nach einem normalen Apfel einen Goldapfel zu setzen
TIMED_SECONDS = 60.0

# ----- Competitive-Modus ----------------------------------------------------
# Spezialäpfel: blau (Slot-Machine) und lila (Längen-Wette).
SPECIAL_LIFETIME = 8.0         # Sekunden, die ein blauer/lila Apfel liegen bleibt
BLUE_CHANCE = 0.12             # Chance je Apfel, einen blauen Apfel nachzulegen
PURPLE_CHANCE = 0.16           # Chance je Apfel, einen lila Apfel nachzulegen
SPAWN_BONUS_TIME = 10.0        # so lange legt der Slot-Bonus zusätzliche Äpfel nach
# Slot-Machine-Timing: Stoppzeitpunkte der drei Walzen + Anzeigedauer danach.
SLOT_REEL_STOPS = (0.9, 1.35, 1.8)
SLOT_SHOW = 1.7
SLOT_SPIN_SPEED = 16.0         # Symbole pro Sekunde beim Drehen

# Banner: gut sichtbare Einblendung oben mittig (z.B. lila Multiplikator)
BANNER_TIME = 2.0

# Wählbare Anzahl gleichzeitig auf dem Feld liegender Äpfel (Setup-Einstellung).
APPLE_COUNTS = (1, 2, 3, 5)

# Apfel-Animationen (nur 2D-Ansicht)
FOOD_SPAWN_ANIM = 0.28         # Sekunden: Apfel "morpht" beim Erscheinen herein
FOOD_EAT_ANIM = 0.30           # Sekunden: Apfel wird beim Essen weggeknabbert


def _ease_out_back(p):
    """Weiche Einblendung mit leichtem Überschwingen (0 -> ~1.1 -> 1)."""
    if p >= 1.0:
        return 1.0
    c1 = 1.70158
    c3 = c1 + 1.0
    return 1.0 + c3 * (p - 1) ** 3 + c1 * (p - 1) ** 2

# Spielzustände
SETUP, PLAY, PERSONALIZE, CAM3D = "setup", "play", "personalize", "cam3d"

COL_BG = (15, 15, 25)
COL_GRID = (25, 25, 40)
COL_FOOD = (240, 90, 90)
COL_GOLD = (255, 205, 70)
COL_TEXT = (230, 230, 230)
COL_DIM = (150, 158, 176)
COL_BTN = (44, 50, 66)
COL_BTN_ON = (60, 120, 80)
COL_WLS_ON = (90, 230, 130)     # WLS grün = durch die Wände gehen ist AN
COL_WLS_OFF = (105, 105, 120)   # WLS grau  = feste Wände
COL_MULT = (255, 210, 90)       # Multiplikator / Prestige (gold)
COL_WALL = (70, 78, 98)         # Hindernis-Blöcke
COL_ACCENT = (90, 160, 240)
COL_BLUE = (90, 150, 245)       # blauer Apfel (Slot-Machine)
COL_PURPLE = (185, 110, 240)    # lila Apfel (Längen-Wette)
COL_HARDCORE = (235, 45, 55)    # HARDCORE-Modus (rotes Leuchten)

# Farben je Schlange: (Körper, Kopf)
SNAKE_COLORS = [
    ((80, 220, 120), (150, 255, 180)),   # Spieler 1 (grün)
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
    dict(key="walls",   name="Hindernisse", desc="Feste Blöcke im Feld - tödlich!"),
    dict(key="portal",  name="Portale",     desc="Teleporter: rein und woanders raus."),
    dict(key="timed",   name="Zeitangriff", desc="60s - so viele Äpfel wie möglich."),
    dict(key="competitive", name="Competitive",
         desc="Level-Aufstieg, Slot-Machine & Wett-Äpfel."),
]
MODE_KEYS = [m["key"] for m in MODES]

# ----- 3D-Ansicht -----------------------------------------------------------
# Die 3D-Ansicht nutzt exakt dieselbe Gitter-Spiellogik und rendert sie als
# Software-3D-Szene. Weltkoordinaten: x = Spalten, z = Zeilen, y = Höhe
# (Boden bei y = 0, eine Rasterzelle = 1.0 Einheiten).
MODES_3D = ("classic", "walls")   # in der 3D-Ansicht wählbare Spielmodi

NEAR = 0.12                       # Nahebene der Kamera (Clipping)
FOG_START, FOG_END = 7.0, 24.0    # Distanz-Nebel: Beginn / volle Stärke
FOV_NORMAL = 1.12                 # Brennweite normal (x Fensterhöhe)
FOV_BOOST = 0.94                  # Brennweite bei Boost (weitwinkliger = Speed-Gefühl)

CAM_BACK = 3.6                    # Kamera: Abstand hinter dem Kopf
CAM_H = 2.6                       # Kamera: Höhe über dem Boden (Standard)
CAM_AHEAD = 2.6                   # Blickpunkt: so weit vor dem Kopf
CAM_LOOK_H = 0.35                 # Blickpunkt: Höhe
CAM_SMOOTH = 6.0                  # Glättungsfaktor der Kamerabewegung (1/s)

# 3D-Kamera-Optionen (im Setup unter "3D-Kamera" einstellbar, in settings.json)
FOV_BOOST_DELTA = FOV_NORMAL - FOV_BOOST   # so viel weitwinkliger beim Boost
CAM_FOV_MIN, CAM_FOV_MAX, CAM_FOV_STEP = 0.80, 1.50, 0.02
CAM_H_MIN, CAM_H_MAX, CAM_H_STEP = 1.6, 4.2, 0.1

COL_SKY_TOP = (8, 10, 22)         # Himmel oben
COL_SKY_HOR = (46, 52, 86)        # Himmel am Horizont
COL_FOG = (34, 39, 66)            # Nebel-/Horizontfarbe (Boden blendet dahin aus)
COL_TILE_A = (26, 29, 48)         # Boden-Schachbrett hell
COL_TILE_B = (32, 36, 58)         # Boden-Schachbrett dunkel
COL_BORDER = (84, 96, 138)        # Bande am Spielfeldrand


def _rotate(direction, turn):
    """Dreht einen Gitter-Richtungsvektor um 90 Grad ("L" oder "R")."""
    dx, dy = direction
    return (dy, -dx) if turn == "L" else (-dy, dx)


class _Snake:
    """Zustand einer einzelnen Schlange (Körper: Kopf am Listenende)."""

    def __init__(self, body, direction, player):
        self.body = list(body)
        self.direction = direction
        self.next_direction = direction
        self.player = player            # "p1" / "p2"
        self.alive = True
        self.score = 0
        self.apples = 0                 # eigene Äpfel (Mehrspieler / Zeitangriff)
        self.grow = 0                   # ausstehende Wachstums-Blöcke
        self.stamina = STAMINA_MAX      # Boost-Ausdauer (0..1)
        self.boost_on = False           # Boost gerade aktiv?
        self.size_frac = 0.0            # Nachkomma-Rest der Größe (aus dem Gambling)
        self.prev_body = list(body)     # Positionen vor dem letzten Schritt (3D-Interpolation)
        self.turn_queue = []            # gepufferte Drehungen in der 3D-Ansicht ("L"/"R")


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
        self.hardcore = bool(snk.get("hardcore", False))   # nur im Competitive aktiv
        ac = int(snk.get("apples", 1))
        self.apple_count = ac if ac in APPLE_COUNTS else 1
        mode_key = snk.get("mode", "classic")
        self.mode_index = MODE_KEYS.index(mode_key) if mode_key in MODE_KEYS else 0
        self.view3d = bool(snk.get("view3d", False))
        # 3D-Kamera-Optionen (Smooth-Shake, FOV, Höhe, Seitwärts-Shake)
        self.cam_smooth = bool(snk.get("cam_smooth", True))
        self.cam_fov = min(CAM_FOV_MAX, max(CAM_FOV_MIN,
                                            float(snk.get("cam_fov", FOV_NORMAL))))
        self.cam_height = min(CAM_H_MAX, max(CAM_H_MIN,
                                             float(snk.get("cam_height", CAM_H))))
        self.cam_turn_shake = bool(snk.get("cam_turn_shake", False))
        if self.view3d_active and self.mode_key not in MODES_3D:
            self.mode_index = MODE_KEYS.index("classic")
        if self.multiplayer and self.mode_key == "competitive":
            self.mode_index = MODE_KEYS.index("classic")   # Competitive = Einzelspieler

        self._small = pygame.font.SysFont("consolas", 16)
        self._tiny = pygame.font.SysFont("consolas", 13)
        self.highscore = highscore.load_highscores().get(self.highscore_key, 0)

        self.particles = []
        self.anim_t = 0.0
        self._ngb_menu = None          # aktives Personalisierungs-Menü (oder None)
        self._grid_surf = None         # gecachtes Raster-Overlay (NGB)
        self._grid_key = None
        self._grid_font = pygame.font.SysFont("consolas", max(9, CELL - 8))

        # Zustand der 3D-Ansicht
        self.particles3d = []          # [x, y, z, vx, vy, vz, life, farbe]
        self._shake = 0.0              # Kamera-Shake (Restdauer)
        self._sky_cache = None         # ((w, h), Surface) - Himmels-Gradient
        self._stars = [(random.uniform(0, math.tau), random.random(),
                        random.choice((1, 1, 2)), random.uniform(0, math.tau))
                       for _ in range(70)]

        self._build_setup_layout()
        self._reset_run_stats()
        self._new_board()
        self.state = SETUP

    @property
    def mode_key(self):
        return MODES[self.mode_index]["key"]

    @property
    def view3d_active(self):
        """3D-Ansicht läuft nur im Einzelspieler (eine Kamera pro Schlange)."""
        return self.view3d and not self.multiplayer

    @property
    def competitive(self):
        """Competitive-Modus (mit Level-Aufstieg, Slot-Machine & Wett-Äpfeln)."""
        return self.mode_key == "competitive" and not self.multiplayer

    @property
    def hardcore_active(self):
        """HARDCORE gibt es nur im Competitive: Boost frisst Länge, alles leuchtet rot."""
        return self.competitive and self.hardcore

    @property
    def wrap_active(self):
        """Wände-durchgehen; in der 3D-Ansicht sind die Wände immer fest."""
        return self.wrap and not self.view3d_active

    def _allowed_modes(self):
        """Indizes der aktuell wählbaren Spielmodi.

        3D: nur Klassisch/Hindernisse. Competitive gibt es nur im Einzelspieler.
        """
        if self.view3d_active:
            return [i for i, m in enumerate(MODES) if m["key"] in MODES_3D]
        if self.multiplayer:
            return [i for i, m in enumerate(MODES) if m["key"] != "competitive"]
        return list(range(len(MODES)))

    def _reset_run_stats(self):
        self.apples_total = 0
        self.apples_bank = 0
        self.prestige = 0
        self.speed_apples = 0
        # Competitive: Level-Aufstieg + Slot-Bonus (zusätzliche Äpfel auf Zeit)
        self.comp_level = 0
        self.spawn_bonus = 0
        self.spawn_bonus_t = 0.0

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
        self._reset_camera()

    def _reset_camera(self):
        """Setzt die 3D-Verfolgerkamera hinter den Kopf von Spieler 1."""
        self._run_t = 0.0              # Laufzeit der Runde (für Einblend-Hinweise)
        self._orbit_a = 0.0            # Orbit-Winkel nach dem Game Over
        self._go_last = None           # Zeitmessung der Game-Over-Animation
        self._fov_mul = self.cam_fov
        sn = self.snakes[0]
        hx, hy = sn.body[-1]
        fx, fz = float(sn.direction[0]), float(sn.direction[1])
        self._cam_dir = (fx, fz)
        head = (hx + 0.5, 0.0, hy + 0.5)
        self._cam_pos = (head[0] - fx * CAM_BACK, self.cam_height, head[2] - fz * CAM_BACK)
        self._cam_look = (head[0] + fx * CAM_AHEAD, CAM_LOOK_H, head[2] + fz * CAM_AHEAD)

    def _start_play(self):
        self.score = 0
        self.game_over = False
        self.winner = None
        self._effects_done = False
        self.particles = []
        self.particles3d = []
        self._shake = 0.0
        self._reset_run_stats()
        self._new_board()
        self.state = PLAY

    # ----- Modus-Layout (Hindernisse / Portale / Zeit) ------------------
    def _build_mode_layout(self):
        self.obstacles = set()
        self.portals = {}
        self.portal_pairs = []
        self.foods = set()             # alle aktuell liegenden Äpfel (Zellen)
        self._food_anim = {}           # Zelle -> Einblend-Alter (Sek.), 2D-Morph
        self._eat_fx = []              # 2D-Iss-Effekte: dicts(cell, t)
        self.golden = None
        self.golden_timer = 0.0
        self.time_left = TIMED_SECONDS

        # Competitive-Spezialäpfel + Slot-Machine + aufsteigende Hinweistexte
        self.specials = {}             # Zelle -> dict(type, timer)  (blau/lila)
        self.slot = None               # aktive Slot-Machine (dict) oder None
        self.float_texts = []          # aufsteigende, verblassende Hinweistexte
        self._banner = None            # große Einblendung oben mittig (Multiplikator)
        self._purple_pending = None    # nach dem Schritt anzuwendender lila Effekt
        self._slot_pending = None      # nach dem Schritt zu öffnende Slot-Machine

        # Zellen, die frei bleiben müssen (Schlangen + Startbahn nach rechts)
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
            # kleine Cluster (1-3 Blöcke) für interessantere Formen
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
        belegt |= self.foods
        belegt |= set(self.specials.keys())
        if self.golden:
            belegt.add(self.golden)
        return belegt

    def _apple_target(self):
        """Gewünschte Anzahl gleichzeitig liegender (normaler) Äpfel.

        Im Competitive-Modus bestimmt das Level die Anzahl (Start: 1 Apfel);
        der Slot-Bonus legt für kurze Zeit weitere Äpfel oben drauf.
        """
        if self.competitive:
            base = competitive.apples_on_field(self.comp_level)
            bonus = self.spawn_bonus if self.spawn_bonus_t > 0 else 0
            return min(competitive.MAX_APPLES + 5, base + bonus)
        return self.apple_count

    def _place_food(self):
        """Füllt das Feld auf die gewünschte Anzahl gleichzeitiger Äpfel auf."""
        belegt = self._blocked_cells()
        frei = [(x, y) for x in range(self.cols) for y in range(self.rows)
                if (x, y) not in belegt]
        random.shuffle(frei)
        while len(self.foods) < self._apple_target() and frei:
            cell = frei.pop()
            self.foods.add(cell)
            self._food_anim[cell] = 0.0        # startet die Einblend-Animation

    # ----- Competitive: Spezialäpfel (blau/lila) ------------------------
    def _maybe_spawn_special(self):
        """Legt nach einem Apfel evtl. einen blauen oder lila Apfel nach."""
        if self.specials:
            return                     # immer nur ein Spezialapfel gleichzeitig
        r = random.random()
        if r < BLUE_CHANCE:
            self._place_special("blue")
        elif r < BLUE_CHANCE + PURPLE_CHANCE:
            self._place_special("purple")

    def _place_special(self, typ):
        belegt = self._blocked_cells()
        frei = [(x, y) for x in range(self.cols) for y in range(self.rows)
                if (x, y) not in belegt]
        if frei:
            self.specials[random.choice(frei)] = dict(type=typ, timer=SPECIAL_LIFETIME)

    def _place_golden(self):
        belegt = self._blocked_cells()
        frei = [(x, y) for x in range(self.cols) for y in range(self.rows)
                if (x, y) not in belegt]
        if frei:
            self.golden = random.choice(frei)
            self.golden_timer = GOLDEN_LIFETIME

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(420, self.width - 60)

        # Personalisieren-Knopf (Pinsel) - ganz oben rechts, ohne Beschriftung
        self.brush_rect = pygame.Rect(self.width - 42, 8, 34, 34)

        # Modus-Auswahl: Pfeile + Panel
        self.mode_panel = pygame.Rect(cx - bw // 2, 106, bw, 58)
        self.mode_left = pygame.Rect(self.mode_panel.left, 106, 40, 58)
        self.mode_right = pygame.Rect(self.mode_panel.right - 40, 106, 40, 58)

        bh, gap = 40, 6
        y0 = 180
        self.view_rect = pygame.Rect(cx - bw // 2, y0, bw, bh)
        self.wrap_rect = pygame.Rect(cx - bw // 2, y0 + (bh + gap), bw, bh)
        self.bonus_rect = pygame.Rect(cx - bw // 2, y0 + 2 * (bh + gap), bw, bh)
        self.apples_rect = pygame.Rect(cx - bw // 2, y0 + 3 * (bh + gap), bw, bh)
        self.start_rect = pygame.Rect(cx - 95, y0 + 4 * (bh + gap) + 4, 190, 48)

        # 3D-Kamera-Menü (eigener Screen; nur im 3D-Modus erreichbar)
        cbw, cbh, cgap, cy0 = min(440, self.width - 50), 46, 10, 150
        self.cam_smooth_rect = pygame.Rect(cx - cbw // 2, cy0, cbw, cbh)
        self.cam_fov_rect = pygame.Rect(cx - cbw // 2, cy0 + (cbh + cgap), cbw, cbh)
        self.cam_height_rect = pygame.Rect(cx - cbw // 2, cy0 + 2 * (cbh + cgap), cbw, cbh)
        self.cam_turn_rect = pygame.Rect(cx - cbw // 2, cy0 + 3 * (cbh + cgap), cbw, cbh)
        self.cam_back_rect = pygame.Rect(cx - 95, cy0 + 4 * (cbh + cgap) + 8, 190, 46)

        def _pm(rect):                     # -/+ Knöpfe rechts in einer Wertzeile
            plus = pygame.Rect(rect.right - 46, rect.centery - 16, 32, 32)
            minus = pygame.Rect(rect.right - 148, rect.centery - 16, 32, 32)
            return minus, plus
        self.cam_fov_minus, self.cam_fov_plus = _pm(self.cam_fov_rect)
        self.cam_height_minus, self.cam_height_plus = _pm(self.cam_height_rect)

    def _save_snake_setting(self, key, value):
        if isinstance(self.settings, dict):
            snk = self.settings.setdefault("snake", {})
            snk[key] = value
            settings_mod.save_settings(self.settings)

    def _toggle_setting(self, key):
        if key == "wrap" and self.view3d_active:
            return                       # in 3D sind die Wände immer fest
        neu = not getattr(self, "wrap" if key == "wrap" else "bonus")
        if key == "wrap":
            self.wrap = neu
        else:
            self.bonus = neu
        self._save_snake_setting("wrap" if key == "wrap" else "bonus_apple", neu)
        self.play_sound("select")

    def _toggle_hardcore(self):
        """HARDCORE ein/aus (nur im Competitive wählbar)."""
        if not self.competitive:
            return
        self.hardcore = not self.hardcore
        self._save_snake_setting("hardcore", self.hardcore)
        self.play_sound("select")

    def _cycle_apples(self):
        """Schaltet die Anzahl gleichzeitig liegender Äpfel weiter (1/2/3/5)."""
        if self.competitive:
            return                     # im Competitive bestimmt das Level die Anzahl
        i = APPLE_COUNTS.index(self.apple_count) if self.apple_count in APPLE_COUNTS else 0
        self.apple_count = APPLE_COUNTS[(i + 1) % len(APPLE_COUNTS)]
        self._save_snake_setting("apples", self.apple_count)
        self.play_sound("select")

    def _cycle_mode(self, step):
        allowed = self._allowed_modes()
        if self.mode_index in allowed:
            i = allowed.index(self.mode_index)
            self.mode_index = allowed[(i + step) % len(allowed)]
        else:
            self.mode_index = allowed[0]
        self._save_snake_setting("mode", self.mode_key)
        self.play_sound("click")

    def _toggle_view(self):
        """Schaltet zwischen 2D- und 3D-Ansicht um (nur Einzelspieler)."""
        if self.multiplayer:
            return
        self.view3d = not self.view3d
        if self.view3d and self.mode_key not in MODES_3D:
            self.mode_index = MODE_KEYS.index("classic")
            self._save_snake_setting("mode", self.mode_key)
        self._save_snake_setting("view3d", self.view3d)
        self.play_sound("select")

    def _open_personalize(self):
        """Öffnet das Personalisierungs-Menü (nur Optik) über ngb.py."""
        self._ngb_menu = ngb.open_head_color_menu(self.width, self.height,
                                                  self.play_sound)
        self.state = PERSONALIZE
        self.play_sound("click")

    def _open_cam3d(self):
        """Öffnet das 3D-Kamera-Menü (Smooth-Shake, FOV, Höhe, Abbiege-Ruckeln)."""
        self.state = CAM3D
        self.play_sound("click")

    def _adjust_fov(self, d):
        self.cam_fov = round(min(CAM_FOV_MAX, max(CAM_FOV_MIN, self.cam_fov + d)), 2)
        self._save_snake_setting("cam_fov", self.cam_fov)
        self.play_sound("click")

    def _adjust_height(self, d):
        self.cam_height = round(min(CAM_H_MAX, max(CAM_H_MIN, self.cam_height + d)), 2)
        self._save_snake_setting("cam_height", self.cam_height)
        self.play_sound("click")

    def _handle_cam3d_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Escape", "Return", "space", "k", "K"):
                self.state = SETUP
                self.play_sound("click")
            elif event.key in ("Left", "a", "A"):
                self._adjust_fov(-CAM_FOV_STEP)
            elif event.key in ("Right", "d", "D"):
                self._adjust_fov(+CAM_FOV_STEP)
            elif event.key in ("Up", "w", "W"):
                self._adjust_height(+CAM_H_STEP)
            elif event.key in ("Down", "s", "S"):
                self._adjust_height(-CAM_H_STEP)
            return
        if event.kind != InputEvent.MOUSEDOWN:
            return
        p = event.pos
        if self.cam_smooth_rect.collidepoint(p):
            self.cam_smooth = not self.cam_smooth
            self._save_snake_setting("cam_smooth", self.cam_smooth)
            self.play_sound("select")
        elif self.cam_turn_rect.collidepoint(p):
            self.cam_turn_shake = not self.cam_turn_shake
            self._save_snake_setting("cam_turn_shake", self.cam_turn_shake)
            self.play_sound("select")
        elif self.cam_fov_minus.collidepoint(p):
            self._adjust_fov(-CAM_FOV_STEP)
        elif self.cam_fov_plus.collidepoint(p):
            self._adjust_fov(+CAM_FOV_STEP)
        elif self.cam_height_minus.collidepoint(p):
            self._adjust_height(-CAM_H_STEP)
        elif self.cam_height_plus.collidepoint(p):
            self._adjust_height(+CAM_H_STEP)
        elif self.cam_back_rect.collidepoint(p):
            self.state = SETUP
            self.play_sound("click")

    def _handle_setup_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Left", "a", "A"):
                self._cycle_mode(-1)
            elif event.key in ("Right", "d", "D"):
                self._cycle_mode(+1)
            elif event.key in ("1", "2", "3", "4", "5", "6"):
                allowed = self._allowed_modes()
                idx = int(event.key) - 1
                if idx < len(allowed):
                    self.mode_index = allowed[idx]
                    self._save_snake_setting("mode", self.mode_key)
                    self.play_sound("click")
            elif event.key in ("v", "V"):
                self._toggle_view()
            elif event.key in ("w", "W"):
                self._toggle_setting("wrap")
            elif event.key in ("b", "B"):
                self._toggle_setting("bonus")
            elif event.key in ("f", "F"):
                self._cycle_apples()
            elif event.key in ("h", "H"):
                self._toggle_hardcore()
            elif event.key in ("c", "C"):
                self._open_personalize()
            elif event.key in ("k", "K"):
                if self.view3d_active:
                    self._open_cam3d()
            elif event.key in ("Return", "space"):
                self.play_sound("click")
                self._start_play()
        elif event.kind == InputEvent.MOUSEDOWN:
            p = event.pos
            if self.brush_rect.collidepoint(p):
                self._open_personalize()
            elif self.mode_left.collidepoint(p):
                self._cycle_mode(-1)
            elif self.mode_right.collidepoint(p):
                self._cycle_mode(+1)
            elif self.mode_panel.collidepoint(p):
                self._cycle_mode(+1)
            elif self.view_rect.collidepoint(p):
                self._toggle_view()
            elif self.wrap_rect.collidepoint(p):
                if self.view3d_active:
                    self._open_cam3d()     # in 3D ist die Zeile der Kamera-Knopf
                else:
                    self._toggle_setting("wrap")
            elif self.bonus_rect.collidepoint(p):
                self._toggle_setting("bonus")
            elif self.apples_rect.collidepoint(p):
                if self.competitive:
                    self._toggle_hardcore()   # im Competitive ist die Zeile der HARDCORE-Schalter
                else:
                    self._cycle_apples()
            elif self.start_rect.collidepoint(p):
                self.play_sound("click")
                self._start_play()

    # ===================================================== Eingabe (Spiel)
    def handle_event(self, event):
        if self.state == PERSONALIZE:
            if self._ngb_menu is not None:
                self._ngb_menu.handle_event(event)
                if self._ngb_menu.done:
                    self._ngb_menu = None
                    self.state = SETUP     # zurück ins Setup; Kopffarbe ist übernommen
            return
        if self.state == CAM3D:
            self._handle_cam3d_event(event)
            return
        if self.state == SETUP:
            self._handle_setup_event(event)
            return

        # Während die Slot-Machine läuft, ist die Steuerung ausgesetzt.
        if self.slot is not None:
            return

        # Boost beenden, sobald die Taste losgelassen wird (gedrückt-halten-Logik)
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
        elif self.view3d_active:
            self._steer_3d(self.snakes[0], event.key)
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

    def _steer_3d(self, sn, key):
        """Relatives Lenken in der 3D-Ansicht: links/rechts drehen.

        Bis zu zwei Drehungen werden gepuffert und pro Spielschritt eine
        angewendet - so klappt auch eine schnelle Kehrtwende (2x drücken).
        """
        turn = None
        if self.is_action(key, "left") or key == "Left":
            turn = "L"
        elif self.is_action(key, "right") or key == "Right":
            turn = "R"
        if turn and len(sn.turn_queue) < 2:
            sn.turn_queue.append(turn)

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
        genug_äpfel = self.apples_bank >= req["apples"]
        genug_länge = len(sn.body) - req["length"] >= MIN_LENGTH
        return req, (genug_äpfel and genug_länge)

    def _try_prestige(self):
        if self.game_over or self.competitive:
            return                     # Competitive nutzt automatischen Level-Aufstieg
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
        self._update_particles3d(dt)
        self._update_food_anim(dt)
        self._update_float_texts(dt)
        self._update_banner(dt)
        if self._shake > 0.0:
            self._shake = max(0.0, self._shake - dt * 1.6)
        if self.state == PLAY and self.view3d_active:
            self._update_camera(dt)              # läuft auch nach Game Over (Orbit)
        if self.state != PLAY or self.game_over:
            return

        # Die Slot-Machine friert die Spielwelt ein, solange sie läuft.
        if self.slot is not None:
            self._update_slot(dt)
            return

        self._run_t += dt
        if self.competitive:
            self._update_competitive(dt)

        # Goldapfel-Lebensdauer (+ Funkeln in 3D)
        if self.golden is not None:
            self.golden_timer -= dt
            if self.golden_timer <= 0:
                self.golden = None
            elif self.view3d_active and random.random() < dt * 6:
                gx, gy = self.golden
                self.particles3d.append(
                    [gx + 0.5 + random.uniform(-0.2, 0.2), 0.55,
                     gy + 0.5 + random.uniform(-0.2, 0.2),
                     random.uniform(-0.4, 0.4), random.uniform(0.8, 1.8),
                     random.uniform(-0.4, 0.4), 0.55, COL_GOLD])

        # Zeitangriff-Countdown
        if self.mode_key == "timed":
            self.time_left -= dt
            if self.time_left <= 0:
                self.time_left = 0.0
                self._finish_timed()
                return

        # Ausdauer regenerieren (für nicht aktiv boostende Schlangen)
        for sn in self.snakes:
            if sn.alive and not sn.boost_on:
                sn.stamina = min(STAMINA_MAX, sn.stamina + STAMINA_REGEN * dt)

        # Schritte abarbeiten
        self._timer += dt
        guard = 0
        while self._timer >= self.interval and not self.game_over and self.slot is None:
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
        if self.game_over or self.slot is not None:
            return                     # ein blauer Apfel hat die Slot-Machine geöffnet
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
                    if self.hardcore_active:
                        self._hardcore_boost_cost(sn)   # Boost frisst Länge
                if sn.stamina <= 0:
                    sn.boost_on = False

    def _hardcore_boost_cost(self, sn):
        """HARDCORE: jeder Boost-Schritt kostet Länge.

        Zuerst wird ausstehendes Wachstum aufgezehrt, danach fällt der Schwanz
        Block für Block weg - aber nie unter die Mindestlänge. Ist die Schlange
        schon am Minimum, endet der Boost automatisch (es gibt nichts mehr zu
        verbrennen).
        """
        cost = HARDCORE_BOOST_LEN_COST
        if sn.grow > 0:                         # erst geplantes Wachstum "verbrennen"
            used = min(sn.grow, cost)
            sn.grow -= used
            cost -= used
        if cost > 0:
            schnitt = min(cost, len(sn.body) - MIN_LENGTH)
            if schnitt > 0:
                self._spawn_particles(sn.body[0], COL_HARDCORE, 3)
                del sn.body[:schnitt]           # Schwanz sofort kürzen
                sn.prev_body = list(sn.body)
            else:
                sn.boost_on = False             # Mindestlänge erreicht -> Boost aus

    def _advance(self, movers):
        """Bewegt die Schlangen in 'movers' um eine Zelle (mit voller Kollision)."""
        new_heads = {}
        for i in movers:
            sn = self.snakes[i]
            if not sn.alive:
                continue
            sn.prev_body = list(sn.body)         # für die 3D-Interpolation
            if self.view3d_active and sn.turn_queue:
                alt = sn.direction
                sn.direction = _rotate(sn.direction, sn.turn_queue.pop(0))
                sn.next_direction = sn.direction
                if self.cam_turn_shake and sn.direction != alt:
                    self._shake = max(self._shake, 0.22)   # Ruckeln beim Abbiegen
            else:
                sn.direction = sn.next_direction
            hx, hy = sn.body[-1]
            nx, ny = hx + sn.direction[0], hy + sn.direction[1]
            if (nx, ny) in self.portals:                 # Teleporter
                nx, ny = self.portals[(nx, ny)]
                nx += sn.direction[0]
                ny += sn.direction[1]
            if self.wrap_active:
                nx %= self.cols
                ny %= self.rows
            new_heads[i] = (nx, ny)

        tot = set()

        # Wandkollision (nur bei festen Wänden)
        if not self.wrap_active:
            for i, (nx, ny) in new_heads.items():
                if nx < 0 or nx >= self.cols or ny < 0 or ny >= self.rows:
                    tot.add(i)
        # Hindernisse (immer tödlich)
        for i, kopf in new_heads.items():
            if kopf in self.obstacles:
                tot.add(i)
        # Kopf-an-Kopf
        for i in new_heads:
            for j in new_heads:
                if i < j and new_heads[i] == new_heads[j]:
                    tot.add(i)
                    tot.add(j)
        # Körperkollision
        belegt = set()
        for i, sn in enumerate(self.snakes):
            if not sn.alive:
                continue
            if i in movers:
                wächst = (new_heads.get(i) in self.foods) \
                    or (self.golden is not None and new_heads.get(i) == self.golden) \
                    or (sn.grow > 0)
                körper = sn.body if wächst else sn.body[1:]
            else:
                körper = sn.body
            belegt |= set(körper)
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
                if self.view3d_active:
                    self._shake = 0.6            # Kamera-Wackler beim Crash
                continue
            kopf = new_heads[i]
            sn.body.append(kopf)
            if kopf in self.foods:
                self.foods.discard(kopf)
                self._food_anim.pop(kopf, None)
                if not self.view3d_active:
                    self._spawn_eat_fx(kopf)
                self._eat_food(sn)
                ate = True
            elif self.golden is not None and kopf == self.golden:
                self._eat_golden(sn)
                ate_gold = True
            elif kopf in self.specials:
                self._eat_special(sn, kopf)
            if sn.grow > 0:
                sn.grow -= 1
            else:
                sn.body.pop(0)

        # Spezialäpfel wirken erst nach dem Schritt (verändern u.U. die Länge)
        if self._purple_pending is not None:
            self._apply_purple(self._purple_pending)
            self._purple_pending = None
        if self._slot_pending is not None:
            self._open_slot(self._slot_pending)
            self._slot_pending = None

        if ate:
            self.play_sound("eat")
            self._place_food()
            if self.golden is None and random.random() < GOLDEN_CHANCE:
                self._place_golden()
            if self.competitive:
                self._maybe_spawn_special()
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
            sn.score += gain * 10 * self._score_multiplier()
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
            sn.score += 50 * self._score_multiplier()
        self._spawn_particles(sn.body[-1], COL_GOLD, 16)
        self.play_sound("point")
        self.rumble(80)

    def _score_multiplier(self):
        """Punkte-Multiplikator: Competitive nutzt das Level, sonst Prestige."""
        if self.competitive:
            return competitive.score_multiplier(self.comp_level)
        return prestige.score_multiplier(self.prestige)

    # ----- Competitive: Spezialäpfel-Wirkung ----------------------------
    def _eat_special(self, sn, cell):
        """Blauer Apfel -> Slot-Machine, lila Apfel -> Längen-Wette."""
        typ = self.specials.pop(cell, {}).get("type")
        if typ == "blue":
            self._slot_pending = sn
        elif typ == "purple":
            self._purple_pending = sn

    def _snake_size(self, sn):
        """Wahre 'Größe' einer Schlange als Kommazahl.

        = sichtbare Länge + ausstehendes Wachstum + Nachkomma-Rest aus dem
        Gambling. (len(body) + grow bleibt bei jeder Bewegung erhalten und
        ändert sich nur durch Essen/Wetten - daher eine stabile Kennzahl.)
        """
        return len(sn.body) + sn.grow + sn.size_frac

    def _set_snake_size(self, sn, size):
        """Setzt die Größe (Kommazahl) und passt Körper/Wachstum entsprechend an.

        Der ganzzahlige Teil steuert die physische Länge (über Wachstum bzw.
        Schwanz-Kürzen), der Nachkomma-Rest wird in ``size_frac`` gemerkt, damit
        weitere Wetten exakt darauf aufbauen. Nie unter die Mindestlänge.
        """
        size = max(MIN_LENGTH, size)
        target_len = int(math.floor(size + 1e-9))
        sn.size_frac = size - target_len
        delta = target_len - (len(sn.body) + sn.grow)
        if delta > 0:
            sn.grow += delta                   # wächst über die nächsten Schritte
        elif delta < 0:
            need = -delta
            used = min(sn.grow, need)           # erst ausstehendes Wachstum abbauen
            sn.grow -= used
            need -= used
            if need > 0:                        # ... dann den Schwanz kürzen
                schnitt = min(need, len(sn.body) - MIN_LENGTH)
                if schnitt > 0:
                    del sn.body[:schnitt]
                    sn.prev_body = list(sn.body)

    def _apply_purple(self, sn):
        """Lila Apfel (Gambling): ein Anteil der Größe wird eingesetzt und mit einem
        zufälligen Faktor multipliziert; der Rest bleibt sicher.

        Normal: 50 % Einsatz, Faktor x0.5..x1.5.
        HARDCORE: 75-90 % Einsatz, Faktor x0.25..x2.25 (riskanter).
            neue Größe = Größe*(1-p) + Größe*p*Faktor
        HARDCORE-Beispiel: Größe 20, p=0.8, Faktor 0.25 -> 20*0.2 + 20*0.8*0.25 = 8.
        Die Größe wird als Kommazahl weitergeführt (Anzeige oben links), damit
        weitere Wetten exakt darauf aufbauen.
        """
        hc = self.hardcore_active
        stake = competitive.purple_stake(hc)     # 50 % bzw. 75-90 % (HARDCORE)
        factor = competitive.purple_factor(hc)   # x0.5..x1.5 bzw. x0.25..x2.25 (HARDCORE)
        size = self._snake_size(sn)
        new_size = max(MIN_LENGTH, size * (1.0 - stake) + size * stake * factor)
        self._set_snake_size(sn, new_size)
        gewonnen = new_size >= size
        col = COL_WLS_ON if gewonnen else COL_FOOD
        self.play_sound("powerup" if gewonnen else "hit")
        self.rumble(80)
        self._spawn_particles(sn.body[-1], COL_PURPLE, 14)
        self._add_float_text(sn.body[-1], f"x{factor:g}", col)
        # Einsatz + Multiplikator gut sichtbar oben mittig einblenden
        self._show_banner(f"{stake*100:.0f}% ×{factor:g}", col,
                          i18n.t("snake.purple_banner"))

    def _show_banner(self, text, color, sub=None):
        """Blendet eine große Einblendung oben mittig ein (Auto-Ausblendung)."""
        self._banner = dict(text=text, color=color, sub=sub, t=BANNER_TIME)

    def _update_banner(self, dt):
        if self._banner is not None:
            self._banner["t"] -= dt
            if self._banner["t"] <= 0:
                self._banner = None

    # ----- Competitive: Slot-Machine ------------------------------------
    def _open_slot(self, sn):
        """Öffnet den Spielautomaten (friert die Welt ein, bis er ausläuft)."""
        reels = competitive.spin_reels()
        mult, result = competitive.slot_outcome(reels)
        self.slot = dict(snake=sn, reels=reels, mult=mult, result=result,
                         stake=max(2, len(sn.body) // 4), stop=list(SLOT_REEL_STOPS),
                         t=0.0, applied=False)
        self.play_sound("point")

    def _update_slot(self, dt):
        """Animiert die Walzen; wendet am Ende den Gewinn/Verlust an."""
        sl = self.slot
        sl["t"] += dt
        for k, ts in enumerate(sl["stop"]):
            if sl["t"] >= ts and not sl.get(f"snd{k}"):
                sl[f"snd{k}"] = True
                self.play_sound("lock")
        if sl["t"] >= sl["stop"][-1] and not sl["applied"]:
            sl["applied"] = True
            self._apply_slot(sl)
        if sl["t"] >= sl["stop"][-1] + SLOT_SHOW:
            self.slot = None

    def _apply_slot(self, sl):
        """Verrechnet das Slot-Ergebnis: Längeneinsatz + zeitweise mehr Äpfel."""
        sn = sl["snake"]
        netto = int(round(sl["stake"] * (sl["mult"] - 1.0)))
        if netto > 0:
            sn.grow += netto
        elif netto < 0:
            schnitt = min(-netto, len(sn.body) - MIN_LENGTH)
            if schnitt > 0:
                del sn.body[:schnitt]
                sn.prev_body = list(sn.body)
        # Der Multiplikator lässt für kurze Zeit zusätzliche Äpfel spawnen.
        self.spawn_bonus = max(self.spawn_bonus, int(round(sl["mult"])))
        self.spawn_bonus_t = SPAWN_BONUS_TIME
        self._place_food()
        if sl["result"] == "jackpot":
            self.play_sound("win"); self.rumble(160)
        elif sl["result"] == "pair":
            self.play_sound("powerup"); self.rumble(80)
        else:
            self.play_sound("hit")
        self._add_float_text(sn.body[-1], f"x{sl['mult']:g}", COL_BLUE)

    # ----- Aufsteigende Hinweistexte ------------------------------------
    def _add_float_text(self, cell, text, color):
        self.float_texts.append(dict(x=cell[0] * CELL + CELL / 2, y=cell[1] * CELL,
                                     text=text, color=color, t=1.1))

    def _update_float_texts(self, dt):
        rest = []
        for ft in self.float_texts:
            ft["y"] -= dt * 26
            ft["t"] -= dt
            if ft["t"] > 0:
                rest.append(ft)
        self.float_texts = rest

    def _update_competitive(self, dt):
        """Level aus gesammelten Äpfeln, Slot-Bonus und Spezialapfel-Lebensdauer."""
        lvl = competitive.level_for_apples(self.apples_total)
        if lvl > self.comp_level:
            self.comp_level = lvl
            self.play_sound("level")
            self.rumble(90)
            self._add_float_text(self.snakes[0].body[-1],
                                 i18n.t("snake.comp.levelup", n=lvl), COL_MULT)
            self._place_food()                 # das neue Level legt einen Apfel nach
        if self.spawn_bonus_t > 0:
            self.spawn_bonus_t = max(0.0, self.spawn_bonus_t - dt)
            if self.spawn_bonus_t == 0:
                self.spawn_bonus = 0
        for cell in list(self.specials):
            self.specials[cell]["timer"] -= dt
            if self.specials[cell]["timer"] <= 0:
                del self.specials[cell]

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
        if self.view3d_active:
            # 3D-Funken: fliegen aus der Zelle hoch und prallen am Boden ab
            wx, wz = cell[0] + 0.5, cell[1] + 0.5
            for _ in range(n):
                ang = random.uniform(0, math.tau)
                spd = random.uniform(1.0, 4.0)
                self.particles3d.append(
                    [wx, random.uniform(0.2, 0.6), wz,
                     math.cos(ang) * spd, random.uniform(1.0, 4.5),
                     math.sin(ang) * spd, random.uniform(0.35, 0.7), color])
            return
        cx = cell[0] * CELL + CELL / 2
        cy = cell[1] * CELL + CELL / 2
        for _ in range(n):
            ang = random.uniform(0, math.tau)
            spd = random.uniform(30, 130)
            self.particles.append([cx, cy, math.cos(ang) * spd, math.sin(ang) * spd,
                                   random.uniform(0.25, 0.55), color])

    def _update_particles3d(self, dt):
        rest = []
        for p in self.particles3d:
            p[0] += p[3] * dt
            p[1] += p[4] * dt
            p[2] += p[5] * dt
            p[4] -= 9.0 * dt                     # Schwerkraft
            if p[1] < 0.04:                      # am Boden abprallen
                p[1] = 0.04
                p[4] *= -0.4
            p[6] -= dt
            if p[6] > 0:
                rest.append(p)
        self.particles3d = rest

    # ----- 3D-Kamera ------------------------------------------------------
    def _update_camera(self, dt):
        """Verfolgerkamera: schwebt geglättet hinter dem Kopf.

        Nach dem Game Over kreist sie stattdessen langsam um die Schlange.
        """
        sn = self.snakes[0]
        cells = self._interp_cells(sn)
        hx, hz = cells[-1]
        head = (hx + 0.5, 0.0, hz + 0.5)

        if self.game_over:
            self._orbit_a += dt * 0.55
            tx, tz = math.sin(self._orbit_a), math.cos(self._orbit_a)
            back = CAM_BACK + 1.6
            look = (head[0], 0.25, head[2])
        else:
            tx, tz = float(sn.direction[0]), float(sn.direction[1])
            back, look = CAM_BACK, None

        # Blickrichtung weich nachziehen. Smooth-Shake = sanftere Glättung, damit
        # die Kamera beim Bewegen/Drehen deutlich weniger ruckelt.
        dir_rate = 2.6 if self.cam_smooth else 4.5
        pos_rate = CAM_SMOOTH * (0.55 if self.cam_smooth else 1.0)
        k = min(1.0, dt * dir_rate)
        fx = self._cam_dir[0] + (tx - self._cam_dir[0]) * k
        fz = self._cam_dir[1] + (tz - self._cam_dir[1]) * k
        ln = math.hypot(fx, fz)
        if ln > 1e-6:
            self._cam_dir = (fx / ln, fz / ln)
        fx, fz = self._cam_dir

        tgt_pos = (head[0] - fx * back, self.cam_height, head[2] - fz * back)
        tgt_look = look or (head[0] + fx * CAM_AHEAD, CAM_LOOK_H,
                            head[2] + fz * CAM_AHEAD)
        kp = min(1.0, dt * pos_rate)
        self._cam_pos = tuple(a + (b - a) * kp
                              for a, b in zip(self._cam_pos, tgt_pos))
        self._cam_look = tuple(a + (b - a) * kp
                               for a, b in zip(self._cam_look, tgt_look))

        # Sichtfeld: Grundwert aus den Optionen, beim Boost weitwinkliger.
        boost = sn.alive and sn.boost_on and sn.stamina > 0
        want = self.cam_fov - (FOV_BOOST_DELTA if boost else 0.0)
        self._fov_mul += (want - self._fov_mul) * min(1.0, dt * 6.0)

    def _interp_cells(self, sn):
        """Körperzellen, zwischen letztem und aktuellem Schritt interpoliert.

        Liefert Fließkomma-Zellkoordinaten in Körperreihenfolge (Kopf am Ende);
        damit gleitet die Schlange in 3D, statt von Zelle zu Zelle zu springen.
        """
        if self.game_over or not sn.alive:
            frac = 1.0
        else:
            frac = max(0.0, min(1.0, self._timer / max(1e-6, self.interval)))
        out = []
        n, m = len(sn.body), len(sn.prev_body)
        for k, cur in enumerate(sn.body):
            pi = m - (n - k)                 # gleiches Segment, vom Kopf her ausgerichtet
            prev = sn.prev_body[pi] if 0 <= pi < m else cur
            if abs(cur[0] - prev[0]) + abs(cur[1] - prev[1]) > 1.5:
                prev = cur                   # Teleport/Spawn: nicht interpolieren
            out.append((prev[0] + (cur[0] - prev[0]) * frac,
                        prev[1] + (cur[1] - prev[1]) * frac))
        return out

    def _update_particles(self, dt):
        rest = []
        for p in self.particles:
            p[0] += p[2] * dt
            p[1] += p[3] * dt
            p[4] -= dt
            if p[4] > 0:
                rest.append(p)
        self.particles = rest

    def _update_food_anim(self, dt):
        """Führt die 2D-Apfel-Animationen weiter: Einblenden + Iss-Effekte."""
        for c in list(self._food_anim):
            if c in self.foods:
                self._food_anim[c] = min(FOOD_SPAWN_ANIM, self._food_anim[c] + dt)
            else:
                del self._food_anim[c]         # Apfel weg -> Einblendung verwerfen
        rest = []
        for e in self._eat_fx:
            e["t"] -= dt
            if e["t"] > 0:
                rest.append(e)
        self._eat_fx = rest

    def _spawn_eat_fx(self, cell):
        """Startet den 'weggeknabbert'-Effekt für einen gegessenen Apfel (2D)."""
        self._eat_fx.append(dict(cell=cell, t=FOOD_EAT_ANIM))
        self._spawn_particles(cell, COL_FOOD, 8)   # rote Krümel fliegen weg

    # ===================================================== Zeichnen
    def draw(self):
        if self.state == PERSONALIZE:
            if self._ngb_menu is not None:
                self._ngb_menu.draw(self.surface)
            return
        if self.state == CAM3D:
            self._draw_cam3d()
            return
        if self.state == SETUP:
            self._draw_setup()
            return

        if self.view3d_active:
            self._draw_world_3d()
        else:
            self._draw_world_2d()

        self._draw_hud()
        self._draw_banner()

        if self.slot is not None:
            self._draw_slot()

        if self.game_over:
            self._draw_game_over()

    def _draw_grid(self, s):
        """Zeichnet das Spielfeld-Gitter - normal oder als NGB-Raster-Overlay.

        Das Raster-Overlay (Schachbrett aus einer Farbreihenfolge) markiert die
        horizontalen und vertikalen Linien, damit man Reihen/Spalten auch auf
        grossen Feldern von Weitem erkennt. Es ist statisch und wird gecacht.
        """
        s.fill(COL_BG)
        seq = ngb.grid_sequence()
        if not seq:
            for x in range(0, self.cols * CELL, CELL):
                pygame.draw.line(s, COL_GRID, (x, 0), (x, self.rows * CELL))
            for y in range(0, self.rows * CELL, CELL):
                pygame.draw.line(s, COL_GRID, (0, y), (self.cols * CELL, y))
            return
        key = (self.cols, self.rows, tuple(tuple(c) for c in seq))
        if self._grid_key != key:
            self._grid_key = key
            self._grid_surf = self._build_grid_overlay(seq)
        s.blit(self._grid_surf, (0, 0))

    @staticmethod
    def _brighten(col, amt):
        return tuple(min(255, c + amt) for c in col)

    @staticmethod
    def _col_label(i):
        """Spaltenname im Tabellen-Stil: a, b, ..., z, aa, ab, ... (beliebig breit)."""
        name = ""
        i += 1
        while i > 0:
            i, r = divmod(i - 1, 26)
            name = chr(ord("a") + r) + name
        return name

    def _faded_text(self, text, color, alpha):
        """Antialiastes Text-Bild mit skalierter Deckkraft (für Watermark-Labels)."""
        img = self._grid_font.render(str(text), True, color)
        img.fill((255, 255, 255, alpha), special_flags=pygame.BLEND_RGBA_MULT)
        return img

    def _build_grid_overlay(self, seq):
        """Rendert den Koordinaten-Wegweiser einmalig auf eine Surface (Cache).

        Reihen (horizontal) tragen **Nummern** (1..N) am linken und rechten Rand;
        Spalten (vertikal) tragen **Buchstaben** (a, b, ...) oben und unten.
        Zusätzlich färbt die gewählte Farbreihenfolge
        die Reihen als dezente Bänder, damit man eine Reihe leicht quer verfolgt.
        So sieht man z.B. bei Position 8z, dass der Apfel bei 8a in derselben
        Reihe (8) liegt.
        """
        n = len(seq)
        w, h = self.cols * CELL, self.rows * CELL
        surf = pygame.Surface((w, h))
        surf.fill(COL_BG)

        # 1) Reihen-Bänder (horizontal, sehr dezent zum Hintergrund gemischt)
        for gy in range(self.rows):
            band = seq[gy % n]
            tint = tuple((c + COL_BG[i] * 3) // 4 for i, c in enumerate(band))
            surf.fill(tint, (0, gy * CELL, w, CELL))

        # 2) dezente Gitterlinien
        line = tuple((c + COL_BG[i]) // 2 for i, c in enumerate(seq[0]))
        for gx in range(self.cols + 1):
            pygame.draw.line(surf, line, (gx * CELL, 0), (gx * CELL, h))
        for gy in range(self.rows + 1):
            pygame.draw.line(surf, line, (0, gy * CELL), (w, gy * CELL))

        # 3) Reihen-Nummern nur an den Rändern (links + rechts)
        for gy in range(self.rows):
            num = str(gy + 1)
            col = self._brighten(seq[gy % n], 110)
            edge = self._faded_text(num, col, 170)
            cy = gy * CELL + CELL // 2
            surf.blit(edge, edge.get_rect(midleft=(3, cy)))
            surf.blit(edge, edge.get_rect(midright=(w - 3, cy)))

        # 4) Spalten-Buchstaben (oben + unten)
        for gx in range(self.cols):
            lab = self._faded_text(self._col_label(gx), (205, 212, 226), 155)
            cx = gx * CELL + CELL // 2
            surf.blit(lab, lab.get_rect(midtop=(cx, 1)))
            surf.blit(lab, lab.get_rect(midbottom=(cx, h - 1)))
        return surf

    def _draw_world_2d(self):
        s = self.surface
        self._draw_grid(s)

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

        # Futter (alle liegenden Äpfel) - blenden mit leichtem Überschwingen ein
        for cell in self.foods:
            p = min(1.0, self._food_anim.get(cell, FOOD_SPAWN_ANIM) / FOOD_SPAWN_ANIM)
            self._draw_apple(s, cell, _ease_out_back(p))
            if p < 1.0:                        # kurzer, aufblitzender Ring beim Morphen
                self._draw_fx_ring(s, cell, int(CELL * 0.28 + CELL * 0.5 * p),
                                   (255, 150, 150), int(150 * (1 - p)))
        # Goldapfel (pulsiert + blinkt wenn er gleich verschwindet)
        if self.golden is not None:
            gx, gy = self.golden
            blink = self.golden_timer > 1.5 or int(self.anim_t * 8) % 2 == 0
            if blink:
                cx, cy = gx * CELL + CELL // 2, gy * CELL + CELL // 2
                r = CELL // 2 - 1 + int(1.5 * math.sin(self.anim_t * 8))
                pygame.draw.circle(s, COL_GOLD, (cx, cy), r)
                pygame.draw.circle(s, (255, 245, 200), (cx - 2, cy - 2), 2)

        # Competitive-Spezialäpfel (blau = Slot-Machine, lila = Längen-Wette)
        self._draw_specials(s)

        # Partikel
        for p in self.particles:
            a = max(0, min(255, int(255 * (p[4] / 0.55))))
            surf = pygame.Surface((6, 6), pygame.SRCALPHA)
            pygame.draw.circle(surf, (*p[5], a), (3, 3), 3)
            s.blit(surf, (p[0] - 3, p[1] - 3))

        # Schlangen
        for idx, sn in enumerate(self.snakes):
            self._draw_snake(s, sn, idx)

        # Iss-Effekte ZULETZT (über dem Schlangenkopf, der auf dem Apfel liegt)
        self._draw_eat_fx(s)

        # Aufsteigende Hinweistexte (Level-Up, Slot- und Wett-Ergebnisse)
        self._draw_float_texts(s)

    def _draw_specials(self, s):
        """Zeichnet die Competitive-Spezialäpfel als pulsierende Edelsteine."""
        for cell, info in self.specials.items():
            if info["timer"] < 1.5 and int(self.anim_t * 8) % 2 == 0:
                continue                       # blinkt kurz vor dem Verschwinden
            blue = info["type"] == "blue"
            col = COL_BLUE if blue else COL_PURPLE
            inner = tuple(min(255, c + 70) for c in col)
            cx = cell[0] * CELL + CELL // 2
            cy = cell[1] * CELL + CELL // 2
            r = CELL // 2 - 2 + int(1.5 * math.sin(self.anim_t * 6 + cell[0]))
            pts = [(cx, cy - r), (cx + r, cy), (cx, cy + r), (cx - r, cy)]   # Raute
            pygame.draw.polygon(s, col, pts)
            pygame.draw.polygon(s, inner, pts, 1)
            glyph = "$" if blue else "±"       # Slot-Machine bzw. mal/geteilt
            g = self._tiny.render(glyph, True, (20, 20, 30))
            s.blit(g, g.get_rect(center=(cx, cy)))

    def _draw_float_texts(self, s):
        for ft in self.float_texts:
            img = self._small.render(ft["text"], True, ft["color"])
            img.set_alpha(max(0, min(255, int(255 * (ft["t"] / 1.1)))))
            s.blit(img, img.get_rect(center=(int(ft["x"]), int(ft["y"]))))

    def _draw_banner(self):
        """Große Einblendung oben mittig (z.B. Multiplikator des lila Apfels).

        Sichtbarkeit, Größe und Deckkraft kommen aus der NGB-Personalisierung.
        """
        b = self._banner
        if not b:
            return
        cfg = ngb.get_banner()
        if not cfg["on"]:
            return
        appear = min(1.0, (BANNER_TIME - b["t"]) / 0.18)     # Einblenden (Pop)
        fade = min(1.0, max(0.0, b["t"] / 0.5))              # Ausblenden am Ende
        a = int(255 * min(appear, fade) * cfg["opacity"])
        if a <= 0:
            return
        big = self.big_font.render(b["text"], True, b["color"])
        sub = self._small.render(b["sub"], True, COL_DIM) if b.get("sub") else None
        pad = 18
        tw = max(big.get_width(), sub.get_width() if sub else 0)
        th = big.get_height() + (sub.get_height() + 4 if sub else 0)
        pw, ph = tw + pad * 2, th + pad
        panel = pygame.Surface((pw, ph), pygame.SRCALPHA)
        pygame.draw.rect(panel, (18, 20, 32, 215), panel.get_rect(), border_radius=12)
        pygame.draw.rect(panel, (*b["color"], 255), panel.get_rect(), 2, border_radius=12)
        y = pad // 2
        if sub:
            panel.blit(sub, sub.get_rect(midtop=(pw // 2, 5)))
            y = 5 + sub.get_height() + 2
        panel.blit(big, big.get_rect(midtop=(pw // 2, y)))
        # Pop beim Erscheinen + eingestellte Größe (kleiner/größer)
        scale = (0.82 + 0.18 * appear) * cfg["size"]
        if abs(scale - 1.0) > 0.01:
            panel = pygame.transform.rotozoom(panel, 0, scale)
        panel.fill((255, 255, 255, a), special_flags=pygame.BLEND_RGBA_MULT)
        top = 44 + int(-14 * (1 - appear))
        self.surface.blit(panel, panel.get_rect(midtop=(self.width // 2, top)))

    def _draw_apple(self, s, cell, scale, alpha=255):
        """Zeichnet einen Apfel skaliert um sein Zentrum (0..~1.1 = eingeblendet).

        Kleiner Apfel wirkt runder, voller Apfel wird zum abgerundeten Quadrat -
        so 'morpht' er beim Einblenden sanft in Form.
        """
        if scale <= 0.03:
            return
        size = (CELL - 4) * scale
        cx = cell[0] * CELL + CELL / 2
        cy = cell[1] * CELL + CELL / 2
        rad = int(min(size / 2, 6 + (size / 2) * (1 - scale)))
        if alpha >= 255:
            rect = pygame.Rect(0, 0, int(size), int(size))
            rect.center = (int(cx), int(cy))
            pygame.draw.rect(s, COL_FOOD, rect, border_radius=rad)
            hl = max(1, int(2.2 * scale))
            pygame.draw.circle(s, (255, 170, 170),
                               (int(cx - size * 0.14), int(cy - size * 0.14)), hl)
        else:
            d = int(size) + 2
            surf = pygame.Surface((d, d), pygame.SRCALPHA)
            pygame.draw.rect(surf, (*COL_FOOD, alpha),
                             pygame.Rect(1, 1, int(size), int(size)), border_radius=rad)
            s.blit(surf, (int(cx - d / 2), int(cy - d / 2)))

    def _draw_fx_ring(self, s, cell, r, color, a):
        """Ausdehnender, verblassender Ring um eine Zelle (Spawn-/Iss-Effekt)."""
        if a <= 4 or r <= 0:
            return
        d = r * 2 + 4
        rs = pygame.Surface((d, d), pygame.SRCALPHA)
        pygame.draw.circle(rs, (*color, a), (d // 2, d // 2), r, 2)
        cx = cell[0] * CELL + CELL // 2
        cy = cell[1] * CELL + CELL // 2
        s.blit(rs, (int(cx - d / 2), int(cy - d / 2)))

    def _draw_eat_fx(self, s):
        """Zeichnet die 'weggeknabbert'-Effekte: schrumpfender Apfel + Ring."""
        for e in self._eat_fx:
            p = 1.0 - max(0.0, e["t"] / FOOD_EAT_ANIM)     # 0 -> 1
            self._draw_apple(s, e["cell"], max(0.0, (1.0 - p) * 0.9),
                             alpha=int(210 * (1.0 - p)))
            self._draw_fx_ring(s, e["cell"], int(CELL * 0.32 + CELL * 0.5 * p),
                               COL_FOOD, int(150 * (1.0 - p)))

    def _head_color(self, idx):
        """Kopffarbe einer Schlange. Spieler 1 nutzt die NGB-Personalisierung."""
        if idx == 0:
            return ngb.head_color()
        return SNAKE_COLORS[idx % len(SNAKE_COLORS)][1]

    def _draw_snake(self, s, sn, idx):
        körper, kopf = SNAKE_COLORS[idx % len(SNAKE_COLORS)]
        kopf = self._head_color(idx)
        if not sn.alive:
            körper = tuple(c // 2 for c in körper)
            kopf = körper

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
                              for k, c in zip(kopf, körper))
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

    # ===================================================== 3D-Renderer
    # Software-3D ohne OpenGL: Punkte werden in den Kameraraum transformiert
    # (Basisvektoren rechts/oben/vorwärts), an der Nahebene geclippt und
    # perspektivisch projiziert. Flächen sammeln wir in einer Liste und
    # zeichnen sie nach Tiefe sortiert (Painter-Algorithmus, ferne zuerst).

    def _draw_world_3d(self):
        s = self.surface

        # Nach dem Game Over ruft der Haupt-Loop update() nicht mehr auf -
        # Orbit-Kamera und Partikel animieren wir deshalb hier weiter.
        if self.game_over:
            self._tick_gameover_anim()

        # Projektionsparameter (inkl. Kamera-Shake nach einem Crash)
        self._scx = self.width / 2
        self._scy = self.height * 0.5
        if self._shake > 0:
            amp = 16 * self._shake * (0.5 if self.cam_smooth else 1.0)
            self._scx += random.uniform(-amp, amp)
            self._scy += random.uniform(-amp, amp)
        self._f = self.height * self._fov_mul
        self._basis = self._view_basis()

        self._draw_sky(s)
        self._draw_stars3d(s)
        self._draw_floor3d(s)

        items = []                       # (tiefe, punkte, farbe, kontur)
        self._add_border_walls(items)
        for (ox, oz) in self.obstacles:
            self._add_box(items, ox, oz, 0.94, 1.05, COL_WALL, (95, 104, 128))
        for f in self.foods:
            r = 0.30 + 0.05 * math.sin(self.anim_t * 5 + f[0] + f[1])
            self._add_octa(items, f, r, COL_FOOD)
        if self.golden is not None and (self.golden_timer > 1.5
                                        or int(self.anim_t * 8) % 2 == 0):
            r = 0.38 + 0.06 * math.sin(self.anim_t * 8)
            self._add_octa(items, self.golden, r, COL_GOLD)
        for idx, sn in enumerate(self.snakes):
            self._add_snake3d(items, sn, idx)

        items.sort(key=lambda it: -it[0])
        for _, pts, col, outline in items:
            pygame.draw.polygon(s, col, pts)
            if outline:
                pygame.draw.polygon(s, outline, pts, 1)

        self._draw_eyes3d(s)
        self._draw_boost_glow3d(s)
        self._draw_particles3d_pass(s)

    def _tick_gameover_anim(self):
        """Animiert Orbit-Kamera/Partikel nach dem Game Over weiter.

        update() wird vom Haupt-Loop nur bis zum Game Over aufgerufen, draw()
        aber weiterhin jeden Frame - also messen wir hier die echte Zeit.
        """
        now = time.monotonic()
        last = self._go_last if self._go_last is not None else now
        self._go_last = now
        dt = min(0.05, max(0.0, now - last))
        if dt <= 0:
            return
        self.anim_t += dt
        self._update_particles3d(dt)
        if self._shake > 0.0:
            self._shake = max(0.0, self._shake - dt * 1.6)
        self._update_camera(dt)

    # ----- Kamera / Projektion -------------------------------------------
    def _view_basis(self):
        """Basisvektoren der Kamera: (rechts, oben, vorwärts)."""
        cx, cy, cz = self._cam_pos
        fx = self._cam_look[0] - cx
        fy = self._cam_look[1] - cy
        fz = self._cam_look[2] - cz
        fl = math.sqrt(fx * fx + fy * fy + fz * fz) or 1.0
        f = (fx / fl, fy / fl, fz / fl)
        rl = math.hypot(f[2], f[0]) or 1.0
        r = (-f[2] / rl, 0.0, f[0] / rl)         # f x (0,1,0), normiert
        u = (r[1] * f[2] - r[2] * f[1],          # r x f
             r[2] * f[0] - r[0] * f[2],
             r[0] * f[1] - r[1] * f[0])
        return r, u, f

    def _to_cam(self, p):
        """Weltpunkt -> Kameraraum (x rechts, y oben, z Tiefe)."""
        r, u, f = self._basis
        dx = p[0] - self._cam_pos[0]
        dy = p[1] - self._cam_pos[1]
        dz = p[2] - self._cam_pos[2]
        return (dx * r[0] + dz * r[2],           # r[1] ist immer 0
                dx * u[0] + dy * u[1] + dz * u[2],
                dx * f[0] + dy * f[1] + dz * f[2])

    def _proj(self, c):
        """Kameraraum -> Bildschirmpixel (perspektivisch)."""
        k = self._f / c[2]
        return (self._scx + c[0] * k, self._scy - c[1] * k)

    @staticmethod
    def _clip_near(pts):
        """Schneidet ein Polygon (Kameraraum) an der Nahebene z = NEAR."""
        out = []
        n = len(pts)
        for i in range(n):
            a, b = pts[i], pts[(i + 1) % n]
            a_in, b_in = a[2] >= NEAR, b[2] >= NEAR
            if a_in:
                out.append(a)
            if a_in != b_in:
                t = (NEAR - a[2]) / (b[2] - a[2])
                out.append((a[0] + (b[0] - a[0]) * t,
                            a[1] + (b[1] - a[1]) * t, NEAR))
        return out

    @staticmethod
    def _shade_col(col, k):
        return (min(255, int(col[0] * k)), min(255, int(col[1] * k)),
                min(255, int(col[2] * k)))

    @staticmethod
    def _fog_color(col, depth):
        """Blendet eine Farbe mit der Entfernung in den Nebel aus."""
        t = (depth - FOG_START) / (FOG_END - FOG_START)
        if t <= 0:
            return col
        t = min(1.0, t)
        return (int(col[0] + (COL_FOG[0] - col[0]) * t),
                int(col[1] + (COL_FOG[1] - col[1]) * t),
                int(col[2] + (COL_FOG[2] - col[2]) * t))

    def _add_poly(self, items, world_pts, color, shade=1.0, outline=None):
        """Transformiert, clippt und projiziert eine Fläche in die Zeichenliste."""
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
        if all(p[0] < -40 for p in pts) or all(p[0] > self.width + 40 for p in pts) \
                or all(p[1] < -40 for p in pts) or all(p[1] > self.height + 40 for p in pts):
            return
        col = self._fog_color(self._shade_col(color, shade), depth)
        # Konturen nur in der Nähe (im Nebel wirken sie unruhig)
        items.append((depth, pts, col, outline if depth < FOG_START + 4 else None))

    # ----- Szenen-Bausteine ------------------------------------------------
    def _add_box(self, items, gx, gz, w, h, color, outline=None):
        """Quader auf dem Boden der Zelle (gx, gz); nur sichtbare Seiten."""
        cx, cz = gx + 0.5, gz + 0.5
        x0, x1 = cx - w / 2, cx + w / 2
        z0, z1 = cz - w / 2, cz + w / 2
        px, py, pz = self._cam_pos
        if py > h:                                   # Deckel
            self._add_poly(items, ((x0, h, z0), (x1, h, z0), (x1, h, z1), (x0, h, z1)),
                           color, 1.0, outline)
        if px < x0:
            self._add_poly(items, ((x0, 0, z0), (x0, 0, z1), (x0, h, z1), (x0, h, z0)),
                           color, 0.62, outline)
        elif px > x1:
            self._add_poly(items, ((x1, 0, z0), (x1, 0, z1), (x1, h, z1), (x1, h, z0)),
                           color, 0.62, outline)
        if pz < z0:
            self._add_poly(items, ((x0, 0, z0), (x1, 0, z0), (x1, h, z0), (x0, h, z0)),
                           color, 0.76, outline)
        elif pz > z1:
            self._add_poly(items, ((x0, 0, z1), (x1, 0, z1), (x1, h, z1), (x0, h, z1)),
                           color, 0.76, outline)

    def _add_octa(self, items, cell, r, color, y=0.45):
        """Rotierender Oktaeder-Kristall (Futter / Goldapfel)."""
        cx, cz = cell[0] + 0.5, cell[1] + 0.5
        top = (cx, y + r, cz)
        bot = (cx, max(0.04, y - r), cz)
        a0 = self.anim_t * 2.4
        eq = [(cx + math.cos(a0 + i * math.pi / 2) * r, y,
               cz + math.sin(a0 + i * math.pi / 2) * r) for i in range(4)]
        px, py, pz = self._cam_pos
        lx, ly, lz = 0.42, -0.82, 0.39               # Lichtrichtung (von oben)
        for i in range(4):
            for tri in ((top, eq[i], eq[(i + 1) % 4]),
                        (bot, eq[(i + 1) % 4], eq[i])):
                a, b, c = tri
                ux, uy, uz = b[0] - a[0], b[1] - a[1], b[2] - a[2]
                vx, vy, vz = c[0] - a[0], c[1] - a[1], c[2] - a[2]
                nx = uy * vz - uz * vy
                ny = uz * vx - ux * vz
                nz = ux * vy - uy * vx
                fcx = (a[0] + b[0] + c[0]) / 3
                fcy = (a[1] + b[1] + c[1]) / 3
                fcz = (a[2] + b[2] + c[2]) / 3
                if nx * (fcx - cx) + ny * (fcy - y) + nz * (fcz - cz) < 0:
                    nx, ny, nz = -nx, -ny, -nz       # Normale nach aussen
                if nx * (fcx - px) + ny * (fcy - py) + nz * (fcz - pz) >= 0:
                    continue                         # Rückseite
                nl = math.sqrt(nx * nx + ny * ny + nz * nz) or 1.0
                lit = max(0.0, -(nx * lx + ny * ly + nz * lz) / nl)
                self._add_poly(items, tri, color, 0.55 + 0.5 * lit)

    def _add_border_walls(self, items):
        """Niedrige Bande rund um das Spielfeld (leicht gestreift)."""
        h = 0.55
        for gx in range(self.cols):
            sh = 0.9 if gx % 2 == 0 else 0.78
            self._add_poly(items, ((gx, 0, 0), (gx + 1, 0, 0),
                                   (gx + 1, h, 0), (gx, h, 0)), COL_BORDER, sh)
            self._add_poly(items, ((gx, 0, self.rows), (gx + 1, 0, self.rows),
                                   (gx + 1, h, self.rows), (gx, h, self.rows)),
                           COL_BORDER, sh)
        for gz in range(self.rows):
            sh = 0.84 if gz % 2 == 0 else 0.7
            self._add_poly(items, ((0, 0, gz), (0, 0, gz + 1),
                                   (0, h, gz + 1), (0, h, gz)), COL_BORDER, sh)
            self._add_poly(items, ((self.cols, 0, gz), (self.cols, 0, gz + 1),
                                   (self.cols, h, gz + 1), (self.cols, h, gz)),
                           COL_BORDER, sh)

    def _add_snake3d(self, items, sn, idx):
        """Die Schlange als Kette aus Quadern (Kopf grösser und heller)."""
        col_body, col_head = SNAKE_COLORS[idx % len(SNAKE_COLORS)]
        col_head = self._head_color(idx)
        if not sn.alive:
            col_body = tuple(c // 2 for c in col_body)
            col_head = col_body
        cells = self._interp_cells(sn)
        n = len(cells)
        for k, (gx, gz) in enumerate(cells):
            if k == n - 1:                           # Kopf
                farbe, w, h = col_head, 0.92, 0.78
            else:
                t = k / max(1, n - 1)
                farbe = tuple(int(kc + (bc - kc) * (1 - t * 0.5))
                              for kc, bc in zip(col_head, col_body))
                w, h = 0.8, 0.58
            self._add_box(items, gx, gz, w, h, farbe,
                          tuple(c // 3 for c in farbe))

    # ----- Hintergrund / Boden ---------------------------------------------
    def _draw_sky(self, s):
        """Vertikaler Himmels-Gradient (gecacht, unten = Nebelfarbe)."""
        if self._sky_cache is None or self._sky_cache[0] != (self.width, self.height):
            surf = pygame.Surface((self.width, self.height))
            hor = int(self.height * 0.40)
            haze = int(self.height * 0.55)
            for y in range(self.height):
                if y < hor:
                    t = y / max(1, hor)
                    c = [int(a + (b - a) * t)
                         for a, b in zip(COL_SKY_TOP, COL_SKY_HOR)]
                elif y < haze:
                    t = (y - hor) / max(1, haze - hor)
                    c = [int(a + (b - a) * t)
                         for a, b in zip(COL_SKY_HOR, COL_FOG)]
                else:
                    c = COL_FOG
                pygame.draw.line(surf, c, (0, y), (self.width, y))
            self._sky_cache = ((self.width, self.height), surf)
        s.blit(self._sky_cache[1], (0, 0))

    def _draw_stars3d(self, s):
        """Funkelnde Sterne, die beim Drehen der Kamera mitwandern (Parallaxe)."""
        _, _, f = self._basis
        yaw = math.atan2(f[0], f[2])
        hor = self.height * 0.40
        for az, hf, size, ph in self._stars:
            d = (az - yaw + math.pi) % math.tau - math.pi
            if abs(d) > 0.7:
                continue
            sx = self._scx + math.tan(d) * self._f
            if sx < -4 or sx > self.width + 4:
                continue
            sy = hor * (0.08 + 0.84 * hf)
            tw = 0.55 + 0.45 * math.sin(self.anim_t * 1.7 + ph)
            c = int(120 + 110 * tw)
            pygame.draw.circle(s, (c, c, min(255, c + 25)), (int(sx), int(sy)), size)

    def _draw_floor3d(self, s):
        """Schachbrett-Boden; Zellecken werden pro Frame nur einmal transformiert."""
        corners = {}

        def corner(gx, gz):
            v = corners.get((gx, gz))
            if v is None:
                v = self._to_cam((float(gx), 0.0, float(gz)))
                corners[(gx, gz)] = v
            return v

        lim = FOG_END + 1.0
        for gz in range(self.rows):
            for gx in range(self.cols):
                c00 = corner(gx, gz)
                # grobes Cull: zu weit weg, hinter der Kamera oder seitlich draussen
                if c00[2] > lim or c00[2] < -1.8:
                    continue
                if abs(c00[0]) > c00[2] * 1.7 + 2.5:
                    continue
                quad = [c00, corner(gx + 1, gz),
                        corner(gx + 1, gz + 1), corner(gx, gz + 1)]
                if all(c[2] < NEAR for c in quad):
                    continue
                poly = self._clip_near(quad)
                if len(poly) < 3:
                    continue
                depth = sum(c[2] for c in poly) / len(poly)
                col = COL_TILE_A if (gx + gz) & 1 == 0 else COL_TILE_B
                pygame.draw.polygon(s, self._fog_color(col, depth),
                                    [self._proj(c) for c in poly])

    # ----- Details / Effekte -------------------------------------------------
    def _draw_eyes3d(self, s):
        """Augen auf der Stirnseite des Kopfes (nur wenn sie zur Kamera zeigen)."""
        sn = self.snakes[0]
        if not sn.alive:
            return
        cells = self._interp_cells(sn)
        hx, hz = cells[-1]
        dx, dz = sn.direction
        fx, fz = hx + 0.5 + dx * 0.47, hz + 0.5 + dz * 0.47
        if (fx - self._cam_pos[0]) * dx + (fz - self._cam_pos[2]) * dz > 0:
            return                                   # Gesicht von der Kamera abgewandt
        px, pz = -dz, dx
        for sign in (-1, 1):
            e = (fx + px * sign * 0.17, 0.52, fz + pz * sign * 0.17)
            c = self._to_cam(e)
            if c[2] < NEAR:
                continue
            sx, sy = self._proj(c)
            rr = max(1, int(self._f * 0.05 / c[2]))
            pygame.draw.circle(s, (250, 250, 250), (int(sx), int(sy)), rr)
            pupil = self._to_cam((e[0] + dx * 0.06, 0.5, e[2] + dz * 0.06))
            if pupil[2] >= NEAR:
                pxy = self._proj(pupil)
                pygame.draw.circle(s, (20, 20, 30),
                                   (int(pxy[0]), int(pxy[1])), max(1, rr // 2))

    def _draw_boost_glow3d(self, s):
        """Pulsierender Glow über dem Kopf, solange der Boost läuft."""
        sn = self.snakes[0]
        if not (sn.alive and sn.boost_on and sn.stamina > 0):
            return
        cells = self._interp_cells(sn)
        hx, hz = cells[-1]
        c = self._to_cam((hx + 0.5, 0.42, hz + 0.5))
        if c[2] < NEAR:
            return
        sx, sy = self._proj(c)
        r = max(8, int(self._f * 0.62 / c[2]))
        pulse = 90 + int(50 * math.sin(self.anim_t * 14))
        glow = pygame.Surface((r * 2, r * 2), pygame.SRCALPHA)
        pygame.draw.circle(glow, (*BOOST_GLOW[0], pulse), (r, r), r)
        s.blit(glow, (sx - r, sy - r))

    def _draw_particles3d_pass(self, s):
        for p in self.particles3d:
            c = self._to_cam((p[0], p[1], p[2]))
            if c[2] < NEAR:
                continue
            sx, sy = self._proj(c)
            if sx < -8 or sx > self.width + 8 or sy < -8 or sy > self.height + 8:
                continue
            r = max(1, int(self._f * 0.045 / c[2]))
            t = max(0.0, min(1.0, p[6] / 0.7))
            col = tuple(int(fc + (pc - fc) * t)
                        for pc, fc in zip(p[7], COL_FOG))
            pygame.draw.circle(s, self._fog_color(col, c[2]),
                               (int(sx), int(sy)), r)

    # ----- HUD ----------------------------------------------------------
    def _draw_hud(self):
        s = self.surface

        # WLS-Anzeige (durch die Wände?) bzw. 3D-Badge
        if self.view3d_active:
            badge = self.font.render(i18n.t("snake.view_3d"), True, COL_ACCENT)
            s.blit(badge, badge.get_rect(midtop=(self.width // 2, 6)))
        else:
            wls_col = COL_WLS_ON if self.wrap_active else COL_WLS_OFF
            wls = self.font.render(i18n.t("snake.wls"), True, wls_col)
            s.blit(wls, wls.get_rect(midtop=(self.width // 2, 6)))
        mode_name = i18n.t("snake.mode." + self.mode_key)
        mimg = self._small.render(mode_name, True, COL_ACCENT)
        s.blit(mimg, mimg.get_rect(midtop=(self.width // 2, 30)))
        if self.view3d_active and not self.game_over and self._run_t < 6.0:
            hint = self._tiny.render(i18n.t("snake.steer_hint"), True, COL_DIM)
            s.blit(hint, hint.get_rect(midtop=(self.width // 2, 50)))

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
        s.blit(self.font.render(i18n.t("common.points", score=self.score),
                                True, COL_TEXT), (10, 8))
        if self.competitive:
            self._draw_comp_hud()
        else:
            s.blit(self._small.render(
                i18n.t("snake.apples_bank", total=self.apples_total, bank=self.apples_bank),
                True, COL_FOOD), (10, 34))
            if self.prestige > 0:
                blocks = prestige.blocks_per_apple(self.prestige)
                info = self._small.render(
                    i18n.t("snake.prestige_info", roman=prestige.roman(self.prestige),
                           blocks=blocks),
                    True, COL_MULT)
                s.blit(info, (10, 54))

        best = self._small.render(i18n.t("snake.best", hs=self.highscore), True, COL_DIM)
        s.blit(best, (self.width - best.get_width() - 10, 10))

        self._draw_boost_bar(10, self.height - 22, 180, self.snakes[0], 0)

        if not self.game_over and self.competitive:
            self._draw_comp_footer()
        elif not self.game_over and self.mode_key != "timed":
            self._draw_next_prestige()

    def _draw_comp_hud(self):
        """Competitive-Kopfzeile: gesammelte Äpfel, Level, Größe, Slot-Bonus."""
        s = self.surface
        lvl = self.comp_level
        line = self._small.render(
            i18n.t("snake.comp.stats", apples=self.apples_total, level=lvl,
                   mult=competitive.score_multiplier(lvl)), True, COL_FOOD)
        s.blit(line, (10, 34))
        # Größe als Kommazahl (fürs Gambling weiter nutzbar), oben links.
        groesse = self._snake_size(self.snakes[0])
        gtxt = f"{groesse:.2f}".rstrip("0").rstrip(".")
        gimg = self._small.render(i18n.t("snake.comp.size", size=gtxt), True, COL_MULT)
        s.blit(gimg, (10, 54))
        if self.spawn_bonus_t > 0 and self.spawn_bonus > 0:
            b = self._small.render(
                i18n.t("snake.comp.bonus", n=self.spawn_bonus, t=self.spawn_bonus_t),
                True, COL_BLUE)
            s.blit(b, (10, 74))
        if self.hardcore:
            puls = 0.5 + 0.5 * math.sin(pygame.time.get_ticks() * 0.006)
            col = (255, int(70 + 80 * puls), int(70 + 80 * puls))
            tag = self._small.render(i18n.t("snake.hardcore_tag"), True, col)
            tr = tag.get_rect(midtop=(self.width // 2, 48))
            box = tr.inflate(16, 8)
            glow = pygame.Surface(box.size, pygame.SRCALPHA)
            pygame.draw.rect(glow, (*COL_HARDCORE, int(40 + 60 * puls)),
                             glow.get_rect(), border_radius=8)
            s.blit(glow, box)
            pygame.draw.rect(s, COL_HARDCORE, box, 1, border_radius=8)
            s.blit(tag, tr)

    def _draw_comp_footer(self):
        """Fortschrittsbalken bis zum nächsten Competitive-Level (unten mittig)."""
        s = self.surface
        step = competitive.next_step(self.apples_total)
        if step is None:
            img = self._small.render(i18n.t("snake.comp.max", level=self.comp_level),
                                     True, COL_MULT)
            s.blit(img, img.get_rect(midbottom=(self.width // 2, self.height - 8)))
            return
        have, need = step
        w, y = 220, self.height - 20
        x = self.width // 2 - w // 2
        pygame.draw.rect(s, (30, 34, 46), (x, y, w, 12), border_radius=5)
        fw = int((w - 2) * max(0.0, min(1.0, have / max(1, need))))
        pygame.draw.rect(s, COL_MULT, (x + 1, y + 1, fw, 10), border_radius=5)
        lab = self._tiny.render(
            i18n.t("snake.comp.next", level=self.comp_level + 1, have=have, need=need),
            True, COL_DIM)
        s.blit(lab, lab.get_rect(midbottom=(self.width // 2, y - 2)))

    # ----- Slot-Machine zeichnen ----------------------------------------
    def _draw_slot(self):
        s = self.surface
        sl = self.slot
        ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        ov.fill((0, 0, 0, 170))
        s.blit(ov, (0, 0))

        pw, ph = min(360, self.width - 40), 260
        panel = pygame.Rect((self.width - pw) // 2, (self.height - ph) // 2, pw, ph)
        pygame.draw.rect(s, (30, 34, 52), panel, border_radius=14)
        pygame.draw.rect(s, COL_BLUE, panel, 3, border_radius=14)

        title = self.font.render(i18n.t("snake.slot.title"), True, COL_BLUE)
        s.blit(title, title.get_rect(midtop=(panel.centerx, panel.top + 12)))
        stake = self._small.render(i18n.t("snake.slot.stake", n=sl["stake"]),
                                   True, COL_DIM)
        s.blit(stake, stake.get_rect(midtop=(panel.centerx, panel.top + 40)))

        rw, rh, gap = 78, 96, 12
        total = rw * 3 + gap * 2
        x0 = panel.centerx - total // 2
        ry = panel.top + 74
        cell_h = rh // 3
        reel = competitive.REEL
        for k in range(3):
            win = pygame.Rect(x0 + k * (rw + gap), ry, rw, rh)
            pygame.draw.rect(s, (16, 18, 30), win, border_radius=8)
            old_clip = s.get_clip()
            s.set_clip(win)
            if sl["t"] >= sl["stop"][k]:               # Walze steht
                anchor = reel.index(sl["reels"][k])
                for row in (-1, 0, 1):
                    sym = reel[(anchor + row) % len(reel)]
                    self._draw_slot_symbol(s, sym, win.centerx,
                                           win.centery + row * cell_h, cell_h // 2 - 4)
            else:                                      # Walze dreht
                base = int(sl["t"] * SLOT_SPIN_SPEED) + k * 5
                scroll = (sl["t"] * SLOT_SPIN_SPEED % 1.0) * cell_h
                for row in (-1, 0, 1, 2):
                    sym = reel[(base + row) % len(reel)]
                    self._draw_slot_symbol(s, sym, win.centerx,
                                           int(win.centery + row * cell_h - scroll),
                                           cell_h // 2 - 4)
            s.set_clip(old_clip)
            pygame.draw.rect(s, (70, 80, 110), win, 2, border_radius=8)
        pygame.draw.line(s, COL_BLUE, (x0 - 4, ry + rh // 2),
                         (x0 + total + 4, ry + rh // 2), 1)

        if sl["applied"]:
            res = i18n.t("snake.slot." + sl["result"])
            col = COL_GOLD if sl["result"] == "jackpot" \
                else (COL_WLS_ON if sl["result"] == "pair" else COL_DIM)
            rimg = self.font.render(f"{res}  x{sl['mult']:g}", True, col)
            s.blit(rimg, rimg.get_rect(midbottom=(panel.centerx, panel.bottom - 14)))

    def _draw_slot_symbol(self, s, key, cx, cy, r):
        """Zeichnet ein Slot-Symbol als kleines Icon."""
        col = competitive.SLOT_SYMBOLS[key]
        if key == "seven":
            img = self.font.render("7", True, col)
            s.blit(img, img.get_rect(center=(cx, cy)))
        elif key == "gem":
            pts = [(cx, cy - r), (cx + r, cy), (cx, cy + r), (cx - r, cy)]
            pygame.draw.polygon(s, col, pts)
            pygame.draw.polygon(s, (255, 255, 255), pts, 1)
        elif key == "bell":
            pygame.draw.circle(s, col, (cx, cy - r // 6), int(r * 0.8))
            pygame.draw.rect(s, col, (cx - r, cy + r // 3, r * 2, max(2, r // 3)),
                             border_radius=2)
            pygame.draw.circle(s, (40, 40, 30), (cx, cy + r // 2), max(1, r // 5))
        elif key == "apple":
            pygame.draw.circle(s, col, (cx, cy), int(r * 0.85))
            pygame.draw.circle(s, (255, 180, 180), (cx - r // 4, cy - r // 4),
                               max(1, r // 6))
        else:  # cherry
            for off in (-r // 2, r // 2):
                pygame.draw.circle(s, col, (cx + off, cy + r // 3), int(r * 0.45))
            pygame.draw.line(s, (120, 200, 120), (cx, cy - r), (cx, cy + r // 3), 1)

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
        lab = self._tiny.render(i18n.t("snake.boost"), True, (15, 15, 20))
        s.blit(lab, (x + 6, y))

    def _draw_next_prestige(self):
        s = self.surface
        req, ok = self._can_prestige()
        if req is None:
            txt = i18n.t("snake.max_prestige", roman=prestige.roman(self.prestige))
            col = COL_MULT
        else:
            txt = i18n.t("snake.next_prestige", roman=req['roman'],
                         apples=req['apples'], length=req['length'])
            if ok:
                txt += i18n.t("snake.press_p")
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
                text, farbe = i18n.t("common.draw"), COL_TEXT
            else:
                text = i18n.t("common.player_wins", n=self.winner + 1)
                farbe = SNAKE_COLORS[self.winner][1]
            self.draw_center_text(text, self.big_font, farbe, -30)
            a0, a1 = self.snakes[0].apples, self.snakes[1].apples
            self.draw_center_text(i18n.t("snake.apples_pp", a0=a0, a1=a1),
                                  self.font, COL_DIM, 14)
            self.draw_center_text(i18n.t("common.enter_restart"), self.font, COL_TEXT, 48)
            return

        titel = i18n.t("snake.time_up") \
            if (self.mode_key == "timed" and self.snakes[0].alive) \
            else i18n.t("common.game_over")
        self.draw_center_text(titel, self.big_font, COL_FOOD, -60)
        self.draw_center_text(i18n.t("snake.apples_collected", n=self.apples_total),
                              self.font, COL_FOOD, -14)
        if self.competitive:
            self.draw_center_text(i18n.t("snake.comp.result", level=self.comp_level),
                                  self.font, COL_MULT, 16)
        elif self.prestige > 0:
            self.draw_center_text(
                i18n.t("snake.prestige", roman=prestige.roman(self.prestige)),
                self.font, COL_MULT, 16)
        self.draw_center_text(i18n.t("common.enter_restart"), self.font, COL_TEXT, 46)

    # ----- Setup zeichnen ----------------------------------------------
    def _draw_setup(self):
        s = self.surface
        s.fill(COL_BG)

        title = self.big_font.render("SNAKE", True, COL_TEXT)
        s.blit(title, title.get_rect(center=(self.width // 2, 52)))
        modus = i18n.t("snake.multiplayer") if self.multiplayer \
            else i18n.t("snake.singleplayer")
        sub = self._small.render(i18n.t("snake.subtitle", mode=modus, hs=self.highscore),
                                 True, COL_DIM)
        s.blit(sub, sub.get_rect(center=(self.width // 2, 88)))

        # Modus-Auswahl
        m = MODES[self.mode_index]
        pygame.draw.rect(s, (38, 44, 60), self.mode_panel, border_radius=10)
        pygame.draw.rect(s, COL_ACCENT, self.mode_panel, 2, border_radius=10)
        name = self.font.render(i18n.t("snake.mode." + m["key"]), True, COL_TEXT)
        s.blit(name, name.get_rect(center=(self.mode_panel.centerx,
                                           self.mode_panel.top + 18)))
        desc = self._small.render(i18n.t("snake.mode." + m["key"] + ".desc"),
                                  True, COL_DIM)
        s.blit(desc, desc.get_rect(center=(self.mode_panel.centerx,
                                           self.mode_panel.top + 41)))
        for r, sym in ((self.mode_left, "<"), (self.mode_right, ">")):
            arr = self.big_font.render(sym, True, COL_ACCENT)
            s.blit(arr, arr.get_rect(center=r.center))
        allowed = self._allowed_modes()
        pos = allowed.index(self.mode_index) if self.mode_index in allowed else 0
        dots = " ".join("*" if i == pos else "." for i in range(len(allowed)))
        d = self._tiny.render(dots, True, COL_DIM)
        s.blit(d, d.get_rect(center=(self.width // 2, self.mode_panel.bottom + 9)))

        # Ansicht 2D/3D
        if self.multiplayer:
            self._draw_disabled_row(self.view_rect, i18n.t("snake.view_toggle"),
                                    i18n.t("snake.view_mp_hint"))
        else:
            pygame.draw.rect(s, COL_BTN_ON if self.view3d else COL_BTN,
                             self.view_rect, border_radius=8)
            pygame.draw.rect(s, COL_ACCENT if self.view3d else COL_DIM,
                             self.view_rect, 1, border_radius=8)
            lab = self.font.render(i18n.t("snake.view_toggle"), True, COL_TEXT)
            s.blit(lab, (self.view_rect.x + 16,
                         self.view_rect.centery - lab.get_height() // 2))
            wert = i18n.t("snake.view_3d") if self.view3d else i18n.t("snake.view_2d")
            col = COL_ACCENT if self.view3d else COL_DIM
            img = self.font.render(f"< {wert} >", True, col)
            s.blit(img, (self.view_rect.right - img.get_width() - 16,
                         self.view_rect.centery - img.get_height() // 2))

        if self.view3d_active:
            self._draw_cam_button(self.wrap_rect)
        else:
            self._draw_setup_toggle(self.wrap_rect, i18n.t("snake.wrap_toggle"),
                                    self.wrap)
        self._draw_setup_toggle(self.bonus_rect, i18n.t("snake.bonus_toggle"), self.bonus)
        if self.competitive:
            # Äpfel sind vom Level bestimmt -> die Zeile wird zum HARDCORE-Schalter.
            self._draw_hardcore_toggle(self.apples_rect, self.hardcore)
        else:
            self._draw_setup_value(self.apples_rect, i18n.t("snake.apples_toggle"),
                                   str(self.apple_count), self.apple_count > 1)

        pygame.draw.rect(s, COL_BTN_ON, self.start_rect, border_radius=10)
        st = self.font.render(i18n.t("common.start"), True, COL_TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))

        if self.view3d_active:
            extra = self._tiny.render(i18n.t("snake.hint_3d"), True, COL_ACCENT)
            s.blit(extra, extra.get_rect(center=(self.width // 2, self.height - 54)))
        hint = self._small.render(i18n.t("snake.setup_hint"), True, COL_DIM)
        s.blit(hint, hint.get_rect(center=(self.width // 2, self.height - 34)))
        if self.competitive and self.hardcore:
            puls = 0.5 + 0.5 * math.sin(pygame.time.get_ticks() * 0.006)
            hc = self._tiny.render(i18n.t("snake.hardcore_hint"), True,
                                   (255, int(80 + 60 * puls), int(80 + 60 * puls)))
            s.blit(hc, hc.get_rect(center=(self.width // 2, self.height - 14)))
        else:
            boost = self._tiny.render(i18n.t("snake.boost_hint"), True, (120, 200, 150))
            s.blit(boost, boost.get_rect(center=(self.width // 2, self.height - 14)))

        self._draw_brush_button(s)

    def _draw_brush_button(self, s):
        """Pinsel-Knopf oben rechts (Personalisieren) - ohne Text, mit Farb-Spitze."""
        r = self.brush_rect
        pygame.draw.rect(s, COL_BTN, r, border_radius=8)
        pygame.draw.rect(s, COL_ACCENT, r, 1, border_radius=8)
        tip = ngb.head_color()                       # Spitze zeigt die aktive Kopffarbe
        hx0, hy0 = r.left + 10, r.bottom - 10        # Borsten-Spitze (unten links)
        hx1, hy1 = r.right - 8, r.top + 8            # Griffende (oben rechts)
        pygame.draw.line(s, (170, 140, 95), (hx0 + 4, hy0 - 4), (hx1, hy1), 4)
        fx = hx0 + (hx1 - hx0) * 0.30                # Metallzwinge
        fy = hy0 + (hy1 - hy0) * 0.30
        pygame.draw.circle(s, (205, 208, 216), (int(fx), int(fy)), 3)
        pygame.draw.polygon(s, tip, [(hx0 - 4, hy0 + 2), (hx0 + 5, hy0 - 5),
                                     (hx0 + 2, hy0 + 6)])
        pygame.draw.circle(s, tip, (hx0 - 1, hy0 + 2), 3)

    def _draw_cam_button(self, rect):
        """Setup-Zeile im 3D-Modus: öffnet das 3D-Kamera-Menü."""
        s = self.surface
        pygame.draw.rect(s, COL_BTN, rect, border_radius=8)
        pygame.draw.rect(s, COL_ACCENT, rect, 1, border_radius=8)
        lab = self.font.render(i18n.t("snake.cam.open"), True, COL_TEXT)
        s.blit(lab, (rect.x + 16, rect.centery - lab.get_height() // 2))
        stat = i18n.t("common.on") if self.cam_smooth else i18n.t("common.off")
        img = self.font.render(f"{stat}  ›", True, COL_ACCENT)
        s.blit(img, (rect.right - img.get_width() - 14,
                     rect.centery - img.get_height() // 2))

    # ----- 3D-Kamera-Menü ----------------------------------------------
    def _draw_cam3d(self):
        s = self.surface
        s.fill(COL_BG)
        title = self.big_font.render(i18n.t("snake.cam.title"), True, COL_TEXT)
        s.blit(title, title.get_rect(center=(self.width // 2, 60)))
        sub = self._small.render(i18n.t("snake.cam.subtitle"), True, COL_DIM)
        s.blit(sub, sub.get_rect(center=(self.width // 2, 100)))

        self._draw_setup_toggle(self.cam_smooth_rect, i18n.t("snake.cam.smooth"),
                                self.cam_smooth)
        self._draw_cam_value(self.cam_fov_rect, i18n.t("snake.cam.fov"),
                             f"{self.cam_fov:.2f}", self.cam_fov_minus, self.cam_fov_plus)
        self._draw_cam_value(self.cam_height_rect, i18n.t("snake.cam.height"),
                             f"{self.cam_height:.1f}", self.cam_height_minus,
                             self.cam_height_plus)
        self._draw_setup_toggle(self.cam_turn_rect, i18n.t("snake.cam.turn_shake"),
                                self.cam_turn_shake)

        pygame.draw.rect(s, COL_BTN_ON, self.cam_back_rect, border_radius=10)
        bt = self.font.render(i18n.t("snake.cam.back"), True, COL_TEXT)
        s.blit(bt, bt.get_rect(center=self.cam_back_rect.center))
        hint = self._tiny.render(i18n.t("snake.cam.hint"), True, COL_DIM)
        s.blit(hint, hint.get_rect(center=(self.width // 2, self.height - 16)))

    def _draw_cam_value(self, rect, label, value, minus, plus):
        """Wertzeile mit -/+ Knöpfen (FOV / Kamerahöhe)."""
        s = self.surface
        pygame.draw.rect(s, COL_BTN, rect, border_radius=8)
        pygame.draw.rect(s, COL_DIM, rect, 1, border_radius=8)
        lab = self.font.render(label, True, COL_TEXT)
        s.blit(lab, (rect.x + 16, rect.centery - lab.get_height() // 2))
        for r, sym in ((minus, "-"), (plus, "+")):
            pygame.draw.rect(s, COL_BTN_ON, r, border_radius=6)
            g = self.font.render(sym, True, COL_TEXT)
            s.blit(g, g.get_rect(center=r.center))
        val = self.font.render(value, True, COL_WLS_ON)
        s.blit(val, val.get_rect(center=((minus.right + plus.left) // 2, rect.centery)))

    def _draw_disabled_row(self, rect, label, note):
        """Abgeblendete Setup-Zeile mit Hinweistext (nicht anklickbar)."""
        s = self.surface
        pygame.draw.rect(s, (36, 40, 52), rect, border_radius=8)
        pygame.draw.rect(s, COL_DIM, rect, 1, border_radius=8)
        lab = self.font.render(label, True, COL_DIM)
        s.blit(lab, (rect.x + 16, rect.centery - lab.get_height() // 2))
        info = self._small.render(note, True, COL_DIM)
        s.blit(info, (rect.right - info.get_width() - 16,
                      rect.centery - info.get_height() // 2))

    def _draw_hardcore_toggle(self, rect, an):
        """HARDCORE-Setup-Zeile: leuchtet kräftig rot, wenn aktiv."""
        s = self.surface
        if an:
            puls = 0.5 + 0.5 * math.sin(pygame.time.get_ticks() * 0.006)
            glow = pygame.Surface((rect.w + 22, rect.h + 22), pygame.SRCALPHA)
            pygame.draw.rect(glow, (*COL_HARDCORE, int(55 + 90 * puls)),
                             glow.get_rect(), border_radius=14)
            s.blit(glow, glow.get_rect(center=rect.center))
            pygame.draw.rect(s, (95, 18, 22), rect, border_radius=8)
            pygame.draw.rect(s, COL_HARDCORE, rect, 2, border_radius=8)
            lab_col, val_col = (255, 210, 210), (255, 120, 120)
        else:
            pygame.draw.rect(s, COL_BTN, rect, border_radius=8)
            pygame.draw.rect(s, COL_DIM, rect, 1, border_radius=8)
            lab_col, val_col = COL_TEXT, COL_DIM
        lab = self.font.render(i18n.t("snake.hardcore_toggle"), True, lab_col)
        s.blit(lab, (rect.x + 16, rect.centery - lab.get_height() // 2))
        wert = i18n.t("common.on") if an else i18n.t("common.off")
        img = self.font.render(f"< {wert} >", True, val_col)
        s.blit(img, (rect.right - img.get_width() - 16,
                     rect.centery - img.get_height() // 2))

    def _draw_setup_toggle(self, rect, label, an):
        wert = i18n.t("common.on") if an else i18n.t("common.off")
        self._draw_setup_value(rect, label, wert, an)

    def _draw_setup_value(self, rect, label, wert, hervor):
        """Setup-Zeile mit einem Wert in < .. >-Klammern (Toggle oder Zahl)."""
        s = self.surface
        pygame.draw.rect(s, COL_BTN_ON if hervor else COL_BTN, rect, border_radius=8)
        pygame.draw.rect(s, COL_DIM, rect, 1, border_radius=8)
        lab = self.font.render(label, True, COL_TEXT)
        s.blit(lab, (rect.x + 16, rect.centery - lab.get_height() // 2))
        col = COL_WLS_ON if hervor else COL_DIM
        img = self.font.render(f"< {wert} >", True, col)
        s.blit(img, (rect.right - img.get_width() - 16,
                     rect.centery - img.get_height() // 2))
