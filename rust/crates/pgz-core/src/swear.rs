//! swear.rs
//! ========
//! Wortfilter fuer alles, was ein Spieler selbst eintippt und weitergibt
//! (Profilnamen, eigene Minigolf-Bahnen und Geometry-Dash-Level).
//! Uebertragen aus `swear.py`.
//!
//! Die Listen sind dieselben Dateien wie in Python (`lang/swear/*.yml` und
//! `lang/lang.expansion/swear/*.yml`), fest in die .exe eingebacken. Jede
//! Zeile `- muster` ist eine Regex (ohne Gross-/Kleinschreibung).
//!
//! **Vorausschau:** Die Muster enden auf `(?=[^\s]*\b)`. Die `regex`-Crate
//! kennt keine Vorausschau - fuer die reine Ja/Nein-Suche (`search`) ist
//! `A(?=X)` aber gleichwertig zu `A(?:X)`: eine Fundstelle von A mit
//! passender Vorausschau gibt es genau dann, wenn A gefolgt von X passt.
//! Genau diese Umformung passiert beim Laden (nur, wenn die Vorausschau am
//! Ende des Musters steht).
//!
//! Normalisierung wie `_norm`: NFKD, Kombinationszeichen weg, `-`/`_` zu
//! Leerzeichen, Leerraum zusammenziehen, `casefold()`, punktloses i zu `i`.

use std::sync::OnceLock;

use icu_normalizer::properties::CanonicalCombiningClassMapBorrowed;
use icu_normalizer::DecomposingNormalizerBorrowed;
use regex::Regex;

/// Laenger als das wird nichts geprueft.
pub const MAX_LEN: usize = 400;

/// Die Musterdateien in der Reihenfolge von `i18n.AVAILABLE`.
const LISTS: &[(&str, &str)] = &[
    ("de", include_str!("../../../../lang/swear/de.yml")),
    ("en", include_str!("../../../../lang/swear/en.yml")),
    ("fr", include_str!("../../../../lang/swear/fr.yml")),
    ("es", include_str!("../../../../lang/swear/es.yml")),
    ("pt", include_str!("../../../../lang/swear/pt.yml")),
    ("pl", include_str!("../../../../lang/lang.expansion/swear/pl.yml")),
    ("tr", include_str!("../../../../lang/lang.expansion/swear/tr.yml")),
    ("da", include_str!("../../../../lang/lang.expansion/swear/da.yml")),
    ("no", include_str!("../../../../lang/lang.expansion/swear/no.yml")),
    ("sv", include_str!("../../../../lang/lang.expansion/swear/sv.yml")),
    ("fi", include_str!("../../../../lang/lang.expansion/swear/fi.yml")),
    ("cs", include_str!("../../../../lang/lang.expansion/swear/cs.yml")),
    ("sl", include_str!("../../../../lang/lang.expansion/swear/sl.yml")),
    ("hr", include_str!("../../../../lang/lang.expansion/swear/hr.yml")),
];

const LOOKAHEAD: &str = r"(?=[^\s]*\b)";

/// `str.casefold()` - die Sonderfaelle gegenueber `lower()`. Dazu das punktlose
/// i (U+0131) als `i`, wie es `_norm` in Python ausdruecklich ersetzt (die
/// `regex`-Crate setzt es bei `(?i)` nicht mit `i` gleich).
fn casefold(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\u{00DF}' | '\u{1E9E}' => out.push_str("ss"),
            '\u{03C2}' => out.push('\u{03C3}'),
            '\u{017F}' => out.push('s'),
            '\u{0131}' => out.push('i'),
            _ => out.extend(c.to_lowercase()),
        }
    }
    out
}

/// `_norm(text)`
pub fn norm(text: &str) -> String {
    let text: String = text.chars().take(MAX_LEN).collect();
    let nfkd = DecomposingNormalizerBorrowed::new_nfkd().normalize(&text);
    let ccc = CanonicalCombiningClassMapBorrowed::new();
    let stripped: String = nfkd.chars().filter(|c| ccc.get_u8(*c) == 0).collect();
    let spaced = stripped.replace(['-', '_'], " ");
    let collapsed: Vec<&str> = spaced.split_whitespace().collect();
    casefold(&collapsed.join(" "))
}

/// `_variants(text)` - normalisiert, plus ohne Leerzeichen bei "G E S P E R R T".
fn variants(text: &str) -> Vec<String> {
    let n = norm(text);
    if n.is_empty() {
        return Vec::new();
    }
    let parts: Vec<&str> = n.split(' ').collect();
    let total: usize = parts.iter().map(|p| p.chars().count()).sum();
    if parts.len() >= 3 && total as f64 / parts.len() as f64 <= 1.5 {
        let joined = n.replace(' ', "");
        return vec![n, joined];
    }
    vec![n]
}

/// `_parse(text)` - kompilierte Muster einer Datei (kaputte uebersprungen).
fn parse(text: &str) -> Vec<Regex> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut pat = if let Some(rest) = line.strip_prefix("- ") {
            rest.trim().to_string()
        } else if let Some(rest) = line.strip_prefix('-') {
            rest.trim().to_string()
        } else {
            continue;
        };
        let b = pat.as_bytes();
        if b.len() >= 2 && b[0] == b[b.len() - 1] && (b[0] == b'"' || b[0] == b'\'') {
            pat = pat[1..pat.len() - 1].to_string();
        }
        if let Some(head) = pat.strip_suffix(LOOKAHEAD) {
            pat = format!("{head}(?:[^\\s]*\\b)");
        }
        if let Ok(rx) = Regex::new(&format!("(?i){pat}")) {
            out.push(rx);
        }
    }
    out
}

fn all_patterns() -> &'static Vec<(&'static str, Vec<Regex>)> {
    static P: OnceLock<Vec<(&'static str, Vec<Regex>)>> = OnceLock::new();
    P.get_or_init(|| LISTS.iter().map(|(code, text)| (*code, parse(text))).collect())
}

/// Anzahl der geladenen Muster einer Sprache (`len(patterns(code))`).
pub fn pattern_count(code: &str) -> usize {
    all_patterns().iter().find(|(c, _)| *c == code).map(|(_, p)| p.len()).unwrap_or(0)
}

/// `codes()` - die 14 Sprachen.
pub fn codes() -> Vec<&'static str> {
    LISTS.iter().map(|(c, _)| *c).collect()
}

/// `check(text)` - `Ok(())` oder `Err(sprachcode)`.
pub fn check(text: &str) -> Result<(), String> {
    let forms = variants(text);
    if forms.is_empty() {
        return Ok(());
    }
    for (code, pats) in all_patterns() {
        for rx in pats {
            if forms.iter().any(|f| rx.is_match(f)) {
                return Err(code.to_string());
            }
        }
    }
    Ok(())
}

/// `is_clean(text)`
pub fn is_clean(text: &str) -> bool {
    check(text).is_ok()
}

/// `all_clean(*texts)` - leere Texte zaehlen als sauber.
pub fn all_clean(texts: &[&str]) -> bool {
    texts.iter().filter(|t| !t.is_empty()).all(|t| is_clean(t))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alle_muster_laden() {
        let total: usize = codes().iter().map(|c| pattern_count(c)).sum();
        let lines: usize = LISTS.iter().map(|(_, t)| t.lines().filter(|l| l.trim().starts_with('-')).count()).sum();
        assert_eq!(total, lines, "jede Musterzeile muss sich uebersetzen lassen");
    }

    #[test]
    fn normalisierung_wie_python() {
        assert_eq!(norm("  P\u{00ED}c\u{00E1}-Ma_p  "), "pica ma p");
        assert_eq!(norm("Stra\u{00DF}e"), "strasse");
        assert_eq!(norm("\u{FB01}x"), "fix");
    }

    #[test]
    fn saubere_namen() {
        for n in ["Lama", "Tunnelblick", "Pick a game", "mein-tunnel", "Grassland 3", ""] {
            assert!(is_clean(n), "{n}");
        }
    }
}
