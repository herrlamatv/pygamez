# -*- coding: utf-8 -*-
"""Headless-Audit für 2048 (games/game2048.py + web/js/games/game2048.js).

Geprüft wird:

Logik      (1) slide_line: Verschmelzregeln inkl. Kettenfällen ([2,2,4,4] ->
               [4,8], [2,2,2,2] -> [4,4], [4,4,8] -> [8,8]),
           (2) plan_move gegen eine unabhängige Referenz für alle Größen 3-8
               und alle vier Richtungen - samt Konsistenz der Gleit-Wege
               (jede Kachel genau einmal, Summen am Ziel, Punkte = Merges),
           (3) spawn_tile: 90/10-Verteilung der 2er/4er, gleichmäßige Felder,
               nie auf belegte Felder, None bei vollem Brett,
           (4) die Web-Fassung liefert für dieselben Züge exakt dasselbe (Node).
Spiel      (5) Tastatur (repeat-Events ignoriert), Eingabe-Puffer während der
               Animation, Maus-/Touchpad-Wischen,
           (6) Rückgängig stellt Brett + Punkte + Züge wieder her, "3 pro
               Partie" ist nach drei Mal leer, "aus" sammelt nichts,
           (7) Undo -> unterstützte Partie: score 0, kein Highscore-Banner;
               Highscore NUR bei 4x4 Klassisch ohne Undo,
           (8) Klassisch: 2048 -> "Weiterspielen?" + Sieg + Erfolg; Endlos ohne
               Unterbrechung; tile_4096; keine Erfolge mit Undo/über 4x4,
           (9) Game Over (Niederlage gemeldet, Spielstand gelöscht) und Undo
               danach, Zeitangriff endet nach 3 Minuten (Uhr ab erstem Zug),
          (10) Speichern/Fortsetzen-Rundlauf (auch Undo-Stapel und Restzeit),
               gedrosselter Autosave, kaputte Spielstände werden verworfen,
               "Neue Partie" sichert den Highscore einer zählenden Partie,
          (11) Setup per Tastatur/Maus, Auflösungswechsel mitten im Spiel.
Layout    (12) 5 Auflösungen x 14 Sprachen x Größen 3-8: Setup, Spiel, Sieg-
               und Endstand-Panel im Bild, ohne Überlappung, Texte passen
               (keine unlesbar kleine Notschrift), UI v4.2 und UI v1.
Tempo     (13) 8x8 bei 1280x960 mit laufender Animation bleibt deutlich unter
               dem 60-FPS-Budget.

Aufruf aus dem Repo-Root:  python tests/audit_2048.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import json
import os
import random
import shutil
import subprocess
import sys
import tempfile
import time

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except (AttributeError, OSError):
    pass

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, REPO)

import pygame
pygame.init()
pygame.display.set_mode((640, 480))

import store
# Testläufe dürfen weder mem.json noch settings.json anfassen
store._PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                           "_audit-2048-mem.json")
import settings as settings_mod
SAVED_SETTINGS = []
settings_mod.save_settings = lambda s: SAVED_SETTINGS.append(json.loads(json.dumps(s)))
import i18n
i18n.init()
import ui
from game_base import InputEvent

try:
    from games import game2048 as g48
except Exception:                       # Paket gerade im Umbau -> Datei direkt laden
    import importlib.util
    _spec = importlib.util.spec_from_file_location(
        "game2048", os.path.join(REPO, "games", "game2048.py"))
    g48 = importlib.util.module_from_spec(_spec)
    _spec.loader.exec_module(g48)

FAILS = []
RESOLUTIONS = ((480, 360), (640, 480), (800, 600), (960, 720), (1280, 960))


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + detail) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


def quiet(game):
    game.events = []
    game.play_sound = lambda name: None
    game.rumble = lambda ms=0: None
    game.ach_event = lambda i, v=None: game.events.append(("ach", i))
    game.report_result = lambda won: game.events.append(("result", won))
    game._tone = lambda v: None
    return game


def wipe_store():
    for p in (store._PATH, store._PATH + ".bak"):
        try:
            os.remove(p)
        except OSError:
            pass


def make(w=640, h=480, size=4, mode="classic", undo="limited"):
    gs = json.loads(json.dumps(settings_mod.DEFAULTS))
    gs["g2048"] = {"size": size, "mode": mode, "undo": undo}
    return quiet(g48.Game2048(pygame.Surface((w, h)), w, h, game_settings=gs))


def settle(g, sec=0.6):
    for _ in range(int(sec * 60)):
        if g.game_over:
            break
        g.update(1 / 60)


def key(g, k, repeat=False):
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key=k, repeat=repeat))


def set_grid(g, grid):
    g.grid = [row[:] for row in grid]
    g.max_tile = g48.max_tile(g.grid)
    g.peak = g.max_tile
    g.stack = []
    g.queue = []
    g.slide = None
    g.over_t = None


def started(**kw):
    g = make(**kw)
    g._start(resume=False)
    settle(g, 0.3)
    return g


# ============================================================ (1)-(3) Logik

def ref_line(vals):
    xs = [v for v in vals if v]
    out, pts, skip = [], 0, False
    for i in range(len(xs)):
        if skip:
            skip = False
            continue
        if i + 1 < len(xs) and xs[i] == xs[i + 1]:
            out.append(xs[i] * 2)
            pts += xs[i] * 2
            skip = True
        else:
            out.append(xs[i])
    return out + [0] * (len(vals) - len(out)), pts


def ref_move(grid, d):
    n = len(grid)
    new = [[0] * n for _ in range(n)]
    pts = 0
    for k in range(n):
        if d in "LR":
            line = grid[k][:] if d == "L" else grid[k][::-1]
            o, p = ref_line(line)
            new[k] = o if d == "L" else o[::-1]
        else:
            col = [grid[r][k] for r in range(n)]
            if d == "D":
                col = col[::-1]
            o, p = ref_line(col)
            if d == "D":
                o = o[::-1]
            for r in range(n):
                new[r][k] = o[r]
        pts += p
    return new, pts


def audit_logic():
    print("\n2048 - Schiebe- und Verschmelzregeln")
    cases = [
        ([2, 2, 4, 4], [4, 8, 0, 0], 12),
        ([2, 2, 2, 2], [4, 4, 0, 0], 8),
        ([4, 4, 8, 0], [8, 8, 0, 0], 8),
        ([2, 0, 2, 4], [4, 4, 0, 0], 4),
        ([0, 0, 0, 2], [2, 0, 0, 0], 0),
        ([2, 4, 8, 16], [2, 4, 8, 16], 0),
        ([2, 2, 2], [4, 2, 0], 4),
        ([8, 8, 8, 8, 8], [16, 16, 8, 0, 0], 32),
        ([2, 2, 4, 4, 8, 8, 16, 16], [4, 8, 16, 32, 0, 0, 0, 0], 60),
        ([2, 2, 2, 2, 2, 2, 2, 2], [4, 4, 4, 4, 0, 0, 0, 0], 16),
        ([4, 0, 4, 0, 8, 8, 16, 0], [8, 16, 16, 0, 0, 0, 0, 0], 24),
    ]
    bad = []
    for vals, want, pts in cases:
        out, p, _w = g48.slide_line(vals)
        if out != want or p != pts:
            bad.append("%s -> %s/%d" % (vals, out, p))
    check(not bad, "slide_line: %d Kettenfälle" % len(cases), "; ".join(bad))

    # [2,2,4,4] in alle vier Richtungen auf einem 4x4-Brett
    g = [[2, 2, 4, 4], [0] * 4, [0] * 4, [0] * 4]
    ok = g48.plan_move(g, "L")["grid"][0] == [4, 8, 0, 0] \
        and g48.plan_move(g, "R")["grid"][0] == [0, 0, 4, 8]
    col = [[2, 0, 0, 0], [2, 0, 0, 0], [4, 0, 0, 0], [4, 0, 0, 0]]
    up = [row[0] for row in g48.plan_move(col, "U")["grid"]]
    down = [row[0] for row in g48.plan_move(col, "D")["grid"]]
    check(ok and up == [4, 8, 0, 0] and down == [0, 0, 4, 8],
          "[2,2,4,4] -> [4,8] in allen vier Richtungen", "U=%s D=%s" % (up, down))

    rng = random.Random(2048)
    pool = [0, 0, 0, 2, 2, 4, 4, 8, 16, 32]
    bad = []
    total = 0
    for n in g48.SIZES:
        for _ in range(250):
            grid = [[rng.choice(pool) for _c in range(n)] for _r in range(n)]
            for d in "LRUD":
                total += 1
                plan = g48.plan_move(grid, d)
                want, pts = ref_move(grid, d)
                if plan["grid"] != want or plan["gained"] != pts:
                    bad.append("n=%d %s Brett/Punkte" % (n, d))
                    continue
                if plan["moved"] != (want != grid):
                    bad.append("n=%d %s moved" % (n, d))
                # Gleit-Wege: jede Kachel genau einmal, Summen am Ziel stimmen
                src = sorted((fr, fc) for fr, fc, _tr, _tc, _v in plan["slides"])
                occupied = sorted((r, c) for r in range(n) for c in range(n) if grid[r][c])
                dest = {}
                for fr, fc, tr, tc, v in plan["slides"]:
                    if grid[fr][fc] != v or (fr != tr and fc != tc):
                        bad.append("n=%d %s Weg" % (n, d))
                        break
                    dest[(tr, tc)] = dest.get((tr, tc), 0) + v
                built = [[dest.get((r, c), 0) for c in range(n)] for r in range(n)]
                merge_sum = sum(v for _r, _c, v in plan["merges"])
                if src != occupied or built != want or merge_sum != pts:
                    bad.append("n=%d %s Konsistenz" % (n, d))
                if grid != [row[:] for row in grid]:
                    bad.append("Eingabe verändert")
    check(not bad, "plan_move = Referenz, Größen 3-8 x 4 Richtungen (%d Züge)" % total,
          "; ".join(sorted(set(bad))[:6]))

    full = [[2, 4], [8, 16]]
    stuck = [[2, 4, 2], [4, 2, 4], [2, 4, 2]]
    check(g48.has_moves([[2, 2], [4, 8]]) and not g48.has_moves(full)
          and not g48.has_moves(stuck) and g48.has_moves([[2, 0], [4, 8]]),
          "has_moves erkennt freie Felder und gleiche Nachbarn")

    print("\n2048 - Erscheinen neuer Kacheln")
    rng = random.Random(7)
    fours = 0
    cells = {}
    N = 40000
    for _ in range(N):
        grid = g48.empty_grid(4)
        r, c, v = g48.spawn_tile(grid, rng)
        fours += v == 4
        cells[(r, c)] = cells.get((r, c), 0) + 1
    share = fours / N
    spread = max(cells.values()) / min(cells.values())
    check(0.09 <= share <= 0.11, "Anteil 4er = %.3f (Soll 0,10)" % share)
    check(len(cells) == 16 and spread < 1.15, "alle 16 Felder gleich oft (max/min %.3f)" % spread)
    grid = [[2] * 5 for _ in range(5)]
    grid[3][1] = 0
    ok = all(g48.spawn_tile([row[:] for row in grid], rng)[:2] == (3, 1) for _ in range(50))
    check(ok and g48.spawn_tile([[2, 4], [8, 16]], rng) is None,
          "Spawn nur auf freie Felder, None bei vollem Brett")


def audit_web_parity():
    print("\n2048 - Web-Fassung rechnet identisch (Node)")
    node = shutil.which("node")
    if not node:
        print("  SKIP node nicht gefunden")
        return
    rng = random.Random(99)
    pool = [0, 0, 2, 2, 4, 8, 8, 16, 1024, 2048]
    cases = []
    for n in g48.SIZES:
        for _ in range(60):
            grid = [[rng.choice(pool) for _c in range(n)] for _r in range(n)]
            cases.append((grid, rng.choice("LRUD")))
    tmp = tempfile.mkdtemp(prefix="audit2048-")
    try:
        cfile = os.path.join(tmp, "cases.json")
        ofile = os.path.join(tmp, "out.json")
        jfile = os.path.join(tmp, "run.js")
        with open(cfile, "w", encoding="utf-8") as f:
            json.dump(cases, f)
        with open(jfile, "w", encoding="utf-8") as f:
            f.write("""
const fs = require("fs"), vm = require("vm");
const [src, cases, out] = process.argv.slice(2);
const ctx = { console };
ctx.window = ctx;
ctx.PG = { ui: {}, draw: {}, t: (k) => k, Game: class {}, register() {},
           rand: { random: Math.random, choice: (a) => a[0] } };
vm.createContext(ctx);
vm.runInContext(fs.readFileSync(src, "utf8"), ctx);
const res = JSON.parse(fs.readFileSync(cases, "utf8")).map(([g, d]) => {
  const p = ctx.PG.g2048.planMove(g, d);
  return [p.grid, p.gained, p.slides, p.merges, p.moved];
});
fs.writeFileSync(out, JSON.stringify(res));
""")
        src = os.path.join(REPO, "web", "js", "games", "game2048.js")
        subprocess.run([node, jfile, src, cfile, ofile], check=True,
                       capture_output=True, timeout=60)
        with open(ofile, encoding="utf-8") as f:
            js = json.load(f)
        bad = 0
        for (grid, d), got in zip(cases, js):
            p = g48.plan_move(grid, d)
            want = [p["grid"], p["gained"], [list(s) for s in p["slides"]],
                    [list(m) for m in p["merges"]], p["moved"]]
            bad += want != got
        check(len(js) == len(cases) and bad == 0,
              "planMove (JS) = plan_move (Python) für %d Züge" % len(cases),
              "%d Abweichungen" % bad)
    except (subprocess.SubprocessError, OSError, ValueError) as e:
        check(False, "Node-Vergleich lief durch", str(e)[:200])
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


# ============================================================ (5)-(11) Spiel

PLAIN = [[2, 0, 0, 2], [0, 4, 0, 0], [0, 0, 8, 0], [16, 0, 0, 0]]


def audit_input():
    print("\n2048 - Eingabe")
    wipe_store()
    g = make()
    check(g.state == g48.SETUP, "Start im Setup-Screen")
    key(g, "Return")
    n_tiles = sum(1 for row in g.grid for v in row if v)
    check(g.state == g48.PLAY and n_tiles == 2 and g.score == 0,
          "Enter startet eine Partie mit zwei Kacheln")
    settle(g)
    set_grid(g, PLAIN)
    key(g, "Left")
    check(g.moves == 1 and g.slide is not None and g.points == 4,
          "Pfeil links: Zug + Gleit-Animation + Punkte")
    key(g, "Right")
    key(g, "d")
    check(g.moves == 1 and g.queue == ["R", "R"], "Eingaben während der Animation gepuffert",
          "queue=%s" % g.queue)
    settle(g, 1.0)
    check(g.moves >= 2 and not g.queue and g.slide is None,
          "Puffer wird nach der Animation abgearbeitet", "moves=%d" % g.moves)
    before = g.moves
    set_grid(g, PLAIN)
    key(g, "Up", repeat=True)
    check(g.moves == before and g.slide is None, "Tasten-Wiederholung (repeat) wird ignoriert")

    # Wischen
    set_grid(g, PLAIN)
    br = g.board_rect
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=br.center))
    g.handle_event(InputEvent(InputEvent.MOUSEUP, pos=(br.centerx + 5, br.centery + 3)))
    check(g.moves == before and g.slide is None, "Mini-Bewegung ist kein Wisch")
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=br.center))
    g.handle_event(InputEvent(InputEvent.MOUSEUP, pos=(br.centerx - 40, br.centery + 160)))
    check(g.moves == before + 1 and g.grid[3][0] == 16 and g.grid[3][3] == 2,
          "Wisch nach unten schiebt nach unten", "grid=%s" % g.grid)
    settle(g)
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=br.center))
    g.handle_event(InputEvent(InputEvent.MOUSEUP, pos=(br.centerx + 90, br.centery + 80)))
    check(g.moves == before + 1, "diagonaler Wisch wird ignoriert")
    # Knopf "Setup" in der Info-Spalte
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=g.btn_rects[2].center))
    check(g.state == g48.SETUP and g.data["saves"].get("4-classic") is not None,
          "Klick auf Setup parkt die Partie als Spielstand")


def audit_undo():
    print("\n2048 - Rückgängig")
    wipe_store()
    g = started()
    set_grid(g, PLAIN)
    g.points = 100
    g.moves = 5
    g._sync_score()
    check(g.score == 100 and g.show_highscore_banner, "4x4 Klassisch ohne Undo zählt (score = Punkte)")
    snap = [row[:] for row in g.grid]
    key(g, "Left")
    settle(g)
    mid = [row[:] for row in g.grid]
    key(g, "u")
    check(g.slide is not None and g.slide["undo"], "Undo spielt eine Rück-Animation")
    settle(g)
    check(g.grid == snap and g.points == 100 and g.moves == 5,
          "Undo stellt Brett, Punkte und Züge wieder her")
    check(g.assisted and g.score == 0 and not g.show_highscore_banner,
          "Undo -> unterstützte Partie: score 0, kein Highscore-Banner")
    check(g._undo_left() == 2 and mid != snap, "3 pro Partie: noch 2 übrig")
    for d in ("Right", "Left", "Right", "Left"):
        key(g, d)
        settle(g)
    ok = True
    for _ in range(2):
        key(g, "BackSpace")
        settle(g)
    left_after = g._undo_left()
    moves_before = g.moves
    key(g, "u")
    settle(g)
    check(left_after == 0 and g.moves == moves_before and g.msg is not None,
          "nach 3 Undos ist Schluss (Meldung statt Undo)")

    g = started(undo="off")
    set_grid(g, PLAIN)
    key(g, "Left")
    settle(g)
    key(g, "u")
    settle(g)
    check(not g.stack and not g.assisted and g.moves == 1, "Undo aus: kein Stapel, keine Wirkung")

    g = started(undo="unlimited")
    set_grid(g, PLAIN)
    for i in range(12):
        key(g, "Left" if i % 2 else "Right")
        settle(g, 0.3)
    n = g.moves
    for _ in range(n):
        key(g, "u")
        settle(g, 0.3)
    check(g.moves == 0 and g.grid == PLAIN, "Unbegrenzt: alle %d Züge rückgängig bis zum Start" % n)

    # Undo-Taste belegt -> fällt weg
    g = started()
    g.controls = {"p1": dict(g.controls["p1"], action="u"), "p2": g.controls["p2"]}
    set_grid(g, PLAIN)
    key(g, "Left")
    settle(g)
    key(g, "u")
    settle(g)
    check(not g.assisted, "Undo-Taste greift nicht, wenn sie einer Aktion zugeordnet ist")


def audit_highscore_rules():
    print("\n2048 - Highscore nur 4x4 Klassisch")
    wipe_store()
    for size, mode, want in ((4, "classic", True), (5, "classic", False), (3, "classic", False),
                             (4, "time", False), (4, "endless", False), (8, "endless", False)):
        g = started(size=size, mode=mode)
        g.points = 500
        g._sync_score()
        check((g.score == 500) == want and g.show_highscore_banner == want,
              "%dx%d %s: %s" % (size, size, mode, "zählt" if want else "score 0, kein Banner"))

    # Verworfene zählende Partie -> Highscore gesichert
    import highscore
    g = started()
    set_grid(g, PLAIN)
    key(g, "Left")
    settle(g)
    g.points = 777
    g._sync_score()
    key(g, "r")
    hs = highscore.load_highscores().get("2048", 0)
    check(hs == 777 and g.points == 0 and g.moves == 0,
          "Neue Partie sichert den Highscore der verworfenen Partie", "hs=%s" % hs)


def win_grid():
    return [[1024, 1024, 0, 0], [2, 4, 8, 16], [32, 64, 128, 256], [0, 0, 0, 0]]


def audit_win():
    print("\n2048 - Sieg, Endlos, Erfolge")
    wipe_store()
    g = started()
    set_grid(g, win_grid())
    key(g, "Left")
    check(g.state == g48.PLAY and g.win_pending, "2048 entsteht - Panel erst nach der Animation")
    settle(g)
    check(g.state == g48.WIN and ("ach", "tile_2048") in g.events and ("result", True) in g.events,
          "Klassisch: Weiterspielen?-Panel, Sieg gemeldet, Erfolg tile_2048")
    moves = g.moves
    key(g, "Left")
    check(g.moves == moves, "Pfeile wirken nicht, solange das Panel offen ist")
    key(g, "Return")
    check(g.state == g48.PLAY and g.continued, "Enter spielt weiter")
    set_grid(g, [[2048, 2048, 0, 0], [2, 4, 8, 16], [32, 64, 128, 256], [0, 0, 0, 0]])
    key(g, "Left")
    settle(g)
    check(g.state == g48.PLAY and ("ach", "tile_4096") in g.events,
          "4096 -> Erfolg tile_4096, kein zweites Panel")

    g = started()
    set_grid(g, win_grid())
    key(g, "Left")
    settle(g)
    key(g, "n")
    check(g.state == g48.PLAY and g.moves == 0 and g.points == 0, "N im Panel startet eine neue Partie")

    g = started(mode="endless")
    set_grid(g, win_grid())
    key(g, "Left")
    for _ in range(12):
        g.update(1 / 60)
    check(g.state == g48.PLAY and g.flash is not None and ("result", True) in g.events,
          "Endlos: keine Unterbrechung, nur Einblendung")

    g = started(mode="time")
    set_grid(g, win_grid())
    key(g, "Left")
    settle(g)
    check(g.state == g48.PLAY and ("ach", "tile_2048") in g.events
          and not any(e[0] == "result" for e in g.events),
          "Zeitangriff: Erfolg ja, keine Sieg/Niederlage-Statistik")

    g = started(size=5, mode="endless")
    grid = [[0] * 5 for _ in range(5)]
    grid[0][0] = grid[0][1] = 1024
    set_grid(g, grid)
    key(g, "Left")
    settle(g)
    check(("ach", "tile_2048") not in g.events and ("result", True) in g.events,
          "5x5: Sieg zählt, Kachel-Erfolg nicht")

    g = started()
    set_grid(g, [[1024, 512, 512, 0], [2, 4, 8, 16], [32, 64, 128, 256], [0, 0, 0, 0]])
    key(g, "Down")
    settle(g)
    key(g, "u")
    settle(g)
    set_grid(g, win_grid())
    key(g, "Left")
    settle(g)
    check(g.state == g48.WIN and not any(e[0] in ("ach", "result") for e in g.events),
          "mit Undo: Panel ja, aber kein Sieg und kein Erfolg")


STUCK_AFTER_RIGHT = [[8, 16, 32, 0], [64, 128, 256, 512], [8, 16, 32, 64],
                     [128, 256, 512, 1024]]


def audit_game_over():
    print("\n2048 - Game Over und Zeitangriff")
    wipe_store()
    g = started()
    set_grid(g, STUCK_AFTER_RIGHT)
    g.points = 900
    key(g, "Left")                # kein Effekt -> Brett stößt an
    check(g.moves == 0 and g.nudge is not None, "Zug ohne Wirkung: Brett stößt an, kein Zug")
    key(g, "Right")
    settle(g, 0.1)
    check(not g.game_over and g.over_t is not None, "letzter Zug klingt erst aus")
    settle(g, 1.5)
    check(g.game_over and g.over_reason == "stuck" and ("result", False) in g.events
          and "4-classic" not in g.data["saves"],
          "Game Over: Niederlage gemeldet, Spielstand gelöscht")
    check(g.new_best and g.data["best"].get("4-classic", 0) >= 900 and g.score == g.points,
          "Bestwert und Highscore-Punkte stehen fest")
    key(g, "u")
    settle(g)
    check(not g.game_over and g.grid == STUCK_AFTER_RIGHT and g.assisted and g.score == 0,
          "Undo nach Game Over holt den letzten Zug zurück (unterstützt)")

    g = started(mode="time")
    settle(g, 5.0)
    check(g.time_left == g48.TIME_LIMIT and not g.clock_on, "Uhr läuft erst ab dem ersten Zug")
    set_grid(g, PLAIN)
    key(g, "Left")
    for _ in range(int(181 * 20)):
        if g.game_over:
            break
        g.update(0.05)
    check(g.game_over and g.over_reason == "time" and g.time_left == 0
          and not any(e[0] == "result" for e in g.events),
          "Zeitangriff endet nach 3 Minuten (ohne Statistik-Ergebnis)")
    key(g, "u")
    rect, _hh, btns, _p, _g = g._panel_geom(False)
    check(g.game_over and all(a != "undo" for _b, a in btns),
          "abgelaufene Zeit lässt sich nicht rückgängig machen")


def audit_save_resume():
    print("\n2048 - Speichern & Fortsetzen")
    wipe_store()
    g = started(size=5, mode="time", undo="unlimited")
    g.rng.seed(3)
    for d in "LURDLURDLU":
        key(g, {"L": "Left", "R": "Right", "U": "Up", "D": "Down"}[d])
        settle(g, 0.3)
    key(g, "u")
    settle(g, 0.3)
    key(g, "Left")
    settle(g, 2.0)
    snap = (g.grid, g.points, g.moves, g.undo_used, g.assisted, round(g.time_left, 1))
    stack_n = min(len(g.stack), g48.SAVE_STACK)
    check(store.load_section("g2048").get("saves", {}).get("5-time") is not None,
          "Autosave hat die Partie gedrosselt gespeichert")
    g.on_exit()

    h = make(size=5, mode="time", undo="unlimited")
    sv = h._saved()
    check(sv is not None and sv["moves"] == snap[2], "neues Spiel sieht den Spielstand im Setup")
    h.draw()
    key(h, "Return")                # setup_act 0 = Fortsetzen
    got = (h.grid, h.points, h.moves, h.undo_used, h.assisted, round(h.time_left, 1))
    check(h.state == g48.PLAY and got == snap and len(h.stack) == stack_n,
          "Fortsetzen stellt Brett, Punkte, Züge, Undo, Restzeit und Stapel her",
          "%s vs %s" % (got[1:], snap[1:]))
    grid_before = [row[:] for row in h.grid]
    key(h, "u")
    settle(h)
    check(h.grid != grid_before and h.moves == snap[2] - 1, "Undo nach dem Fortsetzen funktioniert")

    # andere Größe/anderer Modus bleibt unberührt, Neue Partie verwirft
    k = make(size=4, mode="classic")
    check(k._saved() is None, "Spielstände sind je Größe/Modus getrennt")
    m = make(size=5, mode="time", undo="unlimited")
    for _ in range(3):              # Fokus auf die Knopfzeile
        key(m, "Down")
    key(m, "Right")                 # Knopf "Neue Partie" wählen
    check(m.setup_act == 1, "Setup: Pfeile wählen zwischen Fortsetzen und Neue Partie",
          "act=%s" % m.setup_act)
    key(m, "Return")
    check(m.moves == 0 and "5-time" not in store.load_section("g2048").get("saves", {}),
          "Neue Partie verwirft den Spielstand")

    # kaputte Daten
    bad = {"grid": [[2, 3], [0, 0]]}
    bad2 = {"grid": [[2, 0], [0, 0], [0, 0]]}
    bad3 = {"grid": [[True, 0], [0, 0]]}
    good = {"grid": [[2, 0], [0, 4096]], "points": -5, "moves": "x", "time_left": 999,
            "stack": [{"grid": [[1, 1], [1, 1]]}, {"grid": [[2, 2], [0, 0]], "slides": [[0, 0, 5, 5, 2]]}]}
    cg = g48.clean_save(good, 2)
    check(g48.clean_save(bad, 2) is None and g48.clean_save(bad2, 2) is None
          and g48.clean_save(bad3, 2) is None and cg is not None
          and cg["points"] == 0 and cg["moves"] == 0 and cg["time_left"] == g48.TIME_LIMIT
          and len(cg["stack"]) == 1 and cg["stack"][0]["slides"] == [],
          "kaputte Spielstände werden verworfen bzw. bereinigt")
    raw = store.load()
    raw["g2048"] = {"best": {"4-classic": "viel", "9-classic": 5}, "saves": {"4-classic": bad},
                    "top_tile": True}
    store.save(raw)
    z = make()
    check(z.data["best"] == {} and z.data["saves"] == {} and z.data["top_tile"] == 0,
          "mem.json-Section mit Unsinn lädt als leerer Stand")


def audit_setup_and_resize():
    print("\n2048 - Setup & Auflösungswechsel")
    wipe_store()
    SAVED_SETTINGS.clear()
    g = make()
    key(g, "7")
    check(g.size == 7 and SAVED_SETTINGS and SAVED_SETTINGS[-1]["g2048"]["size"] == 7,
          "Taste 7 wählt 7x7 und speichert sofort")
    key(g, "Down")
    key(g, "Right")
    check(g.gmode == "time" and SAVED_SETTINGS[-1]["g2048"]["mode"] == "time",
          "Runter + Rechts wechselt den Modus")
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=g.undo_rects[2].center))
    check(g.undo_rule == "unlimited" and SAVED_SETTINGS[-1]["g2048"]["undo"] == "unlimited",
          "Klick wählt Rückgängig: Unbegrenzt")
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=g.start_rect.center))
    check(g.state == g48.PLAY and g.size == 7 and len(g.grid) == 7 and g.time_rect.h > 0,
          "START-Knopf startet 7x7 Zeitangriff mit Zeitbalken")
    g.rng.seed(1)
    for d in ("Left", "Up", "Right"):
        key(g, d)
        settle(g, 0.3)
    key(g, "Down")
    g.update(1 / 60)
    grid = [row[:] for row in g.grid]
    pts = g.points
    for w, h in ((1280, 960), (480, 360)):
        g.surface = pygame.Surface((w, h))
        g.width, g.height = w, h
        g.on_surface_changed()
        g.draw()
    settle(g)
    inside = (g.board_rect.right <= 480 and g.side_rect.right <= 480
              and g.side_rect.bottom <= 360)
    check(g.grid == grid and g.points == pts and inside and g.state == g48.PLAY,
          "Auflösungswechsel mitten in der Animation: Partie bleibt, Layout passt")


# ============================================================ (12) Layout

def rects_overlap(rects):
    for i, a in enumerate(rects):
        for b in rects[i + 1:]:
            if a.colliderect(b):
                return (tuple(a), tuple(b))
    return None


def fit_problems(g, tag):
    """Notschriften: stark geschrumpfte Texte gelten als "passt nicht"."""
    out = []
    for (text, px, max_w, bold), size in g._fit_sizes.items():
        if text.replace("+", "").isdigit():
            continue
        font = ui.font(size, bold=bold)
        if font.size(text)[0] > max_w:
            out.append("%s: zu breit '%s'" % (tag, text))
        elif size < 9 or size < px * 0.62:
            out.append("%s: %d->%dpx '%s'" % (tag, px, size, text[:30]))
    return out


def audit_layout():
    print("\n2048 - Layout (5 Auflösungen x 14 Sprachen x Größen 3-8)")
    lang_before = i18n.get_language()
    theme_before = ui.theme_name()
    for w, h in RESOLUTIONS:
        # Geometrie je Brettgröße (sprachunabhängig)
        geo = []
        for size in g48.SIZES:
            for mode in ("classic", "time"):
                g = make(w, h, size=size, mode=mode)
                setup_rects = (g.size_rects + g.mode_rects + g.undo_rects
                               + [g.info_rect_setup, g.start_rect])
                both = [g.cont_rect, g.new_rect]
                if any(r.left < 0 or r.right > w or r.top < 0 or r.bottom > h - 34
                       for r in setup_rects + both):
                    geo.append("setup außerhalb %dx%d" % (size, size))
                if rects_overlap(setup_rects) or rects_overlap(g.size_rects + g.mode_rects + g.undo_rects
                                                               + [g.info_rect_setup] + both):
                    geo.append("setup überlappt")
                lh = g._tiny.get_height()
                for upper, lower in ((g.size_rects, g.mode_rects), (g.mode_rects, g.undo_rects)):
                    if lower[0].top - upper[0].bottom < lh + 3:
                        geo.append("Beschriftung eingeklemmt")
                g._start(resume=False)
                side = [g.tag_rect, g.score_rect, g.best_rect, g.info_rect] + g.btn_rects
                if (g.board_rect.left < 0 or g.board_rect.bottom > h or g.side_rect.right > w
                        or g.side_rect.bottom > h or g.board_rect.colliderect(g.side_rect)):
                    geo.append("Brett/Spalte %dx%d %s" % (size, size, mode))
                if any(not g.side_rect.contains(r) for r in side) or rects_overlap(side):
                    geo.append("Info-Spalte %dx%d" % (size, size))
                if mode == "time" and (g.time_rect.bottom > h or g.time_rect.top <= g.board_rect.bottom):
                    geo.append("Zeitbalken")
                if g.cell < 24 and size <= 6:
                    geo.append("Zellen zu klein %d" % g.cell)
                for win in (True, False):
                    rect, _hh, btns, _p, _g = g._panel_geom(win)
                    if (not g.board_rect.contains(rect)
                            or any(not rect.contains(b) for b, _a in btns)
                            or rects_overlap([b for b, _a in btns])):
                        geo.append("Panel %dx%d" % (size, size))
                # Performance-unabhängig: jede Größe einmal zeichnen
                set_grid(g, [[2 ** ((r * size + c) % 17 + 1) for c in range(size)]
                             for r in range(size)])
                g.draw()
        check(not geo, "%4dx%d: Geometrie aller Größen/Modi" % (w, h), "; ".join(sorted(set(geo))[:5]))

        probs = []
        for theme in ("v42", "v1"):
            ui.set_theme(theme)
            for code, _name in i18n.AVAILABLE:
                if theme == "v1" and code not in ("de", "fi", "pl", "hr"):
                    continue
                i18n.set_language(code, persist=False)
                wipe_store()
                for mode, undo in (("classic", "limited"), ("time", "unlimited"), ("endless", "off")):
                    g = make(w, h, size=4, mode=mode, undo=undo)
                    g.draw()                                       # Setup ohne Spielstand
                    g._start(resume=False)
                    set_grid(g, win_grid())
                    g.points, g.moves = 123456, 4321
                    key(g, "Left")
                    for _ in range(3):
                        g.update(1 / 60)
                    g.draw()                                       # mitten in der Animation
                    settle(g, 0.5)
                    g._panel_ticks = -10 ** 9
                    g.draw()                                       # Sieg-Panel (Klassisch)
                    if g.state == g48.WIN:
                        key(g, "Return")
                    g._message(i18n.t("g2048.undo_empty"))
                    g.undo_used = 1
                    g.assisted = True
                    g.draw()                                       # Spiel + Meldung
                    g._finish(timeout=(mode == "time"))
                    g.draw()                                       # Endstand-Panel
                    g.new_best = False
                    g.draw()
                    g._to_setup()
                    g.data["best_assisted"]["4-" + mode] = 999999
                    g.data["best_tile"]["4-" + mode] = 131072
                    g.data["saves"]["4-" + mode] = g48.clean_save({"grid": win_grid(), "points": 9876543,
                                                                   "moves": 99999}, 4)
                    g.draw()                                       # Setup mit Fortsetzen
                    probs += fit_problems(g, "%s/%s/%s" % (theme, code, mode))
        i18n.set_language(lang_before, persist=False)
        ui.set_theme(theme_before)
        check(not probs, "%4dx%d: Texte passen (14 Sprachen, v4.2 + v1)" % (w, h),
              "; ".join(sorted(set(probs))[:6]))
    wipe_store()


# ============================================================ (13) Tempo

def audit_performance():
    print("\n2048 - Tempo 8x8 bei 1280x960")
    wipe_store()
    theme_before = ui.theme_name()
    for theme in ("v42", "v1"):
        ui.set_theme(theme)
        g = started(w=1280, h=960, size=8, mode="time", undo="unlimited")
        rng = random.Random(5)
        big = [2 ** k for k in range(1, 18)]

        def refill():
            set_grid(g, [[rng.choice(big) if rng.random() < 0.8 else 0 for _c in range(8)]
                         for _r in range(8)])

        refill()
        for _ in range(30):                      # Caches warm laufen lassen
            g.draw()
            g.update(1 / 60)
        times = []
        dirs = "LURD"
        for i in range(360):
            t0 = time.perf_counter()
            if g.slide is None and not g.game_over:
                if not g._do_move(dirs[i % 4]):
                    refill()
                    g._do_move(dirs[(i + 1) % 4])
            if g.over_t is not None or g.game_over:
                g.game_over = False
                refill()
            g.update(1 / 60)
            g.draw()
            times.append(time.perf_counter() - t0)
        times.sort()
        avg = sum(times) / len(times) * 1000
        p95 = times[int(len(times) * 0.95)] * 1000
        check(avg < 10.0 and p95 < 16.0,
              "%s: Frame im Schnitt %.2f ms, p95 %.2f ms (Budget 16,7 ms)" % (theme, avg, p95))
    ui.set_theme(theme_before)


if __name__ == "__main__":
    t0 = time.time()
    wipe_store()
    audit_logic()
    audit_web_parity()
    audit_input()
    audit_undo()
    audit_highscore_rules()
    audit_win()
    audit_game_over()
    audit_save_resume()
    audit_setup_and_resize()
    audit_layout()
    audit_performance()
    wipe_store()
    print("\n%s  (%.1f s)" % ("ALLE PRÜFUNGEN BESTANDEN" if not FAILS
                              else "%d FEHLER: %s" % (len(FAILS), FAILS),
                              time.time() - t0))
    sys.exit(1 if FAILS else 0)
