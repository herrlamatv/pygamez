# -*- coding: utf-8 -*-
"""
tetris.py
=========
Tetris - Einzelspieler oder Versus (2 Spieler nebeneinander).

- Steuerung über die belegten Tasten (Standard: P1 = WASD + Leertaste,
  P2 = Pfeile + Enter): links/rechts verschieben, hoch = drehen,
  runter = Soft-Drop, Aktion = Hard-Drop.
- Volle Reihen lösen sich auf und geben Punkte; alle 10 Reihen steigt das Level.
- Versus: Wessen Stapel zuerst oben anstößt, verliert - der andere gewinnt.
- Highscore wird gespeichert.
"""

import random
import pygame

from game_base import Game, InputEvent

COLS = 10
ROWS = 20

COL_BG = (12, 14, 22)
COL_GRID = (26, 28, 40)
COL_FRAME = (70, 80, 110)
COL_TEXT = (230, 230, 235)
COL_GHOST = (60, 64, 82)

SHAPES = {
    "I": [[(0, 1), (1, 1), (2, 1), (3, 1)],
          [(2, 0), (2, 1), (2, 2), (2, 3)]],
    "O": [[(1, 0), (2, 0), (1, 1), (2, 1)]],
    "T": [[(1, 0), (0, 1), (1, 1), (2, 1)],
          [(1, 0), (1, 1), (2, 1), (1, 2)],
          [(0, 1), (1, 1), (2, 1), (1, 2)],
          [(1, 0), (0, 1), (1, 1), (1, 2)]],
    "S": [[(1, 0), (2, 0), (0, 1), (1, 1)],
          [(1, 0), (1, 1), (2, 1), (2, 2)]],
    "Z": [[(0, 0), (1, 0), (1, 1), (2, 1)],
          [(2, 0), (1, 1), (2, 1), (1, 2)]],
    "J": [[(0, 0), (0, 1), (1, 1), (2, 1)],
          [(1, 0), (2, 0), (1, 1), (1, 2)],
          [(0, 1), (1, 1), (2, 1), (2, 2)],
          [(1, 0), (1, 1), (0, 2), (1, 2)]],
    "L": [[(2, 0), (0, 1), (1, 1), (2, 1)],
          [(1, 0), (1, 1), (1, 2), (2, 2)],
          [(0, 1), (1, 1), (2, 1), (0, 2)],
          [(0, 0), (1, 0), (1, 1), (1, 2)]],
}

COLORS = {
    "I": (80, 210, 220), "O": (240, 220, 90), "T": (190, 110, 220),
    "S": (110, 220, 120), "Z": (235, 100, 100), "J": (100, 130, 230),
    "L": (240, 160, 80),
}


class _Board:
    """Ein einzelnes Tetris-Spielfeld mit eigenem Stein, Punkten und Level."""

    def __init__(self, game):
        self.game = game            # für Sound/Haptik-Rückrufe
        self.grid = [[None] * COLS for _ in range(ROWS)]
        self.level = 1
        self.lines = 0
        self.score = 0
        self.dead = False
        self._fall_timer = 0.0
        self._bag = []
        self.next_kind = self._bag_next()
        self._spawn()

    # ----- Steine -------------------------------------------------------

    def _bag_next(self):
        if not self._bag:
            self._bag = list(SHAPES.keys())
            random.shuffle(self._bag)
        return self._bag.pop()

    def _spawn(self):
        self.kind = self.next_kind
        self.next_kind = self._bag_next()
        self.rot = 0
        self.px = COLS // 2 - 2
        self.py = -1 if self.kind == "I" else 0
        if self._collision(self.px, self.py, self.rot):
            self.dead = True

    def _cells(self, px, py, rot):
        form = SHAPES[self.kind]
        for cx, cy in form[rot % len(form)]:
            yield px + cx, py + cy

    def _collision(self, px, py, rot):
        for x, y in self._cells(px, py, rot):
            if x < 0 or x >= COLS or y >= ROWS:
                return True
            if y >= 0 and self.grid[y][x] is not None:
                return True
        return False

    def cur_cells(self):
        return list(self._cells(self.px, self.py, self.rot))

    def ghost_cells(self):
        gy = self.py
        while not self._collision(self.px, gy + 1, self.rot):
            gy += 1
        return list(self._cells(self.px, gy, self.rot))

    # ----- Eingaben -----------------------------------------------------

    def move(self, dx):
        if not self.dead and not self._collision(self.px + dx, self.py, self.rot):
            self.px += dx
            self.game.play_sound("move")

    def rotate(self):
        if self.dead:
            return
        neu = self.rot + 1
        for dx in (0, -1, 1, -2, 2):
            if not self._collision(self.px + dx, self.py, neu):
                self.px += dx
                self.rot = neu
                self.game.play_sound("rotate")
                return

    def soft(self):
        if self.dead:
            return
        if not self._collision(self.px, self.py + 1, self.rot):
            self.py += 1
            self.score += 1
        else:
            self._lock()

    def hard(self):
        if self.dead:
            return
        while not self._collision(self.px, self.py + 1, self.rot):
            self.py += 1
            self.score += 2
        self._lock()

    # ----- Logik --------------------------------------------------------

    def update(self, dt):
        if self.dead:
            return
        interval = max(0.05, 0.55 - (self.level - 1) * 0.045)
        self._fall_timer += dt
        if self._fall_timer < interval:
            return
        self._fall_timer = 0.0
        if not self._collision(self.px, self.py + 1, self.rot):
            self.py += 1
        else:
            self._lock()

    def _lock(self):
        farbe = COLORS[self.kind]
        for x, y in self._cells(self.px, self.py, self.rot):
            if y >= 0:
                self.grid[y][x] = farbe
        self.game.play_sound("lock")
        self._clear_lines()
        self._spawn()

    def _clear_lines(self):
        behalten = [row for row in self.grid if any(c is None for c in row)]
        entfernt = ROWS - len(behalten)
        if entfernt:
            self.grid = [[None] * COLS for _ in range(entfernt)] + behalten
            self.lines += entfernt
            punkte = {1: 40, 2: 100, 3: 300, 4: 1200}.get(entfernt, 0)
            self.score += punkte * self.level
            self.level = 1 + self.lines // 10
            self.game.play_sound("line")
            self.game.rumble(120)


class TetrisGame(Game):
    name = "Tetris"
    highscore_key = "tetris"
    supports_multiplayer = True

    def reset(self):
        self.score = 0
        self.game_over = False
        self.winner = None

        if self.multiplayer:
            self.boards = [_Board(self), _Board(self)]
            # Zwei Felder nebeneinander (kleinere Zellen), Punkte darüber.
            self.cell = min((self.height - 70) // ROWS,
                            (self.width - 80 - 40) // (2 * COLS))
            bw = self.cell * COLS
            gap = 40
            gesamt = 2 * bw + gap
            x0 = (self.width - gesamt) // 2
            oy = (self.height - self.cell * ROWS) // 2 + 16
            self.layout = [(x0, oy), (x0 + bw + gap, oy)]
        else:
            self.boards = [_Board(self)]
            self.cell = min((self.height - 40) // ROWS, (self.width - 200) // COLS)
            oy = (self.height - self.cell * ROWS) // 2
            self.layout = [(24, oy)]

    # ----- Eingabe ------------------------------------------------------

    def handle_event(self, event):
        if event.kind != InputEvent.KEYDOWN:
            return

        if self.game_over:
            if event.key in ("Return", "space"):
                self.reset()
            return

        if self.multiplayer:
            self._input(self.boards[0], event.key, "p1")
            self._input(self.boards[1], event.key, "p2")
        else:
            self._input(self.boards[0], event.key, None)

    def _input(self, board, key, player):
        if self.is_action(key, "left", player):
            board.move(-1)
        elif self.is_action(key, "right", player):
            board.move(1)
        elif self.is_action(key, "up", player):
            board.rotate()
        elif self.is_action(key, "down", player):
            board.soft()
        elif self.is_action(key, "action", player):
            board.hard()

    # ----- Logik --------------------------------------------------------

    def update(self, dt):
        if self.game_over:
            return
        for b in self.boards:
            b.update(dt)

        self.score = max(b.score for b in self.boards)

        tote = [i for i, b in enumerate(self.boards) if b.dead]
        if tote:
            self.game_over = True
            self.play_sound("gameover")
            self.rumble(200)
            if self.multiplayer:
                lebende = [i for i in range(len(self.boards)) if i not in tote]
                if len(lebende) == 1:
                    self.winner = lebende[0]
                else:
                    s0, s1 = self.boards[0].score, self.boards[1].score
                    self.winner = 0 if s0 > s1 else (1 if s1 > s0 else None)

    # ----- Zeichnen -----------------------------------------------------

    def _draw_cell(self, ox, oy, cx, cy, farbe):
        r = pygame.Rect(ox + cx * self.cell, oy + cy * self.cell,
                        self.cell, self.cell)
        pygame.draw.rect(self.surface, farbe, r.inflate(-2, -2), border_radius=3)

    def _draw_board(self, b, ox, oy):
        s = self.surface
        bw = self.cell * COLS
        bh = self.cell * ROWS

        for cx in range(COLS + 1):
            x = ox + cx * self.cell
            pygame.draw.line(s, COL_GRID, (x, oy), (x, oy + bh))
        for cy in range(ROWS + 1):
            y = oy + cy * self.cell
            pygame.draw.line(s, COL_GRID, (ox, y), (ox + bw, y))

        for cy in range(ROWS):
            for cx in range(COLS):
                if b.grid[cy][cx] is not None:
                    self._draw_cell(ox, oy, cx, cy, b.grid[cy][cx])

        if not b.dead:
            for x, y in b.ghost_cells():
                if y >= 0:
                    self._draw_cell(ox, oy, x, y, COL_GHOST)
            farbe = COLORS[b.kind]
            for x, y in b.cur_cells():
                if y >= 0:
                    self._draw_cell(ox, oy, x, y, farbe)

        pygame.draw.rect(s, COL_FRAME, (ox, oy, bw, bh), 2)

    def draw(self):
        s = self.surface
        s.fill(COL_BG)

        for i, b in enumerate(self.boards):
            ox, oy = self.layout[i]
            self._draw_board(b, ox, oy)

            if self.multiplayer:
                label = f"P{i + 1}  {b.score}"
                farbe = COLORS["S"] if i == 0 else COLORS["I"]
                img = self.font.render(label, True, farbe)
                s.blit(img, (ox, oy - 26))
            else:
                self._draw_info(b, ox + self.cell * COLS + 24, oy)

        if self.game_over:
            if self.multiplayer:
                if self.winner is None:
                    text, farbe = "UNENTSCHIEDEN", COL_TEXT
                else:
                    text = f"SPIELER {self.winner + 1} GEWINNT"
                    farbe = COLORS["S"] if self.winner == 0 else COLORS["I"]
                self.draw_center_text(text, self.big_font, farbe, -20)
            else:
                self.draw_center_text("GAME OVER", self.big_font, (235, 110, 110), -20)
            self.draw_center_text("Enter = Neustart", self.font, COL_TEXT, 30)

    def _draw_info(self, b, info_x, oy):
        s = self.surface
        s.blit(self.font.render("Punkte", True, COL_TEXT), (info_x, oy))
        s.blit(self.big_font.render(str(b.score), True, COL_TEXT), (info_x, oy + 26))
        s.blit(self.font.render(f"Level: {b.level}", True, COL_TEXT), (info_x, oy + 90))
        s.blit(self.font.render(f"Reihen: {b.lines}", True, COL_TEXT),
               (info_x, oy + 120))
        s.blit(self.font.render("Nächster", True, COL_TEXT), (info_x, oy + 170))
        for cx, cy in SHAPES[b.next_kind][0]:
            r = pygame.Rect(info_x + cx * self.cell, oy + 200 + cy * self.cell,
                            self.cell, self.cell)
            pygame.draw.rect(s, COLORS[b.next_kind], r.inflate(-2, -2), border_radius=3)
