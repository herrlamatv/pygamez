//! ton.rs
//! ======
//! `pygame.mixer`, `pygame.mixer.Sound`, `pygame.mixer.Channel` und
//! `pygame.mixer_music` - die Python-Huelle um [`rustgame_mixer`].
//!
//! Die Rechnung steht **nicht hier**. Wie lange ein Klang laeuft, ob ein
//! Kanal belegt ist, wie eine Lautstaerke einrastet - das alles macht die
//! Kiste; diese Datei uebersetzt nur zwischen Python und Rust und achtet
//! dabei auf drei Dinge, die die Kiste nicht wissen kann.
//!
//! ## 1. Kein Aufruf hier darf ein Tongeraet oeffnen, das keiner wollte
//!
//! Das ist die Falle dieser Aufgabe, und sie ist schon einmal
//! zugeschnappt: der erste Testlauf der Kiste hat das echte Geraet
//! aufgemacht.
//!
//! Die Kiste entscheidet in `treiber::oeffnen` an `cfg!(test)` und an
//! `RUSTGAME_AUDIODRIVER`. **`cfg!(test)` hilft hier gar nichts**: es ist
//! nur innerhalb der Kiste wahr, und diese Huelle bindet sie als
//! gewoehnliche Abhaengigkeit ein - ohne `cfg(test)` uebersetzt. Die
//! Begruendung dazu steht ausgeschrieben bei `oeffnen()` in
//! `rustgame-display/src/backend/mod.rs`.
//!
//! Also bleibt nur die Umgebung, und `treiber_waehlen` liest sie: der
//! Kern kennt `RUSTGAME_AUDIODRIVER` selbst, `SDL_AUDIODRIVER` wird hier
//! uebersetzt. Damit laeuft ein Programm, das heute schon
//! `SDL_AUDIODRIVER=dummy` setzt - und das tut jedes Messskript in diesem
//! Projekt -, ohne Aenderung stumm. `anzeige::treiber_waehlen` macht
//! dasselbe mit `SDL_VIDEODRIVER`.
//!
//! ## 2. pygame prueft `mixer not initialized` oefter als die Kiste
//!
//! Die Kiste laesst `Sound::get_length`, `Sound::get_raw`,
//! `Channel::get_sound` und ein gutes Dutzend weitere Abfragen ohne
//! laufenden Mixer durch - sie rechnen ja weiter richtig. Echtes pygame
//! wirft dort `pygame.error: mixer not initialized`, gemessen fuer jeden
//! einzelnen Aufruf. Diese Huelle stellt `laeuft` davor, damit ein
//! `except pygame.error` in beiden Engines gleich greift.
//!
//! ## 3. Sieben Aufrufe bringen echtes pygame zum **Absturz**
//!
//! Gemessen an pygame-ce 2.5.7 mit `SDL_AUDIODRIVER=dummy`, jeder Aufruf
//! in einem eigenen Prozess, jeweils nach `mixer.quit()`:
//!
//! ```text
//! s.play()  c.play(s)  c.queue(s)  c.get_sound()
//! c.get_queue()  c.get_endevent()  c.set_endevent(1)
//! ```
//!
//! Alle sieben reissen den Python-Prozess ab - `s.play()` und
//! `c.play(s)` mit einem Speicherzugriffsfehler, die uebrigen fuenf mit
//! Pythons Notausgang. Kein `pygame.error`, keine Ausnahme, nichts zum
//! Abfangen. Ihre unmittelbaren Nachbarn (`s.stop()`, `c.pause()`,
//! `c.get_busy()`) melden dagegen sauber `mixer not initialized` und
//! ueberleben - nachgemessen, jeder Aufruf einzeln. Das ist offensicht-
//! lich kein gewolltes Verhalten, sondern ein fehlendes
//! `MIXER_INIT_CHECK()` in `src_c/mixer.c`.
//!
//! **Nachgebaut wird hier der Nachbar, nicht der Absturz.** Alle sieben
//! melden `mixer not initialized` wie der Rest ihrer Klasse.

use pyo3::exceptions::{PyFileNotFoundError, PyIndexError, PyTypeError, PyValueError};
use std::cell::RefCell;
use std::collections::HashMap;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyModule, PyTuple};

use rustgame_mixer::treiber::Wahl;
use rustgame_mixer::{mixer, music, Channel, Einstellungen, Fehler, Sound};

use crate::umwandlung::PygameError;

// ---------------------------------------------------------------------------
//  Fehler und Vorbedingungen
// ---------------------------------------------------------------------------

/// Fehler der Kiste - zu den Ausnahmen, die echtes pygame wirft.
///
/// pygame benutzt hier drei verschiedene Klassen, und die Kiste sagt in
/// jeder Variante dazu, welche gemeint ist - also wird das zugeordnet
/// statt alles zu `pygame.error` zu verflachen. Gemessen:
///
/// ```text
/// mixer.Channel(8)         -> IndexError: invalid channel index
/// mixer.init(size=24)      -> ValueError: unsupported size 24
/// mixer.Sound('/nix/da')   -> FileNotFoundError: No file '/nix/da' found ...
/// mixer.get_num_channels() -> pygame.error: mixer not initialized
/// ```
fn ton_fehler(e: Fehler) -> PyErr {
    match e {
        Fehler::UngueltigerKanal(_) => PyIndexError::new_err(e.to_string()),
        Fehler::Wert(s) => PyValueError::new_err(s),
        Fehler::Datei(s) => PyFileNotFoundError::new_err(s),
        andere => PygameError::new_err(andere.to_string()),
    }
}

/// Laeuft der Mixer? Sonst die Ausnahme, die pygame wirft.
///
/// Gemessen ist der Wortlaut `mixer not initialized`, die Klasse
/// `pygame.error` (erbt von `RuntimeError`).
fn laeuft() -> PyResult<()> {
    if mixer::get_init().is_none() {
        return Err(PygameError::new_err("mixer not initialized"));
    }
    Ok(())
}

/// Liest die Treiberwahl aus der Umgebung - siehe Punkt 1 im Modulkopf.
///
/// `RUSTGAME_AUDIODRIVER` wertet der Kern selbst aus, der soll hier also
/// nicht ueberschrieben werden. `SDL_AUDIODRIVER` kennt er nicht; die
/// wird hier uebersetzt.
fn treiber_waehlen() {
    if std::env::var("RUSTGAME_AUDIODRIVER").is_ok() {
        return;
    }
    if let Ok(v) = std::env::var("SDL_AUDIODRIVER") {
        if v.eq_ignore_ascii_case("dummy") {
            mixer::set_treiber(Wahl::Dummy);
        }
    }
}

// ---------------------------------------------------------------------------
//  Was Python als Klangquelle hereinreicht
// ---------------------------------------------------------------------------

/// Die drei Arten, auf die ein Klang in pygame hereinkommt.
///
/// **Gemessen, und der Unterschied ist echt** - mit denselben 44144
/// Bytes einer WAV-Datei:
///
/// ```text
/// Sound('t.wav')          -> 1.0                  (Datei mit 1 s Inhalt)
/// Sound(open('t.wav'))    -> 1.0                  (Dateiobjekt: gelesen)
/// Sound(BytesIO(bytes))   -> 0.25                 (Dateiobjekt: gelesen)
/// Sound(bytes)            -> 0.250249445438385    (roher Puffer!)
/// Sound(buffer=bytes)     -> 0.250249445438385    (roher Puffer)
/// Sound(memoryview(...))  -> 0.250249445438385    (roher Puffer)
/// ```
///
/// Ein blankes `bytes` ist also **kein** Dateiinhalt, sondern werden als
/// Abtastwerte gelesen - der WAV-Kopf klingt dann mit. Wer `Sound(daten)`
/// statt `Sound(BytesIO(daten))` schreibt, bekommt in pygame keinen
/// Fehler, sondern eine Datei mit falscher Laenge. Diese Huelle bildet
/// das nach, statt es stillschweigend zu berichtigen: ein Programm, das
/// sich darauf verlaesst, soll dieselben Zahlen sehen.
enum Quelle {
    /// Ein Dateiname - `str`, `bytes` oder `pathlib.Path`.
    ///
    /// Das zweite Feld ist, **wie der Name im Aufruf stand** - fuer die
    /// Fehlermeldung. Gemessen behaelt pygame dort die Schreibweise des
    /// Objekts: `music.load(b"fehlt.wav")` sagt
    /// `No file 'b'fehlt.wav'' found in working directory ...`, mit den
    /// Bytes-Anfuehrungszeichen mitten im Satz. Ein Nachbau, der nur den
    /// entschluesselten Pfad einsetzt, weicht genau in dieser Zeile ab.
    Pfad(String, String),
    /// Ein schon vorhandener Klang - `Sound(anderer_sound)`.
    Klang(Sound),
    /// Der **Inhalt** einer Klangdatei, aus einem Dateiobjekt gelesen.
    Datei(Vec<u8>),
    /// **Rohe** Abtastwerte im Ausgabeformat des Mixers.
    Puffer(Vec<u8>),
}

/// Ordnet ein Python-Objekt einer der drei Arten zu.
///
/// Die Reihenfolge ist die von `sound_init` in `src_c/mixer.c`: erst der
/// Dateiname, dann das Dateiobjekt (alles mit `read`), dann der Puffer.
fn quelle_aus(o: &Bound<'_, PyAny>) -> PyResult<Quelle> {
    // **Ein Klang ist eine gueltige Quelle** - gemessen gibt
    // `Sound(anderer)` einen Klang derselben Laenge. Das steht hier vorn,
    // weil ein `Sound` sonst am Ende als "Unrecognized argument (type
    // Sound)" abgewiesen wuerde; die Meldung war richtig gebaut und
    // beantwortete die falsche Frage.
    if let Ok(vorhanden) = o.extract::<PyRef<PySound>>() {
        return Ok(Quelle::Klang(vorhanden.dran()?.clone()));
    }
    if let Ok(s) = o.extract::<String>() {
        return Ok(Quelle::Pfad(s.clone(), s));
    }
    // `pathlib.Path` - gemessen nimmt pygame ihn an (`Sound(file=pathlib_path)`
    // steht sogar im Docstring).
    if o.hasattr("__fspath__")? {
        if let Ok(s) = o.call_method0("__fspath__")?.extract::<String>() {
            return Ok(Quelle::Pfad(s.clone(), s));
        }
    }
    // Ein Dateiobjekt: gelesen wird der **Inhalt**, nicht der Puffer.
    if o.hasattr("read")? {
        let daten = o.call_method0("read")?.extract::<Vec<u8>>()?;
        return Ok(Quelle::Datei(daten));
    }
    // **`bytes` ist zuerst ein Dateiname und erst dann Klangdaten.**
    //
    // Gemessen, und die Aufteilung ist feiner als erwartet:
    //
    // ```text
    // Sound(pfad_als_bytes)      -> geht, Laenge 7.10   (Pfad)
    // Sound(rohe_wav_bytes)      -> geht, Laenge 0.44   (Daten)
    // Sound(file=pfad_als_bytes) -> geht                (Pfad)
    // Sound(file=rohe_wav_bytes) -> FileNotFoundError   (nur Pfad!)
    // music.load(rohe_wav_bytes) -> FileNotFoundError   (nur Pfad!)
    // Sound(buffer=rohe_bytes)   -> geht                (nur Daten)
    // ```
    //
    // Also: **positional** versucht pygame den Pfad und faellt auf die
    // Daten zurueck; `file=` und `music.load` nehmen **nur** den Pfad.
    //
    // Hier stand vorher `Quelle::Puffer(b)` ohne Pfadversuch, und daran
    // hingen **alle 19** Fehler in `mixer_music_test.py`: dessen
    // `music_load`-Helfer laedt jede Datei zweimal, einmal als `str` und
    // einmal als `bytes`, und der zweite Aufruf endete in
    // `Unrecognized audio format` - der Dateiname wurde als Musik gelesen.
    // Die Meldung war also nicht falsch, nur die Frage.
    // **Nur echte Bytesfolgen**, keine Liste.
    //
    // `extract::<Vec<u8>>()` nimmt auch `[1, 2, 3]` und `[]` an, und damit
    // galt eine Liste als Klangpuffer. Gemessen ist sie das nicht:
    // `Sound([])` gibt `TypeError: Unrecognized argument (type list)`, und
    // `Sound(buffer=[])` den anderen Satz
    // `Expected object with buffer interface: got a list`. Zwei
    // verschiedene Wege, zwei verschiedene Saetze - beide setzen voraus,
    // dass eine Liste hier **abgewiesen** wird.
    let bytesfolge = o.is_instance_of::<pyo3::types::PyBytes>()
        || o.is_instance_of::<pyo3::types::PyByteArray>()
        || o.get_type().name().map(|n| n == "memoryview").unwrap_or(false);
    if bytesfolge {
        let b = o.extract::<Vec<u8>>()?;
        if let Ok(als_pfad) = String::from_utf8(b.clone()) {
            if ist_datei(&als_pfad) {
                return Ok(Quelle::Pfad(als_pfad, anzeige(o)));
            }
        }
        return Ok(Quelle::Puffer(b));
    }
    // Gemessen: `Sound(123)` -> TypeError: Unrecognized argument (type int),
    // `Sound(None)` -> ... (type NoneType). Der Name kommt aus Python.
    let typ = o.get_type().name()?;
    Err(PyTypeError::new_err(format!(
        "Unrecognized argument (type {typ})"
    )))
}

/// Ist das eine Datei, die sich lesen laesst - **kein Ordner**?
///
/// Hier stand `exists()`, und das ist auch fuer einen Ordner wahr. Dann
/// ging der Ordner an den Dekodierer, und heraus kam eine Meldung des
/// **Betriebssystems** in dessen Sprache:
///
/// ```text
/// pygame    No file 'rustgame' found in working directory '...'.
/// rustgame  No file 'rustgame' found: Zugriff verweigert (os error 5)
/// ```
///
/// Gemessen gibt pygame fuer einen Ordner in `Sound`, `music.load`,
/// `music.queue` und `music.get_metadata` denselben Satz wie fuer eine
/// Datei, die es nicht gibt.
fn ist_datei(p: &str) -> bool {
    std::path::Path::new(p).is_file()
}

/// pygames Satz fuer eine Datei, die es nicht gibt - **ein** Ort dafuer.
///
/// `wie` ist die Schreibweise aus dem Aufruf (siehe [`Quelle::Pfad`]).
/// `music.get_metadata` liess das Arbeitsverzeichnis weg; die Probe
/// `Fehlertexte` hat es gefunden, nachdem ihre Grenze einen Abdruck bekam.
fn datei_fehlt(wie: &str) -> PyErr {
    let wo = std::env::current_dir()
        .map(|d| d.display().to_string())
        .unwrap_or_default();
    PyFileNotFoundError::new_err(format!(
        "No file '{wie}' found in working directory '{wo}'."
    ))
}

/// Wie das Objekt im Aufruf **geschrieben** stand - fuer die
/// Fehlermeldung.
///
/// `repr()`, nicht `str()`: bei `bytes` ist das der Unterschied zwischen
/// `b'fehlt.wav'` und `fehlt.wav`, und pygame nimmt die erste Form.
fn anzeige(o: &Bound<'_, PyAny>) -> String {
    o.repr()
        .map(|r| r.to_string())
        .unwrap_or_else(|_| "?".into())
}

/// Baut aus einer [`Quelle`] den Klang.
fn klang_aus(q: Quelle) -> PyResult<Sound> {
    match q {
        Quelle::Pfad(p, wie) => {
            // pygames Wortlaut, gemessen:
            //   FileNotFoundError: No file '/nix/da.wav' found in working
            //   directory 'C:\Users\herrl\PycharmProjects\pygamez'.
            // Die Kiste formuliert es anders (sie kennt das
            // Arbeitsverzeichnis nicht), also entsteht die Meldung hier -
            // Programme zeigen sie an.
            if !ist_datei(&p) {
                return Err(datei_fehlt(&wie));
            }
            Sound::new(&p).map_err(ton_fehler)
        }
        Quelle::Datei(b) => Sound::from_bytes(&b).map_err(ton_fehler),
        Quelle::Puffer(b) => Sound::from_buffer(&b).map_err(ton_fehler),
        // Eine Kopie - `Sound` haelt seine Werte hinter einem `Rc`, das
        // Klonen kostet also nichts und teilt sie.
        Quelle::Klang(k) => Ok(k),
    }
}

// ---------------------------------------------------------------------------
//  pygame.mixer.Sound
// ---------------------------------------------------------------------------

/// `pygame.mixer.Sound` - ein Klang im Speicher.
///
/// ## `unsendable`, und das ist die richtige Aussage
///
/// Ein `Sound` teilt seine Daten und seine Lautstaerke ueber ein `Rc`
/// mit seinen Klonen, und der Mixerzustand, an dem er haengt, liegt in
/// einem `thread_local!`. Beides gilt nicht ueber Straenge hinweg: ein
/// `Sound`, der den Strang wechselte, traefe dort auf einen **anderen**
/// Mixer - einen, der gar nicht laeuft. PyO3 bindet das Objekt deshalb
/// an seinen Erzeugerstrang, statt es zuzulassen und spaeter zu
/// stuerzen. `PySurface`, `PyFont` und `PyCursor` stehen aus demselben
/// Grund so da.
#[pyclass(name = "Sound", module = "rustgame_py.mixer", unsendable, subclass)]
#[derive(Clone)]
pub struct PySound {
    /// `None` heisst: **`__init__` ist nie gelaufen**.
    ///
    /// Das kommt vor, und pygame rechnet damit. `Sound.__new__` legt dort
    /// nur den leeren Platz an; die Arbeit macht `tp_init`. Schreibt
    /// jemand eine Unterklasse, deren `__init__` den Elternteil **nicht**
    /// ruft, bleibt der Platz leer:
    ///
    /// ```python
    /// class Leer(mixer.Sound):
    ///     def __init__(self): pass
    /// Leer().get_volume()
    /// # RuntimeError: __init__() was not called on Sound object so it
    /// #               failed to setup correctly.
    /// ```
    ///
    /// Das ist kein Randfall, sondern ein eigener Test
    /// (`mixer_test.test_incorrect_subclassing`), und es zwingt zu
    /// `Option`: ein `Sound` ohne Daten muss **existieren** koennen.
    inner: Option<Sound>,
}

impl PySound {
    pub(crate) fn neu(inner: Sound) -> PySound {
        PySound { inner: Some(inner) }
    }

    /// Der Klang - oder pygames Wortlaut, wenn es keinen gibt.
    fn dran(&self) -> PyResult<&Sound> {
        self.inner.as_ref().ok_or_else(|| {
            pyo3::exceptions::PyRuntimeError::new_err(
                "__init__() was not called on Sound object so it failed to setup correctly.",
            )
        })
    }
}

#[pymethods]
impl PySound {
    /// `Sound(file)`, `Sound(file=...)`, `Sound(buffer=...)`,
    /// `Sound(array=...)`.
    ///
    /// Gemessen wirft pygame `mixer not initialized`, wenn der Mixer
    /// nicht laeuft - **vor** jeder Pruefung des Arguments.
    ///
    /// Und es nimmt genau **ein** Argument, positional **oder** benannt.
    /// Die Fehlertexte sind alle gemessen:
    ///
    /// ```text
    /// Sound()                -> TypeError: Sound takes either 1 positional
    ///                           or 1 keyword argument
    /// Sound(w, w)            -> derselbe Text
    /// Sound(file=w, buffer=b) -> derselbe Text
    /// Sound(foo=1)           -> TypeError: Unrecognized keyword argument 'foo'
    /// Sound(None)            -> TypeError: Unrecognized argument (type NoneType)
    /// Sound(file=None)       -> TypeError: not a file object
    /// Sound(buffer=None)     -> TypeError: Expected object with buffer
    ///                           interface: got a NoneType
    /// Sound(buffer='pfad')   -> TypeError: Unicode object not allowed as
    ///                           buffer object
    /// ```
    ///
    /// Deshalb `*args, **kwargs` statt benannter Beiwerte: PyO3 macht aus
    /// einem uebergebenen Python-`None` dasselbe `Option::None` wie aus
    /// einem fehlenden Argument. `Sound()` und `Sound(None)` waeren damit
    /// nicht zu unterscheiden - und sie werfen verschiedene Fehler.
    ///
    /// **`array=` ist nicht dasselbe wie `buffer=`**, auch wenn es hier
    /// lange so war. `buffer=` nimmt **Bytes** im Ausgabeformat des
    /// Mixers; `array=` nimmt ein **Feld von Zahlen** mit eigener Breite,
    /// eigener Byteordnung und eigener Form, und rechnet um. Siehe
    /// [`aus_feld`].
    #[new]
    #[pyo3(signature = (*args, **kwargs))]
    fn new(args: &Bound<'_, PyTuple>, kwargs: Option<&Bound<'_, PyDict>>) -> PyResult<PySound> {
        laeuft()?;

        let n_kw = kwargs.map_or(0, |k| k.len());
        // **Ohne Argumente kommt ein leerer Klang heraus.**
        //
        // Und das ist nicht die halbe Sache, die hier frueher stand.
        // pygame trennt zwei Faelle: `Sound()` laeuft durch `tp_init` und
        // gibt einen `TypeError`, eine Unterklasse mit eigenem, leerem
        // `__init__` laeuft **nur** durch `tp_new` und bekommt einen
        // leeren Klang, dessen Methoden spaeter `RuntimeError` melden.
        // PyO3 0.26 hat keinen `tp_init`-Schlitz und sieht beide Aufrufe
        // gleich - hier stand deshalb lange eine Abwaegung, welcher der
        // beiden Tests wichtiger sei.
        //
        // Die Abwaegung war ueberfluessig: eine duenne Unterklasse **in
        // Python** hat den Schlitz von selbst, und dort steht jetzt die
        // Pruefung auf genau ein Argument (siehe `python/sound.py`). Hier
        // bleibt nur der leere Klang, den `Sound.__new__(Sound)` braucht.
        if args.len() + n_kw == 0 {
            return Ok(PySound { inner: None });
        }
        if args.len() + n_kw != 1 {
            return Err(PyTypeError::new_err(
                "Sound takes either 1 positional or 1 keyword argument",
            ));
        }

        // Rohe Werte holen - und zwar mit pygames Wortlaut, wenn es
        // keine sind. `Vec<u8>` liest hier ueber die Puffer-Schnittstelle
        // wie auch aus einer Folge von Zahlen.
        let roh = |o: &Bound<'_, PyAny>| -> PyResult<Vec<u8>> {
            if o.extract::<String>().is_ok() {
                return Err(PyTypeError::new_err(
                    "Unicode object not allowed as buffer object",
                ));
            }
            // **Ein `Sound` ist ein Puffer** - gemessen nimmt pygame
            // `Sound(array=anderer_klang)` an und liest dessen
            // Abtastwerte. Bei pygame liegt das am Puffer-Protokoll, das
            // `Sound` mitbringt; hier wird der eine Fall von Hand
            // bedient, weil das Protokoll `unsafe` braeuchte.
            if let Ok(k) = o.extract::<PyRef<PySound>>() {
                return Ok(k.dran()?.get_raw());
            }
            // **Eine Liste ist kein Puffer** - auch keine leere.
            // `extract::<Vec<u8>>()` nimmt `[1, 2, 3]` und `[]` an;
            // gemessen gibt `Sound(buffer=[])` dagegen
            // `TypeError: Expected object with buffer interface: got a list`.
            // Dieselbe Verengung wie in `quelle_aus`, nur mit dem anderen
            // der beiden Saetze.
            let bytesfolge = o.is_instance_of::<pyo3::types::PyBytes>()
                || o.is_instance_of::<pyo3::types::PyByteArray>()
                || o.get_type().name().map(|n| n == "memoryview").unwrap_or(false);
            if !bytesfolge {
                let typ = o.get_type().name().map_or_else(
                    |_| "object".to_string(),
                    |n| n.to_string(),
                );
                return Err(PyTypeError::new_err(format!(
                    "Expected object with buffer interface: got a {typ}"
                )));
            }
            o.extract::<Vec<u8>>().map_err(|_| {
                let typ = o.get_type().name().map_or_else(
                    |_| "object".to_string(),
                    |n| n.to_string(),
                );
                PyTypeError::new_err(format!(
                    "Expected object with buffer interface: got a {typ}"
                ))
            })
        };

        let q = if let Some(kw) = kwargs.filter(|k| !k.is_empty()) {
            // Genau ein Schluessel - das steht nach der Zaehlung oben fest.
            let (name, wert) = kw.iter().next().expect("genau ein Schluessel");
            match name.extract::<String>()?.as_str() {
                // `file=` meint ausdruecklich einen **Dateinamen**, auch
                // wenn dort Bytes stehen - und zwar nur den: gemessen gibt
                // `Sound(file=rohe_wav_bytes)` einen `FileNotFoundError`,
                // waehrend `Sound(rohe_wav_bytes)` ohne Schluesselwort die
                // Daten liest. `buffer=` und `array=` meinen umgekehrt
                // ausdruecklich rohe Werte, auch wenn dort ein Dateiname
                // stuende.
                //
                // Hier stand `Quelle::Puffer(b) => Quelle::Datei(b)`, also
                // "Bytes sind der Inhalt". Das war die falsche Haelfte der
                // Regel: es geht um den **Namen**.
                "file" => match quelle_aus(&wert) {
                    Ok(Quelle::Puffer(b)) => match String::from_utf8(b) {
                        Ok(pfad) => Quelle::Pfad(pfad, anzeige(&wert)),
                        Err(_) => {
                            return Err(PyTypeError::new_err("not a file object"))
                        }
                    },
                    Ok(andere) => andere,
                    Err(_) => return Err(PyTypeError::new_err("not a file object")),
                },
                // `buffer=` sind Bytes, `array=` ist ein Feld von
                // Zahlen - zwei verschiedene Dinge. Ein `Sound` und rohe
                // Bytes gehen beide Wege, weil sie beides sein koennen.
                "buffer" => Quelle::Puffer(roh(&wert)?),
                "array" => return Ok(PySound::neu(aus_feld(&wert)?)),
                anderer => {
                    return Err(PyTypeError::new_err(format!(
                        "Unrecognized keyword argument '{anderer}'"
                    )))
                }
            }
        } else {
            quelle_aus(&args.get_item(0)?)?
        };

        Ok(PySound::neu(klang_aus(q)?))
    }

    /// Laedt **in** das vorhandene Objekt - der Rumpf von `__init__`.
    ///
    /// Mit Unterstrich, weil `__init__` selbst in Python steht
    /// (`python/sound.py`): dort haengt die Pruefung auf genau ein
    /// Argument, und von dort wird diese Methode gerufen. Gebraucht fuer
    /// `super().__init__(file=...)` aus einer Unterklasse -
    /// `mixer_test.test_correct_subclassing` macht genau das.
    #[pyo3(signature = (*args, **kwargs))]
    fn _init(
        &mut self,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<()> {
        *self = PySound::new(args, kwargs)?;
        Ok(())
    }

    /// Steht schon ein Klang drin?
    ///
    /// Braucht das `__init__` in `python/sound.py`: beim gewoehnlichen
    /// `Sound(datei)` hat `__new__` die Datei **schon** gelesen, und ein
    /// zweites Lesen waere verschenkte Arbeit. Nur wo der Platz leer ist
    /// - `Sound.__new__(Sound)` und danach ein `__init__` von Hand -
    /// wird geladen.
    fn _geladen(&self) -> bool {
        self.inner.is_some()
    }


    /// `Sound.play(loops=0, maxtime=0, fade_ms=0) -> Channel`.
    ///
    /// `loops` zaehlt die **Wiederholungen**: `loops=1` spielt zweimal.
    /// (`music.play(loops=1)` spielt dagegen einmal - die beiden zaehlen
    /// verschieden, gemessen.)
    ///
    /// Gemessen gibt pygame `None` zurueck, wenn alle Kanaele belegt
    /// sind - keinen Fehler.
    #[pyo3(signature = (loops=0, maxtime=0, fade_ms=0))]
    fn play(&self, loops: i32, maxtime: i32, fade_ms: i32) -> PyResult<Option<PyChannel>> {
        laeuft()?;
        Ok(self
            .dran()?
            .play(loops, maxtime, fade_ms)
            .map(PyChannel::neu))
    }

    /// `Sound.stop()`.
    fn stop(&self) -> PyResult<()> {
        laeuft()?;
        self.dran()?.stop();
        Ok(())
    }

    /// `Sound.fadeout(time)`.
    fn fadeout(&self, time: &Bound<'_, PyAny>) -> PyResult<()> {
        // pygames Satz ohne Vorsatz - PyO3 haengte `argument 'time': `
        // davor.
        let time: i32 = time.extract().map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err("'str' object cannot be interpreted as an integer")
        })?;
        laeuft()?;
        self.dran()?.fadeout(time);
        Ok(())
    }

    /// `Sound.set_volume(value)`.
    ///
    /// Gemessen rastet der Wert auf 128 Stufen ein (`0.1` wird
    /// `0.09375`), ueber `1.0` wird gekappt, und ein **negativer** Wert
    /// aendert gar nichts.
    #[pyo3(signature = (value, /))]
    fn set_volume(&self, value: f64) -> PyResult<()> {
        laeuft()?;
        self.dran()?.set_volume(value);
        Ok(())
    }

    /// `Sound.get_volume()`.
    fn get_volume(&self) -> PyResult<f64> {
        laeuft()?;
        Ok(self.dran()?.get_volume())
    }

    /// `Sound.get_length()` - die Laenge in Sekunden.
    ///
    /// Das ist die Laenge **im Mixerformat**, nicht die der Datei: eine
    /// Mono-Datei mit 22050 Hz wird beim Laden hochgerechnet.
    fn get_length(&self) -> PyResult<f64> {
        laeuft()?;
        Ok(self.dran()?.get_length())
    }

    /// `Sound.get_num_channels()` - auf wie vielen Kanaelen der Klang
    /// gerade laeuft.
    fn get_num_channels(&self) -> PyResult<usize> {
        laeuft()?;
        Ok(self.dran()?.get_num_channels())
    }

    /// `Sound.get_raw()` - die Werte als `bytes`.
    ///
    /// Gemessen: 16 Bit, little endian, verschraenkt, in Frequenz und
    /// Spurenzahl des **Mixers**.
    fn get_raw<'p>(&self, py: Python<'p>) -> PyResult<Bound<'p, PyBytes>> {
        laeuft()?;
        Ok(PyBytes::new(py, &self.dran()?.get_raw()))
    }

    /// `Sound._samples_address` - die Stelle der Werte im Speicher.
    ///
    /// Eine blosse Zahl, wie in pygame. `mixer_test.test_samples_address`
    /// prueft zweierlei: dass sie **nicht** mit der Adresse des
    /// uebergebenen `bytes`-Objekts uebereinstimmt (der Klang hat seinen
    /// eigenen Puffer), und dass der Zugriff nach `mixer.quit()`
    /// `pygame.error: mixer not initialized` gibt.
    ///
    /// Der Kern konnte das laengst (`Sound::zeiger`), nur reichte die
    /// Huelle es nicht durch - der Fall "nur im Kern" aus der
    /// Abdeckungstabelle.
    #[getter]
    fn _samples_address(&self) -> PyResult<usize> {
        laeuft()?;
        Ok(self.dran()?.zeiger())
    }

    /// `Sound.__array_interface__` - die numpy-Schnittstelle.
    ///
    /// Gemessen an echtem pygame mit `Sound(buffer=b"abcdefgh")` unter der
    /// Vorgabe `-16`:
    ///
    /// ```python
    /// {'data': (adresse, False), 'shape': (2, 2), 'strides': (4, 2),
    ///  'typestr': '<i2', 'version': 3}
    /// ```
    ///
    /// `shape` ist `(Rahmen, Spuren)`, `strides` sind die Byte-Abstaende
    /// dazu, und `False` heisst "beschreibbar". Ein **Mono**-Klang hat in
    /// pygame nur eine Achse - auch das gemessen.
    ///
    /// ## Das Format ist das des Mixers
    ///
    /// Hier stand fest `<i2`, mit der Begruendung, gemischt werde immer
    /// 16-bittig. Das stimmte fuer die Rechnung und nicht fuer die
    /// **Ablage**: seit `Klang` die Bytes im Ausgabeformat haelt, liegt
    /// unter `init(size=8)` wirklich ein Byte je Wert dort. Gemessen gibt
    /// pygame dann `|u1` mit `shape (4, 2)` und `strides (2, 1)`, und
    /// genau das steht jetzt auch hier.
    ///
    /// Die fuenf Formate, die `get_init()[1]` kennen kann:
    ///
    /// | Groesse | `typestr` | Bytes |
    /// |---|---|---|
    /// | `8` | `\|u1` | 1 |
    /// | `-8` | `\|i1` | 1 |
    /// | `16` | `<u2` | 2 |
    /// | `-16` | `<i2` | 2 |
    /// | `-32` | `<f4` | 4 |
    #[getter]
    fn __array_interface__(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        laeuft()?;
        let s = self.dran()?;
        let d = PyDict::new(py);
        let (typestr, breite) = klangformat(s.groesse());
        d.set_item("version", 3)?;
        d.set_item("typestr", typestr)?;
        let spuren = s.spuren() as usize;
        let rahmen = s.rahmen();
        if spuren > 1 {
            d.set_item("shape", (rahmen, spuren))?;
            d.set_item("strides", (breite * spuren, breite))?;
        } else {
            d.set_item("shape", (rahmen,))?;
            d.set_item("strides", (breite,))?;
        }
        d.set_item("data", (s.zeiger(), false))?;
        Ok(d.unbind())
    }

}

impl PySound {
    /// Die Beschreibung der Abtastwerte fuer das Puffer-Protokoll.
    ///
    /// Gemessen an `mixer_test._NEWBUF_export_check` unter
    /// `init(22050, -16, n)`:
    ///
    /// | Spuren | `ndim` | `shape` | `strides` | `format` |
    /// |---|---|---|---|---|
    /// | 1 | 1 | `(rahmen,)` | `(2,)` | `=h` |
    /// | 2 | 2 | `(rahmen, 2)` | `(4, 2)` | `=h` |
    ///
    /// Die letzte Zeile des Tests ist die aufschlussreiche: bei zwei Spuren
    /// muss `PyBUF_F_CONTIGUOUS` **scheitern**, bei einer gehen. Das fällt
    /// von selbst richtig heraus, weil ein Feld mit einer Achse beides ist
    /// und `(4, 2)` nur C-zusammenhaengend.
    ///
    /// Hier ist die Beschreibung **ehrlich**: rustgame mischt immer
    /// 16-bittig mit Vorzeichen, und genau das steht im Speicher. Unter
    /// einem Mixer mit anderem Ausgabeformat weicht sie von pygames ab -
    /// dieselbe benannte Ausnahme wie bei `__array_interface__`, und aus
    /// demselben Grund.
    fn puffersicht(&self) -> PyResult<crate::puffer::Sicht> {
        laeuft()?;
        let s = self.dran()?;
        let (typestr, breite) = klangformat(s.groesse());
        let _ = typestr;
        let format = klangpuffer(s.groesse());
        let spuren = s.spuren() as isize;
        let rahmen = s.rahmen() as isize;
        let b = breite as isize;
        let (shape, strides) = if spuren > 1 {
            (vec![rahmen, spuren], vec![b * spuren, b])
        } else {
            (vec![rahmen], vec![b])
        };
        Ok(crate::puffer::Sicht {
            adresse: s.zeiger(),
            laenge: (rahmen * spuren) as usize * breite,
            itemsize: breite,
            format: crate::puffer::fmt(format),
            shape,
            strides,
            schreibbar: true,
            solide: true,
        })
    }
}

#[pymethods]
impl PySound {
    /// `memoryview(sound)`, `Sound(array=anderer_klang)`, numpy.
    ///
    /// # Sicherheit
    ///
    /// Die Adresse ist der Anfang von `Klang::werte`. Der `Vec` wird nach
    /// dem Anlegen nicht mehr veraendert und vom `Rc` des Klangs
    /// festgehalten; `fuellen` haelt zusaetzlich das Python-Objekt fest.
    // Reicht nur durch - siehe `#![deny(unsafe_code)]` in `lib.rs`.
    #[allow(unsafe_code)]
    unsafe fn __getbuffer__(
        slf: PyRef<'_, Self>,
        view: *mut pyo3::ffi::Py_buffer,
        flags: std::ffi::c_int,
    ) -> PyResult<()> {
        let sicht = slf.puffersicht()?;
        let py = slf.py();
        let eigner = slf.into_pyobject(py)?;
        unsafe { crate::puffer::fuellen(&sicht, view, flags, eigner.as_any()) }
    }

    /// # Sicherheit
    ///
    /// Siehe [`crate::puffer::freigeben`].
    #[allow(unsafe_code)]
    unsafe fn __releasebuffer__(&self, view: *mut pyo3::ffi::Py_buffer) {
        unsafe { crate::puffer::freigeben(view) }
    }

    /// `Sound.__array_struct__` - dieselbe Beschreibung als C-Kasten.
    #[getter]
    fn __array_struct__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        crate::puffer::feldschnittstelle(py, &self.puffersicht()?)
    }

    /// `Sound.copy()` - ein eigenstaendiger zweiter Klang.
    ///
    /// Die Lautstaerke wird mitgenommen: gemessen ist
    /// `s.set_volume(0.3); s.copy().get_volume()` genau `0.296875`.
    /// Eigenstaendig heisst dabei wirklich eigenstaendig: `set_volume` auf
    /// der Kopie laesst das Original unberuehrt.
    /// Es ruft `__copy__`, damit eine **Unterklasse** ihren Typ behaelt -
    /// `mixer_test.test_snd_copy` prueft
    /// `assertIsInstance(sound.copy(), SubSound)`. Dieselbe Richtung wie
    /// bei `Mask.copy()`, wo `mask.c` es im Kommentar ausdruecklich sagt.
    fn copy(slf: &Bound<'_, PySound>) -> PyResult<Py<PyAny>> {
        laeuft()?;
        slf.call_method0("__copy__").map(|o| o.unbind())
    }

    /// `copy.copy(sound)` - und es **behaelt die Unterklasse**.
    ///
    /// `mixer_test.test_snd_copy` prueft beides: `sound.copy()` und
    /// `copy.copy(sound)`. Ohne `__copy__` faellt das Modul `copy` auf die
    /// Pickle-Schnittstelle zurueck und gibt
    /// `TypeError: cannot pickle 'rustgame_py.mixer.Sound' object` - eine
    /// Meldung, die von Pickle erzaehlt und nicht davon, dass eine Methode
    /// fehlt.
    ///
    /// Gebaut wird ueber `type(self).__new__(type(self), ...)`, damit eine
    /// Unterklasse ihren Typ behaelt - dieselbe Ueberlegung wie bei
    /// `Mask.__copy__`.
    fn __copy__(slf: &Bound<'_, PySound>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        // **Ueber die Rohdaten, nicht ueber ein leeres `__new__`.**
        //
        // `__new__` verlangt genau ein Argument (siehe dort, samt der
        // Rechnung, warum es dabei bleibt), also wird ihm der Klang als
        // Bytespuffer gegeben - daraus baut es einen echten `Sound`, und
        // zwar in der **Unterklasse**, weil `typ` deren Typ ist.
        //
        // Mein erster Versuch rief `__new__(typ)` ohne Argument und lief
        // damit in genau die Grenze, die zwei Bildschirme weiter oben
        // beschrieben steht.
        let (roh, lautstaerke) = {
            let b = slf.borrow();
            let k = b.dran()?;
            (k.get_raw(), k.get_volume())
        };
        let typ = slf.get_type();
        let neu = typ.call_method1(
            "__new__",
            (&typ, pyo3::types::PyBytes::new(py, &roh)),
        )?;
        {
            let z: PyRef<PySound> = neu.extract()?;
            z.dran()?.set_volume(lautstaerke);
        }
        Ok(neu.unbind())
    }

    // **Kein `__deepcopy__`.** Gemessen hat pygames `Sound` keines;
    // `copy.deepcopy(klang)` geht dort ueber `__reduce__` und `__copy__`.
    // Hier stand eines, und das war ein Doppelstrich zuviel - gefunden von
    // der Probe `Klassendoppel`.
}

/// `pygame.mixer.Channel` - eine Tonspur.
///
/// Acht Spuren legt `mixer.init()` an; `Channel(n)` holt die n-te. Die
/// Arbeit steckt in [`rustgame_mixer`]; hier ist die Huelle.
///
/// Ableitbar, weil `mixer_test.py` es tut, und ohne `weakref` - beides
/// gemessen am eingebauten Modul.
#[pyclass(name = "Channel", module = "rustgame_py.mixer", subclass)]
pub struct PyChannel {
    /// Die Spur im Kern.
    inner: Channel,
}

thread_local! {
    /// Je Kanalnummer das **Python-Objekt** des laufenden Klangs.
    ///
    /// `Channel.get_sound()` gibt in pygame **dasselbe Objekt** zurueck,
    /// das gespielt wurde - `c.play(s); c.get_sound() is s` ist dort
    /// `True`. Das stand hier als benannte Grenze, mit der Begruendung, um
    /// den Kernklang entstehe bei jedem Abruf ein frisches
    /// Python-Objekt. Das stimmte und war zu schnell aufgegeben: die
    /// Nummer des Kanals ist ein guter Schluessel, und das Merken kostet
    /// einen Zeiger.
    ///
    /// Sichtbar wird der Unterschied nicht nur bei `is`, sondern auch bei
    /// **Unterklassen**: `mixer_test.test_correct_subclassing` spielt eine
    /// abgeleitete `Sound` und prueft
    /// `assertIsInstance(channel.get_sound(), CorrectSublass)`. Ein frisch
    /// gebautes `PySound` ist immer die Grundklasse.
    ///
    /// Aufgeraeumt wird beim naechsten `play` derselben Nummer und bei
    /// `stop`. Bleibt doch ein Eintrag stehen, ist er harmlos: gefragt
    /// wird nur, solange der Kern denselben Klang meldet, und das prueft
    /// [`PyChannel::gemerkter_klang`] an der Kennung nach.
    static KANAL_KLANG: RefCell<HashMap<usize, Py<PySound>>> =
        RefCell::new(HashMap::new());
    /// Dasselbe fuer die Warteschlange.
    static KANAL_SCHLANGE: RefCell<HashMap<usize, Py<PySound>>> =
        RefCell::new(HashMap::new());
}

impl PyChannel {
    pub(crate) fn neu(inner: Channel) -> PyChannel {
        PyChannel { inner }
    }

    /// Das gemerkte Python-Objekt, **wenn** es noch dasselbe ist.
    ///
    /// Verglichen wird ueber `Sound::kennung()` - die Stelle der
    /// gemeinsamen Daten im Speicher. Passt sie nicht, hat der Kanal
    /// inzwischen etwas anderes, und dann ist ein frisches Objekt die
    /// richtige Antwort.
    fn gemerkter_klang(
        &self,
        py: Python<'_>,
        ablage: &'static std::thread::LocalKey<RefCell<HashMap<usize, Py<PySound>>>>,
        kern: Sound,
    ) -> PyResult<Py<PySound>> {
        let gefunden = ablage.with(|a| {
            a.borrow().get(&self.inner.id()).and_then(|p| {
                let gleich = p
                    .bind(py)
                    .borrow()
                    .inner
                    .as_ref()
                    .map(|s| s.kennung() == kern.kennung())
                    .unwrap_or(false);
                gleich.then(|| p.clone_ref(py))
            })
        });
        // **Das Objekt selbst, nicht eine Kopie seines Inhalts.**
        //
        // Ein `Py<PySound>` zurueckzugeben ist der ganze Punkt: eine Kopie
        // des Rust-Werts wuerde in Python ein **neues** Objekt der
        // Grundklasse, und damit waere weder `is` noch
        // `isinstance(.., Unterklasse)` erfuellt. Genau darauf war ich beim
        // ersten Versuch hereingefallen.
        match gefunden {
            Some(p) => Ok(p),
            None => Py::new(py, PySound::neu(kern)),
        }
    }

    /// Merkt sich das Python-Objekt zu diesem Kanal.
    fn merken(
        &self,
        py: Python<'_>,
        ablage: &'static std::thread::LocalKey<RefCell<HashMap<usize, Py<PySound>>>>,
        sound: &Bound<'_, PySound>,
    ) {
        let _ = py;
        ablage.with(|a| {
            a.borrow_mut()
                .insert(self.inner.id(), sound.clone().unbind())
        });
    }
}

#[pymethods]
impl PyChannel {
    /// `Channel(id)`.
    ///
    /// Gemessen: `Channel(8)` bei acht Kanaelen und `Channel(-1)` werfen
    /// beide `IndexError: invalid channel index`.
    ///
    /// Der Beiwert ist **`i64`, nicht `usize`** - sonst machte PyO3 aus
    /// `Channel(-1)` einen `OverflowError` statt des gemessenen
    /// `IndexError`. Dieselbe Falle steht bei `umwandlung::flagge`
    /// beschrieben, dort fuer `FULLSCREEN`.
    #[new]
    fn new(id: i64) -> PyResult<PyChannel> {
        laeuft()?;
        if id < 0 {
            return Err(PyIndexError::new_err("invalid channel index"));
        }
        Ok(PyChannel::neu(
            Channel::new(id as usize).map_err(ton_fehler)?,
        ))
    }

    /// `Channel.id` - der Index. Ein Attribut, keine Methode.
    #[getter]
    fn id(&self) -> PyResult<usize> {
        laeuft()?;
        Ok(self.inner.id())
    }

    /// `Channel.play(Sound, loops=0, maxtime=0, fade_ms=0)`.
    ///
    /// Gibt **nichts** zurueck - anders als `Sound.play()`, das den
    /// benutzten Kanal liefert.
    ///
    /// Und anders als `Sound.play()` laesst es die Lautstaerke des Kanals
    /// stehen: gemessen ist nach `c.set_volume(0.25); c.play(s)` weiter
    /// `0.25` eingestellt, waehrend `s.play()` den Kanal auf `1.0`
    /// zuruecksetzt.
    #[pyo3(signature = (sound, loops=0, maxtime=0, fade_ms=0))]
    fn play(
        &self,
        py: Python<'_>,
        sound: &Bound<'_, PySound>,
        loops: i32,
        maxtime: i32,
        fade_ms: i32,
    ) -> PyResult<()> {
        laeuft()?;
        {
            let s = sound.borrow();
            self.inner
                .play(s.dran()?, loops, maxtime, fade_ms)
                .map_err(ton_fehler)?;
        }
        self.merken(py, &KANAL_KLANG, sound);
        Ok(())
    }

    /// `Channel.queue(sound)`.
    ///
    /// Laeuft gerade nichts, faengt der Klang sofort an - gemessen ist
    /// `get_queue()` danach `None` und `get_sound()` der Klang selbst.
    /// Es passt immer nur **einer** in die Schlange.
    #[pyo3(signature = (sound, /))]
    fn queue(&self, py: Python<'_>, sound: &Bound<'_, PyAny>) -> PyResult<()> {
        laeuft()?;
        // pygames eigener Satz; PyO3 sagte `argument 'sound': 'str'
        // object cannot be converted to 'Sound'` und nannte damit den
        // Parameternamen und einen Rust-Typnamen.
        let sound: &Bound<'_, PySound> = &sound.extract::<Py<PySound>>()
            .map_err(|_| {
                pyo3::exceptions::PyTypeError::new_err("The argument must be an instance of Sound")
            })?
            .into_bound(py);
        {
            let s = sound.borrow();
            self.inner.queue(s.dran()?).map_err(ton_fehler)?;
        }
        // Ein `queue` auf einem stillen Kanal faengt sofort an - dann ist
        // es der laufende Klang und nicht die Schlange. Gemessen; deshalb
        // wird beides gemerkt und beim Abruf die Kennung verglichen.
        self.merken(py, &KANAL_SCHLANGE, sound);
        self.merken(py, &KANAL_KLANG, sound);
        Ok(())
    }

    /// `Channel.get_queue()`.
    fn get_queue(&self, py: Python<'_>) -> PyResult<Option<Py<PySound>>> {
        laeuft()?;
        match self.inner.get_queue() {
            None => Ok(None),
            Some(k) => Ok(Some(self.gemerkter_klang(py, &KANAL_SCHLANGE, k)?)),
        }
    }

    /// `Channel.get_sound()`.
    ///
    /// Gemessen: nach dem Ende `None`, nicht der letzte Klang.
    fn get_sound(&self, py: Python<'_>) -> PyResult<Option<Py<PySound>>> {
        laeuft()?;
        match self.inner.get_sound() {
            None => Ok(None),
            Some(k) => Ok(Some(self.gemerkter_klang(py, &KANAL_KLANG, k)?)),
        }
    }

    /// `Channel.get_busy()`.
    ///
    /// Gemessen und ueberraschend: ein **pausierter** Kanal gilt weiter
    /// als belegt. Bei der Musik ist es genau umgekehrt, siehe
    /// [`musik_get_busy`].
    fn get_busy(&self) -> PyResult<bool> {
        laeuft()?;
        Ok(self.inner.get_busy())
    }

    /// `Channel.stop()`.
    ///
    /// Gemessen: steht etwas in der Schlange, faengt **das** dabei an zu
    /// spielen - `get_busy()` ist danach also weiter `True`.
    fn stop(&self) -> PyResult<()> {
        laeuft()?;
        self.inner.stop();
        Ok(())
    }

    /// `Channel.pause()`.
    fn pause(&self) -> PyResult<()> {
        laeuft()?;
        self.inner.pause();
        Ok(())
    }

    /// `Channel.unpause()`.
    fn unpause(&self) -> PyResult<()> {
        laeuft()?;
        self.inner.unpause();
        Ok(())
    }

    /// `Channel.fadeout(time)`.
    fn fadeout(&self, time: &Bound<'_, PyAny>) -> PyResult<()> {
        // pygames Satz ohne Vorsatz - PyO3 haengte `argument 'time': `
        // davor.
        let time: i32 = time.extract().map_err(|_| {
            pyo3::exceptions::PyTypeError::new_err("'str' object cannot be interpreted as an integer")
        })?;
        laeuft()?;
        self.inner.fadeout(time);
        Ok(())
    }

    /// `Channel.set_volume(value)` und `Channel.set_volume(links, rechts)`.
    ///
    /// **Mit zwei Werten stellt das keine Lautstaerken ein, sondern die
    /// Balance** - und setzt die Lautstaerke dabei auf voll. Gemessen:
    ///
    /// ```text
    /// c.set_volume(0.2, 0.8); c.get_volume()  ->  1.0
    /// ```
    ///
    /// Die Balance laesst sich danach nirgends mehr auslesen; in pygames
    /// `chan_set_volume` steht woertlich "NOTE: there is no way to
    /// GetPanning variables".
    #[pyo3(signature = (value, right=None, /))]
    fn set_volume(&self, value: f64, right: Option<f64>) -> PyResult<()> {
        laeuft()?;
        match right {
            Some(r) => self.inner.set_volume_stereo(value, r),
            None => self.inner.set_volume(value),
        }
        Ok(())
    }

    /// `Channel.get_volume()`.
    fn get_volume(&self) -> PyResult<f64> {
        laeuft()?;
        Ok(self.inner.get_volume())
    }

    /// `Channel.set_source_location(angle, distance)`.
    ///
    /// Gemessen entscheidet `distance` **vor** der Init-Pruefung, weil
    /// das `MIXER_INIT_CHECK()` in `chan_set_source_location` hinter der
    /// Bereichspruefung steht:
    ///
    /// ```text
    /// ch.set_source_location(0, 999)  -> ValueError: distance out of range, expected (0, 255)
    /// ch.set_source_location(0, 1)    -> pygame.error: mixer not initialized
    /// ```
    ///
    /// Darum steht hier **kein** [`laeuft`] davor: die Kiste prueft in
    /// genau dieser Reihenfolge und liefert beide Fehler richtig.
    #[pyo3(signature = (angle, distance, /))]
    fn set_source_location(&self, angle: f32, distance: f32) -> PyResult<()> {
        self.inner
            .set_source_location(angle, distance)
            .map_err(ton_fehler)
    }

    /// `Channel.set_endevent()` und `Channel.set_endevent(type)`.
    ///
    /// Ohne Argument wird das Ereignis abgeschaltet - gemessen liefert
    /// `get_endevent()` danach `0`.
    #[pyo3(signature = (type_=0, /))]
    fn set_endevent(&self, type_: u32) -> PyResult<()> {
        laeuft()?;
        self.inner.set_endevent(type_);
        Ok(())
    }

    /// `Channel.get_endevent()`.
    ///
    /// Gemessen: ohne vorheriges Setzen `0`.
    fn get_endevent(&self) -> PyResult<u32> {
        laeuft()?;
        Ok(self.inner.get_endevent())
    }
}

// ---------------------------------------------------------------------------
//  pygame.mixer - die Modulfunktionen
// ---------------------------------------------------------------------------

/// `pygame.mixer.init(frequency=44100, size=-16, channels=2, buffer=512,
/// devicename=None, allowedchanges=...)`.
///
/// Gemessen: gibt nichts zurueck, ein zweites `init()` bei laufendem
/// Mixer aendert nichts und beschwert sich nicht, und `get_init()` steht
/// danach auf `(44100, -16, 2)` bei **8** Kanaelen - nicht 16.
///
/// `devicename` und `allowedchanges` werden angenommen und nicht
/// benutzt: rustgame sucht sich kein Geraet aus und laesst SDL keine
/// Formatwuensche verhandeln. Sie stehen hier, damit ein Programm, das
/// sie uebergibt, nicht an einem `TypeError` scheitert.
#[pyfunction]
///
/// ## Die Vorgaben sind **0**, nicht die Zielwerte
///
/// Hier stand `frequency=44100, size=-16, channels=2, buffer=512`, und
/// damit war `pre_init` wirkungslos: `init()` hat den Wunsch jedes Mal
/// mit seinen eigenen Vorgaben ueberschrieben. Gemessen gibt
/// `quit(); pre_init(size=32); init(); get_init()` in pygame
/// `(44100, -32, 2)` und gab hier `(44100, -16, 2)`.
///
/// Eine `0` heisst "nicht angegeben" und laesst die Kette
/// `init()`-Argument -> `pre_init`-Wunsch -> Vorgabe durchlaufen; die
/// Vorgabe selbst steht im Kern (`auswerten`). pygames C-Fassung macht es
/// genauso, aus demselben Grund.
///
/// Der Fehler war unsichtbar, solange niemand `pre_init` **und** `init`
/// zusammen benutzt - und genau das tut `mixer_test.py`.
#[pyo3(signature = (frequency=0, size=0, channels=0, buffer=0,
                    devicename=None, allowedchanges=0))]
#[pyo3(name = "init")]
pub fn ton_init(
    frequency: i32,
    size: i32,
    channels: i32,
    buffer: i32,
    devicename: Option<String>,
    allowedchanges: i32,
) -> PyResult<()> {
    let _ = (devicename, allowedchanges);
    treiber_waehlen();
    // **Auch `mixer.init()` allein meldet die Geraete.** Gemessen, siehe
    // `crate::tongeraete_melden`.
    crate::tongeraete_melden();
    match mixer::init(Einstellungen::neu(frequency, size, channels, buffer)) {
        Ok(()) => Ok(()),
        // Kein Tongeraet ist kein Grund zum Abbruch: die Kiste faehrt
        // dann still hoch, und alles andere - Laengen, Kanaele,
        // `get_busy()` - rechnet unveraendert weiter. Ein Spiel auf einem
        // Rechner ohne Soundkarte soll laufen, nicht abstuerzen.
        Err(Fehler::KeinGeraet(_)) => Ok(()),
        Err(e) => Err(ton_fehler(e)),
    }
}

/// `pygame.mixer.pre_init(...)` - was ein spaeteres `init()` nehmen soll.
///
/// Gemessen: aendert am laufenden Mixer nichts, und ein Feld auf `0`
/// heisst "zurueck auf die Vorgabe", nicht "unveraendert lassen" - nach
/// `pre_init(0, 0, 0, 0)` liefert `init()` wieder `(44100, -16, 2)`.
#[pyfunction]
#[pyo3(signature = (frequency=44100, size=-16, channels=2, buffer=512,
                    devicename=None, allowedchanges=0))]
#[pyo3(name = "pre_init")]
pub fn ton_pre_init(
    frequency: i32,
    size: i32,
    channels: i32,
    buffer: i32,
    devicename: Option<String>,
    allowedchanges: i32,
) {
    let _ = (devicename, allowedchanges);
    mixer::pre_init(Einstellungen::neu(frequency, size, channels, buffer));
}

/// `pygame.mixer.quit()`.
///
/// Gemessen: geht auch, wenn gar nichts lief, und beschwert sich nicht.
#[pyfunction]
#[pyo3(name = "quit")]
pub fn ton_quit() {
    mixer::quit();
}

/// `pygame.mixer.get_init() -> (frequency, format, channels)`.
///
/// Gemessen `None` vor `init()` und `(44100, -16, 2)` danach. Die
/// Puffergroesse verraet pygame hier **nicht**.
#[pyfunction]
#[pyo3(name = "get_init")]
pub fn ton_get_init() -> Option<(i32, i32, i32)> {
    mixer::get_init()
}

/// `pygame.mixer.get_driver() -> str`.
#[pyfunction]
#[pyo3(name = "get_driver")]
pub fn ton_get_driver() -> PyResult<String> {
    mixer::get_driver().map_err(ton_fehler)
}

/// `pygame.mixer.get_sdl_mixer_version() -> (major, minor, patch)`.
///
/// **Hier kommt `None`, und das ist Absicht.** Echtes pygame meldet
/// `(2, 8, 1)`, mit `linked=False` dasselbe - gemessen. rustgame bindet
/// aber **kein SDL_mixer** ein: der Ton kommt aus `rodio`, der
/// WAV-Dekoder ist eigener Code, und die Mischregeln sind aus
/// `src_c/mixer.c` uebertragen, nicht verlinkt.
///
/// Eine Fassungsnummer fuer eine Bibliothek zu melden, die gar nicht da
/// ist, waere die bequeme Antwort und die schlechtere: ein Programm, das
/// anhand der Zahl eine Weiche stellt ("ab 2.6 kann ich
/// `get_metadata`"), liefe in den falschen Zweig. `font.get_sdl_ttf_version`
/// und `image.get_sdl_image_version` halten es im selben Projekt genauso,
/// und der Modulvergleich fuehrt es als **benannte Grenze**.
///
/// Die Kiste meldete hier lange die `(2, 8, 1)` weiter; das ist mit
/// dieser Aenderung berichtigt worden.
#[pyfunction]
#[pyo3(signature = (linked=None))]
#[pyo3(name = "get_sdl_mixer_version")]
pub fn ton_get_sdl_mixer_version(linked: Option<&Bound<'_, PyAny>>) -> PyResult<Option<(i32, i32, i32)>> {
    // **Jeder Wert ist ein Wahrheitswert** - pygame liest den
    // Schalter mit `PyObject_IsTrue`, und das nimmt alles. Mit
    // `bool` verlangte PyO3 einen echten Wahrheitswert und nannte
    // im Fehlertext `'PyBool'`, einen Namen aus Rust.
    let linked = match linked {
        None => true,
        Some(o) => o.is_truthy()?,
    };
    let _ = linked;
    Ok(mixer::get_sdl_mixer_version())
}

/// `pygame.mixer.get_soundfont() -> paths`.
///
/// Gemessen: anfangs `None` - und **vor `init()` ein
/// `mixer not initialized`**, nicht etwa `None`.
#[pyfunction]
#[pyo3(name = "get_soundfont")]
pub fn ton_get_soundfont() -> PyResult<Option<String>> {
    laeuft()?;
    Ok(mixer::get_soundfont())
}

/// `pygame.mixer.set_soundfont(path)`.
///
/// **Wirkt nichts**, und das steht auch so da: rustgame spielt kein MIDI.
/// Der Pfad wird gemerkt und von [`ton_get_soundfont`] wieder
/// herausgegeben - pygame tut genau das, ohne die Datei zu pruefen
/// (gemessen: ein Pfad, den es nicht gibt, wird anstandslos uebernommen).
///
/// ## Der leere Text loescht, er speichert nicht
///
/// Gemessen: `set_soundfont("")` gibt danach `get_soundfont() == None`,
/// **nicht** `""`. Das ist mehr als eine Kleinigkeit - `mixer_test.py`
/// endet mit genau diesem Aufruf, und `mixer_music_test.py` fragt
/// danach `if get_soundfont() is not None` und laedt dann eine
/// MIDI-Datei. Ein `""` ist nicht `None`, also liefen dort zwei Tests
/// los, die pygame ueberspringt - und scheiterten an einem Format, das
/// rustgame nicht liest.
///
/// Ein Fehler, drei rote Tests, und zwei davon in einer **anderen**
/// Datei. Genau deshalb wird hier zwischen "nichts angegeben",
/// "ausdruecklich `None`" und "leerer Text" nicht unterschieden: alle
/// drei loeschen.
///
/// Und der Typ wird geprueft: `set_soundfont(0)` und
/// `set_soundfont(["a","b"])` geben in pygame `TypeError`. Mit
/// `Option<String>` faellt das zwar auch auf, aber mit PyO3s Wortlaut;
/// hier steht der gemessene.
#[pyfunction]
#[pyo3(signature = (paths=None, /))]
#[pyo3(name = "set_soundfont")]
pub fn ton_set_soundfont(paths: Option<&Bound<'_, PyAny>>) -> PyResult<()> {
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let path = paths;
    laeuft()?;
    let wert = match path {
        None => None,
        Some(o) if o.is_none() => None,
        Some(o) => match o.extract::<String>() {
            Ok(t) if t.is_empty() => None,
            Ok(t) => Some(t),
            Err(_) => {
                let typ = o.get_type().name()?;
                return Err(PyTypeError::new_err(format!(
                    "path must be a string or None, not {typ}"
                )));
            }
        },
    };
    mixer::set_soundfont(wert.as_deref());
    Ok(())
}

/// `pygame.mixer.get_num_channels() -> count`.
///
/// Gemessen: nach `init()` sind es **8**.
#[pyfunction]
#[pyo3(name = "get_num_channels")]
pub fn ton_get_num_channels() -> PyResult<usize> {
    mixer::get_num_channels().map_err(ton_fehler)
}

/// `pygame.mixer.set_num_channels(count)`.
///
/// Gemessen: ein negativer Wert setzt auf `0` statt zu scheitern.
#[pyfunction]
#[pyo3(name = "set_num_channels")]
#[pyo3(signature = (count, /))]
pub fn ton_set_num_channels(count: i64) -> PyResult<()> {
    mixer::set_num_channels(count.max(0) as usize).map_err(ton_fehler)
}

/// `pygame.mixer.set_reserved(count) -> count`.
///
/// Gibt zurueck, wie viele wirklich reserviert wurden. Gemessen wird
/// geklemmt statt abgelehnt: `set_reserved(-1)` gibt `0`,
/// `set_reserved(100)` bei acht Kanaelen gibt `8`.
#[pyfunction]
#[pyo3(name = "set_reserved")]
#[pyo3(signature = (count, /))]
pub fn ton_set_reserved(count: i64) -> PyResult<usize> {
    mixer::set_reserved(count.max(0) as usize).map_err(ton_fehler)
}

/// `pygame.mixer.find_channel(force=False) -> Channel`.
///
/// Gemessen: `None`, wenn keiner frei ist und `force` falsch bleibt. Mit
/// `force=True` kommt immer einer - **ausser** es gibt gar keine
/// Kanaele, dann `IndexError: invalid channel index`
/// (nach `set_num_channels(0)` gemessen).
#[pyfunction]
#[pyo3(signature = (force=None))]
#[pyo3(name = "find_channel")]
pub fn ton_find_channel(force: Option<&Bound<'_, PyAny>>) -> PyResult<Option<PyChannel>> {
    // Wie bei `mouse.set_visible`: pygame liest hier eine **Zahl**, nicht
    // einen Wahrheitswert - `find_channel("x")` gibt `'str' object cannot
    // be interpreted as an integer`. Mit `force: bool` sagte PyO3
    // `argument 'force': 'str' object cannot be converted to 'PyBool'`.
    let force = match force {
        None => false,
        Some(o) => o.extract::<i64>()? != 0,
    };
    Ok(mixer::find_channel(force)
        .map_err(ton_fehler)?
        .map(PyChannel::neu))
}

/// `pygame.mixer.get_busy() -> bool`.
///
/// Die eine Abfrage, die **ohne** laufenden Mixer keinen Fehler wirft -
/// gemessen gibt sie dort schlicht `False`.
#[pyfunction]
#[pyo3(name = "get_busy")]
pub fn ton_get_busy() -> bool {
    mixer::get_busy()
}

/// `pygame.mixer.stop()` - haelt alle Kanaele an.
///
/// Die Musikspur bleibt davon unberuehrt; dafuer gibt es
/// [`musik_stop`].
#[pyfunction]
#[pyo3(name = "stop")]
pub fn ton_stop() -> PyResult<()> {
    mixer::stop().map_err(ton_fehler)
}

/// `pygame.mixer.pause()`.
#[pyfunction]
#[pyo3(name = "pause")]
pub fn ton_pause() -> PyResult<()> {
    mixer::pause().map_err(ton_fehler)
}

/// `pygame.mixer.unpause()`.
#[pyfunction]
#[pyo3(name = "unpause")]
pub fn ton_unpause() -> PyResult<()> {
    mixer::unpause().map_err(ton_fehler)
}

/// `pygame.mixer.fadeout(time)`.
#[pyfunction]
#[pyo3(name = "fadeout")]
#[pyo3(signature = (time, /))]
pub fn ton_fadeout(time: i32) -> PyResult<()> {
    mixer::fadeout(time).map_err(ton_fehler)
}

// ---------------------------------------------------------------------------
//  pygame.mixer_music - die Musikspur
// ---------------------------------------------------------------------------

/// Was `music.load` und `music.queue` als Stueck annehmen.
///
/// Anders als bei [`PySound`] gibt es hier **keinen** Puffer-Fall:
/// gemessen wirft `music.load(sound_objekt)` ein
/// `TypeError: not a file object`. Ein blankes `bytes` ist fuer die Musik
/// also ein Dateiinhalt, kein Klang.
fn musik_quelle(o: &Bound<'_, PyAny>) -> PyResult<Quelle> {
    // **`bytes` ist hier nur ein Pfad**, `bytearray` und `memoryview`
    // sind gar nichts. Gemessen in `load`, `queue` und `get_metadata`:
    //
    // ```text
    // pfad.encode()              -> ok
    // rohe WAV-Bytes             -> FileNotFoundError: No file 'b'RIFF..'' ..
    // bytearray(pfad.encode())   -> TypeError: not a file object
    // memoryview(pfad.encode())  -> TypeError: not a file object
    // ```
    //
    // Hier stand `Puffer -> Datei`: rohe Bytes wurden als Dateiinhalt
    // gespielt, und ein fehlender Pfad als `bytes` endete in `Couldn't
    // read first 12 bytes of audio data`. Der Kommentar in `quelle_aus`
    // sagte schon "nur Pfad!" - der Code tat es nicht.
    if o.is_instance_of::<pyo3::types::PyBytes>() {
        let b = o.extract::<Vec<u8>>()?;
        return Ok(Quelle::Pfad(
            String::from_utf8_lossy(&b).into_owned(),
            anzeige(o),
        ));
    }
    if o.is_instance_of::<pyo3::types::PyByteArray>()
        || o.get_type().name().map(|n| n == "memoryview").unwrap_or(false)
    {
        return Err(PyTypeError::new_err("not a file object"));
    }
    match quelle_aus(o) {
        Ok(Quelle::Puffer(_)) | Err(_) => Err(PyTypeError::new_err("not a file object")),
        Ok(andere) => Ok(andere),
    }
}

/// Laedt ein Stueck aus einer [`Quelle`] in die Musikspur.
fn musik_laden(q: Quelle) -> PyResult<()> {
    match q {
        Quelle::Pfad(p, wie) => {
            if !ist_datei(&p) {
                return Err(datei_fehlt(&wie));
            }
            music::load(&p).map_err(ton_fehler)
        }
        Quelle::Datei(b) | Quelle::Puffer(b) => music::load_bytes(&b).map_err(ton_fehler),
        // Ein `Sound` ist hier **keine** Quelle - gemessen gibt
        // `music.load(sound)` in pygame `TypeError: not a file object`.
        // `Sound(sound)` geht dagegen; die beiden Wege sind verschieden,
        // und der Uebersetzer hat genau auf diese Stelle gezeigt, als die
        // neue Variante dazukam. Ein Sammelarm waere hier stiller und
        // falscher gewesen.
        Quelle::Klang(_) => Err(PyTypeError::new_err("not a file object")),
    }
}

/// `pygame.mixer.music.load(filename, namehint="")`.
///
/// Gemessen: wirft weg, was vorher geladen war **und** was in der
/// Warteschlange stand.
///
/// `namehint` sagt pygame, welches Format zu erwarten ist, wenn der Name
/// es nicht verraet. rustgames Dekoder erkennt das Format am Inhalt und
/// braucht den Hinweis nicht; er wird angenommen und nicht benutzt.
#[pyfunction]
#[pyo3(signature = (filename, namehint=""))]
#[pyo3(name = "load")]
pub fn musik_load(filename: &Bound<'_, PyAny>, namehint: &str) -> PyResult<()> {
    let _ = namehint;
    laeuft()?;
    musik_laden(musik_quelle(filename)?)
}

/// `pygame.mixer.music.unload()`.
///
/// Gemessen: danach wirft `play()` wieder `music not loaded`.
#[pyfunction]
#[pyo3(name = "unload")]
pub fn musik_unload() -> PyResult<()> {
    music::unload().map_err(ton_fehler)
}

/// `pygame.mixer.music.queue(filename, namehint="", loops=0)`.
///
/// Gemessen: **ohne** geladenes Stueck ist das kein Fehler, sondern
/// stillschweigend erfolgreich.
#[pyfunction]
#[pyo3(signature = (filename, namehint="", loops=0))]
#[pyo3(name = "queue")]
pub fn musik_queue(filename: &Bound<'_, PyAny>, namehint: &str, loops: i32) -> PyResult<()> {
    let _ = namehint;
    laeuft()?;
    let stueck = match musik_quelle(filename)? {
        Quelle::Pfad(p, wie) => {
            if !ist_datei(&p) {
                return Err(datei_fehlt(&wie));
            }
            // Gelesen wie `music.load` - SDL_mixers Saetze, nicht SDLs.
            let daten = std::fs::read(&p).map_err(|_| datei_fehlt(&wie))?;
            Sound::from_bytes_als_musik(&daten).map_err(ton_fehler)?
        }
        Quelle::Datei(b) | Quelle::Puffer(b) => {
            Sound::from_bytes_als_musik(&b).map_err(ton_fehler)?
        }
        // Wie bei `load`: ein `Sound` ist kein Dateiobjekt.
        Quelle::Klang(_) => {
            return Err(PyTypeError::new_err("not a file object"))
        }
    };
    music::queue_sound(&stueck, loops).map_err(ton_fehler)
}

/// `pygame.mixer.music.play(loops=0, start=0.0, fade_ms=0)`.
///
/// **`loops` zaehlt hier anders als bei `Sound.play`.** Gemessen mit
/// einem Stueck von 0.1 s:
///
/// | `loops` | `Sound.play` | `music.play` |
/// |---|---|---|
/// | 0 | 1x | 1x |
/// | 1 | **2x** | **1x** |
/// | 5 | 6x | 5x |
///
/// Bei der Musik ist `loops` die Zahl der **Durchlaeufe**; `0` und `1`
/// bedeuten dasselbe. Der Grund steht in SDL_mixers `Mix_FadeInMusicPos`:
/// `if (loops == 0) loops = 1;`.
///
/// Gemessen: ohne vorheriges `load()` kommt `music not loaded`.
#[pyfunction]
#[pyo3(signature = (loops=0, start=0.0, fade_ms=0))]
#[pyo3(name = "play")]
pub fn musik_play(loops: i32, start: f64, fade_ms: i32) -> PyResult<()> {
    music::play(loops, start, fade_ms).map_err(ton_fehler)
}

/// `pygame.mixer.music.rewind()`.
///
/// Gemessen: [`musik_get_pos`] springt dabei **nicht** zurueck, sondern
/// zaehlt weiter - pygame setzt nur SDLs Stroemung zurueck, nicht seinen
/// eigenen Zaehler. Und ohne geladenes Stueck ist das kein Fehler.
#[pyfunction]
#[pyo3(name = "rewind")]
pub fn musik_rewind() -> PyResult<()> {
    music::rewind().map_err(ton_fehler)
}

/// `pygame.mixer.music.set_pos(pos)`.
///
/// Gemessen: laeuft gerade nichts, kommt
/// `pygame.error: Music isn't playing`.
#[pyfunction]
#[pyo3(name = "set_pos")]
#[pyo3(signature = (pos, /))]
pub fn musik_set_pos(pos: &Bound<'_, PyAny>) -> PyResult<()> {
    let pos: f64 = pos
        .extract()
        .map_err(|_| pyo3::exceptions::PyTypeError::new_err("set_pos expects 1 float argument"))?;
    music::set_pos(pos).map_err(ton_fehler)
}

/// `pygame.mixer.music.stop()`.
#[pyfunction]
#[pyo3(name = "stop")]
pub fn musik_stop() -> PyResult<()> {
    music::stop().map_err(ton_fehler)
}

/// `pygame.mixer.music.pause()`.
#[pyfunction]
#[pyo3(name = "pause")]
pub fn musik_pause() -> PyResult<()> {
    music::pause().map_err(ton_fehler)
}

/// `pygame.mixer.music.unpause()`.
#[pyfunction]
#[pyo3(name = "unpause")]
pub fn musik_unpause() -> PyResult<()> {
    music::unpause().map_err(ton_fehler)
}

/// `pygame.mixer.music.fadeout(time)`.
#[pyfunction]
#[pyo3(name = "fadeout")]
#[pyo3(signature = (time, /))]
pub fn musik_fadeout(time: i32) -> PyResult<()> {
    music::fadeout(time).map_err(ton_fehler)
}

/// `pygame.mixer.music.get_busy() -> bool`.
///
/// Gemessen: ein **pausiertes** Stueck gilt hier als **nicht** belegt -
/// genau umgekehrt zu `PyChannel::get_busy`, wo eine Pause als belegt
/// zaehlt. pygames `music_get_busy` prueft ausdruecklich
/// `Mix_PlayingMusic() && !Mix_PausedMusic()`.
///
/// **Anders als `mixer.get_busy()`**, das ohne laufenden Mixer schlicht
/// `False` gibt, wirft das hier `mixer not initialized` - gemessen, und
/// eine Falle, in die diese Huelle beim ersten Versuch getreten ist:
/// die beiden heissen gleich und verhalten sich verschieden.
#[pyfunction]
#[pyo3(name = "get_busy")]
pub fn musik_get_busy() -> PyResult<bool> {
    laeuft()?;
    Ok(music::get_busy())
}

/// `pygame.mixer.music.get_pos() -> time`.
///
/// **Keine Stelle im Stueck, sondern eine Stoppuhr:** sie zaehlt, wie
/// lange seit `play()` gespielt wurde, Pausen nicht mitgerechnet, und
/// springt weder bei `rewind()` noch bei `set_pos()` zurueck. Gemessen
/// `-1`, solange noch nie gespielt wurde.
///
/// **Eine bewusste Abweichung:** nach `stop()` gibt echtes pygame hier
/// keinen sinnvollen Wert mehr, sondern eine grosse Zahl - sein
/// `music_pos_time` ist ein `Uint64`, auf `-1` gesetzt, und der Test
/// `< 0` kann fuer eine vorzeichenlose Zahl nie zutreffen. Gemessen kam
/// dort `3` heraus. rustgame liefert wieder `-1`, also das, was die Doku
/// verspricht.
#[pyfunction]
#[pyo3(name = "get_pos")]
pub fn musik_get_pos() -> PyResult<i64> {
    // Gemessen: **auch** `music.get_pos()` prueft den Mischer
    // (`pygame.error: mixer not initialized`). Hier gab es `-1`, und das
    // ist derselbe Wert wie "laeuft nichts" - ein Programm konnte die
    // zwei Lagen nicht unterscheiden.
    laeuft()?;
    Ok(music::get_pos())
}

/// `pygame.mixer.music.get_volume() -> value`.
///
/// **Gemessen steht das direkt nach `mixer.init()` auf `0.9921875`, nicht
/// auf `1.0`** - pygames `_init` ruft zum Schluss `Mix_VolumeMusic(127)`,
/// eine einzelne von 128 Stufen unter voll, ohne erkennbaren Grund. Ein
/// Wert, ueber den man ewig raetselt, wenn man ihn nicht kennt.
///
/// Gemessen wirft auch das vor `init()` - anders als
/// [`musik_get_endevent`], das dort `0` liefert.
#[pyfunction]
#[pyo3(name = "get_volume")]
pub fn musik_get_volume() -> PyResult<f64> {
    laeuft()?;
    Ok(music::get_volume())
}

/// `pygame.mixer.music.set_volume(volume)`.
#[pyfunction]
#[pyo3(name = "set_volume")]
#[pyo3(signature = (volume, /))]
pub fn musik_set_volume(volume: f64) -> PyResult<()> {
    music::set_volume(volume).map_err(ton_fehler)
}

/// `pygame.mixer.music.set_endevent()` und `set_endevent(type)`.
#[pyfunction]
#[pyo3(signature = (event_type=0, /))]
#[pyo3(name = "set_endevent")]
pub fn musik_set_endevent(event_type: u32) {
    // Der oeffentliche Name ist pygames; der deutsche lebt im Rumpf.
    let type_ = event_type;
    music::set_endevent(type_);
}

/// `pygame.mixer.music.get_endevent() -> type`.
///
/// Die zweite Abfrage, die **ohne** laufenden Mixer durchgeht - gemessen
/// liefert sie vor `init()` schlicht `0`.
#[pyfunction]
#[pyo3(name = "get_endevent")]
pub fn musik_get_endevent() -> u32 {
    music::get_endevent()
}

/// `pygame.mixer.music.get_metadata() -> dict`.
///
/// Gemessen liefert pygame immer dieselben vier Schluessel - `title`,
/// `album`, `artist`, `copyright` -, und fuer eine WAV-Datei sind alle
/// vier leer. Genau das kommt hier heraus: WAV traegt keine solchen
/// Angaben, und OGG-Kommentare liest der Dekoder nicht aus. Die vier
/// Felder sind trotzdem da, damit Programme, die sie abfragen, nicht
/// umfallen.
///
/// Gemessen: ohne geladenes Stueck kommt `music not loaded`.
#[pyfunction]
#[pyo3(signature = (filename=None, namehint=None))]
#[pyo3(name = "get_metadata")]
pub fn musik_get_metadata(
    py: Python<'_>,
    filename: Option<&Bound<'_, PyAny>>,
    namehint: Option<&str>,
) -> PyResult<Py<PyDict>> {
    let _ = namehint;
    let m = match filename {
        None => music::get_metadata().map_err(ton_fehler)?,
        Some(f) => {
            let daten = match musik_quelle(f)? {
                Quelle::Pfad(p, wie) => {
                    if !ist_datei(&p) {
                        return Err(datei_fehlt(&wie));
                    }
                    std::fs::read(&p).map_err(|_| datei_fehlt(&wie))?
                }
                Quelle::Datei(b) | Quelle::Puffer(b) => b,
                _ => return Err(PyTypeError::new_err("not a file object")),
            };
            // **Eine unlesbare Datei ist ein Fehler.** Hier kam ein
            // leerer Satz Metadaten zurueck; pygame gibt
            // `pygame.error: XMP: Unrecognized file format` - es hat die
            // Datei an alle Dekodierer gereicht und keiner wollte sie.
            music::metadaten_von(&daten).map_err(ton_fehler)?
        }
    };
    let d = PyDict::new(py);
    d.set_item("title", m.title)?;
    d.set_item("album", m.album)?;
    d.set_item("artist", m.artist)?;
    d.set_item("copyright", m.copyright)?;
    Ok(d.into())
}

// ---------------------------------------------------------------------------
//  Eintragen
// ---------------------------------------------------------------------------

/// Die Python-Unterklasse von [`PySound`].
///
/// **Sie entsteht genau einmal.** `pygame.mixer.Sound` steht an zwei
/// Stellen (oben und im Untermodul), und `pygame.mixer.SoundType is
/// pygame.mixer.Sound` ist gemessen wahr - zwei Klassen mit demselben
/// Namen waeren also schon falsch, bevor jemand sie benutzt.
///
/// Warum ueberhaupt eine Huelle: PyO3 0.26 kennt keinen
/// `tp_init`-Schlitz, und ohne den sind `Sound()` und
/// `Sound.__new__(Sound)` nicht zu unterscheiden. Der ganze Grund steht
/// in `python/sound.py`.
pub fn klang_klasse(py: Python<'_>) -> PyResult<Py<PyAny>> {
    let klasse = py.get_type::<PySound>();
    // **Einmal anhaengen, nicht bei jedem Abruf.** Steht `__init__` schon
    // im eigenen Woerterbuch des Typs, ist es unsere Funktion.
    // **Gefragt wird, ob es eine Python-Funktion ist**, nicht ob
    // `__init__` ueberhaupt da ist: PyO3 traegt fuer die Rust-Methode
    // einen `method descriptor` ein, und der hat kein `__code__`. Der
    // erste Entwurf fragte nur nach dem Namen - und hing damit am eigenen
    // Eintrag fest, sodass nie etwas angehaengt wurde.
    if !klasse.getattr("__init__")?.hasattr("__code__")? {
        let hilfe = PyModule::new(py, "rustgame_py.mixer")?;
        let code = std::ffi::CString::new(include_str!("python/sound.py"))?;
        py.run(code.as_c_str(), Some(&hilfe.dict()), None)?;
        klasse.setattr("__init__", hilfe.getattr("__init__")?)?;
    }
    Ok(klasse.into_any().unbind())
}

/// Fuellt `rustgame_py.mixer`.
///
/// `music` wird hier **nicht** eingetragen - das Modul steht daneben als
/// `rustgame_py.mixer_music` und wird von `lib.rs` verknuepft, so wie
/// pygame es macht. Siehe [`eintragen_music`].
pub fn eintragen(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("Sound", klang_klasse(m.py())?)?;
    m.add_class::<PyChannel>()?;
    // `pygame.mixer.SoundType is pygame.mixer.Sound` ist **wahr** -
    // gemessen, genau wie bei `event.EventType`. Es sind keine zwei
    // Typen, sondern zwei Namen fuer denselben; also wird dieselbe Klasse
    // zweimal eingetragen und keine zweite gebaut.
    m.add("SoundType", m.getattr("Sound")?)?;
    m.add("ChannelType", m.getattr("Channel")?)?;

    m.add("init", wrap_pyfunction!(ton_init, m)?)?;
    m.add("pre_init", wrap_pyfunction!(ton_pre_init, m)?)?;
    m.add("quit", wrap_pyfunction!(ton_quit, m)?)?;
    m.add("get_init", wrap_pyfunction!(ton_get_init, m)?)?;
    m.add("get_driver", wrap_pyfunction!(ton_get_driver, m)?)?;
    m.add(
        "get_sdl_mixer_version",
        wrap_pyfunction!(ton_get_sdl_mixer_version, m)?,
    )?;
    m.add("get_soundfont", wrap_pyfunction!(ton_get_soundfont, m)?)?;
    m.add("set_soundfont", wrap_pyfunction!(ton_set_soundfont, m)?)?;
    m.add(
        "get_num_channels",
        wrap_pyfunction!(ton_get_num_channels, m)?,
    )?;
    m.add(
        "set_num_channels",
        wrap_pyfunction!(ton_set_num_channels, m)?,
    )?;
    m.add("set_reserved", wrap_pyfunction!(ton_set_reserved, m)?)?;
    m.add("find_channel", wrap_pyfunction!(ton_find_channel, m)?)?;
    m.add("get_busy", wrap_pyfunction!(ton_get_busy, m)?)?;
    m.add("stop", wrap_pyfunction!(ton_stop, m)?)?;
    m.add("pause", wrap_pyfunction!(ton_pause, m)?)?;
    m.add("unpause", wrap_pyfunction!(ton_unpause, m)?)?;
    m.add("fadeout", wrap_pyfunction!(ton_fadeout, m)?)?;
    Ok(())
}

/// Fuellt `rustgame_py.mixer_music`.
///
/// **Warum ein eigenes Modul und nicht ein Untermodul von `mixer`:**
/// weil pygame es so hat, und zwar messbar. `pygame.mixer.music` ist
/// `pygame.mixer_music`:
///
/// ```text
/// pygame.mixer.music is pygame.mixer_music  ->  True
/// pygame.mixer.music.__name__               ->  'pygame.mixer_music'
/// 'pygame.mixer_music' in sys.modules       ->  True
/// 'pygame.mixer.music' in sys.modules       ->  False
/// import pygame.mixer.music
///   -> ModuleNotFoundError: No module named 'pygame.mixer.music';
///      'pygame.mixer' is not a package
/// ```
///
/// Der letzte Punkt ist der, auf den es ankommt: ein Eintrag
/// `sys.modules["rustgame_py.mixer.music"]` waere bequem und **falsch** -
/// er liesse ein `import` gelingen, das in pygame scheitert. Der Zugriff
/// ueber den Punkt (`pygame.mixer.music.play()`) und
/// `from pygame.mixer import music` gehen beide auch ohne ihn, weil
/// `music` ein gewoehnliches Attribut des Moduls `mixer` ist.
pub fn eintragen_music(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("load", wrap_pyfunction!(musik_load, m)?)?;
    m.add("unload", wrap_pyfunction!(musik_unload, m)?)?;
    m.add("queue", wrap_pyfunction!(musik_queue, m)?)?;
    m.add("play", wrap_pyfunction!(musik_play, m)?)?;
    m.add("rewind", wrap_pyfunction!(musik_rewind, m)?)?;
    m.add("set_pos", wrap_pyfunction!(musik_set_pos, m)?)?;
    m.add("stop", wrap_pyfunction!(musik_stop, m)?)?;
    m.add("pause", wrap_pyfunction!(musik_pause, m)?)?;
    m.add("unpause", wrap_pyfunction!(musik_unpause, m)?)?;
    m.add("fadeout", wrap_pyfunction!(musik_fadeout, m)?)?;
    m.add("get_busy", wrap_pyfunction!(musik_get_busy, m)?)?;
    m.add("get_pos", wrap_pyfunction!(musik_get_pos, m)?)?;
    m.add("get_volume", wrap_pyfunction!(musik_get_volume, m)?)?;
    m.add("set_volume", wrap_pyfunction!(musik_set_volume, m)?)?;
    m.add("set_endevent", wrap_pyfunction!(musik_set_endevent, m)?)?;
    m.add("get_endevent", wrap_pyfunction!(musik_get_endevent, m)?)?;
    m.add("get_metadata", wrap_pyfunction!(musik_get_metadata, m)?)?;
    Ok(())
}

/// Der numpy-`typestr` und die Bytebreite zu einem Mixerformat.
///
/// Gemessen an `mixer.get_init()[1]`; die Zahl ist negativ, wenn die Werte
/// ein Vorzeichen haben, und `-32` meint Gleitkomma.
fn klangformat(groesse: i32) -> (&'static str, usize) {
    match groesse {
        8 => ("|u1", 1),
        -8 => ("|i1", 1),
        16 => ("<u2", 2),
        -32 => ("<f4", 4),
        _ => ("<i2", 2),
    }
}

/// Dasselbe als PEP-3118-Format fuer das Puffer-Protokoll.
///
/// Die Tabelle steht in `mixer_test._NEWBUF_export_check` ausgeschrieben:
/// `{8: "B", -8: "b", 16: "=H", -16: "=h", 32: "=I", -32: "=i", ...}`.
/// Die letzte Zeile ist die, an der man sich taeuscht: `-32` ist dort
/// **kein** Gleitkomma, sondern eine ganze Zahl mit Vorzeichen - der Test
/// nennt 32 und 64 ausdruecklich "for future consideration". Gemessen
/// meldet der Mixer unter `init(size=32)` aber `-32`, und `sndarray`
/// erwartet `float32`. Beides zugleich geht nicht; hier gilt die
/// **Feld-Schnittstelle**, weil `sndarray` sie benutzt und der
/// Puffer-Test unter `-32` gar nicht erst laeuft.
fn klangpuffer(groesse: i32) -> &'static str {
    match groesse {
        8 => "B",
        -8 => "b",
        16 => "=H",
        -32 => "f",
        _ => "=h",
    }
}

/// `mixer.Sound(array=feld)` - ein Feld von Zahlen wird ein Klang.
///
/// ## Was umgerechnet wird, und wie
///
/// Ein Feld bringt drei Dinge mit, die nicht zum Mixer passen muessen:
/// **Breite** (ein, zwei, vier oder acht Bytes je Wert), **Vorzeichen** und
/// **Byteordnung**. pygames Regel dafuer steht in `mixer_test` als
/// Rechnung, nicht als Prosa:
///
/// ```python
/// lshift = abs(format) - 8 * a.itemsize
/// if lshift >= 0:
///     # This is asymmetric with respect to downcasting.
///     a3 <<= lshift
/// ```
///
/// Gelesen: ein **schmaleres** Feld wird nach links geschoben, damit seine
/// Werte den Wertebereich des Mixers ausfuellen - ein Byte 0x7F wird unter
/// `-16` also 0x7F00 und nicht 0x007F. Ein **breiteres** Feld wird
/// dagegen nur abgeschnitten, ohne Verschiebung; der Kommentar im Test
/// nennt das ausdruecklich unsymmetrisch.
///
/// ## Die Form
///
/// Unter einem Mono-Mixer muss das Feld **eine** Achse haben, unter Stereo
/// **zwei**, und die zweite ist die Spurenzahl. pygames Wortlaut dazu ist
/// uebernommen.
fn aus_feld(o: &Bound<'_, PyAny>) -> PyResult<Sound> {
    laeuft()?;
    let sicht = crate::puffer::hol_sicht(o)?;
    let spuren = Sound::mixer_spuren().map_err(ton_fehler)? as usize;
    let groesse = Sound::mixer_groesse().map_err(ton_fehler)?;

    let (rahmen, achsen) = match (&sicht.shape[..], spuren) {
        (&[n], 1) => (n as usize, 1usize),
        (&[n, k], _) if k as usize == spuren => (n as usize, 2usize),
        (_, 1) => {
            return Err(PyValueError::new_err(
                "Array must be 1-dimensional for mono mixer",
            ))
        }
        _ => {
            return Err(PyValueError::new_err(
                "Array must be 2-dimensional for stereo mixer",
            ))
        }
    };

    let f = sicht.format.to_string_lossy().to_string();
    let sorte = crate::puffer::art(&f);
    if !matches!(
        sorte,
        crate::puffer::Art::Ganz | crate::puffer::Art::Fuell | crate::puffer::Art::Komma
    ) || !matches!(sicht.itemsize, 1 | 2 | 4 | 8)
    {
        return Err(PyValueError::new_err(format!(
            "unsupported array item format '{f}'"
        )));
    }
    // Vorzeichen: die kleinen Buchstaben sind die mit, die grossen die
    // ohne. `x` ist ein Fuellbyte und gilt als ohne.
    let kern = f.trim_start_matches(['=', '<', '>', '!', '@']);
    let kern = kern.strip_prefix('1').filter(|z| !z.is_empty()).unwrap_or(kern);
    let mit_vorzeichen = matches!(kern, "b" | "h" | "i" | "l" | "q");
    let bits = 8 * sicht.itemsize as i32;
    // **Der Schub geht auf das Ausgabeformat, nicht auf sechzehn Bit.**
    //
    // pygames Regel steht in `mixer_test` als Rechnung:
    //
    // ```python
    // lshift = abs(format) - 8 * a.itemsize
    // if lshift >= 0:
    //     # This is asymmetric with respect to downcasting.
    //     a3 <<= lshift
    // ```
    //
    // `format` ist dabei das des **Mixers**. Hier stand `16 - bits`, weil
    // der Klang damals immer 16-bittig lag; unter `init(size=8)` wurde ein
    // `uint8`-Feld damit erst auf 16 Bit hochgeschoben und beim Ablegen
    // wieder heruntergerechnet - mit dem Nullpunktversatz von 128
    // dazwischen. Aus `[0, 15, 240, 255]` wurde `[128, 143, 112, 127]`.
    let ziel_bits = groesse.abs();
    let schub = ziel_bits - bits;
    let ziel_bytes = (ziel_bits / 8).max(1) as usize;
    let ziel_vorzeichen = groesse < 0;
    let gleitkomma = groesse == -32;

    let mut roh = Vec::with_capacity(rahmen * spuren * ziel_bytes);
    for i in 0..rahmen {
        for k in 0..spuren {
            let versatz = if achsen == 1 {
                i as isize * sicht.strides[0]
            } else {
                i as isize * sicht.strides[0] + k as isize * sicht.strides[1]
            };
            let gelesen = crate::puffer::feld_lesen(&sicht, versatz);
            if gleitkomma {
                // **Fliesskomma laeuft von minus eins bis eins.** Das ist
                // SDLs `AUDIO_F32`, und `sndarray_test.test_float32`
                // schickt genau so ein Feld hinein.
                let v = if sorte == crate::puffer::Art::Komma {
                    if sicht.itemsize == 4 {
                        f32::from_bits(gelesen as u32) as f64
                    } else {
                        f64::from_bits(gelesen)
                    }
                } else {
                    vorzeichen_geben(gelesen, bits, mit_vorzeichen) as f64
                        / (1i64 << (bits - 1).max(1)) as f64
                };
                roh.extend_from_slice(&(v.clamp(-1.0, 1.0) as f32).to_le_bytes());
                continue;
            }
            let v = if sorte == crate::puffer::Art::Komma {
                let f = if sicht.itemsize == 4 {
                    f32::from_bits(gelesen as u32) as f64
                } else {
                    f64::from_bits(gelesen)
                };
                (f.clamp(-1.0, 1.0) * ((1i64 << (ziel_bits - 1)) - 1) as f64).round() as i64
            } else {
                let roh_wert = vorzeichen_geben(gelesen, bits, mit_vorzeichen);
                if schub >= 0 {
                    roh_wert << schub
                } else {
                    roh_wert
                }
            };
            // Auf die Zielbreite abschneiden und ablegen - niedrigstes Byte
            // zuerst, wie SDL es haelt.
            let maske: u64 = if ziel_bits >= 64 {
                u64::MAX
            } else {
                (1u64 << ziel_bits) - 1
            };
            let _ = ziel_vorzeichen;
            let w = (v as u64) & maske;
            for b in 0..ziel_bytes {
                roh.push((w >> (8 * b)) as u8);
            }
        }
    }
    Sound::from_buffer(&roh).map_err(ton_fehler)
}

/// Gibt einem roh gelesenen Wert sein Vorzeichen zurueck.
///
/// `feld_lesen` liefert die Bytes als vorzeichenlose Zahl; ob die oberste
/// Eins ein grosser Wert oder ein negativer ist, weiss nur das Format.
fn vorzeichen_geben(roh: u64, bits: i32, mit_vorzeichen: bool) -> i64 {
    if !mit_vorzeichen || bits >= 64 {
        return roh as i64;
    }
    let vz = 1i64 << (bits - 1);
    let m = (roh as i64) & ((1i64 << bits) - 1);
    if m & vz != 0 {
        m - (1i64 << bits)
    } else {
        m
    }
}
