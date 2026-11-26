# -*- coding: utf-8 -*-
"""
lamaplinko.py
=============
Lama Plinko - Kugel-Spiel mit Lama-Chips: Das Lama oben am Brett lässt goldene
Wollknäuel durch 8 bis 16 Reihen Stifte fallen. An jedem Stift springt die
Kugel nach links oder rechts (je 50 %); das Fach unten bestimmt den
Multiplikator. Außen ist es heiß (selten, hoch), in der Mitte kühl.

- 8-16 Reihen (←/→) und Risiko Niedrig/Mittel/Hoch (1-3), beides gesperrt,
  solange Kugeln fliegen. 27 feste Tabellen mit ~97 % Auszahlungsquote
  (``lamaplinko_logic``), Rand bei 16 Reihen ×16 / ×110 / ×1000.
- Einsatz je Kugel 1 ... 1000. Leertaste/Klick = werfen, gedrückt halten =
  Dauerfeuer, bis zu 50 Kugeln gleichzeitig; Turbo (T) halbiert die Flugzeit.
- Jede Kugel ist eine eigene **Fairness-Runde** (Fenster F): Der Prüfcode
  (SHA-256 des Server-Seeds) steht vor dem Wurf fest, der Seed wird nach der
  Landung gezeigt und lässt sich nachrechnen - mit Mini-Vorschau des Wegs.
- **Statistik** (S): Kacheln, Bilanzkurve und die Treffer-Verteilung je
  Reihenzahl gegen die erwartete Binomialkurve.
- **Auto-Modus** (A): 10 ... 1000 Kugeln, Stopp bei Gewinn/Verlust ab ...

Buchung: Der Weg der Kugel kommt beim Abwurf aus dem Fairness-Zufall - das
Ergebnis steht damit schon fest und wird sofort mit ``lamabank.bet_and_pay``
gebucht (ein Speichern inkl. Verlauf, Fairness-Log und Statistik). Gezeigt
wird erst bei der Landung (``pending``, angezeigte Chips wie bei Lama
Crossing); Erfolge fallen ebenfalls bei der Landung, ``on_exit`` holt sie für
noch fliegende Kugeln nach. Highscore = Höchststand von 1000 +
Lama-Plinko-Bilanz; ``game_over`` wird nie gesetzt.

Die Flugbahn folgt dem Pfad Stift für Stift: Bogensprünge mit Schwerkraft
(Parabel, Scheitel bei ~30 %), kleines kosmetisches Zittern aus ``self.rng``
(beim Abwurf gezogen), die Position ist eine reine Funktion der Flugzeit -
Spur und Auflösungswechsel brauchen keinen Zwischenstand.

Steuerung (alle Tasten roh, ohne die p1/p2-Belegung - wie Lama Crossing):
Leertaste/Enter/Klick auf WERFEN oder aufs Brett = werfen (halten =
Dauerfeuer), ←/→ = Reihen, 1-3 = Risiko, ↑/↓/+/-/Mausrad = Einsatz,
A = Auto, S = Statistik, F = Fairness, T = Turbo, Esc = Fenster schließen
bzw. Auto stoppen. In den Fenstern: Auto ↑/↓ Zeile, ←/→ Wert, Enter =
START/STOPP; Statistik ←/→ Reihenzahl; Fairness ↑/↓/Mausrad = Kugel,
Enter = Nachrechnen, C = neuer Client-Seed.
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
from . import lamaplinko_draw as D
from . import lamaplinko_logic as L

KEY = L.KEY

DROP_T = 0.30          # vom Maul des Lamas bis zum ersten Stift
BOUNCE = 0.24          # Sprunghöhe über dem Stift (Anteil am Reihenabstand)
FLASH_T = 0.32         # Leuchtdauer eines getroffenen Stifts
BUMP_T = 0.32          # Fach wippt nach einem Treffer
NOD_T = 0.30           # Lama nickt beim Abwurf
HOLD_DELAY = 0.32      # gehaltene Taste: erst danach Dauerfeuer
HOLD_BALLS = 30        # Dauerfeuer: so viele Kugeln sind dann gleichzeitig unterwegs
AUTO_RATE = 0.24       # Abstand im Auto-Modus (Turbo: halb so lang)
TRAIL = 6              # Spurpunkte je Kugel
TRAIL_DT = 0.024       # Flugzeit zwischen zwei Spurpunkten
TICK_GAP = 0.045       # kleinster Abstand zweier Stift-Klicks (Ton)
BIG_MULT = 1000        # ab x10: Münzregen
HUGE_MULT = 10000      # ab x100: großer Gewinn (Banner, Konfetti)
AUTO_BALLS = (10, 25, 50, 100, 250, 500, 1000)
AUTO_LIMITS = (0, 100, 500, 1000, 5000, 10000, 50000)
RISK_KEYS = ("1", "2", "3")


def _ri(v):
    """Kaufmännisch runden (x.5 nach oben) - anders als Pythons round() in
    allen Sprachen gleich (Web/Rust: floor(v + 0.5))."""
    return int(math.floor(v + 0.5))


def seg_time(rows):
    """Flugzeit von Stiftreihe zu Stiftreihe (mehr Reihen = kürzere Sprünge)."""
    return 0.165 - (rows - 8) * 0.005


def flight_time(rows):
    """Gesamte Flugzeit einer Kugel (ohne Turbo)."""
    return DROP_T + rows * seg_time(rows)


def fire_rate(rows):
    """Abstand zweier Kugeln im Dauerfeuer (ohne Turbo): Flugzeit / HOLD_BALLS.

    Damit sind beim Halten bei jeder Reihenzahl ~30 Kugeln gleichzeitig auf dem
    Brett (16 Reihen: 2,30 s / 30 = 0,0767 s, 8 Reihen: 1,62 s / 30 = 0,054 s) -
    der Erfolg "Wollknäuel-Regen" (25 gleichzeitig) ist so sicher erreichbar.
    Turbo halbiert Flugzeit und Takt, die Zahl bleibt gleich.
    """
    return flight_time(rows) / HOLD_BALLS


class LamaPlinkoGame(Game):
    match_manual = True       # jede Kugel ist ein Match (Game.log_match)
    name = "Lama Plinko"
    highscore_key = KEY
    supports_multiplayer = False

    # ===================================================== Aufbau / Reset
    def reset(self):
        if getattr(self, "balls", None):
            self._flush_flying()
        self.game_over = False
        lamabank.load()
        self.rng = random.Random()
        rows = self._opt("rows", L.DEFAULT_ROWS)
        self.rows = rows if rows in L.ROWS else L.DEFAULT_ROWS
        risk = self._opt("risk", L.DEFAULT_RISK)
        self.risk = risk if risk in L.RISKS else L.DEFAULT_RISK
        bet = self._opt("bet", 10)
        self.bet = bet if bet in L.BETS else L.BETS[0]
        self.turbo = self._opt("turbo", False) is True
        ab = self._opt("auto_balls", 25)
        self.auto_balls = ab if ab in AUTO_BALLS else 25
        aw = self._opt("auto_win", 0)
        self.auto_win = aw if aw in AUTO_LIMITS else 0
        al = self._opt("auto_loss", 0)
        self.auto_loss = al if al in AUTO_LIMITS else 0
        extra = lamabank.get_extra(KEY, {})
        extra = extra if isinstance(extra, dict) else {}
        self.hist = L.clean_history(extra.get("hist"))
        self.fair = fairness.clean_state(extra.get("fair"), self.rng)
        self.fairlog = fairness.clean_log(extra.get("fairlog"))
        self.stats = chipstats.clean(extra.get("stats"))
        self.carry = L.clean_carry(extra.get("carry"))   # Rest-Hundertstel (siehe _drop)
        self.stats["x"]["hits"] = L.clean_hits(self.stats["x"].get("hits"))
        if extra.get("fair") != self.fair:
            # neue Seeds sofort festschreiben: ein gezeigter Prüfcode gilt auch nach einem Neustart
            lamabank.set_extra(KEY, self._extra())
        # Angezeigter Stand (bei der Landung nachgeführt) - gespeichert wird beim Abwurf
        self.hist_shown = list(self.hist)
        self.fairlog_shown = [dict(e) for e in self.fairlog]
        self.stats_shown = chipstats.clean(self.stats)
        self.stats_shown["x"]["hits"] = L.clean_hits(self.stats["x"].get("hits"))
        self.balls = []
        self.pending = 0              # Gewinne fliegender Kugeln (gebucht, nicht gezeigt)
        self.shown_chips = float(lamabank.balance())
        self.score = lamabank.score_for(KEY)
        self.rec_shown = self.score
        self.shown_ledger = lamabank.ledger(KEY)
        self.sess = {"balls": 0, "net": 0, "best": 0}   # diese Sitzung (bei der Landung gezählt)
        self.auto = None              # laufender Auto-Modus
        self.panel = None             # offenes Fenster: "auto" | "stats" | "fair"
        self.auto_focus = 0
        self.stats_rows = self.rows
        self.stats_hover = None
        self.fair_sel = 0
        self.fair_scroll = 0
        self.fair_checks = {}         # nonce -> (Prüfcode ok, Ergebnis ok, Pfad)
        self.hold = False             # Leertaste/Maus auf WERFEN gehalten
        self.hold_t = 0.0
        self.fire_cd = 0.0
        self.auto_cd = 0.0
        self.tick_cd = 0.0
        self.nod = 0.0
        self.particles = []
        self.coins_fx = []
        self.popups = []
        self.banner = None
        self.toast = None
        self.shake = 0.0
        self.hist_anim = 1.0
        self.time = 0.0
        self.mouse = (-1, -1)
        self.broke = False
        self._tick_cd = 0.0
        self._tick_n = 0              # Zähl-Klick der Chips (Tonhöhe ohne Zufall)
        self._lang = get_language()
        self._layout()
        self._new_board_fx()
        self._check_broke()
        if not self.broke:
            self._say(t("lpl.hint"), 3.6)

    def on_surface_changed(self):
        D.clear_cache()
        self._layout()

    def on_exit(self):
        """Verlassen: Erfolge noch fliegender Kugeln nachholen (gebucht ist schon alles)."""
        self._flush_flying()
        self.score = lamabank.score_for(KEY)

    @property
    def wants_escape(self):
        return self.panel is not None or self.auto is not None

    def _flush_flying(self):
        for b in self.balls:
            self._ball_achievements(b)
        self.balls = []
        self.pending = 0

    def _new_board_fx(self):
        self.peg_flash = [[0.0] * (r + 3) for r in range(self.rows)]
        self.slot_bump = [0.0] * (self.rows + 1)

    # ===================================================== Layout
    def _layout(self):
        W, H = self.width, self.height
        self.k = k = min(W / 800.0, H / 600.0)
        self.f_tiny = ui.font(max(11, int(13 * k)))
        self.f_small = ui.font(max(12, int(15 * k)))
        self.f_btn = ui.font(max(12, int(15 * k)), bold=True)
        self.f_big = ui.font(max(15, int(22 * k)), bold=True)
        self.f_huge = ui.font(max(24, int(46 * k)), bold=True)
        self.f_mono = ui.font(max(10, int(13 * k)), mono=True)
        self.margin = m = max(6, int(12 * k))
        self.hud_h = max(28, int(42 * k))
        self.strip_h = max(46, int(64 * k))
        self.strip = pygame.Rect(0, H - self.strip_h, W, self.strip_h)
        self.region = pygame.Rect(0, self.hud_h, W, self.strip.y - self.hud_h)
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
        self._dim = pygame.Surface((W, H), pygame.SRCALPHA)
        self._dim.fill((6, 4, 12, 150))
        # --- Seitenleisten: links Risiko + Reihen, rechts Verlauf
        self.side_w = side = max(92, int(150 * k))
        top = self.region.y + m
        self.left = pygame.Rect(m, top, side, self.region.h - 2 * m)
        self.right = pygame.Rect(W - m - side, top, side, self.region.h - 2 * m)
        pad = max(5, int(9 * k))
        bh = max(22, int(30 * k))
        x = self.left.x + pad
        w = self.left.w - 2 * pad
        y = self.left.y + pad + self.f_tiny.get_height() + 3
        self.r_risks = []
        for i in range(3):
            self.r_risks.append(pygame.Rect(x, y, w, bh))
            y += bh + 4
        self.lbl_rows_y = y + pad // 2
        y = self.lbl_rows_y + self.f_tiny.get_height() + 3
        self.r_rows_m = pygame.Rect(x, y, bh, bh)
        self.r_rows_p = pygame.Rect(x + w - bh, y, bh, bh)
        self.r_rows_v = pygame.Rect(self.r_rows_m.right + 3, y, self.r_rows_p.x - self.r_rows_m.right - 6, bh)
        self.info_y = y + bh + pad
        self.pill_h = max(16, int(22 * k))
        # --- Bedienleiste: Einsatz links, Fenster-Knöpfe Mitte, WERFEN rechts
        sh = self.strip_h
        bh = max(30, int(sh * 0.62))
        by = self.strip.y + (sh - bh) // 2
        lbl_w = max(self.f_tiny.size(t("cas.bet"))[0], self.f_btn.size("1000")[0]) + int(14 * k)
        self.r_minus = pygame.Rect(m, by, bh, bh)
        self.r_bet = pygame.Rect(self.r_minus.right + 4, by, lbl_w, bh)
        self.r_plus = pygame.Rect(self.r_bet.right + 4, by, bh, bh)
        pad = int(22 * max(0.7, k))
        drop_w = max(self.f_btn.size(t("lpl.drop"))[0], self.f_btn.size(t("chip.auto_stop"))[0]) + pad
        drop_w = max(drop_w, int(130 * k))
        self.r_drop = pygame.Rect(W - m - drop_w, by, drop_w, bh)
        gap = max(4, int(7 * k))
        n_icons = 4
        room = self.r_drop.x - self.r_plus.right - 2 * m
        ib = min(bh, (room - (n_icons - 1) * gap) // n_icons)
        x0 = self.r_plus.right + m + (room - (n_icons * ib + (n_icons - 1) * gap)) // 2
        iy = by + (bh - ib) // 2
        self.r_icons = [pygame.Rect(x0 + i * (ib + gap), iy, ib, ib) for i in range(n_icons)]
        self._board_layout()
        self._panel_layout()

    def _board_layout(self):
        """Brett-Geometrie für die aktuelle Reihenzahl (Stifte, Fächer, Lama)."""
        k, m = self.k, self.margin
        n = self.rows
        bx0 = self.left.right + m
        bx1 = self.right.x - m
        self.cx = (bx0 + bx1) / 2.0
        self.llama_n = max(32, int(62 * k))
        self.foot_y = self.region.y + int(self.llama_n * 0.95) + 2   # Ohren knapp unter dem HUD
        self.top = self.foot_y + max(8, int(14 * k))
        avail_h = self.region.bottom - m - 3 - self.top
        gap = min((bx1 - bx0 - 10) / (n + 2.6), avail_h / (n + 0.36))
        self.gap = gap = max(8.0, gap)
        self.peg_r = max(2, int(gap * 0.12))
        self.ball_r = max(3, int(gap * 0.28))
        self.slot_w = max(6, int(gap - max(2, gap * 0.1)))
        self.slot_h = max(10, int(gap * 0.78))
        self.slot_top = int(self.top + (n - 1) * gap + gap * 0.55)
        self.slot_font = max(8, int(gap * 0.44))
        # Lama: Maul knapp links über dem Einwurf
        self.foot_x = self.cx - D.LLAMA_MOUTH[0] * self.llama_n + D.LLAMA_PIVOT[0] * self.llama_n \
            - gap * 0.35
        mx, my = D.mouth_pos(self.llama_n, 0, self.foot_x, self.foot_y)
        # kleines Knäuel im Maul; beim Abwurf wächst es auf volle Größe
        self.mouth_r = max(3, min(self.ball_r, int(self.llama_n * 0.085)))
        self.mouth = (mx + self.mouth_r * 0.55, my + self.mouth_r * 0.2)
        # Brett-Umriss (Trapez) + Stifte relativ zur Brett-Box
        half_top = gap * 1.6
        half_bot = (n + 2) / 2.0 * gap + gap * 0.25
        y0 = self.top - gap * 0.55
        y1 = self.slot_top + self.slot_h + 6
        self.board_box = pygame.Rect(int(self.cx - half_bot) - 4, int(y0) - 4,
                                     int(2 * half_bot) + 8, int(y1 - y0) + 8)
        bb = self.board_box
        self.plate_pts = [(self.cx - half_top - bb.x, y0 - bb.y), (self.cx + half_top - bb.x, y0 - bb.y),
                          (self.cx + half_bot - bb.x, y1 - bb.y), (self.cx - half_bot - bb.x, y1 - bb.y)]
        pegs = []
        for r in range(n):
            for j in range(r + 3):
                x, y = self._peg_xy(r, j)
                pegs.append((_ri(x) - bb.x, _ri(y) - bb.y))
        self.pegs_local = pegs
        self.slot_rects = [self._slot_rect(i) for i in range(n + 1)]

    def _panel_layout(self):
        """Fenster (Auto, Statistik, Fairness): mittig; Auto so hoch wie nötig,
        Statistik und Fairness so groß wie möglich."""
        W, H, k, m = self.width, self.height, self.k, self.margin
        self.pan_head = max(30, int(42 * k))
        pw = min(W - 2 * m, max(int(660 * k), 456))
        ph = min(H - 2 * m, max(int(480 * k), 340))
        if self.panel == "auto":
            ph = min(ph, self.pan_head + m // 2 + self._auto_content_h() + m)
        self.pan = pygame.Rect((W - pw) // 2, (H - ph) // 2, pw, ph)
        cb = self.pan_head - 10
        self.r_close = pygame.Rect(self.pan.right - 6 - cb, self.pan.y + 5, cb, cb)

    # ===================================================== Geometrie
    def _peg_xy(self, r, j):
        """Mittelpunkt von Stift j in Reihe r (Reihe r hat r+3 Stifte)."""
        return self.cx + (j - (r + 2) / 2.0) * self.gap, self.top + r * self.gap

    def _slot_x(self, i):
        return self.cx + (i - self.rows / 2.0) * self.gap

    def _slot_rect(self, i):
        r = pygame.Rect(0, 0, self.slot_w, self.slot_h)
        r.midtop = (_ri(self._slot_x(i)), self.slot_top)
        return r

    def _contact(self, r, k, jit):
        """Aufprallpunkt der Kugel über dem Stift (r, k+1) - mit Zittern."""
        x = self.cx + (k - r / 2.0 + jit) * self.gap
        y = self.top + r * self.gap - (self.peg_r + self.ball_r) * 0.92
        return x, y

    def _landing(self, k, jit):
        return self._slot_x(k) + jit * self.gap * 0.6, self.slot_top + self.ball_r * 0.15

    def _ball_radius(self, t):
        """Radius einer Kugel nach der Flugzeit t (wächst aus dem Maul heraus)."""
        f = t / (DROP_T * 0.7)
        if f >= 1.0:
            return self.ball_r
        return self.mouth_r + int((self.ball_r - self.mouth_r) * max(0.0, f))

    def _ball_pos(self, b, t):
        """Position einer Kugel nach der Flugzeit t (reine Funktion von t)."""
        n = b["rows"]
        if t <= 0:
            return self.mouth
        if t < DROP_T:
            p = t / DROP_T
            x1, y1 = self._contact(0, 0, b["jit"][0])
            x0, y0 = self.mouth
            return x0 + (x1 - x0) * p, y0 + (y1 - y0) * p * p
        st = seg_time(n)
        u = (t - DROP_T) / st
        seg = min(n - 1, int(u))
        p = min(1.0, u - seg)
        ka = b["pre"][seg]
        xa, ya = self._contact(seg, ka, b["jit"][seg])
        kb = ka + b["path"][seg]
        if seg < n - 1:
            xb, yb = self._contact(seg + 1, kb, b["jit"][seg + 1])
        else:
            xb, yb = self._landing(kb, b["jit"][n])
        hgt = BOUNCE * self.gap * b["hb"][seg]
        dy = yb - ya
        a = 2.0 * hgt + 2.0 * math.sqrt(hgt * hgt + hgt * dy)
        bq = dy + a
        return xa + (xb - xa) * p, ya - a * p + bq * p * p

    # ===================================================== Einstellungen
    def _opt(self, key, default):
        sec = self.settings.get(KEY) if isinstance(self.settings, dict) else None
        if isinstance(sec, dict) and key in sec:
            return sec[key]
        return default

    def _set_opt(self, key, value):
        if not isinstance(self.settings, dict):
            return
        sec = self.settings.setdefault(KEY, {})
        if sec.get(key) != value:
            sec[key] = value
            settings_mod.save_settings(self.settings)

    # ===================================================== Konto
    def _extra(self):
        return {"hist": list(self.hist), "fair": dict(self.fair), "fairlog": list(self.fairlog),
                "stats": self.stats, "carry": self.carry}


    def _avail(self):
        """Angezeigtes Guthaben (ohne die Gewinne noch fliegender Kugeln)."""
        return max(0, lamabank.balance() - self.pending)

    def _clamp_bet(self):
        chips = self._avail()
        if self.bet > chips:
            ok = [b for b in L.BETS if b <= chips]
            self.bet = ok[-1] if ok else L.BETS[0]

    def _check_broke(self):
        was = self.broke
        self.broke = not self.balls and lamabank.is_broke(KEY)
        if self.broke and not was:
            self.play_sound("gameover")
            self.panel = None
            self.auto = None

    def _take_credit(self):
        if lamabank.refill_if_broke(KEY):
            self.play_sound("powerup")
            ui.spawn_burst(self.width // 2, self.height // 2, ui.GOLD, n=26)
        self.broke = False
        self.shown_chips = float(lamabank.balance())
        self._clamp_bet()

    def _say(self, text, secs=1.8):
        self.toast = [text, secs]

    def _sep(self):
        return t("lpl.decimal")

    def _fmt(self, m100):
        return "×" + L.fmt_short(m100, self._sep())

    def _rtp_text(self):
        fr = L.expected_return(self.rows, self.risk)
        v = (fr.numerator * 20000 + fr.denominator) // (2 * fr.denominator)
        return f"{v // 100}{self._sep()}{v % 100:02d}"

    def _risk_name(self, risk):
        return t("lpl.risk." + risk)

    # ===================================================== Abwurf
    def _speed(self):
        return 2.0 if self.turbo else 1.0

    def _drop(self, quiet=False):
        """Eine Kugel werfen: Ergebnis ziehen, buchen, Flug starten. True = geworfen."""
        if self.broke:
            return False
        if len(self.balls) >= L.MAX_BALLS:
            if not quiet:
                self._say(t("lpl.max_balls", n=L.MAX_BALLS), 1.2)
            return False
        bet = self.bet
        if bet > self._avail():
            if not quiet:
                self._say(t("cas.not_enough"))
                self.play_sound("hit")
            return False
        rows, risk = self.rows, self.risk
        entry = fairness.begin(self.fair)
        fairness.advance(self.fair, self.rng)
        path = L.drop_path(fairness.entry_rand(KEY, entry), rows)
        slot = L.slot_of(path)
        m100 = L.mult100(rows, risk, slot)
        # Rest-Hundertstel-Übertrag: x0,4 bei Einsatz 1 zahlt sonst nie etwas
        win, self.carry = L.payout_carry(bet, rows, risk, slot, self.carry)
        outcome = {"rows": rows, "risk": risk, "slot": slot, "m": m100, "bet": bet, "win": win}
        fairness.record(self.fairlog, entry, outcome)
        chipstats.record(self.stats, bet, win, m100)
        net = win - bet
        self.log_match(net, 1 if net > 0 else 0 if net < 0 else 2,
                       {"bet": bet, "win": win, "x": m100, "rw": rows, "lv": risk})
        L.add_hit(self.stats["x"]["hits"], rows, slot)
        self.hist = (self.hist + [m100])[-L.HISTORY:]
        if not lamabank.bet_and_pay(bet, win, KEY, KEY, self._extra()):
            return False                  # (kann nach der Prüfung oben nicht passieren)
        self.score = lamabank.score_for(KEY)
        self.pending += win
        pre = [0]
        for d in path:
            pre.append(pre[-1] + d)
        # kosmetisches Zittern (nur Anzeige - das Ergebnis steht schon fest)
        jit = [self.rng.uniform(-0.06, 0.06) for _ in range(rows + 1)]
        hb = [self.rng.uniform(0.82, 1.18) for _ in range(rows)]
        spin = self.rng.randrange(12)
        ball = dict(rows=rows, risk=risk, bet=bet, win=win, m=m100, slot=slot, path=path, pre=pre,
                    jit=jit, hb=hb, spin=spin, t=0.0, hit=0, log=fairness.make_entry(entry, outcome),
                    msg=None)
        self.balls.append(ball)
        self.nod = NOD_T
        self.play_sound("move")
        if self.auto is not None:
            self.auto["net"] += win - bet
        if len(self.balls) >= L.RAIN_BALLS:
            self.ach_event("lpl_rain")
        return True

    def _start_hold(self):
        if self.auto is not None:
            self._stop_auto(None)
            return
        self.hold = True
        self.hold_t = 0.0
        self.fire_cd = 0.0
        self._drop()

    # ===================================================== Auto-Modus
    def _start_auto(self):
        if self.broke or self.auto is not None:
            return
        if self.bet > self._avail():
            self._say(t("cas.not_enough"))
            self.play_sound("hit")
            return
        self.auto = {"left": self.auto_balls, "net": 0}
        self.auto_cd = 0.0
        self.panel = None
        self.hold = False
        self.play_sound("select")

    def _stop_auto(self, msg_key):
        """Auto beenden. msg_key wird gezeigt, sobald die letzte Kugel gelandet ist."""
        if self.auto is None:
            return
        self.auto = None
        if msg_key is None:
            self.play_sound("click")
            return
        if self.balls:
            self.balls[-1]["msg"] = msg_key
        else:
            self._say(t(msg_key), 2.4)

    def _auto_step(self):
        a = self.auto
        if a["left"] <= 0:
            self._stop_auto("chip.auto_done")
            return
        if self.bet > self._avail():
            if self.balls:
                return                     # erst landen lassen - vielleicht reicht es dann
            self._stop_auto("chip.auto_no_chips")
            self._clamp_bet()
            return
        if len(self.balls) >= L.MAX_BALLS:
            return
        if not self._drop(quiet=True):
            return
        a["left"] -= 1
        if self.auto_win and a["net"] >= self.auto_win:
            self._stop_auto("chip.auto_hit_win")
        elif self.auto_loss and -a["net"] >= self.auto_loss:
            self._stop_auto("chip.auto_hit_loss")
        elif a["left"] <= 0:
            self._stop_auto("chip.auto_done")

    # ===================================================== Landung
    @staticmethod
    def match_text(key, value):
        """Anzeige im Matches-Screen: Risiko-Stufe statt interner Kennung."""
        if key == "lv" and value in ("low", "medium", "high"):
            return t("lpl.risk." + value)
        return None

    def _land(self, b):
        slot, m100, win = b["slot"], b["m"], b["win"]
        self.pending = max(0, self.pending - win)
        self.shown_ledger += win - b["bet"]
        self.rec_shown = min(self.score, max(self.rec_shown, lamabank.START_CHIPS + self.shown_ledger))
        self.hist_shown = (self.hist_shown + [m100])[-L.HISTORY:]
        self.hist_anim = 0.0
        self.sess["balls"] += 1
        self.sess["net"] += win - b["bet"]
        self.sess["best"] = max(self.sess["best"], m100)
        chipstats.record(self.stats_shown, b["bet"], win, m100)
        L.add_hit(self.stats_shown["x"]["hits"], b["rows"], slot)
        self.fairlog_shown.append(b["log"])
        del self.fairlog_shown[:-fairness.LOG]
        if 0 <= slot < len(self.slot_bump):
            self.slot_bump[slot] = BUMP_T
        r = self.slot_rects[slot] if slot < len(self.slot_rects) else self._slot_rect(slot)
        col = D.slot_color(m100)
        self._spawn(r.centerx, r.y, 6 + min(14, m100 // 200), col, 160, True, 2.6, 0.5, 520)
        big = m100 >= BIG_MULT
        self._popup(r.centerx, r.y - 6 * self.k, self._fmt(m100), col if m100 >= 100 else ui.TEXT_DIM,
                    1.0 + 0.4 * big, big)
        self._land_sound(m100)
        if m100 >= HUGE_MULT:
            title = t("cas.s.jackpot") if (b["rows"] == 16 and b["risk"] == "high" and L.is_edge(16, slot)) \
                else (t("cas.s.mega_win") if m100 >= 5 * HUGE_MULT else t("cas.s.big_win"))
            self.banner = dict(title=title, sub=f"{self._fmt(m100)}   ·   +{win}", t=0.0, dur=2.6)
            ui.spawn_confetti(self.width, self.height, 120 if m100 >= 5 * HUGE_MULT else 80)
            self._coin_shower(90 if m100 >= 5 * HUGE_MULT else 50)
            self.shake = 0.35
            self.rumble(240)
        elif big:
            self._coin_shower(18)
            self.rumble(90)
        self._ball_achievements(b)
        if b["msg"]:
            self._say(t(b["msg"]), 2.4)

    def _ball_achievements(self, b):
        if not L.is_edge(b["rows"], b["slot"]):
            return
        if b["rows"] >= L.EDGE_ROWS:
            self.ach_event("lpl_edge")
        if b["rows"] == 16 and b["risk"] == "high":
            self.ach_event("lpl_jackpot")

    def _land_sound(self, m100):
        if m100 >= HUGE_MULT:
            self.play_sound("level")
        elif m100 >= BIG_MULT:
            self.play_sound("win")
        elif m100 >= 200:
            audio.tone(784.0 if m100 >= 500 else 587.0, 0.09, self.settings, "square", 0.12)
        elif m100 >= 100:
            audio.tone(440.0, 0.07, self.settings, "square", 0.10)
        else:
            audio.tone(262.0, 0.06, self.settings, "sine", 0.10)

    # ===================================================== Einstellungen (Spiel)
    def _set_bet(self, direction):
        if self.auto is not None:
            return
        i = L.BETS.index(self.bet)
        j = max(0, min(len(L.BETS) - 1, i + direction))
        if j != i:
            self.bet = L.BETS[j]
            self._set_opt("bet", self.bet)
            self.play_sound("select")
            if self.bet > self._avail():
                self._say(t("cas.not_enough"), 1.2)

    def _locked(self):
        if self.balls or self.auto is not None:
            self._say(t("lpl.locked"), 1.4)
            self.play_sound("hit")
            return True
        return False

    def _set_rows(self, rows):
        rows = max(L.ROWS[0], min(L.ROWS[-1], rows))
        if rows == self.rows or self._locked():
            return
        self.rows = rows
        self._set_opt("rows", rows)
        self._board_layout()
        self._new_board_fx()
        self.play_sound("click")

    def _set_risk(self, i):
        risk = L.RISKS[i]
        if risk == self.risk or self._locked():
            return
        self.risk = risk
        self._set_opt("risk", risk)
        self.play_sound("click")

    def _toggle_turbo(self):
        self.turbo = not self.turbo
        self._set_opt("turbo", self.turbo)
        self._say(t("lpl.turbo_on") if self.turbo else t("lpl.turbo_off"), 1.2)
        self.play_sound("select")

    def _open(self, name):
        if self.panel == name:
            self.panel = None
            self.play_sound("click")
            return
        self.panel = name
        self.hold = False
        self._panel_layout()
        if name == "stats":
            self.stats_rows = self.rows
        elif name == "fair":
            self.fair_sel = 0
            self.fair_scroll = 0
        self.play_sound("select")

    def _new_client(self):
        if self._locked():
            return
        fairness.new_client(self.fair, self.rng)
        lamabank.set_extra(KEY, self._extra())
        self._say(t("lpl.client_new", c=self.fair["client"]), 1.8)
        self.play_sound("select")

    def _verify(self, e):
        """Nachrechnen: Prüfcode + Ergebnis aus Seed, Client-Seed und Nr. neu."""
        o = e.get("o") if isinstance(e.get("o"), dict) else {}
        rows = o.get("rows") if o.get("rows") in L.ROWS else None
        ok_hash = fairness.verify(e)
        path = L.path_from_seed(rows, e["s"], e["c"], e["n"]) if rows else []
        ok_res = bool(rows) and L.slot_of(path) == o.get("slot") \
            and o.get("risk") in L.RISKS and L.mult100(rows, o["risk"], L.slot_of(path)) == o.get("m")
        self.fair_checks[e["n"]] = (ok_hash, ok_res, path)
        self.play_sound("point" if ok_hash and ok_res else "hit")

    # ===================================================== Effekte
    def _spawn(self, x, y, n, col, speed, up, r, life, grav):
        for _ in range(n):
            a = self.rng.uniform(math.pi * 1.1, math.pi * 1.9) if up else self.rng.uniform(0, math.tau)
            sp = speed * self.rng.uniform(0.35, 1.0) * self.k
            self.particles.append(dict(x=x, y=y, vx=math.cos(a) * sp, vy=math.sin(a) * sp,
                                       r=r * self.k * self.rng.uniform(0.6, 1.2), col=col, t=0.0,
                                       life=life * self.rng.uniform(0.7, 1.1), g=grav * self.k))
        if len(self.particles) > 260:
            self.particles = self.particles[-260:]

    def _coin_shower(self, n=40):
        W = self.width
        for _ in range(n):
            self.coins_fx.append(dict(
                x=self.rng.uniform(W * 0.1, W * 0.9),
                y=self.rng.uniform(-self.height * 0.4, -10),
                vx=self.rng.uniform(-60, 60) * self.k,
                vy=self.rng.uniform(-80, 120) * self.k,
                spin=self.rng.uniform(0, math.tau),
                vs=self.rng.uniform(6, 14),
                r=self.rng.uniform(7, 12) * max(0.7, self.k)))
        if len(self.coins_fx) > 160:
            self.coins_fx = self.coins_fx[-160:]

    def _popup(self, x, y, text, color, dur=1.0, big=False):
        self.popups.append(dict(x=x, y=y, text=text, color=color, t=0.0, dur=dur, big=big))
        if len(self.popups) > 14:
            self.popups = self.popups[-14:]

    # ===================================================== Eingabe
    @staticmethod
    def _is_drop_key(key):
        """Leertaste und Enter werfen. Alle Tasten werden ROH ausgewertet (wie
        Lama Crossing und Lama-Slot) - nicht über die p1/p2-Belegung, sonst
        gingen A und S bei der Standardbelegung WASD nie."""
        return key in ("space", "Return", "KP_Enter")

    def handle_event(self, event):
        kind = event.kind
        if kind in (InputEvent.MOUSEMOVE, InputEvent.MOUSEDOWN,
                    InputEvent.MOUSEUP) and event.pos is not None:
            self.mouse = event.pos
        if kind == InputEvent.KEYUP and self._is_drop_key(event.key):
            self.hold = False
            return
        if kind == InputEvent.MOUSEUP:
            self.hold = False
            return
        if self.broke:
            if kind == InputEvent.MOUSEDOWN or (
                    kind == InputEvent.KEYDOWN and event.key in ("Return", "space", "KP_Enter")):
                self._take_credit()
            return
        if self.panel is not None:
            self._panel_event(event)
            return
        if kind == InputEvent.KEYDOWN:
            key = event.key
            if self._is_drop_key(key):
                if event.repeat:
                    if self.auto is None and not self.hold:
                        self.hold = True
                        self.hold_t = HOLD_DELAY
                else:
                    self._start_hold()
            elif key == "Escape":
                self._stop_auto(None)
            elif key == "Left":
                self._set_rows(self.rows - 1)
            elif key == "Right":
                self._set_rows(self.rows + 1)
            elif key in ("plus", "KP_Add", "Up", "equal"):
                self._set_bet(1)
            elif key in ("minus", "KP_Subtract", "Down"):
                self._set_bet(-1)
            elif key in RISK_KEYS or key in ("KP_1", "KP_2", "KP_3"):
                self._set_risk(int(key[-1]) - 1)
            elif event.repeat:
                return
            elif key in ("a", "A"):
                self._open("auto")
            elif key in ("s", "S"):
                self._open("stats")
            elif key in ("f", "F"):
                self._open("fair")
            elif key in ("t", "T"):
                self._toggle_turbo()
            return
        if kind == InputEvent.WHEEL:
            self._set_bet(1 if event.delta > 0 else -1)
            return
        if kind != InputEvent.MOUSEDOWN or event.button != 1:
            return
        pos = event.pos
        if self.r_drop.collidepoint(pos):
            self._start_hold()
        elif self.r_minus.collidepoint(pos):
            self._set_bet(-1)
        elif self.r_plus.collidepoint(pos):
            self._set_bet(1)
        elif self.r_rows_m.collidepoint(pos):
            self._set_rows(self.rows - 1)
        elif self.r_rows_p.collidepoint(pos):
            self._set_rows(self.rows + 1)
        else:
            for i, r in enumerate(self.r_risks):
                if r.collidepoint(pos):
                    self._set_risk(i)
                    return
            for name, r in zip(("auto", "stats", "fair", "turbo"), self.r_icons):
                if r.collidepoint(pos):
                    if name == "turbo":
                        self._toggle_turbo()
                    else:
                        self._open(name)
                    return
            if self.board_box.collidepoint(pos) and pos[1] < self.slot_top:
                self._start_hold()      # Klick aufs Brett wirft ebenfalls

    # ----- Fenster ---------------------------------------------------------
    def _panel_event(self, event):
        kind = event.kind
        name = self.panel
        if kind == InputEvent.KEYDOWN:
            key = event.key
            letter = {"auto": ("a", "A"), "stats": ("s", "S"), "fair": ("f", "F")}[name]
            if key == "Escape" or (key in letter and not event.repeat):
                self.panel = None
                self.play_sound("click")
                return
            if name == "auto":
                self._auto_key(key, event.repeat)
            elif name == "stats":
                if key == "Left":
                    self.stats_rows = max(L.ROWS[0], self.stats_rows - 1)
                elif key == "Right":
                    self.stats_rows = min(L.ROWS[-1], self.stats_rows + 1)
            elif name == "fair":
                self._fair_key(key)
            return
        if kind == InputEvent.WHEEL and name == "fair":
            self._fair_move(-1 if event.delta > 0 else 1)
            return
        if kind != InputEvent.MOUSEDOWN or event.button != 1:
            return
        pos = event.pos
        if self.r_close.collidepoint(pos) or not self.pan.collidepoint(pos):
            self.panel = None
            self.play_sound("click")
            return
        for rect, action in self._panel_hits():
            if rect.collidepoint(pos):
                action()
                return

    def _auto_key(self, key, repeat):
        rows = (AUTO_BALLS, AUTO_LIMITS, AUTO_LIMITS)
        if key == "Up":
            self.auto_focus = (self.auto_focus - 1) % 4
        elif key == "Down":
            self.auto_focus = (self.auto_focus + 1) % 4
        elif key in ("Left", "Right") and self.auto_focus < 3 and self.auto is None:
            d = -1 if key == "Left" else 1
            opts = rows[self.auto_focus]
            cur = (self.auto_balls, self.auto_win, self.auto_loss)[self.auto_focus]
            j = max(0, min(len(opts) - 1, opts.index(cur) + d))
            self._auto_set(self.auto_focus, opts[j])
        elif key in ("Left", "Right") and self.auto_focus == 3:
            self._toggle_turbo()
        elif key in ("t", "T") and not repeat:
            self._toggle_turbo()
        elif key in ("Return", "KP_Enter", "space") and not repeat:
            if self.auto is not None:
                self._stop_auto(None)
            else:
                self._start_auto()

    def _auto_set(self, row, value):
        if self.auto is not None:
            return
        if row == 0:
            self.auto_balls = value
            self._set_opt("auto_balls", value)
        elif row == 1:
            self.auto_win = value
            self._set_opt("auto_win", value)
        else:
            self.auto_loss = value
            self._set_opt("auto_loss", value)
        self.play_sound("click")

    def _fair_key(self, key):
        if key in ("Up", "Left"):
            self._fair_move(-1)
        elif key in ("Down", "Right"):
            self._fair_move(1)
        elif key in ("Return", "KP_Enter", "space"):
            e = self._fair_entry()
            if e is not None:
                self._verify(e)
        elif key in ("c", "C"):
            self._new_client()

    def _fair_entries(self):
        return list(reversed(self.fairlog_shown))

    def _fair_entry(self):
        es = self._fair_entries()
        if not es:
            return None
        self.fair_sel = max(0, min(len(es) - 1, self.fair_sel))
        return es[self.fair_sel]

    def _fair_move(self, d):
        es = self._fair_entries()
        if not es:
            return
        self.fair_sel = max(0, min(len(es) - 1, self.fair_sel + d))
        vis = self._fair_visible()
        if self.fair_sel < self.fair_scroll:
            self.fair_scroll = self.fair_sel
        elif self.fair_sel >= self.fair_scroll + vis:
            self.fair_scroll = self.fair_sel - vis + 1
        self.play_sound("move")

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
            step = max(abs(diff) * min(1.0, dt * 6.0), 60.0 * dt)
            self.shown_chips += math.copysign(min(abs(diff), step), diff)
            if diff > 0:
                self._tick_cd -= dt
                if self._tick_cd <= 0:
                    self._tick_cd = 0.07
                    self._tick_n += 1
                    audio.tone(1500.0 + (self._tick_n * 113) % 300, 0.025, self.settings, "square", 0.08)
        if self.toast:
            self.toast[1] -= dt
            if self.toast[1] <= 0:
                self.toast = None
        self.shake = max(0.0, self.shake - dt)
        self.nod = max(0.0, self.nod - dt)
        self.tick_cd = max(0.0, self.tick_cd - dt)
        self.hist_anim = min(1.0, self.hist_anim + dt / 0.22)
        if self.banner is not None:
            self.banner["t"] += dt
            if self.banner["t"] >= self.banner["dur"]:
                self.banner = None
        for row in self.peg_flash:
            for j in range(len(row)):
                if row[j] > 0:
                    row[j] = max(0.0, row[j] - dt)
        for i in range(len(self.slot_bump)):
            if self.slot_bump[i] > 0:
                self.slot_bump[i] = max(0.0, self.slot_bump[i] - dt)
        self._update_fx(dt)
        self._update_balls(dt)
        # Dauerfeuer (gehaltene Leertaste bzw. Maus auf WERFEN)
        if self.hold and self.auto is None and self.panel is None:
            self.hold_t += dt
            if self.hold_t >= HOLD_DELAY:
                # Rest aufheben statt neu setzen: der Takt stimmt bei jeder Bildrate
                self.fire_cd -= dt
                n = 0
                while self.fire_cd <= 0 and n < 3:
                    self.fire_cd += fire_rate(self.rows) / self._speed()
                    self._drop(quiet=True)
                    n += 1
                if self.fire_cd <= 0:
                    self.fire_cd = 0.0
        if self.auto is not None:
            self.auto_cd -= dt
            if self.auto_cd <= 0:
                self.auto_cd = AUTO_RATE / self._speed()
                self._auto_step()

    def _update_balls(self, dt):
        if not self.balls:
            return
        sp = self._speed()
        landed = []
        for b in self.balls:
            b["t"] += dt * sp
            n = b["rows"]
            st = seg_time(n)
            while b["hit"] < n and b["t"] >= DROP_T + b["hit"] * st:
                r = b["hit"]
                j = b["pre"][r] + 1
                if r < len(self.peg_flash) and j < len(self.peg_flash[r]):
                    self.peg_flash[r][j] = FLASH_T
                if self.tick_cd <= 0:
                    self.tick_cd = TICK_GAP
                    audio.tone(880.0 + r * 40.0, 0.025, self.settings, "sine", 0.05)
                b["hit"] += 1
            if b["t"] >= flight_time(n):
                landed.append(b)
        if landed:
            self.balls = [b for b in self.balls if b["t"] < flight_time(b["rows"])]
            for b in landed:
                self._land(b)
            if not self.balls:
                if self.auto is None:
                    self._clamp_bet()       # (im Auto bleibt der Einsatz fest)
                self._check_broke()

    def _update_fx(self, dt):
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

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        W, H = self.width, self.height
        glow_c = (int(self.cx), self.region.y + self.region.h * 11 // 20)   # unabhängig von den Reihen
        s.blit(D.background(W, H, glow_c, int(max(W, H) * 0.42), self.accent), (0, 0))
        ox = 0
        if self.shake > 0:
            ox = int(math.sin(self.time * 70) * 6 * self.k * self.shake / 0.35)
        self._draw_board(s, ox)
        self._draw_llama(s, ox)
        self._draw_balls(s, ox)
        self._draw_particles(s, ox)
        self._draw_left(s)
        self._draw_right(s)
        self._draw_hud(s)
        self._draw_strip(s)
        self._draw_fx(s)
        if self.banner is not None:
            self._draw_banner(s)
        if self.panel is None and not self.broke:
            self._draw_hover(s)
        if self.panel == "auto":
            self._draw_auto(s)
        elif self.panel == "stats":
            self._draw_stats(s)
        elif self.panel == "fair":
            self._draw_fair(s)
        # Hinweis ÜBER den Fenstern - sonst bleibt z.B. "Neuer Client-Seed"
        # im Fairness-Fenster unsichtbar
        if self.toast:
            self._draw_toast(s)
        if self.broke:
            self._draw_broke(s)

    # ----- Brett ------------------------------------------------------------
    def _draw_board(self, s, ox):
        bb = self.board_box
        plate = D.board_plate(self.plate_pts, bb.w, bb.h, self.accent)
        s.blit(plate, (bb.x + ox, bb.y))
        s.blit(D.peg_layer(bb.w, bb.h, self.pegs_local, self.peg_r), (bb.x + ox, bb.y))
        for r, row in enumerate(self.peg_flash):
            for j, f in enumerate(row):
                if f > 0:
                    level = max(1, min(8, int(8 * f / FLASH_T + 0.999)))
                    g = D.peg_glow(self.peg_r, level)
                    x, y = self._peg_xy(r, j)
                    s.blit(g, g.get_rect(center=(_ri(x) + ox, _ri(y))))
        hot = self._hover_slot()
        table = L.table(self.rows, self.risk)
        bump_px = self.gap * 0.28
        for i, rect in enumerate(self.slot_rects):
            m100 = table[i]
            img = D.slot_surface(rect.w, rect.h, m100, L.fmt_short(m100, self._sep()), self.slot_font,
                                 hot=i == hot)
            dy = 0
            if self.slot_bump[i] > 0:
                p = 1.0 - self.slot_bump[i] / BUMP_T
                dy = int(bump_px * math.sin(math.pi * p) * (1.0 - p * 0.5))
            s.blit(img, (rect.x + ox, rect.y + dy))

    def _draw_llama(self, s, ox):
        n = self.llama_n
        k = self.k
        # Sims unter dem Lama (endet knapp vor dem Einwurf)
        ledge_r = int(self.cx - self.gap * 0.55) + ox
        ledge = pygame.Rect(0, 0, int(n * 1.05), max(4, int(7 * k)))
        ledge.topright = (ledge_r, int(self.foot_y))
        pygame.draw.rect(s, ui.mix(self.accent, (0, 0, 0), 0.55), ledge.move(0, 2), border_radius=ledge.h // 2)
        pygame.draw.rect(s, ui.mix(self.accent, (40, 20, 10), 0.35), ledge, border_radius=ledge.h // 2)
        pygame.draw.line(s, ui.mix(self.accent, (255, 255, 255), 0.4), (ledge.x + 3, ledge.y),
                         (ledge.right - 3, ledge.y), 1)
        # Trichter über dem ersten Stift (das Wollknäuel fällt aus dem Maul hinein)
        fw = max(self.ball_r * 3 + 6, int(self.gap * 1.35))
        fbot = int(self.top - self.gap * 0.42)
        fh = max(8, min(int(self.gap * 1.0), fbot - int(self.mouth[1] + self.ball_r + 2)))
        fun = D.funnel(fw, fh, self.accent)
        s.blit(fun, (int(self.cx - fw / 2) + ox, fbot - fh))
        deg = 0
        if self.nod > 0:
            p = 1.0 - self.nod / NOD_T
            deg = -_ri(10 * math.sin(math.pi * p))
        img = D.llama_bow(n, deg, self.accent)
        x, y = D.llama_blit_pos(img, n, deg, self.foot_x + ox, self.foot_y)
        s.blit(img, (x, y))
        # nächstes Wollknäuel im Maul (wenn ein Wurf möglich ist)
        ready = not self.broke and self.nod <= 0 and len(self.balls) < L.MAX_BALLS \
            and self.bet <= self._avail()
        if ready:
            mx, my = self.mouth
            ball = D.yarn_ball(self.mouth_r, 0)
            s.blit(ball, (int(mx) - self.mouth_r - 1 + ox, int(my) - self.mouth_r - 1))

    def _draw_balls(self, s, ox):
        r = self.ball_r
        sprite_cache = {}
        for b in self.balls:
            t0 = b["t"]
            for i in range(TRAIL, 0, -1):
                tt = t0 - i * TRAIL_DT
                if tt <= 0:
                    continue
                x, y = self._ball_pos(b, tt)
                a = 150 * (TRAIL + 1 - i) // (TRAIL + 1)
                rr = max(1, self._ball_radius(tt) * (TRAIL + 2 - i) // (TRAIL + 2))
                dot = D.trail_dot(rr, a)
                s.blit(dot, (int(x) - rr - 1 + ox, int(y) - rr - 1))
        for b in self.balls:
            x, y = self._ball_pos(b, b["t"])
            step = (b["spin"] + int(b["t"] * 16)) % 12
            br = self._ball_radius(b["t"])
            if br != r:
                s.blit(D.yarn_ball(br, step), (int(x) - br - 1 + ox, int(y) - br - 1))
                continue
            img = sprite_cache.get(step)
            if img is None:
                img = sprite_cache[step] = D.yarn_ball(r, step)
            s.blit(img, (int(x) - r - 1 + ox, int(y) - r - 1))

    def _draw_particles(self, s, ox):
        for p in self.particles:
            rr = p["r"] * (1 - p["t"] / p["life"])
            if rr >= 0.5:
                pygame.draw.circle(s, p["col"], (int(p["x"] + ox), int(p["y"])), max(1, int(rr)))

    # ----- Seitenleisten ----------------------------------------------------
    def _draw_left(self, s):
        k = self.k
        box = self.left.copy()
        box.h = self.info_y - box.y + 2 * self.f_tiny.get_height() + int(14 * k)
        ui.draw_panel(s, box, accent_top=ui.mix(self.accent, CD.COL_GOLD, 0.3))
        pad = max(5, int(9 * k))
        locked = bool(self.balls) or self.auto is not None
        lbl = self.f_tiny.render(t("lpl.risk"), True, ui.TEXT_DIM)
        s.blit(lbl, (box.x + pad, box.y + pad - 2))
        for i, r in enumerate(self.r_risks):
            risk = L.RISKS[i]
            self._btn(s, r, f"{i + 1}  {self._risk_name(risk)}", active=risk == self.risk,
                      on=not locked or risk == self.risk, hot=r.collidepoint(self.mouse) and not locked)
        lbl = self.f_tiny.render(t("lpl.rows"), True, ui.TEXT_DIM)
        s.blit(lbl, (box.x + pad, self.lbl_rows_y - 2))
        self._icon_btn(s, self.r_rows_m, "left", on=not locked and self.rows > L.ROWS[0])
        self._icon_btn(s, self.r_rows_p, "right", on=not locked and self.rows < L.ROWS[-1])
        rv = self.r_rows_v
        pygame.draw.rect(s, (16, 12, 26), rv, border_radius=max(6, rv.h // 4))
        pygame.draw.rect(s, ui.BORDER, rv, 1, border_radius=max(6, rv.h // 4))
        val = self.f_btn.render(str(self.rows), True, ui.GOLD if not locked else ui.TEXT_DIM)
        s.blit(val, val.get_rect(center=rv.center))
        top = max(L.table(self.rows, self.risk))
        y = self.info_y
        for txt, col in ((t("lpl.max", m=L.fmt_short(top, self._sep())), CD.COL_GOLD),
                         (t("lpl.rtp", p=self._rtp_text()), ui.TEXT_DIM)):
            img = self.f_tiny.render(txt, True, col)
            if img.get_width() > box.w - 2 * pad:
                img = D.fit_font(txt, box.w - 2 * pad, self.f_tiny.get_height(), bold=False).render(txt, True, col)
            s.blit(img, (box.x + pad, y))
            y += self.f_tiny.get_height() + 2
        self._draw_session(s, box.bottom + self.margin)

    def _draw_session(self, s, top):
        """Kasten "Sitzung" unter den Einstellungen (nur wenn Platz ist)."""
        k = self.k
        pad = max(5, int(9 * k))
        lh = self.f_tiny.get_height()
        box = pygame.Rect(self.left.x, top, self.left.w, pad * 2 + lh + 3 * (lh + 3) + 2)
        if box.bottom > self.left.bottom:
            return
        ui.draw_panel(s, box, accent_top=ui.mix(self.accent, CD.COL_GOLD, 0.3))
        s.blit(self.f_tiny.render(t("lpl.session"), True, ui.TEXT_DIM), (box.x + pad, box.y + pad - 2))
        net = self.sess["net"]
        rows = ((t("chip.auto_balls"), str(self.sess["balls"]), ui.TEXT),
                (t("chip.st_net"), ("+" if net > 0 else "") + str(net),
                 ui.GREEN if net > 0 else (ui.RED if net < 0 else ui.TEXT)),
                (t("lpl.best"), self._fmt(self.sess["best"]) if self.sess["best"] else "–", CD.COL_GOLD))
        y = box.y + pad + lh + 2
        w = box.w - 2 * pad
        for lbl, val, col in rows:
            vi = self.f_btn.render(val, True, col)
            if vi.get_width() > w // 2:
                vi = D.fit_font(val, w // 2, self.f_tiny.get_height()).render(val, True, col)
            li = D.fit_font(lbl, w - vi.get_width() - 6, lh, bold=False).render(lbl, True, ui.TEXT_DIM)
            s.blit(li, li.get_rect(midleft=(box.x + pad, y + (lh + 3) // 2)))
            s.blit(vi, vi.get_rect(midright=(box.right - pad, y + (lh + 3) // 2)))
            y += lh + 3

    def _draw_right(self, s):
        k = self.k
        pad = max(5, int(9 * k))
        ph = self.pill_h
        n_vis = L.HISTORY
        head = self.f_tiny.get_height() + pad
        box = self.right.copy()
        box.h = min(self.right.h, head + pad + n_vis * (ph + 3))
        ui.draw_panel(s, box, accent_top=ui.mix(self.accent, CD.COL_GOLD, 0.3))
        lbl = self.f_tiny.render(t("chip.history"), True, ui.TEXT_DIM)
        s.blit(lbl, (box.x + pad, box.y + pad - 2))
        y0 = box.y + head + 2
        w = box.w - 2 * pad
        slide = (1.0 - self.hist_anim) * (ph + 3)
        clip = s.get_clip()
        s.set_clip(pygame.Rect(box.x, y0 - 2, box.w, box.bottom - y0))
        ghost = ui.mix(ui.PANEL, ui.BG_TOP, 0.45)
        for i in range(len(self.hist_shown), n_vis):
            y = int(y0 + i * (ph + 3))
            if y > box.bottom - ph:
                break
            pygame.draw.rect(s, ghost, (box.x + pad, y, w, ph - 2), border_radius=ph // 2)
        for i, m100 in enumerate(reversed(self.hist_shown)):
            y = int(y0 + i * (ph + 3) - slide)
            if y > box.bottom - ph:
                break
            img = D.pill(w, ph, m100, self._fmt(m100), max(9, int(ph * 0.62)))
            if i == 0 and self.hist_anim < 1.0:
                img = img.copy()
                img.set_alpha(int(255 * self.hist_anim))
            s.blit(img, (box.x + pad, y))
        s.set_clip(clip)

    # ----- HUD + Leiste -----------------------------------------------------
    def _draw_hud(self, s):
        W = self.width
        s.blit(self._hud_bg, (0, 0))
        pygame.draw.line(s, ui.mix(CD.COL_GOLD, self.accent, 0.35), (0, self.hud_h - 1), (W, self.hud_h - 1), 1)
        cy = self.hud_h // 2
        pad = self.margin
        coin_r = max(6, int(self.hud_h * 0.26))
        coin = CD.symbol_surface("coin", coin_r * 2 + 4)
        s.blit(coin, coin.get_rect(midleft=(pad - 2, cy)))
        left = self.f_big.render(f"{t('cas.chips')}: {_ri(self.shown_chips)}", True, ui.GOLD)
        s.blit(left, left.get_rect(midleft=(pad + coin_r * 2 + 6, cy)))
        right_txt = f"{t('cas.record')}: {self.rec_shown}   ·   {t('cas.credits', n=lamabank.refills())}"
        right = self.f_small.render(right_txt, True, ui.TEXT_DIM)
        if right.get_width() > W * 0.42:
            right = self.f_tiny.render(right_txt, True, ui.TEXT_DIM)
        rr = right.get_rect(midright=(W - pad, cy))
        s.blit(right, rr)
        parts = []
        if self.balls:
            parts.append(t("lpl.in_flight", n=len(self.balls)))
        if self.auto is not None:
            parts.append(t("chip.auto_left", n=self.auto["left"]))
        if self.turbo:
            parts.append(t("chip.turbo"))
        if not parts:
            return
        mid_txt = "   ·   ".join(parts)
        x0 = pad + coin_r * 2 + 6 + left.get_width() + pad
        x1 = rr.left - pad
        mid = self.f_small.render(mid_txt, True, ui.TEXT)
        if mid.get_width() > x1 - x0:
            mid = self.f_tiny.render(mid_txt, True, ui.TEXT)
        if mid.get_width() <= x1 - x0:
            s.blit(mid, mid.get_rect(center=((x0 + x1) // 2, cy)))

    def _draw_strip(self, s):
        s.blit(self._strip_bg, self.strip.topleft)
        pygame.draw.line(s, ui.mix(CD.COL_GOLD, self.accent, 0.35), self.strip.topleft, self.strip.topright, 2)
        hot = self.mouse
        free = self.auto is None
        self._btn(s, self.r_minus, "–", on=free and self.bet > L.BETS[0], hot=self.r_minus.collidepoint(hot))
        self._btn(s, self.r_plus, "+", on=free and self.bet < L.BETS[-1], hot=self.r_plus.collidepoint(hot))
        rb = self.r_bet
        pygame.draw.rect(s, (16, 12, 26), rb, border_radius=max(6, rb.h // 4))
        pygame.draw.rect(s, ui.BORDER, rb, 1, border_radius=max(6, rb.h // 4))
        lbl = self.f_tiny.render(t("cas.bet"), True, ui.TEXT_DIM)
        s.blit(lbl, lbl.get_rect(midtop=(rb.centerx, rb.y + 2)))
        col = ui.RED if self.bet > self._avail() else ui.GOLD
        val = self.f_btn.render(str(self.bet), True, col)
        s.blit(val, val.get_rect(midbottom=(rb.centerx, rb.bottom - 3)))
        names = ("auto", "stats", "fair", "turbo")
        letters = ("A", "S", "F", "T")
        for name, letter, r in zip(names, letters, self.r_icons):
            active = (name == "turbo" and self.turbo) or (name == "auto" and self.auto is not None)
            self._icon_btn(s, r, name, active=active, hot=r.collidepoint(hot), letter=letter)
        if self.auto is not None:
            self._btn(s, self.r_drop, t("chip.auto_stop"), primary=True, hot=self.r_drop.collidepoint(hot))
        else:
            ok = not self.broke and self.bet <= self._avail() and len(self.balls) < L.MAX_BALLS
            self._btn(s, self.r_drop, t("lpl.drop"), primary=True, on=ok,
                      hot=self.r_drop.collidepoint(hot) or self.hold)

    def _pulse(self, speed, phase=0.0):
        return 0.5 + 0.5 * math.sin(self.time * speed + phase)

    def _btn(self, s, rect, label, primary=False, on=True, hot=False, active=False):
        r = pygame.Rect(rect)
        rad = max(6, r.h // 4)
        if primary and on:
            base = ui.mix(self.accent, (0, 0, 0), 0.18)
            fill = ui.mix(base, (255, 255, 255), 0.12 if hot else 0.0)
            pygame.draw.rect(s, ui.mix(base, (0, 0, 0), 0.4), r.move(0, 2), border_radius=rad)
            pygame.draw.rect(s, fill, r, border_radius=rad)
            border = ui.mix(CD.COL_GOLD, (255, 255, 255), 0.35 * self._pulse(2.4))
            pygame.draw.rect(s, border, r, 2, border_radius=rad)
            col = (255, 255, 255)
        else:
            fill = ui.BTN_SEL if (on and (hot or active)) else ui.BTN
            pygame.draw.rect(s, fill, r, border_radius=rad)
            edge = CD.COL_GOLD if active else (ui.BORDER_LIGHT if on else ui.BORDER)
            pygame.draw.rect(s, edge, r, 2 if active else 1, border_radius=rad)
            col = ui.TEXT if on else ui.TEXT_FAINT
        img = self.f_btn.render(label, True, col)
        if img.get_width() > r.w - 8:
            img = self.f_tiny.render(label, True, col)
        if img.get_width() > r.w - 4:
            img = D.fit_font(label, r.w - 4, self.f_tiny.get_height(), bold=False).render(label, True, col)
        s.blit(img, img.get_rect(center=r.center))

    def _icon_btn(self, s, rect, name, on=True, hot=False, active=False, letter=None):
        r = pygame.Rect(rect)
        rad = max(5, r.h // 4)
        fill = ui.BTN_SEL if (on and (hot or active)) else ui.BTN
        pygame.draw.rect(s, fill, r, border_radius=rad)
        edge = CD.COL_GOLD if active else (ui.BORDER_LIGHT if on else ui.BORDER)
        pygame.draw.rect(s, edge, r, 2 if active else 1, border_radius=rad)
        col = (CD.COL_GOLD if active else ui.TEXT) if on else ui.TEXT_FAINT
        size = max(8, int(r.h * (0.52 if letter else 0.5)))
        img = D.icon(name, size, col)
        s.blit(img, img.get_rect(center=(r.centerx, r.centery - (r.h // 14 if letter else 0))))
        if letter and r.h >= 26:
            f = ui.font(max(8, int(r.h * 0.26)), bold=True)
            li = f.render(letter, True, ui.TEXT_DIM)
            s.blit(li, li.get_rect(bottomright=(r.right - 3, r.bottom - 1)))

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
            fnt = self.f_big if p["big"] else self.f_btn
            img = fnt.render(p["text"], True, p["color"])
            sh = fnt.render(p["text"], True, (10, 6, 16))
            if f > 0.6:
                a = int(255 * (1 - (f - 0.6) / 0.4))
                img.set_alpha(a)
                sh.set_alpha(a)
            y = p["y"] - (46 if p["big"] else 30) * self.k * (1 - (1 - f) ** 2)
            r = img.get_rect(midbottom=(int(p["x"]), int(y)))
            r.clamp_ip(pygame.Rect(2, 0, self.width - 4, self.height))
            s.blit(sh, r.move(1, 1))
            s.blit(img, r)

    def _draw_banner(self, s):
        b = self.banner
        f = b["t"] / b["dur"]
        k = self.k
        W = self.width
        grow = min(1.0, b["t"] / 0.18)
        timg = self.f_huge.render(b["title"], True, CD.COL_GOLD)
        if timg.get_width() > W - 40:
            timg = self.f_big.render(b["title"], True, CD.COL_GOLD)
        simg = self.f_big.render(b["sub"], True, ui.TEXT)
        if simg.get_width() > W - 40:
            simg = self.f_small.render(b["sub"], True, ui.TEXT)
        pw = min(W - 20, max(timg.get_width(), simg.get_width()) + int(48 * k))
        ph = timg.get_height() + simg.get_height() + int(30 * k)
        panel = pygame.Rect(0, 0, int(pw * (0.7 + 0.3 * grow)), int(ph * (0.7 + 0.3 * grow)))
        panel.center = (int(self.cx), self.top + int((self.slot_top - self.top) * 0.38))
        layer = pygame.Surface(s.get_size(), pygame.SRCALPHA)
        ui.draw_panel(layer, panel, accent_top=CD.COL_GOLD)
        if grow >= 1.0:
            layer.blit(timg, timg.get_rect(midtop=(panel.centerx, panel.y + int(10 * k))))
            layer.blit(simg, simg.get_rect(midbottom=(panel.centerx, panel.bottom - int(10 * k))))
        if f > 0.8:
            layer.set_alpha(int(255 * (1 - (f - 0.8) / 0.2)))
        s.blit(layer, (0, 0))

    def _draw_toast(self, s):
        text, left = self.toast
        img = self.f_small.render(text, True, ui.TEXT)
        if img.get_width() > self.width - 40:
            img = self.f_tiny.render(text, True, ui.TEXT)
        box = img.get_rect()
        box.inflate_ip(24, 12)
        box.midtop = (int(self.cx), int(self.top + (self.slot_top - self.top) * 0.16))
        box.clamp_ip(pygame.Rect(4, 0, self.width - 8, self.height))
        panel = pygame.Surface(box.size, pygame.SRCALPHA)
        pygame.draw.rect(panel, (12, 8, 20, 225), panel.get_rect(), border_radius=box.h // 2)
        pygame.draw.rect(panel, (*ui.mix(CD.COL_GOLD, self.accent, 0.4), 255),
                         panel.get_rect(), 1, border_radius=box.h // 2)
        if left < 0.3:
            panel.set_alpha(int(255 * left / 0.3))
            img.set_alpha(int(255 * left / 0.3))
        s.blit(panel, box)
        s.blit(img, img.get_rect(center=box.center))

    def _hover_slot(self):
        if self.panel is not None:
            return None
        for i, r in enumerate(self.slot_rects):
            if r.inflate(2, 6).collidepoint(self.mouse):
                return i
        return None

    def _draw_hover(self, s):
        i = self._hover_slot()
        if i is not None:
            m100 = L.mult100(self.rows, self.risk, i)
            lines = [self._fmt(m100) + "   ·   " + t("lpl.slot", n=i),
                     t("lpl.tip_chance", p=L.fmt_prob(self.rows, i, self._sep())),
                     t("lpl.tip_win", b=self.bet, n=L.fmt_short(self.bet * m100, self._sep()))]
            charts.tooltip(s, self.mouse, lines)
            return
        tips = ((self.r_icons[0], t("chip.auto_title") + "  (A)"), (self.r_icons[1], t("chip.stats") + "  (S)"),
                (self.r_icons[2], t("chip.fair_title") + "  (F)"), (self.r_icons[3], t("chip.turbo") + "  (T)"))
        for r, txt in tips:
            if r.collidepoint(self.mouse):
                charts.tooltip(s, self.mouse, [txt])
                return

    def _draw_broke(self, s):
        s.blit(self._overlay, (0, 0))
        cx, cy = self.width // 2, self.height // 2
        pw = min(self.width - 40, int(480 * max(0.8, self.k)))
        ph = int(200 * max(0.75, self.k))
        panel = pygame.Rect(cx - pw // 2, cy - ph // 2, pw, ph)
        ui.draw_panel(s, panel, accent_top=ui.RED)
        head = self.f_huge.render(t("cas.broke"), True, ui.RED)
        s.blit(head, head.get_rect(center=(cx, panel.y + ph * 0.28)))
        for i, (txt, col) in enumerate(((t("cas.broke_sub"), ui.TEXT_DIM),
                                        (t("cas.broke_restart", n=lamabank.START_CHIPS), ui.TEXT))):
            img = self.f_small.render(txt, True, col)
            if img.get_width() > pw - 16:
                img = self.f_tiny.render(txt, True, col)
            s.blit(img, img.get_rect(center=(cx, panel.y + ph * (0.58 + 0.2 * i))))

    # ===================================================== Fenster
    def _panel_frame(self, s, icon_name, title):
        s.blit(self._dim, (0, 0))
        ui.draw_panel(s, self.pan, accent_top=self.accent)
        k = self.k
        p = self.pan
        hy = p.y + self.pan_head // 2
        ic = D.icon(icon_name, max(12, int(self.pan_head * 0.5)), CD.COL_GOLD)
        s.blit(ic, ic.get_rect(midleft=(p.x + self.margin, hy)))
        timg = self.f_big.render(title, True, ui.TEXT)
        s.blit(timg, timg.get_rect(midleft=(p.x + self.margin + ic.get_width() + int(8 * k), hy)))
        self._icon_btn(s, self.r_close, "close", hot=self.r_close.collidepoint(self.mouse))
        pygame.draw.line(s, ui.BORDER, (p.x + 8, p.y + self.pan_head), (p.right - 8, p.y + self.pan_head), 1)
        return pygame.Rect(p.x + self.margin, p.y + self.pan_head + self.margin // 2,
                           p.w - 2 * self.margin, p.h - self.pan_head - self.margin)

    def _panel_hits(self):
        """Klickflächen des offenen Fensters: [(rect, aktion), ...]."""
        if self.panel == "auto":
            return self._auto_rects()[1]
        if self.panel == "stats":
            body = self._panel_body()
            r_m, r_p = self._stats_row_btns(body)
            return [(r_m, lambda: setattr(self, "stats_rows", max(L.ROWS[0], self.stats_rows - 1))),
                    (r_p, lambda: setattr(self, "stats_rows", min(L.ROWS[-1], self.stats_rows + 1)))]
        if self.panel == "fair":
            return self._fair_rects()[2]
        return []

    def _panel_body(self):
        p = self.pan
        return pygame.Rect(p.x + self.margin, p.y + self.pan_head + self.margin // 2,
                           p.w - 2 * self.margin, p.h - self.pan_head - self.margin)

    # ----- Auto -------------------------------------------------------------
    def _auto_metrics(self):
        k = self.k
        return dict(lh=self.f_tiny.get_height(), bh=max(22, int(30 * k)), gap=max(3, int(5 * k)),
                    row_gap=max(6, int(12 * k)), sep=max(8, int(14 * k)), sum_h=self.f_small.get_height(),
                    start_h=max(30, int(42 * k)), start_gap=max(6, int(10 * k)))

    def _auto_content_h(self):
        a = self._auto_metrics()
        return (3 * (a["lh"] + 3 + a["bh"] + a["row_gap"]) + a["bh"] + a["sep"] + a["sum_h"]
                + a["start_gap"] + a["start_h"])

    def _auto_rects(self):
        """Geometrie des Auto-Fensters: (Zeilen, Turbo, Summen-y, Start), Klickflächen."""
        body = self._panel_body()
        a = self._auto_metrics()
        lh, bh, gap = a["lh"], a["bh"], a["gap"]
        rows, hits = [], []
        y = body.y
        for ri, opts in enumerate((AUTO_BALLS, AUTO_LIMITS, AUTO_LIMITS)):
            y += lh + 3
            bw = (body.w - (len(opts) - 1) * gap) // len(opts)
            rects = []
            for j, v in enumerate(opts):
                r = pygame.Rect(body.x + j * (bw + gap), y, bw, bh)
                rects.append((r, v))
                hits.append((r, (lambda ri=ri, v=v: (setattr(self, "auto_focus", ri), self._auto_set(ri, v)))))
            rows.append((y - lh - 3, rects))
            y += bh + a["row_gap"]
        turbo = pygame.Rect(body.x, y, min(body.w, max(int(body.w * 0.36),
                                                       self.f_btn.size(t("chip.turbo"))[0] + 3 * bh + 10)), bh)
        hits.append((turbo, lambda: (setattr(self, "auto_focus", 3), self._toggle_turbo())))
        y += bh + a["sep"]
        sum_y = y
        y += a["sum_h"] + a["start_gap"]
        start_h = a["start_h"]
        start = pygame.Rect(body.x + body.w // 4, min(body.bottom - start_h, y), body.w // 2, start_h)
        hits.append((start, lambda: self._stop_auto(None) if self.auto is not None else self._start_auto()))
        return (rows, turbo, sum_y, start), hits

    def _draw_auto(self, s):
        self._panel_frame(s, "auto", t("chip.auto_title"))
        (rows, turbo, sum_y, start), _ = self._auto_rects()
        labels = (t("chip.auto_balls"), t("chip.auto_stop_win"), t("chip.auto_stop_loss"))
        cur = (self.auto_balls, self.auto_win, self.auto_loss)
        running = self.auto is not None
        for ri, (ly, rects) in enumerate(rows):
            col = ui.TEXT if self.auto_focus == ri else ui.TEXT_DIM
            img = self.f_tiny.render(labels[ri], True, col)
            s.blit(img, (rects[0][0].x, ly))
            for r, v in rects:
                txt = str(v) if (ri == 0 or v) else t("chip.auto_off")
                self._btn(s, r, txt, active=v == cur[ri], on=not running or v == cur[ri],
                          hot=r.collidepoint(self.mouse) and not running)
        self._toggle_btn(s, turbo, t("chip.turbo"), self.turbo, focus=self.auto_focus == 3)
        total = self.auto_balls * self.bet
        txt = t("lpl.auto_sum", n=self.auto_balls, b=self.bet, t=total)
        col = ui.RED if self.bet > self._avail() else ui.TEXT_DIM
        img = self.f_small.render(txt, True, col)
        if img.get_width() > self.pan.w - 2 * self.margin:
            img = self.f_tiny.render(txt, True, col)
        s.blit(img, img.get_rect(midtop=(self.pan.centerx, sum_y)))
        if running:
            self._btn(s, start, t("chip.auto_stop") + "  ·  " + t("chip.auto_left", n=self.auto["left"]),
                      primary=True, hot=start.collidepoint(self.mouse))
        else:
            self._btn(s, start, t("chip.auto_start"), primary=True, on=not self.broke,
                      hot=start.collidepoint(self.mouse))

    def _toggle_btn(self, s, r, label, on, focus=False):
        hot = r.collidepoint(self.mouse)
        pygame.draw.rect(s, ui.BTN_SEL if (hot or focus) else ui.BTN, r, border_radius=max(6, r.h // 4))
        pygame.draw.rect(s, CD.COL_GOLD if on else ui.BORDER_LIGHT, r, 2 if on else 1,
                         border_radius=max(6, r.h // 4))
        sw = pygame.Rect(0, 0, int(r.h * 1.5), int(r.h * 0.56))
        sw.midleft = (r.x + 8, r.centery)
        pygame.draw.rect(s, CD.COL_GOLD if on else (60, 60, 76), sw, border_radius=sw.h // 2)
        knob = sw.h // 2 - 2
        kx = sw.right - knob - 3 if on else sw.x + knob + 3
        pygame.draw.circle(s, (255, 255, 255), (kx, sw.centery), knob)
        ic = D.icon("turbo", max(8, int(r.h * 0.5)), CD.COL_GOLD if on else ui.TEXT_DIM)
        s.blit(ic, ic.get_rect(midleft=(sw.right + 6, r.centery)))
        img = self.f_btn.render(label, True, ui.TEXT)
        s.blit(img, img.get_rect(midleft=(sw.right + 10 + ic.get_width(), r.centery)))

    # ----- Statistik --------------------------------------------------------
    def _stats_row_btns(self, body):
        bh = max(18, int(24 * self.k))
        y = self._stats_dist_y(body)
        r_p = pygame.Rect(body.right - bh, y, bh, bh)
        r_m = pygame.Rect(r_p.x - bh - 4, y, bh, bh)
        return r_m, r_p

    def _stats_dist_y(self, body):
        tile_h = max(34, int(46 * self.k))
        return body.y + 3 * (tile_h + 4) + max(6, int(10 * self.k))

    def _draw_stats(self, s):
        body = self._panel_frame(s, "stats", t("chip.stats"))
        st = self.stats_shown
        k = self.k
        sep = self._sep()
        if st["rounds"] <= 0:
            img = self.f_small.render(t("chip.st_empty"), True, ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=body.center))
            return
        rtp = chipstats.rtp(st)
        rtp_txt = "–" if rtp is None else self._pct(st["paid"], st["wagered"])
        tiles = ((t("chip.st_rounds"), str(st["rounds"]), ui.TEXT),
                 (t("chip.st_wins"), str(st["wins"]), ui.GREEN),
                 (t("chip.st_losses"), str(st["losses"]), ui.RED),
                 (t("chip.st_wagered"), str(st["wagered"]), ui.TEXT),
                 (t("chip.st_paid"), str(st["paid"]), ui.TEXT),
                 (t("chip.st_net"), ("+" if st["net"] > 0 else "") + str(st["net"]),
                  ui.GREEN if st["net"] >= 0 else ui.RED),
                 (t("chip.st_best_win"), "+" + str(st["best_win"]), CD.COL_GOLD),
                 (t("chip.st_best_mult"), "×" + L.fmt_short(st["best_mult"], sep), CD.COL_GOLD),
                 (t("chip.st_rtp"), rtp_txt + " %", ui.TEXT))
        tile_h = max(34, int(46 * k))
        half = body.w // 2 - 4
        tw = (half - 2 * 4) // 3
        for i, (lbl, val, col) in enumerate(tiles):
            r = pygame.Rect(body.x + (i % 3) * (tw + 4), body.y + (i // 3) * (tile_h + 4), tw, tile_h)
            pygame.draw.rect(s, ui.mix(ui.PANEL, ui.BG_TOP, 0.35), r, border_radius=max(4, r.h // 5))
            li = D.fit_font(lbl, r.w - 6, self.f_tiny.get_height(), bold=False).render(lbl, True, ui.TEXT_DIM)
            s.blit(li, li.get_rect(midtop=(r.centerx, r.y + 3)))
            vi = D.fit_font(val, r.w - 6, self.f_btn.get_height()).render(val, True, col)
            s.blit(vi, vi.get_rect(midbottom=(r.centerx, r.bottom - 3)))
        # Bilanzkurve rechts
        cr = pygame.Rect(body.x + half + 8, body.y, body.w - half - 8, 3 * (tile_h + 4) - 4)
        pygame.draw.rect(s, ui.mix(ui.PANEL, ui.BG_TOP, 0.35), cr, border_radius=max(4, int(6 * k)))
        cap = t("chip.st_curve", n=len(st["curve"]))
        ci = D.fit_font(cap, cr.w - 8, self.f_tiny.get_height(), bold=False).render(cap, True, ui.TEXT_DIM)
        s.blit(ci, (cr.x + 6, cr.y + 3))
        plot = pygame.Rect(cr.x + 6, cr.y + ci.get_height() + 8, cr.w - 12, cr.h - ci.get_height() - 14)
        if len(st["curve"]) >= 2:
            col = ui.GREEN if st["curve"][-1] >= 0 else ui.RED
            charts.line(s, plot, [0] + st["curve"], color=col, fmt=lambda v: str(_ri(v)))
        # Treffer-Verteilung (gewählte Reihenzahl) gegen die erwartete Binomialkurve
        dy = self._stats_dist_y(body)
        n = self.stats_rows
        cap = t("lpl.st_dist", n=n)
        r_m, r_p = self._stats_row_btns(body)
        self._icon_btn(s, r_m, "left", on=n > L.ROWS[0], hot=r_m.collidepoint(self.mouse))
        self._icon_btn(s, r_p, "right", on=n < L.ROWS[-1], hot=r_p.collidepoint(self.mouse))
        cap_img = D.fit_font(cap, r_m.x - body.x - 8, self.f_small.get_height()).render(cap, True, ui.TEXT)
        s.blit(cap_img, cap_img.get_rect(midleft=(body.x, r_m.centery)))
        hits = st["x"]["hits"][str(n)]
        total = 0
        for v in hits:
            total += v
        area = pygame.Rect(body.x, r_m.bottom + 6, body.w, body.bottom - r_m.bottom - 6)
        leg = t("lpl.st_legend")
        li = D.fit_font(leg, area.w, self.f_tiny.get_height(), bold=False).render(leg, True, ui.TEXT_FAINT)
        s.blit(li, li.get_rect(bottomleft=(area.x, area.bottom)))
        area.h -= li.get_height() + 4
        if total <= 0:
            img = self.f_small.render(t("lpl.st_none", n=n), True, ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=area.center))
            return
        exp = [total * L.comb(n, i) / (2 ** n) for i in range(n + 1)]
        top = max(max(hits), max(exp))
        table = L.table(n, self.risk)
        cols = [ui.mix(D.slot_color(m), ui.PANEL, 0.25) for m in table]
        hover = None
        lbls = [str(i) for i in range(n + 1)] if area.w / (n + 1) >= 16 else None
        hits_r = charts.bars(s, area, hits, labels=lbls, colors=cols, max_value=top, hover=self.stats_hover,
                             label_size=max(9, int(10 * k)))
        # erwartete Kurve: Punkte auf Höhe des Erwartungswerts über den Balkenmitten
        fnt_h = ui.font(max(9, int(10 * k))).get_height() + 2 if lbls else 0
        val_h = ui.font(max(9, int(10 * k)), bold=True).get_height() + 2
        plot_h = max(8, area.h - fnt_h - val_h)
        pts = []
        for (cell, i) in hits_r:
            y = area.y + val_h + plot_h - plot_h * (exp[i] / top)
            pts.append((cell.centerx, int(y)))
            if cell.collidepoint(self.mouse):
                hover = i
        if len(pts) >= 2:
            pygame.draw.lines(s, CD.COL_GOLD, False, pts, 2)
        for p in pts:
            pygame.draw.circle(s, CD.COL_GOLD, p, 3)
            pygame.draw.circle(s, (40, 26, 8), p, 1)
        self.stats_hover = hover
        if hover is not None:
            e = exp[hover]
            ev = int(e * 10 + 0.5)
            charts.tooltip(s, self.mouse, [t("lpl.slot", n=hover) + "   ·   " + self._fmt(table[hover]),
                                           t("lpl.st_bar", n=hits[hover], e=f"{ev // 10}{sep}{ev % 10}")])

    def _pct(self, num, den):
        """Prozent mit zwei Nachkommastellen (ganzzahlig gerundet)."""
        v = (num * 20000 + den) // (2 * den)
        return f"{v // 100}{self._sep()}{v % 100:02d}"

    # ----- Fairness ---------------------------------------------------------
    def _fair_geom(self):
        """Geometrie des Fairness-Fensters (Kopfzeilen, Liste, Details, Knöpfe)."""
        body = self._panel_body()
        k = self.k
        lh = self.f_tiny.get_height()
        note_h = lh + 2
        bh = max(22, int(28 * k))
        y = body.y + 2 * lh + 4                       # unter der Erklärung
        nc_w = min(body.w // 2, self.f_btn.size(t("chip.fair_new_client"))[0] + 20)
        r_new = pygame.Rect(body.right - nc_w, y + lh - bh // 2, nc_w, bh)
        hash_y = y + 2 * lh + 2                       # Prüfcode der nächsten Runde (volle Breite)
        ly = hash_y + lh + max(6, int(8 * k))
        bottom = body.bottom - note_h
        lw = max(int(body.w * 0.34), 118)
        step = self.pill_h + 4
        vis = max(1, (bottom - ly) // step)
        detail = pygame.Rect(body.x + lw + self.margin, ly, body.w - lw - self.margin, bottom - ly)
        pv = max(24, min(int(detail.w * 0.36), int((detail.h - bh - 8) * 0.62)))
        preview = pygame.Rect(detail.right - pv - 4, detail.y + 4, pv, int(pv * 1.05))
        ver_w = min(detail.w - 8, max(self.f_btn.size(t("chip.fair_verify"))[0] + 28, int(detail.w * 0.4)))
        r_ver = pygame.Rect(detail.x + 4, detail.bottom - bh - 4, ver_w, bh)
        return dict(body=body, lh=lh, y=y, r_new=r_new, hash_y=hash_y, ly=ly, lw=lw, step=step,
                    vis=vis, detail=detail, preview=preview, r_ver=r_ver, bottom=bottom)

    def _fair_visible(self):
        return self._fair_geom()["vis"]

    def _fair_rects(self):
        g = self._fair_geom()
        rows = []
        hits = [(g["r_new"], self._new_client)]
        es = self._fair_entries()
        for i in range(self.fair_scroll, min(len(es), self.fair_scroll + g["vis"])):
            r = pygame.Rect(g["body"].x, g["ly"] + (i - self.fair_scroll) * g["step"], g["lw"], self.pill_h)
            rows.append((r, i))
            hits.append((r, (lambda i=i: (setattr(self, "fair_sel", i), self.play_sound("move")))))
        if es:
            hits.append((g["r_ver"], lambda: self._verify(self._fair_entry())))
        return g, rows, hits

    def _draw_fair(self, s):
        self._panel_frame(s, "fair", t("chip.fair_title"))
        g, rows, _ = self._fair_rects()
        body, lh, detail = g["body"], g["lh"], g["detail"]
        k = self.k
        # Erklärung (höchstens zwei Zeilen)
        y = body.y
        for line in self._wrap(t("chip.fair_info"), self.f_tiny, body.w, 2):
            s.blit(self.f_tiny.render(line, True, ui.TEXT_DIM), (body.x, y))
            y += lh
        # Nächster Prüfcode, Client-Seed, Knopf "Neuer Client-Seed"
        y = g["y"]
        r_new = g["r_new"]
        left_w = r_new.x - body.x - 8
        img = D.fit_font(t("chip.fair_next") + ":", left_w, lh, bold=False).render(
            t("chip.fair_next") + ":", True, ui.TEXT)
        s.blit(img, (body.x, y))
        ci_txt = f"{t('chip.fair_client')}: {self.fair['client']}   ·   {t('chip.fair_nonce')} {self.fair['nonce']}"
        ci = D.fit_font(ci_txt, left_w, lh, bold=False).render(ci_txt, True, ui.TEXT_DIM)
        s.blit(ci, (body.x, y + lh))
        self._btn(s, r_new, t("chip.fair_new_client"), on=not self.balls and self.auto is None,
                  hot=r_new.collidepoint(self.mouse))
        hi = self._mono_fit(fairness.commit(self.fair["server"]), body.w, CD.COL_GOLD)
        s.blit(hi, (body.x, g["hash_y"]))
        pygame.draw.line(s, ui.BORDER, (body.x, g["ly"] - max(3, int(4 * k))),
                         (body.right, g["ly"] - max(3, int(4 * k))), 1)
        note = D.fit_font(t("chip.fair_note"), body.w, lh, bold=False).render(t("chip.fair_note"), True,
                                                                              ui.TEXT_FAINT)
        s.blit(note, note.get_rect(midbottom=(body.centerx, body.bottom)))
        es = self._fair_entries()
        if not es:
            img = self.f_small.render(t("chip.fair_empty"), True, ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=(body.centerx, (g["ly"] + g["bottom"]) // 2)))
            return
        sel = self._fair_entry()
        # Liste der letzten Kugeln (neueste oben)
        for r, i in rows:
            e = es[i]
            o = e.get("o") if isinstance(e.get("o"), dict) else {}
            on = i == self.fair_sel
            hot = r.collidepoint(self.mouse)
            pygame.draw.rect(s, ui.BTN_SEL if (on or hot) else ui.mix(ui.PANEL, ui.BG_TOP, 0.35), r,
                             border_radius=r.h // 2)
            if on:
                pygame.draw.rect(s, CD.COL_GOLD, r, 1, border_radius=r.h // 2)
            ni = self.f_tiny.render(f"#{e['n']}", True, ui.TEXT if on else ui.TEXT_DIM)
            s.blit(ni, ni.get_rect(midleft=(r.x + 8, r.centery)))
            m = o.get("m") if isinstance(o.get("m"), int) and not isinstance(o.get("m"), bool) else 0
            pw = max(30, r.w * 2 // 5)
            pill = D.pill(pw, max(10, r.h - 4), m, self._fmt(m), max(8, int(r.h * 0.55)))
            s.blit(pill, pill.get_rect(midright=(r.right - 3, r.centery + 1)))
            chk = self.fair_checks.get(e["n"])
            if chk is not None:
                good = chk[0] and chk[1]
                ic = D.icon("check" if good else "cross", max(8, r.h - 6), ui.GREEN if good else ui.RED)
                s.blit(ic, ic.get_rect(midright=(r.right - pw - 6, r.centery)))
        if len(es) > len(rows) and rows:
            more = self.f_tiny.render(f"{self.fair_scroll + 1}-{self.fair_scroll + len(rows)} / {len(es)}",
                                      True, ui.TEXT_FAINT)
            last = rows[-1][0]
            if last.bottom + more.get_height() <= g["bottom"]:
                s.blit(more, more.get_rect(midtop=(last.centerx, last.bottom + 1)))
        # Details des gewählten Eintrags
        pygame.draw.rect(s, ui.mix(ui.PANEL, ui.BG_TOP, 0.35), detail, border_radius=max(4, int(6 * k)))
        pv = g["preview"]
        o = sel.get("o") if isinstance(sel.get("o"), dict) else {}
        # Multiplikator robust lesen: ein beschädigter Spielstand ("m" als Text)
        # darf das Fenster nicht abstürzen lassen
        om = o.get("m") if isinstance(o.get("m"), int) and not isinstance(o.get("m"), bool) else 0
        xx = detail.x + 6
        tw = pv.x - xx - 6
        y = detail.y + 4
        head = f"{t('chip.fair_nonce')} {sel['n']}"
        hi = self.f_btn.render(head, True, ui.TEXT)
        s.blit(hi, (xx, y))
        y += hi.get_height() + 1
        if o.get("rows") in L.ROWS and o.get("risk") in L.RISKS:
            res = t("lpl.fair_result", rows=o["rows"], risk=self._risk_name(o["risk"]), slot=o.get("slot"),
                    m=L.fmt_short(om, self._sep()))
            ri = D.fit_font(res, tw, self.f_tiny.get_height() + 1).render(res, True, CD.COL_GOLD)
            s.blit(ri, (xx, y))
            y += ri.get_height() + 3
        for lbl, vals in ((t("chip.fair_hash"), (sel["h"][:32], sel["h"][32:])),
                          (t("chip.fair_server"), (sel["s"],)),
                          (t("chip.fair_client"), (sel["c"],))):
            li = D.fit_font(lbl + ":", tw, lh, bold=False).render(lbl + ":", True, ui.TEXT_DIM)
            s.blit(li, (xx, y))
            y += lh
            for v in vals:
                vi = self._mono_fit(v, tw, ui.TEXT, ref=32)
                s.blit(vi, (xx, y))
                y += vi.get_height()
            y += 2
        # Vorschau-Rahmen rechts oben (nach dem Nachrechnen mit dem Weg der Kugel)
        pygame.draw.rect(s, ui.mix(ui.PANEL, ui.BG_TOP, 0.6), pv, border_radius=max(3, int(5 * k)))
        chk = self.fair_checks.get(sel["n"])
        r_ver = g["r_ver"]
        self._btn(s, r_ver, t("chip.fair_verify"), primary=chk is None, hot=r_ver.collidepoint(self.mouse))
        if chk is None:
            ic = D.icon("fair", max(12, pv.w // 3), ui.TEXT_FAINT)
            s.blit(ic, ic.get_rect(center=pv.center))
            return
        ok_hash, ok_res, path = chk
        good = ok_hash and ok_res
        col = ui.GREEN if good else ui.RED
        if path:
            img = D.path_preview(o["rows"], path, pv.w - 4, pv.h - 4, om)
            s.blit(img, (pv.x + 2, pv.y + 2))
        # Ergebnis der Prüfung unter der Vorschau
        y2 = pv.bottom + 4
        ic = D.icon("check" if good else "cross", lh + 4, col)
        word = t("chip.fair_ok") if good else t("chip.fair_bad")
        wi = D.fit_font(word, pv.w - ic.get_width() - 4, self.f_btn.get_height()).render(word, True, col)
        s.blit(ic, (pv.x, y2))
        s.blit(wi, wi.get_rect(midleft=(pv.x + ic.get_width() + 4, y2 + ic.get_height() // 2)))
        if path:
            calc = t("lpl.fair_recalc", slot=L.slot_of(path))
            cimg = D.fit_font(calc, pv.w + 4, lh, bold=False).render(calc, True, ui.TEXT_DIM)
            if y2 + ic.get_height() + cimg.get_height() <= r_ver.y + r_ver.h:
                s.blit(cimg, (pv.x - 2, y2 + ic.get_height() + 1))

    def _mono_fit(self, text, max_w, color=None, ref=None):
        """Hex-Werte in Monospace: größte Schrift, in der ref Zeichen (Standard:
        der Text selbst) in max_w passen; notfalls gekürzt mit '…'."""
        color = color or ui.TEXT
        max_w = max(20, max_w)
        probe = "0" * ref if ref else text
        f = ui.font(9, mono=True)
        for sz in range(max(10, int(14 * self.k)), 8, -1):
            f = ui.font(sz, mono=True)
            if f.size(probe)[0] <= max_w:
                break
        cut = text
        while len(cut) > 4 and f.size(cut)[0] > max_w:
            cut = cut[:-2] + "…"
        return f.render(cut, True, color)

    def _wrap(self, text, fnt, w, max_lines):
        words = text.split()
        lines, cur = [], ""
        for wd in words:
            test = (cur + " " + wd).strip()
            if fnt.size(test)[0] <= w or not cur:
                cur = test
            else:
                lines.append(cur)
                cur = wd
        if cur:
            lines.append(cur)
        if len(lines) > max_lines:
            lines = lines[:max_lines]
            last = lines[-1]
            while len(last) > 3 and fnt.size(last + "…")[0] > w:
                last = last[:-1]
            lines[-1] = last + "…"
        return lines
