# -*- coding: utf-8 -*-
"""Was sagt ein Objekt, das `__new__` hatte und **kein `__init__`**?

## Warum ein eigenes Werkzeug und keine Probe

Weil **pygame dabei abstuerzt.** Der erste Lauf fragte alle Klassen in
einem Prozess und endete bei `(haupt).Font .italic` mit

    Fatal Python error: pygame_parachute: (pygame parachute) Segmentation
    Fault

- 341 der 447 Fragen waren danach nicht mehr gestellt. Also laeuft jede
Klasse in einem eigenen Prozess, wie in `zustand.py`, und ein Absturz
wird zur **Antwort** statt zum Ende des Laufs.

## Was gemessen wird

Fuer jede Klasse, die pygame oeffentlich fuehrt:
`o = Klasse.__new__(Klasse)`, dann `repr`, `str`, `bool`, `len`, `hash`,
`iter`, `o == o`, `o.__dict__` und **jede** Eigenschaft, die keine
Methode ist. Nur Lesen - eine Methode braeuchte Argumente, und manche
taeten etwas.

Die Namen kommen aus **pygames** Klasse, nicht aus der eigenen: sonst
fragt jede Engine ihre eigene Liste ab (derselbe Geraetefehler wie in
Runde 11 und 17, siehe `mycache/REGELN.md`).

## Was die erste Messung fand (Runde 17)

**374 von 507** Antworten gingen auseinander. Behoben wurde die eine
Gruppe, die eine Sache ist und keine Bauart:

| Klasse | Fund |
|---|---|
| `freetype.Font` | 30 Stellen: 19 Eigenschaften antworten bei pygame aus dem leeren Zettel (`antialiased` `True`, `strength` `1/36`, `fgcolor` `Color(0,0,0,255)`), 6 geben `RuntimeError: _freetype.Font instance is not initialized` - und hier gaben **alle 25** einen `RuntimeError` mit dem Satz von `pygame.font` |
| `freetype.Font` | `repr` ist `<uninitialized Font object at 0x...>`, `name`/`style_name` geben `AttributeError: <uninitialized Font object>`, `path` `AttributeError: path unavailable` |
| `key.ScancodeWrapper` | hat **kein** `__dict__` (`__slots__ = ()`), und sein `AttributeError` nennt den ganzen Weg der Klasse |

## Was eine benannte Grenze ist

Drei Gruppen, alle mit Messung - sie stehen in `GRENZEN` unten und
machen den Lauf **nicht** rot:

1. **pygame stuerzt ab** bei `font.Font`, `mask.Mask` und
   `_render.Texture`. Ein Absturz ist nichts, was man nachbaut.
2. **Die Figuren und der Zeiger stehen hier in Rust, bei pygame in
   Python.** Ein halbfertiges Python-Objekt hat ein leeres `__dict__`
   und gibt auf **jede** Frage `AttributeError`; ein Rust-Objekt
   antwortet aus seinen Feldern. Das sind 95 der Stellen - zu treffen
   nur, indem `sprite` und `cursors` wieder in Python wandern.
3. **`Window` liest bei pygame freigegebenen Speicher**: `size` und
   `position` geben Zufallszahlen (gemessen `(1485696640, -627844288)`,
   bei jedem Lauf andere). Das ist nicht nachzubauen, und `repr` sagt
   dort `<Window(Destroyed)>` - was hier nach `destroy()` ebenfalls
   steht.

## Fahren

```bash
.venv/Scripts/python.exe rustgame/tools/halbfertig.py
```

Endet mit 1, sobald eine Antwort auseinandergeht, die keiner der
benannten Grenzen angehoert. `--genau <Klasse>` zeigt eine Klasse ganz.
"""
import importlib
import os
import re
import subprocess
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))
HUELLE = os.path.join(WURZEL, "rustgame", "crates", "rustgame-py")

#: Die Unterschiede, die benannt sind - mit dem Grund.
#:
#: Der Schluessel ist der Weg der Klasse, wie er in der Ausgabe steht
#: (alles, was mit ihm anfaengt, gilt als benannt) - oder
#: `Klasse|Frage`, dann gilt die Grenze **nur** fuer diese eine Frage
#: und der Rest der Klasse wird weiter streng verglichen.
GRENZEN = {
    "(haupt).Font": "pygame stuerzt ab (Segmentation Fault im Parachute)",
    "font.Font": "pygame stuerzt ab (Segmentation Fault im Parachute)",
    "font.FontType": "pygame stuerzt ab (Segmentation Fault im Parachute)",
    "(haupt).Mask": "pygame stuerzt ab (Segmentation Fault im Parachute)",
    "mask.Mask": "pygame stuerzt ab (Segmentation Fault im Parachute)",
    "mask.MaskType": "pygame stuerzt ab (Segmentation Fault im Parachute)",
    "_render.Texture": "pygame stuerzt ab (Segmentation Fault im Parachute)",
    "_render.Renderer": "Python-Klasse bei pygame, Rust-Klasse hier",
    "_render.Image": "Python-Klasse bei pygame, Rust-Klasse hier",
    "sprite.": "Python-Klasse bei pygame, Rust-Klasse hier",
    "(haupt).Cursor": "Python-Klasse bei pygame, Rust-Klasse hier",
    "cursors.Cursor": "Python-Klasse bei pygame, Rust-Klasse hier",
    "(haupt).Window": "pygame liest freigegebenen Speicher",
    "window.Window": "pygame liest freigegebenen Speicher",
    "(haupt).PixelArray": "zwei Argumentleser, zwei Saetze",
    "pixelarray.PixelArray": "zwei Argumentleser, zwei Saetze",
    "freetype.Font": "der Pfad der Vorgabeschrift liegt woanders",
    "ftfont.Font": "der Pfad der Vorgabeschrift liegt woanders",
    # Nur diese eine Frage, nicht die ganze Klasse: `Line` ist bei
    # pygame hashbar und sein eigener `Circle` nicht - die benannte
    # Grenze `Line ohne __eq__`. Alles andere an `Line` wird verglichen.
    "geometry.Line|hash": "pygame macht Line hashbar, Circle nicht",
    "(haupt).Event": "erfundener Leerzustand (Typ 0 ohne Woerterbuch)",
    "event.Event": "erfundener Leerzustand (Typ 0 ohne Woerterbuch)",
    "event.EventType": "erfundener Leerzustand (Typ 0 ohne Woerterbuch)",
    "(haupt).Channel": "erfundener Leerzustand (Spur 0 ohne Mischer)",
    "mixer.Channel": "erfundener Leerzustand (Spur 0 ohne Mischer)",
    "mixer.ChannelType": "erfundener Leerzustand (Spur 0 ohne Mischer)",
}

#: Namen, die ein halbfertiges Objekt nicht gefragt werden - sie wuerden
#: etwas tun, wenn die Klasse sie doch beantwortet.
AUS = {"quit", "init", "stop", "pause", "unpause", "close", "destroy"}


def klassenliste():
    """`[(modul, klasse)]` - **aus pygames Paket** abgelesen."""
    import pygame

    ordner = os.path.dirname(os.path.abspath(pygame.__file__))
    namen = ["(haupt)"] + [d[:-4] for d in sorted(os.listdir(ordner))
                           if d.endswith(".pyi") and d != "__init__.pyi"]
    aus, gesehen = [], set()
    for mname in namen:
        try:
            m = (pygame if mname == "(haupt)"
                 else importlib.import_module("pygame.%s" % mname))
        except Exception:                            # noqa: BLE001
            continue
        for a in sorted(dir(m)):
            if a.startswith("_"):
                continue
            try:
                kl = getattr(m, a)
            except Exception:                        # noqa: BLE001
                continue
            if not isinstance(kl, type) or issubclass(kl, BaseException):
                continue
            if kl in gesehen:
                continue
            gesehen.add(kl)
            aus.append((mname, a))
    return aus


def arbeiter(engine, mname, a):
    """Eine Klasse, ein Prozess - die Antworten auf die Standardausgabe."""
    if engine == "rustgame":
        sys.path.insert(0, HUELLE)
        import rustgame_py as pg
    else:
        import pygame as pg
    import pygame as echt

    echtm = (echt if mname == "(haupt)"
             else importlib.import_module("pygame.%s" % mname))
    felder = [b for b in sorted(dir(getattr(echtm, a)))
              if not b.startswith("_") and b not in AUS
              and not callable(getattr(getattr(echtm, a), b, None))]

    pg.init()
    try:
        pg.mixer.init()
    except Exception:                                # noqa: BLE001
        pass

    def gestalt(t):
        t = t.replace(pg.__name__, "@").replace("pygame", "@")
        return re.sub(r"0x[0-9a-fA-F]+", "0xADR", t)

    def zeig(marke, ruf):
        try:
            w = repr(ruf())
        except BaseException as e:                   # noqa: BLE001
            w = "%s: %s" % (type(e).__name__, e)
        print("%s\t%s" % (marke, gestalt(w)[:120]), flush=True)

    m = (pg if mname == "(haupt)"
         else importlib.import_module("%s.%s" % (pg.__name__, mname)))
    kl = getattr(m, a)
    try:
        o = kl.__new__(kl)
    except BaseException as e:                       # noqa: BLE001
        print("(bau)\t%s: %s" % (type(e).__name__, gestalt(str(e))[:90]),
              flush=True)
        return 0
    zeig("repr", lambda: repr(o))
    zeig("str", lambda: str(o))
    zeig("bool", lambda: bool(o))
    zeig("len", lambda: len(o))
    zeig("hash", lambda: type(hash(o)).__name__)
    zeig("iter", lambda: type(iter(o)).__name__)
    zeig("== sich", lambda: o == o)
    zeig("dict", lambda: sorted(o.__dict__))
    for b in felder:
        zeig(".%s" % b, lambda b=b: getattr(o, b))
    return 0


def fahren(engine, mname, a):
    """Ruft den Arbeiter und gibt `{frage: antwort}` zurueck."""
    p = subprocess.run(
        [sys.executable, "-W", "ignore", os.path.abspath(__file__),
         "--arbeiter", engine, mname, a],
        capture_output=True, text=True, encoding="utf-8", errors="replace")
    aus = {}
    for z in (p.stdout or "").replace("\r\n", "\n").split("\n"):
        if "\t" not in z:
            continue
        frage, _, antwort = z.partition("\t")
        aus[frage] = antwort
    if p.returncode != 0:
        aus["(absturz)"] = "Rueckgabe %d" % p.returncode
    return aus


def benannt(weg, frage=None):
    """Gehoert das zu einer benannten Grenze?

    `frage` leer heisst "die ganze Klasse"; mit `frage` zaehlen auch die
    Eintraege der Form `Klasse|Frage`.
    """
    for k, grund in GRENZEN.items():
        kl, _, nurfrage = k.partition("|")
        if not weg.startswith(kl):
            continue
        if not nurfrage:
            return grund
        if frage is not None and frage == nurfrage:
            return grund
    return None


def main():
    if "--arbeiter" in sys.argv:
        i = sys.argv.index("--arbeiter")
        return arbeiter(sys.argv[i + 1], sys.argv[i + 2], sys.argv[i + 3])
    nur = None
    if "--genau" in sys.argv:
        nur = sys.argv[sys.argv.index("--genau") + 1]

    print("Halbfertige Objekte: pygame-ce gegen die rustgame-Huelle")
    print("=" * 64)
    liste = klassenliste()
    offen = gefragt = gegrenzt = 0
    gruende = {}
    for mname, a in liste:
        weg = "%s.%s" % (mname, a)
        if nur and nur not in weg:
            continue
        a_pg = fahren("pygame", mname, a)
        a_rg = fahren("rustgame", mname, a)
        gem = sorted(set(a_pg) & set(a_rg))
        anders = [k for k in gem if a_pg[k] != a_rg[k]]
        allein = sorted(set(a_pg) ^ set(a_rg))
        gefragt += len(gem)
        grund = benannt(weg)
        if grund and (anders or allein):
            gegrenzt += len(anders) + len(allein)
            gruende.setdefault(grund, []).append(weg)
            if nur:
                print("%-30s [bekannt] %s" % (weg, grund))
            continue
        # Einzelne Fragen koennen benannt sein, ohne dass die Klasse es
        # ist - dann faellt nur die Frage heraus.
        for k in list(anders):
            g = benannt(weg, k)
            if g:
                anders.remove(k)
                gegrenzt += 1
                gruende.setdefault(g, []).append("%s %s" % (weg, k))
        if not anders and not allein:
            if nur:
                print("%-30s gleich (%d Fragen)" % (weg, len(gem)))
            continue
        offen += len(anders) + len(allein)
        print("%s" % weg)
        for k in allein:
            print("   nur bei %-8s %-14s %s"
                  % ("pygame" if k in a_pg else "rustgame", k,
                     (a_pg.get(k) or a_rg.get(k))[:60]))
        for k in anders:
            print("   %-16s pygame   %s" % (k, a_pg[k][:70]))
            print("   %-16s rustgame %s" % ("", a_rg[k][:70]))
    print("-" * 64)
    # **Eine Messung ohne Eintraege ist immer gleich.**
    if not nur and gefragt < 400:
        print("** nur %d gemeinsame Fragen - das ist zu wenig" % gefragt)
        return 1
    for grund, wege in sorted(gruende.items()):
        print("[bekannt] %-46s %d Klassen" % (grund, len(wege)))
    if offen == 0:
        print("%d Fragen an %d Klassen, Antwort fuer Antwort dieselbe "
              "(%d benannt)." % (gefragt, len(liste), gegrenzt))
        return 0
    print("%d Antworten gehen auseinander (%d benannt, %d Fragen)."
          % (offen, gegrenzt, gefragt))
    return 1


if __name__ == "__main__":
    sys.exit(main())
