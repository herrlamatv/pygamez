"""Bot fuer Pinball: laedt den Abschuss, schlaegt mit den Flippern, stupst ab und zu.

Aufruf: bot_pinball.py <ordner> <frames> [setup-tasten]   (PIN_MODE=multi fuer zwei Spieler, PIN_PASSIV=1 ohne Flipper)
Schreibt <ordner>/pin_keys.txt.
"""
import os
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames = sys.argv[1], int(sys.argv[2])
vorlauf = (sys.argv[3].split() if len(sys.argv) > 3 else []) + ["Return"]
spur = []
zustand = {"Left": False, "Right": False, "space": 0, "halten": 0}
orig = vergleich.ereignis


def bot(game, k, IE):
    n = len(spur)
    if n < len(vorlauf):
        spur.append(vorlauf[n])
        orig(game, vorlauf[n], IE)
        return
    taste = ""
    if game.state == "play":
        if game.phase == "launch":
            if zustand["space"] == 0 and not game.charging:
                taste = "+space"
                zustand["space"] = 1
            elif game.charging:
                zustand["space"] += 1
                # unterschiedlich lange laden
                if zustand["space"] > 30 + (n % 50):
                    taste = "-space"
                    zustand["space"] = 0
        else:
            nah = {"Left": False, "Right": False}
            for b in (game.balls if not os.environ.get("PIN_PASSIV") else []):
                if b.y > 122 and b.vy > -20:
                    if b.x < 50:
                        nah["Left"] = True
                    else:
                        nah["Right"] = True
            for seite in ("Left", "Right"):
                if taste:
                    break
                # nicht ewig halten (sonst ruht die Kugel auf dem Flipper)
                if zustand[seite] and zustand["halten"] > 24:
                    taste = "-" + seite
                    zustand[seite] = False
                    zustand["halten"] = -30
                elif nah[seite] and not zustand[seite] and zustand["halten"] >= 0:
                    taste = "+" + seite
                    zustand[seite] = True
                elif not nah[seite] and zustand[seite]:
                    taste = "-" + seite
                    zustand[seite] = False
            zustand["halten"] += 1 if (zustand["Left"] or zustand["Right"]) else 0
            if zustand["halten"] < 0:
                zustand["halten"] += 1
            elif not (zustand["Left"] or zustand["Right"]):
                zustand["halten"] = 0
            if not taste and n % 400 == 199:
                taste = "Up"
    elif game.state == "over" and n % 120 == 0:
        taste = "Return"
        zustand.update(Left=False, Right=False, space=0)
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("pinball", os.environ.get("PIN_MODE", "single"), 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
open(os.path.join(sp, "pin_keys.txt"), "w").write(",".join(spur))
print("Ende")
