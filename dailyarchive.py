# -*- coding: utf-8 -*-
"""
dailyarchive.py
===============
Tages-Archiv für die Tagesmodi (Crossy Road: Tagesstrecke, Sudoku:
Tagesrätsel, Wordle: Tageswort).

Der Tages-Seed hängt nur vom Datum ab (``seedrand.daily_seed(spiel, datum)``,
in Python, JS und Rust bitgleich) - jeder vergangene Tag lässt sich also
exakt neu erzeugen. Nachspielbar sind:

* die letzten ``WINDOW`` Tage (heute eingeschlossen) und
* jeder ältere Tag, den der Spieler schon gespielt hat.

Gespeichert wird kompakt in der Section des Spiels als Liste von Zeilen,
deren erstes Feld die Tagesnummer ist (``seedrand.day_index``, Tage seit
2026-01-01) - statt eines Datumstexts je Eintrag::

    "arch": [[329, 57, 3], [330, 112, 1], ...]      # Crossy: Tag, Bestwert, Versuche

Die übrigen Felder bestimmt das Spiel. Die Liste bleibt nach Tag sortiert.

``Picker`` ist das Auswahlfenster (Kalender der letzten 30 Tage + Leiste der
älteren gespielten Tage), das die drei Spiele über ihrem Setup zeigen.
Gegenstücke: ``web/js/core/dailyarchive.js``, ``rust/.../dailyarchive.rs``.
"""

import datetime

import pygame

import charts
import seedrand
import ui
from game_base import InputEvent
from i18n import t

# Tage im Kalender (heute eingeschlossen)
WINDOW = 30
# Höchstens so viele Tage merkt sich ein Archiv (gut 5 Jahre - Schutz gegen
# beschädigte Dateien, im Alltag nie erreicht).
MAX_DAYS = 2000


# ---------------------------------------------------------------------------
#  Tage & Daten
# ---------------------------------------------------------------------------

def today_index():
    return seedrand.day_index()


def window_days(today=None):
    """Die letzten WINDOW Tage als Tagesnummern, älteste zuerst."""
    today = today_index() if today is None else today
    return list(range(today - WINDOW + 1, today + 1))


def available(played, today=None):
    """Alle wählbaren Tage (Fenster + ältere gespielte), älteste zuerst."""
    today = today_index() if today is None else today
    days = set(window_days(today))
    for d in played:
        if isinstance(d, int) and not isinstance(d, bool) and 0 <= d <= today:
            days.add(d)
    return sorted(days)


def clean_rows(raw, width, today=None):
    """Archiv-Zeilen prüfen: [tag, ...] mit genau 'width' Feldern, sortiert.

    Ungültige Zeilen fallen weg, doppelte Tage: die letzte gewinnt. Tage in
    der Zukunft (Uhr verstellt) bleiben erhalten, werden aber nicht angeboten.
    """
    if not isinstance(raw, list):
        return []
    rows = {}
    for row in raw:
        if not isinstance(row, list) or len(row) != width:
            continue
        day = row[0]
        if not isinstance(day, int) or isinstance(day, bool) or day < 0:
            continue
        rows[day] = list(row)
    out = [rows[d] for d in sorted(rows)]
    return out[-MAX_DAYS:]


def find(rows, day):
    """Zeile eines Tages oder None."""
    for row in rows:
        if row[0] == day:
            return row
    return None


def upsert(rows, row):
    """Zeile eintragen/ersetzen (Liste bleibt nach Tag sortiert)."""
    day = row[0]
    for i, old in enumerate(rows):
        if old[0] == day:
            rows[i] = row
            return rows
        if old[0] > day:
            rows.insert(i, row)
            return rows
    rows.append(row)
    if len(rows) > MAX_DAYS:
        del rows[:len(rows) - MAX_DAYS]
    return rows


def date_str(day):
    return seedrand.date_of(day)


def fit_text(fnt, text, room):
    """Kürzt einen Text mit '…', bis er in 'room' Pixel passt."""
    if fnt.size(text)[0] <= room:
        return text
    while text and fnt.size(text + "…")[0] > room:
        text = text[:-1]
    return text.rstrip() + "…" if text else ""


def chip_date(day):
    """'23.11.25' (kurze Leiste der älteren Tage)."""
    d = datetime.date.fromisoformat(date_str(day))
    return "%02d.%02d.%02d" % (d.day, d.month, d.year % 100)


def short_date(day):
    """'26.11.' (für Chips und HUD)."""
    d = datetime.date.fromisoformat(date_str(day))
    return "%02d.%02d." % (d.day, d.month)


def long_date(day):
    """'Mi, 26.11.2026' (Wochentag aus den Sprachdateien)."""
    d = datetime.date.fromisoformat(date_str(day))
    wd = charts.weekday_names()[d.weekday()]
    return "%s, %02d.%02d.%04d" % (wd, d.day, d.month, d.year)


def day_label(day, today=None):
    """'Heute · 26.11.' / 'Gestern · 25.11.' / 'Mo, 23.11.2026'."""
    today = today_index() if today is None else today
    if day == today:
        return t("arch.today", date=short_date(day))
    if day == today - 1:
        return t("arch.yesterday", date=short_date(day))
    return long_date(day)


def seed_hex(game, day):
    """Seed des Tages als 8 Hex-Zeichen (Anzeige)."""
    return "%08X" % seedrand.daily_seed(game, date_str(day))


# ---------------------------------------------------------------------------
#  Datums-Leiste im Setup:  [◀] [ Heute · 26.11. ] [▶] [Archiv]
# ---------------------------------------------------------------------------

def day_row_rects(x, y, w, h):
    """Rechtecke (zurück, Datum, vor, Archiv) einer Datums-Leiste.

    Schmale Leisten (unter 300 px) bekommen einen quadratischen
    Archiv-Knopf (nur Symbol), damit das Datum lesbar bleibt.
    """
    gap = 6
    arrow = min(h, max(18, w // 12))
    arch = h if w < 300 else max(90, w // 4)
    mid = w - 2 * arrow - arch - 3 * gap
    r_prev = pygame.Rect(x, y, arrow, h)
    r_date = pygame.Rect(r_prev.right + gap, y, mid, h)
    r_next = pygame.Rect(r_date.right + gap, y, arrow, h)
    r_arch = pygame.Rect(r_next.right + gap, y, x + w - (r_next.right + gap), h)
    return r_prev, r_date, r_next, r_arch


def draw_triangle(s, rect, direction, color):
    cx, cy = rect.center
    k = max(4, rect.h // 5)
    if direction < 0:
        pts = [(cx + k // 2, cy - k), (cx + k // 2, cy + k), (cx - k, cy)]
    else:
        pts = [(cx - k // 2, cy - k), (cx - k // 2, cy + k), (cx + k, cy)]
    pygame.draw.polygon(s, color, pts)


def draw_day_row(s, rects, day, accent, can_prev, can_next, fnt, archived=False):
    """Zeichnet die Datums-Leiste (Pfeile ausgegraut, wenn es nicht weitergeht).

    Passt "Gestern · 25.11." bzw. "Mo, 23.11.2026" nicht, steht nur das kurze
    Datum da.
    """
    r_prev, r_date, r_next, r_arch = rects
    label = day_label(day)
    if fnt.size(label)[0] > r_date.w - 10:
        label = short_date(day)
    for rc, d, on in ((r_prev, -1, can_prev), (r_next, 1, can_next)):
        pygame.draw.rect(s, ui.BTN, rc, border_radius=8)
        pygame.draw.rect(s, ui.BORDER, rc, 1, border_radius=8)
        draw_triangle(s, rc, d, ui.TEXT if on else ui.TEXT_FAINT)
    pygame.draw.rect(s, ui.BTN_SEL if archived else ui.BTN, r_date, border_radius=8)
    pygame.draw.rect(s, ui.GOLD if archived else accent, r_date, 1, border_radius=8)
    img = fnt.render(fit_text(fnt, label, r_date.w - 10), True,
                     ui.GOLD if archived else ui.TEXT)
    s.blit(img, img.get_rect(center=r_date.center))
    pygame.draw.rect(s, ui.BTN, r_arch, border_radius=8)
    pygame.draw.rect(s, accent, r_arch, 1, border_radius=8)
    # Kalender-Symbol + Text (bei schmalem Knopf nur das Symbol)
    ic = pygame.Rect(0, 0, max(10, r_arch.h // 2), max(10, r_arch.h // 2))
    lab = fnt.render(t("arch.button"), True, ui.TEXT)
    total = ic.w + 6 + lab.get_width()
    show_lab = total <= r_arch.w - 10
    if not show_lab:
        total = ic.w
    ic.midleft = (r_arch.centerx - total // 2, r_arch.centery)
    pygame.draw.rect(s, ui.TEXT_DIM, ic, 1, border_radius=2)
    pygame.draw.line(s, ui.TEXT_DIM, (ic.x, ic.y + ic.h // 3), (ic.right - 1, ic.y + ic.h // 3))
    if show_lab:
        s.blit(lab, lab.get_rect(midleft=(ic.right + 6, r_arch.centery)))


# ---------------------------------------------------------------------------
#  Auswahlfenster
# ---------------------------------------------------------------------------

class Picker:
    """Kalender der letzten WINDOW Tage + ältere gespielte Tage.

    ``info``: {tag: (zustand, wert_text)} - zustand ist "solved" (gelöst/
    geschafft), "played" (gespielt, nicht gelöst), "started" (angefangen)
    oder fehlt (offen). ``handle_event`` liefert ("pick", tag), ("close",
    None) oder None.
    """

    def __init__(self, game_key, accent, sel, info, today=None):
        self.game_key = game_key
        self.accent = accent
        self.today = today_index() if today is None else today
        self.info = dict(info)
        self.days = available(self.info.keys(), self.today)
        self.window = window_days(self.today)
        self.older = [d for d in self.days if d < self.window[0]]
        self.sel = sel if sel in self.days else self.today
        self.hover = None
        self.rect = None

    # ----- Layout -------------------------------------------------------

    def layout(self, W, H):
        pw = min(640, W - 32)
        ph = min(H - 32, 470)
        self.rect = pygame.Rect(W // 2 - pw // 2, H // 2 - ph // 2, pw, ph)
        r = self.rect
        top = r.y + 64
        older_h = 54 if self.older else 0
        foot = 54
        grid_h = r.bottom - foot - older_h - top - 22
        # Wochen-Raster: Montag der Woche des ersten Fenstertags
        first = datetime.date.fromisoformat(date_str(self.window[0]))
        start = self.window[0] - first.weekday()
        weeks = (self.today - start) // 7 + 1
        cw = (r.w - 32 - 6 * 6) // 7
        ch = max(26, min(54, (grid_h - (weeks - 1) * 5) // max(1, weeks)))
        gx = r.x + (r.w - (7 * cw + 6 * 6)) // 2
        self.head_y = top
        self.cells = []
        for w_i in range(weeks):
            for d_i in range(7):
                day = start + w_i * 7 + d_i
                rc = pygame.Rect(gx + d_i * (cw + 6), top + 18 + w_i * (ch + 5), cw, ch)
                self.cells.append((rc, day if day in self.window else None))
        self.grid_x, self.cell_w = gx, cw
        # Leiste der älteren gespielten Tage (neueste zuerst)
        self.chips = []
        if self.older:
            y = top + 18 + weeks * (ch + 5) + 22
            x = r.x + 16
            fnt = ui.font(11, bold=True)
            for day in reversed(self.older):
                w = fnt.size(chip_date(day))[0] + 18
                if x + w > r.right - 16:
                    break
                self.chips.append((pygame.Rect(x, y, w, 24), day))
                x += w + 6
            self.older_y = y - 14

    # ----- Eingabe ------------------------------------------------------

    def _step(self, d):
        i = self.days.index(self.sel)
        self.sel = self.days[max(0, min(len(self.days) - 1, i + d))]

    def handle_event(self, event, game=None):
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k == "Escape":
                return ("close", None)
            if k in ("Return", "space", "KP_Enter"):
                return ("pick", self.sel)
            if k == "Left" or (game and game.is_action(k, "left")):
                self._step(-1)
            elif k == "Right" or (game and game.is_action(k, "right")):
                self._step(1)
            elif k == "Up" or (game and game.is_action(k, "up")):
                if self.sel - 7 in self.days:
                    self.sel -= 7
                else:
                    self._step(-1)
            elif k == "Down" or (game and game.is_action(k, "down")):
                if self.sel + 7 in self.days:
                    self.sel += 7
                else:
                    self._step(1)
            elif k == "Home":
                self.sel = self.days[0]
            elif k == "End":
                self.sel = self.today
            return None
        if event.kind == InputEvent.MOUSEMOVE:
            self.hover = self._hit(event.pos)
            if self.hover is not None:
                self.sel = self.hover
        elif event.kind == InputEvent.MOUSEDOWN and event.button == 1:
            day = self._hit(event.pos)
            if day is not None:
                return ("pick", day)
            if self.rect is not None and not self.rect.collidepoint(event.pos):
                return ("close", None)
        return None

    def _hit(self, pos):
        for rc, day in self.cells:
            if day is not None and rc.collidepoint(pos):
                return day
        for rc, day in self.chips:
            if rc.collidepoint(pos):
                return day
        return None

    # ----- Zeichnen -----------------------------------------------------

    def _state_color(self, state):
        return {"solved": ui.GREEN, "played": ui.GOLD,
                "started": self.accent}.get(state, ui.TEXT_FAINT)

    def draw(self, s):
        W, H = s.get_size()
        if self.rect is None:
            self.layout(W, H)
        shade = pygame.Surface((W, H), pygame.SRCALPHA)
        shade.fill((0, 0, 0, 150))
        s.blit(shade, (0, 0))
        r = self.rect
        ui.draw_panel(s, r, radius=14)
        pygame.draw.rect(s, self.accent, r, 1, border_radius=14)
        title = ui.font(20, bold=True).render(t("arch.title"), True, ui.TEXT)
        s.blit(title, title.get_rect(midtop=(r.centerx, r.y + 12)))
        sub = ui.font(12).render(t("arch.sub", n=WINDOW), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(midtop=(r.centerx, r.y + 40)))
        # Wochentage
        names = charts.weekday_names()
        hf = ui.font(11, bold=True)
        for i in range(7):
            img = hf.render(names[i], True, ui.TEXT_FAINT)
            cx = self.grid_x + i * (self.cell_w + 6) + self.cell_w // 2
            s.blit(img, img.get_rect(midtop=(cx, self.head_y)))
        nf = ui.font(14, bold=True)
        vf = ui.font(10)
        for rc, day in self.cells:
            if day is None:
                continue
            state, value = self.info.get(day, (None, ""))
            sel = day == self.sel
            pygame.draw.rect(s, ui.BTN_SEL if sel else ui.BTN, rc, border_radius=7)
            border = self.accent if sel else (ui.GOLD if day == self.today else ui.BORDER)
            pygame.draw.rect(s, border, rc, 2 if (sel or day == self.today) else 1,
                             border_radius=7)
            num = datetime.date.fromisoformat(date_str(day)).day
            img = nf.render(str(num), True, ui.TEXT if state else ui.TEXT_DIM)
            s.blit(img, (rc.x + 6, rc.y + 3))
            if state:
                pygame.draw.circle(s, self._state_color(state), (rc.right - 8, rc.y + 9), 4)
            if value and rc.h >= 34:
                vi = vf.render(fit_text(vf, str(value), rc.w - 8), True,
                               self._state_color(state))
                s.blit(vi, vi.get_rect(bottomleft=(rc.x + 5, rc.bottom - 3)))
        if self.older:
            lab = ui.font(11, bold=True).render(t("arch.older", n=len(self.older)), True,
                                                ui.TEXT_FAINT)
            s.blit(lab, (r.x + 16, self.older_y))
            cf = ui.font(11, bold=True)
            for rc, day in self.chips:
                state, _ = self.info.get(day, (None, ""))
                sel = day == self.sel
                pygame.draw.rect(s, ui.BTN_SEL if sel else ui.BTN, rc, border_radius=12)
                pygame.draw.rect(s, self.accent if sel else self._state_color(state), rc, 1,
                                 border_radius=12)
                img = cf.render(chip_date(day), True, ui.TEXT)
                s.blit(img, img.get_rect(center=rc.center))
        # Detailzeile des gewählten Tags + Hinweis
        state, value = self.info.get(self.sel, (None, ""))
        parts = [day_label(self.sel, self.today), t("arch.seed", s=seed_hex(self.game_key, self.sel))]
        parts.append(t("arch.state." + (state or "open")))
        if value:
            parts.append(str(value))
        df = ui.font(13, bold=True)
        det = df.render(fit_text(df, "  ·  ".join(parts), r.w - 24), True,
                        self._state_color(state) if state else ui.TEXT)
        s.blit(det, det.get_rect(midbottom=(r.centerx, r.bottom - 30)))
        hint = ui.font(11).render(t("arch.hint"), True, ui.TEXT_FAINT)
        s.blit(hint, hint.get_rect(midbottom=(r.centerx, r.bottom - 10)))
