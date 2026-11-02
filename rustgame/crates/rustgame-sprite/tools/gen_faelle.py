"""Erzeugt die Falltabellen fuer den Abgleich Rust <-> echtes pygame.

Aufruf (aus dem Wurzelverzeichnis des Projekts):

    set SDL_VIDEODRIVER=dummy
    .venv\Scripts\python.exe rustgame/crates/rustgame-sprite/tools/gen_faelle.py \
        rustgame/crates/rustgame-sprite/tests/daten

Drei Dateien entstehen:

  collide.txt   Zufallsfaelle fuer collide_rect/_ratio/_circle/_ratio
  mask.txt      Zufallsfaelle fuer collide_mask
  skripte.txt   Zufaellige Folgen von Gruppen-Operationen samt
                Zustandsabdruck nach jedem Schritt

Die Rust-Seite spielt genau dieselben Faelle nach und vergleicht.
"""

import os
import random
import sys
import warnings
import zlib

os.environ["SDL_VIDEODRIVER"] = "dummy"
warnings.simplefilter("ignore")

import pygame
from pygame.sprite import (
    Sprite, DirtySprite, Group, GroupSingle, LayeredUpdates, LayeredDirty,
    RenderUpdates, collide_rect, collide_rect_ratio, collide_circle,
    collide_circle_ratio, collide_mask,
)

pygame.init()
pygame.display.set_mode((1, 1))

ZIEL = sys.argv[1]
os.makedirs(ZIEL, exist_ok=True)

# ===========================================================================
#  1. collide_*
# ===========================================================================

def gen_collide(n, seed=20260925):
    rnd = random.Random(seed)
    zeilen = []
    for _ in range(n):
        # Bewusst gemischte Groessen: flache, hohe, leere und negative
        # Rechtecke - die Randfaelle stecken genau dort.
        def rect():
            art = rnd.randrange(6)
            if art == 0:
                w, h = 0, rnd.randrange(0, 20)
            elif art == 1:
                w, h = rnd.randrange(1, 60), 1
            elif art == 2:
                w, h = 1, rnd.randrange(1, 60)
            elif art == 3:
                w, h = rnd.randrange(-10, 3), rnd.randrange(-10, 3)
            else:
                w, h = rnd.randrange(0, 40), rnd.randrange(0, 40)
            return (rnd.randrange(-40, 60), rnd.randrange(-40, 60), w, h)

        a = rect()
        b = rect()
        ratio = rnd.choice([
            0.25, 0.5, 0.75, 1.0, 1.3, 1.5, 1.999, 2.0, 3.0, 3.7,
            rnd.uniform(0.1, 4.0),
        ])
        # -1.0 heisst "radius nicht gesetzt"
        ra = -1.0 if rnd.random() < 0.6 else rnd.uniform(0.0, 40.0)
        rb = -1.0 if rnd.random() < 0.6 else rnd.uniform(0.0, 40.0)

        def mk(r, radius):
            s = Sprite()
            s.rect = pygame.Rect(*r)
            if radius >= 0.0:
                s.radius = radius
            return s

        r1 = collide_rect(mk(a, ra), mk(b, rb))
        r2 = collide_rect_ratio(ratio)(mk(a, ra), mk(b, rb))
        r3 = collide_circle(mk(a, ra), mk(b, rb))
        r4 = collide_circle_ratio(ratio)(mk(a, ra), mk(b, rb))

        zeilen.append(
            f"{a[0]} {a[1]} {a[2]} {a[3]} {b[0]} {b[1]} {b[2]} {b[3]} "
            f"{ratio!r} {ra!r} {rb!r} "
            f"{int(r1)}{int(r2)}{int(r3)}{int(r4)}"
        )
    return zeilen


# ===========================================================================
#  2. collide_mask
# ===========================================================================

def gen_mask(n, seed=770077):
    rnd = random.Random(seed)
    zeilen = []
    for _ in range(n):
        wa, ha = rnd.randrange(1, 13), rnd.randrange(1, 13)
        wb, hb = rnd.randrange(1, 13), rnd.randrange(1, 13)

        def bild(w, h):
            s = pygame.Surface((w, h), pygame.SRCALPHA)
            bits = []
            for y in range(h):
                for x in range(w):
                    # Alphawerte auch knapp um die Schwelle 127 herum.
                    alpha = rnd.choice([0, 0, 1, 126, 127, 128, 200, 255, 255])
                    s.set_at((x, y), (200, 100, 50, alpha))
                    bits.append(alpha)
            return s, bits

        ia, ba = bild(wa, ha)
        ib, bb = bild(wb, hb)
        ax, ay = rnd.randrange(-15, 15), rnd.randrange(-15, 15)
        bx, by = rnd.randrange(-15, 15), rnd.randrange(-15, 15)

        sa, sb = Sprite(), Sprite()
        sa.image, sa.rect = ia, pygame.Rect(ax, ay, wa, ha)
        sb.image, sb.rect = ib, pygame.Rect(bx, by, wb, hb)
        treffer = collide_mask(sa, sb)
        erg = "n" if treffer is None else f"{treffer[0]}:{treffer[1]}"
        # Zusaetzlich: wie viele Stellen ueberlappen ueberhaupt? Bei genau
        # einer ist der zurueckgegebene Punkt eindeutig, bei mehreren
        # haengt er an der Suchreihenfolge.
        ma = pygame.mask.from_surface(ia)
        mb = pygame.mask.from_surface(ib)
        anzahl = ma.overlap_area(mb, (bx - ax, by - ay))

        zeilen.append(
            f"{wa} {ha} {''.join('%02x' % a for a in ba)} "
            f"{wb} {hb} {''.join('%02x' % a for a in bb)} "
            f"{ax} {ay} {bx} {by} {erg} {anzahl}"
        )
    return zeilen


# ===========================================================================
#  3. Gruppen-Skripte
# ===========================================================================

BREITE, HOEHE = 48, 48
ANZ_SPRITES = 6
ANZ_GRUPPEN = 5

FARBEN = [
    (255, 0, 0), (0, 255, 0), (0, 0, 255),
    (255, 255, 0), (0, 255, 255), (255, 0, 255),
]
# Flaechen-Alpha statt SRCALPHA - gemischt wird trotzdem.
#
# Der Grund fuer diese Wahl ist gemessen: bei einem **vollstaendig
# weggeschnittenen** Blit gibt pygame fuer eine SRCALPHA-Quelle
# `Rect(18, 0, 0, 0)` zurueck, fuer jede andere `Rect(18, -5, 0, 0)` -
# der Alphaweg klemmt negative Koordinaten auf null, der Kopierweg nicht.
# rustgame-core klemmt nie. Das ist ein Unterschied im Blit-Kern, nicht
# in der Sprite-Kiste, und er wird hier umgangen statt verdeckt.
ALPHA = [None, None, None, 200, 128, None]


class Welt:
    """Sechs Sprites, fuenf Gruppen, eine Flaeche - in pygame."""

    def __init__(self, rnd):
        self.rnd = rnd
        self.sprites = []
        self.start = []
        for i in range(ANZ_SPRITES):
            w = rnd.randrange(3, 14)
            h = rnd.randrange(3, 14)
            x = rnd.randrange(-6, BREITE - 2)
            y = rnd.randrange(-6, HOEHE - 2)
            # Haelfte einfache Sprites, Haelfte DirtySprites - nur so
            # kommt der Fall "_layer fehlt" ueberhaupt vor.
            s = Sprite() if i < 3 else DirtySprite()
            img = pygame.Surface((w, h))
            img.fill(FARBEN[i])
            if ALPHA[i] is not None:
                img.set_alpha(ALPHA[i])
            s.image = img
            s.rect = pygame.Rect(x, y, w, h)
            self.sprites.append(s)
            self.start.append((x, y, w, h, i))

        self.gruppen = [
            Group(), RenderUpdates(), GroupSingle(),
            LayeredUpdates(), LayeredDirty(),
        ]
        self.flaeche = pygame.Surface((BREITE, HOEHE))
        self.flaeche.fill((20, 30, 40))
        self.bgd = pygame.Surface((BREITE, HOEHE))
        self.bgd.fill((7, 8, 9))
        self.letzte_rects = []

    # ----- ein Schritt ------------------------------------------------

    def fuehre_aus(self, op):
        t = op[0]
        self.letzte_rects = []
        if t == "ADD":
            self.gruppen[op[1]].add(self.sprites[op[2]])
        elif t == "ADDL":
            self.gruppen[op[1]].add(self.sprites[op[2]], layer=op[3])
        elif t == "REM":
            self.gruppen[op[1]].remove(self.sprites[op[2]])
        elif t == "KILL":
            self.sprites[op[1]].kill()
        elif t == "EMPTY":
            self.gruppen[op[1]].empty()
        elif t == "DRAW":
            self.letzte_rects = list(self.gruppen[op[1]].draw(self.flaeche))
        elif t == "MOVE":
            self.sprites[op[1]].rect.move_ip(op[2], op[3])
        elif t == "CHL":
            try:
                self.gruppen[op[1]].change_layer(self.sprites[op[2]], op[3])
            except ValueError:
                pass
        elif t == "SETSPR":
            self.gruppen[op[1]].sprite = self.sprites[op[2]]
        elif t == "UPD":
            self.gruppen[op[1]]._use_update = bool(op[2])
        elif t == "DIRTY":
            self.sprites[op[1]].dirty = op[2]
        elif t == "VIS":
            self.sprites[op[1]].visible = op[2]
        elif t == "CLEAR":
            self.gruppen[op[1]].clear(self.flaeche, self.bgd)
        elif t == "BGD":
            self.gruppen[op[1]].clear(self.flaeche, self.bgd)
        else:
            raise AssertionError(t)

    # ----- Abdruck ----------------------------------------------------

    def idx(self, s):
        for i, x in enumerate(self.sprites):
            if x is s:
                return i
        return -1

    def abdruck(self):
        teile = []
        for g in self.gruppen:
            namen = ",".join(str(self.idx(s)) for s in g.sprites())
            # `None` und `LayeredUpdates._init_rect` bedeuten beide
            # "noch nie gezeichnet" - auf der Rust-Seite ist das ein
            # `None`, hier also beides als "n".
            dict_ = ",".join(
                f"{self.idx(s)}="
                + ("n" if (r is None or r is LayeredUpdates._init_rect)
                   else f"{r[0]}:{r[1]}:{r[2]}:{r[3]}")
                for s, r in g.spritedict.items()
            )
            lost = ",".join(
                ("n" if r is None else f"{r[0]}:{r[1]}:{r[2]}:{r[3]}")
                for r in g.lostsprites
            )
            teile.append(f"{namen}/{dict_}/{lost}")
        sp = []
        for s in self.sprites:
            layer = getattr(s, "_layer", -999)
            dirty = getattr(s, "dirty", -1)
            sp.append(f"{int(s.alive())},{len(s.groups())},{layer},{dirty}")
        rects = ",".join(f"{r[0]}:{r[1]}:{r[2]}:{r[3]}" for r in self.letzte_rects)
        pruef = zlib.crc32(pygame.image.tobytes(self.flaeche, "RGBA"))
        return "|".join(teile) + "#" + ";".join(sp) + "#" + rects + "#" + str(pruef)


def gen_skript(rnd, welt, laenge):
    """Eine zufaellige Folge - ohne die Faelle, in denen pygame wirft."""
    ops = []
    for _ in range(laenge):
        art = rnd.randrange(14)
        if art in (0, 1, 2):
            g = rnd.randrange(ANZ_GRUPPEN)
            # LayeredDirty nimmt nur DirtySprites (sonst AttributeError).
            s = rnd.randrange(3, ANZ_SPRITES) if g == 4 else rnd.randrange(ANZ_SPRITES)
            if g in (3, 4) and rnd.random() < 0.5:
                ops.append(("ADDL", g, s, rnd.randrange(-3, 4)))
            else:
                ops.append(("ADD", g, s))
        elif art == 3:
            g = rnd.randrange(ANZ_GRUPPEN)
            s = rnd.randrange(3, ANZ_SPRITES) if g == 4 else rnd.randrange(ANZ_SPRITES)
            ops.append(("REM", g, s))
        elif art == 4:
            ops.append(("KILL", rnd.randrange(ANZ_SPRITES)))
        elif art == 5:
            ops.append(("EMPTY", rnd.randrange(ANZ_GRUPPEN)))
        elif art in (6, 7, 8):
            g = rnd.randrange(ANZ_GRUPPEN)
            if g == 4:
                ops.append(("UPD", g, rnd.randrange(2)))
            ops.append(("DRAW", g))
        elif art == 9:
            ops.append(("MOVE", rnd.randrange(ANZ_SPRITES),
                        rnd.randrange(-9, 10), rnd.randrange(-9, 10)))
        elif art == 10:
            g = rnd.choice([3, 4])
            s = rnd.randrange(3, ANZ_SPRITES) if g == 4 else rnd.randrange(ANZ_SPRITES)
            ops.append(("CHL", g, s, rnd.randrange(-3, 4)))
        elif art == 11:
            ops.append(("SETSPR", 2, rnd.randrange(ANZ_SPRITES)))
        elif art == 12:
            s = rnd.randrange(3, ANZ_SPRITES)
            if rnd.random() < 0.5:
                ops.append(("DIRTY", s, rnd.randrange(3)))
            else:
                ops.append(("VIS", s, rnd.randrange(2)))
        else:
            ops.append(("CLEAR", rnd.choice([0, 1, 3])))
    return ops


def gen_skripte(n, laenge, seed=31415):
    rnd = random.Random(seed)
    zeilen = []
    schritte = 0
    for _ in range(n):
        welt = Welt(rnd)
        ops = gen_skript(rnd, welt, laenge)
        zeilen.append("W " + " ".join(
            f"{x}:{y}:{w}:{h}" for (x, y, w, h, _) in welt.start))
        for op in ops:
            zeilen.append("O " + " ".join(str(v) for v in op))
            welt.fuehre_aus(op)
            zeilen.append("F " + welt.abdruck())
            schritte += 1
        zeilen.append("E")
    return zeilen, schritte


# ===========================================================================

def schreibe(name, zeilen, kopf):
    p = os.path.join(ZIEL, name)
    with open(p, "w", encoding="utf-8", newline="\n") as f:
        f.write(kopf)
        f.write("\n".join(zeilen))
        f.write("\n")
    print(f"{name}: {len(zeilen)} Zeilen")


N_COLLIDE = 12000
N_MASK = 4000
N_SKRIPTE = 4000
LAENGE = 16

kopf = ("# Erzeugte Datei - Messwerte aus pygame-ce 2.5.7.\n"
        "# Quelle: tools/gen_faelle.py. Nicht von Hand aendern.\n")

schreibe("collide.txt", gen_collide(N_COLLIDE), kopf)
schreibe("mask.txt", gen_mask(N_MASK), kopf)
skripte, schritte = gen_skripte(N_SKRIPTE, LAENGE)
schreibe("skripte.txt", skripte, kopf)
print(f"Skripte: {N_SKRIPTE} Stueck, {schritte} Einzelschritte")
print(f"Zusammen: {N_COLLIDE * 4 + N_MASK + schritte} Vergleiche")
