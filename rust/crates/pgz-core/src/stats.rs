//! stats.rs
//! ========
//! Gegenstueck zu `stats.py` - die Statistik-Datei `stats.pgzlamastats`.
//!
//! **Zeile fuer Zeile aus Python, auf demselben JSON-Dokument.** Die alte
//! Fassung hielt eigene Strukturen und wich an mehreren Stellen ab (Duelle
//! ohne Gegner, `distinct` ohne Spielzeit, Tage ohne Pruefung, UTC statt
//! Ortszeit). Jetzt liegt die Datei im Speicher als `serde_json::Value` -
//! wie Pythons `dict` -, und jede Funktion tut, was ihr Python-Gegenstueck
//! tut. Damit ist die Datei zwischen den Fassungen austauschbar, und die
//! Erfolge sehen dieselben Zahlen.
//!
//! Aufbau der Datei siehe Kopf von `stats.py`.

use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Instant;

use serde_json::{json, Map, Value};

/// Kopfzeile der Datei.
pub const FORMAT: &str = "pygamez.stats";
/// Formatversion.
pub const VERSION: i64 = 1;
/// Dateiendung.
pub const EXT: &str = ".pgzlamastats";
/// Dateiname neben `mem.json`.
pub const FILENAME: &str = "stats.pgzlamastats";
/// Hoechstens so oft (Sekunden) wird gedrosselt geschrieben.
pub const FLUSH_EVERY: f64 = 20.0;
/// Ereignis-Verlauf je Profil.
pub const MAX_EVENTS: usize = 4000;
/// Punktwerte je Spiel.
pub const MAX_SERIES: usize = 250;
/// Kalendertage.
pub const MAX_DAYS: usize = 1100;

#[derive(Clone, Copy)]
enum Vorgabe {
    I(i64),
    F(f64),
    S,
}

/// Felder eines Spiel-Eintrags mit Standardwerten (Reihenfolge = JSON).
const FIELDS: &[(&str, Vorgabe)] = &[
    ("plays", Vorgabe::I(0)),
    ("time", Vorgabe::F(0.0)),
    ("wins", Vorgabe::I(0)),
    ("losses", Vorgabe::I(0)),
    ("records", Vorgabe::I(0)),
    ("best", Vorgabe::I(0)),
    ("best_at", Vorgabe::S),
    ("first", Vorgabe::S),
    ("last", Vorgabe::S),
    ("longest", Vorgabe::F(0.0)),
];

impl Vorgabe {
    fn value(self) -> Value {
        match self {
            Vorgabe::I(i) => json!(i),
            Vorgabe::F(f) => json!(f),
            Vorgabe::S => json!(""),
        }
    }
}

struct State {
    doc: Option<Value>,
    dirty: bool,
    last_flush: Option<Instant>,
    sess_key: Option<String>,
    sess_time: f64,
}

fn st() -> MutexGuard<'static, State> {
    static S: OnceLock<Mutex<State>> = OnceLock::new();
    S.get_or_init(|| {
        Mutex::new(State { doc: None, dirty: false, last_flush: None, sess_key: None, sess_time: 0.0 })
    })
    .lock()
    .unwrap_or_else(|e| e.into_inner())
}

fn path() -> std::path::PathBuf {
    crate::store::path_for(FILENAME)
}

fn now_str() -> String {
    let (y, m, d, h, mi, _) = crate::store::civil_from_unix(crate::store::local_secs());
    format!("{y:04}-{m:02}-{d:02} {h:02}:{mi:02}")
}

fn today_str() -> String {
    crate::store::today()
}

fn now_hour() -> usize {
    let (_, _, _, h, _, _) = crate::store::civil_from_unix(crate::store::local_secs());
    (h as usize).min(23)
}

/// `round(x, 1)` - Python rundet die exakte Binaerzahl, `{:.1}` auch.
fn round1(x: f64) -> f64 {
    format!("{x:.1}").parse().unwrap_or(x)
}

// ---------------------------------------------------------------------------
//  Leere Eintraege und Bereinigen
// ---------------------------------------------------------------------------

fn blank_game() -> Value {
    let mut m = Map::new();
    for (f, d) in FIELDS {
        m.insert((*f).into(), d.value());
    }
    Value::Object(m)
}

fn blank_profile() -> Value {
    json!({"games": {}, "days": {}, "hours": vec![0.0f64; 24], "series": {},
           "events": [], "duel": {},
           "streak": {"cur": 0, "best": 0, "first_day": "", "last_day": ""},
           "points": 0, "level": 1, "longest": 0.0})
}

fn blank_doc() -> Value {
    json!({"format": FORMAT, "v": VERSION, "app": "PyGameZ", "updated": now_str(), "profiles": {}})
}

/// `_num(value, default, lo)` - Zahl oder Standard, nach unten begrenzt.
fn num(v: Option<&Value>, default: f64, lo: f64) -> f64 {
    match v {
        Some(Value::Number(n)) => n.as_f64().unwrap_or(default).max(lo),
        _ => default,
    }
}

fn as_f(v: Option<&Value>) -> f64 {
    v.and_then(Value::as_f64).unwrap_or(0.0)
}

fn as_i(v: Option<&Value>) -> i64 {
    match v {
        Some(Value::Number(n)) => n.as_i64().unwrap_or_else(|| n.as_f64().unwrap_or(0.0) as i64),
        _ => 0,
    }
}

fn as_s(v: Option<&Value>) -> String {
    match v {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

fn kurz(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

fn truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

fn clean_game(raw: Option<&Value>) -> Value {
    let mut entry = blank_game();
    let Some(Value::Object(raw)) = raw else { return entry };
    let m = entry.as_object_mut().expect("blank_game ist ein Objekt");
    for (f, d) in FIELDS {
        let value = raw.get(*f);
        let neu = match d {
            Vorgabe::I(def) => match value {
                Some(Value::Number(n)) => json!(n.as_i64().unwrap_or_else(|| n.as_f64().unwrap_or(0.0) as i64).max(0)),
                Some(Value::Bool(b)) => json!(i64::from(*b)),
                Some(Value::String(s)) => json!(s.trim().parse::<i64>().unwrap_or(*def).max(0)),
                _ => json!(def),
            },
            Vorgabe::F(def) => match value {
                Some(Value::Number(n)) => json!(n.as_f64().unwrap_or(*def).max(0.0)),
                Some(Value::Bool(b)) => json!(if *b { 1.0 } else { 0.0 }),
                Some(Value::String(s)) => json!(s.trim().parse::<f64>().unwrap_or(*def).max(0.0)),
                _ => json!(def),
            },
            Vorgabe::S => match value {
                None => json!(""),
                Some(Value::String(s)) => json!(s),
                Some(Value::Null) => json!("None"),
                Some(other) => json!(other.to_string()),
            },
        };
        m.insert((*f).into(), neu);
    }
    entry
}

fn clean_profile(raw: &Value) -> Value {
    let mut prof = blank_profile();
    let Value::Object(raw) = raw else { return prof };
    let p = prof.as_object_mut().expect("Profil ist ein Objekt");
    if let Some(Value::Object(games)) = raw.get("games") {
        let g = p["games"].as_object_mut().expect("games");
        for (key, entry) in games {
            g.insert(key.clone(), clean_game(Some(entry)));
        }
    }
    if let Some(Value::Object(days)) = raw.get("days") {
        let dmap = p["days"].as_object_mut().expect("days");
        for (day, entry) in days {
            let Value::Object(e) = entry else { continue };
            let mut clean = json!({"t": num(e.get("t"), 0.0, 0.0), "p": num(e.get("p"), 0.0, 0.0) as i64,
                                   "w": num(e.get("w"), 0.0, 0.0) as i64, "a": num(e.get("a"), 0.0, 0.0) as i64,
                                   "g": {}});
            if let Some(Value::Object(per)) = e.get("g") {
                let g = clean["g"].as_object_mut().expect("g");
                for (k, v) in per {
                    g.insert(k.clone(), json!(num(Some(v), 0.0, 0.0)));
                }
            }
            dmap.insert(kurz(day, 10), clean);
        }
    }
    if let Some(Value::Array(hours)) = raw.get("hours") {
        let h = p["hours"].as_array_mut().expect("hours");
        for (i, v) in hours.iter().take(24).enumerate() {
            h[i] = json!(num(Some(v), 0.0, 0.0));
        }
    }
    if let Some(Value::Object(series)) = raw.get("series") {
        let s = p["series"].as_object_mut().expect("series");
        for (key, points) in series {
            let Value::Array(points) = points else { continue };
            let mut clean = Vec::new();
            for item in points {
                if let Value::Array(it) = item {
                    if it.len() >= 2 {
                        if let Value::String(stamp) = &it[0] {
                            let rec = it.len() > 2 && truthy(&it[2]);
                            clean.push(json!([kurz(stamp, 16), num(Some(&it[1]), 0.0, 0.0) as i64, i64::from(rec)]));
                        }
                    }
                }
            }
            if !clean.is_empty() {
                let start = clean.len().saturating_sub(MAX_SERIES);
                s.insert(key.clone(), Value::Array(clean[start..].to_vec()));
            }
        }
    }
    if let Some(Value::Array(events)) = raw.get("events") {
        let ev = p["events"].as_array_mut().expect("events");
        for item in events {
            if let Value::Object(o) = item {
                if matches!(o.get("t"), Some(Value::String(_))) && matches!(o.get("k"), Some(Value::String(_))) {
                    ev.push(item.clone());
                }
            }
        }
        let n = ev.len();
        if n > MAX_EVENTS {
            ev.drain(..n - MAX_EVENTS);
        }
    }
    if let Some(Value::Object(duel)) = raw.get("duel") {
        let d = p["duel"].as_object_mut().expect("duel");
        for (other, games) in duel {
            let Value::Object(games) = games else { continue };
            let mut slot = Map::new();
            for (key, wl) in games {
                if let Value::Object(wl) = wl {
                    slot.insert(
                        key.clone(),
                        json!({"w": num(wl.get("w"), 0.0, 0.0) as i64, "l": num(wl.get("l"), 0.0, 0.0) as i64}),
                    );
                }
            }
            if !slot.is_empty() {
                d.insert(other.clone(), Value::Object(slot));
            }
        }
    }
    if let Some(Value::Object(s)) = raw.get("streak") {
        let tag = |k: &str| match s.get(k) {
            Some(Value::String(x)) => kurz(x, 10),
            _ => String::new(),
        };
        p.insert(
            "streak".into(),
            json!({"cur": num(s.get("cur"), 0.0, 0.0) as i64, "best": num(s.get("best"), 0.0, 0.0) as i64,
                   "first_day": tag("first_day"), "last_day": tag("last_day")}),
        );
    }
    p.insert("points".into(), json!(num(raw.get("points"), 0.0, 0.0) as i64));
    p.insert("level".into(), json!((num(raw.get("level"), 1.0, 0.0) as i64).max(1)));
    p.insert("longest".into(), json!(num(raw.get("longest"), 0.0, 0.0)));
    prof
}

fn read_file(path: &std::path::Path) -> Option<Value> {
    let text = std::fs::read_to_string(path).ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;
    if v.is_object() {
        Some(v)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
//  Laden und Schreiben
// ---------------------------------------------------------------------------

fn ensure(s: &mut State) {
    if s.doc.is_some() {
        return;
    }
    let p = path();
    let mut raw = read_file(&p);
    if raw.is_none() {
        let mut bak = p.clone().into_os_string();
        bak.push(".bak");
        raw = read_file(std::path::Path::new(&bak));
    }
    let mut doc = blank_doc();
    if let Some(Value::Object(profiles)) = raw.as_ref().and_then(|r| r.get("profiles")) {
        let d = doc["profiles"].as_object_mut().expect("profiles");
        for (pid, prof) in profiles {
            d.insert(pid.clone(), clean_profile(prof));
        }
    }
    s.doc = Some(doc);
    if raw.is_none() {
        migrate_from_mem(s);
        s.dirty = true;
        flush_locked(s);
    }
}

fn profile_mut<'a>(s: &'a mut State, pid: Option<&str>) -> &'a mut Value {
    ensure(s);
    let pid = pid.map(String::from).unwrap_or_else(crate::store::active_id);
    let doc = s.doc.as_mut().expect("geladen");
    let profs = doc["profiles"].as_object_mut().expect("profiles");
    if !profs.contains_key(&pid) {
        profs.insert(pid.clone(), blank_profile());
    }
    profs.get_mut(&pid).expect("eben angelegt")
}

fn flush_locked(s: &mut State) {
    if !s.dirty || s.doc.is_none() {
        return;
    }
    let doc = s.doc.as_ref().expect("geladen");
    let mut out = json!({"format": FORMAT, "v": VERSION, "app": "PyGameZ", "updated": now_str(), "profiles": {}});
    let outp = out["profiles"].as_object_mut().expect("profiles");
    if let Some(profs) = doc["profiles"].as_object() {
        for (pid, prof) in profs {
            let mut entry = prof.clone();
            let e = entry.as_object_mut().expect("Profil");
            let mut games = Map::new();
            if let Some(g) = prof["games"].as_object() {
                for (key, game) in g {
                    let mut game = game.clone();
                    game["time"] = json!(round1(as_f(game.get("time"))));
                    game["longest"] = json!(round1(as_f(game.get("longest"))));
                    games.insert(key.clone(), game);
                }
            }
            e.insert("games".into(), Value::Object(games));
            let mut days = Map::new();
            if let Some(dm) = prof["days"].as_object() {
                for (day, value) in dm {
                    let mut value = value.clone();
                    value["t"] = json!(round1(as_f(value.get("t"))));
                    let mut g = Map::new();
                    if let Some(per) = value["g"].as_object() {
                        for (k, v) in per {
                            g.insert(k.clone(), json!(round1(as_f(Some(v)))));
                        }
                    }
                    value["g"] = Value::Object(g);
                    days.insert(day.clone(), value);
                }
            }
            e.insert("days".into(), Value::Object(days));
            let hours: Vec<Value> = prof["hours"]
                .as_array()
                .map(|h| h.iter().map(|v| json!(round1(as_f(Some(v))))).collect())
                .unwrap_or_default();
            e.insert("hours".into(), Value::Array(hours));
            e.insert("longest".into(), json!(round1(as_f(prof.get("longest")))));
            outp.insert(pid.clone(), entry);
        }
    }
    crate::store::write_json_atomic(&path(), &out);
    s.dirty = false;
    s.last_flush = Some(Instant::now());
}

/// `flush()` - ungespeicherte Aenderungen schreiben.
pub fn flush() {
    let mut s = st();
    flush_locked(&mut s);
}

/// `maybe_flush()` - gedrosselt schreiben.
pub fn maybe_flush() {
    let mut s = st();
    let faellig = s.last_flush.map(|t| t.elapsed().as_secs_f64() >= FLUSH_EVERY).unwrap_or(true);
    if s.dirty && faellig {
        flush_locked(&mut s);
    }
}

/// `reset_cache()` - nach einem Profilwechsel.
pub fn reset_cache() {
    let mut s = st();
    end_session_locked(&mut s, None);
    s.sess_key = None;
    s.sess_time = 0.0;
}

/// Rueckruf fuer den Profilwechsel anmelden.
pub fn register_switch_hook() {
    crate::store::on_switch(reset_cache);
}

// ---------------------------------------------------------------------------
//  Migration aus mem.json
// ---------------------------------------------------------------------------

fn migrate_from_mem(s: &mut State) {
    let mut data = crate::store::load();
    let Some(Value::Object(players)) = data.get("players").cloned() else { return };
    let mut touched = false;
    let mut players = players;
    for (pid, pdata) in players.iter_mut() {
        let prof = profile_mut(s, Some(pid));
        if let Some(Value::Object(old)) = pdata.get("stats").cloned() {
            for (key, entry) in &old {
                if !entry.is_object() {
                    continue;
                }
                let mut game = clean_game(Some(entry));
                let last = kurz(&as_s(entry.get("last")), 16);
                game["last"] = json!(last);
                if as_s(game.get("first")).is_empty() {
                    game["first"] = json!(last);
                }
                prof["games"][key] = game.clone();
                let day = kurz(&last, 10);
                if !day.is_empty() {
                    let plays = as_i(game.get("plays")).max(0);
                    let slot = day_entry(prof, &day);
                    slot["p"] = json!(as_i(slot.get("p")) + plays);
                    slot["t"] = json!(as_f(slot.get("t")) + 0.0);
                }
            }
            if let Some(o) = pdata.as_object_mut() {
                o.shift_remove("stats");
            }
            touched = true;
        }
        if let Some(Value::Object(scores)) = pdata.get("highscores") {
            for (key, value) in scores {
                let v = match value {
                    Value::Number(n) => n.as_i64().unwrap_or_else(|| n.as_f64().unwrap_or(0.0) as i64),
                    Value::String(x) => match x.trim().parse::<i64>() {
                        Ok(v) => v,
                        Err(_) => continue,
                    },
                    _ => continue,
                };
                let games = prof["games"].as_object_mut().expect("games");
                if !games.contains_key(key) {
                    games.insert(key.clone(), blank_game());
                }
                let game = games.get_mut(key).expect("eben angelegt");
                if v > as_i(game.get("best")) {
                    let last = as_s(game.get("last"));
                    game["best"] = json!(v);
                    game["best_at"] = json!(last);
                }
            }
        }
        if let Some(Value::Object(unlocked)) = pdata.get("achievements") {
            let mut items: Vec<(String, String)> =
                unlocked.iter().map(|(k, v)| (kurz(&as_s(Some(v)), 16), k.clone())).collect();
            items.sort();
            for (stamp, ach_id) in items {
                prof["events"].as_array_mut().expect("events").push(json!({"t": stamp, "k": "ach", "id": ach_id, "p": 0}));
                let day = kurz(&stamp, 10);
                if !day.is_empty() {
                    let slot = day_entry(prof, &day);
                    slot["a"] = json!(as_i(slot.get("a")) + 1);
                }
            }
            let ev = prof["events"].as_array_mut().expect("events");
            let n = ev.len();
            if n > MAX_EVENTS {
                ev.drain(..n - MAX_EVENTS);
            }
        }
        recalc_streak(prof);
    }
    if touched {
        data.insert("players".into(), Value::Object(players));
        crate::store::save(data);
    }
}

// ---------------------------------------------------------------------------
//  Hilfen
// ---------------------------------------------------------------------------

fn entry_mut<'a>(prof: &'a mut Value, key: &str) -> &'a mut Value {
    let games = prof["games"].as_object_mut().expect("games");
    if !games.contains_key(key) {
        games.insert(key.to_string(), blank_game());
    }
    games.get_mut(key).expect("eben angelegt")
}

fn day_entry<'a>(prof: &'a mut Value, day: &str) -> &'a mut Value {
    let days = prof["days"].as_object_mut().expect("days");
    if !days.contains_key(day) {
        days.insert(day.to_string(), json!({"t": 0.0, "p": 0, "w": 0, "a": 0, "g": {}}));
        if days.len() > MAX_DAYS {
            let mut alle: Vec<String> = days.keys().cloned().collect();
            alle.sort();
            let weg = alle.len() - MAX_DAYS;
            for old in &alle[..weg] {
                days.shift_remove(old);
            }
        }
    }
    days.get_mut(day).expect("Tag vorhanden")
}

fn add_i(v: &mut Value, key: &str, d: i64) {
    let neu = as_i(v.get(key)) + d;
    v[key] = json!(neu);
}

fn add_f(v: &mut Value, key: &str, d: f64) {
    let neu = as_f(v.get(key)) + d;
    v[key] = json!(neu);
}

fn recalc_streak(prof: &mut Value) {
    let mut days: Vec<String> =
        prof["days"].as_object().map(|d| d.keys().filter(|k| k.chars().count() == 10).cloned().collect()).unwrap_or_default();
    days.sort();
    if days.is_empty() {
        prof["streak"] = json!({"cur": 0, "best": 0, "first_day": "", "last_day": ""});
        return;
    }
    let (mut best, mut cur) = (1i64, 1i64);
    let mut prev: Option<chrono::NaiveDate> = None;
    for day in &days {
        let Ok(date) = chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d") else { continue };
        if let Some(p) = prev {
            cur = if (date - p).num_days() == 1 { cur + 1 } else { 1 };
        }
        prev = Some(date);
        best = best.max(cur);
    }
    let today = chrono::Local::now().date_naive();
    if let Some(p) = prev {
        if (today - p).num_days() > 1 {
            cur = 0;
        }
    }
    prof["streak"] = json!({"cur": cur, "best": best, "first_day": days[0], "last_day": days[days.len() - 1]});
}

fn add_event(prof: &mut Value, event: Value) {
    let ev = prof["events"].as_array_mut().expect("events");
    ev.push(event);
    let n = ev.len();
    if n > MAX_EVENTS {
        ev.drain(..n - MAX_EVENTS);
    }
}

// ---------------------------------------------------------------------------
//  Ereignisse
// ---------------------------------------------------------------------------

/// `game_started(key)` - eine neue Partie beginnt.
pub fn game_started(key: &str) {
    let mut s = st();
    end_session_locked(&mut s, None);
    let now = now_str();
    let today = today_str();
    let prof = profile_mut(&mut s, None);
    let entry = entry_mut(prof, key);
    add_i(entry, "plays", 1);
    entry["last"] = json!(now);
    if as_s(entry.get("first")).is_empty() {
        entry["first"] = json!(now);
    }
    add_i(day_entry(prof, &today), "p", 1);
    recalc_streak(prof);
    s.dirty = true;
    flush_locked(&mut s);
}

/// `add_playtime(key, seconds)` - aktive Spielzeit sammeln.
pub fn add_playtime(key: &str, seconds: f64) {
    if seconds <= 0.0 {
        return;
    }
    let mut s = st();
    if s.sess_key.as_deref() != Some(key) {
        end_session_locked(&mut s, None);
        s.sess_key = Some(key.to_string());
    }
    s.sess_time += seconds;
    let today = today_str();
    let hour = now_hour();
    let prof = profile_mut(&mut s, None);
    add_f(entry_mut(prof, key), "time", seconds);
    let slot = day_entry(prof, &today);
    add_f(slot, "t", seconds);
    let g = slot["g"].as_object_mut().expect("g");
    let alt = as_f(g.get(key));
    g.insert(key.to_string(), json!(alt + seconds));
    if let Some(h) = prof["hours"].as_array_mut() {
        if let Some(slot) = h.get_mut(hour) {
            *slot = json!(as_f(Some(slot)) + seconds);
        }
    }
    s.dirty = true;
}

fn end_session_locked(s: &mut State, pid: Option<&str>) -> f64 {
    let t = s.sess_time;
    let Some(key) = s.sess_key.clone().filter(|_| t > 0.0) else {
        s.sess_key = None;
        s.sess_time = 0.0;
        return 0.0;
    };
    let prof = profile_mut(s, pid);
    if let Some(entry) = prof["games"].get_mut(&key) {
        if t > as_f(entry.get("longest")) {
            entry["longest"] = json!(t);
        }
    }
    if t > as_f(prof.get("longest")) {
        prof["longest"] = json!(t);
    }
    s.sess_key = None;
    s.sess_time = 0.0;
    s.dirty = true;
    t
}

/// `end_session()` - laufende Sitzung abschliessen; gibt ihre Laenge.
pub fn end_session() -> f64 {
    let mut s = st();
    end_session_locked(&mut s, None)
}

/// `session_time()`
pub fn session_time() -> f64 {
    st().sess_time
}

/// `record_result(key, won)`
pub fn record_result(key: &str, won: bool) {
    let mut s = st();
    let today = today_str();
    let prof = profile_mut(&mut s, None);
    add_i(entry_mut(prof, key), if won { "wins" } else { "losses" }, 1);
    if won {
        add_i(day_entry(prof, &today), "w", 1);
    }
    s.dirty = true;
    flush_locked(&mut s);
}

/// `record_broken(key)` - neuer Highscore-Rekord.
pub fn record_broken(key: &str) {
    let mut s = st();
    let prof = profile_mut(&mut s, None);
    add_i(entry_mut(prof, key), "records", 1);
    s.dirty = true;
    flush_locked(&mut s);
}

/// `score_result(key, score, record)` - Punktwert in Serie und Verlauf.
pub fn score_result(key: &str, score: i64, record: bool) {
    let mut s = st();
    let stamp = now_str();
    let prof = profile_mut(&mut s, None);
    {
        let entry = entry_mut(prof, key);
        if score > as_i(entry.get("best")) {
            entry["best"] = json!(score);
            entry["best_at"] = json!(stamp);
        }
    }
    if score > 0 || record {
        let series = prof["series"].as_object_mut().expect("series");
        if !series.contains_key(key) {
            series.insert(key.to_string(), json!([]));
        }
        let list = series.get_mut(key).and_then(Value::as_array_mut).expect("Liste");
        list.push(json!([stamp, score, i64::from(record)]));
        if list.len() > MAX_SERIES {
            let rec = |p: &Value| p.get(2).map(truthy).unwrap_or(false);
            let keep: Vec<Value> = list.iter().filter(|p| rec(p)).cloned().collect();
            let rest_all: Vec<Value> = list.iter().filter(|p| !rec(p)).cloned().collect();
            let behalten = MAX_SERIES as i64 - keep.len() as i64;
            // Python: rest[-(n):] mit n = MAX_SERIES - len(keep); n <= 0 gibt alles.
            let rest: Vec<Value> = if behalten <= 0 {
                rest_all
            } else {
                let start = rest_all.len().saturating_sub(behalten as usize);
                rest_all[start..].to_vec()
            };
            let mut neu: Vec<Value> = keep.into_iter().chain(rest).collect();
            neu.sort_by(|a, b| as_s(a.get(0)).cmp(&as_s(b.get(0))));
            *list = neu;
        }
    }
    if record {
        add_event(prof, json!({"t": stamp, "k": "rec", "g": key, "v": score}));
    }
    s.dirty = true;
}

/// `record_duel(key, opponent, won, pid)` - lokales Duell festhalten.
pub fn record_duel(key: &str, opponent: &str, won: bool, pid: Option<&str>) {
    let mut s = st();
    let stamp = now_str();
    let prof = profile_mut(&mut s, pid);
    let duel = prof["duel"].as_object_mut().expect("duel");
    if !duel.contains_key(opponent) {
        duel.insert(opponent.to_string(), json!({}));
    }
    let games = duel.get_mut(opponent).and_then(Value::as_object_mut).expect("Gegner");
    if !games.contains_key(key) {
        games.insert(key.to_string(), json!({"w": 0, "l": 0}));
    }
    add_i(games.get_mut(key).expect("Spiel"), if won { "w" } else { "l" }, 1);
    add_event(prof, json!({"t": stamp, "k": "duel", "g": key, "o": opponent, "w": i64::from(won)}));
    s.dirty = true;
    flush_locked(&mut s);
}

/// `duel_total(pid)` - Anzahl aller Duelle.
pub fn duel_total(pid: Option<&str>) -> i64 {
    let mut s = st();
    let prof = profile_mut(&mut s, pid);
    let mut n = 0;
    if let Some(d) = prof["duel"].as_object() {
        for games in d.values() {
            if let Some(g) = games.as_object() {
                for wl in g.values() {
                    n += as_i(wl.get("w")) + as_i(wl.get("l"));
                }
            }
        }
    }
    n
}

/// `log_ach(ach_id, points, stamp)`
pub fn log_ach(ach_id: &str, points: i64, stamp: Option<&str>) {
    let mut s = st();
    let stamp = stamp.map(String::from).unwrap_or_else(now_str);
    let prof = profile_mut(&mut s, None);
    add_event(prof, json!({"t": stamp, "k": "ach", "id": ach_id, "p": points}));
    let day = kurz(&stamp, 10);
    if !day.is_empty() {
        add_i(day_entry(prof, &day), "a", 1);
    }
    s.dirty = true;
}

/// `log_level(level, stamp)`
pub fn log_level(level: i64, stamp: Option<&str>) {
    let mut s = st();
    let stamp = stamp.map(String::from).unwrap_or_else(now_str);
    let prof = profile_mut(&mut s, None);
    add_event(prof, json!({"t": stamp, "k": "lvl", "v": level}));
    s.dirty = true;
    flush_locked(&mut s);
}

/// `set_points(points, pid)` - Punkte und Level neu setzen.
pub fn set_points(points: i64, pid: Option<&str>) {
    let mut s = st();
    let prof = profile_mut(&mut s, pid);
    let p = points.max(0);
    prof["points"] = json!(p);
    prof["level"] = json!(crate::levels::level_for(p));
    s.dirty = true;
}

/// `points_of(pid)`
pub fn points_of(pid: Option<&str>) -> i64 {
    let mut s = st();
    as_i(profile_mut(&mut s, pid).get("points"))
}

/// `level_of(pid)` - `(Level, Punkte)`.
pub fn level_of(pid: Option<&str>) -> (i64, i64) {
    let mut s = st();
    let prof = profile_mut(&mut s, pid);
    (as_i(prof.get("level")), as_i(prof.get("points")))
}

// ---------------------------------------------------------------------------
//  Abfragen
// ---------------------------------------------------------------------------

/// Ein Spiel-Eintrag als Struktur.
#[derive(Debug, Clone, Default, PartialEq)]
#[allow(missing_docs)]
pub struct GameStats {
    pub plays: i64,
    pub time: f64,
    pub wins: i64,
    pub losses: i64,
    pub records: i64,
    pub best: i64,
    pub best_at: String,
    pub first: String,
    pub last: String,
    pub longest: f64,
}

impl GameStats {
    fn aus(v: &Value) -> GameStats {
        GameStats {
            plays: as_i(v.get("plays")),
            time: as_f(v.get("time")),
            wins: as_i(v.get("wins")),
            losses: as_i(v.get("losses")),
            records: as_i(v.get("records")),
            best: as_i(v.get("best")),
            best_at: as_s(v.get("best_at")),
            first: as_s(v.get("first")),
            last: as_s(v.get("last")),
            longest: as_f(v.get("longest")),
        }
    }
}

/// `get(key, pid)` - Eintrag eines Spiels (leer, wenn nie gespielt).
pub fn get(key: &str, pid: Option<&str>) -> GameStats {
    let mut s = st();
    let prof = profile_mut(&mut s, pid);
    prof["games"].get(key).map(GameStats::aus).unwrap_or_default()
}

/// `all_stats(pid)` - alle Spiel-Eintraege in Datei-Reihenfolge.
pub fn all_stats(pid: Option<&str>) -> Vec<(String, GameStats)> {
    let mut s = st();
    let prof = profile_mut(&mut s, pid);
    prof["games"].as_object().map(|g| g.iter().map(|(k, v)| (k.clone(), GameStats::aus(v))).collect()).unwrap_or_default()
}

/// Gesamtwerte (`totals()`).
#[derive(Debug, Clone, Default, PartialEq)]
#[allow(missing_docs)]
pub struct Totals {
    pub plays: i64,
    pub time: f64,
    pub wins: i64,
    pub losses: i64,
    pub records: i64,
    pub distinct: i64,
    pub win_games: i64,
    pub favorite: Option<String>,
    pub days: i64,
    pub streak: i64,
    pub best_streak: i64,
    pub duels: i64,
    pub points: i64,
    pub level: i64,
    pub longest: f64,
}

impl Totals {
    /// `totals.get(stat, 0)` - fuer die Erfolge.
    pub fn stat(&self, name: &str) -> f64 {
        match name {
            "plays" => self.plays as f64,
            "time" => self.time,
            "wins" => self.wins as f64,
            "losses" => self.losses as f64,
            "records" => self.records as f64,
            "distinct" => self.distinct as f64,
            "win_games" => self.win_games as f64,
            "days" => self.days as f64,
            "streak" => self.streak as f64,
            "best_streak" => self.best_streak as f64,
            "duels" => self.duels as f64,
            "points" => self.points as f64,
            "level" => self.level as f64,
            "longest" => self.longest,
            _ => 0.0,
        }
    }
}

/// `totals(pid)`
pub fn totals(pid: Option<&str>) -> Totals {
    let duels = duel_total(pid);
    let mut s = st();
    let prof = profile_mut(&mut s, pid);
    let mut t = Totals::default();
    let mut best_time = 0.0;
    if let Some(g) = prof["games"].as_object() {
        for (key, e) in g {
            let plays = as_i(e.get("plays"));
            let time = as_f(e.get("time"));
            t.plays += plays;
            t.time += time;
            t.wins += as_i(e.get("wins"));
            t.losses += as_i(e.get("losses"));
            t.records += as_i(e.get("records"));
            if plays > 0 || time > 0.0 {
                t.distinct += 1;
            }
            if as_i(e.get("wins")) > 0 {
                t.win_games += 1;
            }
            if time > best_time {
                best_time = time;
                t.favorite = Some(key.clone());
            }
        }
    }
    t.days = prof["days"]
        .as_object()
        .map(|d| d.values().filter(|v| as_f(v.get("t")) > 0.0 || as_i(v.get("p")) > 0).count() as i64)
        .unwrap_or(0);
    t.streak = as_i(prof["streak"].get("cur"));
    t.best_streak = as_i(prof["streak"].get("best"));
    t.duels = duels;
    t.points = as_i(prof.get("points"));
    t.level = as_i(prof.get("level"));
    t.longest = as_f(prof.get("longest"));
    t
}

/// Ein Kalendertag.
#[derive(Debug, Clone, Default, PartialEq)]
#[allow(missing_docs)]
pub struct DayStats {
    pub t: f64,
    pub p: i64,
    pub w: i64,
    pub a: i64,
    pub g: Vec<(String, f64)>,
}

/// `days_map(pid)` - `{Datum: {t, p, w, a, g}}` in Datei-Reihenfolge.
pub fn days_map(pid: Option<&str>) -> Vec<(String, DayStats)> {
    let mut s = st();
    let prof = profile_mut(&mut s, pid);
    prof["days"]
        .as_object()
        .map(|d| {
            d.iter()
                .map(|(day, v)| {
                    (
                        day.clone(),
                        DayStats {
                            t: as_f(v.get("t")),
                            p: as_i(v.get("p")),
                            w: as_i(v.get("w")),
                            a: as_i(v.get("a")),
                            g: v["g"].as_object().map(|g| g.iter().map(|(k, x)| (k.clone(), as_f(Some(x)))).collect()).unwrap_or_default(),
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `hours_list(pid)` - Spielzeit je Tagesstunde.
pub fn hours_list(pid: Option<&str>) -> Vec<f64> {
    let mut s = st();
    let prof = profile_mut(&mut s, pid);
    prof["hours"].as_array().map(|h| h.iter().map(|v| as_f(Some(v))).collect()).unwrap_or_else(|| vec![0.0; 24])
}

/// `weekday_list(pid)` - Spielzeit je Wochentag (Montag zuerst).
pub fn weekday_list(pid: Option<&str>) -> [f64; 7] {
    let mut out = [0.0; 7];
    for (day, e) in days_map(pid) {
        if let Some(wd) = crate::store::weekday_of(&day) {
            out[wd as usize] += e.t;
        }
    }
    out
}

/// `events(limit, kinds, pid)` - neueste zuerst.
pub fn events(limit: Option<usize>, kinds: Option<&[&str]>, pid: Option<&str>) -> Vec<Value> {
    let mut s = st();
    let prof = profile_mut(&mut s, pid);
    let mut items: Vec<Value> = prof["events"].as_array().cloned().unwrap_or_default();
    if let Some(k) = kinds.filter(|k| !k.is_empty()) {
        items.retain(|e| e.get("k").and_then(Value::as_str).map(|x| k.contains(&x)).unwrap_or(false));
    }
    items.reverse();
    match limit {
        Some(n) if n > 0 => items.into_iter().take(n).collect(),
        _ => items,
    }
}

/// `series_of(key, pid)` - `[(zeit, punkte, rekord)]`.
pub fn series_of(key: &str, pid: Option<&str>) -> Vec<(String, i64, i64)> {
    let mut s = st();
    let prof = profile_mut(&mut s, pid);
    prof["series"]
        .get(key)
        .and_then(Value::as_array)
        .map(|l| l.iter().map(|p| (as_s(p.get(0)), as_i(p.get(1)), as_i(p.get(2)))).collect())
        .unwrap_or_default()
}

/// `played_games(pid)` - Spiele mit Punkteverlauf, laengster zuerst.
pub fn played_games(pid: Option<&str>) -> Vec<String> {
    let mut s = st();
    let prof = profile_mut(&mut s, pid);
    let mut keys: Vec<(String, usize)> = prof["series"]
        .as_object()
        .map(|m| m.iter().map(|(k, v)| (k.clone(), v.as_array().map(Vec::len).unwrap_or(0))).collect())
        .unwrap_or_default();
    // Python: sorted(..., key=-len) ist stabil, `sort_by_key` auch.
    keys.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    keys.into_iter().map(|(k, _)| k).collect()
}

/// Tagesserie.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[allow(missing_docs)]
pub struct Streak {
    pub cur: i64,
    pub best: i64,
    pub first_day: String,
    pub last_day: String,
}

/// `streak(pid)`
pub fn streak(pid: Option<&str>) -> Streak {
    let mut s = st();
    let prof = profile_mut(&mut s, pid);
    let k = &prof["streak"];
    Streak { cur: as_i(k.get("cur")), best: as_i(k.get("best")), first_day: as_s(k.get("first_day")), last_day: as_s(k.get("last_day")) }
}

/// `duel_table(pid)` - `[(gegner, [(spiel, w, l)])]`.
pub fn duel_table(pid: Option<&str>) -> Vec<(String, Vec<(String, i64, i64)>)> {
    let mut s = st();
    let prof = profile_mut(&mut s, pid);
    prof["duel"]
        .as_object()
        .map(|d| {
            d.iter()
                .map(|(o, g)| {
                    (
                        o.clone(),
                        g.as_object()
                            .map(|g| g.iter().map(|(k, wl)| (k.clone(), as_i(wl.get("w")), as_i(wl.get("l")))).collect())
                            .unwrap_or_default(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// `profile_ids()`
pub fn profile_ids() -> Vec<String> {
    let mut s = st();
    ensure(&mut s);
    s.doc.as_ref().and_then(|d| d["profiles"].as_object()).map(|p| p.keys().cloned().collect()).unwrap_or_default()
}

/// `drop_profile(pid)`
pub fn drop_profile(pid: &str) {
    let mut s = st();
    ensure(&mut s);
    let weg = s.doc.as_mut().and_then(|d| d["profiles"].as_object_mut()).map(|p| p.shift_remove(pid).is_some()).unwrap_or(false);
    if weg {
        s.dirty = true;
        flush_locked(&mut s);
    }
}

/// `format_time(seconds)` - `3h 24m` / `12m 05s` / `45s`.
pub fn format_time(seconds: f64) -> String {
    let seconds = (seconds as i64).max(0);
    let (h, rest) = (seconds / 3600, seconds % 3600);
    let (m, s) = (rest / 60, rest % 60);
    if h != 0 {
        format!("{h}h {m:02}m")
    } else if m != 0 {
        format!("{m}m {s:02}s")
    } else {
        format!("{s}s")
    }
}

/// `export_to(path)`
pub fn export_to(path_out: &std::path::Path) -> bool {
    flush();
    let doc = read_file(&path()).unwrap_or_else(blank_doc);
    crate::store::write_json_atomic(path_out, &doc)
}

/// `import_from(path, target, merge)` - `(ok, grund)` mit `"io"`/`"format"`/`""`.
pub fn import_from(path_in: &std::path::Path, target: Option<&str>, merge: bool) -> (bool, &'static str) {
    let Some(raw) = read_file(path_in) else { return (false, "io") };
    if raw.get("format").and_then(Value::as_str) != Some(FORMAT) || !raw.get("profiles").map(Value::is_object).unwrap_or(false) {
        return (false, "format");
    }
    let mut s = st();
    ensure(&mut s);
    let profs: Vec<(String, Value)> =
        raw["profiles"].as_object().map(|p| p.iter().map(|(k, v)| (k.clone(), v.clone())).collect()).unwrap_or_default();
    for (pid, prof) in profs {
        let clean = clean_profile(&prof);
        let dest = target.map(String::from).unwrap_or(pid);
        let doc = s.doc.as_mut().expect("geladen");
        let dp = doc["profiles"].as_object_mut().expect("profiles");
        if merge && dp.contains_key(&dest) {
            merge_profile(dp.get_mut(&dest).expect("vorhanden"), &clean);
        } else {
            dp.insert(dest, clean);
        }
    }
    s.dirty = true;
    flush_locked(&mut s);
    (true, "")
}

fn merge_profile(dest: &mut Value, src: &Value) {
    if let Some(g) = src["games"].as_object() {
        for (key, entry) in g {
            let slot = entry_mut(dest, key);
            for f in ["plays", "wins", "losses", "records"] {
                add_i(slot, f, as_i(entry.get(f)));
            }
            add_f(slot, "time", as_f(entry.get("time")));
            slot["longest"] = json!(as_f(slot.get("longest")).max(as_f(entry.get("longest"))));
            if as_i(entry.get("best")) > as_i(slot.get("best")) {
                slot["best"] = json!(as_i(entry.get("best")));
                slot["best_at"] = json!(as_s(entry.get("best_at")));
            }
            let first = as_s(entry.get("first"));
            if !first.is_empty() && (as_s(slot.get("first")).is_empty() || first < as_s(slot.get("first"))) {
                slot["first"] = json!(first);
            }
            if as_s(entry.get("last")) > as_s(slot.get("last")) {
                slot["last"] = json!(as_s(entry.get("last")));
            }
        }
    }
    if let Some(d) = src["days"].as_object() {
        for (day, entry) in d {
            let slot = day_entry(dest, day);
            add_f(slot, "t", as_f(entry.get("t")));
            add_i(slot, "p", as_i(entry.get("p")));
            add_i(slot, "w", as_i(entry.get("w")));
            add_i(slot, "a", as_i(entry.get("a")));
            if let Some(per) = entry["g"].as_object() {
                let g = slot["g"].as_object_mut().expect("g");
                for (k, v) in per {
                    let alt = as_f(g.get(k));
                    g.insert(k.clone(), json!(alt + as_f(Some(v))));
                }
            }
        }
    }
    if let (Some(dh), Some(sh)) = (dest["hours"].as_array().cloned(), src["hours"].as_array()) {
        let neu: Vec<Value> =
            dh.iter().enumerate().map(|(i, v)| json!(as_f(Some(v)) + sh.get(i).map(|x| as_f(Some(x))).unwrap_or(0.0))).collect();
        dest["hours"] = Value::Array(neu);
    }
    if let Some(series) = src["series"].as_object() {
        for (key, points) in series {
            let mut merged: Vec<Value> = dest["series"].get(key).and_then(Value::as_array).cloned().unwrap_or_default();
            merged.extend(points.as_array().cloned().unwrap_or_default());
            merged.sort_by(|a, b| as_s(a.get(0)).cmp(&as_s(b.get(0))));
            let start = merged.len().saturating_sub(MAX_SERIES);
            dest["series"][key] = Value::Array(merged[start..].to_vec());
        }
    }
    let mut ev: Vec<Value> = dest["events"].as_array().cloned().unwrap_or_default();
    ev.extend(src["events"].as_array().cloned().unwrap_or_default());
    ev.sort_by(|a, b| as_s(a.get("t")).cmp(&as_s(b.get("t"))));
    let start = ev.len().saturating_sub(MAX_EVENTS);
    dest["events"] = Value::Array(ev[start..].to_vec());
    if let Some(duel) = src["duel"].as_object() {
        for (other, games) in duel {
            let d = dest["duel"].as_object_mut().expect("duel");
            if !d.contains_key(other) {
                d.insert(other.clone(), json!({}));
            }
            let slot = d.get_mut(other).and_then(Value::as_object_mut).expect("Gegner");
            if let Some(g) = games.as_object() {
                for (k, wl) in g {
                    if !slot.contains_key(k) {
                        slot.insert(k.clone(), json!({"w": 0, "l": 0}));
                    }
                    let cur = slot.get_mut(k).expect("Spiel");
                    add_i(cur, "w", as_i(wl.get("w")));
                    add_i(cur, "l", as_i(wl.get("l")));
                }
            }
        }
    }
    dest["points"] = json!(as_i(dest.get("points")).max(as_i(src.get("points"))));
    dest["longest"] = json!(as_f(dest.get("longest")).max(as_f(src.get("longest"))));
    recalc_streak(dest);
    let p = as_i(dest.get("points"));
    dest["level"] = json!(crate::levels::level_for(p));
}
