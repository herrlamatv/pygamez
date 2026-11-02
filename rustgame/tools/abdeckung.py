# -*- coding: utf-8 -*-
"""Wie viele der Namen wurden je **gerufen**? - die Frage hinter der Tabelle.

## Was `API.md` nicht beantwortet

`api_coverage.py` sagt `3471 von 3471 Namen (100 %)`. Das heisst: jeder
Name, den pygame hat, **existiert** hier auch. Es heisst **nicht**, dass
irgendetwas davon je gelaufen ist.

Ein Name, der da ist und nie gerufen wurde, besteht jede Messung dieses
Projekts: die Abdeckungstabelle zaehlt ihn, `dir()` zeigt ihn, die
Basisklassen stimmen, pygames Testsuite kommt an ihm vorbei. Er bricht
erst im Programm eines Menschen. **Das ist die letzte Sorte Luecke, die
hier keine Messung hatte.**

## Wie gemessen wird

`sys.monitoring` (Python 3.12+) liefert mit `events.CALL` fuer **jeden**
Aufruf das gerufene Objekt - auch fuer C- und PyO3-Funktionen, nicht nur
fuer Python. Nachgemessen am Beispiel:

```text
rg.Surface((4,4))   ('type', 'rustgame_py.surface', 'Surface')
s.fill(...)         ('method_descriptor', None, 'Surface.fill')
r.inflate(2, 2)     ('method_descriptor', None, 'Rect.inflate')
```

Bei einer Methode ist `__module__` leer, aber `__qualname__` traegt
`Klasse.methode` - das genuegt zum Zuordnen.

## Der Fehler, den dieses Werkzeug beim ersten Lauf gemacht hat

Der erste Bericht meldete `event.post`, `transform.flip`, `draw.arc` und
250 weitere als **nie gerufen** - und `event_test.py` ruft `post`
siebenundvierzig Mal. Der Grund lag nicht im Lauf, sondern im Namen:
`#[pyfunction]` nimmt den **Rust**-Namen, wenn kein `#[pyo3(name = ..)]`
dabeisteht, also hiess die Funktion `ereignis_post` und passte auf keinen
erwarteten Namen.

**Das war ein echter Fund, nur nicht der, nach dem gesucht wurde:** 258
von 390 Funktionen trugen so ihren Rust-Namen in `__name__` - sichtbar in
jedem Rueckverfolgungsauszug und in jeder Fehlermeldung ueber ein
Schluesselwort. Seitdem haelt die Probe `Funktionsnamen` das nach, und
dieses Werkzeug zaehlt richtig.

**Ein Messgeraet, das etwas Unerwartetes meldet, hat zwei Verdaechtige** -
und einer davon ist es selbst.

Gefahren werden beide Quellen, aus denen hier Aufrufe kommen:

1. `vergleich_module.proben()` - die 810 Einzelproben,
2. `pygame_tests.py` - pygames eigene Testsuite, 2364 Tests.

Was danach **nicht** in der Menge steht, hat kein Aufruf dieses Projekts
je angefasst.

**Jede Quelle in ihrem eigenen Prozess, und die Testsuite Datei fuer
Datei** - warum das sein muss und was es gekostet hat, steht bei
[`teil_messen`]. Kurz: die Zahl haing sonst von der Reihenfolge ab, und
eine Messung, die von der Reihenfolge abhaengt, ist keine. Der Lauf
dauert deshalb Minuten statt Sekunden.

## Was die Zahl nicht ist

Sie ist keine Zeilen-Abdeckung. Eine Funktion, die **einmal** gerufen
wurde, zaehlt hier als abgedeckt - auch wenn nur einer ihrer Zweige lief.
Und sie zaehlt nur **Rufbares**: Konstanten und Felder werden gelesen,
nicht gerufen, und `sys.monitoring` kennt kein Ereignis fuer einen
Attributzugriff.

Die Zahl beantwortet also genau eine Frage, und die hatte vorher keine
Antwort: **welche Funktion hat hier noch niemand angefasst?**

## Gebrauch

```bash
.venv/Scripts/python.exe rustgame/tools/abdeckung.py              # alles
.venv/Scripts/python.exe rustgame/tools/abdeckung.py --proben      # nur die Proben
.venv/Scripts/python.exe rustgame/tools/abdeckung.py --teil suite:math
```

Die letzte Form ist der Baustein: **ein** Teil, in diesem Prozess, die
gerufenen Namen als Zeilen auf die Ausgabe. Der volle Lauf setzt sich
daraus zusammen.
"""

import importlib
import os
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
RUSTGAME = os.path.dirname(HIER)
HUELLE = os.path.join(RUSTGAME, "crates", "rustgame-py")

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("RUSTGAME_VIDEODRIVER", "dummy")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
os.environ.setdefault("RUSTGAME_AUDIODRIVER", "dummy")
os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")

sys.path.insert(0, HUELLE)
sys.path.insert(0, os.path.join(HUELLE, "tools"))
sys.path.insert(0, HIER)

#: Die Nummer, unter der sich dieses Werkzeug bei `sys.monitoring` anmeldet.
#:
#: 0 bis 2 sind fuer Debugger und Profiler vorgesehen, 3 bis 5 fuer
#: alles andere - diese hier ist frei und stoert keinen Debugger.
WERKZEUG = 3


def erwartet(paket):
    """Jeden **rufbaren** Namen des Pakets, als `Modul.name` bzw.
    `Klasse.name`.

    Die Liste wird **gelesen**: die Module kommen aus
    `api_coverage.alle_untermodule`, die Klassen aus
    `api_coverage.alle_klassen`. Beide lesen pygame ab, nicht eine
    gepflegte Aufzaehlung - siehe dort, warum das der neunte und elfte
    blinde Fleck war.
    """
    import api_coverage as A

    aus = {}
    for n in [""] + A.alle_untermodule():
        voll = paket.__name__ + ("." + n if n else "")
        try:
            m = importlib.import_module(voll)
        except Exception:                            # noqa: BLE001
            continue
        for a in dir(m):
            if a.startswith("_"):
                continue
            w = getattr(m, a, None)
            if callable(w) and not isinstance(w, type):
                # **Die eigene Heimat, nicht der Weg dorthin.**
                #
                # `sysfont.get_fonts`, `font.get_fonts`,
                # `freetype.get_fonts` und `ftfont.get_fonts` sind
                # **dieselbe** Funktion; `_freetype.init` und
                # `freetype.init` ebenso. Der erste Entwurf nahm den Weg
                # als Schluessel und zaehlte damit jeden Alias einzeln -
                # `rustgame_py.sysfont` stand mit acht nie gerufenen
                # Namen im Bericht, obwohl drei davon unter einem anderen
                # Weg gerufen worden waren. Dieselbe Falle wie bei den
                # geerbten Methoden, die hier schon ueber `__qualname__`
                # geloest ist.
                heim = getattr(w, "__module__", None) or voll
                name = getattr(w, "__name__", None) or a
                if not heim.startswith(paket.__name__):
                    heim = voll
                aus["%s.%s" % (heim, name)] = ("Modul", heim, name)
    for weg in A.alle_klassen():
        obj = paket
        for stueck in weg.split("."):
            obj = getattr(obj, stueck, None)
            if obj is None:
                break
        if not isinstance(obj, type):
            continue
        aus["%s()" % weg] = ("Klasse", weg, "")
        for a in dir(obj):
            if a.startswith("_"):
                continue
            w = getattr(obj, a, None)
            if not callable(w):
                continue
            # **Der Schluessel ist das `__qualname__` der Methode selbst**,
            # nicht `Klasse.name`.
            #
            # `Group.add` ist dieselbe Methode wie `AbstractGroup.add` -
            # geerbt, nicht neu. `sys.monitoring` meldet beim Aufruf den
            # Namen, unter dem sie **definiert** ist, also
            # `AbstractGroup.add`. Der erste Entwurf hat
            # `Klasse.name` erwartet und damit jede geerbte Methode in
            # **jeder** Unterklasse als nie gerufen gezaehlt: dreizehn Mal
            # fuer `Group`, dreizehn fuer `GroupSingle`, dreizehn fuer
            # `OrderedUpdates` - alles dieselben Methoden.
            schluessel = getattr(w, "__qualname__", None) or (
                "%s.%s" % (obj.__name__, a))
            aus[schluessel] = ("Methode", weg, a)
    return aus


def kennung(rufbar, paket):
    """Der Name, unter dem ein gerufenes Objekt in [`erwartet`] steht.

    Drei Gestalten, alle gemessen:

    * eine **Funktion** eines Moduls hat `__module__` und `__name__`,
    * eine **Methode** hat kein `__module__`, aber `__qualname__` in der
      Form `Klasse.methode`,
    * eine **Klasse** gerufen (`Surface((4,4))`) kommt als `type` mit
      `__module__` und `__name__`.
    """
    mod = getattr(rufbar, "__module__", None) or ""
    name = getattr(rufbar, "__qualname__", None) or getattr(
        rufbar, "__name__", None) or ""

    def unser(m):
        """Gehoert dieses Modul zu uns?

        ## Der vierte Fehler dieses Werkzeugs an sich selbst - und die
        ## Erklaerung fuer den "Ereignisverlust"

        Hier stand nur `m.startswith(paket)`, also `rustgame_py`. Im
        Suitenlauf heissen die Module aber **`pygame.*`**:
        `pygame_tests.umbiegen()` haengt jedes Untermodul unter
        `sys.modules["pygame.X"]` ein, damit pygames eigene Tests ihre
        Importe finden - und `Rect.__module__` liest sich dort
        `pygame.rect`. Also passte **kein einziger** Aufruf aus dem
        Testlauf, und der Bericht meldete 32 nie gerufene
        `Vector3`-Methoden, obwohl `math_test.py` sie 75 Mal ruft.

        Daraus wurde erst "ein Rueckruf, der wirft" (widerlegt: ein
        Zaehler blieb bei null), dann "die Reihenfolge der zwei Quellen"
        (halb richtig - getrennte Prozesse halfen, aus einem anderen
        Grund), dann "Datei fuer Datei" (half ebenfalls). **Drei
        Erklaerungen, und keine war die Ursache.** Gefunden hat sie
        schliesslich die einfachste Frage, die vorher keiner gestellt
        hatte: *laeuft die Ueberwachung ueberhaupt noch?* Sie lief, und
        der Rueckruf hatte 264.049 Aufrufe gesehen - die Ereignisse waren
        nie verloren, sie wurden nur alle weggeworfen.

        *Wer dreimal eine Ursache vermutet, ohne sie zu messen, hat
        dreimal geraten.*
        """
        return m.startswith(paket) or m.startswith("pygame")

    def heim_name(m):
        """`pygame.math` wird zu `rustgame_py.math`.

        Den Namen nur **anzuerkennen** genuegt nicht: der Schluessel in
        [`erwartet`] lautet `rustgame_py.math.lerp`, und der Suitenlauf
        meldet `pygame.math.lerp`. Ohne diese Zeile stimmten die Module
        und die Schluessel trotzdem nicht.
        """
        if m == "pygame" or m.startswith("pygame."):
            return paket + m[len("pygame"):]
        return m

    if isinstance(rufbar, type):
        if unser(mod):
            return "%s()" % name
        return None
    # **Ein Punkt im `__qualname__` zaehlt zuerst.**
    #
    # [`erwartet`] schluesselt jede Methode mit ihrem `__qualname__`, auch
    # eine, die in Python geschrieben ist. Eine solche hat aber **auch**
    # ein `__module__`, und der Zweig darunter griff zuerst: aus
    # `_font_klasse.<locals>.Font.get_bold` wurde
    # `rustgame_py.ftfont.get_bold`, und das steht in keiner
    # Erwartungsliste. `ftfont.Font` stand darum mit sechzehn nie
    # gerufenen Namen im Bericht, **obwohl eine Probe sie alle ruft** -
    # der siebte Fehler dieses Werkzeugs an sich selbst.
    #
    # Fuer eine Modulfunktion ist `__qualname__` gleich `__name__` und
    # enthaelt keinen Punkt; die Reihenfolge schadet dort also nicht.
    if "." in name and unser(mod):
        return name
    if unser(mod):
        return "%s.%s" % (heim_name(mod), name)
    # Methode: der Weg steckt im `__qualname__`, der Besitzer in
    # `__objclass__` (bei einem Beschreiber) oder in `__self__` (bei einer
    # gebundenen Methode).
    #
    # ## Der dritte Fehler dieses Werkzeugs an sich selbst
    #
    # Hier stand `if heim.startswith(paket) or not heim:` - und das
    # `or not heim` hat **jede fremde Funktion mit einem Punkt im
    # `__qualname__`** angenommen. Im Suitenlauf waren das die Testfaelle
    # selbst: `Vector3TypeTest.testAdd`, `Vector2TypeTest.setUp`, 551
    # Stueck je Datei. Die Zahl unten wurde davon nicht falsch (was nicht
    # in [`erwartet`] steht, zaehlt nicht mit), aber die Ausgabe von
    # `--teil suite:math` sah aus wie eine Messung und war keine - und
    # genau daran liess sich der Ereignisverlust nicht untersuchen.
    #
    # `__objclass__` ist die Antwort, die ein Beschreiber selbst gibt:
    # `rustgame_py.Vector3.distance_to.__objclass__` ist `Vector3`, und
    # dessen `__module__` ist `rustgame_py.math`. Ohne Besitzer gibt es
    # keine Zuordnung - und dann gehoert der Aufruf nicht hierher.
    if "." in name:
        selbst = getattr(rufbar, "__self__", None)
        besitzer = getattr(rufbar, "__objclass__", None)
        if besitzer is None and selbst is not None:
            besitzer = selbst if isinstance(selbst, type) else type(selbst)
        heim = getattr(besitzer, "__module__", "") if besitzer else ""
        if unser(heim):
            return name
    return None


def messen(paket, teil):
    """Fahrt **einen** Teil und gibt die Menge der gerufenen Namen.

    `teil` ist `"proben"`, `"suite"` oder `"suite:<datei>"` - siehe
    [`teil_messen`], warum es drei und nicht eins ist.
    """
    gerufen = set()
    m = sys.monitoring
    m.use_tool_id(WERKZEUG, "abdeckung")

    # Ein Fehler im Rueckruf schaltet die Ueberwachung ab - still. Das war
    # die **erste Erklaerung** fuer die verlorenen Ereignisse (siehe
    # [`teil_messen`]), und sie war falsch: der Zaehler blieb bei null und
    # die Zahl aenderte sich nicht. Der `try` bleibt trotzdem stehen, denn
    # die Gefahr ist echt - nur war sie hier nicht die Ursache. Ein
    # Vermerk, der eine Ursache **behauptet**, die nie gemessen wurde, ist
    # schlimmer als keiner.
    schlucker = [0]
    #: Wie viele Aufrufe **ueberhaupt** gemeldet wurden.
    #:
    #: Ohne diese Zahl war drei Runden lang nicht zu unterscheiden, ob die
    #: Ueberwachung nichts meldet oder ob [`kennung`] alles wegwirft - und
    #: es war das Zweite. Ein Messgeraet, das nur sein Ergebnis zeigt und
    #: nicht seinen Durchsatz, laesst genau diese Frage offen.
    gesamt = [0]

    # **Der Name wird jetzt gelesen, nicht im Rueckruf.**
    #
    # `pygame_tests.umbiegen()` legt die Huelle als `sys.modules["pygame"]`
    # ab und **benennt sie um**: `rustgame_py.__name__` lautet danach
    # `pygame`, `rustgame_py.math.__name__` lautet `pygame.math` (fuenf
    # Tests in `font_test.py` fragen genau diesen Namen ab). Im Rueckruf
    # gelesen ergab `paket.__name__` damit `pygame`, und `kennung`
    # verglich `pygame` gegen `rustgame_py.math` - also **nichts**
    # zugeordnet: 196.223 gemeldete Aufrufe, 34 Namen.
    eigener = paket.__name__

    def bei_aufruf(code, offset, rufbar, arg0):
        gesamt[0] += 1
        try:
            k = kennung(rufbar, eigener)
            if k is not None:
                gerufen.add(k)
        except BaseException:                        # noqa: BLE001
            schlucker[0] += 1

    m.register_callback(WERKZEUG, m.events.CALL, bei_aufruf)
    m.set_events(WERKZEUG, m.events.CALL)
    try:
        if teil == "proben":
            import vergleich_module

            try:
                vergleich_module.proben(paket)
            except Exception as e:                   # noqa: BLE001
                sys.stderr.write("Proben brachen ab: %s: %s\n"
                                 % (type(e).__name__, e))
        if teil.startswith("suite"):
            # `suite:math` fahrt **eine** Datei. Siehe [`teil_messen`],
            # warum es nicht eine einzige Messung ueber alle ist.
            _, _, nur = teil.partition(":")
            if nur:
                sys.argv = [sys.argv[0], nur]
            import pygame_tests

            try:
                pygame_tests.main()
            except SystemExit:
                pass
            except Exception as e:                   # noqa: BLE001
                print("   Testsuite brach ab: %s: %s" % (type(e).__name__, e))
    finally:
        m.set_events(WERKZEUG, 0)
        m.free_tool_id(WERKZEUG)
    sys.stderr.write("   [%s] %d Aufrufe gemeldet, %d Namen zugeordnet\n"
                     % (teil, gesamt[0], len(gerufen)))
    if schlucker[0]:
        print("   **%d Aufrufe konnten nicht zugeordnet werden** - die Zahl "
              "unten ist zu niedrig." % schlucker[0])
    return gerufen


def teil_messen(teil):
    """Ein Teil in **diesem** Prozess, Ergebnis als Zeilen auf die Ausgabe.

    ## Warum getrennte Prozesse

    Der erste Entwurf fuhr beides hintereinander in **einem** Prozess.
    Dass das falsche Zahlen gab, hatte **zwei** Ursachen, und nur eine
    davon ist hier behoben; die andere steht bei [`kennung`] (der
    Paketname wird waehrend des Suitenlaufs umbenannt, und das Werkzeug
    hat deswegen jeden Aufruf weggeworfen).

    Der Rest ist ein **echter** Verlust, gemessen mit dem Durchsatzzaehler
    in [`messen`]:

    ```text
    [suite]      158461 Aufrufe gemeldet,  28 Namen   (alle 50 Dateien)
    [suite:math] 196223 Aufrufe gemeldet, 121 Namen   (eine Datei)
    ```

    **Fuenfzig Dateien melden weniger Aufrufe als eine.** Innerhalb eines
    Suitenlaufs geht also wirklich etwas verloren - ein Unterprozess ist
    es nicht, `pygame_tests.py` fahrt alles ueber `unittest` im eigenen
    Prozess, und die Ueberwachung ist nachweislich bis zum Ende an
    (`get_events` liefert weiter 16).

    Woran, ist **offen**. Was hilft, ist dasselbe Mittel wie bei
    `uebersprungen.py`: getrennte Prozesse, unabhaengige Mengen, und die
    Vereinigung zaehlt - hier einer je Testdatei. Das kostet Minuten statt
    Sekunden und ist der Preis dafuer, dass die Zahl etwas bedeutet.

    *Eine Messung, die von der Reihenfolge abhaengt, ist keine.*
    """
    import rustgame_py

    for name in sorted(messen(rustgame_py, teil)):
        print("GERUFEN %s" % name)
    return 0


def suite_dateien():
    """Die Namen der Testdateien - **gelesen**, nicht aufgezaehlt."""
    pass  # die Pfade stehen schon oben
    import pygame_tests

    ordner = pygame_tests.pygame_tests_ordner()
    return pygame_tests.alle_dateien(ordner)


def teil_holen(teil):
    """Fahrt diese Datei als eigenen Prozess und liest die Namen zurueck."""
    import subprocess

    umgebung = dict(os.environ)
    umgebung["PYTHONIOENCODING"] = "utf-8"
    fertig = subprocess.run(
        [sys.executable, os.path.abspath(__file__), "--teil", teil],
        cwd=RUSTGAME, env=umgebung, stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL, text=True, encoding="utf-8",
        errors="replace",
    )
    return {z[len("GERUFEN "):].strip()
            for z in fertig.stdout.splitlines() if z.startswith("GERUFEN ")}


def main():
    if "--teil" in sys.argv:
        return teil_messen(sys.argv[sys.argv.index("--teil") + 1])
    nur_proben = "--proben" in sys.argv
    import rustgame_py

    soll = erwartet(rustgame_py)
    print("%d rufbare Namen in der Schnittstelle" % len(soll))
    print("Messe%s ..." % ("" if nur_proben else " (das dauert einige Minuten)"))
    ist = teil_holen("proben")
    print("   Proben: %d Namen" % len(ist))
    if not nur_proben:
        for datei in suite_dateien():
            s = teil_holen("suite:" + datei)
            print("   %-22s %4d Namen (zusammen %d)"
                  % (datei, len(s), len(ist | s)))
            ist = ist | s

    nie = sorted(k for k in soll if k not in ist)
    nach_ort = {}
    for k in nie:
        art, ort, _ = soll[k]
        nach_ort.setdefault(ort, []).append(k)

    print()
    print("gerufen: %d von %d (%.0f %%)"
          % (len(soll) - len(nie), len(soll),
             100.0 * (len(soll) - len(nie)) / len(soll)))
    if not nie:
        print("Jeder rufbare Name wurde mindestens einmal gerufen.")
        return 0
    print()
    print("**Nie gerufen** - %d Namen, nach Ort:" % len(nie))
    for ort in sorted(nach_ort, key=lambda o: (-len(nach_ort[o]), o)):
        namen = [k.split(".")[-1] for k in nach_ort[ort]]
        print("  %-28s %3d  %s" % (ort or "(haupt)", len(namen),
                                   ", ".join(sorted(namen))[:90]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
