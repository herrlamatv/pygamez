"""Bot fuer Breakout: folgt mit der Maus dem tiefsten fallenden Ball.

Aufruf: bot_breakout.py <ordner> <frames> [setup-tasten]
Die Setup-Tasten (durch Leerzeichen getrennt, z. B. "3 Up Up m") kommen vor
dem Start; danach Return. Schreibt <ordner>/bo_keys.txt.
"""
import os
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames = sys.argv[1], int(sys.argv[2])
vorlauf = (sys.argv[3].split() if len(sys.argv) > 3 else []) + ["Return"]
spur = []
orig = vergleich.ereignis


def bot(game, k, IE):
    n = len(spur)
    if n < len(vorlauf):
        spur.append(vorlauf[n])
        orig(game, vorlauf[n], IE)
        return
    taste = ""
    if game.state == "play":
        if game.ball_hängt or any(b.stuck for b in game.balls):
            if game.intro_timer <= 0 and n % 25 == 0:
                taste = "space"
        else:
            fallend = [b for b in game.balls if b.vy > 0] or game.balls
            ziel = max(fallend, key=lambda b: b.y)
            # leichter Versatz, damit die Abprallwinkel wechseln
            versatz = ((n // 90) % 5 - 2) * game.paddle_w * 0.18
            x = int(ziel.x - versatz)
            taste = "~%d:%d" % (x, game.paddle_y)
            if game.fx["laser"] > 0 and n % 40 == 0:
                taste = "space"
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("breakout", "single", 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
open(os.path.join(sp, "bo_keys.txt"), "w").write(",".join(spur))
print("Ende")
