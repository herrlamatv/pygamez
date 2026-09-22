# -*- coding: utf-8 -*-
"""
geodash_extra.py
================
Der Reiter **Weitere** von Geometry Dash: die Level-Bibliothek.

80 zusätzliche Level in zehn Kategorien (Einsteiger, Rhythmus, Münzen,
Schiff, UFO, Ball, Schwerkraft, Tempo, Welle, Dämon) - erzeugt und vom
Solver bewiesen von ``devtools/build_geodash_extra.py``, gespeichert in
``games/levels/geodash_extra.json``.

Oben eine Kategorie-Leiste, darunter ein Suchfeld, darunter die Liste. Jedes
Level lässt sich spielen oder **als Vorlage** in die eigenen Level kopieren -
von dort aus kann man es im Editor umbauen.

Fortschritt: Bestwert, Münzen und Versuche werden je Level unter dem
Schlüssel ``x:<id>`` gemerkt (siehe ``GeometryDashGame._best_key``). Sterne
und Erfolge bleiben der Kampagne vorbehalten, damit die Sternskala dort
aussagekräftig bleibt.
"""

import random

import pygame

import ugc
import ui
from game_base import InputEvent
from i18n import t

from . import geodash_core as core
from . import geodash_draw as gdraw
from .geodash_edit import GAME, _btn

# Reihenfolge der Kategorie-Knöpfe ("all" = alles zeigen).
CATS = ("all", "starter", "rhythm", "coins", "ship", "ufo", "ball",
        "gravity", "speed", "wave", "demon")

BUTTONS = ("play", "template", "random")


class ExtraList:
    """Die Level-Bibliothek: filtern, suchen, spielen, als Vorlage kopieren."""

    def __init__(self, game):
        self.game = game
        self.levels = []
        self.view = []
        self.cat = "all"
        self.query = ui.TextInput(maxlen=20, placeholder=t("gd.more.search"))
        self.search_focus = False
        self.sel = 0
        self.first = 0
        self.toast = ""
        self.toast_t = 0.0
        self._hay = []
        self.reload()
        self.layout()
        cat = game._gs().get("last_cat", "all")
        if cat in CATS:
            self.cat = cat
        self._filter()

    # ----- Daten --------------------------------------------------------
    def reload(self):
        from .geodash import load_extra_levels
        if not self.levels:
            self.levels = load_extra_levels()
            # Suchtext je Level einmal vorbereiten (Filtern kostet dann nichts).
            self._hay = [("%s %s %s" % (d.get("name", ""), d.get("id", ""),
                                        d.get("category", ""))).lower()
                         for d in self.levels]
        self._filter()

    def _filter(self):
        """Sichtbare Level nach Kategorie und Suchtext bestimmen."""
        q = self.query.text.strip().lower()
        self.view = []
        for i, d in enumerate(self.levels):
            if self.cat != "all" and d.get("category") != self.cat:
                continue
            if q and q not in self._hay[i]:
                continue
            self.view.append(i)
        self.sel = max(0, min(self.sel, max(0, len(self.view) - 1)))
        self._clamp()

    def selected(self):
        if not self.view:
            return None
        return self.levels[self.view[self.sel]]

    def has_next(self):
        return bool(self.view) and self.sel < len(self.view) - 1

    def play_next(self):
        if self.has_next():
            self.sel += 1
            self._clamp()
            self._action("play")

    def done_count(self):
        """Wie viele Level der Bibliothek sind schon geschafft?"""
        best = self.game.best
        return sum(1 for d in self.levels
                   if best.get("x:" + d.get("id", ""), [0, 0])[0] >= 100)

    # ----- Layout -------------------------------------------------------
    def layout(self):
        g = self.game
        w, h = g.width, g.height
        self.fnt = ui.font(max(13, h // 30))
        self.tiny = ui.font(max(10, h // 40))
        fh, th = self.fnt.get_height(), self.tiny.get_height()
        self.two_lines = h >= 400
        self.row_h = fh + th + 10 if self.two_lines else max(26, fh + 12)

        # Kategorie-Leiste (eine oder zwei Zeilen, je nach Breite)
        cols = 6 if w >= 720 else 4
        ch = max(20, min(28, h // 18), th + 8)
        gap = 5
        cw = (w - 24 - gap * (cols - 1)) / float(cols)
        rows_c = (len(CATS) + cols - 1) // cols
        self.cat_rects = {}
        top = g.tab_bottom + 6
        for i, key in enumerate(CATS):
            r, c = divmod(i, cols)
            self.cat_rects[key] = pygame.Rect(int(12 + c * (cw + gap)),
                                              int(top + r * (ch + gap)),
                                              int(cw), ch)
        cat_bottom = top + rows_c * (ch + gap)

        # Suchfeld
        ff = max(22, min(30, h // 15), th + 10)
        self.search_rect = pygame.Rect(12, int(cat_bottom), w - 24, ff)

        # Knopfleiste unten
        bh = max(24, min(34, h // 13), th + 12)
        bw = (w - 24 - gap * (len(BUTTONS) - 1)) / float(len(BUTTONS))
        bottom = h - 12
        self.btn_rects = {}
        for i, key in enumerate(BUTTONS):
            self.btn_rects[key] = pygame.Rect(int(12 + i * (bw + gap)),
                                              int(bottom - bh), int(bw), bh)
        self.list_top = self.search_rect.bottom + 6
        self.list_bottom = bottom - bh - th - 8
        self.rows_visible = max(1, int((self.list_bottom - self.list_top - 4)
                                       // self.row_h))
        self.list_w = w - 30
        self._clamp()

    def _clamp(self):
        n = len(self.view)
        vis = getattr(self, "rows_visible", 1)
        self.first = max(0, min(self.first, max(0, n - vis)))
        if self.sel < self.first:
            self.first = self.sel
        elif self.sel >= self.first + vis:
            self.first = self.sel - vis + 1

    def _toast(self, text):
        self.toast = text
        self.toast_t = 2.8

    def update(self, dt):
        if self.toast_t > 0:
            self.toast_t -= dt
            if self.toast_t <= 0:
                self.toast = ""

    # ----- Eingabe ------------------------------------------------------
    def handle(self, event):
        g = self.game
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if self.search_focus:
                if k in ("Return", "KP_Enter"):
                    self.search_focus = False
                    return True
                if k == "Escape":
                    self.query.set_text("")
                    self.search_focus = False
                    self._filter()
                    return True
                if self.query.handle(event):
                    self._filter()
                    return True
            if k in ("Up", "Left") or g.is_action(k, "up"):
                self._move(-1)
            elif k in ("Down", "Right") or g.is_action(k, "down"):
                self._move(1)
            elif k == "Prior":
                self._move(-self.rows_visible)
            elif k == "Next":
                self._move(self.rows_visible)
            elif k in ("Return", "space", "KP_Enter"):
                self._action("play")
            elif k in ("t", "T"):
                self._action("template")
            elif k in ("r", "R"):
                self._action("random")
            elif k in ("slash", "f", "F"):
                self.search_focus = True
            elif k in ("c", "C"):
                self._step_cat(1)
            return True
        if event.kind == InputEvent.WHEEL:
            self.first = max(0, min(max(0, len(self.view) - self.rows_visible),
                                    self.first - event.delta))
            return True
        if event.kind == InputEvent.MOUSEDOWN:
            pos = event.pos
            for key, rc in self.cat_rects.items():
                if rc.collidepoint(pos):
                    self._set_cat(key)
                    return True
            if self.search_rect.collidepoint(pos):
                self.search_focus = True
                return True
            self.search_focus = False
            for key, rc in self.btn_rects.items():
                if rc.collidepoint(pos):
                    self._action(key)
                    return True
            for i in range(self.rows_visible):
                idx = self.first + i
                if idx >= len(self.view):
                    break
                if self._row_rect(i).collidepoint(pos):
                    if self.sel == idx:
                        self._action("play")
                    else:
                        self.sel = idx
                        self.game.play_sound("move")
                    return True
            return True
        return False

    def _row_rect(self, i):
        return pygame.Rect(12, self.list_top + i * self.row_h, self.list_w,
                           self.row_h - 3)

    def _move(self, d):
        if not self.view:
            return
        self.sel = max(0, min(len(self.view) - 1, self.sel + d))
        self._clamp()
        self.game.play_sound("move")

    def _set_cat(self, key):
        if key not in CATS or key == self.cat:
            return
        self.cat = key
        self.sel = 0
        self.first = 0
        self._filter()
        self.game._save_setting("last_cat", key)
        self.game.play_sound("click")

    def _step_cat(self, d):
        self._set_cat(CATS[(CATS.index(self.cat) + d) % len(CATS)])

    def _action(self, key):
        g = self.game
        if key == "random":
            if self.view:
                self.sel = random.randrange(len(self.view))
                self._clamp()
                self._action("play")
            return
        d = self.selected()
        if d is None:
            return
        if key == "play":
            g.play_sound("click")
            g.start_level(d, "extra")
        elif key == "template":
            self._copy_as_template(d)

    def _copy_as_template(self, d):
        """Ein Level in die eigenen Level kopieren - dort baut man es um."""
        g = self.game
        if ugc.is_full(GAME):
            self._toast(t("gd.ugc.err.full", max=ugc.max_items(GAME)))
            return
        from .geodash_edit import new_level
        m = new_level(name=d.get("name", ""), level_id="")
        m["id"] = ugc.unique_id(ugc.slug(d.get("id", "level")), GAME)
        m["objects"] = [list(o) for o in d.get("objects", ())]
        for key in ("speed", "mode", "bg", "ground", "music", "length"):
            if key in d:
                m[key] = d[key]
        m["from"] = d.get("id", "")
        ok, _why = ugc.save_map(m, GAME)
        if not ok:
            self._toast(t("gd.ugc.err.full", max=ugc.max_items(GAME)))
            return
        g.play_sound("click")
        if g.lists is not None:
            g.lists.reload()
        g.ugc_edit(m)

    # ----- Zeichnen -----------------------------------------------------
    def draw(self, s):
        g = self.game
        for key, rc in self.cat_rects.items():
            _btn(s, rc, t("gd.more.cat." + key), self.tiny, on=(key == self.cat))
        self.query.draw(s, self.search_rect, self.fnt, focused=self.search_focus)
        if not self.view:
            self._draw_empty(s)
        else:
            for i in range(self.rows_visible):
                idx = self.first + i
                if idx >= len(self.view):
                    break
                self._draw_row(s, self.levels[self.view[idx]], idx,
                               self._row_rect(i))
            if len(self.view) > self.rows_visible:
                self._draw_scrollbar(s)
        cnt = self.tiny.render(
            t("gd.more.count", n=len(self.view), max=len(self.levels))
            + "   ·   " + t("gd.more.done", n=self.done_count(),
                            max=len(self.levels)), True, ui.TEXT_DIM)
        s.blit(cnt, (12, self.list_bottom + 2))
        has = self.selected() is not None
        for key, rc in self.btn_rects.items():
            _btn(s, rc, t("gd.more.btn_" + key), self.tiny, enabled=has)
        if self.toast:
            img = self.tiny.render(self.toast, True, ui.TEXT)
            w, h = img.get_width() + 26, img.get_height() + 12
            r = pygame.Rect(g.width // 2 - w // 2, self.list_bottom - h - 2, w, h)
            ui.draw_panel(s, r, radius=h // 2, shadow=False)
            pygame.draw.rect(s, ui.ACCENT2, r, 1, border_radius=h // 2)
            s.blit(img, img.get_rect(center=r.center))

    def _draw_empty(self, s):
        g = self.game
        box = pygame.Rect(24, self.list_top + 10, g.width - 48,
                          max(50, self.list_bottom - self.list_top - 24))
        ui.draw_panel(s, box, shadow=False)
        img = self.fnt.render(t("gd.more.empty"), True, ui.TEXT_DIM)
        if img.get_width() > box.w - 16:
            img = self.tiny.render(t("gd.more.empty"), True, ui.TEXT_DIM)
        s.blit(img, img.get_rect(center=box.center))

    def _draw_row(self, s, d, idx, rc):
        g = self.game
        sel = (idx == self.sel)
        pygame.draw.rect(s, ui.PANEL_LIGHT if sel else ui.PANEL, rc, border_radius=7)
        pygame.draw.rect(s, g.accent if sel else ui.BORDER, rc, 2 if sel else 1,
                         border_radius=7)
        pad = 10
        diff = int(d.get("difficulty", 0))
        col = gdraw.DIFF_COLORS[max(0, min(len(gdraw.DIFF_COLORS) - 1, diff))]
        pygame.draw.rect(s, col, (rc.x + 5, rc.y + 5, 5, rc.h - 10), border_radius=2)
        x = rc.x + pad + 6
        name = self.fnt.render(d.get("name", ""), True, ui.TEXT)
        s.blit(name, (x, rc.y + 3))
        key = "x:" + d.get("id", "")
        best = g.best.get(key, [0, 0])[0]
        mask = g.coins.get(key, 0)
        sub = "%s  ·  %s  ·  %s" % (
            t("gd.more.cat." + d.get("category", "all")),
            t("gd.diff." + DIFF_KEY(diff)),
            t("gd.ugc.length", n=core.level_length(d)))
        sub += "  ·  " + t("gd.coins_of", n=bin(mask & 7).count("1"), max=3)
        img = self.tiny.render(sub, True, ui.TEXT_DIM)
        if img.get_width() > rc.w - 90:
            kurz = sub
            while len(kurz) > 4 and self.tiny.size(kurz + "...")[0] > rc.w - 90:
                kurz = kurz[:-1]
            img = self.tiny.render(kurz + "...", True, ui.TEXT_DIM)
        if self.two_lines:
            s.blit(img, (x, rc.bottom - img.get_height() - 3))
        pct = self.fnt.render("%d%%" % best, True,
                              gdraw.COL_P1 if best >= 100 else ui.TEXT_DIM)
        s.blit(pct, pct.get_rect(midright=(rc.right - pad, rc.centery)))

    def _draw_scrollbar(self, s):
        track = pygame.Rect(self.list_w + 14, self.list_top, 4,
                            self.rows_visible * self.row_h)
        pygame.draw.rect(s, ui.PANEL, track, border_radius=2)
        frac = self.rows_visible / float(len(self.view))
        h = max(24, int(track.h * frac))
        pos = self.first / float(max(1, len(self.view) - self.rows_visible))
        y = track.y + int((track.h - h) * min(1.0, pos))
        pygame.draw.rect(s, ui.BORDER_LIGHT, (track.x, y, 4, h), border_radius=2)


def DIFF_KEY(diff):
    """Schwierigkeits-Schlüssel (easy ... demon) zur Zahl."""
    from .geodash import DIFF_KEYS
    return DIFF_KEYS[max(0, min(len(DIFF_KEYS) - 1, int(diff)))]
