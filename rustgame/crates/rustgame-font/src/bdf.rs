//! bdf.rs
//! ======
//! **Schriften, die aus Pixeln bestehen** - das Glyph Bitmap Distribution
//! Format.
//!
//! Eine BDF-Datei hat keine Kurven. Sie zaehlt fuer jedes Zeichen auf, welche
//! Pixel schwarz sind, und zwar als Text:
//!
//! ```text
//! STARTCHAR A
//! ENCODING 65
//! DWIDTH 8 0
//! BBX 6 6 0 0
//! BITMAP
//! 78
//! C4
//! ...
//! ENDCHAR
//! ```
//!
//! `BBX b h xo yo` ist der Kasten des Bildes: `b` Spalten, `h` Zeilen, und
//! seine linke untere Ecke liegt `(xo, yo)` von der Schreibmarke auf der
//! Grundlinie entfernt. Jede Zeile der `BITMAP` ist hexadezimal, **auf
//! ganze Bytes aufgefuellt**: bei `b = 6` stehen dort zwei Ziffern, und die
//! zwei untersten Bits gehoeren nicht zum Bild.
//!
//! ## Warum das hier steht
//!
//! `freetype_test.py` laedt drei solche Dateien in `setUpClass`
//! (`PyGameMono-8.bdf`, `PyGameMono-18-75dpi.bdf`,
//! `PyGameMono-18-100dpi.bdf`). Scheitert eine davon, fallen **alle 43
//! Tests der Klasse** aus - nicht, weil sie Bitmapschriften pruefen,
//! sondern weil das Aufstellen scheitert. Acht von ihnen fassen die
//! Bitmapschriften wirklich an.
//!
//! fontdue liest nur Umrisse. Also wird die Datei hier gelesen und in
//! dieselben Zahlen uebersetzt, die sonst aus `hhea` und `hmtx` kommen:
//!
//! | BDF | entspricht |
//! |---|---|
//! | `PIXEL_SIZE` (oder `SIZE`) | `unitsPerEm` |
//! | `FONT_ASCENT` | `hhea.ascender` |
//! | `-FONT_DESCENT` | `hhea.descender` |
//! | `DWIDTH` | `hmtx`-Vorschub |
//! | `UNDERLINE_POSITION/_THICKNESS` | `post` |
//!
//! Das ist keine Notloesung, sondern genau das, was FreeType selbst tut:
//! ein Geviert von `PIXEL_SIZE` Einheiten macht aus jeder Schrifteinheit
//! genau ein Pixel, sobald die verlangte Groesse die eingebaute ist. Der
//! ganze Rechenweg in [`crate::font`] bleibt damit unveraendert.
//!
//! ## Was gemessen ist
//!
//! ```text
//! f = ft.Font("PyGameMono-8.bdf")
//! f.fixed_sizes        1
//! f.scalable           False
//! f.get_sizes()        [(8, 8, 8, 8, 8)]
//! f.size               (8.0, 8.0)     # ein Paar, keine Zahl
//! f.fixed_width        True
//! f.style = STYLE_STRONG   ->  AttributeError
//! ```
//!
//! Die vorletzte Zeile ist die ueberraschende: bei einer Bitmapschrift ist
//! `Font.size` ein **Paar**, weil die eingebaute Groesse waagerecht und
//! senkrecht verschieden sein darf.

use std::collections::HashMap;

/// Ein einzelnes Zeichen aus der Datei.
#[derive(Debug, Clone)]
pub struct Zeichen {
    /// Der Kodierungswert - `-1`, wenn das Zeichen keinen hat.
    pub kodierung: i32,
    /// Der Vorschub in Pixeln (`DWIDTH`).
    pub vorschub: i32,
    /// Breite des Bildes in Pixeln.
    pub breite: u32,
    /// Hoehe des Bildes in Pixeln.
    pub hoehe: u32,
    /// Abstand der linken Bildkante von der Schreibmarke.
    pub x_versatz: i32,
    /// Abstand der **unteren** Bildkante von der Grundlinie.
    pub y_versatz: i32,
    /// Deckung, ein Byte je Pixel, Zeile 0 **oben** - wie bei fontdue.
    pub deckung: Vec<u8>,
}

impl Zeichen {
    fn leer() -> Zeichen {
        Zeichen {
            kodierung: -1,
            vorschub: 0,
            breite: 0,
            hoehe: 0,
            x_versatz: 0,
            y_versatz: 0,
            deckung: Vec::new(),
        }
    }
}

/// Eine gelesene BDF-Datei.
#[derive(Debug)]
pub struct Bitmapschrift {
    /// Die eingebaute Groesse in Pixeln, waagerecht und senkrecht.
    pub ppem: (u32, u32),
    /// Die Punktgroesse, wie sie `SIZE` nennt.
    pub punktgroesse: i32,
    /// FreeTypes `bsize->size` in 26.6 (`BDF_Face_Init`): `POINT_SIZE` von
    /// Dezipunkten auf 72 je Zoll umgerechnet, sonst die mittlere Breite.
    /// `PyGameMono-18-75dpi.bdf`: 1148, also 17,9375 - nicht 18.
    pub groesse26: i64,
    /// FreeTypes `bsize->y_ppem` in 26.6: `PIXEL_SIZE << 6`, sonst aus
    /// `size` und `RESOLUTION_Y`.
    pub y_ppem26: i64,
    /// Die Aufloesung in dpi.
    pub aufloesung: (u32, u32),
    /// Der Kasten, in den jedes Zeichen passt: Breite, Hoehe, x, y.
    pub kasten: (u32, u32, i32, i32),
    /// Wie hoch die Zeichen ueber die Grundlinie reichen.
    pub ascent: i32,
    /// Wie tief sie darunter reichen - **positiv** gezaehlt.
    pub descent: i32,
    /// Wo der Unterstrich liegt, von der Grundlinie aus.
    pub unterstrich_lage: i16,
    /// Wie dick er ist.
    pub unterstrich_dicke: i16,
    /// `FAMILY_NAME`.
    pub familie: String,
    /// `WEIGHT_NAME` und `SLANT` zusammengefasst - der Schnittname.
    pub schnitt: String,
    /// Feste Laufweite? `SPACING` ist dann `"C"` oder `"M"`.
    pub feste_breite: bool,
    /// Alle Zeichen, in der Reihenfolge der Datei. Nummer 0 ist die
    /// Glyphnummer 0, also `.notdef` - genau wie bei einer TTF.
    pub zeichen: Vec<Zeichen>,
    /// Kodierungswert -> Nummer in [`Self::zeichen`].
    nach_nummer: HashMap<u32, u16>,
}

impl Bitmapschrift {
    /// Die Glyphnummer eines Zeichens - unbekannt wird zu 0.
    pub fn glyph(&self, c: char) -> u16 {
        *self.nach_nummer.get(&(c as u32)).unwrap_or(&0)
    }

    /// Das Zeichen zu einer Glyphnummer.
    pub fn bei(&self, gid: u16) -> Option<&Zeichen> {
        self.zeichen.get(gid as usize)
    }
}

/// `FT_MulDiv`: `a * b / c`, gerundet.
fn ft_mul_div(a: i64, b: i64, c: i64) -> i64 {
    if c == 0 {
        return 0x7FFF_FFFF;
    }
    let negativ = (a < 0) ^ (b < 0) ^ (c < 0);
    let (a, b, c) = (a.abs(), b.abs(), c.abs());
    let v = (a * b + c / 2) / c;
    if negativ {
        -v
    } else {
        v
    }
}

/// Liest eine BDF-Datei.
///
/// Gibt `None`, wenn es keine ist - der Aufrufer probiert dann das
/// naechste Format. Ein `STARTFONT` am Anfang ist das einzige
/// Erkennungszeichen, das das Format hat.
pub fn lesen(bytes: &[u8]) -> Option<Bitmapschrift> {
    let text = String::from_utf8_lossy(bytes);
    if !text.trim_start().starts_with("STARTFONT") {
        return None;
    }

    let mut ppem = (0u32, 0u32);
    let mut punktgroesse = 0i32;
    // Die Eigenschaften, aus denen FreeType `available_sizes` baut.
    let mut point_size: Option<i64> = None;
    let mut pixel_size: Option<i64> = None;
    let mut average_width: Option<i64> = None;
    let mut res_y: Option<i64> = None;
    let mut aufloesung = (75u32, 75u32);
    let mut kasten = (0u32, 0u32, 0i32, 0i32);
    let mut ascent: Option<i32> = None;
    let mut descent: Option<i32> = None;
    let mut unterstrich_lage = 0i16;
    let mut unterstrich_dicke = 0i16;
    let mut familie = String::new();
    let mut gewicht = String::new();
    let mut neigung = String::new();
    let mut spacing = String::new();
    let mut zeichen: Vec<Zeichen> = Vec::new();

    let mut jetzt: Option<Zeichen> = None;
    // Wie viele Bildzeilen noch kommen, und wie breit sie sind.
    let mut bitzeilen = 0usize;

    for zeile in text.lines() {
        let z = zeile.trim();
        if bitzeilen > 0 {
            if let Some(c) = jetzt.as_mut() {
                zeile_lesen(c, z);
            }
            bitzeilen -= 1;
            continue;
        }
        let (wort, rest) = match z.split_once(char::is_whitespace) {
            Some((w, r)) => (w, r.trim()),
            None => (z, ""),
        };
        match wort {
            "SIZE" => {
                let z: Vec<i64> = zahlen(rest);
                if !z.is_empty() {
                    punktgroesse = z[0] as i32;
                }
                if z.len() >= 3 {
                    aufloesung = (z[1] as u32, z[2] as u32);
                }
            }
            "FONTBOUNDINGBOX" => {
                let z: Vec<i64> = zahlen(rest);
                if z.len() >= 4 {
                    kasten = (z[0] as u32, z[1] as u32, z[2] as i32, z[3] as i32);
                }
            }
            "PIXEL_SIZE" => {
                let z: Vec<i64> = zahlen(rest);
                if !z.is_empty() {
                    ppem = (z[0] as u32, z[0] as u32);
                    pixel_size = Some(z[0]);
                }
            }
            "POINT_SIZE" => point_size = zahlen(rest).first().copied(),
            "AVERAGE_WIDTH" => average_width = zahlen(rest).first().copied(),
            "RESOLUTION_X" => {
                if let Some(v) = zahlen(rest).first() {
                    aufloesung.0 = *v as u32;
                }
            }
            "RESOLUTION_Y" => {
                if let Some(v) = zahlen(rest).first() {
                    aufloesung.1 = *v as u32;
                    res_y = Some(*v);
                }
            }
            "FONT_ASCENT" => ascent = zahlen(rest).first().map(|v| *v as i32),
            "FONT_DESCENT" => descent = zahlen(rest).first().map(|v| *v as i32),
            "UNDERLINE_POSITION" => {
                if let Some(v) = zahlen(rest).first() {
                    unterstrich_lage = *v as i16;
                }
            }
            "UNDERLINE_THICKNESS" => {
                if let Some(v) = zahlen(rest).first() {
                    unterstrich_dicke = *v as i16;
                }
            }
            "FAMILY_NAME" => familie = ohne_gaensefuesschen(rest),
            "WEIGHT_NAME" => gewicht = ohne_gaensefuesschen(rest),
            "SLANT" => neigung = ohne_gaensefuesschen(rest),
            "SPACING" => spacing = ohne_gaensefuesschen(rest),
            "STARTCHAR" => jetzt = Some(Zeichen::leer()),
            "ENCODING" => {
                if let (Some(c), Some(v)) = (jetzt.as_mut(), zahlen(rest).first()) {
                    c.kodierung = *v as i32;
                }
            }
            "DWIDTH" => {
                if let (Some(c), Some(v)) = (jetzt.as_mut(), zahlen(rest).first()) {
                    c.vorschub = *v as i32;
                }
            }
            "BBX" => {
                if let Some(c) = jetzt.as_mut() {
                    let z = zahlen(rest);
                    if z.len() >= 4 {
                        c.breite = z[0].max(0) as u32;
                        c.hoehe = z[1].max(0) as u32;
                        c.x_versatz = z[2] as i32;
                        c.y_versatz = z[3] as i32;
                    }
                }
            }
            "BITMAP" => {
                if let Some(c) = jetzt.as_mut() {
                    c.deckung = Vec::with_capacity((c.breite * c.hoehe) as usize);
                    bitzeilen = c.hoehe as usize;
                }
            }
            "ENDCHAR" => {
                if let Some(mut c) = jetzt.take() {
                    // Eine abgeschnittene Datei darf nicht zu einem Bild
                    // fuehren, das kuerzer ist als es behauptet.
                    c.deckung.resize((c.breite * c.hoehe) as usize, 0);
                    zeichen.push(c);
                }
            }
            _ => {}
        }
    }

    if zeichen.is_empty() {
        return None;
    }
    if ppem == (0, 0) {
        // Ohne `PIXEL_SIZE` ist die Hoehe des Kastens die beste Auskunft,
        // die die Datei gibt.
        let h = if kasten.1 > 0 {
            kasten.1
        } else {
            punktgroesse.max(1) as u32
        };
        ppem = (h, h);
    }
    let ascent = ascent.unwrap_or((kasten.1 as i32 + kasten.3).max(0));
    let descent = descent.unwrap_or((-kasten.3).max(0));
    // `BDF_Face_Init`: `bsize->height`, `->width`, `->size`, `->y_ppem`.
    let bs_hoehe = i64::from(ascent + descent);
    let bs_breite = average_width.map_or(bs_hoehe * 2 / 3, |w| (w + 5) / 10);
    let groesse26 = point_size.map_or(bs_breite * 64, |p| ft_mul_div(p, 64 * 7200, 72270));
    let y_ppem26 = match pixel_size {
        Some(p) => p << 6,
        None => match res_y {
            Some(r) => ft_mul_div(groesse26, r, 72),
            None => groesse26,
        },
    };

    let mut nach_nummer = HashMap::new();
    for (i, c) in zeichen.iter().enumerate() {
        if c.kodierung >= 0 {
            nach_nummer.entry(c.kodierung as u32).or_insert(i as u16);
        }
    }

    let schnitt = match (gewicht.as_str(), neigung.as_str()) {
        ("", "") => "Regular".to_string(),
        (g, "R") | (g, "") => {
            if g.is_empty() {
                "Regular".to_string()
            } else {
                g.to_string()
            }
        }
        (g, "I") | (g, "O") => format!("{g} Italic").trim().to_string(),
        (g, _) => g.to_string(),
    };

    Some(Bitmapschrift {
        ppem,
        punktgroesse: if punktgroesse > 0 {
            punktgroesse
        } else {
            ppem.1 as i32
        },
        groesse26,
        y_ppem26,
        aufloesung,
        kasten,
        ascent,
        descent,
        unterstrich_lage,
        unterstrich_dicke,
        familie,
        schnitt,
        feste_breite: spacing == "C" || spacing == "M",
        zeichen,
        nach_nummer,
    })
}

/// Eine Zeile der `BITMAP` - hexadezimal, auf ganze Bytes aufgefuellt.
///
/// **Die Fuellbits gehoeren nicht zum Bild.** Bei `BBX 6 ...` stehen dort
/// zwei Hexziffern, also acht Bits; die beiden untersten sind Beiwerk. Wer
/// sie mitnimmt, bekommt Zeichen, die zwei Pixel zu breit sind und rechts
/// ausfransen.
fn zeile_lesen(c: &mut Zeichen, z: &str) {
    let mut bits: Vec<u8> = Vec::with_capacity(c.breite as usize);
    let mut gelesen = 0u32;
    for ziffer in z.chars() {
        let Some(v) = ziffer.to_digit(16) else {
            continue;
        };
        for k in (0..4).rev() {
            if gelesen >= c.breite {
                break;
            }
            bits.push(if (v >> k) & 1 != 0 { 255 } else { 0 });
            gelesen += 1;
        }
    }
    bits.resize(c.breite as usize, 0);
    c.deckung.extend_from_slice(&bits);
}

/// Alle ganzen Zahlen einer Zeile, Vorzeichen eingeschlossen.
fn zahlen(s: &str) -> Vec<i64> {
    s.split_whitespace().filter_map(|t| t.parse::<i64>().ok()).collect()
}

/// `"PyGameMono"` aus `"PyGameMono"`.
fn ohne_gaensefuesschen(s: &str) -> String {
    s.trim().trim_matches('"').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const KLEIN: &str = "STARTFONT 2.1\n\
FONT -test\n\
SIZE 8 75 75\n\
FONTBOUNDINGBOX 6 7 0 0\n\
STARTPROPERTIES 4\n\
FAMILY_NAME \"PyGameMono\"\n\
PIXEL_SIZE 8\n\
FONT_ASCENT 6\n\
FONT_DESCENT 2\n\
ENDPROPERTIES\n\
CHARS 2\n\
STARTCHAR .notdef\n\
ENCODING 0\n\
DWIDTH 8 0\n\
BBX 6 2 0 0\n\
BITMAP\n\
FC\n\
84\n\
ENDCHAR\n\
STARTCHAR A\n\
ENCODING 65\n\
DWIDTH 8 0\n\
BBX 4 1 1 0\n\
BITMAP\n\
F0\n\
ENDCHAR\n\
ENDFONT\n";

    #[test]
    fn was_in_der_datei_steht_kommt_auch_an() {
        let f = lesen(KLEIN.as_bytes()).expect("ist eine BDF");
        assert_eq!(f.ppem, (8, 8));
        assert_eq!(f.punktgroesse, 8);
        assert_eq!(f.ascent, 6);
        assert_eq!(f.descent, 2);
        assert_eq!(f.familie, "PyGameMono");
        assert_eq!(f.zeichen.len(), 2);
        assert_eq!(f.glyph('A'), 1);
        // Unbekannt landet auf 0 - wie `.notdef` bei einer TTF.
        assert_eq!(f.glyph('Z'), 0);
    }

    #[test]
    fn die_fuellbits_gehoeren_nicht_zum_bild() {
        let f = lesen(KLEIN.as_bytes()).unwrap();
        let notdef = f.bei(0).unwrap();
        assert_eq!(notdef.breite, 6);
        // `FC` ist 1111_1100; bei sechs Spalten bleiben sechs Einsen.
        assert_eq!(&notdef.deckung[0..6], &[255, 255, 255, 255, 255, 255]);
        // `84` ist 1000_0100. Die untersten **zwei** Bits sind Beiwerk;
        // was bleibt, sind die Bits 7 bis 2 - also die erste und die
        // sechste Spalte, nicht nur die erste.
        assert_eq!(&notdef.deckung[6..12], &[255, 0, 0, 0, 0, 255]);
    }

    #[test]
    fn der_versatz_bleibt_erhalten() {
        let f = lesen(KLEIN.as_bytes()).unwrap();
        let a = f.bei(1).unwrap();
        assert_eq!((a.breite, a.hoehe), (4, 1));
        assert_eq!((a.x_versatz, a.y_versatz), (1, 0));
        assert_eq!(a.vorschub, 8);
        assert_eq!(a.deckung, vec![255, 255, 255, 255]);
    }

    #[test]
    fn was_keine_bdf_ist_wird_abgelehnt() {
        assert!(lesen(b"\x00\x01\x00\x00").is_none());
        assert!(lesen(b"OTTO....").is_none());
        // Ein Kopf ohne Zeichen ist auch keine brauchbare Schrift.
        assert!(lesen(b"STARTFONT 2.1\nENDFONT\n").is_none());
    }
}
