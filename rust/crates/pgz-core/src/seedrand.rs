//! seedrand.rs
//! ===========
//! Ein kleiner Zufallsgenerator, der in Rust, Python UND im Browser exakt
//! dieselben Zahlen liefert (Gegenstuecke: `seedrand.py`,
//! `web/js/games/seedrand.js`).
//!
//! Jede Sprache wuerfelt aus demselben Seed anders - ein "Level 12" waere am
//! PC ein anderes Raetsel als im Browser. Fuer alles, was ueberall gleich
//! sein muss (Tagesstrecke, Tageswort, Tages-Sudoku, Crossy-Road-Strecken),
//! benutzen deshalb alle drei Fassungen **mulberry32**: reine
//! 32-Bit-Ganzzahl-Arithmetik, also bitgenau identisch.
//!
//! ```
//! use pgz_core::seedrand::{Rand, seed_from};
//! let mut rng = Rand::new(seed_from(&["crossy", "42"]));
//! let wurf = rng.randint(1, 6);
//! ```
//!
//! Auch `randint`/`shuffle`/`choice` sind selbst geschrieben, damit sie die
//! Zufallszahlen in derselben Reihenfolge verbrauchen wie die anderen
//! Fassungen.

/// mulberry32 - deterministisch, schnell, identisch zu `seedrand.py`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rand {
    state: u32,
}

impl Rand {
    /// Neuer Generator mit gegebenem Seed.
    pub fn new(seed: u32) -> Self {
        Self { state: seed }
    }

    /// Neuer Generator, der von der Uhr angestossen wird (fuer alles, was
    /// NICHT ueberall gleich sein muss).
    pub fn from_clock() -> Self {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() ^ d.as_secs() as u32)
            .unwrap_or(0x6D2B_79F5);
        Self::new(nanos)
    }

    /// Naechste Zufallszahl als vorzeichenlose 32-Bit-Ganzzahl.
    pub fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_add(0x6D2B_79F5);
        let mut t = self.state;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        t ^ (t >> 14)
    }

    /// Gleitkommazahl in `[0, 1)`.
    pub fn random(&mut self) -> f64 {
        self.next_u32() as f64 / 4_294_967_296.0
    }

    /// Ganzzahl in `[a, b]` (beide Grenzen eingeschlossen).
    pub fn randint(&mut self, a: i64, b: i64) -> i64 {
        if b < a {
            return a;
        }
        a + (self.random() * (b - a + 1) as f64) as i64
    }

    /// Index in `[0, n)` - der haeufigste Fall, ohne Umweg ueber `randint`.
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.random() * n as f64) as usize
    }

    /// Gleitkommazahl in `[a, b)`.
    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.random()
    }

    /// True mit Wahrscheinlichkeit `p`.
    pub fn chance(&mut self, p: f64) -> bool {
        self.random() < p
    }

    /// Zufaelliges Element einer nicht leeren Liste.
    pub fn choice<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }
        items.get(self.below(items.len()))
    }

    /// Mischt die Liste an Ort und Stelle (Fisher-Yates von hinten).
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        if items.len() < 2 {
            return;
        }
        for i in (1..items.len()).rev() {
            let j = (self.random() * (i + 1) as f64) as usize;
            items.swap(i, j.min(i));
        }
    }

    /// Index gemaess Gewichten (Liste nicht-negativer Zahlen).
    pub fn weighted(&mut self, weights: &[f64]) -> usize {
        // Bewusst eine schlichte Schleife statt einer Summenfunktion: Python
        // rechnet sum() seit 3.12 mit Fehlerkompensation, JavaScript nicht -
        // bei krummen Gewichten zoegen die Fassungen sonst minimal anders.
        let mut total = 0.0;
        for w in weights {
            total += *w;
        }
        let r = self.random() * total;
        let mut acc = 0.0;
        for (i, w) in weights.iter().enumerate() {
            acc += *w;
            if r < acc {
                return i;
            }
        }
        weights.len().saturating_sub(1)
    }
}

/// 32-Bit-Seed aus beliebigen Teilen (FNV-1a ueber den Text).
///
/// Die Teile werden mit `|` verbunden; nur ASCII verwenden (die JS-Fassung
/// rechnet mit Zeichencodes, bei ASCII identisch zu UTF-8-Bytes).
pub fn seed_from(parts: &[&str]) -> u32 {
    let mut h: u32 = 0x811C_9DC5;
    let joined = parts.join("|");
    for b in joined.bytes() {
        h ^= b as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h
}

/// Heutiges lokales Datum als `YYYY-MM-DD`.
pub fn today_str() -> String {
    crate::store::today()
}

/// Seed des Tages fuer ein Spiel - an einem Datum fuer alle gleich.
pub fn daily_seed(game: &str, date: Option<&str>) -> u32 {
    let today = today_str();
    seed_from(&["daily", game, date.unwrap_or(&today)])
}

/// Fortlaufende Tagesnummer (Tage seit 2026-01-01) fuer ein Datum.
pub fn day_index(date: Option<&str>) -> i64 {
    let today = today_str();
    let text = date.unwrap_or(&today);
    let mut parts = text.split('-');
    let y: i64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(2026);
    let m: i64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    let d: i64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(1);
    days_from_civil(y, m, d) - days_from_civil(2026, 1, 1)
}

/// `seconds_to_midnight(now)` aus wordle.py: Sekunden bis zum naechsten
/// lokalen Tageswechsel.
///
/// Wie Python: `time.localtime(now)` (abgerundet auf ganze Sekunden), dann
/// `time.mktime` fuer 0:00 des Folgetags, Differenz mit `round()`
/// (Unentschieden zur geraden Zahl), nie negativ.
pub fn seconds_to_midnight(now: f64) -> i64 {
    use chrono::{Duration, LocalResult, TimeZone};
    let Some(lt) = chrono::Local.timestamp_opt(now.floor() as i64, 0).earliest() else {
        return 0;
    };
    let Some(ndt) = (lt.date_naive() + Duration::days(1)).and_hms_opt(0, 0, 0) else {
        return 0;
    };
    let midnight = match chrono::Local.from_local_datetime(&ndt) {
        LocalResult::Single(t) => t.timestamp(),
        LocalResult::Ambiguous(a, _) => a.timestamp(),
        // 0:00 faellt in eine Zeitumstellung: mktime schiebt nach vorn.
        LocalResult::None => match chrono::Local.from_local_datetime(&(ndt + Duration::hours(1))).earliest() {
            Some(t) => t.timestamp() - 3600,
            None => return 0,
        },
    };
    0.max((midnight as f64 - now).round_ties_even() as i64)
}

/// Tage seit 1970-01-01 fuer ein Datum (Gegenstueck zu `civil_from_unix`).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die ersten Zahlen muessen exakt denen von `seedrand.py` entsprechen.
    /// (Referenzwerte mit der Python-Fassung erzeugt.)
    #[test]
    fn mulberry32_ist_bitgleich() {
        let mut rng = Rand::new(0);
        let got: Vec<u32> = (0..5).map(|_| rng.next_u32()).collect();
        assert_eq!(
            got,
            vec![1_144_304_738, 1_416_247, 958_946_056, 627_933_444, 2_007_157_716]
        );
    }

    #[test]
    fn seed_from_ist_bitgleich() {
        assert_eq!(seed_from(&["daily", "wordle", "2026-09-23"]), 1_650_640_481);
        assert_eq!(seed_from(&["crossy", "42"]), 3_302_698_996);
    }

    #[test]
    fn grenzen_werden_eingehalten() {
        let mut rng = Rand::new(12345);
        for _ in 0..2000 {
            let v = rng.randint(1, 6);
            assert!((1..=6).contains(&v));
            let f = rng.random();
            assert!((0.0..1.0).contains(&f));
        }
    }

    #[test]
    fn shuffle_behaelt_alle_elemente() {
        let mut rng = Rand::new(7);
        let mut items: Vec<i32> = (0..50).collect();
        rng.shuffle(&mut items);
        items.sort_unstable();
        assert_eq!(items, (0..50).collect::<Vec<i32>>());
    }

    #[test]
    fn tagesnummer_zaehlt_richtig() {
        assert_eq!(day_index(Some("2026-01-01")), 0);
        assert_eq!(day_index(Some("2026-01-31")), 30);
        assert_eq!(day_index(Some("2026-12-31")), 364);
        assert_eq!(day_index(Some("2027-01-01")), 365);
    }
}
