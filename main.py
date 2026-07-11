# -*- coding: utf-8 -*-
"""
main.py
=======
Desktop-Spielesammlung: Tkinter (Oberfläche/Menü) + Pygame (Spiel-Rendering).

So funktioniert die Anzeige (Off-Screen-Rendering)
--------------------------------------------------
Pygame zeichnet NICHT in ein eigenes/eingebettetes Fenster. Der frühere Weg über
``SDL_WINDOWID`` (pygame direkt in ein Tkinter-Fenster zeichnen lassen) ist auf
Windows/SDL2 unzuverlässig: das SDL-Fenster "kämpft" mit Tkinters Layout, das
Fenster rastet/rüttelt beim Resize, und auf macOS/Wayland klappt es gar nicht.

Stattdessen läuft SDL mit dem Dummy-Video-Treiber (``SDL_VIDEODRIVER=dummy``) -
es gibt also KEIN echtes SDL-Fenster. Jedes Spiel zeichnet auf eine Surface
(``self.canvas``). Diese wird pro Frame seitenverhältnistreu skaliert, in ein
Bild (PPM) umgewandelt und in ein ``tk.Label`` (``self.embed``) gesetzt. Das
Label ist ein ganz normales Tkinter-Widget -> es skaliert/verhält sich sauber,
ohne Kampf mit einem nativen Fenster.

Damit sich Tkinter und Pygame nicht gegenseitig blockieren, gibt es KEINE eigene
while-Schleife für pygame. Stattdessen treibt Tkinters Ereignisschleife alles an:
root.after(....) ruft regelmäßig _loop() auf, das ein einzelnes Frame des Spiels
aktualisiert, zeichnet und ins Label bringt. Tastatur/Maus fangen wir über
Tkinter-Bindings ab und reichen sie als InputEvent an das aktive Spiel weiter.

Die Sidebar (linke Seite) ist komplett Canvas-basiert gezeichnet: abgerundete,
weich animierte Buttons (NeoButton), eine scrollbare Spieleliste mit eigenen
Mini-Piktogrammen und Highscores (GameList) sowie eine Status-Karte mit
Zustands-LED und Live-FPS. Alles bleibt reines Tkinter - keine Zusatzpakete.
"""

import math
import os
import sys
import tkinter as tk

# Das Verzeichnis dieser Datei sicher auf sys.path legen, damit die lokalen
# Module (i18n, settings, audio, games, ...) unabhängig vom Startverzeichnis
# gefunden werden - egal ob per Doppelklick, IDE oder "python main.py".
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import i18n
from i18n import t
import logo as logo_mod

# Standard-Spielfläche und -Bildrate. Die tatsächlichen Werte kommen aus den
# Einstellungen (settings.py) und liegen zur Laufzeit in self.game_w/-_h/-fps.
GAME_W = 640
GAME_H = 480
FPS = 60

# ---------------------------------------------------------------------------
#  Farbschema der Tkinter-Seite (abgestimmt auf ui.py der Pygame-Seite)
# ---------------------------------------------------------------------------
C_SIDEBAR = "#12151f"      # Sidebar-Hintergrund
C_HEADER = "#0c0f18"       # Kopfbereich (etwas dunkler)
C_CARD = "#1a2030"         # Karten/Panels
C_BTN = "#1f2636"          # Buttons/Zeilen normal
C_BTN_HOVER = "#2c3650"    # Buttons/Zeilen unter der Maus
C_ACCENT = "#589cff"       # Primär-Akzent (Blau)
C_ACCENT2 = "#9b6eff"      # Zweit-Akzent (Violett)
C_DANGER = "#8e3540"       # Beenden
C_DANGER_HOVER = "#ab414e"
C_TEXT = "#e9edf5"
C_TEXT_DIM = "#98a2b8"
C_TEXT_FAINT = "#5f6680"
C_BORDER = "#2a3147"
C_GREEN = "#6ecd8c"        # Status: läuft
C_GOLD = "#f5cd64"         # Status: Pause
C_RED = "#e15f5f"          # Status: Game Over


def _mix_hex(c1, c2, f):
    """Mischt zwei '#rrggbb'-Farben (f = Anteil von c2, 0..1)."""
    f = max(0.0, min(1.0, f))
    a = [int(c1.lstrip("#")[i:i + 2], 16) for i in (0, 2, 4)]
    b = [int(c2.lstrip("#")[i:i + 2], 16) for i in (0, 2, 4)]
    return "#%02x%02x%02x" % tuple(int(x + (y - x) * f) for x, y in zip(a, b))


def _round_rect(cv, x1, y1, x2, y2, r, **kw):
    """Abgerundetes Rechteck als glatt interpoliertes Canvas-Polygon."""
    pts = (x1 + r, y1, x2 - r, y1, x2, y1, x2, y1 + r, x2, y2 - r, x2, y2,
           x2 - r, y2, x1 + r, y2, x1, y2, x1, y2 - r, x1, y1 + r, x1, y1)
    return cv.create_polygon(pts, smooth=True, **kw)


class NeoButton(tk.Canvas):
    """Abgerundeter Sidebar-Button mit weichem Hover-Übergang + Symbol.

    Komplett selbst gezeichnet (Canvas), damit der Look zur Pygame-Seite
    passt: flach, abgerundet, sanfte Farb-Animation statt hartem Umschalten.
    """

    RADIUS = 9

    def __init__(self, parent, text, command, *, fill, hover, fg, bg,
                 icon=None, height=28, bold=False, center=False):
        super().__init__(parent, height=height, bg=bg, bd=0,
                         highlightthickness=0, cursor="hand2")
        self.text = text
        self.command = command
        self.icon = icon
        self.fill = fill
        self.hover = hover
        self.fg = fg
        self.center = center
        self._font = ("Segoe UI", 9, "bold") if bold else ("Segoe UI", 9)
        self._v = 0.0          # Hover-Fortschritt 0..1
        self._target = 0.0
        self._job = None
        self._pressed = False
        self.bind("<Configure>", lambda e: self._redraw())
        self.bind("<Enter>", lambda e: self._animate_to(1.0))
        self.bind("<Leave>", lambda e: self._on_leave())
        self.bind("<Button-1>", self._on_down)
        self.bind("<ButtonRelease-1>", self._on_up)

    def set_text(self, text):
        """Neue Beschriftung (Sprachwechsel)."""
        self.text = text
        self._redraw()

    # ----- Ereignisse ---------------------------------------------------

    def _on_leave(self):
        self._pressed = False
        self._animate_to(0.0)

    def _on_down(self, _e):
        self._pressed = True
        self._redraw()

    def _on_up(self, e):
        was = self._pressed
        self._pressed = False
        self._redraw()
        if was and 0 <= e.x < self.winfo_width() and 0 <= e.y < self.winfo_height():
            self.command()

    # ----- Animation/Zeichnen --------------------------------------------

    def _animate_to(self, target):
        self._target = target
        if self._job is None:
            self._job = self.after(16, self._step)

    def _step(self):
        self._job = None
        d = self._target - self._v
        if abs(d) < 0.05:
            self._v = self._target
        else:
            self._v = self._v + d * 0.3
            self._job = self.after(16, self._step)
        self._redraw()

    def _redraw(self):
        self.delete("all")
        w, h = self.winfo_width(), self.winfo_height()
        if w < 4 or h < 4:
            return
        col = _mix_hex(self.fill, self.hover, self._v)
        if self._pressed:
            col = _mix_hex(col, "#000000", 0.18)
        dy = 1 if self._pressed else 0
        _round_rect(self, 1, 1, w - 2, h - 2, self.RADIUS, fill=col,
                    outline=_mix_hex(C_BORDER, C_ACCENT, self._v * 0.7))
        fg = _mix_hex(self.fg, "#ffffff", self._v * 0.35)
        if self.center:
            import tkinter.font as tkfont
            f = tkfont.Font(family=self._font[0], size=self._font[1])
            tw = f.measure(self.text)
            x = w // 2 - tw // 2
            if self.icon:
                self.create_text(x - 8, h // 2 + dy, text=self.icon, anchor="e",
                                 fill=fg, font=("Segoe UI Symbol", 10))
            self.create_text(x, h // 2 + dy, text=self.text, anchor="w",
                             fill=fg, font=self._font)
        else:
            x = 12
            if self.icon:
                self.create_text(x, h // 2 + dy, text=self.icon, anchor="w",
                                 fill=_mix_hex(C_TEXT_DIM, C_ACCENT, self._v),
                                 font=("Segoe UI Symbol", 10))
                x += 22
            self.create_text(x, h // 2 + dy, text=self.text, anchor="w",
                             fill=fg, font=self._font)


def _draw_icon(cv, cx, cy, name, color, bg):
    """Zeichnet ein Mini-Piktogramm (ca. 20x20 px) für ein Spiel.

    Bewusst mit Canvas-Primitiven statt Emoji: sieht auf jedem System gleich
    aus und übernimmt die Akzentfarbe des Spiels.
    """
    if name == "SnakeGame":
        cv.create_line(cx - 8, cy + 6, cx - 8, cy - 1, cx + 1, cy - 1,
                       cx + 1, cy + 6, cx + 7, cy + 6, fill=color, width=3,
                       capstyle="round", joinstyle="round")
        cv.create_oval(cx + 5, cy - 8, cx + 9, cy - 4, fill=color, outline="")
    elif name == "PongGame":
        cv.create_rectangle(cx - 9, cy - 7, cx - 6, cy + 3, fill=color, outline="")
        cv.create_rectangle(cx + 6, cy - 3, cx + 9, cy + 7, fill=color, outline="")
        cv.create_oval(cx - 2, cy - 2, cx + 2, cy + 2, fill=color, outline="")
    elif name == "AirHockeyGame":
        cv.create_oval(cx - 9, cy - 9, cx + 1, cy + 1, outline=color, width=2)
        cv.create_oval(cx - 6, cy - 6, cx - 2, cy - 2, fill=color, outline="")
        cv.create_oval(cx + 3, cy + 3, cx + 9, cy + 9, fill=color, outline="")
    elif name == "TicTacToeGame":
        cv.create_line(cx - 3, cy - 9, cx - 3, cy + 9, fill=color, width=2)
        cv.create_line(cx + 3, cy - 9, cx + 3, cy + 9, fill=color, width=2)
        cv.create_line(cx - 9, cy - 3, cx + 9, cy - 3, fill=color, width=2)
        cv.create_line(cx - 9, cy + 3, cx + 9, cy + 3, fill=color, width=2)
    elif name == "BreakoutGame":
        for i in range(3):
            cv.create_rectangle(cx - 10 + i * 7, cy - 9, cx - 5 + i * 7, cy - 6,
                                fill=color, outline="")
        cv.create_oval(cx - 2, cy - 2, cx + 2, cy + 2, fill=color, outline="")
        cv.create_rectangle(cx - 6, cy + 6, cx + 6, cy + 9, fill=color, outline="")
    elif name == "TetrisGame":
        u = 6
        for gx, gy in ((-1, 0), (0, 0), (1, 0), (0, 1)):
            x0 = cx + gx * u - 2
            y0 = cy - 5 + gy * u
            cv.create_rectangle(x0 - 2, y0, x0 + 3, y0 + 5, fill=color, outline="")
    elif name == "InvadersGame":
        cv.create_rectangle(cx - 7, cy - 4, cx + 7, cy + 3, fill=color, outline="")
        cv.create_line(cx - 7, cy - 4, cx - 10, cy - 8, fill=color, width=2)
        cv.create_line(cx + 7, cy - 4, cx + 10, cy - 8, fill=color, width=2)
        cv.create_rectangle(cx - 4, cy - 2, cx - 2, cy, fill=bg, outline="")
        cv.create_rectangle(cx + 2, cy - 2, cx + 4, cy, fill=bg, outline="")
        cv.create_line(cx - 5, cy + 3, cx - 5, cy + 7, fill=color, width=2)
        cv.create_line(cx + 5, cy + 3, cx + 5, cy + 7, fill=color, width=2)
    elif name == "AsteroidsGame":
        cv.create_polygon(cx - 2, cy - 8, cx - 8, cy + 7, cx - 2, cy + 3,
                          cx + 4, cy + 7, fill="", outline=color, width=2)
        cv.create_oval(cx + 4, cy - 8, cx + 10, cy - 2, outline=color, width=2)
    elif name == "PacmanGame":
        cv.create_arc(cx - 8, cy - 8, cx + 8, cy + 8, start=35, extent=290,
                      fill=color, outline="")
        cv.create_oval(cx + 6, cy - 2, cx + 9, cy + 1, fill=color, outline="")
    elif name == "FlappyGame":
        cv.create_oval(cx - 8, cy - 6, cx + 4, cy + 6, fill=color, outline="")
        cv.create_polygon(cx + 3, cy - 2, cx + 9, cy, cx + 3, cy + 2,
                          fill=color, outline="")
        cv.create_oval(cx - 6, cy - 2, cx - 1, cy + 3, fill=bg, outline="")
    elif name == "DoodleGame":
        cv.create_line(cx - 9, cy + 8, cx - 1, cy + 8, fill=color, width=3,
                       capstyle="round")
        cv.create_line(cx + 1, cy, cx + 9, cy, fill=color, width=3,
                       capstyle="round")
        cv.create_line(cx - 9, cy - 8, cx - 1, cy - 8, fill=color, width=3,
                       capstyle="round")
        cv.create_oval(cx + 3, cy - 7, cx + 8, cy - 2, fill=color, outline="")
    elif name == "Game2048":
        _round_rect(cv, cx - 8, cy - 8, cx + 8, cy + 8, 4, fill="",
                    outline=color)
        cv.create_text(cx, cy, text="2", fill=color,
                       font=("Segoe UI", 9, "bold"))
    elif name == "MinesweeperGame":
        cv.create_line(cx + 3, cy - 8, cx + 3, cy + 7, fill=color, width=2)
        cv.create_polygon(cx + 3, cy - 8, cx - 6, cy - 4, cx + 3, cy,
                          fill=color, outline="")
        cv.create_line(cx - 2, cy + 7, cx + 8, cy + 7, fill=color, width=2)
    elif name == "SudokuGame":
        _round_rect(cv, cx - 9, cy - 9, cx + 9, cy + 9, 3, fill="",
                    outline=color)
        cv.create_line(cx - 3, cy - 9, cx - 3, cy + 9, fill=color)
        cv.create_line(cx + 3, cy - 9, cx + 3, cy + 9, fill=color)
        cv.create_line(cx - 9, cy - 3, cx + 9, cy - 3, fill=color)
        cv.create_line(cx - 9, cy + 3, cx + 9, cy + 3, fill=color)
        cv.create_text(cx - 6, cy - 6, text="5", fill=color,
                       font=("Segoe UI", 6, "bold"))
        cv.create_text(cx + 6, cy + 6, text="3", fill=color,
                       font=("Segoe UI", 6, "bold"))
    else:
        cv.create_text(cx, cy, text=(name[:1] or "?"), fill=color,
                       font=("Segoe UI", 11, "bold"))


class GameList:
    """Scrollbare Spieleliste auf einem Canvas.

    Jede Zeile: abgerundete Karte mit Akzentstreifen, Mini-Piktogramm,
    Spielname und Highscore. Hover wird weich animiert, das laufende Spiel
    bleibt markiert. Bei Platzmangel scrollt die Liste per Mausrad (rechts
    erscheint ein dezenter Scroll-Indikator).
    """

    ROW_H = 44
    GAP = 4

    def __init__(self, parent, app):
        self.app = app
        self.cv = tk.Canvas(parent, bg=C_SIDEBAR, bd=0, highlightthickness=0,
                            yscrollincrement=8)
        self.classes = []
        self.scores = {}
        self.rows = []           # je Zeile: Canvas-Item-IDs + Akzentfarbe
        self.active = None       # Klasse des laufenden/ausgewählten Spiels
        self._hover = None
        self._anim = {}          # Zeilen-Index -> Hover-Wert 0..1
        self._job = None
        self.cv.bind("<Configure>", lambda e: self._layout())
        self.cv.bind("<Motion>", self._on_motion)
        self.cv.bind("<Leave>", lambda e: self._set_hover(None))
        self.cv.bind("<Button-1>", self._on_click)
        self.cv.bind("<MouseWheel>", self._on_wheel)

    def pack(self, **kw):
        self.cv.pack(**kw)

    def build(self, classes):
        self.classes = list(classes)
        self.refresh_scores(relayout=False)
        self._layout()

    def refresh_scores(self, relayout=True):
        """Liest die Highscores neu ein (z.B. nach einer Partie)."""
        import highscore
        self.scores = highscore.load_highscores()
        if relayout:
            self._layout()

    def set_active(self, cls):
        """Markiert das laufende Spiel dauerhaft in der Liste."""
        self.active = cls
        for i in range(len(self.rows)):
            self._apply_row_colors(i)

    # ----- Aufbau ---------------------------------------------------------

    def _layout(self):
        import ui as ui_mod
        cv = self.cv
        cv.delete("all")
        self.rows = []
        if not self.classes:
            return
        w = max(120, cv.winfo_width())
        for i, cls in enumerate(self.classes):
            accent = ui_mod.GAME_COLORS.get(cls.__name__, C_ACCENT)
            y0 = i * (self.ROW_H + self.GAP)
            y1 = y0 + self.ROW_H
            cy = (y0 + y1) // 2
            bg = _round_rect(cv, 1, y0 + 1, w - 7, y1 - 1, 10,
                             fill=C_BTN, outline="")
            stripe = cv.create_rectangle(6, y0 + 14, 10, y1 - 14,
                                         fill=accent, width=0)
            chip_bg = _mix_hex(accent, C_SIDEBAR, 0.84)
            _round_rect(cv, 17, y0 + 8, 45, y1 - 8, 8, fill=chip_bg, outline="")
            _draw_icon(cv, 31, cy, cls.__name__, accent, chip_bg)
            hs = int(self.scores.get(cls.highscore_key, 0))
            if hs > 0:
                cv.create_text(54, cy - 9, text=cls.name, anchor="w",
                               fill=C_TEXT, font=("Segoe UI", 10, "bold"))
                cv.create_text(54, cy + 9, anchor="w", fill=C_TEXT_DIM,
                               text="★ " + f"{hs:,}".replace(",", " "),
                               font=("Segoe UI", 8))
            else:
                cv.create_text(54, cy, text=cls.name, anchor="w",
                               fill=C_TEXT, font=("Segoe UI", 10, "bold"))
            self.rows.append(dict(bg=bg, stripe=stripe, accent=accent,
                                  y0=y0, y1=y1))
            self._apply_row_colors(i)
        total = len(self.classes) * (self.ROW_H + self.GAP)
        cv.configure(scrollregion=(0, 0, w, total))
        self._draw_scrollbar()

    def _apply_row_colors(self, i):
        if i >= len(self.rows):
            return
        row = self.rows[i]
        v = self._anim.get(i, 0.0)
        active = (self.classes[i] is self.active)
        base = _mix_hex(C_BTN, row["accent"], 0.14) if active else C_BTN
        hover = _mix_hex(C_BTN_HOVER, row["accent"], 0.10)
        self.cv.itemconfig(row["bg"], fill=_mix_hex(base, hover, v),
                           outline=row["accent"] if active else "")
        # Akzentstreifen wächst bei Hover/Aktiv ein Stück.
        m = 8 if active else int(14 - 6 * v)
        self.cv.coords(row["stripe"], 6, row["y0"] + m, 10, row["y1"] - m)

    # ----- Interaktion ------------------------------------------------------

    def _index_at(self, ey):
        y = self.cv.canvasy(ey)
        i = int(y // (self.ROW_H + self.GAP))
        if 0 <= i < len(self.classes) and (y % (self.ROW_H + self.GAP)) <= self.ROW_H:
            return i
        return None

    def _on_motion(self, e):
        self._set_hover(self._index_at(e.y))

    def _set_hover(self, i):
        if i == self._hover:
            return
        self._hover = i
        self.cv.configure(cursor="hand2" if i is not None else "")
        if self._job is None:
            self._job = self.cv.after(16, self._animate)

    def _animate(self):
        self._job = None
        busy = False
        for i in range(len(self.rows)):
            target = 1.0 if i == self._hover else 0.0
            v = self._anim.get(i, 0.0)
            if abs(v - target) < 0.05:
                nv = target
            else:
                nv = v + (target - v) * 0.3
                busy = True
            if nv != v:
                self._anim[i] = nv
                self._apply_row_colors(i)
        if busy:
            self._job = self.cv.after(16, self._animate)

    def _on_click(self, e):
        i = self._index_at(e.y)
        if i is None:
            return
        self.app._sound_click()
        self._anim[i] = 1.0
        self._apply_row_colors(i)
        self.app.spiel_starten(self.classes[i])

    def _on_wheel(self, e):
        total = len(self.classes) * (self.ROW_H + self.GAP)
        if total <= self.cv.winfo_height():
            return
        self.cv.yview_scroll(int(-e.delta / 120) * 4, "units")
        self._draw_scrollbar()

    def _draw_scrollbar(self):
        """Dezenter Scroll-Indikator rechts (nur wenn die Liste scrollt)."""
        cv = self.cv
        cv.delete("sbar")
        view_h = cv.winfo_height()
        total = len(self.classes) * (self.ROW_H + self.GAP)
        if total <= view_h or view_h <= 1:
            return
        f0, f1 = cv.yview()
        x = cv.winfo_width() - 4
        top = cv.canvasy(0)
        cv.create_rectangle(x, top + f0 * view_h, x + 3, top + f1 * view_h,
                            fill="#3a4460", width=0, tags="sbar")


def _enable_dpi_awareness():
    """Macht den Prozess unter Windows DPI-aware (vor dem ersten Tk-Fenster!).

    Grund: Beim Einbetten zeichnet SDL/Pygame direkt in das native Fenster des
    Tkinter-Frames. Ist der Prozess NICHT DPI-aware, rechnet Windows Tkinter in
    logischen Pixeln, während SDL physische Pixel meint. Auf Displays mit
    Skalierung (125/150/200 %) passen die Größen dann nicht zusammen -> beim
    Resize verstellt set_mode() den Frame, Tkinter zieht zurück -> Rüttel-
    Schleife. Mit gleicher Pixel-Basis ist set_mode(Frame-Größe) ein No-Op.
    """
    if not sys.platform.startswith("win"):
        return
    try:
        import ctypes
        try:
            # Per-Monitor-DPI-aware (Windows 8.1+); 2 = PER_MONITOR_AWARE.
            ctypes.windll.shcore.SetProcessDpiAwareness(2)
        except Exception:
            # Fallback für ältere Windows-Versionen.
            ctypes.windll.user32.SetProcessDPIAware()
    except Exception:
        pass


class App:
    """Die Tkinter-Anwendung mit eingebettetem Pygame-Display."""

    # Farbschema als Klassen-Attribute verfügbar halten (siehe Konstanten oben).
    C_SIDEBAR = C_SIDEBAR
    C_HEADER = C_HEADER
    C_CARD = C_CARD
    C_BTN = C_BTN
    C_BTN_HOVER = C_BTN_HOVER
    C_ACCENT = C_ACCENT
    C_DANGER = C_DANGER
    C_DANGER_HOVER = C_DANGER_HOVER
    C_TEXT = C_TEXT
    C_TEXT_DIM = C_TEXT_DIM

    def __init__(self):
        # Sprache laden (aus mem.json), bevor irgendein Text aufgebaut wird.
        i18n.init()

        # MUSS vor dem ersten Tk-Fenster passieren (sonst wirkungslos).
        _enable_dpi_awareness()

        self.root = tk.Tk()
        self.root.title(t("app.title"))
        # Fenster darf vergrößert/verkleinert werden -> nötig für Vollbild,
        # bei dem das Pygame-Display weiter eingebettet "im Fenster" bleibt.
        self.root.resizable(True, True)
        self.root.minsize(840, 480)   # wird nach dem UI-Aufbau nachjustiert
        self.root.protocol("WM_DELETE_WINDOW", self.beenden)

        self._closing = False
        self._fullscreen = False
        self.current = None          # aktuell laufendes Spiel (Game-Objekt)
        # Skalierung/Versatz für die Darstellung der logischen Fläche
        self._scale = 1.0
        self._off = (0, 0)
        # Referenz auf das aktuell angezeigte Bild (sonst raeumt Tk es weg).
        self._photo = None
        # Live-FPS-Messung für die Status-Karte.
        self._fps_n = 0
        self._fps_t0 = 0

        # Einstellungen VOR pygame laden: die Auflösung bestimmt die Canvas-Größe,
        # die FPS die Loop-Frequenz.
        import settings
        self.settings = settings.load_settings()
        res = self.settings.get("resolution", [GAME_W, GAME_H])
        self.game_w, self.game_h = int(res[0]), int(res[1])
        self.fps = int(self.settings.get("fps", FPS))

        self._build_ui()

        # WICHTIG: erst das Fenster real erzeugen lassen, damit winfo_id() gültig ist.
        self.root.update_idletasks()
        self.root.update()

        self._init_pygame()
        self._set_window_icon()
        self._bind_events()

        # Sound-Engine starten (nach pygame.init()).
        import audio
        audio.init()

        # Pygame importieren wir erst nach _init_pygame (dort gesetzt), Spiele danach.
        from games import ALL_GAMES
        self._game_classes = ALL_GAMES
        self._build_game_buttons()

        # Sidebar-Breite/Fenster-Mindesthöhe an den tatsächlichen Bedarf koppeln.
        self._fit_sidebar()

        # Auto-Modus: die logische Auflösung schon beim Start an die tatsächliche
        # Fenster-/Frame-Größe anpassen.
        if self.settings.get("auto_resolution"):
            self.root.update_idletasks()
            self.disp_w = max(1, self.embed.winfo_width())
            self.disp_h = max(1, self.embed.winfo_height())
            self._match_resolution_to_window()

        # Beim ersten Start (noch keine Sprache in mem.json): Sprache wählen lassen.
        if not i18n.has_language():
            self.open_language()

        # Game-Loop über die Tkinter-Schleife starten
        self.root.after(0, self._loop)

    # ----- Oberfläche --------------------------------------------------

    def _sound_click(self):
        """Klick-Sound für die Tkinter-Seite (respektiert die Einstellungen)."""
        try:
            import audio
            audio.play("click", self.settings)
        except Exception:
            pass

    def _with_click(self, cmd):
        """Umhüllt einen Button-Befehl mit dem Klick-Sound."""
        def run():
            self._sound_click()
            cmd()
        return run

    def _load_header_logo(self):
        """Kleines Logo für den Sidebar-Kopf (kleinstes verfügbares PNG)."""
        for size in sorted(logo_mod.LOGO_SIZES):
            pfad, ext = logo_mod.find_logo(size)
            if pfad and ext == "png":
                try:
                    img = tk.PhotoImage(file=pfad)
                    k = max(1, round(img.width() / 30))
                    return img.subsample(k, k)
                except tk.TclError:
                    return None
        return None

    def _build_header_gradient(self, parent):
        """Akzentlinie unter dem Kopf: Verlauf Blau -> Violett."""
        grad = tk.Canvas(parent, height=2, bg=C_HEADER, bd=0,
                         highlightthickness=0)
        grad.pack(fill="x")

        def paint(_e=None):
            grad.delete("all")
            w = max(1, grad.winfo_width())
            n = 32
            for i in range(n):
                col = _mix_hex(C_ACCENT, C_ACCENT2, i / (n - 1))
                grad.create_rectangle(w * i / n, 0, w * (i + 1) / n + 1, 2,
                                      fill=col, width=0)

        grad.bind("<Configure>", paint)

    def _build_ui(self):
        # Linke Seite: Sidebar mit Kopf, Spieleliste, Aktionen und Status.
        menu = tk.Frame(self.root, width=232, bg=C_SIDEBAR)
        menu.pack(side="left", fill="y")
        menu.pack_propagate(False)
        self._menu_frame = menu

        # --- Kopfbereich: Logo + App-Name + BETA-Chip + Untertitel --------
        header = tk.Frame(menu, bg=C_HEADER)
        header.pack(fill="x")
        title_row = tk.Frame(header, bg=C_HEADER)
        title_row.pack(pady=(14, 2))
        self._header_logo = self._load_header_logo()
        if self._header_logo is not None:
            tk.Label(title_row, image=self._header_logo,
                     bg=C_HEADER).pack(side="left", padx=(0, 8))
        tk.Label(title_row, text="PyGameZ", fg=C_TEXT, bg=C_HEADER,
                 font=("Segoe UI", 16, "bold")).pack(side="left")
        tk.Label(title_row, text="BETA", fg=C_ACCENT2, bg=C_HEADER,
                 font=("Segoe UI", 7, "bold")).pack(side="left", anchor="n",
                                                    padx=(6, 0), pady=(3, 0))
        self._header_sub = tk.Label(header, text=t("app.menu_title"),
                                    fg=C_TEXT_DIM, bg=C_HEADER,
                                    font=("Segoe UI", 9))
        self._header_sub.pack(pady=(0, 10))
        self._build_header_gradient(menu)

        # --- Status-Karte unten ----------------------------------------------
        # (Der untere Block wird VOR der Spieleliste gepackt: bottom-Widgets
        #  bekommen so ihren Platz zuerst und bleiben auch bei kleiner
        #  Fensterhöhe sichtbar - die Spieleliste scrollt dann.)
        self.status_var = tk.StringVar(value=t("app.no_game"))
        self._perf_var = tk.StringVar(value="")
        status_card = tk.Frame(menu, bg=C_CARD, highlightbackground=C_BORDER,
                               highlightthickness=1)
        status_card.pack(side="bottom", fill="x", padx=12, pady=(4, 12))
        tk.Frame(status_card, bg=C_ACCENT, width=3).pack(side="left", fill="y")
        body = tk.Frame(status_card, bg=C_CARD)
        body.pack(side="left", fill="x", expand=True, padx=8, pady=6)
        top_row = tk.Frame(body, bg=C_CARD)
        top_row.pack(fill="x")
        # Zustands-LED: grau (Menü), grün (läuft), gold (Pause), rot (Game Over)
        self._dot = tk.Canvas(top_row, width=9, height=9, bg=C_CARD, bd=0,
                              highlightthickness=0)
        self._dot_id = self._dot.create_oval(1, 1, 8, 8, fill=C_TEXT_DIM, width=0)
        self._dot.pack(side="left", anchor="n", pady=3)
        self._status_label = tk.Label(top_row, textvariable=self.status_var,
                                      fg=C_TEXT_DIM, bg=C_CARD,
                                      font=("Consolas", 9), justify="left",
                                      anchor="w", wraplength=168)
        self._status_label.pack(side="left", padx=(6, 0), fill="x")
        tk.Label(body, textvariable=self._perf_var, fg=C_TEXT_FAINT,
                 bg=C_CARD, font=("Consolas", 8),
                 anchor="w").pack(fill="x", pady=(4, 0))

        # --- Steuer-Buttons unten (Referenzen für refresh_language) ---------
        self._ctrl_buttons = {}
        qbtn = NeoButton(menu, t("app.quit"), self._with_click(self.beenden),
                         icon="✕", fill=C_DANGER, hover=C_DANGER_HOVER,
                         fg=C_TEXT, bg=C_SIDEBAR, height=32, bold=True,
                         center=True)
        qbtn.pack(side="bottom", fill="x", padx=12, pady=(4, 8))
        self._ctrl_buttons["quit"] = qbtn

        for key, cmd, icon in (("back_to_menu", self.zum_menü, "⌂"),
                               ("fullscreen", self.toggle_fullscreen, "⛶"),
                               ("language", self.open_language, "🌐"),
                               ("options", self.open_options, "⚙")):
            btn = NeoButton(menu, t("app." + key), self._with_click(cmd),
                            icon=icon, fill=C_BTN, hover=C_BTN_HOVER,
                            fg=C_TEXT, bg=C_SIDEBAR, height=28)
            btn.pack(side="bottom", fill="x", padx=12, pady=2)
            self._ctrl_buttons[key] = btn

        tk.Frame(menu, bg=C_BORDER, height=1).pack(side="bottom", fill="x",
                                                   padx=12, pady=6)

        # --- Abschnitt: Spiele (füllt den verbleibenden Platz) --------------
        sec = tk.Frame(menu, bg=C_SIDEBAR)
        sec.pack(fill="x", padx=16, pady=(10, 4))
        self.games_label = tk.Label(sec, text=t("app.games"), fg=C_ACCENT,
                                    bg=C_SIDEBAR, anchor="w",
                                    font=("Segoe UI", 9, "bold"))
        self.games_label.pack(side="left")
        self._games_count = tk.Label(sec, text="", fg=C_TEXT_FAINT,
                                     bg=C_SIDEBAR, font=("Segoe UI", 8, "bold"))
        self._games_count.pack(side="right")

        self.game_list = GameList(menu, self)
        self.game_list.pack(fill="both", expand=True, padx=12)

        # Rechte Seite: Anzeige des Spielbildes. Ein normales tk.Label zeigt das
        # pro Frame erzeugte Bild (siehe _present). fill/expand -> es füllt den
        # verfügbaren Platz; wir bauen das Bild jeweils in dieser Größe.
        self.embed = tk.Label(self.root, bg="black", bd=0, highlightthickness=0)
        self.embed.pack(side="right", fill="both", expand=True)
        # width/height beim Label sind Zeichen/Pixel je nach Inhalt -> sobald ein
        # Bild gesetzt ist, zählt dessen Größe. fill/expand bestimmt die Fläche.
        # Fokus, damit Tastatur-Events (auf root gebunden) sicher ankommen.
        self.embed.configure(takefocus=True)
        self.embed.focus_set()

    def _fit_sidebar(self):
        """Passt die Sidebar-Breite an die Inhalte an.

        Nötig, weil Beschriftungen je nach Sprache/DPI unterschiedlich breit
        ausfallen; die Sidebar hat pack_propagate(False) und würde lange
        Texte sonst abschneiden. Die Spieleliste scrollt bei Platzmangel,
        deshalb braucht die Fensterhöhe kein hartes Minimum mehr.
        """
        import tkinter.font as tkfont
        self.root.update_idletasks()

        f_ctrl = tkfont.Font(family="Segoe UI", size=9)
        f_game = tkfont.Font(family="Segoe UI", size=10, weight="bold")
        w_ctrl = max((f_ctrl.measure(b.text)
                      for b in self._ctrl_buttons.values()), default=0) + 60
        w_game = max((f_game.measure(c.name) for c in self._game_classes),
                     default=0) + 80
        need_w = min(340, max(232, w_ctrl, w_game))
        self._menu_frame.config(width=need_w)
        self._status_label.config(wraplength=max(120, need_w - 70))
        self.root.minsize(880, 560)

        # Start-Fenstergröße einmalig so wählen, dass die komplette Spiele-
        # liste sichtbar ist (gedeckelt auf die Bildschirmgröße). Danach darf
        # der Nutzer frei verkleinern - die Liste scrollt dann per Mausrad.
        if not getattr(self, "_initial_size_done", False):
            self._initial_size_done = True
            rows_h = len(self._game_classes) * (GameList.ROW_H + GameList.GAP)
            need_h = 8
            for ch in self._menu_frame.winfo_children():
                if not ch.winfo_manager():
                    continue
                pady = ch.pack_info().get("pady", 0)
                pad = (sum(int(p) for p in pady)
                       if isinstance(pady, (tuple, list)) else int(pady) * 2)
                if ch is self.game_list.cv:
                    need_h += rows_h + pad
                else:
                    need_h += ch.winfo_reqheight() + pad
            h = max(560, min(need_h, self.root.winfo_screenheight() - 120))
            w = min(self.root.winfo_screenwidth() - 80,
                    need_w + max(640, int(h * self.game_w / max(1, self.game_h))))
            self.root.geometry(f"{w}x{h}")

    def _build_game_buttons(self):
        """Füllt die Canvas-Spieleliste mit allen Spielen + Highscores."""
        self.game_list.build(self._game_classes)
        self._games_count.config(text=str(len(self._game_classes)))

    # ----- Pygame (Off-Screen-Rendering) --------------------------------

    def _init_pygame(self):
        # KEIN echtes Fenster: Dummy-Video-Treiber. So gibt es kein natives
        # SDL-Fenster, das mit Tkinter um Größe/Position kämpft. Muss VOR
        # pygame.init()/set_mode() gesetzt sein.
        os.environ["SDL_VIDEODRIVER"] = "dummy"

        import pygame
        self.pygame = pygame
        pygame.init()

        # Aktuelle Größe der Anzeigefläche (des Labels). Kann durch Vollbild/
        # Fenstergröße wachsen; wird im Loop pro Frame frisch gelesen.
        self.disp_w = max(self.game_w, self.embed.winfo_width())
        self.disp_h = max(self.game_h, self.embed.winfo_height())
        # Ein (unsichtbarer) Video-Modus, damit Surface.convert() funktioniert.
        pygame.display.set_mode((1, 1))

        # Alle Spiele zeichnen auf diese LOGISCHE Fläche (game_w x game_h aus den
        # Einstellungen). Im Loop wird sie passend auf die Anzeige skaliert.
        self.canvas = pygame.Surface((self.game_w, self.game_h))

        self.clock = pygame.time.Clock()

        # Gemeinsames UI-Toolkit (Palette/Verläufe/Panels/Effekte) für alle Screens.
        import ui
        self.ui = ui

    def _set_window_icon(self):
        """Setzt das Fenster-/Taskleisten-Icon aus dem Logo (siehe logo.py).

        Welche Datei genau (Nummer/Format) genommen wird, entscheidet das
        Modul ``logo``: PNG wird direkt von Tk geladen, JPG ueber den alten
        PPM-Umweg umgewandelt. Fehlt jedes Logo, startet das Programm einfach
        mit dem Standard-Icon.
        """
        # Referenzen halten, sonst räumt Tk die Bilder weg.
        self._icons = logo_mod.icon_photos(self.pygame, tk)
        if self._icons:
            try:
                # True = gilt auch für alle künftigen Fenster (z.B. Dialoge).
                self.root.iconphoto(True, *self._icons)
            except tk.TclError:
                pass

    # ----- Eingaben von Tkinter zu Spielen weiterreichen ----------------

    def _bind_events(self):
        # Tastatur global am Hauptfenster abfangen
        self.root.bind("<KeyPress>", self._on_key)
        self.root.bind("<KeyRelease>", self._on_key_up)
        # Maus auf der Spielfläche
        self.embed.bind("<Button-1>", self._on_click)
        self.embed.bind("<Button-3>", self._on_right_click)
        self.embed.bind("<Motion>", self._on_motion)
        # Klick auf die Fläche holt den Fokus (für Tastatur)
        self.embed.bind("<Button-1>", lambda e: self.embed.focus_set(), add="+")
        # Kein <Configure>-Handler mehr nötig: die aktuelle Anzeigegröße wird
        # pro Frame in _sync_display_size() gelesen. Da es KEIN natives
        # SDL-Fenster gibt, kann nichts mit Tkinter um die Größe "kämpfen".

    def _sync_display_size(self):
        """Liest die aktuelle Label-Größe; passt bei Auto-Auflösung die Fläche an.

        Wird pro Frame aufgerufen. Ohne echtes SDL-Fenster ist das gefahrlos:
        wir zeichnen ja nur ein Bild in der jeweils aktuellen Größe.
        """
        w = max(1, self.embed.winfo_width())
        h = max(1, self.embed.winfo_height())
        if (w, h) == (self.disp_w, self.disp_h):
            return
        self.disp_w, self.disp_h = w, h
        # Auto-Modus: logische Auflösung an die neue Fenstergröße anpassen.
        if self.settings.get("auto_resolution"):
            self._match_resolution_to_window()

    def _to_logical(self, x, y):
        """Rechnet Fenster-/Mauskoordinaten in logische Spielkoordinaten um."""
        ox, oy = self._off
        s = self._scale or 1.0
        lx = max(0, min(self.game_w - 1, (x - ox) / s))
        ly = max(0, min(self.game_h - 1, (y - oy) / s))
        return (int(lx), int(ly))

    def _on_key(self, event):
        from game_base import InputEvent

        # F11 schaltet den Vollbildmodus um (Spiel bleibt eingebettet)
        if event.keysym == "F11":
            self.toggle_fullscreen()
            return

        # ESC: bei Menü-Screens als "Zurück" durchreichen, sonst Pause umschalten.
        if event.keysym == "Escape":
            if self.current and getattr(self.current, "is_menu", False):
                self.current.handle_event(InputEvent(InputEvent.KEYDOWN, key="Escape"))
            elif self.current and not self.current.game_over:
                self.current.paused = not self.current.paused
            return

        if self.current and not self.current.paused:
            self.current.handle_event(InputEvent(InputEvent.KEYDOWN, key=event.keysym))

    def _on_key_up(self, event):
        from game_base import InputEvent
        if self.current and not self.current.paused:
            self.current.handle_event(InputEvent(InputEvent.KEYUP, key=event.keysym))

    def _on_click(self, event):
        from game_base import InputEvent
        if self.current and not self.current.paused:
            self.current.handle_event(
                InputEvent(InputEvent.MOUSEDOWN, pos=self._to_logical(event.x, event.y)))

    def _on_right_click(self, event):
        # Rechtsklicks bekommen nur Spiele, die sie ausdrücklich wollen
        # (z.B. Minesweeper zum Flaggen) - alle anderen bleiben unberührt.
        from game_base import InputEvent
        if self.current and not self.current.paused \
                and getattr(self.current, "wants_right_click", False):
            self.current.handle_event(
                InputEvent(InputEvent.MOUSEDOWN,
                           pos=self._to_logical(event.x, event.y), button=3))

    def _on_motion(self, event):
        from game_base import InputEvent
        if self.current and not self.current.paused:
            self.current.handle_event(
                InputEvent(InputEvent.MOUSEMOVE, pos=self._to_logical(event.x, event.y)))

    def toggle_fullscreen(self):
        """Schaltet den Vollbildmodus des Tkinter-Fensters um."""
        self._fullscreen = not self._fullscreen
        self.root.attributes("-fullscreen", self._fullscreen)
        self.embed.focus_set()

    # ----- Spielsteuerung -----------------------------------------------

    def spiel_starten(self, game_cls):
        """Zeigt zuerst den Vorspiel-Screen (Modus/Optionen) für dieses Spiel."""
        from menu import PreGameScreen
        self.game_list.set_active(game_cls)
        self.show_screen(PreGameScreen(self.canvas, self.game_w, self.game_h,
                                       self, game_cls))

    def launch_game(self, game_cls, mode):
        """Startet das eigentliche Spiel im gewählten Modus mit den Einstellungen."""
        self.current = game_cls(self.canvas, self.game_w, self.game_h,
                                mode=mode, game_settings=self.settings)
        self.current.paused = False
        self.game_list.set_active(game_cls)
        self.ui.begin_transition()
        self.embed.focus_set()

    def show_screen(self, screen):
        """Macht einen Menü-Screen (Vorspiel/Optionen) zum aktiven 'current'."""
        self.current = screen
        self.ui.begin_transition()
        self.embed.focus_set()

    def back_to_menu(self):
        """Zurück zum leeren Startbildschirm (ohne Highscore-Effekte)."""
        self.current = None
        self.status_var.set(t("app.no_game"))
        self._set_state_dot(C_TEXT_DIM)
        self.game_list.set_active(None)
        self.game_list.refresh_scores()
        self.ui.begin_transition()

    def open_options(self):
        """Öffnet die Optionen/Steuerung im Spielbereich (aus dem Tk-Menü)."""
        from menu import OptionsScreen
        self.show_screen(OptionsScreen(self.canvas, self.game_w, self.game_h, self,
                                       on_close=self.back_to_menu))

    def open_language(self):
        """Zeigt den Sprachauswahl-Screen im Spielbereich."""
        from menu import LanguageScreen
        self.show_screen(LanguageScreen(self.canvas, self.game_w, self.game_h, self,
                                        on_done=self.back_to_menu))

    def refresh_language(self):
        """Beschriftet das Tkinter-Menü nach einem Sprachwechsel neu."""
        self.root.title(t("app.title"))
        self.games_label.config(text=t("app.games"))
        self._header_sub.config(text=t("app.menu_title"))
        for key, btn in self._ctrl_buttons.items():
            btn.set_text(t("app." + key))
        if self.current is None:
            self.status_var.set(t("app.no_game"))
        # Der Highscore-Ticker enthält übersetzten Text -> neu aufbauen.
        self._ticker_key = None
        # Neue Beschriftungen können breiter/schmaler sein -> Sidebar anpassen.
        self._fit_sidebar()

    def apply_resolution(self, w, h, persist=True):
        """Setzt die logische Auflösung neu (Canvas wird passend neu erzeugt).

        Der aktuell aktive Screen wird auf die neue Fläche umgehängt, damit die
        Änderung sofort sichtbar ist (praktisch im Options-Screen).

        persist=False bei der automatischen Anpassung an die Fenstergröße -> die
        vom Nutzer gewählte feste Auflösung bleibt gespeichert.
        """
        self.game_w, self.game_h = max(1, int(w)), max(1, int(h))
        if persist:
            self.settings["resolution"] = [self.game_w, self.game_h]
        self.canvas = self.pygame.Surface((self.game_w, self.game_h))
        if self.current is not None:
            self.current.surface = self.canvas
            self.current.width = self.game_w
            self.current.height = self.game_h
            # Menü-Screens berechnen ihr Layout aus width/height -> neu aufbauen.
            if hasattr(self.current, "on_surface_changed"):
                self.current.on_surface_changed()

    def _match_resolution_to_window(self):
        """Setzt die logische Auflösung gleich der aktuellen Fenster-/Frame-Größe."""
        if (self.game_w, self.game_h) != (self.disp_w, self.disp_h):
            self.apply_resolution(self.disp_w, self.disp_h, persist=False)

    def set_auto_resolution(self, on):
        """Schaltet die automatische Anpassung an die Fenstergröße um."""
        self.settings["auto_resolution"] = bool(on)
        if on:
            # Sofort an die aktuelle Fenstergröße anpassen.
            self._match_resolution_to_window()
        else:
            # Zurück auf die gespeicherte feste Auflösung.
            res = self.settings.get("resolution", [GAME_W, GAME_H])
            self.apply_resolution(int(res[0]), int(res[1]))

    def apply_fps(self, fps):
        """Setzt die Ziel-Bildrate neu (wirkt ab dem nächsten Frame)."""
        self.fps = max(5, min(240, int(fps)))
        self.settings["fps"] = self.fps

    def zum_menü(self):
        """Beendet das aktuelle Spiel und kehrt zum Startbildschirm zurück."""
        if self.current:
            self._highscore_speichern(self.current)
        self.current = None
        self.status_var.set(t("app.no_game"))
        self._set_state_dot(C_TEXT_DIM)
        self.game_list.set_active(None)
        self.game_list.refresh_scores()
        self.ui.begin_transition()

    # ----- Highscores ---------------------------------------------------

    def _highscore_speichern(self, game):
        # Menü-Screens sind keine Spiele -> kein Highscore.
        if game is None or getattr(game, "is_menu", False):
            return
        import highscore
        hs, rekord = highscore.update_highscore(game.highscore_key, game.score)
        # Für die Game-Over-Einblendung merken.
        game._hs_value = hs
        game._hs_record = rekord

    def _draw_highscore_overlay(self, game):
        """Blendet bei Game Over für JEDES Spiel den Highscore als Banner ein."""
        ui = self.ui
        hs = getattr(game, "_hs_value", 0)
        record = getattr(game, "_hs_record", False)
        fnt = ui.font(18, bold=True)
        if record:
            # Rekord: goldener Verlaufstext
            img = ui.grad_text(fnt, t("app.new_highscore", score=game.score),
                               top=(255, 235, 170), bottom=(235, 175, 70))
        else:
            img = fnt.render(t("app.highscore", hs=hs), True, ui.TEXT_DIM)
        bw, bh = img.get_width() + 36, img.get_height() + 14
        banner = self.pygame.Rect(self.game_w // 2 - bw // 2,
                                  self.game_h - bh - 10, bw, bh)
        ui.draw_panel(self.canvas, banner, radius=bh // 2, shadow=False)
        if record:
            # Rekord: goldener, pulsierender Rahmen
            glow = self.pygame.Surface((bw + 8, bh + 8), self.pygame.SRCALPHA)
            self.pygame.draw.rect(glow, (*ui.GOLD, int(90 * ui.pulse(3.0))),
                                  (0, 0, bw + 8, bh + 8),
                                  border_radius=bh // 2 + 4)
            self.canvas.blit(glow, (banner.x - 4, banner.y - 4))
            self.pygame.draw.rect(self.canvas, ui.GOLD, banner, width=1,
                                  border_radius=bh // 2)
        self.canvas.blit(img, img.get_rect(center=banner.center))

    # ----- Zentrale Game-Loop -------------------------------------------

    def _loop(self):
        if self._closing:
            return

        pygame = self.pygame
        # Nur die vergangene Zeit MESSEN (kein Blockieren): das Timing/Pacing
        # macht bereits root.after(...) am Ende der Schleife. Ein blockierendes
        # clock.tick(fps) würde die Tkinter-Schleife jeden Frame anhalten und die
        # Oberfläche zäh/ruckelig machen. dt wird gedeckelt, damit die Spiele nach
        # einem kurzen Hänger nicht "springen".
        dt = min(self.clock.tick() / 1000.0, 0.1)   # vergangene Zeit in Sekunden

        # pygame-interne Ereignisse leeren (hält SDL "lebendig")
        pygame.event.pump()

        # Live-FPS für die Status-Karte (alle 500 ms aktualisiert).
        self._fps_n += 1
        now_ms = pygame.time.get_ticks()
        if now_ms - self._fps_t0 >= 500:
            if self._fps_t0:
                fps = self._fps_n * 1000.0 / (now_ms - self._fps_t0)
                self._perf_var.set(f"{self.game_w}x{self.game_h}"
                                   f"   ·   {fps:.0f}/{self.fps} FPS")
            self._fps_t0 = now_ms
            self._fps_n = 0

        # Aktuelle Anzeigegröße übernehmen (VOR dem Zeichnen -> Auto-Auflösung
        # baut die Canvas ggf. neu, bevor das Spiel darauf zeichnet).
        self._sync_display_size()

        if self.current is None:
            self._draw_menu_screen()
        else:
            game = self.current
            if not game.paused and not game.game_over:
                game.update(dt)
            game.draw()

            if game.paused:
                self._draw_pause_overlay()

            self._update_status(game)

            # Highscore beim Übergang zu Game Over genau einmal sichern
            if game.game_over and not getattr(game, "_hs_saved", False):
                self._highscore_speichern(game)
                game._hs_saved = True
                # Neuer Rekord -> Konfetti-Regen über dem Game-Over-Bild.
                if getattr(game, "_hs_record", False):
                    self.ui.spawn_confetti(self.game_w, self.game_h)
            if not game.game_over:
                game._hs_saved = False

            # Highscore bei Game Over für jedes Spiel einblenden.
            if game.game_over and not getattr(game, "is_menu", False):
                self._draw_highscore_overlay(game)

        # Globale Effekte (Partikel/Konfetti + Screen-Übergang) obendrauf.
        self.ui.draw_fx(self.canvas, self.game_w, self.game_h, dt)

        # Logische Fläche skaliert (mit Letterbox) auf das echte Display bringen
        self._present()

        # nächstes Frame über Tkinter planen -> Tkinter bleibt reaktiv
        self.root.after(max(1, int(1000 / self.fps)), self._loop)

    def _present(self):
        """Baut das Anzeigebild (Letterbox) und zeigt es im Label."""
        pygame = self.pygame
        sw, sh = self.disp_w, self.disp_h
        scale = min(sw / self.game_w, sh / self.game_h)
        tw, th = max(1, int(self.game_w * scale)), max(1, int(self.game_h * scale))
        self._scale = scale
        self._off = ((sw - tw) // 2, (sh - th) // 2)

        # Anzeigefläche mit schwarzen Letterbox-Rändern zusammensetzen.
        frame = pygame.Surface((sw, sh))
        frame.fill((0, 0, 0))
        if (tw, th) == (self.game_w, self.game_h):
            frame.blit(self.canvas, self._off)
        else:
            frame.blit(pygame.transform.scale(self.canvas, (tw, th)), self._off)

        # Surface -> PPM (P6) -> tk.PhotoImage. Referenz halten, sonst GC.
        data = b"P6 %d %d 255 " % (sw, sh) + pygame.image.tostring(frame, "RGB")
        self._photo = tk.PhotoImage(data=data, format="ppm")
        self.embed.configure(image=self._photo)

    def _menu_logo(self, size):
        """Lädt das Logo (siehe logo.py), skaliert es und rundet die Ecken.

        Das Ergebnis wird je Größe gecacht. Fehlt das Bild, gibt die Methode
        None zurück - der Startbildschirm zeigt dann den Schriftzug als Fallback.
        Für die Menü-Anzeige wird eine mittlere Größe bevorzugt (schöner beim
        Herunterskalieren).
        """
        cache = getattr(self, "_logo_cache", None)
        if cache and cache[0] == size:
            return cache[1]
        raw = logo_mod.load_surface(self.pygame, prefer_sizes=(256, 512, 128))
        out = None
        if raw is not None:
            scaled = self.pygame.transform.smoothscale(raw, (size, size))
            out = self.pygame.Surface((size, size), self.pygame.SRCALPHA)
            out.blit(scaled, (0, 0))
            mask = self.pygame.Surface((size, size), self.pygame.SRCALPHA)
            r = max(12, size // 8)
            self.pygame.draw.rect(mask, (255, 255, 255, 255),
                                  (0, 0, size, size), border_radius=r)
            out.blit(mask, (0, 0), special_flags=self.pygame.BLEND_RGBA_MULT)
        self._logo_cache = (size, out)
        return out

    def _draw_score_ticker(self, s, w, h):
        """Laufband mit den besten Highscores am unteren Rand des Startbildschirms."""
        ui = self.ui
        entries = [(cls.name, int(self.game_list.scores.get(cls.highscore_key, 0)))
                   for cls in self._game_classes]
        entries = [(n, v) for n, v in entries if v > 0]
        if not entries:
            return
        entries.sort(key=lambda e: -e[1])
        key = tuple(entries[:10])
        if key != getattr(self, "_ticker_key", None):
            text = t("app.top_scores").upper() + "   —   " + "   ·   ".join(
                f"{n.upper()}  " + f"{v:,}".replace(",", " ") for n, v in key)
            self._ticker_img = ui.font(14, bold=True).render(
                text, True, ui.TEXT_DIM)
            self._ticker_key = key

        img = self._ticker_img
        band_h = 26
        band_y = h - 64
        # Halbtransparentes Band (pro Breite gecacht) + zarte Akzentlinien.
        band = getattr(self, "_ticker_band", None)
        if band is None or band.get_width() != w:
            band = self.pygame.Surface((w, band_h), self.pygame.SRCALPHA)
            band.fill((14, 18, 32, 130))
            self._ticker_band = band
        s.blit(band, (0, band_y))
        line_col = ui.mix(ui.BG_BOTTOM, ui.ACCENT, 0.35)
        self.pygame.draw.line(s, line_col, (0, band_y), (w, band_y))
        self.pygame.draw.line(s, line_col, (0, band_y + band_h),
                              (w, band_y + band_h))

        # Nahtloses Scrollen: Bild mehrfach versetzt zeichnen, Clip aufs Band.
        speed, gap = 42.0, 140
        span = img.get_width() + gap
        off = int((self.pygame.time.get_ticks() / 1000.0 * speed) % span)
        y = band_y + band_h // 2 - img.get_height() // 2
        prev_clip = s.get_clip()
        s.set_clip((0, band_y, w, band_h))
        x = -off
        while x < w:
            s.blit(img, (x, y))
            x += span
        s.set_clip(prev_clip)

    def _draw_menu_screen(self):
        """Start-/Leerlaufbildschirm: Aurora-Hintergrund, schwebendes Logo mit
        Orbit-Funken, Highscore-Laufband und pulsierender Hinweis."""
        ui = self.ui
        w, h, s = self.game_w, self.game_h, self.canvas
        ui.draw_background(s, w, h)

        ticks = self.pygame.time.get_ticks() / 1000.0
        cx, cy = w // 2, h // 2
        bob = int(6 * math.sin(ticks * 1.3))   # sanftes Auf und Ab

        # Logo-Grafik mit weichem Akzent-Glow (Fallback: Schriftzug "PyGameZ").
        size = min(176, max(112, h // 3))
        logo = self._menu_logo(size)
        center_y = cy - 46 + bob
        if logo is not None:
            lrect = logo.get_rect(center=(cx, center_y))
            rad = max(12, size // 8)
            glow = self.pygame.Surface((lrect.w + 26, lrect.h + 26),
                                       self.pygame.SRCALPHA)
            self.pygame.draw.rect(glow, (*ui.ACCENT, int(55 + 35 * ui.pulse(1.4))),
                                  glow.get_rect(), border_radius=rad + 8)
            s.blit(glow, glow.get_rect(center=lrect.center))
            s.blit(logo, lrect)
            self.pygame.draw.rect(s, ui.ACCENT, lrect.inflate(4, 4), 2,
                                  border_radius=rad + 2)
            base_y, line_w = lrect.bottom - bob, lrect.w
        else:
            logo_font = ui.font(min(64, max(40, w // 11)), bold=True)
            glow = logo_font.render("PyGameZ", True, ui.ACCENT)
            glow.set_alpha(int(40 + 30 * ui.pulse(1.4)))
            img = ui.grad_text(logo_font, "PyGameZ")
            for dx, dy in ((-2, 0), (2, 0), (0, -2), (0, 2)):
                s.blit(glow, glow.get_rect(center=(cx + dx, center_y + dy)))
            s.blit(img, img.get_rect(center=(cx, center_y)))
            base_y = cy - 46 + img.get_height() // 2
            line_w = img.get_width() + 30

        # Drei kleine Funken kreisen ums Logo (je eigene Bahn/Tempo/Farbe).
        for k, col in enumerate((ui.ACCENT, ui.ACCENT2, ui.GOLD)):
            ang = ticks * (0.6 + 0.17 * k) + k * 2.09
            ox = cx + int(math.cos(ang) * (size // 2 + 34))
            oy = center_y + int(math.sin(ang) * (size // 2 + 12))
            if 2 <= ox < w - 2 and 2 <= oy < h - 2:
                g = ui.mix((0, 0, 0), col, 0.5)
                s.fill(g, (ox - 2, oy - 2, 5, 5),
                       special_flags=self.pygame.BLEND_ADD)
                self.pygame.draw.circle(s, col, (ox, oy), 2)

        # Akzentlinie + Untertitel (übersetzt).
        self.pygame.draw.rect(s, ui.ACCENT, (cx - line_w // 2, base_y + 14, line_w, 3),
                              border_radius=2)
        sub = ui.font(18).render(t("app.menu_title"), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, base_y + 40)))

        # Pulsierender Hinweis ("Wähle links ein Spiel aus.")
        hint = ui.font(15).render(t("app.menu_sub"), True, ui.ACCENT)
        hint.set_alpha(int(255 * ui.pulse(2.2, lo=0.45)))
        s.blit(hint, hint.get_rect(center=(cx, base_y + 72)))

        # Highscore-Laufband + Fußzeile mit dezenten Eckdaten.
        self._draw_score_ticker(s, w, h)
        ui.draw_footer(s, w, h, f"{len(self._game_classes)} Games   ·   "
                                f"{self.game_w}x{self.game_h} @ {self.fps} FPS")

    def _draw_pause_overlay(self):
        """Echter Weichzeichner über dem Spielbild + zentrierte Pause-Karte."""
        ui = self.ui
        pygame = self.pygame
        w, h, s = self.game_w, self.game_h, self.canvas

        # Günstiger Blur: stark herunter- und wieder hochskalieren.
        small = pygame.transform.smoothscale(s, (max(1, w // 10), max(1, h // 10)))
        s.blit(pygame.transform.smoothscale(small, (w, h)), (0, 0))
        overlay = pygame.Surface((w, h), pygame.SRCALPHA)
        overlay.fill((8, 10, 18, 140))
        s.blit(overlay, (0, 0))

        # Karte in der Mitte mit pulsierendem Akzentrahmen
        cw, ch = min(360, w - 40), 150
        card = pygame.Rect(w // 2 - cw // 2, h // 2 - ch // 2, cw, ch)
        ui.draw_panel(s, card, radius=14)
        pygame.draw.rect(s, ui.ACCENT, (card.x, card.y, card.w, 4),
                         border_top_left_radius=14,
                         border_top_right_radius=14)
        pygame.draw.rect(s, ui.mix(ui.BORDER, ui.ACCENT, ui.pulse(2.0)),
                         card.inflate(8, 8), width=1, border_radius=16)

        img = ui.grad_text(ui.font(42, bold=True), t("app.pause"))
        s.blit(img, img.get_rect(center=(card.centerx, card.centery - 18)))
        img2 = ui.font(16).render(t("app.pause_resume"), True, ui.TEXT_DIM)
        img2.set_alpha(int(255 * ui.pulse(2.0, lo=0.5)))
        s.blit(img2, img2.get_rect(center=(card.centerx, card.centery + 34)))

    def _set_state_dot(self, color):
        """Färbt die Zustands-LED in der Status-Karte um."""
        self._dot.itemconfig(self._dot_id, fill=color)

    def _update_status(self, game):
        if getattr(game, "is_menu", False):
            self.status_var.set(t("app.status_menu", name=game.name))
            self._set_state_dot(C_ACCENT)
            return
        import highscore
        hs = highscore.load_highscores().get(game.highscore_key, 0)
        if game.paused:
            zustand, dot = t("app.state_pause"), C_GOLD
        elif game.game_over:
            zustand, dot = t("app.state_over"), C_RED
        else:
            zustand, dot = t("app.state_running"), C_GREEN
        self._set_state_dot(dot)
        self.status_var.set(
            t("app.status", name=game.name, state=zustand, score=game.score, hs=hs))

    # ----- Sauberes Beenden ---------------------------------------------

    def beenden(self):
        """Schliesst Pygame und Tkinter sauber."""
        if self._closing:
            return
        self._closing = True
        if self.current:
            self._highscore_speichern(self.current)
        try:
            self.pygame.quit()
        except Exception:
            pass
        try:
            self.root.destroy()
        except Exception:
            pass

    def run(self):
        self.root.mainloop()


def _check_dependencies():
    """Prueft vor dem Start, ob pygame verfuegbar ist.

    Ohne diese Pruefung wuerde das Tkinter-Fenster kurz erscheinen und beim
    spaeteren 'import pygame' sofort wieder verschwinden - genau das passiert
    auf einem PC ohne installiertes pygame (z. B. ohne .venv). Stattdessen
    zeigen wir eine verstaendliche Meldung.
    """
    try:
        import pygame  # noqa: F401
        return True
    except ImportError:
        msg = (
            "Das Modul 'pygame' ist nicht installiert.\n\n"
            "So behebst du das:\n"
            "  - Unter Windows einfach start.bat ausfuehren\n"
            "    (installiert pygame automatisch), ODER\n"
            "  - im Terminal:  python -m pip install pygame\n\n"
            "In PyCharm: pygame im Interpreter des Projekts installieren."
        )
        try:
            import tkinter as _tk
            from tkinter import messagebox as _mb
            _r = _tk.Tk()
            _r.withdraw()
            _mb.showerror("pygame fehlt", msg)
            _r.destroy()
        except Exception:
            pass
        print(msg, file=sys.stderr)
        return False


if __name__ == "__main__":
    if _check_dependencies():
        App().run()
    else:
        sys.exit(1)
