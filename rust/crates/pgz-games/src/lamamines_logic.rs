//! lamamines_logic.rs
//! ==================
//! Regeln von Lama Mines - uebertragen aus `games/lamamines_logic.py`.
//!
//! N = grid x grid Plaetze, M Minen, der Rest Edelsteine. Multiplikator
//! (x100) nach k Edelsteinen - ganzzahlig, schrittweise, in Python, JS und
//! Rust bitgleich:
//!
//! ```text
//! raw(j)  = HOUSE * C(N, M) // C(N - j, M)
//! m_0     = 100
//! m_j     = max(raw(j), m_(j-1) + 1)      (j = 1 .. k)
//! m100(k) = min(MAX_MULT100, m_k)
//! ```
//!
//! Das Minenfeld kommt aus dem Fairness-Zufall
//! (`fairness::round_rand(KEY, server, client, nonce)`): Liste 0 .. N-1
//! mischen, die ersten M Plaetze sind Minen. Platz = Zeile * grid + Spalte.

use pgz_core::fairness;
use pgz_core::seedrand::Rand;
use serde_json::{Map, Value};

/// Spiel-Schluessel (Lama-Bank, Fairness-Zufall).
pub const KEY: &str = "lamamines";
/// Feldgroessen (Tasten 1-4).
pub const GRIDS: [i32; 4] = [4, 5, 6, 7];
/// Standard-Feldgroesse.
pub const DEFAULT_GRID: i32 = 5;
/// Standard-Minenzahl.
pub const DEFAULT_MINES: i32 = 3;
/// Einsatzstufen (Mindesteinsatz = erste Stufe).
pub const BETS: [i64; 10] = [10, 20, 50, 100, 200, 500, 1000, 2500, 5000, 10000];
/// Auszahlungsquote in Prozent.
pub const HOUSE: i64 = 97;
/// Hoechster Multiplikator x100 (x100.000).
pub const MAX_MULT100: i64 = 10_000_000;
/// Laenge des Verlaufs.
pub const HISTORY: usize = 12;
/// Erfolg "Mutig": ab x10 ausgezahlt.
pub const BRAVE_MULT: i64 = 1000;
/// Erfolg "Legende": ab x1000 ausgezahlt.
pub const LEGEND_MULT: i64 = 100000;

/// `MINE_PRESETS[grid]` - Schnellwahl der Minenzahl.
pub fn mine_presets(grid: i32) -> &'static [i32] {
    match grid {
        4 => &[1, 2, 3, 5, 8, 12],
        6 => &[1, 3, 6, 12, 20, 35],
        7 => &[1, 3, 7, 14, 24, 48],
        _ => &[1, 3, 5, 10, 15, 24],
    }
}

/// `cells(grid)` - Anzahl der Plaetze.
pub fn cells(grid: i32) -> i32 {
    grid * grid
}

/// `max_mines(grid)` - N-1.
pub fn max_mines(grid: i32) -> i32 {
    cells(grid) - 1
}

/// `gems(grid, mines)` - N-M.
pub fn gems(grid: i32, mines: i32) -> i32 {
    cells(grid) - mines
}

/// `clamp_mines(grid, mines)` - auf 1 .. N-1.
pub fn clamp_mines(grid: i32, mines: i32) -> i32 {
    mines.min(max_mines(grid)).max(1)
}

/// `comb(n, k)` - Binomialkoeffizient, schrittweise wie in Python.
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

/// `raw_mult100(grid, mines, j)` - reine Formel ohne Untergrenze und Deckel.
pub fn raw_mult100(grid: i32, mines: i32, j: i32) -> i64 {
    let n = cells(grid) as i64;
    HOUSE * comb(n, mines as i64) / comb(n - j as i64, mines as i64)
}

/// `mult100(grid, mines, k)` - Multiplikator x100 nach k Edelsteinen.
pub fn mult100(grid: i32, mines: i32, k: i32) -> i64 {
    if k <= 0 {
        return 100;
    }
    let n = cells(grid) as i64;
    let k = k.min(cells(grid) - mines) as i64;
    let top = HOUSE * comb(n, mines as i64);
    let mut m: i64 = 100;
    for j in 1..=k {
        m = (top / comb(n - j, mines as i64)).max(m + 1);
        if m >= MAX_MULT100 {
            return MAX_MULT100;
        }
    }
    m
}

/// `payout(bet, grid, mines, k)` - abgerundet auf ganze Chips.
pub fn payout(bet: i64, grid: i32, mines: i32, k: i32) -> i64 {
    bet * mult100(grid, mines, k) / 100
}

/// `payout_carry(bet, grid, mines, k, carry)` - (Auszahlung, neuer Uebertrag)
/// mit Rest-Hundertstel-Uebertrag wie bei Plinko.
pub fn payout_carry(bet: i64, grid: i32, mines: i32, k: i32, carry: i64) -> (i64, i64) {
    let total = bet * mult100(grid, mines, k) + carry;
    (total.div_euclid(100), total.rem_euclid(100))
}

/// `clean_carry(raw)` - ganze Zahl 0..99, sonst 0.
pub fn clean_carry(raw: Option<&Value>) -> i64 {
    py_int(raw).filter(|v| (0..=99).contains(v)).unwrap_or(0)
}

/// `next_mult100(grid, mines, k)` - None, wenn alle Edelsteine gefunden.
pub fn next_mult100(grid: i32, mines: i32, k: i32) -> Option<i64> {
    if k >= gems(grid, mines) {
        None
    } else {
        Some(mult100(grid, mines, k + 1))
    }
}

/// `safe_chance(grid, mines, k)` - (N-M-k, N-k).
pub fn safe_chance(grid: i32, mines: i32, k: i32) -> (i64, i64) {
    let n = cells(grid) as i64;
    ((n - mines as i64 - k as i64).max(0), (n - k as i64).max(1))
}

/// `safe_percent(grid, mines, k)` - Zehntel-Prozent, kaufmaennisch gerundet.
pub fn safe_percent(grid: i32, mines: i32, k: i32) -> i64 {
    let (num, den) = safe_chance(grid, mines, k);
    (num * 2000 + den) / (den * 2)
}

/// `all_safe_percent(grid, mines, k)` - Chance, dass k bestimmte Felder ALLE
/// sicher sind, in Zehntel-Prozent (C(N-k, M) / C(N, M), gerundet; i128).
pub fn all_safe_percent(grid: i32, mines: i32, k: i32) -> i64 {
    let n = cells(grid) as i64;
    if k <= 0 {
        return 1000;
    }
    if k > gems(grid, mines) {
        return 0;
    }
    let num = comb(n - k as i64, mines as i64) as i128;
    let den = comb(n, mines as i64) as i128;
    ((num * 2000 + den) / (den * 2)) as i64
}

/// `fmt_percent(p10, sep)` - 845 -> "84.5".
pub fn fmt_percent(p10: i64, sep: &str) -> String {
    format!("{}{}{}", p10 / 10, sep, p10 % 10)
}

/// `is_max(grid, mines, k)` - Hoechstgewinn erreicht.
pub fn is_max(grid: i32, mines: i32, k: i32) -> bool {
    mult100(grid, mines, k) >= MAX_MULT100
}

/// `is_clear(grid, mines, k)` - alle Edelsteine gefunden.
pub fn is_clear(grid: i32, mines: i32, k: i32) -> bool {
    k >= gems(grid, mines)
}

/// `ladder(grid, mines, k, count)` - die naechsten Stufen (Schritt, x100).
pub fn ladder(grid: i32, mines: i32, k: i32, count: i32) -> Vec<(i32, i64)> {
    (k + 1..=gems(grid, mines).min(k + count)).map(|j| (j, mult100(grid, mines, j))).collect()
}

/// `make_field(rand, grid, mines)` - sortierte Minenplaetze.
pub fn make_field(rand: &mut Rand, grid: i32, mines: i32) -> Vec<i32> {
    let mut order: Vec<i32> = (0..cells(grid)).collect();
    rand.shuffle(&mut order);
    let mut f: Vec<i32> = order[..mines.clamp(0, cells(grid)) as usize].to_vec();
    f.sort_unstable();
    f
}

/// `field_from_seed(grid, mines, server, client, nonce)`.
pub fn field_from_seed(grid: i32, mines: i32, server: &str, client: &str, nonce: i64) -> Vec<i32> {
    make_field(&mut fairness::round_rand(KEY, server, client, nonce), grid, mines)
}

/// `fmt_mult(m100, sep)` - 235 -> "2.35".
pub fn fmt_mult(m100: i64, sep: &str) -> String {
    format!("{}{}{:02}", m100 / 100, sep, m100 % 100)
}

/// `expected_return(grid, mines, k)` - Gleitkomma wie die JS-Fassung
/// (Python rechnet exakt mit Fraction).
pub fn expected_return(grid: i32, mines: i32, k: i32) -> f64 {
    let n = cells(grid) as i64;
    let k = k.min(cells(grid) - mines).max(0);
    (comb(n - k as i64, mines as i64) as f64 / comb(n, mines as i64) as f64) * mult100(grid, mines, k) as f64 / 100.0
}

/// Laufende Runde (lamabank-Extra "round").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub bet: i64,
    pub grid: i32,
    pub mines: i32,
    pub n: i64,
    pub s: String,
    pub c: String,
    pub h: String,
    /// Aufgedeckte Plaetze in Klick-Reihenfolge.
    pub open: Vec<i32>,
}

impl Round {
    /// Als JSON (Reihenfolge wie das Python-dict).
    pub fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("bet".into(), Value::from(self.bet));
        m.insert("grid".into(), Value::from(self.grid));
        m.insert("mines".into(), Value::from(self.mines));
        m.insert("n".into(), Value::from(self.n));
        m.insert("s".into(), Value::from(self.s.clone()));
        m.insert("c".into(), Value::from(self.c.clone()));
        m.insert("h".into(), Value::from(self.h.clone()));
        m.insert("open".into(), Value::Array(self.open.iter().map(|v| Value::from(*v)).collect()));
        Value::Object(m)
    }
}

fn py_int(v: Option<&Value>) -> Option<i64> {
    fairness::py_int(v)
}

/// `clean_round(raw)` - gespeicherte, noch laufende Runde pruefen.
pub fn clean_round(raw: Option<&Value>) -> Option<Round> {
    let obj = raw?.as_object()?;
    let bet = py_int(obj.get("bet"))?;
    let grid = py_int(obj.get("grid"))?;
    let mines = py_int(obj.get("mines"))?;
    if bet <= 0 || !GRIDS.iter().any(|g| *g as i64 == grid) {
        return None;
    }
    let grid = grid as i32;
    if mines < 1 || mines > max_mines(grid) as i64 {
        return None;
    }
    let mines = mines as i32;
    let n = py_int(obj.get("n"))?;
    let s = obj.get("s")?.as_str()?;
    let c = obj.get("c")?.as_str()?;
    let h = obj.get("h")?.as_str()?;
    if !(fairness::valid_nonce(n)
        && fairness::valid_server(s)
        && fairness::valid_client(c)
        && fairness::valid_hash(h)
        && fairness::commit(s) == h)
    {
        return None;
    }
    let Some(Value::Array(list)) = obj.get("open") else {
        return None;
    };
    if list.len() >= gems(grid, mines) as usize {
        return None;
    }
    let mut open: Vec<i32> = Vec::with_capacity(list.len());
    for v in list {
        let x = py_int(Some(v))?;
        if !(0..cells(grid) as i64).contains(&x) || open.contains(&(x as i32)) {
            return None;
        }
        open.push(x as i32);
    }
    let field = field_from_seed(grid, mines, s, c, n);
    if open.iter().any(|v| field.contains(v)) {
        return None;
    }
    Some(Round { bet, grid, mines, n, s: s.to_string(), c: c.to_string(), h: h.to_string(), open })
}

/// `clean_history(raw)` - Multiplikatoren x100 (0 = verloren), letzte 12.
pub fn clean_history(raw: Option<&Value>) -> Vec<i64> {
    let Some(Value::Array(list)) = raw else {
        return Vec::new();
    };
    let out: Vec<i64> =
        list.iter().filter_map(|v| py_int(Some(v))).filter(|v| (0..=MAX_MULT100).contains(v)).collect();
    let cut = out.len().saturating_sub(HISTORY);
    out[cut..].to_vec()
}

/// Auto-Markierungen je Feldgroesse (Index 0..3 = GRIDS).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Marks {
    pub lists: [Vec<i32>; 4],
}

impl Marks {
    /// Markierungen einer Feldgroesse.
    pub fn get(&self, grid: i32) -> &Vec<i32> {
        &self.lists[GRIDS.iter().position(|g| *g == grid).unwrap_or(1)]
    }

    /// Markierungen einer Feldgroesse (veraenderbar).
    pub fn get_mut(&mut self, grid: i32) -> &mut Vec<i32> {
        &mut self.lists[GRIDS.iter().position(|g| *g == grid).unwrap_or(1)]
    }

    /// Als JSON `{"4": [...], "5": [...], "6": [...], "7": [...]}`.
    pub fn to_value(&self) -> Value {
        let mut m = Map::new();
        for (g, l) in GRIDS.iter().zip(self.lists.iter()) {
            m.insert(g.to_string(), Value::Array(l.iter().map(|v| Value::from(*v)).collect()));
        }
        Value::Object(m)
    }
}

/// `clean_marks(raw)` - immer alle vier Groessen, eindeutig, sortiert, <= N-1.
pub fn clean_marks(raw: Option<&Value>) -> Marks {
    let obj = raw.and_then(|v| v.as_object());
    let mut out = Marks::default();
    for (i, grid) in GRIDS.iter().enumerate() {
        let mut marks: Vec<i32> = Vec::new();
        if let Some(Value::Array(list)) = obj.and_then(|o| o.get(&grid.to_string())) {
            for v in list {
                if let Some(c) = py_int(Some(v)) {
                    if (0..cells(*grid) as i64).contains(&c) && !marks.contains(&(c as i32)) {
                        marks.push(c as i32);
                    }
                }
            }
        }
        marks.sort_unstable();
        marks.truncate(max_mines(*grid) as usize);
        out.lists[i] = marks;
    }
    out
}

#[cfg(test)]
mod tests {
    //! Gleichstand mit Python: Pruefwerte aus `devtools/export_chiplogic_ref.py`
    //! (Datei `data/chiplogic_ref.json`). Aufruf:
    //! `cargo test -p pgz-games --offline --lib -- chiplogic`
    use super::*;
    use pgz_core::{chipstats, fairness as f};
    use pgz_engine::py;

    fn refdata() -> Value {
        serde_json::from_str(include_str!("data/chiplogic_ref.json")).unwrap()
    }

    fn js(v: &Value) -> String {
        serde_json::to_string(v).unwrap()
    }

    fn opt_value<T>(v: Option<T>, to: impl Fn(&T) -> Value) -> Value {
        v.as_ref().map(to).unwrap_or(Value::Null)
    }

    #[test]
    fn chiplogic_sha256_wie_python() {
        let r = refdata();
        for (text, want) in r["sha"].as_object().unwrap() {
            assert_eq!(f::sha256_hex(text), want.as_str().unwrap(), "{text}");
        }
    }

    #[test]
    fn chiplogic_fairness_seeds_wie_python() {
        let r = refdata();
        let fr = &r["fair"];
        let mut rng = py::Random::new(fr["seed"].as_u64().unwrap());
        let mut st = f::blank_state(&mut |k| rng.getrandbits(k));
        assert_eq!(js(&st.to_value()), js(&fr["blank"]));
        let e = f::begin(&st);
        assert_eq!(js(&e.to_value()), js(&fr["begin"]));
        f::advance(&mut st, &mut |k| rng.getrandbits(k));
        f::new_client(&mut st, &mut |k| rng.getrandbits(k));
        assert_eq!(js(&st.to_value()), js(&fr["after"]));
        let mut r3 = py::Random::new(fr["clean_seed"].as_u64().unwrap());
        for (raw, want) in fr["clean_in"].as_array().unwrap().iter().zip(fr["clean_out"].as_array().unwrap()) {
            let got = f::clean_state(Some(raw), &mut |k| r3.getrandbits(k));
            assert_eq!(js(&got.to_value()), js(want));
        }
        let rd = &r["rand"];
        let mut rr = f::round_rand(
            rd["game"].as_str().unwrap(),
            rd["s"].as_str().unwrap(),
            rd["c"].as_str().unwrap(),
            rd["n"].as_i64().unwrap(),
        );
        let got: Vec<Value> = (0..5).map(|_| Value::from(rr.next_u32())).collect();
        assert_eq!(js(&Value::Array(got)), js(&rd["u32"]));
        let log = f::clean_log(Some(&r["log"]["in"]));
        assert_eq!(js(&f::log_to_value(&log)), js(&r["log"]["out"]));
        let ver: Vec<Value> = log.iter().map(|e| Value::from(e.verify())).collect();
        assert_eq!(js(&Value::Array(ver)), js(&r["log"]["verify"]));
    }

    #[test]
    fn chiplogic_chipstats_wie_python() {
        let r = refdata();
        let s = &r["stats"];
        let mut st = chipstats::blank();
        for op in s["ops"].as_array().unwrap() {
            let o: Vec<i64> = op.as_array().unwrap().iter().map(|v| v.as_i64().unwrap()).collect();
            st.record(o[0], o[1], o[2]);
        }
        assert_eq!(js(&st.to_value()), js(&s["after"]));
        assert_eq!(st.rtp().unwrap(), s["rtp"].as_str().unwrap().parse::<f64>().unwrap());
        assert_eq!(st.win_rate().unwrap(), s["win_rate"].as_str().unwrap().parse::<f64>().unwrap());
        for (raw, want) in s["clean_in"].as_array().unwrap().iter().zip(s["clean_out"].as_array().unwrap()) {
            assert_eq!(js(&chipstats::clean(Some(raw)).to_value()), js(want));
        }
    }

    #[test]
    fn chiplogic_mines_wie_python() {
        let r = refdata();
        let m = &r["mines"];
        let (mut mult, mut safe, mut pay, mut all) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for g in GRIDS {
            for mi in 1..cells(g) {
                let ks = 0..=gems(g, mi) + 1;
                mult.push(Value::Array(ks.clone().map(|k| Value::from(mult100(g, mi, k))).collect()));
                safe.push(Value::Array(ks.clone().map(|k| Value::from(safe_percent(g, mi, k))).collect()));
                pay.push(Value::Array(ks.clone().map(|k| Value::from(payout(55, g, mi, k))).collect()));
                all.push(Value::Array(ks.map(|k| Value::from(all_safe_percent(g, mi, k))).collect()));
            }
        }
        assert_eq!(f::sha256_hex(&js(&Value::Array(mult))), m["digest_mult"].as_str().unwrap());
        assert_eq!(f::sha256_hex(&js(&Value::Array(safe))), m["digest_safe"].as_str().unwrap());
        assert_eq!(f::sha256_hex(&js(&Value::Array(pay))), m["digest_pay55"].as_str().unwrap());
        assert_eq!(f::sha256_hex(&js(&Value::Array(all))), m["digest_allsafe"].as_str().unwrap());
        let m53: Vec<Value> = (0..8).map(|k| Value::from(mult100(5, 3, k))).collect();
        assert_eq!(js(&Value::Array(m53)), js(&m["mult_5_3"]));
        let m71: Vec<Value> = (0..6).map(|k| Value::from(mult100(7, 1, k))).collect();
        assert_eq!(js(&Value::Array(m71)), js(&m["mult_7_1"]));
        for fc in m["fields"].as_array().unwrap() {
            let a = fc.as_array().unwrap();
            let got = field_from_seed(
                a[0].as_i64().unwrap() as i32,
                a[1].as_i64().unwrap() as i32,
                a[2].as_str().unwrap(),
                a[3].as_str().unwrap(),
                a[4].as_i64().unwrap(),
            );
            assert_eq!(js(&serde_json::json!(got)), js(&a[5]));
        }
        for (raw, want) in m["rounds_in"].as_array().unwrap().iter().zip(m["rounds_out"].as_array().unwrap()) {
            assert_eq!(js(&opt_value(clean_round(Some(raw)), |x| x.to_value())), js(want), "{raw}");
        }
        for (raw, want) in m["marks_in"].as_array().unwrap().iter().zip(m["marks_out"].as_array().unwrap()) {
            assert_eq!(js(&clean_marks(Some(raw)).to_value()), js(want));
        }
        for (raw, want) in m["history_in"].as_array().unwrap().iter().zip(m["history_out"].as_array().unwrap()) {
            assert_eq!(js(&serde_json::json!(clean_history(Some(raw)))), js(want));
        }
        for e in m["expected"].as_array().unwrap() {
            let a = e.as_array().unwrap();
            let (g, mi, k) = (a[0].as_i64().unwrap() as i32, a[1].as_i64().unwrap() as i32, a[2].as_i64().unwrap() as i32);
            assert_eq!(expected_return(g, mi, k), a[3].as_str().unwrap().parse::<f64>().unwrap());
        }
        let lad: Vec<Value> =
            [0, 20, 22].iter().map(|k| serde_json::json!(ladder(5, 3, *k, 5).iter().map(|(j, v)| vec![*j as i64, *v]).collect::<Vec<_>>())).collect();
        assert_eq!(js(&Value::Array(lad)), js(&m["ladder"]));
        let text = vec![fmt_mult(1142, ","), fmt_mult(10_000_000, "."), fmt_percent(845, ","), fmt_percent(5, ".")];
        assert_eq!(js(&serde_json::json!(text)), js(&m["text"]));
        let mut carry = 0;
        for c in m["carry"].as_array().unwrap() {
            let a = c.as_array().unwrap();
            let (win, nc) = payout_carry(
                a[0].as_i64().unwrap(),
                a[1].as_i64().unwrap() as i32,
                a[2].as_i64().unwrap() as i32,
                a[3].as_i64().unwrap() as i32,
                carry,
            );
            carry = nc;
            assert_eq!((win, carry), (a[4].as_i64().unwrap(), a[5].as_i64().unwrap()));
        }
        assert_eq!(clean_carry(Some(&Value::from(42))), 42);
        assert_eq!(clean_carry(Some(&Value::from(100))), 0);
        assert_eq!(clean_carry(Some(&Value::Bool(true))), 0);
        assert_eq!(next_mult100(5, 3, 22), None);
        assert!(is_max(7, 24, 25) && is_clear(5, 3, 22));
    }
}
