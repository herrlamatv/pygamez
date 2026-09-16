# -*- coding: utf-8 -*-
"""
geodash_draw.py
===============
Zeichnen für Geometry Dash - Spiel UND Level-Editor benutzen dieselben
Funktionen, damit ein Level im Editor genauso aussieht wie beim Spielen.

Die Welt wird in Blöcken gerechnet: ``ts`` ist die Kachelgröße in Pixeln,
``cam_x``/``cam_y`` die Welt-Koordinate der linken unteren Bildschirmecke.
Weltachse y zeigt nach OBEN (Reihe 0 = direkt über dem Boden), der
Bildschirm nach unten - ``Renderer.sy`` rechnet um.

Teure Flächen werden je Kachelgröße gecacht: Blöcke (16 Kanten-Varianten,
damit zusammenhängende Blöcke wie EINE Fläche mit Außenkontur wirken),
Stacheln je Drehung, Leuchtscheiben für Orbs/Pads/Portale. Pro Frame wird
nur noch geblittet - so bleibt es auch bei 1280x960 flüssig.
"""

import math

import pygame

import ui

from . import geodash_core as core

# Farben der Objekte
COL_YELLOW = (255, 214, 40)
COL_PINK = (255, 90, 210)
COL_BLUE = (70, 200, 255)
COL_PAD = (COL_YELLOW, COL_PINK, COL_BLUE)
COL_MODE = {core.CUBE: (90, 255, 110), core.SHIP: (255, 110, 220),
            core.BALL: (255, 120, 50), core.UFO: (255, 190, 40),
            core.WAVE: (70, 170, 255)}
COL_GRAV = {1: (70, 200, 255), -1: (255, 225, 60)}
COL_SPEED = ((255, 170, 60), (90, 200, 255), (90, 255, 140), (255, 90, 200))
COL_COIN = (255, 205, 70)
COL_CHECK = (90, 255, 140)

# Spieler (Würfel: Grundfarbe + Zweitfarbe)
COL_P1 = (185, 242, 58)
COL_P2 = (58, 214, 242)

# Schwierigkeiten: Name-Schlüssel + Farbe des Gesichts
DIFF_COLORS = ((90, 200, 255), (110, 230, 90), (255, 200, 60),
               (255, 120, 60), (255, 70, 150), (220, 40, 50))


def mix(c1, c2, f):
    f = max(0.0, min(1.0, f))
    return (int(c1[0] + (c2[0] - c1[0]) * f), int(c1[1] + (c2[1] - c1[1]) * f),
            int(c1[2] + (c2[2] - c1[2]) * f))


def color_at(lv, xb, which):
    """Hintergrund- (which=0) bzw. Bodenfarbe (1) an Welt-x (Blöcke).

    Die Farb-Trigger wirken der Reihe nach: ab ihrer Position blenden sie
    über 'Dauer' Blöcke von der bisherigen zur neuen Farbe.
    """
    d = lv.data
    col = tuple(d["bg"] if which == 0 else d["ground"])
    for i in lv.triggers:
        tx = lv.ox[i]
        if tx > xb:
            break
        p = lv.par[i]
        if p[0] != which:
            continue
        f = (xb - tx) / float(max(1, p[4])) if p[4] > 0 else 1.0
        col = mix(col, (p[1], p[2], p[3]), f)
    return col


def _glow(radius, color, strength=1.0):
    """Weiche, runde Leuchtscheibe (SRCALPHA) - einmal bauen, dann blitten."""
    size = max(4, int(radius * 2))
    surf = pygame.Surface((size, size), pygame.SRCALPHA)
    c = size / 2.0
    steps = max(6, int(radius // 2))
    for i in range(steps, 0, -1):
        r = radius * i / steps
        a = int(90 * strength * (1.0 - i / steps) ** 1.6)
        pygame.draw.circle(surf, (color[0], color[1], color[2], a),
                           (int(c), int(c)), int(r))
    return surf


class Renderer:
    """Zeichnet Welt, Objekte und Spieler bei gegebener Kachelgröße."""

    def __init__(self):
        self.ts = 0
        self.w = self.h = 0
        self._cache = {}

    # ----- Größe ----------------------------------------------------------
    def resize(self, w, h, ts):
        ts = max(8, int(ts))
        if (w, h, ts) == (self.w, self.h, self.ts):
            return
        self.w, self.h, self.ts = w, h, ts
        self._cache.clear()

    def sx(self, xb, cam_x):
        return (xb - cam_x) * self.ts

    def sy(self, yb, cam_y):
        """Bildschirm-y der Welthöhe yb (Blöcke)."""
        return self.h - (yb - cam_y) * self.ts

    # ----- Caches --------------------------------------------------------
    def _block(self, mask, half=0):
        """Block-Kachel: dunkle Füllung, helle Kontur an freien Kanten.

        mask: 1 oben frei, 2 rechts, 4 unten, 8 links. half: 0 voll,
        sonst 1..4 = Halbblock unten/links/oben/rechts.
        """
        key = ("blk", mask, half)
        surf = self._cache.get(key)
        if surf is not None:
            return surf
        ts = self.ts
        surf = pygame.Surface((ts, ts), pygame.SRCALPHA)
        rect = pygame.Rect(0, 0, ts, ts)
        if half == 1:
            rect = pygame.Rect(0, ts // 2, ts, ts - ts // 2)
        elif half == 2:
            rect = pygame.Rect(0, 0, ts // 2, ts)
        elif half == 3:
            rect = pygame.Rect(0, 0, ts, ts // 2)
        elif half == 4:
            rect = pygame.Rect(ts // 2, 0, ts - ts // 2, ts)
        # Füllung mit leichtem Verlauf
        for i in range(rect.h):
            f = i / float(max(1, rect.h - 1))
            c = mix((34, 36, 58), (8, 8, 16), f)
            pygame.draw.line(surf, c + (238,), (rect.x, rect.y + i),
                             (rect.right - 1, rect.y + i))
        lw = max(1, ts // 14)
        line = (250, 252, 255)
        if half:
            pygame.draw.rect(surf, line, rect, lw)
        else:
            if mask & 1:
                pygame.draw.rect(surf, line, (0, 0, ts, lw))
            if mask & 2:
                pygame.draw.rect(surf, line, (ts - lw, 0, lw, ts))
            if mask & 4:
                pygame.draw.rect(surf, line, (0, ts - lw, ts, lw))
            if mask & 8:
                pygame.draw.rect(surf, line, (0, 0, lw, ts))
            # dezentes Innenmuster
            inset = max(3, ts // 4)
            inner = pygame.Rect(inset, inset, ts - 2 * inset, ts - 2 * inset)
            pygame.draw.rect(surf, (255, 255, 255, 26), inner, max(1, lw // 2))
        self._cache[key] = surf
        return surf

    def _spike(self, rot, small=False):
        key = ("spk", rot, small)
        surf = self._cache.get(key)
        if surf is not None:
            return surf
        ts = self.ts
        surf = pygame.Surface((ts, ts), pygame.SRCALPHA)
        top = ts * 0.5 if small else ts * 0.06
        pts = [(ts * 0.06, ts - 1), (ts * 0.94, ts - 1), (ts * 0.5, top)]
        lw = max(1, ts // 16)
        pygame.draw.polygon(surf, (12, 12, 22), pts)
        # innerer Glanz
        glint = [(ts * 0.5, top + (ts - top) * 0.28),
                 (ts * 0.34, ts - 3), (ts * 0.5, ts - 3)]
        pygame.draw.polygon(surf, (60, 64, 90), glint)
        pygame.draw.polygon(surf, (250, 252, 255), pts, lw)
        if rot:
            surf = pygame.transform.rotate(surf, -90 * rot)
        self._cache[key] = surf
        return surf

    def _glow(self, key, radius, color, strength=1.0):
        k = ("glow", key, int(radius), color, strength)
        surf = self._cache.get(k)
        if surf is None:
            surf = _glow(radius, color, strength)
            self._cache[k] = surf
        return surf

    def _shade(self):
        """Senkrechter Abdunkel-Verlauf über dem Hintergrund (gecacht)."""
        surf = self._cache.get("shade")
        if surf is None:
            surf = pygame.Surface((1, 64), pygame.SRCALPHA)
            for i in range(64):
                a = int(120 * (1.0 - i / 63.0) ** 1.4)
                surf.set_at((0, i), (0, 0, 20, a))
            surf = pygame.transform.smoothscale(surf, (max(1, self.w),
                                                       max(1, self.h)))
            self._cache["shade"] = surf
        return surf

    # ----- Hintergrund, Boden, Decke -----------------------------------------
    def draw_backdrop(self, s, bg, cam_x, cam_y, pulse=0.0):
        """Hintergrundfarbe mit Parallax-Quadraten und Beat-Puls."""
        col = mix(bg, (255, 255, 255), 0.06 * pulse)
        s.fill(mix(col, (0, 0, 0), 0.25))
        ts = self.ts
        # Große, langsam wandernde Quadrate (zwei Ebenen, dezent)
        for layer, (speed, size, alpha) in enumerate(((0.10, 4.6, 0.045),
                                                      (0.25, 2.2, 0.06))):
            px = size * ts
            span = px * 2.6
            off = (cam_x * speed * ts) % span
            n = int(self.w / span) + 2
            for i in range(-1, n):
                x = int(i * span - off)
                k = int(cam_x * speed * ts // span) + i
                yb = ((k * 37 + layer * 11) % 7) / 7.0
                y = int(self.h * (0.08 + 0.5 * yb) + cam_y * ts * speed)
                c = mix(bg, (255, 255, 255), alpha + 0.03 * pulse)
                pygame.draw.rect(s, c, (x, y, int(px), int(px)),
                                 max(1, ts // 12) if layer else 0,
                                 border_radius=max(2, ts // 6))
        s.blit(self._shade(), (0, 0))

    def draw_ground(self, s, lv, ground, cam_x, cam_y, pulse=0.0):
        """Boden ab Reihe 0 abwärts, mit Lücken über den Gruben."""
        ts = self.ts
        y0 = int(self.sy(0, cam_y))
        if y0 >= self.h:
            return
        dark = mix(ground, (0, 0, 0), 0.35)
        pygame.draw.rect(s, dark, (0, y0, self.w, self.h - y0))
        band = max(2, ts // 2)
        for i in range(band):
            pygame.draw.line(s, mix(ground, dark, i / float(band)),
                             (0, y0 + i), (self.w, y0 + i))
        # Fliesenfugen (alle 4 Blöcke)
        tile = 4 * ts
        off = int((cam_x * ts) % tile)
        fug = mix(dark, (255, 255, 255), 0.08)
        for i in range(-1, self.w // tile + 2):
            x = i * tile - off
            pygame.draw.line(s, fug, (x, y0 + band), (x, self.h))
        # Gruben: schwarze Löcher mit rotem Schimmer
        if lv is not None and lv.pits:
            c0 = int(cam_x) - 1
            c1 = int(cam_x + self.w / ts) + 1
            for c in range(c0, c1 + 1):
                if c in lv.pits:
                    x = int(self.sx(c, cam_x))
                    pygame.draw.rect(s, (4, 2, 8), (x, y0, ts + 1, self.h - y0))
                    for j in range(max(2, ts // 3)):
                        a = 1.0 - j / float(max(2, ts // 3))
                        pygame.draw.line(s, mix((4, 2, 8), (255, 60, 60), 0.35 * a),
                                         (x, self.h - 1 - j), (x + ts, self.h - 1 - j))
        glow = mix(ground, (255, 255, 255), 0.55 + 0.35 * pulse)
        lw = max(2, ts // 16)
        pygame.draw.rect(s, glow, (0, y0 - lw // 2, self.w, lw))

    def draw_ceiling(self, s, ground, ceil_b, cam_y, pulse=0.0, alpha=1.0):
        """Decke des Korridors (Schiff/Ball/UFO/Welle)."""
        if ceil_b <= 0 or alpha <= 0:
            return
        ts = self.ts
        y1 = int(self.sy(ceil_b, cam_y))
        if y1 <= 0:
            return
        dark = mix(ground, (0, 0, 0), 0.35)
        if alpha >= 0.99:
            pygame.draw.rect(s, dark, (0, 0, self.w, y1))
        else:
            ov = pygame.Surface((self.w, y1), pygame.SRCALPHA)
            ov.fill(dark + (int(255 * alpha),))
            s.blit(ov, (0, 0))
        lw = max(2, ts // 16)
        glow = mix(ground, (255, 255, 255), 0.55 + 0.35 * pulse)
        line = pygame.Surface((self.w, lw), pygame.SRCALPHA)
        line.fill(glow + (int(255 * alpha),))
        s.blit(line, (0, y1 - lw // 2))

    # ----- Objekte ----------------------------------------------------------
    def draw_objects(self, s, lv, cam_x, cam_y, t, pulse, used=(), editor=False,
                     masks=None, hide_coins=0, dim=None):
        """Alle sichtbaren Objekte. used: ausgelöste Objekte (Orbs/Münzen)."""
        ts = self.ts
        c0 = int(math.floor(cam_x)) - 2
        c1 = int(cam_x + self.w / float(ts)) + 2
        cols = lv.cache.get("draw_cols")
        if cols is None:
            cols = {}
            for i in range(lv.n):
                cols.setdefault(lv.ox[i], []).append(i)
            lv.cache["draw_cols"] = cols
        if masks is None:
            masks = block_masks(lv)
        glows = []
        portals = []
        for c in range(c0, c1 + 1):
            for i in cols.get(c, ()):
                k = lv.kind[i]
                name = core.KIND_NAMES[k]
                x = int(self.sx(lv.ox[i], cam_x))
                y = int(self.sy(lv.oy[i] + 1, cam_y))
                if y > self.h + 3 * ts or y < -4 * ts:
                    continue
                cls = lv.cls[i]
                if name == "block":
                    s.blit(self._block(masks.get(i, 15)), (x, y))
                elif name == "half":
                    s.blit(self._block(15, lv.rot[i] + 1), (x, y))
                elif cls == core.C_HAZARD:
                    s.blit(self._spike(lv.rot[i], name == "spike_s"), (x, y))
                elif cls == core.C_PAD:
                    self._pad(s, x, y, lv.val[i], lv.rot[i], t, i in used)
                elif cls == core.C_ORB:
                    glows.append((i, x, y))
                elif cls in (core.C_MODE, core.C_GRAV, core.C_SPEED):
                    portals.append((i, x, y))
                elif cls == core.C_COIN:
                    if i in used:
                        continue
                    self._coin(s, x + ts // 2, y + ts // 2, t, ts * 0.36)
                elif editor and cls == core.C_TRIGGER:
                    self._trigger_icon(s, x, y, lv.par[i])
                elif editor and cls == core.C_PIT:
                    y0 = int(self.sy(0, cam_y))
                    pygame.draw.rect(s, (255, 70, 70), (x + 2, y0 + 2, ts - 4,
                                                        max(3, ts // 6)))
        for i, x, y in glows:
            self._orb(s, x, y, lv.val[i], t, pulse, i in used)
        for i, x, y in portals:
            self._portal(s, x, y, lv.cls[i], lv.val[i], t)

    def _pad(self, s, x, y, which, rot, t, used):
        ts = self.ts
        col = COL_PAD[which]
        g = self._glow("pad", ts * 0.7, col, 0.9)
        cy = y + ts - ts // 8 if rot == 0 else y + ts // 8
        s.blit(g, (x + ts // 2 - g.get_width() // 2, cy - g.get_height() // 2))
        w = int(ts * 0.8)
        h = max(3, ts // 5)
        r = pygame.Rect(x + (ts - w) // 2, y + ts - h if rot == 0 else y, w, h)
        pygame.draw.ellipse(s, col if not used else mix(col, (255, 255, 255), 0.5),
                            r.inflate(0, h))
        pygame.draw.rect(s, (12, 12, 22), (r.x, y + ts - 2 if rot == 0 else y,
                                           r.w, 2))
        # aufsteigende Funken
        for k in range(3):
            ph = (t * 1.6 + k / 3.0) % 1.0
            px = x + ts * (0.25 + 0.25 * k)
            dy = ph * ts * 0.9
            py = (y + ts - h - dy) if rot == 0 else (y + h + dy)
            rr = max(1, int(ts * 0.05 * (1.0 - ph)))
            pygame.draw.circle(s, mix(col, (255, 255, 255), 0.4), (int(px), int(py)), rr)

    def _orb(self, s, x, y, which, t, pulse, used):
        ts = self.ts
        col = COL_PAD[which]
        cx, cy = x + ts // 2, y + ts // 2
        g = self._glow("orb", ts * 0.95, col, 1.0)
        s.blit(g, (cx - g.get_width() // 2, cy - g.get_height() // 2))
        r = int(ts * 0.34)
        pygame.draw.circle(s, mix(col, (255, 255, 255), 0.25), (cx, cy), r)
        pygame.draw.circle(s, (255, 255, 255), (cx, cy), r, max(1, ts // 14))
        pygame.draw.circle(s, mix(col, (255, 255, 255), 0.7),
                           (cx - r // 3, cy - r // 3), max(1, r // 3))
        # Ring pulsiert mit dem Takt (nach Benutzung: aufblühen)
        ring = r + int(ts * (0.12 + 0.18 * pulse))
        if used:
            ring = r + int(ts * 0.4)
        pygame.draw.circle(s, col, (cx, cy), ring, max(1, ts // 18))

    def _portal(self, s, x, y, cls, val, t):
        ts = self.ts
        cx = x + ts // 2
        cy = y + ts // 2
        if cls == core.C_SPEED:
            col = COL_SPEED[val]
            g = self._glow("spd", ts * 1.2, col, 0.7)
            s.blit(g, (cx - g.get_width() // 2, cy - g.get_height() // 2))
            n = val + 1
            wv = ts * 0.34
            for i in range(n):
                ox = cx + (i - (n - 1) / 2.0) * wv * 0.8
                pts = [(ox - wv * 0.5, cy - ts * 0.55), (ox + wv * 0.4, cy),
                       (ox - wv * 0.5, cy + ts * 0.55), (ox - wv * 0.05, cy)]
                pygame.draw.polygon(s, col, pts)
                pygame.draw.polygon(s, (255, 255, 255), pts, max(1, ts // 20))
            return
        col = COL_MODE[val] if cls == core.C_MODE else COL_GRAV[val]
        g = self._glow("prt", ts * 1.6, col, 0.8)
        s.blit(g, (cx - g.get_width() // 2, cy - g.get_height() // 2))
        rect = pygame.Rect(0, 0, int(ts * 0.9), int(ts * 3.0))
        rect.center = (cx, cy)
        wob = 0.08 * math.sin(t * 4.0)
        inner = rect.inflate(-int(ts * (0.35 + wob)), -int(ts * 0.5))
        pygame.draw.ellipse(s, (10, 10, 20), rect)
        pygame.draw.ellipse(s, mix(col, (0, 0, 0), 0.45), inner)
        pygame.draw.ellipse(s, col, rect, max(2, ts // 8))
        pygame.draw.ellipse(s, (255, 255, 255), rect.inflate(-max(2, ts // 5), -max(2, ts // 5)),
                            max(1, ts // 20))
        # Sinnbild in der Mitte
        r = max(3, int(ts * 0.18))
        if cls == core.C_GRAV:
            d = -1 if val > 0 else 1
            pygame.draw.polygon(s, (255, 255, 255), [(cx - r, cy - d * r * 0.4),
                                                      (cx + r, cy - d * r * 0.4),
                                                      (cx, cy + d * r)])
        else:
            self._mode_glyph(s, cx, cy, r, val)

    def _mode_glyph(self, s, cx, cy, r, mode):
        white = (255, 255, 255)
        if mode == core.CUBE:
            pygame.draw.rect(s, white, (cx - r, cy - r, 2 * r, 2 * r), max(1, r // 3))
        elif mode == core.SHIP:
            pygame.draw.polygon(s, white, [(cx - r, cy + r * 0.6), (cx + r, cy + r * 0.2),
                                           (cx - r * 0.4, cy - r * 0.8)], max(1, r // 3))
        elif mode == core.BALL:
            pygame.draw.circle(s, white, (cx, cy), r, max(1, r // 3))
        elif mode == core.UFO:
            pygame.draw.ellipse(s, white, (cx - r, cy - r * 0.2, 2 * r, r), max(1, r // 3))
            pygame.draw.arc(s, white, (cx - r * 0.5, cy - r * 0.8, r, r * 1.2), 0, math.pi,
                            max(1, r // 3))
        else:
            pygame.draw.lines(s, white, False, [(cx - r, cy + r * 0.5), (cx - r * 0.3, cy - r * 0.5),
                                                (cx + r * 0.3, cy + r * 0.5), (cx + r, cy - r * 0.5)],
                              max(1, r // 3))

    def _coin(self, s, cx, cy, t, r):
        g = self._glow("coin", self.ts * 0.9, COL_COIN, 0.8)
        s.blit(g, (int(cx) - g.get_width() // 2, int(cy) - g.get_height() // 2))
        draw_coin(s, cx, cy, r, t)

    def _trigger_icon(self, s, x, y, par):
        ts = self.ts
        col = (par[1], par[2], par[3])
        r = pygame.Rect(x + ts // 6, y + ts // 6, ts - ts // 3, ts - ts // 3)
        pygame.draw.rect(s, col, r, border_radius=max(2, ts // 6))
        pygame.draw.rect(s, (255, 255, 255), r, max(1, ts // 16),
                         border_radius=max(2, ts // 6))
        f = ui.font(max(9, ts // 3), bold=True)
        img = f.render("BG" if par[0] == 0 else "G", True, (255, 255, 255))
        s.blit(img, img.get_rect(center=r.center))

    # ----- Spieler ---------------------------------------------------------------
    def draw_player(self, s, x, y, mode, grav, angle, cam_x, cam_y, t, trail=None,
                    held=False, alpha=255):
        """Spieler an Welt-Position (x, y = linke untere Ecke, in Blöcken)."""
        ts = self.ts
        cx = self.sx(x + 0.5, cam_x)
        cy = self.sy(y + 0.5, cam_y)
        if trail and len(trail) > 1:
            pts = [(self.sx(px + 0.5, cam_x), self.sy(py + 0.5, cam_y)) for px, py in trail]
            if mode == core.WAVE:
                w = max(2, ts // 6)
                pygame.draw.lines(s, mix(COL_P2, (255, 255, 255), 0.2), False, pts, w + 4)
                pygame.draw.lines(s, (255, 255, 255), False, pts, max(1, w // 2))
            else:
                for i, (px, py) in enumerate(pts[:-1]):
                    f = i / float(len(pts))
                    rr = max(1, int(ts * 0.14 * f))
                    pygame.draw.circle(s, mix(COL_P2, (255, 255, 255), f), (int(px), int(py)), rr)
        key = ("pl", mode, int(angle) // 3 * 3 if mode != core.WAVE else int(angle), grav)
        surf = self._cache.get(key)
        if surf is None:
            surf = self._player_surface(mode, angle, grav)
            if len(self._cache) > 900:
                for k in [k for k in self._cache if k[0] == "pl"]:
                    del self._cache[k]
            self._cache[key] = surf
        if alpha < 255:
            surf = surf.copy()
            surf.set_alpha(alpha)
        s.blit(surf, surf.get_rect(center=(int(cx), int(cy))))

    def _player_surface(self, mode, angle, grav):
        ts = self.ts
        big = ts * 2
        surf = pygame.Surface((big, big), pygame.SRCALPHA)
        c = big // 2
        lw = max(1, ts // 14)
        dark = (14, 14, 24)
        if mode == core.CUBE:
            self._cube_face(surf, c, c, int(ts * 0.96))
            surf = pygame.transform.rotate(surf, -angle)
        elif mode == core.BALL:
            r = int(ts * 0.48)
            pygame.draw.circle(surf, dark, (c, c), r)
            pygame.draw.circle(surf, COL_P1, (c, c), r - lw)
            for k in range(4):
                a = math.radians(k * 90)
                pygame.draw.line(surf, dark, (c, c), (c + math.cos(a) * r, c + math.sin(a) * r), lw + 1)
            pygame.draw.circle(surf, COL_P2, (c, c), int(r * 0.45))
            pygame.draw.circle(surf, dark, (c, c), int(r * 0.45), lw)
            pygame.draw.circle(surf, dark, (c, c), r, lw)
            surf = pygame.transform.rotate(surf, -angle)
        elif mode == core.SHIP:
            # kleiner Würfel im Cockpit, darunter der Rumpf
            self._cube_face(surf, c + int(ts * 0.02), c - int(ts * 0.22), int(ts * 0.5))
            hull = [(c - ts * 0.55, c - ts * 0.02), (c + ts * 0.62, c + ts * 0.12),
                    (c + ts * 0.3, c + ts * 0.42), (c - ts * 0.5, c + ts * 0.42)]
            pygame.draw.polygon(surf, COL_P2, hull)
            pygame.draw.polygon(surf, dark, hull, lw + 1)
            pygame.draw.line(surf, COL_P1, (c - ts * 0.4, c + ts * 0.2), (c + ts * 0.35, c + ts * 0.24), lw + 1)
            if grav < 0:
                surf = pygame.transform.flip(surf, False, True)
            surf = pygame.transform.rotate(surf, angle)
        elif mode == core.UFO:
            dome = pygame.Rect(0, 0, int(ts * 0.7), int(ts * 0.62))
            dome.midbottom = (c, c + int(ts * 0.08))
            pygame.draw.ellipse(surf, (200, 240, 255, 90), dome)
            self._cube_face(surf, c, c - int(ts * 0.14), int(ts * 0.42))
            pygame.draw.ellipse(surf, dark, dome, lw)
            saucer = pygame.Rect(0, 0, int(ts * 1.1), int(ts * 0.36))
            saucer.center = (c, c + int(ts * 0.18))
            pygame.draw.ellipse(surf, COL_P2, saucer)
            pygame.draw.ellipse(surf, dark, saucer, lw + 1)
            for k in (-1, 0, 1):
                pygame.draw.circle(surf, COL_P1, (c + int(k * ts * 0.28), saucer.centery), max(1, ts // 16))
            if grav < 0:
                surf = pygame.transform.flip(surf, False, True)
        else:
            # Welle: Pfeilspitze in Flugrichtung
            pts = [(c + ts * 0.42, c), (c - ts * 0.34, c - ts * 0.3), (c - ts * 0.16, c),
                   (c - ts * 0.34, c + ts * 0.3)]
            pygame.draw.polygon(surf, COL_P1, pts)
            pygame.draw.polygon(surf, dark, pts, lw + 1)
            pygame.draw.circle(surf, COL_P2, (int(c - ts * 0.02), c), max(2, ts // 9))
            surf = pygame.transform.rotate(surf, angle)
        return surf

    def _cube_face(self, surf, cx, cy, size):
        """Das Lama-Würfelgesicht (Grundfarbe außen, Zweitfarbe innen)."""
        dark = (14, 14, 24)
        lw = max(1, size // 14)
        r = pygame.Rect(0, 0, size, size)
        r.center = (cx, cy)
        pygame.draw.rect(surf, COL_P1, r)
        inner = r.inflate(-size // 3, -size // 3)
        pygame.draw.rect(surf, COL_P2, inner)
        pygame.draw.rect(surf, dark, inner, lw)
        eye = max(2, size // 7)
        pygame.draw.rect(surf, dark, (inner.x + inner.w // 4 - eye // 2, inner.y + inner.h // 3 - eye // 2, eye, eye))
        pygame.draw.rect(surf, dark, (inner.right - inner.w // 4 - eye // 2, inner.y + inner.h // 3 - eye // 2, eye, eye))
        pygame.draw.rect(surf, dark, (inner.x + inner.w // 4, inner.bottom - inner.h // 3, inner.w // 2, max(1, lw)))
        # Lama-Ohren
        ear = max(2, size // 6)
        pygame.draw.rect(surf, COL_P2, (r.x + size // 8, r.y + 2, ear, ear))
        pygame.draw.rect(surf, COL_P2, (r.right - size // 8 - ear, r.y + 2, ear, ear))
        pygame.draw.rect(surf, dark, r, lw + 1)


def block_masks(lv):
    """Kanten-Masken aller Blöcke (freie Seiten bekommen eine Kontur)."""
    masks = lv.cache.get("masks")
    if masks is not None:
        return masks
    cells = set()
    for i in range(lv.n):
        if core.KIND_NAMES[lv.kind[i]] == "block":
            cells.add((lv.ox[i], lv.oy[i]))
    masks = {}
    for i in range(lv.n):
        if core.KIND_NAMES[lv.kind[i]] != "block":
            continue
        x, y = lv.ox[i], lv.oy[i]
        m = 0
        if (x, y + 1) not in cells:
            m |= 1
        if (x + 1, y) not in cells:
            m |= 2
        if (x, y - 1) not in cells and y > 0:
            m |= 4
        if (x - 1, y) not in cells:
            m |= 8
        masks[i] = m
    lv.cache["masks"] = masks
    return masks


def draw_coin(s, cx, cy, r, t, dim=False):
    """Goldmünze mit Dreh-Glanz (auch für HUD/Level-Auswahl)."""
    squash = abs(math.cos(t * 2.4))
    w = max(2, int(2 * r * (0.35 + 0.65 * squash)))
    h = max(2, int(2 * r))
    rect = pygame.Rect(0, 0, w, h)
    rect.center = (int(cx), int(cy))
    base = (120, 110, 90) if dim else COL_COIN
    pygame.draw.ellipse(s, mix(base, (0, 0, 0), 0.35), rect.move(max(1, int(r // 6)), 0))
    pygame.draw.ellipse(s, base, rect)
    pygame.draw.ellipse(s, mix(base, (255, 255, 255), 0.45), rect.inflate(-max(2, w // 3), -max(2, h // 3)),
                        max(1, int(r // 5)))
    pygame.draw.ellipse(s, (60, 40, 10), rect, max(1, int(r // 6)))


def draw_star(s, cx, cy, r, col, outline=(20, 20, 30)):
    pts = []
    for k in range(10):
        a = -math.pi / 2 + k * math.pi / 5
        rr = r if k % 2 == 0 else r * 0.45
        pts.append((cx + math.cos(a) * rr, cy + math.sin(a) * rr))
    pygame.draw.polygon(s, col, pts)
    pygame.draw.polygon(s, outline, pts, max(1, int(r // 6)))


def draw_face(s, cx, cy, r, diff, t=0.0):
    """Schwierigkeits-Gesicht (0 leicht .. 5 Dämon) - gezeichnet, nicht Text."""
    col = DIFF_COLORS[max(0, min(5, diff))]
    dark = (18, 14, 24)
    lw = max(1, int(r // 8))
    cx, cy, r = int(cx), int(cy), int(r)
    if diff >= 5:
        # Hörner und Flammen-Rand
        for sgn in (-1, 1):
            pygame.draw.polygon(s, (240, 60, 40), [(cx + sgn * r * 0.55, cy - r * 0.55),
                                                    (cx + sgn * r * 1.05, cy - r * 1.25),
                                                    (cx + sgn * r * 0.2, cy - r * 0.85)])
            pygame.draw.polygon(s, dark, [(cx + sgn * r * 0.55, cy - r * 0.55),
                                          (cx + sgn * r * 1.05, cy - r * 1.25),
                                          (cx + sgn * r * 0.2, cy - r * 0.85)], lw)
    pygame.draw.circle(s, col, (cx, cy), r)
    pygame.draw.circle(s, mix(col, (255, 255, 255), 0.35), (cx - r // 3, cy - r // 3), max(1, r // 4))
    pygame.draw.circle(s, dark, (cx, cy), r, lw + 1)
    ey = cy - r // 5
    ex = int(r * 0.38)
    er = max(1, r // 6)
    if diff <= 1:
        pygame.draw.circle(s, dark, (cx - ex, ey), er)
        pygame.draw.circle(s, dark, (cx + ex, ey), er)
        pygame.draw.arc(s, dark, (cx - r // 2, cy - r // 4, r, int(r * 0.8)), math.pi * 1.1, math.pi * 1.9,
                        lw + 1)
    elif diff == 2:
        pygame.draw.circle(s, dark, (cx - ex, ey), er)
        pygame.draw.circle(s, dark, (cx + ex, ey), er)
        pygame.draw.line(s, dark, (cx - r // 3, cy + r // 3), (cx + r // 3, cy + r // 3), lw + 1)
    elif diff == 3:
        for sgn in (-1, 1):
            pygame.draw.line(s, dark, (cx + sgn * ex - er * 2, ey - er * 2 * sgn * -1),
                             (cx + sgn * ex + er * 2, ey + er * 2 * sgn * -1), lw + 1)
            pygame.draw.circle(s, dark, (cx + sgn * ex, ey + er), er)
        pygame.draw.arc(s, dark, (cx - r // 3, cy + r // 5, int(r * 0.66), int(r * 0.6)), 0.15, math.pi - 0.15,
                        lw + 1)
    else:
        for sgn in (-1, 1):
            pygame.draw.polygon(s, (255, 255, 255) if diff == 5 else dark,
                                [(cx + sgn * ex - er * 2, ey - er), (cx + sgn * ex + er * 2, ey - er),
                                 (cx + sgn * (ex - er * 2 * sgn), ey + er * 2)])
            pygame.draw.line(s, dark, (cx + sgn * (ex + er * 3), ey - er * 3),
                             (cx + sgn * (ex - er * 2), ey - er), lw + 1)
        mouth = pygame.Rect(cx - r // 2, cy + r // 5, r, max(3, r // 3))
        pygame.draw.rect(s, dark, mouth, border_radius=max(1, r // 8))
        for k in range(1, 4):
            x = mouth.x + mouth.w * k // 4
            pygame.draw.line(s, (255, 255, 255), (x, mouth.y), (x, mouth.bottom - 1), max(1, lw // 2))
