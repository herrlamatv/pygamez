# -*- coding: utf-8 -*-
"""
ngb.py
======
NGB - die **visuelle Personalisierung** der Spielesammlung (Skins/"Mods", die
ausschließlich die Optik verändern, niemals die Spiellogik).

Hier laufen alle rein optischen Einstellungen zusammen:

- Die aktive **Kopffarbe** der Snake-Schlange (Vorlagen + eigene Farbe).
- Das **Raster-Overlay**: ein Schachbrett-Muster, das die horizontalen und
  vertikalen Gitterlinien in einer wählbaren Farbreihenfolge markiert. So sieht
  man auch auf großen Feldern von Weitem, welche Reihe/Spalte wo liegt (5
  Vorlagen + eigene A/B-Farben).
- Das **Personalisierungs-Menü** (:class:`PersonalizeMenu`) mit zwei Tabs
  (Kopf / Raster), das über den Pinsel-Knopf im Snake-Setup geöffnet wird.

Gespeichert wird alles in **mem-ngb.json** (neben diesem Modul); beim ersten
Zugriff wird die Datei geladen, bei jeder Änderung zurückgeschrieben. Da NGB nur
Optik betrifft, hat es eine eigene Datei getrennt von Spielständen/Highscores.
"""

import json
import os
import sys

import pygame

import i18n
from game_base import InputEvent

# In einer PyInstaller-.exe (sys.frozen) zeigt __file__ in den temporären
# Entpack-Ordner, der beim Beenden verschwindet - dann neben der .exe speichern.
if getattr(sys, "frozen", False):
    _DIR = os.path.dirname(sys.executable)
else:
    _DIR = os.path.dirname(os.path.abspath(__file__))
_FILE = os.path.join(_DIR, "mem-ngb.json")

# ----- Kopffarben-Vorlagen (nur Optik) ----------------------------------
# 4x Blau-Türkis (von "mehr Blau" zu "mehr Türkis"), dann Rot, Orange, Eigene.
HEAD_PRESETS = [
    {"id": "aqua1",  "color": (90, 165, 245)},   # mehr Blau
    {"id": "aqua2",  "color": (72, 195, 240)},
    {"id": "aqua3",  "color": (58, 218, 224)},   # Standard - schön türkis
    {"id": "aqua4",  "color": (52, 234, 198)},   # mehr Türkis
    {"id": "red",    "color": (255, 95, 95)},
    {"id": "orange", "color": (255, 160, 60)},
    {"id": "custom", "color": None},             # nutzt die gespeicherte RGB-Farbe
]
_HEAD_IDS = [p["id"] for p in HEAD_PRESETS]
DEFAULT_HEAD_ID = "aqua3"                         # Standard: der Kopf ist türkis
DEFAULT_CUSTOM = (200, 120, 255)

# ----- Raster-Vorlagen (Schachbrett-Farbreihenfolgen) -------------------
# "off" = kein Overlay. Jede Vorlage ist eine Farb-Sequenz; die Zellen/Linien
# durchlaufen sie im Schachbrett-Takt. "custom" nutzt zwei eigene Farben (A/B).
GRID_PRESETS = [
    {"id": "off",   "seq": None},
    {"id": "grid1", "seq": [(44, 66, 108), (26, 40, 70)]},                 # Blau
    {"id": "grid2", "seq": [(36, 74, 58), (24, 50, 42)]},                  # Grün
    {"id": "grid3", "seq": [(70, 46, 98), (44, 32, 66)]},                  # Violett
    {"id": "grid4", "seq": [(92, 68, 34), (60, 46, 26)]},                  # Bernstein
    {"id": "grid5", "seq": [(58, 62, 74), (30, 33, 42), (44, 47, 58)]},    # Kontrast (3er)
    {"id": "custom", "seq": None},                                          # nutzt grid_custom
]
_GRID_IDS = [p["id"] for p in GRID_PRESETS]
DEFAULT_GRID_ID = "off"
DEFAULT_GRID_CUSTOM = [[46, 64, 96], [28, 40, 66]]   # A, B (eigenes Schachbrett)

# ----- Banner (Multiplikator-Einblendung oben mittig) -------------------
DEFAULT_BANNER = {"on": True, "size": 1.0, "opacity": 1.0}
BANNER_SIZE_MIN, BANNER_SIZE_MAX = 0.6, 1.6      # kleiner .. größer
BANNER_OP_MIN, BANNER_OP_MAX = 0.2, 1.0          # transparenter .. deckend

# ----- Menü-Farben (lokal, um keine Spielmodule zu importieren) ---------
COL_BG = (14, 15, 24)
COL_TEXT = (232, 232, 238)
COL_DIM = (150, 158, 176)
COL_ACCENT = (90, 160, 240)
COL_BTN = (44, 50, 66)
COL_BTN_ON = (60, 120, 80)
COL_SEL = (255, 255, 255)
COL_TRACK = (36, 40, 54)
COL_TAB_OFF = (34, 38, 52)

_state = None                                    # gecachte Daten aus mem-ngb.json


# ===================================================== Laden / Speichern
def _clamp_color(c, fallback):
    try:
        col = tuple(max(0, min(255, int(v))) for v in c)[:3]
        return col if len(col) == 3 else tuple(fallback)
    except (TypeError, ValueError):
        return tuple(fallback)


def _load():
    global _state
    if _state is not None:
        return _state
    data = {}
    try:
        with open(_FILE, "r", encoding="utf-8") as f:
            raw = json.load(f)
            if isinstance(raw, dict):
                data = raw
    except (OSError, json.JSONDecodeError, ValueError):
        data = {}

    hid = data.get("head_id", DEFAULT_HEAD_ID)
    if hid not in _HEAD_IDS:
        hid = DEFAULT_HEAD_ID
    gid = data.get("grid_id", DEFAULT_GRID_ID)
    if gid not in _GRID_IDS:
        gid = DEFAULT_GRID_ID

    gc = data.get("grid_custom", DEFAULT_GRID_CUSTOM)
    if not isinstance(gc, list) or len(gc) != 2:
        gc = DEFAULT_GRID_CUSTOM
    grid_custom = [list(_clamp_color(gc[0], DEFAULT_GRID_CUSTOM[0])),
                   list(_clamp_color(gc[1], DEFAULT_GRID_CUSTOM[1]))]

    def _num(v, lo, hi, fallback):
        try:
            return max(lo, min(hi, float(v)))
        except (TypeError, ValueError):
            return fallback

    _state = {
        "head_id": hid,
        "head_custom": _clamp_color(data.get("head_custom", DEFAULT_CUSTOM), DEFAULT_CUSTOM),
        "grid_id": gid,
        "grid_custom": grid_custom,
        "banner_on": bool(data.get("banner_on", DEFAULT_BANNER["on"])),
        "banner_size": _num(data.get("banner_size", DEFAULT_BANNER["size"]),
                            BANNER_SIZE_MIN, BANNER_SIZE_MAX, DEFAULT_BANNER["size"]),
        "banner_opacity": _num(data.get("banner_opacity", DEFAULT_BANNER["opacity"]),
                               BANNER_OP_MIN, BANNER_OP_MAX, DEFAULT_BANNER["opacity"]),
    }
    return _state


def _save():
    st = _load()
    try:
        with open(_FILE, "w", encoding="utf-8") as f:
            json.dump({"head_id": st["head_id"],
                       "head_custom": list(st["head_custom"]),
                       "grid_id": st["grid_id"],
                       "grid_custom": [list(st["grid_custom"][0]),
                                       list(st["grid_custom"][1])],
                       "banner_on": st["banner_on"],
                       "banner_size": round(st["banner_size"], 3),
                       "banner_opacity": round(st["banner_opacity"], 3)},
                      f, ensure_ascii=False, indent=2)
    except OSError:
        pass


# ===================================================== Öffentliche API (Kopf)
def head_preset(pid):
    for p in HEAD_PRESETS:
        if p["id"] == pid:
            return p
    return HEAD_PRESETS[0]


# Kompatibilitäts-Alias (frühere Versionen nutzten preset()).
preset = head_preset


def get_head_id():
    return _load()["head_id"]


def set_head_id(pid):
    if pid in _HEAD_IDS:
        _load()["head_id"] = pid
        _save()


def get_custom():
    return tuple(_load()["head_custom"])


def set_custom(color):
    _load()["head_custom"] = _clamp_color(color, DEFAULT_CUSTOM)
    _save()


def head_color():
    """Aktive Kopffarbe der Schlange (Spieler 1) - rein visuell."""
    st = _load()
    if st["head_id"] == "custom":
        return tuple(st["head_custom"])
    return head_preset(st["head_id"])["color"]


# ===================================================== Öffentliche API (Raster)
def grid_preset(pid):
    for p in GRID_PRESETS:
        if p["id"] == pid:
            return p
    return GRID_PRESETS[0]


def get_grid_id():
    return _load()["grid_id"]


def set_grid_id(pid):
    if pid in _GRID_IDS:
        _load()["grid_id"] = pid
        _save()


def get_grid_custom():
    return [tuple(c) for c in _load()["grid_custom"]]


def set_grid_custom(colors):
    st = _load()
    st["grid_custom"] = [list(_clamp_color(colors[0], DEFAULT_GRID_CUSTOM[0])),
                         list(_clamp_color(colors[1], DEFAULT_GRID_CUSTOM[1]))]
    _save()


def grid_sequence():
    """Aktive Farbreihenfolge fürs Raster-Overlay - oder None (aus)."""
    st = _load()
    gid = st["grid_id"]
    if gid == "off":
        return None
    if gid == "custom":
        return [tuple(c) for c in st["grid_custom"]]
    return grid_preset(gid)["seq"]


# ===================================================== Öffentliche API (Banner)
def get_banner():
    """Banner-Einstellungen: dict(on, size, opacity) - rein visuell."""
    st = _load()
    return {"on": st["banner_on"], "size": st["banner_size"],
            "opacity": st["banner_opacity"]}


def set_banner_on(value):
    _load()["banner_on"] = bool(value)
    _save()


def set_banner_size(value):
    _load()["banner_size"] = max(BANNER_SIZE_MIN, min(BANNER_SIZE_MAX, float(value)))
    _save()


def set_banner_opacity(value):
    _load()["banner_opacity"] = max(BANNER_OP_MIN, min(BANNER_OP_MAX, float(value)))
    _save()


def open_head_color_menu(width, height, play_sound=None):
    """Erzeugt das Personalisierungs-Menü (wird vom Snake-Setup aufgerufen)."""
    return PersonalizeMenu(width, height, play_sound)


# ===================================================== Menü
class PersonalizeMenu:
    """Personalisierungs-Menü mit zwei Tabs (nur Optik):

    - **Kopf**:   Kopffarbe der Schlange (Vorlage oder eigene RGB-Farbe).
    - **Raster**: Schachbrett-Overlay mit wählbarer Farbreihenfolge (Vorlage
      oder zwei eigene Farben A/B) für einen besseren Überblick.

    Änderungen werden sofort in mem-ngb.json gespeichert. ``done`` wird True,
    sobald das Menü geschlossen werden soll.
    """

    CHANNELS = ("R", "G", "B")
    CH_COL = {"R": (235, 90, 90), "G": (90, 210, 120), "B": (90, 150, 245)}

    def __init__(self, width, height, play_sound=None):
        self.width = width
        self.height = height
        self.done = False
        self._play = play_sound or (lambda name: None)
        # Erfolg "Ganz mein Stil": die Personalisierung entdeckt.
        import achievements
        achievements.event("painter")
        self._big = pygame.font.SysFont("consolas", 32, bold=True)
        self._font = pygame.font.SysFont("consolas", 20)
        self._small = pygame.font.SysFont("consolas", 15)
        self._tiny = pygame.font.SysFont("consolas", 12)

        self.tab = "head"
        self._sel = get_head_id()
        self._custom = list(get_custom())
        self._grid_sel = get_grid_id()
        self._grid_custom = [list(c) for c in get_grid_custom()]
        self._grid_edit = 0                      # 0 = Farbe A, 1 = Farbe B
        self._build_layout()

    # ----- Layout -------------------------------------------------------
    def _build_layout(self):
        cx = self.width // 2
        tabw, tgap = 120, 8
        names = ("head", "grid", "banner")
        total = len(names) * tabw + (len(names) - 1) * tgap
        x = cx - total // 2
        self.tab_rects = {}
        for nm in names:
            self.tab_rects[nm] = pygame.Rect(x, 46, tabw, 30)
            x += tabw + tgap

        # --- Kopf-Seite: 7 Kacheln in 2 Reihen ---
        tw, th, gap = 92, 46, 12
        self.head_tiles = []
        y = 116
        for row in (HEAD_PRESETS[:4], HEAD_PRESETS[4:]):
            total = len(row) * tw + (len(row) - 1) * gap
            x = cx - total // 2
            for p in row:
                self.head_tiles.append((pygame.Rect(x, y, tw, th), p))
                x += tw + gap
            y += th + 18 + gap
        self._sliders_head = self._make_sliders(y + 4)

        # --- Raster-Seite: 7 Kacheln in einer Reihe ---
        gtw, gth, ggap = 74, 46, 8
        self.grid_tiles = []
        total = len(GRID_PRESETS) * gtw + (len(GRID_PRESETS) - 1) * ggap
        x = cx - total // 2
        gy = 116
        for p in GRID_PRESETS:
            self.grid_tiles.append((pygame.Rect(x, gy, gtw, gth), p))
            x += gtw + ggap
        aby = gy + gth + 22 + 12
        self.grid_ab = {"A": pygame.Rect(cx - 104, aby, 92, 26),
                        "B": pygame.Rect(cx + 12, aby, 92, 26)}
        self._sliders_grid = self._make_sliders(aby + 42)

        # --- Banner-Seite: An/Aus + Größe + Deckkraft ---
        self.banner_toggle = pygame.Rect(cx - 150, 118, 300, 36)
        self.banner_slider = {
            "size": self._one_slider(200),
            "opacity": self._one_slider(258),
        }

        self.done_rect = pygame.Rect(cx - 80, self.height - 78, 160, 32)

    def _one_slider(self, top):
        cx = self.width // 2
        track = pygame.Rect(cx - 84, top, 168, 14)
        return dict(track=track,
                    minus=pygame.Rect(track.left - 30, top - 4, 22, 22),
                    plus=pygame.Rect(track.right + 8, top - 4, 22, 22))

    def _make_sliders(self, top):
        cx = self.width // 2
        d, sy = {}, top
        for ch in self.CHANNELS:
            track = pygame.Rect(cx - 100, sy, 200, 14)
            d[ch] = dict(track=track,
                         minus=pygame.Rect(track.left - 30, sy - 4, 22, 22),
                         plus=pygame.Rect(track.right + 8, sy - 4, 22, 22))
            sy += 30
        return d

    # ----- Zustand ------------------------------------------------------
    def _active_sliders(self):
        return self._sliders_head if self.tab == "head" else self._sliders_grid

    def _editable(self):
        """Die aktuell per RGB editierbare Farbe (3er-Liste) - oder None."""
        if self.tab == "head" and self._sel == "custom":
            return self._custom
        if self.tab == "grid" and self._grid_sel == "custom":
            return self._grid_custom[self._grid_edit]
        return None

    def _persist(self):
        if self.tab == "head":
            set_custom(self._custom)
        else:
            set_grid_custom(self._grid_custom)

    def _head_color(self):
        return tuple(self._custom) if self._sel == "custom" \
            else head_preset(self._sel)["color"]

    def _grid_seq(self):
        if self._grid_sel == "off":
            return None
        if self._grid_sel == "custom":
            return [tuple(c) for c in self._grid_custom]
        return grid_preset(self._grid_sel)["seq"]

    def _select_head(self, pid):
        self._sel = pid
        set_head_id(pid)
        self._play("select")

    def _select_grid(self, pid):
        self._grid_sel = pid
        set_grid_id(pid)
        self._play("select")

    def _slider_set(self, ed, ch, value):
        ed[self.CHANNELS.index(ch)] = max(0, min(255, int(value)))
        self._persist()

    def _step(self, ed, ch, delta):
        self._slider_set(ed, ch, ed[self.CHANNELS.index(ch)] + delta)
        self._play("click")

    @staticmethod
    def _value_from_x(track, x):
        return round((x - track.left) / max(1, track.width) * 255)

    def _close(self):
        self.done = True
        self._play("click")

    # ----- Banner-Steuerung ---------------------------------------------
    @staticmethod
    def _range_from_x(track, x, lo, hi):
        frac = max(0.0, min(1.0, (x - track.left) / max(1, track.width)))
        return lo + frac * (hi - lo)

    @staticmethod
    def _banner_range(kind):
        return (BANNER_SIZE_MIN, BANNER_SIZE_MAX) if kind == "size" \
            else (BANNER_OP_MIN, BANNER_OP_MAX)

    def _set_banner(self, kind, value):
        (set_banner_size if kind == "size" else set_banner_opacity)(value)

    def _on_click_banner(self, p):
        if self.banner_toggle.collidepoint(p):
            set_banner_on(not get_banner()["on"])
            self._play("select")
            return True
        cfg = get_banner()
        for kind, sl in self.banner_slider.items():
            lo, hi = self._banner_range(kind)
            if sl["track"].collidepoint(p):
                self._set_banner(kind, self._range_from_x(sl["track"], p[0], lo, hi))
                self._play("click")
                return True
            if sl["minus"].collidepoint(p):
                self._set_banner(kind, cfg[kind] - 0.1)
                self._play("click")
                return True
            if sl["plus"].collidepoint(p):
                self._set_banner(kind, cfg[kind] + 0.1)
                self._play("click")
                return True
        return False

    # ----- Eingabe ------------------------------------------------------
    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Escape", "Return", "space"):
                self._close()
            elif event.key == "Tab":
                self.tab = "grid" if self.tab == "head" else "head"
                self._play("click")
            elif event.key in tuple("1234567"):
                idx = int(event.key) - 1
                if self.tab == "head" and idx < len(HEAD_PRESETS):
                    self._select_head(HEAD_PRESETS[idx]["id"])
                elif self.tab == "grid" and idx < len(GRID_PRESETS):
                    self._select_grid(GRID_PRESETS[idx]["id"])
            return
        if event.kind == InputEvent.MOUSEDOWN and event.pos:
            self._on_click(event.pos)

    def _on_click(self, p):
        for name, r in self.tab_rects.items():
            if r.collidepoint(p):
                self.tab = name
                self._play("click")
                return
        if self.tab == "head":
            for rect, pr in self.head_tiles:
                if rect.collidepoint(p):
                    self._select_head(pr["id"])
                    return
        elif self.tab == "grid":
            for rect, pr in self.grid_tiles:
                if rect.collidepoint(p):
                    self._select_grid(pr["id"])
                    return
            if self._grid_sel == "custom":
                for k, r in self.grid_ab.items():
                    if r.collidepoint(p):
                        self._grid_edit = 0 if k == "A" else 1
                        self._play("click")
                        return
        elif self.tab == "banner":
            if self._on_click_banner(p):
                return
        ed = self._editable()
        if ed is not None:
            for ch, sl in self._active_sliders().items():
                if sl["track"].collidepoint(p):
                    self._slider_set(ed, ch, self._value_from_x(sl["track"], p[0]))
                    self._play("click")
                    return
                if sl["minus"].collidepoint(p):
                    self._step(ed, ch, -8)
                    return
                if sl["plus"].collidepoint(p):
                    self._step(ed, ch, +8)
                    return
        if self.done_rect.collidepoint(p):
            self._close()

    def update(self, dt):
        pass                                     # rein statisches Menü

    # ----- Zeichnen -----------------------------------------------------
    def draw(self, s):
        s.fill(COL_BG)
        cx = self.width // 2
        title = self._big.render(i18n.t("ngb.title"), True, COL_TEXT)
        s.blit(title, title.get_rect(midtop=(cx, 12)))
        self._draw_head_preview(s, self.width - 44, 30, self._head_color())
        self._draw_tabs(s)

        if self.tab == "head":
            self._draw_head_page(s)
        elif self.tab == "grid":
            self._draw_grid_page(s)
        else:
            self._draw_banner_page(s)

        pygame.draw.rect(s, COL_BTN_ON, self.done_rect, border_radius=10)
        dt = self._font.render(i18n.t("ngb.done"), True, COL_TEXT)
        s.blit(dt, dt.get_rect(center=self.done_rect.center))
        sub = self._tiny.render(i18n.t("ngb.subtitle"), True, COL_DIM)
        s.blit(sub, sub.get_rect(midbottom=(cx, self.height - 34)))
        foot = self._tiny.render(i18n.t("ngb.hint"), True, COL_DIM)
        s.blit(foot, foot.get_rect(midbottom=(cx, self.height - 16)))

    def _draw_tabs(self, s):
        labels = {"head": i18n.t("ngb.tab_head"), "grid": i18n.t("ngb.tab_grid"),
                  "banner": i18n.t("ngb.tab_banner")}
        for name, r in self.tab_rects.items():
            active = name == self.tab
            pygame.draw.rect(s, COL_ACCENT if active else COL_TAB_OFF, r,
                             border_radius=8)
            pygame.draw.rect(s, COL_SEL if active else COL_DIM, r, 1, border_radius=8)
            lab = self._small.render(labels[name], True,
                                     COL_BG if active else COL_DIM)
            s.blit(lab, lab.get_rect(center=r.center))

    # ----- Kopf-Seite ---------------------------------------------------
    def _draw_head_page(self, s):
        label = self._small.render(i18n.t("ngb.head_label"), True, COL_ACCENT)
        s.blit(label, (self.head_tiles[0][0].left, self.head_tiles[0][0].top - 22))
        for rect, pr in self.head_tiles:
            col = tuple(self._custom) if pr["id"] == "custom" else pr["color"]
            self._draw_tile(s, rect, col, pr["id"] == self._sel,
                            i18n.t("ngb.head." + pr["id"]))
        if self._sel == "custom":
            self._draw_sliders(s)
        else:
            hint = self._small.render(i18n.t("ngb.custom_pick"), True, COL_DIM)
            s.blit(hint, hint.get_rect(midtop=(self.width // 2,
                                               self.head_tiles[-1][0].bottom + 30)))

    # ----- Raster-Seite -------------------------------------------------
    def _draw_grid_page(self, s):
        label = self._small.render(i18n.t("ngb.grid_label"), True, COL_ACCENT)
        s.blit(label, (self.grid_tiles[0][0].left, self.grid_tiles[0][0].top - 22))
        for rect, pr in self.grid_tiles:
            seq = self._grid_custom if pr["id"] == "custom" else pr["seq"]
            selected = pr["id"] == self._grid_sel
            self._draw_grid_swatch(s, rect, seq)
            if selected:
                pygame.draw.rect(s, COL_SEL, rect.inflate(8, 8), 2, border_radius=10)
            name = self._tiny.render(i18n.t("ngb.grid." + pr["id"]), True,
                                     COL_TEXT if selected else COL_DIM)
            s.blit(name, name.get_rect(midtop=(rect.centerx, rect.bottom + 4)))

        if self._grid_sel == "custom":
            # A/B-Auswahl (zeigt die beiden eigenen Farben)
            for k, r in self.grid_ab.items():
                idx = 0 if k == "A" else 1
                pygame.draw.rect(s, tuple(self._grid_custom[idx]), r, border_radius=7)
                if self._grid_edit == idx:
                    pygame.draw.rect(s, COL_SEL, r, 2, border_radius=7)
                else:
                    pygame.draw.rect(s, COL_DIM, r, 1, border_radius=7)
                lab = self._small.render(k, True, COL_TEXT)
                s.blit(lab, lab.get_rect(center=r.center))
            self._draw_sliders(s)
        else:
            hint = self._small.render(i18n.t("ngb.grid_pick"), True, COL_DIM)
            s.blit(hint, hint.get_rect(midtop=(self.width // 2,
                                               self.grid_tiles[0][0].bottom + 34)))

    # ----- Banner-Seite -------------------------------------------------
    def _draw_banner_page(self, s):
        cx = self.width // 2
        cfg = get_banner()
        on = cfg["on"]

        label = self._small.render(i18n.t("ngb.banner_label"), True, COL_ACCENT)
        s.blit(label, (self.banner_toggle.left, self.banner_toggle.top - 22))

        # An/Aus-Schalter
        pygame.draw.rect(s, COL_BTN_ON if on else COL_BTN, self.banner_toggle,
                         border_radius=8)
        pygame.draw.rect(s, COL_SEL if on else COL_DIM, self.banner_toggle, 1,
                         border_radius=8)
        tl = self._font.render(i18n.t("ngb.banner_toggle"), True, COL_TEXT)
        s.blit(tl, (self.banner_toggle.x + 16,
                    self.banner_toggle.centery - tl.get_height() // 2))
        stat = i18n.t("common.on" if on else "common.off")
        col = (150, 235, 150) if on else COL_DIM
        stimg = self._font.render(f"< {stat} >", True, col)
        s.blit(stimg, (self.banner_toggle.right - stimg.get_width() - 16,
                       self.banner_toggle.centery - stimg.get_height() // 2))

        self._draw_banner_slider(s, "size", cfg["size"], on)
        self._draw_banner_slider(s, "opacity", cfg["opacity"], on)

        if on:
            self._draw_banner_preview(s, cx, 344, cfg)
        else:
            hint = self._small.render(i18n.t("ngb.banner_off_hint"), True, COL_DIM)
            s.blit(hint, hint.get_rect(center=(cx, 330)))

    def _draw_banner_slider(self, s, kind, val, enabled):
        sl = self.banner_slider[kind]
        track = sl["track"]
        lo, hi = self._banner_range(kind)
        frac = (val - lo) / (hi - lo)
        base = COL_ACCENT if kind == "size" else (150, 200, 120)
        col = base if enabled else COL_DIM
        lab = self._small.render(i18n.t("ngb.banner_" + kind), True, col)
        s.blit(lab, lab.get_rect(midright=(sl["minus"].left - 8, track.centery)))
        pygame.draw.rect(s, COL_TRACK, track, border_radius=7)
        pygame.draw.rect(s, col, (track.left, track.top, int(track.width * frac),
                                  track.height), border_radius=7)
        knob = track.left + int(track.width * frac)
        pygame.draw.circle(s, COL_SEL if enabled else COL_DIM, (knob, track.centery), 7)
        pygame.draw.circle(s, col, (knob, track.centery), 5)
        for r, sym in ((sl["minus"], "-"), (sl["plus"], "+")):
            pygame.draw.rect(s, COL_BTN, r, border_radius=5)
            g = self._font.render(sym, True, COL_TEXT if enabled else COL_DIM)
            s.blit(g, g.get_rect(center=r.center))
        pct = self._small.render(f"{int(round(val * 100))}%", True,
                                 COL_TEXT if enabled else COL_DIM)
        s.blit(pct, pct.get_rect(midleft=(sl["plus"].right + 8, track.centery)))

    def _draw_banner_preview(self, s, cx, cy, cfg):
        """Live-Vorschau des Banners mit aktueller Größe/Deckkraft."""
        big = self._big.render("×1.4", True, (150, 235, 150))
        sub = self._tiny.render(i18n.t("snake.purple_banner"), True, COL_DIM)
        pad = 14
        tw = max(big.get_width(), sub.get_width())
        th = big.get_height() + sub.get_height() + 4
        pw, ph = tw + pad * 2, th + pad
        panel = pygame.Surface((pw, ph), pygame.SRCALPHA)
        pygame.draw.rect(panel, (18, 20, 32, 215), panel.get_rect(), border_radius=12)
        pygame.draw.rect(panel, (150, 235, 150, 255), panel.get_rect(), 2,
                         border_radius=12)
        panel.blit(sub, sub.get_rect(midtop=(pw // 2, 5)))
        panel.blit(big, big.get_rect(midtop=(pw // 2, 5 + sub.get_height() + 2)))
        if abs(cfg["size"] - 1.0) > 0.01:
            panel = pygame.transform.rotozoom(panel, 0, cfg["size"])
        panel.fill((255, 255, 255, int(255 * cfg["opacity"])),
                   special_flags=pygame.BLEND_RGBA_MULT)
        s.blit(panel, panel.get_rect(center=(cx, cy)))

    # ----- gemeinsame Bausteine -----------------------------------------
    def _draw_tile(self, s, rect, col, selected, name):
        pygame.draw.rect(s, col, rect, border_radius=8)
        pygame.draw.rect(s, tuple(min(255, c + 50) for c in col), rect, 1,
                         border_radius=8)
        if selected:
            pygame.draw.rect(s, COL_SEL, rect.inflate(8, 8), 2, border_radius=10)
        lab = self._tiny.render(name, True, COL_TEXT if selected else COL_DIM)
        s.blit(lab, lab.get_rect(midtop=(rect.centerx, rect.bottom + 4)))

    def _draw_grid_swatch(self, s, rect, seq):
        """Mini-Vorschau einer Farbreihenfolge als Wegweiser (Reihen-Bänder + 1a)."""
        if not seq:
            pygame.draw.rect(s, (30, 32, 42), rect, border_radius=8)
            pygame.draw.rect(s, COL_DIM, rect, 1, border_radius=8)
            x = self._small.render("x", True, COL_DIM)
            s.blit(x, x.get_rect(center=rect.center))
            return
        n = len(seq)
        rows = 4
        rh = rect.height / rows
        for j in range(rows):
            col = tuple(seq[j % n])
            r = pygame.Rect(rect.left, int(rect.top + j * rh),
                            rect.width, int(rh) + 1)
            pygame.draw.rect(s, col, r)
        hint = self._tiny.render("1a", True, (232, 234, 240))
        s.blit(hint, hint.get_rect(center=rect.center))
        pygame.draw.rect(s, tuple(min(255, c + 40) for c in seq[0]), rect, 1,
                         border_radius=4)

    def _draw_sliders(self, s):
        ed = self._editable()
        if ed is None:
            return
        for ch, sl in self._active_sliders().items():
            track = sl["track"]
            val = ed[self.CHANNELS.index(ch)]
            pygame.draw.rect(s, COL_TRACK, track, border_radius=7)
            fill = pygame.Rect(track.left, track.top,
                               int(track.width * val / 255), track.height)
            pygame.draw.rect(s, self.CH_COL[ch], fill, border_radius=7)
            knob_x = track.left + int(track.width * val / 255)
            pygame.draw.circle(s, COL_SEL, (knob_x, track.centery), 7)
            pygame.draw.circle(s, self.CH_COL[ch], (knob_x, track.centery), 5)
            lab = self._small.render(ch, True, self.CH_COL[ch])
            s.blit(lab, lab.get_rect(midright=(sl["minus"].left - 6, track.centery)))
            for r, sym in ((sl["minus"], "-"), (sl["plus"], "+")):
                pygame.draw.rect(s, COL_BTN, r, border_radius=5)
                g = self._font.render(sym, True, COL_TEXT)
                s.blit(g, g.get_rect(center=r.center))
            num = self._small.render(str(val), True, COL_TEXT)
            s.blit(num, num.get_rect(midleft=(sl["plus"].right + 8, track.centery)))

    def _draw_head_preview(self, s, cx, cy, color):
        """Kleine Vorschau: ein Schlangenkopf in der aktuell gewählten Farbe."""
        r = pygame.Rect(0, 0, 30, 30)
        r.center = (cx, cy)
        pygame.draw.rect(s, color, r, border_radius=8)
        for sign in (-1, 1):
            ex = cx + sign * 6
            pygame.draw.circle(s, (250, 250, 250), (ex, cy - 2), 3)
            pygame.draw.circle(s, (20, 20, 30), (ex + 1, cy - 1), 1)
