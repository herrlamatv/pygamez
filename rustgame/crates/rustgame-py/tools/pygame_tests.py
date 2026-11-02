# -*- coding: utf-8 -*-
"""
Laesst **pygames eigene Testsuite** gegen rustgame laufen.

Das ist die vierte Pruefstufe des Projekts und die einzige, die von
**fremder** Hand stammt: `rect_test.py` hat 3566 Zeilen, `math_test.py`
3309, `color_test.py` 1593 - zusammen ueber 21.000 Zeilen, die niemand
hier geschrieben hat und die deshalb auch keine der hiesigen Annahmen
teilen.

Aufruf (aus dem Projektordner `pygamez`)::

    .venv/Scripts/python.exe rustgame/crates/rustgame-py/tools/pygame_tests.py
    .venv/Scripts/python.exe rustgame/crates/rustgame-py/tools/pygame_tests.py rect color
    .venv/Scripts/python.exe rustgame/crates/rustgame-py/tools/pygame_tests.py --pygame

Mit `--pygame` laeuft dieselbe Auswahl gegen **echtes** pygame. Das ist
kein Beiwerk: die Zahl allein sagt nichts, solange man nicht weiss, wie
viele dieser Tests auf dieser Maschine ueberhaupt durchlaufen. Manche
setzen ein echtes Fenster voraus, manche numpy - beides gibt es hier
nicht, und dann scheitern sie auf **beiden** Seiten.

Wie der Austausch funktioniert
------------------------------
Die Testdateien schreiben `import pygame`. Dieser Laeufer legt
`rustgame_py` unter dem Namen `pygame` in `sys.modules` ab, samt jedem
Untermodul unter `pygame.<name>`. Danach findet `from pygame import
Rect` die Huelle.

`pygame.tests.test_utils` wird **zweimal** geladen: einmal vor dem
Umbiegen, damit `pygame.tests` ueberhaupt auffindbar ist, und einmal
danach. Der zweite Lauf ist der wichtige - die Datei enthaelt neben
Werkzeug auch Basisklassen wie `SurfaceSubclass`, und die muessen von
der **Huelle** ableiten, nicht vom echten pygame. Die Begruendung steht
bei `hilfswerkzeug_neu_laden`; hier stand vorher, das Hilfswerkzeug
komme "weiter aus dem echten Paket", und das war zur Haelfte falsch.
"""

import inspect
import os
import re
import sys
import types
import unittest

HIER = os.path.dirname(os.path.abspath(__file__))
HUELLE = os.path.dirname(HIER)
# tools/ -> rustgame-py -> crates -> rustgame -> pygamez
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.dirname(HIER))))

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("RUSTGAME_VIDEODRIVER", "dummy")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
os.environ.setdefault("RUSTGAME_AUDIODRIVER", "dummy")
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")

# Welche Testdateien ueberhaupt in Frage kommen.
#
# Nicht dabei und warum:
#   * `gfxdraw`, `freetype`, `ftfont`, `midi`, `camera`, `controller`,
#     `geometry`, `bufferproxy`, `pixelcopy`, `surfarray`, `sndarray` -
#     Module, die rustgame ausdruecklich nicht hat (siehe `API.md`).
#   * `docs_test` prueft pygames Dokumentationsdateien, nicht seine API.
#   * `sprite_test` laedt `pygame.tests.test_utils.buftools`, das ein
#     Puffer-Protokoll auf `Surface` voraussetzt - das hat rustgame
#     nicht, und der Plan schliesst es aus.
# **Der Ordner hat 50 Dateien, hier standen 24.**
#
# Das ist derselbe Fehler wie beim Ausschluss, der zu viel traf - nur eine
# Ebene hoeher und viel teurer: was hier nicht steht, wird nicht gefahren,
# faellt nicht auf und taucht in keiner Zahl auf. Die Liste war von Hand
# gepflegt und ist beim Wachsen des Projekts stehengeblieben; `sprite`,
# `pixelarray`, `pixelcopy`, `bufferproxy` und `window` sind laengst
# gebaut und wurden trotzdem nie geprueft.
#
# Jetzt steht hier **alles**, was der Ordner hergibt. Was rustgame nicht
# hat, faellt dann eben rot aus - das ist der Sinn der Uebung. Drei
# Dateien bleiben draussen, und zwar mit Grund:
#
# * `docs` und `debug` pruefen pygames eigene Dokumentation und
#   Fehlersuchausgabe, nicht die API.
# * `image__save_gl_surface` braucht einen OpenGL-Kontext.
#
# `surfarray` und `sndarray` bleiben drin, obwohl sie numpy brauchen -
# ohne numpy ueberspringt pygames eigener Laeufer sie, und genau das soll
# auch hier passieren, statt dass jemand sie stillschweigend weglaesst.
# Die Reihenfolge, in der der Bericht liest - **nicht** die Liste dessen,
# was gefahren wird. Die kommt aus dem Ordner; siehe `alle_dateien`.
REIHENFOLGE = [
    "rect", "color", "math", "mask", "cursors", "version", "locals",
    "constants", "time", "event", "key", "mouse", "draw", "transform",
    "image", "surface", "display", "font", "joystick", "scrap", "mixer",
    "mixer_music", "blit", "system",
    # ----- die 26, die lange nie gefahren wurden -----------------------
    "base", "bufferproxy", "camera", "controller", "freetype", "ftfont",
    "geometry", "gfxdraw", "imageext", "midi", "pixelarray", "pixelcopy",
    "render", "rwobject", "sndarray", "sprite", "surfarray", "surflock",
    "sysfont", "touch", "typing", "video", "window",
]


def alle_dateien(ordner):
    """Jede `*_test.py` im Ordner - **gelesen, nicht aufgezaehlt**.

    ## Hier stand eine abgetippte Liste

    Und daneben, im Bericht, der Satz: "die Testdateien werden aus dem
    Ordner gelesen statt aufgezaehlt". Das war **nicht wahr**. Nachgezaehlt
    lagen fuenfzig `*_test.py` im Ordner und siebenundvierzig Namen in der
    Liste; **drei Dateien wurden nie gefragt**:

    ```text
    debug_test.py                   1 Test  - pygame.print_debug_info
    docs_test.py                    1 Test  - pygame.docs
    image__save_gl_surface_test.py  1 Test  - ueberspringt sich selbst
    ```

    Die ersten zwei waren echte Luecken: `pygame.print_debug_info` und
    `pygame.docs` gab es hier nicht. Und `api_coverage.py` hatte sie
    ebenfalls nicht gemeldet, weil es nie nach den Namen des **Hauptmoduls**
    fragte - zwei Messgeraete, dieselbe Luecke, derselbe Grund.
    **Ein fehlender Eintrag faellt irgendwann auf, eine fehlende Frage
    nie.**

    Das ist das sechste Mal, dass das Messgeraet selbst danebenlag. Darum
    liest diese Funktion den Ordner, und [`REIHENFOLGE`] gibt nur noch an,
    in welcher Folge der Bericht sie auflistet - was dort fehlt, haengt
    hinten dran und faellt auf.
    """
    da = sorted(
        f[: -len("_test.py")]
        for f in os.listdir(ordner)
        if f.endswith("_test.py")
    )
    bekannt = [n for n in REIHENFOLGE if n in da]
    neu = [n for n in da if n not in REIHENFOLGE]
    return bekannt + neu


def pygame_tests_ordner():
    """Wo pygames eigene Tests liegen."""
    import pygame.tests

    return os.path.dirname(pygame.tests.__file__)


def hilfswerkzeug_laden(ordner):
    """Laedt `pygame.tests.test_utils` aus dem **echten** Paket.

    Muss **vor** dem Umbiegen passieren: danach zeigt der Name `pygame`
    auf die Huelle, und `pygame.tests` gaebe es dort nicht.
    """
    import pygame.tests
    import pygame.tests.test_utils  # noqa: F401

    # **Auch das gehoert zum Werkzeug** - und es muss **hier** geladen
    # werden, nicht spaeter: nach dem Umbiegen zeigt der Name `pygame`
    # auf die Huelle, und `import pygame.newbuffer` suchte dann ein
    # Untermodul von `rustgame_py`.
    try:
        import pygame.newbuffer  # noqa: F401
    except Exception:                                # noqa: BLE001
        pass

    # Beide festhalten, damit `umbiegen` sie wieder anhaengen kann.
    return (sys.modules["pygame.tests"], sys.modules["pygame.tests.test_utils"])


def umbiegen():
    """Legt `rustgame_py` unter dem Namen `pygame` ab."""
    sys.path.insert(0, HUELLE)
    import rustgame_py

    # Die Untermodule einzeln - `from pygame.math import Vector2` sucht
    # in `sys.modules` nach `pygame.math`, nicht nach einem Feld.
    # **Auch eine Stufe tiefer** - `joystick_test.py` schreibt
    # `import pygame._sdl2.controller`, und das sucht den vollen Namen in
    # `sys.modules`. Eine Ebene genuegte, solange alle Untermodule flach
    # lagen; `_sdl2` ist das erste mit einem eigenen Untermodul.
    # Genau **zwei** Ebenen, und die zweite ohne Rekursion: die Module
    # verweisen aufeinander (`mixer.music is mixer_music`), und eine
    # naive Rekursion laeuft darin sofort in einen Ring - gemessen, 994
    # Wiederholungen bis zum `RecursionError`.
    #
    # Auf der zweiten Ebene **nur eigene Module**. Ein Untermodul, das in
    # Python geschrieben ist, haelt auch `os`, `sys` oder das Paket selbst
    # (`sprite.pygame`) als Feld; das legte `pygame.sysfont.os` und
    # `pygame.cursors.pygame` in `sys.modules` - und der letzte Name gewann
    # beim Umbenennen unten: `pygame.__name__` las sich
    # `pygame.cursors.pygame` (Runde 21, gefunden beim ersten Programm,
    # das ueber `tools/starten.py` lief).
    def eigenes(wert):
        n = getattr(wert, "__name__", "")
        return wert is not rustgame_py and n.startswith("rustgame_py.")

    def untermodule(eltern, pfad):
        for name in dir(eltern):
            wert = getattr(eltern, name)
            if isinstance(wert, types.ModuleType) and eigenes(wert):
                sys.modules[pfad + "." + name] = wert

    for name in dir(rustgame_py):
        wert = getattr(rustgame_py, name)
        if isinstance(wert, types.ModuleType):
            untermodule(wert, "pygame." + name)
    for name in dir(rustgame_py):
        wert = getattr(rustgame_py, name)
        if isinstance(wert, types.ModuleType):
            sys.modules["pygame." + name] = wert
    sys.modules["pygame"] = rustgame_py
    # **Und der Name geht mit.**
    #
    # Fuenf Tests in `font_test.py` fragen `pygame_font.__name__ ==
    # "pygame.ftfont"` und ueberspringen dann, was nur `pygame.font` kann
    # (Kontur, Schriftsystem, Laufrichtung, Punktgroesse). Das ist keine
    # Spitzfindigkeit, sondern die Art, wie diese Datei zwei Module mit
    # derselben Schnittstelle auseinanderhaelt - sie wird zweimal
    # gefahren, einmal je Modul.
    #
    # Wer ein Modul unter `pygame.ftfont` in `sys.modules` legt, muss es
    # auch so nennen; sonst ist die Umleitung halb gemacht und die Datei
    # prueft an fuenf Stellen das falsche Modul.
    #
    # Der neue Name kommt aus dem **eigenen** Namen des Moduls, nicht aus
    # dem Schluessel, unter dem es gefunden wurde: dasselbe Modul steht
    # unter mehreren Schluesseln (`pygame.mixer_music` und
    # `pygame.mixer.music`, `pygame.draw` und `pygame.ftfont.draw`), und
    # pygame nennt es `pygame.mixer_music` und `pygame.draw`.
    for voll, modul in list(sys.modules.items()):
        alt = getattr(modul, "__name__", "")
        if voll.startswith("pygame") and (alt == "rustgame_py" or alt.startswith("rustgame_py.")):
            try:
                modul.__name__ = "pygame" + alt[len("rustgame_py"):]
            except (AttributeError, TypeError):
                pass
    # `from pygame.tests.test_utils import arrinter` braucht **beides**:
    # den Eintrag in `sys.modules` (steht schon, aus dem echten Paket)
    # **und** ein Feld `tests` am Modul `pygame`. Der Importeur holt das
    # Untermodul als Attribut des Elternteils; ohne dieses Feld kommt
    # `ImportError: cannot import name 'tests' from 'rustgame_py'`.
    #
    # Das Hilfswerkzeug bleibt dabei das **echte** - es ist Werkzeug der
    # Tests, kein Prueflilng.
    # **Werkzeug bleibt Werkzeug.** `pygame.newbuffer` ist kein Stueck
    # API, sondern ein Hilfsmodul der Testsuite: es stellt `BufferMixin`,
    # `Importer` und `Exporter` bereit, mit denen die Tests das
    # Puffer-Protokoll **von aussen** anfassen. Genau wie
    # `pygame.tests.test_utils` kommt es deshalb aus dem echten Paket.
    #
    # Ohne diese Zeile laden **drei ganze Dateien nicht** - `base`,
    # `pixelarray` und `pixelcopy`, zusammen 71 Tests, die nicht rot
    # waren, sondern unsichtbar. Dass rustgame das Puffer-Protokoll nicht
    # anbietet, zeigt sich dann dort, wo es wirklich fehlt, statt schon
    # beim Import.
    for werkzeug in ("newbuffer",):
        voll = "pygame." + werkzeug
        if voll in sys.modules:
            try:
                setattr(rustgame_py, werkzeug, sys.modules[voll])
            except AttributeError:
                pass
    for name in ("pygame.tests", "pygame.tests.test_utils"):
        if name in sys.modules:
            kurz = name.rsplit(".", 1)[-1]
            eltern = sys.modules[name.rsplit(".", 1)[0]]
            try:
                setattr(eltern, kurz, sys.modules[name])
            except AttributeError:
                pass
    return rustgame_py


def hilfswerkzeug_neu_laden():
    """Laedt `pygame.tests.test_utils` **nach** dem Umbiegen noch einmal.

    Und damit widerspricht dieser Laeufer sich selbst - absichtlich, denn
    der Satz oben im Modulkopf ("kommt weiter aus dem echten Paket") ist
    **zur Haelfte falsch**, und die falsche Haelfte hat Tests umgebracht,
    die nie etwas fehlten.

    `test_utils` ist zweierlei auf einmal:

    * Werkzeug - `question()`, `prompt()`, `tostring()`. Das soll aus dem
      echten Paket kommen, es prueft nichts.
    * **Basisklassen** - `SurfaceSubclass`, `RectSubclass` und
      Geschwister, die von `pygame.Surface` bzw. `pygame.Rect` ableiten.
      Und die erbt es von dem `pygame`, das beim Laden gerade gilt.

    Wird die Datei vor dem Umbiegen geladen, leitet `SurfaceSubclass` vom
    **echten** pygame ab. Ein Test, der sie an `transform.laplacian`
    weiterreicht, scheitert dann zwangslaeufig - nicht weil die Huelle
    etwas falsch macht, sondern weil sie eine fremde Flaeche bekommt.
    Nachweisbar an einer Zeile: `SurfaceSubclass.__mro__[1] is
    rustgame_py.Surface` war `False`.

    Drei Tests in `transform` hingen daran, und vermutlich weitere in
    jeder Datei, die eine dieser Klassen benutzt. Es waren **Fehler des
    Messgeraets**, nicht des Nachbaus - die unangenehmste Sorte, weil sie
    wie echte Lucken aussehen.

    Die zweite Zeile ist genauso noetig wie die erste: `from pygame.tests
    import test_utils` holt das **Attribut** des Paketmoduls, nicht den
    Eintrag in `sys.modules`. Ohne sie bleibt die alte Fassung im Umlauf,
    und die Aenderung wirkt nicht.
    """
    import importlib.util

    alt = sys.modules.get("pygame.tests.test_utils")
    if alt is None or not getattr(alt, "__file__", None):
        return
    spec = importlib.util.spec_from_file_location(
        "pygame.tests.test_utils", alt.__file__
    )
    neu = importlib.util.module_from_spec(spec)
    sys.modules["pygame.tests.test_utils"] = neu
    spec.loader.exec_module(neu)
    # Ohne diese Zeile wirkt die vorige nicht - siehe oben.
    sys.modules["pygame.tests"].test_utils = neu


# Welche Marken einen Test von vornherein ausschliessen.
#
# **Das sind pygames eigene Regeln, nicht unsere.** In
# `pygame/tests/test_utils/run_tests.py` steht:
#
# ```python
# if not option_interactive and "interactive" not in option_exclude:
#     option_exclude += ("interactive",)
# elif "ignore" not in option_exclude:
#     option_exclude += ("ignore",)
# option_exclude += ("python3_ignore",)
# option_exclude += ("SDL2_ignore",)
# ```
#
# Wer diese Regeln nicht anwendet, misst etwas anderes als pygame und
# haelt das Ergebnis fuer eine Aussage ueber den Nachbau. Das ist hier
# teuer geworden - siehe unten.
AUSGESCHLOSSEN = {
    # Braucht einen Menschen an der Tastatur. `test_utils.question()`
    # schreibt eine Frage auf die Konsole und **wartet**.
    "interactive":
        "braucht eine Eingabe von Hand",
    # pygame markiert damit, was es selbst nicht laufen laesst.
    "ignore":
        "von pygame selbst uebersprungen",
    "subprocess_ignore":
        "von pygame selbst uebersprungen",
    "python3_ignore":
        "von pygame selbst uebersprungen",
    "SDL2_ignore":
        "von pygame selbst uebersprungen",
    # Braucht numpy-Felder.
    "array":
        "braucht numpy",
}

# Einzelne Tests, die den **Lauf zum Stehen bringen** - mit Begruendung.
#
# ## Warum das keine Sammelregel ist
#
# Hier stand zuerst `"timing"` in `AUSGESCHLOSSEN`, weil der eine haengende
# Test diese Marke traegt. Das hat funktioniert und war falsch:
# `time_test.py` markiert **seine beiden Klassen** ebenfalls mit `timing`,
# und damit fielen 14 einwandfreie Tests weg, von denen 11 gruen waren. Ein
# Ausschluss, der zehnmal mehr trifft als er soll, verbirgt mehr als er
# erklaert.
#
# Also ein Name statt einer Marke. Ein Name kostet eine Zeile mehr und sagt
# genau, was gemeint ist.
HAENGT = {
    "pgtest_event.EventModuleTestsWithTiming.test_event_wait":
        "haengt auch mit echtem pygame endlos - nachgemessen. Der "
        "vorhergehende `test_set_blocked_all` ruft `set_blocked(None)`, "
        "sperrt damit **alle** Ereignisse und nimmt es nie zurueck; das "
        "`setUp` der Timing-Klasse ruft nur `display.init()` und "
        "`event.clear()`. Danach verschluckt `post()` sein Ereignis, und "
        "`wait()` ohne Zeitschranke wartet auf eines, das nie kommt. "
        "pygames eigener Laeufer faellt nicht darauf, weil er jede Datei "
        "in einem eigenen Prozess mit 120 Sekunden Zeitschranke fahrt "
        "(`run_tests.py`, `option_time_out`) - er besteht den Test also "
        "auch nicht, er bricht ihn ab.",
}

# `|tags:foo,bar|` im Docstring einer Methode - pygames Schreibweise.
MARKEN_RE = re.compile(r"\|[tT]ags:(-?[ a-zA-Z,0-9_\n]+)\|", re.M)


def marken(modul, klasse, methode):
    """Die Marken eines einzelnen Tests - aus drei Quellen.

    Zeilengetreu nach `test_utils/test_machinery.py`: Modul, Klasse und
    Docstring der Methode, in dieser Reihenfolge, und ein `-name` **nimmt**
    eine Marke wieder weg. Die Reihenfolge ist wichtig, weil eine Klasse
    eine Modulmarke aufheben darf.
    """
    modul_marken = getattr(modul, "__tags__", [])
    klassen_marken = getattr(klasse, "__tags__", [])
    doku = inspect.getdoc(methode) or ""
    treffer = MARKEN_RE.search(doku)
    test_marken = [t.strip() for t in treffer.group(1).split(",")] if treffer else []

    zusammen = set()
    for liste in (modul_marken, klassen_marken, test_marken):
        if not liste:
            continue
        dazu = {t for t in liste if not t.startswith("-")}
        weg = {t[1:] for t in liste if t not in dazu}
        if dazu:
            zusammen.update(dazu)
        if weg:
            zusammen.difference_update(weg)
    return zusammen


def sieben(suite, modul, auch):
    """Nimmt aus `suite` alles heraus, was eine ausgeschlossene Marke traegt.

    Gibt `(gesiebte Suite, [(id, Grund), ...])` zurueck - die zweite Liste
    wird gezaehlt und gemeldet, damit ein Ausschluss **sichtbar** bleibt
    und nicht wie ein bestandener Test aussieht.

    ## Warum das hier steht und nicht nur bei pygame

    Drei Raetsel dieses Projekts hatten dieselbe Ursache, und keines sah
    danach aus:

    * `font` **haengte** den ganzen Lauf. `font_test.py:1104` traegt
      `interactive`; die Klasse fragt auf der Konsole nach und wartet.
    * `event` brach stumm ab - in Wahrheit **haengte** es, und das
      `tail` in meiner Messung verbarg den Abbruchcode. Ein einzelner
      Test, siehe `HAENGT`.
    * `scrap` meldete **-2 gute Tests**, eine unmoegliche Zahl. Die ganze
      Datei traegt `("ignore", "subprocess_ignore")`, also laesst pygame
      sie nie laufen; `setUpModule` scheitert dann, und der Zaehler zog
      mehr ab als er hatte.

    Alle drei waren **Fehler des Messgeraets**, nicht des Nachbaus. Das
    ist in diesem Projekt schon die dritte Sorte dieser Art, nach dem
    `test_utils`, das vom falschen pygame erbte.
    """
    behalten = []
    verworfen = []
    def durch(s):
        for t in s:
            if isinstance(t, unittest.TestSuite):
                durch(t)
            elif isinstance(t, unittest.TestCase):
                klasse = type(t)
                methode = getattr(klasse, t._testMethodName, None)
                m = marken(modul, klasse, methode)
                schuld = [x for x in m if x in AUSGESCHLOSSEN and x not in auch]
                if t.id() in HAENGT and "haenger" not in auch:
                    verworfen.append((t.id(), HAENGT[t.id()][:60]))
                elif schuld:
                    verworfen.append((t.id(), AUSGESCHLOSSEN[schuld[0]]))
                else:
                    behalten.append(t)
            else:
                # Ein Ladefehler kommt als `_FailedTest` herein - der bleibt.
                behalten.append(t)
    durch(suite)
    return unittest.TestSuite(behalten), verworfen


def datei_laden(pfad, modulname):
    """Laedt eine Testdatei als Modul - ohne sie zu importieren.

    **Mit einem Scheinpaket drumherum.** `ftfont_test.py` schreibt
    `from . import font_test` - es faehrt die ganze Schrift-Testdatei ein
    zweites Mal, nur gegen `pygame.ftfont`. Ohne ein Paket, auf das sich
    der Punkt beziehen kann, scheitert der Import, und 72 Tests fallen
    aus, bevor einer gelaufen ist.

    Das Paket heisst `pgtest`, sein Suchpfad ist pygames Testordner, und
    die Dateien behalten ihren Namen `pgtest_<was>` - sonst wuerden die
    Ausnahmelisten weiter oben nicht mehr greifen.
    """
    import importlib.util
    import types

    paket = sys.modules.get("pgtest")
    if paket is None:
        paket = types.ModuleType("pgtest")
        paket.__path__ = [os.path.dirname(pfad)]
        sys.modules["pgtest"] = paket

    spec = importlib.util.spec_from_file_location(modulname, pfad)
    modul = importlib.util.module_from_spec(spec)
    modul.__package__ = "pgtest"
    sys.modules[modulname] = modul
    spec.loader.exec_module(modul)
    return modul


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    echtes_pygame = "--pygame" in sys.argv
    ausfuehrlich = "-v" in sys.argv
    # `--zeige N` druckt die ersten N Fehler je Datei - damit sichtbar
    # wird, ob hinter hundert roten Tests **eine** Ursache steckt.
    zeige = 0
    for a in sys.argv[1:]:
        if a.startswith("--zeige"):
            zeige = int(a.split("=", 1)[1]) if "=" in a else 5
    # `--spur` druckt zu jedem gezeigten Fehler die ganze Spur.
    spur = "--spur" in sys.argv
    # **`--uebersprungen`**: nur die Namen der uebersprungenen Tests, einer
    # je Zeile. Damit laesst sich fragen, was die Zahl nicht beantwortet:
    # **sind es dieselben?** 121 gegen 121 kann bedeuten, dass rustgame
    # einen Test ueberspringt, den pygame fahrt - und dann ist er nicht
    # gruen, sondern unsichtbar. Dieselbe Sorte Fehler wie eine Testdatei,
    # die gar nicht geladen wird.
    nur_uebersprungen = "--uebersprungen" in sys.argv
    if spur and not zeige:
        zeige = 5
    # `--auch=timing,interactive` nimmt Marken wieder herein - fuer den
    # Fall, dass man sie einzeln von Hand ansehen will.
    auch = set()
    for a in sys.argv[1:]:
        if a.startswith("--auch"):
            auch |= {t.strip() for t in a.split("=", 1)[1].split(",") if t.strip()}
    args = [a for a in args if not a.isdigit()]

    ordner = pygame_tests_ordner()
    hilfswerkzeug_laden(ordner)
    if not echtes_pygame:
        umbiegen()
        hilfswerkzeug_neu_laden()

    auswahl = args or alle_dateien(ordner)
    gesamt = geglueckt = gescheitert = fehlerhaft = uebersprungen = 0
    bericht = []

    for name in auswahl:
        pfad = os.path.join(ordner, name + "_test.py")
        if not os.path.isfile(pfad):
            bericht.append((name, "keine Datei", 0, 0, 0, 0))
            continue
        try:
            modul = datei_laden(pfad, "pgtest_" + name)
        except Exception as e:                      # noqa: BLE001
            # Schon das Laden kann scheitern - etwa, wenn die Datei
            # beim Import etwas anfasst, das es hier nicht gibt.
            bericht.append((name, "%s: %s" % (type(e).__name__, e), 0, 0, 0, 0))
            continue

        lader = unittest.TestLoader()
        suite = lader.loadTestsFromModule(modul)
        suite, verworfen = sieben(suite, modul, auch)
        lauf = unittest.TextTestRunner(
            stream=open(os.devnull, "w", encoding="utf-8"),
            verbosity=2 if ausfuehrlich else 0,
        )
        e = lauf.run(suite)
        if zeige:
            for art, liste in (("FEHLER", e.errors), ("FALSCH", e.failures)):
                for fall, text in liste[:zeige]:
                    letzte = [z for z in text.strip().splitlines() if z.strip()][-1]
                    print("  %s %-52s %s" % (art, fall, letzte[:110]))
                    # `--spur` zeigt die ganze Fehlerspur. Die letzte Zeile
                    # allein sagt oft nur *was* schiefging, nicht *wo* - und
                    # bei einem Test mit mehreren Teilschritten ist genau
                    # das die Frage.
                    if spur:
                        for z in text.strip().splitlines():
                            print("      " + z)
        if zeige and verworfen:
            for kennung, grund in verworfen[:zeige]:
                print("  MARKE  %-52s %s" % (kennung, grund))
        # **Fehler auf Modulebene zaehlen nicht als Testlauf** - und ohne
        # diese Zeile kam bei `image` eine unmoegliche Zahl heraus: `-3`
        # gute Tests.
        #
        # `setUpModule`, `tearDownModule` und `setUpClass` melden ihre
        # Fehler als `unittest._ErrorHolder`, und `testsRun` zaehlt sie
        # **nicht** mit. Bei `image_test.py` bringt ein Absturz beim
        # Abraeumen (`PySurface is unsendable, but is being dropped on
        # another thread`) drei solche Eintraege mit, und die Rechnung
        # `gut = gelaufen - uebersprungen - falsch - fehlerhaft` zog dann
        # mehr ab, als es Laeufe gab.
        #
        # Eine negative Zahl ist harmlos, solange man sie sieht. Gefaehrlich
        # war sie, weil sie **jede andere Zahl der Zeile mitverschob**: die
        # Datei sah aus wie "52 Fehler von 50 Tests", und der wahre Befund -
        # ein Absturz ausserhalb aller Tests - stand nirgends.
        # `unittest` zaehlt **Eintraege**, nicht Tests: scheitert ein Test
        # und scheitert danach auch sein `tearDown`, stehen zwei Eintraege
        # in der Liste und `testsRun` zaehlt einen. Bei `image_test.py`
        # kamen so 53 Eintraege auf 50 Laeufe, und die Rechnung
        # `gut = gelaufen - uebersprungen - falsch - fehlerhaft` gab **-3**.
        #
        # Eine negative Zahl ist harmlos, solange man sie sieht. Gefaehrlich
        # war sie, weil sie jede andere Zahl der Zeile mitverschob: die
        # Datei sah aus wie "52 Fehler von 50 Tests", und der wahre Befund -
        # ein Test, der beim Abraeumen ein zweites Mal umfaellt - stand
        # nirgends.
        #
        # Fehler **ausserhalb** aller Tests (`setUpModule`,
        # `tearDownModule`) kommen als `unittest._ErrorHolder` und zaehlen
        # gar nicht in `testsRun`; die werden hier dazugerechnet, damit die
        # Summe wieder stimmt.
        kaputt = set()
        aussen = 0
        for liste in (e.failures, e.errors):
            for fall, _ in liste:
                if not isinstance(fall, unittest.TestCase):
                    aussen += 1
                    continue
                # **Untertests zaehlen zum Test, nicht als eigene.** Ein
                # `with self.subTest(fname=...)` erzeugt je Durchlauf einen
                # Eintrag mit eigener Kennung
                # (`testLoadBytesIO (fname='data/alien1.gif')`), waehrend
                # `testsRun` den Test **einmal** zaehlt. Bei `image_test.py`
                # kamen so 53 Eintraege auf 50 Laeufe.
                #
                # `_SubTest` traegt den Ursprung in `test_case`; ohne diesen
                # Umweg zieht die Rechnung mehr ab, als es Laeufe gab. Dass
                # `getattr` genommen wird und nicht ein Typvergleich, liegt
                # daran, dass `unittest.case._SubTest` nicht oeffentlich ist.
                ursprung = getattr(fall, "test_case", None)
                kaputt.add((ursprung or fall).id())
        # Auch die beiden Spalten zaehlen **Tests**, nicht Eintraege -
        # sonst stand in der Zeile "52 Fehler von 50 Tests", was niemand
        # zusammenrechnen kann. Die Eintraege selbst zeigt `--zeige`.
        def welche(liste):
            aus = set()
            for fall, _ in liste:
                if isinstance(fall, unittest.TestCase):
                    aus.add((getattr(fall, "test_case", None) or fall).id())
            return aus

        falsche = welche(e.failures)
        # Ein Test, der **beides** hat (gescheitert und danach beim
        # Abraeumen umgefallen), zaehlt als gescheitert - nicht zweimal.
        fehler = welche(e.errors) - falsche
        n = e.testsRun + len(verworfen) + aussen
        s = len(e.skipped) + len(verworfen)
        f = len(falsche)
        x = len(fehler) + aussen
        ok = n - s - f - x
        gesamt += n
        geglueckt += ok
        gescheitert += f
        fehlerhaft += x
        uebersprungen += s
        bericht.append((name, None, ok, f, x, s))
        if nur_uebersprungen:
            for fall, grund in e.skipped:
                kennung = (getattr(fall, "test_case", None) or fall).id()
                print("UEBERSPRUNGEN %s" % kennung)
            for kennung, _grund in verworfen:
                print("UEBERSPRUNGEN %s" % kennung)

    quelle = "echtes pygame" if echtes_pygame else "rustgame"
    print("pygames eigene Testsuite gegen %s" % quelle)
    print("=" * 62)
    print("%-14s %7s %7s %7s %7s" % ("Datei", "gut", "falsch", "Fehler", "uebersp"))
    for name, grund, ok, f, x, s in bericht:
        if grund:
            print("%-14s  %s" % (name, grund[:60]))
        else:
            print("%-14s %7d %7d %7d %7d" % (name, ok, f, x, s))
    print("-" * 62)
    print("%-14s %7d %7d %7d %7d   (von %d)"
          % ("zusammen", geglueckt, gescheitert, fehlerhaft, uebersprungen, gesamt))
    # **Ein roter Test muss den Lauf rot machen.** Hier stand `return 0`,
    # und in Runde 18 endete ein Lauf mit `2363 / 1 / 0` mit Rueckgabewert
    # 0 - wer nur den Rueckgabewert pruefte (die Abnahme tat es), sah
    # nichts. Die Tabelle war ehrlich, der Waechter nicht.
    return 1 if (gescheitert or fehlerhaft) else 0


if __name__ == "__main__":
    sys.exit(main())
