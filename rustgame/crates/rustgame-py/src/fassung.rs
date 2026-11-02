//! fassung.rs
//! ==========
//! `pygame.version`.
//!
//! ## Warum hier Python-Quelltext steht und kein `#[pyclass]`
//!
//! `pygame/version.py` baut `SoftwareVersion` als **Unterklasse von
//! `tuple`**. Daran haengt mehr, als man auf den ersten Blick sieht:
//!
//! ```text
//! vernum == (2, 5, 7)   ->  True
//! vernum >= (2, 0, 0)   ->  True      <- die uebliche Fassungsabfrage
//! tuple(vernum)         ->  (2, 5, 7)
//! len(vernum)           ->  3
//! str(vernum)           ->  '2.5.7'
//! repr(vernum)          ->  'PygameVersion(major=2, minor=5, patch=7)'
//! ```
//!
//! Ein `#[pyclass]` muesste `__eq__`, `__lt__`, `__le__`, `__gt__`,
//! `__ge__`, `__len__`, `__getitem__`, `__iter__` und `__hash__` einzeln
//! nachbauen - und jede dieser neun Stellen waere eine Gelegenheit,
//! anders zu vergleichen als ein echtes Tupel. PyO3 kann nicht von
//! `tuple` erben.
//!
//! Deshalb steht die Klasse hier als **uebertragener** Python-Quelltext,
//! Zeile fuer Zeile aus `pygame/version.py`. Das ist dieselbe
//! Entscheidung wie bei `sysfont.py` und `sprite.py`: was in pygame schon
//! Python ist, wird uebernommen und nicht nachempfunden.
//!
//! ## Was rustgame hier meldet - und warum es pygames Zahlen sind
//!
//! `vernum` ist **2.5.7**, nicht eine eigene Zaehlung. Das ist keine
//! Hochstapelei, sondern die Antwort auf die Frage, die gestellt wird:
//! `pygame.version.vernum >= (2, 0, 0)` fragt "kann ich mich auf die
//! API von 2.0 verlassen?", und die Antwort ist ja - genau dafuer gibt
//! es dieses Projekt.
//!
//! `rev` ist dagegen **`"rustgame"`** statt der leeren Zeichenkette, die
//! echtes pygame dort hat. `rev` ist die Quelltextfassung, und die ist
//! nun einmal eine andere. Wer wissen will, worauf er laeuft, findet es
//! dort - ohne dass eine Fassungsabfrage danebengeht.

use pyo3::prelude::*;
use pyo3::types::PyModule;

/// Zeilengetreu aus `pygame/version.py` (pygame-ce 2.5.7).
const VERSION_PY: &str = r#"
class SoftwareVersion(tuple):
    """A class for storing data about software versions."""

    __slots__ = ()
    fields = "major", "minor", "patch"

    def __new__(cls, major, minor, patch):
        return tuple.__new__(cls, (major, minor, patch))

    def __repr__(self):
        fields = (f"{fld}={val}" for fld, val in zip(self.fields, self))
        return f"{str(self.__class__.__name__)}({', '.join(fields)})"

    def __str__(self):
        return f"{self[0]}.{self[1]}.{self[2]}"

    major = property(lambda self: self[0])
    minor = property(lambda self: self[1])
    patch = property(lambda self: self[2])


class PygameVersion(SoftwareVersion):
    """Pygame Version class."""


class SDLVersion(SoftwareVersion):
    """SDL Version class."""
"#;

/// Fuellt `rustgame_py.version`.
///
/// Gibt `(vernum, ver)` zurueck, weil beide auch **oben** stehen:
/// `pygame.vernum` und `pygame.ver` gibt es zusaetzlich zu
/// `pygame.version.vernum` - gemessen.
pub fn eintragen<'p>(v: &Bound<'p, PyModule>) -> PyResult<(Py<PyAny>, String)> {
    let py = v.py();
    let quelle = PyModule::from_code(
        py,
        std::ffi::CString::new(VERSION_PY)?.as_c_str(),
        std::ffi::CString::new("version.py")?.as_c_str(),
        std::ffi::CString::new("rustgame_py_version")?.as_c_str(),
    )?;

    let software = quelle.getattr("SoftwareVersion")?;
    let pygame_version = quelle.getattr("PygameVersion")?;
    let sdl_version = quelle.getattr("SDLVersion")?;

    let ver = rustgame_core::PYGAME_VERSION;
    let teile: Vec<u32> = ver
        .split('.')
        .take(3)
        .filter_map(|t| t.parse().ok())
        .collect();
    let vernum = pygame_version.call1((teile[0], teile[1], teile[2]))?;
    // Dieselbe Fassung, die `display.get_driver()` und Freunde melden -
    // nachgemessen an echtem pygame auf dieser Maschine.
    let sdl = sdl_version.call1((2u32, 32u32, 10u32))?;

    v.add("SoftwareVersion", &software)?;
    v.add("PygameVersion", &pygame_version)?;
    v.add("SDLVersion", &sdl_version)?;
    v.add("vernum", &vernum)?;
    v.add("SDL", &sdl)?;
    v.add("ver", ver)?;
    // Siehe den Modulkopf: hier steht bewusst nicht die leere
    // Zeichenkette, die echtes pygame meldet.
    v.add("rev", "rustgame")?;
    v.add_function(wrap_pyfunction!(fassung_get_sdl_version, v)?)?;
    Ok((vernum.unbind(), ver.to_string()))
}

/// `pygame.version.get_sdl_version(linked=True)`.
///
/// Gemessen: `linked=False` gibt dasselbe Tupel - pygame baut SDL fest
/// ein, also fallen "wogegen uebersetzt" und "womit gelaufen" zusammen.
#[pyfunction]
#[pyo3(name = "get_sdl_version", signature = (linked=None))]
pub fn fassung_get_sdl_version(linked: Option<&Bound<'_, PyAny>>) -> PyResult<(u32, u32, u32)> {
    // **Jeder Wert ist ein Wahrheitswert** - pygame liest den Schalter
    // mit `PyObject_IsTrue`, und das nimmt alles. Mit `bool` verlangte
    // PyO3 einen echten Wahrheitswert und nannte im Fehlertext
    // `'PyBool'`, einen Namen aus Rust.
    let linked = match linked {
        None => true,
        Some(o) => o.is_truthy()?,
    };
    let _ = linked;
    Ok((2, 32, 10))
}
