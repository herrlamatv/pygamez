# -*- coding: utf-8 -*-
"""
casino.py
=========
Casino - europäisches Roulette und der Lama-Slot, gespielt mit den Lama-Chips
der gemeinsamen Lama-Bank (``lamabank.py``, geteilt mit Blackjack und Poker).

Modi (Vorspiel-Screen):

- **Roulette** (``casino_roulette.py``): Setztisch mit allen klassischen
  Wetten (Plein, Cheval, Transversale, Carré, Sixain, Kolonne, Dutzend,
  Rot/Schwarz, Gerade/Ungerade, Manque/Passe). Chips 1/5/25/100/500,
  Linksklick setzt, Rechtsklick nimmt weg. Der Kessel dreht, die Kugel
  läuft spiralförmig ein und fällt genau in das Fach, das beim Drehen
  gezogen wurde.
- **Lama-Slot** (``casino_slots.py``): 5 Walzen x 3 Reihen, 10 Gewinnlinien,
  Lama = Wild, Goldmünze = Scatter (3+ = 10 Freispiele mit doppelten
  Gewinnen), Auto-Spin, Turbo. Auszahlungsquote ~96 %, exakt aus den
  Walzenstreifen berechnet (``casino_logic.exact_rtp``).

Geld-Fluss: Der Einsatz wird beim Drehen sofort abgebucht, das (vorher
gezogene) Ergebnis sofort gutgeschrieben - angezeigt wird der Gewinn aber erst,
wenn Kugel bzw. Walzen stehen (``self.pending``). Wer mitten im Dreh geht,
verliert also nichts, sieht aber auch nichts vorab.

Highscore = Höchststand von 1000 + Casino-Bilanz (nur Roulette + Slot, ohne
Bank-Kredite); ``game_over`` wird nie gesetzt, gesichert wird beim Menü-Rückweg.
Pleite (unter dem Mindesteinsatz des Modus) = Bank-Kredit auf 1000.

Rendering: alles Teure (Filz, Tisch, Kessel, Symbole, Jetons) liegt gecacht
vor und wird nur bei Auflösungs- oder Sprachwechsel neu gebaut.
"""

import math
import random

import pygame

import audio
import lamabank
import settings as settings_mod
import ui
from game_base import Game, InputEvent
from i18n import get_language, t

from . import casino_draw as D
from .casino_roulette import RouletteMixin
from .casino_slots import SlotsMixin

MODES = ("roulette", "slots")


class CasinoGame(RouletteMixin, SlotsMixin, Game):
    name = "Casino"
    highscore_key = "casino"
    supports_multiplayer = False
    wants_right_click = True

    MODES = [("roulette", "cas.mode.roulette"), ("slots", "cas.mode.slots")]

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.game_over = False
        if self.mode not in MODES:
            self.mode = "roulette"
        lamabank.load()
        self.rng = random.Random()
        self.pending = 0                  # schon gutgeschrieben, noch nicht gezeigt
        self.shown_chips = float(lamabank.balance())
        self.mouse = (-1, -1)
        self.coins_fx = []                # Münzregen bei großen Gewinnen
        self.popups = []                  # aufsteigende Gewinnzahlen
        self.toast = None                 # kurze Hinweiszeile (Text, Restzeit)
        self.broke = False
        self._tick_cd = 0.0               # Taktgeber für Zähl-Klicks
        self._lang = get_language()
        self._layout()
        if self.mode == "roulette":
            self._r_init()
        else:
            self._s_init()
        self.score = lamabank.score_for("casino")
        self._check_broke()

    def on_surface_changed(self):
        self._layout()

    def on_exit(self):
        # Offene Freispiele sind schon gespeichert; nichts weiter zu tun.
        pass

    def _layout(self):
        W, H = self.width, self.height
        self.k = k = min(W / 800.0, H / 600.0)
        self.f_tiny = ui.font(max(11, int(13 * k)))
        self.f_small = ui.font(max(12, int(15 * k)))
        self.f_btn = ui.font(max(12, int(15 * k)), bold=True)
        self.f_big = ui.font(max(15, int(22 * k)), bold=True)
        self.f_huge = ui.font(max(26, int(58 * k)), bold=True)
        self.margin = max(6, int(12 * k))
        self.hud_h = max(28, int(42 * k))
        self.strip_h = max(46, int(64 * k))
        self.strip = pygame.Rect(0, H - self.strip_h, W, self.strip_h)
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
        if self.mode == "roulette":
            self._r_layout()
        else:
            self._s_layout()

    # ===================================================== Einstellungen
    def _opt(self, key, default):
        sec = self.settings.get("casino") if isinstance(self.settings, dict) else None
        if isinstance(sec, dict) and key in sec:
            return sec[key]
        return default

    def _set_opt(self, key, value):
        if not isinstance(self.settings, dict):
            return
        sec = self.settings.setdefault("casino", {})
        if sec.get(key) != value:
            sec[key] = value
            settings_mod.save_settings(self.settings)

    # ===================================================== Konto
    def _busy(self):
        return self._r_busy() if self.mode == "roulette" else self._s_busy()

    def _check_broke(self):
        """Pleite = unter dem Mindesteinsatz des Modus (und nichts läuft)."""
        if self._busy():
            self.broke = False
            return
        free = self.mode == "slots" and self.s_free > 0
        self.broke = (not free) and lamabank.is_broke("casino", self.mode)
        if self.broke:
            self.play_sound("gameover")

    def _take_credit(self):
        if lamabank.refill_if_broke("casino", self.mode):
            self.play_sound("powerup")
            ui.spawn_burst(self.width // 2, self.height // 2, ui.GOLD, n=26)
        self.broke = False
        self.shown_chips = float(lamabank.balance())

    def _target_chips(self):
        """Angezeigter Kontostand: ohne zurückgehaltene Gewinne und ohne die
        Chips, die gerade auf dem Roulette-Tisch liegen."""
        return max(0, lamabank.balance() - self.pending - self._reserved())

    def _say(self, text, secs=1.8):
        self.toast = [text, secs]

    # ===================================================== Eingabe
    def handle_event(self, event):
        if event.kind in (InputEvent.MOUSEMOVE, InputEvent.MOUSEDOWN,
                          InputEvent.MOUSEUP) and event.pos is not None:
            self.mouse = event.pos
        if self.broke:
            if event.kind == InputEvent.MOUSEDOWN or (
                    event.kind == InputEvent.KEYDOWN
                    and event.key in ("Return", "space", "KP_Enter")):
                self._take_credit()
            return
        if self.mode == "roulette":
            self._r_event(event)
        else:
            self._s_event(event)

    # ===================================================== Update
    def update(self, dt):
        if self._lang != get_language():
            self._lang = get_language()
            D.clear_cache()
            self._layout()
        target = self._target_chips()
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
                    audio.tone(1500 + self.rng.random() * 300, 0.025,
                               self.settings, "square", 0.08)
        if self.toast:
            self.toast[1] -= dt
            if self.toast[1] <= 0:
                self.toast = None
        self._update_fx(dt)
        if self.mode == "roulette":
            self._r_update(dt)
        else:
            self._s_update(dt)

    def _update_fx(self, dt):
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

    def _coin_shower(self, n=40):
        """Goldmünzen-Regen (große Gewinne)."""
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

    def _popup(self, x, y, text, color=None, dur=1.4):
        self.popups.append(dict(x=x, y=y, text=text, color=color or ui.GOLD,
                                t=0.0, dur=dur))

    # ===================================================== Zeichnen
    def draw(self):
        s = self.surface
        if self.mode == "roulette":
            self._r_draw(s)
        else:
            self._s_draw(s)
        self._draw_hud(s)
        self._draw_fx(s)
        if self.toast:
            self._draw_toast(s)
        if self.broke:
            self._draw_broke(s)

    def _draw_hud(self, s):
        W = self.width
        s.blit(self._hud_bg, (0, 0))
        pygame.draw.line(s, ui.mix(D.COL_GOLD, self.accent, 0.35),
                         (0, self.hud_h - 1), (W, self.hud_h - 1), 1)
        cy = self.hud_h // 2
        pad = self.margin
        coin_r = max(6, int(self.hud_h * 0.26))
        coin = D.symbol_surface("coin", coin_r * 2 + 4)
        s.blit(coin, coin.get_rect(midleft=(pad - 2, cy)))
        left = self.f_big.render(f"{t('cas.chips')}: {int(round(self.shown_chips))}",
                                 True, ui.GOLD)
        s.blit(left, left.get_rect(midleft=(pad + coin_r * 2 + 6, cy)))
        right_txt = f"{t('cas.record')}: {self.score}   ·   " \
                    f"{t('cas.credits', n=lamabank.refills())}"
        right = self.f_small.render(right_txt, True, ui.TEXT_DIM)
        if right.get_width() > W * 0.42:
            right = self.f_tiny.render(right_txt, True, ui.TEXT_DIM)
        rr = right.get_rect(midright=(W - pad, cy))
        s.blit(right, rr)
        mid_txt = self._r_hud_text() if self.mode == "roulette" else self._s_hud_text()
        if mid_txt:
            x0 = pad + coin_r * 2 + 6 + left.get_width() + pad
            x1 = rr.left - pad
            mid = self.f_small.render(mid_txt, True, ui.TEXT)
            if mid.get_width() > x1 - x0:
                mid = self.f_tiny.render(mid_txt, True, ui.TEXT)
            if mid.get_width() <= x1 - x0:
                s.blit(mid, mid.get_rect(center=((x0 + x1) // 2, cy)))

    def _draw_fx(self, s):
        for c in self.coins_fx:
            w = max(2, int(c["r"] * 2 * abs(math.cos(c["spin"]))))
            h = int(c["r"] * 2)
            rect = pygame.Rect(0, 0, w, h)
            rect.center = (int(c["x"]), int(c["y"]))
            pygame.draw.ellipse(s, D.COL_GOLD_D, rect.move(1, 1))
            pygame.draw.ellipse(s, (246, 196, 60), rect)
            if w > 6:
                pygame.draw.ellipse(s, (255, 236, 150), rect.inflate(-w // 2, -h // 2))
        for p in self.popups:
            f = p["t"] / p["dur"]
            img = self.f_big.render(p["text"], True, p["color"])
            if f > 0.6:
                img.set_alpha(int(255 * (1 - (f - 0.6) / 0.4)))
            y = p["y"] - 40 * self.k * (1 - (1 - f) ** 2)
            s.blit(img, img.get_rect(center=(int(p["x"]), int(y))))

    def _draw_toast(self, s):
        text, left = self.toast
        img = self.f_small.render(text, True, ui.TEXT)
        box = img.get_rect()
        box.inflate_ip(24, 12)
        box.midbottom = (self.width // 2, self.strip.y - self.margin)
        panel = pygame.Surface(box.size, pygame.SRCALPHA)
        pygame.draw.rect(panel, (12, 8, 20, 225), panel.get_rect(),
                         border_radius=box.h // 2)
        pygame.draw.rect(panel, (*ui.mix(D.COL_GOLD, self.accent, 0.4), 255),
                         panel.get_rect(), 1, border_radius=box.h // 2)
        if left < 0.3:
            panel.set_alpha(int(255 * left / 0.3))
            img.set_alpha(int(255 * left / 0.3))
        s.blit(panel, box)
        s.blit(img, img.get_rect(center=box.center))

    def _draw_strip(self, s):
        s.blit(self._strip_bg, self.strip.topleft)
        pygame.draw.line(s, ui.mix(D.COL_GOLD, self.accent, 0.35),
                         self.strip.topleft, self.strip.topright, 2)

    def _btn(self, s, rect, label, primary=False, on=True, hot=False,
             active=False):
        """Button im Theme-Look; primary = Akzentfläche mit sanftem Puls."""
        r = pygame.Rect(rect)
        rad = max(6, r.h // 4)
        if primary and on:
            base = ui.mix(self.accent, (0, 0, 0), 0.18)
            fill = ui.mix(base, (255, 255, 255), 0.12 if hot else 0.0)
            pygame.draw.rect(s, ui.mix(base, (0, 0, 0), 0.4), r.move(0, 2),
                             border_radius=rad)
            pygame.draw.rect(s, fill, r, border_radius=rad)
            border = ui.mix(D.COL_GOLD, (255, 255, 255),
                            0.35 * ui.pulse(2.4, 0.0, 1.0))
            pygame.draw.rect(s, border, r, 2, border_radius=rad)
            col = (255, 255, 255)
        else:
            fill = ui.BTN_SEL if (on and (hot or active)) else ui.BTN
            pygame.draw.rect(s, fill, r, border_radius=rad)
            edge = D.COL_GOLD if active else (ui.BORDER_LIGHT if on else ui.BORDER)
            pygame.draw.rect(s, edge, r, 2 if active else 1, border_radius=rad)
            col = ui.TEXT if on else ui.TEXT_FAINT
        fnt = self.f_btn
        img = fnt.render(label, True, col)
        if img.get_width() > r.w - 8:
            img = self.f_tiny.render(label, True, col)
        s.blit(img, img.get_rect(center=r.center))

    def _hot(self, rect):
        return pygame.Rect(rect).collidepoint(self.mouse)

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
                                        (t("cas.broke_restart",
                                           n=lamabank.START_CHIPS), ui.TEXT))):
            img = self.f_small.render(txt, True, col)
            if img.get_width() > pw - 16:
                img = self.f_tiny.render(txt, True, col)
            s.blit(img, img.get_rect(center=(cx, panel.y + ph * (0.58 + 0.2 * i))))
