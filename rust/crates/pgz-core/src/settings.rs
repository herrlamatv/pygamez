//! settings.rs
//! ===========
//! Globale, dauerhaft gespeicherte Einstellungen - Gegenstueck zu
//! `settings.py`. Geschrieben wird `settings.json` im Datenordner, im selben
//! Format wie in der Python-Fassung:
//!
//! - `theme`      : UI-Design (siehe `pgz-engine::ui`)
//! - `sound`      : Soundeffekte an/aus,  `volume` 0.0 .. 1.0
//! - `haptik`     : Vibrations-Feedback (nur mit Gamepad wirksam)
//! - `resolution` : `[breite, hoehe]` der logischen Spielflaeche
//! - `fps`        : Ziel-Bildrate
//! - `controls`   : Tastenbelegung je Spieler und Aktion
//!
//! Die Tastennamen sind dieselben Zeichenketten wie in Python (Tkinter-
//! "keysym": `"Up"`, `"Left"`, `"space"`, `"Return"`, `"w"`), damit
//! bestehende `settings.json`-Dateien unveraendert weiterlaufen. Die
//! Engine uebersetzt Tastendruecke in genau diese Namen.

use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

/// Aktionen, die jedes Spiel kennt (nicht jedes nutzt alle).
pub const ACTIONS: &[&str] = &["up", "down", "left", "right", "action"];

/// Auswaehlbare Aufloesungen: `(i18n-Schluessel, Breite, Hoehe)`.
pub const RESOLUTIONS: &[(&str, i32, i32)] = &[
    ("res.480", 480, 360),
    ("res.640", 640, 480),
    ("res.800", 800, 600),
    ("res.960", 960, 720),
    ("res.1280", 1280, 960),
];

/// Auswaehlbare Ziel-Bildraten.
pub const FPS_OPTIONS: &[i32] = &[15, 30, 45, 60, 120];

/// Verfuegbare UI-Designs (Reihenfolge = Anzeige im Options-Screen).
pub const THEMES: &[&str] = &[
    "v42", "v41", "v411", "v412", "v413", "v414", "modern", "classic", "v2", "v1",
];

/// Tastenbelegung eines Spielers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bindings {
    pub up: String,
    pub down: String,
    pub left: String,
    pub right: String,
    pub action: String,
}

impl Bindings {
    fn new(up: &str, down: &str, left: &str, right: &str, action: &str) -> Self {
        Self {
            up: up.into(),
            down: down.into(),
            left: left.into(),
            right: right.into(),
            action: action.into(),
        }
    }

    /// Belegte Taste fuer eine Aktion (`"up"`, `"action"`, ...).
    pub fn get(&self, action: &str) -> &str {
        match action {
            "up" => &self.up,
            "down" => &self.down,
            "left" => &self.left,
            "right" => &self.right,
            _ => &self.action,
        }
    }

    /// Setzt die Taste einer Aktion.
    pub fn set(&mut self, action: &str, key: &str) {
        let slot = match action {
            "up" => &mut self.up,
            "down" => &mut self.down,
            "left" => &mut self.left,
            "right" => &mut self.right,
            _ => &mut self.action,
        };
        *slot = key.to_string();
    }

    /// Alle belegten Tasten.
    pub fn keys(&self) -> [&str; 5] {
        [&self.up, &self.down, &self.left, &self.right, &self.action]
    }

    fn to_value(&self) -> Value {
        json!({
            "up": self.up, "down": self.down, "left": self.left,
            "right": self.right, "action": self.action,
        })
    }

    fn from_value(raw: Option<&Value>, fallback: &Bindings) -> Self {
        let get = |k: &str, d: &str| {
            raw.and_then(|v| v.get(k))
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(d)
                .to_string()
        };
        Self {
            up: get("up", &fallback.up),
            down: get("down", &fallback.down),
            left: get("left", &fallback.left),
            right: get("right", &fallback.right),
            action: get("action", &fallback.action),
        }
    }
}

/// Standard: Spieler 1 = WASD + Leertaste, Spieler 2 = Pfeile + Enter.
pub fn default_controls() -> (Bindings, Bindings) {
    (
        Bindings::new("w", "s", "a", "d", "space"),
        Bindings::new("Up", "Down", "Left", "Right", "Return"),
    )
}

/// Auswaehlbare Vorlagen im Options-Menue: `(i18n-Schluessel, p1, p2)`.
pub fn presets() -> Vec<(&'static str, Bindings, Bindings)> {
    vec![
        (
            "preset.wasd_arrows",
            Bindings::new("w", "s", "a", "d", "space"),
            Bindings::new("Up", "Down", "Left", "Right", "Return"),
        ),
        (
            "preset.wasd_ijkl",
            Bindings::new("w", "s", "a", "d", "space"),
            Bindings::new("i", "k", "j", "l", "o"),
        ),
        (
            "preset.arrows_wasd",
            Bindings::new("Up", "Down", "Left", "Right", "Return"),
            Bindings::new("w", "s", "a", "d", "space"),
        ),
    ]
}

/// Die kompletten Einstellungen.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub theme: String,
    pub sound: bool,
    pub volume: f64,
    pub haptik: bool,
    pub resolution: (i32, i32),
    /// True -> die logische Aufloesung folgt der Fenstergroesse.
    pub auto_resolution: bool,
    pub fps: i32,
    pub p1: Bindings,
    pub p2: Bindings,
    /// Spielspezifische Abschnitte (`"snake"`, `"pong"`, `"sudoku"` ...).
    ///
    /// Sie bleiben bewusst frei geformt: jedes Spiel liest seine Optionen
    /// ueber [`Settings::game_bool`] & Co. Unbekannte Schluessel ueberleben
    /// das Speichern, damit die Python-Fassung ihre Optionen behaelt.
    pub games: Map<String, Value>,
}

impl Default for Settings {
    fn default() -> Self {
        let (p1, p2) = default_controls();
        Self {
            theme: "v42".into(),
            // Standardmaessig AUS (Erststart ist leise).
            sound: false,
            volume: 0.6,
            haptik: false,
            resolution: (640, 480),
            auto_resolution: false,
            fps: 60,
            p1,
            p2,
            games: default_game_sections(),
        }
    }
}

/// Standardwerte der spielspezifischen Abschnitte.
///
/// Sie kommen **aus der Python-Fassung** (siehe
/// [`crate::defs::GAME_DEFAULTS_JSON`], erzeugt von
/// `rust/tools/export_defs.py`) - geraten waere fatal, weil ein falscher
/// Schluessel die Option beim naechsten Laden still verschwinden liesse.
pub fn default_game_sections() -> Map<String, Value> {
    match serde_json::from_str::<Value>(crate::defs::GAME_DEFAULTS_JSON) {
        Ok(Value::Object(m)) => m,
        _ => Map::new(),
    }
}

/// Kennt die Rust-Fassung diesen Abschnitt?
///
/// Bekannt ist, was die Python-Fassung als Standard mitbringt - und
/// zusaetzlich jeder Abschnitt, der so heisst wie ein Spiel. Das ist
/// bewusst grosszuegiger als Python: dort hat jedes aeltere Spiel noch
/// einen handgeschriebenen Block in `_merge_defaults()`, der genau seine
/// Schluessel durchlaesst (Snake etwa `apples` und `mode`, die gar nicht in
/// `DEFAULTS` stehen). Diese Bloecke einzeln nachzubauen waere eine
/// Fehlerquelle ohne Gegenwert - die Rust-Fassung behaelt stattdessen, was
/// in einem bekannten Abschnitt steht.
fn section_is_known(name: &str) -> bool {
    default_game_sections().contains_key(name)
        || crate::defs::GAMES.iter().any(|g| g.key == name)
}

impl Settings {
    /// Liest eine Wahrheits-Option eines Spiels.
    pub fn game_bool(&self, section: &str, key: &str, default: bool) -> bool {
        self.games
            .get(section)
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    /// Liest eine Zahl-Option eines Spiels.
    pub fn game_i64(&self, section: &str, key: &str, default: i64) -> i64 {
        self.games
            .get(section)
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_i64())
            .unwrap_or(default)
    }

    /// Liest eine Text-Option eines Spiels.
    pub fn game_str(&self, section: &str, key: &str, default: &str) -> String {
        self.games
            .get(section)
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_str())
            .unwrap_or(default)
            .to_string()
    }

    /// Setzt eine Option eines Spiels (und legt den Abschnitt bei Bedarf an).
    pub fn set_game(&mut self, section: &str, key: &str, value: Value) {
        let entry = self
            .games
            .entry(section.to_string())
            .or_insert_with(|| json!({}));
        if let Some(obj) = entry.as_object_mut() {
            obj.insert(key.to_string(), value);
        }
    }

    /// Tastenbelegung eines Spielers (`"p1"` / `"p2"`).
    pub fn bindings(&self, player: &str) -> &Bindings {
        if player == "p2" {
            &self.p2
        } else {
            &self.p1
        }
    }

    /// Veraenderbare Tastenbelegung eines Spielers.
    pub fn bindings_mut(&mut self, player: &str) -> &mut Bindings {
        if player == "p2" {
            &mut self.p2
        } else {
            &mut self.p1
        }
    }

    /// Index der aktuellen Aufloesung in [`RESOLUTIONS`] (0, wenn eigen).
    pub fn resolution_index(&self) -> usize {
        RESOLUTIONS
            .iter()
            .position(|(_, w, h)| (*w, *h) == self.resolution)
            .unwrap_or(1)
    }
}

fn path() -> std::path::PathBuf {
    crate::store::path_for("settings.json")
}

/// Liest `settings.json` als Objekt (`None`, wenn nichts Lesbares da ist).
///
/// Ist `settings.json` beschaedigt oder fehlt sie nach einem abgebrochenen
/// Speichern, wird die Sicherung `settings.json.bak` gelesen (Stand vor dem
/// letzten Speichern, siehe `store::write_json_atomic`) - sonst waeren alle
/// Einstellungen und Tastenbelegungen weg, obwohl die Sicherung daneben liegt
/// (wie `settings.py`).
fn read_file() -> Option<Map<String, Value>> {
    let path = path();
    let mut bak = path.as_os_str().to_os_string();
    bak.push(".bak");
    for p in [path, std::path::PathBuf::from(bak)] {
        let Ok(text) = std::fs::read_to_string(&p) else {
            continue;
        };
        if let Ok(Value::Object(raw)) = serde_json::from_str::<Value>(&text) {
            return Some(raw);
        }
    }
    None
}

/// Liest `settings.json` (Standardwerte, wenn die Datei fehlt oder klemmt
/// und auch keine Sicherung da ist).
pub fn load() -> Settings {
    let mut s = Settings::default();
    let Some(raw) = read_file() else {
        return s;
    };

    if let Some(theme) = raw.get("theme").and_then(|v| v.as_str()) {
        if THEMES.contains(&theme) {
            s.theme = theme.to_string();
        }
    }
    if let Some(v) = raw.get("sound").and_then(|v| v.as_bool()) {
        s.sound = v;
    }
    if let Some(v) = raw.get("volume").and_then(|v| v.as_f64()) {
        s.volume = v.clamp(0.0, 1.0);
    }
    if let Some(v) = raw.get("haptik").and_then(|v| v.as_bool()) {
        s.haptik = v;
    }
    if let Some(v) = raw.get("auto_resolution").and_then(|v| v.as_bool()) {
        s.auto_resolution = v;
    }
    if let Some(a) = raw.get("resolution").and_then(|v| v.as_array()) {
        if a.len() == 2 {
            let w = a[0].as_i64().unwrap_or(640).clamp(320, 3840) as i32;
            let h = a[1].as_i64().unwrap_or(480).clamp(240, 2160) as i32;
            s.resolution = (w, h);
        }
    }
    if let Some(v) = raw.get("fps").and_then(|v| v.as_i64()) {
        s.fps = (v as i32).clamp(10, 240);
    }
    let (d1, d2) = default_controls();
    let controls = raw.get("controls");
    s.p1 = Bindings::from_value(controls.and_then(|c| c.get("p1")), &d1);
    s.p2 = Bindings::from_value(controls.and_then(|c| c.get("p2")), &d2);

    // Spiel-Abschnitte: die Standardwerte, darueber das Gespeicherte.
    // Unbekannte Abschnitte (etwa von einem Spiel, das es nur in Python
    // gibt) bleiben aussen vor - sie wandern beim Speichern trotzdem
    // unveraendert zurueck in die Datei, siehe `save`.
    for (name, value) in &raw {
        if !section_is_known(name) {
            continue;
        }
        let Some(saved) = value.as_object() else {
            continue;
        };
        let entry = s
            .games
            .entry(name.clone())
            .or_insert_with(|| json!({}));
        if let Some(obj) = entry.as_object_mut() {
            for (k, v) in saved {
                obj.insert(k.clone(), v.clone());
            }
        }
    }
    s
}

/// Schreibt `settings.json`.
///
/// Unbekannte Abschnitte der Datei (z.B. Optionen von Spielen, die erst in
/// der Python-Fassung existieren) bleiben dabei erhalten.
pub fn save(s: &Settings) -> bool {
    let existing: Map<String, Value> = read_file().unwrap_or_default();

    // Wie settings.py: "_generated" (Anlege-Datum der Datei) steht immer als
    // erster Eintrag, ein vorhandener Stempel bleibt erhalten.
    let stamp = match existing.get("_generated") {
        Some(Value::String(st)) if !st.is_empty() => st.clone(),
        _ => crate::store::generated_stamp(),
    };
    // Reihenfolge wie das Einstellungs-Dict in Python (DEFAULTS): Grund-
    // einstellungen, Spiel-Abschnitte, "controls".
    let mut out = Map::new();
    out.insert("_generated".into(), Value::String(stamp));
    out.insert("theme".into(), Value::String(s.theme.clone()));
    out.insert("sound".into(), Value::Bool(s.sound));
    out.insert("volume".into(), json!(s.volume));
    out.insert("haptik".into(), Value::Bool(s.haptik));
    out.insert("resolution".into(), json!([s.resolution.0, s.resolution.1]));
    out.insert("auto_resolution".into(), Value::Bool(s.auto_resolution));
    out.insert("fps".into(), json!(s.fps));
    for (name, section) in &s.games {
        out.insert(name.clone(), section.clone());
    }
    out.insert(
        "controls".into(),
        json!({"p1": s.p1.to_value(), "p2": s.p2.to_value()}),
    );
    // Was die Datei sonst noch enthaelt, bleibt hinten erhalten (Python
    // wuerde es verwerfen - so geht beim Wechsel nichts verloren).
    for (k, v) in existing {
        if k != "_generated" {
            out.entry(k).or_insert(v);
        }
    }
    crate::store::write_json_atomic(&path(), &Value::Object(out))
}

/// Speichert **eine** Spiel-Option sofort - der bequeme Weg aus einem Spiel.
///
/// Laedt die Datei frisch, aendert nur diesen einen Wert und schreibt zurueck.
/// So gehen Optionen nicht verloren, die woanders gerade geaendert wurden.
///
/// ```no_run
/// use pgz_core::settings::{store_game_option, OptionValue};
/// store_game_option("snake", "wrap", OptionValue::Bool(true));
/// ```
pub fn store_game_option(section: &str, key: &str, value: OptionValue) -> bool {
    let mut s = load();
    s.set_game(section, key, value.into_json());
    save(&s)
}

/// Der Wert einer Spiel-Option.
///
/// Haelt `serde_json` aus den Spielen heraus - die sollen von der
/// Speicherform nichts wissen muessen.
#[derive(Debug, Clone, PartialEq)]
pub enum OptionValue {
    Bool(bool),
    Int(i64),
    Text(String),
}

impl OptionValue {
    fn into_json(self) -> Value {
        match self {
            OptionValue::Bool(v) => Value::Bool(v),
            OptionValue::Int(v) => json!(v),
            OptionValue::Text(v) => Value::String(v),
        }
    }
}

impl From<bool> for OptionValue {
    fn from(v: bool) -> Self {
        OptionValue::Bool(v)
    }
}

impl From<i64> for OptionValue {
    fn from(v: i64) -> Self {
        OptionValue::Int(v)
    }
}

impl From<i32> for OptionValue {
    fn from(v: i32) -> Self {
        OptionValue::Int(v as i64)
    }
}

impl From<&str> for OptionValue {
    fn from(v: &str) -> Self {
        OptionValue::Text(v.to_string())
    }
}

/// Alle belegten Tasten beider Spieler - fuer die Konflikt-Anzeige.
pub fn used_keys(s: &Settings) -> BTreeMap<String, usize> {
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for b in [&s.p1, &s.p2] {
        for k in b.keys() {
            *counts.entry(k.to_string()).or_insert(0) += 1;
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standardwerte_passen_zu_python() {
        let s = Settings::default();
        assert_eq!(s.theme, "v42");
        assert!(!s.sound);
        assert_eq!(s.resolution, (640, 480));
        assert_eq!(s.fps, 60);
        assert_eq!(s.p1.up, "w");
        assert_eq!(s.p2.action, "Return");
    }

    #[test]
    fn tastenbelegung_laesst_sich_aendern() {
        let mut s = Settings::default();
        s.bindings_mut("p2").set("up", "i");
        assert_eq!(s.bindings("p2").up, "i");
        assert_eq!(s.bindings("p1").up, "w");
    }

    #[test]
    fn spiel_optionen_kommen_aus_der_python_fassung() {
        let d = default_game_sections();
        assert!(!d.is_empty(), "GAME_DEFAULTS_JSON liess sich nicht lesen");
        // Stichproben gegen settings.DEFAULTS der Python-Fassung.
        assert_eq!(d["pong"]["hold_p1"], serde_json::json!(false));
        assert_eq!(d["memory"]["size"], serde_json::json!("6x6"));
        assert_eq!(d["tetris"]["das"], serde_json::json!(170));
        assert_eq!(d["connect4"]["difficulty"], serde_json::json!(1));
        // 2048 heisst im Speicher "g2048", nicht "2048".
        assert!(d.contains_key("g2048"));
    }

    #[test]
    fn jeder_abschnitt_eines_spiels_gilt_als_bekannt() {
        // Sonst faellt eine Option beim naechsten Laden still weg.
        for game in crate::defs::GAMES {
            assert!(
                section_is_known(game.key),
                "Abschnitt {} gilt nicht als bekannt",
                game.key
            );
        }
        assert!(!section_is_known("gibt-es-nicht"));
    }
}
