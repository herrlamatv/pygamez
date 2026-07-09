# -*- coding: utf-8 -*-
"""
ngb.py
======
NGB - die **visuelle Personalisierung** der Spielesammlung (Skins/"Mods", die
ausschliesslich die Optik verändern, niemals die Spiellogik).

Hier laufen alle rein optischen Einstellungen zusammen:

- Die aktive **Kopffarbe** der Snake-Schlange (Vorlagen + eigene Farbe).
- Das kleine **Personalisierungs-Menü** (:class:`HeadColorMenu`), das über den
  Pinsel-Knopf im Snake-Setup geöffnet wird.

Gespeichert wird alles in **mem-ngb.json** (neben diesem Modul); beim ersten
Zugriff wird die Datei geladen, bei jeder Änderung zurückgeschrieben. Da NGB nur
Optik betrifft, hat es eine eigene Datei getrennt von Spielständen/Highscores.
"""

import json
import os

import pygame

import i18n
from game_base import InputEvent

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
_IDS = [p["id"] for p in HEAD_PRESETS]
DEFAULT_ID = "aqua3"                             # Standard: der Kopf ist türkis
DEFAULT_CUSTOM = (200, 120, 255)

# ----- Menü-Farben (lokal, um keine Spielmodule zu importieren) ---------
COL_BG = (14, 15, 24)
COL_TEXT = (232, 232, 238)
COL_DIM = (150, 158, 176)
COL_ACCENT = (90, 160, 240)
COL_BTN = (44, 50, 66)
COL_BTN_ON = (60, 120, 80)
COL_SEL = (255, 255, 255)
COL_TRACK = (36, 40, 54)

_state = None                                    # gecachte Daten aus mem-ngb.json


# ===================================================== Laden / Speichern
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
    hid = data.get("head_id", DEFAULT_ID)
    if hid not in _IDS:
        hid = DEFAULT_ID
    custom = data.get("head_custom", DEFAULT_CUSTOM)
    try:
        custom = tuple(max(0, min(255, int(c))) for c in custom)[:3]
        if len(custom) != 3:
            custom = DEFAULT_CUSTOM
    except (TypeError, ValueError):
        custom = DEFAULT_CUSTOM
    _state = {"head_id": hid, "head_custom": custom}
    return _state


def _save():
    st = _load()
    try:
        with open(_FILE, "w", encoding="utf-8") as f:
            json.dump({"head_id": st["head_id"],
                       "head_custom": list(st["head_custom"])},
                      f, ensure_ascii=False, indent=2)
    except OSError:
        pass


# ===================================================== Öffentliche API
def preset(pid):
    for p in HEAD_PRESETS:
        if p["id"] == pid:
            return p
    return HEAD_PRESETS[0]


def get_head_id():
    return _load()["head_id"]


def set_head_id(pid):
    if pid in _IDS:
        _load()["head_id"] = pid
        _save()


def get_custom():
    return tuple(_load()["head_custom"])


def set_custom(color):
    _load()["head_custom"] = tuple(max(0, min(255, int(c))) for c in color[:3])
    _save()


def head_color():
    """Aktive Kopffarbe der Schlange (Spieler 1) - rein visuell."""
    st = _load()
    if st["head_id"] == "custom":
        return tuple(st["head_custom"])
    return preset(st["head_id"])["color"]


def open_head_color_menu(width, height, play_sound=None):
    """Erzeugt das Personalisierungs-Menü (wird vom Snake-Setup aufgerufen)."""
    return HeadColorMenu(width, height, play_sound)


# ===================================================== Menü
class HeadColorMenu:
    """Personalisierungs-Menü: Kopffarben-Vorlage wählen oder eigene Farbe mischen.

    Verändert ausschliesslich die Optik. Änderungen werden sofort in
    mem-ngb.json gespeichert. ``done`` wird True, sobald das Menü geschlossen
    werden soll (der Aufrufer kehrt dann zu seinem Screen zurück).
    """

    CHANNELS = ("R", "G", "B")
    CH_COL = {"R": (235, 90, 90), "G": (90, 210, 120), "B": (90, 150, 245)}

    def __init__(self, width, height, play_sound=None):
        self.width = width
        self.height = height
        self.done = False
        self._play = play_sound or (lambda name: None)
        self._big = pygame.font.SysFont("consolas", 34, bold=True)
        self._font = pygame.font.SysFont("consolas", 20)
        self._small = pygame.font.SysFont("consolas", 15)
        self._tiny = pygame.font.SysFont("consolas", 12)
        self._sel = get_head_id()
        self._custom = list(get_custom())
        self._build_layout()

    # ----- Layout -------------------------------------------------------
    def _build_layout(self):
        cx = self.width // 2
        tw, th, gap = 92, 50, 12
        rows = (HEAD_PRESETS[:4], HEAD_PRESETS[4:])
        self.tiles = []                          # (rect, preset)
        y = 96
        for row in rows:
            total = len(row) * tw + (len(row) - 1) * gap
            x = cx - total // 2
            for p in row:
                self.tiles.append((pygame.Rect(x, y, tw, th), p))
                x += tw + gap
            y += th + 18 + gap                   # Platz für das Label unter der Kachel

        self._slider_top = y + 6
        self.sliders = {}                        # 'R'/'G'/'B' -> dict(track, minus, plus)
        sy = self._slider_top
        for ch in self.CHANNELS:
            track = pygame.Rect(cx - 100, sy, 200, 14)
            self.sliders[ch] = dict(
                track=track,
                minus=pygame.Rect(track.left - 30, sy - 4, 22, 22),
                plus=pygame.Rect(track.right + 8, sy - 4, 22, 22))
            sy += 30

        self.done_rect = pygame.Rect(cx - 80, self.height - 46, 160, 36)

    # ----- Zustand ------------------------------------------------------
    def _current_color(self):
        if self._sel == "custom":
            return tuple(self._custom)
        return preset(self._sel)["color"]

    def _select(self, pid):
        self._sel = pid
        set_head_id(pid)
        self._play("select")

    def _set_channel(self, ch, value):
        idx = self.CHANNELS.index(ch)
        self._custom[idx] = max(0, min(255, int(value)))
        set_custom(self._custom)

    def _step(self, ch, delta):
        idx = self.CHANNELS.index(ch)
        self._set_channel(ch, self._custom[idx] + delta)
        self._play("click")

    @staticmethod
    def _value_from_x(track, x):
        return round((x - track.left) / max(1, track.width) * 255)

    def _close(self):
        self.done = True
        self._play("click")

    # ----- Eingabe ------------------------------------------------------
    def handle_event(self, event):
        if event.kind == InputEvent.KEYDOWN:
            if event.key in ("Escape", "Return", "space"):
                self._close()
            elif event.key in tuple("1234567"):
                idx = int(event.key) - 1
                if idx < len(HEAD_PRESETS):
                    self._select(HEAD_PRESETS[idx]["id"])
            return
        if event.kind == InputEvent.MOUSEDOWN and event.pos:
            p = event.pos
            for rect, pr in self.tiles:
                if rect.collidepoint(p):
                    self._select(pr["id"])
                    return
            if self._sel == "custom":
                for ch, sl in self.sliders.items():
                    if sl["track"].collidepoint(p):
                        self._set_channel(ch, self._value_from_x(sl["track"], p[0]))
                        self._play("click")
                        return
                    if sl["minus"].collidepoint(p):
                        self._step(ch, -8)
                        return
                    if sl["plus"].collidepoint(p):
                        self._step(ch, +8)
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
        s.blit(title, title.get_rect(midtop=(cx, 14)))
        sub = self._small.render(i18n.t("ngb.subtitle"), True, COL_DIM)
        s.blit(sub, sub.get_rect(midtop=(cx, 52)))
        self._draw_head_preview(s, self.width - 46, 30, self._current_color())

        label = self._small.render(i18n.t("ngb.head_label"), True, COL_ACCENT)
        s.blit(label, (self.tiles[0][0].left, self.tiles[0][0].top - 22))

        for rect, pr in self.tiles:
            col = tuple(self._custom) if pr["id"] == "custom" else pr["color"]
            selected = pr["id"] == self._sel
            pygame.draw.rect(s, col, rect, border_radius=8)
            # heller Rand + Auswahl-Ring
            pygame.draw.rect(s, tuple(min(255, c + 50) for c in col), rect, 1,
                             border_radius=8)
            if selected:
                ring = rect.inflate(8, 8)
                pygame.draw.rect(s, COL_SEL, ring, 2, border_radius=10)
            name = self._tiny.render(i18n.t("ngb.head." + pr["id"]), True,
                                     COL_TEXT if selected else COL_DIM)
            s.blit(name, name.get_rect(midtop=(rect.centerx, rect.bottom + 4)))

        if self._sel == "custom":
            self._draw_sliders(s)
        else:
            hint = self._small.render(i18n.t("ngb.custom_pick"), True, COL_DIM)
            s.blit(hint, hint.get_rect(midtop=(cx, self._slider_top + 6)))

        pygame.draw.rect(s, COL_BTN_ON, self.done_rect, border_radius=10)
        dt = self._font.render(i18n.t("ngb.done"), True, COL_TEXT)
        s.blit(dt, dt.get_rect(center=self.done_rect.center))
        foot = self._tiny.render(i18n.t("ngb.hint"), True, COL_DIM)
        s.blit(foot, foot.get_rect(midbottom=(cx, self.height - 2)))

    def _draw_sliders(self, s):
        for ch, sl in self.sliders.items():
            track = sl["track"]
            idx = self.CHANNELS.index(ch)
            val = self._custom[idx]
            pygame.draw.rect(s, COL_TRACK, track, border_radius=7)
            fill = pygame.Rect(track.left, track.top,
                               int(track.width * val / 255), track.height)
            pygame.draw.rect(s, self.CH_COL[ch], fill, border_radius=7)
            knob_x = track.left + int(track.width * val / 255)
            pygame.draw.circle(s, COL_SEL, (knob_x, track.centery), 7)
            pygame.draw.circle(s, self.CH_COL[ch], (knob_x, track.centery), 5)
            lab = self._small.render(ch, True, self.CH_COL[ch])
            s.blit(lab, lab.get_rect(midright=(sl["minus"].left - 6, track.centery)))
            for rect, sym in ((sl["minus"], "-"), (sl["plus"], "+")):
                pygame.draw.rect(s, COL_BTN, rect, border_radius=5)
                g = self._font.render(sym, True, COL_TEXT)
                s.blit(g, g.get_rect(center=rect.center))
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
