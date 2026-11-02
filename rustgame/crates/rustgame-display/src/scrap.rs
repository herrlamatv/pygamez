//! scrap.rs
//! ========
//! `pygame.scrap` - die Zwischenablage.
//!
//! ## Der Befund, der dieses Modul erst moeglich macht
//!
//! Unter `SDL_VIDEODRIVER=dummy` liest echtes pygame **nicht** die
//! Zwischenablage des Systems. Nachgemessen: erst mit PowerShell
//! `Set-Clipboard -Value "aus-powershell-123"`, dann
//! `pygame.scrap.get_text()` - heraus kommt `''`. Was `put_text` ablegt,
//! findet `get_text` wieder; von ausserhalb kommt nichts herein und nach
//! aussen geht nichts hinaus.
//!
//! Damit ist ein **prozesslokaler** Speicher hier keine Notloesung,
//! sondern genau das, was die Vorlage in diesem Betrieb tut. Kaeme
//! eines Tages ein Fensterstueck mit echter Zwischenablage dazu, wandert
//! sie hierher und die Huelle bleibt.
//!
//! ## Was pygame in diesem Zustand tut - alles gemessen
//!
//! | Aufruf | Antwort |
//! |---|---|
//! | `init()` | `pygame.error: That operation is not supported` |
//! | `get_init()` | `False` - auch nach dem Versuch |
//! | `put_text("hallo")` / `get_text()` | `None` / `'hallo'` |
//! | `has_text()` | `True`, sobald etwas abgelegt ist |
//! | `contains(typ)` | **`False`** - kein Fehler |
//! | `get_types()`, `get()`, `put()`, `lost()`, `set_mode()` | `pygame.error: scrap system not initialized.` |
//!
//! Der Punkt am Ende von "not initialized." gehoert dazu; die uebrigen
//! Meldungen dieser Kiste haben keinen.
//!
//! Die alte Haelfte des Moduls (`init`, `get_init`, `get_types`,
//! `contains`, `get`, `put`) ist seit pygame 2.2 **veraltet** und warnt.
//! `lost` und `set_mode` warnen **nicht** - auch das gemessen, und es
//! sieht nach einem Versehen in pygame aus, ist aber sein Verhalten.

use std::sync::Mutex;

use crate::{Ergebnis, Fehler};

/// Der abgelegte Text.
///
/// Prozesslokal - siehe den Modulkopf, warum das hier die richtige
/// Nachbildung ist und keine Abkuerzung.
static ABLAGE: Mutex<Option<String>> = Mutex::new(None);

/// Der Fehler, den alle alten Aufrufe geben.
fn nicht_bereit<T>() -> Ergebnis<T> {
    Err(Fehler::Pygame("scrap system not initialized."))
}

/// `pygame.scrap.init()` - scheitert, wie gemessen.
pub fn init() -> Ergebnis<()> {
    Err(Fehler::Pygame("That operation is not supported"))
}

/// `pygame.scrap.get_init()` - immer `false`.
pub fn get_init() -> bool {
    false
}

/// `pygame.scrap.put_text(text)`.
pub fn put_text(text: &str) {
    *ABLAGE.lock().unwrap() = Some(text.to_string());
}

/// `pygame.scrap.get_text()`.
///
/// Gemessen: ohne Inhalt eine **leere Zeichenkette**, nicht `None`.
pub fn get_text() -> String {
    ABLAGE.lock().unwrap().clone().unwrap_or_default()
}

/// `pygame.scrap.has_text()`.
pub fn has_text() -> bool {
    ABLAGE
        .lock()
        .unwrap()
        .as_deref()
        .map(|t| !t.is_empty())
        .unwrap_or(false)
}

/// `pygame.scrap.contains(typ)` - gibt `false`, **ohne** zu scheitern.
pub fn contains(_typ: &str) -> bool {
    false
}

/// `pygame.scrap.get_types()`.
pub fn get_types() -> Ergebnis<Vec<String>> {
    nicht_bereit()
}

/// `pygame.scrap.get(typ)`.
pub fn get(_typ: &str) -> Ergebnis<Vec<u8>> {
    nicht_bereit()
}

/// `pygame.scrap.put(typ, daten)`.
pub fn put(_typ: &str, _daten: &[u8]) -> Ergebnis<()> {
    nicht_bereit()
}

/// `pygame.scrap.lost()`.
pub fn lost() -> Ergebnis<bool> {
    nicht_bereit()
}

/// `pygame.scrap.set_mode(art)`.
pub fn set_mode(_art: u32) -> Ergebnis<()> {
    nicht_bereit()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sollwerte an echtem pygame-ce 2.5.7 unter `SDL_VIDEODRIVER=dummy`
    /// gemessen.
    #[test]
    fn die_ablage_verhaelt_sich_wie_pygame() {
        assert!(init().is_err(), "init scheitert unter dummy");
        assert!(!get_init());

        put_text("hallo");
        assert_eq!(get_text(), "hallo");
        assert!(has_text());

        // Leerer Text zaehlt nicht als Inhalt.
        put_text("");
        assert_eq!(get_text(), "");
        assert!(!has_text());

        // `contains` gibt `false` **ohne** Fehler - anders als der Rest.
        assert!(!contains("text/plain"));
        assert!(get_types().is_err());
        assert!(get("text/plain").is_err());
        assert!(put("text/plain", b"x").is_err());
        assert!(lost().is_err());
        assert!(set_mode(0).is_err());
    }
}
