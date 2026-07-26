# -*- coding: utf-8 -*-
"""
dame.py
=======
Dame / Checkers - drei Regelwerke, 1 Spieler gegen KI oder 2 Spieler lokal.

Waehlbare Varianten (im Setup-Screen):
- Deutsche Dame (8x8): 12 Steine je Seite; Maenner ziehen 1 diagonal vorwaerts,
  SCHLAGEN aber vor- UND rueckwaerts; die Dame FLIEGT beliebig weit diagonal.
  Schlagzwang, Mehrfachschlag - das Maximum muss aber NICHT genommen werden.
- Internationale Dame (10x10): 20 Steine je Seite; Maenner schlagen vor/rueckwaerts,
  fliegende Damen. Schlagzwang MIT Maximum-Regel (laengste Schlagfolge ist Pflicht).
- Checkers 8x8 (englisch): Maenner schlagen NUR vorwaerts, die Dame (King) zieht
  nur EIN Feld (nicht fliegend). Schlagzwang, kein Maximum.

Gemeinsam: Steine stehen auf den dunklen Feldern. Wer keinen Zug mehr hat (kein
Stein oder eingeschlossen), verliert. Der Mehrfachschlag muss vollstaendig
ausgefuehrt werden (Schlagzwang). Ein Mann, der beim Schlagen die letzte Reihe
erreicht, wird zur Dame und die Schlagfolge endet dort.

Einzelspieler: KI mit Minimax + Alpha-Beta (Material + Vormarsch), drei Staerken.
Punkte (Highscore) = Siege gegen die KI in dieser Sitzung; Mehrspieler wird nicht
gewertet.

Steuerung: Stein anklicken, dann Zielfeld(er). Mehrfachsprung Schritt fuer Schritt
anklicken. Tastatur: Pfeile/WASD bewegen den Rahmen, Leertaste/Enter waehlt.
Nach Rundenende: Enter = neue Runde, S = Setup (Variantenwahl, beide Modi).
"""

import random

import pygame

import settings as settings_mod
import ui
from game_base import Game, InputEvent, LocalizedName
from i18n import t

# ---- Feldkodierung
EMPTY = 0
P0_MAN, P0_KING = 1, 2      # Spieler 0 (unten, hell) - zieht nach oben (row -)
P1_MAN, P1_KING = 3, 4      # Spieler 1 (oben, dunkel) - zieht nach unten (row +)

DIRS4 = [(-1, -1), (-1, 1), (1, -1), (1, 1)]

# ---- Brett-Identitätsfarben (generische UI-Farben liefert die ui-Palette)
COL_LIGHT = (222, 196, 150)     # helle (unbespielte) Felder
COL_DARK = (120, 84, 52)        # dunkle (bespielte) Felder
COL_PLATE = (40, 28, 20)        # Brettrahmen
COL_P0 = (236, 230, 214)        # Steine Spieler 0 (creme)
COL_P0_HI = (255, 255, 250)
COL_P1 = (188, 60, 60)          # Steine Spieler 1 (rot)
COL_P1_HI = (232, 120, 120)
COL_KING = (245, 205, 90)       # Krone/Ring der Dame
COL_HINT = (250, 236, 150)      # Auswahl-/Zughinweise auf dem Holzbrett
COL_CAP = (232, 90, 80)         # Schlagzwang-/Schlag-Markierungen

VARIANTS = ["german", "international", "checkers"]
DIFFS = ["easy", "medium", "hard"]

SETUP, PLAY, OVER = "setup", "play", "over"


def owner(code):
    if code == EMPTY:
        return None
    return 0 if code <= P0_KING else 1


def is_king(code):
    return code in (P0_KING, P1_KING)


def make_code(side, king):
    if side == 0:
        return P0_KING if king else P0_MAN
    return P1_KING if king else P1_MAN


def variant_flags(variant, size_override=None):
    """Regel-Flags je Variante."""
    if variant == "german":
        return dict(size=8, men_back=True, flying=True, maximum=False)
    if variant == "international":
        return dict(size=10, men_back=True, flying=True, maximum=True)
    # checkers (englisch)
    return dict(size=8, men_back=False, flying=False, maximum=False)


def initial_board(flags):
    size = flags["size"]
    rows = 3 if size == 8 else 4
    b = [[EMPTY] * size for _ in range(size)]
    for r in range(size):
        for c in range(size):
            if (r + c) % 2 != 1:            # nur dunkle Felder
                continue
            if r < rows:
                b[r][c] = P1_MAN            # oben = Spieler 1
            elif r >= size - rows:
                b[r][c] = P0_MAN            # unten = Spieler 0
    return b


def _in(r, c, size):
    return 0 <= r < size and 0 <= c < size


def _back_rank(row, side, size):
    return row == 0 if side == 0 else row == size - 1


# ---------------------------------------------------------------------------
#  Zug- und Schlaggenerierung
# ---------------------------------------------------------------------------

def _single_jumps(b, r, c, side, king, captured, flags):
    """Ein-Schritt-Schlaege von (r,c) auf Brett b. Liefert [(cap_pos, land_pos)]."""
    size = flags["size"]
    opp = 1 - side
    out = []
    if not king:
        dirs = DIRS4 if flags["men_back"] else \
            ([(-1, -1), (-1, 1)] if side == 0 else [(1, -1), (1, 1)])
        for dr, dc in dirs:
            mr, mc, lr, lc = r + dr, c + dc, r + 2 * dr, c + 2 * dc
            if _in(lr, lc, size) and owner(b[mr][mc]) == opp \
                    and (mr, mc) not in captured and b[lr][lc] == EMPTY:
                out.append(((mr, mc), (lr, lc)))
        return out
    # Dame
    if flags["flying"]:
        for dr, dc in DIRS4:
            i = 1
            while _in(r + dr * i, c + dc * i, size) and b[r + dr * i][c + dc * i] == EMPTY:
                i += 1
            er, ec = r + dr * i, c + dc * i
            if _in(er, ec, size) and owner(b[er][ec]) == opp and (er, ec) not in captured:
                j = i + 1
                while _in(r + dr * j, c + dc * j, size) and b[r + dr * j][c + dc * j] == EMPTY:
                    out.append(((er, ec), (r + dr * j, c + dc * j)))
                    j += 1
    else:
        for dr, dc in DIRS4:
            mr, mc, lr, lc = r + dr, c + dc, r + 2 * dr, c + 2 * dc
            if _in(lr, lc, size) and owner(b[mr][mc]) == opp \
                    and (mr, mc) not in captured and b[lr][lc] == EMPTY:
                out.append(((mr, mc), (lr, lc)))
    return out


def _capture_sequences(board, r, c, flags):
    """Alle vollstaendigen Schlagfolgen des Steins auf (r,c)."""
    code = board[r][c]
    side = owner(code)
    king0 = is_king(code)
    size = flags["size"]
    results = []

    def rec(b, cr, cc, cur_king, captured, path):
        jumps = _single_jumps(b, cr, cc, side, cur_king, captured, flags)
        if not jumps:
            return False
        for cap, (lr, lc) in jumps:
            nb = [row[:] for row in b]
            nb[cr][cc] = EMPTY
            promoted = (not cur_king) and _back_rank(lr, side, size)
            nk = cur_king or promoted
            nb[lr][lc] = make_code(side, nk)
            ncap = set(captured)
            ncap.add(cap)
            npath = path + [(lr, lc)]
            if promoted:
                results.append((npath, ncap))          # Mann wird Dame -> Ende
            elif not rec(nb, lr, lc, nk, ncap, npath):
                results.append((npath, ncap))           # keine Fortsetzung
        return True

    b0 = [row[:] for row in board]
    rec(b0, r, c, king0, set(), [(r, c)])
    return results


def _simple_moves(board, r, c, flags):
    code = board[r][c]
    side = owner(code)
    king = is_king(code)
    size = flags["size"]
    out = []
    if not king:
        fdir = -1 if side == 0 else 1
        for dc in (-1, 1):
            lr, lc = r + fdir, c + dc
            if _in(lr, lc, size) and board[lr][lc] == EMPTY:
                out.append(((r, c), (lr, lc)))
    elif flags["flying"]:
        for dr, dc in DIRS4:
            i = 1
            while _in(r + dr * i, c + dc * i, size) and board[r + dr * i][c + dc * i] == EMPTY:
                out.append(((r, c), (r + dr * i, c + dc * i)))
                i += 1
    else:
        for dr, dc in DIRS4:
            lr, lc = r + dr, c + dc
            if _in(lr, lc, size) and board[lr][lc] == EMPTY:
                out.append(((r, c), (lr, lc)))
    return out


def legal_moves(board, side, flags):
    """(Liste Zuege, forced) - bei Schlagzwang nur Schlagzuege.

    Ein Zug: dict(path=[(r,c),...], caps=frozenset(), start, end)."""
    caps = []
    size = flags["size"]
    for r in range(size):
        for c in range(size):
            if owner(board[r][c]) == side:
                for path, capset in _capture_sequences(board, r, c, flags):
                    caps.append(dict(path=path, caps=frozenset(capset),
                                     start=path[0], end=path[-1]))
    if caps:
        if flags["maximum"]:
            m = max(len(x["caps"]) for x in caps)
            caps = [x for x in caps if len(x["caps"]) == m]
        return caps, True
    simples = []
    for r in range(size):
        for c in range(size):
            if owner(board[r][c]) == side:
                for st, en in _simple_moves(board, r, c, flags):
                    simples.append(dict(path=[st, en], caps=frozenset(),
                                        start=st, end=en))
    return simples, False


def apply_move(board, move, flags):
    b = [row[:] for row in board]
    sr, sc = move["start"]
    er, ec = move["end"]
    code = b[sr][sc]
    side = owner(code)
    king = is_king(code)
    b[sr][sc] = EMPTY
    for cr, cc in move["caps"]:
        b[cr][cc] = EMPTY
    if not king and _back_rank(er, side, flags["size"]):
        king = True
    b[er][ec] = make_code(side, king)
    return b


def _count(board):
    a = bcnt = 0
    for row in board:
        for v in row:
            o = owner(v)
            if o == 0:
                a += 1
            elif o == 1:
                bcnt += 1
    return a, bcnt


class DameGame(Game):
    name = LocalizedName("Checkers", de="Dame", fr="Jeu de dames",
                         es="Damas", pt="Damas")
    highscore_key = "dame"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        dv = self.settings.get("dame", {}) if isinstance(self.settings, dict) else {}
        self.variant = dv.get("variant", "german")
        if self.variant not in VARIANTS:
            self.variant = "german"
        self.diff = max(0, min(2, int(dv.get("difficulty", 1))))

        self._make_fonts()
        self.wins = [0, 0]
        self.starter = 0
        self._build_setup_layout()
        self._new_round()
        self.state = SETUP

    def _make_fonts(self):
        """Theme-Schriften, Grösse abhängig von der Fensterhöhe."""
        self._small = ui.font(max(14, self.height // 32))
        self._tiny = ui.font(max(12, self.height // 40))
        self._huge = ui.font(max(26, self.height // 11), bold=True)

    def on_surface_changed(self):
        self._make_fonts()
        self._build_setup_layout()
        self._layout()

    def _layout(self):
        n = self.flags["size"]
        self.hud_h = 46
        self.cell = int(min((self.width - 40) / n,
                            (self.height - self.hud_h - 24) / n))
        self.bw = n * self.cell
        self.bh = n * self.cell
        self.bx = (self.width - self.bw) // 2
        self.by = self.hud_h + max(8, (self.height - self.hud_h - self.bh) // 2)
        self.pr = int(self.cell * 0.38)

    def _new_round(self):
        self.flags = variant_flags(self.variant)
        n = self.flags["size"]
        self.depths = [1, 3, 4] if n == 8 else [1, 2, 3]
        self.board = initial_board(self.flags)
        self.player = self.starter
        self.cursor = [n // 2, n // 2 - 1 if (n // 2) % 2 == 0 else n // 2]
        self.moves, self.forced = legal_moves(self.board, self.player, self.flags)
        self.sel = None
        self.partial = []
        self.step_options = set()
        self.last_move = None
        self.winner = None
        self.ai_delay = 0.0
        self.msg = None
        self.msg_t = 0.0
        self._layout()

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(380, self.width - 60)
        y0 = int(self.height * 0.26)
        self.var_rects = [pygame.Rect(cx - bw // 2, y0 + i * 50, bw, 42)
                          for i in range(3)]
        y1 = y0 + 3 * 50 + 18
        dw = min(120, (bw - 20) // 3)
        gap = (bw - 3 * dw) // 2
        self.diff_rects = [pygame.Rect(cx - bw // 2 + i * (dw + gap), y1, dw, 40)
                           for i in range(3)]
        self.start_rect = pygame.Rect(cx - 95, y1 + 60, 190, 46)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("dame", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3"):
                self._pick_variant(int(k) - 1)
            elif k in ("Up", "w", "W"):
                self._pick_variant((VARIANTS.index(self.variant) - 1) % 3)
            elif k in ("Down", "s", "S"):
                self._pick_variant((VARIANTS.index(self.variant) + 1) % 3)
            elif k in ("Left", "a", "A") and not self.multiplayer:
                self.diff = (self.diff - 1) % 3
                self._save_setting("difficulty", self.diff)
                self.play_sound("move")
            elif k in ("Right", "d", "D") and not self.multiplayer:
                self.diff = (self.diff + 1) % 3
                self._save_setting("difficulty", self.diff)
                self.play_sound("move")
            elif k in ("Return", "space"):
                self._start_game()
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.var_rects):
                if rc.collidepoint(event.pos):
                    self._pick_variant(i)
                    return
            if not self.multiplayer:
                for i, rc in enumerate(self.diff_rects):
                    if rc.collidepoint(event.pos):
                        self.diff = i
                        self._save_setting("difficulty", i)
                        self.play_sound("click")
                        return
            if self.start_rect.collidepoint(event.pos):
                self._start_game()

    def _pick_variant(self, i):
        self.variant = VARIANTS[i]
        self._save_setting("variant", self.variant)
        self.play_sound("click")

    def _start_game(self):
        self._new_round()
        self.state = PLAY
        self.play_sound("select")

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.state == OVER:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._restart()
                elif event.key in ("s", "S"):
                    # Setup (Variantenwahl) gibt es hier auch im Mehrspieler -
                    # der Hinweis "S = Setup" gilt daher für beide Modi.
                    self.game_over = False
                    self.state = SETUP
                    self.play_sound("click")
            elif event.kind == InputEvent.MOUSEDOWN:
                self._restart()
            return
        if self.state != PLAY:
            return
        if not self.multiplayer and self.player == 1:
            return                                  # KI ist am Zug
        if event.kind == InputEvent.MOUSEMOVE:
            rc = self._cell_at(event.pos)
            if rc:
                self.cursor = list(rc)
        elif event.kind == InputEvent.MOUSEDOWN:
            rc = self._cell_at(event.pos)
            if rc:
                self.cursor = list(rc)
                self._choose(tuple(rc))
        elif event.kind == InputEvent.KEYDOWN:
            n = self.flags["size"]
            k = event.key
            if self.is_action(k, "up") or k == "Up":
                self.cursor[0] = (self.cursor[0] - 1) % n
            elif self.is_action(k, "down") or k == "Down":
                self.cursor[0] = (self.cursor[0] + 1) % n
            elif self.is_action(k, "left") or k == "Left":
                self.cursor[1] = (self.cursor[1] - 1) % n
            elif self.is_action(k, "right") or k == "Right":
                self.cursor[1] = (self.cursor[1] + 1) % n
            elif self.is_action(k, "action") or k in ("space", "Return"):
                self._choose((self.cursor[0], self.cursor[1]))
                return
            else:
                return
            self.play_sound("move")

    def _cell_at(self, pos):
        c = (pos[0] - self.bx) // self.cell
        r = (pos[1] - self.by) // self.cell
        n = self.flags["size"]
        if _in(int(r), int(c), n):
            return (int(r), int(c))
        return None

    def _start_squares(self):
        return {m["start"] for m in self.moves}

    def _choose(self, rc):
        """Zentraler Auswahl-Handler fuer Maus und Tastatur (Mehrfachsprung)."""
        if self.sel is None:
            if rc in self._start_squares():
                self.sel = rc
                self.partial = [rc]
                self._recompute_options()
                self.play_sound("select")
            else:
                self.play_sound("click")
            return
        # Es ist ein Stein gewaehlt -> naechster Schritt?
        pmoves = [m for m in self.moves if m["start"] == self.sel]
        depth = len(self.partial)
        cand = [m for m in pmoves if len(m["path"]) > depth
                and m["path"][:depth] == self.partial and m["path"][depth] == rc]
        if not cand:
            # Umwahl oder Abwahl
            if rc == self.sel and depth == 1:
                self.sel = None
                self.partial = []
                self.step_options = set()
                self.play_sound("click")
            elif rc in self._start_squares():
                self.sel = rc
                self.partial = [rc]
                self._recompute_options()
                self.play_sound("select")
            else:
                self.play_sound("click")
            return
        self.partial.append(rc)
        complete = [m for m in cand if len(m["path"]) == len(self.partial)]
        if complete:
            self._do_move(complete[0])
        else:
            self._recompute_options()
            self.play_sound("lock")

    def _recompute_options(self):
        pmoves = [m for m in self.moves if m["start"] == self.sel]
        depth = len(self.partial)
        self.step_options = {m["path"][depth] for m in pmoves
                             if len(m["path"]) > depth
                             and m["path"][:depth] == self.partial}

    def _do_move(self, move):
        promoted = self._is_promotion(move)
        self.board = apply_move(self.board, move, self.flags)
        self.last_move = move
        self.sel = None
        self.partial = []
        self.step_options = set()
        if move["caps"]:
            self.play_sound("lock")
        else:
            self.play_sound("move")
        if promoted:
            self.play_sound("powerup")
        self._advance_turn()

    def _is_promotion(self, move):
        sr, sc = move["start"]
        code = self.board[sr][sc]
        if is_king(code):
            return False
        return _back_rank(move["end"][0], owner(code), self.flags["size"])

    def _advance_turn(self):
        other = 1 - self.player
        moves, forced = legal_moves(self.board, other, self.flags)
        if not moves:
            self._ende(self.player)              # Gegner kann nicht ziehen
            return
        self.player = other
        self.moves = moves
        self.forced = forced
        self.sel = None
        self.partial = []
        self.step_options = set()
        if not self.multiplayer and self.player == 1:
            self.ai_delay = 0.45

    def _restart(self):
        self.starter = 1 - self.starter
        self.game_over = False
        self._new_round()
        self.state = PLAY
        self.play_sound("click")

    def _ende(self, winner):
        self.winner = winner
        self.wins[winner] += 1
        self.state = OVER
        if not self.multiplayer:
            if winner == 0:
                self.score = self.wins[0]
                self.play_sound("win")
                self.report_result(True)
            else:
                self.play_sound("gameover")
                self.report_result(False)
        else:
            self.play_sound("win")
        self.game_over = True                    # main.py sichert den Score

    # ===================================================== KI
    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None
        if self.state == PLAY and not self.multiplayer and self.player == 1:
            self.ai_delay -= dt
            if self.ai_delay <= 0:
                self._ai_play()

    def _ai_play(self):
        move = self._ai_choose()
        if move is None:
            self._ende(0)
            return
        promoted = self._is_promotion(move)
        self.board = apply_move(self.board, move, self.flags)
        self.last_move = move
        if move["caps"]:
            self.play_sound("lock")
        else:
            self.play_sound("move")
        if promoted:
            self.play_sound("powerup")
        self._advance_turn()

    def _ai_choose(self):
        moves = self.moves
        if not moves:
            return None
        if self.diff == 0 and random.random() < 0.55:
            return random.choice(moves)
        depth = self.depths[self.diff]
        best = -1e18
        best_moves = []
        for m in moves:
            nb = apply_move(self.board, m, self.flags)
            val = self._search(nb, 0, depth - 1, -1e18, 1e18)
            if val > best:
                best = val
                best_moves = [m]
            elif val == best:
                best_moves.append(m)
        if self.diff == 1 and len(moves) > 1 and random.random() < 0.2:
            return random.choice(moves)
        return random.choice(best_moves)

    def _search(self, board, side, depth, alpha, beta):
        """Minimax aus Sicht der KI (Spieler 1 maximiert)."""
        moves, _ = legal_moves(board, side, self.flags)
        if not moves:
            return -1e9 if side == 1 else 1e9       # wer nicht ziehen kann, verliert
        if depth <= 0:
            return self._evaluate(board)
        if side == 1:
            best = -1e18
            for m in moves:
                nb = apply_move(board, m, self.flags)
                best = max(best, self._search(nb, 0, depth - 1, alpha, beta))
                alpha = max(alpha, best)
                if alpha >= beta:
                    break
            return best
        best = 1e18
        for m in moves:
            nb = apply_move(board, m, self.flags)
            best = min(best, self._search(nb, 1, depth - 1, alpha, beta))
            beta = min(beta, best)
            if alpha >= beta:
                break
        return best

    def _evaluate(self, board):
        size = self.flags["size"]
        score = 0
        for r in range(size):
            for c in range(size):
                code = board[r][c]
                if code == EMPTY:
                    continue
                o = owner(code)
                val = 300 if is_king(code) else 100
                if not is_king(code):
                    val += (r * 4) if o == 1 else ((size - 1 - r) * 4)
                if c == 0 or c == size - 1:
                    val += 6
                score += val if o == 1 else -val
        return score

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height, stars=False, aurora=True)
        if self.state == SETUP:
            self._draw_setup(s)
            return
        if not hasattr(self, "cell"):
            self._layout()
        self._draw_hud(s)
        self._draw_board(s)
        if self.state == OVER:
            self._draw_over(s)

    def _draw_board(self, s):
        n = self.flags["size"]
        plate = pygame.Rect(self.bx - 8, self.by - 8, self.bw + 16, self.bh + 16)
        pygame.draw.rect(s, COL_PLATE, plate, border_radius=10)
        pygame.draw.rect(s, ui.mix(COL_PLATE, self.accent, 0.45), plate, 1,
                         border_radius=10)
        for r in range(n):
            for c in range(n):
                x = self.bx + c * self.cell
                y = self.by + r * self.cell
                dark = (r + c) % 2 == 1
                pygame.draw.rect(s, COL_DARK if dark else COL_LIGHT,
                                 (x, y, self.cell, self.cell))

        human_turn = self.state == PLAY and (self.multiplayer or self.player == 0)

        # Geschlagene Steine des letzten Zuges dezent markieren (schon entfernt)
        if self.last_move:
            for (cr, cc) in self.last_move["caps"]:
                x = self.bx + cc * self.cell
                y = self.by + cr * self.cell
                pygame.draw.line(s, COL_CAP, (x + 6, y + 6),
                                 (x + self.cell - 6, y + self.cell - 6), 2)
                pygame.draw.line(s, COL_CAP, (x + self.cell - 6, y + 6),
                                 (x + 6, y + self.cell - 6), 2)
            last_col = ui.mix(COL_DARK, self.accent,
                              0.55 + 0.35 * ui.pulse(1.6, 0.0, 1.0))
            for pos in (self.last_move["start"], self.last_move["end"]):
                x = self.bx + pos[1] * self.cell
                y = self.by + pos[0] * self.cell
                pygame.draw.rect(s, last_col, (x, y, self.cell, self.cell), 2)

        # waehlbare Steine (Schlagzwang: nur diese) hervorheben
        if human_turn and self.sel is None:
            for (sr, sc) in self._start_squares():
                cx = self.bx + sc * self.cell + self.cell // 2
                cy = self.by + sr * self.cell + self.cell // 2
                col = COL_CAP if self.forced else COL_HINT
                pygame.draw.circle(s, col, (cx, cy), self.pr + 3, 2)

        # gewaehlter Stein + moegliche Schritte
        if human_turn and self.sel is not None:
            cur = self.partial[-1]
            x = self.bx + cur[1] * self.cell
            y = self.by + cur[0] * self.cell
            pygame.draw.rect(s, COL_HINT, (x, y, self.cell, self.cell), 3)
            for (dr, dc) in self.step_options:
                cx = self.bx + dc * self.cell + self.cell // 2
                cy = self.by + dr * self.cell + self.cell // 2
                pygame.draw.circle(s, COL_HINT, (cx, cy), max(4, self.pr // 3))

        # Steine
        for r in range(n):
            for c in range(n):
                v = self.board[r][c]
                if v == EMPTY:
                    continue
                self._draw_piece(s, r, c, v)

        # Cursor
        if human_turn:
            r, c = self.cursor
            x = self.bx + c * self.cell
            y = self.by + r * self.cell
            k = ui.pulse(2.2, 0.0, 1.0)
            pygame.draw.rect(s, (int(150 + 100 * k),) * 3,
                             (x + 1, y + 1, self.cell - 2, self.cell - 2), 2)

    def _draw_piece(self, s, r, c, v):
        cx = self.bx + c * self.cell + self.cell // 2
        cy = self.by + r * self.cell + self.cell // 2
        o = owner(v)
        base = COL_P0 if o == 0 else COL_P1
        hi = COL_P0_HI if o == 0 else COL_P1_HI
        dark = tuple(int(x * 0.55) for x in base)
        pygame.draw.circle(s, (10, 8, 6), (cx, cy + 2), self.pr)
        pygame.draw.circle(s, dark, (cx, cy), self.pr)
        pygame.draw.circle(s, base, (cx, cy), int(self.pr * 0.86))
        pygame.draw.circle(s, hi, (cx - self.pr // 3, cy - self.pr // 3),
                           max(2, self.pr // 5))
        if is_king(v):
            pygame.draw.circle(s, COL_KING, (cx, cy), int(self.pr * 0.5), 2)
            # kleine Krone
            kr = int(self.pr * 0.34)
            pts = [(cx - kr, cy + kr // 2), (cx - kr, cy - kr // 2),
                   (cx - kr // 2, cy), (cx, cy - kr), (cx + kr // 2, cy),
                   (cx + kr, cy - kr // 2), (cx + kr, cy + kr // 2)]
            pygame.draw.polygon(s, COL_KING, pts)

    def _draw_hud(self, s):
        panel = pygame.Rect(8, 6, self.width - 16, self.hud_h - 10)
        ui.draw_panel(s, panel, shadow=False, accent_top=self.accent)
        cy = panel.centery
        a, b = _count(self.board)
        pygame.draw.circle(s, COL_P0, (panel.x + 18, cy), 9)
        pygame.draw.circle(s, ui.BORDER_LIGHT, (panel.x + 18, cy), 9, 1)
        img = self._small.render(str(a), True, ui.TEXT)
        s.blit(img, img.get_rect(midleft=(panel.x + 34, cy)))
        pygame.draw.circle(s, COL_P1, (panel.right - 18, cy), 9)
        pygame.draw.circle(s, ui.BORDER_LIGHT, (panel.right - 18, cy), 9, 1)
        img = self._small.render(str(b), True, ui.TEXT)
        s.blit(img, img.get_rect(midright=(panel.right - 34, cy)))

        if self.state == PLAY:
            if not self.multiplayer and self.player == 1:
                mid = t("dame.ai_thinks")
            elif not self.multiplayer:
                mid = t("dame.your_turn")
            else:
                who = t("common.player1") if self.player == 0 else t("common.player2")
                mid = t("dame.turn", name=who)
            img = self._small.render(mid, True, self.accent)
            s.blit(img, img.get_rect(center=(self.width // 2, cy)))
            if self.forced:
                fi = self._tiny.render(t("dame.must_capture"), True, ui.RED)
                s.blit(fi, fi.get_rect(center=(self.width // 2, self.hud_h + 11)))

    def _draw_over(self, s):
        cx = self.width // 2
        if self.multiplayer:
            head = self._huge.render(t("common.player_wins", n=self.winner + 1),
                                     True, COL_P0 if self.winner == 0 else COL_P1)
        else:
            key = "dame.win_you" if self.winner == 0 else "dame.win_ai"
            head = self._huge.render(t(key), True,
                                     self.accent if self.winner == 0
                                     else ui.TEXT_DIM)
        hint = self._tiny.render(t("dame.new_round"), True, ui.TEXT_DIM)
        w = min(self.width - 24, max(head.get_width(), hint.get_width()) + 64)
        panel = pygame.Rect(cx - w // 2, self.height // 2 - 48, w, 96)
        ui.draw_panel(s, panel, shadow=False, accent_top=self.accent)
        s.blit(head, head.get_rect(center=(cx, panel.y + 36)))
        s.blit(hint, hint.get_rect(center=(cx, panel.y + 74)))

    # ----- Setup zeichnen -----------------------------------------------
    def _draw_setup(self, s):
        cx = self.width // 2
        ui.draw_title(s, self.width, t("dame.title"), subtitle=t("dame.variant"),
                      y=int(self.height * 0.13), big=self._huge,
                      small=self._small, accent=self.accent)
        for i, rc in enumerate(self.var_rects):
            ui.draw_button(s, rc, t("dame.var." + VARIANTS[i]), self._small,
                           selected=(VARIANTS[i] == self.variant),
                           accent=self.accent)
        if not self.multiplayer:
            dl = self._tiny.render(t("dame.difficulty"), True, ui.TEXT_DIM)
            s.blit(dl, dl.get_rect(midbottom=(cx, self.diff_rects[0].y - 4)))
            for i, rc in enumerate(self.diff_rects):
                ui.draw_button(s, rc, t("dame.diff." + DIFFS[i]), self._tiny,
                               selected=(i == self.diff), accent=self.accent)
        ui.draw_button(s, self.start_rect, t("common.start"), self._small,
                       selected=True, accent=self.accent)
        ui.draw_footer(s, self.width, self.height, t("dame.setup_hint"),
                       self._tiny)
