# -*- coding: utf-8 -*-
"""Headless-Audit des Tages-Archivs (dailyarchive.py + Crossy Road, Sudoku, Wordle).

Geprüft wird:

Baustein  (1) seedrand.date_of ist die Umkehrung von day_index; verfügbare Tage
              = letzte 30 + ältere gespielte, nie Zukunft; clean_rows/upsert/
              find; Datums-Texte; Picker: Pfeile/Bild/Pos1/Ende, Enter wählt,
              Esc schließt, Klick auf Tag und auf ältere Tage.
Crossy    (2) Archiv-Tag spielt den Seed DIESES Datums; Bestwert/Versuche je Tag
              in "arch", "daily" (heute) bleibt unberührt; heute schreibt beide;
              Altbestand wandert ins Archiv; Bild auf/ab begrenzt; Match-Flags.
Sudoku    (3) Archiv-Rätsel = generate_daily(datum); Lösen ändert Serie und
              Tageszähler NICHT, aber Bestzeit/Sterne je Tag; heute wie bisher
              + Archiv-Zeile; Altbestand wandert ins Archiv.
Wordle    (4) Archiv-Wort = daily_answer(datum); Fortschritt eines Archiv-Tags
              wird gespeichert und fortgesetzt; Statistik/Serie/heutiger Stand
              bleiben unberührt; heute wird ins Archiv gespiegelt; die
              gespeicherte Antwort gilt auch, wenn sie nicht (mehr) in der
              Wortliste steht.
Layout    (5) Setup mit Datums-Leiste und Archiv-Fenster: 5 Auflösungen x
              14 Sprachen - im Bild, ohne Überlappung, zeichnet fehlerfrei.
Platz     (6) ein Jahr Archiv bleibt klein (Bytes je Tag).

Aufruf aus dem Repo-Root:  python tests/audit_dailyarchive.py
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
store._DIR = tempfile.mkdtemp(prefix="pgz-audit-arch-")
store._PATH = os.path.join(store._DIR, "mem.json")
store._LEGACY_HS_PATH = os.path.join(store._DIR, "highscores.json")
import replay
replay._PATH = os.path.join(store._DIR, "replay.json")
import settings as settings_mod
settings_mod.save_settings = lambda s: None
import i18n
i18n.init()
import dailyarchive as DA
import matches
import seedrand
from game_base import InputEvent
from games import crossyroad as cr
from games import sudoku as sd
from games import sudoku_gen
from games import wordle as wd

GS = json.loads(json.dumps(settings_mod.DEFAULTS))
FAILS = []
RES = ((480, 360), (640, 480), (800, 600), (960, 720), (1280, 960))
LANGS = [e[0] if isinstance(e, (tuple, list)) else e for e in i18n.AVAILABLE]
TODAY = seedrand.day_index()


def check(ok, label, detail=""):
    print("  %-4s %s%s" % ("OK" if ok else "FAIL", label,
                           ("  -> " + str(detail)) if detail and not ok else ""))
    if not ok:
        FAILS.append(label)
    return ok


def quiet(game):
    game.play_sound = lambda name: None
    game.rumble = lambda ms=0: None
    game.events = []
    game.ach_event = lambda *a, **k: game.events.append(a[0] if a else None)
    game.report_result = lambda won: None
    return game


def key(k):
    return InputEvent(InputEvent.KEYDOWN, key=k)


def click(pos):
    return InputEvent(InputEvent.MOUSEDOWN, pos=pos, button=1)


def overlap(rects):
    return any(rects[i].colliderect(rects[j]) for i in range(len(rects))
               for j in range(i + 1, len(rects)))


# ---------------------------------------------------------------- (1) Baustein

def audit_core():
    print("\nBaustein (dailyarchive.py)")
    ok = all(seedrand.day_index(seedrand.date_of(i)) == i for i in range(-400, 3000, 7))
    check(ok and seedrand.date_of(0) == "2026-01-01" and seedrand.date_of(365) == "2027-01-01",
          "date_of = Umkehrung von day_index")
    days = DA.available([TODAY - 100, TODAY - 5, TODAY + 3, -1, True], TODAY)
    check(days[0] == TODAY - 100 and days[-1] == TODAY and len(days) == DA.WINDOW + 1
          and TODAY + 3 not in days, "verfügbar: 30 Tage + ältere gespielte, keine Zukunft", days[:3])
    rows = DA.clean_rows([[5, 1, 1], [3, 2, 2], [5, 9, 9], ["x", 1, 1], [7, 1], [-2, 1, 1],
                          [True, 1, 1]], 3)
    check(rows == [[3, 2, 2], [5, 9, 9]], "clean_rows: sortiert, letzte gewinnt, Müll raus", rows)
    DA.upsert(rows, [4, 0, 0])
    DA.upsert(rows, [5, 1, 1])
    DA.upsert(rows, [9, 3, 3])
    check([r[0] for r in rows] == [3, 4, 5, 9] and DA.find(rows, 5) == [5, 1, 1]
          and DA.find(rows, 6) is None, "upsert/find", rows)
    check(DA.short_date(seedrand.day_index("2026-11-26")) == "26.11."
          and DA.chip_date(seedrand.day_index("2025-12-31")) == "31.12.25"
          and DA.long_date(seedrand.day_index("2026-11-26")).endswith("26.11.2026"),
          "Datums-Texte")
    check(DA.day_label(TODAY) == i18n.t("arch.today", date=DA.short_date(TODAY))
          and DA.day_label(TODAY - 1) == i18n.t("arch.yesterday", date=DA.short_date(TODAY - 1)),
          "Heute/Gestern")
    check(DA.seed_hex("crossy", TODAY) == "%08X" % seedrand.daily_seed("crossy"),
          "Seed-Anzeige = daily_seed")
    # Picker
    p = DA.Picker("crossy", (200, 200, 0), TODAY, {TODAY - 60: ("solved", "x"), TODAY - 2: ("played", "")})
    p.layout(640, 480)
    p.handle_event(key("Left"))
    check(p.sel == TODAY - 1, "Picker: Links = Vortag")
    p.handle_event(key("Up"))
    check(p.sel == TODAY - 8, "Picker: Hoch = eine Woche zurück")
    p.handle_event(key("Home"))
    check(p.sel == TODAY - 60, "Picker: Pos1 = ältester Tag (gespielter Alt-Tag)")
    p.handle_event(key("End"))
    check(p.sel == TODAY, "Picker: Ende = heute")
    check(p.handle_event(key("Return")) == ("pick", TODAY), "Picker: Enter wählt")
    check(p.handle_event(key("Escape")) == ("close", None), "Picker: Esc schließt")
    cell = [rc for rc, d in p.cells if d == TODAY - 3][0]
    check(p.handle_event(click(cell.center)) == ("pick", TODAY - 3), "Picker: Klick auf einen Tag")
    chip = p.chips[0]
    check(chip[1] == TODAY - 60 and p.handle_event(click(chip[0].center)) == ("pick", TODAY - 60),
          "Picker: Klick auf älteren gespielten Tag")
    check(p.handle_event(click((2, 2))) == ("close", None), "Picker: Klick daneben schließt")
    check(all(d is None or TODAY - 29 <= d <= TODAY for _, d in p.cells)
          and sum(1 for _, d in p.cells if d is not None) == DA.WINDOW, "Kalender = genau 30 Tage")


# ---------------------------------------------------------------- (2) Crossy

def audit_crossy():
    print("\nCrossy Road (Tagesstrecke)")
    surf = pygame.Surface((640, 480))
    store.save_section("crossy", {"coins": 3, "daily": {"date": seedrand.date_of(TODAY - 1), "best": 8}})
    g = quiet(cr.CrossyRoadGame(surf, 640, 480, mode="daily", game_settings=GS))
    check(g.data["arch"] == [[TODAY - 1, 8, 1]], "Altbestand (gestern) wandert ins Archiv", g.data["arch"])
    check(g.day == TODAY and g._daily_best() == 0 and not g._is_archive(), "Start: heute, Bestwert 0")
    g._step_day(1)
    check(g.day == TODAY, "Bild ab über heute hinaus: bleibt heute")
    g.handle_event(key("Prior"))
    check(g.day == TODAY - 1 and g._is_archive() and g._daily_best() == 8, "Bild auf: gestern, Bestwert 8")
    g._new_run()
    check(g.world.seed == seedrand.daily_seed("crossy", seedrand.date_of(TODAY - 1))
          if hasattr(g.world, "seed") else True, "Archiv-Lauf nutzt den Seed des Datums")
    w2 = cr.cw.World(seedrand.daily_seed("crossy", seedrand.date_of(TODAY - 1)))
    same = all(g.world.row(r) == w2.row(r) for r in range(1, 60)) if hasattr(g.world, "row") else True
    check(same, "Strecke identisch zur Tagesstrecke dieses Datums")
    g.best = 15
    flags = g.match_flags()
    g._finish_run()
    d = store.load_section("crossy")
    check(DA.find(d["arch"], TODAY - 1) == [TODAY - 1, 15, 2] and g.record
          and d["daily"] == {"date": seedrand.date_of(TODAY - 1), "best": 8},
          "Archiv-Tag: Bestwert/Versuche im Archiv, 'daily' unberührt", (d["arch"], d["daily"]))
    check(flags == matches.F_DAILY | matches.F_ARCHIVE and g.match_info()["day"] == TODAY - 1,
          "Match: Flags Tagesmodus+Archiv, Info mit Tag")
    g = quiet(cr.CrossyRoadGame(surf, 640, 480, mode="daily", game_settings=GS))
    g._new_run()
    g.best = 21
    g._finish_run()
    d = store.load_section("crossy")
    check(d["daily"] == {"date": seedrand.today_str(), "best": 21}
          and DA.find(d["arch"], TODAY) == [TODAY, 21, 1], "heute: 'daily' + Archiv", d)
    check(g.match_flags() == matches.F_DAILY, "heute: nur Flag Tagesmodus")
    g._to_setup("play")
    g._open_picker()
    check(g.wants_escape and g.picker.info[TODAY - 1][0] == "played", "Archiv-Fenster: Esc gehört dem Spiel")
    g.handle_event(key("Escape"))
    check(g.picker is None and g.state == cr.SETUP, "Esc schließt nur das Fenster")
    g._open_picker()
    g.picker.sel = TODAY - 1
    g.handle_event(key("Return"))
    check(g.picker is None and g.day == TODAY - 1, "Enter im Fenster wählt den Tag")
    e = cr.CrossyRoadGame(surf, 640, 480, mode="endless", game_settings=GS)
    check(e.match_flags() == 0 and "day" not in e.match_info(), "Endlos: keine Tages-Flags")


# ---------------------------------------------------------------- (3) Sudoku

def audit_sudoku():
    print("\nSudoku (Tagesrätsel)")
    surf = pygame.Surface((800, 600))
    yday = seedrand.date_of(TODAY - 1)
    store.save_section("sudoku", {"daily": {"date": yday, "streak": 4, "best": 6, "time": 300,
                                            "stars": 2, "count": 9}})
    g = quiet(sd.SudokuGame(surf, 800, 600, mode="classic", game_settings=GS))
    check(g.arch == [[TODAY - 1, 300, 2]], "Altbestand wandert ins Archiv", g.arch)
    check(g.daily_streak() == 4, "Serie (gestern gelöst) läuft")
    g._step_day(-3)
    check(g.arch_day == TODAY - 3 and g.is_archive(), "Bild auf x3: Archiv-Tag")
    g._start_daily()
    g._do_generate()
    puzzle, solution, diff = sudoku_gen.generate_daily(seedrand.date_of(TODAY - 3))
    board = getattr(g, "solution", None)
    check(g.daily_date == seedrand.date_of(TODAY - 3) and g.play_diff == diff,
          "Archiv-Rätsel = Tagesrätsel dieses Datums (Stufe aus dem Wochentag)")
    if board is not None:
        flat = [v for row in board for v in row] if isinstance(board[0], list) else list(board)
        check(flat == list(solution) or board == solution, "Lösung identisch")
    g.elapsed = 412.0
    g.errors = 0
    g.hints_used = 0
    g._win()
    d = store.load_section("sudoku")
    check(d["daily"]["streak"] == 4 and d["daily"]["date"] == yday and d["daily"]["count"] == 9,
          "Archiv-Tag ändert Serie und Zähler nicht", d["daily"])
    row = DA.find(d["arch"], TODAY - 3)
    check(row is not None and row[1] == 412 and 1 <= row[2] <= 3, "Archiv-Zeile: Zeit + Sterne", row)
    check(g.match_flags() == matches.F_DAILY | matches.F_ARCHIVE and g.match_info()["tm"] == 412,
          "Match: Archiv-Flags, Zeit")
    # zweites Lösen: Bestzeit bleibt das Minimum
    g._start_daily()
    g._do_generate()
    g.elapsed = 500.0
    g._win()
    check(DA.find(store.load_section("sudoku")["arch"], TODAY - 3)[1] == 412 and not g.new_record,
          "langsamer gelöst: Bestzeit bleibt")
    # heute
    g._back_to_setup()
    g._set_arch_day(TODAY)
    g._start_daily()
    g._do_generate()
    g.elapsed = 200.0
    g._win()
    d = store.load_section("sudoku")
    check(d["daily"]["date"] == seedrand.today_str() and d["daily"]["streak"] == 5
          and DA.find(d["arch"], TODAY)[1] == 200, "heute: Serie +1 und Archiv-Zeile", d["daily"])
    g._back_to_setup()
    g._open_picker()
    check(g.picker.info[TODAY][0] == "solved" and g.picker.info[TODAY - 3][0] == "solved",
          "Archiv-Fenster zeigt gelöste Tage")


# ---------------------------------------------------------------- (4) Wordle

def audit_wordle():
    print("\nWordle (Tageswort)")
    surf = pygame.Surface((800, 600))
    store.save_section("wordle", {})
    g = quiet(wd.WordleGame(surf, 800, 600, mode="daily", game_settings=GS))
    i18n.set_language("de", persist=False)
    g._start()
    today_answer = g.boards[0].answer
    check(today_answer == wd.daily_answer(g.words, g.lang, g.length, seedrand.today_str()),
          "heute: Tageswort")
    # einen falschen Versuch heute -> gespiegelt
    wrong = next(w for w in g.words if w != today_answer)
    g.current = wrong
    g._submit()
    g._finish_reveal()
    data = wd.load_data()
    rows = wd.arch_rows(data, g._daily_slot())
    check(DA.find(rows, TODAY) == [TODAY, today_answer, wrong, 2], "heute: Stand ins Archiv gespiegelt",
          rows)
    st_before = json.dumps(data["stats"], sort_keys=True)
    daily_before = json.dumps(data["daily"], sort_keys=True)
    # Archiv-Tag
    g._to_setup()
    g.arch_day = TODAY - 5
    check(g.is_archive(), "Archiv-Tag gewählt")
    g._start()
    ans = g.boards[0].answer
    check(ans == wd.daily_answer(g.words, g.lang, g.length, seedrand.date_of(TODAY - 5)),
          "Archiv-Wort = Tageswort dieses Datums")
    other = next(w for w in g.words if w not in (ans,))
    g.current = other
    g._submit()
    g._finish_reveal()
    # Neustart -> Fortschritt da
    g2 = quiet(wd.WordleGame(surf, 800, 600, mode="daily", game_settings=GS))
    g2.arch_day = TODAY - 5
    g2._start()
    check(g2.guesses == [other], "Archiv-Fortschritt wird fortgesetzt", g2.guesses)
    g2.current = ans
    g2._submit()
    g2._finish_reveal()
    data = wd.load_data()
    check(json.dumps(data["stats"], sort_keys=True) == st_before, "Archiv-Sieg ändert die Statistik nicht")
    check(json.dumps(data["daily"], sort_keys=True) == daily_before, "heutiger Stand bleibt unberührt")
    row = DA.find(wd.arch_rows(data, g2._daily_slot()), TODAY - 5)
    check(row == [TODAY - 5, ans, other + ans, 1], "Archiv-Zeile: Antwort, Versuche, gelöst", row)
    check("wordle_daily7" not in g2.events, "kein Serien-Erfolg für Archiv-Tage")
    check(g2.match_flags() == matches.F_DAILY | matches.F_ARCHIVE and g2.match_info()["g"] == 2
          and g2.match_score() > 0, "Match: Archiv-Flags, 2 Versuche, Punkte")
    # gespeicherte Antwort ausserhalb der Wortliste
    data["arch"][g2._daily_slot()] = [[TODAY - 7, "ZZZZZ", "", 2]]
    wd.save_data(data)
    g3 = quiet(wd.WordleGame(surf, 800, 600, mode="daily", game_settings=GS))
    g3.arch_day = TODAY - 7
    g3._start()
    g3.current = "ZZZZZ"
    g3._submit()
    check(g3.reveal is not None, "gespeicherte Antwort gilt, auch wenn sie nicht in der Liste steht")
    # Picker-Info
    g3._to_setup()
    g3.arch_day = TODAY
    g3._open_picker()
    check(g3.picker.info.get(TODAY, ("",))[0] in ("started", "played", "solved"), "Archiv-Fenster: heute")


# ---------------------------------------------------------------- (5) Layout

def audit_layout():
    print("\nLayout: Datums-Leiste + Archiv-Fenster (5 Auflösungen x 14 Sprachen)")
    errors, bad = [], []
    for code in LANGS:
        i18n.set_language(code, persist=False)
        for w, h in RES:
            surf = pygame.Surface((w, h))
            try:
                c = quiet(cr.CrossyRoadGame(surf, w, h, mode="daily", game_settings=GS))
                c.day = TODAY - 2
                c.draw()
                rr = list(c.day_rects)
                if any(r.left < 0 or r.right > w or r.bottom > h - 34 for r in rr) or overlap(
                        rr + c.night_rects + c.shadow_rects + [c.start_rect]):
                    bad.append(("crossy", code, w))
                c._open_picker()
                c.draw()
                s = quiet(sd.SudokuGame(surf, w, h, mode="classic", game_settings=GS))
                s._set_arch_day(TODAY - 2)
                s.draw()
                rr = list(s.dayrow_rects)
                if any(not s.panel_rect.contains(r) for r in rr) or overlap(
                        rr + [s.daily_rect, s.limit_rect]):
                    bad.append(("sudoku", code, w))
                s._open_picker()
                s.draw()
                q = quiet(wd.WordleGame(surf, w, h, mode="daily", game_settings=GS))
                q.arch_day = TODAY - 2
                q.draw()
                rr = list(q.su_day)
                if any(r.left < 0 or r.bottom > h - 20 for r in rr) or overlap(
                        rr + [q.su_cb, q.su_start]):
                    bad.append(("wordle", code, w))
                q._open_picker()
                q.draw()
                p = q.picker
                if p.rect.left < 0 or p.rect.right > w or p.rect.top < 0 or p.rect.bottom > h:
                    bad.append(("picker", code, w))
                cells = [rc for rc, d in p.cells if d is not None]
                if any(not p.rect.contains(rc) for rc in cells + [c for c, _ in p.chips]):
                    bad.append(("picker-zellen", code, w))
            except Exception as exc:          # noqa: BLE001
                errors.append((code, w, repr(exc)))
    i18n.set_language("de", persist=False)
    check(not errors, "zeichnet fehlerfrei", errors[:3])
    check(not bad, "im Bild, ohne Überlappung", bad[:8])


# ---------------------------------------------------------------- (6) Platz

def audit_space():
    print("\nPlatz")
    rows = [[TODAY - i, 100 + i, 3] for i in range(365)]
    size = len(json.dumps(rows, separators=(",", ":")))
    check(size / 365 < 20, "Crossy: ein Jahr Archiv = %d Byte (%.1f je Tag)" % (size, size / 365.0))
    rows = [[TODAY - i, "LLAMA", "CRANESLATELLAMA", 1] for i in range(365)]
    size = len(json.dumps(rows, separators=(",", ":")))
    check(size / 365 < 45, "Wordle: ein Jahr Archiv = %d Byte (%.1f je Tag)" % (size, size / 365.0))


def main_():
    t0 = time.time()
    audit_core()
    audit_crossy()
    audit_sudoku()
    audit_wordle()
    audit_layout()
    audit_space()
    print()
    if FAILS:
        print("%d FEHLER: %s  (%.1f s)" % (len(FAILS), FAILS, time.time() - t0))
        sys.exit(1)
    print("ALLE PRÜFUNGEN BESTANDEN  (%.1f s)" % (time.time() - t0))


if __name__ == "__main__":
    main_()
