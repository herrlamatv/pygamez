# -*- coding: utf-8 -*-
"""Headless-Audit für Tetris (Regelwerk, KI, Oberfläche, Web-Gleichstand).

Geprüft wird:

Regeln    (1) SRS: Kick-Tabellen gegen die Guideline-Tabellen, Wand- und
              Boden-Kicks, T-Spin-Triple- und Fin-Kick (Test 5),
          (2) T-Spin-Erkennung: voll (3-Ecken-Regel und TST-Kick), Mini an der
              Wand, kein Spin nach Verschieben,
          (3) Wertung: Tetris, Back-to-Back x1,5, Combo, Perfect Clear,
          (4) Angriffstabelle, Aufrechnen gegen eingehenden Müll, Müll steigt
              erst nach der Verzögerung und höchstens 8 Zeilen je Stein,
          (5) Lock Delay 0,5 s, 15 Resets, Reset beim Erreichen neuer Tiefe,
          (6) Game Over: Lock Out über dem Feld, Block Out, Top Out durch Müll,
              gleiche Steinfolge für beide Versus-Felder.
Spiel     (7) DAS/ARR: Wiederhol-Events des Systems werden ignoriert, Takt
              stimmt auf die Millisekunde, ARR 0 schiebt sofort an die Wand,
          (8) Sprint/Ultra/Versus füllen keinen Highscore, Bestwerte landen in
              mem.json, Erfolge, Neustart nicht über die Hard-Drop-Tasten,
          (9) KI: räumt auf mittlerer Stärke > 100 Zeilen ohne Top Out, spielt
              im Versus in Echtzeit, Müll wandert zum Gegner.
Optik    (10) Layout aller Auflösungen x 14 Sprachen (Setup, Solo-HUD, Versus,
              Ergebnis) - alles im Bild, nichts überlappt, Texte passen,
         (11) alle Zustände zeichnen fehlerfrei (auch nach Auflösungswechsel),
         (12) Performance bei 1280x960.
Web      (13) web/js/games/tetris_core.js spielt eine Eingabefolge exakt wie
              tetris_core.py (Steinfolge, Kicks, Wertung, Müll) - falls Node da ist.

Aufruf aus dem Repo-Root:  python tests/audit_tetris.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import json
import os
import random
import shutil
import subprocess
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
store._PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                           "_audit-tetris-mem.json")
import settings as settings_mod
SAVED = []
settings_mod.save_settings = lambda s: SAVED.append(json.loads(json.dumps(s)))
import i18n
i18n.init()
import ui

from game_base import InputEvent
from games import tetris as tg
from games import tetris_ai as ai
from games import tetris_core as core

GS = json.loads(json.dumps(settings_mod.DEFAULTS))
FAILS = []
H = core.ROWS
RESOLUTIONS = ((480, 360), (640, 480), (800, 600), (960, 720), (1280, 960))


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + detail) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


def quiet(game):
    game.play_sound = lambda name: None
    game.rumble = lambda ms=0: None
    game.ach_event = lambda *a, **k: None
    game.report_result = lambda won: None
    game._tone = lambda *a, **k: None
    return game


def wipe_mem():
    """Eigene Test-mem.json samt Sicherung (.bak/.tmp) löschen."""
    for suffix in ("", ".bak", ".tmp"):
        try:
            os.remove(store._PATH + suffix)
        except OSError:
            pass


def board_from(rows_bottom_up, seed=1):
    """Feld aus Textzeilen (unterste zuerst, 'X' = belegt)."""
    b = core.Board(seed)
    b.rows = [0] * H
    b.cells = [[None] * core.COLS for _ in range(H)]
    for i, line in enumerate(rows_bottom_up):
        y = H - 1 - i
        for x, ch in enumerate(line):
            if ch == "X":
                b.rows[y] |= 1 << x
                b.cells[y][x] = "G"
    b.events.clear()
    return b


def put(b, kind, rot, x, y):
    """Aktiven Stein direkt setzen (ohne Spawn)."""
    b.kind, b.rot, b.x, b.y = kind, rot, x, y
    b.active = True
    b.last_rot = False
    b.last_kick = -1
    b.lock_timer = 0.0
    b.lock_resets = 0
    b.lowest = y
    b.fall_acc = 0.0
    assert not core.collides(b.rows, kind, rot, x, y), (kind, rot, x, y)


def game(mode="solo", w=640, h=480, variant=None):
    gs = json.loads(json.dumps(GS))
    if variant:
        gs["tetris"]["solo"] = variant
    g = quiet(tg.TetrisGame(pygame.Surface((w, h)), w, h, mode=mode,
                            game_settings=gs))
    return g


def to_play(g):
    g._start()
    while g.state == tg.COUNT:
        g.update(0.05)
    return g


# ---------------------------------------------------------------- (1) SRS

# Unabhängige Abschrift der Guideline-Tabellen (dy nach OBEN positiv).
REF_JLSTZ = {
    "0>R": "(0,0) (-1,0) (-1,+1) (0,-2) (-1,-2)",
    "R>0": "(0,0) (+1,0) (+1,-1) (0,+2) (+1,+2)",
    "R>2": "(0,0) (+1,0) (+1,-1) (0,+2) (+1,+2)",
    "2>R": "(0,0) (-1,0) (-1,+1) (0,-2) (-1,-2)",
    "2>L": "(0,0) (+1,0) (+1,+1) (0,-2) (+1,-2)",
    "L>2": "(0,0) (-1,0) (-1,-1) (0,+2) (-1,+2)",
    "L>0": "(0,0) (-1,0) (-1,-1) (0,+2) (-1,+2)",
    "0>L": "(0,0) (+1,0) (+1,+1) (0,-2) (+1,-2)",
}
REF_I = {
    "0>R": "(0,0) (-2,0) (+1,0) (-2,-1) (+1,+2)",
    "R>0": "(0,0) (+2,0) (-1,0) (+2,+1) (-1,-2)",
    "R>2": "(0,0) (-1,0) (+2,0) (-1,+2) (+2,-1)",
    "2>R": "(0,0) (+1,0) (-2,0) (+1,-2) (-2,+1)",
    "2>L": "(0,0) (+2,0) (-1,0) (+2,+1) (-1,-2)",
    "L>2": "(0,0) (-2,0) (+1,0) (-2,-1) (+1,+2)",
    "L>0": "(0,0) (+1,0) (-2,0) (+1,-2) (-2,+1)",
    "0>L": "(0,0) (-1,0) (+2,0) (-1,+2) (+2,-1)",
}
STATE = {"0": 0, "R": 1, "2": 2, "L": 3}


def _parse(ref):
    out = {}
    for k, v in ref.items():
        a, b = k.split(">")
        out[(STATE[a], STATE[b])] = tuple(
            tuple(int(n) for n in p.strip("()").split(",")) for p in v.split())
    return out


def audit_srs():
    print("\nSRS - Kick-Tabellen und Kicks")
    check(_parse(REF_JLSTZ) == core.KICKS_JLSTZ, "Kick-Tabelle J/L/S/T/Z = Guideline")
    check(_parse(REF_I) == core.KICKS_I, "Kick-Tabelle I = Guideline")
    # Zustände: T zeigt in Zustand R nach rechts, I steht in R in Spalte 2
    t_r = set(core.SHAPES["T"][1].cells)
    check(t_r == {(1, 0), (1, 1), (2, 1), (1, 2)}, "T-Zustand R (Nase rechts)",
          str(sorted(t_r)))
    i_r = set(core.SHAPES["I"][1].cells)
    check(i_r == {(2, 0), (2, 1), (2, 2), (2, 3)}, "I-Zustand R in Spalte 2")
    i_l = set(core.SHAPES["I"][3].cells)
    check(i_l == {(1, 0), (1, 1), (1, 2), (1, 3)}, "I-Zustand L in Spalte 1")
    # Spawn nach Guideline: Zeilen 21/22, links-mittig, sofort eine Zeile tiefer
    b = core.Board(7)
    kinds = b.queue.peek(0)
    b2 = core.Board(7)
    first = b2.kind
    cols = sorted({x for x, _y in b2.cells_of()})
    ok = cols[0] >= 3 and cols[-1] <= 6
    check(ok and b2.y == core.SPAWN_Y + 1, "Spawn mittig über dem Feld (%s)" % first,
          "cols=%s y=%d" % (cols, b2.y))
    del kinds, b

    # Wand-Kick: J in Zustand R an der linken Wand, R->2 braucht Test 2 (+1,0)
    b = board_from([])
    put(b, "J", 1, -1, 30)
    ok = b.rotate(1)
    check(ok and (b.rot, b.x, b.y, b.last_kick) == (2, 0, 30, 1),
          "J an der Wand: R->2 nutzt Kick 2 (+1, 0)",
          "rot=%d x=%d y=%d kick=%d" % (b.rot, b.x, b.y, b.last_kick))
    # Boden-Kick I: liegend am Boden, 0->R braucht Test 5 (+1, +2)
    b = board_from([])
    put(b, "I", 0, 3, H - 2)
    ok = b.rotate(1)
    check(ok and (b.rot, b.x, b.y, b.last_kick) == (1, 4, H - 4, 4),
          "I am Boden: 0->R nutzt Kick 5 (+1, +2)",
          "rot=%d x=%d y=%d kick=%d" % (b.rot, b.x, b.y - H, b.last_kick))
    # I an der rechten Wand: senkrecht (L) in Spalte 9 -> L->0 mit Kick
    b = board_from([])
    put(b, "I", 3, 8, 20)
    ok = b.rotate(1)
    cols = sorted({x for x, _y in b.cells_of()})
    check(ok and b.rot == 0 and cols == [6, 7, 8, 9] and b.last_kick == 2,
          "I an der rechten Wand: L->0 schiebt 2 nach links (Kick 3)",
          "cols=%s kick=%d" % (cols, b.last_kick))
    # O dreht nicht und verrutscht nicht
    b = board_from([])
    put(b, "O", 0, 3, 25)
    before = sorted(b.cells_of())
    b.rotate(1)
    check(sorted(b.cells_of()) == before, "O bleibt beim Drehen stehen")

    # TST-Kick: klassischer T-Spin-Triple-Schacht mit Überhang
    b = board_from(["X.XXXXXXXX", "X..XXXXXXX", "X.XXXXXXXX", "X...XXXXXX",
                    "XX........"])
    put(b, "T", 0, 1, H - 5)
    ok = b.rotate(1)
    check(ok and (b.rot, b.x, b.y, b.last_kick) == (1, 0, H - 3, 4),
          "TST: 0->R nutzt Kick 5 (-1, -2)",
          "rot=%d x=%d y=%d kick=%d" % (b.rot, b.x, b.y - H, b.last_kick))
    info = b.hard_drop()
    check(info and info["lines"] == 3 and info["spin"] == "full"
          and info["attack"] == 6 and info["points"] == 1600,
          "TST räumt 3 Zeilen: T-Spin Triple, 1600 Punkte, 6 Angriff",
          str(info and {k: info[k] for k in ("lines", "spin", "attack", "points")}))
    # gespiegelter TST mit Linksdrehung
    b = board_from(["XXXXXXXX.X", "XXXXXXX..X", "XXXXXXXX.X", "XXXXXX...X",
                    "........XX"])
    put(b, "T", 0, 6, H - 5)
    ok = b.rotate(-1)
    check(ok and (b.rot, b.x, b.y, b.last_kick) == (3, 7, H - 3, 4)
          and b.spin_type() == "full",
          "TST gespiegelt: 0->L nutzt Kick 5 (+1, -2)",
          "rot=%d x=%d y=%d kick=%d" % (b.rot, b.x, b.y - H, b.last_kick))

    # Fin-Kick: T liegt umgedreht (Zustand 2) und dreht 2->R in eine Lage
    # zwei Zeilen tiefer. Alle Zellen außer Start- und Ziel-Lage sind belegt;
    # nach der Abschrift oben passt dann NUR Test 5.
    start = (4, H - 6)
    fin_rot = 1
    ref = _parse(REF_JLSTZ)[(2, 1)]
    tx, ty = start[0] + ref[4][0], start[1] - ref[4][1]
    open_cells = set(b.cells_of("T", 2, *start)) | set(b.cells_of("T", fin_rot, tx, ty))
    b = board_from([])
    for y in range(H - 8, H):
        for x in range(core.COLS):
            if (x, y) not in open_cells:
                b.rows[y] |= 1 << x
    fits = [all(c in open_cells for c in b.cells_of("T", fin_rot, start[0] + dx,
                                                   start[1] - dy))
            for dx, dy in ref]
    put(b, "T", 2, *start)
    ok = b.rotate(-1)
    check(fits == [False, False, False, False, True] and ok
          and (b.x, b.y, b.last_kick) == (tx, ty, 4) and b.spin_type() == "full",
          "Fin: 2->R nutzt Kick 5 (-1, -2) und zählt als voller T-Spin",
          "fits=%s kick=%d" % (fits, b.last_kick))


# ---------------------------------------------------------- (2) T-Spins

def audit_tspin():
    print("\nT-Spin-Erkennung")
    # TSD: T kommt senkrecht (R) und dreht nach unten (R->2 ohne Kick)
    b = board_from(["XXXX.XXXXX", "XXX...XXXX", "...X......"])
    put(b, "T", 1, 3, H - 3)
    ok = b.rotate(1)
    info = b.hard_drop()
    check(ok and info["spin"] == "full" and info["lines"] == 2
          and info["points"] == 1200 and info["attack"] == 4,
          "T-Spin Double: voll (beide vorderen Ecken), 1200 Punkte, 4 Angriff",
          str({k: info[k] for k in ("spin", "lines", "points", "attack")}))
    # Dieselbe Lage durch Verschieben erreicht -> kein Spin
    b = board_from(["XXXX.XXXXX", "XXX...XXXX", "...X......"])
    put(b, "T", 2, 3, H - 3)
    info = b.hard_drop()
    check(info["spin"] is None and info["lines"] == 2 and info["points"] == 300,
          "gleiche Lage ohne Drehung erreicht: normaler Double (300)",
          str({k: info[k] for k in ("spin", "lines", "points")}))
    # Mini an der Wand: 3 Ecken, nur eine vordere, kein TST-Kick
    b = board_from(["..XXXXXXXX"])
    b.rows[H - 1] |= 1 << 1
    put(b, "T", 0, 0, H - 3)
    ok = b.rotate(1)
    mini = b.spin_type()
    info = b.hard_drop()
    check(ok and b.last_kick in (-1, 1) and mini == "mini" and info["spin"] == "mini"
          and info["lines"] == 1 and info["points"] == 200 and info["attack"] == 0,
          "T-Spin Mini Single an der Wand: 200 Punkte, 0 Angriff",
          "spin=%s %s" % (mini, {k: info[k] for k in ("lines", "points", "attack")}))
    # T-Spin ohne Zeilen (voll) -> 400 Punkte, bricht Back-to-Back nicht
    b = board_from(["XXXX.XXXX.", "XXX...XXX.", "...X......"])
    b.b2b = True
    put(b, "T", 1, 3, H - 3)
    b.rotate(1)
    info = b.hard_drop()
    check(info["spin"] == "full" and info["lines"] == 0 and info["points"] == 400
          and b.b2b, "T-Spin ohne Zeile: 400 Punkte, Back-to-Back bleibt",
          str({k: info[k] for k in ("spin", "lines", "points")}))
    # Nach dem Drehen noch gefallen -> kein Spin mehr
    b = board_from(["XXXX.XXXXX", "XXX...XXXX", "...X......"])
    put(b, "T", 1, 3, H - 6)
    b.rotate(1)
    b.tick(3.0)
    info = b.events and b.events[-1][1] if b.events else None
    if info is None or not isinstance(info, dict):
        info = b.hard_drop()
    check(info["spin"] is None, "Drehung in der Luft, danach gefallen: kein Spin")


# ---------------------------------------------------------- (3) Wertung

def audit_scoring():
    print("\nWertung: Tetris, Back-to-Back, Combo, Perfect Clear")
    rows = ["XXXXXXXXX."] * 8 + ["X........."]
    b = board_from(rows)
    put(b, "I", 1, 7, 20)
    a = b.hard_drop()
    check(a["lines"] == 4 and a["points"] == 800 and not a["b2b"] and a["attack"] == 4,
          "Tetris: 800 Punkte, 4 Angriff",
          str({k: a[k] for k in ("lines", "points", "b2b", "attack")}))
    first_drop = a["drop"]
    put(b, "I", 1, 7, 20)
    a = b.hard_drop()
    check(a["b2b"] and a["combo"] == 1 and a["points"] == 1200 + 50
          and a["attack"] == 4 + 1 and not a["pc"],
          "Back-to-Back-Tetris mit Combo 1: 1200 + 50 Punkte, 5 Angriff",
          str({k: a[k] for k in ("b2b", "combo", "points", "attack", "pc")}))
    check(first_drop > 0 and b.score == 800 + 1250 + 2 * (first_drop + a["drop"]),
          "Hard Drop gibt 2 Punkte je Zeile dazu",
          "score=%d drops=%d/%d" % (b.score, first_drop, a["drop"]))
    # Level-Multiplikator
    b = board_from(["XXXXXXXXX."] * 4 + ["X........."])
    b.level = 3
    b.fixed_level = True
    put(b, "I", 1, 7, 20)
    a = b.hard_drop()
    check(a["points"] == 800 * 3, "Punkte mal Level (Level 3: 2400)", str(a["points"]))
    # Perfect Clear mit Tetris
    b = board_from(["XXXXXXXXX."] * 4)
    put(b, "I", 1, 7, 20)
    a = b.hard_drop()
    check(a["pc"] and a["points"] == 800 + 2000 and a["attack"] == 4 + 10,
          "Perfect Clear (Tetris): 2800 Punkte, 14 Angriff",
          str({k: a[k] for k in ("pc", "points", "attack")}))
    # Perfect Clear mit Single
    b = board_from(["XXXXXX...."])
    put(b, "I", 0, 6, H - 2)
    a = b.hard_drop()
    check(a["pc"] and a["lines"] == 1 and a["points"] == 100 + 800,
          "Perfect Clear (Single): 900 Punkte", str(a["points"]))
    # Combo: drei Singles hintereinander (Feld vor jedem Stein neu gelegt)
    b = board_from([])
    combos = []
    for _ in range(3):
        b.rows = [0] * H
        b.rows[H - 1] = 0b0111111111
        put(b, "I", 1, 7, 20)
        a = b.hard_drop()
        combos.append((a["lines"], a["combo"], a["points"]))
    put(b, "O", 0, 0, 20)
    a = b.hard_drop()
    combos.append((a["lines"], b.combo))
    check([c[:2] for c in combos[:3]] == [(1, 0), (1, 1), (1, 2)]
          and combos[1][2] == 100 + 50 and combos[2][2] == 100 + 100
          and combos[3] == (0, -1),
          "Combo zählt Folge-Clears (+50 je Stufe) und reißt ohne Clear ab",
          str(combos))
    # Combo-Tabelle direkt
    check(core.COMBO_ATTACK[:6] == (0, 0, 1, 1, 2, 2) and core.B2B_ATTACK == 1
          and core.PC_ATTACK == 10, "Combo-/B2B-/PC-Angriff nach Guideline")
    b = board_from(["XXXXXXXX.."] * 3)
    b.combo = 1                  # zwei Clears liefen schon
    put(b, "O", 0, 7, H - 3)
    a = b.hard_drop()
    check(a["lines"] == 2 and a["combo"] == 2 and a["attack"] == 1 + 1
          and a["points"] == 300 + 50 * 2,
          "Double in Combo 2: 300 + 100 Punkte, 1 + 1 Angriff",
          str({k: a[k] for k in ("lines", "combo", "attack", "points")}))
    # normaler Clear bricht Back-to-Back
    b = board_from(["XXXXXXXX.."] * 2 + ["X........."])
    b.b2b = True
    put(b, "O", 0, 7, H - 3)
    a = b.hard_drop()
    check(not a["b2b"] and not b.b2b, "normaler Double bricht Back-to-Back")
    # Gravitation nach Guideline
    check(abs(core.gravity_interval(1) - 1.0) < 1e-9
          and abs(core.gravity_interval(2) - 0.793) < 1e-9
          and abs(core.gravity_interval(10) - 0.737 ** 9) < 1e-9
          and core.gravity_interval(25) == core.gravity_interval(20),
          "Gravitationstabelle (1,000 s / 0,793 s / ... / 20G ab Level 20)")


# ---------------------------------------------------------- (4) Müll

def audit_garbage():
    print("\nAngriff, Aufrechnen und Müll")
    table = {("", 1): 0, ("", 2): 1, ("", 3): 2, ("", 4): 4, ("full", 1): 2,
             ("full", 2): 4, ("full", 3): 6, ("mini", 1): 0, ("mini", 2): 1}
    check(all(core.ATTACK_TABLE[k] == v for k, v in table.items()),
          "Angriffstabelle (Single 0, Double 1, Triple 2, Tetris 4, TSS 2, "
          "TSD 4, TST 6, Mini 0/1)")
    b = board_from(["XXXXXXXXX."] * 8 + ["X........."])
    b.receive(5)
    put(b, "I", 1, 7, 20)
    a = b.hard_drop()
    check(a["attack"] == 4 and a["cancelled"] == 4 and a["sent"] == 0
          and b.pending_lines() == 1,
          "Tetris gegen 5 eingehende: 4 getilgt, 1 bleibt, nichts gesendet",
          str({k: a[k] for k in ("attack", "cancelled", "sent")}))
    put(b, "I", 1, 7, 20)
    a = b.hard_drop()
    check(a["attack"] == 5 and a["cancelled"] == 1 and a["sent"] == 4
          and b.pending_lines() == 0,
          "Back-to-Back-Tetris: 1 getilgt, 4 gesendet",
          str({k: a[k] for k in ("attack", "cancelled", "sent")}))
    check(not any(ev[0] == "garbage" for ev in b.events),
          "Clear-Züge lassen keinen Müll aufsteigen")

    # Aufsteigen erst nach der Verzögerung
    b = board_from(["XXXXXXXXX."])
    b.receive(3)
    put(b, "O", 0, 0, 30)
    b.hard_drop()
    check(b.garbage_received == 0 and b.pending_lines() == 3,
          "zu frischer Müll wartet noch (Verzögerung 0,5 s)")
    b.tick(0.6)
    put(b, "O", 0, 0, 20)
    b.hard_drop()
    holes = {[x for x in range(10) if not b.rows[y] >> x & 1][0]
             for y in range(H - 3, H)}
    check(b.pending_lines() == 0 and len(holes) == 1
          and all(bin(b.rows[y]).count("1") == 9 for y in range(H - 3, H)),
          "3 Müllzeilen steigen auf, alle mit demselben Loch",
          "holes=%s" % holes)
    check(b.rows[H - 4] == 0b0111111111, "Stapel wurde um 3 Zeilen hochgeschoben")
    # Höchstens 8 Zeilen je Stein
    b = board_from([])
    b.receive(12)
    b.tick(1.0)
    put(b, "O", 0, 0, 20)
    b.hard_drop()
    check(b.garbage_received == 8 and b.pending_lines() == 4,
          "höchstens 8 Müllzeilen je Stein", str(b.garbage_received))
    # gleiche Steinfolge und Lochfolge für beide Felder
    b1, b2 = core.Board(4242), core.Board(4242)
    check(b1.queue.peek(21) == b2.queue.peek(21) and b1.kind == b2.kind,
          "Versus: beide Felder bekommen dieselbe Steinfolge")
    bag = core.PieceQueue(99).peek(14)
    check(sorted(bag[:7]) == sorted(core.KINDS) and sorted(bag[7:]) == sorted(core.KINDS),
          "7-Bag: jede Siebenergruppe enthält jeden Stein genau einmal")


# ---------------------------------------------------------- (5) Lock Delay

def audit_lock_delay():
    print("\nLock Delay und Resets")
    b = board_from([])
    put(b, "T", 0, 3, H - 2)
    b.tick(0.45)
    check(b.active and b.pieces == 0, "0,45 s am Boden: noch nicht eingerastet")
    b.tick(0.06)
    check(b.pieces == 1, "nach 0,5 s eingerastet")

    b = board_from([])
    put(b, "T", 0, 3, H - 2)
    alive = True
    for k in range(15):
        b.tick(0.4)
        if b.pieces:
            alive = False
            break
        b.move(1 if k % 2 == 0 else -1)
    check(alive and b.lock_resets == 15, "15 Resets verlängern das Lock Delay",
          "resets=%d pieces=%d" % (b.lock_resets, b.pieces))
    b.move(1)
    b.tick(0.001)
    check(b.pieces == 1, "16. Bewegung am Boden: rastet sofort ein")

    # neue Tiefe setzt den Zähler zurück
    b = board_from(["XXXX.XXXXX"])
    put(b, "I", 1, 2, H - 5)
    for _ in range(10):
        b.move(-1)
        b.move(1)
    before = b.lock_resets
    b.move(-1)
    b.move(-1)      # über das Loch: fällt eine Zeile tiefer
    b.tick(1.2)
    check(before > 0 and (b.pieces == 1 or b.lock_resets <= 2),
          "Fall auf neue Tiefe setzt die Resets zurück",
          "before=%d after=%d" % (before, b.lock_resets))
    # Soft Drop: 20-fache Geschwindigkeit, 1 Punkt je Zeile
    b = board_from([])
    y0 = b.y
    b.tick(0.5, soft=True)
    check(b.y - y0 >= 9 and b.score == b.y - y0, "Soft Drop fällt schnell, 1 Punkt je Zeile",
          "zeilen=%d score=%d" % (b.y - y0, b.score))


# ---------------------------------------------------------- (6) Game Over

def audit_game_over():
    print("\nGame Over: Lock Out, Block Out, Top Out")
    # Lock Out: Stein rastet komplett über dem sichtbaren Feld ein
    b = board_from(["XXXXXXXXX."] * 20)
    for y in range(core.BUFFER, H):
        b.rows[y] = 0b1111111111 & ~(1 << 9) if y > core.BUFFER else 0b1111111110
    put(b, "O", 0, 0, core.BUFFER - 2)
    b.hard_drop()
    check(b.dead and b.dead_reason == "lockout", "Lock Out über Zeile 20 -> Game Over",
          str(b.dead_reason))
    # teilweise sichtbar ist KEIN Lock Out
    b = board_from([])
    for y in range(core.BUFFER + 1, H):
        b.rows[y] = 0b0111111111
    put(b, "I", 1, 7, core.BUFFER - 3)
    info = b.hard_drop()
    check(not b.dead or b.dead_reason != "lockout",
          "Stein ragt nur teilweise über das Feld: kein Lock Out", str(b.dead_reason))
    del info
    # Block Out
    b = board_from([])
    for y in range(core.SPAWN_Y, core.SPAWN_Y + 3):
        b.rows[y] = 0b0001111000
    b.spawn()
    check(b.dead and b.dead_reason == "blockout", "Spawn belegt -> Block Out")
    # Top Out durch Müll
    b = board_from([])
    b.rows[0] = 1
    b.receive(1)
    b.tick(1.0)
    put(b, "O", 0, 6, 30)
    b.hard_drop()
    check(b.dead and b.dead_reason == "topout", "Müll schiebt Blöcke aus dem Puffer -> Top Out")
    # Der alte Fehler: Stein oberhalb des Feldes wurde still "weggesperrt"
    g = to_play(game("solo", variant="marathon"))
    bd = g.boards[0]
    for y in range(core.BUFFER, H):
        bd.rows[y] = 0b1111111110
        bd.cells[y] = ["G"] * 9 + [None]
    put(bd, "O", 0, 3, core.BUFFER - 2)
    bd.hard_drop()
    g.update(1 / 60)
    check(g.state == tg.FINISH, "Spiel endet nach Lock Out (Status %s)" % g.state)


# ---------------------------------------------------------- (7) DAS / ARR

def audit_das():
    print("\nDAS / ARR")
    g = to_play(game("solo", variant="sprint"))
    g.das, g.arr = 170, 50
    b = g.boards[0]
    put(b, "T", 0, 3, 22)
    b.level = 1
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="Right"))
    x_after_press = b.x
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="Right", repeat=True))
    check(x_after_press == 4 and b.x == 4,
          "Druck schiebt sofort 1, System-Wiederholung wird ignoriert", str(b.x))
    xs = {}
    for ms in range(5, 321, 5):
        g.update(0.005)
        xs[ms] = b.x
    ok = (xs[165] == 4 and xs[170] == 5 and xs[215] == 5 and xs[220] == 6
          and xs[270] == 7 and xs[320] == 7)
    check(ok, "DAS 170 ms, danach alle 50 ms ein Feld (bis zur Wand)",
          "165:%d 170:%d 220:%d 270:%d" % (xs[165], xs[170], xs[220], xs[270]))
    g.handle_event(InputEvent(InputEvent.KEYUP, key="Right"))
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="Left"))
    x0 = b.x
    for _ in range(10):
        g.update(0.01)
    check(x0 == 6 and b.x == 6, "Loslassen stoppt, Gegenrichtung schiebt sofort 1",
          "%d -> %d" % (x0, b.x))
    g.handle_event(InputEvent(InputEvent.KEYUP, key="Left"))

    # ARR 0 = sofort an die Wand, mehrere Schritte in einem Frame
    g.arr = 0
    put(b, "T", 0, 3, 22)
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="a"))
    g.update(0.16)
    before = b.x
    g.update(0.02)
    check(before == 2 and b.x == 0, "ARR 0: nach DAS in einem Frame an der Wand",
          "%d -> %d" % (before, b.x))
    g.handle_event(InputEvent(InputEvent.KEYUP, key="a"))
    # Soft Drop hält, bis die Taste losgelassen wird
    put(b, "T", 0, 3, 22)
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="Down"))
    g.update(0.2)
    y_soft = b.y
    g.handle_event(InputEvent(InputEvent.KEYUP, key="Down"))
    g.update(0.2)
    check(y_soft >= 22 + 5 and b.y - y_soft <= 1, "Soft Drop solange gehalten",
          "%d -> %d" % (y_soft, b.y))
    # Hold und Linksdrehung über feste Tasten (nur wenn frei)
    put(b, "T", 0, 3, 22)
    kind = b.kind
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="c"))
    check(b.hold_kind == kind and b.hold_used, "C legt den Stein in den Hold")
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="Shift_L"))
    check(b.hold_kind == kind, "Hold nur einmal je Stein")
    put(b, "T", 0, 3, 22)
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="z"))
    check(b.rot == 3, "Z dreht nach links")
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="Up"))
    check(b.rot == 0, "Hoch dreht nach rechts")
    g.controls = json.loads(json.dumps(g.controls))
    g.controls["p1"]["up"] = "z"
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="z"))
    check(b.rot == 1, "Z als Aktion belegt: dreht wie die Aktion (rechts)")
    # 2 Spieler: getrennte Tasten
    g2 = to_play(game("multi"))
    b1, b2 = g2.boards
    put(b1, "T", 0, 3, 22)
    put(b2, "T", 0, 3, 22)
    k1, k2 = b1.kind, b2.kind
    g2.handle_event(InputEvent(InputEvent.KEYDOWN, key="q"))
    g2.handle_event(InputEvent(InputEvent.KEYDOWN, key="Control_R"))
    check(b1.hold_kind == k1 and b2.hold_kind is None and b2.rot == 3,
          "2 Spieler: Q = Hold P1, Strg rechts = links drehen P2")
    g2.handle_event(InputEvent(InputEvent.KEYDOWN, key="Shift_R"))
    g2.handle_event(InputEvent(InputEvent.KEYDOWN, key="e"))
    check(b2.hold_kind == k2 and b1.rot == 3,
          "2 Spieler: Shift rechts = Hold P2, E = links drehen P1")


# ---------------------------------------------------------- (8) Modi

def audit_modes():
    print("\nModi: Highscore, Bestwerte, Erfolge, Neustart")
    wipe_mem()
    events = []
    # Sprint
    g = to_play(game("solo", variant="sprint"))
    g.ach_event = lambda eid, v=None: events.append(eid)
    b = g.boards[0]
    check(not g.show_highscore_banner, "Sprint: kein Highscore-Banner")
    g.elapsed = 95.0
    b.lines = 40
    g.update(1 / 60)
    check(g.state == tg.FINISH and g.score == 0 and "tetris_sprint" in events,
          "Sprint 40 in 1:35: fertig, score 0, Erfolg tetris_sprint",
          "state=%s score=%d events=%s" % (g.state, g.score, events))
    mem = store.load_section("tetris")
    check(abs(mem.get("sprint", 0) - 95.0) < 0.1, "Sprint-Bestzeit in mem.json",
          str(mem))
    for _ in range(200):
        if g.game_over:
            break
        g.update(1 / 60)
    check(g.game_over and g.score == 0, "Sprint endet mit Game Over ohne Punkte")
    # keine neue Bestzeit bei langsamerem Lauf, kein Erfolg über 2:00
    events.clear()
    g._start()
    while g.state != tg.PLAY:
        g.update(0.05)
    g.elapsed = 130.0
    g.boards[0].lines = 41
    g.update(1 / 60)
    check("tetris_sprint" not in events and not g.result["new_best"]
          and abs(store.load_section("tetris")["sprint"] - 95.0) < 0.1,
          "langsamer Sprint: Bestzeit bleibt, kein Erfolg")
    # Ultra
    g = to_play(game("solo", variant="ultra"))
    g.boards[0].score = 12345
    g.elapsed = tg.ULTRA_TIME - 0.001
    g.update(0.05)
    check(g.state == tg.FINISH and g.score == 0 and not g.show_highscore_banner
          and store.load_section("tetris").get("ultra") == 12345,
          "Ultra: nach 2:00 Schluss, score 0, Bestwert in mem.json",
          "state=%s score=%d" % (g.state, g.score))
    # Marathon zählt
    g = to_play(game("solo", variant="marathon"))
    g.boards[0].score = 777
    g.update(1 / 60)
    check(g.score == 777 and g.show_highscore_banner,
          "Marathon füllt score und zeigt das Highscore-Banner")
    # Versus: nie Highscore
    for mode in ("versus_ai", "multi"):
        gv = to_play(game(mode))
        gv.boards[0].score = 5000
        gv.update(1 / 60)
        check(gv.score == 0 and not gv.show_highscore_banner,
              "%s: score 0, kein Banner" % mode)

    # Neustart: Enter/Leertaste (= Hard Drop) starten NICHT neu
    g = to_play(game("solo", variant="marathon"))
    g.boards[0]._die("topout")
    for _ in range(200):
        g.update(1 / 60)
        if g.game_over:
            break
    check(g.state == tg.OVER, "Top Out -> Ergebnis-Screen")
    for key in ("Return", "space"):
        g.over_at = 0.0
        g.handle_event(InputEvent(InputEvent.KEYDOWN, key=key))
    check(g.state == tg.OVER, "Enter/Leertaste starten nach Game Over nicht neu")
    g.over_at = time.monotonic()
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="r"))
    check(g.state == tg.OVER, "R während der kurzen Sperre: nichts")
    g.over_at = 0.0
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="r"))
    check(g.state == tg.COUNT and not g.game_over, "R nach der Sperre: neue Runde")
    g.boards[0]._die("topout")
    for _ in range(200):
        g.update(1 / 60)
    g.over_at = 0.0
    g.draw()
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=g.over_rects["setup"].center,
                              button=1))
    check(g.state == tg.SETUP and not g.game_over, "Klick auf Setup öffnet das Setup")

    # Erfolge nur im eigenen Feld und nicht zu zweit
    for mode, want in (("versus_ai", True), ("multi", False)):
        gv = to_play(game(mode))
        got = []
        gv.ach_event = lambda eid, v=None: got.append(eid)
        for i in (0, 1):
            bb = gv.boards[i]
            for y in range(H - 4, H):
                bb.rows[y] = 0b0111111111
                bb.cells[y] = ["G"] * 9 + [None]
            put(bb, "I", 1, 7, 22)
            bb.hard_drop()
        gv.update(1 / 60)
        check(("tetris_four" in got) == want and got.count("tetris_four") <= 1,
              "%s: Erfolg tetris_four %s" % (mode, "nur fürs eigene Feld" if want
                                             else "gibt es zu zweit nicht"), str(got))
    # Versus-Ergebnis zählt Siege und speichert die Bilanz gegen die KI
    gv = to_play(game("versus_ai"))
    res = []
    gv.report_result = lambda won: res.append(won)
    gv.boards[1]._die("topout")
    gv.update(1 / 60)
    check(gv.wins == [1, 0] and res == [True]
          and store.load_section("tetris")["vs"][gv.ai_level][0] >= 1,
          "Versus KI: Sieg gezählt, gemeldet und gespeichert")
    # Setup speichert sofort
    SAVED.clear()
    gs = game("solo")
    gs.state = tg.SETUP
    gs.setup_focus = gs._setup_items().index("das")
    gs.handle_event(InputEvent(InputEvent.KEYDOWN, key="Right"))
    check(SAVED and SAVED[-1]["tetris"]["das"] == gs.das == 180,
          "Setup: DAS +10 ms wird sofort gespeichert")
    kept = settings_mod._merge_defaults({"tetris": {"das": 999, "arr": 0, "ghost": False,
                                                    "solo": "ultra", "ai_level": 2,
                                                    "start_level": 9}})["tetris"]
    check(kept == {"das": 400, "arr": 0, "ghost": False, "solo": "ultra",
                   "ai_level": 2, "start_level": 9},
          "Optionen überleben das Laden von settings.json", str(kept))


# ---------------------------------------------------------- (9) KI

def audit_ai():
    print("\nKI")
    for level, pieces, need in ((1, 400, 100), (0, 300, 60), (2, 300, 60)):
        b = core.Board(20260916 + level)
        rng = random.Random(level)
        t0 = time.time()
        n = 0
        while n < pieces and not b.dead:
            for act in ai.plan(b, level, rng):
                ai.apply(b, act)
            n += 1
        ms = (time.time() - t0) * 1000 / max(1, n)
        check(not b.dead and b.lines > need,
              "Stufe %d: %d Steine, %d Zeilen ohne Top Out (%.2f ms je Planung)"
              % (level, n, b.lines, ms), "dead=%s lines=%d" % (b.dead, b.lines))
    b = core.Board(77)
    rng = random.Random(1)
    for _ in range(300):
        for act in ai.plan(b, 2, rng):
            ai.apply(b, act)
    check(b.tetrises >= 5, "Schwer baut auf Tetris (%d Tetris in 300 Steinen)" % b.tetrises)
    # Echtzeit im Versus: KI setzt Steine, Müll wandert
    g = to_play(game("versus_ai"))
    g.ai_level = 2
    g.ai = tg._AIPlayer(2, 5)
    for _ in range(60 * 20):
        g.update(1 / 60)
        if g.state != tg.PLAY:
            break
    b = g.boards[1]
    check(b.pieces >= 25 and b.lines >= 8,
          "Versus in Echtzeit: schwere KI setzt %d Steine in 20 s" % b.pieces,
          "pieces=%d lines=%d" % (b.pieces, b.lines))
    g = to_play(game("versus_ai"))
    bd = g.boards[0]
    for y in range(H - 4, H):
        bd.rows[y] = 0b0111111111
        bd.cells[y] = ["G"] * 9 + [None]
    bd.rows[H - 5] = 1                  # kein Perfect Clear
    put(bd, "I", 1, 7, 20)
    bd.hard_drop()
    g.update(1 / 60)
    check(g.boards[1].pending_lines() == 4 and g.missiles,
          "Tetris des Spielers schickt 4 Müllzeilen zur KI (mit Flug-Effekt)")


# ---------------------------------------------------------- (10) Layout

def _inside(r, w, h):
    return r.left >= 0 and r.top >= 0 and r.right <= w and r.bottom <= h


def audit_layout():
    print("\nLayout: 5 Auflösungen x 14 Sprachen")
    before = i18n.get_language()
    try:
        for w, h in RESOLUTIONS:
            problems = []
            for mode in ("solo", "versus_ai", "multi"):
                g = game(mode, w, h)
                rects = [r for k, v in g.setup_rects.items()
                         for r in (v if isinstance(v, list) else [v])]
                if not all(_inside(r, w, h - g._tiny.get_height() * 2 - 12)
                           for r in rects):
                    problems.append("%s: Setup-Knopf außerhalb" % mode)
                for i, a in enumerate(rects):
                    for b in rects[i + 1:]:
                        if a.colliderect(b):
                            problems.append("%s: Setup überlappt %s/%s" % (mode, a, b))
                opt = [n for n in ("level", "ghost", "das", "arr") if n in g.setup_rects]
                label_bottom = g.setup_rects[opt[0]].top
                if "choice" in g.setup_rects:
                    if g.setup_desc_y + 2 * g._tiny.get_height() + 6 > \
                            label_bottom - g._tiny.get_height() - 3:
                        problems.append("%s: Beschreibung stößt an Optionen" % mode)
                if g.setup_help_y + g._tiny.get_height() > g.setup_rects["start"].top:
                    problems.append("%s: Hilfezeile stößt an START" % mode)
                if g.setup_rects["start"].bottom > g.setup_footer_y[0]:
                    problems.append("%s: START stößt an die Fußzeile" % mode)
                # Spielfeld-Layout
                to_play(g)
                boxes = []
                for L in g.sides:
                    for key in ("field", "hold", "next", "stats", "bar"):
                        r = L.get(key)
                        if r is None:
                            continue
                        if not _inside(r, w, h):
                            problems.append("%s: %s außerhalb %s" % (mode, key, r))
                        boxes.append((key, r))
                for i, (ka, a) in enumerate(boxes):
                    for kb, b in boxes[i + 1:]:
                        if a.colliderect(b):
                            problems.append("%s: %s überlappt %s" % (mode, ka, kb))
                if g.versus:
                    top = g.sides[0]["field"].y - g._f_value.get_height() - 6
                    bot = g.sides[0]["field"].bottom + 6 + g._f_label.get_height()
                    if top < 0 or bot > h:
                        problems.append("%s: Namen/Statistik außerhalb" % mode)
                    if g.sides[0]["next"].right >= g.sides[1]["hold"].left:
                        problems.append("%s: Seiten berühren sich" % mode)
                c = g.sides[0]["cell"]
                if c < (12 if g.versus else 15):
                    problems.append("%s: Zellen zu klein (%d px)" % (mode, c))
            # Texte in allen Sprachen
            for code, _name in i18n.AVAILABLE:
                i18n.set_language(code, persist=False)
                for mode in ("solo", "versus_ai", "multi"):
                    g = game(mode, w, h)
                    W = w - 20
                    texts = [(g._small, t) for t in (i18n.t("tetris.subtitle." + mode),)]
                    texts += [(g._tiny, i18n.t("tetris.setup_hint")),
                              (g._tiny, i18n.t("tetris.keys_solo")),
                              (g._tiny, i18n.t("tetris.keys_p1")),
                              (g._tiny, i18n.t("tetris.keys_p2"))]
                    for fnt, text in texts:
                        if fnt.size(text)[0] > W:
                            problems.append("%s: zu breit '%s'" % (code, text[:30]))
                    if "choice" in g.setup_rects:
                        chs = g.setup_rects["choice"]
                        full = chs[0].union(chs[-1])
                        names = ([i18n.t("tetris.variant." + v) for v in tg.VARIANTS]
                                 if mode == "solo" else
                                 [i18n.t("tetris.ai.%d" % k) for k in range(3)])
                        for n in names:
                            if g._small_b.size(n)[0] > chs[0].w - 10:
                                problems.append("%s: Knopf '%s'" % (code, n))
                        descs = ([i18n.t("tetris.variant_desc." + v) for v in tg.VARIANTS]
                                 if mode == "solo" else
                                 [i18n.t("tetris.ai_desc.%d" % k) for k in range(3)])
                        descs += [i18n.t("tetris.best_none"),
                                  i18n.t("tetris.vs_record", w=99, l=99),
                                  i18n.t("tetris.best_time", time="1:23.45")]
                        for d in descs:
                            if g._tiny.size(d)[0] > full.w:
                                problems.append("%s: Beschreibung '%s'" % (code, d[:28]))
                    opt = [n for n in ("level", "ghost", "das", "arr") if n in g.setup_rects]
                    full = g.setup_rects[opt[0]].union(g.setup_rects[opt[-1]])
                    for n in opt:
                        r = g.setup_rects[n]
                        if g._tiny.size(i18n.t("tetris.lbl." + n))[0] > r.w + 6:
                            problems.append("%s: Label '%s'" % (code, n))
                        if g._tiny.size(i18n.t("tetris.help." + n))[0] > full.w:
                            problems.append("%s: Hilfe '%s'" % (code, n))
                    if g._small_b.size(i18n.t("common.on"))[0] > g.setup_rects["ghost"].w - 10:
                        problems.append("%s: AN/AUS" % code)
                    if g._small_b.size(i18n.t("common.start"))[0] > g.setup_rects["start"].w - 10:
                        problems.append("%s: START" % code)
                    # HUD-Beschriftungen im Solo
                    if mode == "solo":
                        to_play(g)
                        L = g.sides[0]
                        for key in ("score", "level", "lines", "time", "left", "pps"):
                            if g._f_label.size(i18n.t("tetris.hud." + key))[0] > L["stats"].w - 12:
                                problems.append("%s: HUD '%s'" % (code, key))
                        for key in ("hold", "next"):
                            if g._f_label.size(i18n.t("tetris.hud." + key))[0] > L[key].w - 6:
                                problems.append("%s: HUD '%s'" % (code, key))
                    # Ergebnis-Screen passt
                    for reason in (("sprint",) if mode == "solo" else ("ko",)):
                        gg = to_play(game(mode, w, h, variant="sprint"))
                        if reason == "ko":
                            gg.boards[1]._die("topout")
                        else:
                            gg.boards[0].lines = 40
                            gg.elapsed = 61.0
                        gg.update(1 / 60)
                        lay = gg._results_layout()
                        p = lay["panel"]
                        if not _inside(p, w, h) or lay["again"].colliderect(lay["setup"]):
                            problems.append("%s/%s: Ergebnis-Panel" % (code, mode))
                        for key in ("tetris.res.again", "tetris.res.rematch",
                                    "tetris.res.setup"):
                            if g._small.size(i18n.t(key))[0] > lay["again"].w - 12:
                                problems.append("%s: Knopf '%s'" % (code, key))
                        if g._tiny.size(i18n.t("tetris.res.hint"))[0] > p.w - 20:
                            problems.append("%s: Ergebnis-Hinweis" % code)
                        for key in ("tetris.res.you_win", "tetris.res.ai_wins",
                                    "tetris.res.time_up", "tetris.res.new_best_time"):
                            if gg._big.size(i18n.t(key))[0] > w - 70:
                                problems.append("%s: Titel '%s'" % (code, key))
                i18n.set_language(before, persist=False)
            check(not problems, "%4dx%d: Setup, Felder, HUD und Ergebnis passen" % (w, h),
                  "; ".join(sorted(set(problems))[:8]))
    finally:
        i18n.set_language(before, persist=False)


# ---------------------------------------------------------- (11) Zeichnen

def audit_draw():
    print("\nZeichnen aller Zustände")
    before = i18n.get_language()
    errors = []
    try:
        for theme in ("v42", "v1", "classic"):
            ui.set_theme(theme)
            for code, _n in i18n.AVAILABLE:
                i18n.set_language(code, persist=False)
                for mode in ("solo", "versus_ai", "multi"):
                    try:
                        w, h = (480, 360) if code in ("de", "fi", "pl", "hr") else (800, 600)
                        g = game(mode, w, h)
                        g.update(0.02)
                        g.draw()
                        g._start()
                        g.draw()
                        while g.state == tg.COUNT:
                            g.update(0.05)
                        g.draw()
                        rng = random.Random(1)
                        for slot in range(len(g.boards)):
                            for _ in range(12):
                                for act in ai.plan(g.boards[slot], 2, rng):
                                    ai.apply(g.boards[slot], act)
                                g._process_events(slot)
                        g.boards[-1].receive(3)
                        for _ in range(4):
                            g.update(1 / 60)
                            g.draw()
                        g.width, g.height = 640, 480
                        g.surface = pygame.Surface((640, 480))
                        g.on_surface_changed()
                        g.draw()
                        g.boards[-1]._die("topout")
                        for _ in range(140):
                            g.update(1 / 60)
                        g.draw()
                        g.over_at = 0.0
                        g.draw()
                    except Exception as exc:     # noqa: BLE001 - Testlauf
                        import traceback
                        errors.append("%s/%s/%s: %s" % (theme, code, mode, exc))
                        traceback.print_exc()
    finally:
        ui.set_theme("v42")
        i18n.set_language(before, persist=False)
    check(not errors, "Setup, Countdown, Spiel, Auflösungswechsel, K.O. und Ergebnis "
                      "zeichnen (3 Themes x 14 Sprachen x 3 Modi)",
          "; ".join(errors[:3]))


# ---------------------------------------------------------- (12) Performance

def audit_perf():
    print("\nPerformance")
    ui.set_theme("v42")
    for mode in ("solo", "versus_ai"):
        g = to_play(game(mode, 1280, 960))
        rng = random.Random(2)
        for slot in range(len(g.boards)):
            for _ in range(30):
                for act in ai.plan(g.boards[slot], 1, rng):
                    ai.apply(g.boards[slot], act)
                g._process_events(slot)
        for _ in range(30):
            g.update(1 / 60)
            g.draw()
        t0 = time.perf_counter()
        n = 120
        worst = 0.0
        for _ in range(n):
            f0 = time.perf_counter()
            g.update(1 / 60)
            g.draw()
            worst = max(worst, time.perf_counter() - f0)
        avg = (time.perf_counter() - t0) / n * 1000
        check(avg < 9.0, "%s 1280x960: %.2f ms je Frame (max %.1f ms)" % (mode, avg,
                                                                        worst * 1000))


# ---------------------------------------------------------- (13) Web

def audit_web():
    print("\nWeb: tetris_core.js = tetris_core.py")
    node = shutil.which("node")
    tool = os.path.join(REPO, "web", "tools", "tetris_replay.js")
    if not node or not os.path.isfile(tool):
        check(False, "Node und web/tools/tetris_replay.js vorhanden")
        return
    # Zufällige Eingaben vor jedem Stein, danach setzt die KI ihn ab ("ai") -
    # so überlebt die Partie lange und durchläuft Kicks, Hold, Soft Drop,
    # Lock Delay, Müll und Wertung.
    rng = random.Random(9)
    script = []
    extras = ["left", "right", "cw", "ccw", "hold", "soft", "tick", "tick", "garbage"]
    while len(script) < 1500:
        for _ in range(rng.randint(0, 4)):
            a = rng.choice(extras)
            if a == "tick":
                script.append(["tick", round(rng.uniform(0.01, 0.6), 3)])
            elif a == "garbage":
                if rng.random() < 0.35:
                    script.append(["garbage", rng.randint(1, 4)])
            else:
                script.append([a])
        script.append(["ai"])
    ai_script = 250
    seed = 123456789

    def run_py():
        b = core.Board(seed)
        trace = []
        for step in script:
            if b.dead:
                break
            name = step[0]
            if name == "tick":
                b.tick(step[1])
            elif name == "garbage":
                b.receive(step[1])
            elif name == "soft":
                b.soft_step()
            elif name == "ai":
                for act in ai.plan(b, 2, None):
                    ai.apply(b, act)
            else:
                ai.apply(b, name)
            trace.append([b.score, b.lines, b.kind or "-", b.rot, b.x, b.y,
                          b.pending_lines(), int(b.dead)])
        b2 = core.Board(seed + 1)
        rng2 = __import__("seedrand").Rand(5)
        for _ in range(ai_script):
            if b2.dead:
                break
            for act in ai.plan(b2, 2, None):
                ai.apply(b2, act)
            if rng2.random() < 0.2:
                b2.receive(rng2.randint(1, 3))
                b2.tick(0.6)
        return dict(trace=trace, rows=b.rows, score=b.score, lines=b.lines,
                    ai_rows=b2.rows, ai_lines=b2.lines, ai_score=b2.score,
                    ai_pieces=b2.pieces)

    py = run_py()
    inp = json.dumps(dict(seed=seed, script=script, ai_script=ai_script))
    try:
        out = subprocess.run([node, tool], input=inp, capture_output=True, text=True,
                             timeout=120, encoding="utf-8")
        js = json.loads(out.stdout)
    except Exception as exc:     # noqa: BLE001
        check(False, "Node-Nachspiel läuft", str(exc))
        return
    same = [k for k in py if py[k] != js.get(k)]
    first = next((i for i, (a, b) in enumerate(zip(py["trace"], js["trace"])) if a != b),
                 None)
    check(len(py["trace"]) > 1200 and py["trace"] == js["trace"],
          "%d Eingaben (%d Zeilen): gleicher Verlauf (Punkte, Zeilen, Stein, Lage, Müll)"
          % (len(py["trace"]), py["lines"]),
          "erste Abweichung bei %s: py=%s js=%s" % (
              first, first is not None and py["trace"][first],
              first is not None and js["trace"][first]))
    check(not same, "gleiches Endfeld, gleiche KI-Partie (%d Steine, %d Zeilen)"
          % (py["ai_pieces"], py["ai_lines"]), "abweichend: %s" % same)


if __name__ == "__main__":
    t0 = time.time()
    audit_srs()
    audit_tspin()
    audit_scoring()
    audit_garbage()
    audit_lock_delay()
    audit_game_over()
    audit_das()
    audit_modes()
    audit_ai()
    audit_layout()
    audit_draw()
    audit_perf()
    audit_web()
    wipe_mem()
    print("\n%s  (%.1f s)" % ("ALLE PRÜFUNGEN BESTANDEN" if not FAILS
                              else "%d FEHLER: %s" % (len(FAILS), FAILS),
                              time.time() - t0))
    sys.exit(1 if FAILS else 0)
