# -*- coding: utf-8 -*-
"""
matchview.py
============
Der Matches-Screen eines Spiels (geöffnet über den Knopf "Matches" im
Vorspiel-Screen, siehe menu.PreGameScreen):

- Reiter "Letzte Matches": Verlaufskurve der letzten 30 Ergebnisse (Rekorde
  markiert, Tooltip beim Überfahren) und darunter die letzten 10 Matches.
- Reiter "Alle Matches"  : alle gespeicherten Matches (matches.CAP je Spiel),
  Filter nach Modus (Taste M), Sortierung neueste/beste (Taste O),
  Bildlauf mit Pfeilen/Mausrad/Bild auf/ab.

Über beiden Reitern stehen die Summen (Matches, Siege, Bestwert, Schnitt,
Spielzeit) - sie laufen unbegrenzt weiter, auch wenn alte Einträge aus der
Liste fallen. Daten: matches.py.
"""

import pygame

import charts
import matches
import seedrand
import ui
from game_base import InputEvent
from i18n import t
from menu import _Screen

ROW_H = 48
TABS = ("recent", "all")
CHART_N = 30
RECENT_N = 10


def mode_label(game_cls, mode):
    """Anzeigename eines Modus (aus MODES des Spiels, sonst Einzel/Mehr)."""
    for key, label_key in getattr(game_cls, "MODES", None) or ():
        if key == mode:
            return t(label_key)
    if mode == "single":
        return t("pregame.single")
    if mode == "multi":
        return t("pregame.multi")
    return mode or "-"


def fmt_num(value):
    """Ganzzahl mit Tausenderpunkten (sprachneutral, wie im HUD)."""
    value = int(value)
    sign = "-" if value < 0 else ""
    txt = "{:,}".format(abs(value)).replace(",", ".")
    return sign + txt


def fmt_score(entry):
    """Punkte eines Eintrags - Chip-Spiele mit Vorzeichen (Gewinn/Verlust)."""
    score = entry[2]
    info = entry[6] or {}
    if "bet" in info:
        return ("+" if score > 0 else "") + fmt_num(score)
    return fmt_num(score)


def fit_text(fnt, text, room):
    """Kürzt einen Text mit '…', bis er in 'room' Pixel passt."""
    if fnt.size(text)[0] <= room:
        return text
    while text and fnt.size(text + "…")[0] > room:
        text = text[:-1]
    return text.rstrip() + "…" if text else ""


def info_parts(game_cls, info):
    """Detail-Texte eines Eintrags (Reihenfolge wie matches.INFO_KEYS)."""
    out = []
    if not info:
        return out
    custom = getattr(game_cls, "match_text", None)
    for key, label_key in matches.INFO_KEYS:
        if key not in info:
            continue
        value = info[key]
        text = custom(key, value) if custom else None
        if text is None:
            if key == "day":
                text = t(label_key, v=seedrand.date_of(value))
            elif key in ("x", "cr"):
                text = t(label_key, v="%d.%02d" % divmod(max(0, int(value)), 100))
            elif key == "tm":
                text = t(label_key, v=matches.format_duration(value))
            elif key in ("bet", "win", "c"):
                text = t(label_key, v=fmt_num(value))
            else:
                text = t(label_key, v=value)
        if text:
            out.append(text)
    return out


class MatchScreen(_Screen):
    """Letzte Matches / Alle Matches eines Spiels."""

    name = "Matches"

    def __init__(self, surface, width, height, app, game_cls, on_close=None,
                 tab="recent"):
        self.game_cls = game_cls
        self.key = game_cls.highscore_key
        self.accent = ui.game_color(game_cls.__name__)
        self.on_close = on_close
        self.tab = tab if tab in TABS else "recent"
        self.filter = None              # None = alle Modi, sonst Modus-Schlüssel
        self.sort_best = False
        self.first = 0
        self.sel = 0
        self._hover = None              # (art, wert) unter der Maus
        self._chart_hits = []
        self._chart_chron = []
        self._mouse = None
        super().__init__(surface, width, height, app)
        # Game.__init__ setzt die Akzentfarbe nach dem Klassennamen - hier gilt
        # die Farbe des Spiels, dessen Matches gezeigt werden.
        self.accent = ui.game_color(game_cls.__name__)
        self.name = t("match.name")
        self._load()
        self._build()

    # ----- Daten & Layout -----------------------------------------------

    def _load(self):
        self.all = matches.entries(self.key)
        self.tot = matches.totals(self.key)
        self.modes = matches.modes_of(self.key)
        if self.filter not in self.modes:
            self.filter = None
        self._apply_view()

    def _apply_view(self):
        items = self.all
        if self.filter is not None:
            items = [e for e in items if e[1] == self.filter]
        if self.sort_best:
            # Stabil: bei gleichen Punkten bleibt das neuere vorn.
            items = sorted(items, key=lambda e: -e[2])
        self.items = items
        self.sel = min(self.sel, max(0, len(self.items) - 1))
        self.first = min(self.first, max(0, len(self.items) - 1))

    def on_surface_changed(self):
        self._build()

    def _build(self):
        W, H = self.width, self.height
        self._left = max(24, W // 2 - 340)
        self._right = W - self._left
        self._bottom = H - 40
        # Reiter mittig unter dem Titel
        self._tab_font = ui.font(14, bold=True)
        labels = [t("match.tab_" + k) for k in TABS]
        widths = [self._tab_font.size(lb)[0] + 30 for lb in labels]
        total = sum(widths) + 8 * (len(widths) - 1)
        x = W // 2 - total // 2
        self.tab_rects = []
        for k, lb, w in zip(TABS, labels, widths):
            self.tab_rects.append((pygame.Rect(x, 92, w, 26), k, lb))
            x += w + 8
        # Summen-Leiste
        self.sum_rect = pygame.Rect(self._left, 126, self._right - self._left, 44)
        top = self.sum_rect.bottom + 10
        if self.tab == "recent":
            ch = max(64, min(130, (self._bottom - top) // 3))
            self.chart_rect = pygame.Rect(self._left, top, self._right - self._left, ch)
            self._top = self.chart_rect.bottom + 10
            self.filter_rects = []
            self.sort_rect = None
        else:
            self.chart_rect = None
            # Filter-Chips (Alle + jeder Modus) links, Sortierung rechts
            fnt = ui.font(12, bold=True)
            self._chip_font = fnt
            sort_lb = t("match.sort_best") if self.sort_best else t("match.sort_new")
            sw = fnt.size(sort_lb)[0] + 24
            self.sort_rect = pygame.Rect(self._right - sw, top, sw, 24)
            self.filter_rects = []
            x = self._left
            limit = self.sort_rect.x - 8
            for mode in [None] + list(self.modes):
                lb = t("match.filter_all") if mode is None else mode_label(self.game_cls, mode)
                w = fnt.size(lb)[0] + 24
                if x + w > limit:
                    break
                self.filter_rects.append((pygame.Rect(x, top, w, 24), mode, lb))
                x += w + 6
            self._head_y = top + 24 + 14
            self._top = top + 24 + 28
        self.rows_visible = max(1, (self._bottom - self._top) // ROW_H)
        self.row_rects = [pygame.Rect(self._left, self._top + i * ROW_H,
                                      self._right - self._left, ROW_H - 6)
                          for i in range(self.rows_visible)]

    def _rows(self):
        """Einträge, die im aktiven Reiter als Zeilen erscheinen."""
        if self.tab == "recent":
            return self.all[:RECENT_N]
        return self.items

    # ----- Eingabe ------------------------------------------------------

    def _close(self):
        if self.on_close:
            self.on_close()
        else:
            self.app.back_to_menu()

    def _switch_tab(self, tab):
        if tab == self.tab:
            return
        self.tab = tab
        self.first = 0
        self.sel = 0
        self._build()
        self.play_sound("move")

    def _cycle_filter(self):
        options = [None] + list(self.modes)
        i = options.index(self.filter) if self.filter in options else 0
        self.filter = options[(i + 1) % len(options)]
        self.first = self.sel = 0
        self._apply_view()
        self.play_sound("move")

    def _toggle_sort(self):
        self.sort_best = not self.sort_best
        self.first = self.sel = 0
        self._apply_view()
        self._build()
        self.play_sound("move")

    def _move(self, d):
        rows = self._rows()
        if not rows:
            return
        self.sel = max(0, min(len(rows) - 1, self.sel + d))
        if self.sel < self.first:
            self.first = self.sel
        elif self.sel >= self.first + self.rows_visible:
            self.first = self.sel - self.rows_visible + 1
        self.play_sound("move")

    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k == "Escape":
                self._close()
            elif k in ("Tab", "ISO_Left_Tab", "Left", "Right") \
                    or self.is_action(k, "left") or self.is_action(k, "right"):
                self._switch_tab("all" if self.tab == "recent" else "recent")
            elif k == "Up" or self.is_action(k, "up"):
                self._move(-1)
            elif k == "Down" or self.is_action(k, "down"):
                self._move(1)
            elif k == "Prior":
                self._move(-self.rows_visible)
            elif k == "Next":
                self._move(self.rows_visible)
            elif k == "Home":
                self._move(-len(self._rows()))
            elif k == "End":
                self._move(len(self._rows()))
            elif k in ("m", "M") and self.tab == "all":
                self._cycle_filter()
            elif k in ("o", "O") and self.tab == "all":
                self._toggle_sort()
        elif event.kind == InputEvent.MOUSEMOVE:
            self._mouse = event.pos
            self._hover = None
            for r, k, _ in self.tab_rects:
                if r.collidepoint(event.pos):
                    self._hover = ("tab", k)
            for r, mode, _ in self.filter_rects:
                if r.collidepoint(event.pos):
                    self._hover = ("filter", mode)
            if self.sort_rect is not None and self.sort_rect.collidepoint(event.pos):
                self._hover = ("sort", None)
            rows = self._rows()
            for i, r in enumerate(self.row_rects):
                if r.collidepoint(event.pos) and self.first + i < len(rows):
                    self.sel = self.first + i
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            for r, k, _ in self.tab_rects:
                if r.collidepoint(event.pos):
                    self._switch_tab(k)
                    return
            for r, mode, _ in self.filter_rects:
                if r.collidepoint(event.pos):
                    if mode != self.filter:
                        self.filter = mode
                        self.first = self.sel = 0
                        self._apply_view()
                        self.play_sound("move")
                    return
            if self.sort_rect is not None and self.sort_rect.collidepoint(event.pos):
                self._toggle_sort()
        elif event.kind == InputEvent.WHEEL:
            self._move(-1 if event.delta > 0 else 1)

    # ----- Zeichnen -----------------------------------------------------

    def draw(self):
        s = self.surface
        W, H = self.width, self.height
        ui.draw_background(s, W, H, stars=False)
        ui.draw_title(s, W, str(self.game_cls.name), subtitle=t("match.subtitle"),
                      y=26, big=ui.font(30, bold=True), accent=self.accent)
        for r, k, lb in self.tab_rects:
            active = k == self.tab
            hover = self._hover == ("tab", k)
            pygame.draw.rect(s, ui.PANEL_LIGHT if (active or hover) else ui.PANEL, r,
                             border_radius=13)
            pygame.draw.rect(s, self.accent if active else ui.BORDER, r, 1,
                             border_radius=13)
            img = self._tab_font.render(lb, True, ui.TEXT if active else ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=r.center))
        self._draw_sums(s)
        if self.tab == "recent":
            self._draw_chart(s)
        else:
            self._draw_filters(s)
        rows = self._rows()
        if not rows:
            self._draw_empty(s)
        else:
            for i, r in enumerate(self.row_rects):
                idx = self.first + i
                if idx >= len(rows):
                    break
                self._draw_row(s, r, rows[idx], idx == self.sel)
            if len(rows) > self.rows_visible:
                self._draw_scrollbar(s, len(rows))
        if self.tab == "recent" and self._chart_hits and self._mouse:
            self._draw_chart_tip(s)
        ui.draw_footer(s, W, H, t("match.hint_" + self.tab))

    def _draw_sums(self, s):
        tot = self.tot
        cells = [(t("match.sum_n"), fmt_num(tot["n"]))]
        decided = tot["w"] + tot["l"] + tot["d"]
        if decided:
            pct = int(round(100.0 * tot["w"] / decided))
            cells.append((t("match.sum_w"), "%s (%d %%)" % (fmt_num(tot["w"]), pct)))
        if tot["n"]:
            cells.append((t("match.sum_best"), fmt_num(tot["best"] or 0)))
            cells.append((t("match.sum_avg"), fmt_num(int(round(tot["sum"] / float(tot["n"]))))))
            cells.append((t("match.sum_time"), matches.format_duration(tot["time"])))
        r = self.sum_rect
        gap = 8
        cw = (r.w - gap * (len(cells) - 1)) // len(cells)
        lab_f = ui.font(11)
        val_f = ui.font(16, bold=True)
        for i, (lab, val) in enumerate(cells):
            cell = pygame.Rect(r.x + i * (cw + gap), r.y, cw, r.h)
            ui.draw_panel(s, cell, color=ui.PANEL, radius=10, shadow=False)
            li = lab_f.render(lab, True, ui.TEXT_FAINT)
            s.blit(li, li.get_rect(midtop=(cell.centerx, cell.y + 5)))
            vi = val_f.render(val, True, self.accent if i == 0 else ui.TEXT)
            s.blit(vi, vi.get_rect(midbottom=(cell.centerx, cell.bottom - 5)))

    def _draw_chart(self, s):
        r = self.chart_rect
        ui.draw_panel(s, r, color=ui.PANEL, radius=10, shadow=False)
        chron = list(reversed(self.all[:CHART_N]))
        lab = ui.font(11, bold=True).render(t("match.chart", n=len(chron)), True,
                                            ui.TEXT_FAINT)
        s.blit(lab, (r.x + 10, r.y + 6))
        # Index unter der Maus aus den Treffern des letzten Bilds - vor dem Leeren
        hover = self._chart_index()
        self._chart_hits = []
        if len(chron) < 2:
            hint = ui.font(12).render(t("match.chart_few"), True, ui.TEXT_FAINT)
            s.blit(hint, hint.get_rect(center=r.center))
            return
        inner = pygame.Rect(r.x + 10, r.y + 34, r.w - 20, r.h - 42)
        marks = [i for i, e in enumerate(chron) if e[5] & matches.F_RECORD]
        self._chart_chron = chron
        self._chart_hits = charts.line(s, inner, [e[2] for e in chron],
                                       color=self.accent, marks=marks,
                                       fmt=lambda v: fmt_num(int(round(v))),
                                       hover=hover)

    def _chart_index(self):
        """Index des Kurvenpunkts unter der Maus (aus den Treffern des letzten Bilds)."""
        if not self._mouse:
            return None
        for rect, idx in self._chart_hits:
            if rect.collidepoint(self._mouse):
                return idx
        return None

    def _draw_chart_tip(self, s):
        idx = self._chart_index()
        if idx is None or idx >= len(self._chart_chron):
            return
        e = self._chart_chron[idx]
        lines = [fmt_score(e), e[0], mode_label(self.game_cls, e[1])]
        charts.tooltip(s, self._mouse, lines)

    def _draw_filters(self, s):
        for r, mode, lb in self.filter_rects:
            active = mode == self.filter
            hover = self._hover == ("filter", mode)
            pygame.draw.rect(s, ui.PANEL_LIGHT if (active or hover) else ui.PANEL, r,
                             border_radius=12)
            pygame.draw.rect(s, self.accent if active else ui.BORDER, r, 1,
                             border_radius=12)
            img = self._chip_font.render(lb, True, ui.TEXT if active else ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=r.center))
        r = self.sort_rect
        hover = self._hover == ("sort", None)
        pygame.draw.rect(s, ui.PANEL_LIGHT if hover else ui.PANEL, r, border_radius=12)
        pygame.draw.rect(s, ui.ACCENT2 if hover else ui.BORDER, r, 1, border_radius=12)
        lb = t("match.sort_best") if self.sort_best else t("match.sort_new")
        img = self._chip_font.render(lb, True, ui.TEXT)
        s.blit(img, img.get_rect(center=r.center))
        cnt = ui.font(12, bold=True).render(
            t("match.stored", n=fmt_num(self.tot["stored"]), total=fmt_num(self.tot["n"]),
              max=matches.CAP), True, ui.TEXT_DIM)
        s.blit(cnt, cnt.get_rect(midleft=(self._left, self._head_y)))

    def _badge(self, s, cx, cy, result, record):
        col = {matches.WIN: ui.GREEN, matches.LOSS: ui.RED,
               matches.DRAW: ui.GOLD}.get(result, ui.mix(ui.BTN, self.accent, 0.35))
        pygame.draw.circle(s, col, (cx, cy), 13)
        dark = ui.mix(col, (0, 0, 0), 0.55)
        if result == matches.WIN:
            pygame.draw.lines(s, dark, False, [(cx - 6, cy), (cx - 2, cy + 5), (cx + 6, cy - 5)], 3)
        elif result == matches.LOSS:
            pygame.draw.line(s, dark, (cx - 5, cy - 5), (cx + 5, cy + 5), 3)
            pygame.draw.line(s, dark, (cx - 5, cy + 5), (cx + 5, cy - 5), 3)
        elif result == matches.DRAW:
            pygame.draw.line(s, dark, (cx - 5, cy - 3), (cx + 5, cy - 3), 3)
            pygame.draw.line(s, dark, (cx - 5, cy + 3), (cx + 5, cy + 3), 3)
        else:
            pygame.draw.circle(s, ui.TEXT_DIM, (cx, cy), 3)
        if record:
            pygame.draw.circle(s, ui.GOLD, (cx, cy), 16, 2)

    def _draw_row(self, s, rect, e, selected):
        stamp, mode, _score, result, dur, flags, info = e
        col = ui.PANEL_LIGHT if selected else ui.PANEL
        ui.draw_panel(s, rect, color=col, radius=10, shadow=False)
        if selected:
            pygame.draw.rect(s, self.accent, rect, 1, border_radius=10)
        pygame.draw.rect(s, self.accent, (rect.x, rect.y + 8, 3, rect.h - 16),
                         border_radius=2)
        self._badge(s, rect.x + 28, rect.centery, result, flags & matches.F_RECORD)

        title = "%s  ·  %s" % (fmt_score(e), mode_label(self.game_cls, mode))
        title = fit_text(ui.font(15, bold=True), title, rect.w - 52 - 90)
        img = ui.font(15, bold=True).render(title, True, ui.TEXT if selected else ui.TEXT_DIM)
        s.blit(img, (rect.x + 52, rect.y + 5))

        parts = []
        if result is not None:
            parts.append(t({matches.WIN: "match.win", matches.LOSS: "match.loss",
                            matches.DRAW: "match.draw"}[result]))
        if flags & matches.F_RECORD:
            parts.append(t("match.record"))
        if flags & matches.F_ABORT:
            parts.append(t("match.aborted"))
        if flags & matches.F_ARCHIVE:
            parts.append(t("match.archive"))
        elif flags & matches.F_DAILY:
            parts.append(t("match.daily"))
        parts += info_parts(self.game_cls, info)
        fnt = ui.font(11)
        sub = fit_text(fnt, "  ·  ".join(parts), rect.w - 52 - 150)
        img = fnt.render(sub, True, ui.GOLD if flags & matches.F_RECORD else ui.TEXT_FAINT)
        s.blit(img, (rect.x + 52, rect.y + 25))

        di = ui.font(13, bold=True).render(matches.format_duration(dur), True, self.accent)
        s.blit(di, di.get_rect(topright=(rect.right - 14, rect.y + 6)))
        si = ui.font(11).render(stamp, True, ui.TEXT_FAINT)
        s.blit(si, si.get_rect(topright=(rect.right - 14, rect.y + 25)))

    def _draw_empty(self, s):
        cx = self.width // 2
        cy = (self._top + self._bottom) // 2
        head = ui.font(17, bold=True).render(
            t("match.empty", game=str(self.game_cls.name)), True, ui.TEXT_DIM)
        s.blit(head, head.get_rect(center=(cx, cy - 12)))
        hint = ui.font(13).render(t("match.empty_hint"), True, ui.TEXT_FAINT)
        s.blit(hint, hint.get_rect(center=(cx, cy + 14)))

    def _draw_scrollbar(self, s, n):
        top, bottom = self._top, self._top + self.rows_visible * ROW_H
        track = pygame.Rect(self._right + 6, top, 4, bottom - top)
        pygame.draw.rect(s, ui.PANEL, track, border_radius=2)
        frac = self.rows_visible / float(n)
        h = max(24, int(track.h * frac))
        pos = self.first / float(max(1, n - self.rows_visible))
        y = top + int((track.h - h) * min(1.0, pos))
        pygame.draw.rect(s, ui.BORDER_LIGHT, (track.x, y, 4, h), border_radius=2)
