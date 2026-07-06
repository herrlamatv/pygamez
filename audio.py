# -*- coding: utf-8 -*-
"""
audio.py
========
Kleine Sound-Engine fuer die Spielesammlung.

- Die Effekte werden zur Laufzeit synthetisiert (kurze Toene/Rauschen), damit
  KEINE externen WAV-Dateien noetig sind und auch KEIN numpy gebraucht wird:
  wir bauen rohe 16-Bit-PCM-Samples mit dem 'array'-Modul und uebergeben sie
  direkt an pygame.mixer.Sound(buffer=...).
- play(name, settings) spielt einen Effekt nur, wenn settings["sound"] aktiv ist,
  und nutzt settings["volume"] als Lautstaerke.
- rumble(settings, ms) loest Gamepad-Vibration aus, sofern ein Controller
  vorhanden und settings["haptik"] aktiv ist (sonst wirkungslos).

Alles ist mit try/except abgesichert: fehlt Audio-Hardware oder Mixer, laeuft
das Spiel trotzdem (nur eben ohne Ton).
"""

import math
import random
from array import array

import pygame

_available = False          # ist der Mixer nutzbar?
_cache = {}                 # name -> pygame.mixer.Sound
_joysticks = []             # initialisierte Gamepads (fuer Rumble)

# Spezifikation der Effekte. f0 = Startfrequenz, f1 = Zielfrequenz (Sweep),
# dur = Dauer in Sekunden, wave = Wellenform, vol = relative Amplitude.
_SPECS = {
    "click":    dict(f0=880, dur=0.05, wave="square", vol=0.25),
    "select":   dict(f0=700, dur=0.05, wave="sine",   vol=0.25),
    "eat":      dict(f0=660, f1=1320, dur=0.09, wave="square", vol=0.30),
    "bounce":   dict(f0=440, dur=0.05, wave="square", vol=0.30),
    "point":    dict(f0=520, f1=1040, dur=0.18, wave="square", vol=0.35),
    "shoot":    dict(f0=900, f1=300,  dur=0.12, wave="saw",    vol=0.28),
    "explode":  dict(f0=200, dur=0.22, wave="noise",  vol=0.40),
    "hit":      dict(f0=160, dur=0.18, wave="square", vol=0.40),
    "rotate":   dict(f0=600, dur=0.04, wave="sine",   vol=0.22),
    "lock":     dict(f0=200, dur=0.07, wave="square", vol=0.30),
    "line":     dict(f0=400, f1=1200, dur=0.25, wave="square", vol=0.40),
    "merge":    dict(f0=500, f1=760,  dur=0.10, wave="sine",   vol=0.30),
    "move":     dict(f0=330, dur=0.03, wave="square", vol=0.18),
    "gameover": dict(f0=440, f1=120,  dur=0.50, wave="saw",    vol=0.40),
    "win":      dict(f0=520, f1=1300, dur=0.45, wave="square", vol=0.40),
}


def init():
    """Initialisiert Mixer (falls noetig) und erzeugt alle Effekte vorab."""
    global _available
    try:
        if pygame.mixer.get_init() is None:
            pygame.mixer.init(frequency=44100, size=-16, channels=1)
        pygame.mixer.set_num_channels(16)
        _available = pygame.mixer.get_init() is not None
    except Exception:
        _available = False
        return

    # Gamepads fuer Rumble vorbereiten (optional).
    try:
        pygame.joystick.init()
        for i in range(pygame.joystick.get_count()):
            js = pygame.joystick.Joystick(i)
            js.init()
            _joysticks.append(js)
    except Exception:
        pass

    for name in _SPECS:
        try:
            _cache[name] = _build(_SPECS[name])
        except Exception:
            _cache[name] = None


def _build(spec):
    """Synthetisiert einen Effekt und gibt ein pygame.mixer.Sound zurueck."""
    freq_hz, fmt, channels = pygame.mixer.get_init()
    n = max(1, int(freq_hz * spec["dur"]))
    f0 = spec["f0"]
    f1 = spec.get("f1", f0)
    wave = spec["wave"]
    amp = spec["vol"] * 32767

    att = max(1, int(n * 0.05))          # kurzer Ein-/Ausblendbereich gegen Knackser
    rel = max(1, int(n * 0.20))

    buf = array("h")
    phase = 0.0
    for i in range(n):
        frac = i / n
        f = f0 + (f1 - f0) * frac
        phase += f / freq_hz             # Phasenakkumulation (fuer Sweeps korrekt)
        p = phase % 1.0

        if wave == "sine":
            val = math.sin(2 * math.pi * p)
        elif wave == "square":
            val = 1.0 if p < 0.5 else -1.0
        elif wave == "saw":
            val = 2.0 * p - 1.0
        else:  # noise
            val = random.uniform(-1.0, 1.0)

        # Huellkurve: einblenden, ausklingen.
        env = min(1.0, i / att, (n - i) / rel)
        s = int(max(-1.0, min(1.0, val)) * amp * env)

        buf.append(s)
        if channels == 2:                # bei Stereo denselben Wert auf L und R
            buf.append(s)

    return pygame.mixer.Sound(buffer=buf.tobytes())


def play(name, settings=None):
    """Spielt Effekt 'name', falls Sound aktiv ist."""
    if not _available:
        return
    if settings is not None and not settings.get("sound", True):
        return
    snd = _cache.get(name)
    if snd is None:
        return
    try:
        vol = 0.6 if settings is None else float(settings.get("volume", 0.6))
        snd.set_volume(max(0.0, min(1.0, vol)))
        snd.play()
    except Exception:
        pass


def rumble(settings=None, ms=120, strong=0.6, weak=0.4):
    """Loest Gamepad-Vibration aus, falls Haptik aktiv und Controller vorhanden."""
    if settings is not None and not settings.get("haptik", False):
        return
    for js in _joysticks:
        try:
            js.rumble(strong, weak, ms)     # verfuegbar ab pygame 2 / SDL2
        except Exception:
            pass
