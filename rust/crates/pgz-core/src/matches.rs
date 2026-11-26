//! matches.rs
//! ==========
//! Gegenstueck zu `matches.py` - die Match-Historie in einer eigenen Datei
//! neben `mem.json`:
//!
//! ```text
//! matches.pgzlamamatches
//! ```
//!
//! Aufbau (kompaktes JSON in einer Zeile, Reihenfolge wie Pythons `dict`):
//!
//! ```text
//! {"format":"pygamez.matches","v":1,"app":"PyGameZ","updated":"2026-11-26 14:03",
//!  "profiles":{"p1":{"crossy":{"n":57,"w":0,"l":0,"d":0,"best":212,"sum":3120,
//!   "time":4210,"first":"2026-11-20 18:02","list":[["2026-11-26 14:03","daily",57,
//!   null,41,4,{"day":329,"c":3,"ch":"chicken"}], ...]}}}}
//! ```
//!
//! Ein Eintrag ist `[zeit, modus, punkte, ergebnis, dauer_s, flags, info]`.
//! **Zeile fuer Zeile aus Python** - die Datei ist Byte fuer Byte dieselbe
//! (wie `replay.rs`: `serde_json` kompakt, Textmodus ueber
//! `store::write_text_atomic`). Gleitkommazahlen kommen nicht vor; die Dauer
//! ist in ganzen Sekunden abgerundet.

use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use serde_json::{Map, Value};

/// Kopfzeile der Datei.
pub const FORMAT: &str = "pygamez.matches";
/// Formatversion.
pub const VERSION: i64 = 1;
/// Dateiendung.
pub const EXT: &str = ".pgzlamamatches";
/// Dateiname neben `mem.json`.
pub const FILENAME: &str = "matches.pgzlamamatches";

/// Gespeicherte Matches je Spiel (Desktop).
pub const CAP: usize = 250;

/// Ergebnis: Sieg.
pub const WIN: i64 = 1;
/// Ergebnis: Niederlage.
pub const LOSS: i64 = 0;
/// Ergebnis: Remis.
pub const DRAW: i64 = 2;

/// Flag: neuer Highscore.
pub const F_RECORD: i64 = 1;
/// Flag: verlassen, bevor die Runde zu Ende war.
pub const F_ABORT: i64 = 2;
/// Flag: Tagesmodus (Tagesstrecke, Tagesraetsel, Tageswort).
pub const F_DAILY: i64 = 4;
/// Flag: vergangener Tag aus dem Tages-Archiv.
pub const F_ARCHIVE: i64 = 8;

/// Bekannte Info-Schluessel (Anzeige-Reihenfolge) -> Text-Schluessel.
pub const INFO_KEYS: &[(&str, &str)] = &[
    ("day", "match.info.day"),
    ("bet", "match.info.bet"),
    ("win", "match.info.win"),
    ("num", "match.info.num"),
    ("cr", "match.info.cr"),
    ("mi", "match.info.mi"),
    ("op", "match.info.op"),
    ("rw", "match.info.rw"),
    ("x", "match.info.x"),
    ("c", "match.info.c"),
    ("ch", "match.info.ch"),
    ("vr", "match.info.vr"),
    ("lv", "match.info.lv"),
    ("ln", "match.info.ln"),
    ("st", "match.info.st"),
    ("tm", "match.info.tm"),
    ("g", "match.info.g"),
    ("wd", "match.info.wd"),
    ("len", "match.info.len"),
    ("vs", "match.info.vs"),
];

/// Schreib-Drosselung in Sekunden.
const FLUSH_EVERY: f64 = 5.0;

/// Kurze Runden, die man verlaesst, ohne etwas erreicht zu haben, zaehlen nicht.
pub const MIN_ABORT_SECONDS: i64 = 5;

const MAX_INT: i64 = (1 << 53) - 1;

/// Ein gespeichertes Match (`[zeit, modus, punkte, ergebnis, dauer, flags, info]`).
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// `"YYYY-MM-DD HH:MM"`
    pub stamp: String,
    /// Modus des Spiels.
    pub mode: String,
    /// Punkte.
    pub score: i64,
    /// 1 Sieg, 0 Niederlage, 2 Remis, `None` unbekannt. Als JSON-Wert, weil
    /// Python aus einer Datei auch `1.0` uebernimmt (`1.0 in (1, 0, 2)`).
    pub result: Option<Value>,
    /// Dauer in ganzen Sekunden.
    pub dur: i64,
    /// Bitmaske `F_*`.
    pub flags: i64,
    /// Kleines dict mit Details oder `None`.
    pub info: Option<Map<String, Value>>,
}

impl Entry {
    /// Ergebnis als Zahl (`None` = unbekannt).
    pub fn result_i(&self) -> Option<i64> {
        self.result.as_ref().and_then(result_num)
    }

    fn to_value(&self) -> Value {
        Value::Array(vec![
            Value::String(self.stamp.clone()),
            Value::String(self.mode.clone()),
            Value::from(self.score),
            self.result.clone().unwrap_or(Value::Null),
            Value::from(self.dur),
            Value::from(self.flags),
            self.info.clone().map(Value::Object).unwrap_or(Value::Null),
        ])
    }
}

/// Historie eines Spiels (Summen + Liste).
#[derive(Debug, Clone, PartialEq)]
struct GameHist {
    n: i64,
    w: i64,
    l: i64,
    d: i64,
    best: Option<i64>,
    sum: i64,
    time: i64,
    first: String,
    list: Vec<Entry>,
}

/// Summen eines Spiels (`totals()`).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Totals {
    /// Matches insgesamt.
    pub n: i64,
    /// Siege.
    pub w: i64,
    /// Niederlagen.
    pub l: i64,
    /// Remis.
    pub d: i64,
    /// Bestwert (`None` = noch keiner).
    pub best: Option<i64>,
    /// Summe der Punkte.
    pub sum: i64,
    /// Spielzeit in Sekunden.
    pub time: i64,
    /// Zeitpunkt des ersten Matches.
    pub first: String,
    /// Gespeicherte Eintraege.
    pub stored: i64,
}

fn blank_game() -> GameHist {
    GameHist { n: 0, w: 0, l: 0, d: 0, best: None, sum: 0, time: 0, first: String::new(), list: Vec::new() }
}

impl GameHist {
    fn to_value(&self) -> Value {
        let mut m = Map::new();
        m.insert("n".into(), Value::from(self.n));
        m.insert("w".into(), Value::from(self.w));
        m.insert("l".into(), Value::from(self.l));
        m.insert("d".into(), Value::from(self.d));
        m.insert("best".into(), self.best.map(Value::from).unwrap_or(Value::Null));
        m.insert("sum".into(), Value::from(self.sum));
        m.insert("time".into(), Value::from(self.time));
        m.insert("first".into(), Value::String(self.first.clone()));
        m.insert("list".into(), Value::Array(self.list.iter().map(Entry::to_value).collect()));
        Value::Object(m)
    }
}

/// Profile in Datei-/Anlage-Reihenfolge, je Profil die Spiele ebenso (wie
/// Pythons `dict`).
type Profiles = Vec<(String, Vec<(String, GameHist)>)>;

struct State {
    doc: Option<Profiles>,
    dirty: bool,
    last_flush: Option<Instant>,
}

fn st() -> MutexGuard<'static, State> {
    static S: OnceLock<Mutex<State>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(State { doc: None, dirty: false, last_flush: None }))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

fn path() -> std::path::PathBuf {
    crate::store::path_for(FILENAME)
}

fn now_str() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()
}

// ---------------------------------------------------------------------------
//  Bereinigen
// ---------------------------------------------------------------------------

/// Ganzzahl aus JSON (`isinstance(v, int) and not isinstance(v, bool)`).
fn as_int(v: &Value) -> Option<i128> {
    match v {
        Value::Number(n) => n.as_i64().map(i128::from).or_else(|| n.as_u64().map(i128::from)),
        _ => None,
    }
}

fn clamp_i(v: i128, lo: i64, hi: i64) -> i64 {
    v.clamp(lo as i128, hi as i128) as i64
}

/// `_clamp_int(v, lo, hi, default=0)`
fn clamp_int(v: Option<&Value>, lo: i64, hi: i64) -> i64 {
    match v.and_then(as_int) {
        Some(i) => clamp_i(i, lo, hi),
        None => 0,
    }
}

/// `result in (WIN, LOSS, DRAW) and not isinstance(result, bool)` - als Zahl.
fn result_num(v: &Value) -> Option<i64> {
    match v {
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                return [0, 1, 2].contains(&i).then_some(i);
            }
            if n.as_u64().is_some() {
                return None;
            }
            let f = n.as_f64()?;
            [0.0, 1.0, 2.0].iter().position(|x| *x == f).map(|i| [0, 1, 2][i])
        }
        _ => None,
    }
}

/// Erste `n` Zeichen (Pythons `s[:n]`).
fn head(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

/// `_clean_info(raw)` - nur flache Werte (int, str bis 40 Zeichen, bool).
pub fn clean_info(raw: Option<&Value>) -> Option<Map<String, Value>> {
    let Some(Value::Object(raw)) = raw else { return None };
    let mut out = Map::new();
    for (key, value) in raw {
        if key.chars().count() > 8 {
            continue;
        }
        match value {
            Value::Bool(b) => {
                out.insert(key.clone(), Value::Bool(*b));
            }
            Value::String(s) => {
                out.insert(key.clone(), Value::String(head(s, 40)));
            }
            v => {
                if let Some(i) = as_int(v) {
                    out.insert(key.clone(), Value::from(clamp_i(i, -MAX_INT, MAX_INT)));
                }
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// `_clean_entry(raw)`
fn clean_entry(raw: &Value) -> Option<Entry> {
    let Value::Array(items) = raw else { return None };
    if items.len() < 6 {
        return None;
    }
    let Value::String(stamp) = &items[0] else { return None };
    let mode = match &items[1] {
        Value::String(m) => head(m, 24),
        _ => String::new(),
    };
    let result = result_num(&items[3]).map(|_| items[3].clone());
    Some(Entry {
        stamp: head(stamp, 16),
        mode,
        score: clamp_int(Some(&items[2]), -MAX_INT, MAX_INT),
        result,
        dur: clamp_int(Some(&items[4]), 0, MAX_INT),
        flags: clamp_int(Some(&items[5]), 0, 255),
        info: clean_info(items.get(6)),
    })
}

/// `_clean_game(raw)`
fn clean_game(raw: &Value) -> GameHist {
    let mut game = blank_game();
    let Value::Object(raw) = raw else { return game };
    let get = |k: &str| raw.get(k);
    game.n = clamp_int(get("n"), 0, MAX_INT);
    game.w = clamp_int(get("w"), 0, MAX_INT);
    game.l = clamp_int(get("l"), 0, MAX_INT);
    game.d = clamp_int(get("d"), 0, MAX_INT);
    game.best = get("best").and_then(as_int).map(|i| clamp_i(i, -MAX_INT, MAX_INT));
    game.sum = clamp_int(get("sum"), 0, MAX_INT);
    game.time = clamp_int(get("time"), 0, MAX_INT);
    game.first = match get("first") {
        Some(Value::String(s)) => head(s, 16),
        _ => String::new(),
    };
    if let Some(Value::Array(items)) = get("list") {
        for item in items {
            if let Some(e) = clean_entry(item) {
                game.list.push(e);
            }
        }
        if game.list.len() > CAP {
            let weg = game.list.len() - CAP;
            game.list.drain(..weg);
        }
    }
    game
}

fn read_file(path: &std::path::Path) -> Option<Map<String, Value>> {
    let text = std::fs::read_to_string(path).ok()?;
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(m)) => Some(m),
        _ => None,
    }
}

/// `_load()` - die Datei einmalig lesen (fehlt sie, ist die Historie leer).
fn load(s: &mut State) -> &mut Profiles {
    if s.doc.is_none() {
        let p = path();
        let raw = read_file(&p).or_else(|| {
            let mut bak = p.as_os_str().to_os_string();
            bak.push(".bak");
            read_file(std::path::Path::new(&bak))
        });
        let mut doc: Profiles = Vec::new();
        if let Some(raw) = raw {
            if raw.get("format").and_then(|v| v.as_str()) == Some(FORMAT) {
                if let Some(Value::Object(profs)) = raw.get("profiles") {
                    for (pid, prof) in profs {
                        let Value::Object(prof) = prof else { continue };
                        let games: Vec<(String, GameHist)> =
                            prof.iter().map(|(k, v)| (k.clone(), clean_game(v))).collect();
                        // doc["profiles"][str(pid)] = ... (doppelte ids gibt es in JSON nicht)
                        doc.push((pid.clone(), games));
                    }
                }
            }
        }
        s.doc = Some(doc);
    }
    s.doc.as_mut().expect("eben geladen")
}

/// `_profile(pid)` - legt das Profil im Speicher an, falls es fehlt.
fn profile<'a>(s: &'a mut State, pid: Option<&str>) -> &'a mut Vec<(String, GameHist)> {
    let pid = match pid {
        Some(p) if !p.is_empty() => p.to_string(),
        _ => crate::store::active_id(),
    };
    let doc = load(s);
    let i = match doc.iter().position(|(p, _)| *p == pid) {
        Some(i) => i,
        None => {
            doc.push((pid, Vec::new()));
            doc.len() - 1
        }
    };
    &mut doc[i].1
}

/// `_game(key, pid, create)`
fn game<'a>(s: &'a mut State, key: &str, pid: Option<&str>, create: bool) -> Option<&'a mut GameHist> {
    let prof = profile(s, pid);
    let i = match prof.iter().position(|(k, _)| k == key) {
        Some(i) => i,
        None => {
            if !create {
                return None;
            }
            prof.push((key.to_string(), blank_game()));
            prof.len() - 1
        }
    };
    Some(&mut prof[i].1)
}

// ---------------------------------------------------------------------------
//  Schreiben
// ---------------------------------------------------------------------------

fn flush_locked(s: &mut State) {
    if !s.dirty || s.doc.is_none() {
        return;
    }
    let mut profiles = Map::new();
    for (pid, prof) in s.doc.as_ref().expect("geladen") {
        let mut games = Map::new();
        for (key, game) in prof {
            if game.n != 0 {
                games.insert(key.clone(), game.to_value());
            }
        }
        if !games.is_empty() {
            profiles.insert(pid.clone(), Value::Object(games));
        }
    }
    let mut out = Map::new();
    out.insert("format".into(), Value::from(FORMAT));
    out.insert("v".into(), Value::from(VERSION));
    out.insert("app".into(), Value::from("PyGameZ"));
    out.insert("updated".into(), Value::from(now_str()));
    out.insert("profiles".into(), Value::Object(profiles));
    if let Ok(text) = serde_json::to_string(&Value::Object(out)) {
        crate::store::write_text_atomic(&path(), &text);
    }
    s.dirty = false;
    s.last_flush = Some(Instant::now());
}

/// `flush()` - ungespeicherte Matches sofort schreiben.
pub fn flush() {
    let mut s = st();
    flush_locked(&mut s);
}

/// `maybe_flush()` - gedrosselt schreiben (hoechstens alle 5 Sekunden; das
/// erste Mal sofort, wie Python mit `_last_flush = 0.0`).
pub fn maybe_flush() {
    let mut s = st();
    let faellig = s.last_flush.map(|t| t.elapsed().as_secs_f64() >= FLUSH_EVERY).unwrap_or(true);
    if s.dirty && faellig {
        flush_locked(&mut s);
    }
}

/// `reset_cache()` - Profilwechsel: Offenes wegschreiben.
pub fn reset_cache() {
    flush();
}

/// Rueckruf fuer den Profilwechsel anmelden (`store.on_switch(reset_cache)`).
pub fn register_switch_hook() {
    crate::store::on_switch(reset_cache);
}

/// Pruefhilfe (Audit): Speicher vergessen, als waere das Programm neu gestartet.
pub fn forget() {
    let mut s = st();
    s.doc = None;
    s.dirty = false;
}

// ---------------------------------------------------------------------------
//  Aufzeichnen
// ---------------------------------------------------------------------------

/// `record(key, mode, score, result, duration, flags, info, pid, replace)` -
/// haelt ein Match fest (neueste zuletzt) und gibt den Eintrag zurueck.
#[allow(clippy::too_many_arguments)]
pub fn record(
    key: &str,
    mode: &str,
    score: i64,
    result: Option<i64>,
    duration: i64,
    flags: i64,
    info: Option<&Map<String, Value>>,
    pid: Option<&str>,
    replace: bool,
) -> Entry {
    let mut s = st();
    let stamp = now_str();
    let raw = Value::Array(vec![
        Value::String(stamp),
        Value::String(mode.to_string()),
        Value::from(score),
        result.map(Value::from).unwrap_or(Value::Null),
        Value::from(duration.max(0)),
        Value::from(flags),
        info.cloned().map(Value::Object).unwrap_or(Value::Null),
    ]);
    let mut entry = clean_entry(&raw).expect("gueltiger Eintrag");
    let game = game(&mut s, key, pid, true).expect("angelegt");
    if replace && !game.list.is_empty() {
        // Dieselbe Partie lief weiter: altes Match raus, Spielzeit addieren.
        let old = game.list.pop().expect("nicht leer");
        sub_totals(game, &old);
        entry.dur = clamp_i(entry.dur as i128 + old.dur as i128, 0, MAX_INT);
    }
    add_totals(game, &entry);
    game.list.push(entry.clone());
    if game.list.len() > CAP {
        let weg = game.list.len() - CAP;
        game.list.drain(..weg);
    }
    s.dirty = true;
    entry
}

fn add_totals(game: &mut GameHist, entry: &Entry) {
    game.n = MAX_INT.min(game.n + 1);
    match entry.result_i() {
        Some(WIN) => game.w += 1,
        Some(LOSS) => game.l += 1,
        Some(DRAW) => game.d += 1,
        _ => {}
    }
    if game.best.is_none_or(|b| entry.score > b) {
        game.best = Some(entry.score);
    }
    game.sum = clamp_i(game.sum as i128 + entry.score as i128, -MAX_INT, MAX_INT);
    game.time = clamp_i(game.time as i128 + entry.dur as i128, 0, MAX_INT);
    if game.first.is_empty() {
        game.first = entry.stamp.clone();
    }
}

fn sub_totals(game: &mut GameHist, entry: &Entry) {
    game.n = 0.max(game.n - 1);
    match entry.result_i() {
        Some(WIN) => game.w = 0.max(game.w - 1),
        Some(LOSS) => game.l = 0.max(game.l - 1),
        Some(DRAW) => game.d = 0.max(game.d - 1),
        _ => {}
    }
    game.sum = clamp_i(game.sum as i128 - entry.score as i128, -MAX_INT, MAX_INT);
    game.time = 0.max(game.time - entry.dur);
}

// ---------------------------------------------------------------------------
//  Abfragen (fuer den Matches-Screen)
// ---------------------------------------------------------------------------

/// `entries(key, pid)` - alle gespeicherten Matches, NEUESTE ZUERST.
pub fn entries(key: &str, pid: Option<&str>) -> Vec<Entry> {
    let mut s = st();
    match game(&mut s, key, pid, false) {
        Some(g) => g.list.iter().rev().cloned().collect(),
        None => Vec::new(),
    }
}

/// `recent(key, n, pid)` - die letzten n Matches, neueste zuerst.
pub fn recent(key: &str, n: usize, pid: Option<&str>) -> Vec<Entry> {
    entries(key, pid).into_iter().take(n).collect()
}

/// `totals(key, pid)` - Summen eines Spiels.
pub fn totals(key: &str, pid: Option<&str>) -> Totals {
    let mut s = st();
    let mut out = Totals::default();
    if let Some(g) = game(&mut s, key, pid, false) {
        out = Totals {
            n: g.n,
            w: g.w,
            l: g.l,
            d: g.d,
            best: g.best,
            sum: g.sum,
            time: g.time,
            first: g.first.clone(),
            stored: g.list.len() as i64,
        };
    }
    out
}

/// `modes_of(key, pid)` - Modi der gespeicherten Matches (erste Sichtung zuerst).
pub fn modes_of(key: &str, pid: Option<&str>) -> Vec<String> {
    let mut s = st();
    let mut seen: Vec<String> = Vec::new();
    if let Some(g) = game(&mut s, key, pid, false) {
        for e in &g.list {
            if !seen.contains(&e.mode) {
                seen.push(e.mode.clone());
            }
        }
    }
    seen
}

/// `drop_profile(pid)` - Historie eines geloeschten Profils entfernen.
pub fn drop_profile(pid: &str) {
    let mut s = st();
    let doc = load(&mut s);
    if let Some(i) = doc.iter().position(|(p, _)| p == pid) {
        doc.remove(i);
        s.dirty = true;
        flush_locked(&mut s);
    }
}

/// `clear_game(key, pid)` - Historie eines Spiels leeren.
pub fn clear_game(key: &str, pid: Option<&str>) {
    let mut s = st();
    let prof = profile(&mut s, pid);
    if let Some(i) = prof.iter().position(|(k, _)| k == key) {
        prof.remove(i);
        s.dirty = true;
        flush_locked(&mut s);
    }
}

/// `format_duration(seconds)` - `'1:05:09'` / `'3:07'` / `'0:42'`.
pub fn format_duration(seconds: i64) -> String {
    let seconds = seconds.max(0);
    let (h, rest) = (seconds / 3600, seconds % 3600);
    let (m, sec) = (rest / 60, rest % 60);
    if h != 0 {
        format!("{h}:{m:02}:{sec:02}")
    } else {
        format!("{m}:{sec:02}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dauer_wie_python() {
        assert_eq!(format_duration(3725), "1:02:05");
        assert_eq!(format_duration(65), "1:05");
        assert_eq!(format_duration(0), "0:00");
    }

    #[test]
    fn eintrag_bereinigt() {
        let raw: Value = serde_json::from_str(r#"["2026-01-01 10:00", 3, "viel", 7, -5, 999, [1]]"#).unwrap();
        let e = clean_entry(&raw).unwrap();
        assert_eq!(e.to_value(), serde_json::from_str::<Value>(r#"["2026-01-01 10:00","",0,null,0,255,null]"#).unwrap());
        let info: Value = serde_json::from_str(r#"{"c": 2, "ch": "fox", "x": 1.5, "zu_langer_schluessel": 1, "ok": true}"#).unwrap();
        assert_eq!(Value::Object(clean_info(Some(&info)).unwrap()), serde_json::from_str::<Value>(r#"{"c":2,"ch":"fox","ok":true}"#).unwrap());
    }
}
