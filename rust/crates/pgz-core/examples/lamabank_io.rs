//! Rust-Seite von `tools/lamabank_check.py`: fuehrt eine Folge von
//! Lama-Bank-Buchungen aus einer Datei aus (eine je Zeile) und schreibt je
//! Zeile das Ergebnis - im Datenordner aus `PYGAMEZ_DATA_DIR`. Die
//! Python-Seite tut dasselbe mit `lamabank.py`; verglichen werden Ausgabe und
//! die geschriebene `mem.json`.
//!
//!     lamabank_io <befehle.txt>
//!
//! Befehle (Leerzeichen-getrennt, JSON-Werte am Zeilenende ohne Leerzeichen):
//! `load`, `reset_cache`, `balance`, `escrow`, `refills`, `ledger G`,
//! `value_for G`, `score_for G`, `min_bet G [M]`, `is_broke G [M]`,
//! `debit N G`, `credit N G 0|1`, `bet_and_pay B W G [KEY JSON]`,
//! `refill G [M]`, `set_escrow N`, `pay_from_escrow N G`, `clear_escrow 0|1`,
//! `settle_escrow W G`, `set_extra KEY JSON`, `get_extra KEY`.

use pgz_core::lamabank as lb;
use serde_json::Value;

fn zahl(s: Option<&&str>) -> i64 {
    s.and_then(|v| v.parse().ok()).unwrap_or(0)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(datei) = args.first() else {
        eprintln!("Aufruf: lamabank_io <befehle.txt>");
        std::process::exit(2);
    };
    let text = std::fs::read_to_string(datei).unwrap_or_default();
    pgz_core::init();
    for zeile in text.lines() {
        let w: Vec<&str> = zeile.split_whitespace().collect();
        let Some(cmd) = w.first() else { continue };
        let g = w.get(1).copied().unwrap_or("");
        let out = match *cmd {
            "load" => {
                lb::load();
                "ok".to_string()
            }
            "reset_cache" => {
                lb::reset_cache();
                "ok".to_string()
            }
            "balance" => lb::balance().to_string(),
            "escrow" => lb::escrow().to_string(),
            "refills" => lb::refills().to_string(),
            "ledger" => lb::ledger(g).to_string(),
            "value_for" => lb::value_for(g).to_string(),
            "score_for" => lb::score_for(g).to_string(),
            "min_bet" => lb::min_bet(g, w.get(2).copied()).to_string(),
            "is_broke" => lb::is_broke(g, w.get(2).copied()).to_string(),
            "debit" => lb::debit(zahl(w.get(1)), w.get(2).copied().unwrap_or("")).to_string(),
            "credit" => {
                lb::credit(zahl(w.get(1)), w.get(2).copied().unwrap_or(""), w.get(3) == Some(&"1"));
                "ok".to_string()
            }
            "bet_and_pay" => {
                let extra = match (w.get(4), w.get(5)) {
                    (Some(k), Some(j)) => Some((*k, serde_json::from_str::<Value>(j).unwrap_or(Value::Null))),
                    _ => None,
                };
                lb::bet_and_pay(zahl(w.get(1)), zahl(w.get(2)), w.get(3).copied().unwrap_or(""), extra).to_string()
            }
            "refill" => lb::refill_if_broke(g, w.get(2).copied()).to_string(),
            "set_escrow" => lb::set_escrow(zahl(w.get(1))).to_string(),
            "pay_from_escrow" => lb::pay_from_escrow(zahl(w.get(1)), w.get(2).copied().unwrap_or("")).to_string(),
            "clear_escrow" => lb::clear_escrow(w.get(1) == Some(&"1")).to_string(),
            "settle_escrow" => {
                lb::settle_escrow(zahl(w.get(1)), w.get(2).copied().unwrap_or(""));
                "ok".to_string()
            }
            "set_extra" => {
                let v = w.get(2).and_then(|j| serde_json::from_str::<Value>(j).ok()).unwrap_or(Value::Null);
                lb::set_extra(g, v);
                "ok".to_string()
            }
            "get_extra" => serde_json::to_string(&lb::get_extra(g).unwrap_or(Value::Null)).unwrap_or_default(),
            other => format!("? {other}"),
        };
        println!("{zeile} -> {out}");
    }
}
