# -*- coding: utf-8 -*-
"""Headless-Audit für Crossy Road (Desktop + Abgleich mit der Web-Fassung).

Geprüft wird:

Generator (1) über viele Seeds und je 500 Reihen gibt es IMMER einen
              begehbaren Weg (eigene Breitensuche, unabhängig vom Generator),
              Münzen liegen nur auf erreichbaren Feldern, Lücken zwischen
              Fahrzeugen/Stämmen bleiben fair,
          (2) Tagesstrecke deterministisch; Python == JavaScript Reihe für
              Reihe (node web/tools/crossyroad_dump.js), Deko-Hash gleich,
              web/js/games/crossyroad_models.js ist aktuell,
          (3) Züge: Warnlicht mindestens ~1 s bevor ein Zug das Feld erreicht.
Regeln    (4) Auto überfährt, Zug erfasst, Wasser, Stamm trägt mit und treibt
              über den Rand, Baum blockiert, Eingabe-Puffer, Tastenwiederholung
              wird ignoriert, Adler (Trödeln / Zurückgehen), Münzen,
              Erfolg "5 Gleise in Folge", Punkte = weiteste Reihe,
              Tagesstrecke füllt keinen Highscore.
Speicher  (5) Figurenkauf/Auswahl, Münzen, Tagesbestwert, kaputte Daten,
              Settings-Section crossy (shadows/daynight) überlebt das Laden.
Layout    (6) Setup, Figuren-Reiter, Game-Over-Panel und HUD: alle 5
              Auflösungen x 14 Sprachen - im Bild, ohne Überlappung, Texte
              passen ohne Kürzung; alle Screens zeichnen fehlerfrei (v4.2 + v1),
              Auflösungswechsel mitten im Spiel.
Leistung  (7) 1280x960: Frame-Zeit (update + draw) bei Tag und bei Nacht.

Aufruf aus dem Repo-Root:  python tests/audit_crossyroad.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import json
import os
import shutil
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

import pygame
pygame.init()
pygame.display.set_mode((640, 480))

import store
store._PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                           "_audit-crossy-mem.json")
import settings as settings_mod
settings_mod.save_settings = lambda s: None
import i18n
i18n.init()
import ui
import seedrand
from game_base import InputEvent

# Das games-Paket importiert ALLE Spiele. Laufen parallel Arbeiten an anderen
# Spielen, darf deren Zwischenstand diesen Audit nicht umwerfen - dann wird
# nur das Paket-Verzeichnis eingebunden.
REGISTERED = None
try:
    import games as _games
    REGISTERED = any(c.__name__ == "CrossyRoadGame" for c in _games.ALL_GAMES)
except Exception as exc:          # noqa: BLE001
    print("  (Hinweis: games/__init__ nicht importierbar: %s)" % exc)
    for name in [m for m in sys.modules if m == "games" or m.startswith("games.")]:
        del sys.modules[name]
    _pkg = types.ModuleType("games")
    _pkg.__path__ = [os.path.join(REPO, "games")]
    sys.modules["games"] = _pkg
from games import crossyroad as cr
from games import crossyroad_world as cw

GS = json.loads(json.dumps(settings_mod.DEFAULTS))
FAILS = []
RES = ((480, 360), (640, 480), (800, 600), (960, 720), (1280, 960))
DT = 1 / 60.0


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + detail) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


def quiet(game):
    game.play_sound = lambda name: None
    game.rumble = lambda ms=0: None
    game.events = []
    game.ach_event = lambda *a, **k: game.events.append(a[0] if a else None)
    game.report_result = lambda won: None
    return game


def new_game(mode="endless", w=640, h=480):
    return quiet(cr.CrossyRoadGame(pygame.Surface((w, h)), w, h, mode=mode,
                                   game_settings=GS))


def start(g, seed):
    g._new_run()
    g.world = cw.World(seed)
    return g


def place(g, r, col, clock=None):
    g.row = r
    g.col = float(col)
    g.best = max(g.best, r)
    g.cam = g.cam_target = float(g.best)
    g.creep = g.best - cw.CREEP_LAG
    g.started = False                 # keine Kamera-Kriechgrenze -> kein Adler
    g.hop = g.log = g.buffer = g.death = None
    if clock is not None:
        g.clock = clock


def key(g, k, repeat=False):
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key=k, repeat=repeat))


def run(g, seconds):
    for _ in range(int(round(seconds / DT))):
        if g.game_over:
            break
        g.update(DT)


def wipe_mem():
    """Eigene mem-Datei samt .bak/.tmp entfernen (store liest sonst die .bak)."""
    for suffix in ("", ".bak", ".tmp"):
        try:
            os.remove(store._PATH + suffix)
        except OSError:
            pass


def free_col(world, r, c):
    row = world.row(r)
    return 0 <= c < cw.COLS and not (row["trees"] and row["trees"][c])


# ------------------------------------------------------------------ Generator

def allowed(row, c):
    if row["kind"] == cw.GRASS:
        return row["trees"][c] == 0
    if row["kind"] == cw.RIVER and row["pads"] is not None:
        return row["pads"][c] == 1
    return True


def walkable(world, n):
    """Breitensuche vorwärts/seitwärts: erreichbare Spalten je Reihe."""
    reach = [cw.START_COL]
    for r in range(n):
        row = world.row(r)
        entry = [c for c in reach if allowed(row, c)] if r else [cw.START_COL]
        seen = set(entry)
        stack = list(entry)
        while stack:
            c = stack.pop()
            for nb in (c - 1, c + 1):
                if 0 <= nb < cw.COLS and nb not in seen and allowed(row, nb):
                    seen.add(nb)
                    stack.append(nb)
        if not seen:
            return r, None
        reach = sorted(seen)
        coin = row["coin"]
        if coin and coin[0] not in reach:
            return r, "Münze unerreichbar"
    return None, None


def lane_gaps(row):
    """(kleinste, größte) Lücke zwischen Objekten einer Spur (mit Umlauf)."""
    objs = sorted(row["objs"])
    gaps = []
    for i, o in enumerate(objs):
        nxt = objs[(i + 1) % len(objs)]
        start = nxt[0] + (cw.LOOP if i + 1 == len(objs) else 0)
        gaps.append(start - (o[0] + o[1]))
    return min(gaps), max(gaps)


def audit_generator():
    print("\nGenerator - begehbarer Weg, Münzen, faire Lücken")
    bad = []
    worst_log = 0.0
    tight_road = 99.0
    kinds = {}
    for seed in list(range(120)) + [0xFFFFFFFF, 987654321, seedrand.daily_seed("crossy", "2026-09-16")]:
        world = cw.World(seed)
        r, why = walkable(world, 500)
        if r is not None:
            bad.append("seed %d Reihe %d %s" % (seed, r, why or "kein Weg"))
        for rr in range(500):
            row = world.row(rr)
            kinds[row["kind"]] = kinds.get(row["kind"], 0) + 1
            if row["objs"]:
                lo, hi = lane_gaps(row)
                if row["kind"] == cw.RIVER:
                    worst_log = max(worst_log, hi)
                    if lo < 0.99:
                        bad.append("seed %d Reihe %d Stämme überlappen" % (seed, rr))
                else:
                    tight_road = min(tight_road, lo)
    check(not bad, "123 Seeds x 500 Reihen: immer ein Weg, Münzen erreichbar",
          "; ".join(bad[:5]))
    check(worst_log <= 9.0, "größte Lücke zwischen Stämmen <= 9 Spalten (%.2f)" % worst_log)
    check(tight_road >= 2.0 - 1e-9, "kleinste Lücke zwischen Fahrzeugen >= 2 Spalten (%.2f)" % tight_road)
    check(all(kinds.get(k, 0) > 1000 for k in cw.KINDS),
          "alle vier Reihenarten kommen reichlich vor", str(kinds))
    world = cw.World(5)
    starts = [world.row(r) for r in range(cw.START_ROWS)]
    check(all(row["kind"] == cw.GRASS and row["trees"][cw.START_COL] == 0 for row in starts)
          and starts[0]["coin"] is None and not any(starts[0]["trees"]),
          "Startwiese: frei an der Startspalte, Reihe 0 ohne Bäume/Münze")
    back = [cw.back_row(r) for r in (-1, -2, -3)]
    check(back[0]["trees"][cw.START_COL] == 0 and all(back[2]["trees"]),
          "hinter dem Start: Reihe -3 ist eine geschlossene Baumwand")


def audit_daily_and_js():
    print("\nTagesstrecke & Python == JavaScript")
    day = "2026-09-16"
    a = cw.World(seedrand.daily_seed("crossy", day)).export(300)
    b = cw.World(seedrand.daily_seed("crossy", day)).export(300)
    c = cw.World(seedrand.daily_seed("crossy", "2026-09-17")).export(300)
    check(a == b, "gleiches Datum -> identische Strecke")
    check(a != c, "anderes Datum -> andere Strecke")
    # Reihen in anderer Reihenfolge abgerufen -> trotzdem gleich
    w = cw.World(seedrand.daily_seed("crossy", day))
    w.row(250)
    check(w.export(300) == a, "Abruf-Reihenfolge ändert nichts (Reihe 250 zuerst)")
    node = shutil.which("node")
    if not node:
        check(False, "node gefunden (für den Vergleich mit crossyroad_world.js)")
        return
    dump = os.path.join(REPO, "web", "tools", "crossyroad_dump.js")
    diffs = []
    for seed in (1, 42, 777, 123456, 0xFFFFFFFF):
        js = json.loads(subprocess.check_output([node, dump, str(seed), "400"]))
        py = json.loads(json.dumps(cw.World(seed).export(400)))
        if js["rows"] != py:
            first = next(i for i, (x, y) in enumerate(zip(js["rows"], py)) if x != y)
            diffs.append("seed %d ab Reihe %d" % (seed, first))
    check(not diffs, "5 Seeds x 400 Reihen: JS == Python", "; ".join(diffs))
    js = json.loads(subprocess.check_output([node, dump, "daily", day, "300"]))
    check(js["seed"] == seedrand.daily_seed("crossy", day)
          and js["rows"] == json.loads(json.dumps(a)),
          "Tagesstrecke %s: JS == Python (Seed + 300 Reihen)" % day)
    same = all(json.loads(subprocess.check_output([node, dump, "hash", str(r), str(cc)]))
               == cw.deco_hash(r, cc) for r, cc in ((-7, 3), (0, -4), (812, 12)))
    check(same, "Deko-Hash (Bäume am Rand) JS == Python")
    out = subprocess.run([sys.executable, os.path.join(REPO, "devtools", "build_crossyroad_models.py"),
                          "--check"], capture_output=True, text=True)
    check(out.returncode == 0, "web/js/games/crossyroad_models.js ist aktuell", out.stdout.strip())


def audit_trains():
    print("\nZüge - Warnung vor der Einfahrt")
    worst = 99.0
    for seed in range(30):
        world = cw.World(seed)
        for r in range(300):
            row = world.row(r)
            if row["kind"] != cw.RAIL:
                continue
            u = 0.0
            while u < row["period"]:
                span = cw.train_span(row, u - row["phase"])
                if span and span[0] < cw.COLS + 0.5 and span[1] > -0.5:
                    worst = min(worst, u)
                    break
                u += 0.005
    check(worst >= cw.TRAIN_WARN + 0.25,
          "Zug erreicht das Feld frühestens %.2f s nach Beginn der Warnung" % worst)


# ------------------------------------------------------------------ Regeln

def find(world, pred, start=3, limit=4000):
    for r in range(start, limit):
        if pred(r, world.row(r)):
            return r
    raise AssertionError("keine passende Reihe")


def audit_rules():
    print("\nRegeln - Kollisionen, Stamm, Adler, Münzen, Punkte")
    seed = 4242
    # --- Auto
    g = start(new_game(), seed)
    r = find(g.world, lambda r, row: row["kind"] == cw.ROAD)
    row = g.world.row(r)
    o = row["objs"][0]
    target = 4.5 - o[1] / 2 + cw.MARGIN
    dist = (target - o[0]) % cw.LOOP if row["dir"] > 0 else (o[0] - target) % cw.LOOP
    t_hit = dist / row["speed"] + cw.LOOP / row["speed"]
    place(g, r, 4, t_hit - DT / 2)
    g.update(DT)
    check(g.death is not None and g.death["cause"] == "car", "Auto überfährt die Figur")
    # --- Zug
    g = start(new_game(), seed)
    r = find(g.world, lambda r, row: row["kind"] == cw.RAIL)
    row = g.world.row(r)
    t = 0.0
    while not cw.train_hits(row, 4.5, t):
        t += 0.004
    place(g, r, 4, t - 0.002)
    g.update(DT)
    check(g.death is not None and g.death["cause"] == "train", "Zug erfasst die Figur")
    # --- Stamm: aufspringen, mitfahren, über den Rand treiben
    g = start(new_game(), seed)
    r = find(g.world, lambda r, row: row["kind"] == cw.RIVER and row["pads"] is None
             and g.world.row(r - 1)["kind"] == cw.GRASS and free_col(g.world, r - 1, 4))
    row = g.world.row(r)
    t = 0.0
    while cw.log_at(row, 4.5, t + cw.HOP_T, grace=-0.5) is None:
        t += 0.01
    place(g, r - 1, 4, t)
    key(g, "Up")
    run(g, 0.25)
    ok = g.death is None and g.row == r and g.log is not None
    check(ok, "Sprung auf einen Stamm: Figur steht darauf")
    if ok:
        g.started = False
        c0 = g.col
        run(g, 0.5)
        moved = g.col - c0
        expect = row["dir"] * row["speed"] * 0.5
        check(g.death is None and abs(moved - expect) < 0.05,
              "Stamm trägt die Figur mit (%.2f statt %.2f Spalten)" % (moved, expect))
        run(g, 30.0)
        check(g.death is not None and g.death["cause"] == "edge",
              "über den Bildrand getragen = raus", str(g.death and g.death["cause"]))
    # --- Wasser
    g = start(new_game(), seed)
    t = 0.0
    while any(cw.log_at(row, 4.5, t + cw.HOP_T + k * 0.02, grace=1.0) for k in range(4)):
        t += 0.01
    place(g, r - 1, 4, t)
    key(g, "w")
    run(g, 0.3)
    check(g.death is not None and g.death["cause"] == "water", "neben den Stamm = Platsch")
    # --- Baum blockiert
    g = start(new_game(), seed)
    rt = find(g.world, lambda r, row: row["kind"] == cw.GRASS and g.world.row(r - 1)["kind"] == cw.GRASS
              and any(row["trees"][c] and free_col(g.world, r - 1, c) for c in range(cw.COLS)))
    c = next(c for c in range(cw.COLS) if g.world.row(rt)["trees"][c] and free_col(g.world, rt - 1, c))
    place(g, rt - 1, c, 0.0)
    key(g, "Up")
    run(g, 0.3)
    check(g.row == rt - 1 and g.col == c and g.death is None, "Baum/Stein blockiert den Sprung")
    # --- Eingabe-Puffer + Tastenwiederholung
    g = start(new_game(), seed)
    place(g, 0, cw.START_COL, 0.0)
    key(g, "Up")
    g.update(DT)
    key(g, "Up")                      # während des Sprungs: wird vorgemerkt
    run(g, 0.5)
    check(g.row == 2 and g.death is None, "Eingabe-Puffer: zweiter Sprung folgt direkt (Reihe %d)" % g.row)
    key(g, "Up", repeat=True)
    run(g, 0.3)
    check(g.row == 2, "automatische Tastenwiederholung löst keinen Sprung aus")
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=(10, 10), button=1))
    run(g, 0.3)
    check(g.hops == 3, "Mausklick = vorwärts")
    # --- Adler: Trödeln
    g = start(new_game(), seed)
    place(g, 0, cw.START_COL, 0.0)
    g.started = True
    t0 = g.clock
    run(g, 30.0)
    idle = g.clock - t0
    check(g.death is not None and g.death["cause"] == "eagle" and 5.0 < idle < 15.0,
          "Adler holt die Figur nach %.1f s Stehen" % idle)
    # --- Adler: zurückgehen
    g = start(new_game(), seed)
    place(g, 20, 4, 0.0)
    g.best = 20
    g.row = 17
    g.started = True
    g._camera(DT)
    check(g.death is None, "3 Reihen zurück: noch kein Adler")
    g.row = 16
    g._camera(DT)
    check(g.death is not None and g.death["cause"] == "eagle", "4 Reihen zurück: Adler")
    # --- Münzen + Punkte
    g = start(new_game(), seed)
    rc = find(g.world, lambda r, row: row["kind"] == cw.GRASS and row["coin"] is not None
              and g.world.row(r - 1)["kind"] == cw.GRASS and free_col(g.world, r - 1, row["coin"][0]))
    coin = g.world.row(rc)["coin"]
    place(g, rc - 1, coin[0], 0.0)
    g.best = 0
    before = g.data["coins"]
    key(g, "Up")
    run(g, 0.3)
    check(g.data["coins"] == before + coin[1] and g.run_coins == coin[1] and rc in g.taken,
          "Münze einsammeln (+%d)" % coin[1])
    check(g.best == rc and g.score == rc, "Punkte = weiteste Reihe (%d)" % g.score)
    # --- Riesenmünze vorhanden?
    worlds = [cw.World(s) for s in range(3)]
    big = any((wd.row(r)["coin"] or [0, 0])[1] == cw.BIG_COIN
              for wd in worlds for r in range(20, 300))
    check(big, "Riesenmünzen (Wert %d) werden erzeugt" % cw.BIG_COIN)
    # --- Tagesstrecke füllt keinen Highscore
    g = start(new_game("daily"), seedrand.daily_seed("crossy"))
    place(g, 0, cw.START_COL, 0.0)
    key(g, "Up")
    run(g, 0.3)
    check(g.best == 1 and g.score == 0 and g.show_highscore_banner is False,
          "Tagesstrecke: score bleibt 0, kein Highscore-Banner")
    g2 = new_game("endless")
    check(g2.show_highscore_banner is True, "Endlos: Highscore-Banner an")
    # --- Erfolg: 5 Gleise in Folge
    seed5 = r5 = None
    for s in range(400):
        w = cw.World(s)
        for rr in range(10, 400):
            if w.row(rr)["kind"] == cw.RAIL and w.row(rr + 1)["kind"] == cw.GRASS \
                    and w.rail_run(rr) >= 5 and free_col(w, rr + 1, 4):
                seed5, r5 = s, rr
                break
        if seed5 is not None:
            break
    check(seed5 is not None, "Strecken mit 5 Gleisen hintereinander kommen vor")
    if seed5 is not None:
        g = start(new_game(), seed5)
        row = g.world.row(r5)
        t = 0.0
        while any(cw.train_hits(row, 4.5, t + k * 0.05, half=1.5) for k in range(8)):
            t += 0.05
        place(g, r5, 4, t)
        g.best = r5
        key(g, "Up")
        run(g, 0.3)
        check("crossy_train" in g.events and g.death is None, "Erfolg crossy_train nach 5 Gleisen",
              str(g.events))
        # nur 1-4 Gleise hintereinander -> kein Erfolg
        r4 = find(g.world, lambda rr, row: row["kind"] == cw.RAIL
                  and g.world.row(rr + 1)["kind"] == cw.GRASS
                  and 1 <= g.world.rail_run(rr) <= 4 and free_col(g.world, rr + 1, 4))
        g = start(new_game(), seed5)
        row = g.world.row(r4)
        t = 0.0
        while any(cw.train_hits(row, 4.5, t + k * 0.05, half=1.5) for k in range(8)):
            t += 0.05
        place(g, r4, 4, t)
        key(g, "Up")
        run(g, 0.3)
        ok4 = g.death is None and g.row == r4 + 1
        check(ok4 and "crossy_train" not in g.events, "weniger als 5 Gleise: kein Erfolg")


# ------------------------------------------------------------------ Speicher

def audit_persistence():
    print("\nSpeicher - Münzen, Figuren, Tagesbestwert, Settings")
    try:
        os.remove(store._PATH)
    except OSError:
        pass
    g = new_game()
    check(g.data["coins"] == 0 and g.data["unlocked"] == ["chicken"] and g.data["selected"] == "chicken",
          "frischer Stand: 0 Münzen, nur das Huhn")
    g.data["coins"] = 30
    i_uni = cw.CHAR_IDS.index("unicorn")
    check(not g._activate_char(i_uni) and "unicorn" not in g.data["unlocked"] and g.data["coins"] == 30,
          "zu wenig Münzen: Kauf abgelehnt")
    i_frog = cw.CHAR_IDS.index("frog")
    ok = g._activate_char(i_frog)
    check(ok and g.data["coins"] == 30 - cw.PRICES["frog"] and "frog" in g.data["unlocked"]
          and g.data["selected"] == "frog" and "crossy_char" in g.events,
          "Frosch gekauft, ausgewählt, Erfolg crossy_char ausgelöst")
    g2 = new_game()
    check(g2.data["coins"] == 30 - cw.PRICES["frog"] and g2.data["unlocked"] == ["chicken", "frog"]
          and g2.data["selected"] == "frog", "Kauf überlebt einen Neustart")
    g2._cycle_char(1)
    check(new_game().data["selected"] == "chicken", "Figur wechseln (Links/Rechts) wird gespeichert")
    # Tagesbestwert
    g = start(new_game("daily"), seedrand.daily_seed("crossy"))
    g.best = 12
    g._finish_run()
    g3 = new_game("daily")
    check(g3.data["daily"]["date"] == seedrand.today_str() and g3._daily_best() == 12 and g.record,
          "Tagesbestwert gespeichert (12, Rekord)")
    g = start(new_game("daily"), seedrand.daily_seed("crossy"))
    g.best = 7
    g._finish_run()
    check(new_game("daily")._daily_best() == 12 and not g.record, "schlechterer Lauf ändert den Tagesbestwert nicht")
    old = dict(store.load_section("crossy"))
    old["daily"] = {"date": "2000-01-01", "best": 99}
    store.save_section("crossy", old)
    check(new_game("daily")._daily_best() == 0, "Tagesbestwert von gestern zählt heute nicht")
    # Münzen einer abgebrochenen Runde
    g = start(new_game(), 1)
    g.run_coins = 3
    g.data["coins"] += 3
    g.on_exit()
    check(new_game().data["coins"] == old["coins"] + 3, "on_exit sichert Münzen einer laufenden Runde")
    # kaputte Daten
    store.save_section("crossy", {"coins": "viele", "unlocked": ["foo", "fox", 3],
                                  "selected": "robot", "daily": [1, 2]})
    g = new_game()
    check(g.data == {"coins": 0, "unlocked": ["chicken", "fox"], "selected": "chicken",
                     "daily": {"date": "", "best": 0}}, "kaputte Section wird bereinigt", str(g.data))
    merged = settings_mod._merge_defaults({"crossy": {"shadows": False, "daynight": False, "x": 1}})
    check(merged["crossy"] == {"shadows": False, "daynight": False},
          "settings: crossy.shadows/daynight überleben das Laden", str(merged.get("crossy")))
    g = new_game()
    g._toggle("daynight")
    check(g.daynight is False and GS["crossy"]["daynight"] is False, "Schalter Tag/Nacht speichert sofort")
    g._toggle("daynight")


# ------------------------------------------------------------------ Layout

def fits(game, text, maxw, fonts):
    return any(f.size(text)[0] <= maxw for f in fonts)


def inside(r, w, h, bottom_margin=0):
    return r.left >= 0 and r.top >= 0 and r.right <= w and r.bottom <= h - bottom_margin


def overlap(rects):
    for i, a in enumerate(rects):
        for b in rects[i + 1:]:
            if a.colliderect(b):
                return "%s/%s" % (a, b)
    return None


def audit_layout():
    print("\nLayout - 5 Auflösungen x 14 Sprachen (Setup, Figuren, Game Over, HUD)")
    lang_before = i18n.get_language()
    for w, h in RES:
        g = new_game(w=w, h=h)
        g.data["coins"] = 99999
        setup = g.tab_rects + [g.card_rect] + g.shadow_rects + g.night_rects + [g.start_rect]
        gap = g.shadow_rects[0].top - g.card_rect.bottom
        check(all(inside(r, w, h, 38) for r in setup) and overlap(setup) is None
              and gap >= g._tiny.get_height() + 6,
              "%4dx%d Setup: im Bild, ohne Überlappung, Platz für Beschriftung" % (w, h),
              "overlap=%s gap=%d" % (overlap(setup), gap))
        chars = g.grid_rects + [g.detail_rect]
        check(all(inside(r, w, h, 20) for r in chars) and overlap(chars) is None
              and g.detail_rect.contains(g.buy_rect),
              "%4dx%d Figuren: Raster + Detailzeile im Bild, ohne Überlappung" % (w, h))
        g._new_run()
        g._layout_over()
        pr = g.over_rect
        content_end = pr.y + 14 + g.over_rows_h
        check(inside(pr, w, h, 48) and all(pr.contains(rc) for _, rc in g.over_btns)
              and overlap([rc for _, rc in g.over_btns]) is None
              and content_end <= g.over_btns[0][1].top - 4,
              "%4dx%d Game-Over-Panel über dem Highscore-Banner, Zeilen über den Knöpfen" % (w, h),
              "Inhalt bis %d, Knöpfe ab %d" % (content_end, g.over_btns[0][1].top))
        too_long = []
        for code, _ in i18n.AVAILABLE:
            i18n.set_language(code, persist=False)
            T = i18n.t
            small = (g._small, g._tiny)
            tiny = (g._tiny,)
            tests = [
                (T("cr.tab_play"), g.tab_rects[0].w - 10, small),
                (T("cr.tab_chars"), g.tab_rects[1].w - 10, small),
                (T("cr.subtitle"), w - 30, small),
                (T("cr.chars_sub"), w - 30, small),
                (T("cr.lbl_shadows"), g.opt_w, tiny),
                (T("cr.lbl_daynight"), g.opt_w, tiny),
                (T("common.on"), g.shadow_rects[0].w - 10, small),
                (T("common.off"), g.shadow_rects[0].w - 10, small),
                (T("common.start"), g.start_rect.w - 10, (g.font,)),
                (T("cr.setup_hint"), w - 20, tiny),
                (T("cr.hint"), w - 20, tiny),
                (T("cr.chars_hint"), w - 20, tiny),
                (T("cr.over_hint"), pr.w - 28, tiny),
                (T("common.game_over"), pr.w - 28, (g._huge, g._small)),
                (T("common.points", score=9999), pr.w - 28, (g.font,)),
                (T("cr.best", n=9999) + "  ·  " + T("cr.new_record"), pr.w - 28, small),
                (T("cr.best_daily", n=9999) + "  ·  " + T("cr.new_record"), pr.w - 28, small),
                (T("cr.coins_run", n=99, total=99999), pr.w - 28, small),
                (T("cr.buy", n=250), g.buy_rect.w - 10, small),
                (T("cr.choose"), g.buy_rect.w - 10, small),
                (T("cr.chosen"), g.buy_rect.w - 10, small),
            ]
            tw = g.card_rect.right - 14 - (g.card_rect.x + g.card_rect.h + 6)
            tests += [(T("cr.mode.endless"), tw, (g._mid, g._small)),
                      (T("cr.mode.daily"), tw, (g._mid, g._small)),
                      (T("cr.mode_desc.endless"), tw, tiny),
                      (T("cr.mode_desc.daily", date="2026-09-16"), tw, tiny),
                      (T("cr.best_daily", n=9999), tw, small)]
            for key_, rc in g.over_btns:
                label = {"again": T("cr.btn_again"), "chars": T("cr.tab_chars").capitalize(),
                         "setup": T("cr.btn_setup")}[key_]
                tests.append((label, rc.w - 10, small))
            for cid in cw.CHAR_IDS:
                name = T("cr.char." + cid)
                tests.append((name, g.grid_rects[0].w - 6, tiny))
                tests.append((T("cr.char_line", name=name), tw, small))
            # HUD: Bestwert-Zeile links darf die Münzen rechts nicht erreichen
            pad = max(8, w // 60)
            coin_w = g._mid.size("99999")[0] + 2 * max(7, g._mid.get_height() // 2) + 16
            tests.append((T("cr.hud_top", n=9999), w - 2 * pad - coin_w, (g._small,)))
            # Tages-Kennzeichen oben mittig zwischen großer Punktzahl und Münzen
            side = max(pad + g._num.size("999")[0], coin_w + pad) + 12
            tests.append((T("cr.daily_tag", date="2026-09-16"), w - 2 * side - 18, tiny))
            tests.append((T("cr.hud_top_daily", n=9999), w - 2 * pad - coin_w, (g._small,)))
            for text, maxw, fonts in tests:
                if not fits(g, text, maxw, fonts):
                    too_long.append("%s:'%s'" % (code, text))
        i18n.set_language(lang_before, persist=False)
        check(not too_long, "%4dx%d alle Texte passen ohne Kürzung (14 Sprachen)" % (w, h),
              "; ".join(too_long))
    # Zeichnen: jede Sprache x Auflösung, alle Screens, beide Extrem-Themes
    errors = []
    for theme in ("v42", "v1"):
        ui.set_theme(theme)
        for w, h in RES:
            for code, _ in i18n.AVAILABLE:
                if theme == "v1" and code not in ("de", "fi"):
                    continue
                i18n.set_language(code, persist=False)
                try:
                    for mode in ("endless", "daily"):
                        g = new_game(mode, w, h)
                        g.data["unlocked"] = ["chicken", "ghost"]
                        g.data["selected"] = "ghost"
                        g.update(DT)
                        g.draw()
                        g._set_tab("chars")
                        g.draw()
                        g._set_tab("play")
                        start(g, 99)
                        place(g, 0, cw.START_COL, 0.0)
                        g.update(DT)
                        g.draw()
                        g._die("water")
                        run(g, 2.0)
                        g.draw()
                        check_over = g.game_over
                        if not check_over:
                            errors.append("%s %dx%d: kein Game Over" % (code, w, h))
                except Exception as exc:  # noqa: BLE001
                    errors.append("%s/%s %dx%d: %r" % (theme, code, w, h, exc))
    i18n.set_language(lang_before, persist=False)
    ui.set_theme("v42")
    check(not errors, "alle Screens zeichnen fehlerfrei (v4.2: 14 Sprachen, v1: de+fi)",
          "; ".join(errors[:4]))
    # Auflösungswechsel mitten im Spiel
    g = start(new_game(w=640, h=480), 3)
    place(g, 0, cw.START_COL, 0.0)
    g.update(DT)
    g.draw()
    g.surface = pygame.Surface((1280, 960))
    g.width, g.height = 1280, 960
    g.on_surface_changed()
    g.update(DT)
    g.draw()
    check(g.vox.tw == 1280 // 14 and g.start_rect.right <= 1280, "Auflösungswechsel im Spiel: neu aufgebaut")


# ------------------------------------------------------------------ Leistung

def audit_performance():
    print("\nLeistung - 1280x960")
    w, h = 1280, 960
    results = {}
    for label, row0 in (("Tag", 30), ("Nacht", 75)):
        g = start(new_game(w=w, h=h), 2024)
        r = find(g.world, lambda r, row: row["kind"] == cw.GRASS and not row["trees"][4], start=row0)
        place(g, r, 4, 3.0)
        g.best = r
        for _ in range(30):                   # Caches aufwärmen
            g.update(DT)
            g.draw()
        n = 240
        t0 = time.perf_counter()
        for i in range(n):
            g.update(DT)
            g.draw()
        ms = (time.perf_counter() - t0) * 1000.0 / n
        results[label] = ms
        check(ms < 10.0, "%s: %.2f ms je Frame (update + draw, Budget 60 FPS = 16,7 ms)" % (label, ms))
    return results


if __name__ == "__main__":
    t0 = time.time()
    wipe_mem()
    if REGISTERED is not None:
        check(REGISTERED, "CrossyRoadGame ist in games.ALL_GAMES registriert")
    audit_generator()
    audit_daily_and_js()
    audit_trains()
    audit_rules()
    audit_persistence()
    audit_layout()
    audit_performance()
    wipe_mem()
    print("\n%s  (%.1f s)" % ("ALLE PRÜFUNGEN BESTANDEN" if not FAILS
                              else "%d FEHLER: %s" % (len(FAILS), FAILS),
                              time.time() - t0))
    sys.exit(1 if FAILS else 0)
