# -*- coding: utf-8 -*-
"""
lamawiki.py
===========
LamaWiki - das In-Game-Wiki der Spielesammlung.

Inhalte
-------
Die Artikel liegen als JSON neben diesem Modul (``de.json``/``en.json``/
``fr.json``/``es.json``/``pt.json``), eine Datei je Sprache mit identischer
Struktur:

    {"pages": [{"id": "...", "category": "general"|"games",
                "game": "SnakeGame" (optional, exakter Klassenname),
                "title": "...", "keywords": ["..."],
                "sections": [{"h": "Überschrift", "body": ["Absatz", "- Bullet"]}]}]}

- Die Reihenfolge der Seiten in der Datei ist die Anzeige-Reihenfolge.
- ``id`` ist über alle Sprachen stabil (Sprung aus dem Vorspiel-Screen).
- ``game`` verknüpft eine Seite mit einem Spiel: Akzentfarbe aus
  ui.GAME_COLORS + direkter Absprung aus dessen Vorspiel-Screen.
- Body-Strings: normaler Absatz oder Bullet ("- "-Präfix). Whitespace-
  getrennte Tokens der Form ``[Taste]`` werden als Tastenkappen-Chips
  gerendert (z.B. "Drücke [F11] für Vollbild.").

Screen
------
``LamaWikiScreen`` ist ein Menü-Screen (wie OptionsScreen): links Suchfeld +
kategorisierte Seitenliste, rechts der scrollbare Artikel. Bedienung:
Hoch/Runter wählt die Seite, Bild-Tasten/Mausrad blättern den Artikel,
Tippen filtert die Liste, Esc löscht erst die Suche und schließt dann.
"""

import json
import os
import re

import pygame

import i18n
import ui
from game_base import InputEvent
from i18n import t
from menu import _Screen

_DIR = os.path.dirname(os.path.abspath(__file__))

_CACHE = {}          # Sprachcode -> geparste Seitenliste

# Whitespace-getrenntes Tastenkappen-Token: "[Esc]" oder "[F11]," usw.
_CHIP_RE = re.compile(r"^\[([^\[\]]{1,14})\]([.,:;!?]?)$")

# Zeilen-/Abstands-Maße der Artikel-Darstellung (Pixel).
_LINE_H = 22         # Zeilenhöhe Fließtext
_PARA_GAP = 8        # Abstand zwischen Absätzen
_SECT_GAP = 16       # Abstand vor einer neuen Überschrift
_BULLET_INDENT = 16  # Einzug für Bullet-Zeilen


# ---------------------------------------------------------------------------
#  Inhalte laden
# ---------------------------------------------------------------------------

def load_pages(lang=None):
    """Liest die Wiki-Seiten der Sprache (Cache; Fallback: Deutsch)."""
    lang = lang or i18n.get_language()
    pages = _CACHE.get(lang)
    if pages is None:
        for code in (lang, "de"):
            try:
                with open(os.path.join(_DIR, f"{code}.json"),
                          encoding="utf-8") as f:
                    data = json.load(f)
                pages = [p for p in data.get("pages", [])
                         if isinstance(p, dict) and p.get("id")
                         and p.get("title") and p.get("sections")]
                if pages:
                    break
            except (OSError, json.JSONDecodeError, ValueError):
                pages = None
        _CACHE[lang] = pages = pages or []
    return pages


def page_id_for_game(cls_name, lang=None):
    """Seiten-id zur Spielklasse (z.B. 'SnakeGame' -> 'snake') oder None."""
    for p in load_pages(lang):
        if p.get("game") == cls_name:
            return p["id"]
    return None


def parse_inline(text):
    """Zerlegt einen Body-String in (is_bullet, tokens).

    tokens: Liste aus ("word", text) und ("chip", label, nachgestellte
    Interpunktion). Bullets beginnen mit "- ".
    """
    is_bullet = text.startswith("- ")
    if is_bullet:
        text = text[2:]
    tokens = []
    for raw in text.split():
        m = _CHIP_RE.match(raw)
        if m:
            tokens.append(("chip", m.group(1), m.group(2)))
        else:
            tokens.append(("word", raw))
    return is_bullet, tokens


# ---------------------------------------------------------------------------
#  Der LamaWiki-Screen
# ---------------------------------------------------------------------------

class LamaWikiScreen(_Screen):
    name = "Wiki"

    def __init__(self, surface, width, height, app, on_close, page_id=None):
        self.on_close = on_close
        self._start_page_id = page_id
        super().__init__(surface, width, height, app)
        self.name = t("lamawiki.name")

    def reset(self):
        super().reset()
        self.pages = load_pages()
        self.query = ""
        self.hover = None          # Zeilen-Index unter der Maus
        self.list_scroll = 0
        self.art_scroll = 0
        self._art_cache = {}       # (page_id, breite) -> (ops, content_h)
        self.page = None
        self._apply_filter()
        wanted = getattr(self, "_start_page_id", None)
        if wanted:
            for p in self.pages:
                if p["id"] == wanted:
                    self.page = p
                    break
        self._build_layout()
        self._ensure_selection_visible()

    def on_surface_changed(self):
        """Nach Auflösungswechsel: Layout + Umbruch neu, Scrolls klemmen."""
        self._build_layout()
        self._art_cache.clear()
        self.art_scroll = max(0, min(self.art_scroll, self._max_art_scroll()))
        self._ensure_selection_visible()

    # ----- Layout -----------------------------------------------------------

    def _build_layout(self):
        W, H = self.width, self.height
        lw = max(150, min(250, int(W * 0.30)))
        self.search_rect = pygame.Rect(16, 48, lw, 26)
        self.list_rect = pygame.Rect(16, 84, lw, H - 34 - 84)
        ax = 16 + lw + 12
        self.art_rect = pygame.Rect(ax, 48, W - ax - 16, H - 34 - 48)
        self._build_rows()

    def _build_rows(self):
        """Flache Zeilenliste (Kategorie-Header + Seiten) mit y-Positionen."""
        self.rows = []             # dicts: kind, y, h, (label|page)
        y = 0
        last_cat = None
        for p in self.filtered:
            cat = p.get("category", "general")
            if cat != last_cat:
                last_cat = cat
                self.rows.append(dict(kind="cat", y=y, h=22,
                                      label=t("lamawiki.cat." + cat)))
                y += 22
            self.rows.append(dict(kind="page", y=y, h=26, page=p))
            y += 26
        self.rows_h = y
        self.list_scroll = max(0, min(self.list_scroll,
                                      max(0, y - self.list_rect.h)))

    # ----- Filter/Auswahl -----------------------------------------------------

    def _apply_filter(self):
        q = self.query.strip().lower()
        if q:
            self.filtered = [p for p in self.pages
                             if q in p["title"].lower()
                             or any(q in k for k in p.get("keywords", ()))]
        else:
            self.filtered = list(self.pages)
        if self.page not in self.filtered:
            self.page = self.filtered[0] if self.filtered else None
            self.art_scroll = 0
        if hasattr(self, "list_rect"):
            self._build_rows()
            self._ensure_selection_visible()

    def _select(self, page):
        if page is not self.page:
            self.page = page
            self.art_scroll = 0

    def _page_rows(self):
        return [r for r in self.rows if r["kind"] == "page"]

    def _ensure_selection_visible(self):
        if not hasattr(self, "list_rect"):
            return
        for r in self.rows:
            if r["kind"] == "page" and r["page"] is self.page:
                if r["y"] < self.list_scroll:
                    self.list_scroll = r["y"]
                elif r["y"] + r["h"] > self.list_scroll + self.list_rect.h:
                    self.list_scroll = r["y"] + r["h"] - self.list_rect.h
                break

    def _max_art_scroll(self):
        if self.page is None:
            return 0
        _ops, content_h = self._article_layout(self.page)
        return max(0, content_h - (self.art_rect.h - 24))

    # ----- Eingabe ------------------------------------------------------------

    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            self._on_key(event.key)
        elif event.kind == InputEvent.MOUSEMOVE:
            self.hover = self._row_at(event.pos)
        elif event.kind == InputEvent.MOUSEDOWN:
            i = self._row_at(event.pos)
            if i is not None and self.rows[i]["kind"] == "page":
                self._select(self.rows[i]["page"])
                self._ensure_selection_visible()
                self.play_sound("click")
        elif event.kind == InputEvent.WHEEL:
            step = event.delta * 40
            if event.pos and event.pos[0] < self.list_rect.right:
                self.list_scroll = max(0, min(self.list_scroll - step,
                                              max(0, self.rows_h - self.list_rect.h)))
            else:
                self.art_scroll = max(0, min(self.art_scroll - step,
                                             self._max_art_scroll()))

    def _on_key(self, key):
        if key == "Escape":
            if self.query:
                self.query = ""
                self._apply_filter()
            else:
                self.on_close()
            return
        if key in ("Up", "Down"):
            pages = self._page_rows()
            if not pages:
                return
            idx = next((i for i, r in enumerate(pages)
                        if r["page"] is self.page), 0)
            idx = max(0, min(len(pages) - 1, idx + (1 if key == "Down" else -1)))
            self._select(pages[idx]["page"])
            self._ensure_selection_visible()
            self.play_sound("move")
        elif key in ("Prior", "Next"):       # Bild hoch / Bild runter
            step = int((self.art_rect.h - 24) * 0.8)
            step = step if key == "Next" else -step
            self.art_scroll = max(0, min(self.art_scroll + step,
                                         self._max_art_scroll()))
        elif key == "BackSpace":
            if self.query:
                self.query = self.query[:-1]
                self._apply_filter()
        elif key == "space":
            if self.query:                   # führende Leerzeichen vermeiden
                self.query += " "
                self._apply_filter()
        elif len(key) == 1 and key.isprintable():
            self.query += key.lower()
            self._apply_filter()

    def _row_at(self, pos):
        if pos is None or not self.list_rect.collidepoint(pos):
            return None
        y = pos[1] - self.list_rect.y + self.list_scroll
        for i, r in enumerate(self.rows):
            if r["y"] <= y < r["y"] + r["h"]:
                return i
        return None

    # ----- Artikel-Umbruch ------------------------------------------------------

    def _article_layout(self, page):
        """Bricht den Artikel auf die Panel-Innenbreite um (gecacht).

        Liefert (ops, content_h); ops = Liste von Zeichen-Anweisungen
        ("title"/"line"/"head"/"word"/"chip"/"dot", ...) mit relativen
        x/y-Positionen innerhalb des Artikel-Panels.
        """
        inner_w = self.art_rect.w - 32
        key = (page["id"], inner_w)
        cached = self._art_cache.get(key)
        if cached is not None:
            return cached

        f_title = ui.font(20, bold=True)
        f_head = ui.font(16, bold=True)
        f_body = ui.font(15)
        f_chip = ui.font(13, mono=True)

        ops = []
        y = 0
        title = page["title"]
        ops.append(("title", title, 0, y))
        y += f_title.get_height() + 4
        tw = min(f_title.size(title)[0] + 16, inner_w)
        ops.append(("line", tw, 0, y))
        y += 12

        for sect in page["sections"]:
            y += _SECT_GAP
            ops.append(("head", sect.get("h", ""), 0, y))
            y += f_head.get_height() + 6
            for body in sect.get("body", ()):
                is_bullet, tokens = parse_inline(body)
                x0 = _BULLET_INDENT if is_bullet else 0
                if is_bullet:
                    ops.append(("dot", None, 5, y + _LINE_H // 2))
                x = x0
                for tok in tokens:
                    if tok[0] == "word":
                        wpx = f_body.size(tok[1] + " ")[0]
                    else:
                        wpx = f_chip.size(tok[1])[0] + 14 + 6 \
                            + (f_body.size(tok[2])[0] if tok[2] else 0)
                    if x > x0 and x + wpx > inner_w:
                        x = x0
                        y += _LINE_H
                    ops.append((tok[0], tok, x, y))
                    x += wpx
                y += _LINE_H + _PARA_GAP
            y -= _PARA_GAP        # letzter Absatz einer Sektion ohne Extra-Lücke

        result = (ops, y + 8)
        if len(self._art_cache) > 24:
            self._art_cache.clear()
        self._art_cache[key] = result
        return result

    # ----- Zeichnen ---------------------------------------------------------------

    def draw(self):
        s = self.surface
        ui.draw_background(s, self.width, self.height, stars=False)

        # Kopfzeile mit Akzentstrich (Options-Stil)
        title_font = ui.font(22, bold=True)
        img = title_font.render(t("lamawiki.title"), True, ui.TEXT)
        s.blit(img, (16, 12))
        pygame.draw.rect(s, ui.ACCENT, (16, 14 + img.get_height(),
                                        img.get_width(), 3), border_radius=2)

        self._draw_search(s)
        self._draw_list(s)
        self._draw_article(s)
        ui.draw_footer(s, self.width, self.height, t("lamawiki.hint"))

    def _draw_search(self, s):
        r = self.search_rect
        ui.draw_panel(s, r, radius=8, shadow=False)
        if self.query:
            img = ui.font(14, mono=True).render(self.query, True, ui.TEXT)
        else:
            img = ui.font(14).render(t("lamawiki.search"), True, ui.TEXT_FAINT)
        prev = s.get_clip()
        s.set_clip(r.inflate(-8, 0))
        s.blit(img, img.get_rect(midleft=(r.x + 8, r.centery)))
        # Pulsierender Eingabe-Cursor hinter dem Text
        if ui.pulse(4.0) > 0.5:
            cx = min(r.right - 6, r.x + 8 + (img.get_width() if self.query else 0))
            pygame.draw.line(s, ui.ACCENT, (cx + 1, r.y + 5),
                             (cx + 1, r.bottom - 5))
        s.set_clip(prev)

    def _draw_list(self, s):
        lr = self.list_rect
        f_cat = ui.font(12, bold=True)
        f_row = ui.font(15)
        prev = s.get_clip()
        s.set_clip(lr)
        if not self.rows:
            img = f_row.render(t("lamawiki.no_results"), True, ui.TEXT_FAINT)
            s.blit(img, img.get_rect(center=lr.center))
        for i, r in enumerate(self.rows):
            y = lr.y + r["y"] - self.list_scroll
            if y + r["h"] < lr.y or y > lr.bottom:
                continue
            if r["kind"] == "cat":
                img = f_cat.render(r["label"], True, ui.ACCENT)
                s.blit(img, (lr.x + 2, y + 6))
                continue
            page = r["page"]
            row_rect = pygame.Rect(lr.x, y, lr.w - 6, r["h"] - 2)
            selected = page is self.page
            if selected:
                pygame.draw.rect(s, ui.PANEL_LIGHT, row_rect, border_radius=6)
                pygame.draw.rect(s, ui.ACCENT,
                                 (row_rect.x, row_rect.y + 3, 3, row_rect.h - 6),
                                 border_radius=2)
            elif i == self.hover:
                pygame.draw.rect(s, ui.BTN, row_rect, border_radius=6)
            x = lr.x + 10
            if page.get("game"):
                pygame.draw.circle(s, ui.game_color(page["game"]),
                                   (x + 3, row_rect.centery), 4)
                x += 14
            img = f_row.render(page["title"], True,
                               ui.TEXT if selected else ui.TEXT_DIM)
            s.blit(img, img.get_rect(midleft=(x, row_rect.centery)))
        s.set_clip(prev)
        # Scroll-Indikator rechts an der Liste
        if self.rows_h > lr.h:
            f0 = self.list_scroll / self.rows_h
            f1 = (self.list_scroll + lr.h) / self.rows_h
            pygame.draw.rect(s, ui.BORDER_LIGHT,
                             (lr.right - 3, lr.y + int(f0 * lr.h), 3,
                              max(12, int((f1 - f0) * lr.h))), border_radius=2)

    def _draw_article(self, s):
        ar = self.art_rect
        if self.page is None:
            ui.draw_panel(s, ar, radius=10, shadow=False)
            return
        accent = ui.game_color(self.page["game"]) if self.page.get("game") \
            else ui.ACCENT
        ui.draw_panel(s, ar, radius=10, shadow=False,
                      accent_top=ui.mix(ui.PANEL, accent, 0.45))

        ops, content_h = self._article_layout(self.page)
        inner = ar.inflate(-32, -24)
        f_title = ui.font(20, bold=True)
        f_head = ui.font(16, bold=True)
        f_body = ui.font(15)
        f_chip = ui.font(13, mono=True)

        prev = s.get_clip()
        s.set_clip(inner)
        oy = inner.y - self.art_scroll
        for op in ops:
            kind, payload, x, y = op
            yy = oy + y
            if yy > inner.bottom or yy + _LINE_H + 10 < inner.y:
                continue
            xx = inner.x + x
            if kind == "title":
                s.blit(f_title.render(payload, True, ui.TEXT), (xx, yy))
            elif kind == "line":
                pygame.draw.rect(s, accent, (xx, yy, payload, 3),
                                 border_radius=2)
            elif kind == "head":
                s.blit(f_head.render(payload, True, accent), (xx, yy))
            elif kind == "dot":
                pygame.draw.circle(s, accent, (xx, yy), 3)
            elif kind == "word":
                img = f_body.render(payload[1], True, ui.TEXT_DIM)
                # An der Unterkante ausrichten: Glyphen mit Diakritika (Ä/É)
                # liefern höhere Surfaces und sähen sonst "abgesackt" aus.
                s.blit(img, (xx, yy + f_body.get_height() - img.get_height()))
            elif kind == "chip":
                _tok, label, trail = payload
                img = f_chip.render(label, True, ui.GOLD)
                chip = pygame.Rect(xx, yy + 1, img.get_width() + 14,
                                   _LINE_H - 4)
                pygame.draw.rect(s, ui.PANEL_LIGHT, chip, border_radius=5)
                pygame.draw.rect(s, ui.BORDER_LIGHT, chip, 1, border_radius=5)
                s.blit(img, img.get_rect(center=chip.center))
                if trail:
                    timg = f_body.render(trail, True, ui.TEXT_DIM)
                    s.blit(timg, (chip.right + 1,
                                  yy + f_body.get_height() - timg.get_height()))
        s.set_clip(prev)

        # Scroll-Indikator rechts am Panel
        view_h = inner.h
        if content_h > view_h:
            f0 = self.art_scroll / content_h
            f1 = (self.art_scroll + view_h) / content_h
            pygame.draw.rect(s, ui.BORDER_LIGHT,
                             (ar.right - 7, ar.y + 6 + int(f0 * (ar.h - 12)), 3,
                              max(16, int((f1 - f0) * (ar.h - 12)))),
                             border_radius=2)
