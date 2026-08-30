# -*- coding: utf-8 -*-
"""
minigolf_draw.py
================
Das Aussehen einer Minigolf-Bahn - an EINER Stelle.

Bis zum Bahn-Editor zeichnete ``minigolf.py`` den Platz selbst. Seit es
eigene Bahnen gibt, muss dasselbe aber zweimal auf den Schirm: im Spiel und
im Editor. Doppelter Zeichencode wäre der sichere Weg, dass Editor und
Spiel mit der Zeit unterschiedlich aussehen - deshalb liegen die Farben und
alle Zeichenfunktionen hier, und beide Seiten rufen sie auf.

Alle Funktionen bekommen eine ``View``: sie kennt Nullpunkt und Maßstab und
rechnet Bahn-Einheiten in Bildschirm-Pixel um.

    view = View(ox, oy, scale)
    draw_course(surface, hole, view, mill_a, move_t)

``hole`` ist das ganz normale Bahn-Dict (siehe ``minigolf_gen.make_hole``);
es muss vorher durch ``minigolf_gen.normalize`` gelaufen sein, damit alle
15 Hindernis-Listen vorhanden sind.
"""

import math

import pygame

import ui

from .minigolf_gen import BORDER, CW, CH

# ------------------------------------------------- Identitätsfarben (Platz)
COL_GREEN = (46, 132, 74)
COL_GREEN_D = (38, 112, 62)
COL_FRINGE = (62, 152, 88)
COL_WALL = (122, 84, 52)
COL_WALL_HI = (156, 112, 72)
COL_SAND = (214, 190, 132)
COL_SAND_D = (190, 164, 108)
COL_WATER = (52, 122, 196)
COL_WATER_D = (36, 92, 158)
COL_SLOPE = (60, 150, 92)
COL_BUMPER = (222, 84, 108)
COL_BUMPER_HI = (246, 140, 160)
COL_MILL = (186, 190, 198)
COL_MILL_D = (128, 134, 146)
COL_BALL = (248, 248, 244)
COL_CUP = (16, 22, 18)
COL_FLAG = (226, 72, 72)
COL_AIM = (245, 245, 210)
COL_LOCK = (248, 208, 96)    # Stärke-Sperre (Balken, Ring, Ziellinie)

# ------------------------------------------------ Farben der neuen Objekte
COL_TUNNEL = (156, 108, 210)     # Rohr: violett, damit es aus dem Grün sticht
COL_TUNNEL_D = (108, 68, 156)
COL_ICE = (176, 222, 240)        # Eis: helles Blauweiß
COL_ICE_D = (132, 186, 214)
COL_BOOST = (250, 176, 60)       # Schub: orange Pfeile
COL_BOOST_D = (206, 132, 26)
COL_MAGNET = (236, 96, 96)       # Magnet: rot (zieht) / blau (stößt ab)
COL_MAGNET_R = (96, 148, 236)
COL_GATE = (120, 200, 150)       # Einbahn-Tor: grünlich, mit Pfeil
COL_GATE_D = (70, 150, 104)
COL_STICKY = (128, 104, 66)      # Klebefeld: dunkles Braun, matschig
COL_STICKY_D = (96, 76, 46)
COL_SPIN = (214, 200, 120)       # Drehscheibe: sandgelb mit Speichen
COL_SPIN_D = (168, 152, 82)
COL_JUMP = (240, 232, 120)       # Sprungrampe: hellgelb wie eine Schanze
COL_JUMP_D = (196, 186, 62)

# Reihenfolge, in der gezeichnet wird. Flächen zuerst, Aufbauten zuletzt -
# so liegt nie eine Wand unter dem Sand.
DRAW_ORDER = ("slopes", "ice", "sticky", "sand", "boosters", "water",
              "magnets", "spinners", "tunnels", "walls", "gates", "movers",
              "bumpers", "jumps", "mills")


class View:
    """Nullpunkt und Maßstab: Bahn-Einheiten <-> Bildschirm-Pixel."""

    def __init__(self, ox, oy, scale):
        self.ox = float(ox)
        self.oy = float(oy)
        self.scale = float(scale)

    def project(self, x, y):
        return (self.ox + x * self.scale, self.oy + y * self.scale)

    def unproject(self, sx, sy):
        return ((sx - self.ox) / self.scale, (sy - self.oy) / self.scale)

    def rect_px(self, r):
        px, py = self.project(r[0], r[1])
        return pygame.Rect(int(px), int(py), max(1, int(r[2] * self.scale)),
                           max(1, int(r[3] * self.scale)))

    def board(self, cw=CW, ch=CH):
        """Das Rechteck des ganzen Platzes in Pixeln."""
        return pygame.Rect(int(self.ox), int(self.oy),
                           int(cw * self.scale), int(ch * self.scale))


def _tsec():
    return pygame.time.get_ticks() / 1000.0


# ---------------------------------------------------------------------------
#  Untergrund
# ---------------------------------------------------------------------------

def draw_ground(s, hole, view):
    """Grün, Mähstreifen und die vier Banden."""
    board = view.board(hole["w"], hole["h"])
    pygame.draw.rect(s, COL_FRINGE, board.inflate(10, 10), border_radius=10)
    pygame.draw.rect(s, COL_GREEN, board, border_radius=6)
    stripe = max(6, int(10 * view.scale))
    for i in range(0, board.h, stripe * 2):
        pygame.draw.rect(s, COL_GREEN_D,
                         (board.x, board.y + i, board.w,
                          min(stripe, board.h - i)))
    b = max(2, int(BORDER * view.scale))
    for r in (pygame.Rect(board.x, board.y, board.w, b),
              pygame.Rect(board.x, board.bottom - b, board.w, b),
              pygame.Rect(board.x, board.y, b, board.h),
              pygame.Rect(board.right - b, board.y, b, board.h)):
        pygame.draw.rect(s, COL_WALL, r)
        pygame.draw.rect(s, COL_WALL_HI, r, 1)
    return board


# ---------------------------------------------------------------------------
#  Die 15 Hindernis-Typen
# ---------------------------------------------------------------------------

def draw_slope(s, r, ax, ay, view):
    """Rampe: Fläche mit Pfeilfeld in Beschleunigungsrichtung."""
    rc = view.rect_px(r)
    pygame.draw.rect(s, COL_SLOPE, rc, border_radius=5)
    ang = math.atan2(ay, ax)
    step = max(16, int(14 * view.scale))
    col = ui.mix(COL_SLOPE, (255, 255, 255), 0.22)
    dx, dy = math.cos(ang) * 6, math.sin(ang) * 6
    for yy in range(rc.y + step // 2, rc.bottom - 2, step):
        for xx in range(rc.x + step // 2, rc.right - 2, step):
            pygame.draw.line(s, col, (xx - dx, yy - dy), (xx + dx, yy + dy), 2)
            pygame.draw.circle(s, col, (int(xx + dx), int(yy + dy)), 2)


def draw_sand(s, r, view):
    rc = view.rect_px(r)
    pygame.draw.rect(s, COL_SAND, rc, border_radius=6)
    pygame.draw.rect(s, COL_SAND_D, rc, 2, border_radius=6)


def draw_water(s, r, view):
    """Wasser: Fläche mit drei laufenden Wellenlinien."""
    rc = view.rect_px(r)
    pygame.draw.rect(s, COL_WATER, rc, border_radius=7)
    t = _tsec()
    for i in range(3):
        yy = rc.y + int(rc.h * (0.25 + 0.25 * i)) + int(math.sin(t * 1.4 + i) * 3)
        if rc.y < yy < rc.bottom:
            pygame.draw.line(s, COL_WATER_D, (rc.x + 6, yy), (rc.right - 6, yy), 2)
    pygame.draw.rect(s, COL_WATER_D, rc, 2, border_radius=7)


def draw_wall(s, rc, mover=False):
    pygame.draw.rect(s, COL_WALL, rc, border_radius=3)
    pygame.draw.rect(s, COL_WALL_HI, rc, 2, border_radius=3)
    if mover:
        pygame.draw.line(s, COL_WALL_HI, (rc.x + 4, rc.centery),
                         (rc.right - 4, rc.centery), 1)


def draw_bumper(s, b, view):
    x, y, r = b[0], b[1], b[2]
    px, py = view.project(x, y)
    rr = max(3, int(r * view.scale))
    pygame.draw.circle(s, COL_BUMPER, (int(px), int(py)), rr)
    pygame.draw.circle(s, COL_BUMPER_HI, (int(px), int(py)), max(2, rr - 3), 2)


def draw_mill(s, mill, view, mill_a):
    """Windmühle: rotierende Flügel um eine Nabe."""
    x, y, length, arms, speed = mill
    px, py = view.project(x, y)
    base = mill_a * speed
    w = max(3, int(3.0 * view.scale))
    for i in range(max(1, int(arms))):
        a = base + i * (2 * math.pi / max(1, int(arms)))
        ex, ey = view.project(x + math.cos(a) * length, y + math.sin(a) * length)
        pygame.draw.line(s, COL_MILL, (px, py), (ex, ey), w)
        pygame.draw.line(s, COL_MILL_D, (px, py), (ex, ey), 1)
    pygame.draw.circle(s, COL_MILL_D, (int(px), int(py)),
                       max(3, int(2.4 * view.scale)))


def draw_tunnel(s, tun, view):
    """Rohr: zwei Mündungen, dazwischen eine gestrichelte Verbindung."""
    x1, y1, x2, y2, r = tun
    ax, ay = view.project(x1, y1)
    bx, by = view.project(x2, y2)
    rr = max(4, int(r * view.scale))
    # Verbindung nur andeuten - sie ist kein Hindernis, sondern eine Erklärung.
    seg, gap = 9, 7
    dx, dy = bx - ax, by - ay
    dist = math.hypot(dx, dy)
    if dist > 1:
        ux, uy = dx / dist, dy / dist
        pos = rr
        while pos < dist - rr:
            e = min(pos + seg, dist - rr)
            pygame.draw.line(s, COL_TUNNEL_D, (ax + ux * pos, ay + uy * pos),
                             (ax + ux * e, ay + uy * e), 2)
            pos = e + gap
    puls = ui.pulse(2.2, 0.55, 1.0)
    for (px, py) in ((ax, ay), (bx, by)):
        pygame.draw.circle(s, COL_TUNNEL_D, (int(px), int(py)), rr)
        pygame.draw.circle(s, ui.mix(COL_TUNNEL, (255, 255, 255), 0.25 * puls),
                           (int(px), int(py)), max(2, rr - 3))
        pygame.draw.circle(s, COL_TUNNEL_D, (int(px), int(py)),
                           max(1, rr // 3))


def draw_ice(s, r, view):
    """Eis: helle Fläche mit ein paar Rissen."""
    rc = view.rect_px(r)
    pygame.draw.rect(s, COL_ICE, rc, border_radius=6)
    for i in range(3):
        yy = rc.y + rc.h * (i + 1) // 4
        pygame.draw.line(s, COL_ICE_D, (rc.x + 5, yy),
                         (rc.right - 5, yy - rc.h // 8), 1)
    pygame.draw.rect(s, COL_ICE_D, rc, 2, border_radius=6)


def draw_booster(s, bo, view):
    """Schub-Feld: Doppelpfeile in Schubrichtung."""
    x, y, w, h, dx, dy = bo[0], bo[1], bo[2], bo[3], bo[4], bo[5]
    rc = view.rect_px((x, y, w, h))
    pygame.draw.rect(s, COL_BOOST_D, rc, border_radius=5)
    pygame.draw.rect(s, COL_BOOST, rc.inflate(-4, -4), border_radius=4)
    ang = math.atan2(dy, dx)
    ux, uy = math.cos(ang), math.sin(ang)
    step = max(14, int(12 * view.scale))
    for k in range(2):
        off = (k - 0.5) * step * 0.7
        for yy in range(rc.y + step // 2, rc.bottom - 2, step):
            for xx in range(rc.x + step // 2, rc.right - 2, step):
                cx, cy = xx - uy * off, yy + ux * off
                tipx, tipy = cx + ux * 6, cy + uy * 6
                pygame.draw.polygon(s, COL_BOOST_D, [
                    (tipx, tipy),
                    (cx - ux * 4 - uy * 4, cy - uy * 4 + ux * 4),
                    (cx - ux * 4 + uy * 4, cy - uy * 4 - ux * 4)])


def draw_magnet(s, mag, view):
    """Magnet: Ringe um die Mitte, rot zieht an, blau stößt ab."""
    x, y, r, force = mag
    px, py = view.project(x, y)
    rr = max(5, int(r * view.scale))
    col = COL_MAGNET if force >= 0 else COL_MAGNET_R
    puls = ui.pulse(1.6, 0.35, 0.9)
    for i in range(3):
        rad = int(rr * (0.4 + 0.3 * i))
        ring = pygame.Surface((rad * 2 + 4, rad * 2 + 4), pygame.SRCALPHA)
        pygame.draw.circle(ring, (*col, int(150 * puls * (1 - i * 0.25))),
                           (rad + 2, rad + 2), rad, 2)
        s.blit(ring, (px - rad - 2, py - rad - 2))
    pygame.draw.circle(s, col, (int(px), int(py)), max(3, rr // 4))


def draw_gate(s, gt, view):
    """Einbahn-Tor: durchscheinende Fläche mit Pfeil in Durchlassrichtung."""
    x, y, w, h, dx, dy = gt
    rc = view.rect_px((x, y, w, h))
    face = pygame.Surface((rc.w, rc.h), pygame.SRCALPHA)
    face.fill((*COL_GATE, 120))
    s.blit(face, rc.topleft)
    pygame.draw.rect(s, COL_GATE_D, rc, 2, border_radius=3)
    ang = math.atan2(dy, dx)
    ux, uy = math.cos(ang), math.sin(ang)
    cx, cy = rc.centerx, rc.centery
    ln = max(6, int(min(rc.w, rc.h) * 0.35))
    tip = (cx + ux * ln, cy + uy * ln)
    pygame.draw.line(s, COL_GATE_D, (cx - ux * ln, cy - uy * ln), tip, 2)
    pygame.draw.polygon(s, COL_GATE_D, [
        tip, (tip[0] - ux * 6 - uy * 5, tip[1] - uy * 6 + ux * 5),
        (tip[0] - ux * 6 + uy * 5, tip[1] - uy * 6 - ux * 5)])


def draw_sticky(s, r, view):
    """Klebefeld: dunkle Fläche mit Blasen."""
    rc = view.rect_px(r)
    pygame.draw.rect(s, COL_STICKY, rc, border_radius=6)
    step = max(12, int(11 * view.scale))
    for yy in range(rc.y + step // 2, rc.bottom - 2, step):
        for xx in range(rc.x + step // 2, rc.right - 2, step):
            pygame.draw.circle(s, COL_STICKY_D, (xx, yy), 3)
    pygame.draw.rect(s, COL_STICKY_D, rc, 2, border_radius=6)


def draw_spinner(s, sp, view, mill_a):
    """Drehscheibe: Scheibe mit mitdrehenden Speichen."""
    x, y, r, speed = sp
    px, py = view.project(x, y)
    rr = max(5, int(r * view.scale))
    pygame.draw.circle(s, COL_SPIN_D, (int(px), int(py)), rr)
    pygame.draw.circle(s, COL_SPIN, (int(px), int(py)), max(2, rr - 3))
    base = mill_a * speed
    for i in range(4):
        a = base + i * (math.pi / 2)
        pygame.draw.line(s, COL_SPIN_D, (px, py),
                         (px + math.cos(a) * rr, py + math.sin(a) * rr), 2)
    pygame.draw.circle(s, COL_SPIN_D, (int(px), int(py)), max(2, rr // 5))


def draw_jump(s, jp, view):
    """Sprungrampe: Schanze mit Stufen quer zur Sprungrichtung."""
    x, y, w, h, dx, dy = jp[0], jp[1], jp[2], jp[3], jp[4], jp[5]
    rc = view.rect_px((x, y, w, h))
    pygame.draw.rect(s, COL_JUMP_D, rc, border_radius=4)
    pygame.draw.rect(s, COL_JUMP, rc.inflate(-4, -4), border_radius=3)
    ang = math.atan2(dy, dx)
    ux, uy = math.cos(ang), math.sin(ang)
    cx, cy = rc.centerx, rc.centery
    ln = max(5, int(min(rc.w, rc.h) * 0.42))
    for k in (-1, 0, 1):
        off = k * max(4, ln // 2)
        pygame.draw.line(s, COL_JUMP_D,
                         (cx - uy * ln + ux * off, cy + ux * ln + uy * off),
                         (cx + uy * ln + ux * off, cy - ux * ln + uy * off), 2)
    tip = (cx + ux * ln, cy + uy * ln)
    pygame.draw.polygon(s, COL_JUMP_D, [
        tip, (tip[0] - ux * 7 - uy * 5, tip[1] - uy * 7 + ux * 5),
        (tip[0] - ux * 7 + uy * 5, tip[1] - uy * 7 - ux * 5)])


def draw_cup(s, cup, view, cup_r):
    """Loch mit Fahne."""
    px, py = view.project(cup[0], cup[1])
    r = max(3, int(cup_r * view.scale))
    pygame.draw.circle(s, (24, 60, 36), (int(px), int(py)), r + 2)
    pygame.draw.circle(s, COL_CUP, (int(px), int(py)), r)
    top = py - max(16, int(18 * view.scale))
    pygame.draw.line(s, (238, 238, 232), (px, py), (px, top), 2)
    wave = math.sin(pygame.time.get_ticks() / 260.0) * 2
    pygame.draw.polygon(s, COL_FLAG, [(px, top), (px + 14, top + 5 + wave),
                                      (px, top + 11)])


def draw_tee(s, tee, view):
    """Abschlagpunkt - im Spiel liegt der Ball darauf, im Editor sieht man ihn."""
    px, py = view.project(tee[0], tee[1])
    r = max(3, int(2.6 * view.scale))
    pygame.draw.circle(s, ui.mix(COL_GREEN, (255, 255, 255), 0.45),
                       (int(px), int(py)), r, 2)
    pygame.draw.circle(s, ui.mix(COL_GREEN, (255, 255, 255), 0.7),
                       (int(px), int(py)), max(1, r // 3))


# ---------------------------------------------------------------------------
#  Alles zusammen
# ---------------------------------------------------------------------------

def mover_rect(m, move_t):
    """Aktuelles Rechteck eines Wanderblocks + seine Geschwindigkeit.

    Pendelt zwischen (x, y) und (x + dx, y + dy). Liegt hier, weil Physik
    (minigolf.py) und Zeichnen dieselbe Stelle brauchen.
    """
    x, y, w, h, dx, dy, speed = m
    if speed <= 0 or (dx == 0 and dy == 0):
        return (x, y, w, h), (0.0, 0.0)
    span = math.hypot(dx, dy)
    period = 2 * span / speed
    ph = (move_t % period) / period
    f = ph * 2 if ph < 0.5 else (1 - ph) * 2
    sign = 1.0 if ph < 0.5 else -1.0
    return ((x + dx * f, y + dy * f, w, h),
            (dx / span * speed * sign, dy / span * speed * sign))


def draw_obstacles(s, hole, view, mill_a=0.0, move_t=0.0):
    """Zeichnet alle Hindernisse in der richtigen Reihenfolge."""
    for key in DRAW_ORDER:
        items = hole.get(key) or ()
        if key == "slopes":
            for it in items:
                draw_slope(s, it[:4], it[4], it[5], view)
        elif key == "sand":
            for it in items:
                draw_sand(s, it, view)
        elif key == "water":
            for it in items:
                draw_water(s, it, view)
        elif key == "walls":
            for it in items:
                draw_wall(s, view.rect_px(it))
        elif key == "movers":
            for it in items:
                draw_wall(s, view.rect_px(mover_rect(it, move_t)[0]), mover=True)
        elif key == "bumpers":
            for it in items:
                draw_bumper(s, it, view)
        elif key == "mills":
            for it in items:
                draw_mill(s, it, view, mill_a)
        elif key == "tunnels":
            for it in items:
                draw_tunnel(s, it, view)
        elif key == "ice":
            for it in items:
                draw_ice(s, it, view)
        elif key == "boosters":
            for it in items:
                draw_booster(s, it, view)
        elif key == "magnets":
            for it in items:
                draw_magnet(s, it, view)
        elif key == "gates":
            for it in items:
                draw_gate(s, it, view)
        elif key == "sticky":
            for it in items:
                draw_sticky(s, it, view)
        elif key == "spinners":
            for it in items:
                draw_spinner(s, it, view, mill_a)
        elif key == "jumps":
            for it in items:
                draw_jump(s, it, view)


def draw_course(s, hole, view, mill_a=0.0, move_t=0.0, cup_r=3.0, tee=False):
    """Kompletter Platz: Untergrund, Loch, alle Hindernisse.

    Das Loch wird VOR den Aufbauten gezeichnet (so wie bisher), damit eine
    Wand davor auch davor liegt. 'tee' zeigt zusätzlich den Abschlagpunkt -
    im Spiel deckt ihn der Ball ab, im Editor braucht man ihn sichtbar.
    """
    draw_ground(s, hole, view)
    draw_cup(s, hole["cup"], view, cup_r)
    if tee:
        draw_tee(s, hole["tee"], view)
    draw_obstacles(s, hole, view, mill_a, move_t)
