# -*- coding: utf-8 -*-
"""
charts.py
=========
Kleine, theme-treue Diagramme für die Statistik-Screens - reines pygame,
keine Fremdbibliothek.

Alle Funktionen zeichnen in ein ``pygame.Rect`` und geben die Trefferflächen
zurück ``[(rect, daten), ...]``, damit der aufrufende Screen daraus Hover und
Tooltips bauen kann (siehe ``tooltip()``). Farben kommen aus ui.py, die
Diagramme passen sich damit jedem Design an.

Enthalten sind:

- ``bars()``          Balken (senkrecht oder waagerecht) mit Werte-Labels
- ``line()``          Linienverlauf mit Rekord-Markern und Grundlinien
- ``calendar_heat()`` Spieltage-Karte im GitHub-Stil (7 Reihen x Wochen)
- ``donut()``         Ringdiagramm für Anteile
- ``clock_ring()``    Tageszeit-Profil (24 Speichen)
- ``tooltip()``       kleiner Hinweiskasten an der Maus
"""

import math

import pygame

import ui


def _fnt(size, bold=False):
    return ui.font(size, bold=bold)


def _grid_color():
    return ui.mix(ui.PANEL, ui.BORDER_LIGHT, 0.6)


# Kurznamen für Wochentage/Monate stehen als Komma-Liste in den Sprachdateien.
# Fehlt der Schlüssel (oder ist die Liste zu kurz), greifen diese Rückfälle -
# so bleibt ein Diagramm auch mit unvollständiger Übersetzung lesbar.
_FALLBACK_WEEKDAYS = ("Mo", "Tu", "We", "Th", "Fr", "Sa", "Su")
_FALLBACK_MONTHS = ("Jan", "Feb", "Mar", "Apr", "May", "Jun",
                    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec")


def name_list(key, expected, fallback):
    """Komma-Liste aus den Sprachdateien (mit Rückfall auf 'fallback')."""
    from i18n import t
    parts = [p.strip() for p in t(key).split(",") if p.strip()]
    if len(parts) < expected:
        return list(fallback)
    return parts[:expected]


def weekday_names():
    return name_list("cal.weekdays_short", 7, _FALLBACK_WEEKDAYS)


def month_names():
    return name_list("cal.months_short", 12, _FALLBACK_MONTHS)


# ---------------------------------------------------------------------------
#  Balken
# ---------------------------------------------------------------------------

def bars(surface, rect, values, labels=None, colors=None, color=None,
         max_value=None, horizontal=False, fmt=None, hover=None,
         label_size=10):
    """Balkendiagramm. 'values' sind Zahlen, 'labels' die Beschriftungen.

    colors : Liste eigener Farben je Balken (sonst 'color' bzw. Akzentfarbe)
    fmt    : Funktion Wert -> Text für die Werte-Beschriftung
    hover  : Index des hervorgehobenen Balkens
    Rückgabe: [(rect, index), ...]
    """
    hits = []
    if not values:
        return hits
    base = color or ui.ACCENT
    top = max_value if max_value is not None else max(values)
    top = max(top, 1e-6)
    n = len(values)
    fnt = _fnt(label_size)
    val_fnt = _fnt(label_size, bold=True)

    if horizontal:
        label_w = 0
        if labels:
            label_w = min(rect.w // 3,
                          max(fnt.size(str(s))[0] for s in labels) + 8)
        gap = 4
        row_h = max(10, (rect.h - gap * (n - 1)) // n)
        for i, value in enumerate(values):
            y = rect.y + i * (row_h + gap)
            col = (colors[i] if colors else base)
            if labels and i < len(labels):
                img = fnt.render(str(labels[i]), True,
                                 ui.TEXT if i == hover else ui.TEXT_DIM)
                surface.blit(img, img.get_rect(
                    midleft=(rect.x, y + row_h // 2)),
                    (0, 0, label_w - 6, img.get_height()))
            bx = rect.x + label_w
            bw = max(0, rect.right - bx)
            track = pygame.Rect(bx, y, bw, row_h)
            pygame.draw.rect(surface, ui.mix(ui.PANEL, ui.BG_TOP, 0.4), track,
                             border_radius=row_h // 2)
            fill = int(bw * (value / top))
            if fill > 1:
                pygame.draw.rect(surface, col, (bx, y, fill, row_h),
                                 border_radius=row_h // 2)
            if fmt:
                img = val_fnt.render(fmt(value), True, ui.TEXT_DIM)
                surface.blit(img, img.get_rect(
                    midright=(rect.right - 6, y + row_h // 2)))
            hits.append((track, i))
        return hits

    gap = max(2, min(8, rect.w // (n * 5)))
    bar_w = max(3, (rect.w - gap * (n - 1)) // n)
    label_h = fnt.get_height() + 2 if labels else 0
    top_pad = val_fnt.get_height() + 2
    plot_h = max(8, rect.h - label_h - top_pad)
    for i, value in enumerate(values):
        x = rect.x + i * (bar_w + gap)
        h = int(plot_h * (value / top))
        y = rect.y + top_pad + (plot_h - h)
        col = (colors[i] if colors else base)
        if i == hover:
            col = ui.mix(col, (255, 255, 255), 0.35)
        cell = pygame.Rect(x, rect.y + top_pad, bar_w, plot_h)
        pygame.draw.rect(surface, ui.mix(ui.PANEL, ui.BG_TOP, 0.35), cell,
                         border_radius=3)
        if h > 1:
            pygame.draw.rect(surface, col, (x, y, bar_w, h), border_radius=3)
        if labels and i < len(labels):
            img = fnt.render(str(labels[i]), True,
                             ui.TEXT if i == hover else ui.TEXT_FAINT)
            surface.blit(img, img.get_rect(
                midtop=(x + bar_w // 2, rect.bottom - label_h + 1)))
        if fmt and (i == hover or n <= 12):
            img = val_fnt.render(fmt(value), True, ui.TEXT_DIM)
            surface.blit(img, img.get_rect(midbottom=(x + bar_w // 2,
                                                      y - 1)))
        hits.append((cell, i))
    return hits


# ---------------------------------------------------------------------------
#  Linienverlauf
# ---------------------------------------------------------------------------

def line(surface, rect, values, color=None, marks=None, fmt=None, hover=None,
         fill=True):
    """Linienverlauf über 'values' (Zahlen in zeitlicher Reihenfolge).

    marks : Indizes, die als Rekord hervorgehoben werden
    Rückgabe: [(rect, index), ...] (schmale Trefferspalten für den Tooltip)
    """
    hits = []
    if not values:
        return hits
    col = color or ui.ACCENT
    marks = set(marks or ())
    lo, hi = min(values), max(values)
    if hi <= lo:
        hi = lo + 1
    n = len(values)

    # Grundraster: drei waagerechte Linien mit Werte-Beschriftung.
    fnt = _fnt(10)
    for frac in (0.0, 0.5, 1.0):
        y = int(rect.bottom - frac * rect.h)
        pygame.draw.line(surface, _grid_color(), (rect.x, y), (rect.right, y))
        if fmt:
            img = fnt.render(fmt(lo + (hi - lo) * frac), True, ui.TEXT_FAINT)
            surface.blit(img, img.get_rect(bottomleft=(rect.x + 2, y - 1)))

    step = rect.w / max(1, n - 1) if n > 1 else 0
    points = []
    for i, value in enumerate(values):
        x = rect.x + (i * step if n > 1 else rect.w / 2)
        y = rect.bottom - (value - lo) / (hi - lo) * rect.h
        points.append((int(x), int(y)))

    if fill and len(points) > 1:
        poly = points + [(points[-1][0], rect.bottom), (points[0][0], rect.bottom)]
        shade = pygame.Surface((rect.w + 2, rect.h + 2), pygame.SRCALPHA)
        pygame.draw.polygon(shade, (*col, 46),
                            [(px - rect.x, py - rect.y) for px, py in poly])
        surface.blit(shade, (rect.x, rect.y))
    if len(points) > 1:
        pygame.draw.lines(surface, col, False, points, 2)
    for i, (x, y) in enumerate(points):
        if i in marks:
            pygame.draw.circle(surface, ui.GOLD, (x, y), 4)
            pygame.draw.circle(surface, ui.mix(ui.GOLD, (255, 255, 255), 0.5),
                               (x, y), 2)
        elif n <= 60:
            pygame.draw.circle(surface, col, (x, y), 2)
        if i == hover:
            pygame.draw.circle(surface, ui.TEXT, (x, y), 5, 1)
        cell = pygame.Rect(int(x - max(2, step / 2)), rect.y,
                           max(4, int(step) or 4), rect.h)
        hits.append((cell, i))
    return hits


# ---------------------------------------------------------------------------
#  Kalender-Heatmap (Spieltage)
# ---------------------------------------------------------------------------

def calendar_heat(surface, rect, days, color=None, weeks=None, today=None,
                  month_labels=True, hover=None):
    """Spieltage-Karte: eine Spalte je Woche, sieben Kästchen je Spalte.

    days  : {"YYYY-MM-DD": wert} (Wert = Spielzeit in Sekunden)
    weeks : Anzahl Wochen (None = so viele wie hineinpassen)
    Rückgabe: [(rect, (datum, wert)), ...]
    """
    import datetime
    hits = []
    col = color or ui.ACCENT
    today = today or datetime.date.today()
    fnt = _fnt(9)
    label_h = fnt.get_height() + 2 if month_labels else 0
    day_label_w = fnt.size("Mo")[0] + 4 if month_labels else 0

    grid = pygame.Rect(rect.x + day_label_w, rect.y + label_h,
                       rect.w - day_label_w, rect.h - label_h)
    cell = max(6, min(16, grid.h // 7 - 2))
    gap = max(1, cell // 6)
    per_week = cell + gap
    fit = max(1, (grid.w + gap) // per_week)
    weeks = min(weeks or fit, fit)

    # Letzte Spalte = aktuelle Woche (Montag als Wochenstart).
    end = today - datetime.timedelta(days=today.weekday())
    start = end - datetime.timedelta(weeks=weeks - 1)
    top = max(days.values()) if days else 1.0
    top = max(top, 1.0)

    month_seen = set()
    for w in range(weeks):
        week_start = start + datetime.timedelta(weeks=w)
        x = grid.x + w * per_week
        for d in range(7):
            date = week_start + datetime.timedelta(days=d)
            if date > today:
                continue
            key = date.isoformat()
            value = days.get(key, 0.0)
            y = grid.y + d * per_week
            box = pygame.Rect(x, y, cell, cell)
            if value > 0:
                # Vier Stufen - wie beim Vorbild aus der Versionsverwaltung.
                frac = min(1.0, (value / top) ** 0.45)
                fill_col = ui.mix(ui.mix(ui.PANEL, col, 0.25), col, frac)
                pygame.draw.rect(surface, fill_col, box, border_radius=2)
            else:
                pygame.draw.rect(surface, ui.mix(ui.PANEL, ui.BG_TOP, 0.35),
                                 box, border_radius=2)
            if date == today:
                pygame.draw.rect(surface, ui.GOLD, box, 1, border_radius=2)
            if hover is not None and hover == key:
                pygame.draw.rect(surface, ui.TEXT, box, 1, border_radius=2)
            hits.append((box, (key, value)))
        if month_labels and week_start.month not in month_seen:
            month_seen.add(week_start.month)
            if w == 0 or week_start.day <= 7:
                img = fnt.render(_month_short(week_start.month), True,
                                 ui.TEXT_FAINT)
                surface.blit(img, (x, rect.y))
    if month_labels:
        short = weekday_names()
        for d in (0, 2, 4):
            if d < len(short):
                img = fnt.render(short[d], True, ui.TEXT_FAINT)
                surface.blit(img, (rect.x, grid.y + d * per_week - 1))
    return hits


def _month_short(month):
    names = month_names()
    return names[month - 1] if 1 <= month <= len(names) else str(month)


def heat_legend(surface, rect, color=None, label_less="", label_more=""):
    """Kleine Legende (wenig -> viel) unter der Kalender-Karte."""
    col = color or ui.ACCENT
    fnt = _fnt(9)
    x = rect.x
    if label_less:
        img = fnt.render(label_less, True, ui.TEXT_FAINT)
        surface.blit(img, img.get_rect(midleft=(x, rect.centery)))
        x += img.get_width() + 6
    size = min(10, rect.h)
    for frac in (0.0, 0.33, 0.66, 1.0):
        box = pygame.Rect(x, rect.centery - size // 2, size, size)
        if frac <= 0:
            pygame.draw.rect(surface, ui.mix(ui.PANEL, ui.BG_TOP, 0.35), box,
                             border_radius=2)
        else:
            pygame.draw.rect(surface, ui.mix(ui.mix(ui.PANEL, col, 0.25), col,
                                             frac), box, border_radius=2)
        x += size + 3
    if label_more:
        img = fnt.render(label_more, True, ui.TEXT_FAINT)
        surface.blit(img, img.get_rect(midleft=(x + 3, rect.centery)))


# ---------------------------------------------------------------------------
#  Ringe
# ---------------------------------------------------------------------------

def donut(surface, center, radius, slices, thickness=None, center_text=None,
          center_sub=None):
    """Ringdiagramm. 'slices' = [(wert, farbe), ...].

    Gibt die gezeichneten Segmente als [(start, end, farbe), ...] zurück.
    """
    cx, cy = int(center[0]), int(center[1])
    r = int(radius)
    thickness = thickness or max(6, r // 3)
    total = sum(max(0.0, float(v)) for v, _c in slices)
    box = pygame.Rect(cx - r, cy - r, r * 2, r * 2)
    pygame.draw.circle(surface, ui.mix(ui.PANEL, ui.BG_TOP, 0.35), (cx, cy), r,
                       thickness)
    out = []
    if total > 0:
        start = -math.pi / 2
        for value, col in slices:
            if value <= 0:
                continue
            end = start + 2 * math.pi * (value / total)
            # Etwas überzeichnen, damit keine Lücken zwischen den Segmenten
            # entstehen (pygame zeichnet Bögen kantig).
            pygame.draw.arc(surface, col, box, -end, -start, thickness)
            out.append((start, end, col))
            start = end
    if center_text:
        img = _fnt(max(12, r // 3), bold=True).render(center_text, True,
                                                      ui.TEXT)
        surface.blit(img, img.get_rect(center=(cx, cy - (3 if center_sub else 0))))
        if center_sub:
            sub = _fnt(max(9, r // 5)).render(center_sub, True, ui.TEXT_DIM)
            surface.blit(sub, sub.get_rect(center=(cx, cy + img.get_height() // 2 + 4)))
    return out


def clock_ring(surface, center, radius, hours, color=None, hover=None):
    """Tageszeit-Profil: 24 Speichen, je länger desto mehr Spielzeit.

    Rückgabe: [(rect, stunde), ...] für den Tooltip.
    """
    cx, cy = int(center[0]), int(center[1])
    r = int(radius)
    col = color or ui.ACCENT
    top = max(hours) if hours else 0.0
    hits = []
    inner = max(8, r // 3)
    pygame.draw.circle(surface, _grid_color(), (cx, cy), r, 1)
    pygame.draw.circle(surface, _grid_color(), (cx, cy), inner, 1)
    fnt = _fnt(9)
    for h in range(24):
        ang = -math.pi / 2 + h * math.pi / 12
        frac = (hours[h] / top) if top > 0 else 0.0
        length = inner + (r - inner) * frac
        x1, y1 = cx + inner * math.cos(ang), cy + inner * math.sin(ang)
        x2, y2 = cx + length * math.cos(ang), cy + length * math.sin(ang)
        spoke = col if h != hover else ui.mix(col, (255, 255, 255), 0.4)
        if frac > 0:
            pygame.draw.line(surface, spoke, (x1, y1), (x2, y2), 5)
        box = pygame.Rect(0, 0, 14, 14)
        box.center = (cx + (r - 6) * math.cos(ang), cy + (r - 6) * math.sin(ang))
        hits.append((box, h))
    for h in (0, 6, 12, 18):
        ang = -math.pi / 2 + h * math.pi / 12
        img = fnt.render(str(h), True, ui.TEXT_FAINT)
        surface.blit(img, img.get_rect(
            center=(cx + (r + 9) * math.cos(ang), cy + (r + 9) * math.sin(ang))))
    return hits


# ---------------------------------------------------------------------------
#  Tooltip
# ---------------------------------------------------------------------------

def tooltip(surface, pos, lines, width_limit=None):
    """Kleiner Hinweiskasten neben der Maus (erste Zeile fett)."""
    lines = [l for l in lines if l]
    if not lines:
        return
    head = _fnt(11, bold=True)
    body = _fnt(10)
    imgs = [head.render(lines[0], True, ui.TEXT)]
    for text in lines[1:]:
        imgs.append(body.render(text, True, ui.TEXT_DIM))
    w = max(i.get_width() for i in imgs) + 16
    h = sum(i.get_height() for i in imgs) + 12
    limit = width_limit or surface.get_width()
    x = min(pos[0] + 14, limit - w - 4)
    y = max(4, pos[1] - h - 10)
    rect = pygame.Rect(max(4, x), y, w, h)
    ui.draw_panel(surface, rect, color=ui.PANEL_LIGHT, radius=8, shadow=False)
    ty = rect.y + 6
    for img in imgs:
        surface.blit(img, (rect.x + 8, ty))
        ty += img.get_height()
