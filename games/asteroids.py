# -*- coding: utf-8 -*-
"""
asteroids.py
============
Asteroids - Einzelspieler oder Koop-Duell (2 Schiffe gleichzeitig).

Features
--------
- Klassische Vektor-Optik: Dreieck-Schiffe mit Schubflamme, zackige
  Polygon-Brocken (jeder Fels hat eine eigene Zufallsform und rotiert),
  Sternenhimmel im Hintergrund.
- ECHTE TRÄGHEITSPHYSIK: Hoch = Schub in Blickrichtung, Links/Rechts = drehen,
  das Schiff driftet weiter (leichte Dämpfung, Tempolimit), alles wickelt
  über die Bildschirmränder (Wrap-Around).
- Brocken zerspringen in zwei kleinere (3 Größen; 20/50/100 Punkte),
  WELLEN mit steigender Anzahl und Banner-Einblendung.
- UFO (im Setup abschaltbar): kreuzt regelmäßig den Bildschirm, zielt auf die
  Schiffe (Zielfehler je Schwierigkeit) - 200 Punkte für den Abschuss.
- POWER-UPS (abschaltbar), fallen aus zerstörten Brocken:
    * S = Schild (6 s unverwundbar)
    * T = Dreifachschuss (8 s)
    * R = Schnellfeuer (8 s, mehr Schüsse gleichzeitig)
- HYPERRAUM (Runter-Taste): Nottransport an eine Zufallsposition,
  4 s Abklingzeit - und 12 % Risiko, dabei zu zerschellen.
- 3 Leben, sicheres Respawnen (erst wenn der Startpunkt frei ist) mit
  Unverwundbarkeits-Blinken; EXTRALEBEN alle 5000 Punkte.
- Explosions-Partikel und Kamera-Shake; nach dem letzten Leben läuft die
  Explosion zu Ende, bevor der Game-Over-Schirm kommt.
- MEHRSPIELER = Koop-Duell: beide Schiffe fliegen gleichzeitig (P1 grün/WASD,
  P2 blau/IJKL), getrennte Leben und Punkte - wer mehr Punkte hat, gewinnt,
  Schluss ist erst, wenn beide keine Leben mehr haben.
- Setup-Screen: Schwierigkeit (Tempo/Anzahl der Brocken, UFO-Zielgenauigkeit),
  UFOs an/aus, Power-Ups an/aus (gespeichert in settings.json, "asteroids").
- Highscore = beste Einzelpunktzahl der Partie.
"""

import math
import random
import pygame

import settings as settings_mod
from game_base import Game, InputEvent
from i18n import t

COL_BG = (8, 10, 20)
COL_TEXT = (232, 234, 240)
COL_DIM = (140, 148, 168)
COL_ROCK = (205, 210, 225)
COL_ROCK_FILL = (26, 30, 44)
COL_P1 = (120, 230, 160)
COL_P2 = (140, 195, 255)
COL_BULLET = (245, 245, 250)
COL_UFO = (240, 150, 90)
COL_UFO_SHOT = (250, 110, 110)
COL_BTN = (44, 50, 66)
COL_BTN_ON = (60, 120, 80)
COL_ACCENT = (185, 194, 217)

SHIP_R = 12                  # Kollisionsradius des Schiffs
SHIP_TURN = 3.9              # Drehgeschwindigkeit (rad/s)
SHIP_THRUST = 300.0          # Schub (Pixel/s^2)
SHIP_DAMP = 0.35             # Dämpfung pro Sekunde (leichtes Ausrollen)
SHIP_MAX = 430.0             # Tempolimit
INVULN_T = 2.5               # Unverwundbarkeit nach dem Respawn
RESPAWN_T = 1.8              # Wartezeit bis zum Respawn
HYPER_CD = 4.0               # Abklingzeit Hyperraum
HYPER_RISK = 0.12            # Risiko, beim Sprung zu zerschellen
EXTRA_LIFE_EVERY = 5000      # Punkte bis zum nächsten Extraleben

BULLET_SPEED = 500.0
BULLET_LIFE = 0.95
SHOT_CD = 0.26               # Feuerpause (Schnellfeuer: 0.12)
MAX_BULLETS = 4              # gleichzeitige Schüsse je Schiff (Schnellfeuer: 8)

# Brocken: Größe -> (Radius, Grundtempo min/max, Punkte)
ROCKS = {3: (44, 40, 90, 20), 2: (26, 60, 130, 50), 1: (14, 90, 180, 100)}

UFO_R = 16
UFO_SHOT_SPEED = 320.0
UFO_POINTS = 200

POWERUP_KINDS = [
    ("shield", "S", (110, 220, 220), 6.0),
    ("triple", "T", (180, 140, 255), 8.0),
    ("rapid",  "R", (245, 205, 90), 8.0),
]
POWERUP_DROP = 0.18          # Drop-Chance je zerstörtem Brocken
POWERUP_LIFE = 9.0           # so lange liegt ein Power-Up herum

# Schwierigkeit: Brocken zu Wellenstart, Tempofaktor, UFO-Takt, UFO-Zielfehler
DIFFS = [
    dict(key="easy",   rocks=3, speed=0.75, ufo_every=30.0, ufo_aim=0.50),
    dict(key="medium", rocks=4, speed=1.00, ufo_every=24.0, ufo_aim=0.28),
    dict(key="hard",   rocks=5, speed=1.30, ufo_every=18.0, ufo_aim=0.12),
]

SETUP, PLAY = "setup", "play"


class _Ship:
    """Zustand eines Schiffs (Position, Drift, Leben, Power-Ups)."""

    def __init__(self, x, y, color):
        self.home = (x, y)          # Respawn-Punkt
        self.color = color
        self.x, self.y = x, y
        self.vx = self.vy = 0.0
        self.angle = -math.pi / 2   # Blick nach oben
        self.lives = 3
        self.score = 0
        self.next_extra = EXTRA_LIFE_EVERY
        self.alive = True
        self.invuln = INVULN_T
        self.respawn = 0.0
        self.cool = 0.0             # Feuerpause
        self.hyper_cd = 0.0
        self.powers = {}            # "shield"/"triple"/"rapid" -> Restzeit
        self.thrusting = False


class AsteroidsGame(Game):
    name = "Asteroids"
    highscore_key = "asteroids"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        self.winner = None

        a = self.settings.get("asteroids", {}) if isinstance(self.settings, dict) else {}
        self.diff = max(0, min(2, int(a.get("difficulty", 1))))
        self.ufos_on = bool(a.get("ufos", True))
        self.powerups_on = bool(a.get("powerups", True))

        self._small = pygame.font.SysFont("consolas", 16)
        self._tiny = pygame.font.SysFont("consolas", 13)
        self.anim_t = 0.0

        # Sternenhimmel (statisch, zwei Helligkeiten)
        self.stars = [(random.randrange(self.width), random.randrange(self.height),
                       random.choice((1, 1, 2)),
                       random.randint(60, 150)) for _ in range(70)]

        self._build_setup_layout()
        self.state = SETUP
        self._start_run()

    def _start_run(self):
        self.score = 0
        self.game_over = False
        self.winner = None
        if self.multiplayer:
            self.ships = [_Ship(self.width * 0.35, self.height / 2, COL_P1),
                          _Ship(self.width * 0.65, self.height / 2, COL_P2)]
        else:
            self.ships = [_Ship(self.width / 2, self.height / 2, COL_P1)]

        self.rocks = []              # dicts: x,y,vx,vy,size,r,shape,rot,spin
        self.bullets = []            # dicts: x,y,vx,vy,life,owner
        self.ufo_shots = []          # dicts: x,y,vx,vy,life
        self.items = []              # Power-Ups: x,y,vx,vy,kind,life
        self.particles = []
        self.ufo = None
        d = DIFFS[self.diff]
        self.ufo_timer = d["ufo_every"] * random.uniform(0.7, 1.1)

        self.wave = 1
        self.banner_t = 1.2
        self._pending_wave = True
        self.flash_msg = None        # (text, farbe, restzeit)
        self.shake = 0.0
        self._ending = 0.0           # Nachlauf zwischen letztem Tod und Game Over

        self._pressed = {"p1": set(), "p2": set()}

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(420, self.width - 60)
        self.diff_panel = pygame.Rect(cx - bw // 2, 108, bw, 56)
        self.diff_left = pygame.Rect(self.diff_panel.left, 108, 40, 56)
        self.diff_right = pygame.Rect(self.diff_panel.right - 40, 108, 40, 56)
        bh, gap = 42, 10
        y0 = 186
        self.ufo_rect = pygame.Rect(cx - bw // 2, y0, bw, bh)
        self.power_rect = pygame.Rect(cx - bw // 2, y0 + (bh + gap), bw, bh)
        self.start_rect = pygame.Rect(cx - 95, y0 + 2 * (bh + gap) + 8, 190, 50)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("asteroids", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _cycle_diff(self, step):
        self.diff = (self.diff + step) % len(DIFFS)
        self._save_setting("difficulty", self.diff)
        self.play_sound("click")

    def _toggle_ufos(self):
        self.ufos_on = not self.ufos_on
        self._save_setting("ufos", self.ufos_on)
        self.play_sound("select")

    def _toggle_powerups(self):
        self.powerups_on = not self.powerups_on
        self._save_setting("powerups", self.powerups_on)
        self.play_sound("select")

    def _start_play(self):
        self._start_run()
        self.state = PLAY
        self.play_sound("click")

    def _handle_setup_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Left", "a", "A"):
                self._cycle_diff(-1)
            elif event.key in ("Right", "d", "D"):
                self._cycle_diff(+1)
            elif event.key in ("u", "U"):
                self._toggle_ufos()
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
            elif self.ufo_rect.collidepoint(p):
                self._toggle_ufos()
            elif self.power_rect.collidepoint(p):
                self._toggle_powerups()
            elif self.start_rect.collidepoint(p):
                self._start_play()

    # ===================================================== Eingabe
    def _scheme_ship(self, scheme):
        """Welches Schiff steuert die Belegung? Einzelspieler: beide -> Schiff 0."""
        if self.multiplayer and scheme == "p2":
            return self.ships[1]
        return self.ships[0]

    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup_event(event)
            return

        if event.kind == InputEvent.KEYUP:
            for scheme in ("p1", "p2"):
                for act in ("left", "right", "up"):
                    if self.is_action(event.key, act, scheme):
                        self._pressed[scheme].discard(act)
            return

        if event.kind != InputEvent.KEYDOWN:
            return

        if self.game_over:
            if event.key in ("Return", "space"):
                self._start_play()
            elif event.key in ("s", "S"):
                self.state = SETUP
                self.play_sound("click")
            return

        for scheme in ("p1", "p2"):
            for act in ("left", "right", "up"):
                if self.is_action(event.key, act, scheme):
                    self._pressed[scheme].add(act)
            if self.is_action(event.key, "action", scheme):
                self._try_shoot(self._scheme_ship(scheme))
            if self.is_action(event.key, "down", scheme):
                self._hyperspace(self._scheme_ship(scheme))

    def _ship_input(self, i):
        """Gedrückte Richtungs-Aktionen für Schiff i (Einzelspieler: p1+p2)."""
        if self.multiplayer:
            return self._pressed["p1"] if i == 0 else self._pressed["p2"]
        return self._pressed["p1"] | self._pressed["p2"]

    # ----- Aktionen -----------------------------------------------------
    def _try_shoot(self, ship):
        if not ship.alive or ship.cool > 0:
            return
        max_b = 8 if "rapid" in ship.powers else MAX_BULLETS
        eigene = sum(1 for b in self.bullets if b["owner"] is ship)
        if eigene >= max_b:
            return
        winkel = [ship.angle]
        if "triple" in ship.powers:
            winkel = [ship.angle - 0.26, ship.angle, ship.angle + 0.26]
        for a in winkel:
            self.bullets.append(dict(
                x=ship.x + math.cos(a) * 14, y=ship.y + math.sin(a) * 14,
                vx=math.cos(a) * BULLET_SPEED + ship.vx,
                vy=math.sin(a) * BULLET_SPEED + ship.vy,
                life=BULLET_LIFE, owner=ship))
        ship.cool = 0.12 if "rapid" in ship.powers else SHOT_CD
        self.play_sound("shoot")

    def _hyperspace(self, ship):
        """Notsprung an eine Zufallsposition - mit Restrisiko."""
        if not ship.alive or ship.hyper_cd > 0:
            return
        self._spawn_particles(ship.x, ship.y, ship.color, 10)
        ship.x = random.uniform(40, self.width - 40)
        ship.y = random.uniform(40, self.height - 40)
        ship.vx *= 0.25
        ship.vy *= 0.25
        ship.hyper_cd = HYPER_CD
        self.play_sound("rotate")
        if random.random() < HYPER_RISK:
            self._kill_ship(ship)
        else:
            self._spawn_particles(ship.x, ship.y, ship.color, 10)

    # ===================================================== Spiellogik
    def update(self, dt):
        self.anim_t += dt
        self._update_particles(dt)
        if self.shake > 0:
            self.shake = max(0.0, self.shake - dt * 1.6)
        if self.state != PLAY or self.game_over:
            return

        if self.flash_msg is not None:
            text, farbe, rest = self.flash_msg
            self.flash_msg = (text, farbe, rest - dt) if rest > dt else None

        # Wellen-Banner / nächste Welle spawnen
        if self.banner_t > 0:
            self.banner_t -= dt
            if self.banner_t <= 0 and self._pending_wave:
                self._spawn_wave()
                self._pending_wave = False

        self._update_ships(dt)
        self._update_bullets(dt)
        for r in self.rocks:
            r["x"], r["y"] = self._wrap(r["x"] + r["vx"] * dt,
                                        r["y"] + r["vy"] * dt, r["r"])
            r["rot"] += r["spin"] * dt
        self._update_items(dt)
        if self.ufos_on:
            self._update_ufo(dt)
        self._collisions()

        # Welle geschafft?
        if not self.rocks and not self._pending_wave and self.ufo is None \
                and self.banner_t <= 0 and self._ending <= 0:
            self.wave += 1
            self.banner_t = 2.0
            self._pending_wave = True
            self.play_sound("level")

        # Nachlauf nach dem letzten Tod, dann Game Over
        if self._ending > 0:
            self._ending -= dt
            if self._ending <= 0:
                self._finish()

    def _wrap(self, x, y, m):
        """Wickelt eine Position mit Rand m über die Bildschirmkanten."""
        if x < -m:
            x += self.width + 2 * m
        elif x > self.width + m:
            x -= self.width + 2 * m
        if y < -m:
            y += self.height + 2 * m
        elif y > self.height + m:
            y -= self.height + 2 * m
        return x, y

    # ----- Schiffe ---------------------------------------------------------
    def _update_ships(self, dt):
        for i, sh in enumerate(self.ships):
            sh.cool = max(0.0, sh.cool - dt)
            sh.hyper_cd = max(0.0, sh.hyper_cd - dt)
            for k in list(sh.powers):
                sh.powers[k] -= dt
                if sh.powers[k] <= 0:
                    del sh.powers[k]

            if not sh.alive:
                # Respawn, sobald die Wartezeit um und der Startpunkt frei ist
                if sh.lives > 0:
                    sh.respawn -= dt
                    if sh.respawn <= 0 and self._area_clear(sh.home, 130):
                        sh.x, sh.y = sh.home
                        sh.vx = sh.vy = 0.0
                        sh.angle = -math.pi / 2
                        sh.alive = True
                        sh.invuln = INVULN_T
                continue

            sh.invuln = max(0.0, sh.invuln - dt)
            tasten = self._ship_input(i)
            if "left" in tasten:
                sh.angle -= SHIP_TURN * dt
            if "right" in tasten:
                sh.angle += SHIP_TURN * dt
            sh.thrusting = "up" in tasten
            if sh.thrusting:
                sh.vx += math.cos(sh.angle) * SHIP_THRUST * dt
                sh.vy += math.sin(sh.angle) * SHIP_THRUST * dt
                if random.random() < dt * 40:
                    # Flammen-Partikel nach hinten
                    ba = sh.angle + math.pi + random.uniform(-0.4, 0.4)
                    self.particles.append(
                        [sh.x + math.cos(ba) * 12, sh.y + math.sin(ba) * 12,
                         math.cos(ba) * 90 + sh.vx * 0.4,
                         math.sin(ba) * 90 + sh.vy * 0.4,
                         random.uniform(0.15, 0.3), (250, 180, 90)])
            # Dämpfung + Tempolimit
            f = max(0.0, 1.0 - SHIP_DAMP * dt)
            sh.vx *= f
            sh.vy *= f
            sp = math.hypot(sh.vx, sh.vy)
            if sp > SHIP_MAX:
                sh.vx *= SHIP_MAX / sp
                sh.vy *= SHIP_MAX / sp
            sh.x, sh.y = self._wrap(sh.x + sh.vx * dt, sh.y + sh.vy * dt, SHIP_R)

    def _area_clear(self, pos, radius):
        return all(math.hypot(r["x"] - pos[0], r["y"] - pos[1]) > radius + r["r"]
                   for r in self.rocks)

    def _kill_ship(self, ship):
        if not ship.alive:
            return
        ship.alive = False
        ship.lives -= 1
        ship.respawn = RESPAWN_T
        ship.powers.clear()
        self.shake = 0.55
        self._spawn_particles(ship.x, ship.y, ship.color, 26)
        self.play_sound("hit")
        self.play_sound("explode")
        self.rumble(220)
        if all(s.lives <= 0 and not s.alive for s in self.ships):
            self._ending = 1.4       # Explosion zu Ende spielen lassen

    def _finish(self):
        self.game_over = True
        self.score = max(s.score for s in self.ships)
        if self.multiplayer:
            s0, s1 = self.ships[0].score, self.ships[1].score
            self.winner = 0 if s0 > s1 else (1 if s1 > s0 else None)
            self.play_sound("win")
        else:
            self.play_sound("gameover")
        self.rumble(250)

    # ----- Punkte / Extraleben ----------------------------------------------
    def _add_points(self, ship, pts):
        ship.score += pts
        self.score = max(s.score for s in self.ships)
        if ship.score >= ship.next_extra:
            ship.next_extra += EXTRA_LIFE_EVERY
            ship.lives += 1
            self.flash_msg = (t("ast.extra_life"), ship.color, 2.0)
            self.play_sound("powerup")

    # ----- Brocken -----------------------------------------------------------
    def _spawn_wave(self):
        d = DIFFS[self.diff]
        anzahl = min(10, d["rocks"] + self.wave - 1)
        for _ in range(anzahl):
            self._spawn_rock(3)

    def _spawn_rock(self, size, pos=None):
        r, v0, v1, _pts = ROCKS[size]
        if pos is None:
            # Am Rand spawnen, mit Abstand zu allen Schiffen
            for _ in range(60):
                if random.random() < 0.5:
                    x = random.choice((-r, self.width + r))
                    y = random.uniform(0, self.height)
                else:
                    x = random.uniform(0, self.width)
                    y = random.choice((-r, self.height + r))
                if all(math.hypot(x - s.x, y - s.y) > 150 for s in self.ships):
                    break
            pos = (x, y)
        ang = random.uniform(0, math.tau)
        spd = random.uniform(v0, v1) * DIFFS[self.diff]["speed"]
        self.rocks.append(dict(
            x=pos[0], y=pos[1],
            vx=math.cos(ang) * spd, vy=math.sin(ang) * spd,
            size=size, r=r,
            shape=[random.uniform(0.72, 1.12) for _ in range(11)],
            rot=random.uniform(0, math.tau),
            spin=random.uniform(-1.6, 1.6)))

    def _break_rock(self, rock, ship):
        """Brocken zerstören: Punkte, Kinder, Partikel, evtl. Power-Up."""
        pts = ROCKS[rock["size"]][3]
        if ship is not None:
            self._add_points(ship, pts)
        self.rocks.remove(rock)
        farbe = (200, 205, 220)
        self._spawn_particles(rock["x"], rock["y"], farbe, 6 + rock["size"] * 4)
        self.play_sound("explode" if rock["size"] == 3 else "hit")
        if rock["size"] > 1:
            for _ in range(2):
                self._spawn_rock(rock["size"] - 1, pos=(rock["x"], rock["y"]))
        if self.powerups_on and len(self.items) < 2 \
                and random.random() < POWERUP_DROP:
            kind = random.randrange(len(POWERUP_KINDS))
            ang = random.uniform(0, math.tau)
            self.items.append(dict(
                x=rock["x"], y=rock["y"],
                vx=math.cos(ang) * 30, vy=math.sin(ang) * 30,
                kind=kind, life=POWERUP_LIFE))

    # ----- Schüsse / Power-Ups / UFO -----------------------------------------
    def _update_bullets(self, dt):
        rest = []
        for b in self.bullets:
            b["life"] -= dt
            if b["life"] <= 0:
                continue
            b["x"], b["y"] = self._wrap(b["x"] + b["vx"] * dt,
                                        b["y"] + b["vy"] * dt, 4)
            rest.append(b)
        self.bullets = rest
        rest = []
        for b in self.ufo_shots:
            b["life"] -= dt
            if b["life"] <= 0:
                continue
            b["x"], b["y"] = self._wrap(b["x"] + b["vx"] * dt,
                                        b["y"] + b["vy"] * dt, 4)
            rest.append(b)
        self.ufo_shots = rest

    def _update_items(self, dt):
        rest = []
        for it in self.items:
            it["life"] -= dt
            if it["life"] <= 0:
                continue
            it["x"], it["y"] = self._wrap(it["x"] + it["vx"] * dt,
                                          it["y"] + it["vy"] * dt, 14)
            rest.append(it)
        self.items = rest

    def _update_ufo(self, dt):
        if self.ufo is None:
            self.ufo_timer -= dt
            if self.ufo_timer <= 0 and self.banner_t <= 0:
                richtung = random.choice((-1, 1))
                base_y = random.uniform(60, self.height - 60)
                self.ufo = dict(
                    x=-UFO_R if richtung > 0 else self.width + UFO_R,
                    y=base_y, base_y=base_y,
                    dir=richtung, t=0.0, shoot=random.uniform(0.8, 1.4))
                self.play_sound("move")
            return
        u = self.ufo
        u["t"] += dt
        u["x"] += u["dir"] * (95 + self.wave * 4) * dt
        u["y"] = u["base_y"] + math.sin(u["t"] * 2.2) * 40
        # verschwindet am anderen Rand
        if (u["dir"] > 0 and u["x"] > self.width + UFO_R) \
                or (u["dir"] < 0 and u["x"] < -UFO_R):
            self.ufo = None
            self.ufo_timer = DIFFS[self.diff]["ufo_every"] * random.uniform(0.8, 1.2)
            return
        # gezielter Schuss auf ein lebendes Schiff
        u["shoot"] -= dt
        if u["shoot"] <= 0:
            u["shoot"] = 1.3
            ziele = [s for s in self.ships if s.alive]
            if ziele:
                z = random.choice(ziele)
                a = math.atan2(z.y - u["y"], z.x - u["x"])
                a += random.uniform(-1, 1) * DIFFS[self.diff]["ufo_aim"]
                self.ufo_shots.append(dict(
                    x=u["x"], y=u["y"],
                    vx=math.cos(a) * UFO_SHOT_SPEED,
                    vy=math.sin(a) * UFO_SHOT_SPEED, life=1.6))
                self.play_sound("shoot")

    # ----- Kollisionen ---------------------------------------------------------
    def _collisions(self):
        # Spielerschüsse gegen Brocken / UFO
        for b in list(self.bullets):
            getroffen = None
            for r in self.rocks:
                if math.hypot(b["x"] - r["x"], b["y"] - r["y"]) < r["r"]:
                    getroffen = r
                    break
            if getroffen is not None:
                self.bullets.remove(b)
                self._break_rock(getroffen, b["owner"])
                continue
            if self.ufo is not None and math.hypot(
                    b["x"] - self.ufo["x"], b["y"] - self.ufo["y"]) < UFO_R + 2:
                self.bullets.remove(b)
                self._spawn_particles(self.ufo["x"], self.ufo["y"], COL_UFO, 18)
                self._add_points(b["owner"], UFO_POINTS)
                self.ufo = None
                self.ufo_timer = DIFFS[self.diff]["ufo_every"] * random.uniform(0.9, 1.3)
                self.play_sound("point")
                self.play_sound("explode")

        for sh in self.ships:
            if not sh.alive:
                continue
            verwundbar = sh.invuln <= 0 and "shield" not in sh.powers
            # Brocken
            if verwundbar:
                for r in self.rocks:
                    if math.hypot(sh.x - r["x"], sh.y - r["y"]) < r["r"] * 0.9 + SHIP_R:
                        self._kill_ship(sh)
                        break
            if not sh.alive:
                continue
            # UFO-Schüsse und UFO-Rumpf
            if verwundbar:
                for b in list(self.ufo_shots):
                    if math.hypot(sh.x - b["x"], sh.y - b["y"]) < SHIP_R + 3:
                        self.ufo_shots.remove(b)
                        self._kill_ship(sh)
                        break
            if not sh.alive:
                continue
            if verwundbar and self.ufo is not None and math.hypot(
                    sh.x - self.ufo["x"], sh.y - self.ufo["y"]) < UFO_R + SHIP_R:
                self._kill_ship(sh)
                continue
            # Power-Ups einsammeln
            for it in list(self.items):
                if math.hypot(sh.x - it["x"], sh.y - it["y"]) < SHIP_R + 14:
                    kind, _sym, farbe, dauer = POWERUP_KINDS[it["kind"]]
                    sh.powers[kind] = dauer
                    self.items.remove(it)
                    self._spawn_particles(it["x"], it["y"], farbe, 10)
                    self.play_sound("powerup")

    # ----- Partikel -------------------------------------------------------------
    def _spawn_particles(self, x, y, color, n):
        for _ in range(n):
            ang = random.uniform(0, math.tau)
            spd = random.uniform(40, 260)
            self.particles.append([x, y, math.cos(ang) * spd, math.sin(ang) * spd,
                                   random.uniform(0.25, 0.7), color])

    def _update_particles(self, dt):
        rest = []
        for p in self.particles:
            p[0] += p[2] * dt
            p[1] += p[3] * dt
            p[2] *= 0.985
            p[3] *= 0.985
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
        ox = oy = 0
        if self.shake > 0:
            amp = 10 * self.shake
            ox = random.uniform(-amp, amp)
            oy = random.uniform(-amp, amp)

        for (x, y, gr, hell) in self.stars:
            tw = hell + int(30 * math.sin(self.anim_t * 1.5 + x))
            c = max(40, min(200, tw))
            pygame.draw.circle(s, (c, c, min(255, c + 25)), (x, y), gr)

        for p in self.particles:
            a = max(0.0, min(1.0, p[4] / 0.7))
            col = tuple(int(c * a + COL_BG[i] * (1 - a))
                        for i, c in enumerate(p[5]))
            pygame.draw.circle(s, col, (int(p[0] + ox), int(p[1] + oy)), 2)

        for r in self.rocks:
            self._draw_rock(s, r, ox, oy)
        for it in self.items:
            self._draw_item(s, it, ox, oy)
        if self.ufo is not None:
            self._draw_ufo(s, ox, oy)
        for b in self.bullets:
            pygame.draw.circle(s, COL_BULLET, (int(b["x"] + ox), int(b["y"] + oy)), 2)
        for b in self.ufo_shots:
            pygame.draw.circle(s, COL_UFO_SHOT, (int(b["x"] + ox), int(b["y"] + oy)), 3)
        for sh in self.ships:
            self._draw_ship(s, sh, ox, oy)

        self._draw_hud(s)
        if self.game_over:
            self._draw_game_over()

    def _draw_rock(self, s, r, ox, oy):
        pts = []
        n = len(r["shape"])
        for i, f in enumerate(r["shape"]):
            a = r["rot"] + i * math.tau / n
            pts.append((r["x"] + math.cos(a) * r["r"] * f + ox,
                        r["y"] + math.sin(a) * r["r"] * f + oy))
        pygame.draw.polygon(s, COL_ROCK_FILL, pts)
        pygame.draw.polygon(s, COL_ROCK, pts, 2)

    def _draw_ship(self, s, sh, ox, oy):
        if not sh.alive:
            return
        # Unverwundbar: blinken
        if sh.invuln > 0 and int(self.anim_t * 12) % 2 == 0:
            return
        a = sh.angle
        x, y = sh.x + ox, sh.y + oy
        nase = (x + math.cos(a) * 16, y + math.sin(a) * 16)
        hl = (x + math.cos(a + 2.5) * 13, y + math.sin(a + 2.5) * 13)
        hr = (x + math.cos(a - 2.5) * 13, y + math.sin(a - 2.5) * 13)
        heck = (x + math.cos(a + math.pi) * 6, y + math.sin(a + math.pi) * 6)
        dunkel = tuple(c // 3 for c in sh.color)
        pygame.draw.polygon(s, dunkel, (nase, hl, heck, hr))
        pygame.draw.polygon(s, sh.color, (nase, hl, heck, hr), 2)
        # Schubflamme (flackert)
        if sh.thrusting and random.random() < 0.85:
            fl = 10 + random.uniform(0, 8)
            fx = (x + math.cos(a + math.pi) * (8 + fl),
                  y + math.sin(a + math.pi) * (8 + fl))
            pygame.draw.polygon(s, (250, 180, 90),
                                (hl, fx, hr))
        if "shield" in sh.powers:
            r = 20 + int(2 * math.sin(self.anim_t * 8))
            pygame.draw.circle(s, (110, 220, 220), (int(x), int(y)), r, 2)

    def _draw_ufo(self, s, ox, oy):
        u = self.ufo
        x, y = int(u["x"] + ox), int(u["y"] + oy)
        pygame.draw.ellipse(s, (40, 30, 26), (x - 18, y - 6, 36, 14))
        pygame.draw.ellipse(s, COL_UFO, (x - 18, y - 6, 36, 14), 2)
        pygame.draw.arc(s, COL_UFO, (x - 9, y - 14, 18, 16), 0, math.pi, 2)
        pygame.draw.line(s, COL_UFO, (x - 18, y + 1), (x + 18, y + 1), 1)

    def _draw_item(self, s, it, ox, oy):
        kind, sym, farbe, _d = POWERUP_KINDS[it["kind"]]
        # kurz vor dem Verschwinden blinken
        if it["life"] < 2.5 and int(self.anim_t * 6) % 2 == 0:
            return
        x, y = int(it["x"] + ox), int(it["y"] + oy)
        r = 12 + int(1.5 * math.sin(self.anim_t * 5))
        pygame.draw.circle(s, (22, 27, 40), (x, y), r)
        pygame.draw.circle(s, farbe, (x, y), r, 2)
        img = self._tiny.render(sym, True, farbe)
        s.blit(img, img.get_rect(center=(x, y)))

    # ----- HUD -----------------------------------------------------------------
    def _draw_hud(self, s):
        for i, sh in enumerate(self.ships):
            rechts = (i == 1)
            x = self.width - 12 if rechts else 12
            img = self.font.render(str(sh.score), True, sh.color)
            s.blit(img, (x - img.get_width() if rechts else x, 8))
            # Leben als kleine Schiffssymbole
            for lv in range(sh.lives):
                lx = x - 14 - lv * 16 if rechts else x + 6 + lv * 16
                self._draw_life_icon(s, lx, 44, sh.color)
            # Aktive Power-Ups
            y = 58
            for kind, sym, farbe, _d in POWERUP_KINDS:
                if kind in sh.powers:
                    txt = self._tiny.render(f"{sym} {sh.powers[kind]:2.0f}s",
                                            True, farbe)
                    s.blit(txt, (x - txt.get_width() if rechts else x, y))
                    y += 15
            # Hyperraum-Anzeige
            if sh.alive:
                if sh.hyper_cd <= 0:
                    txt = self._tiny.render(t("ast.hyper"), True, (120, 200, 150))
                else:
                    txt = self._tiny.render(f"{t('ast.hyper')} {sh.hyper_cd:.0f}s",
                                            True, COL_DIM)
                s.blit(txt, (x - txt.get_width() if rechts else x, y))

        w = self._small.render(t("ast.wave", n=self.wave), True, COL_ACCENT)
        s.blit(w, w.get_rect(midtop=(self.width // 2, 8)))

        if self.banner_t > 0 and not self.game_over:
            img = self.big_font.render(t("ast.wave", n=self.wave), True, COL_ACCENT)
            img.set_alpha(max(0, min(255, int(255 * self.banner_t))))
            s.blit(img, img.get_rect(center=(self.width // 2, self.height // 2 - 40)))

        if self.flash_msg is not None:
            text, farbe, _rest = self.flash_msg
            img = self.font.render(text, True, farbe)
            s.blit(img, img.get_rect(midtop=(self.width // 2, 40)))

        if self.anim_t < 6:
            hint = self._tiny.render(t("ast.controls_hint"), True, COL_DIM)
            s.blit(hint, hint.get_rect(midbottom=(self.width // 2, self.height - 8)))

    def _draw_life_icon(self, s, x, y, color):
        pygame.draw.polygon(s, color,
                            ((x + 5, y - 6), (x, y + 6), (x + 5, y + 3),
                             (x + 10, y + 6)), 1)

    def _draw_game_over(self):
        ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        ov.fill((0, 0, 0, 150))
        self.surface.blit(ov, (0, 0))
        if self.multiplayer:
            if self.winner is None:
                text, farbe = t("common.draw"), COL_TEXT
            else:
                text = t("common.player_wins", n=self.winner + 1)
                farbe = self.ships[self.winner].color
            self.draw_center_text(text, self.big_font, farbe, -50)
            stand = f"P1: {self.ships[0].score}    P2: {self.ships[1].score}"
            self.draw_center_text(stand, self.font, COL_TEXT, -6)
        else:
            self.draw_center_text(t("common.game_over"), self.big_font,
                                  (245, 110, 110), -50)
            self.draw_center_text(t("common.points", score=self.ships[0].score),
                                  self.font, COL_TEXT, -6)
        self.draw_center_text(t("ast.wave_reached", n=self.wave),
                              self.font, COL_DIM, 24)
        self.draw_center_text(t("ah.restart_hint"), self.font, COL_TEXT, 56)

    # ----- Setup zeichnen ---------------------------------------------------
    def _draw_setup(self):
        s = self.surface
        s.fill(COL_BG)
        for (x, y, gr, hell) in self.stars:
            pygame.draw.circle(s, (hell, hell, min(255, hell + 25)), (x, y), gr)

        title = self.big_font.render("ASTEROIDS", True, COL_TEXT)
        s.blit(title, title.get_rect(center=(self.width // 2, 50)))
        modus = t("snake.multiplayer") if self.multiplayer else t("snake.singleplayer")
        sub = self._small.render(modus, True, COL_DIM)
        s.blit(sub, sub.get_rect(center=(self.width // 2, 86)))

        d = DIFFS[self.diff]
        pygame.draw.rect(s, (38, 44, 60), self.diff_panel, border_radius=10)
        pygame.draw.rect(s, COL_ACCENT, self.diff_panel, 2, border_radius=10)
        name = self.font.render(
            t("ah.difficulty") + ":  " + t("ah.diff." + d["key"]), True, COL_TEXT)
        s.blit(name, name.get_rect(center=(self.diff_panel.centerx,
                                           self.diff_panel.top + 19)))
        info = self._tiny.render(t("ast.diff_note"), True, COL_DIM)
        s.blit(info, info.get_rect(center=(self.diff_panel.centerx,
                                           self.diff_panel.top + 41)))
        for r, sym in ((self.diff_left, "<"), (self.diff_right, ">")):
            arr = self.big_font.render(sym, True, COL_ACCENT)
            s.blit(arr, arr.get_rect(center=r.center))

        self._draw_row(self.ufo_rect, t("ast.ufos"),
                       t("common.on") if self.ufos_on else t("common.off"),
                       self.ufos_on)
        self._draw_row(self.power_rect, t("ah.powerups"),
                       t("common.on") if self.powerups_on else t("common.off"),
                       self.powerups_on)

        pygame.draw.rect(s, COL_BTN_ON, self.start_rect, border_radius=10)
        st = self.font.render(t("common.start"), True, COL_TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))

        hint = self._small.render(t("ast.setup_hint"), True, COL_DIM)
        s.blit(hint, hint.get_rect(center=(self.width // 2, self.height - 34)))
        h2 = self._tiny.render(t("ast.controls_hint"), True, (120, 200, 150))
        s.blit(h2, h2.get_rect(center=(self.width // 2, self.height - 14)))

    def _draw_row(self, rect, label, wert, an):
        s = self.surface
        pygame.draw.rect(s, COL_BTN_ON if an else COL_BTN, rect, border_radius=8)
        pygame.draw.rect(s, COL_DIM, rect, 1, border_radius=8)
        lab = self.font.render(label, True, COL_TEXT)
        s.blit(lab, (rect.x + 16, rect.centery - lab.get_height() // 2))
        img = self.font.render(f"< {wert} >", True,
                               COL_ACCENT if an else COL_DIM)
        s.blit(img, (rect.right - img.get_width() - 16,
                     rect.centery - img.get_height() // 2))
