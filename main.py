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
"""

import os
import sys
import tkinter as tk
from tkinter import ttk

# Das Verzeichnis dieser Datei sicher auf sys.path legen, damit die lokalen
# Module (i18n, settings, audio, games, ...) unabhängig vom Startverzeichnis
# gefunden werden - egal ob per Doppelklick, IDE oder "python main.py".
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import i18n
from i18n import t

# Standard-Spielfläche und -Bildrate. Die tatsächlichen Werte kommen aus den
# Einstellungen (settings.py) und liegen zur Laufzeit in self.game_w/-_h/-fps.
GAME_W = 640
GAME_H = 480
FPS = 60


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

    # Farbschema der Tkinter-Seite (abgestimmt auf ui.py der Pygame-Seite)
    C_SIDEBAR = "#141824"      # Sidebar-Hintergrund
    C_HEADER = "#0f1320"       # Kopfbereich (etwas dunkler)
    C_CARD = "#1d2333"         # Karten/Panels
    C_BTN = "#232a3d"          # Buttons normal
    C_BTN_HOVER = "#2e3852"    # Buttons unter der Maus
    C_ACCENT = "#589cff"       # Primär-Akzent
    C_DANGER = "#8e3540"       # Beenden
    C_DANGER_HOVER = "#ab414e"
    C_TEXT = "#e9edf5"
    C_TEXT_DIM = "#98a2b8"

    def _hover(self, widget, normal, hover):
        """Bindet einen dezenten Hover-Effekt (Hintergrundwechsel) an ein Widget."""
        widget.bind("<Enter>", lambda e: widget.config(bg=hover))
        widget.bind("<Leave>", lambda e: widget.config(bg=normal))

    def _make_button(self, parent, text, command, kind="normal", anchor="center"):
        """Einheitlicher Sidebar-Button mit Hover-Effekt und Hand-Cursor."""
        if kind == "danger":
            bg, hov = self.C_DANGER, self.C_DANGER_HOVER
            fnt, pady = ("Segoe UI", 10, "bold"), 5
        elif kind == "game":
            bg, hov = self.C_BTN, self.C_BTN_HOVER
            fnt, pady = ("Segoe UI", 11), 4
        else:
            bg, hov = self.C_BTN, self.C_BTN_HOVER
            fnt, pady = ("Segoe UI", 9), 3
        btn = tk.Button(parent, text=text, command=command, bg=bg, fg=self.C_TEXT,
                        relief="flat", bd=0, font=fnt, cursor="hand2",
                        activebackground=hov, activeforeground=self.C_TEXT,
                        anchor=anchor, padx=10, pady=pady,
                        highlightthickness=0)
        self._hover(btn, bg, hov)
        return btn

    def _build_ui(self):
        # Linke Seite: Sidebar mit Kopf, Spieleliste, Aktionen und Status.
        menu = tk.Frame(self.root, width=216, bg=self.C_SIDEBAR)
        menu.pack(side="left", fill="y")
        menu.pack_propagate(False)
        self._menu_frame = menu

        # --- Kopfbereich: App-Name + Untertitel + Akzentlinie -------------
        header = tk.Frame(menu, bg=self.C_HEADER)
        header.pack(fill="x")
        tk.Label(header, text="PyGameZ", fg=self.C_TEXT, bg=self.C_HEADER,
                 font=("Segoe UI", 17, "bold")).pack(pady=(12, 0))
        self._header_sub = tk.Label(header, text=t("app.menu_title"),
                                    fg=self.C_TEXT_DIM, bg=self.C_HEADER,
                                    font=("Segoe UI", 9))
        self._header_sub.pack(pady=(0, 10))
        tk.Frame(menu, bg=self.C_ACCENT, height=2).pack(fill="x")

        # --- Status-Karte unten ----------------------------------------------
        # (Der untere Block wird VOR der Spieleliste gepackt: bottom-Widgets
        #  bekommen so ihren Platz zuerst und bleiben auch bei kleiner
        #  Fensterhöhe sichtbar - die Spieleliste gibt dann nach.)
        self.status_var = tk.StringVar(value=t("app.no_game"))
        status_card = tk.Frame(menu, bg=self.C_CARD,
                               highlightbackground="#2a3147",
                               highlightthickness=1)
        status_card.pack(side="bottom", fill="x", padx=12, pady=(4, 12))
        tk.Frame(status_card, bg=self.C_ACCENT, width=3).pack(
            side="left", fill="y")
        tk.Label(status_card, textvariable=self.status_var, fg=self.C_TEXT_DIM,
                 bg=self.C_CARD, font=("Consolas", 9), justify="left",
                 anchor="w", wraplength=168, padx=8, pady=6).pack(fill="x")

        # --- Steuer-Buttons unten (Referenzen für refresh_language) ---------
        self._ctrl_buttons = {}
        self._ctrl_buttons["quit"] = self._make_button(
            menu, t("app.quit"), self.beenden, kind="danger")
        self._ctrl_buttons["quit"].pack(side="bottom", fill="x", padx=12, pady=(4, 8))

        for key, cmd in (("back_to_menu", self.zum_menü),
                         ("fullscreen", self.toggle_fullscreen),
                         ("language", self.open_language),
                         ("options", self.open_options)):
            btn = self._make_button(menu, t("app." + key), cmd, anchor="w")
            btn.pack(side="bottom", fill="x", padx=12, pady=2)
            self._ctrl_buttons[key] = btn

        ttk.Separator(menu, orient="horizontal").pack(side="bottom", fill="x",
                                                      padx=12, pady=6)

        # --- Abschnitt: Spiele (füllt den verbleibenden Platz) --------------
        self.games_label = tk.Label(menu, text=t("app.games"), fg=self.C_ACCENT,
                                    bg=self.C_SIDEBAR, anchor="w",
                                    font=("Segoe UI", 9, "bold"))
        self.games_label.pack(fill="x", padx=16, pady=(10, 4))

        self.button_frame = tk.Frame(menu, bg=self.C_SIDEBAR)
        self.button_frame.pack(fill="x", padx=12)

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

    # Eigene Akzentfarbe je Spiel (linker Farbstreifen im Menü).
    _GAME_COLORS = {
        "SnakeGame": "#6ecd8c", "PongGame": "#589cff", "TicTacToeGame": "#f0a05a",
        "BreakoutGame": "#e15f5f", "TetrisGame": "#b07fe8", "InvadersGame": "#5ad4d4",
        "Game2048": "#f5cd64", "AirHockeyGame": "#6fe0d0", "MinesweeperGame": "#f08fb0",
        "AsteroidsGame": "#b9c2d9", "PacmanGame": "#ffd83b",
    }

    def _fit_sidebar(self):
        """Passt Sidebar-Breite und Fenster-Mindesthöhe an den Inhalt an.

        Nötig, weil Beschriftungen je nach Sprache/DPI unterschiedlich breit
        bzw. hoch ausfallen; die Sidebar hat pack_propagate(False) und würde
        lange Texte sonst abschneiden.
        """
        import tkinter.font as tkfont
        self.root.update_idletasks()

        # Breite: längste Beschriftung (Steuer-Buttons / Spielnamen) + Ränder.
        f_ctrl = tkfont.Font(family="Segoe UI", size=9)
        f_game = tkfont.Font(family="Segoe UI", size=11)
        w_ctrl = max((f_ctrl.measure(b.cget("text"))
                      for b in self._ctrl_buttons.values()), default=0)
        w_game = max((f_game.measure(c.name) for c in self._game_classes),
                     default=0)
        need_w = max(216, w_ctrl + 48, w_game + 44)
        self._menu_frame.config(width=need_w)

        # Höhe: Bedarf aller gepackten Kinder inkl. äußerer pady summieren.
        need_h = 0
        for ch in self._menu_frame.winfo_children():
            if not ch.winfo_manager():
                continue
            pady = ch.pack_info().get("pady", 0)
            if isinstance(pady, (tuple, list)):
                pad = sum(int(p) for p in pady)
            else:
                pad = int(pady) * 2
            need_h += ch.winfo_reqheight() + pad
        self.root.minsize(840, max(480, need_h + 8))

    def _build_game_buttons(self):
        """Spieleliste: kompakte Zeilen mit farbigem Akzentstreifen + Hover."""
        for cls in self._game_classes:
            accent = self._GAME_COLORS.get(cls.__name__, self.C_ACCENT)
            row = tk.Frame(self.button_frame, bg=self.C_BTN, cursor="hand2")
            row.pack(fill="x", pady=2)
            tk.Frame(row, bg=accent, width=4).pack(side="left", fill="y")
            lbl = tk.Label(row, text=cls.name, bg=self.C_BTN, fg=self.C_TEXT,
                           font=("Segoe UI", 11), anchor="w", padx=10, pady=5)
            lbl.pack(side="left", fill="x", expand=True)

            def enter(_e, r=row, l=lbl):
                r.config(bg=self.C_BTN_HOVER)
                l.config(bg=self.C_BTN_HOVER)

            def leave(_e, r=row, l=lbl):
                r.config(bg=self.C_BTN)
                l.config(bg=self.C_BTN)

            for w in (row, lbl):
                w.bind("<Button-1>", lambda _e, c=cls: self.spiel_starten(c))
                w.bind("<Enter>", enter)
                w.bind("<Leave>", leave)

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

        # Gemeinsames UI-Toolkit (Palette/Verläufe/Panels) für alle Screens.
        import ui
        self.ui = ui

    def _set_window_icon(self):
        """Setzt das Fenster-/Taskleisten-Icon aus logo/pygamez-*.jpg.

        Tk kann JPG nicht direkt laden, deshalb derselbe Weg wie beim
        Spielbild: pygame lädt das JPG, wir wandeln es in PPM-Daten um und
        machen daraus ein tk.PhotoImage. Es werden alle Größen übergeben -
        Tk/Windows wählt die passende für Titelleiste, Taskleiste und
        Alt+Tab selbst aus. Fehlt eine Datei, wird sie übersprungen; ganz
        ohne Logo startet das Programm einfach mit dem Standard-Icon.
        """
        basis = os.path.dirname(os.path.abspath(__file__))
        self._icons = []          # Referenzen halten, sonst räumt Tk die Bilder weg
        for size in (512, 256, 128):
            pfad = os.path.join(basis, "logo", f"pygamez-{size}.jpg")
            try:
                img = self.pygame.image.load(pfad)
                w, h = img.get_size()
                data = b"P6 %d %d 255 " % (w, h) \
                    + self.pygame.image.tobytes(img, "RGB")
                self._icons.append(tk.PhotoImage(data=data, format="ppm"))
            except Exception:
                pass
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
                from game_base import InputEvent
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
        self.show_screen(PreGameScreen(self.canvas, self.game_w, self.game_h,
                                       self, game_cls))

    def launch_game(self, game_cls, mode):
        """Startet das eigentliche Spiel im gewählten Modus mit den Einstellungen."""
        self.current = game_cls(self.canvas, self.game_w, self.game_h,
                                mode=mode, game_settings=self.settings)
        self.current.paused = False
        self.embed.focus_set()

    def show_screen(self, screen):
        """Macht einen Menü-Screen (Vorspiel/Optionen) zum aktiven 'current'."""
        self.current = screen
        self.embed.focus_set()

    def back_to_menu(self):
        """Zurück zum leeren Startbildschirm (ohne Highscore-Effekte)."""
        self.current = None
        self.status_var.set(t("app.no_game"))

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
            btn.config(text=t("app." + key))
        if self.current is None:
            self.status_var.set(t("app.no_game"))
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
        if record:
            text, farbe = t("app.new_highscore", score=game.score), ui.GOLD
        else:
            text, farbe = t("app.highscore", hs=hs), ui.TEXT_DIM

        img = ui.font(18, bold=True).render(text, True, farbe)
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
            if not game.game_over:
                game._hs_saved = False

            # Highscore bei Game Over für jedes Spiel einblenden.
            if game.game_over and not getattr(game, "is_menu", False):
                self._draw_highscore_overlay(game)

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

    def _draw_menu_screen(self):
        """Start-/Leerlaufbildschirm: animierter Hintergrund + Logo + Hinweis."""
        ui = self.ui
        w, h, s = self.game_w, self.game_h, self.canvas
        ui.draw_background(s, w, h)

        cx, cy = w // 2, h // 2

        # Logo "PyGameZ" mit weichem Akzent-Glow dahinter.
        logo_font = ui.font(min(64, max(40, w // 11)), bold=True)
        glow = logo_font.render("PyGameZ", True, ui.ACCENT)
        glow.set_alpha(int(40 + 30 * ui.pulse(1.4)))
        img = logo_font.render("PyGameZ", True, ui.TEXT)
        for dx, dy in ((-2, 0), (2, 0), (0, -2), (0, 2)):
            s.blit(glow, glow.get_rect(center=(cx + dx, cy - 46 + dy)))
        s.blit(img, img.get_rect(center=(cx, cy - 46)))

        # Akzentlinie + Untertitel (übersetzt).
        lw = img.get_width() + 30
        self.pygame.draw.rect(s, ui.ACCENT, (cx - lw // 2, cy - 8, lw, 3),
                              border_radius=2)
        sub = ui.font(18).render(t("app.menu_title"), True, ui.TEXT_DIM)
        s.blit(sub, sub.get_rect(center=(cx, cy + 18)))

        # Pulsierender Hinweis ("Wähle links ein Spiel aus.")
        hint = ui.font(15).render(t("app.menu_sub"), True, ui.ACCENT)
        hint.set_alpha(int(255 * ui.pulse(2.2, lo=0.45)))
        s.blit(hint, hint.get_rect(center=(cx, cy + 56)))

        # Fußzeile mit dezenten Eckdaten.
        ui.draw_footer(s, w, h, f"{len(self._game_classes)} Games   ·   "
                                f"{self.game_w}x{self.game_h} @ {self.fps} FPS")

    def _draw_pause_overlay(self):
        """Abdunkeln + zentrierte Pause-Karte mit Akzentrahmen."""
        ui = self.ui
        w, h, s = self.game_w, self.game_h, self.canvas
        overlay = self.pygame.Surface((w, h), self.pygame.SRCALPHA)
        overlay.fill((8, 10, 18, 170))
        s.blit(overlay, (0, 0))

        # Karte in der Mitte
        cw, ch = min(360, w - 40), 150
        card = self.pygame.Rect(w // 2 - cw // 2, h // 2 - ch // 2, cw, ch)
        ui.draw_panel(s, card, radius=14)
        self.pygame.draw.rect(s, ui.ACCENT, (card.x, card.y, card.w, 4),
                              border_top_left_radius=14,
                              border_top_right_radius=14)

        img = ui.font(42, bold=True).render(t("app.pause"), True, ui.TEXT)
        s.blit(img, img.get_rect(center=(card.centerx, card.centery - 18)))
        img2 = ui.font(16).render(t("app.pause_resume"), True, ui.TEXT_DIM)
        img2.set_alpha(int(255 * ui.pulse(2.0, lo=0.5)))
        s.blit(img2, img2.get_rect(center=(card.centerx, card.centery + 34)))

    def _update_status(self, game):
        if getattr(game, "is_menu", False):
            self.status_var.set(t("app.status_menu", name=game.name))
            return
        import highscore
        hs = highscore.load_highscores().get(game.highscore_key, 0)
        zustand = (t("app.state_pause") if game.paused
                   else (t("app.state_over") if game.game_over
                         else t("app.state_running")))
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
