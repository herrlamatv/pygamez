# -*- coding: utf-8 -*-
"""
tempo.py
========
Misst, wie schnell die Rust-Fassung rechnet und zeichnet: jedes Spiel laeuft
im Pruefstand mit seiner ersten Regressionsfolge (``regression.LAEUFE``),
gemessen werden Millisekunden je Bild (Update + Zeichnen + Einblendungen,
ohne Fenster). Startzeit des Programms wird ueber einen 2-Bilder-Lauf
abgezogen.

    .venv/Scripts/python.exe rust/tools/tempo.py [--aufloesung 1600x1000] [spiel ...]

Alles unter ~16 ms je Bild reicht fuer 60 FPS.
"""

import os
import subprocess
import sys
import tempfile
import time

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))
sys.path.insert(0, HIER)

import regression  # noqa: E402

EXE = os.path.join(WURZEL, "rust.output", "release", "pygamez.exe")


AUFLOESUNG = None


def lauf(key, mode, frames, liste, ordner):
    daten = os.path.join(ordner, "daten_" + key)
    os.makedirs(daten, exist_ok=True)
    with open(os.path.join(daten, "mem.json"), "w", encoding="utf-8") as f:
        f.write('{"mem": {"lang": "de"}}')
    if AUFLOESUNG:
        with open(os.path.join(daten, "settings.json"), "w", encoding="utf-8") as f:
            f.write('{"resolution": [%d, %d], "auto_resolution": false}' % AUFLOESUNG)
    env = dict(os.environ)
    env.update(PYGAMEZ_DATA_DIR=daten, PYGAMEZ_GAME=key, PYGAMEZ_MODE=mode, PYGAMEZ_SEED="7",
               PYGAMEZ_FRAMES=str(frames), PYGAMEZ_KEYS_FILE=liste,
               PYGAMEZ_SHOT=os.path.join(ordner, key + ".png"))
    env.pop("PYGAMEZ_SCREEN", None)
    t0 = time.perf_counter()
    subprocess.run([EXE], env=env, capture_output=True)
    return time.perf_counter() - t0


def main():
    global AUFLOESUNG
    nur = sys.argv[1:]
    if len(nur) >= 2 and nur[0] == "--aufloesung":
        w, h = nur[1].lower().split("x")
        AUFLOESUNG = (int(w), int(h))
        nur = nur[2:]
    ordner = tempfile.mkdtemp(prefix="pgz_tempo_")
    zeilen = []
    for key, laeufe in regression.LAEUFE.items():
        if nur and key not in nur:
            continue
        modus, bilder, tokens, abstand = laeufe[0]
        liste = os.path.join(ordner, key + ".txt")
        with open(liste, "w", encoding="utf-8") as f:
            f.write(",".join(regression.folge(tokens, abstand)))
        basis = min(lauf(key, modus, 2, liste, ordner) for _ in range(2))
        ganz = lauf(key, modus, bilder, liste, ordner)
        ms = max(0.0, ganz - basis) * 1000.0 / max(1, bilder - 2)
        zeilen.append((ms, key, modus, bilder))
        print("%-12s %-10s %5d Bilder  %6.2f ms/Bild" % (key, modus, bilder, ms), flush=True)
    zeilen.sort(reverse=True)
    print("\nLangsamste:")
    for ms, key, modus, bilder in zeilen[:8]:
        print("  %-12s %6.2f ms/Bild%s" % (key, ms, "   <-- zu langsam fuer 60 FPS" if ms > 16.0 else ""))


if __name__ == "__main__":
    main()
