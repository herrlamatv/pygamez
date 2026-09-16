# -*- coding: utf-8 -*-
"""
audio.py
========
Kleine Sound-Engine für die Spielesammlung.

- Die Effekte werden zur Laufzeit synthetisiert (kurze Töne/Rauschen), damit
  KEINE externen WAV-Dateien nötig sind und auch KEIN numpy gebraucht wird:
  wir bauen rohe 16-Bit-PCM-Samples mit dem 'array'-Modul und übergeben sie
  direkt an pygame.mixer.Sound(buffer=...).
- play(name, settings) spielt einen Effekt nur, wenn settings["sound"] aktiv ist,
  und nutzt settings["volume"] als Lautstärke.
- rumble(settings, ms) löst Gamepad-Vibration aus, sofern ein Controller
  vorhanden und settings["haptik"] aktiv ist (sonst wirkungslos).

Alles ist mit try/except abgesichert: fehlt Audio-Hardware oder Mixer, läuft
das Spiel trotzdem (nur eben ohne Ton).
"""

import math
import random
from array import array

import pygame

_available = False          # ist der Mixer nutzbar?
_cache = {}                 # name -> pygame.mixer.Sound
_tone_cache = {}            # (freq, dur, wave) -> pygame.mixer.Sound (frei erzeugt)
_joysticks = []             # initialisierte Gamepads (für Rumble)

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
    "powerup":  dict(f0=600, f1=1200, dur=0.18, wave="square", vol=0.32),
    "level":    dict(f0=440, f1=1100, dur=0.32, wave="square", vol=0.38),
}


def init():
    """Initialisiert Mixer (falls nötig) und erzeugt alle Effekte vorab."""
    global _available
    try:
        if pygame.mixer.get_init() is None:
            pygame.mixer.init(frequency=44100, size=-16, channels=1)
        pygame.mixer.set_num_channels(16)
        _available = pygame.mixer.get_init() is not None
    except Exception:
        _available = False
        return

    # Gamepads für Rumble vorbereiten (optional).
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
    """Synthetisiert einen Effekt und gibt ein pygame.mixer.Sound zurück."""
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
        phase += f / freq_hz             # Phasenakkumulation (für Sweeps korrekt)
        p = phase % 1.0

        if wave == "sine":
            val = math.sin(2 * math.pi * p)
        elif wave == "square":
            val = 1.0 if p < 0.5 else -1.0
        elif wave == "saw":
            val = 2.0 * p - 1.0
        else:  # noise
            val = random.uniform(-1.0, 1.0)

        # Hüllkurve: einblenden, ausklingen.
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


def tone(freq, dur=0.18, settings=None, wave="sine", vol=0.35):
    """Spielt einen frei wählbaren Ton (Frequenz in Hz).

    Anders als play() sind die Effekte nicht vorab definiert: der Ton wird bei
    Bedarf synthetisiert und danach gecacht (nach Frequenz/Dauer/Wellenform).
    Gedacht z.B. für Simon/Senso, das je Feld einen eigenen Klang braucht.
    Respektiert - wie play() - settings["sound"] und settings["volume"].
    """
    if not _available:
        return
    if settings is not None and not settings.get("sound", True):
        return
    key = (round(float(freq), 1), round(float(dur), 3), wave)
    snd = _tone_cache.get(key)
    if snd is None:
        try:
            snd = _build(dict(f0=freq, dur=dur, wave=wave, vol=1.0))
        except Exception:
            snd = None
        _tone_cache[key] = snd
    if snd is None:
        return
    try:
        v = 0.6 if settings is None else float(settings.get("volume", 0.6))
        snd.set_volume(max(0.0, min(1.0, v * vol)))
        snd.play()
    except Exception:
        pass


def rumble(settings=None, ms=120, strong=0.6, weak=0.4):
    """Löst Gamepad-Vibration aus, falls Haptik aktiv und Controller vorhanden."""
    if settings is not None and not settings.get("haptik", False):
        return
    for js in _joysticks:
        try:
            js.rumble(strong, weak, ms)     # verfügbar ab pygame 2 / SDL2
        except Exception:
            pass


# ---------------------------------------------------------------------------
#  Musik-Schleifen (z.B. Level-Soundtrack in Geometry Dash)
# ---------------------------------------------------------------------------
#
# Die Musik besteht aus mehreren STIMMEN (Drums, Bass, Melodie ...), jede ein
# eigener, gleich langer Sound, der auf einem eigenen reservierten Kanal in
# Dauerschleife läuft. So muss Python nichts mischen - das macht SDL.
#
# Eine Stimme ist monophon: sie wird aus Noten zusammengesetzt, deren Samples
# EINMAL synthetisiert (gecacht) und dann per Byte-Slice an ihre Position
# kopiert werden. Das kostet für eine 30-Sekunden-Schleife nur Millisekunden -
# Sample für Sample (wie bei den Effekten) wäre es um Sekunden zu langsam.
#
# Die Musik gehört einem Besitzer (dem Spiel-Objekt). main.py ruft jedes Frame
# music_tick(aktives_spiel) auf: ist der Besitzer nicht mehr aktiv, endet die
# Musik; ist er pausiert, pausiert sie mit.

MUSIC_CHANNELS = 4          # reservierte Kanäle 0..3 (Sound.play() nimmt sie nie)
_music = {"owner": None, "channels": [], "sounds": [], "vols": [],
          "paused": False, "volume": 1.0, "settings": None}
_note_cache = {}            # (freq, samples, wave, vol, decay, rate) -> bytes (mono)


def _reserve_music_channels():
    """Reserviert die Musik-Kanäle (einmalig, nach dem Mixer-Start)."""
    if _music["channels"] or not _available:
        return bool(_music["channels"])
    try:
        if pygame.mixer.get_num_channels() < 16 + MUSIC_CHANNELS:
            pygame.mixer.set_num_channels(16 + MUSIC_CHANNELS)
        pygame.mixer.set_reserved(MUSIC_CHANNELS)
        _music["channels"] = [pygame.mixer.Channel(i) for i in range(MUSIC_CHANNELS)]
    except Exception:
        _music["channels"] = []
    return bool(_music["channels"])


def mixer_rate():
    """Abtastrate des Mixers in Hz (44100, falls kein Mixer läuft)."""
    try:
        init = pygame.mixer.get_init()
        return init[0] if init else 44100
    except Exception:
        return 44100


def note_bytes(freq, dur, wave="square", vol=0.3, decay=0.0, rate=None):
    """Mono-16-Bit-Samples EINER Note als bytes (gecacht).

    freq  : Frequenz in Hz (bei kick/snare/hat ohne Bedeutung)
    dur   : Länge in Sekunden
    wave  : sine / square / saw / triangle / noise / kick / snare / hat
    vol   : Lautstärke 0..1
    decay : 0 = gleichmäßig mit kurzem Ausklang, > 0 = exponentielles
            Abklingen (Faktor pro Sekunde, z.B. 8 für gezupfte Töne)
    """
    rate = rate or mixer_rate()
    n = max(1, int(rate * dur))
    key = (round(float(freq), 2), n, wave, round(float(vol), 3),
           round(float(decay), 2), rate)
    data = _note_cache.get(key)
    if data is not None:
        return data
    buf = array("h")
    amp = max(0.0, min(1.0, vol)) * 32767
    rng = random.Random(int(freq * 100) + n)   # Rauschen reproduzierbar
    att = max(1, int(rate * 0.004))
    rel = max(1, min(n // 3, int(rate * 0.03)))
    phase = 0.0
    step = freq / rate
    for i in range(n):
        tt = i / rate
        if wave == "kick":
            phase += (150.0 * math.exp(-tt * 28.0) + 45.0) / rate
            val = math.sin(2 * math.pi * phase)
            env = math.exp(-tt * 9.0)
        elif wave == "snare":
            phase += 190.0 / rate
            val = 0.65 * rng.uniform(-1.0, 1.0) + 0.35 * math.sin(2 * math.pi * phase)
            env = math.exp(-tt * 18.0)
        elif wave == "hat":
            val = rng.uniform(-1.0, 1.0)
            env = math.exp(-tt * 45.0)
        else:
            phase += step
            p = phase % 1.0
            if wave == "sine":
                val = math.sin(2 * math.pi * p)
            elif wave == "saw":
                val = 2.0 * p - 1.0
            elif wave == "triangle":
                val = 4.0 * p - 1.0 if p < 0.5 else 3.0 - 4.0 * p
            elif wave == "noise":
                val = rng.uniform(-1.0, 1.0)
            else:
                val = 1.0 if p < 0.5 else -1.0
            env = math.exp(-tt * decay) if decay > 0 else 1.0
        env *= min(1.0, i / att, (n - i) / rel)
        buf.append(int(max(-1.0, min(1.0, val)) * amp * env))
    data = buf.tobytes()
    _note_cache[key] = data
    return data


def render_voice(notes, length, rate=None):
    """Baut eine Stimme: notes = [(start_sekunde, bytes), ...] -> Mono-bytes.

    'length' ist die Gesamtlänge in Sekunden. Überlappt eine Note die nächste
    oder das Ende, wird sie abgeschnitten (eine Stimme ist monophon).
    """
    rate = rate or mixer_rate()
    total = int(rate * length) * 2
    out = bytearray(total)
    for start, data in notes:
        pos = int(rate * start) * 2
        if pos < 0 or pos >= total:
            continue
        chunk = data[: total - pos]
        out[pos:pos + len(chunk)] = chunk
    return bytes(out)


def _mono_to_mixer(mono):
    """Mono-16-Bit-bytes -> Format des Mixers (bei Stereo: links = rechts)."""
    try:
        channels = pygame.mixer.get_init()[2]
    except Exception:
        channels = 2
    if channels == 1:
        return mono
    n = len(mono) // 2
    lo, hi = mono[0:n * 2:2], mono[1:n * 2:2]
    out = bytearray(n * 2 * channels)
    for c in range(channels):
        out[c * 2::channels * 2] = lo
        out[c * 2 + 1::channels * 2] = hi
    return bytes(out)


def music_play(voices, owner, settings=None, volume=1.0):
    """Startet eine Musik-Schleife aus mehreren Stimmen (max. MUSIC_CHANNELS).

    voices : Liste von (mono_bytes, relative_lautstärke) - alle gleich lang
    owner  : das Spiel, dem die Musik gehört (siehe music_tick)
    Eine bereits laufende Musik wird vorher beendet. Gibt True bei Erfolg.
    """
    music_stop()
    if not _available or not _reserve_music_channels():
        return False
    try:
        if abs(pygame.mixer.get_init()[1]) != 16:
            return False
    except Exception:
        return False
    sounds, vols = [], []
    for mono, vol in list(voices)[:MUSIC_CHANNELS]:
        try:
            sounds.append(pygame.mixer.Sound(buffer=_mono_to_mixer(mono)))
            vols.append(float(vol))
        except Exception:
            pass
    _music.update(owner=owner, sounds=sounds, vols=vols, paused=False,
                  volume=float(volume), settings=settings)
    _apply_music_volume()
    for ch, snd in zip(_music["channels"], sounds):
        try:
            ch.play(snd, loops=-1)
        except Exception:
            pass
    return bool(sounds)


def music_stop(owner=None):
    """Beendet die Musik (nur die von 'owner', falls angegeben)."""
    if owner is not None and _music["owner"] is not owner:
        return
    for ch in _music["channels"]:
        try:
            ch.stop()
        except Exception:
            pass
    _music.update(owner=None, sounds=[], vols=[], paused=False)


def music_set_volume(volume):
    """Relative Lautstärke der laufenden Musik (0..1, z.B. fürs Ausblenden)."""
    _music["volume"] = max(0.0, min(1.0, float(volume)))
    _apply_music_volume()


def music_playing(owner=None):
    """True, wenn gerade Musik läuft (optional: die von 'owner')."""
    return bool(_music["sounds"]) and (owner is None or _music["owner"] is owner)


def _apply_music_volume():
    settings = _music["settings"]
    base = 0.6
    if settings is not None:
        base = float(settings.get("volume", 0.6)) if settings.get("sound", True) else 0.0
    for ch, vol in zip(_music["channels"], _music["vols"]):
        try:
            ch.set_volume(max(0.0, min(1.0, base * vol * _music["volume"])))
        except Exception:
            pass


def music_tick(current):
    """Wird von main.py jedes Frame aufgerufen: die Musik folgt ihrem Besitzer.

    Ist 'current' nicht der Besitzer, endet die Musik. Pausiert der Besitzer
    (ESC), pausiert die Musik; Sound aus / Lautstärke wirken sofort.
    """
    owner = _music["owner"]
    if owner is None:
        return
    if current is not owner:
        music_stop()
        return
    want_pause = bool(getattr(owner, "paused", False))
    if want_pause != _music["paused"]:
        _music["paused"] = want_pause
        for ch in _music["channels"]:
            try:
                if want_pause:
                    ch.pause()
                else:
                    ch.unpause()
            except Exception:
                pass
    _apply_music_volume()
