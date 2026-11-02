# -*- coding: utf-8 -*-
"""
Ein **vollstaendiges** pygame-Programm, einmal auf pygame-ce und einmal
auf der rustgame-Huelle - und dann ein byteweiser Vergleich des Bildes.

Anders als `vergleich_module.py`, das einzelne Aufrufe prueft, laeuft hier
das, worauf es am Ende ankommt: `init`, `set_mode`, eine Schleife mit
Ereignissen, Uhr, Zeichnen, Blitten, Umformen, Masken, Vektoren, `flip` -
und zum Schluss dasselbe Bild.

Der Text laeuft absichtlich **nicht** ins Bild: rustgame-font rastert mit
fontdue, pygame mit FreeType, die Pixel koennen also gar nicht gleich
sein. Verglichen werden dort die Masse, die aus der Schriftdatei kommen.
"""

import os
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
# tools/ -> rustgame-py -> crates -> rustgame -> pygamez
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(HIER))))
HUELLE = os.path.dirname(HIER)

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("RUSTGAME_VIDEODRIVER", "dummy")
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")

BREITE, HOEHE = 160, 120

# Wie viele Bilder verglichen werden. Der Plan nennt als Endabnahme
# **300**; dreissig genuegen fuer den taeglichen Lauf, weil die Szene in
# dreissig Bildern jede Zeichenart einmal durchlaeuft. Mit einer Zahl als
# Argument laesst sich das hochdrehen:
#
#     python tools/vergleich_spiel.py 300
#
# Bei 300 sind es 300 x 160 x 120 x 4 = **23.040.000 Pixelbytes**.
BILDER = 30
for _a in sys.argv[1:]:
    if _a.isdigit():
        BILDER = int(_a)


def spiel(pg):
    """Ein kleines Spiel - dasselbe Skript fuer beide Engines."""
    pg.init()
    schirm = pg.display.set_mode((BREITE, HOEHE))
    pg.display.set_caption("rustgame gegen pygame")
    uhr = pg.time.Clock()

    # Ein Spielstein mit Alphakanal, den wir drehen und skalieren.
    stein = pg.Surface((24, 24), pg.SRCALPHA)
    pg.draw.rect(stein, (220, 70, 70), pg.Rect(0, 0, 24, 24), 0, 6)
    pg.draw.circle(stein, (250, 220, 90), (12, 12), 6, 2)
    pg.draw.line(stein, (255, 255, 255, 160), (2, 2), (21, 21), 3)

    # Der Hintergrund - einmal gezeichnet, dann nur noch geblittet.
    hintergrund = pg.Surface((BREITE, HOEHE))
    hintergrund.fill((24, 26, 34))
    for i in range(0, BREITE, 16):
        pg.draw.line(hintergrund, (34, 38, 50), (i, 0), (i, HOEHE), 1)
    pg.draw.ellipse(hintergrund, (40, 70, 60), pg.Rect(10, 70, 140, 40), 2)

    # Die Maske des Steins - fuer die Beruehrungspruefung.
    steinmaske = pg.mask.from_surface(stein)
    hindernis = pg.Rect(110, 40, 30, 30)
    hindernismaske = pg.mask.Mask((30, 30), fill=True)

    ort = pg.math.Vector2(20.0, 20.0)
    schub = pg.math.Vector2(3.0, 1.5)
    treffer = 0
    ereignisse = []

    for bild in range(BILDER):
        # Ereignisse einspeisen - so bleibt der Lauf ohne Fenster
        # reproduzierbar und die Schlange trotzdem echt.
        if bild == 5:
            pg.event.post(pg.event.Event(pg.KEYDOWN, key=pg.K_SPACE,
                                         mod=0, unicode=" ", scancode=44))
        if bild == 9:
            pg.event.post(pg.event.Event(pg.USEREVENT + 1, {"punkte": 10}))
        if bild == 20:
            pg.event.post(pg.event.Event(pg.QUIT))

        laeuft = True
        for e in pg.event.get():
            ereignisse.append((bild, e.type, sorted(e.dict.items())))
            if e.type == pg.QUIT:
                laeuft = False
            elif e.type == pg.KEYDOWN and e.key == pg.K_SPACE:
                schub = schub.rotate(37.0)
            elif e.type >= pg.USEREVENT:
                treffer += e.punkte

        ort += schub
        if ort.x < 0 or ort.x > BREITE - 24:
            schub.x = -schub.x
            ort.x = max(0.0, min(float(BREITE - 24), ort.x))
        if ort.y < 0 or ort.y > HOEHE - 24:
            schub.y = -schub.y
            ort.y = max(0.0, min(float(HOEHE - 24), ort.y))

        stelle = (int(ort.x), int(ort.y))
        versatz = (hindernis.x - stelle[0], hindernis.y - stelle[1])
        if steinmaske.overlap(hindernismaske, versatz):
            treffer += 1

        schirm.blit(hintergrund, (0, 0))
        pg.draw.rect(schirm, (90, 140, 200), hindernis, 0, 4)
        gedreht = pg.transform.rotate(stein, bild * 7.0)
        klein = pg.transform.scale(gedreht, (gedreht.get_width() // 2,
                                             gedreht.get_height() // 2))
        schirm.blit(gedreht, gedreht.get_rect(center=(stelle[0] + 12,
                                                      stelle[1] + 12)))
        schirm.blit(klein, (4, 4), None, pg.BLEND_RGBA_ADD)
        pg.draw.arc(schirm, (255, 200, 120), pg.Rect(60, 20, 40, 30),
                    0.0, 3.14159 * (bild / BILDER), 2)
        pg.display.flip()
        uhr.tick(0)

        if not laeuft:
            break

    # Die Schriftmasse - die Pixel koennen nicht gleich sein, die Masse
    # schon.
    pg.font.init()
    schrift = pg.font.Font(None, 22)
    text = schrift.render("Punkte: %d" % treffer, True, (255, 255, 255))

    bild_bytes = pg.image.tobytes(schirm, "RGBA")
    return {
        "treffer": treffer,
        "bilder": bild + 1,
        "ereignisse": ereignisse,
        "ort": (round(ort.x, 9), round(ort.y, 9)),
        "schub": (round(schub.x, 9), round(schub.y, 9)),
        "maske": (steinmaske.count(), steinmaske.get_size()),
        "caption": pg.display.get_caption(),
        "schirm_flaggen": schirm.get_flags(),
        "schrift": (schrift.get_height(), schrift.get_linesize(),
                    schrift.get_ascent(), schrift.get_descent(),
                    schrift.size("Punkte: %d" % treffer), text.get_size(),
                    text.get_flags()),
        "bild": bild_bytes,
    }


def main():
    import pygame
    echt = spiel(pygame)

    sys.path.insert(0, HUELLE)
    import rustgame_py as rg
    nach = spiel(rg)

    print("pygame-ce %s gegen rustgame - ein ganzes Programm" % pygame.version.ver)
    fehler = 0
    for schluessel in echt:
        if schluessel == "bild":
            continue
        a, b = echt[schluessel], nach[schluessel]
        gleich = repr(a) == repr(b)
        print("  %-16s %s" % (schluessel, "gleich" if gleich else "ABWEICHUNG"))
        if not gleich:
            fehler += 1
            print("      pygame   %r" % (a,))
            print("      rustgame %r" % (b,))

    a, b = echt["bild"], nach["bild"]
    if len(a) != len(b):
        print("  Bild: verschiedene Groesse %d gegen %d" % (len(a), len(b)))
        return 1
    anders = [i for i in range(len(a)) if a[i] != b[i]]
    print("  Pixelbytes: %d von %d unterschiedlich" % (len(anders), len(a)))
    for i in anders[:8]:
        px = i // 4
        print("     (%3d,%3d) pygame %s  rustgame %s"
              % (px % BREITE, px // BREITE,
                 a[px * 4:px * 4 + 4].hex(), b[px * 4:px * 4 + 4].hex()))
    return 1 if (fehler or anders) else 0


if __name__ == "__main__":
    sys.exit(main())
