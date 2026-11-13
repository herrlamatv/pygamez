"""Bot fuer Solitaire: zufaellige, aber sinnnahe Aktionen aus dem Spielzustand.

Klick-Klick-Zuege (Quelle, Ziel), echte Drags (<x:y, ~x:y, >x:y),
Rechtsklicks aufs Fundament, Stock (space) und Undo (u). Der Zufall des
Bots ist ein eigener Generator - er beruehrt den Spielzufall nicht.

Aufruf: bot_solitaire.py <ordner> <frames> <modus> [setup-tasten]
Schreibt <ordner>/sol_keys.txt.
"""
import os
import random
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames, modus = sys.argv[1], int(sys.argv[2]), sys.argv[3]
vorlauf = (sys.argv[4].split() if len(sys.argv) > 4 else []) + ["Return"]
spur = []
plan = []
r = random.Random(4242)
orig = vergleich.ereignis


def mitte(rc):
    return rc.centerx, rc.y + min(rc.h // 2, 8)


def aktion(game):
    piles = [(i, p) for i, p in enumerate(game.piles) if p.kind != "removed"]
    voll = [(i, p) for i, p in piles if p.cards]
    wahl = r.random()
    if wahl < 0.12:
        return ["space"]
    if wahl < 0.17:
        return ["u"]
    if not voll:
        return ["space"]
    i, p = r.choice(voll)
    rects = p.rects(game.cw, game.ch)
    ci = r.randrange(len(rects))
    if wahl < 0.30:
        x, y = mitte(rects[-1])
        return ["@%d:%d:3" % (x, y)]
    j, q = r.choice(piles)
    qr = q.rects(game.cw, game.ch)
    import pygame
    ziel = qr[-1] if qr else pygame.Rect(q.x, q.y, game.cw, game.ch)
    sx, sy = mitte(rects[ci])
    tx, ty = mitte(ziel)
    if wahl < 0.65:
        return ["@%d:%d" % (sx, sy), "", "@%d:%d" % (tx, ty)]
    # echter Drag ueber mehrere Bilder
    mx, my = (sx + tx) // 2, (sy + ty) // 2
    return ["<%d:%d" % (sx, sy), "~%d:%d" % (mx, my), "~%d:%d" % (tx, ty), ">%d:%d" % (tx, ty)]


def bot(game, k, IE):
    n = len(spur)
    if n < len(vorlauf):
        spur.append(vorlauf[n])
        orig(game, vorlauf[n], IE)
        return
    taste = ""
    if game.state == "play":
        if game.game_over:
            if n % 90 == 0:
                taste = "Return"
        elif plan:
            taste = plan.pop(0)
        elif n % 6 == 0:
            plan.extend(aktion(game))
            taste = plan.pop(0)
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("solitaire", modus, 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
open(os.path.join(sp, "sol_keys.txt"), "w").write(",".join(spur))
print("Ende", sum(1 for s in spur if s))
