"""Bot fuer Schiffe versenken: stellt die Flotte per Zufall, Ziehen aus dem
Dock, Klicks, Drehen und Tastatur auf, feuert per Klick und Tastatur, bestaetigt
Uebergaben (2 Spieler) und startet nach Rundenende neu oder geht ins Setup.

Aufruf: bot_battleship.py <ordner> <frames> [setup-tasten]   (BS_MODE=single|multi)
Schreibt <ordner>/bs_keys.txt. Eigener Zufall (random.Random(17)).
"""
import os
import random
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames = sys.argv[1], int(sys.argv[2])
vorlauf = sys.argv[3].split() if len(sys.argv) > 3 else []
spur = []
plan = []
r = random.Random(17)
orig = vergleich.ereignis
besucht = {}
wechsel = []
LANG = int(os.environ.get("BS_LANG", "0"))


def mitte(rect):
    return "%d:%d" % rect.center


def feld(game, rr, cc):
    return "%d:%d" % (game.bx + cc * game.cb + game.cb // 2, game.by + rr * game.cb + game.cb // 2)


def bot(game, k, IE):
    n = len(spur)
    if n < len(vorlauf):
        spur.append(vorlauf[n])
        orig(game, vorlauf[n], IE)
        return
    taste = ""
    st = game.state
    if st != besucht.get("_l"):
        wechsel.append((n, st))
    besucht["_l"] = st
    besucht[st] = besucht.get(st, 0) + 1
    if plan:
        taste = plan.pop(0)
    elif n % 5 == 0:
        x = r.random()
        if st == "setup":
            if x < 0.5:
                taste = r.choice(["Return", "space", "@" + mitte(game.start_rect)])
            elif x < 0.7:
                taste = r.choice(["1", "2", "3", "Left", "Right", "Up", "Down"])
            elif game.rule_rects:
                taste = "@" + mitte(r.choice(game.rule_rects))
        elif st == "handover":
            taste = r.choice(["Return", "space", "@320:240"])
        elif st == "place":
            sea = game.seas[game.place_player]
            if x < 0.25:
                taste = r.choice(["x", "@" + mitte(game.btn_rects["random"])])
            elif x < 0.45 and sea.complete() and besucht.get("place", 0) % 400 > LANG:
                taste = r.choice(["Return", "@" + mitte(game.btn_rects["ready"])])
            elif x < 0.60:
                # aus dem Dock aufs Brett ziehen
                i = r.randrange(5)
                d = game.dock_rects[i]
                start = "%d:%d" % (d.x + 12, d.centery)
                ziel = feld(game, r.randrange(10), r.randrange(10))
                plan.extend(["<" + start, "~" + start, "", "~" + ziel, "", ">" + ziel])
                taste = plan.pop(0)
            elif x < 0.70:
                taste = "@" + feld(game, r.randrange(10), r.randrange(10))
                if r.random() < 0.5:
                    plan.append(r.choice(["r", "@" + feld(game, r.randrange(10), r.randrange(10))]))
            elif x < 0.78:
                taste = "@%s:3" % feld(game, r.randrange(10), r.randrange(10))
            elif x < 0.90:
                taste = r.choice(["Up", "Down", "Left", "Right", "space", "space", "r"])
            elif x < 0.94:
                taste = r.choice(["c", "@" + mitte(game.btn_rects["clear"]), "@" + mitte(game.btn_rects["rotate"])])
            else:
                taste = "~%d:%d" % (r.randrange(game.width), r.randrange(game.height))
        elif st == "play":
            if not game._is_ai(game.turn) and game.flight is None:
                if x < 0.7:
                    rr, cc = r.randrange(10), r.randrange(10)
                    plan.append("@" + feld(game, rr, cc))
                    taste = "~" + feld(game, rr, cc)
                elif x < 0.9:
                    taste = r.choice(["Up", "Down", "Left", "Right", "space", "Return"])
                else:
                    taste = "~%d:%d" % (r.randrange(game.width), r.randrange(game.height))
        elif st == "over":
            if game._now() - game.over_t0 > 1.2:
                taste = r.choice(["Return", "space", "@320:300", "s"] if x < 0.9 else ["s"])
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("battleship", os.environ.get("BS_MODE", "single"), 7, frames, ["x"] * frames, d,
                      os.path.join(sp, "bot.png"))
open(os.path.join(sp, "bs_keys.txt"), "w").write(",".join(spur))
print("Ende", besucht)
print("Wechsel", wechsel[:40])
