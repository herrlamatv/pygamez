//! zeiger.rs
//! =========
//! `pygame.cursors.Cursor` und die drei Zeiger-Aufrufe von
//! `pygame.mouse`.
//!
//! ## Warum das eine eigene Datei ist
//!
//! `mouse.get_cursor()` gibt kein Tupel zurueck, sondern ein
//! **`pygame.cursors.Cursor`-Objekt** - gemessen. Ohne diese Klasse
//! liessen sich `get_cursor`, `set_cursor` und `set_system_cursor` gar
//! nicht wahrheitsgemaess durchreichen; ein Tupel waere eine Antwort,
//! die fast stimmt und deshalb nicht auffaellt.
//!
//! ## Die drei Bauformen, alle gemessen
//!
//! | Aufruf | `type` | `data` |
//! |---|---|---|
//! | `Cursor()` | `system` | `(0,)` - also `SYSTEM_CURSOR_ARROW` |
//! | `Cursor(11)` | `system` | `(11,)` |
//! | `Cursor(size, hotspot, xormasks, andmasks)` | `bitmap` | vier Tupel |
//! | `Cursor(hotspot, surface)` | `color` | `(hotspot, surface)` |
//! | `Cursor(anderer_zeiger)` | wie dieser | dessen Daten |
//!
//! Ein falsch geformter Aufruf gibt
//! `TypeError: Arguments must match a cursor specification`.
//!
//! ## Eine bewusste Abweichung
//!
//! Bei `color` gibt pygames `data[1]` **dasselbe** Flaechenobjekt zurueck,
//! das hineingereicht wurde (`f.data[1] is s` ist wahr). Hier kommt eine
//! Flaeche um denselben geteilten Pixelpuffer heraus - `is` ist also
//! falsch, `==` und jede Pixelabfrage stimmen. Der Grund ist die Bauart:
//! der Kern haelt `Rc<Surface>`, nicht das Python-Objekt. Das im Zeiger
//! mitzufuehren waere moeglich, brauchte aber einen `Py<PySurface>` im
//! Kern - und der Kern soll ohne Python uebersetzbar bleiben.

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyModule, PyTuple};

use rustgame_display::mouse::{self, Cursor};

use crate::flaeche::PySurface;

/// `pygame.cursors.Cursor`.
///
/// ## Warum `dict`
///
/// pygames `Cursor` ist eine gewoehnliche Python-Klasse, und `type` und
/// `data` liegen als Felder in ihrem `__dict__`. Daraus folgt dreierlei,
/// das eine PyO3-Klasse von selbst nicht hat und das gemessen ist:
///
/// ```text
/// hash(c)                     pygame: geht     hier vorher: TypeError
/// {c: 1}, set([c])            pygame: geht     hier vorher: TypeError
/// copy.deepcopy(c)            pygame: geht     hier vorher: TypeError
/// pickle.dumps(c)             pygame: geht     hier vorher: TypeError
/// c.eigenes_feld = 1          pygame: geht     hier vorher: AttributeError
/// ```
///
/// Gefunden hat das die Probe `Objekte`, die fragt, was man mit einem
/// Objekt tun kann **ausser seine Methoden zu rufen** - acht Runden
/// hatten nur die Methoden gemessen.
#[pyclass(
    name = "Cursor",
    module = "rustgame_py.cursors",
    unsendable,
    subclass,
    dict
)]
#[derive(Clone)]
pub struct PyCursor(pub(crate) Cursor);

impl PyCursor {
    /// Aus einer Kernform.
    pub fn neu(c: Cursor) -> PyCursor {
        PyCursor(c)
    }
}

/// Die eine Fehlermeldung, die pygame fuer jede falsche Form hat.
fn falsche_form() -> PyErr {
    PyTypeError::new_err("Arguments must match a cursor specification")
}

#[pymethods]
impl PyCursor {
    /// `Cursor()`, `Cursor(konstante)`, `Cursor(anderer)`,
    /// `Cursor(hotspot, flaeche)` oder
    /// `Cursor(groesse, hotspot, xormasks, andmasks)`.
    #[new]
    #[pyo3(signature = (*args))]
    fn new(args: &Bound<'_, PyTuple>) -> PyResult<PyCursor> {
        match args.len() {
            // Gemessen: ohne Angabe kommt SYSTEM_CURSOR_ARROW, also 0.
            0 => Ok(PyCursor(Cursor::System(0))),
            1 => {
                let a = args.get_item(0)?;
                if let Ok(c) = a.extract::<PyRef<PyCursor>>() {
                    return Ok(PyCursor(c.0.clone()));
                }
                // **Als `i64` lesen, nicht als `u32`.** Gemessen gibt
                // `Cursor(-1)` in pygame
                // `TypeError: Arguments must match a cursor specification`
                // - dieselbe Meldung wie fuer jede andere falsche Form.
                // Ein `u32`-Beiwert haette daraus einen `OverflowError`
                // des Argumentlesers gemacht, und `cursors_test.py` ist
                // genau daran gescheitert.
                //
                // Dasselbe gilt fuer eine Zahl, die keine Systemform
                // ist: `Cursor(99)` gibt denselben `TypeError`.
                let nummer = a.extract::<i64>().map_err(|_| falsche_form())?;
                if nummer < 0 || system_name(nummer as u32).is_none() {
                    return Err(falsche_form());
                }
                Ok(PyCursor(Cursor::System(nummer as u32)))
            }
            2 => {
                let hotspot: (i32, i32) = args.get_item(0)?.extract()?;
                let flaeche: PyRef<PySurface> = args.get_item(1)?.extract()?;
                let bild = flaeche.leihen()?.clone();
                Ok(PyCursor(Cursor::farbe(hotspot, bild)))
            }
            4 => {
                let groesse: (u32, u32) = args.get_item(0)?.extract()?;
                let hotspot: (i32, i32) = args.get_item(1)?.extract()?;
                let xormasks: Vec<u8> = args.get_item(2)?.extract()?;
                let andmasks: Vec<u8> = args.get_item(3)?.extract()?;
                Ok(PyCursor(Cursor::Bitmap {
                    groesse,
                    hotspot,
                    xormasks,
                    andmasks,
                }))
            }
            _ => Err(falsche_form()),
        }
    }

    /// `"system"`, `"bitmap"` oder `"color"`.
    #[getter]
    fn r#type(&self) -> &'static str {
        self.0.art()
    }

    /// Die Bauteile als Tupel - je nach Art unterschiedlich lang.
    #[getter]
    fn data(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let t = match &self.0 {
            Cursor::System(k) => PyTuple::new(py, [*k])?.into_any(),
            Cursor::Bitmap {
                groesse,
                hotspot,
                xormasks,
                andmasks,
            } => PyTuple::new(
                py,
                [
                    groesse.into_pyobject(py)?.into_any().unbind(),
                    hotspot.into_pyobject(py)?.into_any().unbind(),
                    PyTuple::new(py, xormasks)?.into_any().unbind(),
                    PyTuple::new(py, andmasks)?.into_any().unbind(),
                ],
            )?
            .into_any(),
            Cursor::Farbe { hotspot, bild } => PyTuple::new(
                py,
                [
                    hotspot.into_pyobject(py)?.into_any().unbind(),
                    PySurface::neu((**bild).clone())
                        .into_pyobject(py)?
                        .into_any()
                        .unbind(),
                ],
            )?
            .into_any(),
        };
        Ok(t.unbind())
    }

    fn copy(&self) -> PyCursor {
        self.clone()
    }
    fn __copy__(&self) -> PyCursor {
        self.clone()
    }

    /// Gleiche Zeiger haben denselben Streuwert.
    ///
    /// Gemessen: `hash(Cursor(SYSTEM_CURSOR_ARROW))` geht bei pygame und
    /// ist fuer zwei gleiche Zeiger dieselbe Zahl - damit laesst sich ein
    /// Zeiger als Woerterbuchschluessel und in einer Menge benutzen.
    /// Eine PyO3-Klasse mit `__eq__` ist von selbst **unhashbar**, genau
    /// wie eine Python-Klasse mit `__eq__` und ohne `__hash__`.
    fn __hash__(&self, py: Python<'_>) -> PyResult<isize> {
        use pyo3::types::PyAnyMethods;
        let t = PyTuple::new(
            py,
            [
                self.r#type().into_pyobject(py)?.into_any().unbind(),
                self.data(py)?,
            ],
        )?;
        t.hash()
    }

    /// `copy.deepcopy` und `pickle` - **beide ueber denselben Weg**.
    ///
    /// `__reduce__` sagt Python, wie das Objekt wieder zu bauen ist: der
    /// Erzeuger und die Argumente, die er bekommen hat. `copy.deepcopy`
    /// und `pickle.dumps` benutzen beide genau das, also genuegt die eine
    /// Methode fuer alle drei Faelle.
    fn __reduce__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let klasse = py.get_type::<PyCursor>();
        let args = self.data(py)?;
        Ok(PyTuple::new(py, [klasse.into_any().unbind(), args])?
            .into_any()
            .unbind())
    }

    // **Kein `__deepcopy__`.** Hier stand eines, obwohl `__reduce__` den
    // Fall schon abdeckt (siehe dort) - ein Name, den pygames `Cursor`
    // nicht hat. Gefunden hat es die Probe `Klassendoppel`, nachdem ihre
    // Grenze einen Abdruck bekam: der Vermerk sprach nur von
    // `__release_buffer__`.

    fn __eq__(&self, other: &Bound<'_, PyAny>) -> bool {
        match other.extract::<PyRef<PyCursor>>() {
            Ok(o) => self.0 == o.0,
            Err(_) => false,
        }
    }

    /// `len(c)` - so lang wie [`Self::data`].
    ///
    /// ## Ein Zeiger ist eine **Folge**
    ///
    /// Hier stand nur ein `__bool__`, das immer `true` gab, mit dem
    /// Vermerk, `Cursor(0)` duerfe nicht `False` sein. Der Vermerk war
    /// richtig und die Loesung war die falsche: gemessen hat pygames
    /// `Cursor` gar kein `__bool__`, sondern **`__len__`,
    /// `__getitem__` und `__iter__**` - und weil die Laenge nie null ist,
    /// kommt `bool()` von selbst richtig heraus.
    ///
    /// ```text
    /// Cursor(SYSTEM_CURSOR_ARROW)   len 1   [0] -> 0
    /// Cursor((8,8),(0,0),xor,and)   len 4   vier Tupel
    /// Cursor((0,0), flaeche)        len 2   Tupel und Flaeche
    /// c[1]                          IndexError: tuple index out of range
    /// c["x"]                        TypeError: tuple indices must be ...
    /// type(iter(c))                 tuple_iterator
    /// 0 in c                        True
    /// ```
    ///
    /// Gefunden hat das die Probe `Klassendoppel`: drei fehlende Namen und
    /// einer zuviel. Und die Umsetzung ist pygames eigene - es reicht an
    /// sein Datentupel durch, darum stimmen auch die Fehlertexte, ohne
    /// dass einer davon hier steht.
    fn __len__(&self, py: Python<'_>) -> PyResult<usize> {
        self.data(py)?.bind(py).len()
    }

    /// `c[i]` - durchgereicht an das Datentupel, samt seiner Fehler.
    fn __getitem__(&self, py: Python<'_>, i: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        Ok(self.data(py)?.bind(py).get_item(i)?.unbind())
    }

    /// `iter(c)` - der Iterator des Datentupels, nicht ein eigener.
    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(self.data(py)?.bind(py).try_iter()?.into_any().unbind())
    }

    fn __repr__(&self) -> String {
        match &self.0 {
            Cursor::System(k) => format!(
                "<Cursor(type: system, constant: {})>",
                system_name(*k).unwrap_or("SYSTEM_CURSOR_ARROW")
            ),
            Cursor::Bitmap {
                groesse, hotspot, ..
            } => format!(
                "<Cursor(type: bitmap, size: ({}, {}), hotspot: ({}, {}))>",
                groesse.0, groesse.1, hotspot.0, hotspot.1
            ),
            Cursor::Farbe { hotspot, bild } => format!(
                "<Cursor(type: color, hotspot: ({}, {}), surf: <Surface({}x{}x32, global_alpha=255)>)>",
                hotspot.0,
                hotspot.1,
                bild.get_width(),
                bild.get_height()
            ),
        }
    }
}

/// Der Name einer Systemform - fuer `__repr__`.
///
/// pygame schreibt in die Darstellung den **Namen** der Konstante, nicht
/// ihre Zahl. Die Zuordnung steht in `SDL_SystemCursor`; sie wird hier
/// aus der Konstantenliste geholt, damit es keine zweite Abschrift gibt.
fn system_name(wert: u32) -> Option<&'static str> {
    crate::konstanten_liste::ZAHLEN
        .iter()
        .find(|(n, w)| *w == wert && n.starts_with("SYSTEM_CURSOR_"))
        .map(|(n, _)| *n)
}

// ---------------------------------------------------------------------------
//  Die drei Aufrufe aus `pygame.mouse`
// ---------------------------------------------------------------------------

/// Fuellt `rustgame_py.cursors` mit dem, was aus pygames eigener
/// `cursors.py` uebernommen ist: die sechs fertigen Zeiger, die fuenf
/// Zeichenkettenformen, `compile` und `load_xbm`.
///
/// `Cursor` wird **hereingereicht**, nicht dort definiert - sonst waere
/// `cursors.arrow` von einem anderen Typ als `mouse.get_cursor()`, und
/// jedes `isinstance` ginge daneben.
pub fn daten_eintragen(c: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = c.py();
    let raum = pyo3::types::PyDict::new(py);
    raum.set_item("Cursor", py.get_type::<PyCursor>())?;
    py.run(
        &std::ffi::CString::new(crate::zeiger_daten::ZEIGER_PY)?,
        Some(&raum),
        None,
    )?;
    for (schluessel, wert) in raum.iter() {
        let name: String = schluessel.extract()?;
        if name.starts_with("__") || name == "Cursor" {
            continue;
        }
        c.add(name.as_str(), wert)?;
    }
    Ok(())
}

/// `pygame.mouse.get_cursor()`.
#[pyfunction]
#[pyo3(name = "get_cursor")]
pub fn maus_get_cursor() -> PyResult<PyCursor> {
    mouse::get_cursor()
        .map(PyCursor::neu)
        .map_err(crate::umwandlung::anzeige_fehler)
}

/// `pygame.mouse.set_cursor(...)`.
///
/// Nimmt dieselben Formen wie `PyCursor::new` - `set_cursor(konstante)`,
/// `set_cursor(zeiger)`, `set_cursor(hotspot, flaeche)` und die
/// vierteilige Bitmap-Form.
#[pyfunction]
#[pyo3(signature = (*args))]
#[pyo3(name = "set_cursor")]
pub fn maus_set_cursor(args: &Bound<'_, PyTuple>) -> PyResult<()> {
    let z = PyCursor::new(args)?;
    mouse::set_cursor(z.0).map_err(crate::umwandlung::anzeige_fehler)
}

/// `pygame.mouse.set_system_cursor(konstante)`.
///
/// In pygame seit 2.2 veraltet: `set_cursor` kann dasselbe. Die Warnung
/// ist gemessen und hier woertlich uebernommen.
#[pyfunction]
#[pyo3(name = "set_system_cursor")]
#[pyo3(signature = (cursor, /))]
pub fn maus_set_system_cursor(py: Python<'_>, cursor: u32) -> PyResult<()> {
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let konstante = cursor;
    let warnungen = py.import("warnings")?;
    let klasse = py.get_type::<pyo3::exceptions::PyDeprecationWarning>();
    warnungen.call_method1(
        "warn",
        (
            "set_cursor() now has all the functionality of \
             set_system_cursor(), so set_system_cursor() will be \
             deprecated in pygame 2.2",
            klasse,
        ),
    )?;
    mouse::set_system_cursor(konstante).map_err(crate::umwandlung::anzeige_fehler)
}
