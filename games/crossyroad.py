# -*- coding: utf-8 -*-
"""
crossyroad.py
=============
Crossy Road - endlos über Wiesen, Straßen, Flüsse und Gleise hüpfen.

Welt und Regeln stehen in ``crossyroad_world.py`` (seed-basiert, bitgenau wie
die Web-Fassung), die Voxel-Grafik in ``crossyroad_draw.py``. Hier: Eingabe,
Spielablauf, Kamera, Effekte, HUD sowie Setup- und Figuren-Screen.

- Punkte = weiteste erreichte Reihe. Münzen liegen verstreut (Riesenmünze = 5)
  und kaufen im Reiter "Figuren" neue Figuren (Huhn, Frosch, Lama, ...).
- Gefahren: Autos/Laster, Züge (Warnlicht + Klingel vorher), Wasser (nur auf
  Stämmen und Seerosen stehen), vom Stamm aus dem Bild getragen werden - und
  der Adler, wenn man zu lange trödelt oder zu weit zurückgeht: Die Kamera
  kriecht langsam vorwärts.
- Modi: Endlos (Highscore) und Tagesstrecke (für alle gleich, eigener
  Tagesbestwert, zählt nicht in den Highscore).
- Ab Reihe 50 wechseln Tag und Nacht; nachts leuchten die Scheinwerfer.

Steuerung: Pfeile/WASD = hüpfen, Leertaste/Enter/Klick = vorwärts.
mem.json-Section "crossy": {coins, unlocked, selected, daily: {date, best}}.
"""

import math
import random
import time

import pygame

import audio
import seedrand
import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent
from i18n import t

from . import crossyroad_draw as cd
from . import crossyroad_world as cw

SETUP, PLAY = "setup", "play"
TABS = ("play", "chars")
LAND_T = 0.12             # Stauch-Dauer nach der Landung
BUMP_T = 0.14             # "Nicken" beim Sprung gegen einen Baum
DEATH_DUR = {"car": 1.25, "train": 1.25, "water": 1.25, "edge": 1.25, "eagle": 1.75}
NIGHT_TINT = (62, 76, 140)
FOCUS_Y = 0.66            # Bildhöhe (Anteil), auf der die Kamera-Reihe liegt
GRID_COLS = 5             # Figuren-Raster


def _rgba(color, alpha):
    return (color[0], color[1], color[2], alpha)


class CrossyRoadGame(Game):
    name = "Crossy Road"
    highscore_key = "crossy"
    supports_multiplayer = False
    MODES = [("endless", "cr.mode.endless"), ("daily", "cr.mode.daily")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        if self.mode not in ("endless", "daily"):
            self.mode = "endless"
        cs = self.settings.get("crossy", {}) if isinstance(self.settings, dict) else {}
        self.shadows = bool(cs.get("shadows", True))
        self.daynight = bool(cs.get("daynight", True))
        self.data = self._load()
        self.anim = 0.0
        self.state = SETUP
        self.tab = "play"
        self.cursor = cw.CHAR_IDS.index(self.data["selected"])
        self.world = None
        self.death = None
        self.particles = []
        self.ripples = []
        self._night = None
        self._vignette = None
        self._last_draw = None
        self._make_fonts()
        self._make_voxels()
        self._layout_setup()

    def _make_fonts(self):
        h = self.height
        self._small = ui.font(max(13, min(22, h // 30)))
        self._tiny = ui.font(max(11, min(18, h // 38)))
        self._huge = ui.font(max(26, h // 11), bold=True)
        self._num = ui.font(max(26, h // 10), bold=True)
        self._mid = ui.font(max(15, min(26, h // 24)), bold=True)

    def _make_voxels(self):
        self.vox = cd.Voxel(max(20, self.width // 14), self.shadows)
        self._stage_vox = None
        self._card_vox = None

    def on_surface_changed(self):
        self._make_fonts()
        self._make_voxels()
        self._layout_setup()
        self._night = None
        self._vignette = None

    @property
    def show_highscore_banner(self):
        return self.mode != "daily"

    @property
    def wants_escape(self):
        return self.state == SETUP and self.tab == "chars"

    # ===================================================== Speicherstand
    def _load(self):
        raw = store.load_section("crossy")
        coins = raw.get("coins", 0)
        coins = coins if isinstance(coins, int) and not isinstance(coins, bool) else 0
        unlocked = [c for c in cw.CHAR_IDS
                    if c == "chicken" or c in (raw.get("unlocked") or [])]
        selected = raw.get("selected")
        if selected not in unlocked:
            selected = "chicken"
        daily = raw.get("daily") if isinstance(raw.get("daily"), dict) else {}
        best = daily.get("best", 0)
        daily = {"date": str(daily.get("date", ""))[:10],
                 "best": best if isinstance(best, int) and best > 0 else 0}
        return {"coins": max(0, coins), "unlocked": unlocked, "selected": selected,
                "daily": daily}

    def _save(self):
        store.save_section("crossy", {
            "coins": int(self.data["coins"]),
            "unlocked": list(self.data["unlocked"]),
            "selected": self.data["selected"],
            "daily": dict(self.data["daily"]),
        })

    def _daily_best(self):
        d = self.data["daily"]
        return d["best"] if d.get("date") == seedrand.today_str() else 0

    def _setup_best(self):
        """Highscore für den Setup-Screen - nur einmal je Besuch von der Platte lesen."""
        if getattr(self, "_setup_hs", None) is None:
            import highscore
            self._setup_hs = highscore.load_highscores().get(self.highscore_key, 0)
        return self._setup_hs

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("crossy", {})[key] = value
            settings_mod.save_settings(self.settings)

    def on_exit(self):
        # Eine laufende Runde: gesammelte Münzen nicht verlieren.
        if self.state == PLAY and self.world is not None and not self.game_over \
                and self.run_coins:
            self._save()

    # ===================================================== Runde
    def _new_run(self):
        if self.mode == "daily":
            seed = seedrand.daily_seed("crossy")
        else:
            seed = random.getrandbits(32)
        self.world = cw.World(seed)
        self.clock = 0.0
        self.col = float(cw.START_COL)
        self.row = 0
        self.facing = "up"
        self.hop = None
        self.buffer = None
        self.log = None
        self.best = 0
        self.score = 0
        self.run_coins = 0
        self.taken = set()
        self.started = False
        self.creep = -cw.CREEP_LAG
        self.cam = 0.0
        self.cam_target = 0.0
        self.death = None
        self.particles = []
        self.ripples = []
        self.land_t = 0.0
        self.bump_t = 0.0
        self.hops = 0
        self.bell_next = 0.0
        self.passes = set()
        self.record = False
        self.game_over = False
        self.state = PLAY
        self._last_draw = None
        import highscore
        self.hs = highscore.load_highscores().get(self.highscore_key, 0)
        self.day_best = self._daily_best()
        self.play_sound("click")

    def _finish_run(self):
        """Todes-Animation vorbei: Ergebnis sichern, Game Over zeigen."""
        if self.mode == "daily":
            today = seedrand.today_str()
            d = self.data["daily"]
            if d.get("date") != today:
                d["date"], d["best"] = today, 0
            self.record = self.best > d["best"]
            if self.record:
                d["best"] = self.best
        else:
            self.record = self.best > self.hs
        self._save()
        self.game_over = True
        self.play_sound("gameover")
        self._layout_over()

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.game_over:
            self._handle_over(event)
            return
        if self.death is not None:
            return
        if event.kind == InputEvent.KEYDOWN:
            if event.repeat:
                return                     # kein Dauerhüpfen durch Tastenwiederholung
            k = event.key
            if self.is_action(k, "up") or self.is_action(k, "action") or k == "Up":
                self._press("up")
            elif self.is_action(k, "down") or k == "Down":
                self._press("down")
            elif self.is_action(k, "left") or k == "Left":
                self._press("left")
            elif self.is_action(k, "right") or k == "Right":
                self._press("right")
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            self._press("up")

    def _press(self, d):
        if self.hop is not None:
            self.buffer = d               # ein Hüpfer wird vorgemerkt
            return
        self._try_hop(d)

    def _try_hop(self, d):
        dc, dr = cw.DIRS[d]
        self.facing = d
        fr = self.row
        tr = fr + dr
        fc = self.col
        cur = self.world.row(fr)
        dst = self.world.row(tr)
        on_log = self.log is not None
        drift = 0.0
        if dr == 0:
            tc = fc + dc
            if on_log:
                drift = cur["dir"] * cur["speed"]
        elif dst["kind"] == cw.RIVER and dst["pads"] is None:
            tc = fc                        # auf einen Stamm: Position behalten
        else:
            tc = float(math.floor(fc + 0.5))
        if self._blocked(dst, tc):
            self.bump_t = BUMP_T
            audio.tone(170, 0.05, self.settings, "square", 0.10)
            return
        if not self.started:
            self.started = True
        self.hop = {"fc": fc, "fr": fr, "tc": tc, "tr": tr, "t": 0.0, "drift": drift,
                    "z0": self._stand_z(cur), "z1": self._stand_z(dst)}
        self.log = None
        self.hops += 1
        audio.tone(540 + 50 * (self.hops % 3), 0.045, self.settings, "sine", 0.16)

    def _blocked(self, row, tc):
        if row["kind"] == cw.RIVER and row["pads"] is None:
            return not (-0.5 <= tc <= cw.COLS - 0.5)
        c = int(math.floor(tc + 0.5))
        if c < 0 or c >= cw.COLS:
            return True
        return bool(row["trees"] and row["trees"][c])

    def _stand_z(self, row):
        """Standhöhe der Figur auf einer Reihe (Stamm/Seerose über dem Wasser)."""
        lvl = cw.LEVEL[row["kind"]]
        if row["kind"] == cw.RIVER:
            return lvl + (0.05 if row["pads"] is not None else 0.26)
        return lvl

    # ===================================================== Spiellogik
    def update(self, dt):
        self.anim += dt
        if self.state != PLAY or self.world is None:
            return
        dt = min(dt, 0.05)
        self.clock += dt
        self._step_effects(dt)
        if self.death is not None:
            self.death["t"] += dt
            if self.death["t"] >= self.death["dur"]:
                self._finish_run()
            return
        self.land_t = max(0.0, self.land_t - dt)
        self.bump_t = max(0.0, self.bump_t - dt)
        if self.hop is not None:
            h = self.hop
            h["t"] += dt
            if h["t"] >= cw.HOP_T:
                self._land()
        elif self.log is not None:
            row = self.world.row(self.row)
            idx, seg = self.log
            self.col = cw.obj_x(row, row["objs"][idx], self.clock) + seg
            cx = self.col + 0.5
            if cx < 0.0 or cx > cw.COLS:
                self._die("edge")
                return
        if self.death is None:
            self._collide()
        if self.death is None:
            self._camera(dt)
        self._sounds()

    def _hop_frac(self):
        return min(1.0, self.hop["t"] / cw.HOP_T) if self.hop else 1.0

    def _pos(self):
        """Aktuelle Figur-Position (x = linke Zellkante, y = Reihe, z)."""
        if self.hop is None:
            z = self._stand_z(self.world.row(self.row))
            if self.log is not None:
                z += 0.012 * math.sin(self.clock * 3.0 + self.log[0])
            return self.col, float(self.row), z
        h = self.hop
        p = self._hop_frac()
        x = h["fc"] + (h["tc"] - h["fc"]) * p + h["drift"] * h["t"]
        y = h["fr"] + (h["tr"] - h["fr"]) * p
        z = h["z0"] + (h["z1"] - h["z0"]) * p + cw.HOP_Z * math.sin(math.pi * p)
        return x, y, z

    def _row_now(self):
        if self.hop is None:
            return self.row
        return self.hop["tr"] if self._hop_frac() >= 0.5 else self.hop["fr"]

    def _land(self):
        h = self.hop
        self.hop = None
        self.row = h["tr"]
        self.col = h["tc"] + h["drift"] * cw.HOP_T
        self.land_t = LAND_T
        row = self.world.row(self.row)
        if row["kind"] == cw.RIVER:
            if row["pads"] is not None:
                c = int(math.floor(self.col + 0.5))
                if 0 <= c < cw.COLS and row["pads"][c]:
                    self.col = float(c)
                    self._ripple(c + 0.5, self.row + 0.5, 0.5)
                else:
                    self._die("water")
                    return
            else:
                hit = cw.log_at(row, self.col + 0.5, self.clock)
                if hit is None:
                    self._die("water")
                    return
                self.log = hit
                self.col = cw.obj_x(row, row["objs"][hit[0]], self.clock) + hit[1]
                self._ripple(self.col + 0.5, self.row + 0.5, 0.6)
        else:
            self.col = float(math.floor(self.col + 0.5))
        # Münze?
        coin = row["coin"]
        if coin and self.row not in self.taken and coin[0] == int(math.floor(self.col + 0.5)):
            self.taken.add(self.row)
            self.run_coins += coin[1]
            self.data["coins"] += coin[1]
            self._sparkle(coin[0] + 0.5, self.row + 0.5, self._stand_z(row) + 0.4, coin[1] > 1)
            self.play_sound("powerup" if coin[1] > 1 else "eat")
        # Neue weiteste Reihe?
        if self.row > self.best:
            self.best = self.row
            if self.mode != "daily":
                self.score = self.best
            if row["kind"] != cw.RAIL and self.world.rail_run(self.row - 1) >= 5:
                self.ach_event("crossy_train")
        if self.buffer is not None and self.death is None:
            d, self.buffer = self.buffer, None
            self._try_hop(d)

    def _collide(self):
        x, _, _ = self._pos()
        cx = x + 0.5
        rr = self._row_now()
        row = self.world.row(rr)
        if row["kind"] == cw.ROAD:
            i = cw.vehicle_at(row, cx, self.clock)
            if i >= 0:
                self._die("car", {"row": rr, "obj": i})
        elif row["kind"] == cw.RAIL:
            if cw.train_hits(row, cx, self.clock):
                self._die("train", {"row": rr})

    def _camera(self, dt):
        if self.started:
            self.creep += cw.creep_rate(self.best) * dt
        self.creep = max(self.creep, self.best - cw.CREEP_LAG)
        self.cam_target = max(self.creep, float(self.best))
        self.cam += (self.cam_target - self.cam) * min(1.0, dt * 5.0)
        if self.started and self.cam_target - self._row_now() > cw.EAGLE_ROWS:
            self._die("eagle")

    def _danger(self):
        """0..1 - wie nah der Adler ist (für die rote Warnung am Rand)."""
        if self.world is None or not self.started or self.death is not None:
            return 0.0
        behind = self.cam_target - self._row_now()
        return max(0.0, min(1.0, (behind - (cw.EAGLE_ROWS - 1.3)) / 1.3))

    def _sounds(self):
        """Klingel für Züge in der Nähe, Rauschen, wenn einer vorbeidonnert."""
        ring = False
        for r in range(self.row - 3, self.row + 7):
            if r < 0:
                continue
            row = self.world.row(r)
            if row["kind"] != cw.RAIL:
                continue
            warn, head, length = cw.train_state(row, self.clock)
            if warn and head is None:
                ring = True
            if head is not None and abs(r - self.row) <= 3:
                n = int((self.clock + row["phase"]) // row["period"])
                if (r, n) not in self.passes:
                    self.passes.add((r, n))
                    audio.tone(90, 0.45, self.settings, "noise", 0.22)
        if ring and self.clock >= self.bell_next:
            self.bell_next = self.clock + 0.3
            audio.tone(1320, 0.07, self.settings, "square", 0.10)

    def _die(self, cause, info=None):
        x, y, z = self._pos()
        self.death = {"cause": cause, "t": 0.0, "dur": DEATH_DUR[cause],
                      "x": x, "y": y, "z": z, "info": info or {}}
        self.hop = None
        self.buffer = None
        self.log = None
        col = cw.CHAR_COLORS[self.data["selected"]]
        if cause == "car":
            self.play_sound("hit")
            audio.tone(620, 0.22, self.settings, "square", 0.12)
            self._burst(x + 0.5, y + 0.5, z + 0.3, col, 14, 2.2)
            self.rumble(180)
        elif cause == "train":
            self.play_sound("explode")
            row = self.world.row(int(round(y)))
            self._burst(x + 0.5, y + 0.5, z + 0.4, col, 26, 4.0, row["dir"] * 6.0)
            self.rumble(260)
        elif cause in ("water", "edge"):
            audio.tone(160, 0.3, self.settings, "noise", 0.3)
            audio.tone(420, 0.12, self.settings, "sine", 0.15)
            wz = cw.LEVEL[cw.RIVER]
            self.death["z"] = wz
            self._ripple(x + 0.5, y + 0.5, 1.0)
            self._ripple(x + 0.5, y + 0.5, 0.6)
            self._burst(x + 0.5, y + 0.5, wz + 0.1, (236, 250, 255), 22, 1.6, 0, 6.0)
            self._burst(x + 0.5, y + 0.5, wz + 0.1, (140, 214, 250), 12, 2.2, 0, 4.0)
            self.rumble(140)
        else:
            self.play_sound("shoot")
            self.rumble(220)

    # ----- Effekte ---------------------------------------------------------
    def _burst(self, x, y, z, col, n, speed, push=0.0, up=3.5):
        for _ in range(n):
            a = random.uniform(0, math.tau)
            sp = random.uniform(0.3, 1.0) * speed
            self.particles.append([x, y, z, math.cos(a) * sp + push * random.uniform(0.4, 1.0),
                                   math.sin(a) * sp * 0.6, random.uniform(0.5, 1.0) * up,
                                   0.0, random.uniform(0.6, 1.1), col,
                                   random.uniform(0.06, 0.12)])

    def _sparkle(self, x, y, z, big):
        for _ in range(16 if big else 9):
            a = random.uniform(0, math.tau)
            sp = random.uniform(0.6, 1.6)
            self.particles.append([x, y, z, math.cos(a) * sp, math.sin(a) * sp * 0.5,
                                   random.uniform(1.5, 3.0), 0.0, random.uniform(0.35, 0.6),
                                   (255, 222, 90), random.uniform(0.05, 0.09)])

    def _ripple(self, x, y, strength):
        self.ripples.append([x, y, 0.0, strength])

    def _step_effects(self, dt):
        keep = []
        for p in self.particles:
            p[6] += dt
            if p[6] >= p[7]:
                continue
            p[0] += p[3] * dt
            p[1] += p[4] * dt
            p[2] += p[5] * dt
            p[5] -= 9.0 * dt
            p[3] *= 1.0 - 1.5 * dt
            if p[2] < -0.2:
                p[2] = -0.2
                p[5] = 0.0
            keep.append(p)
        self.particles = keep[-240:]
        self.ripples = [r for r in self.ripples if r[2] + dt < 0.8]
        for r in self.ripples:
            r[2] += dt

    # ===================================================== Setup-Screen
    def _layout_setup(self):
        W, H = self.width, self.height
        cx = W // 2
        tab_h = max(22, min(32, H // 15))
        tab_w = min(170, (W - 40) // 2)
        tab_y = int(H * 0.205)
        self.tab_rects = [pygame.Rect(cx - tab_w - 4, tab_y, tab_w, tab_h),
                          pygame.Rect(cx + 4, tab_y, tab_w, tab_h)]
        top = tab_y + tab_h + max(10, H // 36)
        bw = min(max(380, int(W * 0.62)), W - 40)
        bh = max(28, min(46, H // 16))
        self.bh = bh
        # --- Reiter SPIELEN: Karte (Figur + Modus + Bestwerte), Schalter, Start
        card_h = max(96, int(H * 0.30))
        lab = max(20, self._tiny.get_height() + 10)
        gap1 = lab + max(0, (H - 360) // 40)
        gap2 = max(12, H // 30)
        used = card_h + gap1 + bh + gap2 + bh + 6
        free = (H - 44) - top - used
        ptop = top + max(0, free // 2)
        self.card_rect = pygame.Rect(cx - bw // 2, ptop, bw, card_h)
        y = self.card_rect.bottom + gap1
        grp = (bw - 18) / 2.0
        self.opt_w = int(grp)

        def pair(x0):
            w = (grp - 8) / 2.0
            return [pygame.Rect(int(x0), y, int(w), bh),
                    pygame.Rect(int(x0 + w + 8), y, int(w), bh)]
        self.shadow_rects = pair(cx - bw / 2)
        self.night_rects = pair(cx + bw / 2 - grp)
        y += bh + gap2
        sw = max(190, int(bw * 0.42))
        self.start_rect = pygame.Rect(cx - sw // 2, y, sw, bh + 6)
        # --- Reiter FIGUREN: 5x2-Raster + Detailzeile
        gw = min(W - 36, 760)
        gap = max(6, W // 100)
        detail_h = bh + 6
        bottom = H - 26 - max(8, H // 40)
        avail = bottom - top - detail_h - gap * 2
        cwid = (gw - gap * (GRID_COLS - 1)) / GRID_COLS
        chei = min(cwid * 1.12, (avail - gap) / 2.0)
        self.grid_rects = []
        for i in range(len(cw.CHAR_IDS)):
            gx, gy = i % GRID_COLS, i // GRID_COLS
            self.grid_rects.append(pygame.Rect(int(cx - gw / 2 + gx * (cwid + gap)),
                                               int(top + gy * (chei + gap)),
                                               int(cwid), int(chei)))
        dy = self.grid_rects[-1].bottom + gap * 2
        self.detail_rect = pygame.Rect(cx - gw // 2, dy, gw, detail_h)
        self.buy_rect = pygame.Rect(self.detail_rect.right - int(gw * 0.36),
                                    dy + 3, int(gw * 0.36) - 3, detail_h - 6)
        self._card_vox = None

    def _handle_setup(self, event):
        if event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.tab_rects):
                if rc.collidepoint(event.pos):
                    self._set_tab(TABS[i])
                    return
        elif event.kind == InputEvent.KEYDOWN and event.key in ("Tab", "ISO_Left_Tab"):
            self._set_tab("chars" if self.tab == "play" else "play")
            return
        if self.tab == "chars":
            self._handle_chars(event)
            return
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("Return", "space"):
                self._new_run()
            elif k == "Left" or self.is_action(k, "left"):
                self._cycle_char(-1)
            elif k == "Right" or self.is_action(k, "right"):
                self._cycle_char(1)
            elif k in ("h", "H") and self.key_is_free(k):
                self._toggle("shadows")
            elif k in ("n", "N") and self.key_is_free(k):
                self._toggle("daynight")
        elif event.kind == InputEvent.MOUSEDOWN:
            for flag, rects in (("shadows", self.shadow_rects), ("daynight", self.night_rects)):
                for i, rc in enumerate(rects):
                    if rc.collidepoint(event.pos):
                        if getattr(self, flag) != (i == 0):
                            self._toggle(flag)
                        return
            if self.start_rect.collidepoint(event.pos):
                self._new_run()
            elif self.card_rect.collidepoint(event.pos):
                self._set_tab("chars")

    def _set_tab(self, tab):
        if tab != self.tab:
            self.tab = tab
            self.cursor = cw.CHAR_IDS.index(self.data["selected"])
            self.play_sound("click")

    def _toggle(self, flag):
        val = not getattr(self, flag)
        setattr(self, flag, val)
        self._save_setting(flag, val)
        if flag == "shadows":
            self._make_voxels()
        self.play_sound("click")

    def _cycle_char(self, step):
        owned = [c for c in cw.CHAR_IDS if c in self.data["unlocked"]]
        i = owned.index(self.data["selected"])
        self.data["selected"] = owned[(i + step) % len(owned)]
        self._save()
        self.play_sound("select")

    def _handle_chars(self, event):
        n = len(cw.CHAR_IDS)
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k == "Escape":
                self._set_tab("play")
            elif k in ("Return", "space"):
                self._activate_char(self.cursor)
            elif k == "Left" or self.is_action(k, "left"):
                self.cursor = (self.cursor - 1) % n
                self.play_sound("move")
            elif k == "Right" or self.is_action(k, "right"):
                self.cursor = (self.cursor + 1) % n
                self.play_sound("move")
            elif k == "Up" or self.is_action(k, "up"):
                self.cursor = (self.cursor - GRID_COLS) % n
                self.play_sound("move")
            elif k == "Down" or self.is_action(k, "down"):
                self.cursor = (self.cursor + GRID_COLS) % n
                self.play_sound("move")
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.grid_rects):
                if rc.collidepoint(event.pos):
                    self.cursor = i
                    if cw.CHAR_IDS[i] in self.data["unlocked"]:
                        self._activate_char(i)
                    else:
                        self.play_sound("move")
                    return
            if self.buy_rect.collidepoint(event.pos):
                self._activate_char(self.cursor)

    def _activate_char(self, i):
        """Besitzt man die Figur: auswählen. Sonst kaufen, wenn die Münzen reichen."""
        cid = cw.CHAR_IDS[i]
        if cid in self.data["unlocked"]:
            if self.data["selected"] != cid:
                self.data["selected"] = cid
                self._save()
            self.play_sound("select")
            return True
        price = cw.PRICES[cid]
        if self.data["coins"] < price:
            self.play_sound("hit")
            return False
        self.data["coins"] -= price
        self.data["unlocked"] = [c for c in cw.CHAR_IDS
                                 if c in self.data["unlocked"] or c == cid]
        self.data["selected"] = cid
        self._save()
        self.play_sound("win")
        r = self.grid_rects[i]
        ui.spawn_burst(r.centerx, r.centery, self.accent)
        self.ach_event("crossy_char")
        return True

    # ===================================================== Game Over
    @staticmethod
    def _text_h(fnt):
        """Echte Höhe eines gerenderten Textes (bei Bahnschrift fett größer
        als font.get_height())."""
        return fnt.size("ÄGgy")[1]

    def _layout_over(self):
        W, H = self.width, self.height
        pw = min(W - 30, max(360, int(W * 0.56)))
        bh = max(28, min(42, H // 17))
        th = self._text_h
        # Titel, Ursache, Punkte, Bestwert, Münzen (je + 8 Abstand), Knöpfe, Hinweis
        self.over_rows_h = (th(self._huge) + th(self._tiny) + th(self.font)
                            + 2 * th(self._small) + 5 * 8)
        ph = 14 + self.over_rows_h + 12 + bh + 8 + th(self._tiny) + 10
        top = max(8, (H - 56 - ph) // 2)
        self.over_rect = pygame.Rect(W // 2 - pw // 2, top, pw, ph)
        by = self.over_rect.bottom - 10 - th(self._tiny) - 8 - bh
        gap = 8
        bw = (pw - 40 - 2 * gap) / 3.0
        self.over_btns = [(key, pygame.Rect(int(self.over_rect.x + 20 + i * (bw + gap)), by,
                                            int(bw), bh))
                          for i, key in enumerate(("again", "chars", "setup"))]

    def _handle_over(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.repeat:
                return
            k = event.key
            if k in ("Return", "space"):
                self._new_run()
            elif k in ("f", "F"):
                self._to_setup("chars")
            elif k in ("s", "S"):
                self._to_setup("play")
        elif event.kind == InputEvent.MOUSEDOWN:
            for key, rc in self.over_btns:
                if rc.collidepoint(event.pos):
                    if key == "again":
                        self._new_run()
                    else:
                        self._to_setup("chars" if key == "chars" else "play")
                    return

    def _to_setup(self, tab):
        self._setup_hs = None           # main.py hat den Highscore inzwischen gesichert
        self.game_over = False
        self.score = 0
        self.state = SETUP
        self.tab = tab
        self.cursor = cw.CHAR_IDS.index(self.data["selected"])
        self.play_sound("click")

    # ===================================================== Zeichnen: Welt
    def draw(self):
        if self.state == SETUP:
            self._draw_setup(self.surface)
            return
        s = self.surface
        # Nach dem Ende laufen Verkehr und Wasser als Kulisse weiter - update()
        # wird bei Game Over nicht mehr aufgerufen, deshalb hier die Zeit.
        now = time.perf_counter()
        if self.game_over and self._last_draw is not None:
            step = max(0.0, min(0.1, now - self._last_draw))
            self.clock += step
            self.anim += step
        self._last_draw = now
        self._draw_world(s)
        self._draw_eagle(s)
        if self.daynight:
            self._draw_night(s)
        if self.game_over:
            self._draw_over(s)          # das Panel zeigt alle Werte - kein HUD dahinter
        else:
            self._draw_hud(s)

    def _setup_proj(self):
        v = self.vox
        W, H = self.width, self.height
        F = self.cam
        self.ox = int(round(W / 2.0 - (cw.COLS / 2.0) * v.tw - (F + 2.5) * v.sk))
        self.oy = int(round(H * FOCUS_Y + (F + 0.5) * v.th))

    def _scr(self, x, y, z):
        v = self.vox
        return (self.ox + x * v.tw + y * v.sk, self.oy - y * v.th - z * v.zh)

    def _blit(self, s, key, x, y, z):
        surf, ax, ay = self.vox.sprite(key)
        px, py = self._scr(x, y, z)
        s.blit(surf, (int(round(px)) - ax, int(round(py)) - ay))

    def _draw_world(self, s):
        v = self.vox
        H = self.height
        self._setup_proj()
        world = self.world
        r_hi = int(math.ceil(self.oy / v.th)) + 1
        r_lo = int(math.floor((self.oy - H - 2.2 * v.zh) / v.th)) - 1
        clock = self.clock
        p_row = None                    # Reihe, in deren Zeichenfolge die Figur liegt
        if self.death is None:
            p_row = min(self.hop["fr"], self.hop["tr"]) if self.hop else self.row
        elif self.death["cause"] in ("car", "water", "edge"):
            p_row = int(math.floor(self.death["y"] + 0.5))
        for r in range(r_hi, r_lo - 1, -1):
            row = world.row(r)
            kind = row["kind"]
            lvl = cw.LEVEL[kind]
            prev = world.row(r - 1)
            plvl = cw.LEVEL[prev["kind"]]
            lower = plvl if plvl < lvl else None
            line = kind == cw.ROAD and world.row(r + 1)["kind"] == cw.ROAD
            surf, ax, ay = v.strip(kind, r & 1, lower, line)
            sx, sy = self._scr(0, r, 0)
            s.blit(surf, (int(sx) - ax, int(sy) - ay))
            items = []
            if kind == cw.GRASS:
                trees = row["trees"]
                for c in range(cw.COLS):
                    if trees[c]:
                        items.append((1, c, "tree%d" % trees[c], c, lvl))
                for c in (-4, -3, -2, -1, cw.COLS, cw.COLS + 1, cw.COLS + 2, cw.COLS + 3):
                    hsh = cw.deco_hash(r, c)
                    if hsh % 100 < (70 if c in (-1, cw.COLS) else 45):
                        items.append((1, c, "tree%d" % (1 + (hsh >> 8) % 3), c, lvl))
            elif kind == cw.ROAD:
                for o in row["objs"]:
                    x = cw.obj_x(row, o, clock)
                    if -7.0 < x < cw.COLS + 5.0:
                        name = ("truck%d" if o[1] > 2 else "car%d") % o[2]
                        items.append((1, x, name + ("" if row["dir"] > 0 else "<"), x, lvl))
            elif kind == cw.RIVER:
                if row["pads"] is not None:
                    for c in range(cw.COLS):
                        if row["pads"][c]:
                            bob = 0.012 * math.sin(clock * 2.0 + c)
                            items.append((0, c, "pad%d" % (cw.deco_hash(r, c) % 3 == 0),
                                          c, lvl + bob))
                else:
                    for i, o in enumerate(row["objs"]):
                        x = cw.obj_x(row, o, clock)
                        if -7.0 < x < cw.COLS + 5.0:
                            bob = 0.012 * math.sin(clock * 3.0 + i)
                            items.append((0, x, "log%d" % o[1], x, lvl + bob))
            else:
                warn, head, length = cw.train_state(row, clock)
                lit = (1 + int(clock * 6) % 2) if warn else 0
                items.append((1, -1.2, "signal%d" % lit, -1.2, lvl))
                if head is not None:
                    n = row["cars"] + 1
                    for k in range(n):
                        if row["dir"] > 0:
                            x = head - cw.TRAIN_CAR * (k + 1)
                            key = "engine" if k == 0 else "wagon"
                        else:
                            x = head + cw.TRAIN_CAR * k
                            key = "engine<" if k == 0 else "wagon<"
                        if -9.0 < x < cw.COLS + 6.0:
                            items.append((1, x, key, x, lvl))
            coin = row["coin"]
            if coin and r not in self.taken:
                bob = 0.06 + 0.05 * math.sin(self.anim * 3.0 + r)
                items.append((1, coin[0] - 0.01, "bigcoin" if coin[1] > 1 else "coin",
                              coin[0], lvl + bob))
            if p_row == r:
                items.append((2, 0, None, 0, 0))
            items.sort(key=lambda it: (it[0] if it[0] != 2 else 1, it[1]))
            for layer, _, key, x, z in items:
                if key is None:
                    self._draw_player(s)
                else:
                    self._blit(s, key, x, r, z)
            if self.ripples:
                self._draw_ripples(s, r)
        self._draw_particles(s)
        self._draw_danger(s)

    def _draw_player(self, s):
        cid = self.data["selected"]
        if self.death is not None:
            d = self.death
            x, y, z = d["x"], d["y"], d["z"]
            if d["cause"] == "car":
                q = min(1.0, d["t"] / 0.08)
                sx, sy = 1.0 + 0.4 * q, 1.0 - 0.8 * q
                self._player_sprite(s, cid, self.facing, x, y, z, sx, sy, shadow=False)
            elif d["cause"] in ("water", "edge"):
                if d["t"] < 0.14:
                    sink = d["t"] / 0.14
                    self._player_sprite(s, cid, self.facing, x, y, z - 0.5 * sink,
                                        1.0, 1.0 - 0.6 * sink, shadow=False)
            return
        x, y, z = self._pos()
        sx = sy = 1.0
        if self.hop is not None:
            k = math.sin(math.pi * self._hop_frac())
            sx, sy = 1.0 - 0.10 * k, 1.0 + 0.16 * k
        elif self.land_t > 0:
            q = self.land_t / LAND_T
            sx, sy = 1.0 + 0.16 * q, 1.0 - 0.24 * q
        elif self.bump_t > 0:
            q = math.sin(math.pi * self.bump_t / BUMP_T)
            sx, sy = 1.0 + 0.08 * q, 1.0 - 0.12 * q
        else:
            # leichtes "Atmen" im Stand
            k = 0.5 + 0.5 * math.sin(self.anim * 4.0)
            sy = 1.0 - 0.03 * k
            sx = 1.0 + 0.02 * k
        ground = self._stand_z(self.world.row(self._row_now()))
        self._player_sprite(s, cid, self.facing, x, y, z, sx, sy, ground=ground)

    def _player_sprite(self, s, cid, facing, x, y, z, sx, sy, shadow=True, ground=None):
        v = self.vox
        if cid in cw.HOVER:
            z += 0.14 + 0.05 * math.sin(self.anim * 3.0)
        if shadow and self.shadows:
            gz = z if ground is None else ground
            lift = max(0.0, z - gz)
            self._shadow(s, x + 0.5, y + 0.5, gz, max(0.45, 1.0 - lift * 0.8))
        key = "%s:%s" % (cid, facing)
        surf, ax, ay = v.scaled(key, sx, sy)
        base, bax, bay = v.sprite(key, False)
        fx = bax + 0.5 * v.tw + 0.5 * v.sk
        fy = bay - 0.5 * v.th
        px, py = self._scr(x + 0.5, y + 0.5, z)
        s.blit(surf, (int(round(px - fx * sx)), int(round(py - fy * sy))))

    def _shadow(self, s, cx, cy, z, scale):
        v = self.vox
        key = int(scale * 10)
        cache = getattr(self, "_shadow_cache", None)
        if cache is None or cache[0] is not v:
            cache = self._shadow_cache = (v, {})
        surf = cache[1].get(key)
        if surf is None:
            w = int(v.tw * 0.7 * key / 10.0) + 2
            h = int(v.th * 0.6 * key / 10.0) + 2
            surf = pygame.Surface((w, h), pygame.SRCALPHA)
            pygame.draw.ellipse(surf, (0, 0, 0, 70), (0, 0, w, h))
            cache[1][key] = surf
        px, py = self._scr(cx + 0.06, cy - 0.04, z)
        s.blit(surf, (int(px) - surf.get_width() // 2, int(py) - surf.get_height() // 2))

    def _draw_ripples(self, s, r):
        v = self.vox
        for x, y, age, strength in self.ripples:
            if int(math.floor(y)) != r:
                continue
            f = age / 0.8
            rad = (0.2 + 0.75 * f) * strength
            px, py = self._scr(x, y, cw.LEVEL[cw.RIVER] + 0.02)
            w, h = int(rad * 2 * v.tw), int(rad * 2 * v.th * 0.9)
            if w < 4 or h < 3:
                continue
            col = ui.mix((255, 255, 255), (130, 206, 250), f)
            pygame.draw.ellipse(s, col, (int(px - w / 2), int(py - h / 2), w, h),
                                max(1, int(v.tw * 0.07 * (1 - f))))

    def _draw_particles(self, s):
        v = self.vox
        for x, y, z, vx, vy, vz, age, life, col, size in self.particles:
            px, py = self._scr(x, y, z)
            sz = max(2, int(size * v.tw * (1.0 - 0.5 * age / life)))
            pygame.draw.rect(s, col, (int(px) - sz // 2, int(py) - sz // 2, sz, sz))
            pygame.draw.rect(s, cd.shade(col, 0.75),
                             (int(px) - sz // 2, int(py) + sz // 2 - max(1, sz // 3), sz,
                              max(1, sz // 3)))

    def _draw_danger(self, s):
        k = self._danger()
        if k <= 0:
            return
        W, H = self.width, self.height
        if self._vignette is None or self._vignette.get_size() != (W, H // 3):
            h = H // 3
            vg = pygame.Surface((W, h), pygame.SRCALPHA)
            for i in range(h):
                a = int(150 * (i / float(h)) ** 2)
                pygame.draw.line(vg, (220, 30, 40, a), (0, i), (W, i))
            self._vignette = vg
        vg = self._vignette
        vg.set_alpha(int(255 * k * (0.6 + 0.4 * ui.pulse(6.0))))
        s.blit(vg, (0, H - vg.get_height()))

    def _draw_eagle(self, s):
        d = self.death
        if d is None or d["cause"] != "eagle":
            return
        v = self.vox
        W, H = self.width, self.height
        p = d["t"] / d["dur"]
        px, py = self._scr(d["x"] + 0.5, d["y"] + 0.5, d["z"])
        grab = 0.42
        if p < grab:
            q = p / grab
            q = q * q * (3 - 2 * q)
            ex = px + (1 - q) * W * 0.25
            ey = -H * 0.35 + (py - v.zh * 0.6 + H * 0.35) * q
            carry = False
        else:
            q = (p - grab) / (1 - grab)
            ex = px + q * W * 0.18
            ey = py - v.zh * 0.6 - q * q * (py + H * 0.5)
            carry = True
        cid = self.data["selected"]
        if not carry:
            self._player_sprite(s, cid, self.facing, d["x"], d["y"], d["z"], 1.0, 1.0)
            # Schatten des Adlers wächst am Boden
            self._shadow(s, d["x"] + 0.5 + (1 - q) * 1.5, d["y"] + 0.5, d["z"], 0.6 + q * 1.6)
        else:
            surf, ax, ay = v.sprite("%s:%s" % (cid, self.facing), False)
            fx = ax + 0.5 * v.tw + 0.5 * v.sk
            fy = ay - 0.5 * v.th
            s.blit(surf, (int(ex - fx), int(ey + 1.25 * v.zh - fy)))
        # Flügelschlag = leichtes Stauchen; die Modellmitte liegt auf (ex, ey)
        flap = math.sin(self.anim * 16.0)
        sy = 1.0 + 0.14 * flap
        surf, ax, ay = v.scaled("eagle_down", 1.0, sy)
        base, bax, bay = v.sprite("eagle_down", False)
        mx, my = v.proj(0.0, 0.0, 0.3)
        s.blit(surf, (int(ex - (bax + mx)), int(ey - (bay + my) * sy)))

    # ----- Nacht -----------------------------------------------------------
    def _draw_night(self, s):
        n = cw.night_factor(self.cam + 4.0)
        if n <= 0.01:
            return
        W, H = self.width, self.height
        v = self.vox
        if self._night is None or self._night.get_size() != (W, H):
            self._night = pygame.Surface((W, H)).convert() \
                if pygame.display.get_surface() else pygame.Surface((W, H))
        ov = self._night
        tint = ui.mix((255, 255, 255), NIGHT_TINT, n)
        ov.fill(tint)
        soft = ui.mix(tint, (255, 238, 200), 0.55)
        bright = ui.mix(tint, (255, 246, 222), 0.92)
        clock = self.clock
        r_hi = int(math.ceil(self.oy / v.th)) + 1
        r_lo = int(math.floor((self.oy - H) / v.th)) - 1

        def poly(col, pts):
            pygame.draw.polygon(ov, col, [self._scr(*p) for p in pts])
        for r in range(r_hi, r_lo - 1, -1):
            row = self.world.row(r)
            lvl = cw.LEVEL[row["kind"]]
            coin = row["coin"]
            if coin and r not in self.taken:
                px, py = self._scr(coin[0] + 0.5, r + 0.5, lvl + 0.4)
                rad = int(v.tw * (0.5 if coin[1] > 1 else 0.36))
                pygame.draw.circle(ov, ui.mix(tint, (255, 236, 160), 0.7), (int(px), int(py)), rad)
            if row["kind"] == cw.ROAD:
                d = row["dir"]
                for o in row["objs"]:
                    x = cw.obj_x(row, o, clock)
                    if not (-9.0 < x < cw.COLS + 7.0):
                        continue
                    fx = x + o[1] if d > 0 else x
                    poly(soft, [(fx, r + 0.2, lvl), (fx + 3.6 * d, r - 0.25, lvl),
                                (fx + 3.6 * d, r + 1.25, lvl), (fx, r + 0.8, lvl)])
                    poly(bright, [(fx, r + 0.28, lvl), (fx + 2.4 * d, r + 0.05, lvl),
                                  (fx + 2.4 * d, r + 0.95, lvl), (fx, r + 0.72, lvl)])
            elif row["kind"] == cw.RAIL:
                warn, head, length = cw.train_state(row, clock)
                if head is not None:
                    d = row["dir"]
                    poly(soft, [(head, r + 0.2, lvl), (head + 6.0 * d, r - 0.4, lvl),
                                (head + 6.0 * d, r + 1.4, lvl), (head, r + 0.8, lvl)])
                if warn and int(clock * 6) % 2 == 0:
                    px, py = self._scr(-1.0, r + 0.1, lvl + 1.1)
                    rad = int(v.tw * 0.5)
                    pygame.draw.circle(ov, ui.mix(tint, (255, 150, 140), 0.8),
                                       (int(px), int(py)), rad)
        if self.death is None or self.death["cause"] != "eagle":
            if self.death is None:
                x, y, z = self._pos()
            else:
                x, y, z = self.death["x"], self.death["y"], self.death["z"]
            px, py = self._scr(x + 0.5, y + 0.5, z + 0.3)
            w, h = int(v.tw * 1.9), int(v.th * 1.9)
            pygame.draw.ellipse(ov, soft, (int(px) - w // 2, int(py) - h // 2, w, h))
        s.blit(ov, (0, 0), special_flags=pygame.BLEND_RGB_MULT)

    # ===================================================== HUD
    def _draw_hud(self, s):
        W, H = self.width, self.height
        pad = max(8, W // 60)
        shown = self.best
        # Punkte groß links oben (mit Schatten, gut lesbar auf hellem Gras)
        txt = str(shown)
        sh = self._num.render(txt, True, (20, 24, 36))
        img = self._num.render(txt, True, (255, 255, 255))
        s.blit(sh, (pad + 2, pad + 3))
        s.blit(img, (pad, pad))
        y = pad + img.get_height() - 2
        best = self.day_best if self.mode == "daily" else self.hs
        best = max(best, self.best)
        lab = t("cr.hud_top_daily", n=best) if self.mode == "daily" else t("cr.hud_top", n=best)
        self._shadow_text(s, self._small, lab, (pad, y), (255, 236, 150))
        # Münzen rechts oben
        coins = str(self.data["coins"])
        img = self._mid.render(coins, True, (255, 255, 255))
        cy = pad + img.get_height() // 2 + 2
        right = W - pad
        self._shadow_text(s, self._mid, coins, (right - img.get_width(), cy - img.get_height() // 2),
                          (255, 255, 255))
        r = max(7, img.get_height() // 2 - 1)
        self._coin_icon(s, right - img.get_width() - r - 8, cy, r)
        if self.run_coins:
            plus = "+%d" % self.run_coins
            pimg = self._small.render(plus, True, (255, 226, 110))
            self._shadow_text(s, self._small, plus, (right - pimg.get_width(), cy + r + 4),
                              (255, 226, 110))
        # Tagesstrecke-Kennzeichen
        if self.mode == "daily":
            tag = t("cr.daily_tag", date=seedrand.today_str())
            timg = self._tiny.render(tag, True, (255, 255, 255))
            rc = timg.get_rect(midtop=(W // 2, pad))
            box = rc.inflate(18, 8)
            ov = pygame.Surface(box.size, pygame.SRCALPHA)
            pygame.draw.rect(ov, (20, 24, 40, 150), ov.get_rect(), border_radius=box.h // 2)
            s.blit(ov, box.topleft)
            s.blit(timg, rc)
        # Starthinweis bis zum ersten Hüpfer
        if not self.started and self.death is None and not self.game_over:
            hint = t("cr.hint")
            img = self._small.render(hint, True, (255, 255, 255))
            rc = img.get_rect(midbottom=(W // 2, H - pad - 6))
            box = rc.inflate(24, 10)
            ov = pygame.Surface(box.size, pygame.SRCALPHA)
            a = int(120 + 60 * ui.pulse(2.5))
            pygame.draw.rect(ov, (20, 24, 40, a), ov.get_rect(), border_radius=box.h // 2)
            s.blit(ov, box.topleft)
            s.blit(img, rc)

    def _shadow_text(self, s, fnt, txt, pos, col):
        s.blit(fnt.render(txt, True, (20, 24, 36)), (pos[0] + 1, pos[1] + 2))
        s.blit(fnt.render(txt, True, col), pos)

    def _coin_icon(self, s, cx, cy, r):
        pygame.draw.circle(s, (176, 120, 20), (int(cx) + 1, int(cy) + 2), r)
        pygame.draw.circle(s, (246, 190, 44), (int(cx), int(cy)), r)
        pygame.draw.circle(s, (255, 226, 110), (int(cx), int(cy)), max(2, r - 3))
        pygame.draw.rect(s, (236, 170, 36), (int(cx) - max(1, r // 4), int(cy) - r // 2,
                                             max(2, r // 2), r))

    # ----- Game-Over-Panel -------------------------------------------------
    def _draw_over(self, s):
        if not hasattr(self, "over_rect"):
            self._layout_over()
        pr = self.over_rect
        key = (pr.size, ui.PANEL)
        if getattr(self, "_over_bg", (None,))[0] != key:
            ov = pygame.Surface(pr.size, pygame.SRCALPHA)
            pygame.draw.rect(ov, _rgba(ui.PANEL, 236), ov.get_rect(), border_radius=14)
            self._over_bg = (key, ov)
        s.blit(self._over_bg[1], pr.topleft)
        pygame.draw.rect(s, self.accent, pr, 2, border_radius=14)
        cx = pr.centerx
        maxw = pr.w - 28
        y = pr.y + 14
        cause = self.death["cause"] if self.death else "car"
        for fnt, txt, col in (
                (self._huge, t("common.game_over"), ui.RED),
                (self._tiny, t("cr.cause." + cause), ui.TEXT_DIM),
                (self.font, t("common.points", score=self.best), ui.TEXT)):
            img = self._fit(fnt, txt, col, maxw)
            s.blit(img, img.get_rect(midtop=(cx, y)))
            y += img.get_height() + 8
        if self.mode == "daily":
            best = max(self.best, self._daily_best())
            line = t("cr.best_daily", n=best)
        else:
            line = t("cr.best", n=max(self.best, self.hs))
        if self.record and self.best > 0:
            line += "  ·  " + t("cr.new_record")
        img = self._fit(self._small, line, ui.GOLD, maxw)
        s.blit(img, img.get_rect(midtop=(cx, y)))
        y += img.get_height() + 8
        line = t("cr.coins_run", n=self.run_coins, total=self.data["coins"])
        img = self._fit(self._small, line, ui.TEXT, maxw)
        rc = img.get_rect(midtop=(cx, y))
        rr = max(6, img.get_height() // 2 - 2)
        rc.x += rr + 3
        self._coin_icon(s, rc.x - rr - 6, rc.centery, rr)
        s.blit(img, rc)
        labels = {"again": t("cr.btn_again"), "chars": t("cr.tab_chars").capitalize(),
                  "setup": t("cr.btn_setup")}
        for key, rc in self.over_btns:
            self._btn(s, rc, labels[key], key == "again")
        hint = self._fit(self._tiny, t("cr.over_hint"), ui.TEXT_DIM, maxw)
        s.blit(hint, hint.get_rect(midbottom=(cx, pr.bottom - 10)))

    def _fit(self, fnt, txt, col, maxw):
        img = fnt.render(txt, True, col)
        if img.get_width() <= maxw:
            return img
        for smaller in (self._small, self._tiny):
            if smaller.get_height() < fnt.get_height():
                img = smaller.render(txt, True, col)
                if img.get_width() <= maxw:
                    return img
        kurz = txt
        while len(kurz) > 2 and self._tiny.size(kurz + "...")[0] > maxw:
            kurz = kurz[:-1]
        return self._tiny.render(kurz + "...", True, col)

    def _btn(self, s, rc, text, on):
        pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=8)
        pygame.draw.rect(s, self.accent if on else ui.BORDER, rc, 2 if on else 1,
                         border_radius=8)
        img = self._fit(self._small, text, ui.TEXT if on else ui.TEXT_DIM, rc.w - 10)
        s.blit(img, img.get_rect(center=rc.center))

    # ===================================================== Setup zeichnen
    def _draw_setup(self, s):
        W, H = self.width, self.height
        ui.draw_background(s, W, H)
        cx = W // 2
        title = self._huge.render(self.name.upper(), True, self.accent)
        s.blit(title, title.get_rect(center=(cx, int(H * 0.10))))
        sub = self._fit(self._small, t("cr.subtitle" if self.tab == "play" else "cr.chars_sub"),
                        ui.TEXT_DIM, W - 30)
        s.blit(sub, sub.get_rect(center=(cx, int(H * 0.165))))
        for i, rc in enumerate(self.tab_rects):
            self._btn(s, rc, t("cr.tab_" + TABS[i]), self.tab == TABS[i])
        if self.tab == "chars":
            self._draw_chars(s)
        else:
            self._draw_play_tab(s)

    def _stage(self):
        """Voxel-Maße für die Vorschau-Bühne im Setup (an die Kartenhöhe angepasst)."""
        tw = max(24, int(self.card_rect.h * 0.42))
        if self._stage_vox is None or self._stage_vox.tw != tw:
            self._stage_vox = cd.Voxel(tw, self.shadows)
        return self._stage_vox

    def _draw_play_tab(self, s):
        W, H = self.width, self.height
        cx = W // 2
        card = self.card_rect
        ui.draw_panel(s, card, radius=12)
        # Bühne: Grasblock mit der gewählten Figur, die sich umsieht und hüpft
        v = self._stage()
        block = [(0.0, 0.0, -0.55, 1.0, 1.0, 0.0, (112, 160, 64)),
                 (0.0, 0.0, 0.0, 1.0, 1.0, 0.16, cd.GRASS_A)]
        bs = getattr(self, "_block_cache", None)
        if bs is None or bs[0] is not v:
            bs = self._block_cache = (v, v.render(block, False))
        bsurf, bax, bay = bs[1]
        stage_w = int(1.0 * v.tw + v.sk)
        sx0 = card.x + 14 + (card.h - 16 - stage_w) // 2
        base_y = card.bottom - 10 - int(0.55 * v.zh)
        s.blit(bsurf, (sx0 - bax, base_y - bay))
        faces = ("down", "right", "down", "left")
        ph = (self.anim % 4.8) / 1.2
        facing = faces[int(ph)]
        frac = ph - int(ph)
        z = 0.16
        sy = 1.0
        if frac > 0.72:
            k = math.sin(math.pi * (frac - 0.72) / 0.28)
            z += 0.35 * k
            sy = 1.0 + 0.12 * k
        cid = self.data["selected"]
        if cid in cw.HOVER:
            z += 0.12 + 0.05 * math.sin(self.anim * 3.0)
        key = "%s:%s" % (cid, facing)
        surf, ax, ay = v.scaled(key, 1.0, sy)
        base, bax2, bay2 = v.sprite(key, False)
        fx = bax2 + 0.5 * v.tw + 0.5 * v.sk
        fy = bay2 - 0.5 * v.th
        ppx = sx0 + 0.5 * v.tw + 0.5 * v.sk
        ppy = base_y - 0.5 * v.th - z * v.zh
        s.blit(surf, (int(ppx - fx), int(ppy - fy * sy)))
        # Texte rechts neben der Bühne
        tx = card.x + card.h + 6
        tw = card.right - 14 - tx
        lines = []
        mode_key = "cr.mode.daily" if self.mode == "daily" else "cr.mode.endless"
        lines.append((self._mid, t(mode_key), self.accent))
        desc = t("cr.mode_desc.daily", date=seedrand.today_str()) if self.mode == "daily" \
            else t("cr.mode_desc.endless")
        lines.append((self._tiny, desc, ui.TEXT_DIM))
        if self.mode == "daily":
            lines.append((self._small, t("cr.best_daily", n=self._daily_best()), ui.GOLD))
        else:
            lines.append((self._small, t("cr.best", n=self._setup_best()), ui.GOLD))
        lines.append((self._small, t("cr.char_line", name=t("cr.char." + cid)), ui.TEXT))
        imgs = [self._fit(f, txt, col, tw) for f, txt, col in lines]
        total = sum(i.get_height() for i in imgs) + 6 * (len(imgs) - 1) \
            + self._mid.get_height() + 6
        y = card.y + max(8, (card.h - total) // 2)
        for img in imgs:
            s.blit(img, (tx, y))
            y += img.get_height() + 6
        coins = str(self.data["coins"])
        cimg = self._mid.render(coins, True, ui.TEXT)
        r = max(6, cimg.get_height() // 2 - 2)
        self._coin_icon(s, tx + r, y + cimg.get_height() // 2, r)
        s.blit(cimg, (tx + 2 * r + 8, y))
        # Schalter: Schatten · Tag/Nacht
        for rects, key, on in ((self.shadow_rects, "cr.lbl_shadows", self.shadows),
                               (self.night_rects, "cr.lbl_daynight", self.daynight)):
            img = self._fit(self._tiny, t(key), ui.TEXT_DIM, self.opt_w)
            mid = (rects[0].left + rects[-1].right) // 2
            s.blit(img, img.get_rect(midbottom=(mid, rects[0].top - 4)))
            for i, rc in enumerate(rects):
                self._btn(s, rc, t("common.on") if i == 0 else t("common.off"), on == (i == 0))
        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=10)
        pygame.draw.rect(s, self.accent, self.start_rect, 2, border_radius=10)
        st = self.font.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        hint = self._fit(self._tiny, t("cr.setup_hint"), ui.TEXT_FAINT, W - 20)
        s.blit(hint, hint.get_rect(center=(cx, H - 30)))
        ctrl = self._fit(self._tiny, t("cr.hint"), ui.mix(self.accent, ui.TEXT, 0.45), W - 20)
        s.blit(ctrl, ctrl.get_rect(center=(cx, H - 12)))

    def _draw_chars(self, s):
        W, H = self.width, self.height
        cx = W // 2
        r0 = self.grid_rects[0]
        tw = max(16, int(min(r0.w * 0.62, (r0.h - self._tiny.get_height()) * 0.5)))
        if self._card_vox is None or self._card_vox.tw != tw:
            self._card_vox = cd.Voxel(tw, False)
        v = self._card_vox
        coins = self.data["coins"]
        for i, rc in enumerate(self.grid_rects):
            cid = cw.CHAR_IDS[i]
            owned = cid in self.data["unlocked"]
            chosen = cid == self.data["selected"]
            cur = i == self.cursor
            pygame.draw.rect(s, ui.BTN_SEL if cur else ui.BTN, rc, border_radius=10)
            border = self.accent if (cur or chosen) else ui.BORDER
            pygame.draw.rect(s, border, rc, 2 if (cur or chosen) else 1, border_radius=10)
            name = self._fit(self._tiny, t("cr.char." + cid), ui.TEXT if owned else ui.TEXT_DIM,
                             rc.w - 6)
            ny = rc.bottom - name.get_height() - 4
            # Figur: die ausgewählte hüpft leicht
            face = "down"
            bob = 0.0
            if cur:
                bob = max(0.0, math.sin(self.anim * 6.0)) * 0.18
            surf, ax, ay = v.sprite("%s:%s" % (cid, face), False)
            if not owned:
                surf = surf.copy()
                surf.fill((70, 74, 90, 255), special_flags=pygame.BLEND_RGBA_MULT)
                surf.fill((40, 44, 60, 0), special_flags=pygame.BLEND_RGBA_ADD)
            fx = ax + 0.5 * v.tw + 0.5 * v.sk
            fy = ay - 0.5 * v.th
            px = rc.centerx - 0.25 * v.sk
            # Figur (ca. 1 Zelle hoch) mittig in den Platz über dem Namen
            room_top = rc.y + 4
            feet = room_top + (ny - room_top) // 2 + int(0.55 * v.zh)
            py = min(ny - int(0.3 * v.th), feet) - int(bob * v.zh)
            s.blit(surf, (int(px - fx), int(py - fy)))
            s.blit(name, name.get_rect(midbottom=(rc.centerx, rc.bottom - 4)))
            if chosen:
                pygame.draw.circle(s, self.accent, (rc.right - 11, rc.y + 11), 6)
                pygame.draw.lines(s, ui.BG_TOP, False, [(rc.right - 14, rc.y + 11),
                                                        (rc.right - 12, rc.y + 14),
                                                        (rc.right - 8, rc.y + 8)], 2)
            elif not owned:
                price = str(cw.PRICES[cid])
                col = ui.GOLD if coins >= cw.PRICES[cid] else ui.TEXT_FAINT
                pim = self._tiny.render(price, True, col)
                rr = max(4, pim.get_height() // 2 - 2)
                pr = pim.get_rect(topright=(rc.right - 6, rc.y + 5))
                s.blit(pim, pr)
                self._coin_icon(s, pr.x - rr - 3, pr.centery, rr)
        # Detailzeile
        d = self.detail_rect
        ui.draw_panel(s, d, radius=10, shadow=False)
        cid = cw.CHAR_IDS[self.cursor]
        owned = cid in self.data["unlocked"]
        coin_txt = str(coins)
        cimg = self._mid.render(coin_txt, True, ui.TEXT)
        r = max(6, cimg.get_height() // 2 - 2)
        self._coin_icon(s, d.x + 12 + r, d.centery, r)
        s.blit(cimg, cimg.get_rect(midleft=(d.x + 2 * r + 18, d.centery)))
        name_x = d.x + 2 * r + 30 + cimg.get_width()
        name = self._fit(self.font, t("cr.char." + cid), ui.TEXT,
                         self.buy_rect.x - name_x - 8)
        s.blit(name, name.get_rect(midleft=(name_x, d.centery)))
        if cid == self.data["selected"]:
            label, on = t("cr.chosen"), False
        elif owned:
            label, on = t("cr.choose"), True
        else:
            label = t("cr.buy", n=cw.PRICES[cid])
            on = coins >= cw.PRICES[cid]
        self._btn(s, self.buy_rect, label, on)
        hint = self._fit(self._tiny, t("cr.chars_hint"), ui.TEXT_FAINT, W - 20)
        s.blit(hint, hint.get_rect(center=(cx, H - 13)))
