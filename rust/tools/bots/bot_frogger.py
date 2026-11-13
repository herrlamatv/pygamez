"""Bot fuer Frogger: entscheidet im Python-Lauf, schreibt die Tastenfolge."""
import os
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames, out = sys.argv[1], int(sys.argv[2]), sys.argv[3]
spur = []
orig = vergleich.ereignis
pause = [0]


def sicher_strasse(g, row, x, vor=0.35):
    lane = g._lane_for_row(row)
    half = g.cell * 0.3 + 6
    for e in lane["ents"]:
        for t in (0.0, vor / 2, vor):
            ex = e["x"] + lane["speed"] * t
            if ex < x + half and x - half < ex + e["w"]:
                return False
    return True


def traeger(g, row, x, vor=0.5):
    lane = g._lane_for_row(row)
    for e in lane["ents"]:
        ok = True
        for t in (0.0, vor):
            ex = e["x"] + lane["speed"] * t
            xx = x + lane["speed"] * t
            if not (ex + 6 <= xx <= ex + e["w"] - 6):
                ok = False
        if ok and not (lane["kind"] == "turtle" and g._turtle_state(e) != "up"):
            return True
    return False


def bot(game, k, IE):
    g = game
    taste = ""
    if len(spur) == 0:
        taste = "Return"
    elif g.state == "play" and not g.game_over and g.death_t <= 0 and g.clear_t <= 0:
        if pause[0] > 0:
            pause[0] -= 1
        else:
            row, x = g.frog_row, g.frog_x
            nr = row - 1
            if row == 1:
                bay = g._bay_index_at(x)
                if bay is not None and not g.bays_done[bay] and not (g.croc and g.croc["bay"] == bay):
                    taste = "Up"
            elif 7 <= nr <= 11:
                if sicher_strasse(g, nr, x):
                    taste = "Up"
            elif 1 <= nr <= 5:
                if traeger(g, nr, x):
                    taste = "Up"
            elif nr == 6:
                taste = "Up"
            if taste == "" and row == 6:
                # auf dem Mittelstreifen Richtung freie Bucht schieben
                frei = [i for i in range(5) if not g.bays_done[i]]
                if frei:
                    ziel = min(frei, key=lambda i: abs((i + 0.5) * g.width / 5 - x))
                    cx = (ziel + 0.5) * g.width / 5
                    if cx - x > g.cell * 0.5:
                        taste = "Right"
                    elif x - cx > g.cell * 0.5:
                        taste = "Left"
            if taste:
                pause[0] = 6
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("frogger", "single", 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
open(os.path.join(sp, out), "w").write(",".join(spur))
print("fertig", len(spur))
