"""Bot fuer Minigolf.

Spielen: waehlt im Setup Kurs/Tour/Schalter, zielt mit der Maus aufs Loch
(mit Streuung), laedt per gedrueckter Maustaste unterschiedlich lange oder
stellt die Staerke mit Pfeilen ein, haelt manchmal die Staerke-Sperre
(rechte Maustaste), bricht mit R ab, setzt mit F zurueck, schaltet G/Z/P und
nutzt am Rundenende Weiter/Nochmal/Setup.

MG_EDIT=1: geht in den MAPS-Reiter, legt neue Bahnen an, baut im Editor
(Vorlagen, Waende aufziehen, Kreise setzen, Rohre, Auswahl/Verschieben,
Parameter +/-, Raster, Undo/Redo, Leeren), tippt Name/id, speichert, testet
die Bahn und spielt sie aus der Liste; Loeschen mit Bestaetigung.

Aufruf: bot_minigolf.py <ordner> <frames>   (MG_MODE=single|multi)
Schreibt <ordner>/mg_keys.txt. Eigener Zufall (random.Random(53)).
"""
import math
import os
import random
import sys
sys.path.insert(0, "rust/tools")
sys.path.insert(0, ".")
import vergleich

sp, frames = sys.argv[1], int(sys.argv[2])
spur = []
plan = []
r = random.Random(int(os.environ.get("MG_BOTSEED", "53")))
orig = vergleich.ereignis
MODUS = os.environ.get("MG_MODE", "single")
EDIT = os.environ.get("MG_EDIT", "0") == "1"
VORLAUF = os.environ.get("MG_VORLAUF", "").split()
besucht = {}
wechsel = []
ergebnisse = []


def pt(x, y):
    return "%d:%d" % (int(x), int(y))


def mitte(rc):
    return pt(*rc.center)


def setup(game, n):
    if n % 10:
        return ""
    x = r.random()
    if EDIT:
        if game.setup_tab != "maps":
            return r.choice(["Tab", "@" + mitte(game.tab_rects[1])])
        return maps(game, n)
    if x < 0.15:
        return r.choice(["1", "2", "3", "4", "5", "@" + mitte(r.choice(game.course_rects))])
    if x < 0.25:
        return r.choice(["Left", "Right", "@" + mitte(game.tour_rects[0]), "@" + mitte(game.tour_rects[2])])
    if x < 0.32:
        return r.choice(["g", "z", "p", "@" + mitte(r.choice(game.guide_rects + game.autoaim_rects + game.pickup_rects))])
    if x < 0.7:
        return r.choice(["Return", "space", "@" + mitte(game.start_rect)])
    return ""


def maps(game, n):
    ml = game.maps
    if ml is None:
        return ""
    if ml.share is not None:
        return r.choice(["Escape", "@" + mitte(ml.share_btn["cancel"]), "Tab", "a", "BackSpace"])
    x = r.random()
    if not ml.items or x < 0.35:
        return r.choice(["n", "@" + mitte(ml.btn_rects["new"])])
    if x < 0.5:
        return r.choice(["Up", "Down"])
    if x < 0.65:
        return r.choice(["Return", "@" + mitte(ml.btn_rects["play"])])
    if x < 0.75:
        return r.choice(["e", "@" + mitte(ml.btn_rects["edit"])])
    if x < 0.82:
        return r.choice(["Delete", "@" + mitte(ml.btn_rects["delete"])])
    if x < 0.86:
        return "@" + mitte(ml.btn_rects["share"])
    if x < 0.9:
        return "@" + mitte(ml.btn_rects["import"])
    return ""


def editor(game, n):
    ed = game.editor
    if n % 6:
        return ""
    if ed.picking:
        return "@" + mitte(r.choice(ed.tpl_rects)[1]) if r.random() < 0.8 else "Escape"
    x = r.random()
    c = ed.canvas
    if ed.focus >= 0:
        if x < 0.75:
            return r.choice(list("lamabahn") + ["space", "BackSpace", "Left", "Right", "Home", "End"])
        return r.choice(["Tab", "Return", "Escape"])
    if x < 0.08:
        return "@" + mitte(r.choice([ed.name_rect, ed.id_rect]))
    if x < 0.14:
        return "@" + mitte(ed.edit_rects["template"])
    if x < 0.3:
        key, rc = r.choice(ed.pal_rects)
        return "@" + mitte(rc)
    if x < 0.55:
        x0 = r.randint(c.x + 10, c.right - 10)
        y0 = r.randint(c.y + 10, c.bottom - 10)
        if r.random() < 0.6:
            x1 = min(c.right - 2, x0 + r.randint(5, 60))
            y1 = min(c.bottom - 2, y0 + r.randint(5, 40))
            plan.extend(["~" + pt(x0 + 3, y0 + 2), "~" + pt(x1, y1), ">" + pt(x1, y1)])
            return "<" + pt(x0, y0)
        return "@" + pt(x0, y0)
    if x < 0.65:
        rects = ed._param_rects()
        if rects:
            p, minus, val, plus = r.choice(rects)
            return "@" + mitte(r.choice([minus, plus]))
        return "@" + mitte(ed.pal_rects[0][1])
    if x < 0.72:
        k = r.choice(list(ed.num_rects))
        return "@" + mitte(r.choice([ed.num_rects[k][0], ed.num_rects[k][2]]))
    if x < 0.8:
        return r.choice(["u", "y", "g", "Delete", "@" + mitte(ed.edit_rects["undo"]), "@" + mitte(ed.edit_rects["redo"]),
                         "@" + mitte(ed.edit_rects["grid"])])
    if x < 0.83:
        return "@" + mitte(ed.edit_rects["clear"])
    if x < 0.9:
        return r.choice(["s", "@" + mitte(ed.save_rect)])
    if x < 0.95:
        return r.choice(["Return", "@" + mitte(ed.edit_rects["test"])])
    return r.choice(["Escape", "@" + mitte(ed.back_rect)])


def spielen(game, n):
    if game.state == "holedone":
        return r.choice(["Return", "space", "@320:240"]) if n % 20 == 0 else ""
    if game.state == "over":
        if n % 40:
            return ""
        x = r.random()
        if x < 0.6:
            return "@" + mitte(r.choice(game.over_rects)[1])
        return r.choice(["Return", "r", "s"])
    if game.phase != "aim" or n % 5:
        return ""
    x = r.random()
    if x < 0.03:
        return r.choice(["g", "z", "p"])
    if x < 0.04:
        return "f"
    cx, cy = game._project(*game.cup)
    bx, by = game._project(game.bx, game.by)
    zx = cx + r.uniform(-12, 12)
    zy = cy + r.uniform(-12, 12)
    d = math.hypot(game.cup[0] - game.bx, game.cup[1] - game.by)
    ziel = max(0.12, min(1.0, 0.1 + d / 140.0 + r.uniform(-0.08, 0.12)))
    if x < 0.45:
        # Maustaste halten: Kraft laedt mit 0.8/s ab 0.05
        lade = max(1, int((ziel - 0.05) / 0.8 * 60))
        plan.append("<" + pt(zx, zy))
        plan.extend([""] * lade)
        if r.random() < 0.06:
            plan.append("r")
            plan.extend(["", "", ""])
        if r.random() < 0.1:
            plan.extend(["<%s:3" % pt(zx, zy)] + [""] * 30 + [">%s:3" % pt(zx, zy)])
        plan.append(">" + pt(zx, zy))
        return "~" + pt(zx, zy)
    # Tastatur: Pfeile fuer Richtung/Staerke, Leertaste
    plan.append("~" + pt(zx, zy))
    stufen = int(round((ziel - game.power) / 0.05))
    taste = "Up" if stufen > 0 else "Down"
    plan.extend([taste, ""] * min(18, abs(stufen)))
    plan.extend([r.choice(["Left", "Right", ""]), "space"])
    return plan.pop(0)


def bot(game, k, IE):
    n = len(spur)
    if n < len(VORLAUF):
        spur.append(VORLAUF[n])
        orig(game, VORLAUF[n], IE)
        return
    st = game.state
    zst = st + (":" + game.setup_tab if st == "setup" else "")
    if zst != besucht.get("_l"):
        wechsel.append((n, zst))
        plan.clear()
        if st == "over":
            ergebnisse.append((n, game.course, game.tour, sum(game.cards[0])))
    besucht["_l"] = zst
    besucht[zst] = besucht.get(zst, 0) + 1
    if plan:
        taste = plan.pop(0)
    elif st == "setup":
        taste = setup(game, n)
    elif st == "edit":
        taste = editor(game, n)
    else:
        taste = spielen(game, n)
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("minigolf", MODUS, int(os.environ.get("MG_SEED", "7")), frames, ["x"] * frames, d,
                      os.path.join(sp, "bot.png"))
open(os.path.join(sp, "mg_keys.txt"), "w").write(",".join(spur))
print("Ende", {k: v for k, v in besucht.items() if not k.startswith("_")})
print("Wechsel", wechsel[:40])
print("Ergebnisse", ergebnisse)
