# -*- coding: utf-8 -*-
"""
slidepuzzle.py
==============
Schiebepuzzle (15-Puzzle) - bring die durchnummerierten Kacheln in die richtige
Reihenfolge, indem du sie in die freie Lücke schiebst.

- Drei Größen (Modi): 3x3 (leicht), 4x4 (klassisch) und 5x5 (schwer).
- Gemischt wird durch viele zufällige, GÜLTIGE Züge ausgehend vom gelösten Feld -
  so ist das Puzzle immer lösbar (kein Paritätsproblem).
- Steuerung: Klick auf eine Kachel in derselben Reihe/Spalte wie die Lücke schiebt
  die ganze Linie; Pfeiltasten schieben die an die Lücke grenzende Kachel hinein.
- Punkte (höher = besser): eine Grundpunktzahl je Größe, von der Züge und Zeit
  abgezogen werden - schnell und mit wenigen Zügen lösen lohnt sich.

Nach dem Lösen speichert main.py automatisch den Highscore; Enter/Klick mischt neu.
"""

import random

import pygame

import ui
from game_base import Game, InputEvent, LocalizedName
from i18n import t

COL_BG = (16, 19, 30)

# Grundpunktzahl und Abzüge je Kantenlänge N.
BASE = {3: 2000, 4: 6000, 5: 12000}
MOVE_PEN = {3: 8, 4: 6, 5: 4}
TIME_PEN = 5

PLAY, SOLVED = "play", "solved"


class SlidingPuzzleGame(Game):
    name = LocalizedName("Sliding Puzzle", de="Schiebepuzzle", fr="Taquin",
                         es="Puzle deslizante", pt="Quebra-cabeça deslizante")
    highscore_key = "slide"
    supports_multiplayer = False

    # Größen-Modi (werden im Vorspiel-Screen als Buttons angeboten).
    MODES = [("3", "slide.mode.3"), ("4", "slide.mode.4"), ("5", "slide.mode.5")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.N = {"3": 3, "4": 4, "5": 5}.get(self.mode, 4)
        self.score = 0
        self.game_over = False
        self.state = PLAY
        self.moves = 0
        self.elapsed = 0.0
        self.flash = {}                 # index -> restliche Aufleucht-Zeit
        self._make_fonts()
        self._new_board()
        self._layout()

    def _make_fonts(self):
        self._tile_font = ui.font(max(22, self.height // 12), bold=True)
        self._hud = ui.font(20, bold=True)
        self._small = ui.font(15)
        self._huge = ui.font(max(30, self.height // 11), bold=True)

    def on_surface_changed(self):
        self._make_fonts()
        self._layout()

    def _new_board(self):
        n = self.N
        # 0 = Lücke; gelöst: [1,2,...,n*n-1, 0]
        self.solved = list(range(1, n * n)) + [0]
        self.board = list(self.solved)
        self.blank = n * n - 1
        self._scramble()
        self.moves = 0
        self.elapsed = 0.0
        self.state = PLAY
        self.flash = {}

    def _scramble(self):
        """Mischt durch viele zufällige gültige Züge (immer lösbar)."""
        n = self.N
        last = -1
        for _ in range(n * n * 60):
            nb = [i for i in self._neighbors(self.blank) if i != last]
            pick = random.choice(nb)
            last = self.blank
            self.board[self.blank], self.board[pick] = \
                self.board[pick], self.board[self.blank]
            self.blank = pick
        if self.board == self.solved:       # extrem unwahrscheinlich
            self._scramble()

    def _neighbors(self, idx):
        n = self.N
        r, c = divmod(idx, n)
        out = []
        if r > 0:
            out.append(idx - n)
        if r < n - 1:
            out.append(idx + n)
        if c > 0:
            out.append(idx - 1)
        if c < n - 1:
            out.append(idx + 1)
        return out

    def _layout(self):
        n = self.N
        top = 74
        bottom = self.height - 40
        avail = min(self.width - 48, bottom - top)
        self.board_px = max(120, avail)
        self.gap = max(3, self.board_px // (n * 22))
        self.tile = (self.board_px - (n + 1) * self.gap) // n
        self.board_px = n * self.tile + (n + 1) * self.gap
        self.ox = (self.width - self.board_px) // 2
        self.oy = top + max(0, (bottom - top - self.board_px) // 2)

    def _tile_rect(self, idx):
        r, c = divmod(idx, self.N)
        x = self.ox + self.gap + c * (self.tile + self.gap)
        y = self.oy + self.gap + r * (self.tile + self.gap)
        return pygame.Rect(x, y, self.tile, self.tile)

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SOLVED:
            if self._is_continue(event):
                self.game_over = False
                self._new_board()
                self.play_sound("click")
            return
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("Up", "w", "W"):
                self._slide_from(self.blank + self.N)     # Kachel unten rückt hoch
            elif k in ("Down", "s", "S"):
                self._slide_from(self.blank - self.N)
            elif k in ("Left", "a", "A"):
                self._slide_from(self.blank + 1)          # Kachel rechts rückt links
            elif k in ("Right", "d", "D"):
                self._slide_from(self.blank - 1)
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            self._click(event.pos)

    def _is_continue(self, event):
        return (event.kind == InputEvent.MOUSEDOWN
                or (event.kind == InputEvent.KEYDOWN
                    and event.key in ("Return", "space")))

    def _click(self, pos):
        for idx in range(self.N * self.N):
            if idx != self.blank and self._tile_rect(idx).collidepoint(pos):
                self._slide_line(idx)
                return

    def _slide_from(self, idx):
        """Schiebt die Kachel an Position idx in die Lücke (nur wenn benachbart)."""
        n = self.N
        if 0 <= idx < n * n and idx in self._neighbors(self.blank):
            self._swap(idx)

    def _slide_line(self, idx):
        """Klick: schiebt die ganze Linie zwischen Lücke und angeklickter Kachel."""
        n = self.N
        br, bc = divmod(self.blank, n)
        cr, cc = divmod(idx, n)
        if cr == br:
            step = 1 if cc > bc else -1
            while self.blank != idx:
                self._swap(self.blank + step)
        elif cc == bc:
            step = n if cr > br else -n
            while self.blank != idx:
                self._swap(self.blank + step)
        else:
            self.play_sound("click")     # keine gültige Linie

    def _swap(self, idx):
        """Vertauscht Lücke mit Kachel idx (idx muss benachbart sein)."""
        self.board[self.blank], self.board[idx] = \
            self.board[idx], self.board[self.blank]
        self.flash[self.blank] = 0.18        # Zielfeld leuchtet kurz
        self.blank = idx
        self.moves += 1
        self.play_sound("move")
        if self.board == self.solved:
            self._win()

    def _win(self):
        n = self.N
        pts = max(50, BASE.get(n, 6000) - self.moves * MOVE_PEN.get(n, 6)
                  - int(self.elapsed) * TIME_PEN)
        self.score = pts
        self.state = SOLVED
        self.game_over = True                # main.py speichert den Highscore
        self.play_sound("win")

    # ===================================================== Update
    def update(self, dt):
        if self.state == PLAY:
            self.elapsed += dt
        for idx in list(self.flash):
            self.flash[idx] -= dt
            if self.flash[idx] <= 0:
                del self.flash[idx]

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height, stars=False, aurora=True)
        self._draw_hud(s)
        self._draw_board(s)
        if self.state == SOLVED:
            self._draw_banner(s)

    def _draw_hud(self, s):
        img = self._hud.render(t("slide.title"), True, self.accent)
        s.blit(img, img.get_rect(midleft=(20, 30)))
        img = self._small.render(t("slide.moves", n=self.moves), True, ui.TEXT)
        s.blit(img, img.get_rect(midright=(self.width - 20, 22)))
        img = self._small.render(t("slide.time", t=self._fmt_time()), True,
                                 ui.TEXT_DIM)
        s.blit(img, img.get_rect(midright=(self.width - 20, 44)))
        img = self._small.render(t("slide.hint"), True, ui.TEXT_FAINT)
        s.blit(img, img.get_rect(center=(self.width // 2, 52)))

    def _fmt_time(self):
        sec = int(self.elapsed)
        return "%d:%02d" % (sec // 60, sec % 60)

    def _draw_board(self, s):
        # Rahmen/Panel hinter dem Feld
        pad = self.gap
        panel = pygame.Rect(self.ox - pad, self.oy - pad,
                            self.board_px + 2 * pad, self.board_px + 2 * pad)
        pygame.draw.rect(s, (24, 28, 42), panel, border_radius=12)
        pygame.draw.rect(s, ui.mix(self.accent, (20, 24, 38), 0.6), panel, 2,
                         border_radius=12)
        for idx in range(self.N * self.N):
            val = self.board[idx]
            if val == 0:
                continue
            rc = self._tile_rect(idx)
            base = ui.mix(self.accent, (235, 240, 250), 0.12)
            if idx in self.flash:
                base = ui.mix(base, (255, 255, 255),
                              min(1.0, self.flash[idx] / 0.18))
            correct = (val == self.solved[idx])
            fill = base if correct else ui.mix(base, (40, 46, 66), 0.42)
            pygame.draw.rect(s, fill, rc, border_radius=8)
            pygame.draw.rect(s, ui.mix(fill, (255, 255, 255), 0.18), rc, 2,
                             border_radius=8)
            col = (20, 24, 34) if correct else ui.TEXT
            img = self._tile_font.render(str(val), True, col)
            s.blit(img, img.get_rect(center=rc.center))

    def _draw_banner(self, s):
        w = min(self.width - 40, 460)
        h = 118
        rc = pygame.Rect((self.width - w) // 2, (self.height - h) // 2, w, h)
        panel = pygame.Surface((w, h), pygame.SRCALPHA)
        panel.fill((16, 18, 24, 236))
        s.blit(panel, rc.topleft)
        pygame.draw.rect(s, self.accent, rc, 2, border_radius=14)
        img = self._huge.render(t("slide.solved"), True, self.accent)
        s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 38)))
        img = self._hud.render(t("slide.score", n=self.score), True, ui.GOLD)
        s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 74)))
        img = self._small.render(t("common.enter_restart"), True, ui.TEXT_DIM)
        s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 100)))
