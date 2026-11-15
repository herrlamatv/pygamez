# -*- coding: utf-8 -*-
"""
lamacrossing.py
===============
Lama Crossing - Mut-Spiel mit Lama-Chips: Das Lama überquert Schritt für
Schritt eine Hängebrücke, einen Fluss oder einen Eissee. Jeder geschaffte
Schritt hebt den Multiplikator; ein Fehltritt (Planke bricht, Stein rutscht
weg, Scholle bricht ein) kostet den Einsatz. Auszahlen geht nach jedem
Schritt, am anderen Ufer wird automatisch ausgezahlt.

- 4 Schwierigkeiten (Leicht 20 Schritte / Mittel 16 / Schwer 13 / Extrem 10),
  Risiko und Multiplikatoren in ``lamacrossing_logic.py`` (Quote ~97 %).
- 3 Welten (nur Optik): Hängebrücke über der Schlucht, Steine im Fluss,
  Eisschollen auf dem See - jede mit eigener Fehltritt-Animation.
- Nach dem Auszahlen zeigt das Spiel, wo es schiefgegangen wäre.
- Verlauf der letzten Runden oben rechts.

Lama-Chips: gemeinsames Konto der Lama-Bank (``lamabank.py``) mit Blackjack,
Poker und Casino. Der Einsatz wird bei "LOS" abgebucht, jeder Schritt sofort
gezogen und gespeichert: Wer mitten in der Überquerung geht, macht beim
nächsten Start genau dort weiter. Highscore = Höchststand von 1000 +
Lama-Crossing-Bilanz; ``game_over`` wird nie gesetzt.

Steuerung: Leertaste/→/Klick auf den nächsten Schritt = weiter (bzw. LOS),
Enter/C = auszahlen, ↑/↓/+/-/Mausrad = Einsatz, 1-4 = Schwierigkeit,
W = Welt wechseln.
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

from . import casino_draw as CD
from . import lamacrossing_draw as D
from . import lamacrossing_logic as L

KEY = "lamacrossing"
IDLE, HOP, WAIT, FALL, CASHED, GOAL = "idle", "hop", "wait", "fall", "cashed", "goal"
HOP_T = 0.36          # Dauer eines Sprungs
FALL_T = 1.25         # Fehltritt-Animation
RESULT_T = 3.2        # Ergebnis-Anzeige, danach zurück ans Startufer


class LamaCrossingGame(Game):
    name = "Lama Crossing"
    highscore_key = KEY
    supports_multiplayer = False

    # ===================================================== Aufbau / Reset
    def reset(self):
        self.game_over = False
        lamabank.load()
        self.rng = random.Random()
        world = self._opt("world", "bridge")
        self.world = world if world in L.WORLDS else "bridge"
        diff = self._opt("difficulty", "medium")
        self.diff = diff if diff in L.DIFFS else "medium"
        bet = self._opt("bet", L.BETS[0])
        self.bet = bet if bet in L.BETS else L.BETS[0]
        extra = lamabank.get_extra(KEY, {})
        extra = extra if isinstance(extra, dict) else {}
        self.hist = L.clean_history(extra.get("hist"))
        self.round = L.clean_round(extra.get("round"))
        self.step = 0
        self.state = IDLE
        self.hop = None
        self.outcome = None
        self.broken = 0               # Nummer des gebrochenen Schritts
        self.reveal = None            # nach dem Auszahlen: dort wäre es passiert
        self.fall_t = 0.0
        self.res_t = 0.0
        self.result = None            # (Titel, Untertitel, Farbe)
        self.win = 0
        self.lost = 0
        self.pending = 0              # gutgeschrieben, aber noch nicht gezeigt
        self.shown_chips = float(lamabank.balance())
        self.particles = []
        self.ripples = []
        self.coins_fx = []
        self.popups = []
        self.toast = None
        self.shake = 0.0
        self.time = 0.0
        self.mouse = (-1, -1)
        self.broke = False
        self._tick_cd = 0.0
        self._lang = get_language()
        self._layout()
        if self.round is not None:
            # Unterbrochene Überquerung genau dort fortsetzen
            self.diff = self.round["diff"]
            self.step = self.round["step"]
            self.state = WAIT
            self._say(t("lcr.resumed"))
        else:
            self._clamp_bet()
        self.cam = self._cam_target()
        self.score = lamabank.score_for(KEY)
        self._check_broke()

    def on_surface_changed(self):
        D.clear_cache()
        self._layout()
        self.cam = self._cam_target()

    def _layout(self):
        W, H = self.width, self.height
        self.k = k = min(W / 800.0, H / 600.0)
        self.f_tiny = ui.font(max(11, int(13 * k)))
        self.f_small = ui.font(max(12, int(15 * k)))
        self.f_btn = ui.font(max(12, int(15 * k)), bold=True)
        self.f_big = ui.font(max(15, int(22 * k)), bold=True)
        self.f_huge = ui.font(max(24, int(48 * k)), bold=True)
        self.margin = m = max(6, int(12 * k))
        self.hud_h = max(28, int(42 * k))
        self.strip_h = max(46, int(64 * k))
        self.strip = pygame.Rect(0, H - self.strip_h, W, self.strip_h)
        self.scene = pygame.Rect(0, self.hud_h, W, self.strip.y - self.hud_h)
        self.path_y = self.scene.y + int(self.scene.h * 0.56)
        self.gap = max(44, int(118 * k))
        self.tw = int(self.gap * 0.62)
        self.llama_n = max(30, int(72 * k))
        self.hop_h = int(44 * k)
        self.sag = int(16 * k)
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
        # Bedienleiste: Einsatz links, LOS/WEITER + AUSZAHLEN rechts
        sh = self.strip_h
        bh = max(30, int(sh * 0.62))
        by = self.strip.y + (sh - bh) // 2
        lbl_w = max(self.f_tiny.size(t("cas.bet"))[0], self.f_btn.size("10000")[0]) + int(14 * k)
        self.r_minus = pygame.Rect(m, by, bh, bh)
        self.r_bet = pygame.Rect(self.r_minus.right + 4, by, lbl_w, bh)
        self.r_plus = pygame.Rect(self.r_bet.right + 4, by, bh, bh)
        pad = int(22 * max(0.7, k))
        go_w = max(self.f_btn.size(t("lcr.go"))[0], self.f_btn.size(t("lcr.step"))[0]) + pad
        go_w = max(go_w, int(110 * k))
        cash_w = self.f_btn.size(t("lcr.cashout") + " 1000000")[0] + pad
        room = W - m - (self.r_plus.right + m) - max(4, int(8 * k))
        if go_w + cash_w > room:
            f = room / (go_w + cash_w)
            go_w, cash_w = int(go_w * f), int(cash_w * f)
        self.r_go = pygame.Rect(W - m - go_w, by, go_w, bh)
        self.r_cash = pygame.Rect(self.r_go.x - max(4, int(8 * k)) - cash_w, by, cash_w, bh)
        # Schwierigkeit (nur vor der Runde) direkt über der Leiste
        bar_h = max(26, int(36 * k))
        bar_y = self.strip.y - m - bar_h
        bw = min(int(150 * k), (W - 2 * m - 3 * 6) // 4)
        x0 = (W - (4 * bw + 3 * 6)) // 2
        self.r_diffs = [pygame.Rect(x0 + i * (bw + 6), bar_y, bw, bar_h) for i in range(4)]
        # Welt-Knöpfe oben links
        wb = max(26, int(36 * k))
        self.r_worlds = [pygame.Rect(m + i * (wb + 4), self.hud_h + m, wb, wb) for i in range(3)]

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

    # ===================================================== Konto / Runde
    def _save_extra(self):
        lamabank.set_extra(KEY, {"round": self.round, "hist": self.hist})

    def _clamp_bet(self):
        """Einsatz auf die höchste bezahlbare Stufe senken (mind. Stufe 1)."""
        chips = lamabank.balance()
        if self.bet > chips:
            ok = [b for b in L.BETS if b <= chips]
            self.bet = ok[-1] if ok else L.BETS[0]

    def _check_broke(self):
        self.broke = self.state == IDLE and lamabank.is_broke(KEY)
        if self.broke:
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

    def _sep(self):
        return t("lcr.decimal")

    def _fmt(self, m100):
        return "×" + L.fmt_mult(m100, self._sep())

    # ===================================================== Geometrie
    def _n(self):
        return L.steps(self.diff)

    def _tile_x(self, i):
        """Weltkoordinate x der Schritt-Mitte (0 = Startufer, n+1 = Zielufer)."""
        return int(self.width * 0.22) + i * self.gap

    def _tile_y(self, i):
        n = self._n()
        if self.world == "bridge" and 1 <= i <= n:
            return self.path_y + int(self.sag * math.sin(math.pi * i / (n + 1)))
        return self.path_y

    def _llama_pos(self):
        """(x, y) der Füße in Weltkoordinaten + Pose."""
        if self.state == HOP and self.hop is not None:
            a, b, p = self.hop["a"], self.hop["b"], min(1.0, self.hop["t"] / HOP_T)
            e = p * p * (3 - 2 * p)
            x = self._tile_x(a) + (self._tile_x(b) - self._tile_x(a)) * e
            y = self._tile_y(a) + (self._tile_y(b) - self._tile_y(a)) * p
            return x, y - math.sin(math.pi * p) * self.hop_h, "hop"
        x, y = self._tile_x(self.step), self._tile_y(self.step)
        if self.state == FALL:
            f = self.fall_t
            if self.world == "bridge":
                return x, y + 0.5 * 1500 * self.k * f * f, "fall"
            return x, y + 46 * self.k * min(1.0, f * 1.6), "fall"
        if self.state in (WAIT, IDLE) and self.world == "bridge" and self.step > 0:
            y += math.sin(self.time * 3.1) * 1.2 * self.k
        return x, y, "stand"

    def _cam_target(self):
        n = self._n()
        if self.state == HOP and self.hop is not None:
            lx = self._tile_x(self.hop["b"])
        else:
            lx = self._tile_x(self.step)
        hi = max(0, self._tile_x(n + 1) + int(self.gap * 1.3) - self.width)
        return max(0.0, min(float(hi), lx - self.width * 0.32))

    # ===================================================== Ablauf
    def _go(self):
        """LOS (vor der Runde) bzw. WEITER (auf einem Schritt)."""
        if self.state == IDLE:
            if self.broke:
                return
            if self.bet > lamabank.balance():
                self._say(t("cas.not_enough"))
                self.play_sound("hit")
                return
            if not lamabank.debit(self.bet, KEY):
                return
            self.round = {"bet": self.bet, "diff": self.diff, "step": 0}
            self.step = 0
            self.reveal = None
            self.play_sound("select")
            self._step()
        elif self.state == WAIT:
            self._step()

    def _step(self):
        n = self._n()
        k = self.step + 1
        bet = self.round["bet"]
        if L.roll(self.rng, self.diff):
            if k >= n:
                self.win = L.payout(bet, self.diff, n)
                lamabank.credit(self.win, KEY)
                self.pending += self.win
                self.hist = (self.hist + [L.mult(self.diff, n)])[-L.HISTORY:]
                self.round = None
                self.outcome = "goal"
            else:
                self.round["step"] = k
                self.outcome = "ok"
        else:
            self.hist = (self.hist + [0])[-L.HISTORY:]
            self.round = None
            self.outcome = "fail"
            self.broken = k
            self.lost = bet
        self._save_extra()
        self.score = lamabank.score_for(KEY)
        self.hop = {"a": self.step, "b": k, "t": 0.0}
        self.state = HOP
        self.play_sound("move")

    def _land(self):
        b = self.hop["b"]
        self.step = b
        x, y = self._tile_x(b), self._tile_y(b)
        n = self._n()
        if b > n:
            # am Zielufer angekommen
            self.hop = None
            self.state = GOAL
            self.res_t = 0.0
            self.pending = max(0, self.pending - self.win)
            mult = L.mult(self.diff, n)
            self.result = (t("lcr.goal"), t("lcr.cashed", n=self.win), ui.GOLD)
            ui.spawn_confetti(self.width, self.height, 120)
            self._coin_shower(70)
            self._popup(x - self.cam, y - self.llama_n - 10 * self.k, "+%d" % self.win)
            self.play_sound("level")
            self.rumble(220)
            self.ach_event("lcr_other_side")
            if self.diff == "extreme":
                self.ach_event("lcr_extreme")
            if mult >= L.BRAVE_MULT:
                self.ach_event("lcr_brave")
            return
        if self.outcome == "fail":
            self.hop = None
            self.state = FALL
            self.fall_t = 0.0
            self.res_t = 0.0
            self.shake = 0.35
            self.result = (t("lcr.fall." + self.world), t("lcr.lost", n=self.lost), ui.RED)
            self._spawn_fail(x, y)
            self.play_sound("hit")
            self.rumble(260)
            return
        self._dust(x, y)
        if self.outcome == "goal":
            # letzter Schritt geschafft: weiter ans Zielufer hüpfen
            self.hop = {"a": b, "b": b + 1, "t": 0.0}
            self.play_sound("point")
            return
        self.hop = None
        self.state = WAIT
        self.play_sound("point")
        self._popup(x - self.cam, y - self.llama_n - 10 * self.k,
                    self._fmt(L.mult(self.diff, b)), ui.GREEN, 0.9)

    def _cash_out(self):
        if self.state != WAIT or self.round is None or self.step < 1:
            return
        bet = self.round["bet"]
        mult = L.mult(self.diff, self.step)
        self.win = L.payout(bet, self.diff, self.step)
        lamabank.credit(self.win, KEY)
        self.hist = (self.hist + [mult])[-L.HISTORY:]
        self.round = None
        self.reveal = L.next_break(self.rng, self.diff, self.step)
        self._save_extra()
        self.score = lamabank.score_for(KEY)
        self.state = CASHED
        self.res_t = 0.0
        sub = t("lcr.reveal", n=self.reveal) if self.reveal is not None else t("lcr.reveal_none")
        self.result = (t("lcr.cashed", n=self.win), sub, ui.GOLD)
        x, y = self._tile_x(self.step), self._tile_y(self.step)
        self._popup(x - self.cam, y - self.llama_n - 10 * self.k, "+%d" % self.win)
        if mult >= 300:
            self._coin_shower(30 if mult < L.BRAVE_MULT else 60)
        self.play_sound("win")
        if mult >= L.BRAVE_MULT:
            self.ach_event("lcr_brave")

    def _to_idle(self):
        self.state = IDLE
        self.step = 0
        self.hop = None
        self.result = None
        self.reveal = None
        self.outcome = None
        self.broken = 0
        self.ripples = []
        self._dust(self._tile_x(0), self._tile_y(0))
        self._clamp_bet()
        self._check_broke()

    def _set_bet(self, direction):
        if self.state != IDLE:
            return
        i = L.BETS.index(self.bet)
        j = max(0, min(len(L.BETS) - 1, i + direction))
        if j != i:
            self.bet = L.BETS[j]
            self._set_opt("bet", self.bet)
            self.play_sound("select")
            if self.bet > lamabank.balance():
                self._say(t("cas.not_enough"), 1.2)

    def _set_diff(self, i):
        if self.state != IDLE or L.DIFFS[i] == self.diff:
            return
        self.diff = L.DIFFS[i]
        self._set_opt("difficulty", self.diff)
        self.cam = self._cam_target()
        self.play_sound("click")

    def _set_world(self, i):
        if self.state != IDLE or L.WORLDS[i] == self.world:
            return
        self.world = L.WORLDS[i]
        self._set_opt("world", self.world)
        self.particles = []
        self.play_sound("click")

    # ===================================================== Effekte
    def _spawn(self, x, y, n, col, speed, up, r, life, grav):
        for _ in range(n):
            a = self.rng.uniform(math.pi * 1.05, math.pi * 1.95) if up else \
                self.rng.uniform(0, math.tau)
            sp = speed * self.rng.uniform(0.35, 1.0) * self.k
            self.particles.append(dict(x=x, y=y, vx=math.cos(a) * sp, vy=math.sin(a) * sp,
                                       r=r * self.k * self.rng.uniform(0.6, 1.2),
                                       col=col, t=0.0, life=life * self.rng.uniform(0.7, 1.1),
                                       g=grav * self.k))
        if len(self.particles) > 220:
            self.particles = self.particles[-220:]

    def _dust(self, x, y):
        col = {"bridge": (196, 160, 120), "river": (200, 230, 246), "ice": (240, 248, 255)}[self.world]
        self._spawn(x, y, 8, col, 90, True, 3.0, 0.45, 260)

    def _spawn_fail(self, x, y):
        if self.world == "bridge":
            self._spawn(x, y, 16, (150, 98, 58), 220, True, 3.2, 1.1, 900)
            self._spawn(x, y, 6, (196, 160, 120), 120, True, 4.0, 0.6, 300)
        elif self.world == "river":
            self._spawn(x, y + 10 * self.k, 22, (214, 238, 250), 260, True, 3.4, 0.8, 900)
            self.ripples.append(dict(x=x, y=y + 12 * self.k, t=0.0))
            self.ripples.append(dict(x=x, y=y + 12 * self.k, t=-0.25))
        else:
            self._spawn(x, y, 14, (232, 246, 255), 200, True, 3.0, 0.9, 800)
            self._spawn(x, y + 8 * self.k, 14, (150, 206, 236), 240, True, 3.0, 0.8, 900)
            self.ripples.append(dict(x=x, y=y + 14 * self.k, t=-0.3))

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
                    kind == InputEvent.KEYDOWN and event.key in ("Return", "space", "KP_Enter")):
                self._take_credit()
            return
        if self.state in (FALL, CASHED, GOAL):
            # Ergebnis wegklicken (frühestens nach kurzer Zeit)
            if self.res_t > 0.6 and (kind == InputEvent.MOUSEDOWN or (
                    kind == InputEvent.KEYDOWN and event.key in ("Return", "space", "KP_Enter"))):
                self._to_idle()
            return
        if kind == InputEvent.KEYDOWN:
            key = event.key
            if key in ("space", "Right"):
                self._go()
            elif key in ("Return", "KP_Enter"):
                if self.state == WAIT:
                    self._cash_out()
                else:
                    self._go()
            elif key in ("c", "C"):
                self._cash_out()
            elif key in ("plus", "KP_Add", "Up", "equal"):
                self._set_bet(1)
            elif key in ("minus", "KP_Subtract", "Down"):
                self._set_bet(-1)
            elif key in ("1", "2", "3", "4"):
                self._set_diff(int(key) - 1)
            elif key in ("w", "W"):
                self._set_world((L.WORLDS.index(self.world) + 1) % len(L.WORLDS))
            return
        if kind == InputEvent.WHEEL:
            self._set_bet(1 if event.delta > 0 else -1)
            return
        if kind != InputEvent.MOUSEDOWN or event.button != 1:
            return
        pos = event.pos
        if self.r_minus.collidepoint(pos):
            self._set_bet(-1)
        elif self.r_plus.collidepoint(pos):
            self._set_bet(1)
        elif self.r_go.collidepoint(pos):
            self._go()
        elif self.r_cash.collidepoint(pos):
            self._cash_out()
        elif self.state == IDLE:
            for i, r in enumerate(self.r_diffs):
                if r.collidepoint(pos):
                    self._set_diff(i)
                    return
            for i, r in enumerate(self.r_worlds):
                if r.collidepoint(pos):
                    self._set_world(i)
                    return
        elif self.state == WAIT and self._next_tile_rect().collidepoint(pos):
            self._go()

    def _next_tile_rect(self):
        i = self.step + 1
        x = self._tile_x(i) - self.cam
        y = self._tile_y(i)
        return pygame.Rect(int(x - self.gap * 0.45), int(y - self.llama_n), int(self.gap * 0.9),
                           int(self.llama_n + self.tw * 0.6))

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
        self.shake = max(0.0, self.shake - dt)
        self._update_fx(dt)
        tgt = self._cam_target()
        self.cam += (tgt - self.cam) * min(1.0, dt * 6.0)
        if abs(tgt - self.cam) < 0.5:
            self.cam = tgt
        if self.state == HOP:
            self.hop["t"] += dt
            if self.hop["t"] >= HOP_T:
                self._land()
        elif self.state in (FALL, CASHED, GOAL):
            self.fall_t += dt
            self.res_t += dt
            if self.res_t >= RESULT_T + (FALL_T if self.state == FALL else 0.0):
                self._to_idle()

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
        for r in self.ripples:
            r["t"] += dt
        self.ripples = [r for r in self.ripples if r["t"] < 1.1]
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
        bg = D.background(self.world, W, H, self.path_y)
        s.blit(bg, (0, 0))
        far = D.far_layer(self.world, W, H, self.path_y)
        off = -int(self.cam * 0.18) % W
        s.blit(far, (off - W, 0))
        s.blit(far, (off, 0))
        ox = -self.cam
        if self.shake > 0:
            ox += math.sin(self.time * 70) * 6 * self.k * self.shake / 0.35
        self._draw_water(s, ox)
        self._draw_world(s, ox)
        self._draw_llama(s, ox)
        self._draw_particles(s, ox)
        if self.world == "ice":
            self._draw_snow(s, 1)
        self._draw_hud(s)
        self._draw_history(s)
        self._draw_worlds(s)
        if self.state == IDLE:
            self._draw_setup(s)
        elif self.state == WAIT:
            self._draw_hint(s, t("lcr.hint_wait"))
        self._draw_strip(s)
        self._draw_fx(s)
        if self.result is not None and (self.state != FALL or self.fall_t > FALL_T * 0.55):
            self._draw_result(s)
        if self.toast:
            self._draw_toast(s)
        if self.broke:
            self._draw_broke(s)

    # ----- Welt -------------------------------------------------------------
    def _draw_water(self, s, ox):
        """Bewegte Wasser-/Nebel-Details hinter den Kacheln."""
        k = self.k
        W = self.width
        if self.world == "river":
            for i in range(26):
                row = i % 6
                y = self.path_y - int(self.height * 0.06) + row * int(20 * k) + int(10 * k)
                if y > self.height:
                    continue
                sp = 28 + row * 9
                x = (i * 137 + self.time * sp * k + ox * 0.6) % (W + 60) - 30
                pygame.draw.line(s, (150, 204, 232), (x, y), (x + 22 * k, y), max(1, int(2 * k)))
        elif self.world == "ice":
            self._draw_snow(s, 0)
            for i in range(10):
                x = (i * 211 + ox * 0.9) % (W + 80) - 40
                y = self.path_y + int(40 * k) + (i % 3) * int(26 * k)
                pygame.draw.line(s, (40, 92, 124), (x, y), (x + 40 * k, y), max(1, int(2 * k)))
        else:
            # ein paar Vögel über der Schlucht
            for i in range(3):
                x = (i * 300 + self.time * 25 * k) % (W + 100) - 50
                y = self.path_y * 0.30 + i * 22 * k + math.sin(self.time * 2 + i) * 4 * k
                wing = math.sin(self.time * 9 + i * 2) * 3 * k
                pygame.draw.lines(s, (70, 50, 60), False, [(x - 7 * k, y - wing), (x, y + 2 * k),
                                                           (x + 7 * k, y - wing)], max(1, int(2 * k)))

    def _draw_snow(self, s, layer):
        k = self.k
        W, H = self.width, self.height
        for i in range(layer, 60, 2):
            x = (i * 97.3 + self.time * (10 + i % 5 * 4) * k + math.sin(self.time + i) * 8 * k) % W
            y = self.hud_h + (i * 53.1 + self.time * (24 + i % 7 * 5) * k) % (H - self.hud_h)
            pygame.draw.circle(s, (255, 255, 255), (int(x), int(y)), max(1, int((1 + layer) * k)))

    def _draw_world(self, s, ox):
        k = self.k
        n = self._n()
        H = self.height
        xl = self._tile_x(0) + self.gap * 0.42 + ox
        xr = self._tile_x(n + 1) - self.gap * 0.42 + ox
        if self.world == "bridge":
            pts = [(xl, self.path_y)]
            for i in range(1, n + 1):
                if self.state == FALL and i == self.broken:
                    continue
                pts.append((self._tile_x(i) + ox, self._tile_y(i) + 4 * k))
            pts.append((xr, self.path_y))
            D.draw_ropes(s, pts, k)
        D.draw_bank(s, self.world, xl, -1, self.path_y, H, k)
        D.draw_bank(s, self.world, xr, 1, self.path_y, H, k)
        self._draw_goal_flag(s, ox)
        cur = self.step if self.state in (WAIT, HOP, FALL, CASHED, GOAL) else 0
        for i in range(1, n + 1):
            cx = self._tile_x(i) + ox
            if cx < -self.gap or cx > self.width + self.gap:
                continue
            y = self._tile_y(i)
            if self.state == FALL and i == self.broken:
                self._draw_broken_tile(s, cx, y, i)
            else:
                if self.state == WAIT and i == self.step + 1:
                    glow = pygame.Rect(0, 0, int(self.tw * 1.3), int(self.tw * 0.5))
                    glow.center = (int(cx), int(y + self.tw * 0.12))
                    col = ui.mix(CD.COL_GOLD, (255, 255, 255), 0.3 * ui.pulse(3.0, 0.0, 1.0))
                    pygame.draw.ellipse(s, col, glow, max(1, int(2 * k)))
                D.draw_tile(s, self.world, cx, y, self.tw, k, self.time, i)
                if self.state == CASHED and self.reveal == i:
                    self._draw_cross(s, cx, y)
            self._draw_label(s, cx, y, i, cur)

    def _draw_goal_flag(self, s, ox):
        k = self.k
        n = self._n()
        x = self._tile_x(n + 1) + int(self.gap * 0.55) + ox
        if x < -40 or x > self.width + 40:
            return
        y = self.path_y
        pole = int(70 * k)
        pygame.draw.line(s, (90, 70, 60), (x, y), (x, y - pole), max(2, int(3 * k)))
        wave = math.sin(self.time * 4) * 3 * k
        pygame.draw.polygon(s, self.accent, [(x, y - pole), (x + 34 * k, y - pole + 8 * k + wave),
                                             (x, y - pole + 18 * k)])
        txt = self._fmt(L.mult(self.diff, n))
        img = self.f_small.render(txt, True, CD.COL_GOLD)
        s.blit(img, img.get_rect(midbottom=(int(x), int(y - pole - 4 * k))))

    def _draw_broken_tile(self, s, cx, y, i):
        k = self.k
        f = self.fall_t
        if self.world == "river":
            alpha = int(255 * max(0.0, 1 - f / 0.8))
            D.draw_tile(s, self.world, cx, y + 26 * k * min(1.0, f), self.tw, k, self.time, i, alpha)
            return
        if self.world == "ice" and f < 0.3:
            D.draw_tile(s, self.world, cx, y, self.tw, k, self.time, i)
            for line in D.crack_lines(cx, y, self.tw, i, f / 0.3):
                pygame.draw.lines(s, (40, 92, 124), False, line, max(1, int(2 * k)))
            return
        g = f - (0.3 if self.world == "ice" else 0.0)
        layer = pygame.Surface((int(self.tw * 1.4) + 8, int(self.tw * 0.9) + 8), pygame.SRCALPHA)
        lx = layer.get_width() / 2
        D.draw_tile(layer, self.world, lx, int(self.tw * 0.2), self.tw, k, self.time, i)
        half = layer.get_width() // 2
        drop = 0.5 * 1100 * k * g * g if self.world == "bridge" else 30 * k * g
        alpha = int(255 * max(0.0, 1 - g / 0.9))
        for side in (0, 1):
            piece = layer.subsurface((side * half, 0, half, layer.get_height())).copy()
            piece.set_alpha(alpha)
            dx = (-1 if side == 0 else 1) * (6 + 40 * g) * k
            s.blit(piece, (cx - lx + side * half + dx, y - int(self.tw * 0.2) + drop))

    def _draw_cross(self, s, cx, y):
        """Markiert den Schritt, an dem es nach dem Auszahlen schiefgegangen wäre."""
        d = self.tw * 0.22
        c = (cx, y + self.tw * 0.10)
        w = max(2, int(3 * self.k))
        pygame.draw.line(s, ui.RED, (c[0] - d, c[1] - d * 0.6), (c[0] + d, c[1] + d * 0.6), w)
        pygame.draw.line(s, ui.RED, (c[0] - d, c[1] + d * 0.6), (c[0] + d, c[1] - d * 0.6), w)

    def _draw_label(self, s, cx, y, i, cur):
        """Multiplikator-Schild unter einem Schritt."""
        txt = self._fmt(L.mult(self.diff, i))
        if self.state == FALL and i == self.broken:
            fg, bg = (255, 255, 255), ui.RED
        elif self.state == CASHED and self.reveal == i:
            fg, bg = (255, 220, 220), (120, 30, 40)
        elif i <= cur:
            fg, bg = (230, 255, 236), (40, 120, 72)
        elif i == cur + 1:
            fg, bg = (30, 24, 10), CD.COL_GOLD
        else:
            fg, bg = (232, 232, 240), (24, 22, 34)
        img = self.f_tiny.render(txt, True, fg)
        box = img.get_rect()
        box.inflate_ip(int(10 * self.k) + 4, int(4 * self.k) + 2)
        box.midtop = (int(cx), int(y + self.tw * 0.36 + 6 * self.k))
        pygame.draw.rect(s, bg, box, border_radius=box.h // 2)
        s.blit(img, img.get_rect(center=box.center))

    def _draw_llama(self, s, ox):
        x, y, pose = self._llama_pos()
        n = self.llama_n
        img = D.llama_sprite(n, pose, self.accent)
        sx = x + ox
        if self.state != FALL and pose != "hop":
            shadow = pygame.Rect(0, 0, int(n * 0.5), max(3, int(n * 0.08)))
            shadow.center = (int(sx + n * 0.02), int(y + 1))
            pygame.draw.ellipse(s, (0, 0, 0), shadow)
        if self.state == FALL:
            f = self.fall_t
            fade = 1.0 - min(1.0, max(0.0, (f - (0.5 if self.world == "bridge" else 0.15)) / 0.5))
            if fade <= 0:
                return
            img = img.copy()
            img.set_alpha(int(255 * fade))
        s.blit(img, (int(sx - n / 2), int(y - n + n * 0.06)))

    def _draw_particles(self, s, ox):
        for p in self.particles:
            r = p["r"] * (1 - p["t"] / p["life"])
            if r >= 0.5:
                pygame.draw.circle(s, p["col"], (int(p["x"] + ox), int(p["y"])), max(1, int(r)))
        for rp in self.ripples:
            if rp["t"] <= 0:
                continue
            w = int((20 + 140 * rp["t"]) * self.k)
            rect = pygame.Rect(0, 0, w, max(2, w // 4))
            rect.center = (int(rp["x"] + ox), int(rp["y"]))
            pygame.draw.ellipse(s, (210, 236, 250), rect, max(1, int(2 * self.k)))

    # ----- Oberfläche -------------------------------------------------------
    def _draw_hud(self, s):
        W = self.width
        s.blit(self._hud_bg, (0, 0))
        pygame.draw.line(s, ui.mix(CD.COL_GOLD, self.accent, 0.35),
                         (0, self.hud_h - 1), (W, self.hud_h - 1), 1)
        cy = self.hud_h // 2
        pad = self.margin
        coin_r = max(6, int(self.hud_h * 0.26))
        coin = CD.symbol_surface("coin", coin_r * 2 + 4)
        s.blit(coin, coin.get_rect(midleft=(pad - 2, cy)))
        left = self.f_big.render(f"{t('cas.chips')}: {int(round(self.shown_chips))}", True, ui.GOLD)
        s.blit(left, left.get_rect(midleft=(pad + coin_r * 2 + 6, cy)))
        right_txt = f"{t('cas.record')}: {self.score}   ·   {t('cas.credits', n=lamabank.refills())}"
        right = self.f_small.render(right_txt, True, ui.TEXT_DIM)
        if right.get_width() > W * 0.42:
            right = self.f_tiny.render(right_txt, True, ui.TEXT_DIM)
        rr = right.get_rect(midright=(W - pad, cy))
        s.blit(right, rr)
        bet = self._round_bet()
        mid_txt = ""
        if self.round is not None:
            mid_txt = f"{t('cas.bet')}: {bet}"
            if self.step > 0:
                mid_txt += f"   ·   {self._fmt(L.mult(self.diff, self.step))} = {L.payout(bet, self.diff, self.step)}"
        if not mid_txt:
            return
        x0 = pad + coin_r * 2 + 6 + left.get_width() + pad
        x1 = rr.left - pad
        mid = self.f_small.render(mid_txt, True, ui.TEXT)
        if mid.get_width() > x1 - x0:
            mid = self.f_tiny.render(mid_txt, True, ui.TEXT)
        if mid.get_width() <= x1 - x0:
            s.blit(mid, mid.get_rect(center=((x0 + x1) // 2, cy)))

    def _draw_history(self, s):
        """Letzte Runden als Schilder oben rechts (neueste rechts)."""
        x = self.width - self.margin
        y = self.hud_h + self.margin
        left_limit = self.r_worlds[-1].right + self.margin + self.f_small.size(
            t("lcr.world." + self.world))[0] + self.margin
        for m100 in reversed(self.hist):
            txt = self._fmt(m100) if m100 > 0 else "×0"
            img = self.f_tiny.render(txt, True, (240, 240, 245))
            box = img.get_rect()
            box.inflate_ip(int(10 * self.k) + 4, int(6 * self.k) + 2)
            box.topright = (x, y)
            if box.left < left_limit:
                break
            col = (40, 120, 72) if m100 >= 200 else ((70, 96, 70) if m100 > 0 else (128, 36, 46))
            pygame.draw.rect(s, col, box, border_radius=box.h // 2)
            s.blit(img, img.get_rect(center=box.center))
            x = box.left - 4

    def _draw_worlds(self, s):
        on = self.state == IDLE
        for i, r in enumerate(self.r_worlds):
            w = L.WORLDS[i]
            active = w == self.world
            hot = on and r.collidepoint(self.mouse)
            fill = ui.BTN_SEL if (active or hot) else ui.BTN
            pygame.draw.rect(s, fill, r, border_radius=max(4, r.h // 4))
            edge = CD.COL_GOLD if active else (ui.BORDER_LIGHT if on else ui.BORDER)
            pygame.draw.rect(s, edge, r, 2 if active else 1, border_radius=max(4, r.h // 4))
            D.world_icon(s, w, r.inflate(-r.w // 5, -r.h // 5),
                         ui.TEXT if (on or active) else ui.TEXT_FAINT)
        name = self.f_small.render(t("lcr.world." + self.world), True, ui.TEXT)
        last = self.r_worlds[-1]
        s.blit(name, name.get_rect(midleft=(last.right + self.margin, last.centery)))

    def _draw_setup(self, s):
        hot = self.mouse
        for i, r in enumerate(self.r_diffs):
            self._btn(s, r, f"{i + 1}  {t('lcr.diff.' + L.DIFFS[i])}",
                      active=L.DIFFS[i] == self.diff, hot=r.collidepoint(hot))
        info = t("lcr.info", n=self._n(), p=L.risk_percent(self.diff),
                 m=L.fmt_mult(L.mult(self.diff, self._n()), self._sep()))
        self._draw_hint(s, info, self.r_diffs[0].y - 4)

    def _draw_hint(self, s, text, bottom=None):
        img = self.f_tiny.render(text, True, ui.TEXT)
        box = img.get_rect()
        box.inflate_ip(18, 8)
        box.midbottom = (self.width // 2, bottom if bottom is not None else self.strip.y - self.margin)
        panel = pygame.Surface(box.size, pygame.SRCALPHA)
        pygame.draw.rect(panel, (12, 8, 20, 190), panel.get_rect(), border_radius=box.h // 2)
        s.blit(panel, box)
        s.blit(img, img.get_rect(center=box.center))

    def _draw_strip(self, s):
        s.blit(self._strip_bg, self.strip.topleft)
        pygame.draw.line(s, ui.mix(CD.COL_GOLD, self.accent, 0.35),
                         self.strip.topleft, self.strip.topright, 2)
        idle = self.state == IDLE
        bet = self._round_bet()
        hot = self.mouse
        self._btn(s, self.r_minus, "–", on=idle and self.bet > L.BETS[0], hot=self.r_minus.collidepoint(hot))
        self._btn(s, self.r_plus, "+", on=idle and self.bet < L.BETS[-1], hot=self.r_plus.collidepoint(hot))
        pygame.draw.rect(s, (16, 12, 26), self.r_bet, border_radius=max(6, self.r_bet.h // 4))
        pygame.draw.rect(s, ui.BORDER, self.r_bet, 1, border_radius=max(6, self.r_bet.h // 4))
        lbl = self.f_tiny.render(t("cas.bet"), True, ui.TEXT_DIM)
        s.blit(lbl, lbl.get_rect(midtop=(self.r_bet.centerx, self.r_bet.y + 2)))
        col = ui.RED if (idle and bet > lamabank.balance()) else ui.GOLD
        val = self.f_btn.render(str(bet), True, col)
        s.blit(val, val.get_rect(midbottom=(self.r_bet.centerx, self.r_bet.bottom - 3)))
        wait = self.state == WAIT
        go_lbl = t("lcr.go") if idle else t("lcr.step")
        self._btn(s, self.r_go, go_lbl, primary=True, on=(idle and not self.broke) or wait,
                  hot=self.r_go.collidepoint(hot))
        cash = t("lcr.cashout")
        if wait and self.round is not None:
            cash += " %d" % L.payout(self.round["bet"], self.diff, self.step)
        self._btn(s, self.r_cash, cash, on=wait, hot=self.r_cash.collidepoint(hot), active=wait)

    def _btn(self, s, rect, label, primary=False, on=True, hot=False, active=False):
        r = pygame.Rect(rect)
        rad = max(6, r.h // 4)
        if primary and on:
            base = ui.mix(self.accent, (0, 0, 0), 0.18)
            fill = ui.mix(base, (255, 255, 255), 0.12 if hot else 0.0)
            pygame.draw.rect(s, ui.mix(base, (0, 0, 0), 0.4), r.move(0, 2), border_radius=rad)
            pygame.draw.rect(s, fill, r, border_radius=rad)
            border = ui.mix(CD.COL_GOLD, (255, 255, 255), 0.35 * ui.pulse(2.4, 0.0, 1.0))
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
        s.blit(img, img.get_rect(center=r.center))

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
            y = p["y"] - 40 * self.k * (1 - (1 - f) ** 2)
            s.blit(img, img.get_rect(center=(int(p["x"]), int(y))))

    def _draw_result(self, s):
        title, sub, col = self.result
        W = self.width
        k = self.k
        timg = self.f_huge.render(title, True, col)
        if timg.get_width() > W - 40:
            timg = self.f_big.render(title, True, col)
        simg = self.f_small.render(sub, True, ui.TEXT)
        if simg.get_width() > W - 40:
            simg = self.f_tiny.render(sub, True, ui.TEXT)
        pw = min(W - 20, max(timg.get_width(), simg.get_width()) + int(48 * k))
        ph = timg.get_height() + simg.get_height() + int(34 * k)
        panel = pygame.Rect(0, 0, pw, ph)
        panel.center = (W // 2, self.scene.y + int(self.scene.h * 0.24) + ph // 2 - int(10 * k))
        ui.draw_panel(s, panel, accent_top=col)
        s.blit(timg, timg.get_rect(midtop=(panel.centerx, panel.y + int(12 * k))))
        s.blit(simg, simg.get_rect(midbottom=(panel.centerx, panel.bottom - int(12 * k))))

    def _draw_toast(self, s):
        text, left = self.toast
        img = self.f_small.render(text, True, ui.TEXT)
        box = img.get_rect()
        box.inflate_ip(24, 12)
        box.midtop = (self.width // 2, self.hud_h + self.margin + self.r_worlds[0].h + self.margin)
        panel = pygame.Surface(box.size, pygame.SRCALPHA)
        pygame.draw.rect(panel, (12, 8, 20, 225), panel.get_rect(), border_radius=box.h // 2)
        pygame.draw.rect(panel, (*ui.mix(CD.COL_GOLD, self.accent, 0.4), 255),
                         panel.get_rect(), 1, border_radius=box.h // 2)
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
        head = self.f_huge.render(t("cas.broke"), True, ui.RED)
        s.blit(head, head.get_rect(center=(cx, panel.y + ph * 0.28)))
        for i, (txt, col) in enumerate(((t("cas.broke_sub"), ui.TEXT_DIM),
                                        (t("cas.broke_restart", n=lamabank.START_CHIPS), ui.TEXT))):
            img = self.f_small.render(txt, True, col)
            if img.get_width() > pw - 16:
                img = self.f_tiny.render(txt, True, col)
            s.blit(img, img.get_rect(center=(cx, panel.y + ph * (0.58 + 0.2 * i))))
