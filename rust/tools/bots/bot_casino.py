"""Bot fuer das Casino.

Roulette: waehlt Chips (Tasten/Klick), setzt auf zufaellige Tischstellen
(Zahlen, Kanten, Ecken, Aussenfelder), nimmt per Rechtsklick weg, bewegt die
Maus ueber den Tisch (Hervorhebung), verdoppelt/wiederholt/loescht, dreht,
spult vor und nutzt die Ergebnis-Knoepfe. Mit CA_RISIKO=1 setzt er hoch
(500er-Chips), bis er pleite ist, und nimmt dann den Bank-Kredit.

Slot: dreht per Taste/Klick/Walzenfenster, schaltet Turbo, startet Auto-Spin
(Taste und Menue), oeffnet die Gewinntabelle, aendert den Einsatz und
faehrt ueber die Linien-Reiter.

Aufruf: bot_casino.py <ordner> <frames>   (CA_MODE=roulette|slots, CA_SEED=7)
Schreibt <ordner>/ca_keys.txt. Eigener Zufall (random.Random(37)).
"""
import os
import random
import sys
sys.path.insert(0, "rust/tools")
sys.path.insert(0, ".")
import vergleich

sp, frames = sys.argv[1], int(sys.argv[2])
spur = []
r = random.Random(int(os.environ.get("CA_BOTSEED", "37")))
orig = vergleich.ereignis
MODUS = os.environ.get("CA_MODE", "roulette")
RISIKO = os.environ.get("CA_RISIKO", "0") == "1"
DAUER = os.environ.get("CA_DAUER", "0") == "1"     # Slot: fast nur drehen (Turbo)
besucht = {}
wechsel = []
ereignisse = []


def pt(rect, rx=None, ry=None):
    rx = r.random() if rx is None else rx
    ry = r.random() if ry is None else ry
    return "%d:%d" % (rect.x + int(rect.w * rx), rect.y + int(rect.h * ry))


def roulette(game, n):
    ph = game.r_phase
    if game.broke:
        return r.choice(["Return", "@320:240", "space"]) if n % 30 == 0 else ""
    if ph == "spin":
        return r.choice(["space", "", "", "@100:100"]) if n % 50 == 0 else ""
    if ph == "result":
        if n % 40 != 0:
            return ""
        x = r.random()
        if x < 0.3:
            return "@" + pt(game.r_btns["spin"], 0.5, 0.5)
        if x < 0.5:
            return "@" + pt(game.r_btns["rebet"], 0.5, 0.5)
        if x < 0.7:
            return r.choice(["Return", "r"])
        return ""
    # Setzen
    if n % 6 != 0:
        return ""
    tisch = game.r_table_rect
    x = r.random()
    if RISIKO:
        if not game.r_bets:
            return r.choice(["5", "@" + pt(game.r_chip_rects[4][1], 0.5, 0.5)]) if game.r_chip != 500 else \
                "@" + pt(tisch, r.uniform(0.1, 0.9), r.uniform(0.85, 0.98))
        if sum(game.r_bets.values()) < 900 and x < 0.5:
            return "@" + pt(tisch, r.uniform(0.1, 0.9), r.uniform(0.05, 0.6))
        return r.choice(["space", "@" + pt(game.r_btns["spin"], 0.5, 0.5), "d"])
    if x < 0.12:
        return r.choice(["1", "2", "3", "4", "5", "@" + pt(r.choice(game.r_chip_rects)[1], 0.5, 0.5)])
    if x < 0.45:
        return "@" + pt(tisch)
    if x < 0.6:
        return "~" + pt(tisch)
    if x < 0.66:
        return "@" + pt(tisch) + ":3"
    if x < 0.72:
        return r.choice(["d", "c", "r", "BackSpace", "@" + pt(game.r_btns["double"], 0.5, 0.5),
                         "@" + pt(game.r_btns["clear"], 0.5, 0.5)])
    if x < 0.82 and game.r_bets:
        return r.choice(["space", "Return", "@" + pt(game.r_btns["spin"], 0.5, 0.5)])
    if x < 0.86:
        return "~" + pt(game.r_panel)
    return ""


def slots(game, n):
    if game.broke:
        return r.choice(["Return", "@320:240"]) if n % 30 == 0 else ""
    if game.s_paytable:
        return r.choice(["i", "Return", "@320:240"]) if n % 50 == 0 else ""
    if n % 9 != 0:
        return ""
    x = r.random()
    if DAUER and not game.s_auto_menu:
        if not game.s_turbo:
            return "t"
        if x < 0.9:
            return "space" if game.s_phase != "spin" else ""
    if game.s_auto_menu:
        if x < 0.7:
            return "@" + pt(r.choice(game.s_auto_opts)[1], 0.5, 0.5)
        return "@20:20"
    if x < 0.4:
        return r.choice(["space", "Return", "@" + pt(game.s_btns["spin"], 0.5, 0.5), "@" + pt(game.s_win)])
    if x < 0.47:
        return r.choice(["t", "@" + pt(game.s_btns["turbo"], 0.5, 0.5)])
    if x < 0.53:
        return r.choice(["a", "A", "@" + pt(game.s_btns["auto"], 0.5, 0.5)])
    if x < 0.56:
        return r.choice(["i", "p", "@" + pt(game.s_btns["pay"], 0.5, 0.5)])
    if x < 0.64:
        return r.choice(["Up", "Down", "plus", "minus", "@" + pt(game.s_minus, 0.5, 0.5),
                         "@" + pt(game.s_plus, 0.5, 0.5)])
    if x < 0.74:
        return "~" + pt(r.choice(list(game.s_tabs.values())), 0.5, 0.5)
    if x < 0.78:
        return "~320:200"
    return ""


def bot(game, k, IE):
    n = len(spur)
    if MODUS == "roulette":
        zst = "pleite" if game.broke else game.r_phase
        if game.r_phase == "result" and besucht.get("_l") != "result":
            ereignisse.append((n, "zahl", game.r_number, game.r_payout, list(game.r_winners)[:3]))
        taste = roulette(game, n)
    else:
        zst = "pleite" if game.broke else ("tabelle" if game.s_paytable else game.s_phase)
        if game.s_banners:
            b = game.s_banners[0]
            key = (b["text"], b["kind"])
            if besucht.get("_b") != key:
                ereignisse.append((n, b["text"], b["amount"]))
            besucht["_b"] = key
        else:
            besucht["_b"] = None
        taste = slots(game, n)
    if zst != besucht.get("_l"):
        wechsel.append((n, zst))
    besucht["_l"] = zst
    besucht[zst] = besucht.get(zst, 0) + 1
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("casino", MODUS, int(os.environ.get("CA_SEED", "7")), frames, ["x"] * frames, d,
                      os.path.join(sp, "bot.png"))
open(os.path.join(sp, "ca_keys.txt"), "w").write(",".join(spur))
print("Ende", {k: v for k, v in besucht.items() if not k.startswith("_")})
print("Wechsel", wechsel[:40])
print("Ereignisse", ereignisse[:40])
