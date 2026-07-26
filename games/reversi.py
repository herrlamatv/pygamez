# -*- coding: utf-8 -*-
"""
reversi.py
==========
Reversi (Othello) - 1 Spieler gegen KI oder 2 Spieler lokal.

- Klassisches 8x8-Brett mit der Standard-Startstellung (vier Steine im Zentrum).
- Ein Zug ist nur erlaubt, wenn er mindestens eine gegnerische Kette in gerader
  Linie einschliesst; alle eingeschlossenen Steine werden umgedreht.
- Hat ein Spieler keinen gueltigen Zug, wird automatisch gepasst; hat KEIN
  Spieler einen Zug, endet die Partie. Sieger = mehr Steine auf dem Brett.
- Einzelspieler: KI mit drei Staerken (easy/medium/hard) ueber Negamax mit
  Alpha-Beta-Schnitt und positionsgewichteter Bewertung (Ecken hoch, X-/C-Felder
  negativ) plus Mobilitaet; easy patzt absichtlich.
- Mehrspieler: Schwarz gegen Weiss abwechselnd am selben Rechner.
- Punkte (Highscore) = kumulierte Siege gegen die KI in einer Sitzung
  (connect4-Konvention); Mehrspieler wird nicht gewertet.

Steuerung: Maus (Feld anklicken) oder Pfeile/WASD bewegen den Auswahlrahmen,
Leertaste/Enter setzt den Stein. Nach Rundenende: Enter = neue Runde,
S = Auswahl (nur Einzelspieler).
"""

import random

import pygame

import settings as settings_mod
import ui
from game_base import Game, InputEvent
from i18n import t

# Identitaetsfarben des Spiels (bewusst fest, unabhaengig vom Theme):
# das gruene Brett und die Schwarz/Weiss-Steine SIND Reversi.
COL_BOARD = (28, 92, 58)
COL_BOARD_DARK = (22, 74, 46)
COL_GRID = (16, 54, 34)
COL_PLATE = (18, 40, 28)
COL_P1 = (30, 33, 42)         # Schwarz (Spieler 0)
COL_P1_HI = (78, 84, 100)
COL_P2 = (238, 240, 246)      # Weiss (Spieler 1)
COL_P2_HI = (255, 255, 255)
COL_HINT = (90, 200, 150)

DIFFS = ["easy", "medium", "hard"]
DEPTHS = [1, 3, 4]

N = 8
DIRS = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)]

# Positionsgewichte fuer die KI-Bewertung (Ecken sehr wertvoll, Nachbarfelder
# der Ecken gefaehrlich). Symmetrisches 8x8-Raster.
WEIGHTS = [
    [120, -20, 20,  5,  5, 20, -20, 120],
    [-20, -40, -5, -5, -5, -5, -40, -20],
    [ 20,  -5, 15,  3,  3, 15,  -5,  20],
    [  5,  -5,  3,  3,  3,  3,  -5,   5],
    [  5,  -5,  3,  3,  3,  3,  -5,   5],
    [ 20,  -5, 15,  3,  3, 15,  -5,  20],
    [-20, -40, -5, -5, -5, -5, -40, -20],
    [120, -20, 20,  5,  5, 20, -20, 120],
]

SETUP, PLAY, OVER = "setup", "play", "over"


def _rgba(color, alpha):
    """Palette-Farbe (RGB) mit einem Alpha-Wert zu RGBA kombinieren."""
    return (color[0], color[1], color[2], alpha)


def _flips_for(board, r, c, pv):
    """Liste der Steine, die ein Zug (r,c) fuer Spielerwert pv umdreht (leer=illegal)."""
    if board[r][c] != 0:
        return []
    opp = 3 - pv
    flips = []
    for dr, dc in DIRS:
        line = []
        rr, cc = r + dr, c + dc
        while 0 <= rr < N and 0 <= cc < N and board[rr][cc] == opp:
            line.append((rr, cc))
            rr += dr
            cc += dc
        if line and 0 <= rr < N and 0 <= cc < N and board[rr][cc] == pv:
            flips.extend(line)
    return flips


def _legal_moves(board, pv):
    """dict {(r,c): [flips]} aller gueltigen Zuege fuer Spielerwert pv."""
    moves = {}
    for r in range(N):
        for c in range(N):
            if board[r][c] == 0:
                f = _flips_for(board, r, c, pv)
                if f:
                    moves[(r, c)] = f
    return moves


def _apply(board, r, c, pv, flips):
    """Setzt den Stein und dreht die eingeschlossenen Steine um (in-place)."""
    board[r][c] = pv
    for (fr, fc) in flips:
        board[fr][fc] = pv


def _count(board):
    """(Anzahl Spieler0-Steine, Anzahl Spieler1-Steine)."""
    a = b = 0
    for row in board:
        for v in row:
            if v == 1:
                a += 1
            elif v == 2:
                b += 1
    return a, b


class ReversiGame(Game):
    name = "Reversi"
    highscore_key = "reversi"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        rv = self.settings.get("reversi", {}) if isinstance(self.settings, dict) else {}
        self.diff = max(0, min(2, int(rv.get("difficulty", 1))))

        self._make_fonts()
        self.wins = [0, 0]
        self.starter = 0
        self._build_setup_layout()
        self._new_round()
        self.state = PLAY if self.multiplayer else SETUP

    def _make_fonts(self):
        """Theme-Schriften, Groessen aus der Fensterhoehe abgeleitet."""
        self._small = ui.font(max(13, min(22, self.height // 30)))
        self._tiny = ui.font(max(11, min(18, self.height // 38)))
        self._huge = ui.font(max(26, self.height // 11), bold=True)

    def on_surface_changed(self):
        self._make_fonts()
        self._build_setup_layout()
        self._layout()

    def _layout(self):
        self.hud_h = max(40, int(self.height * 0.085))
        self.cell = int(min((self.width - 40) / N,
                            (self.height - self.hud_h - 24) / N))
        self.bw = N * self.cell
        self.bh = N * self.cell
        self.bx = (self.width - self.bw) // 2
        self.by = self.hud_h + max(8, (self.height - self.hud_h - self.bh) // 2)
        self.r = int(self.cell * 0.42)

    def _new_round(self):
        self.board = [[0] * N for _ in range(N)]
        mid = N // 2
        self.board[mid - 1][mid - 1] = 2
        self.board[mid][mid] = 2
        self.board[mid - 1][mid] = 1
        self.board[mid][mid - 1] = 1
        self.player = self.starter        # 0 = Schwarz beginnt
        self.cursor = [mid, mid]
        self.moves = _legal_moves(self.board, self.player + 1)
        self.winner = None
        self.msg = None
        self.msg_t = 0.0
        self.ai_delay = 0.0
        self.last_move = None
        self._layout()

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        bw = min(360, self.width - 60)
        y0 = int(self.height * 0.32)
        self.diff_rects = [pygame.Rect(cx - bw // 2, y0 + i * 58, bw, 48)
                           for i in range(3)]
        self.start_rect = pygame.Rect(cx - 95, y0 + 3 * 58 + 14, 190, 46)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("reversi", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("1", "2", "3"):
                self.diff = int(event.key) - 1
                self._save_setting("difficulty", self.diff)
                self.play_sound("click")
            elif event.key in ("Up", "w", "W"):
                self.diff = (self.diff - 1) % 3
                self._save_setting("difficulty", self.diff)
                self.play_sound("move")
            elif event.key in ("Down", "s", "S"):
                self.diff = (self.diff + 1) % 3
                self._save_setting("difficulty", self.diff)
                self.play_sound("move")
            elif event.key in ("Return", "space"):
                self._new_round()
                self.state = PLAY
                self.play_sound("click")
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.diff_rects):
                if rc.collidepoint(event.pos):
                    self.diff = i
                    self._save_setting("difficulty", i)
                    self.play_sound("click")
                    return
            if self.start_rect.collidepoint(event.pos):
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
        # Im Einzelspieler ist nur Spieler 0 (Schwarz) steuerbar.
        if not self.multiplayer and self.player == 1:
            return
        if event.kind == InputEvent.MOUSEMOVE:
            rc = self._cell_at(event.pos)
            if rc:
                self.cursor = list(rc)
        elif event.kind == InputEvent.MOUSEDOWN:
            rc = self._cell_at(event.pos)
            if rc:
                self.cursor = list(rc)
                self._try_place(rc[0], rc[1])
        elif event.kind == InputEvent.KEYDOWN:
            k = event.key
            if self.is_action(k, "up") or k == "Up":
                self.cursor[0] = (self.cursor[0] - 1) % N
                self.play_sound("move")
            elif self.is_action(k, "down") or k == "Down":
                self.cursor[0] = (self.cursor[0] + 1) % N
                self.play_sound("move")
            elif self.is_action(k, "left") or k == "Left":
                self.cursor[1] = (self.cursor[1] - 1) % N
                self.play_sound("move")
            elif self.is_action(k, "right") or k == "Right":
                self.cursor[1] = (self.cursor[1] + 1) % N
                self.play_sound("move")
            elif self.is_action(k, "action") or k in ("space", "Return"):
                self._try_place(self.cursor[0], self.cursor[1])

    def _cell_at(self, pos):
        c = (pos[0] - self.bx) // self.cell
        r = (pos[1] - self.by) // self.cell
        if 0 <= r < N and 0 <= c < N:
            return (int(r), int(c))
        return None

    def _restart(self):
        self.starter = 1 - self.starter
        self.game_over = False
        self._new_round()
        self.state = PLAY
        self.play_sound("click")

    def _try_place(self, r, c):
        flips = self.moves.get((r, c))
        if not flips:
            self.msg = t("rev.illegal")
            self.msg_t = 1.0
            self.play_sound("click")
            return
        _apply(self.board, r, c, self.player + 1, flips)
        self.last_move = (r, c)
        self.play_sound("lock")
        self._advance_turn()

    def _advance_turn(self):
        """Wechselt den Spieler; behandelt Passen und Spielende."""
        other = 1 - self.player
        other_moves = _legal_moves(self.board, other + 1)
        if other_moves:
            self.player = other
            self.moves = other_moves
            self.ai_delay = 0.35
            return
        # Gegner muss passen - hat der aktuelle Spieler noch Zuege?
        self_moves = _legal_moves(self.board, self.player + 1)
        if self_moves:
            self.moves = self_moves
            who = t("common.player2") if other == 1 else t("common.player1")
            self.msg = t("rev.pass", name=who)
            self.msg_t = 1.4
            self.play_sound("select")
            self.ai_delay = 0.35
            return
        # Keiner kann ziehen -> Partie zu Ende.
        self._ende()

    def _ende(self):
        a, b = _count(self.board)
        if a > b:
            self.winner = 0
        elif b > a:
            self.winner = 1
        else:
            self.winner = None
        self.state = OVER
        if self.winner is not None:
            self.wins[self.winner] += 1
            if not self.multiplayer:
                if self.winner == 0:
                    self.score = self.wins[0]
                    self.play_sound("win")
                    self.report_result(True)
                else:
                    self.play_sound("gameover")
                    self.report_result(False)
            else:
                self.play_sound("win")
        else:
            self.play_sound("select")
        self.game_over = True     # main.py speichert den Score einmalig

    # ===================================================== Spiellogik
    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None
        if self.state == PLAY and not self.multiplayer and self.player == 1:
            self.ai_delay -= dt
            if self.ai_delay <= 0:
                self._ai_play()

    # ===================================================== KI
    def _ai_play(self):
        move = self._ai_move()
        if move is None:
            self._advance_turn()
            return
        r, c = move
        flips = self.moves.get((r, c)) or _flips_for(self.board, r, c, 2)
        _apply(self.board, r, c, 2, flips)
        self.last_move = (r, c)
        self.play_sound("lock")
        self._advance_turn()

    def _ai_move(self):
        """Waehlt den KI-Zug (Spielerwert 2) je nach Staerke."""
        moves = list(self.moves.keys())
        if not moves:
            return None
        # easy: meist zufaellig, gelegentlich gierig.
        if self.diff == 0:
            if random.random() < 0.6:
                return random.choice(moves)
        depth = DEPTHS[self.diff]
        best_val = -1e18
        best = []
        for (r, c) in moves:
            nb = [row[:] for row in self.board]
            _apply(nb, r, c, 2, _flips_for(nb, r, c, 2))
            val = self._negamax(nb, depth - 1, -1e18, 1e18, 1)  # 1 = Mensch am Zug
            if val > best_val:
                best_val = val
                best = [(r, c)]
            elif val == best_val:
                best.append((r, c))
        # medium: mit kleiner Wahrscheinlichkeit nicht optimal.
        if self.diff == 1 and len(moves) > 1 and random.random() < 0.25:
            return random.choice(moves)
        return random.choice(best)

    def _negamax(self, board, depth, alpha, beta, turn):
        """turn: 0 = KI (Wert 2) am Zug, 1 = Mensch (Wert 1). Bewertung aus KI-Sicht."""
        pv = 2 if turn == 0 else 1
        moves = _legal_moves(board, pv)
        if depth == 0:
            return self._evaluate(board)
        if not moves:
            # Passen - kann der andere ziehen?
            if not _legal_moves(board, 3 - pv):
                return self._terminal(board)
            return self._negamax(board, depth - 1, alpha, beta, 1 - turn)
        # Zugreihenfolge: hoch bewertete Felder zuerst (Alpha-Beta profitiert).
        ordered = sorted(moves.keys(),
                         key=lambda m: WEIGHTS[m[0]][m[1]], reverse=True)
        if turn == 0:   # maximierend (KI)
            best = -1e18
            for (r, c) in ordered:
                nb = [row[:] for row in board]
                _apply(nb, r, c, pv, moves[(r, c)])
                best = max(best, self._negamax(nb, depth - 1, alpha, beta, 1))
                alpha = max(alpha, best)
                if alpha >= beta:
                    break
            return best
        else:           # minimierend (Mensch)
            best = 1e18
            for (r, c) in ordered:
                nb = [row[:] for row in board]
                _apply(nb, r, c, pv, moves[(r, c)])
                best = min(best, self._negamax(nb, depth - 1, alpha, beta, 0))
                beta = min(beta, best)
                if alpha >= beta:
                    break
            return best

    def _terminal(self, board):
        """Endstellungs-Bewertung: klarer Sieg/Verlust dominiert."""
        a, b = _count(board)     # a = Mensch(1), b = KI(2)
        if b > a:
            return 100000 + (b - a)
        if a > b:
            return -100000 - (a - b)
        return 0

    def _evaluate(self, board):
        """Positionsgewichtung + Mobilitaet, aus Sicht der KI (Wert 2)."""
        score = 0
        for r in range(N):
            for c in range(N):
                v = board[r][c]
                if v == 2:
                    score += WEIGHTS[r][c]
                elif v == 1:
                    score -= WEIGHTS[r][c]
        my_mob = len(_legal_moves(board, 2))
        op_mob = len(_legal_moves(board, 1))
        if my_mob + op_mob:
            score += int(8 * (my_mob - op_mob) / (my_mob + op_mob + 1) * 10)
        return score

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        if self.state == SETUP:
            self._draw_setup(s)
            return
        if not hasattr(self, "cell"):
            self._layout()
        self._draw_hud(s)
        self._draw_board(s)
        if self.state == OVER:
            self._draw_over(s)

    def _disc_color(self, p):
        return COL_P1 if p == 0 else COL_P2

    def _disc_hi(self, p):
        return COL_P1_HI if p == 0 else COL_P2_HI

    def _draw_board(self, s):
        # Rahmenplatte
        plate = pygame.Rect(self.bx - 8, self.by - 8, self.bw + 16, self.bh + 16)
        pygame.draw.rect(s, COL_PLATE, plate, border_radius=10)

        # Felder (Schachbrett-Gruen) + Rasterlinien
        for r in range(N):
            for c in range(N):
                x = self.bx + c * self.cell
                y = self.by + r * self.cell
                base = COL_BOARD if (r + c) % 2 == 0 else COL_BOARD_DARK
                pygame.draw.rect(s, base, (x, y, self.cell, self.cell))
        for i in range(N + 1):
            gx = self.bx + i * self.cell
            gy = self.by + i * self.cell
            pygame.draw.line(s, COL_GRID, (gx, self.by), (gx, self.by + self.bh))
            pygame.draw.line(s, COL_GRID, (self.bx, gy), (self.bx + self.bw, gy))

        human_turn = self.state == PLAY and (self.multiplayer or self.player == 0)

        # Zughinweise (kleine Punkte) fuer den steuerbaren Spieler
        if human_turn:
            for (r, c) in self.moves:
                cx = self.bx + c * self.cell + self.cell // 2
                cy = self.by + r * self.cell + self.cell // 2
                pygame.draw.circle(s, COL_HINT, (cx, cy), max(3, self.r // 5))

        # Steine
        for r in range(N):
            for c in range(N):
                v = self.board[r][c]
                if v == 0:
                    continue
                cx = self.bx + c * self.cell + self.cell // 2
                cy = self.by + r * self.cell + self.cell // 2
                pygame.draw.circle(s, (8, 14, 10), (cx, cy + 2), self.r)
                pygame.draw.circle(s, self._disc_color(v - 1), (cx, cy), self.r)
                pygame.draw.circle(s, self._disc_hi(v - 1),
                                   (cx - self.r // 3, cy - self.r // 3),
                                   max(2, self.r // 4))

        # Letzter Zug markieren
        if self.last_move:
            r, c = self.last_move
            x = self.bx + c * self.cell
            y = self.by + r * self.cell
            pygame.draw.rect(s, self.accent, (x, y, self.cell, self.cell), 2)

        # Auswahlrahmen (Tastatur/Maus)
        if human_turn:
            r, c = self.cursor
            x = self.bx + c * self.cell
            y = self.by + r * self.cell
            k = 0.5 + 0.5 * abs(pygame.time.get_ticks() % 900 - 450) / 450
            pygame.draw.rect(s, (int(120 + 120 * k),) * 3,
                             (x + 1, y + 1, self.cell - 2, self.cell - 2), 2)

    def _draw_hud(self, s):
        pygame.draw.rect(s, ui.PANEL, (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h), (self.width, self.hud_h))
        cy = self.hud_h // 2
        a, b = _count(self.board)
        # Stein-Zaehler links (Schwarz) und rechts (Weiss)
        pygame.draw.circle(s, COL_P1, (18, cy), 9)
        pygame.draw.circle(s, COL_P1_HI, (15, cy - 3), 3)
        pygame.draw.circle(s, ui.BORDER_LIGHT, (18, cy), 9, 1)
        img = self._small.render(str(a), True, ui.TEXT)
        s.blit(img, img.get_rect(midleft=(32, cy)))
        pygame.draw.circle(s, COL_P2, (self.width - 18, cy), 9)
        pygame.draw.circle(s, ui.BORDER_LIGHT, (self.width - 18, cy), 9, 1)
        img = self._small.render(str(b), True, ui.TEXT)
        s.blit(img, img.get_rect(midright=(self.width - 32, cy)))

        if self.state == PLAY:
            if not self.multiplayer and self.player == 1:
                mid = t("rev.ai_thinks")
            elif not self.multiplayer:
                mid = t("rev.your_turn")
            else:
                who = t("common.player1") if self.player == 0 else t("common.player2")
                mid = t("rev.turn", name=who)
            img = self._small.render(mid, True, self.accent)
            s.blit(img, img.get_rect(center=(self.width // 2, cy)))
        if self.msg:
            img = self._tiny.render(self.msg, True, ui.GOLD)
            s.blit(img, img.get_rect(center=(self.width // 2, self.hud_h + 12)))

    def _draw_over(self, s):
        hh = self._huge.get_height()
        sh = self._small.get_height()
        th = self._tiny.get_height()
        band_h = hh + sh + th + 40
        y = self.height // 2 - band_h // 2
        ov = pygame.Surface((self.width, band_h), pygame.SRCALPHA)
        ov.fill(_rgba(ui.PANEL, 235))
        s.blit(ov, (0, y))
        pygame.draw.line(s, self.accent, (0, y), (self.width, y), 2)
        pygame.draw.line(s, self.accent, (0, y + band_h - 1),
                         (self.width, y + band_h - 1), 2)
        cx = self.width // 2
        a, b = _count(self.board)
        yy = y + 12
        if self.winner is None:
            head = self._huge.render(t("common.draw"), True, ui.TEXT_DIM)
            s.blit(head, head.get_rect(midtop=(cx, yy)))
        elif self.multiplayer:
            # Siegtext in Theme-Farbe (lesbar), Stein-Symbol zeigt die Farbe.
            head = self._huge.render(t("common.player_wins", n=self.winner + 1),
                                     True, ui.TEXT)
            ri = max(8, hh // 3)
            x0 = cx - (head.get_width() + 2 * ri + 14) // 2
            cyc = yy + hh // 2
            pygame.draw.circle(s, self._disc_color(self.winner),
                               (x0 + ri, cyc), ri)
            pygame.draw.circle(s, self._disc_hi(self.winner),
                               (x0 + ri - ri // 3, cyc - ri // 3),
                               max(2, ri // 4))
            pygame.draw.circle(s, ui.BORDER_LIGHT, (x0 + ri, cyc), ri, 1)
            s.blit(head, head.get_rect(midleft=(x0 + 2 * ri + 14, cyc)))
        else:
            key = "rev.win_you" if self.winner == 0 else "rev.win_ai"
            head = self._huge.render(
                t(key), True, self.accent if self.winner == 0 else ui.RED)
            s.blit(head, head.get_rect(midtop=(cx, yy)))
        yy += hh + 8
        sub = self._small.render(f"{a} : {b}", True, ui.TEXT)
        s.blit(sub, sub.get_rect(midtop=(cx, yy)))
        yy += sh + 6
        # Im Mehrspieler gibt es kein Setup -> nur den Neustart-Hinweis zeigen.
        hint_key = "common.enter_restart" if self.multiplayer else "rev.new_round"
        hint = self._tiny.render(t(hint_key), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(midtop=(cx, yy)))

    # ----- Setup zeichnen -----------------------------------------------
    def _draw_setup(self, s):
        cx = self.width // 2
        title = self._huge.render(self.name.upper(), True, self.accent)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.14))))
        sub = self._small.render(t("rev.subtitle"), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(self.height * 0.21))))
        for i, rc in enumerate(self.diff_rects):
            on = (i == self.diff)
            pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc,
                             border_radius=10)
            pygame.draw.rect(s, self.accent if on else ui.BORDER, rc,
                             2 if on else 1, border_radius=10)
            lbl = self.font.render(t("rev.diff." + DIFFS[i]), True,
                                   ui.TEXT if on else ui.TEXT_DIM)
            s.blit(lbl, lbl.get_rect(midleft=(rc.x + 18, rc.centery)))
        pygame.draw.rect(s, ui.BTN_SEL, self.start_rect, border_radius=10)
        pygame.draw.rect(s, self.accent, self.start_rect, 2, border_radius=10)
        st = self.font.render(t("common.start"), True, ui.TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        hint = self._tiny.render(t("rev.setup_hint"), True, ui.TEXT_FAINT)
        s.blit(hint, hint.get_rect(center=(cx, self.height - 16)))
