//! ugc.rs
//! ======
//! Eigene Inhalte ("User Generated Content") - eigene Minigolf-Bahnen und
//! Geometry-Dash-Level: Speicher, Pruefung, Austausch. Uebertragen aus
//! `ugc.py`.
//!
//! Gespeichert wird in `ugc.json` im Datenordner (Python: neben dem Modul
//! bzw. der .exe - die Rust-.exe liegt mit ihren Daten an einem Ort):
//!
//! ```text
//! {"_generated": "...", "v": 1, "author": "Lama",
//!  "minigolf": [ {map}, ... ], "geodash": [ {level}, ... ]}
//! ```
//!
//! Inhalte sind JSON-Objekte (`Map`), genau wie die Python-Dicts: Kopfdaten
//! (`id`, `name`, `author`, `created`, `edited`) plus die Bahn bzw. das Level.

use std::path::Path;

use pgz_core::{profiles, store, swear};
use serde_json::{json, Map, Value};

use crate::minigolf_gen as gen;

/// Format-Version.
pub const VERSION: i64 = 1;
/// Hoechstzahl eigener Minigolf-Bahnen.
pub const MAX_MAPS: usize = 60;
/// Endung einer geteilten Bahn.
pub const EXT: &str = ".lamapgzmap";
/// Kennung im Dateikopf.
pub const FORMAT: &str = "pygamez.minigolf.map";
/// Erlaubte Zeichen einer id.
pub const ID_CHARS: &str = "abcdefghijklmnopqrstuvwxyz0123456789-_";
/// Laengste id.
pub const MAX_ID: usize = 32;
/// Laengster Name.
pub const MAX_NAME: usize = 28;
/// Laengster Creator-Name.
pub const MAX_AUTHOR: usize = 24;
/// Bahnbreite im Editor.
pub const MIN_W: f64 = 60.0;
/// Bahnbreite im Editor.
pub const MAX_W: f64 = 160.0;
/// Schrittweite Breite.
pub const STEP_W: f64 = 10.0;
/// Bahnhoehe im Editor.
pub const MIN_H: f64 = 80.0;
/// Bahnhoehe im Editor.
pub const MAX_H: f64 = 240.0;
/// Schrittweite Hoehe.
pub const STEP_H: f64 = 10.0;
/// Kleinstes Par.
pub const MIN_PAR: i64 = 1;
/// Groesstes Par.
pub const MAX_PAR: i64 = 9;

/// Spiele mit eigenen Inhalten.
pub const GAMES: [&str; 2] = ["minigolf", "geodash"];

fn path() -> std::path::PathBuf {
    store::path_for("ugc.json")
}

// ---------------------------------------------------------------------------
//  Registry der Arten
// ---------------------------------------------------------------------------

fn head_ok(m: &Map<String, Value>) -> bool {
    m.get("id").and_then(|v| v.as_str()).is_some_and(valid_id) && m.get("name").and_then(|v| v.as_str()).is_some_and(|n| !n.trim().is_empty())
}

fn validate(m: &Value, game: &str) -> bool {
    let Some(o) = m.as_object() else {
        return false;
    };
    if !head_ok(o) {
        return false;
    }
    if game == "geodash" {
        o.get("objects").is_some_and(|v| v.is_array())
    } else {
        o.get("tee").is_some_and(|v| v.is_array()) && o.get("cup").is_some_and(|v| v.is_array())
    }
}

fn normalize(m: &Map<String, Value>, game: &str) -> Map<String, Value> {
    if game == "geodash" {
        return crate::geodash_core::normalize_level(m);
    }
    golf_normalize(m)
}

/// `minigolf_gen.normalize` auf einer Map: fehlende Bahn-Schluessel ergaenzen.
pub fn golf_normalize(m: &Map<String, Value>) -> Map<String, Value> {
    let mut out = m.clone();
    out.entry("par").or_insert(json!(3));
    out.entry("tee").or_insert(json!([gen::CW / 2.0, gen::CH - 18.0]));
    out.entry("cup").or_insert(json!([gen::CW / 2.0, 22.0]));
    for (key, def) in [("w", gen::CW), ("h", gen::CH)] {
        let v = out.get(key).and_then(|x| x.as_f64().or_else(|| x.as_str().and_then(|s| s.trim().parse::<f64>().ok()))).unwrap_or(def);
        out.insert(key.into(), json!(v));
    }
    for key in gen::HOLE_LISTS {
        let ok = out.get(key).is_some_and(|v| v.is_array());
        if !ok {
            out.insert(key.into(), json!([]));
        }
    }
    out
}

/// `ext(game)`
pub fn ext(game: &str) -> &'static str {
    if game == "geodash" {
        ".lamapgzlevel"
    } else {
        EXT
    }
}

fn format_of(game: &str) -> &'static str {
    if game == "geodash" {
        "pygamez.geodash.level"
    } else {
        FORMAT
    }
}

fn key_of(game: &str) -> &'static str {
    if game == "geodash" {
        "level"
    } else {
        "map"
    }
}

/// `max_items(game)`
pub fn max_items(_game: &str) -> usize {
    60
}

// ---------------------------------------------------------------------------
//  id und Namen
// ---------------------------------------------------------------------------

/// `slug(text)` - "Mein Tunnel!" -> "mein-tunnel".
pub fn slug(text: &str) -> String {
    let mut out = text.trim().to_lowercase();
    for (src, dst) in [("\u{e4}", "ae"), ("\u{f6}", "oe"), ("\u{fc}", "ue"), ("\u{df}", "ss"), ("\u{e5}", "a"), ("\u{e6}", "ae"), ("\u{f8}", "oe")] {
        out = out.replace(src, dst);
    }
    let mapped: String = out.chars().map(|c| if ID_CHARS.contains(c) { c } else { '-' }).collect();
    let mut collapsed = String::new();
    for c in mapped.chars() {
        if c == '-' && collapsed.ends_with('-') {
            continue;
        }
        collapsed.push(c);
    }
    let trimmed = collapsed.trim_matches(|c| c == '-' || c == '_');
    trimmed.chars().take(MAX_ID).collect()
}

/// `valid_id(text)`
pub fn valid_id(text: &str) -> bool {
    let n = text.chars().count();
    n > 0 && n <= MAX_ID && text.chars().all(|c| ID_CHARS.contains(c))
}

/// `unique_id(wanted, game, ignore)`
pub fn unique_id(wanted: &str, game: &str, ignore: Option<&str>) -> String {
    let mut base = slug(wanted);
    if base.is_empty() {
        base = if game == "geodash" { "level".into() } else { "map".into() };
    }
    let taken: Vec<String> = load_maps(game).iter().filter_map(|m| m.get("id").and_then(|v| v.as_str()).map(|s| s.to_string())).filter(|id| Some(id.as_str()) != ignore).collect();
    if !taken.contains(&base) {
        return base;
    }
    for n in 2..1000 {
        let keep = MAX_ID - n.to_string().len() - 1;
        let cand = format!("{}-{}", base.chars().take(keep).collect::<String>(), n);
        if !taken.contains(&cand) {
            return cand;
        }
    }
    base
}

// ---------------------------------------------------------------------------
//  Datei lesen/schreiben
// ---------------------------------------------------------------------------

/// Liest eine JSON-Datei als Objekt (`None` bei Fehler/fehlender Datei).
fn read_json(path: &Path) -> Option<Map<String, Value>> {
    let text = std::fs::read_to_string(path).ok()?;
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(map)) => Some(map),
        _ => None,
    }
}

/// Liest `ugc.json` roh. Ist sie beschaedigt oder fehlt sie nach einem
/// abgebrochenen Speichern, wird die Sicherung `ugc.json.bak` gelesen - sonst
/// wuerde das naechste Speichern alle eigenen Bahnen und Level mit nur diesem
/// einen Inhalt ueberschreiben (wie `ugc.py`).
fn read_raw() -> Map<String, Value> {
    let path = path();
    if let Some(map) = read_json(&path) {
        return map;
    }
    let mut bak = path.as_os_str().to_os_string();
    bak.push(".bak");
    read_json(Path::new(&bak)).unwrap_or_default()
}

fn now(fmt: &str) -> String {
    chrono::Local::now().format(fmt).to_string()
}

fn write_raw(data: &Map<String, Value>) -> bool {
    let stamp = data.get("_generated").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()).unwrap_or_else(|| format!("generated by PyGameZ at {} (YYYY-MM-DD HH:MM:SS)", now("%Y-%m-%d %H:%M:%S")));
    let mut out = Map::new();
    out.insert("_generated".into(), json!(stamp));
    out.insert("v".into(), json!(VERSION));
    if let Some(a) = data.get("author").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
        out.insert("author".into(), json!(a));
    }
    let mut keys: Vec<String> = GAMES.iter().map(|s| s.to_string()).collect();
    let mut others: Vec<String> = data.keys().filter(|k| !GAMES.contains(&k.as_str())).cloned().collect();
    others.sort();
    keys.extend(others);
    for key in keys {
        if ["_generated", "v", "author"].contains(&key.as_str()) {
            continue;
        }
        if let Some(Value::Array(items)) = data.get(&key) {
            if !items.is_empty() {
                out.insert(key, Value::Array(items.clone()));
            }
        }
    }
    // Wie ugc.py ueber store.write_json_atomic: Temp-Datei + Umbenennen, die
    // vorherige Fassung bleibt als ugc.json.bak liegen (Textmodus wie bisher).
    let Ok(text) = serde_json::to_string_pretty(&Value::Object(out)) else {
        return false;
    };
    store::write_text_atomic(&path(), &text)
}

// ---------------------------------------------------------------------------
//  Sammlung
// ---------------------------------------------------------------------------

/// `load_maps(game)` - alle eigenen Inhalte (wie gespeichert).
pub fn load_maps(game: &str) -> Vec<Map<String, Value>> {
    match read_raw().get(game) {
        Some(Value::Array(items)) => items.iter().filter(|m| validate(m, game)).filter_map(|m| m.as_object().map(|o| normalize(o, game))).collect(),
        _ => Vec::new(),
    }
}

/// `get(map_id, game)`
pub fn get(map_id: &str, game: &str) -> Option<Map<String, Value>> {
    load_maps(game).into_iter().find(|m| m.get("id").and_then(|v| v.as_str()) == Some(map_id))
}

/// `count(game)`
pub fn count(game: &str) -> usize {
    load_maps(game).len()
}

/// `is_full(game)`
pub fn is_full(game: &str) -> bool {
    count(game) >= max_items(game)
}

fn s_of<'a>(m: &'a Map<String, Value>, key: &str) -> &'a str {
    m.get(key).and_then(|v| v.as_str()).unwrap_or("")
}

/// `save_map(m, game)` -> (ok, grund).
pub fn save_map(m: &Map<String, Value>, game: &str) -> (bool, &'static str) {
    let mv = Value::Object(m.clone());
    if !validate(&mv, game) {
        return (false, "invalid");
    }
    if !valid_id(s_of(m, "id")) {
        return (false, "id");
    }
    if !swear::all_clean(&[s_of(m, "name"), s_of(m, "id"), s_of(m, "author")]) {
        return (false, "swear");
    }
    let mut data = read_raw();
    let items = match data.get(game) {
        Some(Value::Array(a)) => a.clone(),
        _ => Vec::new(),
    };
    let id = s_of(m, "id").to_string();
    let mut items: Vec<Value> = items.into_iter().filter(|x| validate(x, game) && x.get("id").and_then(|v| v.as_str()) != Some(id.as_str())).collect();
    if items.len() >= max_items(game) {
        return (false, "full");
    }
    let mut m = m.clone();
    m.insert("v".into(), json!(VERSION));
    let edited = now("%Y-%m-%d %H:%M");
    m.insert("edited".into(), json!(edited));
    m.entry("created").or_insert(json!(edited));
    items.push(Value::Object(m));
    data.insert(game.into(), Value::Array(items));
    if write_raw(&data) {
        (true, "")
    } else {
        (false, "io")
    }
}

/// `delete_map(map_id, game)`
pub fn delete_map(map_id: &str, game: &str) -> bool {
    let mut data = read_raw();
    let Some(Value::Array(items)) = data.get(game).cloned() else {
        return false;
    };
    let keep: Vec<Value> = items.iter().filter(|x| !x.is_object() || x.get("id").and_then(|v| v.as_str()) != Some(map_id)).cloned().collect();
    if keep.len() == items.len() {
        return false;
    }
    data.insert(game.into(), Value::Array(keep));
    write_raw(&data)
}

// ---------------------------------------------------------------------------
//  Creator-Name
// ---------------------------------------------------------------------------

/// `last_author()` - zuletzt benutzter Creator-Name (sonst Profilname).
pub fn last_author() -> String {
    if let Some(a) = read_raw().get("author").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
        return a.to_string();
    }
    let name = profiles::name_of(None);
    if swear::is_clean(&name) {
        name
    } else {
        String::new()
    }
}

/// `set_last_author(name)`
pub fn set_last_author(name: &str) -> bool {
    let name = name.trim();
    if name.is_empty() {
        return false;
    }
    let name: String = name.chars().take(MAX_AUTHOR).collect();
    if !swear::is_clean(&name) {
        return false;
    }
    let mut data = read_raw();
    data.insert("author".into(), json!(name));
    write_raw(&data)
}

// ---------------------------------------------------------------------------
//  Bahn <-> Map (nur Minigolf)
// ---------------------------------------------------------------------------

/// `to_hole(m)` - reine Bahn aus einer Map.
pub fn to_hole(m: &Map<String, Value>) -> gen::Hole {
    let mut h = gen::Hole::from_json(&Value::Object(golf_normalize(m)));
    h.family = None;
    h
}

/// `new_map(name, map_id, author, hole)`
pub fn new_map(name: &str, map_id: &str, author: &str, hole: Option<&gen::Hole>) -> Map<String, Value> {
    let hole = hole.cloned().unwrap_or_else(|| gen::make_hole(3, (gen::CW / 2.0, gen::CH - 16.0), (gen::CW / 2.0, 22.0)));
    let stamp = now("%Y-%m-%d %H:%M");
    let mut m = Map::new();
    m.insert("v".into(), json!(VERSION));
    m.insert("id".into(), json!(map_id));
    m.insert("name".into(), json!(name));
    m.insert("author".into(), json!(author));
    m.insert("created".into(), json!(stamp));
    m.insert("edited".into(), json!(stamp));
    for (k, v) in hole.to_json() {
        m.insert(k, v);
    }
    m
}

// ---------------------------------------------------------------------------
//  Teilen
// ---------------------------------------------------------------------------

/// `default_filename(m, game)`
pub fn default_filename(m: &Map<String, Value>, game: &str) -> String {
    let fallback = if game == "geodash" { "level" } else { "map" };
    let id = s_of(m, "id");
    format!("{}{}", if id.is_empty() { fallback } else { id }, ext(game))
}

/// `export_to(m, path, game)` -> (ok, grund).
pub fn export_to(m: &Map<String, Value>, target: &str, game: &str) -> (bool, &'static str) {
    if !validate(&Value::Object(m.clone()), game) {
        return (false, "invalid");
    }
    if !swear::all_clean(&[s_of(m, "name"), s_of(m, "id"), s_of(m, "author")]) {
        return (false, "swear");
    }
    let mut payload = Map::new();
    payload.insert("format".into(), json!(format_of(game)));
    payload.insert("v".into(), json!(VERSION));
    payload.insert("app".into(), json!("PyGameZ"));
    payload.insert("exported".into(), json!(now("%Y-%m-%d %H:%M")));
    payload.insert(key_of(game).into(), Value::Object(m.clone()));
    let p = Path::new(target);
    if let Some(folder) = p.parent() {
        if !folder.as_os_str().is_empty() && !folder.is_dir() && std::fs::create_dir_all(folder).is_err() {
            return (false, "io");
        }
    }
    let text = serde_json::to_string_pretty(&Value::Object(payload)).unwrap_or_default();
    match std::fs::write(p, store::text_mode(&text).as_bytes()) {
        Ok(()) => (true, ""),
        Err(_) => (false, "io"),
    }
}

/// `detect_game(raw)`
pub fn detect_game(raw: &Value) -> Option<&'static str> {
    let f = raw.get("format")?.as_str()?;
    GAMES.iter().copied().find(|g| format_of(g) == f)
}

/// `import_from(path, game)` -> (ok, grund, inhalt).
pub fn import_from(source: &str, game: Option<&str>) -> (bool, &'static str, Option<Map<String, Value>>) {
    let Some(raw) = std::fs::read_to_string(source).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) else {
        return (false, "io", None);
    };
    if !raw.is_object() {
        return (false, "format", None);
    }
    let (found, m) = match detect_game(&raw) {
        None => {
            if !matches!(game, None | Some("minigolf")) {
                return (false, "format", None);
            }
            ("minigolf", raw.clone())
        }
        Some(f) => {
            if game.is_some_and(|g| g != f) {
                return (false, "format", None);
            }
            (f, raw.get(key_of(f)).cloned().unwrap_or(Value::Null))
        }
    };
    let Some(mut m) = m.as_object().cloned() else {
        return (false, "format", None);
    };
    let id_src = [s_of(&m, "id"), s_of(&m, "name")].into_iter().find(|s| !s.is_empty()).unwrap_or("").to_string();
    let id = slug(&id_src);
    m.insert("id".into(), json!(id));
    let name_src = if !s_of(&m, "name").is_empty() { s_of(&m, "name").to_string() } else { id.clone() };
    m.insert("name".into(), json!(name_src.trim().chars().take(MAX_NAME).collect::<String>()));
    if let Some(a) = m.get("author").and_then(|v| v.as_str()).map(|s| s.trim().chars().take(MAX_AUTHOR).collect::<String>()) {
        m.insert("author".into(), json!(a));
    }
    let m = normalize(&m, found);
    if !validate(&Value::Object(m.clone()), found) {
        return (false, "format", None);
    }
    if !swear::all_clean(&[s_of(&m, "name"), s_of(&m, "id"), s_of(&m, "author")]) {
        return (false, "swear", None);
    }
    if is_full(found) {
        return (false, "full", None);
    }
    let mut m = m;
    let uid = unique_id(s_of(&m, "id"), found, None);
    m.insert("id".into(), json!(uid));
    let (ok, reason) = save_map(&m, found);
    if ok {
        (true, "", Some(m))
    } else {
        (false, reason, None)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn slug_wie_python() {
        assert_eq!(super::slug("Mein Tunnel!"), "mein-tunnel");
        assert_eq!(super::slug("  Größe -- Über_ "), "groesse-ueber");
        assert_eq!(super::slug("!!!"), "");
        assert!(super::valid_id("a-b_9"));
        assert!(!super::valid_id("A"));
    }
}
