# -*- coding: utf-8 -*-
"""
cards.py
========
Gemeinsames Spielkarten-Toolkit (für Solitär u.a.).

Alles wird mit pygame-Primitiven gezeichnet - keine Bild-Dateien:
- ``Card``            : Rang 1..13 (A..K), Farbe 0..3 (Pik/Herz/Karo/Kreuz),
                        face_up-Flag.
- ``make_deck``       : Standard-52er-Deck oder Spider-Varianten (104 Karten).
- ``CardRenderer``    : rendert Vorder-/Rückseiten in beliebiger Größe und
                        cached sie je (Rang, Farbe, Breite, Höhe, Seite).
                        Die Schrift kommt aus ui.font (Theme-Schrift), die
                        Rückseite trägt den Akzentton des jeweiligen Spiels.
                        Die Farbsymbole (Pips) sind Polygone/Kreise:
                        Herz = 2 Kreise + Dreieck, Karo = Raute,
                        Pik = umgedrehtes Herz + Fuß, Kreuz = 3 Kreise + Fuß.
- ``draw_slot``       : gestrichelte Umrandung für leere Ablagen.
- ``fan_rects``       : Trefferflächen eines aufgefächerten Stapels.
- ``hit_index``       : oberste getroffene Karte eines Stapels.
- ``make_felt``       : einmalig gerenderter Filz-Hintergrund (Verlauf +
                        Tischlicht + Vignette), damit alle Kartenspiele
                        denselben Tisch-Look teilen.
"""

import pygame

import ui

SUIT_SPADE, SUIT_HEART, SUIT_DIAMOND, SUIT_CLUB = 0, 1, 2, 3

RANK_LABELS = {1: "A", 11: "J", 12: "Q", 13: "K"}

# Karten-Identität: leicht entsättigt, damit sie zum v4.1-Look passt.
COL_RED = (198, 58, 62)
COL_BLACK = (36, 40, 52)
COL_FACE = (240, 242, 247)
COL_FACE_EDGE = (150, 158, 178)


def rank_label(rank):
    return RANK_LABELS.get(rank, str(rank))


def is_red(suit):
    return suit in (SUIT_HEART, SUIT_DIAMOND)


class Card:
    __slots__ = ("rank", "suit", "face_up")

    def __init__(self, rank, suit, face_up=False):
        self.rank = rank
        self.suit = suit
        self.face_up = face_up

    @property
    def red(self):
        return is_red(self.suit)

    def __repr__(self):
        return f"{rank_label(self.rank)}{'shdc'[self.suit]}"


def make_deck(suits=(0, 1, 2, 3), copies=1):
    """Deck bauen: Standard = 52 Karten. Spider: 1 Farbe x8, 2 Farben x4,
    4 Farben x2 - jeweils 104 Karten."""
    deck = []
    for _ in range(copies):
        for suit in suits:
            for rank in range(1, 14):
                deck.append(Card(rank, suit))
    return deck


def shuffle(deck, rng=None):
    import random as _random
    (rng or _random).shuffle(deck)


# ---------------------------------------------------------------------------
#  Zeichnen
# ---------------------------------------------------------------------------

def _draw_suit(surf, suit, cx, cy, r, col):
    """Zeichnet ein Farbsymbol mit Radius r zentriert auf (cx, cy)."""
    cx, cy, r = int(cx), int(cy), max(2, int(r))
    if suit == SUIT_HEART:
        rr = max(2, int(r * 0.55))
        pygame.draw.circle(surf, col, (cx - rr + 1, cy - rr // 2), rr)
        pygame.draw.circle(surf, col, (cx + rr - 1, cy - rr // 2), rr)
        pygame.draw.polygon(surf, col, [(cx - 2 * rr + 1, cy - rr // 6),
                                        (cx + 2 * rr - 1, cy - rr // 6),
                                        (cx, cy + int(rr * 1.6))])
    elif suit == SUIT_DIAMOND:
        pygame.draw.polygon(surf, col, [(cx, cy - r), (cx + int(r * 0.7), cy),
                                        (cx, cy + r), (cx - int(r * 0.7), cy)])
    elif suit == SUIT_SPADE:
        rr = max(2, int(r * 0.55))
        pygame.draw.circle(surf, col, (cx - rr + 1, cy + rr // 2), rr)
        pygame.draw.circle(surf, col, (cx + rr - 1, cy + rr // 2), rr)
        pygame.draw.polygon(surf, col, [(cx - 2 * rr + 1, cy + rr // 6),
                                        (cx + 2 * rr - 1, cy + rr // 6),
                                        (cx, cy - int(rr * 1.6))])
        pygame.draw.polygon(surf, col, [(cx - rr // 2, cy + int(rr * 1.7)),
                                        (cx + rr // 2, cy + int(rr * 1.7)),
                                        (cx, cy + rr // 2)])
    else:   # Kreuz
        rr = max(2, int(r * 0.45))
        pygame.draw.circle(surf, col, (cx, cy - rr), rr)
        pygame.draw.circle(surf, col, (cx - rr, cy + rr // 2), rr)
        pygame.draw.circle(surf, col, (cx + rr, cy + rr // 2), rr)
        pygame.draw.polygon(surf, col, [(cx - rr // 2, cy + int(rr * 2.1)),
                                        (cx + rr // 2, cy + int(rr * 2.1)),
                                        (cx, cy)])


class CardRenderer:
    """Rendert und cached Kartenflächen. accent = Rückseiten-Farbe (RGB)."""

    def __init__(self, accent=(47, 167, 124)):
        self.accent = accent
        self._cache = {}

    def clear(self):
        self._cache.clear()

    def get(self, card, w, h):
        if not card.face_up:
            return self.back(w, h)
        key = (card.rank, card.suit, w, h, True)
        surf = self._cache.get(key)
        if surf is None:
            surf = self._face(card.rank, card.suit, w, h)
            self._cache[key] = surf
        return surf

    def back(self, w, h):
        key = ("back", w, h)
        surf = self._cache.get(key)
        if surf is None:
            surf = pygame.Surface((w, h), pygame.SRCALPHA)
            rad = max(3, w // 8)
            # Akzent leicht abgedunkelt = Grundton der Rückseite
            base = ui.mix(self.accent, (16, 20, 28), 0.30)
            dark = ui.mix(base, (0, 0, 0), 0.35)
            # weißer Kartenrand wie bei echten Rückseiten
            pygame.draw.rect(surf, COL_FACE, (0, 0, w, h), border_radius=rad)
            inner = pygame.Rect(2, 2, w - 4, h - 4)
            irad = max(2, rad - 2)
            pygame.draw.rect(surf, base, inner, border_radius=irad)
            # Rauten-Muster (auf die Innenfläche geclippt)
            pat = inner.inflate(-4, -4)
            step = max(6, w // 5)
            surf.set_clip(pat)
            for yy in range(pat.y, pat.bottom, step):
                for xx in range(pat.x, pat.right, step):
                    pygame.draw.line(surf, dark, (xx, yy + step // 2),
                                     (xx + step // 2, yy), 1)
                    pygame.draw.line(surf, dark, (xx + step // 2, yy),
                                     (xx + step, yy + step // 2), 1)
            surf.set_clip(None)
            pygame.draw.rect(surf, dark, inner, 1, border_radius=irad)
            pygame.draw.rect(surf, COL_FACE_EDGE, (0, 0, w, h), 1,
                             border_radius=rad)
            self._cache[key] = surf
        return surf

    def _face(self, rank, suit, w, h):
        surf = pygame.Surface((w, h), pygame.SRCALPHA)
        rad = max(3, w // 8)
        pygame.draw.rect(surf, COL_FACE, (0, 0, w, h), border_radius=rad)
        pygame.draw.rect(surf, COL_FACE_EDGE, (0, 0, w, h), 1, border_radius=rad)
        col = COL_RED if is_red(suit) else COL_BLACK
        fnt = ui.font(max(10, h // 5), bold=True)
        label = rank_label(rank)
        img = fnt.render(label, True, col)
        # Rang + kleines Symbol oben links
        surf.blit(img, (max(2, w // 12), 2))
        pr = max(2, w // 10)
        _draw_suit(surf, suit, max(2, w // 12) + img.get_width() // 2,
                   4 + img.get_height() + pr, pr, col)
        # Gespiegelt unten rechts
        img2 = pygame.transform.rotate(img, 180)
        surf.blit(img2, (w - img2.get_width() - max(2, w // 12),
                         h - img2.get_height() - 2))
        # Großes Symbol in der Mitte (Bildkarten: großer Buchstabe + Symbol)
        big = max(4, w // 4)
        if rank > 10 or rank == 1:
            bfnt = ui.font(max(14, h // 3), bold=True)
            bimg = bfnt.render(label, True, col)
            surf.blit(bimg, bimg.get_rect(center=(w // 2, h // 2 - big // 2)))
            _draw_suit(surf, suit, w // 2, h // 2 + big, big // 2 + 2, col)
        else:
            _draw_suit(surf, suit, w // 2, h // 2, big, col)
        return surf


def make_felt(w, h, base=(22, 48, 38), edge=(13, 30, 23)):
    """Filz-Hintergrund EINMAL rendern (Software-Rendering: cachen!):
    sanfter vertikaler Verlauf, weiches Tischlicht oben und eine dunkle
    Bande mit Zierlinie am Rand. Rückgabe: opake Surface in (w, h)."""
    surf = pygame.Surface((max(1, w), max(1, h)))
    top = ui.mix(base, (255, 255, 255), 0.10)
    for y in range(h):
        f = y / max(1, h - 1)
        surf.fill(ui.mix(top, edge, f * 0.85), (0, y, w, 1))
    # weiches Oval-Licht in der oberen Tischhälfte
    glow = pygame.Surface((w, h), pygame.SRCALPHA)
    gw, gh = int(w * 0.86), int(h * 0.62)
    pygame.draw.ellipse(glow, (255, 255, 255, 14),
                        (w // 2 - gw // 2, int(h * 0.14), gw, gh))
    pygame.draw.ellipse(glow, (255, 255, 255, 10),
                        (w // 2 - int(gw * 0.42), int(h * 0.20),
                         int(gw * 0.84), int(gh * 0.8)))
    surf.blit(glow, (0, 0))
    # Bande/Vignette am Rand
    pygame.draw.rect(surf, edge, (0, 0, w, h), 6)
    pygame.draw.rect(surf, ui.mix(edge, base, 0.5), (6, 6, w - 12, h - 12), 2)
    pygame.draw.rect(surf, ui.mix(base, (255, 255, 255), 0.06),
                     (8, 8, w - 16, h - 16), 1)
    return surf


def draw_slot(surf, rect, color=(92, 118, 104)):
    """Umrandung für eine leere Ablage (gestrichelter Look)."""
    r = pygame.Rect(rect)
    rad = max(3, r.w // 8)
    step = 7
    pygame.draw.rect(surf, tuple(int(v * 0.5) for v in color), r, 1,
                     border_radius=rad)
    for x in range(r.x + rad, r.right - rad, step * 2):
        pygame.draw.line(surf, color, (x, r.y), (min(x + step, r.right - rad), r.y))
        pygame.draw.line(surf, color, (x, r.bottom - 1),
                         (min(x + step, r.right - rad), r.bottom - 1))
    for y in range(r.y + rad, r.bottom - rad, step * 2):
        pygame.draw.line(surf, color, (r.x, y), (r.x, min(y + step, r.bottom - rad)))
        pygame.draw.line(surf, color, (r.right - 1, y),
                         (r.right - 1, min(y + step, r.bottom - rad)))


def fan_rects(x, y, cards, w, h, dy_down, dy_up):
    """Trefferflächen eines vertikal aufgefächerten Stapels (oben = Ende)."""
    rects = []
    yy = y
    for card in cards:
        rects.append(pygame.Rect(int(x), int(yy), w, h))
        yy += dy_up if card.face_up else dy_down
    return rects


def hit_index(rects, pos):
    """Index der obersten (zuletzt gezeichneten) getroffenen Karte."""
    for i in range(len(rects) - 1, -1, -1):
        if rects[i].collidepoint(pos):
            return i
    return None
