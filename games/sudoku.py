# -*- coding: utf-8 -*-
"""
sudoku.py
=========
Sudoku in vier Varianten (je 4 Stufen x 100 Level), Tages-Sudoku und vier
Spielmodi. Gezeichnet wird in sudoku_draw.py, die Rätsel kommen aus
sudoku_gen.py.

Varianten (Setup-Screen)
------------------------
- Klassisch : 9x9 - die 400 bekannten Level (alter Generator, unverändert,
              damit abgehakte Level ihre Rätsel behalten).
- X-Sudoku  : 9x9, zusätzlich enthalten beide Diagonalen jede Ziffer einmal.
- Killer    : 9x9 mit Käfigen - die Ziffern eines Käfigs ergeben die kleine
              Summe in seiner Ecke und wiederholen sich nicht (vorab erzeugt,
              games/levels/sudoku-killer.json).
- Mini 6x6  : 2x3-Blöcke, Ziffern 1-6 - für zwischendurch.
- Tages-Sudoku: ein klassisches Rätsel pro Tag (überall dasselbe, seedrand),
              Stufe je Wochentag, eigene Serie.

Spielstand (mem.json, Abschnitt "sudoku")
-----------------------------------------
  solved : {"0": [1, 5, ...], ...}   gelöste klassische Level (wie bisher)
  best   : {variante: {stufe: {level: [sterne, sekunden]}}}
  saves  : {"x:1:12": {...}, "daily:2026-09-16": {...}}  angefangene Rätsel
  last   : {variante: {stufe: level}} Levelcursor der neuen Varianten
  daily  : {date, streak, best, time, stars, count}   Tages-Sudoku
Angefangene Rätsel werden beim Verlassen, beim Zurück zur Levelwahl und
gedrosselt während des Spiels gesichert und beim nächsten Start fortgesetzt.

Sterne je Level: 1 = gelöst, 2 = fehlerfrei und ohne Tipps, 3 = zusätzlich
unter der Zielzeit der Stufe (TARGET_TIME).

Spielmodi (Vorspiel-Screen, aufsteigende Hilfe-Stufen)
------------------------------------------------------
- classic  (x2,0 Punkte): keine Hilfen - pur wie auf Papier.
- notes    (x1,5 Punkte): + Bleistift-Notizen und Kandidaten-Automatik.
- comfort  (x1,0 Punkte): + falsche Ziffern sofort rot, Konflikt- und
             Gleiche-Ziffer-Hervorhebung, korrekte Eingaben rasten ein,
             Killer: falsche Käfigsummen werden markiert.
- assist   (x0,7 Punkte): + Tipp-Funktion (H, max. 3, kostet Punkte).
Wird ein angefangenes Rätsel in einem anderen Modus fortgesetzt, gilt der
kleinste der bisher benutzten Multiplikatoren.

Fehler-Regel (in ALLEN Modi gleich)
-----------------------------------
Jede Eingabe wird sofort gegen die eindeutige Lösung geprüft; eine falsche
Ziffer zählt als Fehler (Rückgängig macht ihn nicht ungeschehen). Mit
3-Fehler-Limit ist beim dritten Fehler die Partie verloren.

Punkte: (Basis je Variante/Stufe - Zeit - Fehler - Tipps) x Multiplikator.

Steuerung: Pfeile/WASD = Zelle wählen, 1-9 = Ziffer (bei "Ziffer zuerst":
Ziffer wählen, dann Zellen anklicken bzw. Leertaste), 0/Backspace/Entf =
radieren (auch Rechtsklick), N = Notizen, C = Kandidaten eintragen,
U/Z = rückgängig, Y = wiederholen, M = Farbmarker, H = Tipp, R = neu,
Q = Levelwahl. Die festen Buchstaben greifen nur, wenn sie keiner Aktion
in den Optionen zugeordnet sind.
"""

import random
import time

import pygame

import seedrand
import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent
from i18n import t

from . import sudoku_gen
from .sudoku_draw import SudokuDraw

VARIANTS = sudoku_gen.VARIANTS

# (i18n-Suffix, Basispunkte klassisch) je Schwierigkeitsgrad
DIFFICULTIES = [("easy", 1000), ("normal", 2000), ("hard", 3500), ("expert", 5000)]

# Basispunkte je Variante und Stufe (Klassisch = DIFFICULTIES wie bisher).
BASE_POINTS = {
    "classic": (1000, 2000, 3500, 5000),
    "x": (1200, 2400, 4000, 5600),
    "killer": (1500, 3000, 5000, 7000),
    "mini": (500, 900, 1400, 2000),
}

# Zielzeit in Sekunden für den dritten Stern.
TARGET_TIME = {
    "classic": (300, 540, 840, 1200),
    "x": (360, 600, 900, 1320),
    "killer": (600, 960, 1440, 1920),
    "mini": (90, 150, 240, 360),
}

MODE_MULT = {"classic": 2.0, "notes": 1.5, "comfort": 1.0, "assist": 0.7}

MAX_HINTS = 3
HINT_COST = 200      # Punktabzug je Tipp
ERR_COST = 150       # Punktabzug je Fehler
TIME_COST = 2        # Punktabzug je Sekunde
FAIL_LIMIT = 3

MARK_COUNT = 6       # Farbmarker (0 = keiner)
MAX_SAVES = 40       # so viele angefangene Rätsel werden höchstens behalten
SAVE_EVERY = 4.0     # Sekunden: gedrosseltes Speichern während des Spiels
MAX_UNDO = 500

SETUP, GENERATING, PLAY = "setup", "generating", "play"

_MOVE = {"Up": (-1, 0), "w": (-1, 0), "W": (-1, 0),
         "Down": (1, 0), "s": (1, 0), "S": (1, 0),
         "Left": (0, -1), "a": (0, -1), "A": (0, -1),
         "Right": (0, 1), "d": (0, 1), "D": (0, 1)}


def fmt_time(sec):
    sec = max(0, int(sec))
    if sec >= 3600:
        return "%d:%02d:%02d" % (sec // 3600, sec // 60 % 60, sec % 60)
    return "%02d:%02d" % (sec // 60, sec % 60)


def stars_for(variant, diff, secs, errors, hints):
    """Sterne für ein gelöstes Rätsel (1-3)."""
    clean = errors == 0 and hints == 0
    fast = secs <= TARGET_TIME[variant][max(0, min(3, diff))]
    return 1 + int(clean) + int(clean and fast)


class SudokuGame(SudokuDraw, Game):
    name = "Sudoku"
    highscore_key = "sudoku"
    supports_multiplayer = False
    wants_right_click = True     # Rechtsklick = radieren

    # Vorspiel-Screen zeigt diese Modi statt Einzel-/Mehrspieler.
    MODES = [("classic", "sud.mode.classic"), ("notes", "sud.mode.notes"),
             ("comfort", "sud.mode.comfort"), ("assist", "sud.mode.assist")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        self.won = False

        sud = self.settings.get("sudoku", {}) if isinstance(self.settings, dict) else {}
        try:
            self.diff = max(0, min(3, int(sud.get("difficulty", 0))))
        except (TypeError, ValueError):
            self.diff = 0
        self.fail_limit = bool(sud.get("fail_limit", True))
        ll = sud.get("last_level", {})
        self._last_level = dict(ll) if isinstance(ll, dict) else {}
        self.variant = sud.get("variant") if sud.get("variant") in VARIANTS else "classic"
        self.input_mode = "digit" if sud.get("input") == "digit" else "cell"
        self.colors_on = sud.get("colors") is True

        # Modus-Fähigkeiten aus dem gewählten mode ableiten.
        self.can_notes = self.mode in ("notes", "comfort", "assist")
        self.can_check = self.mode in ("comfort", "assist")   # rot + Konflikte
        self.can_hint = self.mode == "assist"

        self.daily = False
        self.daily_date = seedrand.today_str()
        self.play_variant = self.variant
        self.play_diff = self.diff
        self.level = 1
        self.lay = sudoku_gen.layout_for("classic")
        self._fresh = False
        self.hover = None
        self.msg = None
        self.msg_t = 0.0
        self.msg_col = None
        self._gen_drawn = False

        self._build_fonts()
        self._reset_caches()
        self._load_progress()
        self.cursor = self._get_last(self.variant, self.diff)
        self._build_setup_layout()
        self.state = SETUP

    def _build_fonts(self):
        """Schriftgrößen aus der aktuellen Auflösung ableiten (Theme-Fonts)."""
        h = self.height
        self._small = ui.font(max(14, h // 30))
        self._tiny = ui.font(max(11, h // 36))
        self._huge = ui.font(max(26, h // 11), bold=True)
        self._title = ui.font(max(24, h // 13), bold=True)
        # Uhr mit fester Zeichenbreite, damit sie beim Ticken nicht "zittert".
        self._clock_font = ui.font(max(18, h // 22), mono=True)

    def on_surface_changed(self):
        """Layout nach einer Auflösungsänderung neu berechnen (Options-Screen)."""
        self._build_fonts()
        self._reset_caches()
        self._build_setup_layout()
        if self.state == PLAY:
            self._build_play_layout()

    def on_exit(self):
        """Spiel wird verlassen: angefangenes Rätsel sichern."""
        if self.state == PLAY and not self.game_over:
            self._write_save()

    # ===================================================== Persistenz
    def _load_progress(self):
        """Liest Fortschritt, Bestzeiten, Spielstände und Tagesserie."""
        data = store.load_section("sudoku")
        known = ("solved", "best", "saves", "last", "daily")
        # Unbekannte Schlüssel (z.B. aus neueren Versionen) bleiben erhalten.
        self._prog_extra = {k: v for k, v in data.items() if k not in known}

        solved = data.get("solved") if isinstance(data.get("solved"), dict) else {}
        self.solved = {}
        for k in ("0", "1", "2", "3"):
            lst = solved.get(k, [])
            if isinstance(lst, list):
                self.solved[k] = sorted({int(v) for v in lst
                                         if isinstance(v, int)
                                         and not isinstance(v, bool)
                                         and 1 <= v <= sudoku_gen.LEVELS})
            else:
                self.solved[k] = []

        raw_best = data.get("best") if isinstance(data.get("best"), dict) else {}
        self.best = {}
        for v in VARIANTS:
            vb = raw_best.get(v) if isinstance(raw_best.get(v), dict) else {}
            out_v = {}
            for d in ("0", "1", "2", "3"):
                db = vb.get(d) if isinstance(vb.get(d), dict) else {}
                out = {}
                for key, rec in db.items():
                    try:
                        lvl = int(key)
                        stars, secs = int(rec[0]), int(rec[1])
                    except (TypeError, ValueError, IndexError, KeyError):
                        continue
                    if 1 <= lvl <= sudoku_gen.LEVELS:
                        out[str(lvl)] = [max(1, min(3, stars)), max(0, secs)]
                if out:
                    out_v[d] = out
            self.best[v] = out_v

        saves = data.get("saves") if isinstance(data.get("saves"), dict) else {}
        self.saves = {k: v for k, v in saves.items()
                      if isinstance(k, str) and isinstance(v, dict)}

        raw_last = data.get("last") if isinstance(data.get("last"), dict) else {}
        self.last = {}
        for v in VARIANTS:
            lv = raw_last.get(v) if isinstance(raw_last.get(v), dict) else {}
            self.last[v] = {d: max(1, min(100, n)) for d, n in lv.items()
                            if d in ("0", "1", "2", "3") and isinstance(n, int)
                            and not isinstance(n, bool)}

        dl = data.get("daily") if isinstance(data.get("daily"), dict) else {}
        self.daily_state = {}
        if isinstance(dl.get("date"), str) and len(dl["date"]) == 10:
            self.daily_state["date"] = dl["date"]
        for k in ("streak", "best", "time", "stars", "count"):
            if isinstance(dl.get(k), int) and not isinstance(dl.get(k), bool):
                self.daily_state[k] = max(0, dl[k])

    def _save_progress(self):
        data = dict(self._prog_extra)
        data.update(solved=self.solved, best=self.best, saves=self.saves,
                    last=self.last, daily=self.daily_state)
        store.save_section("sudoku", data)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("sudoku", {})[key] = value
            settings_mod.save_settings(self.settings)

    # ----- Level-Daten -----------------------------------------------------
    def _get_last(self, variant, diff):
        if variant == "classic":
            n = self._last_level.get(str(diff), 1)
        else:
            n = self.last.get(variant, {}).get(str(diff), 1)
        return max(1, min(100, n)) if isinstance(n, int) else 1

    def _set_last(self, variant, diff, level):
        if variant == "classic":
            self._last_level[str(diff)] = level
            self._save_setting("last_level", dict(self._last_level))
        else:
            self.last.setdefault(variant, {})[str(diff)] = level

    def _record(self, variant, diff, level):
        """(sterne, sekunden oder None) eines Levels - (0, None) = ungelöst."""
        rec = self.best.get(variant, {}).get(str(diff), {}).get(str(level))
        if rec:
            return rec[0], rec[1]
        if variant == "classic" and level in self.solved.get(str(diff), ()):
            return 1, None          # vor den Sternen gelöst -> ein Stern
        return 0, None

    def _stage_summary(self, variant, diff):
        """(gelöste Level, Sterne) einer Stufe."""
        n = stars = 0
        for lvl in range(1, sudoku_gen.LEVELS + 1):
            s, _ = self._record(variant, diff, lvl)
            if s:
                n += 1
                stars += s
        return n, stars

    def three_star_count(self):
        """Anzahl Level (alle Varianten) mit drei Sternen."""
        return sum(1 for vb in self.best.values() for db in vb.values()
                   for rec in db.values() if rec[0] >= 3)

    def _save_key(self):
        if self.daily:
            return "daily:" + self.daily_date
        return "%s:%d:%d" % (self.play_variant, self.play_diff, self.level)

    def _level_save(self, variant, diff, level):
        return self.saves.get("%s:%d:%d" % (variant, diff, level))

    def daily_streak(self):
        """Laufende Serie: zählt nur, wenn heute oder gestern gelöst wurde."""
        last = self.daily_state.get("date")
        if not last:
            return 0
        try:
            gap = seedrand.day_index(self.daily_date) - seedrand.day_index(last)
        except ValueError:
            return 0
        return self.daily_state.get("streak", 0) if gap in (0, 1) else 0

    def daily_done_today(self):
        return self.daily_state.get("date") == self.daily_date

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        W, H = self.width, self.height
        cx = W // 2
        m = max(10, W // 48)
        gap = max(6, H // 90)
        self.su_title_y = max(20, int(H * 0.06))
        self.su_sub_y = (self.su_title_y + self._title.get_height() // 2
                         + self._tiny.get_height() // 2 + 1)
        row_w = W - 2 * m if W < 900 else int(W * 0.8)
        left = cx - row_w // 2
        bh = max(26, min(44, H // 15))
        y0 = self.su_sub_y + self._tiny.get_height() // 2 + gap + 2

        def row(y, n):
            cw = (row_w - gap * (n - 1)) / n
            return [pygame.Rect(int(left + i * (cw + gap)), y, int(cw), bh)
                    for i in range(n)]

        self.var_rects = row(y0, 4)
        self.diff_rects = row(y0 + bh + gap, 4)
        body_top = y0 + 2 * bh + 3 * gap
        foot = self._tiny.get_height() + 10
        prog_h = self._small.get_height() + 6
        avail_h = H - foot - body_top - prog_h
        grid = min(avail_h, int(row_w * 0.57))
        cell = max(18, grid // 10)
        # übrige Höhe (breite Fenster) teilweise oben einschieben
        body_top += max(0, (avail_h - 10 * cell) // 3)
        self.lv_cell = cell
        self.lv_x = left
        self.lv_y = body_top
        self._lv_font = ui.font(max(10, cell * 2 // 5), mono=True)
        self.prog_y = body_top + 10 * cell + prog_h // 2 + 1

        # Rechte Spalte: Tages-Sudoku, Schalter, Level-Info
        px = left + 10 * cell + 2 * gap
        pw = left + row_w - px
        self.panel_rect = pygame.Rect(px, body_top, pw, 10 * cell)
        pad = max(6, gap)
        ix, iw = px + pad, pw - 2 * pad
        dh = self._small.get_height() + 2 * self._tiny.get_height() + 12
        self.daily_rect = pygame.Rect(ix, body_top + pad, iw, dh)
        th = max(22, min(34, H // 22))
        ty = self.daily_rect.bottom + gap
        self.limit_rect = pygame.Rect(ix, ty, iw, th)
        self.input_rect = pygame.Rect(ix, ty + th + 4, iw, th)
        self.colors_rect = pygame.Rect(ix, ty + 2 * (th + 4), iw, th)
        info_top = self.colors_rect.bottom + gap
        self.info_rect = pygame.Rect(ix, info_top, iw,
                                     max(0, self.panel_rect.bottom - pad - info_top))
        # Start-Knopf unten in der Info, wenn genug Platz ist
        need = th + 4 + self._small.get_height() + 2 * self._tiny.get_height() + 8
        if self.info_rect.h >= need:
            bh2 = max(th, min(40, H // 18))
            self.start_rect = pygame.Rect(ix, self.info_rect.bottom - bh2, iw, bh2)
            self.info_rect.h -= bh2 + 6
        else:
            self.start_rect = None

    def _level_at(self, pos):
        """Pixel -> Levelnummer 1..100 (oder None)."""
        x, y = pos
        c = (x - self.lv_x) // self.lv_cell
        r = (y - self.lv_y) // self.lv_cell
        if 0 <= c < 10 and 0 <= r < 10:
            return int(r * 10 + c + 1)
        return None

    def _select_variant(self, v):
        if v not in VARIANTS:
            return
        self.variant = v
        self._save_setting("variant", v)
        self.cursor = self._get_last(v, self.diff)
        self.play_sound("click")

    def _select_difficulty(self, i):
        self.diff = max(0, min(3, i))
        self._save_setting("difficulty", self.diff)
        self.cursor = self._get_last(self.variant, self.diff)
        self.play_sound("click")

    def _toggle_fail_limit(self):
        self.fail_limit = not self.fail_limit
        self._save_setting("fail_limit", self.fail_limit)
        self.play_sound("select")

    def _toggle_input(self):
        self.input_mode = "cell" if self.input_mode == "digit" else "digit"
        self._save_setting("input", self.input_mode)
        self.play_sound("select")

    def _toggle_colors(self):
        self.colors_on = not self.colors_on
        self._save_setting("colors", self.colors_on)
        self.play_sound("select")

    def _setup_hit(self, pos):
        """Welches Setup-Element liegt unter pos? (für Klick + Hover)"""
        for i, r in enumerate(self.var_rects):
            if r.collidepoint(pos):
                return ("variant", i)
        for i, r in enumerate(self.diff_rects):
            if r.collidepoint(pos):
                return ("diff", i)
        for key, r in (("daily", self.daily_rect), ("limit", self.limit_rect),
                       ("input", self.input_rect), ("colors", self.colors_rect),
                       ("start", self.start_rect)):
            if r is not None and r.collidepoint(pos):
                return (key, 0)
        return None

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3", "4"):
                self._select_difficulty(int(k) - 1)
            elif k in ("v", "V") and self.key_is_free(k):
                step = -1 if k == "V" else 1
                self._select_variant(VARIANTS[(VARIANTS.index(self.variant) + step)
                                              % len(VARIANTS)])
            elif k in ("f", "F") and self.key_is_free(k):
                self._toggle_fail_limit()
            elif k in ("i", "I") and self.key_is_free(k):
                self._toggle_input()
            elif k in ("m", "M") and self.key_is_free(k):
                self._toggle_colors()
            elif k in ("t", "T") and self.key_is_free(k):
                self._start_daily()
            elif k in ("Left", "a", "A"):
                self.cursor = (self.cursor - 2) % 100 + 1
                self.play_sound("move")
            elif k in ("Right", "d", "D"):
                self.cursor = self.cursor % 100 + 1
                self.play_sound("move")
            elif k in ("Up", "w", "W"):
                self.cursor = (self.cursor - 11) % 100 + 1
                self.play_sound("move")
            elif k in ("Down", "s", "S"):
                self.cursor = (self.cursor + 9) % 100 + 1
                self.play_sound("move")
            elif k in ("Return", "space", "KP_Enter"):
                self._start_level(self.cursor)
        elif event.kind == InputEvent.MOUSEMOVE:
            lv = self._level_at(event.pos)
            if lv is not None:
                self.cursor = lv
            self.hover = self._setup_hit(event.pos)
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            hit = self._setup_hit(event.pos)
            if hit is not None:
                kind, i = hit
                if kind == "variant":
                    self._select_variant(VARIANTS[i])
                elif kind == "diff":
                    self._select_difficulty(i)
                elif kind == "daily":
                    self._start_daily()
                elif kind == "limit":
                    self._toggle_fail_limit()
                elif kind == "input":
                    self._toggle_input()
                elif kind == "colors":
                    self._toggle_colors()
                elif kind == "start":
                    self._start_level(self.cursor)
                return
            lv = self._level_at(event.pos)
            if lv is not None:
                self._start_level(lv)

    # ===================================================== Level starten
    def _begin_generate(self):
        self.game_over = False
        self.won = False
        self.score = 0
        self.reveal = False
        self.hover = None
        self._gen_drawn = False
        self.state = GENERATING
        self.play_sound("click")

    def _start_level(self, n, fresh=False):
        if self.variant == "killer" and sudoku_gen.killer_level(self.diff, 1) is None:
            self._say(t("sud.killer_missing"), ui.RED)
            self.play_sound("hit")
            return
        self.daily = False
        self.play_variant = self.variant
        self.play_diff = self.diff
        self.level = max(1, min(sudoku_gen.LEVELS, int(n)))
        self.cursor = self.level
        self._set_last(self.variant, self.diff, self.level)
        self._fresh = fresh
        self._begin_generate()

    def _start_daily(self, fresh=False):
        self.daily = True
        self.daily_date = seedrand.today_str()
        self.play_variant = "classic"
        self.play_diff = sudoku_gen.daily_diff(self.daily_date)
        self.level = 0
        self._fresh = fresh
        self._begin_generate()

    def _restart(self):
        """R: dasselbe Rätsel von vorn (Spielstand wird verworfen)."""
        if self.daily:
            self._start_daily(fresh=True)
        else:
            self.variant, self.diff = self.play_variant, self.play_diff
            self._start_level(self.level, fresh=True)

    def _back_to_setup(self):
        if self.state == PLAY and not self.game_over:
            self._write_save()
        self.state = SETUP
        self.game_over = False
        self.hover = None
        self.msg = None
        self.daily_date = seedrand.today_str()
        if not self.daily:
            self.variant, self.diff = self.play_variant, self.play_diff
            self.cursor = self.level
        self.play_sound("click")

    def _do_generate(self):
        """Erzeugt/lädt das Rätsel (blockierend, < 1 s) und baut das Brett auf."""
        v = self.play_variant
        cages = []
        if self.daily:
            puzzle, solution, self.play_diff = sudoku_gen.generate_daily(self.daily_date)
        elif v == "classic":
            puzzle, solution = sudoku_gen.generate(self.play_diff, self.level)
        elif v == "killer":
            data = sudoku_gen.killer_level(self.play_diff, self.level)
            if data is None:                   # Datei unvollständig
                self.state = SETUP
                self._say(t("sud.killer_missing"), ui.RED)
                return
            puzzle, solution, cages = data
        else:
            puzzle, solution = sudoku_gen.generate_variant(v, self.play_diff, self.level)
        self.lay = lay = sudoku_gen.layout_for("classic" if self.daily else v)
        self.N = lay.n
        self.cells = lay.size
        self.puzzle = list(puzzle)
        self.solution = list(solution)
        self.cages = [(int(total), tuple(cells)) for total, cells in cages]
        self.cage_of = [-1] * self.cells
        for ci, (_, cells) in enumerate(self.cages):
            for i in cells:
                self.cage_of[i] = ci
        # Killer: auch die Käfig-Nachbarn dürfen keine gleiche Ziffer haben.
        self.peers = []
        for i in range(self.cells):
            ps = set(lay.peers[i])
            if self.cage_of[i] >= 0:
                ps.update(self.cages[self.cage_of[i]][1])
                ps.discard(i)
            self.peers.append(tuple(sorted(ps)))
        # Einheiten je Zelle (für "Einheit komplett"-Effekte), inkl. Käfig
        self.groups = [tuple(u) for u in lay.units] + [c for _, c in self.cages]
        self.groups_of = [[] for _ in range(self.cells)]
        for g, cells in enumerate(self.groups):
            for i in cells:
                self.groups_of[i].append(g)

        self.board = list(puzzle)
        self.given = [v != 0 for v in puzzle]
        self.locked = list(self.given)
        self.notes = [0] * self.cells          # Bitmaske, Bit d-1 = Ziffer d
        self.marks = [0] * self.cells          # Farbmarker 0..MARK_COUNT
        self.wrong = set()
        self.hinted = set()
        self.sel = next((i for i in range(self.cells) if not self.given[i]), 0)
        self.errors = 0
        self.hints_used = 0
        self.elapsed = 0.0
        self.mult = MODE_MULT.get(self.mode, 1.0)
        self.note_mode = False
        self.brush = 0                         # "Ziffer zuerst": gewählte Ziffer
        self.mark_brush = 0
        self.reveal = False
        self.msg = None
        self.msg_t = 0.0
        self.undo_stack = []
        self.redo_stack = []
        self._step = None
        self._dirty = False
        self._save_t = 0.0
        self._conflicts = frozenset()
        self._bad_cages = frozenset()
        self.fx_pop = {}
        self.fx_shake = {}
        self.fx_units = []
        self._win_ticks = 0
        self._star_snd = set()
        self.result_stars = 0
        self.new_record = False

        if self._fresh:
            if self.saves.pop(self._save_key(), None) is not None:
                self._save_progress()
        elif self._restore_save():
            self._say(t("sud.resumed"), self.accent)
        self._fresh = False
        self._update_conflicts()
        self._reset_caches()
        self._build_play_layout()
        self.state = PLAY

    # ----- Spielstand sichern / fortsetzen --------------------------------
    def _has_progress(self):
        return bool(self.errors or self.hints_used or any(self.notes)
                    or any(self.marks)
                    or any(b != p for b, p in zip(self.board, self.puzzle)))

    def _write_save(self):
        """Sichert das laufende Rätsel in mem.json (nur mit Fortschritt)."""
        self._dirty = False
        self._save_t = 0.0
        key = self._save_key()
        if not self._has_progress():
            if self.saves.pop(key, None) is not None:
                self._save_progress()
            return
        todo = sum(1 for p in self.puzzle if not p) or 1
        filled = sum(1 for i in range(self.cells)
                     if not self.given[i] and self.board[i])
        self.saves[key] = {
            "b": "".join(map(str, self.board)),
            "n": list(self.notes),
            "k": "".join(map(str, self.marks)),
            "l": [i for i in range(self.cells)
                  if self.locked[i] and not self.given[i]],
            "h": sorted(self.hinted),
            "e": self.errors,
            "hu": self.hints_used,
            "t": round(self.elapsed, 1),
            "m": self.mult,
            "p": int(100 * filled / todo),
            "ts": int(time.time()),
        }
        if len(self.saves) > MAX_SAVES:
            # Älteste Spielstände zuerst verwerfen (der aktuelle bleibt).
            old = sorted((v.get("ts", 0) if isinstance(v.get("ts"), int) else 0, k)
                         for k, v in self.saves.items() if k != key)
            for _, k in old[:len(self.saves) - MAX_SAVES]:
                del self.saves[k]
        self._save_progress()

    def _restore_save(self):
        """Setzt einen gespeicherten Stand fort. True bei Erfolg."""
        raw = self.saves.get(self._save_key())
        if not isinstance(raw, dict):
            return False
        n, size = self.N, self.cells
        try:
            board = [int(ch) for ch in raw["b"]]
            if len(board) != size or any(not 0 <= v <= n for v in board):
                return False
            if any(self.given[i] and board[i] != self.puzzle[i] for i in range(size)):
                return False
            notes = raw.get("n", [])
            notes = [int(x) & self.lay.all for x in notes] \
                if isinstance(notes, list) and len(notes) == size else [0] * size
            marks = [int(ch) for ch in raw.get("k", "")]
            if len(marks) != size or any(not 0 <= v <= MARK_COUNT for v in marks):
                marks = [0] * size
            locked = {int(i) for i in raw.get("l", []) if 0 <= int(i) < size}
            hinted = {int(i) for i in raw.get("h", []) if 0 <= int(i) < size}
            errors = max(0, int(raw.get("e", 0)))
            hints = max(0, min(MAX_HINTS, int(raw.get("hu", 0))))
            elapsed = max(0.0, float(raw.get("t", 0.0)))
            mult = float(raw.get("m", self.mult))
        except (TypeError, ValueError, KeyError):
            return False
        if self.fail_limit and errors >= FAIL_LIMIT:
            return False
        self.board = board
        for i in range(size):
            if board[i]:
                notes[i] = 0
        self.notes = notes
        self.marks = marks
        self.wrong = {i for i in range(size) if board[i] and board[i] != self.solution[i]}
        for i in locked:
            if board[i] and board[i] == self.solution[i]:
                self.locked[i] = True
        self.hinted = {i for i in hinted if self.locked[i] and not self.given[i]}
        self.errors = errors
        self.hints_used = hints
        self.elapsed = elapsed
        if 0.0 < mult <= 2.0:
            self.mult = min(self.mult, mult)
        self.sel = next((i for i in range(size) if not self.board[i]), self.sel)
        return True

    def _delete_save(self):
        if self.saves.pop(self._save_key(), None) is not None:
            return True
        return False

    # ===================================================== Spiel-Layout
    def _build_play_layout(self):
        W, H = self.width, self.height
        N = self.lay.n
        self.hud_h = max(40, H // 12)
        foot = self._tiny.get_height() + 12

        # Brett links/mittig, Ziffernfeld rechts daneben, unten die
        # Steuerungs-Hinweiszeile.
        pad_w = max(128, W // 5)
        size = min(H - self.hud_h - foot - 12, W - pad_w - 44)
        self.cell = max(20, size // N)
        bs = self.cell * N
        self.bs = bs
        self.bx = max(12, (W - pad_w - 24 - bs) // 2)
        self.by = self.hud_h + max(6, (H - self.hud_h - foot - bs) // 2)

        # Mono-Fonts: Ziffern stehen so in jeder Zelle exakt gleich breit.
        self._num_font = ui.font(max(14, self.cell * 3 // 5), bold=True, mono=True)
        if self.cages:
            self._note_font = ui.font(max(7, self.cell * 2 // 9), mono=True)
        else:
            self._note_font = ui.font(max(8, self.cell * 2 // 7), mono=True)
        self._sum_font = ui.font(max(8, self.cell // 4), bold=True)
        self._pad_font = ui.font(max(14, min(self.cell * 3 // 5, 44)), bold=True, mono=True)

        # Ziffernfeld: Ziffern in 3 Spalten, darunter Funktionsknöpfe
        # (Radieren/Rückgängig/Wiederholen, Notizen/Kandidaten/Tipp) und
        # optional die Farbmarker.
        px = self.bx + bs + 24
        pb = max(24, min((W - px - 12 - 8) // 3, self.cell + 8))
        fw = 3 * pb + 8
        fh = max(22, pb * 2 // 3)
        rows = (N + 2) // 3
        func_rows = [["erase", "undo", "redo"]]
        second = [k for k, ok in (("note", self.can_notes), ("cands", self.can_notes),
                                  ("hint", self.can_hint)) if ok]
        if second:
            func_rows.append(second)
        sh = max(14, fh * 2 // 3) if self.colors_on else 0
        cap_h = 2 * self._tiny.get_height() + 6
        total = rows * (pb + 4) + 4 + len(func_rows) * (fh + 4) \
            + (sh + 6 if sh else 0) + cap_h
        py = self.by + max(0, (bs - total) // 2)
        self.pad_rects = {}
        for d in range(1, N + 1):
            r, c = (d - 1) // 3, (d - 1) % 3
            self.pad_rects[str(d)] = pygame.Rect(px + c * (pb + 4), py + r * (pb + 4), pb, pb)
        y = py + rows * (pb + 4) + 4
        for keys in func_rows:
            for c, key in enumerate(keys):
                self.pad_rects[key] = pygame.Rect(px + c * (pb + 4), y, pb, fh)
            y += fh + 4
        if sh:
            y += 2
            sw = (fw - (MARK_COUNT - 1) * 3) // MARK_COUNT
            for k in range(MARK_COUNT):
                self.pad_rects["mark%d" % (k + 1)] = pygame.Rect(px + k * (sw + 3), y, sw, sh)
            y += sh + 4
        self.pad_x = px
        self.pad_w = fw
        self.caption_y = y + 2
        self.caption_bottom = min(H - foot, y + 2 + cap_h)

    def _cell_at(self, pos):
        x, y = pos
        c = (x - self.bx) // self.cell
        r = (y - self.by) // self.cell
        if 0 <= c < self.N and 0 <= r < self.N:
            return int(r * self.N + c)
        return None

    def _pad_at(self, pos):
        for key, r in self.pad_rects.items():
            if r.collidepoint(pos):
                return key
        return None

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.state != PLAY:
            return
        if self.game_over:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space", "KP_Enter"):
                    if self.won:
                        self._next_level()
                    else:
                        self._restart()
                elif event.key in ("s", "S", "q", "Q"):
                    self._back_to_setup()
                elif event.key in ("a", "A"):
                    # Banner ausblenden und die Lösung auf dem Brett zeigen
                    # (nochmal A = zurück zum Banner).
                    self.reveal = not self.reveal
                    self.play_sound("select")
            elif event.kind == InputEvent.MOUSEDOWN and self.reveal:
                self.reveal = False
            return

        if event.kind == InputEvent.KEYDOWN:
            self._handle_play_key(event.key)
        elif event.kind == InputEvent.MOUSEMOVE:
            self.hover = self._pad_at(event.pos)
        elif event.kind == InputEvent.MOUSEDOWN:
            cell = self._cell_at(event.pos)
            if event.button == 3:
                if cell is not None:
                    self.sel = cell
                    self._erase(cell)
                return
            if cell is not None:
                self.sel = cell
                if self.input_mode == "digit" and self.brush:
                    self._apply_brush(cell)
                else:
                    self.play_sound("move")
                return
            pad = self._pad_at(event.pos)
            if pad is None:
                return
            if pad.isdigit():
                d = int(pad)
                if self.input_mode == "digit":
                    self._set_brush(0 if self.brush == d else d)
                else:
                    self._enter_digit(self.sel, d)
            elif pad.startswith("mark"):
                self._set_mark(self.sel, int(pad[4:]))
            else:
                self._pad_action(pad)

    def _pad_action(self, key):
        if key == "erase":
            self._erase(self.sel)
        elif key == "undo":
            self._undo()
        elif key == "redo":
            self._redo()
        elif key == "note":
            self._toggle_note_mode()
        elif key == "cands":
            self._auto_candidates()
        elif key == "hint":
            self._use_hint()

    def _handle_play_key(self, k):
        n = self.N
        if k in _MOVE:
            dr, dc = _MOVE[k]
            r, c = divmod(self.sel, n)
            self.sel = ((r + dr) % n) * n + (c + dc) % n
            self.play_sound("move")
            return
        digit = None
        if len(k) == 1 and k.isdigit():
            digit = int(k)
        elif k.startswith("KP_") and len(k) == 4 and k[3].isdigit():
            digit = int(k[3])
        if digit is not None and 1 <= digit <= n:
            if self.input_mode == "digit":
                self._set_brush(0 if self.brush == digit else digit)
            else:
                self._enter_digit(self.sel, digit)
        elif digit == 0 or k in ("BackSpace", "Delete"):
            self._erase(self.sel)
        elif k in ("space", "Return", "KP_Enter"):
            if self.input_mode == "digit" and self.brush:
                self._apply_brush(self.sel)
        elif k in ("n", "N"):
            self._toggle_note_mode()
        elif k in ("c", "C") and self.key_is_free(k):
            self._auto_candidates()
        elif k in ("u", "U", "z", "Z") and self.key_is_free(k):
            self._undo()
        elif k in ("y", "Y") and self.key_is_free(k):
            self._redo()
        elif k in ("m", "M") and self.key_is_free(k):
            if self.colors_on:
                self._set_mark(self.sel, (self.marks[self.sel] + 1) % (MARK_COUNT + 1),
                               toggle=False)
        elif k in ("h", "H"):
            self._use_hint()
        elif k in ("r", "R"):
            self._restart()
        elif k in ("q", "Q"):
            self._back_to_setup()

    def _set_brush(self, d):
        self.brush = d
        self.play_sound("select" if d else "move")

    def _apply_brush(self, idx):
        """"Ziffer zuerst": gewählte Ziffer in die Zelle setzen (bzw. Notiz)."""
        d = self.brush
        if not d or self.locked[idx]:
            self.play_sound("move")
            return
        if not self.note_mode and self.board[idx] == d:
            self._erase(idx)                   # nochmal klicken = wieder weg
        else:
            self._enter_digit(idx, d)

    # ===================================================== Rückgängig/Wiederholen
    def _cell_state(self, i):
        return (self.board[i], self.notes[i], self.marks[i])

    def _begin(self, primary):
        self._step = {"cell": primary, "before": {}}

    def _snap(self, *cells):
        before = self._step["before"]
        for i in cells:
            if i not in before:
                before[i] = self._cell_state(i)

    def _commit(self):
        step, self._step = self._step, None
        changes = []
        for i, before in step["before"].items():
            after = self._cell_state(i)
            if after != before:
                changes.append((i, before, after))
        if changes:
            self.undo_stack.append((step["cell"], changes))
            if len(self.undo_stack) > MAX_UNDO:
                del self.undo_stack[0]
            self.redo_stack.clear()
            self._touch()

    def _apply_state(self, i, st):
        self.board[i], self.notes[i], self.marks[i] = st
        if self.board[i] and self.board[i] != self.solution[i]:
            self.wrong.add(i)
        else:
            self.wrong.discard(i)

    def _undo(self):
        """Letzten Schritt zurücknehmen. Eingerastete Ziffern (Komfort, Tipps)
        bleiben stehen - Schritte, die genau so eine Zelle betrafen, entfallen."""
        if self.game_over:
            return
        while self.undo_stack:
            primary, changes = self.undo_stack.pop()
            if primary >= 0 and self.locked[primary]:
                continue
            for i, before, _after in changes:
                if not self.locked[i]:
                    self._apply_state(i, before)
            self.redo_stack.append((primary, changes))
            if primary >= 0:
                self.sel = primary
            self._update_conflicts()
            self._touch()
            self.play_sound("rotate")
            return
        self.play_sound("move")

    def _redo(self):
        if self.game_over:
            return
        while self.redo_stack:
            primary, changes = self.redo_stack.pop()
            if primary >= 0 and self.locked[primary]:
                continue
            for i, _before, after in changes:
                if not self.locked[i]:
                    self._apply_state(i, after)
                    # Komfort: korrekte Ziffern rasten wieder ein.
                    if self.can_check and self.board[i] \
                            and self.board[i] == self.solution[i]:
                        self.locked[i] = True
            self.undo_stack.append((primary, changes))
            if primary >= 0:
                self.sel = primary
            self._update_conflicts()
            self._touch()
            self.play_sound("rotate")
            self._check_win()
            return
        self.play_sound("move")

    def _touch(self):
        """Etwas hat sich geändert -> gedrosselt speichern."""
        self._dirty = True

    # ===================================================== Spiellogik
    def _toggle_note_mode(self):
        if not self.can_notes:
            return
        self.note_mode = not self.note_mode
        self.play_sound("select")

    def _enter_digit(self, idx, d):
        if self.locked[idx] or not 1 <= d <= self.N:
            return
        if self.note_mode:
            self._toggle_note(idx, d)
            return
        if self.board[idx] == d:
            return
        self._begin(idx)
        self._snap(idx, *self.peers[idx])
        self.board[idx] = d
        self.notes[idx] = 0
        # Notizen der Nachbarn bereinigen - bei JEDER Eingabe, sonst verriete
        # das Verschwinden der Notizen im Notizen-Modus die richtige Ziffer.
        if self.can_notes:
            self._prune_notes(idx, d)
        now = pygame.time.get_ticks()
        if d == self.solution[idx]:
            self.wrong.discard(idx)
            if self.can_check:
                self.locked[idx] = True          # korrekt -> rastet ein
            self._commit()
            self.fx_pop[idx] = now
            self._update_conflicts()
            if self._check_units(idx, now):
                self.play_sound("line")
            else:
                self.play_sound("select")
            self._check_win()
        else:
            self.wrong.add(idx)
            self.errors += 1
            self._commit()
            self.fx_shake[idx] = now
            self.play_sound("hit")
            self.rumble(120)
            self._update_conflicts()
            if self.fail_limit and self.errors >= FAIL_LIMIT:
                self._lose()
            else:
                # Auch ein durch eine falsche Ziffer voll gewordenes Brett
                # prüfen -> zeigt in classic/notes die "noch Fehler"-Meldung.
                self._check_win()

    def _toggle_note(self, idx, d):
        if self.board[idx] or not self.can_notes:
            return
        self._begin(idx)
        self._snap(idx)
        self.notes[idx] ^= 1 << (d - 1)
        self._commit()
        self.play_sound("move")

    def _erase(self, idx):
        if self.locked[idx]:
            return
        if self.board[idx] or self.notes[idx] or self.marks[idx]:
            self._begin(idx)
            self._snap(idx)
            if self.board[idx] or self.notes[idx]:
                self.board[idx] = 0
                self.notes[idx] = 0
            else:
                self.marks[idx] = 0             # leere Zelle: Farbe weg
            self.wrong.discard(idx)
            self._commit()
            self._update_conflicts()
            self.play_sound("move")

    def _set_mark(self, idx, color, toggle=True):
        if not self.colors_on or not 0 <= color <= MARK_COUNT:
            return
        if toggle and self.marks[idx] == color:
            color = 0
        self._begin(idx)
        self._snap(idx)
        self.marks[idx] = color
        self._commit()
        self.mark_brush = color
        self.play_sound("select")

    def _prune_notes(self, idx, d):
        """Entfernt die Ziffer d aus den Notizen aller Nachbar-Zellen."""
        m = ~(1 << (d - 1))
        for j in self.peers[idx]:
            self.notes[j] &= m

    def candidates(self, i):
        """Kandidaten einer leeren Zelle nach dem aktuellen Brett (Bitmaske)."""
        used = 0
        for j in self.peers[i]:
            if self.board[j]:
                used |= 1 << (self.board[j] - 1)
        return self.lay.all & ~used

    def _auto_candidates(self):
        """Trägt in alle leeren Zellen die noch möglichen Ziffern als Notiz ein."""
        if not self.can_notes or self.game_over:
            return
        self._begin(-1)
        for i in range(self.cells):
            if self.board[i] or self.locked[i]:
                continue
            self._snap(i)
            self.notes[i] = self.candidates(i)
        self._commit()
        self._say(t("sud.cands_done"), self.accent)
        self.play_sound("powerup")

    def _use_hint(self):
        if not self.can_hint or self.hints_used >= MAX_HINTS or self.game_over:
            return
        idx = self.sel
        if self.locked[idx] or (self.board[idx] and idx not in self.wrong):
            empties = [i for i in range(self.cells)
                       if not self.locked[i] and self.board[i] != self.solution[i]]
            if not empties:
                return
            idx = random.choice(empties)
        d = self.solution[idx]
        self.board[idx] = d
        self.notes[idx] = 0
        self.wrong.discard(idx)
        self.locked[idx] = True
        self.hinted.add(idx)
        self.hints_used += 1
        self._prune_notes(idx, d)
        self.sel = idx
        now = pygame.time.get_ticks()
        self.fx_pop[idx] = now
        self._touch()
        self.play_sound("powerup")
        self._update_conflicts()
        self._check_units(idx, now)
        self._check_win()

    def _check_units(self, idx, now):
        """Einheiten (Zeile/Spalte/Block/Diagonale/Käfig) um idx komplett
        und richtig? -> Leucht-Welle. Rückgabe: ob mindestens eine fertig ist."""
        done = False
        for g in self.groups_of[idx]:
            cells = self.groups[g]
            if all(self.board[i] == self.solution[i] for i in cells):
                self.fx_units.append((cells, idx, now))
                done = True
        if len(self.fx_units) > 12:
            del self.fx_units[:-12]
        return done

    def _update_conflicts(self):
        """Zellen, deren Ziffer mit einem Nachbarn kollidiert, und Käfige mit
        unmöglicher Summe (nur comfort+)."""
        if not self.can_check:
            return
        bad = set()
        for i in range(self.cells):
            v = self.board[i]
            if v and any(self.board[j] == v for j in self.peers[i]):
                bad.add(i)
        bad_cages = set()
        for ci, (total, cells) in enumerate(self.cages):
            vals = [self.board[i] for i in cells if self.board[i]]
            s = sum(vals)
            if len(set(vals)) != len(vals) or s > total \
                    or (len(vals) == len(cells) and s != total) \
                    or (len(vals) < len(cells) and s >= total):
                bad_cages.add(ci)
        self._conflicts = frozenset(bad)
        self._bad_cages = frozenset(bad_cages)

    def _check_win(self):
        if 0 in self.board:
            return
        if self.board == self.solution:
            self._win()
        elif not self.can_check:
            # Voll, aber falsch: kurzer Hinweis (Fehler wurden schon gezählt).
            self._say(t("sud.full_wrong"), ui.RED)

    def _say(self, text, col=None, secs=2.5):
        self.msg = text
        self.msg_col = col
        self.msg_t = secs

    def _win(self):
        self.won = True
        self.game_over = True
        v, d = self.play_variant, self.play_diff
        secs = int(self.elapsed)
        base = BASE_POINTS[v][d]
        raw = base - TIME_COST * secs - ERR_COST * self.errors \
            - HINT_COST * self.hints_used
        self.score = int(max(50, raw) * self.mult)
        stars = stars_for(v, d, secs, self.errors, self.hints_used)
        self.result_stars = stars
        self.result_time = secs
        self.new_record = False
        self.best_time = secs
        if self.daily:
            st = self.daily_state
            if st.get("date") == self.daily_date:
                old = st.get("time", secs + 1)
                self.new_record = secs < old
                st["time"] = min(old, secs)
                st["stars"] = max(st.get("stars", 0), stars)
            else:
                streak = self.daily_streak() + 1 if self.daily_streak() else 1
                st.update(date=self.daily_date, streak=streak,
                          best=max(st.get("best", 0), streak), time=secs,
                          stars=stars, count=st.get("count", 0) + 1)
            self.best_time = st["time"]
        else:
            levels = self.best.setdefault(v, {}).setdefault(str(d), {})
            old = levels.get(str(self.level))
            if old:
                self.new_record = secs < old[1]
                levels[str(self.level)] = [max(old[0], stars), min(old[1], secs)]
            else:
                levels[str(self.level)] = [stars, secs]
            self.best_time = levels[str(self.level)][1]
            if v == "classic" and self.level not in self.solved[str(d)]:
                self.solved[str(d)] = sorted(self.solved[str(d)] + [self.level])
        self.saves.pop(self._save_key(), None)
        self._save_progress()
        self.report_result(True)
        if self.errors == 0:
            self.ach_event("sudoku_clean")
        if v == "killer" and not self.daily:
            self.ach_event("sudoku_killer")
        self.ach_event("sudoku_stars", self.three_star_count())
        self._win_ticks = pygame.time.get_ticks()
        self._star_snd = set()
        self.play_sound("win")
        self.rumble(200)
        if stars == 3:
            ui.spawn_confetti(self.width, self.height, n=70)

    def _lose(self):
        self.won = False
        self.game_over = True
        self.score = 0
        if self.saves.pop(self._save_key(), None) is not None:
            self._save_progress()
        self.report_result(False)
        self._win_ticks = pygame.time.get_ticks()
        self.play_sound("gameover")
        self.rumble(250)

    def _next_level(self):
        if self.daily or self.level >= sudoku_gen.LEVELS:
            self._back_to_setup()
            return
        self.variant, self.diff = self.play_variant, self.play_diff
        self._start_level(self.level + 1)

    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None
        if self.state == GENERATING:
            # Erst einen Frame "Erzeuge Puzzle..." anzeigen lassen (draw setzt
            # _gen_drawn), dann blockierend generieren.
            if self._gen_drawn:
                self._do_generate()
            return
        if self.state != PLAY or self.game_over:
            return
        self.elapsed += dt
        if self._dirty:
            self._save_t += dt
            if self._save_t >= SAVE_EVERY:
                self._write_save()
