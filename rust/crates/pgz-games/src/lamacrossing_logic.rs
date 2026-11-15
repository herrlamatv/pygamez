//! lamacrossing_logic.rs
//! =====================
//! Regeln von Lama Crossing - uebertragen aus `games/lamacrossing_logic.py`.
//!
//! Das Lama ueberquert Schritt fuer Schritt eine Schlucht, einen Fluss oder
//! einen Eissee. Jeder Schritt haelt mit Wahrscheinlichkeit `1 - p` und hebt
//! den Multiplikator; ein Fehltritt kostet den Einsatz. Die Multiplikatoren
//! stehen als Ganzzahlen in Hundertsteln fest in den Tabellen
//! (`floor(0,97 / (1 - p)^k * 100)`), in Python, JS und Rust bitgleich.

use pgz_engine::prelude::*;
use serde_json::Value;

/// Schwierigkeiten.
pub const DIFFS: [&str; 4] = ["easy", "medium", "hard", "extreme"];
/// Welten (nur Optik).
pub const WORLDS: [&str; 3] = ["bridge", "river", "ice"];
/// Einsatzstufen (Mindesteinsatz = erste Stufe).
pub const BETS: [i64; 10] = [10, 20, 50, 100, 200, 500, 1000, 2500, 5000, 10000];
/// Ab diesem Multiplikator (x100) gibt es den Erfolg "Nervenstark".
pub const BRAVE_MULT: i64 = 1000;
/// Laenge des Verlaufs.
pub const HISTORY: usize = 12;

const T_EASY: [i64; 20] = [105, 115, 125, 137, 149, 163, 178, 194, 212, 231, 252, 275, 300, 327, 357, 390, 425, 464, 506, 552];
const T_MEDIUM: [i64; 16] = [113, 132, 154, 179, 209, 244, 285, 332, 388, 453, 528, 616, 719, 839, 979, 1142];
const T_HARD: [i64; 13] = [129, 172, 229, 306, 408, 545, 726, 968, 1291, 1722, 2296, 3062, 4082];
const T_EXTREME: [i64; 10] = [161, 269, 449, 748, 1247, 2079, 3465, 5775, 9625, 16042];

/// `TABLES[diff]` - Multiplikator (x100) nach k geschafften Schritten.
pub fn table(diff: &str) -> &'static [i64] {
    match diff {
        "easy" => &T_EASY,
        "hard" => &T_HARD,
        "extreme" => &T_EXTREME,
        _ => &T_MEDIUM,
    }
}

/// `RISK[diff]` - Risiko je Schritt als Bruch (Zaehler, Nenner).
pub fn risk_frac(diff: &str) -> (i64, i64) {
    match diff {
        "easy" => (1, 12),
        "hard" => (1, 4),
        "extreme" => (2, 5),
        _ => (1, 7),
    }
}

/// `steps(diff)` - Anzahl der Schritte bis zum anderen Ufer.
pub fn steps(diff: &str) -> i32 {
    table(diff).len() as i32
}

/// `risk(diff)` - Fehltritt-Wahrscheinlichkeit je Schritt.
pub fn risk(diff: &str) -> f64 {
    let (num, den) = risk_frac(diff);
    num as f64 / den as f64
}

/// `mult(diff, k)` - Multiplikator (x100) nach k Schritten (k = 0 -> 100).
pub fn mult(diff: &str, k: i32) -> i64 {
    if k <= 0 {
        return 100;
    }
    let t = table(diff);
    t[(k as usize).min(t.len()) - 1]
}

/// `payout(bet, diff, k)` - Auszahlung (abgerundet).
pub fn payout(bet: i64, diff: &str, k: i32) -> i64 {
    py::fdiv(bet * mult(diff, k), 100)
}

/// `roll(rng, diff)` - true = haelt (genau ein random()-Aufruf).
pub fn roll(rng: &mut py::Random, diff: &str) -> bool {
    rng.random() >= risk(diff)
}

/// `next_break(rng, diff, k)` - wo waere es nach dem Auszahlen schiefgegangen?
pub fn next_break(rng: &mut py::Random, diff: &str, k: i32) -> Option<i32> {
    ((k + 1)..(steps(diff) + 1)).find(|_| !roll(rng, diff))
}

/// `risk_percent(diff)` - Risiko in ganzen Prozent.
pub fn risk_percent(diff: &str) -> i64 {
    let (num, den) = risk_frac(diff);
    py::fdiv(num * 200 + den, den * 2)
}

/// `fmt_mult(m100, sep)` - 235 -> "2.35".
pub fn fmt_mult(m100: i64, sep: &str) -> String {
    format!("{}{}{:02}", py::fdiv(m100, 100), sep, py::pmod(m100, 100))
}

/// Laufende Runde (`{"bet", "diff", "step"}`).
#[derive(Clone, Debug, PartialEq)]
pub struct Round {
    pub bet: i64,
    pub diff: String,
    pub step: i32,
}

impl Round {
    /// Als JSON (Reihenfolge wie das Python-dict).
    pub fn to_value(&self) -> Value {
        let mut m = serde_json::Map::new();
        m.insert("bet".into(), Value::from(self.bet));
        m.insert("diff".into(), Value::from(self.diff.clone()));
        m.insert("step".into(), Value::from(self.step));
        Value::Object(m)
    }
}

fn py_int(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Number(n)) if n.is_i64() || n.is_u64() => n.as_i64(),
        _ => None,
    }
}

/// `clean_round(raw)` - gespeicherte, noch laufende Runde pruefen.
pub fn clean_round(raw: Option<&Value>) -> Option<Round> {
    let obj = raw?.as_object()?;
    let diff = obj.get("diff").and_then(|v| v.as_str())?;
    if !DIFFS.contains(&diff) {
        return None;
    }
    let bet = py_int(obj.get("bet"))?;
    let step = py_int(obj.get("step"))?;
    if bet <= 0 || !(1 <= step && step < steps(diff) as i64) {
        return None;
    }
    Some(Round { bet, diff: diff.to_string(), step: step as i32 })
}

/// `clean_history(raw)` - Liste von Multiplikatoren x100 (0 = verloren).
pub fn clean_history(raw: Option<&Value>) -> Vec<i64> {
    let Some(Value::Array(list)) = raw else {
        return Vec::new();
    };
    let out: Vec<i64> = list.iter().filter_map(|v| py_int(Some(v))).filter(|v| *v >= 0).collect();
    let cut = out.len().saturating_sub(HISTORY);
    out[cut..].to_vec()
}
