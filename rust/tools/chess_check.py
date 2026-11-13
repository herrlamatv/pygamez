# -*- coding: utf-8 -*-
"""Schach-Engine Rust gegen Python: gleiche Stellungen und Stufen muessen
denselben Zug, Wert, dieselbe fertige Tiefe und dieselbe Streu-Liste liefern.

    .venv/Scripts/python.exe rust/tools/chess_check.py [anzahl]

Stellungen: Grundstellung, ein paar Mittelspiele und die ersten Raetsel aus
games/levels/chess-puzzles.json. Eine Transpositionstabelle fuer alle Zeilen.
"""
import json
import os
import subprocess
import sys

HIER = os.path.dirname(os.path.abspath(__file__))
WURZEL = os.path.dirname(os.path.dirname(HIER))
sys.path.insert(0, WURZEL)

from games import chess_engine as ce  # noqa: E402


def zeilen(anzahl):
    fens = [ce.START_FEN,
            "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
            ce.chess960_fen(123)]
    with open(os.path.join(WURZEL, "games", "levels", "chess-puzzles.json"), encoding="utf-8") as f:
        data = json.load(f)
    for st in data["stages"]:
        for p in st["puzzles"][:3]:
            fens.append(p["fen"])
    out = []
    stufen = ["2", "3", "4", "1", "h", "5"]
    for i, fen in enumerate(fens[:anzahl]):
        out.append((stufen[i % len(stufen)], fen))
    return out


def main():
    anzahl = int(sys.argv[1]) if len(sys.argv) > 1 else 12
    liste = zeilen(anzahl)
    tt = {}
    py = []
    for lv, fen in liste:
        level = ce.HINT_LEVEL if lv == "h" else ce.LEVELS[int(lv)]
        pos = ce.from_fen(fen)
        s = ce.Search(pos, level, tt=tt, rng=None)
        # Streuung nicht ziehen lassen: die Liste wird verglichen.
        s.spread_keep = True
        gen = s.run()
        try:
            while True:
                next(gen)
        except StopIteration:
            pass
        good = []
        if s.spread and s.root_scores:
            best = max(s.root_scores.values())
            good = [m for m in s.root_moves if s.root_scores.get(m, -ce.INF) >= best - s.spread]
        py.append("%d|%d|%d|%s" % (s.best_move if not s.spread else 0, s.best_score, s.depth_done,
                                   ",".join(str(m) for m in good)))
    exe = os.path.join(os.path.dirname(WURZEL), os.path.basename(WURZEL), "rust.output", "release",
                       "examples", "chess_check.exe")
    eingabe = "\n".join("%s|%s" % z for z in liste) + "\n"
    r = subprocess.run([exe], input=eingabe, capture_output=True, text=True)
    rs = r.stdout.strip().splitlines()
    schlecht = 0
    for i, (a, b) in enumerate(zip(py, rs)):
        lv = liste[i][0]
        if lv in ("1", "2", "3"):
            # Streu-Stufen: best_move haengt am Zufall, verglichen wird der Rest.
            b = "0|" + b.split("|", 1)[1]
        if a != b:
            schlecht += 1
            print("ANDERS %2d stufe %s: py %s / rs %s" % (i, lv, a, b))
    print("Schach-Engine: %d/%d gleich" % (len(py) - schlecht, len(py)))
    return 1 if schlecht or len(rs) != len(py) else 0


if __name__ == "__main__":
    sys.exit(main())
