//! synth.rs
//! ========
//! Siebzehn Klaenge aus einer Handvoll Zahlen - ganz ohne Dateien.
//!
//! **Uebernommen aus `rust/crates/pgz-engine/src/audio.rs`** desselben
//! Projekts (dort MIT, hier LGPL-2.1 - derselbe Autor, also
//! unproblematisch). Dort stehen dieselben Werte wie in `audio.py`, und
//! dort sind sie seit Langem in Gebrauch: Piepen, Klicken, Explosionen
//! fuer 46 Spiele. Uebernommen sind die Bauplaene, die Wellenformen, die
//! Huellkurve und der Rauschgenerator; angepasst ist nur der Ausgang -
//! statt macroquad bekommt hier [`crate::Sound`] die Werte.
//!
//! ## Wozu das in einer Mixer-Kiste steht
//!
//! Damit `cargo test` und `cargo run --example toene` **ohne jede
//! Klangdatei** auskommen. Ein Testlauf, der erst Dateien im Dateisystem
//! sucht, ist bruechig; einer, der sich seine Klaenge ausrechnet, nicht.
//! Und der Nachweis "beide Treiber gehen" braucht etwas zu spielen.
//!
//! ```
//! use rustgame_mixer::{mixer, synth};
//! # mixer::set_treiber(rustgame_mixer::treiber::Wahl::Dummy);
//! let _ = mixer::init(Default::default());
//! let klang = synth::sound("explode").unwrap();
//! assert!(klang.get_length() > 0.2);
//! mixer::quit();
//! ```

use crate::{Ergebnis, Sound};

/// Wellenform eines Effekts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Welle {
    /// Sinus - weich.
    Sinus,
    /// Rechteck - der Klang der alten Spielautomaten.
    Rechteck,
    /// Saegezahn - schneidend.
    Saege,
    /// Rauschen - fuer Explosionen.
    Rauschen,
}

/// Bauplan eines Effekts: Name, Startfrequenz, Endfrequenz, Dauer,
/// Wellenform, Lautstaerke.
///
/// Die Werte stehen genauso in `audio._SPECS` und in
/// `rust/crates/pgz-engine/src/audio.rs` - wer dort etwas aendert,
/// aendert es hier mit.
pub const SPECS: &[(&str, f32, f32, f32, Welle, f32)] = &[
    ("click", 880.0, 880.0, 0.05, Welle::Rechteck, 0.25),
    ("select", 700.0, 700.0, 0.05, Welle::Sinus, 0.25),
    ("eat", 660.0, 1320.0, 0.09, Welle::Rechteck, 0.30),
    ("bounce", 440.0, 440.0, 0.05, Welle::Rechteck, 0.30),
    ("point", 520.0, 1040.0, 0.18, Welle::Rechteck, 0.35),
    ("shoot", 900.0, 300.0, 0.12, Welle::Saege, 0.28),
    ("explode", 200.0, 200.0, 0.22, Welle::Rauschen, 0.40),
    ("hit", 160.0, 160.0, 0.18, Welle::Rechteck, 0.40),
    ("rotate", 600.0, 600.0, 0.04, Welle::Sinus, 0.22),
    ("lock", 200.0, 200.0, 0.07, Welle::Rechteck, 0.30),
    ("line", 400.0, 1200.0, 0.25, Welle::Rechteck, 0.40),
    ("merge", 500.0, 760.0, 0.10, Welle::Sinus, 0.30),
    ("move", 330.0, 330.0, 0.03, Welle::Rechteck, 0.18),
    ("gameover", 440.0, 120.0, 0.50, Welle::Saege, 0.40),
    ("win", 520.0, 1300.0, 0.45, Welle::Rechteck, 0.40),
    ("powerup", 600.0, 1200.0, 0.18, Welle::Rechteck, 0.32),
    ("level", 440.0, 1100.0, 0.32, Welle::Rechteck, 0.38),
];

/// Abtastrate der erzeugten Klaenge.
pub const RATE: u32 = 44_100;

/// Ein einfacher, reproduzierbarer Zufall.
///
/// Uebernommen aus `pgz_core::seedrand` - ein eigener Generator ist hier
/// wichtig, damit `explode` bei jedem Start gleich klingt und ein Test
/// darueber etwas aussagen kann.
struct Rand(u64);

impl Rand {
    fn neu(saat: u64) -> Rand {
        Rand(saat)
    }

    /// Ein Wert von 0.0 bis unter 1.0.
    fn wert(&mut self) -> f32 {
        // Xorshift64 - klein, schnell, gut genug fuer Rauschen.
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        ((self.0 >> 11) as f32) / ((1u64 << 53) as f32)
    }
}

/// Alle Namen, die [`wav`] und [`sound`] kennen.
pub fn namen() -> impl Iterator<Item = &'static str> {
    SPECS.iter().map(|(name, ..)| *name)
}

/// Der Bauplan zu einem Namen.
fn spec(name: &str) -> Option<&'static (&'static str, f32, f32, f32, Welle, f32)> {
    SPECS.iter().find(|(n, ..)| *n == name)
}

/// Erzeugt die Rohdaten eines Effekts als 16-Bit-Mono-Abtastwerte.
///
/// Wortgleich uebernommen aus dem Fundus - nur die Aufzaehlung heisst
/// anders.
pub fn synth(f0: f32, f1: f32, dauer: f32, welle: Welle, lautstaerke: f32) -> Vec<i16> {
    let n = ((RATE as f32 * dauer) as usize).max(1);
    let amp = lautstaerke * 32_767.0;
    // Kurze Ein- und Ausblendbereiche gegen Knackser.
    let att = ((n as f32 * 0.05) as usize).max(1);
    let rel = ((n as f32 * 0.20) as usize).max(1);

    let mut heraus = Vec::with_capacity(n);
    let mut phase = 0.0f32;
    let mut rauschen = Rand::neu(0x5EED);
    for i in 0..n {
        let anteil = i as f32 / n as f32;
        let f = f0 + (f1 - f0) * anteil;
        // Phasenakkumulation - nur so stimmen die Frequenz-Verlaeufe.
        phase += f / RATE as f32;
        let p = phase.fract();
        let wert = match welle {
            Welle::Sinus => (p * std::f32::consts::TAU).sin(),
            Welle::Rechteck => {
                if p < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            Welle::Saege => 2.0 * p - 1.0,
            Welle::Rauschen => rauschen.wert() * 2.0 - 1.0,
        };
        // Huellkurve: einblenden, ausklingen.
        let huelle = (1.0f32)
            .min(i as f32 / att as f32)
            .min((n - i) as f32 / rel as f32);
        heraus.push((wert.clamp(-1.0, 1.0) * amp * huelle) as i16);
    }
    heraus
}

/// Die Abtastwerte eines Effekts - `None`, wenn es ihn nicht gibt.
pub fn werte(name: &str) -> Option<Vec<i16>> {
    let (_, f0, f1, dauer, welle, lautstaerke) = spec(name)?;
    Some(synth(*f0, *f1, *dauer, *welle, *lautstaerke))
}

/// Ein Effekt als fertige WAV-Datei im Speicher.
///
/// Praktisch, um den Leseweg zu pruefen, ohne eine Datei anzulegen:
/// `Sound::from_bytes(&synth::wav("click").unwrap())`.
pub fn wav(name: &str) -> Option<Vec<u8>> {
    Some(crate::wav::schreiben(&werte(name)?, RATE, 1))
}

/// Ein Effekt als fertiger [`Sound`].
///
/// Braucht einen laufenden Mixer, weil der Klang dabei auf dessen
/// Ausgabeformat umgerechnet wird.
pub fn sound(name: &str) -> Ergebnis<Sound> {
    let werte = werte(name).ok_or_else(|| {
        crate::Fehler::Wert(format!("There is no sound named '{name}'"))
    })?;
    Sound::from_samples(&werte, RATE, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jeder_effekt_ergibt_hoerbare_daten() {
        // Derselbe Test wie im Fundus - so faellt auf, wenn beim
        // Uebernehmen etwas verrutscht ist.
        for (name, f0, f1, dauer, welle, lautstaerke) in SPECS {
            let werte = synth(*f0, *f1, *dauer, *welle, *lautstaerke);
            assert_eq!(
                werte.len(),
                (RATE as f32 * dauer) as usize,
                "{name} hat die falsche Laenge"
            );
            let spitze = werte.iter().map(|s| s.unsigned_abs()).max().unwrap_or(0);
            assert!(spitze > 1000, "{name} ist praktisch stumm ({spitze})");
        }
    }

    #[test]
    fn keine_knackser_an_den_raendern() {
        let werte = synth(440.0, 440.0, 0.2, Welle::Rechteck, 0.4);
        assert!(werte[0].abs() < 2000);
        assert!(werte[werte.len() - 1].abs() < 2000);
    }

    #[test]
    fn es_sind_siebzehn_und_alle_verschieden() {
        assert_eq!(SPECS.len(), 17);
        let mut namen: Vec<&str> = namen().collect();
        namen.sort_unstable();
        namen.dedup();
        assert_eq!(namen.len(), 17);
    }

    #[test]
    fn wav_laesst_sich_wieder_lesen() {
        let daten = wav("explode").unwrap();
        let zurueck = crate::wav::lesen(&daten).unwrap();
        assert_eq!(zurueck.frequenz, RATE);
        assert_eq!(zurueck.spuren, 1);
        assert_eq!(zurueck.werte, werte("explode").unwrap());
    }

    #[test]
    fn rauschen_ist_bei_jedem_lauf_gleich() {
        assert_eq!(werte("explode"), werte("explode"));
    }

    #[test]
    fn unbekannter_name_gibt_nichts() {
        assert!(wav("gibtsnicht").is_none());
        assert!(werte("gibtsnicht").is_none());
    }
}
