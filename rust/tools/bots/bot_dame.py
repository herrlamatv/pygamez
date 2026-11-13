"""Bot fuer Dame: klickt im Python-Lauf legale Zuege, schreibt die Ereignisse."""
import os
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames, out, vorab, mode = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4].split(","), sys.argv[5]
spur = []
orig = vergleich.ereignis
plan = []
zug = [0]


def mitte(g, rc):
    r, c = rc
    return "@%d:%d" % (g.bx + c * g.cell + g.cell // 2, g.by + r * g.cell + g.cell // 2)


def bot(game, k, IE):
    g = game
    if len(spur) < len(vorab):
        e = vorab[len(spur)]
    else:
        e = ""
        if g.state == "over":
            if not plan:
                plan.extend([""] * 40 + ["Return"])
        elif g.state == "play" and (g.multiplayer or g.player == 0) and not plan and g.sel is None:
            if g.moves:
                m = g.moves[(zug[0] * 7) % len(g.moves)]
                zug[0] += 1
                plan.extend([mitte(g, p) for p in m["path"]] + [""] * 3)
        if plan:
            e = plan.pop(0)
    spur.append(e)
    if e:
        orig(game, e, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("dame", mode, 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
open(os.path.join(sp, out), "w").write(",".join(spur))
print("fertig")
