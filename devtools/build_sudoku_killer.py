# -*- coding: utf-8 -*-
"""
build_sudoku_killer.py
======================
Erzeugt die eingebauten Killer-Sudoku-Level (4 Stufen x 100 Level) und
schreibt sie nach

    games/levels/sudoku-killer.json      (Desktop)
    web/js/games/sudoku_killer.js        (Browser, gleiche Daten als Skript)

Warum vorab? Ein Killer-Sudoku hat statt vieler Vorgaben Käfige mit Summen.
Der Beweis, dass so ein Rätsel genau EINE Lösung hat, braucht eine
Suche mit Käfig-Kombinationen - in Python dauert das pro Level zu lange, um
es beim Spielstart zu machen. Hier läuft es einmal (parallel auf allen Kernen).

Ablauf je Level (deterministisch über seedrand):
1. volle Lösung erzeugen (``sudoku_gen.fill_grid``),
2. das Brett in Käfige zerlegen: zusammenhängende Gruppen ohne doppelte
   Ziffer, Größe je Stufe (Leicht klein ... Experte groß),
3. Vorgaben ergänzen, bis die Lösung eindeutig ist: findet der Löser eine
   zweite Lösung, wird eine Zelle aufgedeckt, in der sich beide unterscheiden,
4. überflüssige Vorgaben wieder entfernen (solange eindeutig),
5. Leicht/Normal bekommen zusätzliche Vorgaben bis zu einer Mindestzahl,
6. Schlussprüfung ohne Knotenlimit: genau eine Lösung.

Stufen: Käfiggrößen und Vorgaben
    Leicht   Käfige 2-3 Zellen, mind. 14 Vorgaben
    Normal   Käfige 2-4 Zellen, mind. 6 Vorgaben
    Schwer   Käfige 2-5 Zellen, nur nötige Vorgaben
    Experte  Käfige 3-6 Zellen, nur nötige Vorgaben (beste von 4 Zerlegungen)

Aufruf aus dem Repo-Root:
    python devtools/build_sudoku_killer.py            # alle 400 Level
    python devtools/build_sudoku_killer.py --check    # nur vorhandene prüfen
"""

import importlib.util
import json
import multiprocessing
import os
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, ROOT)

import seedrand                                  # noqa: E402


def _load_gen():
    """games/sudoku_gen.py direkt laden - ohne games/__init__.py (das zieht
    alle Spiele samt pygame nach sich)."""
    path = os.path.join(ROOT, "games", "sudoku_gen.py")
    spec = importlib.util.spec_from_file_location("sudoku_gen", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


gen = _load_gen()

OUT_JSON = os.path.join(ROOT, "games", "levels", "sudoku-killer.json")
OUT_JS = os.path.join(ROOT, "web", "js", "games", "sudoku_killer.js")

LEVELS = 100
# (kleinste, größte Käfiggröße, Mindest-Vorgaben, Zerlegungs-Versuche)
STAGES = ((2, 3, 14, 1), (2, 4, 6, 1), (2, 5, 0, 2), (3, 6, 0, 4))
BUDGET = 6000           # Knotenlimit je Prüfung (sonst "unbekannt" -> Vorgabe bleibt)


def _neighbors(i):
    r, c = divmod(i, 9)
    out = []
    if r > 0:
        out.append(i - 9)
    if r < 8:
        out.append(i + 9)
    if c > 0:
        out.append(i - 1)
    if c < 8:
        out.append(i + 1)
    return out


def make_cages(sol, rng, lo, hi):
    """Zerlegt das Brett in zusammenhängende Käfige ohne doppelte Ziffer."""
    cage_of = [-1] * 81
    cages = []
    order = list(range(81))
    rng.shuffle(order)
    for start in order:
        if cage_of[start] >= 0:
            continue
        target = rng.randint(lo, hi)
        cells = [start]
        digits = 1 << sol[start]
        cage_of[start] = len(cages)
        while len(cells) < target:
            opts = sorted({j for i in cells for j in _neighbors(i)
                           if cage_of[j] < 0 and not digits >> sol[j] & 1})
            if not opts:
                break
            j = rng.choice(opts)
            cells.append(j)
            digits |= 1 << sol[j]
            cage_of[j] = len(cages)
        cages.append(cells)
    # Einzelne Zellen (zu klein) an einen passenden Nachbarkäfig hängen.
    for ci, cells in enumerate(cages):
        if len(cells) != 1:
            continue
        i = cells[0]
        opts = sorted({cage_of[j] for j in _neighbors(i)
                       if cage_of[j] != ci
                       and all(sol[k] != sol[i] for k in cages[cage_of[j]])})
        if opts:
            cj = rng.choice(opts)
            cages[cj].append(i)
            cage_of[i] = cj
            cages[ci] = []
    return [(sum(sol[i] for i in c), tuple(sorted(c))) for c in cages if c]


def _unique(puz, cages, budget=BUDGET):
    return gen.count_killer(puz, cages, 2, budget=budget) == 1


def build_level(args):
    """Erzeugt ein Level -> (stufe, level, dict, statistik)."""
    diff, level = args
    lo, hi, min_givens, tries = STAGES[diff]
    lay = gen.layout_for("killer")
    best = None
    for attempt in range(tries):
        rng = seedrand.Rand(seedrand.seed_from("sudoku-killer", diff, level,
                                               attempt))
        sol = gen.fill_grid(lay, rng)
        cages = make_cages(sol, rng, lo, hi)
        puz = [0] * 81
        # 3. Vorgaben ergänzen, bis eindeutig
        while True:
            sols = []
            n = gen.count_killer(puz, cages, 2, sols, budget=BUDGET)
            if n == 1:
                break
            alt = next((s for s in sols if s != sol), None)
            if alt is not None:
                spots = [i for i in range(81) if alt[i] != sol[i] and not puz[i]]
            else:                       # Knotenlimit: irgendeine Zelle aufdecken
                spots = [i for i in range(81) if not puz[i]]
            i = rng.choice(spots)
            puz[i] = sol[i]
        # 4. überflüssige Vorgaben entfernen
        given = [i for i in range(81) if puz[i]]
        rng.shuffle(given)
        for i in given:
            puz[i] = 0
            if not _unique(puz, cages):
                puz[i] = sol[i]
        count = sum(1 for v in puz if v)
        if best is None or count < best[0]:
            best = (count, sol, puz, cages)
        if count == 0:
            break
    count, sol, puz, cages = best
    # 5. Mindest-Vorgaben für die leichten Stufen
    rng = seedrand.Rand(seedrand.seed_from("sudoku-killer-extra", diff, level))
    free = [i for i in range(81) if not puz[i]]
    rng.shuffle(free)
    while sum(1 for v in puz if v) < min_givens and free:
        i = free.pop()
        puz[i] = sol[i]
    # 6. Schlussprüfung ohne Knotenlimit
    assert gen.count_killer(puz, cages, 2) == 1, (diff, level)
    data = {"s": "".join(map(str, sol)), "g": "".join(map(str, puz)),
            "c": [[total, list(cells)] for total, cells in cages]}
    stats = (sum(1 for v in puz if v), len(cages),
             max(len(c) for _, c in cages))
    return diff, level, data, stats


def write_outputs(levels):
    doc = {"format": 1,
           "info": "Killer-Sudoku-Level, erzeugt von devtools/build_sudoku_killer.py",
           "levels": {str(d): levels[d] for d in range(4)}}
    with open(OUT_JSON, "w", encoding="utf-8", newline="\n") as f:
        json.dump(doc, f, ensure_ascii=False, separators=(",", ":"))
        f.write("\n")
    # Browser: gleiche Daten als Skript (file:// erlaubt kein fetch).
    lines = ["/*",
             " * sudoku_killer.js - eingebaute Killer-Sudoku-Level (4 Stufen x 100)",
             " * ====================================================================",
             " * ERZEUGT von devtools/build_sudoku_killer.py - nicht von Hand ändern.",
             " * Gleiche Daten wie games/levels/sudoku-killer.json:",
             " *   s = Lösung (81 Ziffern), g = Vorgaben (0 = leer),",
             " *   c = Käfige [[summe, [zellen...]], ...]",
             " */",
             "(function () {",
             '  "use strict";',
             "  PG.sudokuKiller = " + json.dumps(doc["levels"], separators=(",", ":")) + ";",
             "})();",
             ""]
    with open(OUT_JS, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines))


def check_existing():
    gen._killer = None
    levels = gen.load_killer()
    bad = 0
    t0 = time.time()
    for d in range(4):
        for n, (puz, sol, cages) in enumerate(levels[d], 1):
            if gen.count_killer(puz, cages, 2) != 1:
                print("NICHT EINDEUTIG: Stufe %d Level %d" % (d, n))
                bad += 1
        print("Stufe %d: %d Level geprüft" % (d, len(levels[d])))
    print("fertig in %.1f s, %d Fehler" % (time.time() - t0, bad))
    return 1 if bad else 0


def main():
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    except (AttributeError, OSError):
        pass
    if "--check" in sys.argv:
        return check_existing()
    jobs = [(d, lv) for d in range(4) for lv in range(1, LEVELS + 1)]
    levels = {d: [None] * LEVELS for d in range(4)}
    stats = {d: [] for d in range(4)}
    t0 = time.time()
    # Nicht alle Kerne belegen - der Rechner soll nebenbei benutzbar bleiben.
    procs = max(2, (os.cpu_count() or 4) // 3)
    with multiprocessing.Pool(procs) as pool:
        for k, (d, lv, data, st) in enumerate(
                pool.imap_unordered(build_level, jobs), 1):
            levels[d][lv - 1] = data
            stats[d].append(st)
            if k % 20 == 0:
                print("  %d/%d Level (%.0f s)" % (k, len(jobs), time.time() - t0),
                      flush=True)
    write_outputs(levels)
    for d in range(4):
        giv = [s[0] for s in stats[d]]
        cg = [s[1] for s in stats[d]]
        print("Stufe %d: Vorgaben %d-%d (Ø %.1f), Käfige %d-%d (Ø %.1f), "
              "größter Käfig %d" % (d, min(giv), max(giv), sum(giv) / len(giv),
                                    min(cg), max(cg), sum(cg) / len(cg),
                                    max(s[2] for s in stats[d])))
    print("geschrieben: %s, %s (%.0f s)" % (OUT_JSON, OUT_JS, time.time() - t0))
    return 0


if __name__ == "__main__":
    sys.exit(main())
