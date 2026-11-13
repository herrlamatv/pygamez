//! levels.rs
//! =========
//! Das Levelsystem der Sammlung: aus **Erfolgspunkten** wird ein Level.
//! Gegenstueck zu `levels.py` - die Kurve ist bitgleich.
//!
//! ```text
//! Kosten fuer den Sprung von Level n auf n+1 :  20 + 8 * (n - 1)
//! Punkte insgesamt bis Level n              :  4 * (n - 1) * (n + 3)
//! ```
//!
//! Level 2 kostet also 20 Punkte, Level 3 weitere 28, Level 10 liegt bei 468
//! Punkten. Wer alle Erfolge holt (~11.000 Punkte), landet bei etwa Level 50.

/// Grundkosten je Level.
const BASE: i64 = 20;
/// Zuwachs der Kosten je Level.
const STEP: i64 = 8;

/// Die 14 Raenge: `(ab Level, i18n-Schluessel, Farbe)`.
pub const RANKS: &[(i64, &str, &str)] = &[
    (1, "level.rank1", "#9aa6bf"),
    (3, "level.rank2", "#7fc1a6"),
    (6, "level.rank3", "#6ecd8c"),
    (10, "level.rank4", "#5ad4d4"),
    (14, "level.rank5", "#589cff"),
    (18, "level.rank6", "#8f7ef2"),
    (22, "level.rank7", "#b07fe8"),
    (27, "level.rank8", "#c86ad8"),
    (32, "level.rank9", "#e05a7d"),
    (37, "level.rank10", "#e2725b"),
    (42, "level.rank11", "#f0a05a"),
    (47, "level.rank12", "#f5cd64"),
    (55, "level.rank13", "#ffd83b"),
    (65, "level.rank14", "#fff1a8"),
];

/// Punkte, die der Sprung von `level` auf `level + 1` kostet.
pub fn cost(level: i64) -> i64 {
    BASE + STEP * (level.max(1) - 1)
}

/// Punkte, die man insgesamt fuer `level` gesammelt haben muss.
pub fn total_for(level: i64) -> i64 {
    let n = level.max(1);
    4 * (n - 1) * (n + 3)
}

/// Level zu einer Punktzahl (Level 1 ab 0 Punkten).
pub fn level_for(points: i64) -> i64 {
    let p = points.max(0);
    // Umkehrung von total_for: n = -1 + sqrt(4 + p/4)
    let mut n = ((-1.0 + (4.0 + p as f64 / 4.0).sqrt()) as i64).max(1);
    // Rundungsfehler abfangen (die Formel ist exakt, f64 ist es nicht).
    while total_for(n + 1) <= p {
        n += 1;
    }
    while n > 1 && total_for(n) > p {
        n -= 1;
    }
    n
}

/// `(Level, Punkte im Level, Punkte bis zum naechsten Level)`.
pub fn progress(points: i64) -> (i64, i64, i64) {
    let p = points.max(0);
    let level = level_for(p);
    (level, p - total_for(level), cost(level))
}

/// `(i18n-Schluessel, Farbe)` des Rangs fuer dieses Level.
pub fn rank(level: i64) -> (&'static str, &'static str) {
    let mut found = (RANKS[0].1, RANKS[0].2);
    for (min_level, key, color) in RANKS {
        if level >= *min_level {
            found = (key, color);
        } else {
            break;
        }
    }
    found
}

/// Lokalisierter Rang-Titel fuer ein Level.
pub fn rank_title(level: i64) -> String {
    crate::i18n::t(rank(level).0)
}

/// Rang-Farbe als `(r, g, b)`.
pub fn rank_color(level: i64) -> (u8, u8, u8) {
    hex_to_rgb(rank(level).1)
}

/// Nummer des Rangs (0-basiert) - fuer Vergleiche und Erfolge.
pub fn rank_index(level: i64) -> usize {
    let mut idx = 0;
    for (i, (min_level, _, _)) in RANKS.iter().enumerate() {
        if level >= *min_level {
            idx = i;
        }
    }
    idx
}

/// Wandelt `"#6ecd8c"` in `(110, 205, 140)`.
pub fn hex_to_rgb(value: &str) -> (u8, u8, u8) {
    let v = value.trim_start_matches('#');
    let byte = |i: usize| u8::from_str_radix(v.get(i..i + 2).unwrap_or("00"), 16).unwrap_or(0);
    (byte(0), byte(2), byte(4))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kurve_stimmt_mit_python_ueberein() {
        assert_eq!(cost(1), 20);
        assert_eq!(cost(2), 28);
        assert_eq!(total_for(1), 0);
        assert_eq!(total_for(2), 20);
        assert_eq!(total_for(3), 48);
        assert_eq!(total_for(10), 468);
    }

    #[test]
    fn level_und_punkte_passen_zusammen() {
        assert_eq!(level_for(0), 1);
        assert_eq!(level_for(19), 1);
        assert_eq!(level_for(20), 2);
        assert_eq!(level_for(467), 9);
        assert_eq!(level_for(468), 10);
        // Jede Punktzahl muss auf ihrem eigenen Level landen.
        for p in 0..5000 {
            let n = level_for(p);
            assert!(total_for(n) <= p && p < total_for(n + 1), "Punkte {p}");
        }
    }

    #[test]
    fn raenge_wachsen_monoton() {
        let mut last = 0;
        for level in 1..80 {
            let idx = rank_index(level);
            assert!(idx >= last);
            last = idx;
        }
        assert_eq!(rank_index(1), 0);
        assert_eq!(rank_index(65), RANKS.len() - 1);
    }

    #[test]
    fn farben_werden_gelesen() {
        assert_eq!(hex_to_rgb("#6ecd8c"), (110, 205, 140));
        assert_eq!(hex_to_rgb("ffffff"), (255, 255, 255));
    }
}
