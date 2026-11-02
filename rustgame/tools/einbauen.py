# -*- coding: utf-8 -*-
"""Baut die Huelle und legt sie **an jede Stelle**, von der sie geladen wird.

## Warum es dieses Werkzeug gibt

**Dreimal** hat derselbe Fehler in diesem Projekt Zeit gekostet, und jedes
Mal war er dieselbe Sorte: **gemessen wurde ein Modul, das nicht das
gebaute war.**

Erstes Mal - die Kette

```bash
cargo build --release | grep -c '^error' && cp ... .pyd
```

`grep -c` gibt bei **null** Treffern den Rueckgabewert 1. Also lief das
`cp` nicht, der Bau war aber in Ordnung, und die naechste Messung galt der
alten Datei. Drei Laeufe lang stand dort "2 Fehler", die es nicht mehr gab.

Zweites Mal - **zwei Kopien**. `vergleich_module.py` legt
`crates/rustgame-py` vor `site-packages` in den Suchpfad, und dort lag eine
zweite `.pyd` von einem frueheren Bau. Erneuert wurde die in
`site-packages`; geladen wurde die andere. Der Fehlertext in der Ausgabe
war der eines Entwurfs, den es im Quelltext gar nicht mehr gab - das war
der Hinweis, und nur deshalb fiel es auf.

Drittes Mal - **`-p` am falschen Ort**. `rustgame-py` steht ausserhalb des
Workspace, also antwortet `cargo build --release -p rustgame-py` vom
Workspace aus mit `error: package ID specification ... did not match any
packages`. Gebaut wurde nichts; die Messung danach galt wieder der alten
Datei. Siehe `bauen()`.

Daraus die Regel, die dieses Werkzeug durchsetzt: **wer baut, legt die
Datei an jede Stelle, von der sie geladen werden kann, und prueft danach,
dass sie wirklich dort liegt.** Nicht der Bau wird geprueft, sondern das
Ergebnis.

## Gebrauch

```bash
.venv/Scripts/python.exe rustgame/tools/einbauen.py          # bauen + einbauen
.venv/Scripts/python.exe rustgame/tools/einbauen.py --pruefen # nur nachsehen
```

Ohne `--pruefen` laeuft `cargo build --release` **im Ordner der Kiste**
(nicht im Workspace, siehe `bauen()`), und zwar **ohne** eine Pipe
dahinter: der Rueckgabewert von `cargo` selbst entscheidet, nicht der von
`grep`.

Danach steht in der Ausgabe fuer jede Stelle, ob sie gelegt wurde, schon
gleich war oder **nicht** geschrieben werden konnte - eine `.pyd`, die ein
laufendes Python geladen hat, laesst sich unter Windows nicht ersetzen.

## Und die Begleitdateien

Die `.pyd` allein ist nicht das Paket. pygame hat `freesansbold.ttf` und
`pygame_icon.bmp` **neben** seinem Modul liegen, und rustgame sucht sie
ebenso dort (`rustgame_font::set_schriftordner`, `pkgdata.getResource`).
Gelegt wurde lange nur die `.pyd`: in `site-packages` fehlte die Schrift,
und `Font(None, 20)` fand sie nur, weil die Suche am Ende in **pygames**
Paket nachsah. Ohne installiertes pygame waere das ein Absturz gewesen.
Darum legt dieses Werkzeug die Dateien aus `BEGLEITER` mit und prueft sie
wie die `.pyd`.
"""

import hashlib
import os
import shutil
import subprocess
import sys
import sysconfig
import time

HIER = os.path.dirname(os.path.abspath(__file__))
RUSTGAME = os.path.dirname(HIER)
WURZEL = os.path.dirname(RUSTGAME)

#: Was `cargo` erzeugt. Unter Windows `.dll`, sonst `.so`/`.dylib`.
QUELLEN = [
    os.path.join(WURZEL, "rustgame.output", "release", name)
    for name in ("rustgame_py.dll", "librustgame_py.so", "librustgame_py.dylib")
]

#: Wie die Datei heissen muss, damit Python sie als Modul nimmt.
ZIELNAME = "rustgame_py" + (".pyd" if sys.platform == "win32" else ".so")

#: Wo die Begleitdateien im Quellbaum liegen.
KISTE = os.path.join(RUSTGAME, "crates", "rustgame-py")

#: Was neben dem Modul liegen muss - siehe "Und die Begleitdateien".
BEGLEITER = ("freesansbold.ttf", "pygame_icon.bmp")


def ziele():
    """**Jede** Stelle, von der die Huelle geladen werden kann.

    Zwei, und die zweite ist die, die den Fehler gemacht hat:

    * `site-packages` - der normale Weg, `import rustgame_py`.
    * `crates/rustgame-py/` - weil `vergleich_module.py`,
      `vergleich_spiel.py` und `pygame_tests.py` diesen Ordner **vor**
      `site-packages` in `sys.path` legen, damit sie ohne Installation
      laufen. Was hier liegt, gewinnt.
    """
    return [
        os.path.join(sysconfig.get_paths()["purelib"], ZIELNAME),
        os.path.join(RUSTGAME, "crates", "rustgame-py", ZIELNAME),
    ]


def summe(pfad):
    """SHA-256 einer Datei - oder `None`, wenn es sie nicht gibt."""
    if not os.path.exists(pfad):
        return None
    h = hashlib.sha256()
    with open(pfad, "rb") as f:
        for stueck in iter(lambda: f.read(1 << 20), b""):
            h.update(stueck)
    return h.hexdigest()


def quelle():
    """Die gebaute Datei - oder ein Abbruch mit dem Grund."""
    for p in QUELLEN:
        if os.path.exists(p):
            return p
    raise SystemExit(
        "Keine gebaute Huelle gefunden. Gesucht:\n  "
        + "\n  ".join(QUELLEN)
    )


def bauen():
    """`cargo build --release` **in der Kiste** - ohne Pipe dahinter.

    Der Rueckgabewert von `cargo` entscheidet. Eine Pipe daran ist genau
    der Fehler, der dieses Werkzeug noetig gemacht hat.

    ## Und `-p rustgame-py` geht hier **nicht**

    Das war der dritte Fall derselben Sorte. `rustgame-py` hat eine eigene
    `[workspace]`-Tabelle und steht damit **ausserhalb** des Workspace
    (Grund: `extension-module` bindet kein libpython ein, also liesse sich
    kein Testprogramm mehr linken, und `cargo test --workspace` waere fuer
    den ganzen Rest hin). Vom Workspace aus gerufen antwortet cargo

    ```text
    error: package ID specification `rustgame-py` did not match any packages
    ```

    und wer das durch ein `grep` schickt, das nur nach bestimmten Zeilen
    sucht, sieht davon unter Umstaenden nichts - und erneuert eine Datei,
    die gar nicht neu gebaut wurde. Gebaut wird darum **im Ordner der
    Kiste** und ohne `-p`.
    """
    # `cargo` muss ueber `which` gesucht werden: `CreateProcess` unter
    # Windows kennt `PATHEXT` nicht und findet `cargo.exe` nicht, wenn man
    # nur `cargo` sagt.
    weg = shutil.which("cargo")
    if weg is None:
        # Auf dieser Kiste liegt `cargo` in `~/.cargo/bin`, und das steht
        # nicht in `PATH` - rustup legt es dort ab und traegt es nur in die
        # Shell-Profile ein. Also dort nachsehen, bevor abgebrochen wird.
        for p in (
            os.path.expanduser("~/.cargo/bin/cargo.exe"),
            os.path.expanduser("~/.cargo/bin/cargo"),
        ):
            if os.path.exists(p):
                weg = p
                break
    if weg is None:
        raise SystemExit(
            "`cargo` ist nicht zu finden - weder im Pfad noch in ~/.cargo/bin."
        )
    kiste = os.path.join(RUSTGAME, "crates", "rustgame-py")
    print("cargo build --release   (in %s)" % kiste)
    fertig = subprocess.run([weg, "build", "--release"], cwd=kiste)
    if fertig.returncode != 0:
        raise SystemExit("cargo ist mit %d ausgestiegen." % fertig.returncode)


def stuecke():
    """Jedes Paar `(quelle, ziel)`, das gelegt sein muss.

    Zuerst die `.pyd` an jede Stelle, dann die Begleitdateien neben jede
    Stelle, die nicht selbst der Quellordner ist.
    """
    q = quelle()
    paare = [(q, z) for z in ziele()]
    for z in ziele():
        ordner = os.path.dirname(z)
        if os.path.normcase(os.path.abspath(ordner)) == os.path.normcase(KISTE):
            continue
        for name in BEGLEITER:
            paare.append((os.path.join(KISTE, name), os.path.join(ordner, name)))
    return paare


def begleiter_einbauen():
    """Legt die Begleitdateien - und zaehlt, was nicht gelang."""
    schlecht = 0
    for q, z in stuecke():
        if not z.endswith(BEGLEITER):
            continue
        qs, alt = summe(q), summe(z)
        if qs is None:
            print("FEHLER   Quelle fehlt: %s" % q)
            schlecht += 1
            continue
        if alt == qs:
            print("gleich   %s" % z)
            continue
        try:
            shutil.copyfile(q, z)
        except OSError as e:
            print("FEHLER   %s: %s" % (z, e))
            schlecht += 1
            continue
        if summe(z) != qs:
            print("FEHLER   %s hat danach einen anderen Inhalt" % z)
            schlecht += 1
        else:
            print("gelegt   %s%s" % (z, "" if alt is None else "  (war %s)" % alt[:16]))
    return schlecht


def einbauen():
    """Legt die gebaute Datei an jede Stelle - und prueft danach."""
    q = quelle()
    qs = summe(q)
    print("gebaut   %s" % q)
    print("         %s  %.1f MiB  %s"
          % (qs[:16], os.path.getsize(q) / (1 << 20),
             time.strftime("%H:%M:%S", time.localtime(os.path.getmtime(q)))))
    schlecht = 0
    for z in ziele():
        alt = summe(z)
        if alt == qs:
            print("gleich   %s" % z)
            continue
        try:
            shutil.copyfile(q, z)
        except OSError as e:
            print("FEHLER   %s: %s" % (z, e))
            schlecht += 1
            continue
        # **Nachsehen, nicht annehmen.** Eine `.pyd`, die gerade von einem
        # laufenden Python geladen ist, laesst sich unter Windows nicht
        # ersetzen - das Kopieren schlaegt dann fehl, und ohne diese
        # Pruefung waere die Messung danach wieder die alte.
        neu = summe(z)
        if neu != qs:
            print("FEHLER   %s hat danach %s" % (z, (neu or "nichts")[:16]))
            schlecht += 1
        else:
            print("gelegt   %s%s" % (z, "" if alt is None else "  (war %s)" % alt[:16]))
    return schlecht + begleiter_einbauen()


def pruefen():
    """Sagt, ob alle Kopien dieselbe sind - ohne etwas zu bauen."""
    q = quelle()
    qs = summe(q)
    print("gebaut   %s  %s" % (qs[:16], q))
    schlecht = 0
    for z in ziele():
        s = summe(z)
        if s is None:
            print("fehlt    %s" % z)
            schlecht += 1
        elif s != qs:
            print("ALT      %s  %s" % (s[:16], z))
            schlecht += 1
        else:
            print("gleich   %s  %s" % (s[:16], z))
    for q, z in stuecke():
        if not z.endswith(BEGLEITER):
            continue
        s = summe(z)
        if s is None:
            print("fehlt    %s" % z)
            schlecht += 1
        elif s != summe(q):
            print("ALT      %s  %s" % (s[:16], z))
            schlecht += 1
        else:
            print("gleich   %s  %s" % (s[:16], z))
    return schlecht


def main():
    nur = "--pruefen" in sys.argv
    if not nur:
        bauen()
    schlecht = pruefen() if nur else einbauen()
    print()
    if schlecht:
        print("%d Stelle(n) nicht auf dem Stand des Baus." % schlecht)
        return 1
    print("Alle Kopien sind der gebaute Stand.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
