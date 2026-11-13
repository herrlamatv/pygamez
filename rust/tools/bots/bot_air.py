"""Bot fuer Air Hockey: Maus folgt dem Puck (Python-Lauf), Spur als Ereignisse."""
import os
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames, out, vorab, mode = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4].split(","), sys.argv[5]
spur = []
orig = vergleich.ereignis
n = [0]


def bot(game, k, IE):
    g = game
    e = ""
    if len(spur) < len(vorab):
        e = vorab[len(spur)]
    elif g.state == "play" and not g.game_over:
        n[0] += 1
        if mode == "multi":
            # P1 per Tasten: Richtung Puck
            e = ["+d", "-d", "+w", "-w", "+s", "-s", "+a", "-a"][n[0] % 8] if n[0] % 5 == 0 else ""
        elif n[0] % 2 == 0:
            tx = int(min(g.cx - 30, g.puck_x - 20))
            ty = int(g.puck_y)
            e = "~%d:%d" % (tx, ty)
    elif g.game_over and n[0] >= 0:
        n[0] = -60
    elif n[0] < 0:
        n[0] += 1
        if n[0] == -1:
            e = "Return"
    spur.append(e)
    if e:
        orig(game, e, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("airhockey", mode, 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
open(os.path.join(sp, out), "w").write(",".join(spur))
print("fertig")
