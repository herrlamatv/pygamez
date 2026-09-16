# -*- coding: utf-8 -*-
"""
build_chess_puzzles.py
======================
Erzeugt die Schachrätsel für den Rätsel-Modus von Schach:

    games/levels/chess-puzzles.json      (Desktop)
    web/js/games/chess_puzzles.js        (Web, PG.chessPuzzles = {...})

Quelle ist die Rätseldatenbank von Lichess (CC0 / gemeinfrei):
https://database.lichess.org/#puzzles  ->  lichess_db_puzzle.csv.zst

Die Datei ist über 300 MB groß. Sie wird deshalb NICHT komplett geladen,
sondern gestreamt und mit ``compression.zstd`` (Python 3.14) Stück für Stück
entpackt; behalten werden nur Zeilen, die in eine der Stufen passen. Sobald
jede Stufe genug Kandidaten hat, bricht der Download ab.

Jedes ausgewählte Rätsel wird mit der eigenen Engine (games/chess_engine.py)
geprüft: alle Züge legal, Matt-Aufgaben enden wirklich mit Matt und das
Matt in N ist erzwungen (jede Verteidigung verliert). Rätsel, die das nicht
bestehen, fliegen raus.

Aufruf aus dem Repo-Root:

    python devtools/build_chess_puzzles.py                 # von lichess.org
    python devtools/build_chess_puzzles.py pfad.csv.zst    # lokale Kopie
"""

import io
import json
import os
import sys
import time
import urllib.request

try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except (AttributeError, OSError):
    pass

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, ROOT)

from compression import zstd  # noqa: E402  (Python 3.14)

import importlib.util  # noqa: E402

# chess_engine direkt laden (ohne games/__init__.py -> kein pygame nötig)
_spec = importlib.util.spec_from_file_location(
    "chess_engine", os.path.join(ROOT, "games", "chess_engine.py"))
ce = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(ce)

URL = "https://database.lichess.org/lichess_db_puzzle.csv.zst"
OUT_JSON = os.path.join(ROOT, "games", "levels", "chess-puzzles.json")
OUT_JS = os.path.join(ROOT, "web", "js", "games", "chess_puzzles.js")

PER_STAGE = 40
CANDIDATES = 6          # so viele Kandidaten je Platz sammeln, dann auswählen

# Taktik-Themen, die im Spiel angezeigt werden (Priorität = Reihenfolge)
THEMES = ("mate", "fork", "pin", "skewer", "discoveredAttack", "doubleCheck",
          "hangingPiece", "trappedPiece", "deflection", "attraction",
          "sacrifice", "promotion")

# id, Rating von/bis, erlaubte Zuglängen (inkl. Gegner-Einleitungszug),
# Pflicht-Thema, verbotene Themen
STAGES = (
    {"id": "mate1", "lo": 400, "hi": 1500, "lens": (2,), "need": "mateIn1"},
    {"id": "mate2", "lo": 800, "hi": 1900, "lens": (4,), "need": "mateIn2"},
    {"id": "mate3", "lo": 1200, "hi": 2300, "lens": (6,), "need": "mateIn3"},
    {"id": "tactic1", "lo": 800, "hi": 1500, "lens": (2, 4), "need": None},
    {"id": "tactic2", "lo": 1500, "hi": 2200, "lens": (4, 6), "need": None},
)
MATE_N = {"mate1": 1, "mate2": 2, "mate3": 3}


def stream_lines(src):
    """Liefert die CSV-Zeilen der (gestreamten) .zst-Datei."""
    if os.path.exists(src):
        raw = open(src, "rb")
    else:
        req = urllib.request.Request(src, headers={"User-Agent": "PyGameZ-build"})
        raw = urllib.request.urlopen(req, timeout=60)
    dec = zstd.ZstdDecompressor()
    buf = b""
    total = 0
    try:
        while True:
            chunk = raw.read(1 << 16)
            if not chunk:
                break
            total += len(chunk)
            data = b""
            # Die Datei kann aus mehreren Frames bestehen (z.B. ein
            # überspringbarer Metadaten-Frame vorneweg): nach jedem Frame-Ende
            # mit einem frischen Entpacker und dem Rest weitermachen.
            while chunk:
                data += dec.decompress(chunk)
                if dec.eof:
                    chunk = dec.unused_data
                    dec = zstd.ZstdDecompressor()
                else:
                    chunk = b""
            buf += data
            *lines, buf = buf.split(b"\n")
            for ln in lines:
                yield ln.decode("utf-8"), total
    finally:
        raw.close()
    if buf:
        yield buf.decode("utf-8"), total


def stage_for(row):
    """Passende Stufe für eine CSV-Zeile (oder None)."""
    moves = row["moves"]
    themes = row["themes"]
    rating = row["rating"]
    mate_tag = next((t for t in themes if t.startswith("mateIn")), None)
    for st in STAGES:
        if not (st["lo"] <= rating <= st["hi"]) or len(moves) not in st["lens"]:
            continue
        if st["need"]:
            if st["need"] in themes:
                return st["id"]
        elif mate_tag is None and any(t in themes for t in THEMES[1:]):
            return st["id"]
    return None


def verify(p, stage):
    """Prüft ein Rätsel mit der eigenen Engine. Liefert (ok, grund)."""
    try:
        pos = ce.from_fen(p["fen"])
    except ValueError as e:
        return False, "FEN: %s" % e
    for i, u in enumerate(p["moves"]):
        m = pos.parse_uci(u)
        if not m:
            return False, "illegaler Zug %s (#%d)" % (u, i)
        if i == 0:
            start = pos.copy()
            start.make(m)
            start.stack = []
        pos.make(m)
    n = MATE_N.get(stage)
    if n:
        if pos.status() != "checkmate":
            return False, "endet nicht mit Matt"
        if len(p["moves"]) != 2 * n:
            return False, "falsche Länge"
        if not ce.forced_mate(start, n):
            return False, "Matt in %d nicht erzwungen" % n
        if n > 1 and ce.forced_mate(start, n - 1):
            return False, "kürzeres Matt vorhanden"
    return True, ""


def pick_spread(cands, n):
    """n Rätsel gleichmäßig über das Rating verteilt (leicht -> schwer)."""
    cands = sorted(cands, key=lambda p: (p["rating"], p["id"]))
    if len(cands) <= n:
        return cands
    step = (len(cands) - 1) / (n - 1)
    out = []
    used = set()
    for i in range(n):
        j = int(round(i * step))
        while j in used:
            j += 1
        used.add(j)
        out.append(cands[j])
    return out


def main():
    src = sys.argv[1] if len(sys.argv) > 1 else URL
    pools = {st["id"]: [] for st in STAGES}
    want = PER_STAGE * CANDIDATES
    t0 = time.time()
    seen = 0
    for line, nbytes in stream_lines(src):
        if not line or line.startswith("PuzzleId"):
            continue
        seen += 1
        parts = line.split(",")
        if len(parts) < 8:
            continue
        try:
            row = {"id": parts[0], "fen": parts[1], "moves": parts[2].split(),
                   "rating": int(parts[3]), "rd": int(parts[4]),
                   "popularity": int(parts[5]), "plays": int(parts[6]),
                   "themes": parts[7].split()}
        except ValueError:
            continue
        # Nur beliebte, oft gespielte Rätsel mit stabilem Rating
        if row["popularity"] < 88 or row["plays"] < 1500 or row["rd"] > 80:
            continue
        st = stage_for(row)
        if st is None or len(pools[st]) >= want:
            continue
        pools[st].append(row)
        if seen % 20000 == 0 or all(len(v) >= want for v in pools.values()):
            print("  %7d Zeilen, %5.1f MB geladen: %s" % (
                seen, nbytes / 1e6,
                " ".join("%s=%d" % (k, len(v)) for k, v in pools.items())))
        if all(len(v) >= want for v in pools.values()):
            break
    print("Kandidaten gesammelt in %.1f s" % (time.time() - t0))

    stages = []
    for st in STAGES:
        sid = st["id"]
        cands = pick_spread(pools[sid], PER_STAGE * 2)
        chosen = []
        for p in cands:
            ok, why = verify(p, sid)
            if not ok:
                print("  verworfen %s (%s): %s" % (p["id"], sid, why))
                continue
            theme = next((t for t in THEMES if t in p["themes"] or
                          (t == "mate" and any(x.startswith("mateIn") for x in p["themes"]))),
                         "")
            chosen.append({"id": p["id"], "fen": p["fen"], "moves": p["moves"],
                           "rating": p["rating"], "theme": theme})
        chosen = pick_spread(chosen, PER_STAGE)
        if len(chosen) < PER_STAGE:
            print("WARNUNG: Stufe %s hat nur %d Rätsel" % (sid, len(chosen)))
        print("Stufe %-8s %d Rätsel, Rating %d-%d" % (
            sid, len(chosen), chosen[0]["rating"], chosen[-1]["rating"]))
        stages.append({"id": sid, "puzzles": chosen})

    data = {
        "source": "Lichess puzzle database - https://database.lichess.org/#puzzles",
        "license": "CC0 1.0 (public domain)",
        "built": time.strftime("%Y-%m-%d"),
        "stages": stages,
    }
    os.makedirs(os.path.dirname(OUT_JSON), exist_ok=True)
    with open(OUT_JSON, "w", encoding="utf-8", newline="\n") as f:
        # Kompakt, aber lesbar: ein Rätsel pro Zeile
        f.write("{\n")
        for k in ("source", "license", "built"):
            f.write(' %s: %s,\n' % (json.dumps(k), json.dumps(data[k], ensure_ascii=False)))
        f.write(' "stages": [\n')
        for si, st in enumerate(stages):
            f.write('  {"id": %s, "puzzles": [\n' % json.dumps(st["id"]))
            rows = [json.dumps(p, ensure_ascii=False) for p in st["puzzles"]]
            f.write(",\n".join("   " + r for r in rows))
            f.write("\n  ]}%s\n" % ("," if si < len(stages) - 1 else ""))
        f.write(" ]\n}\n")
    with open(OUT_JS, "w", encoding="utf-8", newline="\n") as f:
        f.write("/*\n * chess_puzzles.js - Schachrätsel für den Rätsel-Modus "
                "(erzeugt von devtools/build_chess_puzzles.py)\n"
                " * Quelle: Lichess-Rätseldatenbank, CC0 1.0 - "
                "https://database.lichess.org/#puzzles\n */\n")
        f.write("PG.chessPuzzles = ")
        json.dump(data, f, ensure_ascii=False, separators=(",", ":"))
        f.write(";\n")
    print("geschrieben:", OUT_JSON, OUT_JS)


if __name__ == "__main__":
    main()
