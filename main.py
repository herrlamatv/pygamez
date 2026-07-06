# -*- coding: utf-8 -*-
"""
main.py
=======
Desktop-Spielesammlung: Tkinter (Oberfläche/Menü) + Pygame (eingebettetes Display).

So funktioniert die Einbettung
------------------------------
Pygame/SDL kann sein Fenster in ein vorhandenes Fenster eines anderen Toolkits
zeichnen, wenn man ihm dessen native Fenster-ID gibt:

    os.environ['SDL_WINDOWID'] = str(frame.winfo_id())

Das MUSS gesetzt werden, BEVOR pygame.display.set_mode() aufgerufen wird.
'frame.winfo_id()' liefert das native Handle des Tkinter-Frames (HWND unter
Windows, XID unter Linux/X11). pygame zeichnet dann direkt in diesen Frame.

Zusätzlich kann der Video-Treiber gesetzt werden (siehe Plattformhinweise unten).

Damit sich Tkinter und Pygame nicht gegenseitig blockieren, gibt es KEINE eigene
while-Schleife für pygame. Stattdessen treibt Tkinters Ereignisschleife alles an:
root.after(...) ruft regelmäßig _loop() auf, das ein einzelnes Frame des Spiels
aktualisiert und zeichnet. Tastatur/Maus fangen wir über Tkinter-Bindings ab und
reichen sie als InputEvent an das aktive Spiel weiter (zuverlässiger als
pygame.event beim Einbetten).
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


def _configure_sdl_for_embedding(window_id):
    """Setzt die nötigen Umgebungsvariablen für die Pygame-Einbettung."""
    os.environ["SDL_WINDOWID"] = str(window_id)

    if sys.platform.startswith("win"):
        # SDL2 (pygame 2.x): den Standard-Treiber ('windows') verwenden -> KEIN
        # SDL_VIDEODRIVER setzen. Hinweis: das alte 'windib' galt nur für SDL1
        # (pygame 1.9.x) und führt unter pygame 2 zu einem Fehler.
        pass
    else:
        # Linux/X11: x11-Treiber erzwingen, damit SDL_WINDOWID greift.
        # (Unter reinem Wayland funktioniert die Einbettung i.d.R. nicht;
        #  dann hilft oft 'SDL_VIDEODRIVER=x11' zusammen mit XWayland.)
        os.environ["SDL_VIDEODRIVER"] = "x11"


class App:
    """Die Tkinter-Anwendung mit eingebettetem Pygame-Display."""

    def __init__(self):
        # Sprache laden (aus mem.json), bevor irgendein Text aufgebaut wird.
        i18n.init()

        self.root = tk.Tk()
        self.root.title(t("app.title"))
        # Fenster darf vergrößert/verkleinert werden -> nötig für Vollbild,
        # bei dem das Pygame-Display weiter eingebettet "im Fenster" bleibt.
        self.root.resizable(True, True)
        self.root.minsize(840, 480)
        self.root.protocol("WM_DELETE_WINDOW", self.beenden)

        self._closing = False
        self._fullscreen = False
        self.current = None          # aktuell laufendes Spiel (Game-Objekt)
        # Skalierung/Versatz für die Darstellung der logischen Fläche
        self._scale = 1.0
        self._off = (0, 0)

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
        self._bind_events()

        # Sound-Engine starten (nach pygame.init()).
        import audio
        audio.init()

        # Pygame importieren wir erst nach _init_pygame (dort gesetzt), Spiele danach.
        from games import ALL_GAMES
        self._game_classes = ALL_GAMES
        self._build_game_buttons()

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

    def _build_ui(self):
        # Linke Seite: Menü
        menu = tk.Frame(self.root, width=200, bg="#1c1f29")
        menu.pack(side="left", fill="y")
        menu.pack_propagate(False)

        self.games_label = tk.Label(menu, text=t("app.games"), fg="#ffffff",
                                    bg="#1c1f29", font=("Segoe UI", 16, "bold"))
        self.games_label.pack(pady=(16, 8))

        self.button_frame = tk.Frame(menu, bg="#1c1f29")
        self.button_frame.pack(fill="x", padx=12)

        # Punktestand-Anzeige
        self.status_var = tk.StringVar(value=t("app.no_game"))
        tk.Label(menu, textvariable=self.status_var, fg="#c8d0e0", bg="#1c1f29",
                 font=("Consolas", 10), justify="left", wraplength=176).pack(
            side="bottom", fill="x", padx=12, pady=8)

        ttk.Separator(menu, orient="horizontal").pack(side="bottom", fill="x", pady=4)

        # Steuer-Buttons unten (Referenzen gemerkt -> refresh_language kann neu beschriften)
        self._ctrl_buttons = {}
        self._ctrl_buttons["quit"] = tk.Button(
            menu, text=t("app.quit"), command=self.beenden,
            bg="#a23b3b", fg="white", relief="flat", font=("Segoe UI", 11, "bold"))
        self._ctrl_buttons["quit"].pack(side="bottom", fill="x", padx=12, pady=(4, 12))

        for key, cmd in (("back_to_menu", self.zum_menü),
                         ("fullscreen", self.toggle_fullscreen),
                         ("language", self.open_language),
                         ("options", self.open_options)):
            btn = tk.Button(menu, text=t("app." + key), command=cmd,
                            bg="#2f3645", fg="white", relief="flat",
                            font=("Segoe UI", 9))
            btn.pack(side="bottom", fill="x", padx=12, pady=4)
            self._ctrl_buttons[key] = btn

        # Rechte Seite: eingebettetes Pygame-Display.
        # fill/expand sorgt dafür, dass der Frame (und damit das Pygame-Display)
        # im Vollbild bzw. beim Vergrößern den verfügbaren Platz ausfüllt.
        self.embed = tk.Frame(self.root, width=self.game_w, height=self.game_h,
                              bg="black", highlightthickness=0)
        self.embed.pack(side="right", fill="both", expand=True)
        self.embed.pack_propagate(False)
        # Fokus, damit Tastatur-Events ankommen
        self.embed.focus_set()

    def _build_game_buttons(self):
        for cls in self._game_classes:
            tk.Button(self.button_frame, text=cls.name,
                      command=lambda c=cls: self.spiel_starten(c),
                      bg="#3a4357", fg="white", relief="flat",
                      activebackground="#4a566f",
                      font=("Segoe UI", 11)).pack(fill="x", pady=4)

    # ----- Pygame-Einbettung --------------------------------------------

    def _init_pygame(self):
        _configure_sdl_for_embedding(self.embed.winfo_id())

        import pygame
        self.pygame = pygame
        pygame.init()

        # Tatsächliche Größe des eingebetteten Bereichs (kann durch Vollbild/
        # Fenstergröße wachsen). Das Display zeichnet in den Tkinter-Frame.
        self.disp_w = max(self.game_w, self.embed.winfo_width())
        self.disp_h = max(self.game_h, self.embed.winfo_height())
        self.screen = pygame.display.set_mode((self.disp_w, self.disp_h))

        # Alle Spiele zeichnen auf diese LOGISCHE Fläche (game_w x game_h aus den
        # Einstellungen). Im Loop wird sie passend auf das echte Display skaliert.
        self.canvas = pygame.Surface((self.game_w, self.game_h))

        self.clock = pygame.time.Clock()

        # Startbildschirm zeichnen
        self._menu_font = pygame.font.SysFont("consolas", 26, bold=True)
        self._menu_sub = pygame.font.SysFont("consolas", 16)

    # ----- Eingaben von Tkinter zu Spielen weiterreichen ----------------

    def _bind_events(self):
        # Tastatur global am Hauptfenster abfangen
        self.root.bind("<KeyPress>", self._on_key)
        self.root.bind("<KeyRelease>", self._on_key_up)
        # Maus auf der Spielfläche
        self.embed.bind("<Button-1>", self._on_click)
        self.embed.bind("<Motion>", self._on_motion)
        # Klick auf die Fläche holt den Fokus (für Tastatur)
        self.embed.bind("<Button-1>", lambda e: self.embed.focus_set(), add="+")
        # Größe des eingebetteten Bereichs änderte sich (Vollbild/Resize)
        self.embed.bind("<Configure>", self._on_resize)

    def _on_resize(self, event):
        """Passt das Pygame-Display an die neue Frame-Größe an."""
        w, h = max(1, event.width), max(1, event.height)
        if (w, h) == (self.disp_w, self.disp_h):
            return
        self.disp_w, self.disp_h = w, h
        try:
            # Display innerhalb desselben eingebetteten Fensters neu dimensionieren
            self.screen = self.pygame.display.set_mode((w, h))
        except Exception:
            pass
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
        for key, btn in self._ctrl_buttons.items():
            btn.config(text=t("app." + key))
        if self.current is None:
            self.status_var.set(t("app.no_game"))

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
        """Blendet bei Game Over für JEDES Spiel den Highscore unten ein."""
        hs = getattr(game, "_hs_value", 0)
        if getattr(game, "_hs_record", False):
            text, farbe = t("app.new_highscore", score=game.score), (255, 215, 90)
        else:
            text, farbe = t("app.highscore", hs=hs), (200, 205, 220)
        if not getattr(self, "_hs_font", None):
            self._hs_font = self.pygame.font.SysFont("consolas", 20, bold=True)
        img = self._hs_font.render(text, True, farbe)
        self.canvas.blit(img, img.get_rect(
            center=(self.game_w // 2, self.game_h - 16)))

    # ----- Zentrale Game-Loop -------------------------------------------

    def _loop(self):
        if self._closing:
            return

        pygame = self.pygame
        dt = self.clock.tick(self.fps) / 1000.0   # vergangene Zeit in Sekunden

        # pygame-interne Ereignisse leeren (hält SDL "lebendig")
        pygame.event.pump()

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
        """Skaliert self.canvas seitenverhältnistreu auf das echte Display."""
        pygame = self.pygame
        sw, sh = self.disp_w, self.disp_h
        scale = min(sw / self.game_w, sh / self.game_h)
        tw, th = max(1, int(self.game_w * scale)), max(1, int(self.game_h * scale))
        self._scale = scale
        self._off = ((sw - tw) // 2, (sh - th) // 2)

        self.screen.fill((0, 0, 0))                 # schwarze Letterbox-Ränder
        if scale == 1.0 and (tw, th) == (self.game_w, self.game_h):
            self.screen.blit(self.canvas, self._off)
        else:
            self.screen.blit(pygame.transform.scale(self.canvas, (tw, th)), self._off)
        pygame.display.flip()

    def _draw_menu_screen(self):
        self.canvas.fill((18, 20, 28))
        title = self._menu_font.render(t("app.menu_title"), True, (235, 235, 245))
        sub = self._menu_sub.render(t("app.menu_sub"), True, (150, 160, 180))
        self.canvas.blit(title, title.get_rect(
            center=(self.game_w // 2, self.game_h // 2 - 20)))
        self.canvas.blit(sub, sub.get_rect(
            center=(self.game_w // 2, self.game_h // 2 + 20)))

    def _draw_pause_overlay(self):
        overlay = self.pygame.Surface((self.game_w, self.game_h), self.pygame.SRCALPHA)
        overlay.fill((0, 0, 0, 150))
        self.canvas.blit(overlay, (0, 0))
        f = self.pygame.font.SysFont("consolas", 48, bold=True)
        img = f.render(t("app.pause"), True, (240, 240, 240))
        self.canvas.blit(img, img.get_rect(center=(self.game_w // 2, self.game_h // 2)))
        f2 = self.pygame.font.SysFont("consolas", 16)
        img2 = f2.render(t("app.pause_resume"), True, (200, 200, 200))
        self.canvas.blit(img2, img2.get_rect(
            center=(self.game_w // 2, self.game_h // 2 + 40)))

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
