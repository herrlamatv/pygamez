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
- Optik: Themen-Hintergrund und ui.*-Palette für das HUD; die Kachelfarben
  bleiben bewusst am Original. Layout/Schriften skalieren mit der Auflösung
  (on_surface_changed).
"""

import random
import pygame

import ui
from game_base import Game, InputEvent
from i18n import t

SIZE = 4                         # 4x4-Raster

# Identitätsfarben des Bretts (bewusst fest, unabhängig vom Theme).
COL_BOARD = (40, 44, 58)
COL_EMPTY = (55, 60, 78)
COL_TILE_TEXT = (235, 235, 240)       # helle Ziffern auf kräftigen Kacheln
COL_TILE_TEXT_DARK = (60, 55, 45)     # dunkle Ziffern auf hellen Kacheln
COL_TILE_BIG = (60, 58, 50)           # Kacheln jenseits von 2048

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

        self._build_layout()

        self.grid = [[0] * SIZE for _ in range(SIZE)]
        self._add_tile()
        self._add_tile()

    # ----- Layout / Theme -----------------------------------------------

    def _build_layout(self):
        """Brett-Geometrie, Schriften und Hintergrund aus width/height ableiten."""
        # Quadratisches Feld, zentriert in der Spielfläche.
        self.board_px = min(self.width, self.height) - 60
        self.pad = 10
        self.cell = (self.board_px - self.pad * (SIZE + 1)) // SIZE
        self.ox = (self.width - self.board_px) // 2
        self.oy = (self.height - self.board_px) // 2 + 10

        h = self.height
        self.font = ui.font(max(16, h // 24))
        self.big_font = ui.font(max(32, h // 10), bold=True)
        # Kachel-Schriften nach Stellenzahl, an die Zellgröße gekoppelt
        # (einmal gecacht statt pro Frame neu erzeugt).
        self._tile_fonts = (
            ui.font(max(18, int(self.cell * 0.40)), bold=True),   # < 100
            ui.font(max(15, int(self.cell * 0.32)), bold=True),   # < 1000
            ui.font(max(12, int(self.cell * 0.26)), bold=True),   # >= 1000
        )

    def on_surface_changed(self):
        """Auflösungswechsel: Geometrie/Schriften/Hintergrund neu, Brett bleibt."""
        self._build_layout()

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
            hatte_2048 = self.won
            if self._move(richtung):
                # Verschmelzung erhöht die Punktzahl -> anderer Klang.
                self.play_sound("merge" if self.score > vorher else "move")
                if self.won and not hatte_2048:
                    # 2048 zum ersten Mal erreicht -> kleine Fanfare.
                    self.play_sound("win")
                    self.rumble(150)
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
        # Themen-Hintergrund (intern gecacht - Sterne/Aurora bleiben lebendig).
        ui.draw_background(s, self.width, self.height)

        # Punkte oben
        s.blit(self.font.render(t("common.points", score=self.score), True, ui.TEXT),
               (12, 8))
        if self.won and not self.game_over:
            hinweis = self.font.render(t("g2048.reached"), True, ui.GOLD)
            s.blit(hinweis, (self.width - hinweis.get_width() - 12, 8))

        # Brett-Hintergrund
        pygame.draw.rect(s, COL_BOARD,
                         (self.ox, self.oy, self.board_px, self.board_px),
                         border_radius=8)

        for r in range(SIZE):
            for c in range(SIZE):
                x = self.ox + self.pad + c * (self.cell + self.pad)
                y = self.oy + self.pad + r * (self.cell + self.pad)
                wert = self.grid[r][c]
                farbe = TILE_COLORS.get(wert, COL_TILE_BIG) if wert else COL_EMPTY
                pygame.draw.rect(s, farbe, (x, y, self.cell, self.cell),
                                 border_radius=6)
                if wert:
                    # dunkler Text für kleine Zahlen, heller für grosse.
                    tcol = COL_TILE_TEXT_DARK if wert <= 4 else COL_TILE_TEXT
                    f = self._tile_fonts[0 if wert < 100 else (1 if wert < 1000 else 2)]
                    img = f.render(str(wert), True, tcol)
                    s.blit(img, img.get_rect(center=(x + self.cell // 2,
                                                     y + self.cell // 2)))

        if self.game_over:
            self._draw_overlay(t("common.game_over"), ui.RED,
                               t("common.enter_restart"))

    def _draw_overlay(self, titel, farbe, hinweis):
        """Endstand-Box: transluzentes Themen-Panel statt Vollbild-Abdunklung."""
        th = self.big_font.get_height()
        hh = self.font.get_height()
        bw = max(self.big_font.size(titel)[0], self.font.size(hinweis)[0]) + 80
        bh = th + hh + 58
        rect = pygame.Rect(0, 0, bw, bh)
        rect.center = (self.width // 2, self.height // 2)

        panel = pygame.Surface(rect.size, pygame.SRCALPHA)
        pygame.draw.rect(panel, (*ui.PANEL[:3], 235), panel.get_rect(), border_radius=16)
        pygame.draw.rect(panel, farbe, panel.get_rect(), 2, border_radius=16)
        self.surface.blit(panel, rect)

        img = self.big_font.render(titel, True, farbe)
        self.surface.blit(img, img.get_rect(midtop=(rect.centerx, rect.y + 20)))
        # Neustart-Hinweis sanft pulsieren lassen.
        hint_col = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.0, 0.2, 1.0))
        img = self.font.render(hinweis, True, hint_col)
        self.surface.blit(img, img.get_rect(midtop=(rect.centerx,
                                                    rect.y + 20 + th + 14)))
