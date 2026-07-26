# -*- coding: utf-8 -*-
"""
kniffel.py
==========
Kniffel (Yahtzee) - Einzelspieler (Highscore-Jagd) oder lokaler 2-Spieler-Hotseat.

Regeln:
- 5 Wuerfel, bis zu 3 Wuerfe pro Zug. Zwischen den Wuerfen duerfen beliebig
  viele Wuerfel "gehalten" werden (Klick/Taste); nur die uebrigen werden neu
  geworfen.
- Danach wird genau EINE der 13 Kategorien gebucht (auch mit 0 = Streichen).
  Nach 13 Buchungen ist der Block voll.
- Oberer Block (Einser..Sechser): bei >= 63 Punkten gibt es 35 Bonuspunkte.
- Unterer Block: Dreier-/Vierer-Pasch (Augensumme), Full House (25), kleine
  Strasse (30), grosse Strasse (40), Kniffel (50), Chance (Augensumme).

- Einzelspieler: Endsumme = Highscore (main.py speichert den Bestwert).
- Mehrspieler: zwei Blocks nebeneinander, abwechselnd; hoehere Endsumme gewinnt
  (nicht gewertet).

Steuerung: Maus - "Wuerfeln" klicken, Wuerfel anklicken = halten, Kategorie in
der Liste anklicken = buchen. Tasten: Leertaste = wuerfeln, 1-5 = Wuerfel halten/
loesen, Pfeile = Kategorie waehlen, Enter = buchen. Nach Ende: Enter = neu.
"""

import random

import pygame

import ui
from game_base import Game, InputEvent
from i18n import t

COL_PANEL = (28, 28, 38)
COL_PANEL_HI = (40, 40, 54)
COL_TEXT = (228, 230, 238)
COL_DIM = (150, 154, 170)
COL_FAINT = (110, 114, 130)
COL_ACCENT = (232, 176, 75)    # = Sidebar-Farbe #e8b04b
COL_GOOD = (110, 205, 140)
COL_P1 = (232, 176, 75)
COL_P2 = (110, 170, 235)
COL_DIE = (240, 242, 248)
COL_DIE_HELD = (250, 226, 160)
COL_PIP = (36, 38, 48)
COL_BTN = (44, 44, 58)
COL_BTN_ON = (92, 70, 34)
COL_BTN_BORDER = (78, 78, 100)

# Kategorien in Anzeigereihenfolge: (key, i18n-Suffix, ist_oberer_block)
CATS = [
    ("ones", "ones", True), ("twos", "twos", True), ("threes", "threes", True),
    ("fours", "fours", True), ("fives", "fives", True), ("sixes", "sixes", True),
    ("three_kind", "three_kind", False), ("four_kind", "four_kind", False),
    ("full_house", "full_house", False), ("small_straight", "small_straight", False),
    ("large_straight", "large_straight", False), ("kniffel", "kniffel", False),
    ("chance", "chance", False),
]
UPPER_KEYS = {"ones": 1, "twos": 2, "threes": 3, "fours": 4, "fives": 5, "sixes": 6}
UPPER_BONUS = 35
UPPER_TARGET = 63

READY, ROLL_ANIM, ROLLED, OVER = "ready", "roll_anim", "rolled", "over"


def score_category(key, dice):
    """Punktwert der Kategorie 'key' fuer die 5 Wuerfel 'dice' (Liste 1..6)."""
    counts = [dice.count(v) for v in range(1, 7)]
    total = sum(dice)
    if key in UPPER_KEYS:
        face = UPPER_KEYS[key]
        return counts[face - 1] * face
    if key == "three_kind":
        return total if max(counts) >= 3 else 0
    if key == "four_kind":
        return total if max(counts) >= 4 else 0
    if key == "full_house":
        nz = sorted(c for c in counts if c)
        return 25 if nz == [2, 3] else 0
    if key == "small_straight":
        faces = set(dice)
        runs = ({1, 2, 3, 4}, {2, 3, 4, 5}, {3, 4, 5, 6})
        return 30 if any(run <= faces for run in runs) else 0
    if key == "large_straight":
        faces = set(dice)
        return 40 if faces in ({1, 2, 3, 4, 5}, {2, 3, 4, 5, 6}) else 0
    if key == "kniffel":
        return 50 if max(counts) >= 5 else 0
    if key == "chance":
        return total
    return 0


def _empty_card():
    return {key: None for key, _, _ in CATS}


def _upper_sum(card):
    return sum(card[k] for k in UPPER_KEYS if card[k] is not None)


def _grand_total(card):
    up = _upper_sum(card)
    bonus = UPPER_BONUS if up >= UPPER_TARGET else 0
    low = sum(card[k] for k, _, _ in CATS
              if k not in UPPER_KEYS and card[k] is not None)
    return up + bonus + low


class KniffelGame(Game):
    name = "Kniffel"
    highscore_key = "kniffel"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        self.nplayers = 2 if self.multiplayer else 1
        self.cards = [_empty_card() for _ in range(self.nplayers)]
        self.player = 0
        self.dice = [1, 1, 1, 1, 1]
        self.held = [False] * 5
        self.rolls_left = 3
        self.rolled_once = False
        self.anim_t = 0.0
        self._final = list(self.dice)
        self.winner = None
        self.sel = 0
        self._make_fonts()
        self._layout()
        self._start_turn()

    def _make_fonts(self):
        self._small = ui.font(15)
        self._tiny = ui.font(12)
        self._row = ui.font(14, mono=True)
        self._big = ui.font(20, bold=True)
        self._huge = ui.font(max(26, self.height // 12), bold=True)

    def on_surface_changed(self):
        self._make_fonts()
        self._layout()

    def _layout(self):
        self.hud_h = 42
        self.dice_h = 92
        self.dice_y = self.height - self.dice_h
        # Kartenbereich zwischen HUD und Wuerfelleiste
        top = self.hud_h + 6
        bottom = self.dice_y - 6
        self.n_rows = len(CATS) + 2      # + Oberer-Bonus-Zeile + Gesamt
        self.row_h = max(16, (bottom - top) // self.n_rows)
        self.card_top = top
        # Namensspalte links, Wertspalten rechts (1 oder 2)
        self.name_x = 16
        col_w = 62
        if self.nplayers == 1:
            self.val_x = [self.width - 20 - col_w]
        else:
            self.val_x = [self.width - 20 - 2 * col_w - 10,
                          self.width - 20 - col_w]
        self.col_w = col_w
        # Zeilen-Trefferflaechen (nur die 13 Kategorien)
        self.row_rects = []
        for i in range(len(CATS)):
            y = self.card_top + i * self.row_h
            self.row_rects.append(pygame.Rect(self.name_x - 6, y,
                                              self.width - self.name_x - 8,
                                              self.row_h))
        # Wuerfel + Wuerfeln-Button
        self.die = int(min(self.dice_h - 40, (self.width - 200) / 5))
        self.die = max(34, self.die)
        gap = 12
        total_w = 5 * self.die + 4 * gap
        x0 = max(14, (self.width - 160 - total_w) // 2)
        self.die_rects = [pygame.Rect(x0 + i * (self.die + gap),
                                      self.dice_y + (self.dice_h - self.die) // 2,
                                      self.die, self.die) for i in range(5)]
        bw = 130
        self.roll_rect = pygame.Rect(self.width - bw - 16,
                                     self.dice_y + (self.dice_h - 44) // 2, bw, 44)

    def _start_turn(self):
        self.rolls_left = 3
        self.held = [False] * 5
        self.rolled_once = False
        self.state = READY
        # Auswahl auf erste offene Kategorie des aktuellen Spielers setzen
        card = self.cards[self.player]
        for i, (k, _, _) in enumerate(CATS):
            if card[k] is None:
                self.sel = i
                break

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == OVER:
            if event.kind == InputEvent.MOUSEDOWN or (
                    event.kind == InputEvent.KEYDOWN
                    and event.key in ("Return", "space")):
                self.game_over = False
                self.reset()
            return
        if event.kind == InputEvent.KEYDOWN:
            self._handle_key(event.key)
        elif event.kind == InputEvent.MOUSEDOWN:
            self._handle_click(event.pos)

    def _handle_key(self, k):
        if k in ("space",):
            self._roll()
        elif k in ("1", "2", "3", "4", "5"):
            self._toggle_hold(int(k) - 1)
        elif k in ("Up", "w", "W"):
            self.sel = (self.sel - 1) % len(CATS)
            self.play_sound("move")
        elif k in ("Down", "s", "S"):
            self.sel = (self.sel + 1) % len(CATS)
            self.play_sound("move")
        elif k in ("Return",):
            self._book(self.sel)

    def _handle_click(self, pos):
        if self.roll_rect.collidepoint(pos):
            self._roll()
            return
        for i, rc in enumerate(self.die_rects):
            if rc.collidepoint(pos):
                self._toggle_hold(i)
                return
        for i, rc in enumerate(self.row_rects):
            if rc.collidepoint(pos):
                self.sel = i
                self._book(i)
                return

    def _toggle_hold(self, i):
        # Halten ergibt nur nach dem ersten Wurf und vor dem letzten Sinn.
        if not self.rolled_once or self.state == ROLL_ANIM:
            return
        self.held[i] = not self.held[i]
        self.play_sound("click")

    def _roll(self):
        if self.state == ROLL_ANIM or self.rolls_left <= 0:
            return
        self._final = [self.dice[i] if (self.rolled_once and self.held[i])
                       else random.randint(1, 6) for i in range(5)]
        self.state = ROLL_ANIM
        self.anim_t = 0.5
        self.play_sound("move")

    def _book(self, cat_i):
        if not self.rolled_once or self.state == ROLL_ANIM:
            return
        key = CATS[cat_i][0]
        card = self.cards[self.player]
        if card[key] is not None:
            self.play_sound("click")
            return
        card[key] = score_category(key, self.dice)
        if key == "kniffel" and card[key] > 0:
            self.ach_event("kniffel_five")
        if card[key] > 0:
            self.play_sound("point")
        else:
            self.play_sound("select")
        if not self.multiplayer:
            self.score = _grand_total(card)
        self._next_turn()

    def _next_turn(self):
        if all(all(v is not None for v in c.values()) for c in self.cards):
            self._ende()
            return
        if self.multiplayer:
            self.player = 1 - self.player
        self._start_turn()

    def _ende(self):
        self.state = OVER
        self.game_over = True
        if self.multiplayer:
            t0 = _grand_total(self.cards[0])
            t1 = _grand_total(self.cards[1])
            self.winner = 0 if t0 > t1 else (1 if t1 > t0 else None)
            self.play_sound("win" if self.winner is not None else "select")
        else:
            self.score = _grand_total(self.cards[0])
            self.play_sound("win")

    # ===================================================== Update
    def update(self, dt):
        if self.state == ROLL_ANIM:
            self.anim_t -= dt
            if self.anim_t <= 0:
                self.dice = list(self._final)
                self.rolls_left -= 1
                self.rolled_once = True
                self.state = ROLLED
                self.play_sound("lock")

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        # Themen-Hintergrund statt flacher Fläche (reagiert auf Theme-Wechsel).
        ui.draw_background(s, self.width, self.height, stars=False, aurora=True)
        self._draw_hud(s)
        self._draw_card(s)
        self._draw_dice(s)
        if self.state == OVER:
            self._draw_over(s)

    def _draw_hud(self, s):
        pygame.draw.rect(s, COL_PANEL, (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, COL_BTN_BORDER, (0, self.hud_h), (self.width, self.hud_h))
        cy = self.hud_h // 2
        if self.multiplayer:
            who = t("common.player1") if self.player == 0 else t("common.player2")
            col = COL_P1 if self.player == 0 else COL_P2
            img = self._big.render(t("kn.turn", name=who), True, col)
        else:
            img = self._big.render(t("kn.total_now", n=_grand_total(self.cards[0])),
                                   True, COL_ACCENT)
        s.blit(img, img.get_rect(midleft=(14, cy)))
        # Wuerfe-Anzeige rechts
        if self.state != OVER:
            rt = t("kn.rolls_left", n=max(0, self.rolls_left))
            img = self._small.render(rt, True, COL_DIM)
            s.blit(img, img.get_rect(midright=(self.width - 14, cy)))

    def _draw_card(self, s):
        card = self.cards[self.player]
        # Spaltenkoepfe bei Mehrspieler
        if self.multiplayer:
            for p in range(2):
                lbl = self._tiny.render(t("common.player%d" % (p + 1)), True,
                                        COL_P1 if p == 0 else COL_P2)
                s.blit(lbl, lbl.get_rect(center=(self.val_x[p] + self.col_w // 2,
                                                 self.card_top - 1)))
        for i, (key, suff, is_upper) in enumerate(CATS):
            y = self.card_top + i * self.row_h
            rc = self.row_rects[i]
            selected = (i == self.sel and self.state != OVER)
            if selected:
                pygame.draw.rect(s, COL_PANEL_HI, rc, border_radius=5)
                pygame.draw.rect(s, COL_ACCENT, (rc.x, rc.y + 2, 3, rc.h - 4))
            name = self._row.render(t("kn." + suff), True,
                                    COL_TEXT if selected else COL_DIM)
            s.blit(name, name.get_rect(midleft=(self.name_x, y + self.row_h // 2)))
            for p in range(self.nplayers):
                pc = self.cards[p]
                cx = self.val_x[p] + self.col_w // 2
                if pc[key] is not None:
                    img = self._row.render(str(pc[key]), True, COL_TEXT)
                    s.blit(img, img.get_rect(center=(cx, y + self.row_h // 2)))
                elif p == self.player and self.rolled_once and self.state != OVER:
                    # Vorschau des moeglichen Werts (gedimmt)
                    val = score_category(key, self.dice)
                    col = COL_GOOD if val > 0 else COL_FAINT
                    img = self._row.render(str(val), True, col)
                    s.blit(img, img.get_rect(center=(cx, y + self.row_h // 2)))
                else:
                    img = self._row.render("-", True, COL_FAINT)
                    s.blit(img, img.get_rect(center=(cx, y + self.row_h // 2)))

        # Oberer-Bonus-Zeile
        y = self.card_top + len(CATS) * self.row_h
        lbl = self._row.render(t("kn.upper_bonus"), True, COL_DIM)
        s.blit(lbl, lbl.get_rect(midleft=(self.name_x, y + self.row_h // 2)))
        for p in range(self.nplayers):
            up = _upper_sum(self.cards[p])
            bonus = UPPER_BONUS if up >= UPPER_TARGET else 0
            cx = self.val_x[p] + self.col_w // 2
            txt = f"{up}/{UPPER_TARGET}" + (f" +{bonus}" if bonus else "")
            img = self._tiny.render(txt, True,
                                    COL_GOOD if bonus else COL_FAINT)
            s.blit(img, img.get_rect(center=(cx, y + self.row_h // 2)))

        # Gesamtsumme
        y += self.row_h
        pygame.draw.line(s, COL_BTN_BORDER, (self.name_x - 6, y),
                         (self.width - 8, y))
        lbl = self._big.render(t("kn.total"), True, COL_TEXT)
        s.blit(lbl, lbl.get_rect(midleft=(self.name_x, y + self.row_h // 2 + 2)))
        for p in range(self.nplayers):
            cx = self.val_x[p] + self.col_w // 2
            col = COL_P1 if (self.multiplayer and p == 0) else \
                (COL_P2 if self.multiplayer else COL_ACCENT)
            img = self._big.render(str(_grand_total(self.cards[p])), True, col)
            s.blit(img, img.get_rect(center=(cx, y + self.row_h // 2 + 2)))

    def _draw_dice(self, s):
        pygame.draw.rect(s, COL_PANEL, (0, self.dice_y, self.width, self.dice_h))
        pygame.draw.line(s, COL_BTN_BORDER, (0, self.dice_y), (self.width, self.dice_y))
        show_anim = self.state == ROLL_ANIM
        for i, rc in enumerate(self.die_rects):
            if not self.rolled_once and self.state == READY:
                val = 0
            elif show_anim and not (self.rolled_once and self.held[i]):
                val = random.randint(1, 6)
            else:
                val = self.dice[i]
            held = self.held[i] and self.rolled_once
            self._draw_die(s, rc, val, held, i)

        # Wuerfeln-Button
        can_roll = self.rolls_left > 0 and self.state in (READY, ROLLED)
        pygame.draw.rect(s, COL_BTN_ON if can_roll else COL_BTN, self.roll_rect,
                         border_radius=10)
        pygame.draw.rect(s, COL_ACCENT if can_roll else COL_BTN_BORDER,
                         self.roll_rect, 2 if can_roll else 1, border_radius=10)
        label = t("kn.roll") if self.state != READY or self.rolled_once \
            else t("kn.roll_start")
        img = self._small.render(label, True, COL_TEXT if can_roll else COL_FAINT)
        s.blit(img, img.get_rect(center=self.roll_rect.center))

    def _draw_die(self, s, rc, val, held, idx):
        base = COL_DIE_HELD if held else COL_DIE
        pygame.draw.rect(s, (10, 12, 18), rc.move(0, 3), border_radius=8)
        pygame.draw.rect(s, base, rc, border_radius=8)
        if held:
            pygame.draw.rect(s, COL_ACCENT, rc, 3, border_radius=8)
        else:
            pygame.draw.rect(s, (200, 202, 210), rc, 1, border_radius=8)
        # Wuerfel-Nummer klein oben links (Halt-Taste)
        num = self._tiny.render(str(idx + 1), True, (170, 172, 180))
        s.blit(num, (rc.x + 4, rc.y + 2))
        if val < 1:
            return
        # Augen (Pips)
        r = max(3, self.die // 10)
        cx, cy = rc.center
        off = self.die // 4
        L, C, R = cx - off, cx, cx + off
        T, M, B = cy - off, cy, cy + off
        pips = {
            1: [(C, M)],
            2: [(L, T), (R, B)],
            3: [(L, T), (C, M), (R, B)],
            4: [(L, T), (R, T), (L, B), (R, B)],
            5: [(L, T), (R, T), (C, M), (L, B), (R, B)],
            6: [(L, T), (R, T), (L, M), (R, M), (L, B), (R, B)],
        }
        for (px, py) in pips.get(val, []):
            pygame.draw.circle(s, COL_PIP, (px, py), r)

    def _draw_over(self, s):
        ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        ov.fill((10, 12, 18, 180))
        s.blit(ov, (0, 0))
        cx, cy = self.width // 2, self.height // 2
        if self.multiplayer:
            if self.winner is None:
                head = self._huge.render(t("common.draw"), True, COL_DIM)
            else:
                col = COL_P1 if self.winner == 0 else COL_P2
                head = self._huge.render(t("common.player_wins", n=self.winner + 1),
                                         True, col)
            s.blit(head, head.get_rect(center=(cx, cy - 40)))
            sub = self._small.render(
                f"{_grand_total(self.cards[0])} : {_grand_total(self.cards[1])}",
                True, COL_TEXT)
            s.blit(sub, sub.get_rect(center=(cx, cy)))
        else:
            head = self._huge.render(t("kn.finished"), True, COL_ACCENT)
            s.blit(head, head.get_rect(center=(cx, cy - 40)))
            sub = self._big.render(t("kn.your_score", n=_grand_total(self.cards[0])),
                                   True, COL_TEXT)
            s.blit(sub, sub.get_rect(center=(cx, cy + 2)))
        hint = self._tiny.render(t("common.enter_restart"), True, COL_DIM)
        s.blit(hint, hint.get_rect(center=(cx, cy + 44)))
