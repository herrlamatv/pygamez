//! rechner.rs
//! ==========
//! `pygame.system`.
//!
//! Die Regeln stehen im Kern ([`rustgame_display::system`]) - dort auch
//! die Begruendung, warum drei der sechs Aufrufe einen klaren Fehler
//! melden statt einer erfundenen Zahl.
//!
//! `PowerState` gibt es hier trotzdem als Klasse, obwohl
//! `get_power_state()` sie nie liefert. Derselbe Grund wie bei
//! `joystick.Joystick`: ein Programm soll den Namen nennen koennen
//! (`isinstance`, Typangaben), und ein `AttributeError` auf dem Modul
//! waere blosse Ratlosigkeit statt einer Auskunft.

use pyo3::prelude::*;
use pyo3::types::PyDict;

use rustgame_display::system;

use crate::umwandlung::anzeige_fehler;

// **`PowerState` steht in Python**, nicht hier.
//
// Hier stand eine `#[pyclass]` mit acht Feldern und einem `__repr__`,
// das pygames Zeile Wort fuer Wort nachbaute. Gemessen ist pygames
// `PowerState` aber ein **echter** `@dataclass(frozen=True)`, und daran
// haengt mehr als das `repr`:
//
//     __match_args__          ('battery_percent', ...)  -  `match`-Muster
//     __dataclass_fields__    dataclasses.fields(P)
//     __replace__             dataclasses.replace(p, ...)
//     frozen                  p.battery_percent = 1 -> FrozenInstanceError
//
// Dazu hatte die Rust-Fassung **acht Namen zuviel**: `get_all` legt die
// Felder als Klassenbeschreiber ab, ein `@dataclass` ohne `slots` nicht
// (`dir(pygame.system.PowerState)` hat **keinen** oeffentlichen Namen).
//
// Gefunden hat beides die Probe `Klassendoppel` zusammen mit
// `Klassennamen`. Die Umsetzung ist jetzt pygames eigene, Zeile fuer
// Zeile: `src/python/data_classes.py`.
//
// Dass das geht, liegt an einer Messung: `get_power_state()` gibt hier
// **immer `None`** (siehe unten), die Klasse wird also von Rust aus nie
// gebaut.

/// `pygame.system.get_cpu_instruction_sets()`.
///
/// Die Reihenfolge der vierzehn Namen ist die von pygame - ein
/// Woerterbuch behaelt sie in Python, also faellt eine andere auf.
#[pyfunction]
#[pyo3(name = "get_cpu_instruction_sets")]
pub fn rechner_get_cpu_instruction_sets(py: Python<'_>) -> PyResult<Py<PyAny>> {
    let d = PyDict::new(py);
    for (name, kann) in system::get_cpu_instruction_sets() {
        d.set_item(name, kann)?;
    }
    Ok(d.into_any().unbind())
}

/// `pygame.system.get_pref_path(org, app)`.
#[pyfunction]
#[pyo3(name = "get_pref_path")]
pub fn rechner_get_pref_path(org: &str, app: &str) -> PyResult<String> {
    system::get_pref_path(org, app).map_err(anzeige_fehler)
}

/// `pygame.system.get_total_ram()` - meldet einen klaren Fehler.
#[pyfunction]
#[pyo3(name = "get_total_ram")]
pub fn rechner_get_total_ram() -> PyResult<u64> {
    system::get_total_ram().map_err(anzeige_fehler)
}

/// `pygame.system.get_power_state()` - meldet einen klaren Fehler.
#[pyfunction]
#[pyo3(name = "get_power_state")]
pub fn rechner_get_power_state(py: Python<'_>) -> PyResult<Py<PyAny>> {
    // **`None`, kein Fehler** - die Begruendung steht im Kern. Ein
    // `unreachable!` stand hier, weil der Kern immer scheiterte; das ist
    // jetzt nicht mehr so, und ein `unreachable!`, das doch erreichbar
    // wird, ist die unangenehmste Sorte Vermerk.
    match system::get_power_state().map_err(anzeige_fehler)? {
        None => Ok(py.None()),
        Some(()) => Ok(py.None()),
    }
}

/// `pygame.system.get_pref_locales()` - meldet einen klaren Fehler.
#[pyfunction]
#[pyo3(name = "get_pref_locales")]
pub fn rechner_get_pref_locales() -> PyResult<Vec<(String, Option<String>)>> {
    system::get_pref_locales().map_err(anzeige_fehler)
}
