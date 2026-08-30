# -*- coding: utf-8 -*-
"""
airhockey.py
============
Air Hockey - Einzelspieler (gegen KI) oder Mehrspieler (2 Spieler lokal).

Features
--------
- Echte 2D-Physik: runde Schläger und Puck mit Impulsübertragung - der Puck
  übernimmt die Schlägergeschwindigkeit beim Treffer, Banden mit Restitution,
  leichte Eisreibung, Tempolimit gegen Tunneln (2 Physik-Unterschritte).
- Tore als Öffnungen in den Seitenwänden; nach jedem Tor kurze Pause mit
  Tor-Einblendung und Anspiel beim Gegentor-Nehmer.
- MAUSSTEUERUNG: Im Einzelspieler folgt der eigene Schläger der Maus
  (sobald man sie bewegt); jede Taste schaltet zurück auf Tastensteuerung.
  Tastatur: die belegten Richtungstasten (Standard P1 = WASD, P2 = IJKL),
  im Einzelspieler steuern beide Belegungen den linken Schläger.
- KI mit drei Schwierigkeitsgraden (Leicht/Mittel/Schwer): verteidigt die
  eigene Toröffnung, greift an, wenn der Puck in ihrer Hälfte liegt, und
  umkurvt den Puck, um kein Eigentor zu schieben.
- POWER-UPS (im Setup abschaltbar): erscheinen auf dem Feld und gehören dem
  Spieler, der den Puck zuletzt berührt hat, wenn der Puck sie einsammelt:
    * XL  - größerer eigener Schläger (8 s)
    * TOR - gegnerisches Tor schrumpft (8 s)
    * >>  - schnellerer eigener Schläger (8 s)
- Setup-Screen: Schwierigkeit, Tore bis zum Sieg (3/5/7/10), Power-Ups an/aus
  (wird in settings.json unter "airhockey" gespeichert).
- Optik: Puck-Leuchtspur, Partikel bei Toren und harten Treffern,
  pulsierende Tor-Mäuler, Effekt-Anzeigen unter dem Spielstand.
- Highscore = eigene Tore (Spieler 1) einer Partie.
"""

import math
import random
import pygame

import settings as settings_mod
import ui
from game_base import Game, InputEvent
from i18n import t

# Identitätsfarben des Tisches und der Spieler - bewusst NICHT aus dem UI-Theme.
COL_TABLE = (21, 27, 44)
COL_TABLE_DARK = (10, 13, 22)      # Tor-Maul (dunkle Öffnung)
COL_TABLE_LINE = (46, 58, 88)
COL_BORDER = (90, 110, 150)
COL_P1 = (120, 230, 160)
COL_P2 = (140, 195, 255)
COL_PUCK = (245, 205, 90)

PUCK_R = 12
MALLET_R = 26
MALLET_SPEED = 420          # Pixel/s (Basis, Spieler)
PUCK_MAX_SPEED = 980
WALL_REST = 0.92            # Energieerhalt an den Banden
FRICTION = 0.28             # "Eisreibung" pro Sekunde
GOAL_H_FRAC = 0.34          # Torhöhe als Anteil der Feldhöhe

POWERUP_EVERY = 9.0         # Sekunden bis zum nächsten Power-Up
POWERUP_DUR = 8.0           # Wirkdauer eines Effekts
POWERUP_R = 15              # Radius des Symbols auf dem Feld
FREEZE_AFTER_GOAL = 1.4     # Pause nach einem Tor (Sekunden)

GOALS_CHOICES = (3, 5, 7, 10)

# KI-Parameter je Schwierigkeit: Tempo, Zielfehler (Pixel), Verteidigungslinie
AI_LEVELS = [
    dict(key="easy",   speed=270, noise=42, defend=0.80),
    dict(key="medium", speed=350, noise=18, defend=0.82),
    dict(key="hard",   speed=440, noise=6,  defend=0.84),
]

# Power-Up-Typen: (Schlüssel, Kurzsymbol, Farbe)
POWERUPS = [
    ("big",    "XL",  (120, 230, 160)),
    ("shrink", "TOR", (240, 150, 90)),
    ("fast",   ">>",  (140, 195, 255)),
]

SETUP, PLAY = "setup", "play"


def _other(side):
    return "p2" if side == "p1" else "p1"


class _Mallet:
    """Ein Schläger: Position, geglättete Geschwindigkeit (für den Impuls)."""

    def __init__(self, x, y):
        self.x = x
        self.y = y
        self.vx = 0.0
        self.vy = 0.0


class AirHockeyGame(Game):
    name = "Air Hockey"
    highscore_key = "airhockey"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        self.winner = None

        ah = self.settings.get("airhockey", {}) if isinstance(self.settings, dict) else {}
        self.diff = max(0, min(2, int(ah.get("difficulty", 1))))
        goals = int(ah.get("goals", 5))
        self.win_goals = goals if goals in GOALS_CHOICES else 5
        self.powerups_on = bool(ah.get("powerups", True))

        self.anim_t = 0.0
        self._bg_cache = None
        self._dim_cache = None

        self._apply_geometry()
        self.state = SETUP
        self._new_match()

    def _make_fonts(self):
        """Schriftgrößen aus der Fensterhöhe ableiten (Theme-Schrift)."""
        h = self.height
        self.font = ui.font(max(16, min(26, h // 26)))
        self.big_font = ui.font(max(30, min(54, h // 10)), bold=True)
        self._small = ui.font(max(13, min(20, h // 32)))
        self._tiny = ui.font(max(11, min(16, h // 42)))

    def _apply_geometry(self):
        """Feldgeometrie, Schriften und Setup-Layout aus width/height ableiten."""
        self._make_fonts()
        self.fx0, self.fy0 = 14, 14
        self.fx1, self.fy1 = self.width - 14, self.height - 14
        self.cx = self.width // 2
        self.cy = self.height // 2
        self._size = (self.width, self.height)
        self._build_setup_layout()

    def on_surface_changed(self):
        """Nach Auflösungswechsel: Geometrie neu berechnen und die laufende
        Partie proportional auf die neue Fläche skalieren."""
        old_w, old_h = getattr(self, "_size", (self.width, self.height))
        self._apply_geometry()
        sx = self.width / max(1, old_w)
        sy = self.height / max(1, old_h)
        for side in ("p1", "p2"):
            m = self.mallets[side]
            m.x *= sx
            m.y *= sy
            self._clamp_mallet(m, side)
        self.puck_x *= sx
        self.puck_y *= sy
        self._mouse_pos = (self._mouse_pos[0] * sx, self._mouse_pos[1] * sy)
        if self.powerup is not None:
            px, py, idx = self.powerup
            self.powerup = (px * sx, py * sy, idx)
        self.trail = []
        self._bg_cache = None
        self._dim_cache = None

    def _new_match(self):
        self.goals = {"p1": 0, "p2": 0}
        self.score = 0
        self.game_over = False
        self.winner = None
        self.effects = {"p1": {}, "p2": {}}   # side -> {effekt: restzeit}
        self.powerup = None                   # (x, y, typ_index) oder None
        self.powerup_timer = POWERUP_EVERY
        self.last_touch = None
        self.freeze = 0.0
        self.goal_flash = 0.0                 # Tor-Einblendung (Restzeit)
        self.goal_flash_side = None           # wer hat getroffen
        self.particles = []
        self.trail = []                       # letzte Puck-Positionen
        self._over_t = 0.0

        qh = self.height / 2
        self.mallets = {
            "p1": _Mallet(self.fx0 + (self.cx - self.fx0) * 0.35, qh),
            "p2": _Mallet(self.fx1 - (self.fx1 - self.cx) * 0.35, qh),
        }
        # Gedrückte Richtungs-Aktionen je Steuerung
        self._pressed = {"p1": set(), "p2": set()}
        # Maussteuerung für P1 (nur Einzelspieler); wird bei Mausbewegung aktiv
        self._mouse_mode = False
        self._mouse_pos = (self.mallets["p1"].x, self.mallets["p1"].y)

        self._center_puck(side=None)

    def _center_puck(self, side):
        """Puck platzieren: Mitte oder Anspiel in der Hälfte von 'side'."""
        if side is None:
            self.puck_x, self.puck_y = float(self.cx), float(self.cy)
        else:
            off = (self.cx - self.fx0) * 0.5
            self.puck_x = self.cx - off if side == "p1" else self.cx + off
            self.puck_y = float(self.cy)
        self.puck_vx = 0.0
        self.puck_vy = 0.0
        self.trail = []

    # ----- Effekt-Helfer --------------------------------------------------
    def _mallet_r(self, side):
        return MALLET_R * (1.35 if "big" in self.effects[side] else 1.0)

    def _mallet_speed(self, side):
        return MALLET_SPEED * (1.45 if "fast" in self.effects[side] else 1.0)

    def _goal_h(self, side):
        """Höhe der Toröffnung von 'side' (schrumpft durch gegnerisches TOR-Up)."""
        h = (self.fy1 - self.fy0) * GOAL_H_FRAC
        if "shrink" in self.effects[_other(side)]:
            h *= 0.6
        return h

    def _goal_range(self, side):
        h = self._goal_h(side)
        return self.cy - h / 2, self.cy + h / 2

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(440, self.width - 60)
        ph = max(52, min(64, int(self.height * 0.12)))
        y0 = max(108, int(self.height * 0.24))
        self.diff_panel = pygame.Rect(cx - bw // 2, y0, bw, ph)
        self.diff_left = pygame.Rect(self.diff_panel.left, y0, 42, ph)
        self.diff_right = pygame.Rect(self.diff_panel.right - 42, y0, 42, ph)
        bh = max(38, min(46, int(self.height * 0.085)))
        gap = 10
        y = self.diff_panel.bottom + 14
        self.goals_rect = pygame.Rect(cx - bw // 2, y, bw, bh)
        self.power_rect = pygame.Rect(cx - bw // 2, y + bh + gap, bw, bh)
        self.start_rect = pygame.Rect(cx - 95, y + 2 * (bh + gap) + 8, 190, 50)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("airhockey", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _cycle_diff(self, step):
        self.diff = (self.diff + step) % len(AI_LEVELS)
        self._save_setting("difficulty", self.diff)
        self.play_sound("click")

    def _cycle_goals(self):
        i = GOALS_CHOICES.index(self.win_goals)
        self.win_goals = GOALS_CHOICES[(i + 1) % len(GOALS_CHOICES)]
        self._save_setting("goals", self.win_goals)
        self.play_sound("click")

    def _toggle_powerups(self):
        self.powerups_on = not self.powerups_on
        self._save_setting("powerups", self.powerups_on)
        self.play_sound("select")

    def _start_play(self):
        self._new_match()
        self.state = PLAY
        self.play_sound("click")

    def _handle_setup_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Left", "a", "A"):
                self._cycle_diff(-1)
            elif event.key in ("Right", "d", "D"):
                self._cycle_diff(+1)
            elif event.key in ("t", "T"):
                self._cycle_goals()
            elif event.key in ("p", "P"):
                self._toggle_powerups()
            elif event.key in ("Return", "space"):
                self._start_play()
        elif event.kind == InputEvent.MOUSEDOWN:
            p = event.pos
            if self.diff_left.collidepoint(p):
                self._cycle_diff(-1)
            elif self.diff_right.collidepoint(p) or self.diff_panel.collidepoint(p):
                self._cycle_diff(+1)
            elif self.goals_rect.collidepoint(p):
                self._cycle_goals()
            elif self.power_rect.collidepoint(p):
                self._toggle_powerups()
            elif self.start_rect.collidepoint(p):
                self._start_play()

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup_event(event)
            return

        if self.game_over:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._start_play()
                elif event.key in ("s", "S"):
                    self.state = SETUP
                    self.play_sound("click")
            elif event.kind == InputEvent.MOUSEDOWN and \
                    self.anim_t - self._over_t > 0.5:
                # Klick startet die Revanche - mit kurzer Sperre nach Spielende
                self._start_play()
            return

        if event.kind == InputEvent.MOUSEMOVE:
            # Maus übernimmt P1 (nur Einzelspieler sinnvoll - im Mehrspieler
            # würde sie mit der Tastatur von Spieler 1 kollidieren).
            if not self.multiplayer:
                self._mouse_mode = True
                self._mouse_pos = event.pos
            return

        if event.kind == InputEvent.KEYUP:
            for scheme in ("p1", "p2"):
                for act in ("up", "down", "left", "right"):
                    if self.is_action(event.key, act, scheme):
                        self._pressed[scheme].discard(act)
            return

        if event.kind != InputEvent.KEYDOWN:
            return

        gedrückt = False
        for scheme in ("p1", "p2"):
            for act in ("up", "down", "left", "right"):
                if self.is_action(event.key, act, scheme):
                    self._pressed[scheme].add(act)
                    gedrückt = True
        if gedrückt:
            self._mouse_mode = False    # Taste gedrückt -> zurück zu Tasten

    def _key_dir(self, scheme):
        p = self._pressed[scheme]
        dx = ("right" in p) - ("left" in p)
        dy = ("down" in p) - ("up" in p)
        return dx, dy

    # ===================================================== Spiellogik
    def update(self, dt):
        self.anim_t += dt
        self._update_particles(dt)
        if self.state != PLAY or self.game_over:
            return

        # Effekt-Timer
        for side in ("p1", "p2"):
            for k in list(self.effects[side]):
                self.effects[side][k] -= dt
                if self.effects[side][k] <= 0:
                    del self.effects[side][k]

        if self.goal_flash > 0:
            self.goal_flash -= dt

        self._move_mallets(dt)

        # Nach einem Tor: kurze Pause, der Puck liegt still
        if self.freeze > 0:
            self.freeze -= dt
            return

        # Power-Ups
        if self.powerups_on:
            self._update_powerups(dt)

        # Puck in 2 Unterschritten bewegen (gegen Tunneln bei hohem Tempo)
        for _ in range(2):
            self._move_puck(dt / 2)
            if self.freeze > 0:      # Tor gefallen
                break

        # Leuchtspur
        self.trail.append((self.puck_x, self.puck_y))
        if len(self.trail) > 10:
            self.trail.pop(0)

    def _move_mallets(self, dt):
        # --- Spieler 1 (links) ---
        m1 = self.mallets["p1"]
        if self._mouse_mode and not self.multiplayer:
            self._mouse_to_mallet(m1, dt)
        else:
            if self.multiplayer:
                dx, dy = self._key_dir("p1")
            else:
                # Einzelspieler: beide Belegungen steuern links
                d1, d2 = self._key_dir("p1"), self._key_dir("p2")
                dx = max(-1, min(1, d1[0] + d2[0]))
                dy = max(-1, min(1, d1[1] + d2[1]))
            self._keys_to_mallet(m1, dx, dy, self._mallet_speed("p1"), dt)
        self._clamp_mallet(m1, "p1")

        # --- Spieler 2 / KI (rechts) ---
        m2 = self.mallets["p2"]
        if self.multiplayer:
            dx, dy = self._key_dir("p2")
            self._keys_to_mallet(m2, dx, dy, self._mallet_speed("p2"), dt)
        else:
            self._ai_move(m2, dt)
        self._clamp_mallet(m2, "p2")

    def _keys_to_mallet(self, m, dx, dy, speed, dt):
        ln = math.hypot(dx, dy)
        if ln > 0:
            tvx, tvy = dx / ln * speed, dy / ln * speed
        else:
            tvx = tvy = 0.0
        # weiches Beschleunigen/Bremsen -> Geschwindigkeit stimmt für den Impuls
        k = min(1.0, dt * 12.0)
        m.vx += (tvx - m.vx) * k
        m.vy += (tvy - m.vy) * k
        m.x += m.vx * dt
        m.y += m.vy * dt

    def _mouse_to_mallet(self, m, dt):
        tx, ty = self._mouse_pos
        k = min(1.0, dt * 16.0)
        nx = m.x + (tx - m.x) * k
        ny = m.y + (ty - m.y) * k
        if dt > 0:
            m.vx = (nx - m.x) / dt
            m.vy = (ny - m.y) / dt
            # Maus-Sprünge deckeln, sonst bekommt der Puck absurde Impulse
            sp = math.hypot(m.vx, m.vy)
            if sp > 1300:
                m.vx *= 1300 / sp
                m.vy *= 1300 / sp
        m.x, m.y = nx, ny

    def _clamp_mallet(self, m, side):
        r = self._mallet_r(side)
        if side == "p1":
            x0, x1 = self.fx0 + r, self.cx - r
        else:
            x0, x1 = self.cx + r, self.fx1 - r
        nx = max(x0, min(x1, m.x))
        ny = max(self.fy0 + r, min(self.fy1 - r, m.y))
        if nx != m.x:
            m.vx = 0.0
        if ny != m.y:
            m.vy = 0.0
        m.x, m.y = nx, ny

    # ----- KI ---------------------------------------------------------------
    def _ai_move(self, m, dt):
        lvl = AI_LEVELS[self.diff]
        speed = lvl["speed"] * (1.45 if "fast" in self.effects["p2"] else 1.0)
        r = self._mallet_r("p2")
        feld_w = self.fx1 - self.fx0

        if self.puck_x > self.cx and self.freeze <= 0:
            # Angriff: hinter den Puck stellen und Richtung Spieler-Tor schieben
            zielpunkt = (self.fx0, self.cy)          # gegnerisches Tor
            dx = zielpunkt[0] - self.puck_x
            dy = zielpunkt[1] - self.puck_y
            ln = math.hypot(dx, dy) or 1.0
            tx = self.puck_x - dx / ln * (r * 0.5)
            ty = self.puck_y - dy / ln * (r * 0.5)
            if self.puck_x > m.x + 4:
                # Puck liegt hinter dem Schläger -> außen herum, kein Eigentor
                tx = min(self.fx1 - r, self.puck_x + r + PUCK_R + 8)
                ty = self.puck_y + (r + PUCK_R + 10 if self.puck_y < m.y else
                                    -(r + PUCK_R + 10))
        else:
            # Verteidigung: auf der Linie vor dem eigenen Tor dem Puck folgen
            tx = self.fx0 + feld_w * lvl["defend"]
            gy0, gy1 = self._goal_range("p2")
            ty = max(gy0 + 10, min(gy1 - 10, self.puck_y))

        tx += random.uniform(-lvl["noise"], lvl["noise"])
        ty += random.uniform(-lvl["noise"], lvl["noise"])

        dx, dy = tx - m.x, ty - m.y
        ln = math.hypot(dx, dy)
        if ln > 4:
            m.vx = dx / ln * speed
            m.vy = dy / ln * speed
            m.x += m.vx * dt
            m.y += m.vy * dt
        else:
            m.vx = m.vy = 0.0

    # ----- Puck-Physik --------------------------------------------------------
    def _move_puck(self, dt):
        # Reibung
        f = max(0.0, 1.0 - FRICTION * dt)
        self.puck_vx *= f
        self.puck_vy *= f
        self.puck_x += self.puck_vx * dt
        self.puck_y += self.puck_vy * dt

        r = PUCK_R
        # Obere/untere Bande
        if self.puck_y - r < self.fy0:
            self.puck_y = self.fy0 + r
            self.puck_vy = abs(self.puck_vy) * WALL_REST
            self._wall_sound()
        elif self.puck_y + r > self.fy1:
            self.puck_y = self.fy1 - r
            self.puck_vy = -abs(self.puck_vy) * WALL_REST
            self._wall_sound()

        # Linke Wand / linkes Tor (dort punktet Spieler 2)
        gy0, gy1 = self._goal_range("p1")
        if self.puck_x - r < self.fx0:
            if gy0 < self.puck_y < gy1:
                if self.puck_x + r < self.fx0:
                    self._goal_scored(by="p2")
                    return
            else:
                self.puck_x = self.fx0 + r
                self.puck_vx = abs(self.puck_vx) * WALL_REST
                self._wall_sound()
        # Rechte Wand / rechtes Tor (dort punktet Spieler 1)
        gy0, gy1 = self._goal_range("p2")
        if self.puck_x + r > self.fx1:
            if gy0 < self.puck_y < gy1:
                if self.puck_x - r > self.fx1:
                    self._goal_scored(by="p1")
                    return
            else:
                self.puck_x = self.fx1 - r
                self.puck_vx = -abs(self.puck_vx) * WALL_REST
                self._wall_sound()

        # Schläger-Kollisionen
        for side in ("p1", "p2"):
            self._collide_mallet(side)

        # Tempolimit
        sp = math.hypot(self.puck_vx, self.puck_vy)
        if sp > PUCK_MAX_SPEED:
            self.puck_vx *= PUCK_MAX_SPEED / sp
            self.puck_vy *= PUCK_MAX_SPEED / sp

    def _wall_sound(self):
        if math.hypot(self.puck_vx, self.puck_vy) > 120:
            self.play_sound("bounce")

    def _collide_mallet(self, side):
        m = self.mallets[side]
        r = self._mallet_r(side) + PUCK_R
        dx = self.puck_x - m.x
        dy = self.puck_y - m.y
        dist = math.hypot(dx, dy)
        if dist >= r:
            return
        if dist < 1e-6:
            ang = random.uniform(0, math.tau)
            dx, dy, dist = math.cos(ang), math.sin(ang), 1.0
        nx, ny = dx / dist, dy / dist
        # Puck aus dem Schläger schieben
        self.puck_x = m.x + nx * (r + 0.5)
        self.puck_y = m.y + ny * (r + 0.5)
        # Relativgeschwindigkeit am Kontakt reflektieren (Schläger = schwer)
        rvx = self.puck_vx - m.vx
        rvy = self.puck_vy - m.vy
        vn = rvx * nx + rvy * ny
        if vn < 0:
            rvx -= 1.9 * vn * nx      # Restitution ~0.9
            rvy -= 1.9 * vn * ny
        self.puck_vx = m.vx + rvx
        self.puck_vy = m.vy + rvy
        self.last_touch = side
        wucht = math.hypot(self.puck_vx, self.puck_vy)
        if wucht > 520:
            self._spawn_particles(self.puck_x, self.puck_y,
                                  COL_P1 if side == "p1" else COL_P2, 6)
            self.play_sound("hit")
            self.rumble(60)
        else:
            self.play_sound("bounce")

    # ----- Tore / Power-Ups -----------------------------------------------
    def _goal_scored(self, by):
        self.goals[by] += 1
        if by == "p1":
            self.score = self.goals["p1"]
        self.goal_flash = 1.0
        self.goal_flash_side = by
        gx = self.fx1 if by == "p1" else self.fx0
        self._spawn_particles(gx, self.puck_y,
                              COL_P1 if by == "p1" else COL_P2, 22)
        self.play_sound("point")
        self.rumble(150)

        if self.goals[by] >= self.win_goals:
            self.game_over = True
            self.winner = by
            self._over_t = self.anim_t
            gewonnen = (by == "p1") or self.multiplayer
            self.play_sound("win" if gewonnen else "gameover")
            if not self.multiplayer:
                self.report_result(by == "p1")
            self.rumble(250)
            return
        # Anspiel beim Spieler, der das Tor kassiert hat
        self.freeze = FREEZE_AFTER_GOAL
        self._center_puck(side=_other(by))
        self.last_touch = None

    def _update_powerups(self, dt):
        if self.powerup is None:
            self.powerup_timer -= dt
            if self.powerup_timer <= 0:
                x = random.uniform(self.cx - 150, self.cx + 150)
                y = random.uniform(self.fy0 + 70, self.fy1 - 70)
                self.powerup = (x, y, random.randrange(len(POWERUPS)))
                self.powerup_timer = POWERUP_EVERY
            return
        px, py, idx = self.powerup
        if math.hypot(self.puck_x - px, self.puck_y - py) < PUCK_R + POWERUP_R:
            if self.last_touch is not None:
                key = POWERUPS[idx][0]
                self.effects[self.last_touch][key] = POWERUP_DUR
                self._spawn_particles(px, py, POWERUPS[idx][2], 12)
                self.play_sound("powerup")
            self.powerup = None

    # ----- Partikel ---------------------------------------------------------
    def _spawn_particles(self, x, y, color, n):
        for _ in range(n):
            ang = random.uniform(0, math.tau)
            spd = random.uniform(40, 240)
            self.particles.append([x, y, math.cos(ang) * spd, math.sin(ang) * spd,
                                   random.uniform(0.25, 0.6), color])

    def _update_particles(self, dt):
        rest = []
        for p in self.particles:
            p[0] += p[2] * dt
            p[1] += p[3] * dt
            p[2] *= 0.98
            p[3] *= 0.98
            p[4] -= dt
            if p[4] > 0:
                rest.append(p)
        self.particles = rest

    # ----- Theme-UI-Helfer ----------------------------------------------
    def _bg(self):
        """Gecachter Theme-Hintergrund hinter dem Tisch."""
        key = (self.width, self.height, ui.BG_TOP, ui.BG_BOTTOM)
        if self._bg_cache is None or self._bg_cache[0] != key:
            surf = pygame.Surface((self.width, self.height))
            ui.draw_background(surf, self.width, self.height)
            self._bg_cache = (key, surf)
        return self._bg_cache[1]

    def _dim(self):
        """Gecachte Abdunkel-Fläche fürs Spielende (kein per-Frame-Fill)."""
        key = (self.width, self.height)
        if self._dim_cache is None or self._dim_cache[0] != key:
            surf = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
            surf.fill((8, 10, 16, 150))
            self._dim_cache = (key, surf)
        return self._dim_cache[1]

    def _panel(self, s, rect, alpha=235, border=2, border_col=None):
        """Halbtransparentes Theme-Panel mit Akzent-Rahmen."""
        surf = pygame.Surface(rect.size, pygame.SRCALPHA)
        pygame.draw.rect(surf, (*ui.PANEL, alpha), surf.get_rect(),
                         border_radius=12)
        s.blit(surf, rect.topleft)
        pygame.draw.rect(s, border_col or self.accent, rect, border,
                         border_radius=12)

    # ===================================================== Zeichnen
    def draw(self):
        if self.state == SETUP:
            self._draw_setup()
            return

        s = self.surface
        s.blit(self._bg(), (0, 0))
        self._draw_table(s)
        self._draw_powerup(s)
        self._draw_trail(s)
        self._draw_puck(s)
        for side in ("p1", "p2"):
            self._draw_mallet(s, side)
        for p in self.particles:
            a = max(0, min(255, int(255 * (p[4] / 0.6))))
            surf = pygame.Surface((6, 6), pygame.SRCALPHA)
            pygame.draw.circle(surf, (*p[5], a), (3, 3), 3)
            s.blit(surf, (p[0] - 3, p[1] - 3))
        self._draw_hud(s)
        if self.game_over:
            self._draw_game_over()

    def _draw_table(self, s):
        pygame.draw.rect(s, COL_TABLE,
                         (self.fx0, self.fy0, self.fx1 - self.fx0,
                          self.fy1 - self.fy0), border_radius=18)
        # Mittellinie (gestrichelt) + Mittelkreis + Anspielpunkt
        for y in range(self.fy0 + 6, self.fy1 - 6, 22):
            pygame.draw.rect(s, COL_TABLE_LINE, (self.cx - 2, y, 4, 12))
        pygame.draw.circle(s, COL_TABLE_LINE, (self.cx, self.cy), 52, 3)
        pygame.draw.circle(s, COL_TABLE_LINE, (self.cx, self.cy), 5)
        # Bande
        pygame.draw.rect(s, COL_BORDER,
                         (self.fx0, self.fy0, self.fx1 - self.fx0,
                          self.fy1 - self.fy0), 3, border_radius=18)
        # Tor-Mäuler: Öffnung dunkel + pulsierender Glow in Spielerfarbe
        pulse = 2 + int(2 * math.sin(self.anim_t * 4))
        for side, farbe, x in (("p1", COL_P1, self.fx0), ("p2", COL_P2, self.fx1)):
            gy0, gy1 = self._goal_range(side)
            pygame.draw.rect(s, COL_TABLE_DARK, (x - 5, gy0, 10, gy1 - gy0))
            pygame.draw.rect(s, farbe, (x - 3, gy0, 6, gy1 - gy0),
                             border_radius=3)
            glow = pygame.Surface((16, int(gy1 - gy0) + 20), pygame.SRCALPHA)
            pygame.draw.rect(glow, (*farbe, 60 + pulse * 10),
                             (0, 0, 16, int(gy1 - gy0) + 20), border_radius=8)
            s.blit(glow, (x - 8, gy0 - 10))

    def _draw_trail(self, s):
        n = len(self.trail)
        for i, (x, y) in enumerate(self.trail):
            a = int(70 * (i + 1) / max(1, n))
            r = max(2, int(PUCK_R * (i + 1) / max(1, n)) - 2)
            surf = pygame.Surface((r * 2, r * 2), pygame.SRCALPHA)
            pygame.draw.circle(surf, (*COL_PUCK, a), (r, r), r)
            s.blit(surf, (x - r, y - r))

    def _draw_puck(self, s):
        pygame.draw.circle(s, COL_PUCK, (int(self.puck_x), int(self.puck_y)), PUCK_R)
        pygame.draw.circle(s, (255, 240, 190),
                           (int(self.puck_x - 3), int(self.puck_y - 3)), 4)
        pygame.draw.circle(s, (120, 95, 30),
                           (int(self.puck_x), int(self.puck_y)), PUCK_R, 2)

    def _draw_mallet(self, s, side):
        m = self.mallets[side]
        r = int(self._mallet_r(side))
        farbe = COL_P1 if side == "p1" else COL_P2
        dunkel = tuple(c // 2 for c in farbe)
        pygame.draw.circle(s, dunkel, (int(m.x), int(m.y)), r)
        pygame.draw.circle(s, farbe, (int(m.x), int(m.y)), r, 4)
        pygame.draw.circle(s, farbe, (int(m.x), int(m.y)), max(4, r // 3))

    def _draw_powerup(self, s):
        if self.powerup is None:
            return
        x, y, idx = self.powerup
        key, label, farbe = POWERUPS[idx]
        r = POWERUP_R + int(2 * math.sin(self.anim_t * 6))
        glow = pygame.Surface((r * 4, r * 4), pygame.SRCALPHA)
        pygame.draw.circle(glow, (*farbe, 50), (r * 2, r * 2), r * 2)
        s.blit(glow, (x - r * 2, y - r * 2))
        pygame.draw.circle(s, (25, 30, 45), (int(x), int(y)), r)
        pygame.draw.circle(s, farbe, (int(x), int(y)), r, 2)
        img = self._tiny.render(label, True, farbe)
        s.blit(img, img.get_rect(center=(int(x), int(y))))

    def _draw_hud(self, s):
        y0 = self.fy0 + 8
        g1 = self.big_font.render(str(self.goals["p1"]), True, COL_P1)
        g2 = self.big_font.render(str(self.goals["p2"]), True, COL_P2)
        s.blit(g1, g1.get_rect(midtop=(self.cx - 70, y0)))
        s.blit(g2, g2.get_rect(midtop=(self.cx + 70, y0)))
        links = "P1" if self.multiplayer else t("ah.you")
        rechts = "P2" if self.multiplayer else t("ah.ai")
        ly = y0 + self.big_font.get_height() + 2
        l1 = self._small.render(links, True, COL_P1)
        l2 = self._small.render(rechts, True, COL_P2)
        s.blit(l1, l1.get_rect(midtop=(self.cx - 70, ly)))
        s.blit(l2, l2.get_rect(midtop=(self.cx + 70, ly)))
        ziel = self._tiny.render(t("ah.first_to", n=self.win_goals), True,
                                 ui.TEXT_DIM)
        s.blit(ziel, ziel.get_rect(midtop=(self.cx,
                                           ly + self._small.get_height() + 4)))

        # Aktive Effekte als kleine Anzeigen unter dem Spielstand
        for side, basex, richtung in (("p1", self.fx0 + 12, 1),
                                      ("p2", self.fx1 - 12, -1)):
            y = self.fy0 + 8
            for key, restzeit in sorted(self.effects[side].items()):
                for pk, label, farbe in POWERUPS:
                    if pk != key:
                        continue
                    text = self._tiny.render(f"{label} {restzeit:2.0f}s", True, farbe)
                    x = basex if richtung == 1 else basex - text.get_width()
                    s.blit(text, (x, y))
                    y += 16

        # Tor-Einblendung
        if self.goal_flash > 0 and not self.game_over:
            farbe = COL_P1 if self.goal_flash_side == "p1" else COL_P2
            a = max(0, min(255, int(255 * self.goal_flash)))
            img = self.big_font.render(t("ah.goal"), True, farbe)
            img.set_alpha(a)
            s.blit(img, img.get_rect(center=(self.cx, self.cy - 60)))

        # Steuerungs-Hinweis (nur die ersten Sekunden)
        if self.anim_t < 6 and not self.multiplayer:
            hint = self._tiny.render(t("ah.mouse_hint"), True, ui.TEXT_DIM)
            s.blit(hint, hint.get_rect(midbottom=(self.cx, self.fy1 - 6)))

    def _draw_game_over(self):
        s = self.surface
        s.blit(self._dim(), (0, 0))
        if self.multiplayer:
            text = t("common.player_wins", n=1 if self.winner == "p1" else 2)
            farbe = COL_P1 if self.winner == "p1" else COL_P2
        else:
            gewonnen = self.winner == "p1"
            text = t("ah.win") if gewonnen else t("ah.lose")
            farbe = COL_P1 if gewonnen else ui.RED
        self.draw_center_text(text, self.big_font, farbe, -40)
        stand = f"{self.goals['p1']} : {self.goals['p2']}"
        self.draw_center_text(stand, self.font, ui.TEXT, 4)
        hint_col = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.4, 0.0, 1.0))
        self.draw_center_text(t("ah.restart_hint"), self.font, hint_col, 44)

    # ----- Setup zeichnen -----------------------------------------------
    def _draw_setup(self):
        s = self.surface
        s.blit(self._bg(), (0, 0))
        modus = t("snake.multiplayer") if self.multiplayer else t("snake.singleplayer")
        ui.draw_title(s, self.width, "AIR HOCKEY", subtitle=modus,
                      accent=self.accent)

        # Schwierigkeit (im Mehrspieler ohne Wirkung -> abgeblendet)
        lvl = AI_LEVELS[self.diff]
        aktiv = not self.multiplayer
        self._panel(s, self.diff_panel,
                    border_col=self.accent if aktiv else ui.BORDER)
        col = ui.TEXT if aktiv else ui.TEXT_DIM
        name = self.font.render(
            t("ah.difficulty") + ":  " + t("ah.diff." + lvl["key"]), True, col)
        s.blit(name, name.get_rect(center=(self.diff_panel.centerx,
                                           self.diff_panel.top
                                           + int(self.diff_panel.h * 0.34))))
        info = self._tiny.render(
            t("ah.diff_note") if aktiv else t("ah.diff_mp"), True, ui.TEXT_DIM)
        s.blit(info, info.get_rect(center=(self.diff_panel.centerx,
                                           self.diff_panel.top
                                           + int(self.diff_panel.h * 0.72))))
        arr_col = ui.mix(self.accent, ui.TEXT, ui.pulse(3.0, 0.0, 0.3)) \
            if aktiv else ui.TEXT_DIM
        for r, sym in ((self.diff_left, "<"), (self.diff_right, ">")):
            arr = self.big_font.render(sym, True, arr_col)
            s.blit(arr, arr.get_rect(center=r.center))

        self._draw_row(self.goals_rect, t("ah.goals"), str(self.win_goals), True)
        self._draw_row(self.power_rect, t("ah.powerups"),
                       t("common.on") if self.powerups_on else t("common.off"),
                       self.powerups_on)

        ui.draw_button(s, self.start_rect, t("common.start"), self.font,
                       selected=True, accent=self.accent)

        h2 = self._tiny.render(t("ah.mouse_hint"), True, ui.GREEN)
        s.blit(h2, h2.get_rect(center=(self.width // 2,
                                       self.start_rect.bottom + 22)))
        ui.draw_footer(s, self.width, self.height, t("ah.setup_hint"))

    def _draw_row(self, rect, label, wert, an):
        s = self.surface
        pygame.draw.rect(s, ui.BTN_SEL if an else ui.BTN, rect, border_radius=8)
        pygame.draw.rect(s, ui.BORDER_LIGHT if an else ui.BORDER, rect, 1,
                         border_radius=8)
        lab = self.font.render(label, True, ui.TEXT)
        s.blit(lab, (rect.x + 16, rect.centery - lab.get_height() // 2))
        img = self.font.render(f"< {wert} >", True,
                               self.accent if an else ui.TEXT_DIM)
        s.blit(img, (rect.right - img.get_width() - 16,
                     rect.centery - img.get_height() // 2))
