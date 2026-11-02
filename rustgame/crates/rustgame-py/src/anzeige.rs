//! anzeige.rs
//! ==========
//! `pygame.display`.
//!
//! ## Warum hier ein `thread_local` steht
//!
//! Gemessen an echtem pygame: `set_mode` gibt bei **jedem** Aufruf
//! dasselbe Objekt zurueck, auch nach einem Groessenwechsel, und
//! `get_surface() is scr` ist wahr. `rustgame_display` bildet das mit
//! einem `Rc` ab - aber ein Rc allein macht noch kein *identisches
//! Python-Objekt*. Deshalb merkt sich diese Datei das einmal erzeugte
//! `Surface` und gibt es wieder heraus, statt jedes Mal eine neue Huelle
//! um denselben Rc zu bauen.
//!
//! Ohne das waere `pygame.display.get_surface() is screen` falsch - und
//! Programme pruefen genau das.
//!
//! ## Treiberwahl
//!
//! Wie im Kern ueber `RUSTGAME_VIDEODRIVER=dummy`; das ist das
//! Gegenstueck zu pygames `SDL_VIDEODRIVER=dummy`. Damit portierter Code
//! nichts umstellen muss, zaehlt hier **auch** `SDL_VIDEODRIVER` - das
//! umgebende Projekt setzt genau die.

use std::cell::RefCell;

use pyo3::prelude::*;

use rustgame_display::display;

use crate::flaeche::PySurface;
use crate::umwandlung::{anzeige_fehler, flagge, rect_aus, PygameError};

thread_local! {
    /// Die eine Anzeigeflaeche als Python-Objekt - siehe Modulkopf.
    static SCHIRM: RefCell<Option<Py<PySurface>>> = const { RefCell::new(None) };
}

/// `pygame.display.init()`.
#[pyfunction]
#[pyo3(name = "init")]
pub fn anzeige_init() -> PyResult<()> {
    // **Auch `display.init()` macht den Zeitgeber auf.** Gemessen geht
    // `time.set_timer` danach, obwohl `pygame.get_init()` weiterhin
    // `False` ist - SDLs Zeitgeber kommt mit dem Videoteilsystem hoch.
    // Siehe [`crate::ZEITGEBER_JE`].
    crate::ZEITGEBER_JE.store(true, std::sync::atomic::Ordering::Relaxed);
    treiber_waehlen();
    display::init().map_err(anzeige_fehler)
}

/// Liest die Treiberwahl aus der Umgebung.
///
/// `RUSTGAME_VIDEODRIVER` wertet der Kern selbst aus; `SDL_VIDEODRIVER`
/// kennt er nicht - die wird hier uebersetzt, damit ein Programm, das
/// heute `SDL_VIDEODRIVER=dummy` setzt, ohne Aenderung kopflos laeuft.
fn treiber_waehlen() {
    if std::env::var("RUSTGAME_VIDEODRIVER").is_ok() {
        return;
    }
    if let Ok(v) = std::env::var("SDL_VIDEODRIVER") {
        if v.eq_ignore_ascii_case("dummy") {
            display::set_treiber(display::Treiber::Dummy);
        }
    }
}

/// `pygame.display.quit()`.
#[pyfunction]
#[pyo3(name = "quit")]
pub fn anzeige_quit() {
    SCHIRM.with(|s| *s.borrow_mut() = None);
    display::quit();
}

/// `pygame.display.get_init()`.
#[pyfunction]
#[pyo3(name = "get_init")]
pub fn anzeige_get_init() -> bool {
    display::get_init()
}

/// `pygame.display.set_mode(size=(0,0), flags=0, depth=0, display=0, vsync=0)`.
///
/// Gibt bei jedem Aufruf **dasselbe** Objekt zurueck - gemessen.
#[pyfunction]
///
/// `size` nimmt **alles mit zwei Zahlen**, auch einen `Vector2` -
/// gemessen: `set_mode(Vector2(40, 30))` gibt in pygame eine 40x30-Flaeche.
/// Mit einem `(i64, i64)`-Beiwert gab es dort
/// `TypeError: 'Vector2' object cannot be converted to 'PyTuple'`.
#[pyo3(signature = (size=None, flags=0, depth=0, display=0, vsync=0))]
#[pyo3(name = "set_mode")]
pub fn anzeige_set_mode(
    py: Python<'_>,
    size: Option<&Bound<'_, PyAny>>,
    flags: i64,
    depth: i32,
    display: i32,
    vsync: i32,
) -> PyResult<Py<PySurface>> {
    let _ = (display, vsync);
    // **Eine fremde Farbtiefe wird gemeldet und verworfen.**
    //
    // Gemessen: `set_mode((8,8), 0, 8)` gibt
    // `DeprecationWarning: The depth argument is deprecated, and is
    // ignored`, `set_mode((8,8), 0, 32)` dagegen **keine** - es warnt also
    // nur, wenn der Wunsch von dem abweicht, was ohnehin herauskommt.
    //
    // Das Verwerfen stand hier schon (`let _ = depth`), das Melden nicht -
    // und damit war es genau die stille Sorte: der Wert wurde angenommen
    // und weggeworfen, ohne dass es jemand erfuhr.
    if depth != 0 && depth != 32 {
        veraltet(py, "The depth argument is deprecated, and is ignored")?;
    }
    treiber_waehlen();
    // Ohne Angabe `(0, 0)` - das heisst in pygame "Bildschirmgroesse".
    // `versatz_aus` nimmt Tupel, Liste und `Vector2` gleichermassen.
    let size: (i64, i64) = match size {
        None => (0, 0),
        Some(o) if o.is_none() => (0, 0),
        Some(o) => {
            let p = crate::umwandlung::versatz_aus_wort(o, "size")?;
            (p.0 as i64, p.1 as i64)
        }
    };
    // Gemessen: eine negative Groesse gibt `pygame.error: Cannot set
    // negative sized display mode`. Deshalb kommt sie als `i64` weiter
    // und nicht als `u32` - sonst waere die Meldung ein `OverflowError`
    // aus dem Argumentleser, und der sagt etwas anderes.
    if size.0 < 0 || size.1 < 0 {
        return Err(PygameError::new_err("Cannot set negative sized display mode"));
    }
    let groesse = (size.0 as u32, size.1 as u32);
    let schirm =
        rustgame_display::display::set_mode(groesse, flagge(flags)).map_err(anzeige_fehler)?;

    SCHIRM.with(|s| {
        let mut s = s.borrow_mut();
        if let Some(vorhanden) = s.as_ref() {
            let gleich = std::sync::Arc::ptr_eq(&vorhanden.bind(py).borrow().inner, &schirm);
            if gleich {
                return Ok(vorhanden.clone_ref(py));
            }
        }
        let neu = Py::new(py, PySurface::aus_geteilt(schirm))?;
        *s = Some(neu.clone_ref(py));
        Ok(neu)
    })
}

/// `pygame.display.get_surface()` - `None` vor dem ersten `set_mode`.
#[pyfunction]
#[pyo3(name = "get_surface")]
pub fn anzeige_get_surface(py: Python<'_>) -> Option<Py<PySurface>> {
    // Vor dem ersten `set_mode` gibt es keine Flaeche - dann auch kein
    // Python-Objekt dazu.
    display::get_surface()?;
    SCHIRM.with(|s| s.borrow().as_ref().map(|x| x.clone_ref(py)))
}

/// `pygame.display.flip()`.
#[pyfunction]
#[pyo3(name = "flip")]
pub fn anzeige_flip() -> PyResult<()> {
    crate::umwandlung::video_pruefen()?;
    display::flip().map_err(anzeige_fehler)
}

/// `pygame.display.update(rects=None)`.
///
/// pygame nimmt dort ein einzelnes Rechteck, eine Liste (auch mit
/// `None`-Eintraegen) oder gar nichts - und **keine** mehreren
/// Einzelargumente; gemessen, das gibt dort einen `ValueError`.
#[pyfunction]
///
/// **Vier Zahlen gehen auch** - gemessen nimmt pygame
/// `display.update(1, 2, 3, 4)` und meint damit dasselbe wie
/// `display.update(Rect(1, 2, 3, 4))`. Hier stand nur ein einziger
/// Beiwert, und der Aufruf gab
/// `TypeError: takes from 0 to 1 positional arguments but 4 were given` -
/// zwei Tests in `display_test.py` hingen daran.
#[pyo3(signature = (*args))]
#[pyo3(name = "update")]
pub fn anzeige_update(args: &Bound<'_, pyo3::types::PyTuple>) -> PyResult<()> {
    crate::umwandlung::video_pruefen()?;
    // **Der Zustand wird vor den Argumenten geprueft** - gemessen:
    // `pygame.display.update(*(["x"] * 17))` ohne `set_mode` gibt
    // `pygame.error: Display mode not set`, nicht den Argumentfehler.
    // pygames C-Stueck fragt zuerst `VIDEO_INIT_CHECK`. Hier kam der
    // Argumentfehler zuerst, und damit sah ein Programm, das vor
    // `set_mode` zeichnet, die falsche Ausnahme. Gefunden hat das die
    // Frage, was jede Funktion bei siebzehn Argumenten sagt.
    if display::get_surface().is_none() {
        return Err(anzeige_fehler(rustgame_display::Fehler::KeinModus));
    }
    // Vier Zahlen sind ein Rechteck; ein Wert ist das gewohnte Argument.
    if args.len() == 4 {
        let mut z = [0i32; 4];
        for (i, a) in args.iter().enumerate() {
            // **`ValueError`, nicht `TypeError`** - gemessen gibt
            // `display.update(100, "asdf", 100, 100)` in pygame einen
            // `ValueError`. Das ist die andere Sorte als bei einem
            // untauglichen Rechteck-Objekt, und `test_update_incorrect_args`
            // prueft ausdruecklich darauf.
            z[i] = a.extract::<f64>().map_err(|_| {
                pyo3::exceptions::PyValueError::new_err(
                    "invalid rectstyle object",
                )
            })? as i32;
        }
        return display::update_rects(&[Some(rustgame_core::rect::Rect::new(
            z[0], z[1], z[2], z[3],
        ))])
        .map_err(anzeige_fehler);
    }
    if args.len() > 1 {
        // **`ValueError`, und der Wortlaut ist pygames.** Hier stand
        // ein `TypeError` mit "sequence of recstyles" - beides falsch:
        // gemessen gibt pygame-ce 2.5.7
        // `ValueError: update requires a rectstyle or an iterable of
        // rectstyles`. Das "recstyles" war ein Tippfehler aus einer
        // aelteren pygame-Fassung, der hier nie nachgemessen wurde.
        return Err(pyo3::exceptions::PyValueError::new_err(
            "update requires a rectstyle or an iterable of rectstyles",
        ));
    }
    let erstes = args.get_item(0).ok();
    let rects = erstes.as_ref();
    let Some(r) = rects else {
        return display::update().map_err(anzeige_fehler);
    };
    if r.is_none() {
        return display::update().map_err(anzeige_fehler);
    }
    // Ein einzelnes Rechteck?
    if let Ok(einzeln) = rect_aus(r) {
        return display::update_bereiche(&[einzeln]).map_err(anzeige_fehler);
    }
    // Eine Folge - entweder von Rechtecken oder **von vier Zahlen**.
    //
    // `update([100, "asdf", 100, 100])` gibt gemessen einen `ValueError`,
    // nicht den `TypeError` eines untauglichen Rechtecks: die Liste sieht
    // aus wie ein Rechteck und zerfaellt erst am zweiten Glied. Genau das
    // prueft `test_update_incorrect_args` in seiner zweiten Haelfte.
    let glieder: Vec<_> = r.try_iter()?.collect::<PyResult<Vec<_>>>()?;
    let sieht_nach_rechteck_aus = glieder.len() == 4
        && glieder
            .iter()
            .any(|e| e.extract::<f64>().is_ok() || e.is_instance_of::<pyo3::types::PyString>());
    if sieht_nach_rechteck_aus {
        let mut z = [0i32; 4];
        for (i, e) in glieder.iter().enumerate() {
            z[i] = e.extract::<f64>().map_err(|_| {
                pyo3::exceptions::PyValueError::new_err("invalid rectstyle object")
            })? as i32;
        }
        return display::update_bereiche(&[rustgame_core::rect::Rect::new(
            z[0], z[1], z[2], z[3],
        )])
        .map_err(anzeige_fehler);
    }
    let mut liste = Vec::new();
    for e in glieder {
        if e.is_none() {
            liste.push(None);
        } else {
            // **`ValueError`, und pygames Satz** - gemessen gibt
            // `display.update("x")` nach `set_mode`
            // `ValueError: update_rects requires a single list of rects`.
            liste.push(Some(rect_aus(&e).map_err(|_| {
                pyo3::exceptions::PyValueError::new_err("update_rects requires a single list of rects")
            })?));
        }
    }
    display::update_rects(&liste).map_err(anzeige_fehler)
}

/// `pygame.display.set_caption(title, icontitle=None)`.
///
/// Gemessen: den zweiten Titel nimmt pygame entgegen und gibt ihn nie
/// wieder heraus - `set_caption("A", "B")` ergibt `('A', 'A')`.
#[pyfunction]
#[pyo3(signature = (title, icontitle=None, /))]
#[pyo3(name = "set_caption")]
pub fn anzeige_set_caption(title: &str, icontitle: Option<&str>) {
    let _ = icontitle;
    display::set_caption(title);
}

/// `pygame.display.get_caption()`.
#[pyfunction]
#[pyo3(name = "get_caption")]
pub fn anzeige_get_caption(py: Python<'_>) -> PyResult<Py<pyo3::types::PyTuple>> {
    use pyo3::types::PyTuple;
    // **Vor dem ersten Fenster ein leeres Tupel** - gemessen, siehe
    // `zustand::Zustand::caption_gesetzt`. Hier stand immer
    // `('pygame window', 'pygame window')`, und das gilt erst, sobald
    // ein Fenster offen ist.
    if display::get_surface().is_none() && !display::caption_gesetzt() {
        return Ok(PyTuple::empty(py).unbind());
    }
    let (a, b) = display::get_caption();
    Ok(PyTuple::new(py, [a, b])?.unbind())
}

/// `pygame.display.set_icon(surface)`.
#[pyfunction]
#[pyo3(name = "set_icon")]
#[pyo3(signature = (surface, /))]
pub fn anzeige_set_icon(surface: &Bound<'_, PyAny>) -> PyResult<()> {
    // pygames eigener Satz; PyO3 sagte
    // `argument 'surface': 'str' object cannot be converted to 'Surface'`.
    let surface: PyRef<'_, PySurface> = surface
        .extract()
        .map_err(|_| pyo3::exceptions::PyTypeError::new_err("Argument to set_icon must be a Surface"))?;
    let surface = &*surface;
    let s = surface.leihen()?;
    display::set_icon(&s).map_err(anzeige_fehler)
}

/// `pygame.display.list_modes(depth=0, flags=FULLSCREEN, display=0)`.
#[pyfunction]
#[pyo3(signature = (depth=0, flags=0, display=0))]
#[pyo3(name = "list_modes")]
pub fn anzeige_list_modes(
    depth: i32,
    flags: i64,
    display: i32,
) -> PyResult<Vec<(u32, u32)>> {
    crate::umwandlung::video_pruefen()?;
    let _ = (depth, flags, display);
    Ok(rustgame_display::display::list_modes())
}

/// `pygame.display.get_window_size()`.
#[pyfunction]
#[pyo3(name = "get_window_size")]
pub fn anzeige_get_window_size() -> PyResult<(u32, u32)> {
    // **Nicht `Display mode not set`.** Gemessen sagt pygame hier
    // `No open window`, und zwar in jedem Zustand - auch nach `init()`
    // ohne `set_mode`. Es fragt das Fenster, nicht den Modus.
    display::get_window_size()
        .map_err(|_| crate::umwandlung::PygameError::new_err("No open window"))
}

/// `pygame.display.get_driver()`.
#[pyfunction]
#[pyo3(name = "get_driver")]
pub fn anzeige_get_driver() -> PyResult<&'static str> {
    display::get_driver().map_err(anzeige_fehler)
}

/// `pygame.display.toggle_fullscreen()`.
///
/// Unter dem fensterlosen Treiber gibt pygame woertlich
/// `That operation is not supported` - hier auch.
#[pyfunction]
#[pyo3(name = "toggle_fullscreen")]
pub fn anzeige_toggle_fullscreen() -> PyResult<()> {
    crate::umwandlung::video_pruefen()?;
    display::toggle_fullscreen().map_err(anzeige_fehler)
}

/// `pygame.display.get_active()` - gibt es ein sichtbares Fenster?
#[pyfunction]
#[pyo3(name = "get_active")]
pub fn anzeige_get_active() -> bool {
    display::get_surface().is_some()
}

/// `pygame.display.Info()`.
///
/// Die Werte kommen aus [`rustgame_display::display::info`] und sind dort
/// an echtem pygame gemessen. Die acht `blit_*`-Felder fuehrt der Kern
/// nicht; unter dem fensterlosen Treiber sind sie alle 0 (gemessen), also
/// stehen sie hier als Null. Siehe Bericht.
/// ## `Info` ist eine **Fabrik**, der Typ heisst `VidInfo`
///
/// Gemessen, und hier stand es andersherum:
///
/// ```text
/// type(pygame.display.Info)   builtin_function_or_method
/// pygame.display.Info()       <class 'pygame.display.VidInfo'>
/// dir(pygame.display)         ...Info...   - **kein** VidInfo
/// ```
///
/// Die Klasse ist also nur ueber `type(Info())` zu erreichen - und genau
/// deshalb hat sie **keine** Probe je gesehen: `api_coverage.alle_klassen`
/// zaehlt auf, was in einem Modul **steht**, nicht was ein Aufruf
/// zurueckgibt. Gefunden hat den Unterschied die Probe `Funktionsnamen`,
/// weil `Info` dort als Klasse gar nicht in der Funktionsliste auftauchte.
///
/// Dasselbe Muster wie bei `joystick.Joystick`: eine Fabrik, die aussieht
/// wie eine Klasse.
#[pyclass(name = "VidInfo", module = "rustgame_py.display")]
pub struct PyVideoInfo(display::Info);

/// `pygame.display.Info()` - die Fabrik, siehe [`PyVideoInfo`].
#[pyfunction]
#[pyo3(name = "Info")]
#[pyo3(signature = (*_args, **_kwargs))]
pub fn anzeige_info(
    py: Python<'_>,
    _args: &Bound<'_, PyAny>,
    _kwargs: Option<&Bound<'_, PyAny>>,
) -> PyResult<Py<PyVideoInfo>> {
    // **Argumente gibt es nicht** - gemessen:
    // `TypeError: pygame.display.Info() takes no arguments (1 given)`.
    //
    // Der erste Entwurf stand hier mit dem Vermerk "nimmt jedes Argument
    // an", und das war eine Messung, die nie eine war: das Messskript
    // druckte `ok`, **ohne das Ergebnis anzusehen**. Ein Messgeraet, das
    // seinen eigenen Erfolg behauptet, ist der Fehler, den dieses Projekt
    // am oeftesten gemacht hat.
    let mut anzahl = _args.len()?;
    if let Some(k) = _kwargs {
        anzahl += k.len()?;
    }
    if anzahl != 0 {
        return Err(pyo3::exceptions::PyTypeError::new_err(format!(
            "rustgame_py.display.Info() takes no arguments ({anzahl} given)"
        )));
    }
    // **Und das Videotor.** Gemessen gibt `display.Info()` ohne
    // `display.init()` den Satz `video system not initialized`; hier kam
    // eine Auskunft ueber einen Bildschirm, den niemand geoeffnet hatte.
    //
    // Erst die Argumente, dann der Zustand - in dieser Reihenfolge
    // prueft pygame auch: `Info(1)` gibt den Argumentfehler, nicht den
    // Zustandsfehler.
    crate::umwandlung::video_pruefen()?;
    Py::new(py, PyVideoInfo(display::info()))
}

#[pymethods]
impl PyVideoInfo {
    /// Schreiben geht nicht - **pygames Wortlaut**.
    ///
    /// `'pygame.display.VidInfo' object has no attribute 'hw' and no
    /// __dict__ for setting new attributes`; PyO3 sagte
    /// `attribute 'hw' ... is not writable`.
    fn __setattr__(&self, name: &str, _wert: &Bound<'_, PyAny>) -> PyResult<()> {
        Err(pyo3::exceptions::PyAttributeError::new_err(format!(
            "'rustgame_py.display.VidInfo' object has no attribute '{name}' and no __dict__ for setting new attributes"
        )))
    }

    /// Jedes Feld - **ueber einen Namen, nicht ueber Beschreiber.**
    ///
    /// ## Warum nicht `#[getter]`
    ///
    /// Hier standen neunzehn `#[getter]`, und sie arbeiteten richtig. Nur
    /// standen sie damit auch in `dir()` - und pygames `VidInfo` zeigt
    /// ihre Felder **nirgends**:
    ///
    /// ```text
    /// dir(type(pygame.display.Info()))   []        - leer
    /// [n for n in dir(pygame.display.Info()) if n[0] != "_"]   []
    /// pygame.display.Info().hw           0         - geht trotzdem
    /// ```
    ///
    /// pygame benutzt den alten Schlitz `tp_getattr`, und der traegt
    /// keinen Namen in das Woerterbuch des Typs. Mit `#[getter]` gibt es
    /// das nicht; mit **einem** `__getattr__` schon: der Doppelstrich wird
    /// von jeder Namensprobe ausgelassen, und die Felder sind nur ueber
    /// ihn zu erreichen - also zeigt `dir()` sie nicht, und `i.hw` geht.
    ///
    /// Gefunden hat den Unterschied die Probe `Instanznamen`, nachdem sie
    /// auch **Rueckgabetypen** vergleicht: `VidInfo` steht in keinem
    /// Modul und war damit fuer jede andere Probe unsichtbar.
    ///
    /// ## Die Namen sind pygames
    ///
    /// Neunzehn, gemessen: `hw`, `wm`, `video_mem`, `bitsize`, `bytesize`,
    /// `masks`, `shifts`, `losses`, `current_w`, `current_h`,
    /// `pixel_format` und die sechs `blit_*`.
    fn __getattr__(&self, py: Python<'_>, name: &str) -> PyResult<Py<PyAny>> {
        let i = &self.0;
        let zahl = |v: i32| -> PyResult<Py<PyAny>> {
            Ok(v.into_pyobject(py)?.into_any().unbind())
        };
        match name {
            "hw" => zahl(i.hw),
            "wm" => zahl(i.wm),
            "video_mem" => zahl(i.video_mem),
            "bitsize" => Ok(i.bitsize.into_pyobject(py)?.into_any().unbind()),
            "bytesize" => Ok(i.bytesize.into_pyobject(py)?.into_any().unbind()),
            "masks" => Ok(i.masks.into_pyobject(py)?.into_any().unbind()),
            "shifts" => Ok(i.shifts.into_pyobject(py)?.into_any().unbind()),
            "losses" => Ok(i.losses.into_pyobject(py)?.into_any().unbind()),
            "current_w" => Ok(i.current_w.into_pyobject(py)?.into_any().unbind()),
            "current_h" => Ok(i.current_h.into_pyobject(py)?.into_any().unbind()),
            "pixel_format" => {
                Ok(i.pixel_format.into_pyobject(py)?.into_any().unbind())
            }
            // Die sechs `blit_*` fuehrt der Kern nicht; unter dem
            // fensterlosen Treiber sind sie alle 0 - gemessen.
            "blit_hw" | "blit_hw_CC" | "blit_hw_A" | "blit_sw"
            | "blit_sw_CC" | "blit_sw_A" => zahl(0),
            // **pygames Wortlaut** fuer alles andere, nicht PyO3s.
            _ => Err(pyo3::exceptions::PyAttributeError::new_err(
                "does not exist in vidinfo",
            )),
        }
    }

    /// Der mehrzeilige Text, den pygame ausgibt - Zeile fuer Zeile
    /// nachgebaut, samt der fehlenden Leerzeichen hinter `masks =` und
    /// `losses =`.
    fn __repr__(&self) -> String {
        let i = &self.0;
        format!(
            "<VideoInfo(hw = {}, wm = {},video_mem = {}\n\
             {ein}blit_hw = 0, blit_hw_CC = 0, blit_hw_A = 0,\n\
             {ein}blit_sw = 0, blit_sw_CC = 0, blit_sw_A = 0,\n\
             {ein}bitsize  = {}, bytesize = {},\n\
             {ein}masks =  (0x{:08x}, 0x{:08x}, 0x{:08x}, 0x{:08x}),\n\
             {ein}shifts = ({}, {}, {}, {}),\n\
             {ein}losses =  ({}, {}, {}, {}),\n\
             {ein}current_w = {}, current_h = {}\n\
             {ein}pixel_format = {})\n>",
            i.hw,
            i.wm,
            i.video_mem,
            i.bitsize,
            i.bytesize,
            i.masks.0,
            i.masks.1,
            i.masks.2,
            i.masks.3,
            i.shifts.0,
            i.shifts.1,
            i.shifts.2,
            i.shifts.3,
            i.losses.0,
            i.losses.1,
            i.losses.2,
            i.losses.3,
            i.current_w,
            i.current_h,
            i.pixel_format,
            ein = "         ",
        )
    }
    fn __str__(&self) -> String {
        self.__repr__()
    }
}

/// Hilfsfunktion fuer `lib.rs`: alles vergessen, was an der Anzeige haengt.
pub fn alles_vergessen() {
    SCHIRM.with(|s| *s.borrow_mut() = None);
}

// ---------------------------------------------------------------------------
//  Die neunzehn, die der Kern schon konnte und die Huelle nicht durchreichte
// ---------------------------------------------------------------------------
//
// Alle Regeln stehen im Kern (`rustgame_display::display`) und sind dort
// an echtem pygame unter `SDL_VIDEODRIVER=dummy` gemessen - samt der
// Faelle, in denen pygame einen **Fehler** gibt statt einer Zahl. Diese
// Datei reicht sie nur durch; eine zweite Fassung der Regeln waere eine
// zweite Gelegenheit, von pygame abzuweichen.

/// `pygame.display.get_allow_screensaver()`.
#[pyfunction]
#[pyo3(name = "get_allow_screensaver")]
pub fn anzeige_get_allow_screensaver() -> PyResult<bool> {
    display::get_allow_screensaver().map_err(anzeige_fehler)
}

/// `pygame.display.set_allow_screensaver(value=True)`.
#[pyfunction]
#[pyo3(signature = (value=None))]
#[pyo3(name = "set_allow_screensaver")]
pub fn anzeige_set_allow_screensaver(value: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
    // **Jeder Wert ist ein Wahrheitswert** - pygame liest den
    // Schalter mit `PyObject_IsTrue`, und das nimmt alles. Mit
    // `bool` verlangte PyO3 einen echten Wahrheitswert und nannte
    // im Fehlertext `'PyBool'`, einen Namen aus Rust.
    let value = match value {
        None => true,
        Some(o) => o.is_truthy()?,
    };
    display::set_allow_screensaver(value).map_err(anzeige_fehler)
}

/// `pygame.display.get_current_refresh_rate()`.
#[pyfunction]
#[pyo3(name = "get_current_refresh_rate")]
pub fn anzeige_get_current_refresh_rate() -> PyResult<u32> {
    display::get_current_refresh_rate().map_err(anzeige_fehler)
}

/// `pygame.display.get_desktop_refresh_rates()`.
#[pyfunction]
#[pyo3(name = "get_desktop_refresh_rates")]
pub fn anzeige_get_desktop_refresh_rates() -> PyResult<Vec<u32>> {
    display::get_desktop_refresh_rates().map_err(anzeige_fehler)
}

/// `pygame.display.get_desktop_sizes()`.
///
/// Antwortet schon **vor** dem ersten `set_mode()` - gemessen: nach
/// blossem `display.init()` kommt `[(1024, 768)]`. Der Kern entscheidet
/// dafuer, welches Fensterstueck gleich geoeffnet wuerde.
#[pyfunction]
#[pyo3(name = "get_desktop_sizes")]
pub fn anzeige_get_desktop_sizes() -> PyResult<Vec<(u32, u32)>> {
    display::get_desktop_sizes().map_err(anzeige_fehler)
}

/// `pygame.display.get_num_displays()`.
#[pyfunction]
#[pyo3(name = "get_num_displays")]
pub fn anzeige_get_num_displays() -> usize {
    display::get_num_displays()
}

/// `pygame.display.get_window_position()`.
#[pyfunction]
#[pyo3(name = "get_window_position")]
pub fn anzeige_get_window_position() -> PyResult<(i32, i32)> {
    display::get_window_position().map_err(anzeige_fehler)
}

/// `pygame.display.set_window_position(position)`.
#[pyfunction]
#[pyo3(name = "set_window_position")]
#[pyo3(signature = (position, /))]
pub fn anzeige_set_window_position(position: &Bound<'_, PyAny>) -> PyResult<()> {
    // `position must be two numbers` ist pygames Satz - PyO3 sagte
    // `argument 'position': 'str' object cannot be converted to
    // 'PyTuple'`, was zudem einen Rust-Typnamen nennt.
    let position = crate::umwandlung::versatz_aus_wort(position, "position")?;
    display::set_window_position(position).map_err(anzeige_fehler)
}

/// `pygame.display.get_wm_info()`.
///
/// Gemessen: unter `SDL_VIDEODRIVER=dummy` ein **leeres** Woerterbuch.
/// Auf anderen Treibern steht dort ein Schluessel `"window"` mit der
/// Fensternummer des Systems.
#[pyfunction]
#[pyo3(name = "get_wm_info")]
pub fn anzeige_get_wm_info(py: Python<'_>) -> PyResult<Py<PyAny>> {
    let eintraege = display::get_wm_info().map_err(anzeige_fehler)?;
    let d = pyo3::types::PyDict::new(py);
    for (k, v) in eintraege {
        d.set_item(k, v)?;
    }
    Ok(d.into_any().unbind())
}

/// `pygame.display.gl_get_attribute(flag)`.
#[pyfunction]
#[pyo3(name = "gl_get_attribute")]
#[pyo3(signature = (flag, /))]
pub fn anzeige_gl_get_attribute(flag: u32) -> PyResult<i32> {
    display::gl_get_attribute(flag).map_err(anzeige_fehler)
}

/// `pygame.display.gl_set_attribute(flag, value)`.
#[pyfunction]
#[pyo3(name = "gl_set_attribute")]
#[pyo3(signature = (flag, value, /))]
pub fn anzeige_gl_set_attribute(flag: u32, value: i32) -> PyResult<()> {
    display::gl_set_attribute(flag, value).map_err(anzeige_fehler)
}

/// `pygame.display.iconify()`.
#[pyfunction]
#[pyo3(name = "iconify")]
pub fn anzeige_iconify() -> PyResult<bool> {
    display::iconify().map_err(anzeige_fehler)
}

/// `pygame.display.is_fullscreen()`.
#[pyfunction]
#[pyo3(name = "is_fullscreen")]
pub fn anzeige_is_fullscreen() -> PyResult<bool> {
    display::is_fullscreen().map_err(anzeige_fehler)
}

/// `pygame.display.is_vsync()`.
#[pyfunction]
#[pyo3(name = "is_vsync")]
pub fn anzeige_is_vsync() -> PyResult<bool> {
    display::is_vsync().map_err(anzeige_fehler)
}

/// `pygame.display.mode_ok(size, flags=0, depth=0, display=0)`.
///
/// Gibt die passende Farbtiefe zurueck oder `0`, wenn die Art nicht
/// geht. Warum das **24** ist, obwohl `Info.bitsize` 32 meldet, steht
/// beim Kern - die beiden Zahlen bedeuten Verschiedenes.
#[pyfunction]
#[pyo3(name = "mode_ok", signature = (size, flags=0, depth=0, display=0))]
pub fn anzeige_mode_ok(
    size: (i32, i32),
    flags: i64,
    depth: u32,
    display: i32,
) -> PyResult<u32> {
    // Es gibt nur einen Bildschirm, also ist die Nummer ohne Wirkung -
    // sie wird angenommen, damit ein Aufruf mit vier Werten nicht
    // scheitert.
    let _ = display;
    rustgame_display::display::mode_ok(size, flagge(flags), depth).map_err(anzeige_fehler)
}

/// `pygame.display.message_box(...)`.
///
/// **Gibt es hier nicht** - und das ist die gemessene Wahrheit, nicht
/// eine Bequemlichkeit: ein Software-Renderer ohne Fenstersystem hat
/// kein Meldungsfenster. Der Kern liefert dafuer denselben Wortlaut, den
/// echtes pygame ohne Meldungssystem gibt.
#[pyfunction]
#[pyo3(signature = (
    title, message=None, message_type="info", parent_window=None,
    buttons=None, return_button=0, escape_button=None
))]
#[allow(clippy::too_many_arguments)]
#[pyo3(name = "message_box")]
pub fn anzeige_message_box(
    title: &str,
    message: Option<&str>,
    message_type: &str,
    parent_window: Option<&Bound<'_, PyAny>>,
    buttons: Option<Vec<String>>,
    return_button: i64,
    escape_button: Option<i64>,
) -> PyResult<usize> {
    // **Erst pruefen, dann melden, dass es keines gibt.**
    //
    // `display_test.MessageBoxTest` faehrt elf falsche Aufrufe durch und
    // erwartet zu jedem einen bestimmten Fehler - `IndexError` fuer eine
    // Knopfnummer daneben, `TypeError` fuer eine leere Knopfliste oder
    // ein Fenster, das keines ist, `ValueError` fuer eine unbekannte Art.
    // Wer zuerst "kein Meldungssystem" sagt, beantwortet eine andere
    // Frage als die gestellte; die Pruefungen gelten unabhaengig davon,
    // ob am Ende ein Fenster aufgeht.
    if let Some(f) = parent_window {
        if !f.is_none() && f.downcast::<crate::fenster::PyWindow>().is_err() {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "'parent_window' must be a Window object",
            ));
        }
    }
    if let Some(b) = &buttons {
        if b.is_empty() {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "'buttons' should be a sequence of at least one string",
            ));
        }
    }
    // Ohne eigene Liste gibt es genau einen Knopf ("OK"), also ist 0 die
    // einzige gueltige Nummer - **und -1**, denn von hinten gezaehlt ist
    // das derselbe Knopf.
    let anzahl = buttons.as_ref().map(|b| b.len()).unwrap_or(1) as i64;
    let pruefe = |v: i64, wie: &str| -> PyResult<()> {
        if v >= anzahl || v < -anzahl {
            return Err(pyo3::exceptions::PyIndexError::new_err(format!(
                "'{wie}' index out of range"
            )));
        }
        Ok(())
    };
    pruefe(return_button, "return_button")?;
    if let Some(e) = escape_button {
        pruefe(e, "escape_button")?;
    }
    let art = match message_type {
        "info" => display::Meldungsart::Info,
        "warn" => display::Meldungsart::Warnung,
        "error" => display::Meldungsart::Fehler,
        // Wortlaut gemessen, nicht erfunden - samt der einfachen
        // Anfuehrungszeichen um die drei erlaubten Woerter.
        andere => {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "type should be 'info', 'warn' or 'error', got '{andere}'"
            )))
        }
    };
    let knoepfe = buttons.unwrap_or_else(|| vec!["OK".to_string()]);
    let als_str: Vec<&str> = knoepfe.iter().map(|s| s.as_str()).collect();
    display::message_box(title, message, art, &als_str).map_err(anzeige_fehler)
}

/// `pygame.display.set_gamma(red, green=..., blue=...)`.
///
/// Gibt `False` zurueck - gemessen tut pygame hier seit SDL2 dasselbe,
/// die Einstellung ist ersatzlos entfallen.
#[pyfunction]
#[pyo3(signature = (red, green=None, blue=None, /))]
#[pyo3(name = "set_gamma")]
pub fn anzeige_set_gamma(
    py: Python<'_>,
    red: f32,
    green: Option<f32>,
    blue: Option<f32>,
) -> PyResult<bool> {
    veraltet(py, "pygame.display.set_gamma deprecated since 2.1.4")?;
    // Gemessen: ein einzelner Wert gilt fuer alle drei Kanaele.
    Ok(display::set_gamma(
        red,
        green.unwrap_or(red),
        blue.unwrap_or(red),
    ))
}

/// Meldet eine `DeprecationWarning` mit pygames Wortlaut.
///
/// pygame warnt an mehreren Stellen, und seine Tests pruefen das mit
/// `assertWarns(DeprecationWarning)`. Eine Funktion, die stillschweigend
/// arbeitet, faellt dort durch - und sie verschweigt dem Programm auch
/// etwas, das es wissen sollte.
fn veraltet(py: Python<'_>, text: &str) -> PyResult<()> {
    let warnungen = py.import("warnings")?;
    let art = py.get_type::<pyo3::exceptions::PyDeprecationWarning>();
    warnungen.call_method1("warn", (text, art))?;
    Ok(())
}

/// `pygame.display.set_gamma_ramp(red, green, blue)` - siehe
/// [`anzeige_set_gamma`].
#[pyfunction]
#[pyo3(name = "set_gamma_ramp")]
#[pyo3(signature = (red, green, blue, /))]
pub fn anzeige_set_gamma_ramp(
    py: Python<'_>,
    red: Vec<u16>,
    green: Vec<u16>,
    blue: Vec<u16>,
) -> PyResult<bool> {
    veraltet(py, "pygame.display.set_gamma_ramp deprecated since 2.1.4")?;
    let hol = |v: Vec<u16>| -> PyResult<[u16; 256]> {
        v.try_into().map_err(|_| {
            pyo3::exceptions::PyValueError::new_err("gamma ramp must be 256 elements long")
        })
    };
    Ok(display::set_gamma_ramp(
        &hol(red)?,
        &hol(green)?,
        &hol(blue)?,
    ))
}

/// `pygame.display.set_palette(palette)`.
///
/// rustgame rechnet ausschliesslich in ARGB8888 - es gibt keine
/// Palettenflaechen, also auch nichts zu setzen. Der Kern sagt, was
/// pygame sagt.
#[pyfunction]
#[pyo3(signature = (palette=None, /))]
#[pyo3(name = "set_palette")]
pub fn anzeige_set_palette(palette: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
    // **Der Zustand zuerst.** Gemessen gibt `set_palette("x")` in pygame
    // `pygame.error: Display mode is not colormapped` - der Bildschirm
    // hat keine Farbtafel, also kommt es auf die Sorte des Arguments
    // nicht mehr an. PyO3 sagte vorher `argument 'palette': Can't
    // extract `str` to `Vec`` und nannte damit zwei Rust-Namen.
    let liste: Vec<(u8, u8, u8)> = match palette {
        None => Vec::new(),
        Some(o) if o.is_none() => Vec::new(),
        Some(o) => o.extract().unwrap_or_default(),
    };
    display::set_palette(&liste).map_err(anzeige_fehler)
}
