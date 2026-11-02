//! rechteck.rs
//! ===========
//! `pygame.Rect`.
//!
//! ## Warum das Makro nur noch die Anker erzeugt
//!
//! `#[pymethods]` nimmt keine Makro-Aufrufe als Einzelposten an - PyO3
//! muss jede Methode sehen, um sie eintragen zu koennen. Also wird aussen
//! herum erzeugt. Achtzehn virtuelle Attribute mal zwei Zugriffe
//! entstehen so aus achtzehn Zeilen Liste statt aus sechsunddreissig von
//! Hand geschriebenen Methoden. Dafuer taugt das Makro, und dafuer bleibt
//! es: die Anker sehen alle gleich aus.
//!
//! Die richtigen Methoden stehen daneben, weil **keine zwei gleich
//! aussehen** - andere Argumentform, andere Fehlermeldung, anderer
//! Rueckgabetyp. Jede braucht `#[pyo3(signature = (*args))]`, denn pygame
//! nimmt ueberall `clip(r)`, `clip((x,y),(w,h))` **und** `clip(x,y,w,h)`
//! an (`METH_FASTCALL`, `rect.c` Zeile 470). Solange sie im Makro
//! standen, gab es nur einen Parameter und damit nur die erste Form, und
//! pygames Testsuite meldete reihenweise
//! `takes 1 positional arguments but 4 were given`. Moeglich macht den
//! zweiten Block PyO3s Merkmal `multiple-pymethods`.
//!
//! **Berichtigter Vermerk.** Hier stand, `#[pyo3(signature = (*args))]`
//! gehe im Makro nicht, weil PyO3 den erzeugten Namen "wegen der
//! Makro-Hygiene nicht wieder findet" - und daran hing die Begruendung
//! fuer den zweiten Block wie fuer `move_to`s Sonderweg in
//! [`crate::frechteck`]. Nachgemessen mit einer eigenen kleinen Kiste
//! (PyO3 0.26, `macro_rules!` erzeugt `#[pymethods]` mit einer Methode
//! `#[pyo3(signature = (*args))] fn viele(...)`): es uebersetzt, und
//! `p.viele(1,2,3)` gibt `3`. Der Satz stimmt also nicht. Er hat aber
//! jahrelang erklaert, warum die Methoden **nur ein** Argument nehmen -
//! und damit die Luecke gedeckt, die pygames Testsuite jetzt in
//! einunddreissig Tests gefunden hat. Wer hier etwas umbaut, baut auf
//! der Messung auf, nicht auf dem alten Satz.
//!
//! ## Vier gemessene Eigenheiten
//!
//! | Frage | pygame-ce 2.5.7 |
//! |---|---|
//! | `repr(Rect(1,2,30,40))` | `Rect(1, 2, 30, 40)` - **nicht** `<rect(...)>` |
//! | `r.clipline((0,0,100,100))` | zwei Paare, nicht vier Zahlen |
//! | dieselbe Strecke daneben | `()` - ein leeres Tupel, kein `None` |
//! | `hash(Rect(...))` | `TypeError` - veraenderlich, also nicht hashbar |
//! | `r[1:3]` | eine **Liste** `[2, 3]` (bei `Color` ist es ein Tupel) |
//!
//! ## Wo die Zahlen herkommen
//!
//! Alles in diesem Modul ist aus `tools/vorlage/rect.c`,
//! `tools/vorlage/rect_impl.h` und `tools/vorlage/base.c` uebertragen -
//! pygame-ce 2.5.7. Wo ein Kommentar eine Zeilennummer nennt, ist sie
//! dort nachzuschlagen; wo er "gemessen" sagt, steht die Messung dabei.

use pyo3::exceptions::{PyAttributeError, PyIndexError, PyTypeError};
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFloat, PyInt, PyList, PySequence, PySlice, PyTuple};

use rustgame_core::rect::Rect as KernRect;

use crate::frechteck::PyFRect;

/// `pygame.Rect`.
///
/// In Rust ist [`KernRect`] `Copy`, in pygame ist ein `Rect` es nicht:
/// `r2 = r1; r2.x = 5` aendert dort auch `r1`. Hier traegt jedes `Rect`
/// seinen Wert im Python-Objekt, und Python-Objekte werden per Verweis
/// gereicht - damit stimmt die Semantik wieder.
// **`subclass`, weil pygame es erlaubt.** Gemessen:
// `type("U", (pygame.Rect,), {})` geht. Ohne den Schalter meldet PyO3
// `TypeError: type '...' is not an acceptable base type`, und an
// **dieser Zeile** ist pygames eigene Testsuite haengengeblieben, bevor
// sie den ersten Test ueberhaupt laden konnte.
//
// Nicht ueberall: `Clock`, `Event` und `Joystick` lassen sich auch in
// pygame **nicht** ableiten - das ist nachgemessen und bleibt so.
#[pyclass(name = "Rect", module = "rustgame_py.rect", subclass, weakref)]
#[derive(Clone, Copy)]
pub struct PyRect(KernRect);

// ---------------------------------------------------------------------
// Die Umwandler - zeilengetreu aus `base.c` und `rect_impl.h`
// ---------------------------------------------------------------------

/// Eine Zahl von Python auf eine ganze - **zur Null hin**.
///
/// Gemessen: `Rect(0,0,10,10).inflate(2.9, -2.9)` gibt
/// `Rect(-1, 1, 12, 8)`. Aus `2.9` wird also `2` und aus `-2.9` wird
/// `-2`; abgerundet waere `-3`. Dasselbe bei `move`.
fn zur_null(v: f64) -> i32 {
    v.trunc() as i32
}

/// Ist das Objekt eine Folge? Das ist `PySequence_Check`, und es trennt
/// schaerfer als `len()`: ein `set` hat eine Laenge, ist aber keine
/// Folge. Gemessen: `Rect({1,2,3,4})` gibt `TypeError`,
/// `Rect(range(4))` dagegen `Rect(0, 1, 2, 3)`.
///
/// **Die zweite Zeile ist kein Zusatz, sondern der Ausgleich fuer einen
/// Unterschied zwischen pygame und PyO3.** `PySequence_Check` fragt nach
/// `tp_as_sequence->sq_item`. Eine Python-Klasse mit `__getitem__`
/// bekommt den Platz von CPython selbst gefuellt, pygames C-Typen fuellen
/// ihn von Hand (`pg_rect_as_sequence`, `rect.c` Zeile 579) - **PyO3
/// traegt `__getitem__` nur in `mp_subscript` ein**. Ohne den Nachschlag
/// waeren `Rect`, `FRect` und `Vector2` fuer den eigenen Code keine
/// Folgen, und zwei Dinge aus pygames Testsuite gingen schief:
/// `r[::-1] = r` (`test_ass_subscript`) und
/// `r.clipline((Vector2(5,6), Vector2(11,19)))`
/// (`test_clipline__two_sequences`).
///
/// Gefragt wird deshalb, was CPython fuer Python-Klassen auch fragt: hat
/// der **Typ** ein `__getitem__`, und ist es kein Woerterbuch? Ein `set`
/// hat kein `__getitem__` und faellt heraus, ein `dict` wird
/// ausdruecklich ausgenommen - genau wie in `PySequence_Check`.
pub(crate) fn ist_folge(o: &Bound<'_, PyAny>) -> bool {
    if o.downcast::<PyDict>().is_ok() {
        return false;
    }
    o.downcast::<PySequence>().is_ok()
        || o.get_type()
            .hasattr(intern!(o.py(), "__getitem__"))
            .unwrap_or(false)
}

/// `pg_IntFromObj` - `base.c` Zeile 389.
///
/// **Fliesskomma wird angenommen und abgeschnitten**, nicht abgelehnt:
/// `Rect(1.9, 2.9, 3.9, 4.9)` ist `Rect(1, 2, 3, 4)`,
/// `Rect(-1.9, …)` ist `Rect(-1, …)`. Hier stand vorher PyO3s eigene
/// `i32`-Auswertung, und die meldete
/// `'float' object cannot be interpreted as an integer`.
pub(crate) fn ganz_aus(o: &Bound<'_, PyAny>) -> Option<i32> {
    if o.is_instance_of::<PyFloat>() {
        return Some(zur_null(o.extract::<f64>().ok()?));
    }
    o.extract::<i32>().ok()
}

/// `pg_TwoIntsFromObj` - `base.c` Zeile 423.
///
/// Der Einzeiler `(1,)` ist ausdruecklich erlaubt und wird ausgepackt -
/// das ist keine Nachsicht, sondern steht so im Original.
pub(crate) fn zwei_ganze_aus(o: &Bound<'_, PyAny>) -> Option<(i32, i32)> {
    let laenge = o.len().ok()?;
    if laenge == 1 && o.is_instance_of::<PyTuple>() {
        return zwei_ganze_aus(&o.get_item(0).ok()?);
    }
    if laenge != 2 || !ist_folge(o) {
        return None;
    }
    let a = ganz_aus(&o.get_item(0).ok()?)?;
    let b = ganz_aus(&o.get_item(1).ok()?)?;
    Some((a, b))
}

/// `pg_FloatFromObj` - `base.c` Zeile 486.
pub(crate) fn flieszt_aus(o: &Bound<'_, PyAny>) -> Option<f32> {
    if o.is_instance_of::<PyFloat>() {
        return Some(o.extract::<f64>().ok()? as f32);
    }
    Some(o.extract::<i32>().ok()? as f32)
}

/// `pg_TwoFloatsFromObj` - `base.c` Zeile 517.
pub(crate) fn zwei_flieszt_aus(o: &Bound<'_, PyAny>) -> Option<(f32, f32)> {
    let laenge = o.len().ok()?;
    if laenge == 1 && o.is_instance_of::<PyTuple>() {
        return zwei_flieszt_aus(&o.get_item(0).ok()?);
    }
    if laenge != 2 || !ist_folge(o) {
        return None;
    }
    let a = flieszt_aus(&o.get_item(0).ok()?)?;
    let b = flieszt_aus(&o.get_item(1).ok()?)?;
    Some((a, b))
}

/// So tief darf ein `rect`-Feld verschachtelt sein.
///
/// **Eine benannte Grenze.** Das Original hat keine: `RectFromObject`
/// ruft sich fuer `obj.rect` ohne Zaehler selbst auf, und ein Objekt,
/// dessen `rect` auf sich selbst zeigt, laesst pygame in den Abgrund
/// laufen. Pygames eigene Testsuite verschachtelt zwei Stufen tief
/// (`_ObjectWithCallableRectAttribute(_ObjectWithRectAttribute(...))`),
/// also ist hier reichlich Luft.
const TIEFE_GRENZE: u32 = 16;

/// `RectExport_RectFromObject` - `rect_impl.h` Zeile 636.
///
/// Nimmt der Reihe nach an: ein `Rect`, ein `FRect`, eine Folge aus vier
/// Zahlen, eine Folge aus zwei Paaren, ein einelementiges **Tupel** (das
/// wird ausgepackt) und zuletzt alles, was ein Feld `rect` hat - das
/// darf auch eine Methode sein und wird dann gerufen.
///
/// Das letzte ist der Grund, warum `pygame.sprite` so bequem ist: ein
/// Sprite **ist** fuer jeden Rechteckaufruf sein Rechteck.
///
/// Gibt `None` statt eines Fehlers zurueck, weil das Original ebenso
/// `NULL` zurueckgibt und **jeder Aufrufer seine eigene Meldung setzt** -
/// `collidedict` sagt "dict must have rectstyle keys", der Konstruktor
/// "Argument must be rect style object".
pub fn rect_aus_objekt(o: &Bound<'_, PyAny>) -> Option<KernRect> {
    rect_aus_objekt_tief(o, 0)
}

fn rect_aus_objekt_tief(o: &Bound<'_, PyAny>, tiefe: u32) -> Option<KernRect> {
    if tiefe > TIEFE_GRENZE {
        return None;
    }
    // Das Original prueft erst auf genau diesen Typ und erst ganz unten
    // auf Unterklassen. Beides trifft hier `downcast`, und dazwischen
    // liegt nur der Folgenzweig - der fuer ein `Rect` dasselbe
    // herausbekaeme, weil `Rect` selbst eine Folge aus vier Zahlen ist.
    if let Ok(r) = o.downcast::<PyRect>() {
        return Some(r.borrow().0);
    }
    // **Ein `FRect` zaehlt auch als `Rect`** und umgekehrt - gemessen:
    // `Rect(FRect(1.5, 2.5, 3.5, 4.5))` ist `Rect(1, 2, 3, 4)`.
    if let Ok(f) = o.downcast::<PyFRect>() {
        return Some(KernRect::from(f.borrow().wert()));
    }

    // Der schnelle Zweig: Liste oder Tupel. Passt die Laenge nicht,
    // bricht das Original hier ab und sucht **kein** `rect`-Feld mehr.
    let schnell = o.is_instance_of::<PyList>() || o.is_instance_of::<PyTuple>();
    if schnell {
        return aus_folge(o, tiefe);
    }
    // Der langsame Zweig: alles andere mit `PySequence_Check`. Passt die
    // Laenge **nicht**, faellt das Original durch zum `rect`-Feld.
    if ist_folge(o) {
        if let Some(r) = aus_folge(o, tiefe) {
            return Some(r);
        }
    }

    // Das Feld `rect` - darf eine Methode sein, dann wird sie gerufen.
    let feld = o.getattr(intern!(o.py(), "rect")).ok()?;
    let feld = if feld.is_callable() {
        feld.call0().ok()?
    } else {
        feld
    };
    rect_aus_objekt_tief(&feld, tiefe + 1)
}

/// Der gemeinsame Kern der beiden Folgenzweige von `RectFromObject`.
fn aus_folge(o: &Bound<'_, PyAny>, tiefe: u32) -> Option<KernRect> {
    match o.len().ok()? {
        4 => Some(KernRect::new(
            ganz_aus(&o.get_item(0).ok()?)?,
            ganz_aus(&o.get_item(1).ok()?)?,
            ganz_aus(&o.get_item(2).ok()?)?,
            ganz_aus(&o.get_item(3).ok()?)?,
        )),
        2 => {
            let (x, y) = zwei_ganze_aus(&o.get_item(0).ok()?)?;
            let (w, h) = zwei_ganze_aus(&o.get_item(1).ok()?)?;
            Some(KernRect::new(x, y, w, h))
        }
        1 if o.is_instance_of::<PyTuple>() => {
            rect_aus_objekt_tief(&o.get_item(0).ok()?, tiefe + 1)
        }
        _ => None,
    }
}

/// `RectExport_RectFromFastcallArgs` - `rect_impl.h` Zeile 784.
///
/// Die drei Schreibweisen jeder Rechteckmethode: eine Angabe, zwei Paare
/// oder vier Zahlen. **Zwei einzelne Zahlen sind keine davon** -
/// gemessen: `Rect(0,0,10,10).clip(1, 2)` gibt `TypeError`, weil `1`
/// kein Zahlenpaar ist.
fn rect_aus_stellen(args: &Bound<'_, PyTuple>) -> Option<KernRect> {
    match args.len() {
        1 => rect_aus_objekt(&args.get_item(0).ok()?),
        4 => Some(KernRect::new(
            ganz_aus(&args.get_item(0).ok()?)?,
            ganz_aus(&args.get_item(1).ok()?)?,
            ganz_aus(&args.get_item(2).ok()?)?,
            ganz_aus(&args.get_item(3).ok()?)?,
        )),
        2 => {
            let (x, y) = zwei_ganze_aus(&args.get_item(0).ok()?)?;
            let (w, h) = zwei_ganze_aus(&args.get_item(1).ok()?)?;
            Some(KernRect::new(x, y, w, h))
        }
        _ => None,
    }
}

/// Was `__reduce__` zurueckgibt: der Typ und seine vier Zahlen.
///
/// Steht als eigener Name da, weil das Paar sonst so tief geschachtelt
/// ist, dass clippy (`type_complexity`) mit Recht widerspricht.
pub(crate) type Bauplan<'py, T> = (Bound<'py, PyAny>, (T, T, T, T));

/// Die Meldung, die pygame fuer ein untaugliches Rechteck-Argument gibt.
pub(crate) fn nicht_rechteckig() -> PyErr {
    PyTypeError::new_err("Argument must be rect style object")
}

/// Die Meldung von `unionall`, `unionall_ip`, `collidelist` und
/// `collidelistall` - gemessen, und **nicht** dieselbe wie bei einem
/// einzelnen Rechteck-Argument.
pub(crate) fn keine_rechteckfolge() -> PyErr {
    PyTypeError::new_err("Argument must be a sequence of rectstyle objects.")
}

/// Die Meldung der Attributsetzer - `RectExport_setwidth` und die
/// neunzehn Geschwister sagen alle dasselbe.
pub(crate) fn ungueltige_zuweisung() -> PyErr {
    PyTypeError::new_err("invalid rect assignment")
}

/// Der Typname fuer die Fehlermeldungen von `collidepoint`.
pub(crate) fn typname(o: &Bound<'_, PyAny>) -> String {
    o.get_type()
        .name()
        .map(|n| n.to_string())
        .unwrap_or_else(|_| "?".to_string())
}

/// `RectExport_pgTwoValuesFromFastcallArgs` - `rect_impl.h` Zeile 939.
///
/// `move`, `move_ip`, `inflate`, `inflate_ip` und `collidepoint` nehmen
/// alle entweder ein Paar oder zwei Zahlen.
fn zwei_werte_aus_stellen(args: &Bound<'_, PyTuple>) -> PyResult<(i32, i32)> {
    match args.len() {
        1 => {
            let a = args.get_item(0)?;
            if let Some(p) = zwei_ganze_aus(&a) {
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
            let x = ganz_aus(&args.get_item(0)?).ok_or_else(|| {
                PyTypeError::new_err(format!(
                    "The first argument must be numeric (got: '{}')",
                    typname(&args.get_item(0).unwrap())
                ))
            })?;
            let y = ganz_aus(&args.get_item(1)?).ok_or_else(|| {
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

/// `four_ints_from_obj` - `rect.c` Zeile 311. Nur `clipline` braucht das.
fn vier_ganze_aus(o: &Bound<'_, PyAny>) -> PyResult<(i32, i32, i32, i32)> {
    let laenge = o.len().unwrap_or(usize::MAX);
    match laenge {
        2 => {
            let a = o.get_item(0)?;
            let (x1, y1) = zwei_ganze_aus(&a).ok_or_else(|| {
                PyTypeError::new_err("number pair expected for first argument")
            })?;
            let b = o.get_item(1)?;
            let (x2, y2) = zwei_ganze_aus(&b).ok_or_else(|| {
                PyTypeError::new_err("number pair expected for second argument")
            })?;
            Ok((x1, y1, x2, y2))
        }
        4 => {
            let n = |i: usize, wort: &str| -> PyResult<i32> {
                ganz_aus(&o.get_item(i)?)
                    .ok_or_else(|| PyTypeError::new_err(format!("number expected for {wort} argument")))
            };
            Ok((n(0, "first")?, n(1, "second")?, n(2, "third")?, n(3, "fourth")?))
        }
        _ => Err(PyTypeError::new_err(format!(
            "sequence argument takes 2 or 4 items ({} given)",
            if laenge == usize::MAX { -1 } else { laenge as i64 }
        ))),
    }
}

/// Die Mehrfaktoren von `scale_by` - `RectExport_scalebyIp`,
/// `rect_impl.h` Zeile 1098.
///
/// Gilt fuer `Rect` **und** `FRect` unveraendert: die Faktoren sind im
/// Original beide Male `float`, auch beim ganzzahligen Rechteck.
pub(crate) fn faktoren_aus(
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<(f32, f32)> {
    if let Some(kw) = kwargs {
        if let Some(wert) = kw.get_item("scale_by")? {
            if kw.len() > 1 {
                return Err(PyTypeError::new_err(
                    "The 'scale_by' keyword cannot be combined with other arguments.",
                ));
            }
            return zwei_flieszt_aus(&wert).ok_or_else(|| {
                PyTypeError::new_err("The 'scale_by' argument must be a number pair")
            });
        }
    }
    // Sonst `PyArg_ParseTupleAndKeywords(args, kwargs, "O|O", {"x","y"})`.
    let mut x_arg = None;
    let mut y_arg = None;
    if !args.is_empty() {
        x_arg = Some(args.get_item(0)?);
    }
    if args.len() > 1 {
        y_arg = Some(args.get_item(1)?);
    }
    if args.len() > 2 {
        return Err(PyTypeError::new_err(
            "function takes at most 2 arguments (3 given)",
        ));
    }
    // **Die Reihenfolge der Fehler ist gemessen, nicht gewaehlt.**
    // `PyArg_ParseTupleAndKeywords` prueft der Reihe nach: zu viele
    // Stellen, derselbe Name zweimal, fehlendes `x`, unbekannter Name.
    // Gemessen an pygame:
    //
    // ```text
    //   scale_by(nope=1)       -> function missing required argument 'x' (pos 1)
    //   scale_by(nope=1, x=2)  -> this function got an unexpected keyword argument 'nope'
    //   scale_by(2, x=3)       -> argument for function given by name ('x') and position (1)
    //   scale_by(2, 3, nope=1) -> function takes at most 2 arguments (3 given)
    // ```
    //
    // Das fehlende `x` schlaegt also den unbekannten Namen - deshalb wird
    // der erst gemerkt und ganz am Ende gemeldet.
    let mut unbekannt: Option<String> = None;
    if let Some(kw) = kwargs {
        for (schluessel, wert) in kw.iter() {
            match schluessel.extract::<String>()?.as_str() {
                "x" if x_arg.is_none() => x_arg = Some(wert),
                "y" if y_arg.is_none() => y_arg = Some(wert),
                name @ ("x" | "y") => {
                    let stelle = if name == "x" { 1 } else { 2 };
                    return Err(PyTypeError::new_err(format!(
                        "argument for function given by name ('{name}') and position ({stelle})"
                    )));
                }
                name => {
                    if unbekannt.is_none() {
                        unbekannt = Some(name.to_string());
                    }
                }
            }
        }
    }
    let Some(x_arg) = x_arg else {
        return Err(PyTypeError::new_err(
            "function missing required argument 'x' (pos 1)",
        ));
    };
    if let Some(name) = unbekannt {
        return Err(PyTypeError::new_err(format!(
            "this function got an unexpected keyword argument '{name}'"
        )));
    }
    // **Erst als Paar versuchen, dann als einzelne Zahl** - `scale_by((2,4))`
    // und `scale_by(2, 4)` sind beide gueltig, und nur diese Reihenfolge
    // trennt sie. Gemessen: `scale_by((2,4), 1)` gibt
    // `TypeError: Cannot pass argument 'y' after passing a sequence scale`.
    if let Some(paar) = zwei_flieszt_aus(&x_arg) {
        if y_arg.is_some() {
            return Err(PyTypeError::new_err(
                "Cannot pass argument 'y' after passing a sequence scale",
            ));
        }
        return Ok(paar);
    }
    let fx = flieszt_aus(&x_arg)
        .ok_or_else(|| PyTypeError::new_err("Argument 'x' must be a number"))?;
    // **Kein Rueckfall auf `fx`.** Die Null ist das Zeichen "nicht
    // angegeben", und der Kern ersetzt sie zeilengetreu wie das Original
    // (`factor_y = (factor_y > 0) ? factor_y : factor_x;`).
    let fy = match y_arg {
        None => 0.0,
        Some(y) => flieszt_aus(&y)
            .ok_or_else(|| PyTypeError::new_err("Argument 'y' must be a number"))?,
    };
    Ok((fx, fy))
}

impl PyRect {
    /// Aus einem Kernrechteck.
    pub fn neu(r: KernRect) -> PyRect {
        PyRect(r)
    }

    /// Das Kernrechteck dahinter.
    pub fn wert(&self) -> KernRect {
        self.0
    }

    /// Ein neues Rechteck **vom Typ des Empfaengers**.
    ///
    /// pygame ruft dafuer `tp_new` mit `Py_TYPE(self)` und schreibt die
    /// vier Werte danach von Hand hinein
    /// (`RectExport_subtypeNew4`, `rect_impl.h` Zeile 814) - **ohne**
    /// `tp_init`. Genau deshalb hat `MyRect(1,2,3,4).copy()` das Feld
    /// nicht, das `MyRect.__init__` setzt; pygames eigene Testsuite
    /// prueft das in `SubclassTest.test_copy` ausdruecklich mit
    /// `assertRaises(AttributeError, ...)`.
    ///
    /// Hier tut es `type(self).__new__(type(self))` - das ist derselbe
    /// Weg (`tp_new` ohne `tp_init`) ohne eigenen unsicheren Block.
    fn neu_wie<'py>(slf: &Bound<'py, PyRect>, r: KernRect) -> PyResult<Bound<'py, PyRect>> {
        let py = slf.py();
        let typ = slf.get_type();
        if typ.is(py.get_type::<PyRect>()) {
            return Bound::new(py, PyRect(r));
        }
        let neu = typ.getattr(intern!(py, "__new__"))?.call1((&typ,))?;
        let neu = neu.downcast_into::<PyRect>()?;
        neu.borrow_mut().0 = r;
        Ok(neu)
    }

    /// Das Rechteck eines Listenelements - mit oder ohne `key`.
    ///
    /// Die drei Meldungen sind gemessen und **anders** als bei
    /// `collidelist`: `collideobjects(5)` sagt
    /// "Argument must be a sequence of objects.", ein untaugliches
    /// Element "Sequence must contain rect or rect-like objects", ein
    /// `key`, der keine Funktion ist,
    /// "Key function must be callable with one argument.".
    fn rechteck_von(
        element: &Bound<'_, PyAny>,
        key: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<KernRect> {
        let gesucht = match key {
            Some(k) => k.call1((element,))?,
            None => element.clone(),
        };
        rect_aus_objekt(&gesucht).ok_or_else(|| {
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

    /// Eine Liste von Rechtecken - fuer `unionall`, `collidelist`, ...
    ///
    /// **Eine Folge, kein Iterator.** Das Original prueft
    /// `PySequence_Check(arg)` und greift dann mit `PySequence_ITEM` zu
    /// (`RectExport_unionall`, `rect_impl.h` Zeile 1216). Gemessen:
    /// `r.unionall(x for x in [...])` gibt in pygame einen TypeError,
    /// `r.unionall((r2,))` geht. Hier stand `try_iter`, und damit ging
    /// beides - bequemer als pygame und darum falsch.
    ///
    /// Dieselbe Meldung gilt fuer eine untaugliche Folge **und** fuer ein
    /// untaugliches Element darin; auch das ist gemessen.
    fn liste_aus(o: &Bound<'_, PyAny>) -> PyResult<Vec<KernRect>> {
        if !ist_folge(o) {
            return Err(keine_rechteckfolge());
        }
        let laenge = o.len()?;
        let mut aus = Vec::with_capacity(laenge);
        for i in 0..laenge {
            aus.push(rect_aus_objekt(&o.get_item(i)?).ok_or_else(keine_rechteckfolge)?);
        }
        Ok(aus)
    }

    /// Setzt eines der virtuellen Attribute ueber seinen Namen.
    ///
    /// Das braucht `Surface.get_rect(center=(x, y))` und `move_to` - und
    /// zwar von aussen, weshalb es hier steht und nicht im
    /// `#[pymethods]`-Block. pygames `move_to` geht denselben Weg
    /// (`PyObject_SetAttr`, `rect_impl.h` Zeile 1060), deshalb wirken
    /// dort auch `x=`, `w=` und `relcenter=`.
    ///
    /// Gemessen: ein unbekannter Name gibt in pygame einen
    /// **AttributeError**, ein untauglicher Wert einen **TypeError**.
    pub fn setze(&mut self, name: &str, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        let zahl = || ganz_aus(wert).ok_or_else(ungueltige_zuweisung);
        let paar = || zwei_ganze_aus(wert).ok_or_else(ungueltige_zuweisung);
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
                    "'pygame.rect.Rect' object has no attribute '{name}' \
                     and no __dict__ for setting new attributes"
                )))
            }
        }
        Ok(())
    }

    /// Wendet einen ganzen Satz Schluesselwoerter an.
    pub fn aus_kwargs(&mut self, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let Some(kw) = kwargs else { return Ok(()) };
        for (schluessel, wert) in kw.iter() {
            let name: String = schluessel.extract()?;
            self.setze(&name, &wert)?;
        }
        Ok(())
    }
}

/// Erzeugt die achtzehn Ankerpaare - siehe Modulkopf.
macro_rules! rect_anker {
    (
        punkte: [$($pl:ident / $ps:ident),* $(,)?],
        zahlen: [$($zl:ident / $zs:ident),* $(,)?],
    ) => {
        #[pymethods]
        impl PyRect {
            $(
                #[getter]
                fn $pl(&self) -> (i32, i32) { self.0.$pl() }
                // **`&Bound` statt `(i32, i32)`.** PyO3s eigene
                // Tupel-Auswertung meldet bei falscher Laenge einen
                // `ValueError` ("expected tuple of length 2, but got
                // tuple of length 1"), pygame einen `TypeError`. Die
                // Testsuite prueft `assertRaises(TypeError)`, und genau
                // daran sind einunddreissig Tests gescheitert.
                #[setter]
                fn $ps(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
                    self.0.$ps(zwei_ganze_aus(v).ok_or_else(ungueltige_zuweisung)?);
                    Ok(())
                }
            )*

            $(
                #[getter]
                fn $zl(&self) -> i32 { self.0.$zl() }
                #[setter]
                fn $zs(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
                    self.0.$zs(ganz_aus(v).ok_or_else(ungueltige_zuweisung)?);
                    Ok(())
                }
            )*
        }
    };
}

rect_anker! {
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
impl PyRect {
    /// `Rect()`, `Rect(r)`, `Rect((x,y),(w,h))` oder `Rect(x,y,w,h)`.
    ///
    /// **Die Angaben werden als ein Stueck gelesen, nicht einzeln.**
    /// pygames `tp_init` reicht das ganze Argumenttupel an
    /// `RectFromObject` weiter (`rect_impl.h` Zeile 894), und daraus
    /// folgt eine Eigenheit, die man von Hand nicht erraten wuerde:
    /// `Rect(1, 2)` ist ein **TypeError**, weil `(1, 2)` als zwei Paare
    /// gelesen wird und `1` kein Paar ist. Gemessen und nachgestellt.
    ///
    /// Schluesselwoerter werden **stillschweigend verworfen**:
    /// `Rect(x=1)` ist `Rect(0, 0, 0, 0)`. Auch das ist gemessen.
    #[new]
    #[pyo3(signature = (*args, **_kwargs))]
    fn new(args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<Self> {
        if args.is_empty() {
            return Ok(PyRect(KernRect::new(0, 0, 0, 0)));
        }
        Ok(PyRect(
            rect_aus_objekt(args.as_any()).ok_or_else(nicht_rechteckig)?,
        ))
    }

    /// `__init__` ist bei pygame **kein Leerlauf**, sondern setzt den
    /// Wert neu: `r.__init__(9,9,9,9)` macht aus `r` das Rechteck
    /// `(9,9,9,9)` (gemessen). PyO3 kennt nur `#[new]`, also steht es
    /// hier als gewoehnliche Methode im Typwoerterbuch.
    ///
    /// Sichtbar wird der Unterschied beim Ableiten, und pygames Testdatei
    /// tut genau das:
    ///
    /// ```python
    /// class MyRect(pygame.Rect):
    ///     def __init__(self, *args, **kwds):
    ///         super().__init__(*args, **kwds)
    /// ```
    ///
    /// Ohne diese Methode landet `super().__init__` bei `object.__init__`
    /// und meldet `takes exactly one argument`; elf Tests der Klasse
    /// `SubclassTest` sind daran gescheitert, bevor der erste `assert`
    /// dran war.
    #[pyo3(signature = (*args, **_kwargs))]
    fn __init__(
        &mut self,
        args: &Bound<'_, PyTuple>,
        _kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        if args.is_empty() {
            return Ok(());
        }
        self.0 = rect_aus_objekt(args.as_any()).ok_or_else(nicht_rechteckig)?;
        Ok(())
    }

    #[getter]
    fn x(&self) -> i32 {
        self.0.x
    }
    #[setter]
    fn set_x(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.0.x = ganz_aus(v).ok_or_else(ungueltige_zuweisung)?;
        Ok(())
    }
    #[getter]
    fn y(&self) -> i32 {
        self.0.y
    }
    #[setter]
    fn set_y(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.0.y = ganz_aus(v).ok_or_else(ungueltige_zuweisung)?;
        Ok(())
    }
    #[getter]
    fn w(&self) -> i32 {
        self.0.w
    }
    #[setter]
    fn set_w(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.0.w = ganz_aus(v).ok_or_else(ungueltige_zuweisung)?;
        Ok(())
    }
    #[getter]
    fn h(&self) -> i32 {
        self.0.h
    }
    #[setter]
    fn set_h(&mut self, v: &Bound<'_, PyAny>) -> PyResult<()> {
        self.0.h = ganz_aus(v).ok_or_else(ungueltige_zuweisung)?;
        Ok(())
    }

    /// Setzt alle vier Werte neu - `r.update(...)`, gibt `None` zurueck.
    #[pyo3(signature = (*args))]
    fn update(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        self.0 = rect_aus_stellen(args).ok_or_else(nicht_rechteckig)?;
        Ok(())
    }

    // ----- Verschieben, vergroessern, skalieren -------------------------

    /// In Python heisst das `move`. In Rust geht das nicht - `move` ist
    /// ein Schluesselwort -, also heisst die Methode `move_` und wird
    /// beim Eintragen umbenannt.
    ///
    /// Ohne diese Umbenennung hiess sie auch in Python `move_`, und ein
    /// Programm mit `r.move(1, 2)` scheiterte. Aufgefallen ist das erst,
    /// als die Abdeckungstabelle die **gebaute Huelle** fragte statt den
    /// Quelltext.
    #[pyo3(name = "move", signature = (*args))]
    fn move_<'py>(
        slf: &Bound<'py, PyRect>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyRect>> {
        let (dx, dy) = zwei_werte_aus_stellen(args)?;
        let r = slf.borrow().0.move_(dx, dy);
        PyRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (*args))]
    fn move_ip(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let (dx, dy) = zwei_werte_aus_stellen(args)?;
        self.0.move_ip(dx, dy);
        Ok(())
    }

    /// `move_to(center=…)` und die neunzehn Geschwister.
    ///
    /// Nimmt **nur** Schluesselwoerter, und mindestens eines: beides ist
    /// gemessen und steht so in `RectExport_moveTo` (`rect_impl.h`
    /// Zeile 1034). Ein unbekannter Name gibt einen AttributeError, kein
    /// TypeError - auch gemessen.
    #[pyo3(signature = (*args, **kwargs))]
    fn move_to<'py>(
        slf: &Bound<'py, PyRect>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Bound<'py, PyRect>> {
        if !args.is_empty() {
            return Err(PyTypeError::new_err("move_to takes no positional arguments"));
        }
        if kwargs.is_none_or(|k| k.is_empty()) {
            return Err(PyTypeError::new_err(
                "move_to cannot be called without keyword arguments",
            ));
        }
        let neu = PyRect::neu_wie(slf, slf.borrow().0)?;
        // Ueber `setze`, weil pygame hier `PyObject_SetAttr` benutzt und
        // damit jedes setzbare Attribut zulaesst - `x`, `w`, `relcenter`.
        {
            let mut leihe = neu.borrow_mut();
            leihe.aus_kwargs(kwargs)?;
        }
        Ok(neu)
    }

    #[pyo3(signature = (*args))]
    fn inflate<'py>(
        slf: &Bound<'py, PyRect>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyRect>> {
        let (dx, dy) = zwei_werte_aus_stellen(args)?;
        let r = slf.borrow().0.inflate(dx, dy);
        PyRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (*args))]
    fn inflate_ip(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let (dx, dy) = zwei_werte_aus_stellen(args)?;
        self.0.inflate_ip(dx, dy);
        Ok(())
    }

    /// `r.scale_by(f)`, `r.scale_by(fx, fy)`, `r.scale_by((fx, fy))`,
    /// `r.scale_by(x=…, y=…)` und `r.scale_by(scale_by=(fx, fy))`.
    ///
    /// Fuenf Schreibweisen, alle im Original. Das Schluesselwort heisst
    /// `scale_by` wie die Methode selbst und nimmt **nur ein Paar** -
    /// gemessen: `scale_by(scale_by=2)` gibt
    /// `TypeError: The 'scale_by' argument must be a number pair`.
    #[pyo3(signature = (*args, **kwargs))]
    fn scale_by<'py>(
        slf: &Bound<'py, PyRect>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Bound<'py, PyRect>> {
        let (fx, fy) = faktoren_aus(args, kwargs)?;
        let r = slf.borrow().0.scale_by(fx, fy);
        PyRect::neu_wie(slf, r)
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

    // ----- Verschnitt und Vereinigung ------------------------------------

    #[pyo3(signature = (*args))]
    fn clamp<'py>(
        slf: &Bound<'py, PyRect>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyRect>> {
        let andere = rect_aus_stellen(args).ok_or_else(nicht_rechteckig)?;
        let r = slf.borrow().0.clamp(andere);
        PyRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (*args))]
    fn clamp_ip(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        self.0
            .clamp_ip(rect_aus_stellen(args).ok_or_else(nicht_rechteckig)?);
        Ok(())
    }

    #[pyo3(signature = (*args))]
    fn clip<'py>(
        slf: &Bound<'py, PyRect>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyRect>> {
        let andere = rect_aus_stellen(args).ok_or_else(nicht_rechteckig)?;
        let r = slf.borrow().0.clip(andere);
        PyRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (*args))]
    fn union<'py>(
        slf: &Bound<'py, PyRect>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyRect>> {
        let andere = rect_aus_stellen(args).ok_or_else(nicht_rechteckig)?;
        let r = slf.borrow().0.union(andere);
        PyRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (*args))]
    fn union_ip(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        self.0
            .union_ip(rect_aus_stellen(args).ok_or_else(nicht_rechteckig)?);
        Ok(())
    }

    /// Nimmt genau **eine** Angabe (`METH_O`), naemlich eine Liste.
    #[pyo3(signature = (others, /))]
    fn unionall<'py>(
        slf: &Bound<'py, PyRect>,
        others: &Bound<'_, PyAny>,
    ) -> PyResult<Bound<'py, PyRect>> {
        let liste = PyRect::liste_aus(others)?;
        let r = slf.borrow().0.unionall(&liste);
        PyRect::neu_wie(slf, r)
    }

    #[pyo3(signature = (others, /))]

    fn unionall_ip(&mut self, others: &Bound<'_, PyAny>) -> PyResult<()> {
        self.0.unionall_ip(&PyRect::liste_aus(others)?);
        Ok(())
    }

    #[pyo3(signature = (*args))]
    fn fit<'py>(
        slf: &Bound<'py, PyRect>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Bound<'py, PyRect>> {
        let andere = rect_aus_stellen(args).ok_or_else(nicht_rechteckig)?;
        let r = slf.borrow().0.fit(andere);
        PyRect::neu_wie(slf, r)
    }

    fn normalize(&mut self) {
        self.0.normalize();
    }

    /// Schneidet eine Strecke am Rechteck zu.
    ///
    /// Fuenf Schreibweisen, alle an pygame gemessen:
    /// `clipline((x1,y1,x2,y2))`, `clipline(((x1,y1),(x2,y2)))`,
    /// `clipline((x1,y1),(x2,y2))`, `clipline(x1,y1,x2,y2)` und
    /// dasselbe mit Listen - und heraus kommt **`((x1,y1),(x2,y2))`**
    /// oder das leere Tupel `()`, wenn die Strecke danebenliegt. Kein
    /// `None`: ein Programm prueft mit `if r.clipline(...):`.
    #[pyo3(signature = (*args))]
    fn clipline(&self, py: Python<'_>, args: &Bound<'_, PyTuple>) -> PyResult<Py<PyAny>> {
        let strecke = match args.len() {
            1 => vier_ganze_aus(&args.get_item(0)?)?,
            2 => {
                let (x1, y1) = zwei_ganze_aus(&args.get_item(0)?).ok_or_else(|| {
                    PyTypeError::new_err("number pair expected for first argument")
                })?;
                let (x2, y2) = zwei_ganze_aus(&args.get_item(1)?).ok_or_else(|| {
                    PyTypeError::new_err("number pair expected for second argument")
                })?;
                (x1, y1, x2, y2)
            }
            4 => {
                let n = |i: usize, wort: &str| -> PyResult<i32> {
                    ganz_aus(&args.get_item(i)?).ok_or_else(|| {
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
            Some((x1, y1, x2, y2)) => PyTuple::new(py, [(x1, y1), (x2, y2)])?.into_any().unbind(),
        })
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
            .colliderect(rect_aus_stellen(args).ok_or_else(nicht_rechteckig)?))
    }

    /// Der Platz des ersten Treffers - **-1**, wenn keiner passt.
    #[pyo3(signature = (others, /))]
    fn collidelist(&self, others: &Bound<'_, PyAny>) -> PyResult<i64> {
        Ok(match self.0.collidelist(&PyRect::liste_aus(others)?) {
            Some(i) => i as i64,
            None => -1,
        })
    }

    #[pyo3(signature = (others, /))]

    fn collidelistall(&self, others: &Bound<'_, PyAny>) -> PyResult<Vec<usize>> {
        Ok(self.0.collidelistall(&PyRect::liste_aus(others)?))
    }

    /// Das erste Paar `(Schluessel, Wert)` aus dem Woerterbuch, dessen
    /// Rechteck dieses hier beruehrt - sonst `None`.
    ///
    /// Ohne `values` sind die **Schluessel** die Rechtecke, mit `values`
    /// die Werte. `RectExport_collidedict`, `rect_impl.h` Zeile 1728.
    ///
    /// Die Methode fehlte ganz, und mit ihr `collidedictall`:
    /// **vierundvierzig** Tests von pygames Testsuite kamen ueber
    /// `AttributeError` nicht hinaus.
    #[pyo3(signature = (rect_dict, values=None))]
    fn collidedict(
        &self,
        py: Python<'_>,
        rect_dict: &Bound<'_, PyAny>,
        values: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        let (woerter, nach_werten) = woerterbuch_pruefen(rect_dict, values)?;
        // Ein flaechenloses Rechteck beruehrt nichts - das Original
        // steigt hier aus, bevor es das Woerterbuch ueberhaupt ansieht.
        if self.0.w != 0 && self.0.h != 0 {
            for (schluessel, wert) in woerter.iter() {
                let anderes = if nach_werten { &wert } else { &schluessel };
                let r = rect_aus_objekt(anderes).ok_or_else(|| schluessel_fehler(nach_werten))?;
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
        if self.0.w != 0 && self.0.h != 0 {
            for (schluessel, wert) in woerter.iter() {
                let anderes = if nach_werten { &wert } else { &schluessel };
                let r = rect_aus_objekt(anderes).ok_or_else(|| schluessel_fehler(nach_werten))?;
                if self.0.colliderect(r) {
                    treffer.append(PyTuple::new(py, [&schluessel, &wert])?)?;
                }
            }
        }
        Ok(treffer.into_any().unbind())
    }

    /// Das erste Element der Liste, dessen Rechteck dieses hier beruehrt.
    ///
    /// Gibt **das Element** zurueck, nicht dessen Rechteck - gemessen an
    /// pygame. Hier stand vorher ein `Rect`, und das ist genau dann
    /// falsch, wenn `collideobjects` seinen Zweck erfuellt: man sucht ja
    /// den Gegner, nicht dessen Umriss.
    ///
    /// `key` bildet ein Element auf sein Rechteck ab. Ohne `key` muss
    /// das Element selbst als Rechteck durchgehen.
    #[pyo3(signature = (liste, key=None))]
    fn collideobjects(
        &self,
        liste: &Bound<'_, PyAny>,
        key: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Option<Py<PyAny>>> {
        let laenge = PyRect::objektfolge(liste, key)?;
        for i in 0..laenge {
            let element = liste.get_item(i)?;
            if self.0.colliderect(PyRect::rechteck_von(&element, key)?) {
                return Ok(Some(element.unbind()));
            }
        }
        Ok(None)
    }

    /// Alle Elemente, deren Rechteck dieses hier beruehrt.
    #[pyo3(signature = (liste, key=None))]
    fn collideobjectsall(
        &self,
        liste: &Bound<'_, PyAny>,
        key: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Vec<Py<PyAny>>> {
        let laenge = PyRect::objektfolge(liste, key)?;
        let mut treffer = Vec::new();
        for i in 0..laenge {
            let element = liste.get_item(i)?;
            if self.0.colliderect(PyRect::rechteck_von(&element, key)?) {
                treffer.push(element.unbind());
            }
        }
        Ok(treffer)
    }

    #[pyo3(signature = (*args))]
    fn contains(&self, args: &Bound<'_, PyTuple>) -> PyResult<bool> {
        Ok(self
            .0
            .contains(rect_aus_stellen(args).ok_or_else(nicht_rechteckig)?))
    }

    fn copy<'py>(slf: &Bound<'py, PyRect>) -> PyResult<Bound<'py, PyRect>> {
        let r = slf.borrow().0;
        PyRect::neu_wie(slf, r)
    }

    fn __copy__<'py>(slf: &Bound<'py, PyRect>) -> PyResult<Bound<'py, PyRect>> {
        PyRect::copy(slf)
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
    /// `RectExport_reduce`, `rect_impl.h` Zeile 2128.
    fn __reduce__<'py>(slf: &Bound<'py, PyRect>) -> PyResult<Bauplan<'py, i32>> {
        let r = slf.borrow().0;
        Ok((slf.get_type().into_any(), (r.x, r.y, r.w, r.h)))
    }

    // ----- Wie eine Folge --------------------------------------------------

    fn __len__(&self) -> usize {
        4
    }

    /// `r[2]` ist die Breite; `r[1:3]` gibt eine **Liste** - gemessen,
    /// und der Unterschied zu `Color`, das dort ein Tupel liefert.
    /// `r[...]` gibt alle vier, ebenfalls als Liste.
    fn __getitem__(&self, py: Python<'_>, op: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let alle = [self.0.x, self.0.y, self.0.w, self.0.h];
        if ist_index(op) {
            // Eine Zahl ausserhalb von `isize` faellt beim Original in
            // dieselbe Kerbe: `PyNumber_AsSsize_t(op, NULL)` klemmt sie
            // fest, und dann meldet `RectExport_item` den IndexError.
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
        slf: &Bound<'_, PyRect>,
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
            alle[i as usize] = ganz_aus(wert)
                .ok_or_else(|| PyTypeError::new_err("Must assign numeric values"))?;
        } else if op.is(py.Ellipsis()) {
            if let Some(v) = ganz_aus(wert) {
                alle = [v; 4];
            } else if let Ok(r) = wert.downcast::<PyRect>() {
                let r = r.borrow().0;
                alle = [r.x, r.y, r.w, r.h];
            } else if ist_folge(wert) {
                if wert.len()? != 4 {
                    return Err(PyTypeError::new_err("Expect a length 4 sequence"));
                }
                for (i, platz) in alle.iter_mut().enumerate() {
                    *platz = ganz_aus(&wert.get_item(i)?)
                        .ok_or_else(|| PyTypeError::new_err("Expected an integer"))?;
                }
            } else {
                return Err(PyTypeError::new_err("Expected an integer or sequence"));
            }
        } else if let Ok(s) = op.downcast::<PySlice>() {
            let teil = s.indices(4)?;
            let werte: Vec<i32> = if let Some(v) = ganz_aus(wert) {
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
                        ganz_aus(&wert.get_item(i)?)
                            .ok_or_else(|| PyTypeError::new_err("Expected an integer"))?,
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

        slf.borrow_mut().0 = KernRect::new(alle[0], alle[1], alle[2], alle[3]);
        Ok(())
    }

    /// pygame lehnt das Loeschen mit einem **TypeError** ab. Ohne diese
    /// Methode kaeme PyO3s eigener `NotImplementedError: can't delete
    /// item` heraus, und der Test prueft `assertRaises(TypeError)`.
    fn __delitem__(&self, _op: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(PyTypeError::new_err("item deletion is not supported"))
    }

    fn __iter__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let t = PyTuple::new(py, [self.0.x, self.0.y, self.0.w, self.0.h])?;
        Ok(t.into_any().try_iter()?.into_any().unbind())
    }

    /// `x in r` - eine **ganze** Zahl, wenn sie einer der vier Werte ist;
    /// ein Rechteck, wenn es ganz darin liegt.
    ///
    /// Gemessen: `1.0 in Rect(0,0,1,2)` ist ein TypeError, `1 in …`
    /// dagegen `True`. Das Original prueft `PyLong_Check` - eine
    /// Fliesskommazahl ist hier also **keine** Zahl.
    fn __contains__(&self, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        if other.is_instance_of::<PyInt>() {
            let v: i32 = other.extract()?;
            return Ok([self.0.x, self.0.y, self.0.w, self.0.h].contains(&v));
        }
        match rect_aus_objekt(other) {
            Some(r) => Ok(self.0.contains(r)),
            None => Err(PyTypeError::new_err(
                "'in <pygame.rect.Rect>' requires rect style object or int as left operand",
            )),
        }
    }

    /// Ein Rechteck ohne Flaeche ist falsch - `if r:` fragt genau das.
    fn __bool__(&self) -> bool {
        self.0.w != 0 && self.0.h != 0
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
        Err(PyTypeError::new_err("unhashable type: 'pygame.rect.Rect'"))
    }

    fn __repr__(&self) -> String {
        format!("Rect({}, {}, {}, {})", self.0.x, self.0.y, self.0.w, self.0.h)
    }
    fn __str__(&self) -> String {
        self.__repr__()
    }
}

// ---------------------------------------------------------------------
// Kleinkram, den beide Rechtecktypen brauchen
// ---------------------------------------------------------------------

/// `PyIndex_Check` - hat das Objekt ein `__index__`?
///
/// Das trennt genau richtig: `int` und `bool` ja, `float`, `str` und
/// `slice` nein. Eine Scheibe darf hier nicht als Zahl durchgehen, sonst
/// waere `r[1:3]` ein Indexfehler statt einer Liste.
///
/// **Nicht ueber `pyo3::ffi::PyIndex_Check`.** Das ist in CPython ein
/// Makro auf `tp_as_number->nb_index`, und die begrenzte Schnittstelle
/// (`abi3`, siehe `Cargo.toml`) traegt es nicht aus: der Binder meldet
/// `unresolved external symbol PyPyIndex_Check`. Gemessen beim Bauen.
/// Die Frage nach `__index__` am Typ ist dasselbe, nur ueber Python.
pub(crate) fn ist_index(o: &Bound<'_, PyAny>) -> bool {
    o.is_instance_of::<PyInt>()
        || o.get_type()
            .hasattr(intern!(o.py(), "__index__"))
            .unwrap_or(false)
}

/// Die Meldung, die pygame fuer `r[4]` gibt.
pub(crate) fn ungueltiger_index() -> PyErr {
    PyIndexError::new_err("Invalid rect Index")
}

/// Prueft das erste Argument von `collidedict` und liest `values`.
///
/// `values` ist im Original ein `p`-Argument, also **irgendetwas
/// Wahres** - gemessen: `collidedict(d, 0)` und `collidedict(d, 1)`
/// gehen beide, nicht nur `True`/`False`.
pub(crate) fn woerterbuch_pruefen<'py>(
    rect_dict: &Bound<'py, PyAny>,
    values: Option<&Bound<'_, PyAny>>,
) -> PyResult<(Bound<'py, PyDict>, bool)> {
    let woerter = rect_dict
        .downcast::<PyDict>()
        .map_err(|_| PyTypeError::new_err("first argument must be a dict"))?;
    let nach_werten = match values {
        None => false,
        Some(v) => v.is_truthy()?,
    };
    Ok((woerter.clone(), nach_werten))
}

/// Die zwei Meldungen von `collidedict`.
fn schluessel_fehler(nach_werten: bool) -> PyErr {
    if nach_werten {
        PyTypeError::new_err("dict must have rectstyle values")
    } else {
        PyTypeError::new_err("dict must have rectstyle keys")
    }
}

/// `RectExport_richcompare` - `rect_impl.h` Zeile 2356.
///
/// pygame vergleicht **der Reihe nach** `x`, `y`, `w`, `h` und kennt
/// dabei alle sechs Vergleiche, nicht nur `==`. Ist die Gegenseite kein
/// Rechteck, kommt `NotImplemented` heraus - dann darf Python selbst
/// entscheiden, und `Rect(...) == "x"` wird zu `False` statt zu einem
/// Fehler.
fn vergleich(
    py: Python<'_>,
    eigen: KernRect,
    other: &Bound<'_, PyAny>,
    passt: impl Fn(i32) -> bool,
) -> Py<PyAny> {
    let Some(o) = rect_aus_objekt(other) else {
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
