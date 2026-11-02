//! vektor.rs
//! =========
//! `pygame.math` - `Vector2`, `Vector3` und die fuenf freien Funktionen.
//!
//! ## Zwei Namen, zwei Regeln - und beide heissen `lerp`
//!
//! Das ist die Stelle, an der ein Nachbau leise auseinanderlaeuft:
//!
//! | Aufruf | pygame-ce 2.5.7 |
//! |---|---|
//! | `Vector2(3,4).lerp(v, 1.5)` | `ValueError: Argument 2 must be in range [0, 1]` |
//! | `pygame.math.lerp(0, 10, 1.5)` | `10.0` - **begrenzt still** |
//! | `pygame.math.remap(0, 10, 0, 100, -1)` | `-10.0` - begrenzt **nicht** |
//!
//! Die dritte Zeile war lange falsch: das Begrenzen steckte im Kern,
//! also auch in `remap`, das dieselbe Rechnung ohne Begrenzung benutzt.
//! Jetzt steht es hier in `math_lerp`, wo `math_lerp` in `math.c` es
//! auch hat. (Kein Doku-Verweis: die Funktion ist modulprivat, und
//! rustdoc warnt zu Recht, dass der Verweis fuer einen Leser der
//! oeffentlichen Doku ins Leere zeigt. Gefunden hat das der erste
//! `cargo doc`-Lauf **in der Huelle** - die Abnahme fuhr ihn nur im
//! Workspace, und der laesst die Huelle aus.)
//!
//! Ebenso: `Vector2(0,0).normalize()` gibt `Err(Fehler::LaengeNull)`, und
//! daraus muss woertlich `Can't normalize Vector of length zero` werden.
//! Der Wortlaut steht im `Display` des Fehlers - er wird von dort
//! genommen und **nicht** abgetippt, damit es nur eine Quelle gibt.
//!
//! ## Wie pygame einen Vektor schreibt - und liest
//!
//! `repr(Vector2(1/3, 2))` ist `Vector2(0.333333, 2)`, `str` davon
//! `[0.333333, 2]`. Das ist Cs `%g` mit sechs geltenden Ziffern - keine
//! Rundung auf sechs Nachkommastellen, und ganze Zahlen ohne `.0`.
//! [`g_zahl`] baut das nach, samt der zweistelligen Hochzahl (`1e+08`,
//! nicht `1e8`).
//!
//! Umgekehrt geht es auch: `Vector2("Vector2(1.2, 3.4)")` ergibt wieder
//! den Vektor. `_vector_coords_from_string` sucht dafuer schlicht nach
//! den Trennstuecken `"Vector2("`, `", "` und `")"` - siehe `aus_text`.
//!
//! ## `from_polar` veraendert an Ort und Stelle
//!
//! Gemessen: `v.from_polar((5, 53.1))` gibt `None` zurueck und setzt `v`.
//! Es sieht aus wie ein Erzeuger und ist keiner.
//!
//! ## Der abgeleitete Typ ueberlebt jede Rechnung
//!
//! ```c
//! #define _vector_subtype_new(x) \
//!     ((pgVector *)(Py_TYPE(x)->tp_new(Py_TYPE(x), NULL, NULL)))
//! ```
//!
//! pygame legt **jedes** Ergebnis mit dem Typ des Empfaengers an, nicht
//! mit `Vector2`. Wer von `Vector2` ableitet, bekommt aus `v + w`,
//! `v.rotate(5)` und sogar `v.elementwise() ** 3` wieder seinen eigenen
//! Typ. Deshalb nehmen hier fast alle Methoden `&Bound<Self>` statt
//! `&self` und gehen durch `neu_wie` - das ist der Grund fuer die
//! vielen `Py<PyAny>` in den Rueckgabetypen.

use pyo3::basic::CompareOp;
use pyo3::exceptions::{
    PyAttributeError, PyIndexError, PyTypeError, PyValueError, PyZeroDivisionError,
};
use pyo3::prelude::*;
use pyo3::types::{PyList, PySlice, PyString, PyTuple};

use rustgame_math as math;
use rustgame_math::{Vector2, Vector3};

/// Ein Fehler aus dem Kern wird zu einem `ValueError` - mit **seinem**
/// Wortlaut, nicht mit einem hier erfundenen.
fn math_fehler(e: math::Fehler) -> PyErr {
    PyValueError::new_err(e.to_string())
}

/// Die eine Stelle, an der pygame eine **eigene** Meldung hat und der
/// Kern sie nicht kennt.
///
/// ```text
/// Vector3(0,0,0).angle_to(v)  ->  angle to zero vector is undefined.
/// ```
///
/// Hier standen einmal drei Stellen, dann eine. Die anderen -
/// `clamp_magnitude`, `slerp`, `project`, `reflect` und
/// `scale_to_length` mit Nullvektor - sind in den Kern gewandert, als
/// dessen Rechnungen zeilengetreu aus `math.c` uebertragen wurden: wenn
/// die Vorbedingung dort schon geprueft wird, darf die Huelle sie nicht
/// ein zweites Mal mit eigenen Worten pruefen. Das ist der Unterschied
/// zwischen einer Meldung, die nur zufaellig gleich lautet, und einer,
/// die es bleiben muss.
fn null_vektor(text: &'static str) -> PyErr {
    PyValueError::new_err(text)
}

/// Der Wortlaut von `vector_move_towards`, wenn das Ziel nicht passt.
fn unpassend() -> PyErr {
    PyTypeError::new_err("Incompatible vector argument")
}

/// Meldet eine `DeprecationWarning` mit pygames Wortlaut.
///
/// pygame hat vier Methodennamen doppelt vergeben und warnt beim
/// jeweils aelteren: `rotate_ip_rad` warnt, `rotate_rad_ip` nicht.
/// Gemessen mit `warnings.catch_warnings(record=True)` - aus dem blossen
/// Lesen der Namen geht nicht hervor, **welcher** der beiden der
/// veraltete ist, und sie legen eher das Gegenteil nahe.
fn veraltet(py: Python<'_>, text: &str) -> PyResult<()> {
    let warnungen = py.import("warnings")?;
    let klasse = py.get_type::<pyo3::exceptions::PyDeprecationWarning>();
    warnungen.call_method1("warn", (text, klasse))?;
    Ok(())
}

/// Cs `%g` mit sechs geltenden Ziffern - so schreibt pygame einen Vektor.
///
/// Rusts `{}` gibt `3` als `3`, aber `0.3333333333333333` in voller
/// Laenge, und `{:e}` schreibt `1e20` statt `1e+20`. Beides passt nicht,
/// also steht die Regel hier ausgeschrieben.
pub fn g_zahl(v: f64) -> String {
    if !v.is_finite() {
        return format!("{v}");
    }
    if v == 0.0 {
        return "0".to_string();
    }
    let exp = v.abs().log10().floor() as i32;
    if !(-4..6).contains(&exp) {
        let mant = kuerzen(format!("{:.5}", v / 10f64.powi(exp)));
        let zeichen = if exp < 0 { '-' } else { '+' };
        return format!("{mant}e{zeichen}{:02}", exp.abs());
    }
    kuerzen(format!("{:.*}", (5 - exp).max(0) as usize, v))
}

/// Nullen am Ende weg, und den Punkt gleich mit.
fn kuerzen(s: String) -> String {
    if !s.contains('.') {
        return s;
    }
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

// ---------------------------------------------------------------------------
//  Was pygame als Vektor durchgehen laesst
// ---------------------------------------------------------------------------

/// `PySequence_Check` - und zwar wirklich das, nicht Pythons `Sequence`.
///
/// ```c
/// #define PySequence_Check(o) (!PyDict_Check(o) && Py_TYPE(o)->tp_as_sequence->sq_item)
/// ```
///
/// PyO3s `downcast::<PySequence>()` ist **enger**: es fragt die
/// Sammlungs-Oberklasse `collections.abc.Sequence`, und eine Klasse, die
/// blosss `__getitem__` und `__len__` hat, ist dort nicht eingetragen.
/// `math_test.py` baut genau so eine (`testConstructionNumericSequence`)
/// und erwartet, dass `Vector2` sie annimmt - deshalb steht die Pruefung
/// hier von Hand.
fn ist_folge(o: &Bound<'_, PyAny>) -> bool {
    !o.is_instance_of::<pyo3::types::PyDict>()
        && o.hasattr(pyo3::intern!(o.py(), "__getitem__"))
            .unwrap_or(false)
}

/// [`ist_folge`] samt Laenge - `None`, wenn eines von beidem fehlt.
fn folge_laenge(o: &Bound<'_, PyAny>) -> Option<usize> {
    if !ist_folge(o) {
        return None;
    }
    o.len().ok()
}

/// `pg_VectorCoordsFromObj` - **genaue** Dimension, kein Fehler gesetzt.
///
/// Ein Vektor der passenden Laenge oder eine Folge mit genau `dim`
/// Zahlen. Wer hier `None` bekommt, entscheidet selbst, was er meldet -
/// und die Aufrufer in `math.c` melden fuenf verschiedene Dinge.
fn koordinaten(o: &Bound<'_, PyAny>, dim: usize) -> Option<Vec<f64>> {
    if dim == 2 {
        if let Ok(v) = o.extract::<PyRef<PyVector2>>() {
            return Some(vec![v.v.x, v.v.y]);
        }
    } else if let Ok(v) = o.extract::<PyRef<PyVector3>>() {
        return Some(vec![v.v.x, v.v.y, v.v.z]);
    }
    if folge_laenge(o)? != dim {
        return None;
    }
    let mut aus = Vec::with_capacity(dim);
    for i in 0..dim {
        aus.push(o.get_item(i).ok()?.extract::<f64>().ok()?);
    }
    Some(aus)
}

/// `pg_VectorCoordsFromObjOldDontUseInNewCode` - nachsichtig bei Vektoren.
///
/// Der Name ist pygames eigener. Der Unterschied zu [`koordinaten`]:
/// ein **Vektor** darf hier eine andere Dimension haben - fehlende
/// Achsen werden null, ueberzaehlige fallen weg. `Vector2(1,2).lerp` mit
/// einem `Vector3` rechnet also, statt zu meckern. Eine **Folge** muss
/// dagegen genau passen, sonst `ValueError`.
fn koordinaten_alt(o: &Bound<'_, PyAny>, dim: usize) -> PyResult<Vec<f64>> {
    let vom_vektor = if let Ok(v) = o.extract::<PyRef<PyVector2>>() {
        Some(vec![v.v.x, v.v.y])
    } else {
        o.extract::<PyRef<PyVector3>>()
            .ok()
            .map(|v| vec![v.v.x, v.v.y, v.v.z])
    };
    if let Some(quelle) = vom_vektor {
        let mut aus = vec![0.0; dim];
        let n = quelle.len().min(dim);
        aus[..n].copy_from_slice(&quelle[..n]);
        return Ok(aus);
    }
    let falsch = || PyValueError::new_err("Sequence has the wrong length.");
    if folge_laenge(o) != Some(dim) {
        return Err(falsch());
    }
    let mut aus = Vec::with_capacity(dim);
    for i in 0..dim {
        aus.push(o.get_item(i)?.extract::<f64>()?);
    }
    Ok(aus)
}

/// Koordinaten mit **pygames Satz fuer diese Rechenart**.
///
/// `koordinaten_alt` sagt ueberall `Sequence has the wrong length.` - das
/// ist pygames Wortlaut an **manchen** Stellen, aber nicht an diesen:
///
/// ```text
/// Vector2(1,2).dot("x")        Cannot perform dot product with this type.
/// Vector3(1,2,3).dot("x")      Cannot perform dot product with this type.
/// Vector3(1,2,3).cross("x")    Incompatible vector argument: cannot
///                              calculate cross product
/// Vector3(1,2,3).angle_to("x") Incompatible vector argument: cannot
///                              calculate angle to
/// ```
///
/// Dazu kommt die **Art**: pygame gibt `TypeError`, `koordinaten_alt`
/// einen `ValueError`. Ein Programm, das `except TypeError` schreibt,
/// sah den Fehler also gar nicht.
fn koordinaten_satz(o: &Bound<'_, PyAny>, dim: usize, satz: &str) -> PyResult<Vec<f64>> {
    koordinaten(o, dim).ok_or_else(|| PyTypeError::new_err(satz.to_string()))
}

/// `RealNumber_Check` - eine Zahl, aber keine komplexe.
///
/// ```c
/// if (obj && PyNumber_Check(obj) && !PyComplex_Check(obj)) return 1;
/// ```
///
/// Der Unterschied zu einem blossen `extract::<f64>()` ist der Fall, in
/// dem das Objekt eine Zahl **sein will** und dabei scheitert:
///
/// ```python
/// class Kaputt:
///     def __float__(self): raise TypeError("Cannot convert to float")
/// pygame.math.Vector2(Kaputt())   #  TypeError, nicht ValueError
/// ```
///
/// Das steht so in `math_test.py` (`testConstructionNumericNonFloat`).
/// Wer nur `if let Ok(z) = x.extract::<f64>()` schreibt, verschluckt den
/// `TypeError` und meldet statt dessen "Vector2 must be set with …".
fn ist_zahl(o: &Bound<'_, PyAny>) -> bool {
    !o.is_instance_of::<pyo3::types::PyComplex>() && ist_numerisch(o)
}

/// `PyNumber_Check` - **mit** komplexen Zahlen.
///
/// ```c
/// return nb && (nb->nb_index || nb->nb_int || nb->nb_float || PyComplex_Check(o));
/// ```
///
/// Der Unterschied zu [`ist_zahl`] ist genau dieses letzte Oder:
/// `clamp` und `smoothstep` lassen eine komplexe Zahl durch die
/// Eingangspruefung und scheitern erst beim Vergleichen, `Vector2(1j)`
/// dagegen schon davor.
fn ist_numerisch(o: &Bound<'_, PyAny>) -> bool {
    if o.is_instance_of::<pyo3::types::PyComplex>() {
        return true;
    }
    ["__float__", "__int__", "__index__"]
        .iter()
        .any(|n| o.hasattr(*n).unwrap_or(false))
}

/// Ein Vektor aus seiner eigenen Schreibweise - `_vector_coords_from_string`.
///
/// `trenner` ist das, was **vor**, **zwischen** und **hinter** den Zahlen
/// steht, also `["Vector2(", ", ", ")"]`. Gesucht wird stur der Reihe
/// nach; alles ausserhalb der Trennstuecke ist der Zahlentext. Deshalb
/// geht auch `Vector2("da steht Vector2(1, 2) drin")`.
fn aus_text(text: &str, trenner: &[&str]) -> Option<Vec<f64>> {
    let dim = trenner.len() - 1;
    let mut start = text.find(trenner[0])? + trenner[0].len();
    let mut aus = Vec::with_capacity(dim);
    for t in trenner.iter().skip(1) {
        let ende = start + text.get(start..)?.find(t)?;
        aus.push(text.get(start..ende)?.trim().parse::<f64>().ok()?);
        start = ende + t.len();
    }
    Some(aus)
}

/// Die Grenzen von `clamp_magnitude` - `vector_clamp_magnitude_ip`.
///
/// ```c
/// switch (nargs) {
///     case 2:
///         min_length = PyFloat_AsDouble(args[0]);
///         /* Fall-through */
///     case 1:
///         max_length = PyFloat_AsDouble(args[nargs - 1]);
///         break;
///     default:
///         return RAISE(PyExc_TypeError,
///                      "Vector clamp function must take one or two floats");
/// }
/// ```
///
/// Eine Angabe ist also die **Obergrenze**, nicht die untere - das
/// `Fall-through` liest `args[nargs - 1]`. Und die Ablehnung hat einen
/// eigenen Wortlaut; mit `#[pyo3(signature = (min, max=None))]` kaeme
/// statt dessen "missing 1 required positional argument", was denselben
/// Typ, aber nicht denselben Text hat.
fn clamp_grenzen(args: &Bound<'_, PyTuple>) -> PyResult<(f64, f64)> {
    match args.len() {
        1 => Ok((0.0, args.get_item(0)?.extract()?)),
        2 => Ok((args.get_item(0)?.extract()?, args.get_item(1)?.extract()?)),
        _ => Err(PyTypeError::new_err(
            "Vector clamp function must take one or two floats",
        )),
    }
}

/// Ein neuer Vektor vom **Typ des Empfaengers** - `_vector_subtype_new`.
///
/// Geht ueber `tp_new` und laesst `tp_init` aus, genau wie das Makro in
/// `math.c`. Die Laenge von `w` entscheidet, ob `PyVector2` oder
/// `PyVector3` erwartet wird; der Empfaenger muss dazu passen, was er
/// bei allen Aufrufern von selbst tut.
fn neu_wie(slf: &Bound<'_, PyAny>, w: &[f64]) -> PyResult<Py<PyAny>> {
    let cls = slf.get_type();
    let obj = cls.call_method1("__new__", (&cls,))?;
    if w.len() == 2 {
        obj.downcast::<PyVector2>()?.borrow_mut().v = Vector2::new(w[0], w[1]);
    } else {
        obj.downcast::<PyVector3>()?.borrow_mut().v = Vector3::new(w[0], w[1], w[2]);
    }
    Ok(obj.unbind())
}

/// Ein blanker `Vector2`/`Vector3` - fuer Sammelnamen, die den Typ
/// wechseln (`Vector2(1,2).xyx` ist ein `Vector3`).
fn neu_blank(py: Python<'_>, w: &[f64]) -> PyResult<Py<PyAny>> {
    Ok(match w.len() {
        2 => PyVector2::neu(Vector2::new(w[0], w[1]))
            .into_pyobject(py)?
            .into_any()
            .unbind(),
        _ => PyVector3::neu(Vector3::new(w[0], w[1], w[2]))
            .into_pyobject(py)?
            .into_any()
            .unbind(),
    })
}

/// `v.yx`, `v.xyx`, `v.xy01` - `vector_getAttr_swizzle`.
///
/// Zwei oder drei Zeichen geben einen Vektor, mehr ein Tupel; `0` und
/// `1` stehen fuer feste Werte. Eine Achse, die es bei dieser Dimension
/// nicht gibt (`Vector2().xz`), laesst den Sammelnamen scheitern - und
/// pygame faellt dann auf die gewoehnliche Suche zurueck, also auf einen
/// `AttributeError`.
fn sammelname_lesen(py: Python<'_>, coords: &[f64], name: &str) -> PyResult<Py<PyAny>> {
    let fehlt = || {
        PyAttributeError::new_err(format!(
            "'rustgame_py.math.Vector{}' object has no attribute '{name}'",
            coords.len()
        ))
    };
    let zeichen: Vec<char> = name.chars().collect();
    if zeichen.len() < 2 || !matches!(zeichen[0], 'x' | 'y' | 'z') {
        return Err(fehlt());
    }
    let mut werte = Vec::with_capacity(zeichen.len());
    for z in &zeichen {
        werte.push(match z {
            'x' | 'y' | 'z' => {
                let i = *z as usize - 'x' as usize;
                *coords.get(i).ok_or_else(fehlt)?
            }
            '0' => 0.0,
            '1' => 1.0,
            _ => return Err(fehlt()),
        });
    }
    if werte.len() == 2 || werte.len() == 3 {
        return neu_blank(py, &werte);
    }
    Ok(PyTuple::new(py, werte)?.into_any().unbind())
}

/// Welche Achsen ein Name schreibend anspricht - oder `None`.
///
/// Anders als beim Lesen gibt es hier **kein** `0` und `1`: was man
/// nicht lesen kann, kann man auch nicht setzen.
fn sammelname_stellen(name: &str, dim: usize) -> Option<Vec<usize>> {
    let zeichen: Vec<char> = name.chars().collect();
    if zeichen.len() < 2 {
        return None;
    }
    zeichen
        .iter()
        .map(|z| match z {
            'x' | 'y' | 'z' => {
                let i = *z as usize - 'x' as usize;
                if i < dim {
                    Some(i)
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect()
}

/// Die Werte fuer einen schreibenden Sammelnamen einsammeln.
///
/// Der Wortlaut endet hier **mit Punkt** (`…conflicts with swizzling.`),
/// bei `Color` ohne. Abgeschrieben, nicht vereinheitlicht.
fn sammelname_setzen(
    coords: &mut [f64],
    stellen: &[usize],
    wert: &Bound<'_, PyAny>,
) -> PyResult<()> {
    let mut gesetzt = vec![false; coords.len()];
    let mut neu = coords.to_vec();
    for (i, &stelle) in stellen.iter().enumerate() {
        if gesetzt[stelle] {
            return Err(PyAttributeError::new_err(
                "Attribute assignment conflicts with swizzling.",
            ));
        }
        gesetzt[stelle] = true;
        let e = wert
            .get_item(i)
            .map_err(|_| PyTypeError::new_err("a sequence is expected"))?;
        neu[stelle] = e.extract::<f64>()?;
    }
    coords.copy_from_slice(&neu);
    Ok(())
}

/// Die gewoehnliche Zuweisung, an `__setattr__` vorbei.
///
/// PyO3 geht bei `__setattr__` - anders als bei `__getattr__` - **nicht**
/// von sich aus zuerst den Normalweg. Ohne diesen Rueckfall liesse sich
/// weder `v.x = 5` setzen noch in einer abgeleiteten Klasse ein eigenes
/// Feld. `object.__setattr__` ist genau das, was pygame mit
/// `PyObject_GenericSetAttr` aufruft.
fn gewoehnlich_setzen(
    obj: &Bound<'_, PyAny>,
    name: &str,
    wert: Option<&Bound<'_, PyAny>>,
) -> PyResult<()> {
    let grund = obj.py().get_type::<PyAny>();
    match wert {
        Some(w) => grund.call_method1("__setattr__", (obj, name, w))?,
        None => grund.call_method1("__delattr__", (obj, name))?,
    };
    Ok(())
}

/// `del v.x` - pygame nennt die Achse beim Namen.
fn achse_loeschen(slf: &Bound<'_, PyAny>, name: &str, dim: usize) -> PyResult<()> {
    let achsen = &["x", "y", "z"][..dim];
    if achsen.contains(&name) {
        return Err(PyTypeError::new_err(format!(
            "Cannot delete the {name} attribute"
        )));
    }
    gewoehnlich_setzen(slf, name, None)
}

/// `v[i]` und `v[a:b]` - `vector_subscript`.
///
/// Der Schnitt gibt eine **Liste**, keinen Vektor und kein Tupel
/// (`vector_GetSlice` baut ein `PyList`). `v[:]` ist also `[1.2, 3.4]`,
/// und genau so prueft es `math_test.py`.
fn feld_lesen(py: Python<'_>, coords: &[f64], i: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let dim = coords.len() as isize;
    if let Ok(s) = i.downcast::<PySlice>() {
        let teil = s.indices(dim)?;
        if teil.slicelength == 0 {
            return Ok(PyList::empty(py).into_any().unbind());
        }
        let mut aus = Vec::with_capacity(teil.slicelength);
        let mut k = teil.start;
        for _ in 0..teil.slicelength {
            aus.push(coords[k as usize]);
            k += teil.step;
        }
        return Ok(PyList::new(py, aus)?.into_any().unbind());
    }
    let Ok(k) = i.extract::<isize>() else {
        return Err(PyTypeError::new_err(format!(
            "vector indices must be integers, not {}",
            i.get_type().name()?
        )));
    };
    let k = if k < 0 { k + dim } else { k };
    if !(0..dim).contains(&k) {
        return Err(PyIndexError::new_err("subscript out of range."));
    }
    Ok(coords[k as usize].into_pyobject(py)?.into_any().unbind())
}

/// `v[i] = x` und `v[a:b] = folge` - `vector_ass_subscript`.
fn feld_setzen(
    coords: &mut [f64],
    i: &Bound<'_, PyAny>,
    wert: &Bound<'_, PyAny>,
) -> PyResult<()> {
    let dim = coords.len() as isize;
    if let Ok(s) = i.downcast::<PySlice>() {
        let teil = s.indices(dim)?;
        let neu = koordinaten_alt(wert, teil.slicelength)?;
        let mut k = teil.start;
        for w in neu {
            coords[k as usize] = w;
            k += teil.step;
        }
        return Ok(());
    }
    let Ok(k) = i.extract::<isize>() else {
        return Err(PyTypeError::new_err(format!(
            "list indices must be integers, not {}",
            i.get_type().name()?
        )));
    };
    let k = if k < 0 { k + dim } else { k };
    if !(0..dim).contains(&k) {
        return Err(PyIndexError::new_err("subscript out of range."));
    }
    coords[k as usize] = wert.extract::<f64>()?;
    Ok(())
}

/// `del v[...]` - zwei Wortlaute, je nach Schrittweite.
fn feld_loeschen(i: &Bound<'_, PyAny>, dim: usize) -> PyResult<()> {
    if let Ok(s) = i.downcast::<PySlice>() {
        if s.indices(dim as isize)?.step != 1 {
            return Err(PyTypeError::new_err(
                "Deletion of vector components is not supported.",
            ));
        }
        return Err(PyTypeError::new_err(
            "Vector object doesn't support item deletion",
        ));
    }
    Err(PyTypeError::new_err("item deletion is not supported"))
}

/// Vergleicht zwei Vektoren mit `epsilon` - `vector_richcompare`.
///
/// **Das ist der Kern der Sache und leicht zu uebersehen:** pygame
/// vergleicht Vektoren **nicht** Bit fuer Bit, sondern mit der Schranke
/// des linken Vektors.
///
/// ```text
/// Vector2(1,0).rotate_rad(math.pi) == (-1.0, 1.2246467991473532e-16)  ->  True
/// ```
///
/// Ohne diese Zeile scheitert rund ein Dutzend Tests in `math_test.py`,
/// die alle aussehen, als ginge es um `reflect`, `slerp` oder
/// `as_spherical` - und in Wirklichkeit ging es jedesmal um `==`.
fn vergleiche(
    coords: &[f64],
    eps: f64,
    other: &Bound<'_, PyAny>,
    op: CompareOp,
) -> PyResult<Py<PyAny>> {
    let py = other.py();
    let ja = |b: bool| Ok(b.into_pyobject(py).unwrap().to_owned().into_any().unbind());
    let Some(o) = koordinaten(other, coords.len()) else {
        // Passt der andere Wert gar nicht, gibt pygame fuer `==`/`!=`
        // schlicht `False`/`True` und sonst `NotImplemented`.
        return match op {
            CompareOp::Eq => ja(false),
            CompareOp::Ne => ja(true),
            _ => Ok(py.NotImplemented()),
        };
    };
    // `diff != diff` faengt NaN ab - so steht es in `math.c`.
    let gleich = coords
        .iter()
        .zip(o.iter())
        .all(|(a, b)| !(a - b).is_nan() && (a - b).abs() < eps);
    match op {
        CompareOp::Eq => ja(gleich),
        CompareOp::Ne => ja(!gleich),
        // **Kein `NotImplemented`**: passt der andere Wert, wirft pygame
        // hier selbst, und zwar mit eigenem Wortlaut. Gemessen:
        // `Vector2(1,0) < Vector2(1,0)` gibt
        // `TypeError: This operation is not supported by vectors`,
        // nicht Pythons "'<' not supported between instances of …".
        _ => Err(PyTypeError::new_err(
            "This operation is not supported by vectors",
        )),
    }
}

/// `round(v)` und `round(v, n)` - `vector___round__`.
fn runden(coords: &[f64], stellen: Option<&Bound<'_, PyAny>>) -> PyResult<Vec<f64>> {
    // Cs `round()`, also **von der Null weg** - Rusts `f64::round` tut
    // dasselbe. `round_ties_even` waere Pythons eingebautes `round` und
    // hier falsch: `vector___round__` ruft die C-Funktion auf.
    let Some(n) = stellen.filter(|o| !o.is_none()) else {
        return Ok(coords.iter().map(|v| v.round()).collect());
    };
    let n: i32 = n
        .extract()
        .map_err(|_| PyTypeError::new_err("Argument must be an integer"))?;
    let f = 10f64.powi(n);
    Ok(coords.iter().map(|v| (v * f).round() / f).collect())
}

/// `v.distance_to(...)` - `_vector_distance_helper`.
///
/// Hat eigene Meldungen, und zwar drei verschiedene je nachdem, woran es
/// lag. Deshalb steht es nicht in [`koordinaten_alt`].
fn abstand_quadrat(coords: &[f64], other: &Bound<'_, PyAny>) -> PyResult<f64> {
    let dim = coords.len();
    let fremd = if let Ok(v) = other.extract::<PyRef<PyVector2>>() {
        Some(vec![v.v.x, v.v.y])
    } else {
        other
            .extract::<PyRef<PyVector3>>()
            .ok()
            .map(|v| vec![v.v.x, v.v.y, v.v.z])
    };
    let o = match fremd {
        Some(o) => {
            if o.len() != dim {
                return Err(PyValueError::new_err("Vectors must be the same size"));
            }
            o
        }
        None => {
            let n = folge_laenge(other)
                .ok_or_else(|| PyTypeError::new_err("A sequence was expected"))?;
            if n != dim {
                return Err(PyValueError::new_err(
                    "Vector and sequence must be the same size",
                ));
            }
            let mut o = Vec::with_capacity(dim);
            for i in 0..dim {
                o.push(other.get_item(i)?.extract::<f64>()?);
            }
            o
        }
    };
    Ok(coords
        .iter()
        .zip(o.iter())
        .map(|(a, b)| (b - a) * (b - a))
        .sum())
}

// ---------------------------------------------------------------------------
//  Vector2
// ---------------------------------------------------------------------------

/// `pygame.math.Vector2`.
#[pyclass(name = "Vector2", module = "rustgame_py.math", subclass)]
#[derive(Clone, Copy)]
pub struct PyVector2 {
    v: Vector2,
    eps: f64,
}

impl PyVector2 {
    /// Aus einem Kernvektor - mit pygames Vorgabe fuer `epsilon`.
    ///
    /// Gemessen: ein **abgeleiteter** Vektor erbt `epsilon`
    /// **nicht**. Wer `v.epsilon = 0.5` setzt und dann
    /// `v.rotate(10)` nimmt, bekommt wieder `1e-06` - denn
    /// `_vector_subtype_new` geht ueber `tp_new`, und das setzt
    /// `VECTOR_EPSILON`. `v.__init__(...)` dagegen laesst die Schranke
    /// **stehen**; auch das ist nachgemessen.
    pub fn neu(v: Vector2) -> PyVector2 {
        PyVector2 {
            v,
            eps: math::EPSILON,
        }
    }
    /// Der Kernvektor dahinter.
    pub fn wert(&self) -> Vector2 {
        self.v
    }
    fn coords(&self) -> [f64; 2] {
        [self.v.x, self.v.y]
    }
    /// `_vector2_set` - der gemeinsame Rumpf von Erzeuger und `update`.
    fn setzen(
        &mut self,
        x: Option<&Bound<'_, PyAny>>,
        y: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let falsch = || {
            PyValueError::new_err(
                "Vector2 must be set with 2 real numbers, a sequence of 2 real numbers, \
                 or another Vector2 instance",
            )
        };
        match x {
            None => {
                self.v.x = 0.0;
                if y.is_none() {
                    self.v.y = 0.0;
                    return Ok(());
                }
            }
            Some(x) => {
                if let Some(c) = koordinaten(x, 2) {
                    // Ein passendes Paar gewinnt - ein zweites Argument
                    // wird dann **nicht** mehr angesehen.
                    self.v = Vector2::new(c[0], c[1]);
                    return Ok(());
                }
                if ist_zahl(x) {
                    self.v.x = x.extract()?;
                    if y.is_none() {
                        self.v.y = self.v.x;
                        return Ok(());
                    }
                } else if let Ok(s) = x.downcast::<PyString>() {
                    let c = aus_text(&s.to_cow()?, &["Vector2(", ", ", ")"]).ok_or_else(falsch)?;
                    self.v = Vector2::new(c[0], c[1]);
                    return Ok(());
                } else {
                    return Err(falsch());
                }
            }
        }
        let y = y.ok_or_else(falsch)?;
        if !ist_zahl(y) {
            return Err(falsch());
        }
        self.v.y = y.extract()?;
        Ok(())
    }
}

#[pymethods]
impl PyVector2 {
    /// `Vector2()`, `Vector2(5)`, `Vector2(x, y)`, `Vector2((x, y))` oder
    /// `Vector2("Vector2(1.2, 3.4)")`.
    ///
    /// Gemessen: `Vector2(5)` ist `(5.0, 5.0)`, nicht `(5.0, 0.0)`.
    #[new]
    #[pyo3(signature = (x=None, y=None))]
    fn new(x: Option<&Bound<'_, PyAny>>, y: Option<&Bound<'_, PyAny>>) -> PyResult<PyVector2> {
        let mut v = PyVector2::neu(Vector2::null());
        v.setzen(x, y)?;
        Ok(v)
    }

    /// `v.__init__(...)` - **setzt den Vektor neu**, kein Leerlauf.
    ///
    /// ```text
    /// v = pygame.math.Vector2(1,2); v.__init__(9,9); tuple(v)  ->  (9.0, 9.0)
    /// ```
    ///
    /// Und, ebenfalls gemessen: `epsilon` bleibt dabei **stehen**. Das
    /// ist der Unterschied zu [`neu`](PyVector2::neu), das die Schranke
    /// zuruecksetzt - `tp_init` fasst sie nicht an, `tp_new` schon.
    ///
    /// Warum es die Methode ueberhaupt gibt, steht bei
    /// [`crate::farbe::PyColor::__init__`]: ohne sie landet
    /// `super().__init__(...)` einer abgeleiteten Klasse bei
    /// `object.__init__`.
    #[pyo3(signature = (x=None, y=None))]
    fn __init__(
        slf: &Bound<'_, Self>,
        x: Option<&Bound<'_, PyAny>>,
        y: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        // Erst eine Kopie, dann zuweisen. Mit `&mut self` waere die Leihe
        // die ganze Zeit offen, und `v.update(v)` - das pygame anstandslos
        // nimmt - gaebe `RuntimeError: Already borrowed`.
        let mut tmp = *slf.borrow();
        tmp.setzen(x, y)?;
        slf.borrow_mut().v = tmp.v;
        Ok(())
    }

    /// Fuer `pickle` und `copy.deepcopy` - `vector2_reduce`.
    fn __reduce__(slf: &Bound<'_, Self>) -> PyResult<(Py<PyAny>, (f64, f64))> {
        let v = slf.borrow().v;
        Ok((slf.get_type().into_any().unbind(), (v.x, v.y)))
    }

    #[getter]
    fn x(&self) -> f64 {
        self.v.x
    }
    #[setter]
    fn set_x(&mut self, v: f64) {
        self.v.x = v;
    }
    #[getter]
    fn y(&self) -> f64 {
        self.v.y
    }
    #[setter]
    fn set_y(&mut self, v: f64) {
        self.v.y = v;
    }

    /// Setzt beide Werte neu und gibt `None` zurueck.
    #[pyo3(signature = (x=None, y=None))]
    fn update(
        slf: &Bound<'_, Self>,
        x: Option<&Bound<'_, PyAny>>,
        y: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        PyVector2::__init__(slf, x, y)
    }

    /// Eine Kopie - **ohne** ein gesetztes `epsilon`.
    ///
    /// `*self` waere hier das Naheliegende und falsch: die Kopie haette
    /// dann die Schranke des Erzeugers. Gemessen an pygame hat sie
    /// wieder `1e-06`. Aufgefallen ist es erst in der Gegenprobe gegen
    /// echtes pygame - im Rust-Test waere es nie aufgefallen, weil der
    /// Kern die Schranke gar nicht mit sich traegt.
    fn copy(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        neu_wie(slf, &slf.borrow().coords())
    }
    fn __copy__(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        PyVector2::copy(slf)
    }

    /// Rechnen je Achse - siehe [`crate::vektor::PyElementwise`].
    fn elementwise(slf: &Bound<'_, Self>) -> PyElementwise {
        PyElementwise {
            besitzer: slf.clone().into_any().unbind(),
            dim: 2,
        }
    }

    // ----- Laenge ---------------------------------------------------------

    fn length(&self) -> f64 {
        self.v.length()
    }
    fn length_squared(&self) -> f64 {
        self.v.length_squared()
    }
    fn magnitude(&self) -> f64 {
        self.v.magnitude()
    }
    fn magnitude_squared(&self) -> f64 {
        self.v.magnitude_squared()
    }

    /// Auf Laenge 1. Ein Nullvektor gibt `ValueError` - Wortlaut aus dem
    /// Kern.
    fn normalize(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        let v = slf.borrow().v.normalize().map_err(math_fehler)?;
        neu_wie(slf, &[v.x, v.y])
    }
    fn normalize_ip(&mut self) -> PyResult<()> {
        self.v = self.v.normalize().map_err(math_fehler)?;
        Ok(())
    }
    /// Die Schranke, ab der pygame zwei Zahlen als gleich ansieht.
    ///
    /// Das ist **kein** Zierrat: gemessen an pygame-ce 2.5.7 ist
    /// `Vector2(0.9, 0).is_normalized()` falsch, nach `v.epsilon = 0.5`
    /// aber wahr. Wer `epsilon` nur ablegt und nie benutzt, baut eine
    /// Eigenschaft, die aussieht wie pygame und nichts tut - schlimmer
    /// als eine, die fehlt.
    ///
    /// Sie wirkt an **acht** Stellen: `is_normalized`, `rotate` und
    /// `rotate_rad` (der Sonderfall fuer rechte Winkel), `slerp`,
    /// `project`, `reflect`, `scale_to_length` - und, am wenigsten
    /// offensichtlich, bei jedem `==` ([`vergleiche`]).
    #[getter]
    fn epsilon(&self) -> f64 {
        self.eps
    }
    #[setter]
    fn set_epsilon(&mut self, v: f64) {
        self.eps = v;
    }
    fn is_normalized(&self) -> bool {
        self.v.is_normalized_mit_eps(self.eps)
    }
    #[pyo3(signature = (length, /))]
    fn scale_to_length(&mut self, length: f64) -> PyResult<()> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let laenge = length;
        self.v = self
            .v
            .scale_to_length_mit_eps(laenge, self.eps)
            .map_err(math_fehler)?;
        Ok(())
    }
    /// `clamp_magnitude(max)` oder `clamp_magnitude(min, max)` - siehe
    /// [`clamp_grenzen`].
    #[pyo3(signature = (*args))]
    fn clamp_magnitude(
        slf: &Bound<'_, Self>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Py<PyAny>> {
        let (min, max) = clamp_grenzen(args)?;
        let v = slf
            .borrow()
            .v
            .clamp_magnitude(min, max)
            .map_err(math_fehler)?;
        neu_wie(slf, &[v.x, v.y])
    }
    #[pyo3(signature = (*args))]
    fn clamp_magnitude_ip(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let (min, max) = clamp_grenzen(args)?;
        self.v = self.v.clamp_magnitude(min, max).map_err(math_fehler)?;
        Ok(())
    }

    // ----- Drehen ---------------------------------------------------------

    #[pyo3(signature = (angle, /))]

    fn rotate(slf: &Bound<'_, Self>, angle: f64) -> PyResult<Py<PyAny>> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let grad = angle;
        PyVector2::rotate_rad(slf, math::deg2rad(grad))
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_ip(&mut self, angle: f64) {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let grad = angle;
        self.v = self.v.rotate_rad_mit_eps(math::deg2rad(grad), self.eps);
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_rad(slf: &Bound<'_, Self>, angle: f64) -> PyResult<Py<PyAny>> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let winkel = angle;
        let s = slf.borrow();
        let v = s.v.rotate_rad_mit_eps(winkel, s.eps);
        drop(s);
        neu_wie(slf, &[v.x, v.y])
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_rad_ip(&mut self, angle: f64) {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let winkel = angle;
        self.v = self.v.rotate_rad_mit_eps(winkel, self.eps);
    }
    /// Der alte Name von [`rotate_rad_ip`](Self::rotate_rad_ip).
    ///
    /// pygame warnt hier - und zwar **nur** hier, nicht bei
    /// `rotate_rad_ip`. Gemessen mit `warnings.catch_warnings`; der
    /// Wortlaut ist woertlich uebernommen, damit ein Programm, das auf
    /// Warnungen prueft, dasselbe zu lesen bekommt.
    #[pyo3(signature = (angle, /))]
    fn rotate_ip_rad(&mut self, py: Python<'_>, angle: f64) -> PyResult<()> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let winkel = angle;
        veraltet(
            py,
            "vector2_rotate_rad_ip() now has all the functionality of vector2_rotate_ip_rad(), so vector2_rotate_ip_rad() will be deprecated in pygame 2.1.1",
        )?;
        self.rotate_rad_ip(winkel);
        Ok(())
    }
    #[pyo3(signature = (other, /))]
    fn angle_to(&self, other: &Bound<'_, PyAny>) -> PyResult<f64> {
        let o = koordinaten(other, 2).ok_or_else(|| {
            PyTypeError::new_err(
                "Incompatible vector argument: cannot calculate angle to",
            )
        })?;
        Ok(self.v.angle_to(Vector2::new(o[0], o[1])))
    }
    /// Der eigene Winkel gegen die x-Achse, in Grad. Nur lesbar.
    #[getter]
    fn angle(&self) -> f64 {
        self.v.angle()
    }
    /// Derselbe Winkel im Bogenmass. Ebenfalls nur lesbar - gemessen:
    /// `v.angle_rad = 1` gibt `AttributeError`, und `Vector3` hat die
    /// Eigenschaft gar nicht.
    #[getter]
    fn angle_rad(&self) -> f64 {
        self.v.angle_rad()
    }
    fn as_polar(&self) -> (f64, f64) {
        self.v.as_polar()
    }
    /// **Veraendert an Ort und Stelle** und gibt `None` zurueck - gemessen.
    ///
    /// Der Name muss `from_polar` lauten, weil pygame ihn so nennt; dass
    /// er trotz `from_` ein `&mut self` nimmt, ist genau die Eigenheit,
    /// die hier abgebildet wird.
    ///
    /// Jede unbrauchbare Angabe ist ein **TypeError**, nicht ein
    /// ValueError: `PyArg_ParseTuple(args, "(dd)")` meldet alles - falsche
    /// Laenge, falscher Inhalt - als TypeError. `math_test.py` prueft das
    /// fuenfmal (`v.from_polar((1,2,3))`, `("ab")`, `((None, 1))`, …).
    #[allow(clippy::wrong_self_convention)]
    #[pyo3(signature = (polar, /))]
    fn from_polar(&mut self, polar: &Bound<'_, PyAny>) -> PyResult<()> {
        let p = koordinaten(polar, 2).ok_or_else(|| {
            PyTypeError::new_err("Vector2.from_polar(): argument 1 must be sequence of length 2")
        })?;
        self.v = Vector2::from_polar(p[0], p[1]);
        Ok(())
    }

    // ----- Zusammenspiel zweier Vektoren -----------------------------------

    #[pyo3(signature = (other, /))]

    fn dot(&self, other: &Bound<'_, PyAny>) -> PyResult<f64> {
        let o = koordinaten_satz(other, 2,
                                 "Cannot perform dot product with this type.")?;
        Ok(self.v.dot(Vector2::new(o[0], o[1])))
    }
    #[pyo3(signature = (other, /))]
    fn cross(&self, other: &Bound<'_, PyAny>) -> PyResult<f64> {
        let o = koordinaten(other, 2).ok_or_else(|| {
            PyTypeError::new_err(
                "Incompatible vector argument: cannot calculate cross product",
            )
        })?;
        Ok(self.v.cross(Vector2::new(o[0], o[1])))
    }
    #[pyo3(signature = (other, /))]
    fn distance_to(&self, other: &Bound<'_, PyAny>) -> PyResult<f64> {
        Ok(abstand_quadrat(&self.coords(), other)?.sqrt())
    }
    #[pyo3(signature = (other, /))]
    fn distance_squared_to(&self, other: &Bound<'_, PyAny>) -> PyResult<f64> {
        abstand_quadrat(&self.coords(), other)
    }
    #[pyo3(signature = (normal, /))]
    fn reflect(slf: &Bound<'_, Self>, normal: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let n = koordinaten_alt(normal, 2)?;
        let s = slf.borrow();
        let v = s
            .v
            .reflect_mit_eps(Vector2::new(n[0], n[1]), s.eps)
            .map_err(math_fehler)?;
        drop(s);
        neu_wie(slf, &[v.x, v.y])
    }
    #[pyo3(signature = (normal, /))]
    fn reflect_ip(slf: &Bound<'_, Self>, normal: &Bound<'_, PyAny>) -> PyResult<()> {
        let n = koordinaten_alt(normal, 2)?;
        let alt = *slf.borrow();
        slf.borrow_mut().v = alt
            .v
            .reflect_mit_eps(Vector2::new(n[0], n[1]), alt.eps)
            .map_err(math_fehler)?;
        Ok(())
    }
    /// Der Anteil in Richtung `other` - `ValueError` bei Laenge null.
    #[pyo3(signature = (other, /))]
    fn project(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let o = koordinaten_alt(other, 2)
            .map_err(|_| PyTypeError::new_err("Expected Vector as argument 1"))?;
        let s = slf.borrow();
        let v = s
            .v
            .project_mit_eps(Vector2::new(o[0], o[1]), s.eps)
            .map_err(math_fehler)?;
        drop(s);
        neu_wie(slf, &[v.x, v.y])
    }
    /// `t` ausserhalb von 0..1 gibt `ValueError` - anders als die freie
    /// Funktion [`math_lerp`], die begrenzt.
    #[pyo3(signature = (other, t, /))]
    fn lerp(
        slf: &Bound<'_, Self>,
        other: &Bound<'_, PyAny>,
        t: f64,
    ) -> PyResult<Py<PyAny>> {
        let o = koordinaten_alt(other, 2)
            .map_err(|_| PyTypeError::new_err("Expected Vector as argument 1"))?;
        let v = slf
            .borrow()
            .v
            .lerp(Vector2::new(o[0], o[1]), t)
            .map_err(math_fehler)?;
        neu_wie(slf, &[v.x, v.y])
    }
    #[pyo3(signature = (other, t, /))]
    fn smoothstep(
        slf: &Bound<'_, Self>,
        other: &Bound<'_, PyAny>,
        t: f64,
    ) -> PyResult<Py<PyAny>> {
        let o = koordinaten_alt(other, 2)?;
        let v = slf.borrow().v.smoothstep(Vector2::new(o[0], o[1]), t);
        neu_wie(slf, &[v.x, v.y])
    }
    #[pyo3(signature = (other, t, /))]
    fn slerp(
        slf: &Bound<'_, Self>,
        other: &Bound<'_, PyAny>,
        t: f64,
    ) -> PyResult<Py<PyAny>> {
        let o = koordinaten_alt(other, 2)
            .map_err(|_| PyTypeError::new_err("Argument 1 must be a vector."))?;
        let s = slf.borrow();
        let v = s
            .v
            .slerp_mit_eps(Vector2::new(o[0], o[1]), t, s.eps)
            .map_err(math_fehler)?;
        drop(s);
        neu_wie(slf, &[v.x, v.y])
    }
    /// `move_towards` ist waehlerischer als seine Nachbarn: es nimmt
    /// `pg_VectorCoordsFromObj` (genaue Dimension) und meldet einen
    /// **TypeError**, wo `lerp` und `reflect` einen ValueError geben.
    #[pyo3(signature = (other, max_distance, /))]
    fn move_towards(
        slf: &Bound<'_, Self>,
        other: &Bound<'_, PyAny>,
        max_distance: f64,
    ) -> PyResult<Py<PyAny>> {
        let o = koordinaten(other, 2).ok_or_else(unpassend)?;
        let v = slf
            .borrow()
            .v
            .move_towards(Vector2::new(o[0], o[1]), max_distance);
        neu_wie(slf, &[v.x, v.y])
    }
    #[pyo3(signature = (other, max_distance, /))]
    fn move_towards_ip(
        slf: &Bound<'_, Self>,
        other: &Bound<'_, PyAny>,
        max_distance: f64,
    ) -> PyResult<()> {
        let o = koordinaten(other, 2).ok_or_else(unpassend)?;
        let alt = slf.borrow().v;
        slf.borrow_mut().v = alt.move_towards(Vector2::new(o[0], o[1]), max_distance);
        Ok(())
    }

    // ----- Wie eine Folge ---------------------------------------------------

    fn __len__(&self) -> usize {
        2
    }
    fn __getitem__(&self, py: Python<'_>, i: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        feld_lesen(py, &self.coords(), i)
    }
    /// `v[0] = 1` und `v[::-1] = v`.
    ///
    /// Die zweite Form steht in `math_test.py` und ist der Grund fuer
    /// `&Bound<Self>` statt `&mut self`: der rechte Wert wird gelesen,
    /// **bevor** die Leihe zum Schreiben aufgemacht wird.
    fn __setitem__(
        slf: &Bound<'_, Self>,
        i: &Bound<'_, PyAny>,
        wert: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let mut c = slf.borrow().coords();
        feld_setzen(&mut c, i, wert)?;
        slf.borrow_mut().v = Vector2::new(c[0], c[1]);
        Ok(())
    }
    fn __delitem__(&self, i: &Bound<'_, PyAny>) -> PyResult<()> {
        feld_loeschen(i, 2)
    }
    /// `x in v` - verglichen wird auf **Gleichheit**, nicht mit `epsilon`.
    fn __contains__(&self, v: &Bound<'_, PyAny>) -> PyResult<bool> {
        let z: f64 = v.extract()?;
        Ok(self.v.x == z || self.v.y == z)
    }
    /// `__safe_for_unpickling__()` - **`True`**, und hier ein **Aufruf**.
    ///
    /// Gemessen ist die Marke bei `Vector2` eine Methode und bei `Rect`
    /// ein Feld. Dieselbe Marke, zwei Gestalten; nachgebaut sind beide.
    fn __safe_for_unpickling__(&self) -> bool {
        true
    }

    /// `iter(v)` - ein [`PyVectorIterator`], der **den Vektor** haelt.
    fn __iter__(slf: &Bound<'_, Self>) -> PyResult<Py<PyVectorIterator>> {
        Py::new(
            slf.py(),
            PyVectorIterator::neu(slf.clone().into_any().unbind(), 2),
        )
    }

    // ----- Sammelnamen ------------------------------------------------------

    fn __getattr__(&self, py: Python<'_>, name: &str) -> PyResult<Py<PyAny>> {
        sammelname_lesen(py, &self.coords(), name)
    }
    fn __setattr__(
        slf: &Bound<'_, Self>,
        name: &str,
        wert: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let Some(stellen) = sammelname_stellen(name, 2) else {
            return gewoehnlich_setzen(slf.as_any(), name, Some(wert));
        };
        let mut c = slf.borrow().coords();
        sammelname_setzen(&mut c, &stellen, wert)?;
        slf.borrow_mut().v = Vector2::new(c[0], c[1]);
        Ok(())
    }
    fn __delattr__(slf: &Bound<'_, Self>, name: &str) -> PyResult<()> {
        achse_loeschen(slf.as_any(), name, 2)
    }

    // ----- Rechnen ----------------------------------------------------------

    fn __add__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        rechne2(slf, other, false, |a, b| a + b)
    }
    /// `(1, 2) + v` - in C erledigt das derselbe Platz wie `v + (1, 2)`,
    /// weil `vector_generic_math` beide Seiten ansieht. PyO3 braucht
    /// dafuer einen eigenen Namen.
    fn __radd__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        rechne2(slf, other, false, |a, b| a + b)
    }
    fn __sub__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        rechne2(slf, other, false, |a, b| a - b)
    }
    /// `(1, 2) - v` - hier **zaehlt** die Reihenfolge, anders als bei
    /// `+`. In `math.c` ist das der eigene Zweig
    /// `OP_SUB | OP_ARG_VECTOR | OP_ARG_REVERSE`.
    fn __rsub__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        rechne2(slf, other, true, |a, b| a - b)
    }
    /// Vektor mal Zahl skaliert, Vektor mal Vektor ist das Skalarprodukt.
    fn __mul__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let c = slf.borrow().coords();
        if let Some(o) = koordinaten(other, 2) {
            let p = c[0] * o[0] + c[1] * o[1];
            return Ok(p.into_pyobject(py)?.into_any().unbind());
        }
        let Ok(f) = other.extract::<f64>() else {
            return Ok(py.NotImplemented());
        };
        neu_wie(slf, &[c[0] * f, c[1] * f])
    }
    fn __rmul__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        PyVector2::__mul__(slf, other)
    }
    fn __truediv__(slf: &Bound<'_, Self>, f: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        teile2(slf, f, false)
    }
    /// `v // 2` - `nb_floor_divide` gibt es bei Vektoren auch, und es
    /// rechnet `floor(x * (1/f))`, nicht `floor(x / f)`.
    fn __floordiv__(slf: &Bound<'_, Self>, f: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        teile2(slf, f, true)
    }

    fn __neg__(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        let c = slf.borrow().coords();
        neu_wie(slf, &[-c[0], -c[1]])
    }
    fn __pos__(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        neu_wie(slf, &slf.borrow().coords())
    }
    fn __bool__(&self) -> bool {
        self.v.x != 0.0 || self.v.y != 0.0
    }
    #[pyo3(signature = (stellen=None))]
    fn __round__(
        slf: &Bound<'_, Self>,
        stellen: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        let c = runden(&slf.borrow().coords(), stellen)?;
        neu_wie(slf, &c)
    }

    /// Nur `==` und `!=`, und beide **mit `epsilon`** - siehe
    /// [`vergleiche`].
    fn __richcmp__(
        &self,
        other: &Bound<'_, PyAny>,
        op: CompareOp,
    ) -> PyResult<Py<PyAny>> {
        vergleiche(&self.coords(), self.eps, other, op)
    }

    fn __repr__(&self) -> String {
        format!("Vector2({}, {})", g_zahl(self.v.x), g_zahl(self.v.y))
    }
    fn __str__(&self) -> String {
        format!("[{}, {}]", g_zahl(self.v.x), g_zahl(self.v.y))
    }
}

/// `+` und `-` fuer `Vector2` - `vector_generic_math`.
///
/// `gedreht` ist der Fall `(1, 2) - v`: dann steht der fremde Wert
/// links. Passt der andere Wert nicht zu einem Vektor, kommt
/// `NotImplemented` - und **nicht** eine eigene Fehlermeldung, damit
/// Python seinen gewohnten `TypeError` bauen kann.
fn rechne2(
    slf: &Bound<'_, PyVector2>,
    other: &Bound<'_, PyAny>,
    gedreht: bool,
    f: impl Fn(f64, f64) -> f64,
) -> PyResult<Py<PyAny>> {
    let Some(o) = koordinaten(other, 2) else {
        return Ok(slf.py().NotImplemented());
    };
    let c = slf.borrow().coords();
    let (a, b): (&[f64], &[f64]) = if gedreht { (&o, &c) } else { (&c, &o) };
    neu_wie(slf, &[f(a[0], b[0]), f(a[1], b[1])])
}

/// `/` und `//` fuer `Vector2`.
///
/// ```c
/// tmp = 1. / tmp;
/// ret->coords[i] = vec_coords[i] * tmp;        /* bzw. floor(...) */
/// ```
///
/// **Kehrwert und Multiplikation**, nicht Division - das ist ein
/// anderes letztes Bit als `x / f`.
fn teile2(
    slf: &Bound<'_, PyVector2>,
    f: &Bound<'_, PyAny>,
    ganz: bool,
) -> PyResult<Py<PyAny>> {
    if koordinaten(f, 2).is_some() {
        return Ok(slf.py().NotImplemented());
    }
    let Ok(t) = f.extract::<f64>() else {
        return Ok(slf.py().NotImplemented());
    };
    if t == 0.0 {
        // Woertlich wie pygame - dort steht **nicht** "float".
        return Err(PyZeroDivisionError::new_err("division by zero"));
    }
    let k = 1.0 / t;
    let c = slf.borrow().coords();
    let w: Vec<f64> = c
        .iter()
        .map(|x| if ganz { (x * k).floor() } else { x * k })
        .collect();
    neu_wie(slf, &w)
}

// ---------------------------------------------------------------------------
//  Vector3
// ---------------------------------------------------------------------------

/// `pygame.math.Vector3`.
#[pyclass(name = "Vector3", module = "rustgame_py.math", subclass)]
#[derive(Clone, Copy)]
pub struct PyVector3 {
    v: Vector3,
    eps: f64,
}

impl PyVector3 {
    /// Aus einem Kernvektor - siehe [`PyVector2::neu`].
    pub fn neu(v: Vector3) -> PyVector3 {
        PyVector3 {
            v,
            eps: math::EPSILON,
        }
    }
    fn coords(&self) -> [f64; 3] {
        [self.v.x, self.v.y, self.v.z]
    }
    /// `_vector3_set`.
    fn setzen(
        &mut self,
        x: Option<&Bound<'_, PyAny>>,
        y: Option<&Bound<'_, PyAny>>,
        z: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let falsch = || {
            PyValueError::new_err(
                "Vector3 must be set with 3 real numbers, a sequence of 3 real numbers, \
                 or another Vector3 instance",
            )
        };
        match x {
            None => {
                self.v = Vector3::null();
                return Ok(());
            }
            Some(x) => {
                if let Some(c) = koordinaten(x, 3) {
                    self.v = Vector3::new(c[0], c[1], c[2]);
                    return Ok(());
                }
                if ist_zahl(x) {
                    let w: f64 = x.extract()?;
                    self.v.x = w;
                    if y.is_none() && z.is_none() {
                        self.v.y = w;
                        self.v.z = w;
                        return Ok(());
                    }
                } else if let Ok(s) = x.downcast::<PyString>() {
                    let c = aus_text(&s.to_cow()?, &["Vector3(", ", ", ", ", ")"])
                        .ok_or_else(falsch)?;
                    self.v = Vector3::new(c[0], c[1], c[2]);
                    return Ok(());
                } else {
                    return Err(falsch());
                }
            }
        }
        // `Vector3(1, 2)` ist ein **ValueError**, kein TypeError - in
        // `_vector3_set` steht dafuer `if (y && !z) goto error;`.
        match (y, z) {
            (Some(y), Some(z)) => {
                if !ist_zahl(y) || !ist_zahl(z) {
                    return Err(falsch());
                }
                self.v.y = y.extract()?;
                self.v.z = z.extract()?;
                Ok(())
            }
            _ => Err(falsch()),
        }
    }
}

#[pymethods]
impl PyVector3 {
    /// `Vector3()`, `Vector3(5)`, `Vector3(x, y, z)`, `Vector3((x,y,z))`
    /// oder `Vector3("Vector3(1, 2, 3)")`.
    #[new]
    #[pyo3(signature = (x=None, y=None, z=None))]
    fn new(
        x: Option<&Bound<'_, PyAny>>,
        y: Option<&Bound<'_, PyAny>>,
        z: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<PyVector3> {
        let mut v = PyVector3::neu(Vector3::null());
        v.setzen(x, y, z)?;
        Ok(v)
    }

    /// Siehe [`PyVector2::__init__`] - dasselbe mit drei Achsen.
    #[pyo3(signature = (x=None, y=None, z=None))]
    fn __init__(
        slf: &Bound<'_, Self>,
        x: Option<&Bound<'_, PyAny>>,
        y: Option<&Bound<'_, PyAny>>,
        z: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let mut tmp = *slf.borrow();
        tmp.setzen(x, y, z)?;
        slf.borrow_mut().v = tmp.v;
        Ok(())
    }

    fn __reduce__(slf: &Bound<'_, Self>) -> PyResult<(Py<PyAny>, (f64, f64, f64))> {
        let v = slf.borrow().v;
        Ok((slf.get_type().into_any().unbind(), (v.x, v.y, v.z)))
    }

    #[getter]
    fn x(&self) -> f64 {
        self.v.x
    }
    #[setter]
    fn set_x(&mut self, v: f64) {
        self.v.x = v;
    }
    #[getter]
    fn y(&self) -> f64 {
        self.v.y
    }
    #[setter]
    fn set_y(&mut self, v: f64) {
        self.v.y = v;
    }
    #[getter]
    fn z(&self) -> f64 {
        self.v.z
    }
    #[setter]
    fn set_z(&mut self, v: f64) {
        self.v.z = v;
    }

    #[pyo3(signature = (x=None, y=None, z=None))]
    fn update(
        slf: &Bound<'_, Self>,
        x: Option<&Bound<'_, PyAny>>,
        y: Option<&Bound<'_, PyAny>>,
        z: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        PyVector3::__init__(slf, x, y, z)
    }

    /// Eine Kopie - **ohne** ein gesetztes `epsilon`, siehe
    /// [`PyVector2::copy`].
    fn copy(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        neu_wie(slf, &slf.borrow().coords())
    }
    /// `copy.copy(v)` - hatte bei `Vector3` gefehlt, bei `Vector2` nicht.
    /// Die Abdeckungstabelle uebergeht Namen mit Unterstrich davor, also
    /// hat sie es nicht gemeldet.
    fn __copy__(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        PyVector3::copy(slf)
    }

    /// Rechnen je Achse - siehe [`crate::vektor::PyElementwise`].
    fn elementwise(slf: &Bound<'_, Self>) -> PyElementwise {
        PyElementwise {
            besitzer: slf.clone().into_any().unbind(),
            dim: 3,
        }
    }

    fn length(&self) -> f64 {
        self.v.length()
    }
    fn length_squared(&self) -> f64 {
        self.v.length_squared()
    }
    fn magnitude(&self) -> f64 {
        self.v.magnitude()
    }
    fn magnitude_squared(&self) -> f64 {
        self.v.magnitude_squared()
    }
    fn normalize(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        let v = slf.borrow().v.normalize().map_err(math_fehler)?;
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    fn normalize_ip(&mut self) -> PyResult<()> {
        self.v = self.v.normalize().map_err(math_fehler)?;
        Ok(())
    }
    /// Siehe [`PyVector2::epsilon`] - hier gilt dasselbe.
    #[getter]
    fn epsilon(&self) -> f64 {
        self.eps
    }
    #[setter]
    fn set_epsilon(&mut self, v: f64) {
        self.eps = v;
    }
    fn is_normalized(&self) -> bool {
        self.v.is_normalized_mit_eps(self.eps)
    }
    #[pyo3(signature = (length, /))]
    fn scale_to_length(&mut self, length: f64) -> PyResult<()> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let laenge = length;
        self.v = self
            .v
            .scale_to_length_mit_eps(laenge, self.eps)
            .map_err(math_fehler)?;
        Ok(())
    }
    /// Siehe [`clamp_grenzen`].
    #[pyo3(signature = (*args))]
    fn clamp_magnitude(
        slf: &Bound<'_, Self>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Py<PyAny>> {
        let (min, max) = clamp_grenzen(args)?;
        let v = slf
            .borrow()
            .v
            .clamp_magnitude(min, max)
            .map_err(math_fehler)?;
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    #[pyo3(signature = (*args))]
    fn clamp_magnitude_ip(&mut self, args: &Bound<'_, PyTuple>) -> PyResult<()> {
        let (min, max) = clamp_grenzen(args)?;
        self.v = self.v.clamp_magnitude(min, max).map_err(math_fehler)?;
        Ok(())
    }

    #[pyo3(signature = (angle, achse, /))]

    fn rotate(
        slf: &Bound<'_, Self>,
        angle: f64,
        achse: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let grad = angle;
        PyVector3::rotate_rad(slf, math::deg2rad(grad), achse)
    }
    #[pyo3(signature = (angle, achse, /))]
    fn rotate_ip(
        slf: &Bound<'_, Self>,
        angle: f64,
        achse: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let grad = angle;
        PyVector3::rotate_rad_ip(slf, math::deg2rad(grad), achse)
    }
    #[pyo3(signature = (angle, achse, /))]
    fn rotate_rad(
        slf: &Bound<'_, Self>,
        angle: f64,
        achse: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let winkel = angle;
        let a = koordinaten_alt(achse, 3)?;
        let s = slf.borrow();
        let v = s
            .v
            .rotate_rad_mit_eps(winkel, Vector3::new(a[0], a[1], a[2]), s.eps)
            .map_err(math_fehler)?;
        drop(s);
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    #[pyo3(signature = (angle, achse, /))]
    fn rotate_rad_ip(
        slf: &Bound<'_, Self>,
        angle: f64,
        achse: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let winkel = angle;
        let a = koordinaten_alt(achse, 3)?;
        let alt = *slf.borrow();
        slf.borrow_mut().v = alt
            .v
            .rotate_rad_mit_eps(winkel, Vector3::new(a[0], a[1], a[2]), alt.eps)
            .map_err(math_fehler)?;
        Ok(())
    }
    /// Der alte Name von [`rotate_rad_ip`](Self::rotate_rad_ip) - warnt.
    #[pyo3(signature = (angle, achse, /))]
    fn rotate_ip_rad(
        slf: &Bound<'_, Self>,
        angle: f64,
        achse: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let winkel = angle;
        veraltet(
            slf.py(),
            "vector3_rotate_rad_ip() now has all the functionality of vector3_rotate_ip_rad(), so vector3_rotate_ip_rad() will be deprecated in pygame 2.1.1",
        )?;
        PyVector3::rotate_rad_ip(slf, winkel, achse)
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_x(slf: &Bound<'_, Self>, angle: f64) -> PyResult<Py<PyAny>> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let grad = angle;
        let v = slf.borrow().v.rotate_x(grad);
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_x_ip(&mut self, angle: f64) {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let grad = angle;
        self.v = self.v.rotate_x(grad);
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_x_rad(slf: &Bound<'_, Self>, angle: f64) -> PyResult<Py<PyAny>> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let a = angle;
        let v = slf.borrow().v.rotate_x_rad(a);
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_x_rad_ip(&mut self, angle: f64) {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let a = angle;
        self.v = self.v.rotate_x_rad(a);
    }
    /// Der alte Name von `rotate_x_rad_ip` - warnt, siehe [`veraltet`].
    #[pyo3(signature = (angle, /))]
    fn rotate_x_ip_rad(&mut self, py: Python<'_>, angle: f64) -> PyResult<()> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let a = angle;
        veraltet(
            py,
            "vector3_rotate_x_rad_ip() now has all the functionality of vector3_rotate_x_ip_rad(), so vector3_rotate_x_ip_rad() will be deprecated in pygame 2.1.1",
        )?;
        self.rotate_x_rad_ip(a);
        Ok(())
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_y(slf: &Bound<'_, Self>, angle: f64) -> PyResult<Py<PyAny>> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let grad = angle;
        let v = slf.borrow().v.rotate_y(grad);
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_y_ip(&mut self, angle: f64) {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let grad = angle;
        self.v = self.v.rotate_y(grad);
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_y_rad(slf: &Bound<'_, Self>, angle: f64) -> PyResult<Py<PyAny>> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let a = angle;
        let v = slf.borrow().v.rotate_y_rad(a);
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_y_rad_ip(&mut self, angle: f64) {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let a = angle;
        self.v = self.v.rotate_y_rad(a);
    }
    /// Der alte Name von `rotate_y_rad_ip` - warnt, siehe [`veraltet`].
    #[pyo3(signature = (angle, /))]
    fn rotate_y_ip_rad(&mut self, py: Python<'_>, angle: f64) -> PyResult<()> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let a = angle;
        veraltet(
            py,
            "vector3_rotate_y_rad_ip() now has all the functionality of vector3_rotate_y_ip_rad(), so vector3_rotate_y_ip_rad() will be deprecated in pygame 2.1.1",
        )?;
        self.rotate_y_rad_ip(a);
        Ok(())
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_z(slf: &Bound<'_, Self>, angle: f64) -> PyResult<Py<PyAny>> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let grad = angle;
        let v = slf.borrow().v.rotate_z(grad);
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_z_ip(&mut self, angle: f64) {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let grad = angle;
        self.v = self.v.rotate_z(grad);
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_z_rad(slf: &Bound<'_, Self>, angle: f64) -> PyResult<Py<PyAny>> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let a = angle;
        let v = slf.borrow().v.rotate_z_rad(a);
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    #[pyo3(signature = (angle, /))]
    fn rotate_z_rad_ip(&mut self, angle: f64) {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let a = angle;
        self.v = self.v.rotate_z_rad(a);
    }
    /// Der alte Name von `rotate_z_rad_ip` - warnt, siehe [`veraltet`].
    #[pyo3(signature = (angle, /))]
    fn rotate_z_ip_rad(&mut self, py: Python<'_>, angle: f64) -> PyResult<()> {
        // Der oeffentliche Name ist pygames (`math.pyi`);
        // der deutsche lebt im Rumpf.
        let a = angle;
        veraltet(
            py,
            "vector3_rotate_z_rad_ip() now has all the functionality of vector3_rotate_z_ip_rad(), so vector3_rotate_z_ip_rad() will be deprecated in pygame 2.1.1",
        )?;
        self.rotate_z_rad_ip(a);
        Ok(())
    }
    fn as_spherical(&self) -> (f64, f64, f64) {
        self.v.as_spherical()
    }
    /// Wie bei `Vector2`: **veraendert an Ort und Stelle**.
    #[allow(clippy::wrong_self_convention)]
    #[pyo3(signature = (s, /))]
    fn from_spherical(&mut self, s: &Bound<'_, PyAny>) -> PyResult<()> {
        let p = koordinaten(s, 3).ok_or_else(|| {
            PyTypeError::new_err(
                "vector3_from_spherical(): argument 1 must be sequence of length 3",
            )
        })?;
        self.v = Vector3::from_spherical(p[0], p[1], p[2]);
        Ok(())
    }

    #[pyo3(signature = (other, /))]

    fn dot(&self, other: &Bound<'_, PyAny>) -> PyResult<f64> {
        let o = koordinaten_satz(other, 3,
                                 "Cannot perform dot product with this type.")?;
        Ok(self.v.dot(Vector3::new(o[0], o[1], o[2])))
    }
    #[pyo3(signature = (other, /))]
    fn cross(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let o = koordinaten_satz(
            other,
            3,
            "Incompatible vector argument: cannot calculate cross product",
        )?;
        let v = slf.borrow().v.cross(Vector3::new(o[0], o[1], o[2]));
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    #[pyo3(signature = (other, /))]
    fn distance_to(&self, other: &Bound<'_, PyAny>) -> PyResult<f64> {
        Ok(abstand_quadrat(&self.coords(), other)?.sqrt())
    }
    #[pyo3(signature = (other, /))]
    fn distance_squared_to(&self, other: &Bound<'_, PyAny>) -> PyResult<f64> {
        abstand_quadrat(&self.coords(), other)
    }
    #[pyo3(signature = (other, /))]
    fn angle_to(&self, other: &Bound<'_, PyAny>) -> PyResult<f64> {
        let o = koordinaten_satz(
            other,
            3,
            "Incompatible vector argument: cannot calculate angle to",
        )?;
        let o = Vector3::new(o[0], o[1], o[2]);
        if self.v.length() == 0.0 || o.length() == 0.0 {
            return Err(null_vektor("angle to zero vector is undefined."));
        }
        Ok(self.v.angle_to(o))
    }
    #[pyo3(signature = (normal, /))]
    fn reflect(slf: &Bound<'_, Self>, normal: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let n = koordinaten_alt(normal, 3)?;
        let s = slf.borrow();
        let v = s
            .v
            .reflect_mit_eps(Vector3::new(n[0], n[1], n[2]), s.eps)
            .map_err(math_fehler)?;
        drop(s);
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    #[pyo3(signature = (normal, /))]
    fn reflect_ip(slf: &Bound<'_, Self>, normal: &Bound<'_, PyAny>) -> PyResult<()> {
        let n = koordinaten_alt(normal, 3)?;
        let alt = *slf.borrow();
        slf.borrow_mut().v = alt
            .v
            .reflect_mit_eps(Vector3::new(n[0], n[1], n[2]), alt.eps)
            .map_err(math_fehler)?;
        Ok(())
    }
    #[pyo3(signature = (other, /))]
    fn project(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let o = koordinaten_alt(other, 3)
            .map_err(|_| PyTypeError::new_err("Expected Vector as argument 1"))?;
        let s = slf.borrow();
        let v = s
            .v
            .project_mit_eps(Vector3::new(o[0], o[1], o[2]), s.eps)
            .map_err(math_fehler)?;
        drop(s);
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    #[pyo3(signature = (other, t, /))]
    fn lerp(
        slf: &Bound<'_, Self>,
        other: &Bound<'_, PyAny>,
        t: f64,
    ) -> PyResult<Py<PyAny>> {
        let o = koordinaten_alt(other, 3)
            .map_err(|_| PyTypeError::new_err("Expected Vector as argument 1"))?;
        let v = slf
            .borrow()
            .v
            .lerp(Vector3::new(o[0], o[1], o[2]), t)
            .map_err(math_fehler)?;
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    #[pyo3(signature = (other, t, /))]
    fn smoothstep(
        slf: &Bound<'_, Self>,
        other: &Bound<'_, PyAny>,
        t: f64,
    ) -> PyResult<Py<PyAny>> {
        let o = koordinaten_alt(other, 3)?;
        let v = slf
            .borrow()
            .v
            .smoothstep(Vector3::new(o[0], o[1], o[2]), t);
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    /// Hat lange gefehlt, ohne dass es auffiel: die Abdeckungstabelle
    /// fragt pygame nach **Namen**, und `slerp` gab es bei `Vector2`
    /// schon - der Name war also abgehakt. Gefunden hat es erst der
    /// Modulvergleich, der jede Klasse einzeln durchgeht.
    #[pyo3(signature = (other, t, /))]
    fn slerp(
        slf: &Bound<'_, Self>,
        other: &Bound<'_, PyAny>,
        t: f64,
    ) -> PyResult<Py<PyAny>> {
        let o = koordinaten_alt(other, 3)
            .map_err(|_| PyTypeError::new_err("Argument 1 must be a vector."))?;
        let s = slf.borrow();
        let v = s
            .v
            .slerp_mit_eps(Vector3::new(o[0], o[1], o[2]), t, s.eps)
            .map_err(math_fehler)?;
        drop(s);
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    /// Siehe [`PyVector2::move_towards`] - auch hier TypeError.
    #[pyo3(signature = (other, max_distance, /))]
    fn move_towards(
        slf: &Bound<'_, Self>,
        other: &Bound<'_, PyAny>,
        max_distance: f64,
    ) -> PyResult<Py<PyAny>> {
        let o = koordinaten(other, 3).ok_or_else(unpassend)?;
        let v = slf
            .borrow()
            .v
            .move_towards(Vector3::new(o[0], o[1], o[2]), max_distance);
        neu_wie(slf, &[v.x, v.y, v.z])
    }
    #[pyo3(signature = (other, max_distance, /))]
    fn move_towards_ip(
        slf: &Bound<'_, Self>,
        other: &Bound<'_, PyAny>,
        max_distance: f64,
    ) -> PyResult<()> {
        let o = koordinaten(other, 3).ok_or_else(unpassend)?;
        let alt = slf.borrow().v;
        slf.borrow_mut().v = alt.move_towards(Vector3::new(o[0], o[1], o[2]), max_distance);
        Ok(())
    }

    fn __len__(&self) -> usize {
        3
    }
    fn __getitem__(&self, py: Python<'_>, i: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        feld_lesen(py, &self.coords(), i)
    }
    fn __setitem__(
        slf: &Bound<'_, Self>,
        i: &Bound<'_, PyAny>,
        wert: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let mut c = slf.borrow().coords();
        feld_setzen(&mut c, i, wert)?;
        slf.borrow_mut().v = Vector3::new(c[0], c[1], c[2]);
        Ok(())
    }
    fn __delitem__(&self, i: &Bound<'_, PyAny>) -> PyResult<()> {
        feld_loeschen(i, 3)
    }
    fn __contains__(&self, v: &Bound<'_, PyAny>) -> PyResult<bool> {
        let z: f64 = v.extract()?;
        Ok(self.coords().contains(&z))
    }
    /// `__safe_for_unpickling__()` - siehe `PyVector2`.
    fn __safe_for_unpickling__(&self) -> bool {
        true
    }

    /// `iter(v)` - siehe [`PyVectorIterator`].
    fn __iter__(slf: &Bound<'_, Self>) -> PyResult<Py<PyVectorIterator>> {
        Py::new(
            slf.py(),
            PyVectorIterator::neu(slf.clone().into_any().unbind(), 3),
        )
    }

    fn __getattr__(&self, py: Python<'_>, name: &str) -> PyResult<Py<PyAny>> {
        sammelname_lesen(py, &self.coords(), name)
    }
    fn __setattr__(
        slf: &Bound<'_, Self>,
        name: &str,
        wert: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let Some(stellen) = sammelname_stellen(name, 3) else {
            return gewoehnlich_setzen(slf.as_any(), name, Some(wert));
        };
        let mut c = slf.borrow().coords();
        sammelname_setzen(&mut c, &stellen, wert)?;
        slf.borrow_mut().v = Vector3::new(c[0], c[1], c[2]);
        Ok(())
    }
    fn __delattr__(slf: &Bound<'_, Self>, name: &str) -> PyResult<()> {
        achse_loeschen(slf.as_any(), name, 3)
    }

    fn __add__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        rechne3(slf, other, false, |a, b| a + b)
    }
    fn __radd__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        rechne3(slf, other, false, |a, b| a + b)
    }
    fn __sub__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        rechne3(slf, other, false, |a, b| a - b)
    }
    fn __rsub__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        rechne3(slf, other, true, |a, b| a - b)
    }
    fn __mul__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let c = slf.borrow().coords();
        if let Some(o) = koordinaten(other, 3) {
            let p = c[0] * o[0] + c[1] * o[1] + c[2] * o[2];
            return Ok(p.into_pyobject(py)?.into_any().unbind());
        }
        let Ok(f) = other.extract::<f64>() else {
            return Ok(py.NotImplemented());
        };
        neu_wie(slf, &[c[0] * f, c[1] * f, c[2] * f])
    }
    fn __rmul__(slf: &Bound<'_, Self>, other: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        PyVector3::__mul__(slf, other)
    }
    fn __truediv__(slf: &Bound<'_, Self>, f: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        teile3(slf, f, false)
    }
    fn __floordiv__(slf: &Bound<'_, Self>, f: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        teile3(slf, f, true)
    }

    fn __neg__(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        let c = slf.borrow().coords();
        neu_wie(slf, &[-c[0], -c[1], -c[2]])
    }
    fn __pos__(slf: &Bound<'_, Self>) -> PyResult<Py<PyAny>> {
        neu_wie(slf, &slf.borrow().coords())
    }
    fn __bool__(&self) -> bool {
        self.v.x != 0.0 || self.v.y != 0.0 || self.v.z != 0.0
    }
    #[pyo3(signature = (stellen=None))]
    fn __round__(
        slf: &Bound<'_, Self>,
        stellen: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        let c = runden(&slf.borrow().coords(), stellen)?;
        neu_wie(slf, &c)
    }
    fn __richcmp__(
        &self,
        other: &Bound<'_, PyAny>,
        op: CompareOp,
    ) -> PyResult<Py<PyAny>> {
        vergleiche(&self.coords(), self.eps, other, op)
    }
    fn __repr__(&self) -> String {
        format!(
            "Vector3({}, {}, {})",
            g_zahl(self.v.x),
            g_zahl(self.v.y),
            g_zahl(self.v.z)
        )
    }
    fn __str__(&self) -> String {
        format!(
            "[{}, {}, {}]",
            g_zahl(self.v.x),
            g_zahl(self.v.y),
            g_zahl(self.v.z)
        )
    }
}

/// `+` und `-` fuer `Vector3` - siehe [`rechne2`].
fn rechne3(
    slf: &Bound<'_, PyVector3>,
    other: &Bound<'_, PyAny>,
    gedreht: bool,
    f: impl Fn(f64, f64) -> f64,
) -> PyResult<Py<PyAny>> {
    let Some(o) = koordinaten(other, 3) else {
        return Ok(slf.py().NotImplemented());
    };
    let c = slf.borrow().coords();
    let (a, b): (&[f64], &[f64]) = if gedreht { (&o, &c) } else { (&c, &o) };
    neu_wie(slf, &[f(a[0], b[0]), f(a[1], b[1]), f(a[2], b[2])])
}

/// `/` und `//` fuer `Vector3` - siehe [`teile2`].
fn teile3(
    slf: &Bound<'_, PyVector3>,
    f: &Bound<'_, PyAny>,
    ganz: bool,
) -> PyResult<Py<PyAny>> {
    if koordinaten(f, 3).is_some() {
        return Ok(slf.py().NotImplemented());
    }
    let Ok(t) = f.extract::<f64>() else {
        return Ok(slf.py().NotImplemented());
    };
    if t == 0.0 {
        return Err(PyZeroDivisionError::new_err("division by zero"));
    }
    let k = 1.0 / t;
    let c = slf.borrow().coords();
    let w: Vec<f64> = c
        .iter()
        .map(|x| if ganz { (x * k).floor() } else { x * k })
        .collect();
    neu_wie(slf, &w)
}

// ---------------------------------------------------------------------------
//  Die freien Funktionen
// ---------------------------------------------------------------------------

/// `pygame.math.clamp(wert, min, max)`.
///
/// Gemessen: sind alle drei ganze Zahlen, kommt auch eine ganze Zahl
/// zurueck (`clamp(5, 1, 3)` ist `3`, nicht `3.0`). Der Kern rechnet in
/// `f64`, also wird hier zurueckverwandelt.
/// `pygame.math.clamp` - **zaehlt die Argumente selbst.**
///
/// pygames C-Stueck schreibt einen eigenen Satz hin
/// (`clamp requires 3 arguments`), und mit benannten Parametern ist der nicht zu
/// erreichen: PyO3 schreibt dann `clamp() takes 3 positional
/// arguments but 17 were given`. Also `*args` und selbst
/// gezaehlt - so wie pygame es tut. Nebenbei hat die
/// Funktion damit **keine** Parameternamen mehr, genau wie
/// pygames.
#[pyfunction]
#[pyo3(name = "clamp")]
#[pyo3(signature = (*args, **kwargs))]
pub fn math_clamp_aussen(
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<Py<PyAny>> {
    crate::umwandlung::keine_woerter(kwargs, "rustgame_py.math.clamp")?;
    if args.len() != 3 {
        return Err(PyTypeError::new_err("clamp requires 3 arguments"));
    }
    let a0 = args.get_item(0)?;
    let a1 = args.get_item(1)?;
    let a2 = args.get_item(2)?;
    math_clamp(&a0, &a1, &a2)
}

fn math_clamp<'py>(
    value: &Bound<'py, PyAny>,
    min: &Bound<'py, PyAny>,
    max: &Bound<'py, PyAny>,
) -> PyResult<Py<PyAny>> {
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let wert = value;
    if !ist_numerisch(wert) || !ist_numerisch(min) || !ist_numerisch(max) {
        return Err(PyTypeError::new_err("clamp requires 3 numeric arguments"));
    }
    // ```c
    // // Using RichCompare instead of converting to C types for performance
    // // reasons.
    // int result = PyObject_RichCompareBool(value, min, Py_LT);
    // if (result == 1) { Py_INCREF(min); return min; }
    // ```
    //
    // Zurueck geht das **Argument**, das gewonnen hat, nicht dessen
    // Zahlenwert - `type(clamp(5, 1, 3))` ist `int`, nicht `float`.
    // Verglichen wird ausserdem **ohne** Umweg ueber `double`, und das
    // ist mehr als eine Frage der Geschwindigkeit: `clamp(12**300, 1, 2)`
    // gibt in pygame `2`, waehrend `PyFloat_AsDouble(12**300)` einen
    // `OverflowError` gaebe. Hier stand genau dieser Umweg.
    if wert.lt(min)? {
        return Ok(min.clone().unbind());
    }
    if wert.gt(max)? {
        return Ok(max.clone().unbind());
    }
    Ok(wert.clone().unbind())
}

/// Eine Zahl fuer eine der freien Funktionen - `RAISE_ARG_TYPE_ERROR`.
///
/// ```c
/// #define RAISE_ARG_TYPE_ERROR(var)                                     \
///     if (PyErr_Occurred()) {                                           \
///         return RAISE(PyExc_TypeError,                                 \
///                      "The argument '" var "' must be a real number"); \
///     }
/// ```
///
/// **Jeder** Fehler von `PyFloat_AsDouble` wird dort zum TypeError, auch
/// ein `OverflowError`. `math_test.py` prueft genau das:
/// `pygame.math.invlerp(12**300, 11**30, 1)` erwartet einen TypeError,
/// und `12**300` passt in kein `double`.
fn zahl_arg(o: &Bound<'_, PyAny>, name: &str) -> PyResult<f64> {
    o.extract().map_err(|_| {
        PyTypeError::new_err(format!("The argument '{name}' must be a real number"))
    })
}

/// `pygame.math.lerp(a, b, t[, do_clamp])`.
///
/// Das vierte Argument gibt es wirklich, und es ist **an**, wenn es
/// fehlt:
///
/// ```text
/// pygame.math.lerp(0, 10, 1.5)         ->  10.0
/// pygame.math.lerp(0, 10, 1.5, False)  ->  15.0
/// ```
///
/// Gemessen. Das Begrenzen steht hier und nicht im Kern - siehe
/// [`rustgame_math::lerp`]; `remap` benutzt dieselbe Rechnung **ohne**
/// Begrenzung, und solange sie im Kern sass, war `remap` falsch.
/// `pygame.math.lerp` - **zaehlt die Argumente selbst.**
///
/// pygames C-Stueck schreibt einen eigenen Satz hin
/// (`lerp requires 3 or 4 arguments`), und mit benannten Parametern ist der nicht zu
/// erreichen: PyO3 schreibt dann `lerp() takes 3 or 4 positional
/// arguments but 17 were given`. Also `*args` und selbst
/// gezaehlt - so wie pygame es tut. Nebenbei hat die
/// Funktion damit **keine** Parameternamen mehr, genau wie
/// pygames.
#[pyfunction]
#[pyo3(name = "lerp")]
#[pyo3(signature = (*args, **kwargs))]
pub fn math_lerp_aussen(
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<f64> {
    crate::umwandlung::keine_woerter(kwargs, "rustgame_py.math.lerp")?;
    if args.len() != 3 && args.len() != 4 {
        return Err(PyTypeError::new_err("lerp requires 3 or 4 arguments"));
    }
    let a0 = args.get_item(0)?;
    let a1 = args.get_item(1)?;
    let a2 = args.get_item(2)?;
    let a3 = if args.len() > 3 { Some(args.get_item(3)?) } else { None };
    math_lerp(&a0, &a1, &a2, a3.as_ref())
}

fn math_lerp(
    a: &Bound<'_, PyAny>,
    b: &Bound<'_, PyAny>,
    t: &Bound<'_, PyAny>,
    do_clamp: Option<&Bound<'_, PyAny>>,
) -> PyResult<f64> {
    let (a, b) = (zahl_arg(a, "min")?, zahl_arg(b, "max")?);
    let t = zahl_arg(t, "value")?;
    let begrenzen = match do_clamp {
        None => true,
        Some(o) => o.is_truthy()?,
    };
    let t = if begrenzen { math::clamp(t, 0.0, 1.0) } else { t };
    Ok(math::lerp(a, b, t))
}

/// `pygame.math.invlerp(a, b, wert)`.
///
/// `b - a == 0` ist ein `ValueError`, kein stilles `0.0` - der Wortlaut
/// ist woertlich uebernommen.
/// `pygame.math.invlerp` - **zaehlt die Argumente selbst.**
///
/// pygames C-Stueck schreibt einen eigenen Satz hin
/// (`invlerp requires exactly 3 numeric arguments`), und mit benannten Parametern ist der nicht zu
/// erreichen: PyO3 schreibt dann `invlerp() takes 3 positional
/// arguments but 17 were given`. Also `*args` und selbst
/// gezaehlt - so wie pygame es tut. Nebenbei hat die
/// Funktion damit **keine** Parameternamen mehr, genau wie
/// pygames.
#[pyfunction]
#[pyo3(name = "invlerp")]
#[pyo3(signature = (*args, **kwargs))]
pub fn math_invlerp_aussen(
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<f64> {
    crate::umwandlung::keine_woerter(kwargs, "rustgame_py.math.invlerp")?;
    if args.len() != 3 {
        return Err(PyTypeError::new_err("invlerp requires exactly 3 numeric arguments"));
    }
    let a0 = args.get_item(0)?;
    let a1 = args.get_item(1)?;
    let a2 = args.get_item(2)?;
    math_invlerp(&a0, &a1, &a2)
}

fn math_invlerp(
    a: &Bound<'_, PyAny>,
    b: &Bound<'_, PyAny>,
    wert: &Bound<'_, PyAny>,
) -> PyResult<f64> {
    let (a, b) = (zahl_arg(a, "a")?, zahl_arg(b, "b")?);
    let wert = zahl_arg(wert, "value")?;
    if b - a == 0.0 {
        return Err(PyValueError::new_err(
            "the result of b - a needs to be different from zero",
        ));
    }
    Ok(math::invlerp(a, b, wert))
}

/// `pygame.math.remap(a_von, a_bis, b_von, b_bis, wert)`.
///
/// **Begrenzt nicht** - `remap(0, 10, 0, 100, -1)` ist `-10.0`.
/// `pygame.math.remap` - **zaehlt die Argumente selbst.**
///
/// pygames C-Stueck schreibt einen eigenen Satz hin
/// (`remap requires exactly 5 numeric arguments`), und mit benannten Parametern ist der nicht zu
/// erreichen: PyO3 schreibt dann `remap() takes 5 positional
/// arguments but 17 were given`. Also `*args` und selbst
/// gezaehlt - so wie pygame es tut. Nebenbei hat die
/// Funktion damit **keine** Parameternamen mehr, genau wie
/// pygames.
#[pyfunction]
#[pyo3(name = "remap")]
#[pyo3(signature = (*args, **kwargs))]
pub fn math_remap_aussen(
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<f64> {
    crate::umwandlung::keine_woerter(kwargs, "rustgame_py.math.remap")?;
    if args.len() != 5 {
        return Err(PyTypeError::new_err("remap requires exactly 5 numeric arguments"));
    }
    let a0 = args.get_item(0)?;
    let a1 = args.get_item(1)?;
    let a2 = args.get_item(2)?;
    let a3 = args.get_item(3)?;
    let a4 = args.get_item(4)?;
    math_remap(&a0, &a1, &a2, &a3, &a4)
}

fn math_remap(
    a_von: &Bound<'_, PyAny>,
    a_bis: &Bound<'_, PyAny>,
    b_von: &Bound<'_, PyAny>,
    b_bis: &Bound<'_, PyAny>,
    wert: &Bound<'_, PyAny>,
) -> PyResult<f64> {
    // Die Reihenfolge der Pruefungen ist pygames: **`value` zuerst**,
    // dann die vier Grenzen. Bei zwei falschen Angaben entscheidet sie,
    // welcher Name in der Meldung steht.
    let wert = zahl_arg(wert, "value")?;
    let (a_von, a_bis) = (zahl_arg(a_von, "i_min")?, zahl_arg(a_bis, "i_max")?);
    let (b_von, b_bis) = (zahl_arg(b_von, "o_min")?, zahl_arg(b_bis, "o_max")?);
    if a_bis - a_von == 0.0 {
        return Err(PyValueError::new_err(
            "the result of i_max - i_min needs to be different from zero",
        ));
    }
    Ok(math::remap(a_von, a_bis, b_von, b_bis, wert))
}

/// `pygame.math.smoothstep(a, b, t)`.
///
/// Die beiden Pruefungen stehen in `math_smoothstep` in **dieser**
/// Reihenfolge: erst `t`, dann `a` und `b`. Bei zwei falschen Angaben
/// entscheidet das, welche der zwei Meldungen herauskommt.
/// `pygame.math.smoothstep` - **zaehlt die Argumente selbst.**
///
/// pygames C-Stueck schreibt einen eigenen Satz hin
/// (`smoothstep requires 3 arguments`), und mit benannten Parametern ist der nicht zu
/// erreichen: PyO3 schreibt dann `smoothstep() takes 3 positional
/// arguments but 17 were given`. Also `*args` und selbst
/// gezaehlt - so wie pygame es tut. Nebenbei hat die
/// Funktion damit **keine** Parameternamen mehr, genau wie
/// pygames.
#[pyfunction]
#[pyo3(name = "smoothstep")]
#[pyo3(signature = (*args, **kwargs))]
pub fn math_smoothstep_aussen(
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<f64> {
    crate::umwandlung::keine_woerter(kwargs, "rustgame_py.math.smoothstep")?;
    if args.len() != 3 {
        return Err(PyTypeError::new_err("smoothstep requires 3 arguments"));
    }
    let a0 = args.get_item(0)?;
    let a1 = args.get_item(1)?;
    let a2 = args.get_item(2)?;
    math_smoothstep(&a0, &a1, &a2)
}

fn math_smoothstep(
    a: &Bound<'_, PyAny>,
    b: &Bound<'_, PyAny>,
    t: &Bound<'_, PyAny>,
) -> PyResult<f64> {
    if !ist_numerisch(t) {
        return Err(PyTypeError::new_err(
            "smoothstep requires the interpolation amount to be number",
        ));
    }
    let t: f64 = t.extract()?;
    if !ist_numerisch(a) || !ist_numerisch(b) {
        return Err(PyTypeError::new_err(
            "smoothstep requires all the arguments to be numbers. To smoothstep \
             between two vectors, please use the Vector class methods.",
        ));
    }
    Ok(math::smoothstep(a.extract()?, b.extract()?, t))
}

// ---------------------------------------------------------------------------
//  Elementweises Rechnen
// ---------------------------------------------------------------------------

/// Die Werte eines Gegenstuecks - Proxy, Vektor, Tupel oder Zahl.
///
/// Eine einzelne Zahl wird auf alle Achsen verteilt: `e * 2` heisst
/// "jede Achse mal zwei". Gemessen, und der Grund, warum hier nicht
/// einfach [`koordinaten`] steht.
fn ew_werte(o: &Bound<'_, PyAny>, dim: usize) -> PyResult<Vec<f64>> {
    if let Ok(p) = o.extract::<PyRef<PyElementwise>>() {
        if p.dim == dim {
            return p.coords(o.py());
        }
    }
    if let Some(c) = koordinaten(o, dim) {
        return Ok(c);
    }
    if let Ok(z) = o.extract::<f64>() {
        return Ok(vec![z; dim]);
    }
    Err(PyTypeError::new_err(
        "elementwise arithmetic expects a vector of the same length or a number",
    ))
}

/// Pythons `%` fuer Gleitkomma - **nicht** Cs `fmod`.
///
/// Uebertragen aus `float_rem` in CPythons `floatobject.c`. Der
/// Unterschied ist das Vorzeichen: `fmod(-7, 3)` ist `-1`, Python sagt
/// `2`. Gemessen an pygame, das hier Pythons Regel folgt:
///
/// ```text
/// Vector2(-7,-7).elementwise() % 3  ->  [2, 2]
/// ```
fn py_rest(a: f64, b: f64) -> f64 {
    let mut rest = a % b;
    if rest != 0.0 {
        if (b < 0.0) != (rest < 0.0) {
            rest += b;
        }
    } else {
        rest = 0.0_f64.copysign(b);
    }
    rest
}

/// Pythons `//` fuer Gleitkomma - aus `float_floor_div`.
///
/// Die Nachkorrektur `if div - floordiv > 0.5` sieht ueberfluessig aus
/// und ist es nicht: sie faengt die Faelle ab, in denen `(a - rest) / b`
/// knapp unter der ganzen Zahl landet.
fn py_ganzteilung(a: f64, b: f64) -> f64 {
    let mut rest = a % b;
    let mut div = (a - rest) / b;
    if rest != 0.0 {
        if (b < 0.0) != (rest < 0.0) {
            div -= 1.0;
        }
    } else {
        rest = 0.0_f64.copysign(b);
        let _ = rest;
    }
    if div != 0.0 {
        let ganz = div.floor();
        if div - ganz > 0.5 {
            ganz + 1.0
        } else {
            ganz
        }
    } else {
        0.0_f64.copysign(a / b)
    }
}

/// `pygame.math.VectorIterator` - was `iter(v)` zurueckgibt.
///
/// Hier stand vorher ein Tupel-Iterator: `__iter__` baute ein `PyTuple`
/// aus den Zahlen und gab dessen Iterator heraus. Das laeuft, und es war
/// trotzdem in zwei Dingen falsch, die beide gemessen sind:
///
/// * **Der Typ.** `type(iter(pygame.Vector2()))` ist
///   `pygame.math.VectorIterator`, nicht `tuple_iterator`. Der Name steht
///   in `dir(pygame.math)`, also darf ein Programm ihn nennen.
/// * **Der Iterator haelt den Vektor, nicht seine Zahlen.** Gemessen:
///
///   ```python
///   w = pygame.Vector2(1, 1)
///   i = iter(w)
///   w.x = 99
///   list(i)        # [99.0, 1.0]  -  nicht [1.0, 1.0]
///   ```
///
///   Ein Tupel haette die 1 festgehalten. Hier wird darum der Vektor
///   selbst gehalten und bei jedem Schritt gefragt.
///
/// Weiter gemessen: `iter(it) is it`, `__length_hint__` gibt die Zahl der
/// **noch** offenen Schritte (0 am Ende), es gibt **kein** `__len__`, und
/// einpoekeln laesst sich der Iterator nicht
/// (`TypeError: cannot pickle 'VectorIterator' object`). Erzeugen laesst
/// er sich auch nicht - ohne `#[new]` sagt PyO3 von selbst
/// `cannot create 'rustgame_py.math.VectorIterator' instances`, und das
/// ist pygames Wortlaut.
#[pyclass(name = "VectorIterator", module = "rustgame_py.math")]
pub struct PyVectorIterator {
    /// Der Vektor - **er**, nicht seine Zahlen.
    vektor: Py<PyAny>,
    /// Wie viele Zahlen er hat: 2 oder 3.
    laenge: usize,
    /// Wie weit gezaehlt ist.
    stelle: usize,
}

impl PyVectorIterator {
    /// Ein Iterator ueber einen Vektor der Laenge `laenge`.
    fn neu(vektor: Py<PyAny>, laenge: usize) -> Self {
        Self {
            vektor,
            laenge,
            stelle: 0,
        }
    }
}

#[pymethods]
impl PyVectorIterator {
    /// `iter(it) is it` - gemessen wahr.
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Die naechste Zahl - **jetzt** vom Vektor geholt.
    fn __next__(&mut self, py: Python<'_>) -> PyResult<Option<f64>> {
        if self.stelle >= self.laenge {
            return Ok(None);
        }
        let wert = self.vektor.bind(py).get_item(self.stelle)?.extract::<f64>()?;
        self.stelle += 1;
        Ok(Some(wert))
    }

    /// Wie viele Schritte noch offen sind - am Ende `0`.
    fn __length_hint__(&self) -> usize {
        self.laenge.saturating_sub(self.stelle)
    }
}

/// `pygame.math.enable_swizzling()` - **leer, und sie warnt**.
///
/// Gemessener Wortlaut, Wort fuer Wort:
///
/// ```text
/// pygame.math.enable_swizzling() is deprecated, and its functionality is
/// removed. This function will be removed in a later version.
/// ```
///
/// Das Tauschen von Feldern (`v.yx`) laesst sich seit Jahren nicht mehr
/// abschalten - es ist immer an. Genau das sagt die Warnung, und darum ist
/// die Funktion hier nicht weggelassen: ein Programm, das sie ruft, soll
/// den Satz lesen und nicht an einem `AttributeError` stehenbleiben.
#[pyfunction]
#[pyo3(name = "enable_swizzling")]
pub fn math_enable_swizzling(py: Python<'_>) -> PyResult<()> {
    swizzling_warnen(py, c"pygame.math.enable_swizzling() is deprecated, and \
its functionality is removed. This function will be removed in a later version.")
}

/// `pygame.math.disable_swizzling()` - dasselbe mit dem anderen Namen.
#[pyfunction]
#[pyo3(name = "disable_swizzling")]
pub fn math_disable_swizzling(py: Python<'_>) -> PyResult<()> {
    swizzling_warnen(py, c"pygame.math.disable_swizzling() is deprecated, and \
its functionality is removed. This function will be removed in a later version.")
}

/// Der gemeinsame Satz der zwei Schalter.
fn swizzling_warnen(py: Python<'_>, text: &std::ffi::CStr) -> PyResult<()> {
    PyErr::warn(
        py,
        &py.get_type::<pyo3::exceptions::PyDeprecationWarning>().into_any(),
        text,
        1,
    )
}

/// `pygame.math.VectorElementwiseProxy`.
///
/// Was `v.elementwise()` zurueckgibt. Jedes Rechenzeichen wirkt dann
/// **je Achse**, und jeder Vergleich fragt **alle** Achsen:
///
/// ```text
/// Vector2(1,2).elementwise() < Vector2(3,4)   ->  True
/// Vector2(1,2).elementwise() < Vector2(3,1)   ->  False
/// ```
///
/// Der Vergleich gibt also ein `bool`, kein Vektor - anders, als man es
/// von numpy kennt. Gemessen.
///
/// Der Proxy haelt den **Vektor selbst** fest, nicht dessen Zahlen
/// (`vector_elementwiseproxy` hat ein Feld `pgVector *vec`). Das ist
/// nicht gleichgueltig: jedes Ergebnis entsteht mit
/// `_vector_subtype_new(self->vec)`, behaelt also den abgeleiteten Typ -
/// `type(MeinVektor(1,2).elementwise() ** 3)` ist `MeinVektor`, und
/// genau das prueft `math_test.py`.
///
/// Ableitbar ist der Proxy nicht (`Py_TPFLAGS_BASETYPE` fehlt);
/// nachgemessen: `class S(type(v.elementwise())): pass` gibt
/// `TypeError: … is not an acceptable base type`.
#[pyclass(name = "VectorElementwiseProxy", module = "rustgame_py.math")]
pub struct PyElementwise {
    besitzer: Py<PyAny>,
    dim: usize,
}

impl PyElementwise {
    /// Die Zahlen des Vektors - **jetzt**, nicht bei `elementwise()`.
    ///
    /// `vector_elementwiseproxy_generic_math` liest `vec->coords` erst
    /// beim Rechnen. Wer die Zahlen beim Anlegen des Proxys abschreibt,
    /// bekommt ein anderes Ergebnis, sobald der Vektor zwischendurch
    /// veraendert wird.
    fn coords(&self, py: Python<'_>) -> PyResult<Vec<f64>> {
        let o = self.besitzer.bind(py);
        if self.dim == 2 {
            let v = o.downcast::<PyVector2>()?.borrow().v;
            Ok(vec![v.x, v.y])
        } else {
            let v = o.downcast::<PyVector3>()?.borrow().v;
            Ok(vec![v.x, v.y, v.z])
        }
    }

    /// Wendet eine Rechnung achsenweise an.
    fn rechne(
        &self,
        py: Python<'_>,
        other: &Bound<'_, PyAny>,
        gedreht: bool,
        f: impl Fn(f64, f64) -> PyResult<f64>,
    ) -> PyResult<Py<PyAny>> {
        let eigen = self.coords(py)?;
        let o = ew_werte(other, self.dim)?;
        let mut aus = Vec::with_capacity(self.dim);
        for (a, b) in eigen.iter().zip(o.iter()) {
            // `gedreht` ist der Fall `2 / e`: dann steht die fremde Zahl
            // links. Ohne diese Unterscheidung waere `2 / e` dasselbe wie
            // `e / 2` - und genau das ist es nicht.
            aus.push(if gedreht { f(*b, *a)? } else { f(*a, *b)? });
        }
        neu_wie(self.besitzer.bind(py), &aus)
    }

    /// Vergleicht achsenweise und verlangt, dass **alle** zutreffen.
    fn vergleiche(&self, other: &Bound<'_, PyAny>, f: impl Fn(f64, f64) -> bool) -> bool {
        let (Ok(eigen), Ok(o)) = (self.coords(other.py()), ew_werte(other, self.dim)) else {
            return false;
        };
        eigen.iter().zip(o.iter()).all(|(a, b)| f(*a, *b))
    }
}

#[pymethods]
impl PyElementwise {
    fn __mul__(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, false, |a, b| Ok(a * b))
    }
    fn __rmul__(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, true, |a, b| Ok(a * b))
    }
    fn __add__(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, false, |a, b| Ok(a + b))
    }
    fn __radd__(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, true, |a, b| Ok(a + b))
    }
    fn __sub__(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, false, |a, b| Ok(a - b))
    }
    fn __rsub__(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, true, |a, b| Ok(a - b))
    }
    fn __truediv__(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, false, teilen)
    }
    fn __rtruediv__(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, true, teilen)
    }
    fn __floordiv__(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, false, ganzteilen)
    }
    fn __rfloordiv__(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, true, ganzteilen)
    }
    fn __mod__(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, false, resten)
    }
    fn __rmod__(&self, py: Python<'_>, o: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, true, resten)
    }
    /// Der dritte Wert (`pow(e, x, m)`) kommt in pygame nicht vor; er
    /// steht trotzdem in der Signatur, weil Python ihn immer mitgibt.
    ///
    /// PyO3 laesst bei `__pow__` **kein** eigenes `signature`-Attribut zu
    /// (`signature cannot be used with magic method __pow__`) - der dritte
    /// Wert muss also als gewoehnliches `Option` dastehen.
    fn __pow__(
        &self,
        py: Python<'_>,
        o: &Bound<'_, PyAny>,
        _m: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, false, hoch)
    }
    fn __rpow__(
        &self,
        py: Python<'_>,
        o: &Bound<'_, PyAny>,
        _m: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        self.rechne(py, o, true, hoch)
    }

    fn __neg__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let w: Vec<f64> = self.coords(py)?.iter().map(|v| -v).collect();
        neu_wie(self.besitzer.bind(py), &w)
    }
    fn __pos__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let w = self.coords(py)?;
        neu_wie(self.besitzer.bind(py), &w)
    }
    fn __abs__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let w: Vec<f64> = self.coords(py)?.iter().map(|v| v.abs()).collect();
        neu_wie(self.besitzer.bind(py), &w)
    }
    fn __bool__(&self, py: Python<'_>) -> PyResult<bool> {
        Ok(self.coords(py)?.iter().any(|v| *v != 0.0))
    }

    /// Alle Vergleiche fragen **alle** Achsen - siehe den Typkopf.
    fn __richcmp__(&self, o: &Bound<'_, PyAny>, op: CompareOp) -> bool {
        match op {
            CompareOp::Eq => self.vergleiche(o, |a, b| a == b),
            CompareOp::Ne => !self.vergleiche(o, |a, b| a == b),
            CompareOp::Lt => self.vergleiche(o, |a, b| a < b),
            CompareOp::Le => self.vergleiche(o, |a, b| a <= b),
            CompareOp::Gt => self.vergleiche(o, |a, b| a > b),
            CompareOp::Ge => self.vergleiche(o, |a, b| a >= b),
        }
    }
}

/// Teilen mit Pythons Fehlermeldung bei Null.
fn teilen(a: f64, b: f64) -> PyResult<f64> {
    if b == 0.0 {
        return Err(PyZeroDivisionError::new_err("division by zero"));
    }
    Ok(a / b)
}

fn ganzteilen(a: f64, b: f64) -> PyResult<f64> {
    if b == 0.0 {
        return Err(PyZeroDivisionError::new_err("division by zero"));
    }
    Ok(py_ganzteilung(a, b))
}

fn resten(a: f64, b: f64) -> PyResult<f64> {
    if b == 0.0 {
        return Err(PyZeroDivisionError::new_err("division by zero"));
    }
    Ok(py_rest(a, b))
}

/// Potenzieren - mit den **zwei** Sonderfaellen, die pygame kennt.
///
/// Beide Wortlaute sind gemessen, nicht erfunden:
///
/// ```text
/// Vector2(-2,4).elementwise() ** 0.5  ->  ValueError:
///     negative number cannot be raised to a fractional power
/// Vector2(0,4).elementwise() ** -1    ->  ZeroDivisionError:
///     zero to a negative power
/// ```
fn hoch(a: f64, b: f64) -> PyResult<f64> {
    if a == 0.0 && b < 0.0 {
        return Err(PyZeroDivisionError::new_err("zero to a negative power"));
    }
    if a < 0.0 && b.fract() != 0.0 {
        return Err(PyValueError::new_err(
            "negative number cannot be raised to a fractional power",
        ));
    }
    Ok(a.powf(b))
}
