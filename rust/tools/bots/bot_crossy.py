"""Bot fuer Crossy Road: huepft vorsichtig vorwaerts (prueft Autos, Zuege,
Staemme, Seerosen und Baeume der Zielreihe), weicht seitwaerts aus, wartet,
geht gelegentlich zurueck; im Setup Schalter, Figuren-Reiter und Kaeufe, nach
dem Tod Neustart/Figuren/Setup.

Aufruf: bot_crossy.py <ordner> <frames> [setup-tasten]   (CR_MODE=endless|daily)
CR_MUT=0..1 = Anteil zufaelliger (unvorsichtiger) Spruenge (Vorgabe 0.08).
Schreibt <ordner>/cr_keys.txt. Eigener Zufall (random.Random(29)).
"""
import os
import random
import sys
sys.path.insert(0, "rust/tools")
sys.path.insert(0, ".")
import vergleich
from games import crossyroad_world as cw

sp, frames = sys.argv[1], int(sys.argv[2])
vorlauf = sys.argv[3].split() if len(sys.argv) > 3 else []
spur = []
plan = []
r = random.Random(29)
orig = vergleich.ereignis
MUT = float(os.environ.get("CR_MUT", "0.08"))
besucht = {}
wechsel = []
ursachen = []
nacht = []


def sicher(game, dc, dr):
    w = game.world
    fr = game.row
    tr = fr + dr
    dst = w.row(tr)
    cx = (game.col + dc) if dr == 0 else (game.col if (dst["kind"] == cw.RIVER and dst["pads"] is None)
                                          else float(int(game.col + 0.5)))
    if game._blocked(dst, cx):
        return False
    t = game.clock
    if dst["kind"] == cw.ROAD:
        return all(cw.vehicle_at(dst, cx + 0.5, t + k * 0.06, 0.6) < 0 for k in range(0, 9))
    if dst["kind"] == cw.RAIL:
        warn, head, _ = cw.train_state(dst, t)
        if warn or head is not None:
            return False
        return all(not cw.train_hits(dst, cx + 0.5, t + k * 0.1, 0.8) for k in range(0, 6))
    if dst["kind"] == cw.RIVER:
        if dst["pads"] is not None:
            c = int(cx + 0.5)
            return 0 <= c < cw.COLS and dst["pads"][c] == 1
        hit = cw.log_at(dst, cx + 0.5, t + cw.HOP_T, 0.05)
        return hit is not None and 0.5 < cx + 0.5 < cw.COLS - 0.5
    # Wiese: auch die aktuelle Reihe muss man verlassen duerfen
    return True


def bot(game, k, IE):
    n = len(spur)
    if n < len(vorlauf):
        spur.append(vorlauf[n])
        orig(game, vorlauf[n], IE)
        return
    taste = ""
    st = game.state
    zst = st if st == "setup" else ("over" if game.game_over else ("tot" if game.death else "lauf"))
    if zst != besucht.get("_l"):
        wechsel.append((n, zst))
        if zst == "tot":
            ursachen.append((n, game.death["cause"], game.best, game.data["coins"]))
    if zst == "lauf" and game.world is not None and cw.night_factor(game.cam + 4.0) > 0.5 and not nacht:
        nacht.append(n)
    besucht["_l"] = zst
    besucht[zst] = besucht.get(zst, 0) + 1
    if plan:
        taste = plan.pop(0)
    elif zst == "setup":
        if n % 12 == 0:
            x = r.random()
            if game.tab == "chars":
                if x < 0.4:
                    taste = r.choice(["Left", "Right", "Up", "Down"])
                elif x < 0.6:
                    taste = r.choice(["Return", "space", "@" + "%d:%d" % game.buy_rect.center])
                elif x < 0.75:
                    taste = "@%d:%d" % r.choice(game.grid_rects).center
                else:
                    taste = r.choice(["Escape", "Tab", "@%d:%d" % game.tab_rects[0].center])
            else:
                if x < 0.45:
                    taste = r.choice(["Return", "space", "@%d:%d" % game.start_rect.center])
                elif x < 0.6:
                    taste = r.choice(["Left", "Right"])
                elif x < 0.7:
                    taste = r.choice(["h", "n"])
                elif x < 0.8:
                    taste = "@%d:%d" % r.choice(game.shadow_rects + game.night_rects).center
                else:
                    taste = r.choice(["Tab", "@%d:%d" % game.tab_rects[1].center, "@%d:%d" % game.card_rect.center])
    elif zst == "over":
        if n % 40 == 0:
            x = r.random()
            if x < 0.6:
                taste = r.choice(["Return", "space", "@%d:%d" % game.over_btns[0][1].center])
            elif x < 0.8:
                taste = r.choice(["f", "@%d:%d" % game.over_btns[1][1].center])
            else:
                taste = r.choice(["s", "@%d:%d" % game.over_btns[2][1].center])
    elif zst == "lauf" and game.hop is None and n % 3 == 0:
        x = r.random()
        if x < MUT:
            taste = r.choice(["Up", "Up", "Left", "Right", "Down", "@320:240", "w"])
        elif sicher(game, 0, 1):
            taste = r.choice(["Up", "Up", "Up", "w", "space", "@320:240"])
        elif x < 0.5 and sicher(game, -1, 0):
            taste = "Left"
        elif x < 0.9 and sicher(game, 1, 0):
            taste = "Right"
        elif game.cam_target - game.row > 2.4 and sicher(game, 0, 1) is False and r.random() < 0.3:
            taste = "Up"
        elif r.random() < 0.05 and sicher(game, 0, -1):
            taste = "Down"
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("crossy", os.environ.get("CR_MODE", "endless"), 7, frames, ["x"] * frames, d,
                      os.path.join(sp, "bot.png"))
open(os.path.join(sp, "cr_keys.txt"), "w").write(",".join(spur))
print("Ende", {k: v for k, v in besucht.items() if k != "_l"})
print("Wechsel", wechsel[:30])
print("Tode", ursachen)
print("Nacht ab", nacht)
