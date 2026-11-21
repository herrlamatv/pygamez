# -*- coding: utf-8 -*-
"""
vergleich.py
============
Misst, ob ein Spiel der Rust-Fassung **dasselbe Bild** zeichnet wie die
Python-Fassung.

Beide laufen mit demselben Startwert fuer ``random``, derselben eingefrorenen
Uhr (60 Bilder je Sekunde), denselben Tasten und einem frischen, gleich
befuellten Datenordner. Python faehrt das Spiel so, wie ``main.App._frame``
es tut (update, draw, Highscore-Banner, ``ui.draw_fx``, Erfolgs-Einblendung);
Rust faehrt seinen Pruefstand (``PYGAMEZ_SHOT``). Danach wird Pixel fuer
Pixel verglichen.

    .venv/Scripts/python.exe rust/tools/vergleich.py pong
    .venv/Scripts/python.exe rust/tools/vergleich.py pong --frames 300 --keys Up,Up,Down
    .venv/Scripts/python.exe rust/tools/vergleich.py all

Endet mit 1, sobald ein Spiel abweicht. Bilder und Unterschiedsbild liegen
im Ordner ``--out`` (Standard: ein Temp-Ordner, wird genannt).
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

HIER = os.path.dirname(os.path.abspath(__file__))
RUST = os.path.dirname(HIER)
WURZEL = os.path.dirname(RUST)
# Bauordner: rust.output/ - oder ein eigener per CARGO_TARGET_DIR (parallele
# Bauten mehrerer Arbeitsstraenge; dieselbe Variable wie fuer cargo).
ZIEL = (os.path.abspath(os.environ["CARGO_TARGET_DIR"]) if os.environ.get("CARGO_TARGET_DIR")
        else os.path.join(os.path.dirname(WURZEL), os.path.basename(WURZEL), "rust.output"))
EXE = os.path.join(ZIEL, "release", "pygamez.exe")

sys.path.insert(0, WURZEL)
os.environ["SDL_VIDEODRIVER"] = "dummy"
os.environ["SDL_AUDIODRIVER"] = "dummy"
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")

MEM_START = {"mem": {"lang": "de"}}
# Optionaler Vorlagen-Ordner (--daten): seine Dateien kommen in jeden frischen
# Datenordner (z.B. aus rust/tools/testdaten.py).
VORLAGE = None


def datenordner(basis, name):
    d = os.path.join(basis, name)
    if os.path.exists(d):
        shutil.rmtree(d)
    os.makedirs(d)
    with open(os.path.join(d, "mem.json"), "w", encoding="utf-8") as f:
        json.dump(MEM_START, f)
    if VORLAGE:
        for name in os.listdir(VORLAGE):
            quelle = os.path.join(VORLAGE, name)
            if os.path.isfile(quelle):
                shutil.copyfile(quelle, os.path.join(d, name))
    return d


def tick_folge(frames):
    uhr = 1000
    out = []
    for i in range(frames):
        out.append(uhr)
        uhr += 1000 // 60 + (1 if i % 3 == 0 else 0)
    return out, uhr


def ereignis(game, k, InputEvent):
    """Einen Eintrag aus ``--keys`` zustellen - wie ``ereignis`` in main.rs.

    ``Taste`` = druecken und loslassen, ``+Taste``/``-Taste`` = nur druecken/
    nur loslassen, ``@x:y`` = Linksklick, ``@x:y:3`` = Rechtsklick,
    ``~x:y`` = Maus bewegen (Spielkoordinaten), ``^dx:dy`` = relative
    Mausbewegung (MOUSEREL, Pointer-Capture), ``<x:y`` / ``>x:y`` = Maustaste
    nur druecken / nur loslassen (Ziehen mit ``~x:y`` dazwischen),
    ``%x:y:d`` = Mausrad bei (x, y) um d Rasten (+ = hoch, ohne d: 1),
    ``*Taste`` = Tastenwiederholung des Systems (KEYDOWN mit ``repeat=True``,
    ohne Loslassen - wie eine gehaltene Taste nach ``+Taste``).
    """
    def zahlen(rest):
        out = []
        for z in rest.split(":"):
            try:
                out.append(int(z))
            except ValueError:
                pass
        return out

    if len(k) > 1 and k[0] in "@~^<>%":
        z = zahlen(k[1:])
        if len(z) < 2 or game.paused:
            return
        if k[0] == "%":
            # main._on_wheel: WHEEL mit pos und delta (Rasten)
            game.handle_event(InputEvent(InputEvent.WHEEL, pos=(z[0], z[1]),
                                         delta=z[2] if len(z) > 2 else 1))
            return
        if k[0] in "<>":
            art = InputEvent.MOUSEDOWN if k[0] == "<" else InputEvent.MOUSEUP
            game.handle_event(InputEvent(art, pos=(z[0], z[1]), button=z[2] if len(z) > 2 else 1))
            return
        if k[0] == "^":
            game.handle_event(InputEvent(InputEvent.MOUSEREL, rel=(z[0], z[1])))
            return
        if k[0] == "~":
            game.handle_event(InputEvent(InputEvent.MOUSEMOVE, pos=(z[0], z[1])))
            return
        knopf = z[2] if len(z) > 2 else 1
        game.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=(z[0], z[1]), button=knopf))
        if not game.paused:
            game.handle_event(InputEvent(InputEvent.MOUSEUP, pos=(z[0], z[1]), button=knopf))
        return
    runter = hoch = True
    wdh = False
    if len(k) > 1 and k[0] == "+":
        k, hoch = k[1:], False
    elif len(k) > 1 and k[0] == "-":
        k, runter = k[1:], False
    elif len(k) > 1 and k[0] == "*":
        k, hoch, wdh = k[1:], False, True
    ch = k if len(k) == 1 else ""
    if k == "Escape":
        # main._on_key: gehaltenes Escape wiederholt nichts
        if not runter or wdh:
            return
        if getattr(game, "is_menu", False) or getattr(game, "wants_escape", False):
            game.handle_event(InputEvent(InputEvent.KEYDOWN, key="Escape"))
        elif not game.game_over:
            game.paused = not game.paused
        return
    if runter and not game.paused:
        game.handle_event(InputEvent(InputEvent.KEYDOWN, key=k, char=ch, repeat=wdh))
    if hoch and not game.paused:
        game.handle_event(InputEvent(InputEvent.KEYUP, key=k))


def python_lauf(key, mode, seed, frames, keys, data_dir, ziel):
    import pygame
    import store
    import settings as settings_mod
    store._DIR = data_dir
    store._PATH = os.path.join(data_dir, "mem.json")
    store._LEGACY_HS_PATH = os.path.join(data_dir, "highscores.json")
    settings_mod._PATH = os.path.join(data_dir, "settings.json")
    # replay.json und ugc.json liegen in Python neben dem Modul (also im Repo
    # mit den echten Daten des Nutzers) - im Pruefstand in den Datenordner,
    # wie die Rust-Fassung (store::data_dir).
    import replay
    import ugc
    replay._PATH = os.path.join(data_dir, "replay.json")
    ugc._PATH = os.path.join(data_dir, "ugc.json")
    # Keine echten Datei-Dialoge im Pruefstand (Rust: filepick::available()
    # ist dort False) - und nie in den echten Downloads-Ordner schreiben.
    import filepick
    filepick.available = lambda: False
    # Nur den Ordner umlenken (wie Rust: <Datenordner>/Downloads, angelegt
    # wie filepick.downloads_dir) - to_downloads selbst bleibt das echte.
    _downloads = os.path.join(data_dir, "Downloads")

    def _downloads_dir():
        os.makedirs(_downloads, exist_ok=True)
        return _downloads
    filepick.downloads_dir = _downloads_dir

    pygame.init()
    pygame.display.set_mode((1, 1))
    uhr = [0]
    pygame.time.get_ticks = lambda: uhr[0]

    import i18n
    i18n.init()
    import ui
    settings = settings_mod.load_settings()
    ui.set_theme(settings.get("theme", ui.DEFAULT_THEME))
    import achievements
    import stats
    import highscore
    achievements.backfill_silent()
    # Wie main.App.__init__ (und die Rust-App beim Start): ohne Willkommens-
    # Bildschirm merkt sich _maybe_show_level_intro, dass die Startlevel-
    # Karte gesehen ist - das landet in mem.json.
    if i18n.has_language():
        flags = store.load_section("progress")
        if not flags.get("intro_seen"):
            flags["intro_seen"] = True
            store.save_section("progress", flags)
    from game_base import InputEvent
    from games import ALL_GAMES

    bildschirm = key.startswith("screen:")
    if not bildschirm:
        cls = [c for c in ALL_GAMES if c.highscore_key == key][0]
    # Wie main.App: logische Flaeche aus den Einstellungen (Standard 640x480;
    # andere ueber --daten mit einer settings.json).
    res = settings.get("resolution", [640, 480])
    w, h = int(res[0]), int(res[1])
    canvas = pygame.Surface((w, h))
    ticks, _ = tick_folge(frames)
    uhr[0] = ticks[0]
    import random
    random.seed(seed)
    # random.Random() ohne Startwert: aus dem globalen Generator ableiten
    # (Rust tut im Pruefstand dasselbe, siehe py::Random::unseeded).
    _orig_seed = random.Random.seed

    def _seed(self, a=None, version=2):
        if a is None and self is not random._inst:
            a = random.getrandbits(64)
        _orig_seed(self, a, version)
    random.Random.seed = _seed
    # Wanduhr der Spiel-Module an die feste Bild-Uhr binden (KI-Zeitbudgets
    # wie in Muehle/Schach waeren sonst vom Rechnertempo abhaengig). Rust tut
    # im Pruefstand dasselbe (py::time_time / py::perf_counter).
    import time as _echte_zeit

    class _FesteZeit:
        def time(self):
            return uhr[0] / 1000.0
        perf_counter = monotonic = time

        def __getattr__(self, name):
            return getattr(_echte_zeit, name)

    for _name, _mod in list(sys.modules.items()):
        if _name.startswith("games.") and _mod is not None:
            for _attr in ("time", "_time"):
                if getattr(_mod, _attr, None) is _echte_zeit:
                    setattr(_mod, _attr, _FesteZeit())
    if bildschirm:
        game = bildschirm_bauen(key, canvas, w, h, settings, ALL_GAMES)
        game.paused = False
    else:
        game = cls(canvas, w, h, mode=mode, game_settings=settings)
        game.player_ids = [None, None]
        game.paused = False
        stats.game_started(key)
        achievements.on_game_started(key)
    ui.begin_transition()

    attrappe = app_attrappe(canvas, w, h, settings, ALL_GAMES)

    def banner():
        attrappe._draw_highscore_overlay(game)

    def speichern():
        hs, rekord = highscore.update_highscore(game.highscore_key, game.score)
        game._hs_value = hs
        game._hs_record = rekord
        if rekord:
            stats.record_broken(game.highscore_key)
        stats.score_result(game.highscore_key, game.score, rekord)
        achievements.on_highscore(game.highscore_key, hs)
        achievements.check_stats()

    dt = 1.0 / 60.0
    # Wie main._frame: Taste P am Rundenende oeffnet den Replay-Screen; das
    # Spiel bleibt "geparkt" und kehrt beim Schliessen zurueck.
    halter = [game]

    class _ReplayApp:
        pass
    replay_app = _ReplayApp()
    replay_app.settings = settings

    def _zurueck_zu(spiel):
        def zurueck():
            halter[0] = spiel
            ui.begin_transition()
        return zurueck
    for i in range(frames):
        uhr[0] = ticks[i]
        game = halter[0]
        if i >= 2 and i - 2 < len(keys) and keys[i - 2]:
            ereignis(game, keys[i - 2], InputEvent)
        game = halter[0]
        wunsch = getattr(game, "replay_request", None)
        if wunsch:
            game.replay_request = None
            from replayview import ReplayScreen
            game = halter[0] = ReplayScreen(canvas, w, h, replay_app,
                                            on_close=_zurueck_zu(game),
                                            pending=wunsch, game=wunsch.get("game"))
            game.paused = False
            ui.begin_transition()
        if not game.paused and not game.game_over:
            game.update(dt)
            if not bildschirm and not getattr(game, "is_menu", False):
                stats.add_playtime(game.highscore_key, dt)
        game.draw()
        if game.paused:
            attrappe._draw_pause_overlay()
        if game.game_over and not getattr(game, "_hs_saved", False):
            speichern()
            game._hs_saved = True
            if getattr(game, "_hs_record", False):
                ui.spawn_confetti(w, h)
        if game.game_over:
            game._was_over = True
        else:
            game._hs_saved = False
            if getattr(game, "_resume_same_game", False):
                game._resume_same_game = False
                game._was_over = False
            elif getattr(game, "_was_over", False):
                game._was_over = False
                game._result_reported = False
                game._duel_reported = False
                stats.game_started(game.highscore_key)
                achievements.on_game_started(game.highscore_key)
        if (game.game_over and not getattr(game, "is_menu", False)
                and getattr(game, "show_highscore_banner", True)):
            banner()
        ui.draw_fx(canvas, w, h, dt)
        achievements.draw_toasts(canvas, w, h, dt, settings)
    pygame.image.save(canvas, ziel)
    # Wie main.App.beenden - die Rust-App ruft es am Ende des Pruefstands
    # ebenso (Highscore, on_exit des Spiels, Statistik-Sitzung schliessen und
    # schreiben). Erst nach dem Foto: es geht nur um die gespeicherten Daten.
    attrappe._highscore_speichern(game)
    hook = getattr(game, "on_exit", None)
    if hook is not None:
        hook()
    stats.end_session()
    stats.flush()
    return canvas


def app_attrappe(canvas, w, h, settings, ALL_GAMES):
    """main.App ohne Tk - nur was Startbildschirm, Pause-Overlay und
    Highscore-Banner brauchen. So zeichnet der Pruefstand mit dem ECHTEN
    Code aus main.py statt mit einem Nachbau."""
    import pygame
    import highscore
    import main as main_mod

    class _App(main_mod.App):
        def __init__(self):
            pass
    m = _App()
    m.ui, m.pygame, m.canvas = sys.modules["ui"], pygame, canvas
    m.game_w, m.game_h, m.fps = w, h, 60
    m.settings = settings
    m._game_classes = list(ALL_GAMES)

    class _Liste:
        scores = highscore.load_highscores()

    class _Embed:
        def configure(self, **_kw):
            pass
    m.game_list, m.embed = _Liste(), _Embed()
    return m


def bildschirm_bauen(key, canvas, w, h, settings, ALL_GAMES):
    """``screen:<name>[:arg]`` - ein Bildschirm statt eines Spiels.

    Die App ist eine Attrappe mit genau dem, was die Bildschirme von ihr
    wollen (``refresh_profile``, ``_game_classes``, ``spiel_starten``).
    Schliessen fuehrt in Python nirgendwohin (Rust zeigt dann das Menue) -
    Laeufe sollen davor enden.
    """
    import achievements
    teile = key.split(":", 2)
    name = teile[1]
    arg = teile[2] if len(teile) > 2 else None

    class _App:
        pass
    app = _App()
    app.settings = settings
    app.canvas = canvas
    app.game_w, app.game_h = w, h
    app._game_classes = list(ALL_GAMES)
    app.refresh_profile = lambda: achievements.backfill_silent()
    app.spiel_starten = lambda cls: None
    app.back_to_menu = lambda: None

    def zu():
        return None
    if name == "menu":
        m = app_attrappe(canvas, w, h, settings, ALL_GAMES)

        class _Startbild:
            is_menu = True
            highscore_key = "_menu"
            game_over = False
            paused = False
            wants_escape = False

            def handle_event(self, ev):
                from game_base import InputEvent
                if ev.kind == InputEvent.MOUSEMOVE:
                    m._menu_motion(ev.pos)

            def update(self, dt):
                pass

            def draw(self):
                m._draw_menu_screen()
        return _Startbild()
    if name == "profiles":
        from profileview import ProfileScreen
        return ProfileScreen(canvas, w, h, app, on_close=zu)
    if name == "levelintro":
        from profileview import LevelIntroScreen
        return LevelIntroScreen(canvas, w, h, app, on_close=zu)
    if name == "pick":
        from profileview import PlayerPickScreen
        cls = [c for c in ALL_GAMES if c.highscore_key == arg][0] if arg else None
        return PlayerPickScreen(canvas, w, h, app, on_pick=lambda pid: None,
                                on_cancel=zu, game_cls=cls)
    if name == "progress":
        from progress import ProgressScreen
        scr = ProgressScreen(canvas, w, h, app, on_close=zu)
        if arg:
            scr.set_tab(arg)
        return scr
    if name == "wiki":
        from lamawiki import LamaWikiScreen
        return LamaWikiScreen(canvas, w, h, app, on_close=zu, page_id=arg)
    if name == "replays":
        from replayview import ReplayScreen
        return ReplayScreen(canvas, w, h, app, on_close=zu)
    raise SystemExit("unbekannter Bildschirm: " + name)


def rust_lauf(key, mode, seed, frames, keys, data_dir, ziel):
    env = dict(os.environ)
    # Die Liste geht ueber eine Datei: Windows begrenzt die Umgebung.
    liste = os.path.join(data_dir, "_keys.txt")
    with open(liste, "w", encoding="utf-8") as f:
        f.write(",".join(keys))
    env.update(PYGAMEZ_DATA_DIR=data_dir, PYGAMEZ_MODE=mode,
               PYGAMEZ_SEED=str(seed), PYGAMEZ_FRAMES=str(frames),
               PYGAMEZ_KEYS_FILE=liste, PYGAMEZ_SHOT=ziel)
    if key.startswith("screen:"):
        env["PYGAMEZ_SCREEN"] = key[len("screen:"):]
        env.pop("PYGAMEZ_GAME", None)
    else:
        env["PYGAMEZ_GAME"] = key
        env.pop("PYGAMEZ_SCREEN", None)
    env.pop("PYGAMEZ_FULL", None)
    env.pop("PYGAMEZ_KEYS", None)
    r = subprocess.run([EXE], env=env, capture_output=True, text=True)
    if r.returncode != 0 or not os.path.exists(ziel):
        raise RuntimeError("Rust-Lauf scheiterte: %s %s" % (r.stdout, r.stderr))


# Zeitstempel der Wanduhr in geteilten Dateien (Python: time.strftime) -
# Python- und Rust-Lauf liegen Sekunden auseinander. Maskiert werden nur die
# Ziffern, das Format bleibt Teil des Vergleichs.
STEMPEL = (re.compile(r"\d{4}([-.])\d{2}\1\d{2}"), re.compile(r"\d{2}:\d{2}(?::\d{2})?"),
           re.compile(r"\d{14}"))


def maskiert(text):
    for rx in STEMPEL:
        text = rx.sub(lambda m: re.sub(r"\d", "#", m.group(0)), text)
    return text


def _texte_vergleichen(name, a_pfad, b_pfad):
    """Fehlertext oder None (Zeitstempel maskiert)."""
    with open(a_pfad, "rb") as f:
        ta = maskiert(f.read().decode("utf-8", "replace"))
    with open(b_pfad, "rb") as f:
        tb = maskiert(f.read().decode("utf-8", "replace"))
    if ta == tb:
        return None
    if ta.replace("\r\n", "\n") == tb.replace("\r\n", "\n"):
        return "%s: nur Zeilenenden verschieden" % name
    if name == "ugc.json" or name.endswith(".lamapgzmap"):
        # Bekannt und harmlos: der Minigolf-Editor rechnet in Python mit
        # wechselnden Typen (round(v) -> 20, round(v, 1) -> 20.0), Rust mit
        # f64. Python laedt und spielt Rust-Bahnen (Kreuzprobe). Verglichen
        # wird deshalb der Inhalt - mit Reihenfolge, aber 20 == 20.0.
        try:
            ja = json.loads(ta, object_pairs_hook=list)
            jb = json.loads(tb, object_pairs_hook=list)
            if ja == jb:
                return None
        except ValueError:
            pass
    za, zb = ta.splitlines(), tb.splitlines()
    for i, (x, y) in enumerate(zip(za, zb)):
        if x != y:
            return "%s Zeile %d: py=%r rust=%r" % (name, i + 1, x.strip()[:100], y.strip()[:100])
    return "%s: %d gegen %d Zeilen" % (name, len(za), len(zb))


# Dateien im Datenordner, die nur der Pruefstand anlegt (oder Sicherungen).
NICHT_VERGLEICHEN = ("_keys.txt",)


def daten_vergleichen(py_daten, rs_daten):
    """Vergleicht die gespeicherten Daten (mem.json, settings.json, ugc.json,
    replay.json, Statistik ...) beider Fassungen nach dem Lauf.

    Rueckgabe: (Zahl der Dateien, Fehlertext oder None).
    """
    def liste(d):
        return sorted(n for n in os.listdir(d)
                      if os.path.isfile(os.path.join(d, n)) and n not in NICHT_VERGLEICHEN
                      and not n.endswith((".bak", ".tmp")))
    la, lb = liste(py_daten), liste(rs_daten)
    if la != lb:
        return len(la), "Dateien py=%s rust=%s" % (la, lb)
    for n in la:
        fehler = _texte_vergleichen(n, os.path.join(py_daten, n), os.path.join(rs_daten, n))
        if fehler:
            return len(la), fehler
    return len(la), None


def downloads_vergleichen(py_daten, rs_daten):
    """Vergleicht, was beide Fassungen nach "Downloads" geschrieben haben
    (Teilen von Bahnen/Leveln, PGN-Export, Replay-Export).

    Rueckgabe: (Zahl der Dateien, Fehlertext oder None).
    """
    a, b = os.path.join(py_daten, "Downloads"), os.path.join(rs_daten, "Downloads")
    la = sorted(os.listdir(a)) if os.path.isdir(a) else []
    lb = sorted(os.listdir(b)) if os.path.isdir(b) else []
    ohne_ziffern = [re.sub(r"\d", "#", n) for n in la]
    if ohne_ziffern != [re.sub(r"\d", "#", n) for n in lb]:
        return len(la), "Dateinamen py=%s rust=%s" % (la, lb)
    for na, nb in zip(la, lb):
        fehler = _texte_vergleichen(na, os.path.join(a, na), os.path.join(b, nb))
        if fehler:
            return len(la), fehler
    return len(la), None


def vergleichen(a_pfad, b_pfad, diff_pfad):
    import pygame
    a = pygame.image.load(a_pfad)
    b = pygame.image.load(b_pfad)
    if a.get_size() != b.get_size():
        return -1, "Groesse %s gegen %s" % (a.get_size(), b.get_size())
    w, h = a.get_size()
    anders = 0
    erste = None
    diff = pygame.Surface((w, h))
    ab = pygame.image.tobytes(a, "RGB")
    bb = pygame.image.tobytes(b, "RGB")
    if ab == bb:
        return 0, ""
    for y in range(h):
        for x in range(w):
            i = (y * w + x) * 3
            if ab[i:i + 3] != bb[i:i + 3]:
                anders += 1
                diff.set_at((x, y), (255, 0, 0))
                if erste is None:
                    erste = "(%d,%d) py=%s rust=%s" % (x, y, tuple(ab[i:i + 3]),
                                                       tuple(bb[i:i + 3]))
            else:
                g = ab[i] // 4
                diff.set_at((x, y), (g, g, g))
    pygame.image.save(diff, diff_pfad)
    return anders, erste


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("spiel")
    ap.add_argument("--mode", default="single")
    ap.add_argument("--seed", type=int, default=7)
    ap.add_argument("--frames", type=int, default=120)
    ap.add_argument("--keys", default="")
    ap.add_argument("--keys-file", default=None,
                    help="Ereignisliste aus einer Datei (lange Folgen)")
    ap.add_argument("--out", default=None)
    ap.add_argument("--daten", default=None,
                    help="Ordner, dessen Dateien in beide Datenordner kopiert werden")
    ap.add_argument("--nur-bild", action="store_true",
                    help="nur das Bild vergleichen, nicht die gespeicherten Daten "
                         "(mem.json, settings.json, ugc.json, replay.json, Statistik)")
    args = ap.parse_args()
    global VORLAGE
    VORLAGE = os.path.abspath(args.daten) if args.daten else None
    out = args.out or tempfile.mkdtemp(prefix="pgz_vergleich_")
    os.makedirs(out, exist_ok=True)
    if args.keys_file:
        with open(args.keys_file, encoding="utf-8") as f:
            args.keys = f.read().strip()
    keys = [k for k in args.keys.split(",")] if args.keys else []

    if args.spiel == "all":
        from games import ALL_GAMES
        spiele = [c.highscore_key for c in ALL_GAMES]
    else:
        spiele = [args.spiel]

    schlecht = 0
    for key in spiele:
        # Jeder Python-Lauf in einem eigenen Prozess: Module halten Zustand.
        if len(spiele) > 1:
            r = subprocess.run([sys.executable, __file__, key, "--mode", args.mode,
                                "--seed", str(args.seed), "--frames", str(args.frames),
                                "--keys", args.keys, "--out", out])
            schlecht += r.returncode != 0
            continue
        datei = key.replace(":", "_")
        py_png = os.path.join(out, datei + "_py.png")
        rs_png = os.path.join(out, datei + "_rust.png")
        # Alte Bilder weg: ein liegengebliebenes Bild darf nie als Messung gelten.
        for alt in (py_png, rs_png):
            if os.path.exists(alt):
                os.remove(alt)
        py_daten = datenordner(out, datei + "_pydata")
        rs_daten = datenordner(out, datei + "_rsdata")
        python_lauf(key, args.mode, args.seed, args.frames, keys, py_daten, py_png)
        try:
            rust_lauf(key, args.mode, args.seed, args.frames, keys, rs_daten, rs_png)
        except RuntimeError as e:
            print("%-14s %s" % (key, e))
            schlecht += 1
            continue
        n, erste = vergleichen(py_png, rs_png, os.path.join(out, datei + "_diff.png"))
        n_dat, dat_fehler = downloads_vergleichen(py_daten, rs_daten)
        if not args.nur_bild and not dat_fehler:
            n_mehr, dat_fehler = daten_vergleichen(py_daten, rs_daten)
            n_dat += n_mehr
        if n == 0 and dat_fehler:
            print("%-14s Dateien anders: %s" % (key, dat_fehler))
            schlecht += 1
        elif n == 0:
            print("%-14s gleich (%d Bilder%s)" % (key, args.frames,
                                                  ", %d Datei(en)" % n_dat if n_dat else ""))
        else:
            print("%-14s %s Pixel anders  %s" % (key, n, erste))
            schlecht += 1
    print("Ordner:", out)
    sys.exit(1 if schlecht else 0)


if __name__ == "__main__":
    main()
