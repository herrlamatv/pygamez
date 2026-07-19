# -*- coding: utf-8 -*-
"""
chess.py
========
Schach - 1 Spieler gegen KI (sechs Stärken) oder 2 Spieler lokal.

- Vollständige Regeln: alle Figurenzüge, Rochade (kurz/lang, mit korrekten
  Bedingungen inkl. Zug durch/ins Schach), En Passant, Bauernumwandlung mit
  Auswahl (Dame/Turm/Läufer/Springer), Schach, Schachmatt und Patt sowie Remis
  durch 50-Züge-Regel, dreifache Stellungswiederholung und ungenügendes Material.
- Einzelspieler: KI über Negamax mit Alpha-Beta-Schnitt, Zugsortierung
  (Schlagzüge zuerst), Figur- und Feldwert-Tabellen (piece-square tables) und
  optionaler Ruhesuche (Quiescence) bei Schlagzügen. Sechs Stufen von Anfänger
  (zufällig) bis Meister; niedrige Stufen patzen absichtlich. Ein Knotenbudget
  deckelt die Suche, damit das Spiel nie einfriert.
- Mehrspieler: Weiß gegen Schwarz abwechselnd am selben Rechner.
- Punkte (Highscore) = Siege gegen die KI in einer Sitzung.

Steuerung: Maus (Figur anklicken, dann Zielfeld) oder Pfeile/WASD bewegen den
Auswahlrahmen, Leertaste/Enter wählt/zieht. Nach Rundenende: Enter = neue Runde,
S = Setup (nur Einzelspieler).
"""

import random
import time

import pygame

import settings as settings_mod
import ui
from game_base import Game, InputEvent, LocalizedName
from i18n import t

# ------------------------------------------------- Brett-Identitätsfarben
# Generische UI-Farben (Hintergrund, Panels, Text) kommen zur Laufzeit aus
# der dynamischen ui-Palette; hier bleiben nur die Brett-/Figurenfarben.
COL_LIGHT = (232, 219, 196)      # helle Felder
COL_DARK = (129, 100, 74)        # dunkle Felder
COL_PLATE = (40, 34, 30)         # Brettrahmen
COL_SEL = (246, 214, 92)
COL_MOVE = (110, 200, 130)
COL_LAST = (120, 160, 240)
COL_CHECK = (224, 84, 84)
COL_WHITE = (244, 244, 248)      # weisse Figuren
COL_BLACK = (34, 32, 38)         # schwarze Figuren
COL_OUTLINE = (16, 14, 18)

# ----------------------------------------------------------------- Regeln
SETUP, PLAY, OVER = "setup", "play", "over"

DIFFS = ["lvl0", "lvl1", "lvl2", "lvl3", "lvl4", "lvl5"]
DIFF_DEPTH = [0, 1, 2, 2, 3, 3]
DIFF_QUIES = [False, False, False, True, True, True]
DIFF_RAND = [1.0, 0.55, 0.25, 0.12, 0.04, 0.0]
# Harte Zeitobergrenze pro KI-Zug (Sekunden), damit die Oberfläche nie
# einfriert - die Suche bricht danach mit der bisherigen Bewertung ab.
TIME_BUDGET = [0.0, 0.25, 0.4, 0.6, 0.9, 1.1]
NODE_BUDGET = 400000

KNIGHT_OFF = [(-2, -1), (-2, 1), (-1, -2), (-1, 2),
              (1, -2), (1, 2), (2, -1), (2, 1)]
KING_OFF = [(-1, -1), (-1, 0), (-1, 1), (0, -1),
            (0, 1), (1, -1), (1, 0), (1, 1)]
BISHOP_DIR = [(-1, -1), (-1, 1), (1, -1), (1, 1)]
ROOK_DIR = [(-1, 0), (1, 0), (0, -1), (0, 1)]

VALUES = {"P": 100, "N": 320, "B": 330, "R": 500, "Q": 900, "K": 20000}

# Feldwert-Tabellen (aus Sicht von Weiss, Index 0 = a8 = Zeile r0). Fuer
# schwarze Figuren wird vertikal gespiegelt (Zeile 7-r).
_PST = {
    "P": [0, 0, 0, 0, 0, 0, 0, 0,
          50, 50, 50, 50, 50, 50, 50, 50,
          10, 10, 20, 30, 30, 20, 10, 10,
          5, 5, 10, 25, 25, 10, 5, 5,
          0, 0, 0, 20, 20, 0, 0, 0,
          5, -5, -10, 0, 0, -10, -5, 5,
          5, 10, 10, -20, -20, 10, 10, 5,
          0, 0, 0, 0, 0, 0, 0, 0],
    "N": [-50, -40, -30, -30, -30, -30, -40, -50,
          -40, -20, 0, 0, 0, 0, -20, -40,
          -30, 0, 10, 15, 15, 10, 0, -30,
          -30, 5, 15, 20, 20, 15, 5, -30,
          -30, 0, 15, 20, 20, 15, 0, -30,
          -30, 5, 10, 15, 15, 10, 5, -30,
          -40, -20, 0, 5, 5, 0, -20, -40,
          -50, -40, -30, -30, -30, -30, -40, -50],
    "B": [-20, -10, -10, -10, -10, -10, -10, -20,
          -10, 0, 0, 0, 0, 0, 0, -10,
          -10, 0, 5, 10, 10, 5, 0, -10,
          -10, 5, 5, 10, 10, 5, 5, -10,
          -10, 0, 10, 10, 10, 10, 0, -10,
          -10, 10, 10, 10, 10, 10, 10, -10,
          -10, 5, 0, 0, 0, 0, 5, -10,
          -20, -10, -10, -10, -10, -10, -10, -20],
    "R": [0, 0, 0, 0, 0, 0, 0, 0,
          5, 10, 10, 10, 10, 10, 10, 5,
          -5, 0, 0, 0, 0, 0, 0, -5,
          -5, 0, 0, 0, 0, 0, 0, -5,
          -5, 0, 0, 0, 0, 0, 0, -5,
          -5, 0, 0, 0, 0, 0, 0, -5,
          -5, 0, 0, 0, 0, 0, 0, -5,
          0, 0, 0, 5, 5, 0, 0, 0],
    "Q": [-20, -10, -10, -5, -5, -10, -10, -20,
          -10, 0, 0, 0, 0, 0, 0, -10,
          -10, 0, 5, 5, 5, 5, 0, -10,
          -5, 0, 5, 5, 5, 5, 0, -5,
          0, 0, 5, 5, 5, 5, 0, -5,
          -10, 5, 5, 5, 5, 5, 0, -10,
          -10, 0, 5, 0, 0, 0, 0, -10,
          -20, -10, -10, -5, -5, -10, -10, -20],
    "K": [-30, -40, -40, -50, -50, -40, -40, -30,
          -30, -40, -40, -50, -50, -40, -40, -30,
          -30, -40, -40, -50, -50, -40, -40, -30,
          -30, -40, -40, -50, -50, -40, -40, -30,
          -20, -30, -30, -40, -40, -30, -30, -20,
          -10, -20, -20, -20, -20, -20, -20, -10,
          20, 20, 0, 0, 0, 0, 20, 20,
          20, 30, 10, 0, 0, 10, 30, 20],
}

GLYPH = {"K": "♚", "Q": "♛", "R": "♜",
         "B": "♝", "N": "♞", "P": "♟"}


# =================================================================== Regel-Helfer
def _start_board():
    """Standard-Grundstellung. r0 = Zeile 8 (Schwarz), r7 = Zeile 1 (Weiss)."""
    back = "RNBQKBNR"
    board = [[None] * 8 for _ in range(8)]
    for c in range(8):
        board[0][c] = "b" + back[c]
        board[1][c] = "bP"
        board[6][c] = "wP"
        board[7][c] = "w" + back[c]
    return board


def _attacked(board, r, c, by):
    """True, wenn Feld (r,c) von einer Figur der Farbe 'by' angegriffen wird."""
    if by == "w":
        for dc in (-1, 1):
            rr, cc = r + 1, c + dc
            if 0 <= rr < 8 and 0 <= cc < 8 and board[rr][cc] == "wP":
                return True
    else:
        for dc in (-1, 1):
            rr, cc = r - 1, c + dc
            if 0 <= rr < 8 and 0 <= cc < 8 and board[rr][cc] == "bP":
                return True
    for dr, dc in KNIGHT_OFF:
        rr, cc = r + dr, c + dc
        if 0 <= rr < 8 and 0 <= cc < 8 and board[rr][cc] == by + "N":
            return True
    for dr, dc in KING_OFF:
        rr, cc = r + dr, c + dc
        if 0 <= rr < 8 and 0 <= cc < 8 and board[rr][cc] == by + "K":
            return True
    for dr, dc in BISHOP_DIR:
        rr, cc = r + dr, c + dc
        while 0 <= rr < 8 and 0 <= cc < 8:
            p = board[rr][cc]
            if p:
                if p[0] == by and p[1] in ("B", "Q"):
                    return True
                break
            rr += dr
            cc += dc
    for dr, dc in ROOK_DIR:
        rr, cc = r + dr, c + dc
        while 0 <= rr < 8 and 0 <= cc < 8:
            p = board[rr][cc]
            if p:
                if p[0] == by and p[1] in ("R", "Q"):
                    return True
                break
            rr += dr
            cc += dc
    return False


def _king_sq(board, color):
    k = color + "K"
    for r in range(8):
        for c in range(8):
            if board[r][c] == k:
                return (r, c)
    return None


def _in_check(board, color):
    ks = _king_sq(board, color)
    if ks is None:
        return False
    return _attacked(board, ks[0], ks[1], "b" if color == "w" else "w")


def _gen_pseudo(board, color, castling, ep):
    """Pseudolegale Züge (ohne Fesselungsprüfung). Rochade wird schon hier
    korrekt auf 'nicht durch/ins Schach' geprüft."""
    moves = []
    opp = "b" if color == "w" else "w"
    fwd = -1 if color == "w" else 1
    start_row = 6 if color == "w" else 1
    promo_row = 0 if color == "w" else 7
    for r in range(8):
        for c in range(8):
            p = board[r][c]
            if not p or p[0] != color:
                continue
            typ = p[1]
            if typ == "P":
                # Ein Feld vor
                nr = r + fwd
                if 0 <= nr < 8 and board[nr][c] is None:
                    if nr == promo_row:
                        moves.append((r, c, nr, c, "Q", "promo"))
                    else:
                        moves.append((r, c, nr, c, None, None))
                    # Zwei Felder von der Grundreihe
                    if r == start_row and board[r + 2 * fwd][c] is None:
                        moves.append((r, c, r + 2 * fwd, c, None, "2step"))
                # Schlagen (inkl. En Passant)
                for dc in (-1, 1):
                    cc = c + dc
                    if not (0 <= nr < 8 and 0 <= cc < 8):
                        continue
                    tgt = board[nr][cc]
                    if tgt and tgt[0] == opp:
                        if nr == promo_row:
                            moves.append((r, c, nr, cc, "Q", "promo"))
                        else:
                            moves.append((r, c, nr, cc, None, None))
                    elif ep is not None and (nr, cc) == ep:
                        moves.append((r, c, nr, cc, None, "ep"))
            elif typ == "N":
                for dr, dc in KNIGHT_OFF:
                    rr, cc = r + dr, c + dc
                    if 0 <= rr < 8 and 0 <= cc < 8:
                        tgt = board[rr][cc]
                        if tgt is None or tgt[0] == opp:
                            moves.append((r, c, rr, cc, None, None))
            elif typ == "K":
                for dr, dc in KING_OFF:
                    rr, cc = r + dr, c + dc
                    if 0 <= rr < 8 and 0 <= cc < 8:
                        tgt = board[rr][cc]
                        if tgt is None or tgt[0] == opp:
                            moves.append((r, c, rr, cc, None, None))
                # Rochade
                _gen_castle(board, color, castling, r, c, moves)
            else:
                dirs = (BISHOP_DIR if typ == "B" else
                        ROOK_DIR if typ == "R" else BISHOP_DIR + ROOK_DIR)
                for dr, dc in dirs:
                    rr, cc = r + dr, c + dc
                    while 0 <= rr < 8 and 0 <= cc < 8:
                        tgt = board[rr][cc]
                        if tgt is None:
                            moves.append((r, c, rr, cc, None, None))
                        else:
                            if tgt[0] == opp:
                                moves.append((r, c, rr, cc, None, None))
                            break
                        rr += dr
                        cc += dc
    return moves


def _gen_castle(board, color, castling, r, c, moves):
    opp = "b" if color == "w" else "w"
    row = 7 if color == "w" else 0
    if r != row or c != 4:
        return
    if _attacked(board, row, 4, opp):
        return   # aus dem Schach darf man nicht rochieren
    ck = color + "K"
    cq = color + "Q"
    if ck in castling and board[row][5] is None and board[row][6] is None \
            and board[row][7] == color + "R" \
            and not _attacked(board, row, 5, opp) \
            and not _attacked(board, row, 6, opp):
        moves.append((row, 4, row, 6, None, "castleK"))
    if cq in castling and board[row][1] is None and board[row][2] is None \
            and board[row][3] is None and board[row][0] == color + "R" \
            and not _attacked(board, row, 3, opp) \
            and not _attacked(board, row, 2, opp):
        moves.append((row, 4, row, 2, None, "castleQ"))


def _apply(board, mv, color, castling, ep):
    """Führt Zug aus und liefert (neues_board, neue_rochaderechte, neues_ep)."""
    nb = [row[:] for row in board]
    fr, fc, tr, tc, promo, flag = mv
    piece = nb[fr][fc]
    nb[tr][tc] = piece
    nb[fr][fc] = None
    new_ep = None
    if flag == "2step":
        new_ep = ((fr + tr) // 2, fc)
    elif flag == "ep":
        nb[fr][tc] = None            # geschlagener Bauer steht neben dem Läufer
    elif flag == "promo":
        nb[tr][tc] = color + (promo or "Q")
    elif flag == "castleK":
        row = fr
        nb[row][5] = nb[row][7]
        nb[row][7] = None
    elif flag == "castleQ":
        row = fr
        nb[row][3] = nb[row][0]
        nb[row][0] = None
    # Rochaderechte robust aus dem Brett ableiten (König-/Turmzug/-schlag).
    ncr = set(castling)
    if nb[7][4] != "wK":
        ncr.discard("wK")
        ncr.discard("wQ")
    if nb[0][4] != "bK":
        ncr.discard("bK")
        ncr.discard("bQ")
    if nb[7][7] != "wR":
        ncr.discard("wK")
    if nb[7][0] != "wR":
        ncr.discard("wQ")
    if nb[0][7] != "bR":
        ncr.discard("bK")
    if nb[0][0] != "bR":
        ncr.discard("bQ")
    return nb, ncr, new_ep


def _legal_moves(board, color, castling, ep):
    """Alle legalen Züge (pseudolegal ohne Selbst-Schach)."""
    res = []
    for mv in _gen_pseudo(board, color, castling, ep):
        nb, _, _ = _apply(board, mv, color, castling, ep)
        if not _in_check(nb, color):
            res.append(mv)
    return res


def _evaluate(board):
    """Material + Feldwerte, positiv = gut für Weiss (Zentipawns)."""
    score = 0
    for r in range(8):
        row = board[r]
        for c in range(8):
            p = row[c]
            if not p:
                continue
            if p[0] == "w":
                score += VALUES[p[1]] + _PST[p[1]][r * 8 + c]
            else:
                score -= VALUES[p[1]] + _PST[p[1]][(7 - r) * 8 + c]
    return score


def _insufficient(board):
    """Remis durch ungenügendes Material (nur Könige, K+L, K+S)."""
    minors = 0
    for r in range(8):
        for c in range(8):
            p = board[r][c]
            if not p:
                continue
            t2 = p[1]
            if t2 in ("P", "R", "Q"):
                return False
            if t2 in ("B", "N"):
                minors += 1
    return minors <= 1


def _pos_key(board, color, castling, ep):
    rows = "/".join("".join(p or "." for p in row) for row in board)
    return rows + " " + color + " " + "".join(sorted(castling)) + " " + str(ep)


class ChessGame(Game):
    name = LocalizedName("Chess", de="Schach", fr="Échecs",
                         es="Ajedrez", pt="Xadrez")
    highscore_key = "chess"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        cs = self.settings.get("chess", {}) if isinstance(self.settings, dict) else {}
        self.diff = max(0, min(5, int(cs.get("difficulty", 2))))
        self.human_color = "b" if cs.get("color") == "black" else "w"

        self._make_fonts()
        self.wins = [0, 0]           # [Mensch, KI] bzw. [Weiss, Schwarz]
        self._make_piece_font()
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
        self._make_piece_font()

    def _make_piece_font(self):
        cell = getattr(self, "cell", max(24, self.height // 12))
        self._piece_font = pygame.font.SysFont(
            "segoeuisymbol,dejavusans,arialunicodems,freeserif",
            int(cell * 0.74))

    def _layout(self):
        self.hud_h = 44
        self.cell = int(min((self.width - 40) / 8,
                            (self.height - self.hud_h - 24) / 8))
        self.bw = self.bh = 8 * self.cell
        self.bx = (self.width - self.bw) // 2
        self.by = self.hud_h + max(8, (self.height - self.hud_h - self.bh) // 2)

    def _new_round(self):
        self.board = _start_board()
        self.castling = {"wK", "wQ", "bK", "bQ"}
        self.ep = None
        self.turn = "w"
        self.halfmove = 0
        self.pos_counts = {}
        self.sel = None
        self.targets = {}
        self.cursor = [6, 4]
        self.last_move = None
        self.winner = None            # 0=Weiss,1=Schwarz,None=Remis/laufend
        self.result_key = None
        self.promo_move = None
        self.msg = None
        self.msg_t = 0.0
        self.ai_delay = 0.6
        self._layout()
        self._make_piece_font()
        self._refresh_legal()

    def _refresh_legal(self):
        self.legal = _legal_moves(self.board, self.turn, self.castling, self.ep)
        self.legal_by_from = {}
        for mv in self.legal:
            self.legal_by_from.setdefault((mv[0], mv[1]), []).append(mv)
        self.check = _in_check(self.board, self.turn)
        key = _pos_key(self.board, self.turn, self.castling, self.ep)
        self.pos_counts[key] = self.pos_counts.get(key, 0) + 1
        self._threefold = self.pos_counts[key] >= 3

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        cx = self.width // 2
        y0 = int(self.height * 0.30)
        bw = min(360, self.width - 60)
        # Sechs Schwierigkeits-Buttons in einer Reihe
        gap = 8
        n = 6
        cellw = (bw - gap * (n - 1)) / n
        self.diff_rects = [
            pygame.Rect(int(cx - bw / 2 + i * (cellw + gap)), y0,
                        int(cellw), 46) for i in range(n)]
        y1 = y0 + 84
        cw = min(150, (bw - gap) // 2)
        self.color_rects = [pygame.Rect(cx - cw - gap // 2, y1, cw, 42),
                            pygame.Rect(cx + gap // 2, y1, cw, 42)]
        self.start_rect = pygame.Rect(cx - 95, y1 + 60, 190, 46)

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("chess", {})[key] = value
            settings_mod.save_settings(self.settings)

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3", "4", "5", "6"):
                self.diff = int(k) - 1
                self._save_setting("difficulty", self.diff)
                self.play_sound("click")
            elif k in ("Left", "a", "A", "Up", "w", "W"):
                self.diff = (self.diff - 1) % 6
                self._save_setting("difficulty", self.diff)
                self.play_sound("move")
            elif k in ("Right", "d", "D", "Down", "s", "S"):
                self.diff = (self.diff + 1) % 6
                self._save_setting("difficulty", self.diff)
                self.play_sound("move")
            elif k in ("c", "C"):
                self.human_color = "b" if self.human_color == "w" else "w"
                self._save_setting("color",
                                   "black" if self.human_color == "b" else "white")
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
            for i, rc in enumerate(self.color_rects):
                if rc.collidepoint(event.pos):
                    self.human_color = "w" if i == 0 else "b"
                    self._save_setting("color",
                                       "white" if i == 0 else "black")
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
        if self.promo_move is not None:
            self._handle_promo(event)
            return
        if not self._human_turn():
            return
        if event.kind == InputEvent.MOUSEMOVE:
            rc = self._cell_at(event.pos)
            if rc:
                # Cursor speichert ANZEIGE-Koordinaten (wichtig bei gedrehtem
                # Brett, wenn der Mensch Schwarz spielt).
                self.cursor = list(self._board_to_disp(*rc))
        elif event.kind == InputEvent.MOUSEDOWN:
            rc = self._cell_at(event.pos)
            if rc:
                self.cursor = list(self._board_to_disp(*rc))
                self._click_cell(rc[0], rc[1])
        elif event.kind == InputEvent.KEYDOWN:
            k = event.key
            if self.is_action(k, "up") or k == "Up":
                self.cursor[0] = max(0, self.cursor[0] - 1)
                self.play_sound("move")
            elif self.is_action(k, "down") or k == "Down":
                self.cursor[0] = min(7, self.cursor[0] + 1)
                self.play_sound("move")
            elif self.is_action(k, "left") or k == "Left":
                self.cursor[1] = max(0, self.cursor[1] - 1)
                self.play_sound("move")
            elif self.is_action(k, "right") or k == "Right":
                self.cursor[1] = min(7, self.cursor[1] + 1)
                self.play_sound("move")
            elif self.is_action(k, "action") or k in ("space", "Return"):
                dr = self._disp_to_board(self.cursor[0], self.cursor[1])
                self._click_cell(dr[0], dr[1])

    def _human_turn(self):
        return self.multiplayer or self.turn == self.human_color

    def _click_cell(self, r, c):
        # Auswahl eines eigenen Steins
        p = self.board[r][c]
        if self.sel is not None and (r, c) in self.targets:
            mv = self.targets[(r, c)]
            if mv[5] == "promo":
                self.promo_move = (mv[0], mv[1], mv[2], mv[3])
                self.play_sound("select")
                return
            self._commit(mv)
            return
        if p and p[0] == self.turn:
            self.sel = (r, c)
            self.targets = {(m[2], m[3]): m
                            for m in self.legal_by_from.get((r, c), [])}
            self.play_sound("click")
        else:
            self.sel = None
            self.targets = {}

    def _handle_promo(self, event):
        order = ["Q", "R", "B", "N"]
        if event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self._promo_rects()):
                if rc.collidepoint(event.pos):
                    self._finish_promo(order[i])
                    return
        elif event.kind == InputEvent.KEYDOWN:
            m = {"q": "Q", "r": "R", "b": "B", "n": "N"}
            if event.key.lower() in m:
                self._finish_promo(m[event.key.lower()])

    def _finish_promo(self, piece):
        fr, fc, tr, tc = self.promo_move
        self.promo_move = None
        self._commit((fr, fc, tr, tc, piece, "promo"))

    def _cell_at(self, pos):
        c = (pos[0] - self.bx) // self.cell
        r = (pos[1] - self.by) // self.cell
        if 0 <= r < 8 and 0 <= c < 8:
            return self._disp_to_board(int(r), int(c))
        return None

    def _flip(self):
        # Menschliche Seite immer unten anzeigen.
        return (not self.multiplayer) and self.human_color == "b"

    def _disp_to_board(self, dr, dc):
        return (7 - dr, 7 - dc) if self._flip() else (dr, dc)

    def _board_to_disp(self, r, c):
        return (7 - r, 7 - c) if self._flip() else (r, c)

    def _restart(self):
        self.game_over = False
        if not self.multiplayer:
            self.human_color = "b" if self.human_color == "w" else "w"
            self._save_setting("color",
                               "black" if self.human_color == "b" else "white")
        self._new_round()
        self.state = PLAY
        self.play_sound("click")

    # ===================================================== Zug ausführen
    def _commit(self, mv):
        piece = self.board[mv[0]][mv[1]]
        capture = self.board[mv[2]][mv[3]] is not None or mv[5] == "ep"
        self.board, self.castling, self.ep = _apply(
            self.board, mv, self.turn, self.castling, self.ep)
        if piece[1] == "P" or capture:
            self.halfmove = 0
        else:
            self.halfmove += 1
        self.last_move = (mv[0], mv[1], mv[2], mv[3])
        self.sel = None
        self.targets = {}
        self.turn = "b" if self.turn == "w" else "w"
        self._refresh_legal()
        self.play_sound("lock" if capture else "move")
        self._check_end()

    def _check_end(self):
        if not self.legal:
            if self.check:
                # Der Spieler am Zug ist matt -> der andere gewinnt.
                self.winner = 1 if self.turn == "w" else 0
                self.result_key = "checkmate"
                self._end()
            else:
                self.winner = None
                self.result_key = "stalemate"
                self._end()
            return
        if self.halfmove >= 100:
            self.winner = None
            self.result_key = "fifty"
            self._end()
            return
        if self._threefold:
            self.winner = None
            self.result_key = "threefold"
            self._end()
            return
        if _insufficient(self.board):
            self.winner = None
            self.result_key = "material"
            self._end()
            return
        if not self.multiplayer and self.turn != self.human_color:
            self.ai_delay = 0.45

    def _end(self):
        self.state = OVER
        if self.winner is not None:
            self.wins[self.winner] += 1
            if not self.multiplayer:
                human_idx = 0 if self.human_color == "w" else 1
                if self.winner == human_idx:
                    self.score = self.wins[human_idx]
                    self.play_sound("win")
                else:
                    self.play_sound("gameover")
            else:
                self.play_sound("win")
        else:
            self.play_sound("select")
        self.game_over = True

    # ===================================================== KI
    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None
        if (self.state == PLAY and not self.multiplayer
                and self.promo_move is None
                and self.turn != self.human_color):
            self.ai_delay -= dt
            if self.ai_delay <= 0:
                self._ai_play()

    def _ai_play(self):
        mv = self._pick_ai_move()
        if mv is None:
            return
        self._commit(mv)

    def _pick_ai_move(self):
        moves = list(self.legal)
        if not moves:
            return None
        if self.diff == 0 or random.random() < DIFF_RAND[self.diff]:
            # Schwache Stufen: bevorzugt einfache Schlagzüge, sonst zufällig.
            caps = [m for m in moves if self.board[m[2]][m[3]] is not None]
            if caps and random.random() < 0.6:
                return random.choice(caps)
            return random.choice(moves)
        depth = DIFF_DEPTH[self.diff]
        self._nodes = 0
        self._deadline = time.time() + TIME_BUDGET[self.diff]
        quies = DIFF_QUIES[self.diff]
        best_val = -10 ** 9
        best = []
        moves = self._order(self.board, moves)
        for mv in moves:
            nb, nc, nep = _apply(self.board, mv, self.turn, self.castling, self.ep)
            opp = "b" if self.turn == "w" else "w"
            val = -self._search(nb, opp, depth - 1, -10 ** 9, 10 ** 9,
                                nc, nep, quies)
            if best and time.time() > self._deadline:
                break     # Zeitbudget erschöpft - bisher bester Zug zählt
            if val > best_val:
                best_val = val
                best = [mv]
            elif val == best_val:
                best.append(mv)
        return random.choice(best) if best else random.choice(moves)

    def _order(self, board, moves):
        """Schlagzüge zuerst (MVV-LVA), das beschleunigt Alpha-Beta stark."""
        def score(m):
            tgt = board[m[2]][m[3]]
            s = 0
            if tgt is not None:
                s = 10 * VALUES[tgt[1]] - VALUES[board[m[0]][m[1]][1]]
            if m[5] == "promo":
                s += 800
            return -s
        return sorted(moves, key=score)

    def _search(self, board, color, depth, alpha, beta, castling, ep, quies):
        self._nodes += 1
        if self._nodes > NODE_BUDGET or (self._nodes & 1023) == 0 \
                and time.time() > self._deadline:
            rel = _evaluate(board)
            return rel if color == "w" else -rel
        moves = _legal_moves(board, color, castling, ep)
        if not moves:
            if _in_check(board, color):
                return -30000 + (5 - depth)     # Matt: je schneller, desto besser
            return 0                            # Patt
        if depth <= 0:
            if quies:
                return self._quiesce(board, color, alpha, beta, castling, ep, 4)
            rel = _evaluate(board)
            return rel if color == "w" else -rel
        moves = self._order(board, moves)
        best = -10 ** 9
        opp = "b" if color == "w" else "w"
        for mv in moves:
            nb, nc, nep = _apply(board, mv, color, castling, ep)
            val = -self._search(nb, opp, depth - 1, -beta, -alpha, nc, nep, quies)
            if val > best:
                best = val
            if best > alpha:
                alpha = best
            if alpha >= beta:
                break
        return best

    def _quiesce(self, board, color, alpha, beta, castling, ep, ply):
        self._nodes += 1
        rel = _evaluate(board)
        stand = rel if color == "w" else -rel
        if ply <= 0 or self._nodes > NODE_BUDGET or time.time() > self._deadline:
            return stand
        if stand >= beta:
            return beta
        if stand > alpha:
            alpha = stand
        opp = "b" if color == "w" else "w"
        caps = [m for m in _legal_moves(board, color, castling, ep)
                if board[m[2]][m[3]] is not None or m[5] == "ep"]
        for mv in self._order(board, caps):
            nb, nc, nep = _apply(board, mv, color, castling, ep)
            val = -self._quiesce(nb, opp, -beta, -alpha, nc, nep, ply - 1)
            if val >= beta:
                return beta
            if val > alpha:
                alpha = val
        return alpha

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
        if self.promo_move is not None:
            self._draw_promo(s)
        if self.state == OVER:
            self._draw_over(s)

    def _sq_rect(self, r, c):
        dr, dc = self._board_to_disp(r, c)
        return pygame.Rect(self.bx + dc * self.cell, self.by + dr * self.cell,
                           self.cell, self.cell)

    def _draw_board(self, s):
        plate = pygame.Rect(self.bx - 7, self.by - 7, self.bw + 14, self.bh + 14)
        pygame.draw.rect(s, COL_PLATE, plate, border_radius=8)
        pygame.draw.rect(s, ui.mix(COL_PLATE, self.accent, 0.45), plate, 1,
                         border_radius=8)
        for r in range(8):
            for c in range(8):
                rect = self._sq_rect(r, c)
                base = COL_LIGHT if (r + c) % 2 == 0 else COL_DARK
                pygame.draw.rect(s, base, rect)
        # Letzter Zug (sanft pulsierend)
        if self.last_move:
            alpha = int(60 + 45 * ui.pulse(1.6))
            for (r, c) in ((self.last_move[0], self.last_move[1]),
                           (self.last_move[2], self.last_move[3])):
                rect = self._sq_rect(r, c)
                ov = pygame.Surface((self.cell, self.cell), pygame.SRCALPHA)
                ov.fill((*COL_LAST, alpha))
                s.blit(ov, rect.topleft)
        # König im Schach markieren
        if self.check and self.state == PLAY:
            ks = _king_sq(self.board, self.turn)
            if ks:
                rect = self._sq_rect(ks[0], ks[1])
                ov = pygame.Surface((self.cell, self.cell), pygame.SRCALPHA)
                ov.fill((*COL_CHECK, 110))
                s.blit(ov, rect.topleft)
        # Auswahl + Zughinweise
        human = self.state == PLAY and self._human_turn()
        if self.sel is not None and human:
            pygame.draw.rect(s, COL_SEL, self._sq_rect(*self.sel), 3)
            for (tr, tc), mv in self.targets.items():
                rect = self._sq_rect(tr, tc)
                cx, cy = rect.center
                if self.board[tr][tc] is not None or mv[5] == "ep":
                    pygame.draw.circle(s, COL_MOVE, (cx, cy),
                                       self.cell // 2 - 3, 3)
                else:
                    pygame.draw.circle(s, COL_MOVE, (cx, cy),
                                       max(4, self.cell // 7))
        # Figuren
        for r in range(8):
            for c in range(8):
                p = self.board[r][c]
                if p:
                    self._draw_piece(s, p, self._sq_rect(r, c))
        # Cursor (Tastatur)
        if human:
            dr, dc = self.cursor
            rect = pygame.Rect(self.bx + dc * self.cell, self.by + dr * self.cell,
                               self.cell, self.cell)
            k = ui.pulse(2.2, 0.0, 1.0)
            pygame.draw.rect(s, (int(120 + 120 * k),) * 3,
                             (rect.x + 1, rect.y + 1, self.cell - 2, self.cell - 2),
                             2)

    def _draw_piece(self, s, piece, rect):
        glyph = GLYPH[piece[1]]
        main = COL_WHITE if piece[0] == "w" else COL_BLACK
        cx, cy = rect.center
        # Outline für Lesbarkeit
        base = self._piece_font.render(glyph, True, COL_OUTLINE)
        for ox, oy in ((-2, 0), (2, 0), (0, -2), (0, 2), (-1, -1), (1, 1)):
            s.blit(base, base.get_rect(center=(cx + ox, cy + oy)))
        img = self._piece_font.render(glyph, True, main)
        s.blit(img, img.get_rect(center=(cx, cy)))

    def _draw_hud(self, s):
        panel = pygame.Rect(8, 6, self.width - 16, self.hud_h - 10)
        ui.draw_panel(s, panel, shadow=False, accent_top=self.accent)
        cy = panel.centery
        # Siegzähler mit Farbpunkt (links Weiss, rechts Schwarz)
        pygame.draw.circle(s, COL_WHITE, (panel.x + 16, cy), 7)
        pygame.draw.circle(s, ui.BORDER_LIGHT, (panel.x + 16, cy), 7, 1)
        left = self._small.render(f"{self.wins[0]}", True, ui.TEXT)
        s.blit(left, left.get_rect(midleft=(panel.x + 30, cy)))
        pygame.draw.circle(s, COL_BLACK, (panel.right - 16, cy), 7)
        pygame.draw.circle(s, ui.BORDER_LIGHT, (panel.right - 16, cy), 7, 1)
        right = self._small.render(f"{self.wins[1]}", True, ui.TEXT)
        s.blit(right, right.get_rect(midright=(panel.right - 30, cy)))
        if self.state == PLAY:
            warn = False
            if self.promo_move is not None:
                mid = t("chess.promote")
            elif not self.multiplayer and self.turn != self.human_color:
                mid = t("chess.ai_thinks")
            elif not self.multiplayer:
                mid = t("chess.check") if self.check else t("chess.your_turn")
                warn = self.check
            else:
                who = t("common.player1") if self.turn == "w" else t("common.player2")
                mid = t("chess.turn", name=who)
                if self.check:
                    mid += "  +"
                    warn = True
            img = self._small.render(mid, True, ui.RED if warn else self.accent)
            s.blit(img, img.get_rect(center=(self.width // 2, cy)))

    def _promo_rects(self):
        n = 4
        w = min(70, (self.width - 40) // n)
        total = w * n + 12 * (n - 1)
        x0 = (self.width - total) // 2
        y = self.height // 2 - w // 2
        return [pygame.Rect(x0 + i * (w + 12), y, w, w) for i in range(n)]

    def _draw_promo(self, s):
        ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
        ov.fill((10, 8, 12, 180))
        s.blit(ov, (0, 0))
        head = self._small.render(t("chess.promote"), True, ui.TEXT)
        rects = self._promo_rects()
        s.blit(head, head.get_rect(center=(self.width // 2, rects[0].y - 26)))
        for i, ch in enumerate(["Q", "R", "B", "N"]):
            rc = rects[i]
            pygame.draw.rect(s, ui.BTN_SEL, rc, border_radius=8)
            pygame.draw.rect(s, self.accent, rc, 2, border_radius=8)
            self._draw_piece(s, self.turn + ch, rc)

    def _draw_over(self, s):
        cx = self.width // 2
        if self.winner is None:
            head = self._huge.render(t("common.draw"), True, ui.TEXT_DIM)
        elif self.multiplayer:
            head = self._huge.render(t("common.player_wins", n=self.winner + 1),
                                     True, self.accent)
        else:
            human_idx = 0 if self.human_color == "w" else 1
            won = self.winner == human_idx
            head = self._huge.render(t("chess.win_you") if won
                                     else t("chess.win_ai"), True,
                                     self.accent if won else ui.TEXT_DIM)
        sub = self._small.render(t("chess.reason." + (self.result_key or "")),
                                 True, ui.TEXT)
        hint_key = "common.enter_restart" if self.multiplayer else "chess.new_round"
        hint = self._tiny.render(t(hint_key), True, ui.TEXT_DIM)
        w = min(self.width - 24, max(head.get_width(), sub.get_width(),
                                     hint.get_width()) + 64)
        panel = pygame.Rect(cx - w // 2, self.height // 2 - 56, w, 112)
        ui.draw_panel(s, panel, shadow=False, accent_top=self.accent)
        s.blit(head, head.get_rect(center=(cx, panel.y + 34)))
        s.blit(sub, sub.get_rect(center=(cx, panel.y + 68)))
        s.blit(hint, hint.get_rect(center=(cx, panel.y + 94)))

    def _draw_setup(self, s):
        cx = self.width // 2
        ui.draw_title(s, self.width, t("chess.title"),
                      subtitle=t("chess.subtitle"), y=int(self.height * 0.14),
                      big=self._huge, small=self._small, accent=self.accent)
        # Schwierigkeits-Buttons (1-6)
        for i, rc in enumerate(self.diff_rects):
            ui.draw_button(s, rc, str(i + 1), self.font,
                           selected=(i == self.diff), accent=self.accent)
        name = self._small.render(t("chess.diff." + DIFFS[self.diff]), True, ui.TEXT)
        s.blit(name, name.get_rect(center=(cx, self.diff_rects[0].bottom + 18)))
        # Farbwahl
        labels = [t("chess.white"), t("chess.black")]
        for i, rc in enumerate(self.color_rects):
            on = (self.human_color == ("w" if i == 0 else "b"))
            ui.draw_button(s, rc, labels[i], self._small,
                           selected=on, accent=self.accent)
        # Start
        ui.draw_button(s, self.start_rect, t("common.start"), self.font,
                       selected=True, accent=self.accent)
        ui.draw_footer(s, self.width, self.height, t("chess.setup_hint"),
                       self._tiny)
