# -*- coding: utf-8 -*-
"""Vergleicht, **welche** Tests jede Engine ueberspringt - nicht wie viele.

## Die Frage, die eine Zahl nicht beantwortet

`pygame_tests.py` meldet `2364 gut, 0 falsch, 0 Fehler, 121 uebersprungen`
und daneben, dass pygame selbst dieselben 121 ueberspringt. Das klingt
abgeschlossen - und es war eine **Zahl**, keine Menge.

121 gegen 121 kann naemlich auch heissen: rustgame ueberspringt Test A,
pygame fahrt ihn, und pygame ueberspringt dafuer Test B, den rustgame
fahrt. Die Summe stimmt, und ein Test ist **unsichtbar** geworden - nicht
rot, sondern gar nicht gelaufen. Genau diese Sorte Fehler hat in diesem
Projekt schon dreimal zugeschlagen: eine Testdatei, die nicht in der Liste
stand; eine, die auf Modulebene nicht lud; fuenfhundert Pruefungen, die der
Laeufer gar nicht fand.

Dieses Werkzeug stellt die **Namen** gegeneinander:

```bash
.venv/Scripts/python.exe rustgame/tools/uebersprungen.py
```

Es fahrt `pygame_tests.py` zweimal - einmal gegen rustgame, einmal mit
`--pygame` gegen das echte - und vergleicht die beiden Listen. Gemessen
beim Einbau: **dieselben 121, Name fuer Name, in beiden Richtungen leer.**

Der Lauf dauert zusammen ein paar Minuten; er gehoert nicht in jede Runde,
aber in jede, die am Laeufer oder an den Ausschlussregeln etwas aendert.
"""

import os
import subprocess
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
RUSTGAME = os.path.dirname(HIER)
LAEUFER = os.path.join(RUSTGAME, "crates", "rustgame-py", "tools",
                       "pygame_tests.py")


def liste(gegen_pygame):
    """Die Kennungen der uebersprungenen Tests, sortiert.

    `--uebersprungen` gibt je Zeile `UEBERSPRUNGEN <kennung>`; alles andere
    aus der Ausgabe wird hier weggelassen. `PYTHONIOENCODING=utf-8` muss
    mit, weil die Konsole sonst an einem Gedankenstrich in einer
    Testmeldung aussteigt - nachgemessen.
    """
    umgebung = dict(os.environ)
    umgebung["PYTHONIOENCODING"] = "utf-8"
    befehl = [sys.executable, LAEUFER, "--uebersprungen"]
    if gegen_pygame:
        befehl.append("--pygame")
    fertig = subprocess.run(
        befehl, cwd=RUSTGAME, env=umgebung,
        stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True,
        encoding="utf-8", errors="replace",
    )
    aus = []
    for zeile in fertig.stdout.splitlines():
        if zeile.startswith("UEBERSPRUNGEN "):
            aus.append(zeile[len("UEBERSPRUNGEN "):].strip())
    return sorted(aus)


def main():
    print("Laeuft gegen rustgame ...")
    rust = liste(False)
    print("Laeuft gegen echtes pygame ...")
    echt = liste(True)

    print()
    print("pygame ueberspringt %d, rustgame %d" % (len(echt), len(rust)))
    nur_rust = [x for x in rust if x not in set(echt)]
    nur_echt = [x for x in echt if x not in set(rust)]

    if nur_rust:
        print()
        print("**Nur rustgame ueberspringt diese - sie sind unsichtbar:**")
        for x in nur_rust:
            print("   ", x)
    if nur_echt:
        print()
        print("Nur pygame ueberspringt diese (rustgame fahrt sie):")
        for x in nur_echt:
            print("   ", x)
    if not rust and not echt:
        print("Keine Liste - ist `--uebersprungen` noch da?")
        return 1
    if nur_rust or nur_echt:
        return 1
    print()
    print("Dieselben %d Tests, Name fuer Name." % len(rust))
    return 0


if __name__ == "__main__":
    sys.exit(main())
