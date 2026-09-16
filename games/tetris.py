# -*- coding: utf-8 -*-
"""
tetris.py
=========
Tetris nach moderner Guideline - Solo, Versus gegen die KI und zu zweit.

Modi (Knöpfe im Vorbereitungs-Screen):
- Solo: im Setup Marathon (endlos, Level alle 10 Zeilen, zählt für den
  Highscore), Sprint 40 Zeilen (Bestzeit) oder Ultra 2 Minuten (Bestwert).
- Versus KI: Heuristik-KI in drei Stärken, Müllzeilen hin und her.
- 2 Spieler: zwei Felder an einer Tastatur, ebenfalls mit Müllzeilen.
Beide Felder bekommen im Versus dieselbe Steinfolge.

Regeln (tetris_core.py): SRS mit echten Kicks, Hold, 5er-Vorschau, Lock
Delay 0,5 s (max. 15 Resets), T-Spins (voll/Mini), Back-to-Back, Combos,
Perfect Clear, Guideline-Gravitation und -Angriffstabelle.

Steuerung (belegbare Aktionen): links/rechts schieben (eigenes DAS/ARR aus
den Optionen, Tasten-Wiederholungen des Systems werden ignoriert), hoch =
rechts drehen, runter = Soft Drop, Aktion = Hard Drop.
Feste Zusatztasten (nur wenn keiner Aktion zugeordnet):
- Solo/Versus KI: C oder Shift = Halten, Z/Y oder Strg rechts = links drehen,
  X = rechts drehen, R = sofortiger Neustart (nur Sprint/Ultra).
- 2 Spieler: Spieler 1 Q = Halten, E = links drehen;
  Spieler 2 Shift rechts = Halten, Strg rechts = links drehen.
Nach dem Spielende: R = nochmal, S = Setup (kurze Sperre gegen versehentliches
Weiterdrücken - die Hard-Drop-Tasten starten bewusst nichts neu).

Nur Marathon füllt self.score (Highscore). Sprint-Bestzeit, Ultra-Bestwert
und die Bilanz gegen die KI liegen in der mem.json-Section "tetris".
"""

import math
import random
import time

import pygame

import audio
import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent
from i18n import t

from . import tetris_ai as ai
from . import tetris_core as core

COLS, VISIBLE, BUFFER = core.COLS, core.VISIBLE, core.BUFFER

# Identitätsfarben der Steine - bewusst NICHT aus dem UI-Theme.
COLORS = {
    "I": (80, 210, 220), "O": (240, 220, 90), "T": (190, 110, 220),
    "S": (110, 220, 120), "Z": (235, 100, 100), "J": (100, 130, 230),
    "L": (240, 160, 80), "G": (128, 134, 150), "X": (96, 100, 112),
}
COL_P1 = COLORS["S"]
COL_P2 = COLORS["I"]
COL_B2B = (255, 206, 92)
COL_PC = (255, 236, 150)

SETUP, COUNT, PLAY, FINISH, OVER = "setup", "count", "play", "finish", "over"
VARIANTS = ("marathon", "sprint", "ultra")
SPRINT_LINES = 40
ULTRA_TIME = 120.0
SPRINT_ACH_TIME = 120.0
COUNT_TIME = 1.25
GO_TIME = 0.7
FINISH_TIME = {"topout": 1.5, "sprint": 1.4, "ultra": 1.4, "ko": 2.0}
OVER_LOCK = 0.5
VS_LEVEL_EVERY = 40.0            # Versus: alle 40 s ein Level schneller ...
VS_LEVEL_MAX = 12                # ... bis Level 12
CLEAR_ANIM = 0.2
DAS_RANGE = (50, 400, 10)        # min, max, Schritt (ms)
ARR_RANGE = (0, 200, 5)

KEYS_SOLO = dict(hold=("c", "C", "Shift_L", "Shift_R"),
                 ccw=("z", "Z", "y", "Y", "Control_R"), cw=("x", "X"))
KEYS_P1 = dict(hold=("q", "Q"), ccw=("e", "E"), cw=())
KEYS_P2 = dict(hold=("Shift_R",), ccw=("Control_R",), cw=())
RESTART_KEYS = ("r", "R")
SETUP_KEYS = ("s", "S")


def _rgba(color, alpha):
    return (color[0], color[1], color[2], int(max(0, min(255, alpha))))


def _ease_out(p):
    p = max(0.0, min(1.0, p))
    return 1.0 - (1.0 - p) ** 3


def fmt_time(sec, cents=True):
    """Sekunden als "m:ss.cc" (bzw. "m:ss")."""
    sec = max(0.0, sec)
    m = int(sec // 60)
    s = sec - 60 * m
    if cents:
        return "%d:%05.2f" % (m, math.floor(s * 100) / 100.0)
    return "%d:%02d" % (m, int(s))


class _Pad:
    """Eingabe-Zustand eines Spielers: gehaltene Tasten + eigenes DAS/ARR."""

    def __init__(self):
        self.left = set()
        self.right = set()
        self.down = set()
        self.dir = 0            # -1 / +1 = aktive Schieberichtung
        self.held_ms = 0.0
        self.auto = 0           # schon ausgeführte automatische Schritte

    def press(self, d, key):
        (self.left if d < 0 else self.right).add(key)
        self.dir = d
        self.held_ms = 0.0
        self.auto = 0

    def release(self, key):
        self.down.discard(key)
        for d, keys in ((-1, self.left), (1, self.right)):
            if key in keys:
                keys.discard(key)
                if not keys and self.dir == d:
                    other = self.right if d < 0 else self.left
                    self.dir = -d if other else 0
                    self.held_ms = 0.0
                    self.auto = 0


class _Fx:
    """Rein optische Effekte eines Spielfelds."""

    def __init__(self):
        self.shake = 0.0
        self.clear_t = 9.0
        self.clear_rows = []        # sichtbare Zeilen (alte Lage) + Farben
        self.shift = {}             # neue Zeile -> um wie viele Zeilen gerutscht
        self.flash_cells = []
        self.flash_t = 9.0
        self.trail = None           # (spalten -> (von_y, bis_y), art)
        self.trail_t = 9.0
        self.level_t = 9.0
        self.rise_t = 9.0
        self.rise_n = 0
        self.cancel_t = 9.0
        self.ko_t = -1.0            # Sekunden seit dem K.O. / Top Out


class _AIPlayer:
    """Steuert ein Feld über tetris_ai: denken, dann Eingabe für Eingabe."""

    def __init__(self, level, seed):
        self.level = max(0, min(2, level))
        self.cfg = ai.LEVELS[self.level]
        self.rng = random.Random(seed)
        self.piece = -1
        self.acts = []
        self.timer = 0.0
        self.replanned = False

    def update(self, dt, board):
        if not board.active:
            return
        if board.piece_id != self.piece:
            self.piece = board.piece_id
            self.acts = ai.plan(board, self.level, self.rng)
            self.timer = self.cfg["think"] * self.rng.uniform(0.7, 1.3)
            self.replanned = False
        self.timer -= dt
        guard = 0
        while self.timer <= 0 and self.acts and board.active and guard < 16:
            guard += 1
            act = self.acts.pop(0)
            ok = ai.apply(board, act)
            if act == "drop":
                self.acts = []
                break
            if act == "hold":
                self.piece = board.piece_id
            elif not ok and not self.replanned:
                # Weg versperrt (Schwerkraft war schneller) -> neu planen.
                self.acts = ai.plan(board, self.level, self.rng,
                                    allow_hold=False)
                self.replanned = True
            self.timer += self.cfg["step"] * self.rng.uniform(0.8, 1.2)


class TetrisGame(Game):
    name = "Tetris"
    highscore_key = "tetris"
    supports_multiplayer = True
    MODES = [("solo", "tetris.mode.solo"), ("versus_ai", "tetris.mode.versus_ai"),
             ("multi", "tetris.mode.multi")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        if self.mode not in ("solo", "versus_ai", "multi"):
            self.mode = "solo"
        self.versus = self.mode != "solo"
        self._load_options()
        self._make_fonts()
        self.anim_t = 0.0
        self.wins = [0, 0]
        self.best = self._load_best()
        self.particles = []
        self.callouts = []
        self.missiles = []
        self._sprites = {}
        self._caches = {}
        self.boards = []
        self.pads = []
        self.fx = []
        self.ai = None
        self.result = None
        self.elapsed = 0.0
        self.go_t = 0.0
        self.over_at = 0.0
        self.setup_focus = 0
        self.setup_hover = None
        self.over_rects = {}
        self.state = SETUP
        self._deco = [self._new_deco(True) for _ in range(9)]
        self._layout()
        self._build_setup_layout()

    @property
    def show_highscore_banner(self):
        return self.mode == "solo" and self.variant == "marathon"

    def _load_options(self):
        ts = self.settings.get("tetris", {}) if isinstance(self.settings, dict) else {}
        d = settings_mod.DEFAULTS["tetris"]

        def num(key, lo, hi):
            v = ts.get(key, d[key])
            if isinstance(v, bool) or not isinstance(v, int):
                v = d[key]
            return max(lo, min(hi, v))

        self.variant = ts.get("solo") if ts.get("solo") in VARIANTS else d["solo"]
        self.start_level = num("start_level", 1, 15)
        self.ai_level = num("ai_level", 0, 2)
        self.das = num("das", DAS_RANGE[0], DAS_RANGE[1])
        self.arr = num("arr", ARR_RANGE[0], ARR_RANGE[1])
        g = ts.get("ghost", d["ghost"])
        self.ghost = g if isinstance(g, bool) else True

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("tetris", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _load_best(self):
        data = store.load_section("tetris")
        out = {"sprint": None, "sprint_pps": 0.0, "ultra": 0, "ultra_lines": 0,
               "vs": [[0, 0], [0, 0], [0, 0]]}
        if not isinstance(data, dict):
            return out
        sp = data.get("sprint")
        if isinstance(sp, (int, float)) and not isinstance(sp, bool) and sp > 0:
            out["sprint"] = float(sp)
        for key in ("sprint_pps",):
            v = data.get(key)
            if isinstance(v, (int, float)) and not isinstance(v, bool):
                out[key] = float(v)
        for key in ("ultra", "ultra_lines"):
            v = data.get(key)
            if isinstance(v, int) and not isinstance(v, bool) and v > 0:
                out[key] = v
        vs = data.get("vs")
        if isinstance(vs, list) and len(vs) == 3:
            for i, pair in enumerate(vs):
                if (isinstance(pair, list) and len(pair) == 2
                        and all(isinstance(n, int) and n >= 0 for n in pair)):
                    out["vs"][i] = [pair[0], pair[1]]
        return out

    def _save_best(self):
        data = {"ultra": self.best["ultra"], "ultra_lines": self.best["ultra_lines"],
                "vs": self.best["vs"]}
        if self.best["sprint"] is not None:
            data["sprint"] = round(self.best["sprint"], 3)
            data["sprint_pps"] = round(self.best["sprint_pps"], 2)
        store.save_section("tetris", data)

    def _make_fonts(self):
        """Theme-Schriften, Größen aus der Fensterhöhe abgeleitet."""
        h = self.height
        self._huge = ui.font(max(26, min(64, h // 11)), bold=True)
        self._big = ui.font(max(20, min(44, h // 16)), bold=True)
        self._small = ui.font(max(13, min(22, h // 30)))
        self._small_b = ui.font(max(13, min(22, h // 30)), bold=True)
        self._tiny = ui.font(max(11, min(18, h // 38)))

    def on_surface_changed(self):
        self._make_fonts()
        self._sprites = {}
        self._caches = {}
        self._layout()
        self._build_setup_layout()

    # ===================================================== Layout
    def _layout(self):
        """Feld-, Hold-, Vorschau- und Statistik-Rechtecke je Seite."""
        W, H = self.width, self.height
        self.sides = []
        if not self.versus:
            m = max(8, H // 28)
            c = max(8, min((H - 2 * m) // VISIBLE, (W - 32) // 22))
            fw, fh = COLS * c, VISIBLE * c
            fx, fy = (W - fw) // 2, (H - fh) // 2
            gap = max(8, c // 2)
            side_w = max(3 * c, min(int(6.8 * c), fx - gap - 10))
            lx, rx = fx - gap - side_w, fx + fw + gap
            hold = pygame.Rect(lx, fy, side_w, int(4.0 * c))
            stats = pygame.Rect(lx, hold.bottom + gap, side_w,
                                fy + fh - hold.bottom - gap)
            nxt = pygame.Rect(rx, fy, side_w, int(14.2 * c))
            self.sides.append(dict(field=pygame.Rect(fx, fy, fw, fh), cell=c,
                                   hold=hold, next=nxt, stats=stats, bar=None,
                                   compact=False, mini=max(5, int(c * 0.78)),
                                   mini2=max(4, int(c * 0.62)), pad=max(6, c // 3)))
        else:
            top = max(24, H // 14)
            bottom = max(20, H // 20)
            c = max(6, min((H - top - bottom - 6) // VISIBLE, int((W - 16) / 35.6)))
            fw, fh = COLS * c, VISIBLE * c
            side = max(16, int(2.95 * c))
            barw = max(4, int(0.42 * c))
            gap = max(3, c // 4)
            side_w = side + gap + barw + fw + gap + side
            center = max(10, c)
            x0 = (W - (2 * side_w + center)) // 2
            fy = top + (H - top - bottom - fh) // 2
            mini = max(4, int(c * 0.6))
            for i in range(2):
                sx = x0 + i * (side_w + center)
                hold = pygame.Rect(sx, fy, side, int(3.3 * c))
                bar = pygame.Rect(sx + side + gap, fy, barw, fh)
                field = pygame.Rect(bar.right, fy, fw, fh)
                nxt = pygame.Rect(field.right + gap, fy, side, int(12.4 * c))
                self.sides.append(dict(field=field, cell=c, hold=hold, next=nxt,
                                       stats=None, bar=bar, compact=True,
                                       mini=mini, mini2=max(4, int(c * 0.5)),
                                       pad=max(4, c // 4)))
        self._hud_fonts()

    def _hud_fonts(self):
        c = self.sides[0]["cell"] if self.sides else 20
        compact = self.versus
        self._f_label = ui.font(max(10, min(17, int(c * (0.56 if compact else 0.62)))),
                                bold=True)
        self._f_value = ui.font(max(12, min(30, int(c * (0.8 if compact else 0.95)))),
                                bold=True)
        self._f_hero = ui.font(max(16, min(46, int(c * 1.35))), bold=True)
        self._f_call = ui.font(max(12, min(34, int(c * (0.78 if compact else 0.92)))),
                               bold=True)
        self._f_call_s = ui.font(max(10, min(24, int(c * 0.62))), bold=True)

    # ===================================================== Setup-Screen
    def _setup_items(self):
        """Fokussierbare Einträge des Setups in Tastatur-Reihenfolge."""
        items = []
        if self.mode == "solo":
            items += ["variant", "level"]
        elif self.mode == "versus_ai":
            items += ["ai"]
        items += ["ghost", "das", "arr", "start"]
        return items

    def _build_setup_layout(self):
        W, H = self.width, self.height
        cx = W // 2
        bw = min(max(500, int(W * 0.64)), W - 40)
        tiny_h = self._tiny.get_height()
        lab_h = tiny_h + 3
        btn_h = max(28, min(48, H // 13))
        line_h = tiny_h + 5
        self.setup_title_y = int(H * 0.095)
        self.setup_sub_y = int(H * 0.165)
        top = int(H * 0.215)
        bottom = H - (2 * tiny_h + 20)

        blocks = []
        if self.mode in ("solo", "versus_ai"):
            blocks.append(("choice", lab_h + btn_h + 2 * line_h + 2))
        else:
            blocks.append(("keys", 2 * line_h + 18))
        blocks.append(("opts", lab_h + btn_h + line_h + 2))
        blocks.append(("start", btn_h + 4))
        total = sum(h for _k, h in blocks)
        sp = max(4, (bottom - top - total) // (len(blocks) + 1))
        y = top + sp
        gap = 8
        self.setup_rects = {}
        for kind, h in blocks:
            if kind == "choice":
                n = 3
                w = (bw - gap * (n - 1)) // n
                rects = [pygame.Rect(cx - bw // 2 + i * (w + gap), y + lab_h, w, btn_h)
                         for i in range(n)]
                self.setup_rects["choice"] = rects
                self.setup_choice_label_y = y
                self.setup_desc_y = y + lab_h + btn_h + 3
            elif kind == "keys":
                self.setup_rects["keys"] = pygame.Rect(cx - bw // 2, y, bw, h)
            elif kind == "opts":
                names = (["level"] if self.mode == "solo" else []) + ["ghost", "das", "arr"]
                n = len(names)
                w = (bw - gap * (n - 1)) // n
                for i, name in enumerate(names):
                    self.setup_rects[name] = pygame.Rect(cx - bw // 2 + i * (w + gap),
                                                         y + lab_h, w, btn_h)
                self.setup_opts_label_y = y
                self.setup_help_y = y + lab_h + btn_h + 3
            else:
                sw = min(bw, max(180, W // 3))
                self.setup_rects["start"] = pygame.Rect(cx - sw // 2, y, sw, btn_h)
            y += h + sp
        self.setup_footer_y = (H - 2 * tiny_h - 12, H - tiny_h - 6)

    def _setup_change(self, item, d):
        """Wert eines Setup-Eintrags um d ändern (Pfeile / Klick)."""
        if item == "variant":
            i = (VARIANTS.index(self.variant) + d) % len(VARIANTS)
            self.variant = VARIANTS[i]
            self._save_setting("solo", self.variant)
        elif item == "ai":
            self.ai_level = (self.ai_level + d) % 3
            self._save_setting("ai_level", self.ai_level)
        elif item == "level":
            if self.variant != "marathon":
                return
            self.start_level = max(1, min(15, self.start_level + d))
            self._save_setting("start_level", self.start_level)
        elif item == "ghost":
            self.ghost = not self.ghost
            self._save_setting("ghost", self.ghost)
        elif item == "das":
            lo, hi, step = DAS_RANGE
            self.das = max(lo, min(hi, self.das + d * step))
            self._save_setting("das", self.das)
        elif item == "arr":
            lo, hi, step = ARR_RANGE
            self.arr = max(lo, min(hi, self.arr + d * step))
            self._save_setting("arr", self.arr)
        else:
            return
        self.play_sound("move")

    def _handle_setup(self, event):
        items = self._setup_items()
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            self.setup_focus = max(0, min(len(items) - 1, self.setup_focus))
            item = items[self.setup_focus]
            if k in ("Up", "w", "W"):
                self.setup_focus = (self.setup_focus - 1) % len(items)
                self.play_sound("click")
            elif k in ("Down", "s", "S", "Tab"):
                self.setup_focus = (self.setup_focus + 1) % len(items)
                self.play_sound("click")
            elif k in ("Left", "a", "A", "minus", "KP_Subtract"):
                self._setup_change(item, -1)
            elif k in ("Right", "d", "D", "plus", "KP_Add"):
                self._setup_change(item, 1)
            elif k in ("1", "2", "3") and self.mode in ("solo", "versus_ai"):
                if self.mode == "solo":
                    self.variant = VARIANTS[int(k) - 1]
                    self._save_setting("solo", self.variant)
                else:
                    self.ai_level = int(k) - 1
                    self._save_setting("ai_level", self.ai_level)
                self.play_sound("click")
            elif k in ("g", "G"):
                self._setup_change("ghost", 1)
            elif k in ("Return", "KP_Enter", "space") and not event.repeat:
                if item != "start" and k == "space" and item in ("ghost",):
                    self._setup_change(item, 1)
                else:
                    self._start()
        elif event.kind == InputEvent.MOUSEMOVE:
            self.setup_hover = self._setup_hit(event.pos)
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            hit = self._setup_hit(event.pos)
            if hit is None:
                return
            item, value = hit
            if item in items:
                self.setup_focus = items.index(item)
            if item == "start":
                self._start()
            elif item in ("variant", "ai"):
                if item == "variant":
                    self.variant = VARIANTS[value]
                    self._save_setting("solo", self.variant)
                else:
                    self.ai_level = value
                    self._save_setting("ai_level", self.ai_level)
                self.play_sound("click")
            else:
                self._setup_change(item, value)

    def _setup_hit(self, pos):
        """(eintrag, wert) unter der Maus - bei Steppern -1/+1 je Hälfte."""
        rects = self.setup_rects
        for i, r in enumerate(rects.get("choice", [])):
            if r.collidepoint(pos):
                return ("variant" if self.mode == "solo" else "ai"), i
        for name in ("level", "das", "arr"):
            r = rects.get(name)
            if r is not None and r.collidepoint(pos):
                return name, (-1 if pos[0] < r.centerx else 1)
        r = rects.get("ghost")
        if r is not None and r.collidepoint(pos):
            return "ghost", 1
        if rects["start"].collidepoint(pos):
            return "start", 0
        return None

    # ===================================================== Partie
    def _start(self):
        """Neue Runde mit den aktuellen Setup-Werten (mit Countdown)."""
        seed = random.getrandbits(32)
        self.seed = seed
        if self.versus:
            self.boards = [core.Board(seed, 1, fixed_level=True),
                           core.Board(seed, 1, fixed_level=True)]
        elif self.variant == "marathon":
            self.boards = [core.Board(seed, self.start_level)]
        else:
            self.boards = [core.Board(seed, 1, fixed_level=True)]
        self.pads = [_Pad() for _ in self.boards]
        self.fx = [_Fx() for _ in self.boards]
        self.ai = _AIPlayer(self.ai_level, seed) if self.mode == "versus_ai" else None
        self.particles = []
        self.callouts = []
        self.missiles = []
        self._caches = {}
        self.elapsed = 0.0
        self.count_t = COUNT_TIME
        self.go_t = 0.0
        self.result = None
        self.score = 0
        self.game_over = False
        self.state = COUNT
        self._layout()
        self.play_sound("select")

    def _human_slots(self):
        if self.multiplayer:
            return ((0, "p1", KEYS_P1), (1, "p2", KEYS_P2))
        return ((0, None, KEYS_SOLO),)

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.state == OVER:
            self._handle_over(event)
            return
        if event.kind == InputEvent.KEYUP:
            for pad in self.pads:
                pad.release(event.key)
            return
        if event.kind != InputEvent.KEYDOWN or event.repeat:
            return
        if self.state not in (COUNT, PLAY):
            return
        key = event.key
        # Schneller Neustart nur in Sprint/Ultra - ein laufender Marathon würde
        # sonst seine Punkte verlieren, bevor main.py den Highscore sichert.
        if (self.mode == "solo" and self.variant != "marathon"
                and key in RESTART_KEYS and self.key_is_free(key)):
            self._start()
            return
        live = self.state == PLAY
        for slot, player, extra in self._human_slots():
            self._play_key(slot, player, extra, key, live)

    def _play_key(self, slot, player, extra, key, live):
        board = self.boards[slot]
        pad = self.pads[slot]
        act = self.is_action
        if act(key, "left", player) or act(key, "right", player):
            d = -1 if act(key, "left", player) else 1
            pad.press(d, key)
            if live and board.move(d):
                self.play_sound("move")
        elif act(key, "down", player):
            pad.down.add(key)
            if live:
                board.soft_step()
        elif act(key, "up", player):
            if live:
                self._rotate(slot, 1)
        elif act(key, "action", player):
            if live:
                self._hard_drop(slot)
        elif key in extra["hold"] and self.key_is_free(key):
            if live and board.hold():
                self.play_sound("select")
        elif key in extra["ccw"] and self.key_is_free(key):
            if live:
                self._rotate(slot, -1)
        elif key in extra["cw"] and self.key_is_free(key):
            if live:
                self._rotate(slot, 1)

    def _rotate(self, slot, d):
        if self.boards[slot].rotate(d):
            self.play_sound("rotate")

    def _hard_drop(self, slot):
        b = self.boards[slot]
        if not b.active:
            return
        f = self.fx[slot]
        gy = b.ghost_y()
        if gy > b.y:
            cols = {}
            for x, y in b.cells_of():
                top, _bot = cols.get(x, (y, y))
                cols[x] = (min(top, y), y + gy - b.y)
            f.trail = (cols, b.kind)
            f.trail_t = 0.0
        f.shake = max(f.shake, 0.12)
        b.hard_drop()

    def _handle_over(self, event):
        if time.monotonic() - self.over_at < OVER_LOCK:
            return
        if event.kind == InputEvent.KEYDOWN and not event.repeat:
            if event.key in RESTART_KEYS:
                self._start()
            elif event.key in SETUP_KEYS:
                self._to_setup()
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            for name, r in self.over_rects.items():
                if r.collidepoint(event.pos):
                    if name == "again":
                        self._start()
                    else:
                        self._to_setup()
                    return

    def _to_setup(self):
        self.game_over = False
        self.state = SETUP
        self.score = 0
        self.boards = []
        self.particles = []
        self.callouts = []
        self.missiles = []
        self._layout()
        self._build_setup_layout()
        self.play_sound("click")

    # ===================================================== Logik
    def update(self, dt):
        self.anim_t += dt
        self._update_effects(dt)
        if self.state == SETUP:
            self._update_deco(dt)
            return
        if self.state == COUNT:
            self.count_t -= dt
            if self.count_t <= 0:
                self.state = PLAY
                self.go_t = GO_TIME
                self.play_sound("powerup")
            return
        if self.state == FINISH:
            self.finish_t += dt
            for f in self.fx:
                if f.ko_t >= 0:
                    f.ko_t += dt
            if self.finish_t >= self.finish_dur:
                self.state = OVER
                self.game_over = True
                self.over_at = time.monotonic()
            return
        if self.state != PLAY:
            return

        self.elapsed += dt
        self.go_t = max(0.0, self.go_t - dt)
        if self.versus:
            lvl = min(VS_LEVEL_MAX, 1 + int(self.elapsed // VS_LEVEL_EVERY))
            if lvl != self.boards[0].level:
                for i, b in enumerate(self.boards):
                    b.level = lvl
                    self._callout(i, [(t("tetris.call.level", n=lvl), ui.GOLD)])
                    self.fx[i].level_t = 0.0
                self.play_sound("level")

        humans = {slot for slot, _p, _e in self._human_slots()}
        for i, b in enumerate(self.boards):
            pad = self.pads[i]
            if i in humans:
                self._das(pad, b, dt)
            b.tick(dt, soft=bool(pad.down) and i in humans)
        if self.ai is not None:
            self.ai.update(dt, self.boards[1])
        for i in range(len(self.boards)):
            self._process_events(i)
        self._check_end()
        if self.mode == "solo" and self.variant == "marathon":
            self.score = self.boards[0].score

    def _das(self, pad, board, dt):
        """Dauerbewegung nach DAS, danach im ARR-Takt (mehrere Schritte je Frame)."""
        if pad.dir == 0:
            return
        pad.held_ms += dt * 1000.0
        if pad.held_ms < self.das or not board.active:
            return
        moved = False
        if self.arr <= 0:
            while board.move(pad.dir):
                moved = True
        else:
            target = 1 + int((pad.held_ms - self.das) // self.arr)
            n = min(target - pad.auto, COLS)
            pad.auto = target
            for _ in range(n):
                if not board.move(pad.dir):
                    break
                moved = True
        if moved:
            self.play_sound("move")

    def _process_events(self, i):
        b = self.boards[i]
        f = self.fx[i]
        human = self._is_human(i)
        for kind, data in b.events:
            if kind == "lock":
                self._on_lock(i, data)
            elif kind == "level":
                f.level_t = 0.0
                self._callout(i, [(t("tetris.call.level", n=data), ui.GOLD)])
                self.play_sound("level")
            elif kind == "garbage":
                f.rise_t = 0.0
                f.rise_n = data
                f.shake = max(f.shake, 0.2)
                if human:
                    self.play_sound("hit")
                    self.rumble(140)
        b.events.clear()

    def _is_human(self, i):
        return i == 0 or self.multiplayer

    def _on_lock(self, i, info):
        b = self.boards[i]
        f = self.fx[i]
        human = self._is_human(i)
        side = self.sides[i]
        c = side["cell"]
        field = side["field"]
        f.flash_cells = [(x, y) for x, y in info["cells"]]
        f.flash_t = 0.0
        lines = info["lines"]
        if human or lines:
            self.play_sound("lock" if not lines else "line")
        if lines:
            f.clear_t = 0.0
            f.clear_rows = [(r, info["cleared"][k]) for k, r in enumerate(info["rows"])]
            full = set(info["rows"])
            kept = [o for o in range(core.ROWS) if o not in full]
            f.shift = {}
            for j, o in enumerate(kept):
                k = lines + j - o
                if k > 0 and lines + j >= BUFFER:
                    f.shift[lines + j] = k
            for r, row in f.clear_rows:
                if r < BUFFER:
                    continue
                y = field.y + (r - BUFFER) * c + c // 2
                for x, kind in enumerate(row):
                    col = COLORS.get(kind or "G", COLORS["G"])
                    for _ in range(2 if human else 1):
                        self._particle(field.x + x * c + c // 2, y, col, c)
            if lines == 4 or info["spin"]:
                f.shake = max(f.shake, 0.18)
                if human:
                    self.rumble(160)

        # Einblendungen
        rows = []
        spin = info["spin"]
        if spin:
            word = t("tetris.call.tspin_mini" if spin == "mini" else "tetris.call.tspin")
            if lines:
                word = t("tetris.call.spin_lines", spin=word,
                         lines=t("tetris.call.lines%d" % lines))
            rows.append((word, COLORS["T"]))
        elif lines == 4:
            rows.append((t("tetris.call.tetris"), COLORS["I"]))
        if info["b2b"]:
            rows.append((t("tetris.call.b2b"), COL_B2B))
        if info["combo"] >= 1:
            rows.append((t("tetris.call.combo", n=info["combo"]),
                         ui.mix(COLORS["L"], (255, 255, 255), 0.15)))
        if info["pc"]:
            rows.append((t("tetris.call.pc"), COL_PC))
        if rows:
            small = []
            if not self.versus and info["points"] and self.variant != "sprint":
                small.append("+%d" % info["points"])
            elif self.versus and info["sent"]:
                small.append("+%d" % info["sent"])
            self._callout(i, rows, small)
        if spin and human:
            self._tone(880 if spin == "full" else 660, 0.12, "square", 0.3)
        if info["pc"]:
            self.play_sound("win")
            for _ in range(40):
                self._particle(field.centerx + random.uniform(-field.w / 2, field.w / 2),
                               field.bottom - random.uniform(0, field.h * 0.3),
                               random.choice([COL_PC, COLORS["I"], COLORS["T"]]), c)
        elif lines == 4 and human:
            self.play_sound("powerup")

        # Erfolge (nicht im 2-Spieler-Modus, nur das eigene Feld)
        if human and not self.multiplayer:
            if lines == 4:
                self.ach_event("tetris_four")
            if spin == "full" and lines == 2:
                self.ach_event("tetris_tspin")
            if info["pc"]:
                self.ach_event("tetris_pc")

        # Versus: Müll zum Gegner schicken, Aufrechnen anzeigen
        if self.versus:
            if info["cancelled"]:
                f.cancel_t = 0.0
            if info["sent"]:
                other = 1 - i
                self.boards[other].receive(info["sent"])
                src = self.sides[i]["field"]
                dst = self.sides[other]["bar"]
                self.missiles.append(dict(x0=src.centerx, y0=src.y + src.h * 0.4,
                                          x1=dst.centerx, y1=dst.bottom - c,
                                          t=0.0, dur=0.38, n=info["sent"],
                                          color=COL_P1 if i == 0 else COL_P2))

    def _check_end(self):
        if not self.versus:
            b = self.boards[0]
            if b.dead:
                self._finish("topout")
            elif self.variant == "sprint" and b.lines >= SPRINT_LINES:
                self._finish("sprint")
            elif self.variant == "ultra" and self.elapsed >= ULTRA_TIME:
                self.elapsed = ULTRA_TIME
                self._finish("ultra")
            return
        dead = [i for i, b in enumerate(self.boards) if b.dead]
        if dead:
            winner = None if len(dead) == 2 else 1 - dead[0]
            self._finish("ko", winner)

    def _stats_of(self, i):
        b = self.boards[i]
        el = max(0.001, self.elapsed)
        return dict(lines=b.lines, score=b.score, level=b.level, pieces=b.pieces,
                    pps=b.pieces / el, attack=b.attack_sent, tetrises=b.tetrises,
                    tspins=b.tspins)

    def _finish(self, reason, winner=None):
        self.state = FINISH
        self.finish_t = 0.0
        self.finish_dur = FINISH_TIME[reason]
        for b in self.boards:
            b.active = False
        res = dict(reason=reason, winner=winner, time=self.elapsed,
                   stats=[self._stats_of(i) for i in range(len(self.boards))],
                   new_best=False, prev_best=None)
        b0 = self.boards[0]
        if not self.versus:
            if reason == "topout":
                self.fx[0].ko_t = 0.0
                self.play_sound("gameover")
                self.rumble(220)
                if self.variant == "marathon":
                    self.score = b0.score
            elif reason == "sprint":
                res["prev_best"] = self.best["sprint"]
                if self.best["sprint"] is None or self.elapsed < self.best["sprint"]:
                    self.best["sprint"] = self.elapsed
                    self.best["sprint_pps"] = res["stats"][0]["pps"]
                    res["new_best"] = True
                    self._save_best()
                if self.elapsed < SPRINT_ACH_TIME:
                    self.ach_event("tetris_sprint")
                self.play_sound("win")
                self._confetti(0)
            else:
                res["prev_best"] = self.best["ultra"] or None
                if b0.score > self.best["ultra"]:
                    self.best["ultra"] = b0.score
                    self.best["ultra_lines"] = b0.lines
                    res["new_best"] = True
                    self._save_best()
                self.play_sound("win")
                self._confetti(0)
        else:
            if winner is not None:
                self.wins[winner] += 1
            for i, b in enumerate(self.boards):
                if b.dead:
                    self.fx[i].ko_t = 0.0
                    field = self.sides[i]["field"]
                    c = self.sides[i]["cell"]
                    for _ in range(50):
                        self._particle(random.uniform(field.left, field.right),
                                       random.uniform(field.centery, field.bottom),
                                       random.choice(list(COLORS.values())[:7]), c,
                                       power=1.8)
            self.play_sound("explode")
            self.rumble(260)
            if self.mode == "versus_ai" and winner is not None:
                rec = self.best["vs"][self.ai_level]
                rec[0 if winner == 0 else 1] += 1
                self._save_best()
                self.report_result(winner == 0)
            if winner is not None:
                self._confetti(winner)
        self.result = res

    # ===================================================== Effekte
    def _tone(self, freq, dur, wave="sine", vol=0.3):
        audio.tone(freq, dur, self.settings, wave, vol)

    def _particle(self, x, y, color, c, power=1.0):
        if len(self.particles) > 420:
            return
        sp = c * 9.0 * power
        a = random.uniform(0, math.tau)
        v = random.uniform(0.25, 1.0) * sp
        life = random.uniform(0.45, 0.9)
        self.particles.append([x, y, math.cos(a) * v, math.sin(a) * v - sp * 0.5,
                               life, life, color, max(2, int(c * random.uniform(0.14, 0.3)))])

    def _confetti(self, i):
        field = self.sides[i]["field"]
        c = self.sides[i]["cell"]
        for _ in range(60):
            self._particle(random.uniform(field.left, field.right),
                           field.y + random.uniform(0, field.h * 0.5),
                           random.choice(list(COLORS.values())[:7]), c, power=1.3)

    def _callout(self, i, rows, small=()):
        """Einblendung über dem Feld: große Zeilen + optionale kleine Zeilen."""
        field = self.sides[i]["field"]
        max_w = field.w - 8
        surfs = []
        for text, color in rows:
            surfs.append(self._fit_render(text, self._f_call, color, max_w))
        for text in small:
            surfs.append(self._fit_render(text, self._f_call_s, ui.TEXT, max_w))
        # ältere Einblendung desselben Felds verdrängen
        self.callouts = [cl for cl in self.callouts if cl["side"] != i]
        self.callouts.append(dict(side=i, surfs=surfs, t=0.0, dur=1.25))

    def _fit_render(self, text, fnt, color, max_w):
        img = fnt.render(text, True, color)
        if img.get_width() > max_w > 10:
            h = max(6, int(img.get_height() * max_w / img.get_width()))
            img = pygame.transform.smoothscale(img.convert_alpha(), (max_w, h))
        return img

    def _update_effects(self, dt):
        for f in self.fx:
            f.shake = max(0.0, f.shake - dt)
            f.clear_t += dt
            f.flash_t += dt
            f.trail_t += dt
            f.level_t += dt
            f.rise_t += dt
            f.cancel_t += dt
        alive = []
        for p in self.particles:
            p[4] -= dt
            if p[4] <= 0:
                continue
            p[0] += p[2] * dt
            p[1] += p[3] * dt
            p[3] += 1400 * dt * (p[7] / 6.0)
            alive.append(p)
        self.particles = alive
        for cl in self.callouts:
            cl["t"] += dt
        self.callouts = [cl for cl in self.callouts if cl["t"] < cl["dur"]]
        keep = []
        for m in self.missiles:
            m["t"] += dt
            if m["t"] < m["dur"]:
                keep.append(m)
            else:
                for _ in range(8):
                    self._particle(m["x1"], m["y1"], m["color"], 12, power=0.8)
        self.missiles = keep

    def _new_deco(self, anywhere=False):
        return [random.choice(core.KINDS), random.uniform(0, 1),
                random.uniform(0, 1) if anywhere else random.uniform(-0.3, -0.1),
                random.uniform(0.02, 0.06), random.randint(0, 3)]

    def _update_deco(self, dt):
        for d in self._deco:
            d[2] += d[3] * dt
        self._deco = [d if d[2] < 1.15 else self._new_deco() for d in self._deco]

    # ===================================================== Zeichen-Helfer
    def _sprite(self, kind, c, style="block"):
        """Gecachter Block (Identitätsfarbe) in Zellgröße c."""
        flat = ui.fx("style") == "v1"
        key = (kind, c, style, flat)
        spr = self._sprites.get(key)
        if spr is not None:
            return spr
        base = COLORS.get(kind, COLORS["G"])
        surf = pygame.Surface((c, c), pygame.SRCALPHA)
        s = c - 1
        if style == "ghost":
            pygame.draw.rect(surf, _rgba(base, 50), (1, 1, s - 1, s - 1),
                             border_radius=max(1, c // 7))
            pygame.draw.rect(surf, _rgba(base, 190), (1, 1, s - 1, s - 1),
                             max(1, c // 12), border_radius=max(1, c // 7))
        elif style == "white":
            pygame.draw.rect(surf, (255, 255, 255, 255), (0, 0, s, s),
                             border_radius=max(1, c // 8))
        elif flat or c < 9:
            pygame.draw.rect(surf, base, (0, 0, s, s), border_radius=max(0, c // 9))
            pygame.draw.rect(surf, tuple(int(v * 0.7) for v in base), (0, 0, s, s),
                             1, border_radius=max(0, c // 9))
        else:
            bev = max(2, c // 6)
            light = ui.mix(base, (255, 255, 255), 0.42)
            dark = tuple(int(v * 0.58) for v in base)
            face = ui.mix(base, (255, 255, 255), 0.06)
            pygame.draw.rect(surf, dark, (0, 0, s, s), border_radius=max(1, c // 10))
            pygame.draw.polygon(surf, light, [(0, 0), (s - 1, 0), (s - bev, bev),
                                              (bev, bev), (bev, s - bev), (0, s - 1)])
            pygame.draw.rect(surf, face, (bev, bev, s - 2 * bev, s - 2 * bev))
            hl = pygame.Surface((max(1, (s - 2 * bev) // 2), max(1, bev // 2 + 1)),
                                pygame.SRCALPHA)
            hl.fill((255, 255, 255, 80))
            surf.blit(hl, (bev + 1, bev + 1))
        self._sprites[key] = surf
        return surf

    def _piece_img(self, kind, mc, alpha=255):
        """Kompletter Stein (Zustand 0) als eigene Fläche für Vorschau/Hold."""
        key = ("piece", kind, mc, alpha, ui.fx("style") == "v1")
        img = self._sprites.get(key)
        if img is None:
            sh = core.SHAPES[kind][0]
            w = (sh.maxx - sh.minx + 1) * mc
            h = (sh.maxy - sh.miny + 1) * mc
            img = pygame.Surface((w, h), pygame.SRCALPHA)
            spr = self._sprite(kind if alpha >= 255 else "X", mc)
            for cx, cy in sh.cells:
                img.blit(spr, ((cx - sh.minx) * mc, (cy - sh.miny) * mc))
            if 0 < alpha < 255:
                img.set_alpha(alpha)
            self._sprites[key] = img
        return img

    def _field_bg(self, w, h, c):
        key = ("fieldbg", w, h, c, ui.PANEL, ui.BORDER, ui.BG_BOTTOM)
        surf = self._caches.get(key)
        if surf is None:
            surf = pygame.Surface((w, h), pygame.SRCALPHA)
            base = ui.mix(ui.BG_BOTTOM, ui.PANEL, 0.35)
            for y in range(h):
                f = y / max(1, h - 1)
                pygame.draw.line(surf, _rgba(ui.mix(base, ui.PANEL, f * 0.5), 238),
                                 (0, y), (w, y))
            grid = _rgba(ui.mix(ui.PANEL, ui.BORDER, 0.45), 110)
            for cx in range(1, COLS):
                pygame.draw.line(surf, grid, (cx * c, 0), (cx * c, h))
            for cy in range(1, VISIBLE):
                pygame.draw.line(surf, grid, (0, cy * c), (w, cy * c))
            self._caches[key] = surf
        return surf

    def _stack_surface(self, i, c, gray_rows):
        b = self.boards[i]
        key = ("stack", i, b.version, c, gray_rows, ui.fx("style") == "v1")
        cached = self._caches.get(("stack", i))
        if cached is not None and cached[0] == key:
            return cached[1]
        surf = pygame.Surface((COLS * c, VISIBLE * c), pygame.SRCALPHA)
        for y in range(BUFFER, core.ROWS):
            row = b.cells[y]
            if not b.rows[y]:
                continue
            gray = (core.ROWS - y) <= gray_rows
            for x in range(COLS):
                kind = row[x]
                if kind is not None:
                    surf.blit(self._sprite("X" if gray else kind, c),
                              (x * c, (y - BUFFER) * c))
        self._caches[("stack", i)] = (key, surf)
        return surf

    def _white(self, w, h):
        key = ("white", w, h)
        surf = self._caches.get(key)
        if surf is None:
            surf = pygame.Surface((max(1, w), max(1, h)), pygame.SRCALPHA)
            surf.fill((255, 255, 255, 255))
            self._caches[key] = surf
        return surf

    def _dim(self, alpha=150):
        key = ("dim", self.width, self.height, alpha)
        surf = self._caches.get(key)
        if surf is None:
            surf = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
            surf.fill((8, 10, 16, alpha))
            self._caches[key] = surf
        return surf

    def _panel(self, s, rect, alpha=205):
        key = ("panel", rect.w, rect.h, ui.PANEL, alpha)
        surf = self._caches.get(key)
        if surf is None:
            surf = pygame.Surface(rect.size, pygame.SRCALPHA)
            pygame.draw.rect(surf, _rgba(ui.PANEL, alpha), surf.get_rect(),
                             border_radius=max(4, min(12, rect.h // 6)))
            self._caches[key] = surf
        s.blit(surf, rect.topleft)
        pygame.draw.rect(s, ui.BORDER, rect, 1, border_radius=max(4, min(12, rect.h // 6)))

    def _label(self, s, text, rect, y, color=None, align="center"):
        """Kleine Überschrift, die sich an die Breite anpasst (sonst weglassen)."""
        max_w = rect.w - 6
        img = self._f_label.render(text, True, color or ui.TEXT_DIM)
        if img.get_width() > max_w:
            small = ui.font(max(9, int(self._f_label.get_height() * 0.72)), bold=True)
            img = small.render(text, True, color or ui.TEXT_DIM)
            if img.get_width() > max_w:
                return y
        if align == "center":
            s.blit(img, img.get_rect(midtop=(rect.centerx, y)))
        else:
            s.blit(img, (rect.x + 8, y))
        return y + img.get_height()

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        if self.state == SETUP:
            self._draw_setup(s)
            self._draw_particles(s)
            return
        for i in range(len(self.boards)):
            self._draw_side(s, i)
        if self.versus:
            self._draw_vs_hud(s)
        self._draw_missiles(s)
        self._draw_particles(s)
        self._draw_callouts(s)
        if self.state == COUNT:
            self._draw_countdown(s)
        elif self.state == PLAY and self.go_t > 0:
            self._draw_go(s)
        elif self.state in (FINISH, OVER):
            self._draw_finish_marks(s)
        if self.state == OVER:
            self._draw_results(s)

    def _draw_side(self, s, i):
        L = self.sides[i]
        f = self.fx[i]
        field = L["field"]
        if f.shake > 0:
            amp = max(1, L["cell"] // 7)
            field = field.move(0, int(amp * math.sin(f.shake * 70) * f.shake * 6))
        self._draw_field(s, i, field, L["cell"])
        self._draw_hold(s, i, L)
        self._draw_next(s, i, L)
        if L["bar"] is not None:
            self._draw_bar(s, i, L)
        if L["stats"] is not None:
            self._draw_stats(s, L)

    def _draw_field(self, s, i, rect, c):
        b = self.boards[i]
        f = self.fx[i]
        s.blit(self._field_bg(rect.w, rect.h, c), rect.topleft)
        gray_rows = 0
        if f.ko_t >= 0:
            gray_rows = int(min(1.0, f.ko_t / 0.8) * VISIBLE + 0.999)
        stack = self._stack_surface(i, c, gray_rows)
        prev_clip = s.get_clip()
        s.set_clip(rect)
        if f.clear_t < CLEAR_ANIM and f.shift:
            e = _ease_out(f.clear_t / CLEAR_ANIM)
            for vy in range(VISIBLE):
                k = f.shift.get(vy + BUFFER, 0)
                dy = -int(k * c * (1.0 - e))
                s.blit(stack, (rect.x, rect.y + vy * c + dy), (0, vy * c, rect.w, c))
        elif f.rise_t < 0.14 and f.rise_n:
            e = _ease_out(f.rise_t / 0.14)
            s.blit(stack, (rect.x, rect.y + int(f.rise_n * c * (1.0 - e))))
        else:
            s.blit(stack, rect.topleft)

        # Aufblitzen der abgeräumten Zeilen
        if f.clear_t < 0.32:
            p = f.clear_t / 0.32
            bar = self._white(rect.w, c)
            for r, _row in f.clear_rows:
                if r < BUFFER:
                    continue
                w = int(rect.w * (0.55 + 0.45 * _ease_out(p * 1.6)))
                bar.set_alpha(int(230 * (1.0 - p)))
                s.blit(bar, (rect.centerx - w // 2, rect.y + (r - BUFFER) * c),
                       (0, 0, w, c))

        # Hard-Drop-Spur
        if f.trail is not None and f.trail_t < 0.2:
            cols, kind = f.trail
            strip = self._trail_strip(kind, c)
            strip.set_alpha(int(200 * (1.0 - f.trail_t / 0.2)))
            for x, (top, bot) in cols.items():
                y0 = max(top, BUFFER) - BUFFER
                y1 = bot - BUFFER
                if y1 <= y0:
                    continue
                hgt = (y1 - y0) * c
                s.blit(strip, (rect.x + x * c, rect.y + y0 * c),
                       (0, strip.get_height() - hgt, c, hgt))

        if b.active:
            if self.ghost:
                gy = b.ghost_y()
                if gy != b.y:
                    spr = self._sprite(b.kind, c, "ghost")
                    for x, y in b.cells_of(y=gy):
                        if y >= BUFFER:
                            s.blit(spr, (rect.x + x * c, rect.y + (y - BUFFER) * c))
            spr = self._sprite(b.kind, c)
            dim = min(1.0, b.lock_timer / core.LOCK_DELAY) if b.grounded() else 0.0
            for x, y in b.cells_of():
                if y >= BUFFER - 2:
                    px, py = rect.x + x * c, rect.y + (y - BUFFER) * c
                    s.blit(spr, (px, py))
                    if dim > 0.05:
                        w = self._white(c - 1, c - 1)
                        w.set_alpha(int(70 * dim))
                        s.blit(w, (px, py), special_flags=0)
        if f.flash_t < 0.14:
            w = self._white(c - 1, c - 1)
            w.set_alpha(int(150 * (1.0 - f.flash_t / 0.14)))
            for x, y in f.flash_cells:
                if y >= BUFFER:
                    s.blit(w, (rect.x + x * c, rect.y + (y - BUFFER) * c))
        s.set_clip(prev_clip)

        # Rahmen: Level-Up leuchtet, Gefahr pulsiert rot
        border = ui.BORDER_LIGHT
        width = 2
        height = b.stack_height()
        danger = not b.dead and (height >= 17 or height + b.pending_lines() >= 19)
        if danger:
            border = ui.mix(ui.BORDER_LIGHT, ui.RED, ui.pulse(7.0, 0.35, 1.0))
            width = 3
        if f.level_t < 0.8:
            border = ui.mix(ui.GOLD, border, f.level_t / 0.8)
            width = 3
        pygame.draw.rect(s, border, rect.inflate(4, 4), width,
                         border_radius=max(3, c // 5))

    def _trail_strip(self, kind, c):
        key = ("trail", kind, c)
        surf = self._caches.get(key)
        if surf is None:
            h = VISIBLE * c
            surf = pygame.Surface((c, h), pygame.SRCALPHA)
            col = COLORS.get(kind, COLORS["G"])
            for y in range(h):
                a = int(110 * (y / h) ** 2)
                pygame.draw.line(surf, _rgba(col, a), (c // 5, y), (c - c // 5 - 1, y))
            self._caches[key] = surf
        return surf

    def _draw_hold(self, s, i, L):
        b = self.boards[i]
        r = L["hold"]
        self._panel(s, r)
        y = self._label(s, t("tetris.hud.hold"), r, r.y + L["pad"] // 2 + 1)
        if b.hold_kind:
            img = self._piece_img(b.hold_kind, L["mini"], 255 if not b.hold_used else 120)
            area_top = y + 2
            cy = area_top + (r.bottom - area_top) // 2
            s.blit(img, img.get_rect(center=(r.centerx, cy)))

    def _draw_next(self, s, i, L):
        b = self.boards[i]
        r = L["next"]
        self._panel(s, r)
        y = self._label(s, t("tetris.hud.next"), r, r.y + L["pad"] // 2 + 1) + L["pad"] // 2
        kinds = b.queue.peek(5)
        slot1 = int(L["mini"] * 2.9)
        rest = (r.bottom - y - slot1 - 4) / 4.0
        for k, kind in enumerate(kinds):
            mc = L["mini"] if k == 0 else L["mini2"]
            img = self._piece_img(kind, mc)
            if k == 0:
                cy = y + slot1 // 2
            else:
                cy = y + slot1 + int(rest * (k - 0.5))
            s.blit(img, img.get_rect(center=(r.centerx, int(cy))))

    def _draw_bar(self, s, i, L):
        b = self.boards[i]
        f = self.fx[i]
        r = L["bar"]
        c = L["cell"]
        pygame.draw.rect(s, _rgba(ui.PANEL, 255), r, border_radius=2)
        y = r.bottom
        for n, _hole, age in b.pending:
            h = min(n * c, y - r.y)
            if h <= 0:
                break
            ready = age >= core.GARBAGE_DELAY
            col = ui.mix(ui.RED, (255, 255, 255), 0.25 * ui.pulse(9.0, 0.0, 1.0)) \
                if ready else ui.GOLD
            pygame.draw.rect(s, col, (r.x, y - h, r.w, h - 1), border_radius=2)
            y -= h
        if f.cancel_t < 0.3:
            w = self._white(r.w + 4, r.h)
            w.set_alpha(int(120 * (1.0 - f.cancel_t / 0.3)))
            s.blit(w, (r.x - 2, r.y))
        pygame.draw.rect(s, ui.BORDER, r.inflate(2, 2), 1, border_radius=2)

    def _stat_rows(self):
        b = self.boards[0]
        el = self.elapsed
        pps = "%.2f" % (b.pieces / el if el > 0.5 else 0.0)
        if self.variant == "sprint":
            left = max(0, SPRINT_LINES - b.lines)
            return [(t("tetris.hud.time"), fmt_time(el), True, None),
                    (t("tetris.hud.lines"), "%d / %d" % (min(b.lines, SPRINT_LINES),
                                                         SPRINT_LINES), False,
                     ui.GREEN if left <= 5 else None),
                    (t("tetris.hud.pps"), pps, False, None),
                    (t("tetris.hud.level"), str(b.level), False, None)]
        if self.variant == "ultra":
            rest = max(0.0, ULTRA_TIME - el)
            return [(t("tetris.hud.left"), fmt_time(rest), True,
                     ui.RED if rest < 10 else None),
                    (t("tetris.hud.score"), str(b.score), False, None),
                    (t("tetris.hud.lines"), str(b.lines), False, None),
                    (t("tetris.hud.pps"), pps, False, None)]
        return [(t("tetris.hud.score"), str(b.score), True, None),
                (t("tetris.hud.level"), str(b.level), False, None),
                (t("tetris.hud.lines"), str(b.lines), False, None),
                (t("tetris.hud.time"), fmt_time(el, cents=False), False, None),
                (t("tetris.hud.pps"), pps, False, None)]

    def _draw_stats(self, s, L):
        r = L["stats"]
        rows = self._stat_rows()
        pad = L["pad"]
        lab_h = self._f_label.get_height()
        y = r.y + pad
        used = []
        for label, value, hero, col in rows:
            fnt = self._f_hero if hero else self._f_value
            h = lab_h + fnt.get_height() + pad
            if y + h > r.bottom:
                break
            used.append((label, value, fnt, col, y))
            y += h
        panel = pygame.Rect(r.x, r.y, r.w, y - r.y)
        self._panel(s, panel)
        for label, value, fnt, col, yy in used:
            img = self._f_label.render(label, True, ui.TEXT_DIM)
            if img.get_width() > r.w - 12:
                img = pygame.transform.smoothscale(
                    img.convert_alpha(), (r.w - 12, img.get_height()))
            s.blit(img, img.get_rect(midtop=(r.centerx, yy)))
            val = self._fit_render(value, fnt, col or (self.accent if fnt is self._f_hero
                                                      else ui.TEXT), r.w - 10)
            s.blit(val, val.get_rect(midtop=(r.centerx, yy + lab_h)))

    def _draw_vs_hud(self, s):
        for i in range(2):
            L = self.sides[i]
            field = L["field"]
            col = COL_P1 if i == 0 else COL_P2
            if self.mode == "versus_ai":
                name = t("tetris.player.you") if i == 0 else \
                    t("tetris.player.ai", level=t("tetris.ai.%d" % self.ai_level))
            else:
                name = t("common.player1") if i == 0 else t("common.player2")
            text = "%s   %d" % (name, self.wins[i])
            img = self._fit_render(text, self._f_value, col, L["next"].right - L["hold"].x)
            s.blit(img, img.get_rect(midbottom=(field.centerx, field.y - 6)))
            b = self.boards[i]
            line = t("tetris.vs_line", lines=b.lines, attack=b.attack_sent)
            img = self._fit_render(line, self._f_label, ui.TEXT_DIM,
                                   L["next"].right - L["hold"].x)
            s.blit(img, img.get_rect(midtop=(field.centerx, field.bottom + 6)))
        tm = self._f_label.render(fmt_time(self.elapsed, cents=False), True, ui.TEXT_DIM)
        gap_mid = (self.sides[0]["next"].right + self.sides[1]["hold"].x) // 2
        s.blit(tm, tm.get_rect(midbottom=(gap_mid, self.sides[0]["field"].y - 6)))

    def _draw_particles(self, s):
        for x, y, _vx, _vy, life, maxl, col, size in self.particles:
            k = life / maxl
            sz = max(1, int(size * (0.5 + 0.5 * k)))
            s.fill(col, (int(x) - sz // 2, int(y) - sz // 2, sz, sz))

    def _draw_missiles(self, s):
        for m in self.missiles:
            p = m["t"] / m["dur"]
            for k in range(5):
                q = max(0.0, p - k * 0.05)
                e = _ease_out(q)
                x = m["x0"] + (m["x1"] - m["x0"]) * e
                y = m["y0"] + (m["y1"] - m["y0"]) * e - math.sin(q * math.pi) * 40
                rad = max(2, int((6 + min(6, m["n"])) * (1.0 - k * 0.18)))
                col = ui.mix(m["color"], (255, 255, 255), 0.5 if k == 0 else 0.0)
                pygame.draw.circle(s, col, (int(x), int(y)), rad)

    def _draw_callouts(self, s):
        for cl in self.callouts:
            field = self.sides[cl["side"]]["field"]
            p = cl["t"] / cl["dur"]
            alpha = int(255 * min(1.0, cl["t"] / 0.08) * min(1.0, (1.0 - p) / 0.3))
            total = sum(img.get_height() for img in cl["surfs"])
            y = field.y + int(field.h * 0.3) - total // 2 - int(p * field.h * 0.05)
            band = pygame.Rect(field.x + 2, y - 4, field.w - 4, total + 8)
            bg = self._caches.get(("callbg", band.w, band.h))
            if bg is None:
                bg = pygame.Surface(band.size, pygame.SRCALPHA)
                bg.fill((6, 8, 14, 120))
                self._caches[("callbg", band.w, band.h)] = bg
            bg.set_alpha(alpha)
            s.blit(bg, band.topleft)
            for img in cl["surfs"]:
                img.set_alpha(alpha)
                s.blit(img, img.get_rect(midtop=(field.centerx, y)))
                y += img.get_height()

    def _center_of_play(self):
        if self.versus:
            return self.width // 2, self.sides[0]["field"].centery
        return self.sides[0]["field"].center

    def _draw_countdown(self, s):
        cx, cy = self._center_of_play()
        p = 1.0 - self.count_t / COUNT_TIME
        text = t("tetris.ready")
        img = self._big.render(text, True, ui.TEXT)
        k = 1.0 + 0.06 * math.sin(p * math.tau * 2)
        w = int(img.get_width() * k)
        h = int(img.get_height() * k)
        img = pygame.transform.smoothscale(img.convert_alpha(), (max(1, w), max(1, h)))
        self._badge(s, img, cx, cy)

    def _draw_go(self, s):
        cx, cy = self._center_of_play()
        p = 1.0 - self.go_t / GO_TIME
        img = self._huge.render(t("tetris.go"), True, ui.GREEN)
        k = 0.8 + 0.5 * _ease_out(p)
        img = pygame.transform.smoothscale(img.convert_alpha(),
                                           (max(1, int(img.get_width() * k)),
                                            max(1, int(img.get_height() * k))))
        img.set_alpha(int(255 * min(1.0, (1.0 - p) * 2.5)))
        s.blit(img, img.get_rect(center=(cx, cy)))

    def _badge(self, s, img, cx, cy, alpha=225):
        box = img.get_rect(center=(cx, cy)).inflate(30, 14)
        self._panel(s, box, alpha)
        s.blit(img, img.get_rect(center=box.center))

    def _draw_finish_marks(self, s):
        """K.O.-Stempel, Sieg-Schriftzug, "Zeit um"/"40 Zeilen"."""
        res = self.result
        if res is None:
            return
        ft = self.finish_t if self.state == FINISH else 9.0
        if res["reason"] == "ko":
            for i, f in enumerate(self.fx):
                field = self.sides[i]["field"]
                if f.ko_t >= 0:
                    self._stamp(s, t("tetris.res.ko"), ui.RED, field, ft - 0.35)
                elif res["winner"] == i:
                    self._stamp(s, t("tetris.res.win"), ui.GOLD, field, ft - 0.7)
        elif res["reason"] in ("sprint", "ultra") and self.state == FINISH:
            field = self.sides[0]["field"]
            text = (t("tetris.res.sprint", n=SPRINT_LINES) if res["reason"] == "sprint"
                    else t("tetris.res.time_up"))
            self._stamp(s, text, ui.GOLD, field, ft - 0.1)

    def _stamp(self, s, text, color, field, tt):
        if tt < 0:
            return
        k = 1.0 + 1.2 * (1.0 - _ease_out(min(1.0, tt / 0.25)))
        img = self._fit_render(text, self._f_hero, color, int(field.w * 0.9))
        w = int(img.get_width() * k)
        h = int(img.get_height() * k)
        img = pygame.transform.smoothscale(img.convert_alpha(), (max(1, w), max(1, h)))
        img.set_alpha(int(255 * min(1.0, tt / 0.15)))
        box = img.get_rect(center=field.center).inflate(20, 10)
        if tt >= 0.25:
            self._panel(s, box, 215)
        s.blit(img, img.get_rect(center=field.center))

    # ----- Ergebnis ---------------------------------------------------------
    def _result_content(self):
        """(titel, farbe, zeilen[(label, wert, farbe)], hervorhebung)."""
        res = self.result
        st = res["stats"]
        if not self.versus:
            s0 = st[0]
            rows = []
            badge = None
            if res["reason"] == "sprint":
                title, color = t("tetris.res.sprint", n=SPRINT_LINES), ui.GOLD
                rows.append((t("tetris.stat.time"), fmt_time(res["time"]), ui.TEXT))
                if res["new_best"]:
                    badge = t("tetris.res.new_best_time")
                elif res["prev_best"] is not None:
                    rows.append((t("tetris.stat.best"), fmt_time(res["prev_best"]),
                                 ui.TEXT_DIM))
            elif res["reason"] == "ultra":
                title, color = t("tetris.res.time_up"), ui.GOLD
                rows.append((t("tetris.stat.score"), str(s0["score"]), ui.TEXT))
                if res["new_best"]:
                    badge = t("tetris.res.new_best")
                elif res["prev_best"]:
                    rows.append((t("tetris.stat.best"), str(res["prev_best"]),
                                 ui.TEXT_DIM))
            else:
                title, color = t("common.game_over"), ui.RED
                if self.variant != "sprint":
                    rows.append((t("tetris.stat.score"), str(s0["score"]), ui.TEXT))
                rows.append((t("tetris.stat.time"), fmt_time(res["time"]), ui.TEXT))
            rows.append((t("tetris.stat.lines"), str(s0["lines"]), ui.TEXT))
            if self.variant == "marathon":
                rows.append((t("tetris.stat.level"), str(s0["level"]), ui.TEXT))
            rows.append((t("tetris.stat.pps"), "%.2f" % s0["pps"], ui.TEXT))
            return title, color, rows, badge, None
        w = res["winner"]
        if w is None:
            title, color = t("common.draw"), ui.TEXT_DIM
        elif self.mode == "versus_ai":
            title = t("tetris.res.you_win") if w == 0 else t("tetris.res.ai_wins")
            color = COL_P1 if w == 0 else COL_P2
        else:
            title = t("common.player_wins", n=w + 1)
            color = COL_P1 if w == 0 else COL_P2
        table = [(t("tetris.stat.lines"), str(st[0]["lines"]), str(st[1]["lines"])),
                 (t("tetris.stat.attack"), str(st[0]["attack"]), str(st[1]["attack"])),
                 (t("tetris.stat.pps"), "%.2f" % st[0]["pps"], "%.2f" % st[1]["pps"]),
                 (t("tetris.stat.tetrises"), str(st[0]["tetrises"]),
                  str(st[1]["tetrises"])),
                 (t("tetris.stat.tspins"), str(st[0]["tspins"]), str(st[1]["tspins"]))]
        return title, color, [], "%d : %d" % tuple(self.wins), table

    def _results_layout(self):
        """Panel und Knöpfe des Ergebnis-Screens (auch für den Audit)."""
        title, color, rows, badge, table = self._result_content()
        W, H = self.width, self.height
        title_img = self._fit_render(title, self._big, color, W - 70)
        row_h = self._small.get_height() + 4
        btn_h = max(28, min(42, H // 14))
        n_rows = len(rows) if table is None else len(table) + 1
        h = (18 + title_img.get_height() + 8
             + (self._small_b.get_height() + 6 if badge else 0)
             + n_rows * row_h + 12 + btn_h + 8 + self._tiny.get_height() + 14)
        pw = min(W - 30, max(300, int(W * 0.46)))
        reserve = 46 if self.show_highscore_banner else 0
        cy = (H - reserve) // 2
        panel = pygame.Rect(0, 0, pw, h)
        panel.center = (W // 2, cy)
        if panel.top < 6:
            panel.top = 6
        bw = (pw - 3 * 14) // 2
        by = panel.bottom - 14 - self._tiny.get_height() - 8 - btn_h
        again = pygame.Rect(panel.x + 14, by, bw, btn_h)
        setup = pygame.Rect(again.right + 14, by, bw, btn_h)
        return dict(panel=panel, title=title_img, rows=rows, badge=badge, table=table,
                    again=again, setup=setup, row_h=row_h)

    def _draw_results(self, s):
        s.blit(self._dim(140), (0, 0))
        lay = self._results_layout()
        panel = lay["panel"]
        ui.draw_panel(s, panel)
        pygame.draw.rect(s, self.accent, (panel.x + 14, panel.y, panel.w - 28, 2))
        cx = panel.centerx
        y = panel.y + 18
        s.blit(lay["title"], lay["title"].get_rect(midtop=(cx, y)))
        y += lay["title"].get_height() + 8
        if lay["badge"]:
            col = ui.mix(ui.GOLD, (255, 255, 255), ui.pulse(4.0, 0.0, 0.5))
            img = self._fit_render(lay["badge"], self._small_b, col, panel.w - 24)
            s.blit(img, img.get_rect(midtop=(cx, y)))
            y += self._small_b.get_height() + 6
        left = panel.x + 24
        right = panel.right - 24
        if lay["table"] is None:
            for label, value, col in lay["rows"]:
                li = self._small.render(label, True, ui.TEXT_DIM)
                vi = self._small_b.render(value, True, col)
                s.blit(li, (left, y))
                s.blit(vi, vi.get_rect(topright=(right, y)))
                y += lay["row_h"]
        else:
            head = [(t("tetris.player.you") if self.mode == "versus_ai"
                     else "P1", COL_P1),
                    (t("common.ai") if self.mode == "versus_ai" else "P2", COL_P2)]
            colw = max([self._small_b.size(h)[0] for h, _c in head]
                       + [self._small_b.size(v)[0] for _l, a, b in lay["table"]
                          for v in (a, b)]) + 22
            c2 = panel.right - 24
            c1 = c2 - colw
            for (text, col), xx in zip(head, (c1, c2)):
                img = self._small_b.render(text, True, col)
                s.blit(img, img.get_rect(topright=(xx, y)))
            y += lay["row_h"]
            for label, v1, v2 in lay["table"]:
                li = self._fit_render(label, self._small, ui.TEXT_DIM,
                                      c1 - left - self._small_b.size(v1)[0] - 10)
                s.blit(li, (left, y))
                for v, xx in ((v1, c1), (v2, c2)):
                    img = self._small_b.render(v, True, ui.TEXT)
                    s.blit(img, img.get_rect(topright=(xx, y)))
                y += lay["row_h"]
        ready = time.monotonic() - self.over_at >= OVER_LOCK
        self.over_rects = {"again": lay["again"], "setup": lay["setup"]}
        again_key = "tetris.res.rematch" if self.versus else "tetris.res.again"
        for name, key in (("again", again_key), ("setup", "tetris.res.setup")):
            r = lay[name]
            ui.draw_button(s, r, t(key), self._small, selected=(name == "again" and ready),
                           accent=self.accent)
        hint = self._fit_render(t("tetris.res.hint"), self._tiny, ui.TEXT_FAINT,
                                panel.w - 20)
        s.blit(hint, hint.get_rect(midbottom=(cx, panel.bottom - 10)))

    # ----- Setup zeichnen -----------------------------------------------------
    def _draw_setup(self, s):
        W, H = self.width, self.height
        cx = W // 2
        # dezente fallende Steine im Hintergrund
        dc = max(10, H // 20)
        for kind, fx, fy, _sp, _r in self._deco:
            img = self._piece_img(kind, dc)
            ghost = self._caches.get(("deco", kind, dc))
            if ghost is None:
                ghost = img.copy()
                ghost.set_alpha(34)
                self._caches[("deco", kind, dc)] = ghost
            s.blit(ghost, (int(fx * (W - 3 * dc)), int(fy * H)))

        title = ui.grad_text(self._huge, "TETRIS", top=(250, 250, 255),
                             bottom=ui.mix(self.accent, (255, 255, 255), 0.2))
        s.blit(title, title.get_rect(center=(cx, self.setup_title_y)))
        sub = self._fit_render(t("tetris.subtitle." + self.mode), self._small,
                               ui.TEXT_DIM, W - 30)
        s.blit(sub, sub.get_rect(center=(cx, self.setup_sub_y)))

        items = self._setup_items()
        focus = items[max(0, min(len(items) - 1, self.setup_focus))]
        rects = self.setup_rects

        def lbl(text, x, y, w, active=False):
            img = self._fit_render(text, self._tiny, ui.TEXT if active else ui.TEXT_DIM, w)
            s.blit(img, img.get_rect(midbottom=(x, y)))

        if "choice" in rects:
            chs = rects["choice"]
            full = chs[0].union(chs[-1])
            if self.mode == "solo":
                names = [t("tetris.variant." + v) for v in VARIANTS]
                cur = VARIANTS.index(self.variant)
                lbl(t("tetris.lbl.variant"), full.centerx, chs[0].top - 3,
                    full.w, focus == "variant")
                desc = t("tetris.variant_desc." + self.variant)
                best = self._best_text()
            else:
                names = [t("tetris.ai.%d" % k) for k in range(3)]
                cur = self.ai_level
                lbl(t("tetris.lbl.ai"), full.centerx, chs[0].top - 3, full.w,
                    focus == "ai")
                desc = t("tetris.ai_desc.%d" % self.ai_level)
                wl = self.best["vs"][self.ai_level]
                best = t("tetris.vs_record", w=wl[0], l=wl[1])
            for k, r in enumerate(chs):
                self._opt_button(s, r, names[k], k == cur,
                                 focus in ("variant", "ai") and k == cur)
            img = self._fit_render(desc, self._tiny, ui.TEXT, full.w)
            s.blit(img, img.get_rect(midtop=(cx, self.setup_desc_y)))
            img = self._fit_render(best, self._tiny, ui.GOLD, full.w)
            s.blit(img, img.get_rect(midtop=(cx, self.setup_desc_y
                                             + self._tiny.get_height() + 3)))
        if "keys" in rects:
            r = rects["keys"]
            self._panel(s, r, 190)
            lines = [(t("tetris.keys_p1"), COL_P1), (t("tetris.keys_p2"), COL_P2)]
            yy = r.y + 9
            for text, col in lines:
                img = self._fit_render(text, self._tiny, col, r.w - 16)
                s.blit(img, img.get_rect(midtop=(r.centerx, yy)))
                yy += self._tiny.get_height() + 5

        opt_names = [n for n in ("level", "ghost", "das", "arr") if n in rects]
        for name in opt_names:
            r = rects[name]
            active = focus == name
            label = t("tetris.lbl." + name)
            lbl(label, r.centerx, r.top - 3, r.w + 6, active)
            if name == "ghost":
                val = t("common.on") if self.ghost else t("common.off")
                self._opt_button(s, r, val, self.ghost, active)
            else:
                enabled = not (name == "level" and self.variant != "marathon")
                if name == "level":
                    val = str(self.start_level) if enabled else "1"
                elif name == "das":
                    val = t("tetris.ms", n=self.das)
                else:
                    val = t("tetris.ms", n=self.arr)
                self._stepper(s, r, val, active, enabled)
        help_key = {"level": "tetris.help.level", "ghost": "tetris.help.ghost",
                    "das": "tetris.help.das", "arr": "tetris.help.arr"}.get(focus)
        if help_key is None:
            help_key = "tetris.help.das"
        full = rects[opt_names[0]].union(rects[opt_names[-1]])
        img = self._fit_render(t(help_key), self._tiny, ui.TEXT_FAINT, full.w)
        s.blit(img, img.get_rect(midtop=(cx, self.setup_help_y)))

        start = rects["start"]
        if focus == "start":
            glow = ui.mix(self.accent, (255, 255, 255), ui.pulse(3.0, 0.0, 0.5))
            pygame.draw.rect(s, glow, start.inflate(8, 8), 2, border_radius=12)
        ui.draw_button(s, start, t("common.start"), self._small_b, selected=True,
                       accent=self.accent)

        y1, y2 = self.setup_footer_y
        img = self._fit_render(t("tetris.setup_hint"), self._tiny, ui.TEXT_FAINT, W - 20)
        s.blit(img, img.get_rect(midtop=(cx, y1)))
        if self.mode != "multi":
            img = self._fit_render(t("tetris.keys_solo"), self._tiny,
                                   ui.mix(self.accent, ui.TEXT, 0.45), W - 20)
            s.blit(img, img.get_rect(midtop=(cx, y2)))

    def _best_text(self):
        if self.variant == "sprint":
            if self.best["sprint"] is None:
                return t("tetris.best_none")
            return t("tetris.best_time", time=fmt_time(self.best["sprint"]))
        if self.variant == "ultra":
            if not self.best["ultra"]:
                return t("tetris.best_none")
            return t("tetris.best_score", score=self.best["ultra"])
        import highscore
        hs = highscore.load_highscores().get(self.highscore_key, 0)
        return t("tetris.best_score", score=hs) if hs else t("tetris.best_none")

    def _opt_button(self, s, r, text, on, focus):
        pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, r, border_radius=8)
        border = self.accent if (on or focus) else ui.BORDER
        pygame.draw.rect(s, border, r, 2 if focus else 1, border_radius=8)
        if focus:
            glow = ui.mix(self.accent, (255, 255, 255), ui.pulse(3.0, 0.0, 0.4))
            pygame.draw.rect(s, glow, r.inflate(4, 4), 1, border_radius=10)
        img = self._fit_render(text, self._small_b if on else self._small,
                               ui.TEXT if on else ui.TEXT_DIM, r.w - 10)
        s.blit(img, img.get_rect(center=r.center))

    def _stepper(self, s, r, text, focus, enabled=True):
        pygame.draw.rect(s, ui.BTN, r, border_radius=8)
        pygame.draw.rect(s, self.accent if focus else ui.BORDER, r,
                         2 if focus else 1, border_radius=8)
        col = ui.TEXT if enabled else ui.TEXT_FAINT
        aw = max(3, r.h // 8)
        arrow = self.accent if (focus and enabled) else ui.TEXT_DIM
        for sx in (-1, 1):
            ax = r.centerx + sx * (r.w // 2 - aw - 5)
            tip = ax + sx * aw // 2
            back = ax - sx * aw // 2
            pygame.draw.polygon(s, arrow, [(tip, r.centery), (back, r.centery - aw),
                                           (back, r.centery + aw)])
        img = self._fit_render(text, self._small_b if focus else self._small, col,
                               r.w - 4 * aw - 22)
        s.blit(img, img.get_rect(center=r.center))
