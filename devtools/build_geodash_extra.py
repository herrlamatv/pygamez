# -*- coding: utf-8 -*-
"""
build_geodash_extra.py
======================
Baut die **Level-Bibliothek** von Geometry Dash (Reiter „Weitere") und die
**Vorlagen** für eigene Level - und beweist mit demselben Solver wie bei den
Kampagnen-Leveln, dass jedes Level schaffbar ist (inklusive aller drei
Münzen, robust gegen ±1 Schritt Versatz).

Geschrieben werden::

    games/levels/geodash_extra.json         Bibliothek (Desktop)
    web/js/games/geodash_extra_index.js     Liste für den Reiter (immer geladen)
    web/js/games/geodash_extra.js           Volldaten (wird nachgeladen)
    games/levels/geodash_templates.json     Vorlagen (Desktop)
    web/js/games/geodash_templates.js       Vorlagen (Browser)
    devtools/geodash_extra_proofs.json      die Lösungen + Seeds

Aufruf aus dem Repo-Root::

    python devtools/build_geodash_extra.py                  # alles bauen
    python devtools/build_geodash_extra.py --cat ship       # nur eine Kategorie
    python devtools/build_geodash_extra.py --only x-ship-03
    python devtools/build_geodash_extra.py --jobs 4         # Prozesse
    python devtools/build_geodash_extra.py --no-solve       # nur schreiben
    python devtools/build_geodash_extra.py --strict         # Exit 1, wenn eins fehlt
    python devtools/build_geodash_extra.py --preview D      # Übersichtsbilder

Die Level entstehen aus den Bausteinen in ``devtools/geodash_sections.py``:
je Kategorie ein Rezept (Formen, Tempo, Härtegrad, Länge, Farben, Musik),
daraus würfelt ein **fester Seed** die Abfolge der Sektionen. Gleicher Seed =
gleiches Level, heute wie in einem Jahr.

Findet der Solver keinen Weg, wird nicht blind neu gewürfelt: erst wird die
Sektion **entschärft**, an der er hängen bleibt (``info["best_x"]``), und erst
danach ein neuer Seed genommen.
"""

import json
import os
import random
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, ROOT)
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, os.path.join(ROOT, "games"))

import geodash_core as core  # noqa: E402
import build_geodash_solver as solver  # noqa: E402
import geodash_sections as sec  # noqa: E402
from build_geodash_levels import LB, _dump_levels, level  # noqa: E402

OUT_JSON = os.path.join(ROOT, "games", "levels", "geodash_extra.json")
OUT_INDEX_JS = os.path.join(ROOT, "web", "js", "games", "geodash_extra_index.js")
OUT_FULL_JS = os.path.join(ROOT, "web", "js", "games", "geodash_extra.js")
OUT_TPL_JSON = os.path.join(ROOT, "games", "levels", "geodash_templates.json")
OUT_TPL_JS = os.path.join(ROOT, "web", "js", "games", "geodash_templates.js")
OUT_PROOFS = os.path.join(ROOT, "devtools", "geodash_extra_proofs.json")

CUBE, SHIP, BALL, UFO, WAVE = core.CUBE, core.SHIP, core.BALL, core.UFO, core.WAVE


# ---------------------------------------------------------------------------
#  Rezepte je Kategorie
# ---------------------------------------------------------------------------
#   modes   : Formen, aus denen die Akte gewürfelt werden (erste = Hauptform)
#   speeds  : erlaubte Tempo-Stufen (1 = normal, 2 = schnell, 3 = sehr schnell)
#   hard    : Härtegrad-Bereich der Sektionen (0 = zahm, 2 = bissig)
#   acts    : Anzahl der Abschnitte (je Abschnitt eine Form)
#   diff    : Schwierigkeit fürs Abzeichen (0 Leicht ... 5 Dämon)
#   stars   : Sterne je Level
#   music   : Musikstil + BPM
#   palette : Farbpaare (Hintergrund, Boden), aus denen die Akte wählen
CATS = {
    "starter": {"modes": (CUBE,), "speeds": (1,), "hard": (0, 0), "acts": (2, 3),
                "diff": 0, "stars": 1, "music": ("chip", 128), "n": 8,
                "palette": [((60, 150, 255), (30, 90, 200)),
                            ((80, 200, 160), (40, 130, 110)),
                            ((120, 170, 255), (60, 100, 190))]},
    "rhythm": {"modes": (CUBE,), "speeds": (1, 2), "hard": (0, 1), "acts": (3, 4),
               "diff": 1, "stars": 3, "music": ("chip", 145), "n": 8,
               "palette": [((255, 140, 60), (170, 80, 30)),
                           ((240, 90, 160), (150, 40, 100)),
                           ((255, 200, 70), (180, 130, 30))]},
    "speed": {"modes": (CUBE,), "speeds": (2, 3), "hard": (1, 2), "acts": (3, 4),
              "diff": 3, "stars": 7, "music": ("drive", 168), "n": 8,
              "palette": [((255, 70, 70), (160, 30, 30)),
                          ((255, 120, 0), (170, 70, 0)),
                          ((200, 40, 90), (120, 20, 60))]},
    "ship": {"modes": (SHIP, CUBE), "speeds": (1, 2), "hard": (0, 2), "acts": (3, 4),
             "diff": 2, "stars": 5, "music": ("drive", 140), "n": 8,
             "palette": [((0, 170, 220), (0, 100, 140)),
                         ((60, 120, 255), (30, 60, 170)),
                         ((0, 200, 190), (0, 120, 120))]},
    "ufo": {"modes": (UFO, CUBE), "speeds": (1, 2), "hard": (0, 2), "acts": (3, 4),
            "diff": 2, "stars": 5, "music": ("dream", 132), "n": 8,
            "palette": [((150, 110, 255), (90, 60, 170)),
                        ((200, 120, 255), (120, 70, 170)),
                        ((110, 140, 255), (60, 80, 180))]},
    "ball": {"modes": (BALL, CUBE), "speeds": (1, 2), "hard": (0, 2), "acts": (3, 4),
             "diff": 2, "stars": 5, "music": ("dream", 124), "n": 8,
             "palette": [((0, 200, 120), (0, 120, 80)),
                         ((120, 200, 60), (70, 130, 30)),
                         ((0, 180, 170), (0, 110, 110))]},
    "wave": {"modes": (WAVE, CUBE), "speeds": (1, 2), "hard": (0, 2), "acts": (3, 4),
             "diff": 4, "stars": 9, "music": ("drive", 160), "n": 8,
             "palette": [((255, 40, 120), (170, 20, 80)),
                         ((220, 0, 200), (140, 0, 130)),
                         ((255, 90, 150), (180, 40, 100))]},
    "gravity": {"modes": (CUBE,), "speeds": (1, 2), "hard": (1, 2), "acts": (3, 4),
                "diff": 3, "stars": 7, "music": ("dream", 136), "n": 8,
                "palette": [((90, 90, 200), (50, 50, 130)),
                            ((60, 160, 200), (30, 90, 130)),
                            ((130, 90, 220), (70, 50, 140))]},
    "coins": {"modes": (CUBE, SHIP), "speeds": (1, 2), "hard": (0, 1), "acts": (3, 4),
              "diff": 1, "stars": 3, "music": ("chip", 138), "n": 8,
              "palette": [((255, 205, 80), (180, 140, 30)),
                          ((240, 180, 60), (160, 110, 20)),
                          ((255, 160, 90), (170, 100, 40))]},
    "demon": {"modes": (CUBE, SHIP, WAVE, BALL, UFO), "speeds": (2, 3),
              "hard": (2, 2), "acts": (4, 5), "diff": 5, "stars": 10,
              "music": ("dark", 176), "n": 8,
              "palette": [((140, 0, 40), (80, 0, 20)),
                          ((60, 0, 90), (30, 0, 50)),
                          ((200, 20, 20), (120, 10, 10))]},
}
CAT_ORDER = ("starter", "rhythm", "coins", "ship", "ufo", "ball", "gravity",
             "speed", "wave", "demon")

# Namen: zwei Wortlisten je Kategorie (englisch, werden NICHT übersetzt -
# genau wie die Namen der Kampagnen-Level).
ADJ = {
    "starter": ("First", "Easy", "Soft", "Little", "Calm", "Fresh", "Bright", "Warm"),
    "rhythm": ("Beat", "Pulse", "Echo", "Groove", "Tempo", "Drum", "Synth", "Loop"),
    "speed": ("Turbo", "Rapid", "Hyper", "Blaze", "Rocket", "Flash", "Sonic", "Nitro"),
    "ship": ("Sky", "Cloud", "Jet", "Hangar", "Aero", "Wing", "Cargo", "Drift"),
    "ufo": ("Alien", "Orbit", "Saucer", "Cosmic", "Beam", "Nova", "Comet", "Lunar"),
    "ball": ("Rolling", "Bounce", "Round", "Tumble", "Spin", "Marble", "Pebble", "Loop"),
    "wave": ("Neon", "Laser", "Zig", "Pixel", "Static", "Signal", "Glitch", "Prism"),
    "gravity": ("Upside", "Magnet", "Polar", "Flip", "Anti", "Mirror", "Twist", "Shift"),
    "coins": ("Golden", "Shiny", "Hidden", "Lucky", "Treasure", "Secret", "Amber", "Coin"),
    "demon": ("Lama", "Inferno", "Chaos", "Nightmare", "Abyss", "Doom", "Crimson", "Void"),
}
NOUN = {
    "starter": ("Steps", "Path", "Start", "Hills", "Garden", "Walk", "Trail", "Meadow"),
    "rhythm": ("Machine", "Line", "Chamber", "Session", "Floor", "Circuit", "Rush", "Set"),
    "speed": ("Run", "Dash", "Sprint", "Drive", "Lane", "Chase", "Burst", "Rally"),
    "ship": ("Route", "Bay", "Corridor", "Deck", "Tunnel", "Harbour", "Lane", "Port"),
    "ufo": ("Drift", "Field", "Gate", "Station", "Belt", "Arc", "Ring", "Dust"),
    "ball": ("Caves", "Halls", "Pipes", "Chambers", "Maze", "Tracks", "Wells", "Drop"),
    "wave": ("Reactor", "Grid", "Tunnel", "Storm", "Stream", "Circuit", "Shard", "Core"),
    "gravity": ("Down", "Fields", "Ceiling", "Flip", "Loop", "Zone", "Wells", "Shift"),
    "coins": ("Hunt", "Vault", "Trail", "Cache", "Run", "Route", "Hoard", "Chase"),
    "demon": ("Gate", "Descent", "Engine", "Spire", "Pit", "Throne", "Storm", "End"),
}


# ---------------------------------------------------------------------------
#  Bauen
# ---------------------------------------------------------------------------

def _rng(cat, idx, salt):
    return random.Random(core.fnv1a("gdx/%s/%02d/%d" % (cat, idx, salt)))


def _name(cat, rng, used):
    for _ in range(40):
        name = "%s %s" % (rng.choice(ADJ[cat]), rng.choice(NOUN[cat]))
        if name not in used:
            used.add(name)
            return name
    name = "%s %s %d" % (rng.choice(ADJ[cat]), rng.choice(NOUN[cat]), len(used))
    used.add(name)
    return name


def plan_acts(cfg, rng):
    """Abfolge der Abschnitte: [(form, tempo, anzahl_sektionen), ...]."""
    n_acts = rng.randint(*cfg["acts"])
    modes = list(cfg["modes"])
    speeds = list(cfg["speeds"])
    acts = []
    for i in range(n_acts):
        if i == 0:
            mode = CUBE if CUBE in modes else modes[0]
        elif len(modes) == 1:
            mode = modes[0]
        elif i % 2 == 1:
            mode = modes[0] if modes[0] != CUBE else rng.choice(modes)
        else:
            mode = CUBE if CUBE in modes else rng.choice(modes)
        speed = rng.choice(speeds)
        acts.append([mode, speed, rng.randint(2, 3)])
    return acts


def compose(cat, idx, salt=0, relax=0):
    """Baut ein Level aus Sektionen. Gibt (level_dict, plan) zurück."""
    cfg = CATS[cat]
    rng = _rng(cat, idx, salt)
    used_names = compose.used.setdefault(cat, set())
    name = _name(cat, rng, used_names)
    acts = plan_acts(cfg, rng)

    L = LB()
    ctx = {"speed": acts[0][1], "hard": cfg["hard"][0], "rng": rng,
           "top": 9, "lo": 2, "mode": CUBE}
    x = sec.intro(L, 0, ctx)
    plan = []
    coin_spots = []
    cur_mode, cur_speed = CUBE, 1
    palette = list(cfg["palette"])
    rng.shuffle(palette)

    for ai, (mode, speed, count) in enumerate(acts):
        # Übergang, wenn sich Form oder Tempo ändert (oder am Anfang das Tempo)
        if mode != cur_mode or speed != cur_speed:
            colors = palette[ai % len(palette)]
            top = 10 if mode == SHIP else (9 if mode == UFO else
                                           (8 if mode == WAVE else 7))
            ctx["mode"], ctx["speed"] = cur_mode, cur_speed
            x += sec.transition(L, x, ctx, mode, speed, colors=colors, top=top)
            cur_mode, cur_speed = mode, speed
            ctx["top"] = top
        ctx["speed"] = cur_speed
        ctx["mode"] = cur_mode
        last_family = None
        for si in range(count):
            hard = min(cfg["hard"][1], max(0, rng.randint(*cfg["hard"]) - relax))
            ctx["hard"] = hard
            pool = [s for s in sec.pick(cur_mode, cur_speed, cfg["diff"])
                    if s.family != last_family]
            if not pool:
                pool = sec.pick(cur_mode, cur_speed, cfg["diff"])
            if not pool:
                break
            s = pool[rng.randrange(len(pool))]
            w, coins = s.build(L, x, ctx)
            plan.append({"sec": s.key, "x": x, "w": w, "act": ai, "hard": hard})
            for cx, cy in coins:
                coin_spots.append((cx, cy, ai))
            x += w + sec.beat_for(cur_speed)
            last_family = s.family
    # Zurück auf den Würfel, damit jedes Level am Boden endet
    if cur_mode != CUBE:
        ctx["mode"], ctx["speed"] = cur_mode, cur_speed
        x += sec.transition(L, x, ctx, CUBE, cur_speed, colors=palette[0])
        cur_mode = CUBE
        ctx["mode"] = CUBE
    x += sec.outro(L, x, ctx)

    # Münzen setzt erst der Solver (auf die bewiesene Bahn, siehe coins_on_path).
    bg, ground = palette[0]
    style, bpm = cfg["music"]
    d = level({"id": "x-%s-%02d" % (cat, idx), "name": name,
               "category": cat, "difficulty": cfg["diff"], "stars": cfg["stars"],
               "speed": acts[0][1], "mode": CUBE, "bg": list(bg),
               "ground": list(ground), "music": style,
               "bpm": bpm + (idx % 4) * 2, "length": x + 10}, L)
    return d, plan


compose.used = {}


def flight_path(d, toggles):
    """Spielt eine Lösung ab und gibt die Bahn als [(x_block, y_block), ...]."""
    lv = core.Level(d)
    st = core.new_state(lv)
    pts = []
    ti, held, k = 0, False, 0
    limit = (lv.end_x // core.SPEEDS[0]) + 4 * core.HZ
    while not st.dead and not st.won and k < limit:
        was = held
        while ti < len(toggles) and toggles[ti] <= k:
            held = not held
            ti += 1
        core.step(st, lv, held, held and not was)
        k += 1
        if k % 3 == 0:
            pts.append((st.x / core.B, st.y / core.B))
    return pts


def occupied(d):
    """Alle belegten Zellen (Blöcke, Stacheln, Portale) als Menge."""
    out = set()
    for o in d["objects"]:
        if o[0] not in ("color", "pit"):
            out.add((o[1], o[2]))
    return out


def coins_on_path(d, toggles, offset=0):
    """Drei Münzen auf die bewiesene Bahn legen (verteilt über das Level).

    Weil die Münzen genau dort liegen, wo der Spieler ohnehin vorbeikommt,
    bleibt die Lösung gültig - das prüft der Aufrufer anschließend mit
    ``solver.verify``. 'offset' verschiebt die Auswahlpunkte ein Stück, falls
    eine Stelle doch nicht passt.
    """
    pts = flight_path(d, toggles)
    if not pts:
        return d
    taken = occupied(d)
    out = dict(d)
    objs = [list(o) for o in d["objects"]]
    used_x = []
    for frac in (0.24, 0.55, 0.84):
        want = (len(pts) - 1) * min(0.95, max(0.05, frac + offset * 0.04))
        for step_i in range(0, 60):
            for sign in (1, -1):
                i = int(want) + sign * step_i * 3
                if not (0 <= i < len(pts)):
                    continue
                cx, cy = int(pts[i][0]), int(pts[i][1] + 0.35)
                if cy < 0 or cy >= core.MAX_ROW:
                    continue
                if (cx, cy) in taken or any(abs(cx - ux) < 18 for ux in used_x):
                    continue
                objs.append(["coin", cx, cy, 0])
                taken.add((cx, cy))
                used_x.append(cx)
                break
            else:
                continue
            break
    out["objects"] = objs
    return core.normalize_level(out)


# ---------------------------------------------------------------------------
#  Vorlagen
# ---------------------------------------------------------------------------
#   (id, Länge in Sektionen, Form, Tempo, Sektionen, Farben, Musik)
TEMPLATES = [
    ("empty", CUBE, 1, [], ((40, 110, 255), (20, 70, 200)), "drive"),
    ("starter", CUBE, 1, ["cube_spikes", "cube_pit", "cube_stairs"],
     ((60, 150, 255), (30, 90, 200)), "chip"),
    ("jumps", CUBE, 1, ["cube_platform", "cube_stairs", "cube_spike_row"],
     ((80, 200, 160), (40, 130, 110)), "chip"),
    ("orbs", CUBE, 1, ["pad_hop", "orb_bed", "orb_double"],
     ((255, 200, 70), (180, 130, 30)), "chip"),
    ("ship", SHIP, 1, ["ship_gates", "ship_tunnel"],
     ((0, 170, 220), (0, 100, 140)), "drive"),
    ("wave", WAVE, 1, ["wave_lane", "wave_zigzag"],
     ((255, 40, 120), (170, 20, 80)), "drive"),
    ("ball", BALL, 1, ["ball_switch", "ball_pads"],
     ((0, 200, 120), (0, 120, 80)), "dream"),
    ("ufo", UFO, 1, ["ufo_gates", "ufo_gates"],
     ((150, 110, 255), (90, 60, 170)), "dream"),
    ("gravity", CUBE, 1, ["grav_run", "grav_pad"],
     ((90, 90, 200), (50, 50, 130)), "dream"),
    ("speed", CUBE, 2, ["cube_spikes", "cube_spike_row", "cube_pit"],
     ((255, 70, 70), (160, 30, 30)), "drive"),
    ("coins", CUBE, 1, ["cube_stairs", "cube_platform", "pad_hop"],
     ((255, 205, 80), (180, 140, 30)), "chip"),
    ("demon", CUBE, 3, ["cube_timing", "cube_spike_row", "orb_double"],
     ((140, 0, 40), (80, 0, 20)), "dark"),
]


def build_template(entry, idx):
    """Eine Vorlage bauen: kurz, klar, sofort weiterbaubar."""
    tid, mode, speed, keys, colors, music = entry
    rng = random.Random(core.fnv1a("gdtpl/%s" % tid))
    L = LB()
    ctx = {"speed": speed, "hard": 0, "rng": rng, "top": 9, "lo": 2,
           "mode": CUBE}
    if not keys:
        # Die erste Kachel ist bewusst LEER: keine Objekte, keine Stacheln -
        # daran erkennt die Oberfläche "ganz von vorn anfangen".
        return level({"id": "tpl-" + tid, "name": "tpl." + tid,
                      "category": "template", "tpl": tid, "difficulty": 0,
                      "stars": 0, "speed": speed, "mode": CUBE,
                      "bg": list(colors[0]), "ground": list(colors[1]),
                      "music": music, "bpm": 130, "length": 80}, L)
    x = sec.intro(L, 0, ctx)
    coin_spots = []
    if mode != CUBE or speed != 1:
        top = 10 if mode == SHIP else (9 if mode == UFO else
                                       (8 if mode == WAVE else 7))
        x += sec.transition(L, x, ctx, mode, speed, colors=colors, top=top)
        ctx["top"], ctx["mode"], ctx["speed"] = top, mode, speed
    for key in keys:
        s = sec.SECTIONS[key]
        w, coins = s.build(L, x, ctx)
        for cx, cy in coins:
            coin_spots.append((cx, cy, 0))
        x += w + sec.beat_for(ctx["speed"])
    if ctx["mode"] != CUBE:
        x += sec.transition(L, x, ctx, CUBE, ctx["speed"], colors=colors)
        ctx["mode"] = CUBE
    x += sec.outro(L, x, ctx)
    # Vorlagen bekommen bewusst keine Münzen - die setzt, wer sie ausbaut.
    return level({"id": "tpl-" + tid, "name": "tpl." + tid, "category": "template",
                  "tpl": tid, "difficulty": 0, "stars": 0, "speed": speed,
                  "mode": CUBE, "bg": list(colors[0]), "ground": list(colors[1]),
                  "music": music, "bpm": 130, "length": max(80, x + 10)}, L)


# ---------------------------------------------------------------------------
#  Lösen
# ---------------------------------------------------------------------------

def solve_level(d, coins=False, budget=(400000, 20.0),
                hard_budget=(1200000, 45.0)):
    """Löst ein Level (standardmäßig ohne Münz-Zwang). Gibt (toggles, info)."""
    lv = core.Level(d)
    tg, info = solver.solve(lv, need_coins=coins, max_nodes=budget[0],
                            time_limit=budget[1])
    if tg is None and info.get("reason") == "limit":
        tg, info = solver.solve(lv, need_coins=coins, max_nodes=hard_budget[0],
                                time_limit=hard_budget[1])
    return tg, info


def build_and_solve(cat, idx, proofs, force=False, solve=True):
    """Baut ein Level, löst es und legt die Münzen auf die bewiesene Bahn.

    Ablauf: Level ohne Münzen lösen (schnell), drei Münzen auf die Bahn legen,
    dieselbe Lösung mit Münzen nachprüfen (auch um ±1 Schritt versetzt).
    Klappt das nicht, wird das Level entschärft und danach neu gewürfelt.
    """
    lid = "x-%s-%02d" % (cat, idx)
    have = proofs.get(lid)
    if not force and have:
        d, _plan = compose(cat, idx, salt=have.get("salt", 0),
                           relax=have.get("relax", 0))
        d["id"] = lid
        d = _with_coins(d, have.get("coins") or [])
        if have.get("content") == core.content_hash(d) and _verify(d, have["toggles"]):
            return d, have, "geprüft"
    if not solve:
        return None, None, "ungelöst"
    for salt in range(3):
        for relax in range(3):
            d, _plan = compose(cat, idx, salt=salt, relax=relax)
            d["id"] = lid
            t0 = time.time()
            tg, info = solve_level(d)
            if tg is None:
                continue
            for offset in range(4):
                full = coins_on_path(d, tg, offset=offset)
                if _coin_count(full) < 3:
                    continue
                if _verify(full, tg):
                    st, _ = core.run_toggles(core.Level(full), tg)
                    proof = {"toggles": tg, "steps": st.step,
                             "hash": core.state_hash(st),
                             "content": core.content_hash(full),
                             "salt": salt, "relax": relax,
                             "coins": [[o[1], o[2]] for o in full["objects"]
                                       if o[0] == "coin"]}
                    return full, proof, "gelöst in %.1f s (%d Knoten)" % (
                        time.time() - t0, info.get("nodes", 0))
            # Münzen wollen nicht passen: mit Münz-Zwang weitersuchen
            full = coins_on_path(d, tg, offset=0)
            tg2, info2 = solve_level(full, coins=True)
            if tg2 is not None and _verify(full, tg2):
                st, _ = core.run_toggles(core.Level(full), tg2)
                proof = {"toggles": tg2, "steps": st.step,
                         "hash": core.state_hash(st),
                         "content": core.content_hash(full),
                         "salt": salt, "relax": relax,
                         "coins": [[o[1], o[2]] for o in full["objects"]
                                   if o[0] == "coin"]}
                return full, proof, "gelöst (mit Münz-Suche, %d Knoten)" %                     info2.get("nodes", 0)
    return None, None, "nicht lösbar"


def _with_coins(d, coins):
    """Setzt gespeicherte Münz-Positionen wieder ins Level."""
    if not coins:
        return d
    out = dict(d)
    out["objects"] = [list(o) for o in d["objects"]] +         [["coin", int(cx), int(cy), 0] for cx, cy in coins]
    return core.normalize_level(out)


def _coin_count(d):
    return sum(1 for o in d["objects"] if o[0] == "coin")


def _verify(d, toggles):
    """Lösung gültig - auch einen Schritt zu früh und zu spät?"""
    lv = core.Level(d)
    return all(solver.verify(lv, toggles, sh)[0] for sh in (-1, 0, 1))


# ---------------------------------------------------------------------------
#  Schreiben
# ---------------------------------------------------------------------------

def index_entry(d):
    """Kompakter Eintrag für die Liste im Spiel (ohne Objekte)."""
    return {"id": d["id"], "name": d["name"], "cat": d.get("category", ""),
            "difficulty": d.get("difficulty", 0), "stars": d.get("stars", 0),
            "speed": d.get("speed", 1), "length": d.get("length", 0),
            "bg": d.get("bg"), "ground": d.get("ground")}


def write_all(levels, templates):
    payload = {"_info": "Erzeugt von devtools/build_geodash_extra.py - "
                        "nicht von Hand bearbeiten.",
               "v": 1, "levels": levels}
    with open(OUT_JSON, "w", encoding="utf-8", newline="\n") as f:
        f.write(_dump_levels(payload))
    with open(OUT_INDEX_JS, "w", encoding="utf-8", newline="\n") as f:
        f.write("// Automatisch erzeugt (devtools/build_geodash_extra.py) - "
                "Liste für den Reiter \"Weitere\".\n")
        f.write("(function () {\n  \"use strict\";\n  window.PG.gdExtraIndex = ")
        f.write(json.dumps([index_entry(d) for d in levels],
                           separators=(",", ":"), ensure_ascii=False))
        f.write(";\n})();\n")
    with open(OUT_FULL_JS, "w", encoding="utf-8", newline="\n") as f:
        f.write("// Automatisch erzeugt (devtools/build_geodash_extra.py) - "
                "Volldaten, werden beim ersten Spielen nachgeladen.\n")
        f.write("(function () {\n  \"use strict\";\n  window.PG.gdExtra = ")
        f.write(json.dumps({d["id"]: d for d in levels},
                           separators=(",", ":"), ensure_ascii=False))
        f.write(";\n  if (window.PG.gdExtraReady) window.PG.gdExtraReady();\n})();\n")
    tpl_payload = {"_info": "Erzeugt von devtools/build_geodash_extra.py - "
                            "Vorlagen für eigene Level.",
                   "v": 1, "levels": templates}
    with open(OUT_TPL_JSON, "w", encoding="utf-8", newline="\n") as f:
        f.write(_dump_levels(tpl_payload))
    with open(OUT_TPL_JS, "w", encoding="utf-8", newline="\n") as f:
        f.write("// Automatisch erzeugt (devtools/build_geodash_extra.py) - "
                "Vorlagen für eigene Level.\n")
        f.write("(function () {\n  \"use strict\";\n  window.PG.gdTemplates = ")
        f.write(json.dumps(templates, separators=(",", ":"), ensure_ascii=False))
        f.write(";\n})();\n")


def main():
    args = sys.argv[1:]
    only = args[args.index("--only") + 1] if "--only" in args else None
    cat_filter = args[args.index("--cat") + 1] if "--cat" in args else None
    solve = "--no-solve" not in args
    force = "--force" in args
    strict = "--strict" in args

    try:
        with open(OUT_PROOFS, "r", encoding="utf-8") as f:
            proofs = json.load(f)
    except (OSError, ValueError):
        proofs = {}

    levels = []
    missing = []
    t_start = time.time()
    for cat in CAT_ORDER:
        cfg = CATS[cat]
        for idx in range(1, cfg["n"] + 1):
            lid = "x-%s-%02d" % (cat, idx)
            # Bei --cat/--only die übrigen Level unverändert aus dem Beweis
            # wieder aufbauen, damit ein Teil-Lauf die Bibliothek nicht kürzt.
            skip = (cat_filter and cat != cat_filter) or (only and lid != only)
            if skip:
                have = proofs.get(lid)
                if not have:
                    continue
                d, _plan = compose(cat, idx, salt=have.get("salt", 0),
                                   relax=have.get("relax", 0))
                d["id"] = lid
                levels.append(_with_coins(d, have.get("coins") or []))
                continue
            d, proof, note = build_and_solve(cat, idx, proofs, force=force,
                                             solve=solve)
            if d is None:
                missing.append(lid)
                print("  %-14s FEHLT (%s)" % (lid, note))
                continue
            if proof:
                proofs[lid] = proof
            levels.append(d)
            print("  %-14s %-22s %s" % (lid, d["name"], note))

    templates = [build_template(e, i) for i, e in enumerate(TEMPLATES)]
    for tpl in templates:
        if not tpl["objects"]:
            continue
        tid = tpl["id"]
        have = proofs.get(tid)
        if not force and have and have.get("content") == core.content_hash(tpl):
            ok = all(solver.verify(core.Level(tpl), have["toggles"], sh)[0]
                     for sh in (-1, 0, 1))
            if ok:
                tpl["verified"] = core.content_hash(tpl)
                print("  %-14s Vorlage geprüft" % tid)
                continue
        if not solve:
            continue
        tg, info = solve_level(tpl)
        if tg is not None and all(solver.verify(core.Level(tpl), tg, sh)[0]
                                  for sh in (-1, 0, 1)):
            s, _ = core.run_toggles(core.Level(tpl), tg)
            proofs[tid] = {"toggles": tg, "steps": s.step,
                           "hash": core.state_hash(s),
                           "content": core.content_hash(tpl)}
            tpl["verified"] = core.content_hash(tpl)
            print("  %-14s Vorlage gelöst (%d Knoten)" % (tid, info.get("nodes", 0)))
        else:
            print("  %-14s Vorlage NICHT gelöst" % tid)
            missing.append(tid)

    if "--preview" in args:
        folder = args[args.index("--preview") + 1]
        os.makedirs(folder, exist_ok=True)
        from build_geodash_levels import preview
        preview(levels, proofs, folder)

    write_all(levels, templates)
    with open(OUT_PROOFS, "w", encoding="utf-8", newline="\n") as f:
        json.dump(proofs, f, indent=1)
        f.write("\n")
    print("\n%d Level, %d Vorlagen in %.1f s%s" % (
        len(levels), len(templates), time.time() - t_start,
        "" if not missing else "  (fehlen: %s)" % ", ".join(missing)))
    print("geschrieben:", OUT_JSON, OUT_INDEX_JS, OUT_FULL_JS, OUT_TPL_JSON,
          OUT_TPL_JS, OUT_PROOFS)
    return 1 if (missing and strict) else 0


if __name__ == "__main__":
    raise SystemExit(main())
