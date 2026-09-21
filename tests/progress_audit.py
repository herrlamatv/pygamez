# -*- coding: utf-8 -*-
"""
progress_audit.py
=================
Prüfstand für das Erfolgs-, Statistik- und Profil-Update.

Geprüft werden:

1. **Profile** (store.py/profiles.py): Migration einer alten mem.json in das
   erste Profil, Anlegen/Wechseln/Löschen, saubere Trennung von Highscores,
   Erfolgen, Chips und Spielständen, Namensprüfung mit Wortfilter.
2. **Statistik** (stats.py): die neue Datei ``stats.pgzlamastats`` - Anlegen,
   Migration aus mem.json, Tage/Stunden/Serien/Verlauf, Duell-Bilanzen,
   Kappungsgrenzen, Beschädigung, Export/Import.
3. **Erfolge & Level** (achievements.py/levels.py): Seltenheitsstufen, Punkte,
   Level-Kurve, stiller Abgleich, Toasts (genau einer je Aufstieg).
4. **Oberfläche**: alle fünf Reiter des Fortschritt-Screens, der Profil-Screen,
   die Spieler-Auswahl und die Startlevel-Karte zeichnen fehlerfrei - in allen
   14 Sprachen und mehreren Auflösungen.

Aufruf (headless)::

    .venv/Scripts/python.exe tests/progress_audit.py

Exit-Code 1, wenn eine Prüfung fehlschlägt.
"""

import json
import os
import shutil
import sys
import tempfile
import time

os.environ.setdefault("SDL_VIDEODRIVER", "dummy")
ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, ROOT)

import pygame  # noqa: E402

pygame.init()
pygame.display.set_mode((16, 16))

TMP = tempfile.mkdtemp(prefix="pgz-progress-")
MEM = os.path.join(TMP, "mem.json")

import store  # noqa: E402

store._PATH = MEM
store._DIR = TMP
store._LEGACY_HS_PATH = os.path.join(TMP, "highscores.json")

import i18n  # noqa: E402
import ui  # noqa: E402

i18n.init()
ui.set_theme(ui.DEFAULT_THEME)

import achievements  # noqa: E402
import levels  # noqa: E402
import profiles  # noqa: E402
import stats  # noqa: E402

FAILED = []
N = [0]


def check(ok, label, detail=""):
    N[0] += 1
    if ok:
        print("  OK   %s" % label)
    else:
        FAILED.append(label)
        print("  FAIL %s%s" % (label, "  -> %s" % detail if detail else ""))


def fresh(content=None):
    """Setzt Speicher und alle Zwischenspeicher zurück."""
    for name in ("mem.json", "mem.json.bak", "mem.json.tmp",
                 stats.FILENAME, stats.FILENAME + ".bak"):
        p = os.path.join(TMP, name)
        if os.path.exists(p):
            os.remove(p)
    if content is not None:
        with open(MEM, "w", encoding="utf-8") as f:
            json.dump(content, f)
    stats._doc = None
    stats._dirty = False
    stats.reset_cache()
    achievements.reset_cache()
    achievements._defs = None


# ===================================================== 1. Profile & Migration
def audit_profiles():
    print("\n[Profile]")
    old = {
        "mem": {"lang": "de"},
        "highscores": {"snake": 640, "tetris": 12000},
        "achievements": {"first_game": "2026-01-02 10:00"},
        "stats": {"snake": {"plays": 30, "time": 1800.0, "wins": 2,
                            "losses": 1, "records": 3,
                            "last": "2026-01-02 10:00"}},
        "casino": {"chips": 2500, "migrated": True},
        "tetris": {"best": {"marathon": 12000}},
    }
    fresh(old)

    check(store.active_id() == "p1", "Migration legt Profil 1 an", store.active_id())
    raw = json.load(open(MEM, encoding="utf-8"))
    check(set(raw) == {"_generated", "mem", "profiles", "players"},
          "mem.json hat nur noch Verwaltung + players", sorted(raw))
    check(raw["players"]["p1"]["highscores"] == {"snake": 640, "tetris": 12000},
          "Highscores ins Profil verschoben")
    check(store.load_section("mem") == {"lang": "de"},
          "Sprache bleibt gerätweit")
    check(store.load_section("tetris") == {"best": {"marathon": 12000}},
          "Spielstände wandern mit")

    # Zweimal laden darf nichts mehr ändern.
    before = open(MEM, encoding="utf-8").read()
    store.load()
    check(before == open(MEM, encoding="utf-8").read(), "Migration ist idempotent")

    # Statistik zieht aus mem.json in die eigene Datei um.
    check(stats.get("snake")["plays"] == 30, "Statistik übernommen",
          stats.get("snake"))
    check(os.path.exists(os.path.join(TMP, stats.FILENAME)),
          "stats.pgzlamastats angelegt")
    check("stats" not in json.load(open(MEM, encoding="utf-8"))["players"]["p1"],
          "alter stats-Abschnitt ist aus mem.json verschwunden")

    # --- zweites Profil ---------------------------------------------------
    pid, err = profiles.create("Alpaka", icon="star", color=2)
    check(pid == "p2" and not err, "Zweites Profil angelegt", err)
    profiles.switch(pid)
    import highscore
    check(profiles.active_id() == "p2" and highscore.load_highscores() == {},
          "Neues Profil startet ohne Highscores")
    check(achievements.counts()[0] == 0, "Neues Profil startet ohne Erfolge")
    import lamabank
    check(lamabank.load()["chips"] == lamabank.START_CHIPS,
          "Neues Profil startet mit frischem Chip-Konto", lamabank.balance())
    highscore.save_highscores({"snake": 99})
    stats.game_started("snake")

    profiles.switch("p1")
    check(highscore.load_highscores()["snake"] == 640,
          "Zurück im ersten Profil: Highscore wieder da")
    check(achievements.counts()[0] >= 1, "Zurück im ersten Profil: Erfolge wieder da")
    check(lamabank.load()["chips"] == 2500, "Chips gehören zum Profil",
          lamabank.balance())
    check(stats.get("snake")["plays"] == 30, "Statistik gehört zum Profil")

    # --- Namensprüfung ----------------------------------------------------
    check(profiles.check_name("") == profiles.ERR_EMPTY, "Leerer Name abgelehnt")
    check(profiles.check_name("x" * 40) == profiles.ERR_LONG, "Zu langer Name abgelehnt")
    check(profiles.check_name("Alpaka") == profiles.ERR_DUP, "Doppelter Name abgelehnt")
    check(profiles.check_name("Lamafreund") == "", "Normaler Name geht durch")
    check(profiles.name_of(profiles.GUEST) == i18n.t("profile.guest"),
          "Gast hat einen Namen")

    # --- Löschen ----------------------------------------------------------
    check(profiles.delete("p2"), "Profil gelöscht")
    check(profiles.count() == 1 and "p2" not in store.profile_ids(),
          "Profil ist aus der Liste verschwunden")
    check(not profiles.delete("p1"), "Das letzte Profil bleibt bestehen")
    check("p2" not in stats.profile_ids(), "Statistik des Profils ist weg")


# ===================================================== 2. Statistik-Datei
def audit_stats():
    print("\n[Statistik-Datei]")
    fresh({"mem": {"lang": "de"}})

    stats.game_started("snake")
    stats.add_playtime("snake", 45.0)
    stats.record_result("snake", True)
    stats.score_result("snake", 500, record=True)
    stats.record_broken("snake")
    stats.flush()

    path = os.path.join(TMP, stats.FILENAME)
    doc = json.load(open(path, encoding="utf-8"))
    check(doc["format"] == stats.FORMAT and doc["v"] == stats.VERSION,
          "Dateikopf stimmt", doc.get("format"))
    prof = doc["profiles"]["p1"]
    check(prof["games"]["snake"]["plays"] == 1 and prof["games"]["snake"]["wins"] == 1,
          "Partie und Sieg gezählt", prof["games"]["snake"])
    today = time.strftime("%Y-%m-%d")
    check(prof["days"][today]["p"] == 1 and prof["days"][today]["t"] > 0,
          "Spieltag mit Zeit eingetragen", prof["days"].get(today))
    check(any(h > 0 for h in prof["hours"]), "Tageszeit-Profil gefüllt")
    check(prof["series"]["snake"][0][1] == 500 and prof["series"]["snake"][0][2] == 1,
          "Punkteverlauf mit Rekord-Marke", prof["series"]["snake"])
    check(any(e["k"] == "rec" for e in prof["events"]), "Rekord steht im Verlauf")
    check(stats.totals()["days"] == 1 and stats.streak()["cur"] == 1,
          "Tagesserie gezählt", stats.streak())

    # Sitzungslänge
    stats.add_playtime("snake", 30.0)
    length = stats.end_session()
    check(length >= 30 and stats.totals()["longest"] >= 30,
          "Längste Sitzung gemerkt", length)

    # Duell-Bilanz (beide Seiten)
    pid, _err = profiles.create("Duellpartner")
    stats.record_duel("pong", pid, True)
    stats.record_duel("pong", "p1", False, pid=pid)
    check(stats.duel_table()["%s" % pid]["pong"] == {"w": 1, "l": 0},
          "Duell-Sieg gebucht", stats.duel_table())
    check(stats.duel_table(pid)["p1"]["pong"] == {"w": 0, "l": 1},
          "Gegenseite gebucht", stats.duel_table(pid))
    check(stats.duel_total() == 1, "Duelle gezählt")
    profiles.delete(pid)

    # Kappungsgrenzen
    prof = stats._profile()
    for i in range(stats.MAX_EVENTS + 50):
        stats._add_event(prof, {"t": "2026-01-01 00:00", "k": "ach", "id": "x%d" % i, "p": 0})
    check(len(prof["events"]) == stats.MAX_EVENTS, "Verlauf wird gekappt",
          len(prof["events"]))
    for i in range(stats.MAX_SERIES + 40):
        stats.score_result("tetris", i + 1, record=(i % 100 == 0))
    series = stats.series_of("tetris")
    check(len(series) <= stats.MAX_SERIES, "Punkteverlauf wird gekappt", len(series))
    check(sum(1 for p in series if p[2]) >= 3, "Rekorde bleiben beim Kappen erhalten",
          sum(1 for p in series if p[2]))

    # Export / Import
    stats.flush()
    out = os.path.join(TMP, "export" + stats.EXT)
    check(stats.export_to(out) and os.path.exists(out), "Export geschrieben")
    plays = stats.get("snake")["plays"]
    ok, reason = stats.import_from(out, target="p1", merge=True)
    check(ok and stats.get("snake")["plays"] == plays * 2,
          "Import addiert beim Zusammenführen", "%s/%s" % (reason, stats.get("snake")["plays"]))
    bad = os.path.join(TMP, "kaputt" + stats.EXT)
    with open(bad, "w", encoding="utf-8") as f:
        f.write("{kein json")
    check(stats.import_from(bad) == (False, "io"), "Kaputte Datei wird abgewiesen")
    with open(bad, "w", encoding="utf-8") as f:
        json.dump({"format": "fremd"}, f)
    check(stats.import_from(bad) == (False, "format"), "Fremdes Format wird abgewiesen")

    # Beschädigte Statistik-Datei -> Sicherung/leerer Stand, kein Absturz
    stats.flush()
    with open(os.path.join(TMP, stats.FILENAME), "w", encoding="utf-8") as f:
        f.write("{kaputt")
    stats._doc = None
    stats.reset_cache()
    try:
        totals = stats.totals()
        check(isinstance(totals, dict), "Beschädigte Datei stürzt nicht ab")
    except Exception as exc:                       # pragma: no cover
        check(False, "Beschädigte Datei stürzt nicht ab", exc)


# ===================================================== 3. Erfolge & Level
def audit_achievements():
    print("\n[Erfolge & Level]")
    fresh({"mem": {"lang": "de"}})

    defs = achievements.all_defs()
    ids = [d["id"] for d in defs]
    check(len(ids) == len(set(ids)), "Erfolgs-ids eindeutig")
    check(len(defs) > 200, "Erfolgs-Umfang", len(defs))
    bad = [d["id"] for d in defs if d["rarity"] not in achievements.RARITY]
    check(not bad, "jede Definition hat eine gültige Seltenheitsstufe", bad)
    cats = {d["cat"] for d in defs}
    check(cats == {"general", "milestone", "master", "regular", "special"},
          "alle fünf Kategorien vorhanden", sorted(cats))
    pts = [achievements.RARITY[r]["points"] for r in achievements.RARITY_ORDER]
    check(pts == sorted(pts) and len(set(pts)) == len(pts),
          "Punktwerte steigen je Stufe", pts)

    # Level-Kurve
    check(levels.level_for(0) == 1 and levels.level_for(20) == 2
          and levels.level_for(levels.total_for(10)) == 10,
          "Level-Kurve trifft ihre Marken",
          [levels.level_for(0), levels.level_for(20), levels.level_for(levels.total_for(10))])
    check(all(levels.level_for(p) <= levels.level_for(p + 7) for p in range(0, 3000, 37)),
          "Level steigt monoton")
    check(levels.rank_index(1) == 0 and levels.rank_index(99) == len(levels.RANKS) - 1,
          "Ränge decken die Kurve ab")

    # Freischalten: Punkte, Verlauf, Toast
    achievements._toasts.clear()
    achievements.event("night_owl")
    d = achievements.by_id("night_owl")
    check(d["id"] in achievements.unlocked(), "Ereignis-Erfolg freigeschaltet")
    check(stats.points_of() == achievements.points_of(d),
          "Punkte gutgeschrieben", stats.points_of())
    check(len(achievements._toasts) >= 1, "Toast eingereiht")
    achievements.event("night_owl")
    check(len(achievements._toasts) == 1, "Zweiter Versuch schaltet nichts erneut frei")

    # Level-Aufstieg feuert genau einen Level-Toast. Der Punktestand wird aus
    # den freigeschalteten Erfolgen gebildet - für den Test tun wir kurz so,
    # als brächte der nächste Erfolg das Konto über die Level-5-Marke.
    achievements._toasts.clear()
    real_points = achievements.total_points
    achievements.total_points = lambda: levels.total_for(5)
    try:
        achievements.event("early_bird")
    finally:
        achievements.total_points = real_points
    level_toasts = [t for t in achievements._toasts if t.get("kind") == "level"]
    check(len(level_toasts) == 1, "Level-Aufstieg zeigt genau einen Toast",
          len(level_toasts))
    check(any(e["k"] == "lvl" for e in stats.events()), "Aufstieg steht im Verlauf")

    # Stiller Abgleich rechnet Bestandsdaten an
    fresh({"mem": {"lang": "de"},
           "highscores": {"snake": 900, "tetris": 40000},
           "stats": {"snake": {"plays": 60, "time": 4000.0, "wins": 5,
                               "losses": 2, "records": 4, "last": "2026-01-02 10:00"}}})
    new, level, points = achievements.backfill_silent()
    unlocked = achievements.unlocked()
    check(new > 0 and points > 0, "Bestandsdaten ergeben Punkte",
          "neu=%d punkte=%d" % (new, points))
    check("score_snake" in unlocked, "Punkte-Meilenstein aus Highscore")
    check("master_tetris" in unlocked, "Meister-Stufe aus Highscore")
    check("regular_snake" in unlocked, "Stammgast-Erfolg aus Partien")
    check(not achievements._toasts, "stiller Abgleich zeigt keine Toasts")
    check(points == achievements.total_points() == stats.points_of(),
          "Punktestand = Summe der Erfolge")
    check(level == levels.level_for(points), "Level passt zum Punktestand")
    check(len(achievements.rarest_unlocked(3)) <= 3, "Seltenste Erfolge abrufbar")


# ===================================================== 4. Oberfläche
class _FakeApp:
    """Minimale App-Hülle für die Screens (wie in den anderen Audits)."""

    def __init__(self, surface):
        import settings as settings_mod
        from games import ALL_GAMES
        self.settings = settings_mod.load_settings()
        self._game_classes = ALL_GAMES
        self.canvas = surface
        self.game_w, self.game_h = surface.get_width(), surface.get_height()
        self.ui = ui

    def spiel_starten(self, cls):
        pass

    def show_screen(self, screen, exit_old=True):
        pass

    def back_to_menu(self):
        pass

    def refresh_profile(self):
        pass

    def launch_game(self, cls, mode, p2=None):
        pass


def audit_screens():
    print("\n[Oberfläche]")
    fresh({"mem": {"lang": "de"},
           "highscores": {"snake": 900, "minigolf": 5200},
           "stats": {"snake": {"plays": 12, "time": 900.0, "wins": 3, "losses": 1,
                               "records": 2, "last": "2026-01-02 10:00"}}})
    achievements.backfill_silent()
    stats.game_started("snake")
    stats.add_playtime("snake", 30.0)
    stats.score_result("snake", 900, record=True)
    stats.record_duel("pong", "p2", True)
    stats.flush()

    from game_base import InputEvent
    from profileview import LevelIntroScreen, PlayerPickScreen, ProfileScreen
    from progress import ProgressScreen

    langs = [code for code, _name in i18n.PRIMARY + i18n.EXTRA]
    sizes = [(800, 600), (640, 480), (1280, 720)]
    errors = []
    for code in langs:
        i18n.set_language(code, persist=False)
        for w, h in sizes:
            surf = pygame.Surface((w, h))
            app = _FakeApp(surf)
            try:
                screen = ProgressScreen(surf, w, h, app, on_close=lambda: None)
                for tab in ProgressScreen.TABS:
                    screen.set_tab(tab)
                    screen.handle_event(InputEvent(InputEvent.MOUSEMOVE,
                                                   pos=(w // 2, h // 2)))
                    screen.draw()
                    screen.scroll = 10 ** 6
                    screen._clamp_scroll()
                    screen.draw()
                for cls in (ProfileScreen, LevelIntroScreen):
                    other = cls(surf, w, h, app, on_close=lambda: None)
                    other.draw()
                pick = PlayerPickScreen(surf, w, h, app, on_pick=lambda pid: None,
                                        on_cancel=lambda: None,
                                        game_cls=app._game_classes[0])
                pick.draw()
            except Exception as exc:               # pragma: no cover
                errors.append("%s %dx%d: %r" % (code, w, h, exc))
    i18n.set_language("de", persist=False)
    check(not errors, "alle Screens zeichnen in %d Sprachen x %d Auflösungen"
          % (len(langs), len(sizes)), errors[:3])

    # Editor + Eingaben im Profil-Screen
    surf = pygame.Surface((800, 600))
    app = _FakeApp(surf)
    scr = ProfileScreen(surf, 800, 600, app, on_close=lambda: None)
    scr._new_profile()
    scr.editor["field"].set_text("Testlama")
    scr._save_editor()
    check(scr.editor is None and profiles.count() == 2, "Profil über den Editor angelegt")
    scr._edit_profile("p1")
    scr.editor["field"].set_text("Umbenannt")
    scr._save_editor()
    check(profiles.name_of("p1") == "Umbenannt", "Profil umbenannt",
          profiles.name_of("p1"))
    scr._delete_profile(profiles.all_profiles()[1][0])
    check(profiles.count() == 1, "Profil im Screen gelöscht")

    # Toasts zeichnen (Erfolg + Level) - darf nie stolpern
    achievements._toasts.clear()
    achievements.event("wiki_reader")
    try:
        for _ in range(300):
            achievements.draw_toasts(surf, 800, 600, 0.016, app.settings)
        check(True, "Erfolgs-Toast gezeichnet")
    except Exception as exc:                       # pragma: no cover
        check(False, "Erfolgs-Toast gezeichnet", exc)

    # Diagramme mit Randfällen
    import charts
    try:
        rect = pygame.Rect(10, 10, 200, 80)
        charts.bars(surf, rect, [])
        charts.bars(surf, rect, [5], labels=["A"], fmt=str)
        charts.line(surf, rect, [3], fmt=str)
        charts.calendar_heat(surf, pygame.Rect(10, 10, 400, 120), {})
        charts.donut(surf, (100, 100), 40, [(0, ui.ACCENT)])
        charts.clock_ring(surf, (100, 100), 40, [0] * 24)
        charts.tooltip(surf, (700, 500), ["Test", "Zeile"])
        check(True, "Diagramme halten Randfälle aus")
    except Exception as exc:                       # pragma: no cover
        check(False, "Diagramme halten Randfälle aus", exc)


# ===================================================== 5. Web-Gleichstand
def audit_web_parity():
    print("\n[Web]")
    path = os.path.join(ROOT, "web", "js", "core", "achdefs.js")
    if not os.path.exists(path):
        check(False, "web/js/core/achdefs.js vorhanden")
        return
    text = open(path, encoding="utf-8").read()
    body = text[text.index("PG.achDefs = ") + len("PG.achDefs = "):text.rindex("};") + 1]
    data = json.loads(body)
    check(data["general"] == [list(e) for e in achievements.GENERAL],
          "Web kennt dieselben Allgemein-Erfolge")
    check(data["specials"] == [list(e) for e in achievements.SPECIALS],
          "Web kennt dieselben besonderen Momente")
    check(data["milestones"] == dict(achievements.MILESTONES)
          and data["masters"] == dict(achievements.MASTERS),
          "Web kennt dieselben Punkt-Stufen")
    check(data["ranks"] == [[lvl, key, col] for lvl, key, col in levels.RANKS],
          "Web kennt dieselben Ränge")


def main():
    print("PyGameZ - Erfolge, Statistik & Profile")
    try:
        audit_profiles()
        audit_stats()
        audit_achievements()
        audit_screens()
        audit_web_parity()
    finally:
        try:
            shutil.rmtree(TMP, ignore_errors=True)
        except OSError:
            pass
    print("")
    if FAILED:
        print("FEHLGESCHLAGEN: %d von %d" % (len(FAILED), N[0]))
        for label in FAILED:
            print("  - %s" % label)
        return 1
    print("ALLE %d PRÜFUNGEN BESTANDEN" % N[0])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
