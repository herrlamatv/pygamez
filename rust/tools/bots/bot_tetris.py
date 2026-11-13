"""Bot fuer Tetris: spielt mit der Spiel-KI (tetris_ai.plan, ohne Zufall)
Stein fuer Stein ueber echte Tasten (Schieben, Drehen in beide Richtungen,
Halten, Hard Drop, gelegentlich Soft Drop), streut zufaellige Tasten ein und
waehlt nach dem Ende Neustart, Setup oder die Knoepfe.

Aufruf: bot_tetris.py <ordner> <frames> [setup-tasten]
  TT_MODE=solo|versus_ai|multi   TT_LV=0..2 (Staerke des Bots, Vorgabe 2)
  TT_MUT=0..1  Anteil zufaelliger Tasten (Vorgabe 0.03)
  TT_P2=ki|zufall   zweiter Mensch im Mehrspieler (Vorgabe zufall)
  TT_TAKT=n   Bilder zwischen zwei Eingaben (Vorgabe 2)
Schreibt <ordner>/tt_keys.txt. Eigener Zufall (random.Random(31)).
"""
import os
import random
import sys
sys.path.insert(0, "rust/tools")
sys.path.insert(0, ".")
import vergleich
from games import tetris_ai

sp, frames = sys.argv[1], int(sys.argv[2])
vorlauf = sys.argv[3].split() if len(sys.argv) > 3 else []
spur = []
r = random.Random(31)
orig = vergleich.ereignis
MODUS = os.environ.get("TT_MODE", "solo")
LV = int(os.environ.get("TT_LV", "2"))
MUT = float(os.environ.get("TT_MUT", "0.03"))
P2 = os.environ.get("TT_P2", "zufall")
TAKT = int(os.environ.get("TT_TAKT", "2"))

# Aktion -> Taste je Spielerplatz
TASTEN = {
    "solo": dict(hold=["c", "Shift_L"], cw=["Up", "x"], ccw=["z", "y"], left=["Left"],
                 right=["Right"], drop=["space"], soft=["Down"]),
    "p1": dict(hold=["q"], cw=["w"], ccw=["e"], left=["a"], right=["d"], drop=["space"], soft=["s"]),
    "p2": dict(hold=["Shift_R"], cw=["Up"], ccw=["Control_R"], left=["Left"], right=["Right"],
               drop=["Return"], soft=["Down"]),
}
plaene = {}
besucht = {}
wechsel = []
enden = []
soft_an = {}


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
        if st == "over" and game.result:
            res = game.result
            enden.append((n, res.get("reason"), res.get("winner"),
                          [(b.lines, b.score) for b in game.boards]))
    besucht["_l"] = st
    besucht[st] = besucht.get(st, 0) + 1
    if st == "setup":
        if n % 20 == 0:
            taste = r.choice(["Return", "Return", "space", "@%d:%d" % game.setup_rects["start"].center])
    elif st == "over":
        if n % 45 == 0:
            x = r.random()
            if x < 0.55:
                taste = r.choice(["r", "R", "@%d:%d" % game.over_rects["again"].center])
            elif x < 0.75:
                taste = r.choice(["s", "@%d:%d" % game.over_rects["setup"].center])
    elif st == "play" and n % TAKT == 0:
        slots = [(0, "p1" if game.multiplayer else "solo")]
        if game.multiplayer:
            slots.append((1, "p2"))
        i, platz = slots[(n // TAKT) % len(slots)]
        b = game.boards[i]
        keys = TASTEN[platz]
        if platz in soft_an:
            taste = "-" + soft_an.pop(platz)
        elif r.random() < MUT or (platz == "p2" and P2 == "zufall"):
            if platz == "p2" and r.random() < 0.6:
                taste = ""
            else:
                taste = r.choice(sum(keys.values(), []))
                if taste in keys["soft"] and r.random() < 0.5:
                    soft_an[platz] = taste
                    taste = "+" + taste
        elif b.active:
            pid = (i, b.piece_id)
            if plaene.get(i, (None,))[0] != pid:
                plaene[i] = (pid, tetris_ai.plan(b, LV, None, True))
            acts = plaene[i][1]
            if acts:
                act = acts.pop(0)
                if act == "drop" and r.random() < 0.15:
                    acts.insert(0, "drop")
                    soft_an[platz] = keys["soft"][0]
                    taste = "+" + keys["soft"][0]
                else:
                    taste = r.choice(keys[act])
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("tetris", MODUS, 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
open(os.path.join(sp, "tt_keys.txt"), "w").write(",".join(spur))
print("Ende", {k: v for k, v in besucht.items() if k != "_l"})
print("Wechsel", wechsel[:30])
print("Enden", enden)
