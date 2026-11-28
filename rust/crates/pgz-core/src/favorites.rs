//! favorites.rs
//! ============
//! Lieblingsspiele (Favoriten) je Profil - wie `favorites.py`.
//!
//! Gespeichert wird als eigener Abschnitt in mem.json (beim aktiven Profil):
//!
//! ```json
//! "favorites": {"games": ["snake", "tetris"]}
//! ```
//!
//! Die Liste enthaelt die `highscore_key` der Spiele in Klick-Reihenfolge;
//! angezeigt wird immer in der normalen Spiele-Reihenfolge. Python, Web und
//! Rust schreiben dieselben Bytes.
//!
//! Ob Favoriten in der Spieleliste der Seitenleiste oben stehen, ist eine
//! Einstellung (settings.json: `"favorites": {"top": false}`).

use serde_json::{json, Map, Value};

use crate::settings::Settings;
use crate::store;

/// Name des Abschnitts in mem.json.
pub const SECTION: &str = "favorites";
/// Obergrenze, damit eine kaputte/fremde Datei die Liste nicht aufblaeht.
pub const MAX_GAMES: usize = 64;
const MAX_KEY_LEN: usize = 32;

/// `_clean(raw)`: nur kurze Texte, keine Doppelten.
fn clean(raw: Option<&Value>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(list) = raw.and_then(|v| v.as_array()) {
        for k in list {
            if let Some(s) = k.as_str() {
                let n = s.chars().count();
                if n > 0 && n <= MAX_KEY_LEN && !out.iter().any(|o| o == s) {
                    out.push(s.to_string());
                }
            }
            if out.len() >= MAX_GAMES {
                break;
            }
        }
    }
    out
}

/// `load()` - Favoriten (highscore_keys) des aktiven Profils.
pub fn load() -> Vec<String> {
    clean(store::load_section(SECTION).get("games"))
}

/// `save(keys)`
pub fn save(keys: &[String]) -> bool {
    let list: Vec<Value> = keys.iter().map(|k| Value::String(k.clone())).collect();
    let cleaned = clean(Some(&Value::Array(list)));
    let mut sect = Map::new();
    sect.insert("games".into(), json!(cleaned));
    store::save_section(SECTION, sect)
}

/// `toggle(key)` - schaltet um und speichert. `true` = jetzt Favorit.
pub fn toggle(key: &str) -> bool {
    let mut favs = load();
    let now = if let Some(i) = favs.iter().position(|k| k == key) {
        favs.remove(i);
        false
    } else {
        favs.push(key.to_string());
        true
    };
    save(&favs);
    now
}

/// `ordered(classes, favs, top)` - Reihenfolge fuer die Seitenleiste als
/// Indizes in `keys` (normale Spiele-Reihenfolge). `top`: erst die
/// Favoriten, dann der Rest - beide Gruppen in normaler Reihenfolge.
pub fn ordered(keys: &[&str], favs: &[String], top: bool) -> Vec<usize> {
    let all: Vec<usize> = (0..keys.len()).collect();
    if !top || favs.is_empty() {
        return all;
    }
    let is_fav = |i: &usize| favs.iter().any(|f| f == keys[*i]);
    let mut out: Vec<usize> = all.iter().copied().filter(is_fav).collect();
    out.extend(all.iter().copied().filter(|i| !is_fav(i)));
    out
}

/// `top_enabled(settings)` - Einstellung "Favoriten oben" (Standard aus).
pub fn top_enabled(s: &Settings) -> bool {
    s.game_bool(SECTION, "top", false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reihenfolge_mit_und_ohne_oben() {
        let keys = ["snake", "pong", "tetris", "chess"];
        let favs = vec!["chess".to_string(), "pong".to_string()];
        assert_eq!(ordered(&keys, &favs, false), vec![0, 1, 2, 3]);
        assert_eq!(ordered(&keys, &favs, true), vec![1, 3, 0, 2]);
        assert_eq!(ordered(&keys, &[], true), vec![0, 1, 2, 3]);
    }

    #[test]
    fn bereinigt_wie_python() {
        let raw = json!(["snake", "snake", "", 5, "x".repeat(33), "pong"]);
        assert_eq!(clean(Some(&raw)), vec!["snake".to_string(), "pong".to_string()]);
        assert!(clean(Some(&json!("snake"))).is_empty());
        assert!(clean(None).is_empty());
    }
}
