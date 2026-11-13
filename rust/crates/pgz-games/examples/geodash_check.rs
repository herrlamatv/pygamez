//! Physik-Vergleich mit Python: liest JSON-Zeilen
//! `{"level": {...}, "toggles": [...], "every": n, "start": b|null}` von stdin
//! und schreibt je Zeile `hash|laenge|endzustand|k:hash,k:hash,...`.
//!
//!     cargo run -p pgz-games --release --example geodash_check < eingabe.txt
//! Gegenstueck: rust/tools/geodash_check.py

use std::io::BufRead;

use pgz_games::geodash_core as gc;

fn main() {
    for line in std::io::stdin().lock().lines() {
        let line = line.unwrap_or_default();
        if line.trim().is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(&line).expect("json");
        let d = v["level"].as_object().cloned().unwrap_or_default();
        let toggles: Vec<i64> = v["toggles"].as_array().map(|a| a.iter().filter_map(|x| x.as_i64()).collect()).unwrap_or_default();
        let every = v["every"].as_i64().unwrap_or(0);
        let lv = gc::Level::new(&d);
        let start = v["start"].as_i64().map(|b| gc::new_state(&lv, Some(b)));
        let (s, trace) = gc::run_toggles(&lv, &toggles, None, every, start.as_ref());
        let tr: Vec<String> = trace.iter().map(|(k, h)| format!("{k}:{h}")).collect();
        println!("{}|{}|{}|{}", gc::content_hash(&d), gc::level_length(&lv.data), gc::state_str(&s), tr.join(","));
    }
}
