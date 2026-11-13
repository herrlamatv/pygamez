# -*- coding: utf-8 -*-
"""
vollbild_check.py
=================
Prueft das Vollbild (F11 / Knopf in der Seitenleiste) der Rust-Fassung im
ECHTEN Fenster - das kann vergleich.py nicht, dort gibt es kein Fenster.

Startet ``pygamez.exe`` mit ``PYGAMEZ_VOLLBILD_PROBE``: das Programm schaltet
selbst um (Bild 30 ueber die Taste F11, Bild 70 ueber den Knopf), misst
Fensterstil, Fensterlage, Innenflaeche, Monitor und Bildpuffer und beendet
sich nach rund zwei Sekunden. Es werden keine Tasten an Windows geschickt und
kein Fokus geholt. Datenordner: frischer Kratzordner (Ton aus).

    .venv/Scripts/python.exe rust/tools/vollbild_check.py [--bilder <ordner>]
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))
EXE = os.path.join(WURZEL, "rust.output", "release", "pygamez.exe")

ZEILE = re.compile(r"(\w+): stil=(0x[0-9a-f]+) rahmen=(\w+) fenster=(-?\d+),(-?\d+) (\d+)x(\d+) "
                   r"innen=(\d+)x(\d+) monitor=(-?\d+),(-?\d+) (\d+)x(\d+) puffer=(\d+)x(\d+)")


START = re.compile(r"start: symbol=(\w+) dpi=(\d+) bildschirm=(\d+)x(\d+) leiste=(\d+) bedarf=(\d+) "
                   r"aufloesung=(\d+)x(\d+)")


def lesen(text):
    stufen = {}
    for z in text.splitlines():
        m = START.match(z.strip())
        if m:
            g = m.groups()
            stufen["start"] = {"symbol": g[0] == "true", "dpi": int(g[1]),
                               "bildschirm": (int(g[2]), int(g[3])), "leiste": int(g[4]),
                               "bedarf": int(g[5]), "aufloesung": (int(g[6]), int(g[7]))}
            continue
        m = ZEILE.match(z.strip())
        if not m:
            continue
        g = m.groups()
        stufen[g[0]] = {"stil": g[1], "rahmen": g[2] == "true",
                        "fenster": tuple(map(int, g[3:7])), "innen": tuple(map(int, g[7:9])),
                        "monitor": tuple(map(int, g[9:13])), "puffer": tuple(map(int, g[13:15]))}
    return stufen


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--bilder", help="Ordner, in den die drei Bilder kopiert werden")
    args = ap.parse_args()
    if not os.path.exists(EXE):
        raise SystemExit("Erst bauen: cargo build -p pygamez --release --offline")

    daten = tempfile.mkdtemp(prefix="pgz_vollbild_")
    with open(os.path.join(daten, "settings.json"), "w", encoding="utf-8") as f:
        json.dump({"sound": False}, f)
    bericht = os.path.join(daten, "probe.txt")
    env = dict(os.environ, PYGAMEZ_DATA_DIR=daten, PYGAMEZ_VOLLBILD_PROBE=bericht)
    for k in ("PYGAMEZ_SHOT", "PYGAMEZ_GAME", "PYGAMEZ_SCREEN", "PYGAMEZ_KEYS", "PYGAMEZ_KEYS_FILE"):
        env.pop(k, None)
    try:
        subprocess.run([EXE], env=env, timeout=60, check=False)
        if not os.path.exists(bericht):
            raise SystemExit("FEHLER: kein Bericht - Fenster liess sich nicht oeffnen?")
        with open(bericht, encoding="utf-8") as f:
            text = f.read()
        print(text, end="")
        s = lesen(text)
        fehler = []
        if set(s) != {"start", "vorher", "vollbild", "zurueck", "klein"}:
            raise SystemExit("FEHLER: unvollstaendiger Bericht")
        st, v, b, z, k = s["start"], s["vorher"], s["vollbild"], s["zurueck"], s["klein"]
        mon = b["monitor"]

        def pruefe(bed, text):
            print(("ok     " if bed else "FEHLER ") + text)
            if not bed:
                fehler.append(text)

        pruefe(st["symbol"], "Programm-Symbol fuer Titel- und Taskleiste geladen")
        # _fit_sidebar: h = max(560, min(need_h, Bildschirm - 120)),
        # w = min(Bildschirm - 80, need_w + max(640, int(h * game_w / game_h)))
        sw, sh = st["bildschirm"]
        gw, gh = st["aufloesung"]
        soll_h = max(560, min(st["bedarf"], sh - 120))
        soll_w = min(sw - 80, st["leiste"] + max(640, int(soll_h * gw / max(1, gh))))
        pruefe(v["innen"] == (soll_w, soll_h), "Startgroesse wie main.py: %dx%d" % (soll_w, soll_h))
        pruefe(v["rahmen"], "vorher: Fenster mit Rahmen")
        pruefe(v["puffer"] == v["innen"], "vorher: Bildpuffer = Innenflaeche %dx%d" % v["innen"])
        pruefe(not b["rahmen"], "F11: Rahmen abgenommen")
        pruefe(b["fenster"] == mon, "F11: Fenster deckt den Monitor %d,%d %dx%d" % mon)
        pruefe(b["innen"] == mon[2:], "F11: Innenflaeche = Monitor")
        pruefe(b["puffer"] == mon[2:], "F11: Bildpuffer folgt (%dx%d)" % b["puffer"])
        pruefe(z["stil"] == v["stil"], "Knopf: Fensterstil wie vorher (%s)" % v["stil"])
        pruefe(z["fenster"] == v["fenster"], "Knopf: Fensterlage wie vorher")
        pruefe(z["puffer"] == v["puffer"], "Knopf: Bildpuffer wie vorher")
        pruefe(k["innen"] == (880, 560), "Mindestgroesse 880x560 haelt (gewuenscht 300x200, ist %dx%d)" % k["innen"])
        pruefe(k["puffer"] == k["innen"], "klein: Bildpuffer folgt")

        try:
            os.environ.setdefault("PYGAME_HIDE_SUPPORT_PROMPT", "1")
            import pygame
            for stufe, werte in s.items():
                if stufe == "start":
                    continue
                bild = pygame.image.load(bericht + "." + stufe + ".png")
                pruefe(bild.get_size() == werte["puffer"], "%s: Bild %dx%d" % ((stufe,) + bild.get_size()))
        except ImportError:
            print("(pygame fehlt - Bildgroessen nicht geprueft)")

        if args.bilder:
            os.makedirs(args.bilder, exist_ok=True)
            for stufe in s:
                if stufe == "start":
                    continue
                shutil.copy(bericht + "." + stufe + ".png", os.path.join(args.bilder, "vollbild_%s.png" % stufe))
        print("Vollbild:", "in Ordnung" if not fehler else "%d Fehler" % len(fehler))
        return 1 if fehler else 0
    finally:
        shutil.rmtree(daten, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
