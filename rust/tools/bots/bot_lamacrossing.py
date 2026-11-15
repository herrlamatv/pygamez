"""Bot fuer Lama Crossing 2.0: spielt einen Plan im Python-Lauf ab, wartet
dabei auf sichere (bzw. sichere Treffer-)Momente im Verkehr und schreibt die
Tastenfolge. Daraus stammen die Laeufe in ``rust/tools/regression.py``.

    .venv\\Scripts\\python.exe rust\\tools\\bots\\bot_lamacrossing.py <ordner> 560 lcr.txt "w:20;safe;idle;safe;wait;k:Return"
    .venv\\Scripts\\python.exe rust\\tools\\vergleich.py lamacrossing --frames 560 --keys-file <ordner>\\lcr.txt

Plan (Befehle durch ';' getrennt):
    k:<taste>    Taste            c:<x>:<y>   Linksklick      m:<x>:<y>  Maus bewegen
    w:<n>        n Bilder warten  idle / wait / res   warten auf IDLE / WAIT /
                 wegklickbares Ergebnis
    safe         in WAIT (oder IDLE) warten, bis die naechste Spur bei der Landung
                 frei ist -> space
    safe+        wie safe, immer wieder, bis ein Fehltritt passiert (FALL)
    safeclick    wie safe, aber Klick auf die naechste Spur
    unsafe       warten, bis ein Treffer bei der Landung sicher ist -> space
Optional ``--daten <ordner>`` wie bei vergleich.py. Ausgabe zusaetzlich: die
Folge in Kurzform (``_*N`` = N leere Bilder, fuer ``regression.spur``).
"""
import os
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames, out, plan_text = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
rest = sys.argv[5:]
if "--daten" in rest:
    vergleich.VORLAGE = os.path.abspath(rest[rest.index("--daten") + 1])
plan = [p.strip() for p in plan_text.split(";") if p.strip()]

from games import lamacrossing as LC  # noqa: E402
from games import lamacrossing_draw as D  # noqa: E402
from games import lamacrossing_logic as L  # noqa: E402

DT = 1.0 / 60.0
HOP_BILDER = 22          # HOP_T = 0.36 s -> Landung im 22. Bild
spur = []
orig = vergleich.ereignis
zustand = {"i": 0, "warten": 0}


def treffer(g, b, rand):
    """Steht bei der Landung (in HOP_BILDER Bildern) ein Hindernis auf Spur b?
    rand > 0 = vorsichtig, rand < 0 = nur sichere Treffer."""
    if not 1 <= b < len(g.lanes):
        return False
    ty = g._target_y()
    reach = g.llama_n * 0.3
    f = L.SPEED_F[g.world]
    lane = g.lanes[b]
    kandidaten = [(o["y"], o["v"], o["kind"], 0) for o in lane["obs"] if o["fade"] is None]
    t = lane["t"]
    for j in range(1, HOP_BILDER + 1):
        t -= DT
        if t <= 0:
            top = g._horizon()
            y0 = top if g.world in ("river", "ice") else top - D.obstacle_len(g.world, 3, g._obs_size(top)) / 2
            kandidaten.append((y0, lane["v"], 3, j))
            break
    for y, v, kind, j0 in kandidaten:
        yl = y + v * f * DT * (HOP_BILDER - j0)
        olen = D.obstacle_len(g.world, kind, g._obs_size(yl))
        if abs(yl - ty) < olen / 2 + reach + rand:
            return True
    return False


def bot(game, k, IE):
    g = game
    taste = ""
    while zustand["i"] < len(plan):
        p = plan[zustand["i"]]
        if p.startswith("k:"):
            taste = p[2:]
            zustand["i"] += 1
            break
        if p[:2] in ("c:", "m:"):
            _, x, y = p.split(":")
            taste = ("@" if p[0] == "c" else "~") + x + ":" + y
            zustand["i"] += 1
            break
        if p.startswith("w:"):
            zustand["warten"] += 1
            if zustand["warten"] >= int(p[2:]):
                zustand["warten"] = 0
                zustand["i"] += 1
            break
        if p == "safe+" and g.state == LC.FALL:
            zustand["i"] += 1
            continue
        if p in ("safe", "safe+", "safeclick", "unsafe"):
            if g.state == LC.WAIT or (g.state == LC.IDLE and p != "safeclick"):
                b = g.step + 1 if g.state == LC.WAIT else 1
                if p == "unsafe":
                    los = treffer(g, b, -12 * g.k)
                else:
                    los = not treffer(g, b, 14 * g.k)
                if los:
                    taste = "@%d:%d" % g._next_tile_rect().center if p == "safeclick" else "space"
                    if p != "safe+":
                        zustand["i"] += 1
            break
        ziel = {"idle": (LC.IDLE,), "wait": (LC.WAIT,), "res": (LC.FALL, LC.CASHED, LC.GOAL)}.get(p)
        if ziel is None:
            raise SystemExit("unbekannter Planschritt: " + p)
        if g.state in ziel and (p != "res" or g.res_t > 0.6):
            zustand["i"] += 1
            continue
        break
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("lamacrossing", "single", 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
open(os.path.join(sp, out), "w").write(",".join(spur))
kurz, leer = [], 0
for taste in spur:
    if taste:
        if leer:
            kurz.append("_*%d" % leer)
        kurz.append(taste)
        leer = 0
    else:
        leer += 1
print("fertig", len(spur), "- Plan uebrig:", plan[zustand["i"]:])
print(" ".join(kurz))
