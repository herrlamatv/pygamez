//! geodash_music.rs
//! ================
//! Prozeduraler Soundtrack fuer Geometry Dash - je Level ein eigener Loop.
//! Uebertragen aus `games/geodash_music.py`.
//!
//! Aus Stil, Tempo und der Level-id entstehen vier Stimmen (Drums, Hats,
//! Bass, Lead); der Zufall kommt aus `seedrand`, damit Python, Browser und
//! Rust dieselbe Melodie bauen.

use pgz_core::seedrand;
use pgz_engine::audio;
use serde_json::{Map, Value};

/// Ein Stil.
pub struct Style {
    /// Standard-bpm.
    pub bpm: i64,
    bass: &'static str,
    lead: &'static str,
    lead_step: usize,
    hat: usize,
    kick: &'static [usize],
    snare: &'static [usize],
    octave: i64,
    decay: f64,
}

/// `STYLES[name]`
pub fn style(name: &str) -> Option<&'static Style> {
    const DRIVE: Style = Style { bpm: 128, bass: "saw", lead: "square", lead_step: 2, hat: 2, kick: &[0, 4, 8, 12], snare: &[4, 12], octave: 0, decay: 9.0 };
    const CHIP: Style = Style { bpm: 140, bass: "square", lead: "square", lead_step: 2, hat: 1, kick: &[0, 6, 8], snare: &[4, 12], octave: 12, decay: 4.0 };
    const DREAM: Style = Style { bpm: 120, bass: "triangle", lead: "triangle", lead_step: 2, hat: 2, kick: &[0, 10], snare: &[8], octave: 12, decay: 6.0 };
    const DARK: Style = Style { bpm: 174, bass: "saw", lead: "saw", lead_step: 4, hat: 1, kick: &[0, 10], snare: &[4, 12], octave: -12, decay: 12.0 };
    match name {
        "drive" => Some(&DRIVE),
        "chip" => Some(&CHIP),
        "dream" => Some(&DREAM),
        "dark" => Some(&DARK),
        _ => None,
    }
}

const PROGRESSIONS: [[i64; 4]; 4] = [[0, 5, 2, 6], [0, 3, 4, 0], [0, 6, 5, 4], [0, 5, 3, 4]];
const MINOR: [i64; 7] = [0, 2, 3, 5, 7, 8, 10];
const BARS: usize = 8;

/// `midi_freq(m)`
pub fn midi_freq(m: i64) -> f64 {
    440.0 * 2.0f64.powf((m - 69) as f64 / 12.0)
}

fn degree(root: i64, deg: i64) -> i64 {
    let octv = deg.div_euclid(7);
    let idx = deg.rem_euclid(7);
    root + 12 * octv + MINOR[idx as usize]
}

/// Eine Note: (start, freq, dauer, welle, lautstaerke, decay).
pub type Note = (f64, f64, f64, &'static str, f64, f64);

/// Ein fertiger Loop.
pub struct Song {
    /// Laenge in Sekunden.
    pub length: f64,
    /// Tempo.
    pub bpm: i64,
    /// Stimmen mit Lautstaerke.
    pub voices: Vec<(Vec<Note>, f64)>,
}

/// `compose(style, bpm, seed_text)`
pub fn compose(style_name: &str, bpm: i64, seed_text: &str) -> Option<Song> {
    let st = style(style_name)?;
    let bpm = if bpm != 0 { bpm } else { st.bpm };
    let mut rng = seedrand::Rand::new(seedrand::seed_from(&["gdmusic", seed_text, style_name]));
    let root = 45 + rng.randint(0, 7);
    let prog = PROGRESSIONS[rng.randint(0, PROGRESSIONS.len() as i64 - 1) as usize];
    let six = 60.0 / bpm as f64 / 4.0;
    let length = (BARS * 16) as f64 * six;
    let (mut drums, mut hats, mut bass, mut lead) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for bar in 0..BARS {
        let chord = prog[bar % prog.len()];
        let base = bar * 16;
        for s in 0..16 {
            let t0 = (base + s) as f64 * six;
            if st.snare.contains(&s) || (bar == BARS - 1 && s >= 12 && s % 2 == 0) {
                drums.push((t0, 190.0, six * 1.8, "snare", 0.55, 0.0));
            } else if st.kick.contains(&s) {
                drums.push((t0, 50.0, six * 2.4, "kick", 0.9, 0.0));
            }
        }
        for s in (0..16).step_by(st.hat) {
            if st.hat == 2 && s % 4 == 0 {
                continue;
            }
            let vol = if s % 4 == 2 { 0.22 } else { 0.14 };
            hats.push(((base + s) as f64 * six, 8000.0, six * 0.9, "hat", vol, 0.0));
        }
        let bass_root = degree(root, chord);
        for s in (0..16).step_by(2) {
            let m = bass_root + if s == 6 || s == 14 { 12 } else { 0 };
            bass.push(((base + s) as f64 * six, midi_freq(m), six * 1.7, st.bass, 0.34, 3.0));
        }
        let tones: Vec<i64> = [0, 2, 4, 7].iter().map(|d| degree(root + 12 + st.octave, chord + d)).collect();
        let mut pattern = [0usize, 1, 2, 3, 2, 1, 2, 3];
        rng.shuffle(&mut pattern);
        let step = st.lead_step;
        for (i, s) in (0..16).step_by(step).enumerate() {
            if rng.chance(0.12) && s % 4 != 0 {
                continue;
            }
            let mut m = tones[pattern[i % pattern.len()]];
            if rng.chance(0.15) {
                m += 12;
            }
            lead.push(((base + s) as f64 * six, midi_freq(m), six * step as f64 * 0.95, st.lead, 0.2, st.decay));
        }
    }
    Some(Song { length, bpm, voices: vec![(drums, 0.9), (hats, 0.7), (bass, 0.8), (lead, 0.75)] })
}

/// `build_voices(style, bpm, seed_text)` - fertige Stimmen fuer `audio::music_play`.
pub fn build_voices(style_name: &str, bpm: i64, seed_text: &str) -> Option<Vec<audio::Voice>> {
    let song = compose(style_name, bpm, seed_text)?;
    let mut out = Vec::new();
    for (notes, vol) in &song.voices {
        let rendered: Vec<(f64, Vec<i16>)> = notes.iter().map(|&(t0, f, dur, wave, v, decay)| (t0, audio::note(f, dur, wave, v, decay))).collect();
        out.push((audio::render_voice(&rendered, song.length), *vol));
    }
    Some(out)
}

/// `style_bpm(level)` - eigenes bpm oder das des Stils.
pub fn style_bpm(level: &Map<String, Value>) -> i64 {
    let Some(st) = level.get("music").and_then(|v| v.as_str()).and_then(style) else {
        return 0;
    };
    let bpm = match level.get("bpm") {
        Some(Value::Number(n)) => n.as_i64().or_else(|| n.as_f64().map(|f| f.trunc() as i64)).unwrap_or(0),
        Some(Value::String(s)) => s.trim().parse::<i64>().unwrap_or(0),
        Some(Value::Bool(b)) => *b as i64,
        _ => 0,
    };
    if (60..=220).contains(&bpm) {
        bpm
    } else {
        st.bpm
    }
}

/// `beat_pulse(t, bpm)` - 1 genau auf dem Schlag, danach abklingend.
pub fn beat_pulse(t: f64, bpm: i64) -> f64 {
    if bpm <= 0 || t < 0.0 {
        return 0.0;
    }
    let beat = t * bpm as f64 / 60.0;
    let frac = beat - beat.trunc();
    0.0f64.max(1.0 - frac * 3.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loop_hat_acht_takte() {
        let s = compose("drive", 128, "lama-launch").unwrap();
        assert!((s.length - 8.0 * 16.0 * 60.0 / 128.0 / 4.0).abs() < 1e-9);
        assert_eq!(s.voices.len(), 4);
        assert!(compose("none", 0, "x").is_none());
    }
}
