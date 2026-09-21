# -*- coding: utf-8 -*-
"""
export_achievements.py
======================
Schreibt die Erfolgs- und Level-Definitionen aus ``achievements.py`` und
``levels.py`` nach ``web/js/core/achdefs.js``.

So gibt es für Desktop und Web nur EINE Quelle: wer hier eine Seltenheitsstufe
ändert oder einen Erfolg ergänzt, führt das Skript aus und die Browser-Version
kennt denselben Stand. Die erzeugte Datei ist reines JavaScript (klassisches
``<script>``, kein Modul) und definiert ``PG.achDefs``.

Aufruf::

    python devtools/export_achievements.py            # schreiben
    python devtools/export_achievements.py --check    # nur prüfen (Exit 1 bei Abweichung)
"""

import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "web", "js", "core", "achdefs.js")

sys.path.insert(0, ROOT)


def build():
    """Erzeugt den Inhalt der achdefs.js (String)."""
    import achievements
    import levels

    data = {
        "rarity": {name: {"points": spec["points"], "color": spec["color"]}
                   for name, spec in achievements.RARITY.items()},
        "rarityOrder": list(achievements.RARITY_ORDER),
        "regularPlays": achievements.REGULAR_PLAYS,
        "general": [list(entry) for entry in achievements.GENERAL],
        "milestones": dict(achievements.MILESTONES),
        "masters": dict(achievements.MASTERS),
        "specials": [list(entry) for entry in achievements.SPECIALS],
        "statDesc": dict(achievements._STAT_DESC),
        "ranks": [[lvl, key, color] for lvl, key, color in levels.RANKS],
        "levelBase": levels._BASE,
        "levelStep": levels._STEP,
    }
    # Je Oberbegriff eine Zeile: kompakt genug zum Durchscrollen, aber noch
    # lesbar (die Datei wird erzeugt, nicht gepflegt).
    lines = []
    keys = sorted(data)
    for i, key in enumerate(keys):
        value = json.dumps(data[key], ensure_ascii=False, sort_keys=True,
                           separators=(", ", ": "))
        comma = "," if i < len(keys) - 1 else ""
        lines.append("    %s: %s%s" % (json.dumps(key), value, comma))
    return (
        "// Erzeugt von devtools/export_achievements.py - NICHT von Hand ändern.\n"
        "// Quelle: achievements.py (Erfolge, Seltenheitsstufen) + levels.py (Ränge).\n"
        "(function () {\n"
        '  "use strict";\n'
        "  PG.achDefs = {\n"
        + "\n".join(lines) + "\n"
        "  };\n"
        "})();\n")


def main():
    text = build()
    check = "--check" in sys.argv
    old = None
    if os.path.exists(OUT):
        with open(OUT, "r", encoding="utf-8") as f:
            old = f.read()
    if check:
        if old == text:
            print("OK: achdefs.js ist aktuell.")
            return 0
        print("FEHLER: achdefs.js weicht ab - "
              "python devtools/export_achievements.py ausführen.")
        return 1
    if old == text:
        print("unverändert:", os.path.relpath(OUT, ROOT))
        return 0
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print("geschrieben:", os.path.relpath(OUT, ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
