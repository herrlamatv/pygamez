# -*- coding: utf-8 -*-
"""
hangman.py
==========
Galgenmaennchen - errate das Wort, bevor der Galgen fertig ist.

- Jeder falsche Buchstabe zeichnet ein weiteres Koerperteil (Kopf, Rumpf, zwei
  Arme, zwei Beine); nach 6 Fehlern ist die Partie verloren.
- Drei Modi ueber die Wortlaenge: Kurze Woerter (3-5), Gemischt (3-12) und
  Lange Woerter (7-12).
- Endlos-Streak wie bei Wordle: jedes erratene Wort bringt Punkte (mehr
  Restleben + laengeres Wort = mehr), danach kommt sofort ein neues Wort. Der
  erste Verlust beendet die Partie - die Summe ist der Highscore.

Steuerung: Buchstaben A-Z tippen oder die Bildschirmtastatur anklicken.
Nach Ende bzw. erratenem Wort: Enter/Klick geht weiter.
"""

import random

import pygame

import ui
import i18n
from game_base import Game, InputEvent
from i18n import t

from .hangman_words import words_for

COL_BG = (18, 20, 30)
COL_KEY = (70, 72, 86)
COL_KEY_TEXT = (232, 233, 240)
COL_CORRECT = (106, 190, 120)
COL_ABSENT = (58, 58, 64)
COL_WORD = (236, 238, 246)

MAX_WRONG = 6

_QWERTY = ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"]
_QWERTZ = ["QWERTZUIOP", "ASDFGHJKL", "YXCVBNM"]

PLAY, SOLVED, OVER = "play", "solved", "over"


class _MenuName:
    """Menü-Name: auf Deutsch 'Galgenmännchen', in allen anderen Sprachen 'Hangman'.

    Als Deskriptor gebaut, weil das Menü den Namen direkt über die KLASSE liest
    (``cls.name``) und nicht übersetzt - so bleibt er trotzdem sprachabhängig.
    """

    def __get__(self, obj, owner=None):
        return "Galgenmännchen" if i18n.get_language() == "de" else "Hangman"


class HangmanGame(Game):
    name = _MenuName()
    highscore_key = "hangman"
    supports_multiplayer = False

    MODES = [("short", "hang.mode.short"), ("mixed", "hang.mode.mixed"),
             ("long", "hang.mode.long")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.accent = ui.game_color(type(self).__name__)
        self.lang = i18n.get_language()
        self.words = words_for(self.lang, self.mode if self.mode in
                               ("short", "mixed", "long") else "mixed")
        self.score = 0
        self.game_over = False
        self.solved_count = 0
        self._make_fonts()
        self._new_word()
        self._layout()

    def _make_fonts(self):
        self._hud = ui.font(20, bold=True)
        self._small = ui.font(15)
        self._key_font = ui.font(16, bold=True)
        self._word_font = ui.font(max(30, self.height // 12), bold=True)
        self._huge = ui.font(max(28, self.height // 12), bold=True)

    def on_surface_changed(self):
        self._make_fonts()
        self._layout()

    def _new_word(self):
        self.word = random.choice(self.words)
        self.guessed = set()
        self.wrong = 0
        self.last_points = 0
        self.state = PLAY

    def _layout(self):
        # Tastatur unten (3 Reihen, nur Buchstaben)
        self.key_h = max(30, self.height // 12)
        self.key_gap = max(3, self.width // 160)
        kb_h = 3 * self.key_h + 4 * self.key_gap
        self.kb_top = self.height - kb_h - 6
        self._build_keyboard()
        # Wortzeile knapp ueber der Tastatur
        self.word_y = self.kb_top - 46
        # Galgen-Bereich zwischen HUD und Wortzeile
        self.draw_top = 64
        self.draw_bottom = self.word_y - 24

    def _build_keyboard(self):
        layout = _QWERTZ if self.lang == "de" else _QWERTY
        self.key_rects = {}
        y = self.kb_top
        for rowstr in layout:
            keys = list(rowstr)
            kw = int((self.width - (len(keys) + 1) * self.key_gap) / len(keys))
            kw = max(20, min(kw, 46))
            row_w = len(keys) * kw + (len(keys) - 1) * self.key_gap
            x = (self.width - row_w) // 2
            for ch in keys:
                self.key_rects[ch] = pygame.Rect(x, y, kw, self.key_h)
                x += kw + self.key_gap
            y += self.key_h + self.key_gap

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state in (SOLVED, OVER):
            if self._is_continue(event):
                if self.state == OVER:
                    self.game_over = False
                    self.reset()
                else:
                    self._new_word()
                    self.play_sound("click")
            return
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if len(k) == 1 and k.isalpha() and k.isascii():
                self._guess(k.upper())
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            for ch, rc in self.key_rects.items():
                if rc.collidepoint(event.pos):
                    self._guess(ch)
                    return

    def _is_continue(self, event):
        return (event.kind == InputEvent.MOUSEDOWN
                or (event.kind == InputEvent.KEYDOWN
                    and event.key in ("Return", "space")))

    def _guess(self, ch):
        if self.state != PLAY or ch in self.guessed:
            return
        self.guessed.add(ch)
        if ch in self.word:
            self.play_sound("click")
            if all(c in self.guessed for c in set(self.word)):
                self._solve()
        else:
            self.wrong += 1
            self.play_sound("move")
            if self.wrong >= MAX_WRONG:
                self.state = OVER
                self.game_over = True        # main.py speichert den Highscore
                self.play_sound("gameover")

    def _solve(self):
        pts = 10 + (MAX_WRONG - self.wrong) * 8 + len(self.word)
        self.last_points = pts
        self.score += pts
        self.solved_count += 1
        self.state = SOLVED
        self.play_sound("win")

    def update(self, dt):
        pass

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height, stars=False, aurora=True)
        self._draw_hud(s)
        self._draw_gallows(s)
        self._draw_word(s)
        self._draw_keyboard(s)
        if self.state == SOLVED:
            self._draw_banner(s, t("hang.solved", n=self.last_points),
                              self.accent, t("hang.next"))
        elif self.state == OVER:
            self._draw_banner(s, t("hang.gameover"), (228, 96, 96),
                              t("hang.word_was", w=self.word) + "   ·   "
                              + t("common.enter_restart"))

    def _draw_hud(self, s):
        img = self._hud.render(t("hang.title"), True, self.accent)
        s.blit(img, img.get_rect(midleft=(20, 30)))
        img = self._small.render(t("hang.score", n=self.score), True, ui.GOLD)
        s.blit(img, img.get_rect(center=(self.width // 2, 22)))
        img = self._small.render(t("hang.misses", a=self.wrong, b=MAX_WRONG),
                                 True, ui.TEXT_DIM)
        s.blit(img, img.get_rect(midright=(self.width - 20, 22)))
        img = self._small.render(t("hang.solved_n", n=self.solved_count), True,
                                 ui.TEXT_DIM)
        s.blit(img, img.get_rect(midright=(self.width - 20, 44)))

    def _draw_gallows(self, s):
        # Zeichenbereich (quadratisch) mittig im oberen Feld
        size = min(self.width - 80, self.draw_bottom - self.draw_top)
        size = max(120, size)
        ox = (self.width - size) // 2
        oy = self.draw_top + max(0, (self.draw_bottom - self.draw_top - size) // 2)
        u = size / 10.0
        wood = ui.mix(self.accent, (170, 140, 110), 0.5)
        lw = max(3, int(u * 0.28))

        def P(gx, gy):
            return (int(ox + gx * u), int(oy + gy * u))

        # Galgen (immer sichtbar)
        pygame.draw.line(s, wood, P(1, 9.4), P(6, 9.4), lw)      # Boden
        pygame.draw.line(s, wood, P(2.5, 9.4), P(2.5, 0.8), lw)  # Pfosten
        pygame.draw.line(s, wood, P(2.5, 0.8), P(6.2, 0.8), lw)  # Balken
        pygame.draw.line(s, wood, P(6.2, 0.8), P(6.2, 1.8), lw)  # Seil

        red = (232, 110, 100)
        parts = self.wrong
        cx, cy = P(6.2, 2.7)
        head_r = int(u * 0.9)
        if parts >= 1:                                          # Kopf
            pygame.draw.circle(s, red, (cx, cy), head_r, max(2, lw - 1))
        if parts >= 2:                                          # Rumpf
            pygame.draw.line(s, red, (cx, cy + head_r),
                             (cx, int(cy + head_r + u * 2.6)), lw)
        body_bottom = int(cy + head_r + u * 2.6)
        shoulder = int(cy + head_r + u * 0.5)
        if parts >= 3:                                          # linker Arm
            pygame.draw.line(s, red, (cx, shoulder),
                             (int(cx - u * 1.3), int(shoulder + u * 1.2)), lw)
        if parts >= 4:                                          # rechter Arm
            pygame.draw.line(s, red, (cx, shoulder),
                             (int(cx + u * 1.3), int(shoulder + u * 1.2)), lw)
        if parts >= 5:                                          # linkes Bein
            pygame.draw.line(s, red, (cx, body_bottom),
                             (int(cx - u * 1.2), int(body_bottom + u * 1.6)), lw)
        if parts >= 6:                                          # rechtes Bein
            pygame.draw.line(s, red, (cx, body_bottom),
                             (int(cx + u * 1.2), int(body_bottom + u * 1.6)), lw)

    def _draw_word(self, s):
        letters = list(self.word)
        n = len(letters)
        slot = min(int(self._word_font.get_height() * 0.9),
                   (self.width - 40) // n)
        gap = max(6, slot // 5)
        total = n * slot + (n - 1) * gap
        x = (self.width - total) // 2
        reveal_all = (self.state == OVER)
        for ch in letters:
            shown = ch in self.guessed or reveal_all
            # Grundlinie
            pygame.draw.line(s, ui.TEXT_DIM, (x, self.word_y + slot),
                             (x + slot, self.word_y + slot), 3)
            if shown:
                col = COL_WORD if ch in self.guessed else (228, 130, 120)
                img = self._word_font.render(ch, True, col)
                s.blit(img, img.get_rect(center=(x + slot // 2,
                                                 self.word_y + slot // 2)))
            x += slot + gap

    def _draw_keyboard(self, s):
        for ch, rc in self.key_rects.items():
            if ch in self.guessed:
                col = COL_CORRECT if ch in self.word else COL_ABSENT
            else:
                col = COL_KEY
            pygame.draw.rect(s, col, rc, border_radius=6)
            txt = ui.TEXT_FAINT if ch in self.guessed and ch not in self.word \
                else COL_KEY_TEXT
            img = self._key_font.render(ch, True, txt)
            s.blit(img, img.get_rect(center=rc.center))

    def _draw_banner(self, s, title, color, sub):
        w = min(self.width - 40, 500)
        h = 100
        rc = pygame.Rect((self.width - w) // 2, self.word_y - 130, w, h)
        rc.top = max(60, rc.top)
        panel = pygame.Surface((w, h), pygame.SRCALPHA)
        panel.fill((16, 18, 24, 238))
        s.blit(panel, rc.topleft)
        pygame.draw.rect(s, color, rc, 2, border_radius=14)
        img = self._huge.render(title, True, color)
        s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 36)))
        img = self._small.render(sub, True, ui.TEXT_DIM)
        s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 74)))
