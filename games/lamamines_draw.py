# -*- coding: utf-8 -*-
"""
lamamines_draw.py
=================
Zeichen-Bausteine von Lama Mines (nur pygame-Primitive, keine Bilddateien):

- ``background``   : Höhlenwand (Verlauf, Gesteinsbrocken, Körnung,
                     leuchtende Kristalladern, Vignette), gecacht je Auflösung,
- ``board``        : Holzrahmen mit Eisenwinkeln und Nieten um die dunkle
                     Grube des Spielfelds, gecacht je Größe,
- ``tile_hidden``  : ungeöffnetes Feld = Steinquader mit Fase und Sprenkeln
                     (4 Varianten, normal / Maus darüber),
- ``tile_open``    : ausgehobenes Feld (Grube mit Innenschatten),
- ``tile_boom``    : Feld der getroffenen Mine (rot glühend, Brandfleck),
- ``dim``          : Abdunklung für das nach der Runde aufgedeckte Feld,
- ``gem``          : geschliffener Edelstein (Brillant, 4 Farben),
- ``mine``         : Seemine mit Hörnern und Signallicht (an/aus),
- ``miner_llama``  : das Maskottchen - Lama aus ``lamacrossing_draw`` mit
                     Grubenhelm, Stirnlampe und Lichtkegel, auf Wunsch rußig,
- ``draw_cart``    : Lore auf Schienen, die sich mit Edelsteinen füllt,
- ``draw_pickaxe`` : Spitzhacke (lehnt neben dem Maskottchen),
- ``draw_mark``    : Auto-Markierung auf einem Feld,
- ``sparkle``      : vierzackiges Funkeln,
- ``icon``         : Piktogramme der Knöpfe (Auto, Statistik, Fairness,
                     Turbo, Würfel, Schließen, Haken, Kreuz, Pfeile).

Die gecachten Flächen werden doppelt so groß gezeichnet und weich
verkleinert. Alle Formen hängen nur von Größe, Index und Zeit ab (kein
Zufall) - damit sind Python-, Web- und Rust-Fassung pixelgleich.
"""

import math

import pygame

import ui

from . import lamacrossing_draw as LD

_cache = {}

# Farben -------------------------------------------------------------------
GEM_COLS = ((30, 212, 174),      # Türkis (Spielfarbe) - der Edelstein im Feld
            (236, 72, 104),      # Rubin
            (84, 140, 248),      # Saphir
            (250, 196, 64))      # Topas
STONE = (66, 70, 98)             # Steinquader
STONE_HOT = (86, 92, 128)
STONE_D = (32, 34, 52)
STONE_L = (112, 118, 156)
PIT = (17, 18, 30)
PIT_D = (9, 9, 17)
PIT_L = (44, 48, 72)
WOOD = (112, 72, 42)
WOOD_D = (66, 40, 24)
WOOD_L = (146, 98, 58)
IRON = (104, 108, 124)
IRON_D = (60, 62, 74)
IRON_L = (168, 172, 188)
HELMET = (248, 192, 44)
MINE_BODY = (48, 46, 58)
MINE_HORN = (92, 90, 104)
MINE_RED = (255, 64, 70)
MINE_RED_OFF = (120, 32, 40)


def _cached(key, build):
    surf = _cache.get(key)
    if surf is None:
        if len(_cache) > 400:
            _cache.clear()
        surf = build()
        _cache[key] = surf
    return surf


def clear_cache():
    _cache.clear()


def _hash(i):
    """Ganzzahl-Streuwert 0..2^32-1 (deterministisch, ohne Zufall/sin)."""
    v = (i * 2654435761 + 40503) % 4294967296
    v = ((v ^ (v >> 15)) * 2246822519) % 4294967296
    return v ^ (v >> 13)


def _shade(col, f):
    """f > 0 Richtung Weiß, f < 0 Richtung Schwarz."""
    return ui.mix(col, (255, 255, 255) if f > 0 else (0, 0, 0), abs(f))


def _gradient(surface, rect, top, bottom):
    r = pygame.Rect(rect)
    h = max(1, r.h - 1)
    for y in range(r.h):
        surface.fill(ui.mix(top, bottom, y / h), (r.x, r.y + y, r.w, 1))


def _smooth(size, draw, ss=2):
    """Zeichnet draw(big, n) auf eine ss-fach große Fläche und verkleinert weich."""
    big = pygame.Surface((size * ss, size * ss), pygame.SRCALPHA)
    draw(big, size * ss)
    return pygame.transform.smoothscale(big, (size, size))


# ============================================================= Hintergrund
def _crystal(s, x, y, h, ang, col):
    """Ein schlanker Kristall (Sechseck-Prisma) mit Glanzkante."""
    ca, sa = math.cos(ang), math.sin(ang)
    w = h * 0.26

    def p(u, v):
        # u entlang der Achse (0 = Fuß, 1 = Spitze), v quer (-1..1)
        return (x + ca * u * h - sa * v * w, y + sa * u * h + ca * v * w)
    pts = [p(0.0, -0.8), p(0.78, -1.0), p(1.0, 0.0), p(0.78, 1.0), p(0.0, 0.8)]
    pygame.draw.polygon(s, _shade(col, -0.45), pts)
    pygame.draw.polygon(s, col, [p(0.0, -0.8), p(0.78, -1.0), p(1.0, 0.0), p(0.05, 0.0)])
    pygame.draw.line(s, _shade(col, 0.55), p(0.08, -0.35), p(0.80, -0.55), max(1, int(h * 0.05)))


def background(w, h):
    """Höhlenwand: dunkler Verlauf, Gesteinsbrocken, Körnung und Kristalle."""
    def build():
        k = min(w / 800.0, h / 600.0)
        s = pygame.Surface((w, h))
        _gradient(s, (0, 0, w, h), (16, 15, 28), (30, 22, 36))
        # große, weiche Gesteinsbrocken (etwas heller als die Wand)
        for i in range(22):
            v = _hash(i * 3 + 1)
            x = v % w
            y = (v >> 11) % h
            r = int((24 + (v >> 5) % 70) * max(0.6, k))
            f = ((v >> 3) % 100) / 100.0
            col = ui.mix((24, 22, 38), (34, 30, 48), f)
            pygame.draw.ellipse(s, col, (x - r, y - int(r * 0.6), 2 * r, int(r * 1.2)))
            pygame.draw.arc(s, ui.mix(col, (60, 54, 80), 0.6),
                            (x - r, y - int(r * 0.6), 2 * r, int(r * 1.2)),
                            math.pi * 0.25, math.pi * 0.85, max(1, int(2 * k)))
        # feine Körnung
        grain = ((40, 36, 56), (12, 11, 20), (28, 26, 42))
        for i in range(w * h // 420):
            v = _hash(5000 + i)
            s.fill(grain[(v >> 24) % 3], (v % w, (v // w) % h, 1, 1))
        # Kristalladern mit Lichthof
        spots = ((0.03, 0.30, 0), (0.97, 0.22, 1), (0.02, 0.78, 2), (0.98, 0.70, 0),
                 (0.50, 0.985, 1), (0.30, 0.99, 2), (0.74, 0.99, 0))
        cols = ((30, 200, 168), (150, 96, 230), (70, 150, 240))
        for i, (fx, fy, ci) in enumerate(spots):
            x, y = int(w * fx), int(h * fy)
            col = cols[ci]
            r = int(46 * max(0.6, k))
            glow = pygame.Surface((r * 2, r * 2), pygame.SRCALPHA)
            for j, a in ((3, 14), (2, 18), (1, 24)):
                pygame.draw.circle(glow, (*col, a), (r, r), int(r * j / 3))
            s.blit(glow, (x - r, y - r))
            base = -math.pi / 2 + (0.5 if fx > 0.5 else -0.5) * (1 if fy < 0.9 else 0.4)
            for j in range(3):
                hh = (18 + 9 * ((i + j) % 3)) * max(0.6, k)
                _crystal(s, x + (j - 1) * 7 * k, y, hh, base + (j - 1) * 0.42, col)
        # Vignette (Ränder dunkler)
        # 10 Ringe von außen nach innen, jeder etwas heller (Alpha 80 -> 0)
        vig = pygame.Surface((w, h), pygame.SRCALPHA)
        steps = 10
        band = max(1, int(min(w, h) * 0.012))
        for i in range(steps):
            a = 80 * (steps - i) * (steps - i) // (steps * steps)
            pygame.draw.rect(vig, (0, 0, 0, a), (i * band, i * band, w - 2 * i * band, h - 2 * i * band), band)
        s.blit(vig, (0, 0))
        return s
    return _cached(("bg", w, h), build)


# ============================================================= Spielbrett
def board(side, pad):
    """Holzrahmen (Breite pad) mit Eisenwinkeln um die dunkle Grube."""
    def build():
        s = pygame.Surface((side, side), pygame.SRCALPHA)
        rad = max(4, int(side * 0.035))
        pygame.draw.rect(s, WOOD_D, (0, 0, side, side), border_radius=rad)
        pygame.draw.rect(s, WOOD, (1, 1, side - 2, side - 3), border_radius=rad)
        # Maserung: Bretter oben/unten waagerecht, links/rechts senkrecht
        lw = max(1, side // 260)
        for j in range(3):
            f = (j + 1) / 4.0
            y0 = int(pad * f)
            pygame.draw.line(s, WOOD_D, (pad, y0), (side - pad, y0 + (j - 1)), lw)
            pygame.draw.line(s, WOOD_D, (pad, side - y0), (side - pad, side - y0 - (j - 1)), lw)
            x0 = int(pad * f)
            pygame.draw.line(s, WOOD_D, (x0, pad), (x0 + (j - 1), side - pad), lw)
            pygame.draw.line(s, WOOD_D, (side - x0, pad), (side - x0 - (j - 1), side - pad), lw)
        for i in range(6):
            v = _hash(i * 11 + 3)
            x = pad + v % max(1, side - 2 * pad)
            knot = pygame.Rect(0, 0, max(3, pad // 2), max(2, pad // 4))
            knot.center = (x, pad // 2 if i % 2 == 0 else side - pad // 2)
            pygame.draw.ellipse(s, WOOD_D, knot, max(1, lw))
        pygame.draw.line(s, WOOD_L, (rad, 1), (side - rad, 1), max(1, lw))
        # Fugen an den Ecken (Gehrung)
        for (ax, ay), (bx, by) in (((0, 0), (pad, pad)), ((side, 0), (side - pad, pad)),
                                   ((0, side), (pad, side - pad)), ((side, side), (side - pad, side - pad))):
            pygame.draw.line(s, WOOD_D, (ax, ay), (bx, by), max(1, lw))
        # Grube mit Innenschatten
        inner = pygame.Rect(pad, pad, side - 2 * pad, side - 2 * pad)
        irad = max(3, rad // 2)
        pygame.draw.rect(s, PIT_D, inner, border_radius=irad)
        pygame.draw.rect(s, (14, 14, 24), inner.inflate(-2, -2).move(0, max(1, pad // 5)),
                         border_radius=irad)
        pygame.draw.rect(s, (40, 26, 16), inner.inflate(2, 2), 1, border_radius=irad)
        # Eisenwinkel mit Nieten
        a = int(pad * 2.1)
        t = max(3, int(pad * 0.62))
        nr = max(1, int(pad * 0.14))
        for cx, cy, sx, sy in ((0, 0, 1, 1), (side, 0, -1, 1), (0, side, 1, -1), (side, side, -1, -1)):
            x0 = cx if sx > 0 else cx - a
            y0 = cy if sy > 0 else cy - t
            pygame.draw.rect(s, IRON_D, (x0, y0 + sy, a, t), border_radius=max(1, t // 3))
            pygame.draw.rect(s, IRON, (x0, y0, a, t), border_radius=max(1, t // 3))
            x1 = cx if sx > 0 else cx - t
            y1 = cy if sy > 0 else cy - a
            pygame.draw.rect(s, IRON_D, (x1 + sx, y1, t, a), border_radius=max(1, t // 3))
            pygame.draw.rect(s, IRON, (x1, y1, t, a), border_radius=max(1, t // 3))
            for ox, oy in ((t // 2, t // 2), (a - t // 2, t // 2), (t // 2, a - t // 2)):
                px, py = cx + sx * ox, cy + sy * oy
                pygame.draw.circle(s, IRON_D, (px, py + 1), nr + 1)
                pygame.draw.circle(s, IRON_L, (px, py), nr)
        return s
    return _cached(("board", side, pad), build)


# ============================================================= Felder
def _tile_rad(size):
    return max(2, int(size * 0.16))


def tile_hidden(size, var, hot=False):
    """Steinquader size x size; var 0..3 = Sprenkel-Variante."""
    def build():
        s = pygame.Surface((size, size), pygame.SRCALPHA)
        rad = _tile_rad(size)
        bev = max(2, int(size * 0.09))
        body = STONE_HOT if hot else STONE
        pygame.draw.rect(s, STONE_D, (0, 0, size, size), border_radius=rad)
        pygame.draw.rect(s, body, (0, 0, size, size - bev), border_radius=rad)
        face = pygame.Rect(bev, bev, size - 2 * bev, size - 3 * bev)
        pygame.draw.rect(s, _shade(body, 0.10), face, border_radius=max(1, rad - bev // 2))
        pygame.draw.line(s, STONE_L if not hot else _shade(STONE_L, 0.2),
                         (rad, max(1, bev // 2)), (size - rad - 1, max(1, bev // 2)), max(1, bev // 2))
        # Sprenkel und ein feiner Riss je Variante
        for j in range(5):
            v = _hash(var * 31 + j * 7 + 1)
            x = face.x + 2 + v % max(1, face.w - 4)
            y = face.y + 2 + (v >> 9) % max(1, face.h - 4)
            r = max(1, int(size * (0.022 + 0.012 * ((v >> 4) % 2))))
            col = _shade(body, -0.25) if j % 2 == 0 else _shade(body, 0.22)
            pygame.draw.circle(s, col, (x, y), r)
        if var % 2 == 1:
            # feiner Haarriss von der oberen Kante schräg nach unten
            v = _hash(var * 97 + 5)
            x0 = face.x + face.w * (0.55 + (v % 25) / 100.0)
            pts = [(x0, face.y), (x0 - face.w * 0.06, face.y + face.h * 0.16),
                   (x0 + face.w * 0.02, face.y + face.h * 0.27),
                   (x0 - face.w * 0.08, face.y + face.h * 0.42)]
            pygame.draw.lines(s, _shade(body, -0.16), False, pts, 1)
        return s
    return _cached(("hid", size, var % 4, hot), build)


def tile_open(size):
    """Ausgehobenes Feld: dunkle Grube mit Innenschatten oben und Lichtkante unten."""
    def build():
        s = pygame.Surface((size, size), pygame.SRCALPHA)
        rad = _tile_rad(size)
        bev = max(2, int(size * 0.09))
        pygame.draw.rect(s, PIT_L, (0, 0, size, size), border_radius=rad)
        pygame.draw.rect(s, PIT_D, (0, 0, size, size - max(1, bev // 2)), border_radius=rad)
        pygame.draw.rect(s, PIT, (max(1, bev // 3), bev, size - 2 * max(1, bev // 3),
                                  size - bev - max(1, bev // 2) - 1), border_radius=max(1, rad - 1))
        return s
    return _cached(("open", size), build)


def tile_boom(size):
    """Feld der getroffenen Mine: rot glühend mit dunklem Brandfleck."""
    def build():
        s = pygame.Surface((size, size), pygame.SRCALPHA)
        rad = _tile_rad(size)
        pygame.draw.rect(s, (110, 20, 28), (0, 0, size, size), border_radius=rad)
        pygame.draw.rect(s, (176, 38, 44), (1, 1, size - 2, size - 3), border_radius=rad)
        c = size // 2
        for i, col in enumerate(((214, 70, 50), (240, 120, 60), (250, 170, 90))):
            pygame.draw.circle(s, col, (c, c), int(size * (0.44 - 0.11 * i)))
        burn = pygame.Rect(0, 0, int(size * 0.64), int(size * 0.5))
        burn.center = (c, int(size * 0.58))
        pygame.draw.ellipse(s, (70, 22, 20), burn)
        for j in range(6):
            a = math.tau * j / 6 + 0.3
            x1 = c + math.cos(a) * size * 0.46
            y1 = c + math.sin(a) * size * 0.40
            pygame.draw.line(s, (90, 26, 22), (c, int(size * 0.58)), (x1, y1), max(1, size // 30))
        return s
    return _cached(("boom", size), build)


def dim(size, alpha=150):
    """Halbdurchsichtige Abdunklung eines Feldes (abgerundet)."""
    def build():
        s = pygame.Surface((size, size), pygame.SRCALPHA)
        pygame.draw.rect(s, (6, 6, 14, alpha), (0, 0, size, size), border_radius=_tile_rad(size))
        return s
    return _cached(("dim", size, alpha), build)


# ============================================================= Edelstein + Mine
def _draw_gem(s, n, col):
    TL, TR = (n * 0.30, n * 0.17), (n * 0.70, n * 0.17)
    L, R, B = (n * 0.09, n * 0.39), (n * 0.91, n * 0.39), (n * 0.50, n * 0.90)
    T = (n * 0.50, n * 0.17)
    G1, G2 = (n * 0.35, n * 0.39), (n * 0.65, n * 0.39)
    outline = [TL, TR, R, B, L]
    # weicher Schatten
    pygame.draw.ellipse(s, (0, 0, 0, 70), (n * 0.22, n * 0.84, n * 0.56, n * 0.10))
    pygame.draw.polygon(s, _shade(col, -0.55), [(x + n * 0.012, y + n * 0.02) for x, y in outline])
    pygame.draw.polygon(s, col, outline)
    pygame.draw.polygon(s, _shade(col, 0.50), [TL, T, G1, L])
    pygame.draw.polygon(s, _shade(col, 0.22), [T, TR, R, G2])
    pygame.draw.polygon(s, _shade(col, 0.70), [T, G2, G1])
    pygame.draw.polygon(s, _shade(col, -0.05), [L, G1, B])
    pygame.draw.polygon(s, _shade(col, 0.28), [G1, G2, B])
    pygame.draw.polygon(s, _shade(col, -0.38), [G2, R, B])
    edge = _shade(col, 0.82)
    lw = max(1, n // 70)
    pygame.draw.line(s, edge, L, R, lw)
    pygame.draw.lines(s, edge, True, outline, lw)
    pygame.draw.line(s, _shade(col, 0.6), G1, B, lw)
    pygame.draw.line(s, _shade(col, 0.6), G2, B, lw)
    # Glanzpunkte
    pygame.draw.circle(s, (255, 255, 255), (n * 0.33, n * 0.25), n * 0.035)
    pygame.draw.polygon(s, (255, 255, 255), [(n * 0.40, n * 0.47), (n * 0.46, n * 0.47),
                                             (n * 0.50, n * 0.70)])


def gem(size, col_idx=0):
    """Geschliffener Edelstein size x size (gecacht)."""
    col = GEM_COLS[col_idx % len(GEM_COLS)]
    return _cached(("gem", size, col_idx % len(GEM_COLS)),
                   lambda: _smooth(size, lambda s, n: _draw_gem(s, n, col)))


def _draw_mine(s, n, lit):
    cx, cy = n * 0.5, n * 0.54
    R = n * 0.25
    pygame.draw.ellipse(s, (0, 0, 0, 80), (n * 0.20, n * 0.84, n * 0.60, n * 0.10))
    lw = max(2, int(n * 0.075))
    for i in range(8):
        a = math.tau * i / 8 + math.pi / 8
        bx, by = cx + math.cos(a) * R * 0.8, cy + math.sin(a) * R * 0.8
        tx, ty = cx + math.cos(a) * R * 1.42, cy + math.sin(a) * R * 1.42
        pygame.draw.line(s, _shade(MINE_HORN, -0.3), (bx, by), (tx, ty), lw)
        pygame.draw.circle(s, MINE_HORN, (tx, ty), n * 0.058)
        pygame.draw.circle(s, _shade(MINE_HORN, 0.45), (tx - n * 0.015, ty - n * 0.015), n * 0.022)
    pygame.draw.circle(s, _shade(MINE_BODY, -0.45), (cx, cy + n * 0.02), R)
    pygame.draw.circle(s, MINE_BODY, (cx, cy), R)
    pygame.draw.ellipse(s, _shade(MINE_BODY, -0.25), (cx - R, cy - R * 0.18, 2 * R, R * 0.36),
                        max(1, int(n * 0.02)))
    pygame.draw.ellipse(s, _shade(MINE_BODY, 0.38), (cx - R * 0.70, cy - R * 0.78, R * 0.75, R * 0.55))
    pygame.draw.circle(s, (230, 230, 240), (cx - R * 0.42, cy - R * 0.55), n * 0.028)
    red = MINE_RED if lit else MINE_RED_OFF
    if lit:
        pygame.draw.circle(s, (255, 120, 110, 90), (cx, cy - R * 0.05), n * 0.13)
    pygame.draw.circle(s, _shade(red, -0.4), (cx, cy - R * 0.05), n * 0.075)
    pygame.draw.circle(s, red, (cx, cy - R * 0.05), n * 0.058)
    pygame.draw.circle(s, _shade(red, 0.6), (cx - n * 0.018, cy - R * 0.05 - n * 0.02), n * 0.018)


def mine(size, lit=True):
    """Seemine size x size mit Signallicht (lit = leuchtet)."""
    return _cached(("mine", size, bool(lit)),
                   lambda: _smooth(size, lambda s, n: _draw_mine(s, n, lit)))


def face(size, kind, dimmed=False):
    """Fertige Vorderseite eines aufgedeckten Feldes (für Aufklappen und Ruhe).

    kind: "gem" (Edelstein in der Grube), "mine" (Mine, Licht aus) oder
    "boom" (getroffene Mine auf glühendem Grund). dimmed = abgedunkelt.
    """
    def build():
        s = pygame.Surface((size, size), pygame.SRCALPHA)
        if kind == "boom":
            s.blit(tile_boom(size), (0, 0))
        else:
            s.blit(tile_open(size), (0, 0))
        g = max(4, int(size * 0.80))
        img = gem(g) if kind == "gem" else mine(g, kind == "boom")
        s.blit(img, img.get_rect(center=(size // 2, size // 2)))
        if dimmed:
            s.blit(dim(size), (0, 0))
        return s
    return _cached(("face", size, kind, bool(dimmed)), build)


def glow(r, col):
    """Lichthof mit Radius r auf schwarzem Grund - zum additiven Blitten
    (``special_flags=pygame.BLEND_RGB_ADD``), wirkt dann wie echtes Licht."""
    def build():
        s = pygame.Surface((2 * r + 2, 2 * r + 2))
        s.fill((0, 0, 0))
        steps = 8
        for i in range(steps):
            f = (i + 1) / steps                    # 1/8 .. 1 (außen -> innen)
            rr = max(1, int(r * (1.0 - i / steps)))
            a = f * f
            pygame.draw.circle(s, (int(col[0] * a), int(col[1] * a), int(col[2] * a)),
                               (r + 1, r + 1), rr)
        return s
    return _cached(("glow", r, tuple(col)), build)


def sparkle(s, x, y, r, col):
    """Vierzackiges Funkeln (Mittelpunkt x/y, Zacken-Länge r)."""
    if r < 1:
        return
    q = r * 0.28
    pts = [(x, y - r), (x + q, y - q), (x + r, y), (x + q, y + q),
           (x, y + r), (x - q, y + q), (x - r, y), (x - q, y - q)]
    pygame.draw.polygon(s, col, pts)
    pygame.draw.circle(s, (255, 255, 255), (int(x), int(y)), max(1, int(r * 0.22)))


def draw_mark(s, rect, col, t, idx):
    """Auto-Markierung: Rahmen in der Spielfarbe + kleine Raute in der Mitte."""
    r = pygame.Rect(rect)
    rad = _tile_rad(r.w)
    lw = max(2, r.w // 18)
    glow = 0.5 + 0.5 * math.sin(t * 3.0 + idx * 0.7)
    edge = ui.mix(col, (255, 255, 255), 0.25 * glow)
    pygame.draw.rect(s, edge, r.inflate(-lw, -lw), lw, border_radius=rad)
    cx, cy = r.centerx, r.centery - r.h // 20
    d = max(3, r.w // 7)
    pts = [(cx, cy - d), (cx + d, cy), (cx, cy + d), (cx - d, cy)]
    pygame.draw.polygon(s, _shade(col, -0.5), [(x + 1, y + 2) for x, y in pts])
    pygame.draw.polygon(s, edge, pts)
    pygame.draw.polygon(s, _shade(col, 0.7), [(cx, cy - d), (cx + d // 2, cy - d // 2), (cx, cy),
                                              (cx - d // 2, cy - d // 2)])


# ============================================================= Maskottchen
def _helmet(s, n):
    """Grubenhelm mit Stirnlampe auf dem Lama-Kopf (Koordinaten wie _draw_llama)."""
    cx, by = n * 0.705, n * 0.132
    rx, ry = n * 0.128, n * 0.118
    dome = [(cx + rx * math.cos(math.pi * i / 16), by - ry * math.sin(math.pi * i / 16))
            for i in range(17)]
    pygame.draw.polygon(s, _shade(HELMET, -0.35), [(x, y + n * 0.012) for x, y in dome])
    pygame.draw.polygon(s, HELMET, dome)
    hi = [(cx - rx * 0.15 + rx * 0.62 * math.cos(math.pi * (0.45 + 0.5 * i / 8)),
           by - ry * 0.18 - ry * 0.70 * math.sin(math.pi * (0.45 + 0.5 * i / 8))) for i in range(9)]
    pygame.draw.lines(s, _shade(HELMET, 0.55), False, hi, max(1, int(n * 0.02)))
    # Mittelgrat
    pygame.draw.polygon(s, _shade(HELMET, -0.15), [(cx - n * 0.012, by - ry * 0.98), (cx + n * 0.012, by - ry * 0.98),
                                                   (cx + n * 0.016, by), (cx - n * 0.016, by)])
    # Krempe
    pygame.draw.rect(s, _shade(HELMET, -0.30), (cx - rx * 1.16, by - n * 0.006, rx * 2.38, n * 0.034),
                     border_radius=int(n * 0.016))
    pygame.draw.rect(s, _shade(HELMET, -0.05), (cx - rx * 1.16, by - n * 0.010, rx * 2.38, n * 0.024),
                     border_radius=int(n * 0.012))
    # Stirnlampe
    lx, ly = cx + rx * 0.80, by - ry * 0.50
    pygame.draw.rect(s, (64, 64, 76), (lx - n * 0.040, ly - n * 0.034, n * 0.062, n * 0.068),
                     border_radius=int(n * 0.014))
    pygame.draw.circle(s, (255, 236, 160), (lx + n * 0.020, ly), n * 0.030)
    pygame.draw.circle(s, (255, 255, 230), (lx + n * 0.024, ly - n * 0.006), n * 0.013)


def _soot(s, n):
    """Rußflecken nach der Explosion (deckende Grautöne)."""
    for x, y, r in ((0.33, 0.50, 0.050), (0.47, 0.46, 0.038), (0.58, 0.55, 0.030),
                    (0.66, 0.30, 0.030), (0.80, 0.22, 0.024), (0.26, 0.56, 0.026)):
        pygame.draw.circle(s, (126, 118, 114), (n * x, n * y), n * r)
        pygame.draw.circle(s, (98, 92, 92), (n * x + n * r * 0.2, n * y + n * r * 0.2), n * r * 0.55)


def miner_llama(size, pose, scarf, soot=False):
    """Maskottchen size x size: Lama mit Grubenhelm (pose: stand/hop/fall)."""
    key = ("miner", size, pose, tuple(scarf), bool(soot))

    def draw(big, n):
        LD._draw_llama(big, n, pose, scarf)
        if soot:
            _soot(big, n)
        _helmet(big, n)
    return _cached(key, lambda: _smooth(size, draw))


def draw_pickaxe(s, x, y, h, k):
    """Spitzhacke, Stiel-Fuß bei (x, y), Höhe h (lehnt nach rechts)."""
    lw = max(2, int(h * 0.07))
    top = (x + h * 0.32, y - h)
    pygame.draw.line(s, (88, 56, 30), (x, y), top, lw + 1)
    pygame.draw.line(s, (150, 104, 60), (x + 1, y - 1), (top[0] + 1, top[1] + 1), max(1, lw // 2))
    # Kopf: gebogenes Eisen quer zum Stiel
    pts = []
    for i in range(9):
        u = -1 + 2 * i / 8
        pts.append((top[0] + u * h * 0.34, top[1] + u * u * h * 0.12 - h * 0.02))
    pygame.draw.lines(s, IRON_D, False, [(px, py + 2) for px, py in pts], lw + 1)
    pygame.draw.lines(s, IRON_L, False, pts, lw)


def draw_cart(s, x, y, w, k, fill, gem_size):
    """Lore (Unterkante der Räder bei y, linke Kante x, Breite w).

    fill = Anzahl sichtbarer Edelsteine im Wagen (0..7).
    """
    h = w * 0.46
    top = y - w * 0.16 - h
    # Schienen
    pygame.draw.line(s, (70, 64, 70), (x - w * 0.15, y), (x + w * 1.15, y), max(2, int(w * 0.035)))
    for i in range(5):
        sx = x - w * 0.1 + i * w * 0.31
        pygame.draw.line(s, (86, 58, 36), (sx, y + 1), (sx + w * 0.12, y + 1), max(2, int(w * 0.05)))
    # Edelstein-Haufen (ragt über den Rand; fy in Edelstein-Größen über der Kante)
    spots = ((0.30, 0.10, 1), (0.52, 0.06, 0), (0.74, 0.10, 2), (0.41, -0.34, 3),
             (0.63, -0.32, 0), (0.52, -0.70, 1), (0.18, -0.08, 2))
    for i in range(min(fill, len(spots))):
        fx, fy, ci = spots[i]
        img = gem(gem_size, ci)
        s.blit(img, img.get_rect(center=(int(x + w * fx), int(top + gem_size * fy))))
    # Wagenkasten (Trapez) mit Rand und Nieten
    body = [(x, top), (x + w, top), (x + w * 0.88, top + h), (x + w * 0.12, top + h)]
    pygame.draw.polygon(s, IRON_D, [(px, py + 2) for px, py in body])
    pygame.draw.polygon(s, IRON, body)
    pygame.draw.polygon(s, _shade(IRON, -0.18), [(x + w * 0.05, top + h * 0.55), (x + w * 0.95, top + h * 0.55),
                                                 (x + w * 0.88, top + h), (x + w * 0.12, top + h)])
    pygame.draw.rect(s, IRON_L, (int(x - w * 0.03), int(top - w * 0.02), int(w * 1.06), max(2, int(w * 0.07))),
                     border_radius=max(1, int(w * 0.03)))
    for i in range(4):
        rx = x + w * (0.20 + 0.2 * i)
        pygame.draw.circle(s, IRON_L, (int(rx), int(top + h * 0.32)), max(1, int(w * 0.025)))
    # Räder
    for fx in (0.27, 0.73):
        cx, cy = int(x + w * fx), int(y - w * 0.12)
        rr = max(3, int(w * 0.12))
        pygame.draw.circle(s, (30, 30, 38), (cx, cy), rr)
        pygame.draw.circle(s, (120, 120, 134), (cx, cy), max(1, rr - max(1, rr // 3)), max(1, rr // 4))
        pygame.draw.circle(s, (170, 170, 184), (cx, cy), max(1, rr // 3))


# ============================================================= Piktogramme
def icon(s, name, rect, col):
    """Piktogramm in rect (Linienstärke wächst mit der Größe)."""
    r = pygame.Rect(rect)
    cx, cy = r.centerx, r.centery
    u = r.w / 10.0
    lw = max(1, int(u * 0.9))
    if name == "auto":
        # Kreispfeil (Wiederholen) mit Play-Dreieck in der Mitte
        rad = u * 3.8
        box = pygame.Rect(0, 0, int(rad * 2), int(rad * 2))
        box.center = (cx, cy)
        pygame.draw.arc(s, col, box, math.pi * 0.35, math.pi * 1.95, max(1, lw))
        a = math.pi * 0.35
        ex, ey = cx + math.cos(a) * rad, cy - math.sin(a) * rad
        pygame.draw.polygon(s, col, [(ex + u * 1.6, ey - u * 0.2), (ex - u * 0.6, ey - u * 1.3),
                                     (ex - u * 0.2, ey + u * 1.2)])
        pygame.draw.polygon(s, col, [(cx - u * 1.0, cy - u * 1.5), (cx + u * 1.6, cy), (cx - u * 1.0, cy + u * 1.5)])
    elif name == "stats":
        base = r.bottom - u * 1.6
        for i, hh in enumerate((3.0, 5.2, 4.0, 6.8)):
            x = r.x + u * (1.4 + i * 2.0)
            pygame.draw.rect(s, col, (int(x), int(base - hh * u), max(1, int(u * 1.3)), int(hh * u)))
        pygame.draw.line(s, col, (r.x + u, base), (r.right - u, base), max(1, lw - 1 if lw > 1 else 1))
    elif name == "fair":
        pts = [(cx, r.y + u * 1.0), (cx + u * 3.6, r.y + u * 2.2), (cx + u * 3.2, cy + u * 1.4),
               (cx, r.bottom - u * 0.8), (cx - u * 3.2, cy + u * 1.4), (cx - u * 3.6, r.y + u * 2.2)]
        pygame.draw.polygon(s, col, pts, max(1, lw))
        pygame.draw.lines(s, col, False, [(cx - u * 1.6, cy), (cx - u * 0.3, cy + u * 1.4),
                                          (cx + u * 1.8, cy - u * 1.4)], max(1, lw))
    elif name == "turbo":
        pygame.draw.polygon(s, col, [(cx + u * 1.0, r.y + u * 0.6), (cx - u * 2.6, cy + u * 0.6),
                                     (cx - u * 0.2, cy + u * 0.6), (cx - u * 1.0, r.bottom - u * 0.6),
                                     (cx + u * 2.6, cy - u * 0.8), (cx + u * 0.2, cy - u * 0.8)])
    elif name == "dice":
        box = pygame.Rect(0, 0, int(u * 7.2), int(u * 7.2))
        box.center = (cx, cy)
        pygame.draw.rect(s, col, box, max(1, lw), border_radius=max(2, int(u * 1.4)))
        pr = max(1, int(u * 0.75))
        for dx, dy in ((-1, -1), (1, 1), (0, 0), (1, -1), (-1, 1)):
            pygame.draw.circle(s, col, (int(cx + dx * u * 1.9), int(cy + dy * u * 1.9)), pr)
    elif name == "close":
        d = u * 2.6
        pygame.draw.line(s, col, (cx - d, cy - d), (cx + d, cy + d), max(2, lw))
        pygame.draw.line(s, col, (cx - d, cy + d), (cx + d, cy - d), max(2, lw))
    elif name == "check":
        pygame.draw.lines(s, col, False, [(cx - u * 3.0, cy), (cx - u * 0.8, cy + u * 2.4),
                                          (cx + u * 3.2, cy - u * 2.6)], max(2, int(lw * 1.4)))
    elif name == "cross":
        d = u * 2.6
        pygame.draw.line(s, col, (cx - d, cy - d), (cx + d, cy + d), max(2, int(lw * 1.4)))
        pygame.draw.line(s, col, (cx - d, cy + d), (cx + d, cy - d), max(2, int(lw * 1.4)))
    elif name in ("left", "right"):
        sx = -1 if name == "left" else 1
        pygame.draw.polygon(s, col, [(cx + sx * u * 2.0, cy), (cx - sx * u * 1.4, cy - u * 2.6),
                                     (cx - sx * u * 1.4, cy + u * 2.6)])
    elif name == "gem":
        img = gem(max(4, int(r.w * 0.9)))
        s.blit(img, img.get_rect(center=(cx, cy)))
    elif name == "mine":
        img = mine(max(4, int(r.w * 0.9)))
        s.blit(img, img.get_rect(center=(cx, cy)))
