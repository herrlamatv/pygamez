# -*- coding: utf-8 -*-
"""
muehle.py
=========
Mühle (Nine Men's Morris) - 1 Spieler gegen KI oder 2 Spieler lokal.

Ablauf in drei Phasen:
  1. Setzen : Jeder Spieler setzt nacheinander seine 9 Steine auf freie Punkte.
  2. Ziehen : Danach werden Steine entlang der Linien auf benachbarte freie
              Punkte gezogen.
  3. Springen (Fliegen): Wer nur noch 3 Steine hat, darf auf JEDEN freien Punkt
              springen (per Feature-Schalter im Setup abschaltbar).

Bildet ein Zug eine Mühle (drei eigene Steine in einer Linie), wird ein
gegnerischer Stein entfernt - möglichst keiner aus einer gegnerischen Mühle,
außer es steht kein anderer zur Verfügung. Verloren hat, wer auf unter 3 Steine
fällt oder keinen Zug mehr machen kann.

- Einzelspieler: KI über Minimax mit Alpha-Beta, phasengerechter Bewertung
  (Material, geschlossene Mühlen, Beweglichkeit, offene Zwickmühlen) und
  Zeitbudget. Drei Stärken; die leichte patzt absichtlich.
- Punkte (Highscore) = Siege gegen die KI in einer Sitzung.

Steuerung: Maus (Punkt anklicken; zum Ziehen erst eigenen Stein, dann Ziel).
Nach Rundenende: Enter = neue Runde, S = Setup (nur Einzelspieler).
"""

import random
import time

import pygame

import settings as settings_mod
import ui
from game_base import Game, InputEvent, LocalizedName
from i18n import t

# ------------------------------------------------- Brett-Identitätsfarben
# Generische UI-Farben (Hintergrund, Panels, Text) liefert die ui-Palette.
COL_PLATE = (24, 38, 31)         # Grundplatte hinter dem Liniennetz
COL_LINE = (120, 168, 140)
COL_SPOT = (46, 74, 60)
COL_P1 = (238, 240, 246)         # Spieler 0 (hell)
COL_P1_HI = (255, 255, 255)
COL_P2 = (44, 52, 66)            # Spieler 1 (dunkel)
COL_P2_HI = (96, 108, 130)
COL_SEL = (246, 214, 92)
COL_HINT = (120, 210, 150)
COL_MILL = (240, 180, 80)
COL_REMOVE = (224, 96, 96)

SETUP, PLAY, OVER = "setup", "play", "over"

DIFFS = ["easy", "medium", "hard"]
DEPTHS = [1, 2, 4]
TIME_BUDGET = [0.2, 0.5, 1.0]
NODE_BUDGET = 300000

# 24 Punkte als (Spalte, Zeile) im 0..6-Raster (siehe Kommentar unten).
POS = [(0, 0), (3, 0), (6, 0),
       (1, 1), (3, 1), (5, 1),
       (2, 2), (3, 2), (4, 2),
       (0, 3), (1, 3), (2, 3), (4, 3), (5, 3), (6, 3),
       (2, 4), (3, 4), (4, 4),
       (1, 5), (3, 5), (5, 5),
       (0, 6), (3, 6), (6, 6)]

ADJ = [
    [1, 9], [0, 2, 4], [1, 14],
    [4, 10], [1, 3, 5, 7], [4, 13],
    [7, 11], [4, 6, 8], [7, 12],
    [0, 10, 21], [3, 9, 11, 18], [6, 10, 15],
    [8, 13, 17], [5, 12, 14, 20], [2, 13, 23],
    [11, 16], [15, 17, 19], [12, 16],
    [10, 19], [16, 18, 20, 22], [13, 19],
    [9, 22], [19, 21, 23], [14, 22],
]

MILLS = [
    [0, 1, 2], [3, 4, 5], [6, 7, 8], [9, 10, 11], [12, 13, 14],
    [15, 16, 17], [18, 19, 20], [21, 22, 23],
    [0, 9, 21], [3, 10, 18], [6, 11, 15], [1, 4, 7],
    [16, 19, 22], [8, 12, 17], [5, 13, 20], [2, 14, 23],
]

# Für jeden Punkt die Mühlen, die ihn enthalten (Vorberechnung).
_MILLS_AT = [[m for m in MILLS if p in m] for p in range(24)]


# =================================================================== Regel-Helfer
def _forms_mill(board, point, val):
    for m in _MILLS_AT[point]:
        if board[m[0]] == val and board[m[1]] == val and board[m[2]] == val:
            return True
    return False


def _removable(board, opp):
    """Entfernbare Gegnersteine: bevorzugt solche außerhalb von Mühlen."""
    in_mill = set()
    for m in MILLS:
        if board[m[0]] == opp and board[m[1]] == opp and board[m[2]] == opp:
            in_mill.update(m)
    opp_pts = [p for p in range(24) if board[p] == opp]
    free = [p for p in opp_pts if p not in in_mill]
    return free if free else opp_pts


def _count(board, val):
    return sum(1 for v in board if v == val)


def _gen_moves(board, placed, player, flying):
    """Vollständige Züge (inkl. evtl. Entfernen) für 'player' (0/1)."""
    val = player + 1
    opp = 2 if val == 1 else 1
    moves = []
    if placed[player] < 9:
        for p in range(24):
            if board[p] == 0:
                board[p] = val
                if _forms_mill(board, p, val):
                    for rp in _removable(board, opp):
                        moves.append(("place", p, None, rp))
                else:
                    moves.append(("place", p, None, None))
                board[p] = 0
    else:
        on = _count(board, val)
        fly = flying and on == 3
        for p in range(24):
            if board[p] != val:
                continue
            dests = range(24) if fly else ADJ[p]
            for d in dests:
                if board[d] == 0:
                    board[p] = 0
                    board[d] = val
                    if _forms_mill(board, d, val):
                        for rp in _removable(board, opp):
                            moves.append(("move", p, d, rp))
                    else:
                        moves.append(("move", p, d, None))
                    board[d] = 0
                    board[p] = val
    return moves


def _apply_move(board, placed, player, mv):
    val = player + 1
    nb = list(board)
    npl = list(placed)
    kind, a, b, rp = mv
    if kind == "place":
        nb[a] = val
        npl[player] += 1
    else:
        nb[a] = 0
        nb[b] = val
    if rp is not None:
        nb[rp] = 0
    return nb, npl


def _has_moves(board, placed, player, flying):
    if placed[player] < 9:
        return True
    val = player + 1
    on = _count(board, val)
    if flying and on == 3:
        return True
    for p in range(24):
        if board[p] == val:
            for d in ADJ[p]:
                if board[d] == 0:
                    return True
    return False


def _eval_side(board, val, placed, player, flying):
    mills = 0
    twos = 0
    for m in MILLS:
        vals = [board[m[0]], board[m[1]], board[m[2]]]
        cnt = vals.count(val)
        if cnt == 3:
            mills += 1
        elif cnt == 2 and vals.count(0) == 1:
            twos += 1
    mob = 0
    on = _count(board, val)
    if not (placed[player] < 9):
        if flying and on == 3:
            mob = 10
        else:
            for p in range(24):
                if board[p] == val:
                    for d in ADJ[p]:
                        if board[d] == 0:
                            mob += 1
    return on * 9 + mills * 6 + twos * 2 + mob


def _evaluate(board, placed, player, flying):
    """Positiv = gut für 'player'."""
    me = player
    opp = 1 - player
    return (_eval_side(board, me + 1, placed, me, flying)
            - _eval_side(board, opp + 1, placed, opp, flying))


class MuehleGame(Game):
    name = LocalizedName("Nine Men's Morris", de="Mühle", fr="Moulin",
                         es="Molino", pt="Trilha")
    highscore_key = "muehle"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        ms = self.settings.get("muehle", {}) if isinstance(self.settings, dict) else {}
        self.diff = max(0, min(2, int(ms.get("difficulty", 1))))
        self.flying = bool(ms.get("flying", True))

        self._make_fonts()
        self.wins = [0, 0]
        self.starter = 0
        self._build_setup_layout()
        self._new_round()
        self.state = PLAY if self.multiplayer else SETUP

    def _make_fonts(self):
        """Theme-Schriften, Grösse abhängig von der Fensterhöhe."""
        self._small = ui.font(max(14, self.height // 34))
        self._tiny = ui.font(max(12, self.height // 44))
        self._huge = ui.font(max(26, self.height // 12), bold=True)

    def on_surface_changed(self):
        self._make_fonts()
        self._build_setup_layout()
        self._layout()

    def _layout(self):
        self.hud_h = 46
        size = int(min(self.width - 60, self.height - self.hud_h - 40))
        self.bsize = size
        self.bx = (self.width - size) // 2
        self.by = self.hud_h + max(10, (self.height - self.hud_h - size) // 2)
        self.step = size / 6.0
        self.pr = max(9, int(self.step * 0.28))     # Steinradius
        self.pts = [(int(self.bx + col * self.step),
                     int(self.by + row * self.step)) for (col, row) in POS]
        # Grundplatte hinter dem Liniennetz (im Fenster/unter dem HUD halten)
        pad = max(10, min(int(self.step * 0.5), self.bx - 8,
                          self.by - self.hud_h - 8))
        self.plate_rect = pygame.Rect(self.bx - pad, self.by - pad,
                                      size + 2 * pad, size + 2 * pad)

    def _new_round(self):
        self.board = [0] * 24
        self.placed = [0, 0]
        self.player = self.starter
        self.remove_mode = False
        self.removable = []
        self.sel = None
        self.targets = []
        self.last_spot = None
        self.mill_spots = []
        self.mill_t = 0.0           # Restzeit der KI-Mühlen-Anzeige
        self.winner = None
        self.msg = None
        self.msg_t = 0.0
        self.ai_delay = 0.6
        self._layout()

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(360, self.width - 60)
        y0 = int(self.height * 0.30)
        self.diff_rects = [pygame.Rect(cx - bw // 2, y0 + i * 52, bw, 44)
                           for i in range(3)]
        self.fly_rect = pygame.Rect(cx - bw // 2, y0 + 3 * 52 + 8, bw, 44)
        self.start_rect = pygame.Rect(cx - 95, y0 + 4 * 52 + 22, 190, 46)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("muehle", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3"):
                self.diff = int(k) - 1
                self._save_setting("difficulty", self.diff)
                self.play_sound("click")
            elif k in ("Up", "w", "W"):
                self.diff = (self.diff - 1) % 3
                self._save_setting("difficulty", self.diff)
                self.play_sound("move")
            elif k in ("Down", "s", "S"):
                self.diff = (self.diff + 1) % 3
                self._save_setting("difficulty", self.diff)
                self.play_sound("move")
            elif k in ("f", "F"):
                self.flying = not self.flying
                self._save_setting("flying", self.flying)
                self.play_sound("select")
            elif k in ("Return", "space"):
                self._start_play()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.diff_rects):
                if rc.collidepoint(event.pos):
                    self.diff = i
                    self._save_setting("difficulty", i)
                    self.play_sound("click")
                    return
            if self.fly_rect.collidepoint(event.pos):
                self.flying = not self.flying
                self._save_setting("flying", self.flying)
                self.play_sound("select")
                return
            if self.start_rect.collidepoint(event.pos):
                self._start_play()

    def _start_play(self):
        self._new_round()
        self.state = PLAY
        self.play_sound("click")

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.state == OVER:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._restart()
                elif event.key in ("s", "S") and not self.multiplayer:
                    self.game_over = False
                    self.state = SETUP
                    self.play_sound("click")
            elif event.kind == InputEvent.MOUSEDOWN:
                self._restart()
            return
        if self.state != PLAY:
            return
        if not self._human_turn():
            return
        if event.kind == InputEvent.MOUSEDOWN:
            p = self._spot_at(event.pos)
            if p is not None:
                self._click_spot(p)
        elif event.kind == InputEvent.KEYDOWN and event.key in ("Escape",):
            self.sel = None
            self.targets = []

    def _human_turn(self):
        return self.multiplayer or self.player == 0

    def _spot_at(self, pos):
        for i, (x, y) in enumerate(self.pts):
            if (pos[0] - x) ** 2 + (pos[1] - y) ** 2 <= (self.pr + 6) ** 2:
                return i
        return None

    def _click_spot(self, p):
        val = self.player + 1
        opp = 2 if val == 1 else 1
        if self.remove_mode:
            if self.board[p] == opp and p in self.removable:
                self.board[p] = 0
                self.last_spot = p
                self.play_sound("hit")
                self.remove_mode = False
                self.removable = []
                self._end_turn()
            else:
                self.play_sound("click")
            return
        if self.placed[self.player] < 9:
            # Setz-Phase
            if self.board[p] == 0:
                self.board[p] = val
                self.placed[self.player] += 1
                self.last_spot = p
                self._after_action(p, val, opp)
            else:
                self.play_sound("click")
            return
        # Zieh-/Spring-Phase
        if self.board[p] == val:
            self.sel = p
            self.targets = self._move_targets(p, val)
            self.play_sound("click")
        elif self.sel is not None and p in self.targets:
            self.board[self.sel] = 0
            self.board[p] = val
            self.last_spot = p
            self.sel = None
            self.targets = []
            self._after_action(p, val, opp)
        else:
            self.sel = None
            self.targets = []

    def _move_targets(self, p, val):
        on = _count(self.board, val)
        if self.flying and on == 3:
            return [d for d in range(24) if self.board[d] == 0]
        return [d for d in ADJ[p] if self.board[d] == 0]

    def _after_action(self, p, val, opp):
        """Nach Setzen/Ziehen: Mühle? -> Entfernen, sonst Zugende."""
        if _forms_mill(self.board, p, val):
            self.mill_spots = [m for m in _MILLS_AT[p]
                               if all(self.board[q] == val for q in m)]
            self.mill_t = 0.0       # Anzeige gehört jetzt dem Menschen
            self.removable = _removable(self.board, opp)
            if self.removable:
                self.remove_mode = True
                self.play_sound("point")
                return
        self.play_sound("lock")
        self._end_turn()

    def _end_turn(self):
        self.mill_spots = []
        self.player = 1 - self.player
        self.sel = None
        self.targets = []
        self._check_state()
        if (self.state == PLAY and not self.multiplayer
                and self.player == 1):
            self.ai_delay = 0.45

    def _check_state(self):
        # Gewinn nur in der Zieh-Phase (beide fertig gesetzt).
        if self.placed[0] >= 9 and self.placed[1] >= 9:
            val = self.player + 1
            if _count(self.board, val) < 3:
                self.winner = 1 - self.player
                self._end()
                return
            if not _has_moves(self.board, self.placed, self.player, self.flying):
                self.winner = 1 - self.player
                self._end()
                return

    def _end(self):
        self.state = OVER
        self.wins[self.winner] += 1
        if not self.multiplayer:
            if self.winner == 0:
                self.score = self.wins[0]
                self.play_sound("win")
            else:
                self.play_sound("gameover")
        else:
            self.play_sound("win")
        self.game_over = True

    def _restart(self):
        self.starter = 1 - self.starter
        self.game_over = False
        self._new_round()
        self.state = PLAY
        self.play_sound("click")

    # ===================================================== KI
    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None
        if self.mill_t > 0:
            # KI-Mühle nur kurz anzeigen, dann wieder ausblenden.
            self.mill_t -= dt
            if self.mill_t <= 0:
                self.mill_t = 0.0
                if not self.remove_mode:
                    self.mill_spots = []
        if (self.state == PLAY and not self.multiplayer and self.player == 1):
            self.ai_delay -= dt
            if self.ai_delay <= 0:
                self._ai_play()

    def _ai_play(self):
        mv = self._pick_ai_move()
        if mv is None:
            self._check_state()
            return
        kind, a, b, rp = mv
        val = 2
        if kind == "place":
            self.board[a] = val
            self.placed[1] += 1
            self.last_spot = a
        else:
            self.board[a] = 0
            self.board[b] = val
            self.last_spot = b
        spot = a if kind == "place" else b
        if rp is not None:
            mills = [m for m in _MILLS_AT[spot]
                     if all(self.board[q] == val for q in m)]
            self.board[rp] = 0
            self.play_sound("hit")
            self._end_turn()
            # _end_turn() löscht mill_spots sofort - die geschlossene
            # KI-Mühle danach kurz anzeigen, sonst sieht man sie nie.
            self.mill_spots = mills
            self.mill_t = 1.2
        else:
            self.play_sound("lock")
            self._end_turn()

    def _pick_ai_move(self):
        moves = _gen_moves(self.board, self.placed, 1, self.flying)
        if not moves:
            return None
        if random.random() < (0.55 if self.diff == 0 else
                              0.18 if self.diff == 1 else 0.0):
            caps = [m for m in moves if m[3] is not None]
            if caps and random.random() < 0.7:
                return random.choice(caps)
            return random.choice(moves)
        depth = DEPTHS[self.diff]
        self._nodes = 0
        self._deadline = time.time() + TIME_BUDGET[self.diff]
        best_val = -10 ** 9
        best = []
        for mv in self._order(moves):
            nb, npl = _apply_move(self.board, self.placed, 1, mv)
            val = -self._search(nb, npl, 0, depth - 1, -10 ** 9, 10 ** 9)
            if best and time.time() > self._deadline:
                break
            if val > best_val:
                best_val = val
                best = [mv]
            elif val == best_val:
                best.append(mv)
        return random.choice(best) if best else random.choice(moves)

    def _order(self, moves):
        return sorted(moves, key=lambda m: 0 if m[3] is not None else 1)

    def _search(self, board, placed, player, depth, alpha, beta):
        self._nodes += 1
        if self._nodes > NODE_BUDGET or (self._nodes & 1023) == 0 \
                and time.time() > self._deadline:
            return _evaluate(board, placed, player, self.flying)
        # Verlust, wenn zugunfähig oder (in Zieh-Phase) unter 3 Steine.
        if placed[0] >= 9 and placed[1] >= 9 and _count(board, player + 1) < 3:
            return -20000 + (5 - depth)
        moves = _gen_moves(board, placed, player, self.flying)
        if not moves:
            return -20000 + (5 - depth)
        if depth <= 0:
            return _evaluate(board, placed, player, self.flying)
        best = -10 ** 9
        for mv in self._order(moves):
            nb, npl = _apply_move(board, placed, player, mv)
            val = -self._search(nb, npl, 1 - player, depth - 1, -beta, -alpha)
            if val > best:
                best = val
            if best > alpha:
                alpha = best
            if alpha >= beta:
                break
        return best

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height, stars=False, aurora=True)
        if self.state == SETUP:
            self._draw_setup(s)
            return
        if not hasattr(self, "pts"):
            self._layout()
        self._draw_hud(s)
        self._draw_board(s)
        if self.state == OVER:
            self._draw_over(s)

    def _draw_board(self, s):
        # Grundplatte, damit sich das Liniennetz vom Hintergrund abhebt
        pygame.draw.rect(s, COL_PLATE, self.plate_rect, border_radius=12)
        pygame.draw.rect(s, ui.mix(COL_PLATE, self.accent, 0.45),
                         self.plate_rect, 1, border_radius=12)
        # Linien (jede Kante einmal)
        for i in range(24):
            for j in ADJ[i]:
                if j > i:
                    pygame.draw.line(s, COL_LINE, self.pts[i], self.pts[j], 3)
        # Mühlen-Hervorhebung
        for m in self.mill_spots:
            pygame.draw.line(s, COL_MILL, self.pts[m[0]], self.pts[m[2]], 5)
        # Punkte + Steine
        human = self.state == PLAY and self._human_turn()
        for i in range(24):
            x, y = self.pts[i]
            v = self.board[i]
            if v == 0:
                pygame.draw.circle(s, COL_SPOT, (x, y), max(4, self.pr // 2))
            else:
                base = COL_P1 if v == 1 else COL_P2
                hi = COL_P1_HI if v == 1 else COL_P2_HI
                pygame.draw.circle(s, (10, 16, 12), (x, y + 2), self.pr)
                pygame.draw.circle(s, base, (x, y), self.pr)
                pygame.draw.circle(s, hi, (x - self.pr // 3, y - self.pr // 3),
                                   max(2, self.pr // 4))
        # Setz-Hinweise (freie Punkte) in der Setz-Phase
        if human and not self.remove_mode and self.placed[self.player] < 9:
            for i in range(24):
                if self.board[i] == 0:
                    pygame.draw.circle(s, COL_HINT, self.pts[i],
                                       max(3, self.pr // 4))
        # Auswahl + Zugziele
        if human and self.sel is not None:
            pygame.draw.circle(s, COL_SEL, self.pts[self.sel], self.pr + 3, 3)
            for d in self.targets:
                pygame.draw.circle(s, COL_HINT, self.pts[d],
                                   max(4, self.pr // 3))
        # Entfern-Markierung
        if human and self.remove_mode:
            k = ui.pulse(2.8, 0.0, 1.0)
            for i in self.removable:
                pygame.draw.circle(s, COL_REMOVE, self.pts[i],
                                   self.pr + 2 + int(3 * k), 3)
        # Letzter Punkt (sanft pulsierender Akzentring)
        if self.last_spot is not None and self.board[self.last_spot] != 0:
            col = ui.mix(COL_SPOT, self.accent,
                         0.55 + 0.35 * ui.pulse(1.6, 0.0, 1.0))
            pygame.draw.circle(s, col, self.pts[self.last_spot],
                               self.pr + 4, 2)

    def _draw_hud(self, s):
        panel = pygame.Rect(8, 6, self.width - 16, self.hud_h - 10)
        ui.draw_panel(s, panel, shadow=False, accent_top=self.accent)
        cy = panel.centery
        # Steinbestand: verbleibend zu setzen + auf dem Brett
        for idx, col in enumerate((COL_P1, COL_P2)):
            on = _count(self.board, idx + 1)
            left = 9 - self.placed[idx]
            txt = f"{on}" + (f" (+{left})" if left else "")
            img = self._small.render(txt, True, ui.TEXT)
            if idx == 0:
                pygame.draw.circle(s, col, (panel.x + 16, cy), 8)
                pygame.draw.circle(s, ui.BORDER_LIGHT, (panel.x + 16, cy), 8, 1)
                s.blit(img, img.get_rect(midleft=(panel.x + 30, cy)))
            else:
                pygame.draw.circle(s, col, (panel.right - 16, cy), 8)
                pygame.draw.circle(s, ui.BORDER_LIGHT, (panel.right - 16, cy), 8, 1)
                s.blit(img, img.get_rect(midright=(panel.right - 30, cy)))
        if self.state == PLAY:
            if self.remove_mode and self._human_turn():
                mid = t("mill.remove")
            elif not self.multiplayer and self.player == 1:
                mid = t("mill.ai_thinks")
            elif self.placed[self.player] < 9:
                if self.multiplayer:
                    who = t("common.player1") if self.player == 0 else t("common.player2")
                    mid = t("mill.place_turn", name=who)
                else:
                    mid = t("mill.place_you")
            else:
                if self.multiplayer:
                    who = t("common.player1") if self.player == 0 else t("common.player2")
                    mid = t("mill.move_turn", name=who)
                else:
                    mid = t("mill.move_you")
            img = self._small.render(mid, True, self.accent)
            s.blit(img, img.get_rect(center=(self.width // 2, cy)))

    def _draw_over(self, s):
        cx = self.width // 2
        if self.multiplayer:
            head = self._huge.render(t("common.player_wins", n=self.winner + 1),
                                     True, self.accent)
        else:
            won = self.winner == 0
            head = self._huge.render(t("mill.win_you") if won else t("mill.win_ai"),
                                     True, self.accent if won else ui.TEXT_DIM)
        hint_key = "common.enter_restart" if self.multiplayer else "mill.new_round"
        hint = self._tiny.render(t(hint_key), True, ui.TEXT_DIM)
        w = min(self.width - 24, max(head.get_width(), hint.get_width()) + 64)
        panel = pygame.Rect(cx - w // 2, self.height // 2 - 48, w, 96)
        ui.draw_panel(s, panel, shadow=False, accent_top=self.accent)
        s.blit(head, head.get_rect(center=(cx, panel.y + 36)))
        s.blit(hint, hint.get_rect(center=(cx, panel.y + 74)))

    def _draw_setup(self, s):
        ui.draw_title(s, self.width, t("mill.title"),
                      subtitle=t("mill.subtitle"), y=int(self.height * 0.14),
                      big=self._huge, small=self._small, accent=self.accent)
        for i, rc in enumerate(self.diff_rects):
            ui.draw_button(s, rc, t("mill.diff." + DIFFS[i]), self.font,
                           selected=(i == self.diff), accent=self.accent)
        # Fliegen-Schalter
        state = t("common.on") if self.flying else t("common.off")
        ui.draw_button(s, self.fly_rect, t("mill.flying") + ": " + state,
                       self.font, selected=self.flying, accent=self.accent)
        ui.draw_button(s, self.start_rect, t("common.start"), self.font,
                       selected=True, accent=self.accent)
        ui.draw_footer(s, self.width, self.height, t("mill.setup_hint"),
                       self._tiny)
