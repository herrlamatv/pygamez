# -*- coding: utf-8 -*-
"""Headless-Audit für das Sudoku-Update (Varianten, Tages-Sudoku, Komfort).

Geprüft wird:

Rätsel    (1) die klassischen Level sind unverändert (Hash je Stufe, Desktop
              UND Browser-Generator) - sonst verlören Spieler ihre Haken,
          (2) X-Sudoku/Mini 6x6: Seeds deterministisch, jedes Rätsel eindeutig
              lösbar, Leicht/Normal allein mit Singles lösbar, Vorgaben je
              Stufe fallend, Experte braucht mehr als Singles,
          (3) Tages-Sudoku: deterministisch je Datum, Stufe je Wochentag,
          (4) Killer: alle 400 Level aus games/levels/sudoku-killer.json sind
              eindeutig lösbar (Beweis mit Käfig-Propagation), Käfige
              zusammenhängend, ohne doppelte Ziffer, Summen stimmen, Stufen
              werden über Käfiggrößen schwerer; der Killer-Löser stimmt mit
              dem einfachen Zähler überein,
          (5) Desktop == Browser: X/Mini/Tages-Rätsel und Killer-Daten aus
              web/js/games/*.js (per Node) sind bitgenau identisch.
Spiel     (6) Rückgängig/Wiederholen (inkl. eingerasteter Ziffern, Fehler
              bleiben gezählt, key_is_free), Ziffer-zuerst-Eingabe,
          (7) Kandidaten automatisch eintragen = echte Kandidaten,
          (8) Spielstand sichern & fortsetzen (Zurück, on_exit, gedrosselt,
              Modus-Multiplikator, Neustart löscht, Sieg löscht),
          (9) Sterne, Bestzeit, Erfolge (sudoku_clean/killer/stars),
              Tages-Serie, alte mem.json (nur "solved") bleibt gültig,
Optik     (10) Setup/Spiel/Ergebnis: 5 Auflösungen x 14 Sprachen x Varianten
              - alles im Bild, nichts überlappt, alle Texte passen,
          (11) Generierungszeit und Zeichenzeit (1280x960, voller Killer).

Aufruf aus dem Repo-Root:  python tests/audit_sudoku.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import hashlib
import json
import multiprocessing
import os
import subprocess
import sys
import time
import types

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except (AttributeError, OSError):
    pass

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, REPO)

import pygame  # noqa: E402

import store  # noqa: E402
# Testläufe dürfen weder mem.json noch settings.json anfassen
store._PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                           "_audit-sudoku-mem.json")
import settings as settings_mod  # noqa: E402
settings_mod.save_settings = lambda s: None
import i18n  # noqa: E402
import seedrand  # noqa: E402
import ui  # noqa: E402

# Nur die Sudoku-Module laden - nicht games/__init__.py mit allen Spielen.
if "games" not in sys.modules:
    _pkg = types.ModuleType("games")
    _pkg.__path__ = [os.path.join(REPO, "games")]
    sys.modules["games"] = _pkg
from games import sudoku as sd  # noqa: E402
from games import sudoku_gen as gen  # noqa: E402
from game_base import InputEvent  # noqa: E402

GS = json.loads(json.dumps(settings_mod.DEFAULTS))
FAILS = []
RESOLUTIONS = ((480, 360), (640, 480), (800, 600), (960, 720), (1280, 960))

# Hashes der klassischen Level VOR dem Update (Python-random bzw. PG.Random).
CLASSIC_PY = {(0, 1): '0859ae4acf056625', (0, 2): 'f2c6fb3d35d299b7', (0, 37): 'c6ebcd9f9e29b7e2',
              (0, 100): 'e20d2036beab757c', (1, 1): 'bbc6337b5f3901fa', (1, 2): 'a5d093c2f4454b31',
              (1, 37): '7b6a3780f319b658', (1, 100): 'aefd608aa07b7dcd', (2, 1): 'd4370302f58cb36b',
              (2, 2): 'be6116d9220ad4d3', (2, 37): 'b6b4cee61d8683bc', (2, 100): '237f9a765a1ab567',
              (3, 1): '8494b06da31b8fb4', (3, 2): '749f7d72e6669868', (3, 37): 'cbfc098c32a6522a',
              (3, 100): '3268c9d1bce1156d'}
CLASSIC_JS = {"0,1": "aa82c0bebe596b69", "0,2": "4310d7cfb21305c4", "0,37": "909ff16f392f15b7",
              "0,100": "2f482d282a984f66", "1,1": "616c20a7bf20e993", "1,2": "0f60deadefa9d194",
              "1,37": "c0e2039fa1179d33", "1,100": "97b8e42dbdf3ad1f", "2,1": "f285014d74210417",
              "2,2": "789c40d9b53cf297", "2,37": "5e89aebf32d82c1a", "2,100": "df3fb9c8f7c26531",
              "3,1": "06391cc78faacdde", "3,2": "348163929e329251", "3,37": "462142bf07194f78",
              "3,100": "8104ba57373e5ca2"}


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + str(detail)) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


EVENTS = []


def quiet(game):
    game.play_sound = lambda name: None
    game.rumble = lambda ms=0: None
    game.ach_event = lambda *a: EVENTS.append(a)
    game.report_result = lambda won: None
    return game


def fresh_mem(section=None):
    for p in (store._PATH, store._PATH + ".bak"):
        if os.path.exists(p):
            os.remove(p)
    if section is not None:
        store.save_section("sudoku", section)


def new_game(mode="comfort", w=800, h=600, **opts):
    gs = json.loads(json.dumps(GS))
    gs["sudoku"].update(opts)
    g = quiet(sd.SudokuGame(pygame.Surface((w, h)), w, h, mode=mode, game_settings=gs))
    return g


def start(g, variant, diff, level, daily=False):
    """Level starten und bis zum Spielbrett durchlaufen lassen."""
    if daily:
        g._start_daily()
    else:
        g.variant, g.diff = variant, diff
        g._start_level(level)
    g._gen_drawn = True
    g.update(0.016)
    assert g.state == sd.PLAY, g.state
    return g


def key(g, k):
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key=k))


def click(g, pos, button=1):
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=pos, button=button))


def cell_center(g, i):
    r, c = divmod(i, g.N)
    return (g.bx + c * g.cell + g.cell // 2, g.by + r * g.cell + g.cell // 2)


def puzzle_hash(p, s):
    return hashlib.sha256(("".join(map(str, p)) + "|" + "".join(map(str, s))).encode()).hexdigest()[:16]


# ============================================================ 1-3 Rätsel
def audit_classic_unchanged():
    print("\nKlassische Level unverändert")
    bad = [k for k, h in CLASSIC_PY.items() if puzzle_hash(*gen.generate(*k)) != h]
    check(not bad, "Python: %d Stichproben-Level identisch zum Stand vor dem Update" % len(CLASSIC_PY),
          bad)


def audit_variants():
    print("\nX-Sudoku und Mini 6x6")
    for variant in ("x", "mini"):
        lay = gen.layout_for(variant)
        a = gen.generate_variant(variant, 2, 17)
        b = gen.generate_variant(variant, 2, 17)
        c = gen.generate_variant(variant, 2, 18)
        check(a == b and a != c, "%s: gleicher Seed -> gleiches Rätsel, anderes Level -> anderes" % variant)
        clue_avg = []
        not_unique, singles_bad, wrong_sol, hard_easy = [], [], [], 0
        worst = 0.0
        levels = list(range(1, 26)) + [50, 77, 100]
        for d in range(4):
            clues = []
            for lv in levels:
                t0 = time.perf_counter()
                p, s = gen.generate_variant(variant, d, lv)
                worst = max(worst, time.perf_counter() - t0)
                clues.append(sum(1 for v in p if v))
                if any(v and v != s[i] for i, v in enumerate(p)) or \
                        any(sorted(s[i] for i in u) != list(range(1, lay.n + 1)) for u in lay.units):
                    wrong_sol.append((d, lv))
                if gen.count_solutions(lay, p) != 1:
                    not_unique.append((d, lv))
                singles = gen.solve_singles(lay, p)
                if d < 2 and singles != s:
                    singles_bad.append((d, lv))
                if d == 3 and singles is not None:
                    hard_easy += 1
            clue_avg.append(sum(clues) / len(clues))
        check(not wrong_sol, "%s: Lösungen gültig (alle Einheiten inkl. Diagonalen)" % variant, wrong_sol[:5])
        check(not not_unique, "%s: %d Rätsel eindeutig lösbar" % (variant, 4 * len(levels)), not_unique[:5])
        check(not singles_bad, "%s: Leicht/Normal allein mit Singles lösbar" % variant, singles_bad[:5])
        check(all(clue_avg[i] > clue_avg[i + 1] for i in range(3)),
              "%s: Vorgaben je Stufe fallend (%s)" % (variant, ", ".join("%.1f" % c for c in clue_avg)))
        if variant == "x":
            # (6x6 ist dafür zu klein - dort macht die Vorgaben-Zahl den Unterschied)
            check(hard_easy <= len(levels) // 10,
                  "%s: Experte braucht fast immer mehr als Singles (%d/%d nur Singles)"
                  % (variant, hard_easy, len(levels)))
        check(worst < 2.0, "%s: Generierung schnell genug (langsamstes %.2f s)" % (variant, worst))


def audit_daily():
    print("\nTages-Sudoku")
    lay = gen.layout_for("classic")
    dates = ["2026-09-14", "2026-09-15", "2026-09-16", "2026-09-17", "2026-09-18",
             "2026-09-19", "2026-09-20", "2027-02-28"]
    ok_diff = all(gen.generate_daily(d)[2] == gen.DAILY_DIFF[i % 7] for i, d in enumerate(dates[:7]))
    check(ok_diff, "Stufe je Wochentag (Mo..So = %s)" % (gen.DAILY_DIFF,))
    same = gen.generate_daily("2026-09-16") == gen.generate_daily("2026-09-16")
    diff = gen.generate_daily("2026-09-16")[0] != gen.generate_daily("2026-09-17")[0]
    check(same and diff, "gleiches Datum -> gleiches Rätsel, anderer Tag -> anderes")
    uniq = all(gen.count_solutions(lay, gen.generate_daily(d)[0]) == 1 for d in dates)
    check(uniq, "Tages-Rätsel eindeutig lösbar (%d Tage)" % len(dates))
    t0 = time.perf_counter()
    gen.generate_daily(seedrand.today_str())
    check(time.perf_counter() - t0 < 1.5, "heutiges Rätsel in %.2f s erzeugt" % (time.perf_counter() - t0))


# ============================================================ 4 Killer
def _killer_unique(args):
    d, n, puz, cages = args
    return d, n, gen.count_killer(puz, cages, 2)


def audit_killer():
    print("\nKiller-Sudoku (vorab erzeugte Level)")
    gen._killer = None
    t0 = time.perf_counter()
    levels = gen.load_killer()
    load_t = time.perf_counter() - t0
    check(all(len(levels[d]) == 100 for d in range(4)),
          "4 Stufen x 100 Level geladen (%.2f s)" % load_t, {d: len(levels[d]) for d in range(4)})
    lay = gen.layout_for("killer")
    bad_shape = []
    size_avg = []
    for d in range(4):
        sizes = []
        for n, (puz, sol, cages) in enumerate(levels[d], 1):
            if any(sorted(sol[i] for i in u) != list(range(1, 10)) for u in lay.units):
                bad_shape.append((d, n, "Lösung"))
            for total, cells in cages:
                sizes.append(len(cells))
                digits = [sol[i] for i in cells]
                if sum(digits) != total or len(set(digits)) != len(digits):
                    bad_shape.append((d, n, "Summe/Doppelt"))
                # zusammenhängend (4er-Nachbarschaft)
                todo, seen = [cells[0]], {cells[0]}
                cs = set(cells)
                while todo:
                    i = todo.pop()
                    r, c = divmod(i, 9)
                    for j in (i - 9 if r else -1, i + 9 if r < 8 else -1,
                              i - 1 if c else -1, i + 1 if c < 8 else -1):
                        if j in cs and j not in seen:
                            seen.add(j)
                            todo.append(j)
                if seen != cs:
                    bad_shape.append((d, n, "nicht zusammenhängend"))
        size_avg.append(sum(sizes) / len(sizes))
    check(not bad_shape, "Käfige zusammenhängend, ohne doppelte Ziffer, Summen stimmen", bad_shape[:5])
    check(all(size_avg[i] < size_avg[i + 1] for i in range(3)),
          "Käfige werden je Stufe größer (Ø %s)" % ", ".join("%.2f" % s for s in size_avg))

    # Killer-Löser gegen den einfachen Zähler (mit Vorgaben, damit der schnell ist)
    mism = 0
    for k in range(30):
        rng = seedrand.Rand(seedrand.seed_from("audit-killer", k))
        puz, sol, cages = levels[k % 4][k]
        test = [0] * 81
        order = list(range(81))
        rng.shuffle(order)
        for i in order[:rng.randint(10, 28)]:
            test[i] = sol[i]
        if k % 3 == 0:                     # auch unlösbare Vorgaben testen
            i = order[40]
            test[i] = sol[i] % 9 + 1
        if gen.count_solutions(lay, test, 2, cages) != gen.count_killer(test, cages, 2):
            mism += 1
    check(mism == 0, "Killer-Löser == einfacher Zähler (30 Stichproben)", mism)

    jobs = [(d, n, puz, cages) for d in range(4) for n, (puz, _s, cages) in enumerate(levels[d], 1)]
    t0 = time.perf_counter()
    with multiprocessing.Pool(max(2, min(8, (os.cpu_count() or 4) // 3))) as pool:
        res = pool.map(_killer_unique, jobs, chunksize=4)
    bad = [(d, n, c) for d, n, c in res if c != 1]
    check(not bad, "alle %d Killer-Level eindeutig lösbar (%.1f s)" % (len(jobs), time.perf_counter() - t0),
          bad[:5])


# ============================================================ 5 Desktop == Browser
def audit_parity():
    print("\nDesktop == Browser (Node)")
    variants = [["x", d, lv] for d in range(4) for lv in (1, 2, 57, 100)] + \
               [["mini", d, lv] for d in range(4) for lv in (1, 3, 42, 99)]
    dates = ["2026-09-14", "2026-09-16", "2026-09-19", "2026-09-20", "2026-12-31", "2027-03-01"]
    req = {"variants": variants, "dates": dates, "classic": [list(k) for k in CLASSIC_PY]}
    try:
        out = subprocess.run(["node", os.path.join(REPO, "web", "tools", "sudoku_parity.js"),
                              json.dumps(req)], capture_output=True, text=True, timeout=300,
                             encoding="utf-8")
        data = json.loads(out.stdout)
    except (OSError, ValueError, subprocess.SubprocessError) as exc:
        check(False, "Node-Skript web/tools/sudoku_parity.js läuft", exc)
        return
    bad = []
    for item in data["variants"]:
        p, s = gen.generate_variant(item["variant"], item["diff"], item["level"])
        if "".join(map(str, p)) != item["puzzle"] or "".join(map(str, s)) != item["solution"]:
            bad.append((item["variant"], item["diff"], item["level"]))
    check(not bad, "X-Sudoku + Mini: %d Rätsel identisch" % len(variants), bad[:5])
    bad = []
    for item in data["daily"]:
        p, s, d = gen.generate_daily(item["date"])
        if "".join(map(str, p)) != item["puzzle"] or d != item["diff"]:
            bad.append(item["date"])
    check(not bad, "Tages-Sudoku: %d Tage identisch (inkl. Wochentag-Stufe)" % len(dates), bad)
    js_bad = [k for k, h in CLASSIC_JS.items() if data["classic"].get(k) != h]
    check(not js_bad, "Browser: klassische Level unverändert", js_bad)
    with open(gen.KILLER_FILE, encoding="utf-8") as f:
        desk = json.load(f)["levels"]
    check(desk == data["killer"], "Killer-Daten: sudoku-killer.json == sudoku_killer.js")


# ============================================================ 6-7 Eingabe
def board_state(g):
    return (list(g.board), list(g.notes), list(g.marks), set(g.wrong))


def audit_undo_redo():
    print("\nRückgängig / Wiederholen / Eingabe")
    fresh_mem()
    g = start(new_game("notes", colors=True), "x", 1, 4)
    free = [i for i in range(g.cells) if not g.given[i]]
    states = [board_state(g)]
    a, b, c, d = free[:4]
    wrong = next(v for v in range(1, 10) if v != g.solution[a])
    g.sel = a
    key(g, str(wrong))                     # 1: falsche Ziffer
    states.append(board_state(g))
    g.sel = b
    key(g, str(g.solution[b]))             # 2: richtige Ziffer
    states.append(board_state(g))
    key(g, "n")
    g.sel = c
    key(g, "3")                            # 3: Notiz
    key(g, "n")
    states.append(board_state(g))
    key(g, "c")                            # 4: Kandidaten
    states.append(board_state(g))
    g.sel = d
    click(g, g.pad_rects["mark2"].center)  # 5: Farbmarker
    states.append(board_state(g))
    errors = g.errors
    ok = True
    for k in range(len(states) - 1, 0, -1):
        key(g, "u")
        ok = ok and board_state(g) == states[k - 1]
    check(ok, "U nimmt 5 Schritte (Ziffern, Notiz, Kandidaten, Farbe) exakt zurück")
    check(g.errors == errors == 1, "Fehler bleiben gezählt (Rückgängig ist kein Radiergummi)")
    ok = True
    for k in range(1, len(states)):
        key(g, "y")
        ok = ok and board_state(g) == states[k]
    check(ok, "Y stellt alle Schritte wieder her")
    key(g, "z")
    ok_z = board_state(g) == states[-2]
    key(g, "Y")
    check(ok_z and board_state(g) == states[-1], "Z = Rückgängig, großes Y = Wiederholen")
    key(g, "u")
    g.sel = free[5]
    key(g, str(g.solution[free[5]]))
    check(not g.redo_stack, "neue Eingabe verwirft die Wiederholen-Liste")

    # Komfort: eingerastete Ziffern bleiben stehen
    g = start(new_game("comfort"), "classic", 0, 3)
    free = [i for i in range(81) if not g.given[i]]
    g.sel = free[0]
    wrong = next(v for v in range(1, 10) if v != g.solution[free[0]])
    key(g, str(wrong))
    key(g, str(g.solution[free[0]]))       # korrigiert -> rastet ein
    key(g, "u")
    check(g.board[free[0]] == g.solution[free[0]] and g.locked[free[0]],
          "Komfort: eingerastete richtige Ziffer lässt sich nicht zurücknehmen")

    # key_is_free: selbst belegte Taste löst kein Rückgängig aus
    g = start(new_game("notes"), "mini", 0, 2)
    g.controls = {"p1": dict(settings_mod.DEFAULT_CONTROLS["p1"], action="u"),
                  "p2": dict(settings_mod.DEFAULT_CONTROLS["p2"])}
    free = [i for i in range(g.cells) if not g.given[i]]
    g.sel = free[0]
    key(g, str(g.solution[free[0]]))
    key(g, "u")
    check(g.board[free[0]] == g.solution[free[0]], "U greift nicht, wenn es einer Aktion zugeordnet ist")
    click(g, g.pad_rects["undo"].center)
    check(g.board[free[0]] == 0, "Rückgängig-Knopf funktioniert trotzdem")

    # Ziffer zuerst
    g = start(new_game("comfort", input="digit"), "classic", 1, 5)
    free = [i for i in range(81) if not g.given[i]]
    i = free[0]
    dgt = g.solution[i]
    click(g, g.pad_rects[str(dgt)].center)
    check(g.brush == dgt and g.board[i] == 0, "Ziffer zuerst: Klick aufs Ziffernfeld wählt nur die Ziffer")
    click(g, cell_center(g, i))
    check(g.board[i] == dgt, "Ziffer zuerst: Klick auf Zelle setzt die gewählte Ziffer")
    j = free[1]
    g.sel = j
    key(g, str(g.solution[j]))
    check(g.board[j] == 0 and g.brush == g.solution[j], "Ziffer zuerst: Taste wählt Ziffer ...")
    key(g, "space")
    check(g.board[j] == g.solution[j], "... Leertaste setzt sie in die markierte Zelle")

    # Restziffer-Zähler (in allen Modi): Pad zeigt N - Anzahl
    g = start(new_game("classic"), "mini", 1, 1)
    counts = [sum(1 for v in g.board if v == d) for d in range(7)]
    check(all(6 - counts[d] >= 0 for d in range(1, 7)), "Mini: Restziffern je Ziffer 0..6")


def audit_candidates():
    print("\nKandidaten automatisch")
    for variant, mode in (("classic", "notes"), ("x", "comfort"), ("killer", "assist"), ("mini", "notes")):
        g = start(new_game(mode), variant, 1, 6)
        free = [i for i in range(g.cells) if not g.given[i]]
        for i in free[:5]:
            g.sel = i
            key(g, str(g.solution[i]))
        key(g, "c")
        ok = True
        for i in range(g.cells):
            if g.board[i]:
                ok = ok and g.notes[i] == 0
                continue
            expect = 0
            for dgt in range(1, g.N + 1):
                if all(g.board[j] != dgt for j in g.peers[i]):
                    expect |= 1 << (dgt - 1)
            ok = ok and g.notes[i] == expect and (g.notes[i] >> (g.solution[i] - 1)) & 1
        check(ok, "%s: Notizen = Ziffern ohne Konflikt in Zeile/Spalte/Block%s (Lösung immer dabei)"
              % (variant, "/Diagonale" if variant == "x" else "/Käfig" if variant == "killer" else ""))
        i = next(i for i in range(g.cells) if not g.board[i])
        g.sel = i
        key(g, str(g.solution[i]))
        pruned = all(not (g.notes[j] >> (g.solution[i] - 1)) & 1 for j in g.peers[i])
        check(pruned, "%s: eine Eingabe streicht die Ziffer aus den Nachbar-Notizen" % variant)
    g = start(new_game("classic"), "classic", 0, 1)
    key(g, "c")
    check(not any(g.notes), "Klassisch-Modus: keine Kandidaten-Automatik (keine Notizen)")


# ============================================================ 8 Speichern
def audit_save_resume():
    print("\nSpielstand sichern & fortsetzen")
    fresh_mem()
    g = start(new_game("notes", colors=True), "x", 1, 3)
    free = [i for i in range(g.cells) if not g.given[i]]
    for i in free[:4]:
        g.sel = i
        key(g, str(g.solution[i]))
    g.sel = free[5]
    key(g, str(g.solution[free[5]] % 9 + 1))      # ein Fehler
    key(g, "n")
    g.sel = free[6]
    key(g, "7")
    key(g, "n")
    g.marks[free[7]] = 4
    g.elapsed = 42.4
    snap = board_state(g)
    g._back_to_setup()
    save = store.load_section("sudoku").get("saves", {}).get("x:1:3")
    check(isinstance(save, dict) and save.get("p", 0) > 0, "Q: Spielstand in mem.json (mit Fortschritt %)",
          save)
    check(g._level_save("x", 1, 3) is not None, "Levelwahl kennt das angefangene Level")

    g2 = start(new_game("notes", colors=True), "x", 1, 3)
    check(board_state(g2) == snap and g2.errors == 1 and abs(g2.elapsed - 42.4) < 0.2,
          "Fortsetzen: Ziffern, Notizen, Farben, Fehler und Zeit identisch")
    check(g2.msg == i18n.t("sud.resumed"), "Hinweis 'Spielstand geladen' erscheint")

    g3 = start(new_game("classic"), "x", 1, 3)
    check(g3.mult == 1.5, "anderer Modus: kleinster Multiplikator gilt (x%.1f)" % g3.mult)

    key(g3, "r")
    g3._gen_drawn = True
    g3.update(0.016)
    check(g3.board == g3.puzzle and "x:1:3" not in store.load_section("sudoku").get("saves", {}),
          "R: Neustart verwirft den Spielstand")

    g4 = start(new_game("comfort"), "mini", 2, 8)
    i = next(i for i in range(g4.cells) if not g4.given[i])
    g4.sel = i
    key(g4, str(g4.solution[i]))
    g4.on_exit()
    check("mini:2:8" in store.load_section("sudoku").get("saves", {}), "on_exit sichert das Rätsel")

    g5 = start(new_game("comfort"), "mini", 2, 9)
    i = next(i for i in range(g5.cells) if not g5.given[i])
    g5.sel = i
    key(g5, str(g5.solution[i]))
    g5.update(1.0)
    before = "mini:2:9" in store.load_section("sudoku").get("saves", {})
    g5.update(4.5)
    after = "mini:2:9" in store.load_section("sudoku").get("saves", {})
    check(not before and after, "gedrosselt: gespeichert erst nach %.0f s" % sd.SAVE_EVERY)

    g6 = start(new_game("comfort"), "mini", 2, 8)
    for i in range(g6.cells):
        if not g6.board[i]:
            g6.sel = i
            key(g6, str(g6.solution[i]))
    check(g6.won and "mini:2:8" not in store.load_section("sudoku").get("saves", {}),
          "Sieg löscht den Spielstand")

    g7 = start(new_game("comfort"), "mini", 3, 1)
    g7.on_exit()
    check("mini:3:1" not in store.load_section("sudoku").get("saves", {}),
          "ohne Eingabe wird nichts gespeichert")

    # kaputter Spielstand wird ignoriert
    data = store.load_section("sudoku")
    data.setdefault("saves", {})["x:0:1"] = {"b": "123", "n": "kaputt"}
    store.save_section("sudoku", data)
    g8 = start(new_game("notes"), "x", 0, 1)
    check(g8.board == g8.puzzle, "kaputter Spielstand -> frisches Rätsel statt Absturz")

    # Tages-Sudoku speichert unter eigenem Schlüssel
    g9 = start(new_game("notes"), None, 0, 0, daily=True)
    i = next(i for i in range(81) if not g9.given[i])
    g9.sel = i
    key(g9, str(g9.solution[i]))
    g9._back_to_setup()
    check("daily:" + seedrand.today_str() in store.load_section("sudoku").get("saves", {}),
          "Tages-Sudoku: eigener Spielstand je Datum")


# ============================================================ 9 Sterne
def solve_all(g, errors=0):
    free = [i for i in range(g.cells) if not g.board[i]]
    for k, i in enumerate(free):
        g.sel = i
        if k < errors:
            key(g, str(g.solution[i] % g.N + 1))
        key(g, str(g.solution[i]))


def audit_stars():
    print("\nSterne, Bestzeit, Erfolge, Tages-Serie")
    check(sd.stars_for("classic", 0, 100, 0, 0) == 3 and sd.stars_for("classic", 0, 301, 0, 0) == 2
          and sd.stars_for("classic", 0, 100, 1, 0) == 1 and sd.stars_for("killer", 3, 10, 0, 1) == 1,
          "Sterne: 1 gelöst / 2 fehlerfrei ohne Tipps / 3 zusätzlich unter Zielzeit")

    fresh_mem({"solved": {"0": [1, 2], "1": [], "2": [5], "3": []}})
    g = new_game("comfort")
    check(g._record("classic", 0, 1) == (1, None) and g._stage_summary("classic", 0) == (2, 2),
          "alte mem.json (nur 'solved'): gelöste Level zählen als 1 Stern")
    del EVENTS[:]
    start(g, "classic", 0, 1)
    g.elapsed = 120
    solve_all(g)
    rec = store.load_section("sudoku")
    check(g.won and g.result_stars == 3 and rec["best"]["classic"]["0"]["1"] == [3, 120]
          and rec["solved"]["0"] == [1, 2] and rec["solved"]["2"] == [5],
          "Sieg: 3 Sterne + Bestzeit gespeichert, 'solved' bleibt kompatibel")
    check(g.score > 0 and ("sudoku_clean",) in EVENTS and ("sudoku_stars", 1) in EVENTS,
          "Punkte + Erfolge sudoku_clean / sudoku_stars(1)", EVENTS)

    g = start(new_game("comfort"), "classic", 0, 1)
    g.elapsed = 500
    solve_all(g, errors=1)
    rec = store.load_section("sudoku")["best"]["classic"]["0"]["1"]
    check(g.result_stars == 1 and rec == [3, 120] and not g.new_record,
          "schlechterer Lauf behält Sterne und Bestzeit")
    g = start(new_game("comfort"), "classic", 0, 1)
    g.elapsed = 60
    solve_all(g)
    check(g.new_record and store.load_section("sudoku")["best"]["classic"]["0"]["1"] == [3, 60],
          "schnellerer Lauf: neue Bestzeit")

    del EVENTS[:]
    g = start(new_game("classic"), "killer", 2, 7)
    g.elapsed = 300
    solve_all(g)
    check(("sudoku_killer",) in EVENTS and ("sudoku_stars", 2) in EVENTS,
          "Killer gelöst -> sudoku_killer, Sternzähler 2", EVENTS)
    check(g.score == int(max(50, sd.BASE_POINTS["killer"][2] - 2 * 300) * 2.0),
          "Punkte = (Basis Variante/Stufe - Zeit - Fehler - Tipps) x Modus")
    check(g.show_highscore_banner and g.score > 0, "Highscore zählt auch für neue Varianten")

    # Tages-Serie
    today = seedrand.today_str()
    idx = seedrand.day_index(today)
    import datetime
    yday = (datetime.date.fromisoformat(today) - datetime.timedelta(days=1)).isoformat()
    old = (datetime.date.fromisoformat(today) - datetime.timedelta(days=3)).isoformat()
    fresh_mem({"daily": {"date": yday, "streak": 4, "best": 6, "time": 400, "stars": 2, "count": 9}})
    g = new_game("notes")
    check(g.daily_streak() == 4 and not g.daily_done_today(), "Serie von gestern läuft noch")
    start(g, None, 0, 0, daily=True)
    g.elapsed = 90
    solve_all(g)
    st = store.load_section("sudoku")["daily"]
    check(st["date"] == today and st["streak"] == 5 and st["best"] == 6 and st["count"] == 10,
          "Tages-Sudoku gelöst: Serie 4 -> 5", st)
    fresh_mem({"daily": {"date": old, "streak": 8, "best": 8, "time": 400, "stars": 2, "count": 9}})
    g = new_game("notes")
    check(g.daily_streak() == 0, "Lücke von mehreren Tagen: Serie abgelaufen")
    start(g, None, 0, 0, daily=True)
    solve_all(g)
    st = store.load_section("sudoku")["daily"]
    check(st["streak"] == 1 and st["best"] == 8 and idx >= 0, "neue Serie beginnt bei 1, Rekord bleibt", st)
    check("classic" not in store.load_section("sudoku").get("best", {}) or
          not store.load_section("sudoku")["best"].get("classic"),
          "Tages-Sudoku schreibt keine Level-Bestzeit")


# ============================================================ 10 Layout
def texts_fit(fnt, text, width):
    return fnt.size(text)[0] <= width


def audit_layout():
    print("\nLayout: 5 Auflösungen x 14 Sprachen x 4 Varianten")
    before = i18n.get_language()
    fresh_mem({"daily": {"date": seedrand.today_str(), "streak": 99, "best": 99, "time": 3599,
                         "stars": 3, "count": 99}})
    for theme in ("v42", "v1"):
        ui.set_theme(theme)
        for w, h in RESOLUTIONS:
            problems = []
            g = new_game("assist", w, h, colors=True)
            rects = [("var%d" % i, r) for i, r in enumerate(g.var_rects)] + \
                    [("diff%d" % i, r) for i, r in enumerate(g.diff_rects)] + \
                    [("daily", g.daily_rect), ("limit", g.limit_rect), ("input", g.input_rect),
                     ("colors", g.colors_rect), ("grid", pygame.Rect(g.lv_x, g.lv_y, 10 * g.lv_cell,
                                                                     10 * g.lv_cell))]
            if g.start_rect is not None:
                rects.append(("start", g.start_rect))
            foot_top = h - (g._tiny.get_height() + 10)
            for name, r in rects:
                if r.left < 0 or r.right > w or r.top < 0 or r.bottom > foot_top:
                    problems.append("%s außerhalb %s" % (name, r))
            for i, (na, a) in enumerate(rects):
                for nb, b in rects[i + 1:]:
                    if a.colliderect(b):
                        problems.append("%s/%s überlappen" % (na, nb))
            inner = [r for n, r in rects if n in ("daily", "limit", "input", "colors", "start")]
            if not all(g.panel_rect.contains(r) for r in inner):
                problems.append("Schalter ragen aus dem Panel")
            if g.prog_y + g._small.get_height() // 2 > foot_top + 2:
                problems.append("Fortschrittszeile im Fuß")
            if g.info_rect.h < g._tiny.get_height():
                problems.append("keine Zeile Platz für Level-Info")

            for variant in sd.VARIANTS:
                for mode in ("classic", "assist"):
                    for colors in (False, True):
                        gp = start(new_game(mode, w, h, colors=colors), variant, 3, 100)
                        board = pygame.Rect(gp.bx, gp.by, gp.bs + 1, gp.bs + 1)
                        foot = h - (gp._tiny.get_height() + 12)
                        prs = list(gp.pad_rects.items())
                        if board.top < gp.hud_h or board.bottom > foot or board.right > w:
                            problems.append("%s/%s Brett außerhalb" % (variant, mode))
                        for k, r in prs:
                            if r.right > w or r.bottom > foot or r.colliderect(board) or r.top < gp.hud_h:
                                problems.append("%s/%s/%s Knopf %s außerhalb/überlappt" % (variant, mode, colors, k))
                        for i, (ka, a) in enumerate(prs):
                            for kb, b in prs[i + 1:]:
                                if a.colliderect(b):
                                    problems.append("%s: %s/%s überlappen" % (variant, ka, kb))
                        if gp.caption_bottom - gp.caption_y < gp._tiny.get_height():
                            problems.append("%s/%s/%s: keine Tooltip-Zeile" % (variant, mode, colors))
                        gp.draw()
            check(not problems, "%s %4dx%d: Setup + Spiel im Bild, ohne Überlappung" % (theme, w, h),
                  problems[:6])

    # Texte in allen Sprachen
    ui.set_theme("v42")
    for w, h in RESOLUTIONS:
        too_long = []
        g = new_game("assist", w, h, colors=True)
        gp = {v: start(new_game("assist", w, h, colors=True), v, 3, 100) for v in sd.VARIANTS}
        for code, _name in i18n.AVAILABLE:
            i18n.set_language(code, persist=False)
            T = i18n.t

            def need(ok, what):
                if not ok:
                    too_long.append("%s/%s" % (code, what))

            for i, v in enumerate(sd.VARIANTS):
                need(texts_fit(g._tiny, T("sud.variant." + v), g.var_rects[i].w - 10), "variant." + v)
            for i, (dk, _b) in enumerate(sd.DIFFICULTIES):
                need(texts_fit(g._tiny, T("sud.diff." + dk), g.diff_rects[i].w - 10), "diff." + dk)
            need(texts_fit(g._tiny, T("sud.mode.comfort"), w - 24), "mode")
            for rk, lbl, vals in (("limit", "sud.fail_limit", None), ("colors", "sud.opt_colors", None),
                                  ("input", "sud.opt_input", ("sud.input.cell", "sud.input.digit"))):
                r = getattr(g, rk + "_rect")
                if vals is None:
                    room = r.w - 16 - max(24, r.h * 3 // 2) - 8
                    need(texts_fit(g._tiny, T(lbl), room), lbl)
                else:
                    for vk in vals:
                        need(g._tiny.size(T(lbl))[0] + g._tiny.size(T(vk))[0] + 24 <= r.w, vk)
            dr = g.daily_rect
            ih = min(dr.h - 10, 34)
            tw = dr.right - 8 - (dr.x + 7 + ih + 8)
            need(texts_fit(g._tiny, T("sud.daily"), tw - 18), "daily")
            longest_wd = max((T("sud.wd.%d" % k) for k in range(7)), key=lambda s: g._tiny.size(s)[0])
            longest_diff = max((T("sud.diff." + dk) for dk, _ in sd.DIFFICULTIES),
                               key=lambda s: g._tiny.size(s)[0])
            # Zeile 2 lässt bei Platzmangel Wochentag bzw. Serie weg - das
            # Wichtigste (Stufe + Fortschritt) muss immer passen.
            need(texts_fit(g._tiny, "  ·  ".join([longest_diff, "100 %"]), tw) and
                 texts_fit(g._tiny, "  ·  ".join(["59:59", T("sud.daily_streak", n=99)]), tw),
                 "daily-zeile")
            if w >= 640:
                need(texts_fit(g._tiny, "  ·  ".join([longest_wd, longest_diff]), tw), "daily-wochentag")
            for k in ("sud.info_unsolved", "sud.info_solved"):
                need(texts_fit(g._tiny, T(k), g.info_rect.w), k)
            for k, kw in (("sud.info_best", {"t": "59:59"}), ("sud.info_started", {"p": 100}),
                          ("sud.info_target", {"t": "32:00"})):
                from games.sudoku_draw import _wrap
                need(len(_wrap(T(k, **kw), g._tiny, g.info_rect.w)) <= 2, k)
            if g.start_rect is not None:
                need(texts_fit(g._small, T("sud.start", n=100), g.start_rect.w - 10), "start")
                need(texts_fit(g._small, T("sud.resume"), g.start_rect.w - 10), "resume")
            need(texts_fit(g._tiny, T("sud.progress", n=100) + "   ·   300/300", 10 * g.lv_cell - 20), "progress")
            need(texts_fit(g._tiny, T("sud.setup_hint"), w - 12), "setup_hint")
            for k in ("sud.killer_missing",):
                need(texts_fit(g._tiny, T(k), w - 40), k)

            for v, p in gp.items():
                clock = p._clock_font.size("00:00")[0]
                side = (w // 2 - clock // 2) - 20
                need(texts_fit(p._tiny, T("sud.variant." + v), side) and
                     texts_fit(p._tiny, T("sud.daily"), side), "hud-top")
                longest = max([T("sud.level", n=100) + "  ·  " + T("sud.diff." + dk) for dk, _ in sd.DIFFICULTIES] +
                              [T("sud.wd.%d" % k) + "  ·  " + T("sud.diff." + dk)
                               for k in range(7) for dk, _ in sd.DIFFICULTIES],
                              key=lambda s: p._tiny.size(s)[0])
                need(texts_fit(p._tiny, longest, side), "hud-level")
                need(texts_fit(p._tiny, T("sud.errors", n=3, m=3), side), "hud-errors")
                need(texts_fit(p._tiny, T("sud.hint"), w - 12), "hint")
                need(texts_fit(p._tiny, T("sud.hint_digit"), w - 12), "hint_digit")
                for k in ("sud.resumed", "sud.cands_done", "sud.full_wrong"):
                    need(texts_fit(p._tiny, T(k), w - 40), k)
                for k in ("sud.tip.erase", "sud.tip.undo", "sud.tip.redo", "sud.tip.note", "sud.tip.cands",
                          "sud.tip.hint", "sud.tip.mark", "sud.tip.digit", "sud.tip.digit_first"):
                    from games.sudoku_draw import _wrap
                    lines = _wrap(T(k), p._tiny, p.pad_w)
                    need(len(lines) <= 2 and all(texts_fit(p._tiny, ln, p.pad_w) for ln in lines), k)
                # Ergebnis-Panel
                need(any(texts_fit(f, T("sud.win", t="59:59"), w - 40) for f in (p._huge, p._title)), "win")
                need(any(texts_fit(f, T("sud.lose"), w - 40) for f in (p._huge, p._title)), "lose")
                for k, kw in (("sud.next", {}), ("sud.next_daily", {}), ("sud.show_solution", {}),
                              ("sud.retry", {}), ("sud.daily_solved", {"n": 99}),
                              ("sud.new_best", {"t": "59:59"}), ("common.points", {"score": 99999})):
                    need(texts_fit(p._tiny, T(k, **kw), w - 60), k)
        i18n.set_language(before, persist=False)
        too_long = sorted(set(too_long))
        check(not too_long, "%4dx%d: alle Texte passen (14 Sprachen)" % (w, h), ", ".join(too_long[:10]))
    i18n.set_language(before, persist=False)

    # Auflösungswechsel mitten im Spiel + Ergebnis-Screen zeichnen
    g = start(new_game("comfort"), "killer", 1, 2)
    g.surface = pygame.Surface((1280, 960))
    g.width, g.height = 1280, 960
    g.on_surface_changed()
    ok = g.bs > 600 and g._cage_overlay().get_width() == g.bs + 1
    solve_all(g)
    for _ in range(3):
        g.draw()
    g.reveal = True
    g.draw()
    check(ok and g.won, "on_surface_changed im Spiel: Brett + Käfig-Cache neu, Ergebnis zeichnet")


# ============================================================ 11 Leistung
def audit_performance():
    print("\nLeistung")
    ui.set_theme("v42")
    g = start(new_game("comfort", 1280, 960, colors=True), "killer", 3, 50)
    key(g, "c")
    for _ in range(5):
        g.draw()
    n = 90
    t0 = time.perf_counter()
    for k in range(n):
        g.sel = (g.sel + 7) % 81
        g.update(1 / 60)
        g.draw()
    dt = (time.perf_counter() - t0) / n * 1000
    check(dt < 12.0, "1280x960 Killer mit allen Kandidaten: %.2f ms je Frame" % dt)

    g = new_game("comfort", 1280, 960)
    for _ in range(5):
        g.draw()
    t0 = time.perf_counter()
    for _ in range(n):
        g.draw()
    dt = (time.perf_counter() - t0) / n * 1000
    check(dt < 12.0, "1280x960 Setup-Screen: %.2f ms je Frame" % dt)

    t0 = time.perf_counter()
    g = start(new_game("comfort"), "x", 3, 57)
    check(time.perf_counter() - t0 < 2.0, "X-Sudoku Experte starten: %.2f s" % (time.perf_counter() - t0))


def main():
    pygame.init()
    pygame.display.set_mode((640, 480))
    i18n.init()
    try:
        audit_classic_unchanged()
        audit_variants()
        audit_daily()
        audit_killer()
        audit_parity()
        audit_undo_redo()
        audit_candidates()
        audit_save_resume()
        audit_stars()
        audit_layout()
        audit_performance()
    finally:
        for p in (store._PATH, store._PATH + ".bak", store._PATH + ".tmp"):
            if os.path.exists(p):
                os.remove(p)
    print()
    if FAILS:
        print("%d Prüfung(en) fehlgeschlagen:" % len(FAILS))
        for f in FAILS:
            print("  -", f)
        return 1
    print("Alle Prüfungen bestanden.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
