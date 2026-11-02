//! rustgame-py
//! ===========
//! Die **Python-Huelle** um rustgame: `import rustgame_py as pygame`.
//!
//! Ziel ist, dass ein pygame-Programm ohne Aenderung laeuft. Deshalb
//! heissen hier alle Dinge wie in pygame - `Surface`, `Rect`,
//! `draw.rect(...)`, `K_SPACE` - und nicht wie im Rust-Kern.
//!
//! ## Was hier steht und was nicht
//!
//! | Datei | pygame-Gegenstueck |
//! |---|---|
//! | [`flaeche`] | `pygame.Surface` |
//! | [`rechteck`] | `pygame.Rect` |
//! | [`farbe`] | `pygame.Color` |
//! | [`zeichnen`] | `pygame.draw` |
//! | [`umformen`] | `pygame.transform` |
//! | [`bild`] | `pygame.image` |
//! | [`zeit`] | `pygame.time` samt `Clock` |
//! | [`anzeige`] | `pygame.display` |
//! | [`ereignis`] | `pygame.event` |
//! | [`taste`] | `pygame.key` |
//! | [`maus`] | `pygame.mouse` |
//! | [`text`] | `pygame.font` |
//! | [`vektor`] | `pygame.math` - `Vector2`, `Vector3`, die freien Funktionen |
//! | [`maske`] | `pygame.mask` |
//! | [`zeiger`] | `pygame.cursors` und die drei Zeiger-Aufrufe von `pygame.mouse` |
//! | [`ton`] | `pygame.mixer` und `pygame.mixer_music` |
//! | [`figur`] | `pygame.sprite` |
//! | [`hebel`] | `pygame.joystick` |
//! | [`ablage`] | `pygame.scrap` |
//! | [`rechner`] | `pygame.system` |
//! | [`fassung`] | `pygame.version` |
//! | [`konstanten`] | die 589 Konstanten und `pygame.locals` |
//! | [`umwandlung`] | die Grenzstelle: Farben, Rechtecke, Ausnahmen |
//!
//! Hier stand jahrelang: "`pygame.mixer` fehlt: die Kiste
//! `rustgame-mixer` uebersetzt derzeit nicht." Das stimmte einmal.
//! Inzwischen ist die Kiste gruen (81 Tests) und die Huelle
//! vollstaendig; der Vermerk aber blieb stehen und hat die Luecke
//! gedeckt, statt sie zu zeigen.
//!
//! **Die Tabelle oben ist dieselbe Falle** gewesen: sie war
//! unvollstaendig, weil niemand sie fortgeschrieben hat. Wer hier ein
//! Modul anlegt, traegt es ein.
//!
//! Die Rechnung selbst steht **nirgends hier** - die kommt aus
//! `rustgame-core`, `rustgame-display`, `rustgame-font` und
//! `rustgame-math`. Diese Kiste
//! uebersetzt nur, und ihre ganze Kunst besteht darin, dabei nichts zu
//! verlieren.
//!
//! ## Wie geprueft wird
//!
//! Nicht "es uebersetzt", sondern "es rechnet wie pygame":
//! `rustgame/tools/vergleich_py.py` beschreibt eine Szene **einmal** und
//! gibt sie beiden Engines mit ihrem eigenen Modul. Stand: 0 von 9600
//! Pixelbytes unterschiedlich.
//!
//! ## Zwei Fallen beim Bauen
//!
//! **`#[pymethods]` nimmt keine Makro-Aufrufe als Einzelposten.** Wer
//! wiederkehrende Methoden erzeugen will, muss den **ganzen** Block
//! erzeugen - siehe `rect_methoden!` in [`rechteck`].
//!
//! **`#[pyo3(signature = (*args))]` geht in einem erzeugten Block
//! nicht** (Makro-Hygiene: PyO3 findet den erzeugten Namen nicht
//! wieder). Benannte, voreingestellte Angaben leisten dasselbe.

// **`unsafe` nur, wo es ausdruecklich erlaubt ist.**
//
// Im README stand "`unsafe` in genau einer Datei", und das war falsch -
// nachgezaehlt liegt es in **sechs**: `puffer.rs` tut die Arbeit (25
// Stellen), und fuenf weitere Dateien haben je vier, naemlich die beiden
// Schlitze `__getbuffer__`/`__releasebuffer__` samt ihren zwei Aufrufen
// nach `puffer.rs`. PyO3 schreibt diese Schlitze als `unsafe fn` vor;
// daran ist nichts zu machen, und sie tun selbst nichts.
//
// Richtig ist also: **das `unsafe` dieses Projekts steht in `puffer.rs`,
// und fuenf Dateien reichen es durch.** Damit das nicht wieder verrutscht,
// ist der Lint hier scharf und wird einzeln ausgenommen - wer eine
// siebte Stelle anlegt, muss sie ausdruecklich erlauben.
#![deny(unsafe_code)]
#![deny(missing_docs)]

use pyo3::prelude::*;
use pyo3::types::PyCFunction;

pub mod ablage;
pub mod anzeige;
pub mod bild;
pub mod ereignis;
pub mod farbe;
pub mod fassung;
pub mod fenster;
pub mod figur;
pub mod flaeche;
pub mod frechteck;
pub mod freitext;
pub mod geometrie;
pub mod gfxzeichnen;
pub mod hebel;
pub mod konstanten;
mod konstanten_liste;
pub mod maske;
pub mod maus;
pub mod kamera;
pub mod midigeraet;
pub mod moduldoku;
pub mod neuerpuffer;
pub mod pixelfeld;
pub mod pixelkopie;
pub mod puffer;
pub mod pufferzeiger;
pub mod rechner;
pub mod rechteck;
pub mod taste;
pub mod text;
pub mod ton;
pub mod umformen;
pub mod umwandlung;
pub mod vektor;
pub mod zeichnen;
pub mod zeiger;
mod zeiger_daten;
pub mod zeit;

use farbe::PyColor;
use flaeche::PySurface;
use rechteck::PyRect;
use umwandlung::PygameError;

// ---------------------------------------------------------------------------
//  Das ganze Paket hoch- und herunterfahren
// ---------------------------------------------------------------------------

/// `pygame.init()`.
///
/// Gibt `(bestanden, gescheitert)` zurueck. Echtes pygame-ce 2.5.7 meldet
/// dort `(5, 0)` - gemessen. rustgame faehrt weniger Teilsysteme hoch
/// (Anzeige, Schrift, Zeit), meldet aber dieselbe Zahl: Programme pruefen
/// `if gescheitert:` und sollen nicht wegen einer anderen Buchfuehrung in
/// einen Fehlerzweig laufen.
///
/// **Der Schriftordner wird hier gesichert.** `pygame.init()` macht
/// `font.get_init()` wahr, also darf danach `Font(None, 20)` kommen -
/// und das suchte `freesansbold.ttf` am Arbeitsverzeichnis entlang, weil
/// nur `font.init()` und `freetype.init()` den Ordner der Huelle ansagten.
/// Gemessen aus `%TEMP%` gestartet: `error: freesansbold.ttf not found`.
/// Im Projektordner fiel es nicht auf - dort fand die Suche **pygames**
/// Datei in `.venv`.
#[pyfunction]
fn init(py: Python<'_>) -> (u32, u32) {
    schriftordner_sichern(py);
    BASIS.store(true, std::sync::atomic::Ordering::Relaxed);
    ZEITGEBER_JE.store(true, std::sync::atomic::Ordering::Relaxed);
    // **Die beiden Schriftschalter gehoeren dazu.** Gemessen ist nach
    // `pygame.init()` sowohl `font.get_init()` als auch
    // `freetype.get_init()` wahr, und `freetype.get_cache_size()` ist
    // dann 64 - pygames Vorgabe.
    text::FONT_INIT.store(true, std::sync::atomic::Ordering::Relaxed);
    freitext::ft_init_schalter(true);
    rustgame_core::time::init();
    let _ = rustgame_display::display::init();
    rustgame_font::init();
    // Gemessen: `pygame.init()` faehrt das Hebel-System mit hoch -
    // `joystick.get_init()` ist danach `True`, ohne dass jemand
    // `joystick.init()` gerufen haette.
    rustgame_display::joystick::init();
    // `Channel.set_endevent` und `music.set_endevent` merken sich ihre
    // Ereignisnummer in `rustgame-mixer` und legen sie dort ab. Ohne
    // diese Naht blieben sie dort liegen: gesetzt, gelesen, aber nie in
    // `pygame.event.get()`. Die Kiste haengt bewusst **nicht** an
    // `rustgame-display` (Ton braucht kein Fenster), also wird der
    // Rueckweg hier gestellt.
    rustgame_mixer::ereignis::set_senke(|e| {
        use rustgame_display::event::{Daten, Event};
        // Gemessen und in `rustgame_mixer::ereignis` festgehalten:
        // pygame legt das Ereignis **gar nicht** ab, wenn das
        // Video-Teilsystem nicht laeuft. Die Tonkiste weiss davon
        // nichts, also wird hier verworfen.
        if !rustgame_display::display::get_init() {
            return;
        }
        // Das `code`-Feld gibt es nur bei Benutzerereignissen - pygame
        // prueft das in `_pg_push_mixer_event`. Bei der Musik ist es
        // ohnehin `None`.
        let ereignis = match e.code {
            Some(kanal) => Event::mit(e.typ, Daten::Nutzer { code: kanal as i32 }),
            None => Event::neu(e.typ),
        };
        rustgame_display::event::post(ereignis);
    });
    // **Die Tonkiste gehoert dazu.** Gemessen: nach `pygame.init()` ist
    // `pygame.mixer.get_init()` ein Tripel und nicht `None`;
    // `base_test.init_assertions` prueft genau das, sobald
    // `pygame.mixer` geladen ist. Ein Fehlschlag ist hier kein Fehler -
    // pygame zaehlt ihn und macht weiter, deshalb steht die Zahl unten
    // unveraendert.
    let _ = rustgame_mixer::mixer::init(Default::default());
    tongeraete_melden();
    (5, 0)
}

/// Legt je Tongeraet ein `AUDIODEVICEADDED` in die Ereignisschlange.
///
/// Gemessen legt pygame sechs davon ab - fuenf Ausgabe-, ein
/// Aufnahmegeraet -, und zwar **vor** allen Fensterereignissen. Es tut das
/// bei **jedem** Hochfahren, also auch bei `mixer.init()` allein und bei
/// einem zweiten `pygame.init()`:
///
/// ```text
/// 1. init           sechs Ereignisse
/// mixer.quit        -
/// mixer.init        sechs Ereignisse
/// quit + init       sechs Ereignisse
/// ```
///
/// Hier fehlten sie ganz, und das fiel erst bei der **Endabnahme des
/// Plans** auf: dort steht, das Spielprogramm solle ueber dreihundert
/// Bilder verglichen werden, und gefahren wurden dreissig. Ueber dreissig
/// Bilder waren die Ereignisse noch nicht alle abgeholt; ueber dreihundert
/// schon, und dann fehlten sechs.
///
/// Die Reihenfolge ist pygames: Ausgabe zuerst, Nummern **je Art** von
/// null an - das Mikrofon ist `which=0, iscapture=1`, nicht `which=5`.
pub(crate) fn tongeraete_melden() {
    use rustgame_display::event::{Daten, Event};
    for (iscapture, eingabe) in [(0, false), (1, true)] {
        let anzahl = rustgame_mixer::geraete::geraetenamen(eingabe).len();
        for which in 0..anzahl {
            rustgame_display::event::post(Event::mit(
                rustgame_display::konstanten::AUDIODEVICEADDED,
                Daten::Tongeraet {
                    which: which as i32,
                    iscapture,
                },
            ));
        }
    }
}

/// `pygame.quit()`.
///
/// Ruft zuerst, was mit [`register_quit`] angemeldet wurde - **in
/// umgekehrter Reihenfolge**, wie SDLs `atexit`. Erst danach gehen die
/// Teilsysteme zu.
#[pyfunction]
fn quit(py: Python<'_>) {
    BASIS.store(false, std::sync::atomic::Ordering::Relaxed);
    // Gemessen: nach `pygame.quit()` sind **beide** Schriftschalter
    // falsch und der Glyphenspeicher meldet wieder 0.
    text::FONT_INIT.store(false, std::sync::atomic::Ordering::Relaxed);
    freitext::ft_init_schalter(false);
    let liste: Vec<Py<PyAny>> = BEIM_ENDE.with(|b| b.borrow_mut().drain(..).collect());
    for f in liste.into_iter().rev() {
        // Ein Fehler in einer Abmeldefunktion darf `quit()` nicht
        // aufhalten - pygame verschluckt ihn ebenso.
        let _ = f.call0(py);
    }
    anzeige::alles_vergessen();
    rustgame_mixer::mixer::quit();
    rustgame_display::joystick::quit();
    rustgame_display::display::quit();
    rustgame_font::quit();
    rustgame_core::time::quit();
}

thread_local! {
    /// Was bei `pygame.quit()` noch gerufen werden soll.
    static BEIM_ENDE: std::cell::RefCell<Vec<Py<PyAny>>> =
        const { std::cell::RefCell::new(Vec::new()) };
    /// SDLs letzte Fehlermeldung.
    static FEHLERTEXT: std::cell::RefCell<String> =
        const { std::cell::RefCell::new(String::new()) };
}

/// `pygame.register_quit(func)`.
///
/// Gemessen: die Funktion wird bei **jedem** `quit()` gerufen, aber nur
/// einmal - danach ist die Anmeldung verbraucht. `base_test` zaehlt mit
/// und verlangt genau eine Zaehlung nach `init(); register_quit(h);
/// quit()`.
#[pyfunction]
#[pyo3(signature = (callable, /))]
fn register_quit(callable: Py<PyAny>) {
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let func = callable;
    BEIM_ENDE.with(|b| b.borrow_mut().push(func));
}

/// `pygame.get_error()` - SDLs letzte Meldung.
///
/// Bei rustgame gibt es kein SDL, und damit auch keinen Fehlertext, den
/// irgendwer von aussen fuellt. Was hier steht, hat [`set_error`]
/// hineingeschrieben - mehr verspricht pygames eigene Doku auch nicht
/// ("SDL maintains an internal error message ... you will rarely need to
/// call this function").
#[pyfunction]
fn get_error() -> String {
    FEHLERTEXT.with(|f| f.borrow().clone())
}

/// `pygame.set_error(text)`.
#[pyfunction]
#[pyo3(signature = (error_msg, /))]
fn set_error(error_msg: &str) {
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let text = error_msg;
    FEHLERTEXT.with(|f| *f.borrow_mut() = text.to_string());
}

/// Faehrt die Tonkiste beim Beenden des Programms herunter.
///
/// **Sonst gibt es beim Verlassen eine Panik.** `pygame.init()` startet
/// seit kurzem auch den Ton, und rodios Abspielfaden ueberlebt das Ende
/// des Deuters nicht sauber: Python raeumt seine Kisten ab, waehrend der
/// Faden noch laeuft, und Rust meldet `threads should not terminate
/// unexpectedly`. Ein Programm, das `pygame.quit()` ruft, merkt davon
/// nichts - aber viele rufen es nicht, und ein Rust-Fehler beim Beenden
/// ist das Letzte, was jemand von einer Spielekiste lesen will.
#[pyfunction]
fn beim_beenden() {
    rustgame_mixer::mixer::quit();
}

/// Lief `pygame.init()`? - der **Basisschalter**.
///
/// Hier stand `rustgame_core::time::get_init()`, und das war etwas
/// anderes: die Zeitkiste faehrt beim ersten Zugriff von selbst hoch,
/// also sagte `pygame.get_init()` **`True`, bevor jemand `init()`
/// gerufen hatte** - gemessen gegen pygames `False`. Nach `quit()`
/// dasselbe. Und `display.init()` allein macht ihn **nicht** wahr:
/// gemessen ist `get_init()` nach `pygame.display.init()` weiterhin
/// `False`.
///
/// Darum ein eigener Schalter, den nur `init()` setzt und nur `quit()`
/// loescht.
static BASIS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Lief `pygame.init()` und noch kein `quit()`?
pub(crate) fn basis_laeuft() -> bool {
    BASIS.load(std::sync::atomic::Ordering::Relaxed)
}

/// Hat SDLs **Zeitgeber** je gelaufen? - eine Sperre, die nur aufgeht.
///
/// `time.set_timer` ist die einzige gemessene Stelle mit dem Satz
/// `pygame is not initialized`, und sie verhaelt sich anders als alle
/// anderen Tore:
///
/// ```text
/// frisch eingefuehrt    pygame.error: pygame is not initialized
/// nach display.init()   geht
/// nach pygame.init()    geht
/// nach pygame.quit()    geht **weiter**
/// ```
///
/// SDLs Zeitgeber bleibt also oben, wenn das Videoteilsystem
/// herunterfaehrt. Darum ein eigener Schalter, der nie wieder
/// geloescht wird - und nicht `basis_laeuft()`, das nach `quit()`
/// falsch ist.
pub(crate) static ZEITGEBER_JE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Hat der Zeitgeber je gelaufen? Siehe [`ZEITGEBER_JE`].
pub(crate) fn zeitgeber_je() -> bool {
    ZEITGEBER_JE.load(std::sync::atomic::Ordering::Relaxed)
}

/// `pygame.get_init()`.
#[pyfunction]
fn get_init() -> bool {
    basis_laeuft()
}

// ---------------------------------------------------------------------------
//  Das Modul
// ---------------------------------------------------------------------------

/// `pygame.get_array_interface(obj)` - die Feld-Schnittstelle als Woerterbuch.
///
/// Der Umweg ist der Witz: die Funktion liest **`__array_struct__`**, also
/// den C-Kasten, und schreibt daraus ein Woerterbuch. Sie ist damit nicht
/// dasselbe wie `obj.__array_interface__`, auch wenn beide gleich
/// aussehen - ein Objekt, das nur den Kasten hat, geht hier durch und dort
/// nicht.
///
/// `bufferproxy_test.test___array_struct___property` benutzt genau das, um
/// den Kasten zu pruefen, ohne ihn in C auseinandernehmen zu muessen.
#[pyfunction]
#[pyo3(signature = (arg, /))]
fn get_array_interface<'p>(arg: &Bound<'p, PyAny>) -> PyResult<Bound<'p, pyo3::types::PyDict>> {
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let o = arg;
    use pyo3::types::{PyDict, PyTuple};
    let py = o.py();
    let s = puffer::kapsel_sicht(o)?;
    let aus = PyDict::new(py);
    aus.set_item("version", 3)?;
    aus.set_item("typestr", s.typestr())?;
    aus.set_item("shape", PyTuple::new(py, s.shape.iter().map(|v| *v as usize))?)?;
    aus.set_item(
        "strides",
        PyTuple::new(py, s.strides.iter().copied())?,
    )?;
    aus.set_item("data", (s.adresse, !s.schreibbar))?;
    Ok(aus)
}

/// Die Kapsel, die in pygame die C-Funktionstafel eines Moduls traegt.
///
/// **Und hier traegt sie nichts.** rustgame hat kein C-API - es gibt keine
/// Funktionstafel, die ein fremdes Erweiterungsmodul benutzen koennte, und
/// eine vorzugeben waere schlimmer als sie weglassen. Die Kapsel steht
/// trotzdem da, weil `bufferproxy_test.test_c_api` nur ihren **Typ**
/// vergleicht:
///
/// ```python
/// api = pygame.bufferproxy._PYGAME_C_API
/// api_type = type(pygame.base._PYGAME_C_API)
/// self.assertIsInstance(api, api_type)
/// ```
///
/// Das ist eine ehrliche Antwort auf eine Frage nach der Form. Wer die
/// Tafel wirklich benutzen will, findet in `API.md` den Vermerk, dass es
/// sie nicht gibt.
fn c_api_kapsel<'p>(py: Python<'p>, modul: &str) -> PyResult<Bound<'p, pyo3::types::PyCapsule>> {
    let name = std::ffi::CString::new(format!("rustgame_py.{modul}._PYGAME_C_API"))?;
    pyo3::types::PyCapsule::new(py, 0u8, Some(name))
}

/// Baut ein Untermodul und haengt es so ein, dass `import` es findet.
///
/// Ohne den Eintrag in `sys.modules` scheitert `from rustgame_py.draw
/// import rect` - der Zugriff ueber den Punkt ginge zwar, aber
/// pygame-Code benutzt beides.
fn untermodul<'p>(
    eltern: &Bound<'p, PyModule>,
    name: &str,
    fuellen: impl FnOnce(&Bound<'p, PyModule>) -> PyResult<()>,
) -> PyResult<()> {
    let py = eltern.py();
    // **Der volle Name, nicht der kurze.** `PyModule::new(py, "draw")`
    // legt `__name__ = "draw"` fest, und daran haengt mehr, als man
    // denkt: PyO3 setzt `__module__` jeder Funktion darin auf denselben
    // Wert. Gemessen ist in pygame beides `pygame.draw`, bei uns stand
    // beides auf `draw` - und damit stand in jeder Fehlermeldung und
    // jeder Hilfeausgabe der falsche Ort.
    let voll = format!("rustgame_py.{name}");
    let m = PyModule::new(py, &voll)?;
    fuellen(&m)?;
    py.import("sys")?.getattr("modules")?.set_item(&voll, &m)?;
    // `add_submodule` haengt es unter dem **kurzen** Namen ein - sonst
    // hiesse das Feld `rustgame_py.rustgame_py.draw`.
    eltern.add(name, &m)?;
    Ok(())
}

/// Traegt eine Funktion unter ihrem **pygame**-Namen ein.
///
/// Die Rust-Namen tragen ein Praefix (`anzeige_init`, `schrift_init`),
/// weil in Rust zwei Funktionen gleichen Namens nicht nebeneinander
/// stehen koennen - in Python heissen beide `init`, jede in ihrem Modul.
fn als(m: &Bound<'_, PyModule>, name: &str, f: Bound<'_, PyCFunction>) -> PyResult<()> {
    m.add(name, f)
}

/// Sagt der Schriftkiste **einmal**, in welchem Ordner dieses Modul liegt.
///
/// ## Warum nicht beim Laden des Moduls
///
/// `freesansbold.ttf` liegt mitgeliefert neben der Huelle, und
/// `rustgame_font::default_font_path` sucht sie sonst am
/// **Arbeitsverzeichnis** entlang - das darf sich aendern. Zwei Tests in
/// `ftfont_test.py` fielen darum mal um und mal nicht.
///
/// Der naechstliegende Weg waere `m.filename()` in der Modul-Einrichtung
/// gewesen. **Der geht nicht:** `__file__` setzt die Einfuhrmaschinerie
/// erst, **nachdem** die Einrichtungsfunktion zurueckgekehrt ist. Gemessen:
/// die Suche fiel danach weiterhin auf das Arbeitsverzeichnis zurueck.
///
/// Also wird der Ordner beim **ersten Schriftzugriff** geholt. Da laeuft
/// Python, `__file__` steht, und die `OnceLock` sorgt dafuer, dass es
/// genau einmal passiert.
pub(crate) fn schriftordner_sichern(py: Python<'_>) {
    static GETAN: std::sync::OnceLock<()> = std::sync::OnceLock::new();
    if GETAN.set(()).is_err() {
        return;
    }
    if let Ok(f) = py
        .import("rustgame_py")
        .and_then(|m| m.getattr("__file__"))
        .and_then(|o| o.extract::<String>())
    {
        if let Some(d) = std::path::Path::new(&f).parent() {
            rustgame_font::set_schriftordner(d.to_path_buf());
        }
    }
}

/// Die Namen aller Tongeraete - siehe [`rustgame_mixer::geraete`].
#[pyfunction]
fn tongeraete(eingabe: bool) -> Vec<String> {
    rustgame_mixer::geraete::geraetenamen(eingabe)
}

#[pymodule]
fn rustgame_py(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();


    m.add_class::<PySurface>()?;
    // **Das `__init__` von `Surface` steht in Python** und wird hier
    // angehaengt - siehe `python/surface.py` und `flaeche.rs::new`. Damit
    // faellt die benannte Grenze `Surface() ohne Groesse`.
    {
        let hilfe = PyModule::new(py, "rustgame_py.surface")?;
        let code = std::ffi::CString::new(include_str!("python/surface.py"))?;
        py.run(code.as_c_str(), Some(&hilfe.dict()), None)?;
        py.get_type::<PySurface>()
            .setattr("__init__", hilfe.getattr("__init__")?)?;
    }
    m.add_class::<PyRect>()?;
    m.add_class::<frechteck::PyFRect>()?;
    m.add_class::<PyColor>()?;
    // **Das `__init__` von `Color` steht in Python** - dieselbe Stelle
    // wie bei `Surface` daneben, und aus demselben Grund: `tp_new` legt
    // die schwarze Farbe an, `tp_init` liest die Argumente. Siehe
    // `python/color.py` und den Kopf von `farbe.rs`.
    {
        let hilfe = PyModule::new(py, "rustgame_py.color")?;
        let code = std::ffi::CString::new(include_str!("python/color.py"))?;
        py.run(code.as_c_str(), Some(&hilfe.dict()), None)?;
        py.get_type::<PyColor>()
            .setattr("__init__", hilfe.getattr("__init__")?)?;
    }
    m.add_class::<zeit::PyClock>()?;
    // pygame fuehrt diese Klassen **auch oben**, nicht nur in ihrem
    // Untermodul - gemessen mit `dir(pygame)`. Ein Programm schreibt
    // `pygame.Mask`, `pygame.Font`, `pygame.Sound`; `pygame.mask.Mask`
    // geht zusaetzlich. Gefunden hat die Luecke pygames **eigene**
    // Testsuite: `mask_test.py` scheiterte reihenweise an
    // `module 'rustgame_py' has no attribute 'Mask'`.
    m.add_class::<maske::PyMask>()?;
    // `pygame.Joystick` ist **dasselbe Objekt** wie `joystick.Joystick`,
    // und das ist die **Fabrikfunktion**, nicht die Klasse - gemessen
    // (`<built-in function Joystick>`). `joystick_test.test_Joystick_alias`
    // prueft die Gleichheit; die Klasse selbst heisst `JoystickType`.
    // `pygame.JoystickType` gibt es **nicht**, auch gemessen.
    als(m, "Joystick", wrap_pyfunction!(hebel::hebel_bauen, m)?)?;
    m.add_class::<text::PyFont>()?;
    // **`Circle` und `Line` stehen hier nicht.** Gemessen hat pygame sie
    // nur unter `pygame.geometry` - `hasattr(pygame, "Circle")` ist
    // `False`, auch nach `import pygame.geometry`. Hier standen sie oben,
    // und das waren zwei Namen zuviel.
    //
    // Gefunden hat das keine der bestehenden Messungen: `api_coverage.py`
    // fragt, was pygame **hat**, und meldet Fehlendes - ein **Ueberschuss**
    // am Hauptmodul fiel durch. Jetzt vergleicht die Probe
    // `Hauptmodul Namen` beide `dir()` in beide Richtungen.
    m.add_class::<ereignis::PyEvent>()?;
    m.add_class::<zeiger::PyCursor>()?;
    m.add("Sound", ton::klang_klasse(py)?)?;
    m.add_class::<ton::PyChannel>()?;
    // `pygame.SurfaceType is pygame.Surface` ist wahr - Zweitname, kein
    // zweiter Typ.
    m.add("SurfaceType", m.getattr("Surface")?)?;
    // `pygame.Vector2` gibt es auch ohne den Umweg ueber `pygame.math`.
    m.add_class::<vektor::PyVector2>()?;
    m.add_class::<vektor::PyVector3>()?;
    m.add("error", py.get_type::<PygameError>())?;
    // `pygame.BufferError` - ein eigener Typ, und `bufferproxy_test.test_write`
    // faengt genau ihn.
    m.add("BufferError", py.get_type::<umwandlung::PygameBufferError>())?;

    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_function(wrap_pyfunction!(quit, m)?)?;
    m.add_function(wrap_pyfunction!(get_init, m)?)?;
    m.add_function(wrap_pyfunction!(register_quit, m)?)?;
    // **`__path__` macht aus dem Modul ein Paket.**
    //
    // Ohne dieses Feld bricht `import rustgame_py._sprite` ab, noch bevor
    // irgendein Sucher gefragt wird: Pythons Importer sieht nach, ob der
    // Elternteil ein `__path__` hat, und meldet sonst woertlich
    // `'rustgame_py' is not a package`. Die Liste bleibt leer - gesucht
    // wird nicht auf der Platte, sondern in `sys.modules` und ueber den
    // Sucher in `sys.meta_path`.
    m.add("__path__", pyo3::types::PyList::empty(py))?;
    py.import("atexit")?
        .call_method1("register", (wrap_pyfunction!(beim_beenden, m)?,))?;
    m.add_function(wrap_pyfunction!(get_error, m)?)?;
    m.add_function(wrap_pyfunction!(set_error, m)?)?;

    konstanten::eintragen(m)?;
    // `from pygame.locals import *` ist die gebraeuchlichste Schreibweise
    // ueberhaupt - deshalb stehen dieselben Namen noch einmal dort.
    untermodul(m, "locals", |l| {
        konstanten::eintragen(l)?;
        // `pygame.locals.pygame` - ein mitimportierter Name, siehe die
        // Begruendung bei `cursors`.
        l.setattr("pygame", m)?;
        // **`Rect` und `Color` gehoeren dazu.** Gemessen: `dir` von
        // `pygame.locals` enthaelt beide - `from pygame.locals import *`
        // bringt sie also mit, und pygames eigene Testdateien verlassen
        // sich darauf. `mask_test.py` schreibt an zwei Stellen schlicht
        // `Rect` und scheiterte hier mit `NameError`.
        //
        // Das sah zuerst nach einem Fehler in pygames Test aus - der
        // Name wird dort nirgends eingefuehrt. Er kommt aus dem
        // Sternchen-Import, und das ist der Grund, warum `locals` mehr
        // ist als eine Konstantenliste.
        l.add_class::<PyRect>()?;
        l.add_class::<PyColor>()?;
        Ok(())
    })?;

    // `vernum` und `ver` stehen **auch oben** - gemessen: es gibt
    // `pygame.vernum` zusaetzlich zu `pygame.version.vernum`.
    let mut oben: Option<(Py<PyAny>, String)> = None;
    untermodul(m, "version", |v| {
        oben = Some(fassung::eintragen(v)?);
        Ok(())
    })?;
    if let Some((vernum, ver)) = oben {
        m.add("vernum", vernum)?;
        // Gemessen: `pygame.ver` und `pygame.__version__` sind beide
        // `'2.5.7'`, und `pygame.get_sdl_version()` gibt es ebenfalls
        // **oben**, nicht nur in `pygame.version`. `version_test.py`
        // prueft genau das und scheiterte an
        // `module has no attribute '__version__'`.
        m.add("__version__", ver.clone())?;
        m.add("ver", ver)?;
        m.add_function(wrap_pyfunction!(fassung::fassung_get_sdl_version, m)?)?;
        // Die Bytereihenfolge gehoert daneben - `surface_test.py` rechnet
        // damit Maskenwerte um und holt sie vom **Hauptmodul**, nicht aus
        // `version`.
        m.add_function(wrap_pyfunction!(pufferzeiger::get_sdl_byteorder, m)?)?;
        // **Drei Namen, die es doppelt gibt**: `pygame.SDL` und
        // `pygame.rev` sind dieselben Werte wie in `pygame.version`, nur
        // noch einmal oben. Gemessen, und zwar erst, als das Messgeraet
        // endlich nach den Namen des **Hauptmoduls** gefragt hat - es
        // zaehlte bis dahin jedes Untermodul und jede Klasse auf, aber
        // nie `dir(pygame)` selbst.
        let vmod = m.getattr("version")?;
        m.add("SDL", vmod.getattr("SDL")?)?;
        m.add("rev", vmod.getattr("rev")?)?;
    }

    // `pygame.HAVE_NEWBUF` - **1**, und das ist hier keine Behauptung mehr.
    //
    // pygame setzt die Zahl auf 1, wenn es gegen eine Python-Fassung mit
    // dem neuen Puffer-Protokoll (PEP 3118) gebaut wurde. Das ist seit
    // Python 3.0 immer der Fall, und seit `puffer.rs` da ist, gilt es hier
    // ebenso: `memoryview(s.get_buffer())` geht, `__array_struct__` geht.
    // Vor dem Puffer-Protokoll waere eine 1 gelogen gewesen - deshalb
    // stand die Zahl lange nicht da.
    m.add("HAVE_NEWBUF", 1i32)?;

    // `pygame.pkgdata` - **reines Python**, und deshalb hier als Text.
    //
    // `image_test.py` holt damit an zwei Stellen das Programmsinnbild:
    // `pygame.pkgdata.getResource("pygame_icon.bmp")` gibt ein offenes
    // Dateiobjekt. Das ist keine Grafikarbeit, sondern
    // Paketverwaltung - pygames eigene Fassung ist 60 Zeilen Python, und
    // sie hier nachzubauen hiesse, `importlib.resources` in Rust
    // nachzubauen.
    //
    // Die Datei selbst liegt jetzt **neben dem Modul**, wie pygame sie
    // neben seinem Paket liegen hat - dieselbe Ueberlegung wie bei
    // `freesansbold.ttf`.
    {
        // Als Rohtext (`r#"..."#`), damit die Anfuehrungszeichen und
        // Zeilenumbrueche des Python-Quelltextes unangetastet bleiben.
        let quelle = r#"
import os

# **Vier Namen, nicht einer.** `dir(pygame.pkgdata)` zeigt `files`,
# `getResource`, `resource_exists` und `resource_stream`; hier stand nur
# der zweite. Die drei anderen sind die Namen, die `importlib.resources`
# hergibt, und pygame reicht sie durch - gemessen:
#
#     resource_exists("pygame", "pygame_icon.bmp")   True
#     resource_exists("pygame", "nix")               False
#     resource_stream("pygame", "pygame_icon.bmp")   BufferedReader
#     files("pygame")                                .../site-packages/pygame
# `sys` steht bei pygame in `dir(pygame.pkgdata)`: `pkgdata.py` schreibt
# `import sys` fuer den Sonderweg unter py2exe. Den gibt es hier nicht, den
# Namen schon - `dir()` ist die Schnittstelle.
import sys                                           # noqa: F401

__all__ = ["files", "getResource", "resource_exists", "resource_stream"]

# **Durchgereicht, nicht nachgebaut.** `pkgdata.py` schreibt
# `from importlib.resources import files`, und damit ist pygames `files`
# *dieselbe* Funktion wie die der Standardbibliothek: Signatur
# `(anchor=None)`, `__module__` `importlib.resources._common`. Hier stand
# eine eigene Fassung `files(package)` - sie tat dasselbe, hiess aber
# anders und nannte in jeder Fehlermeldung den falschen Parameternamen.
from importlib.resources import files


def resource_exists(_package_or_requirement, _resource_name):
    """Gibt es diese Datei im Paket?"""
    try:
        return (files(_package_or_requirement) / _resource_name).is_file()
    except (ImportError, OSError, TypeError):
        return False


def resource_stream(_package_or_requirement, _resource_name):
    """Die Datei als offener Datenstrom."""
    return (files(_package_or_requirement) / _resource_name).open("rb")


def getResource(identifier, pkgname=None):
    # Oeffnet eine Datei, die neben dem Modul liegt.
    import rustgame_py

    ordner = os.path.dirname(os.path.abspath(rustgame_py.__file__))
    return open(os.path.join(ordner, identifier), "rb")
"#;
        let m2 = PyModule::from_code(
            py,
            std::ffi::CString::new(quelle)?.as_c_str(),
            std::ffi::CString::new("rustgame_py/pkgdata.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py.pkgdata")?.as_c_str(),
        )?;
        py.import("sys")?
            .getattr("modules")?
            .set_item("rustgame_py.pkgdata", &m2)?;
        m.add("pkgdata", &m2)?;
    }

    // `pygame.rwobject`, `pygame.typing` und `pygame.sysfont` - alle drei
    // **reines Python**, und alle drei aus demselben Grund hier als Text:
    //
    // * `rwobject` sind zwei Funktionen, die nichts tun als Kodieren. Sie
    //   in Rust zu schreiben hiesse, Pythons Kodierer nachzubauen - samt
    //   `unicode_escape`, `backslashreplace` und dem Sonderfall, dass ein
    //   eingebettetes Nullbyte `None` gibt.
    // * `typing` sind sieben Typnamen, sonst nichts.
    // * `sysfont` ist pygames eigene Schriftsuche; was daran zaehlt,
    //   steht als `SysFont`, `get_fonts` und `match_font` in
    //   `pygame.font`, und dieses Modul reicht es unter seinem zweiten
    //   Namen durch.
    //
    // Jede dieser drei Dateien hat in pygames Testsuite eine eigene, und
    // keine davon lief hier je - `rwobject_test.py` allein sind 19 Tests.
    for (name, quelle) in [
        (
            "midi",
            r#"
"""`pygame.midi` - Umrechnungen, Ereignisse **und echte Geraete**.

**Hier stand einmal "ohne MIDI-Geraet".** `Input` und `Output` warfen
einen Fehler, mit der Begruendung, dafuer fehle PortMidi und PortMidi
gaebe es nur ueber FFI. Der erste Teil ist wahr, der zweite ist dieselbe
Verwechslung wie bei `system.get_total_ram`: **`unsafe` braucht
irgendwer, aber nicht dieses Programm.** Die Kiste `midir` spricht unter
Windows mit WinMM, unter Linux mit ALSA und unter macOS mit CoreMIDI.

Was es ausgeloest hat, war eine einzige Probe: `pygame.midi.get_count()`
gibt auf dieser Maschine **2**, und die beiden sind `Microsoft MIDI
Mapper` und `GS Wavetable Synth`. Es gab also Geraete, und pygame
benutzte sie.

Die Rechenseite des Moduls stimmt wie vorher auf den Wert:

    frequency_to_midi(27.5)   ->  21
    midi_to_frequency(21)     ->  27.5
    midi_to_ansi_note(60)     ->  "C4"

`midi_test.py` faehrt genau diese Seite - die acht Tests, die ohne
Geraet laufen, brauchen kein einziges Byte vom Betriebssystem. Sie
scheiterten bisher nicht an einer Luecke, sondern daran, dass das Modul
gar nicht da war.
"""

import atexit                                         # noqa: F401
import math

# **`MIDIIN` und `MIDIOUT` stehen weiter unten** - das `__getattr__` am
# Dateiende holt sie vom Wurzelmodul. Hier standen sie kurzzeitig als
# Zahlen (32771 und 32772, gemessen), und das war falsch: zwei Quellen fuer
# dieselbe Zahl sind eine Quelle zuviel. Gefehlt hat nie der Wert, sondern
# nur der Eintrag in `dir()` - siehe `__dir__`.


def _paket():
    """Das Paket selbst - bei pygame heisst der Name `pygame`.

    `pygame/midi.py` schreibt `import pygame`, und damit steht `pygame` in
    `dir(pygame.midi)`. Hier wird derselbe Name gesetzt, aber **erst beim
    ersten Gebrauch**: dieser Text laeuft waehrend der Initialisierung von
    `rustgame_py`, und ein Import von dort waere ein Ring.
    """
    import rustgame_py

    return rustgame_py


__all__ = [
    "Input",
    "Output",
    "MidiException",
    "MIDIIN",
    "MIDIOUT",
    "init",
    "quit",
    "get_init",
    "get_count",
    "get_device_info",
    "get_default_input_id",
    "get_default_output_id",
    "midis2events",
    "frequency_to_midi",
    "midi_to_frequency",
    "midi_to_ansi_note",
    "time",
]


def __dir__():
    """Auch die drei Namen, die erst beim Zugriff entstehen.

    `MIDIIN`, `MIDIOUT` und `pygame` holt das `__getattr__` am Dateiende;
    im Woerterbuch des Moduls stehen sie nicht, und `dir()` liest genau
    dieses Woerterbuch. Sie fehlten darum in `dir(pygame.midi)` - erreichbar
    waren sie die ganze Zeit.

    **Und das `__getattr__` steht nur einmal.** Ein zweites, weiter oben
    eingefuegtes, hat das untere stillschweigend ersetzt - Python nimmt die
    letzte Zuweisung -, und `getattr(midi, "pygame")` warf danach einen
    `AttributeError`, obwohl `dir()` den Namen zeigte. Gefunden hat das
    `pygame_tests.py`, das beim Umbiegen jeden Namen einmal anfasst.
    """
    return sorted(
        set(__all__)
        | {n for n in globals() if not n.startswith("_")}
        | {"MIDIIN", "MIDIOUT", "pygame"}
    )


def _pg():
    import rustgame_py

    return rustgame_py


class MidiException(Exception):
    """Ein Fehler aus dem MIDI-Teil.

    Traegt sein Argument als `parameter` - so steht es in pygames
    `midi.py`, und `test_MidiException` liest genau dieses Feld.
    """

    def __init__(self, value):
        super().__init__(value)
        self.parameter = value

    def __str__(self):
        return repr(self.parameter)


_gestartet = False


_null = None


def init():
    global _gestartet, _null
    _gestartet = True
    _null = _pg().time.get_ticks()


def quit():                                          # noqa: A001
    global _gestartet
    _gestartet = False


def get_init():
    return _gestartet


def time():                                          # noqa: A001
    """Die Zeit der MIDI-Uhr in Millisekunden.

    In pygame ist das PortMidis eigener Zaehler. Gemessen gibt
    `pygame.midi.time()` direkt nach `init()` eine **0** - der Nullpunkt
    liegt also beim Start des MIDI-Teils und nicht beim Start des
    Programms. Vor `init()` ist es ebenfalls 0, und zwar ohne Fehler:
    pygame ruft hier PortMidi, und das antwortet auch ungefragt.
    """
    if _null is None:
        return 0
    return _pg().time.get_ticks() - _null


def _pruefen():
    if not _gestartet:
        raise RuntimeError("pygame.midi not initialised.")


def get_count():
    """Wie viele MIDI-Geraete das System hat."""
    _pruefen()
    return _pg()._midi_anzahl()


def _geraet(an_id):
    """Die rohe Auskunft - fuer `Input` und `Output` hier im Modul."""
    return _pg()._midi_geraet(an_id)


def _als_c_int(wert):
    """Wie Cython ein `int`-Argument nimmt - `_pypm.GetDeviceInfo(int i)`.

    Gemessen bei pygame: Text, `bytes` und `None` geben `TypeError: an
    integer is required`, eine Kommazahl wird abgeschnitten (`1.5` ist
    Geraet 1), `True` ist 1, und alles jenseits eines C-`long` gibt
    `OverflowError: Python int too large to convert to C long` - unter
    Windows ist das schon `2**31`. Hier stand `an_id < 0` ohne Pruefung,
    und `get_device_info("x")` sagte `'<' not supported between
    instances of 'str' and 'int'`.
    """
    import ctypes

    if isinstance(wert, int):
        zahl = int(wert)
    elif hasattr(type(wert), "__int__") and not isinstance(
            wert, (str, bytes, bytearray)):
        zahl = int(wert)
    else:
        raise TypeError("an integer is required")
    bits = 8 * ctypes.sizeof(ctypes.c_long)
    if not -(1 << (bits - 1)) <= zahl < (1 << (bits - 1)):
        raise OverflowError("Python int too large to convert to C long")
    if not -(1 << 31) <= zahl < (1 << 31):
        raise OverflowError("value too large to convert to int")
    return zahl


def get_device_info(an_id):
    """Auskunft ueber ein Geraet - `None`, wenn es die Nummer nicht gibt.

    **`None` und kein `IndexError`.** Genau das sieht pygames
    Schnittstelle vor, und sein eigener Test prueft es:
    `self.assertIsNone(pygame.midi.get_device_info(out_of_range))`.

    Die Form ist `(schnittstelle, name, eingang, ausgang, offen)`, die
    ersten beiden als `bytes`. Gemessen auf dieser Maschine, wie bei
    pygame mit dem Mapper vorn:
    `0 -> (b'MMSystem', b'Microsoft MIDI Mapper', 0, 1, 0)`,
    `1 -> (b'MMSystem', b'Microsoft GS Wavetable Synth', 0, 1, 0)`.
    """
    _pruefen()
    an_id = _als_c_int(an_id)
    if an_id < 0:
        return None
    return _pg()._midi_geraet(an_id)


def get_default_input_id():
    """Der erste Eingang - `-1`, wenn es keinen gibt."""
    _pruefen()
    return _pg()._midi_erster_eingang()


def get_default_output_id():
    """Der erste Ausgang - `-1`, wenn es keinen gibt."""
    _pruefen()
    return _pg()._midi_erster_ausgang()


# **Derselbe Text fuer Eingang und Ausgang, und er spricht von "Output".**
# Das ist pygames Wortlaut, woertlich, auch in `Input.__init__` - gemessen
# gibt `pygame.midi.Input(-1)` genau diesen Satz. Ihn hier zu glaetten
# hiesse, eine Zeichenkette zu aendern, auf die fremder Code prueft.
_KEINE_VORGABE = (
    "Device id is -1, not a valid output id.  "
    "-1 usually means there were no default "
    "Output devices."
)


def _geraet_pruefen(device_id):
    """Die Auskunft zu einer Nummer - oder die Ausnahme, die pygame wirft.

    Drei Sorten, und alle drei sind gemessen: ein `TypeError` fuer alles,
    was keine ganze Zahl ist, ein `OverflowError` fuer eine zu grosse, und
    ein `MidiException` fuer eine, die es nicht gibt. pygame holt die
    ersten beiden von seinem C-Teil und faengt sie ausdruecklich auf, um
    sie mit eigenem Wortlaut weiterzuwerfen; dasselbe passiert hier.
    """
    try:
        info = _pg().midi._geraet(device_id)
    except OverflowError:
        raise OverflowError("long int too large to convert to int") from None
    except TypeError:
        raise TypeError("an integer is required") from None
    if info is None:
        raise MidiException("Device id invalid, out of range.")
    return info


class Input:
    """Ein MIDI-Eingang.

    pygames Fassung ist eine Huelle um PortMidi; diese ist eine um
    `midir`. Nach aussen ist es dieselbe: `poll()` fragt, ob etwas
    anliegt, `read(n)` nimmt bis zu n Ereignisse heraus, `close()`
    schliesst.

    Ein Ereignis hat die Form `[[status, data1, data2, data3], zeit]` -
    vier Datenbytes, von denen die hinteren null sind, wenn die Nachricht
    kuerzer war. Genau so gibt pygame sie heraus, und `midis2events`
    daneben rechnet damit.
    """

    def __init__(self, device_id, buffer_size=4096):
        _pruefen()
        if device_id == -1:
            raise MidiException(_KEINE_VORGABE)
        info = _geraet_pruefen(device_id)
        # **Die Reihenfolge ist pygames und nicht beliebig**: erst die
        # Richtung, dann der Rest. Wer einen Ausgang als Eingang oeffnet,
        # soll das lesen und nicht "ungueltig".
        if not info[2]:
            raise MidiException(
                "Device id given is not a valid input id, it is an output id."
                if info[3] else
                "Device id given is not a valid input id."
            )
        self.device_id = device_id
        self._eingang = _pg()._MidiEingang(device_id, buffer_size)

    def _offen(self):
        if self._eingang is None or not self._eingang.offen:
            raise MidiException("midi not open.")

    def close(self):
        """Schliesst den Eingang. Ein zweites Mal ist kein Fehler."""
        _pruefen()
        if self._eingang is not None:
            self._eingang.schliessen()

    def poll(self):
        """Liegt etwas an?"""
        _pruefen()
        self._offen()
        return self._eingang.anliegt()

    def read(self, num_events):
        """Nimmt bis zu `num_events` Ereignisse heraus."""
        _pruefen()
        self._offen()
        aus = []
        for roh, zeit in self._eingang.lesen(num_events):
            # **Immer vier Datenbytes.** Eine MIDI-Nachricht ist ein, zwei
            # oder drei Bytes lang; pygame fuellt auf vier auf, und
            # `midis2events` liest genau vier. Wer hier kuerzer liefert,
            # bringt jede Schleife darueber zum Stolpern.
            vier = list(roh[:4]) + [0] * (4 - len(roh[:4]))
            aus.append([vier, zeit])
        return aus


class Output:
    """Ein MIDI-Ausgang - siehe `Input`.

    Die Zaehlarbeit steht hier und nicht im Kern: `note_on` rechnet Kanal
    und Status zusammen, `write` nimmt eine Liste mit Zeitstempeln,
    `pitch_bend` zerlegt einen Wert in zwei Siebenbitgruppen. Genau so
    macht es pygames `midi.py` auch - der Kern sendet nur Bytes.
    """

    def __init__(self, device_id, latency=0, buffer_size=256):
        _pruefen()
        if device_id == -1:
            raise MidiException(_KEINE_VORGABE)
        info = _geraet_pruefen(device_id)
        if not info[3]:
            raise MidiException(
                "Device id given is not a valid output id, it is an input id."
                if info[2] else
                "Device id given is not a valid output id."
            )
        self.device_id = device_id
        self._latency = latency
        self._ausgang = _pg()._MidiAusgang(device_id)

    def _offen(self):
        if self._ausgang is None or not self._ausgang.offen:
            raise MidiException("midi not open.")

    def close(self):
        """Schliesst den Ausgang. Ein zweites Mal ist kein Fehler."""
        _pruefen()
        if self._ausgang is not None:
            self._ausgang.schliessen()

    def abort(self):
        """Bricht Ausstehendes ab.

        **Hier ist nichts ausstehend.** PortMidi haelt einen Puffer mit
        Zeitstempeln, aus dem `abort` den Rest wegwirft; `midir` sendet
        sofort. Es bleibt der Teil, den pygame ebenfalls tut: ein
        `All Sound Off` (Steuernachricht 120) auf allen sechzehn Kanaelen,
        damit nichts haengenbleibt.
        """
        _pruefen()
        self._offen()
        for kanal in range(16):
            self._ausgang.senden(bytes([0xB0 | kanal, 120, 0]))

    def write(self, data):
        """Schreibt eine Liste von Ereignissen.

        Die Form ist `[[[status, data1, data2, data3], zeit], ...]`, und
        die Datenbytes sind **einzeln weglassbar**: pygames Doku sagt
        ausdruecklich, `[[[0xc0,0,0],20000]]` sei dasselbe wie
        `[[[0xc0],20000]]`.

        Mehr als 1024 Eintraege sind ein `IndexError` - auch das steht
        dort und ist uebernommen.
        """
        _pruefen()
        self._offen()
        if len(data) > 1024:
            raise IndexError("maximum list length is 1024")
        for eintrag in data:
            nachricht = eintrag[0] if isinstance(eintrag, (list, tuple)) else eintrag
            bytes_ = [int(b) & 0xFF for b in nachricht]
            if not bytes_:
                continue
            self._ausgang.senden(bytes(bytes_[: _laenge(bytes_[0], len(bytes_))]))

    def write_short(self, status, data1=0, data2=0):
        """Schreibt **eine** kurze Nachricht."""
        _pruefen()
        self._offen()
        roh = [int(status) & 0xFF, int(data1) & 0x7F, int(data2) & 0x7F]
        self._ausgang.senden(bytes(roh[: _laenge(roh[0], 3)]))

    def write_sys_ex(self, when, msg):
        """Schreibt eine Systemnachricht.

        `msg` darf eine Liste von Bytes oder eine Zeichenkette sein -
        pygame nimmt beides. Der Zeitstempel wird hier nicht gebraucht:
        gesendet wird sofort.
        """
        _pruefen()
        self._offen()
        _ = when
        if isinstance(msg, str):
            roh = [ord(c) & 0xFF for c in msg]
        else:
            roh = [int(b) & 0xFF for b in msg]
        self._ausgang.senden(bytes(roh))

    def note_on(self, note, velocity, channel=0):
        """Schlaegt eine Note an."""
        _kanal_pruefen(channel)
        self.write_short(0x90 | channel, note, velocity)

    def note_off(self, note, velocity=None, channel=0):
        """Laesst eine Note los.

        **`velocity=None` ist nicht null, sondern "egal".** pygame setzt
        dort selbst eine Null ein; der Unterschied faellt nur auf, wenn
        jemand `None` durchreicht und einen `TypeError` erwartet.
        """
        _kanal_pruefen(channel)
        self.write_short(0x80 | channel, note, 0 if velocity is None else velocity)

    def set_instrument(self, instrument_id, channel=0):
        """Waehlt ein Instrument - 0 bis 127."""
        _kanal_pruefen(channel)
        if not 0 <= int(instrument_id) <= 127:
            raise ValueError(f"Undefined instrument id: {instrument_id}")
        self.write_short(0xC0 | channel, instrument_id)

    def pitch_bend(self, value=0, channel=0):
        """Verstimmt eine Spur - `-8192` bis `8191`.

        Der Wert wird in **zwei Siebenbitgruppen** zerlegt, die niedrige
        zuerst; so sieht es die MIDI-Norm vor, und pygame rechnet genauso.
        Der Nullpunkt liegt bei 8192.
        """
        _kanal_pruefen(channel)
        if not -8192 <= int(value) <= 8191:
            raise ValueError(
                f"Pitch bend value must be between -8192 and 8191, not {value}."
            )
        v = int(value) + 8192
        self.write_short(0xE0 | channel, v & 0x7F, (v >> 7) & 0x7F)


def _kanal_pruefen(channel):
    """Sechzehn Kanaele, von null an gezaehlt."""
    if not 0 <= int(channel) <= 15:
        raise ValueError("Channel not between 0 and 15.")


def _laenge(status, hoechstens):
    """Wie viele Bytes eine Nachricht mit diesem Status hat.

    MIDI ist nicht selbstbeschreibend genug, um es dem Empfaenger zu
    ueberlassen: `0xC0` (Programmwechsel) und `0xD0` (Kanaldruck) haben
    **ein** Datenbyte, alles andere im Kanalbereich zwei. `midir` nimmt
    nur vollstaendige Nachrichten an und beschwert sich sonst, also wird
    hier gekuerzt.

    Ausserhalb des Kanalbereichs (`0xF0` und darueber) bleibt die Laenge,
    wie sie kam - dort steht eine Systemnachricht, und die weiss selbst,
    wie lang sie ist.
    """
    s = status & 0xF0
    if status >= 0xF0:
        return hoechstens
    if s in (0xC0, 0xD0):
        return min(2, hoechstens)
    return min(3, hoechstens)


def midis2events(midis, device_id):
    """Macht aus MIDI-Ereignissen pygame-Ereignisse.

    Ein Eintrag hat die Form `((status, data1, data2, data3), zeit)` -
    **genau vier** Datenwerte und **genau zwei** Glieder. Alles andere
    gibt einen `ValueError`; gemessen an pygames eigenem Test, der
    fehlende und ueberzaehlige Werte einzeln durchspielt.

    Das Feld heisst `vice_id`, nicht `device_id` - ein Tippfehler in
    pygames `midi.py`, den sein eigener Test festschreibt.
    """
    pg = _pg()
    aus = []
    for midi in midis:
        try:
            ((status, data1, data2, data3), timestamp) = midi
        except ValueError:
            raise ValueError("Unexpected midi event: %s" % (midi,)) from None
        # **`pg.MIDIIN`, nicht `MIDIIN`.** Das Modul-`__getattr__`
        # unten beantwortet nur Zugriffe von aussen; ein freier Name im
        # eigenen Rumpf sucht in den Modulglobalen und dann in den
        # Builtins - und findet nichts.
        e = pg.event.Event(
            pg.MIDIIN,
            status=status,
            data1=data1,
            data2=data2,
            data3=data3,
            timestamp=timestamp,
            vice_id=device_id,
        )
        aus.append(e)
    return aus


def frequency_to_midi(frequency):
    """Eine Frequenz als MIDI-Notennummer - gerundet.

    Der Parameter hiess hier `freq`; `midi.pyi` nennt ihn `frequency`,
    und `pygame.midi.frequency_to_midi(frequency=440)` ist gemessen
    gueltig - der Name ist also Schnittstelle.
    """
    return int(round(69 + 12 * math.log(frequency / 440.0) / math.log(2)))


def midi_to_frequency(midi_note):
    """Eine MIDI-Notennummer als Frequenz - auf ein Zehntel gerundet."""
    return round(440.0 * 2 ** ((midi_note - 69) * (1.0 / 12.0)), 1)


def midi_to_ansi_note(midi_note):
    """Der ANSI-Name einer MIDI-Note, etwa `"C4"`."""
    notes = [
        "A",
        "A#",
        "B",
        "C",
        "C#",
        "D",
        "D#",
        "E",
        "F",
        "F#",
        "G",
        "G#",
    ]
    num_notes = 12
    note_name = notes[int((midi_note - 21) % num_notes)]
    # Zeilengetreu aus pygames `midi.py`. Hier stand
    # `round((midi_note - 21) / 11.0)`, was fuer 21, 60 und 127 zufaellig
    # dasselbe gibt und fuer 71 nicht: B**5** statt B4. Drei
    # uebereinstimmende Stichproben sind kein Nachweis einer Formel.
    note_number = (midi_note - 12) // num_notes
    return "%s%s" % (note_name, note_number)


def __getattr__(name):
    # `MIDIIN` und `MIDIOUT` sind dieselben Zahlen wie am Wurzelmodul -
    # `test_midiin` prueft genau diese Gleichheit. Sie werden erst beim
    # Zugriff geholt, weil dieser Text waehrend der Initialisierung von
    # `rustgame_py` laeuft.
    if name in ("MIDIIN", "MIDIOUT"):
        return getattr(_pg(), name)
    # `pygame.midi.pygame` - `pygame/midi.py` schreibt `import pygame`, und
    # damit steht das Paket in `dir(pygame.midi)`. Hier zeigt der Name auf
    # dieses Paket; erst beim Zugriff, aus demselben Grund wie oben.
    if name == "pygame":
        return _paket()
    raise AttributeError(
        "module %r has no attribute %r" % (__name__, name)
    )
"#,
        ),
        (
            "imageext",
            r#"
"""`pygame.imageext` - der Lader mit SDL_image, unter seinem alten Namen.

`imageext_test.py` holt `save_extended` und `load_extended` von hier.
Fehlt das Modul, faellt der Import auf das **echte** pygame zurueck, und
das lehnt dann eine fremde Flaeche ab:

    TypeError: argument 1 must be pygame.surface.Surface,
               not rustgame_py.surface.Surface

Eine Fehlermeldung, die nach einem Typfehler aussieht und in Wahrheit ein
fehlendes Modul meldet.
"""

__all__ = ["load_extended", "save_extended"]


# **Erst beim Zugriff holen.** Dieser Text laeuft waehrend der
# Initialisierung von `rustgame_py`; ein Import von dort waere ein Ring.
def __getattr__(name):
    if name in __all__:
        import rustgame_py

        return getattr(rustgame_py.image, name)
    raise AttributeError(
        "module %r has no attribute %r" % (__name__, name)
    )


def __dir__():
    """Die zwei Namen - **sonst waere `dir()` leer**.

    Ein `__getattr__` allein traegt jeden einzelnen Zugriff, aber es taucht
    in `dir()` nicht auf: gemessen hatte `dir(rustgame_py.imageext)` null
    Namen, wo pygame zwei hat. Derselbe Fall wie bei `_sprite`, und eine
    Weiterleitung, die man nicht sehen kann, ist keine Schnittstelle.
    """
    return sorted(__all__)
"#,
        ),
        (
            "rwobject",
            r#"
"""`pygame.rwobject` - Zeichenketten fuer die C-Schicht kodieren."""

__all__ = ["encode_string", "encode_file_path"]


def _pruefe_etype(etype):
    if etype is None:
        return
    if not (isinstance(etype, type) and issubclass(etype, BaseException)):
        raise TypeError("Expected an exception class: got %s" % (etype,))


_FEHLT = object()


def encode_string(obj=_FEHLT, encoding=None, errors=None, etype=None):
    # Gemessen an echtem pygame:
    #   gar kein Argument -> SyntaxError("Forwarded exception")
    #   None              -> None
    #   bytes             -> unveraendert, auch mit Nullbytes
    #   os.PathLike       -> ueber __fspath__
    #   etwas anderes     -> None (kein Fehler!)
    #   Kodierfehler      -> None, oder etype, wenn eines gereicht wurde
    _pruefe_etype(etype)
    if obj is _FEHLT:
        raise SyntaxError("Forwarded exception")
    if obj is None:
        return None
    if isinstance(obj, bytes):
        return obj
    if not isinstance(obj, str):
        # pathlib.PurePath und alles andere mit __fspath__.
        try:
            obj = obj.__fspath__()
        except AttributeError:
            return None
        if isinstance(obj, bytes):
            return obj
        if not isinstance(obj, str):
            return None
    if encoding is None:
        encoding = "unicode_escape"
    if errors is None:
        errors = "backslashreplace"
    if not isinstance(encoding, str):
        raise TypeError("Expected an encoding: got %s" % (encoding,))
    if not isinstance(errors, str):
        raise TypeError("Expected an error handler: got %s" % (errors,))
    try:
        return obj.encode(encoding, errors)
    except Exception as e:
        if etype is not None:
            raise etype(str(e)) from None
        return None


def encode_file_path(obj=_FEHLT, etype=None):
    # Wie `encode_string`, nur mit der Kodierung des Dateisystems - und
    # mit einer zusaetzlichen Regel: **ein eingebettetes Nullbyte gibt
    # `None`**. Ein Dateiname kann keines enthalten.
    import sys

    _pruefe_etype(etype)
    if obj is _FEHLT:
        raise SyntaxError("Forwarded exception")
    if obj is None:
        return None
    if isinstance(obj, bytes):
        b = obj
    elif isinstance(obj, str):
        try:
            b = obj.encode(sys.getfilesystemencoding(), "surrogateescape")
        except Exception as e:
            if etype is not None:
                raise etype(str(e)) from None
            return None
    else:
        try:
            weiter = obj.__fspath__()
        except AttributeError:
            return None
        return encode_file_path(weiter, etype)
    # Gemessen: ein Nullbyte gibt None, mit etype wird geworfen.
    if b'\x00' in b:
        if etype is not None:
            raise etype("embedded null byte")
        return None
    return b
"#,
        ),
        (
            "typing",
            r#"
"""`pygame.typing` - nur Typnamen."""

# `abstractmethod` steht bei pygame hier, und zwar benutzt: siehe
# `SequenceLike`.
from abc import abstractmethod
from os import PathLike as _PathProtocol

from typing import IO, Any, Callable, Protocol, TypeVar, Sequence, Tuple, Union

try:
    from typing import TypeAlias
except ImportError:                                  # pragma: no cover
    TypeAlias = Any

__all__ = [
    "SequenceLike",
    "FileLike",
    "ColorLike",
    "RectLike",
    "Point",
    "IntPoint",
]


_T_co = TypeVar("_T_co", covariant=True)


class SequenceLike(Protocol[_T_co]):
    """Alles, was sich wie eine Folge verhaelt.

    **Generisch, und das ist gemessen.** Hier stand `Protocol` ohne
    Typparameter, und damit war `Point` nur `Any`: `pygame.typing.Point`
    ist `SequenceLike[float]`, also traegt es `__name__ == "SequenceLike"`.
    Gefunden hat das die Probe `Funktionsnamen` - sie vergleicht den
    **eigenen** Namen jedes Dings, nicht den, unter dem es im Modul steht.
    """

    # **`@abstractmethod`, und das ist gemessen.** Ohne die Marke bleibt
    # `__abstractmethods__` leer, und `SequenceLike()` sagt dann
    # `TypeError: Protocols cannot be instantiated`; pygame sagt
    # `Can't instantiate abstract class SequenceLike without an
    # implementation for abstract methods '__getitem__', '__len__'` -
    # ABCMeta prueft die Pflichtmethoden, **bevor** `Protocol` zum Zug
    # kommt. `pygame/typing.py` schreibt `from abc import abstractmethod`
    # genau dafuer.
    @abstractmethod
    def __getitem__(self, index: int) -> _T_co:
        ...

    @abstractmethod
    def __len__(self) -> int:
        ...


# **Die drei Aliasse sind echte Vereinigungen, nicht `Any`.**
#
# Hier stand dreimal `Any`, und damit waren sie **rufbar**: `Any()` gibt
# `TypeError: Any cannot be instantiated`, wo pygame gar nichts gibt,
# weil ein `Union`-Objekt sich nicht rufen laesst. Gefunden hat das die
# Probe `Argumentfehler` - sie zaehlte in dieser Huelle drei rufbare
# Namen mehr als in pygame.
#
# Woertlich aus `pygame/typing.py`, samt der Hilfsnamen mit Unterstrich,
# die dort auch stehen.
_PathLike: TypeAlias = str | bytes | _PathProtocol[str] | _PathProtocol[bytes]
FileLike: TypeAlias = _PathLike | IO[bytes] | IO[str]

# `Point` und `IntPoint` sind bei pygame **Folgen von Zahlen**, nicht
# `Any` - und ihr `__name__` ist darum `SequenceLike`.
Point: TypeAlias = SequenceLike[float]
IntPoint: TypeAlias = SequenceLike[int]

# **Vorwaertsverweise statt Einfuhren.** pygames `typing.py` schreibt
# hier `from pygame.color import Color` und `from pygame.rect import
# FRect, Rect`. Dieser Text laeuft aber **waehrend** der Einrichtung von
# `rustgame_py`, und ein `import rustgame_py.color` haengt dann: das
# Elternmodul steht noch nicht in `sys.modules`, und der Einfuhrhaken
# wartet auf sich selbst. Das ist hier schon einmal passiert und war nur
# mit `faulthandler.dump_traceback_later` zu finden.
#
# Ein Vereinigungstyp nimmt Zeichenketten als Verweis an, und das Ergebnis
# ist dasselbe, was hier zaehlt: ein `Union`-Objekt, das sich **nicht
# rufen** laesst.
ColorLike: TypeAlias = Union["Color", SequenceLike[int], str, int]


class _HasRectAttribute(Protocol):
    # Ein Objekt mit einem `rect`-Feld - das Feld darf selbst ein Rechteck
    # sein oder eine Funktion, die eines gibt.
    @property
    def rect(self) -> "RectLike | Callable[[], RectLike]":
        ...


RectLike: TypeAlias = Union[
    "Rect", "FRect", SequenceLike[float], SequenceLike[Point], _HasRectAttribute
]

# **Namensraum aufraeumen** - mit demselben Vermerk, den pygames eigenes
# `typing.py` darueber schreibt (dort "cleanup namespace") und demselben
# Werkzeug. Ohne das stehen sieben Namen in `dir(pygame.typing)`, die
# pygame dort nicht hat: `Any`, `Callable`, `IO`, `Protocol`, `Sequence`,
# `Tuple`, `Union`.
#
# Es ist kein Schoenheitsfehler. `from pygame.typing import *` holt sonst
# `Any` mit, und wer danach `Any` schreibt, bekommt ein anderes `Any` als
# das, das er meint. pygame loest es genauso - und `TypeAlias` bleibt bei
# beiden stehen, weil es in den Zeilen darueber vorkommt.
del abstractmethod, Any, Callable, IO, Protocol, Sequence, Tuple, TypeVar, Union
"#,
        ),
        (
            "sysfont",
            r#"
"""`pygame.sysfont` - die Schriftsuche unter ihrem zweiten Namen."""

import difflib                                        # noqa: F401
import itertools                                     # noqa: F401
import os
import sys
import warnings                                      # noqa: F401
from os.path import basename, dirname, exists, join, splitext

__all__ = ["SysFont", "get_fonts", "match_font", "Font"]

# **`is_init` und `pygame_main_file` sind keine Importe, sondern Zustand.**
#
# `pygame/sysfont.py` merkt sich in `is_init`, ob die Systemschriften schon
# eingelesen sind, und in `pygame_main_file` den Pfad zu `pygame/__init__.py`
# - letzteren braucht die macOS-Suche, um neben dem Paket nach
# mitgelieferten Schriften zu sehen. Gemessen:
#
#     pygame.sysfont.is_init            False
#     pygame.sysfont.pygame_main_file   '.../site-packages/pygame/__init__.py'
#
# Beide stehen hier aus demselben Grund wie die fuenf `os.path`-Namen: wer
# `dir()` abtastet, findet sie bei pygame.
is_init = False


def _hauptdatei():
    """Der Pfad zu `__init__.py` des Pakets - erst beim Gebrauch.

    Die Huelle ist eine **einzelne** Datei (`rustgame_py.pyd`) und kein
    Ordner; es gibt also kein `__init__.py`. Gemeldet wird darum der Pfad,
    den die Huelle selbst hat - dieselbe Rolle: "hier wohnt das Paket".
    """
    import rustgame_py

    return getattr(rustgame_py, "__file__", None)

# **`join` und `splitext` stehen hier, weil sie bei pygame hier stehen.**
# `pygame/sysfont.py` schreibt `from os.path import basename, dirname,
# exists, join, splitext`, und damit sind die fuenf Teil des Namensraums:
# `from pygame.sysfont import join` geht dort. Sie auszulassen waere eine
# Luecke von fuenf Namen, die niemandem nuetzt - und eine Ausnahme in der
# Abdeckungstabelle waere eine Zeile Erklaerung fuer eine Zeile Import.

# **Die vier Namen werden erst beim Zugriff geholt.** Dieser Text wird
# *waehrend* der Initialisierung von `rustgame_py` ausgefuehrt: ein
# `import rustgame_py` waere ein Ring und haengt, und `sys.modules` hat
# den Eintrag zu diesem Zeitpunkt noch gar nicht. Ein Modul-`__getattr__`
# (PEP 562) verschiebt die Frage auf den ersten Gebrauch, und dann ist
# alles da.
def __getattr__(name):
    if name in ("Font", "SysFont", "get_fonts", "match_font"):
        import rustgame_py

        return getattr(rustgame_py.font, name)
    if name == "pygame_main_file":
        return _hauptdatei()
    raise AttributeError(
        "module %r has no attribute %r" % (__name__, name)
    )


def __dir__():
    """Damit `dir()` die vier durchgereichten Namen auch zeigt.

    Ein Modul-`__getattr__` (PEP 562) macht einen Namen **erreichbar**,
    aber nicht **sichtbar**: `dir(modul)` liest das Woerterbuch des
    Moduls, und dort steht nichts. `api_coverage.py` meldete die vier
    darum als fehlend oder "nur im Kern", obwohl jeder Zugriff darauf
    laeuft. Dieselbe Zeile steht aus demselben Grund in `video.py` und
    bei den Stellvertretern von `surfarray`/`sndarray`.

    **Ein Name, den niemand sieht, ist so gut wie keiner** - wer eine
    Schnittstelle abtastet statt sie zu kennen, findet ihn nicht.
    """
    return sorted(set(__all__) | set(globals()) | {"pygame_main_file"})


def _simplename(name):
    """Nur Buchstaben und Ziffern, klein geschrieben - `str.isalnum`, also
    **mit** Umlauten. Dieselbe Regel steht im Kern als `schlichter_name`."""
    return "".join(c.lower() for c in name if c.isalnum())


def _addfont(name, bold, italic, font, fontdict):
    """Traegt einen Schnitt ein; ein spaeterer Eintrag gewinnt."""
    if name not in fontdict:
        fontdict[name] = {}
    fontdict[name][bold, italic] = font


# **Die drei plattformeigenen Suchen - uebertragen, nicht durchgereicht.**
#
# pygame hat alle drei auf jedem System, und ein Programm, das
# `sysfont.initsysfonts_win32()` schreibt, darf sie finden. Hier reichten
# sie frueher `initsysfonts()` durch. Das gab das Falsche zurueck, gemessen
# unter Windows:
#
#                            pygame                   vorher hier
#     initsysfonts_win32()   neues dict, 390 Namen    Sysfonts, 391
#     initsysfonts_darwin()  {}                       Sysfonts, 391
#     initsysfonts_unix()    {} und eine Warnung      Sysfonts, 391
#
# Die drei lesen die Systemtabellen **jedes Mal neu** und geben ein
# eigenes Woerterbuch; `Sysfonts` und `is_init` fassen sie nicht an, und
# `freesansbold` steht nicht darin - das haengt erst `initsysfonts()` an.
# Die Suche, die `initsysfonts()` selbst faehrt, steht im Kern
# (`rustgame_font::sysfont`) und folgt derselben Regel.
def initsysfonts_win32():
    """Liest die Schriften aus der Registry - wie pygame ueber `winreg`."""
    import winreg as _winreg

    fontdir = join(os.environ.get("WINDIR", "C:\\Windows"), "Fonts")
    fonts = {}

    microsoft_font_dirs = [
        "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion\\Fonts",
        "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Fonts",
    ]

    for domain in [_winreg.HKEY_LOCAL_MACHINE, _winreg.HKEY_CURRENT_USER]:
        for font_dir in microsoft_font_dirs:
            try:
                key = _winreg.OpenKey(domain, font_dir)
            except FileNotFoundError:
                continue

            for i in range(_winreg.QueryInfoKey(key)[1]):
                try:
                    name, font, _ = _winreg.EnumValue(key, i)
                except OSError:
                    break

                if splitext(font)[1].lower() not in OpenType_extensions:
                    continue
                if not dirname(font):
                    font = join(fontdir, font)

                # "Cambria & Cambria Math" traegt zwei Namen.
                for name in name.split("&"):
                    if os.path.exists(font):
                        _parse_font_entry_win(name, font, fonts)

    return fonts


def _parse_font_entry_win(name, font, fonts):
    """Fett und kursiv aus dem Registry-Namen lesen.

    `rstrip` mit einer **Zeichenmenge** wie bei pygame - nicht "Endung
    abschneiden". Siehe den Kopf von `rustgame_font::sysfont`.
    """
    true_type_suffix = "(TrueType)"
    mods = ("demibold", "narrow", "light", "unicode", "bt", "mt")
    if name.endswith(true_type_suffix):
        name = name.rstrip(true_type_suffix).rstrip()
    name = name.lower().split()
    bold = italic = False
    for mod in mods:
        if mod in name:
            name.remove(mod)
    if "bold" in name:
        name.remove("bold")
        bold = True
    if "italic" in name:
        name.remove("italic")
        italic = True
    name = "".join(name)
    name = _simplename(name)

    _addfont(name, bold, italic, font, fonts)


def _parse_font_entry_darwin(name, filepath, fonts):
    """Fett und kursiv aus dem Dateinamen lesen (macOS)."""
    name = _simplename(name)

    mods = ("regular",)

    for mod in mods:
        if mod in name:
            name = name.replace(mod, "")

    bold = italic = False
    if "bold" in name:
        name = name.replace("bold", "")
        bold = True
    if "italic" in name:
        name = name.replace("italic", "")
        italic = True

    _addfont(name, bold, italic, filepath, fonts)


def _font_finder_darwin():
    """Die Schriftordner von macOS absuchen."""
    locations = [
        "/Library/Fonts",
        "/Network/Library/Fonts",
        "/System/Library/Fonts",
        "/System/Library/Fonts/Supplemental",
    ]

    if username := os.getenv("USER"):
        locations.append(f"/Users/{username}/Library/Fonts")

    strange_root = "/System/Library/Assets/com_apple_MobileAsset_Font3"
    if exists(strange_root):
        locations += [
            f"{strange_root}/{loc}/AssetData" for loc in os.listdir(strange_root)
        ]

    fonts = {}

    for location in locations:
        if not exists(location):
            continue

        files = os.listdir(location)
        for file in files:
            name, extension = splitext(file)
            if extension in OpenType_extensions:
                _parse_font_entry_darwin(name, join(location, file), fonts)

    return fonts


def initsysfonts_darwin():
    """Die Schriftsuche unter macOS: `fc-list` von X11, sonst die Ordner."""
    if exists("/usr/X11/bin/fc-list"):
        fonts = initsysfonts_unix("/usr/X11/bin/fc-list")
    elif exists("/usr/X11R6/bin/fc-list"):
        fonts = initsysfonts_unix("/usr/X11R6/bin/fc-list")
    else:
        fonts = _font_finder_darwin()

    return fonts


def initsysfonts_unix(path="fc-list"):
    """Die Schriftsuche ueber `fc-list` von fontconfig.

    Fehlt das Programm, **warnt** pygame und gibt ein leeres Woerterbuch -
    gemessen unter Windows mit pygames Wortlaut:
    `'fc-list' is missing, system fonts cannot be loaded on your platform`.
    """
    fonts = {}

    if sys.platform == "emscripten":
        return fonts

    import subprocess

    try:
        proc = subprocess.run(
            [path, ":", "file", "family", "style"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=True,
            timeout=1,
        )

    except FileNotFoundError:
        warnings.warn(
            f"'{path}' is missing, system fonts cannot be loaded on your platform"
        )

    except subprocess.TimeoutExpired:
        warnings.warn(
            f"Process running '{path}' timed-out! System fonts cannot be loaded on "
            "your platform"
        )

    except subprocess.CalledProcessError as e:
        warnings.warn(
            f"'{path}' failed with error code {e.returncode}! System fonts cannot be "
            f"loaded on your platform. Error log is:\n{e.stderr}"
        )

    else:
        for entry in proc.stdout.decode("ascii", "ignore").splitlines():
            try:
                _parse_font_entry_unix(entry, fonts)
            except ValueError:
                pass

    return fonts


def _parse_font_entry_unix(entry, fonts):
    """Eine Zeile von `fc-list` auswerten: `datei:familie:stil`."""
    filename, family, style = entry.split(":", 2)
    if splitext(filename)[1].lower() in OpenType_extensions:
        bold = "Bold" in style
        italic = "Italic" in style
        oblique = "Oblique" in style
        for name in family.strip().split(","):
            if name:
                break
        else:
            name = splitext(basename(filename))[0]

        _addfont(_simplename(name), bold, italic or oblique, filename, fonts)

# pygames eigene Innereien - Programme greifen darauf zu, deshalb gibt es
# sie, auch wenn die Suche selbst in Rust steht.
Sysfonts = {}
Sysalias = {}
# Ein `frozenset`, kein Tupel - so steht es in pygames `sysfont.py`
# Zeile 32. Beide antworten auf `in` gleich, aber `type()`, `repr()` und
# jede Mengenrechnung (`| {".pfb"}`) nicht. Gefunden hat das die Probe
# `Konstantenwerte`.
OpenType_extensions = frozenset((".ttf", ".ttc", ".otf"))

# Die fuenf Gruppen aus pygames `create_aliases`, wortwoertlich. Hier stand
# eine eigene Tabelle mit 33 Namen und Zielen wie `sans-serif`, `cursive`
# und `fantasy`, die pygame nicht kennt, und jeder Zweitname bekam vier
# Schnitte mit demselben Pfad. pygame setzt den Zweitnamen auf **dasselbe**
# Woerterbuch wie die gefundene Schrift: `Sysalias["monospace"] is
# Sysfonts["consolas"]` ist dort wahr.
def create_aliases():
    """Zweitnamen fuer gaengige Schriften, die es nicht gibt, auf die
    erste Schrift ihrer Gruppe, die es gibt."""
    alias_groups = (
        (
            "monospace",
            "misc-fixed",
            "courier",
            "couriernew",
            "console",
            "fixed",
            "mono",
            "freemono",
            "bitstreamverasansmono",
            "verasansmono",
            "monotype",
            "lucidaconsole",
            "consolas",
            "dejavusansmono",
            "liberationmono",
        ),
        (
            "sans",
            "arial",
            "helvetica",
            "swiss",
            "freesans",
            "bitstreamverasans",
            "verasans",
            "verdana",
            "tahoma",
            "calibri",
            "gillsans",
            "segoeui",
            "trebuchetms",
            "ubuntu",
            "dejavusans",
            "liberationsans",
        ),
        (
            "serif",
            "times",
            "freeserif",
            "bitstreamveraserif",
            "roman",
            "timesroman",
            "timesnewroman",
            "dutch",
            "veraserif",
            "georgia",
            "cambria",
            "constantia",
            "dejavuserif",
            "liberationserif",
        ),
        ("wingdings", "wingbats"),
        ("comicsansms", "comicsans"),
    )
    for alias_set in alias_groups:
        for name in alias_set:
            if name in Sysfonts:
                found = Sysfonts[name]
                break
        else:
            continue
        for name in alias_set:
            if name not in Sysfonts:
                Sysalias[name] = found


def initsysfonts():
    """Liest die Systemschriften einmal ein und fuellt `Sysfonts`.

    Die Suche selbst steht im Kern (`rustgame_font::sysfont`); dort haengt
    auch `freesansbold` hinten an. Der Ablauf hier ist pygames: einmal
    pruefen, dann `Sysfonts` fuellen, `create_aliases()` rufen, `is_init`
    setzen - und **nichts** zurueckgeben. Gemessen gibt pygames
    `initsysfonts()` `None`; hier kam `Sysfonts` zurueck.

    Vorher stand hier `if not Sysfonts:` statt `if is_init:`. Wer
    `is_init = False` setzt, bekommt bei pygame eine neue Suche.
    """
    global is_init
    if is_init:
        return

    import rustgame_py

    fonts, _alias = rustgame_py.font._sysfont_tabelle()
    Sysfonts.update(fonts)
    create_aliases()
    is_init = True


def font_constructor(fontpath, size, bold, italic):
    """Baut eine `font.Font` - Fett und Kursiv nur, wenn verlangt.

    pygame ruft `set_bold(True)` und `set_italic(True)` nur bei einer
    wahren Angabe; eine falsche laesst den Stil, wie die Schrift ihn
    mitbringt.
    """
    font = Font(fontpath, size)
    if bold:
        font.set_bold(True)
    if italic:
        font.set_italic(True)

    return font
"#,
        ),
    ] {
        let m2 = PyModule::from_code(
            py,
            std::ffi::CString::new(quelle)?.as_c_str(),
            std::ffi::CString::new(format!("rustgame_py/{name}.py"))?.as_c_str(),
            std::ffi::CString::new(format!("rustgame_py.{name}"))?.as_c_str(),
        )?;
        py.import("sys")?
            .getattr("modules")?
            .set_item(format!("rustgame_py.{name}"), &m2)?;
        // **`imageext` bleibt ein Feld des Wurzelmoduls**, und das war
        // einmal anders - fuer eine halbe Stunde.
        //
        // Eine Messung von Hand sagte `hasattr(pygame, "imageext")` ist
        // `False`, auch nach `import pygame.imageext`: pygames
        // `__init__.py` holt sich zwei Funktionen von dort und **loescht
        // den Namen** wieder, und die `import`-Anweisung setzt ihn fuer
        // ein Modul, das schon in `sys.modules` steht, nicht neu. Also
        // wurde das Feld hier weggelassen.
        //
        // **Das war falsch, und die bessere Messung sagte es schon.** Die
        // Probe `Hauptmodul Namen` laedt mit `importlib.import_module`,
        // und **das** setzt das Feld - bei pygame also auch. Sie hatte
        // "gleich" gemeldet; korrigiert wurde nach der schwaecheren
        // Messung.
        //
        // Was es gekostet hat: drei Fehler in `imageext_test.py`.
        // `from pygame import imageext` findet bei pygame das Untermodul
        // ueber `__path__`, wenn das Feld fehlt - `rustgame_py` ist **ein
        // einziges Modul** und hat kein `__path__`. Ohne das Feld greift
        // der Test zum echten pygame und bekommt
        // `TypeError: argument 1 must be pygame.surface.Surface`.
        //
        // **Zwei Messungen, die sich widersprechen, messen meist zwei
        // Zustaende** - und hier war die eine ausserdem die schwaechere.
        m.add(name, &m2)?;
        // **`rwobject` liegt auch am Wurzelmodul.** pygames
        // `__init__.py` schreibt `from pygame.rwobject import
        // encode_string, encode_file_path`, und `rwobject_test.py` holt
        // sie von dort - ohne diese zwei Zeilen laedt die Datei nicht.
        if name == "rwobject" {
            for f in ["encode_string", "encode_file_path"] {
                m.add(f, m2.getattr(f)?)?;
            }
        }
    }

    // **Die zwei Vorwaertsverweise in `typing` aufloesen.**
    //
    // `ColorLike` und `RectLike` stehen oben mit `"Color"`, `"Rect"` und
    // `"FRect"` als Zeichenkette, weil ein `import rustgame_py.color`
    // mitten im Modulaufbau haengt (siehe den Vermerk dort). Ein
    // `ForwardRef` ist aber **kein** gleichwertiger Ersatz, und das ist
    // gemessen:
    //
    // ```python
    // def f(c: pygame.typing.ColorLike): ...
    // typing.get_type_hints(f)
    // pygame     {'c': pygame.color.Color | ... | str | int}
    // rustgame   NameError: name 'Color' is not defined
    // ```
    //
    // Ein Programm, das seine Anmerkungen zur Laufzeit aufloest, scheiterte
    // hier also. Gebraucht wird dafuer **kein Import**: die drei Klassen
    // liegen als Rust-Typen vor, und ein `Union` laesst sich mit ihnen
    // unmittelbar bauen. Danach werden die Hilfsnamen wieder gestrichen,
    // damit `dir(typing)` dieselben elf Namen zeigt wie bei pygame.
    {
        let t = py
            .import("sys")?
            .getattr("modules")?
            .get_item("rustgame_py.typing")?;
        let raum = t.downcast::<PyModule>()?.dict();
        raum.set_item("Union", py.import("typing")?.getattr("Union")?)?;
        raum.set_item("Color", py.get_type::<crate::farbe::PyColor>())?;
        raum.set_item("Rect", py.get_type::<crate::rechteck::PyRect>())?;
        raum.set_item("FRect", py.get_type::<crate::frechteck::PyFRect>())?;
        py.run(
            std::ffi::CString::new(
                "ColorLike = Union[Color, SequenceLike[int], str, int]\n\
                 RectLike = Union[Rect, FRect, SequenceLike[float], \
                 SequenceLike[Point], _HasRectAttribute]\n\
                 del Union, Color, Rect, FRect\n",
            )?
            .as_c_str(),
            Some(&raum),
            None,
        )?;
    }


    // `pygame._sdl2.controller` - **nur die Verwaltung**, und das ist hier
    // die ganze Wahrheit.
    //
    // `joystick_test.py` schreibt in seiner **vierten** Zeile
    // `import pygame._sdl2.controller` und ruft spaeter genau einmal
    // `controller.quit()`. Ohne das Modul laedt die Datei nicht, und ihre
    // fuenf Tests sind nicht rot, sondern unsichtbar - dieselbe Lage wie
    // bei `bufferproxy` und `surface_test.py`.
    //
    // Was hier steht, ist die Buchfuehrung: `init`, `quit`, `get_init`,
    // `get_count`, `is_controller`. **`get_count()` gibt 0**, und das ist
    // keine Ausrede: rustgame hat keine Spielsteuerungs-Schicht, und unter
    // dem Treiber `dummy` - dem Modus dieses Projekts - findet auch echtes
    // pygame keine. Nachgemessen: `init(); get_count()` gibt dort
    // ebenfalls 0.
    //
    // `Controller(i)` gibt einen klaren Fehler statt eines Scheinobjekts.
    // Ein Programm, das eine Steuerung erwartet, soll es hier erfahren und
    // nicht an einem Objekt, das nie eine Taste meldet.
    {
        let quelle = r#"
_laeuft = False

__all__ = ["init", "quit", "get_init", "get_count", "is_controller", "Controller"]


def init():
    global _laeuft
    _laeuft = True


def quit():
    global _laeuft
    _laeuft = False


def get_init():
    return _laeuft


def _pruefen():
    if not _laeuft:
        import rustgame_py

        raise rustgame_py.error("Controller system not initialized")


def get_count():
    # Keine Spielsteuerungs-Schicht, und unter dem Treiber "dummy" gibt es
    # auch nichts zu finden.
    _pruefen()
    return 0


def is_controller(index):
    """Ist das Geraet mit dieser Nummer ein Spielgeraet?

    **Ein Text ist keine Nummer.** Gemessen gibt
    `controller.is_controller("Test")` einen `TypeError`, und
    `controller_test` prueft genau diese eine Zeile - hier kaeme sonst
    klaglos `False` heraus.
    """
    _pruefen()
    if isinstance(index, bool) or not isinstance(index, int):
        raise TypeError("index must be an integer")
    return False


def _nie():
    """Der Fehler, den es hier immer gibt - mit pygames Wortlaut.

    **Zwei Texte, und welcher kommt, haengt an `init()`** - gemessen:
    vor dem Hochfahren `Controller system not initialized`, danach `Invalid index`.
    Hier stand nur der zweite (bzw. ein selbst erfundener deutscher Satz),
    und damit war der haeufigere Fall - jemand ruft `Controller(0)`, ohne
    vorher `init()` zu rufen - falsch beantwortet.
    """
    import rustgame_py

    if not get_init():
        return rustgame_py.error("Controller system not initialized")
    return rustgame_py.error("Invalid index")


class Controller:
    """Ein Spielgeraet - **es gibt hier keines.**

    ## Warum die Klasse ihre Namen trotzdem traegt

    Hier stand eine Klasse mit **einem** Namen (`__init__`), und das fiel
    keinem Test auf: `controller_test.py` ueberspringt sich ohne Geraet.
    Gefunden hat es `alle_klassen()` in `api_coverage.py`, nachdem die
    Klassenliste gelesen statt genannt wurde - **15 Namen, keiner da.**

    Gerufen werden kann keiner, denn `Controller(i)` scheitert immer; die
    Namen sind reine Schnittstelle. Ein Programm darf
    `hasattr(Controller, "get_axis")` fragen, und ein Pruefwerkzeug darf
    `dir()` vergleichen.

    ## Die gemessenen Fehlertexte

    Sie haengen am Init-Zustand, und die beiden Fassungen sagen **nicht
    dasselbe**:

    ```text
    vor  init()   error: Controller system not initialized
    nach init()   error: Invalid index
    Controller()  TypeError: function missing required argument 'device_index' (pos 1)
    Controller(x) TypeError: 'str' object cannot be interpreted as an integer
    ```

    `Controller(True)` ist dabei **keine** Typverletzung: ein `bool` ist
    eine Zahl, und pygame gibt denselben Fehler wie fuer `1`.
    """

    def __init_subclass__(cls, **kwargs):
        """**Nicht ableitbar** - gemessen.

        `class X(Controller)` gibt bei pygame
        `TypeError: type 'Controller' is not an acceptable base type`, weil
        der Cython-Typ kein `Py_TPFLAGS_BASETYPE` traegt. Eine
        Python-Klasse ist immer ableitbar; `__init_subclass__` ist die
        Stelle, an der sich das nachbilden laesst - und der Wortlaut
        stimmt.

        Die **alte** Fassung laesst sich dagegen ableiten, auch gemessen.
        Darum steht das hier und nicht dort.
        """
        raise TypeError("type 'Controller' is not an acceptable base type")

    def __init__(self, *args):
        """**`*args`, damit die Saetze stimmen.**

        Gemessen sagt diese Fassung bei falscher Anzahl
        `function missing required argument 'device_index' (pos 1)`
        beziehungsweise `function takes at most 1 argument (2 given)`.
        Mit einem benannten Argument schreibt Python seinen eigenen Satz,
        und der passt zu keinem von beiden.
        """
        if len(args) != 1:
            raise TypeError(
                "function missing required argument 'device_index' (pos 1)"
                if not args
                else "function takes at most 1 argument (%d given)" % len(args))
        device_index = args[0]
        if isinstance(device_index, bool):
            # Ein `bool` ist eine Zahl - gemessen, siehe Klassenkopf.
            device_index = int(device_index)
        if not isinstance(device_index, int):
            # **Diese Fassung nimmt kein `0.5`**, die alte schon - Cython
            # schneidet dort ab. Gemessen, beide.
            raise TypeError(
                "'%s' object cannot be interpreted as an integer"
                % type(device_index).__name__)
        raise _nie()

    @property
    def id(self):
        """Die Nummer, mit der das Geraet geoeffnet wurde."""
        raise _nie()

    @property
    def name(self):
        """Der Name des Geraets."""
        raise _nie()

    @staticmethod
    def from_joystick(joystick):
        """Baut eine Steuerung aus einem `pygame.joystick.Joystick`."""
        raise TypeError("argument 1 must be pygame.joystick.Joystick, not %s" % type(joystick).__name__)

    def as_joystick(self):
        """Gibt dasselbe Geraet als `Joystick`."""
        raise _nie()

    def attached(self):
        """Haengt das Geraet noch dran?"""
        raise _nie()

    def init(self):
        """Oeffnet das Geraet."""
        raise _nie()

    def quit(self):
        """Schliesst es."""
        raise _nie()

    def get_init(self):
        """Ist es offen?"""
        raise _nie()

    def get_axis(self, axis):
        """Eine Achse - Fliesskomma von -1 bis 1."""
        raise _nie()

    def get_button(self, button):
        """Ein Knopf."""
        raise _nie()

    def get_mapping(self):
        """Die Belegungstabelle als Woerterbuch."""
        raise _nie()

    def set_mapping(self, mapping):
        """Setzt die Belegungstabelle."""
        raise _nie()

    def rumble(self, low_frequency, high_frequency, duration):
        """Ruettelt."""
        raise _nie()

    def stop_rumble(self):
        """Hoert auf zu ruetteln."""
        raise _nie()

    def set_led(self, *args):
        """Die Leuchte am Geraet - **nur die neue Fassung hat das.**"""
        raise _nie()
"#;
        let c = PyModule::from_code(
            py,
            std::ffi::CString::new(quelle)?.as_c_str(),
            std::ffi::CString::new("rustgame_py/_sdl2/controller.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py._sdl2.controller")?.as_c_str(),
        )?;
        // `pygame._sdl2.touch` - **kein Beruehrungsgeraet**, aber die
        // Fehlerarten stimmen. `touch_test.py` prueft nur die: eine
        // ungueltige Nummer gibt `pygame.error`, ein Text `TypeError`.
        // Vier von fuenf Tests kommen ohne Geraet aus.
        let touch_quelle = r#"
"""`pygame._sdl2.touch` - Beruehrungsgeraete.

Es gibt hier keine; SDLs Geraeteliste braeuchte FFI. Was die Tests
pruefen, sind aber die **Fehlerarten**, und die stimmen:

    get_device(-1234)     pygame.error
    get_device("test")    TypeError
"""

__all__ = ["get_num_devices", "get_device", "get_num_fingers", "get_finger"]


def _fehler():
    import rustgame_py

    return rustgame_py.error


def get_num_devices():
    """Wie viele Beruehrungsgeraete es gibt - hier keines."""
    return 0


def get_device(index):
    """Die Kennung des Geraets mit dieser Nummer."""
    if isinstance(index, bool) or not isinstance(index, int):
        raise TypeError("index must be an integer")
    raise _fehler()("Unable to get touch device")


def get_num_fingers(touchid):
    """Wie viele Finger dieses Geraet gerade meldet."""
    if isinstance(touchid, bool) or not isinstance(touchid, int):
        raise TypeError("touchid must be an integer")
    raise _fehler()("Unable to get number of fingers")


def get_finger(touchid, index):
    """Ein einzelner Finger als Woerterbuch - oder `None`."""
    if isinstance(touchid, bool) or not isinstance(touchid, int):
        raise TypeError("touchid must be an integer")
    return None
"#;
        let t = PyModule::from_code(
            py,
            std::ffi::CString::new(touch_quelle)?.as_c_str(),
            std::ffi::CString::new("rustgame_py/_sdl2/touch.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py._sdl2.touch")?.as_c_str(),
        )?;

        // `pygame._render` und `pygame._sdl2.video` - **dieselben** drei
        // Klassen unter zwei Namen. Der Text liegt als eigene Datei
        // daneben, nicht als Zeichenkette in dieser Datei: es sind an die
        // sechshundert Zeilen Python, und in einem `r#"..."#` waere davon
        // nichts mehr lesbar.
        let r = PyModule::from_code(
            py,
            std::ffi::CString::new(include_str!("python/_render.py"))?.as_c_str(),
            std::ffi::CString::new("rustgame_py/_render.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py._render")?.as_c_str(),
        )?;
        let v = PyModule::from_code(
            py,
            std::ffi::CString::new(include_str!("python/video.py"))?.as_c_str(),
            std::ffi::CString::new("rustgame_py/_sdl2/video.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py._sdl2.video")?.as_c_str(),
        )?;

        // **Fuenf Untermodule, die lange gefehlt haben.**
        //
        // `pygame._sdl2` hat **acht**: `audio`, `controller`,
        // `controller_old`, `mixer`, `sdl2`, `touch`, `video`, `window`.
        // Hier standen drei, und `api_coverage.py` fragte nach **einem**
        // (`_sdl2.video`) - die anderen fuenf waren damit nie Gegenstand
        // einer Messung.
        //
        // Gefunden hat es nicht die Abdeckungstabelle, sondern die
        // **Endabnahme des Plans**: dort steht, das Spielprogramm solle
        // ueber dreihundert Bilder verglichen werden, und gefahren wurden
        // dreissig. Bei dreihundert wich die Ereignisschlange ab, und in
        // der Spur stand `AUDIODEVICEADDED` - ein Ereignis, das es hier
        // nicht gab, weil es die Tongeraete nicht gab.
        //
        // **Eine Zahl im Plan, die nie eingelöst wurde, ist dasselbe wie
        // ein Vermerk, den niemand nachhaelt.**
        let a = PyModule::from_code(
            py,
            std::ffi::CString::new(include_str!("python/sdl2_audio.py"))?.as_c_str(),
            std::ffi::CString::new("rustgame_py/_sdl2/audio.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py._sdl2.audio")?.as_c_str(),
        )?;
        let co = PyModule::from_code(
            py,
            std::ffi::CString::new(include_str!("python/sdl2_controller_old.py"))?
                .as_c_str(),
            std::ffi::CString::new("rustgame_py/_sdl2/controller_old.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py._sdl2.controller_old")?.as_c_str(),
        )?;
        let mx = PyModule::from_code(
            py,
            std::ffi::CString::new(include_str!("python/sdl2_mixer.py"))?.as_c_str(),
            std::ffi::CString::new("rustgame_py/_sdl2/mixer.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py._sdl2.mixer")?.as_c_str(),
        )?;
        let s2 = PyModule::from_code(
            py,
            std::ffi::CString::new(include_str!("python/sdl2_sdl2.py"))?.as_c_str(),
            std::ffi::CString::new("rustgame_py/_sdl2/sdl2.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py._sdl2.sdl2")?.as_c_str(),
        )?;
        // `_sdl2.window` ist **dasselbe** `Window` wie in `pygame.window` -
        // gemessen: `_sdl2.window.Window is pygame.window.Window` ist wahr.
        // Also kein zweites Modul mit zweiter Klasse, sondern ein Modul,
        // das auf die vorhandene zeigt.
        let wi = PyModule::new(py, "rustgame_py._sdl2.window")?;
        // **Der Typ direkt, nicht ueber `m.getattr("window")`.** Das
        // Untermodul `window` entsteht weiter unten; hier waere es noch
        // nicht da, und `getattr` gaebe einen `AttributeError`.
        wi.add("Window", py.get_type::<fenster::PyWindow>())?;

        // **`_sdl2` traegt die Namen seiner Kinder mit** - gemessen hat
        // `dir(pygame._sdl2)` achtundvierzig Eintraege. Das Paket ist
        // darum Python und kein leeres `PyModule::new`: es braucht ein
        // `__getattr__` auf Modulebene.
        //
        // Der erste Versuch stand hier und lief ueber die Kinder, um jeden
        // Namen mit `getattr` zu holen. **Das haengt:** `video.py` holt
        // seine Klassen nachtraeglich und schreibt dafuer
        // `import rustgame_py` - und waehrend `rustgame_py` eingerichtet
        // wird, steht es nicht in `sys.modules`. Der Import faengt von
        // vorne an und wartet auf sich selbst.
        let sdl2 = PyModule::from_code(
            py,
            std::ffi::CString::new(include_str!("python/sdl2_init.py"))?.as_c_str(),
            std::ffi::CString::new("rustgame_py/_sdl2/__init__.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py._sdl2")?.as_c_str(),
        )?;
        // **`setattr`, nicht `add`.** PyO3s `PyModule::add` traegt den
        // Namen zusaetzlich in `__all__` ein - und `sdl2_init.py` hat die
        // acht Untermodule dort schon. Gemessen hatte
        // `dir(rustgame_py._sdl2)` darum **59** Eintraege mit acht
        // Dubletten, wo pygame 51 hat. `dir()` entdoppelt nicht, wenn ein
        // Modul sein `__dir__` selbst stellt.
        for (name, kind) in [
            ("audio", &a),
            ("controller", &c),
            ("controller_old", &co),
            ("mixer", &mx),
            ("sdl2", &s2),
            ("touch", &t),
            ("video", &v),
            ("window", &wi),
        ] {
            sdl2.setattr(name, kind)?;
        }
        let module = py.import("sys")?.getattr("modules")?;
        module.set_item("rustgame_py._sdl2", &sdl2)?;
        module.set_item("rustgame_py._sdl2.audio", &a)?;
        module.set_item("rustgame_py._sdl2.controller", &c)?;
        module.set_item("rustgame_py._sdl2.controller_old", &co)?;
        module.set_item("rustgame_py._sdl2.mixer", &mx)?;
        module.set_item("rustgame_py._sdl2.sdl2", &s2)?;
        module.set_item("rustgame_py._sdl2.touch", &t)?;
        module.set_item("rustgame_py._sdl2.video", &v)?;
        module.set_item("rustgame_py._sdl2.window", &wi)?;
        module.set_item("rustgame_py._render", &r)?;
        m.add("_sdl2", &sdl2)?;
        m.add("_render", &r)?;
    }

    // **`pygame._sprite` entsteht erst, wenn jemand es importiert.**
    //
    // Es ist der alte Name von `pygame.sprite` und warnt beim Import.
    // Stuende es in der Liste oben, liefe der Text schon beim Laden von
    // `rustgame_py` - und dann bekaeme **jedes** Programm die
    // Veraltungsmeldung zu lesen, ohne den Namen je benutzt zu haben.
    // `sprite_test.test_deprecation__sprite_submodule` faellt darauf
    // ebenfalls herein: die Warnung waere laengst verklungen, wenn der
    // Test importiert.
    //
    // Also wird ein Sucher in `sys.meta_path` gehaengt, der das Modul auf
    // Zuruf baut.
    {
        let quelle = r#"
"""Haengt `rustgame_py._sprite` als Sucher ein - siehe `lib.rs`."""

import importlib.util
import sys
import warnings


class _Lader:
    """Baut `rustgame_py._sprite`, wenn jemand danach fragt."""

    def create_module(self, spec):
        return None

    def exec_module(self, modul):
        warnings.warn(
            "pygame._sprite is deprecated, use pygame.sprite instead",
            DeprecationWarning,
            stacklevel=2,
        )
        modul.__doc__ = "`pygame._sprite` - der alte Name von `pygame.sprite`."

        # **Die Namen werden eingetragen, nicht nur weitergeleitet.**
        #
        # Hier stand allein ein `__getattr__`, und das traegt fuer jeden
        # einzelnen Zugriff - aber `dir(pygame._sprite)` war **leer**. Die
        # Abdeckungstabelle meldete darum `0 von 18`, sobald sie das Modul
        # ueberhaupt zu fragen begann, und eine Weiterleitung, die man
        # nicht sehen kann, ist keine Schnittstelle.
        # **Und nicht alle.** Gemessen hat `dir(pygame._sprite)` genau
        # diese neunzehn Namen - das C-Modul von damals kannte die
        # Hilfsnamen nicht, die `sprite.py` heute mitbringt
        # (`OrderedUpdates`, `Rect`, `from_surface`, `get_ticks`,
        # `pygame`, `types`, `warn`), und es traegt dafuer `warnings`.
        import warnings as _warnings

        import rustgame_py

        _ALT = (
            "AbstractGroup", "DirtySprite", "Group", "GroupSingle",
            "LayeredDirty", "LayeredUpdates", "RenderClear", "RenderPlain",
            "RenderUpdates", "Sprite", "collide_circle",
            "collide_circle_ratio", "collide_mask", "collide_rect",
            "collide_rect_ratio", "groupcollide", "spritecollide",
            "spritecollideany",
        )
        for _n in _ALT:
            setattr(modul, _n, getattr(rustgame_py.sprite, _n))
        modul.warnings = _warnings

        def hole(name):
            # Bleibt fuer alles, was `sprite` erst spaeter bekommt.
            #
            # **Aber keine Doppelstriche.** Hier stand nur die letzte
            # Zeile, und damit reichte das Modul auch `__all__` weiter:
            # `from pygame._sprite import *` brachte 25 Namen statt der
            # 19, die im Woerterbuch stehen - pygames `_sprite` hat gar
            # kein `__all__`. Gefunden hat das die Probe `Sternimport`.
            if name.startswith("__"):
                raise AttributeError(
                    "module %r has no attribute %r" % (__name__, name)
                )
            return getattr(rustgame_py.sprite, name)

        modul.__getattr__ = hole


class _Sucher:
    """Faengt `<paket>._sprite` ab - aber nur fuer dieses Paket.

    Der Name kann `rustgame_py._sprite` lauten oder `pygame._sprite`:
    wer rustgame als Ersatz einsetzt, legt es unter dem alten Namen ab,
    und dann sucht Python auch unter dem alten. Geprueft wird deshalb
    nicht der Text, sondern **wer der Elternteil ist**.
    """

    def find_spec(self, name, path=None, target=None):
        eltern, _, letzt = name.rpartition(".")
        if letzt != "_sprite" or not eltern:
            return None
        import rustgame_py

        if sys.modules.get(eltern) is not rustgame_py:
            return None
        return importlib.util.spec_from_loader(name, _Lader())


if not any(type(f).__name__ == "_Sucher" for f in sys.meta_path):
    sys.meta_path.insert(0, _Sucher())
"#;
        PyModule::from_code(
            py,
            std::ffi::CString::new(quelle)?.as_c_str(),
            std::ffi::CString::new("rustgame_py/_sprite_sucher.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py._sprite_sucher")?.as_c_str(),
        )?;
    }

    // `pygame.bufferproxy` - da, weil `surface_test.py` sonst nicht einmal
    // laedt; die Begruendung steht im Modulkopf von `pufferzeiger.rs`.
    untermodul(m, "bufferproxy", |b| {
        b.add_class::<pufferzeiger::PyBufferProxy>()?;
        b.add("_PYGAME_C_API", c_api_kapsel(b.py(), "bufferproxy")?)?;
        Ok(())
    })?;
    m.add_class::<pufferzeiger::PyBufferProxy>()?;
    m.add_function(wrap_pyfunction!(get_array_interface, m)?)?;

    // Die Geraeteseite von MIDI. Mit Unterstrich, weil sie niemandem
    // gehoert als dem Python-Modul `pygame.midi` daneben - die Zaehlarbeit
    // steht dort, hier nur das Anfassen des Geraets.
    als(m, "_midi_anzahl", wrap_pyfunction!(midigeraet::midi_anzahl, m)?)?;
    als(m, "_midi_geraet", wrap_pyfunction!(midigeraet::midi_geraet, m)?)?;
    als(
        m,
        "_midi_erster_eingang",
        wrap_pyfunction!(midigeraet::midi_erster_eingang, m)?,
    )?;
    als(
        m,
        "_midi_erster_ausgang",
        wrap_pyfunction!(midigeraet::midi_erster_ausgang, m)?,
    )?;
    m.add_class::<midigeraet::PyMidiAusgang>()?;
    m.add_class::<midigeraet::PyMidiEingang>()?;

    // Dasselbe fuer die Kamera, und aus demselben Grund: `colorspace`,
    // `AbstractCamera` und die Reglerbuchhaltung sind Rechnerei und stehen
    // in `python/camera.py`; hier liegt nur das Geraet.
    als(m, "_kamera_geraete", wrap_pyfunction!(kamera::kamera_geraete, m)?)?;
    // Die Tongeraete - `_sdl2.audio.get_audio_device_names` holt sie hier.
    als(m, "_tongeraete", wrap_pyfunction!(tongeraete, m)?)?;
    als(
        m,
        "_kamera_rueckwaende",
        wrap_pyfunction!(kamera::kamera_rueckwaende, m)?,
    )?;
    m.add_class::<kamera::PyKamera>()?;

    // `pygame.base` - das Modul, in dem pygame seine Grundfunktionen hat.
    //
    // In pygame-ce ist `base` die C-Kiste, aus der alles andere waechst;
    // `pygame/__init__.py` holt ihre Namen ins Hauptmodul hoch, laesst das
    // Modul selbst aber stehen. Zwei Tests greifen darauf zu:
    // `bufferproxy_test.test_c_api` vergleicht
    // `type(pygame.base._PYGAME_C_API)` mit dem der eigenen Kapsel, und
    // `pygame.base.BufferError is pygame.BufferError` ist wahr.
    //
    // Also steht hier kein Nachbau, sondern **derselbe** Satz Namen noch
    // einmal - ein Modul, das auf die Dinge zeigt, die oben schon stehen.
    // `pygame.base.HAVE_NEWBUF` - **1**, und hier dieselbe Zahl.
    //
    // `pygame.HAVE_NEWBUF` steht schon oben; `base` traegt sie ein zweites
    // Mal, weil sie dort entsteht. Der Wert sagt, dass das Puffer-Protokoll
    // da ist - und seit `neuerpuffer.rs` ist das auch von Python aus wahr.
    untermodul(m, "base", |b| {
        b.add("HAVE_NEWBUF", 1i32)?;
        let py = b.py();
        b.add("_PYGAME_C_API", c_api_kapsel(py, "base")?)?;
        b.add("error", py.get_type::<PygameError>())?;
        b.add("BufferError", py.get_type::<umwandlung::PygameBufferError>())?;
        b.add_function(wrap_pyfunction!(get_array_interface, b)?)?;
        b.add_function(wrap_pyfunction!(init, b)?)?;
        b.add_function(wrap_pyfunction!(quit, b)?)?;
        b.add_function(wrap_pyfunction!(get_init, b)?)?;
        b.add_function(wrap_pyfunction!(register_quit, b)?)?;
        b.add_function(wrap_pyfunction!(get_error, b)?)?;
        b.add_function(wrap_pyfunction!(set_error, b)?)?;
        b.add_function(wrap_pyfunction!(pufferzeiger::get_sdl_byteorder, b)?)?;
        b.add_function(wrap_pyfunction!(fassung::fassung_get_sdl_version, b)?)?;
        Ok(())
    })?;

    untermodul(m, "draw", |d| {
        als(d, "rect", wrap_pyfunction!(zeichnen::zeichne_rect, d)?)?;
        als(d, "circle", wrap_pyfunction!(zeichnen::zeichne_circle, d)?)?;
        als(d, "ellipse", wrap_pyfunction!(zeichnen::zeichne_ellipse, d)?)?;
        als(d, "arc", wrap_pyfunction!(zeichnen::zeichne_arc, d)?)?;
        als(d, "line", wrap_pyfunction!(zeichnen::zeichne_line, d)?)?;
        als(d, "lines", wrap_pyfunction!(zeichnen::zeichne_lines, d)?)?;
        als(d, "polygon", wrap_pyfunction!(zeichnen::zeichne_polygon, d)?)?;
        als(d, "aaline", wrap_pyfunction!(zeichnen::zeichne_aaline, d)?)?;
        als(d, "aalines", wrap_pyfunction!(zeichnen::zeichne_aalines, d)?)?;
        als(d, "aacircle", wrap_pyfunction!(zeichnen::zeichne_aacircle, d)?)?;
        als(
            d,
            "flood_fill",
            wrap_pyfunction!(zeichnen::zeichne_flood_fill, d)?,
        )?;
        Ok(())
    })?;

    untermodul(m, "transform", |t| {
        als(t, "scale", wrap_pyfunction!(umformen::tf_scale, t)?)?;
        als(t, "scale_by", wrap_pyfunction!(umformen::tf_scale_by, t)?)?;
        als(t, "smoothscale", wrap_pyfunction!(umformen::tf_smoothscale, t)?)?;
        als(
            t,
            "smoothscale_by",
            wrap_pyfunction!(umformen::tf_smoothscale_by, t)?,
        )?;
        als(t, "flip", wrap_pyfunction!(umformen::tf_flip, t)?)?;
        als(t, "rotate", wrap_pyfunction!(umformen::tf_rotate, t)?)?;
        als(t, "scale2x", wrap_pyfunction!(umformen::tf_scale2x, t)?)?;
        als(t, "grayscale", wrap_pyfunction!(umformen::tf_grayscale, t)?)?;
        als(
            t,
            "average_color",
            wrap_pyfunction!(umformen::tf_average_color, t)?,
        )?;
        als(
            t,
            "invert",
            wrap_pyfunction!(umformen::tf_invert, t)?,
        )?;
        als(
            t,
            "laplacian",
            wrap_pyfunction!(umformen::tf_laplacian, t)?,
        )?;
        als(
            t,
            "chop",
            wrap_pyfunction!(umformen::tf_chop, t)?,
        )?;
        als(
            t,
            "box_blur",
            wrap_pyfunction!(umformen::tf_box_blur, t)?,
        )?;
        als(
            t,
            "gaussian_blur",
            wrap_pyfunction!(umformen::tf_gaussian_blur, t)?,
        )?;
        als(
            t,
            "pixelate",
            wrap_pyfunction!(umformen::tf_pixelate, t)?,
        )?;
        als(
            t,
            "solid_overlay",
            wrap_pyfunction!(umformen::tf_solid_overlay, t)?,
        )?;
        als(
            t,
            "hsl",
            wrap_pyfunction!(umformen::tf_hsl, t)?,
        )?;
        als(
            t,
            "rotozoom",
            wrap_pyfunction!(umformen::tf_rotozoom, t)?,
        )?;
        als(
            t,
            "average_surfaces",
            wrap_pyfunction!(umformen::tf_average_surfaces, t)?,
        )?;
        als(
            t,
            "get_smoothscale_backend",
            wrap_pyfunction!(umformen::tf_get_smoothscale_backend, t)?,
        )?;
        als(
            t,
            "set_smoothscale_backend",
            wrap_pyfunction!(umformen::tf_set_smoothscale_backend, t)?,
        )?;
        als(
            t,
            "threshold",
            wrap_pyfunction!(umformen::tf_threshold, t)?,
        )?;
        Ok(())
    })?;

    untermodul(m, "image", |i| {
        als(i, "load", wrap_pyfunction!(bild::bild_load, i)?)?;
        als(i, "save", wrap_pyfunction!(bild::bild_save, i)?)?;
        // pygame trennt den Lader **ohne** SDL_image (`load_basic`, nur
        // BMP) vom Lader **mit** (`load_extended`). rustgame hat diese
        // Trennung nicht; beide Namen zeigen auf denselben Lader, damit
        // ein Programm, das sie aufruft, nicht scheitert.
        als(i, "load_basic", wrap_pyfunction!(bild::bild_load_basic, i)?)?;
        als(
            i,
            "load_animation",
            wrap_pyfunction!(bild::bild_load_animation, i)?,
        )?;
        als(
            i,
            "load_sized_svg",
            wrap_pyfunction!(bild::bild_load_sized_svg, i)?,
        )?;
        als(
            i,
            "load_extended",
            wrap_pyfunction!(bild::bild_load_extended, i)?,
        )?;
        als(
            i,
            "save_extended",
            wrap_pyfunction!(bild::bild_save_extended, i)?,
        )?;
        als(i, "tobytes", wrap_pyfunction!(bild::bild_tobytes, i)?)?;
        als(i, "tostring", wrap_pyfunction!(bild::bild_tostring, i)?)?;
        als(i, "frombytes", wrap_pyfunction!(bild::bild_frombytes, i)?)?;
        als(i, "fromstring", wrap_pyfunction!(bild::bild_fromstring, i)?)?;
        als(i, "frombuffer", wrap_pyfunction!(bild::bild_frombuffer, i)?)?;
        als(
            i,
            "get_extended",
            wrap_pyfunction!(bild::bild_get_extended, i)?,
        )?;
        als(
            i,
            "get_sdl_image_version",
            wrap_pyfunction!(bild::bild_get_sdl_image_version, i)?,
        )?;
        Ok(())
    })?;

    untermodul(m, "time", |t| {
        t.add_class::<zeit::PyClock>()?;
        als(t, "get_ticks", wrap_pyfunction!(zeit::zeit_get_ticks, t)?)?;
        als(t, "wait", wrap_pyfunction!(zeit::zeit_wait, t)?)?;
        als(t, "delay", wrap_pyfunction!(zeit::zeit_delay, t)?)?;
        als(t, "set_timer", wrap_pyfunction!(zeit::zeit_set_timer, t)?)?;
        Ok(())
    })?;

    untermodul(m, "display", |d| {
        // **`VidInfo` steht nicht im Modul** - gemessen hat
        // `dir(pygame.display)` nur `Info`, und das ist die Fabrik. Die
        // Klasse ist allein ueber `type(Info())` zu erreichen; siehe
        // `anzeige.rs`.
        als(d, "Info", wrap_pyfunction!(anzeige::anzeige_info, d)?)?;
        als(d, "init", wrap_pyfunction!(anzeige::anzeige_init, d)?)?;
        als(d, "quit", wrap_pyfunction!(anzeige::anzeige_quit, d)?)?;
        als(d, "get_init", wrap_pyfunction!(anzeige::anzeige_get_init, d)?)?;
        als(d, "set_mode", wrap_pyfunction!(anzeige::anzeige_set_mode, d)?)?;
        als(
            d,
            "get_surface",
            wrap_pyfunction!(anzeige::anzeige_get_surface, d)?,
        )?;
        als(d, "flip", wrap_pyfunction!(anzeige::anzeige_flip, d)?)?;
        als(d, "update", wrap_pyfunction!(anzeige::anzeige_update, d)?)?;
        als(
            d,
            "set_caption",
            wrap_pyfunction!(anzeige::anzeige_set_caption, d)?,
        )?;
        als(
            d,
            "get_caption",
            wrap_pyfunction!(anzeige::anzeige_get_caption, d)?,
        )?;
        als(d, "set_icon", wrap_pyfunction!(anzeige::anzeige_set_icon, d)?)?;
        als(
            d,
            "list_modes",
            wrap_pyfunction!(anzeige::anzeige_list_modes, d)?,
        )?;
        als(
            d,
            "get_window_size",
            wrap_pyfunction!(anzeige::anzeige_get_window_size, d)?,
        )?;
        als(
            d,
            "get_driver",
            wrap_pyfunction!(anzeige::anzeige_get_driver, d)?,
        )?;
        als(
            d,
            "toggle_fullscreen",
            wrap_pyfunction!(anzeige::anzeige_toggle_fullscreen, d)?,
        )?;
        als(
            d,
            "get_active",
            wrap_pyfunction!(anzeige::anzeige_get_active, d)?,
        )?;
        als(
            d,
            "get_allow_screensaver",
            wrap_pyfunction!(anzeige::anzeige_get_allow_screensaver, d)?,
        )?;
        als(
            d,
            "set_allow_screensaver",
            wrap_pyfunction!(anzeige::anzeige_set_allow_screensaver, d)?,
        )?;
        als(
            d,
            "get_current_refresh_rate",
            wrap_pyfunction!(anzeige::anzeige_get_current_refresh_rate, d)?,
        )?;
        als(
            d,
            "get_desktop_refresh_rates",
            wrap_pyfunction!(anzeige::anzeige_get_desktop_refresh_rates, d)?,
        )?;
        als(
            d,
            "get_desktop_sizes",
            wrap_pyfunction!(anzeige::anzeige_get_desktop_sizes, d)?,
        )?;
        als(
            d,
            "get_num_displays",
            wrap_pyfunction!(anzeige::anzeige_get_num_displays, d)?,
        )?;
        als(
            d,
            "get_window_position",
            wrap_pyfunction!(anzeige::anzeige_get_window_position, d)?,
        )?;
        als(
            d,
            "set_window_position",
            wrap_pyfunction!(anzeige::anzeige_set_window_position, d)?,
        )?;
        als(
            d,
            "get_wm_info",
            wrap_pyfunction!(anzeige::anzeige_get_wm_info, d)?,
        )?;
        als(
            d,
            "gl_get_attribute",
            wrap_pyfunction!(anzeige::anzeige_gl_get_attribute, d)?,
        )?;
        als(
            d,
            "gl_set_attribute",
            wrap_pyfunction!(anzeige::anzeige_gl_set_attribute, d)?,
        )?;
        als(
            d,
            "iconify",
            wrap_pyfunction!(anzeige::anzeige_iconify, d)?,
        )?;
        als(
            d,
            "is_fullscreen",
            wrap_pyfunction!(anzeige::anzeige_is_fullscreen, d)?,
        )?;
        als(
            d,
            "is_vsync",
            wrap_pyfunction!(anzeige::anzeige_is_vsync, d)?,
        )?;
        als(
            d,
            "mode_ok",
            wrap_pyfunction!(anzeige::anzeige_mode_ok, d)?,
        )?;
        als(
            d,
            "message_box",
            wrap_pyfunction!(anzeige::anzeige_message_box, d)?,
        )?;
        als(
            d,
            "set_gamma",
            wrap_pyfunction!(anzeige::anzeige_set_gamma, d)?,
        )?;
        als(
            d,
            "set_gamma_ramp",
            wrap_pyfunction!(anzeige::anzeige_set_gamma_ramp, d)?,
        )?;
        als(
            d,
            "set_palette",
            wrap_pyfunction!(anzeige::anzeige_set_palette, d)?,
        )?;
        Ok(())
    })?;

    // `pygame.Window` gibt es **auch oben** - gemessen:
    // `pygame.Window is pygame.window.Window` ist wahr.
    // `get_grabbed_window` dagegen **nicht**; das steht nur im
    // Untermodul.
    m.add_class::<fenster::PyWindow>()?;
    untermodul(m, "window", |w| {
        w.add_class::<fenster::PyWindow>()?;
        als(
            w,
            "get_grabbed_window",
            wrap_pyfunction!(fenster::fenster_get_grabbed_window, w)?,
        )?;
        Ok(())
    })?;

    untermodul(m, "event", |e| {
        e.add_class::<ereignis::PyEvent>()?;
        // `pygame.event.EventType is pygame.event.Event` ist **wahr** -
        // gemessen. Es sind keine zwei Typen, sondern zwei Namen fuer
        // denselben; entsprechend wird hier dieselbe Klasse zweimal
        // eingetragen und keine zweite gebaut.
        e.add("EventType", e.getattr("Event")?)?;
        als(e, "get", wrap_pyfunction!(ereignis::ereignis_get, e)?)?;
        als(e, "poll", wrap_pyfunction!(ereignis::ereignis_poll, e)?)?;
        als(e, "wait", wrap_pyfunction!(ereignis::ereignis_wait, e)?)?;
        als(e, "peek", wrap_pyfunction!(ereignis::ereignis_peek, e)?)?;
        als(e, "clear", wrap_pyfunction!(ereignis::ereignis_clear, e)?)?;
        als(e, "post", wrap_pyfunction!(ereignis::ereignis_post, e)?)?;
        als(
            e,
            "set_allowed",
            wrap_pyfunction!(ereignis::ereignis_set_allowed, e)?,
        )?;
        als(
            e,
            "set_blocked",
            wrap_pyfunction!(ereignis::ereignis_set_blocked, e)?,
        )?;
        als(
            e,
            "get_blocked",
            wrap_pyfunction!(ereignis::ereignis_get_blocked, e)?,
        )?;
        als(e, "pump", wrap_pyfunction!(ereignis::ereignis_pump, e)?)?;
        als(
            e,
            "event_name",
            wrap_pyfunction!(ereignis::ereignis_event_name, e)?,
        )?;
        als(
            e,
            "custom_type",
            wrap_pyfunction!(ereignis::ereignis_custom_type, e)?,
        )?;
        als(e, "set_grab", wrap_pyfunction!(ereignis::ereignis_set_grab, e)?)?;
        als(e, "get_grab", wrap_pyfunction!(ereignis::ereignis_get_grab, e)?)?;
        Ok(())
    })?;

    untermodul(m, "key", |k| {
        // **`ScancodeWrapper` ist eine Unterklasse von `tuple`**, und das
        // laesst sich mit PyO3 nicht bauen (eine Unterklasse braucht eine
        // feste Groesse, `tuple` hat eine veraenderliche). Also steht die
        // Klasse in Python - die Begruendung samt Messung in `taste.rs`
        // bei `huelle`.
        //
        // Der Modulname beim Uebersetzen ist **`rustgame_py.key`**, damit
        // `ScancodeWrapper.__module__` stimmt; in `sys.modules` landet
        // dieses Hilfsmodul nicht - dort steht das echte `key`, das
        // `untermodul` eintraegt.
        {
            let hilfe = PyModule::from_code(
                k.py(),
                std::ffi::CString::new(include_str!("python/scancode.py"))?
                    .as_c_str(),
                std::ffi::CString::new("rustgame_py/key.py")?.as_c_str(),
                std::ffi::CString::new("rustgame_py.key")?.as_c_str(),
            )?;
            k.add("ScancodeWrapper", hilfe.getattr("ScancodeWrapper")?)?;
        }
        als(k, "_platz", wrap_pyfunction!(taste::taste_platz, k)?)?;
        als(k, "get_pressed", wrap_pyfunction!(taste::taste_get_pressed, k)?)?;
        als(k, "get_mods", wrap_pyfunction!(taste::taste_get_mods, k)?)?;
        als(k, "set_mods", wrap_pyfunction!(taste::taste_set_mods, k)?)?;
        als(k, "name", wrap_pyfunction!(taste::taste_name, k)?)?;
        als(k, "key_code", wrap_pyfunction!(taste::taste_key_code, k)?)?;
        als(k, "get_focused", wrap_pyfunction!(taste::taste_get_focused, k)?)?;
        als(k, "set_repeat", wrap_pyfunction!(taste::taste_set_repeat, k)?)?;
        als(k, "get_repeat", wrap_pyfunction!(taste::taste_get_repeat, k)?)?;
        als(
            k,
            "start_text_input",
            wrap_pyfunction!(taste::taste_start_text_input, k)?,
        )?;
        als(
            k,
            "stop_text_input",
            wrap_pyfunction!(taste::taste_stop_text_input, k)?,
        )?;
        als(
            k,
            "get_just_pressed",
            wrap_pyfunction!(taste::taste_get_just_pressed, k)?,
        )?;
        als(
            k,
            "get_just_released",
            wrap_pyfunction!(taste::taste_get_just_released, k)?,
        )?;
        als(
            k,
            "set_text_input_rect",
            wrap_pyfunction!(taste::taste_set_text_input_rect, k)?,
        )?;
        Ok(())
    })?;

    untermodul(m, "mouse", |z| {
        als(z, "get_pos", wrap_pyfunction!(maus::maus_get_pos, z)?)?;
        als(z, "get_pressed", wrap_pyfunction!(maus::maus_get_pressed, z)?)?;
        als(z, "set_pos", wrap_pyfunction!(maus::maus_set_pos, z)?)?;
        als(z, "get_rel", wrap_pyfunction!(maus::maus_get_rel, z)?)?;
        als(z, "set_visible", wrap_pyfunction!(maus::maus_set_visible, z)?)?;
        als(z, "get_visible", wrap_pyfunction!(maus::maus_get_visible, z)?)?;
        als(z, "get_focused", wrap_pyfunction!(maus::maus_get_focused, z)?)?;
        als(
            z,
            "get_just_pressed",
            wrap_pyfunction!(maus::maus_get_just_pressed, z)?,
        )?;
        als(
            z,
            "get_just_released",
            wrap_pyfunction!(maus::maus_get_just_released, z)?,
        )?;
        als(
            z,
            "get_relative_mode",
            wrap_pyfunction!(maus::maus_get_relative_mode, z)?,
        )?;
        als(
            z,
            "set_relative_mode",
            wrap_pyfunction!(maus::maus_set_relative_mode, z)?,
        )?;
        als(z, "get_cursor", wrap_pyfunction!(zeiger::maus_get_cursor, z)?)?;
        als(z, "set_cursor", wrap_pyfunction!(zeiger::maus_set_cursor, z)?)?;
        als(
            z,
            "set_system_cursor",
            wrap_pyfunction!(zeiger::maus_set_system_cursor, z)?,
        )?;
        Ok(())
    })?;

    // `pygame.math` hat drei Namen mehr, als man vermutet.
    //
    // **`VectorIterator`** ist der Typ, den `iter(Vector2())` und
    // `iter(Vector3())` zurueckgeben - gemessen derselbe fuer beide. Er ist
    // nicht aufrufbar (`TypeError: cannot create ... instances`); er steht
    // da, damit `type(it) is pygame.math.VectorIterator` geht.
    //
    // **`enable_swizzling`/`disable_swizzling`** sind leergeraeumt: sie
    // geben `None` und warnen. Gemessener Wortlaut:
    //
    // ```text
    // pygame.math.enable_swizzling() is deprecated, and its functionality
    // is removed. This function will be removed in a later version.
    // ```
    //
    // Das Tauschen von Feldern (`v.yx`) geht trotzdem - es laesst sich
    // seit Jahren nicht mehr abschalten, und genau das sagt die Warnung.
    untermodul(m, "math", |v| {
        v.add_class::<vektor::PyVector2>()?;
        v.add_class::<vektor::PyVector3>()?;
        v.add_class::<vektor::PyElementwise>()?;
        als(v, "clamp", wrap_pyfunction!(vektor::math_clamp_aussen, v)?)?;
        als(v, "lerp", wrap_pyfunction!(vektor::math_lerp_aussen, v)?)?;
        als(v, "invlerp", wrap_pyfunction!(vektor::math_invlerp_aussen, v)?)?;
        als(v, "remap", wrap_pyfunction!(vektor::math_remap_aussen, v)?)?;
        als(v, "smoothstep", wrap_pyfunction!(vektor::math_smoothstep_aussen, v)?)?;
        v.add_class::<vektor::PyVectorIterator>()?;
        // **Die fuenf Zuweisungsoperatoren stehen in Python.** PyO3
        // verlangt fuer `nb_inplace_*` den Rueckgabetyp `()` und kann
        // damit kein `NotImplemented` geben - genau das gibt pygame fuer
        // einen Operanden, der nicht passt. Die Begruendung samt Messung
        // steht in `python/vektor_anort.py`.
        {
            let hilfe = PyModule::new(v.py(), "rustgame_py.math")?;
            let code =
                std::ffi::CString::new(include_str!("python/vektor_anort.py"))?;
            v.py().run(code.as_c_str(), Some(&hilfe.dict()), None)?;
            for name in [
                "__iadd__",
                "__isub__",
                "__imul__",
                "__itruediv__",
                "__ifloordiv__",
            ] {
                let f = hilfe.getattr(name)?;
                v.py()
                    .get_type::<vektor::PyVector2>()
                    .setattr(name, &f)?;
                v.py()
                    .get_type::<vektor::PyVector3>()
                    .setattr(name, &f)?;
            }
        }
        als(
            v,
            "enable_swizzling",
            wrap_pyfunction!(vektor::math_enable_swizzling, v)?,
        )?;
        als(
            v,
            "disable_swizzling",
            wrap_pyfunction!(vektor::math_disable_swizzling, v)?,
        )?;
        Ok(())
    })?;

    untermodul(m, "mask", |k| {
        k.add_class::<maske::PyMask>()?;
        // Gemessen: `pygame.mask.MaskType is pygame.mask.Mask` ist wahr -
        // kein eigener Typ, ein Zweitname.
        k.add("MaskType", k.getattr("Mask")?)?;
        als(
            k,
            "from_surface",
            wrap_pyfunction!(maske::maske_from_surface, k)?,
        )?;
        als(
            k,
            "from_threshold",
            wrap_pyfunction!(maske::maske_from_threshold, k)?,
        )?;
        Ok(())
    })?;

    // `figur.rs` ist eine **Uebertragung** von `pygame/sprite.py`, keine
    // Huelle um `rustgame-sprite`. Der Grund ist eine einzige gemessene
    // Zeile: `self.rect.x += 1`. Python macht daraus
    // `tmp = self.rect; tmp.x = tmp.x + 1` und schreibt nichts zurueck -
    // das wirkt nur, wenn `sprite.rect` **jedes Mal dasselbe**
    // Python-Objekt ist. Ein Zugriff, der aus dem Kern ein frisches
    // `PyRect` baut, macht die haeufigste Zeile jedes pygame-Spiels
    // wirkungslos.
    untermodul(m, "sprite", figur::eintragen)?;

    // **Sechs Namen, die `sprite.py` mitimportiert.** Gemessen hat
    // `dir(pygame.sprite)` fuenfundzwanzig Eintraege, und sechs davon sind
    // Importe: `Rect`, `from_surface`, `get_ticks`, `pygame`, `types`,
    // `warn`. Zwei davon nuetzen sogar - `pygame.sprite.Rect` und
    // `pygame.sprite.get_ticks` sind kuerzere Wege zu demselben -, aber der
    // Grund, sie einzutragen, ist ein anderer: was `dir()` zeigt, ist die
    // Schnittstelle. **Ein Name zuviel ist derselbe Fehler wie einer
    // zuwenig**, und wer entscheidet, welcher Import "wichtig" ist, pflegt
    // wieder eine Liste von Hand.
    {
        let sp = m.getattr("sprite")?;
        sp.setattr("Rect", m.getattr("Rect")?)?;
        sp.setattr("from_surface", m.getattr("mask")?.getattr("from_surface")?)?;
        sp.setattr("get_ticks", m.getattr("time")?.getattr("get_ticks")?)?;
        sp.setattr("pygame", m)?;
        sp.setattr("types", py.import("types")?)?;
        sp.setattr("warn", py.import("warnings")?.getattr("warn")?)?;
        // **`AbstractGroup[Sprite]` muss gehen.** Gemessen traegt pygames
        // `AbstractGroup` ein `__class_getitem__` - genau
        // `classmethod(types.GenericAlias)` -, und damit erben es alle
        // Gruppen. Hier fehlte es bei `AbstractGroup`, `GroupSingle`,
        // `LayeredUpdates` und `LayeredDirty`; `Group` hatte es. Eine
        // getippte Anmerkung `Group[Sprite]` lief also, `GroupSingle[..]`
        // nicht. Gefunden von der Probe `Klassendoppel`.
        let generic = py
            .import("types")?
            .getattr("GenericAlias")?;
        let klassenmethode = py
            .import("builtins")?
            .getattr("classmethod")?
            .call1((generic,))?;
        sp.getattr("AbstractGroup")?
            .setattr("__class_getitem__", klassenmethode)?;
    }

    // `pygame.PixelArray` steht **oben** und im Untermodul - gemessen:
    // beide Wege gibt es.
    //
    // **Die Rust-Klasse selbst, mit angehaengtem `__init__`.**
    //
    // `pygame.PixelArray(s)` traegt sich als **Sperre** in die Flaeche ein
    // (`s.get_locks()` gibt dann `(ar,)`), und dafuer braucht es das
    // fertige Python-Objekt - in `__new__` gibt es das noch nicht. Hier
    // stand deshalb eine duenne Python-**Unterklasse**, weil PyO3 keinen
    // `tp_init`-Schlitz kennt.
    //
    // Der Schluss war falsch: eine PyO3-Klasse ist ein Heap-Typ, und dort
    // setzt `setattr(Klasse, "__init__", f)` den Schlitz mit. Siehe
    // `python/pixelarray.py`. Die Unterklasse ist weg, und mit ihr eine
    // Ebene in `__mro__`.
    {
        let klasse = py.get_type::<pixelfeld::PyPixelArray>();
        let pm = PyModule::new(py, "rustgame_py.pixelarray")?;
        let code = std::ffi::CString::new(include_str!("python/pixelarray.py"))?;
        py.run(code.as_c_str(), Some(&pm.dict()), None)?;
        klasse.setattr("__init__", pm.getattr("__init__")?)?;
        // Das Hilfsmodul traegt nur die Funktion; das echte `pixelarray`
        // traegt die Klasse.
        let echt = PyModule::new(py, "rustgame_py.pixelarray")?;
        echt.add("PixelArray", &klasse)?;
        py.import("sys")?
            .getattr("modules")?
            .set_item("rustgame_py.pixelarray", &echt)?;
        m.add("pixelarray", &echt)?;
        m.add("PixelArray", &klasse)?;
    }
    // Die vier melden **nur einen klaren Fehler** - begruendet im
    // Modulkopf von `pixelkopie.rs`. Sie stehen trotzdem da, damit ein
    // Programm uebersetzt und den Grund liest, statt an einem
    // `AttributeError` zu raten.
    // pygame hat fuer jeden dieser Typen ein **eigenes Modul**, nicht
    // nur den Namen oben. `mask_test.py` schreibt `pygame.surface.Surface`,
    // `rect_test.py` verlaesst sich auf `pygame.rect`. Gefunden hat das
    // pygames eigene Testsuite - der Modulvergleich hier fragt nach
    // Verhalten, nicht nach Namensraeumen.
    //
    // `SurfaceType`, `RectType`, `FRectType` und `MaskType` sind in
    // pygame **dasselbe Objekt** wie ihr kurzer Name (`is` sagt ja) -
    // also wird hier dieselbe Klasse zweimal eingetragen und keine
    // zweite gebaut.
    untermodul(m, "surface", |su| {
        su.add_class::<PySurface>()?;
        su.add("SurfaceType", su.getattr("Surface")?)?;
        Ok(())
    })?;
    untermodul(m, "rect", |re| {
        re.add_class::<PyRect>()?;
        re.add_class::<frechteck::PyFRect>()?;
        re.add("RectType", re.getattr("Rect")?)?;
        re.add("FRectType", re.getattr("FRect")?)?;
        Ok(())
    })?;
    // `THECOLORS` haengt an **beiden** Namen, und zwar als **dasselbe**
    // Objekt: gemessen ist
    // `pygame.color.THECOLORS is pygame.colordict.THECOLORS` wahr. Zwei
    // getrennte Woerterbuecher waeren 665 Eintraege doppelt und ein
    // `is`, das nein sagt.
    let farbnamen = farbe::thecolors(py)?;
    untermodul(m, "color", |co| {
        co.add_class::<PyColor>()?;
        co.add("THECOLORS", farbnamen.clone_ref(py))?;
        Ok(())
    })?;
    untermodul(m, "colordict", |cd| {
        cd.add("THECOLORS", farbnamen.clone_ref(py))?;
        Ok(())
    })?;
    // `pygame.constants` traegt dieselben Namen wie `pygame.locals` -
    // gemessen: beide haben 594 Eintraege.
    untermodul(m, "constants", konstanten::eintragen)?;

    untermodul(m, "pixelcopy", |pc| {
        als(
            pc,
            "surface_to_array",
            wrap_pyfunction!(pixelkopie::kopie_surface_to_array, pc)?,
        )?;
        als(
            pc,
            "array_to_surface",
            wrap_pyfunction!(pixelkopie::kopie_array_to_surface, pc)?,
        )?;
        als(pc, "map_array", wrap_pyfunction!(pixelkopie::kopie_map_array, pc)?)?;
        als(
            pc,
            "make_surface",
            wrap_pyfunction!(pixelkopie::kopie_make_surface, pc)?,
        )?;
        Ok(())
    })?;

    untermodul(m, "system", |y| {
        // **`PowerState` wird weiter unten nachgetragen**, sobald
        // `_data_classes` steht - siehe dort. Es ist ein echter
        // `@dataclass(frozen=True)`, und der Aufruf des Dekorators
        // **mitten in der Modul-Einrichtung bringt den Import zum
        // Stehen**: gemessen haengt `import rustgame_py`, wenn
        // `data_classes.py` an dieser Stelle ausgefuehrt wird. Also
        // zuerst alle Module, dann die Klasse.
        als(
            y,
            "get_cpu_instruction_sets",
            wrap_pyfunction!(rechner::rechner_get_cpu_instruction_sets, y)?,
        )?;
        als(
            y,
            "get_pref_path",
            wrap_pyfunction!(rechner::rechner_get_pref_path, y)?,
        )?;
        als(
            y,
            "get_total_ram",
            wrap_pyfunction!(rechner::rechner_get_total_ram, y)?,
        )?;
        als(
            y,
            "get_power_state",
            wrap_pyfunction!(rechner::rechner_get_power_state, y)?,
        )?;
        als(
            y,
            "get_pref_locales",
            wrap_pyfunction!(rechner::rechner_get_pref_locales, y)?,
        )?;
        Ok(())
    })?;

    untermodul(m, "scrap", |s| {
        als(s, "init", wrap_pyfunction!(ablage::ablage_init, s)?)?;
        als(s, "get_init", wrap_pyfunction!(ablage::ablage_get_init, s)?)?;
        als(s, "put_text", wrap_pyfunction!(ablage::ablage_put_text, s)?)?;
        als(s, "get_text", wrap_pyfunction!(ablage::ablage_get_text, s)?)?;
        als(s, "has_text", wrap_pyfunction!(ablage::ablage_has_text, s)?)?;
        als(s, "contains", wrap_pyfunction!(ablage::ablage_contains, s)?)?;
        als(s, "get_types", wrap_pyfunction!(ablage::ablage_get_types, s)?)?;
        als(s, "get", wrap_pyfunction!(ablage::ablage_get, s)?)?;
        als(s, "put", wrap_pyfunction!(ablage::ablage_put, s)?)?;
        als(s, "lost", wrap_pyfunction!(ablage::ablage_lost, s)?)?;
        als(s, "set_mode", wrap_pyfunction!(ablage::ablage_set_mode, s)?)?;
        Ok(())
    })?;

    untermodul(m, "joystick", |j| {
        // **`JoystickType` ist die Klasse, `Joystick` die Fabrik.**
        //
        // Hier stand ein `add_class` fuer `Joystick` und daneben
        // `JoystickType = Joystick` mit dem Vermerk, ein zweiter, leerer
        // Typ waere eine Nachahmung ohne Nutzen. Der zweite Typ ist aber
        // nicht leer: gemessen traegt `JoystickType` **neunzehn** Namen,
        // und `Joystick` ist eine `builtin_function_or_method`. Siehe
        // `hebel.rs`.
        j.add_class::<hebel::PyJoystick>()?;
        j.add("JoystickType", j.getattr("Joystick")?)?;
        // **Dasselbe Objekt** wie `pygame.Joystick`, nicht ein zweites mit
        // demselben Namen: `wrap_pyfunction!` baut bei jedem Aufruf ein
        // neues, und `joystick_test.test_Joystick_alias` prueft `is`.
        j.add("Joystick", m.getattr("Joystick")?)?;
        als(j, "init", wrap_pyfunction!(hebel::hebel_init, j)?)?;
        als(j, "quit", wrap_pyfunction!(hebel::hebel_quit, j)?)?;
        als(j, "get_init", wrap_pyfunction!(hebel::hebel_get_init, j)?)?;
        als(j, "get_count", wrap_pyfunction!(hebel::hebel_get_count, j)?)?;
        Ok(())
    })?;

    // ## Warum ein mitimportierter Name hier dazugehoert
    //
    // `pygame.cursors.pygame`, `pygame.sprite.types`, `pygame.midi.atexit`:
    // das sind keine Schnittstellen, sondern Importe, die in `dir()`
    // mitlaufen. Trotzdem stehen sie hier, und zwar aus demselben Grund,
    // aus dem `_sdl2.video` seinen `messagebox` **verloren** hat: was
    // `dir()` zeigt, ist, was ein Programm sieht. **Ein Name zuviel ist
    // derselbe Fehler wie einer zuwenig** - und wer entscheidet, welcher
    // Import "wichtig" ist, pflegt wieder eine Liste von Hand.
    //
    // Der Name `pygame` zeigt dabei auf **dieses** Paket. Wer rustgame als
    // Ersatz einsetzt, legt es unter dem alten Namen ab, und dann stimmt
    // auch der Text; wer es unter seinem eigenen Namen nimmt, bekommt das
    // Paket, das er benutzt - beides richtiger als ein fehlender Name.
    untermodul(m, "cursors", |c| {
        c.add_class::<zeiger::PyCursor>()?;
        zeiger::daten_eintragen(c)?;
        c.setattr("pygame", m)?;
        Ok(())
    })?;

    // `pygame.mixer.music` **ist** `pygame.mixer_music` - gemessen:
    // `pygame.mixer.music is pygame.mixer_music` gibt `True`, und
    // `music.__name__` ist `'pygame.mixer_music'`. Also erst das eigene
    // Modul, dann `mixer`, dann die Verknuepfung.
    //
    // **Kein** Eintrag unter `sys.modules["rustgame_py.mixer.music"]`:
    // gemessen ist `'pygame.mixer.music' in sys.modules` falsch, und
    // `import pygame.mixer.music` scheitert mit
    // `ModuleNotFoundError: ... 'pygame.mixer' is not a package`.
    // `pygame.mixer.music.play()` und `from pygame.mixer import music`
    // gehen trotzdem beide.
    untermodul(m, "mixer_music", ton::eintragen_music)?;
    untermodul(m, "mixer", ton::eintragen)?;
    m.getattr("mixer")?.setattr("music", m.getattr("mixer_music")?)?;

    // `pygame.freetype` - das zweite Schriftmodul, siehe `freitext.rs`.
    untermodul(m, "freetype", |f| {
        use freitext as ft;
        f.add_class::<ft::PyFreetypeFont>()?;
        f.add("STYLE_NORMAL", ft::STYLE_NORMAL)?;
        f.add("STYLE_STRONG", ft::STYLE_STRONG)?;
        f.add("STYLE_OBLIQUE", ft::STYLE_OBLIQUE)?;
        f.add("STYLE_UNDERLINE", ft::STYLE_UNDERLINE)?;
        f.add("STYLE_WIDE", ft::STYLE_WIDE)?;
        f.add("STYLE_DEFAULT", ft::STYLE_DEFAULT)?;
        als(f, "init", wrap_pyfunction!(ft::ft_init, f)?)?;
        als(f, "quit", wrap_pyfunction!(ft::ft_quit, f)?)?;
        als(f, "get_init", wrap_pyfunction!(ft::ft_get_init, f)?)?;
        als(f, "was_init", wrap_pyfunction!(ft::ft_was_init, f)?)?;
        als(f, "get_error", wrap_pyfunction!(ft::ft_get_error, f)?)?;
        als(f, "SysFont", wrap_pyfunction!(ft::ft_sysfont, f)?)?;
        als(f, "get_version", wrap_pyfunction!(ft::ft_get_version, f)?)?;
        als(
            f,
            "get_cache_size",
            wrap_pyfunction!(ft::ft_get_cache_size, f)?,
        )?;
        als(
            f,
            "get_default_font",
            wrap_pyfunction!(ft::ft_get_default_font, f)?,
        )?;
        als(
            f,
            "get_default_resolution",
            wrap_pyfunction!(ft::ft_get_default_resolution, f)?,
        )?;
        als(
            f,
            "set_default_resolution",
            wrap_pyfunction!(ft::ft_set_default_resolution, f)?,
        )?;
        // Die Schriftsuche teilt sich `pygame.freetype` mit
        // `pygame.font` - dieselben Dateien, dieselbe Antwort.
        als(f, "get_fonts", wrap_pyfunction!(text::schrift_get_fonts, f)?)?;
        als(f, "match_font", wrap_pyfunction!(text::schrift_match_font, f)?)?;
        // `_internal_mod_init` gibt es nur, weil `ftfont.py` es einfuehrt.
        // In pygame faehrt es das Modul ohne Ereignisschlange hoch; hier
        // ist es dasselbe wie `init`.
        als(f, "_internal_mod_init", wrap_pyfunction!(ft::ft_init, f)?)?;
        Ok(())
    })?;

    // **`pygame._freetype` ist ein eigener Kasten** - und das war hier
    // einmal anders.
    //
    // Erst stand hier `m.add("_freetype", m.getattr("freetype")?)`: ein
    // Name, zwei Zeiger auf dasselbe Modul. Naheliegend, denn `freetype.py`
    // ist bei pygame wirklich nur eine Huelle um `_freetype`. Gemessen sind
    // es aber **zwei verschiedene Namensraeume**, und sie unterscheiden sich
    // in sieben Namen:
    //
    // ```text
    // nur freetype : SysFont, get_fonts, match_font
    // nur _freetype: BBOX_EXACT, BBOX_EXACT_GRIDFIT, BBOX_PIXEL,
    //                BBOX_PIXEL_GRIDFIT
    // ```
    //
    // `freetype.py` holt seine Namen also **einzeln** von `_freetype` und
    // nicht mit `import *` - sonst waeren die vier `BBOX_*` dort auch zu
    // sehen. Und die drei Suchfunktionen legt es selbst dazu, denn die
    // Schriftsuche ist Python und steht in `sysfont.py`.
    //
    // `Font` ist auf beiden Seiten **dieselbe Klasse** (gemessen:
    // `freetype.Font is _freetype.Font`), darum wird hier das Objekt
    // kopiert und keine zweite Klasse gebaut.
    //
    // Die vier `BBOX_*`-Zahlen sagen, wie `Font.get_rect` die Glyphenbox
    // bestimmt: `EXACT` 0 rechnet in Unterpixeln, `PIXEL` 2 rundet auf
    // ganze Pixel, und die `_GRIDFIT`-Fassungen 1 und 3 rasten vorher
    // aufs Pixelgitter ein. Sie sind gemessen, nicht hergeleitet.
    {
        let ft = m.getattr("freetype")?;
        let f = PyModule::new(py, "rustgame_py._freetype")?;
        f.setattr(
            "__doc__",
            "`pygame._freetype` - der Kasten unter `pygame.freetype`.",
        )?;
        // Die drei Namen, die **nur** `freetype` hat.
        let nur_huelle = ["SysFont", "get_fonts", "match_font"];
        for name in ft.dir()?.iter() {
            let Ok(name) = name.extract::<String>() else {
                continue;
            };
            if name.starts_with("__") || nur_huelle.contains(&name.as_str()) {
                continue;
            }
            if let Ok(wert) = ft.getattr(name.as_str()) {
                f.setattr(name.as_str(), wert)?;
            }
        }
        for (name, wert) in [
            ("BBOX_EXACT", 0i32),
            ("BBOX_EXACT_GRIDFIT", 1),
            ("BBOX_PIXEL", 2),
            ("BBOX_PIXEL_GRIDFIT", 3),
        ] {
            f.setattr(name, wert)?;
        }
        py.import("sys")?
            .getattr("modules")?
            .set_item("rustgame_py._freetype", &f)?;
        m.add("_freetype", &f)?;
    }

    // `pygame.camera` - **ohne Kamera**, aber mit klarer Auskunft.
    // `camera_test.py` ueberspringt sich selbst, sobald `list_cameras()`
    // leer ist; ohne das Modul laedt die Datei gar nicht erst.
    {
        let cam = PyModule::from_code(
            py,
            std::ffi::CString::new(include_str!("python/camera.py"))?.as_c_str(),
            std::ffi::CString::new("rustgame_py/camera.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py.camera")?.as_c_str(),
        )?;
        py.import("sys")?
            .getattr("modules")?
            .set_item("rustgame_py.camera", &cam)?;
        m.add("camera", &cam)?;
    }

    // `pygame._debug` samt `pygame.print_debug_info` - **woertlich
    // uebertragen**, siehe `python/_debug.py`.
    //
    // Die Funktion fiel durch zwei Luecken im Messgeraet: `pygame_tests.py`
    // hatte `debug_test.py` nie in seiner (abgetippten) Liste, und
    // `api_coverage.py` fragte nie nach den Namen des Hauptmoduls. Beide
    // sind geschlossen.
    {
        let dbg = PyModule::from_code(
            py,
            std::ffi::CString::new(include_str!("python/_debug.py"))?.as_c_str(),
            std::ffi::CString::new("rustgame_py/_debug.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py._debug")?.as_c_str(),
        )?;
        py.import("sys")?
            .getattr("modules")?
            .set_item("rustgame_py._debug", &dbg)?;
        m.add("_debug", &dbg)?;
        // pygame holt den einen Namen ins Hauptmodul hoch und laesst das
        // Modul stehen. Genau so hier.
        m.add("print_debug_info", dbg.getattr("print_debug_info")?)?;
    }

    // **Die kleinen Untermodule, die `dir(pygame)` zeigt.**
    //
    // Gefunden hat sie der **neunte blinde Fleck**: `api_coverage.py` hat
    // seine Modulliste von Hand gepflegt, und `pygame` bringt sechzig
    // Untermodule mit. Neun davon waren hier nicht einmal importierbar.
    // Seit die Liste gelesen statt genannt wird, stehen sie in der
    // Tabelle - und jetzt auch hier.
    // **Das Modul `_data_classes` wird festgehalten**, weil
    // `system.PowerState` es gleich braucht - und ein
    // `py.import("rustgame_py._data_classes")` hier **nicht** geht:
    // Python importiert bei einem Punktnamen erst das Elternmodul, und
    // `rustgame_py` steht waehrend seiner eigenen Einrichtung noch nicht
    // in `sys.modules`. Der Import faengt dann von vorn an und wartet auf
    // sich selbst - gemessen hing `import rustgame_py`, und der
    // Faden-Auszug zeigte zwei verschachtelte `create_module`. Dieselbe
    // Falle wie bei `_sdl2`, siehe `python/sdl2_init.py`.
    let mut datenklassen: Option<Bound<'_, PyModule>> = None;
    for (name, datei, quelle) in [
        ("macosx", "rustgame_py/macosx.py", include_str!("python/macosx.py")),
        (
            "__pyinstaller",
            "rustgame_py/__pyinstaller/__init__.py",
            include_str!("python/pyinstaller.py"),
        ),
        (
            "_data_classes",
            "rustgame_py/_data_classes.py",
            include_str!("python/data_classes.py"),
        ),
        ("_camera", "rustgame_py/_camera.py", include_str!("python/camera_c.py")),
        ("pypm", "rustgame_py/pypm.py", include_str!("python/pypm.py")),
    ] {
        let voll = format!("rustgame_py.{name}");
        let modul = PyModule::from_code(
            py,
            std::ffi::CString::new(quelle)?.as_c_str(),
            std::ffi::CString::new(datei)?.as_c_str(),
            std::ffi::CString::new(voll.as_str())?.as_c_str(),
        )?;
        py.import("sys")?
            .getattr("modules")?
            .set_item(voll.as_str(), &modul)?;
        if name == "_data_classes" {
            datenklassen = Some(modul.clone());
        }
        m.add(name, &modul)?;
    }

    // **Jetzt `system.PowerState`** - die Klasse wohnt in
    // `_data_classes`, und `pygame.system.PowerState is
    // pygame._data_classes.PowerState` ist gemessen wahr. Sie wird hier
    // nachgetragen und nicht im `system`-Block, weil der Dekorator
    // `@dataclass` mitten in der Modul-Einrichtung den Import zum Stehen
    // bringt - gemessen, siehe den Vermerk dort.
    if let Some(dc) = datenklassen {
        m.getattr("system")?
            .setattr("PowerState", dc.getattr("PowerState")?)?;
    }

    // **Zwei Pakete ohne einen einzigen Namen** - und das ist gemessen:
    // `dir(pygame.examples)` und `dir(pygame.__briefcase)` sind beide
    // leer. Was sie tragen, sind **Unterpakete**: bei `examples` die
    // Beispielprogramme (`aliens`, `aacircle`, ...), bei `__briefcase`
    // ein Haken fuer das Packwerkzeug gleichen Namens.
    //
    // rustgame liefert weder Beispielprogramme noch einen
    // Briefcase-Haken mit. Die Pakete stehen trotzdem da, damit
    // `import pygame.examples` nicht an einem `ModuleNotFoundError`
    // scheitert - dieselbe Ueberlegung wie bei `pygame.docs`, das
    // ehrlich `False` sagt.
    //
    // `pygame.__briefcase.pygame_ce` scheitert bei pygame uebrigens
    // ebenfalls, solange `briefcase` nicht installiert ist. **Dieselbe
    // Antwort ist keine Luecke.**
    for name in ["examples", "__briefcase"] {
        let voll = format!("rustgame_py.{name}");
        let leer = PyModule::new(py, voll.as_str())?;
        leer.add(
            "__doc__",
            "Dieses Paket traegt keine Namen - rustgame liefert den Inhalt \
nicht mit. Siehe lib.rs.",
        )?;
        py.import("sys")?
            .getattr("modules")?
            .set_item(voll.as_str(), &leer)?;
        m.add(name, &leer)?;
    }

    // `pygame.newbuffer` - das Puffer-Protokoll **von Python aus**.
    //
    // Zwei Namen, und sie waren die letzten zwei der ganzen Schnittstelle:
    // `Py_buffer` und `BufferMixin`. pygame baut damit in seiner eigenen
    // Testsuite Felder mit jeder Form und Byteordnung
    // (`tests/test_utils/buftools.py`) - es ist also das Werkzeug, mit dem
    // pygame sein Puffer-Verhalten prueft, und kein Beiwerk.
    untermodul(m, "newbuffer", |nb| {
        nb.add_class::<neuerpuffer::PyPyBuffer>()?;
        nb.add_class::<neuerpuffer::PyBufferMixin>()?;
        // Die `PyBUF_*`-Zahlen stehen bei pygame **auch hier**, nicht nur
        // in `puffer.rs`. Gemessen sind es achtzehn plus
        // `PyBUFFER_SIZEOF` - und die Gruppen (`PyBUF_FULL`,
        // `PyBUF_RECORDS`) sind Summen, die man nicht erraten kann.
        for (name, wert) in [
            ("PyBUFFER_SIZEOF", 80),
            ("PyBUF_SIMPLE", 0),
            ("PyBUF_WRITABLE", 1),
            ("PyBUF_FORMAT", 4),
            ("PyBUF_ND", 8),
            ("PyBUF_STRIDES", 24),
            ("PyBUF_C_CONTIGUOUS", 56),
            ("PyBUF_F_CONTIGUOUS", 88),
            ("PyBUF_ANY_CONTIGUOUS", 152),
            ("PyBUF_INDIRECT", 280),
            ("PyBUF_CONTIG", 9),
            ("PyBUF_CONTIG_RO", 8),
            ("PyBUF_STRIDED", 25),
            ("PyBUF_STRIDED_RO", 24),
            ("PyBUF_RECORDS", 29),
            ("PyBUF_RECORDS_RO", 28),
            ("PyBUF_FULL", 285),
            ("PyBUF_FULL_RO", 284),
        ] {
            nb.add(name, wert)?;
        }
        Ok(())
    })?;

    // `pygame.docs` - sagt, ob die erzeugte Doku mitgeliefert wurde.
    //
    // Hier **nein**, und das ist die Aufgabe des Moduls und kein Mangel:
    // `docs_test.test_doc_import_works` will nur, dass `PKG_DIR` und
    // `has_local_docs` da sind.
    {
        let docs = PyModule::from_code(
            py,
            std::ffi::CString::new(include_str!("python/docs.py"))?.as_c_str(),
            std::ffi::CString::new("rustgame_py/docs.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py.docs")?.as_c_str(),
        )?;
        py.import("sys")?
            .getattr("modules")?
            .set_item("rustgame_py.docs", &docs)?;
        m.add("docs", &docs)?;
    }

    // `pygame.surflock` - **ein Modul ohne einen einzigen Namen.**
    //
    // Gemessen: `[n for n in dir(pygame.surflock) if not n.startswith("_")]`
    // ist eine leere Liste. Die Sperrarbeit steckt in der C-Kiste und ist
    // von Python aus nur ueber `Surface.lock()` und Co. zu erreichen - die
    // es hier gibt. Das Modul steht trotzdem da, weil `dir(pygame)` es
    // zeigt und ein Programm es anfassen darf.
    untermodul(m, "surflock", |_s| Ok(()))?;

    // `pygame.ftfont` - `pygame.font` auf dem zweiten Schriftsystem.
    //
    // Es kostet eine Datei und faehrt dafuer **die ganze `font_test.py`
    // ein zweites Mal**: `ftfont_test.py` tauscht nur das Modul aus und
    // baut sich aus jeder Testklasse eine eigene.
    {
        let ff = PyModule::from_code(
            py,
            std::ffi::CString::new(include_str!("python/ftfont.py"))?.as_c_str(),
            std::ffi::CString::new("rustgame_py/ftfont.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py.ftfont")?.as_c_str(),
        )?;
        py.import("sys")?
            .getattr("modules")?
            .set_item("rustgame_py.ftfont", &ff)?;
        m.add("ftfont", &ff)?;
    }

    // `pygame.surfarray` und `pygame.sndarray` - die numpy-Bruecken.
    //
    // **Wortgleich uebertragen**, beide; siehe die Koepfe der Dateien. Sie
    // sind in pygame selbst reines Python und rechnen nichts - sie reichen
    // zwischen `pixelcopy` beziehungsweise `mixer.Sound` und numpy hin und
    // her. Damit gab es sie in rustgame erst, als es beides gab.
    //
    // ## Warum sie erst auf Zuruf entstehen
    //
    // `surfarray.py` schreibt in seiner ersten Zeile
    // `from rustgame_py.pixelcopy import ...`. Wer das **hier** ausfuehrt,
    // laesst Python mitten im Aufbau von `rustgame_py` nach
    // `rustgame_py.pixelcopy` suchen - und der Aufbau ist noch nicht
    // fertig, also findet Python das halbe Paket nicht und faengt von
    // vorne an. Der erste Versuch hing fest, ohne eine Zeile Ausgabe.
    //
    // Also derselbe Weg wie bei `_sprite`: ein Sucher in `sys.meta_path`,
    // der die Datei baut, wenn jemand sie importiert. Bis dahin steht am
    // Modul ein Stellvertreter, der beim ersten Zugriff nachlaedt.
    //
    // ## Ohne numpy
    //
    // Dann scheitert der Import, und der Stellvertreter sagt es mit einem
    // `NotImplementedError` - genau wie pygames `MissingModule`. Ein
    // Attribut, das sich erklaert, ist besser als ein `AttributeError`
    // und viel besser als ein Importfehler, der das ganze Paket mitnimmt.
    {
        let quelle = r#"
"""Haengt `rustgame_py.surfarray` und `.sndarray` als Sucher ein."""

import importlib
import importlib.util
import sys

QUELLEN = {}


class _Lader:
    """Fuehrt den mitgelieferten Text im neuen Modul aus."""

    def __init__(self, text, pfad):
        self._text = text
        self._pfad = pfad

    def create_module(self, spec):
        return None

    def exec_module(self, modul):
        modul.__file__ = self._pfad
        exec(compile(self._text, self._pfad, "exec"), modul.__dict__)
        # **Pygames Satz darueber - hier und nicht beim Modulaufbau.**
        # `surfarray` und `sndarray` laden erst beim ersten Zugriff, und
        # zwar auf **zwei** Wegen: ueber den Stellvertreter `Spaet` und
        # ueber ein blankes `import rustgame_py.surfarray`. Ein Haken im
        # Stellvertreter faengt nur den ersten; hier laufen beide
        # durch. Gefunden hat das die Probe `Dokumentation`, nachdem ihre
        # Untergrenze berichtigt war.
        try:
            import rustgame_py

            kurz = modul.__name__.rpartition(".")[2]
            satz = rustgame_py._moduldoku.get(kurz)
            if satz is not None:
                modul.__doc__ = satz
        except Exception:                            # noqa: BLE001
            pass


class _FeldSucher:
    """Faengt `<paket>.surfarray` und `<paket>.sndarray` ab.

    Wie bei `_sprite` wird nicht der Name geprueft, sondern **wer der
    Elternteil ist**: wer rustgame als Ersatz einsetzt, legt es unter
    `pygame` ab, und dann sucht Python auch dort.
    """

    def find_spec(self, name, path=None, target=None):
        eltern, _, letzt = name.rpartition(".")
        if letzt not in QUELLEN or not eltern:
            return None
        import rustgame_py

        if sys.modules.get(eltern) is not rustgame_py:
            return None
        text, pfad = QUELLEN[letzt]
        return importlib.util.spec_from_loader(name, _Lader(text, pfad))


class Spaet:
    """Ein Modul, das beim ersten Zugriff nachgeladen wird.

    Es steht als Attribut am Hauptmodul, damit `pygame.surfarray` kein
    `AttributeError` ist, ohne dass die Datei schon beim Laden von
    `rustgame_py` laufen muesste. Sobald jemand etwas davon will, wird
    wirklich importiert - und ab dann ersetzt Python das Attribut von
    selbst durch das echte Modul.
    """

    def __init__(self, name):
        self._kurz = name
        self.__name__ = "rustgame_py." + name

    def _laden(self):
        try:
            # Den Satz legt der Lader auf (siehe `_Lader.exec_module`) -
            # er faengt auch ein blankes `import rustgame_py.surfarray`.
            return importlib.import_module(self.__name__)
        except ImportError as e:
            raise NotImplementedError(
                f"{self._kurz} module not available ({e})"
            ) from None

    def __getattr__(self, was):
        if was.startswith("_"):
            raise AttributeError(was)
        return getattr(self._laden(), was)

    def __dir__(self):
        """Wie beim echten Modul - **und das ist nicht Schmuck**.

        `api_coverage.py` fragt die Huelle mit `dir()` nach ihren Namen.
        Ohne diese Methode stand `surfarray` dort mit **0 von 27** da,
        obwohl jede einzelne Funktion laeuft: der Stellvertreter hatte
        nichts zu zeigen, und eine Frage, die ins Leere geht, klingt wie
        ein Mangel.
        """
        try:
            return dir(self._laden())
        except NotImplementedError:
            return []

    def __bool__(self):
        try:
            self._laden()
        except NotImplementedError:
            return False
        return True

    def __repr__(self):
        return f"<module {self.__name__!r} (noch nicht geladen)>"


if not any(type(f).__name__ == "_FeldSucher" for f in sys.meta_path):
    sys.meta_path.insert(0, _FeldSucher())
"#;
        let sucher = PyModule::from_code(
            py,
            std::ffi::CString::new(quelle)?.as_c_str(),
            std::ffi::CString::new("rustgame_py/_feld_sucher.py")?.as_c_str(),
            std::ffi::CString::new("rustgame_py._feld_sucher")?.as_c_str(),
        )?;
        let quellen = sucher.getattr("QUELLEN")?;
        for (name, text) in [
            ("surfarray", include_str!("python/surfarray.py")),
            ("sndarray", include_str!("python/sndarray.py")),
        ] {
            quellen.set_item(name, (text, format!("rustgame_py/{name}.py")))?;
            m.add(name, sucher.getattr("Spaet")?.call1((name,))?)?;
        }
    }

    // `pygame.gfxdraw` - die Grundformen aus SDL_gfx.
    untermodul(m, "gfxdraw", |g| {
        use gfxzeichnen as gz;
        for (name, f) in [
            ("pixel", wrap_pyfunction!(gz::gfx_pixel, g)?),
            ("hline", wrap_pyfunction!(gz::gfx_hline, g)?),
            ("vline", wrap_pyfunction!(gz::gfx_vline, g)?),
            ("line", wrap_pyfunction!(gz::gfx_line, g)?),
            ("rectangle", wrap_pyfunction!(gz::gfx_rectangle, g)?),
            ("box", wrap_pyfunction!(gz::gfx_box, g)?),
            ("circle", wrap_pyfunction!(gz::gfx_circle, g)?),
            ("aacircle", wrap_pyfunction!(gz::gfx_aacircle, g)?),
            ("filled_circle", wrap_pyfunction!(gz::gfx_filled_circle, g)?),
            ("ellipse", wrap_pyfunction!(gz::gfx_ellipse, g)?),
            ("aaellipse", wrap_pyfunction!(gz::gfx_aaellipse, g)?),
            ("filled_ellipse", wrap_pyfunction!(gz::gfx_filled_ellipse, g)?),
            ("arc", wrap_pyfunction!(gz::gfx_arc, g)?),
            ("pie", wrap_pyfunction!(gz::gfx_pie, g)?),
            ("trigon", wrap_pyfunction!(gz::gfx_trigon, g)?),
            ("aatrigon", wrap_pyfunction!(gz::gfx_aatrigon, g)?),
            ("filled_trigon", wrap_pyfunction!(gz::gfx_filled_trigon, g)?),
            ("polygon", wrap_pyfunction!(gz::gfx_polygon, g)?),
            ("aapolygon", wrap_pyfunction!(gz::gfx_aapolygon, g)?),
            ("filled_polygon", wrap_pyfunction!(gz::gfx_filled_polygon, g)?),
            ("textured_polygon", wrap_pyfunction!(gz::gfx_textured_polygon, g)?),
            ("bezier", wrap_pyfunction!(gz::gfx_bezier, g)?),
        ] {
            als(g, name, f)?;
        }
        Ok(())
    })?;

    // `pygame.geometry` - `Circle` und `Line`, seit pygame-ce 2.5.
    untermodul(m, "geometry", |g| {
        g.add_class::<geometrie::PyCircle>()?;
        g.add_class::<geometrie::PyLine>()?;
        // **`__init__` in Python** - dieselbe Trennung wie bei `Surface`
        // und `Color`: `__new__` gibt den Nullzustand, `__init__` liest
        // die Argumente. Siehe `python/nullbau.py`.
        {
            let hilfe = PyModule::new(g.py(), "rustgame_py.geometry")?;
            let code = std::ffi::CString::new(include_str!("python/nullbau.py"))?;
            g.py()
                .run(code.as_c_str(), Some(&hilfe.dict()), None)?;
            let f = hilfe.getattr("__init__")?;
            g.py()
                .get_type::<geometrie::PyCircle>()
                .setattr("__init__", &f)?;
            g.py().get_type::<geometrie::PyLine>().setattr("__init__", &f)?;
        }
        Ok(())
    })?;

    // `pygame.font.UCS4` - **1**: die Schrift nimmt Zeichen oberhalb von
    // U+FFFF. Bei pygame haengt das daran, ob SDL_ttf mit vier Byte je
    // Zeichen gebaut wurde; hier ist Text immer Rusts `str`, also UTF-8,
    // und damit gibt es die Frage nicht - die Antwort ist immer ja.
    untermodul(m, "font", |f| {
        f.add("UCS4", 1i32)?;
        f.add_class::<text::PyFont>()?;
        // `pygame.font.FontType` ist derselbe Typ wie `Font` - ein alter
        // Zweitname, den pygames eigene Tests benutzen:
        // `font_test.test_SysFont` prueft `isinstance(o, pygame_font.FontType)`
        // und faellt sonst schon beim Zugriff.
        f.add("FontType", f.py().get_type::<text::PyFont>())?;
        als(f, "init", wrap_pyfunction!(text::schrift_init, f)?)?;
        als(f, "quit", wrap_pyfunction!(text::schrift_quit, f)?)?;
        als(f, "get_init", wrap_pyfunction!(text::schrift_get_init, f)?)?;
        als(
            f,
            "get_default_font",
            wrap_pyfunction!(text::schrift_get_default_font, f)?,
        )?;
        als(f, "get_fonts", wrap_pyfunction!(text::schrift_get_fonts, f)?)?;
        // Kein pygame-Name, darum mit Unterstrich: `sysfont.initsysfonts`
        // holt darueber die Tabelle, die pygame in `Sysfonts` fuehrt.
        als(
            f,
            "_sysfont_tabelle",
            wrap_pyfunction!(text::schrift_sysfont_tabelle, f)?,
        )?;
        als(f, "match_font", wrap_pyfunction!(text::schrift_match_font, f)?)?;
        als(f, "SysFont", wrap_pyfunction!(text::schrift_sysfont, f)?)?;
        als(
            f,
            "get_sdl_ttf_version",
            wrap_pyfunction!(text::schrift_get_sdl_ttf_version, f)?,
        )?;
        Ok(())
    })?;

    // ----- `__all__`: was `from pygame.x import *` bringt --------------
    //
    // **pygame hat fast nirgends ein `__all__`, rustgame hatte ueberall
    // eines.** PyO3 traegt jeden Namen, der mit `PyModule::add` kommt,
    // zusaetzlich in `__all__` ein; die Python-Teile dieser Huelle
    // schreiben es selbst. Gemessen haben **51 von 60** Untermodulen ein
    // anderes `__all__` als pygame, und bei **14** brachte
    // `from X import *` deshalb etwas anderes:
    //
    // ```text
    // camera    fehlt ABC, abstractmethod, os, platform, sys, warnings
    // cursors   fehlt pygame
    // _sprite   fehlt warnings, zuviel OrderedUpdates
    // freetype  zuviel SysFont
    // ftfont    zuviel FontType
    // ```
    //
    // Die ersten drei waren Namen, die mit `setattr` kamen und damit nicht
    // in `__all__` landeten - im Woerterbuch standen sie. Die letzten zwei
    // sind der umgekehrte Fall: dort hat **pygame** ein eigenes `__all__`,
    // und es ist **schmaler** als sein Woerterbuch.
    //
    // ## Die Regel, gemessen
    //
    // Genau **elf** Module geben sich bei pygame selbst ein `__all__`:
    // `constants`, `freetype`, `ftfont`, `locals`, `macosx`, `midi`,
    // `pkgdata`, `sndarray`, `surfarray`, `typing`, `version`. Bei allen
    // anderen - und beim Hauptmodul - gibt es keines, und `import *`
    // nimmt dann jeden Namen ohne Unterstrich aus dem Woerterbuch.
    //
    // Hier wird darum fuer **jedes** Modul `__all__` auf die
    // oeffentlichen Namen gesetzt, abzueglich der Namen, die pygames
    // eigenes `__all__` auslaesst. Diese Ausnahmen sind keine
    // Entdeckungsliste, sondern **gemessene Unterschiede** - und die Probe
    // `Sternimport` in `vergleich_module.py` vergleicht alle sechzig
    // Module, damit sie nicht veralten.
    {
        // Was pygames eigenes `__all__` **auslaesst**, obwohl es im
        // Woerterbuch steht. Gemessen, Modul fuer Modul.
        let ausser: &[(&str, &[&str])] = &[
            ("freetype", &["SysFont"]),
            ("ftfont", &["FontType", "draw"]),
            ("locals", &["pygame"]),
            ("macosx", &["os", "platform", "sys"]),
            ("midi", &["atexit", "math", "pygame"]),
            (
                "pkgdata",
                &["files", "os", "resource_exists", "resource_stream", "sys"],
            ),
            ("sndarray", &["mixer", "numpy", "warnings"]),
            (
                "surfarray",
                &[
                    "numpy",
                    "numpy_array",
                    "numpy_empty",
                    "numpy_floats",
                    "numpy_ndarray",
                    "numpy_uint32",
                    "pix_make_surface",
                    "pix_map_array",
                ],
            ),
            ("typing", &["TypeAlias"]),
            ("version", &["PygameVersion", "SDLVersion", "SoftwareVersion",
                          "get_sdl_version"]),
        ];

        let setzen = |modul: &Bound<'_, PyAny>, weg: &[&str]| -> PyResult<()> {
            let mut namen: Vec<String> = modul
                .dir()?
                .iter()
                .filter_map(|o| o.extract::<String>().ok())
                .filter(|n| !n.starts_with('_') && !weg.contains(&n.as_str()))
                .collect();
            namen.sort();
            namen.dedup();
            modul.setattr("__all__", namen)?;
            Ok(())
        };

        // Das Hauptmodul zuerst - pygame hat auch dort keines.
        setzen(m.as_any(), &[])?;
        let module = py.import("sys")?.getattr("modules")?;
        let namen: Vec<String> = m
            .dir()?
            .iter()
            .filter_map(|o| o.extract::<String>().ok())
            .collect();
        for name in namen {
            let Ok(kind) = m.getattr(name.as_str()) else {
                continue;
            };
            // Nur echte Untermodule - `__name__` ist der Weg dorthin.
            let voll = format!("rustgame_py.{name}");
            if !module.contains(&voll).unwrap_or(false) {
                continue;
            }
            let weg = ausser
                .iter()
                .find(|(n, _)| *n == name.as_str())
                .map(|(_, w)| *w)
                .unwrap_or(&[]);
            setzen(&kind, weg)?;
        }
    }

    // Die Tabelle **auch nach Python**, fuer die Dinge, die erst
    // spaeter entstehen: die faulen Module (`surfarray`, `sndarray`),
    // `ftfont.Font` und `_camera.Camera`. Beide Namen tragen einen
    // Unterstrich - pygame hat sie nicht.
    {
        let mdoku = pyo3::types::PyDict::new(py);
        for (name, satz) in moduldoku::MODULDOKU {
            mdoku.set_item(*name, *satz)?;
        }
        m.add("_moduldoku", mdoku)?;
        let kdoku = pyo3::types::PyDict::new(py);
        for (modul, klasse, satz) in moduldoku::KLASSENDOKU {
            kdoku.set_item(format!("{modul}.{klasse}"), *satz)?;
        }
        m.add("_klassendoku", kdoku)?;
    }

    // ----- Die Modul-Docstrings ------------------------------------------
    //
    // **`__doc__` ist Schnittstelle.** `help(pygame.draw)` zeigt ihn, und
    // gemessen fehlte er bei **36** von 60 Modulen ganz; bei den
    // eingebetteten Python-Modulen stand deutsche Prosa darin. Beides ist
    // dieselbe Sorte Fehler wie ein deutscher Fehlertext - nur sieht ihn
    // niemand, der nur Programme laufen laesst.
    //
    // Der deutsche Vermerk bleibt im Quelltext; hier wird pygames Satz
    // darueber gelegt, aus der erzeugten Tabelle `moduldoku::MODULDOKU`
    // (`tools/gen_moduldoku.py`). **Nur Module**: an einer Methode ist
    // `__doc__` nicht schreibbar (`attribute '__doc__' of
    // 'method_descriptor' objects is not writable`) - das ist die Grenze
    // `__doc__ traegt deutsche Prosa`, und sie ist gemessen.
    {
        let module = py.import("sys")?.getattr("modules")?;
        for (name, satz) in moduldoku::MODULDOKU {
            if name.is_empty() {
                m.setattr("__doc__", *satz)?;
                continue;
            }
            // Nur was es hier auch gibt - `pygame` hat Module, die diese
            // Huelle nicht nachbaut (`examples`, `__briefcase`).
            if let Ok(kind) = module.get_item(format!("rustgame_py.{name}")) {
                let _ = kind.setattr("__doc__", *satz);
            }
        }
        // Und die Klassen - dort ist `__doc__` ebenfalls schreibbar.
        // Dieselbe Klasse steht oft unter mehreren Namen (`Channel`,
        // `mixer.Channel`, `mixer.ChannelType` sind **ein** Objekt);
        // derselbe Satz zweimal kostet nichts.
        //
        // **Gefragt wird das `__dict__`, nicht `getattr`.** Ein
        // `getattr` loest das Modul-`__getattr__` aus, und `_camera.py`
        // laedt dort nach: ein Import **waehrend** dieses Aufbaus haengt,
        // weil `rustgame_py` noch nicht in `sys.modules` steht. Genau
        // dieser Haenger ist hier schon dreimal passiert und nur mit
        // `faulthandler.dump_traceback_later` zu sehen - die Spur endete
        // in `_camera.py` Zeile 55. Klassen, die **nur** ueber ein
        // `__getattr__` kommen, bleiben damit ohne Ueberlagerung; das
        // sind die Geraetemodule.
        for (modul, klasse, satz) in moduldoku::KLASSENDOKU {
            let wo = if modul.is_empty() {
                Ok(m.as_any().clone())
            } else {
                module.get_item(format!("rustgame_py.{modul}"))
            };
            let Ok(wo) = wo else { continue };
            let Ok(raum) = wo.getattr("__dict__") else {
                continue;
            };
            let Ok(raum) = raum.downcast_into::<pyo3::types::PyDict>() else {
                continue;
            };
            if let Ok(Some(k)) = raum.get_item(*klasse) {
                let _ = k.setattr("__doc__", *satz);
            }
        }
    }

    // **Zuletzt der Gruss**, wie bei pygame am Ende von `__init__.py` -
    // siehe `python/gruss.py`.
    {
        let code = std::ffi::CString::new(include_str!("python/gruss.py"))?;
        let raum = pyo3::types::PyDict::new(py);
        raum.set_item("ver", m.getattr("version")?.getattr("ver")?)?;
        raum.set_item("get_sdl_version", m.getattr("get_sdl_version")?)?;
        py.run(code.as_c_str(), Some(&raum), None)?;
    }

    Ok(())
}
