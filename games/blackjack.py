# -*- coding: utf-8 -*-
"""
blackjack.py
============
Blackjack gegen den Dealer - mit Chips, Double Down und Split.

Regeln:
- 4-Deck-Schuh (208 Karten), neu gemischt wenn weniger als 52 übrig sind.
- Dealer steht auf ALLEN 17 (auch Soft 17). Blackjack zahlt 3:2.
- Dealer-Peek bei Ass/Zehnerkarte: Dealer-Blackjack beendet die Runde sofort
  (eigener Blackjack = Push). Keine Insurance.
- Double Down: nur auf den ersten beiden Karten (auch nach Split, außer bei
  Split-Assen), kostet den gleichen Einsatz, genau eine Karte, Auto-Stand.
- Split: genau EINMAL, bei gleichem Kartenwert (K+10 geht); Split-Asse
  bekommen je genau eine Karte; 21 nach Split zählt als 21, nicht Blackjack.

Chips: Start 500, Einsätze 10/25/50/100 (stapelbar). Der Chipstand bleibt
über Sitzungen erhalten (mem.json, Abschnitt "blackjack"). Der Highscore ist
der höchste jemals erreichte Chipstand; er wird beim Menü-Rückweg gespeichert
(game_over wird nie gesetzt). Pleite = Neustart mit 500 (Bestwert bleibt).

Steuerung: Buttons anklicken oder H = Hit, S = Stand, D = Double, X = Split,
1-4 = Chips setzen, Backspace = Einsatz löschen, Enter = Geben/Weiter.
"""

import random

import pygame

import store
from game_base import Game, InputEvent
from i18n import t

from . import cards as C

COL_FELT = (20, 46, 36)
COL_FELT_EDGE = (14, 34, 26)
COL_TEXT = (225, 228, 238)
COL_DIM = (170, 190, 180)
COL_ACCENT = (200, 56, 79)     # = Sidebar-Farbe #c8384f
COL_OK = (110, 205, 140)
COL_BAD = (225, 95, 95)
COL_GOLD = (245, 205, 100)
COL_BTN = (30, 64, 50)
COL_BTN_ON = (44, 92, 72)
COL_BTN_BORDER = (80, 120, 100)
CHIP_COLS = {10: (110, 160, 235), 25: (110, 205, 140),
             50: (230, 120, 90), 100: (40, 40, 48)}

START_CHIPS = 500
BETS = (10, 25, 50, 100)
DEAL_T = 0.22       # Tween-Dauer je Karte
FLIP_T = 0.25       # Hole-Card-Flip
DEALER_T = 0.5      # Pause je Dealer-Karte

BET, DEALING, PLAYER, DEALER, PAYOUT, BROKE = \
    "bet", "dealing", "player", "dealer", "payout", "broke"


def hand_value(cards_):
    """(Summe, soft?) - Asse zählen 11, solange kein Bust."""
    total = 0
    aces = 0
    for c in cards_:
        v = min(c.rank, 10)
        if c.rank == 1:
            aces += 1
            v = 11
        total += v
    while total > 21 and aces:
        total -= 10
        aces -= 1
    return total, aces > 0


class BlackjackGame(Game):
    name = "Blackjack"
    highscore_key = "blackjack"
    supports_multiplayer = False

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.game_over = False
        data = store.load_section("blackjack")
        try:
            self.chips = max(0, int(data.get("chips", START_CHIPS)))
        except (TypeError, ValueError):
            self.chips = START_CHIPS
        try:
            self.best = max(self.chips, int(data.get("best", START_CHIPS)))
        except (TypeError, ValueError):
            self.best = max(self.chips, START_CHIPS)
        self.score = self.best

        self._small = pygame.font.SysFont("consolas", 16)
        self._tiny = pygame.font.SysFont("consolas", 13)
        self._big = pygame.font.SysFont("consolas", 22, bold=True)
        self._huge = pygame.font.SysFont("consolas", max(26, self.height // 11),
                                         bold=True)
        self.renderer = C.CardRenderer(COL_ACCENT)
        self._layout()

        self.shoe = []
        self.bet = 0
        self.hands = []          # Listen von Cards (1 oder 2 nach Split)
        self.hand_bets = []
        self.hand_done = []
        self.active = 0
        self.split_aces = False
        self.dealer = []
        self.hole_hidden = True
        self.results = []        # Texte je Hand im PAYOUT
        self.tweens = []         # (card, from, to, t, dur, flip_up)
        self.dealer_wait = 0.0
        self.flip_t = 0.0
        self.state = BET if self.chips >= BETS[0] else BROKE

    def on_surface_changed(self):
        self._huge = pygame.font.SysFont("consolas", max(26, self.height // 11),
                                         bold=True)
        self.renderer.clear()
        self._layout()

    def _layout(self):
        self.ch = int(self.height * 0.20)
        self.cw = int(self.ch * 0.72)
        self.shoe_pos = (self.width - self.cw - 16, 12)
        self.dealer_y = int(self.height * 0.20)
        self.player_y = int(self.height * 0.58)
        self.strip = pygame.Rect(0, self.height - 72, self.width, 72)
        # BET-Bedienung
        self.chip_rects = [pygame.Rect(24 + i * 56, self.strip.y + 14, 44, 44)
                           for i in range(4)]
        bw = int(self.width * 0.14)
        self.deal_rect = pygame.Rect(self.width - bw - 16,
                                     self.strip.y + 14, bw, 44)
        self.clear_rect = pygame.Rect(self.width - 2 * bw - 28,
                                      self.strip.y + 14, bw, 44)
        # PLAYER-Buttons
        bw2 = int(self.width * 0.16)
        gap = (self.width - 4 * bw2) // 5
        self.action_rects = {}
        for i, key in enumerate(("hit", "stand", "double", "split")):
            self.action_rects[key] = pygame.Rect(
                gap + i * (bw2 + gap), self.strip.y + 14, bw2, 44)

    def _save(self):
        store.save_section("blackjack", {"chips": self.chips,
                                         "best": self.best})

    # ===================================================== Schuh / Hände
    def _ensure_shoe(self):
        if len(self.shoe) < 52:
            self.shoe = C.make_deck(copies=4)
            C.shuffle(self.shoe, random.Random())

    def _draw_card(self, face_up=True):
        card = self.shoe.pop()
        card.face_up = face_up
        return card

    def _hand_positions(self, hand_i, n):
        """Bildschirmpositionen der Karten einer Spielerhand."""
        if len(self.hands) == 1:
            cx = self.width // 2
        else:
            cx = int(self.width * (0.35 if hand_i == 0 else 0.65))
        dx = int(self.cw * 0.55)
        x0 = cx - (self.cw + (n - 1) * dx) // 2
        return [(x0 + i * dx, self.player_y) for i in range(n)]

    def _dealer_positions(self, n):
        dx = int(self.cw * 0.55)
        x0 = self.width // 2 - (self.cw + (n - 1) * dx) // 2
        return [(x0 + i * dx, self.dealer_y) for i in range(n)]

    # ===================================================== Runden-Ablauf
    def _start_deal(self):
        if self.bet < BETS[0] or self.bet > self.chips:
            return
        self._ensure_shoe()
        self.chips -= self.bet
        self.hands = [[]]
        self.hand_bets = [self.bet]
        self.hand_done = [False]
        self.active = 0
        self.split_aces = False
        self.dealer = []
        self.hole_hidden = True
        self.results = []
        self.tweens = []
        self.state = DEALING
        # Reihenfolge: Spieler, Dealer, Spieler, Dealer (verdeckt)
        for i, (target, up) in enumerate(((self.hands[0], True),
                                          (self.dealer, True),
                                          (self.hands[0], True),
                                          (self.dealer, False))):
            card = self._draw_card(up)
            target.append(card)
            self._enqueue(card, target is self.dealer)
        self.play_sound("move")

    def _enqueue(self, card, is_dealer):
        self.tweens.append(dict(card=card, t=0.0,
                                delay=len(self.tweens) * DEAL_T))

    def _after_deal(self):
        """Nach der Startausgabe: Peek/Blackjack prüfen."""
        pv, _ = hand_value(self.hands[0])
        dv, _ = hand_value(self.dealer)
        player_bj = pv == 21
        up = self.dealer[0]
        if up.rank == 1 or min(up.rank, 10) == 10:
            if dv == 21:                      # Dealer-Blackjack
                self.hole_hidden = False
                if player_bj:
                    self._settle(["push"])
                else:
                    self._settle(["lose"])
                return
        if player_bj:
            self.hole_hidden = False
            self._settle(["blackjack"])
            return
        self.state = PLAYER
        self.play_sound("select")

    # ----- Spieler-Aktionen ------------------------------------------------

    def _can_double(self):
        h = self.hands[self.active]
        return (len(h) == 2 and not self.split_aces
                and self.chips >= self.hand_bets[self.active])

    def _can_split(self):
        if len(self.hands) != 1:
            return False
        h = self.hands[0]
        return (len(h) == 2
                and min(h[0].rank, 10) == min(h[1].rank, 10)
                and self.chips >= self.hand_bets[0])

    def _hit(self):
        h = self.hands[self.active]
        card = self._draw_card()
        h.append(card)
        self.play_sound("move")
        total, _ = hand_value(h)
        if total >= 21:
            self._next_hand()

    def _stand(self):
        self._next_hand()

    def _double(self):
        if not self._can_double():
            return
        self.chips -= self.hand_bets[self.active]
        self.hand_bets[self.active] *= 2
        h = self.hands[self.active]
        h.append(self._draw_card())
        self.play_sound("move")
        self._next_hand()

    def _split(self):
        if not self._can_split():
            return
        h = self.hands[0]
        self.split_aces = (h[0].rank == 1)
        self.chips -= self.hand_bets[0]
        self.hands = [[h[0]], [h[1]]]
        self.hand_bets = [self.hand_bets[0], self.hand_bets[0]]
        self.hand_done = [False, False]
        self.active = 0
        # Jede Hand bekommt sofort eine zweite Karte.
        self.hands[0].append(self._draw_card())
        self.hands[1].append(self._draw_card())
        self.play_sound("rotate")
        if self.split_aces:
            # Split-Asse: keine weiteren Aktionen.
            self.hand_done = [True, True]
            self._start_dealer()

    def _next_hand(self):
        self.hand_done[self.active] = True
        if self.active + 1 < len(self.hands):
            self.active += 1
            total, _ = hand_value(self.hands[self.active])
            if total >= 21:
                self._next_hand()
            return
        # Alle Hände durch -> Dealer (nur wenn nicht alles Bust)
        busted_all = all(hand_value(h)[0] > 21 for h in self.hands)
        if busted_all:
            self.hole_hidden = False
            self._settle(None)
        else:
            self._start_dealer()

    def _start_dealer(self):
        self.state = DEALER
        self.flip_t = FLIP_T
        self.dealer_wait = DEALER_T

    # ----- Dealer + Auszahlung ------------------------------------------------

    def _dealer_step(self):
        total, _ = hand_value(self.dealer)
        if total < 17:                       # steht auf ALLEN 17 (S17)
            self.dealer.append(self._draw_card())
            self.play_sound("move")
            self.dealer_wait = DEALER_T
        else:
            self._settle(None)

    def _settle(self, forced):
        """Zahlt alle Hände aus. forced: Liste je Hand oder None (berechnen)."""
        self.hole_hidden = False
        dv, _ = hand_value(self.dealer)
        dealer_bust = dv > 21
        self.results = []
        delta = 0
        for i, h in enumerate(self.hands):
            bet = self.hand_bets[i]
            if forced is not None:
                res = forced[min(i, len(forced) - 1)]
            else:
                pv, _ = hand_value(h)
                if pv > 21:
                    res = "bust"
                elif dealer_bust:
                    res = "dealer_bust"
                elif pv > dv:
                    res = "win"
                elif pv < dv:
                    res = "lose"
                else:
                    res = "push"
            if res == "blackjack":
                delta += bet + bet * 3 // 2
            elif res in ("win", "dealer_bust"):
                delta += bet * 2
            elif res == "push":
                delta += bet
            self.results.append(res)
        self.chips += delta
        if self.chips > self.best:
            self.best = self.chips
        self.score = self.best
        self._save()
        self.state = PAYOUT
        if any(r == "blackjack" for r in self.results):
            self.play_sound("win")
        elif any(r in ("win", "dealer_bust") for r in self.results):
            self.play_sound("point")
        elif all(r == "push" for r in self.results):
            self.play_sound("select")
        else:
            self.play_sound("gameover")

    def _to_bet(self):
        self.bet = 0
        if self.chips < BETS[0]:
            self.state = BROKE
            self.play_sound("gameover")
        else:
            self.state = BET

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == BROKE:
            if event.kind == InputEvent.MOUSEDOWN or \
                    (event.kind == InputEvent.KEYDOWN
                     and event.key in ("Return", "space")):
                self.chips = START_CHIPS
                self._save()
                self._to_bet()
                self.play_sound("click")
            return
        if self.state == BET:
            self._handle_bet(event)
        elif self.state == PLAYER:
            self._handle_player(event)
        elif self.state == PAYOUT:
            if event.kind == InputEvent.MOUSEDOWN or \
                    (event.kind == InputEvent.KEYDOWN
                     and event.key in ("Return", "space")):
                self._to_bet()

    def _handle_bet(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3", "4"):
                self._add_chip(BETS[int(k) - 1])
            elif k == "BackSpace":
                # Einsatz zurücknehmen (wird erst beim Geben abgezogen)
                self.bet = 0
                self.play_sound("move")
            elif k in ("Return", "space"):
                self._start_deal()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, r in enumerate(self.chip_rects):
                if r.collidepoint(event.pos):
                    self._add_chip(BETS[i])
                    return
            if self.clear_rect.collidepoint(event.pos):
                self.bet = 0
                self.play_sound("move")
            elif self.deal_rect.collidepoint(event.pos):
                self._start_deal()

    def _add_chip(self, value):
        if self.bet + value <= self.chips:
            self.bet += value
            self.play_sound("click")

    def _handle_player(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("h", "H"):
                self._hit()
            elif k in ("s", "S"):
                self._stand()
            elif k in ("d", "D"):
                self._double()
            elif k in ("x", "X"):
                self._split()
        elif event.kind == InputEvent.MOUSEDOWN:
            for key, r in self.action_rects.items():
                if r.collidepoint(event.pos):
                    if key == "hit":
                        self._hit()
                    elif key == "stand":
                        self._stand()
                    elif key == "double" and self._can_double():
                        self._double()
                    elif key == "split" and self._can_split():
                        self._split()
                    return

    # ===================================================== Update
    def update(self, dt):
        if self.state == DEALING:
            done = True
            for tw in self.tweens:
                tw["t"] += dt
                if tw["t"] < tw["delay"] + DEAL_T:
                    done = False
            if done:
                self.tweens = []
                self._after_deal()
        elif self.state == DEALER:
            if self.flip_t > 0:
                self.flip_t -= dt
                return
            self.dealer_wait -= dt
            if self.dealer_wait <= 0:
                self._dealer_step()

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        s.fill(COL_FELT)
        pygame.draw.rect(s, COL_FELT_EDGE, (0, 0, self.width, self.height), 6)

        # Schuh oben rechts
        s.blit(self.renderer.back(self.cw, self.ch), self.shoe_pos)
        n = self._tiny.render(str(len(self.shoe)), True, COL_DIM)
        s.blit(n, (self.shoe_pos[0], self.shoe_pos[1] + self.ch + 4))

        self._draw_dealer(s)
        self._draw_player(s)
        self._draw_topbar(s)

        if self.state == BET:
            self._draw_bet_ui(s)
        elif self.state == PLAYER:
            self._draw_action_ui(s)
        elif self.state == PAYOUT:
            self._draw_results(s)
        elif self.state == BROKE:
            self._draw_broke(s)

    def _card_img(self, card, w=None, h=None):
        return self.renderer.get(card, w or self.cw, h or self.ch)

    def _draw_dealer(self, s):
        lbl = self._small.render(t("bj.dealer"), True, COL_DIM)
        s.blit(lbl, lbl.get_rect(midbottom=(self.width // 2,
                                            self.dealer_y - 8)))
        pos = self._dealer_positions(len(self.dealer))
        for i, card in enumerate(self.dealer):
            x, y = pos[i]
            if i == 1 and self.hole_hidden:
                s.blit(self.renderer.back(self.cw, self.ch), (x, y))
            elif i == 1 and self.state == DEALER and self.flip_t > 0:
                # Flip-Animation: Breite skaliert
                f = self.flip_t / FLIP_T
                w = max(2, int(self.cw * abs(2 * f - 1)))
                img = self.renderer.back(self.cw, self.ch) if f > 0.5 \
                    else self._card_img(card)
                img = pygame.transform.scale(img, (w, self.ch))
                s.blit(img, (x + (self.cw - w) // 2, y))
            else:
                s.blit(self._card_img(card), (x, y))
        if self.dealer and not self.hole_hidden and \
                not (self.state == DEALER and self.flip_t > 0):
            dv, _ = hand_value(self.dealer)
            img = self._big.render(str(dv), True,
                                   COL_BAD if dv > 21 else COL_TEXT)
            s.blit(img, (pos[-1][0] + self.cw + 12, self.dealer_y + 4))

    def _draw_player(self, s):
        for hi, h in enumerate(self.hands):
            pos = self._hand_positions(hi, len(h))
            for i, card in enumerate(h):
                s.blit(self._card_img(card), pos[i])
            if h:
                total, soft = hand_value(h)
                txt = f"{total}" + ("s" if soft and total <= 21 else "")
                col = COL_BAD if total > 21 else \
                    (COL_GOLD if total == 21 else COL_TEXT)
                img = self._big.render(txt, True, col)
                s.blit(img, (pos[-1][0] + self.cw + 12, self.player_y + 4))
                bet = self._tiny.render(f"{t('bj.bet')}: {self.hand_bets[hi]}",
                                        True, COL_DIM)
                s.blit(bet, (pos[0][0], self.player_y + self.ch + 6))
            # Pfeil auf aktive Hand (bei Split)
            if len(self.hands) > 1 and hi == self.active \
                    and self.state == PLAYER:
                k = abs(pygame.time.get_ticks() % 800 - 400) / 400
                ax = pos[0][0] - 18
                ay = self.player_y + self.ch // 2 + int(6 * k)
                pygame.draw.polygon(s, COL_GOLD,
                                    [(ax, ay - 8), (ax + 12, ay),
                                     (ax, ay + 8)])

    def _draw_topbar(self, s):
        img = self._big.render(f"{t('bj.chips')}: {self.chips}", True,
                               COL_GOLD)
        s.blit(img, (16, 12))
        img = self._small.render(f"{t('bj.best')}: {self.best}", True, COL_DIM)
        s.blit(img, (16, 40))

    def _draw_bet_ui(self, s):
        pygame.draw.rect(s, (14, 34, 26), self.strip)
        pygame.draw.line(s, COL_BTN_BORDER, self.strip.topleft,
                         self.strip.topright)
        for i, r in enumerate(self.chip_rects):
            val = BETS[i]
            col = CHIP_COLS[val]
            pygame.draw.circle(s, col, r.center, 22)
            pygame.draw.circle(s, (240, 240, 245), r.center, 22, 2)
            pygame.draw.circle(s, (240, 240, 245), r.center, 15, 1)
            img = self._tiny.render(str(val), True,
                                    (20, 24, 30) if val != 100 else COL_TEXT)
            s.blit(img, img.get_rect(center=r.center))
        mid = self._big.render(f"{t('bj.bet')}: {self.bet}", True, COL_TEXT)
        s.blit(mid, mid.get_rect(center=(self.width // 2,
                                         self.strip.centery)))
        for r, key, active in ((self.clear_rect, "bj.clear", self.bet > 0),
                               (self.deal_rect, "bj.deal",
                                self.bet >= BETS[0])):
            pygame.draw.rect(s, COL_BTN_ON if active else COL_BTN, r,
                             border_radius=10)
            pygame.draw.rect(s, COL_BTN_BORDER, r, 1, border_radius=10)
            img = self._small.render(t(key), True,
                                     COL_TEXT if active else COL_DIM)
            s.blit(img, img.get_rect(center=r.center))
        if self.bet < BETS[0]:
            hint = self._tiny.render(t("bj.min_bet", n=BETS[0]), True,
                                     COL_DIM)
            s.blit(hint, hint.get_rect(midbottom=(self.width // 2,
                                                  self.strip.y - 6)))

    def _draw_action_ui(self, s):
        pygame.draw.rect(s, (14, 34, 26), self.strip)
        pygame.draw.line(s, COL_BTN_BORDER, self.strip.topleft,
                         self.strip.topright)
        avail = {"hit": True, "stand": True,
                 "double": self._can_double(), "split": self._can_split()}
        keys = {"hit": "H", "stand": "S", "double": "D", "split": "X"}
        for key, r in self.action_rects.items():
            on = avail[key]
            pygame.draw.rect(s, COL_BTN_ON if on else COL_BTN, r,
                             border_radius=10)
            pygame.draw.rect(s, COL_BTN_BORDER, r, 1, border_radius=10)
            img = self._small.render(f"{t('bj.' + key)} ({keys[key]})", True,
                                     COL_TEXT if on else COL_DIM)
            s.blit(img, img.get_rect(center=r.center))

    def _draw_results(self, s):
        pygame.draw.rect(s, (14, 34, 26), self.strip)
        texts = []
        for i, res in enumerate(self.results):
            label = t("bj." + res)
            if len(self.results) > 1:
                label = f"{t('bj.hand')} {i + 1}: {label}"
            texts.append(label)
        img = self._big.render("   ·   ".join(texts), True, COL_GOLD)
        s.blit(img, img.get_rect(center=(self.width // 2,
                                         self.strip.centery - 10)))
        hint = self._tiny.render(t("common.enter_restart"), True, COL_DIM)
        s.blit(hint, hint.get_rect(center=(self.width // 2,
                                           self.strip.bottom - 14)))

    def _draw_broke(self, s):
        ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        ov.fill((10, 12, 22, 185))
        s.blit(ov, (0, 0))
        cx, cy = self.width // 2, self.height // 2
        head = self._huge.render(t("bj.broke"), True, COL_BAD)
        s.blit(head, head.get_rect(center=(cx, cy - 40)))
        sub = self._small.render(t("bj.broke_restart", n=START_CHIPS), True,
                                 COL_DIM)
        s.blit(sub, sub.get_rect(center=(cx, cy + 8)))
