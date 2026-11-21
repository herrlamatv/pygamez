"""Bot fuer Lama Mines: spielt einen Plan im Python-Lauf ab und schreibt die
Tastenfolge. Das Minenfeld liest er aus dem Spielzustand (``g.is_mine`` -
es steht mit den Seeds fest), er zieht selbst keinen Zufall. Daraus stammen
die Laeufe in ``rust/tools/regression.py``.

    .venv\\Scripts\\python.exe rust\\tools\\bots\\bot_lamamines.py <ordner> 300 lmi.txt "w:5;k:space;ready;gem:2;k:Return"
    .venv\\Scripts\\python.exe rust\\tools\\vergleich.py lamamines --frames 300 --keys-file <ordner>\\lmi.txt

Plan (Befehle durch ';' getrennt):
    k:<taste>     Taste (auch +x/-x/*x wie vergleich.py)
    c:<x>:<y>     Linksklick          m:<x>:<y>   Maus bewegen
    r:<x>:<y>:<d> Mausrad (d Rasten, + = hoch)
    t:<i>         Klick auf Feld i     tm:<i>     Maus auf Feld i
    b:<name>      Klick auf einen Knopf (Attribut ``r_<name>`` des Spiels, z.B.
                  b:go, b:cash, b:random, b:plus, b:auto_start; b:tool:auto;
                  Listen mit Index: b:grids.2, b:presets.4, b:pills.0;
                  Auto-Fenster: b:ar.<zeile>.<knopf>)
    bm:<name>     wie b, aber nur die Maus darauf bewegen
    w:<n>         n Bilder warten
    ready         warten, bis eine Runde laeuft und kein Feld mehr aufklappt
    idle          warten auf den Ruhezustand      res   Ergebnis wegdrueckbar
    noauto        warten, bis der Auto-Modus fertig ist
    gem[:n]       n sichere Felder per Klick (dazwischen ``ready``)
    gemk[:n]      dasselbe mit Pfeilen + Leertaste
    mine / minek  eine Mine per Klick bzw. Tastatur
    clear         sichere Felder klicken, bis die Runde endet
Optional ``--daten <ordner>`` wie bei vergleich.py. Ausgabe zusaetzlich: die
Folge in Kurzform (``_*N`` = N leere Bilder, fuer ``regression.spur``).
"""
import os
import sys
sys.path.insert(0, "rust/tools")
import vergleich  # noqa: E402

sp, frames, out, plan_text = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
rest = sys.argv[5:]
if "--daten" in rest:
    vergleich.VORLAGE = os.path.abspath(rest[rest.index("--daten") + 1])
plan = [p.strip() for p in plan_text.split(";") if p.strip()]

from games import lamamines as LM  # noqa: E402

spur = []
orig = vergleich.ereignis
zustand = {"i": 0, "warten": 0, "zaehler": 0, "ziel": None}


def bereit(g):
    return (g.state == LM.PLAY and g.round is not None
            and all(c["flip"] >= 1.0 and c["delay"] <= 0 for c in g.cells))


def mitte(g, i):
    return "@%d:%d" % g.tile_r[i].center


def feld(g, mine):
    """Erstes verdecktes Feld (sicher bzw. Mine) in Platz-Reihenfolge."""
    for i, c in enumerate(g.cells):
        if c["face"] == "hidden" and g.is_mine[i] == mine:
            return i
    return None


def pfeil(g, ziel):
    """Naechste Taste, um den Tastatur-Cursor auf ziel zu bringen (oder space)."""
    if not g.kb:
        return "Left"
    n = g.grid
    cx, cy = g.cursor % n, g.cursor // n
    zx, zy = ziel % n, ziel // n
    if cx < zx:
        return "Right"
    if cx > zx:
        return "Left"
    if cy < zy:
        return "Down"
    if cy > zy:
        return "Up"
    return "space"


def bot(game, k, IE):
    g = game
    taste = ""
    while zustand["i"] < len(plan):
        p = plan[zustand["i"]]
        name, _, arg = p.partition(":")
        if name == "k":
            taste = arg
            zustand["i"] += 1
            break
        if name in ("c", "m"):
            taste = ("@" if name == "c" else "~") + arg
            zustand["i"] += 1
            break
        if name == "r":
            taste = "%" + arg
            zustand["i"] += 1
            break
        if name in ("t", "tm"):
            x, y = g.tile_r[int(arg)].center
            taste = ("@" if name == "t" else "~") + "%d:%d" % (x, y)
            zustand["i"] += 1
            break
        if name in ("b", "bm"):
            if arg.startswith("tool:"):
                r = dict(g.r_tools)[arg[5:]]
            elif arg.startswith("ar."):
                _, zeile, j = arg.split(".")
                r = g.auto_rows[int(zeile)][1][int(j)]
            elif "." in arg:
                nm, ix = arg.split(".")
                r = getattr(g, "r_" + nm)[int(ix)]
                if isinstance(r, tuple):        # r_fair_rows: (Index, Rechteck)
                    r = r[1]
            else:
                r = getattr(g, "r_" + arg)
            taste = ("@" if name == "b" else "~") + "%d:%d" % r.center
            zustand["i"] += 1
            break
        if name == "w":
            zustand["warten"] += 1
            if zustand["warten"] >= int(arg):
                zustand["warten"] = 0
                zustand["i"] += 1
            break
        if p == "ready":
            if bereit(g):
                zustand["i"] += 1
                continue
            break
        if p == "idle":
            if g.state == LM.IDLE:
                zustand["i"] += 1
                continue
            break
        if p == "res":
            if g.state in (LM.BOOM, LM.CASHED) and g._result_visible() and g.res_t >= LM.READ_T:
                zustand["i"] += 1
                continue
            break
        if p == "noauto":
            if g.auto is None:
                zustand["i"] += 1
                continue
            break
        if name in ("gem", "gemk", "mine", "minek", "clear"):
            if name == "clear" and g.state != LM.PLAY:
                zustand["i"] += 1
                continue
            if not bereit(g):
                break
            if zustand["ziel"] is None:
                zustand["ziel"] = feld(g, name.startswith("mine"))
                if zustand["ziel"] is None:
                    raise SystemExit("kein passendes Feld mehr: " + p)
            ziel = zustand["ziel"]
            if name.endswith("k"):
                taste = pfeil(g, ziel)
                if taste != "space":
                    break
            else:
                taste = mitte(g, ziel)
            zustand["ziel"] = None
            if name == "clear":
                break
            zustand["zaehler"] += 1
            if zustand["zaehler"] >= (int(arg) if arg else 1):
                zustand["zaehler"] = 0
                zustand["i"] += 1
            break
        raise SystemExit("unbekannter Planschritt: " + p)
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("lamamines", "single", 7, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
with open(os.path.join(sp, out), "w") as f:
    f.write(",".join(spur))
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
