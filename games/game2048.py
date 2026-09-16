# -*- coding: utf-8 -*-
"""
game2048.py
===========
2048 - das Zahlen-Schiebespiel, groß ausgebaut.

Setup-Screen (vor jeder Partie):
- Brettgröße 3x3 bis 8x8.
- Modus: Klassisch (Ziel 2048, danach "Weiterspielen?"), Zeitangriff
  (3 Minuten, so viele Punkte wie möglich - die Uhr startet mit dem ersten
  Zug) und Endlos (ohne Ziel, ohne Unterbrechung).
- Rückgängig: aus / 3 pro Partie / unbegrenzt. Wer es benutzt, spielt ab da
  eine "unterstützte" Partie: kein Highscore, kein Sieg/keine Niederlage in
  der Statistik, keine Kachel-Erfolge - Bestwerte zählen getrennt.
- Bestwerte je Größe/Modus und "Fortsetzen", sobald für Größe/Modus eine
  gespeicherte Partie existiert (Autosave nach Zügen, gedrosselt, und beim
  Verlassen des Spiels).

Spiel:
- Pfeile/WASD oder Wischen mit Maus/Touchpad schieben alle Kacheln. Gehaltene
  Tasten wiederholen NICHT (repeat-Events werden ignoriert).
- Animationen: Kacheln gleiten (ease-out), Verschmelzen poppt, neue Kacheln
  wachsen hinein, Punkte-Popups, Funken ab 128, Ring ab 2048. Eingaben während
  einer Animation werden gepuffert (max. 4) und beschleunigen die Folgezüge.
- Zusatztasten (nur, wenn keiner Aktion zugeordnet): [U]/[Rücktaste] =
  Rückgängig, [R]/[N] = neue Partie, [Tab]/[S] = Setup.

Highscore "2048": NUR 4x4 Klassisch ohne Rückgängig. In allen anderen Partien
bleibt self.score 0 (main.py speichert jeden score als Highscore) - die
Punkte stehen in self.points, die Bestwerte in der mem.json-Section "g2048":

    {"best":          {"4-classic": 12000, ...},   # ohne Rückgängig
     "best_assisted": {"4-classic": 16000, ...},   # mit Rückgängig
     "best_tile":     {"4-classic": 2048, ...},    # größte Kachel je Größe/Modus
     "top_tile":      4096,                         # größte Kachel überhaupt
     "saves":         {"4-classic": {...laufende Partie...}, ...}}

Erfolge: "tile_2048" und "tile_4096" nur in Partien ohne Rückgängig und auf
Brettern bis 4x4 - auf 8x8 wäre eine 4096 reine Fleißarbeit. Statistik
(report_result): Sieg beim ersten Erreichen von 2048 (Klassisch/Endlos),
Niederlage bei Game Over ohne 2048 im Klassik-Modus - beides nur ohne
Rückgängig; der Zeitangriff kennt kein Gewinnen/Verlieren.

Die reine Spiellogik (slide_line, plan_move, spawn_tile, has_moves ...) liegt
als Modul-Funktionen vorn, damit tests/audit_2048.py sie ohne Oberfläche
prüfen kann; web/js/games/game2048.js spiegelt sie 1:1.
"""

import math
import random

import pygame

import audio
import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent
from i18n import t

# ----- Regeln ---------------------------------------------------------------
SIZES = (3, 4, 5, 6, 7, 8)
MODE_KEYS = ("classic", "time", "endless")
UNDO_KEYS = ("off", "limited", "unlimited")
UNDO_LIMIT = 3                   # "3 pro Partie"
TIME_LIMIT = 180.0               # Zeitangriff: 3 Minuten
WIN_TILE = 2048
SPAWN_FOUR = 0.1                 # Anteil der 4er unter den neuen Kacheln
HS_SIZE, HS_MODE = 4, "classic"  # nur diese Kombination füllt den Highscore
ACH_MAX_SIZE = 4                 # Kachel-Erfolge nur auf Brettern bis 4x4

# ----- Speicher ---------------------------------------------------------------
SECTION = "g2048"
STACK_MAX = 256                  # Undo-Tiefe im Speicher ("unbegrenzt")
SAVE_STACK = 3                   # so viele Undo-Stände wandern in mem.json
SAVE_EVERY = 2.5                 # Autosave höchstens alle x Sekunden
VALID_KEYS = {"%d-%s" % (n, m) for n in SIZES for m in MODE_KEYS}

# ----- Animation (Sekunden) ---------------------------------------------------
SLIDE_T = 0.12                   # Gleiten
SLIDE_FAST = 0.07                # Gleiten, wenn weitere Züge gepuffert sind
POP_T = 0.18                     # Verschmelz-Pop
SPAWN_T = 0.16                   # neue Kachel wächst hinein
NUDGE_T = 0.16                   # Brett "stößt an" (Zug ohne Wirkung)
OVER_DELAY = 0.6                 # letzter Zug klingt aus, dann Game Over
FLASH_T = 1.3                    # große Einblendung (2048!, 4096! ...)
MSG_T = 1.8                      # kleine Meldung oben am Brett
POPUP_T = 0.8                    # "+32" am Punktestand
QUEUE_MAX = 4                    # gepufferte Eingaben
MAX_PARTICLES = 260

SETUP, PLAY, WIN = "setup", "play", "win"

# Richtung -> (Pfeiltaste, Aktion, Einheitsvektor dx/dy fürs Anstoßen)
DIRS = {
    "L": ("Left", "left", (-1, 0)),
    "R": ("Right", "right", (1, 0)),
    "U": ("Up", "up", (0, -1)),
    "D": ("Down", "down", (0, 1)),
}

# ----- Identitätsfarben (bewusst fest, unabhängig vom Theme) -----------------
COL_BOARD = (40, 44, 58)
COL_EMPTY = (55, 60, 78)
COL_TEXT_DARK = (104, 94, 84)         # Ziffern auf den hellen 2/4
COL_TEXT_LIGHT = (250, 247, 242)
COL_TILE_BEYOND = (44, 46, 60)        # ab 262144: dunkel mit Goldschrift
COL_TEXT_BEYOND = (255, 214, 102)

# Klassische Farben bis 2048, danach ein eigener Verlauf über Grün, Blau und
# Violett bis Karmin - jede Stufe bleibt klar unterscheidbar.
TILE_COLORS = {
    2: (238, 228, 218), 4: (237, 224, 200), 8: (242, 177, 121),
    16: (245, 149, 99), 32: (246, 124, 95), 64: (246, 94, 59),
    128: (237, 207, 114), 256: (237, 204, 97), 512: (237, 200, 80),
    1024: (237, 197, 63), 2048: (237, 194, 46),
    4096: (92, 196, 146), 8192: (52, 168, 196), 16384: (82, 122, 226),
    32768: (138, 96, 222), 65536: (190, 80, 196), 131072: (228, 70, 118),
}

# Schriftgröße je Stellenzahl (Anteil der Zellgröße).
DIGIT_SCALE = {1: 0.52, 2: 0.50, 3: 0.42, 4: 0.34, 5: 0.28, 6: 0.24}


# =============================================================================
#  Reine Spiellogik (ohne pygame) - von Test und Web-Fassung gespiegelt
# =============================================================================

def empty_grid(n):
    return [[0] * n for _ in range(n)]


def slide_line(vals):
    """Schiebt eine Linie zum Rand bei Index 0 und verschmilzt gleiche Zahlen.

    Jede Kachel verschmilzt pro Zug höchstens einmal: [2, 2, 4, 4] -> [4, 8],
    [2, 2, 2, 2] -> [4, 4], [4, 4, 8] -> [8, 8]. Rückgabe
    (neue_werte, punkte, wege) mit wege = [(von, nach, verschmolzen)];
    "verschmolzen" markiert die zweite Kachel eines Paares.
    """
    out = [0] * len(vals)
    wege = []
    punkte = 0
    pos = -1
    frei = False                 # darf out[pos] in diesem Zug noch verschmelzen?
    for i, v in enumerate(vals):
        if not v:
            continue
        if frei and out[pos] == v:
            out[pos] = v * 2
            punkte += v * 2
            frei = False
            wege.append((i, pos, True))
        else:
            pos += 1
            out[pos] = v
            frei = True
            wege.append((i, pos, False))
    return out, punkte, wege


def line_cells(n, direction, k):
    """Zellen (r, c) der k-ten Linie in Schieberichtung (Index 0 = Zielrand)."""
    if direction == "L":
        return [(k, c) for c in range(n)]
    if direction == "R":
        return [(k, c) for c in range(n - 1, -1, -1)]
    if direction == "U":
        return [(r, k) for r in range(n)]
    return [(r, k) for r in range(n - 1, -1, -1)]


def plan_move(grid, direction):
    """Berechnet einen Zug, ohne 'grid' zu verändern.

    Rückgabe dict(grid, gained, slides, merges, moved):
    slides = [(von_r, von_c, nach_r, nach_c, wert)] für JEDE Kachel (auch
    stehenbleibende), merges = [(r, c, neuer_wert)].
    """
    n = len(grid)
    new = empty_grid(n)
    slides, merges = [], []
    gained = 0
    for k in range(n):
        cells = line_cells(n, direction, k)
        vals = [grid[r][c] for r, c in cells]
        out, pts, wege = slide_line(vals)
        gained += pts
        for i, (r, c) in enumerate(cells):
            new[r][c] = out[i]
        for von, nach, verschmolzen in wege:
            fr, fc = cells[von]
            tr, tc = cells[nach]
            slides.append((fr, fc, tr, tc, vals[von]))
            if verschmolzen:
                merges.append((tr, tc, out[nach]))
    return dict(grid=new, gained=gained, slides=slides, merges=merges,
                moved=(new != grid))


def spawn_tile(grid, rng):
    """Setzt eine neue Kachel (90 % eine 2, 10 % eine 4) auf ein freies Feld.

    Gibt (r, c, wert) zurück oder None, wenn das Brett voll ist.
    """
    frei = [(r, c) for r, row in enumerate(grid) for c, v in enumerate(row) if not v]
    if not frei:
        return None
    r, c = rng.choice(frei)
    v = 4 if rng.random() < SPAWN_FOUR else 2
    grid[r][c] = v
    return (r, c, v)


def has_moves(grid):
    """True, solange ein Zug möglich ist (freies Feld oder gleiche Nachbarn)."""
    n = len(grid)
    for r in range(n):
        for c in range(n):
            v = grid[r][c]
            if not v:
                return True
            if c + 1 < n and v == grid[r][c + 1]:
                return True
            if r + 1 < n and v == grid[r + 1][c]:
                return True
    return False


def max_tile(grid):
    return max((v for row in grid for v in row), default=0)


def valid_grid(grid, n):
    """Brett aus gespeicherten Daten: n x n, nur 0 oder Zweierpotenzen >= 2."""
    if not isinstance(grid, list) or len(grid) != n:
        return False
    for row in grid:
        if not isinstance(row, list) or len(row) != n:
            return False
        for v in row:
            if isinstance(v, bool) or not isinstance(v, int):
                return False
            if v and (v < 2 or v > 2 ** 40 or v & (v - 1)):
                return False
    return True


def _int(v, lo, hi, default):
    if isinstance(v, bool) or not isinstance(v, int):
        return default
    return max(lo, min(hi, v))


def _num(v, lo, hi, default):
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        return default
    return max(lo, min(hi, float(v)))


def _clean_slides(raw, n):
    out = []
    if not isinstance(raw, list):
        return out
    for sl in raw:
        if (isinstance(sl, (list, tuple)) and len(sl) == 5
                and all(isinstance(x, int) and not isinstance(x, bool) for x in sl)
                and all(0 <= x < n for x in sl[:4]) and sl[4] >= 2):
            out.append(tuple(sl))
    return out


def _clean_spawn(raw, n):
    if (isinstance(raw, (list, tuple)) and len(raw) == 3
            and all(isinstance(x, int) and not isinstance(x, bool) for x in raw)
            and 0 <= raw[0] < n and 0 <= raw[1] < n and raw[2] in (2, 4)):
        return tuple(raw)
    return None


def clean_save(sv, n):
    """Prüft einen gespeicherten Spielstand; liefert ein sauberes dict oder None."""
    if not isinstance(sv, dict) or not valid_grid(sv.get("grid"), n):
        return None
    out = dict(
        grid=[row[:] for row in sv["grid"]],
        points=_int(sv.get("points"), 0, 10 ** 12, 0),
        moves=_int(sv.get("moves"), 0, 10 ** 9, 0),
        undo_used=_int(sv.get("undo_used"), 0, 10 ** 9, 0),
        assisted=sv.get("assisted") is True,
        reached=sv.get("reached") is True,
        continued=sv.get("continued") is True,
        peak=_int(sv.get("peak"), 0, 2 ** 40, 0),
        time_left=_num(sv.get("time_left"), 0.0, TIME_LIMIT, TIME_LIMIT),
        clock_on=sv.get("clock_on") is True,
        elapsed=_num(sv.get("elapsed"), 0.0, 1e7, 0.0),
    )
    out["peak"] = max(out["peak"], max_tile(out["grid"]))
    stack = []
    raw = sv.get("stack")
    for e in (raw if isinstance(raw, list) else [])[-SAVE_STACK:]:
        if isinstance(e, dict) and valid_grid(e.get("grid"), n):
            stack.append(dict(grid=[row[:] for row in e["grid"]],
                              points=_int(e.get("points"), 0, 10 ** 12, 0),
                              moves=_int(e.get("moves"), 0, 10 ** 9, 0),
                              reached=e.get("reached") is True,
                              slides=_clean_slides(e.get("slides"), n),
                              spawn=_clean_spawn(e.get("spawn"), n)))
    out["stack"] = stack
    return out


def tile_color(v):
    return TILE_COLORS.get(v, COL_TILE_BEYOND)


def tile_text_color(v):
    if v <= 4:
        return COL_TEXT_DARK
    return COL_TEXT_LIGHT if v in TILE_COLORS else COL_TEXT_BEYOND


def fmt_time(sec):
    sec = max(0, int(math.ceil(sec - 1e-6)))
    return "%d:%02d" % (sec // 60, sec % 60)


def ease_out_cubic(p):
    return 1.0 - (1.0 - p) ** 3


def ease_out_back(p):
    c1 = 1.70158
    q = p - 1.0
    return 1.0 + (c1 + 1.0) * q ** 3 + c1 * q ** 2


def _shade(col, f):
    return (int(col[0] * f), int(col[1] * f), int(col[2] * f))


# =============================================================================
#  Das Spiel
# =============================================================================

class Game2048(Game):
    name = "2048"
    highscore_key = "2048"
    supports_multiplayer = False

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        gs = self.settings.get(SECTION, {}) if isinstance(self.settings, dict) else {}
        size = gs.get("size", 4)
        self.size = size if size in SIZES else 4
        self.gmode = gs.get("mode") if gs.get("mode") in MODE_KEYS else "classic"
        self.undo_rule = gs.get("undo") if gs.get("undo") in UNDO_KEYS else "limited"

        self.rng = random.Random()       # nur für neue Kacheln (im Test seedbar)
        self.data = self._load_data()
        self.anim_t = 0.0
        self.hover = None
        self.drag = None
        self._txt_cache = {}
        self._fit_cache = {}
        self._fit_sizes = {}
        self._tile_cache = {}
        self._board_key = None
        self._board_cache = None
        self._dim_key = None
        self._dim = None

        self.setup_focus = 0              # 0 Größe, 1 Modus, 2 Rückgängig, 3 Knöpfe
        self.setup_act = 0                # 0 Fortsetzen, 1 Neue Partie
        self._blank_game()
        self._make_fonts()
        self._layout()
        self._build_setup_layout()
        self.state = SETUP

    def _blank_game(self):
        """Grundzustand einer Partie (damit draw()/update() nie scheitern)."""
        n = self.size
        self.grid = empty_grid(n)
        self.points = 0
        self.moves = 0
        self.undo_used = 0
        self.assisted = False
        self.reached = False
        self.continued = False
        self.peak = 0
        self.max_tile = 0
        self.time_left = TIME_LIMIT
        self.clock_on = False
        self.elapsed = 0.0
        self.stack = []
        self.queue = []
        self.slide = None
        self.pops = {}
        self.spawns = {}
        self.particles = []
        self.rings = []
        self.popups = []
        self.flash = None
        self.msg = None
        self.nudge = None
        self.over_t = None
        self.over_reason = "stuck"
        self.new_best = False
        self.win_pending = False
        self._panel_ticks = -10 ** 9
        self._best_start = {"best": 0, "best_assisted": 0}
        self._dirty = False
        self._save_cd = SAVE_EVERY

    def _make_fonts(self):
        """Theme-Schriften, Größen aus der Fensterhöhe abgeleitet."""
        h = self.height
        self._tiny = ui.font(max(11, h // 40))
        self._small = ui.font(max(13, h // 32))
        self.font = ui.font(max(15, h // 26))
        self._lab_px = max(10, h // 44)
        self._big_px = max(20, h // 15)
        self._mid_px = max(15, h // 24)
        self._row_px = max(12, h // 34)
        self._f_label = ui.font(self._lab_px)
        self._f_big = ui.font(self._big_px, bold=True)
        self._f_mid = ui.font(self._mid_px, bold=True)
        self._f_row = ui.font(self._row_px)
        self._f_row_b = ui.font(self._row_px, bold=True)
        self._f_title = ui.font(max(24, h // 13), bold=True)
        self._f_setup_title = ui.font(max(28, h // 12), bold=True)

    def on_surface_changed(self):
        """Auflösungswechsel: Layout, Schriften und Caches neu - Partie bleibt."""
        self._make_fonts()
        self._txt_cache.clear()
        self._fit_cache.clear()
        self._fit_sizes.clear()
        self._layout()
        self._build_setup_layout()
        self.particles.clear()
        self.rings.clear()
        self.drag = None

    # ----- Hilfen ----------------------------------------------------------
    def _key(self):
        return "%d-%s" % (self.size, self.gmode)

    def _counts(self):
        """Zählt diese Partie für den Highscore "2048"?"""
        return (self.size == HS_SIZE and self.gmode == HS_MODE
                and not self.assisted)

    @property
    def show_highscore_banner(self):
        return self._counts()

    def _sync_score(self):
        self.score = self.points if self._counts() else 0

    def _undo_left(self):
        if self.undo_rule == "limited":
            return max(0, UNDO_LIMIT - self.undo_used)
        return 0

    def _can_undo(self):
        if self.undo_rule == "off" or not self.stack:
            return False
        if self.game_over and self.over_reason == "time":
            return False             # abgelaufene Zeit holt kein Undo zurück
        return self.undo_rule == "unlimited" or self._undo_left() > 0

    def _free(self, key):
        """Feste Zusatztaste nur, wenn weder klein noch groß einer Aktion gehört."""
        return all(self.key_is_free(k) for k in {key, key.lower(), key.upper()})

    def _tone(self, value):
        """Kleiner Ton, dessen Höhe mit der verschmolzenen Kachel steigt."""
        step = max(0, int(math.log2(max(2, value))) - 6)
        audio.tone(330.0 * 2 ** (step * 2 / 12.0), 0.11, self.settings,
                   wave="triangle", vol=0.22)

    # ===================================================== Persistenz
    def _load_data(self):
        try:
            raw = store.load_section(SECTION)
        except Exception:
            raw = {}
        data = {"best": {}, "best_assisted": {}, "best_tile": {},
                "top_tile": 0, "saves": {}}
        for name in ("best", "best_assisted", "best_tile"):
            src = raw.get(name)
            if isinstance(src, dict):
                for k, v in src.items():
                    if k in VALID_KEYS and _int(v, 1, 10 ** 12, 0) > 0:
                        data[name][k] = v
        data["top_tile"] = _int(raw.get("top_tile"), 0, 2 ** 40, 0)
        saves = raw.get("saves")
        if isinstance(saves, dict):
            for k, sv in saves.items():
                if k in VALID_KEYS:
                    clean = clean_save(sv, int(k.split("-")[0]))
                    if clean is not None:
                        data["saves"][k] = clean
        return data

    def _persist(self):
        try:
            store.save_section(SECTION, self.data)
        except Exception:
            pass

    def _snapshot(self):
        """Laufende Partie als JSON-taugliches dict (für "Fortsetzen")."""
        stack = []
        for e in self.stack[-SAVE_STACK:]:
            stack.append(dict(grid=[row[:] for row in e["grid"]],
                              points=e["points"], moves=e["moves"],
                              reached=e["reached"],
                              slides=[list(sl) for sl in e["slides"]],
                              spawn=list(e["spawn"]) if e["spawn"] else None))
        return dict(grid=[row[:] for row in self.grid], points=self.points,
                    moves=self.moves, undo_used=self.undo_used,
                    assisted=self.assisted, reached=self.reached,
                    continued=self.continued, peak=self.peak,
                    time_left=round(self.time_left, 2), clock_on=self.clock_on,
                    elapsed=round(self.elapsed, 1), stack=stack)

    def _store_game(self):
        """Laufende Partie in self.data ablegen (geschrieben wird per _persist)."""
        if self.state in (PLAY, WIN) and not self.game_over and self.moves > 0:
            self.data["saves"][self._key()] = self._snapshot()

    def _note_best(self):
        """Bestwerte live nachführen (Punkte getrennt mit/ohne Rückgängig)."""
        key = self._key()
        book = self.data["best_assisted" if self.assisted else "best"]
        if self.points > book.get(key, 0):
            book[key] = self.points
        if self.max_tile > self.data["best_tile"].get(key, 0):
            self.data["best_tile"][key] = self.max_tile
        if self.max_tile > self.data["top_tile"]:
            self.data["top_tile"] = self.max_tile

    def _bank_highscore(self):
        """Eine zählende Partie wird ohne Game Over verworfen (neue Partie):
        ihren Stand trotzdem als Highscore sichern - main.py sähe ihn nie."""
        if self.score <= 0 or self.is_menu:
            return
        try:
            import achievements
            import highscore
            hs, _rekord = highscore.update_highscore(self.highscore_key, self.score)
            achievements.on_highscore(self.highscore_key, hs)
        except Exception:
            pass

    def on_exit(self):
        """Spiel wird verlassen: laufende Partie für "Fortsetzen" sichern."""
        self._store_game()
        self._persist()
        self._dirty = False

    # ===================================================== Partie starten
    def _start(self, resume=False):
        key = self._key()
        sv = self.data["saves"].get(key) if resume else None
        if sv is None and not self.game_over:
            # Eine laufende (oder in den Setup geparkte) zählende Partie wird
            # verworfen -> ihren Punktestand nicht verlieren. Nach Game Over
            # hat main.py ihn schon gesichert.
            self._bank_highscore()
        self._blank_game()
        self._layout()
        self.new_round_result()
        self.game_over = False
        self.state = PLAY
        if sv is not None:
            self.grid = [row[:] for row in sv["grid"]]
            for name in ("points", "moves", "undo_used", "assisted", "reached",
                         "continued", "peak", "time_left", "clock_on", "elapsed"):
                setattr(self, name, sv[name])
            self.stack = [dict(e) for e in sv["stack"]]
            if self.undo_rule == "off":
                self.stack = []
            # Beim Fortsetzen wachsen alle Kacheln einmal hinein.
            self.spawns = {(r, c): -0.02 * (r + c) for r in range(self.size)
                           for c in range(self.size) if self.grid[r][c]}
        else:
            if self.data["saves"].pop(key, None) is not None:
                self._persist()
            for _ in range(2):
                sp = spawn_tile(self.grid, self.rng)
                if sp:
                    self.spawns[(sp[0], sp[1])] = 0.0
        self.max_tile = max_tile(self.grid)
        self.peak = max(self.peak, self.max_tile)
        self._best_start = {"best": self.data["best"].get(key, 0),
                            "best_assisted": self.data["best_assisted"].get(key, 0)}
        self._sync_score()
        if self.reached and self.gmode == "classic" and not self.continued:
            self.state = WIN
            self._panel_ticks = pygame.time.get_ticks()
        elif self.gmode == "time" and not self.clock_on:
            self._message(t("g2048.clock_hint"))
        self.play_sound("click")

    def _to_setup(self):
        self._store_game()
        self._persist()
        self._dirty = False
        self.queue.clear()
        self.slide = None
        self.game_over = False
        self.setup_act = 0
        self.state = SETUP
        self.play_sound("click")

    # ===================================================== Layout (Spiel)
    def _layout(self):
        """Brett links, Info-Spalte rechts - aus width/height abgeleitet."""
        w, h, n = self.width, self.height, self.size
        m = max(10, h // 40)
        side_w = int(max(150, min(340, w * 0.27)))
        bar = max(6, h // 64) if self.gmode == "time" else 0
        bar_gap = max(5, h // 90) if bar else 0
        area_w = w - side_w - 3 * m
        area_h = h - 2 * m - bar - bar_gap
        board = max(60, min(area_w, area_h))
        gap = max(3, int(board * (0.13 if n <= 4 else 0.10) / (n + 1)))
        cell = max(8, (board - gap * (n + 1)) // n)
        board = cell * n + gap * (n + 1)
        bx = m + (area_w - board) // 2
        by = m + (area_h - board) // 2
        self.gap, self.cell = gap, cell
        self.board_rect = pygame.Rect(bx, by, board, board)
        self.time_rect = pygame.Rect(bx, by + board + bar_gap, board, bar)
        self.side_rect = pygame.Rect(w - m - side_w, by, side_w,
                                     board + bar + bar_gap)
        self._tile_cache.clear()
        self._layout_side()

    def _layout_side(self):
        sd = self.side_rect
        h = self.height
        pad = max(6, h // 64)
        gap = max(5, h // 90)
        self._pad = pad
        lh = self._f_label.get_height()
        y = sd.y
        self.tag_rect = pygame.Rect(sd.x, y, sd.w, lh)
        y += lh + gap
        ch = lh + self._f_big.get_height() + 2 * pad + 2
        self.score_rect = pygame.Rect(sd.x, y, sd.w, ch)
        y += ch + gap
        ch = lh + self._f_mid.get_height() + 2 * pad + 2
        self.best_rect = pygame.Rect(sd.x, y, sd.w, ch)
        y += ch + gap
        self.row_h = self._f_row.get_height() + max(4, h // 90)
        self.info_rect = pygame.Rect(sd.x, y, sd.w, 4 * self.row_h + 2 * pad)
        y = self.info_rect.bottom + gap
        # Knöpfe: gestapelt mit Beschriftung, wenn Platz ist - sonst Symbolreihe.
        bh = max(26, h // 17)
        stacked_h = 3 * bh + 2 * gap
        if sd.bottom - y >= stacked_h:
            top = sd.bottom - stacked_h
            self.btn_rects = [pygame.Rect(sd.x, top + i * (bh + gap), sd.w, bh)
                              for i in range(3)]
            self.btn_stacked = True
        else:
            bw = (sd.w - 2 * gap) // 3
            self.btn_rects = [pygame.Rect(sd.x + i * (bw + gap), sd.bottom - bh,
                                          bw, bh) for i in range(3)]
            self.btn_stacked = False
        self.hint_area = pygame.Rect(sd.x, y, sd.w,
                                     max(0, self.btn_rects[0].top - gap - y))

    def _cell_pos(self, r, c):
        ox, oy = getattr(self, "_off", (0, 0))
        br = self.board_rect
        return (br.x + ox + self.gap + c * (self.cell + self.gap),
                br.y + oy + self.gap + r * (self.cell + self.gap))

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        w, h = self.width, self.height
        cx = w // 2
        th = self._f_setup_title.get_height()
        sh = self._small.get_height()
        lh = self._tiny.get_height()
        self.title_y = max(30, int(h * 0.085))
        bottom = h - 36 - max(4, h // 90)
        g = max(5, h // 72)
        bw = min(620, w - 40)
        info_pad = max(5, h // 90)
        act_min = sh + lh + 18          # Platz für Unterzeile + v4.2-Linie

        # Erst mit Untertitel versuchen; wird es zu eng, fällt er weg.
        for with_sub in (True, False):
            if with_sub:
                top = self.title_y + max(th // 2 + 36, 42) + sh // 2 + max(10, h // 40)
            else:
                top = self.title_y + th // 2 + 14 + max(6, h // 60)
            info_h = 3 * lh + 2 * info_pad + 4
            avail = bottom - top
            fixed = 3 * (lh + 3) + 4 * g + info_h
            row_h = int((avail - fixed - act_min) / 3)
            row_h = max(22, min(row_h, max(36, h // 15)))
            act_h = max(act_min, row_h + 6)
            used = fixed + 3 * row_h + act_h
            if used <= avail:
                break
        self.setup_sub = with_sub
        # Freien Platz zur Hälfte in die Abstände zwischen den Gruppen geben,
        # den Rest als Rand oben/unten - wirkt auf großen Auflösungen luftiger.
        extra = max(0, avail - used)
        vg = g + min(extra // 8, h // 28)
        used += 4 * (vg - g)
        y = top + max(0, (avail - used) // 2)

        def row(yy, n):
            cw = (bw - g * (n - 1)) / n
            return [pygame.Rect(int(cx - bw / 2 + i * (cw + g)), yy, int(cw), row_h)
                    for i in range(n)]

        self.size_rects = row(y + lh + 3, len(SIZES))
        y += lh + 3 + row_h + vg
        self.mode_rects = row(y + lh + 3, len(MODE_KEYS))
        y += lh + 3 + row_h + vg
        self.undo_rects = row(y + lh + 3, len(UNDO_KEYS))
        y += lh + 3 + row_h + vg
        self.info_rect_setup = pygame.Rect(cx - bw // 2, y, bw, info_h)
        self._info_pad = info_pad
        y += info_h + vg
        half = (bw - g) // 2
        self.cont_rect = pygame.Rect(cx - bw // 2, y, half, act_h)
        self.new_rect = pygame.Rect(cx - bw // 2 + half + g, y, half, act_h)
        sw = min(bw, max(200, w // 3))
        self.start_rect = pygame.Rect(cx - sw // 2, y, sw, act_h)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault(SECTION, {})[key] = value
            settings_mod.save_settings(self.settings)

    def _saved(self):
        return self.data["saves"].get(self._key())

    def _set_size(self, n):
        if n == self.size:
            return
        self.size = n
        self._save_setting("size", n)
        self.setup_act = 0
        self._layout()
        self.play_sound("click")

    def _set_mode(self, m):
        if m == self.gmode:
            return
        self.gmode = m
        self._save_setting("mode", m)
        self.setup_act = 0
        self._layout()
        self.play_sound("click")

    def _set_undo(self, u):
        if u == self.undo_rule:
            return
        self.undo_rule = u
        self._save_setting("undo", u)
        self.play_sound("select")

    def _setup_step(self, step):
        if self.setup_focus == 0:
            i = (SIZES.index(self.size) + step) % len(SIZES)
            self._set_size(SIZES[i])
        elif self.setup_focus == 1:
            i = (MODE_KEYS.index(self.gmode) + step) % len(MODE_KEYS)
            self._set_mode(MODE_KEYS[i])
        elif self.setup_focus == 2:
            i = (UNDO_KEYS.index(self.undo_rule) + step) % len(UNDO_KEYS)
            self._set_undo(UNDO_KEYS[i])
        elif self._saved() is not None:
            self.setup_act = 1 - self.setup_act
            self.play_sound("move")

    def _setup_activate(self):
        self._start(resume=(self._saved() is not None and self.setup_act == 0))

    def _handle_setup(self, event):
        if event.kind == InputEvent.MOUSEMOVE:
            self.hover = event.pos
            return
        if event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            p = event.pos
            for rects, focus, keys, setter in (
                    (self.size_rects, 0, SIZES, self._set_size),
                    (self.mode_rects, 1, MODE_KEYS, self._set_mode),
                    (self.undo_rects, 2, UNDO_KEYS, self._set_undo)):
                for i, rc in enumerate(rects):
                    if rc.collidepoint(p):
                        self.setup_focus = focus
                        setter(keys[i])
                        return
            if self._saved() is not None:
                if self.cont_rect.collidepoint(p):
                    self.setup_focus, self.setup_act = 3, 0
                    self._start(resume=True)
                elif self.new_rect.collidepoint(p):
                    self.setup_focus, self.setup_act = 3, 1
                    self._start(resume=False)
            elif self.start_rect.collidepoint(p):
                self._start(resume=False)
            return
        if event.kind != InputEvent.KEYDOWN:
            return
        k = event.key
        if k in ("3", "4", "5", "6", "7", "8"):
            self.setup_focus = 0
            self._set_size(int(k))
        elif k in ("Up", "w", "W"):
            self.setup_focus = (self.setup_focus - 1) % 4
            self.play_sound("move")
        elif k in ("Down", "s", "S"):
            self.setup_focus = (self.setup_focus + 1) % 4
            self.play_sound("move")
        elif k in ("Left", "a", "A"):
            self._setup_step(-1)
        elif k in ("Right", "d", "D"):
            self._setup_step(1)
        elif k in ("m", "M"):
            self.setup_focus = 1
            self._setup_step(1)
        elif k in ("u", "U"):
            self.setup_focus = 2
            self._setup_step(1)
        elif k in ("Return", "space", "KP_Enter"):
            self._setup_activate()

    # ===================================================== Eingabe (Spiel)
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        kind = event.kind
        if kind == InputEvent.MOUSEMOVE:
            self.hover = event.pos
            return
        if kind == InputEvent.MOUSEDOWN:
            if event.button != 1:
                return
            self.drag = None
            if self.state == WIN or self.game_over:
                self._panel_click(event.pos)
                return
            for rc, act in zip(self.btn_rects, ("undo", "new", "setup")):
                if rc.collidepoint(event.pos):
                    self._button(act)
                    return
            self.drag = event.pos
            return
        if kind == InputEvent.MOUSEUP:
            start, self.drag = self.drag, None
            if start is None or event.button != 1 or event.pos is None:
                return
            d = self._swipe_dir(start, event.pos)
            if d:
                self._input(d)
            return
        if kind != InputEvent.KEYDOWN or event.repeat:
            return

        key = event.key
        undo_key = key in ("u", "U", "BackSpace") and self._free(key)
        new_key = key in ("r", "R", "n", "N") and self._free(key)
        setup_key = key == "Tab" or (key in ("s", "S") and self._free(key))

        if self.game_over:
            if key in ("Return", "space", "KP_Enter") or new_key:
                self._start(resume=False)
            elif undo_key:
                self._input("undo")
            elif setup_key:
                self._to_setup()
            return
        if self.state == WIN:
            if key in ("Return", "space", "KP_Enter"):
                self._panel_action("continue")
            elif new_key:
                self._start(resume=False)
            elif undo_key:
                self._input("undo")
            elif setup_key:
                self._to_setup()
            return

        for d, (arrow, action, _vec) in DIRS.items():
            if key == arrow or self.is_action(key, action):
                self._input(d)
                return
        if undo_key:
            self._input("undo")
        elif new_key:
            self._start(resume=False)
        elif setup_key:
            self._to_setup()

    def _swipe_dir(self, a, b):
        """Wisch-Richtung aus Start/Ende (None bei zu kurzem/diagonalem Wisch)."""
        dx, dy = b[0] - a[0], b[1] - a[1]
        need = max(18, self.cell * 0.3)
        if max(abs(dx), abs(dy)) < need:
            return None
        if abs(dx) > abs(dy) * 1.2:
            return "R" if dx > 0 else "L"
        if abs(dy) > abs(dx) * 1.2:
            return "D" if dy > 0 else "U"
        return None

    def _button(self, act):
        if act == "undo":
            self._input("undo")
        elif act == "new":
            self._start(resume=False)
        else:
            self._to_setup()

    def _input(self, what):
        """Zug ("L"/"R"/"U"/"D") oder "undo" - während Animationen gepuffert."""
        if what == "undo":
            if self.slide is not None:
                if len(self.queue) < QUEUE_MAX:
                    self.queue.append(what)
                return
            self._undo()
            return
        if self.state != PLAY or self.game_over or self.over_t is not None:
            return
        if self.slide is not None:
            if len(self.queue) < QUEUE_MAX:
                self.queue.append(what)
            return
        self._do_move(what)

    # ===================================================== Züge
    def _do_move(self, d):
        plan = plan_move(self.grid, d)
        if not plan["moved"]:
            vec = DIRS[d][2]
            self.nudge = [vec[0], vec[1], 0.0]
            return False
        entry = dict(grid=[row[:] for row in self.grid], points=self.points,
                     moves=self.moves, reached=self.reached,
                     slides=plan["slides"], spawn=None)
        self.grid = plan["grid"]
        self.points += plan["gained"]
        self.moves += 1
        spawn = spawn_tile(self.grid, self.rng)
        entry["spawn"] = spawn
        if self.undo_rule != "off":
            self.stack.append(entry)
            cap = STACK_MAX if self.undo_rule == "unlimited" else UNDO_LIMIT
            if len(self.stack) > cap:
                del self.stack[0]
        if self.gmode == "time":
            self.clock_on = True
        self.pops.clear()
        self.spawns.clear()
        self.slide = dict(t=0.0, dur=SLIDE_FAST if self.queue else SLIDE_T,
                          slides=plan["slides"], merges=plan["merges"],
                          spawn=spawn, vanish=None, undo=False, flash=None)
        self._after_move(plan)
        return True

    def _after_move(self, plan):
        if plan["gained"]:
            self.popups.append([plan["gained"], 0.0])
            del self.popups[:-3]
        top = max((v for _r, _c, v in plan["merges"]), default=0)
        if top:
            self.play_sound("merge")
            if top >= 128:
                self._tone(top)
        else:
            self.play_sound("move")

        self.max_tile = max_tile(self.grid)
        new_peak = self.max_tile > self.peak
        self.peak = max(self.peak, self.max_tile)

        if not self.reached and self.max_tile >= WIN_TILE:
            self.reached = True
            if not self.assisted:
                if self.gmode != "time":
                    self.report_result(True)
                if self.size <= ACH_MAX_SIZE:
                    self.ach_event("tile_2048")
            if self.gmode == "classic" and not self.continued:
                self.win_pending = True
        if (self.max_tile >= 4096 and not self.assisted
                and self.size <= ACH_MAX_SIZE):
            self.ach_event("tile_4096")
        if new_peak and self.max_tile >= WIN_TILE and not self.win_pending:
            self.slide["flash"] = self.max_tile

        self._note_best()
        self._sync_score()
        self._dirty = True
        if not has_moves(self.grid):
            self.over_t = OVER_DELAY
            self.queue.clear()

    def _undo(self):
        if self.undo_rule == "off":
            self._message(t("g2048.undo_disabled"))
            self.play_sound("hit")
            return False
        if not self.stack:
            self.play_sound("hit")
            return False
        if self.undo_rule == "limited" and self._undo_left() <= 0:
            self._message(t("g2048.undo_empty"))
            self.play_sound("hit")
            return False
        if self.game_over and self.over_reason == "time":
            return False
        e = self.stack.pop()
        self.undo_used += 1
        self.assisted = True
        self.grid = [row[:] for row in e["grid"]]
        self.points = e["points"]
        self.moves = e["moves"]
        self.reached = e["reached"]
        self.max_tile = max_tile(self.grid)
        if self.game_over:
            self.resume_from_game_over()      # dieselbe Partie, kein Neustart
        self.over_t = None
        self.win_pending = False
        self.state = PLAY
        self.pops.clear()
        self.spawns.clear()
        if e["slides"]:
            rev = [(tr, tc, fr, fc, v) for (fr, fc, tr, tc, v) in e["slides"]]
            self.slide = dict(t=0.0, dur=SLIDE_T, slides=rev, merges=[],
                              spawn=None, vanish=e["spawn"], undo=True, flash=None)
        else:
            self.spawns = {(r, c): 0.0 for r in range(self.size)
                           for c in range(self.size) if self.grid[r][c]}
        self._note_best()
        self._sync_score()
        self._dirty = True
        self.play_sound("select")
        return True

    def _end_slide(self):
        sl = self.slide
        self.slide = None
        if not sl["undo"]:
            for r, c, v in sl["merges"]:
                self.pops[(r, c)] = 0.0
                if v >= 128:
                    x, y = self._cell_pos(r, c)
                    cx, cy = x + self.cell / 2, y + self.cell / 2
                    n = min(22, 4 + int(math.log2(v)))
                    self._burst(cx, cy, tile_color(v), n, self.cell * 2.6)
                    if v >= WIN_TILE:
                        self.rings.append([cx, cy, 0.0, tile_color(v)])
            if sl["spawn"]:
                self.spawns[(sl["spawn"][0], sl["spawn"][1])] = 0.0
            if sl["flash"]:
                v = sl["flash"]
                self.flash = [str(v), 0.0, tile_color(v)]
                self.play_sound("powerup")
        if self.win_pending:
            self.win_pending = False
            self.state = WIN
            self.queue.clear()
            self._panel_ticks = pygame.time.get_ticks()
            br = self.board_rect
            for _ in range(3):
                self._burst(br.centerx + random.uniform(-br.w * 0.3, br.w * 0.3),
                            br.centery + random.uniform(-br.h * 0.3, br.h * 0.3),
                            tile_color(WIN_TILE), 18, self.cell * 3.2)
            self.play_sound("win")
            self.rumble(180)

    def _finish(self, timeout=False):
        """Partie vorbei (keine Züge mehr oder Zeit abgelaufen)."""
        self.over_t = None
        self.queue.clear()
        self.slide = None
        self.pops.clear()
        self.spawns.clear()
        self.over_reason = "time" if timeout else "stuck"
        self.state = PLAY
        self.win_pending = False
        self._note_best()
        book = "best_assisted" if self.assisted else "best"
        self.new_best = self.points > max(0, self._best_start.get(book, 0))
        if not self.assisted and self.gmode == "classic" and not self.reached:
            self.report_result(False)
        self.data["saves"].pop(self._key(), None)
        self._persist()
        self._dirty = False
        self._panel_ticks = pygame.time.get_ticks()
        self._sync_score()
        self.game_over = True
        self.play_sound("level" if timeout else "gameover")
        self.rumble(220)

    def _panel_action(self, act):
        if act == "continue":
            self.continued = True
            self.state = PLAY
            self._dirty = True
            self.play_sound("click")
        elif act == "new":
            self._start(resume=False)
        elif act == "undo":
            self._input("undo")
        elif act == "setup":
            self._to_setup()

    def _message(self, text):
        self.msg = [text, 0.0]

    def _burst(self, x, y, color, n, speed):
        light = ui.mix(color, (255, 255, 255), 0.35)
        size = max(1.5, self.cell / 34)
        for _ in range(n):
            if len(self.particles) >= MAX_PARTICLES:
                break
            a = random.uniform(0, math.tau)
            sp = speed * random.uniform(0.35, 1.0)
            self.particles.append([x, y, math.cos(a) * sp, math.sin(a) * sp, 0.0,
                                   random.uniform(0.35, 0.7),
                                   light if random.random() < 0.5 else color,
                                   size * random.uniform(0.7, 1.6)])

    # ===================================================== Update
    def update(self, dt):
        self.anim_t += dt
        if self.state == SETUP:
            return

        # Uhr (Zeitangriff) und Spielzeit
        if self.state == PLAY and self.over_t is None:
            if self.moves > 0:
                self.elapsed += dt
            if self.gmode == "time" and self.clock_on:
                self.time_left -= dt
                if self.time_left <= 0:
                    self.time_left = 0.0
                    self._finish(timeout=True)
                    return

        if self.slide is not None:
            self.slide["t"] += dt
            if self.slide["t"] >= self.slide["dur"]:
                self._end_slide()
        # Gepufferte Eingaben abarbeiten (ungültige Züge starten keine Animation)
        while (self.slide is None and self.queue and self.state == PLAY
               and self.over_t is None and not self.game_over):
            nxt = self.queue.pop(0)
            if nxt == "undo":
                self._undo()
            else:
                self._do_move(nxt)

        for d, dur in ((self.pops, POP_T), (self.spawns, SPAWN_T)):
            for k in list(d):
                d[k] += dt
                if d[k] >= dur:
                    del d[k]
        if self.particles:
            grav = self.cell * 3.0
            alive = []
            for p in self.particles:
                p[4] += dt
                if p[4] < p[5]:
                    p[0] += p[2] * dt
                    p[1] += p[3] * dt
                    p[2] *= 0.92
                    p[3] = p[3] * 0.92 + grav * dt
                    alive.append(p)
            self.particles = alive
        self.rings = [rg for rg in self.rings if self._age(rg, 2, dt, 0.55)]
        self.popups = [pp for pp in self.popups if self._age(pp, 1, dt, POPUP_T)]
        if self.flash and not self._age(self.flash, 1, dt, FLASH_T):
            self.flash = None
        if self.msg and not self._age(self.msg, 1, dt, MSG_T):
            self.msg = None
        if self.nudge and not self._age(self.nudge, 2, dt, NUDGE_T):
            self.nudge = None

        # Verzögertes Game Over (wartet auch, solange "Weiterspielen?" offen ist)
        if self.over_t is not None and self.slide is None and self.state == PLAY:
            self.over_t -= dt
            if self.over_t <= 0:
                self._finish()
                return

        # Autosave gedrosselt
        self._save_cd -= dt
        if self._dirty and self._save_cd <= 0:
            self._store_game()
            self._persist()
            self._dirty = False
            self._save_cd = SAVE_EVERY

    @staticmethod
    def _age(item, idx, dt, life):
        item[idx] += dt
        return item[idx] < life

    # ===================================================== Zeichen-Hilfen
    def _txt(self, text, fnt, color):
        key = (text, id(fnt), fnt.get_height(), tuple(color))
        img = self._txt_cache.get(key)
        if img is None:
            if len(self._txt_cache) > 400:
                self._txt_cache.clear()
            img = fnt.render(text, True, color)
            self._txt_cache[key] = img
        return img

    def _fit_font(self, text, px, max_w, bold=False, min_px=8):
        """Größte Schrift <= px, in der 'text' in max_w Pixel passt (gecacht)."""
        key = (text, px, int(max_w), bold)
        f = self._fit_cache.get(key)
        if f is None:
            size = int(px)
            f = ui.font(size, bold=bold)
            while size > min_px and f.size(text)[0] > max_w:
                size -= 1
                f = ui.font(size, bold=bold)
            if len(self._fit_cache) > 500:
                self._fit_cache.clear()
                self._fit_sizes.clear()
            self._fit_cache[key] = f
            self._fit_sizes[key] = size       # für den Layout-Test
        return f

    def _tile_surface(self, v):
        """Kachel samt Glow und Ziffern - je (Wert, Zellgröße) einmal gerendert."""
        key = (v, self.cell)
        surf = self._tile_cache.get(key)
        if surf is not None:
            return surf
        cell = self.cell
        col = tile_color(v)
        rad = max(3, cell // 10)
        glow = v >= 128
        pad = max(2, cell // 7) if glow else 0
        size = cell + 2 * pad
        surf = pygame.Surface((size, size), pygame.SRCALPHA)
        if glow:
            strength = min(1.0, (math.log2(v) - 6) / 6.0)
            g = pygame.Surface((size, size), pygame.SRCALPHA)
            gcol = col if v in TILE_COLORS else COL_TEXT_BEYOND
            pygame.draw.rect(g, (*gcol, int(50 + 130 * strength)),
                             (pad // 2, pad // 2, cell + pad, cell + pad),
                             border_radius=rad + pad // 2)
            small = pygame.transform.smoothscale(g, (max(1, size // 5),
                                                     max(1, size // 5)))
            surf.blit(pygame.transform.smoothscale(small, (size, size)), (0, 0))
        depth = max(1, cell // 28)
        pygame.draw.rect(surf, _shade(col, 0.78), (pad, pad + depth, cell, cell - depth),
                         border_radius=rad)
        pygame.draw.rect(surf, col, (pad, pad, cell, cell - depth), border_radius=rad)
        if v not in TILE_COLORS:
            pygame.draw.rect(surf, COL_TEXT_BEYOND, (pad, pad, cell, cell - depth),
                             max(1, cell // 30), border_radius=rad)
        text = str(v)
        px = max(7, int(cell * DIGIT_SCALE.get(len(text), 0.20)))
        fnt = self._fit_font(text, px, cell * 0.84, bold=True, min_px=6)
        img = fnt.render(text, True, tile_text_color(v))
        surf.blit(img, img.get_rect(center=(pad + cell / 2, pad + (cell - depth) / 2)))
        if len(self._tile_cache) > 120:
            self._tile_cache.clear()
        self._tile_cache[key] = surf
        return surf

    def _blit_tile(self, s, v, x, y, scale=1.0):
        if scale <= 0.03:
            return
        surf = self._tile_surface(v)
        pad = (surf.get_width() - self.cell) // 2
        if abs(scale - 1.0) < 0.01:
            s.blit(surf, (int(x) - pad, int(y) - pad))
            return
        sz = max(1, int(surf.get_width() * scale))
        img = pygame.transform.smoothscale(surf, (sz, sz))
        s.blit(img, (int(x + self.cell / 2 - sz / 2), int(y + self.cell / 2 - sz / 2)))

    def _board_surface(self):
        br = self.board_rect
        style = ui.fx("style")
        key = (br.w, self.size, self.cell, self.gap, style)
        if self._board_key != key:
            pad = 0 if style == "v1" else max(4, br.w // 40)
            surf = pygame.Surface((br.w + 2 * pad, br.h + 2 * pad), pygame.SRCALPHA)
            rad = max(6, br.w // 45)
            if pad:
                sh = pygame.Surface(surf.get_size(), pygame.SRCALPHA)
                pygame.draw.rect(sh, (0, 0, 0, 110), (pad // 2, pad, br.w + pad, br.h + pad // 2),
                                 border_radius=rad + pad // 2)
                sw, shh = sh.get_size()
                small = pygame.transform.smoothscale(sh, (max(1, sw // 6), max(1, shh // 6)))
                surf.blit(pygame.transform.smoothscale(small, (sw, shh)), (0, 0))
            pygame.draw.rect(surf, COL_BOARD, (pad, pad, br.w, br.h), border_radius=rad)
            crad = max(3, self.cell // 10)
            for r in range(self.size):
                for c in range(self.size):
                    pygame.draw.rect(surf, COL_EMPTY,
                                     (pad + self.gap + c * (self.cell + self.gap),
                                      pad + self.gap + r * (self.cell + self.gap),
                                      self.cell, self.cell), border_radius=crad)
            self._board_cache = (surf, pad)
            self._board_key = key
        return self._board_cache

    def _dim_surface(self, size):
        if self._dim_key != size:
            self._dim_key = size
            self._dim = pygame.Surface(size, pygame.SRCALPHA)
            pygame.draw.rect(self._dim, (12, 14, 22, 165), (0, 0, *size),
                             border_radius=max(6, size[0] // 45))
        return self._dim

    def _icon(self, s, kind, cx, cy, size, color):
        """Knopf-Symbole aus Grundformen (Pfeil zurück, Plus, Regler)."""
        w = max(2, size // 8)
        if kind == "undo":
            r = size * 0.36
            rect = pygame.Rect(0, 0, int(2 * r), int(2 * r))
            rect.center = (cx, int(cy + size * 0.06))
            pygame.draw.arc(s, color, rect, -math.pi / 3, math.pi, w)
            a = size * 0.2
            tx, ty = rect.left + w / 2, rect.centery
            pygame.draw.polygon(s, color, [(tx - a, ty - a * 0.2), (tx + a, ty - a * 0.2),
                                           (tx, ty + a)])
        elif kind == "new":
            ln = int(size * 0.34)
            pygame.draw.rect(s, color, (cx - ln, cy - w // 2, 2 * ln, w), border_radius=w // 2)
            pygame.draw.rect(s, color, (cx - w // 2, cy - ln, w, 2 * ln), border_radius=w // 2)
        else:
            ln = int(size * 0.36)
            for i, knob in ((-1, 0.35), (0, -0.3), (1, 0.1)):
                yy = cy + i * int(size * 0.26)
                pygame.draw.line(s, color, (cx - ln, yy), (cx + ln, yy), max(1, w // 2))
                pygame.draw.circle(s, color, (int(cx + knob * ln), yy), max(2, w))

    def _infinity(self, s, right, cy, h, color):
        """Unendlich-Zeichen (die UI-Schrift hat kein ∞); gibt die Breite zurück."""
        r = max(3, int(h * 0.26))
        w = max(1, h // 9)
        pygame.draw.circle(s, color, (right - r, cy), r, w)
        pygame.draw.circle(s, color, (right - 3 * r + w, cy), r, w)
        return 4 * r

    # ===================================================== Zeichnen (Spiel)
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        if self.state == SETUP:
            self._draw_setup(s)
            return
        self._draw_board(s)
        self._draw_side(s)
        if self.gmode == "time":
            self._draw_timebar(s)
        self._draw_effects(s)
        if self.state == WIN:
            self._draw_panel(s, win=True)
        elif self.game_over:
            self._draw_panel(s, win=False)

    def _draw_board(self, s):
        off = (0, 0)
        if self.nudge:
            dx, dy, tt = self.nudge
            amp = max(3.0, self.cell * 0.07) * math.sin(math.pi * min(1.0, tt / NUDGE_T))
            off = (int(dx * amp), int(dy * amp))
        self._off = off
        surf, pad = self._board_surface()
        s.blit(surf, (self.board_rect.x - pad + off[0], self.board_rect.y - pad + off[1]))

        sl = self.slide
        if sl is not None:
            p = ease_out_cubic(min(1.0, sl["t"] / sl["dur"]))
            if sl["vanish"]:
                r, c, v = sl["vanish"]
                x, y = self._cell_pos(r, c)
                self._blit_tile(s, v, x, y, 1.0 - p)
            for fr, fc, tr, tc, v in sl["slides"]:
                x0, y0 = self._cell_pos(fr, fc)
                x1, y1 = self._cell_pos(tr, tc)
                self._blit_tile(s, v, x0 + (x1 - x0) * p, y0 + (y1 - y0) * p)
            self._off = (0, 0)
            return

        popping = []
        n = self.size
        for r in range(n):
            row = self.grid[r]
            for c in range(n):
                v = row[c]
                if not v:
                    continue
                x, y = self._cell_pos(r, c)
                tt = self.spawns.get((r, c))
                if tt is not None:
                    self._blit_tile(s, v, x, y, ease_out_back(max(0.0, min(1.0, tt / SPAWN_T))))
                    continue
                tt = self.pops.get((r, c))
                if tt is not None:
                    popping.append((v, x, y, 1.0 + 0.2 * math.sin(math.pi * min(1.0, tt / POP_T))))
                    continue
                self._blit_tile(s, v, x, y)
        for v, x, y, scale in popping:
            self._blit_tile(s, v, x, y, scale)
        self._off = (0, 0)

    def _card(self, s, rect, label, value, px, color):
        ui.draw_panel(s, rect, radius=max(6, rect.h // 5), shadow=False)
        pad = self._pad
        lf = self._fit_font(label, self._lab_px, rect.w - 2 * pad)
        im = self._txt(label, lf, ui.TEXT_DIM)
        s.blit(im, (rect.x + pad, rect.y + pad))
        lh = self._f_label.get_height()
        vf = self._fit_font(value, px, rect.w - 2 * pad, bold=True)
        im = self._txt(value, vf, color)
        free_h = rect.h - 2 * pad - lh
        s.blit(im, (rect.x + pad, rect.y + pad + lh + max(2, (free_h - im.get_height()) // 2 + 2)))
        return im

    def _best_value(self):
        book = self.data["best_assisted" if self.assisted else "best"]
        return book.get(self._key(), 0)

    def _draw_side(self, s):
        sd = self.side_rect
        tag = "%d×%d · %s" % (self.size, self.size, t("g2048.mode." + self.gmode))
        tag_col = ui.mix(self.accent, ui.TEXT, 0.25)
        if self.assisted:
            # "mit Rückgängig" ausgeschrieben, wenn es passt - sonst nur das
            # Rückgängig-Symbol hinter Größe und Modus.
            full = tag + " · " + t("g2048.assisted")
            if self._f_label.size(full)[0] <= sd.w - 4:
                tag = full
            else:
                lh = self._f_label.get_height()
                f = self._fit_font(tag, self._lab_px, sd.w - lh - 10)
                img = self._txt(tag, f, tag_col)
                s.blit(img, img.get_rect(midleft=(sd.x + 2, self.tag_rect.centery)))
                self._icon(s, "undo", sd.x + 2 + img.get_width() + 4 + lh // 2,
                           self.tag_rect.centery, lh, ui.TEXT_DIM)
                tag = None
        if tag is not None:
            f = self._fit_font(tag, self._lab_px, sd.w - 4)
            img = self._txt(tag, f, tag_col)
            s.blit(img, img.get_rect(midleft=(sd.x + 2, self.tag_rect.centery)))

        self._card(s, self.score_rect, t("g2048.hud.score"), str(self.points),
                   self._big_px, ui.TEXT)
        best_lbl = t("g2048.hud.best")
        self._card(s, self.best_rect, best_lbl, str(max(self._best_value(), 0)),
                   self._mid_px, ui.GOLD)

        # "+32"-Popups steigen rechts im Punktefeld auf
        for gained, tt in self.popups:
            p = tt / POPUP_T
            img = self._txt("+%d" % gained, self._f_row_b, ui.GREEN).copy()
            img.set_alpha(int(255 * (1.0 - p) ** 0.8))
            r = self.score_rect
            y = r.bottom - self._pad - img.get_height() - int(p * r.h * 0.55)
            s.blit(img, (r.right - self._pad - img.get_width(), y))

        # Info-Liste: Züge, größte Kachel, Rückgängig, Zeit
        ir = self.info_rect
        pad = self._pad
        ui.draw_panel(s, ir, radius=max(6, self.row_h // 2), shadow=False)
        rows = [("moves", t("g2048.hud.moves")), ("max", t("g2048.hud.max")),
                ("undo", t("g2048.hud.undo")), ("time", t("g2048.hud.time"))]
        fh = self._f_row.get_height()
        for i, (kind, label) in enumerate(rows):
            cy = ir.y + pad + i * self.row_h + self.row_h // 2
            right = ir.right - pad
            col = ui.TEXT
            if kind == "moves":
                vw = self._blit_value(s, str(self.moves), right, cy, col)
            elif kind == "max":
                vw = self._blit_value(s, str(self.max_tile), right, cy, col)
                sq = max(6, int(fh * 0.62))
                rc = pygame.Rect(right - vw - sq - 5, cy - sq // 2, sq, sq)
                pygame.draw.rect(s, tile_color(max(2, self.max_tile)), rc,
                                 border_radius=max(2, sq // 4))
                if self.max_tile not in TILE_COLORS and self.max_tile:
                    pygame.draw.rect(s, COL_TEXT_BEYOND, rc, 1, border_radius=max(2, sq // 4))
                vw += sq + 5
            elif kind == "undo":
                if self.undo_rule == "off":
                    vw = self._blit_value(s, "–", right, cy, ui.TEXT_FAINT)
                elif self.undo_rule == "limited":
                    left = self._undo_left()
                    vw = self._blit_value(s, "%d/%d" % (left, UNDO_LIMIT), right, cy,
                                          ui.TEXT if left else ui.TEXT_FAINT)
                else:
                    vw = self._infinity(s, right, cy, fh, ui.TEXT)
            else:
                if self.gmode == "time":
                    col = ui.TEXT
                    if self.time_left < 15 and self.clock_on and not self.game_over:
                        col = ui.mix(ui.RED, ui.TEXT, 0.4 * ui.pulse(8.0, 0.0, 1.0))
                    vw = self._blit_value(s, fmt_time(self.time_left), right, cy, col)
                else:
                    vw = self._blit_value(s, fmt_time(self.elapsed), right, cy, ui.TEXT_DIM)
            lf = self._fit_font(label, self._row_px, ir.w - 2 * pad - vw - 6)
            im = self._txt(label, lf, ui.TEXT_DIM)
            s.blit(im, im.get_rect(midleft=(ir.x + pad, cy)))

        # Knöpfe
        specs = (("undo", t("g2048.btn.undo"), "U"), ("new", t("g2048.btn.new"), "R"),
                 ("setup", t("g2048.btn.setup"), "Tab"))
        blocked = self.state == WIN or self.game_over
        for (kind, label, cap), rc in zip(specs, self.btn_rects):
            hov = (not blocked and self.hover is not None and rc.collidepoint(self.hover))
            ui.draw_button(s, rc, "", self._f_row, selected=hov, accent=self.accent)
            enabled = kind != "undo" or self._can_undo()
            col = ui.TEXT if enabled else ui.TEXT_FAINT
            isz = max(12, int(rc.h * 0.56))
            if self.btn_stacked:
                ix = rc.x + pad + isz // 2 + 2
                self._icon(s, kind, ix, rc.centery, isz, col)
                cap_img = self._txt(cap, self._f_label, ui.TEXT_FAINT)
                cap_w = cap_img.get_width() + 8
                avail = rc.w - (ix + isz // 2 + 8 - rc.x) - pad - cap_w
                if avail > 20:
                    lf = self._fit_font(label, self._row_px, avail)
                    im = self._txt(label, lf, col)
                    s.blit(im, im.get_rect(midleft=(ix + isz // 2 + 8, rc.centery)))
                    s.blit(cap_img, cap_img.get_rect(midright=(rc.right - pad, rc.centery)))
            else:
                self._icon(s, kind, rc.centerx, rc.centery, isz, col)

        # Tastenhinweise im freien Platz dazwischen (nur wenn komplett passend)
        area = self.hint_area
        if area.h > 0:
            parts = [p.strip() for p in t("g2048.play_hint").split("·")]
            lines, cur = [], ""
            fnt = self._f_label
            for part in parts:
                trial = part if not cur else cur + " · " + part
                if fnt.size(trial)[0] <= area.w:
                    cur = trial
                else:
                    if cur:
                        lines.append(cur)
                    cur = part
            if cur:
                lines.append(cur)
            lh = fnt.get_height() + 2
            if (lines and len(lines) * lh <= area.h
                    and all(fnt.size(ln)[0] <= area.w for ln in lines)):
                y = area.centery - len(lines) * lh // 2
                for ln in lines:
                    im = self._txt(ln, fnt, ui.TEXT_FAINT)
                    s.blit(im, im.get_rect(midtop=(area.centerx, y)))
                    y += lh

    def _blit_value(self, s, text, right, cy, color):
        im = self._txt(text, self._f_row_b, color)
        s.blit(im, im.get_rect(midright=(right, cy)))
        return im.get_width()

    def _draw_timebar(self, s):
        r = self.time_rect
        if r.h <= 0:
            return
        rad = r.h // 2
        pygame.draw.rect(s, COL_EMPTY, r, border_radius=rad)
        frac = max(0.0, min(1.0, self.time_left / TIME_LIMIT))
        col = ui.GREEN if frac > 0.5 else (ui.GOLD if frac > 0.2 else ui.RED)
        if self.time_left < 15 and self.clock_on and not self.game_over:
            col = ui.mix(ui.RED, (255, 255, 255), 0.35 * ui.pulse(8.0, 0.0, 1.0))
        fw = int(r.w * frac)
        if fw > 0:
            pygame.draw.rect(s, col, (r.x, r.y, max(fw, r.h), r.h), border_radius=rad)

    def _draw_effects(self, s):
        for x, y, _vx, _vy, age, life, col, size in self.particles:
            k = 1.0 - age / life
            pygame.draw.circle(s, col, (int(x), int(y)), max(1, int(size * (0.4 + 0.6 * k))))
        br = self.board_rect
        if self.rings:
            # Druckwelle ab 2048 - bleibt auf dem Brett
            s.set_clip(br)
            for cx, cy, tt, col in self.rings:
                p = tt / 0.55
                rad = int(self.cell * (0.55 + 0.45 * ease_out_cubic(p)))
                wid = max(1, int(max(2, self.cell // 14) * (1.0 - p)) + 1)
                pygame.draw.circle(s, ui.mix(ui.mix(col, (255, 255, 255), 0.3), COL_BOARD, p),
                                   (int(cx), int(cy)), rad, wid)
            s.set_clip(None)
        if self.flash:
            text, tt, col = self.flash
            px = max(30, int(br.w * (0.2 if len(text) <= 4 else 0.15)))
            fnt = ui.font(px, bold=True)
            img = self._txt(text, fnt, col)
            shadow = self._txt(text, fnt, (0, 0, 0))
            scale = ease_out_back(min(1.0, tt / 0.22))
            alpha = 255 if tt < FLASH_T - 0.4 else int(255 * max(0.0, (FLASH_T - tt) / 0.4))
            rise = int(max(0.0, tt - (FLASH_T - 0.4)) * br.h * 0.15)
            for src, dx, dy, a in ((shadow, 3, 4, alpha * 0.55), (img, 0, 0, alpha)):
                sz = (max(1, int(src.get_width() * scale)), max(1, int(src.get_height() * scale)))
                im = pygame.transform.smoothscale(src, sz) if scale != 1.0 else src.copy()
                im.set_alpha(int(a))
                s.blit(im, im.get_rect(center=(br.centerx + dx, br.centery + dy - rise)))
        if self.msg:
            text, tt = self.msg
            fnt = self._fit_font(text, self._row_px, br.w - 40)
            im = self._txt(text, fnt, ui.TEXT)
            pill = im.get_rect(midtop=(br.centerx, br.y + max(10, br.h // 14))).inflate(28, 12)
            drop = int(min(1.0, tt / 0.18) * 6)
            pill.y += drop - 6
            ui.draw_panel(s, pill, radius=pill.h // 2, shadow=False)
            s.blit(im, im.get_rect(center=pill.center))

    # ----- Sieg-/Endstand-Panel -------------------------------------------
    def _panel_geom(self, win):
        br = self.board_rect
        h = self.height
        pad = max(10, h // 42)
        gap = max(4, h // 110)
        bh = max(26, h // 17)
        rowh = self._f_row.get_height()
        if win:
            heights = [self._f_title.get_height(), rowh, rowh]
            acts = ["continue", "new"]
        else:
            heights = [self._f_title.get_height(), self._f_big.get_height(), rowh, rowh]
            acts = ["new"] + (["undo"] if self._can_undo() else []) + ["setup"]
        ph = (2 * pad + sum(heights) + gap * len(heights) + bh + gap
              + self._f_label.get_height())
        pw = min(br.w - 2 * max(6, br.w // 30), max(int(br.w * 0.86), 200))
        rect = pygame.Rect(0, 0, pw, ph)
        rect.center = br.center
        k = len(acts)
        bw = (pw - 2 * pad - (k - 1) * gap) // k
        by = rect.bottom - pad - self._f_label.get_height() - gap - bh
        btns = [(pygame.Rect(rect.x + pad + i * (bw + gap), by, bw, bh), a)
                for i, a in enumerate(acts)]
        return rect, heights, btns, pad, gap

    def _panel_click(self, pos):
        _rect, _h, btns, _p, _g = self._panel_geom(self.state == WIN)
        for rc, act in btns:
            if rc.collidepoint(pos):
                self._panel_action(act)
                return True
        return False

    def _draw_panel(self, s, win):
        br = self.board_rect
        since = (pygame.time.get_ticks() - self._panel_ticks) / 1000.0
        k = ease_out_cubic(max(0.0, min(1.0, since / 0.3)))
        dim = self._dim_surface(br.size)
        dim.set_alpha(int(255 * k))
        s.blit(dim, br.topleft)
        rect, heights, btns, pad, gap = self._panel_geom(win)
        lift = int((1.0 - k) * 18)
        rect = rect.move(0, lift)
        frame = ui.GOLD if win or self.over_reason == "time" else ui.RED
        radius = max(10, self.height // 40)
        ui.draw_panel(s, rect, radius=radius, accent_top=self.accent)
        pygame.draw.rect(s, frame, rect, 2, border_radius=radius)
        inner = rect.w - 2 * pad
        y = rect.y + pad
        key = self._key()
        if win:
            lines = [(t("g2048.reached"), self._f_title, True, ui.GOLD),
                     (t("g2048.keep_going"), self._f_row, False, ui.TEXT),
                     (t("g2048.stats", score=self.points, moves=self.moves),
                      self._f_row, False, ui.TEXT_DIM)]
        else:
            title = t("g2048.time_up") if self.over_reason == "time" else t("common.game_over")
            if self.new_best:
                best_line = (t("g2048.new_best"), self._f_row, True,
                             ui.mix(ui.GOLD, (255, 255, 255), 0.35 * ui.pulse(4.0, 0.0, 1.0)))
            else:
                book = self.data["best_assisted" if self.assisted else "best"]
                best_line = (t("g2048.best_is", score=book.get(key, 0)), self._f_row,
                             False, ui.TEXT_DIM)
            lines = [(title, self._f_title, True, frame),
                     (str(self.points), self._f_big, True, ui.GOLD if self.new_best else ui.TEXT),
                     best_line,
                     (t("g2048.over_stats", moves=self.moves, tile=self.max_tile),
                      self._f_row, False, ui.TEXT_DIM)]
        for (text, fnt, bold, col), hh in zip(lines, heights):
            f = self._fit_font(text, self._font_px(fnt), inner, bold=bold)
            im = self._txt(text, f, col)
            s.blit(im, im.get_rect(center=(rect.centerx, y + hh // 2)))
            y += hh + gap
        # Drei Knöpfe nebeneinander -> kurzes "Neu", sonst "Neue Partie".
        labels = {"continue": t("g2048.btn.continue"),
                  "new": t("g2048.btn.new" if len(btns) > 2 else "g2048.btn.new_game"),
                  "undo": t("g2048.btn.undo"), "setup": t("g2048.btn.setup")}
        for i, (rc, act) in enumerate(btns):
            rc = rc.move(0, lift)
            hov = self.hover is not None and rc.collidepoint(self.hover)
            label = labels[act]
            f = self._fit_font(label, self._row_px, rc.w - 12)
            ui.draw_button(s, rc, label, f, selected=(i == 0 or hov), accent=self.accent)
        hint = t("g2048.win_hint") if win else t("g2048.over_hint")
        f = self._fit_font(hint, self._lab_px, inner)
        col = ui.mix(ui.TEXT_FAINT, ui.TEXT_DIM, ui.pulse(2.0, 0.0, 1.0))
        im = self._txt(hint, f, col)
        s.blit(im, im.get_rect(midbottom=(rect.centerx, rect.bottom - pad + 2)))

    def _font_px(self, fnt):
        """Pixelgröße einer der eigenen Schriften (für _fit_font)."""
        if fnt is self._f_title:
            return max(24, self.height // 13)
        if fnt is self._f_big:
            return self._big_px
        if fnt is self._f_mid:
            return self._mid_px
        return self._row_px

    # ----- Setup zeichnen -----------------------------------------------
    def _draw_setup(self, s):
        w, h = self.width, self.height
        cx = w // 2
        ui.draw_title(s, w, "2048", subtitle=t("g2048.subtitle") if self.setup_sub else None,
                      y=self.title_y, big=self._f_setup_title, small=self._small,
                      accent=self.accent)
        lh_font = self._tiny
        groups = ((self.size_rects, "g2048.lbl_size",
                   ["%d×%d" % (n, n) for n in SIZES], [n == self.size for n in SIZES]),
                  (self.mode_rects, "g2048.lbl_mode",
                   [t("g2048.mode." + m) for m in MODE_KEYS], [m == self.gmode for m in MODE_KEYS]),
                  (self.undo_rects, "g2048.lbl_undo",
                   [t("g2048.undo." + u) for u in UNDO_KEYS],
                   [u == self.undo_rule for u in UNDO_KEYS]))
        btn_px = max(12, min(self.height // 30, int(self.size_rects[0].h * 0.5)))
        for gi, (rects, lbl, labels, sel) in enumerate(groups):
            col = self.accent if self.setup_focus == gi else ui.TEXT_DIM
            im = self._txt(t(lbl), lh_font, col)
            s.blit(im, im.get_rect(midbottom=(cx, rects[0].top - 3)))
            for rc, label, on in zip(rects, labels, sel):
                f = self._fit_font(label, btn_px, rc.w - 14)
                ui.draw_button(s, rc, label, f, selected=on, accent=self.accent)

        # Info: Modus-Beschreibung, Bestwerte, Highscore-Hinweis
        ir = self.info_rect_setup
        ui.draw_panel(s, ir, radius=max(8, ir.h // 5), shadow=False)
        key = self._key()
        best = self.data["best"].get(key, 0)
        best_a = self.data["best_assisted"].get(key, 0)
        tile = self.data["best_tile"].get(key, 0)
        if best or best_a or tile:
            line2 = t("g2048.best_line", score=best, tile=tile)
            if best_a > best:
                line2 += "  ·  " + t("g2048.best_undo", score=best_a)
            col2 = ui.GOLD
        else:
            line2, col2 = t("g2048.no_best"), ui.TEXT_FAINT
        if self.size == HS_SIZE and self.gmode == HS_MODE:
            line3, col3 = t("g2048.hs_yes"), ui.mix(ui.GREEN, ui.TEXT, 0.25)
        else:
            line3, col3 = t("g2048.hs_no"), ui.TEXT_FAINT
        lines = ((t("g2048.mode_desc." + self.gmode), ui.TEXT_DIM), (line2, col2),
                 (line3, col3))
        lh = self._tiny.get_height()
        y = ir.y + self._info_pad
        for text, col in lines:
            f = self._fit_font(text, max(11, h // 40), ir.w - 2 * self._info_pad)
            im = self._txt(text, f, col)
            s.blit(im, im.get_rect(center=(cx, y + lh // 2)))
            y += lh + 2

        sv = self._saved()
        act_px = max(13, h // 30)
        if sv is not None:
            sub = t("g2048.stats", score=sv["points"], moves=sv["moves"])
            for i, (rc, label) in enumerate(((self.cont_rect, t("g2048.btn.resume")),
                                              (self.new_rect, t("g2048.btn.new_game")))):
                f = self._fit_font(label, act_px, rc.w - 20, bold=(i == 0))
                sf = self._fit_font(sub, max(11, h // 42), rc.w - 20)
                ui.draw_button(s, rc, label, f, selected=(self.setup_act == i),
                               sub=sub if i == 0 else None, sub_font=sf,
                               accent=ui.GREEN if i == 0 else self.accent)
        else:
            f = self._fit_font(t("common.start"), act_px, self.start_rect.w - 20, bold=True)
            ui.draw_button(s, self.start_rect, t("common.start"), f, selected=True,
                           accent=ui.GREEN)
        hint = t("g2048.setup_hint")
        ui.draw_footer(s, w, h, hint, fnt=self._fit_font(hint, 14, w - 24))
