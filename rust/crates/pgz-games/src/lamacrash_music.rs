//! lamacrash_music.rs
//! ==================
//! Spannungsmusik von Lama Crash - uebertragen aus `games/lamacrash_music.py`
//! (Browser: `lamacrash_music.js`): je Hoehenwelt eine Stufe, jede schneller
//! und hoeher als die vorige (0 Wiese .. 4 Weltall, 5 = Fanfare der
//! Mondlandung). Laeuft ueber die Musik-API von `pgz_engine::audio`
//! (hoechstens 4 Stimmen, vorab gerendert, in Dauerschleife) - wie die Musik
//! von Geometry Dash (`geodash_music.rs`).
//!
//! Ein Stueck in sechs Stufen, alle mit der Akkordfolge I - bVI - IV - V (ein
//! Akkord je Takt) und demselben Motiv (fuenf Toene je Takt, der Spitzenton
//! steigt von Takt zu Takt); erst die Fanfare (I - bVI - bVII - I) fuehrt die
//! Linie bis zum Grundton:
//!
//! ```text
//! Stufe  Welt       bpm  Tonart           Takte
//! 0      Wiese       96  A-Dur              4
//! 1      Wolken     108  H-Dur              4
//! 2      Gewitter   120  cis-Moll           4
//! 3      Nordlicht  134  Dis-Dur (maj7)     8
//! 4      Weltall    150  f-Moll (dorisch)   8
//! 5      Mond       112  G-Dur              4
//! ```
//!
//! Vier monophone Stimmen je Stufe (Schlagzeug, Bass, Arpeggio, Melodie).
//! Kein Zufall - alles steht in den Tabellen unten.
//!
//! **Feste Schnittstelle** (wie Python/Web): `STAGES`, `stage_of(m100)`,
//! `bpm(stage)`, `beat_seconds(stage)`, `compose(stage)`, `voices(stage)`,
//! `prerender_step()`, `ready(stage)`, `squeak(m100)`, `clear()`. Nichts hier
//! haengt vom Platzpunkt ab - nur vom aktuellen Multiplikator.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use pgz_engine::{audio, py};

use crate::lamacrash_logic as l;

/// Zahl der Stufen (0..4 Welten, 5 = Mondlandung).
pub const STAGES: usize = 6;
const BPM: [i64; STAGES] = [96, 108, 120, 134, 150, 112];

/// Grundtoene je Takt in Halbtoenen ueber dem Grundton der Stufe:
/// 0 = Thema I - bVI - IV - V (fallender Bass), 1 = Fanfare I - bVI - bVII - I.
const ROOTS: [[i64; 4]; 2] = [[0, -4, -7, -5], [0, -4, -2, 0]];

/// Modus: Akkorde fuers Arpeggio (4 Toene je Takt) und Motiv-Zellen je Takt,
/// alles in Halbtoenen ueber dem Grundton.
struct Mode {
    chords: [[i64; 4]; 4],
    cells: [&'static [i64]; 4],
}

/// `_MODES[name]`
fn mode(name: &str) -> &'static Mode {
    // Dur mit geborgter bVI: C - As - F - G
    const DUR: Mode = Mode {
        chords: [[0, 4, 7, 12], [0, 3, 8, 12], [0, 5, 9, 12], [-1, 2, 7, 11]],
        cells: [&[12, 16, 19, 21, 19], &[12, 15, 20, 22, 20], &[12, 17, 21, 22, 21], &[14, 19, 23, 24, 23]],
    };
    // Moll, die V als verminderter Septakkord (G7b9 ohne Grundton): cm - As - fm - G
    const MOLL: Mode = Mode {
        chords: [[0, 3, 7, 12], [0, 3, 8, 12], [0, 5, 8, 12], [-1, 2, 5, 8]],
        cells: [&[12, 15, 19, 20, 19], &[12, 15, 20, 22, 20], &[12, 17, 20, 22, 20], &[14, 17, 23, 24, 23]],
    };
    // Traum: Cmaj9 - Asmaj7 - Fmaj7 (lydische Quarte) - Gsus2
    const TRAUM: Mode = Mode {
        chords: [[4, 7, 11, 14], [3, 7, 8, 12], [5, 9, 12, 16], [2, 7, 9, 14]],
        cells: [&[12, 16, 19, 21, 19], &[12, 15, 20, 22, 20], &[12, 17, 21, 23, 21], &[14, 19, 23, 24, 23]],
    };
    // Dorisch: cm (mit grosser Sexte) - As - F - G
    const DORISCH: Mode = Mode {
        chords: [[0, 3, 7, 12], [0, 3, 8, 12], [0, 5, 9, 12], [-1, 2, 7, 11]],
        cells: [&[12, 15, 19, 21, 19], &[12, 15, 20, 22, 20], &[12, 17, 21, 22, 21], &[14, 19, 23, 24, 23]],
    };
    // Fanfare: C - As - B - C, die Linie steigt bis zum Grundton
    const FANFARE: Mode = Mode {
        chords: [[0, 4, 7, 12], [0, 3, 8, 12], [2, 5, 10, 14], [4, 7, 12, 16]],
        cells: [&[12, 16, 19, 21, 19], &[12, 15, 20, 22, 20], &[14, 17, 22, 24, 22], &[16, 19, 24]],
    };
    match name {
        "moll" => &MOLL,
        "traum" => &TRAUM,
        "dorisch" => &DORISCH,
        "fanfare" => &FANFARE,
        _ => &DUR,
    }
}

/// `_KIT[zeichen]` - (Welle, Frequenz, Lautstaerke, laengste Dauer in s, decay);
/// ein Schlag klingt hoechstens bis zum naechsten (die Stimme ist monophon).
fn kit(c: u8) -> (&'static str, f64, f64, f64, f64) {
    match c {
        b'K' => ("kick", 50.0, 0.36, 0.22, 0.0),
        b'k' => ("kick", 50.0, 0.24, 0.18, 0.0),
        b'S' => ("snare", 190.0, 0.22, 0.16, 0.0),
        b's' => ("snare", 190.0, 0.12, 0.12, 0.0),
        b'H' => ("hat", 8000.0, 0.09, 0.05, 0.0),
        b'h' => ("hat", 8000.0, 0.05, 0.04, 0.0),
        // Becken / Donner
        _ => ("noise", 1000.0, 0.09, 0.50, 5.0),
    }
}

/// Rhythmus der Motiv-Zelle als (Position, Laenge) in 16teln: gerade Achtel.
const R8: &[(i64, i64)] = &[(0, 2), (2, 2), (4, 6), (10, 2), (12, 4)];
/// Fanfare: punktiert.
const RF: &[(i64, i64)] = &[(0, 3), (3, 1), (4, 6), (10, 2), (12, 4)];
/// Fanfare: Schlusston.
const RF4: &[(i64, i64)] = &[(0, 2), (2, 2), (4, 12)];
/// Tonfolge der Zelle: Thema bzw. Antwort (2. Haelfte der 8-Takt-Stufen).
const ORDER: &[usize] = &[0, 1, 2, 3, 4];
const ANSWER: &[usize] = &[0, 3, 2, 1, 4];

/// 16tel-Pumpbass (Weltall): Grundton, auf jeder zweiten Achtel die Oktave
/// (`(i, 1, 12 if i % 4 == 2 else 0) for i in range(16)`).
const PUMP: &[(i64, i64, i64)] = &[
    (0, 1, 0),
    (1, 1, 0),
    (2, 1, 12),
    (3, 1, 0),
    (4, 1, 0),
    (5, 1, 0),
    (6, 1, 12),
    (7, 1, 0),
    (8, 1, 0),
    (9, 1, 0),
    (10, 1, 12),
    (11, 1, 0),
    (12, 1, 0),
    (13, 1, 0),
    (14, 1, 12),
    (15, 1, 0),
];

/// Bass: je Takt `[(Position, Laenge, Intervall)]`, reihum.
struct Bass {
    wave: &'static str,
    vol: f64,
    decay: f64,
    cap: f64,
    oct: i64,
    pat: &'static [&'static [(i64, i64, i64)]],
}

/// Arpeggio: Akkordton-Indizes im Raster `step` (ab 4 = eine Oktave hoeher),
/// je 4 Takte das naechste Muster.
struct Arp {
    wave: &'static str,
    vol: f64,
    decay: f64,
    cap: f64,
    step: i64,
    pat: &'static [&'static [i64]],
}

/// Melodie: Zellen-Rhythmus je Takt (reihum), Tonfolge je 4 Takte;
/// Hoechstdauer short (Laenge <= 3), long (<= 6), hold (laenger).
struct Lead {
    wave: &'static str,
    vol: f64,
    decay: f64,
    short: f64,
    long: f64,
    hold: f64,
    rhythm: &'static [&'static [(i64, i64)]],
    order: &'static [&'static [usize]],
}

/// Eine Stufe: Grundton (MIDI, Lage des Arpeggios), Modus, Akkordfolge,
/// Takte, Schlagzeug (16 Zeichen je Takt; `drums` reihum, `fill` im letzten
/// Takt), Bass/Arpeggio/Melodie.
struct Plan {
    key: i64,
    mode: &'static str,
    prog: usize,
    bars: i64,
    drums: &'static [&'static str],
    fill: &'static str,
    bass: Bass,
    arp: Arp,
    lead: Lead,
}

// Klangfarben, die mehrere Stufen teilen (`_SOFT_LEAD`, `_HARP`): gleiche
// Welle/Lautstaerke/decay/Hoechstdauer -> beim Ganzton-Wechsel liegt ein Teil
// der Noten schon im Cache.
const SOFT_LEAD: Lead =
    Lead { wave: "triangle", vol: 0.22, decay: 2.5, short: 0.18, long: 0.40, hold: 0.40, rhythm: &[R8], order: &[ORDER] };
const HARP: Arp = Arp { wave: "triangle", vol: 0.16, decay: 4.0, cap: 0.20, step: 1, pat: &[] };

const PLAN: [Plan; STAGES] = [
    // 0 Wiese - ruhig, aber erwartungsvoll: Herzschlag, Pendel in Vierteln
    Plan {
        key: 57,
        mode: "dur",
        prog: 0,
        bars: 4,
        drums: &["k.h...h.k.h...h."],
        fill: "k.h...h.k.h.h.hH",
        bass: Bass { wave: "triangle", vol: 0.34, decay: 3.0, cap: 0.30, oct: -12, pat: &[&[(0, 8, 0), (8, 8, 0)]] },
        arp: Arp { step: 4, pat: &[&[2, 1, 2, 1]], ..HARP },
        lead: SOFT_LEAD,
    },
    // 1 Wolken - leicht, huepfend
    Plan {
        key: 59,
        mode: "dur",
        prog: 0,
        bars: 4,
        drums: &["K.h.s.h.k.h.s.h."],
        fill: "K.h.s.h.k.h.s.ss",
        bass: Bass {
            wave: "triangle",
            vol: 0.30,
            decay: 4.0,
            cap: 0.24,
            oct: -12,
            pat: &[&[(0, 3, 0), (3, 3, 0), (6, 2, 12), (8, 3, 0), (11, 3, 0), (14, 2, 12)]],
        },
        arp: Arp { step: 2, pat: &[&[0, 2, 1, 3, 0, 2, 1, 3]], ..HARP },
        lead: SOFT_LEAD,
    },
    // 2 Gewitter - dunkel: Moll, Galopp, Donner auf der Eins
    Plan {
        key: 61,
        mode: "moll",
        prog: 0,
        bars: 4,
        drums: &["X...S.h.K.hKS.hh", "K.hhS.h.K.hKS.hh", "K.hhS.h.K.hKS.hh", "K.hhS.h.K.hKS.hh"],
        fill: "K.hhS.h.K.hKSSSS",
        bass: Bass {
            wave: "saw",
            vol: 0.22,
            decay: 7.0,
            cap: 0.20,
            oct: -12,
            pat: &[&[
                (0, 2, 0),
                (2, 1, 0),
                (3, 1, 0),
                (4, 2, 0),
                (6, 1, 0),
                (7, 1, 0),
                (8, 2, 0),
                (10, 1, 0),
                (11, 1, 0),
                (12, 2, 0),
                (14, 1, 12),
                (15, 1, 12),
            ]],
        },
        arp: Arp {
            wave: "square",
            vol: 0.07,
            decay: 9.0,
            cap: 0.11,
            step: 1,
            pat: &[&[0, 2, 1, 2, 0, 2, 1, 2, 0, 2, 1, 2, 0, 2, 1, 2]],
        },
        lead: Lead {
            wave: "square",
            vol: 0.12,
            decay: 1.5,
            short: 0.18,
            long: 0.34,
            hold: 0.34,
            rhythm: &[R8],
            order: &[ORDER],
        },
    },
    // 3 Nordlicht - traeumerisch schimmernd
    Plan {
        key: 63,
        mode: "traum",
        prog: 0,
        bars: 8,
        drums: &["X...shHhkhHhshHh", "khHhshHhkhHhshHh", "khHhshHhkhHhshHh", "khHhshHhkhHhshHh"],
        fill: "khHhshHhkhHhsSsS",
        bass: Bass {
            wave: "triangle",
            vol: 0.24,
            decay: 4.0,
            cap: 0.20,
            oct: -12,
            pat: &[&[(0, 2, 0), (2, 2, 0), (4, 2, 12), (6, 2, 0), (8, 2, 0), (10, 2, 0), (12, 2, 12), (14, 2, 0)]],
        },
        arp: Arp {
            wave: "sine",
            vol: 0.13,
            decay: 6.0,
            cap: 0.09,
            step: 1,
            pat: &[&[0, 1, 2, 3, 4, 5, 6, 7, 6, 5, 4, 3, 2, 1, 2, 3], &[7, 6, 5, 4, 3, 2, 1, 0, 1, 2, 3, 4, 5, 6, 5, 4]],
        },
        lead: Lead { order: &[ORDER, ANSWER], ..SOFT_LEAD },
    },
    // 4 Weltall - treibend: Pumpbass und Arpeggio in 16teln
    Plan {
        key: 65,
        mode: "dorisch",
        prog: 0,
        bars: 8,
        drums: &["X...S.hKK.HhS.hK", "K.HhS.hKK.HhS.hK", "K.HhS.hKK.HhS.hK", "K.HhS.hKK.HhS.hK"],
        fill: "K.HhS.hKK.SSSSSS",
        bass: Bass { wave: "saw", vol: 0.24, decay: 10.0, cap: 0.09, oct: -24, pat: &[PUMP] },
        arp: Arp {
            wave: "square",
            vol: 0.10,
            decay: 10.0,
            cap: 0.09,
            step: 1,
            pat: &[&[0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3, 0, 1, 2, 3], &[0, 1, 2, 3, 4, 5, 6, 7, 4, 5, 6, 7, 0, 1, 2, 3]],
        },
        lead: Lead {
            wave: "square",
            vol: 0.13,
            decay: 2.5,
            short: 0.18,
            long: 0.34,
            hold: 0.34,
            rhythm: &[R8],
            order: &[ORDER, ANSWER],
        },
    },
    // 5 Mond - Fanfare (Schleife), die Linie landet auf dem Grundton
    Plan {
        key: 55,
        mode: "fanfare",
        prog: 1,
        bars: 4,
        drums: &["X...S.s.K.s.S.ss", "K.s.S.s.K.s.S.ss", "K.s.S.s.K.s.S.ss", "K.s.S.s.ssssSSSS"],
        fill: "",
        bass: Bass {
            wave: "saw",
            vol: 0.22,
            decay: 3.0,
            cap: 0.26,
            oct: -12,
            pat: &[
                &[(0, 4, 0), (4, 4, 0), (8, 4, 0), (12, 4, 0)],
                &[(0, 4, 0), (4, 4, 0), (8, 4, 0), (12, 4, 0)],
                &[(0, 4, 0), (4, 4, 0), (8, 4, 0), (12, 4, 0)],
                &[(0, 4, 0), (4, 4, 0), (8, 4, 0), (12, 4, -5)],
            ],
        },
        arp: Arp { wave: "square", vol: 0.08, decay: 5.0, cap: 0.14, step: 2, pat: &[&[0, 2, 1, 2, 0, 2, 1, 2]] },
        lead: Lead {
            wave: "square",
            vol: 0.15,
            decay: 1.2,
            short: 0.18,
            long: 0.34,
            hold: 0.8,
            rhythm: &[RF, RF, RF, RF4],
            order: &[ORDER],
        },
    },
];

/// Lautstaerke der vier Stimmen (Schlagzeug, Bass, Arpeggio, Melodie).
const VOICE_VOL: [f64; 4] = [0.8, 0.9, 0.8, 0.9];

/// `_midi(m)` - Frequenz eines MIDI-Tons (69 = a' = 440 Hz), mit Pythons `**`.
fn midi(m: i64) -> f64 {
    440.0 * py::pow(2.0, (m - 69) as f64 / 12.0)
}

/// `_stage(stage)` - auf 0 .. STAGES-1 begrenzt.
fn stage_ix(stage: usize) -> usize {
    stage.min(STAGES - 1)
}

/// `stage_of(m100)` - Musikstufe zum Multiplikator (0..4; Weltgrenzen wie
/// `lamacrash_logic::WORLD_AT`). Die 5 spielt nur die Landung.
pub fn stage_of(m100: i64) -> usize {
    4.min(l::world_of(m100))
}

/// `bpm(stage)` - Tempo der Stufe.
pub fn bpm(stage: usize) -> i64 {
    BPM[stage_ix(stage)]
}

/// `beat_seconds(stage)` - Laenge eines Schlags (Wechsel nur auf dem Schlag).
pub fn beat_seconds(stage: usize) -> f64 {
    60.0 / bpm(stage) as f64
}

/// Eine Note: (start_s, freq, dauer, welle, lautstaerke, decay).
pub type Note = (f64, f64, f64, &'static str, f64, f64);

/// Eine Stimme der Schleife (`{"vol": v, "notes": [...]}`).
#[derive(Clone, Debug, PartialEq)]
pub struct Track {
    pub vol: f64,
    pub notes: Vec<Note>,
}

/// Die Noten einer Stufe (`compose(stage)`).
#[derive(Clone, Debug, PartialEq)]
pub struct Song {
    /// Laenge in Sekunden (ganze Takte).
    pub length: f64,
    pub bars: i64,
    pub bpm: i64,
    /// Schlagzeug, Bass, Arpeggio, Melodie.
    pub voices: Vec<Track>,
}

/// `compose(stage)` - Noten der Schleife einer Stufe, ohne Audio (testbar).
///
/// Vier monophone Stimmen; keine Note reicht in die naechste oder ueber das
/// Schleifenende ("length" = ganze Takte).
pub fn compose(stage: usize) -> Song {
    let s = stage_ix(stage);
    let p = &PLAN[s];
    let six = beat_seconds(s) / 4.0; // 16tel in Sekunden
    let bars = p.bars;
    let key = p.key;
    let roots = ROOTS[p.prog];
    let md = mode(p.mode);

    // Schlagzeug: erst alle Schlaege sammeln, dann klingt jeder bis zum naechsten
    let mut hits: Vec<(i64, u8)> = Vec::new();
    for bar in 0..bars {
        let mut pat = p.drums[bar as usize % p.drums.len()];
        if bar == bars - 1 && !p.fill.is_empty() {
            pat = p.fill;
        }
        for (i, c) in pat.bytes().take(16).enumerate() {
            if c != b'.' {
                hits.push((bar * 16 + i as i64, c));
            }
        }
    }
    let mut drums: Vec<Note> = Vec::with_capacity(hits.len());
    for j in 0..hits.len() {
        let (pos, c) = hits[j];
        let nxt = if j + 1 < hits.len() { hits[j + 1].0 } else { bars * 16 };
        let (wave, freq, vol, cap, decay) = kit(c);
        drums.push((pos as f64 * six, freq, cap.min((nxt - pos) as f64 * six * 0.95), wave, vol, decay));
    }

    let (b, a, m) = (&p.bass, &p.arp, &p.lead);
    let (mut bass, mut arp, mut lead): (Vec<Note>, Vec<Note>, Vec<Note>) = (Vec::new(), Vec::new(), Vec::new());
    for bar in 0..bars {
        let base = bar * 16;
        let half = (bar / 4) as usize;
        let bi = bar as usize;
        let root = key + b.oct + roots[bi % 4];
        for &(pos, ln, iv) in b.pat[bi % b.pat.len()] {
            bass.push((
                (base + pos) as f64 * six,
                midi(root + iv),
                b.cap.min(ln as f64 * six * 0.9),
                b.wave,
                b.vol,
                b.decay,
            ));
        }
        let chord = md.chords[bi % 4];
        let pat = a.pat[half % a.pat.len()];
        let step = a.step;
        for (i, &pi) in pat.iter().enumerate() {
            let tone = key + chord[pi.rem_euclid(4) as usize] + 12 * pi.div_euclid(4);
            arp.push((
                (base + i as i64 * step) as f64 * six,
                midi(tone),
                a.cap.min(step as f64 * six * 0.9),
                a.wave,
                a.vol,
                a.decay,
            ));
        }
        let cell = md.cells[bi % 4];
        let rhythm = m.rhythm[bi % m.rhythm.len()];
        let order = m.order[half % m.order.len()];
        for (j, &(pos, ln)) in rhythm.iter().enumerate() {
            let cap = if ln <= 3 {
                m.short
            } else if ln <= 6 {
                m.long
            } else {
                m.hold
            };
            lead.push(((base + pos) as f64 * six, midi(key + cell[order[j]]), cap.min(ln as f64 * six * 0.9), m.wave, m.vol, m.decay));
        }
    }

    let voices = [drums, bass, arp, lead]
        .into_iter()
        .zip(VOICE_VOL)
        .map(|(notes, vol)| Track { vol, notes })
        .collect();
    Song { length: (bars * 16) as f64 * six, bars, bpm: bpm(s), voices }
}

/// Gerenderte Stufe: (Abtastrate, Stimmen).
type Rendered = (i32, Rc<Vec<audio::Voice>>);

thread_local! {
    /// `_voices` - Stufe -> (Abtastrate, [(Abtastwerte, Lautstaerke), ...]).
    static VOICES: RefCell<HashMap<usize, Rendered>> = RefCell::new(HashMap::new());
}

/// `_render(stage, rate)` - alle Stimmen einer Stufe als Mono-Werte (Noten
/// ueber den Cache von `audio::note`).
fn render(stage: usize) -> Vec<audio::Voice> {
    let song = compose(stage);
    song.voices
        .iter()
        .map(|v| {
            let notes: Vec<(f64, Vec<i16>)> =
                v.notes.iter().map(|&(t0, freq, dur, wave, vol, decay)| (t0, audio::note(freq, dur, wave, vol, decay))).collect();
            (audio::render_voice(&notes, song.length), v.vol)
        })
        .collect()
}

/// `voices(stage)` - gerenderte Stimmen der Stufe (fuer `audio::music_play`)
/// oder `None` (noch nicht gerendert).
pub fn voices(stage: usize) -> Option<Rc<Vec<audio::Voice>>> {
    if !ready(stage) {
        return None;
    }
    VOICES.with(|v| v.borrow().get(&stage).map(|e| e.1.clone()))
}

/// `ready(stage)` - Stufe schon gerendert (mit der Abtastrate des Mixers)?
pub fn ready(stage: usize) -> bool {
    let rate = audio::mixer_rate();
    VOICES.with(|v| v.borrow().get(&stage).is_some_and(|e| e.0 == rate))
}

/// `prerender_step()` - rendert die naechste fehlende Stufe (0, 1, ... 5);
/// `true`, wenn etwas gerendert wurde (je Aufruf genau eine Stufe).
pub fn prerender_step() -> bool {
    let rate = audio::mixer_rate();
    for s in 0..STAGES {
        if !ready(s) {
            let done = Rc::new(render(s));
            VOICES.with(|v| v.borrow_mut().insert(s, (rate, done)));
            return true;
        }
    }
    false
}

/// Quietsch-Schwellen (m100): ab der k-ten klingt der Ton k Halbtoene ueber
/// c'' (523 Hz); dicht im haeufigen Bereich x2..x10, oben weit.
const SQUEAK_AT: [i64; 24] = [
    200, 230, 265, 300, 350, 400, 470, 550, 650, 750, 870, 1000, 1300, 1700, 2200, 3000, 4000, 5500, 7500, 10000, 30000,
    100000, 1000000, 10000000,
];
const SQUEAK_BASE: i64 = 72;

/// `squeak(m100)` - Dehn-Quietschen ab x2: (Frequenz in Hz, auf Hundertstel
/// gerundet; Abstand in s) oder `None`.
pub fn squeak(m100: i64) -> Option<(f64, f64)> {
    if m100 < SQUEAK_AT[0] {
        return None;
    }
    let mut k = 0;
    for (i, v) in SQUEAK_AT.iter().enumerate() {
        if m100 >= *v {
            k = i as i64;
        }
    }
    let top = SQUEAK_AT.len() as i64 - 1;
    let gap_ms = 500 + 2500 * (top - k) / top;
    Some((py::round_to(midi(SQUEAK_BASE + k), 2), gap_ms as f64 / 1000.0))
}

/// `clear()` - gerenderte Stufen vergessen.
pub fn clear() {
    VOICES.with(|v| v.borrow_mut().clear());
}

#[cfg(test)]
mod tests {
    //! Gleichstand mit Python: je Stufe der SHA-256 der Noten (alle Gleitkomma-
    //! zahlen als IEEE-Bits), erzeugt mit
    //! `.venv/Scripts/python.exe rust/tools/lamacrash_music_ref.py`.
    use super::*;
    use pgz_core::fairness::sha256_hex;

    fn bits(v: f64) -> String {
        format!("{:016x}", v.to_bits())
    }

    /// Kanonischer Text einer Stufe (wie `lamacrash_music_ref.py`).
    fn text(song: &Song) -> String {
        let mut out = format!("{}|{}|{}", bits(song.length), song.bars, song.bpm);
        for v in &song.voices {
            out.push_str(&format!("#{}", bits(v.vol)));
            for n in &v.notes {
                out.push_str(&format!(";{},{},{},{},{},{}", bits(n.0), bits(n.1), bits(n.2), n.3, bits(n.4), bits(n.5)));
            }
        }
        out
    }

    #[test]
    fn lamacrash_music_wie_python() {
        let want = include_str!("data/lamacrash_music_ref.txt");
        let mut lines = want.lines();
        for s in 0..STAGES {
            let song = compose(s);
            assert_eq!(sha256_hex(&text(&song)), lines.next().unwrap(), "Stufe {s}");
            // monophon, nichts ueber das Schleifenende hinaus
            for v in &song.voices {
                for w in v.notes.windows(2) {
                    assert!(w[0].0 + w[0].2 <= w[1].0 + 1e-9, "Stufe {s}: Ueberlappung");
                }
                if let Some(last) = v.notes.last() {
                    assert!(last.0 + last.2 <= song.length + 1e-9, "Stufe {s}: ueber das Ende");
                }
            }
        }
        let sq: Vec<String> = (0..=10_500_000i64)
            .step_by(997)
            .chain([199, 200, 229, 230, 10_000_000])
            .map(|m| match squeak(m) {
                Some((f, g)) => format!("{m}:{},{}", bits(f), bits(g)),
                None => format!("{m}:-"),
            })
            .collect();
        assert_eq!(sha256_hex(&sq.join(";")), lines.next().unwrap(), "squeak");
        let st: Vec<String> = [100, 149, 150, 300, 999, 1000, 10000, 10_000_000]
            .iter()
            .map(|m| stage_of(*m).to_string())
            .chain((0..8).map(|s| format!("{}:{}", bpm(s), bits(beat_seconds(s)))))
            .collect();
        assert_eq!(st.join(","), lines.next().unwrap(), "stage_of/bpm");
        assert_eq!(compose(99), compose(5));
    }

    #[test]
    fn lamacrash_music_vorrendern() {
        clear();
        assert!(!ready(0) && voices(0).is_none());
        let mut n = 0;
        while prerender_step() {
            n += 1;
        }
        assert_eq!(n, STAGES);
        for s in 0..STAGES {
            let v = voices(s).unwrap();
            assert_eq!(v.len(), 4);
            let want = (audio::mixer_rate() as f64 * compose(s).length) as usize;
            assert!(v.iter().all(|(data, _)| data.len() == want), "Stufe {s}");
        }
        assert!(voices(STAGES).is_none());
        clear();
        assert!(!ready(5));
    }
}
