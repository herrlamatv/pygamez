"""Bot fuer Doodle Jump: steuert im Python-Lauf, schreibt die Ereignisse."""
import os
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames, out, vorab = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4].split(",") if len(sys.argv) > 4 else []
spur = []
orig = vergleich.ereignis
halt = {"l": False, "r": False}
warte = [0]


def bot(game, k, IE):
    g = game
    if len(spur) < len(vorab):
        e = vorab[len(spur)]
        spur.append(e)
        if e:
            orig(game, e, IE)
        return
    ev = []
    if g.state == "gameover":
        warte[0] += 1
        if warte[0] > 40:
            warte[0] = 0
            for key in halt:
                halt[key] = False
            ev.append("Return")
    if g.state == "play":
        ziel = None
        best = 1e9
        for p in g.platforms:
            if p.dead or p.kind == "brittle":
                continue
            dy = p.y - (g.y + g.dr)
            if g.vy > 0 and dy > -4 and dy < best:
                best, ziel = dy, p
            elif g.vy <= 0 and -260 < dy < 0 and -dy < best:
                best, ziel = -dy, p
        want = {"l": False, "r": False}
        if ziel is not None:
            cx = ziel.x + ziel.w / 2
            if cx < g.x - 8:
                want["l"] = True
            elif cx > g.x + 8:
                want["r"] = True
        for key, name in (("l", "Left"), ("r", "Right")):
            if want[key] and not halt[key]:
                ev.append("+" + name)
                halt[key] = True
            elif not want[key] and halt[key]:
                ev.append("-" + name)
                halt[key] = False
        if not ev:
            for m in g.monsters:
                if abs(m["x"] - g.x) < 14 and 0 < g.y - m["y"] < 220 and g.shoot_cd <= 0:
                    ev.append("space")
                    break
    e = ev[0] if ev else ""
    spur.append(e)
    if e:
        orig(game, e, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("doodle", "single", 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
open(os.path.join(sp, out), "w").write(",".join(spur))
print("fertig")
