# -*- coding: utf-8 -*-
"""
battleship.py
=============
Battleship / Schiffe versenken - gegen die KI oder zu zweit am selben Rechner.

- 10x10-Bretter, Flotte aus Flugzeugträger (5), Schlachtschiff (4), Kreuzer (3),
  U-Boot (3) und Zerstörer (2). Regeln und KI stecken in battleship_core.py.
- Setup: KI-Stärke (Leicht/Mittel/Schwer) und drei Regel-Schalter, alle
  sofort gespeichert: Schiffe dürfen sich berühren · Salven-Modus (Schüsse
  pro Zug = eigene Schiffe) · nach einem Treffer nochmal schießen.
- Aufstellen: Schiffe per Drag & Drop aus dem Dock aufs Brett ziehen (oder
  anklicken und wieder ablegen), R/Rechtsklick dreht, "Zufällig" und "Alle
  entfernen" als Knöpfe; die Vorschau zeigt grün/rot, ob die Lage erlaubt ist.
  Mit der Tastatur: Pfeile bewegen, Leertaste nimmt/legt, Enter = los.
- Gefecht: links das große Zielbrett mit Radar-Sweep, rechts das eigene Brett
  und die Flottenübersicht beider Seiten. Schüsse fliegen als Granate ein;
  Wasser spritzt, Treffer brennen und qualmen, versenkte Schiffe werden mit
  ihrem Umriss enthüllt.
- Einzelspieler: Punkte (Highscore) = Siege gegen die KI in einer Sitzung
  (connect4-Konvention). Mehrspieler: Übergabe-Bildschirm vor jedem Zug, der
  beide Flotten verdeckt, bis der nächste Spieler bestätigt; keine Wertung.

Steuerung: Maus oder Pfeile/WASD + Leertaste/Enter. Nach Rundenende:
Enter = neue Runde, S = Setup.
"""

import math
import random

import pygame

import settings as settings_mod
import ui
from game_base import Game, InputEvent, LocalizedName
from i18n import t

from . import battleship_core as core

N = core.N
FLEET = core.FLEET

# Identitätsfarben des Spiels (bewusst fest, unabhängig vom Theme):
# tiefblaues Meer, graue Kriegsschiffe, grünes Radar.
COL_SEA_TOP = (22, 70, 116)
COL_SEA_BOT = (10, 38, 72)
COL_RADAR_TOP = (14, 54, 78)
COL_RADAR_BOT = (6, 28, 46)
COL_GRID = (70, 128, 176)
COL_GRID_RADAR = (48, 128, 120)
COL_EDGE = (8, 22, 40)
COL_WAVE = (150, 205, 240)
COL_RING = (90, 200, 160)
COL_HULL = (118, 128, 144)
COL_HULL_DARK = (44, 50, 62)
COL_DECK = (150, 160, 174)
COL_BRIDGE = (196, 202, 212)
COL_TURRET = (82, 90, 104)
COL_SUB = (70, 82, 96)
COL_FLAME = (255, 120, 40)
COL_FLAME_IN = (255, 220, 110)
COL_SCORCH = (40, 18, 14)
COL_MISS = (215, 234, 248)
COL_VALID = (80, 220, 120)
COL_INVALID = (240, 70, 70)
COL_AIM = (255, 214, 110)
COL_AI_AIM = (255, 96, 80)
COL_P1 = (120, 200, 255)       # Spieler 1 / Du
COL_P2 = (255, 150, 110)       # Spieler 2 / KI

DIFFS = ("easy", "medium", "hard")
RULES = ("touch", "salvo", "extra_shot")
RULE_TEXT = {"touch": "bs.rule.touch", "salvo": "bs.rule.salvo",
             "extra_shot": "bs.rule.extra"}
BUTTONS = ("rotate", "random", "clear", "ready")
LETTERS = "ABCDEFGHIJ"

SETUP, PLACE, HANDOVER, PLAY, OVER = "setup", "place", "handover", "play", "over"

MAX_PARTICLES = 420


def _rgba(color, alpha):
    """Palette-Farbe (RGB) mit einem Alpha-Wert zu RGBA kombinieren."""
    return (color[0], color[1], color[2], alpha)


def _mix(c1, c2, f):
    f = max(0.0, min(1.0, f))
    return (int(c1[0] + (c2[0] - c1[0]) * f), int(c1[1] + (c2[1] - c1[1]) * f),
            int(c1[2] + (c2[2] - c1[2]) * f))


def _smooth(x):
    x = max(0.0, min(1.0, x))
    return x * x * (3 - 2 * x)


class BattleshipGame(Game):
    name = LocalizedName("Battleship", de="Schiffe versenken")
    highscore_key = "battleship"
    supports_multiplayer = True
    wants_right_click = True

    @property
    def show_highscore_banner(self):
        # Der 2-Spieler-Modus wird nicht gewertet (score bleibt 0).
        return not self.multiplayer

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        bs = self.settings.get("battleship", {}) if isinstance(self.settings, dict) else {}
        defaults = settings_mod.DEFAULTS.get("battleship", {})
        try:
            self.diff = max(0, min(2, int(bs.get("difficulty", 1))))
        except (TypeError, ValueError):
            self.diff = 1
        self.rules = {k: bool(bs.get(k, defaults.get(k, False))) for k in RULES}

        self._cache = {}
        self._make_fonts()
        self.wins = [0, 0]
        self.starter = 0
        self.layouts = [None, None]      # letzte Aufstellung je Spieler
        self.setup_sel = 0
        self.mouse = (-1, -1)
        self._last_draw = None
        self._build_setup_layout()
        self._new_round()
        self.state = SETUP

    def _make_fonts(self):
        """Theme-Schriften, Größen aus der Fensterhöhe abgeleitet."""
        h = self.height
        self._small = ui.font(max(13, min(22, h // 30)))
        self._tiny = ui.font(max(11, min(18, h // 38)))
        self._huge = ui.font(max(26, h // 11), bold=True)
        self._bold = ui.font(max(15, min(30, h // 24)), bold=True)

    def on_surface_changed(self):
        self._cache = {}
        self.particles = []
        self._make_fonts()
        self._build_setup_layout()
        self._layout()

    def _new_round(self):
        self.seas = [core.Sea(), core.Sea()]      # seas[p] = Brett von Spieler p
        self.stats = [dict(shots=0, hits=0, turns=0) for _ in range(2)]
        self.turn = self.starter
        self.shots_left = 0
        self.turn_report = dict(shots=0, hits=0)
        self.place_player = 0
        self.held = None          # gehaltenes Schiff beim Aufstellen
        self.place_horiz = True
        self.kcursor = [4, 4]
        self.aim = [[4, 4], [4, 4]]
        self.kb_mode = False
        self.flight = None        # fliegende Granate
        self.wait = 0.0
        self.after = None         # "next" | "end" | "win" nach der Wartezeit
        self.ai = core.ShotAI(self.diff, self.rules["touch"])
        self.ai_target = None
        self.ai_timer = 0.0
        self.ai_think = 1.0
        self.ai_from = (4.5, 4.5)
        self.ai_pos = (4.5, 4.5)
        self.handover = None
        self.reveals = []
        self.shake_t0 = -9.0
        self.shake_amp = 0.0
        self.winner = None
        self.msg = None
        self.msg_col = ui.TEXT
        self.msg_t = 0.0
        self.over_t0 = 0.0
        self.particles = []
        self._layout()

    def _now(self):
        """Uhr für reine Zeichen-Animationen (läuft auch nach Game Over)."""
        return pygame.time.get_ticks() / 1000.0

    # ===================================================== Layout
    def _layout(self):
        w, h = self.width, self.height
        self.hud_h = max(40, int(h * 0.085))
        m = max(8, w // 64)
        gap = max(12, w // 48)
        self.status_h = self._small.get_height() + 12
        top = self.hud_h + m
        cb = int((h - top - self.status_h - m) / 10.6)
        cb = min(cb, int((w - 2 * m - gap) * 0.63 / 10.6))
        self.cb = max(10, cb)
        self.lab = max(12, int(self.cb * 0.6))
        colw = w - 2 * m - gap - (self.lab + 10 * self.cb)
        self.cs = max(8, int(min(self.cb * 0.62, colw / 10)))
        total = self.lab + 10 * self.cb + gap + max(10 * self.cs, colw)
        x0 = max(m, (w - total) // 2)
        self.bx = x0 + self.lab
        self.by = top + self.lab
        self.col_x = self.bx + 10 * self.cb + gap
        self.col_w = w - m - self.col_x
        self.sx = self.col_x + (self.col_w - 10 * self.cs) // 2
        self.sy = self.by
        board_bottom = self.by + 10 * self.cb
        self.status_y = board_bottom + self.status_h // 2 + 2
        self._label_font = ui.font(max(10, min(24, int(self.cb * 0.42))), bold=True)

        # Gefecht: Beschriftung unter dem eigenen Brett, darunter die Flotten
        cap_y = self.sy + 10 * self.cs + 4
        self.caption_y = cap_y + self._tiny.get_height() // 2
        fy = cap_y + self._tiny.get_height() + max(6, h // 80)
        fh = max(40, (board_bottom + self.status_h - 6) - fy)
        half = (self.col_w - 10) // 2
        self.fleet_rects = [pygame.Rect(self.col_x, fy, half, fh),
                            pygame.Rect(self.col_x + half + 10, fy, half, fh)]

        # Aufstellen: Titel, Dock mit den 5 Schiffen, Knöpfe
        title_h = self._small.get_height() + 8
        self.btn_h = max(22, min(40, int(h * 0.058)))
        bgap = max(4, self.btn_h // 6)
        ready_h = self.btn_h + max(4, self.btn_h // 4)
        buttons_h = 3 * self.btn_h + 3 * bgap + ready_h
        dock_top = self.by - self.lab + title_h
        dock_bottom = board_bottom - buttons_h - bgap
        row_h = max(12, (dock_bottom - dock_top) // len(FLEET))
        self.dock_title_y = self.by - self.lab + title_h // 2
        self.dock_rects = [pygame.Rect(self.col_x, dock_top + i * row_h, self.col_w, row_h)
                           for i in range(len(FLEET))]
        # Silhouetten schmal genug, dass daneben der Schiffsname Platz hat
        self.dock_cell = max(6, int(min(self.cb * 0.7, row_h * 0.62,
                                        (self.col_w - 30) / 9.5)))
        y = dock_top + len(FLEET) * row_h + bgap
        self.btn_rects = {}
        for key in ("rotate", "random", "clear"):
            self.btn_rects[key] = pygame.Rect(self.col_x, y, self.col_w, self.btn_h)
            y += self.btn_h + bgap
        self.btn_rects["ready"] = pygame.Rect(self.col_x, y, self.col_w, ready_h)

    def _layout_check(self):
        if not hasattr(self, "cb"):
            self._layout()

    def _viewer(self):
        """Wessen Sicht zeigt das Gefecht? (Einzelspieler immer Spieler 0)."""
        if not self.multiplayer:
            return 0
        if self.state == OVER and self.winner is not None:
            return self.winner
        return self.turn

    def _board_geom(self, side):
        """(x, y, cell) des Bretts von Spieler 'side' in der aktuellen Ansicht."""
        v = self._viewer()
        if self.state == PLACE:
            return self.bx, self.by, self.cb
        if side == v:
            return self.sx, self.sy, self.cs
        return self.bx, self.by, self.cb

    @staticmethod
    def _cell_at(pos, x, y, cell):
        if pos is None:
            return None
        c = (pos[0] - x) // cell
        r = (pos[1] - y) // cell
        if 0 <= r < N and 0 <= c < N:
            return int(r), int(c)
        return None

    # ===================================================== Setup-Screen
    def _setup_items(self):
        return ([] if self.multiplayer else ["diff"]) + list(RULES) + ["start"]

    def _build_setup_layout(self):
        w, h = self.width, self.height
        cx = w // 2
        bw = min(max(360, int(w * 0.7)), w - 40)
        th = self._tiny.get_height()
        bh = max(26, min(46, int(h * 0.075)))
        gap = max(5, bh // 6)
        lab = th + 6
        block = (0 if self.multiplayer else lab + bh + 2 * gap) \
            + lab + 3 * bh + 2 * gap + 3 * gap + bh + 6
        top = int(h * 0.25)
        bottom = h - 26
        y = top + max(0, (bottom - top - block) // 2)
        y = min(y, top + int(h * 0.05))
        self.diff_rects = []
        if not self.multiplayer:
            y += lab
            cw = (bw - 2 * gap) / 3.0
            self.diff_rects = [pygame.Rect(int(cx - bw / 2 + i * (cw + gap)), y, int(cw), bh)
                               for i in range(3)]
            self.diff_label_y = y - 4
            y += bh + 2 * gap
        y += lab
        self.rules_label_y = y - 4
        self.rule_rects = [pygame.Rect(cx - bw // 2, y + i * (bh + gap), bw, bh)
                           for i in range(3)]
        y += 3 * bh + 2 * gap + 3 * gap
        sw = max(190, int(bw * 0.42))
        self.start_rect = pygame.Rect(cx - sw // 2, y, sw, bh + 6)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("battleship", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _set_diff(self, d):
        self.diff = d % 3
        self._save_setting("difficulty", self.diff)
        self.play_sound("click")

    def _toggle_rule(self, key):
        self.rules[key] = not self.rules[key]
        self._save_setting(key, self.rules[key])
        self.play_sound("select")

    def _handle_setup(self, event):
        items = self._setup_items()
        self.setup_sel = max(0, min(len(items) - 1, self.setup_sel))
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            cur = items[self.setup_sel]
            if k in ("1", "2", "3") and not self.multiplayer:
                self._set_diff(int(k) - 1)
            elif k == "Up" or self.is_action(k, "up"):
                self.setup_sel = (self.setup_sel - 1) % len(items)
                self.play_sound("move")
            elif k == "Down" or self.is_action(k, "down"):
                self.setup_sel = (self.setup_sel + 1) % len(items)
                self.play_sound("move")
            elif k in ("Left", "Right") or self.is_action(k, "left") \
                    or self.is_action(k, "right"):
                step = -1 if (k == "Left" or self.is_action(k, "left")) else 1
                if cur == "diff":
                    self._set_diff(self.diff + step)
                elif cur in RULES:
                    self._toggle_rule(cur)
            elif k in ("Return", "KP_Enter"):
                self._start_round()
            elif k == "space":
                if cur == "diff":
                    self._set_diff(self.diff + 1)
                elif cur in RULES:
                    self._toggle_rule(cur)
                else:
                    self._start_round()
        elif event.kind == InputEvent.MOUSEMOVE:
            for i, it in enumerate(items):
                if any(r.collidepoint(event.pos) for r in self._setup_item_rects(it)):
                    self.setup_sel = i
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            for i, rc in enumerate(self.diff_rects):
                if rc.collidepoint(event.pos):
                    self.setup_sel = 0
                    self._set_diff(i)
                    return
            for i, rc in enumerate(self.rule_rects):
                if rc.collidepoint(event.pos):
                    self.setup_sel = items.index(RULES[i])
                    self._toggle_rule(RULES[i])
                    return
            if self.start_rect.collidepoint(event.pos):
                self._start_round()

    def _setup_item_rects(self, item):
        if item == "diff":
            return self.diff_rects
        if item in RULES:
            return [self.rule_rects[RULES.index(item)]]
        return [self.start_rect]

    # ===================================================== Rundenablauf
    def _start_round(self):
        self.game_over = False
        self._new_round()
        for p in (0, 1):
            if self.multiplayer or p == 0:
                self._restore_layout(p)
        self.place_player = 0
        if self.multiplayer:
            self._handover(0, "place")
        else:
            self.state = PLACE
        self.play_sound("click")

    def _restore_layout(self, p):
        """Die letzte Aufstellung als Vorschlag wieder auslegen (soweit erlaubt)."""
        sea = self.seas[p]
        for (idx, r, c, horiz) in self.layouts[p] or []:
            if sea.can_place(idx, r, c, horiz, self.rules["touch"]):
                sea.place(idx, r, c, horiz)

    def _restart(self):
        self.starter = 1 - self.starter
        self._start_round()

    def _handover(self, player, kind, report=None):
        self.handover = dict(to=player, kind=kind, report=report, t=0.0)
        self.held = None
        self.flight = None
        self.state = HANDOVER

    def _confirm_handover(self):
        ho = self.handover
        if ho is None or ho["t"] < 0.45:
            return
        self.handover = None
        self.play_sound("select")
        if ho["kind"] == "place":
            self.place_player = ho["to"]
            self.state = PLACE
        else:
            self._begin_turn(ho["to"])

    def _ready(self):
        sea = self.seas[self.place_player]
        if self.held is not None:
            self._drop_held()
        if not sea.complete():
            self._say(t("bs.place_all"), ui.GOLD, 1.6)
            self.play_sound("hit")
            return
        self.layouts[self.place_player] = [(s.idx, s.r, s.c, s.horiz) for s in sea.ships]
        self.play_sound("level")
        if not self.multiplayer:
            self.seas[1].randomize(self.rules["touch"])
            self._begin_turn(self.starter)
        elif self.place_player == 0:
            self._handover(1, "place")
        else:
            self._handover(self.starter, "fire")

    def _is_ai(self, p):
        return not self.multiplayer and p == 1

    def _begin_turn(self, p):
        self.turn = p
        self.state = PLAY
        self.shots_left = self.seas[p].ships_left() if self.rules["salvo"] else 1
        self.stats[p]["turns"] += 1
        self.turn_report = dict(shots=0, hits=0)
        self.flight = None
        self.after = None
        self.wait = 0.0
        self.ai_target = None
        if not self._is_ai(p):
            self.tone_turn()

    def tone_turn(self):
        """Leises Radar-Ping zu Beginn eines eigenen Zugs."""
        try:
            import audio
            audio.tone(1320, 0.07, self.settings, "sine", 0.18)
        except Exception:
            pass

    def _say(self, text, color, secs=1.4):
        self.msg = text
        self.msg_col = color
        self.msg_t = secs

    def _fire(self, r, c):
        """Feuert für den Spieler am Zug auf (r, c). True = Schuss unterwegs."""
        if self.state != PLAY or self.flight is not None:
            return False
        if self.after not in (None, "next") or self.shots_left <= 0:
            return False
        target = self.seas[1 - self.turn]
        if target.shots[r][c] != core.UNKNOWN:
            if not self._is_ai(self.turn):
                self._say(t("bs.already"), ui.TEXT_DIM, 1.2)
                self.play_sound("click")
            return False
        self.after = None
        self.wait = 0.0
        self.shots_left -= 1
        self.flight = dict(r=r, c=c, t=0.0, shooter=self.turn,
                           dur=0.46 if self._is_ai(self.turn) else 0.36)
        self.play_sound("shoot")
        return True

    def _impact(self):
        f = self.flight
        self.flight = None
        shooter = f["shooter"]
        side = 1 - shooter
        target = self.seas[side]
        res, ship = target.fire(f["r"], f["c"])
        if res is None:
            self.after, self.wait = "next", 0.1
            return
        st = self.stats[shooter]
        st["shots"] += 1
        self.turn_report["shots"] += 1
        x, y, cell = self._board_geom(side)
        px = x + f["c"] * cell + cell / 2
        py = y + f["r"] * cell + cell / 2
        mine = (side == self._viewer())      # traf es MEINE Flotte?
        if res == "miss":
            self._fx_splash(px, py, cell)
            self._sfx_splash()
            self._say(t("bs.miss"), COL_MISS, 1.0)
            pause = 0.55
        else:
            st["hits"] += 1
            self.turn_report["hits"] += 1
            self._fx_explosion(px, py, cell, big=(res == "sunk"))
            self.play_sound("explode")
            self.shake_t0 = self._now()
            self.shake_amp = max(2.0, cell * (0.16 if res == "sunk" else 0.09))
            if mine:
                self.rumble(260 if res == "sunk" else 140)
            if self.rules["extra_shot"]:
                self.shots_left += 1
            if res == "sunk":
                self.reveals.append(dict(side=side, ship=ship, t0=self._now()))
                for (rr, cc) in ship.cells:
                    self._fx_explosion(x + cc * cell + cell / 2, y + rr * cell + cell / 2,
                                       cell, big=False, quiet=True)
                self.play_sound("line")
                name = t("bs.ship." + core.SHIP_KEYS[ship.idx])
                self._say(t("bs.sunk", ship=name), COL_P2 if mine else ui.GOLD, 2.2)
                pause = 1.15
            else:
                key = "bs.hit_again" if self.rules["extra_shot"] else "bs.hit"
                self._say(t(key), COL_FLAME, 1.3)
                pause = 0.7
        if target.all_sunk():
            self.after, self.wait = "win", 1.7
        elif self.shots_left <= 0 or target.untouched() == 0:
            self.after, self.wait = "end", max(pause, 0.95)
        else:
            self.after, self.wait = "next", pause

    def _continue(self):
        after, self.after = self.after, None
        if after == "win":
            self._finish(self.turn)
        elif after == "end":
            nxt = 1 - self.turn
            if self.multiplayer:
                self._handover(nxt, "fire", report=dict(self.turn_report))
            else:
                self._begin_turn(nxt)

    def _finish(self, winner):
        self.winner = winner
        self.state = OVER
        self.over_t0 = self._now()
        self.wins[winner] += 1
        if not self.multiplayer:
            if winner == 0:
                self.score = self.wins[0]
                self.play_sound("win")
                self.report_result(True)
                if not any(s.sunk for s in self.seas[0].ships):
                    self.ach_event("bs_flawless")
                if self.diff == core.HARD:
                    self.ach_event("bs_hard")
            else:
                self.play_sound("gameover")
                self.report_result(False)
        else:
            self.play_sound("win")
        self.game_over = True     # main.py speichert den Score einmalig

    # ===================================================== Spiellogik
    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None
        if self.state == HANDOVER and self.handover is not None:
            self.handover["t"] += dt
            return
        if self.state != PLAY:
            return
        if self.flight is not None:
            self.flight["t"] += dt
            if self.flight["t"] >= self.flight["dur"]:
                self._impact()
            return
        if self.after is not None:
            self.wait -= dt
            if self.wait <= 0:
                self._continue()
            return
        if self._is_ai(self.turn):
            self._ai_step(dt)

    def _ai_step(self, dt):
        if self.ai_target is None:
            pick = self.ai.choose(self.seas[0])
            if pick is None:
                self.after, self.wait = "end", 0.2
                return
            self.ai_target = pick
            first = self.turn_report["shots"] == 0
            self.ai_think = (0.8 if first else 0.42) + random.uniform(0.0, 0.25)
            self.ai_timer = self.ai_think
            self.ai_from = self.ai_pos
        self.ai_timer -= dt
        k = _smooth(1.0 - max(0.0, self.ai_timer) / self.ai_think)
        r, c = self.ai_target
        self.ai_pos = (self.ai_from[0] + (c + 0.5 - self.ai_from[0]) * k,
                       self.ai_from[1] + (r + 0.5 - self.ai_from[1]) * k)
        if self.ai_timer <= 0:
            self.ai_pos = (c + 0.5, r + 0.5)
            self.ai_target = None
            self._fire(r, c)

    # ===================================================== Eingabe
    def handle_event(self, event):
        if event.kind in (InputEvent.MOUSEMOVE, InputEvent.MOUSEDOWN, InputEvent.MOUSEUP):
            if event.pos is not None:
                self.mouse = event.pos
        if self.state == SETUP:
            self._handle_setup(event)
        elif self.state == HANDOVER:
            if (event.kind == InputEvent.KEYDOWN
                    and event.key in ("Return", "KP_Enter", "space")) \
                    or (event.kind == InputEvent.MOUSEDOWN and event.button == 1):
                self._confirm_handover()
        elif self.state == PLACE:
            self._handle_place(event)
        elif self.state == PLAY:
            self._handle_play(event)
        elif self.state == OVER:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "KP_Enter", "space"):
                    self._restart()
                elif event.key in ("s", "S"):
                    self.game_over = False
                    self.state = SETUP
                    self.setup_sel = 0
                    self._build_setup_layout()
                    self.play_sound("click")
            elif (event.kind == InputEvent.MOUSEDOWN and event.button == 1
                  and self._now() - self.over_t0 > 0.8):
                self._restart()

    # ----- Gefecht -----------------------------------------------------
    def _handle_play(self, event):
        if self._is_ai(self.turn):
            return
        aim = self.aim[self.turn]
        if event.kind == InputEvent.MOUSEMOVE:
            rc = self._cell_at(event.pos, self.bx, self.by, self.cb)
            if rc:
                aim[0], aim[1] = rc
                self.kb_mode = False
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            rc = self._cell_at(event.pos, self.bx, self.by, self.cb)
            if rc:
                aim[0], aim[1] = rc
                self.kb_mode = False
                self._fire(*rc)
        elif event.kind == InputEvent.KEYDOWN:
            k = event.key
            moved = True
            if k == "Up" or self.is_action(k, "up"):
                aim[0] = (aim[0] - 1) % N
            elif k == "Down" or self.is_action(k, "down"):
                aim[0] = (aim[0] + 1) % N
            elif k == "Left" or self.is_action(k, "left"):
                aim[1] = (aim[1] - 1) % N
            elif k == "Right" or self.is_action(k, "right"):
                aim[1] = (aim[1] + 1) % N
            else:
                moved = False
            if moved:
                self.kb_mode = True
                self.play_sound("move")
            elif k in ("space", "Return", "KP_Enter") or self.is_action(k, "action"):
                self.kb_mode = True
                self._fire(aim[0], aim[1])

    # ----- Aufstellen ---------------------------------------------------
    def _place_sea(self):
        return self.seas[self.place_player]

    def _pick(self, idx, grab=0, via_mouse=True):
        """Nimmt Schiff 'idx' auf (vom Brett oder aus dem Dock)."""
        sea = self._place_sea()
        ship = sea.ship_by_idx(idx)
        origin = None
        horiz = self.place_horiz
        if ship is not None:
            origin = (ship.r, ship.c, ship.horiz)
            horiz = ship.horiz
            sea.remove(ship)
        self.held = dict(idx=idx, horiz=horiz, grab=max(0, min(FLEET[idx] - 1, grab)),
                         origin=origin, mouse=via_mouse, press=self.mouse, moved=False)
        self.play_sound("select")

    def _held_anchor(self):
        """(r, c) des ersten Felds der Vorschau oder None (Maus neben dem Brett)."""
        hd = self.held
        if hd is None:
            return None
        size = FLEET[hd["idx"]]
        if hd["mouse"]:
            rc = self._cell_at(self.mouse, self.bx, self.by, self.cb)
            if rc is None:
                return None
            r, c = rc
        else:
            r, c = self.kcursor
        if hd["horiz"]:
            c -= hd["grab"]
        else:
            r -= hd["grab"]
        if not hd["mouse"]:
            # Tastatur: Schiff immer ganz im Brett halten
            if hd["horiz"]:
                c = max(0, min(N - size, c))
            else:
                r = max(0, min(N - size, r))
        return r, c

    def _held_valid(self):
        anchor = self._held_anchor()
        if anchor is None:
            return False
        return self._place_sea().can_place(self.held["idx"], anchor[0], anchor[1],
                                           self.held["horiz"], self.rules["touch"])

    def _put_held(self):
        """Legt das gehaltene Schiff an der Vorschau-Position ab (falls erlaubt)."""
        anchor = self._held_anchor()
        if anchor is None:
            return False
        hd = self.held
        if not self._place_sea().can_place(hd["idx"], anchor[0], anchor[1], hd["horiz"],
                                           self.rules["touch"]):
            self._say(t("bs.place_invalid"), COL_INVALID, 1.2)
            self.play_sound("hit")
            return False
        self._place_sea().place(hd["idx"], anchor[0], anchor[1], hd["horiz"])
        self.held = None
        self.play_sound("lock")
        return True

    def _drop_held(self):
        """Bricht das Halten ab: zurück an den alten Platz oder ins Dock."""
        hd = self.held
        self.held = None
        if hd and hd["origin"] is not None:
            r, c, horiz = hd["origin"]
            sea = self._place_sea()
            if sea.can_place(hd["idx"], r, c, horiz, self.rules["touch"]):
                sea.place(hd["idx"], r, c, horiz)

    def _rotate_held(self):
        hd = self.held
        hd["horiz"] = not hd["horiz"]
        self.place_horiz = hd["horiz"]
        self.play_sound("rotate")

    def _rotate_placed(self, ship, pivot):
        """Dreht ein liegendes Schiff um das Feld 'pivot' (wenn es passt)."""
        sea = self._place_sea()
        k = (pivot[1] - ship.c) if ship.horiz else (pivot[0] - ship.r)
        horiz = not ship.horiz
        r, c = (pivot[0], pivot[1] - k) if horiz else (pivot[0] - k, pivot[1])
        tries = [(r, c)] + [((r, c + d) if horiz else (r + d, c))
                            for d in (-1, 1, -2, 2, -3, 3, -4, 4)]
        for (rr, cc) in tries:
            if sea.can_place(ship.idx, rr, cc, horiz, self.rules["touch"], ignore=ship):
                sea.place(ship.idx, rr, cc, horiz)
                self.play_sound("rotate")
                return True
        self._say(t("bs.place_invalid"), COL_INVALID, 1.2)
        self.play_sound("hit")
        return False

    def _press_button(self, key):
        sea = self._place_sea()
        if key == "rotate":
            if self.held is not None:
                self._rotate_held()
            else:
                rc = tuple(self.kcursor) if self.kb_mode else \
                    self._cell_at(self.mouse, self.bx, self.by, self.cb)
                ship = sea.ship_at(*rc) if rc else None
                if ship is not None:
                    self._rotate_placed(ship, rc)
                else:
                    self.place_horiz = not self.place_horiz
                    self.play_sound("rotate")
        elif key == "random":
            self.held = None
            sea.clear()
            sea.randomize(self.rules["touch"])
            self.play_sound("merge")
        elif key == "clear":
            self.held = None
            sea.clear()
            self.play_sound("click")
        elif key == "ready":
            self._ready()

    def _next_unplaced(self):
        sea = self._place_sea()
        for idx in range(len(FLEET)):
            if sea.ship_by_idx(idx) is None:
                return idx
        return None

    def _handle_place(self, event):
        sea = self._place_sea()
        if event.kind == InputEvent.MOUSEMOVE:
            hd = self.held
            if hd is not None:
                if not hd["mouse"] and self._cell_at(event.pos, self.bx, self.by, self.cb):
                    hd["mouse"] = True
                if hd["press"] is not None and (abs(event.pos[0] - hd["press"][0])
                                                + abs(event.pos[1] - hd["press"][1]) > 6):
                    hd["moved"] = True
            if self._cell_at(event.pos, self.bx, self.by, self.cb):
                self.kb_mode = False
            return
        if event.kind == InputEvent.MOUSEDOWN:
            pos = event.pos
            rc = self._cell_at(pos, self.bx, self.by, self.cb)
            if event.button == 3:
                if self.held is not None:
                    self._rotate_held()
                elif rc and sea.ship_at(*rc):
                    self._rotate_placed(sea.ship_at(*rc), rc)
                else:
                    self.place_horiz = not self.place_horiz
                    self.play_sound("rotate")
                return
            if event.button != 1:
                return
            self.kb_mode = False
            if self.held is not None:
                # Tragen per Klick: auf dem Brett ablegen, sonst zurück
                if rc:
                    self.held["mouse"] = True
                    self._put_held()
                    return
                for key, brc in self.btn_rects.items():
                    if brc.collidepoint(pos):
                        self._drop_held()
                        self._press_button(key)
                        return
                self._drop_held()
                self.play_sound("click")
                return
            for key, brc in self.btn_rects.items():
                if brc.collidepoint(pos):
                    self._press_button(key)
                    return
            if rc:
                ship = sea.ship_at(*rc)
                if ship is not None:
                    grab = (rc[1] - ship.c) if ship.horiz else (rc[0] - ship.r)
                    self._pick(ship.idx, grab)
                return
            for idx, drc in enumerate(self.dock_rects):
                if drc.collidepoint(pos):
                    grab = int((pos[0] - (drc.x + 8)) // max(1, self.dock_cell))
                    self._pick(idx, grab)
                    self.held["horiz"] = True if sea.ship_by_idx(idx) is None \
                        else self.held["horiz"]
                    return
            return
        if event.kind == InputEvent.MOUSEUP:
            hd = self.held
            if hd is None or not hd["moved"] or not hd["mouse"]:
                return
            if self._cell_at(event.pos, self.bx, self.by, self.cb) and self._held_valid():
                self._put_held()
            else:
                if self._cell_at(event.pos, self.bx, self.by, self.cb):
                    self._say(t("bs.place_invalid"), COL_INVALID, 1.2)
                    self.play_sound("hit")
                self._drop_held()
            return
        if event.kind != InputEvent.KEYDOWN:
            return
        k = event.key
        dr = dc = 0
        if k == "Up" or self.is_action(k, "up"):
            dr = -1
        elif k == "Down" or self.is_action(k, "down"):
            dr = 1
        elif k == "Left" or self.is_action(k, "left"):
            dc = -1
        elif k == "Right" or self.is_action(k, "right"):
            dc = 1
        if dr or dc:
            if not self.kb_mode and self.held is None:
                rc = self._cell_at(self.mouse, self.bx, self.by, self.cb)
                if rc:
                    self.kcursor = list(rc)
            self.kb_mode = True
            if self.held is not None and self.held["mouse"]:
                self.held["mouse"] = False
                rc = self._cell_at(self.mouse, self.bx, self.by, self.cb)
                if rc:
                    self.kcursor = list(rc)
            self.kcursor[0] = (self.kcursor[0] + dr) % N
            self.kcursor[1] = (self.kcursor[1] + dc) % N
            self.play_sound("move")
            return
        if k in ("r", "R") and self.key_is_free(k):
            self._press_button("rotate")
            return
        if k in ("x", "X") and self.key_is_free(k):
            self._press_button("random")
            return
        if k in ("c", "C", "Delete", "BackSpace") and (len(k) > 1 or self.key_is_free(k)):
            self._press_button("clear")
            return
        is_enter = k in ("Return", "KP_Enter")
        if k == "space" or is_enter or self.is_action(k, "action"):
            self.kb_mode = True
            if self.held is not None:
                self.held["mouse"] = False
                self._put_held()
                return
            ship = sea.ship_at(*self.kcursor)
            if is_enter and sea.complete():
                self._ready()
            elif ship is not None and not is_enter:
                grab = (self.kcursor[1] - ship.c) if ship.horiz else (self.kcursor[0] - ship.r)
                self._pick(ship.idx, grab, via_mouse=False)
            else:
                idx = self._next_unplaced()
                if idx is not None:
                    self._pick(idx, 0, via_mouse=False)
                elif ship is not None:
                    grab = (self.kcursor[1] - ship.c) if ship.horiz \
                        else (self.kcursor[0] - ship.r)
                    self._pick(ship.idx, grab, via_mouse=False)

    # ===================================================== Effekte
    def _spawn(self, **p):
        if len(self.particles) < MAX_PARTICLES:
            p.setdefault("age", 0.0)
            p.setdefault("g", 0.0)
            p.setdefault("vx", 0.0)
            p.setdefault("vy", 0.0)
            self.particles.append(p)

    def _fx_splash(self, x, y, cell):
        k = cell / 40.0
        self._spawn(kind="flash", x=x, y=y, life=0.16, r=cell * 0.45, col=(210, 235, 255))
        self._spawn(kind="ring", x=x, y=y, life=0.7, r0=cell * 0.12, r1=cell * 0.62)
        self._spawn(kind="ring", x=x, y=y, life=0.95, r0=cell * 0.05, r1=cell * 0.9)
        for _ in range(16):
            ang = random.uniform(math.pi * 1.05, math.pi * 1.95)
            sp = random.uniform(70, 230) * k
            self._spawn(kind="drop", x=x, y=y, vx=math.cos(ang) * sp,
                        vy=math.sin(ang) * sp * 1.4, g=620 * k,
                        life=random.uniform(0.45, 0.8), size=random.uniform(1.2, 2.8) * max(1, k))

    def _fx_explosion(self, x, y, cell, big=False, quiet=False):
        k = cell / 40.0
        n = 10 if quiet else (34 if big else 22)
        self._spawn(kind="flash", x=x, y=y, life=0.22 if not quiet else 0.16,
                    r=cell * (1.1 if big else 0.8), col=(255, 190, 90))
        for _ in range(n):
            ang = random.uniform(0, math.tau)
            sp = random.uniform(80, 360 if big else 280) * k
            self._spawn(kind="spark", x=x, y=y, vx=math.cos(ang) * sp,
                        vy=math.sin(ang) * sp - 60 * k, g=520 * k,
                        life=random.uniform(0.3, 0.7), size=random.uniform(1.5, 3.2) * max(1, k),
                        col=random.choice(((255, 230, 120), (255, 160, 60), (255, 110, 40))))
        for _ in range(3 if quiet else 6):
            self._spawn(kind="smoke", x=x + random.uniform(-0.2, 0.2) * cell,
                        y=y + random.uniform(-0.1, 0.2) * cell,
                        vx=random.uniform(-12, 12) * k, vy=random.uniform(-40, -18) * k,
                        life=random.uniform(0.9, 1.6), r0=cell * 0.16, r1=cell * 0.55)

    def _sfx_splash(self):
        try:
            import audio
            audio.tone(180, 0.22, self.settings, "noise", 0.32)
        except Exception:
            pass
        self.play_sound("bounce")

    def _glow(self, radius, color):
        radius = max(4, int(radius) // 3 * 3)
        key = ("glow", radius, color)
        surf = self._cache.get(key)
        if surf is None:
            surf = pygame.Surface((radius * 2, radius * 2), pygame.SRCALPHA)
            steps = 10
            for i in range(steps):
                f = i / steps
                pygame.draw.circle(surf, _rgba(color, int(210 * f * f)), (radius, radius),
                                   max(1, int(radius * (1 - f))))
            self._cache[key] = surf
        return surf

    def _puff(self, radius):
        radius = max(3, int(radius) // 2 * 2)
        key = ("puff", radius)
        surf = self._cache.get(key)
        if surf is None:
            surf = pygame.Surface((radius * 2, radius * 2), pygame.SRCALPHA)
            steps = 6
            for i in range(steps):
                f = i / steps
                pygame.draw.circle(surf, (70, 72, 78, int(40 + 120 * f)), (radius, radius),
                                   max(1, int(radius * (1 - f * 0.8))))
            self._cache[key] = surf
        return surf

    def _step_particles(self, s, dt):
        alive = []
        for p in self.particles:
            p["age"] += dt
            if p["age"] >= p["life"]:
                continue
            f = p["age"] / p["life"]
            p["vy"] += p["g"] * dt
            p["x"] += p["vx"] * dt
            p["y"] += p["vy"] * dt
            kind = p["kind"]
            x, y = int(p["x"]), int(p["y"])
            if kind == "spark":
                sz = max(1, int(p["size"] * (1 - f * 0.6)))
                s.fill(_mix(p["col"], (120, 30, 10), f), (x, y, sz, sz))
            elif kind == "drop":
                pygame.draw.circle(s, _mix((240, 250, 255), COL_SEA_TOP, f * 0.9), (x, y),
                                   max(1, int(p["size"])))
            elif kind == "ring":
                rr = p["r0"] + (p["r1"] - p["r0"]) * math.sqrt(f)
                pygame.draw.circle(s, _mix((220, 240, 255), COL_SEA_TOP, f), (x, y),
                                   max(2, int(rr)), max(1, int(3 * (1 - f)) + 1))
            elif kind == "smoke":
                rr = p["r0"] + (p["r1"] - p["r0"]) * f
                img = self._puff(rr)
                img.set_alpha(int(170 * (1 - f)))
                s.blit(img, (x - img.get_width() // 2, y - img.get_height() // 2))
                img.set_alpha(255)
            elif kind == "flash":
                img = self._glow(p["r"] * (0.6 + 0.6 * f), p["col"])
                img.set_alpha(int(255 * (1 - f)))
                s.blit(img, (x - img.get_width() // 2, y - img.get_height() // 2))
                img.set_alpha(255)
            alive.append(p)
        self.particles = alive

    # ===================================================== Sprites
    def _ship_sprite(self, idx, cell, horiz, style="normal"):
        key = ("ship", idx, cell, horiz, style)
        surf = self._cache.get(key)
        if surf is None:
            if style == "sunk":
                base = self._ship_sprite(idx, cell, True).copy()
                base.fill((150, 96, 84, 255), special_flags=pygame.BLEND_RGBA_MULT)
                size = FLEET[idx]
                for i in range(size):     # Brandspuren
                    cx = int((i + 0.5) * cell)
                    pygame.draw.circle(base, (24, 12, 10, 150),
                                       (cx + int(cell * 0.1 * ((i * 7) % 3 - 1)), cell // 2),
                                       max(2, int(cell * 0.2)))
                surf = base
            else:
                surf = self._render_ship(idx, cell)
            if not horiz:
                surf = pygame.transform.rotate(surf, -90)
            self._cache[key] = surf
        return surf

    def _render_ship(self, idx, cell):
        size = FLEET[idx]
        L, H = size * cell, cell
        surf = pygame.Surface((L, H), pygame.SRCALPHA)
        p = max(2, int(cell * 0.14))
        top, bot = p, H - p
        mid = H / 2.0
        hull_h = bot - top
        x0, x1 = p, L - max(1, p // 2)
        lw = max(1, cell // 18)
        if core.SHIP_KEYS[idx] == "submarine":
            body = pygame.Rect(x0, int(top + hull_h * 0.14), x1 - x0, max(3, int(hull_h * 0.72)))
            pygame.draw.ellipse(surf, COL_SUB, body)
            hi = body.inflate(-body.w * 0.16, -body.h * 0.55)
            hi.y = body.y + max(1, body.h // 6)
            pygame.draw.ellipse(surf, _mix(COL_SUB, (255, 255, 255), 0.22), hi)
            pygame.draw.ellipse(surf, COL_HULL_DARK, body, lw)
            tw = max(3, int(cell * 0.62))
            th = max(3, int(hull_h * 0.5))
            tower = pygame.Rect(int(L * 0.4 - tw / 2), int(mid - th / 2), tw, th)
            pygame.draw.rect(surf, COL_DECK, tower, border_radius=max(1, th // 2))
            pygame.draw.rect(surf, COL_HULL_DARK, tower, lw, border_radius=max(1, th // 2))
            pygame.draw.line(surf, COL_HULL_DARK, (tower.centerx, int(mid)),
                             (min(x1 - p, tower.right + int(cell * 0.35)), int(mid)), lw)
            return surf
        bow = min(cell * 0.95, L * 0.32)
        pts = [(x0 + p, top), (x1 - bow, top), (x1, mid), (x1 - bow, bot),
               (x0 + p, bot), (x0, bot - p * 0.7), (x0, top + p * 0.7)]
        pygame.draw.polygon(surf, COL_HULL, pts)
        # Deck (etwas heller, eingerückt)
        d = max(1, int(hull_h * 0.2))
        deck = [(x0 + p + d * 0.5, top + d), (x1 - bow + d * 0.2, top + d),
                (x1 - d * 1.6, mid), (x1 - bow + d * 0.2, bot - d),
                (x0 + p + d * 0.5, bot - d), (x0 + d, bot - p * 0.7 - d * 0.3),
                (x0 + d, top + p * 0.7 + d * 0.3)]
        pygame.draw.polygon(surf, COL_DECK, deck)
        pygame.draw.polygon(surf, COL_HULL_DARK, pts, lw)
        key = core.SHIP_KEYS[idx]
        if key == "carrier":
            # Flugdeck mit Mittellinie und Insel an der Seite
            dash = max(2, cell // 4)
            yy = int(mid)
            xx = int(x0 + p + d)
            while xx < x1 - bow:
                pygame.draw.line(surf, (235, 238, 240), (xx, yy), (min(xx + dash, int(x1 - bow)), yy),
                                 max(1, cell // 20))
                xx += dash * 2
            iw = max(3, int(cell * 0.7))
            ih = max(2, int(hull_h * 0.3))
            island = pygame.Rect(int(L * 0.58), int(top + 1), iw, ih)
            pygame.draw.rect(surf, COL_BRIDGE, island, border_radius=max(1, ih // 3))
            pygame.draw.rect(surf, COL_HULL_DARK, island, lw, border_radius=max(1, ih // 3))
            return surf
        # Brücke
        bw_ = max(3, int(cell * 0.62))
        bh_ = max(3, int(hull_h * 0.52))
        bridge = pygame.Rect(int(L * 0.44 - bw_ / 2), int(mid - bh_ / 2), bw_, bh_)
        pygame.draw.rect(surf, COL_BRIDGE, bridge, border_radius=max(1, bh_ // 4))
        pygame.draw.rect(surf, COL_HULL_DARK, bridge, lw, border_radius=max(1, bh_ // 4))
        if cell >= 20:
            pygame.draw.line(surf, (60, 90, 120), (bridge.right - max(2, bw_ // 4), bridge.y + 2),
                             (bridge.right - max(2, bw_ // 4), bridge.bottom - 3), max(1, cell // 16))
        turrets = {"battleship": (0.18, 0.7, 0.84), "cruiser": (0.2, 0.76),
                   "destroyer": (0.74,)}.get(key, ())
        tr = max(2, int(hull_h * 0.22))
        for fx in turrets:
            tx = int(L * fx)
            if tx + tr >= x1 - bow * 0.35:
                tx = int(x1 - bow * 0.35 - tr - 1)
            pygame.draw.line(surf, COL_HULL_DARK, (tx, int(mid)), (tx + int(tr * 1.9), int(mid)),
                             max(1, tr // 2))
            pygame.draw.circle(surf, COL_TURRET, (tx, int(mid)), tr)
            pygame.draw.circle(surf, COL_HULL_DARK, (tx, int(mid)), tr, max(1, lw))
        return surf

    def _board_base(self, cell, kind):
        """Gecachte Brettfläche: Wasserverlauf, Schachbrett-Schimmer, Raster."""
        key = ("base", cell, kind)
        surf = self._cache.get(key)
        if surf is None:
            size = cell * N
            surf = pygame.Surface((size, size))
            radar = kind == "target"
            top, bot = (COL_RADAR_TOP, COL_RADAR_BOT) if radar else (COL_SEA_TOP, COL_SEA_BOT)
            for yy in range(size):
                pygame.draw.line(surf, _mix(top, bot, yy / max(1, size - 1)), (0, yy), (size, yy))
            shade = (5, 8, 10) if not radar else (3, 9, 8)
            for r in range(N):
                for c in range(N):
                    if (r + c) % 2:
                        surf.fill(shade, (c * cell, r * cell, cell, cell),
                                  special_flags=pygame.BLEND_RGB_ADD)
            if radar:
                ctr = (size // 2, size // 2)
                for i in (1, 2, 3, 4):
                    pygame.draw.circle(surf, _mix(COL_RADAR_TOP, COL_RING, 0.28), ctr,
                                       int(size * 0.125 * i), 1)
                pygame.draw.line(surf, _mix(COL_RADAR_TOP, COL_RING, 0.2), (ctr[0], 0), (ctr[0], size))
                pygame.draw.line(surf, _mix(COL_RADAR_TOP, COL_RING, 0.2), (0, ctr[1]), (size, ctr[1]))
            grid = COL_GRID_RADAR if radar else COL_GRID
            for i in range(N + 1):
                col = _mix(bot, grid, 0.55)
                pygame.draw.line(surf, col, (i * cell, 0), (i * cell, size))
                pygame.draw.line(surf, col, (0, i * cell), (size, i * cell))
            pygame.draw.rect(surf, _mix(bot, grid, 0.9), (0, 0, size, size), max(1, cell // 20))
            self._cache[key] = surf
        return surf

    def _wave_glyphs(self, cell):
        key = ("waves", cell)
        glyphs = self._cache.get(key)
        if glyphs is None:
            gw = max(6, int(cell * 0.5))
            gh = max(3, int(cell * 0.18))
            glyphs = []
            for alpha in (26, 52, 80, 110):
                g = pygame.Surface((gw, gh), pygame.SRCALPHA)
                pts = [(i, gh / 2 + math.sin(i / gw * math.tau) * gh * 0.38)
                       for i in range(0, gw + 1, max(1, gw // 10))]
                pygame.draw.lines(g, _rgba(COL_WAVE, alpha), False, pts, max(1, cell // 22))
                glyphs.append(g)
            self._cache[key] = glyphs
        return glyphs

    def _tint(self, cell, color, alpha):
        key = ("tint", cell, color, alpha)
        surf = self._cache.get(key)
        if surf is None:
            surf = pygame.Surface((cell, cell), pygame.SRCALPHA)
            surf.fill(_rgba(color, alpha))
            self._cache[key] = surf
        return surf

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        now = self._now()
        dt = 0.0 if self._last_draw is None else max(0.0, min(0.05, now - self._last_draw))
        if self.paused:
            dt = 0.0
        self._last_draw = now
        ui.draw_background(s, self.width, self.height)
        if self.state == SETUP:
            self._draw_setup(s, now)
            return
        if self.state == HANDOVER:
            self._draw_handover(s, now)
            return
        self._layout_check()
        self._draw_hud(s)
        if self.state == PLACE:
            self._draw_place(s, now)
        else:
            self._draw_battle(s, now, dt)
        self._step_particles(s, dt)
        if self.state == OVER:
            self._draw_over(s, now)

    # ----- Brett ---------------------------------------------------------
    def _draw_waves(self, s, x, y, cell, now, skip=None):
        glyphs = self._wave_glyphs(cell)
        gw, gh = glyphs[0].get_size()
        for r in range(N):
            for c in range(N):
                if skip is not None and skip[r][c]:
                    continue
                ph = r * 1.37 + c * 2.11
                a = math.sin(now * 1.25 + ph)
                if a < -0.35:
                    continue
                lvl = min(3, int((a + 0.35) / 1.35 * 4))
                dx = math.cos(now * 0.9 + ph) * cell * 0.12
                yy = y + r * cell + cell * (0.3 + 0.35 * ((r * 3 + c * 5) % 3) / 2) - gh / 2
                s.blit(glyphs[lvl], (x + c * cell + (cell - gw) / 2 + dx, yy))

    def _draw_labels(self, s, x, y, cell):
        f = self._label_font
        key = ("labels", cell, f.get_height(), tuple(ui.TEXT_DIM))
        imgs = self._cache.get(key)
        if imgs is None:
            imgs = ([f.render(LETTERS[i], True, ui.TEXT_DIM) for i in range(N)],
                    [f.render(str(i + 1), True, ui.TEXT_DIM) for i in range(N)])
            self._cache[key] = imgs
        for i in range(N):
            s.blit(imgs[0][i], imgs[0][i].get_rect(center=(x + i * cell + cell // 2,
                                                          y - self.lab // 2)))
            s.blit(imgs[1][i], imgs[1][i].get_rect(center=(x - self.lab // 2,
                                                          y + i * cell + cell // 2)))

    def _draw_miss(self, s, x, y, cell, now, seed):
        cx, cy = int(x + cell / 2), int(y + cell / 2)
        fr = (now * 0.55 + seed * 0.137) % 1.0
        rr = int(cell * (0.16 + 0.26 * fr))
        if rr > 2:
            pygame.draw.circle(s, _mix(COL_MISS, COL_SEA_BOT, 0.35 + 0.6 * fr), (cx, cy), rr,
                               max(1, cell // 26))
        pygame.draw.circle(s, _mix(COL_MISS, COL_SEA_BOT, 0.25), (cx, cy), max(2, int(cell * 0.2)),
                           max(1, cell // 16))
        pygame.draw.circle(s, COL_MISS, (cx, cy), max(1, int(cell * 0.08)))

    def _draw_fire(self, s, x, y, cell, now, seed, strong=True):
        cx, cy = x + cell / 2, y + cell / 2
        glow = self._glow(cell * (0.55 if strong else 0.4), (255, 120, 40))
        glow.set_alpha(int((150 if strong else 90) + 50 * math.sin(now * 11 + seed)))
        s.blit(glow, (cx - glow.get_width() / 2, cy - glow.get_height() / 2))
        glow.set_alpha(255)
        pygame.draw.circle(s, COL_SCORCH, (int(cx), int(cy + cell * 0.06)), max(2, int(cell * 0.22)))
        base = cy + cell * 0.22
        scale = 1.0 if strong else 0.6
        for i in range(3):
            ph = now * 9.0 + seed * 1.7 + i * 2.1
            fh = cell * (0.34 + 0.1 * math.sin(ph)) * scale * (0.8 if i != 1 else 1.0)
            fx = cx + (i - 1) * cell * 0.13
            sway = math.sin(ph * 1.3) * cell * 0.05
            w = cell * 0.1 * scale
            pygame.draw.polygon(s, COL_FLAME, [(fx - w, base), (fx + sway, base - fh), (fx + w, base)])
            pygame.draw.polygon(s, COL_FLAME_IN, [(fx - w * 0.45, base),
                                                  (fx + sway * 0.6, base - fh * 0.55),
                                                  (fx + w * 0.45, base)])

    def _draw_sea(self, s, side, x, y, cell, kind, now):
        """kind: "own" (eigene Flotte sichtbar) oder "target" (Nebel des Krieges)."""
        sea = self.seas[side]
        s.blit(self._board_base(cell, kind), (x, y))
        occupied = [[False] * N for _ in range(N)]
        shown = []
        reveal_all = self.state == OVER
        for ship in sea.ships:
            if kind == "own" or ship.sunk or reveal_all:
                shown.append(ship)
                for (r, c) in ship.cells:
                    occupied[r][c] = True
        for r in range(N):
            for c in range(N):
                if sea.shots[r][c] != core.UNKNOWN:
                    occupied[r][c] = True
        self._draw_waves(s, x, y, cell, now, skip=occupied)
        for ship in shown:
            if ship.sunk:
                img = self._ship_sprite(ship.idx, cell, ship.horiz, "sunk")
                s.blit(img, (x + ship.c * cell, y + ship.r * cell))
            elif kind == "own":
                img = self._ship_sprite(ship.idx, cell, ship.horiz)
                s.blit(img, (x + ship.c * cell, y + ship.r * cell))
            else:
                img = self._ship_sprite(ship.idx, cell, ship.horiz)
                img.set_alpha(150)
                s.blit(img, (x + ship.c * cell, y + ship.r * cell))
                img.set_alpha(255)
        sunk_cells = {rc for sh in sea.ships if sh.sunk for rc in sh.cells}
        for r in range(N):
            for c in range(N):
                v = sea.shots[r][c]
                if v == core.MISS:
                    self._draw_miss(s, x + c * cell, y + r * cell, cell, now, r * 10 + c)
                elif v == core.HIT:
                    self._draw_fire(s, x + c * cell, y + r * cell, cell, now, r * 10 + c,
                                    strong=(r, c) not in sunk_cells)
        if kind == "target":
            self._draw_radar(s, x, y, cell * N, now)
        # Versenkt-Enthüllung: Umriss pulsiert auf
        for rv in self.reveals:
            if rv["side"] != side:
                continue
            k = (now - rv["t0"]) / 1.4
            if not 0 <= k < 1:
                continue
            sh = rv["ship"]
            w = (sh.size if sh.horiz else 1) * cell
            h = (1 if sh.horiz else sh.size) * cell
            grow = int(cell * 0.35 * (1 - k))
            col = _mix((255, 255, 255), ui.GOLD, k)
            rect = pygame.Rect(x + sh.c * cell, y + sh.r * cell, w, h).inflate(grow, grow)
            pygame.draw.rect(s, col, rect, max(2, int(cell * 0.09 * (1 - k)) + 1),
                             border_radius=max(2, cell // 5))

    def _draw_sunk_banners(self, s, now, ox, oy):
        """"VERSENKT!" steigt über dem gerade enthüllten Schiff auf und verblasst."""
        v = self._viewer()
        for rv in self.reveals:
            k = (now - rv["t0"]) / 1.6
            if not 0 <= k < 1:
                continue
            side = rv["side"]
            x, y, cell = self._board_geom(side)
            sh = rv["ship"]
            cx = x + ox + (sh.c + (sh.size / 2.0 if sh.horiz else 0.5)) * cell
            cy = y + oy + (sh.r + (0.5 if sh.horiz else sh.size / 2.0)) * cell
            font = self._bold if side != v else self._small
            col = ui.GOLD if side != v else COL_P2
            key = ("banner", t("bs.sunk_banner"), id(font), col)
            imgs = self._cache.get(key)
            if imgs is None:
                imgs = (font.render(t("bs.sunk_banner"), True, (8, 12, 20)),
                        font.render(t("bs.sunk_banner"), True, col))
                self._cache[key] = imgs
            pop = 1.0 + 0.25 * max(0.0, 1 - k * 6)          # kurzes Aufploppen
            alpha = int(255 * min(1.0, (1 - k) / 0.35))
            yy = cy - cell * (0.2 + 0.9 * _smooth(k))
            for i, img in enumerate(imgs):
                if pop > 1.01:
                    img = pygame.transform.smoothscale(
                        img, (int(img.get_width() * pop), int(img.get_height() * pop)))
                img.set_alpha(alpha)
                off = 2 if i == 0 else 0
                half = img.get_width() // 2 + 4
                bx_ = int(max(half, min(self.width - half, cx)))
                s.blit(img, img.get_rect(center=(bx_ + off, int(yy) + off)))
                img.set_alpha(255)

    def _draw_radar(self, s, x, y, size, now):
        key = ("sweep", size)
        surf = self._cache.get(key)
        if surf is None:
            surf = pygame.Surface((size, size))
            self._cache[key] = surf
        surf.fill((0, 0, 0))
        ang = now * 1.5
        cx = cy = size / 2.0
        R = size * 0.72
        steps = 12
        span = 1.05
        for i in range(steps):
            a0 = ang - span * (i + 1) / steps
            a1 = ang - span * i / steps
            k = (1 - i / steps) ** 2
            pygame.draw.polygon(surf, (int(6 * k), int(62 * k), int(44 * k)),
                                [(cx, cy), (cx + math.cos(a0) * R, cy + math.sin(a0) * R),
                                 (cx + math.cos(a1) * R, cy + math.sin(a1) * R)])
        pygame.draw.line(surf, (40, 150, 105), (cx, cy),
                         (cx + math.cos(ang) * R, cy + math.sin(ang) * R), max(2, size // 220))
        s.blit(surf, (x, y), special_flags=pygame.BLEND_RGB_ADD)

    def _draw_brackets(self, s, rect, color, width):
        L = max(4, rect.w // 3)
        x0, y0, x1, y1 = rect.left, rect.top, rect.right - 1, rect.bottom - 1
        for (ax, ay, dx, dy) in ((x0, y0, 1, 1), (x1, y0, -1, 1), (x0, y1, 1, -1), (x1, y1, -1, -1)):
            pygame.draw.line(s, color, (ax, ay), (ax + dx * L, ay), width)
            pygame.draw.line(s, color, (ax, ay), (ax, ay + dy * L), width)

    # ----- Gefecht -------------------------------------------------------
    def _shake_offset(self, now):
        k = (now - self.shake_t0) / 0.32
        if not 0 <= k < 1 or self.paused:
            return 0, 0
        a = self.shake_amp * (1 - k)
        return int(math.sin(now * 90) * a), int(math.cos(now * 77) * a)

    def _draw_battle(self, s, now, dt):
        v = self._viewer()
        ox, oy = self._shake_offset(now)
        bx, by, sx, sy = self.bx + ox, self.by + oy, self.sx + ox, self.sy + oy
        cb, cs = self.cb, self.cs
        self._draw_labels(s, self.bx, self.by, cb)
        self._draw_sea(s, 1 - v, bx, by, cb, "target", now)
        self._draw_sea(s, v, sx, sy, cs, "own", now)

        human = self.state == PLAY and not self._is_ai(self.turn) and self.turn == v
        if human and self.flight is None:
            ar, ac = self.aim[v]
            rect = pygame.Rect(bx + ac * cb, by + ar * cb, cb, cb)
            free = self.seas[1 - v].shots[ar][ac] == core.UNKNOWN
            pul = 0.5 + 0.5 * math.sin(now * 6)
            col = _mix(COL_AIM, (255, 255, 255), pul * 0.4) if free else ui.TEXT_FAINT
            if free:
                s.blit(self._tint(cb, COL_AIM, int(24 + 22 * pul)), rect.topleft)
            self._draw_brackets(s, rect.inflate(-max(2, cb // 10), -max(2, cb // 10)), col,
                                max(2, cb // 16))
        if self.state == PLAY and self._is_ai(self.turn):
            px, py = self.ai_pos
            rect = pygame.Rect(int(sx + (px - 0.5) * cs), int(sy + (py - 0.5) * cs), cs, cs)
            self._draw_brackets(s, rect, COL_AI_AIM, max(1, cs // 12))
            pygame.draw.circle(s, COL_AI_AIM, rect.center, max(2, cs // 3), 1)

        if self.flight is not None:
            self._draw_flight(s, now)
        self._draw_sunk_banners(s, now, ox, oy)

        # Beschriftung unter dem eigenen Brett
        cap = t("bs.your_waters") if not self.multiplayer else \
            t("bs.waters_of", name=self._pname(v))
        img = self._fit(self._tiny, cap, self.col_w, ui.TEXT_DIM)
        s.blit(img, img.get_rect(center=(self.sx + 5 * cs, self.caption_y)))
        # Flottenübersicht: Gegner links, eigene rechts
        self._draw_fleet(s, self.fleet_rects[0], self.seas[1 - v], enemy=True)
        self._draw_fleet(s, self.fleet_rects[1], self.seas[v], enemy=False)
        self._draw_status(s, now)

    def _draw_flight(self, s, now):
        f = self.flight
        side = 1 - f["shooter"]
        x, y, cell = self._board_geom(side)
        tx, ty = x + f["c"] * cell + cell / 2, y + f["r"] * cell + cell / 2
        ox, oy, ocell = self._board_geom(f["shooter"])
        sx_, sy_ = ox + ocell * 5, oy + ocell * 5
        k = min(1.0, f["t"] / f["dur"])
        dist = math.hypot(tx - sx_, ty - sy_)
        for j in range(6, -1, -1):
            kk = max(0.0, k - j * 0.035)
            e = kk * kk * (3 - 2 * kk) * 0.35 + kk * 0.65
            px = sx_ + (tx - sx_) * e
            py = sy_ + (ty - sy_) * e - math.sin(math.pi * kk) * dist * 0.28
            rad = max(2, int(cell * 0.16 * (1 - j / 8)))
            col = _mix((255, 240, 190), (255, 120, 50), j / 6)
            if j == 0:
                g = self._glow(cell * 0.35, (255, 200, 120))
                s.blit(g, (px - g.get_width() / 2, py - g.get_height() / 2))
            pygame.draw.circle(s, col, (int(px), int(py)), rad)
        # Zielmarkierung, auf die die Granate zufliegt
        pygame.draw.circle(s, COL_AI_AIM if side == self._viewer() else COL_AIM,
                           (int(tx), int(ty)), max(3, int(cell * 0.42 * (1 - k * 0.6))), 1)

    def _pname(self, p):
        if self.multiplayer:
            return t("common.player1") if p == 0 else t("common.player2")
        return t("bs.you") if p == 0 else t("common.ai")

    def _fit(self, font, text, maxw, color):
        """Rendert 'text' höchstens 'maxw' breit: erst kleinere Schriften, dann kürzen."""
        chain = [font]
        if font is self._huge:
            chain.append(self._bold)
        if font in (self._huge, self._bold):
            chain.append(self._small)
        if font is not self._tiny:
            chain.append(self._tiny)
        for f in chain:
            img = f.render(text, True, color)
            if img.get_width() <= maxw:
                return img
        kurz = text
        while len(kurz) > 2 and self._tiny.size(kurz + "…")[0] > maxw:
            kurz = kurz[:-1]
        return self._tiny.render(kurz + "…", True, color)

    def _draw_fleet(self, s, rect, sea, enemy):
        title = t("bs.fleet_enemy") if enemy else t("bs.fleet_own")
        left = sea.ships_left()
        head = self._fit(self._tiny, "%s  %d/%d" % (title, left, len(FLEET)), rect.w,
                         COL_P2 if enemy else COL_P1)
        s.blit(head, head.get_rect(midtop=(rect.centerx, rect.top)))
        y0 = rect.top + head.get_height() + 4
        row_h = max(6, (rect.bottom - y0) // len(FLEET))
        u = max(4, int(min((rect.w - 6) / 5.0, row_h * 0.78)))
        for idx in range(len(FLEET)):
            ship = sea.ship_by_idx(idx)
            size = FLEET[idx]
            sw = size * u
            xx = rect.centerx - (5 * u) // 2
            yy = y0 + idx * row_h + (row_h - u) // 2
            sunk = ship is not None and ship.sunk
            img = self._ship_sprite(idx, u, True, "sunk" if sunk else "normal")
            if sunk:
                img.set_alpha(170)
            s.blit(img, (xx, yy))
            img.set_alpha(255)
            if sunk:
                pygame.draw.line(s, COL_INVALID, (xx - 2, yy + u // 2), (xx + sw + 2, yy + u // 2),
                                 max(2, u // 7))
            elif not enemy and ship is not None:
                for (r, c) in ship.hits:
                    i = (c - ship.c) if ship.horiz else (r - ship.r)
                    pygame.draw.circle(s, COL_FLAME, (xx + int((i + 0.5) * u), yy + u // 2),
                                       max(2, u // 4))

    def _draw_status(self, s, now):
        y = self.status_y
        x0 = self.bx
        right = self.bx + 10 * self.cb
        # Munition (Salve / Extraschüsse) rechts als Granaten
        pip_w = 0
        human = self.state == PLAY and not self._is_ai(self.turn)
        if self.state == PLAY and (self.rules["salvo"] or self.shots_left > 1):
            r = max(3, self._small.get_height() // 5)
            n = max(0, self.shots_left)
            pip_w = n * (2 * r + 4)
            for i in range(n):
                cx = right - r - i * (2 * r + 4)
                col = COL_AIM if human else COL_AI_AIM
                pygame.draw.circle(s, col, (cx, y), r)
                pygame.draw.circle(s, _mix(col, (0, 0, 0), 0.5), (cx, y), r, 1)
            lbl = self._tiny.render(t("bs.shots_left", n=n), True, ui.TEXT_DIM)
            if right - pip_w - 8 - lbl.get_width() > x0 + 120:
                s.blit(lbl, lbl.get_rect(midright=(right - pip_w - 6, y)))
                pip_w += lbl.get_width() + 10
        if self.msg:
            text, col = self.msg, self.msg_col
        elif self.state == PLAY and human:
            text, col = t("bs.fire_hint"), ui.TEXT_FAINT
        else:
            text, col = "", ui.TEXT_DIM
        if text:
            img = self._fit(self._small, text, right - x0 - pip_w - 8, col)
            s.blit(img, img.get_rect(midleft=(x0, y)))

    # ----- Aufstellen ----------------------------------------------------
    def _draw_place(self, s, now):
        sea = self._place_sea()
        x, y, cell = self.bx, self.by, self.cb
        self._draw_labels(s, x, y, cell)
        s.blit(self._board_base(cell, "own"), (x, y))
        occ = [[False] * N for _ in range(N)]
        for ship in sea.ships:
            for (r, c) in ship.cells:
                occ[r][c] = True
        self._draw_waves(s, x, y, cell, now, skip=occ)
        if not self.rules["touch"]:
            # Sperrzone um liegende Schiffe sichtbar machen
            halo = self._tint(cell, (255, 255, 255), 16)
            seen = set()
            for ship in sea.ships:
                for (r, c) in ship.cells:
                    for dr in (-1, 0, 1):
                        for dc in (-1, 0, 1):
                            rr, cc = r + dr, c + dc
                            if core.in_board(rr, cc) and not occ[rr][cc] and (rr, cc) not in seen:
                                seen.add((rr, cc))
                                s.blit(halo, (x + cc * cell, y + rr * cell))
        for ship in sea.ships:
            s.blit(self._ship_sprite(ship.idx, cell, ship.horiz), (x + ship.c * cell, y + ship.r * cell))

        hd = self.held
        if hd is not None:
            size = FLEET[hd["idx"]]
            anchor = self._held_anchor()
            img = self._ship_sprite(hd["idx"], cell, hd["horiz"])
            if anchor is not None:
                valid = self._held_valid()
                tint = self._tint(cell, COL_VALID if valid else COL_INVALID, 70 if valid else 90)
                for (r, c) in core.cells_of(anchor[0], anchor[1], size, hd["horiz"]):
                    if core.in_board(r, c):
                        s.blit(tint, (x + c * cell, y + r * cell))
                prev = s.get_clip()
                s.set_clip(pygame.Rect(x, y, cell * N, cell * N))
                img.set_alpha(235 if valid else 150)
                s.blit(img, (x + anchor[1] * cell, y + anchor[0] * cell))
                img.set_alpha(255)
                s.set_clip(prev)
                rect = pygame.Rect(x + anchor[1] * cell, y + anchor[0] * cell,
                                   (size if hd["horiz"] else 1) * cell,
                                   (1 if hd["horiz"] else size) * cell)
                pygame.draw.rect(s, COL_VALID if valid else COL_INVALID, rect.clip(
                    pygame.Rect(x, y, cell * N, cell * N)), max(2, cell // 14),
                    border_radius=max(2, cell // 6))
            else:
                mx, my = self.mouse
                off = (hd["grab"] + 0.5) * cell
                px = mx - (off if hd["horiz"] else cell / 2)
                py = my - (cell / 2 if hd["horiz"] else off)
                img.set_alpha(190)
                s.blit(img, (px, py))
                img.set_alpha(255)
        elif self.kb_mode:
            r, c = self.kcursor
            rect = pygame.Rect(x + c * cell, y + r * cell, cell, cell)
            self._draw_brackets(s, rect, COL_AIM, max(2, cell // 16))

        # Rechte Spalte: Dock + Knöpfe
        title = t("bs.fleet_own") + "  %d/%d" % (len(sea.ships), len(FLEET))
        img = self._fit(self._small, title, self.col_w, COL_P1 if self.place_player == 0 else COL_P2)
        s.blit(img, img.get_rect(center=(self.col_x + self.col_w // 2, self.dock_title_y)))
        dc = self.dock_cell
        for idx, rc in enumerate(self.dock_rects):
            placed = sea.ship_by_idx(idx) is not None
            holding = hd is not None and hd["idx"] == idx
            hover = rc.collidepoint(self.mouse) and hd is None
            inner = rc.inflate(0, -max(2, rc.h // 8))
            pygame.draw.rect(s, ui.BTN_SEL if (hover or holding) else ui.BTN, inner,
                             border_radius=max(4, inner.h // 4))
            pygame.draw.rect(s, self.accent if holding else ui.BORDER, inner,
                             2 if holding else 1, border_radius=max(4, inner.h // 4))
            img = self._ship_sprite(idx, dc, True)
            sy_ = rc.centery - dc // 2
            if placed or holding:
                img.set_alpha(80)
            s.blit(img, (rc.x + 8, sy_))
            img.set_alpha(255)
            if placed:
                # Haken mitten auf der verblassten Silhouette
                ck = max(4, min(rc.h // 4, dc // 2))
                cxk, cyk = rc.x + 8 + FLEET[idx] * dc // 2, rc.centery
                pygame.draw.lines(s, COL_VALID, False, [(cxk - ck, cyk), (cxk - ck // 3, cyk + ck * 2 // 3),
                                                       (cxk + ck, cyk - ck * 2 // 3)], max(2, ck // 3))
            tx = rc.x + 14 + 5 * dc
            room = rc.right - tx - 8
            label = t("bs.ship." + core.SHIP_KEYS[idx])
            col = ui.TEXT_FAINT if placed else ui.TEXT
            if self._tiny.size(label)[0] > room:
                # Kleine Auflösung + langer Name: nur die Länge, der Name
                # steht beim Überfahren in der Statuszeile.
                label = "×%d" % FLEET[idx]
            if room > 16:
                lbl = self._fit(self._tiny, label, room, col)
                s.blit(lbl, lbl.get_rect(midleft=(tx, rc.centery)))
        labels = {"rotate": t("bs.btn_rotate"), "random": t("bs.btn_random"),
                  "clear": t("bs.btn_clear"), "ready": t("bs.btn_ready")}
        keys = {"rotate": "R", "random": "X", "clear": "C", "ready": "Enter"}
        for key in BUTTONS:
            rc = self.btn_rects[key]
            hover = rc.collidepoint(self.mouse)
            if key == "ready":
                on = sea.complete()
                pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=10)
                pygame.draw.rect(s, self.accent if on else ui.BORDER, rc, 2 if on else 1,
                                 border_radius=10)
                if on and hover:
                    pygame.draw.rect(s, (255, 255, 255), rc.inflate(-4, -4), 1, border_radius=9)
                col = ui.TEXT if on else ui.TEXT_FAINT
            else:
                pygame.draw.rect(s, ui.BTN_SEL if hover else ui.BTN, rc, border_radius=8)
                pygame.draw.rect(s, ui.BORDER_LIGHT if hover else ui.BORDER, rc, 1, border_radius=8)
                col = ui.TEXT if hover else ui.TEXT_DIM
            kimg = self._tiny.render(keys[key], True, ui.TEXT_FAINT)
            room = rc.w - kimg.get_width() - 24
            lbl = self._fit(self._small, labels[key], room, col)
            s.blit(lbl, lbl.get_rect(center=(rc.x + 8 + room // 2, rc.centery)))
            s.blit(kimg, kimg.get_rect(midright=(rc.right - 8, rc.centery)))

        # Statuszeile
        if self.msg:
            text, col = self.msg, self.msg_col
        elif hd is not None:
            text = t("bs.holding", ship=t("bs.ship." + core.SHIP_KEYS[hd["idx"]]))
            col = ui.TEXT_DIM
        else:
            hover = next((i for i, rc in enumerate(self.dock_rects) if rc.collidepoint(self.mouse)),
                         None)
            if hover is not None:
                text = "%s (%d)" % (t("bs.ship." + core.SHIP_KEYS[hover]), FLEET[hover])
                col = ui.TEXT_DIM
            else:
                text, col = t("bs.place_hint"), ui.TEXT_FAINT
        img = self._fit(self._small, text, self.col_x + self.col_w - x, col)
        s.blit(img, img.get_rect(midleft=(x, self.status_y)))

    # ----- HUD / Rundenende ----------------------------------------------
    def _draw_hud(self, s):
        pygame.draw.rect(s, ui.PANEL, (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h), (self.width, self.hud_h))
        cy = self.hud_h // 2
        img_l = self._small.render("%s: %d" % (self._pname(0), self.wins[0]), True, COL_P1)
        s.blit(img_l, img_l.get_rect(midleft=(12, cy)))
        img_r = self._small.render("%d :%s" % (self.wins[1], self._pname(1)), True, COL_P2)
        s.blit(img_r, img_r.get_rect(midright=(self.width - 12, cy)))
        mid, col = self._hud_center()
        if mid:
            room = self.width - 2 * max(img_l.get_width(), img_r.get_width()) - 48
            img = self._fit(self._small, mid, room, col)
            s.blit(img, img.get_rect(center=(self.width // 2, cy)))

    def _hud_center(self):
        if self.state == PLACE:
            if self.multiplayer:
                return (t("bs.place_title_p", name=self._pname(self.place_player)),
                        COL_P1 if self.place_player == 0 else COL_P2)
            return t("bs.place_title"), COL_P1
        if self.state == PLAY:
            col = COL_P1 if self.turn == 0 else COL_P2
            if self._is_ai(self.turn):
                return t("bs.ai_thinks"), col
            if not self.multiplayer:
                return t("bs.your_turn"), col
            return t("bs.turn", name=self._pname(self.turn)), col
        return "", ui.TEXT

    def _over_rows(self):
        rows = []
        for key, fn in (("bs.stat_shots", lambda p: str(self.stats[p]["shots"])),
                        ("bs.stat_hits", lambda p: str(self.stats[p]["hits"])),
                        ("bs.stat_acc", lambda p: "%d%%" % round(
                            100 * self.stats[p]["hits"] / max(1, self.stats[p]["shots"]))),
                        ("bs.stat_turns", lambda p: str(self.stats[p]["turns"])),
                        ("bs.stat_left", lambda p: "%d/%d" % (self.seas[p].ships_left(), len(FLEET)))):
            rows.append((t(key), fn(0), fn(1)))
        return rows

    def _over_panel_rect(self):
        w, h = self.width, self.height
        hh = self._huge.get_height()
        rh = self._small.get_height() + 4
        th = self._tiny.get_height()
        ph = 14 + hh + 10 + rh * 6 + 10 + th + 16
        pw = min(w - 32, max(340, int(w * 0.56)))
        top, bottom = self.hud_h + 6, h - 58
        py = top + max(0, (bottom - top - ph) // 2)
        return pygame.Rect((w - pw) // 2, py, pw, ph)

    def _draw_over(self, s, now):
        rect = self._over_panel_rect()
        k = _smooth((now - self.over_t0) / 0.35)
        rect = rect.move(0, int((1 - k) * 18))
        # Bretter leicht abdunkeln, Panel deckend unterlegen (Glas-Themes)
        dim = self._cache.get(("dim", self.width, self.height))
        if dim is None:
            dim = pygame.Surface((self.width, self.height - self.hud_h - 1))
            dim.fill((4, 8, 16))
            self._cache[("dim", self.width, self.height)] = dim
        dim.set_alpha(int(110 * k))
        s.blit(dim, (0, self.hud_h + 1))
        pygame.draw.rect(s, ui.PANEL, rect, border_radius=12)
        ui.draw_panel(s, rect, accent_top=self.accent)
        cx = rect.centerx
        y = rect.top + 14
        if self.multiplayer:
            head = t("common.player_wins", n=(self.winner or 0) + 1)
            col = COL_P1 if self.winner == 0 else COL_P2
        elif self.winner == 0:
            head, col = t("bs.win_you"), ui.GOLD
        else:
            head, col = t("bs.win_ai"), COL_P2
        img = self._fit(self._huge, head, rect.w - 24, col)
        s.blit(img, img.get_rect(midtop=(cx, y)))
        y += self._huge.get_height() + 10
        rh = self._small.get_height() + 4
        c1 = rect.left + 18
        c3 = rect.right - 18
        colw = max(self._small.size(self._pname(0))[0], self._small.size(self._pname(1))[0],
                   self._small.size("100%")[0]) + 12
        c2 = c3 - colw
        for i, txt in enumerate((self._pname(0), self._pname(1))):
            im = self._small.render(txt, True, COL_P1 if i == 0 else COL_P2)
            s.blit(im, im.get_rect(midright=(c2 if i == 0 else c3, y + rh // 2)))
        pygame.draw.line(s, ui.BORDER, (c1, y + rh), (c3, y + rh))
        y += rh + 2
        for (label, a, b) in self._over_rows():
            im = self._fit(self._small, label, c2 - colw - c1 - 6, ui.TEXT_DIM)
            s.blit(im, im.get_rect(midleft=(c1, y + rh // 2)))
            for val, xx in ((a, c2), (b, c3)):
                im = self._small.render(val, True, ui.TEXT)
                s.blit(im, im.get_rect(midright=(xx, y + rh // 2)))
            y += rh
        y += 8
        hint = self._fit(self._tiny, t("bs.new_round"), rect.w - 20, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(midtop=(cx, y)))

    # ----- Übergabe ------------------------------------------------------
    def _draw_handover(self, s, now):
        w, h = self.width, self.height
        key = ("curtain", w, h)
        bg = self._cache.get(key)
        if bg is None:
            bg = pygame.Surface((w, h))
            for yy in range(h):
                pygame.draw.line(bg, _mix((16, 50, 88), (4, 16, 34), yy / max(1, h - 1)), (0, yy), (w, yy))
            self._cache[key] = bg
        s.blit(bg, (0, 0))
        cell = max(24, h // 12)
        glyphs = self._wave_glyphs(cell)
        for row in range(4):
            yy = int(h * (0.62 + row * 0.085))
            for i in range(-1, w // cell + 2):
                ph = row * 1.7 + i * 0.9
                a = math.sin(now * 1.2 + ph)
                if a < -0.3:
                    continue
                lvl = min(3, int((a + 0.3) / 1.3 * 4))
                xx = i * cell + (now * (18 + row * 9)) % cell - cell
                s.blit(glyphs[lvl], (xx, yy + math.sin(now * 1.6 + ph) * 3))
        ho = self.handover or dict(to=0, kind="fire", report=None, t=0.0)
        # Schiff schaukelt auf den Wellen
        sc = max(12, h // 22)
        img = self._ship_sprite(1, sc, True)
        rot = math.sin(now * 1.4) * 4
        ship = pygame.transform.rotate(img, rot)
        s.blit(ship, ship.get_rect(center=(w // 2, int(h * 0.58 + math.sin(now * 1.9) * 4))))
        col = COL_P1 if ho["to"] == 0 else COL_P2
        cy = int(h * 0.2)
        title = self._fit(self._huge, t("bs.handover_title", name=self._pname(ho["to"])), w - 40, col)
        s.blit(title, title.get_rect(center=(w // 2, cy)))
        cy += title.get_height() // 2 + self._bold.get_height() // 2 + 8
        sub = t("bs.handover_place") if ho["kind"] == "place" else t("bs.handover_fire")
        img = self._fit(self._bold, sub, w - 40, ui.TEXT)
        s.blit(img, img.get_rect(center=(w // 2, cy)))
        cy += self._bold.get_height() // 2 + self._small.get_height() // 2 + 10
        img = self._fit(self._small, t("bs.handover_away", name=self._pname(1 - ho["to"])),
                        w - 40, ui.TEXT_DIM)
        s.blit(img, img.get_rect(center=(w // 2, cy)))
        if ho.get("report"):
            cy += self._small.get_height() + 6
            rp = ho["report"]
            img = self._fit(self._small, t("bs.handover_report", shots=rp["shots"], hits=rp["hits"]),
                            w - 40, ui.GOLD)
            s.blit(img, img.get_rect(center=(w // 2, cy)))
        if ho["t"] >= 0.45:
            pul = 0.55 + 0.45 * math.sin(now * 4)
            img = self._fit(self._small, t("bs.handover_ready"), w - 40,
                            _mix(ui.TEXT_DIM, ui.TEXT, pul))
            s.blit(img, img.get_rect(center=(w // 2, h - max(24, h // 14))))

    # ----- Setup zeichnen ------------------------------------------------
    def _draw_setup(self, s, now):
        w, h = self.width, self.height
        cx = w // 2
        title = self._fit(self._huge, self.name.upper(), w - 30, self.accent)
        s.blit(title, title.get_rect(center=(cx, int(h * 0.11))))
        sub = self._fit(self._small, t("bs.subtitle"), w - 30, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(h * 0.11) + title.get_height() // 2
                                         + sub.get_height() // 2 + 6)))
        items = self._setup_items()
        sel = items[max(0, min(len(items) - 1, self.setup_sel))]
        if not self.multiplayer:
            lbl = self._tiny.render(t("bs.lbl_diff"), True, ui.TEXT_DIM)
            s.blit(lbl, lbl.get_rect(midbottom=(cx, self.diff_label_y)))
            for i, rc in enumerate(self.diff_rects):
                on = i == self.diff
                pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=9)
                pygame.draw.rect(s, self.accent if on else ui.BORDER, rc, 2 if on else 1,
                                 border_radius=9)
                if sel == "diff" and on:
                    pygame.draw.rect(s, ui.TEXT, rc.inflate(4, 4), 1, border_radius=11)
                im = self._fit(self._small, t("bs.diff." + DIFFS[i]), rc.w - 10,
                               ui.TEXT if on else ui.TEXT_DIM)
                s.blit(im, im.get_rect(center=rc.center))
        lbl = self._tiny.render(t("bs.lbl_rules"), True, ui.TEXT_DIM)
        s.blit(lbl, lbl.get_rect(midbottom=(cx, self.rules_label_y)))
        on_txt, off_txt = t("common.on"), t("common.off")
        for i, rc in enumerate(self.rule_rects):
            key = RULES[i]
            on = self.rules[key]
            focus = sel == key
            pygame.draw.rect(s, ui.BTN_SEL if focus else ui.BTN, rc, border_radius=9)
            pygame.draw.rect(s, self.accent if focus else ui.BORDER, rc, 2 if focus else 1,
                             border_radius=9)
            isz = max(10, int(rc.h * 0.5))
            self._draw_rule_icon(s, key, rc.x + 10 + isz // 2, rc.centery, isz)
            pill_w, pill_h = self._pill_size(rc.h)
            pill = pygame.Rect(rc.right - 10 - pill_w, rc.centery - pill_h // 2, pill_w, pill_h)
            pygame.draw.rect(s, _mix(ui.BTN, ui.GREEN, 0.75) if on else ui.PANEL, pill,
                             border_radius=pill_h // 2)
            pygame.draw.rect(s, ui.GREEN if on else ui.BORDER_LIGHT, pill, 1, border_radius=pill_h // 2)
            kr = pill_h // 2 - 3
            kx = pill.right - pill_h // 2 if on else pill.left + pill_h // 2
            pygame.draw.circle(s, ui.TEXT if on else ui.TEXT_DIM, (kx, pill.centery), kr)
            st = self._tiny.render(on_txt if on else off_txt, True, (20, 30, 24) if on else ui.TEXT_DIM)
            s.blit(st, st.get_rect(midleft=(pill.left + 8, pill.centery)) if on
                   else st.get_rect(midright=(pill.right - 8, pill.centery)))
            room = pill.left - (rc.x + 20 + isz) - 8
            im = self._fit(self._small, t(RULE_TEXT[key]), room, ui.TEXT if on else ui.TEXT_DIM)
            s.blit(im, im.get_rect(midleft=(rc.x + 20 + isz, rc.centery)))
        focus = sel == "start"
        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=10)
        pygame.draw.rect(s, ui.TEXT if focus else self.accent, self.start_rect, 2, border_radius=10)
        st = self.font.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        hint = self._fit(self._tiny, t("bs.setup_hint"), w - 20, ui.TEXT_FAINT)
        s.blit(hint, hint.get_rect(center=(cx, h - 13)))

    def _pill_size(self, row_h):
        """Breite/Höhe des AN/AUS-Schalters (passt sich der Sprache an)."""
        ph = max(16, int(row_h * 0.62))
        tw = max(self._tiny.size(t("common.on"))[0], self._tiny.size(t("common.off"))[0])
        return tw + ph + 16, ph

    def _draw_rule_icon(self, s, key, cx, cy, size):
        col = ui.TEXT_DIM
        u = max(3, size // 4)
        if key == "touch":
            # zwei Schiffe dicht an dicht
            pygame.draw.rect(s, col, (cx - size // 2, cy - u - 1, size, u), border_radius=u // 2)
            pygame.draw.rect(s, col, (cx - size // 2, cy + 1, size - u, u), border_radius=u // 2)
        elif key == "salvo":
            for i in (-1, 0, 1):
                pygame.draw.circle(s, col, (cx + i * (u + 2), cy + (i % 2) * 2 - 1), max(2, u // 2 + 1))
        else:
            pygame.draw.circle(s, col, (cx, cy), size // 2, max(1, size // 10))
            pygame.draw.line(s, col, (cx - u, cy), (cx + u, cy), max(1, size // 8))
            pygame.draw.line(s, col, (cx, cy - u), (cx, cy + u), max(1, size // 8))
