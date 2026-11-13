# -*- coding: utf-8 -*-
"""
import_check.py
===============
Importieren geteilter Dateien gegen Python pruefen - Minigolf-Bahnen,
Geometry-Dash-Level und Replays. In der App geht das nur ueber den
Datei-Dialog, den der Pruefstand nicht hat; verglichen werden deshalb die
Kernfunktionen (``ugc.import_from`` / ``replay.import_from``).

Ueber Kreuz: die Regressionslaeufe zum Teilen erzeugen echte Exportdateien
aus Python UND aus Rust. Jede Fassung importiert beide (je zweimal: das
zweite Mal muss eine neue id "-2" bzw. "dup" ergeben), dazu Fehlerfaelle
(falsche Art, kaputte Datei). Verglichen werden Antwort und gespeicherte
Daten (ugc.json, replay.json, mem.json ...).

    .venv/Scripts/python.exe rust/tools/import_check.py

Nur Kratzordner, nie die echten Spielstaende.
"""

import os
import re
import shutil
import subprocess
import sys
import tempfile

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))
sys.path.insert(0, HIER)
from regression import LAEUFE, folge  # noqa: E402
from stats_check import ordner_vergleichen  # noqa: E402

EXE = os.path.join(WURZEL, "rust.output", "release", "examples", "import_io.exe")
PY = os.path.join(WURZEL, ".venv", "Scripts", "python.exe")
if not os.path.exists(PY):
    PY = sys.executable

# Regressionslaeufe, die etwas nach "Downloads" teilen: (spiel, Index in LAEUFE)
QUELLEN = [("geodash", 3), ("minigolf", 2), ("snake", 1)]

PY_SEITE = r'''
import os, sys
sys.path.insert(0, %(wurzel)r)
import store
store._DIR = %(daten)r
store._PATH = os.path.join(store._DIR, "mem.json")
store._LEGACY_HS_PATH = os.path.join(store._DIR, "highscores.json")
import ugc, replay
ugc._PATH = os.path.join(store._DIR, "ugc.json")
replay._PATH = os.path.join(store._DIR, "replay.json")
# Wie der Programmstart (Python: i18n.init -> store.load, Rust: pgz_core::init)
store.load()
art, datei, spiel = %(args)r
spiel = None if spiel == "-" else spiel
if art == "ugc":
    ok, why, m = ugc.import_from(datei, spiel)
else:
    ok, why, m = replay.import_from(datei, spiel)
print("%%s %%s %%s" %% (str(ok).lower(), why, (m or {}).get("id", "")))
'''


def python(daten, args):
    code = PY_SEITE % {"wurzel": WURZEL, "daten": daten, "args": tuple(args)}
    r = subprocess.run([PY, "-c", code], capture_output=True, text=True, cwd=WURZEL)
    return (r.stdout.strip().splitlines() or [r.stderr.strip()[-200:]])[-1]


def rust(daten, args):
    env = dict(os.environ, PYGAMEZ_DATA_DIR=daten)
    r = subprocess.run([EXE] + list(args), capture_output=True, text=True, env=env)
    return (r.stdout.strip().splitlines() or [r.stderr.strip()[-200:]])[-1]


def exporte(tmp):
    """Laesst die Teilen-Laeufe laufen und sammelt die Dateien aus beiden
    Fassungen: {(spiel, "py"|"rust"): pfad}."""
    out = {}
    for spiel, nr in QUELLEN:
        modus, bilder, tokens, abstand = LAEUFE[spiel][nr]
        ordner = os.path.join(tmp, "lauf_%s" % spiel)
        os.makedirs(ordner, exist_ok=True)
        liste = os.path.join(ordner, "keys.txt")
        with open(liste, "w", encoding="utf-8") as f:
            f.write(",".join(folge(tokens, abstand)))
        subprocess.run([PY, os.path.join(HIER, "vergleich.py"), spiel, "--mode", modus,
                        "--frames", str(bilder), "--keys-file", liste, "--out", ordner],
                       cwd=WURZEL, capture_output=True, text=True)
        for seite, name in (("py", "_pydata"), ("rust", "_rsdata")):
            dl = os.path.join(ordner, spiel + name, "Downloads")
            dateien = sorted(os.listdir(dl)) if os.path.isdir(dl) else []
            if not dateien:
                raise SystemExit("FEHLER: kein Export aus %s (%s)" % (spiel, seite))
            out[(spiel, seite)] = os.path.join(dl, dateien[0])
    return out


def main():
    if not os.path.exists(EXE):
        raise SystemExit("Erst bauen: cargo build -p pgz-games --release --offline --example import_io")
    tmp = tempfile.mkdtemp(prefix="pgz_import_")
    try:
        quellen = exporte(tmp)
        kaputt = os.path.join(tmp, "kaputt.lamapgzlevel")
        with open(kaputt, "w", encoding="utf-8") as f:
            f.write("{ das ist kein JSON")
        art = {"geodash": ("ugc", "geodash"), "minigolf": ("ugc", "minigolf"), "snake": ("replay", "-")}
        faelle = []
        for (spiel, seite), pfad in sorted(quellen.items()):
            a, sp = art[spiel]
            faelle.append(("%s aus %s, zweimal" % (spiel, seite), [[a, pfad, sp], [a, pfad, sp]]))
        faelle += [
            ("Level als Minigolf-Bahn", [["ugc", quellen[("geodash", "py")], "minigolf"]]),
            ("Bahn ohne Art (alle)", [["ugc", quellen[("minigolf", "rust")], "-"]]),
            ("Replay im falschen Reiter", [["replay", quellen[("snake", "py")], "tetris"]]),
            ("kaputte Datei", [["ugc", kaputt, "geodash"]]),
            ("fehlende Datei", [["replay", os.path.join(tmp, "weg.lamapgzreplay"), "-"]]),
        ]
        schlecht = 0
        for nr, (name, schritte) in enumerate(faelle):
            dp, dr = os.path.join(tmp, "f%d_py" % nr), os.path.join(tmp, "f%d_rs" % nr)
            os.makedirs(dp)
            os.makedirs(dr)
            ap = [python(dp, s) for s in schritte]
            ar = [rust(dr, s) for s in schritte]
            fehler = ordner_vergleichen(dp, dr)
            ok = fehler is None and ap == ar
            schlecht += not ok
            print("%-6s %-28s py: %-36s rust: %-36s %s" % ("ok" if ok else "FEHLER", name, " | ".join(ap),
                                                          " | ".join(ar), fehler or ""))
        print("Import geteilter Dateien:", "in Ordnung" if not schlecht else "%d Fehler" % schlecht)
        return 1 if schlecht else 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
