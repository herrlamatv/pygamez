# -*- coding: utf-8 -*-
"""Headless-Audit für Battleship / Schiffe versenken.

Geprüft wird:

Regeln     (1) Aufstellen: Brettgrenzen, Überlappung, Berühr-Regel (Seite und
               Ecke), Zufallsaufstellungen sind immer legal,
           (2) Aufstellen über echte Eingaben: Drag & Drop aus dem Dock, Drehen
               (R / Rechtsklick), Umsetzen, ungültiges Ablegen, Tastatur-Weg,
               "Zufällig", "Alle entfernen", "Los" erst mit voller Flotte,
           (3) Schießen: Wasser/Treffer/Versenkt, kein Feld doppelt,
               Salven-Modus (Schüsse = eigene Schiffe), Nochmal-Schießen nach
               Treffer, Sieg -> Punkte, Statistik und Erfolge.
KI         (4) jede Stärke versenkt jede Flotte in <= 100 Schüssen ohne doppelten
               Schuss; im Mittel gilt schwer < mittel < leicht,
           (5) die KI schaut nicht unter die Wasseroberfläche (gleiches Wissen ->
               gleicher Schuss), ein KI-Zug im Spiel feuert genau seine Schüsse.
Hotseat    (6) der Übergabe-Bildschirm zeichnet keine Flotte, verlangt eine
               kurze Pause vor dem Bestätigen und führt durch den ganzen Ablauf.
Persistenz (7) Setup-Änderungen landen sofort in den Einstellungen; die
               Prüfregeln in settings.SCHEMAS halten ungültige Werte fern.
Layout     (8) Setup, HUD, Aufstellen, Gefecht, Rundenende und Übergabe passen in
               allen 5 Auflösungen und allen 14 Sprachen; Auflösungswechsel und
               beide Extrem-Themes (v42, v1) zeichnen fehlerfrei.
Tempo      (9) Zeichnen bei 1280x960 und die schwere KI bleiben im Frame-Budget.

Aufruf aus dem Repo-Root:  python tests/audit_battleship.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import json
import os
import random
import sys
import time
import types

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
# Testläufe dürfen weder mem.json noch settings.json anfassen
store._PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                           "_audit-battleship-mem.json")
import settings as settings_mod
SAVED = []
settings_mod.save_settings = lambda s: SAVED.append(json.loads(json.dumps(s.get("battleship", {}))))
import i18n
i18n.init()
import ui

from game_base import InputEvent
try:
    from games import battleship as bs
except Exception as exc:          # anderes Spiel gerade kaputt -> nur unser Modul laden
    print("Hinweis: games-Paket nicht importierbar (%s) - lade Battleship direkt" % exc)
    for name in [m for m in sys.modules if m == "games" or m.startswith("games.")]:
        del sys.modules[name]
    _pkg = types.ModuleType("games")
    _pkg.__path__ = [os.path.join(REPO, "games")]
    sys.modules["games"] = _pkg
    from games import battleship as bs
from games import battleship_core as core

GS = json.loads(json.dumps(settings_mod.DEFAULTS))
FAILS = []
RESOLUTIONS = ((480, 360), (640, 480), (800, 600), (960, 720), (1280, 960))
LANGS = [code for code, _ in i18n.AVAILABLE]


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + detail) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


def quiet(game):
    game.play_sound = lambda name: None
    game.rumble = lambda ms=0: None
    game.tone_turn = lambda: None
    game._sfx_splash = lambda: None
    game.events = []
    game.results = []
    game.ach_event = lambda *a, **k: game.events.append(a[0])
    game.report_result = lambda won: game.results.append(won)
    return game


def new_game(w=640, h=480, mode="single", settings=None):
    gs = settings if settings is not None else json.loads(json.dumps(GS))
    return quiet(bs.BattleshipGame(pygame.Surface((w, h)), w, h, mode=mode,
                                   game_settings=gs))


def key(game, k):
    game.handle_event(InputEvent(InputEvent.KEYDOWN, key=k))


def mouse(game, kind, pos, button=1):
    game.handle_event(InputEvent(kind, pos=(int(pos[0]), int(pos[1])), button=button))


def cell_pos(game, r, c):
    return (game.bx + c * game.cb + game.cb // 2, game.by + r * game.cb + game.cb // 2)


def run(game, secs, dt=1 / 60.0):
    for _ in range(int(secs / dt)):
        game.update(dt)


def layout_of(sea):
    return sorted((s.idx, s.r, s.c, s.horiz) for s in sea.ships)


# ------------------------------------------------------------- (1) Regeln
def audit_rules():
    print("\n(1) Aufstellungsregeln")
    sea = core.Sea()
    check(not sea.can_place(0, 0, 6, True), "Flugzeugträger ragt nicht über den Rand (J)")
    check(not sea.can_place(0, 6, 0, False), "Flugzeugträger ragt nicht über den Rand (10)")
    check(sea.can_place(0, 0, 5, True) and sea.can_place(0, 5, 0, False),
          "Flugzeugträger passt genau bis an den Rand")
    sea.place(0, 4, 2, True)                      # C5..G5
    check(not sea.can_place(1, 2, 4, False), "Überlappung wird abgelehnt")
    check(sea.can_place(1, 5, 2, True, touch=True), "berühren an der Seite: erlaubt mit touch=True")
    check(not sea.can_place(1, 5, 2, True, touch=False),
          "berühren an der Seite: verboten mit touch=False")
    check(sea.can_place(4, 5, 7, True, touch=True) and not sea.can_place(4, 5, 7, True, touch=False),
          "Berührung über Eck: nur mit touch=True erlaubt")
    check(not sea.can_place(4, 4, 7, True, touch=False) and sea.can_place(4, 4, 8, True, touch=False),
          "Heck an Bug verboten, ein Feld Abstand erlaubt (touch=False)")
    check(sea.can_place(0, 6, 2, True, touch=False), "Schiff darf seine eigene Lage neu wählen")
    rng = random.Random(11)
    bad = 0
    for touch in (True, False):
        for _ in range(250):
            s2 = core.Sea()
            if not s2.randomize(touch, rng) or not s2.valid_layout(touch):
                bad += 1
    check(bad == 0, "500 Zufallsaufstellungen (mit/ohne Berühren) sind alle legal", "%d kaputt" % bad)
    s3 = core.Sea()
    s3.place(0, 0, 0, True)
    s3.place(1, 1, 0, True)
    s3.place(2, 3, 0, True)
    s3.place(3, 5, 0, True)
    s3.place(4, 7, 0, True)
    check(s3.valid_layout(True) and not s3.valid_layout(False),
          "valid_layout erkennt Berührung je nach Regel")


# --------------------------------------------------- (2) Aufstellen im Spiel
def audit_placement_input():
    print("\n(2) Aufstellen über Maus und Tastatur")
    g = new_game()
    key(g, "Return")
    check(g.state == bs.PLACE, "Enter im Setup startet das Aufstellen")
    sea = g.seas[0]
    sea.clear()
    # Drag & Drop: Kreuzer (idx 2) aus dem Dock nach B3 (r=2, c=1), am 1. Feld gegriffen
    drc = g.dock_rects[2]
    grab_x = drc.x + 8 + g.dock_cell // 2
    mouse(g, InputEvent.MOUSEDOWN, (grab_x, drc.centery))
    check(g.held is not None and g.held["idx"] == 2 and g.held["grab"] == 0,
          "Klick aufs Dock nimmt das Schiff auf (Griff am Heck)")
    mouse(g, InputEvent.MOUSEMOVE, cell_pos(g, 2, 1))
    check(g._held_anchor() == (2, 1) and g._held_valid(), "Vorschau rastet am Brett ein und ist gültig")
    mouse(g, InputEvent.MOUSEUP, cell_pos(g, 2, 1))
    ship = sea.ship_by_idx(2)
    check(ship is not None and (ship.r, ship.c, ship.horiz) == (2, 1, True) and g.held is None,
          "Loslassen legt den Kreuzer ab (B3, waagerecht)")
    # Rechtsklick dreht das liegende Schiff um das angeklickte Feld
    mouse(g, InputEvent.MOUSEDOWN, cell_pos(g, 2, 1), button=3)
    ship = sea.ship_by_idx(2)
    check((ship.r, ship.c, ship.horiz) == (2, 1, False), "Rechtsklick dreht ein liegendes Schiff")
    # Umsetzen per Drag: Schiff greifen (mittleres Feld) und nach E6 ziehen, unterwegs R
    mouse(g, InputEvent.MOUSEDOWN, cell_pos(g, 3, 1))
    check(g.held is not None and g.held["grab"] == 1 and sea.ship_by_idx(2) is None,
          "liegendes Schiff lässt sich wieder aufnehmen")
    mouse(g, InputEvent.MOUSEMOVE, cell_pos(g, 5, 4))
    key(g, "r")
    check(g.held["horiz"] is True, "R dreht das gehaltene Schiff")
    mouse(g, InputEvent.MOUSEUP, cell_pos(g, 5, 4))
    ship = sea.ship_by_idx(2)
    check(ship is not None and (ship.r, ship.c, ship.horiz) == (5, 3, True),
          "Umsetzen mit Drehen landet um den Griffpunkt (D6..F6)")
    # Ungültig: Schlachtschiff quer über den Kreuzer ziehen -> zurück ins Dock
    drc = g.dock_rects[1]
    mouse(g, InputEvent.MOUSEDOWN, (drc.x + 8 + g.dock_cell // 2, drc.centery))
    mouse(g, InputEvent.MOUSEMOVE, cell_pos(g, 5, 2))
    valid = g._held_valid()
    mouse(g, InputEvent.MOUSEUP, cell_pos(g, 5, 2))
    check(not valid and sea.ship_by_idx(1) is None and g.held is None,
          "ungültige Lage wird rot angezeigt und nicht übernommen")
    # Berühr-Regel im Spiel
    g.rules["touch"] = False
    mouse(g, InputEvent.MOUSEDOWN, (drc.x + 8 + g.dock_cell // 2, drc.centery))
    mouse(g, InputEvent.MOUSEMOVE, cell_pos(g, 6, 3))
    touch_blocked = not g._held_valid()
    mouse(g, InputEvent.MOUSEMOVE, cell_pos(g, 7, 3))
    gap_ok = g._held_valid()
    mouse(g, InputEvent.MOUSEUP, cell_pos(g, 7, 3))
    check(touch_blocked and gap_ok and sea.ship_by_idx(1) is not None,
          "ohne Berühren: direkt darunter rot, eine Reihe Abstand grün")
    g.rules["touch"] = True
    # Tastatur: Enter nimmt das nächste Schiff, Pfeile, R, Leertaste legt ab
    sea.clear()
    g.kcursor = [0, 0]
    key(g, "Return")
    check(g.held is not None and g.held["idx"] == 0 and not g.held["mouse"],
          "Enter nimmt das nächste fehlende Schiff (Tastatur)")
    for k in ("Down", "Down", "Right"):
        key(g, k)
    key(g, "r")
    key(g, "space")
    ship = sea.ship_by_idx(0)
    check(ship is not None and (ship.r, ship.c, ship.horiz) == (2, 1, False),
          "Pfeile + R + Leertaste setzen den Träger senkrecht auf B3")
    key(g, "Return")
    g.kcursor = [9, 9]
    key(g, "space")
    ship = sea.ship_by_idx(1)
    check(ship is not None and all(core.in_board(r, c) for (r, c) in ship.cells)
          and (9, 9) in ship.cells and not ship.horiz,
          "Tastatur-Vorschau bleibt im Brett (senkrecht in der Ecke J10)")
    key(g, "Return")
    check(g.state == bs.PLACE, "Enter ohne volle Flotte startet nicht")
    # Knöpfe
    mouse(g, InputEvent.MOUSEDOWN, g.btn_rects["clear"].center)
    check(not sea.ships, "Knopf 'Alle entfernen' leert das Brett")
    mouse(g, InputEvent.MOUSEDOWN, g.btn_rects["ready"].center)
    check(g.state == bs.PLACE, "'Los' ohne Schiffe bleibt im Aufstellen")
    mouse(g, InputEvent.MOUSEDOWN, g.btn_rects["random"].center)
    check(sea.complete() and sea.valid_layout(True), "Knopf 'Zufällig' stellt eine legale Flotte auf")
    key(g, "x")
    check(sea.complete(), "Taste X würfelt ebenfalls")
    mouse(g, InputEvent.MOUSEDOWN, g.btn_rects["ready"].center)
    check(g.state == bs.PLAY and g.seas[1].complete() and g.seas[1].valid_layout(True),
          "'Los' startet das Gefecht, die KI-Flotte steht legal")
    # Die Aufstellung bleibt als Vorschlag für die nächste Runde
    before = layout_of(sea)
    g._finish(0)
    key(g, "Return")
    check(g.state == bs.PLACE and layout_of(g.seas[0]) == before,
          "neue Runde schlägt die letzte Aufstellung wieder vor")


# ----------------------------------------------------------- (3) Schießen
def fixed_sea():
    """Flotte in den Zeilen 0, 2, 4, 6, 8 ab Spalte 0 (waagerecht)."""
    sea = core.Sea()
    for idx in range(5):
        sea.place(idx, idx * 2, 0, True)
    return sea


def battle_game(rules=None, mode="single"):
    g = new_game(mode=mode)
    for k, v in (rules or {}).items():
        g.rules[k] = v
    g._new_round()
    g.seas = [fixed_sea(), fixed_sea()]
    return g


def shoot(g, r, c):
    ok = g._fire(r, c)
    run(g, 0.6)                 # Granate fliegt, Einschlag
    return ok


def audit_shooting():
    print("\n(3) Schießen, Salve, Extraschuss, Sieg")
    sea = fixed_sea()
    check(sea.fire(1, 0) == ("miss", None), "Schuss ins Wasser = miss")
    res, ship = sea.fire(8, 0)
    check(res == "hit" and ship.idx == 4, "Treffer auf den Zerstörer = hit")
    check(sea.fire(8, 0) == (None, None), "gleiches Feld zweimal wird abgewiesen")
    res, ship = sea.fire(8, 1)
    check(res == "sunk" and ship.sunk and sea.ships_left() == 4, "zweiter Treffer versenkt den Zerstörer")
    grid, sunk, remaining = sea.knowledge()
    check(sorted(remaining) == [3, 3, 4, 5] and sunk == [[80, 81]] and grid[10] == core.MISS,
          "knowledge(): versenkte Lage + Restlängen, sonst nur Treffer/Wasser")

    g = battle_game({"salvo": False, "extra_shot": False})
    g._begin_turn(0)
    check(g.shots_left == 1, "normal: ein Schuss pro Zug")
    shoot(g, 0, 0)
    check(g.after == "end", "normal: nach einem Treffer ist der Zug trotzdem vorbei")
    run(g, 1.2)
    check(g.turn == 1, "danach ist die KI am Zug")
    g._begin_turn(0)
    check(not g._fire(0, 0) and g.flight is None, "bereits beschossenes Feld verbraucht keinen Schuss")

    g = battle_game({"salvo": True, "extra_shot": False})
    g._begin_turn(0)
    check(g.shots_left == 5, "Salve: 5 eigene Schiffe = 5 Schüsse")
    shoot(g, 1, 5)
    check(g.shots_left == 4 and g.after == "next" and g.turn == 0, "Salve: nach 1 Schuss bleiben 4")
    for c in range(6, 10):
        shoot(g, 1, c)
    check(g.shots_left == 0 and g.after == "end", "Salve: nach 5 Schüssen endet der Zug")
    g.seas[0].fire(8, 0)
    g.seas[0].fire(8, 1)
    g.seas[0].fire(6, 0)
    g.seas[0].fire(6, 1)
    g.seas[0].fire(6, 2)
    g._begin_turn(0)
    check(g.shots_left == 3, "Salve: nach zwei versenkten eigenen Schiffen nur noch 3 Schüsse")

    g = battle_game({"salvo": False, "extra_shot": True})
    g._begin_turn(0)
    shoot(g, 0, 0)
    check(g.shots_left == 1 and g.after == "next" and g.turn == 0,
          "Extraschuss: Treffer gibt einen neuen Schuss")
    shoot(g, 1, 9)
    check(g.shots_left == 0 and g.after == "end", "Extraschuss: Wasser beendet den Zug")

    g = battle_game({"salvo": True, "extra_shot": True})
    g._begin_turn(0)
    shoot(g, 0, 0)
    shoot(g, 0, 1)
    check(g.shots_left == 5, "Salve + Extraschuss: Treffer erstatten den Schuss")

    # Sieg gegen die KI (schwer, ohne eigenen Verlust)
    g = battle_game({"salvo": False, "extra_shot": True})
    g.diff = core.HARD
    g._begin_turn(0)
    for (r, c) in [(i * 2, c) for i in range(5) for c in range(core.FLEET[i])]:
        shoot(g, r, c)
    check(g.after == "win", "letztes Schiff versenkt -> Sieg steht an")
    run(g, 2.0)
    check(g.state == bs.OVER and g.game_over and g.winner == 0, "Rundenende nach der Versenkt-Animation")
    check(g.score == 1 and g.wins == [1, 0] and g.results == [True],
          "Sieg: Punkte = Siege, report_result(True)")
    check("bs_flawless" in g.events and "bs_hard" in g.events,
          "Erfolge: bs_flawless und bs_hard ausgelöst", str(g.events))
    check(g.stats[0]["shots"] == 17 and g.stats[0]["hits"] == 17, "Statistik zählt Schüsse und Treffer")
    # Niederlage bzw. Sieg mit Verlust
    g = battle_game()
    g.diff = core.MEDIUM
    for (r, c) in [(8, 0), (8, 1)]:
        g.seas[0].fire(r, c)
    g._begin_turn(0)
    g._finish(0)
    check("bs_flawless" not in g.events and "bs_hard" not in g.events,
          "kein bs_flawless nach verlorenem Schiff, kein bs_hard auf Mittel")
    g = battle_game()
    g._finish(1)
    check(g.results == [False] and g.score == 0, "Niederlage: report_result(False), keine Punkte")
    key(g, "s")
    check(g.state == bs.SETUP and not g.game_over, "S führt nach Rundenende zurück ins Setup")
    gm = battle_game(mode="multi")
    gm._finish(1)
    check(gm.score == 0 and gm.results == [] and not gm.show_highscore_banner,
          "2 Spieler: keine Wertung, kein Highscore-Banner")


# ------------------------------------------------------------------ (4) KI
def audit_ai():
    print("\n(4) KI: Schusszahlen")
    games = 160
    for touch in (True, False):
        avgs = []
        for level, name in ((core.EASY, "leicht"), (core.MEDIUM, "mittel"), (core.HARD, "schwer")):
            rng = random.Random(4000 + level * 17 + touch)
            res = [core.play_out(level, touch, rng) for _ in range(games)]
            avg = sum(res) / len(res)
            avgs.append(avg)
            check(max(res) <= 100, "%s (Berühren %s): alle %d Flotten in <= 100 Schüssen versenkt"
                  % (name, "an" if touch else "aus", games), "max=%d" % max(res))
            print("         Durchschnitt %.1f  (min %d, max %d)" % (avg, min(res), max(res)))
        check(avgs[2] + 5 < avgs[1] < avgs[0] - 5,
              "Berühren %s: schwer < mittel < leicht (%.1f / %.1f / %.1f)"
              % ("an" if touch else "aus", avgs[2], avgs[1], avgs[0]))


def audit_ai_fair():
    print("\n(5) KI: nur öffentliches Wissen, KI-Zug im Spiel")
    rng = random.Random(77)
    same = True
    for trial in range(60):
        a = core.Sea()
        a.randomize(True, rng)
        shots = rng.sample(range(100), 25)
        # zweite Flotte mit denselben sichtbaren Informationen, aber anderer
        # Lage der unversehrten Schiffe
        grid_a = None
        for i in shots:
            a.fire(*divmod(i, 10))
        grid_a, sunk_a, rem_a = a.knowledge()
        for level in (0, 1, 2):
            ai1 = core.ShotAI(level, True, random.Random(trial))
            ai2 = core.ShotAI(level, True, random.Random(trial))
            p1 = ai1.choose(a)
            p2 = ai2.choose_from(list(grid_a), [list(x) for x in sunk_a], list(rem_a))
            same &= (p1 == p2)
    check(same, "ShotAI.choose hängt nur von knowledge() ab")

    g = battle_game({"salvo": True})
    g._begin_turn(1)
    fired_before = g.stats[1]["shots"]
    steps = 0
    while g.turn == 1 and steps < 60 * 30:
        g.update(1 / 60.0)
        steps += 1
    check(g.turn == 0 and g.stats[1]["shots"] - fired_before == 5,
          "KI-Zug mit Salve feuert genau 5 Schüsse und gibt ab", "shots=%d" % (g.stats[1]["shots"] - fired_before))
    check(steps / 60.0 > 5 * 0.4, "KI-Züge haben eine sichtbare Denkpause (%.1f s)" % (steps / 60.0))


# ------------------------------------------------------------- (6) Hotseat
def audit_hotseat():
    print("\n(6) 2 Spieler: Übergabe verdeckt die Flotten")
    g = new_game(mode="multi")
    check(g.state == bs.SETUP and not g.diff_rects, "Mehrspieler-Setup ohne KI-Stärke")
    key(g, "Return")
    check(g.state == bs.HANDOVER and g.handover["to"] == 0 and g.handover["kind"] == "place",
          "Start -> Übergabe an Spieler 1 zum Aufstellen")
    key(g, "Return")
    check(g.state == bs.HANDOVER, "sofortiges Enter bestätigt noch nicht (Schutzpause)")
    run(g, 0.5)
    key(g, "Return")
    check(g.state == bs.PLACE and g.place_player == 0, "nach der Pause: Spieler 1 stellt auf")
    mouse(g, InputEvent.MOUSEDOWN, g.btn_rects["random"].center)
    mouse(g, InputEvent.MOUSEDOWN, g.btn_rects["ready"].center)
    check(g.state == bs.HANDOVER and g.handover["to"] == 1 and g.handover["kind"] == "place",
          "Spieler 1 fertig -> Übergabe an Spieler 2")

    # Zeichnet die Übergabe irgendetwas von den Flotten?
    calls = []
    orig = (bs.BattleshipGame._draw_sea, bs.BattleshipGame._draw_place, bs.BattleshipGame._ship_sprite,
            bs.BattleshipGame._draw_fleet)
    bs.BattleshipGame._draw_sea = lambda self, *a, **k: calls.append("sea")
    bs.BattleshipGame._draw_place = lambda self, *a, **k: calls.append("place")
    bs.BattleshipGame._draw_fleet = lambda self, *a, **k: calls.append("fleet")

    def spy_sprite(self, idx, cell, horiz, style="normal"):
        calls.append(("sprite", horiz))
        return orig[2](self, idx, cell, horiz, style)
    bs.BattleshipGame._ship_sprite = spy_sprite
    g.draw()
    try:
        leaked = [c for c in calls if c != ("sprite", True)]
        check(not leaked and len(calls) <= 1, "Übergabe zeichnet weder Bretter noch Flotten", str(calls))
    finally:
        (bs.BattleshipGame._draw_sea, bs.BattleshipGame._draw_place, bs.BattleshipGame._ship_sprite,
         bs.BattleshipGame._draw_fleet) = orig

    # Pixelvergleich: zwei völlig verschiedene Partien -> identisches Übergabebild
    bg_orig, now_orig = ui.draw_background, bs.BattleshipGame._now
    ui.draw_background = lambda surf, w, h, *a, **k: surf.fill((0, 0, 0))
    bs.BattleshipGame._now = lambda self: 12.5
    try:
        imgs = []
        for seed in (1, 2):
            h = new_game(mode="multi")
            h._start_round()
            for p in (0, 1):
                h.seas[p].randomize(bool(seed % 2), random.Random(seed * 10 + p))
                for i in random.Random(seed).sample(range(100), 40):
                    h.seas[p].fire(*divmod(i, 10))
            h._handover(1, "fire", report=dict(shots=1, hits=0))
            h.handover["t"] = 1.0
            h.draw()
            imgs.append(pygame.image.tobytes(h.surface, "RGB"))
        check(imgs[0] == imgs[1], "Übergabebild verrät nichts (zwei Partien, gleiche Pixel)")
    finally:
        ui.draw_background, bs.BattleshipGame._now = bg_orig, now_orig

    run(g, 0.5)
    key(g, "space")
    check(g.state == bs.PLACE and g.place_player == 1, "Spieler 2 stellt auf")
    key(g, "x")
    key(g, "Return")
    check(g.state == bs.HANDOVER and g.handover["kind"] == "fire" and g.handover["to"] == g.starter,
          "beide fertig -> Übergabe zum ersten Schuss")
    run(g, 0.5)
    mouse(g, InputEvent.MOUSEDOWN, (5, 5))
    check(g.state == bs.PLAY and g.turn == 0 and g._viewer() == 0, "Spieler 1 feuert, sieht seine Sicht")
    free = next((r, c) for r in range(10) for c in range(10) if g.seas[1].ship_at(r, c) is None)
    mouse(g, InputEvent.MOUSEMOVE, cell_pos(g, *free))
    mouse(g, InputEvent.MOUSEDOWN, cell_pos(g, *free))
    run(g, 2.0)
    check(g.state == bs.HANDOVER and g.handover["to"] == 1 and g.handover["report"] == dict(shots=1, hits=0),
          "nach dem Zug: Übergabe an Spieler 2 mit Bericht")
    run(g, 0.5)
    key(g, "Return")
    check(g.state == bs.PLAY and g.turn == 1 and g._viewer() == 1, "Spieler 2 ist dran, Ansicht gewechselt")


# ---------------------------------------------------------- (7) Persistenz
def audit_persistence():
    print("\n(7) Einstellungen")
    gs = json.loads(json.dumps(GS))
    g = new_game(settings=gs)
    del SAVED[:]
    key(g, "3")
    mouse(g, InputEvent.MOUSEDOWN, g.rule_rects[0].center)
    mouse(g, InputEvent.MOUSEDOWN, g.rule_rects[1].center)
    mouse(g, InputEvent.MOUSEDOWN, g.rule_rects[2].center)
    check(SAVED and SAVED[-1] == {"difficulty": 2, "touch": False, "salvo": True, "extra_shot": True},
          "Setup speichert jede Änderung sofort", str(SAVED[-1:] or None))
    g2 = new_game(settings=gs)
    check(g2.diff == 2 and g2.rules == {"touch": False, "salvo": True, "extra_shot": True},
          "neue Partie liest die gespeicherten Regeln")
    merged = settings_mod._merge_defaults({"battleship": {"difficulty": 7, "touch": "ja",
                                                          "salvo": True, "extra_shot": 1}})
    check(merged["battleship"] == {"difficulty": 2, "touch": True, "salvo": True, "extra_shot": False},
          "SCHEMAS: Stärke begrenzt, falsche Typen fallen auf Standard", str(merged["battleship"]))
    # Tastatur im Setup
    g3 = new_game(settings=json.loads(json.dumps(GS)))
    g3.setup_sel = 1
    before = g3.rules["touch"]
    key(g3, "space")
    check(g3.rules["touch"] != before, "Leertaste schaltet die fokussierte Regel")
    key(g3, "Up")
    key(g3, "Right")
    check(g3.diff == 2, "Pfeil rechts erhöht die KI-Stärke")


# -------------------------------------------------------------- (8) Layout
def text_fits(g, font, text, maxw):
    """Passt der Text mindestens in der kleinsten Stufe der _fit-Kette?"""
    return g._tiny.size(text)[0] <= maxw if font is not None else True


def inside(rect, w, h):
    return rect.left >= 0 and rect.top >= 0 and rect.right <= w and rect.bottom <= h


def audit_layout():
    print("\n(8) Layout: 5 Auflösungen x 14 Sprachen")
    lang_before = i18n.get_language()
    longest_ship = {}
    for (w, h) in RESOLUTIONS:
        problems = []
        for mode in ("single", "multi"):
            g = new_game(w, h, mode=mode)
            rects = list(g.diff_rects) + list(g.rule_rects) + [g.start_rect]
            if not all(inside(r, w, h - 24) for r in rects):
                problems.append("%s: Setup-Knöpfe außerhalb" % mode)
            if any(a.colliderect(b) for i, a in enumerate(rects) for b in rects[i + 1:]):
                problems.append("%s: Setup-Knöpfe überlappen" % mode)
            top_needed = int(h * 0.11) + g._huge.get_height() // 2 + g._small.get_height() + 6
            first = (g.diff_rects[0] if g.diff_rects else g.rule_rects[0]).top - g._tiny.get_height() - 6
            if first < top_needed:
                problems.append("%s: Setup-Knöpfe rutschen in den Untertitel (%d < %d)"
                                % (mode, first, top_needed))
        g = new_game(w, h)
        g._start_round()
        board = pygame.Rect(g.bx - g.lab, g.by - g.lab, g.lab + 10 * g.cb, g.lab + 10 * g.cb)
        own = pygame.Rect(g.sx, g.sy, 10 * g.cs, 10 * g.cs)
        side = list(g.dock_rects) + list(g.btn_rects.values())
        if not inside(board, w, h) or not inside(own, w, h):
            problems.append("Bretter außerhalb")
        if g.status_y + g._small.get_height() // 2 > h:
            problems.append("Statuszeile außerhalb")
        if any(r.colliderect(board) or not inside(r, w, h) for r in side):
            problems.append("Dock/Knöpfe außerhalb oder über dem Brett")
        if any(a.colliderect(b) for i, a in enumerate(side) for b in side[i + 1:]):
            problems.append("Dock/Knöpfe überlappen")
        if g.btn_rects["ready"].bottom > g.by + 10 * g.cb:
            problems.append("Knöpfe ragen unter das Brett")
        for fr in g.fleet_rects:
            if fr.colliderect(board) or fr.colliderect(own) or not inside(fr, w, h):
                problems.append("Flottenübersicht kollidiert")
        if g.fleet_rects[0].colliderect(g.fleet_rects[1]):
            problems.append("Flottenübersichten überlappen")
        if g.cs < 12 or g.cb < 20:
            problems.append("Felder zu klein (%d/%d)" % (g.cb, g.cs))
        over = g._over_panel_rect()
        if not inside(over, w, h - 50) or over.top < g.hud_h:
            problems.append("Ergebnis-Panel kollidiert mit HUD/Highscore-Banner")
        check(not problems, "%4dx%d: Geometrie" % (w, h), "; ".join(problems))

        too_long = []
        for code in LANGS:
            i18n.set_language(code, persist=False)
            for mode in ("single", "multi"):
                gm = new_game(w, h, mode=mode)
                # Setup
                if gm._tiny.size(i18n.t("bs.setup_hint"))[0] > w - 20:
                    too_long.append("%s/setup_hint" % code)
                if gm._small.size(i18n.t("bs.subtitle"))[0] > w - 30:
                    too_long.append("%s/subtitle" % code)
                if gm._bold.size(gm.name.upper())[0] > w - 30:
                    too_long.append("%s/title" % code)
                for i, rc in enumerate(gm.rule_rects):
                    isz = max(10, int(rc.h * 0.5))
                    pill_w, _ = gm._pill_size(rc.h)
                    room = rc.w - 10 - pill_w - (20 + isz) - 8
                    if gm._small.size(i18n.t(bs.RULE_TEXT[bs.RULES[i]]))[0] > room:
                        too_long.append("%s/%s" % (code, bs.RULES[i]))
                for i, rc in enumerate(gm.diff_rects):
                    if gm._small.size(i18n.t("bs.diff." + bs.DIFFS[i]))[0] > rc.w - 10:
                        too_long.append("%s/diff" % code)
                gm._layout()
                # HUD: Mitte darf die Seiten nicht berühren
                left = gm._small.size("%s: %d" % (gm._pname(0), 12))[0]
                right = gm._small.size("%d :%s" % (12, gm._pname(1)))[0]
                room = w - 2 * max(left, right) - 48
                mids = [i18n.t("bs.place_title"), i18n.t("bs.your_turn"), i18n.t("bs.ai_thinks")] \
                    if mode == "single" else \
                    [i18n.t("bs.place_title_p", name=gm._pname(1)), i18n.t("bs.turn", name=gm._pname(1))]
                for m in mids:
                    if gm._tiny.size(m)[0] > room:
                        too_long.append("%s/hud:%s" % (code, m))
                # Aufstellen
                full = gm.col_x + gm.col_w - gm.bx
                for k in ("bs.place_hint", "bs.place_invalid", "bs.place_all"):
                    if gm._tiny.size(i18n.t(k))[0] > full:
                        too_long.append("%s/%s" % (code, k))
                for idx, sk in enumerate(core.SHIP_KEYS):
                    name = i18n.t("bs.ship." + sk)
                    if gm._tiny.size(i18n.t("bs.holding", ship=name))[0] > full:
                        too_long.append("%s/holding" % code)
                    room = gm.col_w - (14 + 5 * gm.dock_cell) - 8
                    # 480x360: lange Namen dürfen auf "x5" ausweichen (Name
                    # steht dann beim Überfahren in der Statuszeile)
                    if gm._tiny.size(name)[0] > room and (w > 480 or gm._tiny.size("×5")[0] > room):
                        too_long.append("%s/dock:%s" % (code, sk))
                    longest_ship[code] = max(longest_ship.get(code, ""), name, key=len)
                title = i18n.t("bs.fleet_own") + "  5/5"
                if gm._tiny.size(title)[0] > gm.col_w:
                    too_long.append("%s/dock_title" % code)
                for bk, lk in (("rotate", "bs.btn_rotate"), ("random", "bs.btn_random"),
                               ("clear", "bs.btn_clear"), ("ready", "bs.btn_ready")):
                    rc = gm.btn_rects[bk]
                    kw = gm._tiny.size("Enter" if bk == "ready" else "R")[0]
                    if gm._small.size(i18n.t(lk))[0] > rc.w - kw - 24:
                        too_long.append("%s/btn:%s" % (code, bk))
                # Gefecht: Statuszeile unter dem Zielbrett, Flottenköpfe, Beschriftung
                width = 10 * gm.cb
                texts = [i18n.t("bs.fire_hint"), i18n.t("bs.already"), i18n.t("bs.hit_again"),
                         i18n.t("bs.miss")]
                texts += [i18n.t("bs.sunk", ship=i18n.t("bs.ship." + sk)) for sk in core.SHIP_KEYS]
                for tx in texts:
                    pips = 5 * (2 * max(3, gm._small.get_height() // 5) + 4)
                    if gm._tiny.size(tx)[0] > width - pips - 8:
                        too_long.append("%s/status:%s" % (code, tx))
                for tx in (i18n.t("bs.fleet_enemy") + "  5/5", i18n.t("bs.fleet_own") + "  5/5"):
                    if gm._tiny.size(tx)[0] > gm.fleet_rects[0].w:
                        too_long.append("%s/fleet_head" % code)
                cap = i18n.t("bs.your_waters") if mode == "single" else \
                    i18n.t("bs.waters_of", name=gm._pname(1))
                if gm._tiny.size(cap)[0] > gm.col_w:
                    too_long.append("%s/caption" % code)
                # Rundenende
                gm.winner = 0
                over = gm._over_panel_rect()
                colw = max(gm._small.size(gm._pname(0))[0], gm._small.size(gm._pname(1))[0],
                           gm._small.size("100%")[0]) + 12
                label_room = (over.right - 18 - colw) - colw - (over.left + 18) - 6
                for (label, a, b) in gm._over_rows():
                    if gm._tiny.size(label)[0] > label_room:
                        too_long.append("%s/over:%s" % (code, label))
                heads = [i18n.t("common.player_wins", n=2)] if mode == "multi" else \
                    [i18n.t("bs.win_you"), i18n.t("bs.win_ai")]
                for hd in heads:
                    if gm._small.size(hd)[0] > over.w - 24:
                        too_long.append("%s/over_head" % code)
                if gm._tiny.size(i18n.t("bs.new_round"))[0] > over.w - 20:
                    too_long.append("%s/new_round" % code)
                # Übergabe
                if mode == "multi":
                    for tx in (i18n.t("bs.handover_title", name=gm._pname(1)), i18n.t("bs.handover_place"),
                               i18n.t("bs.handover_fire"), i18n.t("bs.handover_away", name=gm._pname(0)),
                               i18n.t("bs.handover_report", shots=5, hits=5), i18n.t("bs.handover_ready")):
                        if gm._small.size(tx)[0] > w - 40:
                            too_long.append("%s/handover:%s" % (code, tx))
        i18n.set_language(lang_before, persist=False)
        check(not too_long, "%4dx%d: alle Texte passen (14 Sprachen)" % (w, h),
              ", ".join(sorted(set(too_long))[:12]))


def all_states(g):
    """Zeichnet jeden Zustand einmal (mit Effekten); liefert Fehlertexte."""
    errs = []

    def step(label, fn):
        try:
            fn()
            g.draw()
        except Exception as exc:          # pragma: no cover - Diagnose
            errs.append("%s: %r" % (label, exc))

    step("setup", lambda: None)
    step("place", g._start_round)

    def hold():
        g.seas[0].clear()
        g.seas[0].place(0, 0, 0, True)
        g._pick(2, 1)
        g.mouse = (g.bx + 3 * g.cb, g.by + 4 * g.cb)
    step("place_hold", hold)

    def off_board():
        g.mouse = (g.width - 5, g.height - 5)
    step("place_hold_off", off_board)

    def kb():
        g.held = None
        g.kb_mode = True
        g.rules["touch"] = False
    step("place_kb", kb)

    def battle():
        g.rules["touch"] = True
        g.seas[0].clear()
        g.seas[0].randomize(True, random.Random(3))
        g._ready()
        rng = random.Random(5)
        for _ in range(40):
            g.seas[1].fire(*core.ShotAI(2, True, rng).choose(g.seas[1]))
            g.seas[0].fire(*core.ShotAI(1, True, rng).choose(g.seas[0]))
        g._begin_turn(0)
        x, y, cell = g._board_geom(1)
        g._fx_explosion(x + cell, y + cell, cell, big=True)
        g._fx_splash(x + 3 * cell, y + 3 * cell, cell)
        g._last_draw = g._now() - 0.03
    step("battle", battle)

    def flight():
        g._fire(*next((r, c) for r in range(10) for c in range(10)
                      if g.seas[1].shots[r][c] == core.UNKNOWN))
        g.update(0.2)
    step("flight", flight)
    step("impact", lambda: g.update(0.4))

    def ai():
        g.flight = None
        g.after = None
        g._begin_turn(1)
        g.update(0.3)
    step("ai", ai)

    def over():
        for s in g.seas[1].ships:
            for rc in s.cells:
                g.seas[1].fire(*rc)
        g.turn = 0
        g._finish(0)
    step("over", over)
    return errs


def audit_draw():
    print("\n(8b) Zeichnen: alle Zustände, Themes, Auflösungswechsel")
    lang_before = i18n.get_language()
    theme_before = ui.theme_name()
    for theme in ("v42", "v1"):
        ui.set_theme(theme)
        errs = []
        for (w, h) in RESOLUTIONS:
            for code in (LANGS if (w, h) in ((480, 360), (1280, 960)) else ["de", "fi"]):
                i18n.set_language(code, persist=False)
                for mode in ("single", "multi"):
                    g = new_game(w, h, mode=mode)
                    errs += ["%dx%d/%s/%s/%s" % (w, h, code, mode, e) for e in all_states(g)]
                    if mode == "multi":
                        g._handover(1, "fire", report=dict(shots=2, hits=1))
                        g.handover["t"] = 1.0
                        try:
                            g.draw()
                        except Exception as exc:
                            errs.append("%dx%d/%s/handover: %r" % (w, h, code, exc))
        i18n.set_language(lang_before, persist=False)
        check(not errs, "Theme %s: alle Zustände zeichnen fehlerfrei" % theme, "; ".join(errs[:4]))
    ui.set_theme(theme_before)
    # Auflösungswechsel mitten im Gefecht und beim Aufstellen
    errs = []
    g = new_game(640, 480)
    g._start_round()
    g._pick(0, 2)
    for (w, h) in ((1280, 960), (480, 360), (800, 600)):
        g.surface = pygame.Surface((w, h))
        g.width, g.height = w, h
        g.on_surface_changed()
        try:
            g.draw()
        except Exception as exc:
            errs.append("place %dx%d: %r" % (w, h, exc))
    g.held = None
    g.seas[0].randomize(True)
    g._ready()
    g._fire(4, 4)
    for (w, h) in ((960, 720), (480, 360)):
        g.surface = pygame.Surface((w, h))
        g.width, g.height = w, h
        g.on_surface_changed()
        try:
            g.update(0.5)
            g.draw()
        except Exception as exc:
            errs.append("battle %dx%d: %r" % (w, h, exc))
    check(not errs and g.cb * 10 <= 480, "on_surface_changed: Aufstellen und Gefecht bauen sauber neu",
          "; ".join(errs))


# ------------------------------------------------------------- (9) Tempo
def audit_perf():
    print("\n(9) Tempo")
    g = new_game(1280, 960)
    g._start_round()
    g.seas[0].randomize(True, random.Random(1))
    g._ready()
    rng = random.Random(2)
    for _ in range(55):
        g.seas[1].fire(*core.ShotAI(1, True, rng).choose(g.seas[1]))
        g.seas[0].fire(*core.ShotAI(1, True, rng).choose(g.seas[0]))
    g._begin_turn(0)
    g.draw()
    t0 = time.perf_counter()
    frames = 90
    for i in range(frames):
        if i % 15 == 0:
            x, y, cell = g._board_geom(1)
            g._fx_explosion(x + 5 * cell, y + 5 * cell, cell, big=True)
            g._fx_splash(x + 2 * cell, y + 7 * cell, cell)
        g._last_draw = g._now() - 1 / 60.0
        g.draw()
    ms = (time.perf_counter() - t0) / frames * 1000
    check(ms < 9.0, "1280x960 Gefecht mit Effekten: %.2f ms pro Frame (< 9 ms)" % ms)
    g._start_round()
    t0 = time.perf_counter()
    for _ in range(60):
        g.draw()
    ms = (time.perf_counter() - t0) / 60 * 1000
    check(ms < 9.0, "1280x960 Aufstellen: %.2f ms pro Frame (< 9 ms)" % ms)
    ai = core.ShotAI(core.HARD, False, random.Random(3))
    sea = core.Sea()
    sea.randomize(False, random.Random(4))
    t0 = time.perf_counter()
    n = 0
    while not sea.all_sunk():
        sea.fire(*ai.choose(sea))
        n += 1
    ms = (time.perf_counter() - t0) / n * 1000
    check(ms < 5.0, "schwere KI: %.2f ms pro Schuss (< 5 ms)" % ms)


if __name__ == "__main__":
    t0 = time.time()
    audit_rules()
    audit_placement_input()
    audit_shooting()
    audit_ai()
    audit_ai_fair()
    audit_hotseat()
    audit_persistence()
    audit_layout()
    audit_draw()
    audit_perf()
    try:
        os.remove(store._PATH)
    except OSError:
        pass
    print("\n%s  (%.1f s)" % ("ALLE PRÜFUNGEN BESTANDEN" if not FAILS
                              else "%d FEHLER: %s" % (len(FAILS), FAILS),
                              time.time() - t0))
    sys.exit(1 if FAILS else 0)
