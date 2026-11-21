# -*- coding: utf-8 -*-
"""
lamaplinko_draw.py
==================
Zeichen-Bausteine von Lama Plinko (nur pygame-Primitive, keine Bilddateien).
Alles Teure wird einmal gerendert und gecacht - auch mit 50 Kugeln im Flug
kostet ein Bild nur ein paar hundert Blits:

- ``background``   : dunkler Spielhallen-Verlauf mit orangem Lichtschein hinter
                     dem Brett und feinem Punktraster (je Auflösung),
- ``board_plate``  : halbdurchsichtige Trapez-Platte unter den Stiften,
- ``peg_layer``    : alle Stifte eines Bretts auf einer Fläche (je Reihenzahl
                     und Größe), ``peg_glow`` = Leuchten eines getroffenen Stifts,
- ``yarn_ball``    : die Kugel - ein goldenes Wollknäuel, 4-fach überabgetastet,
                     in 12 Drehstufen; ``trail_dot`` = Leuchtpunkt der Spur,
- ``slot_color``   : Farbe eines Fachs nach Multiplikator (Mitte kühl, außen heiß),
- ``slot_surface`` : ein Fach mit Verlauf, Kante und Beschriftung,
- ``funnel``       : Einwurf-Trichter unter dem Lama,
- ``llama_bow``    : das Lama oben am Brett, um die Hufe gedreht (Nicken),
- ``icon``         : Piktogramme der Knöpfe (Auto, Statistik, Fairness, Turbo,
                     Schließen, Pfeile),
- ``path_preview`` : Mini-Vorschau eines Kugelwegs (Fairness-Fenster).

Alle Formen hängen nur von Größen und ganzzahligen Werten ab (kein Zufall,
keine Zeit) - damit lassen sich Web- und Rust-Fassung pixelgleich nachbauen.
Farb-Verläufe rechnen mit Ganzzahlen (``_lerp_i``).
"""

import math

import pygame

import ui

from . import lamacrossing_draw as LCD

_cache = {}

GOLD = (236, 184, 64)
GOLD_L = (255, 228, 140)
GOLD_D = (168, 112, 30)
GOLD_DD = (110, 68, 18)
PEG = (226, 230, 246)
PEG_D = (120, 124, 150)

# Fach-Farben: (Multiplikator x100, Farbe) - dazwischen linear (ganzzahlig)
SLOT_STOPS = (
    (0, (40, 96, 196)),
    (20, (52, 120, 222)),
    (50, (40, 170, 206)),
    (100, (70, 196, 140)),
    (150, (150, 210, 84)),
    (200, (236, 206, 60)),
    (500, (248, 154, 44)),
    (2000, (240, 86, 52)),
    (10000, (226, 40, 86)),
    (100000, (200, 50, 226)),
)


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


def _ri(v):
    """Kaufmännisch runden (x.5 nach oben), in Python, JS und Rust gleich."""
    return int(math.floor(v + 0.5))


def _lerp_i(c1, c2, num, den):
    """Ganzzahlige Farbmischung c1 + (c2 - c1) * num / den (abgerundet)."""
    den = max(1, den)
    return tuple(a + (b - a) * num // den for a, b in zip(c1, c2))


def _gradient(surface, rect, top, bottom):
    """Senkrechter Farbverlauf in rect (ganzzahlig gemischt)."""
    r = pygame.Rect(rect)
    h = max(1, r.h - 1)
    for y in range(r.h):
        surface.fill(_lerp_i(top, bottom, y, h), (r.x, r.y + y, r.w, 1))


def _circle_mask(size, c, r):
    mask = pygame.Surface((size, size), pygame.SRCALPHA)
    mask.fill((255, 255, 255, 0))
    pygame.draw.circle(mask, (255, 255, 255, 255), c, r)
    return mask


# ============================================================= Hintergrund
def background(w, h, glow_center, glow_r, accent):
    """Spielhallen-Hintergrund: Verlauf, Lichtschein hinter dem Brett, Raster."""
    def build():
        s = pygame.Surface((w, h))
        _gradient(s, (0, 0, w, h), (26, 16, 38), (8, 6, 16))
        # weicher Lichtschein: konzentrische Kreise von außen nach innen, jeder
        # überschreibt die Mitte mit etwas mehr Deckkraft (Schritte ~1,5 -
        # ergibt einen glatten Kegel ohne sichtbare Ringe)
        gx, gy = int(glow_center[0]), int(glow_center[1])
        glow = pygame.Surface((w, h), pygame.SRCALPHA)
        steps = 40
        rgb = (accent[0], accent[1] // 2 + 40, accent[2] // 3)
        for i in range(steps, 0, -1):
            a = (steps - i + 1) * 58 // steps
            pygame.draw.circle(glow, (*rgb, a), (gx, gy), max(2, glow_r * i // steps))
        s.blit(glow, (0, 0))
        # feines Punktraster
        step = max(14, min(w, h) // 26)
        dot = (40, 30, 56)
        for yy in range(step // 2, h, step):
            for xx in range(step // 2 + (yy // step % 2) * (step // 2), w, step):
                s.fill(dot, (xx, yy, 1 + (w >= 1200), 1 + (w >= 1200)))
        # dunkle Ränder (Vignette)
        vig = pygame.Surface((w, h), pygame.SRCALPHA)
        band = max(8, min(w, h) // 7)
        for i in range(band):
            a = (band - i) * 90 // band
            pygame.draw.rect(vig, (0, 0, 0, a // 3), (i, i, w - 2 * i, h - 2 * i), 1)
        s.blit(vig, (0, 0))
        return s
    return _cached(("bg", w, h, int(glow_center[0]), int(glow_center[1]), glow_r, tuple(accent)), build)


def board_plate(points, w, h, accent):
    """Trapez-Platte unter den Stiften (points = 4 Ecken, Bildschirmkoordinaten)."""
    pts = tuple((int(x), int(y)) for x, y in points)

    def build():
        s = pygame.Surface((w, h), pygame.SRCALPHA)
        # Verlauf (oben warm, unten dunkel), mit dem Trapez ausgestanzt
        top = (accent[0] // 4 + 22, accent[1] // 6 + 12, accent[2] // 6 + 26)
        bot = (accent[0] // 9 + 14, accent[1] // 12 + 8, accent[2] // 10 + 22)
        y0 = min(p[1] for p in pts)
        y1 = max(p[1] for p in pts)
        hh = max(1, y1 - y0)
        for y in range(y0, y1 + 1):
            c = _lerp_i(top, bot, y - y0, hh)
            s.fill((c[0], c[1], c[2], 150), (0, y, w, 1))
        mask = pygame.Surface((w, h), pygame.SRCALPHA)
        pygame.draw.polygon(mask, (255, 255, 255, 255), pts)
        s.blit(mask, (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
        pygame.draw.polygon(s, (*ui.mix(accent, (255, 255, 255), 0.25), 80), pts, 2)
        # Lichtkante oben + feiner Schimmer an den Schrägen
        pygame.draw.line(s, (*ui.mix(accent, (255, 255, 255), 0.55), 140), pts[0], pts[1], 2)
        pygame.draw.line(s, (*ui.mix(accent, (255, 255, 255), 0.4), 50),
                         (pts[0][0] + 3, pts[0][1] + 2), (pts[3][0] + 3, pts[3][1] - 2), 1)
        pygame.draw.line(s, (*ui.mix(accent, (255, 255, 255), 0.4), 50),
                         (pts[1][0] - 3, pts[1][1] + 2), (pts[2][0] - 3, pts[2][1] - 2), 1)
        return s
    return _cached(("plate", pts, w, h, tuple(accent)), build)


# ============================================================= Stifte
def peg_sprite(r):
    """Ein Stift: Schatten, Körper, Glanzpunkt (4-fach überabgetastet)."""
    def build():
        ss = 4
        size = (2 * r + 4) * ss
        big = pygame.Surface((size, size), pygame.SRCALPHA)
        c = (size // 2, size // 2)
        R = r * ss
        pygame.draw.circle(big, (0, 0, 0, 110), (c[0] + ss, c[1] + 2 * ss), R)
        pygame.draw.circle(big, PEG_D, c, R)
        pygame.draw.circle(big, PEG, (c[0] - ss // 2, c[1] - ss // 2), R - ss)
        pygame.draw.circle(big, (255, 255, 255), (c[0] - R // 3, c[1] - R // 3), max(ss, R // 3))
        return pygame.transform.smoothscale(big, (size // ss, size // ss))
    return _cached(("peg", r), build)


def peg_layer(w, h, pegs, r):
    """Alle Stifte (Liste ganzzahliger Mittelpunkte) auf einer Fläche w x h."""
    key = ("pegs", w, h, r, tuple(pegs))

    def build():
        s = pygame.Surface((w, h), pygame.SRCALPHA)
        spr = peg_sprite(r)
        off = spr.get_width() // 2
        for (x, y) in pegs:
            s.blit(spr, (x - off, y - off))
        return s
    return _cached(key, build)


def peg_glow(r, level):
    """Leuchten eines getroffenen Stifts; level 1..8 (8 = frisch getroffen)."""
    def build():
        R = int(r * 3.2) + 2
        s = pygame.Surface((2 * R + 2, 2 * R + 2), pygame.SRCALPHA)
        c = (R + 1, R + 1)
        for i in range(6, 0, -1):
            rr = max(1, R * i // 6)
            a = (7 - i) * 10 * level // 8
            pygame.draw.circle(s, (255, 214, 120, a), c, rr)
        pygame.draw.circle(s, (255, 250, 220, 40 + 26 * level), c, r + 1)
        return s
    return _cached(("glow", r, level), build)


# ============================================================= Kugel
def yarn_ball(r, step):
    """Goldenes Wollknäuel mit Radius r in Drehstufe step (0..11)."""
    step %= 12

    def build():
        ss = 4
        size = (2 * r + 2) * ss
        R = r * ss
        c = (size // 2, size // 2)
        big = pygame.Surface((size, size), pygame.SRCALPHA)
        pygame.draw.circle(big, GOLD_D, c, R)
        pygame.draw.circle(big, GOLD, (c[0] - R // 8, c[1] - R // 8), R - R // 7)
        # Wollfäden: gebogene Linien quer über das Knäuel, gedreht je Stufe
        ang = step * math.pi / 6.0
        ca, sa = math.cos(ang), math.sin(ang)
        lw = max(ss, R // 6)
        for i, (off, col) in enumerate(((-0.52, GOLD_DD), (-0.12, GOLD_D), (0.30, GOLD_DD),
                                        (0.66, GOLD_D), (0.05, GOLD_L))):
            half = math.sqrt(max(0.0, 1.0 - off * off)) * 0.98
            pts = []
            for j in range(9):
                u = -half + 2 * half * j / 8
                v = off + 0.22 * (1 - (u / half) ** 2 if half > 0 else 0)
                x, y = u * R, v * R
                pts.append((c[0] + x * ca - y * sa, c[1] + x * sa + y * ca))
            pygame.draw.lines(big, col, False, pts, lw if i < 4 else max(ss, lw * 2 // 3))
        # zweite Fadenlage quer dazu
        ang2 = ang + math.pi / 2.4
        ca, sa = math.cos(ang2), math.sin(ang2)
        for off in (-0.34, 0.40):
            half = math.sqrt(max(0.0, 1.0 - off * off)) * 0.98
            pts = []
            for j in range(9):
                u = -half + 2 * half * j / 8
                v = off - 0.18 * (1 - (u / half) ** 2)
                x, y = u * R, v * R
                pts.append((c[0] + x * ca - y * sa, c[1] + x * sa + y * ca))
            pygame.draw.lines(big, GOLD_D, False, pts, max(ss, lw * 3 // 4))
        # Licht oben links, Schatten unten rechts (fest, dreht nicht mit)
        shade = pygame.Surface((size, size), pygame.SRCALPHA)
        pygame.draw.circle(shade, (90, 50, 10, 90), (c[0] + R // 3, c[1] + R // 3), R)
        pygame.draw.circle(shade, (0, 0, 0, 0), (c[0] - R // 6, c[1] - R // 6), R * 4 // 5)
        big.blit(shade, (0, 0))
        pygame.draw.circle(big, (255, 246, 210), (c[0] - R * 2 // 5, c[1] - R * 2 // 5), max(ss, R // 5))
        big.blit(_circle_mask(size, c, R), (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
        pygame.draw.circle(big, (70, 40, 10), c, R, max(ss, R // 9))
        return pygame.transform.smoothscale(big, (size // ss, size // ss))
    return _cached(("yarn", r, step), build)


def trail_dot(r, alpha):
    """Leuchtpunkt der Kugelspur (alpha in Stufen von 16)."""
    alpha = max(16, min(255, alpha // 16 * 16))

    def build():
        R = max(1, r)
        s = pygame.Surface((2 * R + 2, 2 * R + 2), pygame.SRCALPHA)
        pygame.draw.circle(s, (255, 196, 80, alpha // 2), (R + 1, R + 1), R)
        pygame.draw.circle(s, (255, 230, 150, alpha), (R + 1, R + 1), max(1, R * 3 // 5))
        return s
    return _cached(("trail", r, alpha), build)


# ============================================================= Fächer
def slot_color(m100):
    """Farbe eines Fachs: kühl (unter x1) über Gelb und Orange bis Magenta."""
    m100 = max(0, int(m100))
    for (a, ca), (b, cb) in zip(SLOT_STOPS, SLOT_STOPS[1:]):
        if m100 <= b:
            return _lerp_i(ca, cb, m100 - a, b - a)
    return SLOT_STOPS[-1][1]


def _text_dark(col):
    """Dunkle Schrift auf hellen Fächern, sonst weiße."""
    return (col[0] * 299 + col[1] * 587 + col[2] * 114) // 1000 > 150


def fit_font(text, max_w, size, bold=True, min_size=7):
    """Größte Schrift (ab size abwärts), in der text in max_w passt."""
    while size > min_size and ui.font(size, bold=bold).size(text)[0] > max_w:
        size -= 1
    return ui.font(size, bold=bold)


def slot_surface(w, h, m100, label, font_size, hot=False):
    """Ein Fach w x h: Verlauf in der Multiplikator-Farbe, Kante, Text."""
    def build():
        col = slot_color(m100)
        if hot:
            col = ui.mix(col, (255, 255, 255), 0.28)
        s = pygame.Surface((w, h + 3), pygame.SRCALPHA)
        rad = max(2, min(w, h) // 4)
        dark = ui.mix(col, (0, 0, 0), 0.45)
        pygame.draw.rect(s, dark, (0, 3, w, h), border_radius=rad)
        body = pygame.Surface((w, h), pygame.SRCALPHA)
        _gradient(body, (0, 0, w, h), ui.mix(col, (255, 255, 255), 0.30), ui.mix(col, (0, 0, 0), 0.12))
        mask = pygame.Surface((w, h), pygame.SRCALPHA)
        pygame.draw.rect(mask, (255, 255, 255, 255), (0, 0, w, h), border_radius=rad)
        body.blit(mask, (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
        s.blit(body, (0, 0))
        pygame.draw.rect(s, ui.mix(col, (255, 255, 255), 0.55), (rad, 1, max(1, w - 2 * rad), 1))
        if hot:
            pygame.draw.rect(s, (255, 255, 255), (0, 0, w, h), 1, border_radius=rad)
        fg = (34, 22, 12) if _text_dark(col) else (255, 255, 255)
        fnt = fit_font(label, w - 2, font_size)
        img = fnt.render(label, True, fg)
        if img.get_width() > w - 1:
            # winzige Fächer: Text weich auf die Fachbreite verkleinern
            f = (w - 1) / img.get_width()
            img = pygame.transform.smoothscale(img, (max(1, int(img.get_width() * f)),
                                                     max(1, int(img.get_height() * f))))
        s.blit(img, img.get_rect(center=(w // 2, h // 2)))
        return s
    return _cached(("slot", w, h, m100, label, font_size, hot), build)


def pill(w, h, m100, label, font_size):
    """Schild im Verlauf (Farbe wie das getroffene Fach)."""
    def build():
        col = slot_color(m100)
        s = pygame.Surface((w, h), pygame.SRCALPHA)
        pygame.draw.rect(s, ui.mix(col, (0, 0, 0), 0.40), (0, 1, w, h - 1), border_radius=h // 2)
        pygame.draw.rect(s, col, (0, 0, w, h - 2), border_radius=h // 2)
        pygame.draw.rect(s, ui.mix(col, (255, 255, 255), 0.45), (h // 2, 1, max(1, w - h), 1))
        fg = (34, 22, 12) if _text_dark(col) else (255, 255, 255)
        fnt = fit_font(label, w - 6, font_size)
        img = fnt.render(label, True, fg)
        s.blit(img, img.get_rect(center=(w // 2, (h - 2) // 2)))
        return s
    return _cached(("pill", w, h, m100, label, font_size), build)


# ============================================================= Trichter + Lama
def funnel(w, h, accent):
    """Einwurf-Trichter (w breit oben, h hoch), Spitze unten in der Mitte."""
    def build():
        ss = 3
        W, H = w * ss, h * ss
        big = pygame.Surface((W, H), pygame.SRCALPHA)
        mid = W // 2
        neck = max(ss * 4, W * 2 // 9)
        rim_h = max(ss * 2, H // 6)
        cone = H * 13 // 20
        # Trichter-Körper: Verlauf in Akzent-Dunkel, mit der Form ausgestanzt
        body = [(0, rim_h // 2), (W, rim_h // 2), (mid + neck, cone), (mid + neck, H),
                (mid - neck, H), (mid - neck, cone)]
        grad = pygame.Surface((W, H), pygame.SRCALPHA)
        _gradient(grad, (0, 0, W, H), ui.mix(accent, (40, 20, 30), 0.35), ui.mix(accent, (10, 6, 16), 0.75))
        mask = pygame.Surface((W, H), pygame.SRCALPHA)
        pygame.draw.polygon(mask, (255, 255, 255, 255), body)
        grad.blit(mask, (0, 0), special_flags=pygame.BLEND_RGBA_MIN)
        big.blit(grad, (0, 0))
        # dunkle Öffnung + Glanzstreifen links
        inner = [(W // 7, rim_h), (W - W // 7, rim_h), (mid + neck * 2 // 3, cone), (mid - neck * 2 // 3, cone)]
        pygame.draw.polygon(big, (18, 10, 24, 220), inner)
        pygame.draw.line(big, (*ui.mix(accent, (255, 255, 255), 0.5), 160), (W // 9, rim_h + ss),
                         (mid - neck + ss, cone), max(ss, W // 30))
        # Ring am Hals und goldener Rand oben
        pygame.draw.rect(big, GOLD_D, (mid - neck - ss, cone, 2 * neck + 2 * ss, max(ss * 2, H // 9)),
                         border_radius=ss)
        pygame.draw.rect(big, GOLD_D, (0, 0, W, rim_h), border_radius=rim_h // 2)
        pygame.draw.rect(big, GOLD, (ss, 0, W - 2 * ss, rim_h - ss), border_radius=rim_h // 2)
        pygame.draw.line(big, GOLD_L, (rim_h, ss), (W - rim_h, ss), max(1, ss))
        return pygame.transform.smoothscale(big, (w, h))
    return _cached(("funnel", w, h, tuple(accent)), build)


LLAMA_PIVOT = (0.42, 0.94)     # Drehpunkt (Hufe) relativ zur Sprite-Größe
LLAMA_MOUTH = (0.895, 0.225)   # Schnauzenspitze relativ zur Sprite-Größe (Blick nach rechts)


def llama_bow(n, deg, scarf):
    """Lama-Sprite (Größe n) um deg Grad gedreht (negativ = nach vorn gebeugt)."""
    def build():
        if deg == 0:
            return LCD.llama_sprite(n, "stand", scarf)
        # doppelt so groß zeichnen, drehen, weich verkleinern (glatte Kanten)
        big = pygame.Surface((n * 2, n * 2), pygame.SRCALPHA)
        LCD._draw_llama(big, n * 2, "stand", scarf)
        rot = pygame.transform.rotate(big, deg)
        return pygame.transform.smoothscale(rot, (rot.get_width() // 2, rot.get_height() // 2))
    return _cached(("llama", n, deg, tuple(scarf)), build)


def llama_blit_pos(img, n, deg, foot_x, foot_y):
    """Linke obere Ecke für das gedrehte Sprite img (= llama_bow(n, deg, …)),
    sodass die Hufe (LLAMA_PIVOT) bei (foot_x, foot_y) stehen bleiben.

    pygame.transform.rotate dreht um die Bildmitte und vergrößert die Fläche;
    der Vektor Mitte -> Hufe wird deshalb mitgedreht (y zeigt nach unten).
    """
    a = math.radians(deg)
    vx = (LLAMA_PIVOT[0] - 0.5) * n
    vy = (LLAMA_PIVOT[1] - 0.5) * n
    rx = vx * math.cos(a) + vy * math.sin(a)
    ry = -vx * math.sin(a) + vy * math.cos(a)
    rw, rh = img.get_size()
    return _ri(foot_x - rx - rw / 2), _ri(foot_y - ry - rh / 2)


def mouth_pos(n, deg, foot_x, foot_y):
    """Bildschirmposition des Mauls (dort startet die Kugel)."""
    a = math.radians(deg)
    vx = (LLAMA_MOUTH[0] - LLAMA_PIVOT[0]) * n
    vy = (LLAMA_MOUTH[1] - LLAMA_PIVOT[1]) * n
    return (foot_x + vx * math.cos(a) + vy * math.sin(a),
            foot_y - vx * math.sin(a) + vy * math.cos(a))


# ============================================================= Piktogramme
def icon(name, size, col):
    """Piktogramm size x size: auto, stats, fair, turbo, close, left, right, check, cross."""
    def build():
        ss = 4
        n = size * ss
        s = pygame.Surface((n, n), pygame.SRCALPHA)
        lw = max(ss, n // 9)
        if name == "auto":
            # Kreispfeil mit Spitze + Play-Dreieck in der Mitte
            c, r = (n // 2, n // 2), n * 2 // 5
            pygame.draw.arc(s, col, (c[0] - r, c[1] - r, 2 * r, 2 * r), 0.55, 5.75, lw)
            tip = (c[0] + r * math.cos(0.55), c[1] - r * math.sin(0.55))
            pygame.draw.polygon(s, col, [(tip[0] - n * 0.16, tip[1] - n * 0.02),
                                         (tip[0] + n * 0.10, tip[1] - n * 0.06),
                                         (tip[0] - n * 0.02, tip[1] + n * 0.18)])
            pygame.draw.polygon(s, col, [(n * 0.42, n * 0.34), (n * 0.66, n * 0.5), (n * 0.42, n * 0.66)])
        elif name == "stats":
            bw = n // 5
            for i, hh in enumerate((0.38, 0.62, 0.86)):
                x = n * 0.14 + i * (bw + n * 0.09)
                pygame.draw.rect(s, col, (int(x), int(n * (0.92 - hh)), bw, int(n * hh)),
                                 border_radius=max(1, ss))
        elif name == "fair":
            pts = [(n * 0.5, n * 0.06), (n * 0.88, n * 0.2), (n * 0.82, n * 0.6), (n * 0.5, n * 0.95),
                   (n * 0.18, n * 0.6), (n * 0.12, n * 0.2)]
            pygame.draw.polygon(s, col, pts, lw)
            pygame.draw.lines(s, col, False, [(n * 0.32, n * 0.5), (n * 0.46, n * 0.64),
                                              (n * 0.7, n * 0.36)], lw)
        elif name == "turbo":
            pygame.draw.polygon(s, col, [(n * 0.58, n * 0.04), (n * 0.2, n * 0.56), (n * 0.48, n * 0.56),
                                         (n * 0.38, n * 0.96), (n * 0.82, n * 0.4), (n * 0.53, n * 0.4)])
        elif name == "close":
            pygame.draw.line(s, col, (n * 0.22, n * 0.22), (n * 0.78, n * 0.78), lw)
            pygame.draw.line(s, col, (n * 0.78, n * 0.22), (n * 0.22, n * 0.78), lw)
        elif name in ("left", "right"):
            pts = [(n * 0.66, n * 0.18), (n * 0.30, n * 0.5), (n * 0.66, n * 0.82)]
            if name == "right":
                pts = [(n - x, y) for x, y in pts]
            pygame.draw.polygon(s, col, pts)
        elif name == "check":
            pygame.draw.lines(s, col, False, [(n * 0.16, n * 0.52), (n * 0.4, n * 0.76),
                                              (n * 0.84, n * 0.26)], max(ss, n // 7))
        elif name == "cross":
            w2 = max(ss, n // 7)
            pygame.draw.line(s, col, (n * 0.2, n * 0.2), (n * 0.8, n * 0.8), w2)
            pygame.draw.line(s, col, (n * 0.8, n * 0.2), (n * 0.2, n * 0.8), w2)
        return pygame.transform.smoothscale(s, (size, size))
    return _cached(("icon", name, size, tuple(col)), build)


# ============================================================= Fairness-Vorschau
def path_preview(rows, path, w, h, m100=None):
    """Mini-Brett w x h mit Stiften und dem Weg der Kugel (path = Liste 0/1)."""
    path = tuple(int(b) for b in path)

    def build():
        ss = 2
        W, H = w * ss, h * ss
        big = pygame.Surface((W, H), pygame.SRCALPHA)
        n = max(1, rows)
        gap = min(W / (n + 2.4), H / (n + 1.6))
        cx = W / 2
        top = gap * 0.7
        dot = max(1, int(gap * 0.12))
        for r in range(n):
            for j in range(r + 3):
                x = cx + (j - (r + 2) / 2) * gap
                pygame.draw.circle(big, (150, 156, 190), (int(x), int(top + r * gap)), dot)
        pts = [(cx, top - gap * 0.6)]
        k = 0
        for r in range(n):
            pts.append((cx + (k - r / 2) * gap, top + r * gap - gap * 0.3))
            if r < len(path):
                k += path[r]
        sy = top + n * gap - gap * 0.2
        sx = cx + (k - n / 2) * gap
        pts.append((sx, sy))
        col = slot_color(m100) if m100 is not None else GOLD
        box = pygame.Rect(0, 0, max(2, int(gap * 0.9)), max(2, int(gap * 0.7)))
        box.center = (int(sx), int(sy + gap * 0.2))
        pygame.draw.rect(big, col, box, border_radius=max(1, box.h // 3))
        pygame.draw.lines(big, GOLD_L, False, [(int(x), int(y)) for x, y in pts], max(2, int(gap * 0.16)))
        pygame.draw.circle(big, GOLD, (int(sx), int(sy)), max(2, int(gap * 0.28)))
        return pygame.transform.smoothscale(big, (w, h))
    return _cached(("preview", rows, path, w, h, m100), build)
