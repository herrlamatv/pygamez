//! aiff.rs
//! =======
//! AIFF, AIFF-C und 8SVX - uebertragen aus SDL_mixer 2.8.1, der Fassung in
//! pygame-ce 2.5.7.
//!
//! ## Zwei Leser, wie bei WAV
//!
//! SDL_mixer liest AIFF an **zwei** Stellen, und die koennen verschieden
//! viel:
//!
//! | Weg | C-Quelle | kann |
//! |---|---|---|
//! | `mixer.Sound` | `load_aiff.c`, `Mix_LoadAIFF_RW` | AIFF und 8SVX, nur 8 und 16 Bit, keine Kompression |
//! | `mixer.music` | `music_wav.c`, `LoadAIFFMusic` | AIFF und AIFF-C (`NONE`, `sowt`, `raw `, `ulaw`/`alaw`, `fl32`, `fl64`), 8 bis 64 Bit, Etiketten aus `NAME`/`AUTH`/`(c) ` und `ID3 ` |
//!
//! Bis Runde 19 las rustgame **keines** davon: `Sound("x.aiff")` gab
//! `Unrecognized audio format`, wo pygame den Klang spielt.
//!
//! Die Werte gehen als 16 Bit in denselben Umrechner wie WAV
//! ([`crate::umrechnen`]): S8 als `v << 8` ist in SDLs Gleitkommaweg
//! **dieselbe** Zahl (`v / 128` gegen `(v << 8) / 32768`), und 16 Bit
//! big-endian wird nur umgedreht.

use crate::{Ergebnis, Fehler};
use crate::wav::Klangdaten;

/// Ein Fehlertext von SDL_mixer (`Mix_SetError`).
fn satz(s: &str) -> Fehler {
    Fehler::Format(s.into())
}

/// Liest vier Bytes als Kennung; hinter dem Ende stehen Nullen - so wie
/// `SDL_ReadLE32` an einem kurzen Strom einen Teil oder nichts fuellt.
fn kennung(d: &[u8], ab: usize) -> [u8; 4] {
    let mut k = [0u8; 4];
    for (i, b) in k.iter_mut().enumerate() {
        *b = d.get(ab + i).copied().unwrap_or(0);
    }
    k
}

fn be32(d: &[u8], ab: usize) -> u32 {
    u32::from_be_bytes(kennung(d, ab))
}

fn be16(d: &[u8], ab: usize) -> u16 {
    u16::from_be_bytes([
        d.get(ab).copied().unwrap_or(0),
        d.get(ab + 1).copied().unwrap_or(0),
    ])
}

/// `SANE_to_Uint32` - die Abtastrate aus dem 80-Bit-Gleitkommawert (aus
/// libsndfile, wie SDL_mixer sagt: "I don't pretend to fully understand
/// it").
fn sane_zu_u32(b: &[u8]) -> u32 {
    if b.len() < 6 {
        return 0;
    }
    if (b[0] & 0x80) != 0 || b[0] <= 0x3F || b[0] > 0x40 || (b[0] == 0x40 && b[1] > 0x1C) {
        return 0;
    }
    let v = (u32::from(b[2]) << 23) | (u32::from(b[3]) << 15) | (u32::from(b[4]) << 7) | (u32::from(b[5]) >> 1);
    v >> (29 - u32::from(b[1]))
}

/// `Mix_LoadAIFF_RW` - der Weg von `mixer.Sound`.
pub(crate) fn klang_lesen(d: &[u8]) -> Ergebnis<Klangdaten> {
    // `FORMchunk = ReadLE32; chunk_length = ReadBE32` - und ein Altfall:
    // steht "AIFF" schon an zweiter Stelle, war `FORM` schon gelesen.
    let form = kennung(d, 0);
    let zweite = be32(d, 4);
    let (form, magie, mut ab) = if zweite == u32::from_le_bytes(*b"AIFF") {
        (*b"FORM", *b"AIFF", 8usize)
    } else {
        (form, kennung(d, 8), 12usize)
    };
    if &form != b"FORM" || (&magie != b"AIFF" && &magie != b"8SVX") {
        return Err(satz("Unrecognized file type (not AIFF nor 8SVX)"));
    }
    let aiff = &magie == b"AIFF";
    let (mut ssnd, mut comm, mut vhdr, mut body) = (false, false, false, false);
    let (mut spuren, mut rahmen, mut bits, mut frequenz) = (0u16, 0u32, 0u16, 0u32);
    let mut start = 0usize;
    loop {
        let art = kennung(d, ab);
        let laenge = be32(d, ab + 4) as usize;
        let inhalt = ab + 8;
        let mut naechster = inhalt + laenge;
        // "Paranoia to avoid infinite loops"
        if laenge == 0 {
            break;
        }
        match &art {
            b"SSND" => {
                ssnd = true;
                let versatz = be32(d, inhalt) as usize;
                start = inhalt + 8 + versatz;
            }
            b"COMM" => {
                comm = true;
                spuren = be16(d, inhalt);
                rahmen = be32(d, inhalt + 2);
                bits = be16(d, inhalt + 6);
                let sane: Vec<u8> = (0..10).map(|i| d.get(inhalt + 8 + i).copied().unwrap_or(0)).collect();
                frequenz = sane_zu_u32(&sane);
                if frequenz == 0 {
                    return Err(satz("Bad AIFF sample frequency"));
                }
            }
            b"VHDR" => {
                vhdr = true;
                frequenz = u32::from(be16(d, inhalt + 12));
                spuren = 1;
                bits = 8;
            }
            b"BODY" => {
                body = true;
                rahmen = laenge as u32;
                start = inhalt;
            }
            _ => {}
        }
        // "a 0 pad byte can be stored for any odd-length chunk"
        if laenge & 1 != 0 {
            naechster += 1;
        }
        let fertig = if aiff { ssnd && comm } else { vhdr && body };
        if fertig {
            break;
        }
        ab = naechster;
    }
    if aiff && !ssnd {
        return Err(satz("Bad AIFF (no SSND chunk)"));
    }
    if aiff && !comm {
        return Err(satz("Bad AIFF (no COMM chunk)"));
    }
    if !aiff && !vhdr {
        return Err(satz("Bad 8SVX (no VHDR chunk)"));
    }
    if !aiff && !body {
        return Err(satz("Bad 8SVX (no BODY chunk)"));
    }
    let breite = match bits {
        8 => 1usize,
        16 => 2,
        _ => return Err(satz("Unsupported AIFF samplesize")),
    };
    // `audio_len = channels * numsamples * (samplesize / 8)` in `Uint32`.
    let laenge = (u32::from(spuren).wrapping_mul(rahmen).wrapping_mul(breite as u32)) as usize;
    if start.checked_add(laenge).is_none_or(|e| e > d.len()) {
        return Err(satz("Unable to read audio data"));
    }
    let roh = &d[start..start + (laenge & !(breite - 1))];
    let werte = if breite == 1 {
        roh.iter().map(|&b| i16::from(b as i8) << 8).collect()
    } else {
        roh.as_chunks::<2>().0.iter().map(|c| i16::from_be_bytes([c[0], c[1]])).collect()
    };
    Ok(Klangdaten { werte, spuren, frequenz })
}

/// Was `LoadAIFFMusic` herausfindet.
struct Musikkopf {
    spuren: u16,
    rahmen: u32,
    bits: u16,
    frequenz: u32,
    kompression: Option<[u8; 4]>,
    start: usize,
    etiketten: crate::id3::Etiketten,
}

/// `LoadAIFFMusic` bis zur Formatentscheidung.
fn musikkopf(d: &[u8]) -> Ergebnis<Musikkopf> {
    let magie = kennung(d, 8);
    if &magie != b"AIFF" && &magie != b"AIFC" {
        return Err(satz("Unrecognized file type (not AIFF or AIFC)"));
    }
    let aifc = &magie == b"AIFC";
    let (mut ssnd, mut comm, mut fver) = (false, false, false);
    let mut k = Musikkopf {
        spuren: 0,
        rahmen: 0,
        bits: 0,
        frequenz: 0,
        kompression: None,
        start: 0,
        etiketten: crate::id3::Etiketten::default(),
    };
    let mut ab = 12usize;
    loop {
        let art = kennung(d, ab);
        let laenge = be32(d, ab + 4) as usize;
        let inhalt = ab + 8;
        let mut naechster = inhalt + laenge;
        if !laenge.is_multiple_of(2) {
            naechster += 1;
        }
        let text = |von: usize| -> String {
            let ende = (von + laenge).min(d.len());
            let roh = d.get(von..ende).unwrap_or(&[]);
            // `SDL_calloc(chunk_length + 1)`: alles bis zur ersten Null.
            let bis = roh.iter().position(|&b| b == 0).unwrap_or(roh.len());
            String::from_utf8_lossy(&roh[..bis]).into_owned()
        };
        match &art {
            b"SSND" => {
                ssnd = true;
                k.start = inhalt + 8 + be32(d, inhalt) as usize;
            }
            b"FVER" => fver = true,
            b"ID3 " => {
                let ende = (inhalt + laenge).min(d.len());
                if inhalt + laenge > d.len() {
                    return Err(Fehler::Format(format!(
                        "Couldn't read {laenge} bytes from WAV file"
                    )));
                }
                let id3 = crate::id3::etiketten(&d[inhalt..ende]);
                for (ziel, quelle) in [
                    (&mut k.etiketten.title, id3.title),
                    (&mut k.etiketten.artist, id3.artist),
                    (&mut k.etiketten.album, id3.album),
                    (&mut k.etiketten.copyright, id3.copyright),
                ] {
                    if !quelle.is_empty() {
                        *ziel = quelle;
                    }
                }
            }
            b"NAME" => k.etiketten.title = text(inhalt),
            b"AUTH" => k.etiketten.artist = text(inhalt),
            b"(c) " => k.etiketten.copyright = text(inhalt),
            b"COMM" => {
                comm = true;
                k.spuren = be16(d, inhalt);
                k.rahmen = be32(d, inhalt + 2);
                k.bits = be16(d, inhalt + 6);
                let sane: Vec<u8> = (0..10).map(|i| d.get(inhalt + 8 + i).copied().unwrap_or(0)).collect();
                k.frequenz = sane_zu_u32(&sane);
                if aifc {
                    k.kompression = Some(kennung(d, inhalt + 18));
                }
            }
            _ => {}
        }
        // `while (next_chunk < file_length && SDL_RWseek(..) >= 0)`
        if naechster >= d.len() {
            break;
        }
        ab = naechster;
    }
    if !ssnd {
        return Err(satz("Bad AIFF/AIFF-C file (no SSND chunk)"));
    }
    if !comm {
        return Err(satz("Bad AIFF/AIFF-C file (no COMM chunk)"));
    }
    if aifc && !fver {
        return Err(satz("Bad AIFF-C file (no FVER chunk)"));
    }
    Ok(k)
}

/// Die Etiketten einer AIFF-Datei fuer `music.get_metadata()`.
pub(crate) fn etiketten(d: &[u8]) -> crate::id3::Etiketten {
    musikkopf(d).map(|k| k.etiketten).unwrap_or_default()
}

/// `LoadAIFFMusic` samt den `fetch_*`-Funktionen - der Weg von
/// `mixer.music`.
pub(crate) fn musik_lesen(d: &[u8]) -> Ergebnis<Klangdaten> {
    let k = musikkopf(d)?;
    let komp = k.kompression;
    let c = |s: &[u8; 4]| komp.as_ref() == Some(s);
    // Wie die Bytes in 16 Bit werden - siehe `wav.rs` fuer dieselben
    // Faelle in little endian.
    enum Art {
        S8,
        U8,
        S16Be,
        S16Le,
        S24Be,
        S32Be,
        S32Le,
        F32Be,
        F64Be,
        Ulaw,
        Alaw,
    }
    let unbekannt = || satz("Unknown samplesize in data format");
    let art = match k.bits {
        8 => match komp {
            None => Art::S8,
            Some(_) if c(b"raw ") => Art::U8,
            Some(_) if c(b"sowt") => Art::S8,
            Some(_) if c(b"ulaw") => Art::Ulaw,
            Some(_) if c(b"alaw") => Art::Alaw,
            _ => return Err(unbekannt()),
        },
        16 => match komp {
            None => Art::S16Be,
            Some(_) if c(b"sowt") => Art::S16Le,
            Some(_) if c(b"NONE") => Art::S16Be,
            Some(_) if c(b"ULAW") => Art::Ulaw,
            Some(_) if c(b"ALAW") => Art::Alaw,
            _ => return Err(unbekannt()),
        },
        // `fetch_pcm24be` schreibt die 24 Bit in die oberen drei Bytes
        // eines S32 - fuer 16 Bit zaehlen die oberen zwei.
        24 => match komp {
            None => Art::S24Be,
            Some(_) if c(b"NONE") => Art::S24Be,
            _ => return Err(unbekannt()),
        },
        32 => match komp {
            None => Art::S32Be,
            Some(_) if c(b"sowt") => Art::S32Le,
            Some(_) if c(b"NONE") => Art::S32Be,
            Some(_) if c(b"fl32") || c(b"FL32") => Art::F32Be,
            _ => return Err(unbekannt()),
        },
        64 => match komp {
            None => Art::F64Be,
            Some(_) if c(b"fl64") => Art::F64Be,
            _ => return Err(unbekannt()),
        },
        _ => return Err(unbekannt()),
    };
    let breite = (k.bits / 8) as usize;
    let ende = k
        .start
        .saturating_add(usize::from(k.spuren) * k.rahmen as usize * breite)
        .min(d.len());
    let roh = d.get(k.start..ende).unwrap_or(&[]);
    let werte: Vec<i16> = match art {
        Art::S8 => roh.iter().map(|&b| i16::from(b as i8) << 8).collect(),
        Art::U8 => roh.iter().map(|&b| ((i32::from(b) - 128) * 256) as i16).collect(),
        Art::S16Be => roh.as_chunks::<2>().0.iter().map(|c| i16::from_be_bytes([c[0], c[1]])).collect(),
        Art::S16Le => roh.as_chunks::<2>().0.iter().map(|c| i16::from_le_bytes([c[0], c[1]])).collect(),
        Art::S24Be => roh.as_chunks::<3>().0.iter().map(|c| i16::from_be_bytes([c[0], c[1]])).collect(),
        Art::S32Be => roh.as_chunks::<4>().0.iter().map(|c| i16::from_be_bytes([c[0], c[1]])).collect(),
        Art::S32Le => roh.as_chunks::<4>().0.iter().map(|c| i16::from_le_bytes([c[2], c[3]])).collect(),
        Art::F32Be => roh
            .as_chunks::<4>().0.iter()
            .map(|c| crate::wav::nach_i16(f64::from(f32::from_be_bytes([c[0], c[1], c[2], c[3]]))))
            .collect(),
        // `fetch_float64be` macht aus jedem `double` erst ein `float`.
        Art::F64Be => roh
            .as_chunks::<8>().0.iter()
            .map(|c| {
                let v = f64::from_be_bytes([c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]]);
                crate::wav::nach_i16(f64::from(v as f32))
            })
            .collect(),
        Art::Ulaw => roh.iter().map(|&b| crate::wav::mulaw_wert(b)).collect(),
        Art::Alaw => roh.iter().map(|&b| crate::wav::alaw_wert(b)).collect(),
    };
    // **Rate oder Spurzahl null laedt trotzdem.** `LoadAIFFMusic` prueft
    // beides nicht; erst der Umrechner scheitert, und gemessen spielt
    // pygame danach **endlos Stille** (`get_busy()` bleibt wahr, `get_pos`
    // laeuft). Hier gibt es einen leeren Klang - das Laden stimmt, das
    // endlose Weiterlaufen eines kaputten Stroms ist nicht nachgebaut
    // (Grenze `Musik-Dekodierer kaputt`).
    if k.frequenz == 0 || k.spuren == 0 {
        return Ok(Klangdaten {
            werte: Vec::new(),
            spuren: k.spuren.max(1),
            frequenz: 0,
        });
    }
    Ok(Klangdaten {
        werte,
        spuren: k.spuren,
        frequenz: k.frequenz,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aiff(bits: u16, spuren: u16, rate_sane: [u8; 10], daten: &[u8]) -> Vec<u8> {
        let rahmen = daten.len() as u32 / u32::from(spuren) / u32::from(bits / 8);
        let mut comm = Vec::new();
        comm.extend_from_slice(&spuren.to_be_bytes());
        comm.extend_from_slice(&rahmen.to_be_bytes());
        comm.extend_from_slice(&bits.to_be_bytes());
        comm.extend_from_slice(&rate_sane);
        let mut ssnd = vec![0u8; 8];
        ssnd.extend_from_slice(daten);
        let mut koerper = b"AIFF".to_vec();
        for (n, inhalt) in [(b"COMM", &comm), (b"SSND", &ssnd)] {
            koerper.extend_from_slice(n);
            koerper.extend_from_slice(&(inhalt.len() as u32).to_be_bytes());
            koerper.extend_from_slice(inhalt);
            if inhalt.len() % 2 == 1 {
                koerper.push(0);
            }
        }
        let mut d = b"FORM".to_vec();
        d.extend_from_slice(&(koerper.len() as u32).to_be_bytes());
        d.extend(koerper);
        d
    }

    /// 44100 als 80-Bit-SANE: Exponent 0x400E, Mantisse 0xAC44 << 48.
    const SANE_44100: [u8; 10] = [0x40, 0x0E, 0xAC, 0x44, 0, 0, 0, 0, 0, 0];

    #[test]
    fn abtastrate_aus_sane() {
        assert_eq!(sane_zu_u32(&SANE_44100), 44100);
    }

    #[test]
    fn sechzehn_bit_big_endian() {
        let d = aiff(16, 1, SANE_44100, &[0x12, 0x34, 0xFF, 0xFE]);
        let k = klang_lesen(&d).unwrap();
        assert_eq!((k.spuren, k.frequenz), (1, 44100));
        assert_eq!(k.werte, vec![0x1234, -2]);
        assert_eq!(musik_lesen(&d).unwrap().werte, vec![0x1234, -2]);
    }

    #[test]
    fn acht_bit_vorzeichenbehaftet() {
        let d = aiff(8, 2, SANE_44100, &[0x7F, 0x80, 0x00, 0x01]);
        assert_eq!(klang_lesen(&d).unwrap().werte, vec![0x7F00, -0x8000, 0, 0x100]);
    }

    #[test]
    fn saetze_wie_sdl_mixer() {
        let mut d = aiff(16, 1, SANE_44100, &[0, 0]);
        d[8..12].copy_from_slice(b"WAVE");
        assert!(matches!(klang_lesen(&d), Err(Fehler::Format(s)) if s == "Unrecognized file type (not AIFF nor 8SVX)"));
        assert!(matches!(musik_lesen(&d), Err(Fehler::Format(s)) if s == "Unrecognized file type (not AIFF or AIFC)"));
    }
}
