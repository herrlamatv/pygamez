# -*- coding: utf-8 -*-
"""
matches_check.py
================
Kreuztest der Match-Historie und des Tages-Archivs: dieselben Befehle laufen
ueber ``matches.py`` / ``dailyarchive.py`` / ``seedrand.py`` und ueber
``pgz-core/src/matches.rs`` / ``pgz-engine/src/dailyarchive.rs`` (Beispiel
``matches_io``), jeweils in einem frischen Datenordner. Verglichen werden die
Antworten Zeile fuer Zeile und die geschriebene ``matches.pgzlamamatches``
Byte fuer Byte (nur die Ziffern der Zeitstempel maskiert).

    cargo build -p pgz-engine --release --offline --example matches_io
    .venv/Scripts/python.exe rust/tools/matches_check.py

Abgedeckt (wie ``tests/audit_matches.py`` Teil 1 und ``tests/audit_dailyarchive.py``
Teil 2-4): Aufzeichnen (neueste zuerst), Kappe 250 bei unbegrenzten Summen,
Sieg/Niederlage/Remis (bool kein Ergebnis), Ersetzen, Info bereinigen (nur
int/str/bool, Schluessel bis 8 Zeichen, Text bis 40), Modi, Profile getrennt,
Profil/Spiel loeschen, Rundlauf ueber die Datei, kaputte Eintraege und kaputte
Datei (Sicherung .bak), Unicode; Tages-Archiv: clean_rows (doppelte Tage,
falsche Breite, bool/negative Tage), upsert (sortiert), available, Datums-
texte, Seeds, Datums-Leiste, ``day_index`` wie ``date.fromisoformat``.
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))
sys.path.insert(0, HIER)
from vergleich import maskiert  # noqa: E402
EXE = os.path.join(os.path.abspath(os.environ.get("CARGO_TARGET_DIR") or os.path.join(WURZEL, "rust.output")),
                   "release", "examples", "matches_io.exe")
PY = os.path.join(WURZEL, ".venv", "Scripts", "python.exe")
if not os.path.exists(PY):
    PY = sys.executable

PY_SEITE = r'''
import json, os, sys
sys.path.insert(0, %(wurzel)r)
os.environ["SDL_VIDEODRIVER"] = "dummy"
import store
store._DIR = %(daten)r
store._PATH = os.path.join(store._DIR, "mem.json")
store._LEGACY_HS_PATH = os.path.join(store._DIR, "highscores.json")
import i18n
i18n.init()
i18n.set_language("de", persist=False)
import matches, dailyarchive as da, seedrand

def j(v):
    return json.dumps(v, ensure_ascii=False, separators=(",", ":"))

def opt(w, i):
    v = w[i] if len(w) > i else ""
    return None if v in ("", "-") else v

pfad = store.path_for(matches.FILENAME)
out = []
for zeile in open(%(befehle)r, encoding="utf-8").read().splitlines():
    w = zeile.split(" | ")
    c = w[0].strip()
    if not c:
        continue
    if c == "record":
        res = None if w[4] in ("null", "") else int(w[4])
        info = json.loads(w[7])
        r = matches.record(w[1], w[2], int(w[3]), res, int(w[5]), int(w[6]), info,
                           pid=opt(w, 8), replace=w[9] == "1")
        out.append(j(r))
    elif c == "entries":
        out.append(j(matches.entries(w[1], opt(w, 2))))
    elif c == "recent":
        out.append(j(matches.recent(w[1], int(w[2]))))
    elif c == "totals":
        t = matches.totals(w[1], opt(w, 2))
        out.append(j([t[k] for k in ("n", "w", "l", "d", "best", "sum", "time", "first", "stored")]))
    elif c == "modes":
        out.append(j(matches.modes_of(w[1])))
    elif c == "flush":
        matches.flush(); out.append("ok")
    elif c == "forget":
        matches._doc = None; matches._dirty = False; out.append("ok")
    elif c == "drop":
        matches.drop_profile(w[1]); out.append("ok")
    elif c == "clear":
        matches.clear_game(w[1]); out.append("ok")
    elif c == "write":
        open(pfad, "w", encoding="utf-8", newline="").write(w[1]); out.append("ok")
    elif c == "writebak":
        open(pfad + ".bak", "w", encoding="utf-8", newline="").write(w[1]); out.append("ok")
    elif c == "fmt":
        out.append(matches.format_duration(int(w[1])))
    elif c == "iso":
        try:
            out.append(str(seedrand.day_index(w[1])))
        except ValueError:
            out.append("ValueError")
    elif c == "date_of":
        out.append(seedrand.date_of(int(w[1])))
    elif c == "dates":
        d, today = int(w[1]), int(w[2])
        out.append("%%s %%s %%s %%s %%s" %% (da.date_str(d), da.chip_date(d), da.short_date(d),
                                       da.long_date(d), da.day_label(d, today)))
    elif c == "clean":
        out.append(j(da.clean_rows(json.loads(w[2]), int(w[1]))))
    elif c == "upsert":
        out.append(j(da.upsert(json.loads(w[1]), json.loads(w[2]))))
    elif c == "avail":
        out.append(j(da.available(json.loads(w[1]), int(w[2]))))
    elif c == "rects":
        out.append(j([list(r) for r in da.day_row_rects(int(w[1]), int(w[2]), int(w[3]), int(w[4]))]))
    elif c == "seed":
        out.append(da.seed_hex(w[1], int(w[2])))
    elif c == "file":
        out.append(open(pfad, encoding="utf-8").read() if os.path.exists(pfad) else "-")
    else:
        out.append("? " + c)
sys.stdout.reconfigure(encoding="utf-8")
print("\n".join(out))
'''

GROSS = '{"format":"pygamez.matches","v":1,"profiles":{"p1":{"snake":{"n":"drei","w":-4,"best":2.5,' \
        '"sum":-7,"time":9007199254740999,"first":"2026-11-20 18:02:59 extra","list":[["x"],' \
        '["2026-01-01 10:00",3,"viel",7,-5,999,[1]],["2026-01-02 11:00:33","single",' \
        '99999999999999999,1.0,12,4,{"day":3,"b":true,"s":"' + "x" * 50 + '","f":1.5,"ueberlang9":1}],' \
        '["2026-01-03 12:00","multi",5,true,3,1]]},"crossy":[1,2]},"p9":[],"p2":{"pong":{"n":2,' \
        '"list":[["2026-01-04 09:00","single",3,0,40,0,null]]}}}}'

BEFEHLE = [
    # Daten (matches.py)
    "record | snake | single | 10 | null | 30 | 0 | null | - | 0",
    "record | snake | single | 50 | null | 12 | 1 | null | - | 0",
    "record | snake | single | 20 | null | 7 | 0 | null | - | 0",
    "entries | snake",
    "totals | snake",
    "record | chess | single | 0 | 1 | 100 | 0 | null | - | 0",
    "record | chess | single | 0 | 0 | 100 | 0 | null | - | 0",
    "record | chess | single | 0 | 2 | 100 | 0 | null | - | 0",
    "record | chess | multi | -3 | 7 | -100 | 300 | {\"vs\":\"Lama\"} | - | 0",
    "totals | chess",
    "record | game2048 | single | 100 | null | 20 | 0 | null | - | 0",
    "record | game2048 | single | 300 | null | 15 | 0 | null | - | 1",
    "entries | game2048",
    "totals | game2048",
    "record | crossy | daily | 5 | null | 3 | 4 | {\"day\":329,\"c\":2,\"ch\":\"fox\",\"x\":1.5,"
    "\"zu_langer_schluessel\":1,\"ok\":true,\"txt\":\"" + "ä" * 45 + "\",\"n\":null,\"l\":[1]} | - | 0",
    "record | crossy | endless | 7 | null | 3 | 0 | {} | - | 0",
    "record | crossy | daily | 9 | null | 4 | 12 | {\"day\":330} | - | 0",
    "modes | crossy",
    "entries | crossy",
    "record | snake | single | 4 | null | 1 | 0 | null | p_other | 0",
    "totals | snake | p_other",
    "flush",
    "file",
    "forget",
    "entries | snake",
    "totals | crossy",
    "drop | p_other",
    "totals | snake | p_other",
    "file",
    "clear | game2048",
    "entries | game2048",
    "fmt | 3725", "fmt | 65", "fmt | 0", "fmt | -5", "fmt | 359999",
    # Kappe
] + ["record | tetris | solo | %d | null | 1 | 0 | null | - | 0" % i for i in range(265)] + [
    "totals | tetris",
    "recent | tetris | 10",
    "record | tetris | solo | 5 | null | 2 | 0 | null | - | 1",
    "totals | tetris",
    "flush",
    "forget",
    "totals | tetris",
    # kaputte Datei / Eintraege / Sicherung
    "write | " + GROSS,
    "forget",
    "entries | snake",
    "totals | snake",
    "totals | crossy",
    "entries | pong | p2",
    "totals | pong | p2",
    "record | pong | single | 1 | null | 2 | 0 | null | p2 | 0",
    "flush",
    "file",
    "write | {kaputt",
    "forget",
    "totals | pong | p2",
    "write | [1,2,3]",
    "writebak | {\"format\":\"falsch\",\"profiles\":{}}",
    "forget",
    "totals | pong | p2",
    "record | snake | single | 77 | null | 6 | 0 | null | - | 1",
    "flush",
    "file",
    # Tages-Archiv (dailyarchive.py)
    "iso | 2026-11-26", "iso | 2026-01-01", "iso | 2025-12-31", "iso | 20261126", "iso | 2026-W48-4",
    "iso | 2026W484", "iso | 2026-W48", "iso | 2026-02-30", "iso | 2026-13-01", "iso | 2026-11-2",
    "iso | ", "iso | None", "iso | 2026-11-26x", "iso | 2026-1126", "iso | 2020-W53-1", "iso | 2021-W53-1",
    "date_of | 0", "date_of | 329", "date_of | -1", "date_of | 365", "date_of | 1000",
    "dates | 329 | 329", "dates | 328 | 329", "dates | 300 | 329", "dates | -40 | 329", "dates | 58 | 329",
    "clean | 3 | [[5,1,1],[3,2,2],[5,9,9],[true,1,1],[-1,1,1],[4,1],\"x\",[2.0,1,1],[7,\"a\",null]]",
    "clean | 4 | {\"a\":1}",
    "clean | 3 | null",
    "upsert | [[1,1,1],[3,3,3]] | [2,2,2]",
    "upsert | [[1,1,1],[3,3,3]] | [3,9,9]",
    "upsert | [[1,1,1],[3,3,3]] | [7,7,7]",
    "upsert | [] | [0,0,0]",
    "avail | [] | 329",
    "avail | [1,2,300,329,330,-3] | 329",
    "avail | [] | 5",
    "rects | 122 | 358 | 396 | 30",
    "rects | 311 | 215 | 310 | 22",
    "rects | 27 | 309 | 284 | 36",
    "rects | 10 | 10 | 120 | 20",
    "seed | crossy | 329", "seed | sudoku | 0", "seed | wordle | 330",
]


def main():
    if not os.path.exists(EXE):
        raise SystemExit("fehlt: %s (cargo build -p pgz-engine --release --offline --example matches_io)" % EXE)
    basis = tempfile.mkdtemp(prefix="pgz_matches_check_")
    befehle = os.path.join(basis, "befehle.txt")
    with open(befehle, "w", encoding="utf-8") as f:
        f.write("\n".join(BEFEHLE) + "\n")
    py_daten, rs_daten = os.path.join(basis, "py"), os.path.join(basis, "rs")
    for d in (py_daten, rs_daten):
        os.makedirs(d)
        with open(os.path.join(d, "mem.json"), "w", encoding="utf-8") as f:
            json.dump({"mem": {"lang": "de"}}, f)
    skript = os.path.join(basis, "py_seite.py")
    with open(skript, "w", encoding="utf-8") as f:
        f.write(PY_SEITE % {"wurzel": WURZEL, "daten": py_daten, "befehle": befehle})
    a = subprocess.run([PY, skript], capture_output=True, text=True, encoding="utf-8")
    if a.returncode != 0:
        raise SystemExit("Python-Seite scheiterte:\n" + a.stderr)
    env = dict(os.environ, PYGAMEZ_DATA_DIR=rs_daten)
    b = subprocess.run([EXE, befehle], capture_output=True, text=True, encoding="utf-8", env=env)
    if b.returncode != 0:
        raise SystemExit("Rust-Seite scheiterte:\n" + b.stderr)
    za, zb = maskiert(a.stdout).splitlines(), maskiert(b.stdout).splitlines()
    schlecht = 0
    for i, (cmd, x, y) in enumerate(zip(BEFEHLE, za, zb)):
        if x != y:
            schlecht += 1
            print("ANDERS %3d  %s\n   py:   %s\n   rust: %s" % (i, cmd[:70], x[:300], y[:300]))
    if len(za) != len(zb):
        schlecht += 1
        print("Zeilenzahl: py %d, rust %d" % (len(za), len(zb)))
    for name in ("matches.pgzlamamatches", "mem.json"):
        pa, pb = os.path.join(py_daten, name), os.path.join(rs_daten, name)
        ta = maskiert(open(pa, "rb").read().decode("utf-8")) if os.path.exists(pa) else None
        tb = maskiert(open(pb, "rb").read().decode("utf-8")) if os.path.exists(pb) else None
        if ta != tb:
            schlecht += 1
            print("Datei %s anders:\n   py:   %s\n   rust: %s" % (name, (ta or "-")[:300], (tb or "-")[:300]))
    print("%d Befehle, %s" % (len(BEFEHLE), "alles gleich" if not schlecht else "%d Abweichungen" % schlecht))
    if not schlecht:
        shutil.rmtree(basis, ignore_errors=True)
    else:
        print("Ordner:", basis)
    sys.exit(1 if schlecht else 0)


if __name__ == "__main__":
    main()
