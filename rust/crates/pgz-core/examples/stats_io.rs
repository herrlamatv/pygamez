//! Rust-Seite von `tools/stats_check.py`: Statistik exportieren bzw.
//! importieren (Fortschritt -> "Exportieren"/"Importieren", dort nur ueber
//! Datei-Dialoge erreichbar) - im Datenordner aus `PYGAMEZ_DATA_DIR`.
//!
//!     stats_io export <datei>
//!     stats_io import <datei> <ziel-profil|-> <merge 0|1>

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    pgz_core::init();
    match args.first().map(String::as_str) {
        Some("export") => {
            let ok = pgz_core::stats::export_to(std::path::Path::new(&args[1]));
            println!("export {ok}");
        }
        Some("import") => {
            let target = args.get(2).filter(|t| t.as_str() != "-").map(String::as_str);
            let merge = args.get(3).map(|m| m == "1").unwrap_or(true);
            let (ok, why) = pgz_core::stats::import_from(std::path::Path::new(&args[1]), target, merge);
            println!("import {ok} {why:?}");
        }
        _ => {
            eprintln!("Aufruf: stats_io export <datei> | import <datei> <ziel|-> <0|1>");
            std::process::exit(2);
        }
    }
    pgz_core::stats::flush();
}
