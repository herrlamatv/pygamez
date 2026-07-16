# -*- coding: utf-8 -*-
"""
bubbleshooter.py
================
Bubble Shooter (Puzzle Bobble) - schiesse Kugeln nach oben und bilde Gruppen aus
mindestens drei gleichen Farben, damit sie platzen.

- Das Feld ist ein versetztes Wabenraster (gerade Reihen bündig links, ungerade
  um einen halben Kugelradius nach rechts versetzt). Jede Kugel hat sechs
  Nachbarn.
- Die Kanone unten zielt zur Maus (oder mit Pfeil links/rechts); Klick bzw.
  Leertaste schiesst. Die Kugel prallt an den Seitenwänden ab und rastet beim
  Treffer in die nächste freie Wabe ein.
- Nach dem Einrasten platzt eine gleichfarbige Gruppe ab drei Kugeln; danach
  fallen alle Kugeln, die dadurch den Halt zur Decke verlieren (Bonuspunkte).
- Drei Modi: Leicht (4 Farben, keine neuen Reihen), Klassik (5 Farben, ab und zu
  rückt oben eine Reihe nach) und Schwer (6 Farben, schneller).

Game Over, sobald eine Kugel zu tief einrastet. Punkte zählt main.py als Highscore.
"""

import math
import random

import pygame

import ui
from game_base import Game, InputEvent
from i18n import t

COL_BG = (16, 18, 28)

BUBBLE_COLORS = [
    (230, 72, 72), (240, 205, 70), (96, 200, 112),
    (80, 140, 240), (175, 110, 235), (240, 150, 52),
]

# (Farbanzahl, Startreihen, Schuss-Abstand zwischen neuen Reihen; 0 = nie).
MODE_CFG = {"easy": (4, 4, 0), "classic": (5, 5, 8), "hard": (6, 6, 6)}

COLS = 12
MAX_ROWS = 40
SPEED = 820.0

PLAY, OVER = "play", "over"


class BubbleShooterGame(Game):
    name = "Bubble Shooter"
    highscore_key = "bubble"
    supports_multiplayer = False

    MODES = [("easy", "bub.mode.easy"), ("classic", "bub.mode.classic"),
             ("hard", "bub.mode.hard")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.ncolors, self.start_rows, self.drop_every = \
            MODE_CFG.get(self.mode, MODE_CFG["classic"])
        self.accent = ui.game_color(type(self).__name__)
        self.score = 0
        self.game_over = False
        self.state = PLAY
        self._make_fonts()
        self._layout()
        self._new_board()

    def _make_fonts(self):
        self._hud = ui.font(20, bold=True)
        self._small = ui.font(15)
        self._huge = ui.font(max(30, self.height // 11), bold=True)

    def on_surface_changed(self):
        self._make_fonts()
        self._layout()
        # Kugeln an das neue Raster anpassen: neu einrasten ist unnötig, da wir
        # in Rasterkoordinaten speichern und beim Zeichnen umrechnen.

    def _layout(self):
        margin = 14
        self.top = 54
        play_w = self.width - 2 * margin
        self.r = min(28, play_w / (2 * COLS + 1))
        self.row_h = self.r * math.sqrt(3)
        board_w = 2 * self.r * COLS + self.r
        self.bx = (self.width - board_w) / 2.0
        self.left = self.bx
        self.right = self.bx + board_w
        self.cannon = (self.width / 2.0, self.height - 30.0)
        self.death_y = self.cannon[1] - 2 * self.r - 26

    def _new_board(self):
        self.grid = []                    # Liste von Reihen; je Reihe COLS Zellen
        self.drop_offset = 0
        for _ in range(self.start_rows):
            self.grid.append([random.randrange(self.ncolors)
                              for _ in range(COLS)])
        self.shots = 0
        self.shot = None                  # fliegende Kugel oder None
        self.aim = -math.pi / 2           # zeigt nach oben
        self.cur = self._pick_color()
        self.nxt = self._pick_color()
        self.state = PLAY

    def _pick_color(self):
        present = {c for row in self.grid for c in row if c is not None}
        if present:
            return random.choice(list(present))
        return random.randrange(self.ncolors)

    # ===================================================== Rasterhelfer
    def _parity(self, row):
        return (row + self.drop_offset) % 2

    def _center(self, row, col):
        x = self.bx + self.r + 2 * self.r * col + (self.r if self._parity(row) else 0)
        y = self.top + self.r + row * self.row_h
        return x, y

    def _filled(self, row, col):
        return (0 <= row < len(self.grid) and 0 <= col < COLS
                and self.grid[row][col] is not None)

    def _neighbors(self, row, col):
        if self._parity(row) == 0:
            offs = ((0, -1), (0, 1), (-1, -1), (-1, 0), (1, -1), (1, 0))
        else:
            offs = ((0, -1), (0, 1), (-1, 0), (-1, 1), (1, 0), (1, 1))
        out = []
        for dr, dc in offs:
            rr, cc = row + dr, col + dc
            if rr >= 0 and 0 <= cc < COLS:
                out.append((rr, cc))
        return out

    def _cell_from_point(self, x, y):
        row = int(round((y - self.top - self.r) / self.row_h))
        row = max(0, row)
        p = (row + self.drop_offset) % 2
        col = int(round((x - self.bx - self.r - (self.r if p else 0))
                        / (2 * self.r)))
        col = max(0, min(COLS - 1, col))
        return row, col

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == OVER:
            if self._is_continue(event):
                self.game_over = False
                self.reset()
            return
        if event.kind == InputEvent.MOUSEMOVE:
            self._aim_at(event.pos)
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            self._aim_at(event.pos)
            self._shoot()
        elif event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("space", "Up", "w", "W"):
                self._shoot()
            elif k in ("Left", "a", "A"):
                self.aim = max(-math.pi + 0.28, self.aim - 0.08)
            elif k in ("Right", "d", "D"):
                self.aim = min(-0.28, self.aim + 0.08)

    def _is_continue(self, event):
        return (event.kind == InputEvent.MOUSEDOWN
                or (event.kind == InputEvent.KEYDOWN
                    and event.key in ("Return", "space")))

    def _aim_at(self, pos):
        dx = pos[0] - self.cannon[0]
        dy = pos[1] - self.cannon[1]
        if dy > -1:
            dy = -1
        ang = math.atan2(dy, dx)
        self.aim = max(-math.pi + 0.28, min(-0.28, ang))

    def _shoot(self):
        if self.shot is not None or self.state != PLAY:
            return
        self.shot = {"x": self.cannon[0], "y": self.cannon[1],
                     "vx": math.cos(self.aim) * SPEED,
                     "vy": math.sin(self.aim) * SPEED, "c": self.cur}
        self.play_sound("move")

    # ===================================================== Update
    def update(self, dt):
        if self.state != PLAY or self.shot is None:
            return
        dt = min(dt, 0.05)
        # Substeps gegen Durchtunneln
        steps = max(1, int(SPEED * dt / (self.r * 0.5)) + 1)
        sdt = dt / steps
        for _ in range(steps):
            if self._advance(sdt):
                break

    def _advance(self, dt):
        sh = self.shot
        sh["x"] += sh["vx"] * dt
        sh["y"] += sh["vy"] * dt
        # Wandreflexion
        if sh["x"] <= self.left + self.r:
            sh["x"] = self.left + self.r
            sh["vx"] = abs(sh["vx"])
        elif sh["x"] >= self.right - self.r:
            sh["x"] = self.right - self.r
            sh["vx"] = -abs(sh["vx"])
        # Decke
        if sh["y"] <= self.top + self.r:
            self._place(sh["x"], sh["y"], sh["c"])
            return True
        # Kollision mit vorhandenen Kugeln
        hit = 2 * self.r * 0.86
        for row in range(len(self.grid)):
            for col in range(COLS):
                if self.grid[row][col] is None:
                    continue
                cx, cy = self._center(row, col)
                if (sh["x"] - cx) ** 2 + (sh["y"] - cy) ** 2 <= hit * hit:
                    self._place(sh["x"], sh["y"], sh["c"])
                    return True
        return False

    def _place(self, x, y, color):
        self.shot = None
        self.shots += 1
        cell = self._snap_cell(x, y)
        if cell is None:
            return
        row, col = cell
        while len(self.grid) <= row:
            self.grid.append([None] * COLS)
        self.grid[row][col] = color
        self.play_sound("click")

        cluster = self._same_color_cluster(row, col)
        if len(cluster) >= 3:
            for (r, c) in cluster:
                self.grid[r][c] = None
            self.score += len(cluster) * 10
            self.play_sound("win")
            dropped = self._drop_floating()
            if dropped:
                self.score += dropped * 20
        # Feld leer? Bonus + Nachfüllen
        if not any(c is not None for rr in self.grid for c in rr):
            self.score += 500
            self.play_sound("win")
            self._new_rows_top(self.start_rows)

        # Neue Reihe von oben (Modus-abhängig)
        if self.drop_every and self.shots % self.drop_every == 0:
            self._new_rows_top(1)

        self.cur = self.nxt
        self.nxt = self._pick_color()
        self._check_over()

    def _snap_cell(self, x, y):
        r0, c0 = self._cell_from_point(x, y)
        best, bestd = None, 1e18
        for rr in range(max(0, r0 - 1), r0 + 3):
            for cc in range(COLS):
                if self._filled(rr, cc):
                    continue
                attached = (rr == 0) or any(self._filled(nr, nc)
                                            for nr, nc in self._neighbors(rr, cc))
                if not attached:
                    continue
                cx, cy = self._center(rr, cc)
                d = (x - cx) ** 2 + (y - cy) ** 2
                if d < bestd:
                    best, bestd = (rr, cc), d
        if best is None:                  # Notfall: irgendeine freie Wabe
            best = (r0, c0)
        return best

    def _same_color_cluster(self, row, col):
        color = self.grid[row][col]
        seen = {(row, col)}
        stack = [(row, col)]
        while stack:
            r, c = stack.pop()
            for nr, nc in self._neighbors(r, c):
                if (nr, nc) not in seen and self._filled(nr, nc) \
                        and self.grid[nr][nc] == color:
                    seen.add((nr, nc))
                    stack.append((nr, nc))
        return seen

    def _drop_floating(self):
        """Entfernt alle Kugeln ohne Verbindung zur obersten Reihe (Reihe 0)."""
        anchored = set()
        stack = [(0, c) for c in range(COLS) if self._filled(0, c)]
        anchored.update(stack)
        while stack:
            r, c = stack.pop()
            for nr, nc in self._neighbors(r, c):
                if (nr, nc) not in anchored and self._filled(nr, nc):
                    anchored.add((nr, nc))
                    stack.append((nr, nc))
        dropped = 0
        for r in range(len(self.grid)):
            for c in range(COLS):
                if self._filled(r, c) and (r, c) not in anchored:
                    self.grid[r][c] = None
                    dropped += 1
        return dropped

    def _new_rows_top(self, n):
        for _ in range(n):
            self.grid.insert(0, [random.randrange(self.ncolors)
                                 for _ in range(COLS)])
            self.drop_offset += 1

    def _check_over(self):
        for r in range(len(self.grid)):
            for c in range(COLS):
                if self.grid[r][c] is not None:
                    _, cy = self._center(r, c)
                    if cy + self.r >= self.death_y:
                        self.state = OVER
                        self.game_over = True   # main.py speichert Highscore
                        self.play_sound("gameover")
                        return

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height, stars=False, aurora=True)
        self._draw_death_line(s)
        self._draw_grid(s)
        if self.state == PLAY:
            self._draw_aim(s)
        self._draw_cannon(s)
        self._draw_hud(s)
        if self.state == OVER:
            self._draw_banner(s)

    def _bubble(self, s, x, y, color, radius=None):
        rad = int(radius or self.r)
        x, y = int(x), int(y)
        pygame.draw.circle(s, color, (x, y), rad)
        pygame.draw.circle(s, ui.mix(color, (255, 255, 255), 0.35), (x, y), rad, 2)
        pygame.draw.circle(s, ui.mix(color, (255, 255, 255), 0.55),
                           (x - rad // 3, y - rad // 3), max(2, rad // 4))

    def _draw_grid(self, s):
        for r in range(len(self.grid)):
            for c in range(COLS):
                col = self.grid[r][c]
                if col is None:
                    continue
                cx, cy = self._center(r, c)
                self._bubble(s, cx, cy, BUBBLE_COLORS[col])

    def _draw_death_line(self, s):
        y = int(self.death_y)
        for x in range(int(self.left), int(self.right), 16):
            pygame.draw.line(s, (120, 60, 70), (x, y), (x + 8, y), 2)

    def _draw_aim(self, s):
        for i, (px, py) in enumerate(self._trace_aim()):
            if i % 2 == 0:
                pygame.draw.circle(s, ui.mix(self.accent, (255, 255, 255), 0.2),
                                   (int(px), int(py)), 3)

    def _trace_aim(self):
        x, y = self.cannon
        vx, vy = math.cos(self.aim), math.sin(self.aim)
        pts = []
        step = self.r * 0.7
        for _ in range(140):
            x += vx * step
            y += vy * step
            if x <= self.left + self.r:
                x = self.left + self.r
                vx = abs(vx)
            elif x >= self.right - self.r:
                x = self.right - self.r
                vx = -abs(vx)
            if y <= self.top + self.r:
                break
            stop = False
            for row in range(len(self.grid)):
                for col in range(COLS):
                    if self.grid[row][col] is None:
                        continue
                    cx, cy = self._center(row, col)
                    if (x - cx) ** 2 + (y - cy) ** 2 <= (2 * self.r * 0.86) ** 2:
                        stop = True
                        break
                if stop:
                    break
            pts.append((x, y))
            if stop:
                break
        return pts

    def _draw_cannon(self, s):
        cx, cy = self.cannon
        pygame.draw.circle(s, (40, 46, 66), (int(cx), int(cy)), int(self.r) + 6)
        pygame.draw.circle(s, self.accent, (int(cx), int(cy)), int(self.r) + 6, 2)
        if self.shot is None and self.state == PLAY:
            self._bubble(s, cx, cy, BUBBLE_COLORS[self.cur])
        if self.shot is not None:
            self._bubble(s, self.shot["x"], self.shot["y"],
                         BUBBLE_COLORS[self.shot["c"]])
        # Nächste Kugel als Vorschau links neben der Kanone
        self._bubble(s, self.left + self.r, cy, BUBBLE_COLORS[self.nxt],
                     radius=self.r * 0.7)

    def _draw_hud(self, s):
        img = self._hud.render(t("bub.title"), True, self.accent)
        s.blit(img, img.get_rect(midleft=(20, 28)))
        img = self._small.render(t("bub.score", n=self.score), True, ui.GOLD)
        s.blit(img, img.get_rect(midright=(self.width - 20, 28)))

    def _draw_banner(self, s):
        w = min(self.width - 40, 460)
        h = 108
        rc = pygame.Rect((self.width - w) // 2, (self.height - h) // 2, w, h)
        panel = pygame.Surface((w, h), pygame.SRCALPHA)
        panel.fill((16, 18, 24, 238))
        s.blit(panel, rc.topleft)
        pygame.draw.rect(s, (228, 96, 96), rc, 2, border_radius=14)
        img = self._huge.render(t("bub.gameover"), True, (228, 96, 96))
        s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 40)))
        img = self._small.render(
            t("bub.score", n=self.score) + "   ·   " + t("common.enter_restart"),
            True, ui.TEXT_DIM)
        s.blit(img, img.get_rect(center=(rc.centerx, rc.y + 78)))
