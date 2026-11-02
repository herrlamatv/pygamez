//! ablage.rs
//! =========
//! `pygame.scrap`.
//!
//! Die Regeln stehen im Kern ([`rustgame_display::scrap`]) - dort auch
//! der Befund, warum ein prozesslokaler Speicher hier die **richtige**
//! Nachbildung ist und keine Abkuerzung.
//!
//! Was diese Datei dazutut, ist die Veraltet-Warnung. Sechs der elf
//! Aufrufe warnen seit pygame 2.2, `lost` und `set_mode` **nicht** -
//! gemessen mit `warnings.catch_warnings(record=True)`. Das sieht nach
//! einem Versehen in pygame aus; nachgebaut wird trotzdem das Verhalten
//! und nicht das, was gemeint gewesen sein duerfte.

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use rustgame_display::scrap;

use crate::umwandlung::anzeige_fehler;

/// Meldet eine `DeprecationWarning` mit pygames Wortlaut.
fn veraltet(py: Python<'_>, name: &str, zusatz: &str) -> PyResult<()> {
    let text = format!("pygame.scrap.{name} deprecated since 2.2.0{zusatz}");
    let warnungen = py.import("warnings")?;
    let klasse = py.get_type::<pyo3::exceptions::PyDeprecationWarning>();
    warnungen.call_method1("warn", (text, klasse))?;
    Ok(())
}

/// Die Ablage meldet sich, **bevor** jemand die Argumente ansieht.
///
/// Gemessen an echtem pygame, mit siebzehn Argumenten:
///
/// ```text
/// scrap.get(*17)       pygame.error: scrap system not initialized.
/// scrap.put(*17)       pygame.error: scrap system not initialized.
/// scrap.set_mode(*17)  pygame.error: scrap system not initialized.
/// scrap.contains(*17)  pygame.error: scrap system not initialized.
/// ```
///
/// pygames C-Stueck fragt zuerst `PYGAME_SCRAP_INIT_CHECK`, und erst
/// danach liest `PyArg_ParseTuple` die Argumente. Hier war es umgekehrt:
/// PyO3 liest die Argumente, bevor der Rumpf laeuft, und ein Programm,
/// das `scrap.get()` als Beweis fuer "nichts eingerichtet" benutzt, sah
/// einen `TypeError`. `base_test.not_init_assertions` faehrt genau diese
/// Zeile.
///
/// Darum nehmen die vier Funktionen `*args` und zaehlen selbst - nach
/// dieser Pruefung.
fn eingerichtet() -> PyResult<()> {
    if !scrap::get_init() {
        return Err(crate::umwandlung::PygameError::new_err(
            "scrap system not initialized.",
        ));
    }
    Ok(())
}

/// Der Satz, den pygame fuer eine falsche Zahl von Argumenten gibt.
fn zu_viele(name: &str, wie_viele: usize, gegeben: usize) -> PyErr {
    pyo3::exceptions::PyTypeError::new_err(format!(
        "{name}() takes exactly {wie_viele} argument{} ({gegeben} given)",
        if wie_viele == 1 { "" } else { "s" }
    ))
}

/// `pygame.scrap.init()`.
#[pyfunction]
#[pyo3(name = "init")]
#[pyo3(signature = (*args, **kwargs))]
pub fn ablage_init(
    py: Python<'_>,
    args: &Bound<'_, pyo3::types::PyTuple>,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<()> {
    crate::umwandlung::keine_woerter(kwargs, "rustgame_py.scrap.init")?;
    veraltet(py, "init", "")?;
    // `init` selbst meldet sich zuerst - mit Fenster oder ohne; siehe
    // [`eingerichtet`] fuer die Reihenfolge.
    scrap::init().map_err(anzeige_fehler)?;
    if !args.is_empty() {
        return Err(zu_viele("init", 0, args.len()));
    }
    Ok(())
}

/// `pygame.scrap.get_init()`.
#[pyfunction]
#[pyo3(name = "get_init")]
pub fn ablage_get_init(py: Python<'_>) -> PyResult<bool> {
    veraltet(py, "get_init", "")?;
    Ok(scrap::get_init())
}

/// `pygame.scrap.put_text(text)` - warnt **nicht**.
#[pyfunction]
#[pyo3(name = "put_text")]
#[pyo3(signature = (text, /))]
pub fn ablage_put_text(text: &str) {
    scrap::put_text(text);
}

/// `pygame.scrap.get_text()` - warnt **nicht**.
#[pyfunction]
#[pyo3(name = "get_text")]
pub fn ablage_get_text() -> String {
    scrap::get_text()
}

/// `pygame.scrap.has_text()` - warnt **nicht**.
#[pyfunction]
#[pyo3(name = "has_text")]
pub fn ablage_has_text() -> bool {
    scrap::has_text()
}

/// `pygame.scrap.contains(typ)`.
#[pyfunction]
#[pyo3(name = "contains")]
#[pyo3(signature = (*args, **kwargs))]
pub fn ablage_contains(
    py: Python<'_>,
    args: &Bound<'_, pyo3::types::PyTuple>,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<bool> {
    crate::umwandlung::keine_woerter(kwargs, "rustgame_py.scrap.contains")?;
    veraltet(py, "contains", "")?;
    // **`contains` prueft den Zustand nicht** - und das ist der
    // Unterschied zu `get`, `put` und `set_mode` daneben. Gemessen bei
    // nicht eingerichteter Ablage:
    //
    // ```text
    // scrap.contains(*17)  TypeError: function takes exactly 1 argument
    // scrap.get(*17)       pygame.error: scrap system not initialized.
    // ```
    //
    // Hier stand ein `eingerichtet()?` auch hier, aus Gleichmass - und
    // genau das war falsch: pygames C-Stueck hat die Pruefung in drei von
    // vier Funktionen, nicht in vier. **Gleichmass ist kein Argument
    // gegen eine Messung.**
    if args.len() != 1 {
        return Err(zu_viele("contains", 1, args.len()));
    }
    let typ: String = args.get_item(0)?.extract()?;
    Ok(scrap::contains(&typ))
}

/// `pygame.scrap.get_types()`.
#[pyfunction]
#[pyo3(name = "get_types")]
pub fn ablage_get_types(py: Python<'_>) -> PyResult<Vec<String>> {
    veraltet(py, "get_types", "")?;
    scrap::get_types().map_err(anzeige_fehler)
}

/// `pygame.scrap.get(typ)`.
///
/// **Der Typ ist erst das Zweite, was geprueft wird.** Gemessen ist
/// `pygame.scrap.get()` ohne Argument ein `pygame.error` ("scrap system
/// not initialized"), kein `TypeError` - die Ablage meldet sich, bevor
/// jemand die Argumente ansieht. `base_test.not_init_assertions` faehrt
/// genau diese Zeile und nimmt sie als Beweis, dass nichts eingerichtet
/// ist; mit einem `TypeError` faellt dort der halbe Test aus.
#[pyfunction]
#[pyo3(signature = (*args, **kwargs))]
#[pyo3(name = "get")]
pub fn ablage_get(
    py: Python<'_>,
    args: &Bound<'_, pyo3::types::PyTuple>,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<Py<PyAny>> {
    crate::umwandlung::keine_woerter(kwargs, "rustgame_py.scrap.get")?;
    veraltet(
        py,
        "get",
        ". Consider using pygame.scrap.get_text instead.",
    )?;
    eingerichtet()?;
    if args.len() > 1 {
        return Err(zu_viele("get", 1, args.len()));
    }
    if args.is_empty() {
        return Err(pyo3::exceptions::PyTypeError::new_err(
            "get() missing required argument 'type' (pos 1)",
        ));
    }
    let typ: String = args.get_item(0)?.extract()?;
    let daten = scrap::get(&typ).map_err(anzeige_fehler)?;
    Ok(PyBytes::new(py, &daten).into_any().unbind())
}

/// `pygame.scrap.put(typ, daten)`.
#[pyfunction]
#[pyo3(name = "put")]
#[pyo3(signature = (*args, **kwargs))]
pub fn ablage_put(
    py: Python<'_>,
    args: &Bound<'_, pyo3::types::PyTuple>,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<()> {
    crate::umwandlung::keine_woerter(kwargs, "rustgame_py.scrap.put")?;
    veraltet(
        py,
        "put",
        ". Consider using pygame.scrap.put_text instead.",
    )?;
    eingerichtet()?;
    if args.len() != 2 {
        return Err(zu_viele("put", 2, args.len()));
    }
    let typ: String = args.get_item(0)?.extract()?;
    let daten: Vec<u8> = args.get_item(1)?.extract()?;
    scrap::put(&typ, &daten).map_err(anzeige_fehler)
}

/// `pygame.scrap.lost()` - warnt **nicht**, siehe den Modulkopf.
#[pyfunction]
#[pyo3(name = "lost")]
pub fn ablage_lost() -> PyResult<bool> {
    scrap::lost().map_err(anzeige_fehler)
}

/// `pygame.scrap.set_mode(art)` - warnt **nicht**.
#[pyfunction]
#[pyo3(name = "set_mode")]
#[pyo3(signature = (*args, **kwargs))]
pub fn ablage_set_mode(
    args: &Bound<'_, pyo3::types::PyTuple>,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<()> {
    crate::umwandlung::keine_woerter(kwargs, "rustgame_py.scrap.set_mode")?;
    eingerichtet()?;
    if args.len() != 1 {
        return Err(zu_viele("set_mode", 1, args.len()));
    }
    let art: u32 = args.get_item(0)?.extract()?;
    scrap::set_mode(art).map_err(anzeige_fehler)
}
