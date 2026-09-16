# -*- coding: utf-8 -*-
"""
geodash.py
==========
Geometry Dash - der Rhythmus-Plattformer: Tippen, Halten, Fliegen, Sterben,
sofort weiter. Mit 8 eingebauten Leveln (Leicht bis Dämon), Übungsmodus,
prozeduralem Soundtrack und einem Level-Editor zum Bauen und Teilen.

Aufbau:
  geodash_core.py   Physik in Festkomma-Ganzzahlen (bitgleich mit dem Web)
  geodash_draw.py   Zeichnen von Welt, Objekten und Spieler
  geodash_music.py  der Soundtrack je Level
  geodash_edit.py   LEVELS-Reiter (eigene Level) und der Level-Editor
  geodash.py        dieses Modul: Level-Auswahl, Spielablauf, Fortschritt

Spielformen: Würfel (halten = springen), Schiff (halten = steigen), Ball
(tippen = Schwerkraft umdrehen), UFO (tippen = Flügelschlag), Welle (halten =
diagonal hoch). Orbs reagieren auf den Klick, Pads auf Berührung.

Zeit: Die Physik läuft in festen 240-Hz-Schritten. update() sammelt die Zeit
in einem Akkumulator; jede Eingabe bekommt beim Eintreffen einen Zeitstempel
und wird dem Schritt zugeordnet, in dem sie tatsächlich passiert ist - der
Absprung sitzt bei 15 FPS genauso wie bei 144 FPS.

Tod und Neustart laufen INTERN (nicht über game_over), damit update() weiter
läuft: Explosion, kurzer Moment, nächster Versuch - Musik von vorn.

Modi: Normal (zählt für Sterne, Münzen, Bestwert) und Übung (Checkpoints:
automatisch und mit [Z] setzen, [X] löschen; zählt nur als Übungs-Bestwert).

Punkte (Highscore) = Sterne gesamt: Sterne der geschafften eingebauten Level
plus deren eingesammelte Münzen - wächst nur.

mem.json, Section "geodash":
  best     {level_id: [normal_pct, übung_pct]}   (eigene Level: "ugc:<id>")
  coins    {level_id: bitmaske}                  (eingesammelte Münzen)
  attempts {level_id: versuche}
  jumps    Klicks/Sprünge gesamt
"""

import json
import math
import os
import random
import time

import pygame

import audio
import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent
from i18n import t

from . import geodash_core as core
from . import geodash_draw as gdraw
from . import geodash_music as music

SELECT, PLAY, COMPLETE, EDIT = "select", "play", "complete", "edit"
TABS = ("play", "levels")
DIFF_KEYS = ("easy", "normal", "hard", "harder", "insane", "demon")

LEVELS_FILE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                           "levels", "geodash.json")

DEATH_PAUSE = 0.85           # Sekunden Explosion bis zum nächsten Versuch
AUTO_CHECK_STEPS = 2 * core.HZ
VIEW_ROWS = 12.5             # sichtbare Reihen (bestimmt die Kachelgröße)
SOURCES_MOUSE = "mouse"


def load_builtin_levels():
    """Die eingebauten Level aus games/levels/geodash.json (normalisiert)."""
    try:
        with open(LEVELS_FILE, "r", encoding="utf-8") as f:
            data = json.load(f)
        levels = [core.normalize_level(d) for d in data.get("levels", [])
                  if isinstance(d, dict) and d.get("id")]
        if levels:
            return levels
    except (OSError, ValueError):
        pass
    # Notnagel: ein kleines Level, damit das Spiel nie ohne Level dasteht
    return [core.normalize_level({
        "id": "lama-launch", "name": "Lama Launch", "difficulty": 0, "stars": 1,
        "objects": [["spike", 20 + 10 * i, 0, 0] for i in range(8)],
        "length": 110})]


def total_coins_mask(mask):
    return bin(int(mask) & 7).count("1")


class GeometryDashGame(Game):
    name = "Geometry Dash"
    highscore_key = "geodash"
    MODES = [("normal", "gd.mode.normal"), ("practice", "gd.mode.practice")]
    # Rechte Maustaste im Editor: Leinwand ziehen bzw. Objekt löschen.
    wants_right_click = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        gs = self._gs()
        self.levels = load_builtin_levels()
        last = gs.get("last_level", 0)
        self.sel = max(0, min(len(self.levels) - 1, int(last or 0)))
        self.practice = (self.mode == "practice")
        self.music_on = bool(gs.get("music", True))
        self.show_bar = bool(gs.get("progress_bar", True))
        self.auto_cp = bool(gs.get("auto_checkpoints", True))
        self.setup_tab = "play"
        self.lists = None            # LEVELS-Reiter (geodash_edit.LevelList)
        self.editor = None
        self._test = None            # Test-Spielen aus dem Editor
        self.renderer = gdraw.Renderer()
        self.t = 0.0
        self.card_anim = 0.0
        self._load_progress()
        self._build_fonts()
        self._layout()
        self.state = SELECT
        # Laufende Partie
        self.cur = None
        self.lv = None
        self.run = None
        self._voices = None
        self._voices_key = None
        self.particles = []
        self.toast = ""
        self.toast_t = 0.0

    def _gs(self):
        if not isinstance(self.settings, dict):
            return {}
        gs = self.settings.get("geodash")
        return gs if isinstance(gs, dict) else {}

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("geodash", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _build_fonts(self):
        h = self.height
        self._huge = ui.font(max(24, h // 13), bold=True)
        self._mid = ui.font(max(16, h // 24), bold=True)
        self._small = ui.font(max(14, h // 32))
        self._tiny = ui.font(max(11, h // 42))
        self._attempt_font = ui.font(max(18, h // 17), bold=True)
        self._btn_font = ui.font(max(15, h // 30), bold=True)      # START

    def on_surface_changed(self):
        self._build_fonts()
        self._layout()
        if self.lists is not None:
            self.lists.layout()
        if self.editor is not None:
            self.editor.layout()

    @property
    def wants_escape(self):
        """ESC heißt im Editor, im LEVELS-Reiter, beim Test-Spielen und auf
        dem Ergebnis-Bildschirm "zurück" - sonst Pause."""
        return (self.state in (EDIT, COMPLETE) or self._test is not None
                or (self.state == SELECT and self.setup_tab == "levels"))

    def on_exit(self):
        audio.music_stop(self)
        self._save_progress()

    # ===================================================== Fortschritt
    def _load_progress(self):
        data = store.load_section("geodash")
        self.best = {}
        for k, v in (data.get("best") or {}).items():
            if isinstance(v, (list, tuple)) and len(v) >= 2:
                try:
                    self.best[str(k)] = [max(0, min(100, int(v[0]))),
                                         max(0, min(100, int(v[1])))]
                except (TypeError, ValueError):
                    continue
        self.coins = {}
        for k, v in (data.get("coins") or {}).items():
            try:
                self.coins[str(k)] = int(v) & 7
            except (TypeError, ValueError):
                continue
        self.attempts = {}
        for k, v in (data.get("attempts") or {}).items():
            try:
                self.attempts[str(k)] = max(0, int(v))
            except (TypeError, ValueError):
                continue
        try:
            self.jumps = max(0, int(data.get("jumps", 0)))
        except (TypeError, ValueError):
            self.jumps = 0
        self._dirty = False
        self.score = self.total_stars()

    def _save_progress(self):
        if not self._dirty:
            return
        store.save_section("geodash", {"best": self.best, "coins": self.coins,
                                       "attempts": self.attempts,
                                       "jumps": self.jumps})
        self._dirty = False

    def total_stars(self):
        """Sterne gesamt = Level-Sterne (geschafft) + Münzen der eingebauten Level."""
        n = 0
        for d in self.levels:
            lid = d["id"]
            if self.best.get(lid, [0, 0])[0] >= 100:
                n += int(d.get("stars", 0))
            n += total_coins_mask(self.coins.get(lid, 0))
        return n

    def max_stars(self):
        return sum(int(d.get("stars", 0)) + 3 for d in self.levels)

    def _best_key(self):
        if self.cur is None:
            return ""
        if self.cur_kind == "main":
            return self.cur["id"]
        return "ugc:" + str(self.cur.get("id", ""))

    # ===================================================== Level starten
    def start_level(self, level, kind="main", start_block=None, practice=None):
        """Startet ein Level. kind: main (eingebaut), ugc (eigenes), test."""
        self.cur = core.normalize_level(level)
        self.cur_kind = kind
        self.lv = core.Level(self.cur)
        gdraw.block_masks(self.lv)
        self.start_block = start_block
        if practice is not None:
            self.practice = practice
        self.checkpoints = []
        self.session_attempts = 0
        self.session_jumps = 0
        self.session_time = 0.0
        self.bpm = music.style_bpm(self.cur)
        self.state = PLAY
        self._new_attempt(first=True)
        self.play_sound("select")

    def _new_attempt(self, first=False, from_checkpoint=False):
        """Nächster Versuch: vom Start bzw. vom letzten Checkpoint."""
        if from_checkpoint and self.checkpoints:
            self.run = self.checkpoints[-1].copy()
        else:
            self.run = core.new_state(self.lv, self.start_block)
        self.session_attempts += 1
        if self.cur_kind != "test":
            key = self._best_key()
            self.attempts[key] = self.attempts.get(key, 0) + 1
            self._dirty = True
        self.acc = 0.0
        self.queue = []
        self.sources = set()
        self._last_update = time.perf_counter()
        self.dead_t = -1.0
        self.last_cp_step = self.run.step
        self.angle = 0.0
        self.trail = []
        px, py = self._player_blocks()
        self.cam_y = self._cam_target(py)
        self.run_time = 0.0
        self.attempt_label_x = px + 2.5
        self.attempt_label_y = py + 4.0
        self.new_best = None
        self.shock = None
        if not (from_checkpoint and self.practice and audio.music_playing(self)):
            self._start_music()

    def _start_music(self):
        audio.music_stop(self)
        self.music_t = 0.0
        if not self.music_on or self.cur is None or self.cur.get("music") == "none":
            return
        key = (self.cur.get("music"), self.bpm, self.cur.get("id", ""))
        if self._voices_key != key:
            try:
                self._voices = music.build_voices(key[0], key[1], key[2])
            except Exception:
                self._voices = None
            self._voices_key = key
        if self._voices:
            audio.music_play(self._voices, self, self.settings)

    def _player_blocks(self):
        s = self.run
        return s.x / float(core.B), s.y / float(core.B)

    # ===================================================== Eingabe
    def _is_jump_key(self, k):
        return (k in ("space", "Up", "w", "W", "Return", "KP_Enter")
                or self.is_action(k, "up") or self.is_action(k, "action"))

    def handle_event(self, event):
        if self.state == EDIT and self.editor is not None:
            self.editor.handle(event)
            return
        if self.state == SELECT:
            self._handle_select(event)
            return
        if self.state == COMPLETE:
            self._handle_complete(event)
            return
        if self.state != PLAY:
            return
        kind = event.kind
        now = time.perf_counter()
        if kind == InputEvent.KEYDOWN:
            k = event.key
            if getattr(event, "repeat", False):
                return
            if k == "Escape" and self._test is not None:
                self._back_to_editor()
                return
            if self._is_jump_key(k):
                self.queue.append((now, True, "k:" + str(k).lower()))
                return
            low = str(k).lower()
            if low == "z" and self.key_is_free(k) and self.practice:
                self._place_checkpoint()
            elif low == "x" and self.key_is_free(k) and self.practice:
                self._remove_checkpoint()
            elif low == "p" and self.key_is_free(k):
                self._toggle_practice_live()
            elif low == "r" and self.key_is_free(k):
                self._restart_now()
            elif k in ("BackSpace", "q", "Q") and self.key_is_free(k):
                if self._test is not None:
                    self._back_to_editor()
                else:
                    self._leave_to_select()
        elif kind == InputEvent.KEYUP:
            k = event.key
            if self._is_jump_key(k):
                self.queue.append((now, False, "k:" + str(k).lower()))
        elif kind == InputEvent.MOUSEDOWN and event.button == 1:
            self.queue.append((now, True, SOURCES_MOUSE))
        elif kind == InputEvent.MOUSEUP and event.button == 1:
            self.queue.append((now, False, SOURCES_MOUSE))

    def _toggle_practice_live(self):
        """[P] im Spiel: Übungsmodus an/aus.

        An: es geht ab hier weiter, der erste Checkpoint liegt an der
        aktuellen Stelle. Aus: neuer normaler Versuch vom Start.
        """
        if self.cur_kind == "test" and self._test and self._test.get("verify"):
            self._test["verify"] = False
        self.practice = not self.practice
        self.checkpoints = []
        self.play_sound("select")
        if self.practice:
            if self.run is not None and not self.run.dead and not self.run.won:
                if self.run.ground or self.run.mode != core.CUBE:
                    self.checkpoints.append(self.run.copy())
        else:
            self._new_attempt()

    def _place_checkpoint(self):
        s = self.run
        if s is None or s.dead or s.won:
            return
        self.checkpoints.append(s.copy())
        if len(self.checkpoints) > 200:
            del self.checkpoints[0]
        self.last_cp_step = s.step
        self.play_sound("powerup")

    def _remove_checkpoint(self):
        if self.checkpoints:
            self.checkpoints.pop()
            self.play_sound("click")

    def _restart_now(self):
        if self.run is None:
            return
        self._record_best(core.progress(self.run, self.lv))
        self._new_attempt(from_checkpoint=self.practice)

    def _leave_to_select(self):
        audio.music_stop(self)
        self._save_progress()
        self.state = SELECT
        self.cur = None
        self.run = None
        self.score = self.total_stars()
        self.play_sound("click")

    # ===================================================== Update
    def update(self, dt):
        self.t += dt
        if self.toast_t > 0:
            self.toast_t -= dt
            if self.toast_t <= 0:
                self.toast = ""
        self._update_particles(dt)
        if self.state == EDIT:
            if self.editor is not None:
                self.editor.update(dt)
            return
        if self.state == SELECT:
            self.card_anim = max(0.0, self.card_anim - dt * 4.0)
            if self.setup_tab == "levels" and self.lists is not None:
                self.lists.update(dt)
            return
        if self.state == COMPLETE:
            self.complete_t += dt
            self.music_t += dt
            return
        if self.state != PLAY or self.run is None:
            return
        self.music_t += dt
        now = time.perf_counter()
        self._last_update = now
        s = self.run
        if s.dead:
            self.queue = []
            self.dead_t += dt
            if self.dead_t >= DEATH_PAUSE:
                self._new_attempt(from_checkpoint=self.practice)
            return
        self.acc += dt
        n = int(self.acc * core.HZ + 1e-9)
        if n <= 0:
            return
        self.acc -= n / float(core.HZ)
        # Die n Schritte dieses Frames decken auf der Uhr die Zeit von t0 bis
        # "jetzt minus Rest im Akkumulator" ab - Schritt i beginnt bei
        # t0 + i/240. Jede Eingabe kommt in den Schritt, in dem sie passiert
        # ist; was erst danach kam, wartet auf den nächsten Frame. So sitzt
        # der Absprung bei jeder Bildrate auf demselben Schritt.
        t0 = now - max(0.0, self.acc) - n / float(core.HZ)
        events = self.queue
        self.queue = []
        ei = 0
        events.sort(key=lambda e: e[0])
        for i in range(n):
            pressed = False
            while ei < len(events):
                te, down, src = events[ei]
                if math.floor((te - t0) * core.HZ) > i:
                    break
                ei += 1
                if down:
                    if src not in self.sources:
                        self.sources.add(src)
                        pressed = True
                        self.jumps += 1
                        self.session_jumps += 1
                        self._dirty = True
                else:
                    self.sources.discard(src)
            held = bool(self.sources)
            core.step(s, self.lv, held or pressed, pressed)
            if s.dead or s.won:
                break
            if (self.practice and self.auto_cp and s.step - self.last_cp_step
                    >= AUTO_CHECK_STEPS and (s.ground or s.mode != core.CUBE)):
                self.checkpoints.append(s.copy())
                self.last_cp_step = s.step
        if s.dead or s.won:
            # übrig gebliebene Ereignisse (Loslassen nach dem Tod) nicht verlieren
            for te, down, src in events[ei:]:
                if down:
                    self.sources.add(src)
                else:
                    self.sources.discard(src)
        else:
            self.queue = events[ei:] + self.queue
        self.run_time += dt
        self.session_time += dt
        self._update_visuals(dt)
        if s.dead:
            self._on_death()
        elif s.won:
            self._on_win()

    def _update_visuals(self, dt):
        s = self.run
        px, py = self._player_blocks()
        held = bool(self.sources)
        mode = s.mode
        g = s.grav
        speed_bps = core.SPEEDS[s.speed] * core.HZ / float(core.B)
        if mode == core.CUBE:
            if s.ground:
                target = round(self.angle / 90.0) * 90.0
                self.angle += (target - self.angle) * min(1.0, dt * 22)
            else:
                self.angle += 420.0 * dt * g
        elif mode == core.BALL:
            self.angle += speed_bps / 0.5 * 57.3 * dt * g
        elif mode == core.SHIP:
            vy = s.vy * core.HZ / float(core.B)
            want = math.degrees(math.atan2(vy, speed_bps)) * 0.8
            self.angle += (want - self.angle) * min(1.0, dt * 14)
        elif mode == core.WAVE:
            self.angle = (45.0 if held else -45.0) * g
        else:
            self.angle *= max(0.0, 1.0 - dt * 10)
        # Spur
        if mode in (core.SHIP, core.WAVE, core.BALL, core.UFO):
            self.trail.append((px, py))
            limit = 60 if mode == core.WAVE else 14
            if len(self.trail) > limit:
                del self.trail[0]
        else:
            self.trail = []
        ty = self._cam_target(py)
        self.cam_y += (ty - self.cam_y) * min(1.0, dt * (5.0 if s.ceil else 3.5))

    def _cam_target(self, py):
        view_h = self.height / float(self._ts())
        s = self.run
        if s is not None and s.ceil > 0:
            c = s.ceil / float(core.B)
            return c / 2.0 - view_h / 2.0
        base = -view_h * 0.18
        if s is not None and s.grav < 0:
            return max(base, py - view_h * 0.62)
        return max(base, py - view_h * 0.45)

    def _ts(self):
        return max(8, int(self.height / VIEW_ROWS))

    # ----- Tod und Ziel -----------------------------------------------------
    def _on_death(self):
        self.dead_t = 0.0
        px, py = self._player_blocks()
        self._burst(px + 0.5, py + 0.5, gdraw.COL_P1, 34)
        self._burst(px + 0.5, py + 0.5, gdraw.COL_P2, 18)
        self.shock = (px + 0.5, py + 0.5, self.t)
        self.play_sound("explode")
        self.rumble(160)
        pct = core.progress(self.run, self.lv)
        if self._record_best(pct):
            self.new_best = (pct, self.t)
        if not self.practice:
            audio.music_stop(self)

    def _record_best(self, pct):
        """Bestwert des aktuellen Modus merken. True = neuer Rekord."""
        if self.cur_kind == "test" or self.start_block:
            return False
        key = self._best_key()
        best = self.best.setdefault(key, [0, 0])
        idx = 1 if self.practice else 0
        if pct > best[idx]:
            best[idx] = pct
            self._dirty = True
            self._save_progress()
            return True
        return False

    def _on_win(self):
        s = self.run
        self.state = COMPLETE
        self.complete_t = 0.0
        self.play_sound("win")
        self.rumble(220)
        px, py = self._player_blocks()
        for c in (gdraw.COL_P1, gdraw.COL_P2, gdraw.COL_COIN):
            self._burst(px + 0.5, py + 0.5, c, 26)
        ui.spawn_confetti(self.width, self.height)
        info = {"attempts": self.session_attempts, "jumps": self.session_jumps,
                "time": self.session_time, "coins": s.coins,
                "coin_count": self.lv.coin_count, "stars": 0, "new_coins": 0,
                "first": False, "practice": self.practice, "verified": False}
        new_record = self._record_best(100)
        if self.cur_kind == "main" and not self.practice:
            lid = self.cur["id"]
            old_mask = self.coins.get(lid, 0)
            new_mask = old_mask | s.coins
            info["new_coins"] = total_coins_mask(new_mask) - total_coins_mask(old_mask)
            if new_record:
                info["stars"] = int(self.cur.get("stars", 0))
                info["first"] = True
                self.ach_event("gd_first")
                if int(self.cur.get("difficulty", 0)) >= 5:
                    self.ach_event("gd_demon")
            if new_mask != old_mask:
                self.coins[lid] = new_mask
                self._dirty = True
            if new_mask == 7:
                self.ach_event("gd_coins")
            self.score = self.total_stars()
        elif self.cur_kind == "test" and self._test is not None:
            if self._test.get("verify") and not self.practice and not self.start_block:
                info["verified"] = True
                if self.editor is not None:
                    self.editor.mark_verified()
        self._save_progress()
        self.complete_info = info
        audio.music_set_volume(0.45)

    # ----- Partikel ------------------------------------------------------------
    def _burst(self, xb, yb, col, n):
        for _ in range(n):
            a = random.uniform(0, math.tau)
            sp = random.uniform(2.0, 9.0)
            self.particles.append([xb, yb, math.cos(a) * sp, math.sin(a) * sp,
                                   random.uniform(0.4, 0.9), col,
                                   random.uniform(0.08, 0.22)])

    def _update_particles(self, dt):
        if not self.particles:
            return
        keep = []
        for p in self.particles:
            p[4] -= dt
            if p[4] <= 0:
                continue
            p[0] += p[2] * dt
            p[1] += p[3] * dt
            p[3] -= 14.0 * dt
            p[2] *= (1.0 - 1.5 * dt)
            keep.append(p)
        self.particles = keep[-400:]

    # ===================================================== Level-Auswahl
    def _layout(self):
        w, h = self.width, self.height
        cx = w // 2
        self.renderer.resize(w, h, self._ts())
        # Höhen wachsen mit der Schrift (bei 1280x960 ist sie deutlich größer)
        sh = self._small.get_height()
        th = self._tiny.get_height()
        tab_h = max(22, min(30, h // 15), sh + 10)
        tab_w = min(max(150, w // 6), (w - 40) // 2)
        tab_y = int(h * 0.20)
        self.tab_rects = [pygame.Rect(cx - tab_w - 4, tab_y, tab_w, tab_h),
                          pygame.Rect(cx + 4, tab_y, tab_w, tab_h)]
        self.tab_bottom = tab_y + tab_h
        foot = th * 2 + 14
        bh = max(24, min(40, h // 14), sh + 14)
        gap = max(6, h // 60)
        opt_h = max(22, min(34, h // 16), th + 12)
        card_top = self.tab_bottom + gap + 2
        card_bottom = h - foot - opt_h - bh - 3 * gap - max(8, h // 50)
        arrow = max(28, min(52, w // 14))
        card_w = min(w - 2 * arrow - 44, max(620, int(w * 0.6)))
        self.card_rect = pygame.Rect(cx - card_w // 2, card_top, card_w,
                                     max(90, card_bottom - card_top))
        self.arrow_rects = [
            pygame.Rect(self.card_rect.x - arrow - 10, self.card_rect.centery - arrow // 2,
                        arrow, arrow),
            pygame.Rect(self.card_rect.right + 10, self.card_rect.centery - arrow // 2,
                        arrow, arrow)]
        self.dots_y = self.card_rect.bottom + max(6, h // 70)
        y = self.dots_y + max(6, h // 70)
        bw = min(w - 40, max(380, int(w * 0.62)))
        x0 = cx - bw // 2
        mode_w = int(bw * 0.29)
        self.mode_rects = [pygame.Rect(x0, y, mode_w, bh),
                           pygame.Rect(x0 + mode_w + gap, y, mode_w, bh)]
        sx = x0 + 2 * mode_w + 2 * gap + gap
        self.start_rect = pygame.Rect(sx, y, x0 + bw - sx, bh)
        y += bh + gap
        ow = (bw - 2 * gap) // 3
        self.opt_rects = {key: pygame.Rect(x0 + i * (ow + gap), y, ow, opt_h)
                          for i, key in enumerate(("music", "bar", "auto"))}
        self.opt_bottom = y + opt_h

    def _set_tab(self, tab):
        if tab not in TABS or tab == self.setup_tab:
            return
        self.setup_tab = tab
        if tab == "levels":
            from . import geodash_edit as edit
            if self.lists is None:
                self.lists = edit.LevelList(self)
            else:
                self.lists.layout()
                self.lists.reload()
        self.play_sound("click")

    def _handle_select(self, event):
        if event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.tab_rects):
                if rc.collidepoint(event.pos):
                    self._set_tab(TABS[i])
                    return
        elif event.kind == InputEvent.KEYDOWN and event.key in ("Tab", "ISO_Left_Tab"):
            self._set_tab("levels" if self.setup_tab == "play" else "play")
            return
        if self.setup_tab == "levels":
            if event.kind == InputEvent.KEYDOWN and event.key == "Escape":
                self._set_tab("play")
                return
            if self.lists is None:
                from . import geodash_edit as edit
                self.lists = edit.LevelList(self)
            self.lists.handle(event)
            return
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k == "Left" or self.is_action(k, "left"):
                self._step_level(-1)
            elif k == "Right" or self.is_action(k, "right"):
                self._step_level(1)
            elif k in ("1", "2", "3", "4", "5", "6", "7", "8", "9"):
                i = int(k) - 1
                if i < len(self.levels):
                    self._step_level(i - self.sel)
            elif k in ("Return", "space", "KP_Enter"):
                self._play_selected()
            elif k in ("p", "P"):
                self.practice = not self.practice
                self.play_sound("select")
            elif k in ("m", "M"):
                self._toggle_option("music")
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            pos = event.pos
            if self.arrow_rects[0].collidepoint(pos):
                self._step_level(-1)
            elif self.arrow_rects[1].collidepoint(pos):
                self._step_level(1)
            elif self.card_rect.collidepoint(pos) or self.start_rect.collidepoint(pos):
                self._play_selected()
            elif self.mode_rects[0].collidepoint(pos):
                if self.practice:
                    self.practice = False
                    self.play_sound("select")
            elif self.mode_rects[1].collidepoint(pos):
                if not self.practice:
                    self.practice = True
                    self.play_sound("select")
            else:
                for key, rc in self.opt_rects.items():
                    if rc.collidepoint(pos):
                        self._toggle_option(key)
                        return
                for i, rc in enumerate(self._dot_rects()):
                    if rc.collidepoint(pos):
                        self._step_level(i - self.sel)
                        return
        elif event.kind == InputEvent.WHEEL:
            self._step_level(-1 if event.delta > 0 else 1)

    def _toggle_option(self, key):
        if key == "music":
            self.music_on = not self.music_on
            self._save_setting("music", self.music_on)
            if not self.music_on:
                audio.music_stop(self)
        elif key == "bar":
            self.show_bar = not self.show_bar
            self._save_setting("progress_bar", self.show_bar)
        else:
            self.auto_cp = not self.auto_cp
            self._save_setting("auto_checkpoints", self.auto_cp)
        self.play_sound("select")

    def _step_level(self, d):
        if not d:
            return
        self.sel = (self.sel + d) % len(self.levels)
        self.card_anim = 1.0
        self.card_dir = 1 if d > 0 else -1
        self._save_setting("last_level", self.sel)
        self.play_sound("move")

    def _play_selected(self):
        self._test = None
        self.start_level(self.levels[self.sel], "main")

    def _dot_rects(self):
        n = len(self.levels)
        r = max(4, self.height // 110)
        gap = r * 4
        x0 = self.width // 2 - (n - 1) * gap // 2
        return [pygame.Rect(x0 + i * gap - r - 2, self.dots_y - r - 2, 2 * r + 4, 2 * r + 4)
                for i in range(n)]

    # ===================================================== Ergebnis
    def _complete_buttons(self):
        w, h = self.width, self.height
        pw = min(w - 30, max(460, int(w * 0.56)))     # wächst mit der Schrift
        sh = self._small.get_height()
        bh = max(26, min(38, h // 15), sh + 12)
        # Höhe aus dem Inhalt: Titel, Name, 3 Werte, Münzen, Extra-Zeile, Knöpfe
        need = (12 + self._huge.get_height() + 4 + sh + 8 + 3 * (sh + 2) + 6
                + 2 * max(8, h // 40) + 8 + sh + 30 + bh + 14)
        ph = min(h - 30, max(220, need))
        panel = pygame.Rect((w - pw) // 2, (h - ph) // 2, pw, ph)
        keys = ["again", "select"]
        if self.cur_kind == "main" and self.levels and self.sel < len(self.levels) - 1:
            keys.append("next")
        if self.cur_kind == "test":
            keys = ["again", "editor"]
        gap = 8
        bw = (pw - 30 - gap * (len(keys) - 1)) // len(keys)
        y = panel.bottom - bh - 14
        rects = [(k, pygame.Rect(panel.x + 15 + i * (bw + gap), y, bw, bh))
                 for i, k in enumerate(keys)]
        return panel, rects

    def _handle_complete(self, event):
        if self.complete_t < 0.35:
            return
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("Return", "space", "r", "R", "KP_Enter"):
                self._complete_action("again")
            elif k in ("Escape", "BackSpace"):
                self._complete_action("editor" if self.cur_kind == "test" else "select")
            elif k in ("n", "N", "Right"):
                self._complete_action("next")
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            _panel, rects = self._complete_buttons()
            for key, rc in rects:
                if rc.collidepoint(event.pos):
                    self._complete_action(key)
                    return

    def _complete_action(self, key):
        audio.music_set_volume(1.0)
        if key == "again":
            self.start_level(self.cur, self.cur_kind, self.start_block)
        elif key == "next":
            if self.cur_kind == "main" and self.sel < len(self.levels) - 1:
                self.sel += 1
                self._save_setting("last_level", self.sel)
                self.start_level(self.levels[self.sel], "main")
        elif key == "editor":
            self._back_to_editor()
        else:
            if self.cur_kind == "ugc":
                self.setup_tab = "levels"
                if self.lists is not None:
                    self.lists.reload()
            self._leave_to_select()

    # ===================================================== Eigene Level
    # Der LEVELS-Reiter und der Editor stecken in geodash_edit.py; hier stehen
    # nur die Übergänge, die beide brauchen.

    @property
    def grid_snap(self):
        return bool(self._gs().get("grid", True))

    def set_grid_snap(self, on):
        self._save_setting("grid", bool(on))

    def ugc_new_level(self):
        from . import geodash_edit as edit
        self.ugc_edit(edit.new_level())

    def ugc_edit(self, m):
        from . import geodash_edit as edit
        audio.music_stop(self)
        self.editor = edit.LevelEditor(self, m)
        self.state = EDIT
        self.play_sound("click")

    def ugc_close_editor(self):
        self.editor = None
        self._test = None
        self.state = SELECT
        self.setup_tab = "levels"
        if self.lists is None:
            from . import geodash_edit as edit
            self.lists = edit.LevelList(self)
        self.lists.reload()
        self.play_sound("click")

    def ugc_play(self, m):
        self._test = None
        self.start_level(m, "ugc")

    def ugc_test(self, level, start_block=None):
        """Level aus dem Editor ausprobieren - ESC führt zurück ans Bauen.

        Nur ein Lauf vom Start im Normalmodus kann das Level verifizieren.
        """
        self._test = {"verify": not start_block}
        self.start_level(level, "test", start_block, practice=False)

    def _back_to_editor(self):
        audio.music_stop(self)
        audio.music_set_volume(1.0)
        self._test = None
        self.run = None
        if self.editor is None:
            self.ugc_close_editor()
            return
        self.state = EDIT
        self.play_sound("click")

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        if self.state == EDIT and self.editor is not None:
            ui.draw_background(s, self.width, self.height)
            self.editor.draw(s)
            return
        if self.state == SELECT:
            self._draw_select(s)
            return
        self._draw_world(s)
        self._draw_hud(s)
        if self.state == COMPLETE:
            self._draw_complete(s)

    # ----- Welt -------------------------------------------------------------
    def _draw_world(self, s, run=None):
        run = run or self.run
        r = self.renderer
        r.resize(self.width, self.height, self._ts())
        ts = r.ts
        px, py = run.x / float(core.B), run.y / float(core.B)
        view_w = self.width / float(ts)
        cam_x = px - view_w * 0.3
        cam_y = self.cam_y
        lv = self.lv
        bg = gdraw.color_at(lv, px, 0)
        ground = gdraw.color_at(lv, px, 1)
        pulse = music.beat_pulse(self.music_t, self.bpm) if self.bpm else 0.0
        r.draw_backdrop(s, bg, cam_x, cam_y, pulse)
        r.draw_objects(s, lv, cam_x, cam_y, self.t, pulse, run.used)
        r.draw_ground(s, lv, ground, cam_x, cam_y, pulse)
        if run.ceil > 0:
            r.draw_ceiling(s, ground, run.ceil / float(core.B), cam_y, pulse)
        # Versuchszähler in der Welt (scrollt mit)
        if self.cur_kind == "test":
            n = self.session_attempts
        else:
            n = self.attempts.get(self._best_key(), 1)
        ax = r.sx(self.attempt_label_x, cam_x)
        ay = r.sy(self.attempt_label_y, cam_y)
        if -self.width < ax < self.width + 50:
            label = t("gd.attempt", n=n)
            img = self._attempt_font.render(label, True, (255, 255, 255))
            sh = self._attempt_font.render(label, True, (0, 0, 0))
            s.blit(sh, (ax + 3, ay + 3))
            s.blit(img, (ax, ay))
        # Checkpoints (Übung)
        if self.practice:
            for cp in self.checkpoints[-30:]:
                cx = r.sx(cp.x / float(core.B) + 0.5, cam_x)
                cy = r.sy(cp.y / float(core.B) + 0.5, cam_y)
                if -ts < cx < self.width + ts:
                    d = ts * 0.32
                    pts = [(cx, cy - d), (cx + d * 0.7, cy), (cx, cy + d), (cx - d * 0.7, cy)]
                    pygame.draw.polygon(s, gdraw.COL_CHECK, pts)
                    pygame.draw.polygon(s, (10, 40, 20), pts, max(1, ts // 16))
        # Partikel
        for p in self.particles:
            x = r.sx(p[0], cam_x)
            y = r.sy(p[1], cam_y)
            size = max(2, int(ts * p[6] * min(1.0, p[4] * 2)))
            pygame.draw.rect(s, p[5], (int(x) - size // 2, int(y) - size // 2, size, size))
        if getattr(self, "shock", None) is not None:
            age = self.t - self.shock[2]
            if age < 0.45:
                f = age / 0.45
                rad = int(ts * (0.4 + 2.6 * f))
                ring = pygame.Surface((2 * rad + 4, 2 * rad + 4), pygame.SRCALPHA)
                pygame.draw.circle(ring, (255, 255, 255, int(220 * (1 - f))),
                                   (rad + 2, rad + 2), rad, max(2, int(ts * 0.18 * (1 - f))))
                s.blit(ring, (r.sx(self.shock[0], cam_x) - rad - 2,
                              r.sy(self.shock[1], cam_y) - rad - 2))
        if not run.dead:
            r.draw_player(s, px, py, run.mode, run.grav, self.angle, cam_x, cam_y, self.t,
                          self.trail, bool(self.sources))

    def _draw_hud(self, s):
        w, h = self.width, self.height
        run = self.run
        pct = core.progress(run, self.lv) if run is not None else 0
        if self.show_bar:
            bw = int(w * 0.42)
            bh = max(8, h // 64)
            x = w // 2 - bw // 2
            y = max(6, h // 60)
            pygame.draw.rect(s, (0, 0, 0), (x - 2, y - 2, bw + 4, bh + 4), border_radius=bh)
            pygame.draw.rect(s, (40, 40, 60), (x, y, bw, bh), border_radius=bh)
            fill = int(bw * pct / 100.0)
            if fill > 0:
                pygame.draw.rect(s, gdraw.COL_P1, (x, y, fill, bh), border_radius=bh)
            img = self._small.render("%d%%" % pct, True, (255, 255, 255))
            sh = self._small.render("%d%%" % pct, True, (0, 0, 0))
            s.blit(sh, sh.get_rect(midleft=(x + bw + 11, y + bh // 2 + 1)))
            s.blit(img, img.get_rect(midleft=(x + bw + 10, y + bh // 2)))
        # Münzen dieses Versuchs (oben rechts)
        if run is not None and self.lv.coin_count:
            r = max(6, h // 50)
            for i in range(self.lv.coin_count):
                cx = w - 14 - r - i * (2 * r + 6)
                got = bool(run.coins & (1 << (self.lv.coin_count - 1 - i)))
                if got:
                    gdraw.draw_coin(s, cx, 14 + r, r, 0.0)
                else:
                    pygame.draw.circle(s, (0, 0, 0), (cx, 14 + r), r, 2)
        # Modus-Hinweise unten
        lines = []
        if self.practice:
            lines.append((t("gd.practice_hud"), gdraw.COL_CHECK))
        if self.cur_kind == "test":
            lines.append((t("gd.test_hud"), (255, 220, 120)))
        yb = h - 8
        for text, col in reversed(lines):
            img = self._tiny.render(text, True, col)
            box = img.get_rect(midbottom=(w // 2, yb))
            bg = pygame.Surface((box.w + 16, box.h + 6), pygame.SRCALPHA)
            bg.fill((0, 0, 0, 140))
            s.blit(bg, (box.x - 8, box.y - 3))
            s.blit(img, box)
            yb = box.y - 8
        # Neuer Bestwert
        if self.new_best is not None and self.state == PLAY:
            age = self.t - self.new_best[1]
            if age < 1.6:
                txt = t("gd.new_best", pct=self.new_best[0])
                sc = 1.0 + 0.25 * max(0.0, 0.25 - age) * 4
                f = ui.font(int(max(18, h // 14) * sc), bold=True)
                img = f.render(txt, True, gdraw.COL_P1)
                sh = f.render(txt, True, (0, 0, 0))
                pos = img.get_rect(center=(w // 2, int(h * 0.32)))
                s.blit(sh, pos.move(3, 3))
                s.blit(img, pos)

    def _draw_complete(self, s):
        w, h = self.width, self.height
        info = self.complete_info
        a = min(1.0, self.complete_t * 3.0)
        veil = pygame.Surface((w, h), pygame.SRCALPHA)
        veil.fill((0, 0, 0, int(140 * a)))
        s.blit(veil, (0, 0))
        panel, rects = self._complete_buttons()
        panel = panel.move(0, int((1.0 - a) * 40))
        ui.draw_panel(s, panel, accent_top=self.accent)
        cx = panel.centerx
        key = "gd.complete_practice" if info["practice"] else "gd.complete"
        title = self._huge.render(t(key), True, self.accent)
        if title.get_width() > panel.w - 20:
            title = self._mid.render(t(key), True, self.accent)
        y = panel.y + 12
        s.blit(title, title.get_rect(midtop=(cx, y)))
        y += title.get_height() + 4
        name = self._small.render(self.cur.get("name", ""), True, ui.TEXT)
        s.blit(name, name.get_rect(midtop=(cx, y)))
        y += name.get_height() + 8
        mins, secs = divmod(int(info["time"]), 60)
        rows = [(t("gd.stat_attempts"), str(info["attempts"])),
                (t("gd.stat_jumps"), str(info["jumps"])),
                (t("gd.stat_time"), "%d:%02d" % (mins, secs))]
        lw = panel.w - 60
        for label, val in rows:
            li = self._small.render(label, True, ui.TEXT_DIM)
            vi = self._small.render(val, True, ui.TEXT)
            s.blit(li, (panel.x + 30, y))
            s.blit(vi, vi.get_rect(topright=(panel.x + 30 + lw, y)))
            y += li.get_height() + 2
        y += 6
        # Münzen + Sterne
        rr = max(8, h // 40)
        n = info["coin_count"]
        if n:
            total_w = n * (2 * rr + 10)
            for i in range(n):
                ccx = cx - total_w // 2 + rr + i * (2 * rr + 10)
                if info["coins"] & (1 << i):
                    gdraw.draw_coin(s, ccx, y + rr, rr, self.t)
                else:
                    pygame.draw.circle(s, ui.TEXT_FAINT, (ccx, y + rr), rr, 2)
            y += 2 * rr + 8
        extra = []
        if info["stars"]:
            extra.append(t("gd.stars_won", n=info["stars"]))
        if info["new_coins"]:
            extra.append(t("gd.coins_won", n=info["new_coins"]))
        if info["verified"]:
            extra.append(t("gd.verified_now"))
        if info["practice"]:
            extra.append(t("gd.practice_note"))
        if extra:
            img = self._small.render("  ·  ".join(extra), True, ui.GOLD)
            if img.get_width() > panel.w - 20:
                img = self._tiny.render("  ·  ".join(extra), True, ui.GOLD)
            s.blit(img, img.get_rect(midtop=(cx, y)))
        labels = {"again": t("gd.btn_again"), "select": t("gd.btn_levels"),
                  "next": t("gd.btn_next"), "editor": t("gd.btn_editor")}
        for key, rc in rects:
            rc = rc.move(0, int((1.0 - a) * 40))
            self._btn(s, rc, labels[key], key == "again")

    # ----- Auswahl -------------------------------------------------------------
    def _draw_select(self, s):
        w, h = self.width, self.height
        ui.draw_background(s, w, h)
        cx = w // 2
        title = self._huge.render(t("gd.title"), True, self.accent)
        s.blit(title, title.get_rect(center=(cx, int(h * 0.085))))
        sub_key = "gd.subtitle" if self.setup_tab == "play" else "gd.ugc.subtitle"
        sub = self._small.render(t(sub_key), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(h * 0.155))))
        for i, rc in enumerate(self.tab_rects):
            self._btn(s, rc, t("gd.tab_" + TABS[i]), self.setup_tab == TABS[i])
        if self.setup_tab == "levels":
            if self.lists is None:
                from . import geodash_edit as edit
                self.lists = edit.LevelList(self)
            self.lists.draw(s)
            return
        self._draw_card(s)
        for i, rc in enumerate(self.arrow_rects):
            pygame.draw.rect(s, ui.BTN, rc, border_radius=rc.w // 4)
            pygame.draw.rect(s, ui.BORDER, rc, 1, border_radius=rc.w // 4)
            d = -1 if i == 0 else 1
            m = rc.w // 4
            pts = [(rc.centerx - d * m * 0.6, rc.centery - m), (rc.centerx + d * m * 0.8, rc.centery),
                   (rc.centerx - d * m * 0.6, rc.centery + m)]
            pygame.draw.polygon(s, self.accent, pts)
        for i, rc in enumerate(self._dot_rects()):
            on = i == self.sel
            col = self.accent if on else ui.BORDER_LIGHT
            pygame.draw.circle(s, col, rc.center, rc.w // 2 - 2 if on else rc.w // 2 - 4)
        self._btn(s, self.mode_rects[0], t("gd.mode.normal"), not self.practice)
        self._btn(s, self.mode_rects[1], t("gd.mode.practice"), self.practice)
        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=9)
        pygame.draw.rect(s, self.accent, self.start_rect, 2, border_radius=9)
        st = self._btn_font.render(t("common.start"), True, ui.TEXT)
        if st.get_width() > self.start_rect.w - 10:
            st = self._small.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        for key, on in (("music", self.music_on), ("bar", self.show_bar), ("auto", self.auto_cp)):
            label = t("gd.opt_" + key) + ": " + (t("common.on") if on else t("common.off"))
            self._btn(s, self.opt_rects[key], label, on, small=True)
        # Fußzeile: Sterne gesamt + Tasten
        stars = self.total_stars()
        line = t("gd.total_stars", n=stars, max=self.max_stars())
        img = self._tiny.render(line, True, ui.GOLD)
        hint_h = self._tiny.get_height()
        yline = h - 6 - hint_h - self._tiny.get_height() // 2 - 2
        r = max(5, self._tiny.get_height() // 2 - 1)
        tw = img.get_width() + 2 * r + 6
        gdraw.draw_star(s, cx - tw // 2 + r, yline, r, ui.GOLD)
        s.blit(img, img.get_rect(midleft=(cx - tw // 2 + 2 * r + 6, yline)))
        hint = self._tiny.render(t("gd.select_hint"), True, ui.TEXT_DIM)
        if hint.get_width() > w - 16:
            hint = ui.font(max(10, h // 48)).render(t("gd.select_hint"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(midbottom=(cx, h - 4)))

    def card_parts(self, rect):
        """Aufteilung der Level-Karte (auch für den Layout-Test).

        Kopf: Gesicht + Schwierigkeit links, Name/Sterne/Münzen/Versuche
        rechts daneben. Unten die Bestwert-Balken (nebeneinander, wenn die
        Karte breit genug ist), dazwischen das Vorschaubild.
        """
        pad = rect.x + 8 + max(6, rect.w // 60) + 12
        head_h = max(self._mid.get_height() + self._small.get_height() + self._tiny.get_height() + 10,
                     2 * max(14, min(rect.h // 7, rect.w // 14, 44)) + self._tiny.get_height() + 6)
        fr = max(12, min(rect.h // 7, rect.w // 14, 44, (head_h - self._tiny.get_height() - 6) // 2))
        lab_h = self._tiny.get_height()
        bar_h = max(10, min(20, rect.h // 12))
        side = rect.w >= 420
        block = lab_h + 2 + bar_h
        bars_top = rect.bottom - 12 - (block if side else 2 * block + 6)
        bars = []
        bw_all = rect.right - 14 - pad
        for i in range(2):
            if side:
                bw = (bw_all - 16) // 2
                bars.append(pygame.Rect(pad + i * (bw + 16), bars_top + lab_h + 2, bw, bar_h))
            else:
                bars.append(pygame.Rect(pad, bars_top + i * (block + 6) + lab_h + 2, bw_all, bar_h))
        head_top = rect.y + 12
        thumb = pygame.Rect(pad, head_top + head_h + 8, bw_all, bars_top - 8 - (head_top + head_h + 8))
        return {"pad": pad, "fr": fr, "head_top": head_top, "head_h": head_h, "bars": bars,
                "thumb": thumb, "right": rect.right - 14}

    def _draw_card(self, s):
        d = self.levels[self.sel]
        lid = d["id"]
        rect = self.card_rect.move(int(self.card_anim * 30 * getattr(self, "card_dir", 1)), 0)
        diff = int(d.get("difficulty", 0))
        col = gdraw.DIFF_COLORS[diff]
        ui.draw_panel(s, rect, accent_top=col)
        # Levelfarbe als Streifen links
        bg = tuple(d.get("bg", (40, 110, 255)))
        strip = pygame.Rect(rect.x + 8, rect.y + 8, max(6, rect.w // 60), rect.h - 16)
        pygame.draw.rect(s, bg, strip, border_radius=strip.w // 2)
        P = self.card_parts(rect)
        pad, fr, top = P["pad"], P["fr"], P["head_top"]
        face_c = (pad + fr, top + fr)
        gdraw.draw_face(s, face_c[0], face_c[1], fr, diff, self.t)
        dl = self._tiny.render(t("gd.diff." + DIFF_KEYS[diff]), True, col)
        if dl.get_width() > 2 * fr + 30:
            dl = ui.font(max(9, self.height // 52)).render(t("gd.diff." + DIFF_KEYS[diff]), True, col)
        s.blit(dl, dl.get_rect(midtop=(face_c[0], face_c[1] + fr + 3)))
        tx = pad + 2 * fr + max(18, dl.get_width() // 2 - fr + 10)
        right = P["right"]
        name = d.get("name", "")
        img = self._mid.render(name, True, ui.TEXT)
        if img.get_width() > right - tx:
            img = self._small.render(name, True, ui.TEXT)
        s.blit(img, (tx, top - 2))
        ny = top - 2 + img.get_height() + 3
        stars = int(d.get("stars", 0))
        sr = max(6, self._small.get_height() // 2 - 1)
        done = self.best.get(lid, [0, 0])[0] >= 100
        gdraw.draw_star(s, tx + sr, ny + sr, sr, ui.GOLD if done else ui.TEXT_FAINT)
        st = self._small.render("%d" % stars, True, ui.GOLD if done else ui.TEXT_DIM)
        s.blit(st, (tx + 2 * sr + 5, ny + sr - st.get_height() // 2))
        mask = self.coins.get(lid, 0)
        cx0 = tx + 2 * sr + 5 + st.get_width() + 16
        for i in range(3):
            ccx = cx0 + sr + i * (2 * sr + 6)
            if mask & (1 << i):
                gdraw.draw_coin(s, ccx, ny + sr, sr, self.t + i)
            else:
                pygame.draw.circle(s, ui.TEXT_FAINT, (ccx, ny + sr), sr, 2)
        att = self._tiny.render(t("gd.attempts_total", n=self.attempts.get(lid, 0)), True,
                                ui.TEXT_FAINT)
        s.blit(att, (tx, ny + 2 * sr + 5))
        thumb = P["thumb"]
        if thumb.h >= 26 and thumb.w >= 60:
            s.blit(self._thumb(d, thumb.w, thumb.h), thumb.topleft)
            pygame.draw.rect(s, ui.BORDER, thumb, 1, border_radius=6)
        best = self.best.get(lid, [0, 0])
        for i, (key, bcol) in enumerate((("gd.mode.normal", gdraw.COL_P1),
                                         ("gd.mode.practice", gdraw.COL_CHECK))):
            bar = P["bars"][i]
            lab = self._tiny.render(t("gd.best_label", mode=t(key)), True, ui.TEXT_DIM)
            s.blit(lab, (bar.x, bar.y - lab.get_height() - 2))
            pct = self._tiny.render("%d%%" % best[i], True, ui.TEXT)
            s.blit(pct, pct.get_rect(bottomright=(bar.right, bar.y - 2)))
            pygame.draw.rect(s, ui.BTN, bar, border_radius=bar.h // 2)
            fill = int(bar.w * best[i] / 100.0)
            if fill > 0:
                pygame.draw.rect(s, bcol, (bar.x, bar.y, max(bar.h, fill), bar.h),
                                 border_radius=bar.h // 2)

    def _thumb(self, d, w, h):
        """Vorschau der ersten Blöcke eines Levels (gecacht je Größe)."""
        key = (d["id"], w, h)
        cache = getattr(self, "_thumbs", None)
        if cache is None or len(cache) > 24:
            cache = self._thumbs = {}
        img = cache.get(key)
        if img is None:
            lv = core.Level(d)
            gdraw.block_masks(lv)
            ren = gdraw.Renderer()
            rows = 7.5
            ren.resize(w, h, max(8, int(h / rows)))
            img = pygame.Surface((w, h))
            cam_x, cam_y = 12.0, -1.2
            ren.draw_backdrop(img, gdraw.color_at(lv, cam_x, 0), cam_x, cam_y)
            ren.draw_objects(img, lv, cam_x, cam_y, 0.2, 0.0, ())
            ren.draw_ground(img, lv, gdraw.color_at(lv, cam_x, 1), cam_x, cam_y)
            ren.draw_player(img, cam_x + 3, 0.0, core.CUBE, 1, 0.0, cam_x, cam_y, 0.0)
            mask = pygame.Surface((w, h), pygame.SRCALPHA)
            pygame.draw.rect(mask, (255, 255, 255, 255), (0, 0, w, h), border_radius=6)
            out = pygame.Surface((w, h), pygame.SRCALPHA)
            out.blit(img, (0, 0))
            out.blit(mask, (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
            img = out
            cache[key] = img
        return img

    def _btn(self, s, rc, text, on, small=False):
        pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=8)
        pygame.draw.rect(s, self.accent if on else ui.BORDER, rc, 2 if on else 1, border_radius=8)
        col = ui.TEXT if on else ui.TEXT_DIM
        im = (self._tiny if small else self._small).render(text, True, col)
        if im.get_width() > rc.w - 12:
            im = self._tiny.render(text, True, col)
        if im.get_width() > rc.w - 8:
            kurz = text
            while len(kurz) > 2 and self._tiny.size(kurz + "...")[0] > rc.w - 8:
                kurz = kurz[:-1]
            im = self._tiny.render(kurz + "...", True, col)
        s.blit(im, im.get_rect(center=rc.center))
