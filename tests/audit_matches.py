# -*- coding: utf-8 -*-
"""Headless-Audit der Match-Historie (matches.py, matchview.py, Spiel-Hooks).

Geprüft wird:

Daten     (1) Aufzeichnen, neueste zuerst, Kappe CAP je Spiel bei unbegrenzten
              Summen, Sieg/Niederlage/Remis, Bestwert, Schnitt, Spielzeit,
              "Ersetzen" (Rückgängig nach Game Over), Profile getrennt,
              Profil löschen, Datei-Rundlauf (kompakt, eine Zeile), kaputte
              Einträge werden bereinigt, Gleitkommazahlen kommen nicht vor.
Hooks     (2) main.App._match_speichern: Game Over -> genau EIN Eintrag (Latch),
              Rekord-Flag, Verlassen mitten im Spiel -> "abgebrochen" nur mit
              Punkten oder >= MIN_ABORT_SECONDS, match_manual-Spiele werden
              übersprungen, Ergebnis aus report_result/report_duel.
          (3) Spiele mit eigenen Matches: Blackjack, Video-Poker, Roulette,
              Slots, Lama Crossing, Labyrinth, Geometry Dash - je Runde ein
              Eintrag mit Einsatz/Auszahlung bzw. Level-Infos; alle 7
              Chip-Spiele + Geodash + Labyrinth setzen match_manual.
Anzeige   (4) Vorspiel-Screen hat den Knopf "Matches" und öffnet den Screen;
              MatchScreen zeichnet für ALLE Spiele in 5 Auflösungen x
              14 Sprachen fehlerfrei, Reiter/Summen/Zeilen im Bild und ohne
              Überlappung; Filter, Sortierung, Bildlauf, Reiterwechsel;
              alle Info-Schlüssel haben Texte in allen Sprachen.

Aufruf aus dem Repo-Root:  python tests/audit_matches.py
Exit-Code 0 = alle Prüfungen bestanden.
"""
import json
import os
import sys
import tempfile
import time

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
os.environ.setdefault("SDL_AUDIODRIVER", "dummy")
try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
except (AttributeError, OSError):
    pass

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, REPO)
os.chdir(REPO)

import pygame
pygame.init()
pygame.display.set_mode((640, 480))

import store
store._DIR = tempfile.mkdtemp(prefix="pgz-audit-matches-")
store._PATH = os.path.join(store._DIR, "mem.json")
store._LEGACY_HS_PATH = os.path.join(store._DIR, "highscores.json")
import replay
replay._PATH = os.path.join(store._DIR, "replay.json")
import ugc
ugc._PATH = os.path.join(store._DIR, "ugc.json")
import settings as settings_mod
settings_mod.save_settings = lambda s: None
import i18n
i18n.init()
import ui
import matches
import matchview
from game_base import Game, InputEvent
from games import ALL_GAMES

GS = json.loads(json.dumps(settings_mod.DEFAULTS))
FAILS = []
RES = ((480, 360), (640, 480), (800, 600), (960, 720), (1280, 960))
LANGS = [e[0] if isinstance(e, (tuple, list)) else e for e in i18n.AVAILABLE]


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + str(detail)) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


def quiet(game):
    game.play_sound = lambda name: None
    game.rumble = lambda ms=0: None
    game.ach_event = lambda *a, **k: None
    return game


def fresh():
    """Leere Historie (Datei + Speicher)."""
    for name in (matches.FILENAME, matches.FILENAME + ".bak"):
        p = store.path_for(name)
        if os.path.exists(p):
            os.remove(p)
    matches._doc = None
    matches._dirty = False


def cls_by_key(key):
    for c in ALL_GAMES:
        if c.highscore_key == key:
            return c
    raise KeyError(key)


def floats_in(v):
    if isinstance(v, float):
        return True
    if isinstance(v, list):
        return any(floats_in(x) for x in v)
    if isinstance(v, dict):
        return any(floats_in(x) for x in v.values())
    return False


# ---------------------------------------------------------------- (1) Daten

def audit_data():
    print("\nDaten (matches.py)")
    fresh()
    matches.record("snake", "single", 10, None, 30)
    matches.record("snake", "single", 50, None, 12, matches.F_RECORD)
    matches.record("snake", "single", 20, None, 7)
    e = matches.entries("snake")
    check([x[2] for x in e] == [20, 50, 10], "neueste zuerst", [x[2] for x in e])
    tot = matches.totals("snake")
    check(tot["n"] == 3 and tot["best"] == 50 and tot["sum"] == 80 and tot["time"] == 49
          and tot["stored"] == 3, "Summen (n, best, sum, time, stored)", tot)
    check(len(e[0]) == 7 and len(e[0][0]) == 16, "Eintrag = 7 Felder, Zeit YYYY-MM-DD HH:MM", e[0])
    # Ergebnisse
    matches.record("chess", "single", 0, matches.WIN, 100)
    matches.record("chess", "single", 0, matches.LOSS, 100)
    matches.record("chess", "single", 0, matches.DRAW, 100)
    matches.record("chess", "single", 0, True, 100)          # bool ist kein Ergebnis
    tot = matches.totals("chess")
    check((tot["w"], tot["l"], tot["d"], tot["n"]) == (1, 1, 1, 4), "Sieg/Niederlage/Remis gezählt", tot)
    check(matches.entries("chess")[0][3] is None, "bool als Ergebnis wird verworfen")
    # Kappe
    for i in range(matches.CAP + 15):
        matches.record("tetris", "solo", i, None, 1)
    tot = matches.totals("tetris")
    e = matches.entries("tetris")
    check(len(e) == matches.CAP and tot["n"] == matches.CAP + 15 and e[0][2] == matches.CAP + 14
          and e[-1][2] == 15, "Kappe %d je Spiel, Summen laufen weiter" % matches.CAP,
          (len(e), tot["n"]))
    check(matches.recent("tetris", 10)[0][2] == matches.CAP + 14 and len(matches.recent("tetris")) == 10,
          "recent() = letzte 10")
    # Ersetzen
    matches.record("game2048", "single", 100, None, 20)
    matches.record("game2048", "single", 300, None, 15, replace=True)
    e = matches.entries("game2048")
    tot = matches.totals("game2048")
    check(len(e) == 1 and e[0][2] == 300 and e[0][4] == 35 and tot["n"] == 1 and tot["sum"] == 300,
          "replace: altes Match raus, Spielzeit addiert", (e, tot))
    # Info bereinigen
    matches.record("crossy", "daily", 5, None, 3, matches.F_DAILY,
                   {"c": 2, "ch": "fox", "x": 1.5, "zu_langer_schluessel": 1, "ok": True})
    info = matches.entries("crossy")[0][6]
    check(info == {"c": 2, "ch": "fox", "ok": True}, "Info: nur int/str/bool, kurze Schlüssel", info)
    # Modi
    matches.record("crossy", "endless", 7, None, 3)
    check(matches.modes_of("crossy") == ["daily", "endless"], "modes_of in Reihenfolge der Sichtung")
    # Datei-Rundlauf
    matches.flush()
    path = store.path_for(matches.FILENAME)
    with open(path, encoding="utf-8") as f:
        text = f.read()
    raw = json.loads(text)
    check("\n" not in text.strip() and raw.get("format") == matches.FORMAT and raw.get("v") == 1,
          "Datei kompakt in einer Zeile, Kopf format/v", text[:80])
    check(not floats_in(raw), "keine Gleitkommazahlen in der Datei")
    before = {k: matches.entries(k) for k in ("snake", "chess", "tetris", "crossy")}
    matches._doc = None
    after = {k: matches.entries(k) for k in ("snake", "chess", "tetris", "crossy")}
    check(before == after, "Rundlauf: Laden == Speichern")
    # Kaputte Datei
    raw["profiles"][store.active_id()]["snake"]["list"].append(["x"])
    raw["profiles"][store.active_id()]["snake"]["list"].append(
        ["2026-01-01 10:00", 3, "viel", 7, -5, 999, [1]])
    raw["profiles"][store.active_id()]["snake"]["n"] = "drei"
    with open(path, "w", encoding="utf-8") as f:
        json.dump(raw, f)
    matches._doc = None
    e = matches.entries("snake")
    check(len(e) == 4 and e[0] == ["2026-01-01 10:00", "", 0, None, 0, 255, None]
          and matches.totals("snake")["n"] == 0, "kaputte Einträge bereinigt", e[:2])
    matches._dirty = True
    matches.flush()                      # vorige Fassung wird zur .bak
    with open(path, "w", encoding="utf-8") as f:
        f.write("{kaputt")
    matches._doc = None
    check(matches.entries("tetris") != [], "kaputte Datei -> Sicherung .bak wird gelesen")
    # Profile
    fresh()
    matches.record("snake", "single", 9, None, 1)
    matches.record("snake", "single", 4, None, 1, pid="p_other")
    check(matches.totals("snake")["n"] == 1 and matches.totals("snake", "p_other")["n"] == 1,
          "Profile getrennt")
    matches.drop_profile("p_other")
    check(matches.totals("snake", "p_other")["n"] == 0, "drop_profile löscht die Historie")
    check(matches.format_duration(3725) == "1:02:05" and matches.format_duration(65) == "1:05",
          "format_duration")


# ---------------------------------------------------------------- (2) main-Hook

class _HookApp:
    """Nur die Methode, die geprüft wird (main.App._match_speichern)."""


def audit_main_hook():
    print("\nmain.App._match_speichern")
    import main
    fresh()
    surf = pygame.Surface((640, 480))
    hook = main.App._match_speichern
    Snake = cls_by_key("snake")
    g = quiet(Snake(surf, 640, 480, mode="single", game_settings=GS))
    g.score = 42
    g.game_over = True
    g._match_time = 33.9
    hook(_HookApp(), g, True, False)
    hook(_HookApp(), g, True, True)                # zum_menü danach: kein zweiter Eintrag
    e = matches.entries("snake")
    check(len(e) == 1 and e[0][2] == 42 and e[0][4] == 33 and e[0][5] & matches.F_RECORD,
          "Game Over -> genau ein Eintrag mit Rekord-Flag, Dauer abgerundet", e)
    # Verlassen ohne Punkte, kurz -> nichts
    g = quiet(Snake(surf, 640, 480, mode="single", game_settings=GS))
    g.score = 0
    g._match_time = 2.0
    hook(_HookApp(), g, False, True)
    check(len(matches.entries("snake")) == 1, "kurz verlassen ohne Punkte -> kein Eintrag")
    g._match_time = matches.MIN_ABORT_SECONDS + 0.5
    hook(_HookApp(), g, False, True)
    e = matches.entries("snake")
    check(len(e) == 2 and e[0][5] & matches.F_ABORT, "lange gespielt und verlassen -> abgebrochen", e[0])
    # nicht Game Over und nicht Verlassen -> nichts
    g = quiet(Snake(surf, 640, 480, mode="single", game_settings=GS))
    g.score = 99
    hook(_HookApp(), g, False, False)
    check(len(matches.entries("snake")) == 2, "laufendes Spiel ohne Verlassen -> kein Eintrag")
    # Ergebnis aus report_result
    Connect = cls_by_key("connect4")
    g = quiet(Connect(surf, 640, 480, mode="single", game_settings=GS))
    g.report_result(True)
    g.game_over = True
    hook(_HookApp(), g, False, False)
    check(matches.entries("connect4")[0][3] == matches.WIN, "Ergebnis aus report_result")
    # Duell: Ergebnis + Gegner
    g = quiet(Connect(surf, 640, 480, mode="multi", game_settings=GS))
    g.report_duel(1)
    g.game_over = True
    hook(_HookApp(), g, False, False)
    e = matches.entries("connect4")[0]
    check(e[3] == matches.LOSS and (e[6] or {}).get("vs"), "Duell: Niederlage + Gegnername", e)
    # match_manual wird übersprungen
    BJ = cls_by_key("blackjack")
    g = quiet(BJ(surf, 640, 480, mode="single", game_settings=GS))
    g.score = 1500
    g.game_over = True
    hook(_HookApp(), g, False, True)
    check(matches.entries("blackjack") == [], "match_manual-Spiele: main schreibt nichts")
    # Rückgängig nach Game Over: Eintrag wird ersetzt
    G2048 = cls_by_key("2048")
    g = quiet(G2048(surf, 640, 480, mode="single", game_settings=GS))
    g.score = 100
    g.game_over = True
    hook(_HookApp(), g, False, False)
    g.resume_from_game_over()
    g.score = 180
    g.game_over = True
    hook(_HookApp(), g, False, False)
    e = matches.entries("2048")
    check(len(e) == 1 and e[0][2] == 180, "Rückgängig nach Game Over ersetzt das Match", e)
    # Spiele mit eigenen Setup-Screens: Dauer zählt erst im Spiel
    check(g.match_active in (True, False), "match_active vorhanden")
    for cls in ALL_GAMES:
        inst = cls.__new__(cls)
        inst.state = "setup"
        if type(inst).match_active is Game.match_active:
            if inst.match_active:
                check(False, "%s: Setup zählt als aktives Match" % cls.__name__)
                break
    else:
        check(True, "Setup-Zustand zählt nicht als Match-Dauer")


# ---------------------------------------------------------------- (3) eigene Matches

def audit_manual_games():
    print("\nSpiele mit eigenen Matches (Game.log_match)")
    manual = {c.highscore_key for c in ALL_GAMES if getattr(c, "match_manual", False)}
    want = {"blackjack", "poker", "casino", "lamacrossing", "lamamines", "lamaplinko",
            "lamacrash", "geodash", "maze"}
    check(want <= manual, "match_manual: 7 Chip-Spiele + Geodash + Labyrinth", sorted(want - manual))
    import lamabank
    fresh()
    surf = pygame.Surface((800, 600))
    # Blackjack: zwei Hände, eine gewinnt
    BJ = cls_by_key("blackjack")
    g = quiet(BJ(surf, 800, 600, mode="single", game_settings=GS))
    import games.cards as C
    g.hands = [[C.Card(10, "H"), C.Card(9, "S")], [C.Card(10, "D"), C.Card(5, "C")]]
    g.hand_bets = [20, 20]
    g.dealer = [C.Card(10, "C"), C.Card(8, "H")]
    g._match_time = 12.4
    g._settle(None)
    e = matches.entries("blackjack")
    check(len(e) == 1 and e[0][2] == 0 and e[0][3] == matches.DRAW
          and e[0][6] == {"bet": 40, "win": 40} and e[0][4] == 12,
          "Blackjack: Hand = Match (Einsatz/Auszahlung, Remis bei +-0)", e)
    # Video-Poker
    PK = cls_by_key("poker")
    g = quiet(PK(surf, 800, 600, mode="video", game_settings=GS))
    g.video_bet = 10
    g._video_deal()
    g.holds = set(range(5))
    g._video_draw()
    e = matches.entries("poker")
    check(len(e) == 1 and e[0][6]["bet"] == 10 and e[0][1] == "video", "Video-Poker: Ziehung = Match", e)
    # Roulette
    CA = cls_by_key("casino")
    g = quiet(CA(surf, 800, 600, mode="roulette", game_settings=GS))
    if hasattr(g, "r_bets"):
        g.r_bets = {"red": 10} if not g.r_bets else g.r_bets
        try:
            g._r_spin()
        except Exception as exc:     # noqa: BLE001
            check(False, "Roulette-Dreh", exc)
    e = matches.entries("casino")
    check(len(e) == 1 and e[0][1] == "roulette" and "num" in (e[0][6] or {}),
          "Roulette: Wurf = Match mit gezogener Zahl", e)
    # Slots
    g = quiet(CA(surf, 800, 600, mode="slots", game_settings=GS))
    before = len(matches.entries("casino"))
    try:
        g._s_spin() if hasattr(g, "_s_spin") else None
    except Exception as exc:         # noqa: BLE001
        check(False, "Slots-Dreh", exc)
    e = matches.entries("casino")
    check(len(e) == before + 1 and e[0][1] == "slots", "Slots: Dreh = Match", e[:1])
    # Labyrinth
    MZ = cls_by_key("maze")
    g = quiet(MZ(surf, 800, 600, mode="single", game_settings=GS))
    g._start_level(2)
    g.elapsed = 21.7
    g._finish()
    e = matches.entries("maze")
    check(len(e) == 1 and e[0][3] == matches.WIN and e[0][6]["lv"] == 2 and e[0][6]["tm"] == 21,
          "Labyrinth: gelöstes Level = Match", e)
    # Geometry Dash: Level-Besuch, verlassen nach Versuchen
    GD = cls_by_key("geodash")
    g = quiet(GD(surf, 800, 600, mode="normal", game_settings=GS))
    from games import geodash_core as core
    lv = core.normalize_level(g.levels[0] if hasattr(g, "levels") and g.levels else
                              {"name": "Test", "objects": []})
    try:
        g.start_level(lv, kind="main")
        g._gd_best_pct = 37
        g._leave_to_select()
        e = matches.entries("geodash")
        check(len(e) == 1 and e[0][2] == 37 and e[0][3] == matches.LOSS and e[0][6]["g"] >= 1,
              "Geometry Dash: Level verlassen = Match mit bestem Fortschritt", e)
        g._leave_to_select()
        check(len(matches.entries("geodash")) == 1, "Geometry Dash: nur ein Eintrag je Besuch")
    except Exception as exc:          # noqa: BLE001
        check(False, "Geometry Dash Level-Besuch", repr(exc))
    lamabank.reset_cache()


# ---------------------------------------------------------------- (4) Anzeige

class FakeApp:
    def __init__(self):
        self.settings = json.loads(json.dumps(settings_mod.DEFAULTS))
        self.shown = None

    def launch_game(self, *a, **k):
        pass

    def back_to_menu(self):
        self.shown = "menu"

    def show_screen(self, s, exit_old=True):
        self.shown = s


def overlap(rects):
    return any(rects[i].colliderect(rects[j]) for i in range(len(rects))
               for j in range(i + 1, len(rects)))


def audit_screens():
    print("\nAnzeige (Vorspiel-Knopf, MatchScreen)")
    import menu
    app = FakeApp()
    surf = pygame.Surface((640, 480))
    pg = menu.PreGameScreen(surf, 640, 480, app, cls_by_key("snake"))
    labels = [b[0] for b in pg.buttons]
    i = labels.index(i18n.t("pregame.matches")) if i18n.t("pregame.matches") in labels else -1
    check(i >= 0, "Vorspiel-Screen hat den Knopf Matches")
    if i >= 0:
        pg.buttons[i][1]()
        check(isinstance(app.shown, matchview.MatchScreen), "Knopf öffnet den MatchScreen")
        app.shown._close()
        check(isinstance(app.shown, menu.PreGameScreen), "Esc/Schließen kehrt zum Vorspiel-Screen zurück")
    # Beispieldaten für einige Spiele (alle Info-Schlüssel)
    fresh()
    keys = [k for k, _ in matches.INFO_KEYS]
    sample = {"day": 330, "bet": 20, "win": 45, "num": 17, "cr": 245, "mi": 3, "op": 5, "rw": 12,
              "x": 225, "c": 4, "ch": "fox", "vr": "x", "lv": 7, "ln": 9, "st": 3, "tm": 125,
              "g": 4, "wd": 6, "len": 5, "vs": "Lama"}
    check(set(keys) == set(sample), "Beispiel deckt alle Info-Schlüssel ab", set(keys) ^ set(sample))
    for n in range(40):
        for key in ("crossy", "blackjack", "sudoku", "wordle", "chess", "lamacrossing"):
            matches.record(key, ("daily" if n % 2 else "endless") if key in ("crossy", "wordle")
                           else "single", (n * 37) % 500 - (100 if key == "blackjack" else 0),
                           (None, 1, 0, 2)[n % 4], n * 13, n % 16, dict(sample))
    bad_txt = []
    for code in LANGS:
        sub = "" if code in ("de", "en", "fr", "es", "pt") else "lang.expansion"
        with open(os.path.join(REPO, "lang", sub, code + ".json"), encoding="utf-8") as f:
            texts = json.load(f)
        for k in keys:
            if "{v}" not in texts.get("match.info." + k, ""):
                bad_txt.append((code, k))
    check(not bad_txt, "Info-Texte enthalten {v}", bad_txt[:5])
    errors, layout = [], []
    for code in LANGS:
        i18n.set_language(code, persist=False)
        for w, h in RES:
            surf = pygame.Surface((w, h))
            for cls in ALL_GAMES:
                for tab in ("recent", "all"):
                    try:
                        scr = matchview.MatchScreen(surf, w, h, app, cls, tab=tab)
                        scr.draw()
                    except Exception as exc:     # noqa: BLE001
                        errors.append((code, w, cls.__name__, tab, repr(exc)))
                        continue
                    rects = [r for r, _, _ in scr.tab_rects] + [scr.sum_rect]
                    if scr.chart_rect is not None:
                        rects.append(scr.chart_rect)
                    rects += [r for r, _, _ in scr.filter_rects]
                    if scr.sort_rect is not None:
                        rects.append(scr.sort_rect)
                    rows = scr.row_rects[:max(1, min(len(scr._rows()), scr.rows_visible))]
                    if any(r.left < 0 or r.right > w or r.top < 0 or r.bottom > h - 30
                           for r in rects + rows) or overlap(rects + rows):
                        layout.append((code, w, cls.__name__, tab))
                    if code == "de" and w == 640:
                        # Bedienung: Filter, Sortierung, Bildlauf, Reiter
                        for kname in ("m", "o", "Down", "Next", "End", "Home", "Tab"):
                            scr.handle_event(InputEvent(InputEvent.KEYDOWN, key=kname))
                        scr.draw()
    i18n.set_language("de", persist=False)
    check(not errors, "MatchScreen zeichnet: alle Spiele x 5 Auflösungen x 14 Sprachen", errors[:3])
    check(not layout, "Reiter/Summen/Kurve/Zeilen im Bild, ohne Überlappung", layout[:5])
    scr = matchview.MatchScreen(pygame.Surface((640, 480)), 640, 480, app, cls_by_key("crossy"), tab="all")
    n_all = len(scr.items)
    scr.handle_event(InputEvent(InputEvent.KEYDOWN, key="m"))
    check(scr.filter is not None and all(e[1] == scr.filter for e in scr.items) and len(scr.items) < n_all,
          "Filter M: nur ein Modus")
    scr.handle_event(InputEvent(InputEvent.KEYDOWN, key="o"))
    vals = [e[2] for e in scr.items]
    check(scr.sort_best and vals == sorted(vals, reverse=True), "Sortierung O: beste zuerst")
    scr.handle_event(InputEvent(InputEvent.KEYDOWN, key="End"))
    check(scr.sel == len(scr.items) - 1 and scr.first + scr.rows_visible > scr.sel,
          "Ende: letzte Zeile sichtbar")
    scr.handle_event(InputEvent(InputEvent.KEYDOWN, key="Tab"))
    check(scr.tab == "recent", "Tab wechselt den Reiter")
    parts = matchview.info_parts(cls_by_key("lamacrossing"), {"lv": "hard", "x": 1525, "bet": 1500})
    check(parts[0].endswith("1.500") and "15.25" in parts[1] and i18n.t("lcr.diff.hard") in parts,
          "Info-Formatierung (Tausenderpunkt, Multiplikator, match_text)", parts)


def main_():
    t0 = time.time()
    audit_data()
    audit_main_hook()
    audit_manual_games()
    audit_screens()
    print()
    if FAILS:
        print("%d FEHLER: %s  (%.1f s)" % (len(FAILS), FAILS, time.time() - t0))
        sys.exit(1)
    print("ALLE PRÜFUNGEN BESTANDEN  (%.1f s)" % (time.time() - t0))


if __name__ == "__main__":
    main_()
