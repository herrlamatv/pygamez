//! Abtastrate und Spurenzahl umrechnen - **wie SDL**, nicht ungefaehr.
//!
//! ## Warum das hier steht
//!
//! Hier stand vorher eine lineare Interpolation. Sie war nicht falsch, nur
//! anders: die Laengen stimmten auf den Rahmen genau mit pygame ueberein,
//! die **Werte** nicht. Gefunden wurde das beim Nachmessen von ADPCM - die
//! Byte-Zahl passte, der SHA-256 nicht:
//!
//! ```text
//! house_lo.wav, 22050 -> 44100, Mono -> Stereo
//! rustgame (linear): -3584, -3712, -3840, -3968
//! pygame   (SDL):    -3578, -4150, -4388, -4339
//! gleich: 10.602 von 626.648 Werten (1,7 %)
//! ```
//!
//! Eine gleiche Laenge ist kein gleicher Inhalt.
//!
//! ## Was SDL tut
//!
//! `SDL_BuildAudioCVT` haengt vier Schritte hintereinander, und die
//! **Reihenfolge** gehoert zum Ergebnis:
//!
//! ```text
//! 1. Zahlen nach f32          SDL_Convert_S16_to_F32
//! 2. Spuren anpassen          SDL_ConvertMonoToStereo / ...StereoToMono
//! 3. Abtastrate umrechnen     SDL_ResampleAudio   <- erst jetzt
//! 4. Zahlen nach i16          SDL_Convert_F32_to_S16
//! ```
//!
//! Erst die Spuren, dann die Rate: wer ein Mono-Stueck zuerst umrechnet
//! und danach verdoppelt, bekommt dasselbe Ergebnis - wer aber Stereo auf
//! Mono mischt, muss es **vor** dem Filter tun, sonst filtert er zwei
//! Spuren einzeln und mittelt hinterher.
//!
//! Der Filter selbst ist ein fensterbegrenzter Sinc mit fuenf
//! Nulldurchgaengen je Fluegel und 512 Stuetzstellen dazwischen; zwischen
//! den Stuetzstellen wird linear interpoliert (deshalb die zweite
//! Tabelle). Die Raender sind **Stille** - `SDL_ResampleCVT` legt dort
//! einen genullten Puffer an.
//!
//! ## Die beiden Zahlenumwandlungen sind Bit-Kniffe
//!
//! SDL rechnet nicht `wert / 32768.0`, sondern baut die Gleitkommazahl
//! direkt zusammen. Das ist kein Schoenheitsfehler, den man begradigen
//! darf: `* (1.0 / 32768.0)` und der Kniff geben **verschiedene**
//! Rundungen, und die wandern durch den Filter in jeden Abtastwert.

use crate::filter::{FILTER, FILTERLAENGE, FILTER_UNTERSCHIED, JE_NULLDURCHGANG};

/// `SDL_Convert_S16_to_F32_Scalar` - ein 16-Bit-Wert als Gleitkommazahl.
///
/// ```c
/// x.u32 = (Uint16)src[i] ^ 0x43808000u;
/// dst[i] = x.f32 - 257.0f;
/// ```
///
/// Gebaut wird eine Zahl im Bereich `[256, 258)`, die dann um 257
/// verschoben wird - so entsteht `[-1, 1)` ohne eine einzige Division.
#[inline]
fn nach_f32(v: i16) -> f32 {
    f32::from_bits((v as u16 as u32) ^ 0x4380_8000) - 257.0
}

/// `SDL_Convert_F32_to_S16_Scalar` - und zurueck, mit Begrenzung.
///
/// ```c
/// x.f32 = src[i] + 384.0f;
/// y = x.u32 - 0x43C00000u;
/// z = 0x7FFFu - (y ^ SIGNMASK(y));
/// y = y ^ (z & SIGNMASK(z));
/// dst[i] = (Sint16)(y & 0xFFFF);
/// ```
///
/// `SIGNMASK(x)` ist `0u - (x >> 31)`, also lauter Einsen bei gesetztem
/// Vorzeichenbit. Die drei Zeilen dazwischen sind die Begrenzung auf
/// `[-32768, 32767]` ohne Verzweigung.
#[inline]
fn nach_i16(v: f32) -> i16 {
    let vorzeichen = |x: u32| 0u32.wrapping_sub(x >> 31);
    let x = (v + 384.0).to_bits();
    let mut y = x.wrapping_sub(0x43C0_0000);
    let z = 0x7FFFu32.wrapping_sub(y ^ vorzeichen(y));
    y ^= z & vorzeichen(z);
    (y & 0xFFFF) as u16 as i16
}

/// `ResamplerPadding` - wie weit der Filter ueber den Rand greift.
fn randbreite(quelle: u32, ziel: u32) -> usize {
    if quelle == ziel {
        return 0;
    }
    if quelle > ziel {
        // SDL schreibt `(a * b + c - 1) / c` - dasselbe wie `div_ceil`.
        return (JE_NULLDURCHGANG * quelle as usize).div_ceil(ziel as usize);
    }
    JE_NULLDURCHGANG
}

/// `SDL_ResampleAudio` - der Kern, Zeile fuer Zeile.
///
/// Gerechnet wird bewusst mit **ganzen Zahlen**, wo SDL ganze Zahlen
/// nimmt: `i * inrate / outrate` und `i * inrate % outrate` statt eines
/// Gleitkomma-Schritts. SDLs Kommentar sagt warum - bei langen Stuecken
/// verliert ein aufsummierter Gleitkommaschritt an Genauigkeit, und die
/// Reihenfolge der Divisionen ist Teil des Ergebnisses.
fn filtern(spuren: usize, quelle: u32, ziel: u32, hinein: &[f32]) -> Vec<f32> {
    let rand = randbreite(quelle, ziel);
    let quell_rahmen = hinein.len() / spuren;
    let ziel_rahmen = (quell_rahmen as u64 * ziel as u64 / quelle as u64) as usize;
    let mut heraus = Vec::with_capacity(ziel_rahmen * spuren);
    // Die Raender sind Stille - `SDL_ResampleCVT` legt dafuer einen
    // genullten Puffer an, links wie rechts.
    let stille = 0.0f32;
    let _ = rand;

    for i in 0..ziel_rahmen {
        let quell_index = (i as u64 * quelle as u64 / ziel as u64) as i64;
        let bruch = (i as u64 * quelle as u64 % ziel as u64) as u32;
        let anteil1 = bruch as f32 / ziel as f32;
        let index1 = (bruch as usize * JE_NULLDURCHGANG) / ziel as usize;
        let anteil2 = 1.0 - anteil1;
        let index2 = ((ziel - bruch) as usize * JE_NULLDURCHGANG) / ziel as usize;

        for kanal in 0..spuren {
            let mut wert = 0.0f32;

            // Der linke Fluegel.
            let mut j = 0usize;
            while index1 + j * JE_NULLDURCHGANG < FILTERLAENGE {
                let f = index1 + j * JE_NULLDURCHGANG;
                let rahmen = quell_index - j as i64;
                let ein = if rahmen < 0 {
                    stille
                } else {
                    hinein[rahmen as usize * spuren + kanal]
                };
                wert += ein * (FILTER[f] + anteil1 * FILTER_UNTERSCHIED[f]);
                j += 1;
            }

            // Der rechte Fluegel.
            let mut j = 0usize;
            while index2 + j * JE_NULLDURCHGANG < FILTERLAENGE {
                let f = index2 + j * JE_NULLDURCHGANG;
                let rahmen = quell_index + 1 + j as i64;
                let ein = if rahmen >= quell_rahmen as i64 {
                    stille
                } else {
                    hinein[rahmen as usize * spuren + kanal]
                };
                wert += ein * (FILTER[f] + anteil2 * FILTER_UNTERSCHIED[f]);
                j += 1;
            }

            heraus.push(wert);
        }
    }
    heraus
}

/// Spuren anpassen - `SDL_ConvertMonoToStereo` und `...StereoToMono`.
///
/// Mono nach mehr: jeder Wert wird verdoppelt. Mehrere nach Mono: der
/// Mittelwert, und zwar als `(a * 0.5) + (b * 0.5)`, nicht `(a + b) * 0.5`
/// - bei Gleitkomma ist das nicht dasselbe.
fn spuren_anpassen(hinein: &[f32], von: usize, nach: usize) -> Vec<f32> {
    if von == nach {
        return hinein.to_vec();
    }
    let rahmen = hinein.len() / von;
    let mut heraus = Vec::with_capacity(rahmen * nach);
    for r in 0..rahmen {
        if von == 1 {
            let v = hinein[r];
            for _ in 0..nach {
                heraus.push(v);
            }
        } else if nach == 1 {
            let mut summe = 0.0f32;
            for k in 0..von {
                summe += hinein[r * von + k] * (1.0 / von as f32);
            }
            heraus.push(summe);
        } else {
            for k in 0..nach {
                heraus.push(if k < von { hinein[r * von + k] } else { 0.0 });
            }
        }
    }
    heraus
}

/// Der ganze Weg: 16 Bit hinein, 16 Bit heraus, wie SDL dazwischen.
///
/// Die Rahmenzahl ist **abgeschnitten**, nicht gerundet: SDL rechnet
/// `(Sint64)inframes * outrate / inrate`, und ganzzahlige Division
/// schneidet ab. 1000 Werte bei 48000 Hz ergeben unter 44100 Hz genau 918
/// Rahmen, nicht 919.
pub fn umrechnen(
    werte: &[i16],
    quell_spuren: u16,
    quell_frequenz: u32,
    ziel_spuren: u16,
    ziel_frequenz: u32,
) -> Vec<i16> {
    if quell_spuren == 0 || quell_frequenz == 0 || ziel_spuren == 0 || ziel_frequenz == 0 {
        return Vec::new();
    }
    let von = quell_spuren as usize;
    let nach = ziel_spuren as usize;
    if werte.len() < von {
        return Vec::new();
    }
    // Nichts zu tun? Dann auch nicht durch f32 und zurueck - das waere
    // zwar verlustfrei, kostet aber nur Zeit.
    if von == nach && quell_frequenz == ziel_frequenz {
        return werte.to_vec();
    }

    // 1. Zahlen nach f32.
    let gleit: Vec<f32> = werte.iter().map(|v| nach_f32(*v)).collect();
    // 2. Spuren anpassen - **vor** dem Filter.
    let gleit = spuren_anpassen(&gleit, von, nach);
    // 3. Abtastrate umrechnen.
    let gleit = if quell_frequenz == ziel_frequenz {
        gleit
    } else {
        filtern(nach, quell_frequenz, ziel_frequenz, &gleit)
    };
    // 4. Zahlen nach i16.
    gleit.iter().map(|v| nach_i16(*v)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die beiden Bit-Kniffe treffen die Eckwerte.
    #[test]
    fn zahlenumwandlung_an_den_raendern() {
        assert_eq!(nach_f32(0), 0.0);
        assert_eq!(nach_f32(-32_768), -1.0);
        assert!((nach_f32(32_767) - 0.999_969_5).abs() < 1e-6);
        assert_eq!(nach_i16(0.0), 0);
        assert_eq!(nach_i16(-1.0), -32_768);
        assert_eq!(nach_i16(1.0), 32_767, "wird begrenzt, nicht ueberlaufen");
        assert_eq!(nach_i16(5.0), 32_767, "auch weit darueber");
        assert_eq!(nach_i16(-5.0), -32_768);
    }

    /// Hin und zurueck ohne Umrechnung ist verlustfrei.
    #[test]
    fn hin_und_zurueck_ist_verlustfrei() {
        for v in [-32_768i16, -1000, -1, 0, 1, 1000, 32_767] {
            assert_eq!(nach_i16(nach_f32(v)), v, "{v}");
        }
    }

    /// Gleiche Rate und gleiche Spuren: die Werte bleiben, wie sie sind.
    #[test]
    fn ohne_aenderung_bleibt_alles_stehen() {
        let w = [1i16, -2, 3, -4];
        assert_eq!(umrechnen(&w, 2, 44_100, 2, 44_100), w.to_vec());
    }

    /// Mono nach Stereo verdoppelt, ohne zu filtern.
    #[test]
    fn mono_nach_stereo_verdoppelt() {
        let w = [100i16, -200, 300];
        assert_eq!(
            umrechnen(&w, 1, 44_100, 2, 44_100),
            vec![100, 100, -200, -200, 300, 300]
        );
    }

    /// Stereo nach Mono mittelt - und zwar je Wert mit `0.5`, wie SDL.
    #[test]
    fn stereo_nach_mono_mittelt() {
        let w = [1000i16, 2000, -1000, -3000];
        let aus = umrechnen(&w, 2, 44_100, 1, 44_100);
        assert_eq!(aus.len(), 2);
        assert!((aus[0] - 1500).abs() <= 1, "{:?}", aus);
        assert!((aus[1] + 2000).abs() <= 1, "{:?}", aus);
    }

    /// Die Rahmenzahl wird **abgeschnitten** - gemessen an pygame.
    #[test]
    fn rahmenzahl_wird_abgeschnitten() {
        let w = vec![0i16; 1000];
        let aus = umrechnen(&w, 1, 48_000, 1, 44_100);
        assert_eq!(aus.len(), 918, "nicht 919");
    }

    /// Der Filter greift ueber den Rand - und dort liegt **Stille**.
    ///
    /// Ein einzelner Ausschlag am Anfang darf danach nicht abgeschnitten
    /// wirken: der linke Fluegel liest Nullen, nicht den letzten Wert.
    #[test]
    fn der_rand_ist_stille() {
        let mut w = vec![0i16; 100];
        w[0] = 20_000;
        let aus = umrechnen(&w, 1, 22_050, 1, 44_100);
        assert_eq!(aus.len(), 200);
        // Der Ausschlag bleibt in der Naehe des Anfangs.
        let groesster = aus.iter().enumerate().max_by_key(|(_, v)| v.abs()).unwrap();
        assert!(groesster.0 < 4, "Ausschlag bei {}", groesster.0);
    }
}
