# -*- coding: utf-8 -*-
"""
wordle.py
=========
Wordle - errate das 5-Buchstaben-Wort in hoechstens 6 Versuchen.

- Nach jedem Rateversuch faerben sich die Buchstaben: gruen = richtig (Position
  stimmt), gelb = im Wort (falsche Position), grau = nicht enthalten. Doppelte
  Buchstaben werden korrekt gezaehlt (Standard-Wordle-Algorithmus).
- Endlos-Streak als Highscore: fuer jedes geloeste Wort gibt es Punkte (weniger
  Versuche = mehr), danach kommt sofort ein neues Wort. Das erste NICHT geloeste
  Wort beendet die Partie; die gesammelten Punkte sind der Highscore.
- Die Loesungswoerter stammen aus einer kuratierten Liste je Sprache
  (games/wordle_words.py, nur A-Z). Rateversuche werden NICHT gegen ein
  Woerterbuch geprueft - jede 5-Buchstaben-Eingabe ist erlaubt.

Steuerung: Buchstabentasten A-Z tippen, Enter = raten (bei 5 Buchstaben),
Backspace = loeschen. Die Bildschirmtastatur unten ist auch anklickbar.
Nach Ende bzw. geloestem Wort: Enter/Klick geht weiter.
"""

import random

import pygame

import ui
import i18n
from game_base import Game, InputEvent
from i18n import t

from .wordle_words import words_for

COL_TILE_EMPTY = (30, 30, 38)
COL_TILE_BORDER = (58, 58, 70)
COL_TILE_ACTIVE = (90, 90, 108)
COL_CORRECT = (106, 170, 100)     # gruen (= Sidebar-Farbe #6aaa64)
COL_PRESENT = (201, 180, 88)      # gelb
COL_ABSENT = (58, 58, 62)         # grau
COL_TEXT = (235, 236, 240)
COL_DIM = (150, 152, 166)
COL_KEY = (70, 72, 86)
COL_KEY_TEXT = (232, 233, 240)
COL_ACCENT = (106, 170, 100)

ROWS, COLS = 6, 5
REVEAL_STEP = 0.14                # Sekunden je Kachel bei der Aufdeckung

PLAY, REVEAL, SOLVED, OVER = "play", "reveal", "solved", "over"

_QWERTY = ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"]
_QWERTZ = ["QWERTZUIOP", "ASDFGHJKL", "YXCVBNM"]

# Rangordnung der Buchstaben-Zustaende (hoeher gewinnt in der Tastatur-Faerbung)
_RANK = {None: 0, "absent": 1, "present": 2, "correct": 3}


def evaluate(guess, answer):
    """Standard-Wordle-Bewertung mit korrekter Doppelbuchstaben-Zaehlung."""
    result = ["absent"] * 5
    rest = list(answer)
    for i in range(5):
        if guess[i] == answer[i]:
            result[i] = "correct"
            rest[i] = None
    for i in range(5):
        if result[i] == "correct":
            continue
        if guess[i] in rest:
            result[i] = "present"
            rest[rest.index(guess[i])] = None
    return result


class WordleGame(Game):
    name = "Wordle"
    highscore_key = "wordle"
    supports_multiplayer = False

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        self.solved_count = 0
        self.lang = i18n.get_language()
        self.words = words_for(self.lang)
        self._make_fonts()
        self._new_word()
        self._layout()

    def _make_fonts(self):
        self._tile_font = ui.font(max(20, self.height // 16), bold=True)
        self._key_font = ui.font(15, bold=True)
        self._hud = ui.font(18, bold=True)
        self._small = ui.font(14)
        self._tiny = ui.font(12)
        self._huge = ui.font(max(24, self.height // 13), bold=True)

    def on_surface_changed(self):
        self._make_fonts()
        self._layout()

    def _new_word(self):
        self.answer = random.choice(self.words)
        self.guesses = []              # Liste (wort, ergebnis)
        self.current = ""
        self.row = 0
        self.keystate = {}             # Buchstabe -> zustand
        self.reveal = None             # dict(row, guess, result, t)
        self.last_points = 0
        self.state = PLAY

    def _layout(self):
        self.hud_h = 44
        # Tastatur unten
        self.key_h = max(30, self.height // 12)
        self.key_gap = max(3, self.width // 160)
        kb_h = 3 * self.key_h + 4 * self.key_gap
        self.kb_top = self.height - kb_h - 6
        self._build_keyboard()
        # Rasterbereich zwischen HUD und Tastatur
        top = self.hud_h + 8
        bottom = self.kb_top - 8
        gap = max(4, self.width // 120)
        self.tile = int(min((self.width - 40 - (COLS - 1) * gap) / COLS,
                            (bottom - top - (ROWS - 1) * gap) / ROWS))
        self.tile = max(24, self.tile)
        self.tile_gap = gap
        grid_w = COLS * self.tile + (COLS - 1) * gap
        grid_h = ROWS * self.tile + (ROWS - 1) * gap
        self.grid_x = (self.width - grid_w) // 2
        self.grid_y = top + max(0, (bottom - top - grid_h) // 2)

    def _build_keyboard(self):
        layout = _QWERTZ if self.lang == "de" else _QWERTY
        self.key_rects = {}
        self.enter_rect = None
        self.del_rect = None
        y = self.kb_top
        for ri, rowstr in enumerate(layout):
            keys = list(rowstr)
            # Untere Reihe bekommt ENTER links und DEL rechts
            n_slots = len(keys) + (2 if ri == 2 else 0)
            kw = int((self.width - (n_slots + 1) * self.key_gap) / max(1, n_slots))
            kw = max(20, min(kw, 44))
            special_w = int(kw * 1.5)
            row_w = len(keys) * kw + (len(keys) - 1) * self.key_gap
            if ri == 2:
                row_w += 2 * (special_w + self.key_gap)
            x = (self.width - row_w) // 2
            if ri == 2:
                self.enter_rect = pygame.Rect(x, y, special_w, self.key_h)
                x += special_w + self.key_gap
            for ch in keys:
                self.key_rects[ch] = pygame.Rect(x, y, kw, self.key_h)
                x += kw + self.key_gap
            if ri == 2:
                self.del_rect = pygame.Rect(x, y, special_w, self.key_h)
            y += self.key_h + self.key_gap

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == OVER:
            if self._is_continue(event):
                self.game_over = False
                self.reset()
            return
        if self.state == SOLVED:
            if self._is_continue(event):
                self._new_word()
                self.play_sound("click")
            return
        if self.state == REVEAL:
            return
        # PLAY
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k == "BackSpace":
                if self.current:
                    self.current = self.current[:-1]
                    self.play_sound("move")
            elif k == "Return":
                self._submit()
            elif len(k) == 1 and k.isalpha() and k.isascii():
                self._type(k.upper())
        elif event.kind == InputEvent.MOUSEDOWN:
            self._click_keyboard(event.pos)

    def _is_continue(self, event):
        return (event.kind == InputEvent.MOUSEDOWN
                or (event.kind == InputEvent.KEYDOWN
                    and event.key in ("Return", "space")))

    def _type(self, ch):
        if len(self.current) < COLS:
            self.current += ch
            self.play_sound("click")

    def _click_keyboard(self, pos):
        if self.enter_rect and self.enter_rect.collidepoint(pos):
            self._submit()
            return
        if self.del_rect and self.del_rect.collidepoint(pos):
            if self.current:
                self.current = self.current[:-1]
                self.play_sound("move")
            return
        for ch, rc in self.key_rects.items():
            if rc.collidepoint(pos):
                self._type(ch)
                return

    def _submit(self):
        if len(self.current) != COLS:
            self.play_sound("click")
            return
        result = evaluate(self.current, self.answer)
        self.reveal = dict(row=self.row, guess=self.current, result=result, t=0.0)
        self.state = REVEAL
        self.play_sound("move")

    def _finish_reveal(self):
        rv = self.reveal
        guess, result = rv["guess"], rv["result"]
        self.guesses.append((guess, result))
        for ch, st in zip(guess, result):
            if _RANK[st] > _RANK.get(self.keystate.get(ch)):
                self.keystate[ch] = st
        self.reveal = None
        self.current = ""
        self.row += 1
        if all(st == "correct" for st in result):
            pts = 10 + (ROWS - self.row) * 10        # weniger Versuche = mehr
            self.last_points = pts
            self.score += pts
            self.solved_count += 1
            self.state = SOLVED
            self.report_result(True)
            if self.row <= 2:
                self.ach_event("wordle_two")
            self.play_sound("win")
        elif self.row >= ROWS:
            self.state = OVER
            self.game_over = True         # main.py speichert den Highscore
            if self.solved_count == 0:
                self.report_result(False)
            self.play_sound("gameover")
        else:
            self.state = PLAY

    # ===================================================== Update
    def update(self, dt):
        if self.state == REVEAL and self.reveal is not None:
            self.reveal["t"] += dt
            if self.reveal["t"] >= COLS * REVEAL_STEP + 0.1:
                self._finish_reveal()

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        # Themen-Hintergrund statt flacher Fläche (reagiert auf Theme-Wechsel).
        ui.draw_background(s, self.width, self.height, stars=False, aurora=True)
        self._draw_hud(s)
        self._draw_grid(s)
        self._draw_keyboard(s)
        if self.state == SOLVED:
            self._draw_banner(s, t("wd.solved", n=self.last_points), COL_ACCENT,
                              t("wd.next"))
        elif self.state == OVER:
            self._draw_banner(s, t("wd.gameover"), (225, 110, 100),
                              t("wd.answer_was", w=self.answer) + "   -   "
                              + t("common.enter_restart"))

    def _draw_hud(self, s):
        pygame.draw.rect(s, (26, 26, 34), (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, COL_TILE_BORDER, (0, self.hud_h), (self.width, self.hud_h))
        cy = self.hud_h // 2
        img = self._hud.render(t("wd.score", n=self.score), True, COL_ACCENT)
        s.blit(img, img.get_rect(midleft=(14, cy)))
        img = self._small.render(t("wd.solved_n", n=self.solved_count), True, COL_DIM)
        s.blit(img, img.get_rect(center=(self.width // 2, cy)))
        img = self._small.render(t("wd.tries", a=len(self.guesses), b=ROWS), True,
                                 COL_DIM)
        s.blit(img, img.get_rect(midright=(self.width - 14, cy)))

    def _tile_color(self, state):
        return {"correct": COL_CORRECT, "present": COL_PRESENT,
                "absent": COL_ABSENT}.get(state, COL_TILE_EMPTY)

    def _draw_grid(self, s):
        for r in range(ROWS):
            for c in range(COLS):
                x = self.grid_x + c * (self.tile + self.tile_gap)
                y = self.grid_y + r * (self.tile + self.tile_gap)
                rc = pygame.Rect(x, y, self.tile, self.tile)
                ch = ""
                fill = COL_TILE_EMPTY
                border = COL_TILE_BORDER
                revealed = False
                if r < len(self.guesses):
                    guess, result = self.guesses[r]
                    ch = guess[c]
                    fill = self._tile_color(result[c])
                    border = fill
                    revealed = True
                elif self.reveal is not None and r == self.reveal["row"]:
                    ch = self.reveal["guess"][c]
                    # Kachel deckt sich nacheinander (Spalte fuer Spalte) auf
                    if self.reveal["t"] >= (c + 1) * REVEAL_STEP:
                        fill = self._tile_color(self.reveal["result"][c])
                        border = fill
                        revealed = True
                    else:
                        border = COL_TILE_ACTIVE
                elif r == self.row:
                    if c < len(self.current):
                        ch = self.current[c]
                        border = COL_TILE_ACTIVE
                pygame.draw.rect(s, fill, rc, border_radius=6)
                pygame.draw.rect(s, border, rc, 2, border_radius=6)
                if ch:
                    col = COL_TEXT
                    img = self._tile_font.render(ch, True, col)
                    s.blit(img, img.get_rect(center=rc.center))

    def _draw_keyboard(self, s):
        for ch, rc in self.key_rects.items():
            st = self.keystate.get(ch)
            col = self._tile_color(st) if st else COL_KEY
            pygame.draw.rect(s, col, rc, border_radius=5)
            img = self._key_font.render(ch, True, COL_KEY_TEXT)
            s.blit(img, img.get_rect(center=rc.center))
        for rc, label in ((self.enter_rect, t("wd.enter")),
                          (self.del_rect, "<")):
            if rc is None:
                continue
            pygame.draw.rect(s, COL_KEY, rc, border_radius=5)
            img = self._tiny.render(label, True, COL_KEY_TEXT)
            s.blit(img, img.get_rect(center=rc.center))

    def _draw_banner(self, s, title, color, sub):
        w = min(self.width - 40, 460)
        h = 92
        rc = pygame.Rect((self.width - w) // 2, self.grid_y - 4, w, h)
        panel = pygame.Surface((w, h), pygame.SRCALPHA)
        panel.fill((16, 18, 24, 235))
        s.blit(panel, rc.topleft)
        pygame.draw.rect(s, color, rc, 2, border_radius=12)
        img = self._huge.render(title, True, color)
        s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 34)))
        img = self._small.render(sub, True, COL_TEXT)
        s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 66)))
