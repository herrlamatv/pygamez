# -*- coding: utf-8 -*-
"""
minigolf_edit.py
================
Eigene Minigolf-Bahnen: der MAPS-Reiter und der Bahn-Editor.

Beides hängt an ``MiniGolfGame`` (minigolf.py) und zeichnet auf dessen
Fläche - eigene Game-Klassen wären hier fehl am Platz, weil Editor und
Spiel dieselbe Instanz teilen (Test-Spielen soll ja zurückführen können).

    MapList    Liste der eigenen Bahnen: Neu, Bearbeiten, Spielen, Löschen,
               Teilen (Export) und Importieren. Der Teilen-Dialog liegt als
               Overlay darüber.
    MapEditor  Der eigentliche Editor: Leinwand, Palette mit 15 Hindernis-
               Typen, Parameterleiste, Vorlagen, Undo/Redo, Test-Spielen.

Gespeichert wird über ``ugc.py`` (ugc.json), gezeichnet über
``minigolf_draw.py`` - dieselben Funktionen wie im Spiel, damit eine Bahn im
Editor genauso aussieht wie später beim Spielen.
"""

import copy
import math
import os

import pygame

import filepick
import swear
import ugc
import ui
from game_base import InputEvent
from i18n import t

from . import minigolf_draw as draw
from . import minigolf_gen as gen
from .minigolf_gen import BALL_R, BORDER, make_hole as _hole

# ---------------------------------------------------------------------------
#  Werkzeuge und Palette
# ---------------------------------------------------------------------------

# Werkzeuge, die kein Hindernis setzen.
TOOLS = ("select", "erase", "tee", "cup")

# Acht Himmelsrichtungen - Richtungen werden im Editor durchgeklickt, statt
# dx und dy einzeln einzustellen. Das spart Platz und man kann nichts
# Ungültiges (0, 0) einstellen.
DIRS = ((0.0, -1.0), (0.7071, -0.7071), (1.0, 0.0), (0.7071, 0.7071),
        (0.0, 1.0), (-0.7071, 0.7071), (-1.0, 0.0), (-0.7071, -0.7071))


def _dir_index(dx, dy):
    """Nächstliegende der acht Richtungen zu (dx, dy)."""
    best, bd = 0, -2.0
    n = math.hypot(dx, dy) or 1.0
    for i, (ux, uy) in enumerate(DIRS):
        d = (dx / n) * ux + (dy / n) * uy
        if d > bd:
            best, bd = i, d
    return best


def _p(key, idx, lo, hi, step, kind="num"):
    """Ein einstellbarer Wert eines Hindernisses (für die Parameterleiste)."""
    return {"key": key, "idx": idx, "lo": lo, "hi": hi, "step": step,
            "kind": kind}


# Alles, was der Editor über die 15 Hindernis-Typen wissen muss:
#   kind    "rect"   aufziehen (Breite/Höhe aus der Mausbewegung)
#           "circle" ein Klick, Größe über die Parameterleiste
#           "pair"   zwei Klicks (Rohr: Eingang und Ausgang)
#   make    baut den Eintrag aus den Rohdaten des Setzens
#   params  was sich nachträglich einstellen lässt
PALETTE = (
    ("walls", "rect", (_p("w", 2, 2, 160, 1), _p("h", 3, 2, 240, 1))),
    ("sand", "rect", (_p("w", 2, 4, 160, 2), _p("h", 3, 4, 240, 2))),
    ("water", "rect", (_p("w", 2, 4, 160, 2), _p("h", 3, 4, 240, 2))),
    ("ice", "rect", (_p("w", 2, 4, 160, 2), _p("h", 3, 4, 240, 2))),
    ("sticky", "rect", (_p("w", 2, 4, 160, 2), _p("h", 3, 4, 240, 2))),
    ("slopes", "rect", (_p("w", 2, 4, 160, 2), _p("h", 3, 4, 240, 2),
                        _p("dir", 4, 0, 0, 1, "dir"),
                        _p("accel", -1, 6, 60, 2, "mag"))),
    ("boosters", "rect", (_p("w", 2, 4, 160, 2), _p("h", 3, 4, 240, 2),
                          _p("dir", 4, 0, 0, 1, "dir"),
                          _p("boost", 6, 40, 200, 10))),
    ("gates", "rect", (_p("w", 2, 4, 160, 2), _p("h", 3, 2, 240, 1),
                       _p("dir", 4, 0, 0, 1, "dir"))),
    ("jumps", "rect", (_p("w", 2, 4, 160, 2), _p("h", 3, 4, 240, 2),
                       _p("dir", 4, 0, 0, 1, "dir"),
                       _p("dist", 6, 10, 90, 5))),
    ("movers", "rect", (_p("w", 2, 4, 160, 2), _p("h", 3, 2, 240, 1),
                        _p("dir", 4, 0, 0, 1, "movedir"),
                        _p("dist", -2, 6, 80, 2, "span"),
                        _p("speed", 6, 6, 60, 2))),
    ("bumpers", "circle", (_p("r", 2, 3, 16, 1),)),
    ("magnets", "circle", (_p("r", 2, 8, 60, 2),
                           _p("force", 3, -260, 260, 20))),
    ("spinners", "circle", (_p("r", 2, 5, 30, 1),
                            _p("speed", 3, -5, 5, 0.5))),
    ("mills", "circle", (_p("len", 2, 5, 40, 1), _p("arms", 3, 2, 4, 1),
                         _p("speed", 4, -3, 3, 0.2))),
    ("tunnels", "pair", (_p("r", 4, 3, 12, 1),)),
)
PALETTE_KEYS = tuple(p[0] for p in PALETTE)
PAL = {p[0]: p for p in PALETTE}

# Farbe je Typ für die Palettenknöpfe (aus minigolf_draw, damit die Knöpfe
# aussehen wie das, was sie setzen).
SWATCH = {
    "walls": draw.COL_WALL, "sand": draw.COL_SAND, "water": draw.COL_WATER,
    "ice": draw.COL_ICE, "sticky": draw.COL_STICKY, "slopes": draw.COL_SLOPE,
    "boosters": draw.COL_BOOST, "gates": draw.COL_GATE,
    "jumps": draw.COL_JUMP, "movers": draw.COL_WALL_HI,
    "bumpers": draw.COL_BUMPER, "magnets": draw.COL_MAGNET,
    "spinners": draw.COL_SPIN, "mills": draw.COL_MILL,
    "tunnels": draw.COL_TUNNEL,
}
ROUND_KEYS = ("bumpers", "magnets", "spinners", "mills", "tunnels")


def new_item(key, x, y, w=0.0, h=0.0, x2=None, y2=None):
    """Baut einen neuen Hindernis-Eintrag mit brauchbaren Startwerten."""
    if key in ("walls", "sand", "water", "ice", "sticky"):
        return (x, y, w, h)
    if key == "slopes":
        return (x, y, w, h, 0.0, 24.0)
    if key == "boosters":
        return (x, y, w, h, 0.0, -1.0, 120.0)
    if key == "gates":
        return (x, y, w, h, 0.0, -1.0)
    if key == "jumps":
        return (x, y, w, h, 0.0, -1.0, 40.0)
    if key == "movers":
        return (x, y, w, h, min(30.0, w * 1.5), 0.0, 24.0)
    if key == "bumpers":
        return (x, y, 6.0)
    if key == "magnets":
        return (x, y, 24.0, 120.0)
    if key == "spinners":
        return (x, y, 12.0, 2.0)
    if key == "mills":
        return (x, y, 13.0, 2.0, 1.5)
    if key == "tunnels":
        return (x, y, x2, y2, 5.0)
    return (x, y, w, h)


# ---------------------------------------------------------------------------
#  Vorlagen
# ---------------------------------------------------------------------------
#
# Jede Vorlage lässt neben ihrem Kunststück IMMER einen normalen Weg offen -
# eine Bahn, die nur mit einem Trick lösbar ist, wäre keine Vorlage, sondern
# eine Falle. tests/newgames_audit.py spielt jede Vorlage mit dem Solver durch.

def _tpl_blank():
    return _hole(2, (50, 146), (50, 20))


def _tpl_tunnel():
    """Riegel mit Durchlass rechts - und einem Rohr als Abkürzung."""
    return _hole(3, (24, 146), (76, 24),
                 walls=[(6, 84, 62, 8)],
                 tunnels=[(22, 118, 30, 44, 5)],
                 sand=[(70, 104, 22, 16)])


def _tpl_island():
    """Inselgrün: Wasser ringsum, ein breiter Hals führt hinauf."""
    return _hole(3, (50, 146), (50, 34),
                 water=[(6, 20, 26, 34), (68, 20, 26, 34),
                        (6, 54, 24, 16), (70, 54, 24, 16)],
                 sand=[(40, 96, 20, 14)])


def _tpl_mill():
    """Korridor mit Windmühle - reines Timing."""
    return _hole(3, (50, 146), (50, 18),
                 walls=[(22, 26, 8, 92), (70, 26, 8, 92)],
                 mills=[(50, 92, 14, 2, 1.4)])


def _tpl_zigzag():
    """Drei versetzte Tore."""
    return _hole(4, (18, 146), (82, 20),
                 walls=[(28, 118, 66, 7), (6, 84, 66, 7), (28, 50, 66, 7)])


def _tpl_water():
    """Trockene Gasse zwischen zwei Teichen."""
    return _hole(3, (50, 146), (50, 22),
                 water=[(6, 60, 30, 44), (64, 60, 30, 44)],
                 sand=[(40, 116, 20, 14)])


def _tpl_bumper():
    """Offenes Feld voller Gummipuffer."""
    return _hole(3, (50, 146), (50, 20),
                 bumpers=[(30, 100, 6), (70, 100, 6), (50, 70, 7),
                          (28, 44, 5), (72, 44, 5)])


def _tpl_ramp():
    """Steigung, die zurückschiebt - hier braucht es Kraft."""
    return _hole(3, (50, 148), (50, 20),
                 slopes=[(20, 52, 60, 56, 0.0, 30.0)],
                 sand=[(18, 116, 22, 14), (60, 116, 22, 14)])


def _tpl_ice():
    """Eisfläche mit Bande - der Ball will einfach nicht anhalten."""
    return _hole(3, (50, 148), (50, 20),
                 ice=[(10, 44, 80, 74)],
                 walls=[(40, 90, 20, 8)],
                 sticky=[(38, 26, 24, 12)])


def _tpl_maze():
    """Labyrinth aus kurzen Wänden."""
    return _hole(4, (16, 148), (84, 18),
                 walls=[(30, 122, 64, 7), (6, 96, 64, 7), (30, 70, 64, 7),
                        (6, 44, 64, 7)],
                 sand=[(74, 106, 18, 12)])


def _tpl_magnet():
    """Zwei Magnete neben der Gasse: einer zieht, einer stößt ab."""
    return _hole(3, (50, 146), (50, 22),
                 magnets=[(18, 96, 30, 140), (82, 62, 30, -140)],
                 walls=[(46, 118, 8, 14)])


def _tpl_jump():
    """Sprungschanze über einen Riegel - außen herum geht es auch."""
    return _hole(3, (50, 150), (50, 20),
                 jumps=[(42, 124, 16, 10, 0.0, -1.0, 46)],
                 walls=[(24, 92, 52, 8)],
                 sand=[(10, 104, 16, 14), (74, 104, 16, 14)])


TEMPLATES = (
    ("blank", _tpl_blank), ("tunnel", _tpl_tunnel), ("island", _tpl_island),
    ("mill", _tpl_mill), ("zigzag", _tpl_zigzag), ("water", _tpl_water),
    ("bumper", _tpl_bumper), ("ramp", _tpl_ramp), ("ice", _tpl_ice),
    ("maze", _tpl_maze), ("magnet", _tpl_magnet), ("jump", _tpl_jump),
)
TEMPLATE_KEYS = tuple(k for k, _ in TEMPLATES)


def template(key):
    """Fertiges Bahn-Dict einer Vorlage (leer, wenn der Name nicht passt)."""
    for name, fn in TEMPLATES:
        if name == key:
            return gen.normalize(fn())
    return gen.normalize(_tpl_blank())


# ---------------------------------------------------------------------------
#  Prüfung einer Bahn
# ---------------------------------------------------------------------------

def validate(hole):
    """Prüft eine Bahn vor dem Speichern. Liefert "" oder einen Fehlerschlüssel.

    Geprüft wird nur, was die Bahn UNSPIELBAR macht: Abschlag und Loch
    müssen im Feld liegen und dürfen nicht in einer Wand, im Wasser, in
    einem Tor oder auf einer Sprungrampe stecken. Ob eine Bahn schwer ist,
    entscheidet der Erbauer selbst.
    """
    hole = gen.normalize(hole)
    cw, ch = hole["w"], hole["h"]
    lo = BORDER + BALL_R
    for (px, py) in (hole["tee"], hole["cup"]):
        if not (lo <= px <= cw - lo and lo <= py <= ch - lo):
            return "bounds"
    clear = BALL_R + 1.0
    for key in ("walls", "water", "gates", "jumps"):
        for r in hole[key]:
            x, y, w, h = r[0], r[1], r[2], r[3]
            for (px, py) in (hole["tee"], hole["cup"]):
                if (x - clear < px < x + w + clear
                        and y - clear < py < y + h + clear):
                    return "blocked"
    for (x, y, r) in ((b[0], b[1], b[2]) for b in hole["bumpers"]):
        for (px, py) in (hole["tee"], hole["cup"]):
            if math.hypot(x - px, y - py) < r + clear:
                return "blocked"
    if math.hypot(hole["tee"][0] - hole["cup"][0],
                  hole["tee"][1] - hole["cup"][1]) < 12.0:
        return "tooclose"
    return ""


# ---------------------------------------------------------------------------
#  Gemeinsame Zeichenhelfer
# ---------------------------------------------------------------------------

def _btn(s, rc, text, fnt, on=False, accent=None, enabled=True):
    """Knopf im Stil des Minigolf-Setups (mit Schrumpfen bei langem Text)."""
    col_bg = ui.BTN_SEL if on else ui.BTN
    pygame.draw.rect(s, col_bg, rc, border_radius=7)
    pygame.draw.rect(s, (accent or ui.ACCENT) if on else ui.BORDER, rc,
                     2 if on else 1, border_radius=7)
    col = ui.TEXT if enabled else ui.TEXT_FAINT
    if on:
        col = ui.TEXT
    img = fnt.render(text, True, col)
    if img.get_width() > rc.w - 8:
        kurz = text
        while len(kurz) > 1 and fnt.size(kurz + "...")[0] > rc.w - 8:
            kurz = kurz[:-1]
        img = fnt.render(kurz + "...", True, col)
    s.blit(img, img.get_rect(center=rc.center))
    return rc


def _arrow(s, center, dx, dy, size, col):
    """Kleiner Richtungspfeil.

    Wird GEZEICHNET statt geschrieben: die Projektschrift (Bahnschrift) hat
    keine Pfeil-Zeichen, die kämen als leere Kästchen heraus. Dasselbe gilt
    für die Werkzeug-Sinnbilder in der Palette.
    """
    n = math.hypot(dx, dy) or 1.0
    ux, uy = dx / n, dy / n
    cx, cy = center
    tip = (cx + ux * size, cy + uy * size)
    tail = (cx - ux * size, cy - uy * size)
    pygame.draw.line(s, col, tail, tip, 2)
    head = size * 0.7
    pygame.draw.polygon(s, col, [
        tip,
        (tip[0] - ux * head - uy * head * 0.6,
         tip[1] - uy * head + ux * head * 0.6),
        (tip[0] - ux * head + uy * head * 0.6,
         tip[1] - uy * head - ux * head * 0.6)])


def _tool_icon(s, rc, key, col):
    """Sinnbild eines Werkzeugs - ebenfalls gezeichnet, nicht geschrieben."""
    cx, cy = rc.center
    r = max(4, min(rc.w, rc.h) // 4)
    if key == "select":                      # Mauszeiger
        pygame.draw.polygon(s, col, [(cx - r, cy - r), (cx + r, cy),
                                     (cx, cy + r * 0.3),
                                     (cx + r * 0.3, cy + r)], 2)
    elif key == "erase":                     # Radierer mit Strich
        box = pygame.Rect(0, 0, r * 2, int(r * 1.5))
        box.center = (cx, cy)
        pygame.draw.rect(s, col, box, 2, border_radius=2)
        pygame.draw.line(s, col, (box.left, box.bottom), (box.right, box.top), 2)
    elif key == "tee":                       # Abschlag: Ring mit Punkt
        pygame.draw.circle(s, col, (cx, cy), r, 2)
        pygame.draw.circle(s, col, (cx, cy), max(1, r // 3))
    else:                                    # Loch mit Fahne
        pygame.draw.circle(s, col, (cx + r // 2, cy + r), max(2, r // 2))
        pygame.draw.line(s, col, (cx + r // 2, cy + r), (cx + r // 2, cy - r), 2)
        pygame.draw.polygon(s, col, [(cx + r // 2, cy - r),
                                     (cx - r, cy - r // 2),
                                     (cx + r // 2, cy)])


def _edit_icon(s, rc, key, col):
    """Sinnbild der Werkzeugleiste (Raster, Undo, Redo, Leeren)."""
    cx, cy = rc.center
    r = max(4, min(rc.w, rc.h) // 4)
    if key == "grid":
        box = pygame.Rect(0, 0, r * 2, r * 2)
        box.center = (cx, cy)
        pygame.draw.rect(s, col, box, 1)
        pygame.draw.line(s, col, (box.centerx, box.top), (box.centerx, box.bottom))
        pygame.draw.line(s, col, (box.left, box.centery), (box.right, box.centery))
    elif key in ("undo", "redo"):
        sign = -1 if key == "undo" else 1
        rect = pygame.Rect(0, 0, r * 2, r * 2)
        rect.center = (cx, cy + r // 2)
        start, end = (0.35, 2.9) if key == "undo" else (0.25, 2.8)
        pygame.draw.arc(s, col, rect, start, end, 2)
        tipx = cx - sign * r
        _arrow(s, (tipx, cy + r // 2), 0, 1, max(3, r // 2), col)
    else:                                    # leeren: Kreuz
        d = r
        pygame.draw.line(s, col, (cx - d, cy - d), (cx + d, cy + d), 2)
        pygame.draw.line(s, col, (cx + d, cy - d), (cx - d, cy + d), 2)


def _label(s, rects, text, fnt, w=None):
    """Beschriftung mittig über eine Knopfgruppe (gekürzt, wenn zu lang)."""
    img = fnt.render(text, True, ui.TEXT_DIM)
    if w and img.get_width() > w:
        kurz = text
        while len(kurz) > 2 and fnt.size(kurz + "...")[0] > w:
            kurz = kurz[:-1]
        img = fnt.render(kurz + "...", True, ui.TEXT_DIM)
    mid = (rects[0].left + rects[-1].right) // 2
    s.blit(img, img.get_rect(midbottom=(mid, rects[0].top - 3)))


# ---------------------------------------------------------------------------
#  MAPS-Reiter: die Liste der eigenen Bahnen
# ---------------------------------------------------------------------------

# Knöpfe unter der Liste. "new" und "import" gehen immer, der Rest braucht
# eine ausgewählte Bahn.
LIST_BUTTONS = ("new", "edit", "play", "delete", "share", "import")
NEEDS_SEL = ("edit", "play", "delete", "share")


class MapList:
    """Der MAPS-Reiter: Bahnen auswählen, bauen, spielen, teilen, importieren."""

    def __init__(self, game):
        self.game = game
        self.items = []
        self.sel = 0
        self.first = 0
        self.toast = ""
        self.toast_t = 0.0
        self.confirm = ""            # id, deren Löschen bestätigt werden will
        # Teilen-Overlay
        self.share = None            # die Bahn, die geteilt wird (oder None)
        self.f_author = ui.TextInput(maxlen=ugc.MAX_AUTHOR)
        self.f_file = ui.TextInput(maxlen=ugc.MAX_ID,
                                   charset=ui.TextInput.ID_CHARS)
        self.focus = 0               # 0 = Ersteller, 1 = Dateiname
        self.err = ""
        self.reload()
        self.layout()

    # ----- Daten --------------------------------------------------------
    def reload(self):
        self.items = ugc.load_maps()
        self.sel = max(0, min(self.sel, len(self.items) - 1))
        self._clamp()

    def selected(self):
        return self.items[self.sel] if self.items else None

    # ----- Layout -------------------------------------------------------
    def layout(self):
        g = self.game
        w, h = g.width, g.height
        self.fnt = ui.font(max(13, h // 30))
        self.tiny = ui.font(max(10, h // 40))
        self.row_h = max(26, min(40, h // 11))
        # Knopfzeile unten: bei schmalen Fenstern zwei Reihen zu drei.
        cols = 6 if w >= 720 else 3
        bh = max(24, min(34, h // 13))
        gap = 6
        rows = (len(LIST_BUTTONS) + cols - 1) // cols
        bw = (w - 24 - gap * (cols - 1)) / float(cols)
        bottom = h - 18
        self.btn_rects = {}
        for i, key in enumerate(LIST_BUTTONS):
            r, c = divmod(i, cols)
            y = bottom - (rows - r) * (bh + gap) + gap
            self.btn_rects[key] = pygame.Rect(int(12 + c * (bw + gap)), int(y),
                                              int(bw), bh)
        self.list_top = g.tab_bottom + 8
        self.list_bottom = bottom - rows * (bh + gap) - 4
        self.rows_visible = max(1, int((self.list_bottom - self.list_top - 14)
                                       // self.row_h))
        self.list_w = w - 30
        self._clamp()
        # Teilen-Overlay
        pw = min(w - 40, 380)
        fh = max(22, min(30, h // 14))
        ph = min(h - 30, 78 + 3 * fh + 62)
        self.share_rect = pygame.Rect((w - pw) // 2, (h - ph) // 2, pw, ph)
        fx = self.share_rect.x + 18
        fw = pw - 36
        self.author_rect = pygame.Rect(fx, self.share_rect.y + 46, fw, fh)
        self.file_rect = pygame.Rect(fx, self.author_rect.bottom + 24, fw, fh)
        sy = self.file_rect.bottom + 16
        sbw = (fw - 10) / 2.0
        self.share_btn = {
            "as": pygame.Rect(fx, sy, int(sbw), fh),
            "dl": pygame.Rect(int(fx + sbw + 10), sy, int(sbw), fh),
            "cancel": pygame.Rect(fx, sy + fh + 6, fw, fh),
        }

    def _clamp(self):
        n = len(self.items)
        vis = getattr(self, "rows_visible", 1)
        self.first = max(0, min(self.first, max(0, n - vis)))
        if self.sel < self.first:
            self.first = self.sel
        elif self.sel >= self.first + vis:
            self.first = self.sel - vis + 1

    # ----- Rückmeldung -------------------------------------------------
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
        if event.kind == InputEvent.KEYDOWN:
            k = event.key
            if k in ("Up", "Left") or self.game.is_action(k, "up"):
                self._move(-1)
            elif k in ("Down", "Right") or self.game.is_action(k, "down"):
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
                rc = pygame.Rect(12, self.list_top + i * self.row_h,
                                 self.list_w, self.row_h - 3)
                if rc.collidepoint(event.pos):
                    if self.sel != idx:
                        self.confirm = ""
                    self.sel = idx
                    self.game.play_sound("move")
                    return True
            return True
        return False

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
            if ugc.is_full():
                self._toast(t("golf.ugc.err.full", max=ugc.MAX_MAPS))
                return
            g.ugc_new_map()
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
            g.ugc_play(m["id"])
        elif key == "share":
            self._open_share(m)
        elif key == "delete":
            if self.confirm != m["id"]:
                self.confirm = m["id"]
                g.play_sound("select")
                return
            ugc.delete_map(m["id"])
            self.confirm = ""
            self.reload()
            self._toast(t("golf.ugc.deleted"))
            g.play_sound("click")

    # ----- Teilen -------------------------------------------------------
    def _open_share(self, m):
        self.share = m
        self.err = ""
        self.focus = 0
        self.f_author.set_text(m.get("author") or ugc.last_author())
        self.f_file.set_text(m.get("id") or "map")
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
            elif self.share_btn["cancel"].collidepoint(event.pos):
                self._close_share()
            elif not self.share_rect.collidepoint(event.pos):
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
        """Teilen: Datei schreiben - per Dialog oder direkt nach Downloads."""
        m = self.share
        if m is None:
            return
        author = self.f_author.text.strip()
        name = self.f_file.text.strip() or (m.get("id") or "map")
        if author and not swear.is_clean(author):
            self.err = t("golf.ugc.err.swear")
            self.game.play_sound("hit")
            return
        out = dict(m)
        if author:
            out["author"] = author
            ugc.set_last_author(author)
        filename = name + ugc.EXT
        if ask:
            if not filepick.available():
                self.err = t("golf.ugc.err.nofile")
                return
            path = filepick.save_as(filename, t("golf.ugc.share_title"))
            if not path:
                return                      # Dialog abgebrochen
        else:
            path = filepick.to_downloads(filename)
        ok, why = ugc.export_to(out, path)
        if not ok:
            self.err = t("golf.ugc.err." + ("swear" if why == "swear" else "io"))
            self.game.play_sound("hit")
            return
        # Der Ersteller-Name gehört auch in die gespeicherte Bahn.
        if author and m.get("author") != author:
            ugc.save_map(out)
            self.reload()
        self._close_share()
        self._toast(t("golf.ugc.exported", path=filepick.short(path)))
        self.game.play_sound("point")

    def _do_import(self):
        if ugc.is_full():
            self._toast(t("golf.ugc.err.full", max=ugc.MAX_MAPS))
            return
        if not filepick.available():
            self._toast(t("golf.ugc.err.nofile"))
            return
        path = filepick.open_file(t("golf.ugc.btn_import"))
        if not path:
            return
        wanted = ugc.slug(os.path.splitext(os.path.basename(path))[0])
        ok, why, m = ugc.import_from(path)
        if not ok:
            known = ("swear", "full", "format", "io")
            self._toast(t("golf.ugc.err." + (why if why in known else "format")))
            self.game.play_sound("hit")
            return
        self.reload()
        for i, x in enumerate(self.items):
            if x["id"] == m["id"]:
                self.sel = i
                self._clamp()
                break
        if wanted and m["id"] != wanted:
            self._toast(t("golf.ugc.renamed", id=m["id"]))
        else:
            self._toast(t("golf.ugc.imported", name=m["name"]))
        self.game.play_sound("point")

    # ----- Zeichnen -----------------------------------------------------
    def draw(self, s):
        if not self.items:
            self._draw_empty(s)
        else:
            for i in range(self.rows_visible):
                idx = self.first + i
                if idx >= len(self.items):
                    break
                self._draw_row(s, self.items[idx], idx,
                               pygame.Rect(12, self.list_top + i * self.row_h,
                                           self.list_w, self.row_h - 3))
            if len(self.items) > self.rows_visible:
                self._draw_scrollbar(s)
        cnt = self.tiny.render(t("golf.ugc.count", n=len(self.items),
                                 max=ugc.MAX_MAPS), True, ui.TEXT_DIM)
        s.blit(cnt, (12, self.list_bottom + 1))
        has = self.selected() is not None
        for key, rc in self.btn_rects.items():
            _btn(s, rc, t("golf.ugc.btn_" + key), self.tiny,
                 enabled=has or key not in NEEDS_SEL)
        if self.toast:
            self._draw_toast(s)
        if self.share is not None:
            self._draw_share(s)

    def _draw_empty(self, s):
        g = self.game
        box = pygame.Rect(24, self.list_top + 10, g.width - 48,
                          max(60, self.list_bottom - self.list_top - 24))
        ui.draw_panel(s, box, shadow=False)
        for i, line in enumerate((t("golf.ugc.empty"), t("golf.ugc.empty2"))):
            img = self.fnt.render(line, True,
                                  ui.TEXT_DIM if i == 0 else ui.TEXT_FAINT)
            s.blit(img, img.get_rect(center=(box.centerx,
                                             box.centery - 12 + i * 22)))

    def _draw_row(self, s, m, idx, rc):
        sel = (idx == self.sel)
        pygame.draw.rect(s, ui.PANEL_LIGHT if sel else ui.PANEL, rc,
                         border_radius=7)
        pygame.draw.rect(s, self.game.accent if sel else ui.BORDER, rc,
                         2 if sel else 1, border_radius=7)
        pad = 10
        name = self.fnt.render(m.get("name", ""), True, ui.TEXT)
        s.blit(name, (rc.x + pad, rc.y + 3))
        sub = "%s  ·  %s  ·  %s" % (
            m.get("id", ""),
            t("golf.ugc.size", w=int(m["w"]), h=int(m["h"])),
            t("golf.par", n=int(m.get("par", 3))))
        if m.get("author"):
            sub += "  ·  " + t("golf.ugc.by", name=m["author"])
        img = self.tiny.render(sub, True, ui.TEXT_DIM)
        if rc.h >= 32:
            s.blit(img, (rc.x + pad, rc.bottom - img.get_height() - 3))
        if self.confirm == m.get("id"):
            warn = self.tiny.render(t("golf.ugc.confirm_delete"), True, ui.RED)
            s.blit(warn, warn.get_rect(midright=(rc.right - pad, rc.centery)))

    def _draw_scrollbar(self, s):
        track = pygame.Rect(self.list_w + 14, self.list_top, 4,
                            self.rows_visible * self.row_h)
        pygame.draw.rect(s, ui.PANEL, track, border_radius=2)
        frac = self.rows_visible / float(len(self.items))
        h = max(24, int(track.h * frac))
        pos = self.first / float(max(1, len(self.items) - self.rows_visible))
        y = track.y + int((track.h - h) * min(1.0, pos))
        pygame.draw.rect(s, ui.BORDER_LIGHT, (track.x, y, 4, h),
                         border_radius=2)

    def _draw_toast(self, s):
        img = self.tiny.render(self.toast, True, ui.TEXT)
        w, h = img.get_width() + 26, img.get_height() + 12
        r = pygame.Rect(self.game.width // 2 - w // 2,
                        self.list_bottom - h - 2, w, h)
        ui.draw_panel(s, r, radius=h // 2, shadow=False)
        pygame.draw.rect(s, ui.ACCENT2, r, 1, border_radius=h // 2)
        s.blit(img, img.get_rect(center=r.center))

    def _draw_share(self, s):
        g = self.game
        veil = pygame.Surface((g.width, g.height), pygame.SRCALPHA)
        veil.fill((0, 0, 0, 150))
        s.blit(veil, (0, 0))
        r = self.share_rect
        ui.draw_panel(s, r, accent_top=g.accent)
        head = self.fnt.render(t("golf.ugc.share_title"), True, g.accent)
        s.blit(head, head.get_rect(midtop=(r.centerx, r.y + 10)))
        for i, (rect, key, field) in enumerate((
                (self.author_rect, "golf.ugc.creator", self.f_author),
                (self.file_rect, "golf.ugc.filename", self.f_file))):
            lab = self.tiny.render(t(key), True, ui.TEXT_DIM)
            s.blit(lab, (rect.x, rect.y - lab.get_height() - 2))
            field.draw(s, rect, self.tiny, focused=(self.focus == i))
        ext = self.tiny.render(ugc.EXT, True, ui.TEXT_FAINT)
        s.blit(ext, (self.file_rect.right - ext.get_width() - 8,
                     self.file_rect.centery - ext.get_height() // 2))
        _btn(s, self.share_btn["as"], t("golf.ugc.export_as"), self.tiny)
        _btn(s, self.share_btn["dl"], t("golf.ugc.export_dl"), self.tiny)
        _btn(s, self.share_btn["cancel"], t("golf.ugc.btn_cancel"), self.tiny)
        if self.err:
            img = self.tiny.render(self.err, True, ui.RED)
            s.blit(img, img.get_rect(midbottom=(r.centerx, r.bottom - 6)))


# ---------------------------------------------------------------------------
#  Der Bahn-Editor
# ---------------------------------------------------------------------------

# Knöpfe der zweiten Kopfzeile. Beschriftet wird nur, was sich nicht als
# Sinnbild sagen lässt - so passt die Zeile auch bei 480x360.
TEXT_BUTTONS = ("template", "test")
ICON_BUTTONS = ("grid", "undo", "redo", "clear")
EDIT_BUTTONS = TEXT_BUTTONS + ICON_BUTTONS

UNDO_MAX = 40


class MapEditor:
    """Bahn bauen: Leinwand links, Palette rechts, Parameter unten.

    Die Bahn selbst ist das gewohnte Bahn-Dict - der Editor ändert nur
    Listen darin. Gezeichnet wird mit denselben Funktionen wie im Spiel
    (minigolf_draw), damit nichts auseinanderläuft.
    """

    def __init__(self, game, m):
        self.game = game
        self.map = dict(m)
        self.hole = gen.normalize(dict(m))
        self.tool = "walls"          # aktives Werkzeug (Typ oder TOOLS-Eintrag)
        self.sel = None              # (typ, index) des gewählten Hindernisses
        self.drag = None             # (typ, x0, y0, x1, y1) beim Aufziehen
        self.move = None             # (typ, index, offx, offy) beim Verschieben
        self.pending = None          # erster Klick eines Rohrs
        self.undo_stack = []
        self.redo_stack = []
        self.dirty = False
        self.err = ""
        self.toast = ""
        self.toast_t = 0.0
        self.picking = False         # Vorlagen-Auswahl offen?
        self.confirm_back = False
        self.t = 0.0
        self.grid = bool(game.grid_snap)
        self.f_name = ui.TextInput(m.get("name", ""), maxlen=ugc.MAX_NAME,
                                   placeholder=t("golf.ugc.name"))
        self.f_id = ui.TextInput(m.get("id", ""), maxlen=ugc.MAX_ID,
                                 charset=ui.TextInput.ID_CHARS,
                                 placeholder=t("golf.ugc.id"))
        self.focus = -1              # -1 = kein Feld, 0 = Name, 1 = id
        self.orig_id = m.get("id", "")
        self.layout()

    # ----- Layout -------------------------------------------------------
    def layout(self):
        g = self.game
        w, h = g.width, g.height
        self.tiny = ui.font(max(10, h // 42))
        self.small = ui.font(max(12, h // 34))
        bh = max(20, min(28, h // 15))
        gap = 5
        # Kopfzeile 1: Zurück | Name | id | Speichern
        y = 6
        back_w = max(48, int(w * 0.11))
        save_w = max(62, int(w * 0.16))
        field_w = (w - 24 - back_w - save_w - 3 * gap) / 2.0
        self.back_rect = pygame.Rect(12, y, back_w, bh)
        self.name_rect = pygame.Rect(int(12 + back_w + gap), y,
                                     int(field_w), bh)
        self.id_rect = pygame.Rect(int(self.name_rect.right + gap), y,
                                   int(field_w), bh)
        self.save_rect = pygame.Rect(int(self.id_rect.right + gap), y,
                                     save_w, bh)
        # Kopfzeile 2: Par, Breite, Höhe (je - Wert +) und sechs Knöpfe
        y2 = y + bh + gap
        step_w = max(16, int(w * 0.036))
        val_w = max(30, int(w * 0.062))
        grp = 2 * step_w + val_w
        self.num_rects = {}
        x = 12
        for key in ("par", "width", "height"):
            self.num_rects[key] = (
                pygame.Rect(x, y2, step_w, bh),
                pygame.Rect(x + step_w, y2, val_w, bh),
                pygame.Rect(x + step_w + val_w, y2, step_w, bh))
            x += grp + gap
        # Rechts der Zahlenfelder: erst zwei beschriftete Knöpfe (Vorlage,
        # Test), dann vier Sinnbild-Knöpfe. Text braucht Platz, Sinnbilder
        # nicht - so bleibt bei 480x360 alles lesbar.
        rest = w - 12 - x
        icon_w = max(22, min(30, bh + 2))
        icons = len(ICON_BUTTONS)
        text_area = rest - icons * (icon_w + gap)
        tbw = max(34, (text_area - gap * len(TEXT_BUTTONS)) / float(
            len(TEXT_BUTTONS)))
        self.edit_rects = {}
        for i, key in enumerate(TEXT_BUTTONS):
            self.edit_rects[key] = pygame.Rect(int(x + i * (tbw + gap)), y2,
                                               int(tbw), bh)
        x2 = x + len(TEXT_BUTTONS) * (tbw + gap)
        for i, key in enumerate(ICON_BUTTONS):
            self.edit_rects[key] = pygame.Rect(int(x2 + i * (icon_w + gap)), y2,
                                               icon_w, bh)
        self.head_bottom = y2 + bh + 5
        # Parameterleiste unten
        self.par_h = bh + 16
        self.par_top = h - self.par_h
        # Palette rechts: zwei Spalten
        cols = 2
        rows = (len(PALETTE) + len(TOOLS) + cols - 1) // cols
        avail = self.par_top - self.head_bottom - 6
        pbh = max(16, min(30, (avail - (rows - 1) * 3) // rows))
        pbw = max(34, min(52, int(w * 0.085)))
        self.pal_w = cols * pbw + 3
        px0 = w - self.pal_w - 6
        self.pal_rects = []
        for i, key in enumerate(TOOLS + PALETTE_KEYS):
            r, c = divmod(i, cols)
            self.pal_rects.append(
                (key, pygame.Rect(px0 + c * (pbw + 3),
                                  self.head_bottom + r * (pbh + 3), pbw, pbh)))
        # Leinwand: was übrig bleibt
        cw, ch = self.hole["w"], self.hole["h"]
        area_w = px0 - 18
        area_h = avail
        self.scale = max(0.8, min(area_w / cw, area_h / ch))
        self.ox = 12 + (area_w - cw * self.scale) / 2.0
        self.oy = self.head_bottom + (area_h - ch * self.scale) / 2.0
        self.canvas = pygame.Rect(int(self.ox), int(self.oy),
                                  int(cw * self.scale), int(ch * self.scale))
        # Vorlagen-Auswahl
        pw = min(w - 30, 420)
        tcols = 3
        trows = (len(TEMPLATES) + tcols - 1) // tcols
        tbh = max(22, min(32, (h - 90) // (trows + 1)))
        ph = 44 + trows * (tbh + 6) + 10
        self.tpl_rect = pygame.Rect((w - pw) // 2, max(6, (h - ph) // 2), pw, ph)
        tbw = (pw - 24 - 6 * (tcols - 1)) / float(tcols)
        self.tpl_rects = []
        for i, (key, _fn) in enumerate(TEMPLATES):
            r, c = divmod(i, tcols)
            self.tpl_rects.append(
                (key, pygame.Rect(int(self.tpl_rect.x + 12 + c * (tbw + 6)),
                                  int(self.tpl_rect.y + 38 + r * (tbh + 6)),
                                  int(tbw), tbh)))

    def view(self):
        return draw.View(self.ox, self.oy, self.scale)

    # ----- Undo / Änderungen -------------------------------------------
    def _push(self):
        """Zustand sichern, bevor etwas geändert wird."""
        self.undo_stack.append(copy.deepcopy(self.hole))
        if len(self.undo_stack) > UNDO_MAX:
            del self.undo_stack[0]
        self.redo_stack.clear()
        self.dirty = True
        self.err = ""

    def _undo(self):
        if not self.undo_stack:
            return
        self.redo_stack.append(copy.deepcopy(self.hole))
        self.hole = self.undo_stack.pop()
        self.sel = None
        self.dirty = True
        self._resize_canvas()
        self.game.play_sound("move")

    def _redo(self):
        if not self.redo_stack:
            return
        self.undo_stack.append(copy.deepcopy(self.hole))
        self.hole = self.redo_stack.pop()
        self.sel = None
        self.dirty = True
        self._resize_canvas()
        self.game.play_sound("move")

    def _resize_canvas(self):
        """Leinwand neu einpassen (nach Größenänderung oder Undo)."""
        self.layout()

    # ----- Rückmeldung -------------------------------------------------
    def _toast(self, text):
        self.toast = text
        self.toast_t = 2.6

    def update(self, dt):
        self.t += dt
        if self.toast_t > 0:
            self.toast_t -= dt
            if self.toast_t <= 0:
                self.toast = ""

    # ----- Koordinaten --------------------------------------------------
    def _to_course(self, pos):
        x = (pos[0] - self.ox) / self.scale
        y = (pos[1] - self.oy) / self.scale
        return self._snap(x), self._snap(y)

    def _snap(self, v):
        return round(v) if self.grid else round(v, 1)

    def _inside(self, pos):
        return self.canvas.collidepoint(pos)

    def _clampx(self, x):
        return max(BORDER, min(self.hole["w"] - BORDER, x))

    def _clampy(self, y):
        return max(BORDER, min(self.hole["h"] - BORDER, y))

    def _pick(self, x, y):
        """Oberstes Hindernis unter (x, y) - (typ, index) oder None."""
        for key in reversed(PALETTE_KEYS):
            items = self.hole[key]
            for i in range(len(items) - 1, -1, -1):
                it = items[i]
                if key in ("bumpers", "magnets", "spinners", "mills"):
                    if math.hypot(x - it[0], y - it[1]) <= max(3.0, it[2]):
                        return (key, i)
                elif key == "tunnels":
                    if (math.hypot(x - it[0], y - it[1]) <= max(3.0, it[4])
                            or math.hypot(x - it[2], y - it[3]) <= max(3.0, it[4])):
                        return (key, i)
                else:
                    if (it[0] <= x <= it[0] + it[2]
                            and it[1] <= y <= it[1] + it[3]):
                        return (key, i)
        return None

    # ----- Eingabe ------------------------------------------------------
    def handle(self, event):
        if self.picking:
            return self._handle_template(event)
        if event.kind == InputEvent.KEYDOWN:
            return self._handle_key(event)
        if event.kind == InputEvent.MOUSEDOWN:
            return self._handle_down(event)
        if event.kind == InputEvent.MOUSEMOVE:
            if self.drag:
                x, y = self._to_course(event.pos)
                self.drag = self.drag[:3] + (self._clampx(x), self._clampy(y))
            elif self.move:
                self._do_move(event.pos)
            return True
        if event.kind == InputEvent.MOUSEUP:
            return self._handle_up(event)
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
        elif k in ("Delete", "BackSpace"):
            self._delete_selected()
        elif k in ("u", "U"):
            self._undo()
        elif k in ("y", "Y"):
            self._redo()
        elif k in ("g", "G"):
            self._toggle_grid()
        elif k in ("s", "S"):
            self._save()
        elif k in ("Return", "KP_Enter"):
            self._test()
        return True

    def _handle_down(self, event):
        pos = event.pos
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
        for key, (minus, _val, plus) in self.num_rects.items():
            if minus.collidepoint(pos):
                self._step_num(key, -1)
                return True
            if plus.collidepoint(pos):
                self._step_num(key, 1)
                return True
        for key, rc in self.edit_rects.items():
            if rc.collidepoint(pos):
                {"template": self._open_template, "grid": self._toggle_grid,
                 "undo": self._undo, "redo": self._redo,
                 "clear": self._clear, "test": self._test}[key]()
                return True
        for key, rc in self.pal_rects:
            if rc.collidepoint(pos):
                self.tool = key
                self.pending = None
                self.sel = None
                self.game.play_sound("select")
                return True
        if self._param_click(pos):
            return True
        if self._inside(pos):
            self._canvas_down(pos)
        return True

    def _canvas_down(self, pos):
        x, y = self._to_course(pos)
        x, y = self._clampx(x), self._clampy(y)
        tool = self.tool
        if tool == "tee":
            self._push()
            self.hole["tee"] = (x, y)
            self.game.play_sound("click")
            return
        if tool == "cup":
            self._push()
            self.hole["cup"] = (x, y)
            self.game.play_sound("click")
            return
        if tool == "erase":
            hit = self._pick(x, y)
            if hit:
                self._push()
                del self.hole[hit[0]][hit[1]]
                self.sel = None
                self.game.play_sound("hit")
            return
        if tool == "select":
            hit = self._pick(x, y)
            self.sel = hit
            if hit:
                it = self.hole[hit[0]][hit[1]]
                self.move = (hit[0], hit[1], it[0] - x, it[1] - y)
                self.game.play_sound("move")
            return
        kind = PAL[tool][1]
        if kind == "circle":
            self._push()
            self.hole[tool].append(new_item(tool, x, y))
            self.sel = (tool, len(self.hole[tool]) - 1)
            self.game.play_sound("click")
        elif kind == "pair":
            if self.pending is None:
                self.pending = (x, y)          # erster Klick: Eingang
                self.game.play_sound("select")
            else:
                self._push()
                self.hole[tool].append(
                    new_item(tool, self.pending[0], self.pending[1],
                             x2=x, y2=y))
                self.sel = (tool, len(self.hole[tool]) - 1)
                self.pending = None
                self.game.play_sound("click")
        else:
            self.drag = (tool, x, y, x, y)

    def _do_move(self, pos):
        key, idx, offx, offy = self.move
        if idx >= len(self.hole[key]):
            self.move = None
            return
        x, y = self._to_course(pos)
        it = list(self.hole[key][idx])
        nx, ny = self._clampx(x + offx), self._clampy(y + offy)
        if key == "tunnels":
            dx, dy = nx - it[0], ny - it[1]
            it[0], it[1] = nx, ny
            it[2], it[3] = self._clampx(it[2] + dx), self._clampy(it[3] + dy)
        else:
            it[0], it[1] = nx, ny
        self.hole[key][idx] = tuple(it)
        self.dirty = True

    def _handle_up(self, event):
        if self.move:
            self.move = None
            return True
        if not self.drag:
            return True
        key, x0, y0, x1, y1 = self.drag
        self.drag = None
        x, y = min(x0, x1), min(y0, y1)
        w, h = abs(x1 - x0), abs(y1 - y0)
        if w < 2.0 or h < 2.0:            # zu klein: als Fehlklick verwerfen
            return True
        self._push()
        self.hole[key].append(new_item(key, x, y, w, h))
        self.sel = (key, len(self.hole[key]) - 1)
        self.game.play_sound("click")
        return True

    def _delete_selected(self):
        if not self.sel:
            return
        key, idx = self.sel
        if idx < len(self.hole[key]):
            self._push()
            del self.hole[key][idx]
        self.sel = None
        self.game.play_sound("hit")

    # ----- Kopfzeilen-Aktionen ------------------------------------------
    def _step_num(self, key, d):
        self._push()
        if key == "par":
            self.hole["par"] = max(ugc.MIN_PAR,
                                   min(ugc.MAX_PAR, int(self.hole["par"]) + d))
        elif key == "width":
            self.hole["w"] = max(ugc.MIN_W, min(ugc.MAX_W,
                                                self.hole["w"] + d * ugc.STEP_W))
            self._clamp_all()
            self._resize_canvas()
        else:
            self.hole["h"] = max(ugc.MIN_H, min(ugc.MAX_H,
                                                self.hole["h"] + d * ugc.STEP_H))
            self._clamp_all()
            self._resize_canvas()
        self.game.play_sound("move")

    def _clamp_all(self):
        """Nach dem Verkleinern alles ins Feld holen, was herausragt."""
        cw, ch = self.hole["w"], self.hole["h"]
        self.hole["tee"] = (self._clampx(self.hole["tee"][0]),
                            self._clampy(self.hole["tee"][1]))
        self.hole["cup"] = (self._clampx(self.hole["cup"][0]),
                            self._clampy(self.hole["cup"][1]))
        for key in PALETTE_KEYS:
            kept = []
            for it in self.hole[key]:
                it = list(it)
                if key in ROUND_KEYS:
                    if key == "tunnels":
                        it[0], it[1] = self._clampx(it[0]), self._clampy(it[1])
                        it[2], it[3] = self._clampx(it[2]), self._clampy(it[3])
                    else:
                        it[0], it[1] = self._clampx(it[0]), self._clampy(it[1])
                else:
                    it[2] = min(it[2], cw - 2 * BORDER)
                    it[3] = min(it[3], ch - 2 * BORDER)
                    it[0] = max(BORDER, min(cw - BORDER - it[2], it[0]))
                    it[1] = max(BORDER, min(ch - BORDER - it[3], it[1]))
                kept.append(tuple(it))
            self.hole[key] = kept

    def _toggle_grid(self):
        self.grid = not self.grid
        self.game.set_grid_snap(self.grid)
        self.game.play_sound("select")

    def _clear(self):
        self._push()
        for key in PALETTE_KEYS:
            self.hole[key] = []
        self.sel = None
        self.game.play_sound("hit")

    def _open_template(self):
        self.picking = True
        self.game.play_sound("click")

    def _handle_template(self, event):
        if event.kind == InputEvent.MOUSEDOWN:
            for key, rc in self.tpl_rects:
                if rc.collidepoint(event.pos):
                    self._apply_template(key)
                    return True
            self.picking = False
            return True
        if event.kind == InputEvent.KEYDOWN and event.key == "Escape":
            self.picking = False
        return True

    def _apply_template(self, key):
        self._push()
        tpl = template(key)
        for field in ("par", "tee", "cup", "w", "h"):
            self.hole[field] = tpl[field]
        for k in PALETTE_KEYS:
            self.hole[k] = list(tpl[k])
        self.sel = None
        self.picking = False
        self._resize_canvas()
        self._toast(t("golf.ugc.tpl." + key))
        self.game.play_sound("point")

    # ----- Parameterleiste ----------------------------------------------
    def _params(self):
        """(Typ, Eintrag, Parameterliste) des gewählten Hindernisses."""
        if not self.sel:
            return None, None, ()
        key, idx = self.sel
        if idx >= len(self.hole[key]):
            self.sel = None
            return None, None, ()
        return key, self.hole[key][idx], PAL[key][2]

    def _param_rects(self):
        """Rechtecke der Parameterleiste: [(param, minus, wert, plus)]."""
        key, item, params = self._params()
        if not params:
            return []
        w = self.game.width
        bh = self.par_h - 10
        step_w = max(16, int(w * 0.036))
        val_w = max(34, int(w * 0.07))
        grp = 2 * step_w + val_w
        total = len(params) * grp + (len(params) - 1) * 8
        x = max(10, (w - total) // 2)
        y = self.par_top + 5
        out = []
        for p in params:
            out.append((p, pygame.Rect(x, y, step_w, bh),
                        pygame.Rect(x + step_w, y, val_w, bh),
                        pygame.Rect(x + step_w + val_w, y, step_w, bh)))
            x += grp + 8
        return out

    def _param_click(self, pos):
        for p, minus, _val, plus in self._param_rects():
            if minus.collidepoint(pos):
                self._step_param(p, -1)
                return True
            if plus.collidepoint(pos):
                self._step_param(p, 1)
                return True
        return False

    def _step_param(self, p, d):
        key, item, _ = self._params()
        if item is None:
            return
        self._push()
        it = list(item)
        kind = p["kind"]
        if kind in ("dir", "movedir"):
            i = (_dir_index(it[p["idx"]], it[p["idx"] + 1]) + d) % len(DIRS)
            ux, uy = DIRS[i]
            if kind == "movedir":
                span = math.hypot(it[4], it[5]) or 20.0
                it[4], it[5] = ux * span, uy * span
            else:
                it[p["idx"]], it[p["idx"] + 1] = ux, uy
        elif kind == "mag":
            # Rampen: Betrag der Beschleunigung, Richtung bleibt.
            ax, ay = it[4], it[5]
            n = math.hypot(ax, ay) or 1.0
            v = max(p["lo"], min(p["hi"], n + d * p["step"]))
            it[4], it[5] = ax / n * v, ay / n * v
        elif kind == "span":
            # Wanderblock: Länge des Wegs, Richtung bleibt.
            dx, dy = it[4], it[5]
            n = math.hypot(dx, dy) or 1.0
            v = max(p["lo"], min(p["hi"], n + d * p["step"]))
            it[4], it[5] = dx / n * v, dy / n * v
        else:
            i = p["idx"]
            v = it[i] + d * p["step"]
            v = max(p["lo"], min(p["hi"], v))
            it[i] = round(v, 2)
        self.hole[key][self.sel[1]] = tuple(it)
        self.game.play_sound("move")

    def _param_text(self, p, item):
        """Anzeigetext eines Parameters - None heißt "als Pfeil zeichnen"."""
        kind = p["kind"]
        if kind in ("dir", "movedir"):
            return None
        if kind in ("mag", "span"):
            return "%d" % round(math.hypot(item[4], item[5]))
        v = item[p["idx"]]
        return "%d" % round(v) if abs(v - round(v)) < 0.05 else "%.1f" % v

    # ----- Speichern / Verlassen ----------------------------------------
    def _save(self):
        name = self.f_name.text.strip()
        map_id = self.f_id.text.strip() or ugc.slug(name)
        if not map_id:
            self.err = t("golf.ugc.err.id_empty")
            return self._fail()
        if not ugc.valid_id(map_id):
            self.err = t("golf.ugc.err.id_chars")
            return self._fail()
        if not name:
            self.err = t("golf.ugc.err.name")
            return self._fail()
        taken = {m["id"] for m in ugc.load_maps() if m["id"] != self.orig_id}
        if map_id in taken:
            self.err = t("golf.ugc.err.id_dup")
            return self._fail()
        bad = validate(self.hole)
        if bad:
            self.err = t("golf.ugc.err." + bad)
            return self._fail()
        if not swear.all_clean(name, map_id):
            self.err = t("golf.ugc.err.swear")
            return self._fail()
        m = dict(self.map)
        m.update(self.hole)
        m["id"] = map_id
        m["name"] = name
        if self.orig_id and self.orig_id != map_id:
            ugc.delete_map(self.orig_id)     # Umbenennen = altes Feld räumen
        ok, why = ugc.save_map(m)
        if not ok:
            self.err = t("golf.ugc.err." + (why if why in
                                            ("swear", "full", "io", "id")
                                            else "invalid"))
            return self._fail()
        self.map = m
        self.orig_id = map_id
        self.dirty = False
        self.f_id.set_text(map_id)
        self._toast(t("golf.ugc.saved"))
        self.game.play_sound("win")
        return True

    def _fail(self):
        self.game.play_sound("hit")
        return False

    def _test(self):
        """Bahn sofort ausprobieren - danach geht es in den Editor zurück."""
        bad = validate(self.hole)
        if bad:
            self.err = t("golf.ugc.err." + bad)
            return self._fail()
        self.game.ugc_test(copy.deepcopy(self.hole))

    def _back(self):
        if self.dirty and not self.confirm_back:
            self.confirm_back = True
            self._toast(t("golf.ugc.unsaved"))
            self.game.play_sound("select")
            return
        self.game.ugc_close_editor()

    # ----- Zeichnen -----------------------------------------------------
    def draw(self, s):
        g = self.game
        self._draw_canvas(s)
        self._draw_head(s)
        self._draw_palette(s)
        self._draw_params(s)
        if self.toast:
            img = self.tiny.render(self.toast, True, ui.TEXT)
            w, h = img.get_width() + 24, img.get_height() + 10
            r = pygame.Rect(self.canvas.centerx - w // 2,
                            self.canvas.bottom - h - 6, w, h)
            ui.draw_panel(s, r, radius=h // 2, shadow=False)
            pygame.draw.rect(s, ui.ACCENT2, r, 1, border_radius=h // 2)
            s.blit(img, img.get_rect(center=r.center))
        if self.picking:
            self._draw_template(s)

    def _draw_canvas(self, s):
        view = self.view()
        draw.draw_course(s, self.hole, view, self.t, self.t,
                         cup_r=3.0, tee=True)
        if self.grid:
            self._draw_grid(s, view)
        # Auswahl hervorheben
        if self.sel:
            key, idx = self.sel
            if idx < len(self.hole[key]):
                self._outline(s, view, key, self.hole[key][idx], self.game.accent)
        # Vorschau beim Aufziehen
        if self.drag:
            key, x0, y0, x1, y1 = self.drag
            rc = view.rect_px((min(x0, x1), min(y0, y1),
                               max(1.0, abs(x1 - x0)), max(1.0, abs(y1 - y0))))
            pygame.draw.rect(s, SWATCH.get(key, ui.ACCENT), rc, 2,
                             border_radius=3)
        # Erster Klick eines Rohrs
        if self.pending:
            px, py = view.project(*self.pending)
            pygame.draw.circle(s, draw.COL_TUNNEL, (int(px), int(py)),
                               max(4, int(5 * self.scale)), 2)
        pygame.draw.rect(s, ui.BORDER, self.canvas.inflate(8, 8), 1,
                         border_radius=8)

    def _draw_grid(self, s, view):
        """Feines Raster - nur so dicht, dass es noch hilft."""
        step = 10.0
        while step * self.scale < 14:
            step *= 2
        col = ui.mix(draw.COL_GREEN, (255, 255, 255), 0.10)
        x = step
        while x < self.hole["w"]:
            px = int(view.project(x, 0)[0])
            pygame.draw.line(s, col, (px, self.canvas.y),
                             (px, self.canvas.bottom), 1)
            x += step
        y = step
        while y < self.hole["h"]:
            py = int(view.project(0, y)[1])
            pygame.draw.line(s, col, (self.canvas.x, py),
                             (self.canvas.right, py), 1)
            y += step

    def _outline(self, s, view, key, it, col):
        if key in ("bumpers", "magnets", "spinners", "mills"):
            px, py = view.project(it[0], it[1])
            pygame.draw.circle(s, col, (int(px), int(py)),
                               max(4, int(it[2] * self.scale)) + 3, 2)
        elif key == "tunnels":
            for (cx, cy) in ((it[0], it[1]), (it[2], it[3])):
                px, py = view.project(cx, cy)
                pygame.draw.circle(s, col, (int(px), int(py)),
                                   max(4, int(it[4] * self.scale)) + 3, 2)
        else:
            pygame.draw.rect(s, col, view.rect_px(it).inflate(6, 6), 2,
                             border_radius=4)

    def _draw_head(self, s):
        g = self.game
        _btn(s, self.back_rect, t("golf.ugc.btn_back"), self.tiny)
        self.f_name.draw(s, self.name_rect, self.tiny, focused=(self.focus == 0),
                         invalid=bool(self.err) and self.focus == 0)
        self.f_id.draw(s, self.id_rect, self.tiny, focused=(self.focus == 1),
                       invalid=bool(self.err) and self.focus == 1)
        _btn(s, self.save_rect, t("golf.ugc.btn_save"), self.tiny,
             on=self.dirty, accent=g.accent)
        for key, (minus, val, plus) in self.num_rects.items():
            _btn(s, minus, "-", self.tiny)
            _btn(s, plus, "+", self.tiny)
            num = {"par": int(self.hole["par"]), "width": int(self.hole["w"]),
                   "height": int(self.hole["h"])}[key]
            pygame.draw.rect(s, ui.PANEL, val, border_radius=7)
            pygame.draw.rect(s, ui.BORDER, val, 1, border_radius=7)
            img = self.tiny.render("%s %d" % (t("golf.ugc.short_" + key), num),
                                   True, ui.TEXT)
            if img.get_width() > val.w - 4:
                img = self.tiny.render(str(num), True, ui.TEXT)
            s.blit(img, img.get_rect(center=val.center))
        for key in TEXT_BUTTONS:
            _btn(s, self.edit_rects[key], t("golf.ugc.btn_" + key), self.tiny,
                 accent=g.accent)
        for key in ICON_BUTTONS:
            rc = self.edit_rects[key]
            on = (key == "grid" and self.grid)
            enabled = {"undo": bool(self.undo_stack),
                       "redo": bool(self.redo_stack)}.get(key, True)
            pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc,
                             border_radius=7)
            pygame.draw.rect(s, g.accent if on else ui.BORDER, rc,
                             2 if on else 1, border_radius=7)
            _edit_icon(s, rc, key,
                       ui.TEXT if (on or enabled) else ui.TEXT_FAINT)
        if self.err:
            img = self.tiny.render(self.err, True, ui.RED)
            s.blit(img, (12, self.head_bottom - 2))

    def _draw_palette(self, s):
        for key, rc in self.pal_rects:
            on = (self.tool == key)
            pygame.draw.rect(s, ui.BTN_SEL if on else ui.BTN, rc,
                             border_radius=6)
            pygame.draw.rect(s, self.game.accent if on else ui.BORDER, rc,
                             2 if on else 1, border_radius=6)
            if key in TOOLS:
                _tool_icon(s, rc, key, ui.TEXT if on else ui.TEXT_DIM)
            else:
                inner = rc.inflate(-10, -10)
                col = SWATCH[key]
                if key in ROUND_KEYS:
                    pygame.draw.circle(s, col, inner.center,
                                       max(3, min(inner.w, inner.h) // 2))
                else:
                    pygame.draw.rect(s, col, inner, border_radius=3)
        # Name des aktiven Werkzeugs - dafür ist auf den Knöpfen kein Platz.
        name = t(("golf.ugc.tool." if self.tool in TOOLS else "golf.ugc.obj.")
                 + self.tool)
        img = self.tiny.render(name, True, self.game.accent)
        s.blit(img, img.get_rect(midbottom=(self.pal_rects[-1][1].centerx,
                                            self.par_top - 2)))

    def _draw_params(self, s):
        rects = self._param_rects()
        if not rects:
            hint = t("golf.ugc.hint_tunnel") if self.pending else (
                t("golf.ugc.hint_select") if self.tool == "select"
                else t("golf.ugc.hint_place"))
            img = self.tiny.render(hint, True, ui.TEXT_FAINT)
            s.blit(img, img.get_rect(center=(self.game.width // 2,
                                             self.par_top + self.par_h // 2)))
            return
        key, item, _ = self._params()
        for p, minus, val, plus in rects:
            _btn(s, minus, "-", self.tiny)
            _btn(s, plus, "+", self.tiny)
            pygame.draw.rect(s, ui.PANEL, val, border_radius=7)
            pygame.draw.rect(s, ui.BORDER, val, 1, border_radius=7)
            txt = self._param_text(p, item)
            if txt is None:                  # Richtung: als Pfeil zeichnen
                i = _dir_index(item[p["idx"]], item[p["idx"] + 1])
                _arrow(s, val.center, DIRS[i][0], DIRS[i][1],
                       max(4, val.h // 4), ui.TEXT)
            else:
                img = self.tiny.render(txt, True, ui.TEXT)
                s.blit(img, img.get_rect(center=val.center))
            lab = self.tiny.render(t("golf.ugc.par." + p["key"]), True,
                                   ui.TEXT_FAINT)
            s.blit(lab, lab.get_rect(midbottom=(val.centerx, val.top - 1)))

    def _draw_template(self, s):
        g = self.game
        veil = pygame.Surface((g.width, g.height), pygame.SRCALPHA)
        veil.fill((0, 0, 0, 160))
        s.blit(veil, (0, 0))
        ui.draw_panel(s, self.tpl_rect, accent_top=g.accent)
        head = self.small.render(t("golf.ugc.tpl_title"), True, g.accent)
        s.blit(head, head.get_rect(midtop=(self.tpl_rect.centerx,
                                           self.tpl_rect.y + 10)))
        for key, rc in self.tpl_rects:
            _btn(s, rc, t("golf.ugc.tpl." + key), self.tiny)
