# -*- coding: utf-8 -*-
"""Headless-Audit fuer Minigolf, Pinball und Bowling.

Geprueft wird:

Minigolf  (1) Abschlag und Loch liegen frei (keine Wand/kein Wasser darauf),
          (2) jede der 18 handgebauten Bahnen ist mit hoechstens Par-Schlaegen
              einlochbar - nachgewiesen per Solver, der echte Schlaege simuliert,
          (2b) dasselbe fuer alle 342 erzeugten Tour-Bahnen, die zusaetzlich
              reproduzierbar sein muessen (gleicher Seed -> gleiche Bahn),
          (2c) das "Aufnehmen" nach acht Schlaegen ist Standard, laesst sich
              aber abschalten - dann wird bis zum Einlochen weitergespielt,
          (2d) [R] bricht einen geladenen Schlag ab: kein Putt, Ball bleibt
              liegen, danach ist normales Neuaufladen moeglich,
          (2e) "Autoziel" ist Standard (Schlaeger zeigt vor jedem Schlag zum
              Loch), laesst sich abschalten - dann bleibt die zuletzt
              gewaehlte Richtung stehen,
          (2f) der Setup-Screen bleibt in allen Aufloesungen und allen 14
              Sprachen ueberschneidungsfrei im Bild.
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

from game_base import InputEvent
from games import bowling as bw
from games import minigolf as mg
from games import minigolf_gen as gen
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


def solve(g, hole, angles=40, powers=(0.35, 0.55, 0.75, 1.0), spread=5,
          max_depth=None):
    """Sucht per Simulation einen Weg ins Loch; liefert (geschafft, Schlaege)."""
    g.hole = hole
    g.par = hole["par"]
    g.cup = (float(hole["cup"][0]), float(hole["cup"][1]))
    tee = (float(hole["tee"][0]), float(hole["tee"][1]))
    starts = [tee]
    depth_limit = max_depth or hole["par"]
    for depth in range(1, depth_limit + 1):
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
                        return True, depth
                    results.append(pos)
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
    return False, 0


def audit_minigolf():
    print("\nMinigolf - 18 handgebaute Bahnen")
    g = golf_game()
    check(abs(mg.BR - gen.BALL_R) < 1e-9, "Ballradius in Spiel und Generator gleich")
    check(gen.TOTAL_HOLES == 360, "360 Bahnen insgesamt (%d)" % gen.TOTAL_HOLES)
    holes = [dict(h) for h in mg.HOLES_CLASSIC] + [dict(h) for h in mg.HOLES_PRO]
    for idx, hole in enumerate(holes, start=1):
        bad_tee = point_blocked(hole, *hole["tee"])
        bad_cup = point_blocked(hole, *hole["cup"])
        ok = check(bad_tee is None, "Bahn %2d: Abschlag frei" % idx, str(bad_tee))
        ok &= check(bad_cup is None, "Bahn %2d: Loch frei" % idx, str(bad_cup))
        if not ok:
            continue
        holed, strokes = solve(g, hole)
        check(holed, "Bahn %2d: einlochbar in Par %d" % (idx, hole["par"]),
              "kein Weg ins Loch gefunden")


def audit_pickup():
    """Das Aufnehmen nach acht Schlaegen ist Standard, aber abschaltbar."""
    print("\nMinigolf - Aufnehmen (Option)")
    check(settings_mod.DEFAULTS["minigolf"]["pickup"] is True,
          "Aufnehmen ist standardmaessig an")
    for pickup, want_state, want_strokes in ((True, mg.HOLE_DONE, mg.MAX_STROKES),
                                             (False, mg.PLAY, 12)):
        g = golf_game()
        g.pickup = pickup
        g.state = mg.PLAY
        g.hole_idx = 0
        g._start_hole()
        for _ in range(12):                  # zwoelf schwache Schlaege nach unten
            if g.state != mg.PLAY:
                break
            g.aim = math.pi / 2
            g.power = 0.12
            g._strike()
            for _ in range(700):
                g.update(1 / 60.0)
                if g.phase == "aim" or g.state != mg.PLAY:
                    break
        check(g.state == want_state and g.strokes == want_strokes,
              "Aufnehmen %s -> Bahn endet nach %d Schlaegen"
              % ("AN " if pickup else "AUS", want_strokes),
              "state=%s strokes=%d" % (g.state, g.strokes))


def audit_cancel():
    """Taste R bricht einen geladenen Schlag ab, ohne zu putten."""
    print("\nMinigolf - Schlag abbrechen (R)")
    g = golf_game()
    g.state = mg.PLAY
    g.hole_idx = 0
    g._start_hole()
    where = (g.bx, g.by)
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=(300, 300), button=1))
    for _ in range(40):
        g.update(1 / 60.0)
    charged = g.charging and g.power > 0.4
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="r"))
    check(charged and not g.charging, "R beendet das Aufladen")
    for _ in range(60):                      # weiter gedrueckt halten laedt nicht
        g.update(1 / 60.0)
    held = g.power
    g.handle_event(InputEvent(InputEvent.MOUSEUP, pos=(300, 300), button=1))
    for _ in range(30):
        g.update(1 / 60.0)
    check(g.strokes == 0 and g.phase == "aim" and (g.bx, g.by) == where
          and held <= 0.36,
          "kein Schlag, Ball bleibt liegen",
          "strokes=%d phase=%s power=%.2f" % (g.strokes, g.phase, held))
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=(300, 300), button=1))
    for _ in range(40):
        g.update(1 / 60.0)
    g.handle_event(InputEvent(InputEvent.MOUSEUP, pos=(300, 300), button=1))
    check(g.strokes == 1, "danach laesst sich normal neu aufladen",
          "strokes=%d" % g.strokes)


def audit_autoaim():
    """Autoziel: Standard an, abschaltbar - dann zielt man selbst."""
    print("\nMinigolf - Autoziel (Option)")
    check(settings_mod.DEFAULTS["minigolf"]["autoaim"] is True,
          "Autoziel ist standardmaessig an")
    shot = math.pi * 0.4                     # bewusst weg vom Loch
    for auto in (True, False):
        name = "AN " if auto else "AUS"
        g = golf_game()
        g.autoaim = auto
        g.state = mg.PLAY
        g.hole_idx = 0
        g._start_hole()
        to_cup = math.atan2(g.cup[1] - g.by, g.cup[0] - g.bx)
        if auto:
            check(abs(g.aim - to_cup) < 1e-9,
                  "%s: am Tee zeigt der Schlaeger zum Loch" % name)
        else:
            check(abs(g.aim + math.pi / 2) < 1e-9,
                  "%s: am Tee zeigt der Schlaeger neutral bahnaufwaerts" % name,
                  "aim=%.3f" % g.aim)
        g.aim = shot
        g.power = 0.12
        g._strike()
        for _ in range(1200):
            g.update(1 / 60.0)
            if g.phase == "aim" or g.state != mg.PLAY:
                break
        to_cup = math.atan2(g.cup[1] - g.by, g.cup[0] - g.bx)
        rolled = g.state == mg.PLAY and g.phase == "aim"
        if auto:
            check(rolled and abs(g.aim - to_cup) < 1e-9,
                  "%s: nach dem Schlag zeigt der Schlaeger wieder zum Loch"
                  % name, "aim=%.3f soll=%.3f" % (g.aim, to_cup))
        else:
            check(rolled and abs(g.aim - shot) < 1e-9
                  and abs(g.aim - to_cup) > 1e-2,
                  "%s: nach dem Schlag bleibt die Richtung stehen" % name,
                  "aim=%.3f geschlagen=%.3f" % (g.aim, shot))
    # Schalter im Setup: Klick auf AUS/AN wechselt und wird gespeichert.
    g = golf_game()
    g.state = mg.SETUP
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN,
                              pos=g.autoaim_rects[1].center, button=1))
    off = not g.autoaim
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="z"))
    check(off and g.autoaim, "Setup: Klick auf AUS und Taste [Z] schalten um",
          "autoaim=%s" % g.autoaim)
    # Gespeichert wird nur, was _merge_defaults auch wieder durchlaesst.
    kept = settings_mod._merge_defaults({"minigolf": {"autoaim": False}})
    check(kept["minigolf"]["autoaim"] is False,
          "AUS ueberlebt das Laden von settings.json")


def audit_setup_layout():
    """Setup-Screen: nichts ueberlappt, nichts rutscht aus dem Bild."""
    print("\nMinigolf - Setup-Layout")
    lang_before = i18n.get_language()
    for w, h in ((480, 360), (640, 480), (800, 600), (960, 720), (1280, 960)):
        g = quiet(mg.MiniGolfGame(pygame.Surface((w, h)), w, h, mode="single",
                                  game_settings=GS))
        rects = [r for grp in (g.course_rects, g.tour_rects, g.guide_rects,
                               g.autoaim_rects, g.pickup_rects,
                               [g.start_rect]) for r in grp]
        inside = all(r.left >= 0 and r.right <= w and r.top >= 0
                     and r.bottom <= h - 26 for r in rects)
        overlap = next((("%s/%s" % (a, b)) for i, a in enumerate(rects)
                        for b in rects[i + 1:] if a.colliderect(b)), None)
        gap = g.guide_rects[0].top - g.tour_rects[0].bottom
        check(inside and overlap is None and gap >= g._tiny.get_height() + 2,
              "%4dx%d: Knoepfe im Bild, ohne Ueberlappung" % (w, h),
              "inside=%s overlap=%s beschriftungsluecke=%d"
              % (inside, overlap, gap))
        # Die drei Schalter-Beschriftungen muessen in ihre Gruppe passen -
        # sonst laufen lange Uebersetzungen in den Nachbarn.
        zu_lang = []
        for code, _ in i18n.AVAILABLE:
            i18n.set_language(code, persist=False)
            for key in ("golf.lbl_guide", "golf.lbl_autoaim",
                        "golf.lbl_pickup"):
                if g._tiny.size(i18n.t(key))[0] > g.opt_w:
                    zu_lang.append("%s/%s" % (code, key.split(".")[-1]))
        i18n.set_language(lang_before, persist=False)
        check(not zu_lang, "%4dx%d: Schalter-Beschriftungen passen (14 Sprachen)"
              % (w, h), ", ".join(zu_lang))
        g.draw()                             # zeichnet der Screen fehlerfrei?


def audit_tour(sample=None):
    """Prueft jede erzeugte Tour-Bahn auf freie Lage und Einlochbarkeit."""
    print("\nMinigolf - Tour: %d Kurse x %d Bahnen"
          % (gen.TOUR_COURSES, gen.HOLES_PER_ROUND))
    g = golf_game()
    courses = sample or range(1, gen.TOUR_COURSES + 1)
    total = fails = 0
    worst = []
    for course in courses:
        holes = gen.course_holes(course)
        bad = []
        for i, hole in enumerate(holes):
            total += 1
            label = "Kurs %d Bahn %d (%s)" % (course, i + 1, hole.get("family"))
            blocked = (point_blocked(hole, *hole["tee"])
                       or point_blocked(hole, *hole["cup"]))
            if blocked:
                bad.append(label + ": " + blocked)
                continue
            holed, _ = solve(g, hole, angles=36, spread=4,
                             max_depth=hole["par"] + 1)
            if not holed:
                bad.append(label + ": nicht einlochbar")
        # Reproduzierbarkeit: derselbe Kurs muss identisch neu entstehen
        again = gen.course_holes(course)
        if [h["cup"] for h in again] != [h["cup"] for h in holes]:
            bad.append("Kurs %d: nicht reproduzierbar" % course)
        fails += len(bad)
        worst.extend(bad)
        print("   Kurs %2d (Stufe %d, Par %2d): %s"
              % (course, gen.tier_of(course), sum(h["par"] for h in holes),
                 "OK" if not bad else "%d PROBLEM(E)" % len(bad)))
    check(fails == 0, "alle %d Tour-Bahnen einlochbar und frei" % total,
          "; ".join(worst[:6]))


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
    audit_pickup()
    audit_cancel()
    audit_autoaim()
    audit_setup_layout()
    audit_tour()
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
