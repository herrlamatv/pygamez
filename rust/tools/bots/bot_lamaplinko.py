"""Bot fuer Lama Plinko: spielt einen Plan im Python-Lauf ab (wartet auf
Landungen, Pleite, Auto-Ende ...) und schreibt die Tastenfolge. Daraus
stammen die Laeufe in ``rust/tools/regression.py``.

    .venv\\Scripts\\python.exe rust\\tools\\bots\\bot_lamaplinko.py <ordner> 600 lpl.txt "w:20;k:space;land;k:s"
    .venv\\Scripts\\python.exe rust\\tools\\vergleich.py lamaplinko --frames 600 --keys-file <ordner>\\lpl.txt

Plan (Befehle durch ';' getrennt):
    k:<token>     ein Eintrag wie in vergleich.py (space, +space, *space, -space,
                  @x:y, ~x:y, %x:y:d, Up, a, ...)
    c:<x>:<y>     Linksklick                     m:<x>:<y>  Maus bewegen
    w:<n>         n Bilder warten
    land          warten, bis keine Kugel mehr fliegt
    auto          warten, bis der Auto-Modus vorbei ist
    broke         warten, bis das Konto pleite ist
    drain         immer werfen (Leertaste), sobald nichts mehr fliegt - bis pleite
    big           Leertaste halten (Dauerfeuer), bis eine Kugel ab x10 gelandet ist
    slot:<i>      Maus ueber Fach i              icon:<i>   Klick auf Knopf i (A/S/F/T)
    hit:<name>    Klick auf ein Rechteck des Spiels (r_drop, r_minus, r_plus,
                  r_rows_m, r_rows_p, r_close, r_risks.<i>, r_icons.<i>)
    over:<name>   Maus ueber dieses Rechteck
    auto:<r>:<j>  Klick im Auto-Fenster auf Wert j von Zeile r (0 Kugeln, 1 Gewinn, 2 Verlust)
    autoturbo / autostart    Klick auf Turbo bzw. START/STOPP im Auto-Fenster
    fair:<i>      Klick auf Zeile i der Fairness-Liste;  ver = Klick auf Nachrechnen;
    newclient     Klick auf "Neuer Client-Seed"
    stat:<-|+>    Klick auf Reihen -/+ im Statistik-Fenster
    bar:<i>       Maus ueber Balken i der Treffer-Verteilung (Statistik-Fenster)
Optional ``--daten <ordner>`` und ``--seed N`` wie bei vergleich.py. Ausgabe
zusaetzlich: die Folge in Kurzform (``_*N`` = N leere Bilder, fuer
``regression.spur``).
"""
import os
import sys
sys.path.insert(0, "rust/tools")
import vergleich

sp, frames, out, plan_text = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
rest = sys.argv[5:]
seed = 7
if "--daten" in rest:
    vergleich.VORLAGE = os.path.abspath(rest[rest.index("--daten") + 1])
if "--seed" in rest:
    seed = int(rest[rest.index("--seed") + 1])
plan = [p.strip() for p in plan_text.split(";") if p.strip()]

spur = []
orig = vergleich.ereignis
zustand = {"i": 0, "warten": 0, "gross": False, "gehalten": False}


def mitte(r):
    return "%d:%d" % r.center


def rechteck(g, name):
    if "." in name:
        attr, idx = name.split(".")
        return getattr(g, attr)[int(idx)]
    return getattr(g, name)


def balken(g, i):
    """Mitte des Balkens i der Treffer-Verteilung (wie charts.bars)."""
    import ui
    body = g._panel_body()
    r_m, _ = g._stats_row_btns(body)
    area_y = r_m.bottom + 6
    area_h = body.bottom - r_m.bottom - 6
    leg_h = ui.font(g.f_tiny.get_height()).get_height()  # grob: nur fuer die Hoehe
    n = g.stats_rows + 1
    gap = max(2, min(8, body.w // (n * 5)))
    bar_w = max(3, (body.w - gap * (n - 1)) // n)
    x = body.x + i * (bar_w + gap) + bar_w // 2
    y = area_y + max(10, (area_h - leg_h) // 2)
    return "%d:%d" % (x, y)


def bot(game, k, IE):
    g = game
    taste = ""
    # gehaltene Leertaste (big): Wiederholung jedes Bild, sonst nichts
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
        if p == "land":
            if not g.balls:
                zustand["i"] += 1
                continue
            break
        if p == "auto":
            if g.auto is None:
                zustand["i"] += 1
                continue
            break
        if p == "broke":
            if g.broke:
                zustand["i"] += 1
                continue
            break
        if p == "drain":
            if g.broke:
                zustand["i"] += 1
                continue
            if not g.balls:
                taste = "space"
            break
        if p == "big":
            if any(pp["dur"] > 1.0 for pp in g.popups) and not zustand["gross"]:
                zustand["gross"] = True
            if zustand["gross"]:
                zustand["gross"] = False
                zustand["gehalten"] = False
                zustand["i"] += 1
                taste = "-space"
                break
            taste = "*space" if zustand["gehalten"] else "+space"
            zustand["gehalten"] = True
            break
        if p.startswith("slot:"):
            r = g.slot_rects[int(p[5:])]
            taste = "~" + mitte(r)
            zustand["i"] += 1
            break
        if p.startswith("icon:"):
            taste = "@" + mitte(g.r_icons[int(p[5:])])
            zustand["i"] += 1
            break
        if p.startswith("hit:") or p.startswith("over:"):
            name = p.split(":", 1)[1]
            taste = ("@" if p.startswith("hit:") else "~") + mitte(rechteck(g, name))
            zustand["i"] += 1
            break
        if p.startswith("auto:"):
            _, r, j = p.split(":")
            (rows, _turbo, _sum_y, _start), _hits = g._auto_rects()
            taste = "@" + mitte(rows[int(r)][1][int(j)][0])
            zustand["i"] += 1
            break
        if p in ("autoturbo", "autostart"):
            (_rows, turbo, _sum_y, start), _hits = g._auto_rects()
            taste = "@" + mitte(turbo if p == "autoturbo" else start)
            zustand["i"] += 1
            break
        if p.startswith("fair:"):
            _g, rows, _h = g._fair_rects()
            want = int(p[5:])
            ziel = [r for r, i in rows if i == want]
            taste = "@" + mitte(ziel[0]) if ziel else ""
            zustand["i"] += 1
            break
        if p in ("ver", "newclient"):
            geo = g._fair_geom()
            taste = "@" + mitte(geo["r_ver"] if p == "ver" else geo["r_new"])
            zustand["i"] += 1
            break
        if p.startswith("stat:"):
            r_m, r_p = g._stats_row_btns(g._panel_body())
            taste = "@" + mitte(r_m if p[5:] == "-" else r_p)
            zustand["i"] += 1
            break
        if p.startswith("bar:"):
            taste = "~" + balken(g, int(p[4:]))
            zustand["i"] += 1
            break
        raise SystemExit("unbekannter Planschritt: " + p)
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("lamaplinko", "single", seed, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
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
