//! taste.rs
//! ========
//! `pygame.key`.
//!
//! ## Der Punkt, an dem fast jeder Nachbau danebengreift
//!
//! `pygame.key.get_pressed()` gibt eine Folge mit **512** Eintraegen
//! zurueck, indiziert mit dem **Tastenplatz** (`KSCAN_*`). Indiziert wird
//! sie im Programm aber mit dem **Tastencode** (`K_*`), und `K_UP` ist
//! 1073741906 - das passt nirgends hinein. pygames `ScancodeWrapper`
//! rechnet im Zugriff still um; [`rustgame_display::key::Gedrueckt`] tut
//! dasselbe sichtbar, und diese Datei reicht es durch.
//!
//! Dazu drei Dinge, die an echtem pygame gemessen sind und hier
//! nachgebildet werden:
//!
//! | Aufruf | pygame-ce 2.5.7 |
//! |---|---|
//! | `len(k)` | 512 |
//! | `k[512]`, `k[-1]` | `False` - **kein** `IndexError` |
//! | `list(k)` | `TypeError: Iterating over key states is not supported` |
//!
//! Das mittlere ist die Falle: eine Folge, die ausserhalb ihres Bereichs
//! nicht meckert, sondern `False` sagt.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

use rustgame_display::key;

/// Die 512 Rohzustaende als Tupel, nach Tastenplatz geordnet.
///
/// Das ist der **Inhalt**, den `ScancodeWrapper` traegt - siehe dort,
/// warum die Klasse in Python steht.
fn als_tupel<'p>(
    py: Python<'p>,
    g: &key::ScancodeWrapper,
) -> PyResult<Bound<'p, pyo3::types::PyTuple>> {
    pyo3::types::PyTuple::new(py, (0..g.len()).map(|i| g.roh(i as u32)))
}

/// Baut pygames `ScancodeWrapper` - **die Klasse steht in Python.**
///
/// ## Warum nicht als `#[pyclass]`
///
/// Gemessen ist `pygame.key.ScancodeWrapper` eine **Unterklasse von
/// `tuple`**:
///
/// ```text
/// isinstance(get_pressed(), tuple)   True
/// get_pressed().count(False)         512
/// get_pressed().index(False)         0
/// get_pressed() + get_pressed()      ein gewoehnliches 1024er-Tupel
/// ```
///
/// Hier stand eine eigenstaendige `#[pyclass]` mit `__len__` und
/// `__getitem__`, und das sah vollstaendig aus: `len`, Zugriff und das
/// abgelehnte Durchlaufen stimmten alle. Es fehlten `count` und `index`,
/// und es fehlte das `isinstance` - gefunden hat das `alle_klassen()` in
/// `api_coverage.py`, **0 von 2 Namen**, nachdem die Klassenliste gelesen
/// statt genannt wurde.
///
/// PyO3 kann `tuple` nicht erweitern (eine Unterklasse braucht eine feste
/// Groesse, `tuple` hat eine veraenderliche). Also steht die Klasse in
/// Python, traegt die 512 Zustaende als Tupelinhalt, und ihr
/// `__getitem__` rechnet mit `_platz` von hier um.
fn huelle(py: Python<'_>, g: &key::ScancodeWrapper) -> PyResult<Py<PyAny>> {
    let klasse = py.import("rustgame_py.key")?.getattr("ScancodeWrapper")?;
    Ok(klasse.call1((als_tupel(py, g)?,))?.unbind())
}

/// `pygame.key._platz(code)` - der Tastenplatz zu einem Tastencode.
///
/// Mit Unterstrich, weil pygame ihn nicht hat: dort steckt die Umrechnung
/// im C-Code von `ScancodeWrapper`. Hier braucht sie die Python-Klasse,
/// und ein Name ohne Unterstrich waere einer zuviel.
///
/// Ein Code, den es nicht gibt - auch ein negativer oder ein zu grosser -
/// gibt Platz **0**, und das ist gemessen: dort steht
/// `SDL_SCANCODE_UNKNOWN`, das nie unten ist.
#[pyfunction]
pub fn taste_platz(code: i64) -> u32 {
    if code < 0 || code > u32::MAX as i64 {
        return 0;
    }
    key::scancode_von(code as u32)
}

/// `pygame.key.get_pressed()`.
#[pyfunction]
#[pyo3(name = "get_pressed")]
pub fn taste_get_pressed(py: Python<'_>) -> PyResult<Py<PyAny>> {
    crate::umwandlung::video_pruefen()?;
    huelle(py, &key::get_pressed())
}

/// `pygame.key.get_mods()` - die gehaltenen Zusatztasten als Bitmaske.
#[pyfunction]
#[pyo3(name = "get_mods")]
pub fn taste_get_mods() -> PyResult<u32> {
    crate::umwandlung::video_pruefen()?;
    Ok(key::get_mods())
}

/// `pygame.key.set_mods(mods)`.
#[pyfunction]
#[pyo3(name = "set_mods")]
#[pyo3(signature = (mods, /))]
pub fn taste_set_mods(mods: u32) -> PyResult<()> {
    crate::umwandlung::video_pruefen()?;
    key::set_mods(mods);
    Ok(())
}

/// `pygame.key.name(key, use_compat=True)`.
///
/// Gemessen: `name(K_a)` ist `"a"`, `name(K_a, use_compat=False)` ist
/// `"A"` - dieselbe Taste, andere Schreibweise. Und `name(K_LSHIFT)` ist
/// `"left shift"` bzw. `"Left Shift"`.
#[pyfunction]
#[pyo3(signature = (key, use_compat=true))]
#[pyo3(name = "name")]
pub fn taste_name(key: u32, use_compat: bool) -> &'static str {
    if use_compat {
        rustgame_display::key::name(key)
    } else {
        rustgame_display::key::name_sdl(key)
    }
}

/// `pygame.key.key_code(name)` - unbekannt gibt `ValueError`, gemessen
/// samt Wortlaut.
#[pyfunction]
#[pyo3(name = "key_code")]
pub fn taste_key_code(name: &str) -> PyResult<u32> {
    key::key_code(name).ok_or_else(|| PyValueError::new_err("unknown key name"))
}

/// `pygame.key.get_focused()`.
#[pyfunction]
#[pyo3(name = "get_focused")]
pub fn taste_get_focused() -> PyResult<bool> {
    crate::umwandlung::video_pruefen()?;
    key::get_focused().map_err(crate::umwandlung::anzeige_fehler)
}

/// `pygame.key.set_repeat(delay=0, interval=0)`.
#[pyfunction]
#[pyo3(signature = (delay=0, interval=0, /))]
#[pyo3(name = "set_repeat")]
pub fn taste_set_repeat(delay: i32, interval: i32) -> PyResult<()> {
    crate::umwandlung::video_pruefen()?;
    key::set_repeat(delay, interval);
    Ok(())
}

/// `pygame.key.get_repeat()` - ohne `set_repeat` steht dort `(0, 0)`.
#[pyfunction]
#[pyo3(name = "get_repeat")]
pub fn taste_get_repeat() -> PyResult<(i32, i32)> {
    crate::umwandlung::video_pruefen()?;
    Ok(key::get_repeat())
}

/// `pygame.key.start_text_input()`.
#[pyfunction]
#[pyo3(name = "start_text_input")]
pub fn taste_start_text_input() {
    key::start_text_input();
}

/// `pygame.key.stop_text_input()`.
#[pyfunction]
#[pyo3(name = "stop_text_input")]
pub fn taste_stop_text_input() {
    key::stop_text_input();
}

/// `pygame.key.get_just_pressed()` - was in **diesem** Durchlauf
/// heruntergekommen ist.
///
/// Der Unterschied zu `get_pressed()`: dort steht, was gehalten **wird**,
/// hier, was gerade erst gedrueckt **wurde**. Ein Sprung, der auf
/// `get_pressed` hoert, loest solange aus, wie die Taste unten ist.
#[pyfunction]
#[pyo3(name = "get_just_pressed")]
pub fn taste_get_just_pressed(py: Python<'_>) -> PyResult<Py<PyAny>> {
    crate::umwandlung::video_pruefen()?;
    huelle(py, &key::get_just_pressed())
}

/// `pygame.key.get_just_released()` - Gegenstueck zu
/// [`taste_get_just_pressed`].
#[pyfunction]
#[pyo3(name = "get_just_released")]
pub fn taste_get_just_released(py: Python<'_>) -> PyResult<Py<PyAny>> {
    crate::umwandlung::video_pruefen()?;
    huelle(py, &key::get_just_released())
}

/// `pygame.key.set_text_input_rect(rect)`.
///
/// Sagt dem System, wo der Text erscheint, damit eine Eingabehilfe
/// (etwa fuer japanische Schrift) ihr Fenster daneben stellen kann.
/// Gemessen: gibt `None` zurueck.
#[pyfunction]
#[pyo3(name = "set_text_input_rect")]
#[pyo3(signature = (rect, /))]
pub fn taste_set_text_input_rect(rect: &Bound<'_, PyAny>) -> PyResult<()> {
    // pygames eigener Satz an dieser Stelle - gemessen
    // `Invalid rect argument`, nicht der aus `rect.c`.
    let r = crate::umwandlung::rect_aus(rect)
        .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Invalid rect argument"))?;
    key::set_text_input_rect(r);
    Ok(())
}
