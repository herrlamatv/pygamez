# -*- coding: utf-8 -*-
"""
doodle.py
=========
Doodle Jump - Klon mit vielen Extras (Einzelspieler).

Features
--------
- Der Doodler springt automatisch, sobald er auf einer Plattform landet;
  gesteuert wird nur links/rechts (mit Trägheit), die Ränder sind offen
  (Wrap-around).  Die Kamera scrollt mit dem Aufstieg nach oben.
- PLATTFORM-TYPEN:
    * grün   - normal
    * blau   - bewegt sich hin und her
    * braun  - zerbricht beim Betreten (man fällt hindurch)
    * weiß   - verschwindet nach einem Sprung
- SPRUNGFEDERN geben einen Superhüpfer; der PROPELLER-HUT trägt einen kurz
  automatisch nach oben (und macht unverwundbar).
- MONSTER: Berührung ist tödlich - aber man kann sie mit Pfeil hoch / Leertaste
  von unten ABSCHIESSEN (Extrapunkte).
- Schwierigkeit (Leicht/Normal/Schwer): Plattform-Abstand und Anteil an
  beweglichen/zerbrechlichen Plattformen sowie Monstern; wird mit der Höhe härter.
- Punkte = erreichte Höhe.  Partikel, Feder-Animation, Highscore.

Steuerung: links/rechts (A/D oder Pfeile) = bewegen,  Pfeil hoch / Leertaste = schießen.
Enter = neu, S = Setup.
"""

import math
import random
import pygame

import highscore
import settings as settings_mod
import ui
from game_base import Game, InputEvent
from i18n import t

GRAV = 1350.0
JUMP_V = -720.0            # normaler Absprung
SPRING_V = -1250.0         # Sprungfeder
PROP_V = -560.0            # Propeller-Steiggeschwindigkeit
MOVE_ACC = 1500.0
MOVE_MAX = 430.0
MOVE_FRICTION = 0.86
SHOOT_CD = 0.28
PROP_TIME = 2.6

# Schwierigkeit: Basis-Plattformabstand (px), Anteil beweglich/zerbrechlich, Monster-Chance
DIFFS = [
    dict(key="easy", gap=62, move=0.12, brittle=0.10, monster=0.05),
    dict(key="normal", gap=74, move=0.20, brittle=0.18, monster=0.09),
    dict(key="hard", gap=86, move=0.28, brittle=0.24, monster=0.14),
]

# Identitätsfarben des Spielfelds (Papier-Look) - bewusst NICHT aus dem UI-Theme.
COL_BG_TOP = (222, 236, 250)
COL_BG_BOT = (238, 246, 252)
COL_GRID = (220, 228, 240)
COL_DOODLE = (120, 210, 90)
COL_DOODLE_DARK = (70, 150, 55)

PLAT_COLORS = {
    "normal": (110, 205, 90), "move": (90, 160, 235),
    "brittle": (190, 130, 80), "vanish": (235, 235, 240),
}

SETUP, PLAY, GAMEOVER = "setup", "play", "gameover"


class _Plat:
    __slots__ = ("x", "y", "w", "kind", "vx", "spring", "prop", "dead", "used")

    def __init__(self, x, y, w, kind):
        self.x = x
        self.y = y
        self.w = w
        self.kind = kind
        self.vx = 0.0
        self.spring = False
        self.prop = False
        self.dead = False
        self.used = False


class DoodleGame(Game):
    name = "Doodle Jump"
    highscore_key = "doodle"
    supports_multiplayer = False

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        dj = self.settings.get("doodle", {}) if isinstance(self.settings, dict) else {}
        self.diff = max(0, min(2, int(dj.get("difficulty", 1))))

        self.highscore = highscore.load_highscores().get(self.highscore_key, 0)
        self.anim_t = 0.0

        self._bg_cache = None
        self._ui_bg_cache = None
        self._dim_cache = None
        self._apply_layout()

        self._new_game()
        self.state = SETUP

    def _make_fonts(self):
        """Schriftgrößen aus der Fensterhöhe ableiten (Theme-Schrift)."""
        h = self.height
        self.font = ui.font(max(18, min(26, h // 26)))
        self.big_font = ui.font(max(32, min(52, h // 12)), bold=True)
        self._small = ui.font(max(14, min(20, h // 32)))
        self._tiny = ui.font(max(12, min(16, h // 42)))
        self._huge = ui.font(max(26, h // 11), bold=True)

    def _apply_layout(self):
        """Layout-Größen aus width/height berechnen (auch nach Resize)."""
        self._make_fonts()
        self.plat_w = max(46, int(self.width * 0.16))
        self.dr = max(12, int(self.height * 0.03))
        self._build_setup_layout()

    def on_surface_changed(self):
        """Nach Auflösungswechsel: Schriften, Layout und Caches neu aufbauen."""
        self._apply_layout()
        self._bg_cache = None
        self._ui_bg_cache = None
        self._dim_cache = None
        # Doodler im gültigen Bereich halten (Wrap-Bereich = Breite)
        self.x = max(0.0, min(float(self.width), self.x))

    def _new_game(self):
        self.score = 0
        self.game_over = False
        self.cam_y = 0.0
        self.x = self.width / 2
        self.y = self.height * 0.7
        self.py_prev = self.y
        self.vx = 0.0
        self.vy = JUMP_V
        self.face = 1
        self.shoot_cd = 0.0
        self.shoot_pose = 0.0
        self.prop_t = 0.0
        self.start_y = self.y
        self.max_rise = 0.0
        self.press = set()
        self.platforms = []
        self.monsters = []       # dicts: x,y,vx,kind
        self.bullets = []        # dicts: x,y
        self.particles = []
        self.new_best = False
        self._over_t = 0.0
        self._prev_kind = ""
        # Startplattform + erste Plattformen aufbauen
        self.platforms.append(_Plat(self.width / 2 - self.plat_w / 2,
                                    self.height * 0.75, self.plat_w, "normal"))
        self.gen_y = self.height * 0.75
        self._fill_platforms()
        self.state = PLAY

    def _diff(self):
        return DIFFS[self.diff]

    def _difficulty_scale(self):
        # steigt mit der Höhe (0..1): Plattformen weiter auseinander, mehr Hazards
        return min(1.0, self.score / 4000.0)

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(440, self.width - 60)
        ph = max(56, min(72, int(self.height * 0.14)))
        y0 = max(140, int(self.height * 0.34))
        self.diff_panel = pygame.Rect(cx - bw // 2, y0, bw, ph)
        self.diff_left = pygame.Rect(self.diff_panel.left, y0, 44, ph)
        self.diff_right = pygame.Rect(self.diff_panel.right - 44, y0, 44, ph)
        self.start_rect = pygame.Rect(cx - 95, self.diff_panel.bottom + 26, 190, 52)

    def _save(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("doodle", {})[key] = value
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
    def _is_left(self, key):
        return self.is_action(key, "left") or key == "Left"

    def _is_right(self, key):
        return self.is_action(key, "right") or key == "Right"

    def _is_shoot(self, key):
        return self.is_action(key, "up") or key in ("Up", "space", "w", "W")

    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.state == GAMEOVER:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._new_game()
                    self.play_sound("click")
                elif event.key in ("s", "S"):
                    self.state = SETUP
                    self.play_sound("click")
            elif event.kind == InputEvent.MOUSEDOWN and \
                    self.anim_t - self._over_t > 0.5:
                # Klick startet neu - mit kurzer Sperre gegen Durchklicken
                self._new_game()
                self.play_sound("click")
            return
        if event.kind == InputEvent.KEYDOWN:
            if self._is_left(event.key):
                self.press.add("l")
            elif self._is_right(event.key):
                self.press.add("r")
            elif self._is_shoot(event.key):
                self._shoot()
        elif event.kind == InputEvent.KEYUP:
            if self._is_left(event.key):
                self.press.discard("l")
            elif self._is_right(event.key):
                self.press.discard("r")

    def _shoot(self):
        if self.shoot_cd > 0:
            return
        self.shoot_cd = SHOOT_CD
        self.shoot_pose = 0.25
        self.bullets.append(dict(x=self.x, y=self.y - self.dr, vy=-820.0))
        self.play_sound("shoot")

    # ===================================================== Spiellogik
    def update(self, dt):
        self.anim_t += dt
        self._update_particles(dt)
        if self.state != PLAY:
            return
        self.shoot_cd = max(0.0, self.shoot_cd - dt)
        self.shoot_pose = max(0.0, self.shoot_pose - dt)

        # Horizontale Steuerung
        if "l" in self.press:
            self.vx -= MOVE_ACC * dt
            self.face = -1
        if "r" in self.press:
            self.vx += MOVE_ACC * dt
            self.face = 1
        if not self.press:
            self.vx *= MOVE_FRICTION
        self.vx = max(-MOVE_MAX, min(MOVE_MAX, self.vx))
        self.x += self.vx * dt
        if self.x < 0:
            self.x += self.width          # Wrap-around
        elif self.x > self.width:
            self.x -= self.width

        # Propeller trägt nach oben
        self.py_prev = self.y             # für die "swept"-Kollision merken
        if self.prop_t > 0:
            self.prop_t -= dt
            self.vy = PROP_V
        else:
            self.vy += GRAV * dt
        self.y += self.vy * dt

        if self.vy > 0:                   # nur beim Fallen auf Plattformen prüfen
            self._platform_collisions()

        self._update_platforms(dt)
        self._update_monsters(dt)
        self._update_bullets(dt)
        self._camera_and_score()
        self._fill_platforms()
        self._cull()

        # Tod: unter den Bildschirm gefallen
        if self.y - self.cam_y > self.height + self.dr:
            self._die()

    def _platform_collisions(self):
        """Swept-Kollision: prüft, ob die Füße die Plattform-Oberkante seit dem
        letzten Frame ÜBERQUERT haben - so tunnelt der Doodler auch bei hohem
        Tempo (Feder!) oder niedriger/schwankender Framerate nicht hindurch."""
        prev_feet = self.py_prev + self.dr
        feet = self.y + self.dr
        for p in self.platforms:
            if p.dead:
                continue
            if not (p.x <= self.x <= p.x + p.w):
                continue
            # Oberkante der Plattform lag zwischen altem und neuem Fußpunkt?
            if not (prev_feet <= p.y + 6 and feet >= p.y - 2):
                continue
            if p.kind == "brittle":       # zerbricht -> kein Absprung, weiterfallen
                p.dead = True
                self._sparkle(self.x, p.y, PLAT_COLORS["brittle"])
                self.play_sound("hit")
                continue
            self.y = p.y - self.dr        # auf die Plattform setzen (kein Re-Trigger)
            if p.prop:
                self.prop_t = PROP_TIME
                p.prop = False
                self.play_sound("powerup")
                self._sparkle(self.x, p.y, (120, 200, 255))
            elif p.spring:
                self.vy = SPRING_V
                self.play_sound("powerup")
                self._sparkle(self.x, p.y, (255, 220, 120))
            else:
                self.vy = JUMP_V
                self.play_sound("bounce")
            if p.kind == "vanish" and not p.used:
                p.used = True
                p.dead = True
            break

    def _update_platforms(self, dt):
        for p in self.platforms:
            if p.kind == "move" and not p.dead:
                p.x += p.vx * dt
                if p.x < 0 or p.x + p.w > self.width:
                    p.vx = -p.vx
                    p.x = max(0, min(self.width - p.w, p.x))

    def _update_monsters(self, dt):
        for m in self.monsters:
            m["x"] += m["vx"] * dt
            if m["x"] < 20 or m["x"] > self.width - 20:
                m["vx"] = -m["vx"]
            # Kollision mit Doodler
            if self.prop_t <= 0 and \
                    math.hypot(self.x - m["x"], self.y - m["y"]) < self.dr + 16:
                self._die()
                return
            elif self.prop_t > 0 and \
                    math.hypot(self.x - m["x"], self.y - m["y"]) < self.dr + 16:
                m["hp"] = 0
                self.score += 200
                self._sparkle(m["x"], m["y"], (255, 120, 120))
                self.play_sound("explode")
        self.monsters = [m for m in self.monsters if m.get("hp", 1) > 0]

    def _update_bullets(self, dt):
        for b in self.bullets:
            b["y"] += b["vy"] * dt
            for m in self.monsters:
                if m.get("hp", 1) > 0 and \
                        math.hypot(b["x"] - m["x"], b["y"] - m["y"]) < 18:
                    m["hp"] = 0
                    b["y"] = -99999
                    self.score += 200
                    self._sparkle(m["x"], m["y"], (255, 120, 120))
                    self.play_sound("explode")
        self.bullets = [b for b in self.bullets if b["y"] - self.cam_y > -30]
        self.monsters = [m for m in self.monsters if m.get("hp", 1) > 0]

    def _camera_and_score(self):
        thresh = self.cam_y + self.height * 0.42
        if self.y < thresh:
            self.cam_y = self.y - self.height * 0.42
        rise = self.start_y - (self.cam_y + self.height * 0.42)
        self.max_rise = max(self.max_rise, rise)
        self.score = max(self.score, int(self.max_rise / 4))

    def _fill_platforms(self):
        """Erzeugt Plattformen nach oben, bis genug über der Kamera liegen.

        WICHTIG (Erreichbarkeit): Das "Gerüst" besteht nur aus bespringbaren
        Plattformen (normal/beweglich/verschwindend) mit Abständen unterhalb
        der maximalen Sprunghöhe. Zerbrechliche Plattformen ERSETZEN keinen
        Gerüst-Schritt, sondern werden zusätzlich dazwischen gestreut - sonst
        entstehen zwei Schritte ohne Absprung-Möglichkeit und es gibt
        stellenweise "keine Blöcke mehr" (unvermeidbarer Tod).
        """
        d = self._diff()
        scale = self._difficulty_scale()
        gap = d["gap"] + scale * 40
        while self.gen_y > self.cam_y - self.height * 0.3:
            prev_y = self.gen_y
            self.gen_y -= random.uniform(gap * 0.7, gap * 1.15)
            x = random.uniform(0, self.width - self.plat_w)
            kind = self._pick_kind(d, scale)
            # Nie zwei verschwindende Plattformen direkt hintereinander -
            # sonst fehlt nach deren Benutzung ein ganzes Stück Gerüst.
            if kind == "vanish" and self._prev_kind == "vanish":
                kind = "normal"
            self._prev_kind = kind
            p = _Plat(x, self.gen_y, self.plat_w, kind)
            if kind == "move":
                p.vx = random.choice((-1, 1)) * random.uniform(60, 110)
            # Feder / Propeller auf stabilen Plattformen
            if kind in ("normal", "move"):
                rr = random.random()
                if rr < 0.06:
                    p.prop = True
                elif rr < 0.16:
                    p.spring = True
            self.platforms.append(p)
            # Zerbrechliche Plattform als zusätzliche Falle ZWISCHEN den
            # Gerüst-Plattformen (nie als Ersatz eines Schritts).
            pbrit = d["brittle"] + scale * 0.10
            if prev_y - self.gen_y > 40 and random.random() < pbrit:
                self.platforms.append(_Plat(
                    random.uniform(0, self.width - self.plat_w),
                    random.uniform(self.gen_y + 14, prev_y - 14),
                    self.plat_w, "brittle"))
            # Monster gelegentlich zwischen die Plattformen
            if random.random() < d["monster"] * (0.5 + scale):
                self.monsters.append(dict(
                    x=random.uniform(40, self.width - 40),
                    y=self.gen_y - random.uniform(20, 40),
                    vx=random.choice((-1, 1)) * random.uniform(20, 55),
                    hp=1, kind=random.randint(0, 2),
                    bob=random.uniform(0, math.tau)))

    def _pick_kind(self, d, scale):
        """Wählt die Art einer GERÜST-Plattform - alle sind bespringbar
        (brüchige Plattformen werden separat als Extras eingestreut)."""
        r = random.random()
        pmove = d["move"] + scale * 0.12
        pvan = 0.06 + scale * 0.06
        if r < pvan:
            return "vanish"
        if r < pvan + pmove:
            return "move"
        return "normal"

    def _cull(self):
        limit = self.cam_y + self.height + 60
        self.platforms = [p for p in self.platforms
                          if p.y < limit and not (p.dead and p.y > self.cam_y + self.height)]
        self.monsters = [m for m in self.monsters if m["y"] < limit]

    def _die(self):
        if self.state != PLAY:
            return
        self.state = GAMEOVER
        self.game_over = True
        self.new_best = self.score > self.highscore
        self.highscore = max(self.highscore, self.score)
        self._over_t = self.anim_t
        self.play_sound("gameover")
        self.rumble(220)
        self._sparkle(self.x, self.y, COL_DOODLE, 16)

    # ----- Effekte -------------------------------------------------------
    def _sparkle(self, x, y, col, n=9):
        for _ in range(n):
            a = random.uniform(0, math.tau)
            sp = random.uniform(40, 160)
            self.particles.append([x, y, math.cos(a) * sp, math.sin(a) * sp,
                                   random.uniform(0.25, 0.5), col])

    def _update_particles(self, dt):
        rest = []
        for p in self.particles:
            p[0] += p[2] * dt
            p[1] += p[3] * dt
            p[3] += 260 * dt
            p[4] -= dt
            if p[4] > 0:
                rest.append(p)
        self.particles = rest

    # ----- Theme-UI-Helfer ----------------------------------------------
    def _ui_bg(self):
        """Gecachter Theme-Hintergrund für den Setup-Screen."""
        key = (self.width, self.height, ui.BG_TOP, ui.BG_BOTTOM)
        if self._ui_bg_cache is None or self._ui_bg_cache[0] != key:
            surf = pygame.Surface((self.width, self.height))
            ui.draw_background(surf, self.width, self.height)
            self._ui_bg_cache = (key, surf)
        return self._ui_bg_cache[1]

    def _dim(self):
        """Gecachte Abdunkel-Fläche fürs Game-Over (kein per-Frame-Fill)."""
        key = (self.width, self.height)
        if self._dim_cache is None or self._dim_cache[0] != key:
            surf = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
            surf.fill((10, 12, 20, 140))
            self._dim_cache = (key, surf)
        return self._dim_cache[1]

    def _panel(self, s, rect, alpha=235, border=2):
        """Halbtransparentes Theme-Panel mit Akzent-Rahmen."""
        surf = pygame.Surface(rect.size, pygame.SRCALPHA)
        pygame.draw.rect(surf, (*ui.PANEL, alpha), surf.get_rect(),
                         border_radius=12)
        s.blit(surf, rect.topleft)
        pygame.draw.rect(s, self.accent, rect, border, border_radius=12)

    # ===================================================== Zeichnen
    def _sy(self, wy):
        return int(wy - self.cam_y)

    def draw(self):
        if self.state == SETUP:
            self._draw_setup()
            return
        s = self.surface
        self._draw_bg(s)
        for p in self.platforms:
            if not p.dead:
                self._draw_plat(s, p)
        for m in self.monsters:
            self._draw_monster(s, m)
        for b in self.bullets:
            pygame.draw.circle(s, (240, 250, 255),
                               (int(b["x"]), self._sy(b["y"])), 5)
            pygame.draw.circle(s, (120, 180, 230),
                               (int(b["x"]), self._sy(b["y"])), 5, 1)
        for p in self.particles:
            a = max(0, min(255, int(255 * p[4] / 0.5)))
            surf = pygame.Surface((5, 5), pygame.SRCALPHA)
            pygame.draw.circle(surf, (*p[5], a), (2, 2), 2)
            s.blit(surf, (p[0] - 2, self._sy(p[1]) - 2))
        self._draw_doodler(s)
        self._draw_hud(s)

    def _draw_bg(self, s):
        key = (self.width, self.height)
        if self._bg_cache is None or self._bg_cache[0] != key:
            surf = pygame.Surface((self.width, self.height))
            for y in range(self.height):
                tt = y / max(1, self.height)
                c = [int(a + (b - a) * tt) for a, b in zip(COL_BG_TOP, COL_BG_BOT)]
                pygame.draw.line(surf, c, (0, y), (self.width, y))
            self._bg_cache = (key, surf)
        s.blit(self._bg_cache[1], (0, 0))
        # feines Karo-Muster (scrollt mit)
        step = 40
        off = int(-self.cam_y) % step
        for y in range(-step, self.height + step, step):
            pygame.draw.line(s, COL_GRID, (0, y + off), (self.width, y + off), 1)
        for x in range(0, self.width, step):
            pygame.draw.line(s, COL_GRID, (x, 0), (x, self.height), 1)

    def _draw_plat(self, s, p):
        sy = self._sy(p.y)
        col = PLAT_COLORS[p.kind]
        rect = pygame.Rect(int(p.x), sy, int(p.w), 14)
        pygame.draw.rect(s, col, rect, border_radius=6)
        pygame.draw.rect(s, tuple(int(c * 0.7) for c in col), rect, 2, border_radius=6)
        if p.kind == "brittle":
            pygame.draw.line(s, (120, 80, 50), (p.x + p.w * 0.4, sy),
                             (p.x + p.w * 0.5, sy + 14), 2)
        if p.spring:
            sx = int(p.x + p.w / 2)
            pygame.draw.rect(s, (200, 200, 210), (sx - 5, sy - 10, 10, 10))
            pygame.draw.line(s, (120, 120, 130), (sx - 4, sy - 8), (sx + 4, sy - 4), 2)
            pygame.draw.line(s, (120, 120, 130), (sx - 4, sy - 4), (sx + 4, sy - 8), 2)
        if p.prop:
            sx = int(p.x + p.w / 2)
            pygame.draw.ellipse(s, (90, 150, 230), (sx - 14, sy - 8, 28, 6))
            pygame.draw.circle(s, (60, 90, 160), (sx, sy - 5), 3)

    def _draw_monster(self, s, m):
        x = int(m["x"])
        y = self._sy(m["y"]) + int(math.sin(self.anim_t * 4 + m["bob"]) * 3)
        cols = [(230, 110, 130), (150, 120, 220), (235, 150, 80)]
        col = cols[m["kind"] % 3]
        pygame.draw.circle(s, col, (x, y), 16)
        pygame.draw.circle(s, tuple(int(c * 0.7) for c in col), (x, y), 16, 2)
        for sx in (-6, 6):
            pygame.draw.circle(s, (255, 255, 255), (x + sx, y - 4), 5)
            pygame.draw.circle(s, (30, 30, 40), (x + sx, y - 4), 2)
        pygame.draw.arc(s, (60, 30, 40), (x - 7, y + 2, 14, 8), math.pi, math.tau, 2)

    def _draw_doodler(self, s):
        x = int(self.x)
        y = self._sy(self.y)
        r = self.dr
        aim_up = self.shoot_pose > 0
        # Körper
        pygame.draw.ellipse(s, COL_DOODLE, (x - r, y - r * 0.9, 2 * r, 1.8 * r))
        pygame.draw.ellipse(s, COL_DOODLE_DARK, (x - r, y - r * 0.9, 2 * r, 1.8 * r), 2)
        # Beine
        pygame.draw.line(s, COL_DOODLE_DARK, (x - r * 0.4, y + r * 0.7),
                         (x - r * 0.6, y + r * 1.2), 3)
        pygame.draw.line(s, COL_DOODLE_DARK, (x + r * 0.4, y + r * 0.7),
                         (x + r * 0.6, y + r * 1.2), 3)
        # Nase/Schnauze (zeigt in Blickrichtung, beim Schießen nach oben)
        if aim_up:
            pygame.draw.ellipse(s, (90, 170, 70), (x - 6, y - r * 1.4, 12, 12))
        else:
            nx = x + self.face * r * 0.8
            # Schnauze je nach Blickrichtung leicht versetzt (vorher: beide
            # Zweige identisch - die Schnauze hing beim Blick nach links schief)
            pygame.draw.ellipse(s, (90, 170, 70),
                                (nx - 6 if self.face > 0 else nx - 8, y - 4, 14, 12))
        # Augen
        for sx in (-1, 1):
            ex = x + sx * r * 0.4
            pygame.draw.circle(s, (255, 255, 255), (int(ex), int(y - r * 0.4)), 5)
            pygame.draw.circle(s, (30, 30, 40),
                               (int(ex + self.face), int(y - r * 0.4)), 2)
        # Propeller-Hut
        if self.prop_t > 0:
            pygame.draw.rect(s, (80, 110, 180), (x - 4, y - r * 1.7, 8, 8))
            w = int(16 + 6 * math.sin(self.anim_t * 40))
            pygame.draw.ellipse(s, (120, 160, 230), (x - w, y - r * 1.8, 2 * w, 6))
        # Wrap-Kopie am Rand
        if x < r:
            self._ghost_doodle(s, x + self.width, y, r)
        elif x > self.width - r:
            self._ghost_doodle(s, x - self.width, y, r)

    def _ghost_doodle(self, s, x, y, r):
        pygame.draw.ellipse(s, COL_DOODLE, (int(x - r), int(y - r * 0.9),
                                            int(2 * r), int(1.8 * r)))

    # ----- HUD / Overlays -----------------------------------------------
    def _draw_hud(self, s):
        # Punkte-Plakette links, Bestwert rechts - Theme-Panels über dem Papier
        img = self._small.render(str(self.score), True, ui.TEXT)
        box = img.get_rect().inflate(24, 10)
        box.topleft = (8, 8)
        self._panel(s, box, alpha=210, border=1)
        s.blit(img, img.get_rect(center=box.center))
        best = self._tiny.render(t("dj.best", hs=self.highscore), True, ui.TEXT_DIM)
        bbox = best.get_rect().inflate(20, 10)
        bbox.topright = (self.width - 8, 8)
        self._panel(s, bbox, alpha=210, border=1)
        s.blit(best, best.get_rect(center=bbox.center))
        if self.state == GAMEOVER:
            self._draw_gameover(s)

    def _draw_gameover(self, s):
        s.blit(self._dim(), (0, 0))
        cx, cy = self.width // 2, self.height // 2
        img = self._huge.render(t("common.game_over"), True, ui.RED)
        s.blit(img, img.get_rect(center=(cx, cy - int(self.height * 0.16))))

        pw = min(340, self.width - 40)
        ph = self.font.get_height() + self._small.get_height() + 46
        panel = pygame.Rect(cx - pw // 2, cy - ph // 2 + 10, pw, ph)
        self._panel(s, panel)
        sc = self.font.render(t("common.points", score=self.score), True, ui.TEXT)
        s.blit(sc, sc.get_rect(midtop=(cx, panel.y + 14)))
        best_col = ui.GOLD if self.new_best else ui.TEXT_DIM
        bs = self._small.render(t("dj.best", hs=self.highscore), True, best_col)
        s.blit(bs, bs.get_rect(midbottom=(cx, panel.bottom - 14)))

        if self.new_best:
            rec_col = ui.mix(ui.GOLD, ui.TEXT, ui.pulse(3.0, 0.0, 0.5))
            rec = self._small.render(t("trex.new_record"), True, rec_col)
            s.blit(rec, rec.get_rect(center=(cx, panel.top - 18)))

        hint_col = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.4, 0.0, 1.0))
        hint = self._small.render(t("dj.restart_hint"), True, hint_col)
        s.blit(hint, hint.get_rect(center=(cx, panel.bottom + 28)))

    # ----- Setup zeichnen -----------------------------------------------
    def _draw_setup(self):
        s = self.surface
        s.blit(self._ui_bg(), (0, 0))
        ui.draw_title(s, self.width, "DOODLE JUMP",
                      subtitle=t("snake.singleplayer"), accent=self.accent)

        d = DIFFS[self.diff]
        self._panel(s, self.diff_panel)
        name = self.font.render(
            t("dj.difficulty") + ":  " + t("dj.diff." + d["key"]), True, ui.TEXT)
        s.blit(name, name.get_rect(center=(self.diff_panel.centerx,
                                           self.diff_panel.top
                                           + int(self.diff_panel.h * 0.36))))
        note = self._tiny.render(t("dj.diff_note"), True, ui.TEXT_DIM)
        s.blit(note, note.get_rect(center=(self.diff_panel.centerx,
                                           self.diff_panel.top
                                           + int(self.diff_panel.h * 0.72))))
        arr_col = ui.mix(self.accent, ui.TEXT, ui.pulse(3.0, 0.0, 0.3))
        for rect, sym in ((self.diff_left, "<"), (self.diff_right, ">")):
            arr = self.big_font.render(sym, True, arr_col)
            s.blit(arr, arr.get_rect(center=rect.center))

        ui.draw_button(s, self.start_rect, t("common.start"), self.font,
                       selected=True, accent=self.accent)

        ctrl = self._tiny.render(t("dj.controls_hint"), True, ui.GREEN)
        s.blit(ctrl, ctrl.get_rect(center=(self.width // 2,
                                           self.start_rect.bottom + 24)))
        ui.draw_footer(s, self.width, self.height, t("dj.setup_hint"))
