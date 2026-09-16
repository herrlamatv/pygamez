# -*- coding: utf-8 -*-
"""Headless-Audit für Geometry Dash (Desktop + Abgleich mit der Web-Fassung).

Geprüft wird:

Kern      (1) Festkomma-Physik: Würfel springt/landet/prallt, Stachel, Grube,
              Pads bei Berührung, Orbs NUR auf Druck, blauer Orb/Pad dreht die
              Schwerkraft, Schiff/Ball/UFO/Welle, Form-/Schwerkraft-/Tempo-
              Portale samt Korridor, Münzen, Start "ab Block x";
              normalize_level räumt kaputte Daten auf, content_hash ignoriert
              Farben, aber keine Objekte.
Level     (2) 8 eingebaute Level (Leicht -> Dämon), JSON == Build-Skript ==
              web/js/games/geodash_levels.js; Solver-Beweis je Level: Ziel +
              alle 3 Münzen, auch mit -1/+1 Schritt Versatz, Entscheidungen im
              5-Schritt-Raster, Mindest-Drück-/Loslass-Dauer; der Solver löst
              ein frisches Stück Level selbst.
Web       (3) Python == JavaScript: web/tools/geodash_replay.js spielt die
              Lösungen und Zufalls-Eingaben (auch Test-Level mit allen
              Objektarten, Start ab Block x) nach - Zustands-Hashes gleich;
              Soundtrack-Noten gleich; Level-Datei Desktop -> Web -> Desktop.
Eingabe   (4) Akkumulator + Zeitstempel: echte Tastendrücke über update() bei
              15/30/60/144/50 FPS schaffen die Level; bei 15/30/60/120 FPS
              ist der Endzustand bitgleich mit dem Kern; Tastenwiederholung
              zählt nicht als Druck.
Spiel     (5) Tod -> Versuch +1 und Neustart, Bestwert %, [R]/[Q]/[P]/[Z]/[X],
              Übungsmodus mit Auto-Checkpoints (Neustart am Checkpoint, kein
              Stern), Sterne/Münzen/Erfolge gd_first/gd_coins/gd_demon, Score =
              Sterne gesamt (wächst nur), mem.json-Section "geodash" überlebt
              Neuladen und kaputte Daten, Settings-Section "geodash",
              Registrierung (ALL_GAMES, Erfolge, Wiki, Manifest).
Editor    (6) Setzen aller Objektarten, Drehen, Werte, Undo/Redo, Rechtsklick,
              Speichern (Fehlerfälle), Teilen als .lamapgzlevel und Import
              (id -2), "Verifiziert" nur nach eigenem Durchlauf vom Start und
              weg nach einer Änderung.
ugc       (7) ugc.py bleibt für Minigolf 100 % kompatibel zur Fassung aus git
              HEAD (Laden, Speichern, Löschen, Export, Import mit/ohne
              Umschlag, Dateiinhalt), Arten sind sauber getrennt.
Layout    (8) Levelauswahl, LEVELS-Reiter, Teilen-Dialog, Editor, Einstellungen,
              Ergebnis-Panel und HUD: 5 Auflösungen x 14 Sprachen im Bild, ohne
              Überlappung, Texte passen; alle Screens zeichnen fehlerfrei
              (v4.2: 14 Sprachen, v1: de+fi), Auflösungswechsel im Spiel.
Leistung  (9) 1280x960: Frame-Zeit (update + draw) über Lama Inferno und im
              Editor.

Aufruf aus dem Repo-Root:  python tests/audit_geodash.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import importlib.util
import json
import os
import random
import shutil
import subprocess
import sys
import tempfile
import time
import types

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except (AttributeError, OSError):
    pass

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, REPO)
sys.path.insert(0, os.path.join(REPO, "devtools"))

import pygame
pygame.init()
pygame.display.set_mode((640, 480))

import store
store._PATH = os.path.join(HERE, "_audit-geodash-mem.json")
import settings as settings_mod
settings_mod.save_settings = lambda s: None
import ugc
ugc._PATH = os.path.join(HERE, "_audit-geodash-ugc.json")
import filepick
import i18n
i18n.init()
import ui
import swear
from game_base import InputEvent

# Das games-Paket importiert ALLE Spiele. Laufen parallel Arbeiten an anderen
# Spielen, darf deren Zwischenstand diesen Audit nicht umwerfen - dann wird
# nur das Paket-Verzeichnis eingebunden.
REGISTERED = None
try:
    import games as _games
    REGISTERED = any(c.__name__ == "GeometryDashGame" for c in _games.ALL_GAMES)
except Exception as exc:          # noqa: BLE001
    print("  (Hinweis: games/__init__ nicht importierbar: %s)" % exc)
    for name in [m for m in sys.modules if m == "games" or m.startswith("games.")]:
        del sys.modules[name]
    _pkg = types.ModuleType("games")
    _pkg.__path__ = [os.path.join(REPO, "games")]
    sys.modules["games"] = _pkg
from games import geodash as gd
from games import geodash_core as core
from games import geodash_draw as gdraw
from games import geodash_edit as edit
from games import geodash_music as music

import build_geodash_levels as build
import build_geodash_solver as solver

GS = json.loads(json.dumps(settings_mod.DEFAULTS))
FAILS = []
RES = ((480, 360), (640, 480), (800, 600), (960, 720), (1280, 960))
B = core.B
TMP = tempfile.mkdtemp(prefix="audit-geodash-")
EXPORT_PATH = os.path.join(HERE, "_audit-geodash-export.lamapgzlevel")
PROOFS = json.load(open(os.path.join(REPO, "devtools", "geodash_proofs.json"), encoding="utf-8"))
LEVELS = gd.load_builtin_levels()


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + str(detail)) if detail and not ok else ""))
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


class FakeTime:
    """Ersetzt time in geodash.py: die Zeitstempel der Eingaben steuert der Test."""
    t = 0.0

    def perf_counter(self):
        return self.t


CLOCK = FakeTime()
gd.time = CLOCK


def new_game(mode="normal", w=640, h=480):
    g = quiet(gd.GeometryDashGame(pygame.Surface((w, h)), w, h, mode=mode,
                                  game_settings=json.loads(json.dumps(GS))))
    g.music_on = False
    return g


def key(g, k, down=True, repeat=False):
    g.handle_event(InputEvent(InputEvent.KEYDOWN if down else InputEvent.KEYUP,
                              key=k, repeat=repeat))


def click(g, pos, button=1, up=False):
    g.handle_event(InputEvent(InputEvent.MOUSEUP if up else InputEvent.MOUSEDOWN,
                              pos=pos, button=button))


def wipe_files():
    for path in (store._PATH, ugc._PATH, EXPORT_PATH):
        for suffix in ("", ".bak", ".tmp"):
            try:
                os.remove(path + suffix)
            except OSError:
                pass


def lvl(objs, length=80, **kw):
    d = {"objects": objs, "length": length}
    d.update(kw)
    return core.Level(d)


def run(lv, toggles=(), steps=None, start=None, until_x=None):
    """Spielt Umschalt-Schritte; merkt sich Extremwerte unterwegs."""
    s = core.new_state(lv, start)
    tg = sorted(toggles)
    ti, held, k = 0, False, 0
    info = {"max_y": s.y, "min_y": s.y, "landed_on": set(), "modes": set(), "gravs": set()}
    limit = steps if steps is not None else lv.end_x // core.SPEEDS[0] + 960
    while k < limit and not s.dead and not s.won:
        was = held
        while ti < len(tg) and tg[ti] <= k:
            held = not held
            ti += 1
        core.step(s, lv, held, held and not was)
        k += 1
        info["max_y"] = max(info["max_y"], s.y)
        info["min_y"] = min(info["min_y"], s.y)
        info["modes"].add(s.mode)
        info["gravs"].add(s.grav)
        if s.ground:
            info["landed_on"].add(s.y)
        if until_x is not None and s.x >= until_x * B:
            break
    return s, info


def step_at(xb):
    """Schritt, in dem der Würfel bei Tempo 1x die Blockposition xb erreicht."""
    return int(xb * B / core.SPEEDS[1])


# ====================================================================== Kern

def audit_core():
    print("\nKern - Festkomma-Physik")
    check(all(isinstance(v, int) for v in core.SPEEDS + core.GRAVITY + core.FALL_MAX)
          and isinstance(core.INNER_OFF, int) and isinstance(core.WAVE_OFF, int)
          and core.fdiv(-1, B) == -1 and core.fdiv(B, B) == 1,
          "nur Ganzzahlen, fdiv rundet nach unten")
    s, _ = run(lvl([]), steps=1000)
    check(not s.dead and s.y == 0 and s.ground == 1 and s.x == 1000 * core.SPEEDS[1],
          "Würfel läuft ohne Eingabe über flachen Boden")

    spike = lvl([["spike", 12, 0, 0]])
    dead, _ = run(spike)
    ok_press = None
    for k in range(step_at(8), step_at(12)):
        s, _ = run(spike, [k, k + 12])
        if s.won:
            ok_press = k
            break
    check(dead.dead and ok_press is not None,
          "Stachel tötet - ein Sprung zur rechten Zeit rettet (Schritt %s)" % ok_press)

    wall = lvl([["block", 12, 0, 0], ["block", 12, 1, 0], ["block", 12, 2, 0]])
    s, _ = run(wall, [0])                     # dauerhaft springen
    check(s.dead, "frontal gegen eine Blockwand = Tod")
    step_blk = lvl([["block", 12, 0, 0], ["block", 13, 0, 0], ["block", 14, 0, 0]])
    landed = None
    for k in range(step_at(8), step_at(12)):
        s, info = run(step_blk, [k, k + 12])
        if s.won and B in info["landed_on"]:
            landed = k
            break
    check(landed is not None, "Würfel landet auf einem Block und läuft weiter")

    pit = lvl([["pit", 12, 0, 0], ["pit", 13, 0, 0], ["pit", 14, 0, 0]])
    s_fall, _ = run(pit)
    jumped = any(run(pit, [k, k + 12])[0].won for k in range(step_at(9), step_at(12)))
    check(s_fall.dead and jumped, "Grube: ohne Sprung hineingefallen, mit Sprung darüber")

    pad = lvl([["pad_y", 12, 0, 0]])
    s, info = run(pad)
    check(not s.dead and info["max_y"] > 3 * B and 0 in s.used,
          "gelbes Pad wirkt bei Berührung (Höhe %.1f Blöcke)" % (info["max_y"] / B))
    pad_b = lvl([["pad_b", 12, 0, 0]], mode=core.BALL)
    s, info = run(pad_b, steps=step_at(20))
    check(-1 in info["gravs"], "blaues Pad dreht die Schwerkraft")

    orb = lvl([["orb_y", 12, 0, 0]])
    s_held, _ = run(orb, [0])                 # von Anfang an gehalten, kein neuer Druck
    press = step_at(11.5)
    s_press, info = run(orb, [press, press + 12])
    check(0 not in s_held.used and 0 in s_press.used and info["max_y"] > 2 * B,
          "Orb reagiert nur auf einen Druck, nicht aufs Halten")
    orb_b = lvl([["orb_b", 12, 0, 0]])
    s, info = run(orb_b, [press, press + 12], steps=step_at(16))
    check(-1 in info["gravs"], "blauer Orb dreht die Schwerkraft")

    ship = lvl([], mode=core.SHIP)
    s_up, _ = run(ship, [0], steps=240)
    s_dn, _ = run(ship, [], steps=240)
    check(s_up.y > s_dn.y and s_up.ceil == core.CORRIDOR and not s_up.dead,
          "Schiff: halten = steigen, loslassen = sinken, Korridor 10 Blöcke")
    ball = lvl([], mode=core.BALL)
    s, info = run(ball, [5, 20], steps=400)
    check(-1 in info["gravs"] and s.ground == 1 and s.y == s.ceil - B,
          "Ball: Tippen am Boden dreht die Schwerkraft, rollt an der Decke")
    ufo = lvl([], mode=core.UFO)
    s1, i1 = run(ufo, [5, 20], steps=100)
    s2, i2 = run(ufo, [5, 20, 60, 75], steps=140)
    check(i2["max_y"] > i1["max_y"], "UFO: zweiter Flügelschlag mitten in der Luft")
    wave = lvl([], mode=core.WAVE)
    s, _ = run(wave, [0], steps=200)
    check(s.y - core.new_state(wave).y > 0 and s.vy == core.SPEEDS[1],
          "Welle: halten = 45° nach oben")
    wave_blk = lvl([["block", 12, 0, 0]], mode=core.WAVE)
    s, _ = run(wave_blk)
    check(s.dead, "Welle: jede Blockberührung ist tödlich")

    portals = lvl([["p_ship", 5, 0, 0, 6], ["g_flip", 10, 1, 0], ["s_fast", 15, 5, 0],
                   ["coin", 20, 5, 0]], mode=core.CUBE)
    s, info = run(portals, [], until_x=22)
    check(s.mode == core.SHIP and s.ceil == 6 * B and -1 in info["gravs"] and s.speed == 2
          and s.coins == 1 and s.y == 5 * B and not s.dead,
          "Portale: Form + Korridorhöhe, Schwerkraft (Schiff steigt zur Decke), Tempo, Münze")
    coins = lvl([["coin", 10, 0, 0], ["coin", 14, 0, 0], ["coin", 18, 0, 0], ["coin", 22, 0, 0]])
    s, _ = run(coins)
    check(coins.coin_count == 3 and s.coins == 7, "Münzen: höchstens 3, jede ein Bit")
    s = core.new_state(portals, 17)
    check(s.mode == core.SHIP and s.speed == 2 and s.grav == -1 and s.x == 17 * B,
          "Start ab Block x: Portale davor gelten als durchflogen")

    d = core.normalize_level({"objects": [["spike", 3, 0, 7], ["pad_y", 2, 0, 1], ["nope", 1, 1],
                                          ["block", "4", 0], ["pit", 5, 9, 0], ["block", 4, 0, 0],
                                          ["color", 1, 0, 0, 1, 999, -5, 20], [1, 2, 3], "x",
                                          ["block", -1, 0, 0]],
                              "speed": 9, "mode": "x", "bg": "rot", "music": "jazz"})
    check(d["objects"] == [["block", 0, 0, 0], ["color", 1, 0, 0, 1, 255, 0, 20, 6], ["pad_y", 2, 0, 2],
                           ["spike", 3, 0, 3], ["block", 4, 0, 0], ["pit", 5, 0, 0]]
          and d["speed"] == 3 and d["mode"] == 0 and d["bg"] == list(core.DEFAULT_BG)
          and d["music"] == "drive",
          "normalize_level: aufräumen, begrenzen, doppelt weg, sortiert", d["objects"])
    base = {"objects": [["spike", 20, 0, 0]], "length": 60}
    h0 = core.content_hash(base)
    h_col = core.content_hash({"objects": base["objects"] + [["color", 5, 0, 0, 0, 1, 2, 3, 4]],
                               "length": 60, "bg": [1, 2, 3], "name": "x"})
    h_obj = core.content_hash({"objects": base["objects"] + [["spike", 25, 0, 0]], "length": 60})
    check(h0 == h_col and h0 != h_obj, "content_hash: Farben egal, Objekte nicht")


# ====================================================================== Level

def _toggle_ok(tg):
    if tg != sorted(tg) or any(t % solver.PERIOD for t in tg):
        return False
    for a, b in zip(tg, tg[1:]):
        if b - a < min(solver.MIN_HOLD, solver.MIN_RELEASE):
            return False
    return True


def audit_levels():
    print("\nEingebaute Level + Solver-Beweis")
    built = [fn() for fn in build.LEVELS]
    raw = json.load(open(gd.LEVELS_FILE, encoding="utf-8"))["levels"]
    check(len(LEVELS) == 8 and json.dumps(raw, sort_keys=True) == json.dumps(built, sort_keys=True),
          "8 Level, games/levels/geodash.json == Build-Skript")
    diffs = [d["difficulty"] for d in LEVELS]
    check(diffs == sorted(diffs) and diffs[0] == 0 and diffs[-1] == 5
          and len({d["id"] for d in LEVELS}) == 8
          and sum(d["stars"] + 3 for d in LEVELS) == 65,
          "Schwierigkeit steigt (Leicht -> Dämon), ids eindeutig, 65 Sterne möglich")
    js = open(os.path.join(REPO, "web", "js", "games", "geodash_levels.js"), encoding="utf-8").read()
    body = js[js.index("window.PG.gdLevels = ") + len("window.PG.gdLevels = "):js.rindex(";\n})();")]
    check(json.dumps(json.loads(body), sort_keys=True) == json.dumps(raw, sort_keys=True),
          "web/js/games/geodash_levels.js == games/levels/geodash.json")
    for d in LEVELS:
        lv = core.Level(d)
        pr = PROOFS.get(d["id"])
        if not check(pr is not None, "%s: Lösung vorhanden" % d["id"]):
            continue
        tg = pr["toggles"]
        oks = [solver.verify(lv, tg, sh)[0] for sh in (-1, 0, 1)]
        s, _ = core.run_toggles(lv, tg)
        check(all(oks) and lv.coin_count == 3 and pr["content"] == core.content_hash(d)
              and pr["hash"] == core.state_hash(s) and _toggle_ok(tg),
              "%-13s Ziel + 3 Münzen bei -1/0/+1 Schritt, %3d Drücke, %.1f s, Raster/Mindestdauer ok"
              % (d["id"], len(tg) // 2, s.step / float(core.HZ)), oks)
    # Der Solver selbst: ein Stück Level ohne gespeicherte Lösung
    piece = dict(built[1])
    piece["objects"] = [o for o in built[1]["objects"] if o[1] < 90]
    piece["length"] = 96
    t0 = time.time()
    tg, info = solver.solve(piece, need_coins=True, max_nodes=200000, time_limit=60)
    ok = tg is not None and all(solver.verify(piece, tg, sh)[0] for sh in (-1, 0, 1))
    check(ok, "Solver löst ein frisches Level-Stück robust (%.1f s)" % (time.time() - t0), info)


# ====================================================================== Web

def node(args, cwd=REPO, stdin=None):
    return subprocess.run(["node"] + args, cwd=cwd, capture_output=True, text=True,
                          encoding="utf-8", timeout=300, input=stdin)


def sink_level():
    """Test-Level mit allen Objektarten (auch gedreht, Portale mit Höhe)."""
    objs = []
    x = 8
    for kind in core.KIND_NAMES:
        if kind == "color":
            objs.append(["color", x, 0, 0, 1, 200, 40, 90, 12])
        elif kind in core.PARAMS:
            objs.append([kind, x, 1, 0, 7])
        else:
            objs.append([kind, x, 0 if kind in ("pit", "pad_y", "pad_p", "pad_b") else 1,
                         2 if kind in ("spike_s", "half") else 0])
        x += 6
    objs += [["block", 30, 2, 0], ["spike", 31, 4, 3], ["half", 40, 3, 1], ["orb_p", 55, 3, 0]]
    return {"id": "sink", "name": "Sink", "objects": objs, "length": x + 20, "speed": 1}


def audit_web():
    print("\nPython == JavaScript (node web/tools/geodash_replay.js)")
    try:
        res = node([os.path.join("web", "tools", "geodash_replay.js")])
    except (OSError, subprocess.SubprocessError) as exc:
        check(False, "node startet", exc)
        return
    try:
        out = json.loads(res.stdout)
    except ValueError:
        check(False, "Replay liefert JSON", res.stderr[-400:])
        return
    bad = []
    for d in LEVELS:
        r = out.get(d["id"], {})
        lv = core.Level(d)
        s, trace = core.run_toggles(lv, PROOFS[d["id"]]["toggles"], trace_every=500)
        if not (r.get("ok") and r.get("state") == core.state_str(s)
                and [list(t) for t in trace] == r.get("trace")
                and r.get("contentHash") == core.content_hash(d)):
            bad.append(d["id"])
    check(res.returncode == 0 and not bad,
          "8 Lösungen: Zwischen-Hashes alle 500 Schritte + Endzustand identisch", bad)
    # Eigene Fälle: Zufalls-Eingaben auf allen Leveln, Test-Level, Start ab Block x
    rng = random.Random(7)
    cases = []
    for d in LEVELS:
        for n in range(3):
            tg, t = [], 0
            while t < 9000:
                t += rng.randint(1, 90)
                tg.append(t)
            start = None if n < 2 else rng.randint(5, core.level_length(d) - 20)
            cases.append({"id": "%s-%d" % (d["id"], n), "level": d, "toggles": tg,
                          "every": 37, "start": start})
    sink = sink_level()
    for n in range(12):
        tg, t = [], rng.randint(0, 30)
        while t < 6000:
            tg.append(t)
            t += rng.randint(3, 60)
        start = None if n % 3 else rng.randint(3, 40)
        for mode in range(5):
            cases.append({"id": "sink-%d-%d" % (n, mode),
                          "level": dict(sink, mode=mode, speed=n % 4), "toggles": tg,
                          "every": 11, "start": start})
    path = os.path.join(TMP, "cases.json")
    with open(path, "w", encoding="utf-8") as f:
        json.dump(cases, f)
    res = node([os.path.join("web", "tools", "geodash_replay.js"), "--cases", path])
    try:
        out = json.loads(res.stdout)
    except ValueError:
        check(False, "Replay mit eigenen Fällen liefert JSON", res.stderr[-400:])
        return
    bad = []
    kinds_used = set()
    for c in cases:
        lv = core.Level(c["level"])
        st = core.new_state(lv, c["start"]) if c["start"] else None
        s, trace = core.run_toggles(lv, c["toggles"], trace_every=c["every"], start=st)
        r = out.get(c["id"], {})
        if c["id"].startswith("sink"):
            kinds_used.update(lv.kind[i] for i in s.used)
        if r.get("state") != core.state_str(s) or r.get("trace") != [list(t) for t in trace] \
                or r.get("progress") != core.progress(s, lv):
            bad.append(c["id"])
    check(not bad, "%d Fälle (Zufalls-Eingaben, alle Objektarten, alle Formen/Tempi, Start ab x): "
                   "jeder Zwischen-Hash gleich" % len(cases), bad[:5])
    check(len(kinds_used) >= 8, "Testfälle lösen viele Objektarten wirklich aus (%d)" % len(kinds_used))

    # Soundtrack: gleiche Noten
    script = """
global.window = {PG: {}}; global.PG = window.PG;
require(%s); require(%s);
const out = {};
for (const [id, style, bpm] of JSON.parse(process.argv[2])) out[id] = PG.gdMusic.compose(style, bpm, id);
process.stdout.write(JSON.stringify(out));
""" % (json.dumps(os.path.join(REPO, "web", "js", "games", "seedrand.js")),
       json.dumps(os.path.join(REPO, "web", "js", "games", "geodash_music.js")))
    spec = [[d["id"], d["music"], music.style_bpm(d)] for d in LEVELS] + [["x", "dark", 0]]
    js_path = os.path.join(TMP, "music.js")
    with open(js_path, "w", encoding="utf-8") as f:
        f.write(script)
    res = node([js_path, json.dumps(spec)])
    try:
        out = json.loads(res.stdout)
    except ValueError:
        out = {}
    bad = []
    for sid, style, bpm in spec:
        py = music.compose(style, bpm, sid)
        jsn = out.get(sid)
        if not jsn or abs(py["length"] - jsn["length"]) > 1e-9 or len(py["voices"]) != len(jsn["voices"]):
            bad.append(sid)
            continue
        for (pv, pvol), (jv, jvol) in zip(py["voices"], jsn["voices"]):
            if len(pv) != len(jv) or pvol != jvol:
                bad.append(sid)
                break
            for a, b in zip(pv, jv):
                if a[3] != b[3] or abs(a[0] - b[0]) > 1e-9 or abs(a[1] - b[1]) > 1e-6 * a[1] \
                        or abs(a[2] - b[2]) > 1e-9 or a[4] != b[4] or a[5] != b[5]:
                    bad.append(sid)
                    break
    check(not bad and len(out) == len(spec), "Soundtrack: gleiche Noten am PC und im Browser", bad)

    # Teilen über Plattformgrenzen: Desktop-Export -> Web-Import -> Web-Export -> Desktop-Import
    wipe_files()
    lv = dict(sink_level(), id="grenzgaenger", name="Grenzgänger", author="Lama")
    lv["verified"] = core.content_hash(lv)
    src = os.path.join(TMP, "desk.lamapgzlevel")
    ok1, _ = ugc.export_to(core.normalize_level(lv), src, "geodash")
    langs = json.dumps([[c, n] for c, n in i18n.AVAILABLE])
    script = """
const fs = require("fs");
const mem = {};
global.window = {PG: {LANGS: %s, store: {get: (k, d) => (k in mem ? JSON.parse(mem[k]) : d), set: (k, v) => { mem[k] = JSON.stringify(v); }}}};
global.PG = window.PG;
require(%s); require(%s);
const text = fs.readFileSync(process.argv[2], "utf8");
const [ok, why, m] = PG.ugc.importText(text, "geodash");
const wrong = PG.ugc.importText(text, "minigolf");
const again = PG.ugc.importText(text, "geodash");
const [eok, ewhy, etext] = PG.ugc.exportText(PG.ugc.get(m.id, "geodash"), "geodash");
fs.writeFileSync(process.argv[3], etext);
process.stdout.write(JSON.stringify({ok, why, id: m && m.id, wrong: wrong[1], again: again[2] && again[2].id,
  hash: PG.gdCore.contentHash(m), eok, n: PG.ugc.count("geodash")}));
""" % (langs, json.dumps(os.path.join(REPO, "web", "js", "games", "geodash_core.js")),
       json.dumps(os.path.join(REPO, "web", "js", "games", "ugc.js")))
    js_path = os.path.join(TMP, "share.js")
    with open(js_path, "w", encoding="utf-8") as f:
        f.write(script)
    back = os.path.join(TMP, "web.lamapgzlevel")
    res = node([js_path, src, back])
    try:
        out = json.loads(res.stdout)
    except ValueError:
        out = {}
    ok2, why2, m2 = (False, "io", None)
    if os.path.exists(back):
        ok2, why2, m2 = ugc.import_from(back, "geodash")
    check(ok1 and out.get("ok") and out.get("wrong") == "format" and out.get("again") == "grenzgaenger-2"
          and out.get("hash") == lv["verified"] and ok2 and m2["objects"] == core.normalize_level(lv)["objects"]
          and edit.is_verified(m2) and m2["name"] == "Grenzgänger",
          "Level-Datei Desktop -> Browser -> Desktop: Objekte, Name und Verifiziert bleiben",
          (out, why2, res.stderr[-300:]))
    wipe_files()


# ====================================================================== Eingabe

def press_play(g, toggles, fps, keyname="space", limit_s=90.0, jitter=None):
    """Spielt eine Lösung mit echten Tasten-Events über update() ab.

    Die Events tragen ihre echte Uhrzeit (Mitte des Soll-Schritts), update()
    bekommt die vergangene Zeit - bei 'jitter' (random.Random) schwankt die
    Frame-Dauer zufällig zwischen 5 und 80 ms.
    """
    CLOCK.t = 0.0
    g._last_update = 0.0
    t_prev, ti = 0.0, 0
    down = True
    while t_prev < limit_s and g.state == gd.PLAY:
        dt = jitter.uniform(0.005, 0.08) if jitter else 1.0 / fps
        t_now = t_prev + dt
        while ti < len(toggles) and (toggles[ti] + 0.5) / core.HZ <= t_now:
            CLOCK.t = (toggles[ti] + 0.5) / core.HZ
            key(g, keyname, down=down)
            down = not down
            ti += 1
        CLOCK.t = t_now
        g.update(dt)
        t_prev = t_now
        if g.run is not None and g.run.dead:
            return "dead"
    return "won" if g.state == gd.COMPLETE else "timeout"


def audit_input():
    print("\nEingabe - Akkumulator und Zeitstempel")
    for idx in (0, 7):
        d = LEVELS[idx]
        tg = PROOFS[d["id"]]["toggles"]
        for fps in (15, 30, 60, 120, 144, 50, 37, "jitter"):
            g = new_game()
            g.start_level(d, "main")
            res = press_play(g, tg, 60 if fps == "jitter" else fps,
                             jitter=random.Random(5) if fps == "jitter" else None)
            same = core.state_hash(g.run) == PROOFS[d["id"]]["hash"]
            check(res == "won" and g.run.coins == 7 and same,
                  "%-13s %6s FPS: geschafft mit allen Münzen, Endzustand bitgleich mit dem Kern"
                  % (d["id"], fps), (res, same))
    # Tastenwiederholung zählt nicht als neuer Druck (Orb wird nicht ausgelöst)
    g = new_game()
    g.start_level({"id": "rep", "objects": [["orb_y", 12, 0, 0]], "length": 40}, "main")
    CLOCK.t = 0.0
    g._last_update = 0.0
    key(g, "space")
    frames = 0
    while g.run.x < 13 * B and frames < 400:
        CLOCK.t += 1 / 60.0
        key(g, "space", repeat=True)
        g.update(1 / 60.0)
        frames += 1
    check(0 not in g.run.used and g.session_jumps == 1 and not g.run.dead,
          "Tastenwiederholung löst keinen Orb aus, zählt keinen Sprung", (g.run.used, g.session_jumps))
    # Maus und Taste gleichzeitig: Loslassen einer Quelle lässt die andere gehalten
    g = new_game()
    g.start_level({"id": "src", "objects": [], "length": 60, "mode": core.SHIP}, "main")
    CLOCK.t = 0.0
    g._last_update = 0.0
    key(g, "space")
    click(g, (10, 10))
    key(g, "space", down=False)
    for _ in range(60):
        CLOCK.t += 1 / 60.0
        g.update(1 / 60.0)
    check(g.sources == {"mouse"} and g.run.y > core.new_state(g.lv).y,
          "Maus + Taste: Loslassen der Taste hält die Maus gedrückt")


# ====================================================================== Spiel

def frames(g, n, dt=1 / 60.0):
    for _ in range(n):
        CLOCK.t += dt
        g.update(dt)


def until_alive(g, dt=1 / 60.0):
    """Frames bis zum nächsten Versuch (bleibt direkt nach dem Neustart stehen)."""
    for _ in range(200):
        if not g.run.dead:
            return
        CLOCK.t += dt
        g.update(dt)


def audit_game():
    print("\nSpielablauf, Fortschritt, Speicher")
    wipe_files()
    g = new_game()
    check(g.state == gd.SELECT and g.score == 0 and g.total_stars() == 0 and g.max_stars() == 65,
          "Start: Levelauswahl, 0 von 65 Sternen")
    key(g, "Right")
    key(g, "3")
    check(g.sel == 2 and g.settings["geodash"]["last_level"] == 2, "Levelauswahl: Pfeile/Ziffern, gemerkt")
    key(g, "Left")
    key(g, "Left")
    key(g, "Return")
    check(g.state == gd.PLAY and g.cur["id"] == "lama-launch" and g.attempts["lama-launch"] == 1,
          "Enter startet das Level, Versuch 1")
    CLOCK.t = 0.0
    g._last_update = 0.0
    frames(g, 150)
    check(g.run.dead and g.dead_t >= 0, "ohne Eingabe: Tod am ersten Stachel")
    pct = core.progress(g.run, g.lv)
    until_alive(g)
    check(not g.run.dead and g.run.x < B and g.attempts["lama-launch"] == 2
          and g.best["lama-launch"][0] == pct and pct > 0,
          "nach der Explosion: sofort Versuch 2, Bestwert %d%% gemerkt" % pct)
    frames(g, 30)
    key(g, "r")
    check(g.run.x == 0 and g.attempts["lama-launch"] == 3, "[R] startet sofort neu")
    key(g, "q")
    check(g.state == gd.SELECT and g.cur is None, "[Q] führt zurück zur Levelauswahl")

    # Übungsmodus: Auto-Checkpoints, Neustart am Checkpoint, kein Stern
    g.practice = True
    g.start_level(LEVELS[0], "main")
    tg = PROOFS["lama-launch"]["toggles"]
    cut = [t for t in tg if t < 6 * core.HZ]
    if len(cut) % 2:
        cut = cut[:-1]
    CLOCK.t = 0.0
    g._last_update = 0.0
    press_play(g, cut, 60, limit_s=12)
    cps = len(g.checkpoints)
    cp_x = g.checkpoints[-1].x if cps else -1
    check(cps >= 2 and g.run.dead, "Übung: Auto-Checkpoints etwa alle 2 s (%d)" % cps)
    until_alive(g)
    check(g.run.x == cp_x and not g.run.dead and g.dead_t < 0,
          "Übung: nach dem Tod geht es am letzten Checkpoint weiter")
    key(g, "z")
    n = len(g.checkpoints)
    key(g, "x")
    check(len(g.checkpoints) == n - 1 and n == cps + 1, "[Z] setzt, [X] löscht einen Checkpoint")
    key(g, "p")
    check(not g.practice and g.run.x == 0 and not g.checkpoints, "[P] beendet die Übung: normaler Versuch vom Start")
    key(g, "p")
    g.practice = True
    g.start_level(LEVELS[0], "main")
    press_play(g, tg, 60)
    check(g.state == gd.COMPLETE and g.complete_info["practice"] and g.complete_info["stars"] == 0
          and g.best["lama-launch"][1] == 100 and g.best["lama-launch"][0] < 100
          and g.coins.get("lama-launch", 0) == 0 and not g.events and g.total_stars() == 0,
          "Übung geschafft: Übungs-Bestwert 100 %, aber keine Sterne/Münzen/Erfolge")

    # Normal geschafft: Sterne, Münzen, Erfolge
    g._complete_action("select")
    g.practice = False
    g.start_level(LEVELS[0], "main")
    press_play(g, tg, 60)
    info = g.complete_info
    check(g.state == gd.COMPLETE and info["stars"] == 1 and info["new_coins"] == 3
          and g.score == 4 and g.total_stars() == 4 and "gd_first" in g.events
          and "gd_coins" in g.events and "gd_demon" not in g.events,
          "Normal geschafft: 1 Stern + 3 Münzen = Score 4, Erfolge gd_first + gd_coins", (info, g.events))
    g.events.clear()
    g._complete_action("again")
    press_play(g, tg, 60)
    check(g.complete_info["stars"] == 0 and g.complete_info["new_coins"] == 0 and g.score == 4
          and "gd_first" not in g.events, "zweiter Sieg: keine doppelten Sterne, Score wächst nicht")
    g._complete_action("next")
    check(g.state == gd.PLAY and g.cur["id"] == "neon-steps" and g.sel == 1, "\"Nächstes\" startet Level 2")
    g.start_level(LEVELS[7], "main")
    press_play(g, PROOFS["lama-inferno"]["toggles"], 60)
    check("gd_demon" in g.events and g.score == 4 + 10 + 3, "Lama Inferno: gd_demon, Score 17")
    g.on_exit()
    g2 = new_game()
    check(g2.score == 17 and g2.best["lama-inferno"] == [100, 0] and g2.coins["lama-launch"] == 7
          and g2.attempts["lama-launch"] >= 5 and g2.jumps > 0,
          "mem.json-Section geodash: Bestwerte, Münzen, Versuche überleben Neuladen")
    data = store.load_section("geodash")
    check(set(data) >= {"best", "coins", "attempts", "jumps"}, "Section enthält best/coins/attempts/jumps")
    store.save_section("geodash", {"best": {"a": "x", "lama-launch": [300, -4]}, "coins": {"b": "y", "c": 99},
                                   "attempts": [], "jumps": "viele"})
    g3 = new_game()
    check(g3.best == {"lama-launch": [100, 0]} and g3.coins == {"c": 3} and g3.attempts == {} and g3.jumps == 0,
          "kaputte Daten werden bereinigt statt abzustürzen")
    wipe_files()

    # Settings-Section und Optionen
    g = new_game()
    g.music_on = True
    for k in ("music", "bar", "auto"):
        g._toggle_option(k)
    gs = g.settings["geodash"]
    merged = settings_mod._merge_defaults({"geodash": {"music": False, "progress_bar": 0, "grid": False,
                                                       "last_level": 99, "auto_checkpoints": False}})
    check(gs["music"] is False and gs["progress_bar"] is False and gs["auto_checkpoints"] is False
          and merged["geodash"] == {"music": False, "auto_checkpoints": False, "progress_bar": True,
                                    "grid": False, "last_level": 7},
          "Optionen landen in settings.geodash, Prüfregeln greifen", merged.get("geodash"))
    check(g.wants_escape is False and (g._set_tab("levels") or g.wants_escape),
          "ESC: Levelauswahl pausiert, LEVELS-Reiter/Editor/Test/Ergebnis heißen \"zurück\"")
    g._set_tab("play")

    # Registrierung
    import achievements
    ids = {s[0] for s in achievements.SPECIALS}
    de = json.load(open(os.path.join(REPO, "lang", "de.json"), encoding="utf-8"))
    check((REGISTERED is None or REGISTERED) and gd.GeometryDashGame.highscore_key == "geodash"
          and len(gd.GeometryDashGame.MODES) <= 4 and gd.GeometryDashGame.wants_right_click
          and achievements.MILESTONES.get("geodash") == 20 and {"gd_first", "gd_coins", "gd_demon"} <= ids
          and all(("ach.%s.%s" % (a, f)) in de for a in ("gd_first", "gd_coins", "gd_demon")
                  for f in ("name", "desc")),
          "registriert: ALL_GAMES, highscore_key, Modi, Rechtsklick, Erfolge + Texte")
    wiki_ok = []
    for code, _ in i18n.AVAILABLE:
        sub = "" if code in ("de", "en", "fr", "es", "pt") else "lang.expansion"
        pages = json.load(open(os.path.join(REPO, "lamawiki", sub, code + ".json"), encoding="utf-8"))["pages"]
        ids_ = [p["id"] for p in pages]
        page = next((p for p in pages if p["id"] == "geodash"), None)
        wiki_ok.append(page is not None and page.get("game") == "GeometryDashGame"
                       and "geodash-levels" in ids_
                       and ids_.index("geodash") == ids_.index("crossyroad") + 1
                       and ids_.index("geodash-levels") == ids_.index("geodash") + 1)
    check(all(wiki_ok), "LamaWiki: Seiten geodash + geodash-levels in allen 14 Sprachen")
    man = open(os.path.join(REPO, "web", "js", "manifest.js"), encoding="utf-8").read()
    line = next((ln for ln in man.splitlines() if '"GeometryDashGame"' in ln), "")
    need = ["seedrand.js", "ugc.js", "geodash_core.js", "geodash_levels.js", "geodash_draw.js",
            "geodash_music.js", "geodash_edit.js", "geodash.js"]
    pos = [line.find("js/games/" + n) for n in need]
    golf = next((ln for ln in man.splitlines() if '"MiniGolfGame"' in ln), "")
    check(all(p >= 0 for p in pos) and pos == sorted(pos)
          and 0 <= golf.find("ugc.js") < golf.find("minigolf_edit.js"),
          "web/js/manifest.js lädt alle Geometry-Dash-Module (und ugc.js vor dem Minigolf-Editor)")

    # Musik
    t0 = time.perf_counter()
    voices = music.build_voices("dark", 174, "lama-inferno")
    ms = (time.perf_counter() - t0) * 1000
    song = music.compose("dark", 174, "lama-inferno")
    pulses = [music.beat_pulse(t / 100.0, 120) for t in range(200)]
    check(voices and len(voices) == 4 and len({len(v[0]) for v in voices}) == 1
          and music.compose("none", 0, "x") is None
          and song == music.compose("dark", 174, "lama-inferno")
          and all(0.0 <= p <= 1.0 for p in pulses) and pulses[0] == 1.0 and pulses[50] == 1.0,
          "Soundtrack: 4 gleich lange Stimmen, deterministisch, Beat-Puls (%.0f ms)" % ms)


# ====================================================================== Editor

def audit_editor():
    print("\nLevel-Editor, Teilen, Verifiziert")
    wipe_files()
    g = new_game(w=800, h=600)
    g._set_tab("levels")
    g.lists.handle(InputEvent(InputEvent.KEYDOWN, key="n"))
    ed = g.editor
    check(g.state == gd.EDIT and ed is not None and g.wants_escape, "LEVELS-Reiter: [N] öffnet den Editor")
    x = 6
    for group, kinds in edit.GROUPS:
        ed._pick_group(group)
        for kind in kinds:
            ed.tool = kind
            ed._place((x, 0 if kind in ("pit",) else 1))
            x += 3
    names = sorted(o[0] for o in ed.objects)
    check(names == sorted(core.KIND_NAMES), "jede Objektart lässt sich setzen (%d)" % len(names))
    ed.tool = "coin"
    for i in range(3):
        ed._place((x + 3 * i, 2))
    check(sum(1 for o in ed.objects if o[0] == "coin") == 3, "höchstens drei Münzen")
    ed.tool = "spike"
    ed._place((200, 1))
    ed._rotate()
    check(ed.objects[ed.sel][3] == 1, "[R] dreht das gewählte Objekt")
    idx = next(i for i, o in enumerate(ed.objects) if o[0] == "color")
    ed.sel = idx
    rects = ed._param_rects()
    ed._param_click(rects[1][1].center)          # R -15
    ed._param_click(rects[4][3].center)          # Dauer +1
    o = ed.objects[idx]
    check(len(rects) == 5 and o[5] == 240 and o[8] == 7, "Farb-Trigger: Werte per +/-", o)
    ed._undo()
    ed._undo()
    check(ed.objects[idx][5] == 255 and ed.objects[idx][8] == 6, "Undo nimmt Werte zurück")
    ed._redo()
    check(ed.objects[idx][5] == 240 and ed.objects[idx][8] == 6 and ed.undo_stack and ed.redo_stack,
          "Redo stellt Schritt für Schritt wieder her")
    ed.sel = None
    ed.tool = "block"
    before = len(ed.objects)
    ed.cam_x, ed.cam_y = 0.0, -1.0
    cell_px = (ed.canvas.x + int((6.5 - ed.cam_x) * ed.ts), ed.canvas.bottom - int((1.5 - ed.cam_y) * ed.ts))
    check(ed.cell_at(cell_px) == (6, 1), "Zeiger -> Zelle", ed.cell_at(cell_px))
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=cell_px, button=3))
    g.handle_event(InputEvent(InputEvent.MOUSEUP, pos=cell_px, button=3))
    check(len(ed.objects) == before - 1, "Rechtsklick ohne Ziehen löscht das Objekt darunter")
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=cell_px, button=3))
    g.handle_event(InputEvent(InputEvent.MOUSEMOVE, pos=(cell_px[0] - 5 * ed.ts, cell_px[1])))
    g.handle_event(InputEvent(InputEvent.MOUSEUP, pos=(cell_px[0] - 5 * ed.ts, cell_px[1]), button=3))
    check(len(ed.objects) == before - 1 and ed.cam_x >= 4.0, "Rechts ziehen scrollt statt zu löschen")
    check(not ed._save() and ed.err, "Speichern ohne Namen/id wird abgelehnt")
    ed.f_name.set_text("Arschloch Level")
    check(not ed._save(), "Wortfilter beim Speichern")
    ed.f_name.set_text("Audit Level")
    ed.f_id.set_text("")
    check(ed._save() and not ed.dirty and ugc.get("audit-level", "geodash") is not None,
          "Speichern: id aus dem Namen, landet in ugc.json (Art geodash)")
    saved = ugc.get("audit-level", "geodash")
    check(saved["objects"] == ed.level_dict()["objects"] and ugc.load_maps("minigolf") == [],
          "gespeicherte Objekte == Editor, Minigolf-Sammlung unberührt")
    key(g, "Escape")
    check(g.state == gd.SELECT and g.setup_tab == "levels" and len(g.lists.items) == 1,
          "ESC ohne Änderungen: zurück in den LEVELS-Reiter")

    # Teilen + Import
    lists = g.lists
    real_dl = filepick.to_downloads
    filepick.to_downloads = lambda name: EXPORT_PATH
    try:
        lists._action("share")
        lists.f_author.set_text("Lama")
        lists._do_export(ask=False)
    finally:
        filepick.to_downloads = real_dl
    raw = json.load(open(EXPORT_PATH, encoding="utf-8")) if os.path.exists(EXPORT_PATH) else {}
    check(raw.get("format") == "pygamez.geodash.level" and raw.get("level", {}).get("author") == "Lama"
          and lists.share is None, "Teilen: .lamapgzlevel mit Umschlag pygamez.geodash.level")
    check(lists.import_path(EXPORT_PATH) and [m["id"] for m in ugc.load_maps("geodash")] == ["audit-level", "audit-level-2"]
          and ugc.get("audit-level-2", "geodash")["objects"] == saved["objects"],
          "Import: gleiche Objekte, vergebene id wird zu -2")
    golf_file = os.path.join(TMP, "golf.lamapgzmap")
    with open(golf_file, "w", encoding="utf-8") as f:
        json.dump({"format": "pygamez.minigolf.map", "map": {"id": "x", "name": "x", "tee": [1, 2], "cup": [3, 4]}}, f)
    check(ugc.import_from(golf_file, "geodash")[:2] == (False, "format") and not lists.import_path(golf_file),
          "eine Minigolf-Bahn lässt sich nicht als Level importieren")
    lists.sel = 0
    lists._action("delete")
    lists._action("delete")
    check([m["id"] for m in ugc.load_maps("geodash")] == ["audit-level-2"], "Löschen mit Bestätigung")

    # Verifiziert: nur nach eigenem Durchlauf vom Start
    tg = PROOFS["lama-launch"]["toggles"]
    piece = dict(LEVELS[0])
    piece["objects"] = [list(o) for o in LEVELS[0]["objects"] if o[1] < 60]
    piece["length"] = 70
    m = edit.new_level("Kurz", "kurz", "Lama")
    m["objects"] = piece["objects"]
    m["length"] = 70
    check(ugc.save_map(core.normalize_level(m), "geodash")[0], "Test-Level gespeichert")
    g._set_tab("levels")
    g.lists.reload()
    g.ugc_edit(ugc.get("kurz", "geodash"))
    ed = g.editor
    ed.cam_x = 20.0
    ed._test(True)                                   # "Ab hier"
    check(g.state == gd.PLAY and g.start_block and not g._test["verify"] and g.wants_escape,
          "\"Ab hier\" startet mitten im Level (verifiziert nicht)")
    key(g, "Escape")
    check(g.state == gd.EDIT and g.run is None, "ESC im Test: zurück an die Leinwand")
    ed._test(False)
    res = press_play(g, tg, 60)
    stored = ugc.get("kurz", "geodash")
    check(res == "won" and g.complete_info["verified"] and edit.is_verified(stored)
          and stored.get("verified") == core.content_hash(stored),
          "Test vom Start geschafft: Level ist verifiziert und gespeichert", res)
    g._complete_action("editor")
    check(g.state == gd.EDIT and not g.best.get("ugc:kurz"), "Test zählt nicht als Bestwert")
    ed.tool = "spike"
    ed._place((65, 0))
    check(not edit.is_verified(ed.level_dict()), "jede Änderung nimmt das Abzeichen wieder weg")
    ed._undo()
    check(edit.is_verified(ed.level_dict()), "... und Rückgängig gibt es zurück")
    ed.tool = "color"
    ed._place((30, 0))
    check(edit.is_verified(ed.level_dict()), "Farb-Trigger ändern nichts am Abzeichen")
    ed._undo()
    ed._place((66, 0))
    key(g, "Escape")
    check(g.state == gd.EDIT and ed.confirm_back, "ESC mit Änderungen: erst nachfragen")
    key(g, "Escape")
    check(g.state == gd.SELECT, "... zweites ESC verwirft")
    g.ugc_play(ugc.get("kurz", "geodash"))
    press_play(g, tg, 60)
    check(g.best.get("ugc:kurz") == [100, 0] and g.score == 0,
          "eigene Level: eigener Bestwert, aber keine Sterne")
    g._complete_action("select")
    check(g.setup_tab == "levels", "nach eigenem Level zurück in den LEVELS-Reiter")
    wipe_files()


# ====================================================================== ugc

def load_head_ugc():
    try:
        src = subprocess.run(["git", "show", "HEAD:ugc.py"], cwd=REPO, capture_output=True,
                             timeout=60).stdout.decode("utf-8")
    except (OSError, subprocess.SubprocessError):
        return None
    if "def load_maps" not in src:
        return None
    path = os.path.join(TMP, "ugc_head.py")
    with open(path, "w", encoding="utf-8") as f:
        f.write(src)
    spec = importlib.util.spec_from_file_location("ugc_head", path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def audit_ugc():
    print("\nugc.py - Minigolf kompatibel zur Fassung aus git HEAD")
    old = load_head_ugc()
    if not check(old is not None, "HEAD:ugc.py lesbar"):
        return
    new = ugc
    saved_path = new._PATH
    results = {}
    for name, mod in (("old", old), ("new", new)):
        mod._PATH = os.path.join(TMP, "ugc_%s.json" % name)
        if os.path.exists(os.path.join(REPO, "ugc.json")):
            shutil.copy(os.path.join(REPO, "ugc.json"), mod._PATH)
        r = []
        r.append(json.dumps(mod.load_maps(), sort_keys=True))
        m1 = mod.new_map("Tunnel Blick", "tunnel-blick", "Lama")
        m2 = mod.new_map("Zweite", "zweite", "")
        r.append(mod.save_map(m1))
        r.append(mod.save_map(m2))
        r.append(mod.save_map(dict(m2, name="Arschloch")))
        r.append(mod.save_map({"id": "Böse ID", "name": "x", "tee": [1, 1], "cup": [2, 2]}))
        r.append(mod.set_last_author("Lama"))
        r.append(mod.last_author())
        r.append(mod.unique_id("Tunnel Blick"))
        r.append((mod.count(), mod.is_full(), mod.get("zweite") is not None))
        exp = os.path.join(TMP, "exp_%s.lamapgzmap" % name)
        r.append(mod.export_to(mod.get("tunnel-blick"), exp))
        payload = json.load(open(exp, encoding="utf-8"))
        payload.pop("exported", None)
        r.append(json.dumps(payload, sort_keys=True))
        r.append(mod.import_from(exp)[:2])
        naked = os.path.join(TMP, "naked_%s.json" % name)
        with open(naked, "w", encoding="utf-8") as f:
            json.dump(dict(mod.get("zweite"), id="nackt"), f)
        r.append(mod.import_from(naked)[:2])
        r.append(mod.delete_map("zweite"))
        r.append(mod.delete_map("gibt-es-nicht"))
        r.append([mod.slug(t) for t in ("Mein Tunnel!", "Ärger über Öl", "__x__", "", 5)])
        r.append([mod.valid_id(t) for t in ("abc", "ABC", "", "a" * 33, None)])
        maps = mod.load_maps()
        for m in maps:
            m.pop("edited", None)
            m.pop("created", None)
        r.append(json.dumps(maps, sort_keys=True))
        data = json.load(open(mod._PATH, encoding="utf-8"))
        data.pop("_generated", None)
        for m in data.get("minigolf", []):
            m.pop("edited", None)
            m.pop("created", None)
        r.append(json.dumps(data, sort_keys=True))
        results[name] = r
    new._PATH = saved_path
    labels = ["vorhandene ugc.json lädt identisch", "save 1", "save 2", "Wortfilter", "ungültige id",
              "Creator merken", "Creator lesen", "unique_id", "count/is_full/get", "export_to",
              "Export-Datei", "Import mit Umschlag", "Import nackte Map", "delete", "delete fehlt",
              "slug", "valid_id", "Sammlung danach", "Dateiinhalt danach"]
    diff = [labels[i] for i, (a, b) in enumerate(zip(results["old"], results["new"])) if a != b]
    check(not diff and len(results["old"]) == len(labels),
          "Minigolf: %d Operationen liefern dasselbe wie vorher (auch Dateiinhalt)" % len(labels), diff)
    check(ugc.max_items() == 60 and ugc.ext() == ".lamapgzmap" and ugc.FORMAT == "pygamez.minigolf.map"
          and ugc.ext("geodash") == ".lamapgzlevel" and ugc.max_items("geodash") == 60
          and ugc.import_exts("geodash") == (".lamapgzlevel", ".json"),
          "Registry: Minigolf-Standardwerte unverändert, geodash eigene Endung")
    # Arten bleiben getrennt, unbekannte Listen überleben
    wipe_files()
    with open(ugc._PATH, "w", encoding="utf-8") as f:
        json.dump({"v": 1, "zukunft": [{"id": "a"}], "minigolf": []}, f)
    ugc.save_map(ugc.new_map("Golf", "golf", ""))
    ugc.save_map(core.normalize_level({"id": "golf", "name": "Level", "objects": [["spike", 20, 0, 0]]}), "geodash")
    data = json.load(open(ugc._PATH, encoding="utf-8"))
    ugc.delete_map("golf")
    data2 = json.load(open(ugc._PATH, encoding="utf-8"))
    check(data.get("zukunft") == [{"id": "a"}] and len(data["minigolf"]) == 1 and len(data["geodash"]) == 1
          and "minigolf" not in data2 and len(data2["geodash"]) == 1 and data2.get("zukunft"),
          "gleiche id in zwei Arten, unbekannte Listen bleiben erhalten")
    lvl_file = os.path.join(TMP, "l.lamapgzlevel")
    ugc.export_to(ugc.get("golf", "geodash"), lvl_file, "geodash")
    naked = os.path.join(TMP, "n.json")
    with open(naked, "w", encoding="utf-8") as f:
        json.dump(ugc.get("golf", "geodash"), f)
    check(ugc.import_from(lvl_file, "minigolf")[:2] == (False, "format")
          and ugc.import_from(naked, "geodash")[:2] == (False, "format")
          and ugc.import_from(lvl_file, None)[2]["id"] == "golf-2",
          "Import prüft die Art im Umschlag (nackt nur Minigolf, None = jede Art)")
    for i in range(70):
        ugc.save_map(core.normalize_level({"id": "l%d" % i, "name": "L", "objects": []}), "geodash")
    check(ugc.count("geodash") == 60 and ugc.is_full("geodash") and not ugc.is_full("minigolf"),
          "Höchstzahl je Art (60)")
    wipe_files()


# ====================================================================== Layout

def fits(text, maxw, fonts):
    return any(f.size(text)[0] <= maxw for f in fonts)


def inside(r, w, h):
    return r.left >= 0 and r.top >= 0 and r.right <= w and r.bottom <= h


def overlap(rects):
    for i, a in enumerate(rects):
        for b in rects[i + 1:]:
            if a.colliderect(b):
                return "%s/%s" % (a, b)
    return None


def audit_layout():
    print("\nLayout - 5 Auflösungen x 14 Sprachen")
    lang_before = i18n.get_language()
    for w, h in RES:
        g = new_game(w=w, h=h)
        sel = g.tab_rects + [g.card_rect] + g.arrow_rects + g.mode_rects + [g.start_rect] + list(g.opt_rects.values())
        dots = g._dot_rects()
        foot_top = h - g._tiny.get_height() * 2 - 14
        check(all(inside(r, w, h) for r in sel) and overlap(sel) is None
              and all(g.card_rect.bottom <= r.top and r.bottom <= g.mode_rects[0].top for r in dots)
              and g.opt_bottom <= foot_top,
              "%4dx%d Levelauswahl: im Bild, ohne Überlappung, Fußzeile frei" % (w, h),
              (overlap(sel), g.opt_bottom, foot_top))
        P = g.card_parts(g.card_rect)
        head_bottom = P["head_top"] + P["head_h"]
        labels_top = min(r.y for r in P["bars"]) - g._tiny.get_height() - 2
        thumb = P["thumb"]
        shown = thumb.h >= 26 and thumb.w >= 60          # wie in _draw_card
        check(all(g.card_rect.contains(r) for r in P["bars"]) and overlap(P["bars"]) is None
              and labels_top >= head_bottom
              and (not shown or (g.card_rect.contains(thumb) and head_bottom <= thumb.top
                                 and thumb.bottom <= labels_top))
              and (shown or w <= 480),
              "%4dx%d Level-Karte: Kopf, %sBestwert-Balken" % (w, h, "Vorschau, " if shown else ""),
              (g.card_rect, P))
        lists = edit.LevelList(g)
        btns = list(lists.btn_rects.values())
        share = [lists.author_rect, lists.file_rect] + list(lists.share_btn.values())
        check(all(inside(r, w, h) for r in btns) and overlap(btns) is None
              and lists.list_bottom < min(r.top for r in btns) and lists.rows_visible >= 3
              and all(lists.share_rect.contains(r) for r in share) and overlap(share) is None
              and inside(lists.share_rect, w, h),
              "%4dx%d LEVELS-Reiter + Teilen-Dialog" % (w, h))
        ed = edit.LevelEditor(g, edit.new_level("x", "x", ""))
        head = [ed.back_rect, ed.name_rect, ed.id_rect, ed.save_rect] + list(ed.edit_rects.values())
        pal = [r for _, r in ed.tool_rects] + [r for _, r in ed.group_rects]
        items = max((ed.item_rects() for ed.group in edit.GROUP_KEYS), key=len)
        ed.group = "blocks"
        allr = head + pal + [r for _, r in items] + [ed.canvas, ed.map_rect]
        ed.objects = [["color", 5, 0, 0, 0, 255, 255, 255, 60], ["p_ship", 8, 1, 0, 0]]
        ed.sel = 0
        prm = [r for rs in ed._param_rects() for r in rs[1:]]
        set_rows = [r for rs in ed.set_rows.values() for r in rs]
        check(all(inside(r, w, h) for r in allr) and overlap(allr) is None
              and max(r.bottom for _, r in items) <= ed.par_top
              and all(r.right <= ed.canvas.right and r.top >= ed.par_top and r.bottom <= ed.map_rect.top
                      for r in prm) and overlap(prm) is None
              and inside(ed.set_rect, w, h) and all(ed.set_rect.contains(r) for r in set_rows + [ed.set_close])
              and overlap(set_rows + [ed.set_close]) is None,
              "%4dx%d Editor: Kopfzeile, Palette, Leinwand, Werte, Einstellungen" % (w, h),
              overlap(allr) or overlap(prm) or overlap(set_rows))
        # Höhen: jede Schrift passt in ihren Knopf / ihre Zeile (auch bei 1280x960,
        # wo die Schrift mitwächst), Beschriftungen stoßen nirgends an.
        tall = []

        def room(rects, fnt, pad, what):
            for r in rects:
                if r.h < fnt.get_height() + pad:
                    tall.append("%s: %d < %d" % (what, r.h, fnt.get_height() + pad))
                    return

        room(g.tab_rects, g._small, 6, "Reiter")
        room(g.mode_rects, g._small, 8, "Modus")
        room([g.start_rect], g._btn_font, 8, "Start")
        room(list(g.opt_rects.values()), g._tiny, 6, "Optionen")
        g.cur_kind, g.sel = "main", 0
        room([rc for _, rc in g._complete_buttons()[1]], g._small, 8, "Ergebnis-Knöpfe")
        room(btns, lists.tiny, 6, "LEVELS-Knöpfe")
        room(share, lists.tiny, 6, "Teilen-Felder")
        room(head, ed.tiny, 6, "Editor-Kopfzeile")
        room([r for rs in ed.set_rows.values() for r in rs[1:2]] + [ed.set_close], ed.tiny, 6, "Einstellungen")
        if lists.two_lines and lists.row_h - 3 < lists.fnt.get_height() + lists.tiny.get_height() + 6:
            tall.append("Listenzeile %d" % lists.row_h)
        if lists.list_bottom + 2 + lists.tiny.get_height() > min(r.top for r in btns):
            tall.append("Zähler stößt an die Knöpfe")
        sr = lists.share_rect
        if (sr.y + 10 + lists.fnt.get_height() > lists.author_rect.y - lists.tiny.get_height() - 2
                or lists.author_rect.bottom > lists.file_rect.y - lists.tiny.get_height() - 2
                or lists.share_btn["cancel"].bottom + lists.tiny.get_height() + 6 > sr.bottom):
            tall.append("Teilen-Dialog: Titel/Beschriftung/Fehlerzeile stoßen an")
        if ed.set_rect.y + 10 + ed.small.get_height() > min(r.top for r in set_rows):
            tall.append("Einstellungen: Titel stößt an die erste Zeile")
        if prm and min(r.top for r in prm) - ed.par_lab_h - 1 < ed.par_top:
            tall.append("Werte-Beschriftung ragt in die Leinwand")
        hint_top = h - 4 - g._tiny.get_height()
        star_top = hint_top - 2 - g._tiny.get_height()
        if star_top < g.opt_bottom + 2:
            tall.append("Fußzeile stößt an die Optionen")
        check(not tall, "%4dx%d Schrift passt in Knöpfe, Zeilen und Dialoge (Höhe)" % (w, h), "; ".join(tall))
        ed.sel = None
        too_long = []
        for code, _ in i18n.AVAILABLE:
            i18n.set_language(code, persist=False)
            T = i18n.t
            ed.layout()                      # Text-Knöpfe richten sich nach der Sprache
            row = sorted(ed.edit_rects.values(), key=lambda r: r.x)
            if row[-1].right > ed.canvas.right or overlap(row):
                too_long.append("%s: Editor-Knopfzeile %d > Leinwand %d" % (code, row[-1].right, ed.canvas.right))
            sm = (g._small, g._tiny)
            ti = (g._tiny,)
            tests = [
                (T("gd.title"), w - 20, (g._huge,)),
                (T("gd.subtitle"), w - 20, (g._small,)),
                (T("gd.ugc.subtitle"), w - 20, (g._small,)),
                (T("gd.mode.normal"), g.mode_rects[0].w - 12, sm),
                (T("gd.mode.practice"), g.mode_rects[1].w - 12, sm),
                (T("common.start"), g.start_rect.w - 10, (g._btn_font, g._small)),
                (T("gd.select_hint"), w - 16, (g._tiny, ui.font(max(10, h // 48)))),
                (T("gd.total_stars", n=65, max=65), w - 40, ti),
                (T("gd.practice_hud"), w - 20, ti),
                (T("gd.test_hud"), w - 20, ti),
                (T("gd.new_best", pct=99), w - 20, (ui.font(int(max(18, h // 14) * 1.25), bold=True),)),
                (T("gd.attempt", n=9999), w * 0.66, (g._attempt_font,)),
            ]
            for i, tab in enumerate(gd.TABS):
                tests.append((T("gd.tab_" + tab), g.tab_rects[i].w - 12, sm))
            for key_, on in (("music", True), ("bar", False), ("auto", True)):
                label = T("gd.opt_" + key_) + ": " + (T("common.on") if on else T("common.off"))
                tests.append((label, g.opt_rects[key_].w - 8, ti))
            # Level-Karte
            P = g.card_parts(g.card_rect)
            for dk in gd.DIFF_KEYS:
                tests.append((T("gd.diff." + dk), 2 * P["fr"] + 30, (g._tiny, ui.font(max(9, h // 52)))))
            tests.append((T("gd.attempts_total", n=9999), P["right"] - (P["pad"] + 2 * P["fr"] + 18), ti))
            for i, mk in enumerate(("gd.mode.normal", "gd.mode.practice")):
                bar = P["bars"][i]
                tests.append((T("gd.best_label", mode=T(mk)), bar.w - g._tiny.size("100%")[0] - 8, ti))
            for d in LEVELS:
                tests.append((d["name"], P["right"] - (P["pad"] + 2 * P["fr"] + 18), (g._mid, g._small)))
            # Ergebnis-Panel
            for kind in ("main", "test"):
                g.cur_kind = kind
                g.sel = 0
                panel, rects = g._complete_buttons()
                labels = {"again": T("gd.btn_again"), "select": T("gd.btn_levels"),
                          "next": T("gd.btn_next"), "editor": T("gd.btn_editor")}
                for k_, rc in rects:
                    tests.append((labels[k_], rc.w - 8, sm))
                tests.append((T("gd.complete"), panel.w - 20, (g._huge, g._mid)))
                tests.append((T("gd.complete_practice"), panel.w - 20, (g._huge, g._mid)))
                extra = "  ·  ".join((T("gd.stars_won", n=10), T("gd.coins_won", n=3)))
                tests.append((extra, panel.w - 20, sm))
                tests.append(("  ·  ".join((T("gd.verified_now"),)), panel.w - 20, sm))
                tests.append((T("gd.practice_note"), panel.w - 20, sm))
                for lab in ("gd.stat_attempts", "gd.stat_jumps", "gd.stat_time"):
                    tests.append((T(lab) + "  9999", panel.w - 60, (g._small,)))
                # Inhalt endet über den Knöpfen
                y = panel.y + 12 + g._huge.get_height() + 4 + g._small.get_height() + 8
                y += 3 * (g._small.get_height() + 2) + 6 + 2 * max(8, h // 40) + 8 + g._small.get_height()
                if y > rects[0][1].top - 2:
                    too_long.append("%s: Ergebnis-Inhalt %d > Knöpfe %d" % (code, y, rects[0][1].top))
            # LEVELS-Reiter + Teilen
            lt = (lists.tiny,)
            for k_, rc in lists.btn_rects.items():
                tests.append((T("gd.ugc.btn_" + k_), rc.w - 8, lt))
            for k_ in ("gd.ugc.empty", "gd.ugc.empty2"):
                tests.append((T(k_), w - 48 - 16, (lists.fnt, lists.tiny)))
            tests.append((T("gd.ugc.count", n=60, max=60), w - 24, lt))
            tests.append((T("gd.ugc.confirm_delete"), lists.list_w * 0.45, lt))
            for k_ in ("gd.ugc.share_title",):
                tests.append((T(k_), lists.share_rect.w - 20, (lists.fnt,)))
            for k_ in ("gd.ugc.creator", "gd.ugc.filename"):
                tests.append((T(k_), lists.share_rect.w - 36, lt))
            tests.append((T("gd.ugc.export_as"), lists.share_btn["as"].w - 8, lt))
            tests.append((T("gd.ugc.export_dl"), lists.share_btn["dl"].w - 8, lt))
            tests.append((T("gd.ugc.btn_cancel"), lists.share_btn["cancel"].w - 8, lt))
            for k_ in ("swear", "io", "nofile"):
                tests.append((T("gd.ugc.err." + k_), lists.share_rect.w - 12, lt))
            toast_max = w - 20
            for k_ in ("gd.ugc.deleted", "gd.ugc.err.format", "gd.ugc.err.full", "gd.ugc.err.nofile"):
                tests.append((T(k_, max=60), toast_max, lt))
            tests.append((T("gd.ugc.imported", name="W" * 28), toast_max, lt))
            tests.append((T("gd.ugc.renamed", id="w" * 30), toast_max, lt))
            # Editor
            et = (ed.tiny,)
            tests.append((T("gd.ugc.btn_back"), ed.back_rect.w - 8, et))
            tests.append((T("gd.ugc.btn_save"), ed.save_rect.w - 8, et))
            for k_ in edit.TEXT_BUTTONS:
                rc = ed.edit_rects[k_]
                tests.append((T("gd.ed.btn_" + k_), rc.w - rc.h - 4, et))
            for tool in edit.TOOLS:
                tests.append((T("gd.ed.tool." + tool), ed.canvas.w - 30, et))
            for group, kinds in edit.GROUPS:
                for kind in kinds:
                    tests.append((T("gd.ed.obj." + kind) + "  ·  " + T("gd.ed.group." + group), ed.canvas.w - 30, et))
            for k_ in ("hint_place", "hint_select", "hint_erase"):
                tests.append((T("gd.ed." + k_), ed.canvas.w, (ed.tiny, ui.font(max(9, h // 52)))))
            for k_ in ("gd.ugc.saved", "gd.ugc.unsaved", "gd.ed.verified", "gd.ed.coins_max"):
                tests.append((T(k_), ed.canvas.w - 24, et))
            for k_ in ("id_empty", "id_chars", "name", "id_dup", "swear", "full", "io", "invalid", "id"):
                tests.append((T("gd.ugc.err." + k_, max=60), ed.canvas.w - 12, et))
            ed.objects = [["color", 5, 0, 0, 0, 255, 255, 255, 60], ["p_ship", 8, 1, 0, 0]]
            for sel_i, labs in ((0, ("gd.ed.par.target", "R", "G", "B", "gd.ed.par.dur")), (1, ("gd.ed.par.height",))):
                ed.sel = sel_i
                prm = ed._param_rects()
                for (i, _m, val, _p), lab in zip(prm, labs):
                    tests.append((T(lab) if lab.startswith("gd.") else lab, val.w + 2 * _m.w, (ui.font(max(9, h // 52)),)))
                    if sel_i == 0 and i == 0:
                        for tk in ("gd.ed.target_bg", "gd.ed.target_ground"):
                            tests.append((T(tk), val.w - 4, (ed.tiny, ui.font(max(9, h // 56)))))
            ed.sel = None
            tests.append((T("gd.ed.settings"), ed.set_rect.w - 20, (ed.small,)))
            for k_ in edit.SETTINGS_ROWS:
                tests.append((T("gd.ed.set_" + k_), ed.lab_w - 6, et))
            vw = ed.set_rows["speed"][1].w - 8
            for mn in core.MODE_NAMES:
                tests.append((T("gd.ed.mode." + mn), vw, et))
            for st in core.MUSIC_STYLES:
                tests.append((T("gd.ed.music." + st), vw, et))
            tests.append((T("gd.ed.close"), ed.set_close.w - 8, et))
            for text, maxw, fonts in tests:
                if not fits(text, maxw, fonts):
                    too_long.append("%s:'%s'" % (code, text))
        i18n.set_language(lang_before, persist=False)
        check(not too_long, "%4dx%d alle Texte passen ohne Kürzung (14 Sprachen)" % (w, h),
              "; ".join(too_long))
    audit_draw()
    # Auflösungswechsel mitten im Spiel
    g = new_game(w=640, h=480)
    g.start_level(LEVELS[3], "main")
    frames(g, 20)
    g.draw()
    g.surface = pygame.Surface((1280, 960))
    g.width, g.height = 1280, 960
    g.on_surface_changed()
    frames(g, 5)
    g.draw()
    check(g.renderer.ts == g._ts() == int(960 / gd.VIEW_ROWS) and g.start_rect.right <= 1280,
          "Auflösungswechsel im Spiel: Kacheln, Schriften und Layout neu")


def _draw_all(g, w, h, errors, tag):
    """Alle Screens einmal zeichnen (Auswahl, fünf Formen, Tod, Ergebnis, Reiter, Editor)."""
    g.update(1 / 60.0)
    g.draw()
    g.practice = True
    g.draw()
    g._set_tab("levels")
    g.draw()
    g.lists.items = [core.normalize_level(dict(edit.new_level("Sehr langer Levelname XY", "sehr-langer-level-name-xyz", "Lama Lama Lama"),
                                               objects=[["spike", 20, 0, 0]], verified="00000000"))] * 12
    g.lists.draw(g.surface)
    g.lists._open_share(g.lists.items[0])
    g.lists.err = i18n.t("gd.ugc.err.swear")
    g.lists.draw(g.surface)
    g.lists._close_share()
    g.lists.items = []
    g._set_tab("play")
    inferno = LEVELS[7]
    starts = {"cube": None, "wave": 126, "ship": 205, "ball": 285, "ufo": 348}
    for form, start in starts.items():
        g.start_level(inferno, "main", start_block=start, practice=(form == "ship"))
        frames(g, 12)
        if g.run.dead:
            errors.append("%s: %s-Abschnitt stirbt sofort" % (tag, form))
        g.draw()
    g.run.dead = 1
    g._on_death()
    frames(g, 3)
    g.draw()
    g.run.dead = 0
    g.run.won = 1
    g.run.coins = 5
    g._on_win()
    g.complete_t = 0.2
    g.draw()
    g.complete_t = 1.0
    g.draw()
    g.ugc_new_level()
    ed = g.editor
    for i, kind in enumerate(core.KIND_NAMES):
        ed.tool = kind
        ed._place((3 + 2 * i, 1 if kind != "pit" else 0))
    ed.sel = next(i for i, o in enumerate(ed.objects) if o[0] == "color")
    ed.hover = (7, 2)
    g.draw()
    ed.tool = "block"
    ed.hover = (9, 3)
    ed.err = i18n.t("gd.ugc.err.id_dup")
    g.draw()
    ed.err = ""
    ed._toast(i18n.t("gd.ugc.saved"))
    ed.verified = core.content_hash(ed.level_dict())
    g.draw()
    ed._open_settings()
    g.draw()
    ed.settings_open = False
    ed._test(False)
    frames(g, 10)
    g.draw()


def audit_draw():
    errors = []
    lang_before = i18n.get_language()
    for theme in ("v42", "v1"):
        ui.set_theme(theme)
        for w, h in RES:
            for code, _ in i18n.AVAILABLE:
                if theme == "v1" and code not in ("de", "fi"):
                    continue
                i18n.set_language(code, persist=False)
                tag = "%s/%s %dx%d" % (theme, code, w, h)
                try:
                    _draw_all(new_game("practice" if code == "fi" else "normal", w, h), w, h, errors, tag)
                except Exception as exc:  # noqa: BLE001
                    import traceback
                    errors.append("%s: %r %s" % (tag, exc, traceback.format_exc(limit=-2)))
    i18n.set_language(lang_before, persist=False)
    ui.set_theme("v42")
    wipe_files()
    check(not errors, "alle Screens zeichnen fehlerfrei (v4.2: 14 Sprachen, v1: de+fi; alle fünf Formen)",
          "; ".join(errors[:3]))


# ====================================================================== Leistung

def audit_performance():
    print("\nLeistung - 1280x960")
    w, h = 1280, 960
    g = new_game(w=w, h=h)
    d = LEVELS[7]
    g.start_level(d, "main")
    tg = PROOFS[d["id"]]["toggles"]
    CLOCK.t = 0.0
    g._last_update = 0.0
    dt = 1 / 60.0
    ti, down, frame = 0, True, 0
    times = []
    while g.state == gd.PLAY and frame < 4000:
        t_now = (frame + 1) * dt
        while ti < len(tg) and (tg[ti] + 0.5) / core.HZ <= t_now:
            CLOCK.t = (tg[ti] + 0.5) / core.HZ
            key(g, "space", down=down)
            down = not down
            ti += 1
        CLOCK.t = t_now
        t0 = time.perf_counter()
        g.update(dt)
        g.draw()
        times.append((time.perf_counter() - t0) * 1000.0)
        frame += 1
    times = times[30:]
    avg = sum(times) / max(1, len(times))
    p95 = sorted(times)[int(len(times) * 0.95)] if times else 0
    check(g.state == gd.COMPLETE and avg < 8.0 and p95 < 14.0,
          "Lama Inferno komplett (%d Frames): Ø %.2f ms, 95 %% unter %.2f ms (Budget 16,7 ms)"
          % (len(times), avg, p95))
    g.ugc_new_level()
    ed = g.editor
    for i in range(400):
        ed.tool = ("block", "spike", "orb_y", "p_ship", "coin", "pad_y")[i % 6]
        ed._place((i // 2, i % 9))
    ed.draw(g.surface)
    t0 = time.perf_counter()
    for i in range(120):
        ed.cam_x = i * 0.25
        ed.update(dt)
        g.draw()
    ms = (time.perf_counter() - t0) * 1000.0 / 120
    check(ms < 12.0, "Editor mit 400 Objekten beim Scrollen: %.2f ms je Frame" % ms)
    return avg


if __name__ == "__main__":
    t_start = time.time()
    wipe_files()
    # --only core,layout,...  (Teile: core levels web input game editor ugc layout performance)
    only = sys.argv[sys.argv.index("--only") + 1].split(",") if "--only" in sys.argv else None
    try:
        if REGISTERED is not None:
            check(REGISTERED, "GeometryDashGame ist in games.ALL_GAMES registriert")
        for part in ("core", "levels", "web", "input", "game", "editor", "ugc", "layout", "performance"):
            if only is None or part in only:
                globals()["audit_" + part]()
    finally:
        wipe_files()
        shutil.rmtree(TMP, ignore_errors=True)
    print("\n%s  (%.1f s)" % ("ALLE PRÜFUNGEN BESTANDEN" if not FAILS
                              else "%d FEHLER: %s" % (len(FAILS), FAILS),
                              time.time() - t_start))
    sys.exit(1 if FAILS else 0)
