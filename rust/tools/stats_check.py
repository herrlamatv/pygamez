# -*- coding: utf-8 -*-
"""
stats_check.py
==============
Statistik exportieren/importieren (Fortschritt -> "Exportieren" /
"Importieren") gegen Python pruefen. Im Pruefstand gibt es diese Knoepfe
nicht (sie brauchen Datei-Dialoge), also werden die Kernfunktionen direkt
verglichen: ``stats.export_to`` und ``stats.import_from`` (mit und ohne
Zusammenfuehren, mit und ohne Zielprofil) - je einmal in Python und in Rust
auf denselben Testdaten (``testdaten.py``), danach Datei fuer Datei.

    .venv/Scripts/python.exe rust/tools/stats_check.py

Nur Kratzordner, nie die echten Spielstaende.
"""

import os
import shutil
import subprocess
import sys
import tempfile

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))
sys.path.insert(0, HIER)
from vergleich import _texte_vergleichen  # noqa: E402

# Bauordner wie vergleich.py (CARGO_TARGET_DIR oder rust.output/)
EXE = os.path.join(os.path.abspath(os.environ.get("CARGO_TARGET_DIR") or os.path.join(WURZEL, "rust.output")),
                   "release", "examples", "stats_io.exe")
PY = os.path.join(WURZEL, ".venv", "Scripts", "python.exe")
if not os.path.exists(PY):
    PY = sys.executable

PY_SEITE = r'''
import os, sys
sys.path.insert(0, %(wurzel)r)
import store
store._DIR = %(daten)r
store._PATH = os.path.join(store._DIR, "mem.json")
store._LEGACY_HS_PATH = os.path.join(store._DIR, "highscores.json")
import stats
args = %(args)r
if args[0] == "export":
    print("export", stats.export_to(args[1]))
else:
    print("import", stats.import_from(args[1], None if args[2] == "-" else args[2], args[3] == "1"))
stats.flush()
'''


def python(daten, args):
    code = PY_SEITE % {"wurzel": WURZEL, "daten": daten, "args": args}
    r = subprocess.run([PY, "-c", code], capture_output=True, text=True, cwd=WURZEL)
    return (r.stdout + r.stderr).strip()


def rust(daten, args):
    env = dict(os.environ, PYGAMEZ_DATA_DIR=daten)
    r = subprocess.run([EXE] + args, capture_output=True, text=True, env=env)
    return (r.stdout + r.stderr).strip()


def antwort(text):
    """'import (False, 'format')' (Python) / 'import false "format"' (Rust)
    -> ('import', False, 'format')."""
    import re
    w = re.findall(r"[A-Za-z]+", text)
    if not w:
        return None
    return (w[0], w[1].lower() == "true" if len(w) > 1 else None, w[2] if len(w) > 2 else "")


def ordner_vergleichen(a, b):
    namen = sorted(n for n in os.listdir(a) if os.path.isfile(os.path.join(a, n))
                   and not n.endswith((".bak", ".tmp")))
    namen_b = sorted(n for n in os.listdir(b) if os.path.isfile(os.path.join(b, n))
                     and not n.endswith((".bak", ".tmp")))
    if namen != namen_b:
        return "Dateien py=%s rust=%s" % (namen, namen_b)
    for n in namen:
        f = _texte_vergleichen(n, os.path.join(a, n), os.path.join(b, n))
        if f:
            return f
    return None


def main():
    if not os.path.exists(EXE):
        raise SystemExit("Erst bauen: cargo build -p pgz-core --release --offline --example stats_io")
    tmp = tempfile.mkdtemp(prefix="pgz_stats_")
    try:
        vorlage = os.path.join(tmp, "testdaten")
        subprocess.run([PY, os.path.join(HIER, "testdaten.py"), vorlage], cwd=WURZEL,
                       capture_output=True, text=True, check=True)
        # Export aus Python dient allen Import-Faellen als Quelle.
        quelle = os.path.join(tmp, "quelle.pgzlamastats")
        q = os.path.join(tmp, "q")
        shutil.copytree(vorlage, q)
        python(q, ["export", quelle])
        kaputt = os.path.join(tmp, "kaputt.pgzlamastats")
        with open(kaputt, "w", encoding="utf-8") as f:
            f.write('{"format": "etwas anderes", "profiles": {}}')
        faelle = [
            ("Export", lambda d: ["export", os.path.join(d, "export.pgzlamastats")]),
            ("Import zusammenfuehren", lambda d: ["import", quelle, "-", "1"]),
            ("Import ersetzen", lambda d: ["import", quelle, "-", "0"]),
            ("Import in Profil p2", lambda d: ["import", quelle, "p2", "1"]),
            ("Import in neues Profil p9", lambda d: ["import", quelle, "p9", "0"]),
            ("Import fremdes Format", lambda d: ["import", kaputt, "-", "1"]),
            ("Import fehlende Datei", lambda d: ["import", os.path.join(tmp, "gibtsnicht.json"), "-", "1"]),
        ]
        schlecht = 0
        for name, args in faelle:
            dp, dr = os.path.join(tmp, name + "_py"), os.path.join(tmp, name + "_rs")
            shutil.copytree(vorlage, dp)
            shutil.copytree(vorlage, dr)
            ap = python(dp, [a.replace(dr, dp) for a in args(dp)])
            ar = rust(dr, args(dr))
            fehler = ordner_vergleichen(dp, dr)
            ok = fehler is None and antwort(ap) is not None and antwort(ap) == antwort(ar)
            schlecht += not ok
            print("%-6s %-28s py: %-22s rust: %-22s %s" % ("ok" if ok else "FEHLER", name, ap, ar,
                                                          fehler or ""))
        print("Statistik-Export/Import:", "in Ordnung" if not schlecht else "%d Fehler" % schlecht)
        return 1 if schlecht else 0
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
