//! wav.rs
//! ======
//! WAV lesen und schreiben - der eine Weg, der immer da sein muss.
//!
//! Der **Schreibteil** stammt aus `rust/crates/pgz-engine/src/audio.rs`
//! desselben Projekts (dort MIT, hier LGPL-2.1 - derselbe Autor, also
//! unproblematisch) und ist dort seit Langem in Gebrauch und geprueft. Er
//! ist hier nur um Abtastrate und Spurenzahl erweitert, die dort fest
//! verdrahtet waren.
//!
//! Der **Leseteil** ist neu. Er kann, was pygame ueber SDL kann und was in
//! freier Wildbahn vorkommt:
//!
//! | `wFormatTag` | Bits | gelesen als |
//! |---|---|---|
//! | 1 (PCM) | 8 | vorzeichenlos, Mitte bei 128 |
//! | 1 (PCM) | 16, 24, 32 | vorzeichenbehaftet, little endian |
//! | 3 (IEEE float) | 32, 64 | -1.0 bis 1.0 |
//! | 0xFFFE (extensible) | wie oben | ueber die ersten zwei Bytes der GUID |
//!
//! Alles landet als **16-Bit-Ganzzahl** im Speicher, weil das die
//! Mischform des Mixers ist - siehe [`crate::sound`].
//!
//! | 2 (MS-ADPCM), 0x11 (IMA-ADPCM) | 4 | ueber die Bloecke, siehe unten |
//! | 6 (A-law), 7 (mu-law) | 8 | G.711, wie SDL |
//!
//! Gelesen wird auf **zwei** Wegen, weil pygame zwei Leser hat - siehe
//! [`Weg`]. Die beiden unterscheiden sich in den Fehlersaetzen und darin,
//! was sie annehmen (`Sound` lehnt 64-Bit-Float ab, `music` nicht).

use crate::{Ergebnis, Fehler};

/// Ein gelesener Klang: Abtastwerte, verschraenkt, dazu Rate und Spuren.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Klangdaten {
    /// Die Abtastwerte, Spur fuer Spur verschraenkt (L, R, L, R, ...).
    pub werte: Vec<i16>,
    /// Wie viele Spuren verschraenkt sind: 1 = Mono, 2 = Stereo.
    pub spuren: u16,
    /// Abtastrate in Hertz.
    pub frequenz: u32,
}

impl Klangdaten {
    /// Wie viele Rahmen (also Abtastwerte je Spur) das sind.
    pub fn rahmen(&self) -> usize {
        if self.spuren == 0 {
            0
        } else {
            self.werte.len() / self.spuren as usize
        }
    }
}

// ---------------------------------------------------------------------------
//  Lesen
// ---------------------------------------------------------------------------

/// Liest die ersten vier Bytes - so erkennt man, was man vor sich hat.
fn marke(daten: &[u8], ab: usize) -> Option<[u8; 4]> {
    daten.get(ab..ab + 4)?.try_into().ok()
}

fn u16_le(d: &[u8], ab: usize) -> Ergebnis<u16> {
    d.get(ab..ab + 2)
        .map(|s| u16::from_le_bytes([s[0], s[1]]))
        .ok_or_else(|| Fehler::Format("Could not read RIFF header".into()))
}

fn u32_le(d: &[u8], ab: usize) -> Ergebnis<u32> {
    d.get(ab..ab + 4)
        .map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
        .ok_or_else(|| Fehler::Format("Could not read RIFF header".into()))
}

/// Welcher der **zwei** WAV-Leser von pygame gemeint ist.
///
/// `mixer.Sound` liest ueber SDLs `SDL_LoadWAV_RW` (`SDL_wave.c`),
/// `mixer.music` ueber SDL_mixers eigenes `music_wav.c`. Beide pruefen in
/// anderer Reihenfolge und sagen andere Saetze - gemessen an pygame-ce
/// 2.5.7 mit derselben kaputten Datei auf beiden Wegen:
///
/// | Fall | `Sound` | `music` |
/// |---|---|---|
/// | kein `fmt ` | `Missing fmt chunk in WAVE file` | `Bad WAV file (no FMT chunk)` |
/// | kein `data` | `Missing data chunk in WAVE file` | `Bad WAV file (no DATA chunk)` |
/// | `data` vor `fmt ` | `fmt chunk after data chunk in WAVE file` | geht |
/// | `fmt ` unter 16 Bytes | `Invalid WAVE fmt chunk length (too small)` | `Wave format chunk too small` |
/// | 0 Spuren | `Invalid number of channels` | `Parameter 'src_channels' is invalid` |
/// | 300 Spuren | `Number of channels exceeds limit of 255` | `Invalid source channels` |
/// | Rate 0 | `Invalid sample rate` | `Source rate is equal to or less than zero` |
/// | PCM mit 0 Bit | `Invalid bits per sample` | `Unknown PCM format with 0 bits` |
/// | PCM mit 13 Bit | `13-bit PCM format not supported` | `Unknown PCM format with 13 bits` |
/// | Float mit 64 Bit | `64-bit IEEE floating-point format not supported` | geht |
/// | Kennung 0xABCD | `Unknown WAVE format tag: 0xabcd` | `Unknown WAVE data format` |
///
/// Hier stand **ein** Leser mit deutschen Saetzen (`WAV ohne fmt-Block`),
/// und der Waechter fuer Fehlertexte sah sie nicht - "ohne" stand nicht in
/// seiner Wortliste.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weg {
    /// `mixer.Sound` - `SDL_wave.c`.
    Klang,
    /// `mixer.music` - `music_wav.c`.
    Musik,
}

/// Liest eine WAV-Datei aus dem Speicher, wie `mixer.Sound` es tut.
pub fn lesen(daten: &[u8]) -> Ergebnis<Klangdaten> {
    lesen_als(daten, Weg::Klang)
}

/// Liest eine WAV-Datei auf dem Weg `weg` - siehe [`Weg`].
pub fn lesen_als(daten: &[u8], weg: Weg) -> Ergebnis<Klangdaten> {
    let satz = |s: &str| Fehler::Format(s.into());

    // ----- Der Kopf ---------------------------------------------------
    //
    // `music` erkennt das Format an zwoelf Bytes (`Mix_LoadMUS_RW`), und
    // nur `RIFF....WAVE` ist dort eine WAV-Datei. `SDL_LoadWAV_RW` liest
    // erst den RIFF-Kopf, dann den Formtyp - und nimmt auch eine Datei,
    // die gleich mit `WAVE` anfaengt (`WaveLoad`: "RIFF chunk is missing,
    // expecting WAVE").
    let mut ab = match weg {
        Weg::Musik => {
            if daten.len() < 12 {
                return Err(satz("Couldn't read first 12 bytes of audio data"));
            }
            if marke(daten, 0) != Some(*b"RIFF") || marke(daten, 8) != Some(*b"WAVE") {
                return Err(satz("Unrecognized audio format"));
            }
            12
        }
        Weg::Klang => match marke(daten, 0) {
            Some(m) if &m == b"RIFF" => {
                if daten.len() < 8 {
                    return Err(satz("Could not read RIFF header"));
                }
                if daten.len() < 12 {
                    return Err(satz("Could not read RIFF form type"));
                }
                if marke(daten, 8) != Some(*b"WAVE") {
                    return Err(satz("RIFF form type is not WAVE (not a Waveform file)"));
                }
                12
            }
            Some(m) if &m == b"WAVE" => 4,
            None => return Err(satz("Could not read RIFF header")),
            Some(_) => {
                return Err(satz(
                    "Could not find RIFF or WAVE identifiers (not a Waveform file)",
                ))
            }
        },
    };

    // ----- Die Bloecke ------------------------------------------------
    //
    // Die angegebene RIFF-Laenge wird bewusst nicht geglaubt: zu viele
    // Dateien in freier Wildbahn haben sie falsch, und der Puffer selbst
    // weiss es besser.
    //
    // `music_wav.c` wertet den `fmt `-Block **beim Lesen** aus - ein
    // kaputter meldet sich also vor einem fehlenden `data`-Block.
    // `SDL_wave.c` sucht erst beide und prueft danach.
    let mut fmt_roh: Option<&[u8]> = None;
    let mut fmt_musik: Option<Fmt> = None;
    let mut roh: Option<&[u8]> = None;
    let mut data_vor_fmt = false;

    while ab + 8 <= daten.len() {
        let kennung = marke(daten, ab).unwrap_or([0; 4]);
        let laenge = u32_le(daten, ab + 4)? as usize;
        let anfang = ab + 8;
        // Ein Block, der ueber das Ende hinausragt, wird bis zum Ende
        // gelesen statt abgelehnt - abgeschnittene WAVs sind haeufig.
        let ende = anfang.saturating_add(laenge).min(daten.len());
        match &kennung {
            b"fmt " if fmt_roh.is_none() => {
                fmt_roh = Some(&daten[anfang..ende]);
                if weg == Weg::Musik {
                    fmt_musik = Some(Fmt::lesen(&daten[anfang..ende], weg)?);
                }
            }
            b"data" if roh.is_none() => {
                // `SDL_wave.c` nimmt nur einen `data`-Block **hinter**
                // dem `fmt `-Block; `music_wav.c` nimmt jeden.
                if weg == Weg::Klang && fmt_roh.is_none() {
                    data_vor_fmt = true;
                } else {
                    roh = Some(&daten[anfang..ende]);
                }
            }
            _ => {}
        }
        // RIFF-Bloecke sind auf gerade Laengen aufgefuellt.
        ab = anfang + laenge + (laenge & 1);
        if laenge == 0 && kennung == [0; 4] {
            break;
        }
    }

    let fmt = match weg {
        Weg::Klang => {
            let Some(f) = fmt_roh else {
                return Err(satz("Missing fmt chunk in WAVE file"));
            };
            if roh.is_none() {
                return Err(satz(if data_vor_fmt {
                    "fmt chunk after data chunk in WAVE file"
                } else {
                    "Missing data chunk in WAVE file"
                }));
            }
            Fmt::lesen(f, weg)?
        }
        Weg::Musik => {
            let Some(f) = fmt_musik else {
                return Err(satz("Bad WAV file (no FMT chunk)"));
            };
            if roh.is_none() {
                return Err(satz("Bad WAV file (no DATA chunk)"));
            }
            // Spuren und Rate prueft erst der Umrechner, den SDL_mixer
            // danach baut (`SDL_NewAudioStream`) - also **nach** den
            // Bloecken.
            if f.spuren == 0 {
                return Err(satz("Parameter 'src_channels' is invalid"));
            }
            if f.spuren > 255 {
                return Err(satz("Invalid source channels"));
            }
            if f.frequenz == 0 {
                return Err(satz("Source rate is equal to or less than zero"));
            }
            f
        }
    };
    let roh = roh.unwrap_or_default();
    let werte = fmt.entpacken(roh)?;
    Ok(Klangdaten {
        werte,
        spuren: fmt.spuren,
        frequenz: fmt.frequenz,
    })
}

/// Der `fmt `-Block, so weit er hier gebraucht wird.
struct Fmt {
    art: u16,
    spuren: u16,
    frequenz: u32,
    bits: u16,
    /// `nBlockAlign` - bei ADPCM die Laenge eines Blocks in Bytes.
    blockgroesse: u16,
    /// Was hinter den 18 Kopfbytes steht. Bei MS-ADPCM stecken dort die
    /// Vorhersage-Beiwerte, bei IMA die Werte je Block.
    zusatz: Vec<u8>,
}

impl Fmt {
    /// Liest den Block und prueft ihn in der Reihenfolge des Weges.
    fn lesen(d: &[u8], weg: Weg) -> Ergebnis<Fmt> {
        let satz = |s: String| Fehler::Format(s);
        if d.len() < 16 {
            return Err(satz(match weg {
                Weg::Klang => "Invalid WAVE fmt chunk length (too small)".into(),
                Weg::Musik => "Wave format chunk too small".into(),
            }));
        }
        let mut art = u16_le(d, 0)?;
        let spuren = u16_le(d, 2)?;
        let frequenz = u32_le(d, 4)?;
        let bits = u16_le(d, 14)?;
        let blockgroesse = u16_le(d, 12)?;
        let zusatz = if d.len() > 18 { d[18..].to_vec() } else { Vec::new() };

        // WAVE_FORMAT_EXTENSIBLE: die wirkliche Art steht in den ersten
        // zwei Bytes der Untergattungs-GUID.
        if art == 0xFFFE && d.len() >= 26 {
            art = u16_le(d, 24)?;
        }
        match weg {
            // `WaveCheckFormat`: erst Spuren und Rate, dann die Bits, dann
            // die Kennung.
            Weg::Klang => {
                if spuren == 0 {
                    return Err(satz("Invalid number of channels".into()));
                }
                if spuren > 255 {
                    return Err(satz("Number of channels exceeds limit of 255".into()));
                }
                if frequenz == 0 {
                    return Err(satz("Invalid sample rate".into()));
                }
                let bekannt = matches!(art, 1 | 2 | 3 | 6 | 7 | 0x11);
                if bekannt && bits == 0 && matches!(art, 1 | 3 | 6 | 7) {
                    return Err(satz("Invalid bits per sample".into()));
                }
                match art {
                    1 if !matches!(bits, 8 | 16 | 24 | 32) => {
                        return Err(satz(format!("{bits}-bit PCM format not supported")));
                    }
                    3 if bits != 32 => {
                        return Err(satz(format!(
                            "{bits}-bit IEEE floating-point format not supported"
                        )));
                    }
                    _ if !bekannt => {
                        return Err(satz(format!("Unknown WAVE format tag: 0x{art:04x}")));
                    }
                    _ => {}
                }
            }
            // `ParseFMT`: nur die Kennung und die Bits - Spuren und Rate
            // erst spaeter, siehe `lesen_als`.
            Weg::Musik => match art {
                1 if !matches!(bits, 8 | 16 | 24 | 32) => {
                    return Err(satz(format!("Unknown PCM format with {bits} bits")));
                }
                3 if !matches!(bits, 32 | 64) => {
                    return Err(satz(format!("Unknown PCM format with {bits} bits")));
                }
                1 | 2 | 3 | 6 | 7 | 0x11 => {}
                _ => return Err(satz("Unknown WAVE data format".into())),
            },
        }
        Ok(Fmt {
            art,
            spuren,
            frequenz,
            bits,
            blockgroesse,
            zusatz,
        })
    }

    /// Macht aus den Rohbytes 16-Bit-Werte.
    fn entpacken(&self, roh: &[u8]) -> Ergebnis<Vec<i16>> {
        match (self.art, self.bits) {
            (1, 8) => Ok(roh
                .iter()
                // Vorzeichenlos mit Mitte bei 128: erst mitteln, dann
                // spreizen. 0 wird -32768, 255 wird 32512 - genau wie SDL.
                .map(|&b| ((b as i32 - 128) * 256) as i16)
                .collect()),
            (1, 16) => Ok(roh
                .as_chunks::<2>().0.iter()
                .map(|c| i16::from_le_bytes([c[0], c[1]]))
                .collect()),
            (1, 24) => Ok(roh
                .as_chunks::<3>().0.iter()
                // Die oberen 16 Bit reichen; das unterste Byte faellt weg.
                .map(|c| i16::from_le_bytes([c[1], c[2]]))
                .collect()),
            (1, 32) => Ok(roh
                .as_chunks::<4>().0.iter()
                .map(|c| (i32::from_le_bytes([c[0], c[1], c[2], c[3]]) >> 16) as i16)
                .collect()),
            (3, 32) => Ok(roh
                .as_chunks::<4>().0.iter()
                .map(|c| nach_i16(f32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f64))
                .collect()),
            (3, 64) => Ok(roh
                .as_chunks::<8>().0.iter()
                .map(|c| {
                    nach_i16(f64::from_le_bytes([
                        c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7],
                    ]))
                })
                .collect()),
            // **A-law und mu-law - G.711, wie SDL es dekodiert.** Hier
            // stand "nicht unterstuetzt"; SDL kann beide auf beiden Wegen.
            // Gemessen an pygame: alle 256 Bytewerte geben genau die
            // Werte der G.711-Formeln in `alaw_wert` und `mulaw_wert`.
            (6, 8) => Ok(roh.iter().map(|&b| alaw_wert(b)).collect()),
            (7, 8) => Ok(roh.iter().map(|&b| mulaw_wert(b)).collect()),
            (2, _) => self.ms_adpcm(roh),
            (0x11, _) => self.ima_adpcm(roh),
            _ => Err(Fehler::Format("Unknown WAVE data format".into())),
        }
    }

    /// **MS-ADPCM** (Art 2) - Microsofts Spielart mit Vorhersage.
    ///
    /// Jeder Block faengt mit einem Kopf je Spur an: ein Byte
    /// Beiwert-Nummer, dann die Schrittweite und die **zwei** letzten
    /// Werte, je 16-bittig. Danach kommen Viererbits, hoehere zuerst.
    ///
    /// ```text
    /// vorhersage = (s1 * a1 + s2 * a2) / 256 + vorzeichen(nibble) * schritt
    /// schritt    = (Anpassung[nibble] * schritt) / 256, mindestens 16
    /// ```
    ///
    /// Gemessen an `examples/data/secosmic_lo.wav`: mono, 11025 Hz,
    /// Bloecke zu 256 Bytes, 500 Werte je Block, 72 Bloecke - macht 36000
    /// Werte und 3.2653 Sekunden, genau wie pygame meldet.
    fn ms_adpcm(&self, roh: &[u8]) -> Ergebnis<Vec<i16>> {
        let spuren = self.spuren as usize;
        let block = self.blockgroesse as usize;
        if block < 7 * spuren {
            return Err(Fehler::Format("MS-ADPCM: blocks too small".into()));
        }
        // Die Beiwerte aus der Datei, sonst die Vorgabe. Im Zusatzteil
        // stehen zuerst zwei Bytes Werte je Block, dann die Anzahl der
        // Beiwerte, dann die Paare.
        let mut beiwerte: Vec<(i32, i32)> = MS_BEIWERTE.to_vec();
        if self.zusatz.len() >= 4 {
            let n = u16::from_le_bytes([self.zusatz[2], self.zusatz[3]]) as usize;
            if n > 0 && self.zusatz.len() >= 4 + n * 4 {
                beiwerte = (0..n)
                    .map(|i| {
                        let a = 4 + i * 4;
                        (
                            i16::from_le_bytes([self.zusatz[a], self.zusatz[a + 1]]) as i32,
                            i16::from_le_bytes([self.zusatz[a + 2], self.zusatz[a + 3]]) as i32,
                        )
                    })
                    .collect();
            }
        }

        let mut aus = Vec::new();
        for teil in roh.chunks(block) {
            if teil.len() < 7 * spuren {
                break;
            }
            let mut nummer = vec![0usize; spuren];
            let mut schritt = vec![0i32; spuren];
            let mut s1 = vec![0i32; spuren];
            let mut s2 = vec![0i32; spuren];
            for k in 0..spuren {
                nummer[k] = (teil[k] as usize).min(beiwerte.len() - 1);
            }
            let mut i = spuren;
            for s in schritt.iter_mut() {
                *s = i16::from_le_bytes([teil[i], teil[i + 1]]) as i32;
                i += 2;
            }
            for s in s1.iter_mut() {
                *s = i16::from_le_bytes([teil[i], teil[i + 1]]) as i32;
                i += 2;
            }
            for s in s2.iter_mut() {
                *s = i16::from_le_bytes([teil[i], teil[i + 1]]) as i32;
                i += 2;
            }
            // Die zwei Kopfwerte gehoeren mit in den Strom - **zweiter
            // zuerst**, denn s2 ist der aeltere.
            for v in &s2 {
                aus.push(*v as i16);
            }
            for v in &s1 {
                aus.push(*v as i16);
            }

            let daten = &teil[i..];
            let anzahl = daten.len() * 2;
            for n in 0..anzahl {
                let k = n % spuren;
                let v = nibble(daten, n) as i32;
                let (a1, a2) = beiwerte[nummer[k]];
                let vorzeichen = if v >= 8 { v - 16 } else { v };
                let mut wert = (s1[k] * a1 + s2[k] * a2) / 256 + vorzeichen * schritt[k];
                wert = wert.clamp(-32_768, 32_767);
                s2[k] = s1[k];
                s1[k] = wert;
                schritt[k] = (MS_ANPASSUNG[v as usize] * schritt[k]) / 256;
                if schritt[k] < 16 {
                    schritt[k] = 16;
                }
                aus.push(wert as i16);
            }
        }
        Ok(aus)
    }

    /// **IMA-ADPCM** (Art 0x11) - die einfachere Spielart ohne Vorhersage.
    ///
    /// Jeder Block faengt mit einem Kopf je Spur an: der erste Wert
    /// (16-bittig) und der Tabellenindex (ein Byte plus ein Fuellbyte).
    /// Danach kommen Viererbits, **niedere zuerst**, und in Paketen von
    /// je vier Bytes pro Spur.
    fn ima_adpcm(&self, roh: &[u8]) -> Ergebnis<Vec<i16>> {
        let spuren = self.spuren as usize;
        let block = self.blockgroesse as usize;
        if block < 4 * spuren {
            return Err(Fehler::Format("IMA-ADPCM: blocks too small".into()));
        }
        let mut aus = Vec::new();
        for teil in roh.chunks(block) {
            if teil.len() < 4 * spuren {
                break;
            }
            let mut wert = vec![0i32; spuren];
            let mut index = vec![0i32; spuren];
            for k in 0..spuren {
                wert[k] = i16::from_le_bytes([teil[k * 4], teil[k * 4 + 1]]) as i32;
                index[k] = (teil[k * 4 + 2] as i32).clamp(0, 88);
                aus.push(wert[k] as i16);
            }
            // Ab hier Pakete zu vier Bytes je Spur, acht Werte je Paket.
            let daten = &teil[4 * spuren..];
            let pakete = daten.len() / (4 * spuren);
            for paket in 0..pakete {
                for n in 0..8 {
                    for k in 0..spuren {
                        let ab = (paket * spuren + k) * 4 + n / 2;
                        let b = daten[ab];
                        // **Niedere vier Bit zuerst** - anders als bei MS.
                        let v = if n % 2 == 0 { b & 0x0F } else { b >> 4 } as i32;
                        let schritt = IMA_SCHRITTE[index[k] as usize];
                        let mut d = schritt >> 3;
                        if v & 4 != 0 {
                            d += schritt;
                        }
                        if v & 2 != 0 {
                            d += schritt >> 1;
                        }
                        if v & 1 != 0 {
                            d += schritt >> 2;
                        }
                        if v & 8 != 0 {
                            wert[k] -= d;
                        } else {
                            wert[k] += d;
                        }
                        wert[k] = wert[k].clamp(-32_768, 32_767);
                        index[k] = (index[k] + IMA_INDEX[v as usize]).clamp(0, 88);
                        aus.push(wert[k] as i16);
                    }
                }
            }
        }
        Ok(aus)
    }
}

/// Die Anpassungstabelle von MS-ADPCM - fuer beide Arten dieselbe Rolle:
/// sie sagt, wie stark der Schrittweite nach jedem Wert nachgefuehrt wird.
const MS_ANPASSUNG: [i32; 16] = [
    230, 230, 230, 230, 307, 409, 512, 614, 768, 614, 512, 409, 307, 230, 230, 230,
];

/// Die sieben Vorgabe-Beiwerte von MS-ADPCM.
///
/// Eine Datei bringt ihre eigenen mit (im Zusatzteil des `fmt`-Blocks);
/// diese hier gelten, wenn keine dabei sind. Gemessen an
/// `secosmic_lo.wav`: sie traegt genau diese sieben.
const MS_BEIWERTE: [(i32, i32); 7] = [
    (256, 0),
    (512, -256),
    (0, 0),
    (192, 64),
    (240, 0),
    (460, -208),
    (392, -232),
];

/// Die Schrittweiten-Tabelle von IMA-ADPCM (89 Eintraege).
const IMA_SCHRITTE: [i32; 89] = [
    7, 8, 9, 10, 11, 12, 13, 14, 16, 17, 19, 21, 23, 25, 28, 31, 34, 37, 41, 45, 50, 55, 60, 66,
    73, 80, 88, 97, 107, 118, 130, 143, 157, 173, 190, 209, 230, 253, 279, 307, 337, 371, 408,
    449, 494, 544, 598, 658, 724, 796, 876, 963, 1060, 1166, 1282, 1411, 1552, 1707, 1878, 2066,
    2272, 2499, 2749, 3024, 3327, 3660, 4026, 4428, 4871, 5358, 5894, 6484, 7132, 7845, 8630,
    9493, 10442, 11487, 12635, 13899, 15289, 16818, 18500, 20350, 22385, 24623, 27086, 29794,
    32767,
];

/// Wie der IMA-Index nach jedem Viererbit nachgefuehrt wird.
const IMA_INDEX: [i32; 16] = [-1, -1, -1, -1, 2, 4, 6, 8, -1, -1, -1, -1, 2, 4, 6, 8];

/// Ein Viererbit mit Vorzeichen - die oberen vier Bit sind zuerst dran.
/// A-law nach G.711 - `SDL_wave.c` rechnet dasselbe.
pub(crate) fn alaw_wert(b: u8) -> i16 {
    let a = b ^ 0x55;
    let mut t = i32::from(a & 0x0F) << 4;
    let segment = (a & 0x70) >> 4;
    match segment {
        0 => t += 8,
        1 => t += 0x108,
        _ => {
            t += 0x108;
            t <<= segment - 1;
        }
    }
    (if a & 0x80 != 0 { t } else { -t }) as i16
}

/// mu-law nach G.711 - `SDL_wave.c` rechnet dasselbe.
pub(crate) fn mulaw_wert(b: u8) -> i16 {
    let u = !b;
    let t = ((i32::from(u & 0x0F) << 3) + 0x84) << ((u & 0x70) >> 4);
    (if u & 0x80 != 0 { 0x84 - t } else { t - 0x84 }) as i16
}

fn nibble(roh: &[u8], n: usize) -> u8 {
    let b = roh[n / 2];
    if n.is_multiple_of(2) {
        b >> 4
    } else {
        b & 0x0F
    }
}

/// Gleitkomma nach 16-Bit-Ganzzahl, mit Begrenzung statt Ueberlauf.
pub(crate) fn nach_i16(v: f64) -> i16 {
    (v.clamp(-1.0, 1.0) * 32_767.0) as i16
}

// ---------------------------------------------------------------------------
//  Schreiben
// ---------------------------------------------------------------------------

/// Packt Abtastwerte in eine WAV-Datei im Speicher.
///
/// Uebernommen aus `rust/crates/pgz-engine/src/audio.rs` (`to_wav`) und um
/// Abtastrate und Spurenzahl erweitert - dort standen 44100 Hz und Mono
/// fest im Kopf.
///
/// ```
/// use rustgame_mixer::wav;
/// let daten = wav::schreiben(&[0, 1000, -1000, 0], 44_100, 1);
/// assert_eq!(&daten[0..4], b"RIFF");
/// let zurueck = wav::lesen(&daten).unwrap();
/// assert_eq!(zurueck.werte, vec![0, 1000, -1000, 0]);
/// assert_eq!(zurueck.frequenz, 44_100);
/// ```
pub fn schreiben(werte: &[i16], frequenz: u32, spuren: u16) -> Vec<u8> {
    let laenge = (werte.len() * 2) as u32;
    let rahmenbytes = spuren as u32 * 2;
    let mut heraus = Vec::with_capacity(44 + laenge as usize);
    heraus.extend_from_slice(b"RIFF");
    heraus.extend_from_slice(&(36 + laenge).to_le_bytes());
    heraus.extend_from_slice(b"WAVEfmt ");
    heraus.extend_from_slice(&16u32.to_le_bytes()); // Laenge des fmt-Blocks
    heraus.extend_from_slice(&1u16.to_le_bytes()); // PCM
    heraus.extend_from_slice(&spuren.to_le_bytes());
    heraus.extend_from_slice(&frequenz.to_le_bytes());
    heraus.extend_from_slice(&(frequenz * rahmenbytes).to_le_bytes()); // Bytes je Sekunde
    heraus.extend_from_slice(&(rahmenbytes as u16).to_le_bytes()); // Bytes je Rahmen
    heraus.extend_from_slice(&16u16.to_le_bytes()); // Bits je Wert
    heraus.extend_from_slice(b"data");
    heraus.extend_from_slice(&laenge.to_le_bytes());
    for w in werte {
        heraus.extend_from_slice(&w.to_le_bytes());
    }
    heraus
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hin_und_zurueck() {
        let werte: Vec<i16> = (0..1000).map(|i| (i * 31 - 15000) as i16).collect();
        let daten = schreiben(&werte, 22_050, 2);
        let zurueck = lesen(&daten).unwrap();
        assert_eq!(zurueck.werte, werte);
        assert_eq!(zurueck.spuren, 2);
        assert_eq!(zurueck.frequenz, 22_050);
        assert_eq!(zurueck.rahmen(), 500);
    }

    #[test]
    fn kopf_stimmt_mit_dem_fundus_ueberein() {
        // Derselbe Test wie in rust/crates/pgz-engine/src/audio.rs, damit
        // der uebernommene Teil nachweislich derselbe geblieben ist.
        let daten = schreiben(&[0i16; 100], 44_100, 1);
        assert_eq!(&daten[0..4], b"RIFF");
        assert_eq!(&daten[8..12], b"WAVE");
        assert_eq!(&daten[36..40], b"data");
        let laenge = u32::from_le_bytes([daten[40], daten[41], daten[42], daten[43]]) as usize;
        assert_eq!(laenge, daten.len() - 44);
    }

    #[test]
    fn acht_bit_vorzeichenlos() {
        // 128 ist die Mitte, 0 und 255 die Raender.
        let mut d = Vec::new();
        d.extend_from_slice(b"RIFF");
        d.extend_from_slice(&(36u32 + 3).to_le_bytes());
        d.extend_from_slice(b"WAVEfmt ");
        d.extend_from_slice(&16u32.to_le_bytes());
        d.extend_from_slice(&1u16.to_le_bytes());
        d.extend_from_slice(&1u16.to_le_bytes());
        d.extend_from_slice(&22_050u32.to_le_bytes());
        d.extend_from_slice(&22_050u32.to_le_bytes());
        d.extend_from_slice(&1u16.to_le_bytes());
        d.extend_from_slice(&8u16.to_le_bytes());
        d.extend_from_slice(b"data");
        d.extend_from_slice(&3u32.to_le_bytes());
        d.extend_from_slice(&[0, 128, 255]);
        let k = lesen(&d).unwrap();
        assert_eq!(k.werte, vec![-32768, 0, 32512]);
        assert_eq!(k.frequenz, 22_050);
    }

    #[test]
    fn zu_kurz_gibt_pygames_meldung() {
        let f = lesen_als(b"kurz", Weg::Musik).unwrap_err();
        assert_eq!(f.to_string(), "Couldn't read first 12 bytes of audio data");
        let f = lesen(b"kurz").unwrap_err();
        assert_eq!(
            f.to_string(),
            "Could not find RIFF or WAVE identifiers (not a Waveform file)"
        );
    }

    /// Baut eine WAV-Datei aus Bloecken - fuer die Fehlertafel.
    fn riff(bloecke: &[&[u8]]) -> Vec<u8> {
        let mut inhalt = b"WAVE".to_vec();
        for b in bloecke {
            inhalt.extend_from_slice(b);
        }
        let mut d = b"RIFF".to_vec();
        d.extend_from_slice(&(inhalt.len() as u32).to_le_bytes());
        d.extend_from_slice(&inhalt);
        d
    }

    fn block(name: &[u8; 4], daten: &[u8]) -> Vec<u8> {
        let mut d = name.to_vec();
        d.extend_from_slice(&(daten.len() as u32).to_le_bytes());
        d.extend_from_slice(daten);
        if daten.len() & 1 == 1 {
            d.push(0);
        }
        d
    }

    fn fmt(art: u16, spuren: u16, rate: u32, bits: u16) -> Vec<u8> {
        let ausr = spuren.wrapping_mul(bits) / 8;
        let mut d = Vec::new();
        for x in [art, spuren] {
            d.extend_from_slice(&x.to_le_bytes());
        }
        d.extend_from_slice(&rate.to_le_bytes());
        d.extend_from_slice(&(rate * u32::from(ausr)).to_le_bytes());
        d.extend_from_slice(&ausr.to_le_bytes());
        d.extend_from_slice(&bits.to_le_bytes());
        block(b"fmt ", &d)
    }

    /// Die Fehlertafel aus [`Weg`] - jede Zeile gemessen an pygame-ce
    /// 2.5.7, `Sound(BytesIO(..))` und `music.load(BytesIO(..))`.
    #[test]
    fn beide_wege_sagen_pygames_saetze() {
        let daten = block(b"data", &[0u8; 64]);
        let gut = fmt(1, 1, 22_050, 16);
        let faelle: Vec<(Vec<u8>, &str, &str)> = vec![
            (
                riff(&[&daten]),
                "Missing fmt chunk in WAVE file",
                "Bad WAV file (no FMT chunk)",
            ),
            (
                riff(&[&gut]),
                "Missing data chunk in WAVE file",
                "Bad WAV file (no DATA chunk)",
            ),
            (
                riff(&[&block(b"fmt ", &[1, 0, 1, 0, 0x22, 0x56, 0, 0]), &daten]),
                "Invalid WAVE fmt chunk length (too small)",
                "Wave format chunk too small",
            ),
            (
                riff(&[&block(b"fmt ", &[1, 0, 1, 0, 1, 0, 1, 0])]),
                "Missing data chunk in WAVE file",
                "Wave format chunk too small",
            ),
            (
                riff(&[&fmt(1, 0, 22_050, 16), &daten]),
                "Invalid number of channels",
                "Parameter 'src_channels' is invalid",
            ),
            (
                riff(&[&fmt(1, 300, 22_050, 16), &daten]),
                "Number of channels exceeds limit of 255",
                "Invalid source channels",
            ),
            (
                riff(&[&fmt(1, 1, 0, 16), &daten]),
                "Invalid sample rate",
                "Source rate is equal to or less than zero",
            ),
            (
                riff(&[&fmt(1, 1, 22_050, 13), &daten]),
                "13-bit PCM format not supported",
                "Unknown PCM format with 13 bits",
            ),
            (
                riff(&[&fmt(0xABCD, 1, 22_050, 16), &daten]),
                "Unknown WAVE format tag: 0xabcd",
                "Unknown WAVE data format",
            ),
            (
                riff(&[&fmt(99, 1, 22_050, 16)]),
                "Missing data chunk in WAVE file",
                "Unknown WAVE data format",
            ),
            (
                riff(&[&fmt(99, 0, 22_050, 16), &daten]),
                "Invalid number of channels",
                "Unknown WAVE data format",
            ),
        ];
        for (d, klang, musik) in &faelle {
            let k = lesen_als(d, Weg::Klang).unwrap_err().to_string();
            let m = lesen_als(d, Weg::Musik).unwrap_err().to_string();
            assert_eq!((k.as_str(), m.as_str()), (*klang, *musik));
        }
        // Und was nur ein Weg annimmt.
        let float64 = riff(&[&fmt(3, 1, 22_050, 64), &daten]);
        assert_eq!(
            lesen_als(&float64, Weg::Klang).unwrap_err().to_string(),
            "64-bit IEEE floating-point format not supported"
        );
        assert!(lesen_als(&float64, Weg::Musik).is_ok());
        let verkehrt = riff(&[&daten, &gut]);
        assert_eq!(
            lesen_als(&verkehrt, Weg::Klang).unwrap_err().to_string(),
            "fmt chunk after data chunk in WAVE file"
        );
        assert!(lesen_als(&verkehrt, Weg::Musik).is_ok());
    }

    /// A-law und mu-law: dieselben Werte wie SDL - Stichproben aus der
    /// gemessenen Tafel (alle 256 Bytewerte gleich G.711).
    #[test]
    fn alaw_und_mulaw_wie_sdl() {
        assert_eq!(alaw_wert(0), -5504);
        assert_eq!(alaw_wert(1), -5248);
        assert_eq!(alaw_wert(128), 5504);
        assert_eq!(alaw_wert(0x55), -8);
        assert_eq!(mulaw_wert(0), -32124);
        assert_eq!(mulaw_wert(1), -31100);
        assert_eq!(mulaw_wert(127), 0);
        assert_eq!(mulaw_wert(128), 32124);
    }

    #[test]
    fn unbekannte_bloecke_werden_uebersprungen() {
        // Ein LIST-Block vor dem data-Block darf nicht stoeren.
        let werte = [1i16, 2, 3, 4];
        let gut = schreiben(&werte, 44_100, 1);
        let mut d = gut[..36].to_vec();
        d.extend_from_slice(b"LIST");
        d.extend_from_slice(&6u32.to_le_bytes());
        d.extend_from_slice(b"INFOxy");
        d.extend_from_slice(&gut[36..]);
        let k = lesen(&d).unwrap();
        assert_eq!(k.werte, werte);
    }
}
