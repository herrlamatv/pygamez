# -*- coding: utf-8 -*-
"""
sudoku_draw.py
==============
Zeichen-Teil von Sudoku (Mixin für ``SudokuGame`` in sudoku.py):

- Setup-Screen: Varianten- und Stufenwahl, Levelraster mit Sternen und
  "angefangen"-Markierung, Tages-Sudoku-Karte, Schalter, Level-Info.
- Spiel: HUD, Brett (Killer-Käfige gestrichelt mit Summen, X-Diagonalen,
  Notizen, Farbmarker, Konflikte, Leucht-Effekte), Ziffernfeld mit
  Restziffer-Zähler und Symbol-Knöpfen, Ergebnis-Panel mit Sternen.

Teure, statische Flächen (Käfig-Linien, Diagonalen, Abdunkelung) und alle
Ziffern-Glyphen werden gecacht - so bleibt auch 1280x960 flüssig.
Farben immer zur Zeichenzeit aus ui.* lesen (Theme-Wechsel).
"""

import math

import pygame

import ui
from i18n import t

# Eigene Ziffern bleiben bewusst blau (klassische Sudoku-Optik).
COL_USER = (110, 165, 255)

# Farbmarker 1..6 (Rot, Orange, Gelb, Grün, Blau, Lila)
MARK_COLORS = ((232, 92, 92), (240, 150, 60), (228, 204, 70),
               (96, 196, 118), (84, 160, 240), (176, 116, 226))

# Symbol-Knöpfe: i18n-Schlüssel der Tooltips und feste Tasten
PAD_TIPS = {"erase": "sud.tip.erase", "undo": "sud.tip.undo",
            "redo": "sud.tip.redo", "note": "sud.tip.note",
            "cands": "sud.tip.cands", "hint": "sud.tip.hint"}
PAD_KEYS = {"erase": "0", "undo": "U", "redo": "Y", "note": "N",
            "cands": "C", "hint": "H"}

_S = None


def _sd():
    """sudoku.py-Konstanten (später Import - sudoku.py importiert uns)."""
    global _S
    if _S is None:
        from . import sudoku as mod
        _S = mod
    return _S


def star_points(cx, cy, r):
    """Eckpunkte eines fünfzackigen Sterns (Spitze oben)."""
    pts = []
    for k in range(10):
        rad = r if k % 2 == 0 else r * 0.45
        a = -math.pi / 2 + k * math.pi / 5
        pts.append((cx + math.cos(a) * rad, cy + math.sin(a) * rad))
    return pts


def draw_star(s, center, r, color, filled=True):
    pts = star_points(center[0], center[1], r)
    if filled:
        pygame.draw.polygon(s, color, pts)
    else:
        pygame.draw.polygon(s, color, pts, max(1, int(r // 5)))


def draw_check(s, center, size, color, width=2):
    cx, cy = center
    pygame.draw.lines(s, color, False,
                      [(cx - size, cy), (cx - size * 0.35, cy + size * 0.6),
                       (cx + size, cy - size * 0.7)], width)


class SudokuDraw:
    """Alle Zeichen-Methoden von SudokuGame."""

    # ===================================================== Caches
    def _reset_caches(self):
        self._glyph_cache = {}
        self._cage_surf = None
        self._cage_key = None
        self._diag_surf = None
        self._diag_key = None
        self._overlay = None

    def _glyph(self, fnt, text, color):
        key = (id(fnt), text, tuple(color))
        img = self._glyph_cache.get(key)
        if img is None:
            if len(self._glyph_cache) > 1500:
                self._glyph_cache.clear()
            img = fnt.render(text, True, color)
            self._glyph_cache[key] = img
        return img

    def _fit(self, text, width, fonts, color):
        """Rendert text in der ersten Schrift, in die er passt (sonst gekürzt)."""
        for fnt in fonts:
            if fnt.size(text)[0] <= width:
                return fnt.render(text, True, color)
        fnt = fonts[-1]
        while len(text) > 1 and fnt.size(text + "...")[0] > width:
            text = text[:-1]
        return fnt.render(text.rstrip() + "...", True, color)

    # ===================================================== Hauptschleife
    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height)
        if self.state == "setup":
            self._draw_setup(s)
        elif self.state == "generating":
            self._draw_generating(s)
        else:
            self._draw_hud(s)
            self._draw_board(s)
            self._draw_pad(s)
            if self.game_over and not self.reveal:
                self._draw_result(s)

    def _draw_generating(self, s):
        lbl = self.font.render(t("sud.generating"), True, ui.TEXT)
        s.blit(lbl, lbl.get_rect(center=(self.width // 2, self.height // 2)))
        self._gen_drawn = True

    # ===================================================== Setup
    def _diff_name(self, d):
        return t("sud.diff." + _sd().DIFFICULTIES[d][0])

    def _draw_setup(self, s):
        S = _sd()
        W, H = self.width, self.height
        cx = W // 2
        title = self._glyph(self._title, "SUDOKU", self.accent)
        s.blit(title, title.get_rect(center=(cx, self.su_title_y)))
        mode_lbl = t("sud.mode." + self.mode) if self.mode in S.MODE_MULT else self.mode
        sub = self._fit(mode_lbl, W - 24, (self._tiny,), ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, self.su_sub_y)))

        # Varianten mit kleinem Symbol, Stufen mit Fortschritt
        for i, r in enumerate(self.var_rects):
            v = S.VARIANTS[i]
            on = v == self.variant
            ui.draw_button(s, r, "", self._small, selected=on, accent=self.accent)
            self._draw_variant_label(s, r, v, on)
        for i, r in enumerate(self.diff_rects):
            label = self._diff_name(i)
            fnt = self._small if self._small.size(label)[0] <= r.w - 10 else self._tiny
            ui.draw_button(s, r, label, fnt, selected=i == self.diff, accent=self.accent)

        self._draw_level_grid(s)

        n, stars = self._stage_summary(self.variant, self.diff)
        txt = t("sud.progress", n=n) + "   ·   %d/300" % stars
        img = self._fit(txt, 10 * self.lv_cell - 20, (self._small, self._tiny), ui.TEXT_DIM)
        rc = img.get_rect(center=(self.lv_x + 5 * self.lv_cell - 10, self.prog_y))
        s.blit(img, rc)
        draw_star(s, (rc.right + 10, rc.centery), max(5, img.get_height() // 3), ui.GOLD)

        ui.draw_panel(s, self.panel_rect)
        self._draw_daily_card(s)
        self._draw_toggle(s, self.limit_rect, "limit", t("sud.fail_limit"), None, self.fail_limit)
        self._draw_toggle(s, self.input_rect, "input", t("sud.opt_input"),
                          t("sud.input." + self.input_mode), True)
        self._draw_toggle(s, self.colors_rect, "colors", t("sud.opt_colors"), None, self.colors_on)
        self._draw_level_info(s)
        if self.start_rect is not None:
            save = self._level_save(self.variant, self.diff, self.cursor)
            label = t("sud.resume") if save else t("sud.start", n=self.cursor)
            ui.draw_button(s, self.start_rect, label, self._small,
                           selected=self.hover == ("start", 0), accent=self.accent)

        self._draw_footer(s, t("sud.setup_hint"), self._tiny.get_height() + 10)

    def _draw_variant_label(self, s, r, v, on):
        """Beschriftung + kleines Symbol (Gitter, Diagonale, Käfig, 6x6)."""
        label = t("sud.variant." + v)
        col = ui.TEXT if on else ui.TEXT_DIM
        img = self._glyph(self._small, label, col)
        if img.get_width() > r.w - 10:
            img = self._glyph(self._tiny, label, col)
        icon = max(10, min(r.h - 12, 22))
        total = icon + 7 + img.get_width()
        if total <= r.w - 12:
            x = r.centerx - total // 2
            self._draw_variant_icon(s, pygame.Rect(x, r.centery - icon // 2, icon, icon),
                                    v, self.accent if on else ui.TEXT_FAINT)
            s.blit(img, img.get_rect(midleft=(x + icon + 7, r.centery)))
        else:
            s.blit(img, img.get_rect(center=r.center))

    def _draw_variant_icon(self, s, r, v, col):
        n = 2 if v == "mini" else 3
        pygame.draw.rect(s, col, r, 1, border_radius=2)
        for k in range(1, n):
            x = r.x + r.w * k // n
            y = r.y + r.h * k // n
            pygame.draw.line(s, col, (x, r.y), (x, r.bottom - 1))
            pygame.draw.line(s, col, (r.x, y), (r.right - 1, y))
        if v == "x":
            pygame.draw.line(s, col, r.topleft, (r.right - 1, r.bottom - 1), 2)
            pygame.draw.line(s, col, (r.right - 1, r.top), (r.left, r.bottom - 1), 2)
        elif v == "killer":
            inner = r.inflate(-r.w // 3, -r.h // 3)
            for k in range(0, inner.w, 4):
                pygame.draw.line(s, col, (inner.x + k, inner.y), (inner.x + min(k + 2, inner.w), inner.y))
                pygame.draw.line(s, col, (inner.x + k, inner.bottom), (inner.x + min(k + 2, inner.w), inner.bottom))
            for k in range(0, inner.h, 4):
                pygame.draw.line(s, col, (inner.x, inner.y + k), (inner.x, inner.y + min(k + 2, inner.h)))
                pygame.draw.line(s, col, (inner.right, inner.y + k), (inner.right, inner.y + min(k + 2, inner.h)))

    def _draw_level_grid(self, s):
        v, d = self.variant, self.diff
        cell = self.lv_cell
        solved_bg = ui.mix(ui.PANEL, ui.GREEN, 0.20)
        gold_bg = ui.mix(ui.PANEL, ui.GOLD, 0.22)
        big = cell >= 30
        for n in range(1, 101):
            i = n - 1
            x = self.lv_x + (i % 10) * cell
            y = self.lv_y + (i // 10) * cell
            rc = pygame.Rect(x + 1, y + 1, cell - 2, cell - 2)
            stars, _secs = self._record(v, d, n)
            save = self._level_save(v, d, n)
            bg = gold_bg if stars == 3 else solved_bg if stars else ui.BTN
            pygame.draw.rect(s, bg, rc, border_radius=4)
            if n == self.cursor:
                pygame.draw.rect(s, self.accent, rc, 2, border_radius=4)
            col = ui.GOLD if stars == 3 else ui.GREEN if stars else ui.TEXT_DIM
            num = self._glyph(self._lv_font, str(n), col)
            cy = rc.centery - (cell // 7 if big and stars else 0)
            s.blit(num, num.get_rect(center=(rc.centerx, cy)))
            if stars:
                if big:
                    sr = max(3, cell // 10)
                    for k in range(3):
                        sx = rc.centerx + (k - 1) * (sr * 2 + 2)
                        draw_star(s, (sx, rc.bottom - sr - 3), sr,
                                  ui.GOLD if k < stars else ui.mix(bg, ui.TEXT_FAINT, 0.5),
                                  filled=True)
                else:
                    for k in range(stars):
                        pygame.draw.circle(s, ui.GOLD, (rc.centerx + (k - 1) * 4, rc.bottom - 3), 1)
            if save:
                # angefangen: Eselsohr oben rechts + Fortschrittsbalken unten
                tri = max(5, cell // 4)
                pygame.draw.polygon(s, self.accent, [(rc.right - tri, rc.top), (rc.right, rc.top),
                                                     (rc.right, rc.top + tri)])
                p = save.get("p", 0) if isinstance(save.get("p"), int) else 0
                bw = int((rc.w - 6) * max(0, min(100, p)) / 100)
                if bw > 0 and not stars:
                    pygame.draw.rect(s, self.accent, (rc.x + 3, rc.bottom - 4, bw, 2))

    def _draw_daily_card(self, s):
        S = _sd()
        r = self.daily_rect
        hov = self.hover == ("daily", 0)
        done = self.daily_done_today()
        base = ui.mix(ui.PANEL_LIGHT, self.accent, 0.10 + (0.10 if hov else 0.0))
        pygame.draw.rect(s, base, r, border_radius=8)
        glow = self.accent if hov else ui.mix(ui.BORDER, self.accent, 0.35 + 0.35 * ui.pulse(2.0, 0.0, 1.0) * (not done))
        pygame.draw.rect(s, glow, r, 2 if hov else 1, border_radius=8)

        # Kalender-Symbol mit Tageszahl
        ih = min(r.h - 10, 34)
        cal = pygame.Rect(r.x + 7, r.centery - ih // 2, ih, ih)
        pygame.draw.rect(s, ui.mix(ui.PANEL, ui.TEXT, 0.08), cal, border_radius=4)
        pygame.draw.rect(s, self.accent, (cal.x, cal.y, cal.w, max(4, ih // 4)),
                         border_top_left_radius=4, border_top_right_radius=4)
        day = self._glyph(self._tiny, str(int(self.daily_date[8:10])), ui.TEXT)
        s.blit(day, day.get_rect(center=(cal.centerx, cal.centery + ih // 8)))

        tx = cal.right + 8
        tw = r.right - 8 - tx
        diff = _sd_daily_diff(self.daily_date)
        wd = t("sud.wd.%d" % _weekday(self.daily_date))
        line1 = self._fit(t("sud.daily"), tw - 18, (self._small, self._tiny), ui.TEXT)
        y1 = r.y + 5
        s.blit(line1, (tx, y1))
        streak = t("sud.daily_streak", n=self.daily_streak()) if self.daily_streak() else None
        save = self.saves.get("daily:" + self.daily_date)
        if done:
            head = [S.fmt_time(self.daily_state.get("time", 0))]
            draw_check(s, (r.right - 14, y1 + line1.get_height() // 2), 5, ui.GREEN, 2)
        else:
            head = [wd, self._diff_name(diff)]
            if isinstance(save, dict) and isinstance(save.get("p"), int):
                head.append("%d %%" % save["p"])
        # Zu breit? Erst den Wochentag, dann die Serie weglassen.
        options = [head + [streak], head[1:] + [streak], head[1:]] if not done             else [head + [streak], head]
        text = ""
        for parts in options:
            text = "  ·  ".join(p for p in parts if p)
            if self._tiny.size(text)[0] <= tw:
                break
        line2 = self._fit(text, tw, (self._tiny,), ui.GREEN if done else ui.TEXT_DIM)
        s.blit(line2, (tx, r.bottom - 5 - line2.get_height()))

    def _draw_toggle(self, s, r, key, label, value, on):
        hov = self.hover == (key, 0)
        pygame.draw.rect(s, ui.BTN_SEL if hov else ui.BTN, r, border_radius=6)
        pygame.draw.rect(s, self.accent if hov else ui.BORDER, r, 1, border_radius=6)
        if value is None:
            # Schalter-Grafik rechts
            sw = max(24, r.h * 3 // 2)
            sh = max(12, r.h - 10)
            pill = pygame.Rect(r.right - 8 - sw, r.centery - sh // 2, sw, sh)
            pygame.draw.rect(s, ui.GREEN if on else ui.mix(ui.BTN, ui.TEXT_FAINT, 0.5),
                             pill, border_radius=sh // 2)
            kx = pill.right - sh // 2 if on else pill.x + sh // 2
            pygame.draw.circle(s, ui.TEXT, (kx, pill.centery), sh // 2 - 2)
            room = pill.x - 8 - (r.x + 8)
            img = self._fit(label, room, (self._tiny,), ui.TEXT if on else ui.TEXT_DIM)
        else:
            val = self._glyph(self._tiny, value, self.accent)
            s.blit(val, val.get_rect(midright=(r.right - 8, r.centery)))
            room = r.w - 24 - val.get_width()
            img = self._fit(label, room, (self._tiny,), ui.TEXT)
        s.blit(img, img.get_rect(midleft=(r.x + 8, r.centery)))

    def _draw_level_info(self, s):
        S = _sd()
        r = self.info_rect
        if r.h < self._tiny.get_height():
            return
        v, d, n = self.variant, self.diff, self.cursor
        stars, secs = self._record(v, d, n)
        save = self._level_save(v, d, n)
        y = r.y
        head = self._glyph(self._small, t("sud.level", n=n), ui.TEXT)
        s.blit(head, (r.x, y))
        sr = max(5, head.get_height() // 3)
        for k in range(3):
            draw_star(s, (r.right - sr - k * (2 * sr + 3), y + head.get_height() // 2), sr,
                      ui.GOLD if 3 - k <= stars else ui.mix(ui.PANEL, ui.TEXT_FAINT, 0.6))
        y += head.get_height() + 2
        lines = []
        if secs is not None:
            lines.append((t("sud.info_best", t=S.fmt_time(secs)), ui.GREEN))
        elif stars:
            lines.append((t("sud.info_solved"), ui.GREEN))
        else:
            lines.append((t("sud.info_unsolved"), ui.TEXT_DIM))
        if isinstance(save, dict):
            p = save.get("p", 0) if isinstance(save.get("p"), int) else 0
            lines.append((t("sud.info_started", p=p), self.accent))
        lines.append((t("sud.info_target", t=S.fmt_time(S.TARGET_TIME[v][d])), ui.TEXT_DIM))
        th = self._tiny.get_height()
        for txt, col in lines:
            for line in _wrap(txt, self._tiny, r.w)[:2]:
                if y + th > r.bottom:
                    return
                img = self._fit(line, r.w, (self._tiny,), col)
                s.blit(img, (r.x, y))
                y += th
            y += 1
        # Beschreibung der Variante (umbrochen), wenn noch Platz ist
        y += 4
        for line in _wrap(t("sud.vdesc." + v), self._tiny, r.w):
            if y + th > r.bottom:
                return
            s.blit(self._glyph(self._tiny, line, ui.TEXT_FAINT), (r.x, y))
            y += th

    # ===================================================== HUD
    def _draw_hud(self, s):
        S = _sd()
        W = self.width
        pygame.draw.rect(s, ui.PANEL, (0, 0, W, self.hud_h))
        pygame.draw.line(s, ui.BORDER, (0, self.hud_h), (W, self.hud_h))
        cy = self.hud_h // 2
        th = self._tiny.get_height()

        # In der Lösungs-Ansicht ersetzt der Zurück-Hinweis die (eingefrorene)
        # Uhr - so bleibt das komplette Brett frei sichtbar.
        if self.game_over and self.reveal:
            clock = self._small.render(t("sud.hide_solution"), True, self.accent)
        else:
            clock = self._clock_font.render(S.fmt_time(self.elapsed), True, self.accent)
        crect = clock.get_rect(center=(W // 2, cy))
        s.blit(clock, crect)
        side = crect.left - 20

        if self.daily:
            top = t("sud.daily")
            bottom = t("sud.wd.%d" % _weekday(self.daily_date)) + "  ·  " + self._diff_name(self.play_diff)
        else:
            top = t("sud.variant." + self.play_variant)
            bottom = t("sud.level", n=self.level) + "  ·  " + self._diff_name(self.play_diff)
        a = self._fit(top, side, (self._small, self._tiny), self.accent)
        b = self._fit(bottom, side, (self._tiny,), ui.TEXT)
        s.blit(a, a.get_rect(bottomleft=(12, cy + 1)))
        s.blit(b, b.get_rect(topleft=(12, cy + 1)))

        if self.fail_limit:
            err_txt = t("sud.errors", n=self.errors, m=S.FAIL_LIMIT)
        else:
            err_txt = t("sud.errors_free", n=self.errors)
        e = self._fit(err_txt, side, (self._small, self._tiny),
                      ui.RED if self.errors else ui.TEXT_DIM)
        s.blit(e, e.get_rect(bottomright=(W - 12, cy + 1)))
        right = []
        if self.can_hint:
            right.append((t("sud.hints", n=S.MAX_HINTS - self.hints_used), ui.TEXT_DIM))
        if self.note_mode:
            right.append((t("sud.pad_note").upper(), self.accent))
        if self.input_mode == "digit" and self.brush:
            right.append((t("sud.brush", d=self.brush), self.accent))
        x = W - 12
        for txt, col in reversed(right):
            img = self._glyph(self._tiny, txt, col)
            if x - img.get_width() < crect.right + 20:
                break
            s.blit(img, img.get_rect(topright=(x, cy + 1)))
            x -= img.get_width() + 12


    # ===================================================== Brett
    def _cage_overlay(self):
        """Gestrichelte Käfigränder (gecacht je Zellgröße/Theme)."""
        col = ui.mix(ui.TEXT_DIM, ui.TEXT, 0.2)
        key = (self.cell, id(self.cages), tuple(col), id(self._sum_font))
        if self._cage_key == key:
            return self._cage_surf
        N, cell = self.N, self.cell
        bs = cell * N
        surf = pygame.Surface((bs + 1, bs + 1), pygame.SRCALPHA)
        c = (*col, 215)
        ins = max(3, cell // 9)
        dash = max(3, cell // 8)
        period = dash + max(2, dash * 3 // 4)
        cage_of = self.cage_of

        def same(r, cc, k):
            return 0 <= r < N and 0 <= cc < N and cage_of[r * N + cc] == k

        def hline(y, xa, xb):
            x = xa
            while x < xb:
                ph = x % period
                if ph < dash:
                    e = min(xb, x + dash - ph)
                    pygame.draw.line(surf, c, (x, y), (e, y))
                    x = e
                else:
                    x += period - ph

        def vline(x, ya, yb):
            y = ya
            while y < yb:
                ph = y % period
                if ph < dash:
                    e = min(yb, y + dash - ph)
                    pygame.draw.line(surf, c, (x, y), (x, e))
                    y = e
                else:
                    y += period - ph

        for i in range(N * N):
            r, cc = divmod(i, N)
            k = cage_of[i]
            x0, y0 = cc * cell, r * cell
            x1, y1 = x0 + cell - 1, y0 + cell - 1
            left, right = same(r, cc - 1, k), same(r, cc + 1, k)
            up, down = same(r - 1, cc, k), same(r + 1, cc, k)
            if not up:
                hline(y0 + ins, x0 if left else x0 + ins, x1 + 1 if right else x1 - ins)
            if not down:
                hline(y1 - ins, x0 if left else x0 + ins, x1 + 1 if right else x1 - ins)
            if not left:
                vline(x0 + ins, y0 if up else y0 + ins, y1 + 1 if down else y1 - ins)
            if not right:
                vline(x1 - ins, y0 if up else y0 + ins, y1 + 1 if down else y1 - ins)
            # Innenecken (L-Form): kurze Stücke bis zur Zellkante
            for dr, dc in ((-1, -1), (-1, 1), (1, -1), (1, 1)):
                if same(r + dr, cc, k) and same(r, cc + dc, k) and not same(r + dr, cc + dc, k):
                    px = x0 + ins if dc < 0 else x1 - ins
                    py = y0 + ins if dr < 0 else y1 - ins
                    if dr < 0:
                        vline(px, y0, py)
                    else:
                        vline(px, py, y1 + 1)
                    if dc < 0:
                        hline(py, x0, px)
                    else:
                        hline(py, px, x1 + 1)
        # Platz für die Summen: Linien unter dem Etikett wegradieren
        self._sum_pos = []
        for ci, (total, cells) in enumerate(self.cages):
            first = min(cells)
            r, cc = divmod(first, N)
            w, h = self._sum_font.size(str(total))
            x, y = cc * cell + 2, r * cell + 1
            surf.fill((0, 0, 0, 0), (x - 1, y, w + 3, h))
            self._sum_pos.append((x, y))
        self._cage_surf = surf
        self._cage_key = key
        return surf

    def _diag_overlay(self):
        col = ui.mix(ui.PANEL_LIGHT, self.accent, 0.55)
        key = (self.cell, tuple(col))
        if self._diag_key == key:
            return self._diag_surf
        bs = self.cell * self.N
        surf = pygame.Surface((bs + 1, bs + 1), pygame.SRCALPHA)
        c = (*col, 90)
        pygame.draw.line(surf, c, (0, 0), (bs, bs), max(1, self.cell // 14))
        pygame.draw.line(surf, c, (bs, 0), (0, bs), max(1, self.cell // 14))
        self._diag_surf = surf
        self._diag_key = key
        return surf

    def _draw_board(self, s):
        S = _sd()
        N, cell, bx, by = self.N, self.cell, self.bx, self.by
        lay = self.lay
        now = pygame.time.get_ticks()
        sel = self.sel
        sel_val = self.board[sel]
        sel_peers = set(self.peers[sel])
        sel_cage = self.cage_of[sel]
        same_val = sel_val
        if self.input_mode == "digit" and self.brush:
            same_val = self.brush

        # Zellfarben je Frame aus Palette + Akzent mischen (Theme-fähig).
        c_cell = ui.PANEL
        c_peer = ui.PANEL_LIGHT
        c_sel = ui.mix(ui.PANEL_LIGHT, self.accent, 0.35)
        c_same = ui.mix(ui.PANEL_LIGHT, self.accent, 0.18)
        c_bad = ui.mix(ui.PANEL, ui.RED, 0.35)
        c_diag = ui.mix(ui.PANEL, self.accent, 0.08)
        c_cage = ui.mix(ui.PANEL, self.accent, 0.13)
        glow = ui.mix(self.accent, (255, 255, 255), 0.45)
        show_solution = self.game_over and self.reveal
        bad_cells = set(self._conflicts)
        if self.can_check:
            for ci in self._bad_cages:
                bad_cells.update(self.cages[ci][1])

        # Leucht-Wellen fertiger Einheiten: Zelle -> Stärke 0..1
        flash = {}
        alive = []
        for cells, origin, t0 in self.fx_units:
            age_all = now - t0
            if age_all > 900:
                continue
            alive.append((cells, origin, t0))
            orr, occ = divmod(origin, N)
            for i in cells:
                rr, c2 = divmod(i, N)
                age = age_all - 45 * (abs(rr - orr) + abs(c2 - occ))
                if 0 <= age < 420:
                    a = math.sin(math.pi * age / 420) * 0.6
                    if a > flash.get(i, 0):
                        flash[i] = a
        self.fx_units = alive
        win_age = now - self._win_ticks if (self.game_over and self.won) else -1

        for i in range(N * N):
            r, c = divmod(i, N)
            x = bx + c * cell
            y = by + r * cell
            # Zellhintergrund: Auswahl > Konflikt > gleiche Ziffer > Käfig
            # > Nachbarn > Diagonale
            if i == sel:
                bg = c_sel
            elif self.can_check and i in bad_cells:
                bg = c_bad
            elif self.can_check and same_val and self.board[i] == same_val:
                bg = c_same
            elif sel_cage >= 0 and self.cage_of[i] == sel_cage:
                bg = c_cage
            elif i in sel_peers:
                bg = c_peer
            elif lay.diagonals and lay.on_diagonal(i):
                bg = c_diag
            else:
                bg = c_cell
            if self.marks[i]:
                bg = ui.mix(bg, MARK_COLORS[self.marks[i] - 1], 0.42)
            a = flash.get(i)
            if a:
                bg = ui.mix(bg, glow, a)
            if win_age >= 0:
                dist = abs(r - (N - 1) / 2) + abs(c - (N - 1) / 2)
                age = win_age - 60 * dist
                if 0 <= age < 520:
                    bg = ui.mix(bg, ui.GREEN, math.sin(math.pi * age / 520) * 0.55)
            pygame.draw.rect(s, bg, (x, y, cell, cell))

        if lay.diagonals:
            s.blit(self._diag_overlay(), (bx, by))
        if self.cages:
            s.blit(self._cage_overlay(), (bx, by))
            for ci, (total, _cells) in enumerate(self.cages):
                if self.can_check and ci in self._bad_cages:
                    col = ui.RED
                elif ci == sel_cage:
                    col = self.accent
                else:
                    col = ui.TEXT_DIM
                img = self._glyph(self._sum_font, str(total), col)
                sx, sy = self._sum_pos[ci]
                s.blit(img, (bx + sx, by + sy))

        # Ziffern und Notizen (Killer: Notizen innerhalb der Käfiglinien und
        # unter der Summe)
        ins = max(3, cell // 9) + 2 if self.cages else 0
        note_top = max(ins, self._sum_font.get_height() - 1) if self.cages else 0
        rows = (N + 2) // 3
        cw = (cell - 2 * ins) // 3
        ch = (cell - note_top - ins) // rows
        for i in range(N * N):
            r, c = divmod(i, N)
            x = bx + c * cell
            y = by + r * cell
            if show_solution and self.board[i] != self.solution[i]:
                img = self._glyph(self._num_font, str(self.solution[i]), self.accent)
                s.blit(img, img.get_rect(center=(x + cell // 2, y + cell // 2)))
                continue
            v = self.board[i]
            if v:
                if self.given[i]:
                    col = ui.TEXT
                elif self.can_check and i in self.wrong:
                    col = ui.RED
                elif self.locked[i]:
                    col = ui.GREEN
                else:
                    col = COL_USER
                img = self._glyph(self._num_font, str(v), col)
                cxm, cym = x + cell // 2, y + cell // 2 + (note_top // 3 if self.cages else 0)
                t0 = self.fx_pop.get(i)
                if t0 is not None:
                    age = now - t0
                    if age < 240:
                        k = 1.0 + 0.4 * math.sin(math.pi * age / 240)
                        img = pygame.transform.smoothscale(
                            img, (max(1, int(img.get_width() * k)), max(1, int(img.get_height() * k))))
                    else:
                        del self.fx_pop[i]
                t1 = self.fx_shake.get(i)
                if t1 is not None:
                    age = now - t1
                    if age < 380:
                        cxm += int(math.sin(age / 22.0) * cell * 0.09 * (1 - age / 380))
                    else:
                        del self.fx_shake[i]
                s.blit(img, img.get_rect(center=(cxm, cym)))
            elif self.notes[i]:
                nm = self.notes[i]
                for d in range(1, N + 1):
                    if nm >> (d - 1) & 1:
                        col = self.accent if (self.can_check and d == same_val) else ui.TEXT_FAINT
                        img = self._glyph(self._note_font, str(d), col)
                        nx = x + ins + ((d - 1) % 3) * cw + cw // 2
                        ny = y + note_top + ((d - 1) // 3) * ch + ch // 2
                        s.blit(img, img.get_rect(center=(nx, ny)))

        # Gitterlinien (dünn + Blockgrenzen fett)
        bs = cell * N
        for k in range(N + 1):
            x = bx + k * cell
            thick = k % lay.box_w == 0
            pygame.draw.line(s, ui.BORDER_LIGHT if thick else ui.BORDER,
                             (x, by), (x, by + bs), 2 if thick else 1)
        for k in range(N + 1):
            y = by + k * cell
            thick = k % lay.box_h == 0
            pygame.draw.line(s, ui.BORDER_LIGHT if thick else ui.BORDER,
                             (bx, y), (bx + bs, y), 2 if thick else 1)
        if not self.game_over:
            r, c = divmod(sel, N)
            pygame.draw.rect(s, self.accent, (bx + c * cell, by + r * cell, cell + 1, cell + 1), 2)

    # ===================================================== Ziffernfeld
    def _draw_pad(self, s):
        S = _sd()
        N = self.N
        counts = [0] * (N + 1)
        for v in self.board:
            counts[v] += 1
        th = self._tiny.get_height()
        for d in range(1, N + 1):
            r = self.pad_rects[str(d)]
            left = N - counts[d]
            done = left <= 0
            on = self.input_mode == "digit" and self.brush == d
            hov = self.hover == str(d)
            bg = ui.BTN_SEL if on else (ui.mix(ui.BTN, ui.PANEL_LIGHT, 0.6) if hov else ui.BTN)
            pygame.draw.rect(s, bg, r, border_radius=6)
            pygame.draw.rect(s, self.accent if on else ui.BORDER, r, 2 if on else 1, border_radius=6)
            col = ui.TEXT_FAINT if done else (self.accent if on else ui.TEXT)
            img = self._glyph(self._pad_font, str(d), col)
            s.blit(img, img.get_rect(center=(r.centerx, r.centery - th // 3)))
            if done:
                draw_check(s, (r.centerx, r.bottom - th // 2 - 2), max(3, th // 3), ui.GREEN, 2)
            else:
                cnt = self._glyph(self._tiny, str(left), ui.TEXT_DIM)
                s.blit(cnt, cnt.get_rect(center=(r.centerx, r.bottom - th // 2 - 1)))

        for key in ("erase", "undo", "redo", "note", "cands", "hint"):
            r = self.pad_rects.get(key)
            if r is None:
                continue
            hov = self.hover == key
            active = key == "note" and self.note_mode
            enabled = True
            if key == "undo":
                enabled = bool(self.undo_stack)
            elif key == "redo":
                enabled = bool(self.redo_stack)
            elif key == "hint":
                enabled = self.hints_used < S.MAX_HINTS
            bg = ui.BTN_SEL if active else (ui.mix(ui.BTN, ui.PANEL_LIGHT, 0.6) if hov else ui.BTN)
            pygame.draw.rect(s, bg, r, border_radius=6)
            pygame.draw.rect(s, self.accent if (active or hov) else ui.BORDER, r,
                             2 if active else 1, border_radius=6)
            col = self.accent if active else (ui.TEXT if enabled else ui.TEXT_FAINT)
            self._draw_pad_icon(s, key, r, col)
            letter = PAD_KEYS[key]
            if key == "hint":
                letter = "%s %d" % (letter, S.MAX_HINTS - self.hints_used)
            if r.h >= 26:
                img = self._glyph(self._tiny, letter, ui.TEXT_FAINT)
                s.blit(img, img.get_rect(bottomright=(r.right - 3, r.bottom)))

        for k in range(S.MARK_COUNT):
            r = self.pad_rects.get("mark%d" % (k + 1))
            if r is None:
                continue
            pygame.draw.rect(s, MARK_COLORS[k], r, border_radius=4)
            if self.marks[self.sel] == k + 1:
                pygame.draw.rect(s, ui.TEXT, r.inflate(2, 2), 2, border_radius=5)

        # Tooltip-Zeile unter dem Feld
        tip = None
        if self.hover in PAD_TIPS:
            tip = t(PAD_TIPS[self.hover])
        elif self.hover and self.hover.startswith("mark"):
            tip = t("sud.tip.mark")
        elif self.hover and self.hover.isdigit():
            tip = t("sud.tip.digit_first") if self.input_mode == "digit" else t("sud.tip.digit")
        elif self.input_mode == "digit" and not self.brush:
            tip = t("sud.tip.digit_first")
        if tip:
            y = self.caption_y
            for line in _wrap(tip, self._tiny, self.pad_w)[:2]:
                if y + th > self.caption_bottom:
                    break
                s.blit(self._glyph(self._tiny, line, ui.TEXT_DIM), (self.pad_x, y))
                y += th

        hint_key = "sud.hint_digit" if self.input_mode == "digit" else "sud.hint"
        self._draw_footer(s, t(hint_key), self._tiny.get_height() + 12)

    def _draw_footer(self, s, text, foot):
        """Fußzeile: Steuerungshinweis - oder, solange aktiv, eine Meldung
        (als kleine Pille in Meldungsfarbe)."""
        cx, cy = self.width // 2, self.height - foot // 2
        if self.msg:
            col = self.msg_col or ui.RED
            img = self._fit(self.msg, self.width - 40, (self._small, self._tiny), col)
            r = img.get_rect(center=(cx, cy))
            bg = r.inflate(20, 4)
            bg.bottom = min(bg.bottom, self.height - 1)
            r.center = bg.center
            pygame.draw.rect(s, ui.PANEL, bg, border_radius=bg.h // 2)
            pygame.draw.rect(s, col, bg, 1, border_radius=bg.h // 2)
            s.blit(img, r)
            return
        hint = self._fit(text, self.width - 12, (self._tiny,), ui.TEXT_DIM)
        s.blit(hint, hint.get_rect(center=(cx, cy)))

    def _draw_pad_icon(self, s, key, r, col):
        cx, cy = r.centerx, r.centery - (2 if r.h >= 26 else 0)
        z = max(5, min(r.w, r.h) * 3 // 10)
        w = max(2, z // 4)
        if key == "erase":
            # Rücktaste: Fünfeck mit Kreuz
            pts = [(cx - z, cy), (cx - z // 2, cy - z * 2 // 3), (cx + z, cy - z * 2 // 3),
                   (cx + z, cy + z * 2 // 3), (cx - z // 2, cy + z * 2 // 3)]
            pygame.draw.polygon(s, col, pts, max(1, w // 2 + 1))
            k = z // 3
            pygame.draw.line(s, col, (cx - k + 2, cy - k), (cx + k + 2, cy + k), max(1, w // 2 + 1))
            pygame.draw.line(s, col, (cx - k + 2, cy + k), (cx + k + 2, cy - k), max(1, w // 2 + 1))
        elif key in ("undo", "redo"):
            rr = pygame.Rect(cx - z, cy - z * 2 // 3, 2 * z, 2 * z)
            pygame.draw.arc(s, col, rr, 0.0, math.pi, max(2, w))
            if key == "undo":
                tip = (rr.left + max(1, w // 2), rr.centery + 2)
                pygame.draw.polygon(s, col, [(tip[0] - z // 2 - 1, tip[1] - 2),
                                             (tip[0] + z // 2 + 1, tip[1] - 2),
                                             (tip[0], tip[1] + z // 2 + 1)])
            else:
                tip = (rr.right - max(1, w // 2), rr.centery + 2)
                pygame.draw.polygon(s, col, [(tip[0] - z // 2 - 1, tip[1] - 2),
                                             (tip[0] + z // 2 + 1, tip[1] - 2),
                                             (tip[0], tip[1] + z // 2 + 1)])
        elif key == "note":
            # Bleistift
            a = (cx - z, cy + z)
            b = (cx + z * 2 // 3, cy - z * 2 // 3)
            pygame.draw.line(s, col, (a[0] + z // 3, a[1] - z // 3), b, max(3, w + 1))
            pygame.draw.polygon(s, col, [a, (a[0] + z // 2, a[1] - z // 6), (a[0] + z // 6, a[1] - z // 2)])
        elif key == "cands":
            # 3x3-Punktraster (Kandidaten)
            step = max(3, z * 2 // 3)
            for rr in range(3):
                for cc in range(3):
                    pygame.draw.circle(s, col, (cx + (cc - 1) * step, cy + (rr - 1) * step),
                                       max(1, z // 6) + (1 if (rr + cc) % 2 == 0 else 0))
        elif key == "hint":
            # Glühbirne
            rad = max(3, z * 2 // 3)
            pygame.draw.circle(s, col, (cx, cy - z // 4), rad, max(1, w // 2 + 1))
            pygame.draw.rect(s, col, (cx - rad // 2, cy - z // 4 + rad, rad, max(2, z // 3)))

    # ===================================================== Ergebnis
    def _draw_result(self, s):
        S = _sd()
        W, H = self.width, self.height
        # Abdunkelung wird gecacht (kein Alpha-Vollbild-Fill pro Frame).
        if self._overlay is None or self._overlay.get_size() != (W, H):
            ov = pygame.Surface((W, H), pygame.SRCALPHA)
            ov.fill((8, 10, 16, 175))
            self._overlay = ov
        s.blit(self._overlay, (0, 0))
        cx, cy = W // 2, H // 2
        now = pygame.time.get_ticks()
        age = now - self._win_ticks
        sh = self._small.get_height()

        if self.won:
            head = self._fit(t("sud.win", t=S.fmt_time(self.result_time)), W - 40,
                             (self._huge, self._title, self._small), ui.GREEN)
            lines = [(t("common.points", score=self.score), ui.TEXT)]
            if self.daily:
                lines.append((t("sud.daily_solved", n=self.daily_streak()), ui.GOLD))
            if self.new_record:
                lines.append((t("sud.new_best", t=S.fmt_time(self.best_time)), ui.GOLD))
            else:
                lines.append((t("sud.info_best", t=S.fmt_time(self.best_time)), ui.TEXT_DIM))
            lines.append((t("sud.next_daily") if self.daily else t("sud.next"), ui.TEXT_DIM))
            lines.append((t("sud.show_solution"), ui.TEXT_DIM))
            star_h = max(26, H // 14)
        else:
            head = self._fit(t("sud.lose"), W - 40, (self._huge, self._title, self._small), ui.RED)
            lines = [(t("sud.retry"), ui.TEXT_DIM), (t("sud.show_solution"), ui.TEXT_DIM)]
            star_h = 0

        imgs = [self._fit(txt, W - 60, (self._small, self._tiny), col) for txt, col in lines]
        pw = max(min(W - 40, 460), head.get_width() + 40,
                 max(i.get_width() for i in imgs) + 40)
        pw = min(pw, W - 16)
        ph = 20 + head.get_height() + (star_h + 10 if star_h else 4) \
            + len(imgs) * (sh + 6) + 14
        # sanftes Hereinfahren
        k = min(1.0, max(0.0, age / 260.0))
        oy = int((1 - k) * 24)
        panel = pygame.Rect(cx - pw // 2, cy - ph // 2 + oy, pw, ph)
        pygame.draw.rect(s, ui.PANEL, panel, border_radius=14)
        pygame.draw.rect(s, self.accent, panel, 2, border_radius=14)

        y = panel.y + 14
        s.blit(head, head.get_rect(midtop=(cx, y)))
        y += head.get_height() + 6
        if star_h:
            r = star_h // 2
            for n in range(3):
                sx = cx + (n - 1) * (star_h + 12)
                sy = y + r
                earned = n < self.result_stars
                t0 = 350 + n * 280
                if earned and age >= t0:
                    p = min(1.0, (age - t0) / 220.0)
                    scale = 1.0 + 0.5 * math.sin(math.pi * p) if p < 1.0 else 1.0
                    draw_star(s, (sx, sy), r * scale, ui.GOLD)
                    if n not in self._star_snd:
                        self._star_snd.add(n)
                        self.play_sound("point")
                else:
                    draw_star(s, (sx, sy), r, ui.mix(ui.PANEL, ui.TEXT_FAINT, 0.7))
                    draw_star(s, (sx, sy), r, ui.mix(ui.PANEL, ui.TEXT_FAINT, 0.9), filled=False)
            y += star_h + 10
        for img in imgs:
            s.blit(img, img.get_rect(midtop=(cx, y)))
            y += sh + 6


def _wrap(text, fnt, width):
    """Bricht text an Leerzeichen in Zeilen <= width um."""
    out, line = [], ""
    for word in text.split():
        test = (line + " " + word).strip()
        if fnt.size(test)[0] <= width or not line:
            line = test
        else:
            out.append(line)
            line = word
    if line:
        out.append(line)
    return out


def _weekday(date):
    import datetime
    return datetime.date.fromisoformat(date).weekday()


def _sd_daily_diff(date):
    from . import sudoku_gen
    return sudoku_gen.daily_diff(date)
