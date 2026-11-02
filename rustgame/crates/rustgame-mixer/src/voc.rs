//! voc.rs
//! ======
//! Creative-Voice-Dateien (`.voc`) - uebertragen aus SDL_mixer 2.8.1
//! `src/codecs/load_voc.c`. `Mix_LoadWAV_RW` erkennt sie an `Crea` und
//! reicht sie an `Mix_LoadVOC_RW`; fuer `mixer.music` gibt es keinen
//! VOC-Leser.
//!
//! Bis Runde 19 fehlte der Leser ganz: `Sound("x.voc")` gab
//! `Unrecognized audio format`.
//!
//! ## Was nicht nachgebaut ist
//!
//! Zwei Wege in `load_voc.c` hinterlassen **keinen** eigenen Fehlertext
//! (eine Datei kuerzer als die Kennung, ein VOC ohne einen einzigen
//! Datenblock): pygame meldet dann den Satz, der vom vorigen Aufruf in
//! SDL liegen geblieben ist. Und ein Datenblock, der hinter dem Dateiende
//! weitergeht, laesst `audio_len` um den fehlenden Rest wachsen - der Rest
//! ist nicht vorbelegter Speicher. Beides steht unter der benannten
//! Grenze `was pygame nicht festhaelt`; hier gibt es einen eigenen Satz
//! bzw. Stille.

use crate::wav::Klangdaten;
use crate::{Ergebnis, Fehler};

const VOC_TERM: u8 = 0;
const VOC_DATA: u8 = 1;
const VOC_CONT: u8 = 2;
const VOC_SILENCE: u8 = 3;
const VOC_MARKER: u8 = 4;
const VOC_TEXT: u8 = 5;
const VOC_LOOP: u8 = 6;
const VOC_LOOPEND: u8 = 7;
const VOC_EXTENDED: u8 = 8;
const VOC_DATA_16: u8 = 9;
const VOC_BAD_RATE: u32 = !0;

const ST_SIZE_BYTE: u32 = 1;
const ST_SIZE_WORD: u32 = 2;

fn satz(s: &str) -> Fehler {
    Fehler::Format(s.into())
}

/// Ein Lesestrom wie `SDL_RWops` ueber dem Speicher.
struct Strom<'a> {
    d: &'a [u8],
    pos: usize,
}

impl Strom<'_> {
    /// `SDL_RWread(src, buf, n, 1) == 1`.
    fn lesen(&mut self, n: usize) -> Option<&[u8]> {
        if self.pos + n > self.d.len() {
            self.pos = self.d.len();
            return None;
        }
        let s = &self.d[self.pos..self.pos + n];
        self.pos += n;
        Some(s)
    }
    fn byte(&mut self) -> Option<u8> {
        self.lesen(1).map(|b| b[0])
    }
}

/// `vs_t`.
#[derive(Default)]
struct Zustand {
    rest: u32,
    rate: u32,
    silent: bool,
    size: u32,
    channels: u8,
    has_extended: bool,
}

/// Was `voc_get_block` an der `SDL_AudioSpec` setzt.
#[derive(Default)]
struct Spec {
    freq: i64,
    channels: u8,
}

/// `voc_get_block`: `Ok(())` fuer 1, `Err(Some(satz))` fuer 0 mit
/// Fehlertext, `Err(None)` fuer 0 ohne.
fn block_holen(s: &mut Strom, v: &mut Zustand, spec: &mut Spec) -> Result<(), Option<Fehler>> {
    v.silent = false;
    while v.rest == 0 {
        let Some(block) = s.byte() else {
            return Ok(());
        };
        if block == VOC_TERM {
            return Ok(());
        }
        let Some(b24) = s.lesen(3) else {
            return Ok(());
        };
        let sblen = u32::from(b24[0]) | (u32::from(b24[1]) << 8) | (u32::from(b24[2]) << 16);
        match block {
            VOC_DATA => {
                let uc = s.byte().ok_or(None)?;
                if !v.has_extended {
                    if uc == 0 {
                        return Err(Some(satz("VOC Sample rate is zero?")));
                    }
                    if v.rate != VOC_BAD_RATE && u32::from(uc) != v.rate {
                        return Err(Some(satz("VOC sample rate codes differ")));
                    }
                    v.rate = u32::from(uc);
                    // `(Uint16)(1000000.0/(256 - v->rate))`
                    spec.freq = i64::from((1_000_000.0 / (256.0 - f64::from(v.rate))) as i64 as u16);
                    v.channels = 1;
                }
                let uc = s.byte().ok_or(None)?;
                if uc != 0 {
                    return Err(Some(satz("VOC decoder only interprets 8-bit data")));
                }
                v.has_extended = false;
                v.rest = sblen.wrapping_sub(2);
                v.size = ST_SIZE_BYTE;
                return Ok(());
            }
            VOC_DATA_16 => {
                let r = s.lesen(4).ok_or(None)?;
                let neu = u32::from_le_bytes([r[0], r[1], r[2], r[3]]);
                if neu == 0 {
                    return Err(Some(satz("VOC Sample rate is zero?")));
                }
                if v.rate != VOC_BAD_RATE && neu != v.rate {
                    return Err(Some(satz("VOC sample rate codes differ")));
                }
                v.rate = neu;
                spec.freq = i64::from(neu as i32);
                match s.byte().ok_or(None)? {
                    8 => v.size = ST_SIZE_BYTE,
                    16 => v.size = ST_SIZE_WORD,
                    _ => return Err(Some(satz("VOC with unknown data size"))),
                }
                v.channels = s.byte().ok_or(None)?;
                s.lesen(6).ok_or(None)?;
                v.rest = sblen.wrapping_sub(12);
                return Ok(());
            }
            VOC_CONT => {
                v.rest = sblen;
                return Ok(());
            }
            VOC_SILENCE => {
                let p = s.lesen(2).ok_or(None)?;
                let mut period = u16::from_le_bytes([p[0], p[1]]);
                let uc = s.byte().ok_or(None)?;
                if uc == 0 {
                    return Err(Some(satz("VOC silence sample rate is zero")));
                }
                if v.rate != VOC_BAD_RATE && u32::from(uc) != v.rate {
                    // `(Uint16)((period * (256 - uc))/(256 - v->rate))`
                    let z = i64::from(period) * (256 - i64::from(uc));
                    let n = 256 - i64::from(v.rate as i32);
                    period = if n == 0 { 0 } else { (z / n) as u16 };
                } else {
                    v.rate = u32::from(uc);
                }
                v.rest = u32::from(period);
                v.silent = true;
                return Ok(());
            }
            VOC_LOOP | VOC_LOOPEND => {
                for _ in 0..sblen {
                    s.byte().ok_or(None)?;
                }
            }
            VOC_EXTENDED => {
                v.has_extended = true;
                let r = s.lesen(2).ok_or(None)?;
                let neu = u16::from_le_bytes([r[0], r[1]]);
                if neu == 0 {
                    return Err(Some(satz("VOC sample rate is zero")));
                }
                if v.rate != VOC_BAD_RATE && u32::from(neu) != v.rate {
                    return Err(Some(satz("VOC sample rate codes differ")));
                }
                v.rate = u32::from(neu);
                if s.byte().ok_or(None)? != 0 {
                    return Err(Some(satz("VOC decoder only interprets 8-bit data")));
                }
                spec.channels = if s.byte().ok_or(None)? != 0 { 2 } else { 1 };
                spec.freq = (256_000_000i64 / (65536 - i64::from(v.rate))) / i64::from(spec.channels);
                continue;
            }
            _ => {
                // `VOC_MARKER` liest zwei Bytes mehr und faellt dann durch.
                if block == VOC_MARKER {
                    s.lesen(2).ok_or(None)?;
                }
                for _ in 0..sblen {
                    s.byte().ok_or(None)?;
                }
                if block == VOC_TEXT {
                    continue;
                }
            }
        }
    }
    Ok(())
}

/// `voc_read`: schreibt den Rest des Blocks nach `ziel` und gibt `done`.
fn lesen(s: &mut Strom, v: &mut Zustand, ziel: &mut Vec<u8>, spec: &mut Spec) -> Result<u32, Option<Fehler>> {
    // Scheitert hier `voc_get_block`, gibt `voc_read` 0 zurueck - und die
    // Schleife in `Mix_LoadVOC_RW` endet **ohne** Fehler.
    if v.rest == 0 && block_holen(s, v, spec).is_err() {
        return Ok(0);
    }
    if v.rest == 0 {
        return Ok(0);
    }
    if v.silent {
        let stille = if v.size == ST_SIZE_WORD { 0x00 } else { 0x80 };
        ziel.extend(std::iter::repeat_n(stille, v.rest as usize));
        let done = v.rest;
        v.rest = 0;
        Ok(done)
    } else {
        let verfuegbar = s.d.len().saturating_sub(s.pos);
        let n = (v.rest as usize).min(verfuegbar);
        ziel.extend_from_slice(&s.d[s.pos..s.pos + n]);
        s.pos += n;
        // Was hinter dem Dateiende fehlt, ist bei SDL nicht vorbelegter
        // Speicher - hier Stille.
        let fehlt = v.rest as usize - n;
        let stille = if v.size == ST_SIZE_WORD { 0x00 } else { 0x80 };
        ziel.extend(std::iter::repeat_n(stille, fehlt));
        v.rest -= n as u32;
        let mut done = n as u32;
        if v.size == ST_SIZE_WORD {
            done >>= 1;
        }
        Ok(done)
    }
}

/// `Mix_LoadVOC_RW`.
pub(crate) fn klang_lesen(d: &[u8]) -> Ergebnis<Klangdaten> {
    let ohne = || satz("Couldn't read VOC header");
    let mut s = Strom { d, pos: 0 };
    // `voc_check_header`
    let kennung = s.lesen(20).ok_or_else(ohne)?;
    if kennung != b"Creative Voice File\x1a" {
        return Err(satz("Unrecognized file type (not VOC)"));
    }
    let o = s.lesen(2).ok_or_else(ohne)?;
    let ofs = u16::from_le_bytes([o[0], o[1]]) as usize;
    if ofs > d.len() {
        return Err(ohne());
    }
    s.pos = ofs;

    let mut v = Zustand {
        rate: VOC_BAD_RATE,
        ..Zustand::default()
    };
    let mut spec = Spec::default();
    // Ein Lesefehler ohne `Mix_SetError` laesst bei SDL den **vorigen**
    // Fehler stehen - im frischen Zustand ist das der leere Satz, gemessen
    // an einem Markerblock, der den naechsten Blockkopf mitliest.
    let fehler = |f: Option<Fehler>| f.unwrap_or_else(|| satz(""));
    block_holen(&mut s, &mut v, &mut spec).map_err(fehler)?;
    if v.rate == VOC_BAD_RATE {
        return Err(satz("VOC data had no sound!"));
    }
    if v.size == 0 {
        return Err(satz("VOC data had invalid word size!"));
    }
    let wort = v.size == ST_SIZE_WORD;
    if spec.channels == 0 {
        spec.channels = v.channels;
    }
    if v.rest == 0 {
        return Err(satz("VOC data had no sound!"));
    }
    let mut roh: Vec<u8> = Vec::with_capacity(v.rest as usize);
    let mut laenge = v.rest as u64;
    loop {
        let done = lesen(&mut s, &mut v, &mut roh, &mut spec).map_err(fehler)?;
        if done == 0 {
            break;
        }
        block_holen(&mut s, &mut v, &mut spec).map_err(fehler)?;
        laenge += u64::from(v.rest);
    }
    let spuren = spec.channels.max(1);
    let groesse = if wort { 2 } else { 1 } * u64::from(spuren);
    // `*audio_len &= ~(samplesize-1)`
    let laenge = (laenge & !(groesse - 1)) as usize;
    roh.resize(laenge, if wort { 0 } else { 0x80 });
    let werte = if wort {
        roh.as_chunks::<2>().0.iter().map(|c| i16::from_le_bytes([c[0], c[1]])).collect()
    } else {
        roh.iter().map(|&b| ((i32::from(b) - 128) * 256) as i16).collect()
    };
    if spec.freq <= 0 {
        return Err(satz("VOC data had no sound!"));
    }
    Ok(Klangdaten {
        werte,
        spuren: u16::from(spuren),
        frequenz: spec.freq as u32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn voc(bloecke: &[u8]) -> Vec<u8> {
        let mut d = b"Creative Voice File\x1a".to_vec();
        d.extend_from_slice(&26u16.to_le_bytes());
        d.extend_from_slice(&0x010Au16.to_le_bytes());
        d.extend_from_slice(&0x1129u16.to_le_bytes());
        d.extend_from_slice(bloecke);
        d
    }

    #[test]
    fn acht_bit_datenblock() {
        // Rate-Code 156: 1000000 / (256 - 156) = 10000 Hz.
        let d = voc(&[VOC_DATA, 5, 0, 0, 156, 0, 0x80, 0xFF, 0x00, VOC_TERM]);
        let k = klang_lesen(&d).unwrap();
        assert_eq!((k.spuren, k.frequenz), (1, 10000));
        assert_eq!(k.werte, vec![0, 127 * 256, -128 * 256]);
    }

    #[test]
    fn kennung_falsch() {
        let mut d = voc(&[VOC_TERM]);
        d[0] = b'X';
        assert!(matches!(klang_lesen(&d), Err(Fehler::Format(s)) if s == "Unrecognized file type (not VOC)"));
    }
}
