# -*- coding: utf-8 -*-
"""
progress.py
===========
Der Erfolge-&-Statistik-Screen ("Fortschritt") der Spielesammlung.

Fünf Reiter:

- Erfolge     : alle Erfolge aus achievements.py, gruppiert nach Kategorie
                (Allgemein / Punkte-Meilensteine / Meister / Stammgast /
                Besondere Momente), mit Badge in der Farbe der Seltenheits-
                stufe, Punktwert, Beschreibung, Freischalt-Datum bzw.
                Fortschrittsbalken. Oben: Level, Rang und XP-Balken.
- Statistiken : Übersichtskarten (Spielzeit, Partien, Siege, Rekorde,
                Spiele, Erfolge) + Top-10-Balken der Spielzeit +
                Sieg/Niederlage-Ring + Tabelle aller gespielten Spiele.
- Kalender    : Spieltage-Karte (Heatmap), Serien, Wochentags-Balken und
                Tageszeit-Ring - dazu der "PyGameZ-Rückblick" mit Fakten.
- Verlauf     : alles mit Zeitstempel - Erfolge, Rekorde, Level-Aufstiege,
                Duelle - plus der Punkteverlauf eines gewählten Spiels.
- Profile     : alle Profile mit Level/Punkten/Spielzeit, die Geräte-
                Bestenliste je Spiel und die Duell-Bilanz Profil gegen Profil.

Alle Reiter scrollen (Mausrad, Pfeile, Bild-Tasten). Ein Klick auf eine
Zeile mit Spielbezug springt direkt in den Vorspiel-Screen dieses Spiels.
Die Daten werden EINMAL beim Aufbau gelesen (``_build``), nie pro Frame.
"""

import datetime
import time

import pygame

import achievements
import charts
import levels
import stats
import ui
from game_base import InputEvent
from i18n import t
from menu import _Screen

# Zeilenhöhen/Abstände (Pixel).
_ROW_ACH = 46        # Erfolgs-Zeile
_ROW_GAME = 30       # Statistik-Zeile je Spiel
_ROW_HEAD = 30       # Kategorie-/Abschnitts-Überschrift
_ROW_EVENT = 28      # Verlaufs-Zeile
_GAP = 6

# Reihenfolge der Erfolgs-Kategorien im ersten Reiter.
_CATS = ("general", "milestone", "master", "regular", "special")

# Filter im Erfolge-Reiter.
_FILTERS = ("all", "unlocked", "locked")


class ProgressScreen(_Screen):
    name = "Fortschritt"

    TABS = ("achievements", "stats", "calendar", "timeline", "profiles")

    def __init__(self, surface, width, height, app, on_close):
        self.on_close = on_close
        self.tab = "achievements"
        self.filter = "all"
        self._tab_hover = None
        self.scroll = 0
        self._hover_row = None
        self._mouse = (0, 0)
        self._hits = {}          # Trefferflächen der Diagramme (letztes Frame)
        self._tip = None         # Tooltip-Zeilen für dieses Frame
        self._series_game = None  # gewähltes Spiel im Verlauf-Reiter
        self._buttons = []       # (rect, callback) - Export/Import & Co.
        self._flash = ""         # kurze Rückmeldung (Export/Import)
        self._flash_t = 0.0
        super().__init__(surface, width, height, app)
        self.name = t("progress.name")
        self._build()

    # ----- Aufbau -------------------------------------------------------

    def set_tab(self, tab):
        if tab in self.TABS:
            self.tab = tab
            self.scroll = 0
            self._build()

    def on_surface_changed(self):
        self._build()

    def _build(self):
        """Berechnet Layout + Zeilenliste des aktiven Reiters (einmalig,
        nicht pro Frame - alle Datei-Zugriffe passieren hier)."""
        W, H = self.width, self.height
        self._left = max(28, W // 2 - 330)
        self._right = W - self._left
        self._top = 132
        self._bottom = H - 52
        self._buttons = []

        # Reiterleiste unter dem Titel.
        self._tab_font = ui.font(13, bold=True)
        self.tab_rects = []
        tx = self._left
        for key in self.TABS:
            tw = self._tab_font.size(t("progress.tab_" + key))[0] + 20
            self.tab_rects.append((pygame.Rect(tx, 48, tw, 26), key))
            tx += tw + 6

        # Daten einmal laden (nicht pro Frame).
        import highscore
        import profiles
        self._profiles = profiles
        self._unlocked = achievements.unlocked()
        self._totals = stats.totals()
        self._scores = highscore.load_highscores()
        self._n_unlocked, self._n_total = achievements.counts()
        self._points = achievements.total_points()
        self._max_points = achievements.max_points()
        self._level, _pts = stats.level_of()

        rows = []       # (y, h, kind, data) - y relativ zum Inhaltsanfang
        y = 0
        builder = getattr(self, "_build_" + self.tab)
        y = builder(rows, y)

        self._rows = rows
        self._content_h = y
        self._clamp_scroll()

    # -- Reiter 1: Erfolge ----------------------------------------------

    def _build_achievements(self, rows, y):
        rows.append((y, 26, "filter", None))
        y += 26 + _GAP
        by_cat = {c: [] for c in _CATS}
        for d in achievements.all_defs():
            ts = self._unlocked.get(d["id"])
            if self.filter == "unlocked" and ts is None:
                continue
            if self.filter == "locked" and ts is not None:
                continue
            by_cat[d["cat"]].append(d)
        for cat in _CATS:
            if not by_cat[cat]:
                continue
            got = sum(1 for d in by_cat[cat] if d["id"] in self._unlocked)
            rows.append((y, _ROW_HEAD, "head",
                         "%s   %d/%d" % (t("progress.cat_" + cat), got,
                                         len(by_cat[cat]))))
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
        if len(rows) <= 1:
            rows.append((y, _ROW_HEAD, "note", t("progress.filter_empty")))
            y += _ROW_HEAD
        return y

    # -- Reiter 2: Statistiken ------------------------------------------

    def _build_stats(self, rows, y):
        rows.append((y, self._cards_height(), "cards", None))
        y += self._cards_height() + 14

        entries = []
        for cls in self.app._game_classes:
            key = cls.highscore_key
            st = stats.get(key)
            if st["plays"] > 0 or st["time"] > 0:
                entries.append((cls, st))
        entries.sort(key=lambda e: e[1]["time"], reverse=True)
        self._entries = entries

        if entries:
            top = entries[:10]
            rows.append((y, _ROW_HEAD, "head", t("progress.top_games")))
            y += _ROW_HEAD
            h = 22 * len(top) + 10
            rows.append((y, h, "chart_top", top))
            y += h + 14

            rows.append((y, _ROW_HEAD, "head", t("progress.balance")))
            y += _ROW_HEAD
            rows.append((y, 150, "chart_split", None))
            y += 150 + 14

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
        return y

    # -- Reiter 3: Kalender ---------------------------------------------

    def _build_calendar(self, rows, y):
        self._days = stats.days_map()
        self._day_time = {d: v.get("t", 0.0) for d, v in self._days.items()}
        self._streak = stats.streak()
        self._facts = self._build_facts()

        h = 24 + 20 * len(self._facts)
        rows.append((y, h, "wrapped", None))
        y += h + 14

        rows.append((y, _ROW_HEAD, "head", t("progress.calendar_head")))
        y += _ROW_HEAD
        rows.append((y, 132, "heat", None))
        y += 132 + 10
        rows.append((y, 40, "streaks", None))
        y += 40 + 14

        rows.append((y, _ROW_HEAD, "head", t("progress.weekday_head")))
        y += _ROW_HEAD
        rows.append((y, 110, "weekdays", None))
        y += 110 + 14

        rows.append((y, _ROW_HEAD, "head", t("progress.clock_head")))
        y += _ROW_HEAD
        rows.append((y, 190, "clock", None))
        y += 190 + 10
        return y

    def _build_facts(self):
        """Ein paar erzählende Fakten für den "Rückblick" (nur was da ist)."""
        facts = []
        totals = self._totals
        days = self._days
        if not days:
            return facts
        best_day = max(days.items(), key=lambda kv: kv[1].get("t", 0.0))
        if best_day[1].get("t", 0) > 0:
            facts.append(t("progress.fact_best_day",
                           day=_pretty_date(best_day[0]),
                           time=stats.format_time(best_day[1]["t"])))
        hours = stats.hours_list()
        if any(hours):
            peak = hours.index(max(hours))
            facts.append(t("progress.fact_peak_hour", h=peak,
                           h2=(peak + 1) % 24))
        if totals.get("longest"):
            facts.append(t("progress.fact_longest",
                           time=stats.format_time(totals["longest"])))
        fav = totals.get("favorite")
        if fav:
            cls = self._game_cls(fav)
            if cls is not None:
                facts.append(t("progress.fact_favorite", game=cls.name,
                               time=stats.format_time(stats.get(fav)["time"])))
        rare = achievements.rarest_unlocked(1)
        if rare:
            d, _ts = rare[0]
            facts.append(t("progress.fact_rarest",
                           name=achievements.display_name(d),
                           rarity=achievements.rarity_name(d)))
        if totals.get("days"):
            facts.append(t("progress.fact_days", n=totals["days"],
                           avg=stats.format_time(
                               totals["time"] / max(1, totals["days"]))))
        return facts[:6]

    # -- Reiter 4: Verlauf ----------------------------------------------

    def _build_timeline(self, rows, y):
        played = stats.played_games()
        if self._series_game not in played:
            self._series_game = played[0] if played else None
        self._played_series = played[:8]

        if self._series_game:
            rows.append((y, 26, "series_pick", None))
            y += 26 + 6
            rows.append((y, 140, "chart_series", None))
            y += 140 + 14

        rows.append((y, _ROW_HEAD, "head", t("progress.timeline_head")))
        y += _ROW_HEAD + 2
        events = stats.events(limit=400)
        if not events:
            rows.append((y, _ROW_HEAD, "note", t("progress.no_events")))
            return y + _ROW_HEAD
        last_day = None
        for ev in events:
            day = str(ev.get("t", ""))[:10]
            if day != last_day:
                last_day = day
                rows.append((y, 24, "day_head", _pretty_date(day)))
                y += 24
            rows.append((y, _ROW_EVENT, "event", ev))
            y += _ROW_EVENT + 2
        return y

    # -- Reiter 5: Profile ----------------------------------------------

    def _build_profiles(self, rows, y):
        self._plist = self._profiles.all_profiles()
        self._pstats = {}
        for pid, _meta in self._plist:
            tot = stats.totals(pid)
            self._pstats[pid] = tot

        rows.append((y, _ROW_HEAD, "head", t("progress.profiles_head")))
        y += _ROW_HEAD + 2
        for pid, meta in self._plist:
            rows.append((y, 54, "profile", (pid, meta)))
            y += 54 + _GAP
        y += 8

        # Geräte-Bestenliste: wer hält je Spiel den Bestwert?
        board = self._build_board()
        if board:
            rows.append((y, _ROW_HEAD, "head", t("progress.board_head")))
            y += _ROW_HEAD + 2
            for entry in board:
                rows.append((y, _ROW_GAME, "board", entry))
                y += _ROW_GAME + 2
            y += 8

        duels = self._build_duels()
        if duels:
            rows.append((y, _ROW_HEAD, "head", t("progress.duel_head")))
            y += _ROW_HEAD + 2
            for entry in duels:
                rows.append((y, _ROW_GAME, "duel", entry))
                y += _ROW_GAME + 2
        return y

    def _build_board(self):
        """[(cls, [(pid, bestwert), ...]), ...] - Bestwerte je Spiel."""
        import store
        out = []
        per_profile = {}
        for pid, _meta in self._plist:
            per_profile[pid] = store.load_section("highscores", profile=pid)
        for cls in self.app._game_classes:
            key = cls.highscore_key
            values = []
            for pid, _meta in self._plist:
                try:
                    score = int(per_profile[pid].get(key, 0))
                except (TypeError, ValueError):
                    score = 0
                if score > 0:
                    values.append((pid, score))
            if len(values) >= 1:
                values.sort(key=lambda v: -v[1])
                out.append((cls, values))
        # Nur Spiele zeigen, die mehr als ein Profil gespielt hat - sonst ist
        # es keine Bestenliste, sondern nur die eigene Tabelle.
        multi = [e for e in out if len(e[1]) > 1]
        return multi or []

    def _build_duels(self):
        """[(cls, pid_a, pid_b, siege_a, siege_b), ...] aus den Duell-Bilanzen."""
        out = []
        me = self._profiles.active_id()
        table = stats.duel_table(me)
        for other, games in table.items():
            for key, wl in games.items():
                cls = self._game_cls(key)
                if cls is None:
                    continue
                out.append((cls, me, other, wl.get("w", 0), wl.get("l", 0)))
        out.sort(key=lambda e: -(e[3] + e[4]))
        return out

    # ----- Hilfen -------------------------------------------------------

    def _game_cls(self, key):
        return next((c for c in self.app._game_classes
                     if c.highscore_key == key), None)

    def _cards_height(self):
        """Höhe des Karten-Blocks oben im Statistik-Reiter (Layout-abhängig)."""
        cols = 3 if (self._right - self._left) >= 560 else 2
        rows = (8 + cols - 1) // cols
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
        if key in ("calendar", "timeline"):
            achievements.event("statistician")
        if key == "calendar":
            achievements.event("wrapped")

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
            elif key in ("Left", "a"):
                idx = self.TABS.index(self.tab)
                self._switch_tab(self.TABS[(idx - 1) % len(self.TABS)])
            elif key in ("Right", "d"):
                idx = self.TABS.index(self.tab)
                self._switch_tab(self.TABS[(idx + 1) % len(self.TABS)])
            elif key == "e":
                self._export()
            elif key == "i":
                self._import()
        elif event.kind == InputEvent.WHEEL:
            self.scroll -= event.delta * 48
            self._clamp_scroll()
        elif event.kind == InputEvent.MOUSEMOVE:
            self._mouse = event.pos
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
            for r, cb in self._buttons:
                if r.collidepoint(event.pos):
                    self.play_sound("click")
                    cb()
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
            if ry <= cy <= ry + rh and kind in ("ach", "game", "board"):
                return i
        return None

    @staticmethod
    def _row_game(row):
        """Spielklasse einer Zeile (für den Klick-Sprung) oder None."""
        _y, _h, kind, data = row
        if kind in ("game", "board"):
            return data[0]
        if kind == "ach":
            return data[0].get("game_cls")
        return None

    # ----- Export / Import ----------------------------------------------

    def _export(self):
        import filepick
        if not filepick.available():
            return
        path = filepick.save_as("pygamez-stats" + stats.EXT, exts=(stats.EXT, ".json"),
                                title=t("progress.export"))
        if not path:
            return
        ok = stats.export_to(path)
        self._set_flash(t("progress.export_ok") if ok
                        else t("progress.export_err"))

    def _import(self):
        import filepick
        if not filepick.available():
            return
        path = filepick.open_file(title=t("progress.import"),
                                  exts=stats.IMPORT_EXTS)
        if not path:
            return
        ok, reason = stats.import_from(path)
        self._set_flash(t("progress.import_ok") if ok
                        else t("progress.import_err", reason=reason))
        if ok:
            self._build()

    def _set_flash(self, text):
        self._flash = text
        self._flash_t = time.monotonic()

    # ----- Zeichnen -----------------------------------------------------

    def draw(self):
        s = self.surface
        W, H = self.width, self.height
        self._tip = None
        self._buttons = []
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

        self._draw_level_head(s, W)

        # Inhalt mit Clipping + Scroll.
        clip = pygame.Rect(0, self._top, W, self._bottom - self._top)
        s.set_clip(clip)
        for i, (ry, rh, kind, data) in enumerate(self._rows):
            y = self._top + ry - self.scroll
            if y + rh < self._top - 120 or y > self._bottom + 20:
                continue
            drawer = getattr(self, "_draw_" + kind, None)
            if drawer is not None:
                drawer(s, y, rh, data, i)
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

        if self._tip:
            charts.tooltip(s, self._mouse, self._tip, width_limit=W)

        hint = t("progress.hint")
        if self._flash and time.monotonic() - self._flash_t < 4.0:
            hint = self._flash
        ui.draw_footer(s, W, H, hint)

    def _draw_level_head(self, s, W):
        """Kopfzeile: Level, Rang, XP-Balken und Erfolgs-Fortschritt."""
        level, points = stats.level_of()
        lvl, in_level, need = levels.progress(points)
        color = levels.rank_color(lvl)
        y = 84
        achievements.draw_badge(s, (self._left + 16, y + 12), 15, "bolt", color)
        name = ui.font(15, bold=True).render(
            "%s %d  ·  %s" % (t("level.short"), lvl, levels.rank_title(lvl)),
            True, ui.TEXT)
        s.blit(name, (self._left + 38, y))
        sub = ui.font(11).render(
            t("progress.points", n=points) + "   ·   "
            + t("progress.next_level", n=max(0, need - in_level), m=lvl + 1),
            True, ui.TEXT_DIM)
        s.blit(sub, (self._left + 38, y + name.get_height() + 1))

        bar_w = min(190, W // 4)
        bx = self._right - bar_w
        pygame.draw.rect(s, ui.PANEL_LIGHT, (bx, y + 4, bar_w, 7),
                         border_radius=4)
        fw = int(bar_w * min(1.0, in_level / max(1, need)))
        if fw > 0:
            pygame.draw.rect(s, color, (bx, y + 4, fw, 7), border_radius=4)
        pygame.draw.rect(s, ui.BORDER, (bx, y + 4, bar_w, 7), 1,
                         border_radius=4)
        head = ui.font(11, bold=True).render(
            t("progress.unlocked_of", n=self._n_unlocked, m=self._n_total),
            True, ui.GOLD)
        s.blit(head, head.get_rect(topright=(self._right, y + 15)))

    # -- Zeilen-Zeichner (ein "_draw_<kind>" je Zeilentyp) ---------------

    def _draw_head(self, s, y, rh, data, _i):
        img = ui.font(13, bold=True).render(data, True, ui.ACCENT)
        s.blit(img, (self._left, y + 8))
        pygame.draw.line(s, ui.BORDER,
                         (self._left + img.get_width() + 12, y + 16),
                         (self._right, y + 16))

    def _draw_day_head(self, s, y, rh, data, _i):
        img = ui.font(11, bold=True).render(data, True, ui.TEXT_FAINT)
        s.blit(img, (self._left, y + 6))

    def _draw_note(self, s, y, rh, data, _i):
        img = ui.font(13).render(data, True, ui.TEXT_FAINT)
        s.blit(img, (self._left, y + 8))

    def _draw_filter(self, s, y, rh, _data, _i):
        """Filterleiste im Erfolge-Reiter + Seltenheits-Übersicht."""
        x = self._left
        fnt = ui.font(11, bold=True)
        for key in _FILTERS:
            label = t("progress.filter_" + key)
            w = fnt.size(label)[0] + 18
            r = pygame.Rect(x, y, w, 22)
            active = (self.filter == key)
            pygame.draw.rect(s, ui.PANEL_LIGHT if active else ui.PANEL, r,
                             border_radius=11)
            pygame.draw.rect(s, ui.ACCENT if active else ui.BORDER, r, 1,
                             border_radius=11)
            img = fnt.render(label, True, ui.TEXT if active else ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=r.center))
            self._buttons.append((r, lambda k=key: self._set_filter(k)))
            x += w + 6

        counts = achievements.rarity_counts()
        x = self._right
        for rarity in reversed(achievements.RARITY_ORDER):
            got, total = counts.get(rarity, (0, 0))
            if not total:
                continue
            label = "%d/%d" % (got, total)
            img = ui.font(10, bold=True).render(label, True,
                                                achievements.rarity_color(rarity))
            x -= img.get_width() + 14
            pygame.draw.circle(s, achievements.rarity_color(rarity),
                               (x - 6, y + 11), 4)
            s.blit(img, (x, y + 11 - img.get_height() // 2))
            box = pygame.Rect(x - 12, y, img.get_width() + 14, 22)
            if box.collidepoint(self._mouse):
                self._tip = [achievements.rarity_name(rarity),
                             t("progress.rarity_points",
                               n=achievements.RARITY[rarity]["points"])]

    def _set_filter(self, key):
        self.filter = key
        self.scroll = 0
        self._build()

    def _draw_ach(self, s, y, rh, data, i):
        d, ts, prog = data
        unlocked = ts is not None
        row = pygame.Rect(self._left, y, self._right - self._left, rh)
        hover = (i == self._hover_row)
        color = ui.PANEL_LIGHT if (hover and d.get("game_cls")) else ui.PANEL
        ui.draw_panel(s, row, color=color, shadow=False, radius=10)

        rarity_col = achievements.rarity_color(d)
        if unlocked:
            # Seltenheits-Streifen links - auf einen Blick sichtbar.
            pygame.draw.rect(s, rarity_col, (row.x, row.y + 6, 3, rh - 12),
                             border_radius=2)
        accent = rarity_col if unlocked else ui.BORDER_LIGHT
        achievements.draw_badge(s, (row.x + 26, row.centery), 15,
                                d["icon"], accent, locked=not unlocked)

        name_col = ui.TEXT if unlocked else ui.TEXT_DIM
        desc_col = ui.TEXT_DIM if unlocked else ui.TEXT_FAINT
        name = ui.font(15, bold=True).render(
            achievements.display_name(d), True, name_col)
        desc = ui.font(12).render(achievements.display_desc(d), True, desc_col)
        tx = row.x + 50
        max_w = row.w - 50 - 150
        s.blit(name, (tx, row.y + 5), (0, 0, max_w, name.get_height()))
        s.blit(desc, (tx, row.y + 8 + name.get_height()),
               (0, 0, max_w, desc.get_height()))

        # Punktwert + Seltenheit rechts oben.
        pts = ui.font(11, bold=True).render(
            "+%d" % achievements.points_of(d), True,
            rarity_col if unlocked else ui.TEXT_FAINT)
        s.blit(pts, pts.get_rect(topright=(row.right - 14, row.y + 6)))

        if unlocked:
            date = ui.font(11).render(str(ts)[:10], True, ui.TEXT_FAINT)
            s.blit(date, date.get_rect(bottomright=(row.right - 14,
                                                    row.bottom - 6)))
        elif prog:
            cur, target = prog
            bw = 86
            bx = row.right - 14 - bw
            byy = row.bottom - 14
            label = ui.font(11).render(self._fmt_progress(d, cur, target),
                                       True, ui.TEXT_FAINT)
            s.blit(label, label.get_rect(midright=(row.right - 14, byy - 8)))
            pygame.draw.rect(s, ui.BTN, (bx, byy, bw, 5), border_radius=3)
            fw = int(bw * cur / max(1, target))
            if fw > 0:
                pygame.draw.rect(s, ui.ACCENT_SOFT, (bx, byy, fw, 5),
                                 border_radius=3)
        if hover and row.collidepoint(self._mouse):
            self._tip = [achievements.display_name(d),
                         "%s  ·  +%d" % (achievements.rarity_name(d),
                                         achievements.points_of(d))]

    @staticmethod
    def _fmt_progress(d, cur, target):
        if d.get("stat") == "time":
            return "%s / %s" % (stats.format_time(cur),
                                stats.format_time(target))
        return "%d / %d" % (cur, target)

    # -- Statistik-Reiter -------------------------------------------------

    def _draw_cards(self, s, y, rh, _data, _i):
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
            ("calendar", str(totals.get("days", 0)), t("progress.total_days")),
            ("bolt",   "%d" % totals.get("points", 0), t("progress.total_points")),
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
            cls = self._game_cls(fav_key)
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

    def _draw_chart_top(self, s, y, rh, data, _i):
        rect = pygame.Rect(self._left, y + 4, self._right - self._left, rh - 8)
        values = [st["time"] for _cls, st in data]
        labels = [cls.name for cls, _st in data]
        colors = [ui.game_color(cls.__name__) for cls, _st in data]
        hover = self._hits.get("top_hover")
        hits = charts.bars(s, rect, values, labels=labels, colors=colors,
                           horizontal=True, fmt=stats.format_time, hover=hover)
        self._track(hits, "top", lambda idx: [
            data[idx][0].name,
            t("progress.col_time") + ": " + stats.format_time(values[idx]),
            t("progress.col_plays") + ": %d" % data[idx][1]["plays"]])

    def _draw_chart_split(self, s, y, rh, _data, _i):
        totals = self._totals
        wins, losses = totals["wins"], totals["losses"]
        cx = self._left + 80
        cy = y + rh // 2
        ratio = (100.0 * wins / (wins + losses)) if (wins + losses) else 0
        charts.donut(s, (cx, cy), 60,
                     [(wins, ui.GREEN),
                      (losses, ui.mix(ui.PANEL_LIGHT, ui.RED, 0.8))],
                     center_text="%d%%" % round(ratio),
                     center_sub=t("progress.win_ratio"))
        # Legende + Kennzahlen daneben.
        fnt = ui.font(12)
        tx = cx + 84
        rows = ((t("progress.total_wins"), str(wins)),
                (t("progress.total_losses"), str(losses)),
                (t("progress.total_records"), str(totals["records"])),
                (t("progress.longest_session"),
                 stats.format_time(totals.get("longest", 0))),
                (t("progress.duels"), str(totals.get("duels", 0))))
        ry = y + 14
        for label, value in rows:
            s.blit(fnt.render(label, True, ui.TEXT_DIM), (tx, ry))
            img = ui.font(12, bold=True).render(value, True, ui.TEXT)
            s.blit(img, img.get_rect(topright=(self._right - 8, ry)))
            ry += 24

    def _draw_table_head(self, s, y, rh, _data, _i):
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

    def _table_columns(self):
        """(x-Positionen der Zahlen-Spalten von rechts) - passt sich der Breite an."""
        w = self._right - self._left
        col_w = max(64, min(92, w // 7))
        return [self._right - 10 - i * col_w for i in range(4)], col_w

    def _draw_game(self, s, y, rh, data, i):
        cls, st = data
        row = pygame.Rect(self._left, y, self._right - self._left, rh)
        if i == self._hover_row:
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

    # -- Kalender-Reiter --------------------------------------------------

    def _draw_wrapped(self, s, y, rh, _data, _i):
        rect = pygame.Rect(self._left, y, self._right - self._left, rh)
        ui.draw_panel(s, rect, color=ui.PANEL_LIGHT, radius=12,
                      accent_top=ui.GOLD)
        head = ui.font(14, bold=True).render(t("progress.wrapped_head"), True,
                                             ui.GOLD)
        s.blit(head, (rect.x + 14, rect.y + 6))
        fnt = ui.font(12)
        ty = rect.y + 8 + head.get_height()
        for fact in self._facts:
            pygame.draw.circle(s, ui.ACCENT, (rect.x + 20, ty + 8), 3)
            img = fnt.render(fact, True, ui.TEXT_DIM)
            s.blit(img, (rect.x + 30, ty), (0, 0, rect.w - 44, img.get_height()))
            ty += 20
        # Export/Import der Statistik-Datei.
        self._draw_io_buttons(s, rect)

    def _draw_io_buttons(self, s, rect):
        import filepick
        if not filepick.available():
            return
        fnt = ui.font(11, bold=True)
        x = rect.right - 10
        for label, cb in ((t("progress.import"), self._import),
                          (t("progress.export"), self._export)):
            w = fnt.size(label)[0] + 20
            r = pygame.Rect(x - w, rect.y + 8, w, 22)
            pygame.draw.rect(s, ui.PANEL, r, border_radius=11)
            pygame.draw.rect(s, ui.BORDER_LIGHT, r, 1, border_radius=11)
            img = fnt.render(label, True, ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=r.center))
            self._buttons.append((r, cb))
            x -= w + 6

    def _draw_heat(self, s, y, rh, _data, _i):
        rect = pygame.Rect(self._left, y, self._right - self._left, rh - 20)
        hover = self._hits.get("heat_hover")
        hits = charts.calendar_heat(s, rect, self._day_time, color=ui.ACCENT,
                                    hover=hover)
        self._hits["heat"] = hits
        for box, (day, value) in hits:
            if box.collidepoint(self._mouse):
                self._hits["heat_hover"] = day
                entry = self._days.get(day, {})
                self._tip = [_pretty_date(day),
                             t("progress.tip_time",
                               time=stats.format_time(value)),
                             t("progress.tip_plays", n=entry.get("p", 0))]
                break
        charts.heat_legend(s, pygame.Rect(rect.x, rect.bottom + 4,
                                          rect.w, 14), color=ui.ACCENT,
                           label_less=t("progress.less"),
                           label_more=t("progress.more"))

    def _draw_streaks(self, s, y, rh, _data, _i):
        cards = ((str(self._streak["cur"]), t("progress.streak_cur"), "flame"),
                 (str(self._streak["best"]), t("progress.streak_best"), "flame"),
                 (str(self._totals.get("days", 0)), t("progress.total_days"),
                  "calendar"))
        gap = 10
        cw = (self._right - self._left - gap * (len(cards) - 1)) // len(cards)
        for i, (value, label, icon) in enumerate(cards):
            rect = pygame.Rect(self._left + i * (cw + gap), y, cw, rh)
            ui.draw_panel(s, rect, shadow=False, radius=10)
            achievements.draw_badge(s, (rect.x + 20, rect.centery), 11, icon,
                                    ui.GOLD if i == 1 else ui.ACCENT)
            img = ui.font(15, bold=True).render(value, True, ui.TEXT)
            s.blit(img, (rect.x + 38, rect.centery - img.get_height() + 2))
            lab = ui.font(10).render(label, True, ui.TEXT_DIM)
            s.blit(lab, (rect.x + 38, rect.centery + 2),
                   (0, 0, rect.w - 46, lab.get_height()))

    def _draw_weekdays(self, s, y, rh, _data, _i):
        rect = pygame.Rect(self._left, y, self._right - self._left, rh)
        values = stats.weekday_list()
        labels = charts.weekday_names()
        hover = self._hits.get("wd_hover")
        hits = charts.bars(s, rect, values, labels=labels, color=ui.ACCENT,
                           fmt=stats.format_time, hover=hover)
        self._track(hits, "wd", lambda idx: [
            labels[idx] if idx < len(labels) else "",
            stats.format_time(values[idx])])

    def _draw_clock(self, s, y, rh, _data, _i):
        hours = stats.hours_list()
        cx = self._left + (self._right - self._left) // 2
        cy = y + rh // 2
        hover = self._hits.get("clock_hover")
        hits = charts.clock_ring(s, (cx, cy), min(80, rh // 2 - 14), hours,
                                 color=ui.ACCENT, hover=hover)
        self._track(hits, "clock", lambda idx: [
            t("progress.tip_hour", h=idx, h2=(idx + 1) % 24),
            stats.format_time(hours[idx])])
        if any(hours):
            peak = hours.index(max(hours))
            img = ui.font(11).render(
                t("progress.fact_peak_hour", h=peak, h2=(peak + 1) % 24),
                True, ui.TEXT_DIM)
            s.blit(img, img.get_rect(midtop=(cx, y + rh - 16)))

    # -- Verlauf-Reiter ---------------------------------------------------

    def _draw_series_pick(self, s, y, rh, _data, _i):
        fnt = ui.font(11, bold=True)
        x = self._left
        label = ui.font(11).render(t("progress.pick_game") + ":", True,
                                   ui.TEXT_FAINT)
        s.blit(label, (x, y + 5))
        x += label.get_width() + 8
        for key in self._played_series:
            cls = self._game_cls(key)
            if cls is None:
                continue
            name = cls.name
            w = fnt.size(name)[0] + 16
            if x + w > self._right:
                break
            r = pygame.Rect(x, y, w, 22)
            active = (key == self._series_game)
            col = ui.game_color(cls.__name__)
            pygame.draw.rect(s, ui.PANEL_LIGHT if active else ui.PANEL, r,
                             border_radius=11)
            pygame.draw.rect(s, col if active else ui.BORDER, r, 1,
                             border_radius=11)
            img = fnt.render(name, True, ui.TEXT if active else ui.TEXT_DIM)
            s.blit(img, img.get_rect(center=r.center))
            self._buttons.append((r, lambda k=key: self._pick_series(k)))
            x += w + 6

    def _pick_series(self, key):
        self._series_game = key
        self._build()

    def _draw_chart_series(self, s, y, rh, _data, _i):
        key = self._series_game
        if not key:
            return
        series = stats.series_of(key)
        if not series:
            return
        cls = self._game_cls(key)
        color = ui.game_color(cls.__name__) if cls else ui.ACCENT
        rect = pygame.Rect(self._left, y + 4, self._right - self._left, rh - 8)
        values = [p[1] for p in series]
        marks = [i for i, p in enumerate(series) if p[2]]
        hover = self._hits.get("series_hover")
        hits = charts.line(s, rect, values, color=color, marks=marks,
                           fmt=lambda v: str(int(v)), hover=hover)
        self._track(hits, "series", lambda idx: [
            str(series[idx][1]),
            _pretty_date(series[idx][0][:10]) + " " + series[idx][0][11:16],
            t("progress.new_record") if series[idx][2] else ""])

    def _draw_event(self, s, y, rh, ev, _i):
        row = pygame.Rect(self._left, y, self._right - self._left, rh)
        kind = ev.get("k")
        stamp = str(ev.get("t", ""))[11:16]
        icon, color, text = "star", ui.ACCENT, ""
        if kind == "ach":
            d = achievements.by_id(ev.get("id", ""))
            if d is not None:
                icon = d["icon"]
                color = achievements.rarity_color(d)
                text = t("progress.ev_ach", name=achievements.display_name(d))
                extra = "+%d" % achievements.points_of(d)
            else:
                text = t("progress.ev_ach", name=str(ev.get("id", "")))
                extra = ""
        elif kind == "rec":
            cls = self._game_cls(ev.get("g", ""))
            icon, color = "trophy", ui.GOLD
            text = t("progress.ev_rec", game=cls.name if cls else ev.get("g"),
                     n=ev.get("v", 0))
            extra = ""
        elif kind == "lvl":
            icon = "bolt"
            color = levels.rank_color(int(ev.get("v", 1)))
            text = t("progress.ev_lvl", n=ev.get("v", 0),
                     rank=levels.rank_title(int(ev.get("v", 1))))
            extra = ""
        elif kind == "duel":
            cls = self._game_cls(ev.get("g", ""))
            icon, color = "swords", ui.ACCENT2
            other = ev.get("o", "")
            text = t("progress.ev_duel", game=cls.name if cls else ev.get("g"),
                     name=self._profiles.name_of(other) if other else "?",
                     result=t("progress.ev_won") if ev.get("w")
                     else t("progress.ev_lost"))
            extra = ""
        else:
            extra = ""
        achievements.draw_badge(s, (row.x + 14, row.centery), 10, icon, color)
        img = ui.font(12).render(text, True, ui.TEXT_DIM)
        s.blit(img, (row.x + 32, row.centery - img.get_height() // 2),
               (0, 0, row.w - 120, img.get_height()))
        if extra:
            pts = ui.font(11, bold=True).render(extra, True, color)
            s.blit(pts, pts.get_rect(midright=(row.right - 54, row.centery)))
        tm = ui.font(11).render(stamp, True, ui.TEXT_FAINT)
        s.blit(tm, tm.get_rect(midright=(row.right - 8, row.centery)))

    # -- Profile-Reiter ---------------------------------------------------

    def _draw_profile(self, s, y, rh, data, _i):
        pid, meta = data
        row = pygame.Rect(self._left, y, self._right - self._left, rh)
        active = (pid == self._profiles.active_id())
        ui.draw_panel(s, row, color=ui.PANEL_LIGHT if active else ui.PANEL,
                      shadow=False, radius=10)
        color = self._profiles.color_rgb(pid)
        achievements.draw_badge(s, (row.x + 28, row.centery), 17,
                                self._profiles.icon_of(pid), color)
        name = ui.font(15, bold=True).render(self._profiles.name_of(pid), True,
                                             ui.TEXT)
        s.blit(name, (row.x + 54, row.y + 8))
        tot = self._pstats.get(pid, {})
        level = tot.get("level", 1)
        sub = ui.font(11).render(
            "%s %d · %s · %s · %s" % (
                t("level.short"), level, levels.rank_title(level),
                t("progress.points", n=tot.get("points", 0)),
                stats.format_time(tot.get("time", 0))),
            True, ui.TEXT_DIM)
        s.blit(sub, (row.x + 54, row.y + 10 + name.get_height()),
               (0, 0, row.w - 70, sub.get_height()))
        if active:
            chip = ui.font(10, bold=True).render(t("profile.active"), True,
                                                 ui.GOLD)
            s.blit(chip, chip.get_rect(topright=(row.right - 12, row.y + 8)))

    def _draw_board(self, s, y, rh, data, i):
        cls, values = data
        row = pygame.Rect(self._left, y, self._right - self._left, rh)
        if i == self._hover_row:
            ui.draw_panel(s, row, color=ui.PANEL_LIGHT, shadow=False, radius=8)
        pygame.draw.circle(s, ui.game_color(cls.__name__),
                           (row.x + 12, row.centery), 4)
        name = ui.font(13, bold=True).render(cls.name, True, ui.TEXT)
        s.blit(name, (row.x + 24, row.centery - name.get_height() // 2),
               (0, 0, 150, name.get_height()))
        x = row.x + 190
        for rank_i, (pid, score) in enumerate(values[:3]):
            col = self._profiles.color_rgb(pid)
            label = "%s %s" % (self._profiles.name_of(pid), f"{score:,}".replace(",", " "))
            img = ui.font(11, bold=True if rank_i == 0 else False).render(
                label, True, ui.GOLD if rank_i == 0 else ui.TEXT_DIM)
            pygame.draw.circle(s, col, (x, row.centery), 3)
            s.blit(img, (x + 8, row.centery - img.get_height() // 2))
            x += img.get_width() + 24
            if x > row.right - 40:
                break

    def _draw_duel(self, s, y, rh, data, _i):
        cls, me, other, wins, losses = data
        row = pygame.Rect(self._left, y, self._right - self._left, rh)
        pygame.draw.circle(s, ui.game_color(cls.__name__),
                           (row.x + 12, row.centery), 4)
        name = ui.font(13, bold=True).render(cls.name, True, ui.TEXT)
        s.blit(name, (row.x + 24, row.centery - name.get_height() // 2),
               (0, 0, 150, name.get_height()))
        label = "%s  %d : %d  %s" % (self._profiles.name_of(me), wins, losses,
                                     self._profiles.name_of(other))
        col = ui.GOLD if wins > losses else ui.TEXT_DIM
        img = ui.font(12, bold=True).render(label, True, col)
        s.blit(img, img.get_rect(midright=(row.right - 10, row.centery)))

    # -- Hover/Tooltip-Verwaltung ----------------------------------------

    def _track(self, hits, name, tip_fn):
        """Merkt Trefferflächen eines Diagramms und baut den Tooltip."""
        self._hits[name] = hits
        for rect, idx in hits:
            if rect.collidepoint(self._mouse):
                self._hits[name + "_hover"] = idx
                lines = tip_fn(idx)
                if lines:
                    self._tip = lines
                return
        self._hits[name + "_hover"] = None


def _pretty_date(day):
    """'2026-09-19' -> '19.09.2026' bzw. 'heute'/'gestern'."""
    try:
        date = datetime.date.fromisoformat(str(day)[:10])
    except ValueError:
        return str(day)
    today = datetime.date.today()
    if date == today:
        return t("cal.today")
    if (today - date).days == 1:
        return t("cal.yesterday")
    return t("cal.date", d=date.day, m=date.month, y=date.year)
