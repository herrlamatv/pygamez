# -*- coding: utf-8 -*-
"""
casino_roulette.py
==================
Roulette-Teil des Casinos (Mixin für ``CasinoGame``).

Ablauf: SETZEN -> DREHEN -> ERGEBNIS -> SETZEN ...

- Setzen: Chipwert wählen (Leiste unten, Tasten 1-5 oder Mausrad), Linksklick
  auf Zahl/Kante/Ecke/Außenfeld setzt, Rechtsklick nimmt einen Chip weg.
  Wiederholen (R) legt die Einsätze der letzten Runde neu, Verdoppeln (D)
  verdoppelt alles, Löschen (Backspace) räumt den Tisch.
- Drehen (Enter/Leertaste): Einsatz wird abgebucht, die Gewinnzahl gezogen
  und das Ergebnis gutgeschrieben (``pending`` hält die Anzeige zurück).
  Der Rotor dreht im Uhrzeigersinn und läuft langsam aus; die Kugel startet
  gegenläufig auf der Bahn. Ihr Winkel RELATIV zum Rotor folgt einer
  kubischen Ease-out-Kurve, die exakt im gezogenen Fach endet (dort ist die
  Relativ-Geschwindigkeit null - die Kugel "rastet" ein und dreht mit).
  Der Radius bleibt erst auf der Bahn, fällt dann spiralförmig und hüpft
  zuletzt über die Stege. Leertaste/Klick spult vor.
- Ergebnis: Gewinnzahl leuchtet auf dem Tisch, getroffene Stapel zeigen
  ihre Auszahlung, verlorene blenden aus; danach neue Runde.
"""

import math

import pygame

import audio
import lamabank
from game_base import InputEvent
from i18n import t
import ui

from . import cards as C
from . import casino_draw as D
from . import casino_logic as L

R_BET, R_SPIN, R_RESULT = "bet", "spin", "result"
LAND_T = 5.6            # Sekunden bis die Kugel liegt
RESULT_T = 3.4          # Ergebnis-Anzeige, danach neue Runde
IDLE_OMEGA = 0.32       # Rotor im Leerlauf (rad/s)
SPIN_OMEGA = 2.3        # Rotor direkt nach dem Anwurf
OMEGA_TAU = 2.8         # Auslaufzeit des Rotors
BALL_TURNS = 6          # volle Relativ-Umläufe der Kugel
HISTORY_KEY = "roulette_history"


class RouletteMixin:
    # ===================================================== Aufbau
    def _r_init(self):
        chip = self._opt("roulette_chip", 5)
        self.r_chip = chip if chip in L.CHIP_VALUES else 5
        self.r_phase = R_BET
        self.r_bets = {}
        self.r_last = {}
        hist = lamabank.get_extra(HISTORY_KEY, [])
        self.r_history = [n for n in hist if isinstance(n, int) and 0 <= n <= 36] \
            if isinstance(hist, list) else []
        self.r_history = self.r_history[:L.HISTORY_LEN]
        self.r_hover = None
        self.r_rot = self.rng.uniform(0, math.tau)
        self.r_omega = IDLE_OMEGA
        self.r_ball = None                # (winkel, radius) oder None
        self.r_trail = []
        self.r_number = None
        self.r_winners = []
        self.r_payout = 0
        self.r_stake = 0
        self.r_t = 0.0
        self.r_fast = False
        self.r_res_t = 0.0
        self.r_msg = None                 # (text, farbe)
        self.r_tick_i = None
        self.r_tick_cd = 0.0
        self.r_drops = {}                 # schlüssel -> Fall-Animation 0..1

    def _r_busy(self):
        return getattr(self, "r_phase", R_BET) == R_SPIN

    def _r_layout(self):
        W, H, k, m = self.width, self.height, self.k, self.margin
        top = self.hud_h + m
        bottom = self.strip.y - m
        uw = (W - 2 * m) / L.TABLE_W
        avail = bottom - top
        uh = min(uw * 0.82, avail * 0.56 / L.TABLE_H)
        self.r_uw, self.r_uh = uw, uh
        th = uh * L.TABLE_H
        self.r_tx = m
        self.r_ty = bottom - th
        self.r_table_rect = pygame.Rect(int(self.r_tx), int(self.r_ty),
                                        int(uw * L.TABLE_W) + 1, int(th) + 1)
        region_h = self.r_ty - m - top
        wd = int(max(60, min(region_h, W * 0.40)))
        self.r_wheel_c = (m + wd // 2 + int(6 * k), int(top + region_h / 2))
        self.r_wheel = D.WheelArt(wd)
        px = self.r_wheel_c[0] + wd // 2 + 2 * m
        self.r_panel = pygame.Rect(px, int(top), W - m - px, int(region_h))
        self.r_felt = C.make_felt(W, H, D.COL_FELT, D.COL_FELT_D)
        self.r_chip_r = max(6, int(min(uw, uh) * 0.34))
        self._r_build_table()
        self._r_build_strip()
        self.r_hl = pygame.Surface((max(1, int(uw)), max(1, int(uh))),
                                   pygame.SRCALPHA)
        self.r_hl.fill((255, 255, 255, 60))
        self.r_win_hl = pygame.Surface((max(1, int(uw)), max(1, int(uh))),
                                       pygame.SRCALPHA)
        self.r_win_hl.fill((255, 214, 90, 80))

    def _r_build_strip(self):
        """Chip-Auswahl links, Buttons rechts - Breiten nach Textlänge."""
        m, k = self.margin, self.k
        sy, sh = self.strip.y, self.strip_h
        cr = max(11, int(sh * 0.30))
        gap = max(4, int(8 * k))
        self.r_chip_rects = []
        x = m + cr
        for v in L.CHIP_VALUES:
            self.r_chip_rects.append((v, pygame.Rect(x - cr, sy + sh // 2 - cr,
                                                     2 * cr, 2 * cr)))
            x += 2 * cr + gap
        self.r_chip_rad = cr
        left = x - gap + m
        keys = ("clear", "double", "rebet", "spin")
        labels = [self._r_btn_label(key) for key in keys]
        pad = int(22 * max(0.7, k))
        widths = [self.f_btn.size(lbl)[0] + pad for lbl in labels]
        widths[-1] = max(widths[-1], int(110 * k))
        bgap = max(4, int(8 * k))
        room = self.width - m - left - bgap * (len(keys) - 1)
        if sum(widths) > room:
            f = room / sum(widths)
            widths = [int(w * f) for w in widths]
        bh = max(30, int(sh * 0.62))
        self.r_btns = {}
        x = self.width - m
        for key, w in reversed(list(zip(keys, widths))):
            self.r_btns[key] = pygame.Rect(x - w, sy + (sh - bh) // 2, w, bh)
            x -= w + bgap

    def _r_btn_label(self, key):
        return {"clear": t("cas.r.clear"), "double": t("cas.r.double"),
                "rebet": t("cas.r.rebet"), "spin": t("cas.spin")}[key]

    # ----- Tisch einmal rendern ------------------------------------------
    def _r_build_table(self):
        uw, uh = self.r_uw, self.r_uh
        tw, th = int(uw * L.TABLE_W) + 1, int(uh * L.TABLE_H) + 1
        surf = pygame.Surface((tw + 8, th + 8), pygame.SRCALPHA)
        ox, oy = 4, 4
        felt = ui.mix(D.COL_FELT, (255, 255, 255), 0.06)
        line = (236, 230, 206)
        lw = max(1, int(min(uw, uh) / 22))
        pygame.draw.rect(surf, (0, 0, 0, 70), (ox + 2, oy + 3, tw, th),
                         border_radius=6)
        pygame.draw.rect(surf, felt, (ox, oy, tw, th), border_radius=6)

        def cell(ux, uy, w=1.0, h=1.0):
            return pygame.Rect(ox + int(round(ux * uw)), oy + int(round(uy * uh)),
                               int(round((ux + w) * uw)) - int(round(ux * uw)),
                               int(round((uy + h) * uh)) - int(round(uy * uh)))

        num_font = ui.font(max(9, int(uh * 0.40)), bold=True)
        lbl_font = ui.font(max(9, int(min(uh * 0.34, uw * 0.30))), bold=True)
        # Null
        zero = [(ox + uw, oy), (ox + uw * 0.42, oy), (ox + 2, oy + 1.5 * uh),
                (ox + uw * 0.42, oy + 3 * uh), (ox + uw, oy + 3 * uh)]
        pygame.draw.polygon(surf, D.COL_GREEN, zero)
        pygame.draw.polygon(surf, line, zero, lw)
        img = num_font.render("0", True, (255, 255, 255))
        surf.blit(img, img.get_rect(center=(ox + uw * 0.62, oy + 1.5 * uh)))
        # Zahlen
        for col in range(12):
            for row in range(3):
                n = L.cell_number(col, row)
                r = cell(1 + col, row)
                pill = r.inflate(-max(4, int(uw * 0.16)), -max(4, int(uh * 0.20)))
                pygame.draw.rect(surf, D.number_color(n), pill,
                                 border_radius=max(3, pill.h // 3))
                img = num_font.render(str(n), True, (255, 255, 255))
                surf.blit(img, img.get_rect(center=r.center))
                pygame.draw.rect(surf, line, r, lw)
        # 2:1-Kolonnen
        for row in range(3):
            r = cell(13, row)
            pygame.draw.rect(surf, line, r, lw)
            img = lbl_font.render("2:1", True, (255, 255, 255))
            surf.blit(img, img.get_rect(center=r.center))
        # Dutzende
        for d in range(3):
            r = cell(1 + 4 * d, 3, 4, L.DOZEN_H)
            pygame.draw.rect(surf, line, r, lw)
            img = lbl_font.render(f"{12 * d + 1}–{12 * d + 12}", True,
                                  (255, 255, 255))
            surf.blit(img, img.get_rect(center=r.center))
        # Einfache Chancen
        for i, key in enumerate(L.OUTSIDE_KEYS):
            r = cell(1 + 2 * i, 3 + L.DOZEN_H, 2, L.OUTSIDE_H)
            pygame.draw.rect(surf, line, r, lw)
            if key in ("red", "black"):
                dw, dh = r.w * 0.26, r.h * 0.34
                pts = [(r.centerx, r.centery - dh), (r.centerx + dw, r.centery),
                       (r.centerx, r.centery + dh), (r.centerx - dw, r.centery)]
                pygame.draw.polygon(surf, D.COL_RED if key == "red" else D.COL_BLACK,
                                    pts)
                pygame.draw.polygon(surf, line, pts, lw)
                continue
            txt = {"low": "1–18", "high": "19–36", "even": t("cas.r.even"),
                   "odd": t("cas.r.odd")}[key]
            fnt = lbl_font
            img = fnt.render(txt, True, (255, 255, 255))
            if img.get_width() > r.w - 6:
                fnt = ui.font(max(8, int(lbl_font.get_height() * 0.72)), bold=True)
                img = fnt.render(txt, True, (255, 255, 255))
            surf.blit(img, img.get_rect(center=r.center))
        pygame.draw.rect(surf, D.COL_GOLD, (ox - 1, oy - 1, tw + 2, th + 2),
                         max(2, lw + 1), border_radius=6)
        self.r_table = surf
        self.r_table_off = (ox, oy)

    # ===================================================== Hilfen
    def _r_total(self):
        return sum(self.r_bets.values())

    def _reserved(self):
        if self.mode == "roulette" and self.r_phase == R_BET:
            return self._r_total()
        return 0

    def _r_to_px(self, ux, uy):
        return self.r_tx + ux * self.r_uw, self.r_ty + uy * self.r_uh

    def _r_key_at(self, pos):
        if pos is None:
            return None
        ux = (pos[0] - self.r_tx) / self.r_uw
        uy = (pos[1] - self.r_ty) / self.r_uh
        return L.hit_test(ux, uy)

    def _r_bet_name(self, key):
        kind = L.bet_kind(key)
        if kind in ("red", "black", "even", "odd"):
            return t("cas.r." + kind)
        if kind == "low":
            return t("cas.r.low") + " 1–18"
        if kind == "high":
            return t("cas.r.high") + " 19–36"
        arg = key.split(":", 1)[1]
        if kind == "column":
            return f"{t('cas.r.column')} {3 - int(arg)}"
        if kind == "dozen":
            d = int(arg)
            return f"{t('cas.r.dozen')} {12 * d + 1}–{12 * d + 12}"
        sep = "/" if kind == "cheval" else "-"
        return f"{t('cas.r.' + kind)} {sep.join(str(n) for n in sorted(L.bet_numbers(key)))}"

    def _r_number_text(self, n):
        if n == 0:
            return t("cas.r.zero")
        parts = [t("cas.r." + L.color_of(n)), t("cas.r.even" if n % 2 == 0 else "cas.r.odd"),
                 t("cas.r.low" if n <= 18 else "cas.r.high")]
        return " · ".join(parts)

    def _r_hud_text(self):
        total = self._r_total() if self.r_phase == R_BET else self.r_stake
        return f"{t('cas.bet')}: {total}"

    # ===================================================== Aktionen
    def _r_select_chip(self, value):
        if value != self.r_chip:
            self.r_chip = value
            self._set_opt("roulette_chip", value)
            self.play_sound("select")

    def _r_available(self):
        return lamabank.balance() - self._r_total()

    def _r_place(self, key):
        avail = self._r_available()
        if avail <= 0:
            self._say(t("cas.not_enough"))
            self.play_sound("hit")
            return
        amount = min(self.r_chip, avail)
        self.r_bets[key] = self.r_bets.get(key, 0) + amount
        self.r_drops[key] = 0.0
        self.play_sound("click")

    def _r_remove(self, key):
        if key not in self.r_bets:
            return
        left = self.r_bets[key] - self.r_chip
        if left > 0:
            self.r_bets[key] = left
        else:
            del self.r_bets[key]
        self.play_sound("move")

    def _r_clear(self):
        if self.r_bets:
            self.r_bets = {}
            self.play_sound("move")

    def _r_rebet(self):
        if not self.r_last:
            return
        need = sum(self.r_last.values())
        if need > lamabank.balance():
            self._say(t("cas.not_enough"))
            self.play_sound("hit")
            return
        self.r_bets = dict(self.r_last)
        for key in self.r_bets:
            self.r_drops[key] = 0.0
        self.play_sound("click")

    def _r_double(self):
        if not self.r_bets:
            return
        if self._r_total() * 2 > lamabank.balance():
            self._say(t("cas.not_enough"))
            self.play_sound("hit")
            return
        self.r_bets = {key: v * 2 for key, v in self.r_bets.items()}
        for key in self.r_bets:
            self.r_drops[key] = 0.0
        self.play_sound("merge")

    def _r_spin(self):
        stake = self._r_total()
        if stake <= 0:
            self._say(t("cas.r.place_first"))
            return
        if not lamabank.debit(stake, "casino"):
            self._say(t("cas.not_enough"))
            self.play_sound("hit")
            return
        self.r_stake = stake
        self.r_last = dict(self.r_bets)
        self.r_number = self.rng.randrange(37)
        payout, winners = L.settle(self.r_bets, self.r_number)
        self.r_payout = payout
        self.r_winners = winners
        lamabank.credit(payout, "casino")
        self.pending += payout
        # Anwurf: Rotor beschleunigt, Kugel startet gegenläufig
        self.r_omega = SPIN_OMEGA
        beta0 = self.rng.uniform(0, math.tau)
        self.r_phi0 = beta0 - self.r_rot
        target = self.r_wheel.pocket_angle(self.r_number)
        d = (self.r_phi0 - target) % math.tau
        self.r_phi_end = self.r_phi0 - d - BALL_TURNS * math.tau
        self.r_t = 0.0
        self.r_fast = False
        self.r_trail = []
        self.r_tick_i = None
        self.r_phase = R_SPIN
        self.r_msg = (t("cas.r.no_more"), ui.TEXT_DIM)
        self.play_sound("shoot")

    def _r_ball_state(self, u):
        """(Relativwinkel, Radius) der Kugel bei Fortschritt u (0..1)."""
        wa = self.r_wheel
        R = wa.R
        e = (1.0 - u) ** 3
        phi = self.r_phi_end + (self.r_phi0 - self.r_phi_end) * e
        if u > 0.62:
            w = (u - 0.62) / 0.38
            phi += 0.16 * math.sin(u * 53.0) * (1.0 - u) * w
        rim = R * 0.74
        if u < 0.52:
            r = wa.track_r - R * 0.004 * math.sin(u * 60)
        elif u < 0.80:
            v = (u - 0.52) / 0.28
            v = v * v * (3 - 2 * v)
            r = wa.track_r + (rim - wa.track_r) * v
        else:
            v = (u - 0.80) / 0.20
            base = rim + (wa.pocket_r - rim) * (v * v * (3 - 2 * v))
            hop = R * 0.06 * abs(math.sin(v * 3.2 * math.pi)) * (1.0 - v) ** 1.5
            r = base + hop
        return phi, r

    def _r_land(self):
        n = self.r_number
        self.pending = max(0, self.pending - self.r_payout)
        self.r_phase = R_RESULT
        self.r_res_t = 0.0
        self.r_history.insert(0, n)
        del self.r_history[L.HISTORY_LEN:]
        lamabank.set_extra(HISTORY_KEY, list(self.r_history))
        self.score = lamabank.score_for("casino")
        self.play_sound("lock")
        if self.r_payout > 0:
            plein = any(L.bet_kind(k) == "plein" for k in self.r_winners)
            big = plein or self.r_payout >= 15 * self.r_stake
            self.r_msg = (f"{t('cas.win')}: +{self.r_payout}", ui.GOLD)
            self.play_sound("win" if big else "point")
            for key in self.r_winners:
                ax, ay = self._r_to_px(*L.anchor(key))
                amount = self.r_bets[key] * (L.payout_multiplier(key) + 1)
                self._popup(ax, ay - self.r_chip_r * 1.5, f"+{amount}")
            if plein:
                self.ach_event("roulette_plein")
            if big:
                ui.spawn_confetti(self.width, self.height)
                self._coin_shower(46 if plein else 30)
                self.rumble(160)
        else:
            self.r_msg = (t("cas.no_win"), ui.TEXT_DIM)
            self.play_sound("select")

    def _r_new_round(self):
        self.r_phase = R_BET
        self.r_bets = {}
        self.r_drops = {}
        self.r_msg = None
        self._check_broke()

    # ===================================================== Eingabe
    def _r_event(self, event):
        kind = event.kind
        if kind == InputEvent.MOUSEMOVE:
            self.r_hover = self._r_key_at(event.pos) if self.r_phase == R_BET else None
            return
        if self.r_phase == R_SPIN:
            if kind == InputEvent.MOUSEDOWN or (
                    kind == InputEvent.KEYDOWN
                    and event.key in ("Return", "space", "KP_Enter")):
                self.r_fast = True
            return
        if self.r_phase == R_RESULT:
            if kind == InputEvent.MOUSEDOWN and event.button == 1:
                hit = next((key for key, r in self.r_btns.items()
                            if r.collidepoint(event.pos)), None)
                self._r_new_round()
                if hit == "rebet":
                    self._r_rebet()
                elif hit == "spin":
                    self._r_rebet()
                    if self.r_bets and not self.broke:
                        self._r_spin()
            elif kind == InputEvent.KEYDOWN and event.key in (
                    "Return", "space", "KP_Enter", "r", "R"):
                self._r_new_round()
                if event.key in ("r", "R"):
                    self._r_rebet()
            return
        # ---- Setzen
        if kind == InputEvent.WHEEL:
            i = L.CHIP_VALUES.index(self.r_chip)
            i = max(0, min(len(L.CHIP_VALUES) - 1, i + (1 if event.delta > 0 else -1)))
            self._r_select_chip(L.CHIP_VALUES[i])
        elif kind == InputEvent.KEYDOWN:
            key = event.key
            if key in ("1", "2", "3", "4", "5"):
                self._r_select_chip(L.CHIP_VALUES[int(key) - 1])
            elif key in ("Return", "space", "KP_Enter"):
                self._r_spin()
            elif key in ("BackSpace", "Delete", "c", "C"):
                self._r_clear()
            elif key in ("r", "R"):
                self._r_rebet()
            elif key in ("d", "D"):
                self._r_double()
        elif kind == InputEvent.MOUSEDOWN:
            pos = event.pos
            if event.button == 3:
                key = self._r_key_at(pos)
                if key:
                    self._r_remove(key)
                return
            for value, rect in self.r_chip_rects:
                if rect.collidepoint(pos):
                    self._r_select_chip(value)
                    return
            for name, rect in self.r_btns.items():
                if rect.collidepoint(pos):
                    {"clear": self._r_clear, "double": self._r_double,
                     "rebet": self._r_rebet, "spin": self._r_spin}[name]()
                    return
            key = self._r_key_at(pos)
            if key:
                self._r_place(key)

    # ===================================================== Update
    def _r_update(self, dt):
        # Rotor: Leerlauf bzw. Auslaufen nach dem Anwurf
        self.r_omega += (IDLE_OMEGA - self.r_omega) * (1.0 - math.exp(-dt / OMEGA_TAU))
        for key in list(self.r_drops):
            self.r_drops[key] += dt * 5.0
            if self.r_drops[key] >= 1.0:
                del self.r_drops[key]
        if self.r_phase == R_SPIN:
            step = dt * (3.5 if self.r_fast else 1.0)
            self.r_rot += self.r_omega * step
            self.r_t += step
            u = min(1.0, self.r_t / LAND_T)
            phi, rad = self._r_ball_state(u)
            ball = (self.r_rot + phi, rad)
            if u < 0.9:
                self.r_trail.append(ball)
                del self.r_trail[:-4]
            else:
                self.r_trail = []
            self.r_ball = ball
            # Klackern über die Stege
            if u > 0.55:
                idx = int(math.floor(phi / (math.tau / 37)))
                self.r_tick_cd -= dt
                if self.r_tick_i is not None and idx != self.r_tick_i \
                        and self.r_tick_cd <= 0:
                    self.r_tick_cd = 0.05 + 0.1 * u
                    audio.tone(1900 + 400 * self.rng.random(), 0.015,
                               self.settings, "square", 0.05)
                self.r_tick_i = idx
            if u >= 1.0:
                self._r_land()
        else:
            self.r_rot += self.r_omega * dt
            if self.r_number is not None:
                self.r_ball = (self.r_rot + self.r_wheel.pocket_angle(self.r_number),
                               self.r_wheel.pocket_r)
            if self.r_phase == R_RESULT:
                self.r_res_t += dt
                if self.r_res_t >= RESULT_T:
                    self._r_new_round()
            elif self.r_hover is None and self.r_phase == R_BET:
                self.r_hover = self._r_key_at(self.mouse)

    # ===================================================== Zeichnen
    def _r_draw(self, s):
        s.blit(self.r_felt, (0, 0))
        self.r_wheel.draw(s, self.r_wheel_c, self.r_rot, self.r_ball,
                          self.r_trail if self.r_phase == R_SPIN else ())
        self._r_draw_panel(s)
        self._r_draw_table(s)
        self._r_draw_strip(s)

    def _r_cell_rect(self, n):
        uw, uh = self.r_uw, self.r_uh
        if n == 0:
            return pygame.Rect(int(self.r_tx), int(self.r_ty), int(uw), int(3 * uh))
        x, y = self._r_to_px(1 + L.num_col(n), L.num_row(n))
        return pygame.Rect(int(x), int(y), int(uw), int(uh))

    def _r_key_rect(self, key):
        """Fläche eines Außenfelds (für Hervorhebungen), sonst None."""
        kind = L.bet_kind(key)
        if kind == "column":
            x, y = self._r_to_px(13, int(key.split(":")[1]))
            return pygame.Rect(int(x), int(y), int(self.r_uw), int(self.r_uh))
        if kind == "dozen":
            x, y = self._r_to_px(1 + 4 * int(key.split(":")[1]), 3)
            return pygame.Rect(int(x), int(y), int(self.r_uw * 4),
                               int(self.r_uh * L.DOZEN_H))
        if kind in L.OUTSIDE_KEYS:
            x, y = self._r_to_px(1 + 2 * L.OUTSIDE_KEYS.index(kind), 3 + L.DOZEN_H)
            return pygame.Rect(int(x), int(y), int(self.r_uw * 2),
                               int(self.r_uh * L.OUTSIDE_H))
        return None

    def _r_highlight(self, s, key, surf):
        rect = self._r_key_rect(key)
        if rect is not None:
            img = pygame.transform.scale(surf, rect.size) if rect.size != surf.get_size() \
                else surf
            s.blit(img, rect)
        for n in L.bet_numbers(key):
            r = self._r_cell_rect(n)
            if n == 0:
                img = pygame.transform.scale(surf, r.size)
                s.blit(img, r)
            else:
                s.blit(surf, r)

    def _r_draw_table(self, s):
        ox, oy = self.r_table_off
        s.blit(self.r_table, (int(self.r_tx) - ox, int(self.r_ty) - oy))
        cr = self.r_chip_r
        if self.r_phase == R_BET and self.r_hover:
            self._r_highlight(s, self.r_hover, self.r_hl)
        if self.r_phase == R_RESULT and self.r_number is not None:
            for key in self.r_winners:
                if self._r_key_rect(key) is not None:
                    self._r_highlight(s, key, self.r_win_hl)
            r = self._r_cell_rect(self.r_number)
            pulse = ui.pulse(6.0, 0.0, 1.0)
            pygame.draw.rect(s, ui.mix(D.COL_GOLD, (255, 255, 255), pulse),
                             r.inflate(4, 4), max(2, int(3 * self.k)), border_radius=4)
            # "Dolly" auf der Gewinnzahl
            dx, dy = r.centerx, r.centery
            dr = max(5, int(min(self.r_uw, self.r_uh) * 0.22))
            pygame.draw.ellipse(s, (0, 0, 0), (dx - dr, dy - dr // 2 + 3, 2 * dr, dr))
            pygame.draw.rect(s, (230, 230, 236), (dx - dr * 0.6, dy - dr * 1.4,
                                                 dr * 1.2, dr * 1.4))
            pygame.draw.ellipse(s, (250, 250, 255), (dx - dr * 0.6, dy - dr * 1.7,
                                                     dr * 1.2, dr * 0.6))
        # Chip-Stapel
        for key, amount in self.r_bets.items():
            ax, ay = self._r_to_px(*L.anchor(key))
            alpha = 255
            lift = 0
            if key in self.r_drops:
                f = self.r_drops[key]
                lift = int((1 - f) ** 2 * cr * 1.4)
            if self.r_phase == R_RESULT and key not in self.r_winners:
                alpha = max(0, int(255 * (1 - self.r_res_t / 0.8)))
                if alpha <= 0:
                    continue
            D.draw_chip_stack(s, (ax, ay), amount, cr, alpha, lift)
            if self.r_phase == R_RESULT and key in self.r_winners:
                ring = int(cr + 3 + 3 * ui.pulse(5.0, 0.0, 1.0))
                pygame.draw.circle(s, D.COL_GOLD, (int(ax), int(ay) - 2), ring, 2)
        if self.r_phase == R_BET and self.r_hover and self.r_hover not in self.r_bets:
            ax, ay = self._r_to_px(*L.anchor(self.r_hover))
            img = D.chip_surface(self.r_chip, cr)
            ghost = img.copy()
            ghost.set_alpha(130)
            s.blit(ghost, ghost.get_rect(center=(int(ax), int(ay))))

    def _r_draw_panel(self, s):
        p = self.r_panel
        if p.w < 40 or p.h < 40:
            return
        k = self.k
        y = p.y
        # Verlauf
        title = self.f_tiny.render(t("cas.r.history"), True, ui.TEXT_DIM)
        s.blit(title, (p.x, y))
        y += title.get_height() + max(2, int(4 * k))
        gap = max(2, int(4 * k))
        per_row = 12
        size = int((p.w - gap * (per_row - 1)) / per_row)
        if size < 18:
            per_row = 6
            size = int((p.w - gap * (per_row - 1)) / per_row)
        size = max(12, min(size, int(34 * k)))
        hfont = ui.font(max(9, int(size * 0.52)), bold=True)
        for i in range(L.HISTORY_LEN):
            cx = p.x + (i % per_row) * (size + gap)
            cy = y + (i // per_row) * (size + gap)
            rect = pygame.Rect(cx, cy, size, size)
            if i < len(self.r_history):
                n = self.r_history[i]
                col = D.number_color(n)
                pygame.draw.rect(s, col, rect, border_radius=max(3, size // 4))
                if i == 0:
                    pygame.draw.rect(s, D.COL_GOLD, rect, 2, border_radius=max(3, size // 4))
                img = hfont.render(str(n), True, (255, 255, 255))
                s.blit(img, img.get_rect(center=rect.center))
            else:
                pygame.draw.rect(s, ui.mix(D.COL_FELT_D, (0, 0, 0), 0.3), rect,
                                 1, border_radius=max(3, size // 4))
        rows = (L.HISTORY_LEN + per_row - 1) // per_row
        y += rows * (size + gap) + max(6, int(12 * k))
        # Letzte Zahl + Meldung
        show_n = self.r_number if self.r_phase == R_RESULT else None
        big_r = max(14, min(int(34 * k), (p.bottom - y) // 3))
        if show_n is not None:
            c = (p.x + big_r, y + big_r)
            pygame.draw.circle(s, (0, 0, 0), (c[0] + 2, c[1] + 3), big_r)
            pygame.draw.circle(s, D.number_color(show_n), c, big_r)
            pygame.draw.circle(s, D.COL_GOLD, c, big_r, 2)
            img = ui.font(max(12, int(big_r * 0.95)), bold=True).render(
                str(show_n), True, (255, 255, 255))
            s.blit(img, img.get_rect(center=c))
            desc = self.f_small.render(self._r_number_text(show_n), True, ui.TEXT)
            if desc.get_width() > p.w - 2 * big_r - 10:
                desc = self.f_tiny.render(self._r_number_text(show_n), True, ui.TEXT)
            s.blit(desc, desc.get_rect(midleft=(p.x + 2 * big_r + 10, c[1])))
            y += 2 * big_r + max(4, int(8 * k))
        msg = self.r_msg
        if msg is None and self.r_phase == R_BET:
            msg = (t("cas.r.place_bets"), ui.TEXT)
        if msg is not None and y < p.bottom:
            fnt = self.f_big
            img = fnt.render(msg[0], True, msg[1])
            if img.get_width() > p.w:
                img = self.f_small.render(msg[0], True, msg[1])
            s.blit(img, (p.x, y))
            y += img.get_height() + max(2, int(4 * k))
        # Wette unter der Maus
        info = None
        if self.r_phase == R_BET and self.r_hover:
            key = self.r_hover
            info = f"{self._r_bet_name(key)} · {L.payout_multiplier(key)}:1"
            if key in self.r_bets:
                info += f" · {self.r_bets[key]}"
        elif self.r_phase == R_SPIN:
            info = t("cas.r.skip_hint")
        elif self.r_phase == R_BET:
            info = t("cas.r.place_hint")
        if info:
            img = self.f_tiny.render(info, True, ui.TEXT_DIM)
            if img.get_width() > p.w:
                img = ui.font(max(9, self.f_tiny.get_height() - 3)).render(
                    info, True, ui.TEXT_DIM)
            yy = max(y, p.bottom - img.get_height())
            if yy + img.get_height() <= p.bottom + 2:
                s.blit(img, (p.x, yy))

    def _r_draw_strip(self, s):
        self._draw_strip(s)
        cr = self.r_chip_rad
        for value, rect in self.r_chip_rects:
            sel = value == self.r_chip
            c = rect.center
            lift = int(5 * self.k) if sel else 0
            if sel:
                glow = ui.mix(D.COL_GOLD, (255, 255, 255), ui.pulse(3.0, 0.0, 0.5))
                pygame.draw.circle(s, glow, (c[0], c[1] - lift), cr + 3, 2)
            img = D.chip_surface(value, cr - (0 if sel else 2))
            s.blit(img, img.get_rect(center=(c[0], c[1] - lift)))
        betting = self.r_phase == R_BET
        result = self.r_phase == R_RESULT
        on = {"clear": betting and bool(self.r_bets),
              "double": betting and bool(self.r_bets),
              "rebet": (betting or result) and bool(self.r_last),
              "spin": (betting and bool(self.r_bets)) or (result and bool(self.r_last))}
        for key, rect in self.r_btns.items():
            self._btn(s, rect, self._r_btn_label(key), primary=(key == "spin"),
                      on=on[key], hot=self._hot(rect))
