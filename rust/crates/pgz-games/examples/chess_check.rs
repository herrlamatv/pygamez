//! Engine-Vergleich mit Python: liest Zeilen "stufe|fen" von stdin und
//! schreibt "beste|wert|tiefe|gute" je Zeile. Stufe "h" = Hinweis-Stufe.
//! Eine gemeinsame Transpositionstabelle ueber alle Zeilen (wie im Spiel).
//!
//!     cargo run -p pgz-games --release --example chess_check < eingabe.txt
//! Gegenstueck: rust/tools/chess_check.py

use std::io::BufRead;

use pgz_games::chess_engine as ce;

fn main() {
    let mut tt = ce::Tt::new();
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap_or_default();
        let Some((lv, fen)) = line.split_once('|') else {
            continue;
        };
        let level = if lv == "h" { ce::HINT_LEVEL } else { ce::LEVELS[lv.parse::<usize>().unwrap_or(0)] };
        let pos = ce::from_fen(fen.trim(), None).expect("fen");
        let mut s = ce::Search::new(&pos, &level, std::mem::take(&mut tt));
        s.run();
        let good: Vec<String> = s.good.iter().map(|m| m.to_string()).collect();
        println!("{}|{}|{}|{}", s.best_move, s.best_score, s.depth_done, good.join(","));
        tt = std::mem::take(&mut s.tt);
    }
}
