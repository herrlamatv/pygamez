"""Bot fuer Tower Defense: waehlt Karten per Ziffer/Klick, baut auf freie
Felder, waehlt Tuerme an, baut aus (u / Knopf / Zweige), verkauft, startet
Wellen, schaltet Tempo und nutzt im Maximal-Modus die Faehigkeiten.

Aufruf: bot_td.py <ordner> <frames> [setup-tasten]   (TD_MODE=classic|compact|maximal)
Schreibt <ordner>/td_keys.txt. Eigener Zufall (random.Random(23)).
"""
import os
import random
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames = sys.argv[1], int(sys.argv[2])
vorlauf = (sys.argv[3].split() if len(sys.argv) > 3 else []) + ["Return", "f"]
spur = []
plan = []
r = random.Random(23)
orig = vergleich.ereignis
PASSIV = os.environ.get("TD_PASSIV") == "1"


def mitte(rect):
    return "%d:%d" % rect.center


def feld(game, c, rr):
    x, y = game._px(c + 0.5, rr + 0.5)
    return "%d:%d" % (int(x), int(y))


def bot(game, k, IE):
    n = len(spur)
    if n < len(vorlauf):
        spur.append(vorlauf[n])
        orig(game, vorlauf[n], IE)
        return
    taste = ""
    st = game.state
    if plan:
        taste = plan.pop(0)
    elif st == "gameover":
        if n % 90 == 0:
            taste = r.choice(["Return", "Return", "m"])
    elif st == "mapsel":
        if n % 30 == 0:
            taste = r.choice(["Right", "Left", "Down", "Return", "@" + mitte(game.map_rects[0])])
    elif n % 7 == 0:
        x = r.random()
        if PASSIV and x < 0.6:
            x = 0.7
        frei = [(c, rr) for c in range(18) for rr in range(12) if game._can_build((c, rr))]
        if x < 0.45 and frei:
            i = r.randrange(len(game.bar_keys))
            wahl = ("%d" % ((i + 1) % 10)) if r.random() < 0.6 else "@" + mitte(game.card_rects[i])
            c, rr = r.choice(frei)
            plan.extend(["~" + feld(game, c, rr), "@" + feld(game, c, rr)])
            taste = wahl
        elif x < 0.60 and game.towers:
            cell = r.choice(list(game.towers))
            taste = "@" + feld(game, *cell)
            tw = game.towers[cell]
            y = r.random()
            if y < 0.5:
                plan.append(r.choice(["u", "@" + mitte(game.up_rect)]))
            elif y < 0.7:
                plan.append(r.choice(["@" + mitte(game.br_a_rect), "@" + mitte(game.br_b_rect)]))
            elif y < 0.73:
                plan.append(r.choice(["x", "@" + mitte(game.sell_rect)]))
            elif y < 0.85:
                plan.append("@%d:%d:3" % (game.width // 3, game.height // 2))
        elif x < 0.78 and st == "build":
            taste = r.choice(["space", "Return", "@" + mitte(game.start_rect)])
        elif x < 0.80:
            taste = r.choice(["f", "g"])
        elif x < 0.83 and getattr(game, "speed_rect", None) is not None:
            taste = "@" + mitte(game.speed_rect)
        elif x < 0.88:
            k = r.choice(["q", "w", "e"])
            taste = k
            if k == "q":
                c, rr = r.randrange(18), r.randrange(12)
                plan.extend(["~" + feld(game, c, rr), "", "@" + feld(game, c, rr)])
        elif x < 0.93:
            taste = "~%d:%d" % (r.randrange(game.width), r.randrange(game.height))
        elif x < 0.96:
            taste = "@%d:%d:3" % (r.randrange(game.width), r.randrange(game.height))
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("lamatowerdef", os.environ.get("TD_MODE", "classic"), 7, frames, ["x"] * frames, d,
                      os.path.join(sp, "bot.png"))
open(os.path.join(sp, "td_keys.txt"), "w").write(",".join(spur))
print("Ende")
