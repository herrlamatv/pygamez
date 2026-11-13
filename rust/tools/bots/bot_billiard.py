"""Bot fuer Billard: zielt per Maus auf eine Kugel, laedt per gedrueckter
Maustaste unterschiedlich stark und stoesst; legt die Weisse bei Ball-in-Hand.

Aufruf: bot_billiard.py <ordner> <frames> [setup-tasten]   (BIL_MODE=multi)
Schreibt <ordner>/bil_keys.txt. Eigener Zufall (random.Random(31)).
"""
import os
import random
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames = sys.argv[1], int(sys.argv[2])
vorlauf = (sys.argv[3].split() if len(sys.argv) > 3 else []) + ["Return"]
spur = []
plan = []
r = random.Random(31)
orig = vergleich.ereignis


def bot(game, k, IE):
    n = len(spur)
    if n < len(vorlauf):
        spur.append(vorlauf[n])
        orig(game, vorlauf[n], IE)
        return
    taste = ""
    if plan:
        taste = plan.pop(0)
    elif game.state == "over":
        if n % 120 == 0:
            taste = "Return"
    elif game.state == "play" and game._human_turn() and n % 10 == 0:
        if game.phase == "place":
            x, y = r.uniform(-100, -30), r.uniform(-40, 40)
            sx, sy, _, _ = game._project(x, y)
            taste = "@%d:%d" % (sx, sy)
        elif game.phase == "aim":
            ziele = [b for b in game.balls if not b.potted and b.num]
            if ziele:
                b = r.choice(ziele)
                sx, sy, _, _ = game._project(b.x + r.uniform(-2, 2), b.y + r.uniform(-2, 2))
                tok = "%d:%d" % (sx, sy)
                plan.extend(["~" + tok, "<" + tok] + [""] * r.randint(3, 50) + [">" + tok])
                taste = plan.pop(0)
            if r.random() < 0.05:
                plan.insert(0, "v")
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("billiard", os.environ.get("BIL_MODE", "single"), 7, frames, ["x"] * frames, d,
                      os.path.join(sp, "bot.png"))
open(os.path.join(sp, "bil_keys.txt"), "w").write(",".join(spur))
print("Ende")
