//! competitive.rs
//! ==============
//! Feineinstellungen fuer den Competitive-Modus von Snake - uebertragen aus
//! `competitive.py`.
//!
//! Level-Aufstieg (Stufen aus `games/levels/snake-comp.json`, eingebunden
//! beim Bauen; fehlerhafte Datei -> eingebaute Standardwerte), blauer Apfel
//! (Slot-Machine) und lila Apfel (Laengen-Wette).

use std::sync::OnceLock;

use pgz_engine::py;

const LEVELS_JSON: &str = include_str!("../../../../games/levels/snake-comp.json");

const DEFAULT_ROWS: [(i64, i64, i64); 15] = [
    (4, 2, 2),
    (10, 3, 3),
    (18, 4, 4),
    (30, 5, 5),
    (46, 6, 6),
    (68, 7, 7),
    (96, 8, 8),
    (132, 9, 9),
    (176, 10, 10),
    (230, 11, 11),
    (296, 12, 12),
    (376, 13, 13),
    (472, 14, 14),
    (586, 15, 15),
    (720, 16, 16),
];

struct Tab {
    steps: Vec<i64>,
    apples: Vec<i64>,
    mult: Vec<i64>,
}

fn tab() -> &'static Tab {
    static T: OnceLock<Tab> = OnceLock::new();
    T.get_or_init(|| {
        let (mut base_apples, mut base_mult, mut rows) = (1i64, 1i64, DEFAULT_ROWS.to_vec());
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(LEVELS_JSON) {
            let num = |v: Option<&serde_json::Value>| v.and_then(|x| x.as_f64()).map(|f| f.trunc() as i64);
            let parsed: Option<Vec<(i64, i64, i64)>> = data
                .get("levels")
                .and_then(|l| l.as_array())
                .map(|a| a.iter().map(|d| Some((num(d.get("threshold"))?, num(d.get("apples"))?, num(d.get("multiplier"))?))).collect::<Option<Vec<_>>>())
                .unwrap_or(Some(Vec::new()));
            if let Some(mut parsed) = parsed {
                if !parsed.is_empty() {
                    base_apples = num(data.get("base_apples")).unwrap_or(base_apples);
                    base_mult = num(data.get("base_multiplier")).unwrap_or(base_mult);
                    parsed.sort_by_key(|r| r.0);
                    rows = parsed;
                }
            }
        }
        Tab {
            steps: rows.iter().map(|r| r.0).collect(),
            apples: std::iter::once(base_apples).chain(rows.iter().map(|r| r.1)).collect(),
            mult: std::iter::once(base_mult).chain(rows.iter().map(|r| r.2)).collect(),
        }
    })
}

/// `MAX_LEVEL`
pub fn max_level() -> i64 {
    tab().steps.len() as i64
}

/// `MAX_APPLES`
pub fn max_apples() -> i64 {
    *tab().apples.iter().max().unwrap_or(&1)
}

/// `level_for_apples(total)`
pub fn level_for_apples(total: i64) -> i64 {
    let mut lvl = 0;
    for need in &tab().steps {
        if total >= *need {
            lvl += 1;
        } else {
            break;
        }
    }
    lvl
}

/// `apples_on_field(level)`
pub fn apples_on_field(level: i64) -> i64 {
    tab().apples[0.max(level.min(max_level())) as usize]
}

/// `score_multiplier(level)`
pub fn score_multiplier(level: i64) -> i64 {
    tab().mult[0.max(level.min(max_level())) as usize]
}

/// `next_step(total)` - (erreicht, benoetigt) oder `None` an der Maximalstufe.
pub fn next_step(total: i64) -> Option<(i64, i64)> {
    let lvl = level_for_apples(total);
    if lvl >= max_level() {
        return None;
    }
    let steps = &tab().steps;
    let prev = if lvl > 0 { steps[(lvl - 1) as usize] } else { 0 };
    Some((total - prev, steps[lvl as usize] - prev))
}

// ===================================================== Slot-Machine (blau)

/// `SLOT_SYMBOLS[key]` - Farbe eines Symbols.
pub fn symbol_color(key: &str) -> (i32, i32, i32) {
    match key {
        "seven" => (255, 210, 90),
        "gem" => (110, 220, 235),
        "bell" => (255, 190, 70),
        "apple" => (240, 90, 90),
        _ => (235, 110, 160),
    }
}

/// `REEL` - Walzenband (seltener = wertvoller).
pub fn reel() -> Vec<&'static str> {
    let mut r = Vec::new();
    for (k, n) in [("cherry", 5), ("apple", 4), ("bell", 3), ("gem", 2), ("seven", 1)] {
        for _ in 0..n {
            r.push(k);
        }
    }
    r
}

fn triple(key: &str) -> f64 {
    match key {
        "seven" => 6.0,
        "gem" => 4.0,
        "bell" => 3.0,
        "apple" => 2.5,
        _ => 2.0,
    }
}

/// `spin_reels()` - drei Symbole (Pythons globales `random`).
pub fn spin_reels() -> [&'static str; 3] {
    let r = reel();
    let a = py::choice(&r);
    let b = py::choice(&r);
    let c = py::choice(&r);
    [a, b, c]
}

/// `slot_outcome(reels)` - (Multiplikator, Ergebnis).
pub fn slot_outcome(reels: &[&str; 3]) -> (f64, &'static str) {
    let (a, b, c) = (reels[0], reels[1], reels[2]);
    if a == b && b == c {
        return (triple(a), "jackpot");
    }
    if a == b || b == c || a == c {
        return (1.5, "pair");
    }
    (0.5, "miss")
}

// ===================================================== Lila Apfel

/// `purple_factor(hardcore)`
pub fn purple_factor(hardcore: bool) -> f64 {
    if hardcore {
        return py::round_to(py::uniform(0.25, 2.25), 2);
    }
    py::round_to(py::uniform(0.5, 1.5), 2)
}

/// `purple_stake(hardcore)`
pub fn purple_stake(hardcore: bool) -> f64 {
    if hardcore {
        return py::round_to(py::uniform(0.75, 0.90), 2);
    }
    0.5
}
