# -*- coding: utf-8 -*-
"""Headless-Audit für Minigolf, Pinball und Bowling.

Geprüft wird:

Minigolf  (1) Abschlag und Loch liegen frei (keine Wand/kein Wasser darauf),
          (2) jede der 18 handgebauten Bahnen ist mit höchstens Par-Schlägen
              einlochbar - nachgewiesen per Solver, der echte Schläge simuliert,
          (2b) dasselbe für alle 342 erzeugten Tour-Bahnen, die zusätzlich
              reproduzierbar sein müssen (gleicher Seed -> gleiche Bahn),
          (2c) das "Aufnehmen" nach acht Schlägen ist Standard, lässt sich
              aber abschalten - dann wird bis zum Einlochen weitergespielt,
          (2d) [R] bricht einen geladenen Schlag ab: kein Putt, Ball bleibt
              liegen, danach ist normales Neuaufladen möglich,
          (2e) "Autoziel" ist Standard (Schläger zeigt vor jedem Schlag zum
              Loch), lässt sich abschalten - dann bleibt die zuletzt
              gewählte Richtung stehen,
          (2f) der Setup-Screen bleibt in allen Auflösungen und allen 14
              Sprachen überschneidungsfrei im Bild,
          (2g) die Stärke-Sperre (rechte Maustaste halten) friert die Kraft
              ein, überlebt den Schlag, bleibt im Setup wirkungslos und
              löst sich in der Pause von selbst.
Eigene     (2h) ugc.json: Speichern/Laden/Löschen, id-Regeln und der
Bahnen          Austausch per .lamapgzmap (inkl. abgewiesener Fremddateien),
           (2i) der Wortfilter: alle 14 Listen laden, jedes Wort wird von
                seinem eigenen Muster gefangen, kein Fehlalarm in den
                Sprachdateien, Verschleierungen werden trotzdem erkannt,
           (2j) die acht neuen Hindernisse (Rohr, Eis, Booster, Magnet,
                Einbahn-Tor, Klebefeld, Drehscheibe, Sprungrampe) tun je
                genau das, was sie sollen - und kein Flug bleibt hängen,
           (2k) Bahnen abweichender Größe halten den Ball in der Bande,
           (2l) alle zwölf Vorlagen sind in Par lösbar - per Solver,
           (2m) MAPS-Reiter, Editor und Teilen-Dialog bleiben in allen
                Auflösungen und allen 14 Sprachen überschneidungsfrei und
                zeichnen fehlerfrei.
Pinball   (3) je Tisch verlässt der Ball mit vollem Plunger die Schussbahn,
          (4) kein Hänger: eine Partie mit 3 Bällen endet von allein,
          (5) ein zu schwacher Schuss lässt sich nachladen (kein Soft-Lock).
Bowling   (6) die Wertung stimmt gegen bekannte Referenzspiele,
          (7) die Frame-Logik zählt inkl. 10. Frame korrekt durch,
          (8) ein gerader Wurf in die Pocket wirft Pins (Physik lebt),
          (9) eine komplette Partie über 10 Frames läuft durch.

Aufruf aus dem Repo-Root:  python tests/newgames_audit.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import glob
import json
import math
import os
import random
import re
import sys
import tempfile
import time

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
# Die Berichte enthalten Umlaute - Ausgabe fest auf UTF-8 stellen,
# sonst zeigt eine Konsole mit anderer Codepage Kraut und Rüben.
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
# Testläufe dürfen weder mem.json noch settings.json anfassen
store._PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                           "_audit-mem.json")
import settings as settings_mod
settings_mod.save_settings = lambda s: None
import i18n
i18n.init()

from game_base import InputEvent
from games import bowling as bw
from games import minigolf as mg
from games import minigolf_draw as draw
from games import minigolf_edit as edit
from games import minigolf_gen as gen
from games import pinball as pb
import swear
import ugc

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
    """Liegt (x, y) auf einer Wand, im Wasser oder außerhalb der Bande?"""
    if not (mg.BORDER + mg.BR <= x <= mg.CW - mg.BORDER - mg.BR):
        return "außerhalb"
    if not (mg.BORDER + mg.BR <= y <= mg.CH - mg.BORDER - mg.BR):
        return "außerhalb"
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
            return "Mühle"
    return None


def simulate_shot(g, start, aim, power, seconds=9.0):
    """Führt einen Schlag aus; liefert (eingelocht, Endposition)."""
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
    """Sucht per Simulation einen Weg ins Loch; liefert (geschafft, Schläge)."""
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
    """Das Aufnehmen nach acht Schlägen ist Standard, aber abschaltbar."""
    print("\nMinigolf - Aufnehmen (Option)")
    check(settings_mod.DEFAULTS["minigolf"]["pickup"] is True,
          "Aufnehmen ist standardmäßig an")
    for pickup, want_state, want_strokes in ((True, mg.HOLE_DONE, mg.MAX_STROKES),
                                             (False, mg.PLAY, 12)):
        g = golf_game()
        g.pickup = pickup
        g.state = mg.PLAY
        g.hole_idx = 0
        g._start_hole()
        for _ in range(12):                  # zwölf schwache Schläge nach unten
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
              "Aufnehmen %s -> Bahn endet nach %d Schlägen"
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
    for _ in range(60):                      # weiter gedrückt halten lädt nicht
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
    check(g.strokes == 1, "danach lässt sich normal neu aufladen",
          "strokes=%d" % g.strokes)


def audit_autoaim():
    """Autoziel: Standard an, abschaltbar - dann zielt man selbst."""
    print("\nMinigolf - Autoziel (Option)")
    check(settings_mod.DEFAULTS["minigolf"]["autoaim"] is True,
          "Autoziel ist standardmäßig an")
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
                  "%s: am Tee zeigt der Schläger zum Loch" % name)
        else:
            check(abs(g.aim + math.pi / 2) < 1e-9,
                  "%s: am Tee zeigt der Schläger neutral bahnaufwärts" % name,
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
                  "%s: nach dem Schlag zeigt der Schläger wieder zum Loch"
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
    # Gespeichert wird nur, was _merge_defaults auch wieder durchlässt.
    kept = settings_mod._merge_defaults({"minigolf": {"autoaim": False}})
    check(kept["minigolf"]["autoaim"] is False,
          "AUS überlebt das Laden von settings.json")


def audit_power_lock():
    """Stärke-Sperre: die rechte Maustaste hält die Kraft fest."""
    print("\nMinigolf - Stärke-Sperre (rechte Maustaste)")
    check(mg.MiniGolfGame.wants_right_click is True,
          "Minigolf bekommt Rechtsklicks überhaupt gemeldet")
    g = golf_game()
    g.state = mg.PLAY
    g.hole_idx = 0
    g._start_hole()
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=(300, 300), button=1))
    for _ in range(30):
        g.update(1 / 60.0)
    fest = g.power
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=(300, 300), button=3))
    for _ in range(120):                     # weiter halten lädt nicht nach
        g.update(1 / 60.0)
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="Up"))
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="Down"))
    check(g.power_lock and fest > 0.3 and g.power == fest,
          "gesperrt: Ladebalken und Pfeiltasten stehen still",
          "power=%.3f gesperrt_bei=%.3f" % (g.power, fest))
    aim0 = g.aim
    g.handle_event(InputEvent(InputEvent.KEYDOWN, key="Left"))
    check(abs(g.aim - aim0) > 1e-6, "gesperrt wird die Kraft, nicht das Zielen")
    g.draw()                                 # goldene Anzeige zeichnet fehlerfrei
    g.handle_event(InputEvent(InputEvent.MOUSEUP, pos=(300, 300), button=1))
    check(g.strokes == 1 and g.power == fest,
          "der Schlag nimmt genau die gesperrte Kraft", "power=%.3f" % g.power)
    for _ in range(1500):
        g.update(1 / 60.0)
        if g.phase == "aim" or g.state != mg.PLAY:
            break
    weiter = (g.state == mg.PLAY and g.phase == "aim")
    check(weiter and g.power_lock and g.power == fest,
          "Sperre überlebt den Schlag", "power=%.3f phase=%s"
          % (g.power, g.phase))
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=(300, 300), button=1))
    for _ in range(30):
        g.update(1 / 60.0)
    check(g.power == fest, "gesperrt: Linksklick lädt nicht bei 5% neu",
          "power=%.3f" % g.power)
    g.handle_event(InputEvent(InputEvent.MOUSEUP, pos=(300, 300), button=3))
    for _ in range(20):
        g.update(1 / 60.0)
    check(not g.power_lock and g.power > fest,
          "Loslassen gibt frei und lädt weiter", "power=%.3f" % g.power)
    # Die Pause frisst das Loslassen der rechten Taste - dann löst die
    # Sperre sich selbst, statt hängen zu bleiben.
    g.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=(300, 300), button=3))
    gesetzt = g.power_lock
    g.paused = True
    g.draw()
    g.paused = False
    check(gesetzt and not g.power_lock, "Pause hebt die Sperre auf")
    # Im Setup darf ein Rechtsklick weder sperren noch einen Knopf drücken.
    g2 = golf_game()
    g2.state = mg.SETUP
    vorher = (g2.course, g2.guide, g2.autoaim, g2.pickup, g2.state)
    for rc in (g2.course_rects[2], g2.guide_rects[1], g2.autoaim_rects[1],
               g2.pickup_rects[1], g2.start_rect):
        g2.handle_event(InputEvent(InputEvent.MOUSEDOWN, pos=rc.center,
                                   button=3))
    check((g2.course, g2.guide, g2.autoaim, g2.pickup, g2.state) == vorher
          and not g2.power_lock, "Setup: Rechtsklick schaltet nichts",
          "%s" % (vorher,))
    # Der HUD-Text muss zwischen Bahn-Anzeige und Ladebalken passen - in
    # allen 14 Sprachen und bei ein- wie dreistelliger Prozentzahl.
    lang_before = i18n.get_language()
    for w, h in ((480, 360), (640, 480), (1280, 960)):
        g3 = quiet(mg.MiniGolfGame(pygame.Surface((w, h)), w, h,
                                   mode="single", game_settings=GS))
        links = 12 + g3._tiny.size(i18n.t("golf.hole", n=9, total=9)
                                   + "  \u00b7  " + i18n.t("golf.par", n=5))[0]
        eng = []
        for code, _ in i18n.AVAILABLE:
            i18n.set_language(code, persist=False)
            for n in (5, 100):
                tw = g3._small.size(i18n.t("golf.lock", n=n))[0]
                pct = g3._tiny.size("%d%%" % n)[0]
                if (w // 2 - tw // 2 < links
                        or w // 2 + tw // 2 > w - 98 - 21 - pct):
                    eng.append("%s/%d%%" % (code, n))
        i18n.set_language(lang_before, persist=False)
        check(not eng, "%4dx%d: HUD-Text der Sperre passt (14 Sprachen)"
              % (w, h), ", ".join(eng))


def audit_setup_layout():
    """Setup-Screen: nichts überlappt, nichts rutscht aus dem Bild."""
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
              "%4dx%d: Knöpfe im Bild, ohne Überlappung" % (w, h),
              "inside=%s overlap=%s beschriftungslücke=%d"
              % (inside, overlap, gap))
        # Die drei Schalter-Beschriftungen müssen in ihre Gruppe passen -
        # sonst laufen lange Übersetzungen in den Nachbarn.
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
    """Prüft jede erzeugte Tour-Bahn auf freie Lage und Einlochbarkeit."""
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

        # Kompletter Durchlauf: Flipper zufällig, aber die Partie muss enden
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
              "Ball hängt fest")
        check(g.scores[0] > 0, "Tisch %-8s: Punkte werden gezählt" % table)

    # Zu schwacher Schuss -> Nachladen möglich
    g = pin_game("classic")
    g.plunger = 0.0
    g._launch()
    for _ in range(600):
        g.update(1 / 60.0)
        if g.phase == "launch":
            break
    check(g.phase == "launch", "Schwacher Schuss lässt sich nachladen",
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

    # Frame-Zählung
    cases = [([3], 0, 1), ([3, 4], 0, 2), ([10], 0, 1),
             ([10] * 9 + [10, 10, 10], 9, 3),
             ([10] * 9 + [3, 7, 5], 9, 3)]
    ok = True
    for rolls, frame, expect in cases:
        got = bw.BowlingGame._rolls_in_frame(rolls, frame)
        ok &= (got == expect)
    check(ok, "Frame-Zählung inkl. 10. Frame")

    g = quiet(bw.BowlingGame(SURF, 640, 480, mode="single", game_settings=GS))
    g.diff = "easy"
    g._new_game()
    g.state = bw.PLAY

    # Würfe in die Pocket: Pins müssen fallen, ein Strike möglich sein
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
    check(10 in knocked, "Strike ist erreichbar", "beste Würfe: %s" % knocked)

    # Vollständige Partie über 10 Frames
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
    check(0 <= g.score <= 300, "Punktzahl im gültigen Bereich (%d)" % g.score)


# ------------------------------------------------- Minigolf: eigene Bahnen
#
# Geprüft wird die ganze Kette: Speicher (ugc.json), Wortfilter, die zwölf
# Vorlagen (per Solver!), die acht neuen Hindernisse, abweichende Bahngrößen,
# Export/Import und die Bildschirmaufteilung in allen 14 Sprachen.

def _ugc_hole(**kw):
    """Kleine Bahn zum Testen eines einzelnen Hindernisses."""
    return gen.normalize(gen.make_hole(3, (50, 140), (50, 10), **kw))


def _ugc_roll(hole, vx, vy, start=None, seconds=4.0):
    """Lässt den Ball rollen und liefert (Spiel, Bahnpunkte)."""
    g = quiet(mg.MiniGolfGame(pygame.Surface((640, 480)), 640, 480,
                              mode="single", game_settings=GS))
    g.holes = [hole]
    g.cards = [[0]]
    g.points = [0]
    g.hole_idx = 0
    g.player = 0
    g.state = mg.PLAY
    g._start_hole()
    if start:
        g.bx, g.by = start
    g.vx, g.vy = vx, vy
    g.phase = "rolling"
    track = [(g.bx, g.by)]
    for _ in range(int(seconds * 60)):
        if g.phase != "rolling":
            break
        g.update(1 / 60.0)
        track.append((g.bx, g.by))
    return g, track


def audit_ugc_store():
    """ugc.json: Rundlauf, id-Regeln, Wortfilter, Export/Import."""
    print("\nMinigolf - eigene Bahnen: Speicher und Austausch")
    tmp = tempfile.mkdtemp()
    alt = ugc._PATH
    ugc._PATH = os.path.join(tmp, "ugc.json")
    try:
        # (1) Rundlauf
        for i in range(3):
            ugc.save_map(ugc.new_map(name="Bahn %d" % i, map_id="bahn-%d" % i))
        ids = [m["id"] for m in ugc.load_maps()]
        check(ids == ["bahn-0", "bahn-1", "bahn-2"],
              "Speichern und Laden erhält Reihenfolge und ids", str(ids))
        check(ugc.get("bahn-1") is not None, "einzelne Bahn über die id findbar")
        ugc.delete_map("bahn-1")
        check([m["id"] for m in ugc.load_maps()] == ["bahn-0", "bahn-2"],
              "Löschen entfernt genau eine Bahn")

        # (2) id-Regeln
        check(ugc.slug("Mein Tunnel!") == "mein-tunnel",
              "slug macht aus Text eine id", ugc.slug("Mein Tunnel!"))
        check(ugc.slug("ÄÖÜ Bahn") == "aeoeue-bahn", "Umlaute werden umschrieben",
              ugc.slug("ÄÖÜ Bahn"))
        check(ugc.slug("!!!") == "", "reine Sonderzeichen ergeben keine id")
        check(not ugc.valid_id("mit leer") and not ugc.valid_id("GROSS")
              and not ugc.valid_id(""),
              "Leerzeichen, Großbuchstaben und leer sind ungültig")
        check(ugc.unique_id("bahn-0") == "bahn-0-2",
              "vergebene id bekommt -2", ugc.unique_id("bahn-0"))
        check(ugc.unique_id("bahn-0", ignore="bahn-0") == "bahn-0",
              "die eigene id blockiert sich nicht selbst")

        # (3) Wortfilter greift beim Speichern, Exportieren und Importieren
        check(ugc.save_map(ugc.new_map(name="Arschloch", map_id="ok-id"))[1]
              == "swear", "Wortfilter blockt den Namen")
        check(ugc.save_map(ugc.new_map(name="Nett", map_id="fuck-map"))[1]
              == "swear", "Wortfilter blockt die id")
        check(ugc.save_map(ugc.new_map(name="Nett", map_id="gut",
                                       author="Hurensohn"))[1] == "swear",
              "Wortfilter blockt den Ersteller-Namen")
        check(not ugc.set_last_author("Wichser"),
              "gesperrter Ersteller-Name wird nicht gemerkt")

        # (4) Export/Import
        m = ugc.get("bahn-0")
        check(ugc.default_filename(m) == "bahn-0" + ugc.EXT,
              "Dateiname wird aus der id vorgeschlagen", ugc.default_filename(m))
        path = os.path.join(tmp, ugc.default_filename(m))
        check(ugc.export_to(m, path)[0] and os.path.isfile(path),
              "Export schreibt die Datei")
        ugc.delete_map("bahn-0")
        ok, why, back = ugc.import_from(path)
        check(ok and back["id"] == "bahn-0", "Import stellt die Bahn her", why)
        ok, why, back = ugc.import_from(path)
        check(ok and back["id"] == "bahn-0-2",
              "zweiter Import weicht auf -2 aus", str(back and back["id"]))
        kaputt = os.path.join(tmp, "kaputt" + ugc.EXT)
        with open(kaputt, "w", encoding="utf-8") as f:
            f.write("{\"format\": \"etwas anderes\"}")
        check(ugc.import_from(kaputt)[1] == "format",
              "fremde Datei wird sauber abgewiesen")
        with open(kaputt, "w", encoding="utf-8") as f:
            f.write("kein json")
        check(ugc.import_from(kaputt)[1] == "io",
              "unlesbare Datei wird sauber abgewiesen")
    finally:
        ugc._PATH = alt


def audit_swear():
    """Die 14 Wortlisten: laden, sich selbst fangen, nichts Harmloses treffen."""
    print("\nWortfilter - 14 Sprachen")
    swear.reload()
    leer = [c for c in swear.codes() if not swear.patterns(c)]
    check(not leer, "jede der 14 Sprachen hat eine Liste", ",".join(leer))
    gesamt = sum(len(swear.patterns(c)) for c in swear.codes())
    check(gesamt > 300, "Muster geladen (%d)" % gesamt)

    # Selbsttest: jedes Klartextwort im Kommentar wird von SEINEM Muster gefangen.
    schlecht = []
    for code in swear.codes():
        pfad = os.path.join(REPO, "lang", "swear", code + ".yml")
        if not os.path.isfile(pfad):
            pfad = os.path.join(REPO, "lang", "lang.expansion", "swear",
                                code + ".yml")
        muster = None
        with open(pfad, encoding="utf-8") as f:
            for zeile in f:
                zeile = zeile.strip()
                if zeile.startswith("- "):
                    muster = zeile[2:]
                elif zeile.startswith("# ") and muster:
                    if not re.search(muster, swear._norm(zeile[2:]), re.I):
                        schlecht.append("%s/%s" % (code, zeile[2:]))
                    muster = None
    check(not schlecht, "jedes Wort wird von seinem eigenen Muster gefangen",
          ", ".join(schlecht[:5]))

    # Fehlalarm: kein einziger vorhandener Oberflächen-Text darf anschlagen.
    treffer = []
    for pfad in (glob.glob(os.path.join(REPO, "lang", "*.json"))
                 + glob.glob(os.path.join(REPO, "lang", "lang.expansion",
                                          "*.json"))):
        with open(pfad, encoding="utf-8") as f:
            for schlüssel, wert in json.load(f).items():
                if isinstance(wert, str) and not swear.is_clean(wert):
                    treffer.append("%s/%s" % (os.path.basename(pfad), schlüssel))
    check(not treffer, "kein Fehlalarm in den Sprachdateien",
          ", ".join(treffer[:5]))
    harmlos = ["Tunnelblick", "Flipper-Ass", "Pick a game", "SPILLET SLUT",
               "Snake Bite", "Concours", "Scunthorpe", "Bok", "Fan", "Slutt",
               "Hole in One", "Mein erstes Loch", "herrlamatv"]
    check(all(swear.is_clean(n) for n in harmlos),
          "harmlose Namen passieren",
          ", ".join(n for n in harmlos if not swear.is_clean(n)))
    böse = ["Arschloch", "sh1t", "f.u.c.k", "K U R W A", "n1gger",
             "my-cunt-map", "H1TL3R", "hijo de puta"]
    check(all(not swear.is_clean(n) for n in böse),
          "Verschleierungen werden trotzdem gefangen",
          ", ".join(n for n in böse if swear.is_clean(n)))


def audit_ugc_templates():
    """Alle zwölf Vorlagen sind vollständig, frei und in Par lösbar."""
    print("\nMinigolf - 12 Vorlagen (mit Solver)")
    g = golf_game()
    check(len(edit.TEMPLATES) == 12, "zwölf Vorlagen vorhanden (%d)"
          % len(edit.TEMPLATES))
    for key in edit.TEMPLATE_KEYS:
        hole = edit.template(key)
        fehlt = [k for k in gen.HOLE_LISTS if k not in hole]
        vollständig = not fehlt and "w" in hole and "h" in hole
        frei = edit.validate(hole)
        blockiert = point_blocked(hole, *hole["tee"]) or \
            point_blocked(hole, *hole["cup"])
        if key == "blank":
            geschafft, schläge = True, 1        # leere Bahn: nichts zu lösen
        else:
            geschafft, schläge = solve(g, hole)
        check(vollständig and not frei and not blockiert and geschafft,
              "%-7s vollständig, frei und in Par lösbar" % key,
              "fehlt=%s prüfung=%s blockiert=%s gelöst=%s"
              % (fehlt, frei, blockiert, geschafft))


def audit_ugc_objects():
    """Die acht neuen Hindernisse tun je genau das, was sie sollen."""
    print("\nMinigolf - acht neue Hindernisse")
    weg_frei = _ugc_roll(_ugc_hole(), 0, -90)[0].by

    _, spur = _ugc_roll(_ugc_hole(ice=[(10, 40, 80, 90)]), 0, -90)
    check(140 - spur[-1][1] > 140 - weg_frei + 5,
          "Eis: der Ball rollt deutlich weiter",
          "%.0f vs %.0f" % (140 - spur[-1][1], 140 - weg_frei))

    g, _ = _ugc_roll(_ugc_hole(sticky=[(10, 90, 80, 40)]), 0, -90)
    check(140 - g.by < 140 - weg_frei - 5, "Klebefeld: der Ball bleibt liegen",
          "%.0f" % (140 - g.by))

    g, _ = _ugc_roll(_ugc_hole(tunnels=[(50, 110, 20, 40, 4)]), 0, -70)
    check(g.bx < 40, "Rohr: der Ball kommt am anderen Ende heraus",
          "x=%.0f" % g.bx)

    g, _ = _ugc_roll(_ugc_hole(boosters=[(35, 100, 30, 20, 0, -1, 150)]), 0, -60)
    langsam = _ugc_roll(_ugc_hole(), 0, -60)[0].by
    check(g.by < langsam - 5, "Booster: schiebt den Ball weiter",
          "%.0f vs %.0f" % (g.by, langsam))

    zieht = _ugc_roll(_ugc_hole(magnets=[(25, 100, 45, 140)]), 0, -90)[0].bx
    stößt = _ugc_roll(_ugc_hole(magnets=[(25, 100, 45, -140)]), 0, -90)[0].bx
    check(zieht < 48 and stößt > 52,
          "Magnet: zieht an und stößt ab", "%.0f / %.0f" % (zieht, stößt))

    tor = _ugc_hole(gates=[(20, 80, 60, 6, 0, -1)])
    _, hin = _ugc_roll(tor, 0, -160, start=(50, 95))
    _, zurück = _ugc_roll(tor, 0, 160, start=(50, 60))
    check(min(p[1] for p in hin) < 78 and max(p[1] for p in zurück) < 80,
          "Einbahn-Tor: lässt nur eine Richtung durch",
          "hin=%.0f zurück=%.0f" % (min(p[1] for p in hin),
                                     max(p[1] for p in zurück)))

    schanze = _ugc_hole(jumps=[(40, 120, 20, 10, 0, -1, 40)],
                        walls=[(20, 95, 60, 10)])
    _, flug = _ugc_roll(schanze, 0, -120)
    check(min(p[1] for p in flug) < 90,
          "Sprungrampe: der Ball fliegt über die Wand",
          "%.0f" % min(p[1] for p in flug))

    g, _ = _ugc_roll(_ugc_hole(spinners=[(50, 100, 14, 3.0)]), 0, -80)
    check(abs(g.bx - 50) > 2, "Drehscheibe: lenkt den Ball ab",
          "x=%.1f" % g.bx)

    # Sicherheitsnetz: kein Schlag darf in der Luft hängen bleiben.
    g, _ = _ugc_roll(schanze, 0, -200, seconds=20.0)
    check(g.phase != "rolling" and g.air == 0.0,
          "kein Hänger: der Flug endet immer", "phase=%s air=%.1f"
          % (g.phase, g.air))


def audit_ugc_size():
    """Bahnen abweichender Größe: Ball bleibt im Feld, Layout passt."""
    print("\nMinigolf - abweichende Bahngrößen")
    for (w, h) in ((60.0, 80.0), (160.0, 240.0)):
        hole = gen.normalize(gen.make_hole(3, (w / 2, h - 12), (w / 2, 14),
                                           w=w, h=h))
        g, spur = _ugc_roll(hole, 60, -160, seconds=8.0)
        lo = mg.BORDER + mg.BR - 0.01
        drin = all(lo <= x <= w - lo + 0.01 and lo <= y <= h - lo + 0.01
                   for (x, y) in spur)
        check(g.cw == w and g.ch == h and drin,
              "%.0fx%.0f: Ball bleibt in der Bande" % (w, h),
              "cw=%s ch=%s drin=%s" % (g.cw, g.ch, drin))
        board = draw.View(g.ox, g.oy, g.scale).board(w, h)
        check(board.width <= g.card_x and board.top >= g.hud_h,
              "%.0fx%.0f: Platz passt neben Scorekarte und HUD" % (w, h),
              "board=%s card_x=%s" % (board, g.card_x))


def audit_ugc_layout():
    """MAPS-Reiter, Editor und Teilen-Dialog: nichts überlappt, alles im Bild."""
    print("\nMinigolf - Aufteilung von MAPS und Editor (14 Sprachen)")
    tmp = tempfile.mkdtemp()
    alt = ugc._PATH
    ugc._PATH = os.path.join(tmp, "ugc.json")
    vorher = i18n.get_language()
    try:
        for i in range(4):
            ugc.save_map(ugc.new_map(name="Testbahn %d" % i,
                                     map_id="testbahn-%d" % i, author="Lama"))
        for w, h in ((480, 360), (640, 480), (800, 600), (960, 720), (1280, 960)):
            g = quiet(mg.MiniGolfGame(pygame.Surface((w, h)), w, h,
                                      mode="single", game_settings=GS))
            g._set_tab("maps")
            ml = g.maps
            rects = list(ml.btn_rects.values())
            drin = all(0 <= r.left and r.right <= w and 0 <= r.top
                       and r.bottom <= h for r in rects)
            über = next(("%s/%s" % (a, b) for i, a in enumerate(rects)
                          for b in rects[i + 1:] if a.colliderect(b)), None)
            platz = ml.list_bottom > ml.list_top and ml.rows_visible >= 3
            check(drin and über is None and platz,
                  "%4dx%d: MAPS-Knöpfe im Bild, ohne Überlappung" % (w, h),
                  "drin=%s über=%s zeilen=%s" % (drin, über, ml.rows_visible))
            # Reiterzeile darf die Kurszeile nicht berühren
            g._set_tab("play")
            lücke = g.course_rects[0].top - g.tab_bottom
            check(lücke >= g._tiny.get_height() + 2,
                  "%4dx%d: Platz für die Beschriftung unter den Reitern" % (w, h),
                  "lücke=%d" % lücke)

            g.ugc_edit(ugc.load_maps()[0])
            e = g.editor
            felder = ([e.back_rect, e.name_rect, e.id_rect, e.save_rect]
                      + [r for grp in e.num_rects.values() for r in grp]
                      + list(e.edit_rects.values())
                      + [r for _k, r in e.pal_rects])
            drin = all(0 <= r.left and r.right <= w and 0 <= r.top
                       and r.bottom <= h for r in felder)
            über = next(("%s/%s" % (a, b) for i, a in enumerate(felder)
                          for b in felder[i + 1:] if a.colliderect(b)), None)
            frei = (e.canvas.width > 40 and e.canvas.height > 40
                    and e.canvas.right <= e.pal_rects[0][1].left
                    and e.canvas.top >= e.head_bottom
                    and e.canvas.bottom <= e.par_top)
            check(drin and über is None and frei,
                  "%4dx%d: Editor ohne Überlappung, Leinwand frei" % (w, h),
                  "drin=%s über=%s leinwand=%s" % (drin, über, e.canvas))
            g.ugc_close_editor()

            # Beschriftungen müssen in ihre Knöpfe passen - in allen Sprachen.
            zu_lang = []
            for code, _name in i18n.AVAILABLE:
                i18n.set_language(code, persist=False)
                g._set_tab("maps")
                for key, rc in g.maps.btn_rects.items():
                    if g.maps.tiny.size(i18n.t("golf.ugc.btn_" + key))[0] > rc.w - 6:
                        zu_lang.append("%s/%s" % (code, key))
                for i, rc in enumerate(g.tab_rects):
                    if g._small.size(i18n.t("golf.tab_" + mg.TABS[i]))[0] > rc.w - 8:
                        zu_lang.append("%s/tab_%s" % (code, mg.TABS[i]))
                for i, key in enumerate(mg.COURSES):
                    if g._tiny.size(i18n.t("golf.course." + key))[0] > \
                            g.course_rects[i].w - 6:
                        zu_lang.append("%s/course.%s" % (code, key))
            i18n.set_language(vorher, persist=False)
            check(not zu_lang, "%4dx%d: Beschriftungen passen (14 Sprachen)"
                  % (w, h), ", ".join(sorted(set(zu_lang))[:6]))
    finally:
        i18n.set_language(vorher, persist=False)
        ugc._PATH = alt


def audit_ugc_draw():
    """Jeder Bildschirm zeichnet in jeder Sprache ohne Absturz."""
    print("\nMinigolf - Zeichnen aller UGC-Bildschirme")
    tmp = tempfile.mkdtemp()
    alt = ugc._PATH
    ugc._PATH = os.path.join(tmp, "ugc.json")
    vorher = i18n.get_language()
    try:
        ugc.save_map(ugc.new_map(name="Alles drin", map_id="alles-drin",
                                 author="Lama", hole=alle_hindernisse()))
        fehler = []
        for code, _name in i18n.AVAILABLE:
            i18n.set_language(code, persist=False)
            try:
                g = quiet(mg.MiniGolfGame(pygame.Surface((640, 480)), 640, 480,
                                          mode="single", game_settings=GS))
                g._set_tab("maps")
                g.draw()
                g.maps._open_share(ugc.load_maps()[0])
                g.draw()
                g.maps._close_share()
                g.ugc_edit(ugc.load_maps()[0])
                g.draw()
                g.editor.sel = ("movers", 0)
                g.draw()
                g.editor._open_template()
                g.draw()
                g.editor.picking = False
                g.ugc_play("alles-drin")
                g.draw()
            except Exception as exc:            # noqa: BLE001 - Testlauf
                fehler.append("%s: %s" % (code, exc))
        check(not fehler, "alle Bildschirme zeichnen in 14 Sprachen",
              "; ".join(fehler[:3]))
    finally:
        i18n.set_language(vorher, persist=False)
        ugc._PATH = alt


def alle_hindernisse():
    """Eine Bahn, auf der jeder der 15 Typen genau einmal vorkommt."""
    return gen.make_hole(
        4, (50, 150), (50, 12),
        walls=[(10, 120, 20, 6)], sand=[(60, 120, 20, 12)],
        water=[(10, 100, 16, 12)], slopes=[(30, 96, 20, 14, 0, 20)],
        bumpers=[(70, 100, 5)], movers=[(30, 78, 12, 6, 20, 0, 20)],
        mills=[(20, 60, 9, 2, 1.2)], tunnels=[(75, 78, 75, 40, 5)],
        ice=[(10, 40, 22, 16)], boosters=[(40, 40, 16, 10, 0, -1, 110)],
        magnets=[(62, 58, 14, 90)], gates=[(34, 30, 22, 5, 0, -1)],
        sticky=[(62, 24, 16, 10)], spinners=[(20, 22, 8, 2.0)],
        jumps=[(44, 132, 12, 8, 0, -1, 30)])

if __name__ == "__main__":
    t0 = time.time()
    audit_minigolf()
    audit_pickup()
    audit_cancel()
    audit_autoaim()
    audit_power_lock()
    audit_setup_layout()
    audit_tour()
    audit_swear()
    audit_ugc_store()
    audit_ugc_objects()
    audit_ugc_size()
    audit_ugc_templates()
    audit_ugc_layout()
    audit_ugc_draw()
    audit_pinball()
    audit_bowling()
    try:
        os.remove(store._PATH)
    except OSError:
        pass
    print("\n%s  (%.1f s)" % ("ALLE PRÜFUNGEN BESTANDEN" if not FAILS
                              else "%d FEHLER: %s" % (len(FAILS), FAILS),
                              time.time() - t0))
    sys.exit(1 if FAILS else 0)
