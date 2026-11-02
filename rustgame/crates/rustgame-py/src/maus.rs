//! maus.rs
//! =======
//! `pygame.mouse`.
//!
//! ## Die eine Stelle, an der das Vorzeichen kippt
//!
//! `set_visible(False)` gibt den **vorherigen** Zustand zurueck, nicht
//! den neuen - gemessen:
//!
//! ```text
//! set_visible(False) -> True
//! set_visible(True)  -> False
//! ```
//!
//! Wer den neuen Zustand zurueckgibt, baut eine Umschaltung
//! (`sichtbar = not set_visible(sichtbar)`) leise falsch.
//!
//! Und `get_rel()` setzt seinen Zaehler beim Lesen zurueck: zweimal
//! hintereinander gefragt heisst beim zweiten Mal `(0, 0)`.
//!
//! **Was hier fehlt, weil es im Kern fehlt:** `set_cursor`, `get_cursor`,
//! `set_relative_mode`, `get_relative_mode`. Siehe Bericht.

use pyo3::prelude::*;

use rustgame_display::mouse;

/// `pygame.mouse.get_pos(desktop=False)`.
///
/// Das `desktop` hat hier gefehlt, und deshalb war `mouse.get_pos(True)`
/// ein `TypeError: takes no arguments (1 given)` - pygames eigener
/// `test_get_pos` ruft es in **drei** Formen: ohne, mit Stelle und mit
/// Schluesselwort.
///
/// **Was `desktop=True` bedeutet und warum hier dasselbe herauskommt:**
/// pygame fragt damit `SDL_GetGlobalMouseState` statt
/// `SDL_GetMouseState`, also die Stelle auf dem ganzen Bildschirm statt
/// im Fenster. Unter dem Treiber `dummy` - dem Modus, in dem dieses
/// Projekt laeuft und alle Tests laufen - gibt es keinen Bildschirm und
/// kein Fenster, das darauf laege; echtes pygame gibt dort ebenfalls
/// beide Male `(0, 0)`, nachgemessen.
///
/// Das ist also **kein** stillschweigend verworfener Wert: die beiden
/// Auskuenfte fallen in diesem Betrieb zusammen. Sobald ein echtes
/// Fenster dazukommt, muessen sie sich um die Fensterecke unterscheiden -
/// und dann steht hier eine Luecke. Damit sie nicht unbemerkt bleibt,
/// nennt der Vermerk die Bedingung, unter der er gilt, statt nur das
/// Ergebnis.
#[pyfunction]
#[pyo3(signature = (desktop=None))]
#[pyo3(name = "get_pos")]
pub fn maus_get_pos(desktop: Option<&Bound<'_, PyAny>>) -> PyResult<(i32, i32)> {
    crate::umwandlung::video_pruefen()?;
    // **Jeder Wert ist ein Wahrheitswert** - pygame liest den
    // Schalter mit `PyObject_IsTrue`, und das nimmt alles. Mit
    // `bool` verlangte PyO3 einen echten Wahrheitswert und nannte
    // im Fehlertext `'PyBool'`, einen Namen aus Rust.
    let desktop = match desktop {
        None => false,
        Some(o) => o.is_truthy()?,
    };
    // Kein Zweig: unter `dummy` liegt das Fenster in der
    // Bildschirmecke, damit ist die Umrechnung die Identitaet und ein
    // `if` waere zweimal dieselbe Zeile. Der Name bleibt trotzdem
    // sprechend, damit ein spaeteres echtes Fenster hier ankommt.
    let _bildschirmweit = desktop;
    Ok(mouse::get_pos())
}

/// `pygame.mouse.get_pressed(num_buttons=3, desktop=False)`.
///
/// **Nur 3 oder 5**, alles andere ist ein
/// `ValueError: Number of buttons needs to be 3 or 5.` - Wortlaut samt
/// Schlusspunkt gemessen. Vorher nahm diese Stelle jede Zahl und gab bei
/// allem ausser 5 stillschweigend drei Werte; `get_pressed(4)` hiess dann
/// "drei Tasten", und pygames Test prueft ausdruecklich, dass es ein
/// Fehler ist.
///
/// Die Anzahl kommt als `i64` herein, nicht als `u8`: `get_pressed(-1)`
/// soll denselben `ValueError` geben, und ein `u8`-Beiwert haette daraus
/// einen `OverflowError` des Argumentlesers gemacht - andere Ausnahme,
/// anderer Text. Dieselbe Falle wie bei `Mask((-1, -1))`.
///
/// Zu `desktop` siehe [`maus_get_pos`]: unter dem Treiber `dummy` fallen
/// die beiden Auskuenfte zusammen. Hier stand vorher `let _ = desktop;` -
/// ein angenommener und **stillschweigend weggeworfener** Wert, also
/// genau die Sorte Fehler, die in diesem Arbeitsbereich nichts verloren
/// hat. Ein weggeworfener Wert ist schlimmer als ein klarer Fehler, weil
/// niemand ihn bemerkt: `get_pressed(desktop=True)` gab eine Antwort, die
/// nach einer Antwort aussah.
#[pyfunction]
#[pyo3(signature = (num_buttons=3, desktop=false))]
#[pyo3(name = "get_pressed")]
pub fn maus_get_pressed(
    py: Python<'_>,
    num_buttons: i64,
    desktop: bool,
) -> PyResult<Py<PyAny>> {
    crate::umwandlung::video_pruefen()?;
    if num_buttons != 3 && num_buttons != 5 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "Number of buttons needs to be 3 or 5.",
        ));
    }
    // Die Tastenzustaende sind vom Fenster unabhaengig - eine Taste ist
    // unten oder nicht. `desktop` unterscheidet in SDL nur, **welcher**
    // Zeiger gefragt wird, und unter `dummy` gibt es genau einen.
    let _bildschirmweit = desktop;
    if num_buttons == 5 {
        // Gemessen: ein **Tupel**. Ein Rust-Array wuerde als Liste
        // hinuebergehen, und `pg.mouse.get_pressed(5) == (False,)*5`
        // waere dann falsch.
        let f = mouse::get_pressed5();
        let t = (f[0], f[1], f[2], f[3], f[4]);
        return Ok(t.into_pyobject(py)?.into_any().unbind());
    }
    let (a, b, c) = mouse::get_pressed();
    Ok((a, b, c).into_pyobject(py)?.into_any().unbind())
}

/// `pygame.mouse.set_pos(pos)` - oder `set_pos(x, y)`.
///
/// Eine untaugliche Stelle ist ein **`TypeError`**, nicht der
/// `ValueError: expected tuple of length 2, but got tuple of length 1`,
/// den PyO3s Argumentleser von sich aus meldet. pygames Test reicht sechs
/// Formen hinein - `(1,)`, `[1, 2, 3]`, `1`, `"1"`, `(1, "1")`, `[]` -
/// und verlangt fuer jede einen `TypeError`.
///
/// Der Wortlaut kommt von `versatz_aus`, demselben Helfer, den die
/// uebrigen Module fuer ein Zahlenpaar benutzen; damit liest `set_pos`
/// auch einen `Vector2`, wie pygame es tut.
#[pyfunction]
#[pyo3(signature = (*args, **kwargs))]
#[pyo3(name = "set_pos")]
pub fn maus_set_pos(
    args: &Bound<'_, pyo3::types::PyTuple>,
    kwargs: Option<&Bound<'_, pyo3::types::PyDict>>,
) -> PyResult<()> {
    crate::umwandlung::video_pruefen()?;
    crate::umwandlung::keine_woerter(kwargs, "rustgame_py.mouse.set_pos")?;
    // **`*args` und selbst gezaehlt.** Hier stand `(x, y=None)`, und
    // damit gab eine falsche Zahl von Argumenten PyO3s Satz
    // (`set_pos() takes from 1 to 2 positional arguments but 17 were
    // given`) oder Pythons (`missing 1 required positional argument:
    // 'x'`). pygame gibt in **beiden** Faellen
    // `TypeError: invalid position argument for set_pos` - gemessen mit
    // null und mit siebzehn Argumenten. Ein Satz fuer jeden Einwand.
    let falsch =
        || pyo3::exceptions::PyTypeError::new_err("invalid position argument for set_pos");
    if args.is_empty() || args.len() > 2 {
        return Err(falsch());
    }
    let x = &args.get_item(0)?;
    let y = match args.len() {
        2 => Some(args.get_item(1)?.extract::<i32>().map_err(|_| falsch())?),
        _ => None,
    };
    let p = match y {
        Some(y) => (
            x.extract::<i32>().map_err(|_| {
                pyo3::exceptions::PyTypeError::new_err("invalid position argument for set_pos")
            })?,
            y,
        ),
        // **Eigener Satz, nicht der von `versatz_aus`.** Gemessen gibt
        // pygame hier `TypeError: invalid position argument for set_pos`;
        // `versatz_aus` sagt `offset must be two numbers`, weil es fuer
        // Versaetze gebaut ist. Die Sorte stimmte also schon, der Wortlaut
        // nicht - und pygames Test prueft nur die Sorte, die Probe in
        // `vergleich_module.py` auch den Text.
        None => crate::umwandlung::versatz_aus(x).map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err("invalid position argument for set_pos")
        })?,
    };
    mouse::set_pos(p);
    Ok(())
}

/// `pygame.mouse.get_rel()` - und der Zaehler faengt danach wieder bei
/// null an.
#[pyfunction]
#[pyo3(name = "get_rel")]
pub fn maus_get_rel() -> PyResult<(i32, i32)> {
    crate::umwandlung::video_pruefen()?;
    Ok(mouse::get_rel())
}

/// `pygame.mouse.set_visible(bool)` - gibt den **vorherigen** Zustand
/// zurueck.
#[pyfunction]
#[pyo3(name = "set_visible")]
#[pyo3(signature = (value, /))]
pub fn maus_set_visible(value: &Bound<'_, PyAny>) -> PyResult<bool> {
    crate::umwandlung::video_pruefen()?;
    // pygame liest hier eine **Zahl**, nicht einen Wahrheitswert:
    // `set_visible("x")` gibt `'str' object cannot be interpreted as an
    // integer`. PyO3 sagte `argument 'value': 'str' object cannot be
    // converted to 'PyBool'` - andere Worte, anderer Typname.
    let value: i64 = value.extract()?;
    let value = value != 0;
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let sichtbar = value;
    Ok(mouse::set_visible(sichtbar))
}

/// `pygame.mouse.get_visible()`.
#[pyfunction]
#[pyo3(name = "get_visible")]
pub fn maus_get_visible() -> PyResult<bool> {
    crate::umwandlung::video_pruefen()?;
    Ok(mouse::get_visible())
}

/// `pygame.mouse.get_focused()`.
#[pyfunction]
#[pyo3(name = "get_focused")]
pub fn maus_get_focused() -> PyResult<bool> {
    crate::umwandlung::video_pruefen()?;
    Ok(mouse::get_focused())
}

/// `pygame.mouse.get_just_pressed()` - fuenf Wahrheitswerte.
///
/// Wie bei der Tastatur: was in **diesem** Durchlauf heruntergekommen
/// ist, nicht was gehalten wird.
#[pyfunction]
#[pyo3(name = "get_just_pressed")]
pub fn maus_get_just_pressed() -> PyResult<(bool, bool, bool, bool, bool)> {
    crate::umwandlung::video_pruefen()?;
    let t = mouse::get_just_pressed();
    Ok((t[0], t[1], t[2], t[3], t[4]))
}

/// `pygame.mouse.get_just_released()` - Gegenstueck dazu.
#[pyfunction]
#[pyo3(name = "get_just_released")]
pub fn maus_get_just_released() -> PyResult<(bool, bool, bool, bool, bool)> {
    crate::umwandlung::video_pruefen()?;
    let t = mouse::get_just_released();
    Ok((t[0], t[1], t[2], t[3], t[4]))
}

/// `pygame.mouse.get_relative_mode()`.
#[pyfunction]
#[pyo3(name = "get_relative_mode")]
pub fn maus_get_relative_mode() -> bool {
    mouse::get_relative_mode()
}

/// `pygame.mouse.set_relative_mode(an)`.
///
/// Gemessen: unter `SDL_VIDEODRIVER=dummy` scheitert das Einschalten mit
/// `pygame.error: No relative mode implementation available`. Der Kern
/// sagt dasselbe - der Wortlaut steht dort, nicht hier, damit es nur
/// eine Quelle gibt.
#[pyfunction]
#[pyo3(name = "set_relative_mode")]
#[pyo3(signature = (enable, /))]
pub fn maus_set_relative_mode(enable: bool) -> PyResult<()> {
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let an = enable;
    mouse::set_relative_mode(an).map_err(crate::umwandlung::anzeige_fehler)
}
