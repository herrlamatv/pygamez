//! Wortfilter-Vergleich: liest Zeilen von stdin, schreibt je Zeile
//! "1" (sauber) oder "0 <code>" (Treffer). Gegenstueck: rust/tools/swear_check.py

use std::io::BufRead;

fn main() {
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap_or_default();
        match pgz_core::swear::check(&line) {
            Ok(()) => println!("1"),
            Err(code) => println!("0 {code}"),
        }
    }
}
