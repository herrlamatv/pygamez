//! Die Bildformate neben PNG und BMP.
//!
//! pygame holt sie ueber SDL_image; hier stehen sie selbst. Zwei Wege:
//!
//! * **Ueber eine Kiste** - WebP (`image-webp`), JPEG (`zune-jpeg` zum
//!   Lesen, `jpeg-encoder` zum Schreiben) und GIF (`gif`). Alle drei sind
//!   reines Rust; einen VP8- oder Huffman-Dekodierer von Hand zu schreiben
//!   waere eine eigene Etappe ohne Gewinn.
//! * **Von Hand** - PNM, QOI, TGA und PCX. Die Formate sind so klein, dass
//!   eine Abhaengigkeit mehr kostet als spart, und sie geben die Kontrolle
//!   ueber Tiefe und Flaggen, auf die es hier ankommt.
//!
//! **Welche Tiefe und welche Flaggen herauskommen, ist gemessen**, nicht
//! hergeleitet - an echtem pygame-ce 2.5.7 mit den Beispieldateien:
//!
//! ```text
//! red.jpg          24 Bit  Flaggen 0        -
//! blue.gif          8 Bit  Flaggen 0        Tafel 256
//! alien1.gif        8 Bit  Flaggen 0x1000   Tafel 256, Schluesselfarbe
//! green.pcx        24 Bit  Flaggen 0        -
//! yellow.tga       24 Bit  Flaggen 0        -
//! black.ppm        24 Bit  Flaggen 0        -
//! grey.pgm          8 Bit  Flaggen 0        Tafel 256 (Graukeil)
//! white.pbm         8 Bit  Flaggen 0        Tafel 2
//! crimson.pnm      24 Bit  Flaggen 0        -
//! scarlet.webp     24 Bit  Flaggen 0        -
//! purple.qoi       32 Bit  Flaggen 0x10000  -
//! ```
//!
//! Die Zeile fuer `alien1.gif` ist die lehrreiche: ein durchsichtiger
//! Index wird zur **Schluesselfarbe**, nicht zu einem Alphakanal - genau
//! wie bei PNG mit `tRNS`, siehe [`crate::image`].

use crate::image::{aus_pixeln, aus_pixeln_auch_leer, Ergebnis, Fehler};
use crate::surface::Surface;

// ---------------------------------------------------------------------------
//  WebP
// ---------------------------------------------------------------------------

/// Liest WebP - sowohl verlustbehaftet (VP8) als auch verlustfrei (VP8L).
///
/// Ohne Alphakanal kommt eine Flaeche mit **24 Bit** heraus, mit
/// Alphakanal 32 Bit und `SRCALPHA`. Gemessen an `scarlet.webp` (24) und
/// den Alphavarianten.
pub fn webp_laden(daten: &[u8]) -> Ergebnis<Surface> {
    let mut dek = image_webp::WebPDecoder::new(std::io::Cursor::new(daten))
        .map_err(|e| Fehler::Format(format!("WebP could not be read: {e}")))?;
    let (w, h) = dek.dimensions();
    let mit_alpha = dek.has_alpha();
    let groesse = dek
        .output_buffer_size()
        .ok_or_else(|| Fehler::Format("WebP: image is too large".into()))?;
    let mut puffer = vec![0u8; groesse];
    dek.read_image(&mut puffer)
        .map_err(|e| Fehler::Format(format!("WebP could not be read: {e}")))?;
    let schritt = if mit_alpha { 4 } else { 3 };
    let zeile = w as usize * schritt;
    aus_pixeln(w, h, mit_alpha, |x, y| {
        let i = y * zeile + x * schritt;
        if mit_alpha {
            (puffer[i], puffer[i + 1], puffer[i + 2], puffer[i + 3])
        } else {
            (puffer[i], puffer[i + 1], puffer[i + 2], 255)
        }
    })
}

// ---------------------------------------------------------------------------
//  JPEG
// ---------------------------------------------------------------------------

/// Liest JPEG. Ein JPEG hat nie Durchsichtigkeit, also immer 24 Bit.
///
/// Graustufen werden auf drei gleiche Kanaele gelegt - so macht es
/// SDL_image auch, und `get_bitsize()` bleibt bei 24.
///
/// **Die Pixel sind nicht pygames.** SDL_image dekodiert mit
/// libjpeg-turbo (`JDCT_ISLOW`, weiches Hochrechnen der Farbebenen),
/// `zune-jpeg` rechnet anders: gemessen in Runde 21 an 41 JPEG-Dateien
/// (pygames eigene, PyGameZ' Logos, Windows-Hintergruende) waren 39
/// verschieden, um 2 bis 61 Stufen. Groesse, Tiefe und Flaggen stimmen.
pub fn jpeg_laden(daten: &[u8]) -> Ergebnis<Surface> {
    use zune_jpeg::zune_core::colorspace::ColorSpace;
    let mut dek = zune_jpeg::JpegDecoder::new(daten);
    let pixel = dek
        .decode()
        // **SDL_images Satz.** libjpeg springt bei jedem Fehler in
        // `my_error_exit`, und `IMG_LoadJPG_RW` sagt dann nur
        // `JPEG loading error` - gemessen an `FF D8 FF D9`. Hier stand die
        // Meldung des Dekodierers (`Premature End of image`).
        .map_err(|_| Fehler::Format("JPEG loading error".into()))?;
    let info = dek
        .info()
        .ok_or_else(|| Fehler::Format("JPEG loading error".into()))?;
    let (w, h) = (info.width as u32, info.height as u32);
    let kanaele = match dek.get_output_colorspace() {
        Some(ColorSpace::Luma) => 1,
        Some(ColorSpace::RGB) => 3,
        Some(ColorSpace::RGBA) => 4,
        anderes => {
            return Err(Fehler::Format(format!(
                "JPEG in an unsupported color space: {anderes:?}"
            )))
        }
    };
    let zeile = w as usize * kanaele;
    aus_pixeln(w, h, false, |x, y| {
        let i = y * zeile + x * kanaele;
        match kanaele {
            1 => (pixel[i], pixel[i], pixel[i], 255),
            _ => (pixel[i], pixel[i + 1], pixel[i + 2], 255),
        }
    })
}

/// Schreibt JPEG - `image.save(s, "bild.jpg")`.
///
/// pygame ruft `IMG_SaveJPG` mit der Guete 85 (`imageext.c`), und
/// SDL_image schreibt ueber libjpeg-turbo. Die Datei hier ist **nicht**
/// dieselbe: anderer Kodierer, andere Pixel nach dem Lesen - eine
/// benannte Grenze seit Runde 21.
pub fn jpeg_schreiben(surface: &Surface, guete: u8) -> Ergebnis<Vec<u8>> {
    let (w, h) = surface.get_size();
    let mut rgb = Vec::with_capacity(w as usize * h as usize * 3);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let c = surface.get_at((x, y));
            rgb.extend_from_slice(&[c.r, c.g, c.b]);
        }
    }
    let mut heraus = Vec::new();
    let kodierer = jpeg_encoder::Encoder::new(&mut heraus, guete);
    kodierer
        .encode(&rgb, w as u16, h as u16, jpeg_encoder::ColorType::Rgb)
        .map_err(|e| Fehler::Format(format!("JPEG could not be written: {e}")))?;
    Ok(heraus)
}

// ---------------------------------------------------------------------------
//  GIF
// ---------------------------------------------------------------------------

/// Liest das **erste** Bild eines GIF - mit SDL_images eigenem Leser
/// (`gifsdl.rs`).
///
/// Ergebnis ist eine 8-Bit-Flaeche mit Farbtafel - und, wenn das Bild
/// einen durchsichtigen Index hat, mit **Schluesselfarbe** statt
/// Alphakanal. Gemessen an `alien1.gif`: Tiefe 8, Flaggen `0x1000`
/// (`SRCCOLORKEY`), `get_colorkey()` gibt `(128, 8, 113, 255)`.
pub fn gif_laden(daten: &[u8]) -> Ergebnis<Surface> {
    crate::gifsdl::laden(daten)
}

/// Alle Bilder eines GIF samt Anzeigedauer - `image.load_animation`.
///
/// **Die Dauer kommt in Millisekunden heraus, nicht in Sekunden.**
/// Gemessen an `animated_sample.gif`: pygame gibt `150.0`, und im
/// Steuerblock der Datei stehen 15 Hundertstelsekunden.
pub fn gif_bildfolge(daten: &[u8]) -> Ergebnis<Vec<(Surface, f64)>> {
    crate::gifsdl::bildfolge(daten)
}

// ---------------------------------------------------------------------------
//  TGA
// ---------------------------------------------------------------------------

/// Liest TGA - unkomprimiert und lauflaengenkodiert, 8 bis 32 Bit.
///
/// **Welche Tiefe herauskommt, ist gemessen** - an selbst gebauten Dateien,
/// weil pygames Beispiele nur die 24-Bit-Spielart enthalten:
///
/// ```text
/// Art  Tiefe  Deskriptor   pygame liefert
///  3     8     0x20        8 Bit, Tafel 256 (Graukeil)
///  1     8     0x20        8 Bit, Tafel so lang wie in der Datei
///  2    16     0x20        **15** Bit, ohne Alpha
///  2    16     0x21        **15** Bit, ohne Alpha - das Alphabit wird
///                          uebergangen, auch wenn es gesetzt ist
///  2    24     0x00        24 Bit, Zeilen von **unten** nach oben
///  2    32     0x20        32 Bit mit SRCALPHA - auch **ohne** Alphabits
///                          im Deskriptor wird das vierte Byte genommen
/// ```
///
/// Die dritte und vierte Zeile sind die ueberraschenden: 16 Bit meldet
/// sich als 15, und das eine Alphabit von A1R5G5B5 zaehlt nicht.
pub fn tga_laden(daten: &[u8]) -> Ergebnis<Surface> {
    use crate::color::Color;

    // ## Uebertragen aus SDL_images `IMG_tga.c`
    //
    // Hier stand ein eigener Leser, der strenger war als pygame - und an
    // einer Stelle **abstuerzte**: ein Echtfarbbild mit 8 Bit gab
    // `index out of bounds` und kam als `PanicException` bis nach Python.
    // Gemessen an pygame-ce 2.5.7 gilt:
    //
    // * Was der Kopf nicht hergibt, ist `Unsupported TGA format`: Art 1/9
    //   ohne Tafel, mit mehr als 256 Farben oder nicht 8 Bit; Art 3/11
    //   nicht 8 Bit; Art 2/10 mit 8 Bit; andere Tiefen als 8/15/16/24/32;
    //   **Ursprung rechts** und verschraenkte Zeilen.
    // * **Gelesen wird ohne Pruefung.** `SDL_RWread` liest, was da ist;
    //   der Rest der Flaeche bleibt null. Eine abgeschnittene Datei ist
    //   also ein Bild mit schwarzem Rest, kein Fehler - auch ein halbes
    //   Pixel wird halb geschrieben.
    // * Bei RLE ueberleben **Kopfbyte und Pixel** das Dateiende: das
    //   letzte Paket wiederholt sich, und ein halb gelesenes Pixel hat nur
    //   seine ersten Bytes neu (gemessen `302070` aus `302010` und `70`).
    // * Eine 15/16-Bit-Tafel wird **ohne** Bitwiederholung gespreizt
    //   (`0x7FFF` -> 248), eine 32-Bit-Tafel setzt den letzten Eintrag
    //   mit Alpha unter 128 als Schluesselfarbe.
    // * Breite oder Hoehe 0 ist eine leere Flaeche, kein Fehler.
    let nicht = || Fehler::Format("Unsupported TGA format".into());
    if daten.len() < 18 {
        return Err(Fehler::Format("Error reading TGA data".into()));
    }
    let le16 = |i: usize| u16::from_le_bytes([daten[i], daten[i + 1]]) as usize;
    let kennung = daten[0] as usize;
    let hat_tafel = daten[1] != 0;
    let art = daten[2];
    let farben = le16(5);
    let tafel_bits = daten[7] as usize;
    let (w, h) = (le16(12), le16(14));
    let bits = daten[16] as usize;
    let deskriptor = daten[17];

    let (rle, indiziert, grau) = match art {
        1 | 9 => {
            if !hat_tafel || bits != 8 || farben > 256 {
                return Err(nicht());
            }
            (art == 9, true, false)
        }
        2 | 10 => (art == 10, false, false),
        3 | 11 => {
            if bits != 8 {
                return Err(nicht());
            }
            (art == 11, true, true)
        }
        _ => return Err(nicht()),
    };
    match bits {
        8 if !indiziert => return Err(nicht()),
        8 | 15 | 16 | 24 | 32 => {}
        _ => return Err(nicht()),
    }
    // `TGA_INTERLEAVE_MASK` (0xC0) und `TGA_ORIGIN_RIGHT` (0x10).
    if deskriptor & 0xC0 != 0 || deskriptor & 0x10 != 0 {
        return Err(nicht());
    }
    let von_oben = deskriptor & 0x20 != 0;
    let bpp = bits.div_ceil(8);

    let mut leser = Leser { daten, stelle: 18 + kennung };

    // Die Farbtafel. Nur ein Tafelbild liest sie; alle anderen springen
    // darueber (`SDL_RWseek`).
    let mut tafel: Vec<Color> = Vec::new();
    let mut schluessel: Option<usize> = None;
    if hat_tafel {
        let eintrag = tafel_bits.div_ceil(8);
        let laenge = farben * eintrag;
        if indiziert && !grau {
            let mut roh = vec![0u8; laenge];
            leser.lesen(&mut roh);
            let byte = |i: usize| roh.get(i).copied().unwrap_or(0);
            if tafel_bits == 15 || tafel_bits == 16 {
                for n in 0..farben {
                    let c = u16::from_le_bytes([byte(2 * n), byte(2 * n + 1)]);
                    tafel.push(Color::rgb(
                        ((c >> 7) & 0xF8) as u8,
                        ((c >> 2) & 0xF8) as u8,
                        (c << 3) as u8,
                    ));
                }
            } else {
                let mut p = 0;
                for n in 0..farben {
                    let (b, g, r) = (byte(p), byte(p + 1), byte(p + 2));
                    p += 3;
                    if tafel_bits == 32 {
                        if byte(p) < 128 {
                            schluessel = Some(n);
                        }
                        p += 1;
                    }
                    tafel.push(Color::rgb(r, g, b));
                }
            }
        } else {
            leser.stelle += laenge;
        }
    }
    // Eine Tafel der Laenge 0 laedt (`palette->ncolors = 0`): jeder Index
    // liegt dann jenseits der Tafel - `get_at` gibt Schwarz, gemessen.

    // Die Pixel - **in der Reihenfolge der Datei**, Zeile fuer Zeile.
    let zeile = w * bpp;
    let mut roh = vec![0u8; zeile * h];
    if rle {
        let (mut roh_rest, mut wdh_rest) = (0usize, 0usize);
        let mut kopf = 0u8;
        let mut pixel = [0u8; 4];
        for z in roh.chunks_mut(zeile.max(1)).take(h) {
            let mut x = 0usize;
            loop {
                if roh_rest > 0 {
                    let n = roh_rest.min(w - x);
                    leser.lesen(&mut z[x * bpp..(x + n) * bpp]);
                    roh_rest -= n;
                    x += n;
                    if x == w {
                        break;
                    }
                } else if wdh_rest > 0 {
                    let n = wdh_rest.min(w - x);
                    wdh_rest -= n;
                    for _ in 0..n {
                        z[x * bpp..(x + 1) * bpp].copy_from_slice(&pixel[..bpp]);
                        x += 1;
                    }
                    if x == w {
                        break;
                    }
                }
                let mut k = [kopf];
                leser.lesen(&mut k);
                kopf = k[0];
                if kopf & 0x80 != 0 {
                    leser.lesen(&mut pixel[..bpp]);
                    wdh_rest = (kopf & 0x7F) as usize + 1;
                } else {
                    roh_rest = kopf as usize + 1;
                }
            }
        }
    } else {
        leser.lesen(&mut roh);
    }

    // Wo liegt das Pixel (x, y) im Strom? Zeile 0 der Datei ist unten,
    // ausser der Deskriptor sagt "Ursprung oben".
    let stelle = move |x: usize, y: usize| {
        let zy = if von_oben { y } else { h - 1 - y };
        (zy * w + x) * bpp
    };

    if indiziert {
        // Graustufen: gemessen 256 Eintraege, nicht die Laenge der Datei.
        let keil: Vec<Color>;
        let farben = if grau {
            keil = (0..256).map(|v| Color::rgb(v as u8, v as u8, v as u8)).collect();
            &keil
        } else {
            &tafel
        };
        let mut s = tafel_flaeche(w as u32, h as u32, farben, |x, y| roh[stelle(x, y)])?;
        if let Some(k) = schluessel {
            if let Some(&c) = tafel.get(k) {
                s.set_colorkey(Some(c));
            }
        }
        return Ok(s);
    }
    let mit_alpha = bpp == 4;
    let mut s = aus_pixeln_auch_leer(w as u32, h as u32, mit_alpha, |x, y| {
        let p = stelle(x, y);
        match bpp {
            2 => {
                let c = fuenf_bit_farbe(u16::from_le_bytes([roh[p], roh[p + 1]]));
                (c.r, c.g, c.b, 255)
            }
            3 => (roh[p + 2], roh[p + 1], roh[p], 255),
            _ => (roh[p + 2], roh[p + 1], roh[p], roh[p + 3]),
        }
    })?;
    // **16 Bit meldet sich als 15** - gemessen. SDL legt an dieser
    // Stelle RGB555 ab und gibt das eine Alphabit nicht weiter.
    if bpp == 2 {
        s.set_format(crate::surface::Format::standard(15, false).map_err(|e| {
            Fehler::Format(format!("TGA: 15-bit surface could not be created: {e}"))
        })?);
    }
    Ok(s)
}

/// Liest wie `SDL_RWread`: was da ist, und **ohne** Fehler, wenn es zu
/// wenig ist - der Rest des Ziels bleibt, wie er war.
struct Leser<'a> {
    daten: &'a [u8],
    stelle: usize,
}

impl Leser<'_> {
    fn lesen(&mut self, ziel: &mut [u8]) {
        let anfang = self.stelle.min(self.daten.len());
        let da = (self.daten.len() - anfang).min(ziel.len());
        ziel[..da].copy_from_slice(&self.daten[anfang..anfang + da]);
        self.stelle = anfang + da;
    }
}

/// Ein A1R5G5B5-Wort als Farbe - die fuenf Bit werden auf acht gespreizt.
fn fuenf_bit_farbe(v: u16) -> crate::color::Color {
    let auf8 = |f: u16| ((f << 3) | (f >> 2)) as u8;
    crate::color::Color::rgb(auf8((v >> 10) & 0x1F), auf8((v >> 5) & 0x1F), auf8(v & 0x1F))
}

/// Baut eine 8-Bit-Flaeche mit Tafel aus Indizes.
fn tafel_flaeche(
    w: u32,
    h: u32,
    tafel: &[crate::color::Color],
    hole: impl Fn(usize, usize) -> u8,
) -> Ergebnis<Surface> {
    let mut s = Surface::new((w, h));
    s.set_format(crate::surface::Format::standard(8, false).expect("8 Bit ist gueltig"));
    // Erst die Tafel, dann die Pixel - siehe `image::aus_indizes`.
    s.set_palette(Some(tafel.to_vec()));
    // **Die rohen Indizes, wie SDL_image sie ablegt.** Hier stand
    // `set_at(tafel[nr.min(len - 1)])`: ueber die Farbe zurueck zum Index
    // trifft bei doppelten Farben einen anderen Eintrag, und ein Index
    // jenseits der Tafel wurde auf den letzten geklemmt. Bei SDL bleibt er
    // stehen - `get_at` gibt dort Schwarz, `get_at_mapped` den Index.
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            s.setz_gepackt((x, y), u32::from(hole(x as usize, y as usize)));
        }
    }
    Ok(s)
}

/// Schreibt TGA - zeilengetreu `SaveTGA_RW` aus `src_c/image.c`.
///
/// pygame ruft den Schreiber **immer** mit `rle = 1`; nachgemessen an
/// einer 1x1-Flaeche, die als 22 Bytes herauskommt (18 Kopf, dann `0x80`
/// und drei Bytes BGR).
///
/// Die Regeln aus der Vorlage:
///
/// ```text
/// 8 Bit   -> Art 1, Farbtafel mit 24 Bit - oder 32, wenn eine
///            Schluesselfarbe gesetzt ist (ihr Eintrag bekommt Alpha 0)
/// sonst   -> Art 2, 32 Bit bei Alphakanal, sonst 24
/// Flaggen -> 0x20 (Ursprung oben) plus 8, wenn Alpha dabei ist
/// ```
pub fn tga_schreiben(surface: &Surface) -> Ergebnis<Vec<u8>> {
    let (w, h) = surface.get_size();
    if surface.get_bitsize() < 8 {
        return Err(Fehler::Format("cannot save <8bpp images as TGA".into()));
    }
    let tafel = surface.get_palette().map(|p| p.to_vec());
    let schluessel = surface.get_colorkey();
    let mit_alpha = surface.has_alpha();
    let acht = surface.get_bitsize() == 8 && tafel.is_some();

    let (art, tafel_bits, tafel_laenge, pixel_bits) = if acht {
        let n = tafel.as_ref().map(|t| t.len()).unwrap_or(0);
        (1u8, if schluessel.is_some() { 32u8 } else { 24 }, n, 8u8)
    } else if mit_alpha {
        (2u8, 0u8, 0usize, 32u8)
    } else {
        (2u8, 0u8, 0usize, 24u8)
    };
    let bpp = (pixel_bits >> 3) as usize;

    let mut aus = Vec::new();
    aus.push(0u8); // Kennungslaenge
    aus.push(u8::from(acht)); // Farbtafel vorhanden
    aus.push(art + 8); // **immer** lauflaengenkodiert
    aus.extend_from_slice(&0u16.to_le_bytes()); // erster Tafeleintrag
    aus.extend_from_slice(&(tafel_laenge as u16).to_le_bytes());
    aus.push(tafel_bits);
    aus.extend_from_slice(&0u16.to_le_bytes()); // x-Ursprung
    aus.extend_from_slice(&0u16.to_le_bytes()); // y-Ursprung
    aus.extend_from_slice(&(w as u16).to_le_bytes());
    aus.extend_from_slice(&(h as u16).to_le_bytes());
    aus.push(pixel_bits);
    aus.push(0x20 | if pixel_bits == 32 { 8 } else { 0 });

    if acht {
        let t = tafel.as_deref().unwrap_or(&[]);
        let schluessel_index = schluessel
            .and_then(|k| t.iter().position(|c| c.r == k.r && c.g == k.g && c.b == k.b));
        for (n, c) in t.iter().enumerate() {
            aus.extend_from_slice(&[c.b, c.g, c.r]);
            if tafel_bits == 32 {
                aus.push(if Some(n) == schluessel_index { 0 } else { 0xFF });
            }
        }
    }

    // Zeile fuer Zeile kodieren - Laeufe reichen nie ueber das Zeilenende.
    let mut zeile = vec![0u8; w as usize * bpp];
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let ab = x as usize * bpp;
            if acht {
                zeile[ab] = surface.naechster_tafelindex((x, y)) as u8;
            } else {
                let c = surface.get_at((x, y));
                zeile[ab] = c.b;
                zeile[ab + 1] = c.g;
                zeile[ab + 2] = c.r;
                if bpp == 4 {
                    zeile[ab + 3] = c.a;
                }
            }
        }
        rle_zeile(&zeile, w as usize, bpp, &mut aus);
    }
    Ok(aus)
}

/// `rle_line` aus `src_c/image.c`, Zeile fuer Zeile uebertragen.
///
/// Die eine Feinheit steht im Original als Kommentar: ein Wiederholblock
/// lohnt erst, **wenn er zwei Bytes oder mehr spart** - sonst wird roh
/// geschrieben. Am Zeilenende (`x == w`) wird der Block trotzdem
/// geschlossen, und genau deshalb kommt eine 1x1-Flaeche als `0x80` plus
/// drei Bytes heraus und nicht als Rohblock.
fn rle_zeile(quelle: &[u8], w: usize, bpp: usize, aus: &mut Vec<u8>) {
    const MAX: usize = 128;
    let gleich =
        |a: usize, b: usize| quelle[a * bpp..(a + 1) * bpp] == quelle[b * bpp..(b + 1) * bpp];
    let mut x = 0usize;
    let mut roh = 0usize;
    while x < w {
        let mut x0 = x;
        x += 1;
        while x < w && gleich(x0, x) && x - x0 < MAX {
            x += 1;
        }
        if (x - x0 - 1) * bpp >= 2 || x == w {
            if x - x0 == 1 {
                x0 += 1;
            }
            while roh < x0 {
                let n = MAX.min(x0 - roh);
                aus.push((n - 1) as u8);
                aus.extend_from_slice(&quelle[roh * bpp..(roh + n) * bpp]);
                roh += n;
            }
            if x > x0 {
                aus.push((0x7F + x - x0) as u8);
                aus.extend_from_slice(&quelle[x0 * bpp..(x0 + 1) * bpp]);
            }
            roh = x;
        }
    }
}

// ---------------------------------------------------------------------------
//  TIFF
// ---------------------------------------------------------------------------

/// Liest TIFF.
///
/// **Immer 32 Bit mit `SRCALPHA`**, auch wenn die Datei keinen Alphakanal
/// hat - gemessen an `turquoise.tif` und `peppers3.tif`, beide melden in
/// pygame Tiefe 32 und Flaggen `0x10000`. Das ist anders als bei WebP und
/// JPEG, wo eine Datei ohne Alpha 24 Bit ergibt; SDL_image setzt TIFF
/// grundsaetzlich in RGBA um.
pub fn tiff_laden(daten: &[u8]) -> Ergebnis<Surface> {
    use tiff::decoder::DecodingResult;
    let mut dek = tiff::decoder::Decoder::new(std::io::Cursor::new(daten))
        .map_err(|e| Fehler::Format(format!("TIFF could not be read: {e}")))?;
    let (w, h) = dek
        .dimensions()
        .map_err(|e| Fehler::Format(format!("TIFF without a size: {e}")))?;
    let farbart = match dek.colortype() {
        Ok(f) => f,
        // **Mit Farbtafel liest die Kiste nicht.** `tiff` 0.11 lehnt
        // `Photometric interpretation RGBPalette` ab, und zwar schon bei
        // der Frage nach dem Farbraum. Das Format selbst ist an der
        // Stelle aber einfach: Indizes plus eine Tafel im Kopf. Also
        // wird genau dieser Fall von Hand gelesen - siehe
        // [`tiff_mit_tafel`]; alles andere bleibt bei der Kiste.
        Err(e) => {
            return tiff_mit_tafel(daten)
                .ok_or_else(|| Fehler::Format(format!("TIFF could not be read: no color type: {e}")))
        }
    };
    let bild = dek
        .read_image()
        .map_err(|e| Fehler::Format(format!("TIFF could not be read: {e}")))?;

    // Alles auf acht Bit je Kanal bringen; TIFF kennt auch 16 und 32.
    let acht: Vec<u8> = match bild {
        DecodingResult::U8(v) => v,
        DecodingResult::U16(v) => v.iter().map(|&x| (x >> 8) as u8).collect(),
        DecodingResult::U32(v) => v.iter().map(|&x| (x >> 24) as u8).collect(),
        DecodingResult::F32(v) => v
            .iter()
            .map(|&x| (x.clamp(0.0, 1.0) * 255.0).round() as u8)
            .collect(),
        anderes => {
            return Err(Fehler::Format(format!(
                "TIFF with an unsupported sample format: {anderes:?}"
            )))
        }
    };

    use tiff::ColorType;
    let kanaele = match farbart {
        ColorType::Gray(_) => 1,
        ColorType::GrayA(_) => 2,
        ColorType::RGB(_) => 3,
        ColorType::RGBA(_) | ColorType::CMYK(_) => 4,
        anderes => {
            return Err(Fehler::Format(format!(
                "TIFF in an unsupported color space: {anderes:?}"
            )))
        }
    };
    let cmyk = matches!(farbart, ColorType::CMYK(_));
    let zeile = w as usize * kanaele;
    if acht.len() < zeile * h as usize {
        return Err(Fehler::Format("TIFF could not be read: not enough image data".into()));
    }
    aus_pixeln(w, h, true, |x, y| {
        let i = y * zeile + x * kanaele;
        match (kanaele, cmyk) {
            (1, _) => (acht[i], acht[i], acht[i], 255),
            (2, _) => (acht[i], acht[i], acht[i], acht[i + 1]),
            (3, _) => (acht[i], acht[i + 1], acht[i + 2], 255),
            (_, true) => {
                // CMYK: der vierte Kanal ist Schwarz, kein Alpha.
                let k = acht[i + 3] as u32;
                let f = |c: u8| ((c as u32 * k) / 255) as u8;
                (f(acht[i]), f(acht[i + 1]), f(acht[i + 2]), 255)
            }
            _ => (acht[i], acht[i + 1], acht[i + 2], acht[i + 3]),
        }
    })
}

// ---------------------------------------------------------------------------
//  PNM - die Familie PBM, PGM, PPM
// ---------------------------------------------------------------------------

/// Liest PBM, PGM und PPM - beide Schreibweisen, `P1` bis `P6`.
///
/// **Welche Tiefe herauskommt, haengt an der Sorte**, und das ist
/// gemessen:
///
/// ```text
/// P1/P4  Schwarzweiss  ->  8 Bit, Tafel **2**: [weiss, schwarz]
/// P2/P5  Graustufen    ->  8 Bit, Tafel **256**, ein Graukeil
/// P3/P6  Farbe         -> 24 Bit, ohne Tafel
/// ```
///
/// Die erste Zeile ist die ueberraschende: bei PBM heisst das Bit `0`
/// **weiss**, nicht schwarz, und pygames Tafel steht genau so herum -
/// `white.pbm` gibt an `(0, 0)` reines Weiss.
///
/// Die vier Beispieldateien decken alle Schreibweisen ab: `black.ppm` ist
/// `P3` (Text), `grey.pgm` ist `P2` (Text), `white.pbm` ist `P4` (Bytes)
/// und `crimson.pnm` ist `P6` (Bytes).
pub fn pnm_laden(daten: &[u8]) -> Ergebnis<Surface> {
    // ## SDL_images Saetze und Grenzen (`IMG_pnm.c`), gemessen
    //
    // ```text
    // Breite/Hoehe fehlt oder 0     Unable to read image width and height
    // Hoechstwert fehlt, 0, >255    unsupported PNM format
    // Daten zu kurz                 file truncated
    // ```
    //
    // **Kein 16-Bit-PNM**: SDL_image lehnt jeden Hoechstwert ueber 255 ab.
    // Hier wurden solche Bilder geladen - mehr als pygame.
    let unbekannt = || Fehler::Format("unsupported PNM format".into());
    let masse = || Fehler::Format("Unable to read image width and height".into());
    if daten.len() < 2 || daten[0] != b'P' {
        return Err(unbekannt());
    }
    let sorte = daten[1];
    if !(b'1'..=b'6').contains(&sorte) {
        return Err(unbekannt());
    }
    let mut leser = PnmLeser { daten, i: 2 };
    let w = leser.zahl().map_err(|_| masse())? as usize;
    let h = leser.zahl().map_err(|_| masse())? as usize;
    if w == 0 || h == 0 {
        return Err(masse());
    }
    // Schwarzweiss hat keinen Hoechstwert.
    let hoechst = if matches!(sorte, b'1' | b'4') {
        1
    } else {
        leser.zahl().map_err(|_| unbekannt())? as usize
    };
    if hoechst == 0 || hoechst > 255 {
        return Err(unbekannt());
    }
    // Nach dem letzten Kopfwert steht **genau ein** Trennzeichen, dann
    // beginnen bei P4/P5/P6 die Bytes.
    let binaer = matches!(sorte, b'4' | b'5' | b'6');
    if binaer {
        leser.i += 1;
    }
    // Einen Wert auf 0..255 bringen - **wie SDL**: erst auf ein Byte
    // gekuerzt (`Uint8`), dann `* 255 / maxval` ohne Rundung und wieder in
    // ein Byte. Gemessen bei Hoechstwert 15: 15 wird 255, 32 wird
    // `544 & 0xFF` = 32. Hier stand eine gerundete und geklemmte Rechnung.
    let auf8 = move |v: usize| {
        let b = v as u8;
        if hoechst < 255 {
            (u32::from(b) * 255 / hoechst as u32) as u8
        } else {
            b
        }
    };
    let gekuerzt = |_: Fehler| Fehler::Format("file truncated".into());

    match sorte {
        // Schwarzweiss.
        b'1' | b'4' => {
            let tafel = vec![
                crate::color::Color::rgb(255, 255, 255),
                crate::color::Color::rgb(0, 0, 0),
            ];
            let mut bits = Vec::with_capacity(w * h);
            if sorte == b'1' {
                for _ in 0..w * h {
                    bits.push(leser.zahl().map_err(gekuerzt)? as u8 & 1);
                }
            } else {
                // Jede Zeile ist auf ganze Bytes aufgefuellt.
                let zeile = w.div_ceil(8);
                if leser.i + zeile * h > daten.len() {
                    return Err(Fehler::Format("file truncated".into()));
                }
                for y in 0..h {
                    for x in 0..w {
                        let b = daten[leser.i + y * zeile + x / 8];
                        bits.push((b >> (7 - x % 8)) & 1);
                    }
                }
            }
            tafel_flaeche(w as u32, h as u32, &tafel, |x, y| bits[y * w + x])
        }
        // Graustufen.
        b'2' | b'5' => {
            let keil: Vec<crate::color::Color> = (0..256)
                .map(|v| crate::color::Color::rgb(v as u8, v as u8, v as u8))
                .collect();
            let mut werte = Vec::with_capacity(w * h);
            for _ in 0..w * h {
                let v = if sorte == b'2' {
                    leser.zahl().map_err(gekuerzt)? as usize
                } else {
                    leser.byte(false).map_err(gekuerzt)?
                };
                werte.push(auf8(v));
            }
            tafel_flaeche(w as u32, h as u32, &keil, |x, y| werte[y * w + x])
        }
        // Farbe.
        _ => {
            let mut rgb = Vec::with_capacity(w * h * 3);
            for _ in 0..w * h * 3 {
                let v = if sorte == b'3' {
                    leser.zahl().map_err(gekuerzt)? as usize
                } else {
                    leser.byte(false).map_err(gekuerzt)?
                };
                rgb.push(auf8(v));
            }
            aus_pixeln(w as u32, h as u32, false, |x, y| {
                let i = (y * w + x) * 3;
                (rgb[i], rgb[i + 1], rgb[i + 2], 255)
            })
        }
    }
}

/// Liest den Kopf und die Textwerte eines PNM.
///
/// Kommentare beginnen mit `#` und reichen bis zum Zeilenende - und sie
/// duerfen **mitten im Kopf** stehen. Alle vier Beispieldateien haben
/// gleich hinter der Kennung eine GIMP-Zeile.
struct PnmLeser<'a> {
    daten: &'a [u8],
    i: usize,
}

impl PnmLeser<'_> {
    /// Die naechste Zahl, Kommentare und Leerraum uebergehend.
    fn zahl(&mut self) -> Ergebnis<u32> {
        loop {
            while self.i < self.daten.len() && self.daten[self.i].is_ascii_whitespace() {
                self.i += 1;
            }
            if self.i < self.daten.len() && self.daten[self.i] == b'#' {
                while self.i < self.daten.len() && self.daten[self.i] != b'\n' {
                    self.i += 1;
                }
                continue;
            }
            break;
        }
        let ab = self.i;
        while self.i < self.daten.len() && self.daten[self.i].is_ascii_digit() {
            self.i += 1;
        }
        if ab == self.i {
            return Err(Fehler::Format("PNM: expected a number".into()));
        }
        std::str::from_utf8(&self.daten[ab..self.i])
            .ok()
            .and_then(|t| t.parse().ok())
            .ok_or_else(|| Fehler::Format("PNM: unreadable number".into()))
    }

    /// Ein Wert aus dem Byte-Teil - ein oder zwei Bytes, gross zuerst.
    fn byte(&mut self, breit: bool) -> Ergebnis<usize> {
        let n = if breit { 2 } else { 1 };
        if self.i + n > self.daten.len() {
            return Err(Fehler::Format("file truncated".into()));
        }
        let v = if breit {
            ((self.daten[self.i] as usize) << 8) | self.daten[self.i + 1] as usize
        } else {
            self.daten[self.i] as usize
        };
        self.i += n;
        Ok(v)
    }
}

// ---------------------------------------------------------------------------
//  QOI
// ---------------------------------------------------------------------------

/// Liest QOI - das "Quite OK Image"-Format.
///
/// **Immer 32 Bit mit `SRCALPHA`**, auch wenn der Kopf drei Kanaele
/// nennt: gemessen an `purple.qoi`, das `channels = 3` traegt und in
/// pygame trotzdem als 32 Bit mit `SRCALPHA` ankommt.
///
/// Das ganze Format sind fuenf Blockarten und eine Streutabelle mit 64
/// Plaetzen; deshalb steht es hier von Hand und nicht als Abhaengigkeit.
pub fn qoi_laden(daten: &[u8]) -> Ergebnis<Surface> {
    // `qoi_decode` gibt bei **jedem** Fehler NULL zurueck, und
    // `IMG_LoadQOI_RW` sagt dann `Couldn't parse QOI image` - gemessen an
    // einer Breite 0 und an zu kurzen Daten.
    if daten.len() < 14 || &daten[0..4] != b"qoif" {
        return Err(Fehler::Format("Couldn't parse QOI image".into()));
    }
    let w = u32::from_be_bytes([daten[4], daten[5], daten[6], daten[7]]);
    let h = u32::from_be_bytes([daten[8], daten[9], daten[10], daten[11]]);
    if w == 0 || h == 0 {
        return Err(Fehler::Format("Couldn't parse QOI image".into()));
    }
    let anzahl = w as usize * h as usize;
    let mut aus: Vec<[u8; 4]> = Vec::with_capacity(anzahl);
    // Der Vorgaenger beginnt als undurchsichtiges Schwarz.
    let mut vorher = [0u8, 0, 0, 255];
    let mut tafel = [[0u8; 4]; 64];
    let streu = |p: [u8; 4]| {
        (p[0] as usize * 3 + p[1] as usize * 5 + p[2] as usize * 7 + p[3] as usize * 11) % 64
    };
    let mut i = 14usize;
    while aus.len() < anzahl && i < daten.len() {
        let b = daten[i];
        i += 1;
        match b {
            // Ganze Farbe ohne Alpha.
            0xFE => {
                if i + 3 > daten.len() {
                    break;
                }
                vorher = [daten[i], daten[i + 1], daten[i + 2], vorher[3]];
                i += 3;
            }
            // Ganze Farbe mit Alpha.
            0xFF => {
                if i + 4 > daten.len() {
                    break;
                }
                vorher = [daten[i], daten[i + 1], daten[i + 2], daten[i + 3]];
                i += 4;
            }
            _ => match b >> 6 {
                // Ein Platz aus der Streutabelle.
                0 => vorher = tafel[(b & 0x3F) as usize],
                // Kleiner Unterschied, je zwei Bit, mit Versatz 2.
                1 => {
                    let dr = ((b >> 4) & 3) as i16 - 2;
                    let dg = ((b >> 2) & 3) as i16 - 2;
                    let db = (b & 3) as i16 - 2;
                    vorher = [
                        (vorher[0] as i16 + dr) as u8,
                        (vorher[1] as i16 + dg) as u8,
                        (vorher[2] as i16 + db) as u8,
                        vorher[3],
                    ];
                }
                // Groesserer Unterschied, an Gruen gemessen.
                2 => {
                    if i >= daten.len() {
                        break;
                    }
                    let zweites = daten[i];
                    i += 1;
                    let dg = (b & 0x3F) as i16 - 32;
                    let dr = dg + ((zweites >> 4) as i16 - 8);
                    let db = dg + ((zweites & 0x0F) as i16 - 8);
                    vorher = [
                        (vorher[0] as i16 + dr) as u8,
                        (vorher[1] as i16 + dg) as u8,
                        (vorher[2] as i16 + db) as u8,
                        vorher[3],
                    ];
                }
                // Eine Wiederholung, Laenge mit Versatz 1.
                _ => {
                    let n = (b & 0x3F) as usize + 1;
                    for _ in 0..n {
                        if aus.len() == anzahl {
                            break;
                        }
                        aus.push(vorher);
                    }
                    continue;
                }
            },
        }
        tafel[streu(vorher)] = vorher;
        aus.push(vorher);
    }
    if aus.len() < anzahl {
        return Err(Fehler::Format("Couldn't parse QOI image".into()));
    }
    aus_pixeln(w, h, true, |x, y| {
        let p = aus[y * w as usize + x];
        (p[0], p[1], p[2], p[3])
    })
}

// ---------------------------------------------------------------------------
//  PCX
// ---------------------------------------------------------------------------

/// Liest PCX - lauflaengenkodiert, mit Bitebenen.
///
/// Gemessen an `green.pcx`: acht Bit je Ebene, drei Ebenen, also **24
/// Bit**. Ein PCX mit **einer** Ebene und acht Bit traegt seine Farbtafel
/// hinten an (768 Bytes hinter einer `0x0C`), eines mit weniger Bit die
/// EGA-Tafel im Kopf.
pub fn pcx_laden(daten: &[u8]) -> Ergebnis<Surface> {
    if daten.len() < 128 || daten[0] != 0x0A {
        return Err(Fehler::Format("Unsupported image format".into()));
    }
    let le16 = |i: usize| u16::from_le_bytes([daten[i], daten[i + 1]]) as i32;
    let kodiert = daten[2] == 1;
    let bits = daten[3] as usize;
    let (xmin, ymin, xmax, ymax) = (le16(4), le16(6), le16(8), le16(10));
    let ebenen = daten[65] as usize;
    let zeilenbytes = le16(66) as usize;
    // Die Groesse geht an `SDL_CreateRGBSurface`: negativ ist dort ein
    // Fehler mit SDLs Satz, **null ist erlaubt** - gemessen gibt pygame
    // fuer `xmax = xmin - 1` eine Flaeche (0, 4).
    let (w, h) = (xmax - xmin + 1, ymax - ymin + 1);
    if w < 0 {
        return Err(Fehler::Format("Parameter 'width' is invalid".into()));
    }
    if h < 0 {
        return Err(Fehler::Format("Parameter 'height' is invalid".into()));
    }
    let (w, h) = (w as usize, h as usize);
    // `IMG_LoadPCX_RW` kennt **drei** Arten: 1 Bit mit 1 bis 4 Ebenen und
    // 8 Bit mit einer Ebene (beide mit Tafel), dazu 8 Bit mit drei Ebenen
    // (Echtfarbe). Gemessen sind 2 und 4 Bit mit einer Ebene sowie 8 Bit
    // mit 2 oder 4 Ebenen `unsupported PCX format`; hier wurden sie
    // geladen.
    let tafelbild = (bits == 1 && (1..=4).contains(&ebenen)) || (bits == 8 && ebenen == 1);
    if !tafelbild && !(bits == 8 && ebenen == 3) {
        return Err(Fehler::Format("unsupported PCX format".into()));
    }

    // Die Bilddaten auspacken: je Zeile `ebenen * zeilenbytes` Bytes.
    let gesamt = zeilenbytes * ebenen * h;
    let mut roh = Vec::with_capacity(gesamt);
    let mut i = 128usize;
    while roh.len() < gesamt && i < daten.len() {
        let b = daten[i];
        i += 1;
        if kodiert && b & 0xC0 == 0xC0 {
            if i >= daten.len() {
                break;
            }
            let n = (b & 0x3F) as usize;
            let wert = daten[i];
            i += 1;
            for _ in 0..n {
                roh.push(wert);
            }
        } else {
            roh.push(b);
        }
    }
    // **Abgeschnitten ist ein Fehler.** `IMG_LoadPCX_RW` liest Zeile fuer
    // Zeile und sagt `file truncated`, sobald ein Byte fehlt - gemessen an
    // einem rohen und einem lauflaengenkodierten Bild. Hier wurde mit
    // Nullen aufgefuellt und geladen.
    if roh.len() < gesamt {
        return Err(Fehler::Format("file truncated".into()));
    }
    roh.truncate(gesamt);

    if ebenen == 3 && bits == 8 {
        // Echtfarbe: drei Ebenen je Zeile, erst alles Rot, dann Gruen, dann Blau.
        return aus_pixeln_auch_leer(w as u32, h as u32, false, |x, y| {
            let z = y * zeilenbytes * 3;
            (
                roh[z + x],
                roh[z + zeilenbytes + x],
                roh[z + 2 * zeilenbytes + x],
                255,
            )
        });
    }

    // ## Die Farbtafel, wie `IMG_LoadPCX_RW` sie sucht
    //
    // Sie hat `1 << (Bits * Ebenen)` Eintraege - gemessen 2, 4 und 16 fuer
    // 1 Bit mit 1, 2 und 4 Ebenen; hier waren es immer 16.
    //
    // Bei 8 Bit liest SDL **hinter den Bilddaten** Byte fuer Byte, bis es
    // die 12 findet, und nimmt die 768 Bytes danach. Findet es keine,
    // springt es auf 768 vor dem Ende - und weil pygames Strom einen
    // Sprung vor den Anfang beim Anfang festhaelt, liest eine kurze Datei
    // ihre Tafel dann **aus dem Kopf**. Was danach fehlt, ist bei pygame
    // ein nicht vorbelegter Stapelspeicher; hier steht dort null.
    // Hier galt nur `daten[len - 769] == 12`, sonst die 16 Kopffarben.
    let anzahl = 1usize << (bits * ebenen);
    let farbe = |c: &[u8; 3]| crate::color::Color::rgb(c[0], c[1], c[2]);
    let tafel: Vec<crate::color::Color> = if anzahl == 256 {
        let ab = i.min(daten.len());
        let start = daten[ab..]
            .iter()
            .position(|&b| b == 12)
            .map(|p| ab + p + 1)
            .unwrap_or(daten.len().saturating_sub(768));
        let mut roh_tafel = [0u8; 768];
        let da = daten.len().saturating_sub(start).min(768);
        roh_tafel[..da].copy_from_slice(&daten[start..start + da]);
        roh_tafel.as_chunks::<3>().0.iter().map(farbe).collect()
    } else {
        daten[16..16 + 3 * anzahl].as_chunks::<3>().0.iter().map(farbe).collect()
    };
    tafel_flaeche(w as u32, h as u32, &tafel, move |x, y| {
        let z = y * zeilenbytes * ebenen;
        if bits == 8 {
            roh[z + x]
        } else {
            let mut nr = 0u8;
            for e in 0..ebenen {
                let b = roh[z + e * zeilenbytes + x * bits / 8];
                let versatz = 8 - bits - (x * bits) % 8;
                let teil = (b >> versatz) & ((1u16 << bits) - 1) as u8;
                nr |= teil << (e * bits);
            }
            nr
        }
    })
}

// ---------------------------------------------------------------------------
//  XPM
// ---------------------------------------------------------------------------

/// Liest XPM - und das ist **C-Quelltext**, kein Binaerformat.
///
/// ```c
/// /* XPM */
/// static char * bild[] = {
/// "32 32 1 1",        /* Breite, Hoehe, Farben, Zeichen je Pixel */
/// "  c #FF00FF",      /* ein Zeichen und seine Farbe */
/// "                ", /* die Zeilen */
/// ...
/// ```
///
/// Gelesen werden also die Zeichenketten zwischen den Anfuehrungszeichen.
/// Gemessen an `purple.xpm`: 8 Bit, Tafel **so lang wie die Farbliste**
/// (dort genau 1), nicht auf 256 aufgefuellt.
pub fn xpm_laden(daten: &[u8]) -> Ergebnis<Surface> {
    let text = String::from_utf8_lossy(daten);
    // SDL_images Saetze (`IMG_xpm.c`), gemessen: ohne jede Zeile und mit
    // zu wenigen Zeilen `Premature end of data`, mit einer Kopfzeile, die
    // keine vier Zahlen hergibt, `Invalid format description`.
    let zu_kurz = || Fehler::Format("Premature end of data".into());
    let kopf_falsch = || Fehler::Format("Invalid format description".into());
    if !text.contains("XPM") {
        return Err(Fehler::Format("Unsupported image format".into()));
    }
    // Alle Zeichenketten einsammeln. Maskierungen kommen in XPM nicht vor.
    let mut stuecke: Vec<String> = Vec::new();
    let mut offen: Option<String> = None;
    for z in text.chars() {
        match (&mut offen, z) {
            (None, '"') => offen = Some(String::new()),
            (Some(s), '"') => {
                stuecke.push(std::mem::take(s));
                offen = None;
            }
            (Some(s), c) => s.push(c),
            _ => {}
        }
    }
    let kopf = stuecke
        .first()
        .ok_or_else(zu_kurz)?;
    let zahlen: Vec<usize> = kopf
        .split_whitespace()
        .filter_map(|t| t.parse().ok())
        .collect();
    if zahlen.len() < 4 {
        return Err(kopf_falsch());
    }
    let (w, h, farben, je_pixel) = (zahlen[0], zahlen[1], zahlen[2], zahlen[3]);
    if w == 0 || h == 0 || farben == 0 || je_pixel == 0 || je_pixel > 4 {
        return Err(kopf_falsch());
    }
    if stuecke.len() < 1 + farben + h {
        return Err(zu_kurz());
    }

    // Die Farbliste: `<zeichen> c <farbe>`, dazu die Schluessel `m`, `g`,
    // `s` - genommen wird `c`, sonst die erste brauchbare Angabe.
    let mut schluessel: Vec<(String, crate::color::Color)> = Vec::new();
    for zeile in &stuecke[1..1 + farben] {
        if zeile.len() < je_pixel {
            continue;
        }
        let (zeichen, rest) = zeile.split_at(je_pixel);
        let worte: Vec<&str> = rest.split_whitespace().collect();
        let mut farbe = None;
        let mut n = 0;
        while n + 1 < worte.len() {
            let wert = worte[n + 1];
            if worte[n] == "c" {
                farbe = xpm_farbe(wert);
                break;
            }
            if farbe.is_none() {
                farbe = xpm_farbe(wert);
            }
            n += 2;
        }
        schluessel.push((
            zeichen.to_string(),
            farbe.unwrap_or(crate::color::Color::rgb(0, 0, 0)),
        ));
    }
    let tafel: Vec<crate::color::Color> = schluessel.iter().map(|(_, c)| *c).collect();
    if tafel.is_empty() {
        return Err(zu_kurz());
    }

    // Die Bildzeilen in Indizes umsetzen.
    let mut indizes = vec![0u8; w * h];
    for (y, zeile) in stuecke[1 + farben..1 + farben + h].iter().enumerate() {
        let z: Vec<char> = zeile.chars().collect();
        for x in 0..w {
            let ab = x * je_pixel;
            if ab + je_pixel > z.len() {
                break;
            }
            let marke: String = z[ab..ab + je_pixel].iter().collect();
            if let Some(n) = schluessel.iter().position(|(k, _)| *k == marke) {
                indizes[y * w + x] = n as u8;
            }
        }
    }
    tafel_flaeche(w as u32, h as u32, &tafel, move |x, y| indizes[y * w + x])
}

/// Eine XPM-Farbangabe: `#RGB`, `#RRGGBB`, `#RRRRGGGGBBBB` oder ein Name.
fn xpm_farbe(wert: &str) -> Option<crate::color::Color> {
    if let Some(hex) = wert.strip_prefix('#') {
        let je = match hex.len() {
            3 => 1,
            6 => 2,
            12 => 4,
            _ => return None,
        };
        let teil = |n: usize| -> Option<u8> {
            let s = &hex[n * je..(n + 1) * je];
            let v = u32::from_str_radix(s, 16).ok()?;
            // Auf acht Bit bringen: bei einer Ziffer verdoppeln, bei
            // vieren die oberen zwei Stellen nehmen.
            Some(match je {
                1 => (v * 17) as u8,
                2 => v as u8,
                _ => (v >> 8) as u8,
            })
        };
        return Some(crate::color::Color::rgb(teil(0)?, teil(1)?, teil(2)?));
    }
    if wert.eq_ignore_ascii_case("none") {
        return Some(crate::color::Color::rgb(0, 0, 0));
    }
    crate::color::by_name(wert)
}

// ---------------------------------------------------------------------------
//  LBM - IFF ILBM und PBM
// ---------------------------------------------------------------------------

/// Liest LBM/IFF - **ILBM** (Bitebenen) und **PBM** (ein Byte je Pixel).
///
/// Zeilengetreu `IMG_LoadLBM_RW` aus SDL_image 2.8.8 (`IMG_lbm.c`), auf
/// einem Lesestrom wie `SDL_RWops`. Bis Runde 19 stand hier ein Nachbau,
/// der eine Handvoll Faelle traf und den Rest verfehlte - gemessen an 37
/// gebauten Dateien waren 28 verschieden:
///
/// * Die Flaeche ist **auf 16 Pixel aufgerundet** breit
///   (`width = (bmhd.w + 15) & ~15`); ein 5 Pixel breites Bild laedt als
///   16 breit, mit den Fuellbits der Ebenen als Pixel.
/// * Ohne `CMAP` bleibt SDLs Vorgabetafel: **256 Mal Weiss**, kein Graukeil.
/// * `PBM ` (Chunky) fehlte ganz; seine Tafel wird nicht gekuerzt.
/// * Maske (`mask & 1`) ist eine **zusaetzliche Ebene**, die als hoechstes
///   Bit in den Index geht; `mask & 2` setzt `tcolor` als Schluesselfarbe.
/// * EHB (Extra-Half-Bright), HAM6/HAM8 und 24 Ebenen geben 24 Bit.
/// * `CAMG` ueberspringt seinen Rest **nicht** (`bytesloaded = size`), und
///   ByteRun1 nimmt `0x80` als Lauf von 129 statt als Leerzug - beides wie
///   in der Vorlage.
pub fn lbm_laden(daten: &[u8]) -> Ergebnis<Surface> {
    /// `SDL_RWops` ueber dem Speicher; `seek` hinter das Ende gelingt (wie
    /// bei einer Datei oder einem Python-Dateiobjekt), gelesen wird dort
    /// nichts mehr.
    struct Strom<'a> {
        d: &'a [u8],
        pos: usize,
    }
    impl Strom<'_> {
        fn lesen(&mut self, n: usize) -> Option<&[u8]> {
            if self.pos.checked_add(n).is_none_or(|e| e > self.d.len()) {
                self.pos = self.d.len().max(self.pos);
                return None;
            }
            let s = &self.d[self.pos..self.pos + n];
            self.pos += n;
            Some(s)
        }
    }
    let fehler = |s: &str| Fehler::Format(s.into());
    let mut s = Strom { d: daten, pos: 0 };

    let id = s.lesen(4).ok_or_else(|| fehler("error reading IFF chunk"))?.to_vec();
    s.lesen(4).ok_or_else(|| fehler("error reading IFF chunk size"))?;
    if id != b"FORM" {
        return Err(fehler("not a IFF file"));
    }
    let mut id = s.lesen(4).ok_or_else(|| fehler("error reading IFF chunk"))?.to_vec();
    let pbm = id == b"PBM ";
    if !pbm && id != b"ILBM" {
        return Err(fehler("not a IFF picture"));
    }

    // BMHD: w, h, x, y, planes, mask, tcomp, pad1, tcolor, xAspect,
    // yAspect, Lpage, Hpage - 20 Bytes.
    let (mut w, mut h, mut planes, mut mask, mut tcomp, mut tcolor) = (0u32, 0u32, 0u32, 0u8, 0u8, 0u32);
    let mut colormap = [0u8; 256 * 3];
    let mut nbcolors = 0usize;
    let (mut ham, mut ehb) = (false, false);
    while id != b"BODY" {
        id = s.lesen(4).ok_or_else(|| fehler("error reading IFF chunk"))?.to_vec();
        let g = s.lesen(4).ok_or_else(|| fehler("error reading IFF chunk size"))?;
        let mut size = u32::from_be_bytes([g[0], g[1], g[2], g[3]]);
        let mut bytesloaded = 0u32;
        if id == b"BMHD" {
            let b = s.lesen(20).ok_or_else(|| fehler("error reading BMHD chunk"))?;
            w = u32::from(u16::from_be_bytes([b[0], b[1]]));
            h = u32::from(u16::from_be_bytes([b[2], b[3]]));
            planes = u32::from(b[8]);
            mask = b[9];
            tcomp = b[10];
            tcolor = u32::from(u16::from_be_bytes([b[12], b[13]]));
            bytesloaded = 20;
        }
        if id == b"CMAP" {
            if size as usize > colormap.len() {
                return Err(fehler("colormap size is too large"));
            }
            let c = s.lesen(size as usize).ok_or_else(|| fehler("error reading CMAP chunk"))?;
            colormap[..c.len()].copy_from_slice(c);
            bytesloaded = size;
            nbcolors = size as usize / 3;
        }
        if id == b"CAMG" {
            let v = s.lesen(4).ok_or_else(|| fehler("error reading CAMG chunk"))?;
            let viewmodes = u32::from_be_bytes([v[0], v[1], v[2], v[3]]);
            // Die Vorlage setzt `bytesloaded = size` - ein laengerer
            // CAMG-Block wird damit **nicht** zu Ende uebersprungen.
            bytesloaded = size;
            ham = viewmodes & 0x0800 != 0;
            ehb = viewmodes & 0x0080 != 0;
        }
        if id != b"BODY" {
            if size & 1 != 0 {
                size = size.wrapping_add(1);
            }
            let rest = size.wrapping_sub(bytesloaded);
            if rest != 0 {
                s.pos = s.pos.saturating_add(rest as usize);
            }
        }
    }

    let width = (w + 15) & 0xFFFF_FFF0;
    let mut bytesperline = w.div_ceil(16) as usize * 2;
    let mut nbplanes = planes as usize;
    if pbm {
        bytesperline *= 8;
        nbplanes = 1;
    }
    let stencil = (mask & 1) as usize;
    let echtfarben = nbplanes == 24 || ham;
    let mut minibuf = vec![0u8; bytesperline * (nbplanes + stencil)];

    let mut bild = Surface::new((width, h));
    if !echtfarben {
        bild.set_format(crate::surface::Format::standard(8, false).expect("8 Bit ist gueltig"));
    } else {
        bild.set_format(crate::surface::Format::standard(24, false).expect("24 Bit ist gueltig"));
    }
    // SDLs Vorgabetafel einer neuen 8-Bit-Flaeche: 256 Mal Weiss.
    let mut tafel = vec![crate::color::Color::rgba(255, 255, 255, 255); 256];
    let mut tafellaenge = 256usize;
    if nbcolors > 0 && !ham && !echtfarben {
        let mut final_ = 1usize << (nbplanes + stencil).min(16);
        for (i, f) in tafel.iter_mut().enumerate().take(nbcolors.min(256)) {
            *f = crate::color::Color::rgb(colormap[3 * i], colormap[3 * i + 1], colormap[3 * i + 2]);
        }
        if (nbcolors == 32 || ehb) && (1usize << nbplanes.min(16)) == 64 {
            nbcolors = 64;
            for (q, f) in tafel[32..64].iter_mut().enumerate() {
                *f = crate::color::Color::rgb(
                    colormap[3 * q] / 2,
                    colormap[3 * q + 1] / 2,
                    colormap[3 * q + 2] / 2,
                );
            }
        }
        if final_ > 1 << nbplanes.min(16) {
            final_ = 1 << nbplanes.min(16);
        }
        for i in nbcolors..final_.min(256) {
            tafel[i] = tafel[i % nbcolors];
        }
        if !pbm {
            tafellaenge = final_.min(256);
        }
    }
    if !echtfarben {
        bild.set_palette(Some(tafel[..tafellaenge].to_vec()));
    }

    fn body(e: Option<&[u8]>) -> Ergebnis<&[u8]> {
        e.ok_or_else(|| Fehler::Format("error reading BODY chunk".into()))
    }
    for zeile in 0..h as usize {
        for plane in 0..(nbplanes + stencil) {
            let mut stelle = plane * bytesperline;
            let mut rest = bytesperline;
            if tcomp == 1 {
                loop {
                    let mut count = body(s.lesen(1))?[0];
                    if count & 0x80 != 0 {
                        count ^= 0xFF;
                        count = count.wrapping_add(2);
                        if count as usize > rest {
                            return Err(fehler("error reading BODY chunk"));
                        }
                        let farbe = body(s.lesen(1))?[0];
                        minibuf[stelle..stelle + count as usize].fill(farbe);
                    } else {
                        count = count.wrapping_add(1);
                        if count as usize > rest {
                            return Err(fehler("error reading BODY chunk"));
                        }
                        let roh = body(s.lesen(count as usize))?;
                        minibuf[stelle..stelle + count as usize].copy_from_slice(roh);
                    }
                    stelle += count as usize;
                    rest -= count as usize;
                    if rest == 0 {
                        break;
                    }
                }
            } else {
                let roh = body(s.lesen(bytesperline))?;
                minibuf[stelle..stelle + bytesperline].copy_from_slice(roh);
            }
        }
        let y = zeile as i32;
        if pbm {
            for (x, &b) in minibuf.iter().take(width as usize).enumerate() {
                bild.setz_gepackt((x as i32, y), u32::from(b));
            }
        } else if !echtfarben {
            let bytes = (width as usize).div_ceil(8);
            for i in 0..bytes {
                let mut px = [0u8; 8];
                for plane in 0..(nbplanes + stencil) {
                    let farbe = minibuf[i + plane * bytesperline];
                    let mut msk = 0x80u8;
                    for (j, p) in px.iter_mut().enumerate() {
                        let wert = u32::from(farbe & msk);
                        if plane + j <= 7 {
                            *p |= (wert >> (7 - plane - j)) as u8;
                        } else {
                            *p |= (wert << (plane + j - 7)) as u8;
                        }
                        msk >>= 1;
                    }
                }
                for (j, &p) in px.iter().enumerate() {
                    bild.setz_gepackt(((i * 8 + j) as i32, y), u32::from(p));
                }
            }
        } else {
            let size = (width as usize).div_ceil(8);
            let mut finalcolor: u32 = 0;
            let mut x = 0usize;
            while x < width as usize {
                let mut mask_bit = 0x80u8;
                for j in 0..8 {
                    let mut pixelcolor: u32 = 0;
                    let mut mask_color: u32 = 1;
                    for plane in 0..nbplanes {
                        let data_body = minibuf[plane * size + x / 8];
                        if data_body & mask_bit != 0 {
                            pixelcolor |= mask_color;
                        }
                        mask_color <<= 1;
                    }
                    if ham {
                        let pc = pixelcolor as usize;
                        match pixelcolor >> (nbplanes - 2) {
                            0 => {
                                finalcolor = u32::from(colormap[(pc * 3) % 768])
                                    + (u32::from(colormap[(pc * 3 + 1) % 768]) << 8)
                                    + (u32::from(colormap[(pc * 3 + 2) % 768]) << 16);
                            }
                            1 => {
                                finalcolor &= 0x00FFFF;
                                finalcolor |= pixelcolor << (16 + (10 - nbplanes));
                            }
                            2 => {
                                finalcolor &= 0xFFFF00;
                                finalcolor |= pixelcolor << (10 - nbplanes);
                            }
                            3 => {
                                finalcolor &= 0xFF00FF;
                                finalcolor |= pixelcolor << (8 + (10 - nbplanes));
                            }
                            _ => {}
                        }
                    } else {
                        finalcolor = pixelcolor;
                    }
                    // BGR24: Byte 0 ist `finalcolor >> 16`, Byte 2 das
                    // unterste - mit den Masken der 24-Bit-Flaeche also R
                    // im untersten Byte von `finalcolor`.
                    let r = (finalcolor & 0xFF) as u8;
                    let g = ((finalcolor >> 8) & 0xFF) as u8;
                    let b = ((finalcolor >> 16) & 0xFF) as u8;
                    bild.set_at(((x + j) as i32, y), crate::color::Color::rgb(r, g, b));
                    mask_bit >>= 1;
                }
                x += 8;
            }
        }
    }
    if mask & 2 != 0 {
        // `SDL_SetColorKey(Image, SDL_TRUE, bmhd.tcolor)` - ein Pixelwert.
        let k = bild.unmap_rgb_gepackt(tcolor);
        bild.set_colorkey(Some(k));
    }
    Ok(bild)
}

// ---------------------------------------------------------------------------
//  XV
// ---------------------------------------------------------------------------

/// `get_header` aus `IMG_xv.c`: `(breite, hoehe, beginn der Pixel)` oder
/// `None`.
///
/// Erste Zeile `P7 332`, dann Kommentare bis `#END_OF_COMMENTS`, dann
/// `breite hoehe`. Ein `#BUILTIN:` heisst "kein Bild". Zeilen enden an
/// `\n`, ein `\r` wird uebergangen, und mehr als 1024 Zeichen sind keine
/// Zeile.
pub(crate) fn xv_kopf(d: &[u8]) -> Option<(i64, i64, usize)> {
    let mut pos = 0usize;
    let mut zeile = || -> Option<Vec<u8>> {
        let mut z = Vec::new();
        loop {
            if z.len() >= 1024 {
                return None;
            }
            let b = *d.get(pos)?;
            pos += 1;
            match b {
                b'\r' => continue,
                b'\n' => return Some(z),
                _ => z.push(b),
            }
        }
    };
    let erste = zeile()?;
    if !erste.starts_with(b"P7 332") {
        return None;
    }
    while let Some(z) = zeile() {
        if z.starts_with(b"#BUILTIN:") {
            return None;
        }
        if z.starts_with(b"#END_OF_COMMENTS") {
            let masse = zeile()?;
            // `sscanf(line, "%d %d", w, h)` - was nicht gelesen wird, bleibt 0.
            let text = String::from_utf8_lossy(&masse);
            let mut zahlen = text.split_whitespace().map(|t| {
                let ende = t
                    .char_indices()
                    .find(|&(i, c)| !(c.is_ascii_digit() || (i == 0 && (c == '-' || c == '+'))))
                    .map_or(t.len(), |(i, _)| i);
                t[..ende].parse::<i64>().ok()
            });
            let erste = zahlen.next().flatten();
            let w = erste.unwrap_or(0);
            let h = if erste.is_some() { zahlen.next().flatten().unwrap_or(0) } else { 0 };
            if w >= 0 && h >= 0 {
                return Some((w, h, pos));
            }
            return None;
        }
    }
    None
}

/// Liest ein XV-Vorschaubild - `IMG_LoadXV_RW` aus SDL_image 2.8.8.
///
/// Die Flaeche ist **RGB332**: acht Bit, gepackt, ohne Tafel. Bis Runde 19
/// kannte rustgame das Format nicht (`Unsupported image format`).
pub fn xv_laden(d: &[u8]) -> Ergebnis<Surface> {
    let (w, h, mut pos) = xv_kopf(d).ok_or_else(|| Fehler::Format("Unsupported image format".into()))?;
    let format = crate::surface::Format::aus_masken(8, (0xE0, 0x1C, 0x03, 0))
        .map_err(|e| Fehler::Format(e.into()))?;
    let mut s = Surface::new((w as u32, h as u32));
    s.set_format(format);
    for y in 0..h as i32 {
        // `SDL_RWread(src, pixels, w, 1)` - eine Breite 0 liest nichts und
        // gilt als Fehler.
        let zeile = d
            .get(pos..pos + w as usize)
            .filter(|_| w > 0)
            .ok_or_else(|| Fehler::Format("Couldn't read image data".into()))?;
        for (x, &b) in zeile.iter().enumerate() {
            s.setz_gepackt((x as i32, y), u32::from(b));
        }
        pos += w as usize;
    }
    Ok(s)
}

// ---------------------------------------------------------------------------
//  ICO und CUR
// ---------------------------------------------------------------------------

/// Liest ein Windows-Symbol (`art` 1) oder einen Mauszeiger (`art` 2) -
/// zeilengetreu `LoadICOCUR_RW` aus SDL_image 2.8.8 (`IMG_bmp.c`).
///
/// Bis Runde 19 fehlte das ganz: jede `.ico` und `.cur` gab `Unsupported
/// image format`, gemessen an zwoelf Dateien aus Windows. Die Eigenheiten
/// der Vorlage, alle uebernommen:
///
/// * Gewaehlt wird der Eintrag mit der **groessten Farbzahl** (0 heisst
///   256; bei Gleichstand der erste) - nicht das groesste Bild.
/// * Ein Eintrag mit PNG darin ist `Unsupported ICO bitmap format`.
/// * Die Flaeche ist ARGB8888 und **halb so hoch** wie `biHeight` (die
///   zweite Haelfte ist die UND-Maske), und die Maske setzt Pixel auf 0.
/// * Tafeleintraege mit Alpha 0 werden deckend; 32 Bit liest das Alpha aus
///   der Datei, wie es dasteht.
pub fn ico_cur_laden(daten: &[u8], art: u16) -> Ergebnis<Surface> {
    let fehler = |s: &str| Fehler::Format(s.into());
    let mut pos = 0usize;
    let mut lies = |n: usize| -> Option<u64> {
        let b = daten.get(pos..pos + n);
        pos += n;
        b.map(|b| b.iter().rev().fold(0u64, |a, &x| (a << 8) | u64::from(x)))
    };
    let reserviert = lies(2).unwrap_or(0);
    let typ = lies(2).unwrap_or(0);
    let anzahl = lies(2).unwrap_or(0);
    if reserviert != 0 || typ != u64::from(art) || anzahl == 0 {
        return Err(Fehler::Format(format!(
            "File is not a Windows {} file",
            if art == 1 { "ICO" } else { "CUR" }
        )));
    }
    let (mut max_col, mut ico_ofs) = (0u64, 0u64);
    for _ in 0..anzahl {
        let _breite = lies(1);
        let _hoehe = lies(1);
        let mut farben = lies(1).unwrap_or(0);
        lies(1);
        lies(2);
        lies(2);
        lies(4);
        let ofs = lies(4).unwrap_or(0);
        if farben == 0 {
            farben = 256;
        }
        if farben > max_col {
            max_col = farben;
            ico_ofs = ofs;
        }
    }
    pos = ico_ofs as usize;
    let mut lies = |n: usize| -> Option<u64> {
        let b = daten.get(pos..pos + n);
        pos += n;
        b.map(|b| b.iter().rev().fold(0u64, |a, &x| (a << 8) | u64::from(x)))
    };
    let bi_size = lies(4).unwrap_or(0);
    if bi_size != 40 {
        return Err(fehler("Unsupported ICO bitmap format"));
    }
    let breite = lies(4).unwrap_or(0) as u32 as i32;
    let hoehe = lies(4).unwrap_or(0) as u32 as i32;
    lies(2);
    let bits = lies(2).unwrap_or(0);
    let kompression = lies(4).unwrap_or(0);
    lies(4);
    lies(4);
    lies(4);
    let mut clr_used = lies(4).unwrap_or(0) as usize;
    lies(4);
    let pos_daten = pos;
    if kompression != 0 {
        return Err(fehler("Compressed ICO files not supported"));
    }
    let expand = match bits {
        1 | 4 | 8 | 24 => bits as usize,
        32 => 0,
        _ => return Err(fehler("ICO file with unsupported bit count")),
    };
    if !(0..=0xFF_FFFF).contains(&breite) || !(0..=0xFF_FFFF).contains(&hoehe) {
        return Err(fehler("Unsupported or invalid ICO dimensions"));
    }
    let (w, h) = (breite as usize, (hoehe >> 1) as usize);
    let mut p = pos_daten;
    let mut tafel = [0u32; 256];
    if bits <= 8 {
        if clr_used == 0 {
            clr_used = 1 << bits;
        }
        if clr_used > 256 {
            return Err(fehler("Unsupported or incorrect biClrUsed field"));
        }
        for t in tafel.iter_mut().take(clr_used) {
            let mut v = daten
                .get(p..p + 4)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .unwrap_or(0);
            p += 4;
            if v & 0xFF00_0000 == 0 {
                v |= 0xFF00_0000;
            }
            *t = v;
        }
    }
    let byte = |p: &mut usize| -> Option<u8> {
        let b = daten.get(*p).copied();
        *p += 1;
        b
    };
    let lesefehler = || fehler("Error reading from ICO");
    let pad_von = |pitch: usize| if !pitch.is_multiple_of(4) { 4 - pitch % 4 } else { 0 };
    let pad = match expand {
        1 => pad_von(w.div_ceil(8)),
        4 => pad_von(w.div_ceil(2)),
        8 => pad_von(w),
        24 => pad_von(w * 3),
        _ => 0,
    };
    // Unten zuerst - BMP steht auf dem Kopf.
    let mut pixel = vec![0u32; w * h];
    for zeile in (0..h).rev() {
        let ziel = &mut pixel[zeile * w..(zeile + 1) * w];
        match expand {
            1 | 4 | 8 => {
                let mut px = 0u8;
                let shift = 8 - expand;
                for (i, z) in ziel.iter_mut().enumerate() {
                    if i % (8 / expand) == 0 {
                        px = byte(&mut p).ok_or_else(lesefehler)?;
                    }
                    *z = tafel[(px >> shift) as usize];
                    px = ((u32::from(px)) << expand) as u8;
                }
            }
            24 => {
                for z in ziel.iter_mut() {
                    let mut v = 0xFF00_0000u32;
                    for j in 0..3 {
                        v |= u32::from(byte(&mut p).ok_or_else(lesefehler)?) << (j * 8);
                    }
                    *z = v;
                }
            }
            _ => {
                let roh = daten
                    .get(p..p + 4 * w)
                    .ok_or_else(|| fehler("Error reading from datastream"))?;
                for (i, z) in ziel.iter_mut().enumerate() {
                    *z = u32::from_le_bytes([roh[4 * i], roh[4 * i + 1], roh[4 * i + 2], roh[4 * i + 3]]);
                }
                p += 4 * w;
            }
        }
        p += pad;
    }
    // Die UND-Maske: ein gesetztes Bit macht das Pixel zu 0.
    let pad = pad_von(w.div_ceil(8));
    for zeile in (0..h).rev() {
        let ziel = &mut pixel[zeile * w..(zeile + 1) * w];
        let mut px = 0u8;
        for (i, z) in ziel.iter_mut().enumerate() {
            if i % 8 == 0 {
                px = byte(&mut p).ok_or_else(lesefehler)?;
            }
            if px >> 7 != 0 {
                *z = 0;
            }
            px <<= 1;
        }
        p += pad;
    }
    let mut s = Surface::with_flags((w as u32, h as u32), crate::SRCALPHA);
    s.mit_pixeln(|ziel, b, (ox, oy)| {
        for y in 0..h {
            ziel[(oy + y) * b + ox..(oy + y) * b + ox + w].copy_from_slice(&pixel[y * w..(y + 1) * w]);
        }
    });
    Ok(s)
}

// ---------------------------------------------------------------------------
//  SVG
// ---------------------------------------------------------------------------

/// Zeichnet ein SVG in eine Flaeche.
///
/// **Das einzige Format hier, das gezeichnet und nicht gelesen wird.**
/// Alle anderen tragen ihre Pixel mit; ein SVG traegt eine Beschreibung -
/// Pfade, Fuellregeln, Verlaeufe, Text, Transformationen -, und die muss
/// jemand ausfuehren. Das tun `usvg` (liest und rechnet aus) und `resvg`
/// (rastert), beide reines Rust.
///
/// ## Was gemessen ist
///
/// ```text
/// image.load("teal.svg")                  Flaeche, Pixel (0, 128, 128, 255)
/// image.load_sized_svg("teal.svg", (10, 10))   Flaeche 10 x 10
/// ```
///
/// Ohne Groessenangabe gilt die, die im Dokument steht. Mit einer wird
/// **massstabsgetreu** gestreckt: pygame legt die gewuenschte Groesse als
/// Rahmen fest und passt das Bild hinein, statt es zu verzerren.
///
/// Die Flaeche hat immer einen Alphakanal - ein SVG hat fast immer
/// durchsichtige Stellen, und `SRCALPHA` ist auch das, was pygame meldet.
pub fn svg_laden(daten: &[u8], groesse: Option<(u32, u32)>) -> Ergebnis<Surface> {
    let baum = resvg::usvg::Tree::from_data(daten, &resvg::usvg::Options::default())
        .map_err(|e| Fehler::Format(format!("SVG could not be read: {e}")))?;
    let eigen = baum.size();
    let (bb, bh) = (eigen.width(), eigen.height());
    if bb <= 0.0 || bh <= 0.0 {
        return Err(Fehler::Format("SVG without a size".into()));
    }
    let (zb, zh) = match groesse {
        Some((b, h)) if b > 0 && h > 0 => (b, h),
        _ => (bb.ceil() as u32, bh.ceil() as u32),
    };
    // **Die Groesse ist ein Rahmen, kein Auftrag.**
    //
    // Gemessen an `image_test.test_load_sized_svg`: bei `size=(1000, 30)`
    // und einem quadratischen Dokument kommt eine Flaeche **30 x 30**
    // heraus - das Seitenverhaeltnis bleibt, und eine der beiden Seiten
    // trifft den Rahmen genau. Der Test prueft beides ausdruecklich:
    // `ret_size` muss in den Rahmen passen **und** dasselbe Verhaeltnis
    // haben wie das Dokument.
    //
    // Wer den Rahmen fuellt, bekommt bei `(0, 0)` zwar die richtige Farbe
    // und faellt erst beim Verhaeltnis auf.
    let f = (zb as f32 / bb).min(zh as f32 / bh);
    let (zb, zh) = (
        ((bb * f).round() as u32).max(1),
        ((bh * f).round() as u32).max(1),
    );
    let mut bild = resvg::tiny_skia::Pixmap::new(zb, zh)
        .ok_or_else(|| Fehler::Format("SVG size is not representable".into()))?;
    resvg::render(
        &baum,
        resvg::tiny_skia::Transform::from_scale(f, f),
        &mut bild.as_mut(),
    );

    let mut flaeche = Surface::with_flags((zb, zh), crate::SRCALPHA);
    let roh = bild.data();
    flaeche.mit_pixeln(|daten2, breite, (ox, oy)| {
        for y in 0..zh as usize {
            for x in 0..zb as usize {
                let i = (y * zb as usize + x) * 4;
                // `tiny-skia` legt **vormultipliziertes** RGBA ab; pygame
                // gibt gerade Werte heraus. Ohne das Zuruecknehmen ist
                // jede halbdurchsichtige Stelle zu dunkel.
                let a = roh[i + 3] as u32;
                // Ohne Deckung gibt es nichts zurueckzurechnen.
                let zurueck = |c: u8| -> u32 {
                    (c as u32 * 255 + a / 2)
                        .checked_div(a)
                        .map(|v| v.min(255))
                        .unwrap_or(0)
                };
                daten2[(oy + y) * breite + ox + x] = (a << 24)
                    | (zurueck(roh[i]) << 16)
                    | (zurueck(roh[i + 1]) << 8)
                    | zurueck(roh[i + 2]);
            }
        }
    });
    Ok(flaeche)
}

// ---------------------------------------------------------------------------
//  XCF
// ---------------------------------------------------------------------------

/// Liest GIMPs eigenes Format.
///
/// **Ein XCF ist kein Bild, sondern ein Stapel.** Es traegt Ebenen mit
/// Versatz, Deckkraft und Mischart; was pygame (ueber SDL_image) daraus
/// macht, ist das **zusammengerechnete** Bild. `xcf-rs` liest die Ebenen
/// und ihre Pixel; das Uebereinanderlegen steht hier.
///
/// Gemessen an `examples/data/tomato.xcf`: eine Ebene, Tomatenrot
/// `(255, 99, 71)`, und das Bild hat einen Alphakanal.
///
/// Die Ebenen stehen in der Datei **von oben nach unten**; gezeichnet
/// wird also rueckwaerts, damit die oberste zuletzt kommt.
pub fn xcf_laden(daten: &[u8]) -> Ergebnis<Surface> {
    let bild = xcf_rs::data::xcf::Xcf::load(std::io::Cursor::new(daten))
        .map_err(|e| Fehler::Format(format!("XCF could not be read: {e:?}")))?;
    let (b, h) = (bild.width(), bild.height());
    if b == 0 || h == 0 {
        return Err(Fehler::Format("XCF without a size".into()));
    }
    let mut flaeche = Surface::with_flags((b, h), crate::SRCALPHA);
    for ebene in bild.layers.iter().rev() {
        let (eb, eh) = ebene.dimensions();
        for y in 0..eh.min(h) {
            for x in 0..eb.min(b) {
                let Some(px) = ebene.pixel(x, y) else {
                    continue;
                };
                let (r, g, bl, a) = (px.0[0], px.0[1], px.0[2], px.0[3]);
                if a == 0 {
                    continue;
                }
                let alt = flaeche.get_at((x as i32, y as i32));
                // Gerade Alphamischung, wie beim Uebereinanderlegen in
                // einem Bildbearbeiter - **nicht** SDLs Formel: die
                // rechnet fuer das Zeichnen auf dem Schirm und laesst
                // einen durchsichtigen Grund die Farbe verdunkeln.
                let (sa, da) = (a as u32, alt.a as u32);
                let na = sa + da * (255 - sa) / 255;
                let misch = |s: u8, d: u8| -> u32 {
                    (s as u32 * sa + d as u32 * da * (255 - sa) / 255)
                        .checked_div(na)
                        .unwrap_or(0)
                };
                let neu = (na << 24)
                    | (misch(r, alt.r) << 16)
                    | (misch(g, alt.g) << 8)
                    | misch(bl, alt.b);
                flaeche.set_at_ungeprueft((x as i32, y as i32), neu);
            }
        }
    }
    Ok(flaeche)
}

// ---------------------------------------------------------------------------
//  TIFF mit Farbtafel
// ---------------------------------------------------------------------------

/// Liest einen TIFF, dessen Pixel **Tafelnummern** sind.
///
/// `tiff` 0.11 kennt diesen Fall nicht ("Photometric interpretation
/// RGBPalette ... unsupported"), und er ist nicht selten: jedes Bild, das
/// aus einem Malprogramm mit fester Farbliste kommt, sieht so aus.
/// `examples/data/turquoise.tif` aus pygames eigenen Beispielen ist genau
/// das - 32x32, **vier** Bit je Pixel, LZW gepackt, sechzehn Farben.
///
/// Gelesen wird nur, was dieser Fall braucht: das Verzeichnis, die Tafel
/// (Marke 320), die Streifen und drei Packarten. Alles andere geht
/// weiter ueber die Kiste.
///
/// ## Die Tafel steht in **sechzehn** Bit
///
/// Marke 320 traegt `3 * 2^bits` Werte vom Typ `SHORT`: erst alle Rot-,
/// dann alle Gruen-, dann alle Blauanteile. Sie laufen von 0 bis 65535,
/// nicht bis 255 - wer sie ungerechnet nimmt, bekommt aus Tuerkis ein
/// Schwarz mit einem Hauch Farbe.
fn tiff_mit_tafel(daten: &[u8]) -> Option<Surface> {
    let gross = match &daten[..2] {
        b"II" => false,
        b"MM" => true,
        _ => return None,
    };
    let u16_ = |p: usize| -> Option<u32> {
        let b = daten.get(p..p + 2)?;
        Some(if gross {
            u32::from(u16::from_be_bytes([b[0], b[1]]))
        } else {
            u32::from(u16::from_le_bytes([b[0], b[1]]))
        })
    };
    let u32_ = |p: usize| -> Option<u32> {
        let b = daten.get(p..p + 4)?;
        Some(if gross {
            u32::from_be_bytes([b[0], b[1], b[2], b[3]])
        } else {
            u32::from_le_bytes([b[0], b[1], b[2], b[3]])
        })
    };

    let ifd = u32_(4)? as usize;
    let anzahl = u16_(ifd)? as usize;
    let mut marken: std::collections::HashMap<u32, (u32, u32, usize)> =
        std::collections::HashMap::new();
    for i in 0..anzahl {
        let e = ifd + 2 + i * 12;
        let marke = u16_(e)?;
        let typ = u16_(e + 2)?;
        let zahl = u32_(e + 4)?;
        marken.insert(marke, (typ, zahl, e + 8));
    }

    // Ein Wert steht im Eintrag selbst, wenn er in vier Byte passt.
    let werte = |marke: u32| -> Option<Vec<u32>> {
        let &(typ, zahl, stelle) = marken.get(&marke)?;
        let breite = match typ {
            1 | 2 | 6 | 7 => 1,
            3 | 8 => 2,
            4 | 9 => 4,
            _ => return None,
        };
        let anfang = if (zahl as usize) * breite <= 4 {
            stelle
        } else {
            u32_(stelle)? as usize
        };
        let mut aus = Vec::with_capacity(zahl as usize);
        for i in 0..zahl as usize {
            aus.push(match breite {
                1 => u32::from(*daten.get(anfang + i)?),
                2 => u16_(anfang + i * 2)?,
                _ => u32_(anfang + i * 4)?,
            });
        }
        Some(aus)
    };

    if werte(262)?.first().copied() != Some(3) {
        return None;
    }
    let b = *werte(256)?.first()?;
    let h = *werte(257)?.first()?;
    let bits = werte(258).and_then(|v| v.first().copied()).unwrap_or(8);
    let packart = werte(259).and_then(|v| v.first().copied()).unwrap_or(1);
    let tafel = werte(320)?;
    let stellen = werte(273)?;
    let laengen = werte(279)?;
    let _zeilen_je_streifen = werte(278)
        .and_then(|v| v.first().copied())
        .unwrap_or(h)
        .max(1);
    if b == 0 || h == 0 || !(1..=8).contains(&bits) {
        return None;
    }

    // Die Streifen auspacken und aneinanderhaengen.
    let mut roh: Vec<u8> = Vec::new();
    for (i, &stelle) in stellen.iter().enumerate() {
        let laenge = *laengen.get(i)? as usize;
        let stueck = daten.get(stelle as usize..stelle as usize + laenge)?;
        match packart {
            1 => roh.extend_from_slice(stueck),
            5 => {
                let mut d = weezl::decode::Decoder::with_tiff_size_switch(
                    weezl::BitOrder::Msb,
                    8,
                );
                roh.extend_from_slice(&d.decode(stueck).ok()?);
            }
            32773 => roh.extend_from_slice(&packbits(stueck)),
            _ => return None,
        }
    }

    let je_zeile = (b as usize * bits as usize).div_ceil(8);
    let mut flaeche = Surface::with_flags((b, h), 0);
    let farben = tafel.len() / 3;
    for y in 0..h as usize {
        for x in 0..b as usize {
            let bitstelle = x * bits as usize;
            let i = y * je_zeile + bitstelle / 8;
            let Some(&byte) = roh.get(i) else { continue };
            let schieben = 8 - bits as usize - (bitstelle % 8);
            let nummer = ((byte >> schieben) as usize) & ((1 << bits) - 1);
            if nummer >= farben {
                continue;
            }
            // Sechzehn Bit auf acht - siehe den Kopf.
            let acht = |v: u32| v >> 8;
            let argb = 0xFF00_0000
                | (acht(tafel[nummer]) << 16)
                | (acht(tafel[farben + nummer]) << 8)
                | acht(tafel[2 * farben + nummer]);
            flaeche.set_at_ungeprueft((x as i32, y as i32), argb);
        }
    }
    Some(flaeche)
}

/// PackBits auspacken - die einfachste Packart, die TIFF kennt.
fn packbits(ein: &[u8]) -> Vec<u8> {
    let mut aus = Vec::new();
    let mut i = 0;
    while i < ein.len() {
        let n = ein[i] as i8;
        i += 1;
        if n >= 0 {
            let laenge = n as usize + 1;
            if i + laenge > ein.len() {
                break;
            }
            aus.extend_from_slice(&ein[i..i + laenge]);
            i += laenge;
        } else if n != -128 {
            if i >= ein.len() {
                break;
            }
            let laenge = (1 - n as i32) as usize;
            aus.extend(std::iter::repeat_n(ein[i], laenge));
            i += 1;
        }
    }
    aus
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::load_from_bytes;
    use crate::surface::SRCALPHA;

    /// Wo liegen pygames Beispieldaten? `None`, wenn kein pygame da ist -
    /// dann wird der Test still uebersprungen, wie die anderen
    /// Vergleichstests auch.
    fn beispiel(name: &str) -> Option<Vec<u8>> {
        let venv = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()?
            .parent()?
            .parent()?
            .join(".venv/Lib/site-packages/pygame/examples/data")
            .join(name);
        std::fs::read(venv).ok()
    }

    /// Jede Zeile hier ist an **echtem pygame-ce 2.5.7** abgelesen:
    /// Tiefe, Flaggen, Tafellaenge und das Pixel bei `(0, 0)`.
    ///
    /// Die Tabelle ist der eigentliche Test - nicht "laedt ohne Fehler",
    /// sondern "kommt genau so an". Tiefe und Tafellaenge sind dabei die
    /// Angaben, die ein Nachbau am leichtesten falsch rraet: warum ein
    /// GIF auf 256 Eintraege aufgefuellt wird und ein PNG nicht, sieht
    /// man der Datei nicht an.
    #[test]
    fn die_beispieldateien_kommen_an_wie_bei_pygame() {
        /// Datei, Tiefe, SRCALPHA, Tafellaenge, Pixel bei (0, 0).
        type Zeile = (&'static str, u32, bool, Option<usize>, (u8, u8, u8));
        let soll: &[Zeile] = &[
            // Datei, Tiefe, SRCALPHA, Tafellaenge, Pixel (0,0)
            ("red.jpg", 24, false, None, (254, 0, 0)),
            ("blue.gif", 8, false, Some(256), (0, 0, 255)),
            ("green.pcx", 24, false, None, (0, 255, 0)),
            ("yellow.tga", 24, false, None, (255, 255, 0)),
            ("purple.xpm", 8, false, Some(1), (255, 0, 255)),
            ("black.ppm", 24, false, None, (0, 0, 0)),
            ("grey.pgm", 8, false, Some(256), (120, 120, 120)),
            ("white.pbm", 8, false, Some(2), (255, 255, 255)),
            ("crimson.pnm", 24, false, None, (220, 20, 60)),
            ("scarlet.webp", 24, false, None, (252, 14, 53)),
            ("purple.qoi", 32, true, None, (159, 38, 240)),
            ("magenta.lbm", 8, false, Some(2), (255, 0, 255)),
            ("chimp.webp", 24, false, None, (255, 0, 0)),
            ("alien1.jpg", 24, false, None, (255, 251, 240)),
        ];
        let mut geprueft = 0;
        for (name, tiefe, alpha, tafel, farbe) in soll {
            let Some(daten) = beispiel(name) else { continue };
            // Wie pygame mit einem Dateinamen: der Typ kommt aus der Endung -
            // ohne ihn wird ein TGA nicht gefunden.
            let s = crate::image::load_from_bytes_als(&daten, crate::image::typ_aus_namen(name))
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(s.get_bitsize(), *tiefe, "{name}: Tiefe");
            assert_eq!(s.get_flags() & SRCALPHA != 0, *alpha, "{name}: SRCALPHA");
            assert_eq!(
                s.get_palette().map(|p| p.len()),
                *tafel,
                "{name}: Tafellaenge"
            );
            let c = s.get_at((0, 0));
            assert_eq!((c.r, c.g, c.b), *farbe, "{name}: Pixel (0, 0)");
            geprueft += 1;
        }
        // **Ohne pygame daneben liefe der Test leer** - und ein Test, der
        // leer laeuft, meldet Erfolg, ohne etwas gemessen zu haben. Genau
        // die Sorte Fehler, die hier schon zweimal Tests unsichtbar
        // gemacht hat. Deshalb sagt er es laut.
        if geprueft == 0 {
            eprintln!("WARNUNG: keine pygame-Beispieldaten gefunden - nichts geprueft");
        } else {
            assert_eq!(geprueft, soll.len(), "nicht alle Beispieldateien gefunden");
        }
    }

    /// Ein GIF mit durchsichtigem Index gibt eine **Schluesselfarbe**,
    /// keinen Alphakanal - dieselbe Regel wie bei PNG mit `tRNS`.
    #[test]
    fn gif_mit_durchsichtigem_index_gibt_eine_schluesselfarbe() {
        let Some(daten) = beispiel("alien1.gif") else { return };
        let s = load_from_bytes(&daten).unwrap();
        assert_eq!(s.get_bitsize(), 8);
        assert_eq!(s.get_flags() & SRCALPHA, 0, "kein Alphakanal");
        let k = s.get_colorkey().expect("Schluesselfarbe");
        assert_eq!((k.r, k.g, k.b, k.a), (128, 8, 113, 255));
    }

    /// Die Bildfolge eines GIF - Anzahl, Dauer und Groesse.
    ///
    /// **Millisekunden**, nicht Sekunden: gemessen `150.0` bei 15
    /// Hundertstelsekunden in der Datei. Und jedes Bild hat die volle
    /// Groesse, auch wenn der Rahmen nur einen Ausschnitt abdeckt.
    #[test]
    fn gif_bildfolge_wie_pygame() {
        let Some(daten) = beispiel("animated_sample.gif") else { return };
        let folge = gif_bildfolge(&daten).unwrap();
        assert_eq!(folge.len(), 10, "zehn Bilder");
        for (bild, dauer) in &folge {
            assert_eq!(*dauer, 150.0, "Dauer in Millisekunden");
            assert_eq!(bild.get_size(), (312, 312));
        }
    }

    /// PBM: das Bit `0` heisst **weiss**, und die Tafel steht so herum.
    ///
    /// Selbst gebaut, weil die Beispieldatei nur weisse Pixel hat und den
    /// Fall damit nicht unterscheidet.
    #[test]
    fn pbm_null_ist_weiss() {
        // P1, 2x1, erstes Bit 0 (weiss), zweites 1 (schwarz).
        let s = pnm_laden(b"P1\n2 1\n0 1\n").unwrap();
        assert_eq!(s.get_bitsize(), 8);
        assert_eq!(s.get_palette().map(|p| p.len()), Some(2));
        assert_eq!(s.get_at((0, 0)), crate::color::Color::rgb(255, 255, 255));
        assert_eq!(s.get_at((1, 0)), crate::color::Color::rgb(0, 0, 0));
    }

    /// PNM rechnet den Hoechstwert auf 255 hoch.
    #[test]
    fn pnm_rechnet_den_hoechstwert_hoch() {
        // P2 mit Hoechstwert 15: der Wert 15 wird zu 255, 0 bleibt 0.
        let s = pnm_laden(b"P2\n2 1\n15\n0 15\n").unwrap();
        assert_eq!(s.get_at((0, 0)), crate::color::Color::rgb(0, 0, 0));
        assert_eq!(s.get_at((1, 0)), crate::color::Color::rgb(255, 255, 255));
    }

    /// Kommentare duerfen **mitten im Kopf** stehen - alle vier
    /// Beispieldateien haben gleich hinter der Kennung eine GIMP-Zeile.
    #[test]
    fn pnm_uebergeht_kommentare_im_kopf() {
        let s = pnm_laden(b"P3\n# GIMP\n1 1\n# noch einer\n255\n10 20 30\n").unwrap();
        assert_eq!(s.get_bitsize(), 24);
        assert_eq!(s.get_at((0, 0)), crate::color::Color::rgb(10, 20, 30));
    }

    /// Ein ILBM mit einer Ebene, 5 Pixel breit, ByteRun1-gepackt.
    fn ilbm_klein(body: &[u8]) -> Vec<u8> {
        let mut bmhd = Vec::new();
        bmhd.extend_from_slice(&5u16.to_be_bytes());
        bmhd.extend_from_slice(&1u16.to_be_bytes());
        bmhd.extend_from_slice(&[0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 10, 11, 0, 5, 0, 1]);
        let mut d = b"ILBM".to_vec();
        for (n, inhalt) in [(b"BMHD", &bmhd[..]), (b"CMAP", &[0, 0, 0, 255, 0, 255][..]), (b"BODY", body)] {
            d.extend_from_slice(n);
            d.extend_from_slice(&(inhalt.len() as u32).to_be_bytes());
            d.extend_from_slice(inhalt);
            if inhalt.len() % 2 == 1 {
                d.push(0);
            }
        }
        let mut f = b"FORM".to_vec();
        f.extend_from_slice(&(d.len() as u32).to_be_bytes());
        f.extend(d);
        f
    }

    /// `IMG_LoadLBM_RW` legt die Flaeche auf 16 Pixel aufgerundet an, und
    /// ByteRun1 packt aus wie dort: `n >= 0` woertlich, sonst wiederholen.
    #[test]
    fn lbm_breite_und_byterun1_wie_sdl_image() {
        // 1 -> zwei Bytes woertlich: 0xA0 0x01 -> Pixel 0 und 2 gesetzt,
        // und Pixel 15 (das unterste Bit des zweiten Bytes).
        let s = lbm_laden(&ilbm_klein(&[1, 0xA0, 0x01])).unwrap();
        assert_eq!(s.get_size(), (16, 1));
        let magenta = crate::color::Color::rgb(255, 0, 255);
        assert_eq!(s.get_at((0, 0)), magenta);
        assert_eq!(s.get_at((1, 0)), crate::color::Color::rgb(0, 0, 0));
        assert_eq!(s.get_at((2, 0)), magenta);
        assert_eq!(s.get_at((15, 0)), magenta);
        // -1 (0xFF) -> das naechste Byte zweimal.
        let s = lbm_laden(&ilbm_klein(&[0xFF, 0xFF])).unwrap();
        assert_eq!(s.get_at((9, 0)), magenta);
        // Zu kurz: SDL_images Satz.
        assert!(matches!(lbm_laden(&ilbm_klein(&[1, 0xA0])), Err(Fehler::Format(m)) if m == "error reading BODY chunk"));
    }

    /// XPM liest Farben als `#RGB`, `#RRGGBB`, `#RRRRGGGGBBBB` und als Name.
    #[test]
    fn xpm_farbangaben() {
        assert_eq!(xpm_farbe("#f0f"), Some(crate::color::Color::rgb(255, 0, 255)));
        assert_eq!(xpm_farbe("#FF00FF"), Some(crate::color::Color::rgb(255, 0, 255)));
        assert_eq!(
            xpm_farbe("#FFFF0000FFFF"),
            Some(crate::color::Color::rgb(255, 0, 255))
        );
        assert_eq!(xpm_farbe("red"), Some(crate::color::Color::rgb(255, 0, 0)));
        assert_eq!(xpm_farbe("#12"), None);
    }
}
