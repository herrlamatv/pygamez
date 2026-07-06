# -*- coding: utf-8 -*-
"""
invaders.py
===========
Space Invaders - komplett überarbeitet, mit zwei Spielmodi:

- KLASSIK : Klassischer Alien-Block. Nach der Auswahl erscheint ein Setup-Screen,
            in dem man einstellt:
              * Bewegung: nur links/rechts (unten)  ODER  frei mit WASD/Pfeilen.
              * Zielen:   immer nach oben  ODER  zur Maus (schießt dorthin, wo
                          der Mauszeiger ist).
            Bei fester Bewegung/festem Zielen ist es das klassische Space
            Invaders; mit freier Bewegung schaut das Schiff dennoch immer nach
            vorne (oben), sofern nicht Maus-Zielen aktiv ist. Zerstörte Aliens
            lassen manchmal Power-ups fallen (u.a. bessere Waffen).

- ARENA   : Freie Bewegung in alle Richtungen. Gegner strömen von allen Rändern
            herein und jagen dich. Geschossen wird in Blickrichtung; die Waffe
            lässt sich jederzeit mit den Tasten 1-4 wechseln.

Gemeinsame Features (beide Modi):
- Levelsystem mit steigender Schwierigkeit und einem BOSS in jedem 4. Level.
- Vier Waffen: Blaster, Streuschuss, Schnellfeuer, Laser (durchschlagend).
- Power-ups: Extraleben, Schutzschild (kurz unverwundbar), Waffen-Upgrade.
- Explosions-Partikel, HUD mit Level/Leben/Waffe/Schild, Highscore.
"""

import math
import random

import pygame

from game_base import Game, InputEvent
from i18n import t

# ----- Farben ---------------------------------------------------------------
COL_BG = (8, 10, 18)
COL_STAR = (40, 46, 66)
COL_PLAYER = (110, 220, 140)
COL_PLAYER_HIT = (240, 240, 240)
COL_TEXT = (230, 230, 235)
COL_MUTE = (150, 158, 176)
COL_SHIELD = (90, 170, 220)
COL_EBULLET = (240, 120, 120)

# Waffenfarben (auch für die Schüsse)
WEAPON_COL = {
    "single": (240, 240, 120),
    "spread": (140, 235, 180),
    "rapid": (240, 190, 110),
    "pierce": (150, 200, 255),
}
WEAPON_ORDER = ["single", "spread", "rapid", "pierce"]

# Gegnertypen -> Farbe
KIND_COL = {
    "grunt": (235, 110, 110),
    "chaser": (235, 150, 80),
    "shooter": (185, 120, 235),
    "drifter": (110, 180, 235),
    "boss": (235, 90, 140),
}
KIND_SCORE = {"grunt": 10, "chaser": 15, "shooter": 20, "drifter": 12, "boss": 250}

# ----- Spielwerte -----------------------------------------------------------
PLAYER_W = 40
PLAYER_H = 22
PLAYER_SPEED = 300         # px/s
PLAYER_Y_OFF = 46          # Abstand vom unteren Rand (nur Klassik)

EBULLET_SPEED = 210
HIT_INVULN = 1.4           # Sekunden Unverwundbarkeit nach einem Treffer
POWERUP_FALL = 70
POWERUP_CHANCE = 0.13
WEAPON_PICKUP_TIME = 14.0  # Dauer eines Waffen-Upgrades (Klassik)


class InvadersGame(Game):
    name = "Invaders"
    highscore_key = "invaders"

    # Vom PreGameScreen ausgewertete Modusauswahl: nur zwei Grundmodi. Für
    # "classic" wählt man danach im Setup-Screen Bewegung/Zielen.
    MODES = [("classic", "inv.mode_classic"), ("free", "inv.mode_free")]

    # ----- Aufbau -------------------------------------------------------

    def reset(self):
        # arena : Arena-Wellen, Zielen in Bewegungsrichtung, Waffenwechsel 1-4.
        self.arena = (self.mode == "free")

        # Klassik-Unteroptionen (werden im Setup-Screen gewählt):
        #   opt_move: "lr" (nur links/rechts unten) oder "free" (WASD, frei)
        #   opt_aim : "fixed" (immer nach oben) oder "mouse" (zur Maus zielen)
        self.opt_move = "lr"
        self.opt_aim = "fixed"
        self.mouse_pos = (self.width / 2, self.height / 4)

        self._init_run()

        # Arena startet sofort; Klassik zeigt zuerst den Setup-Screen.
        if self.arena:
            self._enter_play()
        else:
            self.state = "setup"
            self._build_setup()

    def _init_run(self):
        """Setzt alle Werte für eine Runde zurück (auch beim Neustart)."""
        self.state = "play"
        self.score = 0
        self.game_over = False
        self.lives = 3
        self.level = 1

        self.bullets = []      # {x,y,vx,vy,dmg,pierce,rad,hits}
        self.ebullets = []     # {x,y,vx,vy,rad}
        self.powerups = []     # {x,y,vy,kind}
        self.particles = []    # {x,y,vx,vy,life,max,col}
        self.aliens = []       # siehe _make_alien
        self.shields = []      # nur Klassik: {x,y,w,h,hp}

        self.weapon = "single"
        self.weapon_timer = 0.0     # >0: befristetes Upgrade (Klassik); 0: dauerhaft
        self.shield_timer = 0.0     # Power-up-Schild
        self.invuln_timer = 0.0     # kurze Unverwundbarkeit nach Treffer
        self._cooldown = 0.0

        self.held = set()           # aktuell gedrückte Aktionen
        self.face = (0.0, -1.0)     # Blickrichtung

        self.px = self.width / 2
        self.py = self.height - PLAYER_Y_OFF

        self._banner = ""
        self._banner_t = 0.0
        self._start_count = 0

        # Sternenhimmel (einmalig gewürfelt)
        self._stars = [(random.randint(0, self.width), random.randint(0, self.height),
                        random.choice((1, 1, 2))) for _ in range(70)]

    def _enter_play(self):
        """Leitet aus den (Klassik-)Optionen die Flags ab und startet Level 1."""
        if self.arena:
            self.free_move = True     # in alle Richtungen
            self.mouse_aim = False    # Arena zielt in Bewegungsrichtung
        else:
            self.free_move = (self.opt_move == "free")
            self.mouse_aim = (self.opt_aim == "mouse")
        self.state = "play"
        self.level = 1
        self._start_level()

    def _restart(self):
        """Neustart nach Game Over - behält die gewählten Klassik-Optionen."""
        self._init_run()
        self._enter_play()

    # ----- Setup-Screen (nur Klassik) -----------------------------------

    def _build_setup(self):
        self._setup_sel = 0
        self._setup_rects = []
        bw, bh, gap = 340, 42, 14
        total = 3 * (bh + gap) - gap
        y0 = max(150, self.height // 2 - total // 2 + 20)
        for i in range(3):
            x = self.width // 2 - bw // 2
            self._setup_rects.append(pygame.Rect(x, y0 + i * (bh + gap), bw, bh))

    def _setup_event(self, event):
        activate = (lambda k: k in ("Return", "space") or self.is_action(k, "action"))
        if event.kind == InputEvent.KEYDOWN:
            if self.is_action(event.key, "up"):
                self._setup_sel = (self._setup_sel - 1) % 3
                self.play_sound("move")
            elif self.is_action(event.key, "down"):
                self._setup_sel = (self._setup_sel + 1) % 3
                self.play_sound("move")
            elif self.is_action(event.key, "left") or self.is_action(event.key, "right"):
                self._setup_toggle()
            elif activate(event.key):
                self._setup_activate()
        elif event.kind == InputEvent.MOUSEMOVE and event.pos:
            for i, r in enumerate(self._setup_rects):
                if r.collidepoint(event.pos):
                    self._setup_sel = i
        elif event.kind == InputEvent.MOUSEDOWN and event.pos:
            for i, r in enumerate(self._setup_rects):
                if r.collidepoint(event.pos):
                    self._setup_sel = i
                    self._setup_activate()
                    break

    def _setup_toggle(self):
        if self._setup_sel == 0:
            self.opt_move = "free" if self.opt_move == "lr" else "lr"
            self.play_sound("select")
        elif self._setup_sel == 1:
            self.opt_aim = "mouse" if self.opt_aim == "fixed" else "fixed"
            self.play_sound("select")

    def _setup_activate(self):
        if self._setup_sel == 2:
            self.play_sound("click")
            self._enter_play()
        else:
            self._setup_toggle()

    def _level_boss(self):
        return self.level % 4 == 0

    def _start_level(self):
        """Baut ein Level (Formation bzw. Arena-Welle) passend zum Modus auf."""
        self.bullets.clear()
        self.ebullets.clear()
        self.powerups.clear()
        self.aliens = []

        boss = self._level_boss()
        # Startbanner (Level 1 zusätzlich mit Steuerungshinweis)
        self._banner = t("inv.boss_level") if boss else t("inv.level_start", level=self.level)
        self._banner_t = 2.2

        # Spieler neu setzen: Arena startet mittig, Klassik-Varianten unten.
        if self.arena:
            self.px, self.py = self.width / 2, self.height / 2
        else:
            self.px, self.py = self.width / 2, self.height - PLAYER_Y_OFF

        if boss:
            self._spawn_boss()
        elif self.arena:
            self._spawn_arena_wave()
        else:
            self._spawn_classic_block()

        if not self.arena:
            self._build_shields()

        self._start_count = max(1, len(self.aliens))

    def _make_alien(self, cx, cy, kind, hp, w=30, h=22, vx=0.0, vy=0.0):
        return {"cx": cx, "cy": cy, "kind": kind, "hp": hp, "maxhp": hp,
                "w": w, "h": h, "vx": vx, "vy": vy, "t": random.random() * 6.28,
                "shoot": random.uniform(0.6, 2.2)}

    def _spawn_classic_block(self):
        rows = min(6, 3 + self.level // 2)
        cols = min(10, 6 + self.level // 3)
        hp = 1 + self.level // 4
        gap_x, gap_y = 16, 14
        block_w = cols * (30 + gap_x) - gap_x
        start_x = (self.width - block_w) / 2 + 15
        start_y = 60
        for r in range(rows):
            for c in range(cols):
                self.aliens.append(self._make_alien(
                    start_x + c * (30 + gap_x), start_y + r * (22 + gap_y),
                    "grunt", hp))
                self.aliens[-1]["row"] = r
        self.alien_dir = 1
        self.alien_speed = 22 + self.level * 8
        self.alien_drop = 16
        self._shoot_timer = 0.0

    def _spawn_arena_wave(self):
        count = 5 + self.level * 2
        hp = 1 + self.level // 3
        kinds = ["chaser", "drifter", "shooter"]
        for _ in range(count):
            cx, cy = self._edge_spawn()
            kind = random.choice(kinds)
            spd = 40 + self.level * 5 + random.uniform(-10, 20)
            ang = math.atan2(self.py - cy, self.px - cx)
            self.aliens.append(self._make_alien(
                cx, cy, kind, hp, vx=math.cos(ang) * spd, vy=math.sin(ang) * spd))

    def _spawn_boss(self):
        bx, by = self.width / 2, 90
        hp = 26 + self.level * 6
        boss = self._make_alien(bx, by, "boss", hp, w=90, h=54,
                                vx=70 + self.level * 4, vy=0)
        boss["burst"] = 2.4
        self.aliens.append(boss)
        # ein paar Begleiter
        for i in range(4):
            if self.arena:
                cx, cy = self._edge_spawn()
                self.aliens.append(self._make_alien(cx, cy, "drifter", 1,
                                                    vx=random.uniform(-60, 60),
                                                    vy=random.uniform(30, 70)))
            else:
                self.aliens.append(self._make_alien(120 + i * 130, 170, "grunt", 1))
                self.aliens[-1]["row"] = 0
        if not self.arena:
            self.alien_dir = 1
            self.alien_speed = 30
            self.alien_drop = 12
            self._shoot_timer = 0.0

    def _edge_spawn(self):
        """Zufällige Position knapp außerhalb eines Randes (für Arena-Gegner)."""
        side = random.choice(("top", "bottom", "left", "right"))
        if side == "top":
            return random.uniform(20, self.width - 20), -20
        if side == "bottom":
            return random.uniform(20, self.width - 20), self.height + 20
        if side == "left":
            return -20, random.uniform(40, self.height - 20)
        return self.width + 20, random.uniform(40, self.height - 20)

    def _build_shields(self):
        self.shields = []
        for i in range(4):
            sx = (i + 1) * self.width / 5 - 26
            self.shields.append({"x": sx, "y": self.height - 130,
                                 "w": 52, "h": 18, "hp": 6})

    def on_surface_changed(self):
        """Falls die Auflösung wechselt: Sterne neu würfeln, Spieler einpassen."""
        self._stars = [(random.randint(0, self.width), random.randint(0, self.height),
                        random.choice((1, 1, 2))) for _ in range(70)]
        self.px = max(16, min(self.width - 16, self.px))
        self.py = max(40, min(self.height - 16, self.py))
        if self.state == "setup":
            self._build_setup()

    # ----- Eingabe ------------------------------------------------------

    def handle_event(self, event):
        # Setup-Screen (nur Klassik, vor dem ersten Level).
        if self.state == "setup":
            self._setup_event(event)
            return

        # Maus: Position merken (zum Zielen), Klick schießt.
        if event.kind == InputEvent.MOUSEMOVE:
            if event.pos:
                self.mouse_pos = event.pos
            return
        if event.kind == InputEvent.MOUSEDOWN:
            if event.pos:
                self.mouse_pos = event.pos
            if not self.game_over:
                self._fire()
            return

        if event.kind == InputEvent.KEYUP:
            for act in ("up", "down", "left", "right", "action"):
                if self.is_action(event.key, act):
                    self.held.discard(act)
            return

        if event.kind != InputEvent.KEYDOWN:
            return

        if self.game_over:
            if event.key in ("Return", "space"):
                self._restart()
            return

        # Waffenwechsel per Zifferntaste nur in der Arena. In Klassik kommen
        # bessere Waffen (befristet) über Power-ups.
        if event.key in ("1", "2", "3", "4"):
            if self.arena:
                self._select_weapon(WEAPON_ORDER[int(event.key) - 1])
            return

        for act in ("up", "down", "left", "right", "action"):
            if self.is_action(event.key, act):
                self.held.add(act)

    def _select_weapon(self, weapon):
        if weapon == self.weapon:
            return
        self.weapon = weapon
        self.weapon_timer = 0.0     # manuell gewählt -> dauerhaft
        self.play_sound("select")

    # ----- Update -------------------------------------------------------

    def update(self, dt):
        if self.game_over or self.state != "play":
            return

        self._cooldown = max(0.0, self._cooldown - dt)
        self.shield_timer = max(0.0, self.shield_timer - dt)
        self.invuln_timer = max(0.0, self.invuln_timer - dt)
        self._banner_t = max(0.0, self._banner_t - dt)

        # Befristetes Waffen-Upgrade (Klassik) läuft ab -> zurück auf Blaster.
        if self.weapon_timer > 0:
            self.weapon_timer -= dt
            if self.weapon_timer <= 0:
                self.weapon = "single"

        self._update_player(dt)
        if "action" in self.held:
            self._fire()

        if self.arena:
            self._update_arena(dt)
        else:
            self._update_classic(dt)

        # In allen Modi mit freier Bewegung schadet Körperkontakt mit Gegnern.
        if self.free_move:
            self._check_contact_damage()

        self._update_bullets(dt)
        self._update_ebullets(dt)
        self._update_powerups(dt)
        self._update_particles(dt)

        # Level geschafft (erst wenn das Startbanner durch ist).
        if not self.aliens and self._banner_t <= 0:
            self._next_level()

    def _next_level(self):
        self.level += 1
        self.score += 50
        self.play_sound("level")
        self._start_level()

    def _update_player(self, dt):
        vx = (("right" in self.held) - ("left" in self.held))
        vy = (("down" in self.held) - ("up" in self.held)) if self.free_move else 0

        nx = ny = 0.0
        if vx or vy:
            length = math.hypot(vx, vy) or 1.0
            nx, ny = vx / length, vy / length
            self.px += nx * PLAYER_SPEED * dt
            if self.free_move:
                self.py += ny * PLAYER_SPEED * dt

        # Blickrichtung bestimmen:
        #  - Maus-Zielen: immer zur Mausposition.
        #  - Arena: der Bewegung folgen.
        #  - sonst (Klassik): IMMER nach vorne/oben - auch beim Rückwärtsgehen.
        if self.mouse_aim:
            dx = self.mouse_pos[0] - self.px
            dy = self.mouse_pos[1] - self.py
            d = math.hypot(dx, dy)
            if d > 1:
                self.face = (dx / d, dy / d)
        elif self.arena:
            if vx or vy:
                self.face = (nx, ny)
        else:
            self.face = (0.0, -1.0)

        # In den Spielbereich zwingen.
        self.px = max(PLAYER_W / 2, min(self.width - PLAYER_W / 2, self.px))
        if self.free_move:
            self.py = max(34 + PLAYER_H / 2,
                          min(self.height - PLAYER_H / 2, self.py))
        else:
            self.py = self.height - PLAYER_Y_OFF

    def _fire(self):
        if self._cooldown > 0:
            return
        fx, fy = self.face
        ang = math.atan2(fy, fx)
        ox, oy = self.px + fx * (PLAYER_H / 2 + 4), self.py + fy * (PLAYER_H / 2 + 4)
        w = self.weapon
        if w == "spread":
            for da in (-0.26, 0.0, 0.26):
                self._add_bullet(ox, oy, ang + da, 460, 1, 0)
            self._cooldown = 0.46
        elif w == "rapid":
            self._add_bullet(ox, oy, ang + random.uniform(-0.05, 0.05), 560, 1, 0)
            self._cooldown = 0.10
        elif w == "pierce":
            self._add_bullet(ox, oy, ang, 720, 2, 3, rad=5)
            self._cooldown = 0.55
        else:  # single
            self._add_bullet(ox, oy, ang, 520, 1, 0)
            self._cooldown = 0.28
        self.play_sound("shoot")

    def _add_bullet(self, x, y, ang, spd, dmg, pierce, rad=3):
        self.bullets.append({"x": x, "y": y, "vx": math.cos(ang) * spd,
                             "vy": math.sin(ang) * spd, "dmg": dmg,
                             "pierce": pierce, "rad": rad, "hits": set()})

    # ----- Gegner: Klassik ----------------------------------------------

    def _update_classic(self, dt):
        minions = [a for a in self.aliens if a["kind"] != "boss"]
        bosses = [a for a in self.aliens if a["kind"] == "boss"]

        if minions:
            factor = 1.0 + (self._start_count - len(self.aliens)) * 0.025
            dx = self.alien_dir * self.alien_speed * factor * dt
            left = min(a["cx"] - a["w"] / 2 for a in minions)
            right = max(a["cx"] + a["w"] / 2 for a in minions)
            if (self.alien_dir > 0 and right + dx >= self.width - 8) or \
               (self.alien_dir < 0 and left + dx <= 8):
                self.alien_dir *= -1
                for a in minions:
                    a["cy"] += self.alien_drop
            else:
                for a in minions:
                    a["cx"] += dx

            # Zufälliger Schuss aus der jeweils untersten Reihe.
            self._shoot_timer -= dt
            if self._shoot_timer <= 0:
                self._shoot_timer = max(0.25, random.uniform(0.5, 1.3) - self.level * 0.03)
                columns = {}
                for a in minions:
                    key = round(a["cx"] / 20)
                    if key not in columns or a["cy"] > columns[key]["cy"]:
                        columns[key] = a
                shooter = random.choice(list(columns.values()))
                self._enemy_shot(shooter["cx"], shooter["cy"] + shooter["h"] / 2, 0, 1)

        for b in bosses:
            self._update_boss(b, dt)

        # Erreichen die Gegner "unten" -> Treffer, Block etwas hoch. Bei freier
        # Bewegung ist das die Feld-Unterkante, sonst die feste Spielerhöhe.
        limit = (self.height - 40) if self.free_move else (self.py - PLAYER_H / 2)
        if any(a["cy"] + a["h"] / 2 >= limit for a in minions):
            self._hit_player()
            for a in minions:
                a["cy"] -= self.alien_drop * 3

    # ----- Gegner: Arena ------------------------------------------------

    def _update_arena(self, dt):
        for a in self.aliens:
            if a["kind"] == "boss":
                self._update_boss(a, dt)
                continue
            self._update_arena_enemy(a, dt)

    def _check_contact_damage(self):
        """Körperkontakt mit einem Gegner kostet ein Leben (mit i-Frames)."""
        if self.invuln_timer > 0 or self.shield_timer > 0:
            return
        for a in self.aliens:
            if abs(a["cx"] - self.px) < (a["w"] + PLAYER_W) / 2 * 0.6 and \
               abs(a["cy"] - self.py) < (a["h"] + PLAYER_H) / 2 * 0.6:
                self._hit_player()
                break

    def _update_arena_enemy(self, a, dt):
        kind = a["kind"]
        a["t"] += dt
        if kind == "chaser":
            ang = math.atan2(self.py - a["cy"], self.px - a["cx"])
            spd = 60 + self.level * 6
            a["vx"], a["vy"] = math.cos(ang) * spd, math.sin(ang) * spd
        elif kind == "drifter":
            # gerade Bahn, an den Rändern abprallen
            if a["cx"] < 12 or a["cx"] > self.width - 12:
                a["vx"] *= -1
            if a["cy"] < 34 or a["cy"] > self.height - 12:
                a["vy"] *= -1
        elif kind == "shooter":
            # Abstand halten und den Spieler beschießen
            dist = math.hypot(self.px - a["cx"], self.py - a["cy"]) or 1
            ang = math.atan2(self.py - a["cy"], self.px - a["cx"])
            drive = 1 if dist > 240 else (-1 if dist < 150 else 0)
            spd = 55 + self.level * 4
            strafe = 0.9
            a["vx"] = (math.cos(ang) * drive - math.sin(ang) * strafe) * spd
            a["vy"] = (math.sin(ang) * drive + math.cos(ang) * strafe) * spd
            a["shoot"] -= dt
            if a["shoot"] <= 0:
                a["shoot"] = random.uniform(1.1, 2.2)
                self._enemy_shot(a["cx"], a["cy"], math.cos(ang), math.sin(ang))

        a["cx"] += a["vx"] * dt
        a["cy"] += a["vy"] * dt
        # innerhalb des Feldes halten (nur weich)
        a["cx"] = max(-30, min(self.width + 30, a["cx"]))
        a["cy"] = max(-30, min(self.height + 30, a["cy"]))

    # ----- Boss (beide Modi) --------------------------------------------

    def _update_boss(self, b, dt):
        b["t"] += dt
        # horizontal pendeln
        b["cx"] += b["vx"] * dt
        if b["cx"] < b["w"] / 2 + 8 or b["cx"] > self.width - b["w"] / 2 - 8:
            b["vx"] *= -1
            b["cx"] = max(b["w"] / 2 + 8, min(self.width - b["w"] / 2 - 8, b["cx"]))
        b["cy"] = 90 + math.sin(b["t"] * 1.3) * 22

        b["burst"] = b.get("burst", 2.4) - dt
        if b["burst"] <= 0:
            b["burst"] = max(1.1, 2.6 - self.level * 0.05)
            # radiale Salve + gezielter Schuss
            n = 10
            for i in range(n):
                ang = (i / n) * math.tau + b["t"]
                self._enemy_shot(b["cx"], b["cy"], math.cos(ang), math.sin(ang), speed=150)
            ang = math.atan2(self.py - b["cy"], self.px - b["cx"])
            self._enemy_shot(b["cx"], b["cy"], math.cos(ang), math.sin(ang), speed=260)

    def _enemy_shot(self, x, y, dx, dy, speed=EBULLET_SPEED):
        length = math.hypot(dx, dy) or 1.0
        self.ebullets.append({"x": x, "y": y, "vx": dx / length * speed,
                              "vy": dy / length * speed, "rad": 4})

    # ----- Projektile & Kollisionen -------------------------------------

    def _update_bullets(self, dt):
        alive = []
        for b in self.bullets:
            b["x"] += b["vx"] * dt
            b["y"] += b["vy"] * dt
            if b["x"] < -20 or b["x"] > self.width + 20 or \
               b["y"] < -20 or b["y"] > self.height + 20:
                continue
            if not self.arena and self._bullet_hits_shield(b):
                continue
            if self._bullet_hits_alien(b):
                if b["pierce"] > 0:
                    b["pierce"] -= 1     # durchschlägt -> weiterfliegen
                    alive.append(b)
                # sonst: Kugel verbraucht
            else:
                alive.append(b)
        self.bullets = alive

    def _bullet_hits_alien(self, b):
        for a in self.aliens:
            if id(a) in b["hits"]:
                continue
            if abs(b["x"] - a["cx"]) <= a["w"] / 2 + b["rad"] and \
               abs(b["y"] - a["cy"]) <= a["h"] / 2 + b["rad"]:
                b["hits"].add(id(a))
                a["hp"] -= b["dmg"]
                if a["hp"] <= 0:
                    self._kill_alien(a)
                else:
                    self.play_sound("hit")
                return True
        return False

    def _kill_alien(self, a):
        if a in self.aliens:
            self.aliens.remove(a)
        self.score += KIND_SCORE.get(a["kind"], 10)
        self._spawn_particles(a["cx"], a["cy"], KIND_COL.get(a["kind"], (235, 110, 110)),
                              18 if a["kind"] == "boss" else 8)
        self.play_sound("explode")
        if a["kind"] == "boss":
            self.rumble(220)
            self._drop_powerup(a["cx"], a["cy"], force=True)
        elif random.random() < POWERUP_CHANCE:
            self._drop_powerup(a["cx"], a["cy"])

    def _bullet_hits_shield(self, b):
        for sh in self.shields:
            if sh["hp"] > 0 and sh["x"] <= b["x"] <= sh["x"] + sh["w"] and \
               sh["y"] <= b["y"] <= sh["y"] + sh["h"]:
                sh["hp"] -= 1
                return True
        return False

    def _update_ebullets(self, dt):
        alive = []
        for b in self.ebullets:
            b["x"] += b["vx"] * dt
            b["y"] += b["vy"] * dt
            if b["x"] < -20 or b["x"] > self.width + 20 or \
               b["y"] < -20 or b["y"] > self.height + 20:
                continue
            if not self.arena and self._ebullet_hits_shield(b):
                continue
            if self._can_be_hit() and \
               abs(b["x"] - self.px) <= (PLAYER_W / 2 + b["rad"]) and \
               abs(b["y"] - self.py) <= (PLAYER_H / 2 + b["rad"]):
                self._hit_player()
                continue
            alive.append(b)
        self.ebullets = alive

    def _ebullet_hits_shield(self, b):
        for sh in self.shields:
            if sh["hp"] > 0 and sh["x"] <= b["x"] <= sh["x"] + sh["w"] and \
               sh["y"] <= b["y"] <= sh["y"] + sh["h"]:
                sh["hp"] -= 1
                return True
        return False

    def _can_be_hit(self):
        return self.invuln_timer <= 0 and self.shield_timer <= 0

    # ----- Power-ups ----------------------------------------------------

    def _drop_powerup(self, x, y, force=False):
        kind = random.choices(("weapon", "shield", "life"),
                              weights=(6, 4, 2))[0]
        self.powerups.append({"x": x, "y": y, "vy": POWERUP_FALL, "kind": kind})

    def _update_powerups(self, dt):
        alive = []
        for p in self.powerups:
            p["y"] += p["vy"] * dt
            if p["y"] > self.height + 20:
                continue
            if abs(p["x"] - self.px) < PLAYER_W and abs(p["y"] - self.py) < PLAYER_H:
                self._collect_powerup(p)
                continue
            alive.append(p)
        self.powerups = alive

    def _collect_powerup(self, p):
        self.play_sound("powerup")
        if p["kind"] == "life":
            self.lives = min(9, self.lives + 1)
        elif p["kind"] == "shield":
            self.shield_timer = 6.0
        else:  # weapon
            new = random.choice(["spread", "rapid", "pierce"])
            self.weapon = new
            # In der Arena kann man ohnehin frei wechseln -> dort dauerhaft.
            self.weapon_timer = 0.0 if self.arena else WEAPON_PICKUP_TIME

    # ----- Partikel -----------------------------------------------------

    def _spawn_particles(self, x, y, col, n):
        for _ in range(n):
            ang = random.uniform(0, math.tau)
            spd = random.uniform(40, 190)
            life = random.uniform(0.3, 0.7)
            self.particles.append({"x": x, "y": y, "vx": math.cos(ang) * spd,
                                   "vy": math.sin(ang) * spd, "life": life,
                                   "max": life, "col": col})

    def _update_particles(self, dt):
        alive = []
        for p in self.particles:
            p["life"] -= dt
            if p["life"] <= 0:
                continue
            p["x"] += p["vx"] * dt
            p["y"] += p["vy"] * dt
            p["vx"] *= 0.92
            p["vy"] *= 0.92
            alive.append(p)
        self.particles = alive

    # ----- Spieler-Treffer ----------------------------------------------

    def _hit_player(self):
        if not self._can_be_hit():
            return
        self.lives -= 1
        self.invuln_timer = HIT_INVULN
        self.ebullets.clear()
        self.play_sound("hit")
        self.rumble(180)
        self._spawn_particles(self.px, self.py, COL_PLAYER, 14)
        if self.lives <= 0:
            self.lives = 0
            self.game_over = True
            self.play_sound("gameover")

    # ----- Zeichnen -----------------------------------------------------

    def draw(self):
        s = self.surface
        s.fill(COL_BG)
        for (sx, sy, r) in self._stars:
            pygame.draw.circle(s, COL_STAR, (sx, sy), r)

        # Setup-Screen (Klassik) statt Spielfeld.
        if self.state == "setup":
            self._draw_setup(s)
            return

        # Klassik-Schutzschilde
        for sh in self.shields:
            if sh["hp"] <= 0:
                continue
            g = sh["hp"] / 6
            farbe = (int(60 + 30 * g), int(110 + 60 * g), int(150 + 70 * g))
            pygame.draw.rect(s, farbe, (sh["x"], sh["y"], sh["w"], sh["h"]),
                             border_radius=4)

        # Gegner
        for a in self.aliens:
            self._draw_alien(s, a)

        # Power-ups
        for p in self.powerups:
            self._draw_powerup(s, p)

        # Schüsse
        for b in self.bullets:
            col = WEAPON_COL.get(self.weapon, (240, 240, 120))
            pygame.draw.circle(s, col, (int(b["x"]), int(b["y"])), b["rad"] + 1)
        for b in self.ebullets:
            pygame.draw.circle(s, COL_EBULLET, (int(b["x"]), int(b["y"])), b["rad"])

        # Partikel
        for p in self.particles:
            a = max(0.0, p["life"] / p["max"])
            r = max(1, int(3 * a))
            pygame.draw.circle(s, p["col"], (int(p["x"]), int(p["y"])), r)

        self._draw_player(s)

        # Maus-Fadenkreuz beim Maus-Zielen
        if self.mouse_aim:
            mx, my = int(self.mouse_pos[0]), int(self.mouse_pos[1])
            pygame.draw.circle(s, (240, 240, 240), (mx, my), 8, 1)
            pygame.draw.line(s, (240, 240, 240), (mx - 11, my), (mx - 4, my))
            pygame.draw.line(s, (240, 240, 240), (mx + 4, my), (mx + 11, my))

        self._draw_hud(s)
        self._draw_banner(s)

        if self.game_over:
            self.draw_center_text(t("common.game_over"), self.big_font,
                                  (235, 110, 110), -20)
            self.draw_center_text(t("common.enter_restart"), self.font, COL_TEXT, 30)

    def _draw_setup(self, s):
        """Zeichnet den Klassik-Setup-Screen (Bewegung/Zielen wählen)."""
        title = self.big_font.render(self.name, True, COL_TEXT)
        s.blit(title, title.get_rect(center=(self.width // 2, 70)))
        sub = self.font.render(t("inv.setup_title"), True, COL_MUTE)
        s.blit(sub, sub.get_rect(center=(self.width // 2, 112)))

        rows = [
            (t("inv.opt_move"),
             t("inv.opt_move_free") if self.opt_move == "free" else t("inv.opt_move_lr")),
            (t("inv.opt_aim"),
             t("inv.opt_aim_mouse") if self.opt_aim == "mouse" else t("inv.opt_aim_fixed")),
            (t("common.start"), None),
        ]
        for i, r in enumerate(self._setup_rects):
            selected = (i == self._setup_sel)
            pygame.draw.rect(s, (70, 96, 150) if selected else (44, 50, 66),
                             r, border_radius=8)
            label, value = rows[i]
            if value is None:   # Start-Button: zentriert
                img = self.font.render(label, True, COL_TEXT)
                s.blit(img, img.get_rect(center=r.center))
            else:
                limg = self.font.render(label, True, COL_TEXT)
                s.blit(limg, (r.x + 14, r.centery - limg.get_height() // 2))
                vimg = self.font.render("< %s >" % value, True,
                                        (240, 210, 120) if selected else COL_MUTE)
                s.blit(vimg, (r.right - vimg.get_width() - 14,
                              r.centery - vimg.get_height() // 2))

        hint = self.font.render(t("inv.setup_hint"), True, COL_MUTE)
        s.blit(hint, hint.get_rect(center=(self.width // 2, self.height - 24)))

    def _draw_alien(self, s, a):
        col = KIND_COL.get(a["kind"], (235, 110, 110))
        # Schaden abdunkeln
        if a["maxhp"] > 1:
            g = 0.4 + 0.6 * (a["hp"] / a["maxhp"])
            col = (int(col[0] * g), int(col[1] * g), int(col[2] * g))
        x = int(a["cx"] - a["w"] / 2)
        y = int(a["cy"] - a["h"] / 2)
        pygame.draw.rect(s, col, (x, y, a["w"], a["h"]), border_radius=6)
        # Augen
        ey = y + a["h"] // 3
        pygame.draw.rect(s, COL_BG, (x + 7, ey, 4, 4))
        pygame.draw.rect(s, COL_BG, (x + a["w"] - 11, ey, 4, 4))
        if a["kind"] == "boss":
            # HP-Balken über dem Boss
            bw = a["w"]
            pygame.draw.rect(s, (60, 60, 70), (x, y - 10, bw, 5))
            pygame.draw.rect(s, (235, 90, 140),
                             (x, y - 10, int(bw * a["hp"] / a["maxhp"]), 5))

    def _draw_powerup(self, s, p):
        icons = {"life": ("+", (240, 110, 120)), "shield": ("S", (90, 170, 220)),
                 "weapon": ("W", (240, 210, 120))}
        letter, col = icons.get(p["kind"], ("?", COL_TEXT))
        pygame.draw.circle(s, col, (int(p["x"]), int(p["y"])), 11, 2)
        img = self.font.render(letter, True, col)
        s.blit(img, img.get_rect(center=(int(p["x"]), int(p["y"]))))

    def _draw_player(self, s):
        # Blinken während der Unverwundbarkeit nach einem Treffer.
        if self.invuln_timer > 0 and int(self.invuln_timer * 12) % 2 == 0:
            col = COL_PLAYER_HIT
        else:
            col = COL_PLAYER
        fx, fy = self.face
        ang = math.atan2(fy, fx)
        # Dreieck (Nase in Blickrichtung)
        tip = (self.px + math.cos(ang) * PLAYER_H * 0.8,
               self.py + math.sin(ang) * PLAYER_H * 0.8)
        left = (self.px + math.cos(ang + 2.5) * PLAYER_W * 0.5,
                self.py + math.sin(ang + 2.5) * PLAYER_W * 0.5)
        right = (self.px + math.cos(ang - 2.5) * PLAYER_W * 0.5,
                 self.py + math.sin(ang - 2.5) * PLAYER_W * 0.5)
        pygame.draw.polygon(s, col, (tip, left, right))

        # Schutzschild-Ring
        if self.shield_timer > 0:
            r = int(max(PLAYER_W, PLAYER_H) * 0.8)
            pygame.draw.circle(s, COL_SHIELD, (int(self.px), int(self.py)), r, 2)

    def _draw_hud(self, s):
        s.blit(self.font.render(t("common.points", score=self.score), True, COL_TEXT),
               (10, 6))
        lvl = self.font.render(t("inv.level", level=self.level), True, COL_TEXT)
        s.blit(lvl, (self.width // 2 - lvl.get_width() // 2, 6))
        leben = self.font.render(t("inv.lives", lives=self.lives), True, COL_TEXT)
        s.blit(leben, (self.width - leben.get_width() - 10, 6))

        # Waffe (unten links), mit befristeter Restzeit falls Upgrade.
        wname = t("inv.wpn_" + self.weapon)
        if self.weapon_timer > 0:
            wname += f"  {self.weapon_timer:0.0f}s"
        wimg = self.font.render(t("inv.weapon", weapon=wname), True,
                                WEAPON_COL.get(self.weapon, COL_TEXT))
        s.blit(wimg, (10, self.height - 26))

    def _draw_banner(self, s):
        if self._banner_t <= 0:
            return
        img = self.big_font.render(self._banner, True, COL_TEXT)
        s.blit(img, img.get_rect(center=(self.width // 2, self.height // 2 - 30)))
        if self.mouse_aim:
            hint = t("inv.hint_mouse")
        elif self.arena:
            hint = t("inv.hint_free")
        elif self.free_move:
            hint = t("inv.hint_classic_free")
        else:
            hint = t("inv.hint_classic")
        himg = self.font.render(hint, True, COL_MUTE)
        s.blit(himg, himg.get_rect(center=(self.width // 2, self.height // 2 + 16)))
