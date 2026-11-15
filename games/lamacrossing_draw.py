# -*- coding: utf-8 -*-
"""
lamacrossing_draw.py
====================
Zeichen-Bausteine von Lama Crossing (nur pygame-Primitive, keine Bilddateien):

- ``llama_sprite``  : das Lama von der Seite (stehen / springen / fallen) mit
                      Schal in der Akzentfarbe, 2-fach überabgetastet und weich
                      verkleinert, gecacht je Größe und Pose,
- ``background``    : Himmel + Ferne je Welt (Hängebrücke, Fluss, Eissee),
                      gecacht je Auflösung,
- ``far_layer``     : kachelbare Silhouetten-Ebene für den Parallax-Effekt,
- ``draw_tile``     : Planke, Stein bzw. Eisscholle eines Schritts,
- ``draw_bank``     : Start- und Zielufer (Fels, Wiese, Schnee),
- ``draw_ropes``    : Seile der Hängebrücke,
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


def background(world, w, h, path_y):
    """Himmel, Sonne/Licht und Untergrund unterhalb des Pfads (statisch)."""
    def build():
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
def draw_bank(s, world, x_edge, side, y, h, k):
    """Ufer: side=-1 links (Fels/Wiese bis x_edge), +1 rechts (ab x_edge)."""
    w = s.get_width()
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


# ============================================================= Kacheln
def draw_tile(s, world, cx, y, tw, k, t, idx, alpha=255):
    """Ein Schritt: Planke / Stein / Scholle, Oberkante bei y, Mitte cx."""
    if alpha <= 0:
        return
    if alpha < 255:
        layer = pygame.Surface((int(tw * 1.4) + 8, int(tw * 0.9) + 8), pygame.SRCALPHA)
        draw_tile(layer, world, layer.get_width() / 2, int(tw * 0.2), tw, k, t, idx)
        layer.set_alpha(alpha)
        s.blit(layer, (cx - layer.get_width() / 2, y - int(tw * 0.2)))
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
    else:
        pygame.draw.polygon(s, col, [(r.x + u, cy), (cx - u * 1.5, cy - u * 2.0),
                                     (cx + u * 2.5, cy - u * 1.6), (r.right - u, cy + u * 0.4),
                                     (cx + u * 0.5, cy + u * 1.8), (cx - u * 2.6, cy + u * 1.4)], lw)
        pygame.draw.lines(s, col, False, [(cx - u * 0.5, cy - u * 1.6), (cx + u * 0.2, cy - u * 0.2),
                                          (cx - u * 0.6, cy + u * 0.6), (cx + u * 0.4, cy + u * 1.6)],
                          max(1, lw - 1))
