# -*- coding: utf-8 -*-
"""
lamacrash_music.py
==================
Spannungsmusik von Lama Crash: je Höhenwelt eine Stufe, jede schneller und
höher als die vorige (0 Wiese … 4 Weltall, 5 = Fanfare der Mondlandung).
Läuft über die Musik-API von ``audio.py`` (höchstens 4 Stimmen, vorab
gerendert, in Dauerschleife).

Ein Stück in sechs Stufen. Alle teilen die Akkordfolge I - bVI - IV - V (ein
Akkord je Takt, der Bass fällt) und das Motiv: eine Zelle aus fünf Tönen je
Takt (Anlauf, Anlauf, Spitzenton, Nachbarton, Spitzenton), deren Spitzenton
von Takt zu Takt steigt (Quinte - kleine Sexte - Sexte - Septime; in Moll
bleibt Takt 3 auf der kleinen Sexte) und am Schleifenende auf die Quinte
zurückfällt. Erst die Fanfare (I - bVI - bVII - I) führt die Linie bis zum
Grundton. Jede Stufe steht einen Ganzton höher, ist schneller und dichter::

    Stufe  Welt       bpm  Tonart           Takte  Satz
    0      Wiese       96  A-Dur              4    Herzschlag, Zupfbass, Pendel in Vierteln
    1      Wolken     108  H-Dur              4    Hüpfbass 3-3-2, Arpeggio in Achteln
    2      Gewitter   120  cis-Moll           4    Donner, Galopp-Bass, Tremolo, verm. V
    3      Nordlicht  134  Dis-Dur (maj7)     8    Glitzer-Arpeggio über zwei Oktaven
    4      Weltall    150  f-Moll (dorisch)   8    16tel-Pumpbass, 16tel-Arpeggio
    5      Mond       112  G-Dur              4    Fanfare, Marschbass, Wirbel

Vier monophone Stimmen je Stufe: Schlagzeug (Kick, Snare, Hi-Hat, Becken in
einer Stimme), Bass, Arpeggio, Melodie. Kein Zufall - alles steht in den
Tabellen unten (Web und Rust übernehmen sie 1:1). Dauern sind gedeckelt und
wiederholen sich, damit ``audio.note_bytes`` (Cache) wenig neu rechnet; Wiese,
Wolken und Nordlicht teilen die Klangfarbe der Melodie, so liegt beim
Ganzton-Wechsel ein Teil der Noten schon bereit. Eine Stufe rendert mit
leerem Cache in unter 0,1 s.

**Feste Schnittstelle** (Web ``lamacrash_music.js`` als ``PG.lamaCrashMusic``,
Rust ``lamacrash_music.rs``) - Signaturen nur nach Absprache ändern:

- ``STAGES = 6``
- ``stage_of(m100)``    -> Stufe 0..4 zum Multiplikator (Weltgrenzen wie
  ``lamacrash_logic.WORLD_AT``); die 5 spielt nur die Landung
- ``bpm(stage)``        -> Tempo der Stufe
- ``beat_seconds(stage)`` -> Länge eines Schlags (Wechsel nur auf dem Schlag)
- ``compose(stage)``    -> Noten der Schleife (testbar, ohne Audio):
  ``{"length": s, "bars": n, "bpm": b, "voices": [{"vol": v, "notes":
  [(start_s, freq, dauer, welle, lautstärke, decay), ...]}, ...]}``
- ``voices(stage)``     -> fertige Stimmen [(bytes, lautstärke)] oder None
  (noch nicht gerendert; eine mit anderer Abtastrate gerenderte Stufe gilt
  als fehlend - Rendern selbst braucht keinen Mixer, dann mit 44100 Hz)
- ``prerender_step()``  -> rendert die nächste fehlende Stufe (0, 1, … 5);
  True, wenn etwas gerendert wurde (das Spiel ruft es in ruhigen Frames auf,
  je Aufruf genau eine Stufe)
- ``ready(stage)``      -> Stufe schon gerendert?
- ``squeak(m100)``      -> (Frequenz, Abstand in s) des Dehn-Quietschens ab x2
  oder None; Frequenz auf Halbtöne gerundet (der Ton-Cache bleibt klein)
- ``clear()``           -> gerenderte Stufen vergessen

Nichts hier hängt vom Platzpunkt ab - nur vom aktuellen Multiplikator.
"""

import audio

from . import lamacrash_logic as L

STAGES = 6
_BPM = (96, 108, 120, 134, 150, 112)
_voices = {}            # Stufe -> (Abtastrate, [(bytes, lautstärke), ...])

# Grundtöne je Takt in Halbtönen über dem Grundton der Stufe:
# 0 = Thema I - bVI - IV - V (fallender Bass), 1 = Fanfare I - bVI - bVII - I
_ROOTS = ((0, -4, -7, -5), (0, -4, -2, 0))

# Modus -> (Akkorde fürs Arpeggio: 4 Töne je Takt, Motiv-Zellen je Takt),
# alles in Halbtönen über dem Grundton. Spitzentöne (3. Ton der Zelle) steigen:
# 19, 20, 21, 23 (Moll 19, 20, 20, 23; Fanfare 19, 20, 22, 24).
_MODES = {
    # Dur mit geborgter bVI: C - As - F - G
    "dur": (((0, 4, 7, 12), (0, 3, 8, 12), (0, 5, 9, 12), (-1, 2, 7, 11)),
            ((12, 16, 19, 21, 19), (12, 15, 20, 22, 20), (12, 17, 21, 22, 21),
             (14, 19, 23, 24, 23))),
    # Moll, die V als verminderter Septakkord (G7b9 ohne Grundton): cm - As - fm - G
    "moll": (((0, 3, 7, 12), (0, 3, 8, 12), (0, 5, 8, 12), (-1, 2, 5, 8)),
             ((12, 15, 19, 20, 19), (12, 15, 20, 22, 20), (12, 17, 20, 22, 20),
              (14, 17, 23, 24, 23))),
    # Traum: Cmaj9 - Asmaj7 - Fmaj7 (lydische Quarte) - Gsus2
    "traum": (((4, 7, 11, 14), (3, 7, 8, 12), (5, 9, 12, 16), (2, 7, 9, 14)),
              ((12, 16, 19, 21, 19), (12, 15, 20, 22, 20), (12, 17, 21, 23, 21),
               (14, 19, 23, 24, 23))),
    # Dorisch: cm (mit großer Sexte) - As - F - G
    "dorisch": (((0, 3, 7, 12), (0, 3, 8, 12), (0, 5, 9, 12), (-1, 2, 7, 11)),
                ((12, 15, 19, 21, 19), (12, 15, 20, 22, 20), (12, 17, 21, 22, 21),
                 (14, 19, 23, 24, 23))),
    # Fanfare: C - As - B - C, die Linie steigt bis zum Grundton (19, 20, 22, 24)
    "fanfare": (((0, 4, 7, 12), (0, 3, 8, 12), (2, 5, 10, 14), (4, 7, 12, 16)),
                ((12, 16, 19, 21, 19), (12, 15, 20, 22, 20), (14, 17, 22, 24, 22),
                 (16, 19, 24))),
}

# Schlagzeug-Zeichen -> (Welle, Frequenz, Lautstärke, längste Dauer in s, decay);
# ein Schlag klingt höchstens bis zum nächsten (die Stimme ist monophon).
_KIT = {
    "K": ("kick", 50.0, 0.36, 0.22, 0.0),
    "k": ("kick", 50.0, 0.24, 0.18, 0.0),
    "S": ("snare", 190.0, 0.22, 0.16, 0.0),
    "s": ("snare", 190.0, 0.12, 0.12, 0.0),
    "H": ("hat", 8000.0, 0.09, 0.05, 0.0),
    "h": ("hat", 8000.0, 0.05, 0.04, 0.0),
    "X": ("noise", 1000.0, 0.09, 0.50, 5.0),     # Becken / Donner
}

# Rhythmus der Motiv-Zelle als (Position, Länge) in 16teln
_R8 = ((0, 2), (2, 2), (4, 6), (10, 2), (12, 4))     # gerade Achtel
_RF = ((0, 3), (3, 1), (4, 6), (10, 2), (12, 4))     # Fanfare: punktiert
_RF4 = ((0, 2), (2, 2), (4, 12))                      # Fanfare: Schlusston
# Tonfolge der Zelle: Thema bzw. Antwort (2. Hälfte der 8-Takt-Stufen)
_ORDER = (0, 1, 2, 3, 4)
_ANSWER = (0, 3, 2, 1, 4)

# 16tel-Pumpbass (Weltall): Grundton, auf jeder zweiten Achtel die Oktave
_PUMP = tuple((i, 1, 12 if i % 4 == 2 else 0) for i in range(16))

# Klangfarben, die mehrere Stufen teilen: gleiche Welle/Lautstärke/decay/
# Höchstdauer -> beim Ganzton-Wechsel liegt ein Teil der Noten schon im Cache.
_SOFT_LEAD = dict(wave="triangle", vol=0.22, decay=2.5, short=0.18, long=0.40, hold=0.40)
_HARP = dict(wave="triangle", vol=0.16, decay=4.0, cap=0.20)


def _with(base, **extra):
    """Kopie von base, ergänzt um extra (für die Tabelle)."""
    out = dict(base)
    out.update(extra)
    return out


# Je Stufe: Grundton (MIDI, Lage des Arpeggios), Modus, Akkordfolge, Takte,
# Schlagzeug (16 Zeichen je Takt; "drums" reihum, "fill" im letzten Takt),
# Bass/Arpeggio/Melodie mit Welle, Lautstärke, decay und Höchstdauern (s).
#   bass.pat  = je Takt [(Position, Länge, Intervall)], reihum
#   arp.pat   = Akkordton-Indizes im Raster "step" (ab 4 = eine Oktave höher),
#               je 4 Takte das nächste Muster
#   lead      = Zellen-Rhythmus je Takt (reihum), Tonfolge je 4 Takte;
#               Höchstdauer short (Länge <= 3), long (<= 6), hold (länger)
_PLAN = (
    # 0 Wiese - ruhig, aber erwartungsvoll: Herzschlag, Pendel in Vierteln
    dict(key=57, mode="dur", prog=0, bars=4,
         drums=("k.h...h.k.h...h.",), fill="k.h...h.k.h.h.hH",
         bass=dict(wave="triangle", vol=0.34, decay=3.0, cap=0.30, oct=-12,
                   pat=(((0, 8, 0), (8, 8, 0)),)),
         arp=_with(_HARP, step=4, pat=((2, 1, 2, 1),)),
         lead=_with(_SOFT_LEAD, rhythm=(_R8,), order=(_ORDER,))),
    # 1 Wolken - leicht, hüpfend
    dict(key=59, mode="dur", prog=0, bars=4,
         drums=("K.h.s.h.k.h.s.h.",), fill="K.h.s.h.k.h.s.ss",
         bass=dict(wave="triangle", vol=0.30, decay=4.0, cap=0.24, oct=-12,
                   pat=(((0, 3, 0), (3, 3, 0), (6, 2, 12), (8, 3, 0), (11, 3, 0),
                         (14, 2, 12)),)),
         arp=_with(_HARP, step=2, pat=((0, 2, 1, 3, 0, 2, 1, 3),)),
         lead=_with(_SOFT_LEAD, rhythm=(_R8,), order=(_ORDER,))),
    # 2 Gewitter - dunkel: Moll, Galopp, Donner auf der Eins
    dict(key=61, mode="moll", prog=0, bars=4,
         drums=("X...S.h.K.hKS.hh", "K.hhS.h.K.hKS.hh", "K.hhS.h.K.hKS.hh",
                "K.hhS.h.K.hKS.hh"),
         fill="K.hhS.h.K.hKSSSS",
         bass=dict(wave="saw", vol=0.22, decay=7.0, cap=0.20, oct=-12,
                   pat=(((0, 2, 0), (2, 1, 0), (3, 1, 0), (4, 2, 0), (6, 1, 0), (7, 1, 0),
                         (8, 2, 0), (10, 1, 0), (11, 1, 0), (12, 2, 0), (14, 1, 12),
                         (15, 1, 12)),)),
         arp=dict(wave="square", vol=0.07, decay=9.0, cap=0.11, step=1,
                  pat=((0, 2, 1, 2, 0, 2, 1, 2, 0, 2, 1, 2, 0, 2, 1, 2),)),
         lead=dict(wave="square", vol=0.12, decay=1.5, short=0.18, long=0.34, hold=0.34,
                   rhythm=(_R8,), order=(_ORDER,))),
    # 3 Nordlicht - träumerisch schimmernd
    dict(key=63, mode="traum", prog=0, bars=8,
         drums=("X...shHhkhHhshHh", "khHhshHhkhHhshHh", "khHhshHhkhHhshHh",
                "khHhshHhkhHhshHh"),
         fill="khHhshHhkhHhsSsS",
         bass=dict(wave="triangle", vol=0.24, decay=4.0, cap=0.20, oct=-12,
                   pat=(((0, 2, 0), (2, 2, 0), (4, 2, 12), (6, 2, 0), (8, 2, 0), (10, 2, 0),
                         (12, 2, 12), (14, 2, 0)),)),
         arp=dict(wave="sine", vol=0.13, decay=6.0, cap=0.09, step=1,
                  pat=((0, 1, 2, 3, 4, 5, 6, 7, 6, 5, 4, 3, 2, 1, 2, 3),
                       (7, 6, 5, 4, 3, 2, 1, 0, 1, 2, 3, 4, 5, 6, 5, 4))),
         lead=_with(_SOFT_LEAD, rhythm=(_R8,), order=(_ORDER, _ANSWER))),
    # 4 Weltall - treibend: Pumpbass und Arpeggio in 16teln
    dict(key=65, mode="dorisch", prog=0, bars=8,
         drums=("X...S.hKK.HhS.hK", "K.HhS.hKK.HhS.hK", "K.HhS.hKK.HhS.hK",
                "K.HhS.hKK.HhS.hK"),
         fill="K.HhS.hKK.SSSSSS",
         bass=dict(wave="saw", vol=0.24, decay=10.0, cap=0.09, oct=-24, pat=(_PUMP,)),
         arp=dict(wave="square", vol=0.10, decay=10.0, cap=0.09, step=1,
                  pat=((0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3),
                       (0, 1, 2, 3, 4, 5, 6, 7, 4, 5, 6, 7, 0, 1, 2, 3))),
         lead=dict(wave="square", vol=0.13, decay=2.5, short=0.18, long=0.34, hold=0.34,
                   rhythm=(_R8,), order=(_ORDER, _ANSWER))),
    # 5 Mond - Fanfare (Schleife), die Linie landet auf dem Grundton
    dict(key=55, mode="fanfare", prog=1, bars=4,
         drums=("X...S.s.K.s.S.ss", "K.s.S.s.K.s.S.ss", "K.s.S.s.K.s.S.ss",
                "K.s.S.s.ssssSSSS"),
         fill="",
         bass=dict(wave="saw", vol=0.22, decay=3.0, cap=0.26, oct=-12,
                   pat=(((0, 4, 0), (4, 4, 0), (8, 4, 0), (12, 4, 0)),
                        ((0, 4, 0), (4, 4, 0), (8, 4, 0), (12, 4, 0)),
                        ((0, 4, 0), (4, 4, 0), (8, 4, 0), (12, 4, 0)),
                        ((0, 4, 0), (4, 4, 0), (8, 4, 0), (12, 4, -5)))),
         arp=dict(wave="square", vol=0.08, decay=5.0, cap=0.14, step=2,
                  pat=((0, 2, 1, 2, 0, 2, 1, 2),)),
         lead=dict(wave="square", vol=0.15, decay=1.2, short=0.18, long=0.34, hold=0.8,
                   rhythm=(_RF, _RF, _RF, _RF4), order=(_ORDER,))),
)

# Lautstärke der vier Stimmen (Schlagzeug, Bass, Arpeggio, Melodie)
_VOICE_VOL = (0.8, 0.9, 0.8, 0.9)


def _midi(m):
    """Frequenz eines MIDI-Tons (69 = a' = 440 Hz)."""
    return 440.0 * 2.0 ** ((m - 69) / 12.0)


def _stage(stage):
    return max(0, min(STAGES - 1, int(stage)))


def stage_of(m100):
    """Musikstufe zum Multiplikator (0..4)."""
    return min(4, L.world_of(m100))


def bpm(stage):
    return _BPM[_stage(stage)]


def beat_seconds(stage):
    return 60.0 / bpm(stage)


def compose(stage):
    """Noten der Schleife einer Stufe - ohne Audio (testbar).

    Vier monophone Stimmen (Schlagzeug, Bass, Arpeggio, Melodie), Noten als
    (start_s, freq, dauer, welle, lautstärke, decay); keine Note reicht in die
    nächste oder über das Schleifenende ("length" = ganze Takte).
    """
    s = _stage(stage)
    p = _PLAN[s]
    six = beat_seconds(s) / 4.0                  # 16tel in Sekunden
    bars = p["bars"]
    key = p["key"]
    roots = _ROOTS[p["prog"]]
    chords, cells = _MODES[p["mode"]]

    # Schlagzeug: erst alle Schläge sammeln, dann klingt jeder bis zum nächsten
    hits = []
    for bar in range(bars):
        pat = p["drums"][bar % len(p["drums"])]
        if bar == bars - 1 and p["fill"]:
            pat = p["fill"]
        for i in range(16):
            if pat[i] != ".":
                hits.append((bar * 16 + i, pat[i]))
    drums = []
    for j in range(len(hits)):
        pos, c = hits[j]
        nxt = hits[j + 1][0] if j + 1 < len(hits) else bars * 16
        wave, freq, vol, cap, decay = _KIT[c]
        drums.append((pos * six, freq, min(cap, (nxt - pos) * six * 0.95), wave, vol, decay))

    b, a, m = p["bass"], p["arp"], p["lead"]
    bass, arp, lead = [], [], []
    for bar in range(bars):
        base = bar * 16
        half = bar // 4
        root = key + b["oct"] + roots[bar % 4]
        for pos, ln, iv in b["pat"][bar % len(b["pat"])]:
            bass.append(((base + pos) * six, _midi(root + iv), min(b["cap"], ln * six * 0.9),
                         b["wave"], b["vol"], b["decay"]))
        chord = chords[bar % 4]
        pat = a["pat"][half % len(a["pat"])]
        step = a["step"]
        for i in range(len(pat)):
            tone = key + chord[pat[i] % 4] + 12 * (pat[i] // 4)
            arp.append(((base + i * step) * six, _midi(tone), min(a["cap"], step * six * 0.9),
                        a["wave"], a["vol"], a["decay"]))
        cell = cells[bar % 4]
        rhythm = m["rhythm"][bar % len(m["rhythm"])]
        order = m["order"][half % len(m["order"])]
        for j in range(len(rhythm)):
            pos, ln = rhythm[j]
            cap = m["short"] if ln <= 3 else (m["long"] if ln <= 6 else m["hold"])
            lead.append(((base + pos) * six, _midi(key + cell[order[j]]),
                         min(cap, ln * six * 0.9), m["wave"], m["vol"], m["decay"]))

    voices_ = [drums, bass, arp, lead]
    return {"length": bars * 16 * six, "bars": bars, "bpm": bpm(s),
            "voices": [{"vol": _VOICE_VOL[i], "notes": voices_[i]} for i in range(4)]}


def _render(stage, rate):
    """Alle Stimmen einer Stufe als Mono-bytes (Noten über den Cache)."""
    song = compose(stage)
    out = []
    for v in song["voices"]:
        notes = [(n[0], audio.note_bytes(n[1], n[2], n[3], n[4], n[5], rate))
                 for n in v["notes"]]
        out.append((audio.render_voice(notes, song["length"], rate), v["vol"]))
    return out


def voices(stage):
    """Gerenderte Stimmen der Stufe [(bytes, lautstärke)] oder None."""
    return _voices[stage][1] if ready(stage) else None


def ready(stage):
    entry = _voices.get(stage)
    return entry is not None and entry[0] == audio.mixer_rate()


def prerender_step():
    """Rendert die nächste fehlende Stufe; True, wenn etwas gerendert wurde."""
    rate = audio.mixer_rate()
    for s in range(STAGES):
        if not ready(s):
            _voices[s] = (rate, _render(s, rate))
            return True
    return False


# Quietsch-Schwellen (m100): ab der k-ten klingt der Ton k Halbtöne über c''
# (523 Hz); dicht im häufigen Bereich x2..x10, oben weit. 24 Töne, Abstand
# 3,0 s (x2) bis 0,5 s (Mond).
_SQUEAK_AT = (200, 230, 265, 300, 350, 400, 470, 550, 650, 750, 870, 1000,
              1300, 1700, 2200, 3000, 4000, 5500, 7500, 10000, 30000, 100000,
              1000000, 10000000)
_SQUEAK_BASE = 72


def squeak(m100):
    """Dehn-Quietschen ab x2: (Frequenz in Hz, Abstand in s) oder None."""
    if m100 < _SQUEAK_AT[0]:
        return None
    k = 0
    for i in range(len(_SQUEAK_AT)):
        if m100 >= _SQUEAK_AT[i]:
            k = i
    top = len(_SQUEAK_AT) - 1
    gap_ms = 500 + 2500 * (top - k) // top
    return round(_midi(_SQUEAK_BASE + k), 2), gap_ms / 1000.0


def clear():
    _voices.clear()
