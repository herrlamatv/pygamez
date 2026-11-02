"""Jede Schrift im Windows-Schriftordner, beide Schriftmodule, beide Engines.

Runde 20. Bis dahin hatte jede Probe ein Dutzend ausgewaehlte Schriften,
und die Funde dieser Runde lagen alle daneben: Calibris `kern`-Tabelle
(Laengenfeld als 16 Bit), Courier New fett (`FLIPRGON` macht beim Hinting
Kontrollpunkte zu Kurvenpunkten), Calibris Strikes (sbit-Vorschub,
senkrechter Satz), `cour.ttf` bei kleinen Groessen (abgeschaltetes
Hinting), Symbolschriften (Zeichentabelle 3,0), `monbaiti.ttf` (ein
scheiterndes CV-Programm).

Ein eigenes Werkzeug und keine Probe, weil **pygame dabei abstuerzt**:
`pygame.freetype` faellt nach einem Ladefehler in einen Speicherzugriffs-
fehler (bei `monbaiti.ttf` und einigen `.fon`), und ein Absturz nimmt in
`vergleich_module.py` den ganzen Lauf mit. Hier laeuft jeder Block von
Schriften in einem eigenen Prozess; stuerzt einer ab, wird die Schrift
vermerkt und der Rest neu gestartet.

    python tools/alle_schriften.py            # beide Module
    python tools/alle_schriften.py ft         # nur pygame.freetype
    python tools/alle_schriften.py font       # nur pygame.font

Gibt je Modul die Zahl der gemeinsamen Faelle, der verschiedenen und die
Schriften dazu aus. Schriften, die nur eine Seite laedt (`.fon`: rustgame
kennt das Format nicht), stehen getrennt.
"""
import hashlib
import importlib
import os
import subprocess
import sys
from collections import Counter

HIER = os.path.dirname(os.path.abspath(__file__))
HUELLE = os.path.join(HIER, "..", "crates", "rustgame-py")
W = os.path.join(os.environ.get("WINDIR", r"C:\Windows"), "Fonts")
TEXT = "Hamburg 0123 \u00c4\u00d6 fi"


def alle():
    return sorted(n for n in os.listdir(W) if n.lower().endswith((".ttf", ".otf", ".ttc", ".fon")))


def arbeiter(eng, modus, dateien):
    """Laeuft im Kindprozess: eine Zeile je Fall."""
    os.environ["SDL_VIDEODRIVER"] = "dummy"
    os.environ["PYGAME_HIDE_SUPPORT_PROMPT"] = "1"
    if eng == "pg":
        import pygame as pg
    else:
        sys.path.insert(0, HUELLE)
        import rustgame_py as pg
    pg.init()

    def bild(s):
        return hashlib.sha256(pg.image.tobytes(s, "RGBA")).hexdigest()[:12]

    def versuch(fn):
        try:
            return fn()
        except Exception as e:  # noqa: BLE001
            return "%s: %s" % (type(e).__name__, str(e)[:60])

    if modus == "ft":
        ft = importlib.import_module(pg.__name__ + ".freetype")
        ft.init()
        for n in dateien:
            f = versuch(lambda: ft.Font(os.path.join(W, n)))
            if isinstance(f, str):
                print(n, "laden", f, flush=True)
                continue
            for g in (8, 11, 16, 23.5):
                for aa in (True, False):
                    f.antialiased = aa
                    print(n, g, aa, versuch(lambda: (lambda s, r: (tuple(r), s.get_size(), bild(s)))(
                        *f.render(TEXT, (255, 255, 255), size=g))), flush=True)
                f.antialiased = True
                f.kerning = True
                print(n, g, "kern", versuch(lambda: tuple(f.get_rect(TEXT, size=g))), flush=True)
                f.kerning = False
    else:
        for n in dateien:
            for g in (9, 13, 20):
                f = versuch(lambda: pg.font.Font(os.path.join(W, n), g))
                if isinstance(f, str):
                    print(n, g, "laden", f, flush=True)
                    continue
                for aa in (True, False):
                    print(n, g, aa, versuch(lambda: (lambda s: (s.get_size(), bild(s)))(
                        f.render(TEXT, aa, (255, 255, 255)))), flush=True)
                print(n, g, "size", versuch(lambda: f.size(TEXT)), versuch(lambda: f.metrics("Ag")),
                      flush=True)


def fahren(eng, modus):
    """Alle Schriften in Bloecken; ein Absturz wird vermerkt."""
    env = dict(os.environ, PYTHONIOENCODING="utf-8")
    zeilen = []
    abstuerze = []
    rest = alle()
    while rest:
        block = rest[:40]
        r = subprocess.run([sys.executable, "-W", "ignore", __file__, "--arbeiter", eng, modus,
                            "|".join(block)], capture_output=True, text=True, encoding="utf-8",
                           env=env)
        neu = r.stdout.splitlines()
        zeilen += neu
        if r.returncode == 0:
            rest = rest[40:]
            continue
        # Die Schrift, bei der die Ausgabe aufhoert, gilt als Taeterin - sie
        # kann schon fertig sein; dann stuerzt die naechste in ihrem eigenen
        # Prozess nicht mehr ab (pygames Absturz ist ein Folgefehler).
        letzte = neu[-1].split(" ")[0] if neu else None
        i = block.index(letzte) if letzte in block else 0
        abstuerze.append(block[i])
        rest = block[i + 1:] + rest[40:]
    return zeilen, abstuerze


def schluessel(z):
    t = z.split(" ", 3)
    if len(t) >= 3 and t[1] == "laden":
        return (t[0], "laden")
    if len(t) >= 4 and t[2] == "laden":
        return (t[0], t[1], "laden")
    return tuple(t[:3])


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "--arbeiter":
        arbeiter(sys.argv[2], sys.argv[3], sys.argv[4].split("|"))
        return 0
    module = [m for m in ("ft", "font") if len(sys.argv) < 2 or sys.argv[1] == m]
    for modus in module:
        ergebnis = {}
        for eng in ("pg", "rg"):
            zeilen, abstuerze = fahren(eng, modus)
            ergebnis[eng] = {schluessel(z): z for z in zeilen}
            if abstuerze:
                print("%s %s: Absturz bei %s" % (modus, eng, ", ".join(abstuerze)))
        a, b = ergebnis["pg"], ergebnis["rg"]
        gemeinsam = [k for k in a if k in b]
        anders = [k for k in gemeinsam if a[k] != b[k]]
        # Abgelehnte Schriften stehen nur als `laden`-Zeile da.
        nur_pg = Counter(k[0] for k in a if k not in b and k[-1] != "laden")
        nur_rg = Counter(k[0] for k in b if k not in a and k[-1] != "laden")
        name = "pygame.freetype" if modus == "ft" else "pygame.font"
        print("%s: %d gemeinsame Faelle, %d verschieden in %d Schriften" % (
            name, len(gemeinsam), len(anders), len({k[0] for k in anders})))
        for schrift, n in sorted(Counter(k[0] for k in anders).items(), key=lambda kv: -kv[1]):
            print("   %-32s %d" % (schrift, n))
        print("  nur bei pygame geladen: %d Schriften (%s)" % (
            len(nur_pg), ", ".join(sorted({n.rsplit(".", 1)[-1].lower() for n in nur_pg}))))
        print("  nur bei rustgame geladen: %d Schriften" % len(nur_rg))
    return 0


if __name__ == "__main__":
    sys.exit(main())
