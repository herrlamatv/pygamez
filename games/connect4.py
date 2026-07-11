# -*- coding: utf-8 -*-
"""
connect4.py
===========
Vier gewinnt (Connect Four) - 1 Spieler gegen KI oder 2 Spieler lokal.

- Klassisches 7x6-Brett; Stein fällt animiert in die gewählte Spalte.
- Einzelspieler: KI mit drei Stärken (easy/medium/hard) über Minimax mit
  Alpha-Beta-Schnitt und Fenster-Bewertung; easy patzt absichtlich.
- Mehrspieler: Rot gegen Gelb abwechselnd am selben Rechner.
- Punkte (Highscore) = kumulierte Siege gegen die KI in einer Sitzung
  (tictactoe-Konvention); Mehrspieler wird nicht gewertet.

Steuerung: Maus (Spalte anklicken) oder Links/Rechts + Runter/Leertaste/
Enter; 1-7 wählt die Spalte direkt. Nach Rundenende: Enter = neue Runde,
S = Setup (nur Einzelspieler).
"""

import random

import pygame

import settings as settings_mod
from game_base import Game, InputEvent
from i18n import t

COL_BG = (13, 16, 27)
COL_PLATE = (38, 58, 140)
COL_PLATE_EDGE = (26, 40, 100)
COL_EMPTY = (18, 22, 36)
COL_P1 = (230, 90, 80)        # Rot
COL_P2 = (245, 205, 90)       # Gelb
COL_TEXT = (225, 228, 238)
COL_DIM = (150, 158, 178)
COL_ACCENT = (255, 143, 46)   # = Sidebar-Farbe #ff8f2e
COL_BTN = (40, 46, 66)
COL_BTN_ON = (92, 62, 36)
COL_BTN_BORDER = (74, 84, 116)

DIFFS = ["easy", "medium", "hard"]
DEPTHS = [2, 4, 5]

COLS, ROWS = 7, 6
ORDER = [3, 2, 4, 1, 5, 0, 6]     # Spaltenreihenfolge für Alpha-Beta

SETUP, PLAY, ANIM, OVER = "setup", "play", "anim", "over"


class ConnectFourGame(Game):
    name = "Vier gewinnt"
    highscore_key = "connect4"
    supports_multiplayer = True

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        c4 = self.settings.get("connect4", {}) if isinstance(self.settings, dict) else {}
        self.diff = max(0, min(2, int(c4.get("difficulty", 1))))

        self._small = pygame.font.SysFont("consolas", 16)
        self._tiny = pygame.font.SysFont("consolas", 13)
        self._huge = pygame.font.SysFont("consolas", max(26, self.height // 11),
                                         bold=True)
        self.wins = [0, 0]
        self.starter = 0
        self._build_setup_layout()
        self._new_round()
        self.state = PLAY if self.multiplayer else SETUP

    def on_surface_changed(self):
        self._huge = pygame.font.SysFont("consolas", max(26, self.height // 11),
                                         bold=True)
        self._build_setup_layout()
        self._layout()

    def _layout(self):
        self.hud_h = 46
        self.cell = int(min((self.width - 40) / 7,
                            (self.height - self.hud_h - 20) / 7))
        self.bw = 7 * self.cell
        self.bh = 6 * self.cell
        self.bx = (self.width - self.bw) // 2
        self.by = self.hud_h + self.cell
        self.r = int(self.cell * 0.42)

    def _new_round(self):
        self.board = [[0] * COLS for _ in range(ROWS)]   # [row][col], 0 oben
        self.player = self.starter                        # 0 = Rot, 1 = Gelb
        self.hover = 3
        self.anim = None          # dict(col, row, y, vy, player)
        self.win_cells = None
        self.winner = None
        self.msg = None
        self.msg_t = 0.0
        self.ai_delay = 0.0
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
            self.settings.setdefault("connect4", {})[key] = value
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
            for i, r in enumerate(self.diff_rects):
                if r.collidepoint(event.pos):
                    self.diff = i
                    self._save_setting("difficulty", i)
                    self.play_sound("click")
                    return
            if self.start_rect.collidepoint(event.pos):
                self._new_round()
                self.state = PLAY
                self.play_sound("click")

    # ===================================================== Brett-Logik
    def _drop_row(self, board, col):
        """Unterste freie Reihe der Spalte (oder None)."""
        for row in range(ROWS - 1, -1, -1):
            if board[row][col] == 0:
                return row
        return None

    def _check_win(self, board, row, col):
        """Gewinnzellen ab dem letzten Zug (oder None)."""
        p = board[row][col]
        for dr, dc in ((0, 1), (1, 0), (1, 1), (1, -1)):
            cells = [(row, col)]
            for sgn in (1, -1):
                rr, cc = row + dr * sgn, col + dc * sgn
                while 0 <= rr < ROWS and 0 <= cc < COLS and board[rr][cc] == p:
                    cells.append((rr, cc))
                    rr += dr * sgn
                    cc += dc * sgn
            if len(cells) >= 4:
                return cells
        return None

    def _is_full(self, board):
        return all(board[0][c] != 0 for c in range(COLS))

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.state == OVER:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self.starter = 1 - self.starter
                    self.game_over = False
                    self._new_round()
                    self.state = PLAY
                    self.play_sound("click")
                elif event.key in ("s", "S") and not self.multiplayer:
                    self.game_over = False
                    self.state = SETUP
                    self.play_sound("click")
            elif event.kind == InputEvent.MOUSEDOWN:
                self.starter = 1 - self.starter
                self.game_over = False
                self._new_round()
                self.state = PLAY
                self.play_sound("click")
            return
        if self.state != PLAY:
            return
        # Im Einzelspieler ist nur Spieler 0 (Rot) dran steuerbar.
        if not self.multiplayer and self.player == 1:
            return
        if event.kind == InputEvent.MOUSEMOVE:
            col = (event.pos[0] - self.bx) // self.cell
            if 0 <= col < COLS:
                self.hover = int(col)
        elif event.kind == InputEvent.MOUSEDOWN:
            col = (event.pos[0] - self.bx) // self.cell
            if 0 <= col < COLS:
                self._try_drop(int(col))
        elif event.kind == InputEvent.KEYDOWN:
            k = event.key
            if self.is_action(k, "left") or k == "Left":
                self.hover = (self.hover - 1) % COLS
                self.play_sound("move")
            elif self.is_action(k, "right") or k == "Right":
                self.hover = (self.hover + 1) % COLS
                self.play_sound("move")
            elif self.is_action(k, "down") or k in ("Down", "space", "Return"):
                self._try_drop(self.hover)
            elif k in "1234567":
                self._try_drop(int(k) - 1)
            elif k.startswith("KP_") and k[3:] in "1234567":
                self._try_drop(int(k[3:]) - 1)

    def _try_drop(self, col):
        row = self._drop_row(self.board, col)
        if row is None:
            self.msg = t("c4.col_full")
            self.msg_t = 1.2
            self.play_sound("click")
            return
        self.hover = col
        self.anim = dict(col=col, row=row, y=float(self.by - self.cell),
                         vy=0.0, player=self.player)
        self.state = ANIM
        self.play_sound("move")

    # ===================================================== Spiellogik
    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None

        if self.state == ANIM and self.anim is not None:
            a = self.anim
            a["vy"] += 2600 * dt
            a["y"] += a["vy"] * dt
            target = self.by + a["row"] * self.cell
            if a["y"] >= target:
                self.board[a["row"]][a["col"]] = a["player"] + 1
                self.play_sound("lock")
                self.anim = None
                cells = self._check_win(self.board, a["row"], a["col"])
                if cells:
                    self._ende(cells, a["player"])
                elif self._is_full(self.board):
                    self._ende(None, None)
                else:
                    self.player = 1 - self.player
                    self.state = PLAY
                    self.ai_delay = 0.35
            return

        if self.state == PLAY and not self.multiplayer and self.player == 1:
            self.ai_delay -= dt
            if self.ai_delay <= 0:
                self._try_drop(self._ai_move())

    def _ende(self, cells, winner):
        self.win_cells = cells
        self.winner = winner
        self.state = OVER
        if winner is not None:
            self.wins[winner] += 1
            if not self.multiplayer:
                if winner == 0:
                    self.score = self.wins[0]
                    self.play_sound("win")
                else:
                    self.play_sound("gameover")
            else:
                self.play_sound("win")
        self.game_over = True     # main.py speichert den Score einmalig

    # ===================================================== KI
    def _valid_cols(self, board):
        return [c for c in ORDER if board[0][c] == 0]

    def _ai_move(self):
        board = self.board
        valid = self._valid_cols(board)
        if not valid:
            return 3
        # 1) Sofortiger Sieg
        for c in valid:
            r = self._drop_row(board, c)
            board[r][c] = 2
            won = self._check_win(board, r, c)
            board[r][c] = 0
            if won:
                return c
        # 2) Sofortigen Verlust blocken (easy patzt zu 40 %)
        for c in valid:
            r = self._drop_row(board, c)
            board[r][c] = 1
            lose = self._check_win(board, r, c)
            board[r][c] = 0
            if lose and not (self.diff == 0 and random.random() < 0.4):
                return c
        # 3) Minimax mit Alpha-Beta
        depth = DEPTHS[self.diff]
        scores = []
        for c in valid:
            r = self._drop_row(board, c)
            board[r][c] = 2
            if self._check_win(board, r, c):
                board[r][c] = 0
                return c
            val = self._minimax(board, depth - 1, -1e9, 1e9, False)
            board[r][c] = 0
            scores.append((val, c))
        scores.sort(reverse=True)
        if self.diff == 0 and len(scores) > 1 and random.random() < 0.3:
            return scores[1][1]
        return scores[0][1]

    def _minimax(self, board, depth, alpha, beta, maximizing):
        valid = self._valid_cols(board)
        if depth == 0 or not valid:
            return self._evaluate(board)
        if maximizing:
            best = -1e9
            for c in valid:
                r = self._drop_row(board, c)
                board[r][c] = 2
                if self._check_win(board, r, c):
                    val = 100000 + depth
                else:
                    val = self._minimax(board, depth - 1, alpha, beta, False)
                board[r][c] = 0
                best = max(best, val)
                alpha = max(alpha, best)
                if alpha >= beta:
                    break
            return best
        best = 1e9
        for c in valid:
            r = self._drop_row(board, c)
            board[r][c] = 1
            if self._check_win(board, r, c):
                val = -100000 - depth
            else:
                val = self._minimax(board, depth - 1, alpha, beta, True)
            board[r][c] = 0
            best = min(best, val)
            beta = min(beta, best)
            if alpha >= beta:
                break
        return best

    def _evaluate(self, board):
        """Fenster-Bewertung aller 69 Viererfenster + Zentrums-Bonus."""
        score = 0
        for r in range(ROWS):
            if board[r][3] == 2:
                score += 6
        windows = []
        for r in range(ROWS):
            for c in range(COLS - 3):
                windows.append([board[r][c + i] for i in range(4)])
        for c in range(COLS):
            for r in range(ROWS - 3):
                windows.append([board[r + i][c] for i in range(4)])
        for r in range(ROWS - 3):
            for c in range(COLS - 3):
                windows.append([board[r + i][c + i] for i in range(4)])
                windows.append([board[r + 3 - i][c + i] for i in range(4)])
        vals = {2: 10, 3: 120}
        for w in windows:
            ai = w.count(2)
            hu = w.count(1)
            if hu == 0 and ai in vals:
                score += vals[ai]
            elif ai == 0 and hu in vals:
                score -= int(vals[hu] * 1.2)
        return score

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        s.fill(COL_BG)
        if self.state == SETUP:
            self._draw_setup(s)
            return
        self._layout_check()
        self._draw_hud(s)
        self._draw_board(s)
        if self.state == OVER:
            self._draw_over(s)

    def _layout_check(self):
        if not hasattr(self, "cell"):
            self._layout()

    def _disc_color(self, p):
        return COL_P1 if p == 0 else COL_P2

    def _draw_board(self, s):
        # Hover-Stein über dem Brett
        if self.state == PLAY and (self.multiplayer or self.player == 0):
            hx = self.bx + self.hover * self.cell + self.cell // 2
            hy = self.by - self.cell // 2
            pygame.draw.circle(s, self._disc_color(self.player), (hx, hy),
                               self.r)
            pygame.draw.circle(s, COL_BG, (hx, hy), self.r, 2)

        # Brettplatte
        plate = pygame.Rect(self.bx - 8, self.by - 8, self.bw + 16,
                            self.bh + 16)
        pygame.draw.rect(s, COL_PLATE, plate, border_radius=12)
        pygame.draw.rect(s, COL_PLATE_EDGE, plate, 3, border_radius=12)

        # Zellen (leer oder Stein)
        for r in range(ROWS):
            for c in range(COLS):
                cx = self.bx + c * self.cell + self.cell // 2
                cy = self.by + r * self.cell + self.cell // 2
                v = self.board[r][c]
                col = COL_EMPTY if v == 0 else self._disc_color(v - 1)
                pygame.draw.circle(s, col, (cx, cy), self.r)
                pygame.draw.circle(s, COL_PLATE_EDGE, (cx, cy), self.r, 2)

        # Fallender Stein (aufs Brett geclippt, damit er "hineinfällt")
        if self.anim is not None:
            prev = s.get_clip()
            s.set_clip(pygame.Rect(self.bx, self.by - self.cell,
                                   self.bw, self.bh + self.cell))
            a = self.anim
            cx = self.bx + a["col"] * self.cell + self.cell // 2
            cy = int(a["y"]) + self.cell // 2
            pygame.draw.circle(s, self._disc_color(a["player"]), (cx, cy),
                               self.r)
            s.set_clip(prev)

        # Sieg-Linie pulsierend hervorheben
        if self.win_cells:
            k = 0.5 + 0.5 * abs(pygame.time.get_ticks() % 1000 - 500) / 500
            for (r, c) in self.win_cells:
                cx = self.bx + c * self.cell + self.cell // 2
                cy = self.by + r * self.cell + self.cell // 2
                pygame.draw.circle(s, (255, 255, 255), (cx, cy),
                                   self.r + 2, max(2, int(4 * k)))

    def _draw_hud(self, s):
        pygame.draw.rect(s, (24, 29, 44), (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, (52, 60, 86), (0, self.hud_h),
                         (self.width, self.hud_h))
        cy = self.hud_h // 2
        n1 = t("common.player1") if self.multiplayer else t("c4.score_sp")
        n2 = t("common.player2") if self.multiplayer else t("common.ai")
        img = self._small.render(f"{n1}: {self.wins[0]}", True, COL_P1)
        s.blit(img, img.get_rect(midleft=(12, cy)))
        img = self._small.render(f"{self.wins[1]} :{n2}", True, COL_P2)
        s.blit(img, img.get_rect(midright=(self.width - 12, cy)))

        if self.state in (PLAY, ANIM):
            if not self.multiplayer and self.player == 1:
                mid = t("c4.ai_thinks")
            else:
                who = t("common.player1") if self.player == 0 \
                    else t("common.player2")
                mid = t("c4.turn", name=who)
            img = self._small.render(mid, True,
                                     self._disc_color(self.player))
            s.blit(img, img.get_rect(center=(self.width // 2, cy)))
        if self.msg:
            img = self._tiny.render(self.msg, True, (245, 160, 90))
            s.blit(img, img.get_rect(center=(self.width // 2,
                                             self.hud_h + 12)))

    def _draw_over(self, s):
        ov = pygame.Surface((self.width, 90), pygame.SRCALPHA)
        ov.fill((10, 12, 22, 200))
        y = self.height // 2 - 45
        s.blit(ov, (0, y))
        cx = self.width // 2
        if self.winner is None:
            head = self._huge.render(t("common.draw"), True, COL_DIM)
        elif self.multiplayer:
            head = self._huge.render(
                t("common.player_wins", n=self.winner + 1), True,
                self._disc_color(self.winner))
        else:
            key = "c4.win_you" if self.winner == 0 else "c4.win_ai"
            head = self._huge.render(t(key), True,
                                     self._disc_color(self.winner))
        s.blit(head, head.get_rect(center=(cx, y + 32)))
        hint = self._small.render(t("c4.new_round"), True, COL_DIM)
        s.blit(hint, hint.get_rect(center=(cx, y + 70)))

    # ----- Setup zeichnen -----------------------------------------------
    def _draw_setup(self, s):
        cx = self.width // 2
        title = self._huge.render("VIER GEWINNT", True, COL_ACCENT)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.14))))
        sub = self._small.render(t("c4.subtitle"), True, COL_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(self.height * 0.21))))
        for i, r in enumerate(self.diff_rects):
            on = (i == self.diff)
            pygame.draw.rect(s, COL_BTN_ON if on else COL_BTN, r,
                             border_radius=10)
            pygame.draw.rect(s, COL_ACCENT if on else COL_BTN_BORDER, r,
                             2 if on else 1, border_radius=10)
            lbl = self.font.render(t("c4.diff." + DIFFS[i]), True,
                                   COL_TEXT if on else COL_DIM)
            s.blit(lbl, lbl.get_rect(midleft=(r.x + 18, r.centery)))
        pygame.draw.rect(s, COL_BTN_ON, self.start_rect, border_radius=10)
        pygame.draw.rect(s, COL_ACCENT, self.start_rect, 2, border_radius=10)
        st = self.font.render(t("common.start"), True, COL_TEXT)
        s.blit(st, st.get_rect(center=self.start_rect.center))
        hint = self._tiny.render(t("c4.setup_hint"), True, COL_DIM)
        s.blit(hint, hint.get_rect(center=(cx, self.height - 16)))
