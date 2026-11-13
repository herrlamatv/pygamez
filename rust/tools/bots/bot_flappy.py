"""Bot fuer Flappy: entscheidet im Python-Lauf, schreibt die Tastenfolge."""
import os
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames, diff = sys.argv[1], int(sys.argv[2]), sys.argv[3]
spur = []
orig = vergleich.ereignis


def bot(game, k, IE):
    if len(spur) < 2:
        spur.append(["0", "Return"][len(spur)] if diff != "0" else "Return")
        if spur[-1] == "Return" and len(spur) == 2:
            orig(game, "Return", IE)
        elif spur[-1] != "0":
            orig(game, spur[-1], IE)
        return
    taste = ""
    if game.state in ("ready",):
        taste = "space"
    elif game.state == "play":
        ziel = game.height * 0.45
        for p in game.pipes:
            if p["x"] + game.pipe_w > game.bird_x - game.bird_r:
                ziel = p["gy"] + p["gap"] * 0.12
                break
        if game.bird_y > ziel and game.bird_v > -60:
            taste = "space"
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("flappy", "single", 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
# Schritt 0 war die Schwierigkeits-Taste (oder nichts), Schritt 1 Return.
out = [("Right" if diff == "1" else "") if spur[0] == "0" else spur[0]] + spur[1:]
open(os.path.join(sp, "fb_keys.txt"), "w").write(",".join(out))
print("Ende")
