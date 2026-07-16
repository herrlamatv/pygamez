# -*- coding: utf-8 -*-
"""
poker.py
========
Poker - drei waehlbare Varianten (Modusauswahl im Vorspiel):

- Texas Hold'em: gegen 1-3 KI-Gegner, mit Dealer-Button, Small/Big Blind und
  vier Setzrunden (Preflop, Flop, Turn, River). Aktionen: Fold, Check, Call,
  Raise, All-In. Beim Showdown gewinnt die beste 5-aus-7-Hand.
- 5 Card Draw: Heads-up gegen die KI. Ante, eine Setzrunde, Karten tauschen,
  zweite Setzrunde, Showdown.
- Video Poker (Jacks or Better): Solo gegen die Auszahlungstabelle. Einsatz,
  fuenf Karten, Halten waehlen, ziehen, Auszahlung nach Tabelle.

Chips: Start 1000, bleiben ueber Sitzungen erhalten (mem.json, Abschnitt
"poker"). Der Highscore ist der hoechste je erreichte Chipstand (game_over wird
nie gesetzt; die Sicherung erfolgt beim Menue-Rueckweg). Pleite = Neustart.

Vereinfachung: Bei All-In wird EIN gemeinsamer Haupt-Pot gefuehrt (keine
Side-Pots) - fuer ein lockeres Spiel gegen die KI voellig ausreichend.

Steuerung: Buttons anklicken oder F = Fold, C = Check/Call, R = Raise,
A = All-In, Enter/Leertaste = Geben/Weiter, 1-5 = Karte halten/tauschen.
"""

import itertools
import random
from collections import Counter

import pygame

import store
from game_base import Game, InputEvent
from i18n import t

from . import cards as C

# ---- Farben (gruener Filz + Gold)
COL_FELT = (18, 52, 40)
COL_FELT_EDGE = (12, 38, 28)
COL_TEXT = (228, 232, 238)
COL_DIM = (168, 190, 178)
COL_ACCENT = (232, 196, 92)     # = Sidebar-Farbe #e8c45c (Gold)
COL_OK = (110, 205, 140)
COL_BAD = (226, 96, 96)
COL_GOLD = (245, 205, 100)
COL_BTN = (28, 66, 52)
COL_BTN_ON = (44, 96, 74)
COL_BTN_BORDER = (76, 120, 98)
COL_POT = (240, 214, 130)
COL_HOLD = (245, 205, 100)

START_CHIPS = 1000
SMALL_BLIND = 10
BIG_BLIND = 20
ANTE = 10
VIDEO_BETS = (10, 25, 50)

# Kategorien (hoeher = besser)
HIGH, PAIR, TWO_PAIR, TRIPS, STRAIGHT, FLUSH, FULL_HOUSE, QUADS, STR_FLUSH = range(9)
CAT_KEY = {
    HIGH: "high_card", PAIR: "pair", TWO_PAIR: "two_pair", TRIPS: "trips",
    STRAIGHT: "straight", FLUSH: "flush", FULL_HOUSE: "full_house",
    QUADS: "quads", STR_FLUSH: "straight_flush",
}

# Video-Poker-Auszahlung (Jacks or Better), Multiplikator auf den Einsatz.
VIDEO_PAYOUT = [
    (STR_FLUSH, "royal", 250),          # Sonderfall Royal Flush (unten geprueft)
    (STR_FLUSH, None, 50),
    (QUADS, None, 25),
    (FULL_HOUSE, None, 9),
    (FLUSH, None, 6),
    (STRAIGHT, None, 4),
    (TRIPS, None, 3),
    (TWO_PAIR, None, 2),
    (PAIR, "jacks", 1),                 # nur Buben oder besser
]


def _val(card):
    return 14 if card.rank == 1 else card.rank


def eval5(cards):
    """Bewertet genau 5 Karten -> vergleichbares Tupel (hoeher = besser)."""
    vals = sorted((_val(c) for c in cards), reverse=True)
    suits = [c.suit for c in cards]
    counts = Counter(vals)
    # (Anzahl, Wert) absteigend sortiert -> bestimmt Paare/Drillinge etc.
    by_count = sorted(counts.items(), key=lambda kv: (kv[1], kv[0]), reverse=True)
    is_flush = len(set(suits)) == 1
    uniq = sorted(set(vals), reverse=True)
    straight_high = None
    if len(uniq) == 5:
        if uniq[0] - uniq[4] == 4:
            straight_high = uniq[0]
        elif uniq == [14, 5, 4, 3, 2]:      # Wheel (A-2-3-4-5)
            straight_high = 5

    if is_flush and straight_high:
        return (STR_FLUSH, straight_high)
    if by_count[0][1] == 4:
        quad = by_count[0][0]
        kicker = max(v for v in vals if v != quad)
        return (QUADS, quad, kicker)
    if by_count[0][1] == 3 and by_count[1][1] == 2:
        return (FULL_HOUSE, by_count[0][0], by_count[1][0])
    if is_flush:
        return (FLUSH, tuple(vals))
    if straight_high:
        return (STRAIGHT, straight_high)
    if by_count[0][1] == 3:
        kick = tuple(v for v in vals if v != by_count[0][0])
        return (TRIPS, by_count[0][0]) + kick
    if by_count[0][1] == 2 and by_count[1][1] == 2:
        hp, lp = by_count[0][0], by_count[1][0]
        kicker = max(v for v in vals if v != hp and v != lp)
        return (TWO_PAIR, hp, lp, kicker)
    if by_count[0][1] == 2:
        kick = tuple(v for v in vals if v != by_count[0][0])
        return (PAIR, by_count[0][0]) + kick
    return (HIGH, tuple(vals))


def best_hand(cards):
    """Beste 5-Karten-Bewertung aus 5..7 Karten -> (rang_tupel, [beste5])."""
    if len(cards) <= 5:
        return eval5(cards), list(cards)
    best = None
    best5 = None
    for combo in itertools.combinations(cards, 5):
        r = eval5(combo)
        if best is None or r > best:
            best = r
            best5 = list(combo)
    return best, best5


def is_royal(rank_tuple):
    return rank_tuple[0] == STR_FLUSH and rank_tuple[1] == 14


def category_key(rank_tuple):
    if is_royal(rank_tuple):
        return "royal"
    return CAT_KEY[rank_tuple[0]]


def hand_strength(cards):
    """Grober Staerke-Score 0..1 fuer die KI (Kategorie + hoechste Karte)."""
    rt, _ = best_hand(cards)
    top = rt[1] if len(rt) > 1 and isinstance(rt[1], int) else 10
    if isinstance(rt[1], tuple):
        top = rt[1][0]
    return min(1.0, rt[0] / 8.0 * 0.82 + (top / 14.0) * 0.18)


def preflop_strength(hole):
    """Hole-Card-Staerke 0..1 fuer Texas Hold'em (Chen-artig, normiert)."""
    a, b = hole
    va, vb = _val(a), _val(b)
    hi, lo = max(va, vb), min(va, vb)
    score = hi / 14.0 * 0.5
    if va == vb:                     # Paar
        score = 0.5 + (va / 14.0) * 0.5
    else:
        if a.suit == b.suit:
            score += 0.12
        gap = hi - lo
        if gap == 1:
            score += 0.08
        elif gap == 2:
            score += 0.04
        score += lo / 14.0 * 0.15
    return max(0.0, min(1.0, score))


class Player:
    __slots__ = ("name", "stack", "hole", "folded", "all_in", "round_bet",
                 "is_human", "result")

    def __init__(self, name, stack, is_human=False):
        self.name = name
        self.stack = stack
        self.hole = []
        self.folded = False
        self.all_in = False
        self.round_bet = 0
        self.is_human = is_human
        self.result = None


# Phasen
PREHAND, BET_VIDEO, ACTING, DRAW_SELECT, VIDEO_HOLD, SHOWDOWN, BROKE = \
    "prehand", "bet_video", "acting", "draw_select", "video_hold", \
    "showdown", "broke"


class PokerGame(Game):
    name = "Poker"
    highscore_key = "poker"
    supports_multiplayer = False

    MODES = [("holdem", "poker.mode.holdem"),
             ("draw", "poker.mode.draw"),
             ("video", "poker.mode.video")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.game_over = False
        if self.mode not in ("holdem", "draw", "video"):
            self.mode = "holdem"

        data = store.load_section("poker")
        try:
            self.chips = max(0, int(data.get("chips", START_CHIPS)))
        except (TypeError, ValueError):
            self.chips = START_CHIPS
        try:
            self.best = max(self.chips, int(data.get("best", START_CHIPS)))
        except (TypeError, ValueError):
            self.best = max(self.chips, START_CHIPS)
        self.score = self.best

        pk = self.settings.get("poker", {}) if isinstance(self.settings, dict) else {}
        self.n_opponents = max(1, min(3, int(pk.get("opponents", 2))))
        self.ai_level = max(0, min(2, int(pk.get("difficulty", 1))))
        if self.mode == "draw":
            self.n_opponents = 1
        elif self.mode == "video":
            self.n_opponents = 0

        self._small = pygame.font.SysFont("consolas", 15)
        self._tiny = pygame.font.SysFont("consolas", 13)
        self._big = pygame.font.SysFont("consolas", 22, bold=True)
        self._huge = pygame.font.SysFont("consolas", max(24, self.height // 12),
                                         bold=True)
        self.renderer = C.CardRenderer(COL_ACCENT)

        self.deck = []
        self.community = []
        self.pot = 0
        self.players = []
        self.dealer = 0
        self.turn = 0
        self.current_bet = 0
        self.min_raise = BIG_BLIND
        self.to_act_set = set()
        self.street = "preflop"
        self.msg = ""
        self.act_delay = 0.0
        self.reveal = False
        self.holds = set()             # gehaltene/zu behaltende Karten (video/draw)
        self.video_bet = VIDEO_BETS[0]
        self.win_amount = 0
        self._layout()

        self.phase = BET_VIDEO if self.mode == "video" else PREHAND
        if self.chips < BIG_BLIND and self.mode != "video":
            self.phase = BROKE
        if self.mode == "video" and self.chips < VIDEO_BETS[0]:
            self.phase = BROKE

    def on_surface_changed(self):
        self._huge = pygame.font.SysFont("consolas", max(24, self.height // 12),
                                         bold=True)
        self.renderer.clear()
        self._layout()

    def _layout(self):
        self.ch = int(self.height * 0.19)
        self.cw = int(self.ch * 0.72)
        self.strip = pygame.Rect(0, self.height - 66, self.width, 66)
        # Aktions-Buttons
        labels = ("fold", "call", "raise", "allin")
        bw = int(self.width * 0.17)
        gap = (self.width - 4 * bw) // 5
        self.action_rects = {}
        for i, key in enumerate(labels):
            self.action_rects[key] = pygame.Rect(
                gap + i * (bw + gap), self.strip.y + 12, bw, 42)
        # Deal-/Weiter-Button (mittig)
        self.deal_rect = pygame.Rect(self.width // 2 - 90, self.strip.y + 12,
                                     180, 42)
        # Video/Draw-Bet-Chips
        self.chip_rects = [pygame.Rect(24 + i * 54, self.strip.y + 12, 42, 42)
                           for i in range(len(VIDEO_BETS))]

    def _save(self):
        store.save_section("poker", {"chips": self.chips, "best": self.best})

    def _sync_chips(self):
        """Uebernimmt den Table-Stack des Menschen zurueck in die Bank."""
        me = self.players[0] if self.players else None
        if me is not None:
            self.chips = me.stack
        if self.chips > self.best:
            self.best = self.chips
        self.score = self.best
        self._save()

    # ===================================================== Deck
    def _fresh_deck(self):
        self.deck = C.make_deck()
        C.shuffle(self.deck, random.Random())

    def _draw(self, face_up=True):
        card = self.deck.pop()
        card.face_up = face_up
        return card

    # ===================================================== Hand-Start
    def _start_hand(self):
        if self.chips < BIG_BLIND:
            self.phase = BROKE
            self.play_sound("gameover")
            return
        self._fresh_deck()
        self.community = []
        self.pot = 0
        self.reveal = False
        self.holds = set()
        self.msg = ""
        self.win_amount = 0
        names = [t("poker.you")] + [t("poker.cpu", n=i + 1)
                                    for i in range(self.n_opponents)]
        stacks = [self.chips] + [max(BIG_BLIND * 20, self.chips)
                                 for _ in range(self.n_opponents)]
        self.players = [Player(names[i], stacks[i], is_human=(i == 0))
                        for i in range(1 + self.n_opponents)]

        if self.mode == "holdem":
            self._start_holdem()
        elif self.mode == "draw":
            self._start_draw()

    # ----- Texas Hold'em -----------------------------------------------
    def _start_holdem(self):
        n = len(self.players)
        self.dealer = self.dealer % n
        for _ in range(2):
            for p in self.players:
                p.hole.append(self._draw(face_up=p.is_human))
        sb = (self.dealer + 1) % n
        bb = (self.dealer + 2) % n
        if n == 2:                       # Heads-up: Dealer = SB
            sb = self.dealer
            bb = (self.dealer + 1) % n
        self._post(self.players[sb], SMALL_BLIND)
        self._post(self.players[bb], BIG_BLIND)
        self.current_bet = BIG_BLIND
        self.min_raise = BIG_BLIND
        self.street = "preflop"
        self.to_act_set = {i for i, p in enumerate(self.players)
                           if not p.all_in}
        self.turn = (bb + 1) % n
        self.phase = ACTING
        self.msg = t("poker.preflop")
        self.play_sound("move")
        self._begin_turn()

    def _post(self, player, amount):
        amount = min(amount, player.stack)
        player.stack -= amount
        player.round_bet += amount
        self.pot += amount
        if player.stack == 0:
            player.all_in = True

    # ----- 5 Card Draw --------------------------------------------------
    def _start_draw(self):
        for p in self.players:
            self._post(p, ANTE)          # Ante von allen
        for _ in range(5):
            for p in self.players:
                p.hole.append(self._draw(face_up=p.is_human))
        self.current_bet = 0
        self.min_raise = BIG_BLIND
        self.street = "draw1"
        self.to_act_set = {i for i, p in enumerate(self.players)
                           if not p.all_in}
        self.turn = (self.dealer + 1) % len(self.players)
        self.phase = ACTING
        self.msg = t("poker.round1")
        self.play_sound("move")
        self._begin_turn()

    # ===================================================== Setzrunde
    def _begin_turn(self):
        """Setzt act_delay fuer die KI oder wartet auf den Menschen."""
        contenders = [p for p in self.players if not p.folded]
        if len(contenders) <= 1:
            self._end_hand()
            return
        if not self.to_act_set:
            self._end_round()
            return
        if self.turn not in self.to_act_set:
            self._advance_turn()
            return
        p = self.players[self.turn]
        self.act_delay = 0.0 if p.is_human else 0.7

    def _advance_turn(self):
        n = len(self.players)
        if not self.to_act_set:
            self._end_round()
            return
        for i in range(1, n + 1):
            cand = (self.turn + i) % n
            if cand in self.to_act_set:
                self.turn = cand
                self._begin_turn()
                return
        self._end_round()

    def _to_call(self, player):
        return max(0, self.current_bet - player.round_bet)

    def _do_action(self, action, raise_to=None):
        p = self.players[self.turn]
        if action == "fold":
            p.folded = True
            self.to_act_set.discard(self.turn)
            self.play_sound("select")
        elif action == "check":
            self.to_act_set.discard(self.turn)
            self.play_sound("click")
        elif action == "call":
            need = min(self._to_call(p), p.stack)
            self._post(p, need)
            self.to_act_set.discard(self.turn)
            self.play_sound("point")
        elif action == "raise":
            target = raise_to if raise_to is not None else \
                self.current_bet + self.min_raise
            target = min(target, p.round_bet + p.stack)   # nicht mehr als Stack
            self._post(p, target - p.round_bet)
            self.min_raise = max(self.min_raise, target - self.current_bet)
            self.current_bet = max(self.current_bet, p.round_bet)
            # alle anderen aktiven muessen erneut handeln
            self.to_act_set = {i for i, q in enumerate(self.players)
                               if not q.folded and not q.all_in and i != self.turn}
            self.play_sound("point")
        elif action == "allin":
            self._post(p, p.stack)
            if p.round_bet > self.current_bet:
                self.min_raise = max(self.min_raise, p.round_bet - self.current_bet)
                self.current_bet = p.round_bet
                self.to_act_set = {i for i, q in enumerate(self.players)
                                   if not q.folded and not q.all_in and i != self.turn}
            else:
                self.to_act_set.discard(self.turn)
            self.play_sound("point")
        self._advance_turn()

    def _end_round(self):
        for p in self.players:
            p.round_bet = 0
        self.current_bet = 0
        self.min_raise = BIG_BLIND
        contenders = [p for p in self.players if not p.folded]
        if len(contenders) <= 1:
            self._end_hand()
            return
        if self.mode == "holdem":
            self._next_street()
        else:
            if self.street == "draw1":
                self._enter_draw_phase()
            else:
                self._showdown()

    def _next_street(self):
        actionable = [p for p in self.players if not p.folded and not p.all_in]
        # Wenn <=1 handlungsfaehig: restliche Karten aufdecken, Showdown.
        fast = len(actionable) <= 1
        if self.street == "preflop":
            self.community = [self._draw() for _ in range(3)]
            self.street = "flop"
            self.msg = t("poker.flop")
        elif self.street == "flop":
            self.community.append(self._draw())
            self.street = "turn"
            self.msg = t("poker.turn")
        elif self.street == "turn":
            self.community.append(self._draw())
            self.street = "river"
            self.msg = t("poker.river")
        else:
            self._showdown()
            return
        self.play_sound("move")
        if fast:
            self._next_street() if self.street != "river" else self._showdown()
            return
        self.to_act_set = {i for i, p in enumerate(self.players)
                           if not p.folded and not p.all_in}
        self.turn = self.dealer
        self._advance_turn()

    # ----- Draw-Phase (Karten tauschen) ---------------------------------
    def _enter_draw_phase(self):
        self.phase = DRAW_SELECT
        self.holds = set()
        self.msg = t("poker.draw_hint")
        self.act_delay = 0.0

    def _do_human_draw(self):
        me = self.players[0]
        new_hole = []
        for i, card in enumerate(me.hole):
            if i in self.holds:
                new_hole.append(card)
            else:
                nc = self._draw(face_up=True)
                new_hole.append(nc)
        me.hole = new_hole
        self.holds = set()
        self.play_sound("rotate")
        # KI tauscht
        for p in self.players[1:]:
            if p.folded:
                continue
            self._ai_draw(p)
        # zweite Setzrunde
        self.current_bet = 0
        self.min_raise = BIG_BLIND
        self.street = "draw2"
        self.to_act_set = {i for i, p in enumerate(self.players)
                           if not p.folded and not p.all_in}
        self.turn = (self.dealer + 1) % len(self.players)
        self.phase = ACTING
        self.msg = t("poker.round2")
        self._begin_turn()

    def _ai_draw(self, p):
        """KI entscheidet, welche Karten sie behaelt (einfache Heuristik)."""
        vals = [_val(c) for c in p.hole]
        counts = Counter(vals)
        suits = Counter(c.suit for c in p.hole)
        keep = set()
        # Paare/Drillinge/Vierlinge behalten
        for i, c in enumerate(p.hole):
            if counts[_val(c)] >= 2:
                keep.add(i)
        # Flush-Ansatz (4 gleiche Farbe) behalten
        fs = suits.most_common(1)[0]
        if fs[1] >= 4:
            keep = {i for i, c in enumerate(p.hole) if c.suit == fs[0]}
        if not keep:                          # sonst hohe Karten behalten
            for i, c in enumerate(p.hole):
                if _val(c) >= 12:
                    keep.add(i)
        new = []
        for i, c in enumerate(p.hole):
            new.append(c if i in keep else self._draw(face_up=False))
        p.hole = new

    # ===================================================== KI-Setzen
    def _ai_action(self, p):
        idx = self.players.index(p)
        to_call = self._to_call(p)
        if self.mode == "holdem":
            if self.street == "preflop" and not self.community:
                strength = preflop_strength(p.hole)
            else:
                strength = hand_strength(p.hole + self.community)
        else:
            strength = hand_strength(p.hole)
        # Schwierigkeit steuert Aggressivitaet / Bluff.
        aggro = (0.05, 0.14, 0.22)[self.ai_level]
        bluff = (0.03, 0.06, 0.10)[self.ai_level]
        r = random.random()

        if to_call == 0:
            # Check oder Setzen
            if strength > 0.6 or r < aggro:
                self._ai_raise(p)
            else:
                self._do_action("check")
            return
        # Es kostet etwas zu bleiben
        pot_odds = to_call / (self.pot + to_call + 1e-9)
        if strength < 0.28 and r > bluff:
            if to_call > p.stack * 0.12:
                self._do_action("fold")
            else:
                self._do_action("call")
            return
        if strength > 0.72 and r < 0.5 + aggro:
            self._ai_raise(p)
            return
        if strength < pot_odds - 0.05 and r > bluff:
            self._do_action("fold")
            return
        self._do_action("call")

    def _ai_raise(self, p):
        raise_to = self.current_bet + max(self.min_raise,
                                          int(self.pot * 0.6) or BIG_BLIND)
        if raise_to >= p.round_bet + p.stack:
            self._do_action("allin")
        else:
            self._do_action("raise", raise_to)

    # ===================================================== Showdown / Ende
    def _showdown(self):
        self.reveal = True
        for p in self.players:
            for c in p.hole:
                c.face_up = True
        contenders = [p for p in self.players if not p.folded]
        ranked = []
        for p in contenders:
            cards = p.hole + self.community if self.mode == "holdem" else p.hole
            rt, _ = best_hand(cards)
            p.result = rt
            ranked.append((rt, p))
        best_rt = max(r for r, _ in ranked)
        winners = [p for rt, p in ranked if rt == best_rt]
        self._award(winners, best_rt)
        self.phase = SHOWDOWN

    def _end_hand(self):
        """Nur noch ein Spieler uebrig (alle anderen gefoldet)."""
        contenders = [p for p in self.players if not p.folded]
        self._award(contenders, None)
        self.phase = SHOWDOWN

    def _award(self, winners, rank_tuple):
        share = self.pot // len(winners)
        rem = self.pot - share * len(winners)
        for i, w in enumerate(winners):
            w.stack += share + (rem if i == 0 else 0)
        me = self.players[0]
        self.win_amount = (self.pot if me in winners else 0)
        if me in winners:
            if rank_tuple is not None:
                self.msg = t("poker.you_win_hand",
                             hand=t("poker.hand." + category_key(rank_tuple)))
            else:
                self.msg = t("poker.you_win")
            self.play_sound("win")
        else:
            self.msg = t("poker.you_lose")
            self.play_sound("gameover")
        self.reveal = True
        self.rank_shown = rank_tuple
        self.winner_names = [w.name for w in winners]
        self.dealer = (self.dealer + 1) % max(1, len(self.players))
        self._sync_chips()

    # ----- Video Poker --------------------------------------------------
    def _video_deal(self):
        if self.chips < self.video_bet:
            self.phase = BROKE
            return
        self.chips -= self.video_bet
        self._save()
        self._fresh_deck()
        self.players = [Player(t("poker.you"), self.chips, is_human=True)]
        self.players[0].hole = [self._draw(face_up=True) for _ in range(5)]
        self.holds = set()
        self.phase = VIDEO_HOLD
        self.msg = t("poker.hold_hint")
        self.win_amount = 0
        self.play_sound("move")

    def _video_draw(self):
        me = self.players[0]
        new = []
        for i, c in enumerate(me.hole):
            new.append(c if i in self.holds else self._draw(face_up=True))
        me.hole = new
        rt, _ = best_hand(me.hole)
        mult = self._video_payout(rt)
        self.win_amount = self.video_bet * mult
        self.chips += self.win_amount
        if self.chips > self.best:
            self.best = self.chips
        self.score = self.best
        self._save()
        if mult > 0:
            self.msg = t("poker.video_win",
                         hand=t("poker.hand." + category_key(rt)),
                         mult=mult)
            self.play_sound("win" if mult >= 6 else "point")
        else:
            self.msg = t("poker.video_none")
            self.play_sound("gameover")
        self.rank_shown = rt
        self.phase = SHOWDOWN

    def _video_payout(self, rt):
        for cat, special, mult in VIDEO_PAYOUT:
            if rt[0] != cat:
                continue
            if special == "royal":
                if is_royal(rt):
                    return mult
                continue
            if special == "jacks":
                if rt[0] == PAIR and rt[1] >= 11:
                    return mult
                continue
            return mult
        return 0

    # ===================================================== Update
    def update(self, dt):
        if self.phase == ACTING:
            p = self.players[self.turn]
            if not p.is_human and not p.folded and not p.all_in:
                self.act_delay -= dt
                if self.act_delay <= 0:
                    self._ai_action(p)

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.phase == BROKE:
            if self._is_confirm(event):
                self.chips = START_CHIPS
                self._save()
                self.phase = BET_VIDEO if self.mode == "video" else PREHAND
                self.play_sound("click")
            return
        if self.phase == PREHAND:
            if self._is_confirm(event):
                self._start_hand()
            return
        if self.phase == BET_VIDEO:
            self._handle_video_bet(event)
            return
        if self.phase == VIDEO_HOLD:
            self._handle_holds(event, self._video_draw)
            return
        if self.phase == DRAW_SELECT:
            self._handle_holds(event, self._do_human_draw)
            return
        if self.phase == SHOWDOWN:
            if self._is_confirm(event):
                self._next_after_showdown()
            return
        if self.phase == ACTING:
            self._handle_acting(event)

    def _is_confirm(self, event):
        return (event.kind == InputEvent.MOUSEDOWN or
                (event.kind == InputEvent.KEYDOWN and
                 event.key in ("Return", "space")))

    def _next_after_showdown(self):
        for p in self.players:
            p.hole = []
            p.folded = False
            p.all_in = False
            p.round_bet = 0
            p.result = None
        if self.mode == "video":
            self.phase = BET_VIDEO if self.chips >= VIDEO_BETS[0] else BROKE
        else:
            self.phase = PREHAND if self.chips >= BIG_BLIND else BROKE
        if self.phase == BROKE:
            self.play_sound("gameover")

    def _handle_video_bet(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3") and int(k) <= len(VIDEO_BETS):
                self.video_bet = VIDEO_BETS[int(k) - 1]
                self.play_sound("click")
            elif k in ("Return", "space"):
                self._video_deal()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, r in enumerate(self.chip_rects):
                if r.collidepoint(event.pos):
                    self.video_bet = VIDEO_BETS[i]
                    self.play_sound("click")
                    return
            if self.deal_rect.collidepoint(event.pos):
                self._video_deal()

    def _handle_holds(self, event, on_confirm):
        me = self.players[0]
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3", "4", "5"):
                i = int(k) - 1
                if i < len(me.hole):
                    self.holds.symmetric_difference_update({i})
                    self.play_sound("click")
            elif k in ("Return", "space"):
                on_confirm()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, rect in enumerate(self._hole_rects(me, center=True)):
                if rect.collidepoint(event.pos):
                    self.holds.symmetric_difference_update({i})
                    self.play_sound("click")
                    return
            if self.deal_rect.collidepoint(event.pos):
                on_confirm()

    def _handle_acting(self, event):
        p = self.players[self.turn]
        if not p.is_human:
            return
        to_call = self._to_call(p)
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("f", "F"):
                self._do_action("fold")
            elif k in ("c", "C"):
                self._do_action("check" if to_call == 0 else "call")
            elif k in ("r", "R"):
                self._human_raise()
            elif k in ("a", "A"):
                self._do_action("allin")
        elif event.kind == InputEvent.MOUSEDOWN:
            if self.action_rects["fold"].collidepoint(event.pos):
                self._do_action("fold")
            elif self.action_rects["call"].collidepoint(event.pos):
                self._do_action("check" if to_call == 0 else "call")
            elif self.action_rects["raise"].collidepoint(event.pos):
                self._human_raise()
            elif self.action_rects["allin"].collidepoint(event.pos):
                self._do_action("allin")

    def _human_raise(self):
        p = self.players[self.turn]
        raise_to = self.current_bet + max(self.min_raise,
                                          int(self.pot * 0.5) or BIG_BLIND)
        if raise_to >= p.round_bet + p.stack:
            self._do_action("allin")
        else:
            self._do_action("raise", raise_to)

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        s.fill(COL_FELT)
        pygame.draw.rect(s, COL_FELT_EDGE, (0, 0, self.width, self.height), 6)
        self._draw_topbar(s)
        if self.phase == BROKE:
            self._draw_broke(s)
            return
        if self.phase == BET_VIDEO:
            self._draw_video_bet(s)
            return
        if self.phase == PREHAND or not self.players:
            self._draw_prehand(s)
            return
        if self.mode == "video":
            self._draw_video(s)
        else:
            self._draw_table(s)

    def _draw_topbar(self, s):
        img = self._big.render(f"{t('poker.chips')}: {self.chips}", True, COL_GOLD)
        s.blit(img, (16, 12))
        img = self._small.render(f"{t('poker.best')}: {self.best}", True, COL_DIM)
        s.blit(img, (16, 40))
        mode_lbl = t("poker.mode." + self.mode)
        mi = self._small.render(mode_lbl, True, COL_ACCENT)
        s.blit(mi, mi.get_rect(topright=(self.width - 16, 14)))
        if self.pot and self.phase not in (BET_VIDEO, PREHAND):
            pi = self._big.render(f"{t('poker.pot')}: {self.pot}", True, COL_POT)
            s.blit(pi, pi.get_rect(center=(self.width // 2, 24)))

    def _hole_rects(self, player, center=False):
        n = len(player.hole)
        if n == 0:
            return []
        dx = int(self.cw * 1.12)
        total = self.cw + (n - 1) * dx
        cx = self.width // 2
        y = int(self.height * 0.62) if (center or self.mode == "video") \
            else int(self.height * 0.60)
        x0 = cx - total // 2
        return [pygame.Rect(x0 + i * dx, y, self.cw, self.ch) for i in range(n)]

    def _draw_table(self, s):
        # Gegner oben
        opp = self.players[1:]
        for i, p in enumerate(opp):
            cx = int(self.width * (i + 1) / (len(opp) + 1))
            self._draw_opponent(s, p, cx, int(self.height * 0.20),
                                i + 1 == self.turn and self.phase == ACTING)
        # Community
        if self.community:
            self._draw_community(s)
        # eigene Hand
        me = self.players[0]
        highlight_hold = self.phase == DRAW_SELECT
        for i, rect in enumerate(self._hole_rects(me)):
            s.blit(self.renderer.get(me.hole[i], self.cw, self.ch), rect)
            if highlight_hold and i in self.holds:
                pygame.draw.rect(s, COL_HOLD, rect, 3, border_radius=6)
                lbl = self._tiny.render(t("poker.keep"), True, COL_HOLD)
                s.blit(lbl, lbl.get_rect(midtop=(rect.centerx, rect.bottom + 2)))
        # Dealer-Button + Namen
        me_label = f"{t('poker.you')}"
        if me.folded:
            me_label += f"  ({t('poker.folded')})"
        li = self._small.render(me_label, True,
                                COL_ACCENT if (self.turn == 0 and
                                               self.phase == ACTING) else COL_DIM)
        s.blit(li, li.get_rect(midtop=(self.width // 2,
                                       int(self.height * 0.60) - 22)))

        # Strip / Buttons
        if self.phase == ACTING and self.players[self.turn].is_human:
            self._draw_actions(s)
        elif self.phase == DRAW_SELECT:
            self._draw_confirm(s, t("poker.draw_btn"))
        elif self.phase == SHOWDOWN:
            self._draw_result_strip(s)
        if self.msg:
            self._draw_msg(s)

    def _draw_opponent(self, s, p, cx, cy, active):
        w = int(self.cw * 0.62)
        h = int(self.ch * 0.62)
        dx = int(w * 0.5)
        x0 = cx - (w + dx) // 2
        for i, card in enumerate(p.hole):
            img = self.renderer.get(card, w, h) if (card.face_up or self.reveal) \
                else self.renderer.back(w, h)
            s.blit(img, (x0 + i * dx, cy))
        name = p.name + (f" ({t('poker.folded')})" if p.folded else "")
        col = COL_ACCENT if active else (COL_DIM if not p.folded else COL_BAD)
        ni = self._tiny.render(name, True, col)
        s.blit(ni, ni.get_rect(midtop=(cx, cy - 16)))
        si = self._tiny.render(f"{p.stack}", True, COL_GOLD)
        s.blit(si, si.get_rect(midtop=(cx, cy + h + 2)))
        if p.round_bet:
            bi = self._tiny.render(f"+{p.round_bet}", True, COL_POT)
            s.blit(bi, bi.get_rect(midtop=(cx, cy + h + 18)))
        if self.phase == SHOWDOWN and p.result and not p.folded:
            hi = self._tiny.render(t("poker.hand." + category_key(p.result)),
                                   True, COL_TEXT)
            s.blit(hi, hi.get_rect(midtop=(cx, cy + h + 18)))

    def _draw_community(self, s):
        n = len(self.community)
        w, h = self.cw, self.ch
        dx = int(w * 1.12)
        total = w + (n - 1) * dx
        x0 = self.width // 2 - total // 2
        y = int(self.height * 0.38)
        for i, card in enumerate(self.community):
            s.blit(self.renderer.get(card, w, h), (x0 + i * dx, y))

    def _draw_actions(self, s):
        pygame.draw.rect(s, (12, 40, 30), self.strip)
        pygame.draw.line(s, COL_BTN_BORDER, self.strip.topleft, self.strip.topright)
        p = self.players[self.turn]
        to_call = self._to_call(p)
        call_lbl = t("poker.check") if to_call == 0 else \
            t("poker.call_n", n=min(to_call, p.stack))
        labels = {"fold": t("poker.fold"), "call": call_lbl,
                  "raise": t("poker.raise"), "allin": t("poker.allin")}
        keys = {"fold": "F", "call": "C", "raise": "R", "allin": "A"}
        for key, r in self.action_rects.items():
            pygame.draw.rect(s, COL_BTN_ON, r, border_radius=10)
            pygame.draw.rect(s, COL_BTN_BORDER, r, 1, border_radius=10)
            img = self._small.render(f"{labels[key]} ({keys[key]})", True, COL_TEXT)
            s.blit(img, img.get_rect(center=r.center))

    def _draw_confirm(self, s, label):
        pygame.draw.rect(s, (12, 40, 30), self.strip)
        pygame.draw.line(s, COL_BTN_BORDER, self.strip.topleft, self.strip.topright)
        pygame.draw.rect(s, COL_BTN_ON, self.deal_rect, border_radius=10)
        pygame.draw.rect(s, COL_ACCENT, self.deal_rect, 2, border_radius=10)
        img = self._small.render(label, True, COL_TEXT)
        s.blit(img, img.get_rect(center=self.deal_rect.center))

    def _draw_result_strip(self, s):
        self._draw_confirm(s, t("poker.next_hand"))

    def _draw_msg(self, s):
        img = self._small.render(self.msg, True, COL_GOLD)
        s.blit(img, img.get_rect(center=(self.width // 2, self.strip.y - 16)))

    # ----- Video Poker zeichnen -----------------------------------------
    def _draw_video(self, s):
        me = self.players[0]
        for i, rect in enumerate(self._hole_rects(me, center=True)):
            s.blit(self.renderer.get(me.hole[i], self.cw, self.ch), rect)
            held = i in self.holds
            if self.phase == VIDEO_HOLD and held:
                pygame.draw.rect(s, COL_HOLD, rect, 3, border_radius=6)
                lbl = self._small.render(t("poker.held"), True, COL_HOLD)
                s.blit(lbl, lbl.get_rect(midbottom=(rect.centerx, rect.y - 3)))
            num = self._tiny.render(str(i + 1), True, COL_DIM)
            s.blit(num, num.get_rect(midtop=(rect.centerx, rect.bottom + 3)))
        self._draw_paytable(s)
        if self.phase == VIDEO_HOLD:
            self._draw_confirm(s, t("poker.draw_btn"))
        elif self.phase == SHOWDOWN:
            self._draw_confirm(s, t("poker.deal_btn"))
        if self.msg:
            self._draw_msg(s)

    def _draw_paytable(self, s):
        rows = [("royal", 250), ("straight_flush", 50), ("quads", 25),
                ("full_house", 9), ("flush", 6), ("straight", 4),
                ("trips", 3), ("two_pair", 2), ("jacks", 1)]
        x = 16
        y = int(self.height * 0.16)
        title = self._tiny.render(t("poker.paytable"), True, COL_ACCENT)
        s.blit(title, (x, y - 18))
        for name, mult in rows:
            key = "poker.hand." + name if name != "jacks" else "poker.jacks"
            lbl = self._tiny.render(f"{t(key)}", True, COL_DIM)
            s.blit(lbl, (x, y))
            mi = self._tiny.render(f"x{mult}", True, COL_GOLD)
            s.blit(mi, (x + 150, y))
            y += 16

    def _draw_video_bet(self, s):
        cx = self.width // 2
        title = self._huge.render(t("poker.mode.video"), True, COL_ACCENT)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.22))))
        self._draw_paytable(s)
        # Chips
        pygame.draw.rect(s, (12, 40, 30), self.strip)
        pygame.draw.line(s, COL_BTN_BORDER, self.strip.topleft, self.strip.topright)
        for i, r in enumerate(self.chip_rects):
            val = VIDEO_BETS[i]
            on = (val == self.video_bet)
            pygame.draw.circle(s, COL_GOLD if on else COL_BTN_ON, r.center, 21)
            pygame.draw.circle(s, (240, 240, 245), r.center, 21, 2)
            img = self._tiny.render(str(val), True,
                                    (20, 24, 30) if on else COL_TEXT)
            s.blit(img, img.get_rect(center=r.center))
        mid = self._big.render(f"{t('poker.bet')}: {self.video_bet}", True, COL_TEXT)
        s.blit(mid, mid.get_rect(center=(cx, self.strip.centery)))
        pygame.draw.rect(s, COL_BTN_ON, self.deal_rect, border_radius=10)
        pygame.draw.rect(s, COL_ACCENT, self.deal_rect, 2, border_radius=10)
        di = self._small.render(t("poker.deal_btn"), True, COL_TEXT)
        s.blit(di, di.get_rect(center=self.deal_rect.center))

    def _draw_broke(self, s):
        ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        ov.fill((8, 16, 12, 200))
        s.blit(ov, (0, 0))
        cx, cy = self.width // 2, self.height // 2
        head = self._huge.render(t("poker.broke"), True, COL_BAD)
        s.blit(head, head.get_rect(center=(cx, cy - 34)))
        sub = self._small.render(t("poker.broke_restart", n=START_CHIPS), True,
                                 COL_DIM)
        s.blit(sub, sub.get_rect(center=(cx, cy + 8)))

    def _draw_prehand(self, s):
        cx = self.width // 2
        title = self._huge.render(t("poker.mode." + self.mode), True, COL_ACCENT)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.28))))
        info = [t("poker.blinds", sb=SMALL_BLIND, bb=BIG_BLIND)] \
            if self.mode == "holdem" else [t("poker.ante", n=ANTE)]
        if self.mode == "holdem":
            info.append(t("poker.opponents", n=self.n_opponents))
        for i, line in enumerate(info):
            im = self._small.render(line, True, COL_DIM)
            s.blit(im, im.get_rect(center=(cx, int(self.height * 0.40) + i * 22)))
        pygame.draw.rect(s, COL_BTN_ON, self.deal_rect, border_radius=10)
        pygame.draw.rect(s, COL_ACCENT, self.deal_rect, 2, border_radius=10)
        di = self._small.render(t("poker.deal_btn"), True, COL_TEXT)
        s.blit(di, di.get_rect(center=self.deal_rect.center))
