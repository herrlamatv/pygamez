//! chipstats.rs
//! ============
//! Statistik der Chip-Spiele Lama Mines und Lama Plinko (Fenster **S**) -
//! uebertragen aus `chipstats.py` (Browser: `web/js/games/chipstats.js`).
//!
//! Je Profil im lamabank-Extra des Spiels unter "stats":
//! `{"rounds", "wins", "losses", "wagered", "paid", "net", "best_win",
//! "best_mult", "curve": [Bilanz nach jeder Runde], "x": {...}}`. `x` gehoert
//! dem Spiel (z.B. Plinko-Treffer je Reihenzahl).

use serde_json::{Map, Value};

/// Laenge der Bilanzkurve.
pub const CURVE: usize = 100;
/// Groesste gespeicherte Zahl (bleibt in JS-Zahlen exakt).
pub const MAX_INT: i64 = 9_007_199_254_740_991;

/// Die Statistik eines Spiels.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct ChipStats {
    pub rounds: i64,
    pub wins: i64,
    pub losses: i64,
    pub wagered: i64,
    pub paid: i64,
    pub net: i64,
    pub best_win: i64,
    pub best_mult: i64,
    pub curve: Vec<i64>,
    pub x: Map<String, Value>,
}

fn int_of(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Number(n)) if n.is_i64() || n.is_u64() => n.as_i64().filter(|i| (-MAX_INT..=MAX_INT).contains(i)),
        _ => None,
    }
}

/// `_plain(v)` - nur JSON-Grundtypen, keine Gleitkommazahlen, Tiefe <= 4.
fn plain(v: &Value, depth: u32) -> bool {
    match v {
        Value::Null | Value::Bool(_) | Value::String(_) => true,
        Value::Number(_) => int_of(Some(v)).is_some(),
        Value::Array(a) => depth < 4 && a.iter().all(|x| plain(x, depth + 1)),
        Value::Object(o) => depth < 4 && o.values().all(|x| plain(x, depth + 1)),
    }
}

/// `blank()` - leere Statistik.
pub fn blank() -> ChipStats {
    ChipStats::default()
}

/// `clean(raw)` - Zaehler int >= 0 (sonst 0), `net = paid - wagered`, Kurve =
/// letzte `CURVE` ganze Zahlen, `x` nur mit JSON-Grundtypen.
pub fn clean(raw: Option<&Value>) -> ChipStats {
    let mut st = blank();
    let Some(obj) = raw.and_then(|v| v.as_object()) else {
        return st;
    };
    let get = |k: &str| int_of(obj.get(k)).filter(|v| *v >= 0).unwrap_or(0);
    st.rounds = get("rounds");
    st.wins = get("wins");
    st.losses = get("losses");
    st.wagered = get("wagered");
    st.paid = get("paid");
    st.best_win = get("best_win");
    st.best_mult = get("best_mult");
    st.net = st.paid - st.wagered;
    if let Some(Value::Array(c)) = obj.get("curve") {
        let all: Vec<i64> = c.iter().filter_map(|v| int_of(Some(v))).collect();
        let cut = all.len().saturating_sub(CURVE);
        st.curve = all[cut..].to_vec();
    }
    if let Some(Value::Object(x)) = obj.get("x") {
        for (k, v) in x {
            if plain(v, 1) {
                st.x.insert(k.clone(), v.clone());
            }
        }
    }
    st
}

impl ChipStats {
    /// `record(st, wager, paid, mult100)` - eine Runde bzw. Kugel eintragen.
    pub fn record(&mut self, wager: i64, paid: i64, mult100: i64) {
        self.rounds += 1;
        if paid > wager {
            self.wins += 1;
        } else if paid < wager {
            self.losses += 1;
        }
        self.wagered += wager;
        self.paid += paid;
        self.net += paid - wager;
        self.best_win = self.best_win.max(paid - wager);
        self.best_mult = self.best_mult.max(mult100);
        self.curve.push(self.net);
        if self.curve.len() > CURVE {
            let cut = self.curve.len() - CURVE;
            self.curve.drain(..cut);
        }
    }

    /// `rtp(st)` - paid / wagered oder None.
    pub fn rtp(&self) -> Option<f64> {
        if self.wagered <= 0 {
            None
        } else {
            Some(self.paid as f64 / self.wagered as f64)
        }
    }

    /// `win_rate(st)` - wins / rounds oder None.
    pub fn win_rate(&self) -> Option<f64> {
        if self.rounds <= 0 {
            None
        } else {
            Some(self.wins as f64 / self.rounds as f64)
        }
    }

    /// Als JSON (Reihenfolge wie das Python-dict).
    pub fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("rounds".into(), Value::from(self.rounds));
        m.insert("wins".into(), Value::from(self.wins));
        m.insert("losses".into(), Value::from(self.losses));
        m.insert("wagered".into(), Value::from(self.wagered));
        m.insert("paid".into(), Value::from(self.paid));
        m.insert("net".into(), Value::from(self.net));
        m.insert("best_win".into(), Value::from(self.best_win));
        m.insert("best_mult".into(), Value::from(self.best_mult));
        m.insert("curve".into(), Value::Array(self.curve.iter().map(|v| Value::from(*v)).collect()));
        m.insert("x".into(), Value::Object(self.x.clone()));
        Value::Object(m)
    }
}

/// `record(st, wager, paid, mult100)` als freie Funktion.
pub fn record(st: &mut ChipStats, wager: i64, paid: i64, mult100: i64) {
    st.record(wager, paid, mult100);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chipstats_wie_python() {
        let mut s = blank();
        s.record(100, 250, 250);
        s.record(100, 0, 0);
        s.record(50, 50, 100);
        assert_eq!((s.rounds, s.wins, s.losses, s.wagered, s.paid, s.net), (3, 1, 1, 250, 300, 50));
        assert_eq!((s.best_win, s.best_mult, s.curve.clone()), (150, 250, vec![150, 50, 50]));
        assert_eq!(s.rtp(), Some(1.2));
        for _ in 0..200 {
            s.record(10, 0, 0);
        }
        assert_eq!(s.curve.len(), CURVE);
        assert_eq!(clean(Some(&s.to_value())), s);
        assert_eq!(clean(Some(&Value::from("x"))), blank());
        let dirty = serde_json::json!({"rounds": true, "wins": -3, "paid": 40, "wagered": 100, "net": 9,
                                       "curve": [1, true, 2.5, -4], "x": {"a": [1], "b": 1.5}});
        let c = clean(Some(&dirty));
        assert_eq!((c.rounds, c.wins, c.net, c.curve.clone()), (0, 0, -60, vec![1, -4]));
        assert_eq!(serde_json::to_string(&Value::Object(c.x)).unwrap(), r#"{"a":[1]}"#);
    }
}
