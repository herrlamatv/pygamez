# -*- coding: utf-8 -*-
"""
casino_draw.py
==============
Zeichen-Bausteine des Casinos (alles aus pygame-Primitiven, keine Bilddateien):

- ``chip_surface`` / ``draw_chip_stack`` : Jetons mit Kantenstreifen und Wert,
- ``WheelArt``      : Roulette-Kessel (feste Schüssel + drehender Zahlenkranz),
                      einmal je Größe gerendert und gecacht,
- ``symbol_surface``: die Symbole des Lama-Slots (Lama, Goldmünze, Sieben,
                      Diamant, Glocke, Kleeblatt, Trauben, Zitrone, Kirschen),
                      2-fach überabgetastet und weich verkleinert (glatte Kanten),
- ``draw_reel``     : eine Walze an beliebiger (gebrochener) Position zeichnen -
                      angelehnt an die Walzen von ``snake.py _draw_slot``, aber mit
                      Bewegungsunschärfe, Zylinder-Schattierung und Hervorhebung.

Farben hier sind Identitätsfarben (Filz, Jetons, Symbole); generische UI-Farben
liest das Spiel zur Zeichenzeit aus ``ui.*``.
"""

import math

import pygame

import ui

from . import casino_logic as L

# ------------------------------------------------------------ Identität
COL_RED = (196, 36, 48)
COL_BLACK = (26, 28, 34)
COL_GREEN = (22, 138, 76)
COL_GOLD = (232, 190, 92)
COL_GOLD_D = (150, 110, 40)
COL_WOOD = (92, 52, 28)
COL_WOOD_D = (58, 32, 18)
COL_FELT = (18, 92, 58)
COL_FELT_D = (10, 56, 36)

CHIP_COLS = {
    1: ((236, 236, 242), (40, 90, 190)),
    5: ((204, 46, 56), (255, 255, 255)),
    25: ((38, 150, 86), (255, 255, 255)),
    100: ((30, 32, 40), (255, 222, 120)),
    500: ((132, 70, 196), (255, 255, 255)),
}

LINE_COLS = ((255, 214, 64), (84, 200, 255), (255, 110, 150), (130, 235, 120),
             (255, 150, 60), (190, 130, 255), (60, 230, 200), (255, 90, 90),
             (160, 200, 60), (240, 240, 255))

_cache = {}


def _cached(key, build):
    surf = _cache.get(key)
    if surf is None:
        if len(_cache) > 220:
            _cache.clear()
        surf = build()
        _cache[key] = surf
    return surf


def clear_cache():
    _cache.clear()


def fmt_amount(n):
    """Kurzform für Jeton-Beschriftungen: 1250 -> '1.2k'."""
    n = int(n)
    if n >= 10000:
        return f"{n // 1000}k"
    if n >= 1000:
        v = n / 1000.0
        return (f"{v:.1f}".rstrip("0").rstrip(".")) + "k"
    return str(n)


def number_color(n):
    c = L.color_of(n)
    return COL_GREEN if c == "green" else (COL_RED if c == "red" else COL_BLACK)


# ============================================================= Jetons
def chip_surface(value, r, label=None):
    """Ein Jeton mit Radius r (gecacht je Wert/Größe/Beschriftung)."""
    label = fmt_amount(value) if label is None else label
    key = ("chip", value, r, label)

    def build():
        base, txt = CHIP_COLS.get(value, CHIP_COLS[_chip_for(value)])
        ss = 3
        R = r * ss
        size = 2 * R + 2 * ss
        big = pygame.Surface((size, size), pygame.SRCALPHA)
        c = (size // 2, size // 2)
        edge = ui.mix(base, (0, 0, 0), 0.35)
        pygame.draw.circle(big, edge, (c[0], c[1] + ss), R)       # Kante
        pygame.draw.circle(big, base, c, R)
        stripe = (250, 250, 250) if value != 1 else (40, 90, 190)
        for i in range(6):                                         # Kantenstreifen
            a = i * math.tau / 6
            pts = []
            for da in (-0.17, 0.17):
                for rr in (R * 0.99, R * 0.74):
                    pts.append((c[0] + math.cos(a + da) * rr,
                                c[1] + math.sin(a + da) * rr))
            pygame.draw.polygon(big, stripe, [pts[0], pts[2], pts[3], pts[1]])
        pygame.draw.circle(big, ui.mix(base, (255, 255, 255), 0.12), c,
                           int(R * 0.66))
        pygame.draw.circle(big, ui.mix(stripe, base, 0.35), c, int(R * 0.66),
                           max(1, ss))
        surf = pygame.transform.smoothscale(big, (size // ss, size // ss))
        fnt = ui.font(max(8, int(r * (0.78 if len(label) <= 2 else 0.62))),
                      bold=True)
        img = fnt.render(label, True, txt)
        surf.blit(img, img.get_rect(center=(surf.get_width() // 2,
                                            surf.get_height() // 2)))
        return surf
    return _cached(key, build)


def _chip_for(amount):
    """Größter Jetonwert, der in amount passt (für die Farbe eines Stapels)."""
    best = L.CHIP_VALUES[0]
    for v in L.CHIP_VALUES:
        if amount >= v:
            best = v
    return best


def draw_chip_stack(surface, center, amount, r, alpha=255, lift=0):
    """Stapel aus passenden Jetons; der oberste trägt die Gesamtsumme."""
    rest = int(amount)
    chips = []
    for v in reversed(L.CHIP_VALUES):
        while rest >= v and len(chips) < 5:
            chips.append(v)
            rest -= v
    if not chips:
        chips = [L.CHIP_VALUES[0]]
    chips.reverse()
    cx, cy = int(center[0]), int(center[1]) - lift
    step = max(2, r // 5)
    for i, v in enumerate(chips):
        top = i == len(chips) - 1
        img = chip_surface(v, r, fmt_amount(amount) if top else "")
        if alpha < 255:
            img = img.copy()
            img.set_alpha(alpha)
        surface.blit(img, img.get_rect(center=(cx, cy - i * step)))


# ============================================================= Roulette-Kessel
class WheelArt:
    """Kessel in Durchmesser d: bowl (fest) + rotor (dreht) + Geometrie."""

    def __init__(self, d):
        self.d = d
        self.R = d / 2.0
        self.track_r = self.R * 0.855        # Kugelbahn
        self.pocket_r = self.R * 0.555       # Kugel liegt im Fach
        self.rotor_r = self.R * 0.78
        self.bowl = self._build_bowl()
        self.rotor = self._build_rotor()

    def _build_bowl(self):
        d, R = self.d, self.R
        ss = 2
        big = pygame.Surface((d * ss, d * ss), pygame.SRCALPHA)
        c = (d * ss // 2, d * ss // 2)
        RR = R * ss
        # Schatten, Holzrand, Kugelbahn
        pygame.draw.circle(big, (0, 0, 0, 90), (c[0] + 3 * ss, c[1] + 5 * ss),
                           int(RR))
        for i in range(10):
            f = i / 9
            pygame.draw.circle(big, ui.mix(COL_WOOD, COL_WOOD_D, f),
                               c, int(RR * (1.0 - 0.012 * i)))
        pygame.draw.circle(big, COL_GOLD_D, c, int(RR * 0.905), max(1, ss))
        for i in range(12):
            f = i / 11
            pygame.draw.circle(big, ui.mix((58, 44, 36), (24, 18, 16), f),
                               c, int(RR * (0.90 - 0.008 * i)))
        pygame.draw.circle(big, (90, 70, 56), c, int(RR * 0.80), max(1, ss))
        # Rauten-Abweiser auf der Bahn
        for i in range(8):
            a = i * math.tau / 8 + math.tau / 16
            rr = RR * 0.84
            px, py = c[0] + math.sin(a) * rr, c[1] - math.cos(a) * rr
            s = RR * 0.035
            pts = [(px + math.sin(a) * s * 1.6, py - math.cos(a) * s * 1.6),
                   (px + math.cos(a) * s, py + math.sin(a) * s),
                   (px - math.sin(a) * s * 1.6, py + math.cos(a) * s * 1.6),
                   (px - math.cos(a) * s, py - math.sin(a) * s)]
            pygame.draw.polygon(big, COL_GOLD, pts)
        # Lichtreflex oben links auf dem Holzrand
        glow = pygame.Surface(big.get_size(), pygame.SRCALPHA)
        rim = (c[0] - RR * 0.955, c[1] - RR * 0.955, RR * 1.91, RR * 1.91)
        pygame.draw.arc(glow, (255, 240, 220, 60), rim, math.pi * 0.55,
                        math.pi * 0.95, max(2, int(RR * 0.035)))
        big.blit(glow, (0, 0))
        return pygame.transform.smoothscale(big, (d, d))

    def _build_rotor(self):
        d, R = self.d, self.R
        ss = 2
        size = int(self.rotor_r * 2 * ss) + 2
        big = pygame.Surface((size, size), pygame.SRCALPHA)
        c = (size / 2, size / 2)
        RR = R * ss
        n = len(L.WHEEL_ORDER)
        seg = math.tau / n

        def ring_poly(a0, a1, r0, r1, steps=4):
            pts = []
            for k in range(steps + 1):
                a = a0 + (a1 - a0) * k / steps
                pts.append((c[0] + math.sin(a) * r1, c[1] - math.cos(a) * r1))
            for k in range(steps, -1, -1):
                a = a0 + (a1 - a0) * k / steps
                pts.append((c[0] + math.sin(a) * r0, c[1] - math.cos(a) * r0))
            return pts

        num_font = ui.font(max(7, int(R * 0.085)), bold=True)
        for i, num in enumerate(L.WHEEL_ORDER):
            a0, a1 = i * seg - seg / 2, i * seg + seg / 2
            col = number_color(num)
            pygame.draw.polygon(big, col, ring_poly(a0, a1, RR * 0.62, RR * 0.78))
            pygame.draw.polygon(big, ui.mix(col, (0, 0, 0), 0.45),
                                ring_poly(a0, a1, RR * 0.50, RR * 0.62))
            img = num_font.render(str(num), True, (245, 240, 230))
            img = pygame.transform.smoothscale(
                img, (img.get_width() * ss, img.get_height() * ss))
            img = pygame.transform.rotate(img, -math.degrees(i * seg))
            px = c[0] + math.sin(i * seg) * RR * 0.70
            py = c[1] - math.cos(i * seg) * RR * 0.70
            big.blit(img, img.get_rect(center=(px, py)))
        # Fächer-Stege (Frets) + Ringe
        for i in range(n):
            a = i * seg - seg / 2
            p0 = (c[0] + math.sin(a) * RR * 0.50, c[1] - math.cos(a) * RR * 0.50)
            p1 = (c[0] + math.sin(a) * RR * 0.78, c[1] - math.cos(a) * RR * 0.78)
            pygame.draw.line(big, COL_GOLD, p0, p1, max(1, ss))
        pygame.draw.circle(big, COL_GOLD, c, int(RR * 0.78), max(1, ss + 1))
        pygame.draw.circle(big, COL_GOLD, c, int(RR * 0.62), max(1, ss))
        pygame.draw.circle(big, COL_GOLD_D, c, int(RR * 0.50), max(1, ss + 1))
        # Konus mit Verlauf + Drehkreuz
        for k in range(16):
            f = k / 15
            pygame.draw.circle(big, ui.mix((120, 72, 38), (196, 142, 80), f),
                               c, int(RR * (0.49 - 0.018 * k)))
        for k in range(4):
            a = k * math.tau / 4
            p1 = (c[0] + math.sin(a) * RR * 0.34, c[1] - math.cos(a) * RR * 0.34)
            pygame.draw.line(big, COL_GOLD_D, c, p1, max(2, int(RR * 0.05)))
            pygame.draw.line(big, COL_GOLD, c, p1, max(1, int(RR * 0.025)))
            pygame.draw.circle(big, COL_GOLD, (int(p1[0]), int(p1[1])),
                               max(2, int(RR * 0.035)))
        pygame.draw.circle(big, COL_GOLD_D, c, int(RR * 0.09))
        pygame.draw.circle(big, (255, 236, 170), c, int(RR * 0.06))
        out = size // ss
        return pygame.transform.smoothscale(big, (out, out))

    def pocket_angle(self, number):
        """Winkel (Bogenmaß, im Uhrzeigersinn ab 12 Uhr) eines Fachs im Rotor."""
        return L.WHEEL_ORDER.index(number) * math.tau / len(L.WHEEL_ORDER)

    def draw(self, surface, center, rotation, ball=None, trail=()):
        """Kessel zeichnen. rotation = Rotor-Winkel (rad, im Uhrzeigersinn);
        ball = (winkel, radius) in Bildschirm-Koordinaten oder None."""
        cx, cy = int(center[0]), int(center[1])
        surface.blit(self.bowl, self.bowl.get_rect(center=(cx, cy)))
        rot = pygame.transform.rotate(self.rotor, -math.degrees(rotation))
        surface.blit(rot, rot.get_rect(center=(cx, cy)))
        if ball is None:
            return
        br = max(3, int(self.R * 0.045))
        for i, (a, r) in enumerate(trail):
            alpha = 40 + 30 * i
            tx = cx + math.sin(a) * r
            ty = cy - math.cos(a) * r
            dot = _ball_dot(br, alpha)
            surface.blit(dot, dot.get_rect(center=(int(tx), int(ty))))
        a, r = ball
        bx = cx + math.sin(a) * r
        by = cy - math.cos(a) * r
        pygame.draw.circle(surface, (0, 0, 0), (int(bx) + 1, int(by) + 2), br)
        pygame.draw.circle(surface, (236, 238, 244), (int(bx), int(by)), br)
        pygame.draw.circle(surface, (255, 255, 255),
                           (int(bx - br * 0.35), int(by - br * 0.35)),
                           max(1, br // 3))


def _ball_dot(r, alpha):
    def build():
        s = pygame.Surface((2 * r + 2, 2 * r + 2), pygame.SRCALPHA)
        pygame.draw.circle(s, (236, 238, 244, alpha), (r + 1, r + 1), r)
        return s
    return _cached(("ball", r, alpha), build)


# ============================================================= Slot-Symbole
def symbol_surface(sym, size, blur=False):
    """Symbol als gecachte Fläche size x size (blur = Bewegungsunschärfe)."""
    key = ("sym", sym, size, blur)

    def build():
        if blur:
            sharp = symbol_surface(sym, size)
            out = pygame.Surface((size, size), pygame.SRCALPHA)
            for dy, a in ((-size // 7, 70), (size // 7, 70), (0, 150)):
                layer = sharp.copy()
                layer.set_alpha(a)
                out.blit(layer, (0, dy))
            return out
        ss = 2
        big = pygame.Surface((size * ss, size * ss), pygame.SRCALPHA)
        _SYMBOL_DRAW[sym](big, size * ss)
        return pygame.transform.smoothscale(big, (size, size))
    return _cached(key, build)


def _shade(col, f):
    return ui.mix(col, (255, 255, 255) if f > 0 else (0, 0, 0), abs(f))


def _sym_cherry(s, n):
    red = (214, 30, 52)
    stem = (70, 150, 60)
    pygame.draw.lines(s, stem, False, [(n * 0.33, n * 0.62), (n * 0.46, n * 0.34),
                                       (n * 0.58, n * 0.18)], max(2, n // 28))
    pygame.draw.lines(s, stem, False, [(n * 0.68, n * 0.66), (n * 0.64, n * 0.38),
                                       (n * 0.58, n * 0.18)], max(2, n // 28))
    pygame.draw.ellipse(s, (60, 170, 70), (n * 0.56, n * 0.12, n * 0.26, n * 0.13))
    for cx, cy in ((0.32, 0.68), (0.66, 0.70)):
        r = n * 0.17
        pygame.draw.circle(s, _shade(red, -0.35), (n * cx + n * 0.015, n * cy + n * 0.02), r)
        pygame.draw.circle(s, red, (n * cx, n * cy), r)
        pygame.draw.circle(s, _shade(red, 0.55), (n * cx - r * 0.35, n * cy - r * 0.35),
                           r * 0.28)


def _sym_lemon(s, n):
    y = (250, 214, 50)
    pts = [(n * 0.14, n * 0.52), (n * 0.30, n * 0.30), (n * 0.52, n * 0.24),
           (n * 0.74, n * 0.32), (n * 0.88, n * 0.50), (n * 0.74, n * 0.70),
           (n * 0.52, n * 0.77), (n * 0.30, n * 0.70)]
    pygame.draw.polygon(s, _shade(y, -0.3), [(x + n * 0.015, yy + n * 0.02) for x, yy in pts])
    pygame.draw.polygon(s, y, pts)
    pygame.draw.ellipse(s, y, (n * 0.20, n * 0.26, n * 0.62, n * 0.50))
    pygame.draw.ellipse(s, _shade(y, 0.55), (n * 0.32, n * 0.34, n * 0.22, n * 0.10))
    pygame.draw.circle(s, _shade(y, -0.35), (n * 0.88, n * 0.50), n * 0.03)


def _sym_grape(s, n):
    p = (136, 58, 176)
    pygame.draw.line(s, (90, 140, 60), (n * 0.50, n * 0.12), (n * 0.52, n * 0.30),
                     max(2, n // 26))
    pygame.draw.ellipse(s, (80, 170, 80), (n * 0.52, n * 0.10, n * 0.28, n * 0.15))
    rows = ((0.30, (0.32, 0.50, 0.68)), (0.47, (0.40, 0.60)),
            (0.47, (0.22, 0.78)), (0.63, (0.32, 0.50, 0.68)), (0.79, (0.41, 0.59)))
    r = n * 0.10
    for cy, xs in rows:
        for cx in xs:
            if cy == 0.47 and cx in (0.22, 0.78):
                continue
            pygame.draw.circle(s, _shade(p, -0.35), (n * cx + n * 0.01, n * cy + n * 0.015), r)
            pygame.draw.circle(s, p, (n * cx, n * cy), r)
            pygame.draw.circle(s, _shade(p, 0.5), (n * cx - r * 0.35, n * cy - r * 0.35),
                               r * 0.28)
    for cx in (0.41, 0.59):
        pygame.draw.circle(s, p, (n * cx, n * 0.47), r)
        pygame.draw.circle(s, _shade(p, 0.5), (n * cx - r * 0.35, n * 0.47 - r * 0.35),
                           r * 0.28)


def _sym_clover(s, n):
    g = (46, 176, 84)
    pygame.draw.lines(s, _shade(g, -0.3), False, [(n * 0.50, n * 0.52), (n * 0.56, n * 0.76),
                                                  (n * 0.66, n * 0.88)], max(3, n // 18))
    for a in range(4):
        ang = a * math.tau / 4 + math.tau / 8
        cx = n * 0.5 + math.cos(ang) * n * 0.17
        cy = n * 0.46 + math.sin(ang) * n * 0.17
        for da in (-0.55, 0.55):
            px = cx + math.cos(ang + da) * n * 0.07
            py = cy + math.sin(ang + da) * n * 0.07
            pygame.draw.circle(s, _shade(g, -0.25), (px + n * 0.01, py + n * 0.015), n * 0.12)
    for a in range(4):
        ang = a * math.tau / 4 + math.tau / 8
        cx = n * 0.5 + math.cos(ang) * n * 0.17
        cy = n * 0.46 + math.sin(ang) * n * 0.17
        for da in (-0.55, 0.55):
            px = cx + math.cos(ang + da) * n * 0.07
            py = cy + math.sin(ang + da) * n * 0.07
            pygame.draw.circle(s, g, (px, py), n * 0.12)
    pygame.draw.circle(s, _shade(g, 0.35), (n * 0.5, n * 0.46), n * 0.05)


def _sym_bell(s, n):
    gold = (246, 190, 50)
    body = [(n * 0.50, n * 0.14), (n * 0.66, n * 0.22), (n * 0.72, n * 0.44),
            (n * 0.78, n * 0.64), (n * 0.88, n * 0.74), (n * 0.12, n * 0.74),
            (n * 0.22, n * 0.64), (n * 0.28, n * 0.44), (n * 0.34, n * 0.22)]
    pygame.draw.polygon(s, _shade(gold, -0.35), [(x + n * 0.015, y + n * 0.02) for x, y in body])
    pygame.draw.polygon(s, gold, body)
    pygame.draw.circle(s, gold, (n * 0.50, n * 0.30), n * 0.17)
    pygame.draw.rect(s, _shade(gold, -0.25), (n * 0.10, n * 0.72, n * 0.80, n * 0.07),
                     border_radius=int(n * 0.03))
    pygame.draw.circle(s, _shade(gold, -0.45), (n * 0.50, n * 0.84), n * 0.07)
    pygame.draw.circle(s, _shade(gold, -0.2), (n * 0.50, n * 0.12), n * 0.05)
    pygame.draw.ellipse(s, _shade(gold, 0.6), (n * 0.34, n * 0.24, n * 0.09, n * 0.30))


def _sym_gem(s, n):
    c = (70, 210, 240)
    top = n * 0.26
    mid = n * 0.42
    pts = [(n * 0.28, top), (n * 0.72, top), (n * 0.90, mid), (n * 0.50, n * 0.88),
           (n * 0.10, mid)]
    pygame.draw.polygon(s, _shade(c, -0.45), [(x + n * 0.015, y + n * 0.02) for x, y in pts])
    pygame.draw.polygon(s, c, pts)
    pygame.draw.polygon(s, _shade(c, 0.45), [(n * 0.28, top), (n * 0.50, top), (n * 0.38, mid),
                                             (n * 0.10, mid)])
    pygame.draw.polygon(s, _shade(c, 0.2), [(n * 0.50, top), (n * 0.72, top), (n * 0.62, mid),
                                            (n * 0.38, mid)])
    pygame.draw.polygon(s, _shade(c, -0.2), [(n * 0.62, mid), (n * 0.90, mid), (n * 0.50, n * 0.88)])
    pygame.draw.polygon(s, _shade(c, 0.15), [(n * 0.38, mid), (n * 0.62, mid), (n * 0.50, n * 0.88)])
    pygame.draw.polygon(s, (255, 255, 255), pts, max(1, n // 60))
    pygame.draw.circle(s, (255, 255, 255), (n * 0.30, n * 0.31), n * 0.03)


def _sym_seven(s, n):
    fnt = ui.font(int(n * 0.78), bold=True)
    red = (226, 36, 48)
    img = fnt.render("7", True, red)
    rect = img.get_rect(center=(n * 0.5, n * 0.52))
    edge = fnt.render("7", True, (120, 12, 20))
    gold = fnt.render("7", True, (255, 214, 90))
    o = max(2, n // 40)
    for dx, dy in ((-o, 0), (o, 0), (0, -o), (0, o), (-o, -o), (o, o), (-o, o), (o, -o)):
        s.blit(gold, rect.move(dx, dy))
    s.blit(edge, rect.move(o, o + o // 2))
    s.blit(img, rect)
    shine = fnt.render("7", True, (255, 150, 150))
    clip = pygame.Surface(shine.get_size(), pygame.SRCALPHA)
    clip.blit(shine, (0, 0))
    clip.fill((255, 255, 255, 0), (0, clip.get_height() // 2, clip.get_width(),
                                   clip.get_height()), special_flags=pygame.BLEND_RGBA_MULT)
    clip.set_alpha(90)
    s.blit(clip, rect.move(-o // 2, -o // 2))


def _llama_head(s, cx, cy, n, wool, dark):
    """Lama-Kopf (Grundform für Wild und Münzprägung)."""
    # Ohren
    for side in (-1, 1):
        ear = [(cx + side * n * 0.10, cy - n * 0.16), (cx + side * n * 0.20, cy - n * 0.44),
               (cx + side * n * 0.26, cy - n * 0.14)]
        pygame.draw.polygon(s, wool, ear)
        if dark is not None:
            inner = [(cx + side * n * 0.14, cy - n * 0.17), (cx + side * n * 0.20, cy - n * 0.36),
                     (cx + side * n * 0.23, cy - n * 0.16)]
            pygame.draw.polygon(s, dark, inner)
    # Kopf + Wuschel
    pygame.draw.ellipse(s, wool, (cx - n * 0.21, cy - n * 0.22, n * 0.42, n * 0.40))
    for k in range(5):
        pygame.draw.circle(s, wool, (cx - n * 0.14 + k * n * 0.07, cy - n * 0.22), n * 0.06)
    # Schnauze
    pygame.draw.ellipse(s, wool, (cx - n * 0.15, cy + n * 0.02, n * 0.30, n * 0.24))


def _sym_lama(s, n):
    wool = (250, 244, 230)
    shadow = (206, 190, 168)
    dark = (60, 44, 40)
    pink = (240, 150, 160)
    # Sonnen-Schein dahinter (Wild!)
    glow = (255, 200, 80)
    for k in range(12):
        a = k * math.tau / 12
        pts = [(n * 0.5, n * 0.44),
               (n * 0.5 + math.cos(a - 0.12) * n * 0.48, n * 0.44 + math.sin(a - 0.12) * n * 0.48),
               (n * 0.5 + math.cos(a + 0.12) * n * 0.48, n * 0.44 + math.sin(a + 0.12) * n * 0.48)]
        pygame.draw.polygon(s, ui.mix(glow, (255, 255, 255), 0.25), pts)
    pygame.draw.circle(s, glow, (n * 0.5, n * 0.44), n * 0.32)
    cx, cy = n * 0.5, n * 0.44
    # Hals
    pygame.draw.rect(s, shadow, (cx - n * 0.13, cy + n * 0.12, n * 0.26, n * 0.30),
                     border_radius=int(n * 0.05))
    _llama_head(s, cx + n * 0.012, cy + n * 0.018, n, shadow, None)
    _llama_head(s, cx, cy, n, wool, pink)
    # Gesicht
    for side in (-1, 1):
        ex, ey = cx + side * n * 0.09, cy - n * 0.03
        pygame.draw.circle(s, dark, (ex, ey), n * 0.035)
        pygame.draw.circle(s, (255, 255, 255), (ex - n * 0.01, ey - n * 0.012), n * 0.011)
        pygame.draw.circle(s, pink, (cx + side * n * 0.15, cy + n * 0.07), n * 0.03)
    pygame.draw.circle(s, dark, (cx - n * 0.035, cy + n * 0.10), n * 0.013)
    pygame.draw.circle(s, dark, (cx + n * 0.035, cy + n * 0.10), n * 0.013)
    pygame.draw.arc(s, dark, (cx - n * 0.05, cy + n * 0.10, n * 0.10, n * 0.07),
                    math.pi * 1.1, math.pi * 1.9, max(1, n // 60))
    # Wild-Banderole
    band = pygame.Rect(0, 0, int(n * 0.70), int(n * 0.20))
    band.center = (int(n * 0.5), int(n * 0.84))
    pygame.draw.rect(s, (150, 30, 90), band.move(0, n // 60), border_radius=int(n * 0.06))
    pygame.draw.rect(s, (214, 51, 108), band, border_radius=int(n * 0.06))
    pygame.draw.rect(s, (255, 214, 90), band, max(1, n // 60), border_radius=int(n * 0.06))
    fnt = ui.font(int(n * 0.15), bold=True)
    img = fnt.render("WILD", True, (255, 244, 220))
    s.blit(img, img.get_rect(center=band.center))


def _sym_coin(s, n):
    gold = (246, 196, 60)
    c = (n * 0.5, n * 0.5)
    pygame.draw.circle(s, _shade(gold, -0.45), (c[0] + n * 0.02, c[1] + n * 0.03), n * 0.40)
    pygame.draw.circle(s, _shade(gold, -0.15), c, n * 0.40)
    pygame.draw.circle(s, gold, c, n * 0.34)
    pygame.draw.circle(s, _shade(gold, -0.25), c, n * 0.34, max(2, n // 45))
    for k in range(24):
        a = k * math.tau / 24
        p0 = (c[0] + math.cos(a) * n * 0.37, c[1] + math.sin(a) * n * 0.37)
        p1 = (c[0] + math.cos(a) * n * 0.40, c[1] + math.sin(a) * n * 0.40)
        pygame.draw.line(s, _shade(gold, -0.35), p0, p1, max(1, n // 80))
    # geprägter Lama-Kopf
    _llama_head(s, c[0] + n * 0.01, c[1] + n * 0.05, n * 0.72, _shade(gold, -0.30), None)
    _llama_head(s, c[0], c[1] + n * 0.04, n * 0.72, _shade(gold, 0.30), None)
    pygame.draw.circle(s, _shade(gold, -0.5), (c[0] - n * 0.06, c[1] + n * 0.02), n * 0.02)
    pygame.draw.circle(s, _shade(gold, -0.5), (c[0] + n * 0.06, c[1] + n * 0.02), n * 0.02)
    # Glanz
    pygame.draw.arc(s, (255, 250, 220), (c[0] - n * 0.30, c[1] - n * 0.30, n * 0.60, n * 0.60),
                    math.pi * 0.55, math.pi * 0.95, max(2, n // 40))
    for sx, sy, r in ((0.80, 0.18, 0.06), (0.18, 0.80, 0.04)):
        x, y = n * sx, n * sy
        pts = [(x, y - n * r), (x + n * r * 0.3, y - n * r * 0.3), (x + n * r, y),
               (x + n * r * 0.3, y + n * r * 0.3), (x, y + n * r), (x - n * r * 0.3, y + n * r * 0.3),
               (x - n * r, y), (x - n * r * 0.3, y - n * r * 0.3)]
        pygame.draw.polygon(s, (255, 255, 230), pts)


_SYMBOL_DRAW = {
    "cherry": _sym_cherry, "lemon": _sym_lemon, "grape": _sym_grape,
    "clover": _sym_clover, "bell": _sym_bell, "gem": _sym_gem,
    "seven": _sym_seven, "lama": _sym_lama, "coin": _sym_coin,
}


# ============================================================= Walzen
def reel_shade(w, h):
    """Zylinder-Schattierung über einer Walze (oben/unten dunkler)."""
    def build():
        s = pygame.Surface((w, h), pygame.SRCALPHA)
        band = max(4, h // 4)
        for y in range(band):
            a = int(150 * (1 - y / band) ** 1.6)
            s.fill((0, 0, 0, a), (0, y, w, 1))
            s.fill((0, 0, 0, a), (0, h - 1 - y, w, 1))
        return s
    return _cached(("shade", w, h), build)


def draw_reel(surface, rect, strip, pos, cell, speed=0.0, dim_rows=None,
              pop=None):
    """Zeichnet eine Walze in rect.

    pos     : gebrochene Streifen-Position des obersten sichtbaren Felds
              (kleiner werdend = Symbole wandern nach unten),
    cell    : Zellhöhe (= Symbolgröße),
    speed   : Felder pro Sekunde (ab ~8 mit Bewegungsunschärfe),
    dim_rows: Reihen, die abgedunkelt werden (Gewinnlinien-Anzeige),
    pop     : {reihe: skalierung} - Symbole, die gerade "hüpfen".
    """
    r = pygame.Rect(rect)
    n = len(strip)
    base = math.floor(pos)
    frac = pos - base
    blur = speed >= 8.0
    old_clip = surface.get_clip()
    surface.set_clip(r.clip(old_clip) if old_clip else r)
    pad = max(2, cell // 14)
    size = cell - 2 * pad
    for i in range(-1, ROWS_PLUS):
        sym = strip[(base + i) % n]
        y = r.y + int(round((i - frac) * cell))
        row = i if frac < 1e-6 else None
        scale = pop.get(row, 1.0) if (pop and row is not None) else 1.0
        if scale != 1.0:
            sz = max(4, int(size * scale))
            img = pygame.transform.smoothscale(symbol_surface(sym, size), (sz, sz))
            surface.blit(img, img.get_rect(center=(r.centerx, y + cell // 2)))
        else:
            img = symbol_surface(sym, size, blur)
            surface.blit(img, (r.centerx - size // 2, y + pad))
        if dim_rows is not None and row is not None and row in dim_rows:
            dim = _dim_cell(r.w, cell)
            surface.blit(dim, (r.x, y))
    surface.blit(reel_shade(r.w, r.h), r.topleft)
    surface.set_clip(old_clip)


ROWS_PLUS = L.ROWS + 1


def _dim_cell(w, h):
    def build():
        s = pygame.Surface((w, h), pygame.SRCALPHA)
        s.fill((0, 0, 0, 120))
        return s
    return _cached(("dim", w, h), build)
