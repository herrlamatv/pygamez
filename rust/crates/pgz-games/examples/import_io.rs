//! Rust-Seite von `tools/import_check.py`: eine geteilte Datei importieren
//! (Minigolf-Bahn, Geometry-Dash-Level, Replay) - in der App nur ueber den
//! Datei-Dialog erreichbar. Datenordner aus `PYGAMEZ_DATA_DIR`.
//!
//!     import_io ugc <datei> <spiel|->
//!     import_io replay <datei> <spiel|->
//!
//! Ausgabe je Aufruf eine Zeile: `<ok> <grund> <id>`.

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    if a.len() < 3 {
        eprintln!("Aufruf: import_io ugc|replay <datei> <spiel|->");
        std::process::exit(2);
    }
    pgz_core::init();
    let game = Some(a[2].as_str()).filter(|g| *g != "-");
    let (ok, why, id) = match a[0].as_str() {
        "ugc" => {
            let (ok, why, m) = pgz_games::ugc::import_from(&a[1], game);
            (ok, why, m.and_then(|m| m.get("id").and_then(|v| v.as_str()).map(String::from)))
        }
        _ => {
            let (ok, why, r) = pgz_core::replay::import_from(std::path::Path::new(&a[1]), game);
            (ok, why, r.and_then(|r| r.get("id").and_then(|v| v.as_str()).map(String::from)))
        }
    };
    println!("{ok} {why} {}", id.unwrap_or_default());
    pgz_core::stats::flush();
}
