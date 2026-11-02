//! pixelfeld.rs
//! ============
//! `pygame.PixelArray` - eine Sicht auf die Pixel einer Flaeche.
//!
//! ## Was gemessen ist
//!
//! | Aufruf | pygame-ce 2.5.7 |
//! |---|---|
//! | `PixelArray(s)` bei 4x3 | `shape (4, 3)`, `ndim 2`, `len 4` |
//! | `pa[0]` | ein **eindimensionales** `PixelArray` der Laenge 3 |
//! | `pa[0][0]`, `pa[0, 0]` | die **gepackte Zahl** im Format der Flaeche |
//! | `pa[1:3, 0:2]` | 2D, `shape (2, 2)` - beide Achsen auf einmal |
//! | `pa[:, 1]` | 1D **ueber x**, Laenge = Breite |
//! | `pa[1, :]` | 1D **ueber y**, Laenge = Hoehe |
//! | `pa[...]` | **dieselbe** Sicht (`pa[...] is pa`) |
//! | `pa[4:4]` | **`None`** - eine leere Scheibe gibt kein Feld |
//! | `strides` bei 32 Bit, Breite 4 | `(4, 16)`; nach `[::2, :]` `(8, 16)` |
//! | `itemsize` | die **Bytezahl der Flaeche**: 1, 2, 3 oder 4 |
//! | `pa.surface is s` | **wahr** |
//! | `pa[0] = wert` | faerbt die ganze Spalte |
//! | `pa[0:2] = [a, b]` | eine Zahlenfolge belegt **Spalte fuer Spalte** |
//! | `pa[:] = (1, 2, 3)` | ein **Tupel** ist eine Farbe |
//! | `pa[:] = [1, 2, 3]` | eine **Liste** ist eine Pixelfolge |
//! | `extract(farbe)` | neue Sicht: **weiss** wo Treffer, sonst schwarz |
//! | `compare(andere)` | **weiss wo gleich**, schwarz wo verschieden |
//! | `replace(alt, neu)` | `None`, aendert an Ort und Stelle |
//! | nach `close()` | `ValueError: Operation on closed PixelArray.` |
//! | `PixelArray(42)` | `TypeError: argument 1 must be pygame.surface.Surface, not int` |
//! | `PixelArray(Surface((5,0)))` | `ValueError` |
//!
//! Beachte die Achsen: **`pa[x]` ist eine Spalte**, nicht eine Zeile.
//! `len(pa)` ist also die Breite, und `pa[0]` hat die Hoehe als Laenge.
//! Wer das verwechselt, baut eine Sicht, die bei quadratischen Flaechen
//! genau richtig aussieht.
//!
//! ## Wie die Sicht gebaut ist
//!
//! Hier stand einmal ein Paar von **Nummernlisten** - je eine fuer die
//! sichtbaren Spalten und Zeilen. Das war einfach zu lesen und an drei
//! Stellen schlicht falsch:
//!
//! * `strides` laesst sich daraus nicht ablesen. pygame meldet nach
//!   `[::2, :]` den doppelten Schritt; eine Liste `[0, 2, 4]` weiss von
//!   keinem Schritt.
//! * `transpose()` einer **eindimensionalen** Sicht gibt `shape (1, n)`.
//!   Mit zwei Listen und einem Merker "eine Achse" ist nicht sagbar,
//!   **welche** Achse das ist - und `pa[:, 1]` laeuft ueber x, `pa[1, :]`
//!   ueber y.
//! * `_pixels_address` haengt am **Anfangspixel**, nicht an der Menge der
//!   getroffenen Pixel.
//!
//! Jetzt steht hier dasselbe Modell wie in pygames `pixelarray.c`: ein
//! **Anfangspixel** und je Achse eine Laenge und ein Schritt. Der Schritt
//! ist ein Paar `(ax, ay)` statt einer Zahl, damit `transpose()` nur zwei
//! Achsen tauscht und nicht einen Sonderfall braucht:
//!
//! ```text
//! (x, y) = (bx + i*ax0 + j*ax1,  by + i*ay0 + j*ay1)
//! ```
//!
//! Normal ist `(ax0, ay0) = (1, 0)` und `(ax1, ay1) = (0, 1)`; nach
//! `transpose()` genau andersherum. Die gemeldeten `strides` fallen damit
//! direkt ab: `ax * itemsize + ay * pitch`.
//!
//! Die Flaeche wird als `Py<PySurface>` festgehalten, nicht als Kopie
//! ihres Puffers: `pa.surface is s` ist gemessen **wahr**, und ein
//! Nachbau, der eine zweite Huelle um denselben Puffer zurueckgibt,
//! faellt genau dort auf.
//!
//! ## Was hier nicht geht
//!
//! `__array_struct__` und das Puffer-Protokoll fehlen - beides verlangt
//! einen echten Zeiger auf Pixel im **gemeldeten** Format, und rustgame
//! haelt jede Flaeche als ARGB32. `__array_interface__` gibt es trotzdem,
//! weil es ein gewoehnliches Woerterbuch ist; die Adresse darin ist die
//! des ARGB-Speichers. Siehe `BEKANNTE_GRENZEN`.

use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PySlice, PyTuple};

use rustgame_core::surface::Surface as KernSurface;

use crate::flaeche::PySurface;

/// Eine Achse der Sicht: wie viele Pixel, und wohin ein Schritt fuehrt.
#[derive(Clone, Copy)]
struct Achse {
    /// Wie viele Pixel auf dieser Achse liegen.
    n: usize,
    /// Wie weit ein Schritt in x fuehrt.
    ax: i64,
    /// Wie weit ein Schritt in y fuehrt.
    ay: i64,
}

impl Achse {
    fn neu(n: usize, ax: i64, ay: i64) -> Achse {
        Achse { n, ax, ay }
    }

    /// Die Achse nach einer Scheibe - gibt auch den **Versatz** des
    /// neuen Anfangspixels zurueck.
    fn geschnitten(&self, start: isize, schritt: isize, n: usize) -> (Achse, i64, i64) {
        let (dx, dy) = (self.ax * start as i64, self.ay * start as i64);
        (
            Achse {
                n,
                ax: self.ax * schritt as i64,
                ay: self.ay * schritt as i64,
            },
            dx,
            dy,
        )
    }
}

/// `pygame.PixelArray`.
#[pyclass(
    name = "PixelArray",
    module = "rustgame_py.pixelarray",
    unsendable,
    subclass,
    dict,
    weakref
)]
pub struct PyPixelArray {
    /// Die Flaeche selbst - damit `pa.surface is s` gilt.
    flaeche: Py<PySurface>,
    /// Das Anfangspixel der Sicht.
    bx: i64,
    by: i64,
    /// Die erste Achse - die, ueber die `len()` und `pa[i]` laufen.
    a0: Achse,
    /// Die zweite Achse. `None` heisst **eindimensional**.
    a1: Option<Achse>,
    geschlossen: bool,
}

/// `ValueError: Operation on closed PixelArray.` - Wortlaut gemessen.
fn zu() -> PyErr {
    PyValueError::new_err("Operation on closed PixelArray.")
}

/// Was ein Index sein kann.
enum Wahl {
    /// Eine einzelne Nummer - die Achse faellt weg.
    Punkt(isize),
    /// Anfang, Schritt und Anzahl.
    Scheibe(isize, isize, usize),
    /// `...` oder `:` - die ganze Achse bleibt.
    Alles,
}

impl PyPixelArray {
    fn pruefen(&self) -> PyResult<()> {
        if self.geschlossen {
            return Err(zu());
        }
        Ok(())
    }

    /// Leiht die Kernflaeche aus und ruft `f`.
    fn mit_kern<R>(&self, py: Python<'_>, f: impl FnOnce(&KernSurface) -> R) -> PyResult<R> {
        let s = self.flaeche.bind(py).borrow();
        let k = s.leihen()?;
        Ok(f(&k))
    }

    /// Die Stelle `(x, y)` fuer die Nummern `(i, j)`.
    fn stelle(&self, i: usize, j: usize) -> (i32, i32) {
        let (mut x, mut y) = (self.bx, self.by);
        x += self.a0.ax * i as i64;
        y += self.a0.ay * i as i64;
        if let Some(a) = self.a1 {
            x += a.ax * j as i64;
            y += a.ay * j as i64;
        }
        (x as i32, y as i32)
    }

    /// Breite und Hoehe der Sicht - bei einer Achse ist die zweite 1.
    fn masse(&self) -> (usize, usize) {
        (self.a0.n, self.a1.map(|a| a.n).unwrap_or(1))
    }

    /// Verpackt eine Sicht als Objekt **desselben Typs** wie `slf`.
    ///
    /// `pygame.PixelArray` ist hier eine duenne Python-Unterklasse (siehe
    /// `python/pixelarray.py`). Wer aus Rust ein `Py::new(py, sicht)`
    /// zurueckgibt, gibt die **Grundklasse** zurueck, und
    /// `isinstance(ar.compare(ar2), pygame.PixelArray)` ist dann falsch -
    /// gemessen faellt genau das in `test_compare` auf.
    ///
    /// Also wird der Typ gefragt und sein `__new__` gerufen; das legt
    /// **keine** Sperre an, und das ist richtig so: gemessen sperrt nur
    /// der Konstruktor, ein Schnitt `ar2[:]` nicht.
    fn wie_typ(slf: &Bound<'_, PyPixelArray>, sicht: PyPixelArray) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let cls = slf.get_type();
        let f = sicht.flaeche.clone_ref(py);
        let obj = cls.call_method1("__new__", (&cls, f))?;
        {
            let b = obj.downcast::<PyPixelArray>()?;
            *b.borrow_mut() = sicht;
        }
        Ok(obj.unbind())
    }

    /// Eine neue Sicht auf dieselbe Flaeche.
    fn sicht(&self, py: Python<'_>, bx: i64, by: i64, a0: Achse, a1: Option<Achse>) -> PyPixelArray {
        PyPixelArray {
            flaeche: self.flaeche.clone_ref(py),
            bx,
            by,
            a0,
            a1,
            geschlossen: false,
        }
    }

    /// Was ein Index bedeutet - `...`, eine Scheibe oder eine Nummer.
    fn wahl(i: &Bound<'_, PyAny>, n: usize) -> PyResult<Wahl> {
        if i.is(i.py().Ellipsis()) {
            return Ok(Wahl::Alles);
        }
        if let Ok(s) = i.downcast::<PySlice>() {
            let t = s.indices(n as isize)?;
            let anzahl = if t.step > 0 {
                if t.stop > t.start {
                    ((t.stop - t.start - 1) / t.step + 1) as usize
                } else {
                    0
                }
            } else if t.start > t.stop {
                ((t.start - t.stop - 1) / (-t.step) + 1) as usize
            } else {
                0
            };
            return Ok(Wahl::Scheibe(t.start, t.step, anzahl));
        }
        Ok(Wahl::Punkt(Self::nummer(i, n)?))
    }

    /// Eine Nummer mit pygames Zaehlung von hinten.
    fn nummer(i: &Bound<'_, PyAny>, n: usize) -> PyResult<isize> {
        let v: isize = i.extract()?;
        let j = if v < 0 { v + n as isize } else { v };
        if j < 0 || j >= n as isize {
            return Err(PyIndexError::new_err("invalid index"));
        }
        Ok(j)
    }

    /// Alle gepackten Pixelwerte der Sicht als `[i][j]`.
    fn werte(&self, py: Python<'_>) -> PyResult<Vec<Vec<u32>>> {
        let (n0, n1) = self.masse();
        self.mit_kern(py, |k| {
            (0..n0)
                .map(|i| (0..n1).map(|j| k.gepackt_an(self.stelle(i, j))).collect())
                .collect()
        })
    }

    /// Baut aus der Sicht eine neue Flaeche.
    fn als_flaeche(&self, py: Python<'_>) -> PyResult<KernSurface> {
        let (n0, n1) = self.masse();
        self.mit_kern(py, |k| {
            let mut neu = KernSurface::wie(k, (n0.max(1) as u32, n1.max(1) as u32));
            for i in 0..n0 {
                for j in 0..n1 {
                    let v = k.gepackt_an(self.stelle(i, j));
                    neu.setz_gepackt((i as i32, j as i32), v);
                }
            }
            neu
        })
    }

    /// Eine Sicht, in der jeder Treffer weiss und alles andere schwarz
    /// ist - die gemeinsame Form von `extract` und `compare`.
    fn maskensicht(
        &self,
        py: Python<'_>,
        treffer: impl Fn(usize, usize) -> bool,
    ) -> PyResult<PyPixelArray> {
        let (n0, n1) = self.masse();
        let (b, h) = (n0.max(1) as u32, n1.max(1) as u32);
        let neu = self.mit_kern(py, |k| {
            let mut n = KernSurface::wie(k, (b, h));
            let weiss = n.map_rgb_gepackt((255u8, 255u8, 255u8));
            let schwarz = n.map_rgb_gepackt((0u8, 0u8, 0u8));
            for i in 0..n0 {
                for j in 0..n1 {
                    let v = if treffer(i, j) { weiss } else { schwarz };
                    n.setz_gepackt((i as i32, j as i32), v);
                }
            }
            n
        })?;
        let flaeche = Py::new(py, PySurface::neu(neu))?;
        Ok(PyPixelArray {
            flaeche,
            bx: 0,
            by: 0,
            a0: Achse::neu(b as usize, 1, 0),
            a1: Some(Achse::neu(h as usize, 0, 1)),
            geschlossen: false,
        })
    }

    /// Bytezahl je Pixel und Zeilenlaenge - **wie die Flaeche sie meldet**,
    /// nicht wie rustgame sie ablegt.
    fn masz(&self, py: Python<'_>) -> PyResult<(i64, i64)> {
        self.mit_kern(py, |k| (k.get_bytesize() as i64, k.get_pitch() as i64))
    }

    /// Der Schritt einer Achse in Bytes.
    fn schritt(a: &Achse, bytes: i64, pitch: i64) -> isize {
        (a.ax * bytes + a.ay * pitch) as isize
    }

    /// Ein einzelner Wert, wie ihn eine Zuweisung ablegt - **ARGB**.
    ///
    /// Eine blanke Zahl ist der **gepackte** Pixelwert dieser Flaeche
    /// (bei 8 Bit also ein Tafelindex) und wird deshalb zurueckgerechnet.
    /// Wer sie roh durchreicht, schreibt auf einer Palettenflaeche die
    /// Zahl 3 als Farbe `(0, 0, 3)` statt als dritten Tafeleintrag.
    fn ein_wert(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<u32> {
        let s = self.flaeche.bind(py).borrow();
        let k = s.leihen()?;
        Self::wert_mit(&k, o)
    }

    /// Dasselbe mit schon geliehener Flaeche.
    ///
    /// **Die Farbe wird vor der Zahl gefragt.** `pygame.Color` laesst sich
    /// nach `int` umwandeln, und wer zuerst `extract::<u32>()` versucht,
    /// bekommt aus `Color(10, 11, 12)` die Zahl `0x0A0B0CFF` und legt sie
    /// als gepackten Pixelwert ab - gemessen kam dabei `(11, 12, 255)`
    /// heraus, um ein Byte verschoben. `test_color_value` prueft genau
    /// diesen Fall.
    fn wert_mit(k: &KernSurface, o: &Bound<'_, PyAny>) -> PyResult<u32> {
        if o.is_instance_of::<pyo3::types::PyBool>() {
            return Err(PyValueError::new_err("invalid color argument"));
        }
        // **Ein gepackter Wert, kein ARGB-Wort.** Seit im Speicher der
        // Tafelindex steht, ist das nicht mehr dasselbe: wer eine Zahl
        // durch eine Farbe schickt und zurueck, bekommt auf pygames
        // Standardtafel aus 128 eine 0 - die obere Haelfte wiederholt die
        // untere. `pixelarray_test.test_self_assign` und
        // `test_extract` haengen genau daran.
        if o.downcast::<crate::farbe::PyColor>().is_ok() || o.downcast::<PyTuple>().is_ok() {
            return Ok(k.map_rgb_gepackt(crate::umwandlung::farbe_aus(o)?));
        }
        if let Ok(z) = o.extract::<u32>() {
            // Eine blanke Zahl **ist** schon der Pixelwert.
            return Ok(z);
        }
        // **Ein Name ist hier keine Farbe.** pygames `PixelArray`
        // nimmt an dieser Stelle nur `Color` und RGB[A]-Tupel; gemessen
        // geben `pa.extract('red')`, `pa.replace((1,2,3), 'red')` und
        // `'red' in pa` alle drei denselben Satz. Der allgemeine
        // `farbe_aus` kennt die Namenstabelle und liess sie durch.
        const ENG: &str = concat!(
            "invalid color (here, generic sequences are restricted, ",
            "but pygame.Color and RGB[A] tuples are allowed)"
        );
        let farbe = crate::umwandlung::farbe_aus(o).map_err(|_| PyValueError::new_err(ENG))?;
        Ok(k.map_rgb_gepackt(farbe))
    }

    /// Ein Wert, wie ihn `__contains__` sucht - **gepackt**.
    fn gesuchter_wert(k: &KernSurface, o: &Bound<'_, PyAny>) -> PyResult<u32> {
        if o.is_instance_of::<pyo3::types::PyBool>() {
            return Err(PyValueError::new_err("invalid color argument"));
        }
        if o.downcast::<crate::farbe::PyColor>().is_ok() || o.downcast::<PyTuple>().is_ok() {
            return Ok(k.map_rgb_gepackt(crate::umwandlung::farbe_aus(o)?));
        }
        if let Ok(z) = o.extract::<u32>() {
            return Ok(z);
        }
        // Derselbe enge Satz wie in [`PyPixelArray::wert_mit`].
        Ok(k.map_rgb_gepackt(crate::umwandlung::farbe_aus(o).map_err(|_| {
            PyValueError::new_err(concat!(
                "invalid color (here, generic sequences are restricted, ",
                "but pygame.Color and RGB[A] tuples are allowed)"
            ))
        })?))
    }

    /// Ist das ein Wert fuer **ein** Pixel - und welcher?
    ///
    /// Gemessen und ueberraschend: ein **Tupel** ist immer eine Farbe,
    /// eine **Liste** nie. `ar[:] = (20, 30, 40)` faerbt alles, waehrend
    /// `ar[:] = [20, 30, 40]` drei Spalten einzeln belegt. Ein Tupel
    /// falscher Laenge gibt deshalb einen `ValueError` und wird nicht
    /// etwa als Folge gelesen.
    fn einzelwert(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Option<u32>> {
        if o.is_instance_of::<pyo3::types::PyBool>() {
            return Ok(None);
        }
        let ist_tupel = o.downcast::<PyTuple>().is_ok();
        let ist_farbe = o.downcast::<crate::farbe::PyColor>().is_ok();
        if ist_tupel || ist_farbe || o.extract::<u32>().is_ok() {
            return Ok(Some(self.ein_wert(py, o)?));
        }
        Ok(None)
    }
}

#[pymethods]
impl PyPixelArray {
    /// `PixelArray(surface)`.
    #[new]
    fn new(py: Python<'_>, surface: &Bound<'_, PyAny>) -> PyResult<PyPixelArray> {
        let f: Py<PySurface> = surface.extract().map_err(|_| {
            PyTypeError::new_err(format!(
                "argument 1 must be pygame.surface.Surface, not {}",
                surface.get_type().name().map(|n| n.to_string()).unwrap_or_default()
            ))
        })?;
        let (b, h) = f.bind(py).borrow().leihen()?.get_size();
        // Gemessen: eine Flaeche ohne Breite oder Hoehe gibt einen
        // `ValueError`, keinen Absturz - pygame-ce Ausgabe 2275.
        if b == 0 || h == 0 {
            return Err(PyValueError::new_err(
                "Cannot create PixelArray with a surface of width or height 0",
            ));
        }
        Ok(PyPixelArray {
            flaeche: f,
            bx: 0,
            by: 0,
            a0: Achse::neu(b as usize, 1, 0),
            a1: Some(Achse::neu(h as usize, 0, 1)),
            geschlossen: false,
        })
    }

    /// Traegt sich als **Sperre** in die Flaeche ein.
    ///
    /// Gemessen: `pygame.PixelArray(s)` macht aus `s.get_locks()` ein
    /// einelementiges Tupel, und darin steht **das PixelArray**, nicht die
    /// Flaeche. `pa.close()` nimmt es wieder heraus.
    ///
    /// Gerufen wird das aus `__init__` - und das steht in Python, eine
    /// Ebene ueber dieser Klasse (siehe `python/pixelarray.py`). Hier
    /// kann es nicht stehen: `__new__` gibt nur den Rust-Wert zurueck,
    /// und den kann man nirgends eintragen; PyO3 0.26 wiederum hat fuer
    /// `__init__` keinen Schlitz, den `type.__call__` beachtet.
    fn _sperre_setzen(slf: &Bound<'_, PyPixelArray>) -> PyResult<()> {
        let py = slf.py();
        let f = slf.borrow().flaeche.clone_ref(py);
        f.bind(py).borrow().sperre_eintragen(py, slf.as_any());
        Ok(())
    }

    /// `pa.surface` - **dieselbe** Flaeche, die hineingereicht wurde.
    #[getter]
    fn surface(&self, py: Python<'_>) -> PyResult<Py<PySurface>> {
        self.pruefen()?;
        Ok(self.flaeche.clone_ref(py))
    }

    #[getter]
    fn shape(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        self.pruefen()?;
        let v: Vec<usize> = match self.a1 {
            None => vec![self.a0.n],
            Some(a) => vec![self.a0.n, a.n],
        };
        Ok(PyTuple::new(py, v)?.unbind())
    }

    #[getter]
    fn ndim(&self) -> PyResult<usize> {
        self.pruefen()?;
        Ok(if self.a1.is_none() { 1 } else { 2 })
    }

    /// Die **Bytezahl der Flaeche** - 1, 2, 3 oder 4.
    ///
    /// Hier stand einmal fest 4, mit der Begruendung "rustgame rechnet nur
    /// in ARGB8888". Das ist die Ablage, nicht die Auskunft: gemessen gibt
    /// `PixelArray(Surface((2,2), 0, 8)).itemsize` eine **1**.
    #[getter]
    fn itemsize(&self, py: Python<'_>) -> PyResult<usize> {
        self.pruefen()?;
        Ok(self.masz(py)?.0 as usize)
    }

    /// Gemessen bei 32 Bit und Breite 4: `(4, 16)`.
    ///
    /// Nach `[::2, :]` steht dort `(8, 16)` - der Schritt geht mit ein.
    /// Mit einer blossen Nummernliste war das nicht mehr ablesbar; siehe
    /// den Modulkopf.
    #[getter]
    fn strides(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        self.pruefen()?;
        let (bytes, pitch) = self.masz(py)?;
        let mut v = vec![Self::schritt(&self.a0, bytes, pitch)];
        if let Some(a) = self.a1 {
            v.push(Self::schritt(&a, bytes, pitch));
        }
        Ok(PyTuple::new(py, v)?.unbind())
    }

    /// Die Stelle des **Anfangspixels** im Speicher.
    ///
    /// Gerechnet mit der gemeldeten Zeilenlaenge, nicht mit der
    /// tatsaechlichen: rustgame legt jede Flaeche als ARGB32 ab, und eine
    /// 8-Bit-Flaeche hat hier keine Bytes, auf die man zeigen koennte.
    /// Geprueft wird von aussen ohnehin nur der **Abstand** zweier
    /// Sichten, und der stimmt damit genau.
    #[getter]
    fn _pixels_address(&self, py: Python<'_>) -> PyResult<usize> {
        self.pruefen()?;
        let (bytes, pitch) = self.masz(py)?;
        let basis = self.mit_kern(py, |k| k.pixel_zeiger())?;
        Ok((basis as i64 + self.by * pitch + self.bx * bytes) as usize)
    }

}

impl PyPixelArray {
    /// Die Puffer-Beschreibung dieses Feldes.
    ///
    /// Sie faellt aus dem Start/Schritt-Modell heraus, ohne dass etwas zu
    /// rechnen waere: `shape`, `strides` und `_pixels_address` stehen
    /// oben, und mehr braucht ein Puffer nicht.
    ///
    /// ## Die Grenze, und warum sie hier steht
    ///
    /// Ein Puffer sagt: **ab dieser Adresse liegen Pixel in diesem
    /// Format.** Bei vier Bytes je Pixel ist das wahr - rustgames Ablage
    /// ist ARGB8888, und genau das meldet die Sicht. Bei einem, zwei oder
    /// drei Bytes ist es nicht wahr: dort stuende im Speicher weiterhin ein
    /// ARGB-Wort, und ein Verbraucher, der auf Byte 0 einen Tafelindex
    /// erwartet, bekaeme das blaue Byte.
    ///
    /// Also gibt es dort keinen Puffer, sondern einen `BufferError`, der
    /// den Grund nennt. Das kostet vier Tests in `pixelarray_test`
    /// (`test_itemsize`, `test_flags`, `test_newbuf_2D`, `test_slicing`),
    /// und es ist die richtige Antwort: die Alternative waere eine
    /// Beschreibung, die stimmt, und Bytes, die es nicht tun.
    fn puffersicht(&self, py: Python<'_>) -> PyResult<crate::puffer::Sicht> {
        self.pruefen()?;
        let (bytes, pitch) = self.masz(py)?;
        let mut shape = vec![self.a0.n as isize];
        let mut strides = vec![Self::schritt(&self.a0, bytes, pitch)];
        if let Some(a) = self.a1 {
            shape.push(a.n as isize);
            strides.push(Self::schritt(&a, bytes, pitch));
        }
        let laenge = shape.iter().product::<isize>().max(0) as usize * bytes as usize;
        let vor = crate::puffer::Sicht {
            adresse: self._pixels_address(py)?,
            laenge,
            itemsize: bytes as usize,
            format: crate::puffer::fmt(match bytes {
                1 => "B",
                2 => "=H",
                3 => "3x",
                _ => "=I",
            }),
            shape,
            strides,
            schreibbar: true,
            solide: false,
        };
        // **Ein Stueck Speicher ist nicht dasselbe wie C-Reihenfolge.**
        //
        // Ein volles `PixelArray` hat `shape (w, h)` und
        // `strides (bpp, pitch)` - das ist **spaltenweise** lueckenlos,
        // nicht zeilenweise, genau wie bei der Flaechensicht. Wer nur
        // `c_zusammen()` fragt, lehnt `PyBUF_SIMPLE` auf einem
        // vollstaendigen Feld ab; `pixelarray_test.test_newbuf_2D`
        // verlangt es.
        //
        // Ein **geschnittenes** Feld (`ar_2D[x]`, eine Spalte mit
        // Schrittweite `pitch`) ist dagegen keines von beiden, und dort
        // scheitert `PyBUF_SIMPLE` weiterhin - `test_newbuf_1D` prueft es.
        Ok(crate::puffer::Sicht {
            solide: vor.c_zusammen() || vor.f_zusammen(),
            ..vor
        })
    }

}

#[pymethods]
impl PyPixelArray {
    /// `ar.__array_struct__` - dieselbe Beschreibung als C-Kasten.
    #[getter]
    fn __array_struct__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        crate::puffer::feldschnittstelle(py, &self.puffersicht(py)?)
    }

    /// `memoryview(ar)`, und damit auch `pixelcopy` und numpy.
    ///
    /// # Sicherheit
    ///
    /// Die Adresse zeigt in den Pixelspeicher der Flaeche, die dieses Feld
    /// gesperrt haelt - solange das `PixelArray` lebt, lebt sie auch. Die
    /// Zusagen stehen in [`crate::puffer`].
    // Reicht nur durch - siehe `#![deny(unsafe_code)]` in `lib.rs`.
    #[allow(unsafe_code)]
    unsafe fn __getbuffer__(
        slf: PyRef<'_, Self>,
        view: *mut pyo3::ffi::Py_buffer,
        flags: std::ffi::c_int,
    ) -> PyResult<()> {
        let py = slf.py();
        let sicht = slf.puffersicht(py)?;
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

    /// Das Woerterbuch, das numpy liest.
    #[getter]
    fn __array_interface__(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        self.pruefen()?;
        let bytes = self.masz(py)?.0;
        let d = PyDict::new(py);
        d.set_item("version", 3)?;
        d.set_item(
            "typestr",
            match bytes {
                1 => "|u1",
                2 => "<u2",
                3 => "<V3",
                _ => "<u4",
            },
        )?;
        d.set_item("shape", self.shape(py)?)?;
        d.set_item("strides", self.strides(py)?)?;
        d.set_item("data", (self._pixels_address(py)?, false))?;
        Ok(d.unbind())
    }

    fn __len__(&self) -> PyResult<usize> {
        self.pruefen()?;
        Ok(self.a0.n)
    }

    /// `pa[i]`, `pa[a:b]`, `pa[i, j]`, `pa[...]`.
    ///
    /// Gibt **`None`** bei einer leeren Scheibe und **die Sicht selbst**
    /// bei einem blossen `...` - beides gemessen, beides ungewoehnlich
    /// genug, um es nicht zu erraten.
    fn __getitem__(slf: &Bound<'_, PyPixelArray>, i: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        slf.borrow().pruefen()?;
        // Ein blosses `...` ist die Einheitsabbildung: `ar[...] is ar`.
        if i.is(py.Ellipsis()) {
            return Ok(slf.clone().into_any().unbind());
        }
        let (erste, zweite) = zerlegen(i)?;
        let ich = slf.borrow();
        let (bx, by, a0, a1) = match ich.geschnitten(py, &erste, zweite.as_ref())? {
            None => return Ok(py.None()),
            Some(t) => t,
        };
        // Beide Achsen auf einen Punkt zusammengefallen - ein Pixelwert.
        if a0.is_none() {
            let v = ich.mit_kern(py, |k| k.gepackt_an((bx as i32, by as i32)))?;
            return Ok(v.into_pyobject(py)?.into_any().unbind());
        }
        let sicht = ich.sicht(py, bx, by, a0.unwrap(), a1);
        drop(ich);
        Self::wie_typ(slf, sicht)
    }

    fn __setitem__(
        slf: &Bound<'_, PyPixelArray>,
        i: &Bound<'_, PyAny>,
        wert: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let py = slf.py();
        slf.borrow().pruefen()?;
        let ich = slf.borrow();
        if i.is(py.Ellipsis()) {
            return ich.belegen(py, ich.bx, ich.by, Some(ich.a0), ich.a1, wert);
        }
        let (erste, zweite) = zerlegen(i)?;
        let (bx, by, a0, a1) = match ich.geschnitten(py, &erste, zweite.as_ref())? {
            None => return Ok(()),
            Some(t) => t,
        };
        ich.belegen(py, bx, by, a0, a1, wert)
    }

    /// `for spalte in pa` - und nach `close()` ein `ValueError`.
    ///
    /// Ohne eigenen `__iter__` faellt Python auf die Folgenschnittstelle
    /// zurueck, die PyO3 fuer `__getitem__` gar nicht setzt; `iter(pa)`
    /// gaebe dann einen `TypeError` statt der gemessenen Meldung.
    fn __iter__(slf: &Bound<'_, PyPixelArray>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        slf.borrow().pruefen()?;
        let n = slf.borrow().a0.n;
        let liste = PyList::empty(py);
        for i in 0..n {
            let nr = i.into_pyobject(py)?;
            liste.append(Self::__getitem__(slf, nr.as_any())?)?;
        }
        Ok(liste.try_iter()?.into_any().unbind())
    }

    /// `farbe in feld` - kommt dieser Wert irgendwo vor?
    fn __contains__(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<bool> {
        self.pruefen()?;
        // **Gepackt vergleichen, nicht in ARGB.** Gemessen ist
        // `0x0000FF in ar` auf einer weiss-schwarzen 8-Bit-Flaeche
        // **falsch**, obwohl Weiss darin vorkommt: 0xFF ist der Index 255,
        // und Weiss liegt in pygames Standardtafel auf 127.
        let gesucht = {
            let s = self.flaeche.bind(py).borrow();
            let k = s.leihen()?;
            Self::gesuchter_wert(&k, o)?
        };
        let alle = self.werte(py)?;
        Ok(alle.iter().any(|z| z.contains(&gesucht)))
    }

    /// Eine neue Flaeche mit dem Inhalt dieser Sicht.
    fn make_surface(&self, py: Python<'_>) -> PyResult<PySurface> {
        self.pruefen()?;
        Ok(PySurface::neu(self.als_flaeche(py)?))
    }

    /// Vertauscht die Achsen.
    ///
    /// Bei einer eindimensionalen Sicht kommt `shape (1, n)` heraus - die
    /// alte Achse wandert nach hinten, und vorne entsteht eine Achse der
    /// Laenge eins. Gemessen; mit zwei Nummernlisten war das nicht
    /// darstellbar.
    fn transpose(slf: &Bound<'_, PyPixelArray>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let ich = slf.borrow();
        ich.pruefen()?;
        let sicht = match ich.a1 {
            Some(a) => ich.sicht(py, ich.bx, ich.by, a, Some(ich.a0)),
            None => ich.sicht(py, ich.bx, ich.by, Achse::neu(1, 0, 0), Some(ich.a0)),
        };
        drop(ich);
        Self::wie_typ(slf, sicht)
    }

    /// **Weiss** wo die Farbe steht, sonst schwarz - gemessen.
    #[pyo3(signature = (color, distance=0.0, weights=None))]
    fn extract(
        slf: &Bound<'_, PyPixelArray>,
        color: &Bound<'_, PyAny>,
        distance: f64,
        weights: Option<(f64, f64, f64)>,
    ) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let ich = slf.borrow();
        ich.pruefen()?;
        let gesucht = ich.ein_wert(py, color)?;
        let g = gewichte(weights);
        let roh = ich.roh(py)?;
        let sicht = ich.maskensicht(py, |i, j| nah(roh[i][j], gesucht, distance, g))?;
        drop(ich);
        Self::wie_typ(slf, sicht)
    }

    /// **Weiss wo gleich**, schwarz wo verschieden - gemessen, und die
    /// Richtung ist die ueberraschende: `compare` markiert nicht die
    /// Unterschiede, sondern die Uebereinstimmungen.
    #[pyo3(signature = (other, distance=0.0, weights=None))]
    fn compare(
        slf: &Bound<'_, PyPixelArray>,
        other: &Bound<'_, PyPixelArray>,
        distance: f64,
        weights: Option<(f64, f64, f64)>,
    ) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let ich = slf.borrow();
        ich.pruefen()?;
        let anderer = other.borrow();
        anderer.pruefen()?;
        if anderer.masse() != ich.masse() {
            return Err(PyValueError::new_err(
                "Operation not supported for arrays of different sizes",
            ));
        }
        let g = gewichte(weights);
        let a = ich.roh(py)?;
        let b = anderer.roh(py)?;
        let sicht = ich.maskensicht(py, |i, j| nah(a[i][j], b[i][j], distance, g))?;
        drop(anderer);
        drop(ich);
        Self::wie_typ(slf, sicht)
    }

    /// Ersetzt eine Farbe durch eine andere - an Ort und Stelle, gibt
    /// `None`.
    #[pyo3(signature = (color, repcolor, distance=0.0, weights=None))]
    fn replace(
        &self,
        py: Python<'_>,
        color: &Bound<'_, PyAny>,
        repcolor: &Bound<'_, PyAny>,
        distance: f64,
        weights: Option<(f64, f64, f64)>,
    ) -> PyResult<()> {
        self.pruefen()?;
        let gesucht = self.ein_wert(py, color)?;
        let neu = self.ein_wert(py, repcolor)?;
        let g = gewichte(weights);
        let (n0, n1) = self.masse();
        let s = self.flaeche.bind(py).borrow();
        let mut k = s.leihen_mut()?;
        for i in 0..n0 {
            for j in 0..n1 {
                let stelle = self.stelle(i, j);
                if nah(k.gepackt_an(stelle), gesucht, distance, g) {
                    k.setz_gepackt(stelle, neu);
                }
            }
        }
        Ok(())
    }

    /// Gibt die Sicht auf - danach scheitert jeder Zugriff, `close()`
    /// eingeschlossen.
    fn close(slf: &Bound<'_, PyPixelArray>) -> PyResult<()> {
        let py = slf.py();
        slf.borrow().pruefen()?;
        // Erst die Sperre loesen, dann schliessen - danach ist
        // `slf.borrow()` fuer den Rest nicht mehr noetig.
        let f = slf.borrow().flaeche.clone_ref(py);
        let ich: Py<PyAny> = slf.clone().into_any().unbind();
        f.bind(py).borrow().sperre_loesen(py, &ich);
        let mut s = slf.borrow_mut();
        s.geschlossen = true;
        Ok(())
    }

    fn __enter__(slf: &Bound<'_, PyPixelArray>) -> PyResult<Py<PyAny>> {
        slf.borrow().pruefen()?;
        Ok(slf.clone().into_any().unbind())
    }

    #[pyo3(signature = (*_args))]
    fn __exit__(slf: &Bound<'_, PyPixelArray>, _args: &Bound<'_, PyTuple>) -> PyResult<bool> {
        Self::close(slf)?;
        Ok(false)
    }

    /// `repr(pa)` zeigt die **Pixel**, nicht die Masse.
    ///
    /// Gemessen bei `Surface((3,1), SRCALPHA, 16)` nach `ar[...] = 42`:
    ///
    /// ```text
    /// PixelArray([
    ///   [42, 42, 42]]
    /// )
    /// ```
    ///
    /// Eine Zeile je **y**, die Werte darin ueber x - und der Name kommt
    /// vom tatsaechlichen Typ, damit eine Unterklasse ihren eigenen zeigt.
    fn __repr__(slf: &Bound<'_, PyPixelArray>) -> PyResult<String> {
        let py = slf.py();
        let name = slf.get_type().name()?.to_string();
        let ich = slf.borrow();
        if ich.geschlossen {
            return Ok(format!("<{name}(closed)>"));
        }
        let w = ich.werte(py)?;
        let (n0, n1) = ich.masse();
        let zeile = |j: usize| {
            let v: Vec<String> = (0..n0).map(|i| w[i][j].to_string()).collect();
            format!("[{}]", v.join(", "))
        };
        if ich.a1.is_none() {
            return Ok(format!("{name}(\n  {}\n)", zeile(0)));
        }
        let zeilen: Vec<String> = (0..n1).map(|j| format!("  {}", zeile(j))).collect();
        Ok(format!("{name}([\n{}]\n)", zeilen.join("\n")))
    }
}

impl PyPixelArray {
    /// Alle **rohen** Pixelwerte der Sicht als `[i][j]` - fuer den
    /// Farbvergleich, der in ARGB rechnet.
    fn roh(&self, py: Python<'_>) -> PyResult<Vec<Vec<u32>>> {
        let (n0, n1) = self.masse();
        self.mit_kern(py, |k| {
            (0..n0)
                .map(|i| {
                    (0..n1)
                        .map(|j| k.gepackt_an(self.stelle(i, j)))
                        .collect()
                })
                .collect()
        })
    }

    /// Die Sicht nach einem Index - `None` heisst **leere Scheibe**.
    ///
    /// Das Ergebnis ist `(bx, by, a0, a1)`; ist `a0` leer, sind beide
    /// Achsen auf einen Punkt zusammengefallen und es geht um **ein**
    /// Pixel.
    #[allow(clippy::type_complexity)]
    fn geschnitten(
        &self,
        _py: Python<'_>,
        erste: &Bound<'_, PyAny>,
        zweite: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Option<(i64, i64, Option<Achse>, Option<Achse>)>> {
        let (mut bx, mut by) = (self.bx, self.by);
        let mut behalten: Vec<Achse> = Vec::new();

        let mut anwenden = |a: Achse, w: Wahl, behalten: &mut Vec<Achse>| -> bool {
            match w {
                Wahl::Alles => behalten.push(a),
                Wahl::Punkt(p) => {
                    bx += a.ax * p as i64;
                    by += a.ay * p as i64;
                }
                Wahl::Scheibe(start, schritt, n) => {
                    if n == 0 {
                        return false;
                    }
                    let (neu, dx, dy) = a.geschnitten(start, schritt, n);
                    bx += dx;
                    by += dy;
                    behalten.push(neu);
                }
            }
            true
        };

        let w0 = Self::wahl(erste, self.a0.n)?;
        if !anwenden(self.a0, w0, &mut behalten) {
            return Ok(None);
        }
        match (self.a1, zweite) {
            (Some(a), Some(z)) => {
                let w1 = Self::wahl(z, a.n)?;
                if !anwenden(a, w1, &mut behalten) {
                    return Ok(None);
                }
            }
            (Some(a), None) => behalten.push(a),
            (None, Some(_)) => {
                return Err(PyIndexError::new_err("too many indices for array"));
            }
            (None, None) => {}
        }
        let mut it = behalten.into_iter();
        Ok(Some((bx, by, it.next(), it.next())))
    }

    /// Schreibt einen Wert in eine (Teil-)Sicht.
    fn belegen(
        &self,
        py: Python<'_>,
        bx: i64,
        by: i64,
        a0: Option<Achse>,
        a1: Option<Achse>,
        wert: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        // Die Zielsicht als eigenes Objekt - dann gelten `stelle` und
        // `masse` unveraendert.
        let ziel = PyPixelArray {
            flaeche: self.flaeche.clone_ref(py),
            bx,
            by,
            a0: a0.unwrap_or(Achse::neu(1, 0, 0)),
            a1,
            geschlossen: false,
        };
        let (n0, n1) = ziel.masse();

        // 1) Eine andere Sicht - Feld auf Feld, mit Verbreiterung bei
        //    Achsen der Laenge eins.
        if let Ok(q) = wert.downcast::<PyPixelArray>() {
            let quelle = q.borrow();
            quelle.pruefen()?;
            let (m0, m1) = quelle.masse();
            if (m0 != n0 && m0 != 1) || (m1 != n1 && m1 != 1) {
                return Err(PyValueError::new_err("array sizes do not match"));
            }
            // **Erst lesen, dann schreiben.** `ar[:, :] = ar[::-1, :]`
            // liegt auf demselben Speicher; wer Pixel fuer Pixel schiebt,
            // spiegelt die erste Haelfte und kopiert die gespiegelte
            // zurueck.
            let gelesen = quelle.roh(py)?;
            let s = self.flaeche.bind(py).borrow();
            let mut k = s.leihen_mut()?;
            for i in 0..n0 {
                for j in 0..n1 {
                    let v = gelesen[if m0 == 1 { 0 } else { i }][if m1 == 1 { 0 } else { j }];
                    k.setz_gepackt(ziel.stelle(i, j), v);
                }
            }
            return Ok(());
        }

        // 2) Eine Zahl, ein Tupel oder eine `Color` - **ein** Wert.
        if let Some(v) = self.einzelwert(py, wert)? {
            let s = self.flaeche.bind(py).borrow();
            let mut k = s.leihen_mut()?;
            for i in 0..n0 {
                for j in 0..n1 {
                    k.setz_gepackt(ziel.stelle(i, j), v);
                }
            }
            return Ok(());
        }

        // 3) Sonst eine **Folge** von Pixelwerten, einer je erster Achse.
        if let Ok(folge) = wert.try_iter() {
            let stuecke: Vec<Bound<'_, PyAny>> = folge.collect::<PyResult<_>>()?;
            if stuecke.len() != n0 {
                return Err(PyValueError::new_err("sequence size mismatch"));
            }
            let s = self.flaeche.bind(py).borrow();
            let mut k = s.leihen_mut()?;
            let mut werte = Vec::with_capacity(n0);
            for st in &stuecke {
                werte.push(Self::wert_mit(&k, st)?);
            }
            for (i, wert) in werte.iter().enumerate() {
                for j in 0..n1 {
                    k.setz_gepackt(ziel.stelle(i, j), *wert);
                }
            }
            return Ok(());
        }
        Err(PyValueError::new_err("invalid color argument"))
    }
}

/// Zerlegt einen Index in bis zu zwei Achsenangaben.
///
/// `pa[i]` und `pa[i,]` sind dasselbe - gemessen, `len(ar[:,])` ist 6 wie
/// `len(ar[:])`.
fn zerlegen<'p>(
    i: &Bound<'p, PyAny>,
) -> PyResult<(Bound<'p, PyAny>, Option<Bound<'p, PyAny>>)> {
    if let Ok(t) = i.downcast::<PyTuple>() {
        return match t.len() {
            0 => Err(PyIndexError::new_err("invalid index")),
            1 => Ok((t.get_item(0)?, None)),
            2 => Ok((t.get_item(0)?, Some(t.get_item(1)?))),
            _ => Err(PyIndexError::new_err("too many indices for array")),
        };
    }
    Ok((i.clone(), None))
}

/// Die Gewichte fuer den Farbabstand.
///
/// pygames Vorgabe ist die uebliche Helligkeitsgewichtung - dieselbe,
/// die `transform.grayscale` benutzt.
fn gewichte(w: Option<(f64, f64, f64)>) -> (f64, f64, f64) {
    w.unwrap_or((0.299, 0.587, 0.114))
}

/// Sind zwei abgebildete Pixel "nah genug"?
///
/// Bei `distance == 0` ist das ein **glatter Vergleich der Zahlen** und
/// keine Rechnung - so trifft es auch Flaechen, deren viertes Byte
/// belegt ist.
fn nah(a: u32, b: u32, distance: f64, g: (f64, f64, f64)) -> bool {
    if distance <= 0.0 {
        return a == b;
    }
    let teil = |v: u32, s: u32| ((v >> s) & 0xFF) as f64 / 255.0;
    let (dr, dg, db) = (
        teil(a, 16) - teil(b, 16),
        teil(a, 8) - teil(b, 8),
        teil(a, 0) - teil(b, 0),
    );
    (dr * dr * g.0 + dg * dg * g.1 + db * db * g.2) <= distance * distance
}
