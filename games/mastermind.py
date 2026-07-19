# -*- coding: utf-8 -*-
"""
mastermind.py
=============
Mastermind - knacke den geheimen Farbcode.

- Der Computer würfelt einen verdeckten Code aus mehreren Farbstiften (Dubletten
  erlaubt). Du legst je Reihe einen Tipp und bekommst Rückmeldung:
  SCHWARZER Pin = richtige Farbe an richtiger Stelle, WEISSER Pin = richtige Farbe
  an falscher Stelle (Standard-Mastermind-Zählung mit Häufigkeiten).
- Drei Modi: Leicht (4 Stifte / 6 Farben / 12 Reihen), Klassik (4 / 6 / 10) und
  Schwer (5 / 8 / 10).
- Endlos-Streak wie bei Wordle: jeder geknackte Code bringt Punkte (weniger
  Versuche + schwererer Modus = mehr), danach kommt sofort ein neuer Code. Der
  erste NICHT geknackte Code beendet die Partie - die Summe ist der Highscore.

Steuerung: Farbe unten anklicken (oder Tasten 1..N) füllt den nächsten Platz,
Klick auf einen gelegten Stift löscht ihn, [Enter]/OK wertet die Reihe aus,
[Rücktaste]/< löscht den letzten. Nach Ende: Enter/Klick geht weiter.
"""

import random
from collections import Counter

import pygame

import ui
from game_base import Game, InputEvent
from i18n import t

COL_BG = (18, 20, 30)
COL_SLOT = (44, 50, 70)
COL_SLOT_BORDER = (70, 78, 104)
COL_PEG_BLACK = (26, 28, 36)
COL_PEG_WHITE = (236, 238, 245)

# Bis zu 8 klar unterscheidbare Stiftfarben.
PALETTE = [
    (228, 72, 72), (240, 150, 52), (240, 214, 74), (96, 200, 112),
    (66, 202, 210), (82, 132, 240), (172, 112, 236), (240, 120, 190),
]

# (pins, colors, rows) je Modus.
MODE_CFG = {"easy": (4, 6, 12), "classic": (4, 6, 10), "hard": (5, 8, 10)}
DIFF_BONUS = {"easy": 0, "classic": 10, "hard": 25}

PLAY, SOLVED, OVER = "play", "solved", "over"


def feedback(guess, secret):
    """(schwarz, weiß) nach Standard-Mastermind-Regeln."""
    black = sum(g == s for g, s in zip(guess, secret))
    sc, gc = Counter(secret), Counter(guess)
    total = sum(min(sc[c], gc[c]) for c in gc)
    return black, total - black


class MastermindGame(Game):
    name = "Mastermind"
    highscore_key = "mastermind"
    supports_multiplayer = False

    MODES = [("easy", "mm.mode.easy"), ("classic", "mm.mode.classic"),
             ("hard", "mm.mode.hard")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.pins, self.colors, self.max_rows = MODE_CFG.get(self.mode,
                                                             MODE_CFG["classic"])
        self.score = 0
        self.game_over = False
        self.solved_count = 0
        self._make_fonts()
        self._new_code()
        self._layout()

    def _make_fonts(self):
        self._hud = ui.font(20, bold=True)
        self._small = ui.font(15)
        self._tiny = ui.font(13, bold=True)
        self._huge = ui.font(max(28, self.height // 12), bold=True)

    def on_surface_changed(self):
        self._make_fonts()
        self._layout()

    def _new_code(self):
        self.secret = [random.randrange(self.colors) for _ in range(self.pins)]
        self.rows_data = []              # (guess, (black, white))
        self.current = [None] * self.pins
        self.row = 0
        self.state = PLAY
        self.last_points = 0

    def _layout(self):
        pins, C = self.pins, self.colors
        gap = 6
        # Palette unten
        self.pal_h = max(46, self.height // 10)
        self.pal_top = self.height - self.pal_h - 10
        self._build_palette()
        # Reihen-Bereich zwischen HUD und Palette
        top = 64
        bottom = self.pal_top - 12
        pitch = (bottom - top) / self.max_rows
        peg = int(min(pitch - gap, (self.width - 150) / pins - gap))
        self.peg = max(16, min(peg, 46))
        self.gap = gap
        self.pitch = pitch
        self.grid_top = top
        grid_w = pins * (self.peg + gap) - gap
        self.fb_w = max(26, self.peg)
        total_w = grid_w + 22 + self.fb_w
        self.grid_x = (self.width - total_w) // 2
        self.fb_x = self.grid_x + grid_w + 22

    def _build_palette(self):
        gap = 8
        C = self.colors
        pw = int(min((self.width - 40) / (C + 3), 52))
        pw = max(28, pw)
        special = int(pw * 1.4)
        row_w = C * pw + (C - 1) * gap + 2 * (special + gap)
        x = (self.width - row_w) // 2
        y = self.pal_top + (self.pal_h - pw) // 2
        h = min(self.pal_h - 8, pw)
        self.del_rect = pygame.Rect(x, y, special, h)
        x += special + gap
        self.swatch_rects = []
        for _ in range(C):
            self.swatch_rects.append(pygame.Rect(x, y, pw, h))
            x += pw + gap
        self.check_rect = pygame.Rect(x, y, special, h)

    def _row_peg_rect(self, row, i):
        x = self.grid_x + i * (self.peg + self.gap)
        y = int(self.grid_top + row * self.pitch)
        return pygame.Rect(x, y, self.peg, self.peg)

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state in (SOLVED, OVER):
            if self._is_continue(event):
                if self.state == OVER:
                    self.game_over = False
                    self.reset()
                else:
                    self._new_code()
                    self.play_sound("click")
            return
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k == "BackSpace":
                self._delete()
            elif k == "Return":
                self._submit()
            elif k.isdigit() and 1 <= int(k) <= self.colors:
                self._place(int(k) - 1)
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            self._click(event.pos)

    def _is_continue(self, event):
        return (event.kind == InputEvent.MOUSEDOWN
                or (event.kind == InputEvent.KEYDOWN
                    and event.key in ("Return", "space")))

    def _click(self, pos):
        if self.check_rect.collidepoint(pos):
            self._submit()
            return
        if self.del_rect.collidepoint(pos):
            self._delete()
            return
        # Klick auf einen gelegten Stift der aktuellen Reihe -> löschen
        for i in range(self.pins):
            if self.current[i] is not None and \
                    self._row_peg_rect(self.row, i).collidepoint(pos):
                self.current[i] = None
                self.play_sound("move")
                return
        for ci, rc in enumerate(self.swatch_rects):
            if rc.collidepoint(pos):
                self._place(ci)
                return

    def _place(self, color_idx):
        for i in range(self.pins):
            if self.current[i] is None:
                self.current[i] = color_idx
                self.play_sound("click")
                return
        self.play_sound("click")

    def _delete(self):
        for i in range(self.pins - 1, -1, -1):
            if self.current[i] is not None:
                self.current[i] = None
                self.play_sound("move")
                return

    def _submit(self):
        if any(c is None for c in self.current):
            self.play_sound("click")
            return
        guess = list(self.current)
        black, white = feedback(guess, self.secret)
        self.rows_data.append((guess, (black, white)))
        self.current = [None] * self.pins
        self.row += 1
        if black == self.pins:
            used = self.row
            pts = 20 + (self.max_rows - used) * 8 + DIFF_BONUS.get(self.mode, 10)
            self.last_points = pts
            self.score += pts
            self.solved_count += 1
            self.state = SOLVED
            self.play_sound("win")
        elif self.row >= self.max_rows:
            self.state = OVER
            self.game_over = True             # main.py speichert den Highscore
            self.play_sound("gameover")
        else:
            self.play_sound("move")

    def update(self, dt):
        pass

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height, stars=False, aurora=True)
        self._draw_hud(s)
        self._draw_rows(s)
        self._draw_palette(s)
        if self.state == SOLVED:
            self._draw_banner(s, t("mm.solved", n=self.last_points), self.accent,
                              t("mm.next"), reveal=False)
        elif self.state == OVER:
            self._draw_banner(s, t("mm.gameover"), (228, 96, 96),
                              t("common.enter_restart"), reveal=True)

    def _draw_hud(self, s):
        img = self._hud.render(t("mm.title"), True, self.accent)
        s.blit(img, img.get_rect(midleft=(20, 30)))
        img = self._small.render(t("mm.score", n=self.score), True, ui.GOLD)
        s.blit(img, img.get_rect(center=(self.width // 2, 22)))
        img = self._small.render(t("mm.rows", a=min(self.row + 1, self.max_rows),
                                    b=self.max_rows), True, ui.TEXT_DIM)
        s.blit(img, img.get_rect(midright=(self.width - 20, 22)))
        img = self._small.render(t("mm.solved_n", n=self.solved_count), True,
                                 ui.TEXT_DIM)
        s.blit(img, img.get_rect(midright=(self.width - 20, 44)))

    def _draw_rows(self, s):
        for r in range(self.max_rows):
            active = (r == self.row and self.state == PLAY)
            for i in range(self.pins):
                rc = self._row_peg_rect(r, i)
                val = None
                if r < len(self.rows_data):
                    val = self.rows_data[r][0][i]
                elif active:
                    val = self.current[i]
                self._draw_peg(s, rc, val, active and val is None)
            # Rückmeldungs-Pins
            if r < len(self.rows_data):
                self._draw_feedback(s, r, self.rows_data[r][1])

    def _draw_peg(self, s, rc, val, highlight):
        cx, cy = rc.center
        rad = rc.w // 2
        if val is None:
            pygame.draw.circle(s, COL_SLOT, (cx, cy), rad)
            border = self.accent if highlight else COL_SLOT_BORDER
            pygame.draw.circle(s, border, (cx, cy), rad, 2)
        else:
            col = PALETTE[val]
            pygame.draw.circle(s, col, (cx, cy), rad)
            pygame.draw.circle(s, ui.mix(col, (255, 255, 255), 0.35),
                               (cx, cy), rad, 2)
            pygame.draw.circle(s, ui.mix(col, (255, 255, 255), 0.5),
                               (cx - rad // 3, cy - rad // 3), max(2, rad // 5))

    def _draw_feedback(self, s, r, fb):
        black, white = fb
        pegs = [COL_PEG_BLACK] * black + [COL_PEG_WHITE] * white
        pegs += [(58, 64, 84)] * (self.pins - len(pegs))
        cols = 2 if self.pins <= 4 else 3
        sr = max(3, self.peg // 6)
        step = sr * 2 + 3
        y0 = int(self.grid_top + r * self.pitch) + self.peg // 2 - step
        for k, pc in enumerate(pegs):
            gx = k % cols
            gy = k // cols
            x = self.fb_x + gx * step + sr
            y = y0 + gy * step
            pygame.draw.circle(s, pc, (x, y), sr)
            if pc == COL_PEG_WHITE:
                pygame.draw.circle(s, (120, 122, 130), (x, y), sr, 1)

    def _draw_palette(self, s):
        for ci, rc in enumerate(self.swatch_rects):
            col = PALETTE[ci]
            pygame.draw.rect(s, col, rc, border_radius=8)
            pygame.draw.rect(s, ui.mix(col, (255, 255, 255), 0.3), rc, 2,
                             border_radius=8)
            img = self._tiny.render(str(ci + 1), True, (20, 22, 30))
            s.blit(img, img.get_rect(center=(rc.centerx, rc.bottom - 9)))
        for rc, label in ((self.del_rect, "<"), (self.check_rect, "OK")):
            pygame.draw.rect(s, (60, 66, 90), rc, border_radius=8)
            pygame.draw.rect(s, ui.BORDER_LIGHT, rc, 2, border_radius=8)
            img = self._tiny.render(label, True, ui.TEXT)
            s.blit(img, img.get_rect(center=rc.center))

    def _draw_banner(self, s, title, color, sub, reveal):
        w = min(self.width - 40, 480)
        h = 150 if reveal else 108
        rc = pygame.Rect((self.width - w) // 2, (self.height - h) // 2, w, h)
        panel = pygame.Surface((w, h), pygame.SRCALPHA)
        panel.fill((16, 18, 24, 238))
        s.blit(panel, rc.topleft)
        pygame.draw.rect(s, color, rc, 2, border_radius=14)
        img = self._huge.render(title, True, color)
        s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 36)))
        if reveal:
            img = self._small.render(t("mm.code_was"), True, ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 70)))
            rad = 12
            tot = self.pins * (rad * 2 + 8) - 8
            x = rc.centerx - tot // 2 + rad
            for val in self.secret:
                pygame.draw.circle(s, PALETTE[val], (x, rc.y + 98), rad)
                pygame.draw.circle(s, ui.mix(PALETTE[val], (255, 255, 255), 0.35),
                                   (x, rc.y + 98), rad, 2)
                x += rad * 2 + 8
            img = self._small.render(sub, True, ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 128)))
        else:
            img = self._small.render(sub, True, ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 78)))
