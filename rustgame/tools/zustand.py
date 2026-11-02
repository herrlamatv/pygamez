# -*- coding: utf-8 -*-
"""Was sagt jedes Modul **vor `init()`** und **nach `quit()`**?

## Warum ein eigenes Werkzeug und keine Probe

`vergleich_module.py` faehrt alles in **einem** Prozess und ruft am
Anfang `pg.init()`. Diese Frage braucht drei Prozesse: ein `quit()`
laesst sich nicht zurueckdrehen, und "vor dem ersten `init()`" gibt es in
einem Prozess nur einmal. Dieselbe Bauart wie `uebersprungen.py`, das
aus demselben Grund beide Engines selbst startet.

## Was gemessen wird

125 Fragen in drei Zustaenden, je Engine ein eigener Prozess:

| Zustand | wie |
|---|---|
| `vorher` | frisch eingefuehrt, nichts hochgefahren |
| `laeuft` | `init()` und `set_mode((64, 48))` |
| `nachher` | dasselbe, dann `quit()` |

## Was die erste Messung fand (Runde 15)

**80 von 375** Antworten gingen auseinander, und fast alle in derselben
Richtung: **pygame sperrt, wir antworten.** Ein Programm, das `init()`
vergisst, bekam hier `(0, 0)` statt eines Hinweises - die schlimmste
Sorte Unterschied, weil nichts auffaellt.

Die Gruppen, alle gemessen und behoben:

| Gruppe | Satz von pygame | Stellen |
|---|---|---:|
| `event.*`, `key.*`, `mouse.*`, `display.Info/list_modes/flip/update/toggle_fullscreen` | `video system not initialized` | 27 |
| `font.Font` | `font not initialized` | 1 |
| `freetype.Font`, `get_error` | `RuntimeError: The FreeType 2 library hasn't been initialized` | 2 |
| `mixer.music.get_pos` | `mixer not initialized` | 1 |
| `time.set_timer` | `pygame is not initialized` - und **nach `quit()` geht es weiter** | 1 |
| `get_init()` | `False`, bevor `pygame.init()` lief - hier war es immer `True` | 1 |
| `display.get_caption()` | `()` ohne Fenster, nicht `('pygame window', ...)` | 1 |
| `freetype.get_cache_size()` | `0` vor `init()`, dann 64 | 1 |
| jedes Untermodul bei unbekanntem Namen | `module 'pygame.camera' has no attribute 'get_init'` - hier stand nur der Name | 12 |

**Welche Funktion ein Tor hat, ist nicht herzuleiten.** `key.name`,
`key.key_code`, `event.custom_type`, `mouse.get_relative_mode`,
`display.get_num_displays` (gibt `0`) und `display.get_active` (gibt
`False`) antworten ohne Video; `time.delay` und `time.wait` auch. Darum
steht hier die ganze Tabelle und nicht eine Regel.

## Fahren

```bash
.venv/Scripts/python.exe rustgame/tools/zustand.py
```

Endet mit 1, sobald eine Antwort auseinandergeht.
"""
import importlib
import os
import re
import subprocess
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HIER))
HUELLE = os.path.join(ROOT, "rustgame", "crates", "rustgame-py")

for k, v in (("SDL_VIDEODRIVER", "dummy"), ("RUSTGAME_VIDEODRIVER", "dummy"),
             ("SDL_AUDIODRIVER", "dummy"), ("RUSTGAME_AUDIODRIVER", "dummy"),
             ("PYGAME_HIDE_SUPPORT_PROMPT", "1")):
    os.environ.setdefault(k, v)

ZUSTAENDE = ("vorher", "laeuft", "nachher")

#: Saetze, die **das Werkzeug** schreibt und nicht pygame - dieselbe
#: Liste wie in `vergleich_module.py`, siehe dort `_MASCHINE`.
_MASCHINE = (
    r"takes (?:exactly|at most|at least) \d+ (?:positional )?arguments?",
    r"takes \d+ positional arguments? but",
    r"takes from \d+ to \d+ positional arguments",
    r"takes no arguments",
    r"takes no keyword arguments",
    r"missing \d+ required positional argument",
    r"argument \d+ must be",
    r"got an unexpected keyword argument",
    r"got some positional-only arguments",
    r"object cannot be converted to",
    r"object cannot be interpreted as an integer",
    r"must be real number, not",
)


def gestalt(text, paket):
    """Der Satz, auf das Vergleichbare gebracht."""
    text = text.replace(paket, "@").replace("pygame", "@")
    m = re.match(r"argument '[^']+': (.*)$", text, re.S)
    if m:
        text = m.group(1)
    for muster in _MASCHINE:
        if re.search(muster, text):
            return "<argumentfehler>"
    return " ".join(text.split())


def fragen(pg):
    """Die Fragen - `(Name, Ruf)`.

    **Die Untermodule werden zuerst eingefuehrt.** `pg.camera` ist bei
    pygame kein Feld, solange niemand `pygame.camera` eingefuehrt hat;
    wer am Hauptmodul fragt, misst die Huelle statt der Funktion. Beim
    ersten Lauf meldete das neun Abweichungen, und keine lag an der
    Huelle - dieselbe Falle wie in Runde 11 bei `geometry`.
    """
    for n in ("camera", "midi", "scrap", "freetype", "joystick", "system",
              "mixer", "font", "display", "event", "key", "mouse", "time",
              "draw", "image", "transform", "mask", "cursors"):
        try:
            importlib.import_module(pg.__name__ + "." + n)
        except Exception:                            # noqa: BLE001
            pass
    aus = []

    def f(name, ruf):
        aus.append((name, ruf))

    # ----- display -------------------------------------------------------
    f("display.get_init", lambda: pg.display.get_init())
    f("display.get_surface", lambda: pg.display.get_surface())
    f("display.get_caption", lambda: pg.display.get_caption())
    f("display.get_driver", lambda: pg.display.get_driver())
    f("display.Info", lambda: str(pg.display.Info()) != "")
    f("display.get_window_size", lambda: pg.display.get_window_size())
    f("display.flip", lambda: pg.display.flip())
    f("display.update", lambda: pg.display.update())
    f("display.get_active", lambda: pg.display.get_active())
    f("display.mode_ok", lambda: pg.display.mode_ok((64, 48)))
    f("display.list_modes", lambda: len(pg.display.list_modes()) >= 0)
    f("display.get_desktop_sizes",
      lambda: len(pg.display.get_desktop_sizes()))
    f("display.set_caption", lambda: pg.display.set_caption("x"))
    f("display.get_allow_screensaver",
      lambda: pg.display.get_allow_screensaver())
    f("display.set_allow_screensaver",
      lambda: pg.display.set_allow_screensaver(True))
    f("display.toggle_fullscreen", lambda: pg.display.toggle_fullscreen())
    f("display.get_num_displays", lambda: pg.display.get_num_displays())
    # ----- event ---------------------------------------------------------
    f("event.get", lambda: pg.event.get())
    f("event.poll", lambda: pg.event.poll().type)
    f("event.peek", lambda: pg.event.peek())
    f("event.clear", lambda: pg.event.clear())
    f("event.pump", lambda: pg.event.pump())
    f("event.get_grab", lambda: pg.event.get_grab())
    f("event.set_grab", lambda: pg.event.set_grab(False))
    f("event.post", lambda: pg.event.post(pg.event.Event(pg.USEREVENT)))
    f("event.get_blocked", lambda: pg.event.get_blocked(pg.QUIT))
    f("event.set_allowed", lambda: pg.event.set_allowed(None))
    f("event.custom_type", lambda: pg.event.custom_type())
    # ----- key -----------------------------------------------------------
    f("key.get_pressed", lambda: len(pg.key.get_pressed()))
    f("key.get_mods", lambda: pg.key.get_mods())
    f("key.set_mods", lambda: pg.key.set_mods(0))
    f("key.get_focused", lambda: pg.key.get_focused())
    f("key.name", lambda: pg.key.name(pg.K_a))
    f("key.key_code", lambda: pg.key.key_code("a"))
    f("key.get_repeat", lambda: pg.key.get_repeat())
    f("key.set_repeat", lambda: pg.key.set_repeat(0))
    f("key.start_text_input", lambda: pg.key.start_text_input())
    f("key.stop_text_input", lambda: pg.key.stop_text_input())
    f("key.set_text_input_rect",
      lambda: pg.key.set_text_input_rect(pg.Rect(0, 0, 1, 1)))
    f("key.get_just_pressed", lambda: len(pg.key.get_just_pressed()))
    f("key.get_just_released", lambda: len(pg.key.get_just_released()))
    # ----- mouse ---------------------------------------------------------
    f("mouse.get_pos", lambda: pg.mouse.get_pos())
    f("mouse.get_pressed", lambda: pg.mouse.get_pressed())
    f("mouse.set_pos", lambda: pg.mouse.set_pos((1, 1)))
    f("mouse.get_rel", lambda: pg.mouse.get_rel())
    f("mouse.get_visible", lambda: pg.mouse.get_visible())
    f("mouse.set_visible", lambda: pg.mouse.set_visible(True))
    f("mouse.get_focused", lambda: pg.mouse.get_focused())
    f("mouse.get_cursor", lambda: type(pg.mouse.get_cursor()).__name__)
    f("mouse.get_just_pressed", lambda: pg.mouse.get_just_pressed())
    f("mouse.get_just_released", lambda: pg.mouse.get_just_released())
    f("mouse.set_relative_mode", lambda: pg.mouse.set_relative_mode(False))
    f("mouse.get_relative_mode", lambda: pg.mouse.get_relative_mode())
    # ----- font ----------------------------------------------------------
    f("font.get_init", lambda: pg.font.get_init())
    f("font.get_default_font", lambda: pg.font.get_default_font())
    f("font.Font", lambda: pg.font.Font(None, 20).get_height() > 0)
    f("font.get_fonts", lambda: len(pg.font.get_fonts()) > 0)
    f("font.match_font", lambda: bool(pg.font.match_font("arial")))
    f("font.SysFont", lambda: pg.font.SysFont("arial", 20).get_height() > 0)
    # ----- freetype ------------------------------------------------------
    f("freetype.get_init", lambda: pg.freetype.get_init())
    f("freetype.Font", lambda: pg.freetype.Font(None, 20).height != 0)
    f("freetype.get_default_resolution",
      lambda: pg.freetype.get_default_resolution())
    f("freetype.get_version", lambda: pg.freetype.get_version())
    f("freetype.get_error", lambda: pg.freetype.get_error())
    f("freetype.get_cache_size", lambda: pg.freetype.get_cache_size())
    f("freetype.get_default_font", lambda: pg.freetype.get_default_font())
    f("freetype.set_default_resolution",
      lambda: pg.freetype.set_default_resolution(72))
    # ----- mixer ---------------------------------------------------------
    f("mixer.get_init", lambda: pg.mixer.get_init())
    f("mixer.get_busy", lambda: pg.mixer.get_busy())
    f("mixer.get_num_channels", lambda: pg.mixer.get_num_channels())
    f("mixer.Sound", lambda: pg.mixer.Sound(buffer=b"\0" * 400).get_length())
    f("mixer.Channel", lambda: type(pg.mixer.Channel(0)).__name__)
    f("mixer.find_channel", lambda: type(pg.mixer.find_channel()).__name__)
    f("mixer.stop", lambda: pg.mixer.stop())
    f("mixer.pause", lambda: pg.mixer.pause())
    f("mixer.unpause", lambda: pg.mixer.unpause())
    f("mixer.fadeout", lambda: pg.mixer.fadeout(1))
    f("mixer.set_num_channels", lambda: pg.mixer.set_num_channels(8))
    f("mixer.get_sdl_mixer_version",
      lambda: len(pg.mixer.get_sdl_mixer_version()))
    f("music.get_busy", lambda: pg.mixer.music.get_busy())
    f("music.get_volume", lambda: pg.mixer.music.get_volume())
    f("music.set_volume", lambda: pg.mixer.music.set_volume(0.5))
    f("music.stop", lambda: pg.mixer.music.stop())
    f("music.pause", lambda: pg.mixer.music.pause())
    f("music.rewind", lambda: pg.mixer.music.rewind())
    f("music.fadeout", lambda: pg.mixer.music.fadeout(1))
    f("music.get_pos", lambda: pg.mixer.music.get_pos())
    f("music.get_endevent", lambda: pg.mixer.music.get_endevent())
    f("music.set_endevent", lambda: pg.mixer.music.set_endevent(0))
    # ----- time ----------------------------------------------------------
    f("time.get_ticks", lambda: pg.time.get_ticks() >= 0)
    f("time.Clock", lambda: pg.time.Clock().get_fps())
    f("time.set_timer", lambda: pg.time.set_timer(pg.USEREVENT, 0))
    f("time.delay", lambda: pg.time.delay(0))
    f("time.wait", lambda: pg.time.wait(0))
    # ----- scrap ---------------------------------------------------------
    f("scrap.get_init", lambda: pg.scrap.get_init())
    f("scrap.get_types", lambda: len(pg.scrap.get_types()) >= 0)
    f("scrap.get", lambda: pg.scrap.get(pg.SCRAP_TEXT))
    f("scrap.contains", lambda: pg.scrap.contains(pg.SCRAP_TEXT))
    f("scrap.lost", lambda: pg.scrap.lost())
    # ----- Geraete -------------------------------------------------------
    f("joystick.get_init", lambda: pg.joystick.get_init())
    f("joystick.get_count", lambda: pg.joystick.get_count())
    f("camera.get_init", lambda: pg.camera.get_init())
    f("camera.list_cameras", lambda: len(pg.camera.list_cameras()) >= 0)
    f("camera.get_backends", lambda: len(pg.camera.get_backends()) > 0)
    f("midi.get_init", lambda: pg.midi.get_init())
    f("midi.get_count", lambda: pg.midi.get_count())
    f("midi.get_default_output_id", lambda: pg.midi.get_default_output_id())
    # ----- Basis und System ----------------------------------------------
    f("get_init", lambda: pg.get_init())
    f("get_sdl_version", lambda: len(pg.get_sdl_version()))
    f("system.get_pref_path", lambda: bool(pg.system.get_pref_path("a", "b")))
    f("system.get_cpu_count", lambda: pg.system.get_cpu_count() > 0)
    f("system.get_total_ram", lambda: pg.system.get_total_ram() > 0)
    # **`get_error` nicht.** Er meldet den *letzten* SDL-Fehler, und der
    # haengt an allem, was vorher lief: bei pygame stand dort nach den
    # Fragen darueber `CreateSystemCursor is not current`. Eine Frage,
    # deren Antwort die Fragen davor sind, misst nichts.
    # ----- Flaechen, Zeichnen, Masken ------------------------------------
    f("Surface(4,4)", lambda: pg.Surface((4, 4)).get_size())
    f("Surface.convert", lambda: pg.Surface((4, 4)).convert().get_size())
    f("Surface.convert_alpha",
      lambda: pg.Surface((4, 4)).convert_alpha().get_size())
    f("draw.rect", lambda: tuple(pg.draw.rect(
        pg.Surface((8, 8)), (1, 2, 3), (0, 0, 4, 4))))
    f("image.frombytes", lambda: pg.image.frombytes(
        b"\0" * 48, (4, 4), "RGB").get_size())
    f("transform.scale", lambda: pg.transform.scale(
        pg.Surface((4, 4)), (8, 8)).get_size())
    f("mask.Mask", lambda: pg.mask.Mask((4, 4)).count())
    f("cursors.Cursor", lambda: type(pg.cursors.Cursor(
        pg.SYSTEM_CURSOR_ARROW)).__name__)
    return aus


def arbeiter(zustand, engine):
    """Beantwortet alle Fragen in einem Zustand - **ein Prozess je Lauf**."""
    if engine == "rustgame":
        sys.path.insert(0, HUELLE)
        pg = importlib.import_module("rustgame_py")
    else:
        pg = importlib.import_module("pygame")
    if zustand in ("laeuft", "nachher"):
        pg.init()
        try:
            pg.display.set_mode((64, 48))
        except Exception:                            # noqa: BLE001
            pass
        try:
            pg.mixer.init()
        except Exception:                            # noqa: BLE001
            pass
    if zustand == "nachher":
        pg.quit()
    for name, ruf in fragen(pg):
        try:
            wert = repr(ruf())
            wert = wert.replace(pg.__name__, "@").replace("pygame", "@")
            wert = re.sub(r"0x[0-9a-fA-F]+", "0xADR", wert)
            print("%s\t%s\t%s" % (name, "ok", " ".join(wert.split())[:90]))
        except BaseException as e:                   # noqa: BLE001
            print("%s\t%s\t%s" % (name, type(e).__name__,
                                  gestalt(str(e), pg.__name__)[:90]))
    return 0


def fahren(zustand, engine):
    u = dict(os.environ, PYTHONIOENCODING="utf-8")
    p = subprocess.run(
        [sys.executable, "-W", "ignore", os.path.abspath(__file__),
         "--arbeiter", zustand, engine],
        capture_output=True, text=True, env=u, encoding="utf-8",
        cwd=os.path.join(ROOT, "rustgame"))
    if p.returncode != 0:
        print("!! %s/%s endete mit %d" % (zustand, engine, p.returncode))
        print(p.stderr[-1200:])
        return {}
    aus = {}
    for zeile in p.stdout.splitlines():
        t = zeile.split("\t")
        if len(t) == 3:
            aus[t[0]] = (t[1], t[2])
    return aus


def main():
    if "--arbeiter" in sys.argv:
        i = sys.argv.index("--arbeiter")
        return arbeiter(sys.argv[i + 1], sys.argv[i + 2])

    print("Zustandsvergleich: pygame-ce gegen die rustgame-Huelle")
    print("=" * 62)
    gesamt = fragen_gesamt = 0
    for z in ZUSTAENDE:
        a = fahren(z, "pygame")
        b = fahren(z, "rustgame")
        if not a or not b:
            print("** %s: ein Lauf hat nichts geliefert" % z)
            return 1
        gem = sorted(set(a) & set(b))
        # **Eine Messung ohne Eintraege ist immer gleich.**
        if len(gem) < 100:
            print("** %s: nur %d gemeinsame Fragen" % (z, len(gem)))
            return 1
        anders = [k for k in gem if a[k] != b[k]]
        gesamt += len(anders)
        fragen_gesamt += len(gem)
        print("%-9s %4d Fragen, %d Abweichungen" % (z, len(gem), len(anders)))
        for k in sorted(set(a) ^ set(b)):
            print("   nur bei %s: %s" % ("pygame" if k in a else "rustgame", k))
        for k in anders:
            print("   %-32s pygame   %-14s %s" % (k, a[k][0], a[k][1][:56]))
            print("   %-32s rustgame %-14s %s" % ("", b[k][0], b[k][1][:56]))
    print("-" * 62)
    if gesamt == 0:
        print("%d Fragen in drei Zustaenden, Antwort fuer Antwort dieselbe."
              % fragen_gesamt)
        return 0
    print("%d von %d Antworten gehen auseinander." % (gesamt, fragen_gesamt))
    return 1


if __name__ == "__main__":
    sys.exit(main())
