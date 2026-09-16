# -*- coding: utf-8 -*-
"""
geodash_edit.py
===============
Eigene Geometry-Dash-Level: der LEVELS-Reiter und der Level-Editor.

Beides hängt an ``GeometryDashGame`` (geodash.py) und zeichnet auf dessen
Fläche - Editor und Spiel teilen sich dieselbe Instanz, damit das Test-
Spielen direkt zurück ans Bauen führt (Vorbild: minigolf_edit.py).

    LevelList    Liste der eigenen Level: Neu, Bearbeiten, Spielen, Löschen,
                 Teilen (Export als .lamapgzlevel) und Importieren.
    LevelEditor  Raster-Leinwand mit Scrollen, Palette in sechs Gruppen
                 (Blöcke, Gefahren, Pads & Orbs, Portale, Tempo, Extras),
                 Auswählen/Setzen/Löschen/Drehen, Undo/Redo, Level-
                 Einstellungen (Tempo, Startform, Farben, Musik), Test-Spielen
                 ab Start oder ab der Kamera, Übersichtsleiste.

"Verifiziert" wird ein Level erst, wenn sein Ersteller es im Test vom Start
weg im Normalmodus geschafft hat. Gespeichert wird dazu die Prüfsumme des
Inhalts (``geodash_core.content_hash``) - jede spätere Änderung am Level
nimmt das Abzeichen wieder weg.

Gespeichert wird über ``ugc.py`` (Art "geodash"), gezeichnet über
``geodash_draw.py`` - dieselben Funktionen wie im Spiel.
"""

import copy
import math
import os
import time

import pygame

import filepick
import swear
import ugc
import ui
from game_base import InputEvent
from i18n import t

from . import geodash_core as core
from . import geodash_draw as gdraw

GAME = "geodash"

# ---------------------------------------------------------------------------
#  Palette
# ---------------------------------------------------------------------------

TOOLS = ("select", "erase")
GROUPS = (
    ("blocks", ("block", "half")),
    ("hazards", ("spike", "spike_s", "pit")),
    ("orbs", ("pad_y", "pad_p", "pad_b", "orb_y", "orb_p", "orb_b")),
    ("portals", ("p_cube", "p_ship", "p_ball", "p_ufo", "p_wave", "g_norm",
                 "g_flip")),
    ("speed", ("s_slow", "s_norm", "s_fast", "s_vfast")),
    ("extras", ("coin", "color")),
)
GROUP_KEYS = tuple(g[0] for g in GROUPS)
GROUP_ICON = {"blocks": "block", "hazards": "spike", "orbs": "orb_y",
              "portals": "p_ship", "speed": "s_fast", "extras": "coin"}

# Farbvorlagen für Hintergrund, Boden und Farb-Trigger
PRESETS = ((40, 110, 255), (20, 70, 200), (170, 60, 255), (255, 60, 150),
           (230, 90, 40), (250, 200, 40), (40, 170, 110), (0, 170, 200),
           (140, 0, 40), (30, 30, 50), (120, 120, 140), (255, 255, 255))

# Beschriftung der Zusatzwerte (Parameterzeile)
PARAM_LABELS = {
    "p_ship": ("gd.ed.par.height",), "p_ball": ("gd.ed.par.height",),
    "p_ufo": ("gd.ed.par.height",), "p_wave": ("gd.ed.par.height",),
    "color": ("gd.ed.par.target", "R", "G", "B", "gd.ed.par.dur"),
}

UNDO_MAX = 60
LAYER_PIT, LAYER_TRIGGER, LAYER_MAIN = 1, 2, 0


def layer_of(kind):
    if kind == "pit":
        return LAYER_PIT
    if kind == "color":
        return LAYER_TRIGGER
    return LAYER_MAIN


def new_level(name="", level_id="", author=None):
    """Frisches, leeres Level (Kopfdaten wie bei den Minigolf-Maps)."""
    now = time.strftime("%Y-%m-%d %H:%M")
    return core.normalize_level({
        "v": 1, "id": level_id, "name": name,
        "author": ugc.last_author() if author is None else author,
        "created": now, "edited": now, "speed": 1, "mode": core.CUBE,
        "bg": list(core.DEFAULT_BG), "ground": list(core.DEFAULT_GROUND),
        "music": "drive", "objects": []})


def is_verified(m):
    """Trägt das Level ein gültiges Verifiziert-Abzeichen?"""
    v = m.get("verified") if isinstance(m, dict) else None
    return isinstance(v, str) and bool(v) and v == core.content_hash(m)


def default_param(kind):
    return [p[0] for p in core.PARAMS.get(kind, ())]


# ---------------------------------------------------------------------------
#  Zeichenhelfer
# ---------------------------------------------------------------------------

def _btn(s, rc, text, fnt, on=False, accent=None, enabled=True):
    col_bg = ui.BTN_SEL if on else ui.BTN
    pygame.draw.rect(s, col_bg, rc, border_radius=7)
    pygame.draw.rect(s, (accent or ui.ACCENT) if on else ui.BORDER, rc,
                     2 if on else 1, border_radius=7)
    col = ui.TEXT if (enabled or on) else ui.TEXT_FAINT
    img = fnt.render(text, True, col)
    if img.get_width() > rc.w - 8:
        kurz = text
        while len(kurz) > 1 and fnt.size(kurz + "...")[0] > rc.w - 8:
            kurz = kurz[:-1]
        img = fnt.render(kurz + "...", True, col)
    s.blit(img, img.get_rect(center=rc.center))
    return rc


def _icon(s, rc, key, col):
    """Sinnbilder der Werkzeugleiste - gezeichnet (die Schrift hat keine)."""
    cx, cy = rc.center
    r = max(4, min(rc.w, rc.h) // 4)
    lw = 2
    if key in ("undo", "redo"):
        sign = -1 if key == "undo" else 1
        box = pygame.Rect(0, 0, r * 2, r * 2)
        box.center = (cx, cy + r // 3)
        pygame.draw.arc(s, col, box, 0.3, 2.9, lw)
        tip = (cx - sign * r, cy + r // 3)
        pygame.draw.polygon(s, col, [(tip[0] - r * 0.5, tip[1] - 1),
                                     (tip[0] + r * 0.5, tip[1] - 1),
                                     (tip[0], tip[1] + r * 0.6)])
    elif key == "rotate":
        box = pygame.Rect(0, 0, r * 2, r * 2)
        box.center = (cx, cy)
        pygame.draw.arc(s, col, box, -0.4, 4.2, lw)
        pygame.draw.polygon(s, col, [(cx + r, cy - r * 0.2), (cx + r * 0.3, cy - r * 0.2),
                                     (cx + r * 0.9, cy + r * 0.6)])
    elif key == "delete":
        pygame.draw.line(s, col, (cx - r, cy - r), (cx + r, cy + r), lw + 1)
        pygame.draw.line(s, col, (cx + r, cy - r), (cx - r, cy + r), lw + 1)
    elif key == "grid":
        box = pygame.Rect(0, 0, r * 2, r * 2)
        box.center = (cx, cy)
        pygame.draw.rect(s, col, box, 1)
        pygame.draw.line(s, col, (box.centerx, box.top), (box.centerx, box.bottom))
        pygame.draw.line(s, col, (box.left, box.centery), (box.right, box.centery))
    elif key == "settings":
        pygame.draw.circle(s, col, (cx, cy), r, lw)
        for k in range(6):
            a = k * math.pi / 3
            pygame.draw.line(s, col, (cx + math.cos(a) * r, cy + math.sin(a) * r),
                             (cx + math.cos(a) * (r + r * 0.55), cy + math.sin(a) * (r + r * 0.55)), lw + 1)
        pygame.draw.circle(s, col, (cx, cy), max(1, r // 3))
    elif key == "select":
        pygame.draw.polygon(s, col, [(cx - r, cy - r), (cx + r, cy), (cx, cy + r * 0.3),
                                     (cx + r * 0.3, cy + r)], 2)
    elif key == "erase":
        box = pygame.Rect(0, 0, r * 2, int(r * 1.5))
        box.center = (cx, cy)
        pygame.draw.rect(s, col, box, 2, border_radius=2)
        pygame.draw.line(s, col, (box.left, box.bottom), (box.right, box.top), 2)
    elif key == "test":
        pygame.draw.polygon(s, col, [(cx - r * 0.7, cy - r), (cx + r, cy), (cx - r * 0.7, cy + r)])
    elif key == "test_here":
        pygame.draw.polygon(s, col, [(cx - r * 0.2, cy - r), (cx + r * 1.2, cy), (cx - r * 0.2, cy + r)])
        pygame.draw.line(s, col, (cx - r, cy - r), (cx - r, cy + r), 2)


def _check_badge(s, cx, cy, r):
    """Grünes Verifiziert-Häkchen."""
    pygame.draw.circle(s, (60, 200, 110), (int(cx), int(cy)), int(r))
    pygame.draw.lines(s, (255, 255, 255), False,
                      [(cx - r * 0.5, cy), (cx - r * 0.1, cy + r * 0.4), (cx + r * 0.55, cy - r * 0.45)],
                      max(2, int(r // 3)))


# ---------------------------------------------------------------------------
#  LEVELS-Reiter
# ---------------------------------------------------------------------------

LIST_BUTTONS = ("new", "edit", "play", "delete", "share", "import")
NEEDS_SEL = ("edit", "play", "delete", "share")


class LevelList:
    """Der LEVELS-Reiter: eigene Level auswählen, bauen, spielen, teilen."""

    def __init__(self, game):
        self.game = game
        self.items = []
        self.sel = 0
        self.first = 0
        self.toast = ""
        self.toast_t = 0.0
        self.confirm = ""
        self.share = None
        self.f_author = ui.TextInput(maxlen=ugc.MAX_AUTHOR)
        self.f_file = ui.TextInput(maxlen=ugc.MAX_ID, charset=ui.TextInput.ID_CHARS)
        self.focus = 0
        self.err = ""
        self.reload()
        self.layout()

    def reload(self):
        self.items = ugc.load_maps(GAME)
        self.sel = max(0, min(self.sel, len(self.items) - 1))
        self._clamp()

    def selected(self):
        return self.items[self.sel] if self.items else None

    def layout(self):
        g = self.game
        w, h = g.width, g.height
        self.fnt = ui.font(max(13, h // 30))
        self.tiny = ui.font(max(10, h // 40))
        fh, th = self.fnt.get_height(), self.tiny.get_height()
        # Zeile: Name, darunter id/Objekte/Länge/Ersteller - bei sehr kleiner
        # Auflösung nur der Name. Alle Höhen wachsen mit der Schrift.
        self.two_lines = h >= 400
        self.row_h = fh + th + 10 if self.two_lines else max(26, fh + 12)
        cols = 6 if w >= 720 else 3
        bh = max(24, min(34, h // 13), th + 12)
        gap = 6
        rows = (len(LIST_BUTTONS) + cols - 1) // cols
        bw = (w - 24 - gap * (cols - 1)) / float(cols)
        bottom = h - 12
        self.btn_rects = {}
        for i, key in enumerate(LIST_BUTTONS):
            r, c = divmod(i, cols)
            y = bottom - (rows - r) * (bh + gap) + gap
            self.btn_rects[key] = pygame.Rect(int(12 + c * (bw + gap)), int(y), int(bw), bh)
        self.list_top = g.tab_bottom + 8
        # zwischen Liste und Knöpfen steht der Zähler "4/60 Level"
        self.list_bottom = bottom - rows * (bh + gap) - th - 6
        self.rows_visible = max(1, int((self.list_bottom - self.list_top - 4) // self.row_h))
        self.list_w = w - 30
        self._clamp()
        pw = min(w - 40, max(380, int(w * 0.5)))      # wächst mit der Schrift
        ff = max(22, min(30, h // 14), th + 10)          # Feld-/Knopfhöhe
        head = fh + 18                                    # Titel
        lab = th + 4                                      # Beschriftung über einem Feld
        ph = min(h - 20, head + 2 * lab + 4 * ff + 30 + th + 14)
        self.share_rect = pygame.Rect((w - pw) // 2, (h - ph) // 2, pw, ph)
        fx = self.share_rect.x + 18
        fw = pw - 36
        fh = ff
        self.author_rect = pygame.Rect(fx, self.share_rect.y + head + lab, fw, fh)
        self.file_rect = pygame.Rect(fx, self.author_rect.bottom + 8 + lab, fw, fh)
        sy = self.file_rect.bottom + 16
        sbw = (fw - 10) / 2.0
        self.share_btn = {"as": pygame.Rect(fx, sy, int(sbw), fh),
                          "dl": pygame.Rect(int(fx + sbw + 10), sy, int(sbw), fh),
                          "cancel": pygame.Rect(fx, sy + fh + 6, fw, fh)}

    def _clamp(self):
        n = len(self.items)
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
        if self.share is not None:
            return self._handle_share(event)
        g = self.game
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("Up", "Left") or g.is_action(k, "up"):
                self._move(-1)
            elif k in ("Down", "Right") or g.is_action(k, "down"):
                self._move(1)
            elif k in ("Return", "space"):
                self._action("play" if self.items else "new")
            elif k in ("Delete", "BackSpace"):
                self._action("delete")
            elif k in ("n", "N"):
                self._action("new")
            elif k in ("e", "E"):
                self._action("edit")
            return True
        if event.kind == InputEvent.WHEEL:
            self.first = max(0, min(max(0, len(self.items) - self.rows_visible),
                                    self.first - event.delta))
            return True
        if event.kind == InputEvent.MOUSEDOWN:
            for key, rc in self.btn_rects.items():
                if rc.collidepoint(event.pos):
                    self._action(key)
                    return True
            for i in range(self.rows_visible):
                idx = self.first + i
                if idx >= len(self.items):
                    break
                if self._row_rect(i).collidepoint(event.pos):
                    if self.sel != idx:
                        self.confirm = ""
                    self.sel = idx
                    g.play_sound("move")
                    return True
            return True
        return False

    def _row_rect(self, i):
        return pygame.Rect(12, self.list_top + i * self.row_h, self.list_w, self.row_h - 3)

    def _move(self, d):
        if not self.items:
            return
        self.sel = (self.sel + d) % len(self.items)
        self.confirm = ""
        self._clamp()
        self.game.play_sound("move")

    def _action(self, key):
        g = self.game
        if key != "delete":
            self.confirm = ""
        if key == "new":
            if ugc.is_full(GAME):
                self._toast(t("gd.ugc.err.full", max=ugc.max_items(GAME)))
                return
            g.ugc_new_level()
            return
        if key == "import":
            self._do_import()
            return
        m = self.selected()
        if m is None:
            return
        if key == "edit":
            g.ugc_edit(m)
        elif key == "play":
            g.ugc_play(m)
        elif key == "share":
            self._open_share(m)
        elif key == "delete":
            if self.confirm != m["id"]:
                self.confirm = m["id"]
                g.play_sound("select")
                return
            ugc.delete_map(m["id"], GAME)
            self.confirm = ""
            self.reload()
            self._toast(t("gd.ugc.deleted"))
            g.play_sound("click")

    # ----- Teilen -------------------------------------------------------
    def _open_share(self, m):
        self.share = m
        self.err = ""
        self.focus = 0
        self.f_author.set_text(m.get("author") or ugc.last_author())
        self.f_file.set_text(m.get("id") or "level")
        self.game.play_sound("click")

    def _close_share(self):
        self.share = None
        self.err = ""

    def _handle_share(self, event):
        if event.kind == InputEvent.MOUSEDOWN:
            if self.author_rect.collidepoint(event.pos):
                self.focus = 0
            elif self.file_rect.collidepoint(event.pos):
                self.focus = 1
            elif self.share_btn["as"].collidepoint(event.pos):
                self._do_export(ask=True)
            elif self.share_btn["dl"].collidepoint(event.pos):
                self._do_export(ask=False)
            elif self.share_btn["cancel"].collidepoint(event.pos) or \
                    not self.share_rect.collidepoint(event.pos):
                self._close_share()
            return True
        if event.kind != InputEvent.KEYDOWN:
            return True
        field = self.f_author if self.focus == 0 else self.f_file
        if field.handle(event):
            self.err = ""
            return True
        if event.key in ("Tab", "ISO_Left_Tab", "Up", "Down"):
            self.focus = 1 - self.focus
        elif event.key == "Escape":
            self._close_share()
        elif event.key in ("Return", "KP_Enter"):
            self._do_export(ask=False)
        return True

    def _do_export(self, ask):
        m = self.share
        if m is None:
            return
        author = self.f_author.text.strip()
        name = self.f_file.text.strip() or (m.get("id") or "level")
        if author and not swear.is_clean(author):
            self.err = t("gd.ugc.err.swear")
            self.game.play_sound("hit")
            return
        out = dict(m)
        if author:
            out["author"] = author
            ugc.set_last_author(author)
        filename = name + ugc.ext(GAME)
        if ask:
            if not filepick.available():
                self.err = t("gd.ugc.err.nofile")
                return
            path = filepick.save_as(filename, t("gd.ugc.share_title"),
                                    exts=ugc.import_exts(GAME))
            if not path:
                return
        else:
            path = filepick.to_downloads(filename)
        ok, why = ugc.export_to(out, path, GAME)
        if not ok:
            self.err = t("gd.ugc.err." + ("swear" if why == "swear" else "io"))
            self.game.play_sound("hit")
            return
        if author and m.get("author") != author:
            ugc.save_map(out, GAME)
            self.reload()
        self._close_share()
        self._toast(t("gd.ugc.exported", path=filepick.short(path)))
        self.game.play_sound("point")

    def _do_import(self):
        if ugc.is_full(GAME):
            self._toast(t("gd.ugc.err.full", max=ugc.max_items(GAME)))
            return
        if not filepick.available():
            self._toast(t("gd.ugc.err.nofile"))
            return
        path = filepick.open_file(t("gd.ugc.btn_import"), exts=ugc.import_exts(GAME))
        if not path:
            return
        self.import_path(path)

    def import_path(self, path):
        """Importiert eine Datei (auch vom Test benutzt)."""
        wanted = ugc.slug(os.path.splitext(os.path.basename(path))[0])
        ok, why, m = ugc.import_from(path, GAME)
        if not ok:
            known = ("swear", "full", "format", "io")
            self._toast(t("gd.ugc.err." + (why if why in known else "format"),
                          max=ugc.max_items(GAME)))
            self.game.play_sound("hit")
            return False
        self.reload()
        for i, x in enumerate(self.items):
            if x["id"] == m["id"]:
                self.sel = i
                self._clamp()
                break
        if wanted and m["id"] != wanted:
            self._toast(t("gd.ugc.renamed", id=m["id"]))
        else:
            self._toast(t("gd.ugc.imported", name=m["name"]))
        self.game.play_sound("point")
        return True

    # ----- Zeichnen -----------------------------------------------------
    def draw(self, s):
        if not self.items:
            self._draw_empty(s)
        else:
            for i in range(self.rows_visible):
                idx = self.first + i
                if idx >= len(self.items):
                    break
                self._draw_row(s, self.items[idx], idx, self._row_rect(i))
            if len(self.items) > self.rows_visible:
                self._draw_scrollbar(s)
        cnt = self.tiny.render(t("gd.ugc.count", n=len(self.items), max=ugc.max_items(GAME)),
                               True, ui.TEXT_DIM)
        s.blit(cnt, (12, self.list_bottom + 2))
        has = self.selected() is not None
        for key, rc in self.btn_rects.items():
            _btn(s, rc, t("gd.ugc.btn_" + key), self.tiny, enabled=has or key not in NEEDS_SEL)
        if self.toast:
            img = self.tiny.render(self.toast, True, ui.TEXT)
            w, h = img.get_width() + 26, img.get_height() + 12
            r = pygame.Rect(self.game.width // 2 - w // 2, self.list_bottom - h - 2, w, h)
            ui.draw_panel(s, r, radius=h // 2, shadow=False)
            pygame.draw.rect(s, ui.ACCENT2, r, 1, border_radius=h // 2)
            s.blit(img, img.get_rect(center=r.center))
        if self.share is not None:
            self._draw_share(s)

    def _draw_empty(self, s):
        g = self.game
        box = pygame.Rect(24, self.list_top + 10, g.width - 48,
                          max(60, self.list_bottom - self.list_top - 24))
        ui.draw_panel(s, box, shadow=False)
        for i, line in enumerate((t("gd.ugc.empty"), t("gd.ugc.empty2"))):
            img = self.fnt.render(line, True, ui.TEXT_DIM if i == 0 else ui.TEXT_FAINT)
            if img.get_width() > box.w - 16:
                img = self.tiny.render(line, True, ui.TEXT_DIM if i == 0 else ui.TEXT_FAINT)
            step = self.fnt.get_height() + 8
            s.blit(img, img.get_rect(center=(box.centerx, box.centery - step // 2 + i * step)))

    def _draw_row(self, s, m, idx, rc):
        g = self.game
        sel = (idx == self.sel)
        pygame.draw.rect(s, ui.PANEL_LIGHT if sel else ui.PANEL, rc, border_radius=7)
        pygame.draw.rect(s, g.accent if sel else ui.BORDER, rc, 2 if sel else 1, border_radius=7)
        pad = 10
        # Farbstreifen des Levels
        bg = tuple(m.get("bg", core.DEFAULT_BG))
        pygame.draw.rect(s, bg, (rc.x + 5, rc.y + 5, 5, rc.h - 10), border_radius=2)
        x = rc.x + pad + 6
        verified = is_verified(m)
        name = self.fnt.render(m.get("name", ""), True, ui.TEXT)
        s.blit(name, (x, rc.y + 3))
        if verified:
            br = max(5, name.get_height() // 3)
            _check_badge(s, x + name.get_width() + br + 6, rc.y + 3 + name.get_height() // 2, br)
        lv_len = core.level_length(m)
        sub = "%s  ·  %s  ·  %s" % (m.get("id", ""),
                                    t("gd.ugc.objects", n=len(m.get("objects", []))),
                                    t("gd.ugc.length", n=lv_len))
        if m.get("author"):
            sub += "  ·  " + t("gd.ugc.by", name=m["author"])
        if not verified:
            sub += "  ·  " + t("gd.ugc.unverified")
        img = self.tiny.render(sub, True, ui.TEXT_DIM)
        if img.get_width() > rc.w - 90:
            kurz = sub
            while len(kurz) > 4 and self.tiny.size(kurz + "...")[0] > rc.w - 90:
                kurz = kurz[:-1]
            img = self.tiny.render(kurz + "...", True, ui.TEXT_DIM)
        if self.two_lines:
            s.blit(img, (x, rc.bottom - img.get_height() - 3))
        best = g.best.get("ugc:" + m.get("id", ""), [0, 0])[0]
        if self.confirm == m.get("id"):
            warn = self.tiny.render(t("gd.ugc.confirm_delete"), True, ui.RED)
            s.blit(warn, warn.get_rect(midright=(rc.right - pad, rc.centery)))
        else:
            pct = self.fnt.render("%d%%" % best, True, gdraw.COL_P1 if best >= 100 else ui.TEXT_DIM)
            s.blit(pct, pct.get_rect(midright=(rc.right - pad, rc.centery)))

    def _draw_scrollbar(self, s):
        track = pygame.Rect(self.list_w + 14, self.list_top, 4, self.rows_visible * self.row_h)
        pygame.draw.rect(s, ui.PANEL, track, border_radius=2)
        frac = self.rows_visible / float(len(self.items))
        h = max(24, int(track.h * frac))
        pos = self.first / float(max(1, len(self.items) - self.rows_visible))
        y = track.y + int((track.h - h) * min(1.0, pos))
        pygame.draw.rect(s, ui.BORDER_LIGHT, (track.x, y, 4, h), border_radius=2)

    def _draw_share(self, s):
        g = self.game
        veil = pygame.Surface((g.width, g.height), pygame.SRCALPHA)
        veil.fill((0, 0, 0, 150))
        s.blit(veil, (0, 0))
        r = self.share_rect
        ui.draw_panel(s, r, accent_top=g.accent)
        head = self.fnt.render(t("gd.ugc.share_title"), True, g.accent)
        s.blit(head, head.get_rect(midtop=(r.centerx, r.y + 10)))
        for i, (rect, key, field) in enumerate((
                (self.author_rect, "gd.ugc.creator", self.f_author),
                (self.file_rect, "gd.ugc.filename", self.f_file))):
            lab = self.tiny.render(t(key), True, ui.TEXT_DIM)
            s.blit(lab, (rect.x, rect.y - lab.get_height() - 2))
            field.draw(s, rect, self.tiny, focused=(self.focus == i))
        ext = self.tiny.render(ugc.ext(GAME), True, ui.TEXT_FAINT)
        s.blit(ext, (self.file_rect.right - ext.get_width() - 8,
                     self.file_rect.centery - ext.get_height() // 2))
        _btn(s, self.share_btn["as"], t("gd.ugc.export_as"), self.tiny)
        _btn(s, self.share_btn["dl"], t("gd.ugc.export_dl"), self.tiny)
        _btn(s, self.share_btn["cancel"], t("gd.ugc.btn_cancel"), self.tiny)
        if self.err:
            img = self.tiny.render(self.err, True, ui.RED)
            s.blit(img, img.get_rect(midbottom=(r.centerx, r.bottom - 6)))


# ---------------------------------------------------------------------------
#  Der Level-Editor
# ---------------------------------------------------------------------------

TEXT_BUTTONS = ("test", "test_here")
ICON_BUTTONS = ("undo", "redo", "rotate", "delete", "grid", "settings")
ROWS_VISIBLE = 11            # Reihen auf der Leinwand (-1 .. 9)
SETTINGS_ROWS = ("speed", "mode", "music", "bpm", "bg", "ground")


class LevelEditor:
    """Level bauen: Leinwand links, Palette rechts, Übersicht unten."""

    def __init__(self, game, m):
        self.game = game
        self.meta = {k: v for k, v in dict(m).items() if k != "objects"}
        base = core.normalize_level(m)
        self.objects = [list(o) for o in base["objects"]]
        for key in ("speed", "mode", "bg", "ground", "music"):
            self.meta[key] = base[key]
        self.meta.setdefault("bpm", 0)
        self.tool = "block"
        self.group = "blocks"
        self.rot = 0
        self.sel = None              # Index in self.objects
        self.drag = None             # ("paint"/"erase"/"move"/"pan", ...)
        self.undo_stack = []
        self.redo_stack = []
        self.dirty = False
        self.err = ""
        self.toast = ""
        self.toast_t = 0.0
        self.confirm_back = False
        self.settings_open = False
        self.t = 0.0
        self.cam_x = -2.0
        self.cam_y = -1.0
        self.hover = None
        self.grid = bool(game.grid_snap)
        self.verified = m.get("verified") if is_verified(m) else ""
        self.f_name = ui.TextInput(m.get("name", ""), maxlen=ugc.MAX_NAME,
                                   placeholder=t("gd.ugc.name"))
        self.f_id = ui.TextInput(m.get("id", ""), maxlen=ugc.MAX_ID,
                                 charset=ui.TextInput.ID_CHARS, placeholder=t("gd.ugc.id"))
        self.focus = -1
        self.orig_id = m.get("id", "")
        self.renderer = gdraw.Renderer()
        self.icons = gdraw.Renderer()
        self._lv = None
        self.layout()

    # ----- Level-Daten ----------------------------------------------------
    def level_dict(self):
        d = dict(self.meta)
        d["objects"] = [list(o) for o in self.objects]
        d = core.normalize_level(d)
        if self.verified and self.verified == core.content_hash(d):
            d["verified"] = self.verified
        else:
            d.pop("verified", None)
        return d

    def compiled(self):
        if self._lv is None:
            self._lv = core.Level(self.level_dict())
            gdraw.block_masks(self._lv)
        return self._lv

    def _changed(self):
        self._lv = None
        self.dirty = True
        self.err = ""
        self.confirm_back = False

    def mark_verified(self):
        """Vom Spiel gerufen: der Ersteller hat sein Level geschafft."""
        d = self.level_dict()
        self.verified = core.content_hash(d)
        self._lv = None
        if not self.dirty and ugc.valid_id(self.orig_id) and ugc.get(self.orig_id, GAME):
            m = self.level_dict()
            m["id"] = self.orig_id
            m["name"] = self.f_name.text.strip() or self.meta.get("name", "")
            m["verified"] = self.verified
            ugc.save_map(m, GAME)
        self._toast(t("gd.ed.verified"))

    # ----- Layout -------------------------------------------------------
    def layout(self):
        g = self.game
        w, h = g.width, g.height
        self.tiny = ui.font(max(10, h // 42))
        self.small = ui.font(max(12, h // 34))
        th = self.tiny.get_height()
        bh = max(20, min(28, h // 15), th + 8)             # wächst mit der Schrift
        gap = 5
        self.bh = bh
        y = 6
        back_w = max(48, int(w * 0.11))
        save_w = max(62, int(w * 0.16))
        field_w = (w - 24 - back_w - save_w - 3 * gap) / 2.0
        self.back_rect = pygame.Rect(12, y, back_w, bh)
        self.name_rect = pygame.Rect(int(12 + back_w + gap), y, int(field_w), bh)
        self.id_rect = pygame.Rect(int(self.name_rect.right + gap), y, int(field_w), bh)
        self.save_rect = pygame.Rect(int(self.id_rect.right + gap), y, save_w, bh)
        y2 = y + bh + gap
        icon_w = max(22, bh + 2)
        # Text-Knöpfe so breit wie ihre (übersetzte) Beschriftung plus Symbol
        need = max(self.tiny.size(t("gd.ed.btn_" + k))[0] for k in TEXT_BUTTONS) + bh + 10
        text_w = max(52, min(int(w * 0.2), need))
        self.edit_rects = {}
        x = 12
        for key in TEXT_BUTTONS:
            self.edit_rects[key] = pygame.Rect(x, y2, text_w, bh)
            x += text_w + gap
        for key in ICON_BUTTONS:
            self.edit_rects[key] = pygame.Rect(x, y2, icon_w, bh)
            x += icon_w + gap
        self.pos_x = x + 4
        self.head_bottom = y2 + bh + 6
        # Unten: Übersichtsleiste + Parameter-/Hinweiszeile
        self.map_h = max(12, h // 30)
        self.map_rect = pygame.Rect(12, h - self.map_h - 6, w - 24, self.map_h)
        # Werte-Zeile: Beschriftung ("Korridor", "R", ...) über den Knöpfen
        self.par_font = ui.font(max(9, h // 52))
        self.par_lab_h = self.par_font.get_height()
        self.par_h = bh + 8 + self.par_lab_h
        self.par_top = self.map_rect.y - self.par_h - 2
        # Palette rechts: Werkzeuge, sechs Gruppen, Objekte der Gruppe
        pbw = max(30, min(max(50, h // 13), int(w * 0.07)))
        cols = 2
        max_items = max(len(g_[1]) for g_ in GROUPS)
        rows = 1 + 3 + (max_items + 1) // 2
        avail = self.par_top - self.head_bottom - 14
        pbh = max(18, min(pbw, (avail - (rows - 1) * 3) // rows))
        self.pal_w = cols * pbw + 3
        px0 = w - self.pal_w - 8
        self.tool_rects = [(k, pygame.Rect(px0 + i * (pbw + 3), self.head_bottom, pbw, pbh))
                           for i, k in enumerate(TOOLS)]
        gy = self.head_bottom + pbh + 6
        self.group_rects = []
        for i, key in enumerate(GROUP_KEYS):
            r, c = divmod(i, cols)
            self.group_rects.append((key, pygame.Rect(px0 + c * (pbw + 3), gy + r * (pbh + 3),
                                                      pbw, pbh)))
        self.items_top = gy + 3 * (pbh + 3) + 6
        self.pbw, self.pbh, self.px0 = pbw, pbh, px0
        # Leinwand
        self.canvas = pygame.Rect(12, self.head_bottom, px0 - 12 - 10,
                                  self.par_top - 4 - self.head_bottom)
        self.ts = max(8, self.canvas.h // ROWS_VISIBLE)
        self.renderer.resize(self.canvas.w, self.canvas.h, self.ts)
        self.icons.resize(pbw, pbh, max(8, min(pbw, pbh) - 8))
        # Einstellungs-Dialog
        pw = min(w - 30, max(440, int(w * 0.5)))
        rh = max(22, min(30, h // 16), th + 8)
        head = self.small.get_height() + 18
        ph = head + len(SETTINGS_ROWS) * (rh + 8) + rh + 18
        self.set_rect = pygame.Rect((w - pw) // 2, max(6, (h - ph) // 2), pw, min(h - 12, ph))
        self.set_rows = {}
        lab_w = int(pw * 0.34)
        for i, key in enumerate(SETTINGS_ROWS):
            ry = self.set_rect.y + head + i * (rh + 8)
            x0 = self.set_rect.x + 14 + lab_w
            vw = self.set_rect.right - 14 - x0
            if key in ("bg", "ground"):
                n = len(PRESETS)
                cw = max(8, (vw - (n - 1) * 3) // n)
                self.set_rows[key] = [pygame.Rect(x0 + j * (cw + 3), ry, cw, rh) for j in range(n)]
            else:
                self.set_rows[key] = [pygame.Rect(x0, ry, rh, rh),
                                      pygame.Rect(x0 + rh + 4, ry, vw - 2 * rh - 8, rh),
                                      pygame.Rect(x0 + vw - rh, ry, rh, rh)]
        cw = max(140, self.tiny.size(t("gd.ed.close"))[0] + 40)
        self.set_close = pygame.Rect(self.set_rect.centerx - cw // 2, self.set_rect.bottom - rh - 10, cw, rh)
        self.lab_w = lab_w

    def item_rects(self):
        items = dict(GROUPS)[self.group]
        out = []
        for i, key in enumerate(items):
            r, c = divmod(i, 2)
            out.append((key, pygame.Rect(self.px0 + c * (self.pbw + 3),
                                         self.items_top + r * (self.pbh + 3), self.pbw, self.pbh)))
        return out

    # ----- Undo -------------------------------------------------------------
    def _snapshot(self):
        return ([list(o) for o in self.objects], copy.deepcopy(self.meta))

    def _push(self):
        self.undo_stack.append(self._snapshot())
        if len(self.undo_stack) > UNDO_MAX:
            del self.undo_stack[0]
        self.redo_stack.clear()

    def _undo(self):
        if not self.undo_stack:
            return
        self.redo_stack.append(self._snapshot())
        self.objects, self.meta = self.undo_stack.pop()
        self.sel = None
        self._changed()
        self.game.play_sound("move")

    def _redo(self):
        if not self.redo_stack:
            return
        self.undo_stack.append(self._snapshot())
        self.objects, self.meta = self.redo_stack.pop()
        self.sel = None
        self._changed()
        self.game.play_sound("move")

    def _toast(self, text):
        self.toast = text
        self.toast_t = 2.6

    def update(self, dt):
        self.t += dt
        if self.toast_t > 0:
            self.toast_t -= dt
            if self.toast_t <= 0:
                self.toast = ""

    # ----- Koordinaten ------------------------------------------------------
    def cell_at(self, pos):
        x = (pos[0] - self.canvas.x) / float(self.ts) + self.cam_x
        y = (self.canvas.bottom - pos[1]) / float(self.ts) + self.cam_y
        return int(math.floor(x)), int(math.floor(y))

    def find(self, cx, cy, layer=None):
        """Index des obersten Objekts in Zelle (cx, cy) - oder None."""
        best = None
        for i, o in enumerate(self.objects):
            if o[1] == cx and (o[2] == cy or (o[0] == "pit" and cy <= 0)):
                if layer is not None and layer_of(o[0]) != layer:
                    continue
                if best is None or layer_of(o[0]) <= layer_of(self.objects[best][0]):
                    best = i
        return best

    def _clamp_cam(self):
        end = core.level_length(self.level_dict()) if self.objects else core.MIN_LENGTH
        self.cam_x = max(-4.0, min(float(end), self.cam_x))
        self.cam_y = max(-1.0, min(float(core.MAX_ROW - ROWS_VISIBLE + 2), self.cam_y))

    # ----- Eingabe ------------------------------------------------------
    def handle(self, event):
        if self.settings_open:
            return self._handle_settings(event)
        if event.kind == InputEvent.KEYDOWN:
            return self._handle_key(event)
        if event.kind == InputEvent.MOUSEDOWN:
            return self._handle_down(event)
        if event.kind == InputEvent.MOUSEMOVE:
            return self._handle_move(event)
        if event.kind == InputEvent.MOUSEUP:
            if self.drag and self.drag[0] == "pan" and not self.drag[3]:
                # Rechtsklick ohne Ziehen: Objekt unter dem Zeiger löschen
                self._erase_at(self.cell_at(event.pos))
            self.drag = None
            return True
        if event.kind == InputEvent.WHEEL:
            if self.map_rect.collidepoint(event.pos) or self.canvas.collidepoint(event.pos):
                self.cam_x -= event.delta * 3
                self._clamp_cam()
            return True
        return False

    def _handle_key(self, event):
        k = event.key
        if self.focus >= 0:
            field = self.f_name if self.focus == 0 else self.f_id
            if field.handle(event):
                self.dirty = True
                self.err = ""
                return True
            if k in ("Tab", "ISO_Left_Tab"):
                self.focus = 1 - self.focus
                return True
            if k in ("Return", "KP_Enter", "Escape"):
                self.focus = -1
                return True
        if k == "Escape":
            self._back()
        elif k in ("Left", "a", "A"):
            self.cam_x -= 4 if k == "Left" else 1
            self._clamp_cam()
        elif k in ("Right", "d", "D"):
            self.cam_x += 4 if k == "Right" else 1
            self._clamp_cam()
        elif k in ("Up", "w", "W"):
            self.cam_y += 1
            self._clamp_cam()
        elif k == "Down":
            self.cam_y -= 1
            self._clamp_cam()
        elif k in ("Home",):
            self.cam_x = -2.0
        elif k in ("End",):
            self.cam_x = core.level_length(self.level_dict()) - 10
            self._clamp_cam()
        elif k in ("Delete", "BackSpace"):
            self._delete_selected()
        elif k in ("r", "R"):
            self._rotate()
        elif k in ("u", "U", "z", "Z"):
            self._undo()
        elif k in ("y", "Y"):
            self._redo()
        elif k in ("g", "G"):
            self._toggle_grid()
        elif k in ("s", "S"):
            self._save()
        elif k in ("Return", "KP_Enter"):
            self._test(False)
        elif k in ("t", "T"):
            self._test(True)
        elif k in ("1", "2", "3", "4", "5", "6"):
            self._pick_group(GROUP_KEYS[int(k) - 1])
        return True

    def _handle_down(self, event):
        pos = event.pos
        if event.button == 3:
            if self.canvas.collidepoint(pos):
                self.drag = ("pan", pos, self.cam_x, False, self.cam_y)
            return True
        if self.name_rect.collidepoint(pos):
            self.focus = 0
            return True
        if self.id_rect.collidepoint(pos):
            self.focus = 1
            return True
        self.focus = -1
        if self.back_rect.collidepoint(pos):
            self._back()
            return True
        if self.save_rect.collidepoint(pos):
            self._save()
            return True
        for key, rc in self.edit_rects.items():
            if rc.collidepoint(pos):
                {"test": lambda: self._test(False), "test_here": lambda: self._test(True),
                 "undo": self._undo, "redo": self._redo, "rotate": self._rotate,
                 "delete": self._delete_selected, "grid": self._toggle_grid,
                 "settings": self._open_settings}[key]()
                return True
        for key, rc in self.tool_rects:
            if rc.collidepoint(pos):
                self.tool = key
                self.sel = None
                self.game.play_sound("select")
                return True
        for key, rc in self.group_rects:
            if rc.collidepoint(pos):
                self._pick_group(key)
                return True
        for key, rc in self.item_rects():
            if rc.collidepoint(pos):
                self.tool = key
                self.sel = None
                self.rot = 2 if (key in core.ROTATE_FLIP and self.rot == 2) else 0
                self.game.play_sound("select")
                return True
        if self.map_rect.collidepoint(pos):
            self._jump_map(pos)
            self.drag = ("map",)
            return True
        if self._param_click(pos):
            return True
        if self.canvas.collidepoint(pos):
            self._canvas_down(pos)
        return True

    def _pick_group(self, key):
        self.group = key
        items = dict(GROUPS)[key]
        if self.tool not in items and self.tool not in TOOLS:
            self.tool = items[0]
        elif self.tool in TOOLS:
            self.tool = items[0]
        self.sel = None
        self.game.play_sound("select")

    def _handle_move(self, event):
        pos = event.pos
        self.hover = self.cell_at(pos) if self.canvas.collidepoint(pos) else None
        d = self.drag
        if not d:
            return True
        if d[0] == "pan":
            dx = (pos[0] - d[1][0]) / float(self.ts)
            dy = (pos[1] - d[1][1]) / float(self.ts)
            moved = d[3] or abs(pos[0] - d[1][0]) + abs(pos[1] - d[1][1]) > 4
            self.cam_x = d[2] - dx
            self.cam_y = d[4] + dy
            self._clamp_cam()
            self.drag = ("pan", d[1], d[2], moved, d[4])
        elif d[0] == "map":
            self._jump_map(pos)
        elif d[0] == "paint" and self.canvas.collidepoint(pos):
            cell = self.cell_at(pos)
            if cell != d[1]:
                self._place(cell, push=False)
                self.drag = ("paint", cell)
        elif d[0] == "erase" and self.canvas.collidepoint(pos):
            self._erase_at(self.cell_at(pos), push=False)
        elif d[0] == "move":
            cell = self.cell_at(pos)
            if self.sel is not None and cell != d[1]:
                o = self.objects[self.sel]
                nx = max(0, min(core.MAX_LENGTH, o[1] + cell[0] - d[1][0]))
                ny = max(0, min(core.MAX_ROW, o[2] + cell[1] - d[1][1]))
                if o[0] == "pit":
                    ny = 0
                if self.find(nx, ny, layer_of(o[0])) in (None, self.sel):
                    o[1], o[2] = nx, ny
                    self._changed()
                self.drag = ("move", cell)
        return True

    def _jump_map(self, pos):
        end = max(core.MIN_LENGTH, core.level_length(self.level_dict()))
        f = (pos[0] - self.map_rect.x) / float(max(1, self.map_rect.w))
        view = self.canvas.w / float(self.ts)
        self.cam_x = f * end - view / 2.0
        self._clamp_cam()

    def _canvas_down(self, pos):
        cell = self.cell_at(pos)
        if cell[0] < 0 or cell[1] < 0:
            return
        if self.tool == "select":
            i = self.find(*cell)
            self.sel = i
            if i is not None:
                self._push()
                self.drag = ("move", cell)
                self.game.play_sound("move")
            return
        if self.tool == "erase":
            self._push()
            self._erase_at(cell, push=False)
            self.drag = ("erase",)
            return
        self._push()
        self._place(cell, push=False)
        self.drag = ("paint", cell)

    def _place(self, cell, push=True):
        x, y = cell
        if x < 0 or y < 0 or x > core.MAX_LENGTH or y > core.MAX_ROW:
            return
        kind = self.tool
        if kind == "pit":
            y = 0
        if kind == "coin":
            coins = [i for i, o in enumerate(self.objects) if o[0] == "coin"]
            if len(coins) >= core.MAX_COINS and self.find(x, y) not in coins:
                self._toast(t("gd.ed.coins_max"))
                self.game.play_sound("hit")
                return
        if push:
            self._push()
        old = self.find(x, y, layer_of(kind))
        rot = self.rot if (kind in core.ROTATE_ALL or kind in core.ROTATE_FLIP) else 0
        obj = core.normalize_object([kind, x, y, rot] + default_param(kind))
        if old is not None:
            if self.objects[old][:4] == obj[:4]:
                self.sel = old
                return
            self.objects[old] = obj
            self.sel = old
        else:
            self.objects.append(obj)
            self.sel = len(self.objects) - 1
        self._changed()
        self.game.play_sound("click")

    def _erase_at(self, cell, push=True):
        i = self.find(*cell)
        if i is None:
            return
        if push:
            self._push()
        del self.objects[i]
        self.sel = None
        self._changed()
        self.game.play_sound("hit")

    def _delete_selected(self):
        if self.sel is None or self.sel >= len(self.objects):
            return
        self._push()
        del self.objects[self.sel]
        self.sel = None
        self._changed()
        self.game.play_sound("hit")

    def _rotate(self):
        if self.sel is not None and self.sel < len(self.objects):
            o = self.objects[self.sel]
            if o[0] in core.ROTATE_ALL or o[0] in core.ROTATE_FLIP:
                self._push()
                o[3] = (o[3] + (2 if o[0] in core.ROTATE_FLIP else 1)) % 4
                self._changed()
                self.game.play_sound("rotate")
            return
        self.rot = (self.rot + (2 if self.tool in core.ROTATE_FLIP else 1)) % 4
        self.game.play_sound("rotate")

    def _toggle_grid(self):
        self.grid = not self.grid
        self.game.set_grid_snap(self.grid)
        self.game.play_sound("select")

    # ----- Parameterzeile ---------------------------------------------------
    def _params(self):
        if self.sel is None or self.sel >= len(self.objects):
            return None, ()
        o = self.objects[self.sel]
        return o, core.PARAMS.get(o[0], ())

    def _param_rects(self):
        o, params = self._params()
        if not params:
            return []
        w = self.game.width
        bh = self.bh
        step_w = max(16, int(w * 0.032))
        val_w = max(30, int(w * 0.06))
        grp = 2 * step_w + val_w
        total = len(params) * grp + (len(params) - 1) * 6 + (val_w // 2 if o[0] == "color" else 0)
        x = max(10, (self.canvas.right - total) // 2)
        y = self.par_top + self.par_lab_h + 2
        out = []
        for i, p in enumerate(params):
            vw = val_w + (val_w // 2 if (o[0] == "color" and i == 0) else 0)
            out.append((i, pygame.Rect(x, y, step_w, bh), pygame.Rect(x + step_w, y, vw, bh),
                        pygame.Rect(x + step_w + vw, y, step_w, bh)))
            x += 2 * step_w + vw + 6
        return out

    def _param_click(self, pos):
        o, params = self._params()
        for i, minus, _val, plus in self._param_rects():
            for rc, d in ((minus, -1), (plus, 1)):
                if rc.collidepoint(pos):
                    default, lo, hi = params[i]
                    stepv = 5 if (o[0] == "color" and 1 <= i <= 3) else 1
                    if o[0] == "color" and 1 <= i <= 3:
                        stepv = 15
                    self._push()
                    o[4 + i] = max(lo, min(hi, o[4 + i] + d * stepv))
                    self._changed()
                    self.game.play_sound("move")
                    return True
        return False

    # ----- Einstellungen ----------------------------------------------------
    def _open_settings(self):
        self.settings_open = True
        self.game.play_sound("click")

    def _handle_settings(self, event):
        if event.kind == InputEvent.KEYDOWN and event.key in ("Escape", "Return"):
            self.settings_open = False
            return True
        if event.kind != InputEvent.MOUSEDOWN:
            return True
        pos = event.pos
        if self.set_close.collidepoint(pos) or not self.set_rect.collidepoint(pos):
            self.settings_open = False
            return True
        for key, rects in self.set_rows.items():
            for j, rc in enumerate(rects):
                if not rc.collidepoint(pos):
                    continue
                self._push()
                if key in ("bg", "ground"):
                    self.meta[key] = list(PRESETS[j])
                elif j in (0, 2):
                    d = -1 if j == 0 else 1
                    if key == "speed":
                        self.meta["speed"] = (self.meta["speed"] + d) % 4
                    elif key == "mode":
                        self.meta["mode"] = (self.meta["mode"] + d) % 5
                    elif key == "music":
                        styles = core.MUSIC_STYLES
                        i = styles.index(self.meta.get("music", "drive"))
                        self.meta["music"] = styles[(i + d) % len(styles)]
                    elif key == "bpm":
                        from . import geodash_music as music
                        cur = music.style_bpm(self.meta) or 120
                        self.meta["bpm"] = max(80, min(200, cur + 4 * d))
                self._changed()
                self.game.play_sound("move")
                return True
        return True

    # ----- Speichern / Testen / Verlassen ----------------------------------------
    def _save(self):
        name = self.f_name.text.strip()
        level_id = self.f_id.text.strip() or ugc.slug(name)
        if not level_id:
            return self._fail("gd.ugc.err.id_empty")
        if not ugc.valid_id(level_id):
            return self._fail("gd.ugc.err.id_chars")
        if not name:
            return self._fail("gd.ugc.err.name")
        taken = {m["id"] for m in ugc.load_maps(GAME) if m["id"] != self.orig_id}
        if level_id in taken:
            return self._fail("gd.ugc.err.id_dup")
        if not swear.all_clean(name, level_id, self.meta.get("author")):
            return self._fail("gd.ugc.err.swear")
        m = self.level_dict()
        m["id"] = level_id
        m["name"] = name
        if self.orig_id and self.orig_id != level_id:
            ugc.delete_map(self.orig_id, GAME)
        ok, why = ugc.save_map(m, GAME)
        if not ok:
            return self._fail("gd.ugc.err." + (why if why in ("swear", "full", "io", "id") else "invalid"),
                              max=ugc.max_items(GAME))
        self.meta["id"] = level_id
        self.meta["name"] = name
        self.orig_id = level_id
        self.dirty = False
        self.f_id.set_text(level_id)
        self._toast(t("gd.ugc.saved"))
        self.game.play_sound("win")
        return True

    def _fail(self, key, **kw):
        self.err = t(key, **kw)
        self.game.play_sound("hit")
        return False

    def _test(self, here):
        d = self.level_dict()
        d["name"] = self.f_name.text.strip() or t("gd.ed.untitled")
        d["id"] = self.f_id.text.strip() or self.orig_id or "test"
        start = None
        if here:
            start = max(1, int(math.floor(self.cam_x + 2)))
            if start >= core.level_length(d) - 2:
                start = None
        self.game.ugc_test(d, start)

    def _back(self):
        if self.settings_open:
            self.settings_open = False
            return
        if self.dirty and not self.confirm_back:
            self.confirm_back = True
            self._toast(t("gd.ugc.unsaved"))
            self.game.play_sound("select")
            return
        self.game.ugc_close_editor()

    # ----- Zeichnen -----------------------------------------------------
    def draw(self, s):
        self._draw_canvas(s)
        self._draw_head(s)
        self._draw_palette(s)
        self._draw_bottom(s)
        if self.toast:
            img = self.tiny.render(self.toast, True, ui.TEXT)
            w, h = img.get_width() + 24, img.get_height() + 10
            r = pygame.Rect(self.canvas.centerx - w // 2, self.canvas.bottom - h - 8, w, h)
            ui.draw_panel(s, r, radius=h // 2, shadow=False)
            pygame.draw.rect(s, ui.ACCENT2, r, 1, border_radius=h // 2)
            s.blit(img, img.get_rect(center=r.center))
        if self.settings_open:
            self._draw_settings(s)

    def _draw_canvas(self, s):
        lv = self.compiled()
        sub = s.subsurface(self.canvas)
        r = self.renderer
        r.resize(self.canvas.w, self.canvas.h, self.ts)
        cam_x, cam_y = self.cam_x, self.cam_y
        bg = gdraw.color_at(lv, cam_x + 4, 0)
        ground = gdraw.color_at(lv, cam_x + 4, 1)
        r.draw_backdrop(sub, bg, cam_x, cam_y, 0.0)
        ts = self.ts
        if self.grid:
            gcol = gdraw.mix(bg, (255, 255, 255), 0.12)
            x0 = int(math.floor(cam_x))
            for c in range(x0, x0 + self.canvas.w // ts + 2):
                x = int(r.sx(c, cam_x))
                pygame.draw.line(sub, gcol, (x, 0), (x, self.canvas.h))
            y0 = int(math.floor(cam_y))
            for row in range(y0, y0 + ROWS_VISIBLE + 2):
                y = int(r.sy(row, cam_y))
                pygame.draw.line(sub, gcol, (0, y), (self.canvas.w, y))
        r.draw_objects(sub, lv, cam_x, cam_y, self.t, 0.0, (), editor=True)
        r.draw_ground(sub, lv, ground, cam_x, cam_y, 0.0)
        # Start- und Ziellinie
        for xb, col in ((0, gdraw.COL_CHECK), (lv.length, (255, 255, 255))):
            x = int(r.sx(xb, cam_x))
            if 0 <= x <= self.canvas.w:
                pygame.draw.line(sub, col, (x, 0), (x, self.canvas.h), 2)
        # Startform am Anfang
        if -2 < r.sx(0, cam_x) < self.canvas.w:
            r.draw_player(sub, -1.4, 0, self.meta.get("mode", 0), 1, 0, cam_x, cam_y, self.t)
        # Auswahl + Zeiger
        if self.sel is not None and self.sel < len(self.objects):
            o = self.objects[self.sel]
            rc = pygame.Rect(int(r.sx(o[1], cam_x)), int(r.sy(o[2] + 1, cam_y)), ts, ts)
            if o[0] in ("p_cube", "p_ship", "p_ball", "p_ufo", "p_wave", "g_norm", "g_flip",
                        "s_slow", "s_norm", "s_fast", "s_vfast"):
                rc = rc.inflate(0, 2 * ts)
            pygame.draw.rect(sub, self.game.accent, rc.inflate(6, 6), 2, border_radius=4)
        if self.hover is not None and self.tool not in TOOLS:
            hx, hy = self.hover
            rc = pygame.Rect(int(r.sx(hx, cam_x)), int(r.sy(hy + 1, cam_y)), ts, ts)
            ghost = pygame.Surface((ts, ts), pygame.SRCALPHA)
            ghost.fill((255, 255, 255, 40))
            sub.blit(ghost, rc)
            pygame.draw.rect(sub, (255, 255, 255), rc, 1)
        pygame.draw.rect(s, ui.BORDER, self.canvas.inflate(4, 4), 1, border_radius=4)

    def _draw_head(self, s):
        g = self.game
        _btn(s, self.back_rect, t("gd.ugc.btn_back"), self.tiny)
        self.f_name.draw(s, self.name_rect, self.tiny, focused=(self.focus == 0),
                         invalid=bool(self.err) and self.focus == 0)
        self.f_id.draw(s, self.id_rect, self.tiny, focused=(self.focus == 1),
                       invalid=bool(self.err) and self.focus == 1)
        _btn(s, self.save_rect, t("gd.ugc.btn_save"), self.tiny, on=self.dirty, accent=g.accent)
        for key in TEXT_BUTTONS:
            rc = self.edit_rects[key]
            pygame.draw.rect(s, ui.BTN, rc, border_radius=7)
            pygame.draw.rect(s, g.accent, rc, 1, border_radius=7)
            ic = pygame.Rect(rc.x + 2, rc.y, rc.h, rc.h)
            _icon(s, ic, key, g.accent)
            lab = t("gd.ed.btn_" + key)
            img = self.tiny.render(lab, True, ui.TEXT)
            room = rc.w - rc.h - 4
            if img.get_width() > room:
                kurz = lab
                while len(kurz) > 1 and self.tiny.size(kurz + ".")[0] > room:
                    kurz = kurz[:-1]
                img = self.tiny.render(kurz + ".", True, ui.TEXT)
            s.blit(img, img.get_rect(midleft=(ic.right, rc.centery)))
        for key in ICON_BUTTONS:
            rc = self.edit_rects[key]
            on = (key == "grid" and self.grid) or (key == "settings" and self.settings_open)
            enabled = {"undo": bool(self.undo_stack), "redo": bool(self.redo_stack),
                       "delete": self.sel is not None}.get(key, True)
            pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=7)
            pygame.draw.rect(s, g.accent if on else ui.BORDER, rc, 2 if on else 1, border_radius=7)
            _icon(s, rc, key, ui.TEXT if (on or enabled) else ui.TEXT_FAINT)
        # Position + Verifiziert
        info = "x %d  ·  %s" % (int(self.cam_x + 2), t("gd.ugc.length", n=self.compiled().length))
        img = self.tiny.render(info, True, ui.TEXT_DIM)
        right = self.canvas.right
        if self.verified and self.verified == core.content_hash(self.level_dict()):
            br = max(6, self.bh // 3)
            _check_badge(s, right - br, self.edit_rects["settings"].centery, br)
            right -= 2 * br + 6
        if self.pos_x + img.get_width() <= right:
            s.blit(img, img.get_rect(midright=(right, self.edit_rects["settings"].centery)))
        if self.err:
            e = self.tiny.render(self.err, True, ui.RED)
            box = e.get_rect(topleft=(self.canvas.x + 6, self.canvas.y + 4))
            pygame.draw.rect(s, (0, 0, 0), box.inflate(8, 4), border_radius=4)
            s.blit(e, box)
        else:
            # Name des Werkzeugs als Plakette auf der Leinwand - in der
            # schmalen Palette wäre dafür kein Platz.
            text = self.tool_label()
            img = self.tiny.render(text, True, g.accent)
            box = img.get_rect(topright=(self.canvas.right - 8, self.canvas.y + 5))
            pygame.draw.rect(s, (0, 0, 0), box.inflate(12, 6), border_radius=6)
            pygame.draw.rect(s, g.accent, box.inflate(12, 6), 1, border_radius=6)
            s.blit(img, box)

    def tool_label(self):
        """"Stachel · Gefahren" bzw. "Auswählen"."""
        if self.tool in TOOLS:
            return t("gd.ed.tool." + self.tool)
        return "%s  ·  %s" % (t("gd.ed.obj." + self.tool), t("gd.ed.group." + self.group))

    def _draw_palette(self, s):
        g = self.game
        for key, rc in self.tool_rects:
            on = self.tool == key
            pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=6)
            pygame.draw.rect(s, g.accent if on else ui.BORDER, rc, 2 if on else 1, border_radius=6)
            _icon(s, rc, key, ui.TEXT if on else ui.TEXT_DIM)
        for key, rc in self.group_rects:
            on = self.group == key
            pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=6)
            pygame.draw.rect(s, g.accent if on else ui.BORDER, rc, 2 if on else 1, border_radius=6)
            self._draw_item_icon(s, rc, GROUP_ICON[key], 0, small=True)
        top = self.group_rects[-1][1].bottom + 2
        pygame.draw.line(s, ui.BORDER, (self.px0, top + 1), (self.px0 + self.pal_w, top + 1))
        for key, rc in self.item_rects():
            on = self.tool == key
            pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc, border_radius=6)
            pygame.draw.rect(s, g.accent if on else ui.BORDER, rc, 2 if on else 1, border_radius=6)
            rot = self.rot if on and (key in core.ROTATE_ALL or key in core.ROTATE_FLIP) else 0
            self._draw_item_icon(s, rc, key, rot)

    def _draw_item_icon(self, s, rc, kind, rot, small=False):
        """Palettenknopf: das Objekt selbst, verkleinert gezeichnet."""
        ir = self.icons
        size = max(8, min(rc.w, rc.h) - (12 if small else 8))
        ir.resize(rc.w, rc.h, size)
        tmp = self.icons._cache.get(("icon", kind, rot, size))
        if tmp is None:
            tmp = pygame.Surface((rc.w, rc.h), pygame.SRCALPHA)
            d = core.normalize_level({"objects": [[kind, 0, 0, rot] + default_param(kind)],
                                      "length": 50})
            if kind == "color":
                d["objects"][0][5:8] = [255, 90, 210]
            lv = core.Level(d)
            gdraw.block_masks(lv)
            cam_x = -(rc.w - size) / 2.0 / size
            cam_y = -(rc.h - size) / 2.0 / size
            if kind == "pit":
                pygame.draw.rect(tmp, (4, 2, 8), ((rc.w - size) // 2, rc.h // 2, size, rc.h // 2 - 3))
                pygame.draw.line(tmp, (255, 80, 80), ((rc.w - size) // 2, rc.h // 2),
                                 ((rc.w + size) // 2, rc.h // 2), 2)
            elif kind.startswith("p_") or kind.startswith("g_") or kind.startswith("s_"):
                big = gdraw.Renderer()
                big.resize(rc.w, rc.h, max(6, size // 3 + 2))
                big.draw_objects(tmp, lv, -(rc.w - big.ts) / 2.0 / big.ts,
                                 -(rc.h - big.ts) / 2.0 / big.ts, 0.0, 0.0, (), editor=True)
            else:
                ir.draw_objects(tmp, lv, cam_x, cam_y, 0.3, 0.0, (), editor=True)
            self.icons._cache[("icon", kind, rot, size)] = tmp
        s.blit(tmp, rc.topleft)

    def _draw_bottom(self, s):
        g = self.game
        rects = self._param_rects()
        if rects:
            o, params = self._params()
            for i, minus, val, plus in rects:
                _btn(s, minus, "-", self.tiny)
                _btn(s, plus, "+", self.tiny)
                pygame.draw.rect(s, ui.PANEL, val, border_radius=7)
                pygame.draw.rect(s, ui.BORDER, val, 1, border_radius=7)
                v = o[4 + i]
                if o[0] == "color" and i == 0:
                    txt = t("gd.ed.target_bg") if v == 0 else t("gd.ed.target_ground")
                elif o[0] != "color" and v == 0:
                    txt = str(core.CORRIDOR // core.B)      # Standardhöhe
                else:
                    txt = str(v)
                img = self.tiny.render(txt, True, ui.TEXT)
                if img.get_width() > val.w - 4:
                    img = ui.font(max(9, g.height // 56)).render(txt, True, ui.TEXT)
                s.blit(img, img.get_rect(center=val.center))
                lab = PARAM_LABELS.get(o[0], ())[i] if i < len(PARAM_LABELS.get(o[0], ())) else ""
                if lab:
                    lab = t(lab) if lab.startswith("gd.") else lab
                    li = self.par_font.render(lab, True, ui.TEXT_FAINT)
                    s.blit(li, li.get_rect(midbottom=(val.centerx, val.top - 1)))
            if o[0] == "color":
                sw = pygame.Rect(rects[-1][3].right + 8, rects[0][1].y, rects[0][1].h, rects[0][1].h)
                if sw.right < self.canvas.right:
                    pygame.draw.rect(s, (o[5], o[6], o[7]), sw, border_radius=5)
                    pygame.draw.rect(s, ui.BORDER, sw, 1, border_radius=5)
        else:
            if self.tool == "select":
                hint = t("gd.ed.hint_select")
            elif self.tool == "erase":
                hint = t("gd.ed.hint_erase")
            else:
                hint = t("gd.ed.hint_place")
            img = self.tiny.render(hint, True, ui.TEXT_FAINT)
            if img.get_width() > self.canvas.w:
                img = ui.font(max(9, g.height // 52)).render(hint, True, ui.TEXT_FAINT)
            s.blit(img, img.get_rect(center=(self.canvas.centerx, self.par_top + self.par_h // 2)))
        # Übersicht
        mr = self.map_rect
        pygame.draw.rect(s, ui.PANEL, mr, border_radius=4)
        lv = self.compiled()
        end = max(core.MIN_LENGTH, lv.length)
        for i in range(lv.n):
            cls = lv.cls[i]
            if cls == core.C_TRIGGER:
                continue
            x = mr.x + int(lv.ox[i] / float(end) * mr.w)
            y = mr.bottom - 2 - int(min(1.0, lv.oy[i] / 12.0) * (mr.h - 4))
            col = (230, 230, 240) if cls == core.C_SOLID else (
                (255, 90, 90) if cls in (core.C_HAZARD, core.C_PIT) else g.accent)
            s.fill(col, (x, y, 2, 2))
        view = self.canvas.w / float(self.ts)
        vx = mr.x + int(max(0.0, self.cam_x) / end * mr.w)
        vw = max(4, int(view / end * mr.w))
        pygame.draw.rect(s, g.accent, (vx, mr.y, min(vw, mr.right - vx), mr.h), 1, border_radius=3)

    def _draw_settings(self, s):
        g = self.game
        veil = pygame.Surface((g.width, g.height), pygame.SRCALPHA)
        veil.fill((0, 0, 0, 160))
        s.blit(veil, (0, 0))
        r = self.set_rect
        ui.draw_panel(s, r, accent_top=g.accent)
        head = self.small.render(t("gd.ed.settings"), True, g.accent)
        s.blit(head, head.get_rect(midtop=(r.centerx, r.y + 10)))
        from . import geodash_music as music
        for key in SETTINGS_ROWS:
            rects = self.set_rows[key]
            lab = self.tiny.render(t("gd.ed.set_" + key), True, ui.TEXT_DIM)
            if lab.get_width() > self.lab_w - 6:
                kurz = t("gd.ed.set_" + key)
                while len(kurz) > 2 and self.tiny.size(kurz + ".")[0] > self.lab_w - 6:
                    kurz = kurz[:-1]
                lab = self.tiny.render(kurz + ".", True, ui.TEXT_DIM)
            s.blit(lab, lab.get_rect(midleft=(r.x + 14, rects[0].centery)))
            if key in ("bg", "ground"):
                cur = tuple(self.meta.get(key, ()))
                for j, rc in enumerate(rects):
                    pygame.draw.rect(s, PRESETS[j], rc, border_radius=4)
                    on = tuple(PRESETS[j]) == cur
                    pygame.draw.rect(s, g.accent if on else ui.BORDER, rc, 2 if on else 1, border_radius=4)
                continue
            _btn(s, rects[0], "<", self.tiny)
            _btn(s, rects[2], ">", self.tiny)
            if key == "speed":
                txt = core.SPEED_LABELS[self.meta["speed"]]
            elif key == "mode":
                txt = t("gd.ed.mode." + core.MODE_NAMES[self.meta["mode"]])
            elif key == "music":
                txt = t("gd.ed.music." + self.meta.get("music", "drive"))
            else:
                bpm = music.style_bpm(self.meta)
                txt = str(bpm) if bpm else "-"
            _btn(s, rects[1], txt, self.tiny, on=True, accent=g.accent)
        _btn(s, self.set_close, t("gd.ed.close"), self.tiny)
