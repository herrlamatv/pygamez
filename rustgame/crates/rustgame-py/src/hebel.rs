//! hebel.rs
//! ========
//! `pygame.joystick`.
//!
//! Die Regeln stehen im Kern ([`rustgame_display::joystick`]), samt der
//! Begruendung, warum dieses Modul **vollstaendig** sein kann, obwohl es
//! keinen Treiber gibt: es gibt nie einen Hebel, und was pygame in genau
//! diesem Zustand tut, ist gemessen.

use pyo3::prelude::*;

use rustgame_display::joystick;

use crate::umwandlung::anzeige_fehler;

/// `pygame.joystick.JoystickType` - **die Klasse**, und sie heisst
/// `Joystick`.
///
/// ## Was hier falsch war, und wie es gedeckt wurde
///
/// Hier stand eine Klasse `Joystick` mit **einer** Methode (`__new__`) und
/// daneben der Vermerk, dieses Modul koenne *vollstaendig* sein, obwohl es
/// keinen Treiber gibt: es gebe nie einen Hebel, und was pygame in genau
/// diesem Zustand tut, sei gemessen.
///
/// Der Vermerk klang gut und war die **elfte** Rasterkante dieses
/// Projekts. Gemessen hat `pygame.joystick.JoystickType` **neunzehn**
/// Namen, und keiner davon war hier da. Gefunden hat das kein Test -
/// `joystick_test.py` ueberspringt sich ohne Geraet - sondern erst
/// `alle_klassen()` in `api_coverage.py`, nachdem die Klassenliste gelesen
/// statt genannt wurde.
///
/// ## Und `Joystick` ist eine **Funktion**
///
/// Das ist der zweite Teil desselben Funds. Gemessen:
///
/// ```text
/// pygame.joystick.Joystick      <built-in function Joystick>
/// pygame.joystick.JoystickType  <class 'pygame.joystick.Joystick'>
/// Joystick is JoystickType      False
/// JoystickType(0)               TypeError: cannot create ... instances
/// class X(JoystickType)         TypeError: not an acceptable base type
/// ```
///
/// `Joystick(id)` ist also eine **Fabrik**, und der Typ, den sie baut,
/// heisst `Joystick`, steht aber unter `JoystickType`. Hier war beides
/// derselbe Name - mit dem Vermerk, "ein zweiter, leerer Typ waere eine
/// Nachahmung ohne Nutzen". Der zweite Typ ist aber nicht leer: er traegt
/// die neunzehn Namen.
///
/// ## Warum die Methoden nie laufen
///
/// Es gibt keinen Hebel: [`rustgame_display::joystick::oeffnen`] gibt
/// immer einen Fehler, also entsteht **nie** eine Instanz. Die Methoden
/// sind reine Schnittstelle - ein Programm darf
/// `hasattr(JoystickType, "get_axis")` fragen und `JoystickType.get_name`
/// als ungebundenen Deskriptor in der Hand halten. Gerufen werden kann
/// keine, und darum gibt jede denselben Fehler wie das Oeffnen.
#[pyclass(name = "Joystick", module = "rustgame_py.joystick")]
pub struct PyJoystick;

/// Der Fehler, den eine Methode gibt, die es nicht geben kann.
///
/// Unerreichbar: es gibt keine Instanz. Ein `unreachable!()` stuende hier
/// als Falle fuer den Tag, an dem jemand einen Treiber anschliesst - ein
/// Fehler ist die freundlichere Antwort.
fn kein_hebel() -> PyErr {
    // Der Weg ueber `anzeige_fehler` geht hier nicht: `joystick::Fehler`
    // ist in der Kiste privat, und ein `pub` dafuer waere eine Oeffnung
    // fuer einen Text. Also derselbe Wortlaut, direkt.
    crate::umwandlung::PygameError::new_err("Invalid joystick device number")
}

#[pymethods]
impl PyJoystick {
    /// `init()` - hier unerreichbar, siehe [`kein_hebel`].
    fn init(&self) -> PyResult<()> {
        Err(kein_hebel())
    }

    /// `quit()`.
    fn quit(&self) -> PyResult<()> {
        Err(kein_hebel())
    }

    /// `get_init()` - ob dieser Hebel offen ist.
    fn get_init(&self) -> PyResult<bool> {
        Err(kein_hebel())
    }

    /// `get_id()` - die Nummer, mit der er geoeffnet wurde.
    fn get_id(&self) -> PyResult<i32> {
        Err(kein_hebel())
    }

    /// `get_instance_id()` - SDLs laufende Kennung.
    fn get_instance_id(&self) -> PyResult<i32> {
        Err(kein_hebel())
    }

    /// `get_guid()` - die weltweit eindeutige Kennung als Text.
    fn get_guid(&self) -> PyResult<String> {
        Err(kein_hebel())
    }

    /// `get_name()`.
    fn get_name(&self) -> PyResult<String> {
        Err(kein_hebel())
    }

    /// `get_power_level()` - `"max"`, `"low"`, `"unknown"` und so weiter.
    fn get_power_level(&self) -> PyResult<String> {
        Err(kein_hebel())
    }

    /// `get_numaxes()`.
    fn get_numaxes(&self) -> PyResult<usize> {
        Err(kein_hebel())
    }

    /// `get_numballs()`.
    fn get_numballs(&self) -> PyResult<usize> {
        Err(kein_hebel())
    }

    /// `get_numbuttons()`.
    fn get_numbuttons(&self) -> PyResult<usize> {
        Err(kein_hebel())
    }

    /// `get_numhats()`.
    fn get_numhats(&self) -> PyResult<usize> {
        Err(kein_hebel())
    }

    /// `get_axis(achse)` - eine Fliesskommazahl von -1 bis 1.
    fn get_axis(&self, _axis_number: usize) -> PyResult<f64> {
        Err(kein_hebel())
    }

    /// `get_ball(kugel)` - die Bewegung seit dem letzten Abruf.
    fn get_ball(&self, _ball_number: usize) -> PyResult<(i32, i32)> {
        Err(kein_hebel())
    }

    /// `get_button(knopf)`.
    fn get_button(&self, _button: usize) -> PyResult<bool> {
        Err(kein_hebel())
    }

    /// `get_hat(kreuz)` - `(x, y)` mit Werten aus -1, 0, 1.
    fn get_hat(&self, _hat_number: usize) -> PyResult<(i32, i32)> {
        Err(kein_hebel())
    }

    /// `rumble(tief, hoch, dauer)` - gibt `True`, wenn es geruettelt hat.
    #[pyo3(signature = (low_frequency, high_frequency, duration))]
    fn rumble(
        &self,
        low_frequency: f64,
        high_frequency: f64,
        duration: u32,
    ) -> PyResult<bool> {
        let _ = (low_frequency, high_frequency, duration);
        Err(kein_hebel())
    }

    /// `stop_rumble()`.
    fn stop_rumble(&self) -> PyResult<()> {
        Err(kein_hebel())
    }

    /// `set_led(color, /)` - **ein** Argument, stellungsgebunden.
    ///
    /// Hier stand `*args` mit dem Vermerk "nimmt eine Farbe oder drei
    /// Zahlen". Das war ungemessen: pygames Stub schreibt
    /// `def set_led(self, color: ColorLike, /) -> bool` und seine
    /// Signatur sagt `(self, object, /)` - **eines**, und nur nach
    /// Stellung. Pruefen laesst sich das hier nur an der Form, weil es
    /// ohne Hebel keinen Aufruf gibt, der durchkommt.
    #[pyo3(signature = (color, /))]
    fn set_led(&self, color: &Bound<'_, PyAny>) -> PyResult<bool> {
        let _ = color;
        Err(kein_hebel())
    }
}

/// `pygame.joystick.Joystick(id)` - **die Fabrik**, keine Klasse.
///
/// Scheitert immer, weil es keinen Hebel gibt. Die drei Fehlertexte sind
/// gemessen, und sie sind nicht herzuleiten:
///
/// ```text
/// Joystick(0), Joystick(5)   error: Invalid joystick device number
/// Joystick(-1), Joystick(-5) error: There are 0 joysticks available
/// Joystick()                 TypeError: function takes exactly 1 argument (0 given)
/// Joystick("x")              TypeError: 'str' object cannot be interpreted as an integer
/// Joystick(0.5)              TypeError: 'float' object cannot be interpreted as an integer
/// Joystick(2**31)            OverflowError: Python int too large to convert to C long
/// ```
///
/// Dass eine **negative** Nummer einen anderen Satz gibt als eine zu
/// grosse, ist die Sorte Eigenheit, die man nur durch Hinsehen findet.
/// `2**31` zeigt ausserdem, dass pygame hier ein C-`long` nimmt, und das
/// ist unter Windows **32 Bit**.
///
/// Geschrieben mit `*args` statt mit einem benannten Argument, damit die
/// Saetze Wort fuer Wort stimmen: PyO3 stellt sonst
/// `argument 'nummer': ` davor.
#[pyfunction]
#[pyo3(name = "Joystick", signature = (*args, **kwargs))]
pub fn hebel_bauen(
    py: Python<'_>,
    args: &Bound<'_, pyo3::types::PyTuple>,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<Py<PyJoystick>> {
    use pyo3::exceptions::{PyOverflowError, PyTypeError};

    // Gemessen lehnt pygame jedes Schluesselwort ab - auch `id`, den
    // `joystick.pyi` selbst nennt. Mit `*args` allein kaeme ein
    // `**kwargs` stillschweigend durch.
    crate::umwandlung::keine_woerter(kwargs, "rustgame_py.joystick.Joystick")?;

    if args.len() != 1 {
        return Err(PyTypeError::new_err(format!(
            "function takes exactly 1 argument ({} given)",
            args.len()
        )));
    }
    let roh = args.get_item(0)?;
    let nummer: i32 = match roh.extract() {
        Ok(n) => n,
        Err(_) => {
            // Zu gross fuer ein `long` oder gar keine Zahl - pygame
            // unterscheidet die beiden Saetze, also hier auch.
            return Err(if roh.is_instance_of::<pyo3::types::PyInt>() {
                PyOverflowError::new_err("Python int too large to convert to C long")
            } else {
                PyTypeError::new_err(format!(
                    "'{}' object cannot be interpreted as an integer",
                    roh.get_type()
                        .name()
                        .map(|n| n.to_string())
                        .unwrap_or_else(|_| "?".into())
                ))
            });
        }
    };
    if nummer < 0 {
        // Gemessen - und ja, die Zahl ist die der **vorhandenen** Hebel.
        return Err(crate::umwandlung::PygameError::new_err(
            "There are 0 joysticks available",
        ));
    }
    joystick::oeffnen(nummer as usize).map_err(anzeige_fehler)?;
    // Unerreichbar: `oeffnen` gibt immer einen Fehler.
    Py::new(py, PyJoystick)
}

/// `pygame.joystick.init()`.
#[pyfunction]
#[pyo3(name = "init")]
pub fn hebel_init() {
    joystick::init();
}

/// `pygame.joystick.quit()`.
#[pyfunction]
#[pyo3(name = "quit")]
pub fn hebel_quit() {
    joystick::quit();
}

/// `pygame.joystick.get_init()`.
#[pyfunction]
#[pyo3(name = "get_init")]
pub fn hebel_get_init() -> bool {
    joystick::get_init()
}

/// `pygame.joystick.get_count()`.
#[pyfunction]
#[pyo3(name = "get_count")]
pub fn hebel_get_count() -> PyResult<usize> {
    joystick::get_count().map_err(anzeige_fehler)
}
