//! umwandlung.rs
//! =============
//! Die Grenzstelle: was Python hereinreicht, wird hier zu dem, was der
//! Rust-Kern erwartet - und was der Kern an Fehlern meldet, wird hier zu
//! einer Python-Ausnahme.
//!
//! Warum das an **einer** Stelle steht: pygame nimmt eine Farbe in fuenf
//! Schreibweisen und ein Rechteck in drei. Wuerde jede der rund
//! zweihundert Funktionen dieser Huelle das selbst entscheiden, gaebe es
//! zweihundert Gelegenheiten, eine Schreibweise zu vergessen.
//!
//! ## Kein `unwrap` auf Werten aus Python
//!
//! Jede Umwandlung hier gibt ein [`PyResult`] zurueck. Ein Programm, das
//! `pygame.draw.rect(s, "kein farbname", ...)` schreibt, bekommt eine
//! `ValueError` - und zwar mit dem Wortlaut, den echtes pygame benutzt,
//! damit ein `except ValueError as e: print(e)` dasselbe ausgibt.

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyInt;

use rustgame_core::color::{self, Color};
use rustgame_display;
use rustgame_core::rect::Rect as KernRect;

use crate::farbe::PyColor;
use crate::rechteck::PyRect;

/// Die Ausnahme selbst - in einem eigenen Modul, weil sie in Python
/// **`error`** heissen muss und nicht `PygameError`.
///
/// Gemessen: `type(e).__name__` ist dort `error`, und ein Programm, das
/// `except Exception as e: print(type(e).__name__, e)` schreibt, gibt
/// genau das aus. PyO3 leitet den Python-Namen aus dem Rust-Namen ab -
/// also heisst der Typ hier klein, und die Warnung dazu ist an dieser
/// einen Stelle abgeschaltet statt in der ganzen Datei.
pub mod ausnahme {
    #![allow(non_camel_case_types)]
    use pyo3::create_exception;
    use pyo3::exceptions::PyRuntimeError;

    create_exception!(
        rustgame_py,
        error,
        PyRuntimeError,
        "`pygame.error`. Erbt von `RuntimeError` - gemessen an echtem \
         pygame, dessen `pygame.error.__mro__` genau so aussieht."
    );

    create_exception!(
        rustgame_py,
        BufferError,
        pyo3::exceptions::PyBufferError,
        "`pygame.BufferError`. Ein **eigener** Typ, der von Pythons \
         `BufferError` erbt - gemessen: `pygame.BufferError.__mro__` ist \
         `(pygame.BufferError, BufferError, Exception, BaseException, \
         object)`. Wer nur Pythons `BufferError` wirft, wird von \
         `except pygame.BufferError` nicht gefangen."
    );
}

pub use ausnahme::error as PygameError;
pub use ausnahme::BufferError as PygameBufferError;

/// Lehnt Schluesselwoerter ab - **mit pygames Satz.**
///
/// Eine Funktion, die `*args` nimmt, muss die Zahl ihrer Argumente selbst
/// zaehlen (nur so kommt pygames eigener Satz heraus, etwa
/// `clamp requires 3 arguments`). Damit verliert sie aber auch PyO3s
/// Ablehnung von Schluesselwoertern: ein `clamp(value=1)` landete im
/// `**kwargs` und kam durch, wo pygame
/// `pygame.math.clamp() takes no keyword arguments` sagt. Dieser Helfer
/// stellt den Satz wieder her.
///
/// Laeuft das Videoteilsystem? Sonst **pygames Satz**.
///
/// pygame prueft das vor jeder Ereignis-, Tasten- und Mausfrage
/// (`VIDEO_INIT_CHECK()` in seinem C-Stueck) und sagt
/// `pygame.error: video system not initialized`. Gemessen in drei
/// Zustaenden - vor `init()`, laufend, nach `quit()` -: **27** Funktionen
/// gingen hier stillschweigend durch und gaben einen Wert, wo pygame
/// einen Fehler gibt. Das ist die schlimmste Sorte Unterschied: ein
/// Programm, das `init()` vergisst, bekam hier `(0, 0)` statt eines
/// Hinweises.
///
/// Welche Funktion das Tor hat und welche nicht, ist **gemessen** und
/// nicht herzuleiten: `key.name`, `key.key_code`, `event.custom_type`,
/// `mouse.get_relative_mode`, `display.get_num_displays` und
/// `display.get_active` antworten ohne Video (die letzten zwei mit `0`
/// und `False`), `time.delay` und `time.wait` auch.
pub fn video_pruefen() -> PyResult<()> {
    if rustgame_display::display::get_init() {
        return Ok(());
    }
    Err(PygameError::new_err("video system not initialized"))
}

/// Gefunden hat die Luecke die Probe `Schluesselwoerter`, die beiden
/// Huellen pygames eigenen Parameternamen vorlegt.
pub fn keine_woerter(
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
    voller_name: &str,
) -> PyResult<()> {
    if kwargs.is_some_and(|k| !k.is_empty()) {
        return Err(pyo3::exceptions::PyTypeError::new_err(format!(
            "{voller_name}() takes no keyword arguments"
        )));
    }
    Ok(())
}

/// Der Wortlaut, mit dem pygame eine unbrauchbare Farbe ablehnt.
///
/// Wortgleich uebernommen, weil Programme ihn anzeigen.
pub const FARBE_KAPUTT: &str = "invalid color (color sequence must have size 3 or 4, \
                                and each element must be an integer in the range [0, 255])";

/// Nimmt alles an, was in pygame als Farbe durchgeht.
///
/// `Color`, ein Name (`"red"`), `#rrggbb`, eine Folge aus drei oder vier
/// Zahlen (Liste **oder** Tupel - pygame nimmt beides) und eine einzelne
/// ganze Zahl. Letztere ist die Falle: pygame liest sie als **RGBA**,
/// nicht als ARGB - `Color(0x11223344)` ist `(17, 34, 51, 68)`, gemessen.
/// Eine Flaggenzahl von Python - **`i64`, nicht `u32`**.
///
/// `pygame.FULLSCREEN` ist `0x8000_0000`. Weil das oberste Bit gesetzt
/// ist, fuehrt pygame die Konstante als **negative** Zahl, und
/// `rustgame_py.FULLSCREEN` tut dasselbe - sonst waere schon der Wert
/// verschieden. Ein `u32`-Beiwert nimmt sie dann nicht an:
///
/// ```text
/// display.set_mode((64, 48), FULLSCREEN)
///   -> OverflowError: can't convert negative int to unsigned
/// ```
///
/// Genau das ist lange passiert, und zwar beim haeufigsten
/// Vollbildaufruf ueberhaupt. Aufgefallen ist es erst, als
/// `display.mode_ok` dazukam und im Vergleich mit echtem pygame stolperte
/// - `set_mode` selbst hatte niemand mit `FULLSCREEN` geprueft.
///
/// Von den 592 Konstanten sind nur drei negativ: `FULLSCREEN` und die
/// beiden `CONTROLLER_*_INVALID` (beide `-1`). Trotzdem geht **jede**
/// Flaggenangabe durch diese Stelle - eine Regel, die nur fuer einen Wert
/// gilt, wird beim naechsten vergessen.
pub fn flagge(v: i64) -> u32 {
    v as u32
}

/// Aus allem, was pygame als Farbe nimmt, eine [`Color`].
///
/// Dreier- und Vierertupel, `Color`, Zahl, Farbname - und jede
/// Ablehnung mit pygames Wortlaut.
pub fn farbe_aus(o: &Bound<'_, PyAny>) -> PyResult<Color> {
    if let Ok(c) = o.extract::<PyRef<PyColor>>() {
        return Ok(c.wert());
    }
    if let Ok(name) = o.extract::<String>() {
        return color::parse(&name)
            // **Ohne den Namen dahinter.** pygame sagt nur
            // `invalid color name`; der Anhang war gut gemeint und
            // steht in keinem pygame-Satz.
            .ok_or_else(|| PyValueError::new_err("invalid color name"));
    }
    if let Ok(folge) = o.extract::<Vec<i64>>() {
        return folge_zu_farbe(&folge);
    }
    if let Ok(v) = o.extract::<u32>() {
        // pygame liest eine blanke Zahl als RGBA - siehe oben.
        return Ok(Color::rgba(
            (v >> 24) as u8,
            (v >> 16) as u8,
            (v >> 8) as u8,
            v as u8,
        ));
    }
    Err(PyValueError::new_err(FARBE_KAPUTT))
}

/// Drei oder vier Zahlen, jede zwischen 0 und 255.
pub fn folge_zu_farbe(folge: &[i64]) -> PyResult<Color> {
    if folge.len() != 3 && folge.len() != 4 {
        return Err(PyValueError::new_err(FARBE_KAPUTT));
    }
    let mut kanal = [0u8; 4];
    kanal[3] = 255;
    for (i, v) in folge.iter().enumerate() {
        if !(0..=255).contains(v) {
            return Err(PyValueError::new_err(FARBE_KAPUTT));
        }
        kanal[i] = *v as u8;
    }
    Ok(Color::rgba(kanal[0], kanal[1], kanal[2], kanal[3]))
}

/// Nimmt ein `Rect` oder ein Viertupel - wie pygame es ueberall tut.
pub fn rect_aus(o: &Bound<'_, PyAny>) -> PyResult<KernRect> {
    // **Eine** Quelle fuer alle. `rechteck::rect_aus_objekt` ist der
    // zeilengetreue Port von `RectFromObject` aus `src_c/rect.c`; was
    // hier vorher stand, war eine Annaeherung daran.
    //
    // Der Unterschied war nicht klein. Die Annaeherung lehnte
    // Fliesskomma ab, kannte `FRect` nicht, folgte einem Feld `rect`
    // nur eine Stufe und packte einelementige Tupel nicht aus. Und weil
    // **alles ausser `Rect`/`FRect` selbst** hier durchgeht -
    // `zeichnen`, `flaeche`, `figur`, `maske`, `text` -, galten die
    // Luecken dort weiter, nachdem sie in `rechteck.rs` schon
    // geschlossen waren.
    //
    // Das ist die Sorte Fehler, die dieses Projekt am haeufigsten
    // gemacht hat: eine Rechnung wird an einer Stelle richtig gestellt
    // und die zweite Abschrift bleibt stehen. Deshalb gibt es hier
    // keine zweite mehr.
    crate::rechteck::rect_aus_objekt(o).ok_or_else(crate::rechteck::nicht_rechteckig)
}

/// Wie [`rect_aus`], aber `None` bleibt `None`.
pub fn rect_aus_opt(o: Option<&Bound<'_, PyAny>>) -> PyResult<Option<KernRect>> {
    match o {
        None => Ok(None),
        Some(x) if x.is_none() => Ok(None),
        Some(x) => Ok(Some(rect_aus(x)?)),
    }
}

/// Eine Liste von Rechtecken.
///
/// Hier stand "fuer `unionall`, `collidelist`, `update`" - genau die drei
/// benutzen es inzwischen **nicht** mehr, seit `rechteck.rs` pygames
/// Folgenpruefung zeilengetreu nachbaut (ein Erzeugerausdruck ist dort
/// ein `TypeError`, hier nicht). Wer die Liste hier benutzt, nimmt die
/// bequemere Regel - das ist fuer `draw.lines` und Freunde richtig und
/// fuer die Rechteckmethoden nicht.
pub fn rects_aus(o: &Bound<'_, PyAny>) -> PyResult<Vec<KernRect>> {
    let mut aus = Vec::new();
    for e in o.try_iter()? {
        aus.push(rect_aus(&e?)?);
    }
    Ok(aus)
}

/// Ein Zielpunkt fuer `blit` - pygame nimmt dort auch ein `Rect` und
/// benutzt dann dessen linke obere Ecke.
///
/// Gleitkomma wird abgeschnitten, nicht gerundet: `blit(s, (1.9, 0))`
/// landet in pygame bei x = 1.
pub fn punkt_aus(o: &Bound<'_, PyAny>) -> PyResult<(i32, i32)> {
    if let Ok(r) = o.extract::<PyRef<PyRect>>() {
        let w = r.wert();
        return Ok((w.x, w.y));
    }
    if let Ok(v) = o.extract::<Vec<f64>>() {
        if v.len() >= 2 {
            return Ok((v[0] as i32, v[1] as i32));
        }
    }
    Err(PyTypeError::new_err("expected a point: (x, y) or a Rect"))
}

/// Ein **Versatz** - zwei Zahlen, und zwar aus allem, was zwei Zahlen
/// hergibt.
///
/// Nicht dasselbe wie [`punkt_aus`]: gemessen nimmt pygames `offset`
/// **kein** Rechteck (`m.overlap(n, Rect(0,0,1,1))` gibt
/// `TypeError: offset must be two numbers`), wohl aber einen `Vector2` -
/// und `mask_test.py` benutzt genau das.
///
/// Der Grund, warum ein `Vector2` nicht von selbst durchgeht: PyO3
/// gewinnt ein `Vec<f64>` ueber das **Folgen**-Protokoll, und ein
/// `#[pymethods]`-`__getitem__` landet bei `mp_subscript`, nicht bei
/// `sq_item`. Deshalb wird hier notfalls von Hand durchgezaehlt.
pub fn versatz_aus(o: &Bound<'_, PyAny>) -> PyResult<(i32, i32)> {
    versatz_aus_wort(o, "offset")
}

/// Dasselbe, aber mit **pygames Wort** fuer diese Stelle.
///
/// pygame nennt das Argument im Fehlertext jedes Mal anders, und zwar
/// nach seiner Rolle - gemessen:
///
/// ```text
/// Mask.get_at("x")              pos must be two numbers
/// Mask.set_at("x")              pos must be two numbers
/// Mask.connected_component("x") pos must be two numbers
/// Mask.scale("x")               scale must be two numbers
/// display.set_mode("x")         size must be two numbers
/// m.overlap(n, "x")             offset must be two numbers
/// ```
///
/// Hier stand ueberall `offset`, weil **ein** Helfer alle Stellen bedient
/// hat. Der Helfer ist richtig, der Satz war es an fuenf Stellen nicht.
pub fn versatz_aus_wort(o: &Bound<'_, PyAny>, wort: &str) -> PyResult<(i32, i32)> {
    if let Ok((x, y)) = o.extract::<(f64, f64)>() {
        return Ok((x as i32, y as i32));
    }
    if let Ok(v) = o.extract::<Vec<f64>>() {
        if v.len() == 2 {
            return Ok((v[0] as i32, v[1] as i32));
        }
    }
    // Der Umweg ueber `__getitem__`, fuer alles mit zwei Stellen.
    if let (Ok(a), Ok(b)) = (o.get_item(0), o.get_item(1)) {
        if let (Ok(x), Ok(y)) = (a.extract::<f64>(), b.extract::<f64>()) {
            if o.len().map(|n| n == 2).unwrap_or(false) {
                return Ok((x as i32, y as i32));
            }
        }
    }
    Err(PyTypeError::new_err(format!("{wort} must be two numbers")))
}

/// Eine Punktfolge fuer `draw.lines` und `draw.polygon`.
pub fn punkte_aus(o: &Bound<'_, PyAny>) -> PyResult<Vec<(i32, i32)>> {
    let mut aus = Vec::new();
    for e in o.try_iter().map_err(|_| {
        PyTypeError::new_err("points must be a sequence of number pairs")
    })? {
        aus.push(punkt_aus(&e?)?);
    }
    Ok(aus)
}

/// Eine Folge von Ereignistypen - `event.get([QUIT, KEYDOWN])` nimmt
/// ebenso eine einzelne Zahl wie eine Liste, und `None` heisst "alle".
pub fn typen_je<F>(o: Option<&Bound<'_, PyAny>>, mut tu: F) -> PyResult<Option<bool>>
where
    F: FnMut(u32) -> PyResult<bool>,
{
    let Some(o) = o else { return Ok(None) };
    if o.is_none() {
        return Ok(None);
    }

    // **Stueckweise, nicht erst zu Ende lesen.** Gemessen an einer
    // Schlange `A B C A B C A B C` und einer Liste, deren zweites Glied
    // kaputt ist:
    //
    // ```text
    // get([A, 2**31])          TypeError - und die A sind **weg**
    // get([2**31, A])          TypeError - nichts angefasst
    // clear([A, 2**31])        TypeError - die A sind geleert
    // set_blocked([B, 2**31])  TypeError - B ist gesperrt und ausgeraeumt
    // peek([A, 2**31])         **True**, kein Fehler
    // peek([2**31, A])         TypeError
    // get_blocked([B, 2**31])  TypeError (B war nicht gesperrt)
    // get_blocked([0, 2**31])  **True** (0 war gesperrt)
    // ```
    //
    // pygame liest also jedes Glied und **tut es sofort**; der Fehler
    // haelt die Schleife dort an, wo er auftritt. Und `peek` wie
    // `get_blocked` hoeren beim ersten Treffer auf - darum sehen sie das
    // kaputte Glied dahinter nie. Hier wurde erst die ganze Liste
    // gelesen und dann gehandelt: `peek([A, 2**31])` gab einen Fehler,
    // wo pygame `True` sagt.
    //
    // **Drei Fehlerarten, und sie haengen an der Gestalt des Werts.**
    // Gemessen, Wert fuer Wert, an echtem pygame:
    //
    // ```text
    // 0 .. 65534      gut
    // 65535           ValueError: event type out of range
    // 2**31-1         dasselbe            <- passt noch in ein C-`int`
    // -1, -2**31      dasselbe
    // 2**31, 2**63    TypeError: type sequence must contain valid event types
    // "x", b"0"       dasselbe            <- ein Text ist eine Folge
    // 1.5             TypeError: event type must be numeric or a sequence
    // {0: 1}, {0, 1}  dasselbe            <- keine Folgen fuer C
    // True            gut                 <- ein `bool` **ist** eine Zahl
    // [1.9]           gut, wird 1         <- in einer Folge zaehlt anders
    // [-0.5]          gut, wird 0         <- abgeschnitten, nicht gerundet
    // ["5"], [b"0"]   TypeError: type sequence ...
    // [None], [[0]]   dasselbe
    // range(3)        gut
    // ```
    //
    // Daraus die Regel, die pygames `event.c` zusammen mit
    // `pg_IntFromObj` ergibt:
    //
    // 1. Eine **ganze Zahl** (`PyLong_Check`, und ein `bool` ist eine):
    //    passt sie nicht in ein C-`int`, ist es ein `TypeError`; liegt
    //    sie ausserhalb `0 .. NUMEVENTS`, ein `ValueError`.
    // 2. Eine **Folge** (`PySequence_Check`: Liste, Tupel, Text, Bytes,
    //    `range` - aber kein Woerterbuch und keine Menge): jedes Glied
    //    einzeln, und zwar **milder** - eine Gleitzahl wird
    //    abgeschnitten, alles andere braucht `__index__`.
    // 3. Sonst: `event type must be numeric or a sequence`.
    //
    // Drei Dinge standen hier anders. Ein `bool` wurde **abgewiesen**,
    // mit dem Vermerk "ein `bool` ist in Python eine Zahl, aber kein
    // Ereignistyp" - gemessen ist er einer (`set_blocked(True)` sperrt
    // Typ 1). Eine Zahl jenseits von `i32` gab `ValueError` statt
    // `TypeError`. Und alles, was weder Zahl noch Folge ist, bekam den
    // Folgensatz statt des eigenen.
    fn falscher_typ() -> PyErr {
        PyTypeError::new_err("type sequence must contain valid event types")
    }
    fn ausser_bereich() -> PyErr {
        pyo3::exceptions::PyValueError::new_err("event type out of range")
    }
    fn im_bereich(n: i64) -> PyResult<u32> {
        // Erst das C-`int`, dann der Bereich - die Reihenfolge ist die
        // der beiden Fehlerarten oben.
        if n < i64::from(i32::MIN) || n > i64::from(i32::MAX) {
            return Err(falscher_typ());
        }
        if !(0..rustgame_display::konstanten::NUMEVENTS as i64).contains(&n) {
            return Err(ausser_bereich());
        }
        Ok(n as u32)
    }

    // 1. Eine ganze Zahl - **vor** dem Iterieren, sonst zerfaellt ein
    //    `int` nicht und ein `str` in seine Zeichen.
    if o.is_instance_of::<PyInt>() {
        // `extract::<i64>()` scheitert bei `2**63` - dann ist es zu
        // gross fuer ein C-`int` und damit derselbe Fall.
        let n = o.extract::<i64>().map_err(|_| falscher_typ())?;
        // **Die Antwort von `tu` zaehlt auch bei einer einzelnen Zahl.**
        // Hier stand `tu(..)?;` und dann `Ok(Some(false))` - damit gab
        // `get_blocked(1)` immer `False`, egal was gesperrt war. Gefunden
        // hat das die neue Probe `Ereignistypen`, eine Stunde nachdem
        // sie geschrieben war.
        return Ok(Some(!tu(im_bereich(n)?)?));
    }
    // 2. Eine Folge im Sinne von `PySequence_Check`.
    if let Ok(folge) = o.downcast::<pyo3::types::PySequence>() {
        for i in 0..folge.len()? {
            let e = folge.get_item(i)?;
            // Eine Gleitzahl wird **abgeschnitten**, nicht gerundet:
            // `pg_IntFromObj` nimmt bei `PyFloat_Check` den Wert
            // direkt. `[-0.5]` gibt also 0 und nicht -1.
            let n = if let Ok(f) = e.downcast::<pyo3::types::PyFloat>() {
                f.extract::<f64>()? as i64
            } else {
                e.extract::<i64>().map_err(|_| falscher_typ())?
            };
            if !tu(im_bereich(n)?)? {
                return Ok(Some(true));
            }
        }
        return Ok(Some(false));
    }
    // 3. Weder Zahl noch Folge.
    Err(PyTypeError::new_err(
        "event type must be numeric or a sequence",
    ))
}

/// Dasselbe, nur fertig gelesen - fuer die Wege, die die ganze Liste
/// brauchen.
pub fn typen_aus(o: Option<&Bound<'_, PyAny>>) -> PyResult<Option<Vec<u32>>> {
    let mut aus = Vec::new();
    match typen_je(o, |t| {
        aus.push(t);
        Ok(true)
    })? {
        None => Ok(None),
        Some(_) => Ok(Some(aus)),
    }
}

// ---------------------------------------------------------------------------
//  Fehler des Kerns -> Python-Ausnahmen
// ---------------------------------------------------------------------------

/// Fehler aus `rustgame_core::image`.
///
/// pygame wirft hier drei verschiedene Ausnahmen, und der Kern sagt in
/// jeder Variante dazu, welche es ist - also wird das hier auch so
/// zugeordnet und nicht alles zu `pygame.error` verflacht.
pub fn bild_fehler(e: rustgame_core::image::Fehler) -> PyErr {
    use rustgame_core::image::Fehler;
    match e {
        Fehler::Datei(io) => PyErr::from(io),
        Fehler::Format(s) => PygameError::new_err(s),
        Fehler::Wert(s) => PyValueError::new_err(s),
    }
}

/// Fehler aus `rustgame_display`. In pygame sind das alle `pygame.error`,
/// und die Texte stimmen dort schon woertlich.
pub fn anzeige_fehler(e: rustgame_display::Fehler) -> PyErr {
    PygameError::new_err(e.to_string())
}

/// Fehler aus `rustgame_font`.
///
/// `OhneBreite` traegt in pygame den Text `Text has zero width` und kommt
/// als `pygame.error`; eine unlesbare Datei ist dort ein `OSError`.
pub fn schrift_fehler(e: rustgame_font::FontError) -> PyErr {
    use rustgame_font::FontError;
    match e {
        FontError::Datei(io) => PyErr::from(io),
        andere => PygameError::new_err(andere.to_string()),
    }
}

/// Zwei Python-Objekte greifen auf dieselbe Flaeche zu.
///
/// Kommt nur vor, wenn ein Programm waehrend eines Blits dieselbe Flaeche
/// noch einmal hereinreicht. pygame sagt dazu `Surfaces must not be locked
/// during blit`; hier wird daraus dieselbe Ausnahme statt einer Panik.
pub fn belegt() -> PyErr {
    PygameError::new_err("Surfaces must not be locked during blit")
}
