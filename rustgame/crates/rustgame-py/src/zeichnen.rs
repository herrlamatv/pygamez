//! zeichnen.rs
//! ===========
//! `pygame.draw`.
//!
//! Die Funktionen heissen im Rust-Modul `zeichne_rect` und in pygame
//! schlicht `rect`.
//!
//! Hier stand: "**beide** Namen stehen hinterher im Untermodul". Das war
//! falsch, und zwar seit immer - `lib.rs` traegt jede Funktion mit
//! `als(d, "rect", ...)` unter **genau einem** Namen ein. Gemessen:
//! `dir(rustgame_py.draw)` nennt elf Namen, alle ohne Praefix, und
//! `rustgame_py.draw.zeichne_rect` gibt es nicht. Der falsche Satz war
//! harmlos, hat aber etwas verdeckt: PyO3 leitete den **Python**-Namen der
//! Funktion aus dem Rust-Namen ab, und deshalb war
//! `draw.line.__name__` `'zeichne_line'` statt `'line'` - sichtbar in
//! jeder Fehlermeldung von PyO3s Argumentleser. Das `#[pyo3(name = ...)]`
//! an jeder Funktion stellt das richtig.
//!
//! Hier stand lange: "Was hier fehlt, weil es im Kern fehlt: `aaline`
//! und `aalines`." Das stimmte einmal. Inzwischen hat der Kern beide,
//! dazu `aacircle` und `flood_fill` - der Vermerk aber blieb stehen, und
//! mit ihm die Luecke in der Huelle. Gefunden hat das erst
//! `tools/api_coverage.py`, seit es die **gebaute** Huelle per `dir()`
//! befragt statt den Rust-Quelltext zu durchsuchen.
//!
//! Die Lehre steht inzwischen an mehreren Stellen dieses Projekts:
//! ein Vermerk ueber einen Baustand veraltet still. Wer auf einem
//! aufbaut, prueft ihn erst nach.
//!
//! ## Warum `pygame.draw` seine Argumente selbst liest
//!
//! Diese Datei benutzt **nicht** `umwandlung::farbe_aus` und
//! **nicht** `umwandlung::punkt_aus`. Das ist keine Doppelung aus
//! Versehen, sondern der Nachbau einer Trennung, die pygame selbst zieht:
//! `src_c/draw.c` ruft `pg_MappedColorFromObj` und `pg_TwoIntsFromObj`,
//! und beide sind **strenger und anders** als das, was der Rest der
//! Huelle braucht. Drei gemessene Unterschiede:
//!
//! * Eine **blanke Zahl** ist hier ein *abgebildeter Pixelwert* der
//!   Zielflaeche, kein RGBA-Wort. `draw.line(s, s.map_rgb("green"), ...)`
//!   zeichnet gruen; `farbe_aus` haette daraus blau gemacht.
//! * Ein **Rechteck ist kein Punkt**. `pg_TwoIntsFromObj(Rect(...))`
//!   scheitert an der Laenge 4; `punkt_aus` nimmt ein `Rect` an und
//!   benutzt seine linke obere Ecke - richtig fuer `blit`, falsch hier.
//! * Eine Folge mit **drei** Zahlen ist kein Punkt. `punkt_aus` liess
//!   `(1, 2, 3)` durch; `draw_test.py` verlangt dafuer einen `TypeError`.
//!
//! Die Fehlerarten sind ebenfalls Teil der Schnittstelle und stehen so in
//! `color.c`: was **folgenartig** ist und trotzdem keine Farbe ergibt,
//! wird ein `ValueError`; was gar keine Folge ist (`2.3`, `None`, ein
//! beliebiges Objekt), ein `TypeError`.

use std::ffi::CStr;

use pyo3::exceptions::{PyDeprecationWarning, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFloat, PyInt, PyString, PyTuple};

use rustgame_core::color::{self, Color};
use rustgame_core::draw::{self, Quadranten};
use rustgame_core::rect::Rect as KernRect;
use rustgame_core::surface::Surface as KernSurface;

use crate::farbe::PyColor;
use crate::flaeche::PySurface;
use crate::rechteck::PyRect;
use crate::umwandlung::FARBE_KAPUTT;

// ---------------------------------------------------------------------------
//  Die Grenzstelle von `pygame.draw`
// ---------------------------------------------------------------------------

/// `PySequence_Check`.
///
/// Gemessen an der gebauten Huelle: fuer `Vector2`, `Rect` und `Color`
/// gibt das echte `PySequence_Check` hier **1** zurueck, fuer Mengen,
/// Woerterbuecher, Zahlen und blanke Objekte **0**. Diese Fassung ueber
/// `__getitem__` trifft dieselbe Einteilung und kommt ohne `unsafe` aus.
///
/// Der Ausschluss von `dict` ist nicht Geschmackssache: CPythons
/// `PySequence_Check` beginnt mit `if (PyDict_Check(s)) return 0;`,
/// obwohl ein `dict` sehr wohl `__getitem__` hat.
fn ist_folge(o: &Bound<'_, PyAny>) -> bool {
    !o.is_instance_of::<PyDict>()
        && o.hasattr(pyo3::intern!(o.py(), "__getitem__"))
            .unwrap_or(false)
}

/// Die Laenge einer Folge - `PySequence_Length`.
///
/// **Nicht** `PySequence_Size` aus der C-Schnittstelle: das liest
/// `sq_length`, und den Platz fuellt PyO3 fuer `Vector2` und `Rect` nicht
/// (gemessen: `PySequence_Size(Vector2(1,2))` scheitert, `len(...)`
/// nicht). `PyObject_Length` findet beide Faecher und trifft damit das,
/// was pygames eigene Typen dort liefern.
fn folgen_laenge(o: &Bound<'_, PyAny>) -> Option<usize> {
    if !ist_folge(o) {
        return None;
    }
    o.len().ok()
}

/// pygames `pg_IntFromObj`.
///
/// Ein Komma wird **abgeschnitten**, nicht gerundet; alles andere geht
/// durch `PyLong_AsLong`, nimmt also `__index__` an und lehnt eine
/// Zeichenkette ab. Der Kuerzungsschritt auf `int` ist in C ein Umbruch,
/// hier entsprechend `as i32`.
fn ganz_aus(o: &Bound<'_, PyAny>) -> Option<i32> {
    if let Ok(f) = o.downcast::<PyFloat>() {
        return Some(f.value() as i32);
    }
    o.extract::<i64>().ok().map(|v| v as i32)
}

/// pygames `pg_FloatFromObj`.
fn gleit_aus(o: &Bound<'_, PyAny>) -> Option<f32> {
    if let Ok(f) = o.downcast::<PyFloat>() {
        return Some(f.value() as f32);
    }
    o.extract::<i64>().ok().map(|v| v as f32)
}

/// pygames `pg_TwoIntsFromObj` - **genau zwei** Zahlen.
///
/// Ein Einertupel wird ausgepackt: `line(s, c, ((1, 2),), ...)` ist
/// dasselbe wie `line(s, c, (1, 2), ...)`. Das steht so im Original und
/// ist der Grund, warum die Pruefung nicht einfach "Laenge gleich zwei"
/// lauten kann.
fn zwei_ganz(o: &Bound<'_, PyAny>) -> Option<(i32, i32)> {
    if let Ok(t) = o.downcast::<PyTuple>() {
        if t.len() == 1 {
            return zwei_ganz(&t.get_item(0).ok()?);
        }
    }
    if folgen_laenge(o)? != 2 {
        return None;
    }
    Some((
        ganz_aus(&o.get_item(0).ok()?)?,
        ganz_aus(&o.get_item(1).ok()?)?,
    ))
}

/// pygames `pg_TwoFloatsFromObj` - `aaline` und `aalines` rechnen in `f32`.
fn zwei_gleit(o: &Bound<'_, PyAny>) -> Option<(f32, f32)> {
    if let Ok(t) = o.downcast::<PyTuple>() {
        if t.len() == 1 {
            return zwei_gleit(&t.get_item(0).ok()?);
        }
    }
    if folgen_laenge(o)? != 2 {
        return None;
    }
    Some((
        gleit_aus(&o.get_item(0).ok()?)?,
        gleit_aus(&o.get_item(1).ok()?)?,
    ))
}

/// Eine Zahl aus einer Farbfolge - `pg_UintFromObj` samt der Schranke 255
/// aus `pg_RGBAFromObj`.
///
/// `PyNumber_Long` schneidet ab, negative Werte fallen durch
/// `PyLong_AsUnsignedLong`.
fn farbkanal(o: &Bound<'_, PyAny>) -> Option<u8> {
    if let Ok(f) = o.downcast::<PyFloat>() {
        let v = f.value().trunc();
        if !(0.0..=255.0).contains(&v) {
            return None;
        }
        return Some(v as u8);
    }
    let v = o.extract::<i64>().ok()?;
    if !(0..=255).contains(&v) {
        return None;
    }
    Some(v as u8)
}

/// Drei oder vier Zahlen - `pg_RGBAFromObj` aus `base.c`.
fn folge_zu_farbe(o: &Bound<'_, PyAny>) -> Option<Color> {
    if let Ok(t) = o.downcast::<PyTuple>() {
        if t.len() == 1 {
            return folge_zu_farbe(&t.get_item(0).ok()?);
        }
    }
    let n = folgen_laenge(o)?;
    if !(3..=4).contains(&n) {
        return None;
    }
    let mut rgba = [0u8, 0, 0, 255];
    for (i, feld) in rgba.iter_mut().enumerate().take(n) {
        *feld = farbkanal(&o.get_item(i).ok()?)?;
    }
    Some(Color::rgba(rgba[0], rgba[1], rgba[2], rgba[3]))
}

/// Der Wortlaut fuer eine ganze Zahl ausserhalb des Bereichs.
const ZAHL_KAPUTT: &str = "invalid color argument (integer out of acceptable range)";

/// pygames `_pg_pylong_to_uint32(val, color, 1)`.
///
/// Das `1` am Ende heisst `handle_negative`: negative Zahlen sind hier
/// **erlaubt** und werden umgedeutet. Der Kommentar daneben nennt den
/// Grund - alte Programme haben abgebildete Pixelwerte mit gesetztem
/// oberstem Bit als negative Zahl weitergereicht.
///
/// Die obere Schranke ist `0xFFFFFFFF`, die untere `-2^31`
/// (`PyLong_AsLongAndOverflow` mit 32-Bit-`long`, wie unter Windows).
fn zahl_zu_pixelwert(o: &Bound<'_, PyAny>) -> PyResult<u32> {
    match o.extract::<i64>() {
        Ok(v) if (-(1i64 << 31)..=0xFFFF_FFFF).contains(&v) => Ok(v as u32),
        _ => Err(PyValueError::new_err(ZAHL_KAPUTT)),
    }
}

/// pygames `pg_MappedColorFromObj(obj, surf, &color, PG_COLOR_HANDLE_ALL)`.
///
/// Der Unterschied zu `umwandlung::farbe_aus` steckt im zweiten Zweig:
/// eine **ganze Zahl ist ein abgebildeter Pixelwert dieser Flaeche**, kein
/// RGBA-Wort. Gemessen an pygame-ce 2.5.7:
///
/// ```text
/// s = pygame.Surface((3, 4))                  # ohne Alphakanal
/// draw.line(s, s.map_rgb("green"), (1,1), (2,1))
/// s.get_at((1,1))  ->  Color(0, 255, 0, 255)
/// ```
///
/// `map_rgb("green")` ist dort `0x0000FF00`; als RGBA gelesen waere das
/// `(0, 0, 255, 0)`, also **blau**. Genau diesen Fehler haben zehn
/// `test_*__valid_color_formats` gemeldet.
///
/// Der Rueckweg laeuft ueber `unmap_rgb_gepackt`, weil der Kern intern mit
/// `Color` zeichnet; fuer jedes Format ist das der Umkehrschritt zu dem,
/// was `Surface.map_rgb` nach Python gibt - auf einer Palettenflaeche also
/// der Tafeleintrag, nicht das Wort.
/// Auch `pygame.mask.Mask.to_surface` braucht genau diese Regeln -
/// deshalb kistenweit sichtbar und nicht zweimal geschrieben.
pub(crate) fn farbe_fuer(s: &KernSurface, o: &Bound<'_, PyAny>) -> PyResult<Color> {
    if let Ok(c) = o.extract::<PyRef<PyColor>>() {
        return Ok(c.wert());
    }
    if o.is_instance_of::<PyInt>() {
        return Ok(s.unmap_rgb_gepackt(zahl_zu_pixelwert(o)?));
    }
    if let Ok(t) = o.downcast::<PyString>() {
        return color::parse(&t.to_cow()?)
            .ok_or_else(|| PyValueError::new_err("invalid color name"));
    }
    if let Some(c) = folge_zu_farbe(o) {
        return Ok(c);
    }
    // Die Aufteilung der Fehlerarten steht woertlich in `color.c`: eine
    // Folge, die keine Farbe hergibt, ist ein `ValueError`; alles andere
    // ein `TypeError`. `draw_test.py` prueft beide Faelle getrennt.
    if ist_folge(o) {
        Err(PyValueError::new_err(FARBE_KAPUTT))
    } else {
        // Das `{:>128}` ist kein Schreibfehler: `color.c` schreibt
        // `PyErr_Format(..., "... of type '%128s' as a color", ...)`, und
        // `%128s` ist in C eine **Mindestbreite**, keine Obergrenze. Der
        // Text ist deshalb rechtsbuendig mit Leerzeichen aufgefuellt.
        // Gemessen an pygame-ce 2.5.7: die Meldung fuer `2.3` ist 176
        // Zeichen lang, davon 123 Leerzeichen vor dem `float`.
        //
        // Das `.to_string()` ist noetig und keine Umstaendlichkeit: Rusts
        // Fuell- und Ausrichtungsangaben wirken nur, wenn die
        // `Display`-Umsetzung `Formatter::pad` benutzt. Die von
        // `Bound<PyString>` schreibt statt dessen unmittelbar - mit ihr
        // blieb `{:>128}` **wirkungslos**, und die Meldung war weiter 53
        // Zeichen lang. Erst gemessen, dann gefunden.
        let name = o.get_type().name()?.to_string();
        Err(PyTypeError::new_err(format!(
            "unable to interpret object of type '{name:>128}' as a color"
        )))
    }
}

/// Ein Rechteck - `pgRect_FromObject` mit pygames Wortlaut fuer den Fehler.
fn rect_fuer(o: &Bound<'_, PyAny>) -> PyResult<KernRect> {
    crate::rechteck::rect_aus_objekt(o)
        .ok_or_else(|| PyTypeError::new_err("rect argument is invalid"))
}

/// Eine Punktfolge fuer `lines`, `polygon` und Freunde.
///
/// Drei getrennte Fehler, und alle drei werden geprueft:
///
/// * keine Folge -> `TypeError: points argument must be a sequence of number pairs`
/// * zu kurz     -> `ValueError` mit dem Wortlaut des Aufrufers
/// * ein Eintrag ist kein Zahlenpaar -> `TypeError: points must be number pairs`
///
/// Wichtig ist ausserdem, dass **alle** Punkte gelesen werden, bevor der
/// erste gezeichnet wird: `test_path_data_validation` prueft, dass nach
/// einem Fehler in der Mitte der Liste nichts auf der Flaeche steht.
fn punkte_ganz(
    o: &Bound<'_, PyAny>,
    mindestens: usize,
    zu_kurz: &'static str,
) -> PyResult<Vec<(i32, i32)>> {
    let n = folgen_laenge(o).ok_or_else(|| {
        PyTypeError::new_err("points argument must be a sequence of number pairs")
    })?;
    if n < mindestens {
        return Err(PyValueError::new_err(zu_kurz));
    }
    let mut aus = Vec::with_capacity(n);
    for i in 0..n {
        let e = o.get_item(i)?;
        aus.push(zwei_ganz(&e).ok_or_else(|| PyTypeError::new_err("points must be number pairs"))?);
    }
    Ok(aus)
}

/// Wie [`punkte_ganz`], aber mit Komma - fuer `aalines`.
fn punkte_gleit(
    o: &Bound<'_, PyAny>,
    mindestens: usize,
    zu_kurz: &'static str,
) -> PyResult<Vec<(f32, f32)>> {
    let n = folgen_laenge(o).ok_or_else(|| {
        PyTypeError::new_err("points argument must be a sequence of number pairs")
    })?;
    if n < mindestens {
        return Err(PyValueError::new_err(zu_kurz));
    }
    let mut aus = Vec::with_capacity(n);
    for i in 0..n {
        let e = o.get_item(i)?;
        aus.push(
            zwei_gleit(&e).ok_or_else(|| PyTypeError::new_err("points must be number pairs"))?,
        );
    }
    Ok(aus)
}

/// Die vier Viertelschalter.
///
/// **`0` heisst nicht "dieses Viertel weglassen", sondern "keine Auswahl
/// treffen".** pygame fuehrt die vier als `int`-Beiwerte mit dem
/// Vorbesetzten `0` und fragt `top_right == 0 && top_left == 0 && ...`;
/// erst wenn davon einer wahr ist, geht es ueberhaupt in den
/// Viertelzweig. `circle(..., draw_top_right=False)` zeichnet deshalb den
/// **ganzen** Kreis - gemessen.
///
/// Hier stand vorher ein `NotImplementedError`, sobald ein Schalter
/// `False` war. Das hat drei Tests zum Scheitern gebracht, die gar keinen
/// Viertelkreis wollten.
fn quadranten(tr: i32, tl: i32, bl: i32, br: i32) -> Quadranten {
    Quadranten {
        top_right: tr != 0,
        top_left: tl != 0,
        bottom_left: bl != 0,
        bottom_right: br != 0,
    }
}

/// Wurde der Beiwert ueberhaupt angegeben?
///
/// `blend` ist in pygame abgekuendigt und ohne Wirkung, die Warnung haengt
/// aber am **Dasein** des Arguments, nicht an seinem Wert: auch
/// `aalines(..., blend=None)` warnt (`blend != NULL` in C).
///
/// Mit `Option<...>` ginge das nicht - PyO3 macht aus einem
/// hereingereichten `None` dasselbe wie aus einem fehlenden Argument.
/// `from_py_with` dagegen laeuft **nur**, wenn etwas uebergeben wurde.
fn wurde_gereicht(_: &Bound<'_, PyAny>) -> PyResult<bool> {
    Ok(true)
}

/// Der Wortlaut der Abkuendigung - woertlich aus `draw.c`.
const BLEND_VERALTET: &CStr = c"blend argument is deprecated and has no functionality and \
will be completely removed in a future version of pygame-ce";

/// Die Warnung ausloesen, wenn `blend` dabei war.
fn blend_warnen(py: Python<'_>, gereicht: bool) -> PyResult<()> {
    if gereicht {
        PyErr::warn(
            py,
            py.get_type::<PyDeprecationWarning>().as_any(),
            BLEND_VERALTET,
            1,
        )?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
//  Die Zeichenfunktionen
// ---------------------------------------------------------------------------

/// `pygame.draw.rect`.
///
/// Die vier Eckradien einzeln sind in pygame `-1`, solange sie nicht
/// gesetzt sind; dann gilt `border_radius` fuer alle.
#[pyfunction]
#[pyo3(name = "rect")]
#[pyo3(signature = (
    surface, color, rect, width=0, border_radius=0,
    border_top_left_radius=-1, border_top_right_radius=-1,
    border_bottom_left_radius=-1, border_bottom_right_radius=-1
))]
#[allow(clippy::too_many_arguments)]
pub fn zeichne_rect(
    surface: &PySurface,
    color: &Bound<'_, PyAny>,
    rect: &Bound<'_, PyAny>,
    width: i32,
    border_radius: i32,
    border_top_left_radius: i32,
    border_top_right_radius: i32,
    border_bottom_left_radius: i32,
    border_bottom_right_radius: i32,
) -> PyResult<PyRect> {
    let r = rect_fuer(rect)?;
    let c = farbe_fuer(&*surface.leihen()?, color)?;
    let ecken = [
        border_top_left_radius,
        border_top_right_radius,
        border_bottom_left_radius,
        border_bottom_right_radius,
    ];
    let mut s = surface.leihen_mut()?;
    if ecken.iter().any(|e| *e >= 0) {
        return Ok(PyRect::neu(draw::rect_corners(
            &mut s,
            c,
            r,
            width,
            border_radius,
            ecken[0],
            ecken[1],
            ecken[2],
            ecken[3],
        )));
    }
    Ok(PyRect::neu(if border_radius > 0 {
        draw::rect_radius(&mut s, c, r, width, border_radius)
    } else {
        draw::rect(&mut s, c, r, width)
    }))
}

/// `pygame.draw.circle`.
///
/// `radius` ist in pygames Signatur ein **Objekt** (`"O!OOO|iiiii"`) und
/// wird erst danach mit `pg_IntFromObj` gelesen - deshalb ist `1.3` hier
/// erlaubt (und wird zu `1`), `"2"` dagegen ein `TypeError`. Ein
/// `radius: i32` in der Signatur haette beides abgelehnt.
#[pyfunction]
#[pyo3(name = "circle")]
#[pyo3(signature = (
    surface, color, center, radius, width=0,
    draw_top_right=0, draw_top_left=0,
    draw_bottom_left=0, draw_bottom_right=0
))]
#[allow(clippy::too_many_arguments)]
pub fn zeichne_circle(
    surface: &PySurface,
    color: &Bound<'_, PyAny>,
    center: &Bound<'_, PyAny>,
    radius: &Bound<'_, PyAny>,
    width: i32,
    draw_top_right: i32,
    draw_top_left: i32,
    draw_bottom_left: i32,
    draw_bottom_right: i32,
) -> PyResult<PyRect> {
    let m = zwei_ganz(center)
        .ok_or_else(|| PyTypeError::new_err("center argument must be a pair of numbers"))?;
    let r = ganz_aus(radius)
        .ok_or_else(|| PyTypeError::new_err("radius argument must be a number"))?;
    let c = farbe_fuer(&*surface.leihen()?, color)?;
    let q = quadranten(
        draw_top_right,
        draw_top_left,
        draw_bottom_left,
        draw_bottom_right,
    );
    Ok(PyRect::neu(draw::circle_quadranten(
        &mut *surface.leihen_mut()?,
        c,
        m,
        r,
        width,
        q,
    )))
}

/// `pygame.draw.ellipse`.
#[pyfunction]
#[pyo3(name = "ellipse")]
#[pyo3(signature = (surface, color, rect, width=0))]
pub fn zeichne_ellipse(
    surface: &PySurface,
    color: &Bound<'_, PyAny>,
    rect: &Bound<'_, PyAny>,
    width: i32,
) -> PyResult<PyRect> {
    let r = rect_fuer(rect)?;
    let c = farbe_fuer(&*surface.leihen()?, color)?;
    Ok(PyRect::neu(draw::ellipse(
        &mut *surface.leihen_mut()?,
        c,
        r,
        width,
    )))
}

/// `pygame.draw.arc`.
#[pyfunction]
#[pyo3(name = "arc")]
#[pyo3(signature = (surface, color, rect, start_angle, stop_angle, width=1))]
pub fn zeichne_arc(
    surface: &PySurface,
    color: &Bound<'_, PyAny>,
    rect: &Bound<'_, PyAny>,
    // `f64`: pygame liest beide Winkel als `double` ("O!OOdd|i") - mit
    // `f32` lagen 1887 von 4116 Boegen daneben, siehe `draw::arc`.
    start_angle: f64,
    stop_angle: f64,
    width: i32,
) -> PyResult<PyRect> {
    let r = rect_fuer(rect)?;
    let c = farbe_fuer(&*surface.leihen()?, color)?;
    Ok(PyRect::neu(draw::arc(
        &mut *surface.leihen_mut()?,
        c,
        r,
        start_angle,
        stop_angle,
        width,
    )))
}

/// `pygame.draw.line`.
#[pyfunction]
#[pyo3(name = "line")]
#[pyo3(signature = (surface, color, start_pos, end_pos, width=1))]
pub fn zeichne_line(
    surface: &PySurface,
    color: &Bound<'_, PyAny>,
    start_pos: &Bound<'_, PyAny>,
    end_pos: &Bound<'_, PyAny>,
    width: i32,
) -> PyResult<PyRect> {
    let c = farbe_fuer(&*surface.leihen()?, color)?;
    let a = zwei_ganz(start_pos)
        .ok_or_else(|| PyTypeError::new_err("invalid start_pos argument"))?;
    let b =
        zwei_ganz(end_pos).ok_or_else(|| PyTypeError::new_err("invalid end_pos argument"))?;
    Ok(PyRect::neu(draw::line(
        &mut *surface.leihen_mut()?,
        c,
        a,
        b,
        width,
    )))
}

/// `pygame.draw.lines`.
///
/// `closed` steht in pygames Signatur als `p` ("predicate") - es nimmt
/// **jedes** Objekt und fragt seine Wahrheit ab. `closed="2"` ist wahr,
/// `closed=()` falsch; `draw_test.py` zaehlt vierzehn solcher Werte auf.
#[pyfunction]
#[pyo3(name = "lines")]
#[pyo3(signature = (surface, color, closed, points, width=1))]
pub fn zeichne_lines(
    surface: &PySurface,
    color: &Bound<'_, PyAny>,
    closed: &Bound<'_, PyAny>,
    points: &Bound<'_, PyAny>,
    width: i32,
) -> PyResult<PyRect> {
    let c = farbe_fuer(&*surface.leihen()?, color)?;
    let p = punkte_ganz(points, 2, "points argument must contain 2 or more points")?;
    Ok(PyRect::neu(draw::lines(
        &mut *surface.leihen_mut()?,
        c,
        closed.is_truthy()?,
        &p,
        width,
    )))
}

/// `pygame.draw.polygon`.
///
/// Mit `width != 0` ruft pygame schlicht `lines(surf, color, 1, points,
/// width)` auf - samt dessen Mindestlaenge **zwei** statt drei. Der Kern
/// macht dasselbe, hier muss nur die Pruefung mitziehen.
#[pyfunction]
#[pyo3(name = "polygon")]
#[pyo3(signature = (surface, color, points, width=0))]
pub fn zeichne_polygon(
    surface: &PySurface,
    color: &Bound<'_, PyAny>,
    points: &Bound<'_, PyAny>,
    width: i32,
) -> PyResult<PyRect> {
    let c = farbe_fuer(&*surface.leihen()?, color)?;
    let p = if width != 0 {
        punkte_ganz(points, 2, "points argument must contain 2 or more points")?
    } else {
        punkte_ganz(points, 3, "points argument must contain more than 2 points")?
    };
    Ok(PyRect::neu(draw::polygon(
        &mut *surface.leihen_mut()?,
        c,
        &p,
        width,
    )))
}

/// `pygame.draw.aaline`.
///
/// Die **einzigen** kantengeglaetteten Zeichenfunktionen sind `aaline`,
/// `aalines` und `aacircle`; alles andere in `pygame.draw` ist binaer
/// rasterisiert (Alpha nur 0 oder 255). Gemessen.
///
/// `blend` ist hier **nur als Schluesselwort** erlaubt (`$O` im
/// Formatwort). Positionell an fuenfter Stelle landet der Wert in `width`
/// - und weil `width <= 1` ohnehin eine einfache Linie zeichnet, kommt
/// dabei fuer `0`, `1`, `True` und `False` dasselbe Bild heraus. Genau das
/// prueft `test_aaline__blend_warning`.
#[pyfunction]
#[pyo3(name = "aaline")]
#[pyo3(signature = (surface, color, start_pos, end_pos, width=1, *, blend=false))]
#[allow(clippy::too_many_arguments)]
pub fn zeichne_aaline(
    py: Python<'_>,
    surface: &PySurface,
    color: &Bound<'_, PyAny>,
    start_pos: &Bound<'_, PyAny>,
    end_pos: &Bound<'_, PyAny>,
    width: i32,
    #[pyo3(from_py_with = wurde_gereicht)] blend: bool,
) -> PyResult<PyRect> {
    blend_warnen(py, blend)?;
    let c = farbe_fuer(&*surface.leihen()?, color)?;
    let a = zwei_gleit(start_pos)
        .ok_or_else(|| PyTypeError::new_err("invalid start_pos argument"))?;
    let b =
        zwei_gleit(end_pos).ok_or_else(|| PyTypeError::new_err("invalid end_pos argument"))?;
    Ok(PyRect::neu(draw::aaline(
        &mut *surface.leihen_mut()?,
        c,
        a,
        b,
        width,
    )))
}

/// `pygame.draw.aalines`.
///
/// Gemessen: **ohne** `width` - anders als `aaline`. Die Signatur ist
/// `aalines(surface, color, closed, points[, blend])`, und `blend` ist
/// hier im Unterschied zu `aaline` auch positionell erlaubt.
#[pyfunction]
#[pyo3(name = "aalines")]
#[pyo3(signature = (surface, color, closed, points, blend=false))]
pub fn zeichne_aalines(
    py: Python<'_>,
    surface: &PySurface,
    color: &Bound<'_, PyAny>,
    closed: &Bound<'_, PyAny>,
    points: &Bound<'_, PyAny>,
    #[pyo3(from_py_with = wurde_gereicht)] blend: bool,
) -> PyResult<PyRect> {
    blend_warnen(py, blend)?;
    let c = farbe_fuer(&*surface.leihen()?, color)?;
    let p = punkte_gleit(points, 2, "points argument must contain 2 or more points")?;
    Ok(PyRect::neu(draw::aalines(
        &mut *surface.leihen_mut()?,
        c,
        closed.is_truthy()?,
        &p,
    )))
}

/// `pygame.draw.aacircle`.
///
/// Wie [`zeichne_circle`], nur mit Xiaolin-Wu-Kanten. Auch hier gilt: die
/// vier Schalter sind `int` mit dem Vorbesetzten `0`, und **kein**
/// gesetzter Schalter ist der Normalfall - der zeichnet bei `width == 0`
/// eine harte Scheibe mit `radius - 1` und mischt einen weichen Ring der
/// Dicke 2 darueber. Wer alle vier auf `True` setzt, bekommt stattdessen
/// einen Ring der Dicke `radius`. Beides gemessen; der Unterschied sind
/// zwei Pixel je Zeile.
#[pyfunction]
#[pyo3(name = "aacircle")]
#[pyo3(signature = (
    surface, color, center, radius, width=0,
    draw_top_right=0, draw_top_left=0,
    draw_bottom_left=0, draw_bottom_right=0
))]
#[allow(clippy::too_many_arguments)]
pub fn zeichne_aacircle(
    surface: &PySurface,
    color: &Bound<'_, PyAny>,
    center: &Bound<'_, PyAny>,
    radius: &Bound<'_, PyAny>,
    width: i32,
    draw_top_right: i32,
    draw_top_left: i32,
    draw_bottom_left: i32,
    draw_bottom_right: i32,
) -> PyResult<PyRect> {
    let m = zwei_ganz(center)
        .ok_or_else(|| PyTypeError::new_err("center argument must be a pair of numbers"))?;
    let r = ganz_aus(radius)
        .ok_or_else(|| PyTypeError::new_err("radius argument must be a number"))?;
    let c = farbe_fuer(&*surface.leihen()?, color)?;
    let q = quadranten(
        draw_top_right,
        draw_top_left,
        draw_bottom_left,
        draw_bottom_right,
    );
    Ok(PyRect::neu(draw::aacircle_quadranten(
        &mut *surface.leihen_mut()?,
        c,
        m,
        r,
        width,
        q,
    )))
}

/// `pygame.draw.flood_fill`.
///
/// `color` darf hier **eine Flaeche** sein; dann wird mit ihr gekachelt
/// statt mit einer Farbe gefuellt. pygame macht daraus vorher mit
/// `PG_ConvertSurface` eine Kopie im Format des Ziels - deshalb steht hier
/// ein `copy()` und nicht eine zweite Ausleihe: sonst waere
/// `flood_fill(s, s, ...)` ein Leihkonflikt statt eines gueltigen Aufrufs.
#[pyfunction]
#[pyo3(name = "flood_fill")]
#[pyo3(signature = (surface, color, start_pos))]
pub fn zeichne_flood_fill(
    surface: &PySurface,
    color: &Bound<'_, PyAny>,
    start_pos: &Bound<'_, PyAny>,
) -> PyResult<PyRect> {
    if let Ok(muster) = color.extract::<PyRef<PySurface>>() {
        let vorlage = muster.leihen()?.copy();
        let p = zwei_ganz(start_pos)
            .ok_or_else(|| PyTypeError::new_err("invalid start_pos argument"))?;
        return Ok(PyRect::neu(draw::flood_fill_muster(
            &mut *surface.leihen_mut()?,
            &vorlage,
            p,
        )));
    }
    let c = farbe_fuer(&*surface.leihen()?, color)?;
    let p = zwei_ganz(start_pos)
        .ok_or_else(|| PyTypeError::new_err("invalid start_pos argument"))?;
    Ok(PyRect::neu(draw::flood_fill(
        &mut *surface.leihen_mut()?,
        c,
        p,
    )))
}
