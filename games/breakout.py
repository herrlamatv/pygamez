# -*- coding: utf-8 -*-
"""
breakout.py
===========
Breakout / Brick-Breaker  -  stark erweiterte Deluxe-Version.

Neu in dieser Version
---------------------
- Neue Steinsorten:
    * Normal   (1-3 Treffer, Farbe = Resthärte)
    * Stahl    (unzerstörbar, prallt nur ab - zählt NICHT zum Levelziel)
    * Bombe    (explodiert und reisst Nachbarn mit)
    * Gold     (viele Extrapunkte)
- Viele neue Power-ups zusätzlich zu den alten:
    Laser (Kanonen am Schläger), Feuerball (durchschlägt Steine),
    Klebrig (Ball haftet), Schild (Auffangnetz unten), Münze (Bonuspunkte).
- Combo-System mit steigendem Punkte-Multiplikator.
- Partikel-Effekte, Ball-Spuren, Bildschirm-Wackeln (Screen-Shake),
  aufsteigende Punkte-Popups.
- Neue Level-Muster: Herz, Wellen, Ringe, Kreuz, Rahmen, Zufall u.a.,
  insgesamt deutlich mehr Level.
- Komplett überarbeitete Oberfläche: Sternen-Hintergrund mit Verlauf,
  neuer Setup-Screen, Level-Intro-Banner, Effekt-Anzeigen, Pause-Screen.

Steuerung
---------
- Setup: 1/2/3 = Schwierigkeit, Pfeil links/rechts = Ballfarbe,
         Hoch/Runter = Startlevel, M = Aufbau, Enter/Leertaste = Start
         (alles auch per Mausklick).
- Spiel: Maus oder Pfeil links/rechts bewegt den Schläger,
         Leertaste/Klick startet bzw. löst den haftenden Ball (und feuert Laser),
         P/Esc = Pause.
"""

import math
import random
import pygame

import i18n
import ui
from game_base import Game, InputEvent

# ---------------------------------------------------------------- Farben
# Allgemeine UI-Farben kommen zur Zeichenzeit dynamisch aus der ui.*-Palette
# (ein Theme-Wechsel färbt das Spiel sofort um). Hier stehen nur noch die
# Identitätsfarben des Spiels (Steine, Bälle, Power-ups).

# Steinfarbe nach Resthärte (Treffer bis zur Zerstörung)
STR_COLORS = {1: (120, 205, 120), 2: (235, 185, 80), 3: (230, 95, 95)}

# Auswählbare Ballfarben (Name, RGB)
BALL_COLORS = [
    ("Gelb",   (255, 230, 120)),
    ("Weiss",  (240, 240, 240)),
    ("Cyan",   (110, 230, 230)),
    ("Pink",   (255, 120, 200)),
    ("Grün",  (120, 240, 140)),
    ("Orange", (255, 160, 70)),
    ("Lila",   (185, 130, 255)),
]

# Schwierigkeitsgrade
DIFFICULTIES = {
    "Easy":   dict(lives=5, paddle=140, ball_speed=300, drop=0.42, bad=0.12, hard_bonus=0),
    "Medium": dict(lives=3, paddle=110, ball_speed=365, drop=0.32, bad=0.28, hard_bonus=0),
    "Hard":   dict(lives=2, paddle=92,  ball_speed=440, drop=0.26, bad=0.42, hard_bonus=1),
}
DIFF_ORDER = ["Easy", "Medium", "Hard"]

# Level-Definitionen.
#   tag  : "Normal" | "Schwer" | "Spass"  (Anzeige + Charakter des Levels)
#   pat  : Muster (siehe _brick_da)
#   rows : Anzahl Steinreihen
#   cols : Anzahl Steinspalten
#   base : Grund-Resthärte der Steine (1-3)
#   drop : Multiplikator für die Power-up-Häufigkeit
#   spec : Anteil "besonderer" Steine (Bombe/Gold/Stahl-Würze), 0..1
LEVEL_DEFS = [
    dict(tag="Normal", pat="full",    rows=3, cols=11, base=1, drop=1.0, spec=0.05),
    dict(tag="Normal", pat="checker", rows=4, cols=11, base=1, drop=1.0, spec=0.06),
    dict(tag="Spass",  pat="full",    rows=6, cols=16, base=1, drop=1.7, spec=0.10),
    dict(tag="Normal", pat="pyramid", rows=6, cols=11, base=1, drop=1.0, spec=0.08),
    dict(tag="Schwer", pat="border",  rows=6, cols=13, base=2, drop=0.8, spec=0.12),
    dict(tag="Spass",  pat="full",    rows=8, cols=20, base=1, drop=2.0, spec=0.14),
    dict(tag="Normal", pat="columns", rows=6, cols=11, base=1, drop=1.0, spec=0.08),
    dict(tag="Schwer", pat="checker", rows=6, cols=11, base=2, drop=0.8, spec=0.12),
    dict(tag="Spass",  pat="waves",   rows=8, cols=18, base=1, drop=1.8, spec=0.12),
    dict(tag="Normal", pat="rows",    rows=7, cols=11, base=1, drop=1.0, spec=0.08),
    dict(tag="Schwer", pat="cross",   rows=8, cols=13, base=2, drop=0.8, spec=0.15),
    dict(tag="Spass",  pat="heart",   rows=8, cols=15, base=1, drop=2.2, spec=0.10),
    dict(tag="Normal", pat="pyramid", rows=8, cols=13, base=1, drop=1.0, spec=0.10),
    dict(tag="Schwer", pat="rings",   rows=8, cols=15, base=2, drop=0.7, spec=0.18),
    dict(tag="Spass",  pat="checker", rows=8, cols=18, base=1, drop=2.0, spec=0.12),
    dict(tag="Normal", pat="diamond", rows=8, cols=13, base=2, drop=1.0, spec=0.12),
    dict(tag="Schwer", pat="border",  rows=8, cols=15, base=3, drop=0.6, spec=0.20),
    dict(tag="Spass",  pat="random",  rows=8, cols=20, base=1, drop=2.4, spec=0.16),
    dict(tag="Schwer", pat="waves",   rows=8, cols=15, base=3, drop=0.6, spec=0.20),
    dict(tag="Normal", pat="heart",   rows=9, cols=17, base=2, drop=1.0, spec=0.12),
    dict(tag="Schwer", pat="cross",   rows=9, cols=15, base=3, drop=0.6, spec=0.22),
    dict(tag="Spass",  pat="full",    rows=9, cols=22, base=1, drop=2.6, spec=0.18),
    dict(tag="Schwer", pat="rings",   rows=9, cols=17, base=3, drop=0.6, spec=0.24),
    dict(tag="Schwer", pat="full",    rows=9, cols=13, base=3, drop=0.55, spec=0.26),
    dict(tag="Schwer", pat="full",    rows=9, cols=11, base=3, drop=0.5, spec=0.30),   # Finale
]
NUM_LEVELS = len(LEVEL_DEFS)

PADDLE_H = 16
BALL_R = 8
BALL_MIN_SPEED = 200
BALL_MAX_SPEED = 760
PADDLE_MIN = 60
PADDLE_MAX = 220
MAX_BALLS = 20
POWERUP_FALL = 165          # Fallgeschwindigkeit der Power-ups (px/s)
LASER_SPEED = 620
LASER_INTERVAL = 0.28       # Sekunden zwischen zwei Laser-Salven

# Dauer der zeitlich begrenzten Effekte (Sekunden)
FX_DUR = dict(laser=9.0, fire=7.0, sticky=11.0, shield=10.0)

# Spielzustände
SETUP, PLAY, PAUSE, OVER = "setup", "play", "pause", "over"


# ==================================================================== Ball
class Ball:
    """Einzelner Ball mit Position, Geschwindigkeit und Spur."""

    def __init__(self, x, y, vx, vy):
        self.x, self.y, self.vx, self.vy = x, y, vx, vy
        self.trail = []                 # letzte Positionen für die Ball-Spur
        self.stuck = False              # haftet gerade am Schläger (Klebrig)?
        self.stuck_off = 0.0            # Abstand zur Schlägermitte beim Haften

    def speed(self):
        return math.hypot(self.vx, self.vy)

    def scale_speed(self, faktor):
        s = self.speed() or 1.0
        neu = max(BALL_MIN_SPEED, min(BALL_MAX_SPEED, s * faktor))
        self.vx *= neu / s
        self.vy *= neu / s


# ================================================================= PowerUp
class PowerUp:
    """Herabfallendes Power-up."""

    def __init__(self, x, y, kind):
        self.kind = kind
        self.y = float(y)               # exakte Fallposition (Rects sind int)
        self.rect = pygame.Rect(int(x) - 17, int(y) - 11, 34, 22)
        self.phase = random.uniform(0, math.tau)

    # Anzeige je Typ: (Beschriftung, Farbe, gut?)
    INFO = {
        "multi":  ("x2", (110, 230, 230), True),
        "spread": ("x3", (110, 170, 255), True),
        "speed":  (">>", (255, 150, 70), False),
        "slow":   ("<<", (185, 130, 255), True),
        "wide":   ("W",  (120, 240, 140), True),
        "shrink": ("S",  (230, 95, 95), False),
        "life":   ("+",  (255, 120, 160), True),
        "laser":  ("L",  (255, 90, 90), True),
        "fire":   ("F",  (255, 170, 60), True),
        "sticky": ("G",  (150, 255, 150), True),
        "shield": ("U",  (120, 200, 255), True),
        "coin":   ("$",  (255, 220, 90), True),
    }


# ================================================================== Brick
class Brick:
    """Ein einzelner Stein mit Sorte und Resthärte."""

    NORMAL, STEEL, BOMB, GOLD = "normal", "steel", "bomb", "gold"

    def __init__(self, rect, strength, kind="normal"):
        self.rect = rect
        self.strength = strength
        self.kind = kind

    @property
    def breakable(self):
        return self.kind != Brick.STEEL


# =============================================================== Hauptspiel
class BreakoutGame(Game):
    name = "Breakout"
    highscore_key = "breakout"

    # ----- Setup / Zustand ---------------------------------------------
    def reset(self):
        self.score = 0
        self.game_over = False
        self.state = SETUP

        # Setup-Auswahl
        self.diff_name = "Medium"
        self.color_index = 0
        self.start_level_index = 0
        self.level_mode = "Standard"    # "Standard" (gemischt) oder "Voll"

        self._make_fonts()

        # Visuelle Extras
        self.particles = []
        self.floaters = []
        self.lasers = []
        self.shake = 0.0
        self.anim_t = 0.0

        self._scene = None
        self._stars = []
        self._build_bg()
        self._build_setup_layout()

    def on_surface_changed(self):
        """Wird vom Rahmen bei Größenänderung aufgerufen."""
        self._scene = None
        self._make_fonts()
        self._build_bg()
        self._build_setup_layout()

    def _make_fonts(self):
        """Schriftgrössen aus der Fensterhöhe ableiten (Theme-Schriftart)."""
        h = self.height
        self._tiny = ui.font(max(11, h // 36))
        self._small = ui.font(max(13, h // 30))
        self._mid = ui.font(max(16, h // 24), bold=True)

    # ----- Hintergrund (Theme-Verlauf + eigene Funkel-Sterne) -----------
    def _build_bg(self):
        w, h = self.width, self.height
        self._stars = [(random.randint(0, w), random.randint(0, h),
                        random.uniform(0.4, 1.0), random.uniform(0.5, 1.6))
                       for _ in range(70)]

    def _get_scene(self):
        if self._scene is None or self._scene.get_size() != (self.width, self.height):
            self._scene = pygame.Surface((self.width, self.height))
            try:
                self._scene = self._scene.convert()
            except pygame.error:
                pass
        return self._scene

    def _blit_bg(self, s):
        ui.draw_background(s, self.width, self.height)
        for (x, y, b, r) in self._stars:
            tw = 0.6 + 0.4 * math.sin(self.anim_t * 1.5 + x)
            col = ui.mix(ui.BG_TOP, ui.TEXT, 0.2 + 0.5 * b * tw)
            pygame.draw.circle(s, col, (x, y), max(1, int(r)))

    def _tag_color(self, tag):
        """Theme-Farbe für den Level-Typ (zur Zeichenzeit ausgewertet)."""
        return {"Normal": ui.TEXT_DIM, "Schwer": ui.RED,
                "Spass": ui.GREEN, "Voll": ui.ACCENT}.get(tag, ui.TEXT_DIM)

    # ===== Setup-Screen =================================================

    def _build_setup_layout(self):
        cx = self.width // 2
        self.diff_rects = {}
        for i, name in enumerate(DIFF_ORDER):
            self.diff_rects[name] = pygame.Rect(cx - 165 + i * 112, 132, 102, 50)

        self.color_rects = []
        total = len(BALL_COLORS)
        sw = 44
        start = cx - (total * (sw + 6)) // 2
        for i in range(total):
            self.color_rects.append(pygame.Rect(start + i * (sw + 6), 262, sw, 44))

        self.start_rect = pygame.Rect(cx - 95, 352, 190, 52)
        self.mode_rect = pygame.Rect(cx - 165, 196, 330, 34)
        self.level_up_rect = pygame.Rect(self.width - 56, 18, 38, 26)
        self.level_down_rect = pygame.Rect(self.width - 56, 50, 38, 26)

    def _draw_setup(self):
        s = self.surface
        self._blit_bg(s)

        # Titel mit leichtem Glanz in der Akzentfarbe
        title = self.big_font.render("BREAKOUT", True, ui.TEXT)
        glow = self.big_font.render("BREAKOUT", True, self.accent)
        s.blit(glow, glow.get_rect(center=(self.width // 2 + 2, 58)))
        s.blit(title, title.get_rect(center=(self.width // 2, 56)))
        sub = self._small.render(i18n.t("bo.deluxe"), True, self.accent)
        s.blit(sub, sub.get_rect(center=(self.width // 2, 86)))

        # Level-Wahl (oben rechts)
        ld = self._level_def(self.start_level_index)
        s.blit(self._small.render(i18n.t("bo.startlevel"), True, ui.TEXT_DIM),
               (self.width - 170, 12))
        lvl_txt = self._mid.render(f"{self.start_level_index + 1:>2}/{NUM_LEVELS}", True, ui.TEXT)
        s.blit(lvl_txt, (self.width - 170, 36))
        tag_txt = self._small.render(i18n.t("bo.tag." + ld["tag"]), True,
                                     self._tag_color(ld["tag"]))
        s.blit(tag_txt, (self.width - 170, 62))
        for r, sym in ((self.level_up_rect, "+"), (self.level_down_rect, "-")):
            self._panel(s, r, ui.BTN)
            t = self._mid.render(sym, True, ui.TEXT)
            s.blit(t, t.get_rect(center=r.center))

        s.blit(self._mid.render(i18n.t("bo.difficulty"), True, ui.TEXT_DIM),
               (self.width // 2 - 165, 104))
        for name, r in self.diff_rects.items():
            aktiv = (name == self.diff_name)
            self._panel(s, r, ui.BTN_SEL if aktiv else ui.BTN,
                        border=self.accent if aktiv else ui.BORDER)
            t = self._mid.render(i18n.t("bo.diff." + name.lower()), True, ui.TEXT)
            s.blit(t, t.get_rect(center=r.center))

        # Aufbau-Umschalter
        voll = (self.level_mode == "Voll")
        self._panel(s, self.mode_rect, ui.BTN_SEL if voll else ui.BTN)
        mt = self._small.render(
            i18n.t("bo.build_full") if voll else i18n.t("bo.build_std"),
            True, ui.TEXT)
        s.blit(mt, mt.get_rect(center=self.mode_rect.center))

        s.blit(self._mid.render(i18n.t("bo.ballcolor"), True, ui.TEXT_DIM),
               (self.width // 2 - 165, 234))
        for i, r in enumerate(self.color_rects):
            pygame.draw.rect(s, BALL_COLORS[i][1], r, border_radius=8)
            if i == self.color_index:
                pygame.draw.rect(s, ui.TEXT, r.inflate(8, 8), 3, border_radius=10)

        # Start-Knopf (pulsierend)
        col = ui.mix(ui.GREEN, (255, 255, 255), ui.pulse(4, 0.0, 0.22))
        pygame.draw.rect(s, col, self.start_rect, border_radius=12)
        pygame.draw.rect(s, ui.TEXT, self.start_rect, 2, border_radius=12)
        st = self._mid.render(i18n.t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))

        hint = self._tiny.render(i18n.t("bo.setup_hint"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(self.width // 2, 428)))
        leg = self._tiny.render(i18n.t("bo.legend"), True, ui.TEXT_FAINT)
        s.blit(leg, leg.get_rect(center=(self.width // 2, 448)))

    def _panel(self, s, rect, fill, border=None):
        pygame.draw.rect(s, fill, rect, border_radius=8)
        pygame.draw.rect(s, border or ui.BORDER, rect, 1, border_radius=8)

    def _handle_setup_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("1", "2", "3"):
                self.diff_name = DIFF_ORDER[int(event.key) - 1]
            elif event.key in ("Left", "a"):
                self.color_index = (self.color_index - 1) % len(BALL_COLORS)
            elif event.key in ("Right", "d"):
                self.color_index = (self.color_index + 1) % len(BALL_COLORS)
            elif event.key in ("Up", "w"):
                self.start_level_index = min(NUM_LEVELS - 1, self.start_level_index + 1)
            elif event.key in ("Down", "s"):
                self.start_level_index = max(0, self.start_level_index - 1)
            elif event.key in ("m", "M"):
                self._toggle_mode()
            elif event.key in ("Return", "space"):
                self.play_sound("select")
                self._start_run()
        elif event.kind == InputEvent.MOUSEDOWN:
            p = event.pos
            for name, r in self.diff_rects.items():
                if r.collidepoint(p):
                    self.diff_name = name
                    self.play_sound("click")
            for i, r in enumerate(self.color_rects):
                if r.collidepoint(p):
                    self.color_index = i
                    self.play_sound("click")
            if self.level_up_rect.collidepoint(p):
                self.start_level_index = min(NUM_LEVELS - 1, self.start_level_index + 1)
            elif self.level_down_rect.collidepoint(p):
                self.start_level_index = max(0, self.start_level_index - 1)
            if self.mode_rect.collidepoint(p):
                self._toggle_mode()
            if self.start_rect.collidepoint(p):
                self.play_sound("select")
                self._start_run()

    def _toggle_mode(self):
        self.level_mode = "Voll" if self.level_mode == "Standard" else "Standard"
        self.play_sound("click")

    def _level_def(self, index):
        if self.level_mode == "Voll":
            base = min(3, 1 + (index + 1) // 7)
            return dict(tag="Voll", pat="full", rows=9, cols=18, base=base,
                        drop=1.4, spec=0.12)
        return LEVEL_DEFS[index]

    # ===== Spiel starten / Level laden ==================================

    def _start_run(self):
        self.cfg = DIFFICULTIES[self.diff_name]
        self.ball_color = BALL_COLORS[self.color_index][1]
        self.score = 0
        self.lives = self.cfg["lives"]
        self.level_index = self.start_level_index
        self.paddle_w = self.cfg["paddle"]
        self.won = False
        self.game_over = False
        self.combo = 0
        self.mult = 1
        self.state = PLAY
        self._load_level()

    def _load_level(self):
        self.level_def = self._level_def(self.level_index)
        self.bricks = self._make_level(self.level_def)
        self.powerups = []
        self.particles = []
        self.lasers = []
        self.floaters = []
        self.fx = dict(laser=0.0, fire=0.0, sticky=0.0, shield=0.0)
        self.laser_cd = 0.0
        self.intro_timer = 2.2
        self.combo = 0
        self.mult = 1
        self._reset_paddle_ball()

    def _reset_paddle_ball(self):
        self.paddle_x = self.width / 2 - self.paddle_w / 2
        self.paddle_y = self.height - 36
        self.move_dir = 0
        spd = self.cfg["ball_speed"]
        b = Ball(self.width / 2, self.paddle_y - BALL_R - 1, spd * 0.5, -spd)
        self.balls = [b]
        self.ball_hängt = True
        self.combo = 0
        self.mult = 1

    def _make_level(self, d):
        """Erzeugt die Steine für ein Level anhand seiner Definition 'd'."""
        rows, cols, pat = d["rows"], d["cols"], d["pat"]
        base = max(1, min(3, d["base"] + self.cfg["hard_bonus"]))
        spec = d.get("spec", 0.0)

        gap = 4
        top = 62
        bw = (self.width - (cols + 1) * gap) // cols
        bh = 22

        bricks = []
        for r in range(rows):
            for c in range(cols):
                if not self._brick_da(pat, r, c, rows, cols):
                    continue
                extra = 1 if (r >= rows - 2 and base >= 2) else 0
                strength = min(3, base + extra)
                x = gap + c * (bw + gap)
                y = top + r * (bh + gap)
                rect = pygame.Rect(x, y, bw, bh)

                kind = Brick.NORMAL
                if random.random() < spec:
                    roll = random.random()
                    if roll < 0.30:
                        kind = Brick.STEEL
                    elif roll < 0.65:
                        kind = Brick.BOMB
                    else:
                        kind = Brick.GOLD
                        strength = 1
                bricks.append(Brick(rect, strength, kind))
        return bricks

    @staticmethod
    def _brick_da(pat, r, c, rows, cols):
        """Entscheidet, ob an (r,c) ein Stein steht (Muster)."""
        mitte = cols // 2
        if pat == "full":
            return True
        if pat == "checker":
            return (r + c) % 2 == 0
        if pat == "pyramid":
            return abs(c - mitte) <= r
        if pat == "columns":
            return c % 2 == 0
        if pat == "rows":
            return r % 2 == 0
        if pat == "diamond":
            return abs(c - mitte) + abs(r - rows // 2) <= max(mitte, rows // 2) - 1
        if pat == "border":
            return r == 0 or r == rows - 1 or c == 0 or c == cols - 1
        if pat == "cross":
            return abs(c - mitte) <= 1 or abs(r - rows // 2) <= 1
        if pat == "columns3":
            return c % 3 != 1
        if pat == "waves":
            # zwei sinusförmige Bänder
            band = (rows - 1) / 2.0 + (rows / 3.0) * math.sin(c * 0.9)
            return abs(r - band) < 1.2
        if pat == "rings":
            d = abs(c - mitte) + abs(r - rows // 2)
            return d % 3 != 1
        if pat == "random":
            return random.random() < 0.72
        if pat == "heart":
            # normierte Koordinaten, klassische Herz-Ungleichung
            x = (c - (cols - 1) / 2.0) / (cols / 2.6)
            y = ((rows - 1) / 2.0 - r) / (rows / 2.4) + 0.35
            v = (x * x + y * y - 1) ** 3 - x * x * (y ** 3)
            return v <= 0
        return True

    # ===== Eingabe ======================================================

    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup_event(event)
            return

        if self.state == OVER:
            if (event.kind == InputEvent.KEYDOWN and event.key in ("Return", "space")) \
               or event.kind == InputEvent.MOUSEDOWN:
                self.state = SETUP
                self.game_over = False
            return

        if self.state == PAUSE:
            if event.kind == InputEvent.KEYDOWN and event.key in ("p", "P", "Escape", "space"):
                self.state = PLAY
            return

        # PLAY
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Left", "a"):
                self.move_dir = -1
            elif event.key in ("Right", "d"):
                self.move_dir = 1
            elif event.key in ("p", "P", "Escape"):
                self.state = PAUSE
                self.play_sound("click")
            elif event.key == "space":
                self._launch_or_fire()
        elif event.kind == InputEvent.MOUSEMOVE:
            self.paddle_x = event.pos[0] - self.paddle_w / 2
            self.move_dir = 0
        elif event.kind == InputEvent.MOUSEDOWN:
            self._launch_or_fire()

    def _launch_or_fire(self):
        """Leertaste/Klick: haftenden Ball lösen und ggf. Laser feuern."""
        released = False
        if self.ball_hängt:
            self.ball_hängt = False
            released = True
        for b in self.balls:
            if b.stuck:
                b.stuck = False
                released = True
        if not released and self.fx["laser"] > 0:
            self._fire_laser()

    # ===== Logik ========================================================

    def update(self, dt):
        self.anim_t += dt
        if self.state != PLAY:
            self._update_particles(dt)     # Effekte laufen auch im Pause-Screen sanft aus
            return

        if self.intro_timer > 0:
            self.intro_timer = max(0.0, self.intro_timer - dt)

        # Effekt-Timer herunterzählen
        for k in self.fx:
            if self.fx[k] > 0:
                self.fx[k] = max(0.0, self.fx[k] - dt)

        # Schläger bewegen (Tastatur)
        if self.move_dir:
            self.paddle_x += self.move_dir * 560 * dt
        self.paddle_x = max(0, min(self.width - self.paddle_w, self.paddle_x))
        paddle_rect = pygame.Rect(self.paddle_x, self.paddle_y, self.paddle_w, PADDLE_H)

        # Haftende / hängende Bälle folgen dem Schläger
        for b in self.balls:
            if self.ball_hängt or b.stuck:
                off = 0 if self.ball_hängt else b.stuck_off
                b.x = self.paddle_x + self.paddle_w / 2 + off
                b.y = self.paddle_y - BALL_R - 1
        if not self.ball_hängt:
            self._update_balls(dt, paddle_rect)

        # Laser automatisch nachladen/feuern
        if self.fx["laser"] > 0:
            self.laser_cd -= dt
            if self.laser_cd <= 0:
                self._fire_laser()
                self.laser_cd = LASER_INTERVAL
        self._update_lasers(dt)
        self._update_powerups(dt, paddle_rect)
        self._update_particles(dt)
        self._update_floaters(dt)
        if self.shake > 0:
            self.shake = max(0.0, self.shake - 55 * dt)

        # Alle Bälle verloren?
        if not self.balls:
            self.lives -= 1
            self.play_sound("hit")
            self.shake = 10
            if self.lives <= 0:
                self._ende(gewonnen=False)
            else:
                self._reset_paddle_ball()
            return

        # Level geschafft? (Stahl zählt nicht)
        if not any(b.breakable for b in self.bricks):
            if self.level_index + 1 < NUM_LEVELS:
                self.level_index += 1
                self.play_sound("point")
                self._load_level()
            else:
                self._ende(gewonnen=True)

    def _update_balls(self, dt, paddle_rect):
        überlebende = []
        fire = self.fx["fire"] > 0
        shield = self.fx["shield"] > 0
        for b in self.balls:
            if b.stuck:
                überlebende.append(b)
                continue

            # Spur mitschreiben
            b.trail.append((b.x, b.y))
            if len(b.trail) > 7:
                b.trail.pop(0)

            b.x += b.vx * dt
            b.y += b.vy * dt

            # Wände
            if b.x - BALL_R <= 0:
                b.x = BALL_R
                b.vx = abs(b.vx)
            elif b.x + BALL_R >= self.width:
                b.x = self.width - BALL_R
                b.vx = -abs(b.vx)
            if b.y - BALL_R <= 0:
                b.y = BALL_R
                b.vy = abs(b.vy)

            # Schild (Auffangnetz) am unteren Rand
            if shield and b.vy > 0 and b.y + BALL_R >= self.height - 6:
                b.y = self.height - 6 - BALL_R
                b.vy = -abs(b.vy)
                self.play_sound("bounce")
                self._spawn_particles(b.x, self.height - 6, (120, 200, 255), 6)

            # Unten raus -> dieser Ball ist weg
            if b.y - BALL_R > self.height:
                continue

            ball_rect = pygame.Rect(b.x - BALL_R, b.y - BALL_R, BALL_R * 2, BALL_R * 2)

            # Schläger
            if ball_rect.colliderect(paddle_rect) and b.vy > 0:
                treff = (b.x - (self.paddle_x + self.paddle_w / 2)) / (self.paddle_w / 2)
                treff = max(-1, min(1, treff))
                speed = b.speed()
                winkel = treff * (math.pi / 3)
                b.vx = speed * math.sin(winkel)
                b.vy = -abs(speed * math.cos(winkel))
                b.y = self.paddle_y - BALL_R - 1
                self.play_sound("bounce")
                self.combo = 0            # Combo endet beim Schläger
                self.mult = 1
                if self.fx["sticky"] > 0:
                    b.stuck = True
                    b.stuck_off = treff * (self.paddle_w / 2 - BALL_R)

            # Steine
            self._ball_bricks(b, ball_rect, fire)
            überlebende.append(b)

        self.balls = überlebende

    def _ball_bricks(self, b, ball_rect, fire):
        # Kopie durchlaufen: _hit_brick/_explode entfernen Steine aus der Liste.
        for brick in list(self.bricks):
            if brick not in self.bricks:      # durch Kettenexplosion schon weg
                continue
            if not ball_rect.colliderect(brick.rect):
                continue
            rect = brick.rect

            # Stahl: immer abprallen (auch beim Feuerball), nie zerstören
            if brick.kind == Brick.STEEL:
                self._bounce(b, ball_rect, rect)
                self.play_sound("bounce")
                return

            if fire:
                # Feuerball durchschlägt Steine ohne abzuprallen
                self._hit_brick(brick, full=True)
                continue

            self._bounce(b, ball_rect, rect)
            self._hit_brick(brick, full=False)
            return

    @staticmethod
    def _bounce(b, ball_rect, rect):
        ux = min(ball_rect.right - rect.left, rect.right - ball_rect.left)
        uy = min(ball_rect.bottom - rect.top, rect.bottom - ball_rect.top)
        if ux < uy:
            b.vx = -b.vx
        else:
            b.vy = -b.vy

    def _hit_brick(self, brick, full):
        """Fügt einem Stein Schaden zu; zerstört ihn ggf. inkl. Effekten."""
        if brick not in self.bricks:
            return
        dmg = brick.strength if full else 1
        brick.strength -= dmg
        if brick.strength > 0:
            self.score += 5
            self.play_sound("eat")
            return
        # WICHTIG: erst entfernen, dann poppen - sonst erwischt die
        # Bomben-Kettenexplosion denselben Stein ein zweites Mal.
        self.bricks.remove(brick)
        self._pop_brick(brick, chain=True)

    def _pop_brick(self, brick, chain):
        """Stein zerstören: Punkte, Combo, Partikel, evtl. Kettenexplosion."""
        self.combo += 1
        self.mult = min(8, 1 + self.combo // 4)

        if brick.kind == Brick.GOLD:
            base = 60
            col = (255, 215, 90)
        elif brick.kind == Brick.BOMB:
            base = 25
            col = (255, 130, 70)
        else:
            base = 10
            col = STR_COLORS.get(max(1, brick.strength + 1), (200, 200, 200))

        pts = base * self.mult
        self.score += pts
        self._add_floater(brick.rect.centerx, brick.rect.centery, f"+{pts}", col)
        self._spawn_particles(brick.rect.centerx, brick.rect.centery, col, 10)
        self.play_sound("explode")
        self._maybe_drop(brick.rect.center)

        if brick.kind == Brick.BOMB and chain:
            self.play_sound("hit")
            self.shake = max(self.shake, 12)
            self._explode(brick.rect.center, radius=70)

    def _explode(self, center, radius):
        """Reisst alle Steine im Umkreis mit (Bomben-Kettenreaktion)."""
        cx, cy = center
        self._spawn_particles(cx, cy, (255, 150, 60), 22, spread=260)
        opfer = [b for b in self.bricks
                 if b.breakable and math.hypot(b.rect.centerx - cx,
                                               b.rect.centery - cy) <= radius]
        for b in opfer:
            if b in self.bricks:
                self.bricks.remove(b)
                self._pop_brick(b, chain=(b.kind == Brick.BOMB))

    def _fire_laser(self):
        """Feuert zwei Laserschüsse von den Schlägerenden nach oben."""
        y = self.paddle_y - 4
        self.lasers.append([self.paddle_x + 8, y])
        self.lasers.append([self.paddle_x + self.paddle_w - 8, y])
        self.play_sound("shoot")

    def _update_lasers(self, dt):
        bleibt = []
        for shot in self.lasers:
            shot[1] -= LASER_SPEED * dt
            if shot[1] < 0:
                continue
            r = pygame.Rect(int(shot[0]) - 2, int(shot[1]) - 10, 4, 12)
            getroffen = False
            for brick in self.bricks:
                if r.colliderect(brick.rect):
                    if brick.kind == Brick.STEEL:
                        getroffen = True     # Stahl schluckt den Schuss
                    else:
                        self._hit_brick(brick, full=False)
                        getroffen = True
                    break
            if not getroffen:
                bleibt.append(shot)
        self.lasers = bleibt

    def _maybe_drop(self, pos):
        chance = min(0.95, self.cfg["drop"] * self.level_def["drop"])
        if random.random() > chance:
            return
        gut = ["multi", "spread", "wide", "life", "slow",
               "laser", "fire", "sticky", "shield", "coin"]
        schlecht = ["shrink", "speed"]
        if random.random() < self.cfg["bad"]:
            kind = random.choice(schlecht)
        else:
            kind = random.choice(gut)
        self.powerups.append(PowerUp(pos[0], pos[1], kind))

    def _update_powerups(self, dt, paddle_rect):
        bleibt = []
        for p in self.powerups:
            # Fallposition als float führen (int-Abschneiden wäre je nach
            # Framerate unterschiedlich schnell).
            p.y += POWERUP_FALL * dt
            p.rect.centery = int(p.y)
            if p.rect.colliderect(paddle_rect):
                self._activate(p.kind)
            elif p.rect.top <= self.height:
                bleibt.append(p)
        self.powerups = bleibt

    def _activate(self, kind):
        label = PowerUp.INFO[kind][0]
        col = PowerUp.INFO[kind][1]
        self._add_floater(self.paddle_x + self.paddle_w / 2, self.paddle_y - 18, label, col)
        gut = PowerUp.INFO[kind][2]
        self.play_sound("point" if gut else "hit")

        if kind == "life":
            self.lives += 1
        elif kind == "wide":
            self.paddle_w = min(PADDLE_MAX, self.paddle_w + 30)
        elif kind == "shrink":
            self.paddle_w = max(PADDLE_MIN, self.paddle_w - 26)
        elif kind == "speed":
            for b in self.balls:
                b.scale_speed(1.25)
        elif kind == "slow":
            for b in self.balls:
                b.scale_speed(0.80)
        elif kind == "coin":
            self.score += 250
        elif kind in ("laser", "fire", "sticky", "shield"):
            self.fx[kind] = FX_DUR[kind]
            if kind == "laser":
                self.laser_cd = 0.0
        elif kind == "multi":
            neu = []
            for b in self.balls:
                if b.stuck:
                    continue
                if len(self.balls) + len(neu) < MAX_BALLS:
                    nb = Ball(b.x, b.y, -b.vx, b.vy)
                    neu.append(nb)
            self.balls.extend(neu)
        elif kind == "spread":
            aktive = [b for b in self.balls if not b.stuck] or self.balls
            if aktive:
                quelle = aktive[0]
                spd = quelle.speed() or self.cfg["ball_speed"]
                for da in (-0.5, 0.5):
                    if len(self.balls) < MAX_BALLS:
                        self.balls.append(
                            Ball(quelle.x, quelle.y,
                                 spd * math.sin(da), -abs(spd * math.cos(da))))

    # ----- Partikel / Popups -------------------------------------------
    def _spawn_particles(self, x, y, color, n, spread=180):
        for _ in range(n):
            ang = random.uniform(0, math.tau)
            spd = random.uniform(40, spread)
            self.particles.append([x, y,
                                   math.cos(ang) * spd, math.sin(ang) * spd,
                                   random.uniform(0.3, 0.7), color,
                                   random.uniform(2, 4)])

    def _update_particles(self, dt):
        bleibt = []
        for p in self.particles:
            p[0] += p[2] * dt
            p[1] += p[3] * dt
            p[3] += 260 * dt          # Schwerkraft
            p[4] -= dt
            if p[4] > 0:
                bleibt.append(p)
        self.particles = bleibt

    def _add_floater(self, x, y, text, color):
        self.floaters.append([x, y, text, color, 0.9])

    def _update_floaters(self, dt):
        bleibt = []
        for f in self.floaters:
            f[1] -= 34 * dt
            f[4] -= dt
            if f[4] > 0:
                bleibt.append(f)
        self.floaters = bleibt

    def _ende(self, gewonnen):
        self.won = gewonnen
        self.state = OVER
        self.game_over = True
        self.play_sound("win" if gewonnen else "gameover")
        self.rumble(220)

    # ===== Zeichnen =====================================================

    def draw(self):
        if self.state == SETUP:
            self._draw_setup()
            return

        s = self._get_scene()
        self._blit_bg(s)

        # Schild-Netz
        if self.state in (PLAY, PAUSE) and self.fx["shield"] > 0:
            netz = pygame.Surface((self.width, 6), pygame.SRCALPHA)
            a = 120 + int(80 * math.sin(self.anim_t * 6))
            netz.fill((110, 190, 255, max(60, a)))
            s.blit(netz, (0, self.height - 6))

        # Steine
        for brick in self.bricks:
            self._draw_brick(s, brick)

        # Partikel
        for p in self.particles:
            a = max(0, min(255, int(255 * (p[4] / 0.7))))
            col = (*p[5], a)
            surf = pygame.Surface((int(p[6] * 2), int(p[6] * 2)), pygame.SRCALPHA)
            pygame.draw.circle(surf, col, (int(p[6]), int(p[6])), int(p[6]))
            s.blit(surf, (p[0] - p[6], p[1] - p[6]))

        # Ball-Spuren
        for b in self.balls:
            for j, (tx, ty) in enumerate(b.trail):
                a = int(90 * (j + 1) / len(b.trail))
                surf = pygame.Surface((BALL_R * 2, BALL_R * 2), pygame.SRCALPHA)
                pygame.draw.circle(surf, (*self.ball_color, a), (BALL_R, BALL_R),
                                   max(2, int(BALL_R * (j + 1) / len(b.trail))))
                s.blit(surf, (tx - BALL_R, ty - BALL_R))

        # Laser
        for shot in self.lasers:
            pygame.draw.rect(s, (255, 90, 90),
                             (int(shot[0]) - 2, int(shot[1]) - 10, 4, 12), border_radius=2)

        # Schläger
        self._draw_paddle(s)

        # Bälle (Feuerball andersfarbig)
        fire = self.fx["fire"] > 0
        for b in self.balls:
            col = (255, 150, 40) if fire else self.ball_color
            if fire:
                glow = pygame.Surface((BALL_R * 4, BALL_R * 4), pygame.SRCALPHA)
                pygame.draw.circle(glow, (255, 120, 30, 90), (BALL_R * 2, BALL_R * 2), BALL_R * 2)
                s.blit(glow, (b.x - BALL_R * 2, b.y - BALL_R * 2))
            pygame.draw.circle(s, col, (int(b.x), int(b.y)), BALL_R)
            pygame.draw.circle(s, (255, 255, 255), (int(b.x - 2), int(b.y - 2)), 2)

        # Power-ups
        for p in self.powerups:
            self._draw_powerup(s, p)

        # Punkte-Popups
        for f in self.floaters:
            a = max(0, min(255, int(255 * (f[4] / 0.9))))
            img = self._small.render(f[2], True, f[3])
            img.set_alpha(a)
            s.blit(img, img.get_rect(center=(int(f[0]), int(f[1]))))

        self._draw_hud(s)

        if self.ball_hängt and self.intro_timer <= 0:
            img = self.font.render(i18n.t("bo.start_ball"), True, ui.TEXT)
            s.blit(img, img.get_rect(center=(self.width // 2, self.height // 2 + 60)))

        # Level-Intro-Banner
        if self.intro_timer > 0:
            self._draw_intro(s)

        # Szene mit Screen-Shake auf die echte Fläche übertragen
        if self.shake > 0.5:
            dx = random.uniform(-self.shake, self.shake)
            dy = random.uniform(-self.shake, self.shake)
            self.surface.fill(ui.BG_BOTTOM)
            self.surface.blit(s, (dx, dy))
        else:
            self.surface.blit(s, (0, 0))

        if self.state == PAUSE:
            self._draw_pause(self.surface)
        if self.state == OVER:
            self._draw_over(self.surface)

    def _draw_paddle(self, s):
        rect = pygame.Rect(int(self.paddle_x), int(self.paddle_y), int(self.paddle_w), PADDLE_H)
        base = ui.TEXT                      # hell im Dark-, dunkel im Light-Theme
        if self.fx["sticky"] > 0:
            base = (150, 240, 160)
        pygame.draw.rect(s, base, rect, border_radius=8)
        pygame.draw.rect(s, self.accent, rect, 2, border_radius=8)
        # Laser-Kanonen
        if self.fx["laser"] > 0:
            for gx in (rect.left + 8, rect.right - 8):
                pygame.draw.rect(s, (255, 90, 90), (gx - 3, rect.top - 8, 6, 8), border_radius=2)

    def _draw_brick(self, s, brick):
        rect = brick.rect
        if brick.kind == Brick.STEEL:
            pygame.draw.rect(s, (120, 128, 140), rect, border_radius=4)
            pygame.draw.rect(s, (170, 178, 190), rect, 2, border_radius=4)
            for bx, by in ((rect.left + 5, rect.top + 5), (rect.right - 5, rect.top + 5),
                           (rect.left + 5, rect.bottom - 5), (rect.right - 5, rect.bottom - 5)):
                pygame.draw.circle(s, (80, 86, 96), (bx, by), 2)
            return
        if brick.kind == Brick.GOLD:
            pygame.draw.rect(s, (245, 200, 70), rect, border_radius=4)
            pygame.draw.rect(s, (255, 240, 160), rect, 2, border_radius=4)
            t = self._small.render("$", True, (120, 80, 10))
            s.blit(t, t.get_rect(center=rect.center))
            return
        if brick.kind == Brick.BOMB:
            pygame.draw.rect(s, (70, 40, 40), rect, border_radius=4)
            pygame.draw.rect(s, (230, 90, 60), rect, 2, border_radius=4)
            pygame.draw.circle(s, (230, 90, 60), rect.center, 5)
            return
        col = STR_COLORS.get(brick.strength, (200, 200, 200))
        pygame.draw.rect(s, col, rect, border_radius=4)
        # oberer Glanzstreifen
        hi = pygame.Surface((rect.width - 4, 4), pygame.SRCALPHA)
        hi.fill((255, 255, 255, 55))
        s.blit(hi, (rect.left + 2, rect.top + 2))
        if brick.strength > 1:
            n = self._small.render(str(brick.strength), True, (20, 20, 20))
            s.blit(n, n.get_rect(center=rect.center))

    def _draw_powerup(self, s, p):
        label, farbe, gut = PowerUp.INFO[p.kind]
        bob = int(2 * math.sin(self.anim_t * 6 + p.phase))
        r = p.rect.move(0, bob)
        glow = pygame.Surface((r.width + 8, r.height + 8), pygame.SRCALPHA)
        pygame.draw.rect(glow, (*farbe, 70), glow.get_rect(), border_radius=8)
        s.blit(glow, (r.left - 4, r.top - 4))
        pygame.draw.rect(s, farbe, r, border_radius=6)
        pygame.draw.rect(s, (20, 20, 30) if gut else (30, 10, 10), r, 2, border_radius=6)
        t = self._small.render(label, True, (20, 20, 20))
        s.blit(t, t.get_rect(center=r.center))

    def _draw_hud(self, s):
        # Halbtransparenter Panel-Streifen oben (im Theme-Ton)
        bar = pygame.Surface((self.width, 34), pygame.SRCALPHA)
        bar.fill((*ui.PANEL, 150))
        s.blit(bar, (0, 0))
        pygame.draw.line(s, ui.BORDER, (0, 33), (self.width, 33))

        s.blit(self.font.render(i18n.t("common.points", score=self.score), True,
                                ui.TEXT), (10, 6))
        tag = self.level_def["tag"]
        mid = self._small.render(
            i18n.t("bo.hud", diff=i18n.t("bo.diff." + self.diff_name.lower()),
                   level=self.level_index + 1,
                   total=NUM_LEVELS, tag=i18n.t("bo.tag." + tag)),
            True, self._tag_color(tag))
        s.blit(mid, mid.get_rect(midtop=(self.width // 2, 8)))

        # Leben als Herzen
        for i in range(min(self.lives, 8)):
            hx = self.width - 20 - i * 20
            pygame.draw.circle(s, (230, 80, 110), (hx - 3, 14), 4)
            pygame.draw.circle(s, (230, 80, 110), (hx + 3, 14), 4)
            pygame.draw.polygon(s, (230, 80, 110),
                                [(hx - 6, 15), (hx + 6, 15), (hx, 23)])
        if self.lives > 8:
            s.blit(self._tiny.render(f"x{self.lives}", True, ui.TEXT),
                   (self.width - 20 - 8 * 20 - 24, 8))

        # Combo-Anzeige
        if self.combo > 1:
            ct = self._mid.render(i18n.t("bo.combo", mult=self.mult), True, ui.GOLD)
            s.blit(ct, ct.get_rect(midtop=(self.width // 2, 38)))

        # Aktive Effekte mit Restzeit-Balken (unten links)
        y = self.height - 22
        order = [("laser", "L", (255, 90, 90)), ("fire", "F", (255, 170, 60)),
                 ("sticky", "G", (150, 255, 150)), ("shield", "U", (120, 200, 255))]
        x = 8
        for key, lab, col in order:
            if self.fx[key] <= 0:
                continue
            frac = self.fx[key] / FX_DUR[key]
            pygame.draw.rect(s, ui.PANEL, (x, y, 54, 14), border_radius=4)
            pygame.draw.rect(s, col, (x, y, int(54 * frac), 14), border_radius=4)
            s.blit(self._tiny.render(lab, True, (10, 10, 10)), (x + 4, y))
            x += 60

    def _draw_intro(self, s):
        a = min(1.0, self.intro_timer / 0.4) if self.intro_timer < 0.4 else 1.0
        alpha = int(220 * min(1.0, a))
        band = pygame.Surface((self.width, 90), pygame.SRCALPHA)
        band.fill((*ui.PANEL, int(alpha * 0.7)))
        band.fill((*self.accent, alpha), (0, 0, self.width, 2))
        band.fill((*self.accent, alpha), (0, 88, self.width, 2))
        s.blit(band, (0, self.height // 2 - 45))
        tag = self.level_def["tag"]
        big = self.big_font.render(
            i18n.t("bo.level_intro", level=self.level_index + 1), True, ui.TEXT)
        big.set_alpha(alpha)
        s.blit(big, big.get_rect(center=(self.width // 2, self.height // 2 - 10)))
        sub = self._mid.render(f"[{i18n.t('bo.tag.' + tag)}]", True,
                               self._tag_color(tag))
        sub.set_alpha(alpha)
        s.blit(sub, sub.get_rect(center=(self.width // 2, self.height // 2 + 26)))

    def _draw_pause(self, s):
        ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        ov.fill((0, 0, 0, 170))
        s.blit(ov, (0, 0))
        self.draw_center_text(i18n.t("app.pause"), self.big_font, ui.TEXT, -10)
        self.draw_center_text(i18n.t("bo.pause_resume"), self.font, ui.TEXT_DIM, 40)

    def _draw_over(self, s):
        ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        ov.fill((0, 0, 0, 170))
        s.blit(ov, (0, 0))
        if self.won:
            self.draw_center_text(i18n.t("bo.cleared"), self.big_font,
                                  ui.GREEN, -30)
        else:
            self.draw_center_text(i18n.t("common.game_over"), self.big_font,
                                  ui.RED, -30)
        self.draw_center_text(i18n.t("common.points", score=self.score),
                              self.font, ui.TEXT, 18)
        self.draw_center_text(i18n.t("bo.back_to_select"), self.font,
                              ui.TEXT_DIM, 52)
