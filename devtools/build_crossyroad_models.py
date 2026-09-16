# -*- coding: utf-8 -*-
"""
build_crossyroad_models.py
==========================
Schreibt die Voxel-Modelle von Crossy Road (``games/crossyroad_world.py``)
als Daten-Datei für die Web-Version nach
``web/js/games/crossyroad_models.js``. So gibt es nur EINE Quelle für das
Aussehen aller Figuren, Fahrzeuge, Bäume, Züge und des Adlers.

Aufruf aus dem Repo-Root:  python devtools/build_crossyroad_models.py
            nur prüfen:     python devtools/build_crossyroad_models.py --check
"""

import json
import os
import sys
import types

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, ROOT)
# Nur das Welt-Modul laden - nicht das ganze games-Paket (pygame, alle Spiele).
_pkg = types.ModuleType("games")
_pkg.__path__ = [os.path.join(ROOT, "games")]
sys.modules.setdefault("games", _pkg)
from games import crossyroad_world as cw  # noqa: E402

OUT = os.path.join(ROOT, "web", "js", "games", "crossyroad_models.js")


def _box(b):
    return [round(v, 4) for v in b[:6]] + [list(b[6])]


def build():
    data = {
        "chars": {cid: [_box(b) for b in cw.CHAR_MODELS[cid]] for cid in cw.CHAR_IDS},
        "models": {name: [_box(b) for b in boxes] for name, boxes in sorted(cw.MODELS.items())},
        "charColors": {cid: list(col) for cid, col in cw.CHAR_COLORS.items()},
    }
    body = json.dumps(data, separators=(",", ":"))
    return ("/*\n"
            " * crossyroad_models.js - Voxel-Modelle für Crossy Road (ERZEUGT)\n"
            " * Quelle: games/crossyroad_world.py, erzeugt von\n"
            " * devtools/build_crossyroad_models.py - nicht von Hand ändern.\n"
            " * Quader: [x0, y0, z0, x1, y1, z1, [r, g, b]] in Kacheleinheiten.\n"
            " */\n"
            "(function () {\n"
            "  \"use strict\";\n"
            "  window.PG.crossyModels = " + body + ";\n"
            "})();\n")


def main():
    text = build()
    if "--check" in sys.argv:
        try:
            with open(OUT, "r", encoding="utf-8") as f:
                same = f.read() == text
        except OSError:
            same = False
        print("aktuell" if same else "VERALTET: bitte neu erzeugen")
        return 0 if same else 1
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print("geschrieben: %s (%d Bytes)" % (OUT, len(text)))
    return 0


if __name__ == "__main__":
    sys.exit(main())
