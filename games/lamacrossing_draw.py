# -*- coding: utf-8 -*-
"""
lamacrossing_draw.py
====================
Zeichen-Bausteine von Lama Crossing (nur pygame-Primitive, keine Bilddateien):

- ``llama_sprite``  : das Lama von der Seite (stehen / springen / fallen) mit
                      Schal in der Akzentfarbe, 2-fach überabgetastet und weich
                      verkleinert, gecacht je Größe und Pose,
- ``flat_llama``    : das von einem Hindernis plattgedrückte Lama (gecacht),
- ``background``    : Himmel + Ferne je Welt (Hängebrücke, Fluss, Eissee) bzw.
                      Asphalt von oben (Straße), gecacht je Auflösung,
- ``far_layer``     : kachelbare Silhouetten-Ebene für den Parallax-Effekt
                      (nicht für die Straße),
- ``draw_tile``     : Planke, Stein, Eisscholle bzw. Zebrastreifen eines Schritts,
- ``draw_bank``     : Start- und Zielufer (Fels, Wiese, Schnee) bzw. Gehweg mit
                      Bordstein, Pflaster, Hecke und Rasen (Straße),
- ``draw_lanes``    : gestrichelte Fahrbahnmarkierungen der Straße,
- ``draw_ropes``    : Seile der Hängebrücke,
- ``obstacle_len``  : senkrechte Länge eines Hindernisses (Treffer-Test),
- ``draw_obstacle`` : bewegte Hindernisse: Autos + Lieferwagen von oben
                      (Straße, 2-fach überabgetastet, gecacht), Felsbrocken
                      (Brücke), Treibholz (Fluss), rutschender Eisblock (Eissee),
- ``draw_barrier``  : rot-weiße Absperrung über einer geschafften Spur,
- ``draw_tire_marks``: Reifenspuren über dem plattgedrückten Lama,
- ``world_icon``    : kleines Piktogramm je Welt für die Welt-Knöpfe.

Alle Formen hängen nur von Größe, Index und Zeit ab (kein Zufall) - damit
sind Python- und Rust-Fassung pixelgleich.
"""

import math

import pygame

import ui

_cache = {}

WOOL = (250, 244, 230)
WOOL_D = (214, 200, 178)
DARK = (60, 44, 40)
HOOF = (82, 60, 52)
PINK = (240, 150, 160)


def _cached(key, build):
    surf = _cache.get(key)
    if surf is None:
        if len(_cache) > 120:
            _cache.clear()
        surf = build()
        _cache[key] = surf
    return surf


def clear_cache():
    _cache.clear()


def _gradient(surface, rect, top, bottom):
    """Senkrechter Farbverlauf in rect."""
    r = pygame.Rect(rect)
    h = max(1, r.h - 1)
    for y in range(r.h):
        surface.fill(ui.mix(top, bottom, y / h), (r.x, r.y + y, r.w, 1))


def _hash(i):
    """Ganzzahl-Streuwert 0..2^32-1 (deterministisch, ohne Zufall/sin)."""
    v = (i * 2654435761 + 40503) % 4294967296
    v = ((v ^ (v >> 15)) * 2246822519) % 4294967296
    return v ^ (v >> 13)


def _rrect(s, col, x, y, w, h, r):
    """Abgerundetes Rechteck mit expliziten Ganzzahlen."""
    pygame.draw.rect(s, col, (int(x), int(y), max(1, int(w)), max(1, int(h))),
                     border_radius=max(0, int(r)))


# ============================================================= Lama
def _leg(s, x0, y0, x1, y1, w, col):
    pygame.draw.line(s, col, (x0, y0), (x1, y1), w)
    pygame.draw.circle(s, HOOF, (x1, y1), w * 0.62)


def _draw_llama(s, n, pose, scarf):
    """Lama in ein n x n großes Feld, Blick nach rechts, Füße unten Mitte."""
    lw = max(2, int(n * 0.055))
    hip, sh = (n * 0.34, n * 0.62), (n * 0.62, n * 0.62)
    if pose == "hop":
        legs = (((-0.02, 0.0), (-0.12, 0.22)), ((0.04, 0.0), (-0.04, 0.24)),
                ((-0.02, 0.0), (0.10, 0.20)), ((0.04, 0.0), (0.16, 0.18)))
    elif pose == "fall":
        legs = (((-0.02, 0.0), (-0.14, -0.10)), ((0.04, 0.0), (-0.08, 0.16)),
                ((-0.02, 0.0), (0.14, -0.08)), ((0.04, 0.0), (0.12, 0.16)))
    else:
        legs = (((-0.02, 0.0), (-0.03, 0.31)), ((0.04, 0.0), (0.05, 0.31)),
                ((-0.02, 0.0), (-0.01, 0.31)), ((0.04, 0.0), (0.06, 0.31)))
    # hintere (ferne) Beine dunkler, zuerst
    for i in (1, 3):
        base = hip if i < 2 else sh
        (ax, ay), (bx, by) = legs[i]
        _leg(s, base[0] + ax * n, base[1] + ay * n, base[0] + bx * n,
             base[1] + by * n, lw, WOOL_D)
    # Schwanz + Rumpf
    pygame.draw.circle(s, WOOL_D, (n * 0.215, n * 0.47), n * 0.055)
    pygame.draw.ellipse(s, WOOL_D, (n * 0.22, n * 0.44, n * 0.54, n * 0.27))
    pygame.draw.ellipse(s, WOOL, (n * 0.22, n * 0.42, n * 0.54, n * 0.27))
    for i in range(6):
        pygame.draw.circle(s, WOOL, (n * (0.27 + i * 0.085), n * 0.44), n * 0.055)
    # Hals + Kopf
    pygame.draw.rect(s, WOOL, (n * 0.60, n * 0.17, n * 0.12, n * 0.38),
                     border_radius=int(n * 0.05))
    pygame.draw.polygon(s, WOOL, [(n * 0.63, n * 0.12), (n * 0.655, n * 0.01),
                                  (n * 0.69, n * 0.11)])
    pygame.draw.polygon(s, PINK, [(n * 0.645, n * 0.11), (n * 0.66, n * 0.04),
                                  (n * 0.68, n * 0.11)])
    pygame.draw.ellipse(s, WOOL, (n * 0.59, n * 0.09, n * 0.24, n * 0.17))
    pygame.draw.ellipse(s, WOOL, (n * 0.72, n * 0.14, n * 0.17, n * 0.12))
    for i in range(3):
        pygame.draw.circle(s, WOOL, (n * (0.62 + i * 0.045), n * 0.10), n * 0.035)
    # Gesicht
    eye = (n * 0.765, n * 0.155)
    if pose == "fall":
        d = n * 0.022
        pygame.draw.line(s, DARK, (eye[0] - d, eye[1] - d), (eye[0] + d, eye[1] + d), max(1, lw // 2))
        pygame.draw.line(s, DARK, (eye[0] - d, eye[1] + d), (eye[0] + d, eye[1] - d), max(1, lw // 2))
        pygame.draw.circle(s, DARK, (n * 0.86, n * 0.225), n * 0.022)
    else:
        pygame.draw.circle(s, DARK, eye, n * 0.024)
        pygame.draw.circle(s, (255, 255, 255), (eye[0] - n * 0.007, eye[1] - n * 0.008), n * 0.008)
        pygame.draw.arc(s, DARK, (n * 0.80, n * 0.19, n * 0.07, n * 0.05),
                        math.pi * 1.15, math.pi * 1.85, max(1, lw // 3))
    pygame.draw.circle(s, PINK, (n * 0.79, n * 0.205), n * 0.022)
    pygame.draw.circle(s, DARK, (n * 0.88, n * 0.18), n * 0.012)
    # vordere Beine
    for i in (0, 2):
        base = hip if i < 2 else sh
        (ax, ay), (bx, by) = legs[i]
        _leg(s, base[0] + ax * n, base[1] + ay * n, base[0] + bx * n,
             base[1] + by * n, lw, WOOL)
    # Schal in der Akzentfarbe (Knoten + flatterndes Ende)
    dark_scarf = ui.mix(scarf, (0, 0, 0), 0.35)
    pygame.draw.rect(s, scarf, (n * 0.585, n * 0.33, n * 0.15, n * 0.07),
                     border_radius=int(n * 0.03))
    end = [(n * 0.61, n * 0.37), (n * 0.66, n * 0.38), (n * 0.56, n * 0.52), (n * 0.50, n * 0.49)]
    if pose == "hop":
        end = [(n * 0.61, n * 0.36), (n * 0.66, n * 0.37), (n * 0.48, n * 0.42), (n * 0.46, n * 0.37)]
    pygame.draw.polygon(s, dark_scarf, end)
    pygame.draw.line(s, dark_scarf, (n * 0.60, n * 0.355), (n * 0.73, n * 0.355), max(1, lw // 3))


def llama_sprite(size, pose, scarf):
    """Lama als gecachte Fläche size x size (Füße an der Unterkante Mitte)."""
    key = ("llama", size, pose, tuple(scarf))

    def build():
        big = pygame.Surface((size * 2, size * 2), pygame.SRCALPHA)
        _draw_llama(big, size * 2, pose, scarf)
        return pygame.transform.smoothscale(big, (size, size))
    return _cached(key, build)


def flat_llama(size, scarf):
    """Plattgedrücktes Lama (Pose "fall" gestaucht), gecacht.

    Fläche int(size * 1.6) x max(4, int(size * 0.34)); das Lama selbst liegt
    etwas rechts der Mitte (wie im Sprite) auf einem dunklen Abdruck, damit
    es sich auch auf dem weißen Zebrastreifen abhebt.
    """
    key = ("flat", size, tuple(scarf))

    def build():
        fw, fh = max(4, int(size * 1.6)), max(4, int(size * 0.34))
        squashed = pygame.transform.smoothscale(llama_sprite(size, "fall", scarf), (fw, fh))
        flat = pygame.Surface((fw, fh), pygame.SRCALPHA)
        # dunkler Abdruck unter dem Lama
        pygame.draw.ellipse(flat, (24, 20, 26, 50), (int(fw * 0.13), 0, int(fw * 0.82), fh))
        pygame.draw.ellipse(flat, (24, 20, 26, 110), (int(fw * 0.17), int(fh * 0.12), int(fw * 0.74),
                                                      max(1, int(fh * 0.80))))
        flat.blit(squashed, (0, 0))
        # Schalzipfel, flach neben dem Hals ausgebreitet
        dark = ui.mix(scarf, (0, 0, 0), 0.35)
        pygame.draw.polygon(flat, dark, [(fw * 0.57, fh * 0.42), (fw * 0.66, fh * 0.42),
                                         (fw * 0.56, fh * 0.98), (fw * 0.47, fh * 0.90)])
        pygame.draw.polygon(flat, scarf, [(fw * 0.58, fh * 0.42), (fw * 0.645, fh * 0.42),
                                          (fw * 0.555, fh * 0.86), (fw * 0.495, fh * 0.82)])
        return flat
    return _cached(key, build)


# ============================================================= Welten
SKY = {
    "bridge": ((58, 82, 140), (238, 170, 120)),
    "river": ((120, 186, 236), (222, 238, 250)),
    "ice": ((150, 182, 214), (232, 240, 248)),
}
FAR = {
    "bridge": ((176, 104, 84), (132, 74, 66)),
    "river": ((70, 128, 84), (46, 98, 66)),
    "ice": ((214, 228, 242), (168, 192, 220)),
}


def _ridge(x, w, h, seed):
    """Weiche Hügellinie (periodisch über w, für kachelbare Ebenen)."""
    a = math.tau * x / w
    return h * (0.5 + 0.22 * math.sin(a * 2 + seed) + 0.16 * math.sin(a * 5 + seed * 2.3)
                + 0.08 * math.sin(a * 11 + seed * 0.7))


ASPHALT = ((52, 54, 62), (66, 68, 76), (44, 46, 54))     # oben, Pfad, unten
ASPHALT_GRAIN = ((80, 82, 90), (38, 40, 46), (58, 60, 68))
LANE = (230, 230, 222)
ZEBRA = (238, 238, 230)
ZEBRA_D = (196, 196, 190)
CURB, CURB_L, CURB_D = (208, 208, 202), (236, 236, 230), (160, 160, 156)
GUTTER = (34, 35, 41)
PAVE, PAVE_2, PAVE_D = (184, 176, 164), (172, 164, 152), (150, 142, 132)
HEDGE, HEDGE_D, HEDGE_L = (58, 128, 62), (38, 92, 46), (92, 162, 82)
GRASS, GRASS_D = (104, 170, 88), (94, 158, 80)


def _road_bg(w, h, path_y):
    """Straße von oben: dunkler Asphalt mit leichtem Verlauf und feiner Körnung."""
    s = pygame.Surface((w, h))
    top, mid, bot = ASPHALT
    py = max(1, min(h - 1, int(path_y)))
    _gradient(s, (0, 0, w, py), top, mid)
    _gradient(s, (0, py, w, h - py), mid, bot)
    dot = 2 if w >= 1200 else 1
    for i in range(w * h // 650):
        v = _hash(i)
        s.fill(ASPHALT_GRAIN[(v >> 24) % 3], ((v % w), (v // w) % h, dot, dot))
    return s


def background(world, w, h, path_y):
    """Himmel, Sonne/Licht und Untergrund unterhalb des Pfads (statisch)."""
    def build():
        if world == "road":
            return _road_bg(w, h, path_y)
        s = pygame.Surface((w, h))
        top, bot = SKY[world]
        _gradient(s, (0, 0, w, path_y), top, bot)
        if world == "bridge":
            sun = (int(w * 0.78), int(path_y * 0.42))
            for i in range(5, 0, -1):
                r = int(path_y * (0.06 + 0.035 * i))
                glow = pygame.Surface((r * 2, r * 2), pygame.SRCALPHA)
                pygame.draw.circle(glow, (255, 214, 150, 26), (r, r), r)
                s.blit(glow, (sun[0] - r, sun[1] - r))
            pygame.draw.circle(s, (255, 226, 170), sun, int(path_y * 0.07))
            _gradient(s, (0, path_y, w, h - path_y), (96, 54, 48), (22, 14, 22))
            # Felsnadeln in der Tiefe der Schlucht
            for i in range(7):
                x = w * (0.07 + 0.145 * i) + math.sin(i * 2.3) * w * 0.03
                top = path_y + (h - path_y) * (0.30 + 0.30 * abs(math.sin(i * 1.7)))
                bw = w * (0.035 + 0.015 * abs(math.cos(i * 1.3)))
                pygame.draw.polygon(s, (66, 38, 40), [(x - bw, h), (x - bw * 0.45, top + bw * 0.4),
                                                      (x, top), (x + bw * 0.5, top + bw * 0.5),
                                                      (x + bw, h)])
                pygame.draw.line(s, (112, 66, 58), (x - bw * 0.45, top + bw * 0.4), (x, top), 2)
        elif world == "river":
            _gradient(s, (0, path_y - int(h * 0.10), w, h - path_y + int(h * 0.10)),
                      (78, 156, 200), (26, 78, 124))
        else:
            _gradient(s, (0, path_y - int(h * 0.08), w, h - path_y + int(h * 0.08)),
                      (34, 78, 108), (10, 30, 52))
        return s
    return _cached(("bg", world, w, h, path_y), build)


def far_layer(world, w, h, path_y):
    """Kachelbare Ferne (Tafelberge / Wald / Schneegipfel), Breite = w."""
    def build():
        s = pygame.Surface((w, path_y), pygame.SRCALPHA)
        if world not in FAR:
            return s          # Straße: keine Ferne (bleibt durchsichtig)
        c1, c2 = FAR[world]
        base = path_y - (int(h * 0.10) if world == "river" else
                         int(h * 0.08) if world == "ice" else 0)
        for layer, (col, amp, seed) in enumerate(((c1, 0.42, 1.0), (c2, 0.26, 2.6))):
            pts = [(0, base)]
            for i in range(0, w + 1, 4):
                pts.append((i, base - _ridge(i, w, path_y * amp, seed)))
            pts.append((w, base))
            if world == "ice" and layer == 0:
                # Schneekante: weiße Grate, die Flanke etwas tiefer in Blau-Weiß
                pygame.draw.polygon(s, (252, 253, 255), pts)
                pts = [(x, y + (5 if 0 < x < w else 0)) for x, y in pts]
            pygame.draw.polygon(s, col, pts)
        if world == "river":
            pygame.draw.rect(s, (96, 160, 92), (0, base - 4, w, 6))
        return s
    return _cached(("far", world, w, h, path_y), build)


# ============================================================= Ufer + Seile
def _band(s, a, b, y0, hgt, col):
    """Senkrechter Streifen zwischen x=a und x=b (Reihenfolge egal)."""
    xa, xb = max(min(a, b), -10), min(max(a, b), s.get_width() + 10)
    if xb > xa and hgt > 0:
        s.fill(col, (int(xa), int(y0), int(xb - xa), int(hgt)))


def _road_bank(s, lip, side, y, h, k, top):
    """Gehweg der Straße: Bordstein an lip, Pflaster, Hecke, Rasen nach außen.

    side=-1: alles links von lip, +1: alles rechts davon. Zur Fahrbahn hin
    kommen nur Rinne und durchgezogene Randlinie dazu.
    """
    w = s.get_width()
    top, h, y = int(top), int(h), int(y)
    hh = h - top
    if hh <= 0:
        return
    sw = int(max(44, int(118 * k)) * 1.15)       # Gehweg inkl. Bordstein
    cw = max(2, int(4 * k))                      # Bordstein
    hw = max(6, int(18 * k))                     # Hecke
    ps = max(8, int(22 * k))                     # Pflasterstein
    jw = max(1, int(1.2 * k))                    # Fugen
    far = -10 if side < 0 else w + 10
    up = max(0, y - top)                         # Raster hängt am Pfad (y)
    span = h - min(y, top)                       # Höhe ab der ersten Rasterzeile
    # Rasen mit Mähstreifen und ein paar Blümchen
    g0 = lip + side * (sw + hw)
    _band(s, g0, far, top, hh, GRASS)
    bw = max(6, int(30 * k))
    fs = max(10, int(64 * k))
    m0 = -(up // fs + 1)
    rows = span // fs + 3
    fr = max(1, int(2 * k))
    near = w + 10 if side < 0 else -10
    j0 = max(0, int((near - g0) * side) // bw - 1)     # Streifen vor dem Bild überspringen
    for j in range(j0, j0 + (w + 20) // bw + 4):
        a = g0 + side * j * bw
        if (a - far) * side > 0:
            break
        if j % 2 == 1:
            _band(s, a, a + side * bw, top, hh, GRASS_D)
        for i in range(rows):
            if (j * 5 + i * 3) % 7 != 0:
                continue
            fx = int(a + side * bw * 0.5 + ((j * 13 + i * 7) % 9 - 4) * k)
            fy = int(y + (m0 + i) * fs + ((j * 7 + i * 11) % 11 - 5) * 2 * k)
            if fy - fr * 2 < top or fy + fr * 2 > h:
                continue
            petal = (250, 250, 240) if (j + i) % 2 == 0 else (246, 190, 210)
            for dx, dy in ((-1, 0), (1, 0), (0, -1), (0, 1)):
                pygame.draw.circle(s, petal, (fx + dx * fr, fy + dy * fr), fr)
            pygame.draw.circle(s, (246, 196, 64), (fx, fy), max(1, fr - 1))
    # Hecke: Band + runde Büsche
    _band(s, lip + side * sw, g0, top, hh, HEDGE_D)
    step = max(4, int(hw * 0.75))
    rad = hw * 0.56
    hx = lip + side * (sw + hw * 0.5)
    m0 = -(up // step + 1)
    for i in range(span // step + 3):
        cy = y + (m0 + i) * step
        cx = hx + (hw * 0.08 if i % 2 == 0 else -hw * 0.08)
        if cy - rad < top or cy + rad > h or cx < -hw or cx > w + hw:
            continue
        pygame.draw.circle(s, HEDGE, (int(cx), int(cy)), int(rad))
        pygame.draw.circle(s, HEDGE_L, (int(cx - rad * 0.28), int(cy - rad * 0.30)),
                           max(1, int(rad * 0.40)))
    # Gehweg: versetzte Pflastersteine
    _band(s, lip + side * cw, lip + side * sw, top, hh, PAVE)
    nc = max(2, int((sw - cw) / ps + 0.5))
    colw = (sw - cw) / nc
    m0 = -(up // ps + 2)
    for j in range(nc):
        xa = lip + side * (cw + j * colw)
        xb = lip + side * (cw + (j + 1) * colw)
        xl, xr = int(min(xa, xb)), int(max(xa, xb))
        if xr < -10 or xl > w + 10:
            continue
        off = ps // 2 if j % 2 == 1 else 0
        for i in range(span // ps + 4):
            yy = y + (m0 + i) * ps + off
            if yy >= h:
                break
            if yy + ps <= top:
                continue
            if (j * 5 + i * 3) % 7 == 0:
                y0 = max(top, yy)
                s.fill(PAVE_2, (xl, y0, xr - xl, min(h, yy + ps) - y0))
            if yy >= top:
                s.fill(PAVE_D, (xl, yy, xr - xl, jw))
        if j > 0:
            _band(s, xa, xa + side * jw, top, hh, PAVE_D)
    # Bordstein mit Fugen, Rinne und Randlinie auf der Fahrbahn
    _band(s, lip, lip + side * cw, top, hh, CURB)
    _band(s, lip, lip + side * max(1, cw // 3), top, hh, CURB_L)
    m0 = -(up // (ps * 3) + 1)
    for i in range(span // (ps * 3) + 3):
        yy = y + (m0 + i) * ps * 3
        if top <= yy < h:
            _band(s, lip, lip + side * cw, yy, 1, CURB_D)
    _band(s, lip, lip - side * max(1, int(2 * k)), top, hh, GUTTER)
    ex = lip - side * int(9 * k)
    _band(s, ex, ex - side * max(2, int(3 * k)), top, hh, LANE)


def draw_bank(s, world, x_edge, side, y, h, k, top=0):
    """Ufer: side=-1 links (Fels/Wiese bis x_edge), +1 rechts (ab x_edge).

    top: Oberkante (nur Straße; die übrigen Welten beginnen kurz über y).
    """
    w = s.get_width()
    if world == "road":
        # Rinne und Randlinie reichen bis ~12k auf die Fahrbahn
        m = int(14 * k) + 4
        if (side < 0 and x_edge < -m) or (side >= 0 and x_edge > w + m):
            return
        _road_bank(s, int(x_edge), -1 if side < 0 else 1, y, h, k, top)
        return
    if side < 0:
        if x_edge < 0:
            return
        x0, x1 = -10, int(x_edge)
    else:
        if x_edge > w:
            return
        x0, x1 = int(x_edge), w + 10
    if world == "bridge":
        rock, rock_d, top = (150, 92, 66), (104, 62, 48), (112, 168, 84)
        lip = int(x_edge)
        pts = [(x0, y), (x1, y), (x1, h), (x0, h)]
        if side < 0:
            pts = [(x0, y), (lip, y), (lip - int(18 * k), h), (x0, h)]
        else:
            pts = [(lip, y), (x1, y), (x1, h), (lip + int(18 * k), h)]
        pygame.draw.polygon(s, rock, pts)
        for i in range(1, 6):
            yy = y + int((h - y) * i / 6)
            off = int(18 * k * i / 6)
            a = (x0, yy) if side < 0 else (lip + off, yy)
            b = (lip - off, yy) if side < 0 else (x1, yy)
            pygame.draw.line(s, rock_d, a, b, max(1, int(2 * k)))
        pygame.draw.rect(s, top, (x0, y - int(6 * k), x1 - x0, int(8 * k)))
    elif world == "river":
        lip = int(x_edge)
        sand = int(22 * k)
        pygame.draw.rect(s, (88, 156, 74), (x0, y - int(8 * k), x1 - x0, h))
        for i in range(1, 5):
            yy = y + int((h - y) * i / 5)
            pygame.draw.line(s, (78, 142, 66), (x0, yy), (x1, yy), max(1, int(2 * k)))
        if side < 0:
            sand_pts = [(lip - sand, y - int(8 * k)), (lip, y - int(4 * k)), (lip + int(6 * k), h),
                        (lip - sand, h)]
        else:
            sand_pts = [(lip + sand, y - int(8 * k)), (lip, y - int(4 * k)), (lip - int(6 * k), h),
                        (lip + sand, h)]
        pygame.draw.polygon(s, (222, 200, 146), sand_pts)
        pygame.draw.line(s, (250, 252, 255), (lip, y - int(4 * k)),
                         (lip + side * -int(6 * k), h), max(1, int(3 * k)))
        pygame.draw.rect(s, (110, 184, 92), (x0, y - int(10 * k), x1 - x0, int(4 * k)))
    else:
        pygame.draw.rect(s, (236, 244, 250), (x0, y - int(4 * k), x1 - x0, h))
        pygame.draw.rect(s, (196, 214, 232), (x0, y + int(10 * k), x1 - x0, int(4 * k)))
        pygame.draw.line(s, (255, 255, 255), (x0, y - int(4 * k)), (x1, y - int(4 * k)),
                         max(1, int(2 * k)))


def draw_ropes(s, points, k, alpha_from=None):
    """Seile der Hängebrücke: Laufseil durch die Plankenenden, Handlauf darüber.

    points: Liste (x, y) der Seil-Anker (Ufer, Planken, Ufer) in Bildkoordinaten.
    """
    rope = (120, 82, 48)
    lw = max(1, int(2 * k))
    rail = int(46 * k)
    for i in range(len(points) - 1):
        (xa, ya), (xb, yb) = points[i], points[i + 1]
        if xb < -40 or xa > s.get_width() + 40:
            continue
        mx, my = (xa + xb) / 2, (ya + yb) / 2 + 5 * k
        pygame.draw.lines(s, rope, False, [(xa, ya), (mx, my), (xb, yb)], lw)
        pygame.draw.lines(s, rope, False, [(xa, ya - rail), (mx, my - rail + 2 * k),
                                           (xb, yb - rail)], lw)
        pygame.draw.line(s, rope, (xb, yb), (xb, yb - rail), max(1, lw - 1))


def draw_lanes(s, xs, top, bottom, k, t):
    """Gestrichelte Spur-Trennlinien der Straße (statisch, t ungenutzt).

    xs: Bild-x der Trennlinien zwischen den Spuren.
    """
    lw = max(2, int(3 * k))
    dash = max(6, int(26 * k))
    gap = max(4, int(18 * k))
    w = s.get_width()
    top, bottom = int(top), int(bottom)
    for x in xs:
        if x < -lw or x > w + lw:
            continue
        xi = int(x) - lw // 2
        yy = top
        while yy < bottom:
            s.fill(LANE, (xi, yy, lw, min(dash, bottom - yy)))
            yy += dash + gap


# ============================================================= Kacheln
def draw_tile(s, world, cx, y, tw, k, t, idx, alpha=255):
    """Ein Schritt: Planke / Stein / Scholle, Oberkante bei y, Mitte cx.

    Straße: Zebrastreifen-Stück von y - tw*0.38 bis y + tw*0.17 (Füße bei y).
    """
    if alpha <= 0:
        return
    if alpha < 255:
        oy = int(tw * 0.45) if world == "road" else int(tw * 0.2)
        layer = pygame.Surface((int(tw * 1.4) + 8, int(tw * 0.9) + 8), pygame.SRCALPHA)
        draw_tile(layer, world, layer.get_width() / 2, oy, tw, k, t, idx)
        layer.set_alpha(alpha)
        s.blit(layer, (cx - layer.get_width() / 2, y - oy))
        return
    if world == "road":
        # Zebrastreifen: weiße Balken parallel zum Verkehr (senkrecht)
        nb = 4
        u = tw / (2 * nb - 1)
        zh = max(4, int(tw * 0.55))
        zy = int(y - tw * 0.38)
        bw = max(2, int(u))
        rad = max(1, int(u * 0.25))
        for j in range(nb):
            bx = int(cx - tw / 2 + j * 2 * u)
            pygame.draw.rect(s, ZEBRA_D, (bx, zy, bw, zh), border_radius=rad)
            pygame.draw.rect(s, ZEBRA, (bx, zy, bw, zh - max(1, int(2 * k))), border_radius=rad)
        return
    if world == "bridge":
        th = max(5, int(13 * k))
        r = pygame.Rect(0, 0, int(tw), th)
        r.midtop = (int(cx), int(y))
        pygame.draw.rect(s, (96, 60, 34), r.move(0, max(1, int(3 * k))), border_radius=max(1, int(3 * k)))
        pygame.draw.rect(s, (168, 112, 66), r, border_radius=max(1, int(3 * k)))
        for j in (0.3, 0.68):
            yy = r.y + int(r.h * j)
            pygame.draw.line(s, (132, 84, 48), (r.x + 4, yy), (r.right - 4 - int(6 * j * k), yy), 1)
        for xx in (r.x + int(5 * k), r.right - int(5 * k)):
            pygame.draw.circle(s, (70, 58, 54), (xx, r.y + r.h // 2), max(1, int(1.6 * k)))
    elif world == "river":
        bob = math.sin(t * 2.2 + idx * 1.3) * 1.5 * k
        h = int(tw * 0.40)
        r = pygame.Rect(0, 0, int(tw), h)
        r.midtop = (int(cx), int(y - h * 0.22 + bob))
        ring = pygame.Rect(0, 0, int(tw * 1.25), int(h * 0.62))
        ring.center = (int(cx), int(y + h * 0.42 + bob))
        pygame.draw.ellipse(s, (170, 214, 236), ring, max(1, int(2 * k)))
        pygame.draw.ellipse(s, (82, 88, 96), r.move(0, int(4 * k)))
        pygame.draw.ellipse(s, (128, 134, 142), r)
        top = r.inflate(-int(tw * 0.28), -int(h * 0.45)).move(-int(tw * 0.05), -int(h * 0.12))
        pygame.draw.ellipse(s, (170, 176, 184), top)
        pygame.draw.ellipse(s, (96, 140, 92), (r.x + int(tw * 0.62), r.y + int(h * 0.10),
                                               int(tw * 0.16), int(h * 0.20)))
    else:
        h = int(tw * 0.34)
        side = max(3, int(7 * k))
        pts = []
        for j in range(7):
            a = math.tau * j / 7 + idx * 0.7
            rad = 0.5 + 0.06 * math.sin(idx * 2.1 + j * 1.7)
            pts.append((cx + math.cos(a) * tw * 0.80 * rad * 1.1,
                        y + h * 0.5 + math.sin(a) * h * rad))
        pygame.draw.polygon(s, (24, 52, 76), [(x, yy + side + 3 * k) for x, yy in pts])
        pygame.draw.polygon(s, (164, 202, 226), [(x, yy + side) for x, yy in pts])
        pygame.draw.polygon(s, (228, 242, 250), pts)
        pygame.draw.line(s, (255, 255, 255), (cx - tw * 0.22, y + h * 0.28),
                         (cx - tw * 0.02, y + h * 0.18), max(1, int(2 * k)))


def crack_lines(cx, y, tw, idx, f):
    """Risslinien einer Eisscholle (f = Fortschritt 0..1)."""
    lines = []
    for j in range(4):
        a = math.tau * j / 4 + idx * 0.9 + 0.4
        pts = [(cx, y + tw * 0.15)]
        for step in range(1, 5):
            if step / 4 > f:
                break
            r = tw * 0.16 * step
            wob = math.sin(idx * 3.1 + j * 2.0 + step * 1.9) * 0.45
            pts.append((cx + math.cos(a + wob) * r, y + tw * 0.15 + math.sin(a + wob) * r * 0.42))
        if len(pts) > 1:
            lines.append(pts)
    return lines


# ============================================================= Hindernisse
CAR_COLS = ((216, 66, 58), (60, 124, 214), (242, 190, 50))
TRUCK_COL = (36, 150, 140)
GLASS = (56, 72, 98)
GLASS_L = (150, 186, 214)
TIRE = (32, 32, 38)
HEAD = (255, 248, 206)
TAIL = (196, 26, 40)


def obstacle_len(world, kind, size):
    """Senkrechte Länge eines Hindernisses in Pixeln (für Treffer-Test/Halten)."""
    if world == "road":
        return int(size * (1.45 if kind == 3 else 0.86))
    if world == "bridge":
        return int(size * 0.42)
    if world == "river":
        return int(size * 0.30)
    return int(size * 0.48)


def _car_shadow(b, x0, y0, w, ln, r):
    """Weicher Schatten nach rechts unten (3 Stufen, ersetzt Alpha)."""
    sh = max(2, int(w * 0.08))
    for i, a in ((2, 28), (1, 52), (0, 78)):
        g = sh * i * 0.5
        _rrect(b, (0, 0, 0, a), x0 + sh - g, y0 + sh * 1.4 - g, w + 2 * g, ln + 2 * g, r + g)


def _draw_car(b, cx, cy, w, ln, kind):
    """Auto von oben, Front unten (fährt nach unten); w/ln in Feld-Pixeln."""
    col = CAR_COLS[kind % 3]
    dark = ui.mix(col, (0, 0, 0), 0.30)
    light = ui.mix(col, (255, 255, 255), 0.32)
    roof = ui.mix(col, (255, 255, 255), 0.14)
    x0, y0 = cx - w / 2, cy - ln / 2

    def v(u):
        return y0 + ln * u
    _car_shadow(b, x0, y0, w, ln, w * 0.30)
    # Räder (schauen seitlich etwas heraus)
    rw, rh = w * 0.14, ln * 0.17
    for u in (0.19, 0.79):
        for sx in (-1, 1):
            _rrect(b, TIRE, cx + sx * (w / 2 - rw * 0.30) - rw / 2, v(u) - rh / 2, rw, rh, rw * 0.35)
    # Karosserie: dunkle Flanken, helle Oberseite
    _rrect(b, dark, x0, y0, w, ln, w * 0.30)
    _rrect(b, col, x0 + w * 0.07, y0 + ln * 0.012, w * 0.86, ln * 0.976, w * 0.26)
    # Glanz auf Motorhaube und Kofferraum
    _rrect(b, light, x0 + w * 0.15, v(0.76), w * 0.07, ln * 0.17, w * 0.035)
    _rrect(b, light, x0 + w * 0.15, v(0.05), w * 0.07, ln * 0.10, w * 0.035)
    # Heckscheibe, Dach, Seitenfenster, Frontscheibe
    pygame.draw.polygon(b, GLASS, [(cx - w * 0.27, v(0.19)), (cx + w * 0.27, v(0.19)),
                                   (cx + w * 0.34, v(0.295)), (cx - w * 0.34, v(0.295))])
    for sx in (-1, 1):
        pygame.draw.polygon(b, GLASS, [(cx + sx * w * 0.345, v(0.30)), (cx + sx * w * 0.405, v(0.32)),
                                       (cx + sx * w * 0.405, v(0.57)), (cx + sx * w * 0.345, v(0.585))])
    _rrect(b, roof, cx - w * 0.34, v(0.285), w * 0.68, ln * 0.31, w * 0.10)
    pygame.draw.polygon(b, GLASS, [(cx - w * 0.34, v(0.585)), (cx + w * 0.34, v(0.585)),
                                   (cx + w * 0.40, v(0.715)), (cx - w * 0.40, v(0.715))])
    lw = max(1, int(w * 0.05))
    pygame.draw.line(b, GLASS_L, (cx - w * 0.24, v(0.70)), (cx - w * 0.06, v(0.60)), lw)
    pygame.draw.line(b, GLASS_L, (cx - w * 0.04, v(0.70)), (cx + w * 0.04, v(0.645)), lw)
    pygame.draw.line(b, GLASS_L, (cx + w * 0.10, v(0.20)), (cx + w * 0.20, v(0.28)), lw)
    if kind % 3 == 0:
        # Rennstreifen über Motorhaube, Dach und Kofferraum
        for u0, u1 in ((0.015, 0.185), (0.30, 0.575), (0.725, 0.985)):
            for sx in (-1, 1):
                _rrect(b, (248, 246, 240), cx + sx * w * 0.075 - w * 0.035, v(u0), w * 0.07,
                       ln * (u1 - u0), 0)
    elif kind % 3 == 1:
        # Schiebedach
        _rrect(b, GLASS, cx - w * 0.17, v(0.35), w * 0.34, ln * 0.13, w * 0.06)
        pygame.draw.line(b, GLASS_L, (cx - w * 0.09, v(0.465)), (cx + w * 0.02, v(0.37)), lw)
    else:
        # Dachträger
        for u in (0.36, 0.51):
            _rrect(b, (66, 66, 74), cx - w * 0.31, v(u), w * 0.62, ln * 0.025, w * 0.02)
        for sx in (-1, 1):
            _rrect(b, (66, 66, 74), cx + sx * w * 0.24 - w * 0.025, v(0.34), w * 0.05, ln * 0.20, 0)
    # Außenspiegel
    for sx in (-1, 1):
        pygame.draw.ellipse(b, dark, (int(cx + sx * w * 0.50 - w * 0.07), int(v(0.585)),
                                      int(w * 0.14), max(2, int(ln * 0.045))))
    # Scheinwerfer vorn (unten), Rückleuchten hinten (oben)
    for sx in (-1, 1):
        _rrect(b, HEAD, cx + sx * w * 0.27 - w * 0.10, v(0.962), w * 0.20, ln * 0.026, w * 0.04)
        _rrect(b, TAIL, cx + sx * w * 0.28 - w * 0.08, v(0.010), w * 0.16, ln * 0.024, w * 0.03)


def _draw_truck(b, cx, cy, w, ln):
    """Lieferwagen von oben: Kofferaufbau hinten, Fahrerhaus vorn (unten)."""
    col = TRUCK_COL
    dark = ui.mix(col, (0, 0, 0), 0.30)
    roof = ui.mix(col, (255, 255, 255), 0.16)
    box, box_d, box_r = (238, 236, 228), (190, 188, 180), (216, 214, 206)
    x0, y0 = cx - w / 2, cy - ln / 2

    def v(u):
        return y0 + ln * u
    _car_shadow(b, x0, y0, w, ln, w * 0.12)
    rw, rh = w * 0.13, ln * 0.09
    for u in (0.12, 0.23, 0.86):
        for sx in (-1, 1):
            _rrect(b, TIRE, cx + sx * (w / 2 - rw * 0.30) - rw / 2, v(u) - rh / 2, rw, rh, rw * 0.35)
    # Kofferaufbau mit Rippen und Zierstreifen
    _rrect(b, box_d, x0, y0, w, ln * 0.71, w * 0.08)
    _rrect(b, box, x0 + w * 0.05, y0 + ln * 0.006, w * 0.90, ln * 0.70, w * 0.06)
    lw = max(1, int(w * 0.025))
    for i in range(1, 11):
        yy = v(0.065 * i)
        pygame.draw.line(b, box_r, (x0 + w * 0.10, yy), (x0 + w * 0.90, yy), lw)
    for sx in (-1, 1):
        _rrect(b, col, cx + sx * w * 0.36 - w * 0.04, v(0.03), w * 0.08, ln * 0.65, 0)
    pygame.draw.line(b, box_d, (cx, v(0.006)), (cx, v(0.05)), lw)
    for sx in (-1, 1):
        _rrect(b, TAIL, cx + sx * w * 0.36 - w * 0.07, v(0.004), w * 0.14, ln * 0.016, w * 0.02)
    # Kupplung + Fahrerhaus
    _rrect(b, (54, 54, 62), cx - w * 0.24, v(0.70), w * 0.48, ln * 0.05, 0)
    _rrect(b, dark, x0 + w * 0.03, v(0.735), w * 0.94, ln * 0.265, w * 0.24)
    _rrect(b, col, x0 + w * 0.09, v(0.742), w * 0.82, ln * 0.252, w * 0.20)
    _rrect(b, roof, cx - w * 0.33, v(0.752), w * 0.66, ln * 0.12, w * 0.08)
    for sx in (-1, 1):
        pygame.draw.circle(b, (255, 184, 56), (int(cx + sx * w * 0.16), int(v(0.77))),
                           max(1, int(w * 0.04)))
    pygame.draw.polygon(b, GLASS, [(cx - w * 0.33, v(0.876)), (cx + w * 0.33, v(0.876)),
                                   (cx + w * 0.38, v(0.935)), (cx - w * 0.38, v(0.935))])
    pygame.draw.line(b, GLASS_L, (cx - w * 0.22, v(0.93)), (cx - w * 0.10, v(0.88)), max(1, int(w * 0.04)))
    for sx in (-1, 1):
        pygame.draw.ellipse(b, dark, (int(cx + sx * w * 0.52 - w * 0.08), int(v(0.86)),
                                      int(w * 0.16), max(2, int(ln * 0.025))))
        _rrect(b, HEAD, cx + sx * w * 0.28 - w * 0.09, v(0.976), w * 0.18, ln * 0.016, w * 0.03)


def _car_sprite(size, kind):
    """Auto/Lieferwagen als gecachte Fläche, 2-fach überabgetastet."""
    def build():
        truck = kind == 3
        bw = max(4, int(size * (0.58 if truck else 0.52)))
        ln = max(4, obstacle_len("road", kind, size))
        m = max(3, int(size * 0.10))
        fw, fh = bw + 2 * m, ln + 2 * m
        big = pygame.Surface((fw * 2, fh * 2), pygame.SRCALPHA)
        if truck:
            _draw_truck(big, fw, fh, bw * 2, ln * 2)
        else:
            _draw_car(big, fw, fh, bw * 2, ln * 2, kind)
        return pygame.transform.smoothscale(big, (fw, fh))
    return _cached(("car", size, kind), build)


def _boulder(s, x, y, size, kind):
    """Fallender Felsbrocken (Brücke), Durchmesser = obstacle_len."""
    r = obstacle_len("bridge", kind, size) / 2

    def ring(sc, dx, dy):
        pts = []
        for j in range(9):
            a = math.tau * j / 9 + kind * 0.6
            rad = r * sc * (0.86 + 0.14 * math.sin(j * 2.4 + kind * 1.9))
            pts.append((x + dx + math.cos(a) * rad * 1.08, y + dy + math.sin(a) * rad))
        return pts
    pygame.draw.polygon(s, (74, 66, 68), ring(1.0, 0, 0))
    pygame.draw.polygon(s, (132, 124, 120), ring(0.84, -r * 0.08, -r * 0.10))
    pygame.draw.polygon(s, (168, 160, 152), ring(0.42, -r * 0.28, -r * 0.32))
    lw = max(1, int(r * 0.09))
    pygame.draw.lines(s, (90, 82, 82), False, [(x + r * 0.12, y - r * 0.18), (x + r * 0.30, y + r * 0.08),
                                               (x + r * 0.20, y + r * 0.40)], lw)
    pygame.draw.line(s, (90, 82, 82), (x - r * 0.42, y + r * 0.22), (x - r * 0.12, y + r * 0.34), lw)


def _log(s, x, y, size, kind):
    """Treibholz (Fluss): liegender Stamm, Stirnseite rechts, Gischt unten."""
    lw = max(6, int(size * 0.80))
    lh = max(3, obstacle_len("river", kind, size))
    x0, y0 = int(x - lw / 2), int(y - lh / 2)
    bark, bark_d, bark_l = (128, 84, 50), (88, 56, 36), (160, 112, 70)
    wood, ring = (228, 188, 130), (180, 132, 84)
    # Stamm (unten dunkler = rund)
    pygame.draw.rect(s, bark_d, (x0, y0, lw, lh), border_radius=lh // 2)
    pygame.draw.rect(s, bark, (x0, y0, lw, max(2, int(lh * 0.70))), border_radius=lh // 2)
    t = max(1, int(lh * 0.10))
    pygame.draw.line(s, bark_l, (x0 + lh * 0.5, y0 + lh * 0.22), (x0 + lw - lh * 0.9, y0 + lh * 0.22), t)
    # Rindenrillen
    for j in range(3):
        yy = y0 + lh * (0.40 + 0.18 * j)
        a = x0 + lh * (0.45 + 0.9 * ((j * 2 + kind) % 3))
        b = a + lw * (0.30 + 0.06 * ((kind + j) % 2))
        pygame.draw.line(s, bark_d, (a, yy), (min(b, x0 + lw - lh * 0.8), yy), t)
    # Aststummel oben
    sx = x0 + lw * (0.30 + 0.12 * (kind % 3))
    pygame.draw.ellipse(s, bark_d, (int(sx - lh * 0.16), int(y0 - lh * 0.12), max(2, int(lh * 0.32)),
                                    max(2, int(lh * 0.30))))
    pygame.draw.ellipse(s, wood, (int(sx - lh * 0.09), int(y0 - lh * 0.07), max(1, int(lh * 0.18)),
                                  max(1, int(lh * 0.16))))
    # Stirnseite mit Jahresringen
    ew = max(2, int(lh * 0.62))
    er = pygame.Rect(x0 + lw - ew, y0, ew, lh)
    pygame.draw.ellipse(s, bark_d, er)
    pygame.draw.ellipse(s, wood, er.inflate(-max(2, int(lh * 0.16)), -max(2, int(lh * 0.16))))
    pygame.draw.ellipse(s, ring, er.inflate(-int(ew * 0.45), -int(lh * 0.45)), max(1, int(lh * 0.06)))
    pygame.draw.circle(s, ring, er.center, max(1, int(lh * 0.06)))
    # kleine Gischt an der Wasserlinie (flache Schaumkronen + Bugwelle)
    foam = (240, 248, 252)
    for j in range(5):
        fw = lw * (0.16 + 0.04 * ((j + kind) % 2))
        fx = x0 + lw * (0.02 + 0.20 * j)
        fh = max(2, int(lh * 0.22))
        fy = y0 + lh * (0.90 + 0.05 * math.sin(j * 1.7 + kind))
        pygame.draw.ellipse(s, foam, (int(fx), int(fy), max(2, int(fw)), fh))
    for sx in (-1, 1):
        ex = x0 - lh * 0.10 if sx < 0 else x0 + lw - lh * 0.12
        pygame.draw.ellipse(s, foam, (int(ex), int(y0 + lh * 0.70), max(2, int(lh * 0.26)),
                                      max(2, int(lh * 0.34))))


def _ice_block(s, x, y, size, kind):
    """Rutschender Eisblock (Eissee): Quader mit Deck- und Seitenfläche."""
    bh = max(4, obstacle_len("ice", kind, size))
    bw = max(4, int(size * 0.62))
    x0, y0 = int(x - bw / 2), int(y - bh / 2)
    x1, y1 = x0 + bw, y0 + bh
    th = int(bh * (0.36 + 0.04 * (kind % 2)))       # Höhe der Deckfläche
    sd = max(2, int(bw * 0.16))                      # Versatz der Seitenfläche
    snow = (236, 246, 252)
    sp = max(2, int(bh * 0.16))
    # Schneestaub seitlich hinter dem Block
    for fx, fy in ((-0.12, -0.20), (1.0, -0.40)):
        rw, rh = max(2, int(sp * 1.6)), max(1, int(sp * 0.9))
        pygame.draw.ellipse(s, snow, (int(x0 + bw * fx - rw / 2), int(y1 + bh * fy - rh / 2), rw, rh))
    _ice_faces(s, x0, y0, x1, y1, th, sd, bw, bh, kind)
    # vorn aufgeschobener Schneewulst + ein paar Flocken
    pygame.draw.ellipse(s, snow, (x0 - int(bw * 0.06), y1 - int(sp * 0.7), bw - sd + int(bw * 0.12),
                                  max(2, int(sp * 1.1))))
    for j, (fx, fy) in enumerate(((-0.17, -0.04), (-0.10, -0.34), (1.06, -0.18))):
        pygame.draw.circle(s, snow, (int(x0 + bw * fx), int(y1 + bh * fy)),
                           max(1, int(sp * (0.30 + 0.10 * ((j + kind) % 2)))))


def _ice_faces(s, x0, y0, x1, y1, th, sd, bw, bh, kind):
    """Flächen, Kanten und Glanz des Eisblocks."""
    edge = (92, 150, 194)
    front = [(x0, y0 + th), (x1 - sd, y0 + th), (x1 - sd, y1), (x0, y1)]
    top = [(x0, y0 + th), (x0 + sd, y0), (x1, y0), (x1 - sd, y0 + th)]
    side = [(x1 - sd, y0 + th), (x1, y0), (x1, y1 - th), (x1 - sd, y1)]
    pygame.draw.polygon(s, (178, 222, 244), front)
    pygame.draw.polygon(s, (128, 182, 220), side)
    pygame.draw.polygon(s, (240, 250, 255), top)
    lw = max(1, int(bw * 0.025))
    for pts in (front, top, side):
        pygame.draw.polygon(s, edge, pts, lw)
    # Glanz + Luftbläschen + Riss
    fw = bw - sd
    pygame.draw.line(s, (255, 255, 255), (x0 + fw * 0.14, y1 - (bh - th) * 0.25),
                     (x0 + fw * 0.38, y0 + th + (bh - th) * 0.22), max(1, int(bw * 0.05)))
    pygame.draw.line(s, (255, 255, 255), (x0 + fw * 0.34, y1 - (bh - th) * 0.20),
                     (x0 + fw * 0.46, y0 + th + (bh - th) * 0.45), max(1, int(bw * 0.03)))
    for j in range(2):
        pygame.draw.circle(s, (220, 242, 252), (int(x0 + fw * (0.62 + 0.16 * j)),
                                                int(y0 + th + (bh - th) * (0.35 + 0.25 * j))),
                           max(1, int(bw * 0.03)))
    if kind % 2 == 1:
        pygame.draw.lines(s, edge, False, [(x0 + fw * 0.70, y0 + th),
                                           (x0 + fw * 0.62, y0 + th + (bh - th) * 0.4),
                                           (x0 + fw * 0.72, y1 - (bh - th) * 0.15)], lw)
    pygame.draw.line(s, (255, 255, 255), (x0 + sd * 0.6, y0 + th * 0.5), (x0 + sd + bw * 0.25, y0 + th * 0.5),
                     max(1, int(bw * 0.03)))


def draw_obstacle(s, world, x, y, size, kind, alpha=255):
    """Hindernis mittig bei (x, y); size = fertige Pixelgröße, kind 0..3.

    Straße: Auto (kind 0-2 rot/blau/gelb) bzw. Lieferwagen (3) von oben,
    Front unten; Brücke: Felsbrocken; Fluss: Treibholz; Eissee: Eisblock.
    Senkrechte Ausdehnung = obstacle_len(world, kind, size).
    """
    size = int(size)
    if alpha <= 0 or size <= 0:
        return
    kind = kind % 4
    if world == "road":
        img = _car_sprite(size, kind)
        if alpha < 255:
            img = img.copy()
            img.set_alpha(alpha)
        s.blit(img, (int(x) - img.get_width() // 2, int(y) - img.get_height() // 2))
        return
    if alpha < 255:
        lw, lh = size + 8, int(size * 0.8) + 8
        layer = pygame.Surface((lw, lh), pygame.SRCALPHA)
        draw_obstacle(layer, world, lw // 2, lh // 2, size, kind)
        layer.set_alpha(alpha)
        s.blit(layer, (int(x) - lw // 2, int(y) - lh // 2))
        return
    if world == "bridge":
        _boulder(s, int(x), int(y), size, kind)
    elif world == "river":
        _log(s, int(x), int(y), size, kind)
    else:
        _ice_block(s, int(x), int(y), size, kind)


def draw_barrier(s, x, y, w, k):
    """Rot-weiße Absperrung (Balken der Breite w mittig bei x, Höhe ~8k bei y)."""
    w = max(8, int(w))
    bh = max(4, int(8 * k))
    pw = max(2, int(4 * k))
    ph = max(4, int(12 * k))
    x0, y0 = int(x) - w // 2, int(y) - bh // 2
    post, post_d = (84, 86, 96), (48, 50, 58)
    # Pfosten mit Füßen
    for px in (x0 + pw, x0 + w - 2 * pw):
        s.fill(post, (px, y0, pw, bh // 2 + ph))
        s.fill(post_d, (px - pw, y0 + bh // 2 + ph - max(2, int(3 * k)), pw * 3, max(2, int(3 * k))))

    def build():
        bar = pygame.Surface((w, bh))
        bar.fill((244, 244, 238))
        st = max(3, int(bh * 1.1))
        for i in range(-1, w // st + 2):
            if i % 2 == 0:
                continue
            sx = i * st
            pygame.draw.polygon(bar, (214, 44, 46), [(sx, 0), (sx + st, 0), (sx + st - bh, bh), (sx - bh, bh)])
        pygame.draw.rect(bar, (120, 28, 34), (0, 0, w, bh), 1)
        return bar
    s.blit(_cached(("barrier", w, bh), build), (x0, y0))
    # kleines Warnlicht links oben
    r = max(2, int(4 * k))
    lx, ly = x0 + pw + pw // 2, y0 - r
    glow = _cached(("glow", r), lambda: _glow_surface(r))
    s.blit(glow, (lx - glow.get_width() // 2, ly - glow.get_height() // 2))
    s.fill(post_d, (lx - r // 2, ly, max(1, r), r + 1))
    pygame.draw.circle(s, (255, 168, 40), (lx, ly), r)
    pygame.draw.circle(s, (255, 236, 170), (lx - r // 3, ly - r // 3), max(1, r // 2))


def _glow_surface(r):
    """Weicher Lichthof für das Warnlicht (3 Ringe, Alpha ersetzt)."""
    g = pygame.Surface((r * 6, r * 6), pygame.SRCALPHA)
    for i, a in ((3, 40), (2, 70), (1, 100)):
        pygame.draw.circle(g, (255, 190, 70, a), (r * 3, r * 3), int(r * (1 + i * 0.6)))
    return g


def draw_tire_marks(s, x, y, w, k):
    """Zwei dunkle, halbdurchsichtige Reifenspuren mittig bei (x, y).

    Länge ~ w*1.2, Abstand der Spuren ~ w*0.32, Spurbreite max(2, 5k).
    """
    sw = max(2, int(5 * k))
    ln = max(4, int(w * 1.2))
    gp = max(sw + 1, int(w * 0.32))
    tread = max(2, int(3 * k))

    def build():
        lay = pygame.Surface((gp + sw, ln), pygame.SRCALPHA)
        half = ln / 2
        for yy in range(ln):
            f = 1.0 - abs(yy + 0.5 - half) / half          # 0 an den Enden, 1 Mitte
            a = int(30 + 110 * min(1.0, f * 2.5))
            if (yy // tread) % 2 == 1:
                a = a * 6 // 10                              # Profilkerben
            for xx in (0, gp):
                lay.fill((22, 20, 26, a), (xx, yy, sw, 1))
        return lay
    lay = _cached(("tire", ln, gp, sw, tread), build)
    s.blit(lay, (int(x) - lay.get_width() // 2, int(y) - ln // 2))


def world_icon(s, world, rect, col):
    """Piktogramm der Welt (für die Welt-Knöpfe)."""
    r = pygame.Rect(rect)
    cx, cy = r.centerx, r.centery
    u = r.w / 10.0
    lw = max(1, int(u * 0.8))
    if world == "bridge":
        pygame.draw.lines(s, col, False, [(r.x + u, cy - u * 2), (cx, cy - u * 0.8),
                                          (r.right - u, cy - u * 2)], lw)
        pygame.draw.lines(s, col, False, [(r.x + u, cy + u), (cx, cy + u * 2.2),
                                          (r.right - u, cy + u)], lw)
        for i in range(-2, 3):
            x = cx + i * u * 1.6
            dy = u * (2.2 - 0.3 * i * i) * 0.5
            pygame.draw.line(s, col, (x, cy + u + dy - u * 0.6), (x, cy + u + dy + u * 0.4), lw)
    elif world == "river":
        for i in range(3):
            yy = cy + u * (1.0 + i * 1.3)
            pygame.draw.lines(s, col, False, [(r.x + u, yy), (cx - u * 1.5, yy - u * 0.6),
                                              (cx + u * 1.5, yy + u * 0.3), (r.right - u, yy - u * 0.4)],
                              max(1, lw - 1))
        pygame.draw.ellipse(s, col, (cx - u * 2.4, cy - u * 2.2, u * 4.8, u * 2.6))
    elif world == "road":
        # Straße in die Ferne: zulaufende Ränder, kürzer werdende Mittelstriche
        for sx in (-1, 1):
            pygame.draw.line(s, col, (cx + sx * u * 4.4, r.bottom - u * 0.6),
                             (cx + sx * u * 1.2, r.y + u * 1.0), lw)
        pygame.draw.line(s, col, (cx, r.bottom - u * 0.6), (cx, r.bottom - u * 2.8), lw + 1)
        pygame.draw.line(s, col, (cx, cy + u * 0.6), (cx, cy - u * 0.9), lw)
        pygame.draw.line(s, col, (cx, r.y + u * 2.6), (cx, r.y + u * 1.6), max(1, lw - 1))
    else:
        pygame.draw.polygon(s, col, [(r.x + u, cy), (cx - u * 1.5, cy - u * 2.0),
                                     (cx + u * 2.5, cy - u * 1.6), (r.right - u, cy + u * 0.4),
                                     (cx + u * 0.5, cy + u * 1.8), (cx - u * 2.6, cy + u * 1.4)], lw)
        pygame.draw.lines(s, col, False, [(cx - u * 0.5, cy - u * 1.6), (cx + u * 0.2, cy - u * 0.2),
                                          (cx - u * 0.6, cy + u * 0.6), (cx + u * 0.4, cy + u * 1.6)],
                          max(1, lw - 1))
