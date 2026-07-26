# -*- coding: utf-8 -*-
"""
solitaire.py
============
Solitär - fünf Varianten unter einem Dach (Auswahl im Vorspiel-Screen):

- KLONDIKE : das klassische Solitär; Ziehen von 1 oder 3 Karten (Option).
- SPIDER   : 10 Spalten, 104 Karten; 1/2/4 Farben (Option); K->A-Ketten
             gleicher Farbe wandern ins Fundament.
- FREECELL : alles offen, 4 freie Zellen; Supermove-Limit
             (frei+1) * 2^leere Spalten (Ziel leer -> halbiert).
- PYRAMID  : Paare mit Summe 13 abtragen (König allein); 2 Redeals.
- TRIPEAKS : Waste-Kette mit +/-1 (A<->K wrappt); Combo-Multiplikator.

Bedienung: Karten mit der Maus ZIEHEN (Drag & Drop) oder per Klick-Klick;
Rechtsklick schickt die oberste Karte aufs Fundament (Klondike/FreeCell);
Leertaste/Enter = Stock, U = Undo (unbegrenzt), R = neues Blatt, S = Setup.

Alle Kartengrafiken kommen aus games/cards.py (reine pygame-Primitiven).
"""

import random
import time as _time

import pygame

import settings as settings_mod
import ui
from game_base import Game, InputEvent
from i18n import t

from . import cards as C

# Tisch-Identität (Filz). Generische UI-Farben kommen zur Zeichenzeit aus
# ui.* (Theme), die Akzentfarbe aus self.accent (= Sidebar-Farbe #2fa77c).
COL_FELT = (20, 44, 35)          # Filz-Hintergrund
COL_FELT_EDGE = (13, 31, 24)

SETUP, PLAY = "setup", "play"

DRAG_PX = 6          # ab dieser Bewegung ist es ein "echter" Drag
CLICK_S = 0.35       # kürzer = Klick (für Klick-Klick-Bedienung)


class Pile:
    """Ein Kartenstapel mit Position und Auffächerungs-Abständen."""

    __slots__ = ("kind", "cards", "x", "y", "dy_down", "dy_up", "meta")

    def __init__(self, kind, x=0, y=0, dy_down=0, dy_up=0, **meta):
        self.kind = kind          # stock/waste/foundation/tableau/cell/grid/removed
        self.cards = []
        self.x = x
        self.y = y
        self.dy_down = dy_down
        self.dy_up = dy_up
        self.meta = meta

    def rects(self, w, h):
        return C.fan_rects(self.x, self.y, self.cards, w, h,
                           self.dy_down, self.dy_up)

    @property
    def top(self):
        return self.cards[-1] if self.cards else None


# ---------------------------------------------------------------------------
#  Varianten
# ---------------------------------------------------------------------------

class Variant:
    """Basis: Regeln einer Solitär-Variante. self.g = SolitaireGame."""

    def __init__(self, game):
        self.g = game

    # Pflicht-Hooks
    def deal(self, rng): ...
    def layout(self, w, h): ...
    def is_won(self):
        return False

    # Interaktion (Standard: nichts erlaubt)
    def can_grab(self, pi, ci):
        return False

    def try_drop(self, cards_, src_i, dst_i):
        """Versucht abzulegen; bei Erfolg Record zurückgeben, sonst None.
        Die Karten sind bereits aus der Quelle entnommen."""
        return None

    def on_stock(self):
        return None

    def can_recycle(self):
        """True, wenn ein Klick auf den leeren Stock die Waste zurücklegt."""
        return False

    def on_right(self, pi):
        return None

    def on_click_card(self, pi, ci):
        return None

    def undo_extra(self, extra):
        pass

    def setup_rows(self):
        return []          # [(label_callable, toggle_callable)]

    def hud_extra(self):
        return []

    # ----- gemeinsame Helfer ------------------------------------------------

    def _flip_top(self, pi, record, score=0):
        """Deckt die oberste verdeckte Karte auf (und protokolliert es)."""
        p = self.g.piles[pi]
        if p.cards and not p.top.face_up:
            p.top.face_up = True
            record["flips"].append((pi, len(p.cards) - 1))
            record["score"] += score

    def _foundation_ok(self, card, pile):
        if pile.cards:
            return (card.suit == pile.top.suit
                    and card.rank == pile.top.rank + 1)
        return card.rank == 1

    def _alt_desc_ok(self, card, pile):
        if pile.cards:
            top = pile.top
            return top.face_up and card.red != top.red \
                and card.rank == top.rank - 1
        return True


class Klondike(Variant):
    key = "klondike"
    # Stapel-Indizes: 0 Stock, 1 Waste, 2-5 Fundament, 6-12 Tableau

    def deal(self, rng):
        g = self.g
        g.piles = [Pile("stock"), Pile("waste")]
        g.piles += [Pile("foundation") for _ in range(4)]
        g.piles += [Pile("tableau") for _ in range(7)]
        deck = C.make_deck()
        C.shuffle(deck, rng)
        for i in range(7):
            for j in range(i + 1):
                card = deck.pop()
                card.face_up = (j == i)
                g.piles[6 + i].cards.append(card)
        g.piles[0].cards = deck                    # Rest verdeckt in den Stock
        self.draw3 = bool(g.settings_get("draw3", False))
        self.recycles = 0

    def layout(self, w, h):
        g = self.g
        g.cw = max(30, w // 9)
        g.ch = int(g.cw * 1.4)
        m = max(6, w // 80)
        top = g.hud_h + m
        g.piles[0].x, g.piles[0].y = m, top
        g.piles[1].x, g.piles[1].y = m + g.cw + m, top
        for i in range(4):
            g.piles[2 + i].x = w - m - (4 - i) * (g.cw + m) + m
            g.piles[2 + i].y = top
        ty = top + g.ch + 2 * m
        span = 7 * g.cw + 6 * m
        x0 = (w - span) // 2
        for i in range(7):
            p = g.piles[6 + i]
            p.x, p.y = x0 + i * (g.cw + m), ty
            p.dy_down, p.dy_up = int(g.ch * 0.12), int(g.ch * 0.28)

    def can_grab(self, pi, ci):
        p = self.g.piles[pi]
        if p.kind == "waste":
            return ci == len(p.cards) - 1
        if p.kind == "foundation":
            return ci == len(p.cards) - 1
        if p.kind == "tableau":
            return p.cards[ci].face_up
        return False

    def try_drop(self, cards_, src_i, dst_i):
        g = self.g
        dst = g.piles[dst_i]
        src = g.piles[src_i]
        if dst.kind == "foundation":
            if len(cards_) == 1 and self._foundation_ok(cards_[0], dst):
                rec = g.commit(cards_, src_i, dst_i, score=10)
                self._flip_top(src_i, rec, score=5)
                return rec
        elif dst.kind == "tableau":
            first = cards_[0]
            ok = (first.rank == 13) if not dst.cards \
                else self._alt_desc_ok(first, dst)
            if ok:
                delta = -10 if src.kind == "foundation" else 0
                rec = g.commit(cards_, src_i, dst_i, score=delta)
                self._flip_top(src_i, rec, score=5)
                return rec
        return None

    def on_stock(self):
        g = self.g
        stock, waste = g.piles[0], g.piles[1]
        if stock.cards:
            n = min(3 if self.draw3 else 1, len(stock.cards))
            rec = g.new_record()
            for _ in range(n):
                card = stock.cards.pop()
                card.face_up = True
                waste.cards.append(card)
            rec["ops"].append((0, 1, n))
            rec["flips"] = [(1, len(waste.cards) - 1 - i) for i in range(n)]
            return rec
        if waste.cards:
            # Recycle: Waste komplett zurück in den Stock (verdeckt).
            n = len(waste.cards)
            for card in reversed(waste.cards):
                card.face_up = False
                stock.cards.append(card)
            waste.cards.clear()
            self.recycles += 1
            rec = g.new_record()
            rec["score"] = -20
            rec["extra"] = {"recycle": n}
            return rec
        return None

    def can_recycle(self):
        return bool(self.g.piles[1].cards) and not self.g.piles[0].cards

    def undo_extra(self, extra):
        if "recycle" in extra:
            g = self.g
            stock, waste = g.piles[0], g.piles[1]
            for card in reversed(stock.cards[-extra["recycle"]:]):
                card.face_up = True
                waste.cards.append(card)
            del stock.cards[-extra["recycle"]:]
            self.recycles -= 1

    def on_right(self, pi):
        g = self.g
        p = g.piles[pi]
        if p.kind not in ("waste", "tableau") or not p.cards or not p.top.face_up:
            return None
        card = p.top
        for fi in range(2, 6):
            if self._foundation_ok(card, g.piles[fi]):
                p.cards.pop()
                return self.try_drop([card], pi, fi)
        return None

    def is_won(self):
        return all(len(self.g.piles[i].cards) == 13 for i in range(2, 6))

    def win_bonus(self):
        return max(0, 1000 - 2 * int(self.g.elapsed))

    def setup_rows(self):
        return [(lambda: t("sol.draw3") + ":  "
                 + (t("common.on") if self.g.settings_get("draw3", False)
                    else t("common.off")),
                 lambda: self.g.settings_toggle("draw3"))]

    def hud_extra(self):
        return []


class Spider(Variant):
    key = "spider"
    # 0 Stock, 1-10 Tableau, 11-18 Fundament-Slots

    def deal(self, rng):
        g = self.g
        g.piles = [Pile("stock")]
        g.piles += [Pile("tableau") for _ in range(10)]
        g.piles += [Pile("foundation") for _ in range(8)]
        suits_n = int(g.settings_get("spider_suits", 1))
        if suits_n == 1:
            deck = C.make_deck((0,), 8)
        elif suits_n == 2:
            deck = C.make_deck((0, 1), 4)
        else:
            deck = C.make_deck(copies=2)
        C.shuffle(deck, rng)
        for i in range(10):
            n = 6 if i < 4 else 5
            for j in range(n):
                card = deck.pop()
                card.face_up = (j == n - 1)
                g.piles[1 + i].cards.append(card)
        g.piles[0].cards = deck        # 50 Karten = 5 Nachschübe
        self.sequences = 0
        self.moves = 0

    def layout(self, w, h):
        g = self.g
        m = max(4, w // 120)
        g.cw = max(26, (w - 11 * m) // 10)
        g.ch = int(g.cw * 1.4)
        top = g.hud_h + m
        g.piles[0].x, g.piles[0].y = m, top
        for i in range(8):
            g.piles[11 + i].x = w - m - (8 - i) * (g.cw // 2 + m) + m
            g.piles[11 + i].y = top
        ty = top + g.ch + 2 * m
        for i in range(10):
            p = g.piles[1 + i]
            p.x, p.y = m + i * (g.cw + m), ty
            p.dy_down, p.dy_up = int(g.ch * 0.12), int(g.ch * 0.24)

    def can_grab(self, pi, ci):
        p = self.g.piles[pi]
        if p.kind != "tableau":
            return False
        run = p.cards[ci:]
        if not run or not run[0].face_up:
            return False
        for a, b in zip(run, run[1:]):
            if not (b.face_up and b.suit == a.suit and b.rank == a.rank - 1):
                return False
        return True

    def try_drop(self, cards_, src_i, dst_i):
        g = self.g
        dst = g.piles[dst_i]
        if dst.kind != "tableau":
            return None
        if dst.cards and not (dst.top.face_up
                              and cards_[0].rank == dst.top.rank - 1):
            return None
        self.moves += 1
        rec = g.commit(cards_, src_i, dst_i, score=-1)
        rec["extra"]["move"] = True
        self._flip_top(src_i, rec, score=0)
        self._check_sequence(dst_i, rec)
        return rec

    def _check_sequence(self, pi, rec):
        """Vollständige K->A-Kette gleicher Farbe am Ende -> ins Fundament."""
        g = self.g
        p = g.piles[pi]
        if len(p.cards) < 13:
            return
        run = p.cards[-13:]
        if run[0].rank != 13 or not all(c.face_up for c in run):
            return
        for a, b in zip(run, run[1:]):
            if b.suit != a.suit or b.rank != a.rank - 1:
                return
        target = next(i for i in range(11, 19) if not g.piles[i].cards)
        moved = p.cards[-13:]
        del p.cards[-13:]
        g.piles[target].cards.extend(moved)
        rec["ops"].append((pi, target, 13))
        rec["score"] += 100
        self.sequences += 1
        rec.setdefault("extra", {})["sequence"] = True
        self._flip_top(pi, rec)
        g.play_sound("line")

    def undo_extra(self, extra):
        if extra.get("sequence"):
            self.sequences -= 1
        if extra.get("deal"):
            pass                          # ops decken das Zurücklegen ab
        if extra.get("move"):
            self.moves -= 1

    def on_stock(self):
        g = self.g
        stock = g.piles[0]
        if not stock.cards:
            return None
        if any(not g.piles[1 + i].cards for i in range(10)):
            g.flash(t("sol.no_empty"))
            return None
        rec = g.new_record()
        for i in range(10):
            card = stock.cards.pop()
            card.face_up = True
            g.piles[1 + i].cards.append(card)
            rec["ops"].append((0, 1 + i, 1))
            rec["flips"].append((1 + i, len(g.piles[1 + i].cards) - 1))
        rec.setdefault("extra", {})["deal"] = True
        for i in range(10):
            self._check_sequence(1 + i, rec)
        return rec

    def is_won(self):
        return self.sequences >= 8

    def win_bonus(self):
        return 0

    def score_now(self):
        return max(0, 500 - self.moves + 100 * self.sequences)

    def setup_rows(self):
        return [(lambda: t("sol.suits", n=self.g.settings_get("spider_suits", 1)),
                 lambda: self.g.settings_cycle("spider_suits", (1, 2, 4)))]

    def hud_extra(self):
        return [t("sol.deals_left", n=len(self.g.piles[0].cards) // 10)]


class FreeCell(Variant):
    key = "freecell"
    # 0-3 Zellen, 4-7 Fundament, 8-15 Tableau

    def deal(self, rng):
        g = self.g
        g.piles = [Pile("cell") for _ in range(4)]
        g.piles += [Pile("foundation") for _ in range(4)]
        g.piles += [Pile("tableau") for _ in range(8)]
        deck = C.make_deck()
        C.shuffle(deck, rng)
        for card in deck:
            card.face_up = True
        for i, card in enumerate(deck):
            g.piles[8 + i % 8].cards.append(card)

    def layout(self, w, h):
        g = self.g
        m = max(5, w // 100)
        g.cw = max(28, (w - 9 * m) // 8 - 2)
        g.ch = int(g.cw * 1.4)
        top = g.hud_h + m
        for i in range(4):
            g.piles[i].x, g.piles[i].y = m + i * (g.cw + m), top
        for i in range(4):
            g.piles[4 + i].x = w - m - (4 - i) * (g.cw + m) + m
            g.piles[4 + i].y = top
        ty = top + g.ch + 2 * m
        span = 8 * g.cw + 7 * m
        x0 = (w - span) // 2
        for i in range(8):
            p = g.piles[8 + i]
            p.x, p.y = x0 + i * (g.cw + m), ty
            p.dy_down, p.dy_up = int(g.ch * 0.24), int(g.ch * 0.24)

    def _limit(self, target_empty):
        free = sum(1 for i in range(4) if not self.g.piles[i].cards)
        empty = sum(1 for i in range(8, 16) if not self.g.piles[i].cards)
        if target_empty and empty > 0:
            empty -= 1
        return (free + 1) * (2 ** empty)

    def can_grab(self, pi, ci):
        p = self.g.piles[pi]
        if p.kind == "cell":
            return bool(p.cards)
        if p.kind == "foundation":
            return False
        run = p.cards[ci:]
        for a, b in zip(run, run[1:]):
            if not (b.red != a.red and b.rank == a.rank - 1):
                return False
        return len(run) <= self._limit(target_empty=False)

    def try_drop(self, cards_, src_i, dst_i):
        g = self.g
        dst = g.piles[dst_i]
        if dst.kind == "cell":
            if len(cards_) == 1 and not dst.cards:
                return g.commit(cards_, src_i, dst_i)
        elif dst.kind == "foundation":
            if len(cards_) == 1 and self._foundation_ok(cards_[0], dst):
                return g.commit(cards_, src_i, dst_i, score=10)
        elif dst.kind == "tableau":
            if len(cards_) > self._limit(target_empty=not dst.cards):
                return None
            if not dst.cards or self._alt_desc_ok(cards_[0], dst):
                return g.commit(cards_, src_i, dst_i)
        return None

    def on_right(self, pi):
        g = self.g
        p = g.piles[pi]
        if p.kind not in ("tableau", "cell") or not p.cards:
            return None
        card = p.top
        for fi in range(4, 8):
            if self._foundation_ok(card, g.piles[fi]):
                p.cards.pop()
                return self.try_drop([card], pi, fi)
        return None

    def is_won(self):
        return all(len(self.g.piles[i].cards) == 13 for i in range(4, 8))

    def win_bonus(self):
        return max(0, 500 - int(self.g.elapsed))


class Pyramid(Variant):
    key = "pyramid"
    # 0 Stock, 1 Waste, 2..29 Pyramide (Einzelkarten), 30 "removed"

    def deal(self, rng):
        g = self.g
        g.piles = [Pile("stock"), Pile("waste")]
        idx = 2
        self.children = {}
        for row in range(7):
            for col in range(row + 1):
                g.piles.append(Pile("grid", row=row, col=col))
                idx += 1
        g.piles.append(Pile("removed"))
        # Abdeck-Beziehungen: Karte (r, c) wird von (r+1, c) und (r+1, c+1) verdeckt.
        def pidx(r, c):
            return 2 + r * (r + 1) // 2 + c
        for row in range(6):
            for col in range(row + 1):
                self.children[pidx(row, col)] = (pidx(row + 1, col),
                                                 pidx(row + 1, col + 1))
        deck = C.make_deck()
        C.shuffle(deck, rng)
        for i in range(28):
            card = deck.pop()
            card.face_up = True
            g.piles[2 + i].cards.append(card)
        g.piles[0].cards = deck
        self.redeals = 2

    def layout(self, w, h):
        g = self.g
        g.cw = max(26, int(w / 9.5))
        g.ch = int(g.cw * 1.4)
        top = g.hud_h + 6
        for i in range(28):
            p = g.piles[2 + i]
            row, col = p.meta["row"], p.meta["col"]
            p.x = w // 2 + int((col - row / 2 - 0.5) * g.cw * 1.08) + g.cw // 12
            p.y = top + int(row * g.ch * 0.45)
        m = max(6, w // 60)
        g.piles[0].x = m
        g.piles[0].y = h - g.ch - m
        g.piles[1].x = m + g.cw + m
        g.piles[1].y = h - g.ch - m
        g.piles[30].x, g.piles[30].y = -1000, -1000    # unsichtbar

    def exposed(self, pi):
        p = self.g.piles[pi]
        if p.kind != "grid" or not p.cards:
            return False
        kids = self.children.get(pi)
        if not kids:
            return True
        return all(not self.g.piles[k].cards for k in kids)

    def on_click_card(self, pi, ci):
        g = self.g
        p = g.piles[pi]
        clickable = (p.kind == "grid" and self.exposed(pi)) or \
                    (p.kind == "waste" and p.cards and ci == len(p.cards) - 1)
        if not clickable:
            return None
        card = p.top
        if card.rank == 13:                     # König: allein entfernen
            p.cards.pop()
            g.piles[30].cards.append(card)
            rec = g.new_record()
            rec["ops"].append((pi, 30, 1))
            rec["score"] += 5
            g.play_sound("point")
            return rec
        if g.sel is None:
            g.sel = (pi, len(p.cards) - 1)
            g.play_sound("select")
            return "selected"
        spi, _sci = g.sel
        g.sel = None
        if spi == pi:
            return None
        sp = g.piles[spi]
        if not sp.cards:
            return None
        other = sp.top
        if other.rank + card.rank == 13:
            rec = g.new_record()
            for src in (spi, pi):
                cc = g.piles[src].cards.pop()
                g.piles[30].cards.append(cc)
                rec["ops"].append((src, 30, 1))
                rec["score"] += 5
            g.play_sound("point")
            return rec
        return None

    def on_stock(self):
        g = self.g
        stock, waste = g.piles[0], g.piles[1]
        g.sel = None
        if stock.cards:
            card = stock.cards.pop()
            card.face_up = True
            waste.cards.append(card)
            rec = g.new_record()
            rec["ops"].append((0, 1, 1))
            rec["flips"].append((1, len(waste.cards) - 1))
            return rec
        if waste.cards and self.redeals > 0:
            n = len(waste.cards)
            for card in reversed(waste.cards):
                card.face_up = False
                stock.cards.append(card)
            waste.cards.clear()
            self.redeals -= 1
            rec = g.new_record()
            rec["extra"] = {"recycle": n}
            return rec
        return None

    def can_recycle(self):
        return (bool(self.g.piles[1].cards) and not self.g.piles[0].cards
                and self.redeals > 0)

    def undo_extra(self, extra):
        if "recycle" in extra:
            g = self.g
            stock, waste = g.piles[0], g.piles[1]
            for card in reversed(stock.cards[-extra["recycle"]:]):
                card.face_up = True
                waste.cards.append(card)
            del stock.cards[-extra["recycle"]:]
            self.redeals += 1

    def is_won(self):
        return all(not self.g.piles[2 + i].cards for i in range(28))

    def win_bonus(self):
        return 500

    def hud_extra(self):
        return [t("sol.redeals_left", n=self.redeals)]


class TriPeaks(Variant):
    key = "tripeaks"
    # 0 Stock, 1 Waste, 2..29 Feld (28 Einzelkarten), 30 "removed" (ungenutzt)

    # (Reihe, Spalten-Offset in halben Kartenbreiten) für 28 Karten
    LAYOUT = ([(0, c) for c in (1.5, 4.5, 7.5)] +
              [(1, c) for c in (1.0, 2.0, 4.0, 5.0, 7.0, 8.0)] +
              [(2, c + 0.5) for c in range(9)] +
              [(3, float(c)) for c in range(10)])

    def deal(self, rng):
        g = self.g
        g.piles = [Pile("stock"), Pile("waste")]
        for i, (row, off) in enumerate(self.LAYOUT):
            g.piles.append(Pile("grid", row=row, off=off))
        deck = C.make_deck()
        C.shuffle(deck, rng)
        for i in range(28):
            card = deck.pop()
            card.face_up = (self.LAYOUT[i][0] == 3)
            g.piles[2 + i].cards.append(card)
        first = deck.pop()
        first.face_up = True
        g.piles[1].cards.append(first)
        g.piles[0].cards = deck
        self.combo = 0

    def layout(self, w, h):
        g = self.g
        g.cw = max(26, int(w / 11.5))
        g.ch = int(g.cw * 1.4)
        top = g.hud_h + 6
        span = 10 * g.cw
        x0 = (w - span) // 2
        for i, (row, off) in enumerate(self.LAYOUT):
            p = g.piles[2 + i]
            p.x = x0 + int(off * g.cw)
            p.y = top + int(row * g.ch * 0.5)
        m = max(6, w // 60)
        g.piles[0].x = w // 2 - g.cw - m
        g.piles[0].y = h - g.ch - m
        g.piles[1].x = w // 2 + m
        g.piles[1].y = h - g.ch - m

    def _covered_by(self, i):
        """Indizes der Feld-Karten, die Karte i (Layout-Index) verdecken."""
        row, off = self.LAYOUT[i]
        result = []
        for j, (r2, o2) in enumerate(self.LAYOUT):
            if r2 == row + 1 and abs(o2 - off) <= 0.51:
                result.append(j)
        return result

    def uncovered(self, i):
        return all(not self.g.piles[2 + j].cards for j in self._covered_by(i))

    def _reveal(self, rec):
        """Frei gewordene Karten aufdecken (protokolliert)."""
        for i in range(28):
            p = self.g.piles[2 + i]
            if p.cards and not p.top.face_up and self.uncovered(i):
                p.top.face_up = True
                rec["flips"].append((2 + i, len(p.cards) - 1))

    def on_click_card(self, pi, ci):
        g = self.g
        p = g.piles[pi]
        if p.kind != "grid" or not p.cards or not p.top.face_up:
            return None
        i = pi - 2
        if not self.uncovered(i):
            return None
        waste_top = g.piles[1].top
        d = abs(p.top.rank - waste_top.rank)
        if d not in (1, 12):                    # +/-1, A<->K wrappt (13-1=12)
            return None
        card = p.cards.pop()
        g.piles[1].cards.append(card)
        self.combo += 1
        rec = g.new_record()
        rec["ops"].append((pi, 1, 1))
        rec["score"] += 10 * self.combo
        rec.setdefault("extra", {})["combo_was"] = self.combo - 1
        self._reveal(rec)
        g.play_sound("point")
        return rec

    def on_stock(self):
        g = self.g
        stock = g.piles[0]
        if not stock.cards:
            return None
        card = stock.cards.pop()
        card.face_up = True
        g.piles[1].cards.append(card)
        rec = g.new_record()
        rec["ops"].append((0, 1, 1))
        rec["flips"].append((1, len(g.piles[1].cards) - 1))
        rec.setdefault("extra", {})["combo_was"] = self.combo
        self.combo = 0
        return rec

    def undo_extra(self, extra):
        if "combo_was" in extra:
            self.combo = extra["combo_was"]

    def is_won(self):
        return all(not self.g.piles[2 + i].cards for i in range(28))

    def win_bonus(self):
        return 500

    def hud_extra(self):
        out = [t("sol.deals_left", n=len(self.g.piles[0].cards))]
        if self.combo > 1:
            out.append(t("sol.combo", n=self.combo))
        return out


VARIANTS = {"klondike": Klondike, "spider": Spider, "freecell": FreeCell,
            "pyramid": Pyramid, "tripeaks": TriPeaks}


# ---------------------------------------------------------------------------
#  Das Spiel (Hülle um die Varianten)
# ---------------------------------------------------------------------------

class SolitaireGame(Game):
    name = "Solitaire"
    highscore_key = "solitaire"
    supports_multiplayer = False
    wants_right_click = True

    MODES = [("klondike", "sol.mode.klondike"), ("spider", "sol.mode.spider"),
             ("freecell", "sol.mode.freecell"), ("pyramid", "sol.mode.pyramid"),
             ("tripeaks", "sol.mode.tripeaks")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        vkey = self.mode if self.mode in VARIANTS else "klondike"
        self.variant = VARIANTS[vkey](self)

        self._make_fonts()
        self.renderer = C.CardRenderer(self.accent)
        self.hud_h = max(34, int(self.height * 0.075))
        self._rebuild_static()

        self.piles = []
        self.sel = None            # Klick-Klick-Auswahl: (pile_i, card_i)
        self.drag = None
        self.undo_stack = []
        self.elapsed = 0.0
        self.moves = 0
        self.won = False
        self.msg = None
        self.msg_t = 0.0
        self._build_setup_layout()
        self.state = SETUP

    def settings_get(self, key, default):
        sol = self.settings.get("solitaire", {}) \
            if isinstance(self.settings, dict) else {}
        return sol.get(key, default)

    def settings_toggle(self, key):
        val = not self.settings_get(key, False)
        self._save_setting(key, val)

    def settings_cycle(self, key, values):
        cur = self.settings_get(key, values[0])
        idx = values.index(cur) if cur in values else 0
        self._save_setting(key, values[(idx + 1) % len(values)])

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("solitaire", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _make_fonts(self):
        """Theme-Schriften; _mono fuer die HUD-Mitte (Zeit/Zahlen ruhig)."""
        self._small = ui.font(16)
        self._tiny = ui.font(13)
        self._mono = ui.font(15, mono=True)
        self._huge = ui.font(max(26, self.height // 11), bold=True)

    def _rebuild_static(self):
        """Groessenabhaengige, gecachte Flaechen (Filz, HUD, Overlay)."""
        self._felt = C.make_felt(self.width, self.height,
                                 COL_FELT, COL_FELT_EDGE)
        self._hud_bg = pygame.Surface((self.width, self.hud_h),
                                      pygame.SRCALPHA)
        self._hud_bg.fill((8, 22, 16, 224))
        self._overlay = pygame.Surface((self.width, self.height),
                                       pygame.SRCALPHA)
        self._overlay.fill((6, 14, 10, 190))

    def on_surface_changed(self):
        self._make_fonts()
        self.hud_h = max(34, int(self.height * 0.075))
        self.renderer.clear()
        self._rebuild_static()
        self._build_setup_layout()
        if self.state == PLAY:
            self.variant.layout(self.width, self.height)

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(380, self.width - 60)
        y0 = int(self.height * 0.36)
        rows = self.variant.setup_rows()
        self.option_rects = [pygame.Rect(cx - bw // 2, y0 + i * 52, bw, 42)
                             for i in range(len(rows))]
        self.start_rect = pygame.Rect(cx - 95, y0 + len(rows) * 52 + 16, 190, 46)

    def _handle_setup(self, event):
        rows = self.variant.setup_rows()
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Return", "space"):
                self._new_deal()
            elif event.key in ("Left", "Right", "a", "d", "A", "D") and rows:
                rows[0][1]()
                self.play_sound("select")
                self._build_setup_layout()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, r in enumerate(self.option_rects):
                if r.collidepoint(event.pos) and i < len(rows):
                    rows[i][1]()
                    self.play_sound("select")
                    return
            if self.start_rect.collidepoint(event.pos):
                self._new_deal()

    # ===================================================== Neues Blatt
    def _new_deal(self):
        self.game_over = False
        self.won = False
        self.score = 500 if isinstance(self.variant, Spider) else 0
        self.sel = None
        self.drag = None
        self.undo_stack = []
        self.elapsed = 0.0
        self.moves = 0
        self.variant.deal(random.Random())
        self.variant.layout(self.width, self.height)
        self.state = PLAY
        self.play_sound("click")

    # ===================================================== Records / Undo
    def new_record(self):
        return {"ops": [], "flips": [], "score": 0, "extra": {}}

    def commit(self, cards_, src_i, dst_i, score=0):
        """Bereits entnommene Karten auf dst legen und protokollieren."""
        self.piles[dst_i].cards.extend(cards_)
        rec = self.new_record()
        rec["ops"].append((src_i, dst_i, len(cards_)))
        rec["score"] = score
        return rec

    def _apply_record(self, rec):
        if rec is None or rec == "selected":
            return False
        self.score += rec["score"]
        self.moves += 1
        self.undo_stack.append(rec)
        self._check_win()
        return True

    def _undo(self):
        if not self.undo_stack or self.game_over:
            return
        # Laufendes Drag zuerst zurücklegen, sonst zieht das Undo Karten
        # unter der Maus weg und der Stapelzustand geht kaputt.
        self._cancel_drag()
        rec = self.undo_stack.pop()
        # WICHTIG: Flips VOR den Ops rückgängig machen - die Indizes beziehen
        # sich auf den Zustand nach dem Zug. (Vorher blieben z.B. nach dem
        # Undo eines Stock-Zugs die zurückgelegten Karten offen im Stock.)
        for pi, ci in rec["flips"]:
            if ci < len(self.piles[pi].cards):
                card = self.piles[pi].cards[ci]
                card.face_up = not card.face_up
        for src_i, dst_i, n in reversed(rec["ops"]):
            moved = self.piles[dst_i].cards[-n:]
            del self.piles[dst_i].cards[-n:]
            self.piles[src_i].cards.extend(moved)
        self.score -= rec["score"]
        self.moves = max(0, self.moves - 1)
        self.variant.undo_extra(rec.get("extra", {}))
        self.sel = None
        self.play_sound("rotate")

    def _check_win(self):
        if self.variant.is_won():
            self.won = True
            self.win_bonus = self.variant.win_bonus()
            self.score = max(0, self.score + self.win_bonus)
            self.game_over = True
            self.report_result(True)
            self.ach_event("solitaire_win")
            self.play_sound("win")
            self.rumble(220)

    def flash(self, text):
        self.msg = text
        self.msg_t = 1.6

    # ===================================================== Treffer-Logik
    def _hit(self, pos):
        """(pile_i, card_i) unter der Position - oberste Karte zuerst."""
        best = None
        for pi, p in enumerate(self.piles):
            if p.kind == "removed":
                continue
            rects = p.rects(self.cw, self.ch)
            ci = C.hit_index(rects, pos)
            if ci is not None:
                best = (pi, ci)      # spätere Piles liegen visuell nicht höher,
        return best                  # aber Überlappungen sind selten kritisch

    def _pile_at(self, pos):
        """Ablage-Ziel unter der Position (inkl. Bereich unter dem Fächer).

        Erst exakte Trefferflächen prüfen, dann großzügig erweiterte -
        so gewinnt bei Überlappungen der direkt getroffene Stapel."""
        def region_of(p, inflate):
            rects = p.rects(self.cw, self.ch)
            if rects:
                region = rects[0].unionall(rects[1:]) \
                    if len(rects) > 1 else rects[0].copy()
            else:
                region = pygame.Rect(p.x, p.y, self.cw, self.ch)
            if inflate:
                region.height += self.ch // 2
            return region

        for inflate in (False, True):
            for pi, p in enumerate(self.piles):
                if p.kind == "removed":
                    continue
                if region_of(p, inflate).collidepoint(pos):
                    return pi
        return None

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.game_over:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space", "r", "R"):
                    self._new_deal()
                elif event.key in ("s", "S"):
                    self.state = SETUP
                    self.play_sound("click")
            return

        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("u", "U"):
                self._undo()
            elif k in ("r", "R"):
                self._new_deal()
            elif k in ("s", "S"):
                self._cancel_drag()
                self.state = SETUP
                self.play_sound("click")
            elif k in ("space", "Return"):
                self._apply_record(self.variant.on_stock())
        elif event.kind == InputEvent.MOUSEDOWN:
            if event.button == 3:
                hit = self._hit(event.pos)
                pi = hit[0] if hit is not None else self._pile_at(event.pos)
                if pi is not None:
                    self._apply_record(self.variant.on_right(pi))
                return
            self._on_down(event.pos)
        elif event.kind == InputEvent.MOUSEMOVE:
            if self.drag is not None:
                self.drag["pos"] = event.pos
                dx = event.pos[0] - self.drag["start"][0]
                dy = event.pos[1] - self.drag["start"][1]
                if dx * dx + dy * dy > DRAG_PX * DRAG_PX:
                    self.drag["moved"] = True
        elif event.kind == InputEvent.MOUSEUP:
            self._on_up(event.pos)

    def _on_down(self, pos):
        # Verwaistes Drag (verlorenes MOUSEUP, z.B. durch Esc-Pause) zuerst
        # als Ablage-Versuch behandeln.
        if self.drag is not None:
            self._on_up(pos)
            return
        hit = self._hit(pos)
        if hit is None:
            # Leerer Stock: Klick auf den kartenlosen Slot löst den Stock-Zug
            # aus - bei Klondike/Pyramid also das Zurücklegen der Waste, damit
            # man den Reststapel erneut durchgehen kann.
            p0 = self.piles[0] if self.piles else None
            if p0 is not None and p0.kind == "stock" and not p0.cards \
                    and pygame.Rect(p0.x, p0.y, self.cw, self.ch) \
                    .collidepoint(pos):
                self.sel = None
                self._apply_record(self.variant.on_stock())
                return
            self.sel = None
            return
        pi, ci = hit
        p = self.piles[pi]
        if p.kind == "stock":
            self.sel = None
            self._apply_record(self.variant.on_stock())
            return
        # Varianten mit eigener Klick-Logik (Pyramid/TriPeaks)
        rec = self.variant.on_click_card(pi, ci)
        if rec is not None:
            if rec != "selected":
                self._apply_record(rec)
            return
        if self.variant.can_grab(pi, ci):
            cards_ = p.cards[ci:]
            del p.cards[ci:]
            self.drag = dict(src=pi, ci=ci, cards=cards_, pos=pos, start=pos,
                             t=_time.time(), moved=False,
                             off=(pos[0] - p.x,
                                  pos[1] - (p.y + ci * (p.dy_up or 1))))
            self.play_sound("select")
        else:
            self.sel = None

    def _on_up(self, pos):
        if self.drag is None:
            return
        d = self.drag
        self.drag = None
        quick = (not d["moved"]) and (_time.time() - d["t"] < CLICK_S)
        if quick:
            # Klick: Karten zurücklegen und Auswahl setzen bzw. ablegen.
            self._return_drag(d)
            if self.sel is not None and self.sel != (d["src"], d["ci"]):
                spi, sci = self.sel
                self.sel = None
                self._try_move(spi, sci, d["src"])
            else:
                self.sel = (d["src"], d["ci"])
            return
        # Echter Drag: Ziel suchen.
        dst = self._pile_at(pos)
        if dst is not None and dst != d["src"]:
            rec = self.variant.try_drop(d["cards"], d["src"], dst)
            if rec is not None:
                self._apply_record(rec)
                self.sel = None
                self.play_sound("move")
                return
        self._return_drag(d)

    def _return_drag(self, d):
        self.piles[d["src"]].cards.extend(d["cards"])

    def _try_move(self, spi, sci, dst):
        """Klick-Klick: Substack von (spi, sci) auf Ziel dst versuchen."""
        p = self.piles[spi]
        if sci >= len(p.cards) or not self.variant.can_grab(spi, sci):
            return
        cards_ = p.cards[sci:]
        del p.cards[sci:]
        rec = self.variant.try_drop(cards_, spi, dst)
        if rec is not None:
            self._apply_record(rec)
            self.play_sound("move")
        else:
            p.cards.extend(cards_)

    def _cancel_drag(self):
        if self.drag is not None:
            self._return_drag(self.drag)
            self.drag = None

    # ===================================================== Update / Zeichnen
    def update(self, dt):
        if self.state != PLAY or self.game_over:
            return
        self.elapsed += dt
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None
        # Spider: Punktestand aus Zählern ableiten (stabil bei Undo).
        if isinstance(self.variant, Spider):
            self.score = self.variant.score_now()

    def _fmt_time(self):
        sec = int(self.elapsed)
        return f"{sec // 60:02d}:{sec % 60:02d}"

    def draw(self):
        if self.state == SETUP:
            self._draw_setup()
            return
        s = self.surface
        s.blit(self._felt, (0, 0))
        self._draw_piles(s)
        if self.drag is not None:
            self._draw_drag(s)
        self._draw_hud(s)
        if self.game_over:
            self._draw_result(s)

    def _draw_piles(self, s):
        cw, ch = self.cw, self.ch
        for pi, p in enumerate(self.piles):
            if p.kind == "removed":
                continue
            if not p.cards:
                if p.kind != "grid":
                    C.draw_slot(s, (p.x, p.y, cw, ch))
                if p.kind == "stock" and self.variant.can_recycle():
                    self._draw_recycle(s, p, cw, ch)
                continue
            if p.kind in ("stock",):
                s.blit(self.renderer.get(p.top, cw, ch)
                       if p.top.face_up else self.renderer.back(cw, ch),
                       (p.x, p.y))
                if len(p.cards) > 1:
                    n = self._tiny.render(str(len(p.cards)), True,
                                          ui.TEXT_DIM)
                    s.blit(n, (p.x + 2, p.y + ch + 2))
            elif p.kind in ("waste", "foundation", "cell", "grid"):
                s.blit(self.renderer.get(p.top, cw, ch), (p.x, p.y))
            else:   # tableau: auffächern
                rects = p.rects(cw, ch)
                for card, r in zip(p.cards, rects):
                    s.blit(self.renderer.get(card, cw, ch), r.topleft)
            # Auswahl-Rahmen
            if self.sel is not None and self.sel[0] == pi:
                rects = p.rects(cw, ch)
                ci = min(self.sel[1], len(rects) - 1)
                if rects:
                    sel_r = rects[ci].unionall(rects[ci:]) \
                        if len(rects) > ci + 1 else rects[ci]
                    pygame.draw.rect(s, ui.GOLD, sel_r.inflate(4, 4), 2,
                                     border_radius=6)

    def _draw_recycle(self, s, p, cw, ch):
        """Kreisförmiger Pfeil (↻) auf dem leeren Stock als Hinweis, dass die
        Waste zum erneuten Durchgehen zurückgelegt werden kann."""
        cx, cy = p.x + cw // 2, p.y + ch // 2
        r = max(8, min(cw, ch) // 4)
        col = self.accent
        # offener Ring (oben rechts eine Lücke für die Pfeilspitze)
        pygame.draw.arc(s, col, (cx - r, cy - r, 2 * r, 2 * r),
                        -0.35, 4.9, max(2, r // 4))
        # Pfeilspitze am oberen Ende des Rings
        tip = (cx + int(r * 0.95), cy - int(r * 0.33))
        pygame.draw.polygon(s, col, [tip,
                                     (tip[0] - r // 2, tip[1] - r // 6),
                                     (tip[0] - r // 6, tip[1] + r // 2)])

    def _draw_drag(self, s):
        d = self.drag
        x = d["pos"][0] - d["off"][0]
        y = d["pos"][1] - d["off"][1]
        p = self.piles[d["src"]]
        dy = p.dy_up or int(self.ch * 0.28)
        for i, card in enumerate(d["cards"]):
            s.blit(self.renderer.get(card, self.cw, self.ch), (x, y + i * dy))

    def _draw_hud(self, s):
        s.blit(self._hud_bg, (0, 0))
        pygame.draw.line(s, self.accent, (0, self.hud_h),
                         (self.width, self.hud_h), 2)
        cy = self.hud_h // 2
        name = self._small.render(t("sol.mode." + self.variant.key), True,
                                  self.accent)
        s.blit(name, name.get_rect(midleft=(12, cy)))
        # Monospace: Zeit/Zahlen aendern sich, ohne dass die Zeile "springt".
        mid = self._mono.render(
            t("common.points", score=max(0, self.score)) + "   ·   "
            + t("sol.moves", n=self.moves) + "   ·   " + self._fmt_time(),
            True, ui.TEXT)
        s.blit(mid, mid.get_rect(center=(self.width // 2, cy)))
        x = self.width - 12
        for txt in self.variant.hud_extra():
            img = self._small.render(txt, True, ui.TEXT_DIM)
            s.blit(img, img.get_rect(midright=(x, cy)))
            x -= img.get_width() + 16
        if self.msg:
            img = self._small.render(self.msg, True, ui.GOLD)
            s.blit(img, img.get_rect(center=(self.width // 2,
                                             self.hud_h + 14)))
        hint = self._tiny.render(t("sol.hint"), True, ui.TEXT_FAINT)
        s.blit(hint, hint.get_rect(midbottom=(self.width // 2,
                                              self.height - 4)))

    def _draw_result(self, s):
        s.blit(self._overlay, (0, 0))
        cx, cy = self.width // 2, self.height // 2
        head = self._huge.render(t("sol.win", bonus=self.win_bonus), True,
                                 ui.GOLD)
        pw = min(self.width - 40, max(380, head.get_width() + 70))
        panel = pygame.Rect(cx - pw // 2, cy - 96, pw, 192)
        ui.draw_panel(s, panel, accent_top=self.accent)
        s.blit(head, head.get_rect(center=(cx, cy - 46)))
        sc = self.font.render(t("common.points", score=self.score), True,
                              ui.TEXT)
        s.blit(sc, sc.get_rect(center=(cx, cy + 10)))
        hint = self._small.render(t("sol.retry"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, cy + 52)))

    # ----- Setup zeichnen -----------------------------------------------
    def _draw_setup(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height, stars=False)
        ui.draw_title(s, self.width, t("sol.mode." + self.variant.key),
                      subtitle=t("sol.subtitle." + self.variant.key),
                      y=int(self.height * 0.14), big=self._huge,
                      accent=self.accent)
        rows = self.variant.setup_rows()
        for i, r in enumerate(self.option_rects):
            if i >= len(rows):
                break
            ui.draw_button(s, r, rows[i][0](), self._small,
                           accent=self.accent)
        ui.draw_button(s, self.start_rect, t("common.start"), self.font,
                       selected=True, accent=self.accent)
        ui.draw_footer(s, self.width, self.height, t("sol.setup_hint"),
                       self._tiny)
