//! fenster.rs
//! ==========
//! `pygame.window` - `Window` und `get_grabbed_window`.
//!
//! Die Rechnung steht in [`rustgame_display::fenster`]; dort steht auch,
//! was an echtem pygame-ce 2.5.7 gemessen wurde und was diese Kiste
//! ehrlich nicht kann. Hier wird nur uebersetzt - und dabei auf drei
//! Dinge geachtet, die eine Huelle leicht verliert.
//!
//! ## Erstens: welche Ausnahme es ist
//!
//! `pygame.window.Window` wirft dreierlei, und wer alles zu
//! `pygame.error` verflacht, gibt einem `except ValueError:` den
//! falschen Zweig. Gemessen:
//!
//! | Anlass | Ausnahme |
//! |---|---|
//! | `Window('T', (0, 48))` | `ValueError` |
//! | `w.title = 5` | `TypeError` |
//! | `w.opacity = 0.5` unter dem Dummy-Treiber | `pygame.error` |
//! | `w.get_surface()` nach `destroy()` | `pygame.error` |
//!
//! Deshalb hat diese Datei ihren eigenen Umsetzer [`fenster_fehler`] und
//! benutzt **nicht** `umwandlung::anzeige_fehler`, der alles zu
//! `pygame.error` macht.
//!
//! ## Vier Abweichungen, die bleiben - und warum
//!
//! Die Gegenprobe gegen echtes pygame (`gegenprobe.py`) meldet an vier
//! Stellen einen Unterschied, der **keine** Luecke ist. Wer sie fuer
//! einen Fehler haelt und "behebt", macht die Huelle schlechter:
//!
//! 1. **Der Typname in `AttributeError`.** Beim Schreiben auf ein nur
//!    lesbares Merkmal sagt pygame `attribute 'opengl' of
//!    'pygame.window.Window' objects is not writable`, hier steht
//!    `'rustgame_py.window.Window'`. Den Namen zu faelschen waere
//!    schlimmer als die Abweichung: die Meldung nennt dann einen Typ, den
//!    es im Prozess nicht gibt, und jede Fehlersuche liefe ins Leere.
//! 2. **PyO3s Zusatz `argument 'wert': ` im `TypeError`** von
//!    `w.opacity = 'x'`. Das schreibt PyO3s Argumentleser dazu; der
//!    Ausnahmetyp und der Kern der Meldung stimmen.
//! 3. **Der Wortlaut bei `opengl=True` / `vulkan=True`.** Beide
//!    scheitern, hier wie dort. pygames Text nennt den SDL-Treibernamen
//!    (`... in current SDL video driver (dummy) ...`) - ein Treiber, den
//!    es hier nicht gibt; siehe [`rustgame_display::fenster::Fenster::neu`].
//! 4. **Nach `display.quit()` oder `destroy()`** liest pygame
//!    freigegebenen Speicher: `size` und `position` geben Zufallszahlen
//!    (gemessen `(999516864, 967804557)`, bei jedem Lauf andere), `id`
//!    und `opacity` geben `pygame.error: Invalid window`. Hier kommt
//!    ueberall dieser Fehler - derselbe Text, den pygame selbst fuer
//!    `get_surface()` nach `destroy()` verwendet. Zufallszahlen
//!    nachzubauen geht nicht und waere auch keine Verbesserung; siehe
//!    `rustgame_display::fenster::Ablage::leeren`.
//!
//!    **Ausser dem Titel.** Der ist bei pygame keine Zufallszahl,
//!    sondern deterministisch leer (`w.title == ''`): pygame fuehrt ihn
//!    in seinem eigenen Zettel, nicht im Fenster. Hier stand dafuer
//!    ebenfalls der Fehler, in einem Satz mit den Zufallszahlen - ein
//!    Vermerk, der zwei Dinge zusammenfasst, die nicht zusammengehoeren.
//!
//! ## Zweitens: dasselbe Objekt, nicht dasselbe Bild
//!
//! Gemessen gibt `w.get_surface()` bei jedem Aufruf **dasselbe**
//! Python-Objekt zurueck, und beim Fenster des Anzeigemoduls ist es
//! obendrein dasselbe, das `pygame.display.set_mode` geliefert hat. Ein
//! `Rc` im Kern genuegt dafuer nicht - eine frische Huelle um denselben
//! `Rc` waere ein anderes Objekt. Also merkt sich [`PyWindow`] seine
//! Flaeche, und das Anzeigefenster holt sie ueber
//! [`crate::anzeige::anzeige_get_surface`] aus derselben Schublade, in
//! der `set_mode` sie ablegt.
//!
//! Aus demselben Grund merkt sich diese Datei das Fenster des
//! Anzeigemoduls: gemessen ist `Window.from_display_module() is
//! Window.from_display_module()` wahr.
//!
//! ## Drittens: Flaggen kommen als `i64` herein
//!
//! `position=pygame.WINDOWPOS_CENTERED` ist `805240832` und passt noch
//! in ein `u32` - aber die Gewohnheit dieser Kiste ist, solche Werte als
//! `i64` zu nehmen, weil `pygame.FULLSCREEN` (`0x80000000`) sonst einen
//! `OverflowError` aus dem Argumentleser wirft statt der pygame-Meldung.

use std::cell::RefCell;

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};

use rustgame_display::backend::Fensterzustand;
use rustgame_display::fenster::{self, Fenster, Stelle, Wunsch};
use rustgame_display::Fehler;

use crate::flaeche::PySurface;
use crate::rechteck::PyRect;
use crate::umwandlung::PygameError;

thread_local! {
    /// Das Fenster des Anzeigemoduls als Python-Objekt.
    ///
    /// Gemessen: `Window.from_display_module()` gibt bei jedem Aufruf
    /// **dasselbe** Objekt zurueck. Siehe Modulkopf.
    static ANZEIGEFENSTER: RefCell<Option<Py<PyWindow>>> = const { RefCell::new(None) };

    /// Jedes lebende Fenster, unter seiner Nummer.
    ///
    /// Gebraucht fuer [`fenster_get_grabbed_window`]: gemessen ist
    /// `pygame.window.get_grabbed_window() is w` **wahr**, und der Kern
    /// fuehrt nur Nummern. Ohne diese Liste kaeme dort eine frische
    /// Huelle um dasselbe Fenster heraus - gleicher Inhalt, anderes
    /// Objekt, und ein `is`-Vergleich im Programm waere falsch.
    ///
    /// Die Liste haelt eine **starke** Bindung; ein Fenster bleibt also
    /// am Leben, solange es nicht zerstoert ist. Bei pygame ist es
    /// genauso - dort haelt SDL das Objekt. Aufgeraeumt wird bei jedem
    /// Blick: was der Kern nicht mehr kennt, fliegt raus.
    static FENSTERLISTE: RefCell<Vec<(u32, Py<PyWindow>)>> = const { RefCell::new(Vec::new()) };
}

/// Traegt ein frisches Fenster in die Liste ein.
fn merken(py: Python<'_>, id: u32, w: &Py<PyWindow>) {
    FENSTERLISTE.with(|l| {
        let mut l = l.borrow_mut();
        aufraeumen(py, &mut l);
        l.push((id, w.clone_ref(py)));
    })
}

/// Wirft heraus, was der Kern nicht mehr kennt.
fn aufraeumen(py: Python<'_>, liste: &mut Vec<(u32, Py<PyWindow>)>) {
    liste.retain(|(_, w)| w.bind(py).borrow().kern.lebt());
}

/// Das Python-Objekt zu einem Fenster des Kerns, wenn es eines gibt.
fn nachschlagen(py: Python<'_>, kern: Fenster) -> Option<Py<PyWindow>> {
    FENSTERLISTE.with(|l| {
        let mut l = l.borrow_mut();
        aufraeumen(py, &mut l);
        l.iter()
            .find(|(_, w)| w.bind(py).borrow().kern == kern)
            .map(|(_, w)| w.clone_ref(py))
    })
}

/// Setzt einen Fehler aus dem Kern in die Ausnahme um, die pygame wirft.
///
/// [`Fehler::Wert`] ist ein `ValueError`, alles andere eine
/// `pygame.error` - beides gemessen, siehe Modulkopf.
pub fn fenster_fehler(e: Fehler) -> PyErr {
    match e {
        Fehler::Wert(s) => PyValueError::new_err(s),
        andere => PygameError::new_err(andere.to_string()),
    }
}

/// Liest eine Groesse, wie pygame sie liest.
///
/// Gemessen: alles, was kein Paar ist, gibt `TypeError: invalid size
/// argument` - auch ein einelementiges Tupel und eine Zeichenkette. Eine
/// Liste geht dagegen. Deshalb wird hier von Hand geprueft statt PyO3s
/// Tupelleser zu nehmen, der `ValueError: expected tuple of length 2`
/// sagen wuerde.
fn groesse_aus(o: &Bound<'_, PyAny>) -> PyResult<(i64, i64)> {
    paar_aus(o).ok_or_else(|| PyTypeError::new_err("invalid size argument"))
}

/// Zwei ganze Zahlen aus einer Folge - oder nichts.
fn paar_aus(o: &Bound<'_, PyAny>) -> Option<(i64, i64)> {
    if o.is_instance_of::<PyString>() {
        return None;
    }
    let folge = o.try_iter().ok()?;
    let mut werte = Vec::new();
    for x in folge {
        werte.push(x.ok()?.extract::<i64>().ok()?);
        if werte.len() > 2 {
            return None;
        }
    }
    match werte[..] {
        [a, b] => Some((a, b)),
        _ => None,
    }
}

/// Eine Groesse als `(u32, u32)`, nachdem der Kern sie freigegeben hat.
///
/// Der Kern prueft selbst auf null und negativ und liefert dort den
/// gemessenen Wortlaut; hier wird nur noch umgerechnet.
fn zu_groesse(paar: (i64, i64)) -> (u32, u32) {
    (paar.0.max(0) as u32, paar.1.max(0) as u32)
}

/// `pygame.window.Window`.
///
/// Traegt nur den Griff aus dem Kern und die gemerkte Flaeche - alles
/// andere steht in [`rustgame_display::fenster`].
#[pyclass(name = "Window", module = "rustgame_py.window", unsendable, subclass)]
pub struct PyWindow {
    kern: Fenster,
    /// Die Flaeche als Python-Objekt, sobald `get_surface()` einmal lief.
    ///
    /// Beim Fenster des Anzeigemoduls bleibt das leer - dessen Flaeche
    /// liegt in [`crate::anzeige`], damit `set_mode` und `get_surface`
    /// **dasselbe** Objekt liefern. Siehe Modulkopf.
    flaeche: Option<Py<PySurface>>,
    /// Ist das das Fenster des Anzeigemoduls?
    vom_anzeigemodul: bool,
}

impl PyWindow {
    /// Der Griff, oder `Invalid window`.
    fn kern(&self) -> PyResult<Fenster> {
        self.kern.id().map_err(fenster_fehler)?;
        Ok(self.kern)
    }
}

#[pymethods]
impl PyWindow {
    /// `Window(title='pygame window', size=(640, 480), position=WINDOWPOS_UNDEFINED, **flags)`.
    ///
    /// Die Vorgaben sind gemessen. `position` nimmt ein Paar **oder**
    /// eine der beiden Konstanten als einzelne Zahl - ebenfalls
    /// gemessen, und beide stellen unter dem Dummy-Treiber mittig.
    ///
    /// Ein unbekanntes Schluesselwort ist ein `TypeError` mit dem
    /// gemessenen Wortlaut `__init__ got an unexpected flag 'name'`.
    #[new]
    #[pyo3(signature = (title=None, size=None, position=None, **flaggen))]
    fn neu(
        title: Option<&Bound<'_, PyAny>>,
        size: Option<&Bound<'_, PyAny>>,
        position: Option<&Bound<'_, PyAny>>,
        flaggen: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<PyWindow> {
        let mut wunsch = Wunsch::default();

        if let Some(t) = title {
            // Gemessen: `Window(5, ..)` sagt `argument 1 must be str, not int`.
            wunsch.titel = t.extract::<String>().map_err(|_| {
                PyTypeError::new_err(format!(
                    "argument 1 must be str, not {}",
                    t.get_type().name().map(|n| n.to_string()).unwrap_or_default()
                ))
            })?;
        }
        let groesse = match size {
            Some(s) => groesse_aus(s)?,
            None => (wunsch.groesse.0 as i64, wunsch.groesse.1 as i64),
        };
        if groesse.0 <= 0 || groesse.1 <= 0 {
            // Der Punkt am Ende gehoert dazu - gemessen; der Setzer
            // `w.size` sagt denselben Satz ohne ihn.
            return Err(PyValueError::new_err(
                "width or height should not be less than or equal to zero.",
            ));
        }
        wunsch.groesse = zu_groesse(groesse);

        if let Some(p) = position {
            wunsch.stelle = stelle_aus(p)?;
        }

        if let Some(d) = flaggen {
            for (schluessel, wert) in d.iter() {
                let name: String = schluessel.extract()?;
                let an = wert.is_truthy()?;
                setze_wunschflagge(&mut wunsch, &name, an)?;
            }
        }

        let kern = Fenster::neu(&wunsch).map_err(fenster_fehler)?;
        Ok(PyWindow {
            kern,
            flaeche: None,
            vom_anzeigemodul: false,
        })
    }

    /// `Window.__init__(...)` - **stellt nach, legt nicht an.**
    ///
    /// Das Fenster entsteht schon in `__new__`; diese Methode gibt es fuer
    /// die eine Zeile, die eine Unterklasse schreibt:
    ///
    /// ```python
    /// class WindowSubclass(Window):
    ///     def __init__(self, title="Anderer Titel", size=(640, 480), **flags):
    ///         super().__init__(title, size, pygame.WINDOWPOS_CENTERED, **flags)
    /// ```
    ///
    /// `WindowSubclass()` ruft `Window.__new__` **ohne** diese Argumente -
    /// Python reicht dem Konstruktor durch, was am Klassenaufruf stand,
    /// und das war nichts. Das Fenster steht danach auf den Vorgaben, und
    /// erst `super().__init__(...)` bringt Titel, Groesse und Stelle
    /// nach. Ohne diese Methode landet der Aufruf bei `object.__init__`
    /// und gibt `takes exactly one argument`; das ist der gemessene
    /// Fehlschlag von `window_test.test_window_subclassable`.
    ///
    /// **Sie haengt nicht im `tp_init`-Schlitz.** PyO3 traegt Methoden in
    /// das Woerterbuch des Typs ein, nicht in die Schlitze; `Window(...)`
    /// laeuft deshalb weiter allein ueber `__new__`, und hier kommt nur
    /// an, wer `__init__` ausdruecklich aufruft. Genau das tut eine
    /// Unterklasse.
    #[pyo3(signature = (title=None, size=None, position=None, **flaggen))]
    fn __init__(
        slf: &Bound<'_, PyWindow>,
        title: Option<&Bound<'_, PyAny>>,
        size: Option<&Bound<'_, PyAny>>,
        position: Option<&Bound<'_, PyAny>>,
        flaggen: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<()> {
        let kern = slf.borrow().kern()?;
        if let Some(t) = title {
            let text: String = t.extract().map_err(|_| {
                PyTypeError::new_err(format!(
                    "argument 1 must be str, not {}",
                    t.get_type().name().map(|n| n.to_string()).unwrap_or_default()
                ))
            })?;
            kern.set_titel(&text).map_err(fenster_fehler)?;
        }
        if let Some(g) = size {
            let paar = groesse_aus(g)?;
            if paar.0 <= 0 || paar.1 <= 0 {
                return Err(PyValueError::new_err(
                    "width or height should not be less than or equal to zero.",
                ));
            }
            kern.set_groesse(zu_groesse(paar)).map_err(fenster_fehler)?;
        }
        if let Some(p) = position {
            kern.set_position(stelle_aus(p)?).map_err(fenster_fehler)?;
        }
        let _ = flaggen;
        Ok(())
    }

    /// `Window.from_display_module()`.
    ///
    /// Gemessen: ohne vorheriges `display.set_mode` ist das ein Fehler,
    /// und bei jedem Aufruf kommt **dasselbe** Objekt zurueck.
    ///
    /// **Eine Klassenmethode, keine statische.** Gemessen gibt
    /// `WindowSubclass.from_display_module()` ein Objekt der Unterklasse;
    /// bei einer statischen Methode kaeme immer ein blankes `Window`.
    /// Gebaut wird es dann mit `cls.__new__(cls)` - das macht dank der
    /// Aufteilung in [`Self::neu`] und `__init__` **kein** Fenster auf,
    /// sondern nur die Huelle, in die der Griff des Anzeigefensters
    /// gelegt wird.
    #[classmethod]
    fn from_display_module(
        cls: &Bound<'_, pyo3::types::PyType>,
        py: Python<'_>,
    ) -> PyResult<Py<PyAny>> {
        let kern = Fenster::vom_anzeigemodul().map_err(fenster_fehler)?;
        // Eine Unterklasse bekommt ein frisches Objekt ihres Typs; nur
        // fuer `Window` selbst gilt die Zusage, **dasselbe** Objekt zu
        // liefern (`test_from_display_module` prueft `win1 is win2`).
        if !cls.is(py.get_type::<PyWindow>()) {
            let obj = cls.call_method1("__new__", (cls,))?;
            {
                let w = obj.downcast::<PyWindow>()?;
                let mut ich = w.borrow_mut();
                // `__new__` hat dabei ein Fenster aufgemacht, das hier
                // niemand will - es wird sofort wieder zugemacht, sonst
                // bliebe bei jedem Aufruf eines uebrig.
                ich.kern.zerstoeren();
                ich.kern = kern;
                ich.flaeche = None;
                ich.vom_anzeigemodul = true;
            }
            return Ok(obj.unbind());
        }
        ANZEIGEFENSTER.with(|f| {
            let mut f = f.borrow_mut();
            if let Some(vorhanden) = f.as_ref() {
                // Nur wiederverwenden, solange es dasselbe Fenster ist:
                // nach `destroy()` und einem neuen `set_mode` traegt der
                // Kern eine neue Nummer.
                if vorhanden.bind(py).borrow().kern == kern {
                    return Ok(vorhanden.clone_ref(py).into_any());
                }
            }
            let neu = Py::new(
                py,
                PyWindow {
                    kern,
                    flaeche: None,
                    vom_anzeigemodul: true,
                },
            )?;
            *f = Some(neu.clone_ref(py));
            Ok(neu.into_any())
        })
    }

    /// Gemessen: `<Window(title='T', id=1)>`, nach `destroy()`
    /// `<Window(Destroyed)>`.
    fn __repr__(&self) -> String {
        // Das Fenster des Anzeigemoduls nennt **weder Titel noch Nummer** -
        // gemessen gibt es woertlich `<Window(From Display)>`, und zwar
        // obwohl es beides haette.
        if self.vom_anzeigemodul && self.kern.id().is_ok() {
            return "<Window(From Display)>".to_string();
        }
        match (self.kern.titel(), self.kern.id()) {
            (Ok(t), Ok(id)) => format!("<Window(title='{t}', id={id})>"),
            _ => "<Window(Destroyed)>".to_string(),
        }
    }

    // -- Titel, Groesse, Stelle -----------------------------------------

    /// `Window.title`.
    #[getter]
    fn get_title(&self) -> PyResult<String> {
        // **Nach `destroy()` ist der Titel leer, kein Fehler.** Gemessen:
        //
        // ```text
        // w = Window(); w.destroy()
        // w.title       ''
        // w.id          pygame.error: Invalid window
        // w.opacity     dasselbe
        // ```
        //
        // Der Titel ist das einzige Feld, das ein zerstoertes Fenster
        // noch beantwortet - pygame fuehrt ihn in seinem eigenen Zettel
        // und nicht im Fenster. Hier gab es fuer alles denselben Fehler.
        if self.kern.id().is_err() {
            return Ok(String::new());
        }
        self.kern()?.titel().map_err(fenster_fehler)
    }

    #[setter]
    fn set_title(&self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        let t: String = wert
            .extract()
            .map_err(|_| PyTypeError::new_err("Argument to set_title must be a str."))?;
        self.kern()?.set_titel(&t).map_err(fenster_fehler)
    }

    /// `Window.size`.
    #[getter]
    fn get_size(&self) -> PyResult<(u32, u32)> {
        self.kern()?.groesse().map_err(fenster_fehler)
    }

    #[setter]
    fn set_size(&self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        let paar = groesse_aus(wert)?;
        let kern = self.kern()?;
        if paar.0 <= 0 || paar.1 <= 0 {
            // Ohne Punkt am Ende - gemessen, siehe `neu`.
            return Err(PyValueError::new_err(
                "width or height should not be less than or equal to zero",
            ));
        }
        kern.set_groesse(zu_groesse(paar)).map_err(fenster_fehler)
    }

    /// `Window.position`.
    #[getter]
    fn get_position(&self) -> PyResult<(i32, i32)> {
        self.kern()?.position().map_err(fenster_fehler)
    }

    #[setter]
    fn set_position(&self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        let stelle = stelle_aus(wert)?;
        self.kern()?.set_position(stelle).map_err(fenster_fehler)
    }

    /// `Window.minimum_size`.
    #[getter]
    fn get_minimum_size(&self) -> PyResult<(u32, u32)> {
        self.kern()?.mindestgroesse().map_err(fenster_fehler)
    }

    #[setter]
    fn set_minimum_size(&self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        let paar = groesse_aus(wert)?;
        self.kern()?
            .set_mindestgroesse(paar)
            .map_err(fenster_fehler)
    }

    /// `Window.maximum_size`.
    #[getter]
    fn get_maximum_size(&self) -> PyResult<(u32, u32)> {
        self.kern()?.hoechstgroesse().map_err(fenster_fehler)
    }

    #[setter]
    fn set_maximum_size(&self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        let paar = groesse_aus(wert)?;
        self.kern()?
            .set_hoechstgroesse(paar)
            .map_err(fenster_fehler)
    }

    // -- Die nur lesbaren Merkmale --------------------------------------

    /// `Window.id` - nur lesbar, gemessen.
    #[getter]
    fn get_id(&self) -> PyResult<u32> {
        self.kern()?.id().map_err(fenster_fehler)
    }

    /// `Window.focused` - nur lesbar, gemessen.
    #[getter]
    fn get_focused(&self) -> PyResult<bool> {
        self.kern()?.fokussiert().map_err(fenster_fehler)
    }

    /// `Window.opengl` - nur lesbar. Immer `False`: der Konstruktor
    /// laesst `opengl=True` gar nicht zu, und gemessen wirft pygame dort
    /// ebenfalls. Siehe Punkt 3 im Modulkopf.
    #[getter]
    fn get_opengl(&self) -> PyResult<bool> {
        self.kern()?.opengl().map_err(fenster_fehler)
    }

    /// `Window.utility` - nur lesbar, gemessen.
    ///
    /// Gemeldet wird der Wunsch aus dem Konstruktor, solange ihm kein
    /// Stueck widersprechen kann; mit minifb `False`. Die Regel samt
    /// Messung steht bei
    /// [`rustgame_display::fenster::Fenster::hilfsfenster`].
    #[getter]
    fn get_utility(&self) -> PyResult<bool> {
        self.kern()?.hilfsfenster().map_err(fenster_fehler)
    }

    /// `Window.mouse_grabbed` - nur lesbar, gemessen.
    #[getter]
    fn get_mouse_grabbed(&self) -> PyResult<bool> {
        self.kern()?.haelt_maus().map_err(fenster_fehler)
    }

    /// `Window.keyboard_grabbed` - nur lesbar, gemessen.
    #[getter]
    fn get_keyboard_grabbed(&self) -> PyResult<bool> {
        self.kern()?.haelt_tastatur().map_err(fenster_fehler)
    }

    // -- Die schaltbaren Merkmale ---------------------------------------

    /// `Window.borderless`.
    #[getter]
    fn get_borderless(&self) -> PyResult<bool> {
        self.kern()?.rahmenlos().map_err(fenster_fehler)
    }

    #[setter]
    fn set_borderless(&self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        // Gemessen: pygame nimmt hier jeden Wert und fragt nur nach der
        // Wahrheit - `w.borderless = 'ja'` wirft nicht.
        self.kern()?
            .set_rahmenlos(wert.is_truthy()?)
            .map_err(fenster_fehler)
    }

    /// `Window.resizable`.
    #[getter]
    fn get_resizable(&self) -> PyResult<bool> {
        self.kern()?.groessenaenderbar().map_err(fenster_fehler)
    }

    #[setter]
    fn set_resizable(&self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        self.kern()?
            .set_groessenaenderbar(wert.is_truthy()?)
            .map_err(fenster_fehler)
    }

    /// `Window.always_on_top`.
    #[getter]
    fn get_always_on_top(&self) -> PyResult<bool> {
        self.kern()?.obenauf().map_err(fenster_fehler)
    }

    #[setter]
    fn set_always_on_top(&self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        self.kern()?
            .set_obenauf(wert.is_truthy()?)
            .map_err(fenster_fehler)
    }

    /// `Window.opacity` - hier immer 1.0; das Setzen geht nicht.
    #[getter]
    fn get_opacity(&self) -> PyResult<f32> {
        self.kern()?.deckkraft().map_err(fenster_fehler)
    }

    #[setter]
    fn set_opacity(&self, wert: f32) -> PyResult<()> {
        self.kern()?.set_deckkraft(wert).map_err(fenster_fehler)
    }

    /// `Window.grab_mouse` - der Wunsch, nicht sein Erfolg.
    #[getter]
    fn get_grab_mouse(&self) -> PyResult<bool> {
        self.kern()?.will_maus().map_err(fenster_fehler)
    }

    /// Nimmt sich selbst als `&Bound`, nicht als `&self`.
    ///
    /// Nur so kommt die Huelle an ihr **eigenes** Python-Objekt und kann
    /// es in [`FENSTERLISTE`] eintragen. Gebraucht wird das fuer
    /// `get_grabbed_window() is w`, und genau hier ist die Stelle: ein
    /// Fenster kann ohne diesen Setzer nie das gegriffene sein.
    #[setter]
    fn set_grab_mouse(slf: &Bound<'_, Self>, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        let kern = slf.borrow().kern()?;
        kern.set_griff_maus(wert.is_truthy()?)
            .map_err(fenster_fehler)?;
        eintragen(slf)
    }

    /// `Window.grab_keyboard` - der Wunsch, nicht sein Erfolg.
    #[getter]
    fn get_grab_keyboard(&self) -> PyResult<bool> {
        self.kern()?.will_tastatur().map_err(fenster_fehler)
    }

    #[setter]
    fn set_grab_keyboard(slf: &Bound<'_, Self>, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        let kern = slf.borrow().kern()?;
        kern.set_griff_tastatur(wert.is_truthy()?)
            .map_err(fenster_fehler)?;
        eintragen(slf)
    }

    /// `Window.mouse_rect`.
    ///
    /// Gemessen: ein Rechteck mit Breite oder Hoehe kleiner-gleich null
    /// kommt als `None` wieder heraus.
    #[getter]
    fn get_mouse_rect(&self) -> PyResult<Option<PyRect>> {
        Ok(self
            .kern()?
            .maus_rechteck()
            .map_err(fenster_fehler)?
            .map(PyRect::neu))
    }

    #[setter]
    fn set_mouse_rect(&self, wert: &Bound<'_, PyAny>) -> PyResult<()> {
        let rechteck = if wert.is_none() {
            None
        } else {
            Some(crate::umwandlung::rect_aus(wert).map_err(|_| {
                PyTypeError::new_err("mouse_rect should be a Rect-like object or None")
            })?)
        };
        self.kern()?
            .set_maus_rechteck(rechteck)
            .map_err(fenster_fehler)
    }

    // -- Die Aufrufe ----------------------------------------------------

    /// `Window.get_surface()`.
    ///
    /// Beim Fenster des Anzeigemoduls **dasselbe** Objekt, das
    /// `display.set_mode` geliefert hat - gemessen. Sonst dasselbe bei
    /// jedem Aufruf; siehe Modulkopf.
    fn get_surface(&mut self, py: Python<'_>) -> PyResult<Py<PySurface>> {
        let kern = self.kern()?;
        let geteilt = kern.flaeche().map_err(fenster_fehler)?;
        if self.vom_anzeigemodul {
            if let Some(s) = crate::anzeige::anzeige_get_surface(py) {
                return Ok(s);
            }
        }
        if let Some(s) = self.flaeche.as_ref() {
            return Ok(s.clone_ref(py));
        }
        let neu = Py::new(py, PySurface::aus_geteilt(geteilt))?;
        self.flaeche = Some(neu.clone_ref(py));
        Ok(neu)
    }

    /// `Window.flip()`.
    fn flip(&self) -> PyResult<()> {
        self.kern()?.flip().map_err(fenster_fehler)
    }

    /// `Window.set_icon(icon)`.
    // `/`: pygames `Window.set_icon` nimmt **kein** Schluesselwort
    // (gemessen `(self, object, /)`), unsere nahm `bild=` an.
    #[pyo3(signature = (bild, /))]
    fn set_icon(&self, bild: &Bound<'_, PyAny>) -> PyResult<()> {
        let s = bild
            .downcast::<PySurface>()
            .map_err(|_| PyTypeError::new_err("Argument to set_icon must be a Surface."))?;
        let geliehen = s.borrow();
        let flaeche = geliehen.leihen()?;
        self.kern()?.set_sinnbild(&flaeche).map_err(fenster_fehler)
    }

    /// `Window.set_modal_for(parent)`.
    // `/`: pygames `Window.set_modal_for` nimmt **kein** Schluesselwort
    // (gemessen `(self, object, /)`), unsere nahm `eltern=` an.
    #[pyo3(signature = (eltern, /))]
    fn set_modal_for(&self, eltern: &Bound<'_, PyAny>) -> PyResult<()> {
        let e = eltern
            .downcast::<PyWindow>()
            .map_err(|_| PyTypeError::new_err("Argument to set_modal_for must be a Window."))?;
        let eltern_kern = e.borrow().kern;
        self.kern()?
            .modal_fuer(eltern_kern)
            .map_err(fenster_fehler)
    }

    /// `Window.show()`.
    fn show(&self) -> PyResult<()> {
        self.kern()?.zeigen().map_err(fenster_fehler)
    }

    /// `Window.hide()`.
    fn hide(&self) -> PyResult<()> {
        self.kern()?.verstecken().map_err(fenster_fehler)
    }

    /// `Window.focus(input_only=False)`.
    #[pyo3(signature = (input_only=false))]
    fn focus(&self, input_only: bool) -> PyResult<()> {
        self.kern()?.nach_vorn(input_only).map_err(fenster_fehler)
    }

    /// `Window.flash(operation)`.
    ///
    /// Gemessen: eine Zahl, die keine der drei `FLASH_*`-Konstanten ist,
    /// gibt einen `ValueError`; etwas, das gar keine Zahl ist, einen
    /// `TypeError` - und zwar **vor** der Frage, ob das Blinken ueberhaupt
    /// geht.
    // `/`: pygames `Window.flash` nimmt **kein** Schluesselwort
    // (gemessen `(self, object, /)`), unsere nahm `operation=` an.
    #[pyo3(signature = (operation, /))]
    fn flash(&self, operation: &Bound<'_, PyAny>) -> PyResult<()> {
        let art: i64 = operation.extract().map_err(|_| {
            PyTypeError::new_err(
                "'operation' must be an integer. Must correspond with \
                 FLASH_CANCEL, FLASH_BRIEFLY, or FLASH_UNTIL_FOCUSED.",
            )
        })?;
        if !(0..=2).contains(&art) {
            return Err(PyValueError::new_err("Unsupported window flash operation."));
        }
        self.kern()?.blinken(art as u32).map_err(fenster_fehler)
    }

    /// `Window.maximize()`.
    fn maximize(&self) -> PyResult<()> {
        self.kern()?
            .zustand_setzen(Fensterzustand::Aufgezogen)
            .map_err(fenster_fehler)
    }

    /// `Window.minimize()`.
    fn minimize(&self) -> PyResult<()> {
        self.kern()?
            .zustand_setzen(Fensterzustand::Abgelegt)
            .map_err(fenster_fehler)
    }

    /// `Window.restore()`.
    fn restore(&self) -> PyResult<()> {
        self.kern()?
            .zustand_setzen(Fensterzustand::Gewoehnlich)
            .map_err(fenster_fehler)
    }

    /// `Window.set_fullscreen(desktop=False)`.
    ///
    /// Gemessen: unter dem Dummy-Treiber liefern beide Wege dieselbe
    /// Groesse, die der Arbeitsflaeche. `desktop` wird darum entgegen-
    /// genommen und wirkt hier nicht anders.
    #[pyo3(signature = (desktop=false))]
    fn set_fullscreen(&self, desktop: bool) -> PyResult<()> {
        let _ = desktop;
        self.kern()?.set_vollbild(true).map_err(fenster_fehler)
    }

    /// `Window.set_windowed()`.
    fn set_windowed(&self) -> PyResult<()> {
        self.kern()?.set_vollbild(false).map_err(fenster_fehler)
    }

    /// `Window.destroy()`.
    ///
    /// Gemessen: ein zweiter Aufruf ist kein Fehler.
    fn destroy(&mut self, py: Python<'_>) {
        // **Die Flaeche stirbt mit.** Gemessen: nach `win.destroy()` gibt
        // `surf.fill(...)` auf der vorher geholten Flaeche einen
        // `pygame.error`. Wer sie nur loslaesst, laesst ein Stueck
        // Fensterspeicher beschreibbar, das es nicht mehr gibt.
        if let Some(s) = self.flaeche.take() {
            s.bind(py).borrow().toeten();
        }
        self.kern.zerstoeren();
    }
}

/// Traegt ein Fenster in [`FENSTERLISTE`] ein, wenn es noch fehlt.
fn eintragen(slf: &Bound<'_, PyWindow>) -> PyResult<()> {
    let py = slf.py();
    let kern = slf.borrow().kern;
    let id = kern.id().map_err(fenster_fehler)?;
    if nachschlagen(py, kern).is_none() {
        merken(py, id, &slf.clone().unbind());
    }
    Ok(())
}

/// Uebersetzt pygames `position`.
///
/// Ein Paar wird zu [`Stelle::Genau`]; eine einzelne Zahl zu
/// [`Stelle::Mittig`] oder [`Stelle::Offen`], wenn sie eine der beiden
/// Konstanten ist. Alles andere ist `TypeError: invalid position
/// argument` - gemessen.
fn stelle_aus(o: &Bound<'_, PyAny>) -> PyResult<Stelle> {
    if let Some(paar) = paar_aus(o) {
        return Ok(Stelle::Genau((paar.0 as i32, paar.1 as i32)));
    }
    // **Eine blanke Zahl ist keine Stelle.** Gemessen ist
    // `w.position = 123` ein `TypeError`; erlaubt sind allein die beiden
    // Konstanten, und die sind keine Koordinaten, sondern Wuensche
    // ("mittig", "such dir was aus"). Hier stand
    // `Stelle::aus_zahl(zahl)`, was aus 123 die Stelle (123, 123) machte -
    // eine Zahl zweimal zu nehmen ist nie das, was jemand meinte.
    match o.extract::<i64>() {
        Ok(zahl)
            if zahl == rustgame_display::konstanten::WINDOWPOS_CENTERED as i64
                || zahl == rustgame_display::konstanten::WINDOWPOS_UNDEFINED as i64 =>
        {
            Ok(Stelle::aus_zahl(zahl))
        }
        _ => Err(PyTypeError::new_err("invalid position argument")),
    }
}

/// Traegt ein Schluesselwort aus dem Konstruktor in den Wunsch ein.
///
/// Die zwanzig Namen stehen so in `help(pygame.Window)`; ein anderer
/// gibt den gemessenen `TypeError`.
fn setze_wunschflagge(wunsch: &mut Wunsch, name: &str, an: bool) -> PyResult<()> {
    match name {
        "fullscreen" => wunsch.vollbild = an,
        "fullscreen_desktop" => wunsch.vollbild_arbeitsflaeche = an,
        "opengl" => wunsch.opengl = an,
        "vulkan" => wunsch.vulkan = an,
        "hidden" => wunsch.versteckt = an,
        "borderless" => wunsch.rahmenlos = an,
        "resizable" => wunsch.groessenaenderbar = an,
        "minimized" => wunsch.abgelegt = an,
        "maximized" => wunsch.aufgezogen = an,
        "mouse_grabbed" => wunsch.griff_maus = an,
        "keyboard_grabbed" => wunsch.griff_tastatur = an,
        "input_focus" => wunsch.eingabefokus = an,
        "mouse_focus" => wunsch.mausfokus = an,
        "allow_high_dpi" => wunsch.hohe_aufloesung = an,
        "mouse_capture" => wunsch.maus_fangen = an,
        "always_on_top" => wunsch.obenauf = an,
        "utility" => wunsch.hilfsfenster = an,
        andere => {
            return Err(PyTypeError::new_err(format!(
                "__init__ got an unexpected flag '{andere}'"
            )))
        }
    }
    Ok(())
}

/// `pygame.window.get_grabbed_window()`.
///
/// **Dieser Aufruf stuerzt hier nicht ab.** Echtes pygame beendet den
/// Prozess hart, wenn das Video-Teilsystem nicht laeuft - siehe den
/// Modulkopf von [`rustgame_display::fenster`].
#[pyfunction]
#[pyo3(name = "get_grabbed_window")]
pub fn fenster_get_grabbed_window(py: Python<'_>) -> PyResult<Option<Py<PyWindow>>> {
    let Some(kern) = fenster::get_grabbed_window() else {
        return Ok(None);
    };
    // Gemessen: `get_grabbed_window() is w` ist **wahr**. Deshalb wird
    // hier nachgeschlagen und nicht neu gebaut - siehe [`FENSTERLISTE`].
    if let Some(w) = nachschlagen(py, kern) {
        return Ok(Some(w));
    }
    let gleich = ANZEIGEFENSTER.with(|f| {
        f.borrow()
            .as_ref()
            .filter(|w| w.bind(py).borrow().kern == kern)
            .map(|w| w.clone_ref(py))
    });
    if let Some(w) = gleich {
        return Ok(Some(w));
    }
    // Hierher kommt man nur, wenn ein Fenster **ohne** die beiden
    // Griff-Setzer gegriffen wurde - was aus Python nicht geht. Der Zweig
    // steht da, damit die Antwort auch dann stimmt, wenn Rust-Code das
    // Fenster gegriffen hat.
    Ok(Some(Py::new(
        py,
        PyWindow {
            kern,
            flaeche: None,
            vom_anzeigemodul: false,
        },
    )?))
}
