//! farbe.rs
//! ========
//! `pygame.Color`.
//!
//! ## Drei Stellen, an denen ein Nachbau leise falsch wird
//!
//! **1. Eine blanke Zahl ist RGBA, nicht ARGB.** `Color(0x11223344)` ist
//! `(17, 34, 51, 68)` - das Alpha steht **hinten**. Im Pixelpuffer steht
//! es vorn. Wer das verwechselt, bekommt Farben, die fast stimmen.
//!
//! **2. `hsva` und die Rueckwege schneiden ab, `lerp` und
//! `correct_gamma` runden kaufmaennisch.** Gemessen an pygame-ce 2.5.7:
//!
//! ```text
//! Color(100,150,200,250).hsva            -> (210.0, 50.0, 78.43…, 98.03…)
//! Color.from_hsva(*das)                  -> (99, 150, 200, 250)   # 99, nicht 100
//! Color(100,150,200,250).correct_gamma(2.2) -> (33, 79, 149, 244)
//! ```
//!
//! Beides steckt schon im Kern ([`rustgame_core::color`]); diese Datei
//! muss es nur unveraendert durchreichen - und genau das ist die Stelle,
//! an der es verlorengehen koennte.
//!
//! **3. `Color` ist nicht hashbar.** `hash(Color(1,2,3,4))` wirft in
//! pygame `TypeError`, weil die Farbe veraenderlich ist. Ein pyclass ist
//! von Haus aus ueber seine Kennung hashbar - deshalb steht hier ein
//! `__hash__`, das ausdruecklich ablehnt.
//!
//! ## Warum hier so viel Geruest fuer so wenig Rechnung steht
//!
//! Gerechnet wird in dieser Datei fast nichts; der Kern macht das. Was
//! hier steht, ist die **Form**, in der pygame seine Farbe anbietet, und
//! die ist reichhaltiger, als sie aussieht:
//!
//! * `_color_new` legt nur `(0,0,0,255)` an, **`_color_init`** liest die
//!   Argumente. Deshalb geht `Color.__new__(Color)`, deshalb setzt
//!   `c.__init__(1,2,3)` eine bestehende Farbe neu - und deshalb steht
//!   hier ein `__init__` als gewoehnliche Methode
//!   (siehe dort, warum es kein Leerlauf ist).
//! * Jede Rechnung gibt **`type(self)`** zurueck, nicht `Color`. Ein
//!   abgeleiteter Typ bleibt also ueber `+`, `~` und `correct_gamma`
//!   hinweg erhalten - `neu_wie` ist die eine Stelle dafuer.
//! * Die Farbe hat eine einstellbare **Laenge** (`set_length`), die
//!   `len()`, das Durchlaufen, `in` und `c[i]` betrifft, aber weder die
//!   gespeicherten Werte noch das Schneiden.
//! * `r`, `g`, `b`, `a` gibt es auch als **Sammelnamen**: `c.rgb`,
//!   `c.rgba`, `c.ar01` - lesend und schreibend
//!   (`__getattr__`).
//!
//! ## Die Grenze, die hier gefallen ist: `Color()` ohne Argumente
//!
//! pygame trennt `tp_new` (legt an) und `tp_init` (liest Argumente):
//!
//! ```text
//! pygame.Color.__new__(pygame.Color)        Color(0, 0, 0, 255)
//! pygame.Color.__new__(pygame.Color,1,2,3)  Color(0, 0, 0, 255)  - sieht nichts an
//! pygame.Color()                            ValueError: invalid color (…)
//! pygame.Color(unsinn=1)                    dasselbe  - Schluesselwoerter zaehlen nicht
//! pygame.Color(1,2,3,unsinn=1)              Color(1, 2, 3, 255)
//! c.__init__()                              ValueError: invalid color (…)
//! ```
//!
//! Hier stand, PyO3 kenne nur `#[new]` und beide Aufrufe kaemen dort mit
//! einem leeren Beutel an, also sei das nicht zu trennen - der Vorzug
//! gehe an die erste Form, weil pygames `test_new` sie prueft. Der
//! Schluss war falsch, und zwar seit `Surface` denselben Weg gegangen
//! ist: eine PyO3-Klasse ist ein **Heap-Typ**, und
//! `setattr(Klasse, "__init__", f)` setzt dort `tp_init` mit (CPythons
//! `type_setattro` ruft `fixup_slot_dispatchers`). Also liegt das
//! `__init__` jetzt in `python/color.py`, `#[new]` legt weiter die
//! schwarze Farbe an - und beide Zeilen oben stimmen.

use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyAttributeError, PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{
    PyAnyMethods, PyBytes, PyDict, PyInt, PySlice, PyString, PyTuple, PyType,
};

use rustgame_core::color::{self, Color};

/// Der Wortlaut, mit dem pygame eine unbrauchbare Farbfolge ablehnt.
///
/// Steht auch in [`crate::umwandlung`]; hier noch einmal, weil diese
/// Datei die **ganze** Kette aus `pg_RGBAFromObjEx` nachbildet und nicht
/// nur ihren haeufigsten Ausgang.
const FOLGE_KAPUTT: &str = "invalid color (color sequence must have size 3 or 4, \
                            and each element must be an integer in the range [0, 255])";

/// Was `__reduce__` zurueckgibt: Erzeuger und seine vier Argumente.
type Bauplan = (Py<PyAny>, (u8, u8, u8, u8));

/// `pygame.Color`.
///
/// `laenge` ist **nicht** die Zahl der gespeicherten Kanaele - die sind
/// immer vier. Sie ist, was `len()` meldet; `set_length(3)` laesst das
/// Alpha stehen und blendet es nur aus. Gemessen: danach ist `c.a`
/// weiterhin lesbar, `c[3]` aber ein `IndexError`.
#[pyclass(name = "Color", module = "rustgame_py.color", subclass)]
#[derive(Clone, Copy)]
pub struct PyColor {
    wert: Color,
    laenge: u8,
}

impl PyColor {
    /// Aus einer Kernfarbe.
    pub fn neu(c: Color) -> PyColor {
        PyColor {
            wert: c,
            laenge: 4,
        }
    }

    /// Dieselbe Farbe, aber `len()` meldet `laenge`.
    ///
    /// Gebraucht fuer `Surface.get_palette()`, dessen Eintraege in pygame
    /// die Laenge **3** haben - der Alphakanal ist gespeichert, zaehlt
    /// aber nicht mit. Siehe dort.
    pub fn mit_laenge(c: Color, laenge: u8) -> PyColor {
        PyColor { wert: c, laenge }
    }
    /// Die Kernfarbe dahinter.
    pub fn wert(&self) -> Color {
        self.wert
    }
}

// ---------------------------------------------------------------------------
//  Was pygame als Farbe durchgehen laesst
// ---------------------------------------------------------------------------

/// Ein einzelner Kanal - `_get_color_int_component`.
///
/// **Nur ganze Zahlen.** `c.r = 1.5` ist in pygame ein `TypeError`, nicht
/// etwa 1; eine Folge dagegen (`Color((1.5, 2, 3))`) nimmt Gleitkomma an
/// und schneidet ab, weil dort `pg_UintFromObj` mit `PyNumber_Long`
/// arbeitet. Zwei Wege in dieselbe Farbe, zwei Regeln - beide gemessen.
fn kanal(v: &Bound<'_, PyAny>) -> PyResult<u8> {
    if let Ok(i) = v.downcast::<PyInt>() {
        return match i.extract::<u64>() {
            Ok(n) if n <= 255 => Ok(n as u8),
            _ => Err(PyValueError::new_err(
                "invalid color component (must be in range [0, 255])",
            )),
        };
    }
    Err(PyTypeError::new_err(format!(
        "color component must be a number, not '{}'",
        v.get_type().name()?
    )))
}

/// Eine Zahl aus einer Farbfolge - `pg_UintFromObj`.
///
/// `PyNumber_Long`, also **abschneiden** statt runden, und Negatives
/// faellt durch (`PyLong_AsUnsignedLong`).
fn folgen_zahl(o: &Bound<'_, PyAny>) -> Option<u32> {
    if let Ok(n) = o.extract::<u64>() {
        return u32::try_from(n).ok();
    }
    let f = o.extract::<f64>().ok()?;
    if f < 0.0 || f > u32::MAX as f64 {
        return None;
    }
    Some(f as u32)
}

/// `PySequence_Check` - siehe [`crate::vektor`], wo dasselbe steht und
/// warum PyO3s `downcast::<PySequence>()` dafuer nicht reicht.
fn ist_folge(o: &Bound<'_, PyAny>) -> bool {
    !o.is_instance_of::<PyDict>()
        && o.hasattr(pyo3::intern!(o.py(), "__getitem__"))
            .unwrap_or(false)
}

/// [`ist_folge`] samt Laenge.
fn folge_laenge(o: &Bound<'_, PyAny>) -> Option<usize> {
    if !ist_folge(o) {
        return None;
    }
    o.len().ok()
}

/// Eine Folge aus drei oder vier Zahlen - `pg_RGBAFromObj` aus `base.c`.
///
/// Die erste Zeile ist die ueberraschende: ein **Einertupel** wird
/// aufgemacht und sein Inhalt erneut gelesen. Das ist der Grund, warum
/// `Color(((1, 2, 3),))` dieselbe Farbe ergibt wie `Color((1, 2, 3))` -
/// gemessen.
fn folge_zu_farbe(o: &Bound<'_, PyAny>) -> Option<Color> {
    if let Ok(t) = o.downcast::<PyTuple>() {
        if t.len() == 1 {
            return folge_zu_farbe(&t.get_item(0).ok()?);
        }
    }
    let n = folge_laenge(o)?;
    if !(3..=4).contains(&n) {
        return None;
    }
    let mut rgba = [0u8, 0, 0, 255];
    for (i, feld) in rgba.iter_mut().enumerate().take(n) {
        let v = folgen_zahl(&o.get_item(i).ok()?)?;
        if v > 255 {
            return None;
        }
        *feld = v as u8;
    }
    Some(Color::rgba(rgba[0], rgba[1], rgba[2], rgba[3]))
}

/// Alles, was pygame als Farbe annimmt - `pg_RGBAFromObjEx`.
///
/// `alles` steht fuer `PG_COLOR_HANDLE_ALL`: dann zaehlen auch eine
/// blanke Zahl und ein Name. Ohne das gilt `PG_COLOR_HANDLE_RESTRICT_SEQ`,
/// und dann sind **nur** `Color` und ein Tupel erlaubt - eine Liste
/// nicht. Genau daran haengt, dass `Color(1,2,3,4) == (1,2,3,4)` wahr
/// und `== [1,2,3,4]` falsch ist; beides gemessen.
fn farbe_aus_roh(o: &Bound<'_, PyAny>, alles: bool) -> PyResult<Color> {
    if let Ok(c) = o.extract::<PyRef<PyColor>>() {
        return Ok(c.wert);
    }
    if alles {
        if let Ok(i) = o.downcast::<PyInt>() {
            let v = i.extract::<u32>().map_err(|_| {
                PyValueError::new_err("invalid color argument (integer out of acceptable range)")
            })?;
            // pygame liest eine blanke Zahl als **RGBA**, nicht als ARGB.
            return Ok(Color::rgba(
                (v >> 24) as u8,
                (v >> 16) as u8,
                (v >> 8) as u8,
                v as u8,
            ));
        }
        if let Ok(s) = o.downcast::<PyString>() {
            return color::parse(&s.to_cow()?)
                .ok_or_else(|| PyValueError::new_err("invalid color name"));
        }
    } else if o.downcast::<PyTuple>().is_err() {
        // Ein `ValueError`, kein `TypeError` - in `color.c` steht
        // ausdruecklich "for backcompat reasons" daneben.
        return Err(PyValueError::new_err(
            "invalid color (here, generic sequences are restricted, \
             but pygame.Color and RGB[A] tuples are allowed)",
        ));
    }
    if let Some(c) = folge_zu_farbe(o) {
        return Ok(c);
    }
    if ist_folge(o) {
        Err(PyValueError::new_err(FOLGE_KAPUTT))
    } else {
        Err(PyTypeError::new_err(format!(
            "unable to interpret object of type '{}' as a color",
            o.get_type().name()?
        )))
    }
}

/// Die Argumentliste von `Color(...)` und `c.__init__(...)`.
///
/// ```c
/// if (PyTuple_GET_SIZE(args) == 1) {
///     args = PyTuple_GET_ITEM(args, 0);
/// }
/// return pg_RGBAFromObjEx(args, self->data, PG_COLOR_HANDLE_ALL) - 1;
/// ```
///
/// Bei drei oder vier Angaben wird also das **Argumenttupel selbst** als
/// Farbfolge gelesen. Deshalb gibt es hier keinen getrennten Zweig fuer
/// `Color(r, g, b)` - es ist derselbe wie fuer `Color((r, g, b))`.
fn farbe_aus_args(args: &Bound<'_, PyTuple>) -> PyResult<Color> {
    if args.len() == 1 {
        farbe_aus_roh(&args.get_item(0)?, true)
    } else {
        farbe_aus_roh(args.as_any(), true)
    }
}

/// Eine neue Farbe vom **Typ des Empfaengers**.
///
/// ```c
/// return (PyObject *)_color_new_internal(Py_TYPE(obj1), rgba);
/// ```
///
/// pygame legt das Ergebnis jeder Rechnung mit `Py_TYPE(self)->tp_alloc`
/// an, ruft also `__init__` **nicht** auf. Das ist nachpruefbar:
///
/// ```text
/// class MyColor(Color):
///     def __init__(self, *a): super().__init__(*a); self.merk = True
/// (MyColor(1,2,3) + Color(1,1,1)).merk  ->  AttributeError
/// ```
///
/// Der Aufruf von `cls.__new__(cls)` bildet das nach: er geht ueber
/// `tp_new` und laesst `tp_init` aus.
fn neu_wie(slf: &Bound<'_, PyAny>, c: Color) -> PyResult<Py<PyAny>> {
    let cls = slf.get_type();
    let obj = cls.call_method1("__new__", (&cls,))?;
    obj.downcast::<PyColor>()?.borrow_mut().wert = c;
    Ok(obj.unbind())
}

/// Drei oder vier Gleitkommazahlen aus einer Folge holen.
///
/// `hsva`, `hsla`, `cmy`, `i1i2i3` und `normalized` lesen ihre Angaben
/// alle so. Der Wortlaut des Fehlers ist je Farbraum ein anderer
/// (`"invalid HSVA value"`), deshalb kommt er als Beiwert herein.
///
/// `mit_alpha` unterscheidet die beiden Gruppen: `hsva`, `hsla` und
/// `normalized` nehmen drei **oder vier** Zahlen, `cmy` und `i1i2i3`
/// **genau drei** (`PySequence_Size(value) != 3`). Ein vierter Wert ist
/// dort also ein `ValueError`, nicht ein stillschweigend ignoriertes
/// Alpha - abgeschrieben, nicht vereinheitlicht.
fn zahlen_aus(
    o: &Bound<'_, PyAny>,
    wortlaut: &'static str,
    grenzen: &[(f64, f64)],
    vorgabe_alpha: f64,
    mit_alpha: bool,
) -> PyResult<[f64; 4]> {
    let schlecht = || PyValueError::new_err(wortlaut);
    let n = folge_laenge(o).ok_or_else(schlecht)?;
    let erlaubt = if mit_alpha { 3..=4 } else { 3..=3 };
    if !erlaubt.contains(&n) {
        return Err(schlecht());
    }
    let mut aus = [0.0, 0.0, 0.0, vorgabe_alpha];
    for (i, feld) in aus.iter_mut().enumerate().take(n) {
        let v: f64 = o.get_item(i).and_then(|x| x.extract()).map_err(|_| schlecht())?;
        let (u, o) = grenzen[i];
        if v < u || v > o {
            return Err(schlecht());
        }
        *feld = v;
    }
    Ok(aus)
}

/// `0..=360` fuer den Farbton, `0..=100` fuer den Rest.
const HSV_GRENZEN: [(f64, f64); 4] = [(0.0, 360.0), (0.0, 100.0), (0.0, 100.0), (0.0, 100.0)];
/// `0..=1` fuer alle vier - `cmy`, `i1i2i3` und `normalized`.
const EINS_GRENZEN: [(f64, f64); 4] = [(0.0, 1.0), (0.0, 1.0), (0.0, 1.0), (0.0, 1.0)];

/// Welchen Wert ein Zeichen aus `r g b a 0 1` beitraegt.
///
/// `0` und `1` sind feste Zahlen: pygame setzt dafuer 0 und **255**,
/// nicht 0 und 1 - der Kanal ist ja ein Byte.
fn sammelname(z: char, c: Color) -> Option<u8> {
    Some(match z {
        'r' => c.r,
        'g' => c.g,
        'b' => c.b,
        'a' => c.a,
        '0' => 0,
        '1' => 255,
        _ => return None,
    })
}

impl PyColor {
    /// Die vier Bytes dieser Farbe als Puffer-Beschreibung.
    ///
    /// **Nur so viele, wie `len()` sagt.** `Color.set_length(3)` macht aus
    /// der Farbe eine Dreierfolge, und der Puffer wird damit drei Bytes
    /// lang; `color_test.test_arraystruct` prueft es fuer alle vier
    /// Laengen.
    ///
    /// Gemessen ist noch etwas, das man fuer einen Fehler halten wuerde:
    /// der Puffer meldet sich als **nur lesbar** (`imp.readonly` ist wahr,
    /// `PyBUF_WRITABLE` gibt `BufferError`), und trotzdem wirkt ein
    /// Schreiben durch ihn:
    ///
    /// ```python
    /// imp = ColorImporter(c, buftools.PyBUF_SIMPLE)
    /// self.assertTrue(imp.readonly)
    /// imp[0] = 60
    /// self.assertEqual(c.r, 60)
    /// ```
    ///
    /// pygame gibt also die echte Adresse heraus und bittet hoeflich, sie
    /// nicht anzufassen. Das ist nachgebaut, weil beide Zeilen im selben
    /// Test stehen - wer den Puffer beschreibbar meldet, faellt an der
    /// ersten, wer eine Kopie gibt, an der zweiten.
    fn sicht(&self) -> crate::puffer::Sicht {
        crate::puffer::Sicht {
            adresse: &self.wert as *const Color as usize,
            laenge: self.laenge as usize,
            itemsize: 1,
            format: crate::puffer::fmt("B"),
            shape: vec![self.laenge as isize],
            strides: vec![1],
            schreibbar: false,
            solide: true,
        }
    }
}

#[pymethods]
impl PyColor {
    /// `c.__array_struct__` - die vier Bytes als C-Kasten.
    #[getter]
    fn __array_struct__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        crate::puffer::feldschnittstelle(py, &self.sicht())
    }

    /// `memoryview(c)` - dieselben Bytes ueber das Puffer-Protokoll.
    ///
    /// # Sicherheit
    ///
    /// Die Adresse zeigt in das Python-Objekt selbst; `fuellen` haelt es
    /// ueber `view.obj` am Leben, und ein `#[pyclass]` wandert nach dem
    /// Anlegen nicht mehr im Speicher.
    // Reicht nur durch - siehe `#![deny(unsafe_code)]` in `lib.rs`.
    #[allow(unsafe_code)]
    unsafe fn __getbuffer__(
        slf: PyRef<'_, Self>,
        view: *mut pyo3::ffi::Py_buffer,
        flags: std::ffi::c_int,
    ) -> PyResult<()> {
        let sicht = slf.sicht();
        let py = slf.py();
        let eigner = slf.into_pyobject(py)?;
        unsafe { crate::puffer::fuellen(&sicht, view, flags, eigner.as_any()) }
    }

    /// # Sicherheit
    ///
    /// Siehe [`crate::puffer::freigeben`].
    #[allow(unsafe_code)]
    unsafe fn __releasebuffer__(&self, view: *mut pyo3::ffi::Py_buffer) {
        unsafe { crate::puffer::freigeben(view) }
    }

    /// `Color(r, g, b[, a])`, `Color((r,g,b))`, `Color("red")`,
    /// `Color("#rrggbb")` oder `Color(0xrrggbbaa)`.
    ///
    /// Ohne Argumente `Color(0, 0, 0, 255)` - das ist `tp_new`. Den
    /// `ValueError` fuer `Color()` gibt das `__init__` in
    /// `python/color.py`; siehe den Dateikopf.
    #[new]
    #[pyo3(signature = (*args, **_kwargs))]
    fn new(args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        if args.is_empty() {
            return Ok(PyColor::neu(Color::rgba(0, 0, 0, 255)));
        }
        Ok(PyColor::neu(farbe_aus_args(args)?))
    }

    /// `c.__init__(...)` - **setzt die Farbe neu**, kein Leerlauf.
    ///
    /// ```text
    /// c = pygame.Color(1,2,3); c.__init__(9,9,9); tuple(c)  ->  (9, 9, 9, 255)
    /// ```
    ///
    /// Sichtbar wird das beim Ableiten, und pygames Testdateien leiten
    /// reihenweise ab:
    ///
    /// ```python
    /// class MyColor(pygame.Color):
    ///     def __init__(self, *args):
    ///         super().__init__(*args)
    /// ```
    ///
    /// Ohne diese Methode landet `super().__init__` bei
    /// `object.__init__` und der meldet `takes exactly one argument`.
    ///
    /// **Sie heisst `_init`**, weil `__init__` jetzt in Python steht
    /// (`python/color.py`) und hierher weiterreicht - derselbe Aufbau
    /// wie bei `Surface`. Der Unterstrichname taucht in keinem `dir()`
    /// auf und in keiner Fehlermeldung: das Python-`__init__` prueft
    /// selbst, bevor es ruft.
    ///
    /// Die Laenge wird dabei auf 4 zurueckgesetzt (`self->len = 4;` ist
    /// die erste Zeile von `_color_init`) - gemessen.
    #[pyo3(name = "_init")]
    #[pyo3(signature = (*args, **_kwargs))]
    fn eigen_init(
        slf: &Bound<'_, Self>,
        args: &Bound<'_, PyTuple>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let neu = farbe_aus_args(args)?;
        let mut m = slf.borrow_mut();
        m.laenge = 4;
        m.wert = neu;
        Ok(())
    }

    /// Fuer `pickle` und `copy.deepcopy`.
    ///
    /// pygame geht einen anderen Weg: es traegt in `pygame/__init__.py`
    /// einen `copyreg.pickle`-Eintrag auf `pygame.__color_constructor`
    /// ein (nachgesehen mit `pickletools.dis`). Das liesse sich nur in
    /// `lib.rs` nachbauen; `__reduce__` erreicht dasselbe von hier aus
    /// und traegt den **abgeleiteten** Typ gleich mit.
    fn __reduce__(slf: &Bound<'_, Self>) -> PyResult<Bauplan> {
        let c = slf.borrow().wert;
        Ok((slf.get_type().into_any().unbind(), c.as_tuple()))
    }

    #[getter]
    fn r(&self) -> u8 {
        self.wert.r
    }
    #[setter]
    fn set_r(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.wert.r = kanal(v)?;
        Ok(())
    }
    #[getter]
    fn g(&self) -> u8 {
        self.wert.g
    }
    #[setter]
    fn set_g(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.wert.g = kanal(v)?;
        Ok(())
    }
    #[getter]
    fn b(&self) -> u8 {
        self.wert.b
    }
    #[setter]
    fn set_b(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.wert.b = kanal(v)?;
        Ok(())
    }
    #[getter]
    fn a(&self) -> u8 {
        self.wert.a
    }
    #[setter]
    fn set_a(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.wert.a = kanal(v)?;
        Ok(())
    }

    /// `Color.update(...)` - wie der Erzeuger, nur an Ort und Stelle.
    ///
    /// Drei Angaben lassen das **alte** Alpha stehen, eine einzelne
    /// setzt es auf 255:
    ///
    /// ```text
    /// c = Color(9,9,9,9); c.update(1,2,3)    ->  (1, 2, 3, 9)
    /// c = Color(9,9,9,9); c.update((1,2,3))  ->  (1, 2, 3, 255)
    /// ```
    ///
    /// Beides gemessen. Der Unterschied ist keine Nachlaessigkeit: mit
    /// drei Angaben schreibt `_color_update` nur `data[0..2]`, mit einer
    /// geht es durch `pg_RGBAFromObjEx`, und das setzt alle vier.
    #[pyo3(signature = (*args))]
    fn update(slf: &Bound<'_, Self>, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        // `&Bound<Self>` statt `&mut self`: `c.update(c)` liest die Farbe
        // waehrend des Schreibens, und eine offene Leihe waere dann ein
        // `RuntimeError`, den pygame nicht kennt.
        let alt = slf.borrow().wert;
        match args.len() {
            1 => {
                let neu = farbe_aus_roh(&args.get_item(0)?, true)?;
                slf.borrow_mut().wert = neu;
            }
            n @ (3 | 4) => {
                let mut neu = [alt.r, alt.g, alt.b, alt.a];
                for (i, feld) in neu.iter_mut().enumerate().take(n) {
                    *feld = kanal(&args.get_item(i)?)?;
                }
                let mut m = slf.borrow_mut();
                m.wert = Color::rgba(neu[0], neu[1], neu[2], neu[3]);
                // Nur mit Alpha wird die Laenge mitgezogen - "because the
                // previous implementation of this function behaved so",
                // sagt `color.c` dazu.
                if n == 4 {
                    m.laenge = 4;
                }
            }
            _ => {
                return Err(PyTypeError::new_err(
                    "update can take only 1, 3 or 4 arguments",
                ))
            }
        }
        Ok(())
    }

    /// `c.set_length(n)` - veraltet, warnt, und begrenzt `len()`.
    ///
    /// Die Grenzen sind `1..=4`; alles andere ist ein `ValueError`,
    /// **auch** ein zu grosser Wert wie `2**33`, den `PyArg_ParseTuple`
    /// gar nicht erst in ein `int` bekommt (`color.c` faengt den
    /// `OverflowError` ab und setzt `INT_MAX`, damit dieselbe Meldung
    /// herauskommt).
    #[pyo3(signature = (n, /))]
    fn set_length(&mut self, py: Python<'_>, n: &Bound<'_, PyAny>) -> PyResult<()> {
        let warnungen = py.import("warnings")?;
        warnungen.call_method1(
            "warn",
            (
                "pygame.Color.set_length deprecated since 2.1.3",
                py.get_type::<pyo3::exceptions::PyDeprecationWarning>(),
            ),
        )?;
        // **Eine Nicht-Zahl ist ein `TypeError`**, keine stille
        // Ersatzzahl. Hier stand `unwrap_or(i32::MAX)`, und damit gab
        // `set_length("x")` einen `ValueError` ueber den Bereich -
        // pygame gibt `TypeError: 'str' object cannot be interpreted as
        // an integer`, weil sein Argumentleser zuerst dran ist.
        let n = n.extract::<i64>()?;
        if !(1..=4).contains(&n) {
            // **`ValueError` nur fuer eine Zahl ausserhalb 1..4.**
            // Etwas, das gar keine Zahl ist, gibt bei pygame einen
            // `TypeError: 'str' object cannot be interpreted as an
            // integer` - das macht der Argumentleser, bevor die Pruefung
            // laeuft. Siehe `set_length` darunter.
            return Err(PyValueError::new_err("Length needs to be 1,2,3, or 4."));
        }
        self.laenge = n as u8;
        Ok(())
    }

    // ----- Farbraeume ---------------------------------------------------

    /// `(H, S, V, A)` - H in Grad, der Rest in Prozent.
    #[getter]
    fn hsva(&self) -> (f64, f64, f64, f64) {
        self.wert.hsva()
    }
    /// **Ohne viertes Feld wird Alpha 0**, nicht 100 und auch nicht das
    /// bisherige.
    ///
    /// ```text
    /// c = Color(1,2,3,4); c.hsva = (0, 100, 100); tuple(c) -> (255, 0, 0, 0)
    /// ```
    ///
    /// Gemessen. In `color.c` steht `double hsva[4] = {0, 0, 0, 0};` und
    /// das vierte Feld wird nur beschrieben, wenn es auch mitkam; danach
    /// gilt bedingungslos `data[3] = (hsva[3] / 100) * 255`. Hier stand
    /// frueher 100 als Vorgabe - das sah vernuenftig aus und war falsch.
    #[setter]
    fn set_hsva(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        let z = zahlen_aus(v, "invalid HSVA value", &HSV_GRENZEN, 0.0, true)?;
        self.wert = Color::from_hsva(z[0], z[1], z[2], z[3]);
        Ok(())
    }

    /// `(H, S, L, A)`.
    #[getter]
    fn hsla(&self) -> (f64, f64, f64, f64) {
        self.wert.hsla()
    }
    /// Alpha wie bei [`set_hsva`](Self::set_hsva) - ohne viertes Feld 0.
    #[setter]
    fn set_hsla(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        let z = zahlen_aus(v, "invalid HSLA value", &HSV_GRENZEN, 0.0, true)?;
        self.wert = Color::from_hsla(z[0], z[1], z[2], z[3]);
        Ok(())
    }

    /// `(C, M, Y)`, je 0..1.
    #[getter]
    fn cmy(&self) -> (f64, f64, f64) {
        self.wert.cmy()
    }
    /// Hier bleibt das Alpha **stehen** - `_color_set_cmy` schreibt nur
    /// `data[0..2]`. Gemessen: `Color(1,2,3,4).cmy = (0,0,0)` gibt
    /// `(255, 255, 255, 4)`.
    #[setter]
    fn set_cmy(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        let z = zahlen_aus(v, "invalid CMY value", &EINS_GRENZEN, 0.0, false)?;
        self.wert = self.wert.from_cmy(z[0], z[1], z[2]);
        Ok(())
    }

    /// `(I1, I2, I3)` nach Ohta.
    #[getter]
    fn i1i2i3(&self) -> (f64, f64, f64) {
        self.wert.i1i2i3()
    }
    /// Alpha bleibt stehen, wie bei [`set_cmy`](Self::set_cmy).
    #[setter]
    fn set_i1i2i3(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        // I2 und I3 duerfen negativ sein - anders als bei `cmy`.
        let grenzen = [(0.0, 1.0), (-0.5, 0.5), (-0.5, 0.5), (0.0, 1.0)];
        let z = zahlen_aus(v, "invalid I1I2I3 value", &grenzen, 0.0, false)?;
        self.wert = self.wert.from_i1i2i3(z[0], z[1], z[2]);
        Ok(())
    }

    /// `#rrggbbaa` - immer mit Alpha, immer klein geschrieben.
    ///
    /// In pygame ist das ein **Attribut**, keine Methode (`c.hex`, nicht
    /// `c.hex()`); gemessen am Deskriptortyp. Und es ist **schreibbar**,
    /// was hier lange fehlte.
    #[getter]
    fn hex(&self) -> String {
        self.wert.hex()
    }
    #[setter]
    fn set_hex(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        let s = v
            .downcast::<PyString>()
            .map_err(|_| PyTypeError::new_err("hex color must be a string"))?;
        self.wert = Color::from_hex(&s.to_cow()?)
            .ok_or_else(|| PyValueError::new_err("invalid hex string"))?;
        Ok(())
    }

    /// Alle vier Kanaele als 0..1.
    fn normalize(&self) -> (f64, f64, f64, f64) {
        self.wert.normalize()
    }

    /// Dasselbe als Eigenschaft - pygame hat beide Schreibweisen.
    ///
    /// Der Kern hatte `normalized` laengst; erreichbar war es aus Python
    /// trotzdem nicht. Solche Faelle hat die Abdeckungstabelle erst
    /// gefunden, seit sie die **gebaute** Huelle fragt statt den
    /// Quelltext beider Seiten in einen Topf zu werfen.
    #[getter]
    fn normalized(&self) -> (f64, f64, f64, f64) {
        self.wert.normalized()
    }
    /// Ohne viertes Feld gilt hier `1.0`, also Alpha 255 - anders als
    /// bei [`set_hsva`](Self::set_hsva). Gerundet wird **kaufmaennisch**
    /// (`round(f * 255.0)`), nicht abgeschnitten.
    #[setter]
    fn set_normalized(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        let z = zahlen_aus(v, "invalid normalized value", &EINS_GRENZEN, 1.0, true)?;
        self.wert = Color::from_normalized(z[0], z[1], z[2], z[3]);
        Ok(())
    }

    /// Die Farbe mit ihrem Alpha vormultipliziert - `((c + 1) * a) >> 8`.
    fn premul_alpha(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        let c = slf.borrow().wert.premul_alpha();
        neu_wie(slf, c)
    }

    /// Gammakorrektur. Rundet kaufmaennisch, anders als die Farbraeume.
    #[pyo3(signature = (gamma, /))]
    fn correct_gamma(slf: &Bound<'_, Self>, gamma: f64) -> PyResult<Py<PyAny>> {
        let c = slf.borrow().wert.correct_gamma(gamma);
        neu_wie(slf, c)
    }

    /// Mischung zwischen zwei Farben, `t` von 0 bis 1.
    ///
    /// Die Namen `color` und `amount` sind Teil der Schnittstelle -
    /// `_color_lerp` ist als einzige Methode `METH_KEYWORDS`. Und die
    /// Schranke ist nicht scharf: `color.c` laesst `1e-6` Spiel, weil
    /// sich beim Hochzaehlen in Schritten von `0.01` am Ende
    /// `1.0000000000000007` ergibt.
    #[pyo3(signature = (color, amount))]
    fn lerp(
        slf: &Bound<'_, Self>,
        color: &Bound<'_, PyAny>,
        amount: f64,
    ) -> PyResult<Py<PyAny>> {
        const SPIEL: f64 = 1e-6;
        let andere = farbe_aus_roh(color, true)?;
        if amount < -SPIEL || amount > 1.0 + SPIEL {
            return Err(PyValueError::new_err("Argument 2 must be in range [0, 1]"));
        }
        let c = slf.borrow().wert.lerp(andere, amount);
        neu_wie(slf, c)
    }

    /// Die Farbe in Grau - `pygame.Color.grayscale()`.
    fn grayscale(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        let c = slf.borrow().wert.grayscale();
        neu_wie(slf, c)
    }

    /// `Color.from_hex("#1a2b3c")`.
    ///
    /// Die sechs `from_*` gehen in pygame alle durch `_color_from_space`,
    /// und das macht **ein einzelnes Argument auf**: `from_hsva(h,s,v)`
    /// und `from_hsva((h,s,v))` sind derselbe Aufruf. Deshalb steht
    /// ueberall `*args` und nicht die ausgeschriebene Liste.
    ///
    /// Die Farbe entsteht dabei als blankes `Color(0,0,0,255)`, auf das
    /// der jeweilige **Setzer** angewandt wird - also gilt dessen Regel
    /// fuer das Alpha, und der abgeleitete Typ geht verloren (auch in
    /// pygame: `pgColor_New` nimmt den Grundtyp).
    #[classmethod]
    #[pyo3(signature = (*args))]
    fn from_hex(_cls: &Bound<'_, PyType>, args: &Bound<'_, PyTuple>) -> PyResult<PyColor> {
        aus_farbraum(args, |c, v| c.set_hex(v))
    }

    /// `Color.from_hsva(h, s, v[, a])` - ohne `a` ist das Alpha **0**.
    #[classmethod]
    #[pyo3(signature = (*args))]
    fn from_hsva(_cls: &Bound<'_, PyType>, args: &Bound<'_, PyTuple>) -> PyResult<PyColor> {
        aus_farbraum(args, |c, v| c.set_hsva(v))
    }

    /// `Color.from_hsla(h, s, l[, a])` - Alpha wie bei `from_hsva`.
    #[classmethod]
    #[pyo3(signature = (*args))]
    fn from_hsla(_cls: &Bound<'_, PyType>, args: &Bound<'_, PyTuple>) -> PyResult<PyColor> {
        aus_farbraum(args, |c, v| c.set_hsla(v))
    }

    /// `Color.from_normalized(r, g, b[, a])` - alle vier von 0 bis 1.
    #[classmethod]
    #[pyo3(signature = (*args))]
    fn from_normalized(_cls: &Bound<'_, PyType>, args: &Bound<'_, PyTuple>) -> PyResult<PyColor> {
        aus_farbraum(args, |c, v| c.set_normalized(v))
    }

    /// `Color.from_cmy(c, m, y)`.
    ///
    /// Das Alpha ist hier **255**, nicht 0 - weil `_color_set_cmy` es gar
    /// nicht anfasst und die Ausgangsfarbe `Color(0,0,0,255)` ist.
    #[classmethod]
    #[pyo3(signature = (*args))]
    fn from_cmy(_cls: &Bound<'_, PyType>, args: &Bound<'_, PyTuple>) -> PyResult<PyColor> {
        aus_farbraum(args, |c, v| c.set_cmy(v))
    }

    /// `Color.from_i1i2i3(i1, i2, i3)` - Alpha 255, siehe
    /// [`from_cmy`](Self::from_cmy).
    #[classmethod]
    #[pyo3(signature = (*args))]
    fn from_i1i2i3(_cls: &Bound<'_, PyType>, args: &Bound<'_, PyTuple>) -> PyResult<PyColor> {
        aus_farbraum(args, |c, v| c.set_i1i2i3(v))
    }

    // ----- Wie eine Folge -----------------------------------------------

    /// `c[0]` bis `c[3]` und `c[0:2]`.
    ///
    /// Zwei Eigenheiten, beide gemessen und beide aus `_color_subscript`:
    ///
    /// * **Negative Zahlen zaehlen nicht von hinten.** `Color(1,2,3,4)[-1]`
    ///   ist ein `IndexError`, kein `4`. pygame traegt `sq_item` und
    ///   `mp_subscript` ein; `c[-1]` geht ueber `mp_subscript`, und das
    ///   reicht die `-1` unveraendert weiter. Beim **Schneiden**
    ///   (`c[-2:]`) zaehlt Python selbst um, dort geht es also doch.
    /// * Geschnitten wird immer ueber **vier** Felder, unabhaengig von
    ///   `set_length` - in `_color_subscript` steht `int len = 4;` fest.
    fn __getitem__(&self, py: Python<'_>, i: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let alle = [self.wert.r, self.wert.g, self.wert.b, self.wert.a];
        if let Ok(s) = i.downcast::<PySlice>() {
            let teil = s.indices(4)?;
            if teil.slicelength == 0 {
                return Ok(PyTuple::empty(py).into_any().unbind());
            }
            if teil.step != 1 {
                return Err(PyTypeError::new_err("slice steps not supported"));
            }
            let von = teil.start.max(0) as usize;
            let bis = (teil.stop.min(4) as usize).max(von);
            return Ok(PyTuple::new(py, &alle[von..bis])?.into_any().unbind());
        }
        let Ok(k) = i.extract::<isize>() else {
            return Err(PyTypeError::new_err(format!(
                "Color indices must be integers, not {}",
                i.get_type().name()?
            )));
        };
        match k {
            0..=3 if k < self.laenge as isize => {
                Ok(alle[k as usize].into_pyobject(py)?.into_any().unbind())
            }
            _ => Err(PyIndexError::new_err("invalid index")),
        }
    }

    /// `c[i] = v` und `c[a:b] = folge`.
    ///
    /// Beim Schneiden gilt hier die **eingestellte** Laenge, beim Lesen
    /// dagegen fest 4 - `_color_set_slice` uebergibt `color->len` an
    /// `PySlice_GetIndicesEx`, `_color_subscript` nicht. Zwei Nachbarn,
    /// zwei Zahlen; abgeschrieben, nicht vereinheitlicht.
    fn __setitem__(
        slf: &Bound<'_, Self>,
        i: &Bound<'_, PyAny>,
        v: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let alt = slf.borrow().wert;
        if let Ok(s) = i.downcast::<PySlice>() {
            let teil = s.indices(slf.borrow().laenge as isize)?;
            let (start, step, n) = (teil.start, teil.step, teil.slicelength);
            // `_color_set_slice` biegt hier `stop` zurecht ("Make sure
            // s[5:2] = [..] inserts at the right place"). Danach liest es
            // `stop` nicht mehr - die Schleife laeuft ueber `slicelength`.
            // Die Zeile steht deshalb hier nicht.
            let m = folge_laenge(v)
                .ok_or_else(|| PyTypeError::new_err("expected sequence"))?;
            if m != n {
                return Err(PyValueError::new_err(format!(
                    "attempting to assign sequence of length {m} to slice of length {n}"
                )));
            }
            let mut alle = [alt.r, alt.g, alt.b, alt.a];
            for k in 0..n {
                let e = v.get_item(k)?;
                let z = e
                    .downcast::<PyInt>()
                    .map_err(|_| PyTypeError::new_err("color components must be integers"))?
                    .extract::<i64>()?;
                if !(0..=255).contains(&z) {
                    return Err(PyValueError::new_err("color component must be 0-255"));
                }
                alle[(start + k as isize * step) as usize] = z as u8;
            }
            slf.borrow_mut().wert = Color::rgba(alle[0], alle[1], alle[2], alle[3]);
            return Ok(());
        }
        let k = i
            .downcast::<PyInt>()
            .map_err(|_| PyIndexError::new_err("Index must be an integer or slice"))?
            .extract::<isize>()?;
        if !(0..=3).contains(&k) {
            return Err(PyIndexError::new_err("invalid index"));
        }
        let z = kanal(v)?;
        let mut m = slf.borrow_mut();
        match k {
            0 => m.wert.r = z,
            1 => m.wert.g = z,
            2 => m.wert.b = z,
            _ => m.wert.a = z,
        }
        Ok(())
    }

    /// pygame kennt kein `del c[0]` - woertlich uebernommene Meldung.
    fn __delitem__(&self, _i: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err(
            "Color object doesn't support item deletion",
        ))
    }

    fn __len__(&self) -> usize {
        self.laenge as usize
    }

    /// `x in c` - **nur ganze Zahlen**, alles andere ist ein `TypeError`.
    fn __contains__(&self, v: &Bound<'_, PyAny>) -> PyResult<bool> {
        let z = v
            .downcast::<PyInt>()
            .map_err(|_| {
                PyTypeError::new_err("'in <pygame.Color>' requires integer object")
            })?
            .extract::<i64>()?;
        let alle = [self.wert.r, self.wert.g, self.wert.b, self.wert.a];
        Ok(alle[..self.laenge as usize].iter().any(|&x| x as i64 == z))
    }

    /// Ohne das hier geht `list(c)` nicht: PyO3 traegt `__getitem__` nur
    /// als `mp_subscript` ein, und der alte Folgen-Umweg von `iter()`
    /// greift auf `sq_item` zu.
    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let alle = [self.wert.r, self.wert.g, self.wert.b, self.wert.a];
        let t = PyTuple::new(py, &alle[..self.laenge as usize])?;
        Ok(t.into_any().try_iter()?.into_any().unbind())
    }

    // ----- Sammelnamen ---------------------------------------------------

    /// `c.rgb`, `c.bgra`, `c.rr01` - jede Mischung aus `r g b a 0 1`.
    ///
    /// Vier Zeichen geben eine `Color`, alles andere ein Tupel - auch
    /// zwei und drei, anders als bei `Vector2`/`Vector3`, wo zwei und
    /// drei Zeichen einen Vektor ergeben. Gemessen:
    ///
    /// ```text
    /// Color(1,2,3,4).rg     ->  (1, 2)
    /// Color(1,2,3,4).rgba   ->  Color(1, 2, 3, 4)
    /// Color(1,2,3,4).rgbar  ->  (1, 2, 3, 4, 1)
    /// ```
    ///
    /// PyO3 ruft `__getattr__` erst, wenn die gewoehnliche Suche
    /// gescheitert ist - `c.r` kommt also gar nicht hier an. pygame
    /// fragt umgekehrt zuerst den Sammelnamen und faellt zurueck; fuer
    /// den Ausgang macht das keinen Unterschied, weil ein Name aus einem
    /// Zeichen dort ausdruecklich uebersprungen wird.
    fn __getattr__(&self, py: Python<'_>, name: &str) -> PyResult<Py<PyAny>> {
        let fehlt = || {
            PyAttributeError::new_err(format!(
                "'rustgame_py.color.Color' object has no attribute '{name}'"
            ))
        };
        let zeichen: Vec<char> = name.chars().collect();
        if zeichen.len() < 2 || !matches!(zeichen[0], 'r' | 'g' | 'b' | 'a') {
            return Err(fehlt());
        }
        let mut werte = Vec::with_capacity(zeichen.len());
        for z in &zeichen {
            werte.push(sammelname(*z, self.wert).ok_or_else(fehlt)?);
        }
        if werte.len() == 4 {
            let c = Color::rgba(werte[0], werte[1], werte[2], werte[3]);
            return Ok(PyColor::neu(c).into_pyobject(py)?.into_any().unbind());
        }
        Ok(PyTuple::new(py, werte)?.into_any().unbind())
    }

    /// `c.rg = (9, 8)` - dieselben Namen, schreibend.
    ///
    /// Anders als beim Lesen gehen hier **nur** `r g b a`; `0` und `1`
    /// haben keine Entsprechung. Ein Zeichen zweimal ist ein
    /// `AttributeError` ("conflicts with swizzling") - gemessen.
    ///
    /// Der Rueckfall auf die gewoehnliche Zuweisung geht ueber
    /// `object.__setattr__`, weil PyO3 bei `__setattr__` - anders als bei
    /// `__getattr__` - **nicht** von sich aus zuerst den Normalweg geht.
    /// Ohne ihn liesse sich weder `c.r = 5` setzen noch in einer
    /// abgeleiteten Klasse ein eigenes Feld.
    fn __setattr__(
        slf: &Bound<'_, Self>,
        name: &str,
        wert: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let zeichen: Vec<char> = name.chars().collect();
        let stellen: Option<Vec<usize>> = if zeichen.len() < 2 {
            None
        } else {
            zeichen
                .iter()
                .map(|z| match z {
                    'r' => Some(0),
                    'g' => Some(1),
                    'b' => Some(2),
                    'a' => Some(3),
                    _ => None,
                })
                .collect()
        };
        let Some(stellen) = stellen else {
            return gewoehnlich_setzen(slf.as_any(), name, Some(wert));
        };
        let mut gesetzt = [false; 4];
        let mut neu = {
            let c = slf.borrow().wert;
            [c.r, c.g, c.b, c.a]
        };
        for (i, &stelle) in stellen.iter().enumerate() {
            if gesetzt[stelle] {
                return Err(PyAttributeError::new_err(
                    "Attribute assignment conflicts with swizzling",
                ));
            }
            gesetzt[stelle] = true;
            let e = wert.get_item(i).map_err(|_| {
                PyTypeError::new_err("A sequence of the corresponding elements is expected")
            })?;
            let z = e.extract::<i64>()?;
            if !(0..=255).contains(&z) {
                return Err(PyTypeError::new_err(
                    "Color element is outside of the range from 0 to 255",
                ));
            }
            neu[stelle] = z as u8;
        }
        slf.borrow_mut().wert = Color::rgba(neu[0], neu[1], neu[2], neu[3]);
        Ok(())
    }

    /// `del c.r` - pygame lehnt jedes Feld einzeln ab.
    fn __delattr__(slf: &Bound<'_, Self>, name: &str) -> PyResult<()> {
        if matches!(
            name,
            "r" | "g" | "b" | "a" | "hsva" | "hsla" | "cmy" | "i1i2i3" | "normalized" | "hex"
        ) {
            return Err(PyAttributeError::new_err(format!(
                "Cannot delete attribute {name}"
            )));
        }
        gewoehnlich_setzen(slf.as_any(), name, None)
    }

    // ----- Rechnen -------------------------------------------------------

    fn __add__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        rechne(slf, other, |a, b| a.saturating_add(b))
    }
    fn __sub__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        rechne(slf, other, |a, b| a.saturating_sub(b))
    }
    fn __mul__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        rechne(slf, other, |a, b| (a as u32 * b as u32).min(255) as u8)
    }
    /// Teilen durch einen Kanal 0 gibt in pygame **0** in diesem Kanal,
    /// keine Ausnahme - gemessen.
    fn __floordiv__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        rechne(slf, other, |a, b| a.checked_div(b).unwrap_or(0))
    }
    fn __mod__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        rechne(slf, other, |a, b| a.checked_rem(b).unwrap_or(0))
    }
    fn __invert__(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        let c = slf.borrow().wert;
        neu_wie(
            slf,
            Color::rgba(255 - c.r, 255 - c.g, 255 - c.b, 255 - c.a),
        )
    }

    /// `int(c)` - **RGBA**, also `r<<24 | g<<16 | b<<8 | a`. Gemessen:
    /// `int(Color(100,150,200,250))` ist 1687603450 = `0x6496c8fa`.
    fn __int__(&self) -> u32 {
        self.gepackt()
    }
    /// Dasselbe als Gleitkomma - `nb_float` zeigt in `color.c` auf
    /// dieselbe Rechnung wie `nb_int`.
    fn __float__(&self) -> f64 {
        self.gepackt() as f64
    }
    /// `nb_index`, ebenfalls dieselbe Rechnung. Ohne das gaeben `hex(c)`
    /// und `oct(c)` einen `TypeError` - pygame liefert dort
    /// `hex(int(c))`.
    fn __index__(&self) -> u32 {
        self.gepackt()
    }
    /// `bytes(c)` - immer vier Bytes, auch nach `set_length`.
    fn __bytes__(&self, py: Python<'_>) -> Py<PyAny> {
        let c = self.wert;
        PyBytes::new(py, &[c.r, c.g, c.b, c.a]).into_any().unbind()
    }

    /// Nur `==` und `!=`; alles andere gibt `NotImplemented` und damit
    /// Pythons eigenen `TypeError`.
    ///
    /// Verglichen wird mit `PG_COLOR_HANDLE_RESTRICT_SEQ` - siehe
    /// [`farbe_aus_roh`]. Deshalb ist `Color(1,2,3,4) == (1,2,3,4)` wahr
    /// und `== [1,2,3,4]` falsch, obwohl der Erzeuger beide Formen nimmt.
    fn __richcmp__(&self, py: Python<'_>, other: &Bound<'_, PyAny>, op: CompareOp) -> Py<PyAny> {
        let gleich = match farbe_aus_roh(other, false) {
            Ok(c) => c == self.wert,
            Err(_) => return py.NotImplemented(),
        };
        let ja = |b: bool| b.into_pyobject(py).unwrap().to_owned().into_any().unbind();
        match op {
            CompareOp::Eq => ja(gleich),
            CompareOp::Ne => ja(!gleich),
            _ => py.NotImplemented(),
        }
    }

    /// pygame lehnt das ab, weil eine Farbe veraenderlich ist.
    fn __hash__(&self) -> PyResult<u64> {
        Err(PyTypeError::new_err("unhashable type: 'pygame.color.Color'"))
    }

    fn __repr__(&self) -> String {
        let c = self.wert;
        format!("Color({}, {}, {}, {})", c.r, c.g, c.b, c.a)
    }
}

impl PyColor {
    /// `r<<24 | g<<16 | b<<8 | a` - dreimal gebraucht, einmal
    /// aufgeschrieben.
    fn gepackt(&self) -> u32 {
        let c = self.wert;
        (c.r as u32) << 24 | (c.g as u32) << 16 | (c.b as u32) << 8 | c.a as u32
    }
}

/// Die gewoehnliche Zuweisung, an der [`PyColor::__setattr__`] vorbei.
///
/// `object.__setattr__` ist genau das, was pygame mit
/// `PyObject_GenericSetAttr` aufruft. Der Umweg ueber Python spart ein
/// `unsafe` und kostet nur einen Aufruf, der pro Zuweisung einmal
/// vorkommt.
fn gewoehnlich_setzen(
    obj: &Bound<'_, PyAny>,
    name: &str,
    wert: Option<&Bound<'_, PyAny>>,
) -> PyResult<()> {
    let py = obj.py();
    let grund = py.get_type::<PyAny>();
    match wert {
        Some(w) => grund.call_method1("__setattr__", (obj, name, w))?,
        None => grund.call_method1("__delattr__", (obj, name))?,
    };
    Ok(())
}

/// Kanalweise rechnen, wie pygame es tut - gesaettigt, nie ueberlaufend.
///
/// **Nur `Color` gegen `Color`.** `Color(1,2,3,4) + (1,1,1,1)` ist in
/// pygame ein `TypeError`, kein Rechenergebnis - `_color_add` gibt fuer
/// alles andere `NotImplemented` zurueck. Hier stand einmal ein Aufruf,
/// der jede Farbangabe annahm; das war bequemer und falsch.
fn rechne(
    slf: &Bound<'_, PyColor>,
    other: &Bound<'_, PyAny>,
    f: impl Fn(u8, u8) -> u8,
) -> PyResult<Py<PyAny>> {
    let py = slf.py();
    let Ok(b) = other.extract::<PyRef<PyColor>>() else {
        return Ok(py.NotImplemented());
    };
    let a = slf.borrow().wert;
    let b = b.wert;
    let c = Color::rgba(
        f(a.r, b.r),
        f(a.g, b.g),
        f(a.b, b.b),
        f(a.a, b.a),
    );
    neu_wie(slf, c)
}

/// Der gemeinsame Rumpf der sechs `from_*` - `_color_from_space`.
fn aus_farbraum(
    args: &Bound<'_, PyTuple>,
    setzen: impl Fn(&mut PyColor, &Bound<'_, PyAny>) -> PyResult<()>,
) -> PyResult<PyColor> {
    let wert = if args.len() == 1 {
        args.get_item(0)?
    } else {
        args.as_any().clone()
    };
    let mut c = PyColor::neu(Color::rgba(0, 0, 0, 255));
    setzen(&mut c, &wert)?;
    Ok(c)
}

/// Die Namenstabelle als Python-`dict` - `pygame.colordict.THECOLORS`.
///
/// Gemessen: die Werte sind **Tupel** aus vier Zahlen, keine `Color`,
/// und `pygame.color.THECOLORS is pygame.colordict.THECOLORS` ist wahr -
/// es ist ein einziges Objekt, das an zwei Namen haengt. Diese Funktion
/// baut es einmal; wer es zweimal eintraegt, soll dasselbe Ergebnis
/// eintragen, nicht zweimal aufrufen.
pub fn thecolors(py: Python<'_>) -> PyResult<Py<PyDict>> {
    let d = PyDict::new(py);
    for (name, (r, g, b)) in color::alle_namen() {
        d.set_item(name, (*r, *g, *b, 255u8))?;
    }
    Ok(d.unbind())
}
