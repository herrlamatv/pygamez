# -*- coding: utf-8 -*-
"""
chess.py
========
Schach - gegen die KI (sechs Stärken), zu zweit am selben Rechner oder im
Rätsel-Modus mit 200 Aufgaben aus der Lichess-Rätseldatenbank.

Modi (MODES)
- Partie:     Mensch gegen KI. Setup: Stärke, Farbe, Schachuhr, Chess960.
- 2 Spieler:  Weiß gegen Schwarz abwechselnd. Setup: Schachuhr, Chess960,
              Brett nach jedem Zug drehen.
- Rätsel:     5 Stufen (Matt in 1/2/3, Taktik I/II) mit je 40 Aufgaben,
              Fortschritt in mem.json (Section "chess").

Während der Partie
- Maus: Figur anklicken oder ziehen; Pfeile/WASD + Leertaste/Enter.
- Seitenleiste: Spieler mit Uhr, geschlagene Figuren + Materialbilanz,
  Zugliste in SAN (Mausrad scrollt) und fünf Knöpfe:
  Rückgängig [U]/[Backspace] (gegen die KI zwei Halbzüge), Hinweis [H]
  (bester Zug als Pfeil), Brett drehen [F], Remis anbieten [O] und
  Aufgeben [X]. Rückgängig und Hinweis machen die Partie "unterstützt":
  ein Sieg zählt dann weder als Punkt noch für Statistik und Erfolge.
- Nach Partieende: [Enter] neue Partie (Farben wechseln), [S] Setup,
  [P] Partie als PGN speichern.
- Schachuhr: ohne / 1+0 / 3+2 / 5+0 / 10+5. Zeitüberschreitung verliert -
  außer der Gegner kann gar nicht mehr mattsetzen (dann Remis).
- Remis anbieten: die KI nimmt nur an, wenn sie ihre Stellung höchstens
  ausgeglichen bewertet; zu zweit entscheidet der Gegner.

KI (siehe chess_engine.py)
    Die Suche ist ein fortsetzbarer Generator: update() rechnet pro Frame
    nur ~7 ms daran weiter (kein Thread, kein Ruckeln); on_exit() bricht sie
    ab. Stufe 1 zieht zufällig, die Stufen 2-4 streuen absichtlich, Stufe 5
    und 6 suchen mit iterativer Vertiefung und Transpositionstabelle.

Punkte (Highscore) = Siege gegen die KI ohne Unterstützung in einer Sitzung.
Zu zweit und im Rätsel-Modus gibt es keine Highscore-Wertung.
"""

import json
import os
import random
import time

import pygame

import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent, LocalizedName
from i18n import t

from . import chess_engine as ce
from .chess_draw import ChessDraw

# ----------------------------------------------------------------- Zustände
SETUP, PLAY, OVER, PUZ_MENU, PUZ = "setup", "play", "over", "puz_menu", "puz"

DIFFS = ["lvl0", "lvl1", "lvl2", "lvl3", "lvl4", "lvl5"]
CLOCKS = ("none", "1+0", "3+2", "5+0", "10+5")
CLOCK_TIME = {"1+0": (60, 0), "3+2": (180, 2), "5+0": (300, 0), "10+5": (600, 5)}

FRAME_BUDGET = 0.007      # Rechenzeit der KI pro Frame (Sekunden)
AI_MIN_WAIT = 0.45        # so lange "denkt" die KI mindestens (Wanduhr)
ANIM_TIME = 0.2           # Dauer der Zug-Animation

STAGES = ("mate1", "mate2", "mate3", "tactic1", "tactic2")
MATE_N = {"mate1": 1, "mate2": 2, "mate3": 3}
PUZ_COLS = 8

PUZZLE_FILE = os.path.join(os.path.dirname(os.path.abspath(__file__)),
                           "levels", "chess-puzzles.json")
_PUZZLES = None


def load_puzzles():
    """Die Rätsel aus games/levels/chess-puzzles.json: {stufe: [rätsel]}."""
    global _PUZZLES
    if _PUZZLES is None:
        out = {sid: [] for sid in STAGES}
        try:
            with open(PUZZLE_FILE, "r", encoding="utf-8") as f:
                data = json.load(f)
            for st in data.get("stages", []):
                if st.get("id") in out:
                    out[st["id"]] = [p for p in st.get("puzzles", [])
                                     if isinstance(p.get("moves"), list)
                                     and len(p["moves"]) >= 2 and p.get("fen")]
        except (OSError, ValueError, AttributeError):
            pass
        _PUZZLES = out
    return _PUZZLES


class ChessGame(ChessDraw, Game):
    name = LocalizedName("Chess", de="Schach", fr="Échecs",
                         es="Ajedrez", pt="Xadrez")
    highscore_key = "chess"
    supports_multiplayer = True
    MODES = [("single", "chess.mode.single"), ("multi", "chess.mode.multi"),
             ("puzzles", "chess.mode.puzzles")]

    @property
    def show_highscore_banner(self):
        # Nur die Partie gegen die KI zählt in den Highscore.
        return self.mode == "single"

    @property
    def wants_escape(self):
        # ESC schließt offene Dialoge (Umwandlung, Aufgeben, Remis) statt zu
        # pausieren.
        return self.state in (PLAY, PUZ) and (self.promo is not None
                                              or self.dialog is not None)

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False
        if self.mode not in ("single", "multi", "puzzles"):
            self.mode = "single"
        self.multiplayer = self.mode == "multi"
        cs = self.settings.get("chess", {}) if isinstance(self.settings, dict) else {}
        try:
            self.diff = max(0, min(5, int(cs.get("difficulty", 2))))
        except (TypeError, ValueError):
            self.diff = 2
        self.human_color = ce.BLACK if cs.get("color") == "black" else ce.WHITE
        self.clock_opt = cs.get("clock") if cs.get("clock") in CLOCKS else "none"
        self.c960 = cs.get("chess960") is True
        self.auto_flip = cs.get("flip") is True

        self.human_wins = 0
        self.wins = [0, 0]              # 2 Spieler: Siege von Weiß / Schwarz
        self.tt = {}                    # Transpositionstabelle (bleibt je Sitzung)
        self.ai_job = None
        self.hint_job = None
        self.dialog = None
        self.promo = None
        self.msg = None
        self.msg_t = 0.0
        self.hover = None
        self.setup_focus = 0
        self.anims = []
        self.fades = []
        self.flip_anim = None
        self.list_top = 0
        self.list_follow = True
        self._reset_caches()
        self._make_fonts()
        self._layout()
        self._build_setup_layout()

        self.puzzles = load_puzzles()
        self._puz_load_progress()
        self.puz = None
        self.puz_cursor = 0
        self._build_puz_menu_layout()

        self._new_game()
        if self.mode == "puzzles":
            self.state = PUZ_MENU
            self._puz_menu_focus_unsolved()
        else:
            self.state = SETUP

    def _make_fonts(self):
        """Theme-Schriften, Größe abhängig von der Fensterhöhe."""
        h = self.height
        self._huge = ui.font(max(26, h // 12), bold=True)
        self._big = ui.font(max(17, h // 24), bold=True)
        self._small = ui.font(max(13, h // 34))
        self._tiny = ui.font(max(11, h // 46))
        self._clockf = ui.font(max(14, h // 30), bold=True, mono=True)
        self._listf = ui.font(max(11, h // 42), mono=True)

    def on_surface_changed(self):
        self._reset_caches()
        self._make_fonts()
        self._layout()
        self._build_setup_layout()
        self._build_puz_menu_layout()

    def on_exit(self):
        # Laufende Suchen verwerfen - der Generator hält nur eine Kopie der
        # Stellung, es bleibt also nichts halb gezogen zurück.
        self._abort_jobs()

    def _abort_jobs(self):
        for job in (self.ai_job, self.hint_job):
            if job and job.get("gen") is not None:
                try:
                    job["gen"].close()
                except Exception:
                    pass
        self.ai_job = None
        self.hint_job = None

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("chess", {})[key] = value
            settings_mod.save_settings(self.settings)

    # ===================================================== Layout
    def _layout(self):
        """Brett links (mit Koordinatenrand), Seitenleiste rechts."""
        w, h = self.width, self.height
        m = max(6, h // 64)
        frame = max(11, int(h * 0.03))
        side_min = max(140, int(w * 0.29))
        cell = int(min((h - 2 * m - 2 * frame) / 8,
                       (w - side_min - 3 * m - 2 * frame) / 8))
        self.cell = cell = max(14, cell)
        self.bw = 8 * cell
        pw = self.bw + 2 * frame
        self.frame = frame
        self.margin = m
        self.plate = pygame.Rect(m, (h - pw) // 2, pw, pw)
        self.bx = self.plate.x + frame
        self.by = self.plate.y + frame
        self.board_rect = pygame.Rect(self.bx, self.by, self.bw, self.bw)
        sx = self.plate.right + m
        sw = w - sx - m
        self.side_rect = pygame.Rect(sx, m, sw, h - 2 * m)
        gap = max(4, h // 90)
        self.gap = gap
        ph = self._small.get_height() + self._tiny.get_height() + 16
        self.top_panel = pygame.Rect(sx, m, sw, ph)
        self.bot_panel = pygame.Rect(sx, h - m - ph, sw, ph)
        n = 5
        bs = int(max(22, min(46, (sw - (n - 1) * gap) / n)))
        by = self.bot_panel.y - gap - bs
        total = n * bs + (n - 1) * gap
        bx0 = sx + (sw - total) // 2
        self.btn_rects = {bid: pygame.Rect(bx0 + i * (bs + gap), by, bs, bs)
                          for i, bid in enumerate(("undo", "hint", "flip", "draw",
                                                   "resign"))}
        sh = self._small.get_height() + 8
        self.status_rect = pygame.Rect(sx, self.top_panel.bottom + gap, sw, sh)
        ly = self.status_rect.bottom + gap
        self.list_rect = pygame.Rect(sx, ly, sw, by - gap - ly)
        self.row_h = self._listf.get_height() + 3
        # Rätsel: Knöpfe unten statt Spieler-Panel
        pbs = int(max(24, min(48, (sw - 3 * gap) / 4)))
        pby = h - m - pbs
        total = 4 * pbs + 3 * gap
        px0 = sx + (sw - total) // 2
        self.puz_btn_rects = {bid: pygame.Rect(px0 + i * (pbs + gap), pby, pbs, pbs)
                              for i, bid in enumerate(("list", "retry", "solution",
                                                       "next"))}
        # Info-Panel oben (Höhe nach Inhalt), darunter die Zugliste
        ih = (16 + self._big.get_height() + 2 * (self._tiny.get_height() + 2) + 6
              + self._small.get_height() + 4 + 2 * (self._tiny.get_height() + 1)
              + 6 + max(4, self._tiny.get_height() // 3) * 2 + 10
              + 2 * (self._big.get_height() + 1))
        ih = min(ih, max(60, (pby - gap - m) * 2 // 3))
        self.puz_info_rect = pygame.Rect(sx, m, sw, ih)
        self.puz_list_rect = pygame.Rect(sx, m + ih + gap, sw, pby - 2 * gap - m - ih)
        # Ergebnis-Panel über dem Brett
        # (Titel, bis zu zwei Zeilen Begründung, Knöpfe - bei schmalem Brett
        # zweireihig, damit die Beschriftungen in jeder Sprache passen)
        ow = min(self.bw - 12, max(220, int(self.bw * 0.9)))
        bh = max(24, h // 18)
        pad = 10
        bw3 = (ow - 2 * pad - 2 * gap) // 3
        labels = [t("chess.btn.new"), t("chess.btn.setup"), t("chess.btn.pgn")]
        two_rows = any(self._tiny.size(lb)[0] > bw3 - 8 for lb in labels)
        btn_h = bh * 2 + gap if two_rows else bh
        oh = int(self._huge.get_height() + 2 * self._small.get_height() + btn_h + 30)
        self.over_rect = pygame.Rect(self.bx + (self.bw - ow) // 2,
                                     self.by + (self.bw - oh) // 2, ow, oh)
        ox, oy = self.over_rect.x + pad, self.over_rect.bottom - pad - btn_h
        if two_rows:
            bw2 = (ow - 2 * pad - gap) // 2
            self.over_btns = {
                "new": pygame.Rect(ox, oy, bw2, bh),
                "setup": pygame.Rect(ox + bw2 + gap, oy, bw2, bh),
                "pgn": pygame.Rect(ox, oy + bh + gap, ow - 2 * pad, bh)}
        else:
            self.over_btns = {bid: pygame.Rect(ox + i * (bw3 + gap), oy, bw3, bh)
                              for i, bid in enumerate(("new", "setup", "pgn"))}

    def _title_bottom(self, y):
        """Unterkante von Titel + Untertitel (ui.draw_title, alle Themes)."""
        return (y + self._huge.get_height() // 2 + 36
                + self._small.get_height() // 2 + 4)

    def _build_setup_layout(self):
        """Optionszeilen des Setup-Screens (je Modus)."""
        w, h = self.width, self.height
        cx = w // 2
        if self.mode == "multi":
            rows = [("clock", len(CLOCKS)), ("c960", 2), ("flip", 2)]
        else:
            rows = [("diff", 6), ("color", 2), ("clock", len(CLOCKS)), ("c960", 2)]
        bw = min(w - 40, max(460, int(w * 0.56)))
        bh = max(24, min(44, int(h * 0.068)))
        lab = self._tiny.get_height() + 3
        top = max(int(h * 0.25), self._title_bottom(int(h * 0.09)) + 4)
        start_h = bh + 4
        bottom = h - 34 - start_h - max(6, h // 50)
        step = min(bh + lab + max(8, h // 40), (bottom - top) / len(rows))
        gap = 6
        self.setup_rows = []
        y = top
        for rid, n in rows:
            y_btn = int(y + lab)
            cw = (bw - gap * (n - 1)) / n
            rects = [pygame.Rect(int(cx - bw / 2 + i * (cw + gap)), y_btn,
                                 int(cw), bh) for i in range(n)]
            self.setup_rows.append((rid, rects))
            y += step
        sw = min(220, w - 60)
        self.start_rect = pygame.Rect(cx - sw // 2, int(y + max(4, h // 60)),
                                      sw, start_h)
        self.setup_focus = min(self.setup_focus, len(self.setup_rows))

    def _build_puz_menu_layout(self):
        w, h = self.width, self.height
        m = max(10, h // 40)
        top = self._title_bottom(int(h * 0.075)) + max(4, h // 60)
        tw = min(w - 2 * m, max(640, int(w * 0.78)))
        gap = max(4, h // 100)
        th = max(30, int(h * 0.09))
        n = len(STAGES)
        tcw = (tw - gap * (n - 1)) / n
        x0 = (w - tw) / 2
        self.stage_rects = [pygame.Rect(int(x0 + i * (tcw + gap)), top, int(tcw), th)
                            for i in range(n)]
        gy = top + th + max(8, h // 40)
        rows = 5
        start_h = max(24, min(40, h // 14))
        foot = 38 + start_h + max(4, h // 90)
        ch = int(min((h - gy - foot - gap * (rows - 1)) / rows, 60))
        cw = int(min((tw - gap * (PUZ_COLS - 1)) / PUZ_COLS, ch * 1.9))
        gw = PUZ_COLS * cw + (PUZ_COLS - 1) * gap
        gx = (w - gw) // 2
        self.grid_rects = [pygame.Rect(gx + (i % PUZ_COLS) * (cw + gap),
                                       gy + (i // PUZ_COLS) * (ch + gap), cw, ch)
                           for i in range(PUZ_COLS * rows)]
        by = gy + rows * (ch + gap) + max(2, h // 90)
        bw = min(240, w - 60)
        self.puz_start_rect = pygame.Rect((w - bw) // 2, by, bw, start_h)

    # ===================================================== Partie aufbauen
    def _new_game(self):
        self._abort_jobs()
        if self.c960 and self.mode in ("single", "multi"):
            self.c960_n = random.randint(0, 959)
            self.pos = ce.from_fen(ce.chess960_fen(self.c960_n), chess960=True)
        else:
            self.c960_n = None
            self.pos = ce.from_fen(ce.START_FEN, chess960=False)
        self.start_fen = self.pos.fen()
        self.start_side = self.pos.side
        self.start_full = self.pos.full
        self.moves = []
        self.sans = []
        self.ucis = []
        self.caps = []
        self.last_move = None
        self.result = None
        self.result_loser = None
        self.assisted = False
        self.sel = None
        self.targets = {}
        self.drag = None
        self.hint_move = 0
        self.dialog = None
        self.promo = None
        self.draw_block = 0
        self.ai_wait = 0.0
        self.ai_last_score = None
        self.low_time_warned = [False, False]
        if self.clock_opt in CLOCK_TIME:
            base, inc = CLOCK_TIME[self.clock_opt]
            self.clock = [float(base), float(base)]
            self.inc = inc
        else:
            self.clock = None
            self.inc = 0
        if self.mode == "single":
            self.view_black = self.human_color == ce.BLACK
        else:
            self.view_black = False
        self.flip_pending = 0.0
        self.cursor = [6, 4]
        self.anims = []
        self.fades = []
        self.list_top = 0
        self.list_follow = True
        self._refresh()

    def _refresh(self):
        pos = self.pos
        self.legal = pos.legal_moves()
        self.by_from = {}
        for m in self.legal:
            self.by_from.setdefault(m & 255, []).append(m)
        self.check = pos.in_check()

    # ===================================================== Koordinaten
    def _sq_to_disp(self, sq):
        r, f = sq >> 4, sq & 7
        return (r, 7 - f) if self.view_black else (7 - r, f)

    def _disp_to_sq(self, dr, dc):
        return dr * 16 + (7 - dc) if self.view_black else (7 - dr) * 16 + dc

    def _sq_rect(self, sq):
        dr, dc = self._sq_to_disp(sq)
        return pygame.Rect(self.bx + dc * self.cell, self.by + dr * self.cell,
                           self.cell, self.cell)

    def _sq_at(self, pos):
        c = (pos[0] - self.bx) // self.cell
        r = (pos[1] - self.by) // self.cell
        if 0 <= r < 8 and 0 <= c < 8:
            return self._disp_to_sq(int(r), int(c))
        return None

    @staticmethod
    def _move_squares(m):
        """(von, nach) für Markierungen - bei Rochaden das Königs-Zielfeld."""
        frm = m & 255
        to = (m >> 8) & 255
        if m >> 20 == ce.M_CASTLE:
            to = (frm & 0x70) + (6 if to > frm else 2)
        return frm, to

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
        elif self.state == PUZ_MENU:
            self._handle_puz_menu(event)
        elif self.state == OVER:
            self._handle_over(event)
        elif self.state in (PLAY, PUZ):
            self._handle_board_state(event)

    def _fixed_key(self, key, *names):
        return key in names and self.key_is_free(key)

    # ----------------------------------------------------------- Setup
    def _setup_values(self, rid):
        """(Anzahl, gewählter Index) einer Setup-Zeile."""
        if rid == "diff":
            return 6, self.diff
        if rid == "color":
            return 2, self.human_color
        if rid == "clock":
            return len(CLOCKS), CLOCKS.index(self.clock_opt)
        if rid == "c960":
            return 2, 1 if self.c960 else 0
        return 2, 1 if self.auto_flip else 0

    def _setup_set(self, rid, i):
        if rid == "diff":
            self.diff = i
            self._save_setting("difficulty", i)
        elif rid == "color":
            self.human_color = i
            self._save_setting("color", "black" if i == ce.BLACK else "white")
        elif rid == "clock":
            self.clock_opt = CLOCKS[i]
            self._save_setting("clock", self.clock_opt)
        elif rid == "c960":
            self.c960 = i == 1
            self._save_setting("chess960", self.c960)
        elif rid == "flip":
            self.auto_flip = i == 1
            self._save_setting("flip", self.auto_flip)

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            n_rows = len(self.setup_rows)
            if k in ("1", "2", "3", "4", "5", "6") and self.mode == "single":
                self._setup_set("diff", int(k) - 1)
                self.play_sound("click")
            elif k in ("c", "C") and self.mode == "single":
                self._setup_set("color", 1 - self.human_color)
                self.play_sound("select")
            elif k == "Up" or self.is_action(k, "up"):
                self.setup_focus = (self.setup_focus - 1) % (n_rows + 1)
                self.play_sound("move")
            elif k == "Down" or self.is_action(k, "down"):
                self.setup_focus = (self.setup_focus + 1) % (n_rows + 1)
                self.play_sound("move")
            elif k in ("Left", "Right") or self.is_action(k, "left") \
                    or self.is_action(k, "right"):
                if self.setup_focus < n_rows:
                    rid = self.setup_rows[self.setup_focus][0]
                    n, cur = self._setup_values(rid)
                    step = -1 if (k == "Left" or self.is_action(k, "left")) else 1
                    self._setup_set(rid, (cur + step) % n)
                    self.play_sound("move")
            elif k in ("Return", "space", "KP_Enter") or self.is_action(k, "action"):
                self._start_play()
        elif event.kind == InputEvent.MOUSEDOWN:
            for ri, (rid, rects) in enumerate(self.setup_rows):
                for i, rc in enumerate(rects):
                    if rc.collidepoint(event.pos):
                        self.setup_focus = ri
                        self._setup_set(rid, i)
                        self.play_sound("click")
                        return
            if self.start_rect.collidepoint(event.pos):
                self._start_play()

    def _start_play(self):
        self.game_over = False
        self._new_game()
        self.state = PLAY
        self.play_sound("click")
        ui.begin_transition()

    # ----------------------------------------------------------- Partie
    def _handle_board_state(self, event):
        if self.promo is not None:
            self._handle_promo(event)
            return
        if self.dialog is not None:
            self._handle_dialog(event)
            return
        k = event.key if event.kind == InputEvent.KEYDOWN else None
        if event.kind == InputEvent.MOUSEMOVE:
            self.hover = self._button_at(event.pos)
        if event.kind == InputEvent.WHEEL:
            rect = self.list_rect if self.state == PLAY else self.puz_list_rect
            if rect.collidepoint(event.pos):
                self._scroll_list(-event.delta * 2)
            return
        # Knöpfe der Seitenleiste
        if event.kind == InputEvent.MOUSEDOWN:
            bid = self._button_at(event.pos)
            if bid:
                self._press_button(bid)
                return
        if k is not None:
            if self.state == PLAY:
                if self._fixed_key(k, "u", "U", "BackSpace"):
                    self._undo()
                    return
                if self._fixed_key(k, "h", "H"):
                    self._request_hint()
                    return
                if self._fixed_key(k, "f", "F"):
                    self._flip_view()
                    return
                if self._fixed_key(k, "o", "O"):
                    self._offer_draw()
                    return
                if self._fixed_key(k, "x", "X"):
                    self._ask_resign()
                    return
                if k in ("Prior", "Next"):
                    self._scroll_list(-6 if k == "Prior" else 6)
                    return
            else:
                if self._fixed_key(k, "h", "H"):
                    self._puz_show_solution()
                    return
                if self._fixed_key(k, "u", "U", "BackSpace"):
                    self._puz_open(self.puz_stage, self.puz_idx)
                    return
                if self._fixed_key(k, "m", "M", "Tab"):
                    self._puz_back_to_menu()
                    return
                if self._fixed_key(k, "n", "N"):
                    self._puz_next()
                    return
                if self.puz_status in ("solved", "shown_done") and \
                        (k in ("Return", "space", "KP_Enter") or self.is_action(k, "action")):
                    self._puz_next()
                    return
        if not self._human_can_move():
            if event.kind == InputEvent.MOUSEUP:
                self.drag = None
            return
        if event.kind == InputEvent.MOUSEMOVE:
            sq = self._sq_at(event.pos)
            if sq is not None:
                self.cursor = list(self._sq_to_disp(sq))
            if self.drag is not None:
                self.drag["pos"] = event.pos
                if abs(event.pos[0] - self.drag["start"][0]) + \
                        abs(event.pos[1] - self.drag["start"][1]) > 4:
                    self.drag["moved"] = True
        elif event.kind == InputEvent.MOUSEDOWN:
            sq = self._sq_at(event.pos)
            if sq is None:
                self.sel = None
                self.targets = {}
                return
            self.cursor = list(self._sq_to_disp(sq))
            if self.sel is not None and sq in self.targets:
                self._try_move(sq, animate=True)
                return
            p = self.pos.b[sq]
            if p and (p >> 3) == self.pos.side:
                was = self.sel == sq
                self._select(sq)
                self.drag = {"sq": sq, "pos": event.pos, "start": event.pos,
                             "moved": False, "was": was}
                self.play_sound("click")
            else:
                self.sel = None
                self.targets = {}
        elif event.kind == InputEvent.MOUSEUP:
            drag = self.drag
            self.drag = None
            if drag is None:
                return
            sq = self._sq_at(event.pos)
            if sq is not None and sq != drag["sq"] and sq in self.targets:
                self._try_move(sq, animate=False)
            elif sq == drag["sq"] and drag["was"] and not drag["moved"]:
                self.sel = None             # zweiter Klick auf dieselbe Figur
                self.targets = {}
        elif k is not None:
            if k == "Up" or self.is_action(k, "up"):
                self._move_cursor(-1, 0)
            elif k == "Down" or self.is_action(k, "down"):
                self._move_cursor(1, 0)
            elif k == "Left" or self.is_action(k, "left"):
                self._move_cursor(0, -1)
            elif k == "Right" or self.is_action(k, "right"):
                self._move_cursor(0, 1)
            elif k in ("Return", "space", "KP_Enter") or self.is_action(k, "action"):
                sq = self._disp_to_sq(*self.cursor)
                if self.sel is not None and sq in self.targets:
                    self._try_move(sq, animate=True)
                else:
                    p = self.pos.b[sq]
                    if p and (p >> 3) == self.pos.side and self.sel != sq:
                        self._select(sq)
                        self.play_sound("click")
                    else:
                        self.sel = None
                        self.targets = {}

    def _move_cursor(self, dr, dc):
        self.cursor[0] = max(0, min(7, self.cursor[0] + dr))
        self.cursor[1] = max(0, min(7, self.cursor[1] + dc))
        self.play_sound("move")

    def _button_at(self, pos):
        rects = self.btn_rects if self.state == PLAY else self.puz_btn_rects
        for bid, rc in rects.items():
            if rc.collidepoint(pos):
                return bid
        return None

    def _press_button(self, bid):
        if bid == "undo":
            self._undo()
        elif bid == "hint":
            self._request_hint()
        elif bid == "flip":
            self._flip_view()
        elif bid == "draw":
            self._offer_draw()
        elif bid == "resign":
            self._ask_resign()
        elif bid == "list":
            self._puz_back_to_menu()
        elif bid == "retry":
            self._puz_open(self.puz_stage, self.puz_idx)
        elif bid == "solution":
            self._puz_show_solution()
        elif bid == "next":
            self._puz_next()

    def _human_can_move(self):
        if self.state == PUZ:
            return self.puz_status == "play" and self.pos.side == self.puz_player
        if self.state != PLAY:
            return False
        return self.mode == "multi" or self.pos.side == self.human_color

    def _select(self, sq):
        """Figur auswählen und ihre Zielfelder sammeln."""
        self.sel = sq
        moves = self.by_from.get(sq, [])
        normal = {(m >> 8) & 255 for m in moves if m >> 20 != ce.M_CASTLE}
        targets = {}
        for m in moves:
            to = (m >> 8) & 255
            if m >> 20 == ce.M_CASTLE:
                # Rochade: König auf den eigenen Turm ziehen - oder aufs
                # Zielfeld, solange das kein normaler Königszug ist.
                targets.setdefault(to, []).append(m)
                kt = (sq & 0x70) + (6 if to > sq else 2)
                if kt != sq and kt != to and kt not in normal:
                    targets.setdefault(kt, []).append(m)
            else:
                targets.setdefault(to, []).append(m)
        self.targets = targets

    def _try_move(self, sq, animate):
        cands = self.targets.get(sq, [])
        if not cands:
            return
        if len(cands) > 1:
            # Umwandlung: Figur wählen lassen (Dame, Turm, Läufer, Springer)
            order = {ce.QUEEN: 0, ce.ROOK: 1, ce.BISHOP: 2, ce.KNIGHT: 3}
            self.promo = sorted(cands, key=lambda m: order.get((m >> 16) & 7, 9))
            self.promo_animate = animate
            self.play_sound("select")
            return
        self._human_move(cands[0], animate)

    def _human_move(self, m, animate):
        if self.state == PUZ:
            self._puz_player_move(m, animate)
        else:
            self._commit(m, animate)

    def _promo_rects(self):
        n = 4
        size = max(36, min(int(self.cell * 1.15), (self.bw - 40) // n))
        gap = max(6, size // 8)
        total = size * n + gap * (n - 1)
        x0 = self.bx + (self.bw - total) // 2
        y = self.by + self.bw // 2 - size // 2 + self._small.get_height() // 2
        return [pygame.Rect(x0 + i * (size + gap), y, size, size) for i in range(n)]

    def _handle_promo(self, event):
        if event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self._promo_rects()):
                if rc.collidepoint(event.pos):
                    self._finish_promo(i)
                    return
            if not self.board_rect.collidepoint(event.pos):
                return
            self.promo = None               # Klick daneben: abbrechen
        elif event.kind == InputEvent.KEYDOWN:
            k = event.key.lower() if event.key else ""
            keys = {"q": 0, "d": 0, "r": 1, "t": 1, "b": 2, "l": 2, "n": 3, "s": 3,
                    "1": 0, "2": 1, "3": 2, "4": 3}
            if k in keys:
                self._finish_promo(keys[k])
            elif k in ("escape", "backspace"):
                self.promo = None

    def _finish_promo(self, i):
        cands = self.promo
        self.promo = None
        if cands and 0 <= i < len(cands):
            self._human_move(cands[i], getattr(self, "promo_animate", True))

    # ----------------------------------------------------------- Dialoge
    def _dialog_rects(self):
        w = min(self.bw - 16, max(240, int(self.bw * 0.8)))
        h = self._big.get_height() + self._small.get_height() + max(30, self.height // 15) + 40
        panel = pygame.Rect(self.bx + (self.bw - w) // 2, self.by + (self.bw - h) // 2,
                            w, h)
        bh = max(26, self.height // 16)
        bw = (w - 36) // 2
        yes = pygame.Rect(panel.x + 12, panel.bottom - 12 - bh, bw, bh)
        no = pygame.Rect(panel.right - 12 - bw, panel.bottom - 12 - bh, bw, bh)
        return panel, yes, no

    def _handle_dialog(self, event):
        answer = None
        if event.kind == InputEvent.MOUSEDOWN:
            _, yes, no = self._dialog_rects()
            if yes.collidepoint(event.pos):
                answer = True
            elif no.collidepoint(event.pos):
                answer = False
        elif event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("Return", "KP_Enter", "j", "J", "y", "Y"):
                answer = True
            elif k in ("Escape", "n", "N", "BackSpace"):
                answer = False
        if answer is None:
            return
        kind, side = self.dialog
        self.dialog = None
        self.play_sound("click")
        if kind == "resign" and answer:
            self._finish(1 - side, "resign")
        elif kind == "draw":
            if answer:
                self._finish(None, "agreed")
            else:
                self._toast(t("chess.draw_declined"))

    # ----------------------------------------------------------- Nach Partieende
    def _handle_over(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("Return", "space", "KP_Enter"):
                self._restart()
            elif k in ("s", "S"):
                self._to_setup()
            elif self._fixed_key(k, "p", "P"):
                self._export_pgn()
            elif self._fixed_key(k, "f", "F"):
                self._flip_view()
            elif k in ("Prior", "Next", "Up", "Down"):
                self._scroll_list(-3 if k in ("Prior", "Up") else 3)
        elif event.kind == InputEvent.MOUSEDOWN:
            for bid, rc in self.over_btns.items():
                if rc.collidepoint(event.pos):
                    if bid == "new":
                        self._restart()
                    elif bid == "setup":
                        self._to_setup()
                    else:
                        self._export_pgn()
                    return
        elif event.kind == InputEvent.WHEEL:
            if self.list_rect.collidepoint(event.pos):
                self._scroll_list(-event.delta * 2)
        elif event.kind == InputEvent.MOUSEMOVE:
            self.hover = None
            for bid, rc in self.over_btns.items():
                if rc.collidepoint(event.pos):
                    self.hover = "over_" + bid

    def _restart(self):
        self.game_over = False
        if self.mode == "single":
            # Farben wechseln nach jeder Partie
            self.human_color = 1 - self.human_color
            self._save_setting("color", "black" if self.human_color == ce.BLACK
                               else "white")
        self._new_game()
        self.state = PLAY
        self.play_sound("click")

    def _to_setup(self):
        self.game_over = False
        self.state = SETUP
        self.play_sound("click")

    def _list_rows(self):
        """Zeilen der Zugliste (beginnt die Stellung mit Schwarz, steht in
        der ersten Zeile links "…")."""
        return (len(self.sans) + self.start_side + 1) // 2

    def _list_visible(self):
        rect = self.puz_list_rect if self.state == PUZ else self.list_rect
        head = self._tiny.get_height() + 10
        return max(1, (rect.h - head) // self.row_h)

    def _scroll_list(self, rows):
        total = self._list_rows()
        visible = self._list_visible()
        top = max(0, min(total - visible, self.list_top + rows))
        self.list_top = top
        self.list_follow = top >= total - visible

    # ===================================================== Züge
    def _commit(self, m, animate=True):
        """Führt Zug m aus (Mensch, KI oder Rätsel-Automatik)."""
        pos = self.pos
        mover = pos.side
        frm = m & 255
        to = (m >> 8) & 255
        flag = m >> 20
        if flag == ce.M_EP:
            cap = ce.PAWN | ((1 - mover) << 3)
            cap_sq = to - 16 if mover == ce.WHITE else to + 16
        elif flag == ce.M_CASTLE:
            cap = 0
            cap_sq = -1
        else:
            cap = pos.b[to]
            cap_sq = to
        san = pos.san(m, self.legal)
        if animate:
            self._animate(m)
        if cap:
            self.fades.append({"p": cap, "sq": cap_sq, "t0": self._now()})
        self.sans.append(san)
        self.ucis.append(pos.uci(m))
        pos.make(m)
        self.moves.append(m)
        self.caps.append(cap)
        self.last_move = self._move_squares(m)
        self.sel = None
        self.targets = {}
        self.drag = None
        self.hint_move = 0
        if self.hint_job:
            self._abort_hint()
        if self.clock and len(self.moves) > 1:
            self.clock[mover] += self.inc
        self._refresh()
        if self.list_follow:
            self._scroll_to_end()
        if self.check:
            self.play_sound("select")
        elif flag == ce.M_CASTLE:
            self.play_sound("rotate")
        elif cap:
            self.play_sound("lock")
            self.rumble(40)
        else:
            self.play_sound("move")
        self.ai_wait = 0.0
        if self.state != PLAY:
            return
        status = None
        if not self.legal:
            status = "checkmate" if self.check else "stalemate"
        elif pos.insufficient_material():
            status = "material"
        elif pos.half >= 100:
            status = "fifty"
        elif pos.repetitions() >= 3:
            status = "threefold"
        if status:
            self._finish(mover if status == "checkmate" else None, status)
            return
        if self.mode == "multi" and self.auto_flip:
            self.flip_pending = ANIM_TIME + 0.25

    def _scroll_to_end(self):
        self.list_top = max(0, self._list_rows() - self._list_visible())
        self.list_follow = True

    def _animate(self, m):
        """Figur(en) gleiten lassen - gezeichnet in draw() über die Uhrzeit."""
        pos = self.pos
        frm = m & 255
        to = (m >> 8) & 255
        now = self._now()
        if m >> 20 == ce.M_CASTLE:
            base = frm & 0x70
            kt, rt = (base + 6, base + 5) if to > frm else (base + 2, base + 3)
            self.anims = [{"p": pos.b[frm], "a": frm, "b": kt, "t0": now},
                          {"p": pos.b[to], "a": to, "b": rt, "t0": now}]
        else:
            promo = (m >> 16) & 7
            p = (promo | (pos.side << 3)) if promo else pos.b[frm]
            self.anims = [{"p": p, "a": frm, "b": to, "t0": now}]

    @staticmethod
    def _now():
        return pygame.time.get_ticks() / 1000.0

    def _undo(self):
        if self.state != PLAY or not self.moves:
            return
        if self.mode == "single":
            human_moves = sum(1 for i in range(len(self.moves))
                              if self._mover(i) == self.human_color)
            if human_moves == 0:
                self._toast(t("chess.undo_none"))
                return
            n = 2 if self.pos.side == self.human_color else 1
        else:
            n = 1
        self._abort_jobs()
        for _ in range(min(n, len(self.moves))):
            self.pos.unmake()
            self.moves.pop()
            self.sans.pop()
            self.ucis.pop()
            self.caps.pop()
        if not self.assisted:
            self._toast(t("chess.assisted"))
        else:
            self._toast(t("chess.undone"))
        self.assisted = True
        self.last_move = self._move_squares(self.moves[-1]) if self.moves else None
        self.sel = None
        self.targets = {}
        self.hint_move = 0
        self.anims = []
        self.fades = []
        self.ai_wait = 0.0
        self.draw_block = min(self.draw_block, len(self.moves))
        self._refresh()
        self._scroll_to_end()
        if self.mode == "multi" and self.auto_flip:
            self.view_black = self.pos.side == ce.BLACK
        self.play_sound("rotate")

    @staticmethod
    def _mover(i):
        """Farbe, die den i-ten Halbzug einer Partie gespielt hat (Weiß beginnt)."""
        return i % 2

    def _flip_view(self):
        self.view_black = not self.view_black
        self.flip_anim = self._now()
        self.play_sound("rotate")

    # ----------------------------------------------------------- Hinweis
    def _request_hint(self):
        if self.state != PLAY or self.hint_job or not self.legal:
            return
        if self.mode == "single" and self.pos.side != self.human_color:
            return
        if not self.assisted:
            self._toast(t("chess.assisted"))
        self.assisted = True
        s = ce.Search(self.pos, ce.HINT_LEVEL, tt=self.tt)
        self.hint_job = {"search": s, "gen": s.run(), "spent": 0.0}
        self.play_sound("select")

    def _abort_hint(self):
        job = self.hint_job
        self.hint_job = None
        if job:
            try:
                job["gen"].close()
            except Exception:
                pass

    def _step_hint(self):
        job = self.hint_job
        s = job["search"]
        if not s.done:
            t0 = time.perf_counter()
            s.slice_end = t0 + FRAME_BUDGET
            try:
                next(job["gen"])
            except StopIteration:
                pass
            job["spent"] += time.perf_counter() - t0
        if s.done or (job["spent"] >= ce.HINT_LEVEL["time"] and s.depth_done >= 1):
            self._abort_hint()
            if s.best_move:
                self.hint_move = s.best_move
                self._toast(t("chess.hint_move", move=self.pos.san(s.best_move, self.legal)))

    # ----------------------------------------------------------- Remis / Aufgeben
    def _offer_draw(self):
        if self.state != PLAY or not self.moves:
            return
        if self.mode == "multi":
            self.dialog = ("draw", self.pos.side)
            self.play_sound("select")
            return
        if len(self.moves) < self.draw_block:
            self._toast(t("chess.draw_wait"))
            return
        ai = 1 - self.human_color
        if self.ai_last_score is not None:
            score = self.ai_last_score
        else:
            e = self.pos.evaluate()
            score = e if self.pos.side == ai else -e
        if score <= 20:
            self._toast(t("chess.draw_accepted"))
            self._finish(None, "agreed")
        else:
            self.draw_block = len(self.moves) + 6
            self._toast(t("chess.draw_declined_ai"))
            self.play_sound("hit")

    def _ask_resign(self):
        if self.state != PLAY:
            return
        side = self.human_color if self.mode == "single" else self.pos.side
        self.dialog = ("resign", side)
        self.play_sound("select")

    # ----------------------------------------------------------- Ende
    def _finish(self, winner, reason, loser=None):
        self._abort_jobs()
        self.state = OVER
        self.result = (winner, reason)
        self.result_loser = loser if loser is not None else (
            1 - winner if winner is not None else None)
        self.dialog = None
        self.promo = None
        self.sel = None
        self.targets = {}
        self.drag = None
        self.over_t0 = self._now()
        if reason == "checkmate":
            loser = 1 - winner
            r = self._sq_rect(self.pos.kings[loser])
            ui.spawn_burst(r.centerx, r.centery, ui.GOLD, n=30)
        if self.mode == "multi":
            if winner is not None:
                self.wins[winner] += 1
                self.play_sound("win")
            else:
                self.play_sound("select")
        else:
            if winner == self.human_color:
                if not self.assisted:
                    self.human_wins += 1
                    self.score = self.human_wins
                    self.report_result(True)
                    self.ach_event("chess_win")
                    if self.diff == 5:
                        self.ach_event("chess_master")
                self.play_sound("win")
            elif winner is None:
                self.play_sound("select")
            else:
                self.report_result(False)
                self.play_sound("gameover")
        self.game_over = True

    def _pgn(self):
        winner, reason = self.result if self.result else ("*", None)
        res = ce.pgn_result_code(winner)
        if self.mode == "single":
            ai = "%s (%s)" % (t("common.ai"), t("chess.diff." + DIFFS[self.diff]))
            you = t("chess.you")
            white, black = (you, ai) if self.human_color == ce.WHITE else (ai, you)
        else:
            white, black = t("common.player1"), t("common.player2")
        tags = [("Event", "PyGameZ"), ("Site", "PyGameZ"),
                ("Date", time.strftime("%Y.%m.%d")), ("Round", "-"),
                ("White", white), ("Black", black), ("Result", res)]
        if self.c960_n is not None:
            tags += [("Variant", "Chess960"), ("SetUp", "1"), ("FEN", self.start_fen)]
        if self.clock_opt in CLOCK_TIME:
            base, inc = CLOCK_TIME[self.clock_opt]
            tags.append(("TimeControl", "%d+%d" % (base, inc)))
        if reason in ("timeout", "timeout_draw"):
            tags.append(("Termination", "time forfeit"))
        return ce.pgn_text(tags, self.sans, res)

    def _export_pgn(self):
        if not self.sans:
            self._toast(t("chess.pgn_empty"))
            return
        import filepick
        name = "pygamez-chess-%s.pgn" % time.strftime("%Y-%m-%d-%H%M")
        if filepick.available():
            path = filepick.save_as(name, title=t("chess.pgn_title"), exts=(".pgn",))
            if not path:
                return
        else:
            path = filepick.to_downloads(name)
        try:
            with open(path, "w", encoding="utf-8", newline="\n") as f:
                f.write(self._pgn())
        except OSError:
            self._toast(t("chess.pgn_failed"))
            self.play_sound("hit")
            return
        self._toast(t("chess.pgn_saved", path=filepick.short(path, 34)), 4.0)
        self.play_sound("point")

    def _toast(self, text, dur=2.6):
        self.msg = text
        self.msg_t = dur

    # ===================================================== Update
    def update(self, dt):
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None
        if self.state == PLAY:
            if self.flip_pending > 0:
                self.flip_pending -= dt
                if self.flip_pending <= 0 and self.view_black != (self.pos.side == ce.BLACK):
                    self._flip_view()
            if self.clock and self.moves and self.dialog is None:
                self._tick_clock(dt)
                if self.state != PLAY:
                    return
            if self.hint_job:
                self._step_hint()
            if self.mode == "single" and self.pos.side != self.human_color \
                    and self.dialog is None:
                self._step_ai(dt)
        elif self.state == PUZ:
            self._puz_update(dt)

    def _tick_clock(self, dt):
        side = self.pos.side
        self.clock[side] -= dt
        if self.clock[side] <= 10 and not self.low_time_warned[side] \
                and self.clock[side] > 0:
            self.low_time_warned[side] = True
            if self.mode == "multi" or side == self.human_color:
                self.play_sound("hit")
        if self.clock[side] <= 0:
            self.clock[side] = 0.0
            winner = 1 - side
            if self.pos.can_mate(winner):
                self._finish(winner, "timeout")
            else:
                self._finish(None, "timeout_draw", loser=side)

    def _step_ai(self, dt):
        self.ai_wait += dt
        job = self.ai_job
        if job is None:
            if self.ai_wait < 0.12 or not self.legal:
                return
            job = self._start_ai()
        if job["move"]:
            if self.ai_wait >= job["wait"]:
                self.ai_job = None
                self._commit(job["move"])
            return
        s = job["search"]
        if not s.done:
            t0 = time.perf_counter()
            s.slice_end = t0 + FRAME_BUDGET
            try:
                next(job["gen"])
            except StopIteration:
                pass
            job["spent"] += time.perf_counter() - t0
        lvl = ce.LEVELS[self.diff]
        out = (job["spent"] >= lvl["time"] and s.depth_done >= 1) or \
            (self.ai_wait >= job["wall_cap"] and s.best_move)
        if (s.done or out) and self.ai_wait >= AI_MIN_WAIT:
            m = s.best_move or self.legal[0]
            if s.depth_done >= 1:
                self.ai_last_score = s.best_score
            self._abort_jobs()
            self._commit(m)

    def _start_ai(self):
        lvl = ce.LEVELS[self.diff]
        pos = self.pos
        job = {"move": 0, "wait": AI_MIN_WAIT, "search": None, "gen": None,
               "spent": 0.0, "wall_cap": float("inf")}
        if self.diff == 0 or random.random() < lvl["random"]:
            job["move"] = ce.pick_weak_move(pos)
            job["wait"] = random.uniform(0.5, 0.9)
        elif lvl["book"] and self.c960_n is None and len(self.moves) < 16:
            cands = [pos.parse_uci(u) for u in ce.book_moves(self.ucis)]
            cands = [c for c in cands if c]
            if cands:
                job["move"] = random.choice(cands)
                job["wait"] = random.uniform(0.45, 0.8)
        if not job["move"]:
            s = ce.Search(pos, lvl, tt=self.tt)
            job["search"] = s
            job["gen"] = s.run()
            if self.clock:
                left = self.clock[pos.side]
                job["wall_cap"] = max(0.2, min(10.0, left / 30.0 + self.inc * 0.7))
        self.ai_job = job
        return job

    # ===================================================== Rätsel
    def _puz_load_progress(self):
        data = store.load_section("chess")
        solved = data.get("puzzles_solved")
        seen = data.get("puzzles_seen")
        self.puz_solved = set(x for x in solved if isinstance(x, str)) \
            if isinstance(solved, list) else set()
        self.puz_seen = set(x for x in seen if isinstance(x, str)) \
            if isinstance(seen, list) else set()
        st = data.get("puzzle_stage")
        self.puz_stage = st if isinstance(st, int) and 0 <= st < len(STAGES) else 0

    def _puz_save_progress(self):
        data = store.load_section("chess")
        data["puzzles_solved"] = sorted(self.puz_solved)
        data["puzzles_seen"] = sorted(self.puz_seen)
        data["puzzle_stage"] = self.puz_stage
        store.save_section("chess", data)

    def _stage_list(self, stage=None):
        return self.puzzles.get(STAGES[self.puz_stage if stage is None else stage], [])

    def _stage_solved(self, stage):
        return sum(1 for p in self._stage_list(stage) if p["id"] in self.puz_solved)

    def _puz_menu_focus_unsolved(self):
        lst = self._stage_list()
        self.puz_cursor = 0
        for i, p in enumerate(lst):
            if p["id"] not in self.puz_solved:
                self.puz_cursor = i
                break

    def _handle_puz_menu(self, event):
        lst = self._stage_list()
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            n = len(lst)
            if k in ("1", "2", "3", "4", "5"):
                self._puz_set_stage(int(k) - 1)
            elif k in ("Tab", "Next", "Prior"):
                step = -1 if k == "Prior" else 1
                self._puz_set_stage((self.puz_stage + step) % len(STAGES))
            elif n and (k == "Left" or self.is_action(k, "left")):
                self.puz_cursor = (self.puz_cursor - 1) % n
                self.play_sound("move")
            elif n and (k == "Right" or self.is_action(k, "right")):
                self.puz_cursor = (self.puz_cursor + 1) % n
                self.play_sound("move")
            elif n and (k == "Up" or self.is_action(k, "up")):
                self.puz_cursor = (self.puz_cursor - PUZ_COLS) % n
                self.play_sound("move")
            elif n and (k == "Down" or self.is_action(k, "down")):
                self.puz_cursor = (self.puz_cursor + PUZ_COLS) % n
                self.play_sound("move")
            elif n and (k in ("Return", "space", "KP_Enter") or self.is_action(k, "action")):
                self._puz_open(self.puz_stage, self.puz_cursor)
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, rc in enumerate(self.stage_rects):
                if rc.collidepoint(event.pos):
                    self._puz_set_stage(i)
                    return
            for i, rc in enumerate(self.grid_rects[:len(lst)]):
                if rc.collidepoint(event.pos):
                    self.puz_cursor = i
                    self._puz_open(self.puz_stage, i)
                    return
            if lst and self.puz_start_rect.collidepoint(event.pos):
                self._puz_open(self.puz_stage, self.puz_cursor)
        elif event.kind == InputEvent.MOUSEMOVE:
            self.hover = None
            for i, rc in enumerate(self.grid_rects[:len(lst)]):
                if rc.collidepoint(event.pos):
                    self.hover = ("cell", i)

    def _puz_set_stage(self, i):
        if i == self.puz_stage:
            return
        self.puz_stage = i
        self._puz_menu_focus_unsolved()
        self.play_sound("select")

    def _puz_open(self, stage, idx):
        lst = self._stage_list(stage)
        if not lst:
            return
        idx = max(0, min(len(lst) - 1, idx))
        p = lst[idx]
        try:
            pos = ce.from_fen(p["fen"], chess960=False)
        except (ValueError, IndexError):
            return
        self._abort_jobs()
        self.puz = p
        self.puz_stage = stage
        self.puz_idx = idx
        self.puz_cursor = idx
        self.pos = pos
        self.start_fen = pos.fen()
        self.start_side = pos.side
        self.start_full = pos.full
        self.list_top = 0
        self.list_follow = True
        self.c960_n = None
        self.clock = None
        self.moves = []
        self.sans = []
        self.ucis = []
        self.caps = []
        self.last_move = None
        self.sel = None
        self.targets = {}
        self.drag = None
        self.promo = None
        self.dialog = None
        self.hint_move = 0
        self.anims = []
        self.fades = []
        self.puz_line = list(p["moves"])
        self.puz_step = 0
        self.puz_player = 1 - pos.side
        self.puz_mate = MATE_N.get(STAGES[stage], 0)
        self.puz_status = "intro"
        self.puz_timer = 0.7
        self.puz_mistakes = 0
        self.puz_shown = False
        self.puz_feedback = ("watch", None)
        self.view_black = self.puz_player == ce.BLACK
        self.cursor = [6, 4]
        self.state = PUZ
        self._refresh()
        self.play_sound("click")

    def _puz_update(self, dt):
        if self.puz_timer > 0:
            self.puz_timer -= dt
            if self.puz_timer > 0:
                return
        st = self.puz_status
        if st == "intro":
            m = self.pos.parse_uci(self.puz_line[0])
            if not m:
                self.puz_status = "shown_done"
                return
            self._commit(m)
            self.puz_step = 1
            self.puz_status = "play"
            self.puz_feedback = ("your_move", None)
        elif st == "reply":
            m = self.pos.parse_uci(self.puz_line[self.puz_step])
            if m:
                self._commit(m)
            self.puz_step += 1
            self.puz_status = "play"
            self.puz_feedback = ("good", ui.GREEN)
        elif st == "wrong":
            if self.moves:
                self.pos.unmake()
                self.moves.pop()
                self.sans.pop()
                self.ucis.pop()
                self.caps.pop()
                self.last_move = self._move_squares(self.moves[-1]) if self.moves else None
                self.anims = []
                self._refresh()
            self.puz_status = "play"
            self.puz_feedback = ("retry", ui.RED)
        elif st == "shown":
            if self.puz_step < len(self.puz_line):
                m = self.pos.parse_uci(self.puz_line[self.puz_step])
                if m:
                    self._commit(m)
                self.puz_step += 1
                self.puz_timer = 0.85
            else:
                self.puz_status = "shown_done"
                self.puz_feedback = ("shown", None)

    def _puz_player_move(self, m, animate):
        if self.puz_status != "play" or self.puz_step >= len(self.puz_line):
            return
        pos = self.pos
        expected = pos.parse_uci(self.puz_line[self.puz_step])
        mate = ce.gives_mate(pos, m)
        if m == expected or mate:
            self._commit(m, animate)
            self.puz_step += 1
            if mate or self.puz_step >= len(self.puz_line):
                self._puz_solved()
            else:
                self.puz_status = "reply"
                self.puz_timer = 0.5
                self.puz_feedback = ("good", ui.GREEN)
                self.play_sound("point")
        else:
            self._commit(m, animate)
            self.puz_status = "wrong"
            self.puz_timer = 0.85
            self.puz_mistakes += 1
            self.puz_feedback = ("wrong", ui.RED)
            self.play_sound("hit")
            self.rumble(90)

    def _puz_solved(self):
        self.puz_status = "solved"
        self.puz_feedback = ("solved", ui.GREEN)
        pid = self.puz["id"]
        if not self.puz_shown and pid not in self.puz_solved:
            self.puz_solved.add(pid)
            self._puz_save_progress()
            self.ach_event("chess_puzzles", len(self.puz_solved))
        self.play_sound("win")
        ui.spawn_burst(self.board_rect.centerx, self.board_rect.centery, ui.GOLD, n=34)

    def _puz_show_solution(self):
        if self.state != PUZ or self.puz_status in ("solved", "shown", "shown_done"):
            return
        if self.puz_status == "wrong" and self.moves:
            self.pos.unmake()
            self.moves.pop()
            self.sans.pop()
            self.ucis.pop()
            self.caps.pop()
            self._refresh()
        if self.puz_step == 0:
            m = self.pos.parse_uci(self.puz_line[0])
            if m:
                self._commit(m, animate=False)
            self.puz_step = 1
        self.puz_shown = True
        if self.puz["id"] not in self.puz_solved:
            self.puz_seen.add(self.puz["id"])
            self._puz_save_progress()
        self.sel = None
        self.targets = {}
        self.puz_status = "shown"
        self.puz_timer = 0.45
        self.puz_feedback = ("showing", None)
        self.play_sound("select")

    def _puz_next(self):
        if self.state != PUZ:
            return
        lst = self._stage_list()
        if self.puz_idx + 1 < len(lst):
            self._puz_open(self.puz_stage, self.puz_idx + 1)
        elif self.puz_stage + 1 < len(STAGES):
            self._puz_open(self.puz_stage + 1, 0)
        else:
            self._puz_back_to_menu()

    def _puz_back_to_menu(self):
        self._abort_jobs()
        self.state = PUZ_MENU
        self.promo = None
        self.dialog = None
        self._puz_save_progress()
        self.play_sound("click")
