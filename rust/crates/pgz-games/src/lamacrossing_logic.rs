//! lamacrossing_logic.rs
//! =====================
//! Regeln von Lama Crossing - uebertragen aus `games/lamacrossing_logic.py`.
//!
//! Das Lama ueberquert Spur fuer Spur eine Strasse, eine Schlucht, einen Fluss
//! oder einen Eissee. Jede Spur haelt mit Wahrscheinlichkeit `1 - p` und hebt
//! den Multiplikator; ein Fehltritt kostet den Einsatz. Dazu kommen sichtbare
//! Hindernisse (Autos, Felsbrocken, Treibholz, Eisbloecke), denen man per
//! Timing ausweichen muss (`traffic`, `hit_test`). Die Multiplikatoren stehen
//! als Ganzzahlen in Hundertsteln fest in den Tabellen
//! (`floor(0,97 / (1 - p)^k * 100)`), in Python, JS und Rust bitgleich.

use pgz_engine::prelude::*;
use serde_json::Value;

/// Schwierigkeiten.
pub const DIFFS: [&str; 4] = ["easy", "medium", "hard", "extreme"];
/// Welten (Strasse = Standard).
pub const WORLDS: [&str; 4] = ["road", "bridge", "river", "ice"];
/// Einsatzstufen (Mindesteinsatz = erste Stufe).
pub const BETS: [i64; 10] = [10, 20, 50, 100, 200, 500, 1000, 2500, 5000, 10000];
/// Ab diesem Multiplikator (x100) gibt es den Erfolg "Nervenstark".
pub const BRAVE_MULT: i64 = 1000;
/// "Drahtseilakt": auf Extrem bei mindestens x100 ausgezahlt.
pub const EXTREME_MULT: i64 = 10000;
/// Raser bei einem Zufalls-Fehltritt auf der Strasse: `v_hi * RASER_F`.
pub const RASER_F: f64 = 1.6;
/// Laenge des Verlaufs.
pub const HISTORY: usize = 12;

const T_EASY: [i64; 22] = [
    105, 115, 125, 137, 149, 163, 178, 194, 212, 231, 252, 275, 300, 327, 357, 390, 425, 464, 506, 552, 603, 657,
];
const T_MEDIUM: [i64; 20] = [
    116, 139, 167, 201, 241, 289, 347, 417, 500, 600, 720, 864, 1037, 1245, 1494, 1793, 2152, 2582, 3098, 3718,
];
const T_HARD: [i64; 18] = [
    138, 197, 282, 403, 577, 824, 1177, 1682, 2403, 3433, 4905, 7008, 10011, 14302, 20431, 29187, 41697, 59567,
];
const T_EXTREME: [i64; 15] = [
    194, 388, 776, 1552, 3104, 6208, 12416, 24832, 49664, 99328, 198656, 397312, 794624, 1589248, 3178496,
];

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
        "hard" => (3, 10),
        "extreme" => (1, 2),
        _ => (1, 6),
    }
}

/// `TRAFFIC[diff]` - (v_lo, v_hi) Tempo der Hindernisse in px/s bei 800x600
/// (wird mit k skaliert), (gap_lo, gap_hi) Abstand zweier Hindernisse einer
/// Spur in Sekunden. Jede Spur hat ein festes Tempo (kein Ueberholen).
pub fn traffic(diff: &str) -> (f64, f64, f64, f64) {
    match diff {
        "easy" => (240.0, 340.0, 1.8, 3.2),
        "hard" => (360.0, 500.0, 1.1, 2.3),
        "extreme" => (440.0, 620.0, 0.9, 1.9),
        _ => (300.0, 420.0, 1.4, 2.8),
    }
}

/// `SPEED_F[world]` - Tempo-Faktor je Welt: Treibholz und Eisbloecke kommen
/// vom Horizont her (kurzer Weg, daher langsamer), Autos und Felsbrocken von
/// oben.
pub fn speed_f(world: &str) -> f64 {
    match world {
        "river" | "ice" => 0.4,
        _ => 1.0,
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

/// `hit_test(obs_y, obs_len, target_y, reach)` - trifft ein Hindernis (Mitte
/// obs_y, senkrechte Laenge obs_len) das Lama, dessen Mitte auf target_y
/// liegt (reach = halbe Trefferhoehe des Lamas)?
pub fn hit_test(obs_y: f64, obs_len: i32, target_y: f64, reach: f64) -> bool {
    (obs_y - target_y).abs() < obs_len as f64 / 2.0 + reach
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

/// `expected_return(diff, k)` - Erwartungswert (Anteil am Einsatz) der
/// Strategie "bei k auszahlen".
pub fn expected_return(diff: &str, k: i32) -> f64 {
    let (num, den) = risk_frac(diff);
    let mut hold = 1.0;
    for _ in 0..k {
        hold *= (den - num) as f64 / den as f64;
    }
    hold * mult(diff, k) as f64 / 100.0
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
