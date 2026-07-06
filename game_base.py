# -*- coding: utf-8 -*-
"""
game_base.py
============
Gemeinsame Grundlagen fuer alle Spiele.

- InputEvent: Eine kleine, plattform-/toolkit-neutrale Ereignis-Klasse.
  Wir benutzen NICHT die pygame-Ereigniswarteschlange, weil diese beim
  Einbetten ueber SDL_WINDOWID unzuverlaessig ist (Tastatur-/Maus-Events
  landen beim Tkinter-Fenster). Stattdessen fangen wir Events in Tkinter
  ab und reichen sie als InputEvent an das aktive Spiel weiter.

- Game: Basisklasse mit update(), draw() und handle_event().
  Jedes Spiel erbt davon.
"""

import pygame

import audio
import settings as settings_mod


class InputEvent:
    """Ein vereinheitlichtes Eingabe-Ereignis (von Tkinter erzeugt)."""

    KEYDOWN = "keydown"      # Taste gedrueckt
    MOUSEDOWN = "mousedown"  # Maustaste gedrueckt
    MOUSEMOVE = "mousemove"  # Maus bewegt

    def __init__(self, kind, key=None, pos=None):
        self.kind = kind      # einer der obigen Strings
        self.key = key        # z.B. "Up", "Left", "space", "w" (Tkinter-keysym)
        self.pos = pos        # (x, y) relativ zur Spielflaeche, bei Maus-Events


class Game:
    """
    Basisklasse fuer alle Spiele.

    Jedes Spiel zeichnet auf 'surface' (das eingebettete pygame-Display).
    Die Spielflaeche ist 'width' x 'height' Pixel gross.
    """

    name = "Spiel"            # Anzeigename (wird im Menue verwendet)
    highscore_key = "base"    # Schluessel fuer die Highscore-JSON

    # Bietet das Spiel einen echten 2-Spieler-Modus? (Menue zeigt "Mehrspieler")
    supports_multiplayer = False
    # Menue-/Options-Screens setzen dies auf True (kein Highscore/Pause).
    is_menu = False

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

        # Standard-Schriftarten (pygame.font wurde von pygame.init() initialisiert)
        self.font = pygame.font.SysFont("consolas", 22)
        self.big_font = pygame.font.SysFont("consolas", 48, bold=True)

        self.reset()

    # ----- Von Unterklassen zu ueberschreiben ---------------------------

    def reset(self):
        """Setzt das Spiel in den Startzustand zurueck."""
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

    # ----- Hilfsfunktionen fuer alle Spiele -----------------------------

    def draw_center_text(self, text, font, color, y_offset=0):
        """Zeichnet zentrierten Text auf die Spielflaeche."""
        img = font.render(text, True, color)
        rect = img.get_rect(center=(self.width // 2, self.height // 2 + y_offset))
        self.surface.blit(img, rect)

    # ----- Steuerung & Feedback -----------------------------------------

    def key_for(self, player, action):
        """Liefert den belegten keysym fuer (player, action) oder None."""
        return self.controls.get(player, {}).get(action)

    def is_action(self, key, action, player=None):
        """
        True, wenn 'key' der Taste fuer 'action' entspricht.

        Ohne 'player' wird gegen BEIDE Spieler geprueft (praktisch fuer den
        Einzelspieler-Modus: sowohl P1- als auch P2-Tasten steuern die Figur).
        Mit 'player' ("p1"/"p2") nur gegen dessen Belegung (Mehrspieler).
        """
        players = (player,) if player else ("p1", "p2")
        return any(key == self.controls.get(p, {}).get(action) for p in players)

    def play_sound(self, name):
        """Spielt einen Soundeffekt (respektiert die Sound-Einstellung)."""
        audio.play(name, self.settings)

    def rumble(self, ms=120):
        """Loest Gamepad-Vibration aus (respektiert die Haptik-Einstellung)."""
        audio.rumble(self.settings, ms)