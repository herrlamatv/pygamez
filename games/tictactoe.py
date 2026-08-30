# -*- coding: utf-8 -*-
"""
tictactoe.py
============
Tic-Tac-Toe mit Setup-Menü, mehreren Brettgrößen und KI-Schwierigkeiten.

- Setup-Screen: Schwierigkeit (Easy/Medium/Hard) und Brettgröße 3x3 .. 9x9.
- Allgemeines m,n,k-Spiel: gewonnen hat, wer K Steine in einer Reihe hat.
  Gewinnlänge K: 3x3 -> 3, 4x4 -> 4, ab 5x5 -> 5 (in einer Reihe).
- KI:
    Easy   - zufällige Züge.
    Medium - gewinnt/blockt sofort, sonst heuristisch bester Zug.
    Hard   - 3x3: volle Minimax-Suche (unschlagbar);
             größere Bretter: tiefenbegrenzte Alpha-Beta-Suche mit Heuristik.
- Modus: 1 Spieler (gegen die KI) oder 2 Spieler (lokal X gegen O).
- Spieler = X, KI/Spieler 2 = O, Klick-Steuerung.
- "Score"/Highscore = Anzahl gewonnener Runden (Siege) im 1-Spieler-Modus.
- Optik: Themen-Hintergrund, ui.*-Palette und responsives Layout
  (on_surface_changed baut Setup- und Brett-Geometrie neu auf).
"""

import random
import pygame

import i18n
import ui
from game_base import Game, InputEvent

# Identitätsfarben des Bretts (bewusst fest, unabhängig vom Theme).
COL_LINE = (90, 95, 120)     # Gitterlinien
COL_X = (90, 180, 255)       # Spieler X (blau)
COL_O = (255, 140, 90)       # Spieler O (orange)
COL_WIN = (240, 240, 120)    # Gewinnlinie
COL_WIN_BG = (60, 70, 40)    # Hinterlegung der Gewinnzellen

HUMAN = "X"
AI = "O"

SETUP, PLAY, OVER = "setup", "play", "over"

DIFF_ORDER = ["Easy", "Medium", "Hard"]
SIZES = [3, 4, 5, 6, 7, 8, 9]

WIN_SCORE = 10_000_000
# Bewertungsgewichte nach Anzahl eigener Steine in einem freien K-Fenster
LINE_WEIGHTS = [0, 1, 12, 120, 1200, 12000]


def win_length(n):
    """Benötigte Anzahl Steine in einer Reihe je Brettgröße."""
    if n <= 4:
        return n
    return 5


class TicTacToeGame(Game):
    name = "Tic-Tac-Toe"
    highscore_key = "tictactoe"
    supports_multiplayer = True      # Menü bietet "Mehrspieler (2 Spieler)"

    def reset(self):
        self.score = 0
        self.wins_x = 0
        self.wins_o = 0
        self.game_over = False
        self.state = SETUP

        self.diff_name = "Hard"
        self.size = 3

        self._make_fonts()
        self._build_setup_layout()

    # ===== Layout / Theme ===============================================

    def _make_fonts(self):
        """Themen-Schriften, Größen aus der Fensterhöhe abgeleitet."""
        h = self.height
        self.font = ui.font(max(16, h // 24))
        self.big_font = ui.font(max(30, h // 11), bold=True)
        self._small = ui.font(max(13, h // 32))
        self._mid = ui.font(max(15, h // 27), bold=True)

    def on_surface_changed(self):
        """Auflösungswechsel: Schriften und Layout neu aufbauen."""
        self._make_fonts()
        self._build_setup_layout()
        if self.state != SETUP:
            self._layout_board()

    @staticmethod
    def _diff_label(name):
        """Lokalisierter Anzeigename einer Schwierigkeit."""
        return i18n.t("ttt.diff." + name.lower())

    # ===== Setup-Screen =================================================

    def _build_setup_layout(self):
        cx = self.width // 2
        h = self.height
        bh = max(40, min(52, h // 10))

        self._diff_y = max(120, int(h * 0.25))
        self.diff_rects = {}
        for i, name in enumerate(DIFF_ORDER):
            self.diff_rects[name] = pygame.Rect(cx - 165 + i * 112, self._diff_y, 100, bh)

        self._size_y = self._diff_y + bh + max(60, int(h * 0.12))
        self.size_rects = {}
        bw, gap = 52, 6
        start = cx - (len(SIZES) * (bw + gap) - gap) // 2
        for i, n in enumerate(SIZES):
            self.size_rects[n] = pygame.Rect(start + i * (bw + gap), self._size_y, bw, bh)

        self.start_rect = pygame.Rect(cx - 90, self._size_y + bh + 46, 180, 50)

    def _draw_setup(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        titel = i18n.t("ttt.setup_title_mp") if self.multiplayer \
            else i18n.t("ttt.setup_title")
        ui.draw_title(s, self.width, titel, accent=self.accent)

        # Schwierigkeit (nur im 1-Spieler-Modus relevant -> sonst abgeblendet)
        label_col = ui.TEXT_FAINT if self.multiplayer else ui.TEXT_DIM
        img = self._mid.render(i18n.t("ttt.difficulty"), True, label_col)
        s.blit(img, (self.width // 2 - 165, self._diff_y - img.get_height() - 8))
        for name, r in self.diff_rects.items():
            if self.multiplayer:
                pygame.draw.rect(s, ui.BTN, r, border_radius=8)
                pygame.draw.rect(s, ui.BORDER, r, 1, border_radius=8)
                lab = self._mid.render(self._diff_label(name), True, ui.TEXT_FAINT)
                s.blit(lab, lab.get_rect(center=r.center))
            else:
                ui.draw_button(s, r, self._diff_label(name), self._mid,
                               selected=(name == self.diff_name),
                               accent=self.accent)

        img = self._mid.render(i18n.t("ttt.field"), True, ui.TEXT_DIM)
        s.blit(img, (self.width // 2 - 165, self._size_y - img.get_height() - 8))
        for n, r in self.size_rects.items():
            ui.draw_button(s, r, f"{n}x{n}", self._mid,
                           selected=(n == self.size), accent=self.accent)

        info = self._small.render(
            i18n.t("ttt.goal", k=win_length(self.size)), True, ui.TEXT_DIM)
        s.blit(info, info.get_rect(center=(self.width // 2, self.start_rect.y - 22)))

        ui.draw_button(s, self.start_rect, i18n.t("common.start"), self._mid,
                       selected=True, accent=ui.GREEN)

        ui.draw_footer(s, self.width, self.height, i18n.t("ttt.setup_hint"))

    def _handle_setup_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("1", "2", "3") and not self.multiplayer:
                self.diff_name = DIFF_ORDER[int(event.key) - 1]
                self.play_sound("select")
            elif event.key in ("Return", "space"):
                self._start_run()
        elif event.kind == InputEvent.MOUSEDOWN:
            p = event.pos
            for name, r in self.diff_rects.items():
                # Im Mehrspieler-Modus ist die Schwierigkeit abgeblendet.
                if r.collidepoint(p) and not self.multiplayer:
                    self.diff_name = name
                    self.play_sound("select")
            for n, r in self.size_rects.items():
                if r.collidepoint(p):
                    self.size = n
                    self.play_sound("select")
            if self.start_rect.collidepoint(p):
                self._start_run()

    # ===== Runde vorbereiten ============================================

    def _start_run(self):
        self.n = self.size
        self.k = win_length(self.n)
        self._max_depth = {3: 9, 4: 4, 5: 3}.get(self.n, 2)  # Suchtiefe für Hard
        self._precompute_windows()
        self._neue_runde()
        self.state = PLAY
        self.play_sound("click")

    def _neue_runde(self):
        n = self.n
        self.board = [""] * (n * n)
        self.current = HUMAN
        self.winner = None          # "X" | "O" | "Unentschieden" | None
        self.win_cells = None
        self.ai_timer = 0.0
        self.game_over = False
        self._layout_board()

    def _layout_board(self):
        """Brett-Geometrie (zentriertes Quadrat) aus width/height ableiten."""
        n = self.n
        self.board_size = min(self.width, self.height) - 90
        self.cell = self.board_size // n
        self.board_size = self.cell * n      # exakt durch n teilbar
        self.ox = (self.width - self.board_size) // 2
        self.oy = (self.height - self.board_size) // 2 + 10

    def _precompute_windows(self):
        """Alle K-Fenster (Reihen/Spalten/Diagonalen) für die Heuristik."""
        n, k = self.n, self.k
        self.windows = []
        for r in range(n):
            for c in range(n):
                # horizontal
                if c + k <= n:
                    self.windows.append([r * n + (c + i) for i in range(k)])
                # vertikal
                if r + k <= n:
                    self.windows.append([(r + i) * n + c for i in range(k)])
                # diagonal nach rechts unten
                if r + k <= n and c + k <= n:
                    self.windows.append([(r + i) * n + (c + i) for i in range(k)])
                # diagonal nach links unten
                if r + k <= n and c - k + 1 >= 0:
                    self.windows.append([(r + i) * n + (c - i) for i in range(k)])

    # ===== Eingabe ======================================================

    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup_event(event)
            return

        if self.state == OVER:
            if event.kind == InputEvent.KEYDOWN and event.key in ("s", "S"):
                self.state = SETUP
                self.play_sound("click")
            elif (event.kind == InputEvent.MOUSEDOWN or
                  (event.kind == InputEvent.KEYDOWN and event.key in ("Return", "space"))):
                self._neue_runde()
                self.state = PLAY
            return

        # PLAY: Klick setzt einen Stein
        if event.kind != InputEvent.MOUSEDOWN:
            return

        if self.multiplayer:
            # Beide Spieler sind menschlich: der aktuelle Spieler zieht.
            idx = self._feld_aus_pos(event.pos)
            if idx is not None and self.board[idx] == "":
                sym = self.current
                self._apply_move(idx, sym)
                if self.state == PLAY:
                    self.current = AI if sym == HUMAN else HUMAN
        elif self.current == HUMAN:
            idx = self._feld_aus_pos(event.pos)
            if idx is not None and self.board[idx] == "":
                self._apply_move(idx, HUMAN)
                if self.state == PLAY:
                    self.current = AI
                    self.ai_timer = 0.25

    def _feld_aus_pos(self, pos):
        x, y = pos
        if not (self.ox <= x < self.ox + self.board_size and
                self.oy <= y < self.oy + self.board_size):
            return None
        col = (x - self.ox) // self.cell
        row = (y - self.oy) // self.cell
        return int(row * self.n + col)

    # ===== Spiellogik ===================================================

    def update(self, dt):
        if self.multiplayer:
            return                       # kein KI-Zug im 2-Spieler-Modus
        if self.state != PLAY or self.current != AI:
            return
        # kleine Denkpause, damit der KI-Zug sichtbar ist
        self.ai_timer -= dt
        if self.ai_timer > 0:
            return
        zug = self._ai_decide()
        if zug is not None:
            self._apply_move(zug, AI)
            if self.state == PLAY:
                self.current = HUMAN

    def _apply_move(self, idx, sym):
        self.board[idx] = sym
        self.play_sound("click")
        cells = self._winning_cells(idx, sym)
        if cells:
            self.winner = sym
            self.win_cells = cells
            if sym == HUMAN:
                self.wins_x += 1
            else:
                self.wins_o += 1
            if not self.multiplayer and sym == HUMAN:
                self.score = self.wins_x
            self._ende()
        elif "" not in self.board:
            self.winner = "Unentschieden"
            self._ende()

    def _ende(self):
        self.state = OVER
        self.game_over = True     # damit main.py den Highscore speichert
        if not self.multiplayer and self.winner in (HUMAN, AI):
            self.report_result(self.winner == HUMAN)
        if self.winner == "Unentschieden":
            self.play_sound("point")
        elif not self.multiplayer and self.winner == AI:
            self.play_sound("gameover")
        else:
            self.play_sound("win")

    def _winning_cells(self, idx, sym):
        """Liefert die Gewinnzellen, falls 'sym' durch Zug auf idx gewinnt."""
        n, k = self.n, self.k
        r, c = divmod(idx, n)
        for dr, dc in ((0, 1), (1, 0), (1, 1), (1, -1)):
            cells = [idx]
            # in eine Richtung
            rr, cc = r + dr, c + dc
            while 0 <= rr < n and 0 <= cc < n and self.board[rr * n + cc] == sym:
                cells.append(rr * n + cc)
                rr += dr
                cc += dc
            # in die Gegenrichtung
            rr, cc = r - dr, c - dc
            while 0 <= rr < n and 0 <= cc < n and self.board[rr * n + cc] == sym:
                cells.append(rr * n + cc)
                rr -= dr
                cc -= dc
            if len(cells) >= k:
                return cells
        return None

    def _won_at(self, idx, sym):
        return self._winning_cells(idx, sym) is not None

    # ----- KI -----------------------------------------------------------

    def _empties(self):
        return [i for i, v in enumerate(self.board) if v == ""]

    def _candidates(self):
        """Sinnvolle Zugfelder: bei kleinen Brettern alle, sonst Nachbarn."""
        n = self.n
        empties = self._empties()
        if n <= 3:
            return empties
        belegt = [i for i, v in enumerate(self.board) if v != ""]
        if not belegt:
            return [n * n // 2]      # Mitte
        nah = set()
        for i in belegt:
            r, c = divmod(i, n)
            for dr in (-1, 0, 1):
                for dc in (-1, 0, 1):
                    rr, cc = r + dr, c + dc
                    if 0 <= rr < n and 0 <= cc < n:
                        j = rr * n + cc
                        if self.board[j] == "":
                            nah.add(j)
        return list(nah) if nah else empties

    def _find_winning(self, sym):
        """Feld, auf dem 'sym' sofort gewinnen würde (oder None)."""
        for i in self._empties():
            self.board[i] = sym
            gewinnt = self._won_at(i, sym)
            self.board[i] = ""
            if gewinnt:
                return i
        return None

    def _ai_decide(self):
        empties = self._empties()
        if not empties:
            return None
        # Erstes Feld: Mitte (gut und schnell)
        if len(empties) == self.n * self.n:
            return self.n * self.n // 2

        # Immer sofort gewinnen, wenn möglich
        gewinn = self._find_winning(AI)
        if gewinn is not None:
            return gewinn

        if self.diff_name == "Easy":
            return random.choice(empties)

        # Gegnerischen Sofortgewinn blocken
        block = self._find_winning(HUMAN)
        if block is not None:
            return block

        if self.diff_name == "Medium":
            return self._greedy()
        return self._search_root()

    def _evaluate(self):
        """Heuristik: Summe der Fensterwerte (AI positiv, HUMAN negativ)."""
        score = 0
        b = self.board
        for win in self.windows:
            ai = hu = 0
            for i in win:
                v = b[i]
                if v == AI:
                    ai += 1
                elif v == HUMAN:
                    hu += 1
            if ai and hu:
                continue                 # gemischt -> wertlos
            if ai:
                score += LINE_WEIGHTS[min(ai, len(LINE_WEIGHTS) - 1)]
            elif hu:
                score -= LINE_WEIGHTS[min(hu, len(LINE_WEIGHTS) - 1)]
        return score

    def _greedy(self):
        """Setzt den Stein dorthin, wo die Stellung am besten bewertet wird."""
        best, beste_idx = -1e18, None
        for idx in self._candidates():
            self.board[idx] = AI
            val = self._evaluate()
            self.board[idx] = ""
            if val > best:
                best, beste_idx = val, idx
        return beste_idx if beste_idx is not None else random.choice(self._empties())

    def _search_root(self):
        best, beste_idx = -1e18, None
        alpha, beta = -1e18, 1e18
        for idx in self._candidates():
            self.board[idx] = AI
            if self._won_at(idx, AI):
                val = WIN_SCORE
            else:
                val = self._search(self._max_depth - 1, alpha, beta, False)
            self.board[idx] = ""
            if val > best:
                best, beste_idx = val, idx
            alpha = max(alpha, best)
        return beste_idx if beste_idx is not None else random.choice(self._empties())

    def _search(self, depth, alpha, beta, maximizing):
        """Alpha-Beta-Suche; bewertet die Stellung aus AI-Sicht."""
        if depth == 0 or "" not in self.board:
            return self._evaluate()
        cand = self._candidates()
        if not cand:
            return self._evaluate()

        if maximizing:                      # AI am Zug
            best = -1e18
            for idx in cand:
                self.board[idx] = AI
                if self._won_at(idx, AI):
                    val = WIN_SCORE - (self._max_depth - depth)
                else:
                    val = self._search(depth - 1, alpha, beta, False)
                self.board[idx] = ""
                best = max(best, val)
                alpha = max(alpha, best)
                if beta <= alpha:
                    break
            return best
        else:                               # HUMAN am Zug
            best = 1e18
            for idx in cand:
                self.board[idx] = HUMAN
                if self._won_at(idx, HUMAN):
                    val = -WIN_SCORE + (self._max_depth - depth)
                else:
                    val = self._search(depth - 1, alpha, beta, True)
                self.board[idx] = ""
                best = min(best, val)
                beta = min(beta, best)
                if beta <= alpha:
                    break
            return best

    # ===== Zeichnen =====================================================

    def draw(self):
        if self.state == SETUP:
            self._draw_setup()
            return

        s = self.surface
        ui.draw_background(s, self.width, self.height)
        n, cell = self.n, self.cell
        lw = max(2, cell // 20)             # Linienbreite passend zur Zellgröße

        # Gewinnzellen hinterlegen (unter dem Gitter)
        if self.win_cells:
            for idx in self.win_cells:
                r, c = divmod(idx, n)
                rect = pygame.Rect(self.ox + c * cell, self.oy + r * cell, cell, cell)
                pygame.draw.rect(s, COL_WIN_BG, rect)

        # Gitter
        for i in range(1, n):
            x = self.ox + i * cell
            y = self.oy + i * cell
            pygame.draw.line(s, COL_LINE, (x, self.oy), (x, self.oy + self.board_size), lw)
            pygame.draw.line(s, COL_LINE, (self.ox, y), (self.ox + self.board_size, y), lw)
        pygame.draw.rect(s, COL_LINE,
                         (self.ox, self.oy, self.board_size, self.board_size), lw)

        # Symbole
        rad = int(cell * 0.30)
        mw = max(3, cell // 12)
        for i, sym in enumerate(self.board):
            if not sym:
                continue
            r, c = divmod(i, n)
            mx = self.ox + c * cell + cell // 2
            my = self.oy + r * cell + cell // 2
            if sym == HUMAN:
                pygame.draw.line(s, COL_X, (mx - rad, my - rad), (mx + rad, my + rad), mw)
                pygame.draw.line(s, COL_X, (mx + rad, my - rad), (mx - rad, my + rad), mw)
            else:
                pygame.draw.circle(s, COL_O, (mx, my), rad, mw)

        # Gewinnlinie
        if self.win_cells and len(self.win_cells) >= 2:
            a, z = self.win_cells[0], self.win_cells[-1]
            ar, ac = divmod(a, n)
            zr, zc = divmod(z, n)
            p1 = (self.ox + ac * cell + cell // 2, self.oy + ar * cell + cell // 2)
            p2 = (self.ox + zc * cell + cell // 2, self.oy + zr * cell + cell // 2)
            pygame.draw.line(s, COL_WIN, p1, p2, max(4, cell // 10))

        # Kopfzeile
        if self.multiplayer:
            kopf = i18n.t("ttt.header_mp", n=n, k=self.k)
            stand = self._small.render(
                i18n.t("ttt.score_mp", x=self.wins_x, o=self.wins_o), True, ui.TEXT)
        else:
            kopf = i18n.t("ttt.header_sp", diff=self._diff_label(self.diff_name),
                          n=n, k=self.k)
            stand = self._small.render(
                i18n.t("ttt.score_sp", wins=self.wins_x), True, ui.TEXT)
        s.blit(self._small.render(kopf, True, ui.TEXT_DIM), (10, 8))
        s.blit(stand, (self.width - stand.get_width() - 10, 8))
        if self.state == PLAY:
            if self.multiplayer:
                zug = i18n.t("ttt.turn_p1") if self.current == HUMAN \
                    else i18n.t("ttt.turn_p2")
            else:
                zug = i18n.t("ttt.turn_you") if self.current == HUMAN \
                    else i18n.t("ttt.turn_ai")
            s.blit(self._small.render(zug, True, ui.TEXT),
                   (10, self.height - self._small.get_height() - 8))

        if self.state == OVER:
            self._draw_over()

    def _draw_over(self):
        s = self.surface
        if self.winner == "Unentschieden":
            msg, farbe = i18n.t("common.draw"), ui.GOLD
        elif self.multiplayer:
            if self.winner == HUMAN:
                msg, farbe = i18n.t("ttt.win_p1"), COL_X
            else:
                msg, farbe = i18n.t("ttt.win_p2"), COL_O
        elif self.winner == HUMAN:
            msg, farbe = i18n.t("ttt.win_you"), ui.GREEN
        else:
            msg, farbe = i18n.t("ttt.win_ai"), ui.RED

        # Kompaktes, transluzentes Panel statt Vollbild-Abdunklung.
        zeilen = [(msg, self.big_font, farbe),
                  (i18n.t("ttt.new_round"), self.font, None),   # None -> pulsierend
                  (i18n.t("ttt.settings"), self._small, ui.TEXT_DIM)]
        bw = max(f.size(txt)[0] for txt, f, _ in zeilen) + 80
        bh = sum(f.get_height() for _, f, _ in zeilen) + 66
        rect = pygame.Rect(0, 0, bw, bh)
        rect.center = (self.width // 2, self.height // 2)

        panel = pygame.Surface(rect.size, pygame.SRCALPHA)
        pygame.draw.rect(panel, (*ui.PANEL[:3], 235), panel.get_rect(), border_radius=16)
        pygame.draw.rect(panel, farbe, panel.get_rect(), 2, border_radius=16)
        s.blit(panel, rect)

        y = rect.y + 20
        for txt, f, col in zeilen:
            if col is None:
                col = ui.mix(ui.TEXT_DIM, ui.TEXT, ui.pulse(2.0, 0.2, 1.0))
            img = f.render(txt, True, col)
            s.blit(img, img.get_rect(midtop=(rect.centerx, y)))
            y += f.get_height() + 11
