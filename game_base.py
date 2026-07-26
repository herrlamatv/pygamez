# -*- coding: utf-8 -*-
"""
game_base.py
============
Gemeinsame Grundlagen für alle Spiele.

- InputEvent: Eine kleine, plattform-/toolkit-neutrale Ereignis-Klasse.
  Wir benutzen NICHT die pygame-Ereigniswarteschlange, weil diese beim
  Einbetten über SDL_WINDOWID unzuverlässig ist (Tastatur-/Maus-Events
  landen beim Tkinter-Fenster). Stattdessen fangen wir Events in Tkinter
  ab und reichen sie als InputEvent an das aktive Spiel weiter.

- Game: Basisklasse mit update(), draw() und handle_event().
  Jedes Spiel erbt davon.
"""

import pygame

import audio
import i18n
import settings as settings_mod
import ui


class LocalizedName:
    """Sprachabhängiger Menü-Name als Deskriptor.

    Das Menü liest den Namen direkt über die KLASSE (``cls.name``) und
    übersetzt ihn nicht - dieser Deskriptor liefert den Namen trotzdem in
    der aktiven Sprache. Verwendung:

        class ChessGame(Game):
            name = LocalizedName("Chess", de="Schach", fr="Échecs",
                                 es="Ajedrez", pt="Xadrez")

    Der erste Wert ist der Fallback für alle nicht angegebenen Sprachen.
    """

    def __init__(self, default, **per_lang):
        self.default = default
        self.per_lang = per_lang

    def __get__(self, obj, owner=None):
        return self.per_lang.get(i18n.get_language(), self.default)


class InputEvent:
    """Ein vereinheitlichtes Eingabe-Ereignis (von Tkinter erzeugt)."""

    KEYDOWN = "keydown"      # Taste gedrückt
    KEYUP = "keyup"          # Taste losgelassen
    MOUSEDOWN = "mousedown"  # Maustaste gedrückt
    MOUSEUP = "mouseup"      # linke Maustaste losgelassen (für Drag & Drop)
    MOUSEMOVE = "mousemove"  # Maus bewegt
    MOUSEREL = "mouserel"    # RELATIVE Mausbewegung (Pointer-Capture, FPS-Look)
    WHEEL = "wheel"          # Mausrad gedreht (delta in Rasten, + = hoch)

    def __init__(self, kind, key=None, pos=None, button=1, delta=0, rel=None):
        self.kind = kind      # einer der obigen Strings
        self.key = key        # z.B. "Up", "Left", "space", "w" (Tkinter-keysym)
        self.pos = pos        # (x, y) relativ zur Spielfläche, bei Maus-Events
        self.button = button  # Maustaste: 1 = links, 3 = rechts (bei MOUSEDOWN)
        self.delta = delta    # Mausrad-Rasten (bei WHEEL), positiv = nach oben
        self.rel = rel        # (dx, dy) in Fenster-Pixeln (bei MOUSEREL)


class Game:
    """
    Basisklasse für alle Spiele.

    Jedes Spiel zeichnet auf 'surface' (das eingebettete pygame-Display).
    Die Spielfläche ist 'width' x 'height' Pixel gross.

    Optionale Erweiterungen (werden vom Shell-Code per getattr/hasattr
    abgefragt, ein Spiel definiert sie nur bei Bedarf):

    - ``MODES = [(mode_key, i18n_key), ...]`` als Klassen-Attribut ersetzt
      im Vorbereitungs-Screen die Einzel-/Mehrspieler-Buttons durch eigene
      Modus-Buttons; der gewählte mode_key landet in ``self.mode``.
    - ``capture_mouse`` (bool, siehe unten): True = Cursor wird eingefangen
      und versteckt, das Spiel erhält relative MOUSEREL-Bewegungen
      (FPS-Look). Darf zur Laufzeit umgeschaltet werden.
    - ``on_surface_changed()`` (siehe unten): Layout/Schriften aus den
      neuen width/height-Werten neu aufbauen.
    """

    name = "Spiel"            # Anzeigename (wird im Menü verwendet);
                              # sprachabhängig über LocalizedName möglich
    highscore_key = "base"    # Schlüssel für die Highscore-JSON

    # Bietet das Spiel einen echten 2-Spieler-Modus? (Menü zeigt "Mehrspieler")
    supports_multiplayer = False
    # Menü-/Options-Screens setzen dies auf True (kein Highscore/Pause).
    is_menu = False
    # Möchte das Spiel Rechtsklicks erhalten? (MOUSEDOWN mit button=3).
    # Standard False, damit sich das Verhalten bestehender Spiele nicht ändert.
    wants_right_click = False
    # Pointer-Capture: True = Cursor einfangen, Spiel bekommt MOUSEREL-Events.
    capture_mouse = False

    def __init__(self, surface, width, height, mode="single", game_settings=None):
        self.surface = surface
        self.width = width
        self.height = height

        # Spielmodus + Einstellungen (Tastenbelegung, Sound, Haptik)
        self.mode = mode
        self.multiplayer = (mode == "multi")
        self.settings = game_settings or settings_mod.load_settings()
        self.controls = self.settings.get("controls", settings_mod.DEFAULT_CONTROLS)

        self.score = 0
        self.game_over = False
        self.paused = False

        # Standard-Schriftarten im aktiven Theme (gecacht über ui.font).
        # Für Stellen, die feste Zeichenbreiten brauchen: ui.font(n, mono=True).
        self.font = ui.font(22)
        self.big_font = ui.font(48, bold=True)

        # Akzentfarbe des Spiels (= Sidebar-Farbe aus ui.GAME_COLORS);
        # für Titel/HUD verwenden, damit jedes Spiel seine Identität behält.
        self.accent = ui.game_color(type(self).__name__)

        self.reset()

    # ----- Von Unterklassen zu überschreiben ---------------------------

    def reset(self):
        """Setzt das Spiel in den Startzustand zurück."""
        self.score = 0
        self.game_over = False

    def update(self, dt):
        """Aktualisiert die Spiellogik. dt = vergangene Sekunden seit letztem Frame."""
        raise NotImplementedError

    def draw(self):
        """Zeichnet den aktuellen Zustand auf self.surface."""
        raise NotImplementedError

    def handle_event(self, event):
        """Verarbeitet ein InputEvent."""
        raise NotImplementedError

    # ----- Optionale Hooks ---------------------------------------------

    def on_surface_changed(self):
        """Wird nach einem Auflösungswechsel aufgerufen.

        self.surface/width/height sind dann bereits aktualisiert; Spiele
        überschreiben das, um Layout und Schriftgrössen neu zu berechnen.
        """
        pass

    # ----- Hilfsfunktionen für alle Spiele -----------------------------

    def draw_center_text(self, text, font, color, y_offset=0):
        """Zeichnet zentrierten Text auf die Spielfläche."""
        img = font.render(text, True, color)
        rect = img.get_rect(center=(self.width // 2, self.height // 2 + y_offset))
        self.surface.blit(img, rect)

    # ----- Steuerung & Feedback -----------------------------------------

    def key_for(self, player, action):
        """Liefert den belegten keysym für (player, action) oder None."""
        return self.controls.get(player, {}).get(action)

    def is_action(self, key, action, player=None):
        """
        True, wenn 'key' der Taste für 'action' entspricht.

        Ohne 'player' wird gegen BEIDE Spieler geprüft (praktisch für den
        Einzelspieler-Modus: sowohl P1- als auch P2-Tasten steuern die Figur).
        Mit 'player' ("p1"/"p2") nur gegen dessen Belegung (Mehrspieler).
        """
        players = (player,) if player else ("p1", "p2")
        return any(key == self.controls.get(p, {}).get(action) for p in players)

    def play_sound(self, name):
        """Spielt einen Soundeffekt (respektiert die Sound-Einstellung)."""
        audio.play(name, self.settings)

    def rumble(self, ms=120):
        """Löst Gamepad-Vibration aus (respektiert die Haptik-Einstellung)."""
        audio.rumble(self.settings, ms)

    # ----- Statistik & Erfolge -------------------------------------------

    def report_result(self, won):
        """Meldet Sieg (True) / Niederlage (False) des Spielers an die Statistik.

        Für Spiele mit klarem Gewonnen/Verloren-Konzept (Brett-, Karten- und
        Rätselspiele); gedacht für den Einzelspieler-/KI-Modus. Pro Partie
        zählt nur der erste Aufruf - den Riegel setzt main.py beim Neustart
        (Game-Over -> neue Partie) automatisch zurück.
        """
        if self.is_menu or getattr(self, "_result_reported", False):
            return
        self._result_reported = True
        import stats
        stats.record_result(self.highscore_key, bool(won))
        import achievements
        achievements.check_stats()

    def ach_event(self, event_id):
        """Löst ein Erfolgs-Ereignis aus (z.B. 'kniffel_five').

        Die bekannten Ereignisse stehen in achievements.py; unbekannte werden
        ignoriert. Bereits freigeschaltete Erfolge lösen nicht erneut aus.
        """
        if self.is_menu:
            return
        import achievements
        achievements.event(event_id)