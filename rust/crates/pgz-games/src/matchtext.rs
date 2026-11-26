//! matchtext.rs
//! ============
//! Die `match_text(key, value)`-Methoden der Spielklassen (statische
//! Methoden in Python, `getattr(game_cls, "match_text", None)` in
//! `matchview.info_parts`): Anzeige eines Info-Werts im Matches-Screen,
//! `None` = Standard-Formatierung.

use pgz_core::i18n::{t, tf};
use serde_json::Value;

use crate::{crossyroad_world as cw, lamacrossing_logic as lcl, sudoku_gen};

/// `game_cls.match_text(key, value)` des Spiels mit Highscore-Schluessel `game`.
pub fn match_text(game: &str, key: &str, value: &Value) -> Option<String> {
    let v = value.as_str()?;
    match game {
        // crossyroad.py: Figur
        "crossy" if key == "ch" && cw::char_ids().contains(&v) => {
            Some(tf("cr.char_line", &[("name", &t(&format!("cr.char.{v}")))]))
        }
        // lamacrossing.py: Schwierigkeit statt interner Kennung
        "lamacrossing" if key == "lv" && lcl::DIFFS.contains(&v) => Some(t(&format!("lcr.diff.{v}"))),
        // lamaplinko.py: Risiko-Stufe
        "lamaplinko" if key == "lv" && ["low", "medium", "high"].contains(&v) => Some(t(&format!("lpl.risk.{v}"))),
        // sudoku.py: Variante
        "sudoku" if key == "vr" && sudoku_gen::VARIANTS.contains(&v) => Some(t(&format!("sud.variant.{v}"))),
        _ => None,
    }
}
