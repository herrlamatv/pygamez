# -*- coding: utf-8 -*-
"""
ui_probe.py
===========
Misst, ob ``ui.rs`` wirklich ``ui.py`` ist.

Rendert dieselbe Szene wie ``crates/pgz-engine/examples/ui_probe.rs`` mit
echtem pygame und dem echten ``ui.py`` - in allen zehn Designs, mit
eingefrorener Uhr - und vergleicht danach Bild fuer Bild mit dem, was die
Rust-Fassung abgelegt hat.

    cargo run -p pgz-engine --example ui_probe -- <ordner>/rust
    .venv/Scripts/python.exe rust/tools/ui_probe.py <ordner>

Ausgabe je Design: abweichende Pixel und die erste Abweichung. Endet mit 1,
sobald ein Design abweicht.
"""

import os
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))
sys.path.insert(0, WURZEL)
os.environ["SDL_VIDEODRIVER"] = "dummy"
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")

import random  # noqa: E402
import pygame  # noqa: E402

pygame.init()
pygame.display.set_mode((1, 1))

import ui  # noqa: E402
from game_base import InputEvent  # noqa: E402

_UHR = [0]
pygame.time.get_ticks = lambda: _UHR[0]


def szene(s, feld):
    w, h = 640, 480
    ui.draw_background(s, w, h)
    ui.draw_title(s, w, "PyGameZ Probe", "Untertitel der Szene")
    ui.draw_panel(s, (40, 120, 560, 250))
    ui.draw_button(s, (70, 150, 230, 46), "Start", ui.font(22), selected=True)
    ui.draw_button(s, (340, 150, 230, 46), "Optionen", ui.font(22), selected=False)
    ui.draw_button(s, (70, 215, 230, 56), "Mehrspieler", ui.font(20, bold=True),
                   selected=False, sub="zwei Spieler", sub_font=ui.font(14),
                   accent=(240, 160, 90))
    ui.draw_panel(s, (340, 215, 230, 56), radius=8, shadow=False, accent_top=ui.GOLD)
    feld.draw(s, (90, 300, 260, 36), ui.font(18), focused=True)
    ui.draw_grad_line(s, 470, 318, 180, 3, glow=True)
    img = ui.grad_text(ui.font(26, bold=True), "Verlauf 123")
    s.blit(img, (390, 330))
    pygame.draw.circle(s, ui.ACCENT, (600, 420), 12)
    ui.draw_footer(s, w, h, "Enter = Start  -  Esc = Pause")
    ui.draw_halo(s, (320, 60), 180)


def main():
    ordner = sys.argv[1] if len(sys.argv) > 1 else "ui_probe"
    os.makedirs(os.path.join(ordner, "py"), exist_ok=True)
    feld = ui.TextInput("", maxlen=20, placeholder="Name")
    for ch in "Lama 42":
        feld.handle(InputEvent(InputEvent.KEYDOWN,
                               key="space" if ch == " " else ch, char=ch))
    schlecht = 0
    for name in ui.THEME_NAMES:
        ui.set_theme(name)
        random.seed(7)   # Sternschnuppen u. a. ziehen aus random
        s = pygame.Surface((640, 480))
        for i in range(12):
            _UHR[0] = 5000 + i * 17
            szene(s, feld)
        pygame.image.save(s, os.path.join(ordner, "py", name + ".png"))
        rpfad = os.path.join(ordner, "rust", name + ".png")
        if not os.path.exists(rpfad):
            print("%-8s fehlt: %s" % (name, rpfad))
            schlecht += 1
            continue
        r = pygame.image.load(rpfad)
        anders = 0
        erste = None
        for y in range(480):
            for x in range(640):
                a, b = s.get_at((x, y)), r.get_at((x, y))
                if a[:3] != b[:3]:
                    anders += 1
                    if erste is None:
                        erste = ((x, y), tuple(a[:3]), tuple(b[:3]))
        print("%-8s %6d Pixel anders%s" % (name, anders,
                                           "" if erste is None else
                                           "  erste %s py=%s rust=%s" % erste))
        if anders:
            schlecht += 1
    sys.exit(1 if schlecht else 0)


if __name__ == "__main__":
    main()
