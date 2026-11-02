//! midigeraet.rs
//! =============
//! Die Geraeteseite von `pygame.midi` - die Huelle um
//! [`rustgame_mixer::midi`].
//!
//! Das Modul `pygame.midi` selbst ist Python (siehe `lib.rs`): es rechnet
//! Notennummern in Frequenzen um, baut `pygame`-Ereignisse aus
//! MIDI-Nachrichten und kennt die Zaehlweise der Kanaele. Hier steht nur,
//! was ein Geraet anfassen muss.
//!
//! ## Warum die Namen nach Python so aussehen und nicht anders
//!
//! `Input` und `Output` in pygame sind Python-Klassen um PortMidi. Die
//! beiden Klassen hier sind **nicht** sie, sondern das, was sie
//! benutzen - `_MidiAusgang` und `_MidiEingang`, mit Unterstrich. Der
//! Grund steht in `midi.py`: `pygame.midi.Output.write` nimmt eine Liste
//! von Nachrichten mit Zeitstempeln, `note_on` rechnet Kanal und Status
//! zusammen, `write_sys_ex` nimmt eine ganze Systemnachricht. Das ist
//! Zaehlarbeit und gehoert nach Python, wo pygame sie auch hat; hierher
//! gehoert nur das Senden.

use pyo3::prelude::*;
use pyo3::types::PyBytes;

use rustgame_mixer::midi;

use crate::umwandlung::PygameError;

/// Wie viele MIDI-Geraete das System hat.
#[pyfunction]
pub fn midi_anzahl() -> usize {
    midi::anzahl()
}

/// Die Nummer des ersten Eingangs - `-1`, wenn es keinen gibt.
#[pyfunction]
pub fn midi_erster_eingang() -> i32 {
    midi::erster_eingang()
}

/// Die Nummer des ersten Ausgangs - `-1`, wenn es keinen gibt.
#[pyfunction]
pub fn midi_erster_ausgang() -> i32 {
    midi::erster_ausgang()
}

/// Die Auskunft zu einem MIDI-Geraet, wie PortMidi sie gibt.
///
/// `(schnittstelle, name, eingang, ausgang, offen)` - die ersten zwei als
/// `bytes`. Als eigener Name, weil Clippy einen fuenffach geschachtelten
/// Typ zu Recht fuer unleserlich haelt.
pub type Geraeteauskunft<'p> = (
    Bound<'p, PyBytes>,
    Bound<'p, PyBytes>,
    i32,
    i32,
    i32,
);

/// Die Beschreibung eines Geraets, oder `None` hinter dem Ende.
///
/// **`None` und kein Fehler** - gemessen gibt `get_device_info` fuer eine
/// Nummer ausserhalb des Bereichs `None` zurueck, und pygames eigener
/// Test prueft es ausdruecklich (`self.assertIsNone(info)`).
///
/// Die beiden Namen kommen als `bytes` heraus, so wie bei pygame:
/// `(b'MMSystem', b'Microsoft GS Wavetable Synth', 0, 1, 0)`.
#[pyfunction]
pub fn midi_geraet<'p>(
    py: Python<'p>,
    nummer: usize,
) -> Option<Geraeteauskunft<'p>> {
    let g = midi::geraet(nummer)?;
    Some((
        PyBytes::new(py, g.schnittstelle.as_bytes()),
        PyBytes::new(py, g.name.as_bytes()),
        i32::from(g.eingang),
        i32::from(g.ausgang),
        // `opened` ist immer 0. PortMidi fuehrt dort mit, ob **dieses**
        // Programm das Geraet offen hat; gemessen steht bei pygame auf
        // dieser Maschine ebenfalls eine Null, auch waehrend ein
        // `Output` offen ist - die Auskunft stammt aus einer Tabelle, die
        // PortMidi beim Aufzaehlen fuellt und danach nicht nachzieht.
        0,
    ))
}

/// Ein offener MIDI-Ausgang.
#[pyclass(name = "_MidiAusgang", module = "rustgame_py.midi", unsendable)]
pub struct PyMidiAusgang {
    inner: midi::Ausgang,
}

#[pymethods]
impl PyMidiAusgang {
    #[new]
    fn new(nummer: usize) -> PyResult<PyMidiAusgang> {
        Ok(PyMidiAusgang {
            inner: midi::Ausgang::oeffnen(nummer)
                .map_err(|e| PygameError::new_err(e.to_string()))?,
        })
    }

    /// Schickt **eine** vollstaendige Nachricht.
    fn senden(&mut self, bytes: &[u8]) -> PyResult<()> {
        self.inner
            .senden(bytes)
            .map_err(|e| PygameError::new_err(e.to_string()))
    }

    fn schliessen(&mut self) {
        self.inner.schliessen();
    }

    #[getter]
    fn offen(&self) -> bool {
        self.inner.offen()
    }
}

/// Ein offener MIDI-Eingang.
#[pyclass(name = "_MidiEingang", module = "rustgame_py.midi", unsendable)]
pub struct PyMidiEingang {
    inner: midi::Eingang,
}

#[pymethods]
impl PyMidiEingang {
    #[new]
    #[pyo3(signature = (nummer, platz=4096))]
    fn new(nummer: usize, platz: usize) -> PyResult<PyMidiEingang> {
        Ok(PyMidiEingang {
            inner: midi::Eingang::oeffnen(nummer, platz)
                .map_err(|e| PygameError::new_err(e.to_string()))?,
        })
    }

    /// Liegt etwas an?
    fn anliegt(&self) -> PyResult<bool> {
        self.inner
            .anliegt()
            .map_err(|e| PygameError::new_err(e.to_string()))
    }

    /// Nimmt bis zu `anzahl` Nachrichten heraus - je `(bytes, zeit_ms)`.
    fn lesen<'p>(
        &self,
        py: Python<'p>,
        anzahl: usize,
    ) -> PyResult<Vec<(Bound<'p, PyBytes>, u64)>> {
        let roh = self
            .inner
            .lesen(anzahl)
            .map_err(|e| PygameError::new_err(e.to_string()))?;
        Ok(roh
            .into_iter()
            .map(|(b, t)| (PyBytes::new(py, &b), t))
            .collect())
    }

    fn schliessen(&mut self) {
        self.inner.schliessen();
    }

    #[getter]
    fn offen(&self) -> bool {
        self.inner.offen()
    }
}
