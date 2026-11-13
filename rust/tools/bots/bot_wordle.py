"""Bot fuer Wordle: tippt Loesungen, erlaubte Woerter, Unsinn und zu kurze
Eingaben (per Taste oder Bildschirmtastatur), loescht, wechselt im Setup
Laenge/hart/farbenblind, geht ueber das Regler-Symbol zurueck und bedient
das Ergebnis-Panel.

Aufruf: bot_wordle.py <ordner> <frames> [setup-tasten]   (WD_MODE=endless|daily|dordle|quordle)
WD_SHARE=1 drueckt auch "c" (Teilen - schreibt in die echte Zwischenablage).
Schreibt <ordner>/wd_keys.txt. Eigener Zufall (random.Random(41)).
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
r = random.Random(41)
orig = vergleich.ereignis
TEILEN = os.environ.get("WD_SHARE") == "1"
besucht = {}
wechsel = []
DIREKT = os.environ.get("WD_DIREKT") == "1"
SCHWACH = os.environ.get("WD_SCHWACH") == "1"


def mitte(rect):
    return "@%d:%d" % rect.center


def wort_tippen(game, wort):
    plan.extend(["BackSpace"] * len(game.current))
    for ch in wort:
        if r.random() < 0.2 and ch in game.key_rects:
            plan.append(mitte(game.key_rects[ch]))
        else:
            plan.append(ch.lower())
        plan.extend([""] * r.randint(0, 3))
    if r.random() < 0.2 and game.enter_rect is not None:
        plan.append(mitte(game.enter_rect))
    else:
        plan.append("Return")


def bot(game, k, IE):
    n = len(spur)
    if n < len(vorlauf):
        spur.append(vorlauf[n])
        orig(game, vorlauf[n], IE)
        return
    taste = ""
    st = game.state
    if st != besucht.get("_letzt"):
        wechsel.append((n, st))
    besucht["_letzt"] = st
    besucht[st] = besucht.get(st, 0) + 1
    if st == "done" and getattr(game, "daily_view", False):
        besucht["daily_view"] = besucht.get("daily_view", 0) + 1
    if plan:
        taste = plan.pop(0)
    elif n % 6 == 0:
        x = r.random()
        if st == "setup":
            if DIREKT:
                taste = "Return"
            elif x < 0.30:
                taste = r.choice(["Return", "space", mitte(game.su_start)])
            elif x < 0.45:
                taste = r.choice(["4", "5", "6", "7", "Left", "Right", mitte(r.choice(game.su_seg))])
            elif x < 0.60:
                taste = r.choice(["h", "f", mitte(game.su_hard), mitte(game.su_cb)])
            elif x < 0.75:
                taste = r.choice(["Up", "Down"])
            else:
                rc = r.choice([game.su_hard, game.su_cb, game.su_start, game.su_seg[0]])
                taste = "~%d:%d" % rc.center
        elif st == "play":
            offen = [b for b in game.boards if not b.solved]
            if x < (0.04 if SCHWACH else 0.30) and offen:
                wort_tippen(game, r.choice(offen).answer)
            elif x < 0.70:
                wort_tippen(game, r.choice(game.words))
            elif x < 0.78:
                wort_tippen(game, "".join(r.choice("ABCDEFGHIJKLMNOPQRSTUVWXYZ")
                                          for _ in range(game.length)))
            elif x < 0.84:
                wort_tippen(game, "AB")
            elif x < 0.92:
                plan.extend(r.choice("ETAOINSR").lower() for _ in range(r.randint(1, 4)))
                plan.extend(["BackSpace"] * r.randint(1, 3))
                if game.del_rect is not None:
                    plan.append(mitte(game.del_rect))
            elif x < 0.95 and not SCHWACH:
                taste = mitte(game.gear_rect)
            else:
                taste = "%d" % r.randint(0, 9)
        elif st == "solved":
            taste = r.choice(["Return", "space", "@320:300"] + (["c"] if TEILEN else []))
        elif st == "done" and game.result and game.anim - game.result["t0"] < 1.3:
            pass
        elif st == "done":
            if x < 0.35:
                taste = r.choice(["Return", "KP_Enter"])
            elif x < 0.50:
                taste = "space"
            elif x < 0.62:
                taste = r.choice(["s", "BackSpace"])
            elif x < 0.80:
                knoepfe = [rc for rc, _ in getattr(game, "_done_buttons", [])]
                if knoepfe:
                    rc = r.choice(knoepfe[1:] or knoepfe) if not TEILEN else r.choice(knoepfe)
                    taste = mitte(rc)
            elif x < 0.90:
                pos = (r.randrange(game.width), r.randrange(game.height))
                knoepfe = [rc for rc, _ in getattr(game, "_done_buttons", [])]
                if TEILEN or not (knoepfe and knoepfe[0].collidepoint(pos)):
                    taste = "@%d:%d" % pos
            elif TEILEN:
                taste = "c"
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("wordle", os.environ.get("WD_MODE", "endless"), 7, frames, ["x"] * frames, d,
                      os.path.join(sp, "bot.png"))
open(os.path.join(sp, "wd_keys.txt"), "w").write(",".join(spur))
print("Ende", besucht)
print("Wechsel", [w for w in wechsel if w[1] in ("done", "solved")][:12])
