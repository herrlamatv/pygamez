//! highscore.rs
//! ============
//! Laedt und speichert Highscores. Sie liegen im Abschnitt `highscores` von
//! `mem.json` (siehe [`crate::store`]) - je Profil getrennt.
//! Gegenstueck zu `highscore.py`.

use std::collections::BTreeMap;

use serde_json::Value;

const SECTION: &str = "highscores";

/// Liest alle Highscores des aktiven Profils als `{spiel_key: score}`.
pub fn load() -> BTreeMap<String, i64> {
    load_ordered().into_iter().collect()
}

/// Wie [`load`], aber in Datei-Reihenfolge (Pythons dict) - wichtig, wo die
/// Reihenfolge sichtbar wird (Nachtragen der Erfolge -> Verlauf).
pub fn load_ordered() -> Vec<(String, i64)> {
    crate::store::load_section(SECTION)
        .iter()
        .filter_map(|(k, v)| value_to_score(v).map(|n| (k.clone(), n)))
        .collect()
}

/// Beschaedigte Werte (Text, Kommazahl) werden weich behandelt - lieber ein
/// gerundeter Wert als ein verlorener Highscore.
fn value_to_score(v: &Value) -> Option<i64> {
    v.as_i64()
        .or_else(|| v.as_f64().map(|f| f as i64))
        .or_else(|| v.as_str().and_then(|s| s.trim().parse::<i64>().ok()))
}

/// Schreibt die Tabelle `{spiel_key: score}` zurueck.
pub fn save(scores: &BTreeMap<String, i64>) -> bool {
    let section = scores
        .iter()
        .map(|(k, v)| (k.clone(), Value::from(*v)))
        .collect();
    crate::store::save_section(SECTION, section)
}

/// Highscore eines Spiels (0, wenn noch keiner da ist).
pub fn get(key: &str) -> i64 {
    load().get(key).copied().unwrap_or(0)
}

/// Aktualisiert den Highscore fuer `key`, falls `score` besser ist.
///
/// Gibt `(neuer_highscore, ist_neuer_rekord)` zurueck.
pub fn update(key: &str, score: i64) -> (i64, bool) {
    // Python: dict aktualisieren - vorhandene Schluessel behalten ihren
    // Platz, neue kommen ans Ende.
    let mut scores: serde_json::Map<String, Value> = load_ordered().into_iter().map(|(k, v)| (k, Value::from(v))).collect();
    let old = scores.get(key).and_then(Value::as_i64).unwrap_or(0);
    if score > old {
        scores.insert(key.to_string(), Value::from(score));
        crate::store::save_section(SECTION, scores);
        (score, true)
    } else {
        (old, false)
    }
}
