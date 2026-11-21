//! lamabank.rs
//! ===========
//! Die **Lama-Bank** - das gemeinsame Chip-Konto von Blackjack, Poker,
//! Casino, Lama Crossing, Lama Mines und Lama Plinko (aus `lamabank.py`).
//!
//! Gespeichert im Abschnitt `casino` der `mem.json` (je Profil):
//! `chips`, `ledger` (Netto-Bilanz je Spiel), `peak` (Hoechststand je Spiel),
//! `escrow` (Poker-Tischstapel), `refills` (Bank-Kredite), `migrated`,
//! `extra` (kleine Spielstaende). Regeln wie in Python: Einsatz sofort
//! abbuchen und speichern, je Spiel eine eigene Bilanz, Pleite = unter dem
//! Mindesteinsatz -> Konto zurueck auf `START_CHIPS`.

use std::sync::Mutex;

use serde_json::{json, Map, Value};

use crate::store;

/// Abschnitt in der `mem.json`.
pub const SECTION: &str = "casino";
/// Startkapital (und Wert nach einem Bank-Kredit).
pub const START_CHIPS: i64 = 1000;
/// Die Spiele mit eigener Bilanz.
pub const GAMES: [&str; 6] = ["blackjack", "poker", "casino", "lamacrossing", "lamamines", "lamaplinko"];

/// Mindesteinsatz je Spiel bzw. `"spiel:modus"` (Pleite-Regel).
/// Blackjack: kleinster Chip 10 - Poker: Big Blind 20 (Hold'em und Draw),
/// Video Poker 10 - Roulette: 1er-Chip - Lama-Slot: 10 Linien x 1 -
/// Lama Crossing und Lama Mines: kleinste Einsatzstufe 10 -
/// Lama Plinko: kleinster Einsatz je Kugel 1.
const MIN_BET: [(&str, i64); 8] = [
    ("blackjack", 10),
    ("poker", 20),
    ("poker:video", 10),
    ("casino", 1),
    ("casino:slots", 10),
    ("lamacrossing", 10),
    ("lamamines", 10),
    ("lamaplinko", 1),
];

static DATA: Mutex<Option<Map<String, Value>>> = Mutex::new(None);

fn lock() -> std::sync::MutexGuard<'static, Option<Map<String, Value>>> {
    DATA.lock().unwrap_or_else(|e| e.into_inner())
}

/// `_int(value, default, lo)` - bool zaehlt nicht als Zahl.
fn int_of(value: Option<&Value>, default: Option<i64>, lo: Option<i64>) -> Option<i64> {
    let v = match value {
        Some(Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                i
            } else if let Some(f) = n.as_f64() {
                f as i64
            } else {
                return default;
            }
        }
        _ => return default,
    };
    Some(match lo {
        Some(l) => v.max(l),
        None => v,
    })
}

fn blank() -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("chips".into(), json!(START_CHIPS));
    m.insert("ledger".into(), Value::Object(GAMES.iter().map(|g| (g.to_string(), json!(0))).collect()));
    m.insert("peak".into(), Value::Object(GAMES.iter().map(|g| (g.to_string(), json!(START_CHIPS))).collect()));
    m.insert("escrow".into(), json!(0));
    m.insert("refills".into(), json!(0));
    m.insert("migrated".into(), json!(false));
    m.insert("extra".into(), json!({}));
    m
}

fn clean(raw: &Map<String, Value>) -> Map<String, Value> {
    let mut out = blank();
    out.insert("chips".into(), json!(int_of(raw.get("chips"), Some(START_CHIPS), Some(0))));
    for (field, default) in [("ledger", 0), ("peak", START_CHIPS)] {
        let src = raw.get(field).and_then(|v| v.as_object()).cloned().unwrap_or_default();
        let mut m = Map::new();
        for g in GAMES {
            m.insert(g.into(), json!(int_of(src.get(g), Some(default), None)));
        }
        out.insert(field.into(), Value::Object(m));
    }
    out.insert("escrow".into(), json!(int_of(raw.get("escrow"), Some(0), Some(0))));
    out.insert("refills".into(), json!(int_of(raw.get("refills"), Some(0), Some(0))));
    out.insert("migrated".into(), json!(raw.get("migrated") == Some(&Value::Bool(true))));
    if let Some(Value::Object(extra)) = raw.get("extra") {
        out.insert("extra".into(), Value::Object(extra.clone()));
    }
    out
}

fn migrate(hs: &Map<String, Value>, sections: &[(&str, Map<String, Value>)]) -> Map<String, Value> {
    let mut data = blank();
    let mut chips = Vec::new();
    for (game, old) in sections {
        if let Some(c) = int_of(old.get("chips"), None, Some(0)) {
            chips.push(c);
        }
        let best = int_of(old.get("best"), Some(0), None).unwrap_or(0).max(int_of(hs.get(*game), Some(0), None).unwrap_or(0));
        data["peak"][*game] = json!(START_CHIPS.max(best));
    }
    data["peak"]["casino"] = json!(START_CHIPS.max(int_of(hs.get("casino"), Some(0), None).unwrap_or(0)));
    data.insert("chips".into(), json!(chips.iter().copied().max().unwrap_or(START_CHIPS)));
    data.insert("migrated".into(), json!(true));
    data
}

fn save(d: &Map<String, Value>) {
    store::save_section(SECTION, d.clone());
}

/// Vergisst das geladene Konto (nach einem Profilwechsel).
pub fn reset_cache() {
    *lock() = None;
}

/// Meldet `reset_cache` beim Profilwechsel an (`store.on_switch`).
pub fn register_switch_hook() {
    store::on_switch(reset_cache);
}

fn load_into(slot: &mut Option<Map<String, Value>>) {
    let raw = store::load_section(SECTION);
    let mut changed;
    let mut data = if raw.get("migrated") == Some(&Value::Bool(true)) {
        let d = clean(&raw);
        changed = d != raw;
        d
    } else {
        let sections = [("blackjack", store::load_section("blackjack")), ("poker", store::load_section("poker"))];
        changed = true;
        migrate(&store::load_section("highscores"), &sections)
    };
    let escrow = data["escrow"].as_i64().unwrap_or(0);
    if escrow > 0 {
        let chips = data["chips"].as_i64().unwrap_or(0);
        data["chips"] = json!(chips + escrow);
        data["escrow"] = json!(0);
        changed = true;
    }
    if changed {
        save(&data);
    }
    *slot = Some(data);
}

/// Liest das Konto frisch (Migration + escrow-Erstattung). Beim Start eines
/// Chip-Spiels aufrufen.
pub fn load() {
    let mut g = lock();
    load_into(&mut g);
}

/// Arbeitet auf dem geladenen Stand (laedt bei Bedarf).
fn with<R>(f: impl FnOnce(&mut Map<String, Value>) -> R) -> R {
    let mut g = lock();
    if g.is_none() {
        load_into(&mut g);
    }
    f(g.as_mut().expect("lamabank: loaded"))
}

fn geti(d: &Map<String, Value>, key: &str) -> i64 {
    d.get(key).and_then(|v| v.as_i64()).unwrap_or(0)
}

fn get_in(d: &Map<String, Value>, field: &str, game: &str, default: i64) -> i64 {
    d.get(field).and_then(|m| m.get(game)).and_then(|v| v.as_i64()).unwrap_or(default)
}

/// Aktueller Kontostand (ohne escrow).
pub fn balance() -> i64 {
    with(|d| geti(d, "chips"))
}

/// Chips, die gerade als Poker-Tischstapel unterwegs sind.
pub fn escrow() -> i64 {
    with(|d| geti(d, "escrow"))
}

/// Netto-Bilanz eines Spiels.
pub fn ledger(game: &str) -> i64 {
    with(|d| get_in(d, "ledger", game, 0))
}

/// `START_CHIPS + Bilanz` (fuer Erfolge).
pub fn value_for(game: &str) -> i64 {
    START_CHIPS + ledger(game)
}

/// Highscore eines Spiels: Hoechststand von `START_CHIPS + Bilanz`.
pub fn score_for(game: &str) -> i64 {
    let peak = with(|d| get_in(d, "peak", game, START_CHIPS));
    peak.max(value_for(game))
}

/// Anzahl der bisher genommenen Bank-Kredite.
pub fn refills() -> i64 {
    with(|d| geti(d, "refills"))
}

/// Mindesteinsatz eines Spiels (optional je Modus).
pub fn min_bet(game: &str, mode: Option<&str>) -> i64 {
    if let Some(m) = mode {
        let k = format!("{game}:{m}");
        if let Some((_, v)) = MIN_BET.iter().find(|(n, _)| *n == k) {
            return *v;
        }
    }
    MIN_BET.iter().find(|(n, _)| *n == game).map(|(_, v)| *v).unwrap_or(1)
}

/// True, wenn das Konto unter dem Mindesteinsatz liegt.
pub fn is_broke(game: &str, mode: Option<&str>) -> bool {
    balance() < min_bet(game, mode)
}

/// Kleiner Zusatz-Spielstand.
pub fn get_extra(key: &str) -> Option<Value> {
    with(|d| d.get("extra").and_then(|e| e.get(key)).cloned())
}

/// Bucht einen Einsatz ab und speichert sofort. `false` = nicht gedeckt.
pub fn debit(n: i64, game: &str) -> bool {
    if n <= 0 {
        return true;
    }
    with(|d| {
        let chips = geti(d, "chips");
        if n > chips {
            return false;
        }
        d["chips"] = json!(chips - n);
        let l = get_in(d, "ledger", game, 0);
        d["ledger"][game] = json!(l - n);
        save(d);
        true
    })
}

/// Schreibt einen Gewinn gut (Bilanz und Hoechststand).
pub fn credit(n: i64, game: &str, save_now: bool) {
    if n <= 0 {
        return;
    }
    with(|d| {
        d["chips"] = json!(geti(d, "chips") + n);
        let l = get_in(d, "ledger", game, 0) + n;
        d["ledger"][game] = json!(l);
        let peak = get_in(d, "peak", game, START_CHIPS).max(START_CHIPS + l);
        d["peak"][game] = json!(peak);
        if save_now {
            save(d);
        }
    })
}

/// `bet_and_pay(bet, win, game, extra_key, extra)` - Einsatz abbuchen UND
/// Gewinn gutschreiben, mit EINEM Speichern.
///
/// Fuer Spiele, deren Ergebnis schon beim Einsatz feststeht (Lama Plinko:
/// der Weg der Kugel wird beim Abwurf gezogen, gezeigt erst bei der
/// Landung). `extra = Some((schluessel, wert))` schreibt im selben Zug einen
/// Zusatz-Spielstand wie `set_extra` (Python `extra_key`/`extra`; ein
/// `None`-Wert dort ist hier `Value::Null`). `false` = Einsatz nicht
/// gedeckt - dann wird gar nichts gebucht (auch kein Zusatz).
///
/// Reihenfolge der Schritte wie in Python, damit `ledger`/`peak` gleich
/// aussehen: erst Einsatz (nur wenn > 0), dann Gewinn (nur wenn > 0) samt
/// Hoechststand; ein bisher fehlender Spiel-Schluessel landet hinten.
pub fn bet_and_pay(bet: i64, win: i64, game: &str, extra: Option<(&str, Value)>) -> bool {
    with(|d| {
        let chips = geti(d, "chips");
        if bet > chips {
            return false;
        }
        if bet > 0 {
            d["chips"] = json!(geti(d, "chips") - bet);
            let l = get_in(d, "ledger", game, 0);
            d["ledger"][game] = json!(l - bet);
        }
        if win > 0 {
            d["chips"] = json!(geti(d, "chips") + win);
            let l = get_in(d, "ledger", game, 0) + win;
            d["ledger"][game] = json!(l);
            let peak = get_in(d, "peak", game, START_CHIPS).max(START_CHIPS + l);
            d["peak"][game] = json!(peak);
        }
        if let Some((key, value)) = extra {
            if let Some(Value::Object(e)) = d.get_mut("extra") {
                e.insert(key.into(), value);
            }
        }
        save(d);
        true
    })
}

/// Bank-Kredit bei Pleite: Konto zurueck auf `START_CHIPS`.
pub fn refill_if_broke(game: &str, mode: Option<&str>) -> bool {
    let mb = min_bet(game, mode);
    with(|d| {
        if geti(d, "chips") >= mb {
            return false;
        }
        d["chips"] = json!(START_CHIPS);
        d["refills"] = json!(geti(d, "refills") + 1);
        save(d);
        true
    })
}

/// Legt n Chips als Tischstapel ins escrow.
pub fn set_escrow(n: i64) -> i64 {
    with(|d| {
        let chips = geti(d, "chips");
        let n = 0.max(n.min(chips));
        d["chips"] = json!(chips - n);
        d["escrow"] = json!(geti(d, "escrow") + n);
        save(d);
        n
    })
}

/// Chips aus dem Tischstapel in den Pot (gilt als Einsatz).
pub fn pay_from_escrow(n: i64, game: &str) -> i64 {
    with(|d| {
        let esc = geti(d, "escrow");
        let n = 0.max(n.min(esc));
        if n <= 0 {
            return 0;
        }
        d["escrow"] = json!(esc - n);
        let l = get_in(d, "ledger", game, 0);
        d["ledger"][game] = json!(l - n);
        save(d);
        n
    })
}

/// Tischstapel zurueck aufs Konto.
pub fn clear_escrow(save_now: bool) -> i64 {
    with(|d| {
        let esc = geti(d, "escrow");
        if esc <= 0 {
            return 0;
        }
        d["chips"] = json!(geti(d, "chips") + esc);
        d["escrow"] = json!(0);
        if save_now {
            save(d);
        }
        esc
    })
}

/// Hand-Ende: Tischstapel zurueck + Pot-Gewinn gutschreiben (ein Speichern).
pub fn settle_escrow(win: i64, game: &str) {
    clear_escrow(false);
    if win > 0 {
        credit(win, game, false);
    }
    with(|d| save(d));
}

/// Speichert einen kleinen Zusatz-Spielstand.
pub fn set_extra(key: &str, value: Value) {
    with(|d| {
        if let Some(Value::Object(e)) = d.get_mut("extra") {
            e.insert(key.into(), value);
        }
        save(d);
    })
}
