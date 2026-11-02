//! zustand.rs
//! ==========
//! Der Zustand, den pygame prozessweit fuehrt - hier am Strang.
//!
//! pygame hat Modulvariablen: eine Anzeigeflaeche, eine Ereignisschlange,
//! einen Tastaturzustand. In Rust liegt dasselbe in einem `thread_local!`.
//!
//! Warum thread-lokal und nicht `static Mutex`: ein Fenster gehoert dem
//! Strang, der es geoeffnet hat (SDL verlangt das genauso, und minifbs
//! `Window` ist nicht `Send`), und so stoeren sich die Tests nicht, die
//! `cargo test` nebenlaeufig faehrt. Ein `Mutex` wuerde beides nicht
//! loesen, sondern nur verdecken.

use std::cell::RefCell;

use crate::backend::Backend;
use crate::event::Schlange;
use crate::konstanten::*;
use crate::Screen;

/// Welches Hintergrundstueck geoeffnet werden soll.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Treiber {
    /// Aus `RUSTGAME_VIDEODRIVER` lesen; unter `cargo test` immer
    /// [`Treiber::Dummy`].
    #[default]
    Automatisch,
    /// Ein echtes Fenster ueber minifb.
    Minifb,
    /// Kein Fenster - alles bleibt im Speicher.
    Dummy,
}

/// Alles, was Tastatur und Maus gerade sind.
///
/// Steht getrennt vom uebrigen Zustand, damit das Hintergrundstueck es
/// fortschreiben kann, waehrend es selbst ausgeliehen ist - Rust erlaubt
/// zwei Ausleihen auf zwei verschiedene Felder desselben Structs.
#[derive(Debug)]
pub struct Eingabe {
    /// Welche Tastenplaetze unten sind.
    ///
    /// Indiziert wird mit dem **Tastenplatz**, nicht dem Tastencode -
    /// gemessen: `len(pygame.key.get_pressed())` ist 512, und `K_UP` ist
    /// 1073741906, passt also gar nicht hinein.
    pub tasten: Box<[bool; NUM_SCANCODES]>,
    /// Die gehaltenen Zusatztasten.
    pub mods: KeyMod,
    /// Wo der Zeiger ist.
    pub maus_pos: (i32, i32),
    /// Wie weit er sich seit dem letzten [`crate::mouse::get_rel`] bewegt hat.
    pub maus_rel: (i32, i32),
    /// Welche Maustasten unten sind (bis zu fuenf, wie bei pygame).
    pub maus_tasten: [bool; 5],
    /// Ist der Zeiger sichtbar?
    pub maus_sichtbar: bool,
    /// Hat das Fenster den Tastaturfokus?
    pub tast_fokus: bool,
    /// Ist der Zeiger ueber dem Fenster?
    pub maus_fokus: bool,
    /// `key.set_repeat` - (Verzoegerung, Abstand) in Millisekunden.
    pub wiederholung: (i32, i32),
    /// Nimmt das Fenster gerade Texteingaben an?
    pub text_eingabe: bool,
    /// Welche Tastenplaetze in **diesem** Durchlauf heruntergegangen sind.
    ///
    /// Das ist `key.get_just_pressed()`. Gemessen an pygame: die Tabelle
    /// wird zurueckgesetzt, sobald wieder Ereignisse verarbeitet werden -
    /// also bei jedem Blick in die Schlange, siehe
    /// `event::durchlauf`.
    pub tasten_neu: Box<[bool; NUM_SCANCODES]>,
    /// Welche Tastenplaetze in diesem Durchlauf heraufgekommen sind -
    /// `key.get_just_released()`.
    pub tasten_weg: Box<[bool; NUM_SCANCODES]>,
    /// Welche Maustasten in diesem Durchlauf heruntergegangen sind -
    /// `mouse.get_just_pressed()`.
    pub maus_neu: [bool; 5],
    /// Welche Maustasten in diesem Durchlauf heraufgekommen sind -
    /// `mouse.get_just_released()`.
    pub maus_weg: [bool; 5],
    /// Laeuft die Maus im Relativbetrieb? - `mouse.get_relative_mode()`.
    ///
    /// Gemessen: das ist der einzige Maus-Aufruf, der auch **vor**
    /// `display.init()` klaglos antwortet, und er antwortet `False`.
    pub relativ: bool,
    /// Die Zeigerform - `mouse.get_cursor()`.
    ///
    /// Gemessen: ganz am Anfang ist das
    /// `Cursor(type: system, constant: SYSTEM_CURSOR_ARROW)`.
    pub zeiger: crate::mouse::Cursor,
    /// Wohin die Kandidatenliste einer Eingabemethode gehoert -
    /// `key.set_text_input_rect()`.
    pub text_eingabe_rect: Option<rustgame_core::Rect>,
}

impl Default for Eingabe {
    fn default() -> Eingabe {
        Eingabe {
            tasten: Box::new([false; NUM_SCANCODES]),
            mods: 0,
            maus_pos: (0, 0),
            maus_rel: (0, 0),
            maus_tasten: [false; 5],
            maus_sichtbar: true,
            tast_fokus: false,
            maus_fokus: false,
            wiederholung: (0, 0),
            text_eingabe: true,
            tasten_neu: Box::new([false; NUM_SCANCODES]),
            tasten_weg: Box::new([false; NUM_SCANCODES]),
            maus_neu: [false; 5],
            maus_weg: [false; 5],
            relativ: false,
            zeiger: crate::mouse::Cursor::System(SYSTEM_CURSOR_ARROW),
            text_eingabe_rect: None,
        }
    }
}

impl Eingabe {
    /// Setzt oder loescht eine Taste und schreibt die Zusatztasten nach.
    ///
    /// Nebenbei werden die Kantentabellen fuer
    /// [`crate::key::get_just_pressed`] und
    /// [`crate::key::get_just_released`] gefuellt. Eine Taste, die schon
    /// unten ist, zaehlt **nicht** noch einmal als frisch gedrueckt -
    /// sonst wuerde jede Tastenwiederholung sie erneut melden.
    pub fn taste_setzen(&mut self, key: Key, unten: bool) {
        let s = crate::key::scancode_von(key) as usize;
        if s < NUM_SCANCODES {
            if unten && !self.tasten[s] {
                self.tasten_neu[s] = true;
            } else if !unten && self.tasten[s] {
                self.tasten_weg[s] = true;
            }
            self.tasten[s] = unten;
        }
        if let Some(bit) = zusatzbit(key) {
            if unten {
                self.mods |= bit;
            } else {
                self.mods &= !bit;
            }
        }
    }

    /// Setzt oder loescht eine Maustaste - `button` zaehlt wie bei pygame
    /// ab 1 (links).
    ///
    /// Fuellt dieselben Kantentabellen wie [`Eingabe::taste_setzen`],
    /// nur fuer [`crate::mouse::get_just_pressed`].
    pub fn maustaste_setzen(&mut self, button: u8, unten: bool) {
        if !(1..=5).contains(&button) {
            return;
        }
        let i = button as usize - 1;
        if unten && !self.maus_tasten[i] {
            self.maus_neu[i] = true;
        } else if !unten && self.maus_tasten[i] {
            self.maus_weg[i] = true;
        }
        self.maus_tasten[i] = unten;
    }

    /// Loescht die vier Kantentabellen - der Anfang eines Durchlaufs.
    ///
    /// Gerufen wird das aus `event::durchlauf`, also genau dann,
    /// wenn jemand in die Ereignisschlange sieht. Das ist pygames
    /// Zeitpunkt: "updated when new events are processed, e.g. in
    /// `pygame.event.get()` or `pygame.event.pump()`".
    pub fn kanten_loeschen(&mut self) {
        self.tasten_neu.fill(false);
        self.tasten_weg.fill(false);
        self.maus_neu = [false; 5];
        self.maus_weg = [false; 5];
    }
}

/// Welches `KMOD_*`-Bit zu einer Zusatztaste gehoert.
///
/// Die Werte stammen aus [`crate::konstanten`], sind also aus echtem
/// pygame ausgelesen; nur die Zuordnung Taste -> Bit steht hier.
fn zusatzbit(key: Key) -> Option<KeyMod> {
    Some(match key {
        K_LSHIFT => KMOD_LSHIFT,
        K_RSHIFT => KMOD_RSHIFT,
        K_LCTRL => KMOD_LCTRL,
        K_RCTRL => KMOD_RCTRL,
        K_LALT => KMOD_LALT,
        K_RALT => KMOD_RALT,
        K_LGUI => KMOD_LGUI,
        K_RGUI => KMOD_RGUI,
        K_NUMLOCK => KMOD_NUM,
        K_CAPSLOCK => KMOD_CAPS,
        K_MODE => KMOD_MODE,
        _ => return None,
    })
}

/// Der ganze Zustand dieser Kiste.
pub struct Zustand {
    /// Ist `display.init()` gelaufen?
    pub init: bool,
    /// Das offene Hintergrundstueck - erst nach `set_mode` da.
    pub backend: Option<Box<dyn Backend>>,
    /// Die Anzeigeflaeche.
    pub screen: Option<Screen>,
    /// Die Flaggen, mit denen `set_mode` gerufen wurde.
    pub flags: u32,
    /// Fenstertitel und Titel des Sinnbilds.
    pub caption: (String, String),
    /// Hat jemand den Titel **ausdruecklich** gesetzt?
    ///
    /// `display.get_caption()` ist leer, solange kein Fenster offen ist
    /// **und** niemand einen Titel gesetzt hat - gemessen:
    ///
    /// ```text
    /// frisch eingefuehrt      ()
    /// nach set_caption('x')   ('x', 'x')
    /// nach init() ohne Modus  ()
    /// nach set_mode           ('pygame window', 'pygame window')
    /// nach quit()             ()
    /// ```
    ///
    /// Der Titel selbst steht daneben weiter als Vorgabe, denn ein
    /// Fenster, das aufgeht, traegt ihn sofort.
    pub caption_gesetzt: bool,
    /// Die Ereignisschlange.
    pub schlange: Schlange,
    /// Tastatur und Maus.
    pub eingabe: Eingabe,
    /// Welches Stueck beim naechsten `set_mode` geoeffnet wird.
    pub treiber: Treiber,
    /// Haelt das Fenster Maus und Tastatur fest? - `event.get_grab()`.
    ///
    /// Gemessen: `set_grab(True)` **ohne** Fenster laeuft klaglos durch,
    /// bleibt aber wirkungslos - `get_grab()` sagt danach weiterhin
    /// `False`. Hier steht darum, was wirklich gilt, nicht was verlangt
    /// wurde.
    pub griff: bool,
    /// Darf der Bildschirmschoner laufen? - `display.get_allow_screensaver()`.
    ///
    /// Gemessen: die Vorgabe ist `False` - pygame haelt den
    /// Bildschirmschoner waehrend des Spiels an.
    pub bildschirmschoner: bool,
    /// Die Fenster als Dinge - `pygame.window`. Siehe [`crate::fenster`].
    pub fenster: crate::fenster::Ablage,
}

impl Zustand {
    fn neu() -> Zustand {
        Zustand {
            init: false,
            backend: None,
            screen: None,
            flags: 0,
            // Der Titel, den ein neues Fenster bekommt. **Nicht** der,
            // den `get_caption()` vor dem ersten Fenster meldet - dort
            // ist es ein leeres Tupel, siehe `caption_gesetzt`. Hier
            // stand "gemessen: ('pygame window', ...)", und das war wahr
            // fuer den Zustand **mit** Fenster.
            caption: ("pygame window".into(), "pygame window".into()),
            caption_gesetzt: false,
            schlange: Schlange::neu(),
            eingabe: Eingabe::default(),
            treiber: Treiber::Automatisch,
            griff: false,
            bildschirmschoner: false,
            fenster: crate::fenster::Ablage::neu(),
        }
    }
}

thread_local! {
    static ZUSTAND: RefCell<Zustand> = RefCell::new(Zustand::neu());
}

/// Leiht den Zustand aus.
///
/// Der Zustand darf waehrenddessen nicht noch einmal ausgeliehen werden -
/// deshalb steht in keiner dieser Klammern ein Aufruf nach draussen.
pub(crate) fn mit_zustand<R>(f: impl FnOnce(&mut Zustand) -> R) -> R {
    ZUSTAND.with(|z| f(&mut z.borrow_mut()))
}
