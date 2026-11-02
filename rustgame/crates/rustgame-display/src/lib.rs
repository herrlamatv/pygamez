//! rustgame-display
//! ================
//! Gegenstueck zu `pygame.display`, `pygame.event`, `pygame.key`,
//! `pygame.mouse`, `pygame.window`.
//!
//! | Modul | pygame-Gegenstueck |
//! |---|---|
//! | [`display`] | `pygame.display` - das eine Fenster als Modulzustand |
//! | [`fenster`] | `pygame.window` - das Fenster als **Ding**, und mehrere davon |
//! | [`event`] | `pygame.event` |
//! | [`key`] | `pygame.key` |
//! | [`mouse`] | `pygame.mouse` und `pygame.cursors` |
//! | [`joystick`] | `pygame.joystick` |
//! | [`scrap`] | `pygame.scrap` |
//! | [`system`] | `pygame.system` |
//! | [`tk`] | kein pygame-Gegenstueck: die Bruecke nach Tkinter |
//! | [`konstanten`] | die `K_*`, `KMOD_*`, Ereignistypen und Flaggen |
//!
//! Wer hier ein Modul anlegt, traegt es ein. Diese Tabelle stand lange
//! nicht da, und deshalb fiel niemandem auf, dass `pygame.window`
//! vollstaendig fehlte.
//!
//! Hier sitzt alles, was ein Fenster und eine Tastatur braucht - im
//! Gegensatz zu [`rustgame_core`], das ganz ohne Betriebssystem auskommt.
//!
//! ## Zwei Hintergrundstuecke, und das ohne Fenster ist der Hauptfall
//!
//! Die Kiste redet nie unmittelbar mit einem Fenster, sondern immer durch
//! [`backend::Backend`]. Davon gibt es zwei:
//!
//! | Hintergrundstueck | wofuer |
//! |---|---|
//! | [`backend::MinifbBackend`] | ein echtes Fenster, ueber `minifb` |
//! | [`backend::DummyBackend`] | **kein** Fenster - Bild und Ereignisse liegen nur im Speicher |
//!
//! Das Dummy-Stueck ist nicht die Notloesung fuer Testlaeufe, sondern der
//! eigentliche Betriebsfall. Das umgebende Projekt faehrt pygame genau so:
//! `SDL_VIDEODRIVER=dummy`, `set_mode((1, 1))`, und das fertige Bild wandert
//! als PPM in ein Tkinter-Fenster. rustgame macht es genauso - siehe
//! [`backend::dummy`] fuer das Einspeisen von Ereignissen und
//! [`backend::dummy::ppm`] fuer den Bildweg. Und weil das Stueck da ist,
//! laeuft die gesamte Testsuite kopflos.
//!
//! Gewaehlt wird ueber die Umgebungsvariable `RUSTGAME_VIDEODRIVER`
//! (`dummy` schaltet das fensterlose Stueck ein, alles andere nimmt
//! minifb) oder ausdruecklich mit [`display::set_treiber`]. Unter
//! `cargo test` ist `dummy` die Vorgabe, damit kein Testlauf ein Fenster
//! aufreisst.
//!
//! ## Warum minifb und nicht winit
//!
//! pygames Ereignisschlange wird **geholt**: `event.get()` fragt, wann es
//! passt. `winit` dreht das um und ruft zurueck, wenn *es* passt - diese
//! Schleife muesste man erst umstuelpen und dann wieder zurueck, mit einer
//! zweiten Warteschlange dazwischen. `minifb` wird ebenfalls abgefragt
//! (`get_keys`, `get_mouse_pos`, ...) und passt damit ohne Umweg.
//!
//! Der Preis: minifb kennt gar keine Ereignisse, nur Zustaende. Die
//! Kantenerkennung - aus "Taste ist unten" wird `KEYDOWN`, sobald sie
//! vorher oben war - macht [`backend::MinifbBackend`].
//!
//! ## Zustand liegt am Strang (thread-lokal)
//!
//! pygame haelt seinen Anzeigezustand prozessweit. Hier liegt er
//! **thread-lokal**, aus zwei Gruenden: ein Fenster gehoert ohnehin dem
//! Strang, der es geoeffnet hat (SDL verlangt das genauso), und so stoeren
//! sich die Tests nicht, die `cargo test` nebenlaeufig faehrt.
//!
//! **Die Flaeche selbst ist davon nicht betroffen.** Ein [`Screen`] und
//! jede [`rustgame_core::Surface`] duerfen den Faden wechseln; nur das
//! Fenster und die Ereignisschlange bleiben, wo sie sind. Siehe
//! [`Screen`].
//!
//! ## Die Anzeigeflaeche
//!
//! [`display::set_mode`] liefert einen [`Screen`] - einen geteilten
//! Behaelter um die Flaeche. Das ist nicht Bequemlichkeit, sondern
//! **gemessen**: in echtem pygame gibt `set_mode` immer wieder *dasselbe*
//! Objekt zurueck, selbst wenn man die Groesse aendert. Ein geteilter
//! Behaelter bildet das genau ab - `set_mode`, [`display::get_surface`]
//! und das, was `flip` anzeigt, sind eine und dieselbe Flaeche.
//!
//! ```no_run
//! use rustgame_display::{display, event, konstanten::*};
//! use rustgame_core::draw;
//!
//! display::init().unwrap();
//! let bild = display::set_mode((320, 240), 0).unwrap();
//! display::set_caption("Beispiel");
//! 'lauf: loop {
//!     for e in event::get() {
//!         if e.typ == QUIT || e.key() == Some(K_ESCAPE) {
//!             break 'lauf;
//!         }
//!     }
//!     {
//!         let mut s = bild.write();
//!         s.fill((20, 20, 30));
//!         draw::circle(&mut *s, (255, 80, 80), (160, 120), 40, 0);
//!     }
//!     display::flip().unwrap();
//! }
//! display::quit();
//! ```
//!
//! Wer waehrend [`display::flip`] noch eine Leihe haelt, bekommt eine
//! Panik - deshalb steht das Zeichnen oben in einem eigenen Block.
//!
//! ## Was diese Kiste ehrlich nicht kann
//!
//! Ein guter Teil von `pygame.display` fragt das Fenstersystem. minifb
//! fragt es fast nie, und unter dem fensterlosen Stueck gibt es gar
//! keines. Die Regel hier ist deshalb: **was echtes pygame unter
//! `SDL_VIDEODRIVER=dummy` liefert, liefert diese Kiste auch** - und wo
//! es nicht geht, steht ein [`Fehler::NichtUnterstuetzt`] mit Grund im
//! Doc-Kommentar, keine erfundene Zahl.
//!
//! | Aufruf | ohne Fenster | mit minifb |
//! |---|---|---|
//! | [`display::get_desktop_sizes`] | `[(1024, 768)]` - gemessen | nein, minifb kennt keine Bildschirmliste |
//! | [`display::get_current_refresh_rate`] | `60` - gemessen | nein, minifb fragt den Schirm nie |
//! | [`display::get_window_position`] | mittig, einmal gesetzt | **ja**, `Window::get_position()` |
//! | [`display::set_window_position`] | ja | **ja**, ausser mittig |
//! | [`display::get_wm_info`] | leer - gemessen | **ja**, die Fensternummer |
//! | [`display::iconify`] | `true`, tut nichts - gemessen | nein |
//! | [`display::toggle_fullscreen`] | nein - gemessen | nein |
//! | [`mouse::set_cursor`] mit eigenem Bild | ja - gemessen | nein |
//! | [`mouse::set_cursor`] mit Systemform | nein - gemessen | **ja**, neun von zwoelf |
//! | [`event::set_grab`] | ja - gemessen | nein |
//! | [`display::gl_get_attribute`] / [`display::gl_set_attribute`] | nein | nein - es gibt keinen GL-Kontext |
//! | [`display::message_box`] | nein - gemessen | nein |
//! | [`display::set_gamma`] / [`display::set_gamma_ramp`] | `false` - gemessen | `false` |
//! | [`mouse::set_relative_mode`] | nein - gemessen | nein |
//! | [`fenster::Fenster::set_deckkraft`] | nein - gemessen auch bei pygame nein | nein |
//! | [`fenster::Fenster::blinken`] | nein - gemessen auch bei pygame nein | nein |
//! | [`fenster::Fenster::modal_fuer`] | nein - gemessen auch bei pygame nein | nein |
//! | [`fenster::Fenster::zustand_setzen`] | ja, tut nichts - gemessen | nein, minifb hat kein `minimize()` |
//! | [`fenster::Fenster::zeigen`] / [`fenster::Fenster::verstecken`] | ja - gemessen | nein |
//! | [`fenster::Fenster::set_vollbild`] | **ja** - gemessen | nein |
//! | [`fenster::Fenster::set_obenauf`] | nein - gemessen auch bei pygame nein | **ja**, `minifb::Window::topmost` |
//!
//! Die vorletzte Zeile ist der Rede wert: `Window.set_fullscreen()` geht
//! unter `SDL_VIDEODRIVER=dummy`, `display.toggle_fullscreen()` im
//! selben Lauf nicht. Beides gemessen.
//!
//! Die letzte ist die Berichtigung eines **veralteten Vermerks**: dass
//! minifb kein "immer obenauf" koenne, stand hier als gesetzt und ist
//! falsch - `minifb::Window::topmost` gibt es, und unter Windows ist es
//! ein echtes `SetWindowPos(.., HWND_TOPMOST, ..)`.
//!
//! ## Die zwei Naehte nach aussen
//!
//! Zwei Dinge aus anderen Kisten haben hier ihre Zustellung, weil die
//! Ereignisschlange nun einmal hier liegt:
//!
//! * **Zeitgeber.** `rustgame_core::time::set_timer` fuehrt nur Buch;
//!   zugestellt wird bei jedem Blick in die Schlange - siehe
//!   [`event`].
//! * **Endereignisse des Tons.** `rustgame_mixer::ereignis` sammelt
//!   sie und bietet einen Rueckruf an; [`display::init`] setzt ihn.

// **Kein `unsafe` in dieser Kiste, und das prueft der Uebersetzer.**
//
// Hier stand der Satz lange nur in der Beschreibung - und eine Behauptung,
// die niemand nachhaelt, veraltet still. Siehe
// `rustgame-py/src/puffer.rs`: dort liegt das `unsafe` des Projekts, und
// nur dort ist der Lint ausdruecklich abgeschaltet.
#![forbid(unsafe_code)]

#![deny(missing_docs)]


pub mod backend;
pub mod display;
pub mod event;
pub mod fenster;
pub mod joystick;
pub mod kamera;
pub mod key;
pub mod konstanten;
pub mod mouse;
pub mod scrap;
pub mod system;
pub mod tk;
mod zustand;

pub use event::Event;
pub use konstanten::{EventType, Key, KeyMod, Scancode};

/// Die Anzeigeflaeche, wie [`display::set_mode`] sie liefert.
///
/// Warum geteilt und nicht einfach eine [`rustgame_core::Surface`]:
/// gemessen an echtem pygame gibt `set_mode` bei jedem Aufruf dasselbe
/// Objekt zurueck - auch nach einem Groessenwechsel wird die vorhandene
/// Flaeche umgebaut, nicht ersetzt. Ein gemeinsamer Behaelter bildet genau
/// das ab.
///
/// ## Hier stand `Rc<RefCell<..>>`
///
/// Und das war richtig, solange die Flaeche einen Faden nicht verliess.
/// `image_test.test_load_bmp_threads` laedt Bilder in zwanzig Faeden und
/// gibt sie an den Hauptfaden weiter; die Huelle musste darum
/// `unsendable` sein, und beim **Wegraeumen** im falschen Faden kam ein
/// Fehler, der wuerfelte. Seit der Kern `Arc<ReentrantMutex<RefCell<..>>>`
/// benutzt, zieht diese Zeile mit.
///
/// `RwLock` und nicht `Mutex`, weil `RefCell` viele Leser erlaubt und ein
/// `Mutex` daraus einen Fehler gemacht haette, den es vorher nicht gab.
/// Gesperrt wird ausschliesslich mit `try_read` und `try_write`: wer
/// waehrend [`display::flip`] noch eine Leihe haelt, bekommt weiterhin
/// eine **Panik** und keinen Haenger. Siehe [`GELEIHT`].
pub type Screen = std::sync::Arc<parking_lot::RwLock<rustgame_core::Surface>>;

/// Was dasteht, wenn jemand die Anzeigeflaeche noch ausgeliehen hat.
///
/// Dieselbe Lage, die `RefCell` mit `already mutably borrowed` meldete -
/// nur sagt die Meldung jetzt, worum es geht.
pub const GELEIHT: &str =
    "die Anzeigeflaeche ist noch ausgeliehen - Zeichnen und Anzeigen \
gehoeren in getrennte Bloecke";

/// Was in dieser Kiste schiefgehen kann.
///
/// Die Texte sind die von pygame - dort kommen sie als `pygame.error`
/// heraus und sind gemessen, nicht erfunden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fehler {
    /// `display.init()` fehlt noch.
    ///
    /// pygame sagt dazu `video system not initialized`.
    NichtInitialisiert,
    /// Es gibt noch keine Anzeigeflaeche, `set_mode` fehlt.
    ///
    /// pygame sagt `Display mode not set`.
    KeinModus,
    /// Es gibt kein offenes Fenster.
    ///
    /// Sieht aus wie [`Fehler::KeinModus`], ist aber ein anderer Text und
    /// trifft andere Aufrufe: gemessen sagt `flip()` ohne `set_mode`
    /// `Display mode not set`, `get_window_position()` dagegen
    /// `No open window`. Wer beides zusammenwirft, gibt Programmen, die
    /// auf den Text pruefen, den falschen.
    KeinFenster,
    /// Das Hintergrundstueck kann das nicht.
    ///
    /// pygame sagt bei `toggle_fullscreen()` unter dem Dummy-Treiber
    /// woertlich `That operation is not supported` - gemessen, und der
    /// Grund, warum das hier ein Fehler ist und keine 0.
    ///
    /// Der Text im Inneren ist **nicht** der von pygame, sondern der
    /// Grund fuer diese Kiste - warum genau minifb oder das fensterlose
    /// Stueck es nicht kann. Er steht in `Debug`, nicht in `Display`.
    NichtUnterstuetzt(&'static str),
    /// Ein Fehler, den pygame mit eigenen Worten meldet.
    ///
    /// Fuer die Faelle, die keiner der obigen trifft. Alle Texte sind an
    /// echtem pygame unter `SDL_VIDEODRIVER=dummy` gemessen:
    ///
    /// | Aufruf | Text |
    /// |---|---|
    /// | `display.message_box(...)` | `No message system available` |
    /// | `display.set_palette(...)` | `Display mode is not colormapped` |
    /// | `display.gl_get_attribute(...)` | `No dynamic GL support in current SDL video driver (dummy)` |
    /// | `mouse.set_relative_mode(True)` | `No relative mode implementation available` |
    /// | `mouse.set_cursor(SYSTEM_CURSOR_*)` | `Error while creating system cursor` |
    /// | `event.custom_type()` nach dem letzten | `pygame.event.custom_type made too many event types.` |
    Pygame(&'static str),
    /// Ein Wert, den pygame als **`ValueError`** zurueckweist.
    ///
    /// Steht getrennt von [`Fehler::Pygame`], weil pygame hier keine
    /// `pygame.error` wirft, sondern einen `ValueError` - wer das
    /// zusammenwirft, gibt einem `except ValueError:` den falschen
    /// Zweig. Alle Texte sind an `pygame.window.Window` gemessen:
    ///
    /// | Aufruf | Text |
    /// |---|---|
    /// | `Window('T', (0, 48))` | `width or height should not be less than or equal to zero.` |
    /// | `w.size = (0, 48)` | derselbe Satz **ohne** den Punkt |
    /// | `w.minimum_size = (-1, 10)` | `minimum width or height should not be less than zero` |
    /// | `w.maximum_size = (-1, 10)` | `maximum width or height should not be less than zero` |
    /// | `w.minimum_size` ueber `maximum_size` | `minimum width or height should not be greater than maximum width or height respectively` |
    /// | `w.maximum_size` unter `minimum_size` | `maximum width or height should not be less than minimum width or height respectively` |
    /// | `w.flash(99)` | `Unsupported window flash operation.` |
    Wert(&'static str),
    /// Das Fenster liess sich nicht oeffnen.
    Fenster(String),
}

impl std::fmt::Display for Fehler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fehler::NichtInitialisiert => write!(f, "video system not initialized"),
            Fehler::KeinModus => write!(f, "Display mode not set"),
            Fehler::KeinFenster => write!(f, "No open window"),
            Fehler::NichtUnterstuetzt(_) => write!(f, "That operation is not supported"),
            Fehler::Pygame(s) => write!(f, "{s}"),
            Fehler::Wert(s) => write!(f, "{s}"),
            Fehler::Fenster(s) => write!(f, "{s}"),
        }
    }
}

impl std::error::Error for Fehler {}

/// Kurzform fuer alles, was hier schiefgehen kann.
pub type Ergebnis<T> = Result<T, Fehler>;
