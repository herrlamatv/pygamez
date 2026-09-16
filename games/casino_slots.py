# -*- coding: utf-8 -*-
"""
casino_slots.py
===============
Lama-Slot des Casinos (Mixin für ``CasinoGame``).

5 Walzen x 3 Reihen, 10 Gewinnlinien, Einsatz je Linie 1/2/5/10.
Lama = Wild, Goldmünze = Scatter (3+ irgendwo = Scatter-Gewinn und
10 Freispiele, in denen alle Gewinne doppelt zählen). Regeln, Walzenstreifen
und Gewinntabelle stehen in ``casino_logic.py``.

Ablauf einer Drehung:
1. Einsatz abbuchen (Freispiele kosten nichts), Stopps ziehen, Ergebnis
   auswerten und sofort gutschreiben (Anzeige wartet über ``pending``).
   Gewonnene Freispiele werden sofort gespeichert und überleben das Verlassen.
2. Walzen laufen mit gleichmäßigem Tempo und bremsen gestaffelt ab; jede
   landet exakt auf ihrem Stopp und federt kurz nach. Liegen schon zwei
   Münzen, drehen die restlichen Walzen spannungsvoll länger ("Anticipation").
   Enter/Leertaste während des Drehens spult vor.
3. Gewinnanzeige: erst alle Linien, dann jede einzeln mit Pfad, Rahmen und
   hüpfenden Symbolen; große Gewinne mit Banner, Konfetti und Münzregen.

Auto-Spin 10/25 und Turbo (schnellere Walzen, kürzere Pausen) - Turbo und der
Linien-Einsatz bleiben in den Einstellungen gespeichert.
"""

import math

import pygame

import audio
import lamabank
from game_base import InputEvent
from i18n import t
import ui

from . import casino_draw as D
from . import casino_logic as L

S_IDLE, S_SPIN, S_SHOW = "idle", "spin", "show"
FREE_KEY = "slots_free"
SPEED = 21.0            # Felder/s beim Drehen
DECEL = 0.34            # Abbremsphase je Walze
BOUNCE = 0.16           # Nachfedern nach dem Stopp
BIG_WIN = 15            # ab x Gesamteinsatz: "Großer Gewinn"
MEGA_WIN = 50


class SlotsMixin:
    # ===================================================== Aufbau
    def _s_init(self):
        bet = self._opt("line_bet", 1)
        self.s_bet = bet if bet in L.LINE_BETS else 1
        self.s_turbo = bool(self._opt("turbo", False))
        self.s_phase = S_IDLE
        self.s_stops = L.random_stops(self.rng)
        self.s_pos = [float(p) for p in self.s_stops]
        self.s_reels = []
        self.s_t = 0.0
        self.s_speed = 1.0
        self.s_res = None
        self.s_free_spin = False          # läuft gerade ein Freispiel?
        self.s_show_t = 0.0
        self.s_win_shown = 0.0
        self.s_auto = 0
        self.s_auto_menu = False
        self.s_paytable = False
        self.s_banners = []
        self.s_anticip = set()
        self.s_hover_line = None
        free = lamabank.get_extra(FREE_KEY, {})
        free = free if isinstance(free, dict) else {}
        self.s_free = max(0, int(free.get("left", 0) or 0))
        fb = free.get("bet", self.s_bet)
        self.s_free_bet = fb if fb in L.LINE_BETS else self.s_bet
        self.s_free_won = max(0, int(free.get("won", 0) or 0))
        self.s_fs_active = self.s_free > 0
        if self.s_free > 0:
            self._s_banner(t("cas.s.free_left", n=self.s_free), None, (255, 150, 200))

    def _s_busy(self):
        return getattr(self, "s_phase", S_IDLE) == S_SPIN

    def _s_layout(self):
        W, H, k, m = self.width, self.height, self.k, self.margin
        top = self.hud_h + m
        bottom = self.strip.y - m
        tb = max(24, int(44 * k))
        ib = max(26, int(40 * k))
        tab = max(16, int(26 * k))
        pad = max(6, int(12 * k))
        gap = max(3, int(6 * k))
        avail_h = bottom - top
        cell = min((avail_h - tb - ib - pad) / 3.0,
                   (W - 2 * m - 2 * tab - 2 * pad - 4 * gap) / 5.0)
        cell = int(max(24, cell))
        reels_w = 5 * cell + 4 * gap
        cab_w = reels_w + 2 * tab + 2 * pad
        cab_h = tb + 3 * cell + ib + pad
        cab = pygame.Rect(0, 0, cab_w, cab_h)
        cab.center = (W // 2, int(top + avail_h / 2))
        self.s_cell, self.s_gap, self.s_tab, self.s_pad = cell, gap, tab, pad
        self.s_tb, self.s_ib = tb, ib
        self.s_cab = cab
        self.s_win = pygame.Rect(cab.x + pad + tab, cab.y + tb, reels_w, 3 * cell)
        self.s_reel_rects = [pygame.Rect(self.s_win.x + r * (cell + gap), self.s_win.y,
                                         cell, 3 * cell) for r in range(L.REELS)]
        fpad = gap + 3                       # Rahmen um das Walzenfenster
        self.s_info = pygame.Rect(cab.x + pad, self.s_win.bottom + fpad,
                                  cab_w - 2 * pad, ib - fpad)
        self._s_build_tabs()
        self._s_build_cabinet()
        self._s_build_strip()
        self.s_bg = self._s_make_bg(W, H)
        self.s_pay_cache = None

    def _s_build_tabs(self):
        """Nummern-Reiter links/rechts: je Reihe gestapelt."""
        cell, tab = self.s_cell, self.s_tab
        self.s_tabs = {}
        for side in (0, 1):
            per_row = {0: [], 1: [], 2: []}
            for i, line in enumerate(L.LINES):
                per_row[line[0 if side == 0 else -1]].append(i)
            for row, lines in per_row.items():
                n = len(lines)
                th = min(int(cell / 4.4), int(tab * 0.9))
                total = n * th + (n - 1) * 2
                y0 = self.s_win.y + row * cell + (cell - total) // 2
                x = self.s_win.x - tab - 1 if side == 0 else self.s_win.right + 1
                for j, i in enumerate(lines):
                    self.s_tabs[(side, i)] = pygame.Rect(x, y0 + j * (th + 2), tab, th)

    def _s_make_bg(self, W, H):
        surf = pygame.Surface((W, H))
        top, bottom = (46, 16, 58), (12, 6, 20)
        for y in range(H):
            f = y / max(1, H - 1)
            surf.fill(ui.mix(top, bottom, f), (0, y, W, 1))
        glow = pygame.Surface((W, H), pygame.SRCALPHA)
        step = max(18, int(34 * self.k))
        for x in range(-H, W, step):
            pygame.draw.line(glow, (255, 255, 255, 7), (x, 0), (x + H, H), 2)
        pygame.draw.ellipse(glow, (255, 120, 190, 22),
                            (W * 0.1, H * 0.12, W * 0.8, H * 0.7))
        surf.blit(glow, (0, 0))
        return surf

    def _s_build_cabinet(self):
        cab = self.s_cab
        k = self.k
        surf = pygame.Surface((cab.w + 16, cab.h + 16), pygame.SRCALPHA)
        o = 8
        body = pygame.Rect(o, o, cab.w, cab.h)
        rad = max(10, int(22 * k))
        pygame.draw.rect(surf, (0, 0, 0, 110), body.move(3, 6), border_radius=rad)
        grad = pygame.Surface(body.size, pygame.SRCALPHA)
        for y in range(body.h):
            f = y / max(1, body.h - 1)
            grad.fill(ui.mix((92, 30, 96), (38, 12, 48), f), (0, y, body.w, 1))
        mask = pygame.Surface(body.size, pygame.SRCALPHA)
        pygame.draw.rect(mask, (255, 255, 255, 255), mask.get_rect(), border_radius=rad)
        grad.blit(mask, (0, 0), special_flags=pygame.BLEND_RGBA_MULT)
        surf.blit(grad, body)
        pygame.draw.rect(surf, D.COL_GOLD_D, body, max(3, int(5 * k)), border_radius=rad)
        pygame.draw.rect(surf, D.COL_GOLD, body.inflate(-4, -4), max(1, int(2 * k)),
                         border_radius=rad)
        # Walzenfenster
        win = self.s_win.move(o - cab.x, o - cab.y)
        frame = win.inflate(self.s_gap * 2 + 6, self.s_gap * 2 + 6)
        pygame.draw.rect(surf, (20, 8, 24), frame, border_radius=max(6, int(10 * k)))
        pygame.draw.rect(surf, D.COL_GOLD, frame, max(2, int(3 * k)),
                         border_radius=max(6, int(10 * k)))
        for r in range(L.REELS):
            rr = self.s_reel_rects[r].move(o - cab.x, o - cab.y)
            for y in range(rr.h):
                f = abs(y / max(1, rr.h - 1) - 0.5) * 2
                surf.fill(ui.mix((252, 248, 238), (206, 196, 180), f ** 2),
                          (rr.x, rr.y + y, rr.w, 1))
        # Titelfeld
        band = pygame.Rect(body.x + int(30 * k), body.y + max(3, int(6 * k)),
                           body.w - int(60 * k), self.s_tb - max(6, int(12 * k)))
        pygame.draw.rect(surf, (26, 8, 30), band, border_radius=band.h // 2)
        pygame.draw.rect(surf, D.COL_GOLD_D, band, max(1, int(2 * k)),
                         border_radius=band.h // 2)
        self.s_title_band = band.move(cab.x - o, cab.y - o)
        self.s_cab_surf = surf
        self.s_cab_off = o
        self.s_bulbs = []
        n = max(8, band.w // max(12, int(22 * k)))
        for i in range(n):
            x = self.s_title_band.x + int((i + 0.5) * self.s_title_band.w / n)
            self.s_bulbs.append((x, self.s_title_band.y - max(2, int(3 * k))))
            self.s_bulbs.append((x, self.s_title_band.bottom + max(2, int(3 * k))))

    def _s_build_strip(self):
        m, k = self.margin, self.k
        sy, sh = self.strip.y, self.strip_h
        bh = max(30, int(sh * 0.62))
        by = sy + (sh - bh) // 2
        pad = int(22 * max(0.7, k))
        sq = bh
        lbl_w = max(self.f_tiny.size(t("cas.s.line_bet"))[0],
                    self.f_btn.size("10 (= 100)")[0]) + int(10 * k)
        self.s_minus = pygame.Rect(m, by, sq, bh)
        self.s_bet_box = pygame.Rect(self.s_minus.right + 4, by, lbl_w, bh)
        self.s_plus = pygame.Rect(self.s_bet_box.right + 4, by, sq, bh)
        left = self.s_plus.right + m
        keys = ("pay", "turbo", "auto", "spin")
        labels = [t("cas.s.paytable"), t("cas.s.turbo"), t("cas.s.auto"),
                  t("cas.spin")]
        widths = [self.f_btn.size(lbl)[0] + pad for lbl in labels]
        widths[2] = max(widths[2], self.f_btn.size(t("cas.s.auto") + " 25")[0] + pad)
        widths[3] = max(widths[3], self.f_btn.size(t("cas.s.stop"))[0] + pad,
                        int(110 * k))
        bgap = max(4, int(8 * k))
        room = self.width - m - left - bgap * (len(keys) - 1)
        if sum(widths) > room:
            f = room / sum(widths)
            widths = [int(w * f) for w in widths]
        self.s_btns = {}
        x = self.width - m
        for key, w in reversed(list(zip(keys, widths))):
            self.s_btns[key] = pygame.Rect(x - w, by, w, bh)
            x -= w + bgap
        a = self.s_btns["auto"]
        mh = int(bh * 0.9)
        self.s_auto_opts = [(n, pygame.Rect(a.x, self.strip.y - (i + 1) * (mh + 4), a.w, mh))
                            for i, n in enumerate(L.AUTO_COUNTS)]

    # ===================================================== Hilfen
    def _s_line_bet(self):
        return self.s_free_bet if self.s_free > 0 or self.s_free_spin else self.s_bet

    def _s_total_bet(self):
        return self._s_line_bet() * len(L.LINES)

    def _s_hud_text(self):
        if self.s_free > 0 or self.s_free_spin:
            return t("cas.s.free_left", n=self.s_free)
        return f"{t('cas.bet')}: {self._s_total_bet()}"

    def _s_banner(self, text, sub, color, dur=2.2, kind="info", amount=0):
        self.s_banners.append(dict(text=text, sub=sub, color=color, t=0.0,
                                   dur=dur * (0.6 if self.s_turbo else 1.0),
                                   kind=kind, amount=amount))

    def _s_save_free(self):
        if self.s_free > 0 or self.s_fs_active:
            lamabank.set_extra(FREE_KEY, {"left": self.s_free, "bet": self.s_free_bet,
                                          "won": self.s_free_won})
        else:
            lamabank.set_extra(FREE_KEY, {})

    def _s_set_bet(self, direction):
        if self.s_phase == S_SPIN or self.s_free > 0 or self.s_fs_active:
            return
        i = L.LINE_BETS.index(self.s_bet)
        j = max(0, min(len(L.LINE_BETS) - 1, i + direction))
        if j != i:
            self.s_bet = L.LINE_BETS[j]
            self._set_opt("line_bet", self.s_bet)
            self.play_sound("select")

    def _s_toggle_turbo(self):
        self.s_turbo = not self.s_turbo
        self._set_opt("turbo", self.s_turbo)
        self.play_sound("click")

    def _s_start_auto(self, n):
        self.s_auto_menu = False
        self.s_auto = n
        self.play_sound("select")
        if self.s_phase != S_SPIN and not self.s_banners:
            self._s_spin()

    # ===================================================== Drehen
    def _s_spin(self):
        if self.s_phase == S_SPIN:
            self.s_speed = 3.2                # vorspulen
            return
        if self.s_banners:
            self.s_banners[0]["t"] = max(self.s_banners[0]["t"],
                                         self.s_banners[0]["dur"] - 0.25)
            return
        free = self.s_free > 0
        if free:
            self.s_free -= 1
            bet = self.s_free_bet
        else:
            bet = self.s_bet
            if not lamabank.debit(bet * len(L.LINES), "casino"):
                self.s_auto = 0
                self._say(t("cas.not_enough"))
                self.play_sound("hit")
                self._check_broke()
                return
            if self.s_auto > 0:
                self.s_auto -= 1
        stops = L.random_stops(self.rng)
        res = L.evaluate(L.window(stops), bet, free)
        lamabank.credit(res["total"], "casino")
        self.pending += res["total"]
        self.s_free_spin = free
        if res["free_spins"]:
            if not self.s_fs_active and not free:
                self.s_free_won = 0
            self.s_free += res["free_spins"]
            self.s_free_bet = bet
            self.s_fs_active = True
        if free or res["free_spins"]:
            won = self.s_free_won + (res["total"] if free else 0)
            lamabank.set_extra(FREE_KEY, {"left": self.s_free, "bet": self.s_free_bet,
                                          "won": won})
        self.s_res = res
        self.s_stops = stops
        self._s_schedule(stops)
        self.s_phase = S_SPIN
        self.s_t = 0.0
        self.s_speed = 1.0
        self.s_auto_menu = False
        self.play_sound("rotate")

    def _s_schedule(self, stops):
        """Legt für jede Walze Tempo und Stoppzeit fest (landet exakt)."""
        turbo = self.s_turbo
        base = 0.34 if turbo else 0.78
        gap = 0.10 if turbo else 0.24
        extra = 0.45 if turbo else 1.05
        speed = SPEED * (1.5 if turbo else 1.0)
        win = L.window(stops)
        self.s_reels = []
        self.s_anticip = set()
        delay = 0.0
        coins = 0
        for r in range(L.REELS):
            if r >= 2 and coins >= 2:
                delay += extra
                self.s_anticip.add(r)
            T = base + r * gap + delay
            n = len(L.STRIPS[r])
            p0 = self.s_pos[r]
            nominal = speed * (T - DECEL / 2.0)
            need = (p0 - nominal - stops[r]) % n
            dist = nominal + need
            v = dist / (T - DECEL / 2.0)
            self.s_reels.append(dict(p0=p0, dist=dist, v=v, T=T, target=stops[r],
                                     landed=False))
            coins += sum(1 for s in win[r] if s == L.SCATTER)

    def _s_reel_pos(self, reel, t):
        """(Position, Tempo) einer Walze zur Zeit t."""
        T, v = reel["T"], reel["v"]
        n = len(L.STRIPS[0])
        if t <= T - DECEL:
            return reel["p0"] - v * t, v
        if t < T:
            u = (t - (T - DECEL)) / DECEL
            d = v * (T - DECEL) + v * DECEL * (u - u * u / 2.0)
            return reel["p0"] - d, v * (1 - u)
        b = (t - T) / BOUNCE
        if b < 1.0:
            return reel["target"] - 0.14 * math.sin(math.pi * b) * (1 - b), 0.0
        return float(reel["target"] % n), 0.0

    def _s_reveal(self):
        res = self.s_res
        self.pending = max(0, self.pending - res["total"])
        self.s_phase = S_SHOW if res["total"] > 0 else S_IDLE
        self.s_show_t = 0.0
        self.s_win_shown = 0.0
        if self.s_free_spin:
            self.s_free_won += res["total"]
        self.score = lamabank.score_for("casino")
        total_bet = self._s_line_bet() * len(L.LINES)
        mult = res["total"] / max(1, total_bet)
        if res["jackpot"]:
            self._s_banner(t("cas.s.jackpot"), None, (255, 214, 90), 3.2, "win",
                           res["total"])
            self.ach_event("slots_jackpot")
            ui.spawn_confetti(self.width, self.height, 140)
            self._coin_shower(90)
            self.play_sound("level")
            self.rumble(300)
        elif mult >= MEGA_WIN:
            self._s_banner(t("cas.s.mega_win"), None, (255, 150, 220), 2.8, "win",
                           res["total"])
            ui.spawn_confetti(self.width, self.height, 110)
            self._coin_shower(70)
            self.play_sound("win")
            self.rumble(220)
        elif mult >= BIG_WIN:
            self._s_banner(t("cas.s.big_win"), None, (255, 214, 90), 2.4, "win",
                           res["total"])
            ui.spawn_confetti(self.width, self.height)
            self._coin_shower(40)
            self.play_sound("win")
        elif res["total"] > 0:
            self.play_sound("point")
        if res["free_spins"]:
            self._s_banner(t("cas.s.free_won", n=res["free_spins"]),
                           t("cas.s.free_info"), (255, 150, 200), 2.4)
            self.play_sound("powerup")
        if self.s_free_spin and self.s_free == 0:
            self._s_save_free()

    def _s_end_free(self):
        self.s_fs_active = False
        self._s_banner(t("cas.s.free_end"), f"+{self.s_free_won}", (255, 214, 90),
                       2.6, "win", self.s_free_won)
        if self.s_free_won >= BIG_WIN * self.s_free_bet * len(L.LINES):
            ui.spawn_confetti(self.width, self.height)
            self._coin_shower(40)
        self.play_sound("level")
        self.s_free_won = 0
        self._s_save_free()

    # ===================================================== Eingabe
    def _s_event(self, event):
        kind = event.kind
        if kind == InputEvent.MOUSEMOVE:
            self.s_hover_line = None
            for (side, i), rect in self.s_tabs.items():
                if rect.collidepoint(event.pos):
                    self.s_hover_line = i
            return
        if self.s_paytable:
            if kind == InputEvent.MOUSEDOWN or (
                    kind == InputEvent.KEYDOWN and event.key in (
                        "i", "I", "p", "P", "Return", "space", "KP_Enter")):
                self.s_paytable = False
                self.play_sound("click")
            return
        if kind == InputEvent.KEYDOWN:
            key = event.key
            if key in ("Return", "space", "KP_Enter"):
                if self.s_auto > 0 and self.s_phase != S_SPIN:
                    self.s_auto = 0
                else:
                    self._s_spin()
            elif key in ("plus", "KP_Add", "Up", "equal"):
                self._s_set_bet(1)
            elif key in ("minus", "KP_Subtract", "Down"):
                self._s_set_bet(-1)
            elif key in ("t", "T"):
                self._s_toggle_turbo()
            elif key == "a":
                if self.s_auto:
                    self.s_auto = 0
                else:
                    self._s_start_auto(L.AUTO_COUNTS[0])
            elif key == "A":
                if self.s_auto:
                    self.s_auto = 0
                else:
                    self._s_start_auto(L.AUTO_COUNTS[1])
            elif key in ("i", "I", "p", "P"):
                self.s_paytable = True
                self.s_auto_menu = False
                self.play_sound("click")
            return
        if kind == InputEvent.WHEEL:
            self._s_set_bet(1 if event.delta > 0 else -1)
            return
        if kind != InputEvent.MOUSEDOWN or event.button != 1:
            return
        pos = event.pos
        if self.s_auto_menu:
            for n, rect in self.s_auto_opts:
                if rect.collidepoint(pos):
                    self._s_start_auto(n)
                    return
            self.s_auto_menu = False
            return
        if self.s_minus.collidepoint(pos):
            self._s_set_bet(-1)
        elif self.s_plus.collidepoint(pos):
            self._s_set_bet(1)
        elif self.s_btns["pay"].collidepoint(pos):
            self.s_paytable = True
            self.play_sound("click")
        elif self.s_btns["turbo"].collidepoint(pos):
            self._s_toggle_turbo()
        elif self.s_btns["auto"].collidepoint(pos):
            if self.s_auto:
                self.s_auto = 0
                self.play_sound("move")
            else:
                self.s_auto_menu = True
                self.play_sound("click")
        elif self.s_btns["spin"].collidepoint(pos) or self.s_win.collidepoint(pos):
            if self.s_auto > 0 and self.s_phase != S_SPIN:
                self.s_auto = 0
            else:
                self._s_spin()

    # ===================================================== Update
    def _s_update(self, dt):
        if self.s_banners:
            b = self.s_banners[0]
            b["t"] += dt
            if b["t"] >= b["dur"]:
                self.s_banners.pop(0)
        if self.s_phase == S_SPIN:
            self.s_t += dt * self.s_speed
            last = 0.0
            for r, reel in enumerate(self.s_reels):
                pos, _ = self._s_reel_pos(reel, self.s_t)
                self.s_pos[r] = pos
                if not reel["landed"] and self.s_t >= reel["T"]:
                    reel["landed"] = True
                    self.play_sound("lock")
                    if r + 1 < L.REELS and (r + 1) in self.s_anticip:
                        audio.tone(420 + 90 * r, 0.22, self.settings, "sine", 0.18)
                last = max(last, reel["T"])
            if self.s_t >= last + BOUNCE:
                for r, reel in enumerate(self.s_reels):
                    self.s_pos[r] = float(reel["target"])
                self._s_reveal()
            return
        self.s_show_t += dt
        if self.s_res and self.s_res["total"] > 0:
            target = self.s_res["total"]
            step = max(target * dt / (0.5 if self.s_turbo else 1.1), 30 * dt)
            self.s_win_shown = min(target, self.s_win_shown + step)
        if self.broke or self.s_paytable or self.s_banners:
            return
        won = self.s_res is not None and self.s_res["total"] > 0
        wait = (1.7 if won else 0.55) * (0.5 if self.s_turbo else 1.0)
        if self.s_fs_active and self.s_free == 0 and self.s_show_t >= wait:
            self._s_end_free()
            return
        if self.s_show_t < wait:
            return
        if self.s_free > 0:
            self._s_spin()
        elif self.s_auto > 0:
            self._s_spin()
            if self.s_phase != S_SPIN:
                self.s_auto = 0
        else:
            self._check_broke_idle()

    def _check_broke_idle(self):
        if not self.broke and lamabank.is_broke("casino", "slots") and \
                self.s_free == 0 and self.pending == 0:
            self._check_broke()

    # ===================================================== Zeichnen
    def _s_highlights(self):
        """(zellen, linien, text) der aktuellen Gewinnanzeige."""
        res = self.s_res
        if self.s_phase != S_SHOW or not res:
            return None
        entries = [("line",) + tuple(e) for e in res["lines"]]
        if res["scatter_win"] > 0:
            entries.append(("scatter", res["scatter"], res["scatter_win"]))
        if not entries:
            return None
        first = 1.0 if not self.s_turbo else 0.6
        step = 1.0 if not self.s_turbo else 0.6
        tt = self.s_show_t
        if tt < first or len(entries) == 1:
            shown = entries
            phase_t = tt
            text = f"{t('cas.win')}: {int(self.s_win_shown)}"
        else:
            idx = int((tt - first) / step) % len(entries)
            shown = [entries[idx]]
            phase_t = (tt - first) % step
            e = entries[idx]
            if e[0] == "line":
                _, i, sym, n, amount = e
                text = t("cas.s.line_win", line=i + 1, n=n, sym=t("cas.sym." + sym),
                         win=amount)
            else:
                text = t("cas.s.scatter_win", n=e[1], win=e[2])
        cells = set()
        lines = []
        for e in shown:
            if e[0] == "line":
                _, i, sym, n, amount = e
                lines.append(i)
                for r in range(n):
                    cells.add((r, L.LINES[i][r]))
            else:
                cells.update(res["coins"])
        return cells, lines, text, phase_t

    def _s_draw(self, s):
        s.blit(self.s_bg, (0, 0))
        cab = self.s_cab
        o = self.s_cab_off
        s.blit(self.s_cab_surf, (cab.x - o, cab.y - o))
        self._s_draw_title(s)
        hl = self._s_highlights()
        cells = hl[0] if hl else None
        phase_t = hl[3] if hl else 0.0
        pop_scale = 1.0 + 0.13 * math.sin(math.pi * min(1.0, phase_t / 0.35)) \
            if phase_t < 0.35 else 1.0
        for r, rect in enumerate(self.s_reel_rects):
            speed = 0.0
            if self.s_phase == S_SPIN and r < len(self.s_reels):
                _, speed = self._s_reel_pos(self.s_reels[r], self.s_t)
            dim = None
            pop = None
            if cells is not None:
                dim = {row for row in range(L.ROWS) if (r, row) not in cells}
                pop = {row: pop_scale for row in range(L.ROWS) if (r, row) in cells}
            D.draw_reel(s, rect, L.STRIPS[r], self.s_pos[r], self.s_cell,
                        speed * self.s_speed, dim, pop)
            if self.s_phase == S_SPIN and r in self.s_anticip and \
                    not self.s_reels[r]["landed"]:
                glow = ui.mix(D.COL_GOLD, (255, 255, 255), ui.pulse(9.0, 0.0, 1.0))
                pygame.draw.rect(s, glow, rect.inflate(6, 6), max(2, int(3 * self.k)),
                                 border_radius=6)
        self._s_draw_tabs(s, hl)
        if hl:
            self._s_draw_win(s, hl)
        elif self.s_hover_line is not None and self.s_phase != S_SPIN:
            self._s_draw_line_path(s, self.s_hover_line, 2)
        self._s_draw_info(s, hl)
        self._s_draw_strip(s)
        if self.s_banners:
            self._s_draw_banner(s, self.s_banners[0])
        if self.s_paytable:
            self._s_draw_paytable(s)

    def _s_draw_title(self, s):
        band = self.s_title_band
        tick = int(pygame.time.get_ticks() / (90 if self.s_fs_active else 160))
        for i, (x, y) in enumerate(self.s_bulbs):
            on = (i // 2 + tick) % 3 == 0
            col = (255, 236, 150) if on else (120, 80, 40)
            r = max(2, int(3 * self.k))
            if on:
                pygame.draw.circle(s, (255, 200, 90), (x, y), r + 2)
            pygame.draw.circle(s, col, (x, y), r)
        if self.s_fs_active:
            title = f"{t('cas.s.free_title')}  {self.s_free}"
            top, bot = (255, 190, 230), (230, 80, 150)
        else:
            title = t("cas.mode.slots").upper()
            top, bot = (255, 240, 170), (226, 150, 40)
        fnt = ui.font(max(13, int(band.h * 0.72)), bold=True)
        img = ui.grad_text(fnt, title, top, bot)
        if img.get_width() > band.w - 10:
            fnt = ui.font(max(11, int(band.h * 0.52)), bold=True)
            img = ui.grad_text(fnt, title, top, bot)
        s.blit(img, img.get_rect(center=band.center))

    def _s_draw_tabs(self, s, hl):
        active = set(hl[1]) if hl else set()
        fnt = ui.font(max(8, int(self.s_tabs[(0, 0)].h * 0.8)), bold=True)
        for (side, i), rect in self.s_tabs.items():
            col = D.LINE_COLS[i]
            on = i in active or i == self.s_hover_line
            fill = col if on else ui.mix(col, (30, 10, 30), 0.55)
            pygame.draw.rect(s, fill, rect, border_radius=max(2, rect.h // 3))
            img = fnt.render(str(i + 1), True, (20, 10, 20) if on else (240, 230, 240))
            s.blit(img, img.get_rect(center=rect.center))

    def _s_line_points(self, i):
        cell = self.s_cell
        pts = [self.s_tabs[(0, i)].midright]
        for r in range(L.REELS):
            rect = self.s_reel_rects[r]
            pts.append((rect.centerx, rect.y + L.LINES[i][r] * cell + cell // 2))
        pts.append(self.s_tabs[(1, i)].midleft)
        return pts

    def _s_draw_line_path(self, s, i, width):
        pts = self._s_line_points(i)
        w = max(2, int(width * self.k * 1.5))
        pygame.draw.lines(s, (20, 8, 24), False, pts, w + 4)
        pygame.draw.lines(s, D.LINE_COLS[i], False, pts, w)

    def _s_draw_win(self, s, hl):
        cells, lines, text, phase_t = hl
        for i in lines:
            self._s_draw_line_path(s, i, 3)
        col = D.LINE_COLS[lines[0]] if len(lines) == 1 else D.COL_GOLD
        pulse = ui.pulse(7.0, 0.0, 1.0)
        for (r, row) in cells:
            rect = pygame.Rect(self.s_reel_rects[r].x, self.s_win.y + row * self.s_cell,
                               self.s_cell, self.s_cell).inflate(-2, -2)
            pygame.draw.rect(s, ui.mix(col, (255, 255, 255), pulse * 0.5), rect,
                             max(2, int(3 * self.k)), border_radius=max(4, int(8 * self.k)))

    def _s_draw_info(self, s, hl):
        r = self.s_info
        if hl:
            text, col = hl[2], ui.GOLD
        elif self.s_phase == S_SPIN:
            text, col = (t("cas.s.free_title") + f"  +{self.s_free_won}"
                         if self.s_fs_active else t("cas.s.good_luck")), ui.TEXT_DIM
        elif self.s_fs_active:
            text, col = t("cas.s.free_total", n=self.s_free_won), (255, 190, 230)
        elif self.s_auto:
            text, col = t("cas.s.auto_left", n=self.s_auto), ui.TEXT_DIM
        elif self.s_res is not None and self.s_res["total"] == 0:
            text, col = t("cas.no_win"), ui.TEXT_DIM
        else:
            text, col = t("cas.s.hint"), ui.TEXT_DIM
        fnt = self.f_big if hl else self.f_small
        img = fnt.render(text, True, col)
        if img.get_width() > r.w - 8:
            img = self.f_small.render(text, True, col)
        if img.get_width() > r.w - 8:
            img = self.f_tiny.render(text, True, col)
        s.blit(img, img.get_rect(center=r.center))

    def _s_draw_strip(self, s):
        self._draw_strip(s)
        locked = self.s_phase == S_SPIN or self.s_free > 0 or self.s_fs_active
        bet = self._s_line_bet()
        self._btn(s, self.s_minus, "–", on=not locked and bet > L.LINE_BETS[0],
                  hot=self._hot(self.s_minus))
        self._btn(s, self.s_plus, "+", on=not locked and bet < L.LINE_BETS[-1],
                  hot=self._hot(self.s_plus))
        box = self.s_bet_box
        lbl = self.f_tiny.render(t("cas.s.line_bet"), True, ui.TEXT_DIM)
        val = self.f_btn.render(f"{bet} (= {bet * len(L.LINES)})", True, ui.GOLD)
        total_h = lbl.get_height() + val.get_height()
        y0 = box.centery - total_h // 2
        s.blit(lbl, lbl.get_rect(midtop=(box.centerx, y0)))
        s.blit(val, val.get_rect(midtop=(box.centerx, y0 + lbl.get_height())))
        spinning = self.s_phase == S_SPIN
        auto_lbl = t("cas.s.auto") + (f" {self.s_auto}" if self.s_auto else "")
        spin_lbl = t("cas.s.stop") if (self.s_auto and not spinning) else t("cas.spin")
        self._btn(s, self.s_btns["pay"], t("cas.s.paytable"),
                  hot=self._hot(self.s_btns["pay"]))
        self._btn(s, self.s_btns["turbo"], t("cas.s.turbo"), active=self.s_turbo,
                  hot=self._hot(self.s_btns["turbo"]))
        self._btn(s, self.s_btns["auto"], auto_lbl, active=bool(self.s_auto),
                  hot=self._hot(self.s_btns["auto"]))
        self._btn(s, self.s_btns["spin"], spin_lbl, primary=True,
                  on=not self.s_banners or spinning, hot=self._hot(self.s_btns["spin"]))
        if self.s_auto_menu:
            for n, rect in self.s_auto_opts:
                self._btn(s, rect, f"{t('cas.s.auto')} {n}", hot=self._hot(rect))

    def _s_draw_banner(self, s, b):
        cab = self.s_cab
        f = b["t"] / max(0.01, b["dur"])
        appear = min(1.0, b["t"] / 0.25)
        fade = 1.0 if f < 0.85 else max(0.0, (1 - f) / 0.15)
        h = int(cab.h * 0.42)
        band = pygame.Surface((cab.w, h), pygame.SRCALPHA)
        band.fill((12, 4, 18, int(210 * appear * fade)))
        pygame.draw.line(band, (*b["color"], int(255 * fade)), (0, 0), (cab.w, 0), 2)
        pygame.draw.line(band, (*b["color"], int(255 * fade)), (0, h - 2), (cab.w, h - 2), 2)
        y = cab.centery - h // 2
        s.blit(band, (cab.x, y))
        scale = 0.6 + 0.4 * (1 - (1 - appear) ** 3) + 0.04 * math.sin(b["t"] * 8)
        size = max(16, int(self.f_huge.get_height() * scale))
        fnt = ui.font(size, bold=True)
        img = ui.grad_text(fnt, b["text"], (255, 255, 255), b["color"])
        if img.get_width() > cab.w - 20:
            fnt = ui.font(max(14, int(size * (cab.w - 20) / img.get_width())), bold=True)
            img = ui.grad_text(fnt, b["text"], (255, 255, 255), b["color"])
        img.set_alpha(int(255 * fade))
        sub = b["sub"]
        if b["kind"] == "win" and b["amount"]:
            shown = int(b["amount"] * min(1.0, b["t"] / (b["dur"] * 0.6)))
            sub = f"+{shown}"
        if sub:
            simg = self.f_big.render(sub, True, (255, 244, 220))
            if simg.get_width() > cab.w - 20:
                simg = self.f_small.render(sub, True, (255, 244, 220))
            simg.set_alpha(int(255 * fade))
            s.blit(img, img.get_rect(center=(cab.centerx, cab.centery - simg.get_height() // 2)))
            s.blit(simg, simg.get_rect(midtop=(cab.centerx, cab.centery + img.get_height() // 2
                                               - simg.get_height() // 2)))
        else:
            s.blit(img, img.get_rect(center=cab.center))

    def _s_draw_paytable(self, s):
        key = (self.width, self.height, self._s_line_bet(), self._lang)
        if self.s_pay_cache is None or self.s_pay_cache[0] != key:
            self.s_pay_cache = (key, self._s_build_paytable())
        surf = self.s_pay_cache[1]
        s.blit(self._overlay, (0, 0))
        s.blit(surf, surf.get_rect(center=(self.width // 2,
                                           (self.hud_h + self.strip.y) // 2)))

    def _s_build_paytable(self):
        k = self.k
        W = min(self.width - 2 * self.margin, int(640 * max(0.75, k)))
        H = self.strip.y - self.hud_h - 2 * self.margin
        surf = pygame.Surface((W, H), pygame.SRCALPHA)
        ui.draw_panel(surf, surf.get_rect(), accent_top=D.COL_GOLD, shadow=False)
        bet = self._s_line_bet()
        pad = max(6, int(12 * k))
        title = self.f_big.render(t("cas.s.paytable_title"), True, D.COL_GOLD)
        surf.blit(title, title.get_rect(midtop=(W // 2, pad)))
        y = pad + title.get_height() + max(2, int(4 * k))
        foot_lines = [t("cas.s.wild_info"), t("cas.s.scatter_info"),
                      t("cas.s.rtp", p=L.RTP_TEXT)]
        foot_h = len(foot_lines) * (self.f_tiny.get_height() + 1)
        rows = [("lama",), ("coin",)] + [(sym,) for sym in L.SYMBOLS[2:]]
        cols = 2 if W > 440 else 1
        per_col = (len(rows) + cols - 1) // cols
        row_h = int((H - y - foot_h - pad * 2) / per_col)
        icon = max(12, min(row_h - 4, int(46 * k)))
        col_w = (W - 2 * pad) // cols
        fnt = self.f_small if row_h >= 22 else self.f_tiny
        for idx, (sym,) in enumerate(rows):
            c = idx // per_col
            rr = idx % per_col
            x = pad + c * col_w
            yy = y + rr * row_h
            img = D.symbol_surface(sym, icon)
            surf.blit(img, (x, yy + (row_h - icon) // 2))
            if sym == L.SCATTER:
                pays = [L.SCATTER_PAYS[n] * bet * len(L.LINES) for n in (3, 4, 5)]
            else:
                pays = [p * bet for p in L.PAYS[sym]]
            txt = "   ".join(f"{n}× {p}" for n, p in zip((3, 4, 5), pays))
            timg = fnt.render(txt, True, ui.TEXT)
            if timg.get_width() > col_w - icon - 12:
                timg = self.f_tiny.render(txt, True, ui.TEXT)
            surf.blit(timg, timg.get_rect(midleft=(x + icon + 8, yy + row_h // 2)))
        fy = H - pad - foot_h
        for line in foot_lines:
            img = self.f_tiny.render(line, True, ui.TEXT_DIM)
            if img.get_width() > W - 2 * pad:
                img = ui.font(max(9, self.f_tiny.get_height() - 3)).render(
                    line, True, ui.TEXT_DIM)
            surf.blit(img, img.get_rect(midtop=(W // 2, fy)))
            fy += self.f_tiny.get_height() + 1
        return surf
