//! i18n.rs
//! =======
//! Die Uebersetzungs-Engine der Sammlung - Gegenstueck zu `i18n.py`.
//!
//! Die Sprach-Strings sind dieselben flachen JSON-Dateien wie in der
//! Python-Fassung (`lang/*.json`, Erweiterungen in `lang/lang.expansion/`).
//! Sie werden beim Uebersetzen fest in die .exe eingebacken: die fertige
//! Datei laeuft allein, ohne Ordner daneben. Wer die Texte aendert, baut neu.
//!
//! Die gewaehlte Sprache liegt wie gehabt im Abschnitt `mem` von `mem.json`,
//! ist also zwischen Python- und Rust-Fassung austauschbar.
//!
//! ```no_run
//! pgz_core::i18n::init();
//! let text = pgz_core::i18n::t("app.quit");          // "Beenden"
//! let s = pgz_core::i18n::tf("snake.score", &[("score", "12")]);
//! ```

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

use serde_json::Value;

/// Standardsprache = Rueckfall, wenn ein Schluessel fehlt.
pub const DEFAULT_LANG: &str = "de";

/// Sofort sichtbare Sprachen im Auswahl-Screen.
pub const PRIMARY: &[(&str, &str)] = &[("de", "Deutsch"), ("en", "English"), ("fr", "Français")];

/// Sprachen hinter dem Knopf "Weitere Sprachen".
pub const EXTRA: &[(&str, &str)] = &[
    ("es", "Español"),
    ("pt", "Português"),
    ("pl", "Polski"),
    ("tr", "Türkçe"),
    ("da", "Dansk"),
    ("no", "Norsk"),
    ("sv", "Svenska"),
    ("fi", "Suomi"),
    ("cs", "Čeština"),
    ("sl", "Slovenščina"),
    ("hr", "Hrvatski"),
];

/// Die eingebetteten Sprachdateien.
const FILES: &[(&str, &str)] = &[
    ("de", include_str!("../../../../lang/de.json")),
    ("en", include_str!("../../../../lang/en.json")),
    ("fr", include_str!("../../../../lang/fr.json")),
    ("es", include_str!("../../../../lang/es.json")),
    ("pt", include_str!("../../../../lang/pt.json")),
    ("pl", include_str!("../../../../lang/lang.expansion/pl.json")),
    ("tr", include_str!("../../../../lang/lang.expansion/tr.json")),
    ("da", include_str!("../../../../lang/lang.expansion/da.json")),
    ("no", include_str!("../../../../lang/lang.expansion/no.json")),
    ("sv", include_str!("../../../../lang/lang.expansion/sv.json")),
    ("fi", include_str!("../../../../lang/lang.expansion/fi.json")),
    ("cs", include_str!("../../../../lang/lang.expansion/cs.json")),
    ("sl", include_str!("../../../../lang/lang.expansion/sl.json")),
    ("hr", include_str!("../../../../lang/lang.expansion/hr.json")),
];

/// Alle Sprachen in Anzeige-Reihenfolge: `(Code, Anzeigename)`.
pub fn available() -> Vec<(&'static str, &'static str)> {
    PRIMARY.iter().chain(EXTRA.iter()).copied().collect()
}

/// True, wenn `code` eine der 14 Sprachen ist.
pub fn is_known(code: &str) -> bool {
    FILES.iter().any(|(c, _)| *c == code)
}

type Strings = HashMap<String, String>;

fn parse(text: &str) -> Strings {
    match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(map)) => map
            .into_iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_string())))
            .collect(),
        _ => Strings::new(),
    }
}

fn load_lang(code: &str) -> Strings {
    FILES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, text)| parse(text))
        .unwrap_or_default()
}

struct State {
    current: String,
    strings: Strings,
    fallback: Strings,
}

fn state() -> &'static RwLock<State> {
    static STATE: OnceLock<RwLock<State>> = OnceLock::new();
    STATE.get_or_init(|| {
        RwLock::new(State {
            current: DEFAULT_LANG.to_string(),
            strings: Strings::new(),
            fallback: Strings::new(),
        })
    })
}

/// Name des Abschnitts in `mem.json`, in dem die Sprache liegt.
const MEM_SECTION: &str = "mem";

/// True, wenn in `mem.json` bereits eine gueltige Sprache gespeichert ist.
///
/// Ist sie es nicht, zeigt die App beim Start den Sprach-Auswahl-Screen.
pub fn has_language() -> bool {
    let mem = crate::store::load_section(MEM_SECTION);
    mem.get("lang")
        .and_then(|v| v.as_str())
        .map(is_known)
        .unwrap_or(false)
}

/// Aktiver Sprachcode, z.B. `"de"`.
pub fn get_language() -> String {
    state()
        .read()
        .map(|s| s.current.clone())
        .unwrap_or_else(|_| DEFAULT_LANG.to_string())
}

/// Setzt die aktive Sprache (und speichert sie, wenn `persist`).
pub fn set_language(code: &str, persist: bool) {
    let code = if is_known(code) { code } else { DEFAULT_LANG };
    if let Ok(mut s) = state().write() {
        s.current = code.to_string();
        s.strings = load_lang(code);
        if s.fallback.is_empty() {
            s.fallback = load_lang(DEFAULT_LANG);
        }
    }
    if persist {
        let mut mem = crate::store::load_section(MEM_SECTION);
        mem.insert("lang".into(), Value::String(code.into()));
        crate::store::save_section(MEM_SECTION, mem);
    }
}

/// Laedt Rueckfall + gespeicherte Sprache. Einmalig beim Programmstart.
pub fn init() {
    if let Ok(mut s) = state().write() {
        s.fallback = load_lang(DEFAULT_LANG);
    }
    let saved = crate::store::load_section(MEM_SECTION)
        .get("lang")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_default();
    let code = if is_known(&saved) {
        saved
    } else {
        DEFAULT_LANG.to_string()
    };
    set_language(&code, false);
}

/// Uebersetzt `key` in die aktive Sprache.
///
/// Fehlt der Schluessel, greift Deutsch; fehlt er auch dort, kommt der
/// Schluessel selbst zurueck - genau wie in der Python-Fassung.
pub fn t(key: &str) -> String {
    let Ok(s) = state().read() else {
        return key.to_string();
    };
    s.strings
        .get(key)
        .or_else(|| s.fallback.get(key))
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

/// Uebersetzt `key` und fuellt Platzhalter `{name}` aus `args`.
///
/// ```no_run
/// # use pgz_core::i18n::tf;
/// tf("snake.score", &[("score", "12")]);   // "Punkte: 12"
/// ```
pub fn tf(key: &str, args: &[(&str, &str)]) -> String {
    fill(&t(key), args)
}

/// Fuellt `{name}`-Platzhalter in einem fertigen Text.
pub fn fill(text: &str, args: &[(&str, &str)]) -> String {
    if args.is_empty() || !text.contains('{') {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len() + 16);
    let mut rest = text;
    'outer: while let Some(start) = rest.find('{') {
        // "{{" ist eine geschuetzte Klammer (wie bei str.format).
        if rest[start + 1..].starts_with('{') {
            out.push_str(&rest[..start + 1]);
            rest = &rest[start + 2..];
            continue;
        }
        let Some(end) = rest[start..].find('}').map(|i| start + i) else {
            break;
        };
        let name = &rest[start + 1..end];
        // Formatangaben wie {score:d} auf den Namen zurechtstutzen.
        let plain = name.split([':', '!']).next().unwrap_or(name);
        for (k, v) in args {
            if *k == plain {
                out.push_str(&rest[..start]);
                out.push_str(v);
                rest = &rest[end + 1..];
                continue 'outer;
            }
        }
        // Unbekannter Platzhalter: unveraendert stehen lassen.
        out.push_str(&rest[..end + 1]);
        rest = &rest[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Zahl mit Tausenderpunkten, wie sie die Oberflaeche ueberall zeigt.
pub fn num(value: i64) -> String {
    let neg = value < 0;
    let digits = value.abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    if neg {
        format!("-{out}")
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alle_vierzehn_sprachen_sind_da() {
        assert_eq!(FILES.len(), 14);
        assert_eq!(available().len(), 14);
        for (code, _) in available() {
            assert!(!load_lang(code).is_empty(), "{code} ist leer");
        }
    }

    #[test]
    fn schluesselmengen_stimmen_ueberein() {
        // Dieselbe Regel wie in der Python-Fassung: alle 14 Sprachen haben
        // identische Schluesselmengen (sonst faellt die Oberflaeche zurueck).
        let de = load_lang("de");
        for (code, _) in available() {
            let other = load_lang(code);
            let fehlend: Vec<&String> = de.keys().filter(|k| !other.contains_key(*k)).collect();
            assert!(
                fehlend.is_empty(),
                "{code} fehlen {} Schluessel, z.B. {:?}",
                fehlend.len(),
                &fehlend[..fehlend.len().min(5)]
            );
        }
    }

    #[test]
    fn platzhalter_werden_gefuellt() {
        assert_eq!(fill("Punkte: {score}", &[("score", "12")]), "Punkte: 12");
        assert_eq!(fill("{a} und {b}", &[("a", "1"), ("b", "2")]), "1 und 2");
        assert_eq!(fill("{unbekannt}", &[("a", "1")]), "{unbekannt}");
    }

    #[test]
    fn zahlen_bekommen_punkte() {
        assert_eq!(num(0), "0");
        assert_eq!(num(1234), "1.234");
        assert_eq!(num(1234567), "1.234.567");
        assert_eq!(num(-4200), "-4.200");
    }
}
