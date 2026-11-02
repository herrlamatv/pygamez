//! tk.rs
//! =====
//! Tkinter-Tastennamen in Tastencodes umrechnen - und zurueck.
//!
//! ## Warum das hier steht
//!
//! Das umgebende Projekt (pygamez) speichert Tastenbelegungen in seiner
//! Einstellungsdatei als **Tkinter-`keysym`**, nicht als pygame-Code:
//!
//! ```text
//! "p1": {"up": "w", "down": "s", "left": "a", "right": "d", "action": "space"},
//! "p2": {"up": "Up", "down": "Down", "left": "Left", "right": "Right",
//!        "action": "Return"},
//! ```
//!
//! Das kommt daher, dass die Oberflaeche auf Tkinter sitzt und
//! `event.keysym` unmittelbar weiterreicht. Wer diese Datei aus rustgame
//! liest, steht also vor `"Return"` und braucht 13.
//!
//! Ohne diese Umrechnung waere genau da die Bruchstelle - und zwar eine
//! stille: `"w"` und `"space"` loesen sich von selbst auf, weil pygame
//! dieselben Namen kennt, `"Return"` und `"Prior"` aber nicht. Die
//! Belegung des zweiten Spielers waere kaputt, die des ersten nicht, und
//! niemand kaeme auf die Idee, in der Namensumrechnung zu suchen.
//!
//! ## Die beiden Faelle, die man nicht raet
//!
//! Tkinter nennt Bild-auf **`"Prior"`** und Bild-ab **`"Next"`** - Namen
//! aus X11, die mit "Seite" nichts zu tun haben. Ausserdem heissen die
//! Zeichentasten bei Tkinter ausgeschrieben: `"space"`, `"period"`,
//! `"minus"`, `"comma"`, `"plus"`, `"slash"`, `"bracketleft"`.
//!
//! Die Tabelle selbst steht in [`crate::konstanten::TK_KEYSYMS`] und ist
//! erzeugt, nicht getippt.

use crate::konstanten::{Key, TK_KEYSYMS};

/// Tkinter-`keysym` in einen Tastencode - `"Up"` wird zu `K_UP`.
///
/// ```
/// use rustgame_display::{konstanten::*, tk};
///
/// assert_eq!(tk::key_code("Up"), Some(K_UP));
/// assert_eq!(tk::key_code("space"), Some(K_SPACE));
/// assert_eq!(tk::key_code("Return"), Some(K_RETURN));
/// assert_eq!(tk::key_code("w"), Some(K_w));
/// // Die beiden aus X11, die man nicht raet:
/// assert_eq!(tk::key_code("Prior"), Some(K_PAGEUP));
/// assert_eq!(tk::key_code("Next"), Some(K_PAGEDOWN));
/// ```
///
/// Unbekannte Namen ergeben `None`. Ein einzelnes Zeichen, das nicht in
/// der Tabelle steht (etwa `"ä"`), kommt als sein Unicode-Wert zurueck -
/// genauso macht es `pygame.key.key_code`.
pub fn key_code(keysym: &str) -> Option<Key> {
    if let Ok(i) = TK_KEYSYMS.binary_search_by_key(&keysym, |(n, _)| *n) {
        return Some(TK_KEYSYMS[i].1);
    }
    crate::key::key_code(keysym)
}

/// Ein Tastencode als Tkinter-`keysym` - `K_UP` wird zu `"Up"`.
///
/// Die Umkehrung von [`key_code`]. Eindeutig ist sie nicht: mehrere
/// Tkinter-Namen zeigen auf denselben Code (`"apostrophe"` und
/// `"quoteright"` etwa), zurueck kommt der erste in alphabetischer
/// Reihenfolge. Fuer das Speichern einer Belegung reicht das - beim
/// Wiederlesen landet man beim selben Code.
pub fn keysym(key: Key) -> Option<&'static str> {
    TK_KEYSYMS
        .iter()
        .find(|(_, k)| *k == key)
        .map(|(n, _)| *n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::konstanten::*;

    #[test]
    fn die_namen_aus_der_einstellungsdatei() {
        // Genau die Werte, die in settings.py stehen.
        for (name, erwartet) in [
            ("w", K_w),
            ("s", K_s),
            ("a", K_a),
            ("d", K_d),
            ("space", K_SPACE),
            ("Up", K_UP),
            ("Down", K_DOWN),
            ("Left", K_LEFT),
            ("Right", K_RIGHT),
            ("Return", K_RETURN),
        ] {
            assert_eq!(key_code(name), Some(erwartet), "keysym {name}");
        }
    }

    #[test]
    fn prior_und_next_sind_die_bildtasten() {
        assert_eq!(key_code("Prior"), Some(K_PAGEUP));
        assert_eq!(key_code("Next"), Some(K_PAGEDOWN));
    }

    #[test]
    fn hin_und_zurueck() {
        for name in ["Up", "Down", "Left", "Right", "Return", "space", "w"] {
            let code = key_code(name).unwrap();
            let zurueck = keysym(code).unwrap();
            assert_eq!(
                key_code(zurueck),
                Some(code),
                "{name} -> {code} -> {zurueck}"
            );
        }
    }

    #[test]
    fn unbekanntes_bleibt_unbekannt() {
        assert_eq!(key_code("Gibtsnicht"), None);
    }

    #[test]
    fn die_tabelle_ist_sortiert() {
        // Sonst geht die binaere Suche daneben - still und unauffaellig.
        for paar in TK_KEYSYMS.windows(2) {
            assert!(paar[0].0 < paar[1].0, "{} vor {}", paar[0].0, paar[1].0);
        }
    }
}
