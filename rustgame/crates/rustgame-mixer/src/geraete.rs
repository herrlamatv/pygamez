//! geraete.rs
//! ==========
//! Die Tongeraete des Rechners - Gegenstueck zu
//! `pygame._sdl2.audio.get_audio_device_names`.
//!
//! ## Warum das hier liegt und nicht im Treiber
//!
//! Der Treiber ([`crate::treiber`]) oeffnet **ein** Geraet und macht Schall.
//! Welche Geraete es gibt, ist eine andere Frage - sie wird auch dann
//! gestellt, wenn gar kein Ton laeuft, und sie darf kein Geraet oeffnen.
//!
//! ## Gemessen
//!
//! `cpal` (das unter `rodio` liegt) sieht auf dieser Maschine **genau
//! dieselben** Geraete wie SDL, in derselben Reihenfolge:
//!
//! ```text
//! SDL (pygame._sdl2.audio)          cpal
//!   MSI G24C4 (NVIDIA ...)            MSI G24C4 (NVIDIA ...)
//!   Speakers (Realtek USB Audio)      Speakers (Realtek USB Audio)
//!   MSI G24C4 E2 (NVIDIA ...)         MSI G24C4 E2 (NVIDIA ...)
//!   Realtek Digital Output (...)      Realtek Digital Output (...)
//!   MSI G24C4 (NVIDIA ...) **(2)**    MSI G24C4 (NVIDIA ...)
//! ```
//!
//! Ein einziger Unterschied, und er steht in SDLs eigener Beschreibung:
//! *"Returns a list of **unique** devicenames"*. Zwei Geraete koennen
//! denselben Namen tragen - hier zweimal derselbe Bildschirmlautsprecher -,
//! und SDL haengt dem zweiten ` (2)` an. Genau das tut [`eindeutig`].
//!
//! Eingabegeraete werden getrennt gezaehlt; dort steht auf dieser Maschine
//! das Mikrofon der Kamera.

use rodio::cpal::traits::{DeviceTrait, HostTrait};

/// Die Namen aller Tongeraete - **eindeutig gemacht**.
///
/// `eingabe = true` fragt die Aufnahmegeraete, `false` die Ausgabe. Eine
/// leere Liste ist keine Ausnahme: ein Rechner ohne Soundkarte bekommt sie
/// und keinen Fehler, genau wie bei pygame.
#[must_use]
pub fn geraetenamen(eingabe: bool) -> Vec<String> {
    let host = rodio::cpal::default_host();
    let roh: Vec<String> = if eingabe {
        host.input_devices()
            .map(|it| it.filter_map(|d| d.name().ok()).collect())
            .unwrap_or_default()
    } else {
        host.output_devices()
            .map(|it| it.filter_map(|d| d.name().ok()).collect())
            .unwrap_or_default()
    };
    eindeutig(&roh)
}

/// Macht eine Namensliste eindeutig - SDLs Regel.
///
/// Der erste Name bleibt, jeder weitere gleiche bekommt ` (2)`, ` (3)` und
/// so weiter. Gemessen an SDL: zwei Geraete mit demselben Namen ergeben
/// `"X"` und `"X (2)"`.
#[must_use]
pub fn eindeutig(namen: &[String]) -> Vec<String> {
    let mut gezaehlt: std::collections::HashMap<&str, u32> =
        std::collections::HashMap::new();
    namen
        .iter()
        .map(|n| {
            let z = gezaehlt.entry(n.as_str()).or_insert(0);
            *z += 1;
            if *z == 1 {
                n.clone()
            } else {
                format!("{n} ({z})")
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SDLs Regel, an dem Fall, der sie ausgeloest hat.
    #[test]
    fn doppelte_namen_bekommen_eine_nummer() {
        let roh: Vec<String> = [
            "MSI G24C4 (NVIDIA)",
            "Speakers",
            "MSI G24C4 E2 (NVIDIA)",
            "Realtek Digital Output",
            "MSI G24C4 (NVIDIA)",
        ]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
        assert_eq!(
            eindeutig(&roh),
            vec![
                "MSI G24C4 (NVIDIA)",
                "Speakers",
                "MSI G24C4 E2 (NVIDIA)",
                "Realtek Digital Output",
                "MSI G24C4 (NVIDIA) (2)",
            ]
        );
    }

    /// Drei gleiche geben `(2)` und `(3)`, nicht zweimal `(2)`.
    #[test]
    fn drei_gleiche_zaehlen_weiter() {
        let roh: Vec<String> = ["A", "A", "A"].iter().map(|s| (*s).to_string()).collect();
        assert_eq!(eindeutig(&roh), vec!["A", "A (2)", "A (3)"]);
    }

    /// Eine leere Liste bleibt leer.
    #[test]
    fn leer_bleibt_leer() {
        assert!(eindeutig(&[]).is_empty());
    }

    /// Die echte Liste ist widerspruchsfrei - auch auf einem Rechner ohne
    /// Soundkarte, wo sie leer ist.
    #[test]
    fn die_geraeteliste_ist_widerspruchsfrei() {
        for eingabe in [false, true] {
            let n = geraetenamen(eingabe);
            let mut sortiert = n.clone();
            sortiert.sort();
            sortiert.dedup();
            assert_eq!(sortiert.len(), n.len(), "Namen sind nicht eindeutig");
            assert!(n.iter().all(|s| !s.is_empty()), "ein Geraet ohne Namen");
        }
    }
}
