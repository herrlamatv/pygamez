//! build.rs - legt unter Windows (MSVC) das PyGameZ-Logo als Programm-Symbol
//! in die .exe: aus `logo/pygamez3-256.png` und `-128.png` (dieselben Dateien,
//! die `main.py` ueber `logo.icon_photos` fuer das Fenster nimmt) wird eine
//! Ressource (.res) mit der Icon-Gruppe 1, die link.exe direkt annimmt.
//! Explorer und Verknuepfungen zeigen so das Logo; zur Laufzeit holt
//! `fenster::symbol_setzen` es fuer Titel- und Taskleiste.
//!
//! Ohne Zusatz-Kisten und ohne rc.exe: das .res-Format ist einfach (Kopf je
//! Eintrag, Daten auf 4 Byte aufgefuellt), PNG-Bilder darf eine Icon-
//! Ressource seit Windows Vista direkt enthalten.

use std::path::PathBuf;

const BILDER: [&str; 2] = ["pygamez3-256.png", "pygamez3-128.png"];

fn main() {
    let logo = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR")).join("../../../logo");
    println!("cargo:rerun-if-changed=build.rs");
    for b in BILDER {
        println!("cargo:rerun-if-changed={}", logo.join(b).display());
    }
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if os != "windows" || env != "msvc" {
        return;
    }
    let mut pngs = Vec::new();
    for b in BILDER {
        match std::fs::read(logo.join(b)) {
            Ok(d) if d.len() > 24 && &d[1..4] == b"PNG" => pngs.push(d),
            _ => {
                println!("cargo:warning=Logo {b} fehlt - das Programm bekommt kein Symbol");
                return;
            }
        }
    }
    let ziel = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("pygamez.res");
    std::fs::write(&ziel, ressource(&pngs)).expect("pygamez.res schreiben");
    println!("cargo:rustc-link-arg-bins={}", ziel.display());
}

/// Kopf eines Eintrags (RESOURCEHEADER mit Zahl-Typ und Zahl-Namen: 32 Byte).
fn kopf(out: &mut Vec<u8>, groesse: usize, typ: u16, name: u16, flags: u16) {
    out.extend((groesse as u32).to_le_bytes());
    out.extend(32u32.to_le_bytes());
    out.extend(0xFFFFu16.to_le_bytes());
    out.extend(typ.to_le_bytes());
    out.extend(0xFFFFu16.to_le_bytes());
    out.extend(name.to_le_bytes());
    out.extend(0u32.to_le_bytes()); // DataVersion
    out.extend(flags.to_le_bytes()); // MemoryFlags
    out.extend(0u16.to_le_bytes()); // LanguageId: neutral
    out.extend(0u32.to_le_bytes()); // Version
    out.extend(0u32.to_le_bytes()); // Characteristics
}

fn auffuellen(out: &mut Vec<u8>) {
    while !out.len().is_multiple_of(4) {
        out.push(0);
    }
}

/// .res-Datei: leerer Kennungs-Eintrag, je Bild ein RT_ICON (3), dann die
/// Gruppe RT_GROUP_ICON (14) mit dem Namen 1.
fn ressource(pngs: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    kopf(&mut out, 0, 0, 0, 0);
    let mut gruppe = Vec::new();
    gruppe.extend(0u16.to_le_bytes());
    gruppe.extend(1u16.to_le_bytes()); // 1 = Icon
    gruppe.extend((pngs.len() as u16).to_le_bytes());
    for (i, png) in pngs.iter().enumerate() {
        let id = i as u16 + 1;
        kopf(&mut out, png.len(), 3, id, 0x1010);
        out.extend(png);
        auffuellen(&mut out);
        // Breite/Hoehe aus dem IHDR-Block (256 wird als 0 eingetragen).
        let w = u32::from_be_bytes([png[16], png[17], png[18], png[19]]);
        let h = u32::from_be_bytes([png[20], png[21], png[22], png[23]]);
        gruppe.push(if w >= 256 { 0 } else { w as u8 });
        gruppe.push(if h >= 256 { 0 } else { h as u8 });
        gruppe.push(0); // Farben
        gruppe.push(0); // reserviert
        gruppe.extend(1u16.to_le_bytes()); // Ebenen
        gruppe.extend(32u16.to_le_bytes()); // Bit je Pixel
        gruppe.extend((png.len() as u32).to_le_bytes());
        gruppe.extend(id.to_le_bytes());
    }
    kopf(&mut out, gruppe.len(), 14, 1, 0x1030);
    out.extend(&gruppe);
    auffuellen(&mut out);
    out
}
