# -*- coding: utf-8 -*-
"""
arcade_casino_audit.py
======================
Gesamt-Audit für das Arcade-&-Casino-Update (Crossy Road, Geometry Dash,
Battleship, Casino + Ausbau von Tetris, 2048, Schach, Wordle, Sudoku).

Prüft zuerst das gemeinsame Fundament selbst:

- Eingabe-Schicht (main.py): Tastenwiederholung wird erkannt (Windows-Stil:
  nur weitere Presses; X11-Stil: Release+Press-Paare), KEYUP trägt den keysym
  des ersten Drucks, Pause lässt gehaltene Tasten los, gehaltenes ESC schaltet
  die Pause nicht mehrfach um.
- seedrand.py liefert bitgenau dieselbe Folge wie web/js/games/seedrand.js
  (per Node, falls installiert).
- settings.SCHEMAS: gültige Werte bleiben, ungültige fallen auf den Standard.
- store.write_json_atomic: schreibt, legt .bak an, liest bei kaputter Datei
  die Sicherung.
- audio: Musik-Schleife folgt ihrem Besitzer (Pause, Wechsel).
- Sprachdateien: alle 14 lang- und LamaWiki-Dateien haben identische
  Schlüssel bzw. Seiten-ids; jede Spielklasse hat eine Wiki-Seite.
- Vorspiel-Screen: für alle Spiele × 14 Sprachen × 5 Auflösungen liegen die
  Knöpfe unter dem Untertitel, über der Fußzeile und überlappen nicht.

Danach startet es jeden Einzeltest ``tests/audit_*.py`` als eigenen Prozess
und fasst die Ergebnisse zusammen. Aufruf:

    .venv/Scripts/python.exe tests/arcade_casino_audit.py [--only-core]
"""

import glob
import json
import os
import shutil
import subprocess
import sys
import tempfile

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except (AttributeError, OSError):
    pass

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, REPO)
TMP = tempfile.mkdtemp(prefix="pgz-audit-")

import pygame
pygame.init()
pygame.display.set_mode((64, 64))

import store
store._PATH = os.path.join(TMP, "mem.json")
import settings as settings_mod
settings_mod._PATH = os.path.join(TMP, "settings.json")
import i18n
i18n.init()

FAILS = []


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + str(detail)) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


# ---------------------------------------------------------------- Eingabe

def audit_input():
    print("\n== Eingabe-Schicht (main.py)")
    import main

    class Spy:
        is_menu = False
        wants_escape = False
        game_over = False
        paused = False

        def __init__(self):
            self.log = []

        def handle_event(self, ev):
            self.log.append((ev.kind, ev.key, ev.repeat))

    class Ev:
        def __init__(self, keysym, keycode, char=""):
            self.keysym, self.keycode, self.char = keysym, keycode, char

    app = object.__new__(main.App)
    app._held_keys, app._pending_keyups = {}, {}
    g = Spy()
    app.current = g

    app._on_key(Ev("space", 32, " "))
    app._on_key(Ev("space", 32, " "))
    app._on_key_up(Ev("space", 32))
    app._flush_keyups()
    check(g.log == [("keydown", "space", False), ("keydown", "space", True),
                    ("keyup", "space", False)], "Windows-Wiederholung erkannt", g.log)

    g.log.clear()
    app._on_key(Ev("w", 87, "w"))
    app._on_key_up(Ev("w", 87))
    app._on_key(Ev("w", 87, "w"))
    app._on_key_up(Ev("W", 87))
    app._flush_keyups()
    check(g.log == [("keydown", "w", False), ("keydown", "w", True),
                    ("keyup", "w", False)], "X11-Paare + Shift -> gleicher keysym", g.log)

    g.log.clear()
    app._on_key(Ev("Up", 38))
    app._on_key(Ev("Escape", 27))
    check(g.paused and ("keyup", "Up", False) in g.log, "Pause lässt Tasten los", g.log)
    app._on_key(Ev("Escape", 27))
    check(g.paused, "gehaltenes ESC schaltet nicht zurück")
    app._on_key_up(Ev("Escape", 27))
    app._flush_keyups()
    app._on_key(Ev("Escape", 27))
    check(not g.paused, "ESC erneut gedrückt setzt fort")


# ---------------------------------------------------------------- seedrand

def audit_seedrand():
    print("\n== seedrand (Python == JS)")
    import seedrand as S
    r = S.Rand(S.seed_from("audit", 7, True))
    seq = [r.next_u32() for _ in range(6)]
    roll = r.randint(-5, 5)
    arr = list(range(12))
    r.shuffle(arr)
    py = [*seq, roll, ",".join(map(str, arr)),
          S.daily_seed("wordle", "2026-09-16"), S.day_index("2026-09-16")]
    node = shutil.which("node")
    if not node:
        print("  (Node fehlt - Vergleich übersprungen)")
        return
    js_path = os.path.join(REPO, "web", "js", "games", "seedrand.js").replace("\\", "/")
    script = (
        "global.window={PG:{}};require(%r);const S=window.PG.seedrand;"
        "const r=new S.Rand(S.seedFrom('audit',7,true));const o=[];"
        "for(let i=0;i<6;i++)o.push(r.nextU32());const a=[...Array(12).keys()];"
        "o.push(r.randint(-5,5));r.shuffle(a);o.push(a.join(','));"
        "o.push(S.dailySeed('wordle','2026-09-16'),S.dayIndex('2026-09-16'));"
        "console.log(JSON.stringify(o.map(String)))" % js_path)
    out = subprocess.run([node, "-e", script], capture_output=True, text=True)
    js = json.loads(out.stdout or "[]")
    check(js == [str(x) for x in py], "gleiche Zahlenfolge", (py, js, out.stderr[-300:]))


# ---------------------------------------------------------------- settings/store

def audit_settings_store():
    print("\n== settings.SCHEMAS + store (atomar)")
    d = settings_mod._merge_defaults({
        "tetris": {"das": 9999, "solo": "sprint", "ghost": "ja"},
        "casino": {"line_bet": True, "roulette_chip": 25},
        "chess": {"clock": "3+2", "chess960": 1},
        "wordle": {"length": 6},
    })
    check(d["tetris"]["das"] == 400 and d["tetris"]["solo"] == "sprint"
          and d["tetris"]["ghost"] is True, "Tetris: begrenzt/gewählt/ungültig", d["tetris"])
    check(d["casino"]["line_bet"] == 1 and d["casino"]["roulette_chip"] == 25,
          "Casino: bool zählt nicht als 1", d["casino"])
    check(d["chess"]["clock"] == "3+2" and d["chess"]["chess960"] is False,
          "Schach: Uhr ok, 1 ist kein bool", d["chess"])
    for sec, rules in settings_mod.SCHEMAS.items():
        for key in rules:
            ok = key in settings_mod.DEFAULTS.get(sec, {})
            if not ok:
                check(False, "SCHEMAS %s.%s hat Standardwert" % (sec, key))
    check(True, "alle SCHEMAS-Schlüssel haben Standardwerte")

    path = os.path.join(TMP, "atomic.json")
    check(store.write_json_atomic(path, {"a": 1}), "write_json_atomic schreibt")
    store.write_json_atomic(path, {"a": 2})
    with open(path + ".bak", encoding="utf-8") as f:
        check(json.load(f) == {"a": 1}, ".bak enthält Vorversion")
    store.save_section("audit", {"x": 1})
    store.save_section("audit", {"x": 2})
    with open(store._PATH, "w", encoding="utf-8") as f:
        f.write("{kaputt")
    check(store.load_section("audit") == {"x": 1}, "kaputte mem.json -> Sicherung gelesen")


# ---------------------------------------------------------------- audio

def audit_music():
    print("\n== Musik-Schleifen")
    import audio
    audio.init()

    class Owner:
        paused = False
    o = Owner()
    kick = audio.note_bytes(0, 0.2, "kick", 0.8)
    voice = audio.render_voice([(i * 0.5, kick) for i in range(8)], 4)
    check(len(voice) == audio.mixer_rate() * 4 * 2, "render_voice-Länge")
    started = audio.music_play([(voice, 1.0)], o, {"sound": True, "volume": 0.5})
    if not started:
        print("  (kein Audiogerät - Rest übersprungen)")
        return
    audio.music_tick(o)
    check(audio.music_playing(o), "läuft beim Besitzer")
    o.paused = True
    audio.music_tick(o)
    check(audio._music["paused"], "pausiert mit dem Besitzer")
    audio.music_tick(object())
    check(not audio.music_playing(), "endet beim Wechsel")


# ---------------------------------------------------------------- i18n / Wiki

def audit_languages():
    print("\n== Sprachdateien + LamaWiki")
    sys.path.insert(0, os.path.join(REPO, "devtools"))
    import merge_staging as ms
    key_sets, page_sets = {}, {}
    for code in ms.LANGS:
        with open(ms.lang_path("lang", code), encoding="utf-8") as f:
            key_sets[code] = set(json.load(f))
        with open(ms.lang_path("lamawiki", code), encoding="utf-8") as f:
            page_sets[code] = [p["id"] for p in json.load(f)["pages"]]
    base = key_sets["de"]
    for code, keys in key_sets.items():
        if keys != base:
            check(False, "lang/%s gleiche Schlüssel wie de" % code,
                  sorted(keys ^ base)[:8])
    check(True, "%d Schlüssel je Sprache verglichen" % len(base))
    for code, ids in page_sets.items():
        if ids != page_sets["de"]:
            check(False, "lamawiki/%s gleiche Seiten wie de" % code)
    from games import ALL_GAMES
    with open(ms.lang_path("lamawiki", "de"), encoding="utf-8") as f:
        linked = {p.get("game") for p in json.load(f)["pages"]}
    missing = [c.__name__ for c in ALL_GAMES if c.__name__ not in linked]
    check(not missing, "jedes Spiel hat eine Wiki-Seite", missing)
    import achievements
    ach_keys = []
    for sid, _icon, _key, _target in achievements.SPECIALS:
        ach_keys += ["ach.%s.name" % sid, "ach.%s.desc" % sid]
    miss = [k for k in ach_keys if k not in base]
    check(not miss, "alle Spezial-Erfolge übersetzt", miss)


# ---------------------------------------------------------------- Vorspiel-Screen

def audit_pregame():
    print("\n== Vorspiel-Screen: alle Spiele × 14 Sprachen × 5 Auflösungen")
    import menu
    from games import ALL_GAMES

    class FakeApp:
        settings = json.loads(json.dumps(settings_mod.DEFAULTS))

        def launch_game(self, *a):
            pass

        def back_to_menu(self):
            pass

        def show_screen(self, s):
            pass

    bad = []
    for entry in i18n.AVAILABLE:
        code = entry[0] if isinstance(entry, (tuple, list)) else entry
        i18n.set_language(code, persist=False)
        for w, h in [wh for _, wh in settings_mod.RESOLUTIONS]:
            surf = pygame.Surface((w, h))
            for cls in ALL_GAMES:
                scr = menu.PreGameScreen(surf, w, h, FakeApp(), cls)
                rs = scr.rects
                if any(r.top < 128 or r.bottom > h - 30 or r.left < 0 or r.right > w
                       for r in rs):
                    bad.append((code, w, cls.__name__, "Rand"))
                elif any(rs[i].colliderect(rs[j]) for i in range(len(rs))
                         for j in range(i + 1, len(rs))):
                    bad.append((code, w, cls.__name__, "Überlappung"))
                else:
                    for i, (label, _) in enumerate(scr.buttons):
                        if scr._button_font(i).size(label)[0] > rs[i].w - 8:
                            bad.append((code, w, cls.__name__, label))
                            break
    i18n.set_language("de", persist=False)
    check(not bad, "alle Knöpfe passen", bad[:10])


# ---------------------------------------------------------------- Einzeltests

def run_game_audits():
    print("\n== Einzeltests tests/audit_*.py")
    results = []
    for path in sorted(glob.glob(os.path.join(REPO, "tests", "audit_*.py"))):
        name = os.path.basename(path)
        proc = subprocess.run([sys.executable, path], cwd=REPO, capture_output=True,
                              text=True, encoding="utf-8", errors="replace")
        ok = proc.returncode == 0
        results.append((name, ok))
        check(ok, name, (proc.stdout + proc.stderr)[-600:])
    return results


def main():
    audit_input()
    audit_seedrand()
    audit_settings_store()
    audit_music()
    audit_languages()
    audit_pregame()
    if "--only-core" not in sys.argv:
        run_game_audits()
    shutil.rmtree(TMP, ignore_errors=True)
    print("\n%s: %d Fehler" % ("FEHLGESCHLAGEN" if FAILS else "ALLES OK", len(FAILS)))
    for f in FAILS:
        print("  -", f)
    sys.exit(1 if FAILS else 0)


if __name__ == "__main__":
    main()
