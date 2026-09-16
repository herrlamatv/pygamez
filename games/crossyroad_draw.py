# -*- coding: utf-8 -*-
"""
crossyroad_draw.py
==================
Voxel-Renderer für Crossy Road.

Projektion (schräg, "isometrisch" wirkend): Reihen laufen waagerecht, jede
Reihe weiter hinten liegt ein Stück höher UND etwas weiter rechts. Dadurch
sieht man von jedem Quader genau drei Flächen - Deckel, Vorderseite und
rechte Seite -, die unterschiedlich hell schattiert werden:

    bild_x = x * tw + y * sk
    bild_y = -y * th - z * zh          (x Spalte, y Reihe, z Höhe)

Alle vier Maße sind ganze Pixel, damit benachbarte Reihen nahtlos
aneinanderstoßen. Teuer ist nur das erste Zeichnen: Jedes Modell wird einmal
je Kachelgröße als Sprite (mit eingebackenem Schatten) gerendert, jede
Boden-Variante einmal als Streifen. Im Spiel sind das dann reine Blits.
"""

import pygame

from . import crossyroad_world as cw

MV = 7                         # Streifen reichen so viele Spalten über das Feld hinaus
SHADOW_ALPHA = 64
DIM = (30, 40, 66)             # Abdunkelung außerhalb des Spielfelds

# Bodenfarben
GRASS_A = (160, 222, 98)
GRASS_B = (150, 212, 90)
GRASS_SIDE = (112, 160, 64)
ROAD = (86, 90, 104)
ROAD_SIDE = (62, 64, 76)
ROAD_LINE = (226, 228, 222)
WATER_A = (96, 198, 248)
WATER_B = (88, 190, 242)
WAVE = (150, 224, 255)
RAIL_BED = (134, 122, 118)
RAIL_SIDE = (104, 94, 92)
SLEEPER = (116, 84, 62)
STEEL = (198, 202, 214)
STEEL_SIDE = (124, 128, 142)


def shade(col, f):
    return (min(255, int(col[0] * f)), min(255, int(col[1] * f)),
            min(255, int(col[2] * f)))


def dim(col, f=0.4):
    return (int(col[0] + (DIM[0] - col[0]) * f), int(col[1] + (DIM[1] - col[1]) * f),
            int(col[2] + (DIM[2] - col[2]) * f))


class Voxel:
    """Projektion + Caches für EINE Kachelgröße."""

    def __init__(self, tw, shadows=True):
        self.tw = max(12, int(tw))
        self.th = max(8, int(round(self.tw * 0.78)))
        self.sk = max(2, int(round(self.tw * 0.28)))
        self.zh = max(6, int(round(self.tw * 0.66)))
        self.shadows = shadows
        self._sprites = {}
        self._strips = {}
        self._scaled = {}

    # ------------------------------------------------------------ Projektion
    def proj(self, x, y, z):
        return (x * self.tw + y * self.sk, -y * self.th - z * self.zh)

    def _depth(self, b):
        cx = (b[0] + b[3]) * 0.5
        cy = (b[1] + b[4]) * 0.5
        cz = (b[2] + b[5]) * 0.5
        return -cx * self.sk / self.tw + cy - cz * self.th / self.zh

    def _rect(self, b):
        xs, ys = [], []
        for x in (b[0], b[3]):
            for y in (b[1], b[4]):
                for z in (b[2], b[5]):
                    px, py = self.proj(x, y, z)
                    xs.append(px)
                    ys.append(py)
        return min(xs), min(ys), max(xs), max(ys)

    def order(self, boxes):
        """Zeichenreihenfolge (hinten -> vorn) als Indexliste.

        Für zwei Quader, deren Bild sich überlappt, entscheidet die erste
        trennende Achse: wer weiter hinten (y), weiter links (x) oder tiefer
        (z) liegt, kommt zuerst. Bei dieser Projektion ist das für
        überlappende Paare immer richtig; die Gesamtreihenfolge ergibt sich
        per topologischer Sortierung (Gleichstand: weiter entfernt zuerst).
        """
        n = len(boxes)
        eps = 1e-6
        rects = [self._rect(b) for b in boxes]
        depth = [self._depth(b) for b in boxes]
        deps = [[] for _ in range(n)]
        for i in range(n):
            ra = rects[i]
            for j in range(i + 1, n):
                rb = rects[j]
                if ra[2] <= rb[0] or rb[2] <= ra[0] or ra[3] <= rb[1] or rb[3] <= ra[1]:
                    continue
                a, b = boxes[i], boxes[j]
                if a[1] >= b[4] - eps:
                    first = i
                elif b[1] >= a[4] - eps:
                    first = j
                elif a[3] <= b[0] + eps:
                    first = i
                elif b[3] <= a[0] + eps:
                    first = j
                elif a[5] <= b[2] + eps:
                    first = i
                elif b[5] <= a[2] + eps:
                    first = j
                else:
                    first = i if depth[i] >= depth[j] else j
                second = j if first == i else i
                deps[second].append(first)
        done = [False] * n
        out = []
        while len(out) < n:
            best = -1
            for k in range(n):
                if done[k] or not all(done[d] for d in deps[k]):
                    continue
                if best < 0 or depth[k] > depth[best]:
                    best = k
            if best < 0:                      # Zyklus: einfach nach Tiefe
                for k in range(n):
                    if not done[k] and (best < 0 or depth[k] > depth[best]):
                        best = k
            done[best] = True
            out.append(best)
        return out

    # ------------------------------------------------------------ Sprites
    def render(self, boxes, shadow=True):
        """Rendert Quader zu (surface, ax, ay); (ax, ay) = Bildpunkt von (0,0,0)."""
        pts = []
        for b in boxes:
            r = self._rect(b)
            pts.append((r[0], r[1]))
            pts.append((r[2], r[3]))
        shadow_polys = []
        if shadow and self.shadows:
            for b in boxes:
                if b[2] > 0.6:
                    continue
                q = [self.proj(b[0] + 0.1, b[1] - 0.02, 0), self.proj(b[3] + 0.18, b[1] - 0.02, 0),
                     self.proj(b[3] + 0.18, b[4] - 0.1, 0), self.proj(b[0] + 0.1, b[4] - 0.1, 0)]
                shadow_polys.append(q)
                pts.extend(q)
        minx = min(p[0] for p in pts)
        maxx = max(p[0] for p in pts)
        miny = min(p[1] for p in pts)
        maxy = max(p[1] for p in pts)
        ax = int(-minx) + 2
        ay = int(-miny) + 2
        w = int(maxx - minx) + 5
        h = int(maxy - miny) + 5
        surf = pygame.Surface((w, h), pygame.SRCALPHA)
        if shadow_polys:
            layer = pygame.Surface((w, h), pygame.SRCALPHA)
            for q in shadow_polys:
                pygame.draw.polygon(layer, (0, 0, 0, 255),
                                    [(round(ax + p[0]), round(ay + p[1])) for p in q])
            layer.fill((255, 255, 255, SHADOW_ALPHA), special_flags=pygame.BLEND_RGBA_MULT)
            surf.blit(layer, (0, 0))
        for i in self.order(boxes):
            self._box(surf, boxes[i], ax, ay)
        return surf, ax, ay

    def _box(self, surf, b, ax, ay):
        x0, y0, z0, x1, y1, z1, col = b
        alpha = col[3] if len(col) > 3 else 255

        def p(x, y, z):
            px, py = self.proj(x, y, z)
            return (round(ax + px), round(ay + py))
        east = shade(col, 0.70)
        front = shade(col, 0.86)
        if alpha < 255:
            east, front, top = east + (alpha,), front + (alpha,), tuple(col[:3]) + (alpha,)
        else:
            top = tuple(col[:3])
        pygame.draw.polygon(surf, east, [p(x1, y0, z0), p(x1, y1, z0), p(x1, y1, z1), p(x1, y0, z1)])
        pygame.draw.polygon(surf, front, [p(x0, y0, z0), p(x1, y0, z0), p(x1, y0, z1), p(x0, y0, z1)])
        pygame.draw.polygon(surf, top, [p(x0, y0, z1), p(x1, y0, z1), p(x1, y1, z1), p(x0, y1, z1)])

    def sprite(self, key, shadow=True):
        """Gecachtes Sprite eines Modells aus crossyroad_world.

        Schlüssel: "tree1", "car3", "car3<", "log2", "chicken:up", ...
        ("<" = nach links fahrend/gespiegelt).
        """
        got = self._sprites.get(key)
        if got is None:
            got = self._sprites[key] = self.render(model_for(key), shadow)
        return got

    def scaled(self, key, sx, sy):
        """Sprite gestaucht/gestreckt (Squash & Stretch), leicht gecacht."""
        sx = round(sx, 2)
        sy = round(sy, 2)
        if sx == 1.0 and sy == 1.0:
            return self.sprite(key, False)
        ck = (key, sx, sy)
        got = self._scaled.get(ck)
        if got is None:
            surf, ax, ay = self.sprite(key, False)
            w = max(1, int(surf.get_width() * sx))
            h = max(1, int(surf.get_height() * sy))
            got = (pygame.transform.smoothscale(surf, (w, h)), ax * sx, ay * sy)
            if len(self._scaled) > 240:
                self._scaled.clear()
            self._scaled[ck] = got
        return got

    # ------------------------------------------------------------ Boden
    def strip(self, kind, parity, lower, line):
        """Boden-Streifen einer Reihe als (surface, ax, ay).

        lower = Bodenhöhe der Reihe davor, falls tiefer (-> sichtbare Kante),
        line = gestrichelte Mittellinie zur nächsten Straßenreihe.
        """
        key = (kind, parity, lower, line)
        got = self._strips.get(key)
        if got is not None:
            return got
        lvl = cw.LEVEL[kind]
        c0, c1 = -MV, cw.COLS + MV
        left = c0 * self.tw - 2
        right = c1 * self.tw + self.sk + 2
        # Oben/unten KEIN Rand: der Streifen der nächsten (vorderen) Reihe
        # würde sonst mit seiner Hintergrundfarbe die Räder der Fahrzeuge
        # dieser Reihe überpinseln.
        top = round(-self.th - lvl * self.zh)
        bottom = round(-(lower if lower is not None else lvl) * self.zh)
        ax, ay = int(-left), int(-top)
        w, h = int(right - left), int(bottom - top) + 1
        surf = pygame.Surface((w, h)).convert() if pygame.display.get_surface() else pygame.Surface((w, h))

        def p(x, y, z):
            px, py = self.proj(x, y, z)
            return (round(ax + px), round(ay + py))

        def poly(col, pts):
            pygame.draw.polygon(surf, col, pts)

        base = {"grass": GRASS_A if parity else GRASS_B, "road": ROAD,
                "river": WATER_A if parity else WATER_B, "rail": RAIL_BED}[kind]
        side = {"grass": GRASS_SIDE, "road": ROAD_SIDE, "river": WATER_B,
                "rail": RAIL_SIDE}[kind]
        surf.fill(dim(base, 0.6))
        for c in range(c0, c1):
            out = c < 0 or c >= cw.COLS
            f = 0.42 if out else 0.0
            col = dim(base, f) if out else base
            if kind == "grass" and not out and (c + parity) % 2 == 0:
                col = shade(col, 1.025)
            poly(col, [p(c, 0, lvl), p(c + 1, 0, lvl), p(c + 1, 1, lvl), p(c, 1, lvl)])
            if lower is not None:
                sc = dim(side, f) if out else side
                poly(sc, [p(c, 0, lower), p(c + 1, 0, lower), p(c + 1, 0, lvl), p(c, 0, lvl)])
            if kind == "river":
                h = cw.deco_hash(parity * 7 + 3, c)
                wx = c + (h % 60) / 100.0
                wy = 0.2 + ((h >> 8) % 55) / 100.0
                wc = dim(WAVE, f) if out else WAVE
                poly(wc, [p(wx, wy, lvl), p(wx + 0.3, wy, lvl), p(wx + 0.3, wy + 0.05, lvl),
                          p(wx, wy + 0.05, lvl)])
            elif kind == "road" and line:
                lc = dim(ROAD_LINE, f) if out else ROAD_LINE
                poly(lc, [p(c + 0.25, 0.93, lvl), p(c + 0.75, 0.93, lvl), p(c + 0.75, 1.0, lvl),
                          p(c + 0.25, 1.0, lvl)])
            elif kind == "rail":
                sl = dim(SLEEPER, f) if out else SLEEPER
                for sx0 in (0.1, 0.6):
                    poly(sl, [p(c + sx0, 0.14, lvl), p(c + sx0 + 0.28, 0.14, lvl),
                              p(c + sx0 + 0.28, 0.86, lvl), p(c + sx0, 0.86, lvl)])
        if kind == "rail":
            st, ss = STEEL, STEEL_SIDE
            for ry in (0.26, 0.66):
                z1 = lvl + 0.05
                poly(ss, [p(c0, ry, lvl), p(c1, ry, lvl), p(c1, ry, z1), p(c0, ry, z1)])
                poly(st, [p(c0, ry, z1), p(c1, ry, z1), p(c1, ry + 0.07, z1), p(c0, ry + 0.07, z1)])
        got = self._strips[key] = (surf, ax, ay)
        return got


def model_for(key):
    """Quader-Liste zu einem Sprite-Schlüssel."""
    if ":" in key:
        cid, facing = key.split(":")
        return cw.rotate(cw.CHAR_MODELS[cid], facing)
    flip = key.endswith("<")
    name = key[:-1] if flip else key
    boxes = cw.MODELS[name]
    if flip:
        length = 1.0
        if name.startswith("car"):
            length = cw.CAR_LEN
        elif name.startswith("truck"):
            length = cw.TRUCK_LEN
        elif name in ("engine", "wagon"):
            length = cw.TRAIN_CAR
        boxes = cw.mirror_x(boxes, length)
    return boxes
