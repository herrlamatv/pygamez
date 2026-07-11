# -*- coding: utf-8 -*-
"""
aimtrainer.py
=============
Aim Trainer - chilliges 3D-Zielschießen (Einzelspieler).

Zielen (FPS-Look wie Minecraft/Fortnite): Das Fadenkreuz sitzt fest in der
Bildmitte, die Maus steuert die Kamera DIREKT - jede Bewegung dreht den
Blick sofort um den Bewegungsbetrag (Pointer-Capture: der Cursor wird
unsichtbar im Fenster festgehalten, main.py liefert relative Bewegung als
InputEvent.MOUSEREL; Esc/Pause gibt die Maus frei). Yaw ist unbegrenzt
(360 Grad), Pitch klemmt bei +-60 Grad. Linksklick schießt exakt durch
die Bildmitte.

Modi (Auswahl im Vorspiel-Screen):
- precision : 60 s, immer 3 statische Kugeln - Abschuss spawnt sofort neu;
              Genauigkeits-Bonus am Ende.
- reflex    : 30 Ziele einzeln, wachsen und schrumpfen (2.0 -> 1.2 s);
              Reaktionszeit-Statistik (Durchschnitt/Bestwert).
- moving    : 60 s, Ziele schweben auf Bahnen (Strafe/Orbit/Bob);
              Combo-Multiplikator bis x4, Fehlschuss resettet.
- chill     : endlos, ohne Timer und ohne Fehlschlag-Strafe; [E] beendet
              die Sitzung, sonst wird der Score beim Menü-Rückweg gespeichert.

Themes (Setup, gespeichert): space (Sternenkugel + schwarzes Loch mit hellem
Ring im Gargantua-Stil + Planet), neon (Synthwave-Grid + Sonne) und range
(Schießstand-Halle). Nur Optik - das Gameplay ist identisch.

Die 3D-Technik (Kamera-Basis, Projektion, Near-Clip, Painter-Sortierung,
Nebel, Billboards) ist von Snakes 3D-Modus portiert - reine Software, ohne
OpenGL und ohne Assets.
"""

import math
import random
import time

import pygame

import settings as settings_mod
from game_base import Game, InputEvent
from i18n import t

# ----- Kamera / Projektion ---------------------------------------------------
CAM_POS = (0.0, 1.6, 0.0)
NEAR = 0.12
FOV_MUL = 1.04                 # _f = height * FOV_MUL  (~65 Grad horizontal)

DEG_PER_PX = 0.12              # Grad Drehung je Maus-Pixel bei sens = 1.0
PITCH_CLAMP = math.radians(60.0)

COOLDOWN = 0.18                # s zwischen zwei Schüssen
FORGIVE_REL = 1.15             # Treffer-Vergebung (relativ + absolut)
FORGIVE_ABS = 0.004

SETUP, PLAY = "setup", "play"

# ----- Modi ------------------------------------------------------------------
MODE_CFG = {
    "precision": dict(duration=60.0, targets=None, simul=3, radius=0.45,
                      paths=False, speed=1.0, yaw=70, pmin=-10, pmax=35,
                      dmin=8, dmax=20, respawn=(0.0, 0.0), relative=False),
    "reflex": dict(duration=None, targets=30, simul=1, radius=0.50,
                   paths=False, speed=1.0, yaw=60, pmin=-15, pmax=30,
                   dmin=6, dmax=14, respawn=(0.4, 0.9), relative=True),
    "moving": dict(duration=60.0, targets=None, simul=2, radius=0.50,
                   paths=True, speed=1.0, yaw=70, pmin=-5, pmax=30,
                   dmin=7, dmax=16, respawn=(0.3, 0.3), relative=False),
    "chill": dict(duration=None, targets=None, simul=3, radius=0.55,
                  paths=True, speed=0.6, yaw=90, pmin=-10, pmax=35,
                  dmin=6, dmax=18, respawn=(0.3, 0.3), relative=False),
}

# ----- Themes ----------------------------------------------------------------
THEMES = {
    "space": dict(
        sky=((4, 5, 12), (10, 8, 24)),
        fog=(10, 10, 22), fog_start=26.0, fog_end=60.0,
        body=(120, 200, 255), ring=(225, 240, 255), dot=(255, 255, 255),
        glow=(90, 160, 255), accent=(240, 245, 255),
    ),
    "neon": dict(
        sky=((18, 8, 31), (58, 22, 80)),
        fog=(58, 22, 80), fog_start=10.0, fog_end=30.0,
        body=(0, 240, 200), ring=(255, 255, 255), dot=(20, 20, 40),
        glow=(0, 255, 220), accent=(255, 230, 120),
    ),
    "range": dict(
        sky=((30, 32, 36), (30, 32, 36)),
        fog=(30, 32, 36), fog_start=14.0, fog_end=40.0,
        body=(230, 70, 70), ring=(240, 240, 240), dot=(230, 70, 70),
        glow=None, accent=(250, 250, 250),
    ),
}
THEME_KEYS = ["space", "neon", "range"]


def _ease_out_back(x):
    """Kleiner Überschwinger am Ende (wie snake.py) - für den Target-Spawn."""
    c1, c3 = 1.70158, 2.70158
    x = max(0.0, min(1.0, x))
    return 1 + c3 * (x - 1) ** 3 + c1 * (x - 1) ** 2


def _dir_from(yaw, pitch):
    cp = math.cos(pitch)
    return (math.sin(yaw) * cp, math.sin(pitch), math.cos(yaw) * cp)


class AimTrainerGame(Game):
    name = "Aim Trainer"
    highscore_key = "aim"
    supports_multiplayer = False

    MODES = [("precision", "aim.mode.precision"), ("reflex", "aim.mode.reflex"),
             ("moving", "aim.mode.moving"), ("chill", "aim.mode.chill")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        self.cfg = MODE_CFG.get(self.mode, MODE_CFG["precision"])
        theme, sens, blur = self._aim_settings()
        self.theme_key = theme
        self.sens = sens
        self.blur = blur             # Motion-Blur-Stärke (0.0 = aus .. 0.8)
        self._prev_frame = None      # letztes Bild für die Blur-Mischung

        self._small = pygame.font.SysFont("consolas", 16)
        self._tiny = pygame.font.SysFont("consolas", 13)
        self._big = pygame.font.SysFont("consolas", 22, bold=True)
        self._huge = pygame.font.SysFont("consolas", max(26, self.height // 11),
                                         bold=True)

        self.yaw = 0.0
        self.pitch = 0.0
        self._kick = 0.0
        self.anim_t = 0.0
        self._go_last = None
        self.capture_mouse = False   # main.py: Pointer-Capture + MOUSEREL

        # Render-Caches (auflösungs-/themenabhängig)
        self._sky_cache = None
        self._bh_cache = None
        self._sun_cache = None
        self._stars = self._make_stars(220)

        self._build_setup_layout()
        self.state = SETUP

    def on_surface_changed(self):
        self._huge = pygame.font.SysFont("consolas", max(26, self.height // 11),
                                         bold=True)
        self._sky_cache = None
        self._bh_cache = None
        self._sun_cache = None
        self._prev_frame = None
        self._build_setup_layout()

    def _aim_settings(self):
        aim = self.settings.get("aim", {}) if isinstance(self.settings, dict) else {}
        theme = aim.get("theme", "space")
        if theme not in THEMES:
            theme = "space"
        try:
            sens = max(0.5, min(2.0, float(aim.get("sens", 1.0))))
        except (TypeError, ValueError):
            sens = 1.0
        try:
            blur = max(0.0, min(0.8, float(aim.get("blur", 0.0))))
        except (TypeError, ValueError):
            blur = 0.0
        return theme, sens, blur

    def _save_aim(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("aim", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _theme(self):
        return THEMES[self.theme_key]

    @staticmethod
    def _make_stars(n):
        rnd = random.Random(4021)
        stars = []
        for _ in range(n):
            z = rnd.uniform(-1.0, 1.0)
            az = rnd.uniform(0, math.tau)
            xy = math.sqrt(max(0.0, 1.0 - z * z))
            stars.append(((math.cos(az) * xy, z, math.sin(az) * xy),
                          rnd.choice((1, 1, 2)), rnd.uniform(0, math.tau)))
        return stars

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(120, (self.width - 80) // 3 - 10)
        total = 3 * bw + 2 * 12
        y0 = int(self.height * 0.28)
        self.theme_rects = [pygame.Rect(cx - total // 2 + i * (bw + 12), y0,
                                        bw, 44) for i in range(3)]
        # Regler-Reihen: Beschriftung steht LINKS daneben (spart Höhe).
        rx = cx + 24
        sy = y0 + 58
        self.sens_minus = pygame.Rect(rx - 60, sy, 44, 40)
        self.sens_plus = pygame.Rect(rx + 116, sy, 44, 40)
        self.sens_box = pygame.Rect(rx - 8, sy, 116, 40)
        by = sy + 52
        self.blur_minus = pygame.Rect(rx - 60, by, 44, 40)
        self.blur_plus = pygame.Rect(rx + 116, by, 44, 40)
        self.blur_box = pygame.Rect(rx - 8, by, 116, 40)
        self.start_rect = pygame.Rect(cx - 95, by + 54, 190, 46)

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3"):
                self._set_theme(int(k) - 1)
            elif k in ("Left", "a", "A"):
                self._set_theme((THEME_KEYS.index(self.theme_key) - 1) % 3)
            elif k in ("Right", "d", "D"):
                self._set_theme((THEME_KEYS.index(self.theme_key) + 1) % 3)
            elif k in ("plus", "equal", "KP_Add"):
                self._change_sens(+0.1)
            elif k in ("minus", "KP_Subtract"):
                self._change_sens(-0.1)
            elif k in ("b", "B"):
                self._cycle_blur()
            elif k in ("Return", "space"):
                self._start_run()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, r in enumerate(self.theme_rects):
                if r.collidepoint(event.pos):
                    self._set_theme(i)
                    return
            if self.sens_minus.collidepoint(event.pos):
                self._change_sens(-0.1)
            elif self.sens_plus.collidepoint(event.pos):
                self._change_sens(+0.1)
            elif self.blur_minus.collidepoint(event.pos):
                self._change_blur(-0.1)
            elif self.blur_plus.collidepoint(event.pos):
                self._change_blur(+0.1)
            elif self.start_rect.collidepoint(event.pos):
                self._start_run()

    def _set_theme(self, idx):
        self.theme_key = THEME_KEYS[idx % 3]
        self._save_aim("theme", self.theme_key)
        self._sky_cache = None
        self.play_sound("click")

    def _change_sens(self, delta):
        self.sens = round(max(0.5, min(2.0, self.sens + delta)), 1)
        self._save_aim("sens", self.sens)
        self._toast = t("aim.sens_toast", v=f"{self.sens:.1f}")
        self._toast_t = 1.0
        self.play_sound("select")

    def _change_blur(self, delta):
        self.blur = round(max(0.0, min(0.8, self.blur + delta)), 1)
        self._save_aim("blur", self.blur)
        if self.blur <= 0:
            self._prev_frame = None
        self.play_sound("select")

    def _cycle_blur(self):
        """[B] im Setup: Blur in 0.1er-Schritten durchschalten (0.8 -> aus)."""
        self.blur = 0.0 if self.blur >= 0.79 else round(self.blur + 0.1, 1)
        self._save_aim("blur", self.blur)
        if self.blur <= 0:
            self._prev_frame = None
        self.play_sound("select")

    def _blur_label(self):
        return t("common.off") if self.blur <= 0 else f"{int(self.blur * 100)}%"

    def _apply_blur(self, s):
        """Motion Blur: das vorige Bild mit Blur-abhängiger Deckkraft über das
        neue mischen (exponentieller Trail). Läuft VOR Fadenkreuz/HUD, damit
        die scharf bleiben."""
        if self.blur <= 0.01:
            self._prev_frame = None
            return
        if self._prev_frame is None \
                or self._prev_frame.get_size() != s.get_size():
            self._prev_frame = s.copy()
            return
        self._prev_frame.set_alpha(int(self.blur * 230))
        s.blit(self._prev_frame, (0, 0))
        self._prev_frame = s.copy()

    # ===================================================== Lauf starten/beenden
    def _start_run(self):
        self.score = 0
        self.game_over = False
        self.play_t = 0.0
        self.time_left = self.cfg["duration"]
        self.targets = []
        self._pending = []          # Respawn-Timer
        self.spawned = 0            # (reflex) bisher erschienene Ziele
        self.hits = 0
        self.shots = 0
        self.misses = 0
        self.combo = 0
        self.max_combo = 0
        self.last_hit_t = 0.0
        self.reactions = []
        self.last_reaction = None
        self.last_reaction_t = -9.0
        self.cooldown = 0.0
        self._flash = 0.0
        self._tracer = 0.0
        self._hitmark = 0.0
        self._toast = None
        self._toast_t = 0.0
        self.particles = []         # [pos, vel, age, life, col]
        self.popups = []            # [text, world_pos, age]
        self._go_last = None
        for _ in range(self.cfg["simul"]):
            self._spawn_target()
        self.state = PLAY
        self.capture_mouse = True
        self.play_sound("level")

    def _finish_run(self):
        if self.mode == "precision" and self.shots > 0:
            bonus = int((self.hits / self.shots) ** 2 * 1500)
            self.acc_bonus = bonus
            self.score += bonus
            if bonus:
                self.play_sound("point")
        else:
            self.acc_bonus = 0
        self.game_over = True
        self.capture_mouse = False
        self.play_sound("win")

    # ===================================================== Targets
    def _spawn_target(self):
        cfg = self.cfg
        if cfg["targets"] is not None and self.spawned >= cfg["targets"]:
            return
        base_yaw = self.yaw if cfg["relative"] else 0.0
        pos = None
        for _ in range(20):
            yw = base_yaw + math.radians(random.uniform(-cfg["yaw"], cfg["yaw"]))
            pt = math.radians(random.uniform(cfg["pmin"], cfg["pmax"]))
            dist = random.uniform(cfg["dmin"], cfg["dmax"])
            d = _dir_from(yw, pt)
            pos = (CAM_POS[0] + d[0] * dist, CAM_POS[1] + d[1] * dist,
                   CAM_POS[2] + d[2] * dist)
            if all(self._angle_between(pos, o) > math.radians(15)
                   for o in self.targets):
                break
        path = None
        if cfg["paths"]:
            sp = cfg["speed"]
            kind = random.choices(("strafe", "orbit", "bob"),
                                  (0.45, 0.30, 0.25))[0]
            if kind == "strafe":
                amp = random.uniform(2.5, 5.0)
                path = dict(kind=kind, amp=amp,
                            om=random.uniform(1.5, 3.5) / amp * sp,
                            ax=(math.cos(yw), 0.0, -math.sin(yw)))
            elif kind == "orbit":
                path = dict(kind=kind, rad=random.uniform(1.5, 3.0),
                            om=random.uniform(0.5, 1.2) * sp)
            else:
                path = dict(kind=kind, amp=random.uniform(0.8, 1.6),
                            om=random.uniform(1.0, 2.0) * sp,
                            drift=random.uniform(-0.4, 0.4) * sp)
        life = None
        if self.mode == "reflex":
            i = min(self.spawned, 29)
            life = 2.0 - 0.8 * i / 29.0
        self.targets.append(dict(base=pos, radius=cfg["radius"], state="grow",
                                 age=0.0, life=life, path=path,
                                 phase=random.uniform(0, math.tau),
                                 spawn_t=self.play_t))
        self.spawned += 1

    def _angle_between(self, pos_a, target_b):
        pb = self._target_pos(target_b)
        va = self._norm_dir(pos_a)
        vb = self._norm_dir(pb)
        d = max(-1.0, min(1.0, va[0] * vb[0] + va[1] * vb[1] + va[2] * vb[2]))
        return math.acos(d)

    @staticmethod
    def _norm_dir(p):
        d = (p[0] - CAM_POS[0], p[1] - CAM_POS[1], p[2] - CAM_POS[2])
        n = math.sqrt(d[0] ** 2 + d[1] ** 2 + d[2] ** 2) or 1.0
        return (d[0] / n, d[1] / n, d[2] / n)

    def _target_pos(self, tg):
        b = tg["base"]
        p = tg.get("path")
        if not p:
            return b
        tt = self.play_t + tg["phase"]
        if p["kind"] == "strafe":
            off = p["amp"] * math.sin(p["om"] * tt)
            ax = p["ax"]
            return (b[0] + ax[0] * off, b[1], b[2] + ax[2] * off)
        if p["kind"] == "orbit":
            return (b[0] + p["rad"] * math.cos(p["om"] * tt), b[1],
                    b[2] + p["rad"] * math.sin(p["om"] * tt))
        # bob: vertikale Sinuswelle + begrenzte horizontale Drift
        dx = max(-2.5, min(2.5, p["drift"] * (tt % 12.0 - 6.0)))
        return (b[0] + dx, b[1] + p["amp"] * math.sin(p["om"] * tt), b[2])

    def _target_scale(self, tg):
        if tg["state"] == "grow":
            return _ease_out_back(tg["age"] / 0.25)
        if tg["state"] == "shrink":
            return max(0.0, 1.0 - tg["age"] / 0.18)
        if tg["life"] is not None:                # reflex: Ende-Schrumpfen
            frac = tg["age"] / tg["life"]
            if frac > 0.8:
                return max(0.3, 1.0 - (frac - 0.8) / 0.2 * 0.7)
        return 1.0

    def _update_targets(self, dt):
        cfg = self.cfg
        alive = []
        for tg in self.targets:
            tg["age"] += dt
            if tg["state"] == "grow" and tg["age"] >= 0.25:
                tg["state"] = "alive"
                tg["age"] = 0.25
            if tg["state"] == "shrink":
                if tg["age"] >= 0.18:
                    continue
            elif tg["life"] is not None and tg["age"] >= tg["life"]:
                # Reflex: abgelaufen = Fehlschuss
                self.misses += 1
                self.play_sound("move")
                self._pending.append(random.uniform(*cfg["respawn"]))
                continue
            alive.append(tg)
        self.targets = alive

        rest = []
        for timer in self._pending:
            timer -= dt
            if timer <= 0:
                self._spawn_target()
            else:
                rest.append(timer)
        self._pending = rest

        # Lauf-Ende (reflex): alle Ziele verbraucht und keins mehr aktiv
        if self.mode == "reflex" and self.spawned >= cfg["targets"] \
                and not self.targets and not self._pending \
                and not self.game_over:
            self._finish_run()

    # ===================================================== Kamera
    def _apply_look(self, rel):
        """Direkte 1:1-Maussteuerung (FPS-Look): Delta-Pixel -> Drehung."""
        k = math.radians(DEG_PER_PX) * self.sens
        self.yaw = (self.yaw + rel[0] * k) % math.tau
        self.pitch = max(-PITCH_CLAMP,
                         min(PITCH_CLAMP, self.pitch - rel[1] * k))

    def _forward(self, extra_pitch=0.0):
        return _dir_from(self.yaw, self.pitch + extra_pitch)

    def _view_basis(self):
        f = self._forward(math.radians(0.6) * self._kick)
        rl = math.hypot(f[0], f[2]) or 1.0
        r = (f[2] / rl, 0.0, -f[0] / rl)      # echter Rechts-Vektor
        u = (f[1] * r[2] - f[2] * r[1],        # up = f x r (zeigt nach oben)
             f[2] * r[0] - f[0] * r[2],
             f[0] * r[1] - f[1] * r[0])
        return (r, u, f)

    def _to_cam(self, p):
        r, u, f = self._basis
        dx = p[0] - CAM_POS[0]
        dy = p[1] - CAM_POS[1]
        dz = p[2] - CAM_POS[2]
        return (dx * r[0] + dy * r[1] + dz * r[2],
                dx * u[0] + dy * u[1] + dz * u[2],
                dx * f[0] + dy * f[1] + dz * f[2])

    def _proj(self, c):
        k = self._f / c[2]
        return (self._scx + c[0] * k, self._scy - c[1] * k)

    @staticmethod
    def _clip_near(pts):
        out = []
        n = len(pts)
        for i in range(n):
            a, b = pts[i], pts[(i + 1) % n]
            da, db = a[2] - NEAR, b[2] - NEAR
            if da >= 0:
                out.append(a)
            if (da >= 0) != (db >= 0):
                tt = da / (da - db)
                out.append((a[0] + (b[0] - a[0]) * tt,
                            a[1] + (b[1] - a[1]) * tt, NEAR))
        return out

    @staticmethod
    def _clip_seg(a, b):
        da, db = a[2] - NEAR, b[2] - NEAR
        if da < 0 and db < 0:
            return None
        if da < 0 or db < 0:
            tt = da / (da - db)
            m = (a[0] + (b[0] - a[0]) * tt, a[1] + (b[1] - a[1]) * tt, NEAR)
            return (m, b) if da < 0 else (a, m)
        return (a, b)

    def _fog_color(self, col, depth):
        th = self._theme()
        if depth <= th["fog_start"]:
            return col
        f = min(1.0, (depth - th["fog_start"]) / (th["fog_end"] - th["fog_start"]))
        fog = th["fog"]
        return (int(col[0] + (fog[0] - col[0]) * f),
                int(col[1] + (fog[1] - col[1]) * f),
                int(col[2] + (fog[2] - col[2]) * f))

    def _bill(self, p):
        """Billboard-Projektion: (sx, sy, k, z) oder None hinter der Kamera."""
        c = self._to_cam(p)
        if c[2] < NEAR:
            return None
        k = self._f / c[2]
        return (self._scx + c[0] * k, self._scy - c[1] * k, k, c[2])

    # ===================================================== Schießen
    def _shoot(self):
        if self.cooldown > 0 or self.game_over:
            return
        self.cooldown = COOLDOWN
        self.shots += 1
        self._flash = 0.06
        self._tracer = 0.08
        self._kick = 1.0
        self.play_sound("shoot")

        f = self._forward()          # echter Ray ohne Recoil-Kick
        best = None
        for tg in self.targets:
            if tg["state"] == "shrink":
                continue
            if tg["state"] == "grow" and self._target_scale(tg) < 0.5:
                continue
            pos = self._target_pos(tg)
            d = self._norm_dir(pos)
            dist = math.dist(pos, CAM_POS)
            ang = math.acos(max(-1.0, min(1.0,
                                          f[0] * d[0] + f[1] * d[1] + f[2] * d[2])))
            ar = math.atan((tg["radius"] * self._target_scale(tg)) / dist)
            if ang <= ar * FORGIVE_REL + FORGIVE_ABS:
                if best is None or ang < best[0]:
                    best = (ang, tg, pos, dist)
        if best is not None:
            self._register_hit(best[1], best[2], best[3])
        else:
            self._register_miss()

    def _register_hit(self, tg, pos, dist):
        self.hits += 1
        self.combo += 1
        self.max_combo = max(self.max_combo, self.combo)
        self.last_hit_t = self.play_t
        self._hitmark = 0.1
        mult = 1.0 + 0.25 * min(self.combo, 12)
        if self.mode == "precision":
            pts = 100 + int((dist - 8) * 6)
        elif self.mode == "reflex":
            reaction = int((self.play_t - tg["spawn_t"]) * 1000)
            self.reactions.append(reaction)
            self.last_reaction = reaction
            self.last_reaction_t = self.play_t
            pts = max(50, 1000 - reaction)
        elif self.mode == "moving":
            pts = int(80 * mult)
            if self.combo in (5, 10):
                self.play_sound("point")
            elif self.combo == 12:
                self.play_sound("level")
        else:
            pts = int(50 * mult)
        self.score += pts
        self.play_sound("hit")
        self.rumble(60)

        tg["state"] = "shrink"
        tg["age"] = 0.0
        self._pending.append(random.uniform(*self.cfg["respawn"]))
        # Treffer-Partikel + Punkte-Popup
        th = self._theme()
        for _ in range(10):
            a = random.uniform(0, math.tau)
            b = random.uniform(-1, 1)
            sp = random.uniform(2.0, 5.0)
            v = (math.cos(a) * sp, b * sp, math.sin(a) * sp)
            self.particles.append([list(pos), v, 0.0,
                                   random.uniform(0.3, 0.55), th["body"]])
        self.popups.append([f"+{pts}", pos, 0.0])

    def _register_miss(self):
        self.misses += 1
        if self.mode == "moving":
            self.combo = 0

    # ===================================================== Eingabe / Update
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.game_over:
            if event.kind == InputEvent.KEYDOWN and \
                    event.key in ("Return", "space"):
                self.reset()
            return
        if event.kind == InputEvent.MOUSEREL:
            self._apply_look(event.rel)
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            self._shoot()
        elif event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("plus", "equal", "KP_Add"):
                self._change_sens(+0.1)
            elif k in ("minus", "KP_Subtract"):
                self._change_sens(-0.1)
            elif k in ("e", "E") and self.mode == "chill":
                self._finish_run()

    def update(self, dt):
        self.anim_t += dt
        if self.state != PLAY or self.game_over:
            return
        self.play_t += dt
        self.cooldown = max(0.0, self.cooldown - dt)
        self._flash = max(0.0, self._flash - dt)
        self._tracer = max(0.0, self._tracer - dt)
        self._hitmark = max(0.0, self._hitmark - dt)
        self._kick -= self._kick * min(1.0, dt * 9.0)
        if self._toast_t > 0:
            self._toast_t -= dt

        self._update_targets(dt)

        # Chill: Combo verfällt nach 5 s ohne Treffer
        if self.mode == "chill" and self.combo > 0 \
                and self.play_t - self.last_hit_t > 5.0:
            self.combo = 0

        # Partikel / Popups
        alive = []
        for p in self.particles:
            p[2] += dt
            if p[2] < p[3]:
                p[0][0] += p[1][0] * dt
                p[0][1] += p[1][1] * dt
                p[0][2] += p[1][2] * dt
                alive.append(p)
        self.particles = alive
        self.popups = [[txt, pos, age + dt] for txt, pos, age in self.popups
                       if age + dt < 0.9]

        if self.time_left is not None:
            self.time_left -= dt
            if self.time_left <= 0:
                self.time_left = 0.0
                self._finish_run()

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        self._scx, self._scy = self.width / 2, self.height / 2
        self._f = self.height * FOV_MUL

        if self.state == SETUP:
            self._draw_setup(s)
            return

        if self.game_over:
            # update() steht still -> Hintergrund über Echtzeit weiterdrehen
            now = time.monotonic()
            if self._go_last is not None:
                gdt = min(0.05, now - self._go_last)
                self.anim_t += gdt
                self.yaw = (self.yaw + math.radians(8) * gdt) % math.tau
            self._go_last = now

        self._basis = self._view_basis()
        self._draw_background(s)
        self._draw_targets(s)
        self._draw_particles(s)
        self._draw_popups(s)
        self._draw_tracer_flash(s)
        self._apply_blur(s)          # Motion Blur nur auf die Szene
        self._draw_crosshair(s)
        self._draw_hud(s)
        if self.game_over:
            self._draw_result(s)

    # ----- Hintergrund / Themes ------------------------------------------------

    def _sky(self):
        key = (self.width, self.height, self.theme_key)
        if self._sky_cache and self._sky_cache[0] == key:
            return self._sky_cache[1]
        top, bottom = self._theme()["sky"]
        surf = pygame.Surface((self.width, self.height))
        for y in range(self.height):
            f = y / max(1, self.height - 1)
            surf.fill((int(top[0] + (bottom[0] - top[0]) * f),
                       int(top[1] + (bottom[1] - top[1]) * f),
                       int(top[2] + (bottom[2] - top[2]) * f)),
                      (0, y, self.width, 1))
        self._sky_cache = (key, surf)
        return surf

    def _draw_background(self, s):
        s.blit(self._sky(), (0, 0))
        if self.theme_key == "space":
            self._draw_stars(s, 220, (200, 210, 235))
            self._draw_planet(s)
            self._draw_blackhole(s)
        elif self.theme_key == "neon":
            self._draw_stars(s, 120, (255, 200, 240))
            self._draw_neon_sun(s)
            self._draw_neon_grid(s)
        else:
            self._draw_range_hall(s)

    def _draw_stars(self, s, count, tint):
        r, u, f = self._basis
        w, h = self.width, self.height
        for d, size, ph in self._stars[:count]:
            cz = d[0] * f[0] + d[1] * f[1] + d[2] * f[2]
            if cz < 0.05:
                continue
            cx = d[0] * r[0] + d[1] * r[1] + d[2] * r[2]
            cy = d[0] * u[0] + d[1] * u[1] + d[2] * u[2]
            k = self._f / cz
            sx = self._scx + cx * k
            sy = self._scy - cy * k
            if 0 <= sx < w and 0 <= sy < h:
                b = 0.55 + 0.45 * math.sin(self.anim_t * 1.7 + ph)
                c = (int(tint[0] * b * 0.6 + 60), int(tint[1] * b * 0.6 + 60),
                     int(tint[2] * b * 0.6 + 60))
                s.fill(c, (int(sx), int(sy), size, size))

    def _blackhole_sprite(self):
        R = max(24, int(self._f * 3.2 / 45.0))
        if self._bh_cache and self._bh_cache[0] == R:
            return self._bh_cache[1], self._bh_cache[2]
        w, h = int(R * 6.8), int(R * 3.4)
        cx, cy = w // 2, h // 2
        surf = pygame.Surface((w, h), pygame.SRCALPHA)
        # dezenter warmer Glow um den Kern
        for rad, alpha in ((int(R * 1.6), 22), (int(R * 1.3), 36)):
            pygame.draw.circle(surf, (255, 240, 220, alpha), (cx, cy), rad)
        # horizontale Akkretions-Scheibe (der "Saturn-Ring")
        for grow, alpha in ((1.15, 60), (1.0, 190)):
            band = pygame.Rect(0, 0, int(R * 3.4 * 2 * grow),
                               int(R * 0.44 * 2 * grow))
            band.center = (cx, cy)
            pygame.draw.ellipse(surf, (255, 236, 210, alpha), band)
        # Loch in der Scheibe, damit der Ring als Ring lesbar bleibt
        inner = pygame.Rect(0, 0, int(R * 1.7 * 2), int(R * 0.30 * 2))
        inner.center = (cx, cy)
        pygame.draw.ellipse(surf, (0, 0, 0, 0), inner)
        # schwarzer Kern (Ereignishorizont)
        pygame.draw.circle(surf, (2, 2, 4, 255), (cx, cy), R)
        # heller Linsen-Ring - leicht gekippt, VOR dem Kern (Gargantua-Look)
        ring = pygame.Surface((int(R * 6.4), int(R * 6.4)), pygame.SRCALPHA)
        rr = pygame.Rect(0, 0, int(R * 3.0 * 2), int(R * 0.9 * 2))
        rr.center = (ring.get_width() // 2, ring.get_height() // 2)
        pygame.draw.ellipse(ring, (255, 244, 224, 255), rr, max(2, R // 22))
        ring = pygame.transform.rotozoom(ring, 18, 1.0)
        # Schimmer-Sprite: NUR der Ring + Photonenring (fuer additives Pulsen)
        shimmer = pygame.Surface((w, h), pygame.SRCALPHA)
        shimmer.blit(ring, ring.get_rect(center=(cx, cy)))
        pygame.draw.circle(shimmer, (255, 244, 224, 220), (cx, cy),
                           int(R * 1.06), max(1, R // 30))
        surf.blit(shimmer, (0, 0))
        self._bh_cache = (R, surf, shimmer)
        return surf, shimmer

    def _draw_blackhole(self, s):
        d = _dir_from(math.radians(40), math.radians(8))
        pos = (CAM_POS[0] + d[0] * 45, CAM_POS[1] + d[1] * 45,
               CAM_POS[2] + d[2] * 45)
        b = self._bill(pos)
        if b is None:
            return
        sprite, shimmer = self._blackhole_sprite()
        rect = sprite.get_rect(center=(int(b[0]), int(b[1])))
        if rect.right < 0 or rect.left > self.width or \
                rect.bottom < 0 or rect.top > self.height:
            return
        s.blit(sprite, rect)
        # Nur der Ring schimmert (additiv, pulsierend)
        shimmer.set_alpha(int(35 + 30 * math.sin(self.anim_t * 0.8)))
        s.blit(shimmer, rect, special_flags=pygame.BLEND_RGBA_ADD)
        shimmer.set_alpha(255)

    def _draw_planet(self, s):
        d = _dir_from(math.radians(-70), math.radians(4))
        pos = (CAM_POS[0] + d[0] * 40, CAM_POS[1] + d[1] * 40,
               CAM_POS[2] + d[2] * 40)
        b = self._bill(pos)
        if b is None:
            return
        r = max(8, int(self._f * 0.6 / 40.0))
        x, y = int(b[0]), int(b[1])
        pygame.draw.circle(s, (30, 26, 40), (x, y), r)
        pygame.draw.circle(s, (180, 150, 220), (x - r // 4, y - r // 4), r, 2)
        pygame.draw.circle(s, (52, 44, 66), (x + r // 5, y + r // 5),
                           int(r * 0.82))

    def _draw_neon_sun(self, s):
        d = _dir_from(0.0, math.radians(10))
        pos = (CAM_POS[0] + d[0] * 50, CAM_POS[1] + d[1] * 50,
               CAM_POS[2] + d[2] * 50)
        b = self._bill(pos)
        if b is None:
            return
        r = max(30, int(self._f * 7.0 / 50.0))
        if self._sun_cache is None or self._sun_cache[0] != r:
            surf = pygame.Surface((2 * r, 2 * r), pygame.SRCALPHA)
            for yy in range(2 * r):
                f = yy / (2 * r - 1)
                col = (int(255), int(120 + 70 * f), int(180 - 90 * f), 255)
                pygame.draw.line(surf, col, (0, yy), (2 * r, yy))
            mask = pygame.Surface((2 * r, 2 * r), pygame.SRCALPHA)
            pygame.draw.circle(mask, (255, 255, 255, 255), (r, r), r)
            surf.blit(mask, (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
            # klassische Streifen in der unteren Hälfte
            for i in range(4):
                yy = r + int(r * (0.15 + i * 0.22))
                hh = max(2, int(r * 0.06 + i * 2))
                pygame.draw.rect(surf, (0, 0, 0, 0), (0, yy, 2 * r, hh))
            self._sun_cache = (r, surf)
        surf = self._sun_cache[1]
        s.blit(surf, surf.get_rect(center=(int(b[0]), int(b[1]))))

    def _draw_neon_grid(self, s):
        col_main = (255, 60, 200)
        col_hi = (255, 120, 220)
        for i, gz in enumerate(range(-40, 42, 2)):
            seg = self._clip_seg(self._to_cam((-40.0, 0.0, float(gz))),
                                 self._to_cam((40.0, 0.0, float(gz))))
            if seg:
                self._grid_line(s, seg, col_hi if i % 5 == 0 else col_main)
        for i, gx in enumerate(range(-40, 42, 2)):
            seg = self._clip_seg(self._to_cam((float(gx), 0.0, -40.0)),
                                 self._to_cam((float(gx), 0.0, 40.0)))
            if seg:
                self._grid_line(s, seg, col_hi if i % 5 == 0 else col_main)

    def _grid_line(self, s, seg, col):
        a, b = seg
        depth = (a[2] + b[2]) / 2
        if depth > self._theme()["fog_end"]:
            return
        pa, pb = self._proj(a), self._proj(b)
        w = self.width
        if (pa[0] < -w and pb[0] < -w) or (pa[0] > 2 * w and pb[0] > 2 * w):
            return
        pygame.draw.line(s, self._fog_color(col, depth),
                         (int(pa[0]), int(pa[1])), (int(pb[0]), int(pb[1])))

    def _draw_range_hall(self, s):
        items = []
        L, HT = 22.0, 6.0

        def add(pts, col):
            cam = [self._to_cam(p) for p in pts]
            if all(c[2] <= NEAR for c in cam):
                return
            cam = self._clip_near(cam)
            if len(cam) < 3:
                return
            depth = sum(c[2] for c in cam) / len(cam)
            proj = [self._proj(c) for c in cam]
            items.append((depth, proj, self._fog_color(col, depth)))

        # Boden (8 Quads) + Decke (4)
        for i in range(4):
            x0 = -L + i * (L / 2)
            add([(x0, 0, -L), (x0 + L / 2, 0, -L), (x0 + L / 2, 0, 0),
                 (x0, 0, 0)], (40, 42, 46))
            add([(x0, 0, 0), (x0 + L / 2, 0, 0), (x0 + L / 2, 0, L),
                 (x0, 0, L)], (43, 45, 49))
            add([(x0, HT, -L), (x0 + L / 2, HT, -L), (x0 + L / 2, HT, L),
                 (x0, HT, L)], (34, 36, 42))
        # Wände (je 4 Streifen)
        for i in range(4):
            z0 = -L + i * (L / 2)
            shade = 0.9 if i % 2 == 0 else 0.75
            col = tuple(int(c * shade) for c in (52, 54, 60))
            add([(-L, 0, z0), (-L, 0, z0 + L / 2), (-L, HT, z0 + L / 2),
                 (-L, HT, z0)], col)
            add([(L, 0, z0), (L, 0, z0 + L / 2), (L, HT, z0 + L / 2),
                 (L, HT, z0)], col)
            x0 = -L + i * (L / 2)
            add([(x0, 0, -L), (x0 + L / 2, 0, -L), (x0 + L / 2, HT, -L),
                 (x0, HT, -L)], col)
            add([(x0, 0, L), (x0 + L / 2, 0, L), (x0 + L / 2, HT, L),
                 (x0, HT, L)], col)
        items.sort(key=lambda it: -it[0])
        for _d, pts, col in items:
            pygame.draw.polygon(s, col, [(int(x), int(y)) for x, y in pts])
        # Bahnlinien auf dem Boden
        for gx in range(-10, 11, 5):
            seg = self._clip_seg(self._to_cam((float(gx) / 2, 0.01, -L)),
                                 self._to_cam((float(gx) / 2, 0.01, L)))
            if seg:
                self._grid_line(s, seg, (66, 70, 78))
        # Lampen
        for lx in (-8.0, 0.0, 8.0):
            for lz in (-16.0, -6.0, 6.0, 16.0):
                b = self._bill((lx, HT - 0.1, lz))
                if b is None:
                    continue
                wpx = max(2, int(b[2] * 0.5))
                pygame.draw.rect(s, (235, 235, 215),
                                 (int(b[0]) - wpx, int(b[1]) - wpx // 3,
                                  2 * wpx, max(2, wpx // 2)))

    # ----- Targets & Effekte -----------------------------------------------------

    def _draw_targets(self, s):
        th = self._theme()
        drawlist = []
        for tg in self.targets:
            pos = self._target_pos(tg)
            b = self._bill(pos)
            if b is None:
                continue
            scale = self._target_scale(tg)
            pulse = 1.0 + 0.04 * math.sin(self.anim_t * 2 + tg["phase"])
            rpx = b[2] * tg["radius"] * scale * pulse
            if rpx < 1:
                continue
            drawlist.append((b[3], b[0], b[1], rpx))
        drawlist.sort(key=lambda d: -d[0])
        for depth, sx, sy, rpx in drawlist:
            x, y, r = int(sx), int(sy), int(rpx)
            if x < -r or x > self.width + r or y < -r or y > self.height + r:
                continue
            if th["glow"]:
                glow = pygame.Surface((3 * r + 2, 3 * r + 2), pygame.SRCALPHA)
                pygame.draw.circle(glow, (*th["glow"], 60),
                                   (glow.get_width() // 2,
                                    glow.get_height() // 2), int(r * 1.5))
                s.blit(glow, glow.get_rect(center=(x, y)),
                       special_flags=pygame.BLEND_RGBA_ADD)
            body = self._fog_color(th["body"], depth)
            ring = self._fog_color(th["ring"], depth)
            dot = self._fog_color(th["dot"], depth)
            pygame.draw.circle(s, body, (x, y), r)
            if r >= 5:
                pygame.draw.circle(s, ring, (x, y), int(r * 0.65),
                                   max(2, r // 5))
            pygame.draw.circle(s, dot, (x, y), max(1, int(r * 0.28)))

    def _draw_particles(self, s):
        for pos, _v, age, life, col in self.particles:
            b = self._bill(tuple(pos))
            if b is None:
                continue
            f = 1.0 - age / life
            r = max(1, int(b[2] * 0.06 * f))
            pygame.draw.circle(s, self._fog_color(col, b[3]),
                               (int(b[0]), int(b[1])), r)

    def _draw_popups(self, s):
        for txt, pos, age in self.popups:
            b = self._bill(pos)
            if b is None:
                continue
            img = self._small.render(txt, True, self._theme()["accent"])
            img.set_alpha(int(255 * (1.0 - age / 0.9)))
            s.blit(img, img.get_rect(center=(int(b[0]),
                                             int(b[1] - age * 46))))

    def _draw_tracer_flash(self, s):
        gx, gy = int(self.width * 0.54), int(self.height * 0.98)
        cx, cy = int(self._scx), int(self._scy)
        ac = self._theme()["accent"]
        if self._tracer > 0:
            f = self._tracer / 0.08
            dim = tuple(int(c * 0.45 * f) for c in ac)
            bright = tuple(int(c * f) for c in ac)
            pygame.draw.line(s, dim, (gx, gy), (cx, cy), 3)
            pygame.draw.line(s, bright, (gx, gy), (cx, cy), 1)
        if self._flash > 0:
            r = 12
            pygame.draw.circle(s, (255, 240, 200), (gx, gy - 4), r // 2)
            for a in (0, 90, 45, 135):
                rad = math.radians(a)
                pygame.draw.line(s, (255, 220, 160),
                                 (gx - math.cos(rad) * r, gy - 4 - math.sin(rad) * r),
                                 (gx + math.cos(rad) * r, gy - 4 + math.sin(rad) * r),
                                 2)

    def _draw_crosshair(self, s):
        cx, cy = int(self._scx), int(self._scy)
        col = self._theme()["accent"]
        if self._hitmark > 0:
            col = (255, 255, 255)
        ext = int(4 * self._kick)
        gap, arm = 5, 9 + ext
        for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)):
            pygame.draw.line(s, col, (cx + dx * gap, cy + dy * gap),
                             (cx + dx * (gap + arm), cy + dy * (gap + arm)), 2)
        pygame.draw.circle(s, col, (cx, cy), 2)
        ring = pygame.Surface((30, 30), pygame.SRCALPHA)
        pygame.draw.circle(ring, (*col, 120), (15, 15), 14, 1)
        s.blit(ring, (cx - 15, cy - 15))

    # ----- HUD / Ergebnis ---------------------------------------------------------

    def _fmt_mmss(self, sec):
        sec = int(sec)
        return f"{sec // 60:02d}:{sec % 60:02d}"

    def _draw_hud(self, s):
        col = self._theme()["accent"]
        img = self._big.render(t("common.points", score=self.score), True, col)
        s.blit(img, (14, 10))
        acc = int(self.hits / self.shots * 100) if self.shots else 100

        right = []
        if self.mode == "precision":
            right = [f"{max(0.0, self.time_left):.1f}",
                     f"{self.hits}/{self.shots} · {acc}%"]
        elif self.mode == "reflex":
            right = [t("aim.targets", n=min(self.spawned, 30), m=30)]
            if self.reactions:
                avg = sum(self.reactions) // len(self.reactions)
                right.append(t("aim.reaction_avg", ms=avg))
        elif self.mode == "moving":
            mult = 1.0 + 0.25 * min(self.combo, 12)
            right = [f"{max(0.0, self.time_left):.1f}",
                     t("aim.combo", m=f"{mult:.2f}"),
                     f"{self.hits}/{self.shots}"]
        else:
            right = [t("aim.hits", n=self.hits),
                     t("aim.session_time", t=self._fmt_mmss(self.play_t))]
            if self.combo > 1:
                mult = 1.0 + 0.25 * min(self.combo, 12)
                right.append(t("aim.combo", m=f"{mult:.2f}"))
        y = 12
        for line in right:
            img = self._small.render(line, True, col)
            s.blit(img, img.get_rect(topright=(self.width - 14, y)))
            y += 22

        # Reflex: letzte Reaktionszeit kurz einblenden
        if self.mode == "reflex" and self.last_reaction is not None \
                and self.play_t - self.last_reaction_t < 0.8:
            ms = self.last_reaction
            c = (110, 220, 140) if ms < 400 else \
                (240, 240, 240) if ms < 700 else (245, 160, 90)
            img = self._big.render(f"{ms} ms", True, c)
            s.blit(img, img.get_rect(center=(self.width // 2,
                                             int(self.height * 0.72))))

        if self.mode == "chill" and not self.game_over:
            img = self._tiny.render(t("aim.end_hint"), True, col)
            s.blit(img, img.get_rect(midbottom=(self.width // 2,
                                                self.height - 6)))

        if self._toast and self._toast_t > 0:
            img = self._small.render(self._toast, True, (240, 240, 240))
            s.blit(img, img.get_rect(midtop=(self.width // 2, 12)))

    def _draw_result(self, s):
        ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        ov.fill((0, 0, 0, 140))
        s.blit(ov, (0, 0))
        cx = self.width // 2
        cy = self.height // 2 - 40
        acc = int(self.hits / self.shots * 100) if self.shots else 100

        head = self._huge.render(t("aim.result_title"), True,
                                 self._theme()["accent"])
        s.blit(head, head.get_rect(center=(cx, cy - 70)))
        lines = [t("common.points", score=self.score)]
        if self.mode == "precision":
            lines += [f"{t('aim.hits', n=self.hits)}   ·   "
                      f"{t('aim.misses', n=self.misses)}",
                      t("aim.accuracy", p=acc),
                      t("aim.bonus_accuracy", n=getattr(self, "acc_bonus", 0))]
        elif self.mode == "reflex":
            avg = sum(self.reactions) // len(self.reactions) \
                if self.reactions else 0
            best = min(self.reactions) if self.reactions else 0
            lines += [f"{t('aim.hits', n=self.hits)} / 30",
                      t("aim.reaction_avg", ms=avg),
                      t("aim.reaction_best", ms=best)]
        elif self.mode == "moving":
            lines += [t("aim.accuracy", p=acc),
                      t("aim.max_combo", m=1.0 + 0.25 * min(self.max_combo, 12))]
        else:
            lines += [t("aim.hits", n=self.hits),
                      t("aim.max_combo", m=1.0 + 0.25 * min(self.max_combo, 12)),
                      t("aim.session_time", t=self._fmt_mmss(self.play_t))]
        y = cy - 24
        for line in lines:
            img = self.font.render(str(line), True, (235, 238, 245))
            s.blit(img, img.get_rect(center=(cx, y)))
            y += 32
        hint = self._small.render(t("common.enter_restart"), True,
                                  (150, 158, 178))
        s.blit(hint, hint.get_rect(center=(cx, y + 10)))

    # ----- Setup zeichnen -----------------------------------------------------------
    def _draw_setup(self, s):
        # Live-Vorschau des gewählten Themes mit langsamer Auto-Drehung
        old_yaw, old_pitch = self.yaw, self.pitch
        self.yaw = (self.anim_t * math.radians(6)) % math.tau
        self.pitch = math.radians(4)
        self._basis = self._view_basis()
        self._draw_background(s)
        self._apply_blur(s)          # Live-Vorschau der Blur-Einstellung
        self.yaw, self.pitch = old_yaw, old_pitch

        ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        ov.fill((5, 6, 12, 120))
        s.blit(ov, (0, 0))

        cx = self.width // 2
        title = self._huge.render("AIM TRAINER", True, self._theme()["accent"])
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.13))))
        mode_lbl = t("aim.mode." + self.mode) if self.mode in MODE_CFG \
            else self.mode
        sub = self._small.render(mode_lbl + "   -   " + t("aim.subtitle"),
                                 True, (200, 205, 220))
        s.blit(sub, sub.get_rect(center=(cx, int(self.height * 0.20))))

        lbl = self._small.render(t("aim.setup_theme"), True, (150, 158, 178))
        s.blit(lbl, lbl.get_rect(midbottom=(cx, self.theme_rects[0].y - 8)))
        for i, r in enumerate(self.theme_rects):
            on = (THEME_KEYS[i] == self.theme_key)
            pygame.draw.rect(s, (48, 60, 84) if on else (32, 38, 54), r,
                             border_radius=8)
            pygame.draw.rect(s, self._theme()["accent"] if on else (74, 84, 116),
                             r, 2 if on else 1, border_radius=8)
            img = self._small.render(t("aim.theme." + THEME_KEYS[i]), True,
                                     (235, 238, 245) if on else (150, 158, 178))
            s.blit(img, img.get_rect(center=r.center))

        for label_key, minus, plus, box, value in (
                ("aim.setup_sens", self.sens_minus, self.sens_plus,
                 self.sens_box, f"{self.sens:.1f}"),
                ("aim.setup_blur", self.blur_minus, self.blur_plus,
                 self.blur_box, self._blur_label())):
            lbl = self._small.render(t(label_key), True, (200, 205, 220))
            s.blit(lbl, lbl.get_rect(midright=(minus.x - 16, minus.centery)))
            for r, sym in ((minus, "-"), (plus, "+")):
                pygame.draw.rect(s, (32, 38, 54), r, border_radius=8)
                pygame.draw.rect(s, (74, 84, 116), r, 1, border_radius=8)
                img = self._big.render(sym, True, (235, 238, 245))
                s.blit(img, img.get_rect(center=r.center))
            img = self._big.render(value, True, self._theme()["accent"])
            s.blit(img, img.get_rect(center=box.center))

        pygame.draw.rect(s, (48, 60, 84), self.start_rect, border_radius=10)
        pygame.draw.rect(s, self._theme()["accent"], self.start_rect, 2,
                         border_radius=10)
        st = self.font.render(t("common.start"), True, (235, 238, 245))
        s.blit(st, st.get_rect(center=self.start_rect.center))

        hint = self._tiny.render(t("aim.setup_hint"), True, (150, 158, 178))
        s.blit(hint, hint.get_rect(center=(cx, self.height - 14)))
