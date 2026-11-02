# -*- coding: utf-8 -*-
"""
rustgame/tools/api_coverage.py
==============================
Vergleicht die API von **echtem pygame-ce** mit dem, was rustgame hat, und
schreibt daraus ``rustgame/API.md``.

Die Liste der pygame-Namen wird **nicht abgetippt**, sondern dem installierten
pygame abgefragt. Das hat zwei Vorteile: sie kann nicht veralten, und sie
kann nicht geschoenfaerbt werden - was pygame kann und rustgame nicht,
steht danach schwarz auf weiss in der Tabelle.

Die Rust-Seite wird aus den Quelltexten gelesen (``pub fn``, ``pub struct``,
``pub const``). Das ist bewusst grob: ein Name, den es gibt, heisst nicht,
dass er sich in jeder Ecke wie pygame verhaelt. Dafuer gibt es die
Referenzbilder und den Fuzzer - diese Tabelle beantwortet nur die Frage
"gibt es das ueberhaupt schon?".

Aufruf::

    .venv/Scripts/python.exe rustgame/tools/api_coverage.py
"""

import os
import importlib
import re
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HIER))
RUSTGAME = os.path.join(ROOT, "rustgame")

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")

import pygame  # noqa: E402

# Die Huelle selbst - **importiert**, nicht aus dem Quelltext geraten.
#
# Die Seite "was pygame kann" wird seit jeher per `dir()` erfragt, damit
# sie nicht veralten kann. Fuer die rustgame-Seite stand hier lange ein
# Regenwurm aus regulaeren Ausdruecken ueber die .rs-Dateien - und der
# hat **beide** Male danebengelegen, als es darauf ankam: er fand
# `pub fn angle` im Kern und meldete den Namen als vorhanden, obwohl
# `rustgame_py.math.Vector2(3,4).angle` nicht existierte.
#
# Ein Muster ueber Quelltext beantwortet die Frage "steht da irgendwo
# dieses Wort". Die Frage lautet aber "kommt ein Python-Programm daran".
# Die einzige Stelle, die das sicher weiss, ist das gebaute Modul.
os.environ.setdefault("RUSTGAME_VIDEODRIVER", "dummy")
sys.path.insert(0, os.path.join(RUSTGAME, "crates", "rustgame-py"))
try:
    import rustgame_py  # noqa: E402
except ImportError as e:
    rustgame_py = None
    print("Warnung: rustgame_py nicht importierbar (%s)." % e)
    print("         Die Huellen-Spalte bleibt leer - erst bauen:")
    print("         cd rustgame/crates/rustgame-py && cargo build --release")


# ---------------------------------------------------------------------------
#  Was pygame kann
# ---------------------------------------------------------------------------

# Modul in pygame -> (Ueberschrift, Kern-Pfade, Huellen-Pfade)
#
# **Zwei** Pfadlisten, und das ist der Kern der Sache. Was zaehlt, ist
# nicht "gibt es den Namen irgendwo in rustgame", sondern "kommt ein
# Python-Programm daran" - das Ziel heisst `import rustgame_py as pygame`.
#
# Hier stand lange **eine** Liste, und beide Seiten wurden in denselben
# Topf geworfen. Das hat zweimal etwas verdeckt:
#
# * `Vector2.angle` gab es als `pub fn angle` im Kern, aber nirgends in
#   der Huelle. Die Tabelle meldete den Namen als vorhanden;
#   `rustgame_py.math.Vector2(3,4).angle` gab es nicht.
# * `pygame.sprite` und `pygame.mixer` standen mit 19 von 19 in der
#   Tabelle, weil die Rust-Kisten fertig sind - die Huelle hatte die
#   beiden Module ueberhaupt nicht.
#
# Deshalb entscheidet jetzt die **Huelle** ueber "da". Ein Name, den nur
# der Kern hat, bekommt eine eigene Zeile: `Nur im Kern`. Er zaehlt als
# fehlend, denn aus Python ist er nicht zu erreichen.
MODULE = [
    # ----- Das Hauptmodul, und es stand hier nicht ------------------------
    #
    # **Der siebte blinde Fleck dieses Messgeraets.** Es zaehlte jedes
    # Untermodul und jede Klasse auf - `Surface`, `Rect`, `math.Vector3`,
    # `_sdl2.video.Image`, 54 Eintraege - und fragte **nie** `dir(pygame)`
    # selbst. Darum stand hier "1085 von 1085 Namen (100 %)", waehrend
    # fuenf Namen fehlten:
    #
    #     print_debug_info   Funktion   (debug_test.py haengte daran)
    #     docs               Modul      (docs_test.py haengte daran)
    #     surflock           Modul
    #     SDL                SDLVersion
    #     rev                str
    #
    # Gefunden hat sie nicht dieses Werkzeug, sondern das andere:
    # `pygame_tests.py` hatte drei Testdateien nie in seiner Liste, und
    # zwei davon pruefen genau diese Namen. **Zwei Messgeraete, dieselbe
    # Luecke** - und beide hatten jahrelang recht behalten, weil niemand
    # die Frage gestellt hat.
    ("", "Das Hauptmodul (`pygame`)",
     ["crates/rustgame-py/src", "crates/rustgame-core/src"]),
    ("Surface", "Flaechen (`pygame.Surface`)", ["crates/rustgame-core/src/surface.rs"]),
    ("Rect", "Rechtecke (`pygame.Rect`)", ["crates/rustgame-core/src/rect.rs"]),
    ("FRect", "Rechtecke mit Komma (`pygame.FRect`)",
     ["crates/rustgame-core/src/frect.rs"]),
    ("Color", "Farben (`pygame.Color`)", ["crates/rustgame-core/src/color.rs"]),
    ("draw", "Zeichnen (`pygame.draw`)", ["crates/rustgame-core/src/draw"]),
    ("transform", "Verwandeln (`pygame.transform`)", ["crates/rustgame-core/src/transform.rs"]),
    ("image", "Bilder (`pygame.image`)", ["crates/rustgame-core/src/image.rs"]),
    ("time", "Zeit (`pygame.time`)", ["crates/rustgame-core/src/time.rs"]),
    ("font", "Text (`pygame.font`)", ["crates/rustgame-font/src"]),
    ("display", "Fenster (`pygame.display`)", ["crates/rustgame-display/src"]),
    ("event", "Ereignisse (`pygame.event`)", ["crates/rustgame-display/src"]),
    ("key", "Tastatur (`pygame.key`)", ["crates/rustgame-display/src"]),
    ("mouse", "Maus (`pygame.mouse`)", ["crates/rustgame-display/src"]),
    ("mixer", "Ton (`pygame.mixer`)", ["crates/rustgame-mixer/src"]),
    ("sprite", "Sprites (`pygame.sprite`)", ["crates/rustgame-sprite/src"]),
    # `mask` und `math` teilen sich eine Kiste: Masken brauchen Rechteck
    # und Flaeche, Vektoren nichts davon - aber zwei Kisten fuer zusammen
    # ein gutes Tausend Zeilen waeren Buchhaltung ohne Gewinn.
    ("mask", "Masken (`pygame.mask`)", ["crates/rustgame-math/src/mask.rs"]),
    ("math", "Vektoren (`pygame.math`)", ["crates/rustgame-math/src"]),

    # ----- Module, die die Tabelle nie abgefragt hat -----------------------
    #
    # Die Liste oben war von Hand gepflegt und deckte siebzehn Module ab.
    # `pygame` hat mehr, und was hier nicht steht, kann auch nicht fehlen -
    # die Zahl war also doppelt zu freundlich: sie zaehlte Namen als da,
    # an die Python nicht herankam, **und** sie fragte zehn Module gar
    # nicht erst.
    #
    # Nicht aufgenommen: `base`, `rwobject`, `surflock`, `pkgdata`,
    # `typing`, `constants` (Innereien ohne eigene API) und `sysfont` -
    # dessen `dir()` besteht zur Haelfte aus durchgereichten
    # Standardbibliotheksnamen (`difflib`, `dirname`, `exists`), das waere
    # Rauschen statt Messung. Was daran zaehlt, steht als `SysFont` und
    # `Font` ohnehin in `pygame.font`.
    ("cursors", "Zeigerformen (`pygame.cursors`)",
     ["crates/rustgame-display/src/mouse.rs"]),
    ("joystick", "Spielhebel (`pygame.joystick`)", ["crates/rustgame-display/src"]),
    ("mixer_music", "Musik (`pygame.mixer.music`)", ["crates/rustgame-mixer/src"]),
    ("pixelarray", "Pixelfeld (`pygame.pixelarray`)", ["crates/rustgame-core/src"]),
    ("pixelcopy", "Pixel kopieren (`pygame.pixelcopy`)", ["crates/rustgame-core/src"]),
    # Die zwei numpy-Bruecken. Sie sind in pygame selbst reines Python und
    # hier wortgleich uebertragen; ihre Heimat ist darum die Huelle und
    # keine Kiste. Ohne numpy gibt es sie nicht - dann sind sie auf beiden
    # Seiten leer, und die Tabelle sagt `0 von 0`.
    ("surfarray", "Flaechen als Felder (`pygame.surfarray`)",
     ["crates/rustgame-py/src/python"]),
    ("sndarray", "Klaenge als Felder (`pygame.sndarray`)",
     ["crates/rustgame-py/src/python"]),
    ("scrap", "Zwischenablage (`pygame.scrap`)", ["crates/rustgame-display/src"]),
    ("system", "System (`pygame.system`)", ["crates/rustgame-display/src"]),
    ("window", "Fenster als Objekt (`pygame.window`)", ["crates/rustgame-display/src"]),
    # Die Klasse selbst hat 33 Namen. Ohne diese Zeile zaehlte die
    # Tabelle nur `Window` und `get_grabbed_window` - zwei von zwei, und
    # die 33 dahinter waeren unsichtbar. Genau die Sorte Messung, die
    # schmeichelt.
    ("window.Window", "Ein Fenster (`pygame.window.Window`)",
     ["crates/rustgame-display/src/fenster.rs",
      "crates/rustgame-py/src/fenster.rs"]),
    ("version", "Fassungen (`pygame.version`)", ["crates/rustgame-core/src"]),
    ("colordict", "Farbnamen (`pygame.colordict`)", ["crates/rustgame-core/src"]),

    # ----- Und noch einmal dieselbe Luecke, eine Ebene hoeher --------------
    #
    # Der Vermerk oben zaehlt auf, was **bewusst** draussen bleibt
    # (`base`, `rwobject`, `surflock`, `typing`, `sysfont`). Sechs Module
    # standen dort nicht, weil sie schlicht **vergessen** wurden - sie
    # werden von `import pygame` nicht mitgeladen und tauchen darum in
    # keinem `dir(pygame)` auf, das man beim Schreiben der Liste ansieht:
    #
    #     import pygame.gfxdraw, pygame.freetype, pygame.midi, ...
    #
    # Die Zahl war damit nicht nur zu freundlich, sie war zu freundlich um
    # mehrere hundert Namen. Ein fehlender Eintrag faellt irgendwann auf;
    # eine fehlende **Frage** nie.
    ("gfxdraw", "Zeichnen mit Kantenglaettung (`pygame.gfxdraw`)",
     ["crates/rustgame-core/src/draw"]),
    ("freetype", "Text ueber FreeType (`pygame.freetype`)",
     ["crates/rustgame-font/src"]),
    ("freetype.Font", "Eine FreeType-Schrift (`pygame.freetype.Font`)",
     ["crates/rustgame-font/src", "crates/rustgame-py/src/freitext.rs"]),
    ("midi", "MIDI (`pygame.midi`)", ["crates/rustgame-mixer/src"]),
    ("camera", "Kamera (`pygame.camera`)", ["crates/rustgame-display/src"]),
    ("_sdl2.video", "Beschleunigtes Zeichnen (`pygame._sdl2.video`)",
     ["crates/rustgame-display/src"]),

    # ----- Und derselbe Fehler noch eine Ebene tiefer ----------------------
    #
    # **Der achte blinde Fleck.** `pygame._sdl2` hat **acht** Untermodule;
    # hier stand genau eines (`_sdl2.video`). Die anderen fuenf waren nie
    # Gegenstand einer Messung - und rustgame hatte drei davon gar nicht:
    #
    #     audio            18 Konstanten, AudioDevice,
    #                      get_audio_device_names
    #     controller_old   die alte Fassung, vier Namen mehr als `controller`
    #     mixer            set_post_mix
    #     sdl2             neun INIT_*-Flaggen, init_subsystem
    #     window           Window (dasselbe wie `pygame.window.Window`)
    #
    # Gefunden hat es nicht diese Tabelle, sondern die **Endabnahme des
    # Plans**: dort steht, das Spielprogramm solle ueber dreihundert Bilder
    # verglichen werden. Gefahren wurden dreissig. Bei dreihundert wich die
    # Ereignisschlange ab, und in der Spur stand `AUDIODEVICEADDED` - ein
    # Ereignis, das es hier nicht gab, weil es die Tongeraete nicht gab.
    #
    # **Eine Zahl im Plan, die nie eingelöst wurde, ist dasselbe wie ein
    # Vermerk, den niemand nachhaelt.**
    ("_sdl2.audio", "Tongeraete (`pygame._sdl2.audio`)",
     ["crates/rustgame-mixer/src", "crates/rustgame-py/src/python"]),
    ("_sdl2.controller_old", "Spielgeraete, alt (`pygame._sdl2.controller_old`)",
     ["crates/rustgame-py/src/python"]),
    ("_sdl2.mixer", "Mischhaken (`pygame._sdl2.mixer`)",
     ["crates/rustgame-py/src/python"]),
    ("_sdl2.sdl2", "Teilsysteme (`pygame._sdl2.sdl2`)",
     ["crates/rustgame-py/src/python"]),
    ("_sdl2.window", "Fenster (`pygame._sdl2.window`)",
     ["crates/rustgame-display/src/fenster.rs"]),

    # ----- Klassen, jede fuer sich -----------------------------------------
    #
    # Die Eintraege oben fragen ein **Modul** ab; die hier eine **Klasse**
    # darin. Das ist nicht doppelt gemoppelt, sondern der Punkt: solange
    # nur Modulnamen verglichen wurden, galt `slerp` als vorhanden, weil
    # `Vector2` es hatte - dass `Vector3.slerp` fehlte, sah niemand. Die
    # Zeile `math.Vector3` unten haette es am ersten Tag gemeldet.
    #
    # Der Pfad zeigt jeweils auf die Datei, die **genau diese** Klasse
    # umsetzt. Ein gemeinsames Verzeichnis waere derselbe Fehler noch
    # einmal.
    # Jeweils Kern **und** Huelle: die `*_ip`-Geschwister gibt es im Kern
    # nicht und sie fehlen dort auch nicht. Ein Rust-`Vector2` ist `Copy`,
    # also schreibt man `v = v.rotate(90)`; erst in Python, wo ein Vektor
    # veraenderlich ist, braucht es `rotate_ip`. Wer nur den Kern abfragt,
    # meldet zwanzig Luecken, die keine sind.
    ("math.Vector2", "Vektor in der Ebene (`pygame.math.Vector2`)",
     ["crates/rustgame-math/src/vector2.rs", "crates/rustgame-py/src/vektor.rs"]),
    ("math.Vector3", "Vektor im Raum (`pygame.math.Vector3`)",
     ["crates/rustgame-math/src/vector3.rs", "crates/rustgame-py/src/vektor.rs"]),
    ("mask.Mask", "Eine Maske (`pygame.mask.Mask`)",
     ["crates/rustgame-math/src/mask.rs", "crates/rustgame-py/src/maske.rs"]),
    ("time.Clock", "Die Uhr (`pygame.time.Clock`)",
     ["crates/rustgame-core/src/time.rs", "crates/rustgame-py/src/zeit.rs"]),
    ("font.Font", "Eine Schrift (`pygame.font.Font`)",
     ["crates/rustgame-font/src/font.rs", "crates/rustgame-py/src/text.rs"]),
    ("mixer.Sound", "Ein Klang (`pygame.mixer.Sound`)",
     ["crates/rustgame-mixer/src"]),
    ("mixer.Channel", "Eine Tonspur (`pygame.mixer.Channel`)",
     ["crates/rustgame-mixer/src"]),
    ("sprite.Sprite", "Eine Spielfigur (`pygame.sprite.Sprite`)",
     ["crates/rustgame-sprite/src"]),
    ("sprite.Group", "Eine Gruppe (`pygame.sprite.Group`)",
     ["crates/rustgame-sprite/src"]),
    # ----- die vierte blinde Stelle -------------------------------------
    #
    # **Drei Mal hat dieses Werkzeug zu wenig gefragt**, und jedes Mal sah
    # die Zahl darum besser aus als die Sache: erst fehlten zehn Module,
    # dann sieben Namen in `EGAL` (inzwischen weg, siehe dort), dann fuenf
    # veraltete Ausnahmen. Hier ist
    # die vierte Stelle, und sie ist die groesste.
    #
    # Gefragt wurde nach `sprite` (dem Modul), `sprite.Sprite` und
    # `sprite.Group`. `pygame.sprite` hat aber **neun weitere Klassen**,
    # und zwei davon sind die dicksten des ganzen Pakets:
    # `LayeredDirty` mit 30 Namen und `LayeredUpdates` mit 25. Dazu
    # kamen `midi.Input`/`Output`, die drei Fassungsklassen, die drei
    # Zeichenklassen aus `_sdl2.video`, `BufferProxy`, `Cursor`, `Event`
    # und - das ganze Modul - `pygame.geometry`.
    #
    # Zusammen gut zweihundert Namen, nach denen nie jemand gefragt hat.
    # Die Abdeckungszahl ist damit erst gefallen und dann wieder
    # gestiegen; die zweite Zahl ist die erste, die etwas bedeutet.
    #
    # **Die Regel, die daraus folgt:** die Liste wird nicht gepflegt,
    # sondern geprueft. Der Abschnitt "Vollstaendigkeit" unten vergleicht
    # sie mit dem, was `pygame` wirklich hat, und meldet jede Luecke.
    ("geometry", "Geometrie (`pygame.geometry`)", ["crates/rustgame-math/src"]),
    ("base", "Grundfunktionen (`pygame.base`)", ["crates/rustgame-core/src"]),
    ("bufferproxy", "Pufferzeiger (`pygame.bufferproxy`)",
     ["crates/rustgame-py/src"]),
    ("rwobject", "Dateiobjekte (`pygame.rwobject`)", ["crates/rustgame-core/src"]),
    ("surflock", "Flaechensperren (`pygame.surflock`)",
     ["crates/rustgame-py/src"]),
    ("sysfont", "Systemschriften (`pygame.sysfont`)", ["crates/rustgame-font/src"]),
    ("ftfont", "Text ueber FreeType (`pygame.ftfont`)",
     ["crates/rustgame-py/src/python"]),
    ("imageext", "Bilder, erweitert (`pygame.imageext`)",
     ["crates/rustgame-core/src"]),
    ("_render", "Beschleunigtes Zeichnen (`pygame._render`)",
     ["crates/rustgame-py/src/python"]),
    ("geometry.Circle", "Ein Kreis (`pygame.geometry.Circle`)",
     ["crates/rustgame-math/src"]),
    ("geometry.Line", "Eine Linie (`pygame.geometry.Line`)",
     ["crates/rustgame-math/src"]),
    ("BufferProxy", "Ein Pufferzeiger (`pygame.BufferProxy`)",
     ["crates/rustgame-py/src"]),
    ("Cursor", "Eine Zeigerform (`pygame.Cursor`)", ["crates/rustgame-py/src"]),
    ("Event", "Ein Ereignis (`pygame.Event`)", ["crates/rustgame-display/src"]),
    ("PixelArray", "Ein Pixelfeld (`pygame.PixelArray`)",
     ["crates/rustgame-py/src"]),
    ("sprite.AbstractGroup", "Die Gruppenbasis (`pygame.sprite.AbstractGroup`)",
     ["crates/rustgame-sprite/src"]),
    ("sprite.DirtySprite", "Eine schmutzige Figur (`pygame.sprite.DirtySprite`)",
     ["crates/rustgame-sprite/src"]),
    ("sprite.GroupSingle", "Eine Einzelgruppe (`pygame.sprite.GroupSingle`)",
     ["crates/rustgame-sprite/src"]),
    ("sprite.LayeredUpdates", "Gruppe mit Ebenen (`pygame.sprite.LayeredUpdates`)",
     ["crates/rustgame-sprite/src"]),
    ("sprite.LayeredDirty", "Ebenen und schmutzige Rechtecke"
     " (`pygame.sprite.LayeredDirty`)", ["crates/rustgame-sprite/src"]),
    ("sprite.OrderedUpdates", "Gruppe in Reihenfolge"
     " (`pygame.sprite.OrderedUpdates`)", ["crates/rustgame-sprite/src"]),
    ("sprite.RenderUpdates", "Gruppe mit Rechtecken"
     " (`pygame.sprite.RenderUpdates`)", ["crates/rustgame-sprite/src"]),
    ("sprite.RenderPlain", "Zweitname von Group (`pygame.sprite.RenderPlain`)",
     ["crates/rustgame-sprite/src"]),
    ("sprite.RenderClear", "Zweitname von Group (`pygame.sprite.RenderClear`)",
     ["crates/rustgame-sprite/src"]),
    ("midi.Input", "MIDI-Eingang (`pygame.midi.Input`)",
     ["crates/rustgame-py/src"]),
    ("midi.Output", "MIDI-Ausgang (`pygame.midi.Output`)",
     ["crates/rustgame-py/src"]),
    ("version.PygameVersion", "Die Fassung (`pygame.version.PygameVersion`)",
     ["crates/rustgame-py/src"]),
    ("version.SDLVersion", "SDLs Fassung (`pygame.version.SDLVersion`)",
     ["crates/rustgame-py/src"]),
    ("_sdl2.video.Renderer", "Ein Zeichner (`pygame._sdl2.video.Renderer`)",
     ["crates/rustgame-py/src/python"]),
    ("_sdl2.video.Texture", "Eine Textur (`pygame._sdl2.video.Texture`)",
     ["crates/rustgame-py/src/python"]),
    ("_sdl2.video.Image", "Ein Bildausschnitt (`pygame._sdl2.video.Image`)",
     ["crates/rustgame-py/src/python"]),
]

# Was es gibt, nur unter anderem Namen oder in anderer Form.
#
# Das hier ist **nicht** dazu da, die Zahl schoener zu machen: solche Namen
# zaehlen weder als "da" noch als "fehlt", sondern stehen in einem eigenen
# Abschnitt mit der Entsprechung daneben. Wer die Tabelle liest, soll
# sehen, was wirklich fehlt - und was nur anders heisst.
ANDERS_GEBAUT = {
    ("Rect", "move_to"): "`get_rect().with_center(p)` und die 19 Geschwister",
    ("FRect", "move_to"): "`get_frect().with_center(p)` und die 19 Geschwister",
    ("FRect", "x"): "oeffentliches Feld `r.x`",
    ("FRect", "y"): "oeffentliches Feld `r.y`",
    ("FRect", "w"): "oeffentliches Feld `r.w`",
    ("FRect", "h"): "oeffentliches Feld `r.h`",
    ("FRect", "collidedict"): "es gibt kein Python-Woerterbuch - `collideobjects`",
    ("FRect", "collidedictall"): "dito - `collideobjectsall`",
    ("Rect", "x"): "oeffentliches Feld `r.x`",
    ("Rect", "y"): "oeffentliches Feld `r.y`",
    ("Rect", "w"): "oeffentliches Feld `r.w`",
    ("Rect", "h"): "oeffentliches Feld `r.h`",
    ("Rect", "collidedict"): "es gibt kein Python-Woerterbuch - `collideobjects`",
    ("Rect", "collidedictall"): "dito - `collideobjectsall`",
    ("Color", "r"): "oeffentliches Feld `c.r`",
    ("Color", "g"): "oeffentliches Feld `c.g`",
    ("Color", "b"): "oeffentliches Feld `c.b`",
    ("Color", "a"): "oeffentliches Feld `c.a`",
    ("Color", "set_length"): "seit pygame 2.1.3 veraltet - `Color` hat hier"
                             " immer vier Kanaele",
    # Namensgleichungen: `pygame.mask.MaskType is pygame.mask.Mask` ist
    # wahr - das sind keine eigenen Typen, sondern Zweitnamen.
    # In pygame sind das blosse Namensimporte in `pygame.sprite` - sie
    # liegen hier in den Kisten, zu denen sie gehoeren.
    ("sprite", "Rect"): "`rustgame_core::Rect`, von der Kiste weitergereicht",
    ("sprite", "from_surface"): "`rustgame_math::mask::from_surface`",
    ("sprite", "get_ticks"): "`rustgame_core::time::get_ticks`",
    ("sprite", "warn"): "Pythons eigenes `warnings.warn` - gibt es in Rust nicht",
    ("mask", "MaskType"): "Zweitname von `Mask` (in pygame ist es dasselbe Objekt)",
    ("font", "FontType"): "Zweitname von `Font`",
    ("mixer", "ChannelType"): "Zweitname von `Channel`",
    ("mixer", "SoundType"): "Zweitname von `Sound`",
    ("font", "SysFont"): "`sys_font` - `SysFont` waere in Rust eine Warnung"
                         " (`non_snake_case`)",
    # In pygame selbst abgeschafft: der Aufruf warnt, dass die Wirkung
    # entfernt wurde. Vertauschen (`v.xy`) geht ohnehin immer.
    ("math", "enable_swizzling"): "in pygame abgeschafft - der Aufruf warnt,"
                                  " dass die Wirkung entfernt wurde",
    ("math", "disable_swizzling"): "dito",
    ("math", "VectorIterator"): "Python-Hilfstyp; in Rust uebernimmt das"
                                " `Iterator`",
    ("system", "get_power_state"):
        "antwortet `None` - \"nicht zu ermitteln\"; den Akku wirklich zu"
        " lesen braeuchte `GetSystemPowerStatus` ueber FFI, und pygames"
        " eigener Test laesst `None` ausdruecklich zu",
    ("system", "get_pref_locales"):
        "liest `LC_ALL`, `LC_MESSAGES`, `LANG`, `LANGUAGE` statt"
        " `GetUserPreferredUILanguages` - auf Windows sind die"
        " gewoehnlich leer, dann kommt eine leere Liste",
}

# Hier standen neun Eintraege fuer `Surface`, und **alle neun waren
# veraltet** - der haeufigste Fehler in diesem Projekt, diesmal in der
# Tabelle, die die Abdeckung ausweist:
#
# ```text
# width, height, size                       "-> get_width()" usw.
# get_palette, get_palette_at,
# set_palette, set_palette_at               "keine Palettenflaechen - nur ARGB8888"
# set_masks, set_shifts                     "das Format liegt fest"
# ```
#
# Jeder Satz stimmte, als er geschrieben wurde. Inzwischen gibt es die
# Eigenschaften `width`, `height` und `size` wirklich, es gibt
# Palettenflaechen, und `set_masks`/`set_shifts` sagen **wortgleich**
# dasselbe wie pygame:
#
# ```text
# TypeError: The surface masks are read-only in SDL2
# TypeError: The surface shifts are read-only in SDL2
# ```
#
# Damit zaehlen alle neun als *da* - und die Zahl, die vorher 598 von 614
# war, sagt jetzt die Wahrheit. Eine Tabelle, die Ausnahmen von Hand
# pflegt, veraltet genau so wie ein Vermerk im Quelltext; sie gehoert bei
# jeder Durchsicht mit geprueft.

# Den Namen gibt es, aber er tut nichts ausser einen klaren Fehler zu
# melden. Das zaehlt **nicht** als vorhanden.
#
# Warum ueberhaupt eintragen, statt die Funktion wegzulassen? Weil ein
# Programm, das sie aufruft, sonst gar nicht erst uebersetzt und man
# raten muss, woran es liegt. So steht die Begruendung in der
# Fehlermeldung - und hier in der Tabelle, damit die Zahl nicht luegt.
# Namen am Hauptmodul, die keine Aufrufe sind und trotzdem zaehlen.
#
# Die Regel in `namen_von` lautet: Konstanten eines Moduls interessieren
# hier nicht, denn dafuer gibt es `gen_konstanten.py`. Am **Hauptmodul**
# stimmt sie nicht ganz: `pygame.SDL` ist eine `SDLVersion`, `pygame.rev`
# eine Zeichenkette - keine Konstanten im Sinne von `K_SPACE`, sondern
# Auskuenfte. Beide fehlten in rustgame, und nichts hat es gemeldet.
HAUPT_WERTE = {"SDL", "rev"}

NUR_FEHLER = {
    # **Diese Liste ist leer, und das ist das Ergebnis.**
    #
    # Hier standen zuletzt drei Namen: `camera.Camera`, `midi.Input` und
    # `midi.Output`. Alle drei mit derselben Begruendung - ein Geraet
    # brauche den Zugriff des Betriebssystems (DirectShow, V4L2,
    # PortMidi), und der ginge nur ueber FFI; der Arbeitsbereich habe kein
    # `unsafe`.
    #
    # Der erste Teil war wahr, der zweite war **dreimal dieselbe
    # Verwechslung**: `unsafe` braucht irgendwer, aber nicht dieses
    # Programm. Die Kiste `midir` spricht mit WinMM, ALSA und CoreMIDI,
    # die Kiste `nokhwa` mit Media Foundation, V4L2 und AVFoundation -
    # beide hinter einer sicheren Schnittstelle, beide ohne eine Zeile
    # `unsafe` in diesem Arbeitsbereich. Vorher war `system.get_total_ram`
    # mit genau demselben Satz gefallen (`sysinfo`); das war die erste von
    # vier.
    #
    # Was jedes Mal die Begruendung zum Einsturz brachte, war keine neue
    # Ueberlegung, sondern **eine Suche nach einer Kiste**. Die Notiz
    # hatte nie danach gesucht; sie hatte aus "ich muesste FFI schreiben"
    # geschlossen, dass niemand es schon geschrieben hat.
    #
    # Gemessen, nachher:
    #
    #     rustgame.camera.list_cameras()  ['USB  Live camera']
    #     rustgame.camera.Camera(..).get_image()
    #         (320, 240), 32 Bit, Masken (ff0000, ff00, ff, 0), nicht schwarz
    #     rustgame.midi.get_count()       1
    #     rustgame.midi.Output(0).note_on(60, 100)   klingt
    #
    # **Der Massstab, und er ist strenger als "der Name ist da".**
    #
    # Ein Name zaehlt, wenn er **seine Arbeit tut**. Einer, der nur einen
    # Fehler wirft, zaehlt als fehlend - auch wenn `dir()` ihn zeigt und
    # `hasattr` wahr sagt. Die Probe dazu ist einfach: **tut pygame auf
    # dieser Maschine mehr?**
    #
    # Gemessen:
    #
    #     pygame.camera.list_cameras()   ['USB  Live camera']
    #     pygame.camera.Camera(..).get_image()   (320, 240), 32 Bit
    #     pygame.midi.get_count()        2
    #       (b'MMSystem', b'Microsoft MIDI Mapper', 0, 1, 0)
    #       (b'MMSystem', b'Microsoft GS Wavetable Synth', 0, 1, 0)
    #
    # Es gibt hier also eine Kamera und zwei MIDI-Ausgaenge, und pygame
    # benutzt sie. Wer dieselben Klassen anbietet und bei jedem Aufruf
    # einen Fehler wirft, hat die Namen und nicht die Sache.
    #
    # `AbstractCamera` war der Gegenfall und fiel eine Runde frueher:
    # eine abstrakte Basisklasse tut ihre Arbeit, indem sie abstrakt ist.
    # Sie hat dieselben sieben Pflichtmethoden wie pygames, und davon
    # laesst sich erben - mehr ist von ihr nicht verlangt.
    # **Hier standen sieben Namen, und keiner davon war eine Luecke.**
    #
    # Der Vermerk lautete: `flash`, `set_modal_for`, `opacity` (Setzer),
    # `opengl`, `maximize`, `minimize` und `restore` tun auf keinem
    # Fensterstueck je ihre Arbeit - minifb habe fuer keines einen Aufruf.
    # Das stimmte fuer minifb und war die falsche Frage.
    #
    # **Gemessen unter demselben Treiber, mit dem alles andere gemessen
    # wird** (`SDL_VIDEODRIVER=dummy` dort, `RUSTGAME_VIDEODRIVER=dummy`
    # hier):
    #
    #     Name                pygame                      rustgame
    #     flash               error: not supported        gleich
    #     maximize            None                        None
    #     minimize            None                        None
    #     restore             None                        None
    #     opacity lesen       1.0                         1.0
    #     opacity setzen      error: not supported        gleich
    #     opengl              False                       False
    #     set_modal_for       error: not supported        gleich
    #
    # Acht Proben, acht Mal dasselbe. **pygame selbst verweigert drei von
    # ihnen** unter `dummy`, und die anderen tun nichts - genau wie hier.
    #
    # Die alte Zeile verglich minifb gegen pygame mit einem echten
    # Treiber, und das ist der einzige Vergleich, den dieses Projekt
    # sonst nirgends anstellt. Die Regel stand sogar schon daneben, zwei
    # Zeilen weiter unten: "`utility`, `always_on_top`, `show`/`hide`,
    # `set_fullscreen`/`set_windowed` und `set_icon` gehen ohne Fenster
    # (wie pygame unter `dummy`) und scheitern nur mit minifb. Halb zaehlt
    # nicht als fehlend." Nach derselben Regel zaehlen diese sieben auch.
    #
    # Das ist das fuenfte Mal, dass ein Vermerk hier etwas anderes
    # gemessen hat als das, was er behauptet.
}

# Auch hier standen veraltete Eintraege - zuletzt fuenf auf einen Streich.
# Siehe darunter:
#
# * **Die vier von `pixelcopy`** - "braucht mehrdimensionale Felder
#   (numpy); nicht messbar". Beide Haelften waren falsch geworden: das
#   Puffer-Protokoll gibt es (`puffer.rs`), und zum Messen braucht es kein
#   numpy - `pygame.tests.test_utils.buftools.Exporter` baut ein Feld mit
#   jeder Form, Schrittweite und Byteordnung, und `arrinter.Array` eines
#   mit rohem `__array_struct__`. Beide lagen die ganze Zeit daneben.
# * **`system.get_total_ram`** - "braeuchte `GlobalMemoryStatusEx` ueber
#   FFI; rustgame kommt ohne `unsafe` aus". Eine Verwechslung: `unsafe`
#   braucht **irgendwer**, aber nicht dieses Programm. Die Kiste `sysinfo`
#   tut es hinter einer sicheren Schnittstelle.
#
# Und frueher `image.load_sized_svg`, das seit `resvg` wirklich zeichnet:
#
# * `image.load_animation` - "braeuchte einen GIF- und WEBP-Dekodierer samt
#   Einzelbild-Verrechnung". Den gibt es jetzt, und er trifft pygame:
#   `animated_sample.gif` gibt in beiden Engines 10 Bilder, je 150.0
#   Millisekunden, je (312, 312).
# * `system.get_power_state` und `get_pref_locales` - "braeuchte ... ueber
#   FFI". Beide **antworten** inzwischen, nur anders: `None` heisst
#   "Ladezustand nicht zu ermitteln" (pygames eigener Test laesst das
#   ausdruecklich zu), und die Sprachliste kommt aus den
#   Umgebungsvariablen statt aus dem Betriebssystem. Sie stehen deshalb
#   unten unter *anders gebaut*, nicht unter *nur als klarer Fehler*.

# Diese Namen zaehlen nicht mit: Python-Innereien und Namen, die pygame
# aus fremden Importen in seine Module hat leaken lassen.
#
# ## Was hier einmal stand - und warum es wegmusste
#
# Die erste Zeile dieser Menge war lange:
#
#     "get_view", "get_buffer", "_from_buffer", "get_locked", "lock",
#     "unlock", "mustlock", "get_locks", "_pixels_address",
#     "premul_alpha_ip",
#
# mit der Begruendung, das seien "Dinge, die es in Rust nicht geben kann
# (Puffer-Protokoll, C-API, numpy-Bruecken)". **Acht dieser Namen gibt es
# inzwischen, und sie tun ihre Arbeit** - `get_view`, `get_buffer`,
# `get_locked`, `lock`, `unlock`, `mustlock`, `get_locks` und
# `premul_alpha_ip`.
#
# Sie hier stehen zu lassen hiesse, die eigene Zahl zu schoenen: eine
# Frage, die nicht gestellt wird, kann nicht mit "fehlt" beantwortet
# werden - aber auch nicht mit "da". Acht erledigte Namen zaehlten nicht
# mit, und `Surface` stand mit 48 von 48 da, obwohl es 56 von 56 waren.
#
# **Das Messgeraet gehoert mitgeprueft.** Es ist das dritte Mal in diesem
# Projekt, dass eine Zahl zu gut aussah, weil das Werkzeug zu wenig
# gefragt hat; die beiden anderen Male waren sechs Module mit "0 von 0"
# und eine Liste veralteter Ausnahmen.
# **Hier stand eine Menge `EGAL`**, und sie ist der zehnte blinde Fleck
# dieses Werkzeugs gewesen. Sie nahm sieben Namen aus - `os`, `sys`,
# `platform`, `warnings`, `ABC`, `abstractmethod`, `errorfnc` - mit der
# Begruendung, kein Programm schreibe `pygame.camera.os`.
#
# Die Begruendung stimmt sogar. Nur ist sie **eine Liste von Hand**, und
# die hat hier zusammen mit dem Konstantenfilter (siehe `namen_von`) fuenf
# wirklich fehlende Namen gedeckt. Jetzt zaehlt jeder oeffentliche Name,
# und die sieben sind in rustgame schlicht vorhanden - sie kosten je eine
# Zeile `import`.
#
# **Ein Filter ist auch dann eine Nennung, wenn er nur wegnimmt.**


def namen_von(wurzel, modulname):
    """Alle oeffentlichen Namen eines Moduls oder Typs unter `wurzel`.

    `modulname` darf Punkte enthalten (`math.Vector3`) - dann wird der
    Weg Schritt fuer Schritt gegangen. So laesst sich eine **Klasse**
    abfragen und nicht nur das Modul, in dem sie liegt.

    Dieselbe Funktion fuer beide Seiten: was fuer pygame gilt, gilt fuer
    die Huelle auch. Zwei Fassungen waeren zwei Gelegenheiten, sich zu
    unterscheiden.
    """
    if wurzel is None:
        return []
    obj = wurzel
    # **Ein leerer Name ist die Wurzel selbst.** Siehe den Eintrag `""` in
    # MODULE: das Hauptmodul wurde jahrelang nicht gefragt.
    for stueck in modulname.split(".") if modulname else []:
        obj = getattr(obj, stueck, None)
        if obj is None:
            return []
    namen = []
    for n in dir(obj):
        if n.startswith("_"):
            continue
        # **Jeder oeffentliche Name zaehlt - auch Konstanten, auch
        # mitimportierte.**
        #
        # Hier standen zwei Filter, und beide waren Listen von Hand:
        #
        # * `EGAL` nahm `os`, `sys`, `platform`, `warnings`, `ABC`,
        #   `abstractmethod` und `errorfnc` aus, weil "kein Programm
        #   `pygame.camera.os` schreibt",
        # * und wer kein Aufruf und kein Modul war, fiel als "Konstante"
        #   heraus mit dem Verweis, die kaemen aus `gen_konstanten.py`.
        #
        # Der **zehnte blinde Fleck** sass genau dort. `gen_konstanten.py`
        # erzeugt die Konstanten des Hauptmoduls und von `locals` - nicht
        # die der Untermodule. Also fragte niemand nach `font.UCS4`,
        # `base.HAVE_NEWBUF`, `_freetype.BBOX_EXACT`, `midi.MIDIIN`,
        # `sysfont.is_init`. Alle fuenf fehlten, und die Tabelle sagte
        # trotzdem 100 %.
        #
        # Gefunden hat sie die Probe `Untermodule gesamt` in
        # `vergleich_module.py`, die stumpf `dir()` gegen `dir()` stellt.
        # Lehre: **was gelesen werden kann, darf nicht genannt werden** -
        # und ein Filter ist auch dann eine Nennung, wenn er nur wegnimmt.
        namen.append(n)
        continue
        # **Echte Untermodule zaehlen mit.** `pygame.surflock` ist kein
        # Aufruf und keine Konstante, sondern ein Modul - und es hat hier
        # gefehlt, ohne dass irgendetwas es gemeldet haette.
        #
        # "Echt" heisst: **der Name des Dings ist der Weg dorthin.** Ohne
        # diese Pruefung meldete die erste Fassung zehn Luecken, von denen
        # keine eine war - `sprite.py` schreibt `import pygame` und
        # `import types`, `midi.py` `import atexit`, `sysfont.py`
        # `difflib` und `itertools`, und alle vier stehen danach in
        # `dir()`. Ein Netz, das weiter wirft, faengt auch, was kein Fisch
        # ist.
        #
        # Gefragt wird nach dem **Namen** und nicht nach dem Typ, und das
        # ist ebenfalls gemessen: `rustgame_py.surfarray` ist kein Modul,
        # sondern ein Platzhalter, der beim ersten Zugriff nachlaedt (die
        # Datei wuerde beim Laden `pixelcopy` importieren und sich dabei
        # selbst im Kreis fangen). Ein `isinstance(.., ModuleType)` haette
        # ihn als fehlend gemeldet - und damit eine Luecke behauptet, die
        # es nicht gibt.
        eigen = getattr(obj, "__name__", None)
        if eigen and getattr(wert, "__name__", "") == "%s.%s" % (eigen, n):
            namen.append(n)
            continue
        # Konstanten eines Moduls interessieren hier nicht - die kommen aus
        # `gen_konstanten.py`. **Mit zwei Ausnahmen am Hauptmodul**, und die
        # Ausnahme ist gemessen: `pygame.SDL` ist eine `SDLVersion` und
        # `pygame.rev` eine Zeichenkette, also beides "Konstanten" nach
        # dieser Regel - und beide fehlten, bis diese Zeile dastand.
        if modulname == "" and n in HAUPT_WERTE:
            namen.append(n)
    return sorted(set(namen))


def pygame_namen(modulname):
    """Was echtes pygame unter diesem Namen hat.

    **`import pygame` laedt nicht alles mit.** `gfxdraw`, `freetype`,
    `midi`, `camera` und `_sdl2.video` sind eigene Module, die erst auf
    Verlangen geladen werden - `getattr(pygame, "gfxdraw")` gibt `None`,
    solange niemand sie eingelesen hat.

    Das war der Grund, warum sechs Module jahrelang mit `0 von 0` in der
    Tabelle standen, also gar nicht: ein Modul ohne Namen kann keine
    vermissen. Ein fehlender Eintrag faellt irgendwann auf, eine fehlende
    **Frage** nie.
    """
    teile = modulname.split(".")
    for i in range(len(teile)):
        try:
            importlib.import_module("pygame." + ".".join(teile[: i + 1]))
        except Exception:                            # noqa: BLE001
            break
    return namen_von(pygame, modulname)


def huellen_namen(modulname):
    """Was `import rustgame_py` unter diesem Namen hergibt.

    **Mit demselben Anlauf wie bei pygame.** [`pygame_namen`] importiert
    die Untermodule erst, weil `import pygame` sie nicht mitlaedt; hier
    stand nur ein `getattr`, und damit sah die Tabelle jedes Modul als
    leer an, das erst beim Importieren entsteht.
    `pygame._sprite` ist so eines (es warnt beim Import, darf also nicht
    beim Laden schon laufen) - es stand mit **0 von 18** da und hat alle
    achtzehn.

    Zwei verschiedene Anlaeufe fuer dieselbe Frage sind zwei
    Gelegenheiten, sich zu unterscheiden.
    """
    if rustgame_py is None:
        return set()
    teile = modulname.split(".") if modulname else []
    for i in range(len(teile)):
        try:
            importlib.import_module("rustgame_py." + ".".join(teile[: i + 1]))
        except Exception:                            # noqa: BLE001
            break
    return set(namen_von(rustgame_py, modulname))


# ---------------------------------------------------------------------------
#  Was rustgame hat
# ---------------------------------------------------------------------------

RUST_MUSTER = re.compile(
    r"^\s*pub(?:\(crate\))?\s+(?:fn|struct|enum|const|type|trait)\s+([A-Za-z_][A-Za-z0-9_]*)",
    re.M,
)

# Ein Teil der API entsteht aus Makros: `rect.rs` erzeugt die 20 virtuellen
# Attribute (`center`, `topleft`, ...) mit je drei Methoden aus einem
# einzigen Makro. Im Quelltext steht dort nur `pub fn $lesen` - der echte
# Name taucht ausschliesslich im **Aufruf** auf. Wer nur nach `pub fn`
# sucht, uebersieht sechzig Methoden und meldet die Abdeckung zu niedrig.
MAKRO_MUSTER = re.compile(
    r"^\s*[a-z_]+_attr!\(\s*([a-z_0-9]+)\s*,\s*([a-z_0-9]+)\s*,\s*([a-z_0-9]+)",
    re.M,
)
# In der Python-Huelle stehen die Methoden **ohne** `pub`: innerhalb eines
# `#[pymethods]`-Blocks macht das Makro sie oeffentlich, und `pub fn`
# waere dort sogar eine Warnung. Das obige Muster findet dort also
# nichts - und genau deshalb hat das Hinzufuegen der Huellen-Pfade zur
# Tabelle zunaechst **keine einzige** Zeile geaendert. Ein Werkzeug, das
# auf eine Datei schaut und nichts daraus liest, sieht genauso aus wie
# eines, in dem nichts steht.
#
# Fuer Dateien unterhalb von `rustgame-py` gilt deshalb das lockere
# Muster. Die paar privaten Hilfsfunktionen, die es mitnimmt
# (`math_fehler`, `g_zahl`), schaden nicht: gezaehlt wird ein Name nur,
# wenn pygame ihn auch hat.
HUELLE_MUSTER = re.compile(r"^\s*(?:pub\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)", re.M)
HUELLE_TEIL = "rustgame-py"
PLATZHALTER = "**Noch nicht umgesetzt.**"


def rust_namen(pfade):
    """Sammelt die oeffentlichen Namen aus Rust-Dateien.

    Eine Datei, die noch den Platzhalter-Satz im Kopf traegt, zaehlt als
    leer - sonst meldete die Tabelle Abdeckung, wo nur ein Geruest steht.
    """
    namen = set()
    fertig = False
    for p in pfade:
        voll = os.path.join(RUSTGAME, p)
        dateien = []
        if os.path.isdir(voll):
            for wurzel, _, ds in os.walk(voll):
                dateien += [os.path.join(wurzel, d) for d in ds if d.endswith(".rs")]
        elif os.path.isfile(voll):
            dateien = [voll]
        for d in dateien:
            with open(d, "r", encoding="utf-8") as f:
                text = f.read()
            if PLATZHALTER in text[:4000]:
                continue
            fertig = True
            muster = HUELLE_MUSTER if HUELLE_TEIL in d else RUST_MUSTER
            namen |= set(muster.findall(text))
            for treffer in MAKRO_MUSTER.findall(text):
                namen |= set(treffer)
    return namen, fertig


def passt(py_name, rust):
    """Gibt es den pygame-Namen auf der Rust-Seite?

    pygame schreibt `set_at`, Rust auch - das trifft meistens direkt. Ein
    paar Namen sind in Rust Schluesselwoerter oder heissen anders; die
    stehen in `ANDERS`.
    """
    ANDERS = {
        "move": "move_",
        "type": "typ",
        "copy": "copy",
        "tostring": "tobytes",
        "fromstring": "frombytes",
        "blit": "blit",
    }
    kandidaten = {py_name, ANDERS.get(py_name, py_name)}
    # `get_rect(center=..)` wird in Rust zu `get_rect().with_center(..)`.
    kandidaten |= {"with_" + py_name, py_name.rstrip("_")}
    return bool(kandidaten & rust)


def alle_untermodule():
    """Jedes Untermodul, das `pygame` mitbringt - **gelesen, nicht genannt**.

    ## Der neunte blinde Fleck

[`MODULE`] ist von Hand gepflegt, und das ist fuer **Module** falsch,
    denn die Liste kann nicht wissen, was pygame alles mitbringt.

    **Hier stand "und das ist fuer Klassen richtig".** Der Satz war der
    bequeme Platz fuer den **elften** blinden Fleck: nachgezaehlt fragte
    `MODULE` 24 Klassenobjekte ab, erreichbar sind 45, und die 21 nie
    gefragten tragen zusammen **354 Namen**. Darunter `pygame.error`
    selbst, `ftfont.Font` (63 Namen), `joystick.Joystick`,
    `key.ScancodeWrapper`, `version.SoftwareVersion`,
    `math.VectorElementwiseProxy`. Siehe [`alle_klassen`] - die Liste wird
    jetzt auch dort gelesen.

    Nachgezaehlt: `pygame` hat **sechzig** Untermodule, `MODULE` nannte
    gut die Haelfte. Neun waren in rustgame gar nicht importierbar und
    standen in keiner Messung:

    ```text
    __briefcase      ein Hilfspaket fuer briefcase
    __pyinstaller    der Haken, mit dem PyInstaller pygames Daten findet
    _camera          die C-Seite von pygame.camera
    _camera_opencv   die OpenCV-Seite - **scheitert bei pygame ebenso**
    _data_classes    PowerState
    examples         die Beispielprogramme
    macosx           Video_AutoInit
    newbuffer        BufferMixin und die PyBUF_*-Zahlen
    pypm             die PortMidi-Bindung hinter pygame.midi
    ```

    Dieselbe Lehre wie bei `pygame_tests.py`, wo die Dateiliste abgetippt
    war: **was gelesen werden kann, darf nicht genannt werden.** Diese
    Funktion liest, [`MODULE`] ergaenzt nur um Klassen und um das, was
    nicht als eigenes Modul daherkommt.
    """
    import pkgutil

    namen = sorted(m.name for m in pkgutil.iter_modules(pygame.__path__))
    # `tests` ist pygames **eigene Testsuite**. Sie gehoert nicht zur
    # Schnittstelle, und `pygame_tests.py` fahrt sie ohnehin ganz.
    return [n for n in namen if n != "tests"]


def gehoert_zum_paket(klasse):
    """Gehoert diese Klasse zu pygame - oder ist sie nur mitimportiert?

    Der naheliegende Test ist `__module__.startswith("pygame")`, und er war
    der **zwoelfte** blinde Fleck - diesmal in einem Werkzeug, das keine
    Stunde alt war. Gemessen:

    ```text
    pygame._sdl2.controller.Controller.__module__      'builtins'
    pygame._sdl2.controller_old.Controller.__module__  'pygame._sdl2.controller_old'
    ```

    **Cython schreibt `builtins`** in `__module__`, wenn die Pyx-Datei kein
    `# cython: ...`-Modul nennt. Die neue `Controller`-Klasse - fuenfzehn
    Namen, eine mehr als die alte - fiel damit durch das Netz, und die
    Tabelle sagte trotzdem 100 %.

    Die Regel hier ist lesend statt nennend: **eine Klasse gehoert
    anderswohin nur dann, wenn sie dort auch zu finden ist.**
    `abc.ABC` behauptet `abc` und steht dort - also fremd.
    `Controller` behauptet `builtins` und steht dort **nicht** - also
    unsere.
    """
    heim = getattr(klasse, "__module__", "") or ""
    if heim.startswith("pygame"):
        return True
    modul = sys.modules.get(heim)
    if modul is None:
        return True
    return getattr(modul, getattr(klasse, "__name__", ""), None) is not klasse


def alle_klassen():
    """Jede erreichbare Klasse - **gelesen, nicht genannt**.

    ## Der elfte blinde Fleck

    Die Module liest `alle_untermodule` seit dem neunten blinden Fleck.
    Die **Klassen** standen weiter von Hand in [`MODULE`], mit dem Vermerk,
    das sei dort richtig. Nachgezaehlt:

    ```text
    MODULE fragt      24 Klassenobjekte
    erreichbar sind   45
    nie gefragt       21  -  zusammen 354 Namen
    ```

    Darunter nichts Entlegenes: `pygame.error` (die Fehlerklasse des ganzen
    Pakets), `ftfont.Font` mit 63 Namen, `joystick.Joystick`,
    `key.ScancodeWrapper`, `math.VectorElementwiseProxy`,
    `version.SoftwareVersion`, `system.PowerState`, beide `pypm`-Klassen und
    die zwei frischen aus `newbuffer`.

    ## Wie gezaehlt wird

    Gesucht wird in jedem Modul nach Attributen, die ein `type` sind und
    deren `__module__` zum Paket gehoert - **und dedupliziert wird nach dem
    Objekt**, nicht nach dem Namen. Das ist noetig, weil fast jede Klasse
    unter mehreren Wegen steht: `pygame.Channel`, `pygame.mixer.Channel` und
    `pygame.mixer.ChannelType` sind **ein** Objekt. Nach Namen gezaehlt
    kaeme 674 heraus, und 320 davon waeren dieselbe Klasse zweimal.

    Behalten wird der **kuerzeste** Weg - `Surface` statt
    `surface.SurfaceType` -, weil er der ist, den ein Programm schreibt.

    ## Die Falle beim Messen dieser Zahl

    Der erste Lauf meldete **25** nie gefragte Klassen und 514 Namen, und
    vier davon waren falsch: `freetype.Font`, `geometry.Circle` und zwei
    `_sdl2`-Klassen **stehen** in `MODULE`. Mein Messskript loeste die Pfade
    mit `getattr`, ohne das Modul vorher zu importieren - und
    `pygame.freetype` ist ohne Import nicht da. `pygame_namen` macht es
    richtig; die Messung muss es genauso machen. **Ein Messgeraet, das
    anders fragt als das geprueefte Werkzeug, misst etwas anderes.**
    """
    import pkgutil

    quellen = [("", pygame)]
    for n in alle_untermodule():
        try:
            quellen.append((n, importlib.import_module("pygame." + n)))
        except Exception:                            # noqa: BLE001
            continue
    # `_sdl2` ist das einzige Untermodul mit eigenen Kindern, und seine
    # Klassen (`Renderer`, `Texture`, `AudioDevice`, ...) wohnen dort.
    try:
        sdl2 = importlib.import_module("pygame._sdl2")
        for kind in pkgutil.iter_modules(sdl2.__path__):
            try:
                quellen.append(
                    ("_sdl2." + kind.name,
                     importlib.import_module("pygame._sdl2." + kind.name)))
            except Exception:                        # noqa: BLE001
                continue
    except Exception:                                # noqa: BLE001
        pass

    gefunden = {}
    for ort, modul in quellen:
        for name in dir(modul):
            if name.startswith("_"):
                continue
            wert = getattr(modul, name, None)
            if not isinstance(wert, type):
                continue
            if not gehoert_zum_paket(wert):
                continue
            weg = ("%s.%s" % (ort, name)) if ort else name
            if id(wert) not in gefunden or len(weg) < len(gefunden[id(wert)]):
                gefunden[id(wert)] = weg
    return sorted(gefunden.values())


def genannte_klassen():
    """Welche Klassenobjekte [`MODULE`] schon abfragt.

    Verglichen wird nach **Objekt**: `MODULE` nennt `mixer.Sound`, und damit
    ist auch `pygame.Sound` und `mixer.SoundType` abgedeckt - es ist dasselbe
    Objekt.
    """
    aus = set()
    for modul, _, _ in MODULE:
        teile = modul.split(".") if modul else []
        # Erst importieren, dann holen - genau wie `pygame_namen`. Ohne das
        # ist `pygame.freetype` nicht da, und die Klasse sieht aus wie nie
        # gefragt.
        for k in range(len(teile)):
            try:
                importlib.import_module("pygame." + ".".join(teile[: k + 1]))
            except Exception:                        # noqa: BLE001
                break
        obj = pygame
        for stueck in teile:
            obj = getattr(obj, stueck, None)
            if obj is None:
                break
        if isinstance(obj, type):
            aus.add(id(obj))
    return aus


def main():
    zeilen = []
    zeilen.append("# API-Abdeckung: pygame-ce %s gegen rustgame" % pygame.version.ver)
    zeilen.append("")
    zeilen.append("**Erzeugt** von `tools/api_coverage.py` - nicht von Hand aendern.")
    zeilen.append("")
    zeilen.append("Die linke Spalte kommt aus dem **installierten pygame** (per")
    zeilen.append("`dir()`), nicht aus einer gepflegten Liste. Sie kann also weder")
    zeilen.append("veralten noch geschoenfaerbt werden.")
    zeilen.append("")
    zeilen.append("Ein Haken heisst: **es gibt den Namen**. Ob sich alles dahinter")
    zeilen.append("bis ins letzte Pixel wie pygame verhaelt, beantworten die")
    zeilen.append("Referenzbilder (`cargo test --test golden`) und der differenzielle")
    zeilen.append("Fuzzer (`cargo test --test fuzz`), nicht diese Tabelle.")
    zeilen.append("")

    gesamt_da = gesamt = 0
    uebersicht = []
    abschnitte = []

    # **Die gelesenen Module zuerst eintragen**, dann die von Hand
    # gepflegten Eintraege darueber - so gewinnt eine ausdrueckliche
    # Beschreibung, und was neu dazukommt, haengt hinten dran und faellt
    # auf. Siehe `alle_untermodule`.
    genannt = {m for m, _, _ in MODULE}
    gelesen = [
        (n, "Untermodul (`pygame.%s`)" % n, ["crates/rustgame-py/src"])
        for n in alle_untermodule()
        if n not in genannt
    ]
    # **Und die Klassen genauso** - siehe `alle_klassen`. Verglichen wird
    # nach dem Objekt, damit `mixer.Sound` aus MODULE nicht gleich dreimal
    # als Tabellenzeile erscheint.
    schon = genannte_klassen()
    gelesene_klassen = []
    for weg in alle_klassen():
        teile = weg.split(".")
        obj = pygame
        for stueck in teile:
            obj = getattr(obj, stueck, None)
            if obj is None:
                break
        if obj is None or id(obj) in schon:
            continue
        schon.add(id(obj))
        gelesene_klassen.append(
            (weg, "Klasse (`pygame.%s`)" % weg, ["crates/rustgame-py/src"]))
    for modul, titel, pfade in MODULE + gelesen + gelesene_klassen:
        py = pygame_namen(modul)
        if not py:
            continue
        kern, fertig = rust_namen(pfade)
        huelle = huellen_namen(modul)
        anders = [n for n in py if (modul, n) in ANDERS_GEBAUT]
        leer = [n for n in py if (modul, n) in NUR_FEHLER]
        frei = [n for n in py if n not in anders and n not in leer]
        # **Die Huelle entscheidet.** Siehe die Begruendung bei MODULE.
        da = [n for n in frei if n in huelle]
        nur_kern = [n for n in frei
                    if n not in da and fertig and passt(n, kern)]
        fehlt = [n for n in frei if n not in da and n not in nur_kern]
        py = [n for n in py if n not in anders]
        gesamt_da += len(da)
        gesamt += len(py)
        anteil = 100.0 * len(da) / len(py) if py else 0.0
        uebersicht.append((titel, len(da), len(py), anteil))

        abschnitte.append("## %s" % titel)
        abschnitte.append("")
        abschnitte.append("%d von %d (%.0f %%)" % (len(da), len(py), anteil))
        abschnitte.append("")
        if da:
            abschnitte.append("**Da:** " + ", ".join("`%s`" % n for n in da))
            abschnitte.append("")
        if nur_kern:
            abschnitte.append(
                "**Nur im Kern** (aus Python nicht erreichbar, zaehlt als"
                " fehlend): " + ", ".join("`%s`" % n for n in nur_kern))
            abschnitte.append("")
        if fehlt:
            abschnitte.append("**Fehlt:** " + ", ".join("`%s`" % n for n in fehlt))
            abschnitte.append("")
        if leer:
            abschnitte.append("**Nur als klarer Fehler** (zaehlt als fehlend):")
            abschnitte.append("")
            for n in leer:
                abschnitte.append("- `%s` -> %s" % (n, NUR_FEHLER[(modul, n)]))
            abschnitte.append("")
        if anders:
            abschnitte.append("**Anders gebaut** (zaehlt weder als da noch als fehlend):")
            abschnitte.append("")
            for n in anders:
                abschnitte.append("- `%s` -> %s" % (n, ANDERS_GEBAUT[(modul, n)]))
            abschnitte.append("")

    zeilen.append("## Uebersicht")
    zeilen.append("")
    zeilen.append("| Bereich | da | von | Anteil |")
    zeilen.append("|---|---:|---:|---:|")
    for titel, d, g, a in uebersicht:
        zeilen.append("| %s | %d | %d | %.0f %% |" % (titel, d, g, a))
    zeilen.append("| **Zusammen** | **%d** | **%d** | **%.0f %%** |"
                  % (gesamt_da, gesamt, 100.0 * gesamt_da / gesamt if gesamt else 0))
    zeilen.append("")
    zeilen += abschnitte

    ziel = os.path.join(RUSTGAME, "API.md")
    with open(ziel, "w", encoding="utf-8") as f:
        f.write("\n".join(zeilen).rstrip() + "\n")

    print("pygame-ce %s" % pygame.version.ver)
    print("%d von %d Namen (%.0f %%) -> %s"
          % (gesamt_da, gesamt, 100.0 * gesamt_da / gesamt if gesamt else 0,
             os.path.relpath(ziel, ROOT)))
    for titel, d, g, a in uebersicht:
        print("   %-34s %3d/%-3d  %3.0f %%" % (titel, d, g, a))
    return 0


if __name__ == "__main__":
    sys.exit(main())
