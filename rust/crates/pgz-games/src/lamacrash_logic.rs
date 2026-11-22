//! lamacrash_logic.rs
//! ==================
//! Regeln von Lama Crash - uebertragen aus `games/lamacrash_logic.py`
//! (Browser: `lamacrash_logic.js`). Alle Multiplikatoren sind **Ganzzahlen in
//! Hundertsteln** (m100: 235 = x2,35); bis auf `world_blend` (Zeichenhilfe)
//! kommt keine Gleitkommazahl vor.
//!
//! Ein Lama steigt im Ballon auf, der Multiplikator waechst mit der Zeit. Wer
//! aussteigt, bekommt Einsatz x aktuellen Multiplikator; platzt der Ballon
//! vorher, ist der noch fliegende Einsatz weg.
//!
//! **Platzpunkt** aus dem Fairness-Zufall der Runde
//! (`fairness::round_rand(KEY, server, client, nonce)`, mulberry32):
//!
//! ```text
//! r        = rand.next_u32()                       # 0 .. 2^32-1
//! crash100 = min(MAX, max(100, 97 * 2^32 // (2^32 - r)))
//! ```
//!
//! **Steigkurve** (ein Takt = 1/60 s):
//!
//! ```text
//! mu_0 = 1_000_000;   mu_(t+1) = mu_t + mu_t // 1000;   m100(t) = mu_t // 10_000
//! ```
//!
//! **Takt-Regel** (`tick`, verbindlich fuer alle Fassungen):
//!
//! ```text
//! t += 1;  m = mult_at(t)
//! 1. jeder noch fliegende Teil mit Auto-Ziel a: a <= m und a <= crash100
//!    -> genau a auszahlen
//! 2. crash100 >= MAX und m >= MAX  -> Mondlandung: alle fliegenden Teile
//!    bekommen MAX, Rundenende "moon"
//! 3. sonst m > crash100            -> Platzen, Rundenende "pop"
//! ```
//!
//! Gespeicherte Runde (Schluesselreihenfolge fest, siehe `Round::to_value`):
//! `{"n", "s", "c", "h", "t", "bets": [{"b", "a", "hm", "xm", "p", "man"}]}`.
//! Fairness-Ergebnis `{"x": crash100, "t": Endtakt, "b": [[b, hm, xm, p], ...]}`.

use std::sync::OnceLock;

use pgz_core::fairness;
use serde_json::{Map, Value};

/// Spiel-Schluessel (Lama-Bank, Fairness-Zufall).
pub const KEY: &str = "lamacrash";

/// Auszahlungsquote in Prozent.
pub const HOUSE: i64 = 97;
/// Hoechster Multiplikator x100 (x100.000 = Mondlandung).
pub const MAX_MULT100: i64 = 10_000_000;
/// Kleinster Ausstieg (x1,01) - auch kleinstes Auto-Ziel.
pub const MIN_CASH: i64 = 101;
/// Einsatzstufen je Einsatz (Mindesteinsatz = erste Stufe, siehe `lamabank::MIN_BET`).
pub const BETS: [i64; 10] = [10, 20, 50, 100, 200, 500, 1000, 2500, 5000, 10000];
/// Hoechstens so viele Einsaetze in einer Runde.
pub const MAX_BETS: usize = 2;
/// Laenge des Verlaufs (Platzpunkte).
pub const HISTORY: usize = 12;

/// Takte je Sekunde (Normal-Tempo).
pub const TPS: i64 = 60;
/// mu bei x1,00.
pub const MU0: i64 = 1_000_000;
/// Je Takt waechst mu um mu // GROWTH (~0,1 %).
pub const GROWTH: i64 = 1000;
/// m100 = mu // MU_UNIT.
pub const MU_UNIT: i64 = 10_000;
/// Erster Takt mit m100 >= MAX_MULT100.
pub const MOON_TICK: i64 = 11520;
/// Takte je 1/60 s im Turbo.
pub const SPEED_TURBO: i64 = 3;
/// Takte je 1/60 s beim Vorspulen (alle ausgestiegen).
pub const SPEED_FF: i64 = 16;

/// Auto-Ziele (Leiter fuer Pfeil links/rechts und das Mausrad).
pub const TARGETS: [i64; 43] = [
    101, 102, 105, 110, 115, 120, 125, 130, 140, 150, 160, 175, 200, 225, 250, 300, 350, 400, 500, 600, 750, 1000,
    1250, 1500, 2000, 2500, 3000, 4000, 5000, 7500, 10000, 15000, 20000, 25000, 50000, 75000, 100000, 250000, 500000,
    1000000, 2500000, 5000000, 10000000,
];
/// Standard-Auto-Ziel.
pub const DEFAULT_TARGET: i64 = 200;

/// Hoehenwelten: Wiese, Wolken, Gewitter, Nordlicht, Weltall, Mond.
pub const WORLDS: [&str; 6] = ["meadow", "clouds", "storm", "aurora", "space", "moon"];
/// Untergrenze (m100) jeder Welt.
pub const WORLD_AT: [i64; 6] = [100, 150, 300, 1000, 10000, MAX_MULT100];
/// Ueberblendung zur naechsten Welt: so viele Takte vor ihrer Schwelle.
pub const BLEND_TICKS: i64 = 90;

/// Erfolg "Polarlicht-Pilot": ab x10 ausgestiegen.
pub const BRAVE_MULT: i64 = 1000;
/// Erfolg "Weltraumlama": ab x1000 ausgestiegen.
pub const SPACE_MULT: i64 = 100000;
/// Erfolg "Haarscharf": von Hand ab x2 ...
pub const CLOSE_MIN: i64 = 200;
/// ... und hoechstens 5 % unter dem Platzpunkt.
pub const CLOSE_PCT: i64 = 95;

/// Statistik: Verteilung der Platzpunkte (Untergrenzen der Faecher).
pub const BUCKETS: [i64; 7] = [100, 150, 200, 500, 1000, 10000, 100000];

/// Kurve: sichtbare Zeit mindestens 8 s.
pub const CHART_MIN_TICKS: i64 = 8 * TPS;
const Y_STEPS: [i64; 24] = [
    10, 20, 25, 50, 100, 200, 250, 500, 1000, 2000, 2500, 5000, 10000, 20000, 25000, 50000, 100000, 200000, 250000,
    500000, 1000000, 2000000, 2500000, 5000000,
];
/// Sekunden.
const X_STEPS: [i64; 9] = [1, 2, 5, 10, 15, 20, 30, 60, 120];

/// 2^32.
pub const TWO32: i64 = 4_294_967_296;

// ===================================================== Steigkurve

/// mu und m100 je Takt 0 .. MOON_TICK (einmal aufgebaut, wie `_table()`).
struct Kurve {
    mu: Vec<i64>,
    m: Vec<i64>,
}

fn kurve() -> &'static Kurve {
    static K: OnceLock<Kurve> = OnceLock::new();
    K.get_or_init(|| {
        let n = (MOON_TICK + 1) as usize;
        let (mut mu_v, mut m_v) = (Vec::with_capacity(n), Vec::with_capacity(n));
        let mut mu = MU0;
        for _ in 0..n {
            mu_v.push(mu);
            m_v.push(MAX_MULT100.min(mu / MU_UNIT));
            mu += mu / GROWTH;
        }
        Kurve { mu: mu_v, m: m_v }
    })
}

/// Takt auf 0 .. MOON_TICK begrenzt (`max(0, min(MOON_TICK, int(tick)))`).
fn tick_index(tick: i64) -> usize {
    tick.clamp(0, MOON_TICK) as usize
}

/// `mu_at(tick)` - mu zum Takt (ungedeckelt; Takt auf 0 .. MOON_TICK begrenzt).
pub fn mu_at(tick: i64) -> i64 {
    kurve().mu[tick_index(tick)]
}

/// `mult_at(tick)` - Multiplikator x100 zum Takt (100 bei Takt 0, hoechstens
/// MAX_MULT100). Ein gebrochener Takt wird wie `int()` abgeschnitten
/// (`tf as i64`).
pub fn mult_at(tick: i64) -> i64 {
    kurve().m[tick_index(tick)]
}

/// `tick_at(m100)` - erster Takt mit `mult_at(t) >= m100` (<= 100 -> 0,
/// > MAX -> MOON_TICK); Binaersuche wie `bisect_left`.
pub fn tick_at(m100: i64) -> i64 {
    MOON_TICK.min(kurve().m.partition_point(|v| *v < m100) as i64)
}

// ===================================================== Platzpunkt

/// `crash_from_u32(r)` - Platzpunkt x100 aus einer 32-Bit-Zufallszahl.
pub fn crash_from_u32(r: u32) -> i64 {
    let v = (HOUSE * TWO32) / (TWO32 - r as i64);
    MAX_MULT100.min(100.max(v))
}

/// `crash_point(server, client, nonce)` - Platzpunkt einer Runde aus ihren
/// Fairness-Werten (zum Nachrechnen).
pub fn crash_point(server: &str, client: &str, nonce: i64) -> i64 {
    crash_from_u32(fairness::round_rand(KEY, server, client, nonce).next_u32())
}

/// `pop_tick(crash100)` - Takt, an dem die Runde endet: Mond -> MOON_TICK,
/// sonst der erste Takt mit `mult_at(t) > crash100`.
pub fn pop_tick(crash100: i64) -> i64 {
    if crash100 >= MAX_MULT100 {
        return MOON_TICK;
    }
    tick_at(crash100 + 1)
}

/// `win_count(t100)` - Anzahl der 2^32 Zufallswerte mit `crash100 >= t100`.
pub fn win_count(t100: i64) -> i64 {
    let t100 = MIN_CASH.max(MAX_MULT100.min(t100));
    (HOUSE * TWO32) / t100
}

fn gcd(a: i64, b: i64) -> i64 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

/// Gekuerzter Bruch (wie `Fraction`).
fn reduce(num: i64, den: i64) -> (i64, i64) {
    let g = gcd(num, den).max(1);
    (num / g, den / g)
}

/// `expected_return(t100)` - erwartete Auszahlung (Anteil am Einsatz) beim
/// Ausstieg bei t100, exakt als gekuerzter Bruch (Zaehler, Nenner).
pub fn expected_return(t100: i64) -> (i64, i64) {
    let t100 = MIN_CASH.max(MAX_MULT100.min(t100));
    reduce(t100 * win_count(t100), 100 * TWO32)
}

/// `fmt_chance(t100, sep)` - Chance, t100 zu erreichen, in Prozent
/// (kaufmaennisch gerundet): ab 1 % zwei Nachkommastellen ("48.50"),
/// darunter vier ("0.0010").
pub fn fmt_chance(t100: i64, sep: &str) -> String {
    let (num, den) = (win_count(t100), TWO32);
    if num * 100 >= den {
        let v = (num * 20000 + den) / (den * 2);
        format!("{}{}{:02}", v / 100, sep, v % 100)
    } else {
        let v = (num * 2_000_000 + den) / (den * 2);
        format!("{}{}{:04}", v / 10000, sep, v % 10000)
    }
}

// ===================================================== Auszahlung

/// `payout_carry(stake, m100, carry)` - (Auszahlung, neuer Uebertrag) mit
/// Rest-Hundertstel-Uebertrag wie bei Mines: `stake * m100 + carry` in
/// Hundertsteln, ganze Chips werden ausgezahlt, der Rest (0..99) bleibt.
pub fn payout_carry(stake: i64, m100: i64, carry: i64) -> (i64, i64) {
    let total = stake * m100 + carry;
    (total.div_euclid(100), total.rem_euclid(100))
}

/// `clean_carry(raw)` - gespeicherter Uebertrag: ganze Zahl 0..99, sonst 0.
pub fn clean_carry(raw: Option<&Value>) -> i64 {
    fairness::py_int(raw).filter(|v| (0..=99).contains(v)).unwrap_or(0)
}

/// `half_stake(b)` - Anteil, der beim Halbieren aussteigt.
pub fn half_stake(b: i64) -> i64 {
    b.div_euclid(2)
}

/// `rest_stake(b)` - Anteil, der nach dem Halbieren weiterfliegt.
pub fn rest_stake(b: i64) -> i64 {
    b - b.div_euclid(2)
}

/// Ein Teil-Einsatz der Runde (`{"b", "a", "hm", "xm", "p", "man"}`).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Bet {
    /// Einsatz.
    pub b: i64,
    /// Auto-Ziel (0 = aus).
    pub a: i64,
    /// Ausstieg der Haelfte (0 = nicht geteilt).
    pub hm: i64,
    /// Ausstieg des Rests bzw. des ganzen Einsatzes (0 = fliegt).
    pub xm: i64,
    /// Bisher ausgezahlt.
    pub p: i64,
    /// Bits 1/2 = Haelfte/Rest von Hand.
    pub man: i64,
}

impl Bet {
    /// Als JSON (Reihenfolge wie das Python-dict).
    pub fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("b".into(), Value::from(self.b));
        m.insert("a".into(), Value::from(self.a));
        m.insert("hm".into(), Value::from(self.hm));
        m.insert("xm".into(), Value::from(self.xm));
        m.insert("p".into(), Value::from(self.p));
        m.insert("man".into(), Value::from(self.man));
        Value::Object(m)
    }
}

/// Laufende Runde (lamabank-Extra "round").
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Round {
    pub n: i64,
    pub s: String,
    pub c: String,
    pub h: String,
    /// Takt.
    pub t: i64,
    pub bets: Vec<Bet>,
}

impl Round {
    /// Als JSON (Reihenfolge wie das Python-dict).
    pub fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("n".into(), Value::from(self.n));
        m.insert("s".into(), Value::from(self.s.clone()));
        m.insert("c".into(), Value::from(self.c.clone()));
        m.insert("h".into(), Value::from(self.h.clone()));
        m.insert("t".into(), Value::from(self.t));
        m.insert("bets".into(), Value::Array(self.bets.iter().map(Bet::to_value).collect()));
        Value::Object(m)
    }

    /// Fairness-Eintrag der Runde `{"n", "s", "c", "h"}`.
    pub fn entry(&self) -> fairness::Entry {
        fairness::Entry { n: self.n, s: self.s.clone(), c: self.c.clone(), h: self.h.clone() }
    }
}

/// `riding(bet)` - noch fliegender Einsatz eines Teil-Einsatzes (0 = ganz
/// ausgestiegen).
pub fn riding(bet: &Bet) -> i64 {
    if bet.xm != 0 {
        return 0;
    }
    if bet.hm != 0 {
        rest_stake(bet.b)
    } else {
        bet.b
    }
}

/// `flying(rnd)` - fliegt in der Runde noch Geld mit?
pub fn flying(rnd: &Round) -> bool {
    rnd.bets.iter().any(|b| riding(b) > 0)
}

/// `stake_total(rnd)` - Gesamteinsatz der Runde.
pub fn stake_total(rnd: &Round) -> i64 {
    rnd.bets.iter().map(|b| b.b).sum()
}

/// `paid_total(rnd)` - bisher ausgezahlt (alle Teile).
pub fn paid_total(rnd: &Round) -> i64 {
    rnd.bets.iter().map(|b| b.p).sum()
}

/// `target_ok(t100)` - gueltiges Auto-Ziel? (MIN_CASH .. MAX_MULT100)
pub fn target_ok(t100: i64) -> bool {
    (MIN_CASH..=MAX_MULT100).contains(&t100)
}

/// `target_ok(v)` fuer einen rohen JSON-Wert (Einstellungen): ganze Zahl,
/// kein bool, MIN_CASH .. MAX_MULT100.
pub fn target_ok_value(v: Option<&Value>) -> bool {
    fairness::py_int(v).is_some_and(target_ok)
}

/// `step_target(t100, direction)` - naechstes Auto-Ziel der Leiter in
/// Richtung direction (+1/-1).
pub fn step_target(t100: i64, direction: i64) -> i64 {
    let t100 = MIN_CASH.max(MAX_MULT100.min(t100));
    if direction > 0 {
        return TARGETS.iter().copied().find(|v| *v > t100).unwrap_or(TARGETS[TARGETS.len() - 1]);
    }
    TARGETS.iter().rev().copied().find(|v| *v < t100).unwrap_or(TARGETS[0])
}

/// `new_bet(b, a)` - Teil-Einsatz zu Rundenbeginn (a = Auto-Ziel oder 0).
pub fn new_bet(b: i64, a: i64) -> Bet {
    Bet { b, a, hm: 0, xm: 0, p: 0, man: 0 }
}

/// `new_round(entry, pairs)` - neue Runde aus dem Fairness-Eintrag
/// (`fairness::begin`) und den Einsaetzen `[(Einsatz, Auto-Ziel oder 0)]`.
pub fn new_round(entry: &fairness::Entry, pairs: &[(i64, i64)]) -> Round {
    Round {
        n: entry.n,
        s: entry.s.clone(),
        c: entry.c.clone(),
        h: entry.h.clone(),
        t: 0,
        bets: pairs.iter().map(|&(b, a)| new_bet(b, a)).collect(),
    }
}

/// `_pay(bet, m100, carry)` - den noch fliegenden Teil bei m100 auszahlen.
fn pay(bet: &mut Bet, m100: i64, carry: i64) -> (i64, i64) {
    let (win, carry) = payout_carry(riding(bet), m100, carry);
    bet.xm = m100;
    bet.p += win;
    (win, carry)
}

/// Ereignis eines Takts: (Index, "auto" | "moon", m100, Gewinn).
pub type Event = (usize, &'static str, i64, i64);

/// `tick(rnd, crash100, carry)` - einen Takt weiter (Takt-Regel oben).
/// Liefert (Ereignisse in Auszahlungsreihenfolge, Uebertrag, Ende):
/// Ende = `None` (fliegt weiter) / `Some("pop")` / `Some("moon")`.
pub fn tick(rnd: &mut Round, crash100: i64, carry: i64) -> (Vec<Event>, i64, Option<&'static str>) {
    let mut carry = carry;
    rnd.t += 1;
    let m = mult_at(rnd.t);
    let mut events: Vec<Event> = Vec::new();
    for (i, bet) in rnd.bets.iter_mut().enumerate() {
        let a = bet.a;
        if a != 0 && riding(bet) > 0 && a <= m && a <= crash100 {
            let (win, c) = pay(bet, a, carry);
            carry = c;
            events.push((i, "auto", a, win));
        }
    }
    if crash100 >= MAX_MULT100 && m >= MAX_MULT100 {
        for (i, bet) in rnd.bets.iter_mut().enumerate() {
            if riding(bet) > 0 {
                let (win, c) = pay(bet, MAX_MULT100, carry);
                carry = c;
                events.push((i, "moon", MAX_MULT100, win));
            }
        }
        return (events, carry, Some("moon"));
    }
    if m > crash100 {
        return (events, carry, Some("pop"));
    }
    (events, carry, None)
}

/// `cash_out(rnd, i, half, carry)` - von Hand aussteigen (ganz oder Haelfte)
/// beim aktuellen Takt. Liefert (Gewinn, m100, neuer Uebertrag) oder `None`
/// (unter MIN_CASH, nichts mehr im Flug, Haelfte schon genommen). Nur
/// aufrufen, solange die Runde fliegt - dann ist `mult_at(t) <= crash100`.
pub fn cash_out(rnd: &mut Round, i: usize, half: bool, carry: i64) -> Option<(i64, i64, i64)> {
    let m = mult_at(rnd.t);
    let bet = rnd.bets.get_mut(i)?;
    if m < MIN_CASH || riding(bet) <= 0 {
        return None;
    }
    let (win, carry) = if half {
        if bet.hm != 0 {
            return None;
        }
        let r = payout_carry(half_stake(bet.b), m, carry);
        bet.hm = m;
        bet.man |= 1;
        r
    } else {
        let r = payout_carry(riding(bet), m, carry);
        bet.xm = m;
        bet.man |= 2;
        r
    };
    bet.p += win;
    Some((win, m, carry))
}

/// `best_cash(rnd)` - hoechster Ausstieg der Runde (m100, 0 = keiner).
pub fn best_cash(rnd: &Round) -> i64 {
    rnd.bets.iter().fold(0, |best, b| best.max(b.hm).max(b.xm))
}

/// `close_call(rnd, crash100)` - "Haarscharf": ein Ausstieg VON HAND ab
/// CLOSE_MIN und hoechstens 5 % unter dem Platzpunkt
/// (`m * 100 >= CLOSE_PCT * crash100`).
pub fn close_call(rnd: &Round, crash100: i64) -> bool {
    rnd.bets.iter().any(|b| {
        [(1, b.hm), (2, b.xm)]
            .iter()
            .any(|&(bit, m)| b.man & bit != 0 && m >= CLOSE_MIN && m * 100 >= CLOSE_PCT * crash100)
    })
}

/// `outcome(rnd, crash100)` - Ergebnis fuer das Fairness-Log (nur ganze
/// Zahlen, Tiefe 3): `{"x": crash100, "t": Takt, "b": [[b, hm, xm, p], ...]}`.
pub fn outcome(rnd: &Round, crash100: i64) -> Value {
    let mut m = Map::new();
    m.insert("x".into(), Value::from(crash100));
    m.insert("t".into(), Value::from(rnd.t));
    let rows: Vec<Value> = rnd.bets.iter().map(|b| serde_json::json!([b.b, b.hm, b.xm, b.p])).collect();
    m.insert("b".into(), Value::Array(rows));
    Value::Object(m)
}

/// `check_outcome(o, crash100)` - passt ein Log-Ergebnis zum nachgerechneten
/// Platzpunkt? (Platzpunkt gleich, jeder Ausstieg 0 oder MIN_CASH .. crash100)
pub fn check_outcome(o: Option<&Value>, crash100: i64) -> bool {
    let Some(obj) = o.and_then(|v| v.as_object()) else {
        return false;
    };
    if fairness::py_int(obj.get("x")) != Some(crash100) {
        return false;
    }
    let Some(Value::Array(bets)) = obj.get("b") else {
        return false;
    };
    if !(1..=MAX_BETS).contains(&bets.len()) {
        return false;
    }
    for row in bets {
        let Some(row) = row.as_array() else {
            return false;
        };
        if row.len() != 4 || !row.iter().all(|v| fairness::py_int(Some(v)).is_some()) {
            return false;
        }
        for v in [&row[1], &row[2]] {
            let m = fairness::py_int(Some(v)).unwrap_or(0);
            if m != 0 && !(MIN_CASH..=crash100).contains(&m) {
                return false;
            }
        }
    }
    true
}

// ===================================================== Hoehenwelten / Kurve

/// `world_of(m100)` - Welt (0 = Wiese .. 5 = Mond) zum Multiplikator.
pub fn world_of(m100: i64) -> usize {
    let mut w = 0;
    for (i, lo) in WORLD_AT.iter().enumerate() {
        if m100 >= *lo {
            w = i;
        }
    }
    w
}

/// `world_ticks()` - Takt, an dem jede Welt beginnt (Wiese = 0).
pub fn world_ticks() -> [i64; 6] {
    static T: OnceLock<[i64; 6]> = OnceLock::new();
    *T.get_or_init(|| WORLD_AT.map(tick_at))
}

/// `world_blend(tick_f)` - (Welt, Anteil 0..255 der naechsten Welt) zum
/// (gebrochenen) Takt; die Ueberblendung beginnt BLEND_TICKS vor der naechsten
/// Schwelle. Nur fuers Zeichnen; die Mond-Welt (5) erreicht die Kurve nie von
/// selbst (Weltall blendet nicht ueber).
pub fn world_blend(tick_f: f64) -> (usize, i64) {
    let starts = world_ticks();
    let mut w = 0;
    for (i, st) in starts.iter().enumerate().take(5).skip(1) {
        if tick_f >= *st as f64 {
            w = i;
        }
    }
    if w >= 4 {
        return (4, 0);
    }
    let nxt = starts[w + 1];
    let left = nxt as f64 - tick_f;
    if left >= BLEND_TICKS as f64 {
        return (w, 0);
    }
    let share = ((BLEND_TICKS as f64 - left) * 255.0 / BLEND_TICKS as f64) as i64;
    (w, share.clamp(0, 255))
}

/// `chart_span(tick)` - Achsen der Kurve (wachsen mit):
/// (x_max in Takten, y_max in m100).
pub fn chart_span(tick: i64) -> (i64, i64) {
    let x_max = CHART_MIN_TICKS.max(tick + tick.div_euclid(5));
    let m = mult_at(tick);
    let y_max = 200.max(m + (m - 100).div_euclid(4));
    (x_max, y_max)
}

/// `chart_ylines(y_max)` - waagerechte Hilfslinien (m100-Werte ueber x1,00,
/// hoechstens 4).
pub fn chart_ylines(y_max: i64) -> Vec<i64> {
    let span = 1.max(y_max - 100);
    let step = Y_STEPS.iter().copied().find(|s| span.div_euclid(*s) <= 4).unwrap_or(Y_STEPS[Y_STEPS.len() - 1]);
    let first = (100 / step + 1) * step;
    let mut out = Vec::new();
    let mut v = first;
    while v <= y_max && out.len() < 4 {
        out.push(v);
        v += step;
    }
    out
}

/// `chart_xlines(x_max)` - senkrechte Hilfslinien als Takte (ganze
/// Sekunden, hoechstens 5).
pub fn chart_xlines(x_max: i64) -> Vec<i64> {
    let step = X_STEPS.iter().copied().find(|s| x_max.div_euclid(s * TPS) <= 5).unwrap_or(X_STEPS[X_STEPS.len() - 1]);
    (1..=x_max.div_euclid(step * TPS)).map(|v| v * step * TPS).take(5).collect()
}

// ===================================================== Statistik

/// `bucket_of(crash100)` - Fach der Verteilung (0 .. BUCKETS.len()-1).
pub fn bucket_of(crash100: i64) -> usize {
    let mut b = 0;
    for (i, lo) in BUCKETS.iter().enumerate() {
        if crash100 >= *lo {
            b = i;
        }
    }
    b
}

/// `bucket_prob(i)` - erwartete Wahrscheinlichkeit eines Fachs, exakt als
/// gekuerzter Bruch (Zaehler, Nenner).
pub fn bucket_prob(i: usize) -> (i64, i64) {
    let lo = BUCKETS[i];
    let num_lo = if lo <= 100 { TWO32 } else { win_count(lo) };
    if i + 1 >= BUCKETS.len() {
        return reduce(num_lo, TWO32);
    }
    reduce(num_lo - win_count(BUCKETS[i + 1]), TWO32)
}

/// `float(bucket_prob(i))` - der Nenner ist eine Zweierpotenz und der Zaehler
/// < 2^53, die Division ist also exakt wie in Python.
pub fn bucket_prob_f(i: usize) -> f64 {
    let (num, den) = bucket_prob(i);
    num as f64 / den as f64
}

/// Namen der Crash-Zaehler in `stats["x"]` (nach `hits`).
pub const X_COUNTERS: [&str; 7] = ["cash", "half", "auto", "inst", "moon", "close", "top"];

/// Crash-Zaehler `stats["x"]`: Treffer je Fach und sieben Zaehler.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct XStats {
    /// Platzpunkte je Fach (`BUCKETS.len()` Werte).
    pub hits: Vec<i64>,
    /// Ausstiege (Haelften und ganze bzw. Rest).
    pub cash: i64,
    /// Halbierungen.
    pub half: i64,
    /// Auto-Ziel- bzw. Mond-Auszahlungen.
    pub auto: i64,
    /// Runden mit Platzen bei x1,00.
    pub inst: i64,
    /// Mondlandungen.
    pub moon: i64,
    /// "Haarscharf"-Runden.
    pub close: i64,
    /// Hoechster Platzpunkt.
    pub top: i64,
}

impl XStats {
    /// Zaehler nach Namen (`X_COUNTERS`; unbekannt = 0).
    pub fn counter(&self, name: &str) -> i64 {
        match name {
            "cash" => self.cash,
            "half" => self.half,
            "auto" => self.auto,
            "inst" => self.inst,
            "moon" => self.moon,
            "close" => self.close,
            "top" => self.top,
            _ => 0,
        }
    }

    fn set(&mut self, name: &str, v: i64) {
        match name {
            "cash" => self.cash = v,
            "half" => self.half = v,
            "auto" => self.auto = v,
            "inst" => self.inst = v,
            "moon" => self.moon = v,
            "close" => self.close = v,
            "top" => self.top = v,
            _ => {}
        }
    }

    /// Als JSON-Objekt (Reihenfolge wie das Python-dict: hits, dann X_COUNTERS).
    pub fn to_map(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("hits".into(), Value::from(self.hits.clone()));
        for k in X_COUNTERS {
            m.insert(k.into(), Value::from(self.counter(k)));
        }
        m
    }

    /// Als JSON.
    pub fn to_value(&self) -> Value {
        Value::Object(self.to_map())
    }
}

/// `blank_x()` - leere Crash-Zaehler.
pub fn blank_x() -> XStats {
    XStats { hits: vec![0; BUCKETS.len()], ..Default::default() }
}

/// `clean_x(raw)` - hits = Liste mit `BUCKETS.len()` Zahlen >= 0 (sonst
/// Nullen); jeder Zaehler ganze Zahl >= 0, sonst 0.
pub fn clean_x(raw: Option<&Value>) -> XStats {
    let mut out = blank_x();
    let Some(obj) = raw.and_then(|v| v.as_object()) else {
        return out;
    };
    let ok = |v: Option<&Value>| fairness::py_int(v).filter(|x| (0..=fairness::MAX_NONCE).contains(x));
    if let Some(Value::Array(hits)) = obj.get("hits") {
        if hits.len() == BUCKETS.len() {
            out.hits = hits.iter().map(|v| ok(Some(v)).unwrap_or(0)).collect();
        }
    }
    for k in X_COUNTERS {
        if let Some(v) = ok(obj.get(k)) {
            out.set(k, v);
        }
    }
    out
}

/// `record_x(x, rnd, crash100, end)` - eine fertige Runde eintragen
/// (`end` = "pop" / "moon").
pub fn record_x(x: &mut XStats, rnd: &Round, crash100: i64, end: &str) {
    if let Some(h) = x.hits.get_mut(bucket_of(crash100)) {
        *h += 1;
    }
    for b in &rnd.bets {
        if b.hm != 0 {
            x.cash += 1;
            x.half += 1;
        }
        if b.xm != 0 {
            x.cash += 1;
            if b.man & 2 == 0 {
                x.auto += 1;
            }
        }
    }
    if crash100 < MIN_CASH {
        x.inst += 1;
    }
    if end == "moon" {
        x.moon += 1;
    }
    if close_call(rnd, crash100) {
        x.close += 1;
    }
    x.top = x.top.max(crash100);
}

// ===================================================== Pruefen / Laden

/// `clean_history(raw)` - Verlauf der letzten Runden: Platzpunkte x100.
pub fn clean_history(raw: Option<&Value>) -> Vec<i64> {
    let Some(Value::Array(list)) = raw else {
        return Vec::new();
    };
    let out: Vec<i64> =
        list.iter().filter_map(|v| fairness::py_int(Some(v))).filter(|v| (100..=MAX_MULT100).contains(v)).collect();
    let cut = out.len().saturating_sub(HISTORY);
    out[cut..].to_vec()
}

/// `_clean_bet(raw, m, crash100)`
fn clean_bet(raw: &Value, m: i64, crash100: i64) -> Option<Bet> {
    let obj = raw.as_object()?;
    let get = |k: &str| fairness::py_int(obj.get(k));
    let (b, a, hm, xm, p, man) = (get("b")?, get("a")?, get("hm")?, get("xm")?, get("p")?, get("man")?);
    if !BETS.contains(&b) || !(a == 0 || target_ok(a)) || !(0..=3).contains(&man) || p < 0 {
        return None;
    }
    for v in [hm, xm] {
        if v != 0 && !(MIN_CASH..=m.min(crash100)).contains(&v) {
            return None;
        }
    }
    if hm != 0 && xm != 0 && xm < hm {
        return None;
    }
    if (man & 1 != 0 && hm == 0) || (man & 2 != 0 && xm == 0) {
        return None;
    }
    if p > b * MAX_MULT100 / 100 + 1 {
        return None;
    }
    let bet = Bet { b, a, hm, xm, p, man };
    // ein schon erreichtes Auto-Ziel muss ausgezahlt sein
    if a != 0 && riding(&bet) > 0 && a <= m && a <= crash100 {
        return None;
    }
    Some(bet)
}

/// `clean_round(raw)` - gespeicherte, noch offene Runde pruefen (aus dem
/// lamabank-Extra "round").
///
/// Gueltig: Fairness-Werte mit h = sha256(s), Takt 0 .. MOON_TICK vor dem
/// Rundenende (Platzpunkt aus den Seeds neu berechnet), 1 .. MAX_BETS
/// Einsaetze mit Ausstiegen hoechstens `min(mult_at(t), crash100)`. Eine
/// Runde ohne fliegende Teile ist gueltig (das Spiel schliesst sie beim Laden
/// ab). Sonst `None`.
pub fn clean_round(raw: Option<&Value>) -> Option<Round> {
    let obj = raw?.as_object()?;
    let n = fairness::py_int(obj.get("n"))?;
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
    let t = fairness::py_int(obj.get("t"))?;
    if !(0..=MOON_TICK).contains(&t) {
        return None;
    }
    let crash100 = crash_point(s, c, n);
    let m = mult_at(t);
    if crash100 >= MAX_MULT100 {
        if t >= MOON_TICK {
            return None;
        }
    } else if m > crash100 {
        return None;
    }
    let Some(Value::Array(raw_bets)) = obj.get("bets") else {
        return None;
    };
    if !(1..=MAX_BETS).contains(&raw_bets.len()) {
        return None;
    }
    let mut bets = Vec::with_capacity(raw_bets.len());
    for rb in raw_bets {
        bets.push(clean_bet(rb, m, crash100)?);
    }
    Some(Round { n, s: s.to_string(), c: c.to_string(), h: h.to_string(), t, bets })
}

// ===================================================== Formatierung

/// `fmt_mult(m100, sep)` - Multiplikator x100 als Text mit zwei Stellen:
/// 235 -> "2.35".
pub fn fmt_mult(m100: i64, sep: &str) -> String {
    format!("{}{}{:02}", m100.div_euclid(100), sep, m100.rem_euclid(100))
}

/// `fmt_short(m100, sep)` - Kurzform fuer Pillen/Leiste (ohne x):
/// 1,23 - 12,3 - 123 - 12,3k - 100k.
pub fn fmt_short(m100: i64, sep: &str) -> String {
    if m100 < 1000 {
        format!("{}{}{:02}", m100.div_euclid(100), sep, m100.rem_euclid(100))
    } else if m100 < 10000 {
        format!("{}{}{}", m100 / 100, sep, (m100 / 10) % 10)
    } else if m100 < 100000 {
        (m100 / 100).to_string()
    } else if m100 < 1000000 {
        format!("{}{}{}k", m100 / 100000, sep, (m100 / 10000) % 10)
    } else {
        format!("{}k", m100 / 100000)
    }
}

#[cfg(test)]
mod tests {
    //! Gleichstand mit Python: Pruefwerte aus `devtools/export_chiplogic_ref.py`
    //! (Abschnitt "crash" der Datei `data/chiplogic_ref.json`). Aufruf:
    //! `cargo test -p pgz-games --offline --lib -- chiplogic`
    use std::collections::VecDeque;

    use super::*;
    use pgz_core::fairness as f;
    use serde_json::json;

    fn refdata() -> Value {
        serde_json::from_str(include_str!("data/chiplogic_ref.json")).unwrap()
    }

    fn js(v: &Value) -> String {
        serde_json::to_string(v).unwrap()
    }

    fn int(v: &Value) -> i64 {
        v.as_i64().unwrap()
    }

    fn arr(v: &Value) -> &Vec<Value> {
        v.as_array().unwrap()
    }

    /// Eine ganze Runde wie `_crash_sims` in export_chiplogic_ref.py:
    /// Hand-Ausstiege je Takt, dann `tick` - Protokoll wie dort.
    fn sim(sim: &Value) -> (Round, i64, Vec<Value>, &'static str) {
        let pairs: Vec<(i64, i64)> = arr(&sim["pairs"]).iter().map(|p| (int(&p[0]), int(&p[1]))).collect();
        let crash = int(&sim["crash"]);
        let mut todo: VecDeque<(i64, usize, bool)> = arr(&sim["acts"])
            .iter()
            .map(|a| (int(&a[0]), int(&a[1]) as usize, a[2].as_bool().unwrap()))
            .collect();
        let entry = f::Entry { n: 1, s: "0".repeat(32), c: "abc".into(), h: "0".repeat(64) };
        let mut rnd = new_round(&entry, &pairs);
        let mut carry = int(&sim["carry0"]);
        let mut log: Vec<Value> = Vec::new();
        let mut end = None;
        while end.is_none() {
            while todo.front().is_some_and(|a| a.0 == rnd.t) {
                let (_, i, half) = todo.pop_front().unwrap();
                let res = cash_out(&mut rnd, i, half, carry);
                let mut row = vec![json!("cash"), json!(rnd.t), json!(i), json!(half)];
                match res {
                    Some((win, m, c)) => {
                        row.extend([json!(win), json!(m), json!(c)]);
                        carry = c;
                    }
                    None => row.extend([Value::Null, Value::Null, Value::Null]),
                }
                log.push(Value::Array(row));
            }
            let (evs, c, e) = tick(&mut rnd, crash, carry);
            carry = c;
            end = e;
            for (i, kind, m, win) in evs {
                log.push(json!(["ev", rnd.t, i, kind, m, win]));
            }
        }
        (rnd, carry, log, end.unwrap())
    }

    #[test]
    fn chiplogic_crash_wie_python() {
        let r = refdata();
        let x = &r["crash"];
        // ---- Konstanten und Tabellen
        let consts = json!([
            HOUSE,
            MAX_MULT100,
            MIN_CASH,
            MU0,
            GROWTH,
            MU_UNIT,
            MOON_TICK,
            SPEED_TURBO,
            SPEED_FF,
            HISTORY,
            DEFAULT_TARGET,
            BLEND_TICKS,
            CHART_MIN_TICKS,
            BRAVE_MULT,
            SPACE_MULT,
            CLOSE_MIN,
            CLOSE_PCT,
            MAX_BETS,
            TPS
        ]);
        assert_eq!(js(&consts), js(&x["consts"]));
        assert_eq!(js(&json!(BETS)), js(&x["bets"]));
        assert_eq!(js(&json!(TARGETS[..])), js(&x["targets"]));
        assert_eq!(js(&json!(WORLD_AT)), js(&x["world_at"]));
        assert_eq!(js(&json!(BUCKETS)), js(&x["buckets"]));
        // ---- Steigkurve
        let curve: Vec<i64> = (0..=MOON_TICK).map(mult_at).collect();
        assert_eq!(f::sha256_hex(&js(&json!(curve))), x["digest_curve"].as_str().unwrap());
        for c in arr(&x["curve"]) {
            let tk = int(&c[0]);
            assert_eq!((mult_at(tk), mu_at(tk)), (int(&c[1]), int(&c[2])), "Takt {tk}");
        }
        assert_eq!((mult_at(-5), mult_at(MOON_TICK + 99)), (100, MAX_MULT100));
        for c in arr(&x["tick_at"]) {
            assert_eq!(tick_at(int(&c[0])), int(&c[1]), "tick_at {}", c[0]);
        }
        // ---- Platzpunkt
        for c in arr(&x["u32"]) {
            assert_eq!(crash_from_u32(c[0].as_u64().unwrap() as u32), int(&c[1]), "u32 {}", c[0]);
        }
        for c in arr(&x["seeds"]) {
            let got = crash_point(c[0].as_str().unwrap(), c[1].as_str().unwrap(), int(&c[2]));
            assert_eq!(got, int(&c[3]), "{c}");
        }
        for c in arr(&x["win_count"]) {
            assert_eq!(win_count(int(&c[0])), int(&c[1]));
        }
        for c in arr(&x["chance"]) {
            let t100 = int(&c[0]);
            assert_eq!(fmt_chance(t100, "."), c[1].as_str().unwrap());
            assert_eq!(fmt_chance(t100, ","), c[2].as_str().unwrap());
        }
        for c in arr(&x["pop_tick"]) {
            assert_eq!(pop_tick(int(&c[0])), int(&c[1]), "pop_tick {}", c[0]);
        }
        // ---- ganze Runden (Hand-Ausstiege, Auto-Ziele, Mond)
        let mut xs = blank_x();
        for (k, want) in arr(&x["sims"]).iter().enumerate() {
            let crash = int(&want["crash"]);
            let (rnd, carry, log, end) = sim(want);
            assert_eq!(js(&Value::Array(log)), js(&want["log"]), "Runde {k}");
            assert_eq!(end, want["end"].as_str().unwrap(), "Runde {k}");
            assert_eq!(rnd.t, int(&want["t"]), "Runde {k}");
            assert_eq!(js(&Value::Array(rnd.bets.iter().map(Bet::to_value).collect())), js(&want["bets"]));
            assert_eq!(carry, int(&want["carry"]), "Runde {k}");
            assert_eq!(js(&outcome(&rnd, crash)), js(&want["outcome"]), "Runde {k}");
            assert_eq!(best_cash(&rnd), int(&want["best"]), "Runde {k}");
            assert_eq!(close_call(&rnd, crash), want["close"].as_bool().unwrap(), "Runde {k}");
            assert!(check_outcome(Some(&outcome(&rnd, crash)), crash), "Runde {k}");
            assert!(!check_outcome(Some(&outcome(&rnd, crash)), crash + 1), "Runde {k}");
            assert!(!flying(&rnd) || end == "pop", "Runde {k}");
            if k < 12 {
                record_x(&mut xs, &rnd, crash, end);
            }
        }
        assert_eq!(js(&xs.to_value()), js(&x["x"]));
        for (raw, want) in arr(&x["x_clean_in"]).iter().zip(arr(&x["x_clean_out"])) {
            assert_eq!(js(&clean_x(Some(raw)).to_value()), js(want), "{raw}");
        }
        // ---- Laden
        for (raw, want) in arr(&x["rounds_in"]).iter().zip(arr(&x["rounds_out"])) {
            let got = clean_round(Some(raw)).map(|r| r.to_value()).unwrap_or(Value::Null);
            assert_eq!(js(&got), js(want), "{raw}");
        }
        assert_eq!(js(&json!(clean_history(Some(&x["history_in"])))), js(&x["history_out"]));
        // ---- Texte, Kurve, Welten, Leiter, Statistik
        for c in arr(&x["fmt"]) {
            let v = int(&c[0]);
            let got = json!([v, fmt_mult(v, "."), fmt_mult(v, ","), fmt_short(v, "."), fmt_short(v, ",")]);
            assert_eq!(js(&got), js(c));
        }
        for c in arr(&x["chart"]) {
            let tk = int(&c[0]);
            let (x_max, y_max) = chart_span(tk);
            let got = json!([tk, [x_max, y_max], chart_ylines(y_max), chart_xlines(x_max)]);
            assert_eq!(js(&got), js(c), "chart {tk}");
        }
        for c in arr(&x["blend"]) {
            let tf: f64 = c[0].as_str().unwrap().parse().unwrap();
            let (w, share) = world_blend(tf);
            assert_eq!(js(&json!([w, share])), js(&c[1]), "blend {tf}");
        }
        for c in arr(&x["world_of"]) {
            assert_eq!(world_of(int(&c[0])) as i64, int(&c[1]), "world_of {}", c[0]);
        }
        for c in arr(&x["step"]) {
            assert_eq!(step_target(int(&c[0]), int(&c[1])), int(&c[2]), "{c}");
        }
        for c in arr(&x["bucket"]) {
            assert_eq!(bucket_of(int(&c[0])) as i64, int(&c[1]), "bucket {}", c[0]);
        }
        for (i, c) in arr(&x["bucket_prob"]).iter().enumerate() {
            assert_eq!(bucket_prob(i), (int(&c[0]), int(&c[1])), "Fach {i}");
            assert_eq!(bucket_prob_f(i), int(&c[0]) as f64 / int(&c[1]) as f64);
        }
        // ---- Kleinigkeiten (Python-Regeln von Hand)
        let ticks = world_ticks();
        assert_eq!(ticks, WORLD_AT.map(tick_at));
        for (k, tv) in TARGETS.iter().enumerate() {
            let (num, den) = expected_return(*tv);
            let q = num as f64 / den as f64;
            assert!((0.96997..=0.97).contains(&q), "Ziel {tv}: {q}");
            assert!(target_ok(*tv));
            if k > 0 {
                assert_eq!(step_target(*tv, -1), TARGETS[k - 1]);
            }
        }
        assert!(!target_ok(100) && !target_ok(MAX_MULT100 + 1));
        assert!(target_ok_value(Some(&json!(150))) && !target_ok_value(Some(&json!(true))));
        assert!(!target_ok_value(Some(&json!(150.0))) && !target_ok_value(None));
        assert_eq!(clean_carry(Some(&json!(42))), 42);
        assert_eq!(clean_carry(Some(&json!(100))), 0);
        assert_eq!(clean_carry(Some(&json!(true))), 0);
        assert_eq!((half_stake(25), rest_stake(25)), (12, 13));
        assert!(!check_outcome(None, 200) && !check_outcome(Some(&json!({"x": 200, "b": []})), 200));
        assert!(!check_outcome(Some(&json!({"x": 200, "b": [[10, 0, 201, 0]]})), 200));
        assert!(!check_outcome(Some(&json!({"x": 200, "b": [[10, 0, 100, 0]]})), 200));
        assert!(!check_outcome(Some(&json!({"x": 200, "b": [[10, 0, true, 0]]})), 200));
        assert!(check_outcome(Some(&json!({"x": 200, "b": [[10, 150, 200, 25]]})), 200));
        let mut rnd = new_round(&f::Entry { n: 1, s: "0".repeat(32), c: "abc".into(), h: "0".repeat(64) }, &[(10, 0)]);
        assert_eq!(cash_out(&mut rnd, 0, false, 0), None, "unter MIN_CASH");
        assert_eq!(cash_out(&mut rnd, 5, false, 0), None, "kein Einsatz 5");
        assert_eq!(js(&rnd.entry().to_value()), js(&json!({"n": 1, "s": "0".repeat(32), "c": "abc", "h": "0".repeat(64)})));
    }
}
