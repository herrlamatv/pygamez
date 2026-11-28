//! pgz-core
//! ========
//! Die **Datenschicht** von PyGameZ - alles, was ohne Grafik auskommt.
//!
//! Das Gegenstueck zu den Python-Modulen `store.py`, `settings.py`,
//! `i18n.py`, `profiles.py`, `highscore.py`, `levels.py`, `stats.py`,
//! `achievements.py` und `seedrand.py`. Die Dateiformate sind bewusst
//! identisch geblieben: eine bestehende `mem.json` bzw. `settings.json` der
//! Python-Fassung wird von der Rust-Fassung gelesen und umgekehrt.
//!
//! Warum ein eigenes Crate? Genau wie in Python (`main.py` prueft die
//! Abhaengigkeiten, bevor pygame zum ersten Mal importiert wird) soll die
//! Datenschicht ohne Fenster, ohne OpenGL und ohne Audio benutzbar sein -
//! das macht sie in Tests pruefbar.

pub mod achievements;
pub mod chipstats;
pub mod defs;
pub mod fairness;
pub mod favorites;
pub mod highscore;
pub mod i18n;
pub mod lamabank;
pub mod levels;
pub mod matches;
pub mod profiles;
pub mod replay;
pub mod seedrand;
pub mod settings;
pub mod stats;
pub mod store;
pub mod swear;

/// Versionsnummer der Sammlung (erscheint im Menue und im Fenstertitel).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Startet die Datenschicht: Speicher pruefen, Sprache laden.
///
/// Einmalig beim Programmstart aufrufen - vor dem ersten Fenster.
pub fn init() {
    store::init();
    i18n::init();
    // Module mit Zwischenspeicher muessen beim Profilwechsel geleert werden -
    // sonst zeigen sie die Zahlen des vorigen Profils.
    stats::register_switch_hook();
    achievements::register_switch_hook();
    lamabank::register_switch_hook();
    // matches.py: store.on_switch(reset_cache) - Offenes wegschreiben.
    matches::register_switch_hook();
}

/// Schreibt alles Offene weg (Statistik-Puffer). Vor dem Beenden aufrufen.
pub fn shutdown() {
    stats::flush();
}
