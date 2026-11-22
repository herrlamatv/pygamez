"""Bot fuer Lama Crash: spielt einen Plan im Python-Lauf ab und schreibt die
Tastenfolge. Den Platzpunkt liest er aus dem Spielzustand (``g._crash`` - er
steht mit den Seeds fest), er zieht selbst keinen Zufall. Daraus stammen die
Laeufe in ``rust/tools/regression.py``.

    .venv\\Scripts\\python.exe rust\\tools\\bots\\bot_lamacrash.py <ordner> 600 lcx.txt "w:5;k:space;at:150;k:space;state:end"
    .venv\\Scripts\\python.exe rust\\tools\\vergleich.py lamacrash --frames 600 --keys-file <ordner>\\lcx.txt

Plan (Befehle durch ';' getrennt):
    k:<eintrag>   ein Eintrag wie in vergleich.py (Taste, +x/-x/*x, @x:y,
                  ~x:y, %x:y:d, !blur = Fokusverlust)
    c:<x>:<y>     Linksklick          m:<x>:<y>   Maus bewegen
    r:<x>:<y>:<d> Mausrad (d Rasten, + = hoch)
    b:<name>      Klick auf ein Rechteck des Spiels: go, half, total, stage,
                  auto_close, auto_turbo, auto_start, fair_new, fair_verify,
                  close (Fenster-Schliessen), tool.<auto|stats|fair|turbo|music>,
                  card.<i>.<feld> (sw, minus, bet, plus, auto, tl, tv, tr, cash,
                  halfc, row_a, row_b, rect), ar.<zeile>.<j> (Auto-Fenster),
                  fair.<idx> (Zeile der Fairness-Liste zum Log-Index idx),
                  bar.<i> (Balken i der Platzpunkt-Verteilung im Statistik-Fenster)
    bm:<name>     wie b, aber nur die Maus darauf bewegen
    w:<n>         n Bilder warten
    state:<s>     warten auf den Zustand idle / count / fly / coast / end
    at:<m100>     im Flug warten, bis der Multiplikator >= m100 ist (Fehler,
                  wenn die Runde vorher endet)
    tick:<t>      im Flug/Leerflug warten, bis der Takt >= t ist
    close         im Flug warten auf einen "Haarscharf"-Moment (>= x2 und
                  hoechstens 5 % unter dem Platzpunkt), dann Leertaste
    end           warten, bis die Runde abgeschlossen ist (Ergebnis oder Ruhe)
    noauto        warten, bis der Auto-Modus fertig ist
    broke         warten, bis das Konto pleite ist
    info          Platzpunkt/Zustand ausgeben (kostet kein Bild)
Optional ``--daten <ordner>`` und ``--seed N`` wie bei vergleich.py. Ausgabe
zusaetzlich: die Folge in Kurzform (``_*N`` = N leere Bilder, fuer
``regression.spur``) und je Runde der Platzpunkt.
"""
import os
import sys
sys.path.insert(0, "rust/tools")
import vergleich  # noqa: E402

sp, frames, out, plan_text = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
rest = sys.argv[5:]
seed = 7
if "--daten" in rest:
    vergleich.VORLAGE = os.path.abspath(rest[rest.index("--daten") + 1])
if "--seed" in rest:
    seed = int(rest[rest.index("--seed") + 1])
plan = [p.strip() for p in plan_text.split(";") if p.strip()]

import charts  # noqa: E402
from games import lamacrash as LC  # noqa: E402
from games import lamacrash_logic as L  # noqa: E402

spur = []
orig = vergleich.ereignis
zustand = {"i": 0, "warten": 0, "bild": 0, "runden": []}

# Balken der Verteilung (Statistik-Fenster): charts.bars liefert die Zellen -
# mitschreiben, ohne am Bild etwas zu aendern.
_balken = []
_bars = charts.bars


def _bars_mit(*a, **kw):
    r = _bars(*a, **kw)
    _balken[:] = list(r)
    return r


charts.bars = _bars_mit


def rechteck(g, name):
    if name == "close":
        return g._window_rect()[1]
    if name.startswith("tool."):
        return dict(g.r_tools)[name[5:]]
    if name.startswith("card."):
        _, i, feld = name.split(".")
        return g.cards[int(i)][feld]
    if name.startswith("ar."):
        _, zeile, j = name.split(".")
        return g.auto_rows[int(zeile)][1][int(j)]
    if name.startswith("fair."):
        idx = int(name[5:])
        for i, r in g.r_fair_rows:
            if i == idx:
                return r
        raise SystemExit("Fairness-Zeile %d nicht sichtbar" % idx)
    if name.startswith("bar."):
        i = int(name[4:])
        for cell, j in _balken:
            if j == i:
                return cell
        raise SystemExit("Balken %d nicht gezeichnet" % i)
    if name in ("stage",):
        return g.stage
    return getattr(g, "r_" + name)


def mult(g):
    return L.mult_at(g.round["t"]) if g.round is not None else 0


def bot(game, k, IE):
    g = game
    taste = ""
    zustand["bild"] += 1
    # Platzpunkt jeder neuen Runde merken (Ausgabe am Ende)
    if g.round is not None and (not zustand["runden"] or zustand["runden"][-1][0] != g.round["n"]):
        zustand["runden"].append((g.round["n"], g._crash, zustand["bild"]))
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
        if name in ("b", "bm"):
            r = rechteck(g, arg)
            taste = ("@" if name == "b" else "~") + "%d:%d" % r.center
            zustand["i"] += 1
            break
        if name == "w":
            zustand["warten"] += 1
            if zustand["warten"] >= int(arg):
                zustand["warten"] = 0
                zustand["i"] += 1
            break
        if name == "info":
            print("Bild %d: Zustand %s, Takt %s, x%s, Platzpunkt %s, Konto %d"
                  % (zustand["bild"], g.state, g.round["t"] if g.round else "-", mult(g), g._crash,
                     LC.lamabank.balance()))
            zustand["i"] += 1
            continue
        if name == "state":
            if g.state == arg:
                zustand["i"] += 1
                continue
            break
        if name == "at":
            want = int(arg)
            if g.state == LC.FLY and g.round is not None and mult(g) >= want:
                zustand["i"] += 1
                continue
            if g.state not in (LC.COUNT, LC.FLY) or (g._crash is not None and g._crash < want):
                raise SystemExit("at:%d unerreichbar (Zustand %s, Platzpunkt %s)" % (want, g.state, g._crash))
            break
        if name == "tick":
            if g.round is not None and g.round["t"] >= int(arg):
                zustand["i"] += 1
                continue
            if g.round is None:
                raise SystemExit("tick:%s - keine Runde" % arg)
            break
        if name == "close":
            crash = g._crash
            if crash is None or crash < L.CLOSE_MIN:
                raise SystemExit("close: Platzpunkt %s zu klein" % crash)
            if g.state == LC.FLY:
                m = mult(g)
                if m >= L.CLOSE_MIN and m * 100 >= L.CLOSE_PCT * crash:
                    taste = "space"
                    zustand["i"] += 1
                    break
            elif g.state != LC.COUNT:
                raise SystemExit("close: Runde vorbei (%s)" % g.state)
            break
        if name == "end":
            if g.round is None and g.state in (LC.END, LC.IDLE):
                zustand["i"] += 1
                continue
            break
        if name == "noauto":
            if g.auto is None:
                zustand["i"] += 1
                continue
            break
        if name == "broke":
            if g.broke:
                zustand["i"] += 1
                continue
            break
        raise SystemExit("unbekannter Planschritt: " + p)
    if zustand["i"] >= len(plan) and "fertig" not in zustand:
        # Plan fertig: dieses Bild ist das letzte, auf das es ankommt
        zustand["fertig"] = zustand["bild"]
    spur.append(taste)
    if taste:
        orig(game, taste, IE)


vergleich.ereignis = bot
d = vergleich.datenordner(sp, "bot_probe")
vergleich.python_lauf("lamacrash", "single", seed, frames, ["x"] * frames, d, os.path.join(sp, "bot.png"))
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
if "fertig" in zustand:
    # Bild der letzten Plan-Zeile = Bild bild+1 -> so viele Bilder fuer vergleich.py
    print("Bilder bis Planende: %d" % (zustand["fertig"] + 2))
for n, crash, bild in zustand["runden"]:
    print("  Runde %d ab Bild %d: Platzpunkt x%s" % (n, bild, L.fmt_mult(crash) if crash is not None else "?"))
print(" ".join(kurz))
