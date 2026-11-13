"""Bot fuer den Aim Trainer: dreht per MOUSEREL (^dx:dy) auf ein Ziel und schiesst.

Aufruf: bot_aim.py <ordner> <frames> <modus> [setup-tasten]
Schreibt <ordner>/aim_keys.txt. Jeder siebte Schuss geht absichtlich
daneben, damit auch Fehlschuesse/Combo-Abbrueche vorkommen.
"""
import math
import os
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames, modus = sys.argv[1], int(sys.argv[2]), sys.argv[3]
vorlauf = (sys.argv[4].split() if len(sys.argv) > 4 else []) + ["Return"]
spur = []
schuesse = [0]
orig = vergleich.ereignis


def bot(game, k, IE):
    n = len(spur)
    if n < len(vorlauf):
        spur.append(vorlauf[n])
        orig(game, vorlauf[n], IE)
        return
    taste = ""
    if game.state == "play" and not game.game_over:
        ziele = [tg for tg in game.targets if tg["state"] != "shrink"]
        if ziele:
            tg = ziele[0]
            d = game._norm_dir(game._target_pos(tg))
            ty = math.atan2(d[0], d[2])
            tp = math.asin(max(-1.0, min(1.0, d[1])))
            dy = (ty - game.yaw + math.pi) % math.tau - math.pi
            dp = tp - game.pitch
            kk = math.radians(0.12) * game.sens
            rx, ry = int(round(dy / kk)), int(round(-dp / kk))
            if abs(rx) <= 1 and abs(ry) <= 1 and game.cooldown <= 0 and n % 3 == 0:
                schuesse[0] += 1
                taste = "@320:240" if schuesse[0] % 7 else "^25:0"
            elif rx or ry:
                # in Schritten, wie eine echte Maus
                taste = "^%d:%d" % (max(-60, min(60, rx)), max(-60, min(60, ry)))
    elif game.game_over and n % 200 == 0:
        taste = ""
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("aim", modus, 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
open(os.path.join(sp, "aim_keys.txt"), "w").write(",".join(spur))
print("Ende", schuesse[0], "Schuesse")
