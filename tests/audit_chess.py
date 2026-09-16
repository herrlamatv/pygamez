# -*- coding: utf-8 -*-
"""Headless-Audit für Schach (games/chess.py, chess_engine.py, chess_draw.py).

Geprüft wird:

Regeln    (1) Perft: Grundstellung Tiefe 1-4 (20/400/8902/197281), Kiwipete,
              Positionen 3-6 mit bekannten Werten,
          (2) Chess960: drei bekannte Perft-Stellungen, alle 960 Grundreihen
              gültig und verschieden (Nr. 518 = Normalstellung), Perft
              zufälliger Chess960-Stellungen gegen einen unabhängigen
              Referenz-Zuggenerator (Rochaden inklusive),
          (3) Zug ausführen/zurücknehmen stellt alles wieder her (FEN,
              Zobrist-Schlüssel, Bewertungssummen), der Schlüssel stimmt
              nach jedem Zug mit einer Neuberechnung überein,
Notation  (4) SAN- und UCI-Rundlauf über zufällige Partien, bekannte SAN-Fälle
              (Mehrdeutigkeit, Umwandlung, Rochade, en passant, Matt),
              FEN-Rundlauf, PGN-Aufbau, Eröffnungsbuch nur legale Züge,
Rätsel    (5) 200 Rätsel in 5 Stufen, alle Züge legal, Matt-Aufgaben enden mit
              Matt und das Matt in N ist erzwungen; im Spiel: Lösung zählt,
              jeder andere Mattzug zählt auch, falscher Zug wird
              zurückgenommen, "Lösung zeigen" zählt nicht, Fortschritt in
              mem.json + Erfolg chess_puzzles,
KI        (6) Generator-Suche in Häppchen liefert bei fester Tiefe denselben
              Zug, dieselbe Bewertung und dieselbe Knotenzahl wie am Stück;
              Matt in 1/2 wird gefunden; die KI rechnet im Spiel nie länger
              als ein Frame-Budget,
Partie    (7) Siegzähler je Mensch (Farbwechsel), Rückgängig (2 Halbzüge) und
              Hinweis machen die Partie "unterstützt" (kein Sieg-Punkt, kein
              report_result-Sieg, keine Erfolge), chess_master bei Stufe 6,
              Remis anbieten (KI nimmt nur bei ausgeglichener Stellung an),
              Aufgeben, Schachuhr inkl. Zeitüberschreitung ohne Mattmaterial,
              Stellungswiederholung, Umwandlung, Chess960-Rochade per Klick,
              2 Spieler mit Remis-Dialog und Brett drehen, PGN-Export,
Layout    (8) alle Auflösungen x 14 Sprachen: Screens zeichnen fehlerfrei,
              Beschriftungen passen, Zugliste zeigt bei 480x360 genug Zeilen,
              Auflösungswechsel mitten in der Partie, UI v4.2 und UI v1,
Tempo     (9) Zeichnen bei 1280x960 flott genug für 60 FPS.

Aufruf aus dem Repo-Root:  python tests/audit_chess.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import json
import os
import random
import sys
import time

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except (AttributeError, OSError):
    pass

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, REPO)

import pygame
pygame.init()
pygame.display.set_mode((640, 480))

import store
HERE = os.path.dirname(os.path.abspath(__file__))
MEM = os.path.join(HERE, "_audit-chess-mem.json")
store._PATH = MEM
import settings as settings_mod
settings_mod.save_settings = lambda s: None
import i18n
i18n.init()
import ui
import filepick

from game_base import InputEvent

# Nur das Schach-Paket laden, nicht games/__init__.py mit allen 46 Spielen -
# so bleibt der Audit unabhängig von Baustellen in anderen Spielen.
if "games" not in sys.modules:
    import types
    _pkg = types.ModuleType("games")
    _pkg.__path__ = [os.path.join(REPO, "games")]
    sys.modules["games"] = _pkg
from games import chess as chess_mod
from games import chess_engine as ce

GS = json.loads(json.dumps(settings_mod.DEFAULTS))
FAILS = []
RESOLUTIONS = [(480, 360), (640, 480), (800, 600), (960, 720), (1280, 960)]


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + detail) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


class Recorder:
    def __init__(self):
        self.results = []
        self.events = []


def quiet(game):
    rec = Recorder()
    game.play_sound = lambda name: None
    game.rumble = lambda ms=0: None
    game.ach_event = lambda *a, **k: rec.events.append(a)
    game.report_result = lambda won: rec.results.append(won)
    game.rec = rec
    return game


def make_game(mode="single", w=800, h=600, **opts):
    gs = json.loads(json.dumps(GS))
    gs["chess"].update(opts)
    surf = pygame.Surface((w, h))
    return quiet(chess_mod.ChessGame(surf, w, h, mode=mode, game_settings=gs))


def frames(g, n, dt=1 / 60):
    for _ in range(n):
        if g.game_over:
            break
        g.update(dt)


def key(g, k):
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key=k))


def click(g, pos):
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=pos))
    g.handle_event(InputEvent(InputEvent.MOUSEUP, pos=pos))


def play_san(g, san):
    m = g.pos.parse_san(san)
    assert m, "SAN %s nicht legal in %s" % (san, g.pos.fen())
    g._commit(m, animate=False)
    return m


# ================================================================ Referenz
# Unabhängiger, bewusst schlichter Zuggenerator (8x8-Array, Kopie je Zug)
# zum Gegenprüfen der 0x88-Engine - insbesondere der Chess960-Rochade.
N_OFF = [(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)]
K_OFF = [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)]


def ref_from_pos(pos):
    board = [[None] * 8 for _ in range(8)]
    for sq in ce.SQUARES:
        p = pos.b[sq]
        if p:
            board[sq >> 4][sq & 7] = ("b" if p >> 3 else "w") + ce.LETTERS[p & 7]
    rights = []
    for i, rsq in enumerate(pos.castle):
        if rsq >= 0:
            rights.append(("w" if i < 2 else "b", rsq & 7))
    ep = (pos.ep >> 4, pos.ep & 7) if pos.ep >= 0 else None
    return board, "b" if pos.side else "w", frozenset(rights), ep


def ref_attacked(board, r, c, by):
    d = 1 if by == "w" else -1
    for dc in (-1, 1):
        rr, cc = r - d, c + dc
        if 0 <= rr < 8 and 0 <= cc < 8 and board[rr][cc] == by + "P":
            return True
    for dr, dc in N_OFF:
        rr, cc = r + dr, c + dc
        if 0 <= rr < 8 and 0 <= cc < 8 and board[rr][cc] == by + "N":
            return True
    for dr, dc in K_OFF:
        rr, cc = r + dr, c + dc
        if 0 <= rr < 8 and 0 <= cc < 8 and board[rr][cc] == by + "K":
            return True
    for dirs, kinds in (([(1, 1), (1, -1), (-1, 1), (-1, -1)], "BQ"),
                        ([(1, 0), (-1, 0), (0, 1), (0, -1)], "RQ")):
        for dr, dc in dirs:
            rr, cc = r + dr, c + dc
            while 0 <= rr < 8 and 0 <= cc < 8:
                p = board[rr][cc]
                if p:
                    if p[0] == by and p[1] in kinds:
                        return True
                    break
                rr += dr
                cc += dc
    return False


def ref_king(board, color):
    for r in range(8):
        for c in range(8):
            if board[r][c] == color + "K":
                return r, c


def ref_children(state):
    board, side, rights, ep = state
    opp = "b" if side == "w" else "w"
    out = []

    def push(nb, nrights, nep):
        kr, kc = ref_king(nb, side)
        if not ref_attacked(nb, kr, kc, opp):
            # Rochaderechte: König oder Turm bewegt/geschlagen -> weg
            keep = set()
            for color, f in nrights:
                back = 0 if color == "w" else 7
                if nb[back][f] == color + "R" and ref_king(nb, color)[0] == back \
                        and (color, f) in rights:
                    keep.add((color, f))
            out.append((nb, opp, frozenset(keep), nep))

    home = 0 if side == "w" else 7
    fwd = 1 if side == "w" else -1
    for r in range(8):
        for c in range(8):
            p = board[r][c]
            if not p or p[0] != side:
                continue
            t = p[1]
            if t == "P":
                targets = []
                if 0 <= r + fwd < 8 and board[r + fwd][c] is None:
                    targets.append((r + fwd, c, None))
                    if r == home + fwd and board[r + 2 * fwd][c] is None:
                        targets.append((r + 2 * fwd, c, (r + fwd, c)))
                for dc in (-1, 1):
                    rr, cc = r + fwd, c + dc
                    if 0 <= rr < 8 and 0 <= cc < 8:
                        q = board[rr][cc]
                        if (q and q[0] == opp) or ep == (rr, cc):
                            targets.append((rr, cc, "ep" if not q else None))
                for rr, cc, extra in targets:
                    promos = ["Q", "R", "B", "N"] if rr in (0, 7) else [None]
                    for pr in promos:
                        nb = [row[:] for row in board]
                        nb[r][c] = None
                        nb[rr][cc] = side + (pr or "P")
                        nep = None
                        if extra == "ep":
                            nb[r][cc] = None
                        elif isinstance(extra, tuple):
                            # Nur setzen, wenn ein Gegnerbauer daneben steht
                            for dc in (-1, 1):
                                if 0 <= cc + dc < 8 and nb[rr][cc + dc] == opp + "P":
                                    nep = extra
                        rights2 = {(col, f) for col, f in rights
                                   if not (col == opp and rr == (7 if side == "w" else 0) and f == cc)}
                        push(nb, rights2, nep)
                continue
            if t in "NK":
                steps = [(r + dr, c + dc) for dr, dc in (N_OFF if t == "N" else K_OFF)]
            else:
                dirs = {"B": [(1, 1), (1, -1), (-1, 1), (-1, -1)],
                        "R": [(1, 0), (-1, 0), (0, 1), (0, -1)]}.get(
                    t, [(1, 1), (1, -1), (-1, 1), (-1, -1), (1, 0), (-1, 0), (0, 1), (0, -1)])
                steps = []
                for dr, dc in dirs:
                    rr, cc = r + dr, c + dc
                    while 0 <= rr < 8 and 0 <= cc < 8:
                        steps.append((rr, cc))
                        if board[rr][cc]:
                            break
                        rr += dr
                        cc += dc
            for rr, cc in steps:
                if not (0 <= rr < 8 and 0 <= cc < 8):
                    continue
                q = board[rr][cc]
                if q and q[0] == side:
                    continue
                nb = [row[:] for row in board]
                nb[r][c] = None
                nb[rr][cc] = p
                nrights = set(rights)
                if t == "K":
                    nrights = {x for x in nrights if x[0] != side}
                if t == "R" and r == home:
                    nrights.discard((side, c))
                if q and q[1] == "R":
                    nrights.discard((opp, cc))
                push(nb, nrights, None)
            if t == "K" and r == home and not ref_attacked(board, r, c, opp):
                for color, rf in rights:
                    if color != side or board[home][rf] != side + "R":
                        continue
                    kingside = rf > c
                    kt, rt = (6, 5) if kingside else (2, 3)
                    lo, hi = min(c, kt, rf, rt), max(c, kt, rf, rt)
                    if any(board[home][f] and f not in (c, rf) for f in range(lo, hi + 1)):
                        continue
                    tmp = [row[:] for row in board]
                    tmp[home][c] = None
                    tmp[home][rf] = None
                    step = 1 if kt > c else -1
                    path = list(range(c + step, kt + step, step)) if kt != c else []
                    if any(ref_attacked(tmp, home, f, opp) for f in path):
                        continue
                    tmp[home][kt] = side + "K"
                    tmp[home][rt] = side + "R"
                    push(tmp, {x for x in rights if x[0] != side}, None)
    return out


def ref_perft(state, depth):
    if depth == 0:
        return 1
    kids = ref_children(state)
    if depth == 1:
        return len(kids)
    return sum(ref_perft(k, depth - 1) for k in kids)


# ================================================================ 1-2 Perft
def audit_perft():
    print("\n(1) Perft - bekannte Werte")
    cases = [
        ("Grundstellung", ce.START_FEN, [20, 400, 8902, 197281], False),
        ("Kiwipete", "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
         [48, 2039, 97862], False),
        ("Position 3", "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", [14, 191, 2812, 43238], False),
        ("Position 4", "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
         [6, 264, 9467], False),
        ("Position 5", "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
         [44, 1486, 62379], False),
        ("Position 6", "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
         [46, 2079, 89890], False),
        ("Chess960 A", "bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9",
         [21, 528, 12189, 326672], True),
        ("Chess960 B", "2nnrbkr/p1qppppp/8/1ppb4/6PP/3PP3/PPP2P2/BQNNRBKR w HEhe - 1 9",
         [21, 807, 18002, 667366], True),
        ("Chess960 C", "b1q1rrkb/pppppppp/3nn3/8/P7/1PPP4/4PPPP/BQNNRKRB w GE - 1 9",
         [20, 479, 10471, 273318], True),
    ]
    for name, fen, expected, c960 in cases:
        pos = ce.from_fen(fen, chess960=c960)
        got = [ce.perft(pos, d) for d in range(1, len(expected) + 1)]
        check(got == expected, "%s: Perft 1-%d = %s" % (name, len(expected), expected),
              "bekommen %s" % got)

    print("\n(2) Chess960")
    ranks = [ce.chess960_backrank(n) for n in range(960)]
    check(len(set(ranks)) == 960, "960 verschiedene Grundreihen")
    ok = True
    for r in ranks:
        k, r1, r2 = r.index("K"), r.index("R"), r.rindex("R")
        bs = [i for i, ch in enumerate(r) if ch == "B"]
        if not (r1 < k < r2 and (bs[0] + bs[1]) % 2 == 1 and sorted(r) == sorted("RNBQKBNR")):
            ok = False
    check(ok, "jede Grundreihe: König zwischen den Türmen, Läufer auf beiden Farben")
    check(ranks[518] == "RNBQKBNR", "Nr. 518 ist die Normalstellung")
    rng = random.Random(960)
    mism = []
    castles_seen = 0
    for trial in range(8):
        n = rng.randrange(960)
        pos = ce.from_fen(ce.chess960_fen(n), chess960=True)
        # ein paar zufällige Züge, bevorzugt freiräumend, damit Rochaden möglich werden
        for _ in range(rng.randrange(6, 16)):
            legal = pos.legal_moves()
            if not legal:
                break
            castles = [m for m in legal if m >> 20 == ce.M_CASTLE]
            castles_seen += len(castles)
            pos.make(rng.choice(legal))
        depth = 3
        a = ce.perft(pos, depth)
        b = ref_perft(ref_from_pos(pos), depth)
        if a != b:
            mism.append("%s: %d != %d" % (pos.fen(), a, b))
    check(not mism, "Perft zufälliger Chess960-Stellungen = Referenzgenerator (8 Stellungen, Tiefe 3)",
          "; ".join(mism))
    for fen in ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                "1r2k2r/8/8/8/8/8/8/R3KR2 w KQk - 0 1", "4k3/8/8/8/8/8/8/1RK4R w KQ - 0 1"):
        pos = ce.from_fen(fen)
        a = ce.perft(pos, 3)
        b = ref_perft(ref_from_pos(pos), 3)
        check(a == b, "Referenz stimmt: %s" % fen.split()[0], "%d != %d" % (a, b))
    # Rochade, bei der der König stehen bleibt (König g1, Turm h1)
    pos = ce.from_fen("4k3/8/8/8/8/8/8/6KR w H - 0 1", chess960=True)
    oo = [m for m in pos.legal_moves() if m >> 20 == ce.M_CASTLE]
    ok = len(oo) == 1
    if ok:
        pos.make(oo[0])
        ok = pos.b[ce.sq_parse("g1")] == ce.KING and pos.b[ce.sq_parse("f1")] == ce.ROOK \
            and pos.b[ce.sq_parse("h1")] == 0
    check(ok, "Chess960: O-O mit König auf g1 (König bleibt, Turm h1->f1)")
    pos = ce.from_fen("4k3/8/8/8/8/8/8/1RK5 w B - 0 1", chess960=True)
    ooo = [m for m in pos.legal_moves() if m >> 20 == ce.M_CASTLE]
    ok = len(ooo) == 1 and pos.san(ooo[0]) == "O-O-O"
    if ok:
        pos.make(ooo[0])
        ok = pos.b[ce.sq_parse("c1")] == ce.KING and pos.b[ce.sq_parse("d1")] == ce.ROOK
    check(ok, "Chess960: O-O-O mit König c1/Turm b1 (Tausch über Kreuz)")
    pos = ce.from_fen("4k3/8/8/8/8/8/8/rRK5 w B - 0 1", chess960=True)
    check(not any(m >> 20 == ce.M_CASTLE for m in pos.legal_moves()),
          "Chess960: keine Rochade, wenn danach ein Turm hinter dem Rochadeturm Schach gibt")


# ================================================================ 3 make/unmake
def audit_make_unmake():
    print("\n(3) Zug ausführen / zurücknehmen")
    rng = random.Random(7)
    bad_hash = 0
    bad_restore = 0
    total = 0
    for game in range(30):
        c960 = game % 3 == 0
        pos = ce.from_fen(ce.chess960_fen(rng.randrange(960)) if c960 else ce.START_FEN,
                          chess960=c960)
        start = (pos.fen(), pos.hash, list(pos.mg), list(pos.eg), pos.phase, list(pos.counts))
        n = 0
        for _ in range(120):
            legal = pos.legal_moves()
            if not legal:
                break
            pos.make(rng.choice(legal))
            n += 1
            total += 1
            fresh = ce.from_fen(pos.fen(), chess960=c960)
            if (fresh.hash, fresh.mg, fresh.eg, fresh.phase, fresh.counts, fresh.phash) != \
                    (pos.hash, pos.mg, pos.eg, pos.phase, pos.counts, pos.phash):
                bad_hash += 1
        for _ in range(n):
            pos.unmake()
        if (pos.fen(), pos.hash, pos.mg, pos.eg, pos.phase, pos.counts) != start:
            bad_restore += 1
    check(bad_hash == 0, "Schlüssel/Summen nach %d Zügen = Neuberechnung" % total,
          "%d Abweichungen" % bad_hash)
    check(bad_restore == 0, "30 Zufallspartien vollständig zurückgenommen = Startstellung",
          "%d Abweichungen" % bad_restore)


# ================================================================ 4 Notation
def audit_notation():
    print("\n(4) SAN / UCI / FEN / PGN")
    rng = random.Random(11)
    bad = []
    count = 0
    for game in range(25):
        c960 = game % 2 == 1
        pos = ce.from_fen(ce.chess960_fen(rng.randrange(960)) if c960 else ce.START_FEN,
                          chess960=c960)
        for _ in range(100):
            legal = pos.legal_moves()
            if not legal:
                break
            m = rng.choice(legal)
            san = pos.san(m, legal)
            count += 1
            if pos.parse_san(san) != m:
                bad.append("SAN %s" % san)
            if pos.parse_uci(pos.uci(m)) != m:
                bad.append("UCI %s" % pos.uci(m))
            fen = pos.fen()
            if ce.from_fen(fen, chess960=c960).fen() != fen:
                bad.append("FEN %s" % fen)
            pos.make(m)
    check(not bad, "SAN/UCI/FEN-Rundlauf über %d Züge" % count, ", ".join(bad[:5]))

    def san_of(fen, uci, c960=False):
        p = ce.from_fen(fen, chess960=c960)
        m = p.parse_uci(uci)
        return p.san(m) if m else None

    cases = [
        ("r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 0 1", "f3d4", None),
        ("4k3/8/8/8/8/8/8/1N1NK3 w - - 0 1", "b1c3", "Nbc3"),
        ("4k3/8/8/1N6/8/1N6/8/4K3 w - - 0 1", "b5d4", "N5d4"),
        ("4k3/8/8/1Q6/8/1Q5Q/8/4K3 w - - 0 1", "b3e6", None),
        ("8/4P1k1/8/8/8/8/8/4K3 w - - 0 1", "e7e8q", "e8=Q"),
        ("3r2k1/4P3/8/8/8/8/8/4K3 w - - 0 1", "e7d8n", "exd8=N"),
        ("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1", "e5d6", "exd6"),
        ("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", "e1c1", "O-O-O"),
        ("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", "e1g1", "O-O"),
        ("6k1/5ppp/8/8/8/8/8/3RK3 w - - 0 1", "d1d8", "Rd8#"),
        ("4k3/8/8/8/8/8/8/3QK3 w - - 0 1", "d1d7", "Qd7+"),
    ]
    for fen, uci, want in cases:
        got = san_of(fen, uci)
        if want is None:
            continue
        check(got == want, "SAN %s -> %s" % (uci, want), "bekommen %s" % got)
    # Mehrdeutig über zwei Wege (Rang und Linie gleich) -> volles Feld
    got = san_of("4k3/8/8/1Q6/8/1Q5Q/8/4K3 w - - 0 1", "b3e6")
    check(got == "Qbe6+", "SAN Dame mit Konkurrenz auf derselben Reihe -> Qbe6+", str(got))
    got = san_of("k7/8/1Q6/8/8/1Q5Q/8/4K3 w - - 0 1", "b3e6")
    check(got == "Qb3e6", "SAN Dame mit Linien- und Reihen-Konkurrenz -> Qb3e6", str(got))
    # PGN
    text = ce.pgn_text([("Event", "PyGameZ"), ("White", 'A "B"')],
                       ["e4", "e5", "Nf3", "Nc6", "Bb5"], "1-0")
    check('[White "A \\"B\\""]' in text and "1. e4 e5 2. Nf3 Nc6 3. Bb5 1-0" in text,
          "PGN: Tags maskiert, Zugnummern, Ergebnis")
    text = ce.pgn_text([], ["Nxe5", "Qc8+"], "*", start_side=ce.BLACK, start_full=20)
    check("20... Nxe5 21. Qc8+ *" in text, "PGN: Beginn mit Schwarz (20...)")
    bad_book = []
    for line in ce.BOOK_LINES:
        p = ce.from_fen(ce.START_FEN)
        for u in line.split():
            m = p.parse_uci(u)
            if not m:
                bad_book.append(u)
                break
            p.make(m)
    check(not bad_book, "Eröffnungsbuch: %d Varianten, alle Züge legal" % len(ce.BOOK_LINES),
          ", ".join(bad_book))
    check(set(ce.book_moves([])) >= {"e2e4", "d2d4", "c2c4", "g1f3"},
          "Buch kennt die üblichen ersten Züge")


# ================================================================ 5 Rätsel
def audit_puzzles():
    print("\n(5) Rätsel")
    data = chess_mod.load_puzzles()
    counts = [len(data[s]) for s in chess_mod.STAGES]
    check(counts == [40] * 5, "5 Stufen mit je 40 Rätseln", str(counts))
    ids = [p["id"] for s in chess_mod.STAGES for p in data[s]]
    check(len(set(ids)) == len(ids) == 200, "200 verschiedene Rätsel")
    bad = []
    t0 = time.perf_counter()
    for sid in chess_mod.STAGES:
        n = chess_mod.MATE_N.get(sid, 0)
        for p in data[sid]:
            pos = ce.from_fen(p["fen"])
            ok = True
            for i, u in enumerate(p["moves"]):
                m = pos.parse_uci(u)
                if not m:
                    ok = False
                    bad.append("%s: %s illegal" % (p["id"], u))
                    break
                pos.make(m)
                if i == 0:
                    start = pos.copy()
            if not ok:
                continue
            if n:
                if pos.status() != "checkmate":
                    bad.append("%s endet nicht matt" % p["id"])
                elif len(p["moves"]) != 2 * n:
                    bad.append("%s falsche Länge" % p["id"])
                elif not ce.forced_mate(start, n):
                    bad.append("%s: Matt in %d nicht erzwungen" % (p["id"], n))
    check(not bad, "alle Züge legal, Matt-Aufgaben enden matt, Matt in N erzwungen "
          "(%.1f s)" % (time.perf_counter() - t0), "; ".join(bad[:5]))
    here = os.path.join(REPO, "games", "levels")
    check(os.path.exists(os.path.join(here, "chess-puzzles.README.md")),
          "Quellen-/Lizenzhinweis liegt neben den Rätseln")
    js = os.path.join(REPO, "web", "js", "games", "chess_puzzles.js")
    try:
        raw = open(js, encoding="utf-8").read()
        web = json.loads(raw[raw.index("=") + 1:raw.rindex(";")])
        same = [[q["id"] for q in st["puzzles"]] for st in web["stages"]] == \
            [[q["id"] for q in data[s]] for s in chess_mod.STAGES]
    except (OSError, ValueError):
        same = False
    check(same, "Web-Rätseldatei enthält dieselben Rätsel")

    # --- im Spiel lösen
    if os.path.exists(MEM):
        os.remove(MEM)
    g = make_game("puzzles")
    check(g.state == chess_mod.PUZ_MENU, "Rätsel-Modus startet in der Auswahl")
    check(not g.show_highscore_banner and g.score == 0, "Rätsel: kein Highscore-Banner")

    def solve_line(g, stage, idx):
        g._puz_open(stage, idx)
        frames(g, 60)
        guard = 0
        while g.puz_status not in ("solved",) and guard < 20:
            guard += 1
            if g.puz_status == "play":
                m = g.pos.parse_uci(g.puz_line[g.puz_step])
                g._puz_player_move(m, False)
            frames(g, 50)
        return g.puz_status == "solved"

    ok = all(solve_line(g, s, i) for s in range(5) for i in (0, 17, 39))
    check(ok, "Lösungslinie löst Rätsel aller Stufen (15 Stichproben)")
    check(len(g.puz_solved) == 15, "15 Rätsel als gelöst gemerkt", str(len(g.puz_solved)))
    sec = store.load_section("chess")
    check(len(sec.get("puzzles_solved", [])) == 15, "Fortschritt in mem.json (Section chess)")
    check(("chess_puzzles", 15) in g.rec.events, "Erfolg chess_puzzles mit Anzahl gemeldet")
    # alle 200 über die Linie lösbar (ohne Speichern schneller)
    g._puz_save_progress = lambda: None
    fails = [(s, i) for s in range(5) for i in range(40) if not solve_line(g, s, i)]
    check(not fails, "alle 200 Rätsel im Spiel über die Lösungslinie lösbar", str(fails[:5]))

    # falscher Zug wird zurückgenommen
    g._puz_open(3, 0)
    frames(g, 60)
    fen = g.pos.fen()
    right = g.pos.parse_uci(g.puz_line[g.puz_step])
    wrong = next(m for m in g.pos.legal_moves() if m != right and not ce.gives_mate(g.pos, m))
    g._puz_player_move(wrong, False)
    check(g.puz_status == "wrong", "falscher Zug wird erkannt")
    frames(g, 80)
    check(g.pos.fen() == fen and g.puz_status == "play" and g.puz_feedback[0] == "retry",
          "falscher Zug zurückgenommen, Hinweis 'nochmal'")
    # jeder Mattzug zählt: Stellung mit zwei Mattzügen
    g2 = make_game("puzzles")
    g2._puz_save_progress = lambda: None
    g2.puzzles = {s: list(v) for s, v in g2.puzzles.items()}
    g2.puzzles["mate1"] = [{"id": "test2mates", "fen": "6k1/5ppp/8/8/8/8/1Q6/K2R4 b - - 0 1",
                            "moves": ["g8h8", "d1d8"], "rating": 1000, "theme": "mate"}]
    g2._puz_open(0, 0)
    frames(g2, 60)
    alt = g2.pos.parse_uci("b2b8")
    check(alt and ce.gives_mate(g2.pos, alt) and g2.pos.uci(alt) != g2.puz_line[1],
          "Test-Rätsel hat einen zweiten Mattzug")
    g2._puz_player_move(alt, False)
    check(g2.puz_status == "solved", "anderer Mattzug zählt als gelöst")
    # Lösung zeigen zählt nicht
    g3 = make_game("puzzles")
    pid = g3.puzzles["mate2"][5]["id"]
    g3.puz_solved.discard(pid)
    g3._puz_open(1, 5)
    frames(g3, 30)
    key(g3, "h")
    frames(g3, 400)
    check(g3.puz_status == "shown_done" and pid not in g3.puz_solved and pid in g3.puz_seen,
          "Lösung zeigen: spielt die Linie vor, zählt nicht als gelöst")
    check(g3.pos.status() == "checkmate", "gezeigte Matt-Lösung endet matt")
    key(g3, "Return")
    check(g3.state == chess_mod.PUZ and g3.puz_idx == 6, "Enter springt zum nächsten Rätsel")
    key(g3, "m")
    check(g3.state == chess_mod.PUZ_MENU, "M kehrt zur Auswahl zurück")


# ================================================================ 6 KI
def audit_search():
    print("\n(6) KI-Suche")
    fens = [ce.START_FEN,
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"]
    same = True
    detail = ""
    for fen in fens:
        pos = ce.from_fen(fen)
        full = ce.Search(pos, ce.LEVELS[5], max_depth=4, node_limit=10 ** 9)
        ce.run_to_end(full)
        sliced = ce.Search(pos, ce.LEVELS[5], max_depth=4, node_limit=10 ** 9)
        sliced.check_every = 0
        gen = sliced.run()
        steps = 0
        while True:
            sliced.slice_end = 0.0          # jede Prüfung gibt ab
            try:
                next(gen)
                steps += 1
            except StopIteration:
                break
        if (full.best_move, full.best_score, full.nodes) != \
                (sliced.best_move, sliced.best_score, sliced.nodes) or steps < 50:
            same = False
            detail = "%s: %s/%s" % (fen, (full.best_move, full.best_score, full.nodes),
                                    (sliced.best_move, sliced.best_score, sliced.nodes, steps))
    check(same, "Generator in Häppchen = Suche am Stück (Zug, Wert, Knoten; Tiefe 4)", detail)
    # Knotenlimit ist deterministisch
    pos = ce.from_fen(fens[2])
    a = ce.Search(pos, ce.LEVELS[4], node_limit=5000)
    b = ce.Search(pos, ce.LEVELS[4], node_limit=5000)
    ce.run_to_end(a)
    ce.run_to_end(b)
    check(a.best_move == b.best_move and a.nodes == b.nodes, "Knotenlimit: gleiche Suche = gleicher Zug")
    # Matt finden
    pos = ce.from_fen("6k1/5ppp/8/8/8/8/5PPP/3R2K1 w - - 0 1")
    s = ce.Search(pos, ce.LEVELS[5])
    ce.run_to_end(s)
    check(pos.san(s.best_move) == "Rd8#" and s.best_score >= ce.MATE - 5, "Matt in 1 gefunden")
    data = chess_mod.load_puzzles()
    found = 0
    for p in data["mate2"][:10]:
        pos = ce.from_fen(p["fen"])
        pos.make(pos.parse_uci(p["moves"][0]))
        s = ce.Search(pos, ce.LEVELS[5], node_limit=400000)
        ce.run_to_end(s)
        if s.best_score >= ce.MATE - 10:
            found += 1
    check(found == 10, "Matt in 2 in 10 Rätselstellungen gefunden", "%d/10" % found)
    # Stufe 6 schlägt Stufe 2 (mit festen Knotenlimits, ohne Uhr)
    rng = random.Random(5)
    results = []
    for game in range(2):
        pos = ce.from_fen(ce.START_FEN)
        strong = game % 2
        tt = [{}, {}]
        for ply in range(160):
            st = pos.status()
            if st:
                break
            side = pos.side
            lvl = 5 if side == strong else 1
            level = dict(ce.LEVELS[lvl])
            if lvl == 5:
                level["nodes"] = 6000
            s = ce.Search(pos, level, tt=tt[side], rng=rng)
            ce.run_to_end(s)
            pos.make(s.best_move)
        st = pos.status()
        if st == "checkmate":
            results.append(1 - pos.side == strong)
        else:
            e = pos.evaluate() * (1 if pos.side == strong else -1)
            results.append(e > 300)
    check(all(results), "Stufe 6 gewinnt/steht klar besser gegen Stufe 2", str(results))
    # Rechenzeit pro Frame im echten Spiel
    g = make_game("single", difficulty=5, color="black")
    g._start_play()
    worst = 0.0
    times = []
    moves_done = 0
    for _ in range(3):
        start_len = len(g.moves)
        for _f in range(900):
            t0 = time.perf_counter()
            g.update(1 / 60)
            dt = time.perf_counter() - t0
            times.append(dt)
            worst = max(worst, dt)
            if len(g.moves) > start_len:
                break
        moves_done += len(g.moves) > start_len
        if g.state != chess_mod.PLAY:
            break
        legal = g.legal
        g._commit(legal[len(legal) // 2], animate=False)
    times.sort()
    p95 = times[int(len(times) * 0.95)] if times else 0
    check(moves_done == 3, "KI (Stufe 6) antwortet über update() ohne Thread")
    check(p95 < 0.012, "Rechenzeit je Frame (95%%): %.1f ms (Budget 7 ms)" % (p95 * 1000))
    check(worst < 0.03, "Rechenzeit je Frame (max): %.1f ms" % (worst * 1000))
    # on_exit bricht ab
    g2 = make_game("single", difficulty=5, color="black")
    g2._start_play()
    frames(g2, 20)
    had = g2.ai_job is not None
    g2.on_exit()
    check(had and g2.ai_job is None, "on_exit() bricht die laufende Suche ab")


# ================================================================ 6b Web
def audit_web_parity():
    print("\n(6b) Web-Engine = Python-Engine (Node)")
    import shutil
    import subprocess
    node = shutil.which("node")
    if not node:
        print("  --   node nicht gefunden, Vergleich übersprungen")
        return
    rng = random.Random(21)
    fens = [(ce.START_FEN, False),
            ("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", False),
            ("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", False),
            ("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", False),
            ("bqnb1rkr/pp3ppp/3ppn2/2p5/5P2/P2P4/NPP1P1PP/BQ1BNRKR w HFhf - 2 9", True),
            ("8/8/8/4k3/8/8/8/4KQ2 w - - 0 1", False),
            ("6k1/5ppp/8/8/8/8/5PPP/3R2K1 w - - 0 1", False),
            ("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1", False)]
    for game in range(6):
        c960 = game % 2 == 1
        pos = ce.from_fen(ce.chess960_fen(rng.randrange(960)) if c960 else ce.START_FEN,
                          chess960=c960)
        for _ in range(rng.randrange(10, 60)):
            legal = pos.legal_moves()
            if not legal:
                break
            pos.make(rng.choice(legal))
        fens.append((pos.fen(), c960))
    cases = []
    for fen, c960 in fens:
        pos = ce.from_fen(fen, chess960=c960)
        legal = pos.legal_moves()
        case = {"fen": fen, "c960": c960, "fen_out": pos.fen(),
                "perft": [ce.perft(pos, d) for d in (1, 2)], "eval": pos.evaluate(),
                "sans": [pos.san(m, legal) for m in legal], "ucis": [pos.uci(m) for m in legal],
                "status": pos.status(), "searches": []}
        # (Stufen mit Streuung wählen zufällig unter gleich guten Zügen - hier
        # ohne Streuung vergleichen, die Suche selbst ist dieselbe)
        for level, limit, depth in ((5, 9000, None), (4, 4000, None), (3, 2500, None),
                                    (5, 10 ** 7, 3)):
            lv = dict(ce.LEVELS[level], spread=0)
            srch = ce.Search(pos, lv, node_limit=limit, max_depth=depth)
            ce.run_to_end(srch)
            case["searches"].append({"level": level, "node_limit": limit, "max_depth": depth,
                                     "spread": 0,
                                     "best": pos.uci(srch.best_move) if srch.best_move else "",
                                     "score": srch.best_score, "nodes": srch.nodes,
                                     "depth": srch.depth_done})
        cases.append(case)
    ref = os.path.join(HERE, "_audit-chess-ref.json")
    with open(ref, "w", encoding="utf-8") as f:
        json.dump({"cases": cases}, f)
    tool = os.path.join(REPO, "web", "tools", "chess_check.js")
    try:
        out = subprocess.run([node, tool, ref], capture_output=True, text=True,
                             encoding="utf-8", timeout=300)
        ok = out.returncode == 0
        detail = "\n".join(l for l in out.stdout.splitlines() if "FAIL" in l)[:600] + out.stderr[:300]
    except (OSError, subprocess.SubprocessError) as exc:
        ok, detail = False, str(exc)
    finally:
        if os.path.exists(ref):
            os.remove(ref)
    check(ok, "%d Stellungen: Perft, FEN, Bewertung, SAN/UCI, Status und Suchergebnis "
          "(Zug/Wert/Knoten) identisch" % len(cases), detail)
    out = subprocess.run([node, tool], capture_output=True, text=True, encoding="utf-8", timeout=300)
    check(out.returncode == 0, "Web-Engine: Perft-Referenzwerte (Normal + Chess960)", out.stdout[-300:])


# ================================================================ 7 Partie
def audit_game():
    print("\n(7) Partie-Logik")
    g = make_game("single", difficulty=2, color="white")
    check(g.state == chess_mod.SETUP and g.show_highscore_banner, "Einzelspieler startet im Setup")
    key(g, "Return")
    check(g.state == chess_mod.PLAY, "Enter startet die Partie")

    def force_win(g):
        """Setzt eine Mattstellung, in der der Mensch mattsetzt."""
        human = g.human_color
        fen = "6k1/5ppp/8/8/8/8/5PPP/3R2K1 w - - 0 1" if human == ce.WHITE else \
            "3r2k1/5ppp/8/8/8/8/5PPP/6K1 b - - 0 1"
        g.pos = ce.from_fen(fen)
        g.moves = [0, 0]
        g.sans = ["e4", "e5"]
        g.ucis = ["e2e4", "e7e5"]
        g.caps = [0, 0]
        g._refresh()
        mv = g.pos.parse_san("Rd8#" if human == ce.WHITE else "Rd1#")
        g._commit(mv, animate=False)

    force_win(g)
    check(g.state == chess_mod.OVER and g.score == 1 and g.rec.results == [True],
          "Sieg ohne Hilfe: Punkt + report_result(True)")
    check(("chess_win",) in g.rec.events and ("chess_master",) not in g.rec.events,
          "Erfolg chess_win (kein chess_master auf Stufe 3)")
    key(g, "Return")                                  # Farben wechseln
    check(g.human_color == ce.BLACK and g.state == chess_mod.PLAY,
          "neue Partie: Mensch spielt jetzt Schwarz")
    frames(g, 400)
    force_win(g)
    check(g.score == 2 and g.human_wins == 2, "Siegzähler zählt den Menschen, nicht die Farbe")

    # Rückgängig macht die Partie unterstützt
    g = make_game("single", difficulty=5, color="white")
    g._start_play()
    play_san(g, "e4")
    for _ in range(600):
        g.update(1 / 60)
        if len(g.moves) == 2:
            break
    check(len(g.moves) == 2, "KI antwortet auf 1. e4")
    key(g, "u")
    check(len(g.moves) == 0 and g.assisted and g.pos.fen() == ce.START_FEN,
          "Rückgängig nimmt zwei Halbzüge zurück und markiert 'unterstützt'")
    force_win(g)
    check(g.score == 0 and g.rec.results == [] and not g.rec.events,
          "unterstützter Sieg: kein Punkt, kein report_result, keine Erfolge")
    # Hinweis
    g = make_game("single", difficulty=5, color="white")
    g._start_play()
    key(g, "h")
    check(g.assisted and g.hint_job is not None, "Hinweis startet eine Suche und markiert 'unterstützt'")
    for _ in range(400):
        g.update(1 / 60)
        if g.hint_move:
            break
    check(g.hint_move in g.legal, "Hinweis liefert einen legalen Zug (Pfeil)")
    # chess_master
    g = make_game("single", difficulty=5, color="white")
    g._start_play()
    force_win(g)
    check(("chess_master",) in g.rec.events, "Sieg gegen Stufe 6 ohne Hilfe -> chess_master")
    # Remis anbieten
    g = make_game("single", difficulty=3, color="white")
    g._start_play()
    play_san(g, "e4")
    g.ai_last_score = None
    g.pos = ce.from_fen("4k3/8/8/8/8/8/8/Q3K3 w - - 0 1")    # Mensch (Weiß) mit Dame
    g._refresh()
    key(g, "o")
    check(g.state == chess_mod.OVER and g.result == (None, "agreed"),
          "KI nimmt Remis an, wenn sie schlechter steht")
    g = make_game("single", difficulty=3, color="white")
    g._start_play()
    play_san(g, "e4")
    g.pos = ce.from_fen("q3k3/8/8/8/8/8/8/4K3 w - - 0 1")    # KI (Schwarz) mit Dame
    g._refresh()
    key(g, "o")
    check(g.state == chess_mod.PLAY and g.draw_block > len(g.moves),
          "KI lehnt Remis ab, wenn sie besser steht (Sperre für ein paar Züge)")
    key(g, "o")
    check(g.state == chess_mod.PLAY, "zweites Angebot sofort danach wird nicht gewertet")
    # Aufgeben
    key(g, "x")
    check(g.dialog is not None and g.wants_escape, "Aufgeben fragt nach (ESC schließt)")
    key(g, "Escape")
    check(g.dialog is None and g.state == chess_mod.PLAY, "ESC bricht Aufgeben ab")
    key(g, "x")
    key(g, "Return")
    check(g.state == chess_mod.OVER and g.result == (ce.BLACK, "resign") and g.rec.results == [False],
          "Aufgeben: KI gewinnt, report_result(False)")
    # Schachuhr
    g = make_game("single", difficulty=2, color="white", clock="1+0")
    g._start_play()
    check(g.clock == [60.0, 60.0], "Uhr 1+0 startet mit 1:00")
    frames(g, 120)
    check(g.clock[0] == 60.0, "Uhr läuft erst nach dem ersten Zug")
    play_san(g, "e4")
    g.ai_job = {"move": 0, "wait": 999, "search": None, "gen": None, "spent": 0,
                "wall_cap": 999}
    g._step_ai = lambda dt: None
    frames(g, 60)
    check(59.0 < g.clock[1] < 59.1, "Uhr von Schwarz läuft (%.2f)" % g.clock[1])
    g.clock[1] = 0.05
    frames(g, 10)
    check(g.state == chess_mod.OVER and g.result == (ce.WHITE, "timeout"),
          "Zeitüberschreitung verliert")
    g = make_game("single", difficulty=2, color="black", clock="3+2")
    g._start_play()
    g.pos = ce.from_fen("4k3/8/8/8/8/8/8/4KB2 b - - 0 1")
    g.moves = [0]
    g._refresh()
    g.clock[ce.BLACK] = 0.01
    frames(g, 5)
    check(g.state == chess_mod.OVER and g.result == (None, "timeout_draw"),
          "Zeitüberschreitung gegen König+Läufer = Remis")
    g = make_game("multi", clock="3+2")
    g._start_play()
    play_san(g, "e4")
    play_san(g, "e5")
    check(g.clock[ce.BLACK] == 182.0, "Inkrement +2 s nach dem Zug")
    # Stellungswiederholung
    g = make_game("multi")
    g._start_play()
    for san in ["Nf3", "Nf6", "Ng1", "Ng8", "Nf3", "Nf6", "Ng1", "Ng8"]:
        if g.state == chess_mod.PLAY:
            play_san(g, san)
    check(g.state == chess_mod.OVER and g.result == (None, "threefold"),
          "dreifache Stellungswiederholung = Remis")
    check(not g.show_highscore_banner and g.score == 0, "2 Spieler: keine Highscore-Wertung")
    # Umwandlung über Klicks
    g = make_game("multi")
    g._start_play()
    g.pos = ce.from_fen("8/P5k1/8/8/8/8/6K1/8 w - - 0 1")
    g._refresh()
    a7 = g._sq_rect(ce.sq_parse("a7")).center
    a8 = g._sq_rect(ce.sq_parse("a8")).center
    click(g, a7)
    click(g, a8)
    check(g.promo is not None and len(g.promo) == 4, "Umwandlung öffnet die Figurenwahl")
    rects = g._promo_rects()
    click(g, rects[3].center)
    check(g.pos.b[ce.sq_parse("a8")] == ce.KNIGHT and g.sans[-1] == "a8=N",
          "Klick auf Springer wandelt in Springer um")
    # Chess960-Rochade per Klick auf den Turm
    g = make_game("multi")
    g._start_play()
    g.pos = ce.from_fen("4k3/8/8/8/8/8/8/1RK5 w B - 0 1", chess960=True)
    g.c960_n = 1
    g._refresh()
    click(g, g._sq_rect(ce.sq_parse("c1")).center)
    click(g, g._sq_rect(ce.sq_parse("b1")).center)
    check(g.sans and g.sans[-1] == "O-O-O", "Chess960: König auf Turm klicken rochiert")
    # Ziehen mit der Maus (Drag & Drop)
    g = make_game("multi")
    g._start_play()
    e2 = g._sq_rect(ce.sq_parse("e2")).center
    e4 = g._sq_rect(ce.sq_parse("e4")).center
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=e2))
    g.handle_event(InputEvent(InputEvent.MOUSEMOVE, pos=(e2[0], e2[1] - 30)))
    g.handle_event(InputEvent(InputEvent.MOUSEMOVE, pos=e4))
    g.handle_event(InputEvent(InputEvent.MOUSEUP, pos=e4))
    check(g.sans == ["e4"], "Figur per Drag & Drop ziehen")
    # Tastatur
    g = make_game("multi")
    g._start_play()
    g.cursor = [6, 3]          # d2
    key(g, "Return")
    key(g, "Up")
    key(g, "Up")
    key(g, "Return")
    check(g.sans == ["d4"], "Tastatur: Cursor + Enter zieht")
    # 2 Spieler: Remis-Dialog + Brett drehen
    g = make_game("multi", flip=True)
    g._start_play()
    play_san(g, "e4")
    frames(g, 60)
    check(g.view_black, "2 Spieler mit 'Brett drehen': Schwarz sieht von unten")
    key(g, "o")
    check(g.dialog == ("draw", ce.BLACK), "2 Spieler: Remis-Angebot öffnet Dialog für den Gegner")
    key(g, "n")
    check(g.dialog is None and g.state == chess_mod.PLAY, "Remis abgelehnt -> weiter")
    key(g, "o")
    key(g, "j")
    check(g.result == (None, "agreed"), "Remis angenommen")
    # PGN-Export
    tmp = os.path.join(HERE, "_audit-chess.pgn")
    filepick.available = lambda: False
    filepick.to_downloads = lambda name: tmp
    key(g, "p")
    ok = os.path.exists(tmp)
    text = open(tmp, encoding="utf-8").read() if ok else ""
    check(ok and '[Result "1/2-1/2"]' in text and "1. e4 1/2-1/2" in text,
          "PGN-Export nach Partieende (Taste P)")
    if ok:
        os.remove(tmp)
    g = make_game("single", chess960=True)
    g._start_play()
    play_san(g, g.pos.san(g.legal[0]))
    g._finish(None, "agreed")
    pgn = g._pgn()
    check('[Variant "Chess960"]' in pgn and '[FEN "' in pgn and '[SetUp "1"]' in pgn,
          "PGN einer Chess960-Partie enthält Variant/SetUp/FEN")
    check(g.c960_n is not None and 0 <= g.c960_n < 960, "Chess960-Nummer wird gemerkt")
    # Setup speichert Einstellungen
    g = make_game("single")
    saved = []
    g._save_setting = lambda k, v: saved.append((k, v))
    rid_rows = {rid: rects for rid, rects in g.setup_rows}
    click(g, rid_rows["clock"][2].center)
    click(g, rid_rows["c960"][1].center)
    click(g, rid_rows["diff"][5].center)
    check(("clock", "3+2") in saved and ("chess960", True) in saved and ("difficulty", 5) in saved,
          "Setup-Klicks speichern Uhr, Chess960 und Stärke")
    g = make_game("multi")
    rows = [rid for rid, _ in g.setup_rows]
    check(rows == ["clock", "c960", "flip"], "Setup 2 Spieler: Uhr, Chess960, Brett drehen")
    check(chess_mod.ChessGame.MODES == [("single", "chess.mode.single"),
                                        ("multi", "chess.mode.multi"),
                                        ("puzzles", "chess.mode.puzzles")],
          "MODES: Partie / 2 Spieler / Rätsel")


# ================================================================ 8 Layout
def fits(fnt, text, width):
    return fnt.size(text)[0] <= width


def audit_layout():
    print("\n(8) Layout: 5 Auflösungen x 14 Sprachen, UI v4.2 + v1")
    t = i18n.t
    langs = [c for c, _ in i18n.AVAILABLE]
    problems = []
    crashes = []
    for theme in ("v42", "v1"):
        ui.set_theme(theme)
        for code in langs:
            i18n.set_language(code, persist=False)
            for (w, h) in RESOLUTIONS:
                if theme == "v1" and code not in ("de", "fi", "pl", "hr", "en"):
                    continue
                tag = "%s/%s/%dx%d" % (theme, code, w, h)
                try:
                    for mode in ("single", "multi"):
                        g = make_game(mode, w, h, clock="3+2")
                        g.draw()
                        scr = pygame.Rect(0, 0, w, h)
                        for rid, rects in g.setup_rows:
                            for rc in rects:
                                if not scr.contains(rc):
                                    problems.append("%s setup %s außerhalb" % (tag, rid))
                        if g.start_rect.bottom > h - 30:
                            problems.append("%s Start-Knopf über der Fußzeile" % tag)
                        tb = g._title_bottom(int(h * 0.09))
                        first = g.setup_rows[0][1][0].y - g._tiny.get_height() - 3
                        if first < tb - 2:
                            problems.append("%s Setup-Zeile unter dem Untertitel" % tag)
                        labels = {"diff": t("chess.lbl.diff", name=t("chess.diff.lvl3")),
                                  "color": t("chess.lbl.color"), "clock": t("chess.lbl.clock"),
                                  "c960": t("chess.lbl.c960"), "flip": t("chess.lbl.flip")}
                        for rid, rects in g.setup_rows:
                            width = rects[-1].right - rects[0].x
                            if not fits(g._tiny, labels[rid], width):
                                problems.append("%s Label %s zu breit" % (tag, rid))
                            for i, rc in enumerate(rects):
                                txt = {"color": [t("chess.white"), t("chess.black")],
                                       "clock": [t("chess.clock.none"), "1+0", "3+2", "5+0", "10+5"],
                                       "c960": [t("common.off"), t("common.on")],
                                       "flip": [t("common.off"), t("common.on")]}.get(
                                    rid, [str(n + 1) for n in range(6)])[i]
                                if not fits(g._tiny, txt, rc.w - 6):
                                    problems.append("%s Knopf %s/%s zu schmal" % (tag, rid, txt))
                        hint = t("chess.setup_hint" if mode == "single" else "chess.setup_hint_multi")
                        if not fits(g._tiny, hint, w - 20):
                            problems.append("%s Setup-Hinweis zu breit" % tag)
                        g._start_play()
                        for san in ["e4", "e5", "Nf3", "Nc6", "Bb5", "a6"]:
                            play_san(g, san)
                        g.draw()
                        if mode == "single" and g._list_visible() < 7:
                            problems.append("%s Zugliste nur %d Zeilen" % (tag, g._list_visible()))
                        for side in (g.top_panel, g.bot_panel, g.list_rect, g.status_rect):
                            if not scr.contains(side) or side.colliderect(g.plate):
                                problems.append("%s Seitenleiste überlappt" % tag)
                        for bid, rc in g.btn_rects.items():
                            if rc.colliderect(g.list_rect) or rc.colliderect(g.bot_panel):
                                problems.append("%s Knopf %s überlappt" % (tag, bid))
                        g.hover = "draw"
                        g.draw()
                        for txt in (t("chess.your_turn"), t("chess.check"),
                                    t("chess.turn", name=t("chess.black"))):
                            if not fits(g._tiny, txt, g.status_rect.w - 8):
                                problems.append("%s Status zu breit: %s" % (tag, txt))
                        key(g, "x")
                        g.draw()
                        _, yes, no = g._dialog_rects()
                        for label in (t("chess.yes"), t("chess.no"), t("chess.accept"),
                                      t("chess.decline")):
                            if not fits(g._tiny, label, yes.w - 8):
                                problems.append("%s Dialogknopf zu schmal: %s" % (tag, label))
                        key(g, "Return")
                        g.over_t0 = -10
                        g.draw()
                        for bid, rc in g.over_btns.items():
                            label = t("chess.btn." + bid)
                            if not fits(g._tiny, label, rc.w - 8):
                                problems.append("%s Ergebnis-Knopf %s zu schmal" % (tag, bid))
                            if not g.over_rect.contains(rc):
                                problems.append("%s Ergebnis-Knopf außerhalb" % tag)
                        head, _, sub = g._result_texts()
                        if not fits(g._small, head, g.over_rect.w - 20):
                            problems.append("%s Ergebnis-Titel zu breit" % tag)
                        for reason in ("checkmate", "stalemate", "fifty", "threefold", "material",
                                       "resign", "agreed", "timeout", "timeout_draw"):
                            txt = t("chess.reason." + reason, color=t("chess.white"))
                            lines = g._wrap(g._tiny, txt, g.over_rect.w - 20)
                            if len(lines) > 2 or not all(fits(g._tiny, ln, g.over_rect.w - 20)
                                                         for ln in lines):
                                problems.append("%s Grund %s zu breit" % (tag, reason))
                        # Auflösungswechsel mitten in der Partie
                        g2 = make_game("single", w, h)
                        g2._start_play()
                        play_san(g2, "d4")
                        nw, nh = RESOLUTIONS[(RESOLUTIONS.index((w, h)) + 2) % 5]
                        g2.surface = pygame.Surface((nw, nh))
                        g2.width, g2.height = nw, nh
                        g2.on_surface_changed()
                        g2.draw()
                        if not pygame.Rect(0, 0, nw, nh).contains(g2.plate):
                            problems.append("%s Auflösungswechsel: Brett außerhalb" % tag)
                    # Rätsel
                    g = make_game("puzzles", w, h)
                    g.draw()
                    scr = pygame.Rect(0, 0, w, h)
                    for i, rc in enumerate(g.stage_rects):
                        name = t("chess.stage." + chess_mod.STAGES[i])
                        if not fits(g._tiny, name, rc.w - 6):
                            problems.append("%s Stufe %s zu breit" % (tag, name))
                    if g.stage_rects[0].y < g._title_bottom(int(h * 0.075)) - 2:
                        problems.append("%s Stufen-Reiter unter dem Untertitel" % tag)
                    if g.puz_start_rect.bottom > h - 30 or not scr.contains(g.grid_rects[-1]):
                        problems.append("%s Rätselraster/Start außerhalb" % tag)
                    if g.grid_rects[-1].bottom > g.puz_start_rect.y:
                        problems.append("%s Raster überlappt Start" % tag)
                    if not fits(g._tiny, t("chess.puz.footer"), w - 20):
                        problems.append("%s Rätsel-Fußzeile zu breit" % tag)
                    g._puz_open(4, 39)
                    frames(g, 60)
                    g.hover = "solution"
                    g.draw()
                    if g.puz_list_rect.h < 3 * g.row_h:
                        problems.append("%s Rätsel-Zugliste zu klein" % tag)
                    for txt in (t("chess.puz.goal_tactic"), t("chess.puz.goal_mate", n=3)):
                        if len(g._wrap(g._tiny, txt, g.puz_info_rect.w - 16)) > 2:
                            problems.append("%s Rätselziel > 2 Zeilen" % tag)
                    for k in ("watch", "your_move", "good", "wrong", "retry", "solved",
                              "showing", "shown"):
                        txt = t("chess.puz.fb." + k, color=t("chess.black"))
                        lines = g._wrap(g._small, txt, g.puz_info_rect.w - 16)
                        if len(lines) > 2 or any(not fits(g._small, ln, g.puz_info_rect.w - 16)
                                                 for ln in lines):
                            problems.append("%s Rückmeldung %s passt nicht" % (tag, k))
                    for bid in ("undo", "hint", "flip", "draw", "resign", "list", "retry",
                                "solution", "next"):
                        if not fits(g._tiny, t("chess.btn." + bid), g.side_rect.w - 16):
                            problems.append("%s Tooltip %s zu breit" % (tag, bid))
                    g._puz_solved()
                    g.draw()
                except Exception as exc:          # pragma: no cover - Bericht
                    import traceback
                    crashes.append("%s: %r" % (tag, exc))
                    traceback.print_exc()
    ui.set_theme("v42")
    i18n.set_language("de", persist=False)
    check(not crashes, "alle Screens zeichnen fehlerfrei", "; ".join(crashes[:3]))
    check(not problems, "Layout: nichts außerhalb, nichts überlappt, alle Texte passen",
          "; ".join(problems[:12]) + (" (+%d)" % (len(problems) - 12) if len(problems) > 12 else ""))
    # Übersetzungen vollständig
    missing = []
    base = json.load(open(os.path.join(REPO, "lang", "de.json"), encoding="utf-8"))
    keys = [k for k in base if k.startswith("chess.") or k.startswith("ach.chess")]
    for code in langs:
        folder = "lang" if code in ("de", "en", "fr", "es", "pt") else os.path.join("lang", "lang.expansion")
        data = json.load(open(os.path.join(REPO, folder, code + ".json"), encoding="utf-8"))
        missing += ["%s:%s" % (code, k) for k in keys if not data.get(k)]
    used = ["chess.mode.puzzles", "chess.puz.fb.solved", "chess.theme.fork", "chess.btn.pgn",
            "ach.chess_puzzles.name", "ach.chess_master.desc", "chess.reason.timeout_draw"]
    check(not missing and all(k in base for k in used),
          "%d Schach-Texte in allen 14 Sprachen" % len(keys), ", ".join(missing[:5]))


# ================================================================ 9 Tempo
def audit_perf():
    print("\n(9) Tempo")
    for theme in ("v42", "v1"):
        ui.set_theme(theme)
        g = make_game("single", 1280, 960, clock="5+0")
        g._start_play()
        for san in ["e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Ba4", "Nf6", "O-O", "Be7"]:
            play_san(g, san)
        g._step_ai = lambda dt: None
        g.hover = "hint"
        for _ in range(10):
            g.draw()
        t0 = time.perf_counter()
        n = 90
        for i in range(n):
            ui._frame_dt = 1 / 60
            g.update(1 / 60)
            if i % 30 == 0:
                g._select(g.pos.parse_san("d4") & 255)
            g.draw()
        ms = (time.perf_counter() - t0) / n * 1000
        check(ms < 12, "UI %s 1280x960: update+draw %.1f ms je Frame" % (theme, ms))
        gp = make_game("puzzles", 1280, 960)
        for _ in range(5):
            gp.draw()
        t0 = time.perf_counter()
        for _ in range(60):
            gp.draw()
        ms = (time.perf_counter() - t0) / 60 * 1000
        check(ms < 12, "UI %s 1280x960: Rätselauswahl %.1f ms je Frame" % (theme, ms))
    ui.set_theme("v42")


def main():
    t0 = time.time()
    audit_perft()
    audit_make_unmake()
    audit_notation()
    audit_puzzles()
    audit_search()
    audit_web_parity()
    audit_game()
    audit_layout()
    audit_perf()
    for p in (MEM, MEM + ".bak"):
        if os.path.exists(p):
            os.remove(p)
    print("\n%d Fehler, %.0f s" % (len(FAILS), time.time() - t0))
    for f in FAILS:
        print("  - " + f)
    sys.exit(1 if FAILS else 0)


if __name__ == "__main__":
    main()
