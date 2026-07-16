# -*- coding: utf-8 -*-
"""
trexrunner.py
=============
T-Rex Runner - der endlose Wuestenlauf (Hommage an das Chrome-Dino-Spiel).

- Ein T-Rex laeuft von links, du springst ueber Kakteen und duckst dich unter
  Flugsauriern hindurch. Das Tempo steigt kontinuierlich mit der Strecke.
- Variable Sprunghoehe: je laenger du die Sprungtaste haeltst, desto hoeher
  springt der Dino (kuerzer Antippen = kleiner Hopser). In der Luft nach unten
  = schneller Fall.
- Flugsaurier tauchen auf drei Hoehen auf: hoch = ducken, tief/mittig = springen.
- Tag/Nacht-Wechsel mit Sternenhimmel und Mond, Parallax-Wolken, scrollender
  Boden mit Bodenwellen. Alle 100 Punkte ein kurzer Ton + Blinken.
- Optionen (bleiben erhalten, settings["trex"]): Schwierigkeit (chill/normal/
  hardcore), Figur/Skin und Tag/Nacht an/aus - im Startbildschirm umschaltbar.

Steuerung: Leertaste/Pfeil-hoch/Aktion = Springen (halten fuer hoeher),
Pfeil-runter = Ducken/schneller fallen. Nach Game Over: Enter/Leertaste = neu.
"""

import random

import pygame

import settings as settings_mod
from game_base import Game, InputEvent
from i18n import t

# ---- Farbpaletten (Tag / Nacht) - werden fuer weiche Uebergaenge interpoliert.
DAY = dict(sky=(244, 246, 250), ground=(84, 84, 92), line=(120, 120, 130),
           obj=(74, 78, 90), cloud=(210, 214, 224), star=(244, 246, 250),
           text=(60, 64, 78), dim=(120, 126, 140))
NIGHT = dict(sky=(24, 26, 42), ground=(150, 154, 168), line=(110, 116, 140),
             obj=(198, 202, 216), cloud=(70, 76, 104), star=(250, 250, 210),
             text=(226, 230, 242), dim=(150, 156, 178))

# Skins = Koerperfarbe des Dinos (Tag / Nacht wird davon leicht abgeleitet).
SKINS = [(94, 106, 122), (86, 200, 130), (240, 150, 70), (110, 160, 240)]

DIFFS = ["chill", "normal", "hardcore"]
# (Start-Tempo, Beschleunigung px/s^2, Vogel-ab-Score) - jeweils Design-Einheiten.
DIFF_PARAMS = {
    "chill":    (300.0, 5.0, 250),
    "normal":   (380.0, 8.0, 150),
    "hardcore": (470.0, 12.0, 60),
}

READY, RUN, OVER = "ready", "run", "over"

# Hindernis-Typen: (Kennung, ist_vogel, Hoehen-Offset-Faktor)
BIRD_HIGH, BIRD_MID, BIRD_LOW = "bird_high", "bird_mid", "bird_low"


class TRexRunnerGame(Game):
    name = "T-Rex"
    highscore_key = "trex"
    supports_multiplayer = False

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.game_over = False
        self.score = 0

        tr = self.settings.get("trex", {}) if isinstance(self.settings, dict) else {}
        self.diff = tr.get("difficulty", "normal")
        if self.diff not in DIFFS:
            self.diff = "normal"
        self.skin = max(0, min(len(SKINS) - 1, int(tr.get("skin", 1))))
        self.night_on = bool(tr.get("night", True))

        self._big = pygame.font.SysFont("consolas", max(26, self.height // 12),
                                        bold=True)
        self._mid = pygame.font.SysFont("consolas", 20, bold=True)
        self._small = pygame.font.SysFont("consolas", 15)
        self._tiny = pygame.font.SysFont("consolas", 13)
        self._mono = pygame.font.SysFont("consolas", 18, bold=True)

        import highscore
        self.best = highscore.load_highscores().get(self.highscore_key, 0)

        self._layout()
        self._new_run()
        self.state = READY

    def on_surface_changed(self):
        self._big = pygame.font.SysFont("consolas", max(26, self.height // 12),
                                        bold=True)
        self._layout()

    def _layout(self):
        self.scale = self.height / 480.0
        self.gy = int(self.height * 0.80)          # Bodenlinie (y)
        self.dino_x = int(self.width * 0.16)
        self.dino_w = int(44 * self.scale)
        self.dino_h = int(48 * self.scale)
        self.duck_w = int(60 * self.scale)
        self.duck_h = int(28 * self.scale)

    def _new_run(self):
        base, accel, _ = DIFF_PARAMS[self.diff]
        self.speed = base * self.scale
        self.base_speed = base * self.scale
        self.accel = accel * self.scale
        self.max_speed = 900 * self.scale
        self.dist = 0.0
        self.score = 0
        self.next_milestone = 100

        # Dino-Zustand
        self.dy = 0.0            # vertikaler Versatz ueber dem Boden (positiv = oben)
        self.vy = 0.0
        self.on_ground = True
        self.ducking = False
        self.jump_held = False
        self.duck_held = False
        self.run_frame = 0.0
        self.dead_t = 0.0
        self.blink = 0.0

        self.obstacles = []      # dicts: x, w, h, bird, oy, flap
        self.spawn_gap = self._rand_gap() * 0.6
        self.clouds = []
        for _ in range(3):
            self.clouds.append([random.uniform(0, self.width),
                                random.uniform(self.height * 0.12,
                                               self.height * 0.42),
                                random.uniform(0.25, 0.5)])
        self.stars = [(random.uniform(0, self.width),
                       random.uniform(0, self.gy * 0.7),
                       random.choice((1, 1, 2))) for _ in range(46)]
        self.ground_bumps = [random.uniform(0, self.width) for _ in range(24)]
        # Tag/Nacht: 0 = Tag, 1 = Nacht (weich interpoliert)
        self.night = 0.0
        self.night_target = 0.0
        self._next_flip = 300

    def _rand_gap(self):
        """Zufaelliger horizontaler Abstand zum naechsten Hindernis (px)."""
        # Mindestabstand skaliert mit dem Tempo (schnell = mehr Platz zum Reagieren).
        base = random.uniform(320, 560) * self.scale
        return base * (self.speed / self.base_speed) ** 0.5

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("trex", {})[key] = value
            settings_mod.save_settings(self.settings)

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == READY:
            self._handle_ready(event)
            return
        if self.state == OVER:
            if event.kind == InputEvent.KEYDOWN and \
                    event.key in ("Return", "space", "Up"):
                self._new_run()
                self.state = RUN
                self.play_sound("select")
            elif event.kind == InputEvent.MOUSEDOWN:
                self._new_run()
                self.state = RUN
                self.play_sound("select")
            return

        # ---- RUN
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("space", "Up", "w", "W") or self.is_action(k, "action") \
                    or self.is_action(k, "up"):
                self.jump_held = True
                self._try_jump()
            elif k in ("Down", "s", "S") or self.is_action(k, "down"):
                self.duck_held = True
        elif event.kind == InputEvent.KEYUP:
            k = event.key
            if k in ("space", "Up", "w", "W"):
                self.jump_held = False
            elif k in ("Down", "s", "S"):
                self.duck_held = False
        elif event.kind == InputEvent.MOUSEDOWN:
            self.jump_held = True
            self._try_jump()
        elif event.kind == InputEvent.MOUSEUP:
            self.jump_held = False

    def _handle_ready(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3"):
                self.diff = DIFFS[int(k) - 1]
                self._save_setting("difficulty", self.diff)
                self._new_run()
                self.play_sound("click")
            elif k in ("n", "N"):
                self.night_on = not self.night_on
                self._save_setting("night", self.night_on)
                self.play_sound("click")
            elif k in ("c", "C"):
                self.skin = (self.skin + 1) % len(SKINS)
                self._save_setting("skin", self.skin)
                self.play_sound("click")
            elif k in ("space", "Up", "Return", "w", "W") \
                    or self.is_action(k, "action"):
                self.state = RUN
                self.jump_held = True
                self._try_jump()
                self.play_sound("select")
        elif event.kind == InputEvent.MOUSEDOWN:
            self.state = RUN
            self.jump_held = True
            self._try_jump()
            self.play_sound("select")

    def _try_jump(self):
        if self.on_ground and self.state == RUN:
            self.vy = -620 * self.scale
            self.on_ground = False
            self.ducking = False
            self.play_sound("bounce")

    # ===================================================== Update
    def update(self, dt):
        dt = min(dt, 0.05)
        if self.blink > 0:
            self.blink -= dt
        for c in self.clouds:                      # Wolken driften immer
            c[0] -= self.base_speed * c[2] * dt * (0.4 if self.state != RUN else 1.0)
            if c[0] < -60 * self.scale:
                c[0] = self.width + random.uniform(20, 120) * self.scale
                c[1] = random.uniform(self.height * 0.12, self.height * 0.42)

        if self.state == OVER:
            self.dead_t += dt
            self._update_night(dt)
            return
        if self.state != RUN:
            self.run_frame += dt * 6
            self._update_night(dt)
            return

        # Tempo + Strecke
        self.speed = min(self.max_speed, self.speed + self.accel * dt)
        self.dist += self.speed * dt
        old = self.score
        self.score = int(self.dist / (10 * self.scale))
        if self.score // 100 > old // 100 and self.score >= self.next_milestone:
            self.next_milestone += 100
            self.blink = 0.6
            self.play_sound("point")

        self._update_dino(dt)
        self._update_obstacles(dt)
        self._update_night(dt)
        self.run_frame += dt * (6 + self.speed / (120 * self.scale))

    def _update_dino(self, dt):
        if self.on_ground:
            self.ducking = self.duck_held
        else:
            # Schwerkraft; beim Halten der Sprungtaste im Aufstieg schwaecher
            g = 1800 * self.scale
            if self.jump_held and self.vy < 0:
                g = 1150 * self.scale
            self.vy += g * dt
            if self.duck_held:                     # schneller Fall
                self.vy += 2600 * self.scale * dt
            self.dy -= self.vy * dt
            if self.dy <= 0:
                self.dy = 0.0
                self.vy = 0.0
                self.on_ground = True
                self.ducking = self.duck_held

    def _dino_rect(self):
        if self.ducking and self.on_ground:
            w, h = self.duck_w, self.duck_h
        else:
            w, h = self.dino_w, self.dino_h
        x = self.dino_x
        y = self.gy - int(self.dy) - h
        return pygame.Rect(x, y, w, h)

    def _update_obstacles(self, dt):
        for o in self.obstacles:
            o["x"] -= self.speed * dt
            if o["bird"]:
                o["flap"] += dt * 8
        self.obstacles = [o for o in self.obstacles if o["x"] + o["w"] > -4]

        # Nachschub, sobald genug Platz hinter dem letzten Hindernis ist.
        self.spawn_gap -= self.speed * dt
        if self.spawn_gap <= 0:
            self._spawn()
            self.spawn_gap = self._rand_gap()

        # Kollision
        dr = self._dino_rect().inflate(int(-8 * self.scale), int(-8 * self.scale))
        for o in self.obstacles:
            r = pygame.Rect(int(o["x"]), int(o["oy"]), int(o["w"]), int(o["h"]))
            r.inflate_ip(int(-5 * self.scale), int(-6 * self.scale))
            if dr.colliderect(r):
                self._die()
                return

    def _spawn(self):
        _, _, bird_at = DIFF_PARAMS[self.diff]
        allow_bird = self.score >= bird_at
        if allow_bird and random.random() < 0.28:
            kind = random.choice((BIRD_HIGH, BIRD_MID, BIRD_LOW))
            w = int(46 * self.scale)
            h = int(30 * self.scale)
            off = {BIRD_HIGH: 92, BIRD_MID: 58, BIRD_LOW: 26}[kind]
            oy = self.gy - int(off * self.scale) - h
            self.obstacles.append(dict(x=float(self.width + 20 * self.scale),
                                       w=w, h=h, bird=True, oy=oy,
                                       flap=0.0, kind=kind))
        else:
            variant = random.random()
            if variant < 0.4:
                w, h = int(18 * self.scale), int(36 * self.scale)
                spikes = 1
            elif variant < 0.72:
                w, h = int(26 * self.scale), int(50 * self.scale)
                spikes = 1
            else:                                  # Gruppe
                spikes = random.choice((2, 3))
                w, h = int((14 * spikes + 6) * self.scale), int(40 * self.scale)
            oy = self.gy - h
            self.obstacles.append(dict(x=float(self.width + 20 * self.scale),
                                       w=w, h=h, bird=False, oy=oy,
                                       spikes=spikes))

    def _update_night(self, dt):
        if not self.night_on:
            self.night += (0.0 - self.night) * min(1.0, dt * 3)
            return
        if self.state == RUN and self.score >= self._next_flip:
            self.night_target = 1.0 - self.night_target
            self._next_flip += 250
        self.night += (self.night_target - self.night) * min(1.0, dt * 1.4)

    def _die(self):
        self.state = OVER
        self.game_over = True                      # main.py sichert den Highscore
        self.dead_t = 0.0
        self.ducking = False
        if self.score > self.best:
            self.best = self.score
        self.play_sound("gameover")

    # ===================================================== Zeichnen
    def _pal(self, key):
        d, n = DAY[key], NIGHT[key]
        f = self.night
        return tuple(int(d[i] + (n[i] - d[i]) * f) for i in range(3))

    def draw(self):
        s = self.surface
        s.fill(self._pal("sky"))
        self._draw_sky(s)
        self._draw_ground(s)
        self._draw_obstacles(s)
        self._draw_dino(s)
        self._draw_hud(s)
        if self.state == READY:
            self._draw_ready(s)
        elif self.state == OVER:
            self._draw_over(s)

    def _draw_sky(self, s):
        # Sonne (Tag) / Mond (Nacht)
        cx = int(self.width * 0.82)
        cy = int(self.height * 0.20)
        r = int(24 * self.scale)
        if self.night > 0.35:
            star_col = self._pal("star")
            for (sx, sy, sr) in self.stars:
                s.fill(star_col, (int(sx), int(sy), sr, sr))
            pygame.draw.circle(s, (232, 234, 220), (cx, cy), r)
            pygame.draw.circle(s, self._pal("sky"),
                               (cx + int(9 * self.scale),
                                cy - int(6 * self.scale)), r)
        else:
            pygame.draw.circle(s, (250, 214, 120), (cx, cy), r)
            pygame.draw.circle(s, (250, 224, 150), (cx, cy), int(r * 1.5), 2)
        # Wolken
        col = self._pal("cloud")
        for (x, y, _sp) in self.clouds:
            self._draw_cloud(s, int(x), int(y), col)

    def _draw_cloud(self, s, x, y, col):
        u = int(9 * self.scale)
        pygame.draw.ellipse(s, col, (x, y, u * 5, u * 2))
        pygame.draw.ellipse(s, col, (x + u, y - u, u * 3, u * 2))

    def _draw_ground(self, s):
        gy = self.gy
        pygame.draw.line(s, self._pal("line"), (0, gy), (self.width, gy),
                         max(2, int(2 * self.scale)))
        # Bodenwellen / Kieselsteine, scrollend
        off = (self.dist * 0.5) % self.width if self.state != READY else 0
        col = self._pal("line")
        for bx in self.ground_bumps:
            x = int((bx - off) % self.width)
            pygame.draw.line(s, col, (x, gy + int(6 * self.scale)),
                             (x + int(10 * self.scale), gy + int(6 * self.scale)),
                             max(1, int(self.scale)))

    def _draw_dino(self, s):
        r = self._dino_rect()
        body = SKINS[self.skin]
        if self.night > 0.5:                       # nachts leicht aufhellen
            body = tuple(min(255, int(c + 60 * (self.night - 0.5) * 2))
                         for c in body)
        dark = tuple(int(c * 0.6) for c in body)
        u = self.scale
        if self.ducking and self.on_ground:
            # Geduckt: langgestreckter Koerper
            pygame.draw.rect(s, body, r, border_radius=int(6 * u))
            eye = (r.right - int(8 * u), r.y + int(7 * u))
            pygame.draw.circle(s, (250, 250, 250), eye, max(2, int(3 * u)))
            pygame.draw.circle(s, (20, 20, 20), eye, max(1, int(1.5 * u)))
            # Beinchen
            f = int(self.run_frame) % 2
            for i in range(2):
                lx = r.x + int((10 + i * 22) * u)
                ly = r.bottom
                dh = int((6 if (i + f) % 2 else 3) * u)
                pygame.draw.rect(s, dark, (lx, ly, int(5 * u), dh))
            return
        # Stehend/Springend: Koerper + Kopf + Schwanz
        head = pygame.Rect(r.right - int(20 * u), r.y, int(20 * u), int(18 * u))
        pygame.draw.rect(s, body, (r.x + int(6 * u), r.y + int(10 * u),
                                   r.w - int(10 * u), r.h - int(18 * u)),
                         border_radius=int(5 * u))
        pygame.draw.rect(s, body, head, border_radius=int(4 * u))
        # Schwanz
        pygame.draw.polygon(s, body, [
            (r.x + int(6 * u), r.y + int(14 * u)),
            (r.x - int(8 * u), r.y + int(8 * u)),
            (r.x + int(6 * u), r.y + int(24 * u))])
        # Auge
        eye = (head.right - int(6 * u), head.y + int(6 * u))
        pygame.draw.circle(s, (250, 250, 250), eye, max(2, int(3 * u)))
        pygame.draw.circle(s, (20, 20, 20), eye, max(1, int(1.5 * u)))
        # Beine (laufen nur am Boden animiert)
        if self.on_ground and self.state == RUN:
            f = int(self.run_frame) % 2
        else:
            f = 0
        for i in range(2):
            lx = r.x + int((10 + i * 14) * u)
            ly = r.bottom - int(10 * u)
            dh = int((10 if (i + f) % 2 else 5) * u)
            pygame.draw.rect(s, dark, (lx, ly, int(6 * u), dh))

    def _draw_obstacles(self, s):
        col = self._pal("obj")
        u = self.scale
        for o in self.obstacles:
            x, oy, w, h = int(o["x"]), int(o["oy"]), int(o["w"]), int(o["h"])
            if o["bird"]:
                cy = oy + h // 2
                pygame.draw.ellipse(s, col, (x + int(8 * u), cy - int(5 * u),
                                             int(28 * u), int(12 * u)))
                # Kopf + Schnabel
                pygame.draw.circle(s, col, (x + w - int(4 * u), cy), int(6 * u))
                pygame.draw.polygon(s, col, [
                    (x + w, cy - int(2 * u)), (x + w + int(8 * u), cy),
                    (x + w, cy + int(2 * u))])
                # Fluegel: auf/ab
                up = (o["flap"] % 2) < 1
                if up:
                    pygame.draw.polygon(s, col, [
                        (x + int(10 * u), cy), (x + int(24 * u), cy),
                        (x + int(14 * u), cy - int(16 * u))])
                else:
                    pygame.draw.polygon(s, col, [
                        (x + int(10 * u), cy), (x + int(24 * u), cy),
                        (x + int(14 * u), cy + int(16 * u))])
            else:
                spikes = o.get("spikes", 1)
                cw = w // spikes
                for i in range(spikes):
                    bx = x + i * cw
                    stem = pygame.Rect(bx + cw // 2 - int(3 * u), oy,
                                       int(6 * u), h)
                    pygame.draw.rect(s, col, stem)
                    # Arme
                    ay = oy + h // 3
                    pygame.draw.rect(s, col, (bx + cw // 2 - int(9 * u), ay,
                                              int(6 * u), int(3 * u)))
                    pygame.draw.rect(s, col, (bx + cw // 2 - int(9 * u),
                                              ay - int(8 * u), int(3 * u),
                                              int(10 * u)))
                    pygame.draw.rect(s, col, (bx + cw // 2 + int(3 * u),
                                              ay + int(4 * u), int(6 * u),
                                              int(3 * u)))
                    pygame.draw.rect(s, col, (bx + cw // 2 + int(6 * u),
                                              ay - int(4 * u), int(3 * u),
                                              int(10 * u)))

    def _draw_hud(self, s):
        # Score rechts oben, Chrome-Stil: "HI 00512  00047"
        cur = f"{self.score:05d}"
        hi = f"{self.best:05d}"
        blink_on = self.blink > 0 and int(self.blink * 8) % 2 == 0
        col = self._pal("dim")
        txt = self._mono.render(f"HI {hi}", True, col)
        s.blit(txt, (self.width - txt.get_width() - int(90 * self.scale), 12))
        if not blink_on:
            cimg = self._mono.render(cur, True, self._pal("text"))
            s.blit(cimg, (self.width - cimg.get_width() - 14, 12))

    def _draw_ready(self, s):
        cx = self.width // 2
        title = self._big.render(t("trex.title"), True, self._pal("text"))
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.24))))
        if int(pygame.time.get_ticks() / 500) % 2 == 0:
            ps = self._mid.render(t("trex.press_start"), True, self._pal("text"))
            s.blit(ps, ps.get_rect(center=(cx, int(self.height * 0.36))))
        ctrl = self._small.render(t("trex.controls"), True, self._pal("dim"))
        s.blit(ctrl, ctrl.get_rect(center=(cx, int(self.height * 0.44))))
        # Optionszeile
        onoff = t("common.on") if self.night_on else t("common.off")
        opts = "  ·  ".join([
            f"[1-3] {t('trex.difficulty')}: {t('trex.diff.' + self.diff)}",
            f"[N] {t('trex.night')}: {onoff}",
            f"[C] {t('trex.skin')}: {self.skin + 1}",
        ])
        oimg = self._tiny.render(opts, True, self._pal("dim"))
        s.blit(oimg, oimg.get_rect(center=(cx, int(self.height * 0.52))))

    def _draw_over(self, s):
        cx = self.width // 2
        cy = int(self.height * 0.34)
        go = self._big.render(t("common.game_over"), True, self._pal("text"))
        s.blit(go, go.get_rect(center=(cx, cy)))
        if self.score >= self.best and self.score > 0:
            rec = self._small.render(t("trex.new_record"), True, (240, 190, 90))
            s.blit(rec, rec.get_rect(center=(cx, cy + int(30 * self.scale))))
        if int(self.dead_t * 2) % 2 == 0:
            hint = self._small.render(t("common.enter_restart"), True,
                                      self._pal("dim"))
            s.blit(hint, hint.get_rect(center=(cx, cy + int(58 * self.scale))))
