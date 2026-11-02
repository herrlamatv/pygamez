# -*- coding: utf-8 -*-
"""
rustgame/tools/vergleich_py.py
==============================
Die **Abnahmepruefung**: dasselbe Python-Skript, einmal auf echtem
pygame-ce, einmal auf rustgame - und dann ein byteweiser Vergleich.

Die Referenzbilder pruefen den Rust-Kern. Der Fuzzer prueft ihn im
Zusammenspiel. Dieses Werkzeug prueft die **Huelle**: ob ein
pygame-Programm, das niemand angefasst hat, durch die Rust-Engine
unveraendert hindurchlaeuft. Das ist am Ende die Frage, auf die es
ankommt - alles davor ist Mittel zum Zweck.

Die Szene wird **einmal** geschrieben und beiden Seiten mit ihrem eigenen
Modul uebergeben. Es gibt also keine zwei Fassungen, die auseinanderlaufen
koennten.

Vorher das Modul bauen::

    cd rustgame/crates/rustgame-py
    $env:PATH = "$env:USERPROFILE\\.cargo\\bin;$env:PATH"
    cargo build --release
    copy ..\\..\\..\\rustgame.output\\release\\rustgame_py.dll rustgame_py.pyd

Dann::

    .venv/Scripts/python.exe rustgame/tools/vergleich_py.py
"""

import os
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HIER))
HUELLE = os.path.join(ROOT, "rustgame", "crates", "rustgame-py")

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")


def szene(pg, Surface, Rect):
    """Eine Szene, die moeglichst viele Ecken gleichzeitig beruehrt.

    Absichtlich keine glatte Vorzeigeszene: Eckenradien, Umrisse mit
    Strichstaerke, ein halbdurchsichtiger Blit, Spiegeln und Skalieren
    hintereinander - also genau die Stellen, an denen ein Nachbau
    auseinandergeht.
    """
    s = Surface((60, 40))
    s.fill((30, 30, 40))
    pg.draw.rect(s, (220, 60, 60), Rect(4, 4, 24, 14), 0, 4)
    pg.draw.rect(s, (60, 200, 120), (32, 4, 24, 14), 2, 6)
    pg.draw.circle(s, (70, 110, 240), (16, 28), 9, 0)
    pg.draw.circle(s, (250, 220, 80), (44, 28), 9, 3)
    pg.draw.line(s, (255, 255, 255), (0, 39), (59, 20), 3)
    pg.draw.polygon(s, (200, 100, 220), [(30, 36), (38, 22), (46, 36)], 0)
    pg.draw.ellipse(s, (120, 220, 220), Rect(2, 18, 20, 8), 1)

    q = Surface((20, 20), pg.SRCALPHA)
    q.fill((255, 255, 255, 90))
    s.blit(q, (20, 10))

    gespiegelt = pg.transform.flip(s, True, False)
    s.blit(pg.transform.scale(gespiegelt, (30, 20)), (15, 10))

    r = s.get_rect(center=(100, 100))
    return s, (r.x, r.y, r.w, r.h)


def als_bytes(s, echtes_pygame):
    """RGBA, zeilenweise - auf beiden Seiten gleich aufgebaut."""
    if echtes_pygame:
        import pygame
        return pygame.image.tobytes(s, "RGBA")
    return bytes(
        kanal
        for y in range(s.get_height())
        for x in range(s.get_width())
        for kanal in tuple(s.get_at((x, y)))[:4]
    )


def main():
    import pygame
    pygame.init()
    pygame.display.set_mode((1, 1))
    echt, rect_echt = szene(pygame, pygame.Surface, pygame.Rect)
    bytes_echt = als_bytes(echt, True)

    sys.path.insert(0, HUELLE)
    try:
        import rustgame_py as rg
    except ImportError:
        print("rustgame_py nicht gefunden. Erst bauen - siehe Kopf dieser Datei.")
        return 2

    nach, rect_nach = szene(rg, rg.Surface, rg.Rect)
    bytes_nach = als_bytes(nach, False)

    print("pygame-ce %s gegen rustgame" % pygame.version.ver)
    print("get_rect(center=(100,100))")
    print("   pygame   %s" % (rect_echt,))
    print("   rustgame %s   %s"
          % (rect_nach, "gleich" if rect_echt == rect_nach else "ABWEICHUNG"))

    if len(bytes_echt) != len(bytes_nach):
        print("Verschiedene Groesse: %d gegen %d Bytes"
              % (len(bytes_echt), len(bytes_nach)))
        return 1

    anders = [i for i in range(len(bytes_echt)) if bytes_echt[i] != bytes_nach[i]]
    breite = echt.get_width()
    print("Pixelbytes: %d von %d unterschiedlich" % (len(anders), len(bytes_echt)))
    if not anders and rect_echt == rect_nach:
        print()
        print("Dasselbe Skript, beide Engines, Byte fuer Byte dasselbe Bild.")
        return 0

    for i in anders[:10]:
        px = i // 4
        print("   (%2d,%2d)  pygame %s   rustgame %s"
              % (px % breite, px // breite,
                 bytes_echt[px * 4:px * 4 + 4].hex(),
                 bytes_nach[px * 4:px * 4 + 4].hex()))
    return 1


if __name__ == "__main__":
    sys.exit(main())
