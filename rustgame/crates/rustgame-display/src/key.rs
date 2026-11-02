//! key.rs
//! ======
//! Gegenstueck zu `pygame.key`: Tastenzustand, Zusatztasten, Namen.
//!
//! ## Was an echtem pygame gemessen wurde
//!
//! | Frage | Antwort |
//! |---|---|
//! | `len(get_pressed())` | **512**, nicht 323 und nicht 1073741906 |
//! | `get_pressed()[0]` | `False` - Platz 0 ist `SDL_SCANCODE_UNKNOWN` |
//! | `get_pressed()[K_UP]` | geht trotzdem, obwohl `K_UP` 1073741906 ist |
//! | `name(K_a)` | `"a"` |
//! | `name(K_a, use_compat=False)` | `"A"` - dieselbe Taste, andere Schreibweise |
//! | `name(K_LSHIFT)` | `"left shift"` (pygame) bzw. `"Left Shift"` (SDL) |
//! | `key_code("Up")` und `key_code("up")` | beide 1073741906 |
//! | `get_repeat()` ohne `set_repeat` | `(0, 0)` |
//! | `set_mods(KMOD_SHIFT)` danach `get_mods()` | 3 - `KMOD_SHIFT` ist links **und** rechts |
//! | `type(get_pressed())` | `ScancodeWrapper` - eine Unterklasse von `tuple` |
//! | `get_pressed()[-1]`, `[99999]` | `False` - **kein** `IndexError` |
//! | `for b in get_pressed()` | `TypeError: Iterating over key states is not supported` |
//! | `len(get_just_pressed())` | 512, ebenfalls ein `ScancodeWrapper` |
//!
//! Der wichtigste Fund: **`get_pressed()` wird mit dem Tastenplatz
//! indiziert, nicht mit dem Tastencode.** Die Tabelle ist 512 Eintraege
//! lang, `K_UP` ist 1073741906 - das passt nicht hinein. pygame rechnet
//! im Zugriff still um. [`ScancodeWrapper`] tut dasselbe, und zwar
//! sichtbar.
//!
//! Und der Fund, der erklaert, warum das ein eigener Typ ist und kein
//! `[bool; 512]`: pygames `ScancodeWrapper` **verbietet das
//! Durchlaufen**. `for b in get_pressed()` wirft `TypeError`, weil die
//! Stellen der Tabelle Tastenplaetze sind und keine Tastencodes - wer
//! sie durchzaehlt, zaehlt das Falsche. Dieser Typ bildet das nach: er
//! hat kein `IntoIterator`.

use rustgame_core::Rect;

use crate::konstanten::*;
use crate::zustand::mit_zustand;

/// Welche Tasten unten sind - pygames `ScancodeWrapper`.
///
/// 512 Plaetze lang, gemessen an echtem pygame. Indiziert wird mit dem
/// **Tastenplatz**; der Zugriff mit `K_*` geht trotzdem, weil
/// [`scancode_von`] dazwischen umrechnet - genauso, wie pygames
/// `ScancodeWrapper` es tut.
///
/// Drei Aufrufe liefern so eine Tabelle: [`get_pressed`],
/// [`get_just_pressed`] und [`get_just_released`].
///
/// ```
/// # use rustgame_display::{display, key, konstanten::*};
/// # display::set_treiber(display::Treiber::Dummy);
/// # display::init().unwrap();
/// # display::set_mode((8, 8), 0).unwrap();
/// let tasten = key::get_pressed();
/// assert_eq!(tasten.len(), 512);
/// assert!(!tasten[K_UP]);   // geht, obwohl K_UP weit ueber 512 liegt
/// ```
///
/// **Kein `IntoIterator`, und das mit Absicht.** In pygame wirft
/// `for b in get_pressed()` ein `TypeError: Iterating over key states is
/// not supported` - die Stellen sind Tastenplaetze, kein Programm will
/// sie der Reihe nach. Wer es doch braucht, nimmt [`Self::roh`] mit
/// einer eigenen Schleife oder [`Self::irgendeine`].
#[derive(Clone)]
pub struct ScancodeWrapper(Box<[bool; NUM_SCANCODES]>);

/// Der alte Name von [`ScancodeWrapper`].
///
/// pygame nennt den Typ `ScancodeWrapper`; diese Kiste hiess ihn erst
/// `Gedrueckt`. Der Name bleibt als Abkuerzung stehen, damit nichts
/// bricht, was ihn schon benutzt.
pub type Gedrueckt = ScancodeWrapper;

impl ScancodeWrapper {
    /// Wie viele Plaetze die Tabelle hat - immer [`NUM_SCANCODES`].
    pub fn len(&self) -> usize {
        NUM_SCANCODES
    }

    /// Nie leer; steht nur da, weil clippy sonst meckert.
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Zugriff ueber den Tastenplatz, ohne Umrechnung.
    pub fn roh(&self, scancode: Scancode) -> bool {
        self.0.get(scancode as usize).copied().unwrap_or(false)
    }

    /// Ist irgendeine Taste unten?
    pub fn irgendeine(&self) -> bool {
        self.0.iter().any(|b| *b)
    }
}

impl std::ops::Index<Key> for ScancodeWrapper {
    type Output = bool;

    /// Zugriff mit einem Tastencode (`K_*`).
    ///
    /// Rechnet auf den Tastenplatz um - das ist die Umrechnung, die
    /// pygames `ScancodeWrapper` im Verborgenen macht.
    ///
    /// Ein Code, den es nicht gibt, ergibt `false` statt einer Panik -
    /// gemessen: `get_pressed()[99999]` und `get_pressed()[-1]` sind in
    /// pygame beide `False`, kein `IndexError`.
    fn index(&self, key: Key) -> &bool {
        let s = scancode_von(key) as usize;
        self.0.get(s).unwrap_or(&false)
    }
}

impl std::fmt::Debug for ScancodeWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let unten: Vec<usize> = self
            .0
            .iter()
            .enumerate()
            .filter(|(_, b)| **b)
            .map(|(i, _)| i)
            .collect();
        write!(f, "Gedrueckt({} Plaetze, unten: {unten:?})", NUM_SCANCODES)
    }
}

/// Der Tastenplatz zu einem Tastencode.
///
/// Zwei Wege, beide gemessen:
///
/// * Tasten ohne Zeichen tragen das oberste Bit ([`SCANCODE_MASK`],
///   `1 << 30`) und enthalten ihren Platz selbst - `K_UP` ist 1073741906,
///   `KSCAN_UP` ist 82, und 1073741906 - 82 ist genau `1 << 30`.
/// * Zeichentasten wie `K_a` (97) tun das nicht; fuer die steht die
///   Zuordnung in [`KEY_TO_SCANCODE`], ausgelesen aus den
///   `KSCAN_*`-Gegenstuecken in echtem pygame.
pub fn scancode_von(key: Key) -> Scancode {
    if key & SCANCODE_MASK != 0 {
        return key & !SCANCODE_MASK;
    }
    match KEY_TO_SCANCODE.binary_search_by_key(&key, |(k, _)| *k) {
        Ok(i) => KEY_TO_SCANCODE[i].1,
        Err(_) => 0,
    }
}

/// Welche Tasten unten sind - `pygame.key.get_pressed()`.
///
/// Holt vorher neue Ereignisse vom Fenster, sonst waere die Antwort von
/// gestern.
pub fn get_pressed() -> ScancodeWrapper {
    mit_zustand(|z| {
        crate::event::pumpen(z);
        ScancodeWrapper(z.eingabe.tasten.clone())
    })
}

/// Welche Tasten in **diesem** Durchlauf heruntergegangen sind -
/// `pygame.key.get_just_pressed()`.
///
/// Dieselbe 512er-Tabelle wie [`get_pressed`], aber `true` nur in dem
/// Durchlauf, in dem die Taste heruntergeht. Gemessen an pygame: der
/// Vermerk entsteht, "when new events are processed, e.g. in
/// `pygame.event.get()` or `pygame.event.pump()`" - hier also in
/// `event::durchlauf`.
///
/// **Dieser Aufruf pumpt nicht**, und das ist der Punkt. Er liest den
/// Vermerk des letzten Durchlaufs; zu pumpen hiesse, ihn vorher zu
/// loeschen. pygames `get_just_pressed` pumpt ebenfalls nicht.
///
/// Damit ergibt sich eine Reihenfolge, an die man sich halten muss:
///
/// ```
/// # use rustgame_display::{display, event, key, backend::dummy, konstanten::*};
/// # display::set_treiber(display::Treiber::Dummy);
/// # display::init().unwrap();
/// # display::set_mode((8, 8), 0).unwrap();
/// # event::clear(None);
/// dummy::taste_druecken(K_a, "a");
/// event::get();                        // ein Durchlauf
/// assert!(key::get_just_pressed()[K_a]);
/// event::get();                        // der naechste Durchlauf
/// assert!(!key::get_just_pressed()[K_a], "nur einen Durchlauf lang");
/// assert!(key::get_pressed()[K_a], "unten ist sie trotzdem noch");
/// ```
///
/// **Vorsicht:** [`get_pressed`] pumpt in dieser Kiste (pygames tut das
/// nicht). Zwischen `event::get()` und `get_just_pressed()` geht das
/// gut - `get_pressed` selbst loescht die Kantentabellen nicht, das tut
/// nur ein Blick in die Schlange.
///
/// Gemessen und wichtig: eine Taste kann hier `true` sein, obwohl
/// [`get_pressed`] sie als oben meldet - naemlich wenn sie im selben
/// Durchlauf gedrueckt **und** losgelassen wurde. Mehrfaches Druecken
/// im selben Durchlauf laesst sich damit nicht unterscheiden; wer das
/// braucht, liest `KEYDOWN` aus der Schlange.
pub fn get_just_pressed() -> ScancodeWrapper {
    mit_zustand(|z| ScancodeWrapper(z.eingabe.tasten_neu.clone()))
}

/// Welche Tasten in diesem Durchlauf heraufgekommen sind -
/// `pygame.key.get_just_released()`.
///
/// Das Gegenstueck zu [`get_just_pressed`], mit denselben Regeln.
pub fn get_just_released() -> ScancodeWrapper {
    mit_zustand(|z| ScancodeWrapper(z.eingabe.tasten_weg.clone()))
}

/// Welche Zusatztasten gehalten werden - `pygame.key.get_mods()`.
///
/// Das Ergebnis ist eine Oder-Verknuepfung von `KMOD_*`. Achtung:
/// gemessen ist `KMOD_SHIFT` gleich 3, also links **und** rechts - eine
/// Pruefung mit `==` geht daneben, `& KMOD_SHIFT != 0` ist richtig.
pub fn get_mods() -> KeyMod {
    mit_zustand(|z| {
        crate::event::pumpen(z);
        z.eingabe.mods
    })
}

/// Setzt die Zusatztasten - `pygame.key.set_mods()`.
pub fn set_mods(mods: KeyMod) {
    mit_zustand(|z| z.eingabe.mods = mods)
}

/// Der Name einer Taste - `pygame.key.name()`.
///
/// Die pygame-eigene Schreibweise: klein, mit Leerzeichen -
/// `"a"`, `"space"`, `"left shift"`. Unbekanntes ergibt `""`, wie bei
/// pygame.
pub fn name(key: Key) -> &'static str {
    match KEY_NAMES.binary_search_by_key(&key, |(k, _, _)| *k) {
        Ok(i) => KEY_NAMES[i].1,
        Err(_) => "",
    }
}

/// Der Name einer Taste in SDL-Schreibweise - `pygame.key.name(k, use_compat=False)`.
///
/// Gross geschrieben: `"A"`, `"Space"`, `"Left Shift"`. Gemessen an
/// echtem pygame, nicht aus der pygame-Schreibweise hochgerechnet - denn
/// aus `"left shift"` wuerde man `"Left shift"` machen, und das waere
/// falsch.
pub fn name_sdl(key: Key) -> &'static str {
    match KEY_NAMES.binary_search_by_key(&key, |(k, _, _)| *k) {
        Ok(i) => KEY_NAMES[i].2,
        Err(_) => "",
    }
}

/// Der Tastencode zu einem Namen - `pygame.key.key_code()`.
///
/// `None`, wenn der Name unbekannt ist; pygame wirft an der Stelle einen
/// Fehler.
///
/// Gemessen: bei Namen aus mehreren Zeichen ist die Gross- und
/// Kleinschreibung egal (`"Up"` und `"up"` ergeben beide `K_UP`), bei
/// einzelnen Zeichen nicht (`"a"` ist `K_a`, `"A"` etwas anderes). Die
/// Tabelle traegt deshalb jede Schreibweise, die echtes pygame aufloest.
pub fn key_code(name: &str) -> Option<Key> {
    if let Ok(i) = KEY_CODES.binary_search_by_key(&name, |(n, _)| *n) {
        return Some(KEY_CODES[i].1);
    }
    // Was in der Tabelle fehlt, kann immer noch ein einzelnes Zeichen
    // sein - pygame bildet die auf ihren Unicode-Wert ab.
    let mut zeichen = name.chars();
    match (zeichen.next(), zeichen.next()) {
        (Some(c), None) if (c as u32) < 0x110000 => Some(c as u32),
        _ => None,
    }
}

/// Hat das Fenster den Tastaturfokus? - `pygame.key.get_focused()`.
pub fn get_focused() -> crate::Ergebnis<bool> {
    // **Vor `display.init()` ist das ein Fehler**, nicht `false`.
    // Gemessen: `pygame.key.get_focused()` vor `pygame.init()` gibt
    // `pygame.error: video system not initialized`; danach `False`, und
    // erst nach `set_mode` `True`. `key_test.py` prueft die Ausnahme
    // ausdruecklich.
    mit_zustand(|z| {
        if !z.init {
            return Err(crate::Fehler::NichtInitialisiert);
        }
        crate::event::pumpen(z);
        Ok(z.eingabe.tast_fokus)
    })
}

/// Stellt die Tastenwiederholung ein - `pygame.key.set_repeat()`.
///
/// `verzoegerung` ist die Zeit bis zur ersten Wiederholung, `abstand` die
/// zwischen den weiteren, beides in Millisekunden. `set_repeat(0, 0)`
/// schaltet sie ab - gemessen ist das auch der Ausgangszustand.
pub fn set_repeat(verzoegerung: i32, abstand: i32) {
    mit_zustand(|z| {
        z.eingabe.wiederholung = (verzoegerung, abstand);
        if let Some(b) = z.backend.as_mut() {
            b.set_repeat(verzoegerung, abstand);
        }
    })
}

/// Wie die Tastenwiederholung eingestellt ist - `pygame.key.get_repeat()`.
///
/// Gemessen: ohne vorheriges [`set_repeat`] kommt `(0, 0)`.
pub fn get_repeat() -> (i32, i32) {
    mit_zustand(|z| z.eingabe.wiederholung)
}

/// Schaltet Texteingaben ein - `pygame.key.start_text_input()`.
///
/// Danach meldet das Fenster `TEXTINPUT` fuer getippten Text. Bei pygame
/// ist das von Anfang an an; hier ebenso.
pub fn start_text_input() {
    mit_zustand(|z| z.eingabe.text_eingabe = true)
}

/// Schaltet Texteingaben ab - `pygame.key.stop_text_input()`.
pub fn stop_text_input() {
    mit_zustand(|z| z.eingabe.text_eingabe = false)
}

/// Sagt, wo die Kandidatenliste einer Eingabemethode hingehoert -
/// `pygame.key.set_text_input_rect()`.
///
/// Wer Japanisch oder Chinesisch tippt, waehlt aus einer Liste, die das
/// Betriebssystem einblendet. Dieses Rechteck sagt dem System, wo im
/// Fenster gerade geschrieben wird, damit die Liste nicht darueber liegt.
///
/// **Hier hoert niemand zu.** Weder minifb noch das fensterlose Stueck
/// haben eine Eingabemethode: minifb meldet fertige Zeichen ueber einen
/// Rueckruf und kennt gar keinen Kandidatenschritt. Das Rechteck wird
/// darum nur gemerkt (und ist ueber [`get_text_input_rect`] wieder zu
/// haben) - kein Fehler, denn gemessen gibt pygame unter
/// `SDL_VIDEODRIVER=dummy` ebenfalls klaglos `None` zurueck, sogar vor
/// `display.init()`.
pub fn set_text_input_rect(rect: Rect) {
    mit_zustand(|z| z.eingabe.text_eingabe_rect = Some(rect))
}

/// Das zuletzt mit [`set_text_input_rect`] gesetzte Rechteck.
///
/// **Das gibt es in pygame nicht** - dort ist `set_text_input_rect` ein
/// reiner Setzer. Es steht hier, damit das Gemerkte nicht in einem
/// schwarzen Loch verschwindet: so laesst sich wenigstens pruefen, dass
/// der Aufruf angekommen ist.
pub fn get_text_input_rect() -> Option<Rect> {
    mit_zustand(|z| z.eingabe.text_eingabe_rect)
}
