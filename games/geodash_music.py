# -*- coding: utf-8 -*-
"""
geodash_music.py
================
Prozeduraler Soundtrack für Geometry Dash - je Level ein eigener Loop.

Aus Stil ("drive", "chip", "dream", "dark"), Tempo (bpm) und einem Seed
(der Level-id) entstehen vier Stimmen, die über die Musik-API von audio.py
als Schleife laufen:

    Drums   Kick + Snare (eine Stimme, sie überlappen sich nie)
    Hats    Hi-Hats im Achtel- bzw. Sechzehntel-Raster
    Bass    Grundtöne der Akkordfolge
    Lead    Arpeggio bzw. Melodie aus den Akkordtönen

Der Zufall kommt aus ``seedrand`` - die Browser-Fassung (geodash.js) baut
aus demselben Level also dieselbe Melodie. Die Noten werden über
``audio.note_bytes`` gecacht; ein Loop (8 Takte) ist nach dem ersten Mal in
Millisekunden gebaut.

``beat_pulse(t, bpm)`` liefert den Puls für die Optik (1 auf dem Schlag,
danach abklingend).
"""

import seedrand

# Stil -> (Standard-bpm, Wellenform Bass, Wellenform Lead, Lead-Raster,
#          Hat-Raster, Kick-Muster, Snare-Muster) - Muster in 16teln je Takt.
STYLES = {
    "drive": dict(bpm=128, bass="saw", lead="square", lead_step=2, hat=2,
                  kick=(0, 4, 8, 12), snare=(4, 12), octave=0, decay=9.0),
    "chip": dict(bpm=140, bass="square", lead="square", lead_step=2, hat=1,
                 kick=(0, 6, 8), snare=(4, 12), octave=12, decay=4.0),
    "dream": dict(bpm=120, bass="triangle", lead="triangle", lead_step=2,
                  hat=2, kick=(0, 10), snare=(8,), octave=12, decay=6.0),
    "dark": dict(bpm=174, bass="saw", lead="saw", lead_step=4, hat=1,
                 kick=(0, 10), snare=(4, 12), octave=-12, decay=12.0),
}

# Akkordfolgen als Stufen der natürlichen Molltonleiter.
PROGRESSIONS = ((0, 5, 2, 6), (0, 3, 4, 0), (0, 6, 5, 4), (0, 5, 3, 4))
MINOR = (0, 2, 3, 5, 7, 8, 10)
BARS = 8


def midi_freq(m):
    return 440.0 * 2.0 ** ((m - 69) / 12.0)


def _degree(root, deg):
    """MIDI-Ton einer Tonleiterstufe (auch über die Oktave hinaus)."""
    octv, idx = divmod(deg, 7)
    return root + 12 * octv + MINOR[idx]


def compose(style, bpm, seed_text):
    """Die Noten des Loops - unabhängig von der Audio-Ausgabe (testbar).

    Rückgabe: dict mit Länge in Sekunden und je Stimme einer Liste
    (start_sekunde, freq, dauer, welle, lautstärke, decay).
    """
    st = STYLES.get(style)
    if st is None:
        return None
    bpm = int(bpm or st["bpm"])
    rng = seedrand.Rand(seedrand.seed_from("gdmusic", seed_text, style))
    root = 45 + rng.randint(0, 7)             # A2 .. E3
    prog = PROGRESSIONS[rng.randint(0, len(PROGRESSIONS) - 1)]
    six = 60.0 / bpm / 4.0                    # Sechzehntel in Sekunden
    length = BARS * 16 * six
    drums, hats, bass, lead = [], [], [], []
    for bar in range(BARS):
        chord = prog[bar % len(prog)]
        base = bar * 16
        # Drums: Snare hat Vorrang, im letzten Takt ein kleiner Wirbel
        for s in range(16):
            t0 = (base + s) * six
            if s in st["snare"] or (bar == BARS - 1 and s >= 12 and s % 2 == 0):
                drums.append((t0, 190.0, six * 1.8, "snare", 0.55, 0.0))
            elif s in st["kick"]:
                drums.append((t0, 50.0, six * 2.4, "kick", 0.9, 0.0))
        for s in range(0, 16, st["hat"]):
            if st["hat"] == 2 and s % 4 == 0:
                continue                      # Offbeat-Hats
            vol = 0.22 if s % 4 == 2 else 0.14
            hats.append(((base + s) * six, 8000.0, six * 0.9, "hat", vol, 0.0))
        # Bass: Achtel auf dem Grundton, jede vierte eine Oktave höher
        bass_root = _degree(root, chord)
        for s in range(0, 16, 2):
            m = bass_root + (12 if s in (6, 14) else 0)
            bass.append(((base + s) * six, midi_freq(m), six * 1.7,
                         st["bass"], 0.34, 3.0))
        # Lead: Arpeggio aus Akkordtönen, pro Takt leicht variiert
        tones = [_degree(root + 12 + st["octave"], chord + d)
                 for d in (0, 2, 4, 7)]
        pattern = [0, 1, 2, 3, 2, 1, 2, 3]
        rng.shuffle(pattern)
        step = st["lead_step"]
        for i, s in enumerate(range(0, 16, step)):
            if rng.chance(0.12) and s % 4:
                continue                      # kleine Pausen = Rhythmus
            m = tones[pattern[i % len(pattern)]]
            if rng.chance(0.15):
                m += 12
            lead.append(((base + s) * six, midi_freq(m), six * step * 0.95,
                         st["lead"], 0.2, st["decay"]))
    return {"length": length, "bpm": bpm,
            "voices": [(drums, 0.9), (hats, 0.7), (bass, 0.8), (lead, 0.75)]}


def build_voices(style, bpm, seed_text):
    """Fertige Stimmen für audio.music_play (oder None bei "none")."""
    import audio
    song = compose(style, bpm, seed_text)
    if song is None:
        return None
    out = []
    for notes, vol in song["voices"]:
        rendered = [(t0, audio.note_bytes(f, dur, wave, v, decay))
                    for (t0, f, dur, wave, v, decay) in notes]
        out.append((audio.render_voice(rendered, song["length"]), vol))
    return out


def style_bpm(level):
    """Tempo eines Levels: eigenes bpm oder das des Stils."""
    st = STYLES.get(level.get("music"))
    if st is None:
        return 0
    try:
        bpm = int(level.get("bpm") or 0)
    except (TypeError, ValueError):
        bpm = 0
    return bpm if 60 <= bpm <= 220 else st["bpm"]


def beat_pulse(t, bpm):
    """Puls 0..1 für die Optik: 1 genau auf dem Schlag, danach abklingend."""
    if bpm <= 0 or t < 0:
        return 0.0
    beat = t * bpm / 60.0
    frac = beat - int(beat)
    return max(0.0, 1.0 - frac * 3.0)
