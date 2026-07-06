# -*- coding: utf-8 -*-
"""
ui.py
=====
Gemeinsames UI-Toolkit für alle Pygame-Screens (Menüs, Overlays, Spiele).

Ziel: ein einheitlicher, moderner Look für die ganze Sammlung -
dunkler Farbverlauf mit dezentem Sternenfeld, abgerundete Panels mit
Schattenwurf, Buttons mit Auswahl-Glow und Akzentbalken sowie Titel
mit Unterstreichung. Alles rein in Software gerendert (SDL-dummy),
deshalb werden teure Flächen (Verläufe, Vignette) gecacht.

Verwendung (in draw()):
    import ui
    ui.draw_background(surface, w, h)          # animierter Hintergrund
    ui.draw_title(surface, w, "TITEL", sub)    # Kopfzeile mit Akzentlinie
    ui.draw_button(surface, rect, "Start", font, selected=True)
    ui.draw_panel(surface, rect)               # Karten-Panel
"""

import math
import random

import pygame

# ---------------------------------------------------------------------------
#  Farbpalette (dunkles Navy mit kühlem Blau-Akzent + warmen Signalfarben)
# ---------------------------------------------------------------------------

BG_TOP = (16, 19, 32)        # Hintergrund-Verlauf oben
BG_BOTTOM = (24, 28, 44)     # Hintergrund-Verlauf unten
PANEL = (30, 35, 52)         # Panel-Fläche
PANEL_LIGHT = (38, 44, 64)   # hellere Panel-Variante / Hover
BORDER = (52, 60, 86)        # Panel-/Button-Rand
BORDER_LIGHT = (74, 84, 116)

BTN = (40, 46, 66)           # Button normal
BTN_SEL = (56, 88, 152)      # Button ausgewählt
ACCENT = (88, 156, 255)      # Primär-Akzent (kühles Blau)
ACCENT_SOFT = (66, 110, 180)
GREEN = (110, 205, 140)      # Erfolg / "AN"
GOLD = (245, 205, 100)       # Highscore / Tastenwerte
RED = (225, 95, 95)          # Gefahr / "AUS"

TEXT = (235, 238, 245)       # Haupttext
TEXT_DIM = (150, 158, 178)   # Nebentext
TEXT_FAINT = (95, 102, 124)  # Fußnoten

# Schriftname mit Fallback-Kette (SysFont probiert der Reihe nach durch).
FONT_UI = "bahnschrift,segoeui,consolas"
FONT_MONO = "consolas,menlo,monospace"

_font_cache = {}


def font(size, bold=False, mono=False):
    """Gecachte Schrift; UI-Schrift (Bahnschrift/Segoe) oder Monospace."""
    key = (size, bold, mono)
    f = _font_cache.get(key)
    if f is None:
        f = pygame.font.SysFont(FONT_MONO if mono else FONT_UI, size, bold=bold)
        _font_cache[key] = f
    return f


# ---------------------------------------------------------------------------
#  Hintergrund: Farbverlauf + Sternenfeld + Vignette (gecacht/animiert)
# ---------------------------------------------------------------------------

_bg_cache = {}       # (w, h) -> Surface mit Verlauf + Vignette
_stars = []          # [x, y, tiefe, radius] in 0..1-Koordinaten
_STAR_COUNT = 70


def _gradient(w, h, top, bottom):
    """Vertikaler Farbverlauf als Surface (zeilenweise, einmalig erzeugt)."""
    surf = pygame.Surface((w, h))
    for y in range(h):
        f = y / max(1, h - 1)
        col = (int(top[0] + (bottom[0] - top[0]) * f),
               int(top[1] + (bottom[1] - top[1]) * f),
               int(top[2] + (bottom[2] - top[2]) * f))
        pygame.draw.line(surf, col, (0, y), (w, y))
    return surf


def _base_background(w, h):
    """Verlauf + Vignette, pro Größe nur einmal berechnet."""
    key = (w, h)
    surf = _bg_cache.get(key)
    if surf is None:
        surf = _gradient(w, h, BG_TOP, BG_BOTTOM)
        # Weiche Vignette: klein zeichnen und hochskalieren -> sanfter Verlauf
        # statt harter Ellipsen-Kante (smoothscale interpoliert die Ränder).
        sw, sh = max(8, w // 8), max(8, h // 8)
        shade = pygame.Surface((sw, sh), pygame.SRCALPHA)
        shade.fill((0, 0, 0, 70))
        pygame.draw.ellipse(shade, (0, 0, 0, 0),
                            (-sw // 3, -sh // 3, sw + 2 * sw // 3, sh + 2 * sh // 3))
        surf.blit(pygame.transform.smoothscale(shade, (w, h)), (0, 0))
        # Cache klein halten (Auto-Auflösung erzeugt viele Größen beim Resize).
        if len(_bg_cache) > 6:
            _bg_cache.clear()
        _bg_cache[key] = surf
    return surf


def _ensure_stars():
    if not _stars:
        rnd = random.Random(20240)   # fester Seed -> ruhiges, stabiles Bild
        for _ in range(_STAR_COUNT):
            _stars.append([rnd.random(), rnd.random(),
                           rnd.uniform(0.25, 1.0), rnd.choice((1, 1, 1, 2))])


def draw_background(surface, w, h, stars=True):
    """Zeichnet den Standard-Hintergrund (Verlauf, optional Sternenfeld)."""
    surface.blit(_base_background(w, h), (0, 0))
    if not stars:
        return
    _ensure_stars()
    ticks = pygame.time.get_ticks() / 1000.0
    for x, y, depth, r in _stars:
        # Langsame Drift nach oben + leichtes Funkeln über Sinus.
        yy = (y - ticks * 0.008 * depth) % 1.0
        tw = 0.5 + 0.5 * math.sin(ticks * (0.8 + depth) + x * 40.0)
        c = int(40 + 70 * depth * tw)
        surface.fill((c, c + 6, c + 18),
                     (int(x * w), int(yy * h), r, r))


# ---------------------------------------------------------------------------
#  Panels & Buttons
# ---------------------------------------------------------------------------

def draw_panel(surface, rect, color=PANEL, border=BORDER, radius=12,
               shadow=True):
    """Abgerundetes Panel mit Rand und weichem Schlagschatten."""
    r = pygame.Rect(rect)
    if shadow:
        sh = pygame.Surface((r.w + 12, r.h + 12), pygame.SRCALPHA)
        pygame.draw.rect(sh, (0, 0, 0, 90), (4, 6, r.w, r.h),
                         border_radius=radius + 2)
        surface.blit(sh, (r.x - 2, r.y - 2))
    pygame.draw.rect(surface, color, r, border_radius=radius)
    pygame.draw.rect(surface, border, r, width=1, border_radius=radius)
    return r


def draw_button(surface, rect, label, fnt, selected=False, icon=None,
                sub=None, sub_font=None):
    """Menü-Button: normal flach, ausgewählt mit Glow + Akzentbalken + Pfeil."""
    r = pygame.Rect(rect)
    if selected:
        # Weicher Außen-Glow
        glow = pygame.Surface((r.w + 20, r.h + 20), pygame.SRCALPHA)
        pygame.draw.rect(glow, (*ACCENT, 45), (0, 0, r.w + 20, r.h + 20),
                         border_radius=16)
        surface.blit(glow, (r.x - 10, r.y - 10))
        pygame.draw.rect(surface, BTN_SEL, r, border_radius=10)
        pygame.draw.rect(surface, ACCENT, r, width=2, border_radius=10)
        # Akzentbalken links
        pygame.draw.rect(surface, ACCENT, (r.x + 6, r.y + 8, 4, r.h - 16),
                         border_radius=2)
        # Pfeil-Marker rechts (gezeichnet, kein Sonderzeichen nötig)
        cx, cy = r.right - 18, r.centery
        pygame.draw.polygon(surface, TEXT,
                            [(cx - 4, cy - 6), (cx + 4, cy), (cx - 4, cy + 6)])
    else:
        pygame.draw.rect(surface, BTN, r, border_radius=10)
        pygame.draw.rect(surface, BORDER, r, width=1, border_radius=10)

    text_col = TEXT if selected else TEXT_DIM
    img = fnt.render(label, True, text_col)
    if sub and sub_font:
        sub_img = sub_font.render(sub, True, TEXT_FAINT if not selected else TEXT_DIM)
        total_h = img.get_height() + 2 + sub_img.get_height()
        y0 = r.centery - total_h // 2
        surface.blit(img, img.get_rect(midtop=(r.centerx, y0)))
        surface.blit(sub_img, sub_img.get_rect(
            midtop=(r.centerx, y0 + img.get_height() + 2)))
    else:
        surface.blit(img, img.get_rect(center=r.center))
    return r


def draw_title(surface, width, title, subtitle=None, y=52, big=None, small=None):
    """Zentrierter Titel mit Schatten, Akzent-Unterstrich und Untertitel."""
    big = big or font(40, bold=True)
    # Schatten leicht versetzt, dann Titel
    sh = big.render(title, True, (0, 0, 0))
    sh.set_alpha(140)
    img = big.render(title, True, TEXT)
    cx = width // 2
    surface.blit(sh, sh.get_rect(center=(cx + 2, y + 3)))
    surface.blit(img, img.get_rect(center=(cx, y)))
    # Akzentlinie unter dem Titel (kurz, mittig)
    lw = min(img.get_width() + 20, width - 80)
    ly = y + img.get_height() // 2 + 8
    pygame.draw.rect(surface, ACCENT, (cx - lw // 2, ly, lw, 3), border_radius=2)
    pygame.draw.rect(surface, ACCENT_SOFT,
                     (cx - lw // 6, ly + 5, lw // 3, 2), border_radius=2)
    if subtitle:
        small = small or font(17)
        sub = small.render(subtitle, True, TEXT_DIM)
        surface.blit(sub, sub.get_rect(center=(cx, ly + 26)))
    return ly


def draw_footer(surface, width, height, text, fnt=None):
    """Fußzeile: dezente Trennlinie + Hinweistext unten."""
    fnt = fnt or font(14)
    img = fnt.render(text, True, TEXT_FAINT)
    y = height - 22
    pygame.draw.line(surface, BORDER, (width // 6, y - 12),
                     (width - width // 6, y - 12))
    surface.blit(img, img.get_rect(center=(width // 2, y)))


def pulse(speed=2.0, lo=0.35, hi=1.0):
    """Sinus-Puls 0..1 für sanfte Blink-/Atem-Animationen."""
    ticks = pygame.time.get_ticks() / 1000.0
    return lo + (hi - lo) * (0.5 + 0.5 * math.sin(ticks * speed))
