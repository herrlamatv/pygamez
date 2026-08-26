# -*- coding: utf-8 -*-
"""Headless-Audit fuer Minigolf, Pinball und Bowling.

Geprueft wird:

Minigolf  (1) Abschlag und Loch liegen frei (keine Wand/kein Wasser darauf),
          (2) jede der 18 Bahnen ist mit hoechstens Par-Schlaegen einlochbar -
              nachgewiesen per Solver, der echte Schlaege simuliert.
Pinball   (3) je Tisch verlaesst der Ball mit vollem Plunger die Schussbahn,
          (4) kein Haenger: eine Partie mit 3 Baellen endet von allein,
          (5) ein zu schwacher Schuss laesst sich nachladen (kein Soft-Lock).
Bowling   (6) die Wertung stimmt gegen bekannte Referenzspiele,
          (7) die Frame-Logik zaehlt inkl. 10. Frame korrekt durch,
          (8) ein gerader Wurf in die Pocket wirft Pins (Physik lebt),
          (9) eine komplette Partie ueber 10 Frames laeuft durch.

Aufruf aus dem Repo-Root:  python tests/newgames_audit.py
Exit-Code 0 = alle Pruefungen bestanden.
"""
import json
import math
import os
import random
import sys
import time

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import pygame
pygame.init()
pygame.display.set_mode((640, 480))

import store
# Testlaeufe duerfen weder mem.json noch settings.json anfassen
store._PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                           "_audit-mem.json")
import settings as settings_mod
settings_mod.save_settings = lambda s: None
import i18n
i18n.init()

from games import bowling as bw
from games import minigolf as mg
from games import pinball as pb

GS = json.loads(json.dumps(settings_mod.DEFAULTS))
SURF = pygame.Surface((640, 480))
FAILS = []


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
    return game


# ---------------------------------------------------------------- Minigolf

def golf_game():
    g = quiet(mg.MiniGolfGame(SURF, 640, 480, mode="single", game_settings=GS))
    return g


def point_blocked(hole, x, y):
    """Liegt (x, y) auf einer Wand, im Wasser oder ausserhalb der Bande?"""
    if not (mg.BORDER + mg.BR <= x <= mg.CW - mg.BORDER - mg.BR):
        return "ausserhalb"
    if not (mg.BORDER + mg.BR <= y <= mg.CH - mg.BORDER - mg.BR):
        return "ausserhalb"
    for (rx, ry, rw, rh) in hole["walls"]:
        if rx - mg.BR < x < rx + rw + mg.BR and ry - mg.BR < y < ry + rh + mg.BR:
            return "Wand"
    for (rx, ry, rw, rh) in hole["water"]:
        if rx < x < rx + rw and ry < y < ry + rh:
            return "Wasser"
    for (bx, by, br) in hole["bumpers"]:
        if math.hypot(x - bx, y - by) < br + mg.BR:
            return "Puffer"
    for (mx, my, ln, arms, sp) in hole["mills"]:
        if math.hypot(x - mx, y - my) < ln + mg.ARM_W + mg.BR:
            return "Muehle"
    return None


def simulate_shot(g, start, aim, power, seconds=9.0):
    """Fuehrt einen Schlag aus; liefert (eingelocht, Endposition)."""
    g.bx, g.by = start
    g.vx = g.vy = 0.0
    g.safe = (g.bx, g.by)
    g.strokes = 0
    g.state = mg.PLAY
    g.phase = "aim"
    g.aim = aim
    g.power = power
    g.charging = False
    g._strike()
    dt = 1 / 60.0
    for _ in range(int(seconds / dt)):
        g.update(dt)
        if g.state == mg.HOLE_DONE:
            return True, (g.bx, g.by)
        if g.phase == "aim":
            break
    return False, (g.bx, g.by)


def audit_minigolf(angles=44, powers=(0.35, 0.55, 0.75, 1.0), spread=6):
    print("\nMinigolf - 18 Bahnen")
    g = golf_game()
    holes = [dict(h) for h in mg.HOLES_CLASSIC] + [dict(h) for h in mg.HOLES_PRO]
    for idx, hole in enumerate(holes, start=1):
        g.hole = hole
        g.par = hole["par"]
        g.cup = (float(hole["cup"][0]), float(hole["cup"][1]))
        tee = (float(hole["tee"][0]), float(hole["tee"][1]))
        bad_tee = point_blocked(hole, *tee)
        bad_cup = point_blocked(hole, *g.cup)
        ok = check(bad_tee is None, "Bahn %2d: Abschlag frei" % idx, str(bad_tee))
        ok &= check(bad_cup is None, "Bahn %2d: Loch frei" % idx, str(bad_cup))
        if not ok:
            continue
        # Solver: Schlag 1 von der Tee-Box, danach von den besten Ruhelagen
        starts = [tee]
        holed = False
        strokes_needed = 0
        for depth in range(1, hole["par"] + 1):
            results = []
            for start in starts:
                base = math.atan2(g.cup[1] - start[1], g.cup[0] - start[0])
                for a in range(angles):
                    aim = base + (a - angles / 2) * (2 * math.pi / angles)
                    for pw in powers:
                        g.mill_a = (a * 0.37) % 6.283
                        g.move_t = (a * 0.53) % 9.0
                        hit, pos = simulate_shot(g, start, aim, pw)
                        if hit:
                            holed = True
                            strokes_needed = depth
                            break
                        results.append(pos)
                    if holed:
                        break
                if holed:
                    break
            if holed:
                break
            # Kandidaten: je zur Haelfte nach Nähe zum Loch und nach
            # Fortschritt in Richtung Loch (kleineres y) ausgewaehlt.
            by_dist = sorted(results, key=lambda p: math.hypot(p[0] - g.cup[0],
                                                               p[1] - g.cup[1]))
            by_progress = sorted(results, key=lambda p: p[1])
            picked = []
            for p in [x for pair in zip(by_dist, by_progress) for x in pair]:
                if all(math.hypot(p[0] - q[0], p[1] - q[1]) > 7 for q in picked):
                    picked.append(p)
                if len(picked) >= spread:
                    break
            starts = picked
            if not starts:
                break
        check(holed, "Bahn %2d: einlochbar (Par %d)" % (idx, hole["par"]),
              "kein Weg ins Loch gefunden")
        if holed and strokes_needed:
            check(strokes_needed <= hole["par"],
                  "Bahn %2d: Par erreichbar (%d Schlaege)" % (idx, strokes_needed))


# ----------------------------------------------------------------- Pinball

def pin_game(table):
    g = quiet(pb.PinballGame(SURF, 640, 480, mode="single", game_settings=GS))
    g.table_key = table
    g._new_game()
    g.state = pb.PLAY
    return g


def audit_pinball():
    print("\nPinball - 3 Tische")
    for table in pb.TABLES:
        g = pin_game(table)
        g.plunger = 1.0
        g._launch()
        left_lane = False
        for _ in range(1200):
            g.update(1 / 60.0)
            if g.balls and g.balls[0].x < 80:
                left_lane = True
                break
        check(left_lane, "Tisch %-8s: voller Plunger erreicht das Spielfeld" % table)

        # Kompletter Durchlauf: Flipper zufaellig, aber die Partie muss enden
        g = pin_game(table)
        random.seed(11)
        ended = False
        for i in range(60 * 60 * 6):          # bis zu 6 Minuten Spielzeit
            if g.phase == "launch":
                g.plunger = random.uniform(0.5, 1.0)
                g._launch()
            g.flip_l_up = (i // 17) % 3 == 0
            g.flip_r_up = (i // 19) % 3 == 0
            g.update(1 / 60.0)
            if g.game_over:
                ended = True
                break
        check(ended, "Tisch %-8s: Partie endet von allein" % table,
              "Ball haengt fest")
        check(g.scores[0] > 0, "Tisch %-8s: Punkte werden gezaehlt" % table)

    # Zu schwacher Schuss -> Nachladen moeglich
    g = pin_game("classic")
    g.plunger = 0.0
    g._launch()
    for _ in range(600):
        g.update(1 / 60.0)
        if g.phase == "launch":
            break
    check(g.phase == "launch", "Schwacher Schuss laesst sich nachladen",
          "Ball bleibt in der Schussbahn liegen")


# ----------------------------------------------------------------- Bowling

REFERENCE_GAMES = [
    ([10] * 12, 300, "perfektes Spiel"),
    ([5, 5] * 10 + [5], 150, "lauter Spares"),
    ([0] * 20, 0, "alles Rinne"),
    ([9, 0] * 10, 90, "immer neun"),
    ([1, 4, 4, 5, 6, 4, 5, 5, 10, 0, 1, 7, 3, 6, 4, 10, 2, 8, 6], 133,
     "gemischtes Spiel"),
]


def audit_bowling():
    print("\nBowling - Wertung und Physik")
    for rolls, expect, label in REFERENCE_GAMES:
        got = bw.total_score(rolls)
        check(got == expect, "Wertung: %-18s = %d" % (label, expect),
              "berechnet %d" % got)

    # Frame-Zaehlung
    cases = [([3], 0, 1), ([3, 4], 0, 2), ([10], 0, 1),
             ([10] * 9 + [10, 10, 10], 9, 3),
             ([10] * 9 + [3, 7, 5], 9, 3)]
    ok = True
    for rolls, frame, expect in cases:
        got = bw.BowlingGame._rolls_in_frame(rolls, frame)
        ok &= (got == expect)
    check(ok, "Frame-Zaehlung inkl. 10. Frame")

    g = quiet(bw.BowlingGame(SURF, 640, 480, mode="single", game_settings=GS))
    g.diff = "easy"
    g._new_game()
    g.state = bw.PLAY

    # Wuerfe in die Pocket: Pins muessen fallen, ein Strike moeglich sein
    shots = [(0.15, 0.00, 0.00, 0.60), (-0.20, 0.12, 0.00, 0.85),
             (0.15, 0.25, -0.30, 1.00), (0.30, 0.12, -0.30, 1.00),
             (0.00, -0.40, 0.30, 0.60)]
    knocked = []
    for pos, aim, spin, power in shots:
        g._new_game()               # frische Partie: Frame 1, volle Aufstellung
        g.state = bw.PLAY
        g.pos, g.aim, g.spin, g.power = pos, aim, spin, power
        g.step = len(bw.STEPS)
        random.seed(1)
        g._throw()
        for _ in range(900):
            g.update(1 / 60.0)
            if g.rolls[0]:
                break
        knocked.append(g.rolls[0][-1] if g.rolls[0] else 0)
    check(max(knocked) >= 7, "Pocket-Wurf wirft Pins", "max %d" % max(knocked))
    check(10 in knocked, "Strike ist erreichbar", "beste Wuerfe: %s" % knocked)

    # Vollstaendige Partie ueber 10 Frames
    g = quiet(bw.BowlingGame(SURF, 640, 480, mode="single", game_settings=GS))
    g._new_game()
    g.state = bw.PLAY
    random.seed(5)
    rolls_done = 0
    for _ in range(60 * 60 * 10):
        if g.ball is None and not g.game_over:
            g.pos = random.uniform(-0.4, 0.4)
            g.aim = random.uniform(-0.35, 0.35)
            g.spin = random.uniform(-0.5, 0.5)
            g.power = random.uniform(0.6, 1.0)
            g.step = len(bw.STEPS)
            g._throw()
            rolls_done += 1
        g.update(1 / 60.0)
        if g.game_over:
            break
    check(g.game_over, "Partie endet nach 10 Frames",
          "Frames: %s" % g.frame)
    check(12 <= len(g.rolls[0]) <= 21,
          "Wurfzahl plausibel (%d)" % len(g.rolls[0]))
    check(g.score == bw.total_score(g.rolls[0]),
          "Endpunktzahl passt zur Wertung")
    check(0 <= g.score <= 300, "Punktzahl im gueltigen Bereich (%d)" % g.score)


if __name__ == "__main__":
    t0 = time.time()
    audit_minigolf()
    audit_pinball()
    audit_bowling()
    try:
        os.remove(store._PATH)
    except OSError:
        pass
    print("\n%s  (%.1f s)" % ("ALLE PRUEFUNGEN BESTANDEN" if not FAILS
                              else "%d FEHLER: %s" % (len(FAILS), FAILS),
                              time.time() - t0))
    sys.exit(1 if FAILS else 0)
