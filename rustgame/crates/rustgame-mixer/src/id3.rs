//! ID3-Etiketten am Anfang einer MP3-Datei.
//!
//! Zwei Aufgaben, und die zweite ist die unerwartete:
//!
//! 1. `music.get_metadata()` liefert `title`, `album`, `artist` und
//!    `copyright`. Bei MP3 stehen die in einem ID3-Block **vor** den
//!    Tondaten.
//! 2. **Der Block muss uebersprungen werden, bevor der Dekoder ihn sieht.**
//!    symphonia liest ihn selbst - und scheitert an
//!    `examples/data/metadata.mp3` mit `out of bounds`. Der Grund steht in
//!    der Datei: der ID3-Kopf nennt seine Laenge als
//!
//!    ```text
//!    Bytes 6..10:  00 00 01 46
//!    ```
//!
//!    Nach der Vorschrift sind das **syncsafe**-Zahlen, also je sieben Bit:
//!    `(1 << 7) | 0x46` = **198**. Die Rahmen im Block sind aber zusammen
//!    **202** Bytes lang, und die Tondaten fangen bei 212 an. Als
//!    gewoehnliche Zahl gelesen waeren es 326 - auch falsch. Die Datei ist
//!    schlicht fehlerhaft, SDL_mixer nimmt es hin, symphonia nicht.
//!
//! Deshalb wird hier **nicht der angegebenen Laenge geglaubt**, sondern
//! durch die Rahmen gelaufen und danach der erste **gueltige**
//! MPEG-Rahmenkopf gesucht. "Gueltig" heisst mehr als die elf Sync-Bits:
//! in derselben Datei stehen bei 336 vier `FF`-Bytes, die die Sync-Pruefung
//! bestehen und trotzdem kein Rahmenkopf sind (Lage 0 ist reserviert).

/// Die vier Felder, die `music.get_metadata()` liefert.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Etiketten {
    /// `TIT2` (bzw. `TT2` in ID3v2.2).
    pub title: String,
    /// `TALB` / `TAL`.
    pub album: String,
    /// `TPE1` / `TP1`.
    pub artist: String,
    /// `TCOP` / `TCR`.
    pub copyright: String,
}

/// Was am Anfang steht - Etiketten und die Stelle, an der der Ton beginnt.
///
/// Ohne ID3-Block kommt `(leer, 0)` zurueck; die Daten gehen dann
/// unveraendert an den Dekoder.
pub fn lesen(daten: &[u8]) -> (Etiketten, usize) {
    let mut e = Etiketten::default();
    if daten.len() < 10 || &daten[0..3] != b"ID3" {
        return (e, 0);
    }
    let fassung = daten[3];
    let flaggen = daten[5];
    let angegeben = syncsafe(&daten[6..10]);

    // Ein erweiterter Kopf sitzt vor den Rahmen und nennt seine eigene
    // Laenge. In v2.3 zaehlt sie sich selbst nicht mit, in v2.4 schon -
    // hier wird beides vertragen, indem der kleinere Sprung genommen und
    // der Rest ueber die Rahmensuche geheilt wird.
    let mut i = 10usize;
    if flaggen & 0x40 != 0 && daten.len() >= i + 4 {
        let laenge = if fassung >= 4 {
            syncsafe(&daten[i..i + 4])
        } else {
            u32::from_be_bytes([daten[i], daten[i + 1], daten[i + 2], daten[i + 3]]) as usize
        };
        i += 4 + laenge.min(daten.len() - i - 4);
    }

    // Durch die Rahmen laufen. Endet bei Fuellbytes (Kennung aus Nullen),
    // bei einer unsinnigen Laenge oder am Ende der Daten.
    let kennungslaenge = if fassung <= 2 { 3 } else { 4 };
    let kopflaenge = if fassung <= 2 { 6 } else { 10 };
    while i + kopflaenge <= daten.len() {
        let kennung = &daten[i..i + kennungslaenge];
        if kennung[0] == 0 {
            break;
        }
        if !kennung.iter().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
            break;
        }
        let groesse = if fassung <= 2 {
            let b = &daten[i + 3..i + 6];
            ((b[0] as usize) << 16) | ((b[1] as usize) << 8) | b[2] as usize
        } else if fassung >= 4 {
            // v2.4 schreibt auch die Rahmenlaengen syncsafe. Manche Werkzeuge
            // tun es trotzdem nicht; passt die syncsafe-Lesung nicht in die
            // Daten, wird die gewoehnliche genommen.
            let s = syncsafe(&daten[i + 4..i + 8]);
            let g = u32::from_be_bytes([
                daten[i + 4],
                daten[i + 5],
                daten[i + 6],
                daten[i + 7],
            ]) as usize;
            if i + kopflaenge + s <= daten.len() {
                s
            } else {
                g
            }
        } else {
            u32::from_be_bytes([daten[i + 4], daten[i + 5], daten[i + 6], daten[i + 7]]) as usize
        };
        let ab = i + kopflaenge;
        if groesse == 0 || ab + groesse > daten.len() {
            break;
        }
        let inhalt = &daten[ab..ab + groesse];
        let ziel: Option<&mut String> = match kennung {
            b"TIT2" | b"TT2" => Some(&mut e.title),
            b"TALB" | b"TAL" => Some(&mut e.album),
            b"TPE1" | b"TP1" => Some(&mut e.artist),
            b"TCOP" | b"TCR" => Some(&mut e.copyright),
            _ => None,
        };
        if let Some(z) = ziel {
            if z.is_empty() {
                *z = text(inhalt);
            }
        }
        i = ab + groesse;
    }

    // Ein Fussblock (nur v2.4) ist zehn Bytes lang.
    if fassung >= 4 && flaggen & 0x10 != 0 {
        i += 10;
    }
    // Wenigstens bis hinter den angegebenen Block, wenn der weiter reicht
    // als die Rahmen - dann waren es Fuellbytes.
    i = i.max(10 + angegeben).min(daten.len());
    (e, tonbeginn(daten, i))
}

/// Nur die Etiketten - der bequeme Weg, wenn die Stelle nicht gebraucht wird.
pub fn etiketten(daten: &[u8]) -> Etiketten {
    // AIFF traegt seine Etiketten in eigenen Bloecken (`NAME`, `AUTH`,
    // `(c) `, `ID3 `) - siehe `crate::aiff`.
    if daten.starts_with(b"FORM") {
        return crate::aiff::etiketten(daten);
    }
    lesen(daten).0
}

/// Nur die Stelle, an der der Ton beginnt.
pub fn tonbeginn_von(daten: &[u8]) -> usize {
    lesen(daten).1
}

/// Eine syncsafe-Zahl: vier Bytes mit je sieben nutzbaren Bit.
fn syncsafe(b: &[u8]) -> usize {
    ((b[0] as usize & 0x7F) << 21)
        | ((b[1] as usize & 0x7F) << 14)
        | ((b[2] as usize & 0x7F) << 7)
        | (b[3] as usize & 0x7F)
}

/// Der Text eines Textrahmens: ein Byte Kodierung, dann die Zeichen.
///
/// `0` ist Latin-1, `1` UTF-16 mit Bytefolgemarke, `2` UTF-16 ohne Marke
/// (immer big endian), `3` UTF-8. Abschliessende Nullen fallen weg.
fn text(inhalt: &[u8]) -> String {
    if inhalt.is_empty() {
        return String::new();
    }
    let (art, rest) = (inhalt[0], &inhalt[1..]);
    let s = match art {
        0 => rest.iter().map(|&b| b as char).collect(),
        1 | 2 => {
            let (rest, gross) = match (art, rest.first(), rest.get(1)) {
                (1, Some(0xFF), Some(0xFE)) => (&rest[2..], false),
                (1, Some(0xFE), Some(0xFF)) => (&rest[2..], true),
                // Ohne Marke gilt big endian - so steht es in der Vorschrift
                // fuer Kodierung 2, und fuer eine Kodierung 1 ohne Marke ist
                // es die vertraeglichere Annahme.
                _ => (rest, true),
            };
            let einheiten: Vec<u16> = rest
                .as_chunks::<2>().0.iter()
                .map(|c| {
                    if gross {
                        u16::from_be_bytes([c[0], c[1]])
                    } else {
                        u16::from_le_bytes([c[0], c[1]])
                    }
                })
                .collect();
            String::from_utf16_lossy(&einheiten)
        }
        _ => String::from_utf8_lossy(rest).into_owned(),
    };
    s.trim_end_matches('\0').to_string()
}

/// Sucht ab `ab` den ersten gueltigen MPEG-Rahmenkopf.
///
/// Gibt es keinen, bleibt es bei `ab` - dann soll der Dekoder selbst
/// meckern, statt dass hier eine Stelle geraten wird.
fn tonbeginn(daten: &[u8], ab: usize) -> usize {
    let grenze = daten.len().saturating_sub(4);
    // Etwas Vorlauf, falls die Rahmensuche einen Schritt zu weit ging.
    let start = ab.saturating_sub(4).min(grenze);
    for i in start..=grenze {
        if rahmenkopf_gueltig(&daten[i..i + 4]) {
            return i;
        }
    }
    ab.min(daten.len())
}

/// Ist das ein MPEG-Audio-Rahmenkopf?
///
/// Geprueft wird mehr als die elf Sync-Bits, denn die allein treffen auch
/// auf Fuellbytes zu (`FF FF FF FF` in `metadata.mp3` bei 336):
///
/// * Lage (Layer) `00` ist reserviert,
/// * Bitrate `1111` ist ungueltig, `0000` heisst "frei" und kommt nicht vor,
/// * Abtastrate `11` ist reserviert.
fn rahmenkopf_gueltig(b: &[u8]) -> bool {
    if b[0] != 0xFF || (b[1] & 0xE0) != 0xE0 {
        return false;
    }
    let lage = (b[1] >> 1) & 0x03;
    let bitrate = (b[2] >> 4) & 0x0F;
    let rate = (b[2] >> 2) & 0x03;
    lage != 0 && bitrate != 0 && bitrate != 0x0F && rate != 3
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Baut einen ID3v2.3-Block mit einer **falschen** Laengenangabe, genau
    /// wie `examples/data/metadata.mp3` sie hat.
    fn mit_falscher_laenge() -> Vec<u8> {
        let mut rahmen = Vec::new();
        for (kennung, inhalt) in [
            (b"TPE1", "Audacity Generator"),
            (b"TIT2", "Small Tone"),
            (b"TCOP", "2023 Nobody"),
            (b"TALB", "Tones"),
        ] {
            let mut nutz = vec![0u8];
            nutz.extend_from_slice(inhalt.as_bytes());
            rahmen.extend_from_slice(kennung);
            rahmen.extend_from_slice(&(nutz.len() as u32).to_be_bytes());
            rahmen.extend_from_slice(&[0, 0]);
            rahmen.extend_from_slice(&nutz);
        }
        let mut d = Vec::new();
        d.extend_from_slice(b"ID3");
        d.extend_from_slice(&[3, 0, 0]);
        // **Zu klein**: die Rahmen sind laenger, als hier steht.
        let falsch = rahmen.len() - 8;
        d.extend_from_slice(&[
            0,
            0,
            ((falsch >> 7) & 0x7F) as u8,
            (falsch & 0x7F) as u8,
        ]);
        d.extend_from_slice(&rahmen);
        // Ein gueltiger Rahmenkopf: MPEG1 Lage 3, 128 kbit, 44100 Hz.
        d.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x00]);
        d.extend_from_slice(&[0u8; 100]);
        d
    }

    #[test]
    fn etiketten_und_tonbeginn_trotz_falscher_laenge() {
        let d = mit_falscher_laenge();
        let (e, ab) = lesen(&d);
        assert_eq!(e.title, "Small Tone");
        assert_eq!(e.album, "Tones");
        assert_eq!(e.artist, "Audacity Generator");
        assert_eq!(e.copyright, "2023 Nobody");
        assert_eq!(
            &d[ab..ab + 2],
            &[0xFF, 0xFB],
            "der Ton beginnt am Rahmenkopf, nicht an der angegebenen Laenge"
        );
    }

    /// Ohne ID3 bleibt alles, wie es ist - die Daten gehen unberuehrt weiter.
    #[test]
    fn ohne_id3_bleibt_der_anfang_stehen() {
        let d = [0xFFu8, 0xFB, 0x90, 0x00, 1, 2, 3];
        assert_eq!(lesen(&d), (Etiketten::default(), 0));
    }

    /// Vier `FF`-Bytes bestehen die Sync-Pruefung und sind trotzdem kein
    /// Rahmenkopf - genau der Fall, der in `metadata.mp3` steht.
    #[test]
    fn fuellbytes_sind_kein_rahmenkopf() {
        assert!(!rahmenkopf_gueltig(&[0xFF, 0xFF, 0xFF, 0xFF]));
        assert!(rahmenkopf_gueltig(&[0xFF, 0xFB, 0x90, 0x00]));
        // Lage 0 ist reserviert.
        assert!(!rahmenkopf_gueltig(&[0xFF, 0xF9, 0x90, 0x00]));
    }

    /// UTF-16 mit Bytefolgemarke, wie iTunes es schreibt.
    #[test]
    fn utf16_mit_marke() {
        let mut v = vec![1u8, 0xFF, 0xFE];
        for c in "Tön".encode_utf16() {
            v.extend_from_slice(&c.to_le_bytes());
        }
        assert_eq!(text(&v), "Tön");
    }
}
