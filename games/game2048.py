# -*- coding: utf-8 -*-
"""
game2048.py
===========
2048 - das Zahlen-Schiebespiel.

- Steuerung: Pfeiltasten (oder WASD) schieben alle Kacheln in eine Richtung.
  Gleiche Zahlen, die zusammenstossen, verschmelzen zur Summe (gibt Punkte).
- Nach jedem gültigen Zug erscheint eine neue Kachel (2 oder 4).
- Ziel: die 2048er-Kachel erreichen. Man kann danach weiterspielen.
- Kein Zug mehr möglich -> Game Over.
- Highscore wird gespeichert.
"""

import random
import pygame

from game_base import Game, InputEvent
from i18n import t

SIZE = 4                         # 4x4-Raster

COL_BG = (18, 20, 28)
COL_BOARD = (40, 44, 58)
COL_EMPTY = (55, 60, 78)
COL_TEXT = (235, 235, 240)
COL_TEXT_DARK = (60, 55, 45)

# Farben je Kachelwert (angelehnt an das Original).
TILE_COLORS = {
    2: (238, 228, 218), 4: (237, 224, 200), 8: (242, 177, 121),
    16: (245, 149, 99), 32: (246, 124, 95), 64: (246, 94, 59),
    128: (237, 207, 114), 256: (237, 204, 97), 512: (237, 200, 80),
    1024: (237, 197, 63), 2048: (237, 194, 46),
}


class Game2048(Game):
    name = "2048"
    highscore_key = "2048"

    def reset(self):
        self.score = 0
        self.game_over = False
        self.won = False

        # Quadratisches Feld, zentriert in der Spielfläche.
        self.board_px = min(self.width, self.height) - 60
        self.pad = 10
        self.cell = (self.board_px - self.pad * (SIZE + 1)) // SIZE
        self.ox = (self.width - self.board_px) // 2
        self.oy = (self.height - self.board_px) // 2 + 10

        self.grid = [[0] * SIZE for _ in range(SIZE)]
        self._add_tile()
        self._add_tile()

    def _add_tile(self):
        """Setzt eine neue Kachel (90% eine 2, 10% eine 4) auf ein freies Feld."""
        frei = [(r, c) for r in range(SIZE) for c in range(SIZE)
                if self.grid[r][c] == 0]
        if not frei:
            return
        r, c = random.choice(frei)
        self.grid[r][c] = 4 if random.random() < 0.1 else 2

    # ----- Eingabe ------------------------------------------------------

    def handle_event(self, event):
        if event.kind != InputEvent.KEYDOWN:
            return

        if self.game_over:
            if event.key in ("Return", "space"):
                self.reset()
            return

        richtung = None
        if self.is_action(event.key, "left"):
            richtung = "L"
        elif self.is_action(event.key, "right"):
            richtung = "R"
        elif self.is_action(event.key, "up"):
            richtung = "U"
        elif self.is_action(event.key, "down"):
            richtung = "D"

        if richtung:
            vorher = self.score
            if self._move(richtung):
                # Verschmelzung erhöht die Punktzahl -> anderer Klang.
                self.play_sound("merge" if self.score > vorher else "move")
                self._add_tile()
                if not self._moves_available():
                    self.game_over = True
                    self.play_sound("gameover")

    def update(self, dt):
        # 2048 ist rundenbasiert; es gibt keine zeitabhängige Logik.
        pass

    # ----- Schiebe-/Verschmelz-Logik ------------------------------------

    def _compress(self, reihe):
        """Schiebt Zahlen einer Reihe nach links und verschmilzt Gleiche."""
        zahlen = [z for z in reihe if z != 0]
        ergebnis = []
        i = 0
        while i < len(zahlen):
            if i + 1 < len(zahlen) and zahlen[i] == zahlen[i + 1]:
                verschmolzen = zahlen[i] * 2
                ergebnis.append(verschmolzen)
                self.score += verschmolzen
                if verschmolzen == 2048:
                    self.won = True
                i += 2
            else:
                ergebnis.append(zahlen[i])
                i += 1
        ergebnis += [0] * (SIZE - len(ergebnis))
        return ergebnis

    def _move(self, richtung):
        """Führt einen Zug aus. Gibt True zurück, wenn sich etwas geändert hat."""
        alt = [row[:] for row in self.grid]

        # Jede Richtung auf "nach links schieben" zurückführen.
        if richtung == "L":
            neu = [self._compress(row) for row in self.grid]
        elif richtung == "R":
            neu = [self._compress(row[::-1])[::-1] for row in self.grid]
        elif richtung == "U":
            spalten = [[self.grid[r][c] for r in range(SIZE)] for c in range(SIZE)]
            gesch = [self._compress(sp) for sp in spalten]
            neu = [[gesch[c][r] for c in range(SIZE)] for r in range(SIZE)]
        else:  # "D"
            spalten = [[self.grid[r][c] for r in range(SIZE)] for c in range(SIZE)]
            gesch = [self._compress(sp[::-1])[::-1] for sp in spalten]
            neu = [[gesch[c][r] for c in range(SIZE)] for r in range(SIZE)]

        self.grid = neu
        return neu != alt

    def _moves_available(self):
        """True, solange ein Zug möglich ist (freies Feld oder Nachbarn gleich)."""
        for r in range(SIZE):
            for c in range(SIZE):
                if self.grid[r][c] == 0:
                    return True
                if c + 1 < SIZE and self.grid[r][c] == self.grid[r][c + 1]:
                    return True
                if r + 1 < SIZE and self.grid[r][c] == self.grid[r + 1][c]:
                    return True
        return False

    # ----- Zeichnen -----------------------------------------------------

    def draw(self):
        s = self.surface
        s.fill(COL_BG)

        # Punkte oben
        s.blit(self.font.render(t("common.points", score=self.score), True, COL_TEXT),
               (10, 8))
        if self.won and not self.game_over:
            hinweis = self.font.render(t("g2048.reached"), True, (240, 200, 90))
            s.blit(hinweis, (self.width - hinweis.get_width() - 10, 8))

        # Brett-Hintergrund
        pygame.draw.rect(s, COL_BOARD,
                         (self.ox, self.oy, self.board_px, self.board_px),
                         border_radius=8)

        for r in range(SIZE):
            for c in range(SIZE):
                x = self.ox + self.pad + c * (self.cell + self.pad)
                y = self.oy + self.pad + r * (self.cell + self.pad)
                wert = self.grid[r][c]
                farbe = TILE_COLORS.get(wert, (60, 58, 50)) if wert else COL_EMPTY
                pygame.draw.rect(s, farbe, (x, y, self.cell, self.cell),
                                 border_radius=6)
                if wert:
                    # dunkler Text für kleine Zahlen, heller für grosse.
                    tcol = COL_TEXT_DARK if wert <= 4 else COL_TEXT
                    größe = 40 if wert < 100 else (32 if wert < 1000 else 26)
                    f = pygame.font.SysFont("consolas", größe, bold=True)
                    img = f.render(str(wert), True, tcol)
                    s.blit(img, img.get_rect(center=(x + self.cell // 2,
                                                     y + self.cell // 2)))

        if self.game_over:
            overlay = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
            overlay.fill((0, 0, 0, 150))
            s.blit(overlay, (0, 0))
            self.draw_center_text(t("common.game_over"), self.big_font,
                                  (235, 110, 110), -20)
            self.draw_center_text(t("common.enter_restart"), self.font, COL_TEXT, 30)
