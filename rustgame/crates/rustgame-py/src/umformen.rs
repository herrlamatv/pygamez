//! umformen.rs
//! ===========
//! `pygame.transform`.
//!
//! Die Groessenangabe nimmt pygame auch als Gleitkomma und **schneidet
//! ab**: `scale(s, (10.9, 10))` ergibt zehn Pixel Breite, nicht elf.
//! Deshalb geht sie hier durch `groesse_aus` und nicht durch ein
//! `(u32, u32)`, das Python vorher runden wuerde.
//!
//! ## `dest_surface` - und warum es lange falsch war
//!
//! Fast jede Funktion hier nimmt ein `dest_surface`. Gemessen tut pygame
//! damit zweierlei: es schreibt das Ergebnis **in diese Flaeche** und
//! gibt **sie selbst** zurueck (`transform.scale(s, g, d) is d` ist
//! wahr). Passt die Groesse nicht, kommt
//! `ValueError: Destination surface not the given width or height.` -
//! und das trifft auch `dest_surface is surface`, solange sich die
//! Groesse aendert.
//!
//! Hier stand der Wert lange als `let _ = dest_surface;` in der
//! Signatur, damit der Aufruf nicht scheitert. Das ist schlimmer als ein
//! Fehler: ein Programm, das `transform.scale(s, g, d)` schreibt und
//! danach `d` benutzt, bekam eine unveraenderte Flaeche und keinerlei
//! Hinweis. `in_ziel` setzt es jetzt wirklich um.
//!
//! Hier stand ausserdem: "Was hier fehlt, weil es im Kern fehlt:
//! `rotozoom`, `laplacian`, `threshold`, `box_blur`, `gaussian_blur`,
//! `average_surfaces`." Auch das stimmte einmal - der Kern hat sie alle.
//!
//! ## Was die Fehlerart angeht
//!
//! Der dritte Vermerk derselben Sorte betraf die Fehlerarten: der Kern
//! meldet jeden Einwand als Wortlaut, und die Huelle machte daraus
//! durchweg einen `ValueError`. pygame nimmt aber an sieben Stellen
//! `TypeError` - bei `threshold` durchgaengig, bei `average_surfaces`
//! auch fuer die leere Liste, und bei jedem ausdruecklichen
//! `dest_surface=None`. Seine eigene Testsuite prueft mit
//! `assertRaises(TypeError, ...)`, und ein `ValueError` besteht das
//! nicht. Welche Meldung wohin gehoert, steht bei der jeweiligen
//! Funktion, jede einzeln gemessen.

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;

use rustgame_core::surface::Surface as KernSurface;
use rustgame_core::transform;

use crate::farbe::PyColor;
use crate::flaeche::{PySurface, Weitergabe};
use crate::umwandlung::rect_aus_opt;

/// Wie streng das Format der Zielflaeche geprueft wird.
///
/// pygame prueft an **drei** Schaerfegraden, und welcher gilt, haengt an
/// der Funktion. Zwei Messreihen machen es eindeutig - die eine mit einer
/// 32-Bit-Quelle ohne Alphakanal, die andere mit dem 24-Bit-Bild
/// `laplacian.png`, das SDL mit **umgekehrten** Masken (`R = 0xff`) laedt:
///
/// ```text
///                          Ziel 32+Alpha  Ziel 24 Bit  Ziel 24 Bit RGB
///                                                      (Quelle 24 BGR)
/// grayscale, invert            geht         Fehler          geht
/// laplacian, scale2x           geht         Fehler          geht
/// box_blur, gaussian_blur      geht         Fehler          geht
/// average_surfaces             geht         Fehler          geht
/// scale, pixelate              geht         Fehler         Fehler
/// hsl, solid_overlay          Fehler        Fehler         Fehler
/// smoothscale                  geht          geht           geht
/// ```
///
/// Die erste Gruppe vergleicht also **nur die Bytes je Pixel** - Masken
/// sind ihr gleichgueltig, Alphakanal auch. Die zweite nimmt die
/// RGB-Masken dazu, die dritte das ganze Format. Und `smoothscale` prueft
/// als einzige gar nichts.
///
/// Hier stand zwischenzeitlich "Tiefe und RGB-Masken" fuer die erste
/// Gruppe. Das war falsch und hat `test_laplacian__24_big_endian`
/// umgebracht - der Test reicht eine BGR-Quelle in ein RGB-Ziel, und
/// pygame laesst das durch. Gemessen ist gemessen.
#[derive(Clone, Copy)]
enum Formatprobe {
    /// Gar keine - nur die Groesse zaehlt (`smoothscale`).
    Keine,
    /// Nur die Bytes je Pixel.
    Bytes(&'static str),
    /// Bytes je Pixel **und** die RGB-Masken; der Alphakanal darf abweichen.
    BytesUndRgb(&'static str),
    /// Das ganze Format muss stimmen, Alphakanal eingeschlossen.
    Vollstaendig(&'static str),
}

/// Die Meldung, die pygame fuer ein untaugliches Zielformat gibt.
const GLEICH: &str = "Source and destination surfaces need the same format.";
/// Dieselbe Lage, andere Worte - `scale` und `pixelate` sagen das hier.
const VERTRAEGLICH: &str = "Source and destination surfaces need to be compatible formats.";

/// Legt das Ergebnis in `dest_surface`, wenn eines gereicht wurde.
///
/// Gibt dann **dieselbe** Flaeche zurueck, die hereinkam - pygames
/// `transform.scale(s, g, d) is d` ist wahr, und ein Nachbau, der eine
/// Kopie zurueckgibt, faellt genau dort auf, wo jemand das ausnutzt.
///
/// Die Groessenpruefung ist gemessen - und der Wortlaut haengt an der
/// **Funktion**, weshalb er hier hereingereicht wird. pygame hat dafuer
/// vier verschiedene Saetze:
///
/// ```text
/// scale, smoothscale, pixelate      Destination surface not the given width or height.
/// scale2x                           Destination surface not 2x bigger.
/// grayscale, invert, solid_overlay, hsl
///                                   Destination surface must be the same size as source surface.
/// laplacian, box_blur, gaussian_blur, average_surfaces
///                                   Destination surface not the same size.
/// ```
///
/// Einen davon fuer alle zu nehmen waere die naheliegende Vereinfachung
/// und in drei Vierteln der Faelle die falsche Meldung.
///
/// ## Ein ausdrueckliches `None` ist ein Fehler
///
/// pygame lehnt `transform.invert(s, None)` mit
/// `TypeError: argument 2 must be pygame.surface.Surface, not None` ab -
/// obwohl seine eigene Signatur `dest_surface=None` sagt. Das gilt fuer
/// **alle** vierzehn geprueften Funktionen; die Nummer im Satz ist die
/// Stelle des Arguments und wechselt mit der Funktion.
///
/// Hier stand lange: "Eine bewusste Abweichung: hier wird `None` genommen
/// und wie ein Weglassen behandelt, damit
/// `transform.invert(s, ziel if flag else None)` das Erwartete tut."
/// Bequemer, aber falsch - `transform_test.test_hsl_args` reicht genau
/// dieses `None` und verlangt den `TypeError`. Und der Unterschied
/// zwischen "weggelassen" und "`None`" laesst sich in PyO3 nicht mit
/// `Option` ausdruecken; dafuer gibt es jetzt
/// [`Weitergabe`](crate::flaeche::Weitergabe).
fn in_ziel(
    py: Python<'_>,
    ergebnis: rustgame_core::surface::Surface,
    vorbild: Option<&PySurface>,
    ziel: Option<&Bound<'_, PyAny>>,
    nummer: usize,
    meldung: &'static str,
    probe: Formatprobe,
) -> PyResult<Py<PyAny>> {
    let Some(z) = ziel else {
        // Die RLE-Erlaubnis geht mit - siehe `PySurface::neu_wie`.
        let neu = match vorbild {
            Some(v) => PySurface::neu_wie(ergebnis, v),
            None => PySurface::neu(ergebnis),
        };
        return Ok(neu.into_pyobject(py)?.into_any().unbind());
    };
    if z.is_none() {
        return Err(PyTypeError::new_err(format!(
            "argument {nummer} must be pygame.surface.Surface, not None"
        )));
    }
    {
        let f: PyRef<PySurface> = z.extract()?;
        let mut d = f.leihen_mut()?;
        // **Format zuerst, Groesse danach** - gemessen:
        //
        // ```python
        // transform.scale(s, (2, 2), Surface((4, 4), 0, 24))
        // pygame    ValueError: Source and destination surfaces need to be compatible formats.
        // ```
        //
        // Beide Einwaende treffen zu (das Ziel hat die falsche Groesse
        // **und** das falsche Format), und pygame nennt das Format. Hier
        // stand die Groesse zuerst und damit der andere Satz. Sichtbar
        // wird so eine Reihenfolge nur, wenn ein Aufruf **beide** Regeln
        // bricht - und genau den baut pygames Testsuite.
        match probe {
            Formatprobe::Keine => {}
            Formatprobe::Bytes(t) => {
                if ergebnis.get_bytesize() != d.get_bytesize() {
                    return Err(PyValueError::new_err(t));
                }
            }
            Formatprobe::BytesUndRgb(t) => {
                let (a, b) = (ergebnis.get_format(), d.get_format());
                let rgb = |m: (u32, u32, u32, u32)| (m.0, m.1, m.2);
                if a.bytesize() != b.bytesize() || rgb(a.masks) != rgb(b.masks) {
                    return Err(PyValueError::new_err(t));
                }
            }
            Formatprobe::Vollstaendig(t) => {
                if ergebnis.get_format() != d.get_format() {
                    return Err(PyValueError::new_err(t));
                }
            }
        }
        if d.get_size() != ergebnis.get_size() {
            return Err(PyValueError::new_err(meldung));
        }
        // **Roh kopieren, nicht blitten.** Ein Blit mit Flag 0 wuerde
        // mischen - bei zwei `SRCALPHA`-Flaechen also das Ergebnis mit
        // dem alten Inhalt des Ziels verrechnen. pygame ersetzt hier.
        let (w, h) = ergebnis.get_size();
        for y in 0..h as i32 {
            for x in 0..w as i32 {
                d.set_at_ungeprueft((x, y), ergebnis.get_at_mapped((x, y)));
            }
        }
    }
    Ok(z.clone().unbind())
}

/// Zwei Flaechen, die sich denselben Pixelspeicher teilen?
///
/// Die Weichzeichner lehnen das ab - gemessen, samt Wortlaut:
/// `box_blur(s, 10, dest_surface=s)` und ebenso zwei **Teilflaechen**
/// derselben Flaeche geben
/// `ValueError: Blur routines do not support dest_surfaces that share
/// pixels with the source surface. …`. Eine gewoehnliche Kopie geht.
fn teilt_speicher(a: &KernSurface, b: &KernSurface) -> bool {
    a.teilt_speicher(b)
}

/// Was die Weichzeichner vor der Arbeit pruefen.
fn blur_probe(quelle: &KernSurface, ziel: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
    if quelle.get_palette().is_some() {
        return Err(PyValueError::new_err("Indexed surfaces cannot be blurred."));
    }
    let Some(z) = ziel else { return Ok(()) };
    if z.is_none() {
        return Ok(());
    }
    let f: PyRef<PySurface> = z.extract()?;
    let d = f.leihen()?;
    if teilt_speicher(quelle, &d) {
        return Err(PyValueError::new_err(
            concat!(
                "Blur routines do not support dest_surfaces that share pixels with ",
                "the source surface. Likely the surfaces are the same, one of them ",
                "is a subsurface, or they are sharing the same buffer."
            ),
        ));
    }
    Ok(())
}

/// Zwei Zahlen, abgeschnitten - und **negativ ist ein Fehler**.
///
/// Hier stand `.max(0.0)`, was aus `(-1, -1)` ein `(0, 0)` machte. pygame
/// gibt dort `ValueError: Cannot scale to negative size` (gemessen, fuer
/// `scale` und `smoothscale`), und `(0, 0)` ist ausdruecklich erlaubt -
/// `transform_test.test_scale__zero_surface_transform` verlangt eine
/// Flaeche der Groesse null zurueck. Die beiden Faelle durften also nicht
/// zusammenfallen.
fn groesse_aus(o: &Bound<'_, PyAny>) -> PyResult<(u32, u32)> {
    let v: Vec<f64> = o
        .extract()
        .map_err(|_| PyTypeError::new_err("size must be two numbers"))?;
    if v.len() < 2 {
        return Err(PyTypeError::new_err("size must be two numbers"));
    }
    if v[0] < 0.0 || v[1] < 0.0 {
        return Err(PyValueError::new_err("Cannot scale to negative size"));
    }
    Ok((v[0] as u32, v[1] as u32))
}

/// Was `smoothscale` vor der Arbeit prueft.
///
/// Gemessen: unter 24 Bit gibt pygame
/// `ValueError: Only 24-bit or 32-bit surfaces can be smoothly scaled` -
/// und zwar auch bei `smoothscale_by`. Ueber 32 Bit kommt man nie hin,
/// weil schon `Surface((20,20), depth=48)` mit "nonstandard bit depth
/// given" abbricht.
fn glatt_probe(s: &KernSurface) -> PyResult<()> {
    if !(24..=32).contains(&s.get_bitsize()) {
        return Err(PyValueError::new_err(
            "Only 24-bit or 32-bit surfaces can be smoothly scaled",
        ));
    }
    Ok(())
}

/// `pygame.transform.scale`.
#[pyfunction]
#[pyo3(signature = (surface, size, dest_surface=Weitergabe::Fehlt))]
#[pyo3(name = "scale")]
pub fn tf_scale(
    py: Python<'_>,
    surface: &PySurface,
    size: &Bound<'_, PyAny>,
    dest_surface: Weitergabe<'_>,
) -> PyResult<Py<PyAny>> {
    let g = groesse_aus(size)?;
    let e = transform::scale(&*surface.leihen()?, g);
    in_ziel(
        py,
        e,
        Some(surface),
        dest_surface.als_option(),
        3,
        "Destination surface not the given width or height.",
        Formatprobe::BytesUndRgb(VERTRAEGLICH),
    )
}

/// `pygame.transform.scale_by`.
///
/// Hier fehlte `dest_surface`. Das war kein Versehen im Kern, sondern in
/// dieser Signatur: `transform_test.test_scale_by` schreibt
/// `scale_by(s, (2.0, 1.5), dest_surface=dest)`, und daran ist der Test
/// mit `unexpected keyword argument` gescheitert, bevor der erste
/// `assert` dran war.
#[pyfunction]
#[pyo3(signature = (surface, factor, dest_surface=Weitergabe::Fehlt))]
#[pyo3(name = "scale_by")]
pub fn tf_scale_by(
    py: Python<'_>,
    surface: &PySurface,
    factor: &Bound<'_, PyAny>,
    dest_surface: Weitergabe<'_>,
) -> PyResult<Py<PyAny>> {
    let (fx, fy) = faktoren_aus(factor)?;
    let e = transform::scale_by(&*surface.leihen()?, fx, fy);
    in_ziel(
        py,
        e,
        Some(surface),
        dest_surface.als_option(),
        3,
        "Destination surface not the given width or height.",
        Formatprobe::BytesUndRgb(VERTRAEGLICH),
    )
}

/// `pygame.transform.smoothscale`.
#[pyfunction]
#[pyo3(signature = (surface, size, dest_surface=Weitergabe::Fehlt))]
#[pyo3(name = "smoothscale")]
pub fn tf_smoothscale(
    py: Python<'_>,
    surface: &PySurface,
    size: &Bound<'_, PyAny>,
    dest_surface: Weitergabe<'_>,
) -> PyResult<Py<PyAny>> {
    let g = groesse_aus(size)?;
    let q = surface.leihen()?;
    glatt_probe(&q)?;
    let e = transform::smoothscale(&q, g);
    drop(q);
    in_ziel(
        py,
        e,
        Some(surface),
        dest_surface.als_option(),
        3,
        "Destination surface not the given width or height.",
        // `smoothscale` ist die **einzige** hier, die das Zielformat nicht
        // prueft: `smoothscale(s32, (8,8), Surface((8,8), depth=24))`
        // laeuft in pygame durch. Gemessen.
        Formatprobe::Keine,
    )
}

/// `pygame.transform.smoothscale_by`.
///
/// Siehe [`tf_scale_by`] - `dest_surface` fehlte hier genauso.
#[pyfunction]
#[pyo3(signature = (surface, factor, dest_surface=Weitergabe::Fehlt))]
#[pyo3(name = "smoothscale_by")]
pub fn tf_smoothscale_by(
    py: Python<'_>,
    surface: &PySurface,
    factor: &Bound<'_, PyAny>,
    dest_surface: Weitergabe<'_>,
) -> PyResult<Py<PyAny>> {
    let (fx, fy) = faktoren_aus(factor)?;
    let q = surface.leihen()?;
    glatt_probe(&q)?;
    let e = transform::smoothscale_by(&q, fx, fy);
    drop(q);
    in_ziel(
        py,
        e,
        Some(surface),
        dest_surface.als_option(),
        3,
        "Destination surface not the given width or height.",
        Formatprobe::Keine,
    )
}

/// Eine Flagge, wie pygame sie liest: als **ganze Zahl**, nicht als
/// beliebiger Wahrheitswert.
///
/// Gemessen an `transform.flip`: `1`, `0` und `True` gehen durch, `2.5`,
/// `"x"`, `None` und `[]` geben
/// `TypeError: '<typ>' object cannot be interpreted as an integer`. Ein
/// `bool`-Parameter in der Huelle waere also zu eng gewesen - und genau
/// daran scheiterten `test_flip` und `test_flip_alpha`, die
/// `flip(image, 1, 0)` schreiben.
fn ganzflagge(o: &Bound<'_, PyAny>) -> PyResult<bool> {
    match o.extract::<i64>() {
        Ok(v) => Ok(v != 0),
        Err(_) => Err(pyo3::exceptions::PyTypeError::new_err(format!(
            "'{}' object cannot be interpreted as an integer",
            o.get_type().name()?
        ))),
    }
}

/// `pygame.transform.flip`.
#[pyfunction]
#[pyo3(name = "flip")]
pub fn tf_flip(
    surface: &PySurface,
    flip_x: &Bound<'_, PyAny>,
    flip_y: &Bound<'_, PyAny>,
) -> PyResult<PySurface> {
    Ok(PySurface::neu_wie(transform::flip(
        &*surface.leihen()?,
        ganzflagge(flip_x)?,
        ganzflagge(flip_y)?,
    ), surface))
}

/// `pygame.transform.rotate` - der Winkel ist gegen den Uhrzeigersinn.
#[pyfunction]
#[pyo3(name = "rotate")]
pub fn tf_rotate(surface: &PySurface, angle: f32) -> PyResult<PySurface> {
    Ok(PySurface::neu_wie(transform::rotate(&*surface.leihen()?, angle), surface))
}

/// `pygame.transform.scale2x`.
#[pyfunction]
#[pyo3(signature = (surface, dest_surface=Weitergabe::Fehlt))]
#[pyo3(name = "scale2x")]
pub fn tf_scale2x(
    py: Python<'_>,
    surface: &PySurface,
    dest_surface: Weitergabe<'_>,
) -> PyResult<Py<PyAny>> {
    let e = transform::scale2x(&*surface.leihen()?);
    in_ziel(
        py,
        e,
        Some(surface),
        dest_surface.als_option(),
        2,
        "Destination surface not 2x bigger.",
        Formatprobe::Bytes(GLEICH),
    )
}

/// `pygame.transform.grayscale`.
#[pyfunction]
#[pyo3(signature = (surface, dest_surface=Weitergabe::Fehlt))]
#[pyo3(name = "grayscale")]
pub fn tf_grayscale(
    py: Python<'_>,
    surface: &PySurface,
    dest_surface: Weitergabe<'_>,
) -> PyResult<Py<PyAny>> {
    let e = transform::grayscale(&*surface.leihen()?);
    in_ziel(
        py,
        e,
        Some(surface),
        dest_surface.als_option(),
        2,
        "Destination surface must be the same size as source surface.",
        Formatprobe::Bytes(GLEICH),
    )
}

/// `pygame.transform.average_color`.
#[pyfunction]
#[pyo3(signature = (surface, rect=None, consider_alpha=false))]
#[pyo3(name = "average_color")]
pub fn tf_average_color(
    surface: &PySurface,
    rect: Option<&Bound<'_, PyAny>>,
    consider_alpha: bool,
) -> PyResult<PyColor> {
    let r = rect_aus_opt(rect)?;
    Ok(PyColor::neu(transform::average_color(
        &*surface.leihen()?,
        r,
        consider_alpha,
    )))
}

/// `f` oder `(fx, fy)` - pygame nimmt beides.
///
/// Hier gab es ausserdem ein drittes Argument `y`, mit dem man
/// `scale_by(s, 2, 3)` schreiben konnte. Das war eine **Erfindung**:
/// pygames dritte Stelle ist `dest_surface`, und `scale_by(s, 2, y=3)`
/// gibt dort `TypeError: this function got an unexpected keyword argument
/// 'y'` (gemessen). Mit dem `y` dazwischen landete
/// `scale_by(s, 2, ziel)` - genau die Schreibweise aus pygames Handbuch -
/// beim falschen Feld.
fn faktoren_aus(o: &Bound<'_, PyAny>) -> PyResult<(f32, f32)> {
    if let Ok(f) = o.extract::<f32>() {
        return Ok((f, f));
    }
    let v: Vec<f32> = o
        .extract()
        .map_err(|_| PyTypeError::new_err("factor must be a number or two numbers"))?;
    if v.len() < 2 {
        return Err(PyTypeError::new_err(
            "factor must be a number or two numbers",
        ));
    }
    Ok((v[0], v[1]))
}

// ---------------------------------------------------------------------------
//  Die dreizehn, die der Kern schon konnte und die Huelle nicht durchreichte
// ---------------------------------------------------------------------------

/// Ein Wortlaut aus dem Kern wird zu `ValueError`.
fn umform_fehler(t: &'static str) -> PyErr {
    pyo3::exceptions::PyValueError::new_err(t)
}

/// `pygame.transform.invert`.
#[pyfunction]
#[pyo3(signature = (surface, dest_surface=Weitergabe::Fehlt))]
#[pyo3(name = "invert")]
pub fn tf_invert(
    py: Python<'_>,
    surface: &PySurface,
    dest_surface: Weitergabe<'_>,
) -> PyResult<Py<PyAny>> {
    let e = transform::invert(&*surface.leihen()?);
    in_ziel(
        py,
        e,
        Some(surface),
        dest_surface.als_option(),
        2,
        "Destination surface must be the same size as source surface.",
        Formatprobe::Bytes(GLEICH),
    )
}

/// `pygame.transform.laplacian` - Kantenhervorhebung.
#[pyfunction]
#[pyo3(signature = (surface, dest_surface=Weitergabe::Fehlt))]
#[pyo3(name = "laplacian")]
pub fn tf_laplacian(
    py: Python<'_>,
    surface: &PySurface,
    dest_surface: Weitergabe<'_>,
) -> PyResult<Py<PyAny>> {
    let e = transform::laplacian(&*surface.leihen()?);
    in_ziel(
        py,
        e,
        Some(surface),
        dest_surface.als_option(),
        2,
        "Destination surface not the same size.",
        Formatprobe::Bytes(GLEICH),
    )
}

/// `pygame.transform.chop` - schneidet Zeilen und Spalten **heraus**.
///
/// Anders als ein Ausschnitt: was im Rechteck liegt, faellt weg, und die
/// Raender ruecken zusammen.
#[pyfunction]
#[pyo3(name = "chop")]
pub fn tf_chop(surface: &PySurface, rect: &Bound<'_, PyAny>) -> PyResult<PySurface> {
    let r = crate::umwandlung::rect_aus(rect)?;
    Ok(PySurface::neu_wie(transform::chop(&*surface.leihen()?, r), surface))
}

/// `pygame.transform.box_blur`.
#[pyfunction]
#[pyo3(signature = (surface, radius, repeat_edge_pixels=true, dest_surface=Weitergabe::Fehlt))]
#[pyo3(name = "box_blur")]
pub fn tf_box_blur(
    py: Python<'_>,
    surface: &PySurface,
    radius: i32,
    repeat_edge_pixels: bool,
    dest_surface: Weitergabe<'_>,
) -> PyResult<Py<PyAny>> {
    let q = surface.leihen()?;
    blur_probe(&q, dest_surface.als_option())?;
    let e = transform::box_blur(&q, radius, repeat_edge_pixels).map_err(umform_fehler)?;
    drop(q);
    in_ziel(
        py,
        e,
        Some(surface),
        dest_surface.als_option(),
        4,
        "Destination surface not the same size.",
        Formatprobe::Bytes(GLEICH),
    )
}

/// `pygame.transform.gaussian_blur`.
///
/// `radius` ist hier das **Sigma** der Glockenkurve, nicht die Anzahl
/// Pixel - siehe den Kern, wo auch steht, warum `radius = 0` eine
/// vollstaendig schwarze, durchsichtige Flaeche ergibt.
#[pyfunction]
#[pyo3(signature = (surface, radius, repeat_edge_pixels=true, dest_surface=Weitergabe::Fehlt))]
#[pyo3(name = "gaussian_blur")]
pub fn tf_gaussian_blur(
    py: Python<'_>,
    surface: &PySurface,
    radius: i32,
    repeat_edge_pixels: bool,
    dest_surface: Weitergabe<'_>,
) -> PyResult<Py<PyAny>> {
    let q = surface.leihen()?;
    blur_probe(&q, dest_surface.als_option())?;
    let e = transform::gaussian_blur(&q, radius, repeat_edge_pixels).map_err(umform_fehler)?;
    drop(q);
    in_ziel(
        py,
        e,
        Some(surface),
        dest_surface.als_option(),
        4,
        "Destination surface not the same size.",
        Formatprobe::Bytes(GLEICH),
    )
}

/// `pygame.transform.pixelate`.
#[pyfunction]
#[pyo3(signature = (surface, pixel_size, dest_surface=Weitergabe::Fehlt))]
#[pyo3(name = "pixelate")]
pub fn tf_pixelate(
    py: Python<'_>,
    surface: &PySurface,
    pixel_size: i32,
    dest_surface: Weitergabe<'_>,
) -> PyResult<Py<PyAny>> {
    let e = transform::pixelate(&*surface.leihen()?, pixel_size).map_err(umform_fehler)?;
    in_ziel(
        py,
        e,
        Some(surface),
        dest_surface.als_option(),
        3,
        "Destination surface not the given width or height.",
        Formatprobe::BytesUndRgb(VERTRAEGLICH),
    )
}

/// `pygame.transform.solid_overlay`.
#[pyfunction]
#[pyo3(signature = (surface, color, dest_surface=Weitergabe::Fehlt, keep_alpha=false))]
#[pyo3(name = "solid_overlay")]
pub fn tf_solid_overlay(
    py: Python<'_>,
    surface: &PySurface,
    color: &Bound<'_, PyAny>,
    dest_surface: Weitergabe<'_>,
    keep_alpha: bool,
) -> PyResult<Py<PyAny>> {
    let c = crate::umwandlung::farbe_aus(color)?;
    let e = transform::solid_overlay(&*surface.leihen()?, c, keep_alpha);
    in_ziel(
        py,
        e,
        Some(surface),
        dest_surface.als_option(),
        3,
        "Destination surface must be the same size as source surface.",
        Formatprobe::Vollstaendig(GLEICH),
    )
}

/// `pygame.transform.hsl` - Farbton, Saettigung, Helligkeit verschieben.
#[pyfunction]
#[pyo3(signature = (surface, hue=0.0, saturation=0.0, lightness=0.0, dest_surface=Weitergabe::Fehlt))]
#[pyo3(name = "hsl")]
pub fn tf_hsl(
    py: Python<'_>,
    surface: &PySurface,
    hue: f32,
    saturation: f32,
    lightness: f32,
    dest_surface: Weitergabe<'_>,
) -> PyResult<Py<PyAny>> {
    let e = transform::hsl(&*surface.leihen()?, hue, saturation, lightness)
        .map_err(umform_fehler)?;
    in_ziel(
        py,
        e,
        Some(surface),
        dest_surface.als_option(),
        5,
        "Destination surface must be the same size as source surface.",
        Formatprobe::Vollstaendig(GLEICH),
    )
}

/// `pygame.transform.rotozoom` - drehen und skalieren in einem Schritt.
#[pyfunction]
#[pyo3(name = "rotozoom")]
pub fn tf_rotozoom(surface: &PySurface, angle: f32, scale: f32) -> PyResult<PySurface> {
    Ok(PySurface::neu_wie(transform::rotozoom(
        &*surface.leihen()?,
        angle,
        scale,
    ), surface))
}

/// `pygame.transform.average_surfaces` - das Mittel mehrerer Flaechen.
#[pyfunction]
#[pyo3(signature = (surfaces, dest_surface=Weitergabe::Fehlt, palette_colors=1))]
#[pyo3(name = "average_surfaces")]
pub fn tf_average_surfaces(
    py: Python<'_>,
    surfaces: &Bound<'_, PyAny>,
    dest_surface: Weitergabe<'_>,
    palette_colors: i32,
) -> PyResult<Py<PyAny>> {
    // `palette_colors` betrifft nur Palettenflaechen; rustgame rechnet
    // ausschliesslich in ARGB8888, also gibt es hier nichts zu waehlen.
    let _ = palette_colors;
    // **Eine Folge, kein Erzeuger.** Gemessen: `average_surfaces(s for s
    // in [a, b])` gibt `TypeError: Argument must be a sequence of surface
    // objects.` - pygame ruft `PySequence_Check`, und ein Generator
    // besteht den nicht. Hier stand `try_iter`, das ihn durchliess.
    if !surfaces.is_instance_of::<pyo3::types::PyList>()
        && !surfaces.is_instance_of::<pyo3::types::PyTuple>()
    {
        return Err(PyTypeError::new_err(
            // pygame nennt das **Glied**, nicht die Folge:
            // `average_surfaces("x")` gibt `Needs to be a surface
            // object.` - gemessen.
            "Needs to be a surface object.",
        ));
    }
    let mut gehalten = Vec::new();
    for f in surfaces.try_iter()? {
        let f = f?;
        let s: PyRef<PySurface> = f
            .extract()
            .map_err(|_| PyTypeError::new_err("Needs to be a surface object."))?;
        let kopie = s.leihen()?.clone();
        gehalten.push(kopie);
    }
    // Die leere Folge gibt einen **TypeError**, nicht den ValueError, den
    // der Wortlaut nahelegt - gemessen.
    if gehalten.is_empty() {
        return Err(PyTypeError::new_err("Needs to be given at least one surface."));
    }
    let verweise: Vec<&rustgame_core::surface::Surface> = gehalten.iter().collect();
    let e = transform::average_surfaces(&verweise).map_err(umform_fehler)?;
    in_ziel(
        py,
        e,
        None,
        dest_surface.als_option(),
        2,
        "Destination surface not the same size.",
        Formatprobe::Bytes(GLEICH),
    )
}

/// `pygame.transform.get_smoothscale_backend`.
#[pyfunction]
#[pyo3(name = "get_smoothscale_backend")]
pub fn tf_get_smoothscale_backend() -> &'static str {
    transform::get_smoothscale_backend()
}

/// `pygame.transform.set_smoothscale_backend`.
#[pyfunction]
#[pyo3(name = "set_smoothscale_backend")]
pub fn tf_set_smoothscale_backend(backend: &str) -> PyResult<()> {
    transform::set_smoothscale_backend(backend).map_err(pyo3::exceptions::PyValueError::new_err)
}

/// `pygame.transform.threshold` - zaehlt Pixel und faerbt sie wahlweise.
///
/// Gibt eine **Zahl** zurueck, keine Flaeche: wie viele Pixel der
/// Bedingung entsprachen.
///
/// ## Sieben Fehlerarten, und alle sind `TypeError`
///
/// Hier lag der eigentliche Fehler. Der Kern prueft dasselbe und meldet
/// es als Wortlaut; die Huelle machte daraus durchweg `ValueError`. pygame
/// nimmt aber `TypeError`, und seine eigene Testdatei prueft genau das
/// (`assertRaises(TypeError, ...)` in `test_threshold_count`,
/// `test_threshold_set_behavior0`, `test_threshold_set_behavior2`,
/// `test_threshold_search_surf`). Gemessen, mit Wortlaut:
///
/// ```text
/// dest_surface=Weitergabe::Fehlt und set_behavior != 0
///     TypeError: argument 1 must be pygame.Surface, or None with set_behavior=1
/// set_behavior == 0 und set_color != None
///     TypeError: if set_behavior==0 set_color should be None
/// set_behavior == 0 und dest_surface != None
///     TypeError: if set_behavior==0 dest_surf_obj should be None
/// set_behavior == 2 und set_color != None
///     TypeError: if set_behavior==2 set_color should be None
/// search_surf und search_color zugleich
///     TypeError: if search_surf is used, search_color should be None
/// dest_surface anderer Groesse
///     TypeError: surf and dest_surf not the same size
/// ```
///
/// Die erste Meldung sagt "set_behavior=1", gemeint ist "= 0" - das ist
/// pygames Wortlaut, nicht ein Tippfehler hier. Und `set_behavior = 3`
/// oder `-1` sind **kein** Fehler, solange ein Ziel dasteht (gemessen:
/// `threshold(ziel, q, farbe, set_behavior=3)` gibt 0 zurueck).
///
/// `inverse_set` ist in pygame eine **Zahl**, kein `bool`:
/// `test_threshold_from_surface` reicht `inverse_set=1`, und eine
/// `bool`-Signatur gab dort
/// `'int' object cannot be converted to 'PyBool'`.
#[pyfunction]
#[pyo3(signature = (
    dest_surface, surface, search_color=None, threshold=None,
    set_color=None, set_behavior=1, search_surf=None, inverse_set=None
))]
#[allow(clippy::too_many_arguments)]
#[pyo3(name = "threshold")]
pub fn tf_threshold(
    dest_surface: Option<&Bound<'_, PyAny>>,
    surface: &PySurface,
    search_color: Option<&Bound<'_, PyAny>>,
    threshold: Option<&Bound<'_, PyAny>>,
    set_color: Option<&Bound<'_, PyAny>>,
    set_behavior: i32,
    search_surf: Option<&PySurface>,
    inverse_set: Option<&Bound<'_, PyAny>>,
) -> PyResult<u32> {
    let quelle = surface.leihen()?;
    let ziel_da = matches!(dest_surface, Some(d) if !d.is_none());
    let setz_da = matches!(set_color, Some(c) if !c.is_none());

    if !ziel_da && set_behavior != 0 {
        return Err(PyTypeError::new_err(
            "argument 1 must be pygame.Surface, or None with set_behavior=1",
        ));
    }
    if set_behavior == 0 && setz_da {
        return Err(PyTypeError::new_err(
            "if set_behavior==0 set_color should be None",
        ));
    }
    if set_behavior == 0 && ziel_da {
        return Err(PyTypeError::new_err(
            "if set_behavior==0 dest_surf_obj should be None",
        ));
    }
    if set_behavior == 2 && setz_da {
        return Err(PyTypeError::new_err(
            "if set_behavior==2 set_color should be None",
        ));
    }
    let such = farbe_oder_nichts(search_color)?;
    if search_surf.is_some() && such.is_some() {
        return Err(PyTypeError::new_err(
            "if search_surf is used, search_color should be None",
        ));
    }
    let schwelle = farbe_oder_nichts(threshold)?;
    let setz = farbe_oder_nichts(set_color)?;
    let umgekehrt = match inverse_set {
        None => false,
        Some(x) if x.is_none() => false,
        // Eine ganze Zahl, wie pygame - `1` heisst wahr, `0` falsch.
        Some(x) => x.extract::<i64>()? != 0,
    };
    let such_bild = match search_surf {
        None => None,
        Some(s) => {
            let b = s.leihen()?;
            // Auch das ist ein `TypeError`, nicht der `ValueError`, den der
            // Kern daraus machen wuerde - `test_threshold_search_surf`
            // prueft es zweimal.
            if b.get_size() != quelle.get_size() {
                return Err(PyTypeError::new_err("surf and search_surf not the same size"));
            }
            Some(b.clone())
        }
    };
    match dest_surface {
        Some(d) if !d.is_none() => {
            let z: PyRef<PySurface> = d.extract()?;
            let mut ziel = z.leihen_mut()?;
            if ziel.get_size() != quelle.get_size() {
                return Err(PyTypeError::new_err("surf and dest_surf not the same size"));
            }
            transform::threshold(
                Some(&mut ziel),
                &quelle,
                such,
                schwelle,
                setz,
                set_behavior,
                such_bild.as_ref(),
                umgekehrt,
            )
            .map_err(umform_fehler)
        }
        _ => transform::threshold(
            None,
            &quelle,
            such,
            schwelle,
            setz,
            set_behavior,
            such_bild.as_ref(),
            umgekehrt,
        )
        .map_err(umform_fehler),
    }
}

/// `None` bleibt `None`, alles andere wird zur Farbe.
fn farbe_oder_nichts(
    o: Option<&Bound<'_, PyAny>>,
) -> PyResult<Option<rustgame_core::color::Color>> {
    match o {
        None => Ok(None),
        Some(c) if c.is_none() => Ok(None),
        Some(c) => Ok(Some(crate::umwandlung::farbe_aus(c)?)),
    }
}
