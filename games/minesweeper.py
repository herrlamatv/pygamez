# -*- coding: utf-8 -*-
"""
minesweeper.py
==============
Minesweeper - der Klassiker, mausgesteuert (Einzelspieler).

Features
--------
- Drei Schwierigkeitsgrade (im Setup wählbar, wird gespeichert):
    * Einsteiger       9 x 9,  10 Minen
    * Fortgeschritten 16 x 16, 40 Minen
    * Experte         30 x 16, 99 Minen
- Der ERSTE Klick ist immer sicher: die Minen werden erst nach dem ersten
  Aufdecken verteilt und sparen das 3x3-Feld um den Klick aus.
- Steuerung: Linksklick = aufdecken, RECHTSKLICK = Flagge (optional mit
  Fragezeichen-Zyklus: Flagge -> ? -> leer), F = Flagge unter dem Mauszeiger,
  R = neues Spiel, S = zurück zum Setup.
- CHORDING: Klick auf eine aufgedeckte Zahl, um die alle Flaggen gesetzt
  sind, deckt die restlichen Nachbarn auf einmal auf (Vorsicht bei falschen
  Flaggen!).
- Klassisches HUD: Minenzähler links, klickbarer SMILEY in der Mitte
  (staunt beim Aufdecken, trägt Sonnenbrille beim Sieg, ist tot beim Boom),
  Timer rechts.
- Bestzeit je Schwierigkeitsgrad wird in settings.json gespeichert und im
  Setup angezeigt. Highscore-Punkte = Grundwert der Stufe minus Sekunden.
- Optik: Themen-Hintergrund + ui.*-Palette für HUD/Setup, klassische
  Zellen mit 3D-Kanten und gefärbten Zahlen (Identitätsfarben), Hover-
  Markierung, rote Explosionszelle, falsche Flaggen werden durchgestrichen,
  Konfetti-Regen beim Sieg. Layout skaliert mit der Auflösung
  (on_surface_changed).
"""

import math
import random
import pygame

import settings as settings_mod
import ui
from game_base import Game, InputEvent
from i18n import t

# Identitätsfarben des Spielfelds (bewusst fest, unabhängig vom Theme).
COL_TILE = (52, 60, 82)          # verdeckte Zelle
COL_TILE_HOVER = (64, 74, 100)
COL_BEVEL_L = (88, 99, 128)      # helle 3D-Kante (oben/links)
COL_BEVEL_D = (30, 34, 48)       # dunkle 3D-Kante (unten/rechts)
COL_OPEN = (33, 38, 54)          # aufgedeckte Zelle
COL_OPEN_LINE = (24, 28, 40)
COL_BOOM = (168, 66, 66)         # explodierte Mine
COL_LED = (240, 90, 90)          # LED-Ziffern (klassisch rot)
COL_FLAG = (235, 90, 90)

# Klassische Zahlenfarben (für dunklen Hintergrund aufgehellt)
NUMBER_COLORS = {
    1: (110, 160, 255), 2: (110, 220, 130), 3: (250, 110, 110),
    4: (180, 140, 255), 5: (230, 160, 90), 6: (110, 220, 220),
    7: (235, 235, 235), 8: (170, 170, 185),
}

# (Schlüssel, Spalten, Zeilen, Minen, Punkte-Grundwert)
PRESETS = [
    ("beginner", 9, 9, 10, 150),
    ("advanced", 16, 16, 40, 500),
    ("expert", 30, 16, 99, 1200),
]
PRESET_KEYS = [p[0] for p in PRESETS]

HUD_H = 52                       # Kopfzeile über dem Spielfeld
SURPRISE_T = 0.22                # so lange staunt der Smiley nach einem Klick

SETUP, PLAY = "setup", "play"


class MinesweeperGame(Game):
    name = "Minesweeper"
    highscore_key = "minesweeper"
    supports_multiplayer = False
    wants_right_click = True     # Rechtsklick = Flagge

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        ms = self.settings.get("minesweeper", {}) if isinstance(self.settings, dict) else {}
        key = ms.get("preset", "beginner")
        self.preset_index = PRESET_KEYS.index(key) if key in PRESET_KEYS else 0
        self.qmarks = bool(ms.get("qmarks", False))
        self.best = dict(ms.get("best", {}))    # preset -> Sekunden (Bestzeit)

        self._make_fonts()
        self.anim_t = 0.0

        self._build_setup_layout()
        self.state = SETUP
        self._start_board()

    @property
    def preset(self):
        return PRESETS[self.preset_index]

    # ----- Layout / Theme --------------------------------------------------
    def _make_fonts(self):
        """Themen-Schriften, Grössen aus der Fensterhöhe abgeleitet."""
        h = self.height
        self.font = ui.font(max(16, h // 26))
        self.big_font = ui.font(max(30, h // 11), bold=True)
        self._small = ui.font(max(13, h // 32))
        self._tiny = ui.font(max(11, h // 40))
        # LED-Anzeige: Mono-Schrift, damit die Ziffern nicht "wackeln".
        self._led = ui.font(max(18, min(26, h // 24)), bold=True, mono=True)

    def on_surface_changed(self):
        """Auflösungswechsel: Schriften und Layout neu aufbauen;
        der Brett-Zustand (Minen, aufgedeckte Zellen, Timer) bleibt erhalten."""
        self._make_fonts()
        self._build_setup_layout()
        self._layout_board()
        self.hover = None

    def _start_board(self):
        """Baut ein frisches Brett für den aktuellen Schwierigkeitsgrad."""
        _, self.cols, self.rows, self.n_mines, self.base_points = self.preset

        self.mines = set()
        self.numbers = {}          # (x,y) -> Anzahl Nachbarminen
        self.revealed = set()
        self.flags = {}            # (x,y) -> 1 = Flagge, 2 = Fragezeichen
        self.first_click = True
        self.exploded = None       # die Zelle, die hochgegangen ist
        self.won = False
        self.game_over = False
        self.score = 0
        self.elapsed = 0.0
        self.running = False       # Timer läuft (ab dem ersten Klick)
        self.hover = None
        self.surprise = 0.0
        self.particles = []

        self._layout_board()

    def _layout_board(self):
        """Zellgröße/Lage: unter dem HUD zentriert, ganzzahlige Pixel."""
        aw = self.width - 24
        ah = self.height - HUD_H - 24
        self.cell = max(10, min(aw // self.cols, ah // self.rows))
        bw, bh = self.cell * self.cols, self.cell * self.rows
        self.bx = (self.width - bw) // 2
        self.by = HUD_H + 12 + (ah - bh) // 2

        self._num_font = ui.font(max(11, int(self.cell * 0.55)), bold=True)

        # Smiley-Knopf im HUD
        self.face_rect = pygame.Rect(self.width // 2 - 20, 8, 40, 40)

    # ----- Hilfen ----------------------------------------------------------
    def _neighbors(self, cell):
        x, y = cell
        for dx in (-1, 0, 1):
            for dy in (-1, 0, 1):
                if dx == 0 and dy == 0:
                    continue
                nx, ny = x + dx, y + dy
                if 0 <= nx < self.cols and 0 <= ny < self.rows:
                    yield (nx, ny)

    def _cell_at(self, pos):
        x = (pos[0] - self.bx) // self.cell
        y = (pos[1] - self.by) // self.cell
        if 0 <= x < self.cols and 0 <= y < self.rows:
            return (int(x), int(y))
        return None

    def _place_mines(self, safe):
        """Verteilt die Minen; das 3x3-Feld um 'safe' bleibt frei."""
        tabu = {safe} | set(self._neighbors(safe))
        alle = [(x, y) for x in range(self.cols) for y in range(self.rows)
                if (x, y) not in tabu]
        self.mines = set(random.sample(alle, min(self.n_mines, len(alle))))
        self.numbers = {}
        for x in range(self.cols):
            for y in range(self.rows):
                if (x, y) in self.mines:
                    continue
                self.numbers[(x, y)] = sum(1 for nb in self._neighbors((x, y))
                                           if nb in self.mines)

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(440, self.width - 60)
        bh, gap = 46, 10
        y0 = max(124, int(self.height * 0.24))
        self.preset_rects = [pygame.Rect(cx - bw // 2, y0 + i * (bh + gap), bw, bh)
                             for i in range(len(PRESETS))]
        y1 = y0 + len(PRESETS) * (bh + gap) + 4
        self.qmark_rect = pygame.Rect(cx - bw // 2, y1, bw, 40)
        self.start_rect = pygame.Rect(cx - 95, y1 + 52, 190, 50)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("minesweeper", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _select_preset(self, i):
        self.preset_index = i
        self._save_setting("preset", PRESET_KEYS[i])
        self.play_sound("click")

    def _toggle_qmarks(self):
        self.qmarks = not self.qmarks
        self._save_setting("qmarks", self.qmarks)
        self.play_sound("select")

    def _start_play(self):
        self._start_board()
        self.state = PLAY
        self.play_sound("click")

    def _handle_setup_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("1", "2", "3"):
                self._select_preset(int(event.key) - 1)
            elif event.key in ("Up", "w", "W"):
                self._select_preset((self.preset_index - 1) % len(PRESETS))
            elif event.key in ("Down", "s", "S"):
                self._select_preset((self.preset_index + 1) % len(PRESETS))
            elif event.key in ("q", "Q", "f", "F"):
                self._toggle_qmarks()
            elif event.key in ("Return", "space"):
                self._start_play()
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            p = event.pos
            for i, r in enumerate(self.preset_rects):
                if r.collidepoint(p):
                    self._select_preset(i)
                    return
            if self.qmark_rect.collidepoint(p):
                self._toggle_qmarks()
            elif self.start_rect.collidepoint(p):
                self._start_play()

    # ===================================================== Eingabe (Spiel)
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup_event(event)
            return

        if event.kind == InputEvent.MOUSEMOVE:
            self.hover = self._cell_at(event.pos)
            return

        if event.kind == InputEvent.MOUSEDOWN:
            # Smiley = neues Spiel (nur Linksklick, wie beim Original)
            if event.button == 1 and self.face_rect.collidepoint(event.pos):
                self._start_board()
                self.play_sound("click")
                return
            if self.game_over:
                return
            cell = self._cell_at(event.pos)
            if cell is None:
                return
            if event.button == 3:
                self._toggle_flag(cell)
            else:
                self._click_cell(cell)
            return

        if event.kind != InputEvent.KEYDOWN:
            return

        if self.game_over:
            if event.key in ("Return", "space", "r", "R"):
                self._start_board()
            elif event.key in ("s", "S"):
                self.state = SETUP
                self.play_sound("click")
            return

        if event.key in ("f", "F") and self.hover:
            self._toggle_flag(self.hover)
        elif event.key in ("r", "R"):
            self._start_board()
            self.play_sound("click")
        elif event.key in ("s", "S"):
            self.state = SETUP
            self.play_sound("click")

    # ----- Spielzüge --------------------------------------------------------
    def _toggle_flag(self, cell):
        if cell in self.revealed or self.game_over:
            return
        zustand = self.flags.get(cell, 0)
        if zustand == 0:
            self.flags[cell] = 1
        elif zustand == 1 and self.qmarks:
            self.flags[cell] = 2
        else:
            self.flags.pop(cell, None)
        self.play_sound("move")

    def _click_cell(self, cell):
        self.surprise = SURPRISE_T
        if cell in self.revealed:
            self._chord(cell)
            return
        if self.flags.get(cell) == 1:
            return                          # geflaggte Zellen sind geschützt
        self._reveal(cell)

    def _chord(self, cell):
        """Zahl anklicken: Nachbarn aufdecken, wenn genug Flaggen gesetzt sind."""
        zahl = self.numbers.get(cell, 0)
        if zahl <= 0:
            return
        flaggen = sum(1 for nb in self._neighbors(cell)
                      if self.flags.get(nb) == 1)
        if flaggen != zahl:
            return
        for nb in self._neighbors(cell):
            if nb not in self.revealed and self.flags.get(nb) != 1:
                self._reveal(nb)
                if self.game_over:
                    return

    def _reveal(self, cell):
        if self.first_click:
            self._place_mines(safe=cell)
            self.first_click = False
            self.running = True

        if cell in self.mines:
            self._lose(cell)
            return

        # Flutfüllung: 0er-Zellen decken ihre Nachbarschaft mit auf
        stapel = [cell]
        neu = 0
        while stapel:
            c = stapel.pop()
            if c in self.revealed or c in self.mines:
                continue
            self.revealed.add(c)
            self.flags.pop(c, None)
            neu += 1
            if self.numbers.get(c, 0) == 0:
                for nb in self._neighbors(c):
                    if nb not in self.revealed:
                        stapel.append(nb)
        if neu:
            self.play_sound("click" if neu == 1 else "merge")
        self._check_win()

    def _lose(self, cell):
        self.exploded = cell
        self.revealed.add(cell)
        self.game_over = True
        self.won = False
        self.score = 0
        self.report_result(False)
        self.play_sound("explode")
        self.play_sound("gameover")
        self.rumble(250)

    def _check_win(self):
        if len(self.revealed) != self.cols * self.rows - self.n_mines:
            return
        self.game_over = True
        self.won = True
        # Restliche Minen automatisch flaggen
        for m in self.mines:
            self.flags[m] = 1
        sekunden = int(self.elapsed)
        self.score = max(10, self.base_points - sekunden)
        # Bestzeit je Schwierigkeitsgrad merken
        key = PRESET_KEYS[self.preset_index]
        if key not in self.best or sekunden < int(self.best[key]):
            self.best[key] = sekunden
            self._save_setting("best", self.best)
        self.report_result(True)
        self.ach_event("mine_win")
        self._confetti()
        self.play_sound("win")
        self.rumble(200)

    def _confetti(self):
        for _ in range(90):
            x = random.uniform(0, self.width)
            farbe = random.choice(list(NUMBER_COLORS.values()))
            self.particles.append([x, random.uniform(-80, 0),
                                   random.uniform(-30, 30),
                                   random.uniform(60, 190),
                                   random.uniform(1.5, 3.2), farbe])

    # ===================================================== Spiellogik
    def update(self, dt):
        self.anim_t += dt
        if self.surprise > 0:
            self.surprise -= dt
        rest = []
        for p in self.particles:
            p[0] += p[2] * dt
            p[1] += p[3] * dt
            p[3] += 60 * dt
            p[4] -= dt
            if p[4] > 0 and p[1] < self.height + 10:
                rest.append(p)
        self.particles = rest
        if self.state == PLAY and self.running and not self.game_over:
            self.elapsed += dt

    # ===================================================== Zeichnen
    def draw(self):
        if self.state == SETUP:
            self._draw_setup()
            return

        s = self.surface
        ui.draw_background(s, self.width, self.height)
        self._draw_hud(s)
        self._draw_board(s)
        for p in self.particles:
            pygame.draw.rect(s, p[5], (int(p[0]), int(p[1]), 4, 6))
        if self.game_over:
            self._draw_result(s)

    # ----- HUD ---------------------------------------------------------------
    def _draw_hud(self, s):
        hud = pygame.Rect(10, 6, self.width - 20, HUD_H - 4)
        pygame.draw.rect(s, ui.PANEL, hud, border_radius=10)
        pygame.draw.rect(s, ui.BORDER, hud, 1, border_radius=10)
        # Minenzähler (Minen minus Flaggen)
        flaggen = sum(1 for v in self.flags.values() if v == 1)
        wert = max(-99, self.n_mines - flaggen)
        led_l = self._draw_led(s, f"{wert:03d}", 24)
        # Timer
        sek = int(self.elapsed)
        self._draw_led(s, f"{min(sek, 999):03d}", None,
                       rechts=self.width - 24)
        # Schwierigkeitsname klein daneben
        name = self._tiny.render(t("mines.preset." + PRESET_KEYS[self.preset_index]),
                                 True, ui.TEXT_DIM)
        s.blit(name, (led_l.right + 12,
                      HUD_H // 2 - name.get_height() // 2 + 2))
        self._draw_face(s)

    def _draw_led(self, s, text, links, rechts=None):
        """Zeichnet eine LED-Anzeige und liefert ihr Rechteck zurück."""
        img = self._led.render(text, True, COL_LED)
        w, h = img.get_width() + 14, img.get_height() + 8
        x = links if rechts is None else rechts - w
        y = (HUD_H - h) // 2 + 2
        rect = pygame.Rect(x, y, w, h)
        pygame.draw.rect(s, (16, 12, 14), rect, border_radius=6)
        pygame.draw.rect(s, (60, 40, 44), rect, 1, border_radius=6)
        s.blit(img, (x + 7, y + h // 2 - img.get_height() // 2))
        return rect

    def _draw_face(self, s):
        r = self.face_rect
        pygame.draw.rect(s, ui.BTN, r, border_radius=8)
        pygame.draw.rect(s, COL_BEVEL_L, r, 2, border_radius=8)
        cx, cy = r.center
        gelb = (245, 205, 90)
        pygame.draw.circle(s, gelb, (cx, cy), 14)
        pygame.draw.circle(s, (120, 95, 30), (cx, cy), 14, 2)
        if self.game_over and not self.won:
            # tot: X-Augen, gerader Mund
            for ex in (cx - 6, cx + 6):
                pygame.draw.line(s, (60, 40, 20), (ex - 3, cy - 8), (ex + 3, cy - 2), 2)
                pygame.draw.line(s, (60, 40, 20), (ex + 3, cy - 8), (ex - 3, cy - 2), 2)
            pygame.draw.line(s, (60, 40, 20), (cx - 6, cy + 7), (cx + 6, cy + 7), 2)
        elif self.game_over and self.won:
            # Sieg: Sonnenbrille + Lächeln
            pygame.draw.rect(s, (30, 30, 40), (cx - 10, cy - 8, 8, 6), border_radius=2)
            pygame.draw.rect(s, (30, 30, 40), (cx + 2, cy - 8, 8, 6), border_radius=2)
            pygame.draw.line(s, (30, 30, 40), (cx - 2, cy - 6), (cx + 2, cy - 6), 2)
            pygame.draw.arc(s, (60, 40, 20), (cx - 7, cy - 2, 14, 11), math.pi, math.tau, 2)
        elif self.surprise > 0:
            # staunen beim Klicken
            pygame.draw.circle(s, (60, 40, 20), (cx - 5, cy - 5), 2)
            pygame.draw.circle(s, (60, 40, 20), (cx + 5, cy - 5), 2)
            pygame.draw.circle(s, (60, 40, 20), (cx, cy + 6), 4, 2)
        else:
            pygame.draw.circle(s, (60, 40, 20), (cx - 5, cy - 5), 2)
            pygame.draw.circle(s, (60, 40, 20), (cx + 5, cy - 5), 2)
            pygame.draw.arc(s, (60, 40, 20), (cx - 7, cy - 3, 14, 12), math.pi, math.tau, 2)

    # ----- Spielfeld ----------------------------------------------------------
    def _draw_board(self, s):
        brett = pygame.Rect(self.bx - 8, self.by - 8,
                            self.cols * self.cell + 16, self.rows * self.cell + 16)
        pygame.draw.rect(s, ui.PANEL, brett, border_radius=10)
        pygame.draw.rect(s, ui.BORDER, brett, 1, border_radius=10)
        for x in range(self.cols):
            for y in range(self.rows):
                self._draw_cell(s, (x, y))

    def _draw_cell(self, s, cell):
        x, y = cell
        c = self.cell
        rx, ry = self.bx + x * c, self.by + y * c
        rect = pygame.Rect(rx, ry, c, c)
        offen = cell in self.revealed
        zeige_mine = self.game_over and cell in self.mines

        if offen or (zeige_mine and self.flags.get(cell) != 1):
            grund = COL_BOOM if cell == self.exploded else COL_OPEN
            pygame.draw.rect(s, grund, rect)
            pygame.draw.rect(s, COL_OPEN_LINE, rect, 1)
            if cell in self.mines:
                self._draw_mine(s, rect)
            else:
                zahl = self.numbers.get(cell, 0)
                if zahl > 0:
                    img = self._num_font.render(str(zahl), True,
                                                NUMBER_COLORS[zahl])
                    s.blit(img, img.get_rect(center=rect.center))
        else:
            grund = COL_TILE_HOVER if (cell == self.hover and not self.game_over) \
                else COL_TILE
            pygame.draw.rect(s, grund, rect)
            # 3D-Kanten
            pygame.draw.line(s, COL_BEVEL_L, (rx, ry), (rx + c - 1, ry), 2)
            pygame.draw.line(s, COL_BEVEL_L, (rx, ry), (rx, ry + c - 1), 2)
            pygame.draw.line(s, COL_BEVEL_D, (rx + 1, ry + c - 1),
                             (rx + c - 1, ry + c - 1), 2)
            pygame.draw.line(s, COL_BEVEL_D, (rx + c - 1, ry + 1),
                             (rx + c - 1, ry + c - 1), 2)
            zustand = self.flags.get(cell, 0)
            if zustand == 1:
                self._draw_flag(s, rect)
                # falsche Flagge bei Spielende durchstreichen
                if self.game_over and cell not in self.mines:
                    pygame.draw.line(s, (250, 90, 90), rect.topleft,
                                     rect.bottomright, 2)
                    pygame.draw.line(s, (250, 90, 90), rect.topright,
                                     rect.bottomleft, 2)
            elif zustand == 2:
                img = self._num_font.render("?", True, ui.TEXT_DIM)
                s.blit(img, img.get_rect(center=rect.center))

    def _draw_mine(self, s, rect):
        cx, cy = rect.center
        r = max(3, self.cell // 4)
        for winkel in range(0, 360, 45):
            a = math.radians(winkel)
            pygame.draw.line(s, (20, 22, 30), (cx, cy),
                             (cx + math.cos(a) * (r + 3), cy + math.sin(a) * (r + 3)), 2)
        pygame.draw.circle(s, (20, 22, 30), (cx, cy), r)
        pygame.draw.circle(s, (90, 95, 110), (cx - r // 3, cy - r // 3),
                           max(1, r // 4))

    def _draw_flag(self, s, rect):
        cx, cy = rect.center
        h = self.cell * 0.6
        x = cx - 1
        pygame.draw.line(s, (200, 205, 215), (x, cy - h / 2), (x, cy + h / 2), 2)
        pygame.draw.polygon(s, COL_FLAG,
                            [(x, cy - h / 2), (x + h * 0.55, cy - h * 0.28),
                             (x, cy - h * 0.06)])
        pygame.draw.line(s, (200, 205, 215), (x - h * 0.25, cy + h / 2),
                         (x + h * 0.3, cy + h / 2), 2)

    def _draw_result(self, s):
        """Ergebnis-Banner mit Neustart-Hinweis (transluzentes Themen-Panel)."""
        if self.won:
            text = t("mines.win", t=f"{int(self.elapsed)}s")
            farbe = ui.GREEN
        else:
            text = t("mines.lose")
            farbe = ui.RED
        img = self.font.render(text, True, farbe)
        hint_col = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.0, 0.2, 1.0))
        hint = self._small.render(t("common.enter_restart"), True, hint_col)

        w = max(img.get_width(), hint.get_width()) + 36
        h = img.get_height() + hint.get_height() + 22
        r = pygame.Rect(self.width // 2 - w // 2, self.by - 10, w, h)
        panel = pygame.Surface(r.size, pygame.SRCALPHA)
        pygame.draw.rect(panel, (*ui.PANEL[:3], 235), panel.get_rect(), border_radius=10)
        pygame.draw.rect(panel, farbe, panel.get_rect(), 2, border_radius=10)
        s.blit(panel, r)
        s.blit(img, img.get_rect(midtop=(r.centerx, r.y + 8)))
        s.blit(hint, hint.get_rect(midtop=(r.centerx,
                                           r.y + 8 + img.get_height() + 4)))

    # ----- Setup zeichnen ------------------------------------------------
    def _draw_setup(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        ui.draw_title(s, self.width, "MINESWEEPER",
                      subtitle=t("mines.subtitle"), accent=self.accent)

        for i, (key, cols, rows, minen, _pts) in enumerate(PRESETS):
            r = self.preset_rects[i]
            an = (i == self.preset_index)
            pygame.draw.rect(s, ui.BTN_SEL if an else ui.BTN, r, border_radius=8)
            pygame.draw.rect(s, self.accent if an else ui.BORDER, r,
                             2 if an else 1, border_radius=8)
            lab = self.font.render(t("mines.preset." + key), True, ui.TEXT)
            s.blit(lab, (r.x + 16, r.y + 5))
            details = f"{cols}x{rows}   {minen} {t('mines.mines')}"
            det = self._tiny.render(details, True, ui.TEXT_DIM)
            s.blit(det, (r.x + 16, r.bottom - det.get_height() - 5))
            best = self.best.get(key)
            btxt = t("mines.best", t=f"{int(best)}s") if best is not None \
                else t("mines.best_none")
            img = self._small.render(btxt, True,
                                     ui.GOLD if best is not None else ui.TEXT_DIM)
            s.blit(img, (r.right - img.get_width() - 16,
                         r.centery - img.get_height() // 2))

        # Fragezeichen-Toggle
        r = self.qmark_rect
        pygame.draw.rect(s, ui.BTN_SEL if self.qmarks else ui.BTN, r, border_radius=8)
        pygame.draw.rect(s, ui.BORDER, r, 1, border_radius=8)
        lab = self.font.render(t("mines.qmarks"), True, ui.TEXT)
        s.blit(lab, (r.x + 16, r.centery - lab.get_height() // 2))
        wert = t("common.on") if self.qmarks else t("common.off")
        img = self.font.render(f"< {wert} >", True,
                               self.accent if self.qmarks else ui.TEXT_DIM)
        s.blit(img, (r.right - img.get_width() - 16,
                     r.centery - img.get_height() // 2))

        ui.draw_button(s, self.start_rect, t("common.start"), self.font,
                       selected=True, accent=ui.GREEN)

        h2 = self._tiny.render(t("mines.hint"), True, ui.GREEN)
        s.blit(h2, h2.get_rect(center=(self.width // 2, self.height - 48)))
        ui.draw_footer(s, self.width, self.height, t("mines.setup_hint"))
