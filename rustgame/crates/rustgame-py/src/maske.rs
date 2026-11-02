//! maske.rs
//! ========
//! `pygame.mask`.
//!
//! ## Drei gemessene Kleinigkeiten
//!
//! | Frage | pygame-ce 2.5.7 |
//! |---|---|
//! | `m.get_at(p)` | **1** oder **0**, kein `True`/`False` |
//! | `m.overlap(...)` ohne Treffer | `None` |
//! | `m.to_surface()` ohne Farben | gesetzt weiss, leer `(0,0,0,255)`, `SRCALPHA` |
//!
//! Das erste ist die Falle: `m.get_at(p) is True` ist in pygame falsch,
//! obwohl `if m.get_at(p)` stimmt. Ein `bool` durchzureichen waere die
//! naheliegende und falsche Uebersetzung.
//!
//! ## Was die Huelle nicht kann, weil der Kern es nicht hat
//!
//! `to_surface` nimmt in pygame ausserdem `surface`, `setsurface`,
//! `unsetsurface` und `dest`; `connected_component` nimmt dort eine
//! Stelle. Beides fehlt in [`rustgame_math::Mask`] - siehe Bericht.

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

use rustgame_math::mask;
use rustgame_math::Mask;

use crate::flaeche::PySurface;
use crate::rechteck::PyRect;
use crate::rechteck::rect_aus_objekt;
use crate::umwandlung::{rect_aus_opt, versatz_aus, versatz_aus_wort};

/// `pygame.mask.Mask`.
#[pyclass(name = "Mask", module = "rustgame_py.mask", subclass)]
#[derive(Clone)]
pub struct PyMask(Mask);

impl PyMask {
    /// Aus einer Kernmaske.
    pub fn neu(m: Mask) -> PyMask {
        PyMask(m)
    }

    /// Liegt die Stelle in der Maske? Sonst `IndexError` - Wortlaut
    /// gemessen: `9, 9 is out of bounds`.
    fn im_bereich(&self, pos: (i32, i32)) -> PyResult<()> {
        let (w, h) = self.0.get_size();
        if pos.0 < 0 || pos.1 < 0 || pos.0 >= w as i32 || pos.1 >= h as i32 {
            return Err(pyo3::exceptions::PyIndexError::new_err(format!(
                "{}, {} is out of bounds",
                pos.0, pos.1
            )));
        }
        Ok(())
    }
}

impl PyMask {
    /// Die gepackten Bitworte als Puffer-Beschreibung.
    ///
    /// Gemessen an echtem pygame, und die Zahlen sagen die Anordnung:
    ///
    /// | Maske | `shape` | `strides` | `itemsize` |
    /// |---|---|---|---|
    /// | 1000 x 100 | `(32, 100)` | `(400, 4)` | 4 |
    /// | 5 x 3 | `(1, 3)` | `(12, 4)` | 4 |
    ///
    /// Ein Schritt in `y` sind vier Bytes, ein Schritt in der Wortspalte
    /// hundert davon - die Worte liegen **spaltenweise**. Das ist SDLs
    /// `bitmask_t`, und `rustgame-math` legt sie seit dem Puffer genauso
    /// ab; vorher stand dort ein Byte je Stelle.
    ///
    /// Zwei Kleinigkeiten stecken im Format `I`, und beide sind gemessen.
    ///
    /// pygame meldet `L`, also `unsigned long` - unter Windows vier Bytes,
    /// unter Linux acht. Dieselbe Maske haette dort ein anderes
    /// `itemsize`. rustgame legt echte `u32` ab, also ist `I` die Aussage,
    /// die auf jeder Maschine stimmt.
    ///
    /// Und es muss ein **maschineneigenes** Format sein, kein `=I`:
    /// `memoryview` laesst sich nur damit mehrdimensional indizieren. Mit
    /// `=I` gab `view[col, y]` ein
    /// `NotImplementedError: memoryview: unsupported format =I`. Die
    /// Flaechensichten duerfen `=I` melden, weil dort niemand so
    /// zugreift - `mask_test.test_buffer_interface` tut es.
    ///
    /// **Welcher Buchstabe, sagt die Plattform.** Gemessen meldet pygame
    /// hier `L`, und das stimmt dort auf jedem System: seine Maske legt
    /// `unsigned long` ab, also vier Bytes unter Windows und acht unter
    /// Linux. rustgame legt echte `u32` ab - unter Windows ist `L`
    /// damit **derselbe** Typ (vier Bytes) und die Aussage gleich; wo ein
    /// `long` acht Bytes hat, waere `L` eine falsche Zusage, und dort
    /// bleibt es `I`. Hier stand immer `I`, und der Vermerk daneben
    /// begruendete nur das Fehlen des `=`.
    ///
    /// Der Unterschied in `itemsize` auf einem 64-Bit-Linux (vier gegen
    /// acht) bleibt - er steckt in der Maske selbst, nicht im Namen.
    fn sicht(&self) -> crate::puffer::Sicht {
        let (w, h) = self.0.get_size();
        let spalten = (w as usize).div_ceil(32) as isize;
        let hi = h as isize;
        crate::puffer::Sicht {
            adresse: self.0.worte().as_ptr() as usize,
            laenge: spalten as usize * h as usize * 4,
            itemsize: 4,
            format: crate::puffer::fmt(if cfg!(windows) { "L" } else { "I" }),
            shape: vec![spalten, hi],
            strides: vec![hi * 4, 4],
            schreibbar: true,
            solide: true,
        }
    }
}

// **Ausserhalb von `#[pymethods]`.** PyO3 nimmt jede Funktion in
// diesem Block fuer eine Python-Methode und versucht, ihre
// Argumente aus Python zu lesen; ein Helfer ohne `&self` wird
// damit zu einer statischen Methode mit unmoeglichen Typen.
impl PyMask {
    /// Die eigentliche Arbeit - **einmal** fuer `__new__` und `__init__`.
    ///
    /// `__new__` nimmt `*args`, weil nur so pygames Wortlaut fuer ein
    /// fehlendes `size` herauskommt; `__init__` nimmt die zwei Werte
    /// benannt, weil eine Unterklasse sie so weitergibt. Beide landen
    /// hier.
    fn bauen(size: (i64, i64), fill: bool) -> PyResult<PyMask> {
        if size.0 < 0 || size.1 < 0 {
            return Err(PyValueError::new_err(
                "cannot create mask with negative size",
            ));
        }
        Ok(PyMask(Mask::new((size.0 as u32, size.1 as u32), fill)))
    }
}

#[pymethods]
impl PyMask {
    /// `memoryview(mask)` - die Bitworte, wie sie liegen.
    ///
    /// # Sicherheit
    ///
    /// Die Adresse ist die des Wortfeldes. Eine Maske wechselt nach dem
    /// Anlegen nie ihre Groesse, also bleibt sie gueltig, solange die Maske
    /// lebt - und genau so lange haelt `fuellen` sie ueber `view.obj` fest.
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

    // **Kein `__array_struct__`.** Gemessen hat pygames `Mask` **keine**
    // Feldschnittstelle - weder `__array_interface__` noch
    // `__array_struct__` -, und `numpy.array(maske)` gibt dort die Form
    // `(1, 4)`, also das Objekt selbst in einem Feld. Hier stand der
    // C-Kasten, und damit baute numpy ein echtes Feld: dieselbe Zeile,
    // zwei verschiedene Ergebnisse. Gefunden von der Probe
    // `Klassendoppel`.

    /// `Mask(size, fill=False)`.
    ///
    /// Die Groesse kommt als `i64` herein und nicht als `u32`: gemessen
    /// gibt `Mask((-1, -1))` in pygame
    /// `ValueError: cannot create mask with negative size`, und ein
    /// `u32`-Beiwert haette daraus einen `OverflowError` des
    /// Argumentlesers gemacht - eine andere Ausnahme mit anderem Text.
    #[new]
    // **`*args`, damit der Satz stimmt.** Gemessen gibt `Mask()` in
    // pygame `TypeError: function missing required argument 'size'
    // (pos 1)` - der Satz von `PyArg_ParseTupleAndKeywords`. PyO3 sagte
    // `Mask.__new__() missing 1 required positional argument: 'size'`.
    // Die Sorte stimmte, der Wortlaut nicht.
    // **`*args` und zwei Schluesselwoerter.** pygame nimmt beides:
    // `Mask((4,4))` und `Mask(size=(4,4))` gehen, `Mask()` gibt
    // `TypeError: function missing required argument 'size' (pos 1)`.
    // Mit benannten Parametern allein kaeme PyO3s Satz heraus, mit
    // `*args` allein ginge das Schluesselwort verloren - also beides.
    #[pyo3(signature = (*args, size=None, fill=false))]
    fn new(
        args: &Bound<'_, pyo3::types::PyTuple>,
        size: Option<(i64, i64)>,
        fill: bool,
    ) -> PyResult<PyMask> {
        if args.len() > 2 {
            return Err(PyTypeError::new_err(
                "function takes at most 2 arguments",
            ));
        }
        let size = match (args.is_empty(), size) {
            // **pygames eigener Satz.** Gemessen gibt `Mask('x')` dort
            // `TypeError: size must be two numbers`; hier kam PyO3s Satz
            // ueber das Umwandeln eines Tupels.
            (false, _) => args
                .get_item(0)?
                .extract()
                .map_err(|_| PyTypeError::new_err("size must be two numbers"))?,
            (true, Some(s)) => s,
            (true, None) => {
                return Err(PyTypeError::new_err(
                    "function missing required argument 'size' (pos 1)",
                ))
            }
        };
        let fill = if args.len() == 2 {
            args.get_item(1)?.is_truthy()?
        } else {
            fill
        };
        Self::bauen(size, fill)
    }


    /// Setzt die Maske **neu** - `m.__init__((9, 9))` macht aus einer
    /// 5x3 eine 9x9.
    ///
    /// PyO3-Klassen haben von sich aus nur `__new__`; pygames Typen
    /// haben zusaetzlich ein echtes `__init__`, und das ist **kein
    /// Leerlauf**. Sichtbar wird der Unterschied, sobald jemand ableitet
    /// und `super().__init__(*args)` ruft - dann landet der Aufruf sonst
    /// auf `object.__init__` und gibt
    /// `TypeError: object.__init__() takes exactly one argument`.
    ///
    /// Gefunden hat das pygames **eigene** Testsuite: 27 Tests in
    /// `mask_test.py` scheiterten daran, alle an derselben Zeile.
    #[pyo3(signature = (size, fill=false))]
    fn __init__(&mut self, size: (i64, i64), fill: bool) -> PyResult<()> {
        *self = PyMask::bauen(size, fill)?;
        Ok(())
    }

    fn get_size(&self) -> (u32, u32) {
        self.0.get_size()
    }
    /// `get_rect(**kwargs)` - mit denselben Ankerwoertern wie
    /// `Surface.get_rect`.
    ///
    /// Gemessen: `m.get_rect(center=(9, 9))` gibt `(6, 7, 6, 4)`, und
    /// ein unbekanntes Wort ist ein **`AttributeError`**, kein
    /// `TypeError` - pygame setzt die Woerter als Felder am Rechteck,
    /// also kommt dessen Fehler durch.
    #[pyo3(signature = (*args, **kwargs))]
    fn get_rect(
        &self,
        args: &Bound<'_, pyo3::types::PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyRect> {
        // **`supports`, nicht `accepts`** - pygame schreibt bei der Maske
        // `get_rect only supports keyword arguments` und bei der Flaeche
        // `only accepts`. Zwei Saetze fuer dieselbe Sache; gemessen sind
        // beide.
        if !args.is_empty() {
            return Err(PyTypeError::new_err(
                "get_rect only supports keyword arguments",
            ));
        }
        let mut r = PyRect::neu(self.0.get_rect());
        r.aus_kwargs(kwargs)?;
        Ok(r)
    }

    /// **1 oder 0**, nicht `True`/`False` - siehe Modulkopf.
    /// Ausserhalb der Maske gibt es einen **`IndexError`**, keine Null.
    ///
    /// Gemessen: `m.get_at((9, 9))` auf einer 5x3 sagt
    /// `IndexError: 9, 9 is out of bounds` - und dasselbe bei negativen
    /// Stellen. Der Kern gibt dort stillschweigend `false` zurueck; das
    /// ist fuer Rust richtig, aber ein Programm, das sich auf die
    /// Ausnahme verlaesst, liefe hier ins Leere.
    fn get_at(&self, pos: &Bound<'_, PyAny>) -> PyResult<u8> {
        let pos = versatz_aus_wort(pos, "pos")?;
        self.im_bereich(pos)?;
        Ok(self.0.get_at(pos) as u8)
    }
    /// `set_at(pos, value=1)` - siehe [`get_at`](Self::get_at).
    #[pyo3(signature = (pos, value=1))]
    fn set_at(&mut self, pos: &Bound<'_, PyAny>, value: i64) -> PyResult<()> {
        let pos = versatz_aus_wort(pos, "pos")?;
        self.im_bereich(pos)?;
        self.0.set_at(pos, value != 0);
        Ok(())
    }

    fn count(&self) -> usize {
        self.0.count()
    }
    fn fill(&mut self) {
        self.0.fill();
    }
    fn clear(&mut self) {
        self.0.clear();
    }
    fn invert(&mut self) {
        self.0.invert();
    }
    /// Eine Kopie - **vom selben Typ wie das Original**.
    ///
    /// Gemessen: leitet jemand `Mask` ab, gibt `copy()` dort die
    /// **Unterklasse** zurueck, nicht die Grundklasse. `mask_test.py`
    /// prueft das an drei Stellen, und sie scheiterten alle mit
    /// `AttributeError: ... has no attribute 'test_attribute'` - weil
    /// die Kopie eine nackte `Mask` war, an die sich nichts anheften
    /// laesst.
    ///
    /// Gebaut wird ueber `type(self).__new__(type(self), groesse)`,
    /// nicht ueber `type(self)(groesse)`: pygame umgeht `__init__`
    /// ebenfalls (es ruft `tp_alloc`), und eine Unterklasse mit anderer
    /// Argumentliste wuerde sonst hier scheitern und dort nicht.
    fn __copy__(slf: &Bound<'_, PyMask>) -> PyResult<Py<PyAny>> {
        let typ = slf.get_type();
        let (vorlage, groesse) = {
            let m = slf.borrow();
            (m.0.clone(), m.0.get_size())
        };
        let neu = typ.call_method1("__new__", (&typ, groesse))?;
        {
            let mut z: PyRefMut<PyMask> = neu.extract()?;
            z.0 = vorlage;
        }
        Ok(neu.unbind())
    }

    /// `copy()` ruft das **`__copy__` der Instanz**, es kopiert nicht selbst.
    ///
    /// Das ist nicht dasselbe wie der umgekehrte Weg, und die Richtung
    /// entscheidet: schreibt eine Unterklasse nur `__copy__`, muss auch
    /// `copy()` darueber laufen. `mask.c` sagt es im Kommentar bei
    /// `mask_call_copy` (Zeile 108) ausdruecklich - "to allow subclasses
    /// that override the __copy__() method to also override the copy()
    /// method automatically".
    ///
    /// Hier war es zuerst andersherum aufgebaut: `copy` tat die Arbeit,
    /// `__copy__` reichte weiter. Das laesst `copy.copy(m)` richtig
    /// laufen, aber `m.copy()` greift an einem eigenen `__copy__` vorbei -
    /// zwei Tests in `mask_test.py` haben es gefunden.
    ///
    /// Der Aufruf geht **ueber Python**, nicht ueber `PyMask::__copy__`,
    /// sonst waere die Ableitung wieder umgangen.
    fn copy(slf: &Bound<'_, PyMask>) -> PyResult<Py<PyAny>> {
        Ok(slf.call_method0("__copy__")?.unbind())
    }

    /// Die erste gemeinsame Stelle - `None`, wenn es keine gibt.
    #[pyo3(signature = (other, offset=None))]
    fn overlap(
        &self,
        other: &PyMask,
        offset: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Option<(i32, i32)>> {
        Ok(self.0.overlap(&other.0, versatz_wahl(offset)?))
    }
    #[pyo3(signature = (other, offset=None))]
    fn overlap_area(&self, other: &PyMask, offset: Option<&Bound<'_, PyAny>>) -> PyResult<usize> {
        Ok(self.0.overlap_area(&other.0, versatz_wahl(offset)?))
    }
    #[pyo3(signature = (other, offset=None))]
    fn overlap_mask(&self, other: &PyMask, offset: Option<&Bound<'_, PyAny>>) -> PyResult<PyMask> {
        Ok(PyMask(self.0.overlap_mask(&other.0, versatz_wahl(offset)?)))
    }
    #[pyo3(signature = (other, offset=None))]
    fn draw(&mut self, other: &PyMask, offset: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        self.0.draw(&other.0, versatz_wahl(offset)?);
        Ok(())
    }
    #[pyo3(signature = (other, offset=None))]
    fn erase(&mut self, other: &PyMask, offset: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
        self.0.erase(&other.0, versatz_wahl(offset)?);
        Ok(())
    }

    /// Der Schwerpunkt der gesetzten Stellen, abgerundet.
    fn centroid(&self) -> (i32, i32) {
        self.0.centroid()
    }
    /// Der Winkel der Hauptachse, in Grad.
    fn angle(&self) -> f64 {
        self.0.angle()
    }
    fn get_bounding_rects(&self) -> Vec<PyRect> {
        self.0.get_bounding_rects().into_iter().map(PyRect::neu).collect()
    }
    /// `connected_components(minimum=0)` - groesster Bereich zuerst.
    /// Ein **negatives** `minimum` ist erlaubt und zaehlt wie 0 -
    /// gemessen: `connected_components(-1)` gibt dasselbe wie `(0)`.
    /// Als `usize` haette der Argumentleser dort einen `OverflowError`
    /// geworfen.
    #[pyo3(signature = (minimum=0))]
    fn connected_components(&self, minimum: i64) -> Vec<PyMask> {
        let minimum = minimum.max(0) as usize;
        self.0
            .connected_components(minimum)
            .into_iter()
            .map(PyMask)
            .collect()
    }
    /// `connected_component(pos=None)`.
    ///
    /// Ohne Stelle der **groesste** Bereich, mit Stelle der Bereich um
    /// **sie** herum. Gemessen: eine leere Stelle gibt eine leere Maske
    /// gleicher Groesse, keinen Fehler.
    ///
    /// Hier stand einmal "der Kern kennt das Argument nicht" - das
    /// stimmte, bis pygames eigene Testsuite fuenf Tests daran scheitern
    /// liess. Jetzt kennt er es.
    #[pyo3(signature = (pos=None))]
    fn connected_component(&self, pos: Option<&Bound<'_, PyAny>>) -> PyResult<PyMask> {
        Ok(match pos {
            None => PyMask(self.0.connected_component()),
            Some(o) if o.is_none() => PyMask(self.0.connected_component()),
            Some(o) => {
                let p = versatz_aus_wort(o, "pos")?;
                self.im_bereich(p)?;
                PyMask(self.0.connected_component_bei(p))
            }
        })
    }
    /// `scale(size)` - **nur positionell**.
    ///
    /// Gemessen: `m.scale(size=(2, 2))` gibt
    /// `TypeError: function missing required argument 'scale' (pos 1)`,
    /// das Wort heisst dort also nicht einmal `size`. Und eine negative
    /// Groesse ist ein `ValueError: cannot scale mask to negative size`,
    /// kein `OverflowError` des Argumentlesers - deshalb `i64`.
    #[pyo3(signature = (scale))]
    fn scale(&self, scale: &Bound<'_, PyAny>) -> PyResult<PyMask> {
        // Das Wort heisst **`scale`**, nicht `size` - und es ist ein
        // ganz gewoehnliches Schluesselwort, kein nur-positionelles.
        //
        // Meine erste Messung hatte `m.scale(size=(2,2))` versucht, das
        // `TypeError: function missing required argument 'scale' (pos 1)`
        // gab - daraus hatte ich geschlossen, der Wert sei
        // nur-positionell. Falsch geschlossen: `mask_test.py` ruft
        // `original_mask.scale(scale=expected_size)`, und das geht. Die
        // Fehlermeldung nannte nur den **richtigen** Namen, nicht die
        // Aufrufart.
        //
        // Es nimmt auch einen `Vector2`.
        let size = paar_aus(scale)?;
        if size.0 < 0 || size.1 < 0 {
            return Err(PyValueError::new_err(
                "cannot scale mask to negative size",
            ));
        }
        Ok(PyMask(self.0.scale((size.0 as u32, size.1 as u32))))
    }

    /// `outline(every=1)` - die aeussere Kontur im Uhrzeigersinn.
    ///
    /// Eine Liste aus `(x, y)`. Der Startpunkt steht vorn und bei
    /// `every == 1` noch einmal hinten. Gemessen: `every` mit `0` oder
    /// negativ ist **kein** Fehler - es bleibt dann nur der Startpunkt
    /// uebrig, wie bei einem `every` groesser als der Umfang.
    #[pyo3(signature = (every=1))]
    fn outline(&self, every: i32) -> Vec<(i32, i32)> {
        self.0.outline(every)
    }

    /// `convolve(other, output=None, offset=(0,0))` - die Faltung.
    ///
    /// Ohne `output` entsteht eine neue Maske, je Richtung `a + b - 1`
    /// gross. Mit `output` wird **genau diese** Maske beschrieben und
    /// zurueckgegeben; gemessen ist `m.convolve(o, ziel) is ziel` wahr,
    /// und was in `ziel` schon stand, bleibt stehen.
    ///
    /// `output` darf dieselbe Maske sein wie `self` oder `other`. pygame
    /// liest die Quelle dann waehrend des Schreibens weiter, und das
    /// Ergebnis haengt an seiner 32-Bit-Wortbreite: gemessen an einer
    /// `80x1`-Maske mit einer einzigen Stelle und `output=self` werden
    /// aus einem Punkt bei `29` **acht** gesetzte Stellen, aus einem bei
    /// `30` nur **sechs**. Das ist ein Ueberlappungsartefakt, kein
    /// Verhalten - hier wird darum zuerst abgeschrieben, die Quelle
    /// bleibt also, wie sie zu Beginn war.
    #[pyo3(signature = (other, output=None, offset=None))]
    fn convolve(
        slf: &Bound<'_, PyMask>,
        other: &Bound<'_, PyMask>,
        output: Option<Bound<'_, PyMask>>,
        offset: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyMask>> {
        let offset = versatz_wahl(offset)?;
        let ziel = match output {
            None => {
                let neu = slf.borrow().0.convolve(&other.borrow().0, offset);
                return Py::new(slf.py(), PyMask(neu));
            }
            Some(z) => z,
        };
        if ziel.is(slf) || ziel.is(other) {
            // Ziel ist zugleich Quelle - erst abschreiben, dann schreiben.
            let a = slf.borrow().0.clone();
            let b = other.borrow().0.clone();
            a.convolve_into(&b, &mut ziel.borrow_mut().0, offset);
        } else {
            let a = slf.borrow();
            let b = other.borrow();
            a.0.convolve_into(&b.0, &mut ziel.borrow_mut().0, offset);
        }
        Ok(ziel.unbind())
    }

    /// Die Maske als Flaeche.
    ///
    /// Voreinstellung wie gemessen: gesetzt `(255,255,255,255)`, leer
    /// `(0,0,0,255)`, Ergebnis mit `SRCALPHA`. `None` laesst die jeweilige
    /// Seite durchsichtig.
    ///
    /// **Nicht angegeben** und **ausdruecklich `None`** bedeuten hier
    /// Verschiedenes: ohne Angabe gilt die Voreinstellung, mit `None`
    /// bleibt die Seite durchsichtig. Eine gewoehnliche `Option` wuerde
    /// beides zu `None` verschmelzen, und `to_surface(unsetcolor=None)`
    /// waere dann schwarz statt leer - gemessen ist es `(0, 0, 0, 0)`.
    /// Darum wird die Argumentliste hier von Hand gelesen.
    ///
    /// Hier stand: "ausgewertet werden die beiden Farben, der Rest
    /// fehlt im Kern". Das stimmte, bis pygames eigene Testsuite
    /// dreissig Tests daran scheitern liess. Jetzt wird alles
    /// ausgewertet.
    ///
    /// `to_surface(surface=None, setsurface=None, unsetsurface=None,
    /// setcolor=(255,255,255,255), unsetcolor=(0,0,0,255), dest=(0,0),
    /// area=None)`.
    ///
    /// Sieben Werte, jeder mit eigener Wirkung - alles an pygame-ce
    /// 2.5.7 gemessen:
    ///
    /// ```text
    /// ohne alles            schwarze SRCALPHA-Flaeche, gesetzt weiss
    /// unsetcolor=None       leere Stellen bleiben **unberuehrt**
    /// setsurface            statt setcolor der Pixel **derselben Stelle**
    /// dest                  verschiebt, wo die Maske im Ziel landet
    /// area                  nimmt nur diesen Ausschnitt der **Maske**
    /// ```
    ///
    /// **Und die Flaeche kommt selbst zurueck**, wenn eine gereicht
    /// wurde: `m.to_surface(s) is s` ist wahr. Hier stand vorher immer
    /// eine neue - sechs Tests in `mask_test.py` haben genau darauf
    /// bestanden, und zwar mit `assertIs`.
    #[pyo3(signature = (*args, **kwargs))]
    fn to_surface(
        &self,
        py: Python<'_>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        // Gemessen: ein unbekanntes Schluesselwort gibt
        // `TypeError: this function got an unexpected keyword argument
        // 'foo'`. Ohne diese Pruefung wird es stillschweigend
        // verschluckt - und ein Tippfehler im Aufruf bliebe unbemerkt.
        if let Some(kw) = kwargs {
            const ERLAUBT: [&str; 7] = [
                "surface",
                "setsurface",
                "unsetsurface",
                "setcolor",
                "unsetcolor",
                "dest",
                "area",
            ];
            for schluessel in kw.keys() {
                let name: String = schluessel.extract()?;
                if !ERLAUBT.contains(&name.as_str()) {
                    return Err(PyTypeError::new_err(format!(
                        "this function got an unexpected keyword argument '{name}'"
                    )));
                }
            }
        }
        // **Die Farben werden erst aufgeloest, wenn die Zielflaeche
        // feststeht.** Eine blanke Zahl ist in pygame der *abgebildete
        // Pixelwert dieser Flaeche*, kein RGBA-Wort - gemessen:
        // `to_surface(setcolor=5)` gibt auf einer ARGB-Flaeche
        // `(0, 0, 5, 0)`. Waere die Farbe vorher gelesen, waere sie
        // gegen die falsche Flaeche abgebildet.
        //
        // Gelesen wird mit `zeichnen::farbe_fuer` - demselben Helfer, den
        // `pygame.draw` benutzt. Dort steckt auch die Aufteilung der
        // Fehlerarten (`ValueError` fuer eine Folge, die keine Farbe
        // hergibt, `TypeError` fuer alles andere), an der sechs Tests in
        // `mask_test.py` gescheitert sind.
        let setfarbe_obj = wahl(args, kwargs, 3, "setcolor")?;
        let unsetfarbe_obj = wahl(args, kwargs, 4, "unsetcolor")?;
        let setfarbe_fehlt = args.len() > 3
            || kwargs.is_some_and(|k| k.get_item("setcolor").is_ok_and(|v| v.is_some()));
        let unsetfarbe_fehlt = args.len() > 4
            || kwargs.is_some_and(|k| k.get_item("unsetcolor").is_ok_and(|v| v.is_some()));
        let ziel_obj = wahl(args, kwargs, 0, "surface")?;
        let setflaeche = wahl(args, kwargs, 1, "setsurface")?;
        let unsetflaeche = wahl(args, kwargs, 2, "unsetsurface")?;
        let dest_obj = wahl(args, kwargs, 5, "dest")?;
        let area_obj = wahl(args, kwargs, 6, "area")?;

        // **Erst als Punkt lesen, dann als Rechteck.** Umgekehrt
        // scheitert `dest=(1, 1)`: seit `rect_aus` an den zeilengetreuen
        // Wandler durchreicht, ist ein Zweierpaar dort kein Rechteck
        // mehr, sondern ein `TypeError: Argument must be rect style
        // object`. Zehn Tests in `mask_test.py` sind genau darueber
        // gestolpert - eine Verbesserung an einer Stelle, die an einer
        // anderen etwas zerbrochen hat.
        //
        // Gemessen nimmt `dest` einen Punkt, ein Rechteck **und** einen
        // `Vector2`; alles andere gibt
        // `TypeError: invalid dest argument`.
        let dest: (i32, i32) = match &dest_obj {
            None => (0, 0),
            Some(o) => match versatz_aus(o) {
                Ok(p) => p,
                Err(_) => match rect_aus_opt(Some(o)) {
                    Ok(Some(r)) => (r.x, r.y),
                    _ => {
                        return Err(PyTypeError::new_err("invalid dest argument"));
                    }
                },
            },
        };
        let area = match &area_obj {
            None => None,
            // **`mask.c` hat sein eigenes Wort** fuer ein untaugliches
            // Rechteck: `invalid rectstyle argument`, nicht das
            // `Argument must be rect style object` aus `rect.c`. Ein
            // durchgereichter Fehler waere also der falsche - deshalb
            // wird er hier ersetzt (`mask.c:2217`).
            Some(o) => match rect_aus_objekt(o) {
                Some(r) => Some((r.x, r.y, r.w, r.h)),
                None => {
                    return Err(PyTypeError::new_err("invalid rectstyle argument"));
                }
            },
        };

        // Die Vorlagen ausleihen - getrennt, damit beide gleichzeitig
        // offen sein duerfen.
        let setref = match &setflaeche {
            None => None,
            Some(o) => Some(o.extract::<PyRef<PySurface>>()?),
        };
        let unsetref = match &unsetflaeche {
            None => None,
            Some(o) => Some(o.extract::<PyRef<PySurface>>()?),
        };
        let setleih = match &setref {
            None => None,
            Some(r) => Some(r.leihen()?),
        };
        let unsetleih = match &unsetref {
            None => None,
            Some(r) => Some(r.leihen()?),
        };

        // Ohne Ziel eine neue Flaeche in der Groesse des Ausschnitts.
        let Some(zo) = ziel_obj else {
            // Die Groesse kommt aus dem **Schnitt** von `area` mit der
            // Maske, nicht aus `area` selbst: `area=Rect(-3, 0, 7, 7)`
            // auf einer 7x7-Maske gibt eine Flaeche von 4x7, und
            // `area=Rect(10, 10, 3, 3)` eine von 0x0. Siehe
            // `Mask::ausschnitt`.
            let (_, _, aw, ah) = Mask::ausschnitt(self.0.get_size(), area);
            let (w, h) = (aw.max(0) as u32, ah.max(0) as u32);
            let mut neu = rustgame_core::surface::Surface::with_flags(
                (w, h),
                rustgame_core::surface::SRCALPHA,
            );
            formate_pruefen(&neu, setleih.as_deref(), unsetleih.as_deref())?;
            let gesetzt = farben(&neu, setfarbe_obj.as_ref(), setfarbe_fehlt,
                                 (255, 255, 255, 255))?;
            let leer = farben(&neu, unsetfarbe_obj.as_ref(), unsetfarbe_fehlt,
                              (0, 0, 0, 255))?;
            self.0.to_surface_voll(
                &mut neu,
                setleih.as_deref(),
                unsetleih.as_deref(),
                gesetzt,
                leer,
                dest,
                area,
            );
            return Ok(PySurface::neu(neu).into_pyobject(py)?.into_any().unbind());
        };

        {
            // pygames eigener Satz - `to_surface("x")` gibt
            // `TypeError: invalid surface argument`, nicht den des
            // Argumentlesers.
            let ziel: PyRef<PySurface> = zo.extract().map_err(|_| {
                PyTypeError::new_err("invalid surface argument")
            })?;
            let mut k = ziel.leihen_mut()?;
            formate_pruefen(&k, setleih.as_deref(), unsetleih.as_deref())?;
            let gesetzt = farben(&k, setfarbe_obj.as_ref(), setfarbe_fehlt,
                                 (255, 255, 255, 255))?;
            let leer = farben(&k, unsetfarbe_obj.as_ref(), unsetfarbe_fehlt,
                              (0, 0, 0, 255))?;
            self.0.to_surface_voll(
                &mut k,
                setleih.as_deref(),
                unsetleih.as_deref(),
                gesetzt,
                leer,
                dest,
                area,
            );
        }
        // **Dieselbe** Flaeche zurueck, nicht eine Kopie.
        Ok(zo.clone().unbind())
    }

    fn __repr__(&self) -> String {
        let (w, h) = self.0.get_size();
        format!("<Mask({w}x{h})>")
    }
}

/// `pygame.mask.from_surface(surface, threshold=127)`.
///
/// Die Schwelle ist ein **echtes Groesser**: ein Pixel mit Alpha genau
/// `threshold` zaehlt nicht. Gemessen, steht so im Kern.
#[pyfunction]
#[pyo3(signature = (surface, threshold=127))]
#[pyo3(name = "from_surface")]
pub fn maske_from_surface(surface: &PySurface, threshold: i64) -> PyResult<PyMask> {
    // Als `i64` gelesen und als `i32` weitergegeben, wie `mask.c` es
    // haelt - **ohne zu klemmen**, die Begruendung steht am Kern.
    // Gelesen wird weit, weil `mask_test.py` auch 256 und -1 hineinreicht
    // und ein `i32`-Beiwert daraus einen `OverflowError` des
    // Argumentlesers machen wuerde statt einer leeren oder vollen Maske.
    let t = threshold.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    Ok(PyMask(mask::from_surface(&*surface.leihen()?, t)))
}

/// `pygame.mask.from_threshold(surface, color, threshold=(0,0,0,255))`.
#[pyfunction]
#[pyo3(signature = (surface, color, threshold=None, othersurface=None, palette_colors=1))]
#[pyo3(name = "from_threshold")]
pub fn maske_from_threshold(
    surface: &PySurface,
    color: &Bound<'_, PyAny>,
    threshold: Option<&Bound<'_, PyAny>>,
    othersurface: Option<&PySurface>,
    palette_colors: i32,
) -> PyResult<PyMask> {
    // `palette_colors` waehlt in pygame, ob auf einer Palettenflaeche die
    // **Tafelnummern** oder die Farben verglichen werden. rustgame rechnet
    // ausschliesslich in ARGB8888 - es gibt keine Tafel, also nichts zu
    // waehlen. Der Wert wird angenommen, damit ein Aufruf mit fuenf
    // Stellen nicht scheitert.
    let _ = palette_colors;
    // **Beide** Werte in allen Farbformen, gegen die Flaeche abgebildet -
    // ein Dreiertupel ebenso wie ein Vierertupel, ein `Color`, ein Name
    // oder eine blanke Zahl. Vorher stand hier ein `(u8, u8, u8)`, und
    // schon `from_threshold(s, (0,0,0,255))` scheiterte mit
    // `ValueError: expected tuple of length 3`.
    let s = surface.leihen()?;
    let c = crate::zeichnen::farbe_fuer(&s, color)?;
    let t = match threshold {
        None => (0, 0, 0),
        Some(x) if x.is_none() => (0, 0, 0),
        Some(x) => {
            let f = crate::zeichnen::farbe_fuer(&s, x)?;
            (f.r, f.g, f.b)
        }
    };
    // Mit `othersurface` tritt deren Pixel an die Stelle der Farbe -
    // siehe `mask::from_threshold_gegen`.
    let andere = match othersurface {
        None => None,
        Some(o) => Some(o.leihen()?),
    };
    Ok(PyMask(mask::from_threshold_gegen(
        &s,
        (c.r, c.g, c.b),
        t,
        andere.as_deref(),
    )))
}

/// Die beiden Vorlagen muessen **genau dasselbe Pixelformat** haben wie
/// das Ziel.
///
/// `mask.c` prueft das mit einer einzigen Zeile:
///
/// ```c
/// return PG_SURF_FORMATENUM(surf) == PG_SURF_FORMATENUM(check_surf);
/// ```
///
/// Also das **ganze** Format, nicht nur die Tiefe: `Surface(g, 0, 32)` und
/// `Surface(g, SRCALPHA, 32)` haben beide 32 Bit und sind trotzdem
/// verschieden, weil die Alphamaske einmal null ist und einmal nicht.
/// Genau daran haengen die vier `different_depths`/`different_srcalphas`-
/// Tests, und die Alphahaelfte davon faellt durch eine Pruefung, die nur
/// `bitsize` vergleicht.
///
/// Dass es hier ueberhaupt etwas zu vergleichen gibt, ist neu: bis vor
/// kurzem kannte der Kern nur ARGB8888, und `get_bitsize()` gab hart 32
/// zurueck. Ein Formattest waere damals immer wahr gewesen - die vier
/// Tests standen zu Recht als Grenze vermerkt, **und der Vermerk ist jetzt
/// veraltet**. Er stand da, weil er stimmte, und er stimmt nicht mehr,
/// seit `rustgame_core::surface::Format` existiert.
///
/// Der Wortlaut ist zweimal fast derselbe und unterscheidet sich nur im
/// ersten Wort - deshalb steht er hier einmal mit Platzhalter.
fn formate_pruefen(
    ziel: &rustgame_core::surface::Surface,
    setflaeche: Option<&rustgame_core::surface::Surface>,
    unsetflaeche: Option<&rustgame_core::surface::Surface>,
) -> PyResult<()> {
    let f = ziel.get_format();
    for (name, vorlage) in [("setsurface", setflaeche), ("unsetsurface", unsetflaeche)] {
        if let Some(v) = vorlage {
            if v.get_format() != f {
                return Err(PyValueError::new_err(format!(
                    "{name} needs to have same bytesize/bitsize/alpha format as surface"
                )));
            }
        }
    }
    Ok(())
}

/// Loest eine der beiden Farben gegen die **Zielflaeche** auf.
///
/// Drei Faelle, und alle drei sind gemessen:
///
/// * nicht angegeben -> die Voreinstellung
/// * ausdruecklich `None` -> `None`, die Seite bleibt **unberuehrt**
/// * sonst -> `zeichnen::farbe_fuer`, also pygames
///   `pg_MappedColorFromObj` gegen genau diese Flaeche
///
/// Der zweite Fall ist der, den eine gewoehnliche `Option` verschluckt:
/// ohne die Unterscheidung waere `to_surface(unsetcolor=None)` schwarz
/// statt leer.
fn farben(
    ziel: &rustgame_core::surface::Surface,
    o: Option<&Bound<'_, PyAny>>,
    angegeben: bool,
    vorgabe: (u8, u8, u8, u8),
) -> PyResult<Option<(u8, u8, u8, u8)>> {
    match o {
        Some(x) => Ok(Some(
            crate::zeichnen::farbe_fuer(ziel, x)?.as_tuple(),
        )),
        // `wahl` hat `None` schon zu `None` gemacht - war der Wert
        // ueberhaupt da, ist die Seite gewollt durchsichtig.
        None if angegeben => Ok(None),
        None => Ok(Some(vorgabe)),
    }
}

/// Holt eine Farbe aus Stelle **oder** Schluesselwort - und sagt den
/// Unterschied zwischen "fehlt" und "ist `None`".
/// Zwei Zahlen aus allem, was zwei Zahlen hergibt - **auch negative**.
///
/// `versatz_aus` schneidet zur Null hin ab und taugt fuer Versaetze;
/// hier geht es um Groessen, und eine negative muss **durchkommen**,
/// damit die Pruefung danach pygames `ValueError` melden kann statt eines
/// `OverflowError` aus dem Argumentleser.
fn paar_aus(o: &Bound<'_, PyAny>) -> PyResult<(i64, i64)> {
    // `scale` ist pygames Wort an dieser Stelle - siehe
    // [`crate::umwandlung::versatz_aus_wort`].
    let (x, y) = versatz_aus_wort(o, "scale").map(|(a, b)| (a as i64, b as i64))?;
    Ok((x, y))
}

/// Ein Versatz mit Vorgabe `(0, 0)`.
///
/// `#[pyo3(signature = ...)]` kann fuer einen `&Bound` keine Vorgabe
/// hinschreiben - `offset=(0, 0)` uebersetzt dort nicht. Also kommt der
/// Wert als `Option` herein und die Vorgabe steht hier.
fn versatz_wahl(o: Option<&Bound<'_, PyAny>>) -> PyResult<(i32, i32)> {
    match o {
        None => Ok((0, 0)),
        Some(x) if x.is_none() => Ok((0, 0)),
        Some(x) => versatz_aus(x),
    }
}

/// Ein Wert aus `*args` oder `**kwargs` - `None` bleibt `None`.
///
/// pygames `to_surface` nimmt **beides**, und zwar gemischt. Ohne diese
/// Stelle muesste jede der sieben Angaben zweimal ausgewertet werden.
fn wahl<'p>(
    args: &Bound<'p, PyTuple>,
    kwargs: Option<&Bound<'p, PyDict>>,
    stelle: usize,
    name: &str,
) -> PyResult<Option<Bound<'p, PyAny>>> {
    let mut gefunden = None;
    if args.len() > stelle {
        gefunden = Some(args.get_item(stelle)?);
    }
    if let Some(kw) = kwargs {
        if let Some(v) = kw.get_item(name)? {
            gefunden = Some(v);
        }
    }
    Ok(match gefunden {
        Some(o) if o.is_none() => None,
        andere => andere,
    })
}
