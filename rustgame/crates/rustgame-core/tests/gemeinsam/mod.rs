//! gemeinsam
//! =========
//! Was sich `golden.rs` und `fuzz.rs` teilen: PNG lesen und schreiben, das
//! Testmuster, der Hash.
//!
//! Warum ein Unterordner? Cargo uebersetzt **jede** `.rs` direkt in `tests/`
//! als eigenes Testprogramm. Ein Unterordner bleibt davon verschont und
//! laesst sich mit `mod gemeinsam;` einbinden - das ist der uebliche Weg
//! fuer geteilten Testcode.

#![allow(dead_code)] // Nicht jeder Testlauf braucht jede Hilfe.

use std::path::{Path, PathBuf};

use rustgame_core::rect::Rect;
use rustgame_core::surface::{Surface, SRCALPHA};

/// Wo die Referenzbilder liegen.
pub fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("golden")
}

/// Wo Bilder zur Fehlersuche abgelegt werden.
pub fn ausgabe_dir(unterordner: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
        .join("rustgame.output")
        .join(unterordner)
}

/// Ein geladenes Referenzbild.
pub struct Bild {
    pub w: u32,
    pub h: u32,
    /// RGBA, zeilenweise.
    pub daten: Vec<u8>,
}

pub fn lade_png(pfad: &Path) -> Option<Bild> {
    let datei = std::fs::File::open(pfad).ok()?;
    let dekoder = png::Decoder::new(datei);
    let mut leser = dekoder.read_info().ok()?;
    let mut puffer = vec![0; leser.output_buffer_size()];
    let info = leser.next_frame(&mut puffer).ok()?;
    puffer.truncate(info.buffer_size());

    // Auf RGBA vereinheitlichen - pygame speichert je nach Inhalt anders:
    // eine Flaeche ohne SRCALPHA wird als RGB gesichert, mit als RGBA.
    let daten = match info.color_type {
        png::ColorType::Rgba => puffer,
        png::ColorType::Rgb => puffer
            .as_chunks::<3>().0.iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::Grayscale => puffer.iter().flat_map(|v| [*v, *v, *v, 255]).collect(),
        png::ColorType::GrayscaleAlpha => puffer
            .as_chunks::<2>().0.iter()
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Indexed => return None,
    };
    Some(Bild {
        w: info.width,
        h: info.height,
        daten,
    })
}

pub fn schreibe_png(pfad: &Path, w: u32, h: u32, rgba: &[u8]) {
    if let Some(d) = pfad.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    let Ok(datei) = std::fs::File::create(pfad) else {
        return;
    };
    let mut kodierer = png::Encoder::new(std::io::BufWriter::new(datei), w, h);
    kodierer.set_color(png::ColorType::Rgba);
    kodierer.set_depth(png::BitDepth::Eight);
    if let Ok(mut schreiber) = kodierer.write_header() {
        let _ = schreiber.write_image_data(rgba);
    }
}

/// Das Testmuster aus `gen_golden.py`, Pixel fuer Pixel gleich.
pub fn muster(w: u32, h: u32) -> Surface {
    let mut s = Surface::new((w, h));
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let r = if (x as u32) < w / 2 { 255 } else { 40 };
            let g = if (y as u32) < h / 2 { 255 } else { 40 };
            let b = if (x + y) % 3 == 0 { 200 } else { 60 };
            s.set_at((x, y), (r, g, b));
        }
    }
    s
}

/// Legt das Muster in `w x h` an und **skaliert es auf die Flaeche**.
///
/// Genau wie `muster()` in `gen_golden.py`, inklusive des Skalierens, das
/// dort leicht zu uebersehen ist. Fehlt dieser Schritt, hat die Flaeche
/// hinterher die falsche Groesse - und die Transform-Faelle vergleichen
/// Aepfel mit Birnen.
pub fn muster_auf(mut s: Surface, w: u32, h: u32) -> Surface {
    let hilf = muster(w, h);
    let gross = rustgame_core::transform::scale(&hilf, s.get_size());
    s.blit(&gross, (0, 0));
    s
}

/// Eine einfarbige Flaeche mit Alphakanal - die Quelle der Blit-Faelle.
pub fn gefuellt(size: (u32, u32), c: (u8, u8, u8, u8)) -> Surface {
    let mut s = Surface::with_flags(size, SRCALPHA);
    s.fill(c);
    s
}

/// FNV-1a, 64 Bit - muss Zahl fuer Zahl dasselbe liefern wie `fnv1a` in
/// `tools/fuzz_ops.py`. Bewusst kein SHA-256: fuer "sind diese zwei Bilder
/// gleich?" reicht das, und es spart eine Fremdkiste auf beiden Seiten.
pub fn fnv1a(daten: &[u8]) -> u64 {
    let mut h: u64 = 0xCBF2_9CE4_8422_2325;
    for b in daten {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
    }
    h
}

/// Legt erwartet | bekommen | Unterschied (achtfach verstaerkt) nebeneinander.
///
/// Ohne das steht im Fehlerfall nur "3 Pixel weichen ab" - mit sieht man
/// auf einen Blick, *welche Form* danebenliegt.
pub fn schreibe_dreitafel(pfad: &Path, erwartet: &[u8], ist: &[u8], w: u32, h: u32) {
    let breite = w * 3 + 4;
    let mut out = vec![0u8; (breite * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let q = ((y * w + x) * 4) as usize;
            let a = [erwartet[q], erwartet[q + 1], erwartet[q + 2], 255];
            let b = [ist[q], ist[q + 1], ist[q + 2], 255];
            let d = [
                a[0].abs_diff(b[0]).saturating_mul(8),
                a[1].abs_diff(b[1]).saturating_mul(8),
                a[2].abs_diff(b[2]).saturating_mul(8),
                255,
            ];
            let mut setze = |sp: u32, p: [u8; 4]| {
                let i = ((y * breite + sp) * 4) as usize;
                out[i..i + 4].copy_from_slice(&p);
            };
            setze(x, a);
            setze(w + 2 + x, b);
            setze(w * 2 + 4 + x, d);
        }
    }
    schreibe_png(pfad, breite, h, &out);
}

/// Die ersten `hoechstens` Stellen, an denen zwei Bilder auseinandergehen.
pub fn abweichungen(erwartet: &[u8], ist: &[u8], w: u32, tol: u8, hoechstens: usize) -> Vec<String> {
    let mut v = Vec::new();
    for i in 0..(ist.len() / 4) {
        let a = &erwartet[i * 4..i * 4 + 4];
        let b = &ist[i * 4..i * 4 + 4];
        if (0..4).any(|k| a[k].abs_diff(b[k]) > tol) {
            if v.len() >= hoechstens {
                break;
            }
            let (x, y) = (i as u32 % w, i as u32 / w);
            v.push(format!(
                "({x},{y}) erwartet {:?} bekommen {:?}",
                (a[0], a[1], a[2], a[3]),
                (b[0], b[1], b[2], b[3])
            ));
        }
    }
    v
}

/// Wie viele Pixel liegen weiter als `tol` auseinander?
pub fn anzahl_abweichungen(erwartet: &[u8], ist: &[u8], tol: u8) -> usize {
    (0..(ist.len() / 4))
        .filter(|i| (0..4).any(|k| erwartet[i * 4 + k].abs_diff(ist[i * 4 + k]) > tol))
        .count()
}

/// Ein Rechteck aus vier Zahlen - spart in den erzeugten Dateien Text.
pub fn r4(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect::new(x, y, w, h)
}
