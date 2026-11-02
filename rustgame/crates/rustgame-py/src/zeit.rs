//! zeit.rs
//! =======
//! `pygame.time` samt `pygame.time.Clock`.
//!
//! ## Die 16, die keine 17 ist
//!
//! `Clock.tick(60)` gibt **16** zurueck, nicht 17. pygame rechnet
//! `(int)((1.0f / framerate) * 1000.0f)` - in `float`, und schneidet dann
//! ab. Ueber eine Sekunde macht das mehr als ein Bild Unterschied, und es
//! ist die Stelle, an der ein Nachbau auffliegt. Die Rechnung steht im
//! Kern ([`rustgame_core::time::Clock`]); hier wird sie nur
//! durchgereicht - und im Vergleich gegen echtes pygame nachgewiesen.
//!
//! ## Zeitgeber
//!
//! `set_timer` legt im Kern nur einen Auftrag an; **zustellen** muss ihn
//! jemand. In pygame macht das SDL, hier tut es diese Huelle: jede
//! Abfrage der Ereignisschlange laesst vorher
//! [`rustgame_core::time::faellige_zeitgeber`] leerlaufen und legt die
//! faelligen Nummern ab (siehe `ereignis.rs`). Ohne das waere
//! `pygame.time.set_timer` eine Buchhaltung ohne Wirkung.

use pyo3::prelude::*;

use rustgame_core::time;

use crate::ereignis::PyEvent;

/// `pygame.time.get_ticks` - Millisekunden seit `pygame.init()`.
#[pyfunction]
#[pyo3(name = "get_ticks")]
pub fn zeit_get_ticks() -> u64 {
    time::get_ticks()
}

/// `pygame.time.wait` - schlaeft grob und gibt die wirklich verstrichene
/// Zeit zurueck.
#[pyfunction]
#[pyo3(name = "wait")]
#[pyo3(signature = (milliseconds, /))]
pub fn zeit_wait(py: Python<'_>, milliseconds: &Bound<'_, PyAny>) -> PyResult<u64> {
    // pygames eigener Satz; PyO3 haengte sonst
    // `argument 'milliseconds': ` vor den seinen.
    let milliseconds: i64 = milliseconds
        .extract()
        .map_err(|_| pyo3::exceptions::PyTypeError::new_err("wait requires one integer argument"))?;
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let millisekunden = milliseconds;
    // Ohne `allow_threads` haenge der ganze Python-Prozess an der GIL,
    // solange gewartet wird - in pygame laeuft das Warten in C ohne GIL.
    Ok(py.detach(|| time::wait(millisekunden)))
}

/// `pygame.time.delay` - wartet genauer und kostet dafuer einen Kern.
#[pyfunction]
#[pyo3(name = "delay")]
#[pyo3(signature = (milliseconds, /))]
pub fn zeit_delay(py: Python<'_>, milliseconds: &Bound<'_, PyAny>) -> PyResult<u64> {
    let milliseconds: i64 = milliseconds
        .extract()
        .map_err(|_| pyo3::exceptions::PyTypeError::new_err("delay requires one integer argument"))?;
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let millisekunden = milliseconds;
    Ok(py.detach(|| time::delay(millisekunden)))
}

/// `pygame.time.set_timer(event, millis, loops=0)`.
///
/// `millis <= 0` haelt einen laufenden Zeitgeber an, `loops = 0` heisst
/// unbegrenzt - beides wie in pygame. Als `event` nimmt pygame eine Zahl
/// **oder** ein ganzes Ereignis.
///
/// ## Was hier geprueft wird und was pygame durchlaesst
///
/// Gemessen, und die Verteilung ist nicht die naheliegende:
///
/// | Aufruf | pygame |
/// |---|---|
/// | `set_timer(-1, 10)` | `ValueError: event type out of range` |
/// | `set_timer(NUMEVENTS, 10)` | dasselbe (`NUMEVENTS` = 65535, also **ausschliesslich**) |
/// | `set_timer("string", 15)` | `TypeError: first argument must be an event type or event object` |
/// | `set_timer(T, -1)` | **geht durch** - haelt den Zeitgeber an |
/// | `set_timer(T, 10, -1)` | **geht durch** |
///
/// Die Grenze wird also nur am **ersten** Argument gezogen; `millis` und
/// `loops` nehmen jede Zahl. Ohne diese Pruefung meldete die Stelle
/// `OverflowError: can't convert negative int to unsigned` aus PyO3s
/// Argumentleser - eine dritte Ausnahmeart mit einem Text, der von
/// rustgame erzaehlt statt von pygame.
///
/// `loops` kommt als `i64` herein, damit `-1` nicht schon am Argumentleser
/// scheitert; unbegrenzt heisst weiterhin `0`, und eine negative Zahl
/// verhaelt sich wie pygame, das sie schlicht annimmt.
#[pyfunction]
#[pyo3(signature = (event, millis, loops=0))]
#[pyo3(name = "set_timer")]
pub fn zeit_set_timer(event: &Bound<'_, PyAny>, millis: i64, loops: i64) -> PyResult<()> {
    // **Ein Riegel, der nur aufgeht.** Gemessen sagt pygame hier
    // `pygame.error: pygame is not initialized`, solange noch nie etwas
    // hochgefahren wurde; nach `display.init()` geht es, und nach
    // `pygame.quit()` geht es **weiter**. Siehe [`crate::ZEITGEBER_JE`].
    if !crate::zeitgeber_je() {
        return Err(crate::umwandlung::PygameError::new_err(
            "pygame is not initialized",
        ));
    }
    // **Ein Ereignisobjekt bringt sein Woerterbuch mit** - und zwar fuer
    // jede Wiederholung dasselbe. Siehe `ereignis::zeitgeber_dict_setzen`.
    let typ = match event.extract::<PyRef<PyEvent>>() {
        Ok(e) => {
            let py = event.py();
            crate::ereignis::zeitgeber_dict_setzen(
                e.typ(),
                if millis > 0 { Some(e.dict_roh(py)) } else { None },
            );
            e.typ()
        }
        Err(_) => {
            // **Erst auf Zahl pruefen, dann auf Bereich.** Umgekehrt
            // waere `set_timer("string", 15)` ein `ValueError`, und
            // pygame nimmt dort `TypeError`.
            //
            // `extract::<i64>()` statt `u32`, sonst faengt der
            // Argumentleser `-1` mit `OverflowError` ab, bevor die
            // Bereichspruefung ueberhaupt dran ist.
            let n = event.extract::<i64>().map_err(|_| {
                pyo3::exceptions::PyTypeError::new_err(
                    "first argument must be an event type or event object",
                )
            })?;
            if !(0..rustgame_display::konstanten::NUMEVENTS as i64).contains(&n) {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "event type out of range",
                ));
            }
            n as u32
        }
    };
    // Ohne Ereignisobjekt gibt es kein Woerterbuch - und ein `millis <= 0`
    // haelt den Zeitgeber an, also auch das Gemerkte weg.
    if millis <= 0 {
        crate::ereignis::zeitgeber_dict_setzen(typ, None);
    }
    time::set_timer(typ, millis, loops.max(0) as u32);
    Ok(())
}

/// `pygame.time.Clock`.
#[pyclass(name = "Clock", module = "rustgame_py.time")]
pub struct PyClock(time::Clock);

#[pymethods]
impl PyClock {
    #[new]
    fn new() -> PyClock {
        PyClock(time::Clock::new())
    }

    /// Einen Durchlauf abschliessen. `framerate = 0` heisst "nicht
    /// warten, nur messen".
    #[pyo3(signature = (framerate=0.0))]
    fn tick(&mut self, py: Python<'_>, framerate: f32) -> u64 {
        py.detach(|| self.0.tick(framerate))
    }

    /// Wie [`Self::tick`], aber der letzte Rest wird aktiv abgewartet.
    #[pyo3(signature = (framerate=0.0))]
    fn tick_busy_loop(&mut self, py: Python<'_>, framerate: f32) -> u64 {
        py.detach(|| self.0.tick_busy_loop(framerate))
    }

    /// Mittel ueber die **letzten zehn** `tick`. Vor dem elften `tick`
    /// steht hier `0.0` - gemessen, kein Fehler.
    fn get_fps(&self) -> f32 {
        self.0.get_fps()
    }
    /// Millisekunden zwischen den beiden letzten `tick`, **mit** Wartezeit.
    fn get_time(&self) -> u64 {
        self.0.get_time()
    }
    /// Dieselbe Spanne **ohne** Wartezeit.
    fn get_rawtime(&self) -> u64 {
        self.0.get_rawtime()
    }

    fn __repr__(&self) -> String {
        self.0.to_string()
    }
    fn __str__(&self) -> String {
        self.0.to_string()
    }
}
