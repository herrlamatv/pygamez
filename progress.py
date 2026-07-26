# -*- coding: utf-8 -*-
"""
progress.py
===========
Der Erfolge-&-Statistik-Screen ("Fortschritt") der Spielesammlung.

Zwei Reiter (wie im Options-Screen):

- Erfolge      : alle Erfolge aus achievements.py, gruppiert nach Kategorie
                 (Allgemein / Punkte-Meilensteine / Besondere Momente), mit
                 Badge, Beschreibung, Freischalt-Datum bzw. Fortschrittsbalken.
- Statistiken  : Übersichtskarten (Gesamtspielzeit, Partien, Siege, Rekorde,
                 ausprobierte Spiele, Erfolge) + Lieblingsspiel + eine nach
                 Spielzeit sortierte Tabelle aller gespielten Spiele.

Beide Reiter scrollen (Mausrad, Pfeile, Bild-Tasten). Ein Klick auf eine
Zeile mit Spielbezug (Meilenstein/Spezial-Erfolg bzw. Statistik-Zeile)
springt direkt in den Vorspiel-Screen dieses Spiels.
"""

import pygame

import achievements
import stats
import ui
from game_base import InputEvent
from i18n import t
from menu import _Screen

# Zeilenhöhen/Abstände (Pixel).
_ROW_ACH = 46        # Erfolgs-Zeile
_ROW_GAME = 30       # Statistik-Zeile je Spiel
_ROW_HEAD = 30       # Kategorie-/Abschnitts-Überschrift
_GAP = 6


class ProgressScreen(_Screen):
    name = "Fortschritt"

    TABS = ("achievements", "stats")

    def __init__(self, surface, width, height, app, on_close):
        self.on_close = on_close
        self.tab = "achievements"
        self._tab_hover = None
        self.scroll = 0
        self._hover_row = None
        super().__init__(surface, width, height, app)
        self.name = t("progress.name")
        self._build()

    # ----- Aufbau -------------------------------------------------------

    def on_surface_changed(self):
        self._build()

    def _build(self):
        """Berechnet Layout + Zeilenliste des aktiven Reiters (einmalig,
        nicht pro Frame - alle Datei-Zugriffe passieren hier)."""
        W, H = self.width, self.height
        self._left = max(28, W // 2 - 330)
        self._right = W - self._left
        self._top = 118
        self._bottom = H - 52

        # Reiterleiste unter dem Titel.
        self._tab_font = ui.font(14, bold=True)
        self.tab_rects = []
        tx = self._left
        for key in self.TABS:
            tw = self._tab_font.size(t("progress.tab_" + key))[0] + 26
            self.tab_rects.append((pygame.Rect(tx, 48, tw, 26), key))
            tx += tw + 8

        # Daten einmal laden (nicht pro Frame).
        self._unlocked = achievements.unlocked()
        self._totals = stats.totals()
        import highscore
        self._scores = highscore.load_highscores()
        self._n_unlocked, self._n_total = achievements.counts()

        rows = []       # (y, h, kind, data) - y relativ zum Inhaltsanfang
        y = 0
        if self.tab == "achievements":
            by_cat = {"general": [], "milestone": [], "special": []}
            for d in achievements.all_defs():
                by_cat[d["cat"]].append(d)
            for cat in ("general", "milestone", "special"):
                if not by_cat[cat]:
                    continue
                rows.append((y, _ROW_HEAD, "head", t("progress.cat_" + cat)))
                y += _ROW_HEAD + _GAP
                for d in by_cat[cat]:
                    ts = self._unlocked.get(d["id"])
                    prog = None
                    if ts is None:
                        prog = achievements.progress_of(
                            d, totals=self._totals, scores=self._scores)
                    rows.append((y, _ROW_ACH, "ach", (d, ts, prog)))
                    y += _ROW_ACH + _GAP
                y += 6
        else:
            rows.append((y, 0, "cards", None))
            card_h = self._cards_height()
            y += card_h + 14

            entries = []
            for cls in self.app._game_classes:
                key = cls.highscore_key
                st = stats.get(key)
                if st["plays"] > 0 or st["time"] > 0:
                    entries.append((cls, st))
            entries.sort(key=lambda e: e[1]["time"], reverse=True)
            self._n_played = len(entries)

            if entries:
                rows.append((y, _ROW_HEAD, "table_head", None))
                y += _ROW_HEAD + 2
                for cls, st in entries:
                    rows.append((y, _ROW_GAME, "game", (cls, st)))
                    y += _ROW_GAME + 2
                rest = len(self.app._game_classes) - len(entries)
                if rest > 0:
                    y += 6
                    rows.append((y, _ROW_HEAD, "note",
                                 t("progress.never_played", n=rest)))
                    y += _ROW_HEAD
            else:
                rows.append((y, _ROW_HEAD, "note", t("progress.no_stats")))
                y += _ROW_HEAD

        self._rows = rows
        self._content_h = y
        self._clamp_scroll()

    def _cards_height(self):
        """Höhe des Karten-Blocks oben im Statistik-Reiter (Layout-abhängig)."""
        cols = 3 if (self._right - self._left) >= 560 else 2
        rows = (6 + cols - 1) // cols
        self._card_cols = cols
        self._card_h = 64
        # + Zeile fürs Lieblingsspiel unter den Karten.
        return rows * (self._card_h + 10) + 30

    def _viewport(self):
        return self._bottom - self._top

    def _clamp_scroll(self):
        self.scroll = max(0, min(self.scroll,
                                 max(0, self._content_h - self._viewport())))

    # ----- Eingabe ------------------------------------------------------

    def _switch_tab(self, key):
        if key == self.tab:
            return
        self.tab = key
        self.scroll = 0
        self._hover_row = None
        self._build()
        self.play_sound("move")

    def _close(self):
        self.on_close()

    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            key = event.key
            if key == "Escape":
                self._close()
            elif key == "Tab":
                idx = self.TABS.index(self.tab)
                self._switch_tab(self.TABS[(idx + 1) % len(self.TABS)])
            elif key in ("Up", "w"):
                self.scroll -= 40
                self._clamp_scroll()
            elif key in ("Down", "s"):
                self.scroll += 40
                self._clamp_scroll()
            elif key == "Prior":
                self.scroll -= self._viewport()
                self._clamp_scroll()
            elif key == "Next":
                self.scroll += self._viewport()
                self._clamp_scroll()
            elif key in ("Left", "a", "Right", "d"):
                idx = self.TABS.index(self.tab)
                self._switch_tab(self.TABS[(idx + 1) % len(self.TABS)])
        elif event.kind == InputEvent.WHEEL:
            self.scroll -= event.delta * 48
            self._clamp_scroll()
        elif event.kind == InputEvent.MOUSEMOVE:
            self._tab_hover = None
            for r, key in self.tab_rects:
                if r.collidepoint(event.pos):
                    self._tab_hover = key
            self._hover_row = self._row_at(event.pos)
        elif event.kind == InputEvent.MOUSEDOWN:
            for r, key in self.tab_rects:
                if r.collidepoint(event.pos):
                    self._switch_tab(key)
                    return
            hit = self._row_at(event.pos)
            if hit is not None:
                cls = self._row_game(self._rows[hit])
                if cls is not None:
                    self.play_sound("click")
                    self.app.spiel_starten(cls)

    def _row_at(self, pos):
        """Index der klickbaren Zeile unter der Maus (oder None)."""
        x, y = pos
        if not (self._left <= x <= self._right
                and self._top <= y <= self._bottom):
            return None
        cy = y - self._top + self.scroll
        for i, (ry, rh, kind, _data) in enumerate(self._rows):
            if ry <= cy <= ry + rh and kind in ("ach", "game"):
                return i
        return None

    @staticmethod
    def _row_game(row):
        """Spielklasse einer Zeile (für den Klick-Sprung) oder None."""
        _y, _h, kind, data = row
        if kind == "game":
            return data[0]
        if kind == "ach":
            return data[0].get("game_cls")
        return None

    # ----- Zeichnen -----------------------------------------------------

    def draw(self):
        s = self.surface
        W, H = self.width, self.height
        ui.draw_background(s, W, H, stars=False)
        ui.draw_title(s, W, t("progress.name"), y=40,
                      big=ui.font(30, bold=True), accent=ui.GOLD)

        # Reiterleiste.
        for r, key in self.tab_rects:
            active = (key == self.tab)
            hover = (key == self._tab_hover)
            col = ui.PANEL_LIGHT if (active or hover) else ui.PANEL
            pygame.draw.rect(s, col, r, border_radius=13)
            pygame.draw.rect(s, ui.GOLD if active else ui.BORDER, r, 1,
                             border_radius=13)
            img = self._tab_font.render(t("progress.tab_" + key), True,
                                        ui.TEXT if active else ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=r.center))

        # Kopfzeile rechts: Gesamt-Fortschritt der Erfolge.
        head_font = ui.font(13, bold=True)
        head = head_font.render(
            t("progress.unlocked_of", n=self._n_unlocked, m=self._n_total),
            True, ui.GOLD)
        bar_w = min(150, W // 5)
        bx = self._right - bar_w
        by = 61
        s.blit(head, head.get_rect(bottomright=(self._right, by - 6)))
        pygame.draw.rect(s, ui.PANEL_LIGHT, (bx, by, bar_w, 7),
                         border_radius=4)
        if self._n_total:
            fw = int(bar_w * self._n_unlocked / self._n_total)
            if fw > 0:
                pygame.draw.rect(s, ui.GOLD, (bx, by, fw, 7), border_radius=4)
        pygame.draw.rect(s, ui.BORDER, (bx, by, bar_w, 7), 1, border_radius=4)

        # Inhalt mit Clipping + Scroll.
        clip = pygame.Rect(0, self._top, W, self._bottom - self._top)
        s.set_clip(clip)
        for i, (ry, rh, kind, data) in enumerate(self._rows):
            y = self._top + ry - self.scroll
            if y + rh < self._top - 80 or y > self._bottom + 10:
                if kind != "cards":
                    continue
            if kind == "head":
                img = ui.font(13, bold=True).render(data, True, ui.ACCENT)
                s.blit(img, (self._left, y + 8))
                pygame.draw.line(s, ui.BORDER,
                                 (self._left + img.get_width() + 12, y + 16),
                                 (self._right, y + 16))
            elif kind == "note":
                img = ui.font(13).render(data, True, ui.TEXT_FAINT)
                s.blit(img, (self._left, y + 8))
            elif kind == "ach":
                self._draw_ach_row(s, y, rh, data, hover=(i == self._hover_row))
            elif kind == "cards":
                self._draw_cards(s, y)
            elif kind == "table_head":
                self._draw_table_head(s, y)
            elif kind == "game":
                self._draw_game_row(s, y, rh, data,
                                    hover=(i == self._hover_row))
        s.set_clip(None)

        # Scrollbar rechts (nur wenn nötig).
        vp = self._viewport()
        if self._content_h > vp:
            track = pygame.Rect(W - 10, self._top, 4, vp)
            pygame.draw.rect(s, ui.PANEL, track, border_radius=2)
            th = max(24, int(vp * vp / self._content_h))
            ty = self._top + int((vp - th) * self.scroll
                                 / max(1, self._content_h - vp))
            pygame.draw.rect(s, ui.BORDER_LIGHT, (W - 10, ty, 4, th),
                             border_radius=2)

        ui.draw_footer(s, W, H, t("progress.hint"))

    # -- Erfolge ---------------------------------------------------------

    def _draw_ach_row(self, s, y, rh, data, hover=False):
        d, ts, prog = data
        unlocked = ts is not None
        row = pygame.Rect(self._left, y, self._right - self._left, rh)
        color = ui.PANEL_LIGHT if (hover and d.get("game_cls")) else ui.PANEL
        ui.draw_panel(s, row, color=color, shadow=False, radius=10)

        accent = (ui.game_color(d["game_cls"].__name__)
                  if d.get("game_cls") else ui.GOLD)
        achievements.draw_badge(s, (row.x + 26, row.centery), 15,
                                d["icon"], accent, locked=not unlocked)

        name_col = ui.TEXT if unlocked else ui.TEXT_DIM
        desc_col = ui.TEXT_DIM if unlocked else ui.TEXT_FAINT
        name = ui.font(15, bold=True).render(
            achievements.display_name(d), True, name_col)
        desc = ui.font(12).render(achievements.display_desc(d), True, desc_col)
        tx = row.x + 50
        max_w = row.w - 50 - 130
        s.blit(name, (tx, row.y + 6), (0, 0, max_w, name.get_height()))
        s.blit(desc, (tx, row.y + 9 + name.get_height()),
               (0, 0, max_w, desc.get_height()))

        if unlocked:
            date = ui.font(11).render(str(ts)[:10], True, ui.TEXT_FAINT)
            s.blit(date, date.get_rect(midright=(row.right - 14, row.centery)))
        elif prog:
            cur, target = prog
            bw = 86
            bx = row.right - 14 - bw
            byy = row.centery + 3
            label = ui.font(11).render(self._fmt_progress(d, cur, target),
                                       True, ui.TEXT_FAINT)
            s.blit(label, label.get_rect(midright=(row.right - 14, byy - 8)))
            pygame.draw.rect(s, ui.BTN, (bx, byy, bw, 5), border_radius=3)
            fw = int(bw * cur / max(1, target))
            if fw > 0:
                pygame.draw.rect(s, ui.ACCENT_SOFT, (bx, byy, fw, 5),
                                 border_radius=3)

    @staticmethod
    def _fmt_progress(d, cur, target):
        if d.get("stat") == "time":
            return "%s / %s" % (stats.format_time(cur),
                                stats.format_time(target))
        return "%d / %d" % (cur, target)

    # -- Statistiken -----------------------------------------------------

    def _draw_cards(self, s, y):
        totals = self._totals
        cards = [
            ("clock",  stats.format_time(totals["time"]), t("progress.total_time")),
            ("flag",   str(totals["plays"]), t("progress.total_plays")),
            ("medal",  str(totals["wins"]), t("progress.total_wins")),
            ("trophy", str(totals["records"]), t("progress.total_records")),
            ("gem",    "%d / %d" % (totals["distinct"],
                                    len(self.app._game_classes)),
             t("progress.distinct")),
            ("star",   "%d / %d" % (self._n_unlocked, self._n_total),
             t("progress.achievements")),
        ]
        cols = self._card_cols
        gap = 10
        cw = (self._right - self._left - (cols - 1) * gap) // cols
        for i, (icon, value, label) in enumerate(cards):
            cx = self._left + (i % cols) * (cw + gap)
            cy = y + (i // cols) * (self._card_h + gap)
            rect = pygame.Rect(cx, cy, cw, self._card_h)
            ui.draw_panel(s, rect, shadow=False, radius=10)
            achievements.draw_badge(s, (rect.x + 24, rect.centery), 13,
                                    icon, ui.ACCENT)
            val = ui.font(19, bold=True).render(value, True, ui.TEXT)
            lab = ui.font(11).render(label, True, ui.TEXT_DIM)
            s.blit(val, (rect.x + 46, rect.y + 10))
            s.blit(lab, (rect.x + 46, rect.y + 14 + val.get_height()),
                   (0, 0, rect.w - 56, lab.get_height()))

        # Lieblingsspiel (meiste Spielzeit) unter den Karten.
        rows = (len(cards) + cols - 1) // cols
        fy = y + rows * (self._card_h + gap) + 4
        fav_key = totals.get("favorite")
        if fav_key:
            cls = next((c for c in self.app._game_classes
                        if c.highscore_key == fav_key), None)
            if cls is not None:
                col = ui.game_color(cls.__name__)
                img1 = ui.font(13).render(t("progress.favorite") + ":", True,
                                          ui.TEXT_DIM)
                img2 = ui.font(13, bold=True).render(
                    " %s (%s)" % (cls.name,
                                  stats.format_time(stats.get(fav_key)["time"])),
                    True, col)
                s.blit(img1, (self._left, fy))
                s.blit(img2, (self._left + img1.get_width(), fy))

    def _table_columns(self):
        """(x-Positionen der Zahlen-Spalten von rechts) - passt sich der Breite an."""
        w = self._right - self._left
        col_w = max(64, min(92, w // 7))
        return [self._right - 10 - i * col_w for i in range(4)], col_w

    def _draw_table_head(self, s, y):
        xs, _cw = self._table_columns()
        fnt = ui.font(11, bold=True)
        img = fnt.render(t("progress.col_game"), True, ui.TEXT_FAINT)
        s.blit(img, (self._left + 8, y + 10))
        for x, key in zip(xs, ("col_best", "col_wins", "col_time",
                               "col_plays")):
            img = fnt.render(t("progress." + key), True, ui.TEXT_FAINT)
            s.blit(img, img.get_rect(topright=(x, y + 10)))
        pygame.draw.line(s, ui.BORDER, (self._left, y + 26),
                         (self._right, y + 26))

    def _draw_game_row(self, s, y, rh, data, hover=False):
        cls, st = data
        row = pygame.Rect(self._left, y, self._right - self._left, rh)
        if hover:
            ui.draw_panel(s, row, color=ui.PANEL_LIGHT, shadow=False, radius=8)
        color = ui.game_color(cls.__name__)
        pygame.draw.circle(s, color, (row.x + 12, row.centery), 4)
        xs, cw = self._table_columns()
        name_w = xs[-1] - cw + 10 - (row.x + 24)
        name = ui.font(13, bold=True).render(cls.name, True, ui.TEXT)
        s.blit(name, (row.x + 24, row.centery - name.get_height() // 2),
               (0, 0, max(40, name_w), name.get_height()))
        fnt = ui.font(12)
        values = (str(self._scores.get(cls.highscore_key, 0)),
                  str(st["wins"]) if (st["wins"] or st["losses"]) else "-",
                  stats.format_time(st["time"]),
                  str(st["plays"]))
        for x, val in zip(xs, values):
            img = fnt.render(val, True, ui.TEXT_DIM)
            s.blit(img, img.get_rect(midright=(x, row.centery)))
