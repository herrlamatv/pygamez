# -*- coding: utf-8 -*-
"""
simon.py
========
Simon / Senso - das klassische Merkspiel mit leuchtenden Feldern.

Die Felder leuchten in einer wachsenden Reihenfolge auf; diese muss exakt
nachgetippt werden. Jede Runde kommt ein Feld hinzu.

Spielmodi:
  - Klassisch : Reihenfolge exakt nachtippen.
  - Speed     : Die Wiedergabe wird von Runde zu Runde schneller.
  - Reverse   : Die Reihenfolge muss RÜCKWÄRTS eingegeben werden.
  - Duell     : Zwei Spieler bauen die Folge abwechselnd auf - erst die
                bisherige Folge nachtippen, dann EIN Feld anhängen. Wer sich
                vertippt, verliert.
  - Gemischt  : Der Modus (Klassisch/Speed/Reverse) wechselt jede Runde.

Ton: "Aus" (nur visuell), "An" (jedes Feld klingt) oder "Gemischt" (mal mit,
mal ohne Ton - trainiert visuelles UND akustisches Gedächtnis). Der globale
Sound-Schalter in den Optionen hat zusätzlich Vorrang.

Feldanzahl 4, 6 oder 9 stellt die Schwierigkeit ein.

Punkte (Highscore) = längste fehlerfrei wiederholte Folge (Einzelmodi).
Steuerung: Maus (Feld anklicken) oder Zifferntasten 1-9.
"""

import random

import pygame

import audio
import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent
from i18n import t

# ----------------------------------------------------------------- Farben
# Die Leuchtfarben der Felder sind die Identität des Spiels und bleiben
# fest; alle generischen UI-Farben kommen dynamisch aus der ui-Palette.
# Leuchtfarben je Feld (bis zu 9).
PAD_COLORS = [
    (46, 204, 113), (231, 76, 60), (241, 196, 15), (52, 152, 219),
    (155, 89, 182), (230, 126, 34), (26, 188, 156), (233, 96, 160),
    (149, 200, 60),
]
# Tonfrequenzen (C-Dur-Pentatonik aufsteigend), Hz.
FREQS = [261.63, 293.66, 329.63, 392.00, 440.00,
         523.25, 587.33, 659.25, 783.99]

SETUP, PLAY, OVER = "setup", "play", "over"
MODES = ["classic", "speed", "reverse", "duel", "mixed"]
AUDIO_MODES = ["off", "on", "mixed"]
PAD_OPTS = [4, 6, 9]
GRID = {4: (2, 2), 6: (3, 2), 9: (3, 3)}
SUBS = ["classic", "speed", "reverse"]   # mögliche Runden-Modi bei "Gemischt"


def _dim(c, f):
    return (int(c[0] * f), int(c[1] * f), int(c[2] * f))


def _lit(c):
    return (min(255, c[0] + 90), min(255, c[1] + 90), min(255, c[2] + 90))


class SimonGame(Game):
    name = "Simon"
    highscore_key = "simon"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        sm = self.settings.get("simon", {}) if isinstance(self.settings, dict) else {}
        self.mode = sm.get("mode", "classic")
        if self.mode not in MODES:
            self.mode = "classic"
        self.audio_mode = sm.get("audio", "on")
        if self.audio_mode not in AUDIO_MODES:
            self.audio_mode = "on"
        self.pads = sm.get("pads", 4)
        if self.pads not in PAD_OPTS:
            self.pads = 4

        self._build_fonts()
        self._over_cache = None
        self._best = self._load_best()
        self._build_setup_layout()
        self._layout()
        self.state = SETUP
        # Grundzustand, damit draw()/update() vor dem Start nicht scheitern.
        self.seq = []
        self.phase = "input"
        self.lit_pad = None
        self.lit_t = 0.0
        self.msg = None
        self.msg_t = 0.0
        self.winner = 0
        self.active = self.mode if self.mode != "mixed" else "classic"

    def _build_fonts(self):
        """Schriftgrößen aus der aktuellen Auflösung ableiten (Theme-Fonts)."""
        h = self.height
        self._small = ui.font(max(14, h // 32))
        self._tiny = ui.font(max(11, h // 40))
        self._huge = ui.font(max(28, h // 10), bold=True)

    def on_surface_changed(self):
        self._build_fonts()
        self._over_cache = None
        self._build_setup_layout()
        self._layout()

    def _load_best(self):
        try:
            data = store.load_section("simon")
            b = data.get("best", {})
            return b if isinstance(b, dict) else {}
        except Exception:
            return {}

    def _save_best(self):
        try:
            store.save_section("simon", {"best": self._best})
        except Exception:
            pass

    def _layout(self):
        self.hud_h = 46
        cols, rows = GRID[self.pads]
        area = min(self.width - 60, self.height - self.hud_h - 50)
        self.pad_gap = max(8, area // 40)
        cell = (area - self.pad_gap * (max(cols, rows) - 1)) / max(cols, rows)
        self.pad_w = cell
        gw = cols * cell + (cols - 1) * self.pad_gap
        gh = rows * cell + (rows - 1) * self.pad_gap
        self.grid_x = (self.width - gw) // 2
        self.grid_y = self.hud_h + max(12, (self.height - self.hud_h - gh) // 2)
        self.cols, self.rows = cols, rows
        self.pad_rects = []
        for i in range(self.pads):
            r, c = divmod(i, cols)
            x = self.grid_x + c * (cell + self.pad_gap)
            y = self.grid_y + r * (cell + self.pad_gap)
            self.pad_rects.append(pygame.Rect(int(x), int(y), int(cell), int(cell)))
        # Glow-Fläche für leuchtende Felder einmalig anlegen (gecacht).
        self._glow = pygame.Surface((int(cell), int(cell)), pygame.SRCALPHA)
        self._glow.fill((255, 255, 255, 60))

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(380, self.width - 50)
        y0 = int(self.height * 0.26)
        gap = 8

        def row(y, n):
            cw = (bw - gap * (n - 1)) / n
            return [pygame.Rect(int(cx - bw / 2 + i * (cw + gap)), y,
                                int(cw), 40) for i in range(n)]

        self.mode_rects = row(y0, 5)
        self.audio_rects = row(y0 + 82, 3)
        self.pad_rects_setup = row(y0 + 150, 3)
        self.start_rect = pygame.Rect(cx - 95, y0 + 210, 190, 46)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("simon", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3", "4", "5"):
                self.mode = MODES[int(k) - 1]
                self._save_setting("mode", self.mode)
                self.play_sound("click")
            elif k in ("a", "A"):
                i = (AUDIO_MODES.index(self.audio_mode) + 1) % 3
                self.audio_mode = AUDIO_MODES[i]
                self._save_setting("audio", self.audio_mode)
                self.play_sound("select")
            elif k in ("p", "P"):
                i = (PAD_OPTS.index(self.pads) + 1) % 3
                self.pads = PAD_OPTS[i]
                self._save_setting("pads", self.pads)
                self._layout()
                self.play_sound("select")
            elif k in ("Return", "space"):
                self._start_play()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.mode_rects):
                if rc.collidepoint(event.pos):
                    self.mode = MODES[i]
                    self._save_setting("mode", self.mode)
                    self.play_sound("click")
                    return
            for i, rc in enumerate(self.audio_rects):
                if rc.collidepoint(event.pos):
                    self.audio_mode = AUDIO_MODES[i]
                    self._save_setting("audio", self.audio_mode)
                    self.play_sound("select")
                    return
            for i, rc in enumerate(self.pad_rects_setup):
                if rc.collidepoint(event.pos):
                    self.pads = PAD_OPTS[i]
                    self._save_setting("pads", self.pads)
                    self._layout()
                    self.play_sound("select")
                    return
            if self.start_rect.collidepoint(event.pos):
                self._start_play()

    def _start_play(self):
        self._layout()
        self.state = PLAY
        self.game_over = False
        self.score = 0
        self.lit_pad = None
        self.lit_t = 0.0
        self.msg = None
        self.msg_t = 0.0
        if self.mode == "duel":
            self._begin_duel()
        else:
            self.seq = [random.randrange(self.pads)]
            self._begin_round()
        self.play_sound("click")

    # ===================================================== Einzelspiel-Runden
    def _begin_round(self):
        # Runden-Untermodus (bei "Gemischt" wechselnd)
        self.active = random.choice(SUBS) if self.mode == "mixed" else self.mode
        rnd = len(self.seq)
        if self.active == "speed":
            self.lit_dur = max(0.14, 0.44 - 0.02 * rnd)
            self.gap_dur = max(0.06, 0.18 - 0.008 * rnd)
        else:
            self.lit_dur = 0.42
            self.gap_dur = 0.18
        self._round_sound = (self.audio_mode != "mixed"
                             or random.random() < 0.5)
        self.expected = (list(reversed(self.seq)) if self.active == "reverse"
                         else list(self.seq))
        self.phase = "show"
        self.show_i = 0
        self.show_state = "pre"
        self.show_t = 0.55
        self.input_i = 0
        self.lit_pad = None

    def _update_show(self, dt):
        self.show_t -= dt
        if self.show_t > 0:
            return
        if self.show_state == "pre":
            self._light_show(0)
        elif self.show_state == "on":
            self.lit_pad = None
            self.show_state = "off"
            self.show_t = self.gap_dur
            self.show_i += 1
        else:  # off
            if self.show_i >= len(self.seq):
                self.phase = "input"
                self.input_i = 0
                self.lit_pad = None
            else:
                self._light_show(self.show_i)

    def _light_show(self, i):
        self.lit_pad = self.seq[i]
        self.show_state = "on"
        self.show_t = self.lit_dur
        self._tone(self.seq[i])

    # ===================================================== Duell
    def _begin_duel(self):
        self.seq = []
        self.turn_player = 0
        self.input_i = 0
        self.adding = True         # erste Aktion: ein Feld anhängen
        self.phase = "input"
        self._round_sound = True

    # ===================================================== Ton
    def _tone(self, i):
        if self.audio_mode == "off":
            return
        if self.audio_mode == "mixed" and not self._round_sound:
            return
        audio.tone(FREQS[i % len(FREQS)], 0.22, self.settings, wave="sine")

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.state == OVER:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._start_play()
                elif event.key in ("s", "S"):
                    self.state = SETUP
                    self.game_over = False
                    self.play_sound("click")
            elif event.kind == InputEvent.MOUSEDOWN:
                self._start_play()
            return
        if self.state != PLAY:
            return
        if self.mode != "duel" and self.phase != "input":
            return   # während der Wiedergabe keine Eingabe
        pad = None
        if event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.pad_rects):
                if rc.collidepoint(event.pos):
                    pad = i
                    break
        elif event.kind == InputEvent.KEYDOWN:
            if event.key in "123456789":
                idx = int(event.key) - 1
                if idx < self.pads:
                    pad = idx
        if pad is None:
            return
        if self.mode == "duel":
            self._duel_press(pad)
        else:
            self._single_press(pad)

    def _single_press(self, pad):
        self.lit_pad = pad
        self.lit_t = 0.2
        self._tone(pad)
        if pad == self.expected[self.input_i]:
            self.input_i += 1
            if self.input_i >= len(self.expected):
                # Runde geschafft
                self.score = len(self.seq)
                self.play_sound("point")
                self.seq.append(random.randrange(self.pads))
                self._begin_round()
        else:
            self._fail_single()

    def _fail_single(self):
        self.play_sound("gameover")
        key = self.mode
        if self._best.get(key, 0) < self.score:
            self._best[key] = self.score
            self._save_best()
        self.state = OVER
        self.game_over = True

    def _duel_press(self, pad):
        self.lit_pad = pad
        self.lit_t = 0.2
        self._tone(pad)
        if self.adding:
            self.seq.append(pad)
            self.adding = False
            self.input_i = 0
            self.turn_player = 1 - self.turn_player
            self.play_sound("select")
            return
        if pad == self.seq[self.input_i]:
            self.input_i += 1
            if self.input_i >= len(self.seq):
                self.adding = True      # Folge korrekt -> jetzt anhängen
                self.play_sound("point")
        else:
            # Vertippt -> aktueller Spieler verliert
            self.winner = 1 - self.turn_player
            self.play_sound("gameover")
            self.state = OVER
            self.game_over = True

    # ===================================================== Update
    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
        if self.lit_t > 0:
            self.lit_t -= dt
            if self.lit_t <= 0 and (self.mode == "duel" or self.phase == "input"):
                self.lit_pad = None
        if self.state == PLAY and self.mode != "duel" and self.phase == "show":
            self._update_show(dt)

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        if self.state == SETUP:
            self._draw_setup(s)
            return
        self._draw_hud(s)
        self._draw_pads(s)
        if self.state == OVER:
            self._draw_over(s)

    def _draw_pads(self, s):
        for i, rc in enumerate(self.pad_rects):
            col = PAD_COLORS[i]
            on = (self.lit_pad == i)
            fill = _lit(col) if on else _dim(col, 0.5)
            pygame.draw.rect(s, fill, rc, border_radius=14)
            pygame.draw.rect(s, _dim(col, 0.8), rc, 3, border_radius=14)
            if on:
                s.blit(self._glow, rc.topleft)
            num = self._small.render(str(i + 1), True, (255, 255, 255)
                                     if on else _dim(col, 0.85))
            s.blit(num, num.get_rect(center=rc.center))

    def _draw_hud(self, s):
        pygame.draw.rect(s, ui.PANEL, (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h), (self.width, self.hud_h))
        cy = self.hud_h // 2
        if self.mode == "duel":
            left = self._small.render(t("common.player1"), True,
                                      self.accent if self.turn_player == 0
                                      else ui.TEXT_DIM)
            s.blit(left, left.get_rect(midleft=(14, cy)))
            right = self._small.render(t("common.player2"), True,
                                       self.accent if self.turn_player == 1
                                       else ui.TEXT_DIM)
            s.blit(right, right.get_rect(midright=(self.width - 14, cy)))
            mid = (t("simon.add") if self.adding else t("simon.repeat"))
            mid += f"  ({len(self.seq)})"
        else:
            sc = self._small.render(t("simon.round", n=len(self.seq)), True,
                                    ui.TEXT)
            s.blit(sc, sc.get_rect(midleft=(14, cy)))
            best = self._best.get(self.mode, 0)
            bimg = self._small.render(t("simon.best", n=best), True, ui.TEXT_DIM)
            s.blit(bimg, bimg.get_rect(midright=(self.width - 14, cy)))
            if self.phase == "show":
                mid = t("simon.watch")
            else:
                mid = t("simon.your_input")
            if self.mode == "mixed":
                mid += "  ·  " + t("simon.mode." + self.active)
        img = self._small.render(mid, True, self.accent)
        s.blit(img, img.get_rect(center=(self.width // 2, cy)))

    def _draw_over(self, s):
        # Halbtransparentes Banner-Panel wird gecacht (Software-Rendering).
        if self._over_cache is None or self._over_cache.get_width() != self.width:
            ov = pygame.Surface((self.width, 108), pygame.SRCALPHA)
            ov.fill((10, 8, 16, 210))
            self._over_cache = ov
        y = self.height // 2 - 54
        s.blit(self._over_cache, (0, y))
        pygame.draw.line(s, self.accent, (0, y), (self.width, y))
        pygame.draw.line(s, self.accent, (0, y + 107), (self.width, y + 107))
        cx = self.width // 2
        if self.mode == "duel":
            head = self._huge.render(t("common.player_wins", n=self.winner + 1),
                                     True, self.accent)
            sub = self._small.render(t("simon.duel_len", n=len(self.seq)),
                                     True, ui.TEXT)
        else:
            head = self._huge.render(t("simon.score", n=self.score), True,
                                     self.accent)
            sub = self._small.render(t("simon.best", n=self._best.get(self.mode, 0)),
                                     True, ui.TEXT)
        s.blit(head, head.get_rect(center=(cx, y + 34)))
        s.blit(sub, sub.get_rect(center=(cx, y + 68)))
        hint = self._tiny.render(t("simon.new_round"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, y + 92)))

    def _draw_setup(self, s):
        cx = self.width // 2
        title = self._huge.render(t("simon.title"), True, self.accent)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.12))))
        sub = self._small.render(t("simon.subtitle"), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(self.height * 0.185))))

        def label(y, txt):
            im = self._tiny.render(txt, True, ui.TEXT_DIM)
            s.blit(im, im.get_rect(midbottom=(cx, y - 4)))

        label(self.mode_rects[0].top, t("simon.lbl_mode"))
        for i, rc in enumerate(self.mode_rects):
            on = (self.mode == MODES[i])
            self._btn(s, rc, t("simon.mode." + MODES[i]), on, self._tiny)
        label(self.audio_rects[0].top, t("simon.lbl_audio"))
        for i, rc in enumerate(self.audio_rects):
            on = (self.audio_mode == AUDIO_MODES[i])
            self._btn(s, rc, t("simon.audio." + AUDIO_MODES[i]), on, self._small)
        label(self.pad_rects_setup[0].top, t("simon.lbl_pads"))
        for i, rc in enumerate(self.pad_rects_setup):
            on = (self.pads == PAD_OPTS[i])
            self._btn(s, rc, str(PAD_OPTS[i]), on, self._small)
        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=9)
        pygame.draw.rect(s, self.accent, self.start_rect, 2, border_radius=9)
        st = self.font.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        hint = self._tiny.render(t("simon.setup_hint"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, self.height - 14)))

    def _btn(self, s, rc, text, on, fnt):
        pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=8)
        pygame.draw.rect(s, self.accent if on else ui.BORDER, rc,
                         2 if on else 1, border_radius=8)
        im = fnt.render(text, True, ui.TEXT if on else ui.TEXT_DIM)
        s.blit(im, im.get_rect(center=rc.center))
