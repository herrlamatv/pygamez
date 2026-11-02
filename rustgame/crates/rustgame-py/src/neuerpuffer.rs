//! neuerpuffer.rs
//! ==============
//! `pygame.newbuffer` - das Puffer-Protokoll **von Python aus**.
//!
//! Zwei Namen, und sie sind das Gegenstueck zueinander:
//!
//! * [`PyPyBuffer`] (`Py_buffer`) ist eine Huelle um CPythons
//!   `Py_buffer`-Struktur. Sie kann sich einen Puffer von einem fremden
//!   Objekt **holen** (`get_buffer`) und ihn wieder hergeben
//!   (`release_buffer`).
//! * [`PyBufferMixin`] (`BufferMixin`) ist eine Basisklasse: wer von ihr
//!   erbt und `_get_buffer(view, flags)` schreibt, wird selbst zum
//!   Puffer-Ausgeber. `memoryview(meinobjekt)` geht dann.
//!
//! pygame benutzt das Paar in seiner eigenen Testsuite
//! (`tests/test_utils/buftools.py` baut damit Felder mit jeder Form,
//! Schrittweite und Byteordnung). Es ist also kein Beiwerk, sondern das
//! Werkzeug, mit dem pygame sein eigenes Puffer-Verhalten prueft.
//!
//! ## Warum das hier und nicht in `puffer.rs`
//!
//! `puffer.rs` ist die Seite, die **rustgame** ausgibt: Flaechen, Masken,
//! Klaenge, Farben. Dieses Modul ist die Seite, die ein **Python-Programm**
//! ausgibt - die Richtung ist umgekehrt, und die Zusagen sind andere.
//!
//! ## Die Schnittstelle ist **rohe Adressen**, nicht Python-Werte
//!
//! Das ist der Punkt, an dem der erste Entwurf dieser Datei falsch war. Er
//! nahm `view.format = "B"` und `view.shape = [4]` an - naheliegend, und
//! genau daran waere pygames eigenes Testwerkzeug gescheitert. Gemessen
//! schreibt `buftools.py` naemlich das:
//!
//! ```python
//! view.buf     = self.buf                  # void*       als int
//! view.format  = addressof(self._format)   # char*       als int
//! view.shape   = addressof(self._shape)    # Py_ssize_t* als int
//! view.strides = addressof(self._strides)  # Py_ssize_t* als int
//! view.suboffsets = None                   # NULL
//! ```
//!
//! Alle sechs Zeigerfelder (`buf`, `format`, `shape`, `strides`,
//! `suboffsets`, `internal`) sind **Zahlen oder `None`** - beim Setzen und
//! beim Lesen. Wer die Werte haelt, auf die sie zeigen, ist Sache des
//! Aufrufers; `buftools.py` haelt sie in `ctypes`-Puffern am Objekt.
//! Diese Datei legt deshalb **nichts** an und muss nichts ueberleben
//! lassen - anders als der erste Entwurf, der `CString` und `Vec` anlegte
//! und sie in `view.internal` parken musste, damit sie den Aufruf
//! ueberdauerten.
//!
//! Gelernt ist daran dies: **eine Schnittstelle, die man erraten kann, ist
//! noch nicht gemessen.** Beide Entwuerfe liefen; nur einer passt zu dem,
//! was pygame tatsaechlich hineinschreibt.
//!
//! ## Drei Zustaende, und sie bestimmen, was schreibbar ist
//!
//! | Zustand | wie man hinkommt | Lesen | Schreiben |
//! |---|---|---|---|
//! | **leer** | `Py_buffer()` | `AttributeError` | `AttributeError` |
//! | **eigen** | `get_buffer(obj, flags)` | Werte | `AttributeError` *read-only* |
//! | **geliehen** | im `_get_buffer`-Rueckruf | Werte | **geht** |
//!
//! Gemessen, jeder Zustand einzeln. Der dritte ist der eigentliche Zweck
//! der Klasse: nur dort darf geschrieben werden, und zwar in die Struktur,
//! die CPython fuer den Aufruf mitgebracht hat.
//!
//! ```text
//! Py_buffer()                     eine unbelegte Sicht, bool() -> False
//! Py_buffer().obj                 AttributeError (siehe unten)
//! Py_buffer().len = 4             AttributeError, derselbe Satz
//! Py_buffer().release_buffer()    None - kein Fehler
//! b.get_buffer(bytearray(b"abc")) None; bool(b) -> True
//! b.get_buffer(.., ..) nochmal    ValueError: The Py_buffer struct is
//!                                 already filled in
//! b.len = 4  (nach get_buffer)    AttributeError: .. is read-only
//! b.buf, b.format, b.shape        int - Adressen
//! b.readonly                      bool, nicht int
//! BufferMixin()._get_buffer(..)   NotImplementedError: abstract method
//! BufferMixin()._release_buffer   None - **kein** abstrakter Haken
//! BufferMixin(1)                  geht - jedes Argument wird angenommen
//! memoryview(ohne _get_buffer)    NotImplementedError: abstract method
//! memoryview(setzt nichts)        MemoryError ohne Text
//! ```
//!
//! ## Zwei Fehler von pygame, die hier nachgebaut sind
//!
//! **Das Fehlerfeld ist 400 Zeichen breit.** pygames Text lautet
//! `property %400s is undefined for an unallocated view`, und `%400s`
//! richtet **rechts** aus: zwischen `property` und dem Feldnamen stehen
//! knapp vierhundert Leerzeichen. Die Meldung ist 446 Zeichen lang und
//! sieht im Terminal aus wie ein Zeilenumbruch. Sie ist hier Zeichen fuer
//! Zeichen nachgebaut, weil ein Programm, das `str(e)` vergleicht, sonst
//! den Unterschied sieht - und weil sie nicht zu raten, sondern zu messen
//! war.
//!
//! **`strides` meldet `shape`.** Fragt man eine leere Sicht nach
//! `strides`, antwortet pygame `property shape is undefined for an
//! unallocated view`. Ein uebersehener Kopiervorgang in `newbuffer.c`.
//! Nachgebaut, mit demselben Grund.
//!
//! ## Was pygame hier zum Absturz bringt
//!
//! Zwei Stellen, und beide sind gemessen:
//!
//! * **`pygame.newbuffer.Py_buffer(1)`** - ein Argument an eine Klasse,
//!   die keines nimmt, und pygame faellt mit einem
//!   Speicherzugriffsfehler um (`pygame_parachute`). Hier gibt es dafuer
//!   einen `TypeError`.
//! * **Ein `_get_buffer`, das `view.format` eine Zeichenkette gibt**,
//!   bekommt bei pygame einen `TypeError` - und weil die Sicht damit
//!   unvollstaendig bleibt, stuerzt das anschliessende `memoryview()` ab.
//!   Hier bleibt der `TypeError`, und die Sicht wird abgelehnt statt
//!   benutzt.
//!
//! Beide stehen in den benannten Grenzen. Ein Absturz ist das einzige
//! Verhalten, das **nicht** nachgebaut wird.
//!
//! ## Und ein Loch, das hier zugemacht ist: die ungenullte Sicht
//!
//! **pygame nullt CPythons Struktur nicht**, bevor es `_get_buffer`
//! ruft. Ein Rueckruf, der ein Feld nicht setzt, laesst dort also stehen,
//! was auf dem Stapel lag. Gemessen am **gleichen** Fall - ein
//! `_get_buffer`, das gar nichts setzt - und dreimal nacheinander:
//!
//! ```text
//! MemoryError
//! (nbytes 0, b"")
//! ValueError: memoryview: number of dimensions must not exceed 64
//! ```
//!
//! Dreimal dasselbe Programm, dreimal eine andere Antwort, weil `ndim`
//! dreimal etwas anderes war. Setzt man `buf` und `len`, aber **nicht**
//! `obj`, kommt auch ein Absturz heraus.
//!
//! Hier wird die Struktur deshalb mit `ptr::write(view, zeroed())`
//! genullt, bevor der Rueckruf sie sieht. Ein unvollstaendiges
//! `_get_buffer` bekommt dann eine klare Antwort (`MemoryError`, weil
//! `buf` NULL ist) statt einer zufaelligen.
//!
//! Darum pruefen die Proben diese Faelle **nicht** gegeneinander: eine
//! Probe, deren Referenzwert vom Stapelinhalt abhaengt, misst nichts - und
//! eine, die den Vergleichslauf abstuerzen laesst, misst weniger als
//! nichts.
//!
//! ## Warum hier `unsafe` steht
//!
//! Das Puffer-Protokoll ist eine C-Schnittstelle, und diese Datei ist
//! genau die Stelle, an der sie beginnt: `__getbuffer__` bekommt einen
//! `*mut Py_buffer` von CPython und muss ihn fuellen. Es gibt keinen
//! sicheren Weg dorthin. Damit ist dies - neben `puffer.rs` - die
//! **zweite** von zwei Dateien im Baum, in denen `unsafe` wirkliche
//! Arbeit tut; fuenf weitere reichen nur Slots durch.
//!
//! Die Regeln, die hier gelten:
//!
//! * Jeder Zugriff auf den geliehenen Zeiger steht in einem Block mit
//!   einem Satz, der sagt, **warum** er gueltig ist.
//! * Die Huelle traegt den Zeiger nur, solange CPython im Aufruf steht -
//!   `__getbuffer__` nimmt ihn am Ende zurueck und loescht ihn, damit kein
//!   Programm die Huelle herausschmuggeln und spaeter benutzen kann.
//! * Geschrieben werden ausschliesslich die elf Felder der Struktur, nie
//!   der Speicher, auf den sie zeigen.

#![allow(unsafe_code)]

use std::os::raw::{c_char, c_int, c_void};

use pyo3::exceptions::{
    PyAttributeError, PyBufferError, PyMemoryError, PyNotImplementedError, PyTypeError,
    PyValueError,
};
use pyo3::ffi;
use pyo3::prelude::*;

/// Die Breite, auf die pygame den Feldnamen schiebt - `%400s`.
const FELDBREITE: usize = 400;

/// Der Satz, mit dem pygame eine leere Sicht ablehnt - **400 Zeichen
/// breit**.
///
/// `property %400s is undefined for an unallocated view`, rechts
/// ausgerichtet. Gemessen: 446 Zeichen lang, 397 Leerzeichen vor `buf`.
fn unbelegt(feld: &str) -> PyErr {
    PyAttributeError::new_err(unbelegt_text(feld))
}

/// Derselbe Satz als **Text** - damit er ohne Interpreter zu pruefen ist.
///
/// Die Tests dieser Datei laufen in `cargo test`, und dort gibt es kein
/// Python: ein `format!("{err}")` auf einen `PyErr` bricht mit *The Python
/// interpreter is not initialized* ab. Der Satz ist aber genau das, was
/// gemessen wurde - also wird er hier gebaut und dort geprueft.
fn unbelegt_text(feld: &str) -> String {
    format!("property {feld:>FELDBREITE$} is undefined for an unallocated view")
}

/// Der Satz fuer eine **eigene** Sicht: alles schreibgeschuetzt.
///
/// Dieselbe Breite, derselbe Grund - gemessen `property <397 Leerzeichen>
/// buf is read-only`.
fn nurlesen(feld: &str) -> PyErr {
    PyAttributeError::new_err(nurlesen_text(feld))
}

/// Derselbe Satz als Text - siehe [`unbelegt_text`].
fn nurlesen_text(feld: &str) -> String {
    format!("property {feld:>FELDBREITE$} is read-only")
}

/// Der Name, den pygame im Fehlertext nennt - **mit seinem Fehler**.
///
/// `strides` meldet `shape`. Ein uebersehener Kopiervorgang in
/// `newbuffer.c`; hier nachgebaut, damit `str(e)` uebereinstimmt.
fn gemeldeter_name(feld: &str) -> &str {
    if feld == "strides" {
        "shape"
    } else {
        feld
    }
}

/// Woher die Struktur kommt, die die Huelle zeigt.
enum Woher {
    /// Nichts da - `Py_buffer()`.
    Leer,
    /// Selbst geholt mit `get_buffer()`; wir geben sie auch wieder her.
    Eigen(Box<ffi::Py_buffer>),
    /// CPythons Struktur, geliehen fuer die Dauer von `__getbuffer__`.
    ///
    /// # Sicherheit
    ///
    /// Der Zeiger ist gueltig, solange CPython in dem Aufruf steht, der
    /// ihn mitgebracht hat. `__getbuffer__` setzt den Zustand am Ende
    /// wieder auf `Leer`, damit die Huelle ihn nicht ueberlebt.
    Geliehen(*mut ffi::Py_buffer),
}

/// `pygame.newbuffer.Py_buffer` - eine Huelle um CPythons Struktur.
///
/// Elf Felder, die Zeiger davon als Zahl oder `None`;
/// `get_buffer`/`release_buffer` holen und geben eine Sicht; `bool()` sagt,
/// ob eine da ist.
// **`subclass`** - pygames Fassung ist eine gewoehnliche
// Python-Klasse, und von der laesst sich erben. PyO3 verbietet
// das ohne dieses Flag (`is not an acceptable base type`).
// Gefunden hat es die Probe `Erben`, die fuer jede der 65
// Klassen fragt, ob eine Ableitung geht - acht Runden hatten
// nur gefragt, ob die Klasse **da** ist.
#[pyclass(
    name = "Py_buffer",
    module = "rustgame_py.newbuffer",
    unsendable,
    subclass
)]
pub struct PyPyBuffer {
    /// Der Zustand - siehe [`Woher`].
    woher: Woher,
}

/// Liest ein Zeigerfeld als Zahl - `None`, wenn es NULL ist.
fn zeiger(wert: *mut c_void) -> Option<isize> {
    if wert.is_null() {
        None
    } else {
        Some(wert as isize)
    }
}

/// Nimmt eine Zahl oder `None` als Zeiger an.
///
/// **Kein `extract` auf alles:** pygame nimmt hier nur Zahlen, und eine
/// Zeichenkette gibt einen `TypeError`. Genau daran ist der erste Entwurf
/// dieser Datei aufgefallen.
fn als_zeiger(feld: &str, wert: &Bound<'_, PyAny>) -> PyResult<*mut c_void> {
    if wert.is_none() {
        return Ok(std::ptr::null_mut());
    }
    let zahl: isize = wert.extract().map_err(|_| {
        PyTypeError::new_err(format!(
            "property {feld} must be an integer address or None, not {}",
            wert.get_type()
                .name()
                .map(|n| n.to_string())
                .unwrap_or_else(|_| "?".into())
        ))
    })?;
    Ok(zahl as *mut c_void)
}

impl PyPyBuffer {
    /// Die Struktur zum Lesen - oder der Fehler fuer eine leere Sicht.
    ///
    /// # Sicherheit
    ///
    /// Bei `Geliehen` ist der Zeiger gueltig, weil CPython in dem Aufruf
    /// steht, der ihn mitgebracht hat (siehe [`Woher::Geliehen`]).
    fn lesen(&self, feld: &str) -> PyResult<&ffi::Py_buffer> {
        match &self.woher {
            Woher::Leer => Err(unbelegt(gemeldeter_name(feld))),
            Woher::Eigen(b) => Ok(b),
            // SICHERHEIT: gueltig fuer die Dauer von `__getbuffer__`.
            Woher::Geliehen(p) => Ok(unsafe { &**p }),
        }
    }

    /// Die Struktur zum Schreiben - nur im geliehenen Zustand.
    ///
    /// Drei Antworten, alle gemessen: leer gibt *undefined for an
    /// unallocated view*, eigen gibt *read-only*, geliehen schreibt.
    ///
    /// # Sicherheit
    ///
    /// Wie [`Self::lesen`].
    fn schreiben(&mut self, feld: &str) -> PyResult<&mut ffi::Py_buffer> {
        match &mut self.woher {
            Woher::Leer => Err(unbelegt(gemeldeter_name(feld))),
            Woher::Eigen(_) => Err(nurlesen(gemeldeter_name(feld))),
            // SICHERHEIT: gueltig fuer die Dauer von `__getbuffer__`.
            Woher::Geliehen(p) => Ok(unsafe { &mut **p }),
        }
    }

    /// Eine Huelle, die CPythons Struktur zeigt - nur fuer
    /// `__getbuffer__`.
    fn geliehen(zeiger: *mut ffi::Py_buffer) -> Self {
        Self {
            woher: Woher::Geliehen(zeiger),
        }
    }

    /// Nimmt den geliehenen Zeiger zurueck.
    ///
    /// Danach ist die Huelle leer, und ein Programm, das sie aus dem
    /// Rueckruf herausgeschmuggelt hat, bekommt `AttributeError` statt
    /// eines Zugriffs auf eine Struktur, die es nicht mehr gibt.
    fn zurueckgeben(&mut self) {
        self.woher = Woher::Leer;
    }
}

/// Die Zahlenfelder als Eigenschaften - Lesen und Schreiben.
///
/// Das Makro schreibt je Feld zwei Methoden. Von Hand waeren es neun fast
/// gleiche Paare, und das neunte haette den Fehler. Der Setzername steht
/// mit im Aufruf, weil es im Baum kein `paste` gibt und ein Makro keine
/// Namen zusammensetzen kann.
macro_rules! felder {
    ($(
        $name:ident / $setzer:ident : $rust:ty,
        |$b:ident| $hole:expr,
        |$bm:ident, $wert:ident| $lege:expr
    );+ $(;)?) => {
        #[pymethods]
        impl PyPyBuffer {
            $(
                #[getter($name)]
                fn $name(&self) -> PyResult<$rust> {
                    let $b = self.lesen(stringify!($name))?;
                    Ok($hole)
                }

                #[setter($name)]
                fn $setzer(&mut self, $wert: &Bound<'_, PyAny>) -> PyResult<()> {
                    // **Der Zustand wird vor dem Wert geprueft** - gemessen
                    // gibt `Py_buffer().format = "B"` den
                    // *unallocated*-Fehler und nicht den `TypeError`.
                    let $bm = self.schreiben(stringify!($name))?;
                    $lege;
                    Ok(())
                }
            )+
        }
    };
}

felder! {
    buf / set_buf: Option<isize>,
        |b| zeiger(b.buf),
        |b, wert| b.buf = als_zeiger("buf", wert)?;
    len / set_len: isize,
        |b| b.len,
        |b, wert| b.len = wert.extract()?;
    itemsize / set_itemsize: isize,
        |b| b.itemsize,
        |b, wert| b.itemsize = wert.extract()?;
    ndim / set_ndim: i32,
        |b| b.ndim,
        |b, wert| b.ndim = wert.extract()?;
    format / set_format: Option<isize>,
        |b| zeiger(b.format.cast()),
        |b, wert| b.format = als_zeiger("format", wert)?.cast::<c_char>();
    shape / set_shape: Option<isize>,
        |b| zeiger(b.shape.cast()),
        |b, wert| b.shape = als_zeiger("shape", wert)?.cast::<ffi::Py_ssize_t>();
    strides / set_strides: Option<isize>,
        |b| zeiger(b.strides.cast()),
        |b, wert| b.strides = als_zeiger("strides", wert)?.cast::<ffi::Py_ssize_t>();
    suboffsets / set_suboffsets: Option<isize>,
        |b| zeiger(b.suboffsets.cast()),
        |b, wert| b.suboffsets = als_zeiger("suboffsets", wert)?.cast::<ffi::Py_ssize_t>();
    internal / set_internal: Option<isize>,
        |b| zeiger(b.internal),
        |b, wert| b.internal = als_zeiger("internal", wert)?;
}

#[pymethods]
impl PyPyBuffer {
    /// Eine leere Sicht.
    ///
    /// **`Py_buffer(1)` stuerzt pygame ab** - ein Argument an eine Klasse,
    /// die keines nimmt, und `pygame_parachute` faengt einen
    /// Speicherzugriffsfehler. Hier gibt es dafuer einen `TypeError`; das
    /// ist die eine Abweichung, die Absicht ist.
    #[new]
    fn neu() -> Self {
        Self { woher: Woher::Leer }
    }

    /// `readonly` - gemessen kommt ein **`bool`** heraus, keine Zahl.
    #[getter(readonly)]
    fn readonly(&self) -> PyResult<bool> {
        Ok(self.lesen("readonly")?.readonly != 0)
    }

    /// `readonly` setzen - `0`, `1`, `True` und `False` gehen alle.
    ///
    /// `buftools.py` schreibt `view.readonly = self.readonly`, und das ist
    /// dort ein `int`; anderswo steht `True`. Beides muss durch.
    #[setter(readonly)]
    fn set_readonly(&mut self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        // **Der Zustand zuerst.** Stand die Umrechnung davor, gab
        // `sicht.readonly = None` auf einer eigenen Sicht einen
        // `TypeError` statt pygames `AttributeError: .. is read-only` -
        // gefunden von der Probe, die jedes Feld mit `None` belegt.
        let b = self.schreiben("readonly")?;
        let an: i64 = if let Ok(w) = wert.extract::<bool>() {
            i64::from(w)
        } else {
            wert.extract()?
        };
        b.readonly = c_int::from(an != 0);
        Ok(())
    }

    /// `obj` - das Objekt, dem der Puffer gehoert, oder `None`.
    #[getter(obj)]
    fn obj(&self, py: Python<'_>) -> PyResult<Option<Py<PyAny>>> {
        let b = self.lesen("obj")?;
        if b.obj.is_null() {
            return Ok(None);
        }
        // SICHERHEIT: `obj` ist ein gueltiger Verweis, solange die Sicht
        // steht; `from_borrowed_ptr` zaehlt ihn hoch, statt ihn zu
        // uebernehmen.
        Ok(Some(unsafe {
            Bound::from_borrowed_ptr(py, b.obj).unbind()
        }))
    }

    /// `obj` setzen - und **den Verweis hochzaehlen**.
    ///
    /// Das ist die Zeile, auf die es ankommt: CPython gibt den Verweis in
    /// `PyBuffer_Release` wieder her, also muss er hier einer sein, der
    /// uns gehoert. Ohne das Hochzaehlen stirbt das Objekt, waehrend die
    /// Sicht noch darauf zeigt - `buftools.Exporter` schreibt
    /// `view.obj = self`, und `self` ist dann die einzige Quelle des
    /// Speichers, auf den `buf` zeigt.
    #[setter(obj)]
    fn set_obj(&mut self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        // Auch hier der Zustand zuerst - sonst waere der Verweis
        // hochgezaehlt und niemand gaebe ihn wieder her.
        let b = self.schreiben("obj")?;
        let neu = if wert.is_none() {
            std::ptr::null_mut()
        } else {
            wert.clone().unbind().into_ptr()
        };
        if !b.obj.is_null() {
            // SICHERHEIT: ein Verweis, den wir vorher selbst hochgezaehlt
            // haben, wird hier wieder hergegeben.
            unsafe { ffi::Py_DECREF(b.obj) };
        }
        b.obj = neu;
        Ok(())
    }

    /// `bool(sicht)` - ist eine Sicht da?
    fn __bool__(&self) -> bool {
        !matches!(self.woher, Woher::Leer)
    }

    /// Holt eine Sicht auf `obj`.
    ///
    /// `flags` sind die `PyBUF_*`-Zahlen und haben die Vorgabe `0`
    /// (`PyBUF_SIMPLE`). Eine zweite Sicht in dieselbe Huelle gibt
    /// `ValueError: The Py_buffer struct is already filled in` - gemessen.
    #[pyo3(signature = (obj, flags = 0))]
    fn get_buffer(&mut self, obj: &Bound<'_, PyAny>, flags: c_int) -> PyResult<()> {
        if !matches!(self.woher, Woher::Leer) {
            return Err(PyValueError::new_err(
                "The Py_buffer struct is already filled in",
            ));
        }
        // SICHERHEIT: eine genullte Struktur, die uns gehoert. `Py_buffer`
        // hat keine Invariante, die eine Null verletzt - CPython fuellt
        // sie gleich selbst.
        let mut sicht: Box<ffi::Py_buffer> = Box::new(unsafe { std::mem::zeroed() });
        // SICHERHEIT: `obj` ist ein gueltiger Verweis, `sicht` gross
        // genug. Schlaegt der Aufruf fehl, steht der Fehler in Pythons
        // Zustand und die Struktur bleibt ungenutzt.
        let schlecht = unsafe { ffi::PyObject_GetBuffer(obj.as_ptr(), &mut *sicht, flags) };
        if schlecht != 0 {
            return Err(PyErr::fetch(obj.py()));
        }
        self.woher = Woher::Eigen(sicht);
        Ok(())
    }

    /// Gibt die Sicht wieder her - **ohne Fehler, wenn keine da ist**.
    ///
    /// Gemessen gibt `Py_buffer().release_buffer()` einfach `None`.
    ///
    /// Im geliehenen Zustand wird **nicht** freigegeben: die Struktur
    /// gehoert CPython, und wer sie dort hergibt, zieht sie dem Aufruf
    /// unter den Fuessen weg. Die Huelle wird nur leer - und pygame tut
    /// gemessen dasselbe (`release_buffer()` im Rueckruf gibt `None`, und
    /// danach ist weiterzuarbeiten).
    fn release_buffer(&mut self) {
        if let Woher::Eigen(mut sicht) = std::mem::replace(&mut self.woher, Woher::Leer) {
            // SICHERHEIT: die Struktur haben wir selbst mit
            // `PyObject_GetBuffer` gefuellt und keinem anderen gegeben.
            unsafe { ffi::PyBuffer_Release(&mut *sicht) };
        }
    }
}

impl Drop for PyPyBuffer {
    /// Eine eigene Sicht wird hergegeben, auch wenn niemand danach fragt.
    ///
    /// Ohne das haelt ein `Py_buffer`, den ein Programm fallen laesst,
    /// sein Ausgabeobjekt fuer immer fest - und bei einer `bytearray`
    /// heisst das, dass sie nicht mehr wachsen darf.
    fn drop(&mut self) {
        self.release_buffer();
    }
}

/// `pygame.newbuffer.BufferMixin` - eine Python-Klasse wird Puffer-Ausgeber.
///
/// Wer davon erbt und `_get_buffer(self, view, flags)` schreibt, kann mit
/// `memoryview(x)` angesprochen werden. `_release_buffer(self, view)` ist
/// freiwillig.
#[pyclass(
    name = "BufferMixin",
    module = "rustgame_py.newbuffer",
    subclass,
    unsendable
)]
pub struct PyBufferMixin;

#[pymethods]
impl PyBufferMixin {
    /// **Nimmt jedes Argument an und sieht keines an.**
    ///
    /// Gemessen: `BufferMixin(1)` geht bei pygame durch. Das ist kein
    /// Zufall, sondern noetig - jede Unterklasse hat ihr eigenes
    /// `__init__` mit eigenen Argumenten, und `buftools.Exporter` nimmt
    /// fuenf.
    #[new]
    #[pyo3(signature = (*_args, **_kwargs))]
    fn neu(_args: &Bound<'_, PyAny>, _kwargs: Option<&Bound<'_, PyAny>>) -> Self {
        Self
    }

    /// Fuellt die Sicht - **in der Unterklasse zu schreiben**.
    ///
    /// Gemessen gibt pygame hier `NotImplementedError: abstract method`,
    /// und zwar sowohl beim direkten Aufruf als auch aus `memoryview()`
    /// heraus.
    fn _get_buffer(&self, _view: &Bound<'_, PyAny>, _flags: c_int) -> PyResult<()> {
        Err(PyNotImplementedError::new_err("abstract method"))
    }

    /// Wird beim Hergeben gerufen - **tut hier nichts**.
    ///
    /// Gemessen: anders als `_get_buffer` ist das kein abstrakter Haken.
    /// Wer nichts freizugeben hat, schreibt ihn nicht.
    fn _release_buffer(&self, _view: &Bound<'_, PyAny>) {}

    /// Das Puffer-Protokoll: CPython fragt, `_get_buffer` antwortet.
    ///
    /// ## Was hier geschieht
    ///
    /// 1. CPythons Struktur wird genullt - ein `_get_buffer`, das ein Feld
    ///    nicht setzt, hinterlaesst dann eine Null und keinen
    ///    Zufallswert. **pygame tut das nicht**, und das ist dort ein
    ///    Loch; siehe den Modulkopf.
    /// 2. Eine Huelle wird darauf geliehen und an `_get_buffer` gegeben.
    /// 3. Danach wird der Zeiger in der Huelle **geloescht**, damit ein
    ///    Programm sie nicht herausschmuggeln und spaeter benutzen kann.
    /// 4. Ist `buf` danach NULL, kommt `MemoryError` **ohne Text** -
    ///    gemessen gibt pygame genau das (`PyErr_NoMemory`), wenn ein
    ///    `_get_buffer` nichts setzt.
    ///
    /// Punkt 3 steht **vor** Punkt 4 und vor dem Weiterwerfen des
    /// Fehlers: der Zeiger ist auch dann zurueckzugeben, wenn der Rueckruf
    /// scheitert.
    ///
    /// # Sicherheit
    ///
    /// `view` ist der Zeiger, den CPython fuer diesen Aufruf mitbringt; er
    /// ist bis zur Rueckkehr gueltig. Geschrieben werden nur die elf
    /// Felder der Struktur, nie der Speicher, auf den sie zeigen.
    unsafe fn __getbuffer__(
        slf: Bound<'_, Self>,
        view: *mut ffi::Py_buffer,
        flags: c_int,
    ) -> PyResult<()> {
        let py = slf.py();
        if view.is_null() {
            return Err(PyBufferError::new_err("view is null"));
        }
        // SICHERHEIT: `view` ist nicht NULL und zeigt auf eine Struktur,
        // die CPython fuer diesen Aufruf bereitgestellt hat.
        unsafe { std::ptr::write(view, std::mem::zeroed()) };

        let huelle = Bound::new(py, PyPyBuffer::geliehen(view))?;
        let ergebnis = slf
            .getattr("_get_buffer")
            .and_then(|f| f.call1((&huelle, flags)))
            .map(|_| ());
        huelle.borrow_mut().zurueckgeben();
        ergebnis?;

        // SICHERHEIT: wie oben - nur gelesen.
        if unsafe { (*view).buf }.is_null() {
            // pygames `PyErr_NoMemory()`: `MemoryError` ohne Text.
            return Err(PyMemoryError::new_err(""));
        }
        Ok(())
    }

    /// Das Gegenstueck: `_release_buffer` darf noch einmal hineinsehen.
    ///
    /// Die Sicht ist dabei **noch vollstaendig** - gemessen liest pygames
    /// Haken dort `view.obj` und `view.len`. Den Verweis auf `obj` gibt
    /// CPython selbst her, nicht wir.
    ///
    /// # Sicherheit
    ///
    /// Wie [`Self::__getbuffer__`].
    unsafe fn __releasebuffer__(slf: Bound<'_, Self>, view: *mut ffi::Py_buffer) {
        let py = slf.py();
        if view.is_null() {
            return;
        }
        if let Ok(huelle) = Bound::new(py, PyPyBuffer::geliehen(view)) {
            // Ein Fehler aus `_release_buffer` ist nicht zu melden -
            // CPython hat an dieser Stelle keinen Platz dafuer, und pygame
            // verschluckt ihn ebenso.
            let _ = slf
                .getattr("_release_buffer")
                .and_then(|f| f.call1((&huelle,)));
            huelle.borrow_mut().zurueckgeben();
        }
    }
}

#[cfg(test)]
mod tests {
    //! Die Zustandsmaschine und die zwei nachgebauten Fehler.
    //!
    //! Was sich nur mit einem lebenden Interpreter pruefen laesst, prueft
    //! `vergleich_module.py` gegen das echte pygame; hier stehen die
    //! Teile, die fuer sich stehen.

    use super::*;

    #[test]
    fn feld_vierhundert_breit() {
        let s = unbelegt_text("buf");
        assert_eq!(s.len(), 446, "pygames Meldung ist 446 Zeichen lang");
        assert!(s.starts_with("property  "));
        assert!(s.ends_with("     buf is undefined for an unallocated view"));
        // Genau 397 Leerzeichen zwischen `property ` und `buf`.
        let rest = &s["property ".len()..];
        assert_eq!(rest.len() - rest.trim_start().len(), 397);
    }

    #[test]
    fn breite_haengt_am_namen() {
        // `itemsize` ist fuenf Zeichen laenger, also stehen fuenf
        // Leerzeichen weniger davor - **und die Meldung bleibt gleich
        // lang**. Das war der eigentliche Fund dieses Tests: der erste
        // Entwurf rechnete `446 + feld.len() - 3` und war damit falsch,
        // denn `%400s` polstert auf eine **feste** Breite. Jede der elf
        // Meldungen ist 446 Zeichen lang, und die Probe gegen pygame
        // bestaetigt genau das fuer alle elf Felder.
        for (feld, leer) in [("buf", 397), ("itemsize", 392), ("suboffsets", 390)] {
            let s = unbelegt_text(feld);
            let rest = &s["property ".len()..];
            assert_eq!(rest.len() - rest.trim_start().len(), leer, "{feld}");
            assert_eq!(s.len(), 446, "{feld}");
            assert_eq!(nurlesen_text(feld).len(), 422, "{feld}");
        }
    }

    #[test]
    fn strides_meldet_shape() {
        // pygames Fehler, nachgebaut: `strides` nennt `shape`.
        assert_eq!(gemeldeter_name("strides"), "shape");
        assert_eq!(gemeldeter_name("shape"), "shape");
        assert_eq!(gemeldeter_name("buf"), "buf");
        let s = unbelegt_text(gemeldeter_name("strides"));
        assert!(s.ends_with("shape is undefined for an unallocated view"));
    }

    #[test]
    fn nurlesen_gleich_breit() {
        let s = nurlesen_text("buf");
        assert!(s.ends_with("     buf is read-only"));
        let rest = &s["property ".len()..];
        assert_eq!(rest.len() - rest.trim_start().len(), 397);
    }

    #[test]
    fn leere_huelle_lehnt_alles_ab() {
        let mut h = PyPyBuffer::neu();
        assert!(!h.__bool__());
        assert!(h.lesen("len").is_err());
        assert!(h.schreiben("len").is_err());
        // Und das Hergeben einer Sicht, die es nicht gibt, ist kein Fehler.
        h.release_buffer();
        assert!(!h.__bool__());
    }

    #[test]
    fn zeiger_null_ist_none() {
        assert_eq!(zeiger(std::ptr::null_mut()), None);
        assert_eq!(zeiger(8 as *mut c_void), Some(8));
    }

    #[test]
    fn zurueckgeben_macht_leer() {
        let mut h = PyPyBuffer::geliehen(std::ptr::null_mut());
        assert!(h.__bool__());
        h.zurueckgeben();
        assert!(!h.__bool__());
        assert!(h.lesen("buf").is_err());
    }
}
