//! frechteck.rs
//! ============
//! `pygame.FRect` - das Rechteck mit Nachkommastellen.
//!
//! Aufgebaut wie [`crate::rechteck`], mit **einem** Unterschied, der im
//! Kern nachgemessen ist und hier hindurchgereicht werden muss:
//!
//! | Leser | Genauigkeit |
//! |---|---|
//! | Einzelwerte (`right`, `centerx`, `width`) | `f64`, **ungerundet** |
//! | Tupel (`center`, `topleft`, `size`) | durch `f32` hindurch |
//!
//! Gemessen an `FRect(-38.5, 0, 2.534, 1)`: `centerx` meldet
//! `-37.23300004005432`, `center[0]` dagegen `-37.233001708984375`.
//! Dieselbe Rechnung, zwei Ergebnisse - weil pygames Tupel-Leser ihre
//! Werte als `float` durchreichen. Der Kern bildet das ab, die Huelle
//! gibt beides unveraendert weiter.
//!
//! Ein zweiter Unterschied ist neu hinzugekommen und steht bei
//! `wie_printf`: pygames `%f` schreibt NaN als `-nan(ind)`, Rust als
//! `NaN`.
//!
//! **Die Umwandler kommen aus [`crate::rechteck`].** `pg_FloatFromObj`
//! und `pg_TwoFloatsFromObj` gelten fuer beide Rechtecktypen - sogar die
//! Faktoren von `scale_by` sind im Original beim ganzzahligen `Rect`
//! Fliesskomma.
//!
//! **Hier stand ein zweiter `#[pymethods]`-Block nur fuer `move_to`**,
//! begruendet damit, PyO3 finde Schluesselwortangaben in einem per Makro
//! erzeugten Block nicht wieder. Der Satz ist nachgemessen falsch; die
//! Messung steht im Kopf von [`crate::rechteck`]. `move_to` steht jetzt
//! bei den anderen Methoden und prueft dabei die beiden Regeln, die es
//! vorher nicht hatte: keine Stellenangaben, mindestens ein
//! Schluesselwort.

use pyo3::exceptions::{PyAttributeError, PyTypeError};
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFloat, PyList, PySlice, PyTuple};

use rustgame_core::frect::FRect as KernFRect;
use rustgame_core::rect::Rect as KernRect;

use crate::rechteck::{
    faktoren_aus, flieszt_aus, ist_folge, ist_index, keine_rechteckfolge, nicht_rechteckig,
    typname, ungueltige_zuweisung, ungueltiger_index, woerterbuch_pruefen, zwei_flieszt_aus,
    Bauplan, PyRect,
};

#[pyclass(name = "FRect", module = "rustgame_py.rect", subclass)]
#[derive(Clone, Copy)]
/// `pygame.FRect` - ein Rechteck mit Nachkommastellen.
pub struct PyFRect(pub(crate) KernFRect);

// ---------------------------------------------------------------------
// Die Umwandler - dieselben Zeilen wie beim `Rect`, nur mit `float`
// ---------------------------------------------------------------------

/// `RectExport_RectFromObject` fuer die `FRect`-Auspraegung -
/// `rect_impl.h` Zeile 636.
///
/// **Ein `Rect` zaehlt als `FRect`** und alles mit einem Feld `rect`
/// ebenso. Das zweite fehlte hier, und pygames Testsuite hat es
/// fuenfundzwanzig Mal bemerkt: `_ObjectWithRectAttribute` und seine
/// Geschwister gingen beim ganzzahligen `Rect` durch und beim `FRect`
/// nicht.
pub fn frect_aus_objekt(o: &Bound<'_, PyAny>) -> Option<KernFRect> {
    frect_aus_objekt_tief(o, 0)
}

/// So tief darf ein `rect`-Feld verschachtelt sein - siehe
/// [`crate::rechteck`], dort steht die Begruendung.
const TIEFE_GRENZE: u32 = 16;

fn frect_aus_objekt_tief(o: &Bound<'_, PyAny>, tiefe: u32) -> Option<KernFRect> {
    if tiefe > TIEFE_GRENZE {
        return None;
    }
    if let Ok(f) = o.downcast::<PyFRect>() {
        return Some(f.borrow().0);
    }
    if let Ok(r) = o.downcast::<PyRect>() {
        return Some(KernFRect::from(r.borrow().wert()));
    }
    let schnell = o.is_instance_of::<PyList>() || o.is_instance_of::<PyTuple>();
    if schnell {
        return aus_folge(o, tiefe);
    }
    if ist_folge(o) {
        if let Some(f) = aus_folge(o, tiefe) {
            return Some(f);
        }
    }
    let feld = o.getattr(intern!(o.py(), "rect")).ok()?;
    let feld = if feld.is_callable() {
        feld.call0().ok()?
    } else {
        feld
    };
    frect_aus_objekt_tief(&feld, tiefe + 1)
}

fn aus_folge(o: &Bound<'_, PyAny>, tiefe: u32) -> Option<KernFRect> {
    match o.len().ok()? {
        4 => Some(KernFRect::new(
            flieszt_aus(&o.get_item(0).ok()?)?,
            flieszt_aus(&o.get_item(1).ok()?)?,
            flieszt_aus(&o.get_item(2).ok()?)?,
            flieszt_aus(&o.get_item(3).ok()?)?,
        )),
        2 => {
            let (x, y) = zwei_flieszt_aus(&o.get_item(0).ok()?)?;
            let (w, h) = zwei_flieszt_aus(&o.get_item(1).ok()?)?;
            Some(KernFRect::new(x, y, w, h))
        }
        1 if o.is_instance_of::<PyTuple>() => {
            frect_aus_objekt_tief(&o.get_item(0).ok()?, tiefe + 1)
        }
        _ => None,
    }
}

/// Nimmt alles an, was in pygame als `FRect` durchgeht - mit Fehler
/// statt `None`. Der alte Name, damit die uebrigen Module bleiben
/// koennen, wie sie sind.
pub fn frect_aus(o: &Bound<'_, PyAny>) -> PyResult<KernFRect> {
    frect_aus_objekt(o).ok_or_else(nicht_rechteckig)
}

/// `RectExport_RectFromFastcallArgs` - eine Angabe, zwei Paare oder vier
/// Zahlen.
fn frect_aus_stellen(args: &Bound<'_, PyTuple>) -> Option<KernFRect> {
    match args.len() {
        1 => frect_aus_objekt(&args.get_item(0).ok()?),
        4 => Some(KernFRect::new(
            flieszt_aus(&args.get_item(0).ok()?)?,
            flieszt_aus(&args.get_item(1).ok()?)?,
            flieszt_aus(&args.get_item(2).ok()?)?,
            flieszt_aus(&args.get_item(3).ok()?)?,
        )),
        2 => {
            let (x, y) = zwei_flieszt_aus(&args.get_item(0).ok()?)?;
            let (w, h) = zwei_flieszt_aus(&args.get_item(1).ok()?)?;
            Some(KernFRect::new(x, y, w, h))
        }
        _ => None,
    }
}

/// `RectExport_pgTwoValuesFromFastcallArgs` - `rect_impl.h` Zeile 939.
fn zwei_werte_aus_stellen(args: &Bound<'_, PyTuple>) -> PyResult<(f32, f32)> {
    match args.len() {
        1 => {
            let a = args.get_item(0)?;
            if let Some(p) = zwei_flieszt_aus(&a) {
                return Ok(p);
            }
            if !ist_folge(&a) {
                return Err(PyTypeError::new_err(format!(
                    "Invalid argument. Expected a sequence but got: '{}'",
                    typname(&a)
                )));
            }
            let groesse = a.len().unwrap_or(0);
            if groesse != 2 {
                return Err(PyTypeError::new_err(format!(
                    "Invalid sequence size. Expected size 2 but got: {groesse}"
                )));
            }
            Err(PyTypeError::new_err(format!(
                "Invalid sequence values. Expected two numeric values but got: '{}', '{}'",
                typname(&a.get_item(0)?),
                typname(&a.get_item(1)?)
            )))
        }
        2 => {
            let x = flieszt_aus(&args.get_item(0)?).ok_or_else(|| {
                PyTypeError::new_err(format!(
                    "The first argument must be numeric (got: '{}')",
                    typname(&args.get_item(0).unwrap())
                ))
            })?;
            let y = flieszt_aus(&args.get_item(1)?).ok_or_else(|| {
                PyTypeError::new_err(format!(
                    "The second argument must be numeric (got: '{}')",
                    typname(&args.get_item(1).unwrap())
                ))
            })?;
            Ok((x, y))
        }
        n => Err(PyTypeError::new_err(format!(
            "Function takes at most 2 arguments ({n} given)"
        ))),
    }
}

/// `four_floats_from_obj` - `rect.c` Zeile 388. Nur `clipline`.
fn vier_flieszt_aus(o: &Bound<'_, PyAny>) -> PyResult<(f32, f32, f32, f32)> {
    let laenge = o.len().unwrap_or(usize::MAX);
    match laenge {
        2 => {
            let (x1, y1) = zwei_flieszt_aus(&o.get_item(0)?).ok_or_else(|| {
                PyTypeError::new_err("number pair expected for first argument")
            })?;
            let (x2, y2) = zwei_flieszt_aus(&o.get_item(1)?).ok_or_else(|| {
                PyTypeError::new_err("number pair expected for second argument")
            })?;
            Ok((x1, y1, x2, y2))
        }
        4 => {
            let n = |i: usize, wort: &str| -> PyResult<f32> {
                flieszt_aus(&o.get_item(i)?).ok_or_else(|| {
                    PyTypeError::new_err(format!("number expected for {wort} argument"))
                })
            };
            Ok((n(0, "first")?, n(1, "second")?, n(2, "third")?, n(3, "fourth")?))
        }
        _ => Err(PyTypeError::new_err(format!(
            "sequence argument takes 2 or 4 items ({} given)",
            if laenge == usize::MAX { -1 } else { laenge as i64 }
        ))),
    }
}

impl PyFRect {
    /// Das Kernrechteck dahinter.
    pub fn wert(&self) -> KernFRect {
        self.0
    }

    /// Ein neues Rechteck **vom Typ des Empfaengers** - siehe
    /// [`crate::rechteck`], dort steht die ganze Begruendung.
    fn neu_wie<'py>(slf: &Bound<'py, PyFRect>, r: KernFRect) -> PyResult<Bound<'py, PyFRect>> {
        let py = slf.py();
        let typ = slf.get_type();
        if typ.is(py.get_type::<PyFRect>()) {
            return Bound::new(py, PyFRect(r));
        }
        let neu = typ.getattr(intern!(py, "__new__"))?.call1((&typ,))?;
        let neu = neu.downcast_into::<PyFRect>()?;
        neu.borrow_mut().0 = r;
        Ok(neu)
    }

    /// Das Rechteck eines Listenelements - mit oder ohne `key`. Die drei
    /// Meldungen sind gemessen; siehe [`crate::rechteck`].
    fn rechteck_von(
        element: &Bound<'_, PyAny>,
        key: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<KernFRect> {
        let gesucht = match key {
            Some(k) => k.call1((element,))?,
            None => element.clone(),
        };
        frect_aus_objekt(&gesucht).ok_or_else(|| {
            PyTypeError::new_err("Sequence must contain rect or rect-like objects")
        })
    }

    /// Prueft Folge und `key` fuer `collideobjects` und gibt die Laenge.
    fn objektfolge(liste: &Bound<'_, PyAny>, key: Option<&Bound<'_, PyAny>>) -> PyResult<usize> {
        if !ist_folge(liste) {
            return Err(PyTypeError::new_err(
                "Argument must be a sequence of objects.",
            ));
        }
        if let Some(k) = key {
            if !k.is_callable() {
                return Err(PyTypeError::new_err(
                    "Key function must be callable with one argument.",
                ));
            }
        }
        liste.len()
    }

    /// Eine Folge, kein Iterator - siehe [`crate::rechteck`], dort steht
    /// die Messung.
    fn liste_aus(o: &Bound<'_, PyAny>) -> PyResult<Vec<KernFRect>> {
        if !ist_folge(o) {
            return Err(keine_rechteckfolge());
        }
        let laenge = o.len()?;
        let mut v = Vec::with_capacity(laenge);
        for i in 0..laenge {
            v.push(frect_aus_objekt(&o.get_item(i)?).ok_or_else(keine_rechteckfolge)?);
        }
        Ok(v)
    }

    /// Setzt eines der virtuellen Attribute ueber seinen Namen - fuer
    /// `move_to(**kwargs)` und `Surface.get_frect(**kwargs)`.
    ///
    /// Gemessen: ein unbekannter Name gibt einen **AttributeError**.
    /// Hier stand ein TypeError, und das war falsch: pygames `move_to`
    /// geht ueber `PyObject_SetAttr`, und das meldet AttributeError.
    pub fn setze(&mut self, name: &str, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        let zahl = || flieszt_aus(wert).ok_or_else(ungueltige_zuweisung);
        let paar = || zwei_flieszt_aus(wert).ok_or_else(ungueltige_zuweisung);
        match name {
            "x" => self.0.x = zahl()?,
            "y" => self.0.y = zahl()?,
            "w" => self.0.w = zahl()?,
            "h" => self.0.h = zahl()?,
            "left" => { self.0.set_left(zahl()?); }
            "top" => { self.0.set_top(zahl()?); }
            "right" => { self.0.set_right(zahl()?); }
            "bottom" => { self.0.set_bottom(zahl()?); }
            "centerx" => { self.0.set_centerx(zahl()?); }
            "centery" => { self.0.set_centery(zahl()?); }
            "width" => { self.0.set_width(zahl()?); }
            "height" => { self.0.set_height(zahl()?); }
            "topleft" => { self.0.set_topleft(paar()?); }
            "topright" => { self.0.set_topright(paar()?); }
            "bottomleft" => { self.0.set_bottomleft(paar()?); }
            "bottomright" => { self.0.set_bottomright(paar()?); }
            "center" => { self.0.set_center(paar()?); }
            "relcenter" => { self.0.set_relcenter(paar()?); }
            "midtop" => { self.0.set_midtop(paar()?); }
            "midbottom" => { self.0.set_midbottom(paar()?); }
            "midleft" => { self.0.set_midleft(paar()?); }
            "midright" => { self.0.set_midright(paar()?); }
            "size" => { self.0.set_size(paar()?); }
            _ => {
                return Err(PyAttributeError::new_err(format!(
                    "'pygame.rect.FRect' object has no attribute '{name}' \
                     and no __dict__ for setting new attributes"
                )))
            }
        }
        Ok(())
    }

    /// Wendet die Ankerwoerter an - `center=`, `topleft=`, `size=`, ...
    ///
    /// Steht hier und nicht im `#[pymethods]`-Block, weil es zwei
    /// Aufrufer gibt: `move_to(**kwargs)` und `Surface.get_frect(**kwargs)`.
    pub fn aus_kwargs(&mut self, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let Some(kw) = kwargs else { return Ok(()) };
        for (schluessel, wert) in kw.iter() {
            let name: String = schluessel.extract()?;
            self.setze(&name, &wert)?;
        }
        Ok(())
    }
}

/// Erzeugt die achtzehn Ankerpaare - siehe den Kopf von
/// [`crate::rechteck`].
macro_rules! frect_anker {
    (
        punkte: [$($pl:ident / $ps:ident),* $(,)?],
        zahlen: [$($zl:ident / $zs:ident),* $(,)?],
    ) => {
        #[pymethods]
        impl PyFRect {
            $(
                #[getter]
                fn $pl(&self) -> (f64, f64) { self.0.$pl() }
                // `&Bound` statt `(f32, f32)`: PyO3 meldet bei falscher
                // Laenge `ValueError`, pygame `TypeError`.
                #[setter]
                fn $ps(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
                    self.0.$ps(zwei_flieszt_aus(v).ok_or_else(ungueltige_zuweisung)?);
                    Ok(())
                }
            )*

            $(
                #[getter]
                fn $zl(&self) -> f64 { self.0.$zl() }
                #[setter]
                fn $zs(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
                    self.0.$zs(flieszt_aus(v).ok_or_else(ungueltige_zuweisung)?);
                    Ok(())
                }
            )*
        }
    };
}

frect_anker! {
    punkte: [
        topleft / set_topleft,
        topright / set_topright,
        bottomleft / set_bottomleft,
        bottomright / set_bottomright,
        center / set_center,
        relcenter / set_relcenter,
        midtop / set_midtop,
        midbottom / set_midbottom,
        midleft / set_midleft,
        midright / set_midright,
        size / set_size,
    ],
    zahlen: [
        left / set_left,
        top / set_top,
        right / set_right,
        bottom / set_bottom,
        centerx / set_centerx,
        centery / set_centery,
        width / set_width,
        height / set_height,
    ],
}

#[pymethods]
impl PyFRect {
    /// `FRect()`, `FRect(r)`, `FRect((x,y),(w,h))` oder `FRect(x,y,w,h)`.
    ///
    /// Wie beim ganzzahligen `Rect` wird das ganze Argumenttupel als ein
    /// Stueck gelesen - `FRect(1, 2)` ist darum ein TypeError.
    #[new]
    #[pyo3(signature = (*args, **_kwargs))]
    fn new(args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        if args.is_empty() {
            return Ok(PyFRect(KernFRect::new(0.0, 0.0, 0.0, 0.0)));
        }
        Ok(PyFRect(
            frect_aus_objekt(args.as_any()).ok_or_else(nicht_rechteckig)?,
        ))
    }

    /// Setzt den Wert neu - siehe [`crate::rechteck`], dort steht,
    /// warum PyO3s `#[new]` allein nicht reicht.
    #[pyo3(signature = (*args, **_kwargs))]
    fn __init__(
        &mut self,
        args: &Bound<'_, PyTuple>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        if args.is_empty() {
            return Ok(());
        }
        self.0 = frect_aus_objekt(args.as_any()).ok_or_else(nicht_rechteckig)?;
        Ok(())
    }

    #[getter]
    fn x(&self) -> f64 { self.0.x as f64 }
    #[setter]
    fn set_x(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.0.x = flieszt_aus(v).ok_or_else(ungueltige_zuweisung)?;
        Ok(())
    }
    #[getter]
    fn y(&self) -> f64 { self.0.y as f64 }
    #[setter]
    fn set_y(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.0.y = flieszt_aus(v).ok_or_else(ungueltige_zuweisung)?;
        Ok(())
    }
    #[getter]
    fn w(&self) -> f64 { self.0.w as f64 }
    #[setter]
    fn set_w(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.0.w = flieszt_aus(v).ok_or_else(ungueltige_zuweisung)?;
        Ok(())
    }
    #[getter]
    fn h(&self) -> f64 { self.0.h as f64 }
    #[setter]
    fn set_h(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.0.h = flieszt_aus(v).ok_or_else(ungueltige_zuweisung)?;
        Ok(())
    }

    /// Setzt alle vier Werte neu - gibt `None` zurueck.
    #[pyo3(signature = (*args))]
    fn update(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        self.0 = frect_aus_stellen(args).ok_or_else(nicht_rechteckig)?;
        Ok(())
    }

    /// In Python `move` - siehe [`crate::rechteck::PyRect`], dort steht
    /// die Begruendung fuer den Unterstrich.
    #[pyo3(name = "move", signature = (*args))]
    fn move_<'py>(
        slf: &Bound<'py, PyFRect>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyFRect>> {
        let (dx, dy) = zwei_werte_aus_stellen(args)?;
        let r = slf.borrow().0.move_(dx, dy);
        PyFRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (*args))]
    fn move_ip(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let (dx, dy) = zwei_werte_aus_stellen(args)?;
        self.0.move_ip(dx, dy);
        Ok(())
    }

    /// `move_to(center=…)` und die neunzehn Geschwister.
    ///
    /// Nimmt **nur** Schluesselwoerter, und mindestens eines. Hier stand
    /// vorher `#[pyo3(signature = (**kwargs))]` ohne die beiden Proben,
    /// und `r.move_to()` gab still eine Kopie zurueck statt des
    /// gemessenen TypeError.
    #[pyo3(signature = (*args, **kwargs))]
    fn move_to<'py>(
        slf: &Bound<'py, PyFRect>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Bound<'py, PyFRect>> {
        if !args.is_empty() {
            return Err(PyTypeError::new_err("move_to takes no positional arguments"));
        }
        if kwargs.is_none_or(|k| k.is_empty()) {
            return Err(PyTypeError::new_err(
                "move_to cannot be called without keyword arguments",
            ));
        }
        let neu = PyFRect::neu_wie(slf, slf.borrow().0)?;
        {
            let mut leihe = neu.borrow_mut();
            leihe.aus_kwargs(kwargs)?;
        }
        Ok(neu)
    }

    #[pyo3(signature = (*args))]
    fn inflate<'py>(
        slf: &Bound<'py, PyFRect>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyFRect>> {
        let (dx, dy) = zwei_werte_aus_stellen(args)?;
        let r = slf.borrow().0.inflate(dx, dy);
        PyFRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (*args))]
    fn inflate_ip(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let (dx, dy) = zwei_werte_aus_stellen(args)?;
        self.0.inflate_ip(dx, dy);
        Ok(())
    }

    /// Fuenf Schreibweisen - siehe [`crate::rechteck::PyRect`].
    #[pyo3(signature = (*args, **kwargs))]
    fn scale_by<'py>(
        slf: &Bound<'py, PyFRect>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Bound<'py, PyFRect>> {
        let (fx, fy) = faktoren_aus(args, kwargs)?;
        let r = slf.borrow().0.scale_by(fx, fy);
        PyFRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (*args, **kwargs))]
    fn scale_by_ip(
        &mut self,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        let (fx, fy) = faktoren_aus(args, kwargs)?;
        self.0.scale_by_ip(fx, fy);
        Ok(())
    }

    fn normalize(&mut self) {
        self.0.normalize();
    }

    fn copy<'py>(slf: &Bound<'py, PyFRect>) -> PyResult<Bound<'py, PyFRect>> {
        let r = slf.borrow().0;
        PyFRect::neu_wie(slf, r)
    }

    fn __copy__<'py>(slf: &Bound<'py, PyFRect>) -> PyResult<Bound<'py, PyFRect>> {
        PyFRect::copy(slf)
    }

    /// `__safe_for_unpickling__` - **`True`**, und bei pygame ein Feld.
    ///
    /// Gemessen ist es dort ein `getset_descriptor`, also von der Instanz
    /// aus ein Wert und kein Aufruf. Bei `Vector2` dagegen eine
    /// **Methode** - dieselbe Marke, zwei Gestalten, und beides ist
    /// nachgemessen.
    ///
    /// Die Marke sagt `copyreg`, dass ein Objekt ohne Argumente neu gebaut
    /// und dann gefuellt werden darf. Gefunden hat den fehlenden Namen die
    /// Probe `Klassendoppel`: sie vergleicht die **Doppelstriche** aller
    /// 65 Klassen, und die liessen alle anderen Proben aus - denn ein
    /// Doppelstrich ist ein Name mit Unterstrich.
    #[getter]
    fn __safe_for_unpickling__(&self) -> bool {
        true
    }

    /// Fuer `pickle` und `copy.deepcopy` - `(typ, (x, y, w, h))`.
    fn __reduce__<'py>(slf: &Bound<'py, PyFRect>) -> PyResult<Bauplan<'py, f64>> {
        let r = slf.borrow().0;
        Ok((
            slf.get_type().into_any(),
            (r.x as f64, r.y as f64, r.w as f64, r.h as f64),
        ))
    }

    // ----- Beruehrungen ---------------------------------------------------

    #[pyo3(signature = (*args))]
    fn collidepoint(&self, args: &Bound<'_, PyTuple>) -> PyResult<bool> {
        Ok(self.0.collidepoint(zwei_werte_aus_stellen(args)?))
    }

    #[pyo3(signature = (*args))]
    fn colliderect(&self, args: &Bound<'_, PyTuple>) -> PyResult<bool> {
        Ok(self
            .0
            .colliderect(frect_aus_stellen(args).ok_or_else(nicht_rechteckig)?))
    }

    #[pyo3(signature = (*args))]
    fn contains(&self, args: &Bound<'_, PyTuple>) -> PyResult<bool> {
        Ok(self
            .0
            .contains(frect_aus_stellen(args).ok_or_else(nicht_rechteckig)?))
    }

    #[pyo3(signature = (*args))]
    fn clip<'py>(
        slf: &Bound<'py, PyFRect>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyFRect>> {
        let andere = frect_aus_stellen(args).ok_or_else(nicht_rechteckig)?;
        let r = slf.borrow().0.clip(andere);
        PyFRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (*args))]
    fn union<'py>(
        slf: &Bound<'py, PyFRect>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyFRect>> {
        let andere = frect_aus_stellen(args).ok_or_else(nicht_rechteckig)?;
        let r = slf.borrow().0.union(andere);
        PyFRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (*args))]
    fn union_ip(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        self.0
            .union_ip(frect_aus_stellen(args).ok_or_else(nicht_rechteckig)?);
        Ok(())
    }

    #[pyo3(signature = (*args))]
    fn clamp<'py>(
        slf: &Bound<'py, PyFRect>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyFRect>> {
        let andere = frect_aus_stellen(args).ok_or_else(nicht_rechteckig)?;
        let r = slf.borrow().0.clamp(andere);
        PyFRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (*args))]
    fn clamp_ip(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        self.0
            .clamp_ip(frect_aus_stellen(args).ok_or_else(nicht_rechteckig)?);
        Ok(())
    }

    #[pyo3(signature = (*args))]
    fn fit<'py>(
        slf: &Bound<'py, PyFRect>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyFRect>> {
        let andere = frect_aus_stellen(args).ok_or_else(nicht_rechteckig)?;
        let r = slf.borrow().0.fit(andere);
        PyFRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (liste, /))]

    fn unionall<'py>(
        slf: &Bound<'py, PyFRect>,
        liste: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyFRect>> {
        let alle = PyFRect::liste_aus(liste)?;
        let r = slf.borrow().0.unionall(&alle);
        PyFRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (liste, /))]

    fn unionall_ip(&mut self, liste: &Bound<'_, PyAny>) -> PyResult<()> {
        self.0.unionall_ip(&PyFRect::liste_aus(liste)?);
        Ok(())
    }

    #[pyo3(signature = (liste, /))]

    fn collidelist(&self, liste: &Bound<'_, PyAny>) -> PyResult<i64> {
        Ok(self
            .0
            .collidelist(&PyFRect::liste_aus(liste)?)
            .map(|i| i as i64)
            .unwrap_or(-1))
    }

    #[pyo3(signature = (liste, /))]

    fn collidelistall(&self, liste: &Bound<'_, PyAny>) -> PyResult<Vec<usize>> {
        Ok(self.0.collidelistall(&PyFRect::liste_aus(liste)?))
    }

    /// Das erste Paar `(Schluessel, Wert)`, dessen Rechteck dieses hier
    /// beruehrt - sonst `None`. `rect_impl.h` Zeile 1728.
    #[pyo3(signature = (rect_dict, values=None))]
    fn collidedict(
        &self,
        py: Python<'_>,
        rect_dict: &Bound<'_, PyAny>,
        values: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        let (woerter, nach_werten) = woerterbuch_pruefen(rect_dict, values)?;
        if self.0.w != 0.0 && self.0.h != 0.0 {
            for (schluessel, wert) in woerter.iter() {
                let anderes = if nach_werten { &wert } else { &schluessel };
                let r = frect_aus_objekt(anderes).ok_or_else(|| schluessel_fehler(nach_werten))?;
                if self.0.colliderect(r) {
                    return Ok(PyTuple::new(py, [schluessel, wert])?.into_any().unbind());
                }
            }
        }
        Ok(py.None())
    }

    /// Alle Paare, deren Rechteck dieses hier beruehrt - eine Liste.
    #[pyo3(signature = (rect_dict, values=None))]
    fn collidedictall(
        &self,
        py: Python<'_>,
        rect_dict: &Bound<'_, PyAny>,
        values: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        let (woerter, nach_werten) = woerterbuch_pruefen(rect_dict, values)?;
        let treffer = PyList::empty(py);
        if self.0.w != 0.0 && self.0.h != 0.0 {
            for (schluessel, wert) in woerter.iter() {
                let anderes = if nach_werten { &wert } else { &schluessel };
                let r = frect_aus_objekt(anderes).ok_or_else(|| schluessel_fehler(nach_werten))?;
                if self.0.colliderect(r) {
                    treffer.append(PyTuple::new(py, [&schluessel, &wert])?)?;
                }
            }
        }
        Ok(treffer.into_any().unbind())
    }

    /// Gibt **das Element** zurueck, nicht dessen Rechteck - gemessen an
    /// pygame, und der Unterschied ist der ganze Zweck der Methode.
    #[pyo3(signature = (liste, key=None))]
    fn collideobjects(
        &self,
        liste: &Bound<'_, PyAny>,
        key: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Option<Py<PyAny>>> {
        let laenge = PyFRect::objektfolge(liste, key)?;
        for i in 0..laenge {
            let element = liste.get_item(i)?;
            if self.0.colliderect(PyFRect::rechteck_von(&element, key)?) {
                return Ok(Some(element.unbind()));
            }
        }
        Ok(None)
    }

    #[pyo3(signature = (liste, key=None))]
    fn collideobjectsall(
        &self,
        liste: &Bound<'_, PyAny>,
        key: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let laenge = PyFRect::objektfolge(liste, key)?;
        let mut treffer = Vec::new();
        for i in 0..laenge {
            let element = liste.get_item(i)?;
            if self.0.colliderect(PyFRect::rechteck_von(&element, key)?) {
                treffer.push(element.unbind());
            }
        }
        Ok(treffer)
    }

    /// Wie in pygame: **zwei Paare**, und bei keinem Schnitt ein leeres
    /// Tupel - nicht `None`.
    #[pyo3(signature = (*args))]
    fn clipline(&self, py: Python<'_>, args: &Bound<'_, PyTuple>) -> PyResult<Py<PyAny>> {
        let strecke = match args.len() {
            1 => vier_flieszt_aus(&args.get_item(0)?)?,
            2 => {
                let (x1, y1) = zwei_flieszt_aus(&args.get_item(0)?).ok_or_else(|| {
                    PyTypeError::new_err("number pair expected for first argument")
                })?;
                let (x2, y2) = zwei_flieszt_aus(&args.get_item(1)?).ok_or_else(|| {
                    PyTypeError::new_err("number pair expected for second argument")
                })?;
                (x1, y1, x2, y2)
            }
            4 => {
                let n = |i: usize, wort: &str| -> PyResult<f32> {
                    flieszt_aus(&args.get_item(i)?).ok_or_else(|| {
                        PyTypeError::new_err(format!("number expected for {wort} argument"))
                    })
                };
                (n(0, "first")?, n(1, "second")?, n(2, "third")?, n(3, "fourth")?)
            }
            _ => {
                return Err(PyTypeError::new_err(
                    "clipline() takes 1, 2, or 4 arguments",
                ))
            }
        };
        Ok(match self.0.clipline(strecke) {
            None => PyTuple::empty(py).into_any().unbind(),
            Some((a, b, c, d)) => ((a, b), (c, d)).into_pyobject(py)?.into_any().unbind(),
        })
    }

    // ----- Wie eine Folge --------------------------------------------------

    fn __len__(&self) -> usize {
        4
    }

    /// `f[2]` ist die Breite, `f[1:3]` eine **Liste**, `f[...]` alle vier.
    fn __getitem__(&self, py: Python<'_>, op: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let alle = [
            self.0.x as f64,
            self.0.y as f64,
            self.0.w as f64,
            self.0.h as f64,
        ];
        if ist_index(op) {
            let i = op.extract::<isize>().map_err(|_| ungueltiger_index())?;
            let i = if (-4..0).contains(&i) { i + 4 } else { i };
            return match i {
                0..=3 => Ok(alle[i as usize].into_pyobject(py)?.into_any().unbind()),
                _ => Err(ungueltiger_index()),
            };
        }
        if op.is(py.Ellipsis()) {
            return Ok(PyList::new(py, alle)?.into_any().unbind());
        }
        if let Ok(s) = op.downcast::<PySlice>() {
            let teil = s.indices(4)?;
            let mut aus = Vec::with_capacity(teil.slicelength);
            for k in 0..teil.slicelength {
                aus.push(alle[(teil.start + teil.step * k as isize) as usize]);
            }
            return Ok(PyList::new(py, aus)?.into_any().unbind());
        }
        Err(PyTypeError::new_err("Invalid Rect slice"))
    }

    /// `RectExport_assSubscript`, `rect_impl.h` Zeile 2237 - mit allen
    /// drei Formen: Zahl, `...` und Scheibe. `r[...] = 5` setzt alle
    /// vier Werte, `r[1:3] = (7, 8)` genau zwei.
    ///
    /// **Erst alles lesen, dann einmal schreiben.** `r[::-1] = r` reicht
    /// das Rechteck als Wert an sich selbst; haelt die Methode dabei
    /// schon eine veraenderliche Leihe (`&mut self`), meldet PyO3
    /// `RuntimeError: Already mutably borrowed`. Pygames
    /// `test_ass_subscript` enthaelt genau diese Zeile. Das Original
    /// sammelt die Werte ebenfalls erst vollstaendig ein, bevor es den
    /// ersten schreibt - die Reihenfolge ist also nicht nur bequem,
    /// sondern uebertragen.
    fn __setitem__(
        slf: &Bound<'_, PyFRect>,
        op: &Bound<'_, PyAny>,
        wert: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let py = slf.py();
        let vorher = slf.borrow().0;
        let mut alle = [vorher.x, vorher.y, vorher.w, vorher.h];

        if ist_index(op) {
            let i = op.extract::<isize>().map_err(|_| ungueltiger_index())?;
            let i = if (-4..0).contains(&i) { i + 4 } else { i };
            if !(0..=3).contains(&i) {
                return Err(ungueltiger_index());
            }
            alle[i as usize] = flieszt_aus(wert)
                .ok_or_else(|| PyTypeError::new_err("Must assign numeric values"))?;
        } else if op.is(py.Ellipsis()) {
            if let Some(v) = flieszt_aus(wert) {
                alle = [v; 4];
            } else if let Ok(r) = wert.downcast::<PyFRect>() {
                let r = r.borrow().0;
                alle = [r.x, r.y, r.w, r.h];
            } else if ist_folge(wert) {
                if wert.len()? != 4 {
                    return Err(PyTypeError::new_err("Expect a length 4 sequence"));
                }
                for (i, platz) in alle.iter_mut().enumerate() {
                    *platz = flieszt_aus(&wert.get_item(i)?)
                        .ok_or_else(|| PyTypeError::new_err("Expected a number"))?;
                }
            } else {
                return Err(PyTypeError::new_err("Expected an integer or sequence"));
            }
        } else if let Ok(s) = op.downcast::<PySlice>() {
            let teil = s.indices(4)?;
            let werte: Vec<f32> = if let Some(v) = flieszt_aus(wert) {
                vec![v; teil.slicelength]
            } else if ist_folge(wert) {
                if wert.len()? != teil.slicelength {
                    return Err(PyTypeError::new_err(format!(
                        "Expected a length {} sequence",
                        teil.slicelength
                    )));
                }
                let mut v = Vec::with_capacity(teil.slicelength);
                for i in 0..teil.slicelength {
                    v.push(
                        flieszt_aus(&wert.get_item(i)?)
                            .ok_or_else(|| PyTypeError::new_err("Expected a number"))?,
                    );
                }
                v
            } else {
                return Err(PyTypeError::new_err("Expected an integer or sequence"));
            };
            for (k, v) in werte.into_iter().enumerate() {
                alle[(teil.start + teil.step * k as isize) as usize] = v;
            }
        } else {
            return Err(PyTypeError::new_err("Invalid Rect slice"));
        }

        slf.borrow_mut().0 = KernFRect::new(alle[0], alle[1], alle[2], alle[3]);
        Ok(())
    }

    /// pygame lehnt das Loeschen mit einem **TypeError** ab.
    fn __delitem__(&self, _op: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err("item deletion is not supported"))
    }

    /// Muss einen **Iterator** liefern, keine Liste - sonst meldet Python
    /// `iter() returned non-iterator`.
    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let t = PyTuple::new(
            py,
            [
                self.0.x as f64,
                self.0.y as f64,
                self.0.w as f64,
                self.0.h as f64,
            ],
        )?;
        Ok(t.into_any().try_iter()?.into_any().unbind())
    }

    /// `x in f` - eine **Fliesskommazahl**, wenn sie einer der vier
    /// Werte ist; ein Rechteck, wenn es ganz darin liegt.
    ///
    /// **`1 in FRect(0,0,1,2)` ist ein TypeError**, `1.0 in …` dagegen
    /// `True`: das Original prueft hier `PyFloat_Check`, und eine ganze
    /// Zahl ist damit keine Zahl. Pygames eigener Test verlangt genau
    /// das. Der Rechteckzweig fehlte hier ganz - `FRect(2,3,1,1) in r`
    /// gab immer `False`.
    fn __contains__(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        if other.is_instance_of::<PyFloat>() {
            let v = other.extract::<f64>()? as f32;
            return Ok(v == self.0.x || v == self.0.y || v == self.0.w || v == self.0.h);
        }
        match frect_aus_objekt(other) {
            Some(r) => Ok(self.0.contains(r)),
            None => Err(PyTypeError::new_err(
                "'in <pygame.rect.FRect>' requires rect style object or float as left operand",
            )),
        }
    }

    /// Ein Rechteck ohne Flaeche ist unwahr - wie in pygame.
    fn __bool__(&self) -> bool {
        self.0.w != 0.0 && self.0.h != 0.0
    }

    fn __eq__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> Py<PyAny> {
        vergleich(py, self.0, other, |c| c == 0)
    }
    fn __ne__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> Py<PyAny> {
        vergleich(py, self.0, other, |c| c != 0)
    }
    fn __lt__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> Py<PyAny> {
        vergleich(py, self.0, other, |c| c < 0)
    }
    fn __le__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> Py<PyAny> {
        vergleich(py, self.0, other, |c| c <= 0)
    }
    fn __gt__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> Py<PyAny> {
        vergleich(py, self.0, other, |c| c > 0)
    }
    fn __ge__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> Py<PyAny> {
        vergleich(py, self.0, other, |c| c >= 0)
    }

    /// pygame lehnt das ab, weil ein Rechteck veraenderlich ist.
    fn __hash__(&self) -> PyResult<u64> {
        Err(PyTypeError::new_err("unhashable type: 'pygame.rect.FRect'"))
    }

    /// pygame schreibt hier `%f` - **sechs** Nachkommastellen, nicht die
    /// kuerzeste Form. Gemessen: `FRect(9.832, …)` meldet
    /// `FRect(9.832000, …)`, `FRect(35.469, …)` dagegen
    /// `FRect(35.469002, …)` - der gespeicherte `f32` schlaegt durch.
    fn __repr__(&self) -> String {
        format!(
            "FRect({}, {}, {}, {})",
            wie_printf(self.0.x as f64),
            wie_printf(self.0.y as f64),
            wie_printf(self.0.w as f64),
            wie_printf(self.0.h as f64)
        )
    }
    fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// Eine Zahl wie Cs `%f` - **samt der Schreibweise fuer NaN**.
///
/// Rusts `{:.6}` und Microsofts `printf` sind sich bei jeder endlichen
/// Zahl und bei Unendlich einig (`inf`, `-inf`), bei NaN aber nicht.
/// Gemessen an pygame auf dieser Maschine:
///
/// ```text
///   FRect(<NaN ohne Vorzeichenbit>, …)  ->  FRect(nan, …)
///   FRect(<NaN mit Vorzeichenbit>, …)   ->  FRect(-nan(ind), …)
/// ```
///
/// Rust schreibt beide Male `NaN`. Auffallen kann das nur bei
/// `fit` mit einer Groesse von Null - dort entsteht NaN -, und genau dort
/// wich die Schreibweise in 527 von 6000 gewuerfelten Proben ab.
fn wie_printf(v: f64) -> String {
    if v.is_nan() {
        return if v.is_sign_negative() {
            "-nan(ind)".to_string()
        } else {
            "nan".to_string()
        };
    }
    format!("{v:.6}")
}

/// Die zwei Meldungen von `collidedict`.
fn schluessel_fehler(nach_werten: bool) -> PyErr {
    if nach_werten {
        PyTypeError::new_err("dict must have rectstyle values")
    } else {
        PyTypeError::new_err("dict must have rectstyle keys")
    }
}

/// `RectExport_richcompare` - der Reihe nach `x`, `y`, `w`, `h`.
fn vergleich(
    py: Python<'_>,
    eigen: KernFRect,
    other: &Bound<'_, PyAny>,
    passt: impl Fn(i32) -> bool,
) -> Py<PyAny> {
    let Some(o) = frect_aus_objekt(other) else {
        return py.NotImplemented();
    };
    let c = if eigen.x != o.x {
        if eigen.x < o.x { -1 } else { 1 }
    } else if eigen.y != o.y {
        if eigen.y < o.y { -1 } else { 1 }
    } else if eigen.w != o.w {
        if eigen.w < o.w { -1 } else { 1 }
    } else if eigen.h != o.h {
        if eigen.h < o.h { -1 } else { 1 }
    } else {
        0
    };
    passt(c).into_pyobject(py).unwrap().to_owned().into_any().unbind()
}

impl From<KernFRect> for PyFRect {
    fn from(r: KernFRect) -> PyFRect {
        PyFRect(r)
    }
}

impl From<PyFRect> for KernRect {
    fn from(r: PyFRect) -> KernRect {
        KernRect::from(r.0)
    }
}
