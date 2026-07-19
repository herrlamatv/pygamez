# -*- coding: utf-8 -*-
"""
sudoku.py
=========
Sudoku mit 400 deterministischen Leveln (4 Stufen x 100) und vier Spielmodi.

Level-System
------------
Die Puzzles kommen aus games/sudoku_gen.py: Level N von Stufe D ist immer
dasselbe Puzzle (Seed-generiert, eindeutig lösbar). Gelöste Level werden je
Stufe in mem.json (Abschnitt "sudoku") gespeichert und in der Levelauswahl
abgehakt.

Spielmodi (Auswahl im Vorspiel-Screen, aufsteigende Hilfe-Stufen)
-----------------------------------------------------------------
- classic  (x2,0 Punkte): keine Hilfen - pur wie auf Papier.
- notes    (x1,5 Punkte): + Bleistift-Notizen (N bzw. Notiz-Button).
- comfort  (x1,0 Punkte): + falsche Ziffern sofort rot, Konflikt- und
             Gleiche-Ziffer-Hervorhebung, korrekte Eingaben rasten ein.
- assist   (x0,7 Punkte): + Tipp-Funktion (H, max. 3, kostet Punkte).

Fehler-Regel (in ALLEN Modi gleich)
-----------------------------------
Jede Eingabe wird sofort gegen die eindeutige Lösung geprüft; eine falsche
Ziffer zählt als Fehler. Ist das 3-Fehler-Limit aktiv (Option im Setup),
ist beim dritten Fehler die Partie verloren. Die Modi unterscheiden nur die
ANZEIGE: classic/notes zeigen Fehler nicht rot an (nur der HUD-Zähler).

Punkte: (Basis je Stufe - Zeit - Fehler - Tipps) x Modus-Multiplikator.

Steuerung: Pfeile/WASD = Zelle wählen, 1-9 = Ziffer, 0/Backspace/Entf =
radieren (auch Rechtsklick), N = Notizen, H = Tipp, R = Level neu,
Q = Levelwahl. Maus: Zellen und Ziffernfeld anklicken.
"""

import random

import pygame

import settings as settings_mod
import store
import ui
from game_base import Game, InputEvent
from i18n import t

from . import sudoku_gen

# ----- Farben ---------------------------------------------------------------
# Eigene Ziffern bleiben bewusst blau (klassische Sudoku-Optik). Alle
# generischen UI-Farben kommen zur Laufzeit dynamisch aus der ui-Palette;
# die Zellhintergründe werden in _draw_board() aus Palette + Akzent gemischt.
COL_USER = (110, 165, 255)       # eigene Ziffern

# (i18n-Suffix, Basispunkte) je Schwierigkeitsgrad; Vorgaben-Anzahl siehe
# sudoku_gen.CLUES.
DIFFICULTIES = [("easy", 1000), ("normal", 2000), ("hard", 3500), ("expert", 5000)]

MODE_MULT = {"classic": 2.0, "notes": 1.5, "comfort": 1.0, "assist": 0.7}

MAX_HINTS = 3
HINT_COST = 200      # Punktabzug je Tipp
ERR_COST = 150       # Punktabzug je Fehler
TIME_COST = 2        # Punktabzug je Sekunde
FAIL_LIMIT = 3

SETUP, GENERATING, PLAY = "setup", "generating", "play"


class SudokuGame(Game):
    name = "Sudoku"
    highscore_key = "sudoku"
    supports_multiplayer = False
    wants_right_click = True     # Rechtsklick = radieren

    # Vorspiel-Screen zeigt diese Modi statt Einzel-/Mehrspieler.
    MODES = [("classic", "sud.mode.classic"), ("notes", "sud.mode.notes"),
             ("comfort", "sud.mode.comfort"), ("assist", "sud.mode.assist")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.score = 0
        self.game_over = False

        sud = self.settings.get("sudoku", {}) if isinstance(self.settings, dict) else {}
        self.diff = max(0, min(3, int(sud.get("difficulty", 0))))
        self.fail_limit = bool(sud.get("fail_limit", True))
        self._last_level = dict(sud.get("last_level", {}))

        # Modus-Fähigkeiten aus dem gewählten mode ableiten.
        self.can_notes = self.mode in ("notes", "comfort", "assist")
        self.can_check = self.mode in ("comfort", "assist")   # rot + Konflikte
        self.can_hint = self.mode == "assist"

        self._build_fonts()
        self._overlay = None

        self._load_progress()
        self.cursor = self._last_level.get(str(self.diff), 1)
        self.level = 1
        self.msg = None
        self.msg_t = 0.0
        self._gen_drawn = False

        self._build_setup_layout()
        self.state = SETUP

    def _build_fonts(self):
        """Schriftgrößen aus der aktuellen Auflösung ableiten (Theme-Fonts)."""
        h = self.height
        self._small = ui.font(max(14, h // 30))
        self._tiny = ui.font(max(11, h // 36))
        self._huge = ui.font(max(26, h // 11), bold=True)
        # Uhr mit fester Zeichenbreite, damit sie beim Ticken nicht "zittert".
        self._clock_font = ui.font(max(18, h // 22), mono=True)

    def on_surface_changed(self):
        """Layout nach einer Auflösungsänderung neu berechnen (Options-Screen)."""
        self._build_fonts()
        self._overlay = None
        self._build_setup_layout()
        if self.state == PLAY:
            self._build_play_layout()

    # ----- Persistenz ------------------------------------------------------

    def _load_progress(self):
        """Liest die gelösten Level je Stufe aus mem.json (Abschnitt 'sudoku')."""
        data = store.load_section("sudoku")
        solved = data.get("solved") if isinstance(data.get("solved"), dict) else {}
        self.solved = {}
        for k in ("0", "1", "2", "3"):
            lst = solved.get(k, [])
            if isinstance(lst, list):
                self.solved[k] = sorted({int(v) for v in lst
                                         if isinstance(v, int)
                                         and 1 <= v <= sudoku_gen.LEVELS})
            else:
                self.solved[k] = []

    def _save_progress(self):
        store.save_section("sudoku", {"solved": self.solved})

    def _save_setting(self, key, value):
        if isinstance(self.settings, dict):
            self.settings.setdefault("sudoku", {})[key] = value
            settings_mod.save_settings(self.settings)

    # ===================================================== Setup-Screen
    def _build_setup_layout(self):
        W, H = self.width, self.height
        cx = W // 2

        # 4 Stufen-Buttons nebeneinander
        bw = min(150, (W - 60) // 4 - 8)
        total = 4 * bw + 3 * 10
        self.diff_rects = [pygame.Rect(cx - total // 2 + i * (bw + 10),
                                       int(H * 0.16), bw, 34)
                           for i in range(4)]

        # Fehler-Limit-Toggle darunter
        self.limit_rect = pygame.Rect(cx - 150, int(H * 0.16) + 44, 300, 26)

        # 10x10-Levelraster: füllt den Platz zwischen Toggle und Fußzeile
        top = self.limit_rect.bottom + 30
        avail_h = H - top - 60
        avail_w = W - 80
        cell = max(18, min(avail_w // 10, avail_h // 10))
        self.lv_cell = cell
        self.lv_x = cx - cell * 5
        self.lv_y = top
        self._lv_font = ui.font(max(10, cell * 2 // 5), mono=True)

    def _level_at(self, pos):
        """Pixel -> Levelnummer 1..100 (oder None)."""
        x, y = pos
        c = (x - self.lv_x) // self.lv_cell
        r = (y - self.lv_y) // self.lv_cell
        if 0 <= c < 10 and 0 <= r < 10:
            return int(r * 10 + c + 1)
        return None

    def _select_difficulty(self, i):
        self.diff = max(0, min(3, i))
        self._save_setting("difficulty", self.diff)
        self.cursor = self._last_level.get(str(self.diff), 1)
        self.play_sound("click")

    def _toggle_fail_limit(self):
        self.fail_limit = not self.fail_limit
        self._save_setting("fail_limit", self.fail_limit)
        self.play_sound("select")

    def _handle_setup(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("1", "2", "3", "4"):
                self._select_difficulty(int(k) - 1)
            elif k in ("f", "F"):
                self._toggle_fail_limit()
            elif k in ("Left", "a", "A"):
                self.cursor = (self.cursor - 2) % 100 + 1
                self.play_sound("move")
            elif k in ("Right", "d", "D"):
                self.cursor = self.cursor % 100 + 1
                self.play_sound("move")
            elif k in ("Up", "w", "W"):
                self.cursor = (self.cursor - 11) % 100 + 1
                self.play_sound("move")
            elif k in ("Down", "s", "S"):
                self.cursor = (self.cursor + 9) % 100 + 1
                self.play_sound("move")
            elif k in ("Return", "space"):
                self._start_level(self.cursor)
        elif event.kind == InputEvent.MOUSEMOVE:
            lv = self._level_at(event.pos)
            if lv is not None:
                self.cursor = lv
        elif event.kind == InputEvent.MOUSEDOWN:
            for i, r in enumerate(self.diff_rects):
                if r.collidepoint(event.pos):
                    self._select_difficulty(i)
                    return
            if self.limit_rect.collidepoint(event.pos):
                self._toggle_fail_limit()
                return
            lv = self._level_at(event.pos)
            if lv is not None:
                self._start_level(lv)

    # ===================================================== Level starten
    def _start_level(self, n):
        self.level = max(1, min(sudoku_gen.LEVELS, int(n)))
        self.cursor = self.level
        self._last_level[str(self.diff)] = self.level
        self._save_setting("last_level", self._last_level)
        self.game_over = False
        self.won = False
        self.score = 0
        self._gen_drawn = False
        self.state = GENERATING
        self.play_sound("click")

    def _do_generate(self):
        """Erzeugt das Puzzle (blockierend, < 1s) und initialisiert das Brett."""
        puzzle, solution = sudoku_gen.generate(self.diff, self.level)
        self.solution = solution
        self.board = list(puzzle)
        self.given = [v != 0 for v in puzzle]
        self.locked = list(self.given)
        self.notes = [set() for _ in range(81)]
        self.wrong = set()
        self.sel = next((i for i in range(81) if not self.given[i]), 0)
        self.errors = 0
        self.hints_used = 0
        self.elapsed = 0.0
        self.note_mode = False
        self.reveal = False      # nach Spielende: Lösung statt Banner zeigen
        self.msg = None
        self.msg_t = 0.0
        self._conflicts = frozenset()
        self._build_play_layout()
        self.state = PLAY

    # ===================================================== Spiel-Layout
    def _build_play_layout(self):
        W, H = self.width, self.height
        self.hud_h = max(40, H // 12)

        # Brett links/mittig, Ziffernfeld rechts daneben. Unten bleiben 28px
        # für die Steuerungs-Hinweiszeile frei.
        pad_w = max(120, W // 5)
        size = min(H - self.hud_h - 40, W - pad_w - 40)
        self.cell = max(20, size // 9)
        bs = self.cell * 9
        self.bx = max(12, (W - pad_w - bs) // 2)
        self.by = self.hud_h + (H - self.hud_h - 28 - bs) // 2

        # Mono-Fonts: Ziffern stehen so in jeder Zelle exakt gleich breit.
        self._num_font = ui.font(max(14, self.cell * 3 // 5),
                                 bold=True, mono=True)
        self._note_font = ui.font(max(8, self.cell * 2 // 7), mono=True)

        # Ziffernfeld: 3x3-Raster + Funktionsleiste darunter.
        px = self.bx + bs + 24
        pb = min((W - px - 16) // 3, self.cell + 8)
        py = self.by + (bs - pb * 3 - 3 * (pb * 2 // 3 + 6)) // 3
        py = max(self.by, py)
        self.pad_rects = {}
        for d in range(1, 10):
            r, c = (d - 1) // 3, (d - 1) % 3
            self.pad_rects[str(d)] = pygame.Rect(px + c * (pb + 4),
                                                 py + r * (pb + 4), pb, pb)
        fy = py + 3 * (pb + 4) + 8
        fw = 3 * pb + 8
        fh = max(24, pb * 2 // 3)
        self.pad_rects["erase"] = pygame.Rect(px, fy, fw, fh)
        row = 1
        if self.can_notes:
            self.pad_rects["note"] = pygame.Rect(px, fy + row * (fh + 6), fw, fh)
            row += 1
        if self.can_hint:
            self.pad_rects["hint"] = pygame.Rect(px, fy + row * (fh + 6), fw, fh)

    def _cell_at(self, pos):
        x, y = pos
        c = (x - self.bx) // self.cell
        r = (y - self.by) // self.cell
        if 0 <= c < 9 and 0 <= r < 9:
            return int(r * 9 + c)
        return None

    def _pad_at(self, pos):
        for key, r in self.pad_rects.items():
            if r.collidepoint(pos):
                return key
        return None

    # ===================================================== Eingabe
    def handle_event(self, event):
        if self.state == SETUP:
            self._handle_setup(event)
            return
        if self.state != PLAY:
            return
        if self.game_over:
            if event.kind == InputEvent.KEYDOWN:
                if event.key in ("Return", "space"):
                    self._next_level() if self.won else self._start_level(self.level)
                elif event.key in ("s", "S", "q", "Q"):
                    self.state = SETUP
                    self.game_over = False
                    self.play_sound("click")
                elif event.key in ("a", "A"):
                    # Banner ausblenden und die Lösung auf dem Brett zeigen
                    # (nochmal A = zurück zum Banner).
                    self.reveal = not self.reveal
                    self.play_sound("select")
            return

        if event.kind == InputEvent.KEYDOWN:
            self._handle_play_key(event.key)
        elif event.kind == InputEvent.MOUSEDOWN:
            if event.button == 3:
                cell = self._cell_at(event.pos)
                if cell is not None:
                    self.sel = cell
                    self._erase(cell)
                return
            cell = self._cell_at(event.pos)
            if cell is not None:
                self.sel = cell
                self.play_sound("move")
                return
            pad = self._pad_at(event.pos)
            if pad == "erase":
                self._erase(self.sel)
            elif pad == "note":
                self._toggle_note_mode()
            elif pad == "hint":
                self._use_hint()
            elif pad is not None:
                self._enter_digit(self.sel, int(pad))

    def _handle_play_key(self, k):
        move = {"Up": -9, "w": -9, "W": -9, "Down": 9, "s": 9, "S": 9,
                "Left": -1, "a": -1, "A": -1, "Right": 1, "d": 1, "D": 1}
        if k in move:
            self.sel = (self.sel + move[k]) % 81
            self.play_sound("move")
        elif k in ("1", "2", "3", "4", "5", "6", "7", "8", "9"):
            self._enter_digit(self.sel, int(k))
        elif k.startswith("KP_") and k[3:] in "123456789":
            self._enter_digit(self.sel, int(k[3:]))
        elif k in ("0", "KP_0", "BackSpace", "Delete"):
            self._erase(self.sel)
        elif k in ("n", "N"):
            self._toggle_note_mode()
        elif k in ("h", "H"):
            self._use_hint()
        elif k in ("r", "R"):
            self._start_level(self.level)
        elif k in ("q", "Q"):
            self.state = SETUP
            self.play_sound("click")

    # ===================================================== Spiellogik
    def _toggle_note_mode(self):
        if not self.can_notes:
            return
        self.note_mode = not self.note_mode
        self.play_sound("select")

    def _enter_digit(self, idx, d):
        if self.locked[idx]:
            return
        if self.note_mode:
            self._toggle_note(idx, d)
            return
        if self.board[idx] == d:
            return
        self.board[idx] = d
        self.notes[idx].clear()
        if d == self.solution[idx]:
            self.wrong.discard(idx)
            if self.can_check:
                self.locked[idx] = True          # korrekt -> rastet ein
            if self.can_notes:
                self._prune_notes(idx, d)
            self.play_sound("select")
            self._update_conflicts()
            self._check_win()
        else:
            self.wrong.add(idx)
            self.errors += 1
            self.play_sound("hit")
            self.rumble(120)
            self._update_conflicts()
            if self.fail_limit and self.errors >= FAIL_LIMIT:
                self._lose()
            else:
                # Auch ein durch eine falsche Ziffer voll gewordenes Brett
                # prüfen -> zeigt in classic/notes die "noch Fehler"-Meldung.
                self._check_win()

    def _toggle_note(self, idx, d):
        if self.board[idx]:
            return
        if d in self.notes[idx]:
            self.notes[idx].discard(d)
        else:
            self.notes[idx].add(d)
        self.play_sound("move")

    def _erase(self, idx):
        if self.locked[idx]:
            return
        if self.board[idx] or self.notes[idx]:
            self.board[idx] = 0
            self.notes[idx].clear()
            self.wrong.discard(idx)
            self._update_conflicts()
            self.play_sound("move")

    def _prune_notes(self, idx, d):
        """Entfernt die Ziffer d aus den Notizen aller Peer-Zellen."""
        for j in sudoku_gen.PEERS[idx]:
            self.notes[j].discard(d)

    def _use_hint(self):
        if not self.can_hint or self.hints_used >= MAX_HINTS or self.game_over:
            return
        idx = self.sel
        if self.locked[idx] or (self.board[idx] and idx not in self.wrong):
            empties = [i for i in range(81)
                       if not self.locked[i] and self.board[i] != self.solution[i]]
            if not empties:
                return
            idx = random.choice(empties)
        self.board[idx] = self.solution[idx]
        self.notes[idx].clear()
        self.wrong.discard(idx)
        self.locked[idx] = True
        self.hints_used += 1
        self._prune_notes(idx, self.solution[idx])
        self.sel = idx
        self.play_sound("powerup")
        self._update_conflicts()
        self._check_win()

    def _update_conflicts(self):
        """Zellen, deren Ziffer mit einem Peer kollidiert (nur comfort+)."""
        if not self.can_check:
            return
        bad = set()
        for i in range(81):
            v = self.board[i]
            if v and any(self.board[j] == v for j in sudoku_gen.PEERS[i]):
                bad.add(i)
        self._conflicts = frozenset(bad)

    def _check_win(self):
        if 0 in self.board:
            return
        if self.board == self.solution:
            self._win()
        elif not self.can_check:
            # Voll, aber falsch: kurzer Hinweis (Fehler wurden schon gezählt).
            self.msg = t("sud.full_wrong")
            self.msg_t = 2.5

    def _win(self):
        self.won = True
        self.game_over = True
        base = DIFFICULTIES[self.diff][1]
        raw = base - TIME_COST * int(self.elapsed) \
            - ERR_COST * self.errors - HINT_COST * self.hints_used
        self.score = int(max(50, raw) * MODE_MULT.get(self.mode, 1.0))
        key = str(self.diff)
        if self.level not in self.solved[key]:
            self.solved[key] = sorted(self.solved[key] + [self.level])
            self._save_progress()
        self.play_sound("win")
        self.rumble(200)

    def _lose(self):
        self.won = False
        self.game_over = True
        self.score = 0
        self.play_sound("gameover")
        self.rumble(250)

    def _next_level(self):
        if self.level >= sudoku_gen.LEVELS:
            self.state = SETUP
            self.game_over = False
            return
        self._start_level(self.level + 1)

    def update(self, dt):
        if self.state == GENERATING:
            # Erst einen Frame "Erzeuge Puzzle..." anzeigen lassen (draw setzt
            # _gen_drawn), dann blockierend generieren.
            if self._gen_drawn:
                self._do_generate()
            return
        if self.state != PLAY or self.game_over:
            return
        self.elapsed += dt
        if self.msg_t > 0:
            self.msg_t -= dt
            if self.msg_t <= 0:
                self.msg = None

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        if self.state == SETUP:
            self._draw_setup(s)
        elif self.state == GENERATING:
            self._draw_generating(s)
        else:
            self._draw_hud(s)
            self._draw_board(s)
            self._draw_pad(s)
            if self.game_over and not self.reveal:
                self._draw_result(s)

    # ----- Setup ----------------------------------------------------------
    def _draw_setup(self, s):
        cx = self.width // 2
        title = self._huge.render("SUDOKU", True, self.accent)
        s.blit(title, title.get_rect(center=(cx, int(self.height * 0.07))))
        mode_lbl = t("sud.mode." + self.mode) if self.mode in MODE_MULT \
            else self.mode
        sub_txt = mode_lbl + "   -   " + t("sud.subtitle")
        sub = self._small.render(sub_txt, True, ui.TEXT_DIM)
        if sub.get_width() > self.width - 24:        # bei 480px Breite kleiner
            sub = self._tiny.render(sub_txt, True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, int(self.height * 0.115))))

        # Stufen-Buttons
        for i, r in enumerate(self.diff_rects):
            on = (i == self.diff)
            pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, r, border_radius=8)
            pygame.draw.rect(s, self.accent if on else ui.BORDER, r,
                             2 if on else 1, border_radius=8)
            lbl = self._small.render(t("sud.diff." + DIFFICULTIES[i][0]), True,
                                     ui.TEXT if on else ui.TEXT_DIM)
            s.blit(lbl, lbl.get_rect(center=r.center))

        # Fehler-Limit-Toggle
        r = self.limit_rect
        pygame.draw.rect(s, ui.BTN, r, border_radius=6)
        pygame.draw.rect(s, ui.BORDER, r, 1, border_radius=6)
        state = t("common.on") if self.fail_limit else t("common.off")
        col = ui.GREEN if self.fail_limit else ui.TEXT_DIM
        lbl = self._small.render(t("sud.fail_limit") + ":  " + state, True, col)
        s.blit(lbl, lbl.get_rect(center=r.center))

        # 10x10-Levelraster
        solved = set(self.solved.get(str(self.diff), []))
        solved_bg = ui.mix(ui.PANEL, ui.GREEN, 0.22)
        for n in range(1, 101):
            i = n - 1
            x = self.lv_x + (i % 10) * self.lv_cell
            y = self.lv_y + (i // 10) * self.lv_cell
            cell = pygame.Rect(x + 1, y + 1, self.lv_cell - 2, self.lv_cell - 2)
            is_solved = n in solved
            pygame.draw.rect(s, solved_bg if is_solved else ui.BTN, cell,
                             border_radius=4)
            if n == self.cursor:
                pygame.draw.rect(s, self.accent, cell, 2, border_radius=4)
            num = self._lv_font.render(str(n), True,
                                       ui.GREEN if is_solved else ui.TEXT_DIM)
            s.blit(num, num.get_rect(center=cell.center))
            if is_solved:
                # kleiner Haken unten rechts
                bx, by = cell.right - 7, cell.bottom - 6
                pygame.draw.lines(s, ui.GREEN, False,
                                  [(bx - 4, by - 2), (bx - 2, by), (bx + 2, by - 5)], 2)

        prog = self._small.render(t("sud.progress", n=len(solved)), True, ui.TEXT_DIM)
        s.blit(prog, prog.get_rect(center=(self.width // 2,
                                           self.lv_y + 10 * self.lv_cell + 18)))
        hint = self._tiny.render(t("sud.setup_hint"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(self.width // 2, self.height - 16)))

    def _draw_generating(self, s):
        lbl = self.font.render(t("sud.generating"), True, ui.TEXT)
        s.blit(lbl, lbl.get_rect(center=(self.width // 2, self.height // 2)))
        self._gen_drawn = True

    # ----- HUD ------------------------------------------------------------
    def _fmt_time(self):
        sec = int(self.elapsed)
        return f"{sec // 60:02d}:{sec % 60:02d}"

    def _draw_hud(self, s):
        pygame.draw.rect(s, ui.PANEL, (0, 0, self.width, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h), (self.width, self.hud_h))
        cy = self.hud_h // 2

        left = t("sud.level", n=self.level) + "  ·  " \
            + t("sud.diff." + DIFFICULTIES[self.diff][0])
        img = self._small.render(left, True, ui.TEXT)
        s.blit(img, img.get_rect(midleft=(12, cy)))

        # In der Lösungs-Ansicht ersetzt der Zurück-Hinweis die (eingefrorene)
        # Uhr - so bleibt das komplette Brett frei sichtbar.
        if self.game_over and self.reveal:
            clock = self._small.render(t("sud.hide_solution"), True, self.accent)
        else:
            clock = self._clock_font.render(self._fmt_time(), True, self.accent)
        s.blit(clock, clock.get_rect(center=(self.width // 2, cy)))

        if self.fail_limit:
            err_txt = t("sud.errors", n=self.errors, m=FAIL_LIMIT)
        else:
            err_txt = t("sud.errors_free", n=self.errors)
        col = ui.RED if self.errors else ui.TEXT_DIM
        right = [(err_txt, col)]
        if self.can_hint:
            right.append((t("sud.hints", n=MAX_HINTS - self.hints_used),
                          ui.TEXT_DIM))
        if self.note_mode:
            right.append((t("sud.pad_note").upper(), self.accent))
        x = self.width - 12
        for txt, c in right:
            img = self._small.render(txt, True, c)
            s.blit(img, img.get_rect(midright=(x, cy)))
            x -= img.get_width() + 16

        if self.msg:
            img = self._small.render(self.msg, True, ui.RED)
            s.blit(img, img.get_rect(center=(self.width // 2,
                                             self.hud_h + 14)))

    # ----- Brett ------------------------------------------------------------
    def _draw_board(self, s):
        bs = self.cell * 9
        sel = self.sel
        sel_val = self.board[sel]
        sel_r, sel_c = sel // 9, sel % 9
        sel_b = sudoku_gen.BOX_OF[sel]

        # Zellfarben je Frame aus Palette + Akzent mischen (Theme-fähig).
        c_cell = ui.PANEL
        c_peer = ui.PANEL_LIGHT
        c_sel = ui.mix(ui.PANEL_LIGHT, self.accent, 0.35)
        c_same = ui.mix(ui.PANEL_LIGHT, self.accent, 0.18)
        c_bad = ui.mix(ui.PANEL, ui.RED, 0.35)

        for i in range(81):
            r, c = i // 9, i % 9
            x = self.bx + c * self.cell
            y = self.by + r * self.cell
            rect = pygame.Rect(x, y, self.cell, self.cell)

            # Zellhintergrund: Auswahl > Konflikt > gleiche Ziffer > Peers
            if i == sel:
                bg = c_sel
            elif self.can_check and i in self._conflicts:
                bg = c_bad
            elif self.can_check and sel_val and self.board[i] == sel_val:
                bg = c_same
            elif r == sel_r or c == sel_c or sudoku_gen.BOX_OF[i] == sel_b:
                bg = c_peer
            else:
                bg = c_cell
            pygame.draw.rect(s, bg, rect)

            # Lösungs-Ansicht (A nach Spielende): fehlende/falsche Zellen
            # zeigen die richtige Ziffer in Akzentfarbe.
            if self.game_over and self.reveal \
                    and self.board[i] != self.solution[i]:
                img = self._num_font.render(str(self.solution[i]), True,
                                            self.accent)
                s.blit(img, img.get_rect(center=rect.center))
                continue

            v = self.board[i]
            if v:
                if self.given[i]:
                    col = ui.TEXT
                elif self.can_check and i in self.wrong:
                    col = ui.RED
                elif self.locked[i]:
                    col = ui.GREEN if not self.given[i] else ui.TEXT
                else:
                    col = COL_USER
                img = self._num_font.render(str(v), True, col)
                s.blit(img, img.get_rect(center=rect.center))
            elif self.notes[i]:
                third = self.cell // 3
                for d in self.notes[i]:
                    nx = x + ((d - 1) % 3) * third + third // 2
                    ny = y + ((d - 1) // 3) * third + third // 2
                    img = self._note_font.render(str(d), True, ui.TEXT_FAINT)
                    s.blit(img, img.get_rect(center=(nx, ny)))

        # Gitterlinien (dünn + 3x3 fett)
        for k in range(10):
            w = 2 if k % 3 == 0 else 1
            col = ui.BORDER_LIGHT if k % 3 == 0 else ui.BORDER
            x = self.bx + k * self.cell
            y = self.by + k * self.cell
            pygame.draw.line(s, col, (x, self.by), (x, self.by + bs), w)
            pygame.draw.line(s, col, (self.bx, y), (self.bx + bs, y), w)

    # ----- Ziffernfeld ------------------------------------------------------
    def _draw_pad(self, s):
        # Verbleibende Anzahl je Ziffer (comfort/assist zeigen sie an).
        counts = [0] * 10
        for v in self.board:
            counts[v] += 1

        for d in range(1, 10):
            r = self.pad_rects[str(d)]
            done = counts[d] >= 9
            pygame.draw.rect(s, ui.BTN, r, border_radius=6)
            pygame.draw.rect(s, ui.BORDER, r, 1, border_radius=6)
            col = ui.TEXT_DIM if done else ui.TEXT
            img = self._num_font.render(str(d), True, col)
            s.blit(img, img.get_rect(center=(r.centerx,
                                             r.centery - (5 if self.can_check else 0))))
            if self.can_check and not done:
                rem = self._tiny.render(str(9 - counts[d]), True, ui.TEXT_DIM)
                s.blit(rem, rem.get_rect(center=(r.centerx, r.bottom - 9)))

        r = self.pad_rects["erase"]
        pygame.draw.rect(s, ui.BTN, r, border_radius=6)
        pygame.draw.rect(s, ui.BORDER, r, 1, border_radius=6)
        # Lokalisiert; bewusst ohne "⌫"-Symbol (nicht in jedem Font enthalten).
        img = self._tiny.render(t("sud.pad_erase"), True, ui.TEXT)
        s.blit(img, img.get_rect(center=r.center))

        if "note" in self.pad_rects:
            r = self.pad_rects["note"]
            on = self.note_mode
            pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, r, border_radius=6)
            pygame.draw.rect(s, self.accent if on else ui.BORDER, r,
                             2 if on else 1, border_radius=6)
            img = self._tiny.render(t("sud.pad_note") + " (N)", True,
                                    ui.TEXT if on else ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=r.center))

        if "hint" in self.pad_rects:
            r = self.pad_rects["hint"]
            left = MAX_HINTS - self.hints_used
            pygame.draw.rect(s, ui.BTN, r, border_radius=6)
            pygame.draw.rect(s, ui.BORDER, r, 1, border_radius=6)
            img = self._tiny.render(f"{t('sud.pad_hint')} (H) x{left}", True,
                                    ui.GREEN if left else ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=r.center))

        hint = self._tiny.render(t("sud.hint"), True, ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(self.width // 2, self.height - 12)))

    # ----- Ergebnis-Overlay ---------------------------------------------------
    def _draw_result(self, s):
        # Abdunkelung wird gecacht (kein Alpha-Vollbild-Fill pro Frame).
        if self._overlay is None \
                or self._overlay.get_size() != (self.width, self.height):
            ov = pygame.Surface((self.width, self.height), pygame.SRCALPHA)
            ov.fill((8, 10, 16, 185))
            self._overlay = ov
        s.blit(self._overlay, (0, 0))
        cx, cy = self.width // 2, self.height // 2

        if self.won:
            head = self._huge.render(t("sud.win", t=self._fmt_time()), True,
                                     ui.GREEN)
            lines = [(t("common.points", score=self.score), ui.TEXT),
                     (t("sud.next"), ui.TEXT_DIM),
                     (t("sud.show_solution"), ui.TEXT_DIM)]
        else:
            head = self._huge.render(t("sud.lose"), True, ui.RED)
            lines = [(t("sud.retry"), ui.TEXT_DIM),
                     (t("sud.show_solution"), ui.TEXT_DIM)]

        # Panel hinter dem Ergebnis (Akzent-Rahmen, Breite folgt dem Inhalt).
        pw = max(min(self.width - 40, 460), head.get_width() + 40)
        panel = pygame.Rect(cx - pw // 2, cy - 92, pw, 184)
        pygame.draw.rect(s, ui.PANEL, panel, border_radius=14)
        pygame.draw.rect(s, self.accent, panel, 2, border_radius=14)

        s.blit(head, head.get_rect(center=(cx, cy - 50)))
        y = cy + 4
        for txt, col in lines:
            img = self._small.render(txt, True, col)
            s.blit(img, img.get_rect(center=(cx, y)))
            y += 30
