# -*- coding: utf-8 -*-
"""
memory.py
=========
Memory (Paare finden) - Solo und lokales 2-Spieler-Duell.

- Brettgrößen 4x4, 6x6 und 8x6 (Setup, gespeichert).
- Die Motive (8 Formen x 3 Farben = 24 eindeutige Paare) werden komplett mit
  pygame-Primitiven gezeichnet - keine Bild-Dateien nötig.
- Karten drehen sich mit einer kurzen Flip-Animation; gefundene Paare bleiben
  gedimmt mit Häkchen liegen.
- Solo: wenige Züge + schnelle Zeit = mehr Punkte.
  Duell: abwechselnd aufdecken, Paar gefunden = nochmal dran; wer am Ende
  mehr Paare hat, gewinnt (Highscore = Paare des Siegers x 100).

Steuerung: Maus oder Pfeile/WASD + Leertaste/Enter, R = neue Runde, S = Setup.
"""

import math
import random

import pygame

import settings as settings_mod
import ui
from game_base import Game, InputEvent
from i18n import t

# Identitätsfarben der Karten & Spieler - bleiben bewusst fest, alle
# generischen UI-Farben kommen zur Laufzeit dynamisch aus der ui-Palette.
COL_CARD_BACK = (56, 64, 92)
COL_CARD_BACK_D = (40, 46, 66)
COL_CARD_FACE = (232, 234, 240)
COL_CARD_DONE = (58, 66, 88)
COL_CARD_EDGE = (74, 84, 116)
COL_P1 = (88, 156, 255)
COL_P2 = (245, 205, 100)

# (Schlüssel, Spalten, Reihen, Basispunkte)
SIZES = [("4x4", 4, 4, 1000), ("6x6", 6, 6, 2500), ("8x6", 8, 6, 4000)]
SIZE_KEYS = [s[0] for s in SIZES]

# Motiv-Farben (3 Gruppen) - kombiniert mit 8 Formen = 24 eindeutige Motive.
MOTIF_COLORS = [(225, 95, 95), (88, 156, 255), (245, 205, 100)]

FLIP_TIME = 0.25       # Dauer der Dreh-Animation
RESOLVE_TIME = 0.8     # Anzeigezeit eines Fehlpaars

SETUP, PLAY = "setup", "play"


# ----- Motiv-Formen (zeichnen in ein Rechteck) -------------------------------

def _pts_star(cx, cy, r):
    pts = []
    for i in range(10):
        rr = r if i % 2 == 0 else r * 0.45
        a = -math.pi / 2 + i * math.pi / 5
        pts.append((cx + rr * math.cos(a), cy + rr * math.sin(a)))
    return pts


def _draw_star(s, rect, col, face=COL_CARD_FACE):
    pygame.draw.polygon(s, col, _pts_star(rect.centerx, rect.centery,
                                          rect.w * 0.45))


def _draw_heart(s, rect, col, face=COL_CARD_FACE):
    r = rect.w // 4
    cx, cy = rect.centerx, rect.centery - r // 3
    pygame.draw.circle(s, col, (cx - r + 1, cy), r)
    pygame.draw.circle(s, col, (cx + r - 1, cy), r)
    pygame.draw.polygon(s, col, [(cx - 2 * r + 1, cy + r // 3),
                                 (cx + 2 * r - 1, cy + r // 3),
                                 (cx, cy + 2 * r)])


def _draw_moon(s, rect, col, face=COL_CARD_FACE):
    # Der "Ausschnitt" wird mit der aktuellen Kartenfarbe gefüllt - sonst
    # bliebe auf gedimmten (gefundenen) Karten ein heller Kreis stehen.
    r = int(rect.w * 0.4)
    pygame.draw.circle(s, col, rect.center, r)
    pygame.draw.circle(s, face,
                       (rect.centerx + r // 2, rect.centery - r // 3), r)


def _draw_diamond(s, rect, col, face=COL_CARD_FACE):
    cx, cy = rect.center
    w, h = rect.w * 0.38, rect.h * 0.46
    pygame.draw.polygon(s, col, [(cx, cy - h), (cx + w, cy),
                                 (cx, cy + h), (cx - w, cy)])


def _draw_triangle(s, rect, col, face=COL_CARD_FACE):
    cx, cy = rect.center
    r = rect.w * 0.42
    pygame.draw.polygon(s, col, [(cx, cy - r), (cx + r, cy + r * 0.7),
                                 (cx - r, cy + r * 0.7)])


def _draw_ring(s, rect, col, face=COL_CARD_FACE):
    pygame.draw.circle(s, col, rect.center, int(rect.w * 0.4),
                       max(3, rect.w // 8))


def _draw_cross(s, rect, col, face=COL_CARD_FACE):
    cx, cy = rect.center
    a = int(rect.w * 0.42)
    b = max(3, rect.w // 7)
    pygame.draw.rect(s, col, (cx - b, cy - a, 2 * b, 2 * a), border_radius=b)
    pygame.draw.rect(s, col, (cx - a, cy - b, 2 * a, 2 * b), border_radius=b)


def _draw_bolt(s, rect, col, face=COL_CARD_FACE):
    cx, cy = rect.center
    w, h = rect.w * 0.32, rect.h * 0.45
    pygame.draw.polygon(s, col, [(cx + w * 0.4, cy - h), (cx - w, cy + h * 0.15),
                                 (cx - w * 0.05, cy + h * 0.15),
                                 (cx - w * 0.4, cy + h), (cx + w, cy - h * 0.15),
                                 (cx + w * 0.05, cy - h * 0.15)])


SHAPES = [_draw_star, _draw_heart, _draw_moon, _draw_diamond,
          _draw_triangle, _draw_ring, _draw_cross, _draw_bolt]


class MemoryGame(Game):
    name = "Memory"
    highscore_key = "memory"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        ms = self.settings.get("memory", {}) if isinstance(self.settings, dict) else {}
        key = ms.get("size", "6x6")
        self.size_idx = SIZE_KEYS.index(key) if key in SIZE_KEYS else 1

        self._build_fonts()
        self._overlay = None
        self._build_setup_layout()
        self.state = SETUP

    def _build_fonts(self):
        """Schriftgrößen aus der aktuellen Auflösung ableiten (Theme-Fonts)."""
        h = self.height
        self._small = ui.font(max(14, h // 30))
        self._tiny = ui.font(max(11, h // 36))
        self._huge = ui.font(max(26, h // 11), bold=True)

    def on_surface_changed(self):
        self._build_fonts()
        self._overlay = None
        self._build_setup_layout()
        if self.state == PLAY:
            self._layout_board()

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(360, self.width - 60)
        y0 = int(self.height * 0.32)
        self.size_rects = [pygame.Rect(cx - bw // 2, y0 + i * 58, bw, 48)
                           for i in range(3)]
        self.start_rect = pygame.Rect(cx - 95, y0 + 3 * 58 + 14, 190, 46)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("memory", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("1", "2", "3"):
                self.size_idx = int(event.key) - 1
                self._save_setting("size", SIZE_KEYS[self.size_idx])
                self.play_sound("click")
            elif event.key in ("Up", "w", "W"):
                self.size_idx = (self.size_idx - 1) % 3
                self._save_setting("size", SIZE_KEYS[self.size_idx])
                self.play_sound("move")
            elif event.key in ("Down", "s", "S"):
                self.size_idx = (self.size_idx + 1) % 3
                self._save_setting("size", SIZE_KEYS[self.size_idx])
                self.play_sound("move")
            elif event.key in ("Return", "space"):
                self._new_game()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, r in enumerate(self.size_rects):
                if r.collidepoint(event.pos):
                    self.size_idx = i
                    self._save_setting("size", SIZE_KEYS[i])
                    self.play_sound("click")
                    return
            if self.start_rect.collidepoint(event.pos):
                self._new_game()

    # ===================================================== Spielaufbau
    def _new_game(self):
        self.score = 0
        self.game_over = False
        _key, cols, rows, base = SIZES[self.size_idx]
        self.cols, self.rows_n, self.base = cols, rows, base
        n_pairs = cols * rows // 2

        # Motive: (Form, Farbe)-Katalog mischen, n Paare ziehen, duplizieren.
        catalog = [(sh, co) for sh in range(len(SHAPES))
                   for co in range(len(MOTIF_COLORS))]
        rng = random.Random()
        rng.shuffle(catalog)
        motifs = catalog[:n_pairs] * 2
        rng.shuffle(motifs)

        self.cards = [dict(motif=m, state="down", p=0.0, target=0.0)
                      for m in motifs]
        self.first = None          # Index der ersten offenen Karte
        self.resolve_t = 0.0       # > 0: Fehlpaar liegt offen
        self.resolve_pair = None
        self.moves = 0
        self.elapsed = 0.0
        self.cursor = 0
        self.turn = 0              # Duell: 0 = Spieler 1, 1 = Spieler 2
        self.pairs = [0, 0]
        self.found = 0
        self._layout_board()
        self.state = PLAY
        self.play_sound("click")

    def _layout_board(self):
        self.hud_h = max(40, int(self.height * 0.09))
        m = max(8, self.width // 100)
        slot_w = (self.width - m * (self.cols + 1)) // self.cols
        slot_h = (self.height - self.hud_h - m * (self.rows_n + 1)) // self.rows_n
        self.ch = min(slot_h, int(slot_w * 4 / 3))
        self.cw = int(self.ch * 0.75)
        total_w = self.cols * self.cw + (self.cols - 1) * m
        total_h = self.rows_n * self.ch + (self.rows_n - 1) * m
        self.bx = (self.width - total_w) // 2
        self.by = self.hud_h + (self.height - self.hud_h - total_h) // 2
        self.gap = m

    def _card_rect(self, i):
        r, c = divmod(i, self.cols)
        return pygame.Rect(self.bx + c * (self.cw + self.gap),
                           self.by + r * (self.ch + self.gap),
                           self.cw, self.ch)

    def _card_at(self, pos):
        for i in range(len(self.cards)):
            if self._card_rect(i).collidepoint(pos):
                return i
        return None

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.game_over:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._new_game()
                elif event.key in ("s", "S"):
                    self.state = SETUP
                    self.play_sound("click")
            return
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("r", "R"):
                self._new_game()
            elif k in ("s", "S") and not self.is_action(k, "down"):
                self.state = SETUP
                self.play_sound("click")
            elif self.is_action(k, "up") or k == "Up":
                self._move_cursor(0, -1)
            elif self.is_action(k, "down") or k == "Down":
                self._move_cursor(0, 1)
            elif self.is_action(k, "left") or k == "Left":
                self._move_cursor(-1, 0)
            elif self.is_action(k, "right") or k == "Right":
                self._move_cursor(1, 0)
            elif k in ("space", "Return"):
                self._flip(self.cursor)
        elif event.kind == InputEvent.MOUSEDOWN:
            i = self._card_at(event.pos)
            if i is not None:
                self.cursor = i
                self._flip(i)

    def _move_cursor(self, dx, dy):
        r, c = divmod(self.cursor, self.cols)
        c = max(0, min(self.cols - 1, c + dx))
        r = max(0, min(self.rows_n - 1, r + dy))
        new = r * self.cols + c
        if new != self.cursor:      # am Rand: kein Klick-Geräusch
            self.cursor = new
            self.play_sound("move")

    # ===================================================== Spiellogik
    def _flip(self, i):
        if self.resolve_t > 0:
            return
        card = self.cards[i]
        if card["state"] != "down":
            return
        card["state"] = "up"
        card["target"] = 1.0
        self.play_sound("click")
        if self.first is None:
            self.first = i
            return
        # Zweite Karte
        self.moves += 1
        a, b = self.cards[self.first], card
        if a["motif"] == b["motif"]:
            a["state"] = b["state"] = "done"
            self.pairs[self.turn if self.multiplayer else 0] += 1
            self.found += 1
            self.first = None
            self.play_sound("merge")
            if self.found >= len(self.cards) // 2:
                self._finish()
        else:
            self.resolve_t = RESOLVE_TIME
            self.resolve_pair = (self.first, i)
            self.first = None

    def _finish(self):
        if self.multiplayer:
            winner = max(self.pairs)
            self.score = winner * 100
        else:
            self.score = max(100, self.base - 15 * self.moves
                             - 2 * int(self.elapsed))
            # Perfekt = jedes Paar im ersten Anlauf gefunden.
            if self.moves == len(self.cards) // 2:
                self.ach_event("memory_perfect")
        self.game_over = True
        self.play_sound("win")
        self.rumble(200)

    def update(self, dt):
        if self.state != PLAY or self.game_over:
            return
        self.elapsed += dt
        # Flip-Animationen
        for card in self.cards:
            if card["p"] < card["target"]:
                card["p"] = min(card["target"], card["p"] + dt / FLIP_TIME)
            elif card["p"] > card["target"]:
                card["p"] = max(card["target"], card["p"] - dt / FLIP_TIME)
        # Fehlpaar wieder zudecken + im Duell den Zug wechseln
        if self.resolve_t > 0:
            self.resolve_t -= dt
            if self.resolve_t <= 0 and self.resolve_pair:
                for i in self.resolve_pair:
                    if self.cards[i]["state"] == "up":
                        self.cards[i]["state"] = "down"
                        self.cards[i]["target"] = 0.0
                self.resolve_pair = None
                if self.multiplayer:
                    self.turn = 1 - self.turn
                self.play_sound("move")

    # ===================================================== Zeichnen
    def draw(self):
        if self.state == SETUP:
            self._draw_setup()
            return
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        for i, card in enumerate(self.cards):
            self._draw_card(s, i, card)
        self._draw_hud(s)
        if self.game_over:
            self._draw_result(s)

    def _draw_card(self, s, i, card):
        rect = self._card_rect(i)
        # Flip: Breite skaliert mit |cos(pi*p)|, Seite wechselt bei p=0.5
        wf = abs(math.cos(math.pi * card["p"]))
        w = max(3, int(rect.w * wf))
        r = pygame.Rect(rect.centerx - w // 2, rect.y, w, rect.h)
        front = card["p"] > 0.5
        rad = max(4, rect.w // 8)

        if front:
            face_col = COL_CARD_DONE if card["state"] == "done" else COL_CARD_FACE
            pygame.draw.rect(s, face_col, r, border_radius=rad)
            pygame.draw.rect(s, COL_CARD_EDGE, r, 2, border_radius=rad)
            if w > rect.w * 0.5:   # Motiv erst zeigen, wenn Karte weit genug offen
                shape_i, col_i = card["motif"]
                col = MOTIF_COLORS[col_i]
                if card["state"] == "done":
                    col = tuple(int(v * 0.55) for v in col)
                inner = rect.inflate(-rect.w // 4, -rect.h // 3)
                SHAPES[shape_i](s, inner, col, face_col)
                if card["state"] == "done":
                    bx, by = rect.right - 12, rect.bottom - 10
                    pygame.draw.lines(s, ui.GREEN, False,
                                      [(bx - 6, by - 3), (bx - 3, by),
                                       (bx + 3, by - 7)], 2)
        else:
            pygame.draw.rect(s, COL_CARD_BACK, r, border_radius=rad)
            pygame.draw.rect(s, COL_CARD_BACK_D, r, 2, border_radius=rad)
            if w > rect.w * 0.5:
                q = self._small.render("?", True, self.accent)
                s.blit(q, q.get_rect(center=rect.center))

        if i == self.cursor and not self.game_over:
            pygame.draw.rect(s, self.accent, rect.inflate(6, 6), 2,
                             border_radius=rad + 3)

    def _fmt_time(self):
        sec = int(self.elapsed)
        return f"{sec // 60:02d}:{sec % 60:02d}"

    def _draw_hud(self, s):
        pygame.draw.rect(s, ui.PANEL, (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h),
                         (self.width, self.hud_h))
        cy = self.hud_h // 2
        if self.multiplayer:
            p1 = self._small.render(
                t("common.player1") + "  " + t("mem.player_pairs", n=self.pairs[0]),
                True, COL_P1 if self.turn == 0 else ui.TEXT_DIM)
            s.blit(p1, p1.get_rect(midleft=(12, cy)))
            p2 = self._small.render(
                t("mem.player_pairs", n=self.pairs[1]) + "  " + t("common.player2"),
                True, COL_P2 if self.turn == 1 else ui.TEXT_DIM)
            s.blit(p2, p2.get_rect(midright=(self.width - 12, cy)))
            who = t("common.player1") if self.turn == 0 else t("common.player2")
            mid = self._small.render(t("mem.turn", name=who), True, self.accent)
            s.blit(mid, mid.get_rect(center=(self.width // 2, cy)))
        else:
            mv = self._small.render(t("mem.moves", n=self.moves), True, ui.TEXT)
            s.blit(mv, mv.get_rect(midleft=(12, cy)))
            tm = self._small.render(t("mem.time", t=self._fmt_time()), True,
                                    self.accent)
            s.blit(tm, tm.get_rect(center=(self.width // 2, cy)))
            left = len(self.cards) // 2 - self.found
            pr = self._small.render(t("mem.pairs", n=left), True, ui.TEXT_DIM)
            s.blit(pr, pr.get_rect(midright=(self.width - 12, cy)))

    def _draw_result(self, s):
        # Abdunkelung wird gecacht (kein Alpha-Vollbild-Fill pro Frame).
        if self._overlay is None \
                or self._overlay.get_size() != (self.width, self.height):
            ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
            ov.fill((8, 10, 16, 185))
            self._overlay = ov
        s.blit(self._overlay, (0, 0))
        cx, cy = self.width // 2, self.height // 2
        if self.multiplayer:
            if self.pairs[0] == self.pairs[1]:
                head = self._huge.render(t("common.draw"), True, ui.TEXT_DIM)
            else:
                n = 1 if self.pairs[0] > self.pairs[1] else 2
                head = self._huge.render(t("common.player_wins", n=n), True,
                                         COL_P1 if n == 1 else COL_P2)
            sub = self.font.render(f"{self.pairs[0]} : {self.pairs[1]}", True,
                                   ui.TEXT)
        else:
            head = self._huge.render(
                t("mem.win", t=self._fmt_time(), m=self.moves), True, ui.GREEN)
            sub = self.font.render(t("common.points", score=self.score), True,
                                   ui.TEXT)
        # Panel hinter dem Ergebnis (Akzent-Rahmen, Breite folgt dem Inhalt).
        pw = max(min(self.width - 40, 460), head.get_width() + 40)
        panel = pygame.Rect(cx - pw // 2, cy - 90, pw, 160)
        pygame.draw.rect(s, ui.PANEL, panel, border_radius=14)
        pygame.draw.rect(s, self.accent, panel, 2, border_radius=14)
        s.blit(head, head.get_rect(center=(cx, cy - 50)))
        s.blit(sub, sub.get_rect(center=(cx, cy + 4)))
        hint = self._small.render(t("mem.retry"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, cy + 40)))

    # ----- Setup zeichnen -----------------------------------------------
    def _draw_setup(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        title = self._huge.render("MEMORY", True, self.accent)
        s.blit(title, title.get_rect(center=(self.width // 2,
                                             int(self.height * 0.14))))
        sub = self._small.render(t("mem.subtitle"), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(self.width // 2,
                                         int(self.height * 0.21))))
        for i, r in enumerate(self.size_rects):
            on = (i == self.size_idx)
            pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, r, border_radius=10)
            pygame.draw.rect(s, self.accent if on else ui.BORDER, r,
                             2 if on else 1, border_radius=10)
            key, cols, rows, _base = SIZES[i]
            name = self.font.render(f"{cols} x {rows}", True,
                                    ui.TEXT if on else ui.TEXT_DIM)
            s.blit(name, name.get_rect(midleft=(r.x + 18, r.centery)))
            pairs = self._tiny.render(t("mem.pairs", n=cols * rows // 2), True,
                                      ui.TEXT_DIM)
            s.blit(pairs, pairs.get_rect(midright=(r.right - 18, r.centery)))
        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=10)
        pygame.draw.rect(s, self.accent, self.start_rect, 2, border_radius=10)
        st = self.font.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        hint = self._tiny.render(t("mem.setup_hint"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(self.width // 2, self.height - 30)))
        ctrl = self._tiny.render(t("mem.hint"), True, ui.TEXT_FAINT)
        s.blit(ctrl, ctrl.get_rect(center=(self.width // 2, self.height - 12)))
