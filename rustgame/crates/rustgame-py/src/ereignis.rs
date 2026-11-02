//! ereignis.rs
//! ===========
//! `pygame.event`.
//!
//! ## Drei gemessene Eigenheiten, die hier nicht verlorengehen duerfen
//!
//! **1. `poll()` auf leerer Schlange gibt `NOEVENT`, nicht `None`.**
//! `<Event(0-NoEvent {})>` ist ein vollwertiges Ereignis; Programme
//! schreiben `if e.type == pygame.NOEVENT`. Ein `None` daraus zu machen
//! waere die naheliegende und falsche Uebersetzung von `Option`.
//!
//! **2. `post()` auf einen gesperrten Typ gibt `False`** und verwirft das
//! Ereignis endgueltig - auch wenn der Typ spaeter wieder freigegeben
//! wird.
//!
//! **3. `event_name(USEREVENT + 3)` ist `"UserEvent"`**, nicht
//! `"Unknown"`.
//!
//! ## Wo die Nutzlast bleibt
//!
//! Ein pygame-Programm schreibt
//! `post(Event(MEIN_TYP, {"spieler": objekt}))`. Die Schlange des Kerns
//! fuehrt aber ein Rust-Enum mit festen Feldern - ein beliebiges
//! Python-Objekt passt dort nicht hinein.
//!
//! Also gehen beide Wege nebeneinander: das Ereignis wandert wie jedes
//! andere in die Kernschlange (damit Sperren, Reihenfolge und `pump`
//! stimmen), und sein Woerterbuch wird hier **daneben** aufbewahrt, je
//! Typ eine eigene Reihe. Kommt das Ereignis wieder heraus, wird das
//! passende Woerterbuch davorgezogen. Weil der Kern die Reihenfolge je
//! Typ erhaelt, treffen sich beide wieder - der einzige Fall, in dem es
//! danebengehen koennte, waere ein Ereignis **desselben** Typs, das
//! zwischendurch vom Fenster kommt; Fenster erzeugen aber keine
//! `USEREVENT`.
//!
//! ## Zeitgeber
//!
//! Die Zustellung liegt in `rustgame-display`: dessen `event::durchlauf`
//! holt die faelligen Auftraege vor jedem Blick in die Schlange ab. Die
//! Huelle muss dafuer nichts tun - sie hat es eine Zeit lang selbst
//! gemacht, solange es dort noch nicht umgesetzt war.

use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

use pyo3::exceptions::{PyAttributeError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};

use rustgame_display::event::{self as kern_event, Daten, Event as KernEvent};
use rustgame_display::konstanten as k;

use crate::umwandlung::{anzeige_fehler, typen_aus};

// ---------------------------------------------------------------------------
//  Event
// ---------------------------------------------------------------------------

/// `pygame.event.Event`.
#[pyclass(name = "Event", module = "rustgame_py.event")]
pub struct PyEvent {
    typ: u32,
    dict: Py<PyDict>,
}

impl PyEvent {
    /// Der Ereignistyp - fuer `time.set_timer(ereignis, ...)`.
    pub fn typ(&self) -> u32 {
        self.typ
    }

    /// Das Woerterbuch als Rust-Wert - fuer `time.set_timer`.
    pub fn dict_roh(&self, py: Python<'_>) -> Py<PyDict> {
        self.dict.clone_ref(py)
    }

    /// Aus einem Kernereignis. Zieht ein aufbewahrtes Woerterbuch vor,
    /// falls das Ereignis einmal aus `post` kam.
    pub fn von_kern(py: Python<'_>, e: &KernEvent) -> PyResult<PyEvent> {
        if let Some(d) = dict_abholen(py, e.typ) {
            return Ok(PyEvent { typ: e.typ, dict: d });
        }
        let d = PyDict::new(py);
        fuelle_dict(py, &d, e)?;
        Ok(PyEvent {
            typ: e.typ,
            dict: d.unbind(),
        })
    }

    /// Nach `rustgame_display` hinueber - so weit, wie das Rust-Enum es
    /// abbilden kann.
    pub fn zu_kern(&self, py: Python<'_>) -> PyResult<KernEvent> {
        let d = self.dict.bind(py);
        let daten = match self.typ {
            k::KEYDOWN | k::KEYUP => Daten::Taste {
                key: zahl(d, "key", 0)?,
                scancode: zahl(d, "scancode", 0)?,
                mods: zahl(d, "mod", 0)?,
                unicode: text(d, "unicode")?,
            },
            k::TEXTINPUT => Daten::Text {
                text: text(d, "text")?,
            },
            k::MOUSEMOTION => Daten::MausBewegung {
                pos: paar(d, "pos")?,
                rel: paar(d, "rel")?,
                buttons: {
                    let b: Vec<bool> = match d.get_item("buttons")? {
                        None => vec![false, false, false],
                        Some(x) => x.extract()?,
                    };
                    (
                        b.first().copied().unwrap_or(false),
                        b.get(1).copied().unwrap_or(false),
                        b.get(2).copied().unwrap_or(false),
                    )
                },
                touch: wahrheit(d, "touch")?,
            },
            k::MOUSEBUTTONDOWN | k::MOUSEBUTTONUP => Daten::MausTaste {
                pos: paar(d, "pos")?,
                button: zahl::<u32>(d, "button", 1)? as u8,
                touch: wahrheit(d, "touch")?,
            },
            k::MOUSEWHEEL => Daten::MausRad {
                x: zahl(d, "x", 0)?,
                y: zahl(d, "y", 0)?,
                precise_x: zahl(d, "precise_x", 0.0)?,
                precise_y: zahl(d, "precise_y", 0.0)?,
                flipped: wahrheit(d, "flipped")?,
            },
            k::VIDEORESIZE => {
                let (w, h) = paar(d, "size")?;
                Daten::Groesse {
                    size: (w.max(0) as u32, h.max(0) as u32),
                    w,
                    h,
                }
            }
            k::ACTIVEEVENT => Daten::Aktiv {
                gain: zahl(d, "gain", 0)?,
                state: zahl(d, "state", 0)?,
            },
            k::AUDIODEVICEADDED | k::AUDIODEVICEREMOVED => Daten::Tongeraet {
                which: zahl(d, "which", 0)?,
                iscapture: zahl(d, "iscapture", 0)?,
            },
            t if t >= k::USEREVENT => Daten::Nutzer {
                code: zahl(d, "code", 0)?,
            },
            _ => Daten::Leer,
        };
        Ok(KernEvent {
            typ: self.typ,
            daten,
        })
    }
}

#[pymethods]
impl PyEvent {
    /// `Event(type[, dict][, **kwargs])`.
    #[new]
    /// `type` ist **nur positionell** und kommt als `i64` herein.
    ///
    /// Gemessen: `Event(type=5)` gibt
    /// `TypeError: function takes at least 1 argument (0 given)` - das
    /// Wort `type` wird dort also gar nicht als Beiwert gelesen, sondern
    /// landet im Woerterbuch und loest die Doppelpruefung aus. Mit einem
    /// benannten Beiwert gab PyO3 stattdessen
    /// `Event.__new__() got multiple values for argument 'type'`.
    ///
    /// Und `i64` statt `u32`, damit `Event(-1)` den gemessenen
    /// `ValueError: event type out of range` gibt und nicht den
    /// `OverflowError` des Argumentlesers.
    // **`&Bound` statt `i64`** fuer den Typ - sonst schreibt PyO3
    // `argument 'r#type': ` vor den Satz, mit dem **Rust-Rohnamen**
    // darin. Gemessen: `Event(None)` gibt bei pygame
    // `TypeError: 'NoneType' object cannot be interpreted as an
    // integer`, hier stand `argument 'r#type': ` davor. Derselbe Griff
    // wie bei `event_name` eine Datei weiter unten; der Vorsatz vor den
    // uebrigen 51 Stellen bleibt eine benannte Grenze, aber ein
    // **Rohname** gehoert dort auf keinen Fall hin.
    #[pyo3(signature = (r#type, dict=None, /, **kwargs))]
    fn new(
        py: Python<'_>,
        r#type: &Bound<'_, PyAny>,
        dict: Option<&Bound<'_, PyDict>>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyEvent> {
        // **Das uebergebene Woerterbuch wird geteilt, nicht kopiert.**
        //
        // Gemessen an echtem pygame:
        //
        // ```python
        // d = {}
        // a, b = pygame.Event(t, d), pygame.Event(t, d)
        // d["hi"] = "hello"      # wirkt auf beide
        // a.hello = "hi"         # wirkt auf d
        // b.dict["amongus"] = "sus"
        // assert a.dict is d and b.dict is d
        // ```
        //
        // Also **ein** Woerterbuch fuer alle drei Namen. Hier stand
        // `PyDict::new(py)` samt `update` - eine Kopie, und damit war
        // `ev.dict is d` falsch. `event_test.test_event_common_dict_changes`
        // prueft genau die Identitaet, an vier Stellen.
        //
        // Nur wenn **kein** Woerterbuch uebergeben wurde, entsteht ein
        // frisches; Schluesselwoerter werden dann hineingelegt. Werden
        // beide angegeben, landen die Schluesselwoerter im **uebergebenen**
        // - auch das ist geteilt, und pygame tut es genauso.
        let r#type: i64 = r#type.extract()?;
        if !(0..rustgame_display::konstanten::NUMEVENTS as i64).contains(&r#type) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "event type out of range",
            ));
        }
        let r#type = r#type as u32;
        let d = match dict {
            Some(vorhanden) => vorhanden.clone(),
            None => PyDict::new(py),
        };
        if let Some(kw) = kwargs {
            d.update(kw.as_mapping())?;
        }
        // Gemessen: `Event(5, {"type": 9})` gibt
        // `ValueError: redundant type field in event dict` - der Typ steht
        // im ersten Argument und darf nicht doppelt da sein.
        if d.get_item("type")?.is_some() {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "redundant type field in event dict",
            ));
        }
        Ok(PyEvent {
            typ: r#type,
            dict: d.unbind(),
        })
    }

    /// `e.type`.
    #[getter]
    fn r#type(&self) -> u32 {
        self.typ
    }

    /// `e.dict` - alle Felder auf einmal.
    #[getter]
    fn dict(&self, py: Python<'_>) -> Py<PyDict> {
        self.dict.clone_ref(py)
    }

    /// `e.__dict__` - und zwar **dasselbe** Objekt wie `e.dict`.
    ///
    /// Gemessen: `e.__dict__ is e.dict` ist in pygame `True`. Das ist kein
    /// Zufall und keine Bequemlichkeit - `event_test.py` schreibt an
    /// mehreren Stellen ueber das eine und liest ueber das andere, und
    /// `pygame.event.Event` hat gar keinen eigenen Instanzspeicher.
    ///
    /// Der Rust-Name traegt ein `feld_` davor, weil PyO3 aus
    /// `fn __dict__` sonst einen Namen baut, der mit dem Getter `dict`
    /// kollidiert - dieselbe Falle wie bei `Surface.width`.
    #[getter]
    #[pyo3(name = "__dict__")]
    fn feld_dict(&self, py: Python<'_>) -> Py<PyDict> {
        self.dict.clone_ref(py)
    }

    /// `e.irgendwas = 5` schreibt **ins Woerterbuch**.
    ///
    /// Gemessen: `e.c = 9` taucht danach in `e.__dict__` auf, und
    /// `e.type = 5` gibt `AttributeError: readonly attribute`. Ohne
    /// `__setattr__` gab PyO3 hier "That operation is not supported" -
    /// eine Meldung, die von rustgame erzaehlt und nicht von pygame, und
    /// drei Tests hingen daran.
    fn __setattr__(&self, py: Python<'_>, name: &str, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        if name == "type" || name == "dict" || name == "__dict__" {
            return Err(PyAttributeError::new_err("readonly attribute"));
        }
        self.dict.bind(py).set_item(name, wert)
    }

    /// `del e.irgendwas` nimmt es aus dem Woerterbuch.
    fn __delattr__(&self, py: Python<'_>, name: &str) -> PyResult<()> {
        let d = self.dict.bind(py);
        if d.get_item(name)?.is_none() {
            return Err(PyAttributeError::new_err(format!(
                "'pygame.event.Event' object has no attribute '{name}'"
            )));
        }
        d.del_item(name)
    }

    /// `e.key`, `e.pos`, `e.button`, ... - was immer im Woerterbuch steht.
    fn __getattr__(&self, py: Python<'_>, name: &str) -> PyResult<Py<PyAny>> {
        match self.dict.bind(py).get_item(name)? {
            Some(v) => Ok(v.unbind()),
            // Woertlich wie pygame - Programme zeigen diese Meldung an.
            None => Err(PyAttributeError::new_err(format!(
                "'pygame.event.Event' object has no attribute '{name}'"
            ))),
        }
    }

    /// `<Event(256-Quit {})>` - gemessen an echtem pygame.
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "<Event({}-{} {})>",
            self.typ,
            kern_event::event_name(self.typ),
            self.dict.bind(py).repr()?
        ))
    }

    fn __eq__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        let Ok(o) = other.extract::<PyRef<PyEvent>>() else {
            return Ok(false);
        };
        if o.typ != self.typ {
            return Ok(false);
        }
        self.dict.bind(py).eq(o.dict.bind(py))
    }
    fn __ne__(&self, py: Python<'_>, other: &Bound<'_, PyAny>) -> PyResult<bool> {
        Ok(!self.__eq__(py, other)?)
    }

    /// `NOEVENT` ist falsch, alles andere wahr.
    fn __bool__(&self) -> bool {
        self.typ != k::NOEVENT
    }
}

// ---------------------------------------------------------------------------
//  Felder hin und her
// ---------------------------------------------------------------------------

/// **Ein ausdrueckliches `None` zaehlt wie ein fehlendes Feld.**
///
/// Das gilt fuer alle vier Leser hier, und es ist gemessen noetig:
/// `event_test.test_post_and_get_keydown` schickt
/// `Event(KEYDOWN, key=K_a, unicode=None)` und
/// `Event(KEYDOWN, key=K_m, mod=None, window=None)` durch die Schlange.
/// pygame nimmt das an - es legt sein Woerterbuch einfach ab. Hier lief
/// der Wert dagegen durch `extract::<String>()` und gab
/// `TypeError: 'NoneType' object cannot be converted to 'PyString'`.
///
/// Dass es folgenlos bleibt, liegt am Rueckweg: ein selbst abgeschicktes
/// Ereignis bekommt beim Abholen **sein eigenes Woerterbuch** zurueck
/// (siehe `dict_merken`), die Kernfelder sind dann gar nicht gefragt. Sie
/// muessen nur **irgendeinen** Wert tragen duerfen, ohne zu scheitern.
fn feld<'p>(d: &Bound<'p, PyDict>, name: &str) -> PyResult<Option<Bound<'p, PyAny>>> {
    Ok(match d.get_item(name)? {
        Some(v) if !v.is_none() => Some(v),
        _ => None,
    })
}

fn zahl<'p, T>(d: &Bound<'p, PyDict>, name: &str, vorgabe: T) -> PyResult<T>
where
    T: FromPyObject<'p>,
{
    match feld(d, name)? {
        None => Ok(vorgabe),
        Some(v) => v.extract(),
    }
}

fn text(d: &Bound<'_, PyDict>, name: &str) -> PyResult<String> {
    match feld(d, name)? {
        None => Ok(String::new()),
        Some(v) => v.extract(),
    }
}

fn paar(d: &Bound<'_, PyDict>, name: &str) -> PyResult<(i32, i32)> {
    match feld(d, name)? {
        None => Ok((0, 0)),
        Some(v) => v.extract(),
    }
}

fn wahrheit(d: &Bound<'_, PyDict>, name: &str) -> PyResult<bool> {
    match feld(d, name)? {
        None => Ok(false),
        Some(v) => v.extract(),
    }
}

/// Die Felder eines Kernereignisses ins Woerterbuch.
///
/// Die Namen sind die von pygame - `mod` statt `mods`, weil `mod` in Rust
/// ein Schluesselwort ist und der Kern deshalb umbenennen musste.
fn fuelle_dict(py: Python<'_>, d: &Bound<'_, PyDict>, e: &KernEvent) -> PyResult<()> {
    // Jedes `WINDOW*`-Ereignis traegt in pygame ein Feld `window` - beim
    // fensterlosen Treiber ist es `None`. Gemessen: `WINDOWENTER` hat
    // dort `{'window': None}`, nicht `{}`. Die Nummern 32773..32790
    // liegen zusammenhaengend, `ACTIVEEVENT` (32768) gehoert **nicht**
    // dazu und hat stattdessen `gain`/`state`.
    if (k::WINDOWSHOWN..=k::WINDOWDISPLAYCHANGED).contains(&e.typ) {
        d.set_item("window", py.None())?;
    }
    match &e.daten {
        Daten::Leer => {}
        Daten::Taste {
            key,
            scancode,
            mods,
            unicode,
        } => {
            d.set_item("key", key)?;
            d.set_item("scancode", scancode)?;
            d.set_item("mod", mods)?;
            // Gemessen: bei `KEYUP` fuehrt pygame dieses Feld gar nicht.
            if e.typ == k::KEYDOWN {
                d.set_item("unicode", unicode)?;
            }
        }
        Daten::Text { text } => d.set_item("text", text)?,
        Daten::MausBewegung {
            pos,
            rel,
            buttons,
            touch,
        } => {
            d.set_item("pos", pos)?;
            d.set_item("rel", rel)?;
            d.set_item("buttons", (buttons.0 as u8, buttons.1 as u8, buttons.2 as u8))?;
            d.set_item("touch", touch)?;
        }
        Daten::MausTaste { pos, button, touch } => {
            d.set_item("pos", pos)?;
            d.set_item("button", button)?;
            d.set_item("touch", touch)?;
        }
        Daten::MausRad {
            x,
            y,
            precise_x,
            precise_y,
            flipped,
        } => {
            d.set_item("x", x)?;
            d.set_item("y", y)?;
            d.set_item("precise_x", precise_x)?;
            d.set_item("precise_y", precise_y)?;
            d.set_item("flipped", flipped)?;
        }
        Daten::Groesse { size, w, h } => {
            d.set_item("size", size)?;
            d.set_item("w", w)?;
            d.set_item("h", h)?;
        }
        Daten::Fenster { x, y } => {
            d.set_item("x", x)?;
            d.set_item("y", y)?;
        }
        Daten::Aktiv { gain, state } => {
            d.set_item("gain", gain)?;
            d.set_item("state", state)?;
        }
        Daten::Tongeraet { which, iscapture } => {
            d.set_item("which", which)?;
            d.set_item("iscapture", iscapture)?;
        }
        Daten::Nutzer { code } => d.set_item("code", code)?,
    }
    Ok(())
}

// ---------------------------------------------------------------------------
//  Die Nebenablage fuer Woerterbuecher
// ---------------------------------------------------------------------------

thread_local! {
    /// Je Ereignistyp eine Reihe abgelegter Woerterbuecher - siehe
    /// Modulkopf.
    static ABLAGE: RefCell<HashMap<u32, VecDeque<Py<PyDict>>>> =
        RefCell::new(HashMap::new());
}

fn dict_merken(typ: u32, d: Py<PyDict>) {
    ABLAGE.with(|a| a.borrow_mut().entry(typ).or_default().push_back(d));
}

thread_local! {
    /// Das Woerterbuch je **Zeitgeber** - anders als [`ABLAGE`] keine
    /// Reihe, sondern ein einzelner Eintrag je Ereignistyp.
    ///
    /// Ein Zeitgeber schickt sein Ereignis immer wieder, und **jedes Mal
    /// mit demselben Woerterbuch**. Gemessen:
    ///
    /// ```python
    /// ev = Event(T, {"foo": "spam"})
    /// time.set_timer(ev, 50); time.delay(350)
    /// [dict(x.dict) for x in event.get() if x.type == T]
    /// # -> sechsmal {'foo': 'spam'}
    /// ```
    ///
    /// Und es ist **dasselbe Objekt**: `test_timer_common_reference`
    /// prueft `e.dict is event.dict`. Darum liegt hier das `Py<PyDict>`
    /// und nicht eine Kopie.
    ///
    /// `ABLAGE` taugt dafuer nicht: sie ist eine Reihe und wird beim
    /// Abholen geleert - richtig fuer `post`, wo jedes Ereignis sein
    /// eigenes Woerterbuch hat, falsch fuer einen Zeitgeber, der ewig
    /// weiterlaeuft.
    static ZEITGEBER_DICTS: RefCell<HashMap<u32, Py<PyDict>>> =
        RefCell::new(HashMap::new());
}

/// Merkt sich das Woerterbuch eines Zeitgebers - oder vergisst es, wenn
/// `d` `None` ist (`set_timer(typ, 0)`).
pub fn zeitgeber_dict_setzen(typ: u32, d: Option<Py<PyDict>>) {
    ZEITGEBER_DICTS.with(|z| {
        let mut z = z.borrow_mut();
        match d {
            None => {
                z.remove(&typ);
            }
            Some(d) => {
                z.insert(typ, d);
            }
        }
    })
}

fn dict_abholen(py: Python<'_>, typ: u32) -> Option<Py<PyDict>> {
    let aus_reihe = ABLAGE.with(|a| {
        let mut a = a.borrow_mut();
        let reihe = a.get_mut(&typ)?;
        let d = reihe.pop_front();
        if reihe.is_empty() {
            a.remove(&typ);
        }
        d
    });
    if aus_reihe.is_some() {
        return aus_reihe;
    }
    // Kein abgelegtes Woerterbuch aus `post` - dann vielleicht eines von
    // einem Zeigeber. Das wird **nicht** verbraucht.
    ZEITGEBER_DICTS.with(|z| z.borrow().get(&typ).map(|d| d.clone_ref(py)))
}

fn ablage_leeren(typen: Option<&[u32]>) {
    ABLAGE.with(|a| {
        let mut a = a.borrow_mut();
        match typen {
            None => a.clear(),
            Some(ts) => {
                for t in ts {
                    a.remove(t);
                }
            }
        }
    })
}

fn liste<'p>(py: Python<'p>, ereignisse: Vec<KernEvent>) -> PyResult<Bound<'p, PyList>> {
    let mut aus = Vec::with_capacity(ereignisse.len());
    for e in &ereignisse {
        aus.push(Py::new(py, PyEvent::von_kern(py, e)?)?);
    }
    PyList::new(py, aus)
}

// ---------------------------------------------------------------------------
//  Die Schlange
// ---------------------------------------------------------------------------

/// `pygame.event.get(eventtype=None, pump=True, exclude=None)`.
#[pyfunction]
#[pyo3(signature = (eventtype=None, pump=true, exclude=None))]
#[pyo3(name = "get")]
pub fn ereignis_get<'p>(
    py: Python<'p>,
    eventtype: Option<&Bound<'_, PyAny>>,
    pump: bool,
    exclude: Option<&Bound<'_, PyAny>>,
) -> PyResult<Bound<'p, PyList>> {
    crate::umwandlung::video_pruefen()?;
    let _ = pump;
    let ausser = typen_aus(exclude)?;
    let auswahl = typen_aus(eventtype)?;
    // **Beides zusammen geht nicht** - gemessen gibt
    // `get(KEYDOWN, exclude=QUIT)` ein
    // `pygame.error: Invalid combination of excluded and included event
    // type`. Hier wurde `exclude` vorher einfach zuerst geprueft und der
    // Typ stillschweigend uebergangen; die Auskunft war also nicht falsch,
    // nur unvollstaendig - und ein Programm, das beides angibt, meint
    // etwas, das es nicht geben kann.
    if ausser.is_some() && auswahl.is_some() {
        return Err(crate::umwandlung::PygameError::new_err(
            "Invalid combination of excluded and included event type",
        ));
    }
    if let Some(ausser) = ausser {
        return liste(py, kern_event::get_ausser(&ausser));
    }
    liste(py, kern_event::get_typen(auswahl.as_deref()))
}

/// `pygame.event.poll()` - auf leerer Schlange kommt `NOEVENT`.
#[pyfunction]
#[pyo3(name = "poll")]
pub fn ereignis_poll(py: Python<'_>) -> PyResult<PyEvent> {
    crate::umwandlung::video_pruefen()?;
    PyEvent::von_kern(py, &kern_event::poll())
}

/// `pygame.event.wait(timeout=0)` - `0` wartet ohne Ende.
#[pyfunction]
// **Kein Schraegstrich.** Hier stand einer, und er war ein
// Kollateralschaden: das Werkzeug, das die 58 Funktionen
// stellungsgebunden machte, suchte sie am **blossen** Namen, und
// `wait` gibt es zweimal - `time.wait` nimmt keine
// Schluesselwoerter, `event.wait` schon
// (`pygame.event.wait(timeout=5)` ist gemessen gueltig). Gefunden
// hat den Fehler die Probe, die beide Huellen mit pygames eigenem
// Parameternamen fragt.
#[pyo3(signature = (timeout=0))]
#[pyo3(name = "wait")]
pub fn ereignis_wait(py: Python<'_>, timeout: i64) -> PyResult<PyEvent> {
    let frist = if timeout <= 0 {
        None
    } else {
        Some(timeout as u32)
    };
    // Das Warten gibt die GIL frei, sonst stuende der ganze Prozess.
    let e = py.detach(|| kern_event::wait(frist));
    PyEvent::von_kern(py, &e)
}

/// `pygame.event.peek(eventtype=None, pump=True)`.
#[pyfunction]
#[pyo3(signature = (eventtype=None, pump=true))]
#[pyo3(name = "peek")]
pub fn ereignis_peek(eventtype: Option<&Bound<'_, PyAny>>, pump: bool) -> PyResult<bool> {
    crate::umwandlung::video_pruefen()?;
    let _ = pump;
    // **Stueck fuer Stueck, und beim ersten Treffer ist Schluss** -
    // siehe [`crate::umwandlung::typen_je`]. `peek([A, kaputt])` gibt
    // bei pygame `True`, nicht den Fehler.
    match crate::umwandlung::typen_je(eventtype, |t| {
        Ok(!kern_event::peek(Some(&[t])))
    })? {
        None => Ok(kern_event::peek(None)),
        Some(gefunden) => Ok(gefunden),
    }
}

/// `pygame.event.clear(eventtype=None, pump=True)`.
#[pyfunction]
#[pyo3(signature = (eventtype=None, pump=true))]
#[pyo3(name = "clear")]
pub fn ereignis_clear(eventtype: Option<&Bound<'_, PyAny>>, pump: bool) -> PyResult<()> {
    crate::umwandlung::video_pruefen()?;
    let _ = pump;
    // Stueckweise: `clear([A, kaputt])` leert die A und wirft danach.
    if crate::umwandlung::typen_je(eventtype, |t| {
        kern_event::clear(Some(&[t]));
        ablage_leeren(Some(&[t]));
        Ok(true)
    })?
    .is_none()
    {
        kern_event::clear(None);
        ablage_leeren(None);
    }
    Ok(())
}

/// `pygame.event.post(event)` - `False`, wenn der Typ gesperrt ist.
///
/// **Ohne Argument** sagt pygame
/// `TypeError: pygame.event.post() takes exactly one argument (0 given)`,
/// also mit dem ganzen Weg davor und der Zahl dahinter. PyO3 schreibt
/// `post() missing 1 required positional argument: 'event'`; deshalb
/// nimmt die Funktion ihre Argumente als Beutel und prueft selbst.
/// Gemessen auch der Fall mit zweien: `post(a, b)` gibt
/// `pygame.event.post() takes exactly one argument (2 given)`.
#[pyfunction]
#[pyo3(name = "post")]
// **Die Form bleibt `(event, /)`.** Der Beutel ist nur da, um die zwei
// Saetze oben selbst zu werfen; `inspect.signature` soll davon nichts
// sehen. Ohne diese Zeile meldete die Probe `Signaturform` fuer
// `event.post` `0+0***` statt `1+0` - gefunden hat das die Probe, nicht
// ich.
#[pyo3(text_signature = "(event, /)")]
#[pyo3(signature = (*args, **kwargs))]
pub fn ereignis_post(
    py: Python<'_>,
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<bool> {
    // **Und gar keine Schluesselwoerter.** pygame sagt
    // `TypeError: pygame.event.post() takes no keyword arguments`; PyO3
    // schreibt sonst `post() got an unexpected keyword argument`. Der
    // Satz ist sonst eine benannte Grenze - hier nicht, weil diese
    // Funktion ihren Beutel selbst nimmt.
    if kwargs.is_some_and(|d| !d.is_empty()) {
        return Err(PyTypeError::new_err(
            "pygame.event.post() takes no keyword arguments",
        ));
    }
    if args.len() != 1 {
        return Err(PyTypeError::new_err(format!(
            "pygame.event.post() takes exactly one argument ({} given)",
            args.len()
        )));
    }
    let event = &args.get_item(0)?;
    crate::umwandlung::video_pruefen()?;
    // pygames Satz nennt kein Argument und keinen Rust-Typ:
    // `argument must be an Event object`.
    let event: PyRef<'_, PyEvent> = event
        .extract()
        .map_err(|_| pyo3::exceptions::PyTypeError::new_err("argument must be an Event object"))?;
    let event = &*event;
    let k = event.zu_kern(py)?;
    let ok = kern_event::post(k);
    if ok {
        dict_merken(event.typ, event.dict.clone_ref(py));
    }
    Ok(ok)
}

/// `pygame.event.set_allowed(type)` - `None` gibt **alles** frei.
///
/// **Das Argument ist Pflicht**, siehe den Vermerk im Rumpf von
/// [`ereignis_set_blocked`].
#[pyfunction]
#[pyo3(name = "set_allowed")]
#[pyo3(signature = (r#type, /))]
pub fn ereignis_set_allowed(r#type: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
    crate::umwandlung::video_pruefen()?;
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let eventtype = r#type;
    // Stueckweise, wie beim Sperren.
    if crate::umwandlung::typen_je(eventtype, |t| {
        kern_event::set_allowed(Some(&[t]));
        Ok(true)
    })?
    .is_none()
    {
        kern_event::set_allowed(None);
    }
    Ok(())
}

/// `pygame.event.set_blocked(type)` - `None` sperrt **alles**.
#[pyfunction]
#[pyo3(name = "set_blocked")]
#[pyo3(signature = (r#type, /))]
pub fn ereignis_set_blocked(r#type: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
    crate::umwandlung::video_pruefen()?;
    // **Kein Vorgabewert.** `None` heisst hier "alle", und genau deshalb
    // stand ein `=None` in der Signatur - das machte das Argument aber
    // **weglassbar**: `set_blocked()` sperrte hier stillschweigend
    // nichts, waehrend pygame sagt
    // `takes exactly one argument (0 given)` (gemessen an allen drei).
    // Ein `Option` ohne Vorgabe ist genau der Unterschied: `None` geht
    // durch, Fehlen nicht.
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let eventtype = r#type;
    // Stueckweise - siehe [`crate::umwandlung::typen_je`]:
    // `set_blocked([B, kaputt])` sperrt B, raeumt dessen Ereignisse aus
    // und wirft **danach**.
    if crate::umwandlung::typen_je(eventtype, |t| {
        kern_event::set_blocked(Some(&[t]));
        Ok(true)
    })?
    .is_none()
    {
        kern_event::set_blocked(None);
    }
    Ok(())
}

/// `pygame.event.get_blocked(type)`.
#[pyfunction]
#[pyo3(signature = (r#type, /))]
#[pyo3(name = "get_blocked")]
pub fn ereignis_get_blocked(r#type: Option<&Bound<'_, PyAny>>) -> PyResult<bool> {
    crate::umwandlung::video_pruefen()?;
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let eventtype = r#type;
    // **Auch eine Folge** - gemessen gibt
    // `get_blocked([KEYDOWN, QUIT])` ein einzelnes `True`, wenn
    // **mindestens einer** gesperrt ist. Vorher stand hier ein blankes
    // `u32`, und eine Liste gab
    // `TypeError: 'list' object cannot be interpreted as an integer`.
    // **`None` ist hier kein "alles"**, sondern ein Fehler. Bei
    // `set_blocked`, `set_allowed`, `get`, `clear` und `peek` heisst
    // `None` "alle Typen"; `get_blocked(None)` dagegen gibt
    // `TypeError: event type must be numeric or a sequence` - gemessen.
    // Hier kam `False` heraus, also die Auskunft "nichts ist gesperrt"
    // auf eine Frage, die pygame nicht beantwortet.
    // Auch hier stueckweise, und beim ersten gesperrten Typ ist Schluss:
    // `get_blocked([0, kaputt])` gibt `True`, wenn 0 gesperrt ist.
    match crate::umwandlung::typen_je(eventtype, |t| {
        Ok(!kern_event::get_blocked(t))
    })? {
        None => Err(PyTypeError::new_err(
            "event type must be numeric or a sequence",
        )),
        Some(gefunden) => Ok(gefunden),
    }
}

/// `pygame.event.pump()`.
#[pyfunction]
#[pyo3(name = "pump")]
pub fn ereignis_pump() -> PyResult<()> {
    crate::umwandlung::video_pruefen()?;
    kern_event::pump();
    Ok(())
}

/// `pygame.event.event_name(type)` - alles ab `USEREVENT` heisst
/// `"UserEvent"`.
#[pyfunction]
#[pyo3(name = "event_name")]
#[pyo3(signature = (r#type, /))]
pub fn ereignis_event_name(r#type: &Bound<'_, PyAny>) -> PyResult<&'static str> {
    // **`&Bound` statt `u32`**, damit PyO3 nicht `argument 'r#type': `
    // vor den Satz haengt. In diesem Vorsatz stand Rusts Rohbezeichner
    // samt Raute - ein Zeichen aus der Rust-Schreibweise, das in einer
    // Python-Meldung nichts zu suchen hat. `extract()` gibt denselben
    // Satz wie pygame, nur ohne Vorsatz.
    let eventtype: u32 = r#type.extract()?;
    Ok(kern_event::event_name(eventtype))
}

/// `pygame.event.custom_type()` - der erste Aufruf liefert `USEREVENT + 1`.
#[pyfunction]
#[pyo3(name = "custom_type")]
pub fn ereignis_custom_type() -> PyResult<u32> {
    kern_event::custom_type().map_err(anzeige_fehler)
}

/// `pygame.event.set_grab(bool)`.
///
/// Reicht durch, statt selbst Buch zu fuehren - und das ist mehr als
/// Aufraeumen: **ohne Fenster bleibt der Griff wirkungslos**, gemessen
/// gibt `get_grab()` dort auch nach `set_grab(True)` noch `False`
/// zurueck. Eine Huelle, die schlicht zurueckgibt, was verlangt wurde,
/// waere bequem und falsch.
#[pyfunction]
#[pyo3(name = "set_grab")]
#[pyo3(signature = (grab, /))]
pub fn ereignis_set_grab(grab: &Bound<'_, PyAny>) -> PyResult<()> {
    crate::umwandlung::video_pruefen()?;
    // **Jeder Wert ist ein Wahrheitswert** - pygame liest den
    // Schalter mit `PyObject_IsTrue`, und das nimmt alles. Mit
    // `bool` verlangte PyO3 einen echten Wahrheitswert und nannte
    // im Fehlertext `'PyBool'`, einen Namen aus Rust.
    let grab = grab.is_truthy()?;
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let an = grab;
    kern_event::set_grab(an).map_err(anzeige_fehler)
}

/// `pygame.event.get_grab()`.
#[pyfunction]
#[pyo3(name = "get_grab")]
pub fn ereignis_get_grab() -> PyResult<bool> {
    crate::umwandlung::video_pruefen()?;
    kern_event::get_grab().map_err(anzeige_fehler)
}
