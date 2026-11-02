//! rustgame-mixer
//! ==============
//! Gegenstueck zu `pygame.mixer` und `pygame.mixer.music`.
//!
//! ## Ein Uhrwerk, zwei Treiber
//!
//! Die Kiste faehrt nach demselben Muster wie [`rustgame_display`]s
//! Hintergrundstuecke: ein Trait, zwei Umsetzungen. Der Unterschied liegt
//! darin, **wie wenig** der Treiber zu tun hat.
//!
//! Alles, was man von aussen sehen kann - ob ein Kanal belegt ist, wie
//! lange ein Stueck schon laeuft, was in der Warteschlange haengt, welche
//! Lautstaerke eingestellt ist - rechnet `zustand` aus
//! [`rustgame_core::time::get_ticks`] aus. Der Treiber bekommt nur den
//! Auftrag "mach daraus Schall" und wird nie gefragt. Deshalb verhalten
//! sich beide Treiber in jedem messbaren Punkt gleich:
//!
//! | Treiber | Schall? | wofuer |
//! |---|---|---|
//! | [`treiber::RodioTreiber`] | ja | der Normalfall am Schreibtisch |
//! | [`treiber::StummerTreiber`] | nein | kopflos, eingebettet, getestet |
//!
//! Eingeschaltet wird der stille Treiber mit `RUSTGAME_AUDIODRIVER=dummy`
//! oder ausdruecklich mit [`mixer::set_treiber`]. Unter `cargo test` ist
//! er die Vorgabe - **kein Testlauf dieser Kiste oeffnet ein Tongeraet.**
//! Auf einem Rechner ganz ohne Soundkarte gibt [`mixer::init`] einen
//! sauberen [`Fehler::KeinGeraet`] zurueck; [`Sound`] und [`Channel`]
//! lassen sich davon unabhaengig anlegen, befuellen und abfragen.
//!
//! ## Gemessen, nicht geraten
//!
//! Die Zahlen in dieser Kiste stammen aus echtem pygame-ce 2.5.7 mit
//! `SDL_AUDIODRIVER=dummy`. Einiges davon widerspricht dem, was man
//! erwarten wuerde, und steht genau deshalb hier:
//!
//! * `mixer.init()` gibt nichts zurueck, `get_init()` davor ist `None`,
//!   danach `(44100, -16, 2)` - und es sind **8** Kanaele, nicht 16.
//! * [`Channel::set_volume`] mit **zwei** Werten stellt nicht die
//!   Lautstaerke ein, sondern die Balance - und setzt die Lautstaerke auf
//!   1.0. `get_volume()` liefert danach `1.0`, und die Balance laesst sich
//!   ueberhaupt nicht mehr auslesen.
//! * Lautstaerken rasten auf **128 Stufen** ein: `set_volume(0.1)` ergibt
//!   `0.09375`, und alles unter `1/256` wird zu `0.0`.
//! * `Sound.play(loops=1)` spielt **zweimal**, `music.play(loops=1)` aber
//!   nur **einmal** - die beiden zaehlen verschieden.
//! * `music.get_volume()` steht direkt nach `init()` auf `0.9921875`,
//!   nicht auf `1.0`.
//!
//! Die Einzelheiten stehen jeweils an Ort und Stelle.
//!
//! ## Ein Beispiel
//!
//! ```
//! use rustgame_mixer::{mixer, synth};
//! use rustgame_mixer::treiber::Wahl;
//!
//! // Damit dieses Beispiel im Testlauf keine Lautsprecher anwirft.
//! // Im Spiel laesst man die Zeile weg.
//! mixer::set_treiber(Wahl::Dummy);
//!
//! // Ohne Tongeraet ist das hier ein sauberes Err - und alles Weitere
//! // geht trotzdem.
//! let _ = mixer::init(Default::default());
//! if let Ok(klick) = synth::sound("click") {
//!     klick.set_volume(0.4);
//!     let _kanal = klick.play(0, 0, 0);
//! }
//! mixer::quit();
//! ```
//!
//! ## Was hier nicht liegt
//!
//! Die Ereignisschlange. `set_endevent` schreibt in pygame in dieselbe
//! Schlange, aus der `pygame.event.get()` liest - die liegt aber in
//! [`rustgame_display`], und diese Kiste haengt nicht daran. Stattdessen
//! sammelt sie die faelligen Ereignisse selbst; siehe [`ereignis`] fuer
//! die zwei Wege, auf denen `rustgame-display` sie spaeter abholen kann.
//!
//! [`rustgame_display`]: https://docs.rs/rustgame-display

// **Kein `unsafe` in dieser Kiste, und das prueft der Uebersetzer.**
//
// Hier stand der Satz lange nur in der Beschreibung - und eine Behauptung,
// die niemand nachhaelt, veraltet still. Siehe
// `rustgame-py/src/puffer.rs`: dort liegt das `unsafe` des Projekts, und
// nur dort ist der Lint ausdruecklich abgeschaltet.
#![forbid(unsafe_code)]

#![deny(missing_docs)]

mod aiff;
pub mod channel;
mod filter;
pub mod ereignis;
pub mod geraete;
pub mod id3;
pub mod konstanten;
pub mod midi;
pub mod mixer;
pub mod music;
pub mod sound;
pub mod synth;
pub mod umrechnen;
mod voc;
pub mod treiber;
pub mod wav;
mod zustand;

pub use channel::Channel;
pub use ereignis::Endereignis;
pub use mixer::Einstellungen;
pub use sound::Sound;

/// Was in dieser Kiste schiefgehen kann.
///
/// Die Texte sind die von pygame - dort kommen sie als `pygame.error`,
/// `ValueError`, `IndexError` oder `FileNotFoundError` heraus und sind
/// gemessen, nicht erfunden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fehler {
    /// `mixer.init()` fehlt noch.
    ///
    /// pygame sagt dazu woertlich `mixer not initialized`. Gemessen: fast
    /// alles wirft das - nur `get_busy()` und `quit()` nicht, die geben
    /// stillschweigend `False` beziehungsweise nichts zurueck.
    NichtInitialisiert,
    /// Es gibt kein Tongeraet, oder es liess sich nicht oeffnen.
    ///
    /// **Das ist der Fall, um den sich diese Kiste dreht.** Ein Rechner
    /// ohne Soundkarte bekommt hier ein `Err` und kann trotzdem alles
    /// andere benutzen - siehe [`treiber::StummerTreiber`].
    KeinGeraet(String),
    /// Der Kanalplatz gibt es nicht.
    ///
    /// pygame wirft `IndexError: invalid channel index` - gemessen fuer
    /// `Channel(8)` bei acht Kanaelen und fuer `Channel(-1)`.
    UngueltigerKanal(usize),
    /// `music.play()` ohne vorheriges `music.load()`.
    ///
    /// pygame sagt `music not loaded`.
    NichtGeladen,
    /// `music.set_pos()`, waehrend nichts laeuft.
    ///
    /// pygame sagt `Music isn't playing`.
    SpieltNicht,
    /// Die Datei war nicht da oder liess sich nicht lesen.
    Datei(String),
    /// Die Daten liessen sich nicht als Klang lesen.
    ///
    /// pygame sagt bei Unsinn woertlich
    /// `Couldn't read first 12 bytes of audio data`.
    Format(String),
    /// Das Format kennt diese Kiste, kann es aber nicht.
    ///
    /// Trifft auf MP3 zu: **eine saubere Meldung statt einer Panik** ist
    /// hier ausdruecklich verlangt.
    NichtUnterstuetzt(String),
    /// Ein Wert ausserhalb des Erlaubten.
    ///
    /// pygame wirft `ValueError`, etwa `unsupported size 24` oder
    /// `'channels' must be 1, 2, 4, or 6`.
    Wert(String),
}

impl std::fmt::Display for Fehler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fehler::NichtInitialisiert => write!(f, "mixer not initialized"),
            Fehler::KeinGeraet(s) => write!(f, "{s}"),
            Fehler::UngueltigerKanal(_) => write!(f, "invalid channel index"),
            Fehler::NichtGeladen => write!(f, "music not loaded"),
            Fehler::SpieltNicht => write!(f, "Music isn't playing"),
            Fehler::Datei(s) => write!(f, "{s}"),
            Fehler::Format(s) => write!(f, "{s}"),
            Fehler::NichtUnterstuetzt(s) => write!(f, "{s}"),
            Fehler::Wert(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for Fehler {}

/// Kurzform fuer alles, was hier schiefgehen kann.
pub type Ergebnis<T> = Result<T, Fehler>;
