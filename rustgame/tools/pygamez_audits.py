# -*- coding: utf-8 -*-
"""
PyGameZ' eigene Audits auf beiden Engines.

`tests/*.py` sind elftausend Zeilen Pruefungen, die niemand fuer rustgame
geschrieben hat: Spiellogik, Layout in vierzehn Sprachen und fuenf
Aufloesungen, Speichern und Laden - und **Zeitbudgets**. Jedes Audit
laeuft hier zweimal, einmal je Engine, in einem eigenen Prozess.

Aufruf (aus dem Projektordner `pygamez`)::

    .venv/Scripts/python.exe rustgame/tools/pygamez_audits.py
    .venv/Scripts/python.exe rustgame/tools/pygamez_audits.py audit_2048 tetris

Endet mit 1, wenn ein Audit auf rustgame scheitert, das auf pygame
besteht. Die vollen Ausgaben liegen in `rustgame.output/pygamez_audits/`.

Zwei Dinge, die das Messgeraet hier beim ersten Lauf falsch gemacht hat
---------------------------------------------------------------------
1. **Kindprozesse.** `arcade_casino_audit.py` startet alle
   `tests/audit_*.py` mit `sys.executable` - die liefen auf echtem
   pygame, und die Zeile "rustgame: ALLES OK" mass zur Haelfte die
   Vorlage. Engine und Datenordner gehen jetzt ueber
   `pygamez_erbe/sitecustomize.py` an jeden Prozess.
2. **Die echte Statistik.** Die Audits leiten `store._PATH` um, aber
   `stats` schreibt ueber `store.path_for` in `store._DIR`. Ein Lauf
   hat so 1125 Duell-Eintraege in `stats.pgzlamastats` geschrieben.
   Jetzt bekommt jeder Lauf einen eigenen leeren Datenordner, und das
   Werkzeug prueft am Ende per SHA-256, dass die Nutzerdateien
   unveraendert sind - sonst endet es mit 1.
"""

import glob
import hashlib
import os
import subprocess
import sys
import tempfile
import time

HIER = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HIER))
ERBE = os.path.join(HIER, "pygamez_erbe")
AUSGABE = os.path.join(ROOT, "rustgame.output", "pygamez_audits")
NUTZERDATEN = ("mem.json", "mem.json.bak", "settings.json", "settings.json.bak",
               "stats.pgzlamastats", "stats.pgzlamastats.bak", "ugc.json",
               "replay.json")


def pruefsummen():
    aus = {}
    for name in NUTZERDATEN:
        p = os.path.join(ROOT, name)
        if os.path.exists(p):
            aus[name] = hashlib.sha256(open(p, "rb").read()).hexdigest()
    return aus


def fahre(pfad, engine):
    env = dict(os.environ, PYTHONIOENCODING="utf-8", PGZ_ENGINE=engine,
               PGZ_DATEN=tempfile.mkdtemp(prefix="pgz-audit-"),
               PYTHONPATH=ERBE, SDL_VIDEODRIVER="dummy",
               SDL_AUDIODRIVER="dummy", PYGAME_HIDE_SUPPORT_PROMPT="1")
    t0 = time.time()
    try:
        r = subprocess.run([sys.executable, "-W", "ignore", pfad], cwd=ROOT, env=env,
                           capture_output=True, timeout=3600)
        code, text = r.returncode, (r.stdout + r.stderr).decode("utf-8", "replace")
    except subprocess.TimeoutExpired as e:
        code = "TIMEOUT"
        text = ((e.stdout or b"") + (e.stderr or b"")).decode("utf-8", "replace")
    return code, text, time.time() - t0


def main():
    wahl = sys.argv[1:]
    dateien = sorted(glob.glob(os.path.join(ROOT, "tests", "*.py")))
    if wahl:
        dateien = [d for d in dateien if any(w in os.path.basename(d) for w in wahl)]
    os.makedirs(AUSGABE, exist_ok=True)
    vorher = pruefsummen()
    schlecht = []
    for d in dateien:
        name = os.path.basename(d)[:-3]
        zeile = []
        codes = {}
        for engine in ("pygame", "rustgame"):
            code, text, dt = fahre(d, engine)
            codes[engine] = code
            with open(os.path.join(AUSGABE, "%s.%s.txt" % (name, engine)), "w",
                      encoding="utf-8") as f:
                f.write(text)
            fehl = [z.strip() for z in text.splitlines() if z.strip().startswith("FAIL")]
            zeile.append("%s %s %5.0fs%s" % (engine, "ok " if code == 0 else "%-3s" % code, dt,
                                            (" (%d FAIL)" % len(fehl)) if fehl else ""))
        if codes["rustgame"] != 0 and codes["pygame"] == 0:
            schlecht.append(name)
        print("%-24s %s" % (name, "   ".join(zeile)), flush=True)
    nachher = pruefsummen()
    if nachher != vorher:
        print("\n!! NUTZERDATEN VERAENDERT:",
              [n for n in set(vorher) | set(nachher) if vorher.get(n) != nachher.get(n)])
        return 1
    print("\nNutzerdaten unveraendert (%d Dateien per SHA-256)." % len(vorher))
    if schlecht:
        print("Nur auf rustgame gescheitert: %s" % ", ".join(schlecht))
        for name in schlecht:
            text = open(os.path.join(AUSGABE, "%s.rustgame.txt" % name), encoding="utf-8").read()
            for z in text.splitlines():
                if z.strip().startswith("FAIL"):
                    print("   %s: %s" % (name, z.strip()[:140]))
        return 1
    print("Jedes Audit, das auf pygame besteht, besteht auch auf rustgame.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
