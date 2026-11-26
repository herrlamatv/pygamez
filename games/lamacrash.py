# -*- coding: utf-8 -*-
"""
lamacrash.py
============
Lama Crash - Mut-Spiel mit Lama-Chips: Ein Lama (bzw. zwei - je Einsatz eins)
steigt im Ballon auf, der Multiplikator wächst mit der Zeit. Wer aussteigt,
bekommt Einsatz × Multiplikator; platzt der Ballon vorher, ist der noch
fliegende Einsatz weg. Nach dem Aussteigen fliegt der leere Ballon weiter, bis
er platzt (Vorspulen möglich) - so sieht man, wie weit es gegangen wäre.

Regeln und Zahlen stehen in ``lamacrash_logic.py`` (ohne pygame, bitgleich in
Web und Rust), die Bilder in ``lamacrash_draw.py`` (Ballon, Korb, Fallschirm …)
und ``lamacrash_world.py`` (Höhenwelten), die Musik in ``lamacrash_music.py``.

- **Zwei Einsätze** (Karte 2 per Schalter/Z), je mit eigenem **Auto-Ziel**;
  jeder Einsatz darf einmal **halbiert** werden (Hälfte steigt aus, Rest fliegt).
- **Höhenwelten**: Wiese → Wolken (×1,5) → Gewitter (×3) → Nordlicht (×10) →
  Weltall (×100) → **Mondlandung** bei ×100.000 (Höchstgewinn).
- **Fairness** (Fenster F): Prüfcode vorab, Platzpunkt aus Server-/Client-Seed
  und Rundennummer, nach der Runde nachrechenbar (Mini-Kurve).
- **Statistik** (S), **Auto-Modus** (A), **Turbo** (T), **Musik** (M).

Lama-Chips: gemeinsames Konto der Lama-Bank. SPIELEN bucht den Einsatz und
speichert die Runde in einem Zug; jeder Ausstieg schreibt den Gewinn gut und
speichert die Runde. Wer mitten im Flug geht, macht beim nächsten Start am
selben Takt weiter. Der Platzpunkt (``self._crash``) wird nur in ``_advance``
(über ``L.tick``) und beim Rundenabschluss gelesen - nichts auf dem Bildschirm
oder im Ton verrät ihn vorher.

Steuerung (alle Tasten roh, nie über die WASD-Belegung): vor der Runde
Leertaste/Enter = SPIELEN, 1/2/Tab = Karte wählen, ↑/↓/+/-/Mausrad = Einsatz,
←/→ = Auto-Ziel, X = Auto-Ziel an/aus, Z = 2. Einsatz; im Flug
Leertaste/Enter/C = alle aussteigen, 1/2 = einzeln, H = Hälfte (alle), Q/W =
Hälfte einzeln; danach Leertaste = Vorspulen. A/S/F/T/M wie oben, Esc schließt
ein Fenster bzw. stoppt Auto (sonst Pause).
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
from . import lamacrash_draw as D
from . import lamacrash_logic as L
from . import lamacrash_music as MU
from . import lamacrash_world as W

KEY = L.KEY
IDLE, COUNT, FLY, COAST, END = "idle", "count", "fly", "coast", "end"

COUNT_T = 1.35         # 3-2-1 vor dem Start
RESUME_T = 1.8         # Countdown beim Fortsetzen einer Runde
END_T = 2.8            # Ergebnis nach dem Platzen, danach Ruhezustand
READ_T = 0.35          # Ergebnis frühestens danach wegdrückbar
AUTO_END_T = 1.0       # Auto: Ergebnis-Anzeige
AUTO_PAUSE_T = 0.35    # Auto: Pause vor der nächsten Runde
TURBO_F = 0.4          # Turbo/Auto: Countdown und Pausen x 0,4
SHAKE_T = 0.6          # Kamera-Wackeln nach dem Platzen
BIG_MULT = 300         # ab x3: Münzregen
PARTY_MULT = 1000      # ab x10: Konfetti
BANNER_MULT = 10000    # ab x100: Gewinn-Banner

ROUND_OPTS = (5, 10, 25, 50, 100, 250)
LIMIT_OPTS = (0, 100, 500, 1000, 5000, 10000, 50000)
TOOLS = ("auto", "stats", "fair", "turbo", "music")
TOOL_LABEL = {"auto": "chip.auto", "stats": "chip.stats", "fair": "chip.fair", "turbo": "chip.turbo",
              "music": "lcx.music"}
TOOL_KEY = {"auto": "A", "stats": "S", "fair": "F", "turbo": "T", "music": "M"}
# Halstuch-/Kartenfarbe und Fallschirm je Einsatz
CARD_COL = ((255, 196, 72), (96, 198, 255))
CHUTE_COL = ((255, 140, 60), (70, 150, 255))
# beim Erreichen dieser Multiplikatoren ein kurzer Ton (steigend)
TONE_AT = (200, 300, 500, 1000, 2000, 5000, 10000, 100000, 1000000)


class LamaCrashGame(Game):
    match_manual = True       # jede Runde ist ein Match (Game.log_match)
    name = "Lama Crash"
    highscore_key = KEY
    supports_multiplayer = False

    @property
    def wants_escape(self):
        # Esc schließt ein offenes Fenster bzw. stoppt den Auto-Modus
        return self.window is not None or self.auto is not None

    @property
    def pause_on_blur(self):
        # Fokus weg, während Geld fliegt -> Pause (der Ballon stiege sonst weiter)
        return self.round is not None and self.state in (COUNT, FLY) and L.flying(self.round)

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.game_over = False
        lamabank.load()
        self.seed_rng = random.Random()        # nur Fairness-Seeds
        self.rng = random.Random()             # Optik (Partikel, Münzen …)
        self.bets = [self._choice_opt("bet1", L.BETS, L.BETS[0]), self._choice_opt("bet2", L.BETS, L.BETS[0])]
        self.autos = [self._opt("auto1", False) is True, self._opt("auto2", True) is True]
        self.targets = [self._target_opt("target1", L.DEFAULT_TARGET), self._target_opt("target2", 1000)]
        self.two = self._opt("two", False) is True
        self.turbo = self._opt("turbo", False) is True
        self.music_on = self._opt("music", True) is True
        self.auto_rounds = self._choice_opt("auto_rounds", ROUND_OPTS, 10)
        self.auto_win = self._choice_opt("auto_win", LIMIT_OPTS, 0)
        self.auto_loss = self._choice_opt("auto_loss", LIMIT_OPTS, 0)
        extra = lamabank.get_extra(KEY, {})
        extra = extra if isinstance(extra, dict) else {}
        self.hist = L.clean_history(extra.get("hist"))
        self.fair = fairness.clean_state(extra.get("fair"), self.seed_rng)
        self.fairlog = fairness.clean_log(extra.get("fairlog"))
        self.stats = chipstats.clean(extra.get("stats"))
        self.stats["x"] = L.clean_x(self.stats["x"])
        self.carry = L.clean_carry(extra.get("carry"))     # Rest-Hundertstel der Auszahlungen
        self.round = L.clean_round(extra.get("round"))
        self._crash = None            # Platzpunkt der laufenden Runde (nur _advance/_finish!)
        self.state = IDLE
        self.sel = 0                  # gewählte Einsatz-Karte
        self.acc = 0.0                # angefangene Takte (Zeit-Akkumulator)
        self.count_t = 0.0            # Countdown-Rest
        self.count_len = COUNT_T
        self.ff = False               # Vorspulen (nur wenn alle ausgestiegen sind)
        self.end = None               # "pop" / "moon" der letzten Runde
        self.end_t = 0.0
        self.last = None              # letzte fertige Runde (Anzeige im Ruhezustand)
        self.tone_i = 0               # nächster Ton aus TONE_AT
        self.world_seen = 0           # höchste schon angesagte Höhenwelt der Runde
        self.world_label = None       # [Text-Schlüssel, Restzeit]
        self.squeak_t = 0.0
        self.flame_t = 0.0            # Brenner-Stoß
        self.env_rect = pygame.Rect(0, 0, 0, 0)   # Hülle im letzten Bild (für den Knall)
        self.chutes = []              # Lamas / Geldsäcke am Fallschirm
        self.fallers = []             # purzelnde Lamas nach dem Platzen
        self.shreds = []              # Gummifetzen
        self.pop_fx = None            # Knall-Stern {x, y, t, r}
        self.basket_fx = None         # fallender Korb nach dem Platzen
        self.particles = []
        self.coins_fx = []
        self.popups = []
        self.toast = None
        self.banner = None            # [Titel, Untertitel, Restzeit, Dauer]
        self.shake = 0.0
        self.time = 0.0
        self.mouse = (-1, -1)
        self.broke = False
        self.window = None            # None / "auto" / "stats" / "fair"
        self.auto = None              # laufender Auto-Modus (dict)
        self.auto_focus = 0
        self.fair_sel = None
        self.fair_scroll = 0
        self.fair_checks = {}         # Nonce -> (stimmt?, Platzpunkt, Ausstiege)
        self.r_fair_rows = []
        self.r_fair_new = pygame.Rect(0, 0, 0, 0)
        self.r_fair_verify = pygame.Rect(0, 0, 0, 0)
        self.stats_hover = None
        self.mus_stage = None
        self.mus_t = 0.0
        self.mus_vol = None
        self._mus_fail = None          # Stufe, deren Start fehlschlug (kein Mixer)
        self.pending = 0
        self.shown_chips = float(lamabank.balance())
        self._tick_cd = 0.0
        self._lang = get_language()
        self._layout()
        if extra.get("fair") != self.fair:
            # frisch gewürfelte Seeds sofort sichern: der gezeigte Prüfcode bleibt gültig
            self._save_extra()
        if self.round is not None:
            self._crash = L.crash_point(self.round["s"], self.round["c"], self.round["n"])
            if L.flying(self.round):
                # Unterbrochene Runde: Countdown, dann geht es am selben Takt weiter
                self.two = len(self.round["bets"]) > 1
                self.state = COUNT
                self.count_t = self.count_len = RESUME_T
                self.tone_i = self._tone_index(L.mult_at(self.round["t"]))
                self.world_seen = min(4, L.world_of(L.mult_at(self.round["t"])))
                self._say(t("lcx.resumed"), 2.4)
            else:
                self._finish_now()
        if self.round is None:
            self._clamp_bets()
        self.score = lamabank.score_for(KEY)
        self._check_broke()

    def on_surface_changed(self):
        D.clear_cache()
        W.clear_cache()
        CD.clear_cache()
        self._layout()

    def on_exit(self):
        """Verlassen: fliegt noch Geld, wird der Takt gespeichert (Fortsetzen);
        sind alle ausgestiegen, wird die Runde sofort abgeschlossen."""
        audio.music_stop(self)
        self.auto = None
        if self.round is not None:
            if L.flying(self.round):
                self._save_extra()
            else:
                self._finish_now()
        self.score = lamabank.score_for(KEY)

    # ===================================================== Layout
    def _layout(self):
        W_, H = self.width, self.height
        self.k = k = min(W_ / 800.0, H / 600.0)
        self.f_micro = ui.font(max(10, int(11 * k)))
        self.f_tiny = ui.font(max(11, int(13 * k)))
        self.f_small = ui.font(max(12, int(15 * k)))
        self.f_btn = ui.font(max(12, int(15 * k)), bold=True)
        self.f_big = ui.font(max(15, int(22 * k)), bold=True)
        self.f_mult = ui.font(max(20, int(40 * k)), bold=True)
        self.f_huge = ui.font(max(26, int(58 * k)), bold=True)
        self.f_mono = ui.font(max(10, int(12 * k)), mono=True)
        self.fs_micro = max(9, int(11 * k))     # Schriftgrößen für gecachte Zeichen-Bausteine
        self.fs_tiny = max(11, int(13 * k))
        self.margin = m = max(6, int(12 * k))
        self.hud_h = max(28, int(42 * k))
        self.strip_h = max(46, int(64 * k))
        self.tool_h = max(26, int(34 * k))
        self.strip = pygame.Rect(0, H - self.strip_h, W_, self.strip_h)
        tool_y = self.hud_h + m
        area_y = tool_y + self.tool_h + m
        area_h = self.strip.y - m - area_y
        panel_w = max(184, min(int(W_ * 0.38), int(300 * k)))
        self.stage = pygame.Rect(m, area_y, W_ - panel_w - 3 * m, area_h)
        self.panel_r = pygame.Rect(self.stage.right + m, area_y, panel_w, area_h)
        self._hud_bg = pygame.Surface((W_, self.hud_h), pygame.SRCALPHA)
        for y in range(self.hud_h):
            a = int(210 - 60 * y / max(1, self.hud_h - 1))
            self._hud_bg.fill((10, 8, 16, a), (0, y, W_, 1))
        self._strip_bg = pygame.Surface(self.strip.size, pygame.SRCALPHA)
        for y in range(self.strip_h):
            a = int(170 + 60 * y / max(1, self.strip_h - 1))
            self._strip_bg.fill((10, 8, 16, a), (0, y, W_, 1))
        self._overlay = pygame.Surface((W_, H), pygame.SRCALPHA)
        self._overlay.fill((6, 4, 12, 200))
        self._bg = pygame.Surface((W_, H))
        for y in range(H):
            self._bg.fill(ui.mix((22, 16, 34), (8, 6, 14), y / max(1, H - 1)), (0, y, W_, 1))
        self._layout_tools(tool_y)
        self._layout_strip()
        self._layout_cards()
        self._layout_stage()
        self._layout_auto()

    def _layout_tools(self, y):
        """Werkzeug-Knöpfe oben links (mit Beschriftung, wenn Platz ist)."""
        m = self.margin
        h = self.tool_h
        pad = max(6, int(10 * self.k))
        gap = max(3, int(5 * self.k))
        widths = [h + pad + self.f_tiny.size(t(TOOL_LABEL[name]))[0] for name in TOOLS]
        total = 0
        for w in widths:
            total += w + gap
        self.tool_labels = total <= int(self.stage.w * 0.72)
        x = m
        self.r_tools = []
        for i, name in enumerate(TOOLS):
            w = widths[i] if self.tool_labels else h
            self.r_tools.append((name, pygame.Rect(x, y, w, h)))
            x += w + gap
        self.tools_right = x

    def _layout_strip(self):
        """Bedienleiste: Gesamteinsatz, Hälfte-Knopf, Status, Hauptknopf."""
        m, k, W_ = self.margin, self.k, self.width
        sh = self.strip_h
        bh = max(30, int(sh * 0.62))
        by = self.strip.y + (sh - bh) // 2
        self.strip_bh = bh
        tw = max(self.f_tiny.size(t("lcx.total"))[0], self.f_btn.size("20.000")[0]) + int(16 * k)
        self.r_total = pygame.Rect(m, by, tw, bh)
        hw = max(bh, bh + self.f_btn.size(t("lcx.half_all"))[0] + int(14 * k))
        self.r_half = pygame.Rect(self.r_total.right + max(4, int(8 * k)), by, hw, bh)
        pad = int(22 * max(0.7, k))
        labels = (t("lcx.play"), t("lcx.cash_all") + " 9.999.999", t("lcx.ff"), t("chip.auto_stop"))
        go_w = 0
        for lab in labels:
            go_w = max(go_w, self.f_btn.size(lab)[0] + pad)
        go_w = min(max(go_w, int(130 * k)), int(W_ * 0.45))
        self.r_go = pygame.Rect(W_ - m - go_w, by, go_w, bh)
        room = self.r_go.x - m - self.r_half.right - m
        self.half_label = room >= 0
        if room < 0:
            # zu eng: Hälfte-Knopf nur mit Symbol
            self.r_half.w = bh
        self.status_x = (self.r_half.right + m, self.r_go.x - m)

    def _layout_cards(self):
        """Seitenleiste: zwei Einsatz-Karten, Info-Kasten, Hinweis."""
        p = self.panel_r
        k = self.k
        self.ppad = pad = max(5, int(8 * k))
        x0, w = p.x + pad, p.w - 2 * pad
        self.panel_inner = (x0, w)
        self.row_h = rh = max(22, int(28 * k))
        self.gap = gap = max(4, int(6 * k))
        cpad = max(4, int(6 * k))
        title_h = self.f_tiny.get_height() + max(2, int(4 * k))
        card_h = 2 * cpad + title_h + 2 * rh + gap
        self.cards = []
        y = p.y + pad
        for i in range(2):
            c = pygame.Rect(x0, y, w, card_h)
            iw = w - 2 * cpad
            ix = c.x + cpad
            ty = c.y + cpad
            ya = ty + title_h
            yb = ya + rh + gap
            sw_w = max(int(rh * 1.6), 30)
            vw = iw - 2 * rh - 8
            tv = iw - 3 * rh - 12
            cash_w = iw - rh - 4
            self.cards.append(dict(
                rect=c, title_y=ty, title_h=title_h,
                sw=pygame.Rect(c.right - cpad - sw_w, ty, sw_w, title_h - 2),
                minus=pygame.Rect(ix, ya, rh, rh), bet=pygame.Rect(ix + rh + 4, ya, vw, rh),
                plus=pygame.Rect(ix + rh + 4 + vw + 4, ya, rh, rh),
                auto=pygame.Rect(ix, yb, rh, rh), tl=pygame.Rect(ix + rh + 4, yb, rh, rh),
                tv=pygame.Rect(ix + 2 * rh + 8, yb, tv, rh),
                tr=pygame.Rect(ix + 2 * rh + 8 + tv + 4, yb, rh, rh),
                cash=pygame.Rect(ix, yb, cash_w, rh), halfc=pygame.Rect(ix + cash_w + 4, yb, rh, rh),
                row_a=pygame.Rect(ix, ya, iw, rh), row_b=pygame.Rect(ix, yb, iw, rh)))
            y = c.bottom + gap
        self.info_y = y
        hint_h = 2 * self.f_micro.get_height() + 2
        self.hint_y = p.bottom - pad - hint_h
        if self.hint_y - self.info_y < 3 * (self.f_tiny.get_height() + 2):
            self.hint_y = p.bottom - pad          # zu eng: kein Hinweis

    def _layout_stage(self):
        """Bühne: Zeichenfläche der Kurve (Platz für den Ballon über der Spitze)."""
        st = self.stage
        self.rig_base = base = max(30, int(st.h * 0.19))
        bw, bh = D.balloon_size(base, D.INFL_MAX)
        basket_h = D.basket_w(base) * 2 // 3
        # Start auf dem Startplatz der Wiese (lamacrash_world: Mitte bei 15 % der
        # Breite, Oberkante max(16, 5 % der Höhe) über dem Bühnenrand)
        left = max(bw // 2 + 4, self.f_micro.size("×1.000")[0] + 10, st.w * 15 // 100)
        right = max(bw // 2 + 4, int(10 * self.k))
        top = int(bh * 0.92)
        bottom = basket_h + max(16, st.h * 5 // 100)
        self.plot = pygame.Rect(st.x + left, st.y + top, max(20, st.w - left - right),
                                max(20, st.h - top - bottom))

    def _layout_auto(self):
        """Auto-Fenster über der Bühne."""
        r = self.stage.inflate(-self.margin, -self.margin)
        self.auto_r = r
        k = self.k
        pad = max(6, int(10 * k))
        cb = max(18, int(24 * k))
        self.r_auto_close = pygame.Rect(r.right - pad - cb, r.y + pad, cb, cb)
        x0, w = r.x + pad, r.w - 2 * pad
        y = r.y + pad + max(cb, self.f_btn.get_height()) + max(3, int(6 * k))
        lab = self.f_tiny.get_height()
        rh = max(20, int(26 * k))
        gap = max(3, int(5 * k))
        rows = []
        for opts in (ROUND_OPTS, LIMIT_OPTS, LIMIT_OPTS):
            ly = y
            y += lab + 2
            n = len(opts)
            first = 0
            if opts[0] == 0:
                first = max(0, min(self.f_tiny.size(t("chip.auto_off"))[0],
                                   self.f_micro.size(t("chip.auto_off"))[0] + 4) + 8)
            bw = (w - first - (n - 1) * 3) // (n if not first else n - 1)
            rects, x = [], x0
            for i in range(n):
                rw = first if (i == 0 and first) else bw
                rects.append(pygame.Rect(x, y, rw, rh))
                x += rw + 3
            rows.append((ly, rects))
            y += rh + gap
        self.auto_rows = rows
        self.r_auto_turbo = pygame.Rect(x0, y, min(w, max(int(w * 0.5), self.f_btn.size(t("chip.turbo"))[0]
                                                          + 3 * rh + 10)), rh)
        y += rh + gap
        self.auto_sum_y = y
        bh = max(26, int(34 * k))
        self.r_auto_start = pygame.Rect(r.centerx - min(w, int(220 * k)) // 2, r.bottom - pad - bh,
                                        min(w, int(220 * k)), bh)

    # ===================================================== Einstellungen
    def _opt(self, key, default):
        sec = self.settings.get(KEY) if isinstance(self.settings, dict) else None
        if isinstance(sec, dict) and key in sec:
            return sec[key]
        return default

    def _choice_opt(self, key, opts, default):
        v = self._opt(key, default)
        return v if (isinstance(v, int) and not isinstance(v, bool) and v in opts) else default

    def _target_opt(self, key, default):
        v = self._opt(key, default)
        return v if L.target_ok(v) else default

    def _set_opt(self, key, value):
        if not isinstance(self.settings, dict):
            return
        sec = self.settings.setdefault(KEY, {})
        if sec.get(key) != value:
            sec[key] = value
            settings_mod.save_settings(self.settings)

    @staticmethod
    def _is_action(key):
        """Leertaste/Enter. Alle Tasten werden ROH ausgewertet (wie Lama Mines) -
        nie über is_action/WASD, sonst hätten A und S eine Doppelbedeutung."""
        return key in ("space", "Return", "KP_Enter")

    # ===================================================== Konto / Speichern
    def _extra(self):
        return {"round": self.round, "hist": self.hist, "fair": self.fair, "fairlog": self.fairlog,
                "stats": self.stats, "carry": self.carry}

    def _save_extra(self):
        lamabank.set_extra(KEY, self._extra())

    def _active(self):
        """Indizes der aktiven Einsatz-Karten (Karte 1 immer, Karte 2 per Schalter)."""
        return [0, 1] if self.two else [0]

    def _total(self):
        total = 0
        for i in self._active():
            total += self.bets[i]
        return total

    def _clamp_bets(self):
        """Einsätze auf das Bezahlbare senken - zuerst Karte 2, dann Karte 1."""
        chips = lamabank.balance()
        for i in ([1, 0] if self.two else [0]):
            others = self._total() - self.bets[i]
            while self.bets[i] > L.BETS[0] and others + self.bets[i] > chips:
                self.bets[i] = L.BETS[L.BETS.index(self.bets[i]) - 1]

    def _check_broke(self):
        self.broke = (self.state == IDLE and self.round is None and self.auto is None
                      and lamabank.is_broke(KEY))
        if self.broke:
            self.window = None
            self.play_sound("gameover")

    def _take_credit(self):
        if lamabank.refill_if_broke(KEY):
            self.play_sound("powerup")
            ui.spawn_burst(self.width // 2, self.height // 2, ui.GOLD, n=26)
        self.broke = False
        self.shown_chips = float(lamabank.balance())

    def _say(self, text, secs=1.8):
        self.toast = [text, secs]

    def _tf(self):
        """Zeitfaktor für Countdown und Pausen (Turbo/Auto schneller)."""
        return TURBO_F if (self.turbo or self.auto is not None) else 1.0

    @staticmethod
    def _entry(rnd):
        return {"n": rnd["n"], "s": rnd["s"], "c": rnd["c"], "h": rnd["h"]}

    # ===================================================== Formatierung
    def _sep(self):
        return t("lcx.decimal")

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
        """Multiplikator x100 als Text: ×1.234,56 (ab ×10.000 ohne Nachkommastellen)."""
        if m100 >= 1000000:
            return "×" + self._group(m100 // 100)
        return "×" + self._group(m100 // 100) + self._sep() + "%02d" % (m100 % 100)

    def _fmt_short(self, m100):
        return "×" + L.fmt_short(m100, self._sep())

    @staticmethod
    def _mult_col(m100):
        """Farbe des großen Multiplikators: weiß → gold → orange → rot → violett."""
        stops = ((100, (240, 244, 255)), (200, (255, 226, 120)), (1000, (255, 168, 60)),
                 (10000, (255, 84, 96)), (100000, (214, 120, 255)))
        if m100 <= stops[0][0]:
            return stops[0][1]
        for (a, ca), (b, cb) in zip(stops, stops[1:]):
            if m100 <= b:
                return ui.mix(ca, cb, (m100 - a) / (b - a))
        return stops[-1][1]

    # ===================================================== Kurve / Höhe
    def _alt(self):
        """Höhe als gebrochener Takt (für Bild und Welt; Geld zählt nur ganze Takte)."""
        if self.state in (FLY, COAST) and self.round is not None:
            return self.round["t"] + self.acc
        if self.state == COUNT and self.round is not None:
            return float(self.round["t"])
        if self.state == END and self.last is not None:
            return float(self.last["rnd"]["t"])
        return 0.0

    @staticmethod
    def _m_at_f(tf):
        """Multiplikator x100 zu einem gebrochenen Takt (nur fürs Zeichnen)."""
        i = int(tf)
        a = L.mult_at(i)
        b = L.mult_at(i + 1)
        return a + (b - a) * (tf - i)

    def _cur_mult(self):
        """Angezeigter Multiplikator (ganzer Takt; nach dem Platzen der Platzpunkt)."""
        if self.state in (FLY, COAST, COUNT) and self.round is not None:
            return L.mult_at(self.round["t"])
        if self.state == END and self.last is not None:
            return self.last["crash"] if self.last["end"] == "pop" else L.MAX_MULT100
        return 100

    @staticmethod
    def _tone_index(m100):
        i = 0
        while i < len(TONE_AT) and m100 >= TONE_AT[i]:
            i += 1
        return i

    # ===================================================== Ablauf
    def _play(self):
        """SPIELEN: Einsätze abbuchen, Runde speichern, Countdown starten."""
        if self.state == END and self.end_t >= READ_T:
            self._to_idle()
        if self.state != IDLE or self.broke or self.round is not None:
            return False
        forced = self.auto is not None
        pairs = []
        for i in self._active():
            a = self.targets[i] if (self.autos[i] or forced) else 0
            pairs.append((self.bets[i], a))
        total = self._total()
        if total > lamabank.balance():
            self._say(t("cas.not_enough"))
            self.play_sound("hit")
            return False
        entry = fairness.begin(self.fair)
        fairness.advance(self.fair, self.seed_rng)
        self.round = L.new_round(entry, pairs)
        if not lamabank.bet_and_pay(total, 0, KEY, KEY, self._extra()):
            self.round = None
            return False
        self._crash = L.crash_point(entry["s"], entry["c"], entry["n"])
        self.state = COUNT
        self.count_t = self.count_len = COUNT_T * self._tf()
        self.acc = 0.0
        self.ff = False
        self.tone_i = self._tone_index(100)
        self.world_seen = 0
        self.world_label = None
        self.squeak_t = 0.0
        self.last = None
        self.end = None
        self.chutes, self.fallers, self.shreds = [], [], []
        self.pop_fx = self.basket_fx = None
        self.window = None
        self.score = lamabank.score_for(KEY)
        self.play_sound("select")
        return True

    def _launch(self):
        """Countdown vorbei: Brenner-Stoß, der Ballon hebt ab."""
        self.state = FLY
        self.acc = 0.0
        self.flame_t = 0.9
        audio.tone(150, 0.45, self.settings, "noise", 0.16)
        audio.tone(1320, 0.1, self.settings, "sine", 0.25)

    def _advance(self, dt):
        """Takte abarbeiten (Takt-Regel in L.tick) - je Takt Auszahlungen und Ende."""
        if self.state == COAST and (self.ff or self.auto is not None):
            speed = L.SPEED_FF
        else:
            speed = L.SPEED_TURBO if self.turbo else 1
        dt = min(dt, 0.1)                       # ein Ruckler wird nicht nachgeholt (wie main.py)
        self.acc += dt * 60.0 * speed
        n = int(self.acc + 1e-9)
        self.acc = max(0.0, self.acc - n)
        for _ in range(n):
            events, self.carry, end = L.tick(self.round, self._crash, self.carry)
            for ev in events:
                self._paid(ev)
            if end is not None:
                self._finish(end)
                return
            if events:
                self._after_pay()

    def _moon_rig(self):
        """Landeplatz des Korbs auf dem Mond (Korbrand-Mitte)."""
        return self.stage.centerx, self.stage.bottom - int(self.stage.h * 0.16)

    def _rig_pos(self):
        """Korbrand-Mitte des Ballons zur aktuellen Höhe (Bildschirm)."""
        tf = self._alt()
        if tf <= 0:
            return float(self.plot.x), float(self.plot.bottom)
        x_max, y_max = L.chart_span(int(tf))
        return self._pt(tf, self._m_at_f(tf), x_max, y_max)

    def _pt(self, tf, m100, x_max, y_max):
        p = self.plot
        x = p.x + p.w * min(1.0, tf / max(1, x_max))
        y = p.bottom - p.h * (m100 - 100) / max(1, y_max - 100)
        return x, y

    def _n_riders(self):
        if self.round is not None:
            return len(self.round["bets"])
        if self.last is not None and self.state == END:
            return len(self.last["rnd"]["bets"])
        return len(self._active())

    def _slot_x(self, i):
        """Platz eines Lamas im Korb (links/rechts) relativ zur Korbmitte."""
        if self._n_riders() < 2:
            return 0
        bw = D.basket_w(self.rig_base)
        return -bw // 4 if i == 0 else bw // 4

    def _paid(self, ev):
        """Auto-Ziel bzw. Mondlandung: gutschreiben (gespeichert wird danach einmal)."""
        i, kind, m100, win = ev
        lamabank.credit(win, KEY, save=False)
        if kind == "auto":
            self._jump(i, "llama", win)
            self._celebrate(m100)
        else:
            x, y = self._moon_rig()
            self._popup(x + self._slot_x(i), y - self.rig_base * 0.9 - i * self.f_big.get_height(),
                        "+" + self._group(win), CD.COL_GOLD, 2.4)
        self._cash_ach(m100)

    def _after_pay(self):
        """Nach Auszahlungen im Flug: einmal speichern, ggf. in den Leerflug."""
        self._save_extra()
        self.score = lamabank.score_for(KEY)
        if self.state == FLY and not L.flying(self.round):
            self.state = COAST
            self.ff = False

    def _cash_ach(self, m100):
        """Erfolge beim Aussteigen sofort (verrät nichts, geht beim Verlassen nicht verloren)."""
        if m100 >= L.BRAVE_MULT:
            self.ach_event("lcx_brave")
        if m100 >= L.SPACE_MULT:
            self.ach_event("lcx_space")

    def _jump(self, i, kind, win):
        """Lama springt ab (bzw. Geldsack fliegt) - mit Fallschirm, Popup, Ton."""
        x, y = self._rig_pos()
        n = D.llama_size(self.rig_base)
        sx = x + self._slot_x(i)
        size = n if kind == "llama" else max(8, n * 2 // 3)
        self.chutes.append(dict(kind=kind, slot=i, x=sx, y=y + n * 0.15, vx=-(40 + 30 * i) * self.k,
                                vy=-150 * self.k, t=0.0, n=size))
        self._popup(sx, y - n, "+" + self._group(win), CD.COL_GOLD, 1.4)
        self.play_sound("win" if kind == "llama" else "merge")
        audio.tone(660, 0.07, self.settings, "sine", 0.22)

    def _cash_parts(self, parts):
        """Von Hand aussteigen: parts = [(Einsatz-Index, Hälfte?)] - ein Speichern."""
        if self.state != FLY or self.round is None:
            return False
        if L.mult_at(self.round["t"]) < L.MIN_CASH:
            self._say(t("lcx.need_cash"), 1.2)
            return False
        done = False
        best = 0
        for i, half in parts:
            res = L.cash_out(self.round, i, half, self.carry)
            if res is None:
                continue
            win, m100, self.carry = res
            lamabank.credit(win, KEY, save=False)
            self._jump(i, "sack" if half else "llama", win)
            self._cash_ach(m100)
            best = max(best, m100)
            done = True
        if done:
            self._after_pay()
            self._celebrate(best)
        return done

    def _cash_all(self):
        if self.round is None:
            return
        parts = [(i, False) for i, b in enumerate(self.round["bets"]) if L.riding(b) > 0]
        if parts and not self._cash_parts(parts):
            self.play_sound("hit")

    def _cash_one(self, i, half):
        if self.round is None or i >= len(self.round["bets"]):
            return
        b = self.round["bets"][i]
        if L.riding(b) <= 0:
            return
        if half and b["hm"]:
            self._say(t("lcx.half_done"), 1.2)
            self.play_sound("hit")
            return
        if not self._cash_parts([(i, half)]):
            self.play_sound("hit")

    def _half_all(self):
        if self.round is None:
            return
        parts = [(i, True) for i, b in enumerate(self.round["bets"]) if L.riding(b) > 0 and not b["hm"]]
        if not parts:
            self._say(t("lcx.half_done"), 1.2)
            self.play_sound("hit")
            return
        if not self._cash_parts(parts):
            self.play_sound("hit")

    def _finish(self, end, quiet=False):
        """Rundenende verbuchen: Zähler, Statistik, Verlauf, Fairness-Log - einmal."""
        rnd = self.round
        if rnd is None:
            return
        crash = self._crash
        L.record_x(self.stats["x"], rnd, crash, end)
        stake, paid = L.stake_total(rnd), L.paid_total(rnd)
        chipstats.record(self.stats, stake, paid, L.best_cash(rnd))
        net = paid - stake
        info = {"bet": stake, "win": paid, "cr": crash}
        if L.best_cash(rnd):
            info["x"] = L.best_cash(rnd)
        self.log_match(net, 1 if net > 0 else 0 if net < 0 else 2, info)
        self.hist = (self.hist + [crash])[-L.HISTORY:]
        fairness.record(self.fairlog, self._entry(rnd), L.outcome(rnd, crash))
        close = L.close_call(rnd, crash)
        lost = [i for i, b in enumerate(rnd["bets"]) if L.riding(b) > 0] if end == "pop" else []
        if self.auto is not None:
            self.auto["net"] += paid - stake
        self.round = None
        self._crash = None
        self.last = dict(rnd=rnd, crash=crash, end=end, stake=stake, paid=paid, lost=lost)
        self.fair_sel = None
        self._save_extra()
        self.score = lamabank.score_for(KEY)
        if close:
            self.ach_event("lcx_close")
        if quiet:
            self.state = IDLE
            return
        self.state = END
        self.end = end
        self.end_t = 0.0
        if end == "pop":
            self._pop(lost)
        else:
            self._moon()
        if close:
            self._say(t("lcx.close"), 2.0)

    def _finish_now(self):
        """Runde ohne fliegendes Geld sofort abschließen (Verlassen im Leerflug, Laden)."""
        if self.round is None:
            return
        end = "moon" if self._crash >= L.MAX_MULT100 else "pop"
        self.round["t"] = L.pop_tick(self._crash)
        self._finish(end, quiet=True)

    def _pop(self, lost):
        """Der Ballon platzt: Knall, Fetzen, Wackeln, purzelnde Lamas, fallender Korb."""
        e = self.env_rect if self.env_rect.w > 0 else pygame.Rect(self.plot.centerx, self.plot.y, 40, 40)
        self.pop_fx = dict(x=e.centerx, y=e.centery, t=0.0, r=max(12, e.w // 2 + int(10 * self.k)))
        self.shake = SHAKE_T
        pieces = D.shreds(max(8, e.w), self.accent)
        for j in range(18):
            a = self.rng.uniform(0, math.tau)
            sp = self.rng.uniform(120, 420) * self.k
            self.shreds.append(dict(x=e.centerx, y=e.centery, vx=math.cos(a) * sp,
                                    vy=math.sin(a) * sp - 80 * self.k, t=0.0,
                                    life=self.rng.uniform(0.9, 1.6), i=j % len(pieces), w=max(8, e.w)))
        x, y = self._rig_pos()
        n = D.llama_size(self.rig_base)
        for i in lost:
            self.fallers.append(dict(x=x + self._slot_x(i), y=y + n * 0.15, vx=(-60 if i == 0 else 60) * self.k,
                                     vy=-120 * self.k, t=0.0, slot=i))
        self.basket_fx = dict(x=x, y=y, vy=-40 * self.k, t=0.0)
        self.play_sound("explode")
        self.rumble(320)
        if lost:
            audio.tone(220, 0.35, self.settings, "saw", 0.18)

    def _moon(self):
        """Mondlandung: Höchstgewinn - Fahne, Konfetti, Münzregen."""
        self.banner = [t("lcx.moon"), t("lcx.moon_sub"), 3.4, 3.4]
        ui.spawn_confetti(self.width, self.height, 150)
        self._coin_shower(90)
        self.play_sound("level")
        self.rumble(300)

    def _celebrate(self, m100):
        """Jubel im Moment des Aussteigens (hängt nur vom Ausstieg ab, verrät nichts):
        Münzregen ab x3, Konfetti ab x10, Gewinn-Banner ab x100."""
        if m100 >= BANNER_MULT:
            self.banner = [t("cas.s.big_win"), self._fmt(m100), 2.6, 2.6]
            self.play_sound("level")
        if m100 >= PARTY_MULT:
            ui.spawn_confetti(self.width, self.height, 110)
            self._coin_shower(60)
            self.rumble(200)
        elif m100 >= BIG_MULT:
            self._coin_shower(28)

    def _to_idle(self):
        """Ergebnis schließen: Ruhezustand (Korb wieder auf dem Startplatz)."""
        self.state = IDLE
        self.end_t = 0.0
        self.pop_fx = None
        self.basket_fx = None
        self.fallers = []
        self._clamp_bets()
        self._check_broke()

    # ----- Einstellungen ändern ------------------------------------------------
    def _leave_result(self):
        """Wer im Ergebnis etwas verstellt, schließt es (Karten zeigen wieder die Einstellungen)."""
        if self.state == END and self.round is None:
            self._to_idle()

    def _can_edit(self):
        return self.state in (IDLE, END) and self.round is None and self.auto is None

    def _set_bet(self, i, direction):
        if not self._can_edit() or i not in self._active():
            return
        self._leave_result()
        j = L.BETS.index(self.bets[i])
        k = max(0, min(len(L.BETS) - 1, j + direction))
        if k != j:
            self.bets[i] = L.BETS[k]
            self._set_opt("bet%d" % (i + 1), self.bets[i])
            self.play_sound("select")
            if self._total() > lamabank.balance():
                self._say(t("cas.not_enough"), 1.2)

    def _set_target(self, i, direction):
        if not self._can_edit() or i not in self._active():
            return
        self._leave_result()
        v = L.step_target(self.targets[i], direction)
        if v != self.targets[i]:
            self.targets[i] = v
            self._set_opt("target%d" % (i + 1), v)
            self.play_sound("click")
        if not self.autos[i]:
            self.autos[i] = True
            self._set_opt("auto%d" % (i + 1), True)

    def _toggle_target(self, i):
        if not self._can_edit() or i not in self._active():
            return
        self._leave_result()
        self.autos[i] = not self.autos[i]
        self._set_opt("auto%d" % (i + 1), self.autos[i])
        self.play_sound("click")

    def _toggle_two(self):
        if not self._can_edit():
            self._say(t("lcx.locked"), 1.2)
            return
        self._leave_result()
        self.two = not self.two
        self._set_opt("two", self.two)
        if not self.two and self.sel == 1:
            self.sel = 0
        self._clamp_bets()
        self._say(t("lcx.two_on") if self.two else t("lcx.two_off"), 1.0)
        self.play_sound("click")

    def _select(self, i):
        if i == 1 and not self.two:
            return
        self._leave_result()
        if self.sel != i:
            self.sel = i
            self.play_sound("rotate")

    def _toggle_turbo(self):
        self.turbo = not self.turbo
        self._set_opt("turbo", self.turbo)
        self._say(t("lcx.turbo_on") if self.turbo else t("lcx.turbo_off"), 1.0)
        self.play_sound("click")

    def _toggle_music(self):
        self.music_on = not self.music_on
        self._set_opt("music", self.music_on)
        if not self.music_on:
            audio.music_stop(self)
            self.mus_stage = None
        self._say(t("lcx.music_on") if self.music_on else t("lcx.music_off"), 1.0)
        self.play_sound("click")

    def _open_window(self, name):
        if self.window == name:
            self.window = None
            self.play_sound("click")
            return
        if name == "auto" and self.auto is not None:
            self._stop_auto(t("chip.auto_done"))
            return
        if self.round is not None and L.flying(self.round):
            self._say(t("lcx.locked_fly"), 1.2)
            self.play_sound("hit")
            return
        if name == "auto":
            if self.state == END and self.end_t >= READ_T:
                self._to_idle()
            if self.state != IDLE or self.broke or self.round is not None:
                self._say(t("lcx.locked"), 1.2)
                return
        self.window = name
        if name == "fair":
            self.fair_scroll = 0
            self.fair_sel = len(self.fairlog) - 1 if self.fairlog else None
        self.play_sound("click")

    # ----- Auto-Modus ------------------------------------------------------------
    def _start_auto(self):
        if self.round is not None or self.state not in (IDLE, END):
            return
        if self._total() > lamabank.balance():
            self._say(t("cas.not_enough"))
            self.play_sound("hit")
            return
        forced = False
        for i in self._active():
            if not self.autos[i]:
                self.autos[i] = True
                self._set_opt("auto%d" % (i + 1), True)
                forced = True
        self.window = None
        if self.state == END:
            self._to_idle()
        self.auto = {"left": self.auto_rounds, "net": 0, "t": 0.0}
        if forced:
            self._say(t("lcx.auto_forced"), 1.8)
        self.play_sound("powerup")

    def _stop_auto(self, msg=None):
        """Auto beenden. Eine laufende Runde bleibt offen und fliegt weiter."""
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
        if a["left"] <= 0:
            self._stop_auto(t("chip.auto_done"))
            return
        if self.auto_win > 0 and a["net"] >= self.auto_win:
            self._stop_auto(t("chip.auto_hit_win"))
            return
        if self.auto_loss > 0 and -a["net"] >= self.auto_loss:
            self._stop_auto(t("chip.auto_hit_loss"))
            return
        if self._total() > lamabank.balance():
            self._stop_auto(t("chip.auto_no_chips"))
            return
        if self._play():
            a["left"] -= 1
        else:
            self._stop_auto(t("chip.auto_no_chips"))

    # ===================================================== Effekte
    def _spawn(self, x, y, n, col, speed, r, life, grav):
        for _ in range(n):
            a = self.rng.uniform(0, math.tau)
            sp = speed * self.rng.uniform(0.3, 1.0) * self.k
            self.particles.append(dict(x=x, y=y, vx=math.cos(a) * sp, vy=math.sin(a) * sp,
                                       r=r * self.k * self.rng.uniform(0.6, 1.25), col=col, t=0.0,
                                       life=life * self.rng.uniform(0.7, 1.15), g=grav * self.k))
        if len(self.particles) > 260:
            self.particles = self.particles[-260:]

    def _coin_shower(self, n=40):
        W_ = self.width
        for _ in range(n):
            self.coins_fx.append(dict(
                x=self.rng.uniform(W_ * 0.05, W_ * 0.95),
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
    def handle_event(self, event):
        kind = event.kind
        if kind in (InputEvent.MOUSEMOVE, InputEvent.MOUSEDOWN,
                    InputEvent.MOUSEUP) and event.pos is not None:
            self.mouse = event.pos
        if self.broke:
            if kind == InputEvent.MOUSEDOWN or (
                    kind == InputEvent.KEYDOWN and self._is_action(event.key)):
                self._take_credit()
            return
        if kind == InputEvent.KEYDOWN:
            self._on_key(event.key, event.repeat)
        elif kind == InputEvent.WHEEL:
            self._on_wheel(event.delta)
        elif kind == InputEvent.MOUSEDOWN and event.button == 1:
            self._on_click(event.pos)

    def _on_key(self, key, repeat=False):
        low = key.lower() if len(key) == 1 else key
        if key == "Escape":
            if self.window is not None:
                self.window = None
                self.play_sound("click")
            elif self.auto is not None:
                self._stop_auto(t("chip.auto_done"))
            return
        if len(key) == 1 and low in ("a", "s", "f", "t", "m"):
            if repeat:
                return
            if low == "t":
                self._toggle_turbo()
            elif low == "m":
                self._toggle_music()
            else:
                self._open_window({"a": "auto", "s": "stats", "f": "fair"}[low])
            return
        if self.window == "auto":
            self._auto_key(key, repeat)
            return
        if self.window == "fair":
            self._fair_key(key, repeat)
            return
        if self.window is not None:
            if self._is_action(key) and not repeat:
                self.window = None
            return
        st = self.state
        if st == FLY:
            if repeat:
                return
            if self._is_action(key) or low == "c":
                self._cash_all()
            elif key in ("1", "2", "KP_1", "KP_2"):
                self._cash_one(int(key[-1]) - 1, False)
            elif low == "h":
                self._half_all()
            elif low in ("q", "w"):
                self._cash_one(0 if low == "q" else 1, True)
            return
        if st == COAST:
            if (self._is_action(key) or low == "c") and not repeat and not self.ff:
                self.ff = True
                self.play_sound("click")
            return
        if st == COUNT:
            return
        if self.auto is not None:
            # Auto läuft (Ruhe/Ergebnis): Leertaste/Enter = STOPP
            if self._is_action(key) and not repeat:
                self._stop_auto(t("chip.auto_done"))
            return
        if st == END and (self.end_t < READ_T or repeat) and self._is_action(key):
            return
        if self._is_action(key):
            if not repeat:
                self._play()
        elif key in ("1", "2", "KP_1", "KP_2"):
            self._select(int(key[-1]) - 1)
        elif key == "Tab":
            if self.two:
                self._select(1 - self.sel)
        elif key in ("plus", "KP_Add", "Up", "equal"):
            self._set_bet(self.sel, 1)
        elif key in ("minus", "KP_Subtract", "Down"):
            self._set_bet(self.sel, -1)
        elif key in ("Left", "Right"):
            self._set_target(self.sel, 1 if key == "Right" else -1)
        elif low == "x" and not repeat:
            self._toggle_target(self.sel)
        elif low == "z" and not repeat:
            self._toggle_two()

    def _auto_key(self, key, repeat):
        """Tasten im Auto-Fenster: ↑/↓ Zeile, ←/→ Wert, Enter/Leertaste = START/STOPP."""
        if key in ("Up", "Down"):
            self.auto_focus = (self.auto_focus + (1 if key == "Down" else -1)) % 4
            self.play_sound("rotate")
        elif key in ("Left", "Right"):
            d = 1 if key == "Right" else -1
            if self.auto_focus == 3:
                self._toggle_turbo()
                return
            opts = (ROUND_OPTS, LIMIT_OPTS, LIMIT_OPTS)[self.auto_focus]
            cur = (self.auto_rounds, self.auto_win, self.auto_loss)[self.auto_focus]
            j = max(0, min(len(opts) - 1, opts.index(cur) + d))
            self._auto_set(self.auto_focus, opts[j])
        elif self._is_action(key) and not repeat:
            if self.auto is not None:
                self._stop_auto(t("chip.auto_done"))
            else:
                self._start_auto()

    def _auto_set(self, row, value):
        if self.auto is not None:
            return
        key = ("auto_rounds", "auto_win", "auto_loss")[row]
        if getattr(self, key) != value:
            setattr(self, key, value)
            self._set_opt(key, value)
            self.play_sound("click")

    def _fair_order(self):
        return list(range(len(self.fairlog) - 1, -1, -1))        # neueste oben

    def _fair_key(self, key, repeat):
        """Fairness-Fenster: ↑/↓ wählt eine Runde, Enter rechnet nach, C = neuer Client-Seed."""
        low = key.lower() if len(key) == 1 else key
        order = self._fair_order()
        if key in ("Up", "Down") and order:
            pos = order.index(self.fair_sel) if self.fair_sel in order else 0
            pos = max(0, min(len(order) - 1, pos + (1 if key == "Down" else -1)))
            self.fair_sel = order[pos]
            self.play_sound("click")
        elif key in ("Return", "KP_Enter") and not repeat:
            if self.fair_sel is not None:
                self._verify(self.fair_sel)
        elif low == "c" and not repeat:
            self._new_client()
        elif key == "space" and not repeat:
            self.window = None

    def _new_client(self):
        if self.round is not None:
            self._say(t("lcx.locked"), 1.4)
            self.play_sound("hit")
            return
        fairness.new_client(self.fair, self.seed_rng)
        self._save_extra()
        self.play_sound("select")

    def _on_wheel(self, delta):
        d = 1 if delta > 0 else -1
        if self.window == "fair":
            self.fair_scroll = max(0, self.fair_scroll - d)
            return
        if self.window is not None or not self._can_edit():
            return
        for i, c in enumerate(self.cards):
            if c["rect"].collidepoint(self.mouse) and i in self._active():
                self._select(i)
                if c["row_b"].collidepoint(self.mouse):
                    self._set_target(i, d)
                else:
                    self._set_bet(i, d)
                return
        self._set_bet(self.sel, d)

    def _on_click(self, pos):
        for name, r in self.r_tools:
            if r.collidepoint(pos):
                if name == "turbo":
                    self._toggle_turbo()
                elif name == "music":
                    self._toggle_music()
                else:
                    self._open_window(name)
                return
        if self.window == "auto":
            self._auto_click(pos)
            return
        if self.window is not None:
            self._window_click(pos)
            return
        st = self.state
        if self.r_go.collidepoint(pos):
            if st == FLY:
                self._cash_all()
            elif st == COAST:
                if not self.ff:
                    self.ff = True
                    self.play_sound("click")
            elif self.auto is not None:
                self._stop_auto(t("chip.auto_done"))
            elif st in (IDLE, END):
                if st == END and self.end_t < READ_T:
                    return
                self._play()
            return
        if self.r_half.collidepoint(pos):
            if st == FLY:
                self._half_all()
            return
        for i, c in enumerate(self.cards):
            if not c["rect"].collidepoint(pos):
                continue
            if st == FLY and self.round is not None:
                if i < len(self.round["bets"]):
                    if c["cash"].collidepoint(pos):
                        self._cash_one(i, False)
                    elif c["halfc"].collidepoint(pos):
                        self._cash_one(i, True)
                return
            if not self._can_edit():
                if self.auto is not None:
                    self._say(t("lcx.locked"), 1.2)
                return
            if i == 1 and (c["sw"].collidepoint(pos) or not self.two):
                self._toggle_two()
                if self.two:
                    self.sel = 1
                return
            self._select(i)
            if c["minus"].collidepoint(pos):
                self._set_bet(i, -1)
            elif c["plus"].collidepoint(pos):
                self._set_bet(i, 1)
            elif c["auto"].collidepoint(pos):
                self._toggle_target(i)
            elif c["tl"].collidepoint(pos):
                self._set_target(i, -1)
            elif c["tr"].collidepoint(pos):
                self._set_target(i, 1)
            return
        if self.stage.collidepoint(pos) and st == IDLE and self.auto is None:
            self._say(t("lcx.press_play"), 1.4)

    def _auto_click(self, pos):
        if self.r_auto_close.collidepoint(pos):
            self.window = None
            self.play_sound("click")
            return
        for row, (ly, rects) in enumerate(self.auto_rows):
            opts = (ROUND_OPTS, LIMIT_OPTS, LIMIT_OPTS)[row]
            for j, r in enumerate(rects):
                if r.collidepoint(pos):
                    self.auto_focus = row
                    self._auto_set(row, opts[j])
                    return
        if self.r_auto_turbo.collidepoint(pos):
            self.auto_focus = 3
            self._toggle_turbo()
        elif self.r_auto_start.collidepoint(pos):
            if self.auto is not None:
                self._stop_auto(t("chip.auto_done"))
            else:
                self._start_auto()
        elif not self.auto_r.collidepoint(pos) and not self.panel_r.collidepoint(pos):
            self.window = None

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
            self._new_client()
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
        """Nachrechnen: Prüfcode = sha256(Seed), Platzpunkt aus den Seeds, Ausstiege passen."""
        if not 0 <= idx < len(self.fairlog):
            return
        e = self.fairlog[idx]
        ok = fairness.verify(e)
        crash = L.crash_point(e["s"], e["c"], e["n"])
        o = e.get("o")
        ok = ok and L.check_outcome(o, crash)
        marks = []
        if isinstance(o, dict) and isinstance(o.get("b"), list):
            for row in o["b"]:
                if isinstance(row, list) and len(row) == 4:
                    for m in (row[1], row[2]):
                        if isinstance(m, int) and not isinstance(m, bool) and m > 0:
                            marks.append(m)
        self.fair_checks[e["n"]] = (ok, crash, tuple(sorted(marks)))
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
            self.banner[2] -= dt
            if self.banner[2] <= 0:
                self.banner = None
        if self.world_label:
            self.world_label[1] -= dt
            if self.world_label[1] <= 0:
                self.world_label = None
        self.shake = max(0.0, self.shake - dt)
        self.flame_t = max(0.0, self.flame_t - dt)
        st = self.state
        if st == COUNT:
            seg = self.count_len / 3.0
            before = int(self.count_t / seg)
            self.count_t -= dt
            if self.count_t <= 0:
                self._launch()
            elif int(self.count_t / seg) != before:
                audio.tone(880, 0.08, self.settings, "sine", 0.22)
        elif st in (FLY, COAST):
            self._advance(dt)
        elif st == END:
            self.end_t += dt
            limit = AUTO_END_T * self._tf() if self.auto is not None else END_T
            if self.end_t >= limit:
                self._to_idle()
                if self.auto is not None:
                    self.auto["t"] = AUTO_PAUSE_T * self._tf()
        if self.auto is not None and self.state == IDLE:
            self._auto_tick(dt)
        self._update_sounds(dt)
        self._update_fx(dt)
        self._update_music(dt)

    def _update_sounds(self, dt):
        """Töne im Flug - nur vom aktuellen Multiplikator abhängig (verraten nichts)."""
        if self.state not in (FLY, COAST) or self.round is None:
            return
        m = L.mult_at(self.round["t"])
        w = min(4, L.world_of(m))
        if w > self.world_seen:
            self.world_seen = w
            self.world_label = [W.name_key(w), 2.4]
            audio.tone(392.0 * 2 ** (w * 2 / 12.0), 0.22, self.settings, "triangle", 0.18)
        while self.tone_i < len(TONE_AT) and m >= TONE_AT[self.tone_i]:
            audio.tone(523.25 * 2 ** (self.tone_i * 2 / 12.0), 0.12, self.settings, "sine", 0.22)
            self.tone_i += 1
        sq = MU.squeak(m)
        if sq is not None:
            self.squeak_t -= dt
            if self.squeak_t <= 0:
                freq, gap = sq
                audio.tone(freq, 0.09, self.settings, "triangle", 0.10)
                self.squeak_t = gap

    def _update_fx(self, dt):
        k = self.k
        alive = []
        for p in self.particles:
            p["t"] += dt
            p["vy"] += p["g"] * dt
            p["x"] += p["vx"] * dt
            p["y"] += p["vy"] * dt
            if p["t"] < p["life"]:
                alive.append(p)
        self.particles = alive
        alive = []
        for c in self.coins_fx:
            c["vy"] += 900 * k * dt
            c["x"] += c["vx"] * dt
            c["y"] += c["vy"] * dt
            c["spin"] += c["vs"] * dt
            if c["y"] < self.height + 30:
                alive.append(c)
        self.coins_fx = alive
        for p in self.popups:
            p["t"] += dt
        self.popups = [p for p in self.popups if p["t"] < p["dur"]]
        bottom = self.stage.bottom + 60 * k
        alive = []
        for c in self.chutes:
            c["t"] += dt
            if c["t"] < 0.35:
                c["vy"] += 520 * k * dt
            else:
                c["vy"] += (60 * k - c["vy"]) * min(1.0, dt * 3.0)
                c["vx"] *= max(0.0, 1.0 - dt * 0.8)
            c["x"] += (c["vx"] + math.sin(c["t"] * 2.2 + c["slot"]) * 18 * k) * dt
            c["y"] += c["vy"] * dt
            if c["y"] < bottom and c["t"] < 9.0:
                alive.append(c)
        self.chutes = alive
        alive = []
        for f in self.fallers:
            f["t"] += dt
            f["vy"] += 900 * k * dt
            f["x"] += f["vx"] * dt
            f["y"] += f["vy"] * dt
            if f["y"] < bottom:
                alive.append(f)
        self.fallers = alive
        alive = []
        for p in self.shreds:
            p["t"] += dt
            p["vy"] += 420 * k * dt
            p["vx"] *= max(0.0, 1.0 - dt * 1.2)
            p["x"] += p["vx"] * dt
            p["y"] += p["vy"] * dt
            if p["t"] < p["life"]:
                alive.append(p)
        self.shreds = alive
        if self.pop_fx is not None:
            self.pop_fx["t"] += dt
            if self.pop_fx["t"] > 0.6:
                self.pop_fx = None
        if self.basket_fx is not None:
            b = self.basket_fx
            b["t"] += dt
            b["vy"] += 900 * k * dt
            b["y"] += b["vy"] * dt
            if b["y"] > bottom:
                self.basket_fx = None

    def _update_music(self, dt):
        """Spannungsmusik: Stufe je Höhenwelt, Wechsel nur auf dem Schlag."""
        self.mus_t += dt
        if not self.music_on:
            return
        st = self.state
        want, vol = 0, 0.45
        if st == END:
            want, vol = (5, 1.0) if self.end == "moon" else (None, 0.0)
        elif st in (FLY, COAST) and self.round is not None:
            want = MU.stage_of(L.mult_at(self.round["t"]))
            vol = 1.0 if st == FLY else 0.75
        elif st == COUNT:
            want, vol = 0, 0.8
        if want is None:
            if self.mus_stage is not None:
                audio.music_stop(self)
                self.mus_stage = None
            return
        guard = 0
        while not MU.ready(want) and guard < MU.STAGES:
            if not MU.prerender_step():
                break
            guard += 1
        if MU.ready(want):
            playing = audio.music_playing(self)
            switch = self.mus_stage is None or not playing
            if not switch and want != self.mus_stage:
                beat = MU.beat_seconds(self.mus_stage)
                switch = int(self.mus_t / beat) != int((self.mus_t - dt) / beat)
            if switch and (want != self.mus_stage or not playing) and want != self._mus_fail:
                if audio.music_play(MU.voices(want), self, self.settings, vol):
                    self.mus_stage = want
                    self.mus_t = 0.0
                    self.mus_vol = vol
                    self._mus_fail = None
                else:
                    self._mus_fail = want      # kein Mixer: nicht jedes Bild neu versuchen
        if self.mus_stage is not None and vol != self.mus_vol:
            audio.music_set_volume(vol)
            self.mus_vol = vol
        if st != FLY:
            MU.prerender_step()                 # restliche Stufen in ruhigen Bildern vorbereiten

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        s.blit(self._bg, (0, 0))
        self._draw_stage(s)
        self._draw_panel(s)
        self._draw_hud(s)
        self._draw_tools(s)
        self._draw_history(s)
        self._draw_strip(s)
        self._draw_fx(s)
        if self.banner:
            self._draw_banner(s)
        if self.window == "auto":
            self._draw_auto(s)
        elif self.window in ("stats", "fair"):
            self._draw_window(s)
        if self.toast:
            self._draw_toast(s)
        self._draw_tooltip(s)
        if self.broke:
            self._draw_broke(s)

    def _shake_off(self):
        if self.shake <= 0:
            return 0, 0
        a = 8 * self.k * self.shake / SHAKE_T
        return int(math.sin(self.time * 71) * a), int(math.cos(self.time * 53) * a * 0.7)

    def _riders(self):
        """Plätze im Korb: [(Halstuch, Pose) oder None] - je Einsatz ein Lama."""
        out = []
        if self.round is not None:
            for i, b in enumerate(self.round["bets"]):
                out.append((CARD_COL[i], "stand") if L.riding(b) > 0 else None)
        elif self.state == END and self.last is not None:
            for i, b in enumerate(self.last["rnd"]["bets"]):
                keep = self.last["end"] == "moon" and (L.riding(b) > 0 or b["xm"] == L.MAX_MULT100)
                out.append((CARD_COL[i], "hop") if keep else None)
        else:
            for i in self._active():
                out.append((CARD_COL[i], "stand"))
        return out

    def _flame(self):
        if self.state == COUNT or self.flame_t > 0:
            return int(self.time * 14) % 6
        if self.state in (FLY, COAST) and (self.time % 3.1) < 0.45:
            return int(self.time * 14) % 6
        return -1

    def _draw_stage(self, s):
        st = self.stage
        ox, oy = self._shake_off()
        clip = s.get_clip()
        s.set_clip(st)
        alt = self._alt()
        moon = self.state == END and self.end == "moon"
        if moon:
            world, share = 5, 0
        else:
            world, share = L.world_blend(alt)
        W.draw(s, st, world, share, alt, self.time)
        if not moon:
            x_max, y_max = L.chart_span(int(alt))
            ylines = [(int(self._pt(0, v, x_max, y_max)[1]) + oy, self._fmt_short(v))
                      for v in L.chart_ylines(y_max)]
            xlines = [(int(self._pt(v, 100, x_max, y_max)[0]) + ox, "%ds" % (v // L.TPS))
                      for v in L.chart_xlines(x_max)]
            D.draw_grid(s, self.plot.move(ox, oy), ylines, xlines, self.fs_micro)
            if alt > 0:
                pts = []
                n = 48
                for j in range(n + 1):
                    tt = alt * j / n
                    x, y = self._pt(tt, self._m_at_f(tt), x_max, y_max)
                    pts.append((x + ox, y + oy))
                if self.state == END:
                    col = ui.RED
                else:
                    col = ui.mix(self.accent, (255, 255, 255), 0.15)
                D.draw_curve(s, pts, col, self.plot.bottom + oy, max(2, int(3 * self.k)))
        self._draw_rig(s, ox, oy, moon)
        self._draw_flyers(s, ox, oy)
        self._draw_center(s)
        s.set_clip(clip)
        pygame.draw.rect(s, ui.mix(self.accent, ui.BORDER, 0.55), st, 1, border_radius=max(4, int(8 * self.k)))

    def _draw_rig(self, s, ox, oy, moon):
        if self.state == END and self.end == "pop":
            return
        n = self._n_riders()
        riders = self._riders()[:n]
        if moon:
            cx, rim_y = self._moon_rig()
            fl = D.flag(max(12, int(self.rig_base * 0.7)), CARD_COL[0])
            s.blit(fl, fl.get_rect(bottomleft=(cx + D.basket_w(self.rig_base) // 2 + int(6 * self.k),
                                               rim_y + D.basket_w(self.rig_base) * 2 // 3)))
            self.env_rect = D.draw_rig(s, cx, rim_y, self.rig_base, D.INFL_MAX, self.accent, riders, -1)
            return
        x, y = self._rig_pos()
        m = self._cur_mult()
        level = D.infl_level(m)
        sway = 0.0
        if self.state in (FLY, COAST):
            sway = math.sin(self.time * 1.7) * self.k * (1.5 + level * 0.35)
        self.env_rect = D.draw_rig(s, x + ox + sway, y + oy, self.rig_base, level, self.accent, riders,
                                   self._flame())

    def _draw_flyers(self, s, ox, oy):
        for c in self.chutes:
            open_f = max(0.0, min(1.0, (c["t"] - 0.25) / 0.4))
            if c["kind"] == "llama":
                D.draw_jumper(s, c["x"] + ox, c["y"] + oy, c["n"], CARD_COL[c["slot"]], CHUTE_COL[c["slot"]], open_f)
            else:
                D.draw_sack(s, c["x"] + ox, c["y"] + oy, c["n"], CHUTE_COL[c["slot"]], open_f)
        n = D.llama_size(self.rig_base)
        for f in self.fallers:
            D.draw_faller(s, f["x"] + ox, f["y"] + oy, n, CARD_COL[f["slot"]], int(f["t"] * 10) % 8)
        if self.basket_fx is not None:
            b = self.basket_fx
            bw = D.basket_w(self.rig_base)
            x = int(b["x"]) - bw // 2 + ox
            y = int(b["y"]) + oy
            s.blit(D.basket(bw, False), (x, y))
            s.blit(D.basket(bw, True), (x, y))
        for p in self.shreds:
            pieces = D.shreds(p["w"], self.accent)
            img = pieces[p["i"] % len(pieces)]
            s.blit(img, img.get_rect(center=(int(p["x"]) + ox, int(p["y"]) + oy)))
        if self.pop_fx is not None:
            f = self.pop_fx
            img = D.pop_star(f["r"], min(5, int(f["t"] / 0.1)))
            s.blit(img, img.get_rect(center=(f["x"] + ox, f["y"] + oy)))
        for p in self.particles:
            fr = p["t"] / p["life"]
            r = p["r"] * (1 - fr)
            if r >= 0.5:
                pygame.draw.circle(s, p["col"], (int(p["x"]), int(p["y"])), max(1, int(r)))

    def _draw_center(self, s):
        """Großer Multiplikator, Countdown bzw. Ergebnis mitten auf der Bühne."""
        st = self.stage
        cx = st.centerx
        cy = st.y + int(st.h * 0.30)
        state = self.state
        if state == COUNT:
            seg = self.count_len / 3.0
            num = max(1, min(3, int(self.count_t / seg) + 1))
            ph = (self.count_t % seg) / seg
            img = self.f_huge.render(str(num), True, CD.COL_GOLD)
            sc = 1.0 + 0.35 * ph
            img = pygame.transform.smoothscale(img, (max(1, int(img.get_width() * sc)),
                                                     max(1, int(img.get_height() * sc))))
            s.blit(img, img.get_rect(center=(cx, cy)))
            return
        if state in (FLY, COAST):
            m = self._cur_mult()
            txt = self._fmt(m)
            col = self._mult_col(m)
            img = self._fit(txt, (self.f_huge, self.f_mult, self.f_big), st.w - 16, col)
            sh = self._fit(txt, (self.f_huge, self.f_mult, self.f_big), st.w - 16, (12, 8, 20))
            s.blit(sh, sh.get_rect(center=(cx + 2, cy + 3)))
            s.blit(img, img.get_rect(center=(cx, cy)))
            y = cy + img.get_height() // 2 + 2
            if state == FLY and self.round is not None:
                now = 0
                for b in self.round["bets"]:
                    now += L.riding(b) * m // 100
                sub = t("lcx.win_now", n=self._group(now))
                scol = ui.GREEN
            else:
                sub = t("lcx.coast")
                scol = ui.TEXT_DIM
            simg = self._fit(sub, (self.f_btn, self.f_tiny), st.w - 16, scol)
            s.blit(simg, simg.get_rect(midtop=(cx, y)))
            if self.world_label:
                key_, left = self.world_label
                wimg = self._fit(t(key_), (self.f_big, self.f_btn, self.f_tiny), st.w - 24, (255, 255, 255))
                box = wimg.get_rect().inflate(int(28 * self.k), int(10 * self.k))
                box.midtop = (cx, st.y + self.margin)
                panel = pygame.Surface(box.size, pygame.SRCALPHA)
                pygame.draw.rect(panel, (10, 8, 18, 170), panel.get_rect(), border_radius=box.h // 2)
                pygame.draw.rect(panel, (*self.accent, 220), panel.get_rect(), 1, border_radius=box.h // 2)
                panel.blit(wimg, wimg.get_rect(center=(box.w // 2, box.h // 2)))
                a = min(1.0, (2.4 - left) / 0.25, left / 0.5)
                panel.set_alpha(int(255 * max(0.0, a)))
                s.blit(panel, box)
            return
        if state == END and self.last is not None:
            f = min(1.0, self.end_t / 0.18)
            if self.last["end"] == "pop":
                title = t("lcx.popped")
                val = self._fmt(self.last["crash"])
                col = ui.RED
            else:
                return                      # Mondlandung: Banner
            timg = self._fit(title, (self.f_big, self.f_btn), st.w - 16, col)
            vimg = self._fit(val, (self.f_huge, self.f_mult, self.f_big), st.w - 16, col)
            if f < 1.0:
                sc = 0.7 + 0.3 * f
                vimg = pygame.transform.smoothscale(vimg, (max(1, int(vimg.get_width() * sc)),
                                                           max(1, int(vimg.get_height() * sc))))
            s.blit(timg, timg.get_rect(midbottom=(cx, cy - vimg.get_height() // 2)))
            s.blit(vimg, vimg.get_rect(center=(cx, cy)))
            res = self.last["paid"] - self.last["stake"]
            sub = t("lcx.round_net", n=("+" if res > 0 else "") + self._group(res))
            simg = self._fit(sub, (self.f_btn, self.f_tiny), st.w - 16, ui.GREEN if res >= 0 else ui.TEXT_DIM)
            s.blit(simg, simg.get_rect(midtop=(cx, cy + vimg.get_height() // 2 + 2)))
            return
        if state == IDLE and self.last is not None:
            txt = t("lcx.last_crash", x=self._fmt(self.last["crash"] if self.last["end"] == "pop"
                                                  else L.MAX_MULT100))
            img = self._fit(txt, (self.f_small, self.f_tiny), st.w - 16, ui.TEXT_DIM)
            s.blit(img, img.get_rect(midtop=(cx, st.y + self.margin)))

    # ----- Seitenleiste -----------------------------------------------------------
    def _draw_panel(self, s):
        ui.draw_panel(s, self.panel_r, accent_top=self.accent)
        for i in range(2):
            self._draw_card(s, i)
        self._draw_info(s)
        self._draw_hint(s)

    def _card_bet(self, i):
        """Teil-Einsatz der laufenden bzw. letzten Runde für Karte i (oder None)."""
        if self.round is not None:
            return self.round["bets"][i] if i < len(self.round["bets"]) else None
        if self.last is not None and (self.state == END or self.auto is not None):
            bets = self.last["rnd"]["bets"]
            return bets[i] if i < len(bets) else None
        return None

    def _draw_card(self, s, i):
        c = self.cards[i]
        r = c["rect"]
        k = self.k
        col = CARD_COL[i]
        hot = self.mouse
        editing = self.state == IDLE and self.round is None and self.auto is None
        bet = self._card_bet(i)
        active = (i in self._active()) if editing or bet is None else True
        if not editing and bet is None:
            active = False
        rad = max(5, int(8 * k))
        pygame.draw.rect(s, ui.mix(ui.PANEL, (0, 0, 0), 0.28), r, border_radius=rad)
        edge = col if (active and (not editing or self.sel == i)) else (ui.mix(col, ui.BORDER, 0.6) if active
                                                                        else ui.BORDER)
        pygame.draw.rect(s, edge, r, 2 if (editing and self.sel == i and active) else 1, border_radius=rad)
        # Titelzeile: Farbpunkt, "Einsatz n", rechts Schalter bzw. Zustand
        ty, th = c["title_y"], c["title_h"]
        dot_r = max(3, th // 4)
        pygame.draw.circle(s, col if active else ui.TEXT_FAINT, (r.x + max(6, int(9 * k)), ty + th // 2), dot_r)
        title = t("lcx.bet_n", n=i + 1)
        if bet is not None:
            title += "  ·  " + self._group(bet["b"])
        right_w = c["sw"].w + 6 if (i == 1 and editing) else 0
        img = self._fit(title, (self.f_tiny, self.f_micro), r.w - right_w - int(24 * k), ui.TEXT if active else ui.TEXT_DIM)
        s.blit(img, img.get_rect(midleft=(r.x + max(6, int(9 * k)) + dot_r + 5, ty + th // 2)))
        if i == 1 and editing:
            self._switch(s, c["sw"], self.two, c["sw"].collidepoint(hot))
        elif bet is not None:
            badge, bcol = self._bet_badge(bet)
            bimg = self._fit(badge, (self.f_micro,), r.w // 2, bcol)
            s.blit(bimg, bimg.get_rect(midright=(r.right - max(5, int(8 * k)), ty + th // 2)))
        if editing:
            if not active:
                msg = t("lcx.second_hint")
                img = self._fit(msg, (self.f_tiny, self.f_micro), r.w - 12, ui.TEXT_FAINT)
                s.blit(img, img.get_rect(center=(r.centerx, (c["row_a"].y + c["row_b"].bottom) // 2)))
                return
            on_m = self.bets[i] > L.BETS[0]
            on_p = self.bets[i] < L.BETS[-1]
            self._btn(s, c["minus"], "–", on=on_m, hot=c["minus"].collidepoint(hot))
            self._btn(s, c["plus"], "+", on=on_p, hot=c["plus"].collidepoint(hot))
            self._value_box(s, c["bet"], self._group(self.bets[i]),
                            ui.RED if self._total() > lamabank.balance() else ui.GOLD)
            self._check(s, c["auto"], self.autos[i], c["auto"].collidepoint(hot))
            on = self.autos[i]
            self._btn(s, c["tl"], "", on=on and self.targets[i] > L.TARGETS[0], hot=c["tl"].collidepoint(hot),
                      icon="left")
            self._btn(s, c["tr"], "", on=on and self.targets[i] < L.TARGETS[-1], hot=c["tr"].collidepoint(hot),
                      icon="right")
            txt = self._fmt(self.targets[i]) if on else t("lcx.target_off")
            self._value_box(s, c["tv"], txt, col if on else ui.TEXT_FAINT, small_label=t("lcx.target"))
            return
        if bet is None:
            img = self._fit("–", (self.f_btn,), r.w - 12, ui.TEXT_FAINT)
            s.blit(img, img.get_rect(center=(r.centerx, (c["row_a"].y + c["row_b"].bottom) // 2)))
            return
        # Runde läuft bzw. ist gerade vorbei
        ra, rb = c["row_a"], c["row_b"]
        m = self._cur_mult()
        riding = L.riding(bet)
        mark = None
        if riding > 0 and self.state != END:
            left = t("lcx.win_now", n=self._group(riding * m // 100))
            lcol = ui.GREEN
        elif bet["xm"]:
            left, lcol, mark = self._fmt(bet["xm"]), CD.COL_GOLD, "check"
        elif self.state == END:
            left, lcol, mark = t("lcx.lost"), ui.RED, "cross"
        else:
            left, lcol = "…", ui.TEXT_DIM
        right = ("+" + self._group(bet["p"])) if bet["p"] > 0 else (
            t("lcx.target_at", x=self._fmt(bet["a"])) if bet["a"] else "")
        rimg = self._fit(right, (self.f_tiny, self.f_micro), ra.w // 2, CD.COL_GOLD if bet["p"] > 0 else ui.TEXT_DIM)
        lx = ra.x
        if mark is not None:
            isz = max(8, int(ra.h * 0.62))
            s.blit(D.icon(mark, isz, lcol), (lx, ra.centery - isz // 2))
            lx += isz + 4
        limg = self._fit(left, (self.f_btn, self.f_tiny, self.f_micro), ra.right - lx - rimg.get_width() - 6, lcol)
        s.blit(limg, limg.get_rect(midleft=(lx, ra.centery)))
        s.blit(rimg, rimg.get_rect(midright=(ra.right, ra.centery)))
        if self.state == FLY and riding > 0:
            can = m >= L.MIN_CASH
            self._btn(s, c["cash"], t("lcx.cash") + " " + self._group(riding * m // 100), primary=True, on=can,
                      hot=c["cash"].collidepoint(hot))
            self._btn(s, c["halfc"], "", on=can and not bet["hm"], hot=c["halfc"].collidepoint(hot), icon="half")
        else:
            info = ""
            if bet["hm"]:
                info = t("lcx.half_at", x=self._fmt(bet["hm"]))
            elif self.state == COUNT:
                info = t("lcx.ready")
            img = self._fit(info, (self.f_tiny, self.f_micro), rb.w, ui.TEXT_DIM)
            s.blit(img, img.get_rect(midleft=(rb.x, rb.centery)))

    def _bet_badge(self, bet):
        if L.riding(bet) > 0 and self.state != END:
            return t("lcx.riding"), ui.GREEN
        if bet["xm"]:
            return t("lcx.out"), CD.COL_GOLD
        return t("lcx.lost"), ui.RED

    def _value_box(self, s, r, text, col, small_label=None):
        rad = max(4, r.h // 4)
        pygame.draw.rect(s, (16, 12, 26), r, border_radius=rad)
        pygame.draw.rect(s, ui.BORDER, r, 1, border_radius=rad)
        img = self._fit(text, (self.f_btn, self.f_tiny, self.f_micro), r.w - 6, col)
        s.blit(img, img.get_rect(center=r.center))

    def _switch(self, s, r, on, hot):
        pygame.draw.rect(s, CD.COL_GOLD if on else (60, 60, 76), r, border_radius=r.h // 2)
        if hot:
            pygame.draw.rect(s, ui.BORDER_LIGHT, r, 1, border_radius=r.h // 2)
        kr = max(2, r.h // 2 - 2)
        kx = r.right - kr - 3 if on else r.x + kr + 3
        pygame.draw.circle(s, (255, 255, 255), (kx, r.centery), kr)

    def _check(self, s, r, on, hot):
        rad = max(4, r.h // 4)
        pygame.draw.rect(s, ui.BTN_SEL if hot else ui.BTN, r, border_radius=rad)
        pygame.draw.rect(s, CD.COL_GOLD if on else ui.BORDER_LIGHT, r, 2 if on else 1, border_radius=rad)
        if on:
            img = D.icon("check", max(8, int(r.h * 0.62)), CD.COL_GOLD)
            s.blit(img, img.get_rect(center=r.center))

    def _info_rows(self):
        """Zeilen des Info-Kastens: [(Beschriftung, Wert, Farbe)]."""
        if self.state == END and self.last is not None:
            last = self.last
            net = last["paid"] - last["stake"]
            x = last["crash"] if last["end"] == "pop" else L.MAX_MULT100
            return [(t("lcx.crash_at"), self._fmt(x), ui.RED if last["end"] == "pop" else CD.COL_GOLD),
                    (t("chip.st_paid"), self._group(last["paid"]), CD.COL_GOLD if last["paid"] else ui.TEXT_DIM),
                    (t("chip.st_net"), ("+" if net > 0 else "") + self._group(net), ui.GREEN if net >= 0 else ui.RED)]
        if self.round is not None and self.state in (FLY, COAST, COUNT):
            m = self._cur_mult()
            secs = self.round["t"] // L.TPS
            rows = [(t("lcx.world"), t(W.name_key(min(4, L.world_of(m)))), self.accent),
                    (t("lcx.flight"), "%d:%02d" % (secs // 60, secs % 60), ui.TEXT),
                    (t("chip.st_paid"), self._group(L.paid_total(self.round)), CD.COL_GOLD)]
            return rows
        rows = [(t("lcx.total"), self._group(self._total()),
                 ui.RED if self._total() > lamabank.balance() else ui.TEXT)]
        for i in self._active():
            if self.autos[i]:
                rows.append((t("lcx.chance", x=self._fmt(self.targets[i])),
                             L.fmt_chance(self.targets[i], self._sep()) + " %", CARD_COL[i]))
        rows.append((t("chip.max_win"), self._fmt(L.MAX_MULT100), CD.COL_GOLD))
        return rows

    def _draw_info(self, s):
        x0, w = self.panel_inner
        y = self.info_y
        rows = self._info_rows()
        avail = self.hint_y - max(4, int(6 * self.k)) - y
        min_lh = self.f_tiny.get_height() + 2
        max_lh = int(self.f_btn.get_height() * 1.6)
        while rows and avail // len(rows) < min_lh:
            rows = rows[:-1]
        if not rows:
            return
        lh = min(max_lh, avail // len(rows))
        for lab, val, col in rows:
            vimg = self._fit(val, (self.f_btn, self.f_tiny), w // 2, col)
            limg = self._fit(lab, (self.f_small, self.f_tiny, self.f_micro), w - vimg.get_width() - 8, ui.TEXT_DIM)
            s.blit(limg, limg.get_rect(midleft=(x0, y + lh // 2)))
            s.blit(vimg, vimg.get_rect(midright=(x0 + w, y + lh // 2)))
            y += lh

    def _draw_hint(self, s):
        if self.hint_y >= self.panel_r.bottom - self.ppad:
            return
        x0, w = self.panel_inner
        if self.auto is not None:
            text = t("chip.auto_left", n=self.auto["left"])
        elif self.state == FLY:
            text = t("lcx.hint_fly")
        elif self.state == COAST:
            text = t("lcx.hint_coast")
        else:
            text = t("lcx.hint_idle")
        y = self.hint_y
        for line in self._wrap(text, self.f_micro, w)[:2]:
            img = self.f_micro.render(line, True, ui.TEXT_FAINT)
            s.blit(img, (x0, y))
            y += self.f_micro.get_height() + 1

    def _fit(self, text, fonts, maxw, col):
        """Text in der ersten Schrift, die passt - sonst gekürzt mit '…'."""
        for f in fonts:
            if f.size(text)[0] <= maxw:
                return f.render(text, True, col)
        f = fonts[-1]
        while text and f.size(text + "…")[0] > maxw:
            text = text[:-1]
        return f.render(text + "…", True, col)

    @staticmethod
    def _wrap(text, font, maxw):
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

    # ----- Kopfzeile, Werkzeuge, Verlauf ---------------------------------------------
    def _draw_hud(self, s):
        W_ = self.width
        s.blit(self._hud_bg, (0, 0))
        pygame.draw.line(s, ui.mix(CD.COL_GOLD, self.accent, 0.35), (0, self.hud_h - 1), (W_, self.hud_h - 1), 1)
        cy = self.hud_h // 2
        pad = self.margin
        coin_r = max(6, int(self.hud_h * 0.26))
        coin = CD.symbol_surface("coin", coin_r * 2 + 4)
        s.blit(coin, coin.get_rect(midleft=(pad - 2, cy)))
        left = self.f_big.render(f"{t('cas.chips')}: {self._group(int(round(self.shown_chips)))}", True, ui.GOLD)
        s.blit(left, left.get_rect(midleft=(pad + coin_r * 2 + 6, cy)))
        right_txt = f"{t('cas.record')}: {self._group(self.score)}   ·   {t('cas.credits', n=lamabank.refills())}"
        right = self.f_small.render(right_txt, True, ui.TEXT_DIM)
        if right.get_width() > W_ * 0.42:
            right = self.f_tiny.render(right_txt, True, ui.TEXT_DIM)
        rr = right.get_rect(midright=(W_ - pad, cy))
        s.blit(right, rr)
        if self.round is None:
            return
        mid_txt = f"{t('cas.bet')}: {self._group(L.stake_total(self.round))}"
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
        if name == "music":
            return self.music_on
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
            icon = name if name != "music" or self.music_on else "mute"
            img = D.icon(icon, max(8, int(r.h * 0.6)), col)
            s.blit(img, img.get_rect(center=(r.x + r.h // 2, r.centery)))
            if self.tool_labels:
                lab = self.f_tiny.render(t(TOOL_LABEL[name]), True, ui.TEXT if (active or h) else ui.TEXT_DIM)
                s.blit(lab, lab.get_rect(midleft=(r.x + r.h - max(2, r.h // 8), r.centery)))

    def _draw_tooltip(self, s):
        if self.window is not None:
            return
        for name, r in self.r_tools:
            if r.collidepoint(self.mouse):
                charts.tooltip(s, self.mouse, ["%s (%s)" % (t(TOOL_LABEL[name]), TOOL_KEY[name])])
                return
        if self.r_half.collidepoint(self.mouse) and self.state == FLY:
            charts.tooltip(s, self.mouse, [t("lcx.half_tip") + " (H)"])

    def _draw_history(self, s):
        """Letzte Platzpunkte als Pillen oben rechts (neueste rechts)."""
        x = self.width - self.margin
        y = self.r_tools[0][1].y
        h = self.tool_h
        ph = int(h * 0.78)
        limit = self.tools_right + self.margin
        for m100 in reversed(self.hist):
            label = self._fmt_short(m100)
            w = self.f_tiny.size(label)[0] + int(12 * self.k) + 6
            if x - w < limit:
                break
            img = D.pill(w, ph, m100, label, self.fs_tiny)
            s.blit(img, (x - w, y + (h - ph) // 2))
            x -= w + 4

    # ----- Bedienleiste -----------------------------------------------------------
    def _draw_strip(self, s):
        s.blit(self._strip_bg, self.strip.topleft)
        pygame.draw.line(s, ui.mix(CD.COL_GOLD, self.accent, 0.35), self.strip.topleft, self.strip.topright, 2)
        hot = self.mouse
        st = self.state
        # Gesamteinsatz
        r = self.r_total
        pygame.draw.rect(s, (16, 12, 26), r, border_radius=max(6, r.h // 4))
        pygame.draw.rect(s, ui.BORDER, r, 1, border_radius=max(6, r.h // 4))
        lbl = self._fit(t("lcx.total"), (self.f_tiny, self.f_micro), r.w - 4, ui.TEXT_DIM)
        s.blit(lbl, lbl.get_rect(midtop=(r.centerx, r.y + 2)))
        total = L.stake_total(self.round) if self.round is not None else self._total()
        col = ui.RED if (self.round is None and total > lamabank.balance()) else ui.GOLD
        val = self._fit(self._group(total), (self.f_btn, self.f_tiny), r.w - 4, col)
        s.blit(val, val.get_rect(midbottom=(r.centerx, r.bottom - 3)))
        # Hälfte
        can_half = (st == FLY and self.round is not None and L.mult_at(self.round["t"]) >= L.MIN_CASH
                    and any(L.riding(b) > 0 and not b["hm"] for b in self.round["bets"]))
        self._btn(s, self.r_half, t("lcx.half_all") if self.half_label else "", on=can_half,
                  hot=self.r_half.collidepoint(hot), icon="half")
        # Status in der Mitte
        x0, x1 = self.status_x
        if self.auto is not None:
            status, scol = t("chip.auto_left", n=self.auto["left"]), self.accent
        elif st == COUNT:
            status, scol = t("lcx.ready"), ui.TEXT_DIM
        elif st == COAST:
            status, scol = t("lcx.coast"), ui.TEXT_DIM
        else:
            status, scol = "", ui.TEXT_DIM
        if status and x1 - x0 > 30:
            img = self._fit(status, (self.f_btn, self.f_tiny, self.f_micro), x1 - x0, scol)
            s.blit(img, img.get_rect(center=((x0 + x1) // 2, self.r_go.centery)))
        # Hauptknopf
        go = self.r_go
        if st == FLY and self.round is not None:
            m = L.mult_at(self.round["t"])
            now = 0
            for b in self.round["bets"]:
                now += L.riding(b) * m // 100
            self._btn(s, go, t("lcx.cash_all") + " " + self._group(now), primary=True, on=m >= L.MIN_CASH,
                      hot=go.collidepoint(hot))
        elif st == COAST:
            self._btn(s, go, t("lcx.ff"), primary=not self.ff and self.auto is None, on=True,
                      hot=go.collidepoint(hot), icon="ff")
        elif self.auto is not None:
            self._btn(s, go, t("chip.auto_stop"), primary=True, hot=go.collidepoint(hot), danger=True)
        else:
            ready = st == IDLE or (st == END and self.end_t >= READ_T)
            self._btn(s, go, t("lcx.play"), primary=True,
                      on=ready and not self.broke and self.window is None and self.round is None,
                      hot=go.collidepoint(hot), icon="play")

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
            isz = max(8, int(r.h * 0.56))
            img = D.icon(icon, isz, col)
            if label:
                fonts = (self.f_tiny, self.f_micro) if small else (self.f_btn, self.f_tiny, self.f_micro)
                limg = self._fit(label, fonts, max(10, r.w - isz - 14), col)
                total = isz + 5 + limg.get_width()
                ix = r.centerx - total // 2
                s.blit(img, img.get_rect(midleft=(ix, r.centery)))
                s.blit(limg, limg.get_rect(midleft=(ix + isz + 5, r.centery)))
            else:
                s.blit(img, img.get_rect(center=r.center))
            return
        if label:
            fonts = (self.f_tiny, self.f_micro) if small else (self.f_btn, self.f_tiny, self.f_micro)
            img = self._fit(label, fonts, r.w - 6, col)
            s.blit(img, img.get_rect(center=(tx, r.centery)))

    # ----- Effekte, Hinweise -------------------------------------------------------
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
            sh = self.f_big.render(p["text"], True, (10, 6, 16))
            if f > 0.6:
                a = int(255 * (1 - (f - 0.6) / 0.4))
                img.set_alpha(a)
                sh.set_alpha(a)
            y = p["y"] - 40 * self.k * (1 - (1 - f) * (1 - f))
            r = img.get_rect(center=(int(p["x"]), int(y)))
            r.clamp_ip(self.stage)
            s.blit(sh, r.move(1, 1))
            s.blit(img, r)

    def _draw_banner(self, s):
        title, sub, left, dur = self.banner
        st = self.stage
        k = self.k
        grow = min(1.0, (dur - left) / 0.18)
        pulse = 0.5 + 0.5 * math.sin(self.time * 8)
        col = ui.mix(CD.COL_GOLD, (255, 255, 255), 0.4 * pulse)
        timg = self._fit(title, (self.f_huge, self.f_mult, self.f_big), st.w - 24, col)
        simg = self._fit(sub, (self.f_big, self.f_btn, self.f_tiny), st.w - 24, ui.TEXT)
        pw = min(st.w - 8, max(timg.get_width(), simg.get_width()) + int(40 * k))
        ph = timg.get_height() + simg.get_height() + int(26 * k)
        panel = pygame.Rect(0, 0, int(pw * (0.7 + 0.3 * grow)), int(ph * (0.7 + 0.3 * grow)))
        panel.center = (st.centerx, st.y + int(st.h * 0.32))
        layer = pygame.Surface(s.get_size(), pygame.SRCALPHA)
        ui.draw_panel(layer, panel, accent_top=CD.COL_GOLD)
        if grow >= 1.0:
            layer.blit(timg, timg.get_rect(midtop=(panel.centerx, panel.y + int(8 * k))))
            layer.blit(simg, simg.get_rect(midbottom=(panel.centerx, panel.bottom - int(8 * k))))
        if left < 0.4:
            layer.set_alpha(int(255 * left / 0.4))
        s.blit(layer, (0, 0))

    def _draw_toast(self, s):
        text, left = self.toast
        img = self._fit(text, (self.f_small, self.f_tiny), self.width - 40, ui.TEXT)
        box = img.get_rect()
        box.inflate_ip(24, 12)
        box.midbottom = (self.stage.centerx, self.stage.bottom - self.margin)
        box.clamp_ip(pygame.Rect(4, 0, self.width - 8, self.height))
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

    # ----- Auto-Fenster (über der Bühne) --------------------------------------------
    def _draw_auto(self, s):
        r = self.auto_r
        ui.draw_panel(s, r, accent_top=self.accent)
        k = self.k
        pad = max(6, int(10 * k))
        x0, w = r.x + pad, r.w - 2 * pad
        hot = self.mouse
        icon = D.icon("auto", max(10, self.r_auto_close.h - 4), self.accent)
        s.blit(icon, icon.get_rect(midleft=(x0, self.r_auto_close.centery)))
        title = self._fit(t("chip.auto_title"), (self.f_big, self.f_btn),
                          self.r_auto_close.x - x0 - icon.get_width() - 10, ui.TEXT)
        s.blit(title, title.get_rect(midleft=(x0 + icon.get_width() + 6, self.r_auto_close.centery)))
        self._btn(s, self.r_auto_close, "", hot=self.r_auto_close.collidepoint(hot), icon="close")
        labels = (t("chip.auto_rounds"), t("chip.auto_stop_win"), t("chip.auto_stop_loss"))
        values = (self.auto_rounds, self.auto_win, self.auto_loss)
        running = self.auto is not None
        for row, (ly, rects) in enumerate(self.auto_rows):
            img = self._fit(labels[row], (self.f_tiny, self.f_micro), w, ui.TEXT if self.auto_focus == row
                            else ui.TEXT_DIM)
            s.blit(img, (x0, ly))
            opts = (ROUND_OPTS, LIMIT_OPTS, LIMIT_OPTS)[row]
            for j, rr in enumerate(rects):
                v = opts[j]
                lab = str(v) if row == 0 else (t("chip.auto_off") if v == 0 else CD.fmt_amount(v))
                self._btn(s, rr, lab, active=v == values[row], on=not running or v == values[row],
                          hot=rr.collidepoint(hot) and not running, small=True)
        tr = self.r_auto_turbo
        self._btn(s, tr, t("chip.turbo"), active=self.turbo, hot=tr.collidepoint(hot) or self.auto_focus == 3,
                  icon="turbo")
        parts = []
        for i in self._active():
            parts.append(self._fmt(self.targets[i]))
        txt = t("lcx.auto_sum", n=self.auto_rounds, b=self._group(self._total()), x=" / ".join(parts))
        y = self.auto_sum_y
        for line in self._wrap(txt, self.f_tiny, w)[:2]:
            img = self.f_tiny.render(line, True, ui.RED if self._total() > lamabank.balance() else ui.TEXT_DIM)
            s.blit(img, (x0, y))
            y += self.f_tiny.get_height()
        note = t("lcx.auto_note")
        for line in self._wrap(note, self.f_micro, w):
            if y + self.f_micro.get_height() >= self.r_auto_start.y - 4:
                break
            img = self.f_micro.render(line, True, ui.TEXT_FAINT)
            s.blit(img, (x0, y))
            y += self.f_micro.get_height()
        if running:
            self._btn(s, self.r_auto_start, t("chip.auto_stop"), primary=True, danger=True,
                      hot=self.r_auto_start.collidepoint(hot))
        else:
            self._btn(s, self.r_auto_start, t("chip.auto_start"), primary=True, on=not self.broke,
                      hot=self.r_auto_start.collidepoint(hot))

    # ----- Statistik- und Fairness-Fenster ---------------------------------------------
    def _window_rect(self):
        W_, H, k, m = self.width, self.height, self.k, self.margin
        if self.window == "fair":
            w, h = min(W_ - 2 * m, int(720 * k)), min(H - 2 * m, int(540 * k))
        else:
            w, h = min(W_ - 2 * m, int(680 * k)), min(H - 2 * m, int(500 * k))
        rect = pygame.Rect(0, 0, w, h)
        rect.center = (W_ // 2, H // 2)
        cb = max(20, int(28 * k))
        close = pygame.Rect(rect.right - self.ppad - cb, rect.y + self.ppad, cb, cb)
        return rect, close

    def _draw_window(self, s):
        s.blit(self._overlay, (0, 0))
        rect, close = self._window_rect()
        ui.draw_panel(s, rect, accent_top=self.accent)
        pad = self.ppad
        icon = D.icon(self.window, max(10, close.h - 6), self.accent)
        s.blit(icon, icon.get_rect(midleft=(rect.x + pad, close.centery)))
        title = t("chip.stats") if self.window == "stats" else t("chip.fair_title")
        title += "  ·  Lama Crash"
        img = self._fit(title, (self.f_big, self.f_btn), close.x - rect.x - icon.get_width() - 3 * pad, ui.TEXT)
        s.blit(img, img.get_rect(midleft=(rect.x + pad + icon.get_width() + pad // 2, close.centery)))
        self._btn(s, close, "", hot=close.collidepoint(self.mouse), icon="close")
        body = pygame.Rect(rect.x + pad, close.bottom + pad, rect.w - 2 * pad, rect.bottom - close.bottom - 2 * pad)
        if self.window == "stats":
            self._draw_stats(s, body)
        else:
            self._draw_fair(s, body)

    def _xint(self, name):
        v = self.stats["x"].get(name)
        return v if isinstance(v, int) and not isinstance(v, bool) and v >= 0 else 0

    def _draw_stats(self, s, body):
        st = self.stats
        k = self.k
        rtp = "–"
        if st["wagered"] > 0:
            p = (st["paid"] * 20000 + st["wagered"]) // (st["wagered"] * 2)
            rtp = "%s%s%02d %%" % (self._group(p // 100), self._sep(), p % 100)
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
            (t("lcx.st_top"), self._fmt(self._xint("top")) if self._xint("top") else "–", self.accent),
            (t("lcx.st_half"), self._group(self._xint("half")), ui.TEXT),
            (t("lcx.st_close"), self._group(self._xint("close")), CD.COL_GOLD),
        ]
        cols = 4
        gap = max(4, int(8 * k))
        tw = (body.w - (cols - 1) * gap) // cols
        th = self.f_tiny.get_height() + self.f_big.get_height() + max(8, int(12 * k))
        for i, (lab, val, col) in enumerate(tiles):
            r = pygame.Rect(body.x + (i % cols) * (tw + gap), body.y + (i // cols) * (th + gap), tw, th)
            pygame.draw.rect(s, ui.mix(ui.PANEL, (0, 0, 0), 0.25), r, border_radius=max(4, int(8 * k)))
            pygame.draw.rect(s, ui.BORDER, r, 1, border_radius=max(4, int(8 * k)))
            limg = self._fit(lab, (self.f_tiny, self.f_micro), tw - 8, ui.TEXT_DIM)
            s.blit(limg, (r.x + 5, r.y + max(3, int(4 * k))))
            vimg = self._fit(val, (self.f_big, self.f_btn, self.f_tiny), tw - 8, col)
            s.blit(vimg, vimg.get_rect(bottomleft=(r.x + 5, r.bottom - max(3, int(4 * k)))))
        y = body.y + 3 * (th + gap) + gap
        half = (body.w - gap) * 45 // 100
        left = pygame.Rect(body.x, y, half, body.bottom - y)
        right = pygame.Rect(left.right + gap, y, body.right - left.right - gap, body.bottom - y)
        # Bilanzkurve
        curve = st["curve"]
        lab = t("chip.st_curve", n=len(curve)) if curve else t("chip.st_empty")
        img = self._fit(lab, (self.f_tiny, self.f_micro), left.w, ui.TEXT_DIM)
        s.blit(img, (left.x, left.y))
        cy = left.y + img.get_height() + 4 + charts._fnt(10).get_height()
        chart = pygame.Rect(left.x, cy, left.w, left.bottom - cy)
        if len(curve) >= 2 and chart.h > 20:
            vals = [0] + list(curve) if len(curve) < chipstats.CURVE else list(curve)
            charts.line(s, chart, vals, color=self.accent, fmt=lambda v: self._group(int(v)))
        # Verteilung der Platzpunkte gegen die Erwartung
        hits = self.stats["x"]["hits"]
        total = 0
        for v in hits:
            total += v
        lab = t("lcx.st_dist")
        img = self._fit(lab, (self.f_tiny, self.f_micro), right.w, ui.TEXT_DIM)
        s.blit(img, (right.x, right.y))
        area = pygame.Rect(right.x, right.y + img.get_height() + 4, right.w,
                           right.h - img.get_height() - 4)
        leg = self._fit(t("lcx.st_legend"), (self.f_micro,), area.w, ui.TEXT_FAINT)
        s.blit(leg, leg.get_rect(bottomleft=(area.x, area.bottom)))
        area.h -= leg.get_height() + 4
        if total <= 0 or area.h < 24:
            if area.h >= 14:
                img = self._fit(t("lcx.st_none"), (self.f_small, self.f_tiny), area.w, ui.TEXT_DIM)
                s.blit(img, img.get_rect(center=area.center))
            return
        exp = [total * float(L.bucket_prob(i)) for i in range(len(L.BUCKETS))]
        top = max(max(hits), max(exp))
        labels = [self._bucket_label(i) for i in range(len(L.BUCKETS))]
        cols = [ui.mix(D.pill_color(lo)[0], ui.PANEL, 0.2) for lo in L.BUCKETS]
        lsz = max(9, int(10 * k))
        hits_r = charts.bars(s, area, hits, labels=labels, colors=cols, max_value=top, hover=self.stats_hover,
                             label_size=lsz)
        fnt_h = ui.font(lsz).get_height() + 2
        val_h = ui.font(lsz, bold=True).get_height() + 2
        plot_h = max(8, area.h - fnt_h - val_h)
        pts = []
        hover = None
        for cell, i in hits_r:
            yy = area.y + val_h + plot_h - plot_h * (exp[i] / top)
            pts.append((cell.centerx, int(yy)))
            if cell.collidepoint(self.mouse):
                hover = i
        if len(pts) >= 2:
            pygame.draw.lines(s, CD.COL_GOLD, False, pts, 2)
        for p in pts:
            pygame.draw.circle(s, CD.COL_GOLD, p, 3)
            pygame.draw.circle(s, (40, 26, 8), p, 1)
        self.stats_hover = hover
        if hover is not None:
            e = int(exp[hover] * 10 + 0.5)
            charts.tooltip(s, self.mouse, [labels[hover],
                                           t("lcx.st_bar", n=hits[hover], e="%d%s%d" % (e // 10, self._sep(), e % 10))])

    def _bucket_label(self, i):
        lo = L.BUCKETS[i]
        if i + 1 >= len(L.BUCKETS):
            return self._fmt_short(lo) + "+"
        return self._fmt_short(lo) + "–" + L.fmt_short(L.BUCKETS[i + 1], self._sep())

    def _draw_fair(self, s, body):
        k = self.k
        gap = max(4, int(7 * k))
        y = body.y
        for line in self._wrap(t("lcx.fair_info"), self.f_micro, body.w)[:3]:
            img = self.f_micro.render(line, True, ui.TEXT_DIM)
            s.blit(img, (body.x, y))
            y += self.f_micro.get_height()
        y += gap
        rows = []
        if self.round is not None:
            rows.append((t("lcx.fair_current"), self.round["h"], ui.TEXT))
        rows.append((t("chip.fair_next"), fairness.commit(self.fair["server"]), self.accent))
        for lab, hx, col in rows:
            img = self._fit(lab, (self.f_tiny, self.f_micro), body.w, ui.TEXT_DIM)
            s.blit(img, (body.x, y))
            y += img.get_height()
            y = self._draw_hex(s, hx, body.x, y, body.w, col)
            y += 2
        bh = max(22, int(28 * k))
        lab = f"{t('chip.fair_client')}: {self.fair['client']}   ·   {t('chip.fair_nonce')} {self.fair['nonce']}"
        bw = min(self.f_btn.size(t("chip.fair_new_client"))[0] + int(20 * k), body.w // 2)
        self.r_fair_new = pygame.Rect(body.right - bw, y, bw, bh)
        img = self._fit(lab, (self.f_tiny, self.f_micro), body.w - bw - gap, ui.TEXT)
        s.blit(img, img.get_rect(midleft=(body.x, y + bh // 2)))
        self._btn(s, self.r_fair_new, t("chip.fair_new_client"), on=self.round is None,
                  hot=self.r_fair_new.collidepoint(self.mouse))
        y += bh + gap
        pygame.draw.line(s, ui.BORDER, (body.x, y), (body.right, y), 1)
        y += gap
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
        order = self._fair_order()
        self.fair_scroll = max(0, min(self.fair_scroll, len(order) - nvis))
        if self.fair_sel in order:
            pos = order.index(self.fair_sel)
            if pos < self.fair_scroll:
                self.fair_scroll = pos
            elif pos >= self.fair_scroll + nvis:
                self.fair_scroll = pos - nvis + 1
        for row, idx in enumerate(order[self.fair_scroll:self.fair_scroll + nvis]):
            e = self.fairlog[idx]
            r = pygame.Rect(lst.x, lst.y + row * rh, lst.w, rh - 2)
            sel = idx == self.fair_sel
            pygame.draw.rect(s, ui.BTN_SEL if sel or r.collidepoint(self.mouse) else ui.mix(ui.PANEL, (0, 0, 0), 0.2),
                             r, border_radius=max(3, rh // 4))
            if sel:
                pygame.draw.rect(s, self.accent, r, 1, border_radius=max(3, rh // 4))
            o = e.get("o") if isinstance(e.get("o"), dict) else {}
            x = o.get("x")
            res = self._fmt_short(x) if isinstance(x, int) and not isinstance(x, bool) and x >= 100 else "?"
            num = self.f_tiny.render("#%d" % e["n"], True, ui.TEXT_DIM)
            s.blit(num, num.get_rect(midleft=(r.x + 5, r.centery)))
            bg, fg = D.pill_color(x if isinstance(x, int) and not isinstance(x, bool) else 100)
            rimg = self.f_tiny.render(res, True, ui.mix(bg, (255, 255, 255), 0.35))
            s.blit(rimg, rimg.get_rect(midright=(r.right - 5, r.centery)))
            hx0 = r.x + 5 + max(num.get_width(), self.f_tiny.size("#0000")[0]) + 6
            hx1 = r.right - 5 - max(rimg.get_width(), self.f_tiny.size("×00,00")[0]) - 6
            if hx1 - hx0 > 20:
                himg = self._fit(fairness.short(e["h"], 10), (self.f_mono,), hx1 - hx0, ui.TEXT_FAINT)
                s.blit(himg, himg.get_rect(midleft=(hx0, r.centery)))
            chk = self.fair_checks.get(e["n"])
            if chk is not None:
                isz = rh // 2
                img = D.icon("check" if chk[0] else "cross", isz, ui.GREEN if chk[0] else ui.RED)
                s.blit(img, (r.right - rimg.get_width() - 10 - isz, r.y + (r.h - isz) // 2))
            self.r_fair_rows.append((idx, r))
        if len(order) > nvis:
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
        if isinstance(o.get("x"), int) and not isinstance(o.get("x"), bool):
            head += "  ·  " + t("lcx.fair_round", x=self._fmt(o["x"]))
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
        if chk is None:
            return
        ok, crash, marks = chk
        isz = bh * 2 // 3
        img = D.icon("check" if ok else "cross", isz, ui.GREEN if ok else ui.RED)
        s.blit(img, (self.r_fair_verify.right + 6, y + (bh - isz) // 2))
        msg = t("chip.fair_ok") if ok else t("chip.fair_bad")
        timg = self._fit(msg, (self.f_btn, self.f_tiny), max(10, det.right - self.r_fair_verify.right - isz - 14),
                         ui.GREEN if ok else ui.RED)
        s.blit(timg, timg.get_rect(midleft=(self.r_fair_verify.right + isz + 10, y + bh // 2)))
        y += bh + 6
        info = t("lcx.fair_calc", x=self._fmt(crash))
        img = self._fit(info, (self.f_tiny, self.f_micro), w, ui.TEXT)
        s.blit(img, (x, y))
        y += img.get_height() + 4
        ph = det.bottom - y
        if ph >= 24 and w >= 40:
            s.blit(D.curve_preview(w, min(ph, int(w * 0.6)), crash, marks), (x, y))
