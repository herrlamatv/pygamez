//! lamaplinko_logic.rs
//! ===================
//! Regeln von Lama Plinko - uebertragen aus `games/lamaplinko_logic.py`.
//!
//! Ein Wollknaeuel faellt durch `rows` Reihen von Stiften (je 50 % links /
//! rechts); das Fach ist die Zahl der Rechts-Spruenge, Wahrscheinlichkeit
//! C(rows, i) / 2^rows. Die 27 Tabellen (x100) stehen fest im Code, erzeugt
//! einmal mit `devtools/gen_plinko_tables.py`: symmetrisch, nach aussen nicht
//! fallend, Quote 96,9-97,1 %, Rand bei 16 Reihen x16 / x110 / x1000. Der Pfad
//! kommt aus dem Fairness-Zufall (`fairness::round_rand(KEY, ...)`): `rows`
//! Wuerfe `random() < 0.5` = rechts.

use pgz_core::fairness;
use pgz_core::seedrand::Rand;
use serde_json::{Map, Value};

/// Spiel-Schluessel (Lama-Bank, Fairness-Zufall).
pub const KEY: &str = "lamaplinko";
/// Reihenzahlen 8 .. 16.
pub const ROWS: [i32; 9] = [8, 9, 10, 11, 12, 13, 14, 15, 16];
/// Standard-Reihenzahl.
pub const DEFAULT_ROWS: i32 = 12;
/// Risikostufen.
pub const RISKS: [&str; 3] = ["low", "medium", "high"];
/// Standard-Risiko.
pub const DEFAULT_RISK: &str = "medium";
/// Einsatz je Kugel (Mindesteinsatz = erste Stufe).
pub const BETS: [i64; 10] = [1, 2, 5, 10, 20, 50, 100, 200, 500, 1000];
/// Hoechstens so viele Kugeln gleichzeitig unterwegs.
pub const MAX_BALLS: usize = 50;
/// Laenge des Verlaufs.
pub const HISTORY: usize = 12;
/// Erfolg "Kante": aeusserstes Fach ab so vielen Reihen.
pub const EDGE_ROWS: i32 = 12;
/// Erfolg "Regen": so viele Kugeln gleichzeitig unterwegs.
pub const RAIN_BALLS: usize = 25;

const T_LOW_8: [i64; 9] = [530, 200, 120, 80, 70, 80, 120, 200, 530];
const T_LOW_9: [i64; 10] = [590, 260, 130, 100, 70, 70, 100, 130, 260, 590];
const T_LOW_10: [i64; 11] = [640, 300, 160, 110, 80, 70, 80, 110, 160, 300, 640];
const T_LOW_11: [i64; 12] = [770, 370, 200, 130, 90, 70, 70, 90, 130, 200, 370, 770];
const T_LOW_12: [i64; 13] = [880, 440, 250, 140, 100, 80, 70, 80, 100, 140, 250, 440, 880];
const T_LOW_13: [i64; 14] = [950, 480, 290, 160, 120, 90, 70, 70, 90, 120, 160, 290, 480, 950];
const T_LOW_14: [i64; 15] = [1200, 640, 340, 200, 120, 100, 80, 70, 80, 100, 120, 200, 340, 640, 1200];
const T_LOW_15: [i64; 16] = [1400, 700, 410, 230, 160, 120, 80, 70, 70, 80, 120, 160, 230, 410, 700, 1400];
const T_LOW_16: [i64; 17] = [1600, 860, 490, 280, 180, 130, 100, 70, 70, 70, 100, 130, 180, 280, 490, 860, 1600];
const T_MEDIUM_8: [i64; 9] = [1300, 340, 130, 60, 40, 60, 130, 340, 1300];
const T_MEDIUM_9: [i64; 10] = [1800, 470, 160, 80, 50, 50, 80, 160, 470, 1800];
const T_MEDIUM_10: [i64; 11] = [2100, 670, 250, 100, 60, 40, 60, 100, 250, 670, 2100];
const T_MEDIUM_11: [i64; 12] = [3000, 880, 340, 140, 80, 40, 40, 80, 140, 340, 880, 3000];
const T_MEDIUM_12: [i64; 13] = [3700, 1200, 460, 180, 90, 60, 40, 60, 90, 180, 460, 1200, 3700];
const T_MEDIUM_13: [i64; 14] = [5000, 1600, 620, 250, 130, 70, 40, 40, 70, 130, 250, 620, 1600, 5000];
const T_MEDIUM_14: [i64; 15] = [6300, 2200, 820, 340, 160, 90, 50, 40, 50, 90, 160, 340, 820, 2200, 6300];
const T_MEDIUM_15: [i64; 16] = [8400, 3000, 1100, 460, 200, 110, 70, 40, 40, 70, 110, 200, 460, 1100, 3000, 8400];
const T_MEDIUM_16: [i64; 17] =
    [11000, 3800, 1400, 620, 280, 140, 80, 50, 40, 50, 80, 140, 280, 620, 1400, 3800, 11000];
const T_HIGH_8: [i64; 9] = [3200, 470, 110, 30, 20, 30, 110, 470, 3200];
const T_HIGH_9: [i64; 10] = [4700, 770, 180, 50, 20, 20, 50, 180, 770, 4700];
const T_HIGH_10: [i64; 11] = [7100, 1200, 270, 80, 30, 20, 30, 80, 270, 1200, 7100];
const T_HIGH_11: [i64; 12] = [12000, 2000, 420, 120, 40, 20, 20, 40, 120, 420, 2000, 12000];
const T_HIGH_12: [i64; 13] = [18000, 2900, 660, 180, 60, 30, 20, 30, 60, 180, 660, 2900, 18000];
const T_HIGH_13: [i64; 14] = [27000, 5000, 1000, 270, 90, 40, 20, 20, 40, 90, 270, 1000, 5000, 27000];
const T_HIGH_14: [i64; 15] = [40000, 7500, 1600, 410, 130, 50, 30, 20, 30, 50, 130, 410, 1600, 7500, 40000];
const T_HIGH_15: [i64; 16] = [66000, 12000, 2500, 620, 190, 70, 40, 20, 20, 40, 70, 190, 620, 2500, 12000, 66000];
const T_HIGH_16: [i64; 17] =
    [100000, 19000, 3800, 970, 280, 90, 50, 30, 20, 30, 50, 90, 280, 970, 3800, 19000, 100000];

/// `TABLES[risk][rows]` - Multiplikatoren x100 aller Faecher 0 .. rows.
pub fn table(rows: i32, risk: &str) -> &'static [i64] {
    match (risk, rows) {
        ("low", 8) => &T_LOW_8,
        ("low", 9) => &T_LOW_9,
        ("low", 10) => &T_LOW_10,
        ("low", 11) => &T_LOW_11,
        ("low", 12) => &T_LOW_12,
        ("low", 13) => &T_LOW_13,
        ("low", 14) => &T_LOW_14,
        ("low", 15) => &T_LOW_15,
        ("low", 16) => &T_LOW_16,
        ("high", 8) => &T_HIGH_8,
        ("high", 9) => &T_HIGH_9,
        ("high", 10) => &T_HIGH_10,
        ("high", 11) => &T_HIGH_11,
        ("high", 12) => &T_HIGH_12,
        ("high", 13) => &T_HIGH_13,
        ("high", 14) => &T_HIGH_14,
        ("high", 15) => &T_HIGH_15,
        ("high", 16) => &T_HIGH_16,
        (_, 8) => &T_MEDIUM_8,
        (_, 9) => &T_MEDIUM_9,
        (_, 10) => &T_MEDIUM_10,
        (_, 11) => &T_MEDIUM_11,
        (_, 13) => &T_MEDIUM_13,
        (_, 14) => &T_MEDIUM_14,
        (_, 15) => &T_MEDIUM_15,
        (_, 16) => &T_MEDIUM_16,
        _ => &T_MEDIUM_12,
    }
}

/// `mult100(rows, risk, slot)`.
pub fn mult100(rows: i32, risk: &str, slot: i32) -> i64 {
    let t = table(rows, risk);
    t[(slot.max(0) as usize).min(t.len() - 1)]
}

/// `payout(bet, rows, risk, slot)` - abgerundet (ohne Uebertrag).
pub fn payout(bet: i64, rows: i32, risk: &str, slot: i32) -> i64 {
    bet * mult100(rows, risk, slot) / 100
}

/// `payout_carry(bet, rows, risk, slot, carry)` - (Auszahlung, neuer Uebertrag):
/// `bet * m100 + carry` in Hundertsteln, ganze Chips werden ausgezahlt, der
/// Rest (0..99) bleibt fuer die naechste Kugel.
pub fn payout_carry(bet: i64, rows: i32, risk: &str, slot: i32, carry: i64) -> (i64, i64) {
    let total = bet * mult100(rows, risk, slot) + carry;
    (total.div_euclid(100), total.rem_euclid(100))
}

/// `clean_carry(raw)` - ganze Zahl 0..99, sonst 0.
pub fn clean_carry(raw: Option<&Value>) -> i64 {
    fairness::py_int(raw).filter(|v| (0..=99).contains(v)).unwrap_or(0)
}

/// `is_edge(rows, slot)` - aeusserstes Fach?
pub fn is_edge(rows: i32, slot: i32) -> bool {
    slot == 0 || slot == rows
}

/// `drop_path(rand, rows)` - je Reihe 1 = rechts (`random() < 0.5`), sonst 0.
pub fn drop_path(rand: &mut Rand, rows: i32) -> Vec<i32> {
    (0..rows).map(|_| if rand.random() < 0.5 { 1 } else { 0 }).collect()
}

/// `slot_of(path)` - Zahl der Rechts-Spruenge.
pub fn slot_of(path: &[i32]) -> i32 {
    path.iter().sum()
}

/// `path_from_seed(rows, server, client, nonce)`.
pub fn path_from_seed(rows: i32, server: &str, client: &str, nonce: i64) -> Vec<i32> {
    drop_path(&mut fairness::round_rand(KEY, server, client, nonce), rows)
}

/// `comb(n, k)` - exakt.
pub fn comb(n: i64, k: i64) -> i64 {
    if k < 0 || k > n {
        return 0;
    }
    let k = k.min(n - k);
    let mut r: i64 = 1;
    for i in 1..=k {
        r = r * (n - k + i) / i;
    }
    r
}

/// `prob(rows, slot)` - (Zaehler, Nenner) = (C(rows, slot), 2^rows).
pub fn prob(rows: i32, slot: i32) -> (i64, i64) {
    (comb(rows as i64, slot as i64), 1i64 << rows)
}

/// `expected_return(rows, risk)` - Quote; exakter Zaehler, eine Division
/// (= `float(Fraction)` in Python).
pub fn expected_return(rows: i32, risk: &str) -> f64 {
    let num: i64 = table(rows, risk).iter().enumerate().map(|(i, m)| comb(rows as i64, i as i64) * m).sum();
    num as f64 / ((1i64 << rows) * 100) as f64
}

/// `fmt_prob(rows, slot, sep)` - Prozent: ab 1 % zwei, darunter vier Stellen.
pub fn fmt_prob(rows: i32, slot: i32, sep: &str) -> String {
    let (num, den) = prob(rows, slot);
    if num * 100 >= den {
        let v = (num * 20000 + den) / (den * 2);
        format!("{}{}{:02}", v / 100, sep, v % 100)
    } else {
        let v = (num * 2_000_000 + den) / (den * 2);
        format!("{}{}{:04}", v / 10000, sep, v % 10000)
    }
}

/// `fmt_mult(m100, sep)` - 20 -> "0.20".
pub fn fmt_mult(m100: i64, sep: &str) -> String {
    format!("{}{}{:02}", m100 / 100, sep, m100 % 100)
}

/// `fmt_short(m100, sep)` - 20 -> "0.2", 1100 -> "11", 125 -> "1.25".
pub fn fmt_short(m100: i64, sep: &str) -> String {
    if m100 % 100 == 0 {
        (m100 / 100).to_string()
    } else if m100 % 10 == 0 {
        format!("{}{}{}", m100 / 100, sep, m100 % 100 / 10)
    } else {
        fmt_mult(m100, sep)
    }
}

/// Trefferzaehler je Reihenzahl (Index rows - 8, je rows + 1 Faecher).
pub type Hits = Vec<Vec<i64>>;

/// `blank_hits()`.
pub fn blank_hits() -> Hits {
    ROWS.iter().map(|r| vec![0; *r as usize + 1]).collect()
}

/// `clean_hits(raw)` - immer alle neun Reihenzahlen; falsche Laenge -> Nullen.
pub fn clean_hits(raw: Option<&Value>) -> Hits {
    let mut out = blank_hits();
    let Some(obj) = raw.and_then(|v| v.as_object()) else {
        return out;
    };
    for (i, r) in ROWS.iter().enumerate() {
        if let Some(Value::Array(list)) = obj.get(&r.to_string()) {
            if list.len() == *r as usize + 1 {
                out[i] = list
                    .iter()
                    .map(|x| fairness::py_int(Some(x)).filter(|v| (0..=fairness::MAX_NONCE).contains(v)).unwrap_or(0))
                    .collect();
            }
        }
    }
    out
}

/// `add_hit(hits, rows, slot)`.
pub fn add_hit(hits: &mut Hits, rows: i32, slot: i32) {
    if let Some(row) = hits.get_mut((rows - 8).max(0) as usize) {
        if let Some(v) = row.get_mut(slot.max(0) as usize) {
            *v += 1;
        }
    }
}

/// Trefferzaehler als JSON `{"8": [...], ..., "16": [...]}`.
pub fn hits_to_value(hits: &Hits) -> Value {
    let mut m = Map::new();
    for (r, row) in ROWS.iter().zip(hits.iter()) {
        m.insert(r.to_string(), Value::Array(row.iter().map(|v| Value::from(*v)).collect()));
    }
    Value::Object(m)
}

/// `clean_history(raw)` - Multiplikatoren x100, letzte 12.
pub fn clean_history(raw: Option<&Value>) -> Vec<i64> {
    let Some(Value::Array(list)) = raw else {
        return Vec::new();
    };
    let out: Vec<i64> = list
        .iter()
        .filter_map(|v| fairness::py_int(Some(v)))
        .filter(|v| (0..=1_000_000).contains(v))
        .collect();
    let cut = out.len().saturating_sub(HISTORY);
    out[cut..].to_vec()
}

#[cfg(test)]
mod tests {
    //! Gleichstand mit Python: Pruefwerte aus `devtools/export_chiplogic_ref.py`
    //! (Datei `data/chiplogic_ref.json`). Aufruf:
    //! `cargo test -p pgz-games --offline --lib -- chiplogic`
    use super::*;
    use pgz_core::fairness as f;

    fn refdata() -> Value {
        serde_json::from_str(include_str!("data/chiplogic_ref.json")).unwrap()
    }

    fn js(v: &Value) -> String {
        serde_json::to_string(v).unwrap()
    }

    #[test]
    fn chiplogic_plinko_wie_python() {
        let r = refdata();
        let p = &r["plinko"];
        let mut tabs = Map::new();
        let mut shorts = Map::new();
        for risk in RISKS {
            let mut t = Map::new();
            let mut s = Map::new();
            for n in ROWS {
                t.insert(n.to_string(), serde_json::json!(table(n, risk)));
                s.insert(n.to_string(), serde_json::json!((0..=n).map(|i| fmt_short(mult100(n, risk, i), ",")).collect::<Vec<_>>()));
                assert_eq!(table(n, risk).len(), n as usize + 1);
            }
            tabs.insert(risk.to_string(), Value::Object(t));
            shorts.insert(risk.to_string(), Value::Object(s));
            let exp: Vec<f64> = ROWS.iter().map(|n| expected_return(*n, risk)).collect();
            let want: Vec<f64> =
                p["expected"][risk].as_array().unwrap().iter().map(|v| v.as_str().unwrap().parse().unwrap()).collect();
            assert_eq!(exp, want, "{risk}");
        }
        assert_eq!(f::sha256_hex(&js(&Value::Object(tabs))), p["digest_tables"].as_str().unwrap());
        assert_eq!(f::sha256_hex(&js(&Value::Object(shorts))), p["digest_short"].as_str().unwrap());
        let mut probs = Map::new();
        for n in ROWS {
            probs.insert(n.to_string(), serde_json::json!((0..=n).map(|i| vec![fmt_prob(n, i, "."), fmt_prob(n, i, ",")]).collect::<Vec<_>>()));
        }
        assert_eq!(f::sha256_hex(&js(&Value::Object(probs))), p["digest_prob"].as_str().unwrap());
        for c in p["paths"].as_array().unwrap() {
            let a = c.as_array().unwrap();
            let got = path_from_seed(a[0].as_i64().unwrap() as i32, a[1].as_str().unwrap(), a[2].as_str().unwrap(), a[3].as_i64().unwrap());
            assert_eq!(js(&serde_json::json!(got)), js(&a[4]));
        }
        for (raw, want) in p["hits_in"].as_array().unwrap().iter().zip(p["hits_out"].as_array().unwrap()) {
            assert_eq!(js(&hits_to_value(&clean_hits(Some(raw)))), js(want));
        }
        for (raw, want) in p["history_in"].as_array().unwrap().iter().zip(p["history_out"].as_array().unwrap()) {
            assert_eq!(js(&serde_json::json!(clean_history(Some(raw)))), js(want));
        }
        let pay = vec![payout(7, 16, "high", 0), payout(5, 8, "high", 4), payout(1, 16, "low", 8)];
        assert_eq!(js(&serde_json::json!(pay)), js(&p["payout"]));
        let mut carry = 0;
        for c in p["carry"].as_array().unwrap() {
            let a = c.as_array().unwrap();
            let (win, nc) = payout_carry(
                a[0].as_i64().unwrap(),
                a[1].as_i64().unwrap() as i32,
                a[2].as_str().unwrap(),
                a[3].as_i64().unwrap() as i32,
                carry,
            );
            carry = nc;
            assert_eq!((win, carry), (a[4].as_i64().unwrap(), a[5].as_i64().unwrap()));
        }
        for (raw, want) in p["carry_clean_in"].as_array().unwrap().iter().zip(p["carry_clean_out"].as_array().unwrap()) {
            assert_eq!(clean_carry(Some(raw)), want.as_i64().unwrap());
        }
        let mut h = blank_hits();
        add_hit(&mut h, 16, 0);
        assert_eq!(h[8][0], 1);
        assert!(is_edge(16, 16) && !is_edge(16, 15) && slot_of(&[1, 0, 1]) == 2);
    }
}
