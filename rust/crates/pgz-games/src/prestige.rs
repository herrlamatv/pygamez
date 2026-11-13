//! prestige.rs
//! ===========
//! Prestige-Stufen fuer Snake - uebertragen aus `prestige.py`.
//!
//! Jeder Eintrag beschreibt, was es kostet, DIESE Stufe zu erreichen:
//! Anzeige-Name, Aepfel (gehen verloren) und Koerper-Segmente (gehen
//! verloren). Nutzen: Wachstum je Apfel = 1 + Level, Punkte x 2^Level.

/// Eine Stufe: (roman, apples, length).
#[derive(Clone, Copy, Debug)]
pub struct Level {
    /// Anzeige-Name.
    pub roman: &'static str,
    /// Benoetigte (und verbrauchte) Aepfel.
    pub apples: i64,
    /// Verlorene Koerper-Segmente.
    pub length: i64,
}

/// Alle Stufen (Prestige I .. X).
pub const PRESTIGE_LEVELS: [Level; 10] = [
    Level { roman: "I", apples: 20, length: 10 },
    Level { roman: "II", apples: 50, length: 25 },
    Level { roman: "III", apples: 200, length: 63 },
    Level { roman: "IV", apples: 500, length: 158 },
    Level { roman: "V", apples: 1200, length: 395 },
    Level { roman: "VI", apples: 3000, length: 988 },
    Level { roman: "VII", apples: 7500, length: 2470 },
    Level { roman: "VIII", apples: 18000, length: 6175 },
    Level { roman: "IX", apples: 45000, length: 15438 },
    Level { roman: "X", apples: 100000, length: 38594 },
];

/// Hoechste Stufe.
pub const MAX_PRESTIGE: i64 = PRESTIGE_LEVELS.len() as i64;

/// `blocks_per_apple(level)`
pub fn blocks_per_apple(level: i64) -> i64 {
    1 + 0.max(level)
}

/// `score_multiplier(level)` - 2 ** level.
pub fn score_multiplier(level: i64) -> i64 {
    2i64.pow(0.max(level) as u32)
}

/// `roman(level)` - Level 0 -> "".
pub fn roman(level: i64) -> &'static str {
    if level <= 0 {
        return "";
    }
    PRESTIGE_LEVELS[(level.min(MAX_PRESTIGE) - 1) as usize].roman
}

/// `next_requirement(level)` - oder `None` an der hoechsten Stufe.
pub fn next_requirement(level: i64) -> Option<Level> {
    if level >= MAX_PRESTIGE {
        return None;
    }
    Some(PRESTIGE_LEVELS[level as usize])
}
