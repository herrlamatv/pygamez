//! bild.rs
//! =======
//! `pygame.image`.
//!
//! ## Die Stelle, an der ein Fehler unbemerkt bleibt
//!
//! `tobytes` und `frombytes` sind zueinander die Umkehrung - **ausser
//! bei zwei Formaten**, und das ist gemessen, nicht geraten:
//!
//! ```text
//! RGB   -> Flaeche ohne SRCALPHA, Alpha geht verloren
//! RGBX  -> Flaeche ohne SRCALPHA, das vierte Byte faellt weg
//! RGBA, ARGB, BGRA, ABGR -> Flaeche mit SRCALPHA, verlustfrei
//! ```
//!
//! Wer das nicht prueft, merkt jahrelang nicht, dass `RGBX` beim
//! Rundlauf sein Alpha abgibt. Der Vergleich im Kratzordner faehrt
//! deshalb alle acht Formate gegen echtes pygame.
//!
//! ## `pitch`
//!
//! pygame schreibt `-1` fuer "so lang wie noetig", der Kern `None`.
//! Uebersetzt wird das in `pitch_aus` - an einer Stelle, nicht in
//! jeder der vier Funktionen.

use std::path::PathBuf;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyString, PyTuple};

use rustgame_core::image;

use crate::flaeche::PySurface;
use crate::umwandlung::bild_fehler;

/// pygames `-1` heisst "so lang wie noetig".
fn pitch_aus(pitch: i64) -> Option<usize> {
    if pitch < 0 {
        None
    } else {
        Some(pitch as usize)
    }
}

/// Ein Dateiname - oder ein Dateiobjekt, aus dem gelesen wird.
enum Quelle {
    Pfad(PathBuf),
    Bytes(Vec<u8>),
}

/// pygame nimmt an jeder Stelle, wo ein Dateiname stehen darf, auch ein
/// offenes Dateiobjekt. Beides wird hier auseinandergehalten.
fn quelle_aus(o: &Bound<'_, PyAny>) -> PyResult<Quelle> {
    if let Ok(s) = o.downcast::<PyString>() {
        return Ok(Quelle::Pfad(PathBuf::from(s.to_cow()?.into_owned())));
    }
    if let Ok(p) = o.call_method0("__fspath__") {
        return Ok(Quelle::Pfad(PathBuf::from(p.extract::<String>()?)));
    }
    if let Ok(daten) = o.call_method0("read") {
        return Ok(Quelle::Bytes(daten.extract::<Vec<u8>>()?));
    }
    Err(PyTypeError::new_err(
        "expected a filename or a file object",
    ))
}

/// pygames Satz fuer eine Datei, die es nicht gibt.
///
/// Das Betriebssystem hat seinen eigenen (`Das System kann die angegebene
/// Datei nicht finden`), und der steht in der Sprache des Systems. pygame
/// schreibt ihn selbst, mit dem Arbeitsverzeichnis dazu - `image_test.py`
/// und `mixer_test.py` zeigen ihn beide an. Siehe `ton.rs`, wo derselbe
/// Satz schon stand.
/// Die Daten und der Typ, so wie `IMG_LoadTyped_RW` sie bekommt.
///
/// ## Hinweis nicht angegeben ist etwas anderes als ein leerer Hinweis
///
/// Gemessen an pygame-ce 2.5.7 mit einer TGA-Datei:
///
/// ```text
/// load("a.tga")              geht         (Typ aus dem Dateinamen)
/// load("a.tga", "")          Unsupported image format
/// load("a.png", "x.tga")     geht         (der Hinweis gewinnt)
/// load(open("a.tga", "rb"))  Unsupported image format  (kein `.name`)
/// load("a.tga", 5)           TypeError: argument 2 must be str, not int
/// ```
///
/// Darum ist der Hinweis hier `Option`: `None` heisst "nicht angegeben".
/// Hier stand `let _ = namehint;` mit der Begruendung, rustgame erkenne
/// "am Inhalt, genau wie SDL_image" - fuer TGA, das keine Kennung hat,
/// stimmt das nicht.
///
/// Ein Ordner ist **keine** Datei: hier stand `exists()`, und pygame sagt
/// fuer einen Ordner dasselbe wie fuer einen fehlenden Namen.
fn daten_und_typ(
    q: Quelle,
    namehint: Option<&Bound<'_, PyAny>>,
) -> PyResult<(Vec<u8>, String)> {
    let hinweis: Option<String> = match namehint {
        None => None,
        Some(h) => match h.downcast::<PyString>() {
            Ok(s) => Some(s.to_cow()?.into_owned()),
            Err(_) => {
                let typ = h.get_type().name()?;
                return Err(pyo3::exceptions::PyTypeError::new_err(format!(
                    "argument 2 must be str, not {typ}"
                )));
            }
        },
    };
    match q {
        Quelle::Pfad(p) => {
            if !p.is_file() {
                return Err(datei_fehlt(&p));
            }
            let daten = std::fs::read(&p).map_err(|_| datei_fehlt(&p))?;
            let name = hinweis.unwrap_or_else(|| {
                p.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            });
            Ok((daten, image::typ_aus_namen(&name).to_string()))
        }
        Quelle::Bytes(b) => {
            let name = hinweis.unwrap_or_default();
            Ok((b, image::typ_aus_namen(&name).to_string()))
        }
    }
}

fn datei_fehlt(wie: &std::path::Path) -> PyErr {
    let wo = std::env::current_dir()
        .map(|d| d.display().to_string())
        .unwrap_or_default();
    pyo3::exceptions::PyFileNotFoundError::new_err(format!(
        "No file '{}' found in working directory '{wo}'.",
        wie.display()
    ))
}

/// `pygame.image.load`.
#[pyfunction]
#[pyo3(signature = (file, namehint=None))]
#[pyo3(name = "load")]
pub fn bild_load(
    file: &Bound<'_, PyAny>,
    namehint: Option<&Bound<'_, PyAny>>,
) -> PyResult<PySurface> {
    // Der Hinweis zaehlt - siehe `daten_und_typ`.
    let (daten, typ) = daten_und_typ(quelle_aus(file)?, namehint)?;
    let s = image::load_from_bytes_als(&daten, &typ);
    Ok(PySurface::neu(s.map_err(bild_fehler)?))
}

/// `pygame.image.save`.
#[pyfunction]
#[pyo3(signature = (surface, file, namehint=""))]
#[pyo3(name = "save")]
pub fn bild_save(
    surface: &PySurface,
    file: &Bound<'_, PyAny>,
    namehint: &str,
) -> PyResult<()> {
    let s = surface.leihen()?;
    if let Ok(pfad) = file.extract::<String>() {
        return image::save(&s, pfad).map_err(bild_fehler);
    }
    if let Ok(p) = file.call_method0("__fspath__") {
        return image::save(&s, p.extract::<String>()?).map_err(bild_fehler);
    }
    // Ein Dateiobjekt: die Endung steckt dann im `namehint` - und **ohne
    // Hinweis wird TGA geschrieben, nicht PNG**. Das steht wortwoertlich
    // in `src_c/image.c`:
    //
    // ```c
    // name = (namehint ? namehint : "tga");
    // ```
    //
    // `image_test.test_save_to_fileobject` prueft genau das: es schreibt
    // ohne Hinweis in ein `BytesIO` und liest mit `load(stream, "tga")`
    // zurueck. Hier stand `"png"` - eine naheliegende Wahl, die den
    // Rundlauf zerreisst.
    let endung = if namehint.is_empty() {
        "tga"
    } else {
        namehint.rsplit('.').next().unwrap_or("tga")
    };
    let mut puffer = Vec::new();
    image::save_to_writer(&s, &mut puffer, endung).map_err(bild_fehler)?;
    file.call_method1("write", (PyBytes::new(file.py(), &puffer),))?;
    Ok(())
}

/// `pygame.image.tobytes`.
#[pyfunction]
#[pyo3(signature = (surface, format, flipped=false, pitch=-1))]
#[pyo3(name = "tobytes")]
pub fn bild_tobytes<'p>(
    py: Python<'p>,
    surface: &PySurface,
    format: &str,
    flipped: bool,
    pitch: i64,
) -> PyResult<Bound<'p, PyBytes>> {
    let s = surface.leihen()?;
    let daten = image::tobytes_ex(&s, format, flipped, pitch_aus(pitch)).map_err(bild_fehler)?;
    Ok(PyBytes::new(py, &daten))
}

/// `pygame.image.frombytes`.
#[pyfunction]
#[pyo3(signature = (bytes, size, format, flipped=false, pitch=-1))]
#[pyo3(name = "frombytes")]
pub fn bild_frombytes(
    bytes: &[u8],
    size: (u32, u32),
    format: &str,
    flipped: bool,
    pitch: i64,
) -> PyResult<PySurface> {
    let s = image::frombytes_ex(bytes, size, format, flipped, pitch_aus(pitch))
        .map_err(bild_fehler)?;
    Ok(PySurface::neu(s))
}

/// `pygame.image.tostring` - der **alte Name** von `tobytes`, und er warnt.
///
/// Gemessen: `DeprecationWarning: pygame.image.tostring deprecated since
/// 2.3.0`. Die beiden alten Namen standen hier als blosse Zweitnamen auf
/// `tobytes`/`frombytes` eingetragen - dieselbe Funktion, also auch
/// dieselbe (fehlende) Warnung. Eine eigene Hulle ist noetig, damit der
/// alte Name sagen kann, dass er alt ist.
#[pyfunction]
#[pyo3(signature = (surface, format, flipped=false, pitch=-1))]
#[pyo3(name = "tostring")]
pub fn bild_tostring<'p>(
    py: Python<'p>,
    surface: &PySurface,
    format: &str,
    flipped: bool,
    pitch: i64,
) -> PyResult<Bound<'p, PyBytes>> {
    veraltet(py, "pygame.image.tostring deprecated since 2.3.0")?;
    bild_tobytes(py, surface, format, flipped, pitch)
}

/// `pygame.image.fromstring` - siehe [`bild_tostring`].
#[pyfunction]
#[pyo3(signature = (bytes, size, format, flipped=false, pitch=-1))]
#[pyo3(name = "fromstring")]
pub fn bild_fromstring(
    py: Python<'_>,
    bytes: &[u8],
    size: (u32, u32),
    format: &str,
    flipped: bool,
    pitch: i64,
) -> PyResult<PySurface> {
    veraltet(py, "pygame.image.fromstring deprecated since 2.3.0")?;
    bild_frombytes(bytes, size, format, flipped, pitch)
}

/// Meldet eine `DeprecationWarning` mit pygames Wortlaut.
fn veraltet(py: Python<'_>, text: &str) -> PyResult<()> {
    let warnungen = py.import("warnings")?;
    let art = py.get_type::<pyo3::exceptions::PyDeprecationWarning>();
    warnungen.call_method1("warn", (text, art))?;
    Ok(())
}

/// `pygame.image.frombuffer`.
///
/// **Ein Unterschied zu pygame, mit Absicht:** dort teilt sich die Flaeche
/// den Speicher mit dem Puffer. Eine `Surface` besitzt ihren Speicher
/// selbst, also wird kopiert - das steht schon so im Kern.
#[pyfunction]
#[pyo3(signature = (buffer, size, format, pitch=-1))]
#[pyo3(name = "frombuffer")]
pub fn bild_frombuffer(
    buffer: &[u8],
    size: (u32, u32),
    format: &str,
    pitch: i64,
) -> PyResult<PySurface> {
    let s = image::frombuffer(buffer, size, format, pitch_aus(pitch)).map_err(bild_fehler)?;
    Ok(PySurface::neu(s))
}

/// `pygame.image.get_extended` - koennen mehr Formate als BMP gelesen
/// werden? rustgame kann PNG, also ja.
#[pyfunction]
#[pyo3(name = "get_extended")]
pub fn bild_get_extended() -> bool {
    image::get_extended()
}

/// `pygame.image.get_sdl_image_version`.
///
/// Immer `None`: rustgame bindet kein SDL_image ein. Echtes pygame gibt
/// hier `(2, 8, 8)` zurueck - eine Zahl zu erfinden waere gelogen, und
/// wer nur wissen will, *ob* PNG geht, fragt [`bild_get_extended`].
#[pyfunction]
#[pyo3(signature = (linked=None))]
#[pyo3(name = "get_sdl_image_version")]
pub fn bild_get_sdl_image_version(linked: Option<&Bound<'_, PyAny>>) -> PyResult<Option<(u32, u32, u32)>> {
    // **Jeder Wert ist ein Wahrheitswert** - pygame liest den
    // Schalter mit `PyObject_IsTrue`, und das nimmt alles. Mit
    // `bool` verlangte PyO3 einen echten Wahrheitswert und nannte
    // im Fehlertext `'PyBool'`, einen Namen aus Rust.
    let linked = match linked {
        None => true,
        Some(o) => o.is_truthy()?,
    };
    let _ = linked;
    Ok(image::get_sdl_image_version())
}

/// `pygame.image.load_basic`.
///
/// In pygame der Lader **ohne** SDL_image: er kann nur BMP. rustgame hat
/// diese Trennung nicht - es gibt genau einen Lader -, aber der Name
/// muss da sein, damit ein Programm ihn aufrufen kann. Der Kern fuehrt
/// ihn deshalb als eigene Funktion, die auf dieselbe Stelle zeigt.
#[pyfunction]
#[pyo3(name = "load_basic")]
#[pyo3(signature = (file, /))]
pub fn bild_load_basic(file: &Bound<'_, PyAny>) -> PyResult<PySurface> {
    let s = match quelle_aus(file)? {
        Quelle::Pfad(p) => {
            if !p.is_file() {
                return Err(datei_fehlt(&p));
            }
            image::load_basic(p)
        }
        Quelle::Bytes(b) => image::load_from_bytes(&b),
    };
    Ok(PySurface::neu(s.map_err(bild_fehler)?))
}

/// `pygame.image.load_animation` - eine bewegte Bildfolge.
///
/// Gibt `list[tuple[Surface, float]]`. **Die Dauer ist in
/// Millisekunden**, gemessen: `animated_sample.gif` liefert zehnmal
/// `150.0`, und in der Datei stehen 15 Hundertstelsekunden.
///
/// Ein Standbild gibt eine Liste mit einem Eintrag - auch das gemessen.
#[pyfunction]
#[pyo3(signature = (file, namehint=None))]
#[pyo3(name = "load_animation")]
pub fn bild_load_animation(
    py: Python<'_>,
    file: &Bound<'_, PyAny>,
    namehint: Option<&Bound<'_, PyAny>>,
) -> PyResult<Py<PyList>> {
    let (daten, typ) = daten_und_typ(quelle_aus(file)?, namehint)?;
    let folge = image::load_animation_from_bytes_als(&daten, &typ).map_err(bild_fehler)?;
    let liste = PyList::empty(py);
    for (s, dauer) in folge {
        let paar = PyTuple::new(py, [PySurface::neu(s).into_pyobject(py)?.into_any(), dauer.into_pyobject(py)?.into_any()])?;
        liste.append(paar)?;
    }
    Ok(liste.unbind())
}

/// `pygame.image.load_sized_svg(file, size)`.
///
/// **Die Groesse wird geprueft, bevor die Datei angefasst wird** - und
/// das ist kein Beiwerk, sondern der Teil der Schnittstelle, den pygames
/// Testsuite mit acht Unterfaellen abklopft. Gemessen:
///
/// ```text
/// [100]                 TypeError: size must be two numbers
/// (0, 0, 10, 3)         TypeError: dasselbe
/// "foo"                 TypeError: dasselbe
/// Vector3(-3, 10, 10)   TypeError: dasselbe
/// [100, 0]              ValueError: both components of size must be positive
/// (0, 0)                ValueError: dasselbe
/// [-2, -1]              ValueError: dasselbe
/// Vector2(-3, 10)       ValueError: dasselbe
/// ```
///
/// Fliesskomma wird angenommen und **abgeschnitten** (`(2.7, 3.9)` gibt
/// zwei mal zwei), und das Ergebnis behaelt das Seitenverhaeltnis:
/// `teal.svg` ist quadratisch, also kommt bei `[4, 2]` eine Flaeche von
/// `(2, 2)` heraus, nicht `(4, 2)`.
///
/// **Gezeichnet wird hier trotzdem nichts** - siehe die benannte Grenze
/// zu SVG. Ein Vektorzeichner mit Pfaden, Fuellregeln, Verlaeufen und
/// Text waere groesser als dieser ganze Kern; die Pruefung aber gehoert
/// zur Schnittstelle und ist unabhaengig davon richtig.
#[pyfunction]
#[pyo3(signature = (file, size))]
#[pyo3(name = "load_sized_svg")]
pub fn bild_load_sized_svg(
    file: &Bound<'_, PyAny>,
    size: &Bound<'_, PyAny>,
) -> PyResult<PySurface> {
    let (w, h) = zwei_zahlen(size)?;
    if w <= 0 || h <= 0 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "both components of size must be positive",
        ));
    }
    let s = match quelle_aus(file)? {
        Quelle::Pfad(p) => {
            if !p.is_file() {
                return Err(datei_fehlt(&p));
            }
            image::load_sized_svg(p, (w as u32, h as u32))
        }
        Quelle::Bytes(b) => image::load_sized_svg_from_bytes(&b, (w as u32, h as u32)),
    };
    Ok(PySurface::neu(s.map_err(bild_fehler)?))
}

/// Genau **zwei** Zahlen - nicht eine, nicht drei.
///
/// Eigener Helfer, weil `zwei_ganze_aus` den Einzeiler `(1,)` auspackt
/// (so steht es in `pg_TwoIntsFromObj`) und `load_sized_svg` ihn
/// ablehnt. Fliesskomma wird zur Null hin abgeschnitten.
fn zwei_zahlen(o: &Bound<'_, PyAny>) -> PyResult<(i64, i64)> {
    let fehler = || PyTypeError::new_err("size must be two numbers");
    if o.is_instance_of::<PyString>() || o.len().map(|n| n != 2).unwrap_or(true) {
        return Err(fehler());
    }
    let zahl = |n: usize| -> PyResult<i64> {
        let e = o.get_item(n).map_err(|_| fehler())?;
        if let Ok(f) = e.extract::<f64>() {
            return Ok(f as i64);
        }
        e.extract::<i64>().map_err(|_| fehler())
    };
    Ok((zahl(0)?, zahl(1)?))
}

/// `pygame.image.load_extended` - siehe [`bild_load_basic`].
#[pyfunction]
#[pyo3(signature = (file, namehint=None))]
#[pyo3(name = "load_extended")]
pub fn bild_load_extended(
    file: &Bound<'_, PyAny>,
    namehint: Option<&Bound<'_, PyAny>>,
) -> PyResult<PySurface> {
    // Derselbe Weg wie `load` - siehe `image::load_extended`.
    let (daten, typ) = daten_und_typ(quelle_aus(file)?, namehint)?;
    let s = image::load_from_bytes_als(&daten, &typ);
    Ok(PySurface::neu(s.map_err(bild_fehler)?))
}

/// `pygame.image.save_extended`.
///
/// In pygame der Speicherer **mit** SDL_image (PNG, JPEG). rustgame
/// schreibt ohnehin nur ueber einen Weg; der Name zeigt darauf, damit
/// ein Programm ihn aufrufen kann.
#[pyfunction]
#[pyo3(signature = (surface, file, namehint=""))]
#[pyo3(name = "save_extended")]
pub fn bild_save_extended(
    surface: &PySurface,
    file: &Bound<'_, PyAny>,
    namehint: &str,
) -> PyResult<()> {
    let s = surface.leihen()?;
    if let Ok(pfad) = file.extract::<String>() {
        return image::save_extended(&s, pfad).map_err(bild_fehler);
    }
    if let Ok(p) = file.call_method0("__fspath__") {
        return image::save_extended(&s, p.extract::<String>()?).map_err(bild_fehler);
    }
    // **Was weder Pfad noch Datei ist, ist ein `TypeError`.** Gemessen
    // gibt `save_extended(surface, [])` genau den; hier kam vorher die
    // Meldung ueber eine unbekannte Endung, und die beschreibt einen
    // ganz anderen Fehler.
    if !file.hasattr("write")? {
        return Err(pyo3::exceptions::PyTypeError::new_err(
            "file argument must be a path or a writable file object",
        ));
    }
    let endung = namehint.rsplit('.').next().unwrap_or("png");
    let mut puffer = Vec::new();
    image::save_to_writer(&s, &mut puffer, endung).map_err(bild_fehler)?;
    file.call_method1("write", (PyBytes::new(file.py(), &puffer),))?;
    Ok(())
}
