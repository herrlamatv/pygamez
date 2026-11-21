# -*- coding: utf-8 -*-
"""
lamamines.py
============
Lama Mines - Edelstein-Suche mit Lama-Chips: Auf einem Feld aus 4x4 bis 7x7
Steinquadern liegen 1 bis N-1 Minen, unter allen anderen ein Edelstein. Jeder
gefundene Edelstein hebt den Multiplikator, eine Mine kostet den Einsatz.
Auszahlen geht ab dem ersten Edelstein; wer alle Edelsteine findet oder den
Höchstgewinn x100.000 erreicht, wird automatisch ausgezahlt.

Regeln und Zahlen stehen in ``lamamines_logic.py`` (ohne pygame, bitgleich in
Web und Rust), die Bilder in ``lamamines_draw.py``.

- **Fairness** (Fenster F): Vor jeder Runde steht der Prüfcode (SHA-256 des
  Server-Seeds) fest, das Minenfeld entsteht aus Server-Seed, Client-Seed und
  Rundennummer (``fairness.py``). Nach der Runde wird der Seed gezeigt; die
  letzten 12 Runden lassen sich nachrechnen (mit Mini-Vorschau des Feldes).
- **Statistik** (Fenster S): Kacheln + Bilanzkurve (``chipstats.py``).
- **Auto-Modus** (Fenster A): Felder auf dem Raster markieren (gemerkt je
  Feldgröße); jede Runde deckt genau diese auf und zahlt aus, wenn alle sicher
  sind. Runden, Stopp bei Gewinn/Verlust ab, Turbo (T).
- Verlauf der letzten 12 Runden oben rechts, Maskottchen-Lama mit Grubenhelm
  und Lore, Explosion mit Kamera-Wackeln, danach wird das ganze Feld
  abgedunkelt aufgedeckt (auch nach dem Auszahlen).

Lama-Chips: gemeinsames Konto der Lama-Bank (``lamabank.py``). Der Einsatz wird
bei SPIELEN abgebucht; die Runde wird bei jedem aufgedeckten Feld gespeichert
(Seeds, Rundennummer, Feld, Minen, Einsatz, aufgedeckte Plätze) - wer mitten
in der Runde geht, macht beim nächsten Start genau dort weiter (das Feld wird
aus dem Seed neu berechnet). Highscore = Höchststand von 1000 +
Lama-Mines-Bilanz; ``game_over`` wird nie gesetzt.

Steuerung (alle Tasten roh, nie über die WASD-Belegung): Leertaste/Enter =
SPIELEN, 1-4 = Feldgröße, ←/→ = Minen, ↑/↓/+/-/Mausrad = Einsatz; in der
Runde Klick oder Pfeile + Leertaste = aufdecken, R = Zufallsfeld, Enter/C =
auszahlen. A = Auto, S = Statistik, F = Fairness, T = Turbo, Esc schließt ein
Fenster bzw. stoppt Auto.
"""

import math
import random

import pygame

import audio
import charts
import chipstats
import fairness
import lamabank
import settings as settings_mod
import ui
from game_base import Game, InputEvent
from i18n import get_language, t

from . import casino_draw as CD
from . import lamamines_draw as D
from . import lamamines_logic as ML

KEY = ML.KEY
IDLE, PLAY, BOOM, CASHED = "idle", "play", "boom", "cashed"

FLIP_T = 0.24          # Aufklappen eines Feldes
POP_T = 0.32           # Edelstein federt nach dem Aufklappen
WAVE_T = 0.05          # Rest-Aufdeckung: Verzögerung je Feld Abstand
BOOM_T = 0.62          # Explosion -> Ergebnis-Tafel
RESULT_T = 2.6         # Ergebnis-Tafel, danach Ruhezustand (Feld bleibt aufgedeckt)
READ_T = 0.35          # Ergebnis frühestens danach wegdrückbar
SHAKE_T = 0.55         # Kamera-Wackeln nach einer Mine
AUTO_STEP_T = 0.26     # Auto: Abstand zwischen zwei Feldern
AUTO_RESULT_T = 1.0    # Auto: Ergebnis-Anzeige
AUTO_PAUSE_T = 0.35    # Auto: Pause vor der nächsten Runde
TURBO_F = 0.4          # Turbo: alle Zeiten x 0,4
BIG_MULT = 300         # ab x3: Münzregen
PARTY_MULT = 1000      # ab x10: Konfetti

ROUND_OPTS = (5, 10, 25, 50, 100)
LIMIT_OPTS = (0, 100, 500, 1000, 5000, 10000, 50000)
TOOLS = ("auto", "stats", "fair", "turbo")
TOOL_LABEL = {"auto": "chip.auto", "stats": "chip.stats", "fair": "chip.fair", "turbo": "chip.turbo"}
TOOL_KEY = {"auto": "A", "stats": "S", "fair": "F", "turbo": "T"}
# Töne beim Finden: Dur-Pentatonik aufwärts (Halbtöne über dem Grundton)
GEM_STEPS = (0, 2, 4, 7, 9, 12, 14, 16, 19, 21, 24, 26, 28, 31, 33, 36)


class LamaMinesGame(Game):
    name = "Lama Mines"
    highscore_key = KEY
    supports_multiplayer = False

    @property
    def wants_escape(self):
        # Esc schließt ein offenes Fenster bzw. stoppt den Auto-Modus
        return self.window is not None or self.auto is not None

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.game_over = False
        lamabank.load()
        self.rng = random.Random()
        grid = self._opt("grid", ML.DEFAULT_GRID)
        self.grid = grid if grid in ML.GRIDS else ML.DEFAULT_GRID
        self.mines = ML.clamp_mines(self.grid, self._int_opt("mines", ML.DEFAULT_MINES))
        bet = self._opt("bet", ML.BETS[0])
        self.bet = bet if bet in ML.BETS else ML.BETS[0]
        self.turbo = self._opt("turbo", False) is True
        v = self._opt("auto_rounds", 10)
        self.auto_rounds = v if v in ROUND_OPTS else 10
        v = self._opt("auto_win", 0)
        self.auto_win = v if v in LIMIT_OPTS else 0
        v = self._opt("auto_loss", 0)
        self.auto_loss = v if v in LIMIT_OPTS else 0
        extra = lamabank.get_extra(KEY, {})
        extra = extra if isinstance(extra, dict) else {}
        self.hist = ML.clean_history(extra.get("hist"))
        self.fair = fairness.clean_state(extra.get("fair"), self.rng)
        self.fairlog = fairness.clean_log(extra.get("fairlog"))
        self.stats = chipstats.clean(extra.get("stats"))
        self.marks = ML.clean_marks(extra.get("auto_marks"))
        self.carry = ML.clean_carry(extra.get("carry"))     # Rest-Hundertstel der Auszahlungen
        self.round = ML.clean_round(extra.get("round"))
        # Spielfeld (laufende bzw. letzte Runde)
        self.field = []               # Minenplätze (sortiert)
        self.is_mine = []             # je Platz True/False
        self.cells = []               # je Platz: Anzeige-Zustand (siehe _blank_cells)
        self.hit = None               # Platz der getroffenen Mine
        self.state = IDLE
        self.result = None            # (Titel, Untertitel, Farbe)
        self.res_t = 0.0
        self.res_delay = 0.0          # Ergebnis-Tafel erscheint erst danach
        self.boom_t = None            # Zeit seit der Explosion (None = keine)
        self.win = 0
        self.lost = 0
        self.last_mult = 0
        self.pending = 0              # gutgeschrieben, aber noch nicht gezeigt
        self.shown_chips = float(lamabank.balance())
        self.particles = []
        self.coins_fx = []
        self.popups = []
        self.flashes = []
        self.toast = None
        self.banner = None            # [Text, Restzeit] (Höchstgewinn)
        self.shake = 0.0
        self.time = 0.0
        self.mouse = (-1, -1)
        self.broke = False
        self.window = None            # None / "auto" / "stats" / "fair"
        self.auto = None              # laufender Auto-Modus (dict)
        self.cursor = 0
        self.kb = False               # Tastatur-Cursor sichtbar?
        self.mood = ["idle", 0.0]     # Maskottchen: idle/happy/cheer/scared + Zeit
        self.sooty = False
        self.cart_fill = 0
        self.fair_sel = None          # gewählter Log-Eintrag (Index)
        self.fair_scroll = 0
        self.fair_checks = {}         # Nonce -> (stimmt?, Minenplätze)
        self.r_fair_rows = []
        self.r_fair_new = pygame.Rect(0, 0, 0, 0)
        self.r_fair_verify = pygame.Rect(0, 0, 0, 0)
        self.cash_info = dict(clear=False, maxed=False, m100=0, shown=True)
        self._tick_cd = 0.0
        self._lang = get_language()
        self._layout()
        if self.round is not None:
            # Unterbrochene Runde genau dort fortsetzen (Feld aus dem Seed)
            self.grid = self.round["grid"]
            self.mines = self.round["mines"]
            self._new_board(self.round)
            for j, i in enumerate(self.round["open"]):
                c = self.cells[i]
                c["face"], c["flip"], c["pop"], c["k"] = "gem", 1.0, POP_T, j + 1
            self.cart_fill = self._cart_level()
            self.state = PLAY
            self._layout()
            self._say(t("lmi.resumed"))
        else:
            self._blank_board()
            self._clamp_bet()
        self.cursor = (self.grid * self.grid) // 2
        self.score = lamabank.score_for(KEY)
        self._check_broke()

    def on_surface_changed(self):
        D.clear_cache()
        CD.clear_cache()
        self._layout()

    # ===================================================== Layout
    def _layout(self):
        W, H = self.width, self.height
        self.k = k = min(W / 800.0, H / 600.0)
        self.f_micro = ui.font(max(10, int(11 * k)))
        self.f_tiny = ui.font(max(11, int(13 * k)))
        self.f_small = ui.font(max(12, int(15 * k)))
        self.f_btn = ui.font(max(12, int(15 * k)), bold=True)
        self.f_big = ui.font(max(15, int(22 * k)), bold=True)
        self.f_mult = ui.font(max(18, int(32 * k)), bold=True)
        self.f_huge = ui.font(max(24, int(48 * k)), bold=True)
        self.f_mono = ui.font(max(10, int(12 * k)), mono=True)
        self.margin = m = max(6, int(12 * k))
        self.hud_h = max(28, int(42 * k))
        self.strip_h = max(46, int(64 * k))
        self.tool_h = max(26, int(34 * k))
        self.strip = pygame.Rect(0, H - self.strip_h, W, self.strip_h)
        tool_y = self.hud_h + m
        area_y = tool_y + self.tool_h + m
        area_h = self.strip.y - m - area_y
        side = min(area_h, int(W * 0.58))
        self.board_r = pygame.Rect(m, area_y + (area_h - side) // 2, side, side)
        self.panel_r = pygame.Rect(self.board_r.right + m, area_y, W - self.board_r.right - 2 * m, area_h)
        self._hud_bg = pygame.Surface((W, self.hud_h), pygame.SRCALPHA)
        for y in range(self.hud_h):
            a = int(210 - 60 * y / max(1, self.hud_h - 1))
            self._hud_bg.fill((10, 8, 16, a), (0, y, W, 1))
        self._strip_bg = pygame.Surface(self.strip.size, pygame.SRCALPHA)
        for y in range(self.strip_h):
            a = int(170 + 60 * y / max(1, self.strip_h - 1))
            self._strip_bg.fill((10, 8, 16, a), (0, y, W, 1))
        self._overlay = pygame.Surface((W, H), pygame.SRCALPHA)
        self._overlay.fill((6, 4, 12, 200))
        self._layout_board()
        self._layout_tools(tool_y)
        self._layout_strip()
        self._layout_panel()

    def _layout_board(self):
        """Felder im Holzrahmen: Rand, Abstand, Feldgröße und Rechtecke."""
        side = self.board_r.w
        n = self.grid
        self.board_pad = pad = max(6, int(side * 0.035))
        inner = side - 2 * pad
        gap = max(2, int(inner * 0.014))
        cell = (inner - gap * (n + 1)) // n
        total = n * cell + (n + 1) * gap
        off = (inner - total) // 2
        x0 = self.board_r.x + pad + off + gap
        y0 = self.board_r.y + pad + off + gap
        self.cell = cell
        self.tile_r = [pygame.Rect(x0 + (i % n) * (cell + gap), y0 + (i // n) * (cell + gap), cell, cell)
                       for i in range(n * n)]

    def _layout_tools(self, y):
        """Werkzeug-Knöpfe oben links (mit Beschriftung, wenn Platz ist)."""
        m = self.margin
        h = self.tool_h
        pad = max(6, int(10 * self.k))
        widths = [h + pad + self.f_tiny.size(t(TOOL_LABEL[name]))[0] for name in TOOLS]
        total = 0
        for w in widths:
            total += w + max(3, int(5 * self.k))
        self.tool_labels = total <= self.board_r.w
        x = m
        self.r_tools = []
        for i, name in enumerate(TOOLS):
            w = widths[i] if self.tool_labels else h
            self.r_tools.append((name, pygame.Rect(x, y, w, h)))
            x += w + max(3, int(5 * self.k))
        self.tools_right = x

    def _layout_strip(self):
        """Bedienleiste: Einsatz links, Zufallsfeld Mitte, AUSZAHLEN + SPIELEN rechts."""
        m, k, W = self.margin, self.k, self.width
        sh = self.strip_h
        bh = max(30, int(sh * 0.62))
        by = self.strip.y + (sh - bh) // 2
        lbl_w = max(self.f_tiny.size(t("cas.bet"))[0], self.f_btn.size("10000")[0]) + int(14 * k)
        self.r_minus = pygame.Rect(m, by, bh, bh)
        self.r_bet = pygame.Rect(self.r_minus.right + 4, by, lbl_w, bh)
        self.r_plus = pygame.Rect(self.r_bet.right + 4, by, bh, bh)
        pad = int(22 * max(0.7, k))
        go_w = max(self.f_btn.size(t("lmi.play"))[0], self.f_btn.size(t("chip.auto_stop"))[0]) + pad
        go_w = max(go_w, int(110 * k))
        cash_w = self.f_btn.size(t("lmi.cashout") + " 99.999.999")[0] + pad
        room = W - m - (self.r_plus.right + m) - max(4, int(8 * k)) - bh - 2 * m
        if go_w + cash_w > room:
            f = room / (go_w + cash_w)
            go_w, cash_w = int(go_w * f), int(cash_w * f)
        self.r_go = pygame.Rect(W - m - go_w, by, go_w, bh)
        self.r_cash = pygame.Rect(self.r_go.x - max(4, int(8 * k)) - cash_w, by, cash_w, bh)
        # Zufallsfeld: so breit wie möglich zwischen Einsatz und AUSZAHLEN
        left, right = self.r_plus.right + m, self.r_cash.x - m
        want = bh + self.f_btn.size(t("lmi.random"))[0] + pad
        w = max(bh, min(want, right - left))
        self.random_label = w >= want
        self.r_random = pygame.Rect(0, by, w, bh)
        self.r_random.centerx = (left + right) // 2

    def _layout_panel(self):
        """Seitenleiste: Kopf (Maskottchen, Lore, Multiplikator), Einstellungen
        bzw. Gewinn-Info, Leiste der nächsten Stufen, Tastenhinweis."""
        p = self.panel_r
        k = self.k
        self.ppad = pad = max(6, int(10 * k))
        x0, w = p.x + pad, p.w - 2 * pad
        gap = max(4, int(7 * k))
        ms = int(min(w * 0.30, 104 * k, p.h * 0.30))
        self.r_mascot = pygame.Rect(x0, p.y + pad, ms, ms)
        self.cart_w = int(ms * 0.62)
        self.head_x = x0 + ms + int(self.cart_w * 0.95) + gap
        y = self.r_mascot.bottom + gap
        self.row_h = rh = max(22, int(30 * k))
        lab = self.f_tiny.get_height()
        # Einstellungen (nur im Ruhezustand)
        self.lab_grid_y = y
        y += lab + 2
        bw = (w - 3 * 4) // 4
        self.r_grids = [pygame.Rect(x0 + i * (bw + 4), y, bw, rh) for i in range(4)]
        y += rh + gap
        self.mines_row_y = y
        vw = max(self.f_btn.size("48")[0] + int(16 * k), rh + 6)
        self.r_mval = pygame.Rect(x0 + w - rh - 4 - vw, y, vw, rh)
        self.r_mminus = pygame.Rect(self.r_mval.x - 4 - rh, y, rh, rh)
        self.r_mplus = pygame.Rect(self.r_mval.right + 4, y, rh, rh)
        y += rh + 4
        npre = len(ML.MINE_PRESETS[4])
        pw = (w - (npre - 1) * 4) // npre
        self.r_presets = [pygame.Rect(x0 + i * (pw + 4), y, pw, max(18, int(rh * 0.82)))
                          for i in range(npre)]
        self.info_y = self.lab_grid_y
        # Leiste der nächsten Stufen + Hinweis am unteren Rand
        hint_h = 2 * self.f_micro.get_height() + 2
        pill_h = max(20, int(28 * k))
        self.ladder_y = p.bottom - pad - hint_h - gap - pill_h
        self.ladder_lab_y = self.ladder_y - lab - 2
        self.r_pills = [pygame.Rect(x0 + i * ((w - 16) // 5 + 4), self.ladder_y, (w - 16) // 5, pill_h)
                        for i in range(5)]
        self.hint_y = p.bottom - pad - hint_h
        self.panel_inner = (x0, w)
        # Auto-Fenster liegt genau über der Seitenleiste
        self._layout_auto()

    def _layout_auto(self):
        p = self.panel_r
        k = self.k
        pad = self.ppad
        x0, w = p.x + pad, p.w - 2 * pad
        gap = max(3, int(6 * k))
        lab = self.f_tiny.get_height()
        rh = max(20, int(28 * k))
        cb = max(18, int(24 * k))
        self.r_auto_close = pygame.Rect(p.right - pad - cb, p.y + pad, cb, cb)
        y = p.y + pad + max(cb, self.f_btn.get_height()) + gap
        rows = []
        for opts in (ROUND_OPTS, LIMIT_OPTS, LIMIT_OPTS):
            ly = y
            y += lab + 2
            n = len(opts)
            bw = (w - (n - 1) * 3) // n
            first = bw
            if opts[0] == 0:
                # "aus" ist je Sprache verschieden lang - der erste Knopf wächst mit
                first = max(bw, min(self.f_tiny.size(t("chip.auto_off"))[0],
                                    self.f_micro.size(t("chip.auto_off"))[0] + 4) + 8)
                bw = (w - first - (n - 1) * 3) // (n - 1)
            rects, x = [], x0
            for i in range(n):
                rw = first if i == 0 else bw
                rects.append(pygame.Rect(x, y, rw, rh))
                x += rw + 3
            rows.append((ly, rects))
            y += rh + gap
        self.auto_rows = rows
        self.auto_info_y = y
        bh = max(24, int(32 * k))
        self.r_auto_start = pygame.Rect(0, p.bottom - pad - bh, 0, bh)
        cw = max(self.f_btn.size(t("lmi.clear_marks"))[0] + int(18 * k), bh)
        self.r_auto_clear = pygame.Rect(x0, self.r_auto_start.y, cw, bh)
        self.r_auto_start.x = self.r_auto_clear.right + gap
        self.r_auto_start.w = x0 + w - self.r_auto_start.x

    # ===================================================== Einstellungen
    def _opt(self, key, default):
        sec = self.settings.get(KEY) if isinstance(self.settings, dict) else None
        if isinstance(sec, dict) and key in sec:
            return sec[key]
        return default

    def _int_opt(self, key, default):
        v = self._opt(key, default)
        return v if isinstance(v, int) and not isinstance(v, bool) else default

    def _set_opt(self, key, value):
        if not isinstance(self.settings, dict):
            return
        sec = self.settings.setdefault(KEY, {})
        if sec.get(key) != value:
            sec[key] = value
            settings_mod.save_settings(self.settings)

    @staticmethod
    def _is_action(key):
        """Leertaste/Enter. Alle Tasten werden ROH ausgewertet (wie Lama Crossing
        und Lama-Slot) - nie über is_action/WASD, sonst hätten A und S mit der
        Standardbelegung eine Doppelbedeutung."""
        return key in ("space", "Return", "KP_Enter")

    # ===================================================== Konto / Speichern
    def _extra(self):
        return {"round": self.round, "hist": self.hist, "fair": self.fair, "fairlog": self.fairlog,
                "stats": self.stats, "auto_marks": self.marks, "carry": self.carry}

    def _win_at(self, bet, k):
        """Auszahlung nach k Edelsteinen - mit dem gespeicherten Übertrag, also genau
        das, was AUSZAHLEN jetzt gutschreiben würde."""
        return ML.payout_carry(bet, self.grid, self.mines, k, self.carry)[0]

    def _save_extra(self):
        lamabank.set_extra(KEY, self._extra())

    def _clamp_bet(self):
        """Einsatz auf die höchste bezahlbare Stufe senken (mind. Stufe 1)."""
        chips = lamabank.balance()
        if self.bet > chips:
            ok = [b for b in ML.BETS if b <= chips]
            self.bet = ok[-1] if ok else ML.BETS[0]

    def _check_broke(self):
        self.broke = self.state == IDLE and self.auto is None and lamabank.is_broke(KEY)
        if self.broke:
            self.window = None
            self.play_sound("gameover")

    def _take_credit(self):
        if lamabank.refill_if_broke(KEY):
            self.play_sound("powerup")
            ui.spawn_burst(self.width // 2, self.height // 2, ui.GOLD, n=26)
        self.broke = False
        self.shown_chips = float(lamabank.balance())

    def _round_bet(self):
        return self.round["bet"] if self.round is not None else self.bet

    def _say(self, text, secs=1.8):
        self.toast = [text, secs]

    def _tf(self):
        """Zeitfaktor: Turbo macht alles schneller."""
        return TURBO_F if self.turbo else 1.0

    # ===================================================== Formatierung
    def _sep(self):
        return t("lmi.decimal")

    def _group(self, n):
        """Ganze Zahl mit Tausenderpunkten (bzw. -kommas, je Sprache)."""
        sep = "." if self._sep() == "," else ","
        s = str(abs(int(n)))
        out = ""
        while len(s) > 3:
            out = sep + s[-3:] + out
            s = s[:-3]
        return ("-" if n < 0 else "") + s + out

    def _fmt(self, m100):
        """Multiplikator x100 als Text mit Tausendertrennung: ×1.234,56."""
        if m100 >= 1000000:
            return "×" + self._group(m100 // 100)
        return "×" + self._group(m100 // 100) + self._sep() + "%02d" % (m100 % 100)

    def _fmt_short(self, m100):
        """Kurzform für die Stufen-Pillen: ×1,23 · ×123,4 · ×1234 · ×12,3k · ×100k."""
        sep = self._sep()
        if m100 < 10000:
            return "×%d%s%02d" % (m100 // 100, sep, m100 % 100)
        if m100 < 100000:
            return "×%d%s%d" % (m100 // 100, sep, (m100 // 10) % 10)
        if m100 < 1000000:
            return "×%d" % (m100 // 100)
        if m100 < 10000000:
            return "×%d%s%dk" % (m100 // 100000, sep, (m100 // 10000) % 10)
        return "×%dk" % (m100 // 100000)

    def _fmt_pct(self, p10):
        return ML.fmt_percent(p10, self._sep())

    # ===================================================== Spielfeld
    def _blank_cells(self):
        n = self.grid * self.grid
        # face: "hidden", "gem", "mine", "boom"; flip: 0..1 (1 = fertig);
        # delay: Wartezeit vor dem Aufklappen; dim: nach der Runde aufgedeckt
        # k: Edelstein Nummer k dieser Runde (für Ton und Popup beim Aufklappen)
        self.cells = [dict(face="hidden", flip=1.0, delay=0.0, dim=False, pop=POP_T, mark=0.0, k=0)
                      for _ in range(n)]

    def _blank_board(self):
        """Leeres, verdecktes Feld der aktuellen Größe (Ruhezustand)."""
        self.field = []
        self.is_mine = [False] * (self.grid * self.grid)
        self.hit = None
        self._blank_cells()

    def _new_board(self, rnd):
        """Minenfeld einer Runde aus ihren Fairness-Werten berechnen."""
        self.field = ML.field_from_seed(rnd["grid"], rnd["mines"], rnd["s"], rnd["c"], rnd["n"])
        n = rnd["grid"] * rnd["grid"]
        self.is_mine = [False] * n
        for i in self.field:
            self.is_mine[i] = True
        self.hit = None
        self._blank_cells()

    def _opened(self):
        return self.round["open"] if self.round is not None else []

    def _k(self):
        return len(self._opened())

    def _cart_level(self):
        """Edelsteine in der Lore (0..7) - wächst mit dem Anteil gefundener Steine."""
        g = ML.gems(self.grid, self.mines)
        k = self._k()
        if k <= 0:
            return 0
        return max(1, min(7, (k * 7 + g - 1) // g))

    def _entry(self):
        r = self.round
        return {"n": r["n"], "s": r["s"], "c": r["c"], "h": r["h"]}

    # ===================================================== Ablauf
    def _play(self):
        """SPIELEN: Einsatz abbuchen, Minenfeld aus dem Fairness-Zufall ziehen."""
        if self.state != IDLE or self.broke or self.round is not None:
            return False
        bet = self.bet
        if bet > lamabank.balance():
            self._say(t("cas.not_enough"))
            self.play_sound("hit")
            return False
        entry = fairness.begin(self.fair)
        fairness.advance(self.fair, self.rng)
        self.round = {"bet": bet, "grid": self.grid, "mines": self.mines, "n": entry["n"],
                      "s": entry["s"], "c": entry["c"], "h": entry["h"], "open": []}
        if not lamabank.bet_and_pay(bet, 0, KEY, KEY, self._extra()):
            self.round = None
            return False
        self.field = ML.make_field(fairness.entry_rand(KEY, entry), self.grid, self.mines)
        self.is_mine = [False] * (self.grid * self.grid)
        for i in self.field:
            self.is_mine[i] = True
        self.hit = None
        self._blank_cells()
        for c in self.cells:
            c["flip"] = 0.0              # Felder "wachsen" kurz ein
            c["face"] = "hidden"
        for i, c in enumerate(self.cells):
            c["delay"] = 0.012 * ((i % self.grid) + (i // self.grid)) * self._tf()
        self.state = PLAY
        self.result = None
        self.sooty = False
        self.cart_fill = 0
        self.mood = ["idle", 0.0]
        self.win = 0
        self.lost = 0
        self.play_sound("select")
        self.score = lamabank.score_for(KEY)
        return True

    def _reveal(self, i):
        """Ein Feld aufdecken (Klick, Leertaste, R oder Auto)."""
        if self.state != PLAY or self.round is None or not 0 <= i < len(self.cells):
            return
        if i in self.round["open"] or self.cells[i]["face"] != "hidden":
            return
        c = self.cells[i]
        c["flip"], c["delay"], c["dim"] = 0.0, 0.0, False
        if self.is_mine[i]:
            # Mine: sofort verbuchen (Verlassen hilft nicht), Explosion nach dem Aufklappen
            c["face"] = "boom"
            self.hit = i
            bet = self.round["bet"]
            opened = self.round["open"] + [i]
            self.lost = bet
            self.last_mult = 0
            self._finish(0, opened)
            self.state = BOOM
            self.boom_t = None
            self.res_delay = FLIP_T * self._tf() + BOOM_T * self._tf()
            self.res_t = 0.0
            self.result = (t("lmi.boom"), t("lmi.lost", n=self._group(bet)), ui.RED)
            self.stats["x"]["booms"] = self._xint("booms") + 1
            self._save_extra()
            self.play_sound("move")
            return
        c["face"] = "gem"
        self.round["open"].append(i)
        k = len(self.round["open"])
        c["k"] = k
        self.stats["x"]["gems"] = self._xint("gems") + 1
        self.play_sound("move")
        if ML.is_clear(self.grid, self.mines, k) or ML.is_max(self.grid, self.mines, k):
            self._cash_out(auto=True)
        else:
            self._save_extra()

    def _xint(self, name):
        v = self.stats["x"].get(name)
        return v if isinstance(v, int) and not isinstance(v, bool) and v >= 0 else 0

    def _finish(self, paid, opened):
        """Rundenende verbuchen: Statistik, Verlauf, Fairness-Log, Runde löschen."""
        r = self.round
        bet = r["bet"]
        m100 = self.last_mult
        chipstats.record(self.stats, bet, paid, m100)
        self.hist = (self.hist + [m100])[-ML.HISTORY:]
        outcome = {"grid": r["grid"], "mines": r["mines"], "open": list(opened), "m": m100, "bet": bet}
        fairness.record(self.fairlog, self._entry(), outcome)
        self.fair_sel = None
        if self.auto is not None:
            self.auto["net"] += paid - bet
        self.round = None

    def _cash_out(self, auto=False):
        """Auszahlen (Enter/C/Knopf) - automatisch bei vollem Feld/Höchstgewinn."""
        if self.state != PLAY or self.round is None:
            return
        k = self._k()
        if k < 1:
            if not auto:
                self._say(t("lmi.need_gem"), 1.2)
            return
        bet = self.round["bet"]
        m100 = ML.mult100(self.grid, self.mines, k)
        self.win, self.carry = ML.payout_carry(bet, self.grid, self.mines, k, self.carry)
        self.last_mult = m100
        opened = list(self.round["open"])
        clear = ML.is_clear(self.grid, self.mines, k)
        maxed = ML.is_max(self.grid, self.mines, k)
        lamabank.credit(self.win, KEY, save=False)
        self.pending += self.win
        if clear:
            self.stats["x"]["clears"] = self._xint("clears") + 1
        self._finish(self.win, opened)
        self._save_extra()
        self.score = lamabank.score_for(KEY)
        self.state = CASHED
        self.res_t = 0.0
        self.res_delay = FLIP_T * self._tf() if auto else 0.0
        sub = f"{self._fmt(m100)}   ·   {t('cas.bet')}: {self._group(bet)}"
        if maxed:
            title = t("lmi.cashed", n=self._group(self.win))
        elif clear:
            title = t("lmi.cleared")
        else:
            title = t("lmi.cashed", n=self._group(self.win))
        if clear and not maxed:
            sub = t("lmi.cashed", n=self._group(self.win)) + "   ·   " + self._fmt(m100)
        self.result = (title, sub, ui.GOLD)
        self.cash_info = dict(clear=clear, maxed=maxed, m100=m100, shown=False)
        # Erfolge sofort (nichts geht beim Verlassen verloren)
        if clear:
            self.ach_event("lmi_clear")
        if m100 >= ML.BRAVE_MULT:
            self.ach_event("lmi_brave")
        if m100 >= ML.LEGEND_MULT:
            self.ach_event("lmi_legend")
        if not auto:
            self._celebrate()

    def _celebrate(self):
        """Gewinn zeigen: Zähler, Popup, Münzen, Konfetti, Rest des Feldes."""
        info = self.cash_info
        if info["shown"]:
            return
        info["shown"] = True
        self.pending = max(0, self.pending - self.win)
        m100 = info["m100"]
        if info["maxed"]:
            self.banner = [t("chip.max_win"), 3.0]
            ui.spawn_confetti(self.width, self.height, 140)
            self._coin_shower(90)
            self.play_sound("level")
            self.rumble(260)
        elif info["clear"] or m100 >= PARTY_MULT:
            ui.spawn_confetti(self.width, self.height, 110)
            self._coin_shower(60)
            self.play_sound("level")
            self.rumble(200)
        else:
            if m100 >= BIG_MULT:
                self._coin_shower(28)
            self.play_sound("win")
        self.mood = ["cheer", 0.0]
        self._reveal_rest(self._opened_last())

    def _opened_last(self):
        """Ursprung der Rest-Aufdeckung: zuletzt gefundener Edelstein."""
        last = None
        for e in reversed(self.fairlog[-1:]):
            o = e.get("o")
            if isinstance(o, dict) and isinstance(o.get("open"), list) and o["open"]:
                last = o["open"][-1]
        return last if isinstance(last, int) and 0 <= last < len(self.cells) else len(self.cells) // 2

    def _reveal_rest(self, origin):
        """Alle noch verdeckten Felder abgedunkelt aufdecken - als Welle ab origin."""
        n = self.grid
        ox, oy = origin % n, origin // n
        for i, c in enumerate(self.cells):
            if c["face"] != "hidden":
                continue
            dx, dy = (i % n) - ox, (i // n) - oy
            d = math.sqrt(dx * dx + dy * dy)
            c["face"] = "mine" if self.is_mine[i] else "gem"
            c["dim"] = True
            c["flip"] = 0.0
            c["delay"] = (0.08 + WAVE_T * d) * self._tf()

    def _explode(self):
        """Mine geht hoch: Blitz, Feuer, Rauch, Trümmer, Wackeln."""
        i = self.hit
        r = self.tile_r[i]
        x, y = r.center
        self.boom_t = 0.0
        self.shake = SHAKE_T
        self.flashes.append(dict(x=x, y=y, t=0.0, dur=0.45, r=int(self.cell * 1.6)))
        self._spawn(x, y, 26, (255, 196, 80), 380, False, 4.0, 0.45, 300, "fire")
        self._spawn(x, y, 18, (255, 110, 50), 300, False, 3.6, 0.55, 200, "fire")
        self._spawn(x, y, 14, (84, 78, 90), 120, True, 7.0, 1.3, -60, "smoke")
        self._spawn(x, y, 16, (70, 60, 66), 340, True, 3.0, 1.0, 1100, "debris")
        self.play_sound("explode")
        self.rumble(320)
        self.mood = ["scared", 0.0]
        self.sooty = True
        self.cart_fill = 0
        self._reveal_rest(i)
        for c in self.cells:
            if c["dim"]:
                c["delay"] += 0.22 * self._tf()

    def _gem_found(self, i):
        """Edelstein fertig aufgeklappt: Funkeln, steigender Ton, Popup."""
        r = self.tile_r[i]
        k = max(1, self.cells[i]["k"])
        self._spawn(r.centerx, r.centery, 12, (120, 255, 220), 200, False, 2.6, 0.5, 120, "spark")
        self._spawn(r.centerx, r.centery, 6, (255, 255, 255), 150, False, 2.0, 0.4, 60, "star")
        self.flashes.append(dict(x=r.centerx, y=r.centery, t=0.0, dur=0.3, r=int(self.cell * 0.9),
                                 col=(40, 160, 130)))
        step = GEM_STEPS[min(len(GEM_STEPS) - 1, max(0, k - 1))]
        audio.tone(523.25 * 2 ** (step / 12.0), 0.12, self.settings, "sine", 0.28)
        if self.state == PLAY and self.round is not None:
            m100 = ML.mult100(self.grid, self.mines, k)
            self._popup(r.centerx, r.centery - int(self.cell * 0.15), self._fmt(m100), ui.GREEN, 0.9)
            self.mood = ["happy", 0.0]
            self.cart_fill = self._cart_level()
        elif self.state == CASHED:
            self.cart_fill = 7 if self.cash_info.get("clear") else max(1, self.cart_fill)

    def _to_idle(self):
        """Ergebnis schließen: Ruhezustand, das aufgedeckte Feld bleibt sichtbar."""
        if self.state in (CASHED,) and not self.cash_info["shown"]:
            self._celebrate()
        for c in self.cells:
            if c["flip"] < 1.0 or c["delay"] > 0:
                c["flip"], c["delay"] = 1.0, 0.0
        self.state = IDLE
        self.result = None
        self.boom_t = None
        self.res_t = 0.0
        self.pending = 0
        if self.mood[0] == "scared":
            self.mood = ["idle", 0.0]
        self._clamp_bet()
        self._check_broke()

    # ----- Einstellungen ändern ------------------------------------------------
    def _idle_change(self):
        """Vor einer Änderung im Ruhezustand: altes Feld verdecken."""
        if self.field or any(c["face"] != "hidden" for c in self.cells):
            self._blank_board()

    def _set_bet(self, direction):
        if self.state != IDLE or self.auto is not None:
            return
        i = ML.BETS.index(self.bet)
        j = max(0, min(len(ML.BETS) - 1, i + direction))
        if j != i:
            self.bet = ML.BETS[j]
            self._set_opt("bet", self.bet)
            self.play_sound("select")
            if self.bet > lamabank.balance():
                self._say(t("cas.not_enough"), 1.2)

    def _set_grid(self, grid):
        if self.state != IDLE or self.auto is not None or grid == self.grid or grid not in ML.GRIDS:
            return
        self.grid = grid
        self.mines = ML.clamp_mines(grid, self.mines)
        self._set_opt("grid", grid)
        self._set_opt("mines", self.mines)
        self.cursor = (grid * grid) // 2
        self._layout()
        self._blank_board()
        self.cart_fill = 0
        self.play_sound("click")

    def _set_mines(self, mines):
        if self.state != IDLE or self.auto is not None:
            return
        mines = ML.clamp_mines(self.grid, mines)
        if mines == self.mines:
            return
        self.mines = mines
        self._set_opt("mines", mines)
        self._idle_change()
        self.play_sound("click")

    def _toggle_turbo(self):
        self.turbo = not self.turbo
        self._set_opt("turbo", self.turbo)
        self._say(t("lmi.turbo_on") if self.turbo else t("lmi.turbo_off"), 1.0)
        self.play_sound("click")

    def _open_window(self, name):
        if self.window == name:
            self.window = None
            self.play_sound("click")
            return
        if name == "auto":
            if self.auto is not None:
                self._stop_auto(t("chip.auto_done"))
                return
            if self.state in (BOOM, CASHED) and self._result_visible() and self.res_t >= READ_T:
                self._to_idle()
            if self.state != IDLE or self.broke:
                self._say(t("lmi.locked"), 1.2)
                return
            self._idle_change()
            self.kb = False
        self.window = name
        if name == "fair":
            self.fair_scroll = 0
            self.fair_sel = len(self.fairlog) - 1 if self.fairlog else None
        self.play_sound("click")

    # ----- Auto-Modus ------------------------------------------------------------
    def _marks(self):
        return self.marks[str(self.grid)]

    def _toggle_mark(self, i):
        lst = self._marks()
        if i in lst:
            lst.remove(i)
        else:
            if len(lst) >= ML.gems(self.grid, self.mines):
                self._say(t("lmi.auto_too_many", n=ML.gems(self.grid, self.mines)), 1.4)
                self.play_sound("hit")
                return
            lst.append(i)
            lst.sort()
            self.cells[i]["mark"] = 0.0
        self._save_extra()
        self.play_sound("rotate")

    def _clear_marks(self):
        self.marks[str(self.grid)] = []
        self._save_extra()
        self.play_sound("click")

    def _start_auto(self):
        lst = self._marks()
        if not lst:
            self._say(t("lmi.auto_need"), 1.6)
            self.play_sound("hit")
            return
        if len(lst) > ML.gems(self.grid, self.mines):
            self._say(t("lmi.auto_too_many", n=ML.gems(self.grid, self.mines)), 1.6)
            self.play_sound("hit")
            return
        if self.bet > lamabank.balance():
            self._say(t("cas.not_enough"))
            self.play_sound("hit")
            return
        self.window = None
        self.auto = {"left": self.auto_rounds, "net": 0, "t": 0.0, "queue": []}
        self.play_sound("powerup")

    def _stop_auto(self, msg=None):
        """Auto beenden. Eine laufende Runde bleibt offen - der Spieler macht von
        Hand weiter (aufdecken oder auszahlen)."""
        if self.auto is None:
            return
        self.auto = None
        if msg:
            self._say(msg, 2.0)
        self.play_sound("click")
        if self.state == IDLE:
            self._check_broke()

    def _auto_tick(self, dt):
        a = self.auto
        a["t"] -= dt
        if a["t"] > 0:
            return
        if self.state == IDLE:
            # Grenzen prüfen, dann die nächste Runde
            if a["left"] <= 0:
                self._stop_auto(t("chip.auto_done"))
                return
            if self.auto_win > 0 and a["net"] >= self.auto_win:
                self._stop_auto(t("chip.auto_hit_win"))
                return
            if self.auto_loss > 0 and -a["net"] >= self.auto_loss:
                self._stop_auto(t("chip.auto_hit_loss"))
                return
            if self.bet > lamabank.balance():
                self._stop_auto(t("chip.auto_no_chips"))
                return
            lst = self._marks()
            if not lst or len(lst) > ML.gems(self.grid, self.mines):
                self._stop_auto(t("lmi.auto_need"))
                return
            if self._play():
                a["left"] -= 1
                a["queue"] = list(lst)
                a["t"] = (AUTO_STEP_T + 0.1) * self._tf()
            else:
                self._stop_auto(t("chip.auto_no_chips"))
        elif self.state == PLAY:
            if any(c["flip"] < 1.0 for c in self.cells):
                return
            if a["queue"]:
                self._reveal(a["queue"].pop(0))
                a["t"] = AUTO_STEP_T * self._tf()
            else:
                self._cash_out(auto=True)
                self.res_delay = 0.0
                self._celebrate()

    # ===================================================== Effekte
    def _spawn(self, x, y, n, col, speed, up, r, life, grav, kind="dot"):
        for _ in range(n):
            a = self.rng.uniform(math.pi * 1.1, math.pi * 1.9) if up else self.rng.uniform(0, math.tau)
            sp = speed * self.rng.uniform(0.3, 1.0) * self.k
            self.particles.append(dict(x=x, y=y, vx=math.cos(a) * sp, vy=math.sin(a) * sp,
                                       r=r * self.k * self.rng.uniform(0.6, 1.25), col=col, t=0.0,
                                       life=life * self.rng.uniform(0.7, 1.15), g=grav * self.k, kind=kind))
        if len(self.particles) > 260:
            self.particles = self.particles[-260:]

    def _coin_shower(self, n=40):
        W = self.width
        for _ in range(n):
            self.coins_fx.append(dict(
                x=self.rng.uniform(W * 0.05, W * 0.95),
                y=self.rng.uniform(-self.height * 0.4, -10),
                vx=self.rng.uniform(-60, 60) * self.k,
                vy=self.rng.uniform(-80, 120) * self.k,
                spin=self.rng.uniform(0, math.tau),
                vs=self.rng.uniform(6, 14),
                r=self.rng.uniform(7, 12) * max(0.7, self.k)))
        if len(self.coins_fx) > 180:
            self.coins_fx = self.coins_fx[-180:]

    def _popup(self, x, y, text, color=None, dur=1.4):
        self.popups.append(dict(x=x, y=y, text=text, color=color or ui.GOLD, t=0.0, dur=dur))

    # ===================================================== Eingabe
    def _tile_at(self, pos):
        for i, r in enumerate(self.tile_r):
            if r.collidepoint(pos):
                return i
        return None

    def handle_event(self, event):
        kind = event.kind
        if kind in (InputEvent.MOUSEMOVE, InputEvent.MOUSEDOWN,
                    InputEvent.MOUSEUP) and event.pos is not None:
            if kind == InputEvent.MOUSEMOVE and self.board_r.collidepoint(event.pos):
                self.kb = False
            self.mouse = event.pos
        if self.broke:
            if kind == InputEvent.MOUSEDOWN or (
                    kind == InputEvent.KEYDOWN and self._is_action(event.key)):
                self._take_credit()
            return
        if kind == InputEvent.KEYDOWN:
            self._on_key(event.key, event.repeat)
        elif kind == InputEvent.WHEEL:
            if self.window == "fair":
                self.fair_scroll = max(0, self.fair_scroll - (1 if event.delta > 0 else -1))
            elif self.window is None and self.state == IDLE:
                self._set_bet(1 if event.delta > 0 else -1)
        elif kind == InputEvent.MOUSEDOWN and event.button == 1:
            self._on_click(event.pos)

    def _on_key(self, key, repeat=False):
        low = key.lower() if len(key) == 1 else key
        # Fenster ----------------------------------------------------------
        if key == "Escape":
            if self.window is not None:
                self.window = None
                self.play_sound("click")
            elif self.auto is not None:
                self._stop_auto(t("chip.auto_done"))
            return
        if low in ("a", "s", "f", "t") and len(key) == 1:
            if repeat:
                return
            if low == "t":
                self._toggle_turbo()
            else:
                self._open_window({"a": "auto", "s": "stats", "f": "fair"}[low])
            return
        if self.window == "auto":
            self._auto_key(key)
            return
        if self.window is not None:
            if self._is_action(key) and not repeat:
                self.window = None
            return
        # Auto läuft: Leertaste/Enter = STOPP --------------------------------
        if self.auto is not None:
            if self._is_action(key) and not repeat:
                self._stop_auto(t("chip.auto_done"))
            return
        # Ergebnis: nach kurzer Lesezeit weiter wie im Ruhezustand ------------
        if self.state in (BOOM, CASHED):
            if not self._result_visible() or self.res_t < READ_T or repeat:
                return
            self._to_idle()
            if self.broke:
                return
        if self.state == IDLE:
            if self._is_action(key):
                if not repeat:
                    self._play()
            elif key in ("1", "2", "3", "4"):
                self._set_grid(ML.GRIDS[int(key) - 1])
            elif key in ("Left", "Right"):
                self._set_mines(self.mines + (1 if key == "Right" else -1))
            elif key in ("plus", "KP_Add", "Up", "equal"):
                self._set_bet(1)
            elif key in ("minus", "KP_Subtract", "Down"):
                self._set_bet(-1)
            return
        if self.state == PLAY:
            n = self.grid
            if key in ("Left", "Right", "Up", "Down"):
                if self.kb:
                    x, y = self.cursor % n, self.cursor // n
                    x += {"Left": -1, "Right": 1}.get(key, 0)
                    y += {"Up": -1, "Down": 1}.get(key, 0)
                    self.cursor = max(0, min(n - 1, y)) * n + max(0, min(n - 1, x))
                self.kb = True
            elif key == "space":
                if not repeat:
                    if not self.kb:
                        self.kb = True
                    else:
                        self._reveal(self.cursor)
            elif key in ("Return", "KP_Enter") or low == "c":
                if not repeat:
                    self._cash_out()
            elif low == "r":
                if not repeat:
                    self._random_tile()

    def _auto_key(self, key):
        """Tasten im Auto-Fenster: Pfeile + Leertaste markieren, Enter startet."""
        n = self.grid
        if key in ("Left", "Right", "Up", "Down"):
            if self.kb:
                x, y = self.cursor % n, self.cursor // n
                x += {"Left": -1, "Right": 1}.get(key, 0)
                y += {"Up": -1, "Down": 1}.get(key, 0)
                self.cursor = max(0, min(n - 1, y)) * n + max(0, min(n - 1, x))
            self.kb = True
        elif key == "space":
            if self.kb:
                self._toggle_mark(self.cursor)
            self.kb = True
        elif key in ("Return", "KP_Enter"):
            self._start_auto()
        elif key in ("Delete", "BackSpace"):
            self._clear_marks()

    def _random_tile(self):
        """R: zufälliges verdecktes Feld (Zufall der Oberfläche, nicht der Runde)."""
        if self.state != PLAY or self.round is None:
            return
        free = [i for i, c in enumerate(self.cells) if c["face"] == "hidden"]
        if not free:
            return
        i = free[self.rng.randrange(len(free))]
        self.cursor = i
        self._reveal(i)

    def _on_click(self, pos):
        # Werkzeug-Knöpfe (immer erreichbar)
        for name, r in self.r_tools:
            if r.collidepoint(pos):
                if name == "turbo":
                    self._toggle_turbo()
                else:
                    self._open_window(name)
                return
        if self.window == "auto":
            self._auto_click(pos)
            return
        if self.window is not None:
            self._window_click(pos)
            return
        if self.auto is not None:
            if self.r_go.collidepoint(pos):
                self._stop_auto(t("chip.auto_done"))
            return
        if self.state in (BOOM, CASHED):
            if not self._result_visible() or self.res_t < READ_T:
                return
            self._to_idle()
            if self.broke:
                return
            if not self.r_go.collidepoint(pos):
                return
        if self.state == IDLE:
            if self.r_minus.collidepoint(pos):
                self._set_bet(-1)
            elif self.r_plus.collidepoint(pos):
                self._set_bet(1)
            elif self.r_go.collidepoint(pos):
                self._play()
            elif self.r_cash.collidepoint(pos):
                self._say(t("lmi.press_play"), 1.4)
            elif self.board_r.collidepoint(pos):
                self._say(t("lmi.press_play"), 1.4)
                self.play_sound("hit")
            else:
                for i, r in enumerate(self.r_grids):
                    if r.collidepoint(pos):
                        self._set_grid(ML.GRIDS[i])
                        return
                if self.r_mminus.collidepoint(pos):
                    self._set_mines(self.mines - 1)
                elif self.r_mplus.collidepoint(pos):
                    self._set_mines(self.mines + 1)
                else:
                    for i, r in enumerate(self.r_presets):
                        if r.collidepoint(pos):
                            self._set_mines(ML.MINE_PRESETS[self.grid][i])
                            return
            return
        if self.state == PLAY:
            if self.r_cash.collidepoint(pos):
                self._cash_out()
            elif self.r_random.collidepoint(pos):
                self._random_tile()
            else:
                i = self._tile_at(pos)
                if i is not None:
                    self.kb = False
                    self.cursor = i
                    self._reveal(i)

    def _auto_click(self, pos):
        if self.r_auto_close.collidepoint(pos):
            self.window = None
            self.play_sound("click")
            return
        i = self._tile_at(pos)
        if i is not None:
            self.kb = False
            self.cursor = i
            self._toggle_mark(i)
            return
        for row, (opts, key) in enumerate(((ROUND_OPTS, "auto_rounds"), (LIMIT_OPTS, "auto_win"),
                                           (LIMIT_OPTS, "auto_loss"))):
            for j, r in enumerate(self.auto_rows[row][1]):
                if r.collidepoint(pos):
                    setattr(self, key, opts[j])
                    self._set_opt(key, opts[j])
                    self.play_sound("click")
                    return
        if self.r_auto_clear.collidepoint(pos):
            self._clear_marks()
        elif self.r_auto_start.collidepoint(pos):
            self._start_auto()

    def _window_click(self, pos):
        """Klick in Statistik-/Fairness-Fenster (außerhalb = schließen)."""
        rect, close = self._window_rect()
        if close.collidepoint(pos) or not rect.collidepoint(pos):
            self.window = None
            self.play_sound("click")
            return
        if self.window != "fair":
            return
        if self.r_fair_new.collidepoint(pos):
            if self.round is not None:
                self._say(t("lmi.locked"), 1.4)
                self.play_sound("hit")
            else:
                fairness.new_client(self.fair, self.rng)
                self._save_extra()
                self.play_sound("select")
            return
        if self.r_fair_verify.collidepoint(pos) and self.fair_sel is not None:
            self._verify(self.fair_sel)
            return
        for idx, r in self.r_fair_rows:
            if r.collidepoint(pos):
                self.fair_sel = idx
                self.play_sound("click")
                return

    def _verify(self, idx):
        """Nachrechnen: Prüfcode = sha256(Seed) und Minenfeld aus den Seeds."""
        if not 0 <= idx < len(self.fairlog):
            return
        e = self.fairlog[idx]
        ok = fairness.verify(e)
        o = e.get("o")
        field = None
        if isinstance(o, dict) and o.get("grid") in ML.GRIDS and isinstance(o.get("mines"), int) \
                and not isinstance(o.get("mines"), bool) and 1 <= o["mines"] <= ML.max_mines(o["grid"]):
            field = ML.field_from_seed(o["grid"], o["mines"], e["s"], e["c"], e["n"])
            opened = o.get("open") if isinstance(o.get("open"), list) else []
            m = o.get("m")
            if m == 0:
                ok = ok and bool(opened) and opened[-1] in field and \
                    not any(v in field for v in opened[:-1])
            else:
                ok = ok and not any(v in field for v in opened) and \
                    m == ML.mult100(o["grid"], o["mines"], len(opened))
        self.fair_checks[e["n"]] = (ok, field)
        self.play_sound("point" if ok else "hit")

    # ===================================================== Update
    def update(self, dt):
        self.time += dt
        if self._lang != get_language():
            self._lang = get_language()
            self._layout()
        target = max(0, lamabank.balance() - self.pending)
        diff = target - self.shown_chips
        if abs(diff) < 0.5:
            self.shown_chips = float(target)
        else:
            step = max(abs(diff) * min(1.0, dt * 4.5), 60.0 * dt)
            self.shown_chips += math.copysign(min(abs(diff), step), diff)
            if diff > 0:
                self._tick_cd -= dt
                if self._tick_cd <= 0:
                    self._tick_cd = 0.07
                    audio.tone(1500 + self.rng.random() * 300, 0.025, self.settings, "square", 0.08)
        if self.toast:
            self.toast[1] -= dt
            if self.toast[1] <= 0:
                self.toast = None
        if self.banner:
            self.banner[1] -= dt
            if self.banner[1] <= 0:
                self.banner = None
        self.shake = max(0.0, self.shake - dt)
        self.mood[1] += dt
        self._update_cells(dt)
        self._update_fx(dt)
        if self.state == BOOM:
            if self.boom_t is None:
                if self.cells[self.hit]["flip"] >= 1.0:
                    self._explode()
            else:
                self.boom_t += dt
        if self.state in (BOOM, CASHED):
            if self.res_delay > 0:
                self.res_delay -= dt
                if self.res_delay <= 0 and self.state == CASHED:
                    self._celebrate()
            else:
                self.res_t += dt
                limit = AUTO_RESULT_T * self._tf() if self.auto is not None else RESULT_T
                if self.res_t >= limit:
                    self._to_idle()
                    if self.auto is not None:
                        self.auto["t"] = AUTO_PAUSE_T * self._tf()
        if self.auto is not None and self.state in (IDLE, PLAY):
            self._auto_tick(dt)

    def _result_visible(self):
        return self.result is not None and self.res_delay <= 0

    def _update_cells(self, dt):
        ft = FLIP_T * self._tf()
        for i, c in enumerate(self.cells):
            c["mark"] += dt
            if c["delay"] > 0:
                c["delay"] -= dt
                if c["delay"] > 0:
                    continue
            if c["flip"] < 1.0:
                c["flip"] = min(1.0, c["flip"] + dt / ft)
                if c["flip"] >= 1.0:
                    c["pop"] = 0.0
                    if c["face"] == "gem" and not c["dim"]:
                        self._gem_found(i)
            elif c["pop"] < POP_T:
                c["pop"] = min(POP_T, c["pop"] + dt)

    def _update_fx(self, dt):
        alive = []
        for p in self.particles:
            p["t"] += dt
            p["vy"] += p["g"] * dt
            if p["kind"] == "smoke":
                p["vx"] *= max(0.0, 1.0 - dt * 1.5)
                p["vy"] *= max(0.0, 1.0 - dt * 1.5)
            p["x"] += p["vx"] * dt
            p["y"] += p["vy"] * dt
            if p["t"] < p["life"]:
                alive.append(p)
        self.particles = alive
        alive = []
        for c in self.coins_fx:
            c["vy"] += 900 * self.k * dt
            c["x"] += c["vx"] * dt
            c["y"] += c["vy"] * dt
            c["spin"] += c["vs"] * dt
            if c["y"] < self.height + 30:
                alive.append(c)
        self.coins_fx = alive
        for p in self.popups:
            p["t"] += dt
        self.popups = [p for p in self.popups if p["t"] < p["dur"]]
        for f in self.flashes:
            f["t"] += dt
        self.flashes = [f for f in self.flashes if f["t"] < f["dur"]]

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        s.blit(D.background(self.width, self.height), (0, 0))
        self._draw_board(s)
        self._draw_panel(s)
        self._draw_hud(s)
        self._draw_tools(s)
        self._draw_history(s)
        self._draw_strip(s)
        self._draw_fx(s)
        if self.result is not None and self._result_visible():
            self._draw_result(s)
        if self.banner:
            self._draw_banner(s)
        if self.window in ("stats", "fair"):
            self._draw_window(s)
        if self.toast:
            self._draw_toast(s)
        self._draw_tooltip(s)
        if self.broke:
            self._draw_broke(s)

    # ----- Spielbrett -------------------------------------------------------------
    def _shake_off(self):
        if self.shake <= 0:
            return 0, 0
        a = 7 * self.k * self.shake / SHAKE_T
        return int(math.sin(self.time * 71) * a), int(math.cos(self.time * 53) * a * 0.7)

    def _draw_board(self, s):
        ox, oy = self._shake_off()
        br = self.board_r.move(ox, oy)
        s.blit(D.board(br.w, self.board_pad), br.topleft)
        cell = self.cell
        play = self.state == PLAY and self.window is None and self.auto is None
        marking = self.window == "auto" or self.auto is not None
        hover = self._tile_at(self.mouse) if (play or self.window == "auto") and not self.kb else None
        marks = self._marks() if marking else []
        for i, c in enumerate(self.cells):
            r = self.tile_r[i].move(ox, oy)
            f = c["flip"]
            if c["face"] == "hidden":
                if f < 1.0:
                    # Einwachsen beim Rundenstart (Welle von links oben)
                    if c["delay"] > 0:
                        continue
                    sz = max(2, int(cell * (0.55 + 0.45 * f)))
                    img = pygame.transform.smoothscale(D.tile_hidden(cell, self._var(i)), (sz, sz))
                    s.blit(img, img.get_rect(center=r.center))
                    continue
                hot = i == hover
                img = D.tile_hidden(cell, self._var(i), hot)
                s.blit(img, r.move(0, -max(1, int(2 * self.k))) if hot else r)
                if i in marks:
                    D.draw_mark(s, r, self.accent, self.time, i)
                continue
            if c["delay"] > 0:
                s.blit(D.tile_hidden(cell, self._var(i)), r)      # wartet aufs Aufklappen
                continue
            if f < 1.0:
                self._draw_flip(s, r, i, c)
                continue
            self._draw_face(s, r, i, c)
        # Tastatur-Cursor
        if self.kb and (play or self.window == "auto"):
            r = self.tile_r[self.cursor].move(ox, oy).inflate(max(4, int(6 * self.k)), max(4, int(6 * self.k)))
            pulse = 0.5 + 0.5 * math.sin(self.time * 6.0)
            col = ui.mix(CD.COL_GOLD, (255, 255, 255), 0.45 * pulse)
            pygame.draw.rect(s, col, r, max(2, int(3 * self.k)), border_radius=max(3, int(cell * 0.2)))
        self._draw_particles(s)
        for fl in self.flashes:
            p = fl["t"] / fl["dur"]
            rr = max(2, int(fl["r"] * (0.4 + 0.8 * p)))
            col = fl.get("col", (255, 200, 120))
            fade = 1.0 - p
            g = D.glow(rr, (int(col[0] * fade), int(col[1] * fade), int(col[2] * fade)))
            s.blit(g, (fl["x"] - rr - 1 + ox, fl["y"] - rr - 1 + oy), special_flags=pygame.BLEND_RGB_ADD)
            if "col" not in fl and p < 0.5:
                ring = max(2, int(fl["r"] * (0.3 + 1.2 * p)))
                pygame.draw.circle(s, ui.mix((255, 240, 200), (255, 140, 60), p * 2),
                                   (fl["x"] + ox, fl["y"] + oy), ring, max(1, int(4 * self.k * (1 - p * 2))))

    def _var(self, i):
        return D._hash(i * 7 + self.grid) % 4

    def _draw_flip(self, s, r, i, c):
        """Aufklappen: erst schrumpft der Stein (Breite), dann wächst die Vorderseite."""
        f = c["flip"]
        cell = self.cell
        lift = math.sin(math.pi * f) * 0.10
        if f < 0.5:
            img = D.tile_hidden(cell, self._var(i))
            w = max(1, int(cell * (1.0 - 2.0 * f) * (1 + lift)))
        else:
            img = D.face(cell, c["face"], c["dim"])
            w = max(1, int(cell * (2.0 * f - 1.0) * (1 + lift)))
        h = max(1, int(cell * (1 + lift)))
        img = pygame.transform.scale(img, (w, h))
        s.blit(img, img.get_rect(center=(r.centerx, r.centery - int(cell * lift * 0.5))))

    def _draw_face(self, s, r, i, c):
        """Aufgedecktes Feld in Ruhe (Edelstein federt nach, funkelt)."""
        cell = self.cell
        kind = c["face"]
        if kind == "gem" and not c["dim"] and c["pop"] < POP_T:
            s.blit(D.tile_open(cell), r)
            p = c["pop"] / POP_T
            sc = 0.80 * (1.0 + 0.22 * math.sin(math.pi * p) * (1 - p * 0.3))
            g = max(4, int(cell * sc) // 2 * 2)
            img = D.gem(g)
            s.blit(img, img.get_rect(center=r.center))
            return
        s.blit(D.face(cell, kind, c["dim"]), r)
        if kind == "gem" and not c["dim"]:
            ph = math.sin(self.time * 2.6 + i * 1.7)
            if ph > 0.55:
                a = (ph - 0.55) / 0.45
                v = D._hash(i * 13 + 5)
                sx = r.x + cell * (0.30 + (v % 40) / 100.0)
                sy = r.y + cell * (0.22 + ((v >> 8) % 30) / 100.0)
                D.sparkle(s, sx, sy, max(1.5, cell * 0.10 * a), (220, 255, 246))
        elif kind == "boom":
            # glimmender Rand um die getroffene Mine
            pulse = 0.5 + 0.5 * math.sin(self.time * 5.0)
            pygame.draw.rect(s, ui.mix((255, 90, 70), (255, 200, 120), pulse), r.inflate(2, 2),
                             max(2, int(3 * self.k)), border_radius=max(2, int(cell * 0.16)))

    def _draw_particles(self, s):
        for p in self.particles:
            f = p["t"] / p["life"]
            if p["kind"] == "smoke":
                # weicher Rauch: additiver grauer Lichthof, wird größer und blasser
                lvl = min(5, int((1.0 - f) * 6))
                if lvl <= 0:
                    continue
                rr = max(2, int(p["r"] * (1.0 + 2.2 * f)) // 2 * 2)
                a = lvl / 6.0
                g = D.glow(rr, (int(p["col"][0] * a), int(p["col"][1] * a), int(p["col"][2] * a)))
                s.blit(g, (int(p["x"]) - rr - 1, int(p["y"]) - rr - 1), special_flags=pygame.BLEND_RGB_ADD)
                continue
            elif p["kind"] == "star":
                D.sparkle(s, p["x"], p["y"], max(1.0, p["r"] * 1.6 * (1 - f)), (255, 255, 255))
                continue
            else:
                r = p["r"] * (1 - f)
                col = p["col"]
                if p["kind"] == "fire":
                    col = ui.mix(p["col"], (120, 40, 30), f)
            if r >= 0.5:
                pygame.draw.circle(s, col, (int(p["x"]), int(p["y"])), max(1, int(r)))

    # ----- Seitenleiste -----------------------------------------------------------
    def _draw_panel(self, s):
        if self.window == "auto":
            self._draw_auto(s)                  # eigenes Fenster an derselben Stelle
            return
        ui.draw_panel(s, self.panel_r, accent_top=self.accent)
        self._draw_mascot(s)
        self._draw_head(s)
        if self.state == IDLE and self.round is None:
            self._draw_setup(s)
            self._draw_info(s, self.r_presets[0].bottom + max(4, int(8 * self.k)))
        else:
            self._draw_info(s, self.info_y)
        self._draw_ladder(s)
        self._draw_hint(s)

    def _draw_mascot(self, s, r=None):
        """Maskottchen mit Spitzhacke und Lore in r (Standard: Kopf der Seitenleiste)."""
        r = r or self.r_mascot
        ms = r.w
        k = self.k
        mood, mt = self.mood
        pose, dx, dy = "stand", 0.0, 0.0
        if mood == "happy" and mt < 0.45:
            pose = "hop"
            dy = -math.sin(math.pi * mt / 0.45) * ms * 0.14
        elif mood == "cheer" and mt < 1.35:
            ph = (mt % 0.45) / 0.45
            pose = "hop"
            dy = -math.sin(math.pi * ph) * ms * 0.16
        elif mood == "scared" or (self.sooty and self.state == BOOM):
            pose = "fall"
            if mt < 0.6:
                dx = math.sin(mt * 60) * ms * 0.04 * (1 - mt / 0.6)
                dy = -math.sin(math.pi * min(1.0, mt / 0.35)) * ms * 0.08
        if pose == "stand":
            dy += math.sin(self.time * 2.2) * ms * 0.012
        ground = r.bottom - max(1, int(2 * k))
        # Felsvorsprung, Spitzhacke, Lore
        pygame.draw.ellipse(s, (34, 30, 44), (r.x - int(ms * 0.04), ground - int(ms * 0.05),
                                              int(ms * 1.0 + self.cart_w * 1.05), int(ms * 0.12)))
        D.draw_pickaxe(s, r.x + int(ms * 0.10), ground - int(ms * 0.02), ms * 0.40, k)
        sh = pygame.Rect(0, 0, int(ms * 0.5), max(3, int(ms * 0.07)))
        sh.center = (r.x + int(ms * 0.52), ground)
        pygame.draw.ellipse(s, (14, 12, 20), sh)
        img = D.miner_llama(ms, pose, self.accent, self.sooty)
        s.blit(img, (int(r.x + dx), int(r.y + dy + ms * 0.04)))
        # Lichthof der Stirnlampe (additiv)
        lr = max(3, int(ms * 0.10))
        flick = 0.85 + 0.15 * math.sin(self.time * 9.0) * math.sin(self.time * 3.3)
        g = D.glow(lr, (int(150 * flick), int(130 * flick), int(60 * flick)))
        lx = r.x + dx + ms * 0.835
        ly = r.y + dy + ms * 0.04 + ms * 0.073
        s.blit(g, (int(lx - lr - 1), int(ly - lr - 1)), special_flags=pygame.BLEND_RGB_ADD)
        # Lore mit Edelsteinen
        D.draw_cart(s, r.x + int(ms * 0.92), ground, self.cart_w, k, self.cart_fill, max(4, int(self.cart_w * 0.30)))
        # Reaktion: Funkeln (Freude) bzw. Ausrufezeichen + Rauch (Schreck)
        hx, hy = r.x + dx + ms * 0.74, r.y + dy + ms * 0.04
        if mood in ("happy", "cheer") and mt < (0.6 if mood == "happy" else 1.4):
            for j in range(3):
                a = self.time * 2.0 + j * 2.1
                D.sparkle(s, hx + math.cos(a) * ms * 0.22, hy - ms * 0.04 + math.sin(a) * ms * 0.10,
                          max(1.5, ms * 0.05 * (0.6 + 0.4 * math.sin(self.time * 9 + j))), (200, 255, 236))
        elif mood == "scared" and mt < 1.6:
            bx, by = int(hx + ms * 0.18), int(hy - ms * 0.02)
            rr = max(5, int(ms * 0.09))
            pygame.draw.circle(s, (250, 250, 250), (bx, by), rr)
            pygame.draw.circle(s, ui.RED, (bx, by), rr, max(1, int(2 * k)))
            ex = self.f_btn.render("!", True, ui.RED)
            s.blit(ex, ex.get_rect(center=(bx, by)))
            for j in range(3):
                ph = (mt * 0.8 + j * 0.33) % 1.0
                pr = max(2, int(ms * (0.05 + 0.07 * ph)) // 2 * 2)
                a = 1.0 - ph
                g = D.glow(pr, (int(80 * a), int(76 * a), int(86 * a)))
                px = int(r.x + dx + ms * (0.66 - 0.06 * j))
                py = int(r.y + dy - ms * 0.20 * ph + ms * 0.02)
                s.blit(g, (px - pr - 1, py - pr - 1), special_flags=pygame.BLEND_RGB_ADD)

    def _draw_head(self, s):
        """Kopfzeile rechts neben Maskottchen und Lore: großer Multiplikator."""
        x0 = self.head_x
        x1 = self.panel_r.right - self.ppad
        w = x1 - x0
        y = self.r_mascot.y
        if self.state == IDLE and self.round is None:
            cap = t("lmi.max_mult")
            val = self._fmt(ML.mult100(self.grid, self.mines, ML.gems(self.grid, self.mines)))
            sub = None                     # Edelsteine/Minen als Bildchen (sprachunabhängig)
            col = CD.COL_GOLD
        elif self.state == BOOM:
            cap = t("lmi.cur_mult")
            val = "×0"
            sub = t("lmi.lost_short", n=self._group(self.lost))
            col = ui.RED
        elif self.state == CASHED:
            cap = t("lmi.cur_mult")
            val = self._fmt(self.last_mult)
            sub = "+" + self._group(self.win)
            col = CD.COL_GOLD
        else:
            k = self._k()
            cap = t("lmi.cur_mult")
            val = self._fmt(ML.mult100(self.grid, self.mines, k))
            sub = t("lmi.payout_now", n=self._group(self._win_at(self._round_bet(), k)))
            col = ui.GREEN if k > 0 else ui.TEXT
        img = self._fit(cap, (self.f_tiny, self.f_micro), w, ui.TEXT_DIM)
        s.blit(img, (x0, y))
        y += img.get_height()
        img = self._fit(val, (self.f_mult, self.f_big, self.f_btn), w, col)
        s.blit(img, (x0, y))
        y += img.get_height()
        if sub is None:
            ih = max(self.f_small.get_height(), int(self.f_big.get_height() * 0.9))
            self._draw_counts(s, x0, y + ih // 2 + 1, ML.gems(self.grid, self.mines),
                              self.mines, self.f_btn, ih)
            return
        img = self._fit(sub, (self.f_small, self.f_tiny, self.f_micro), w, ui.TEXT)
        s.blit(img, (x0, y))

    def _draw_counts(self, s, x, cy, gems, mines, font, ih, center_w=None):
        """Edelstein- und Minen-Bildchen mit Anzahl (links bei x bzw. mittig in
        center_w), senkrecht mittig bei cy."""
        parts = [(D.gem(ih), font.render(self._group(gems), True, self.accent)),
                 (D.mine(ih, True), font.render(str(mines), True, ui.RED))]
        gap = max(8, ih * 2 // 3)
        total = 0
        for icon, txt in parts:
            total += icon.get_width() + 3 + txt.get_width()
        total += gap
        if center_w is not None:
            x += (center_w - total) // 2
        for icon, txt in parts:
            s.blit(icon, icon.get_rect(midleft=(x, cy)))
            x += icon.get_width() + 3
            s.blit(txt, txt.get_rect(midleft=(x, cy)))
            x += txt.get_width() + gap

    def _fit(self, text, fonts, maxw, col):
        """Text in der ersten Schrift, die passt - sonst gekürzt mit '…'."""
        for f in fonts:
            if f.size(text)[0] <= maxw:
                return f.render(text, True, col)
        f = fonts[-1]
        while text and f.size(text + "…")[0] > maxw:
            text = text[:-1]
        return f.render(text + "…", True, col)

    def _draw_setup(self, s):
        x0, w = self.panel_inner
        hot = self.mouse
        img = self._fit(t("lmi.grid"), (self.f_tiny,), w, ui.TEXT_DIM)
        s.blit(img, (x0, self.lab_grid_y))
        for i, r in enumerate(self.r_grids):
            g = ML.GRIDS[i]
            self._btn(s, r, "%d×%d" % (g, g), active=g == self.grid, hot=r.collidepoint(hot))
            key = self.f_micro.render(str(i + 1), True, ui.TEXT_FAINT)
            s.blit(key, (r.x + max(4, r.h // 5), r.y + max(2, r.h // 10)))
        y = self.mines_row_y
        rh = self.row_h
        lab = self._fit(t("lmi.mines"), (self.f_small, self.f_tiny), self.r_mminus.x - x0 - 4, ui.TEXT)
        s.blit(lab, lab.get_rect(midleft=(x0, y + rh // 2)))
        self._btn(s, self.r_mminus, "", on=self.mines > 1, hot=self.r_mminus.collidepoint(hot))
        D.icon(s, "left", self.r_mminus.inflate(-self.r_mminus.w // 3, -self.r_mminus.h // 3),
               ui.TEXT if self.mines > 1 else ui.TEXT_FAINT)
        mx = ML.max_mines(self.grid)
        self._btn(s, self.r_mplus, "", on=self.mines < mx, hot=self.r_mplus.collidepoint(hot))
        D.icon(s, "right", self.r_mplus.inflate(-self.r_mplus.w // 3, -self.r_mplus.h // 3),
               ui.TEXT if self.mines < mx else ui.TEXT_FAINT)
        pygame.draw.rect(s, (16, 12, 26), self.r_mval, border_radius=max(4, rh // 4))
        pygame.draw.rect(s, ui.mix(ui.RED, ui.BORDER, 0.4), self.r_mval, 1, border_radius=max(4, rh // 4))
        val = self.f_btn.render(str(self.mines), True, ui.RED)
        s.blit(val, val.get_rect(center=self.r_mval.center))
        for i, r in enumerate(self.r_presets):
            v = ML.MINE_PRESETS[self.grid][i]
            self._btn(s, r, str(v), active=v == self.mines, hot=r.collidepoint(hot), small=True)

    def _info_rows(self):
        """Zeilen der Gewinn-Info: [(Beschriftung, Wert, Farbe)], Index der Zeile
        mit dem Chancen-Balken (oder None) und die Chance in Zehntel-Prozent."""
        g, m = self.grid, self.mines
        if self.state in (BOOM, CASHED):
            o = self.fairlog[-1].get("o") if self.fairlog else None
            bet = o.get("bet", 0) if isinstance(o, dict) else 0
            paid = self.win if self.state == CASHED else 0
            rows = [(t("cas.bet"), self._group(bet), ui.TEXT),
                    (t("lmi.found"), "%d / %d" % (self._last_k(), ML.gems(g, m)), ui.TEXT),
                    (t("chip.st_paid"), self._group(paid), CD.COL_GOLD if paid else ui.RED)]
            return rows, None, 0
        k = self._k() if self.state == PLAY else 0
        bet = self._round_bet()
        nxt = ML.next_mult100(g, m, k)
        p10 = ML.safe_percent(g, m, k)
        rows = [(t("lmi.next_mult"), self._fmt(nxt) if nxt is not None else "–", ui.GREEN),
                (t("lmi.safe"), self._fmt_pct(p10) + " %", ui.TEXT),
                (t("lmi.next_win"), self._group(self._win_at(bet, k + 1)) if nxt is not None else "–",
                 CD.COL_GOLD)]
        if self.state == PLAY:
            rows.append((t("lmi.found"), "%d / %d" % (k, ML.gems(g, m)), ui.TEXT))
        return rows, 1, p10

    def _draw_info(self, s, y):
        """Gewinn-Info (Ruhe: erstes Feld, Runde: nächstes Feld, Ergebnis: Bilanz).
        Die Zeilen verteilen sich auf den freien Platz bis zur Stufen-Leiste."""
        x0, w = self.panel_inner
        rows, bar_row, p10 = self._info_rows()
        bh = max(4, int(6 * self.k))
        bar_total = bh + max(2, int(4 * self.k)) if bar_row is not None else 0
        avail = self.ladder_lab_y - max(4, int(6 * self.k)) - y
        min_lh = self.f_tiny.get_height() + 2
        max_lh = int(self.f_btn.get_height() * 1.75)
        while rows and (avail - bar_total) // len(rows) < min_lh:
            rows = rows[:-1]
            if bar_row is not None and bar_row >= len(rows):
                bar_row, bar_total = None, 0
        if not rows:
            return
        lh = min(max_lh, (avail - bar_total) // len(rows))
        for j, (lab, val, col) in enumerate(rows):
            vimg = self._fit(val, (self.f_btn, self.f_tiny), w // 2, col)
            limg = self._fit(lab, (self.f_small, self.f_tiny, self.f_micro), w - vimg.get_width() - 8,
                             ui.TEXT_DIM)
            s.blit(limg, limg.get_rect(midleft=(x0, y + lh // 2)))
            s.blit(vimg, vimg.get_rect(midright=(x0 + w, y + lh // 2)))
            y += lh
            if j == bar_row:
                # Balken: grün = sicher, rot = Mine
                bar = pygame.Rect(x0, y - max(1, lh // 8), w, bh)
                pygame.draw.rect(s, (110, 30, 40), bar, border_radius=bh // 2)
                gw = int(w * p10 / 1000)
                if gw > 0:
                    pygame.draw.rect(s, (40, 170, 120), (bar.x, bar.y, gw, bh), border_radius=bh // 2)
                y += bar_total
        # Noch im Feld: verbleibende Edelsteine und Minen als Bildchen
        ih = max(18, int(30 * self.k))
        room = self.ladder_lab_y - max(4, int(6 * self.k)) - y
        if self.state == PLAY and room >= ih + max(8, int(12 * self.k)):
            left = ML.gems(self.grid, self.mines) - self._k()
            self._draw_counts(s, x0, y + room // 2, left, self.mines, self.f_big, ih, w)

    def _last_k(self):
        if self.fairlog:
            o = self.fairlog[-1].get("o")
            if isinstance(o, dict) and isinstance(o.get("open"), list):
                n = len(o["open"])
                return n - 1 if o.get("m") == 0 else n
        return 0

    def _draw_ladder(self, s):
        """Leiste der nächsten 5 Stufen (in Ruhe: die ersten 5)."""
        x0, w = self.panel_inner
        k = self._k() if self.state == PLAY else 0
        steps = ML.ladder(self.grid, self.mines, k, 5)
        img = self._fit(t("lmi.ladder"), (self.f_tiny, self.f_micro), w, ui.TEXT_DIM)
        s.blit(img, (x0, self.ladder_lab_y))
        for i, r in enumerate(self.r_pills):
            if i >= len(steps):
                pygame.draw.rect(s, (20, 18, 30), r, 1, border_radius=r.h // 2)
                continue
            j, m100 = steps[i]
            nxt = i == 0 and self.state == PLAY
            bg = CD.COL_GOLD if nxt else (26, 24, 40)
            fg = (30, 24, 10) if nxt else (232, 232, 240)
            pygame.draw.rect(s, bg, r, border_radius=r.h // 2)
            if not nxt:
                pygame.draw.rect(s, ui.mix(self.accent, ui.BORDER, 0.6), r, 1, border_radius=r.h // 2)
            txt = self._fmt_short(m100)
            img = self._fit(txt, (self.f_tiny, self.f_micro), r.w - 4, fg)
            s.blit(img, img.get_rect(center=r.center))

    def _draw_hint(self, s):
        x0, w = self.panel_inner
        if self.auto is not None:
            text = t("chip.auto_left", n=self.auto["left"])
        elif self.state == PLAY:
            text = t("lmi.hint_play")
        else:
            text = t("lmi.hint_idle")
        lines = self._wrap(text, self.f_micro, w)
        y = self.hint_y
        for line in lines[:2]:
            img = self.f_micro.render(line, True, ui.TEXT_FAINT)
            s.blit(img, (x0, y))
            y += self.f_micro.get_height() + 1

    def _wrap(self, text, font, maxw):
        words = text.split(" ")
        lines, cur = [], ""
        for wd in words:
            test = (cur + " " + wd) if cur else wd
            if font.size(test)[0] <= maxw or not cur:
                cur = test
            else:
                lines.append(cur)
                cur = wd
        if cur:
            lines.append(cur)
        return lines

    # ----- Auto-Fenster (über der Seitenleiste) -----------------------------------
    def _draw_auto(self, s):
        p = self.panel_r
        ui.draw_panel(s, p, accent_top=self.accent)
        x0, w = self.panel_inner
        hot = self.mouse
        title = self._fit(t("chip.auto_title"), (self.f_big, self.f_btn), self.r_auto_close.x - x0 - 4, ui.TEXT)
        s.blit(title, title.get_rect(midleft=(x0, self.r_auto_close.centery)))
        self._btn(s, self.r_auto_close, "", hot=self.r_auto_close.collidepoint(hot))
        D.icon(s, "close", self.r_auto_close.inflate(-self.r_auto_close.w // 4, -self.r_auto_close.h // 4), ui.TEXT)
        labels = (t("chip.auto_rounds"), t("chip.auto_stop_win"), t("chip.auto_stop_loss"))
        values = (self.auto_rounds, self.auto_win, self.auto_loss)
        for row, (ly, rects) in enumerate(self.auto_rows):
            img = self._fit(labels[row], (self.f_tiny, self.f_micro), w, ui.TEXT_DIM)
            s.blit(img, (x0, ly))
            opts = ROUND_OPTS if row == 0 else LIMIT_OPTS
            for j, r in enumerate(rects):
                v = opts[j]
                lab = str(v) if row == 0 else (t("chip.auto_off") if v == 0 else CD.fmt_amount(v))
                self._btn(s, r, lab, active=v == values[row], hot=r.collidepoint(hot), small=True)
        # Markierungen + Gewinn bei Erfolg
        y = self.auto_info_y
        n = len(self._marks())
        g = ML.gems(self.grid, self.mines)
        img = self._fit(t("lmi.marked", n=n), (self.f_btn, self.f_tiny), w, ui.TEXT if n else ui.TEXT_DIM)
        s.blit(img, (x0, y))
        y += img.get_height() + 1
        if 0 < n <= g:
            info = t("lmi.marked_info", x=self._fmt(ML.mult100(self.grid, self.mines, n))[1:],
                     p=self._fmt_pct(ML.all_safe_percent(self.grid, self.mines, n)))
            col = CD.COL_GOLD
        elif n > g:
            info = t("lmi.auto_too_many", n=g)
            col = ui.RED
        else:
            info = t("lmi.hint_auto")
            col = ui.TEXT_DIM
        room = self.r_auto_start.y - y - 4
        for line in self._wrap(info, self.f_tiny, w)[:max(1, room // max(1, self.f_tiny.get_height()))]:
            img = self.f_tiny.render(line, True, col)
            s.blit(img, (x0, y))
            y += self.f_tiny.get_height()
        if 0 < n <= g and y + self.f_micro.get_height() < self.r_auto_start.y - 4:
            for line in self._wrap(t("lmi.hint_auto"), self.f_micro, w):
                if y + self.f_micro.get_height() >= self.r_auto_start.y - 4:
                    break
                img = self.f_micro.render(line, True, ui.TEXT_FAINT)
                s.blit(img, (x0, y))
                y += self.f_micro.get_height()
        # freier Platz: das Maskottchen schaut zu
        free = self.r_auto_start.y - y - 2 * self.ppad
        ms = self.r_mascot.w
        if free >= ms + 4:
            mw = ms + int(self.cart_w * 1.0)
            self._draw_mascot(s, pygame.Rect(x0 + (w - mw) // 2, y + self.ppad + (free - ms) // 2, ms, ms))
        self._btn(s, self.r_auto_clear, t("lmi.clear_marks"), on=n > 0, hot=self.r_auto_clear.collidepoint(hot))
        self._btn(s, self.r_auto_start, t("chip.auto_start"), primary=True, on=0 < n <= g,
                  hot=self.r_auto_start.collidepoint(hot))

    # ----- Kopfzeile, Werkzeuge, Verlauf ---------------------------------------------
    def _draw_hud(self, s):
        W = self.width
        s.blit(self._hud_bg, (0, 0))
        pygame.draw.line(s, ui.mix(CD.COL_GOLD, self.accent, 0.35), (0, self.hud_h - 1), (W, self.hud_h - 1), 1)
        cy = self.hud_h // 2
        pad = self.margin
        coin_r = max(6, int(self.hud_h * 0.26))
        coin = CD.symbol_surface("coin", coin_r * 2 + 4)
        s.blit(coin, coin.get_rect(midleft=(pad - 2, cy)))
        left = self.f_big.render(f"{t('cas.chips')}: {self._group(int(round(self.shown_chips)))}", True, ui.GOLD)
        s.blit(left, left.get_rect(midleft=(pad + coin_r * 2 + 6, cy)))
        right_txt = f"{t('cas.record')}: {self._group(self.score)}   ·   {t('cas.credits', n=lamabank.refills())}"
        right = self.f_small.render(right_txt, True, ui.TEXT_DIM)
        if right.get_width() > W * 0.42:
            right = self.f_tiny.render(right_txt, True, ui.TEXT_DIM)
        rr = right.get_rect(midright=(W - pad, cy))
        s.blit(right, rr)
        if self.round is None:
            return
        bet = self.round["bet"]
        k = self._k()
        mid_txt = f"{t('cas.bet')}: {self._group(bet)}"
        if k > 0:
            m100 = ML.mult100(self.grid, self.mines, k)
            mid_txt += f"   ·   {self._fmt(m100)} = {self._group(self._win_at(bet, k))}"
        x0 = pad + coin_r * 2 + 6 + left.get_width() + pad
        x1 = rr.left - pad
        mid = self.f_small.render(mid_txt, True, ui.TEXT)
        if mid.get_width() > x1 - x0:
            mid = self.f_tiny.render(mid_txt, True, ui.TEXT)
        if mid.get_width() <= x1 - x0:
            s.blit(mid, mid.get_rect(center=((x0 + x1) // 2, cy)))

    def _tool_active(self, name):
        if name == "turbo":
            return self.turbo
        if name == "auto":
            return self.window == "auto" or self.auto is not None
        return self.window == name

    def _draw_tools(self, s):
        hot = self.mouse
        for name, r in self.r_tools:
            active = self._tool_active(name)
            h = r.collidepoint(hot)
            rad = max(5, r.h // 4)
            fill = ui.BTN_SEL if (active or h) else ui.BTN
            pygame.draw.rect(s, fill, r, border_radius=rad)
            edge = self.accent if active else (ui.BORDER_LIGHT if h else ui.BORDER)
            if name == "auto" and self.auto is not None:
                edge = ui.mix(self.accent, (255, 255, 255), 0.5 * (0.5 + 0.5 * math.sin(self.time * 5)))
            pygame.draw.rect(s, edge, r, 2 if active else 1, border_radius=rad)
            col = self.accent if active else ui.TEXT
            ir = pygame.Rect(r.x, r.y, r.h, r.h).inflate(-r.h // 3, -r.h // 3)
            D.icon(s, name, ir, col)
            if self.tool_labels:
                img = self.f_tiny.render(t(TOOL_LABEL[name]), True, ui.TEXT if (active or h) else ui.TEXT_DIM)
                s.blit(img, img.get_rect(midleft=(r.x + r.h - max(2, r.h // 8), r.centery)))

    def _draw_tooltip(self, s):
        for name, r in self.r_tools:
            if r.collidepoint(self.mouse):
                txt = "%s (%s)" % (t(TOOL_LABEL[name]), TOOL_KEY[name])
                img = self.f_tiny.render(txt, True, ui.TEXT)
                box = img.get_rect().inflate(12, 6)
                box.topleft = (r.x, r.bottom + 4)
                pygame.draw.rect(s, (12, 10, 22), box, border_radius=box.h // 2)
                pygame.draw.rect(s, ui.BORDER_LIGHT, box, 1, border_radius=box.h // 2)
                s.blit(img, img.get_rect(center=box.center))

    def _draw_history(self, s):
        """Letzte Runden als Schilder oben rechts (neueste rechts)."""
        x = self.width - self.margin
        y = self.r_tools[0][1].y
        h = self.tool_h
        limit = self.tools_right + self.margin
        for m100 in reversed(self.hist):
            txt = self._fmt_short(m100) if m100 > 0 else "×0"
            img = self.f_tiny.render(txt, True, (240, 240, 245))
            box = img.get_rect()
            box.w += int(12 * self.k) + 4
            box.h = int(h * 0.78)
            box.topright = (x, y + (h - box.h) // 2)
            if box.left < limit:
                break
            if m100 >= 200:
                col = (34, 132, 100)
            elif m100 > 100:
                col = (62, 96, 82)
            elif m100 > 0:
                col = (70, 72, 92)
            else:
                col = (128, 36, 46)
            pygame.draw.rect(s, col, box, border_radius=box.h // 2)
            s.blit(img, img.get_rect(center=box.center))
            x = box.left - 4

    # ----- Bedienleiste -----------------------------------------------------------
    def _draw_strip(self, s):
        s.blit(self._strip_bg, self.strip.topleft)
        pygame.draw.line(s, ui.mix(CD.COL_GOLD, self.accent, 0.35), self.strip.topleft, self.strip.topright, 2)
        idle = self.state == IDLE and self.auto is None
        bet = self._round_bet()
        hot = self.mouse
        self._btn(s, self.r_minus, "–", on=idle and self.bet > ML.BETS[0], hot=self.r_minus.collidepoint(hot))
        self._btn(s, self.r_plus, "+", on=idle and self.bet < ML.BETS[-1], hot=self.r_plus.collidepoint(hot))
        pygame.draw.rect(s, (16, 12, 26), self.r_bet, border_radius=max(6, self.r_bet.h // 4))
        pygame.draw.rect(s, ui.BORDER, self.r_bet, 1, border_radius=max(6, self.r_bet.h // 4))
        lbl = self.f_tiny.render(t("cas.bet"), True, ui.TEXT_DIM)
        s.blit(lbl, lbl.get_rect(midtop=(self.r_bet.centerx, self.r_bet.y + 2)))
        col = ui.RED if (idle and bet > lamabank.balance()) else ui.GOLD
        val = self._fit(self._group(bet), (self.f_btn, self.f_tiny), self.r_bet.w - 4, col)
        s.blit(val, val.get_rect(midbottom=(self.r_bet.centerx, self.r_bet.bottom - 3)))
        play = self.state == PLAY
        if self.auto is not None:
            # Mitte: Fortschritt, rechts: STOPP
            txt = t("chip.auto_left", n=self.auto["left"])
            img = self._fit(txt, (self.f_btn, self.f_tiny), self.r_cash.x - self.r_plus.right - 2 * self.margin,
                            self.accent)
            s.blit(img, img.get_rect(center=((self.r_plus.right + self.r_cash.x) // 2, self.r_go.centery)))
            self._btn(s, self.r_go, t("chip.auto_stop"), primary=True, hot=self.r_go.collidepoint(hot), danger=True)
        else:
            ready = idle or (self.state in (BOOM, CASHED) and self._result_visible() and self.res_t >= READ_T)
            self._btn(s, self.r_go, t("lmi.play"), primary=True, on=ready and not self.broke and self.window is None,
                      hot=self.r_go.collidepoint(hot))
            if play:
                on = self.window is None
                self._btn(s, self.r_random, t("lmi.random") if self.random_label else "", on=on,
                          hot=self.r_random.collidepoint(hot), icon="dice")
        cash = t("lmi.cashout")
        can = play and self._k() > 0 and self.auto is None
        if can:
            cash += " " + self._group(self._win_at(self.round["bet"], self._k()))
        self._btn(s, self.r_cash, cash, on=can, hot=self.r_cash.collidepoint(hot), active=can)

    def _btn(self, s, rect, label, primary=False, on=True, hot=False, active=False, small=False,
             icon=None, danger=False):
        r = pygame.Rect(rect)
        rad = max(5, r.h // 4)
        if primary and on:
            base = ui.mix(ui.RED if danger else self.accent, (0, 0, 0), 0.30 if not danger else 0.1)
            fill = ui.mix(base, (255, 255, 255), 0.12 if hot else 0.0)
            pygame.draw.rect(s, ui.mix(base, (0, 0, 0), 0.4), r.move(0, 2), border_radius=rad)
            pygame.draw.rect(s, fill, r, border_radius=rad)
            pulse = 0.5 + 0.5 * math.sin(self.time * 2.4)
            pygame.draw.rect(s, ui.mix(CD.COL_GOLD, (255, 255, 255), 0.35 * pulse), r, 2, border_radius=rad)
            col = (255, 255, 255)
        else:
            fill = ui.BTN_SEL if (on and (hot or active)) else ui.BTN
            pygame.draw.rect(s, fill, r, border_radius=rad)
            edge = CD.COL_GOLD if active else (ui.BORDER_LIGHT if on else ui.BORDER)
            pygame.draw.rect(s, edge, r, 2 if active else 1, border_radius=rad)
            col = ui.TEXT if on else ui.TEXT_FAINT
        tx = r.centerx
        if icon is not None:
            isz = int(r.h * 0.62)
            if label:
                lw = self.f_btn.size(label)[0]
                ix = r.centerx - (isz + 6 + lw) // 2
                D.icon(s, icon, pygame.Rect(ix, r.centery - isz // 2, isz, isz), col)
                tx = ix + isz + 6 + lw // 2
            else:
                D.icon(s, icon, pygame.Rect(r.centerx - isz // 2, r.centery - isz // 2, isz, isz), col)
        if label:
            fonts = (self.f_tiny, self.f_micro) if small else (self.f_btn, self.f_tiny, self.f_micro)
            img = self._fit(label, fonts, r.w - 6, col)
            s.blit(img, img.get_rect(center=(tx, r.centery)))

    # ----- Effekte, Ergebnis, Hinweise ---------------------------------------------
    def _draw_fx(self, s):
        for c in self.coins_fx:
            w = max(2, int(c["r"] * 2 * abs(math.cos(c["spin"]))))
            h = int(c["r"] * 2)
            rect = pygame.Rect(0, 0, w, h)
            rect.center = (int(c["x"]), int(c["y"]))
            pygame.draw.ellipse(s, CD.COL_GOLD_D, rect.move(1, 1))
            pygame.draw.ellipse(s, (246, 196, 60), rect)
            if w > 6:
                pygame.draw.ellipse(s, (255, 236, 150), rect.inflate(-w // 2, -h // 2))
        for p in self.popups:
            f = p["t"] / p["dur"]
            img = self.f_big.render(p["text"], True, p["color"])
            if f > 0.6:
                img.set_alpha(int(255 * (1 - (f - 0.6) / 0.4)))
            y = p["y"] - 40 * self.k * (1 - (1 - f) * (1 - f))
            s.blit(img, img.get_rect(center=(int(p["x"]), int(y))))

    def _draw_result(self, s):
        title, sub, col = self.result
        br = self.board_r
        k = self.k
        maxw = br.w - int(24 * k)
        fonts = (self.f_huge, self.f_mult, self.f_big, self.f_btn)
        if self.f_huge.size(title)[0] > br.w * 0.62:
            fonts = fonts[1:]
        timg = self._fit(title, fonts, maxw, col)
        simg = self._fit(sub, (self.f_small, self.f_tiny, self.f_micro), maxw, ui.TEXT)
        pw = min(br.w - 8, max(timg.get_width(), simg.get_width()) + int(40 * k))
        ph = timg.get_height() + simg.get_height() + int(30 * k)
        panel = pygame.Rect(0, 0, pw, ph)
        panel.center = br.center
        f = min(1.0, self.res_t / 0.18)
        if f < 1.0:
            panel = panel.inflate(-int(pw * 0.15 * (1 - f)), -int(ph * 0.15 * (1 - f)))
        ui.draw_panel(s, panel, accent_top=col)
        s.blit(timg, timg.get_rect(midtop=(panel.centerx, panel.y + int(10 * k))))
        s.blit(simg, simg.get_rect(midbottom=(panel.centerx, panel.bottom - int(10 * k))))

    def _draw_banner(self, s):
        text, left = self.banner
        br = self.board_r
        pulse = 0.5 + 0.5 * math.sin(self.time * 8)
        col = ui.mix(CD.COL_GOLD, (255, 255, 255), 0.4 * pulse)
        fonts = (self.f_huge, self.f_mult, self.f_big)
        font = fonts[-1]
        for f in fonts:
            if f.size(text)[0] <= br.w - 16:
                font = f
                break
        img = self._fit(text, (font,), br.w - 16, col)
        sh = self._fit(text, (font,), br.w - 16, (40, 20, 0))
        y = br.y + int(br.h * 0.2)
        if left < 0.4:
            img.set_alpha(int(255 * left / 0.4))
            sh.set_alpha(int(255 * left / 0.4))
        s.blit(sh, sh.get_rect(center=(br.centerx + 3, y + 3)))
        s.blit(img, img.get_rect(center=(br.centerx, y)))

    def _draw_toast(self, s):
        text, left = self.toast
        img = self._fit(text, (self.f_small, self.f_tiny), self.width - 40, ui.TEXT)
        box = img.get_rect()
        box.inflate_ip(24, 12)
        box.midtop = (self.board_r.centerx, self.board_r.y + self.margin)
        if box.left < 4:
            box.x = 4
        panel = pygame.Surface(box.size, pygame.SRCALPHA)
        pygame.draw.rect(panel, (12, 8, 20, 225), panel.get_rect(), border_radius=box.h // 2)
        pygame.draw.rect(panel, (*ui.mix(CD.COL_GOLD, self.accent, 0.4), 255), panel.get_rect(), 1,
                         border_radius=box.h // 2)
        if left < 0.3:
            panel.set_alpha(int(255 * left / 0.3))
            img.set_alpha(int(255 * left / 0.3))
        s.blit(panel, box)
        s.blit(img, img.get_rect(center=box.center))

    def _draw_broke(self, s):
        s.blit(self._overlay, (0, 0))
        cx, cy = self.width // 2, self.height // 2
        pw = min(self.width - 40, int(480 * max(0.8, self.k)))
        ph = int(200 * max(0.75, self.k))
        panel = pygame.Rect(cx - pw // 2, cy - ph // 2, pw, ph)
        ui.draw_panel(s, panel, accent_top=ui.RED)
        head = self._fit(t("cas.broke"), (self.f_huge, self.f_big), pw - 16, ui.RED)
        s.blit(head, head.get_rect(center=(cx, panel.y + ph * 0.28)))
        for i, (txt, col) in enumerate(((t("cas.broke_sub"), ui.TEXT_DIM),
                                        (t("cas.broke_restart", n=lamabank.START_CHIPS), ui.TEXT))):
            img = self._fit(txt, (self.f_small, self.f_tiny, self.f_micro), pw - 16, col)
            s.blit(img, img.get_rect(center=(cx, panel.y + ph * (0.58 + 0.2 * i))))

    # ----- Statistik- und Fairness-Fenster ---------------------------------------------
    def _window_rect(self):
        W, H, k, m = self.width, self.height, self.k, self.margin
        if self.window == "fair":
            w, h = min(W - 2 * m, int(720 * k)), min(H - 2 * m, int(540 * k))
        else:
            w, h = min(W - 2 * m, int(640 * k)), min(H - 2 * m, int(470 * k))
        rect = pygame.Rect(0, 0, w, h)
        rect.center = (W // 2, H // 2)
        cb = max(20, int(28 * k))
        close = pygame.Rect(rect.right - self.ppad - cb, rect.y + self.ppad, cb, cb)
        return rect, close

    def _draw_window(self, s):
        s.blit(self._overlay, (0, 0))
        rect, close = self._window_rect()
        ui.draw_panel(s, rect, accent_top=self.accent)
        pad = self.ppad
        icon = pygame.Rect(rect.x + pad, close.y, close.h, close.h)
        D.icon(s, self.window, icon.inflate(-icon.w // 6, -icon.h // 6), self.accent)
        title = t("chip.stats") if self.window == "stats" else t("chip.fair_title")
        title += "  ·  Lama Mines"
        img = self._fit(title, (self.f_big, self.f_btn), close.x - icon.right - 2 * pad, ui.TEXT)
        s.blit(img, img.get_rect(midleft=(icon.right + pad // 2, close.centery)))
        self._btn(s, close, "", hot=close.collidepoint(self.mouse))
        D.icon(s, "close", close.inflate(-close.w // 4, -close.h // 4), ui.TEXT)
        body = pygame.Rect(rect.x + pad, close.bottom + pad, rect.w - 2 * pad, rect.bottom - close.bottom - 2 * pad)
        if self.window == "stats":
            self._draw_stats(s, body)
        else:
            self._draw_fair(s, body)

    def _draw_stats(self, s, body):
        st = self.stats
        k = self.k
        sep = self._sep()
        rtp = "–"
        if st["wagered"] > 0:
            p10 = (st["paid"] * 2000 + st["wagered"]) // (st["wagered"] * 2)
            rtp = ML.fmt_percent(p10, sep) + " %"
        net = st["net"]
        tiles = [
            (t("chip.st_rounds"), self._group(st["rounds"]), ui.TEXT),
            (t("chip.st_wins"), self._group(st["wins"]), ui.GREEN),
            (t("chip.st_losses"), self._group(st["losses"]), ui.RED),
            (t("chip.st_rtp"), rtp, ui.TEXT),
            (t("chip.st_wagered"), self._group(st["wagered"]), ui.TEXT),
            (t("chip.st_paid"), self._group(st["paid"]), ui.TEXT),
            (t("chip.st_net"), ("+" if net > 0 else "") + self._group(net), ui.GREEN if net >= 0 else ui.RED),
            (t("chip.st_best_win"), ("+" if st["best_win"] > 0 else "") + self._group(st["best_win"]), CD.COL_GOLD),
            (t("chip.st_best_mult"), self._fmt(st["best_mult"]) if st["best_mult"] > 0 else "–", CD.COL_GOLD),
            (t("lmi.st_gems"), self._group(self._xint("gems")), self.accent),
            (t("lmi.st_booms"), self._group(self._xint("booms")), ui.RED),
            (t("lmi.st_clears"), self._group(self._xint("clears")), CD.COL_GOLD),
        ]
        cols = 4
        gap = max(4, int(8 * k))
        tw = (body.w - (cols - 1) * gap) // cols
        th = self.f_tiny.get_height() + self.f_big.get_height() + max(8, int(14 * k))
        for i, (lab, val, col) in enumerate(tiles):
            r = pygame.Rect(body.x + (i % cols) * (tw + gap), body.y + (i // cols) * (th + gap), tw, th)
            pygame.draw.rect(s, ui.mix(ui.PANEL, (0, 0, 0), 0.25), r, border_radius=max(4, int(8 * k)))
            pygame.draw.rect(s, ui.BORDER, r, 1, border_radius=max(4, int(8 * k)))
            limg = self._fit(lab, (self.f_tiny, self.f_micro), tw - 8, ui.TEXT_DIM)
            s.blit(limg, (r.x + 5, r.y + max(3, int(5 * k))))
            vimg = self._fit(val, (self.f_big, self.f_btn, self.f_tiny), tw - 8, col)
            s.blit(vimg, vimg.get_rect(bottomleft=(r.x + 5, r.bottom - max(3, int(5 * k)))))
        y = body.y + 3 * (th + gap) + gap
        curve = st["curve"]
        lab = t("chip.st_curve", n=len(curve)) if curve else t("chip.st_empty")
        img = self._fit(lab, (self.f_tiny, self.f_micro), body.w, ui.TEXT_DIM)
        s.blit(img, (body.x, y))
        y += img.get_height() + 4 + charts._fnt(10).get_height()     # Platz für die obere Achsen-Zahl
        chart = pygame.Rect(body.x, y, body.w, body.bottom - y)
        if len(curve) >= 2 and chart.h > 20:
            vals = [0] + list(curve) if len(curve) < chipstats.CURVE else list(curve)
            charts.line(s, chart, vals, color=self.accent, fmt=lambda v: self._group(int(v)))
            # Null-Linie
            lo, hi = min(vals), max(vals)
            if lo < 0 < hi:
                zy = int(chart.bottom - (0 - lo) / (hi - lo) * chart.h)
                pygame.draw.line(s, ui.mix(ui.RED, ui.PANEL, 0.4), (chart.x, zy), (chart.right, zy), 1)
        elif chart.h > 20:
            img = D.miner_llama(max(16, min(chart.h - 8, int(90 * k))), "stand", self.accent)
            s.blit(img, img.get_rect(center=chart.center))

    def _draw_fair(self, s, body):
        k = self.k
        gap = max(4, int(7 * k))
        y = body.y
        for line in self._wrap(t("chip.fair_info"), self.f_micro, body.w)[:3]:
            img = self.f_micro.render(line, True, ui.TEXT_DIM)
            s.blit(img, (body.x, y))
            y += self.f_micro.get_height()
        y += gap
        # Nächster Prüfcode (+ laufende Runde)
        rows = []
        if self.round is not None:
            rows.append((t("lmi.fair_current"), self.round["h"], ui.TEXT))
        rows.append((t("chip.fair_next"), fairness.commit(self.fair["server"]), self.accent))
        for lab, hx, col in rows:
            img = self._fit(lab, (self.f_tiny, self.f_micro), body.w, ui.TEXT_DIM)
            s.blit(img, (body.x, y))
            y += img.get_height()
            y = self._draw_hex(s, hx, body.x, y, body.w, col)
            y += 2
        # Client-Seed + Knopf
        bh = max(22, int(28 * k))
        lab = f"{t('chip.fair_client')}: {self.fair['client']}   ·   {t('chip.fair_nonce')} {self.fair['nonce']}"
        bw = self.f_btn.size(t("chip.fair_new_client"))[0] + int(20 * k)
        bw = min(bw, body.w // 2)
        self.r_fair_new = pygame.Rect(body.right - bw, y, bw, bh)
        img = self._fit(lab, (self.f_tiny, self.f_micro), body.w - bw - gap, ui.TEXT)
        s.blit(img, img.get_rect(midleft=(body.x, y + bh // 2)))
        self._btn(s, self.r_fair_new, t("chip.fair_new_client"), on=self.round is None,
                  hot=self.r_fair_new.collidepoint(self.mouse))
        y += bh + gap
        pygame.draw.line(s, ui.BORDER, (body.x, y), (body.right, y), 1)
        y += gap
        # Liste links, Einzelheiten rechts
        area = pygame.Rect(body.x, y, body.w, body.bottom - y - self.f_micro.get_height() - 2)
        note = self._fit(t("chip.fair_note"), (self.f_micro,), body.w, ui.TEXT_FAINT)
        s.blit(note, (body.x, body.bottom - note.get_height()))
        self.r_fair_rows = []
        self.r_fair_verify = pygame.Rect(0, 0, 0, 0)
        if not self.fairlog:
            img = self._fit(t("chip.fair_empty"), (self.f_small, self.f_tiny), area.w, ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=area.center))
            return
        lw = max(int(area.w * 0.40), min(area.w // 2, int(220 * k)))
        lst = pygame.Rect(area.x, area.y, lw, area.h)
        det = pygame.Rect(lst.right + gap, area.y, area.w - lw - gap, area.h)
        rh = max(self.f_tiny.get_height() + 6, int(24 * k))
        nvis = max(1, lst.h // rh)
        order = list(range(len(self.fairlog) - 1, -1, -1))        # neueste oben
        self.fair_scroll = max(0, min(self.fair_scroll, len(order) - nvis))
        for row, idx in enumerate(order[self.fair_scroll:self.fair_scroll + nvis]):
            e = self.fairlog[idx]
            r = pygame.Rect(lst.x, lst.y + row * rh, lst.w, rh - 2)
            sel = idx == self.fair_sel
            pygame.draw.rect(s, ui.BTN_SEL if sel or r.collidepoint(self.mouse) else ui.mix(ui.PANEL, (0, 0, 0), 0.2),
                             r, border_radius=max(3, rh // 4))
            if sel:
                pygame.draw.rect(s, self.accent, r, 1, border_radius=max(3, rh // 4))
            o = e.get("o") if isinstance(e.get("o"), dict) else {}
            m = o.get("m")
            res = (self._fmt_short(m) if m else "×0") if isinstance(m, int) and not isinstance(m, bool) else "?"
            num = self.f_tiny.render("#%d" % e["n"], True, ui.TEXT_DIM)
            s.blit(num, num.get_rect(midleft=(r.x + 5, r.centery)))
            rimg = self.f_tiny.render(res, True, ui.GREEN if m else ui.RED)
            s.blit(rimg, rimg.get_rect(midright=(r.right - 5, r.centery)))
            hx0 = r.x + 5 + max(num.get_width(), self.f_tiny.size("#0000")[0]) + 6
            hx1 = r.right - 5 - max(rimg.get_width(), self.f_tiny.size("×00,00")[0]) - 6
            if hx1 - hx0 > 20:
                himg = self._fit(fairness.short(e["h"], 10), (self.f_mono,), hx1 - hx0, ui.TEXT_FAINT)
                s.blit(himg, himg.get_rect(midleft=(hx0, r.centery)))
            chk = self.fair_checks.get(e["n"])
            if chk is not None:
                ir = pygame.Rect(r.right - rimg.get_width() - 10 - rh // 2, r.y + rh // 4, rh // 2, rh // 2)
                D.icon(s, "check" if chk[0] else "cross", ir, ui.GREEN if chk[0] else ui.RED)
            self.r_fair_rows.append((idx, r))
        if len(order) > nvis:
            # Bildlauf-Leiste
            sb = pygame.Rect(lst.right - 3, lst.y, 3, lst.h)
            pygame.draw.rect(s, ui.BORDER, sb)
            hh = max(8, sb.h * nvis // len(order))
            hy = sb.y + (sb.h - hh) * self.fair_scroll // max(1, len(order) - nvis)
            pygame.draw.rect(s, self.accent, (sb.x, hy, 3, hh))
        if self.fair_sel is None or not 0 <= self.fair_sel < len(self.fairlog):
            return
        self._draw_fair_detail(s, det, self.fairlog[self.fair_sel])

    def _draw_hex(self, s, hx, x, y, w, col):
        """Hex-Text in Monospace, bei Bedarf auf mehrere Zeilen umbrochen."""
        cw = max(1, self.f_mono.size("0")[0])
        per = max(8, w // cw)
        for i in range(0, len(hx), per):
            img = self.f_mono.render(hx[i:i + per], True, col)
            s.blit(img, (x, y))
            y += self.f_mono.get_height()
        return y

    def _draw_fair_detail(self, s, det, e):
        k = self.k
        x, y, w = det.x, det.y, det.w
        o = e.get("o") if isinstance(e.get("o"), dict) else {}
        head = "#%d" % e["n"]
        if o.get("grid") in ML.GRIDS and isinstance(o.get("mines"), int):
            head += "  ·  " + t("lmi.fair_round", g=o["grid"], m=o["mines"])
        img = self._fit(head, (self.f_btn, self.f_tiny), w, ui.TEXT)
        s.blit(img, (x, y))
        y += img.get_height() + 2
        for lab, val in ((t("chip.fair_hash"), e["h"]), (t("chip.fair_server"), e["s"])):
            img = self._fit(lab, (self.f_micro,), w, ui.TEXT_DIM)
            s.blit(img, (x, y))
            y += img.get_height()
            y = self._draw_hex(s, val, x, y, w, ui.TEXT)
        img = self._fit(f"{t('chip.fair_client')}: {e['c']}", (self.f_tiny, self.f_micro), w, ui.TEXT)
        s.blit(img, (x, y + 1))
        y += img.get_height() + 4
        bh = max(22, int(28 * k))
        bw = min(w, self.f_btn.size(t("chip.fair_verify"))[0] + int(24 * k))
        self.r_fair_verify = pygame.Rect(x, y, bw, bh)
        self._btn(s, self.r_fair_verify, t("chip.fair_verify"), primary=True,
                  hot=self.r_fair_verify.collidepoint(self.mouse))
        chk = self.fair_checks.get(e["n"])
        if chk is not None:
            ok, field = chk
            ir = pygame.Rect(self.r_fair_verify.right + 6, y + bh // 6, bh * 2 // 3, bh * 2 // 3)
            D.icon(s, "check" if ok else "cross", ir, ui.GREEN if ok else ui.RED)
            img = self._fit(t("chip.fair_ok") if ok else t("chip.fair_bad"), (self.f_btn, self.f_tiny),
                            max(10, det.right - ir.right - 6), ui.GREEN if ok else ui.RED)
            s.blit(img, img.get_rect(midleft=(ir.right + 4, y + bh // 2)))
            y += bh + 6
            if field is not None:
                self._draw_mini(s, pygame.Rect(x, y, w, det.bottom - y), o, field)

    def _draw_mini(self, s, area, o, field):
        """Mini-Vorschau des nachgerechneten Feldes: wie das echte Brett, nur
        klein - aufgedeckte Plätze hell, der Rest abgedunkelt, die getroffene
        Mine rot."""
        g = o["grid"]
        side = min(area.w, area.h)
        if side < g * 6:
            return
        gap = max(1, side // 80)
        cs = (side - gap * (g - 1)) // g
        opened = o.get("open") if isinstance(o.get("open"), list) else []
        mines = [False] * (g * g)
        for v in field:
            mines[v] = True
        hit = opened[-1] if o.get("m") == 0 and opened else None
        for i in range(g * g):
            x = area.x + (i % g) * (cs + gap)
            y = area.y + (i // g) * (cs + gap)
            if i == hit:
                kind = "boom"
            else:
                kind = "mine" if mines[i] else "gem"
            s.blit(D.face(cs, kind, i not in opened), (x, y))
