//! fenster.rs
//! ==========
//! Gegenstueck zu `pygame.window`: das Fenster als **Ding** statt als
//! Modulzustand.
//!
//! `pygame.display` fuehrt genau ein Fenster und redet darueber in freien
//! Funktionen. `pygame.window.Window` dreht das um: jedes Fenster ist ein
//! Objekt mit Eigenschaften, und es darf mehrere geben. Beide Wege fuehren
//! zum selben Fenster, wenn man es darauf anlegt - siehe
//! [`Fenster::vom_anzeigemodul`].
//!
//! ## Was an echtem pygame-ce 2.5.7 gemessen wurde
//!
//! Alles unter `SDL_VIDEODRIVER=dummy`, jede Messung in einem **eigenen
//! Prozess** (siehe den Absatz ueber den Absturz ganz unten).
//!
//! | Frage | Antwort |
//! |---|---|
//! | `Window()` ohne Argumente | Titel `'pygame window'`, Groesse `(640, 480)` |
//! | `Window()` vor `pygame.init()` | geht - und faehrt die Anzeige nebenbei hoch |
//! | `Window('T', (64, 48)).position` | `(480, 360)` - mittig auf `(1024, 768)` |
//! | `Window('T', (2000, 2000)).position` | `(-488, -616)` - dieselbe Rechnung, auch wenn es nicht passt |
//! | `position=WINDOWPOS_CENTERED` / `..._UNDEFINED` | beide mittig |
//! | Nummern zweier Fenster | `1`, `2` - fortlaufend, und nach `destroy()` **nicht** wiederverwendet |
//! | `focused` bei drei Fenstern | nur das **zuletzt** angelegte |
//! | `Window(.., hidden=True)` | `focused` ist `False`, `show()` macht es `True` |
//! | `Window(.., fullscreen=True).size` | `(1024, 768)` |
//! | `Window(.., opengl=True)` | Fehler - kein GL im Dummy-Treiber |
//! | `Window('T', (0, 48))` | `ValueError: width or height should not be less than or equal to zero.` |
//! | `w.size = (0, 48)` | derselbe Text **ohne** den Punkt am Ende |
//! | `Window('T', (64, 48), quatsch=True)` | `TypeError: __init__ got an unexpected flag 'quatsch'` |
//! | `repr(w)` | `<Window(title='T', id=1)>`, nach `destroy()` `<Window(Destroyed)>` |
//!
//! ### Die drei Merkmale, die sich nicht aendern lassen
//!
//! `borderless`, `resizable` und `always_on_top` nehmen im Konstruktor
//! jeden Wert an und behalten ihn - der Setzer **wirkt nicht**, in keine
//! Richtung, und wirft dabei auch keinen Fehler. `w.borderless = True`
//! auf einem Fenster ohne Rahmenwunsch gibt hinterher weiter `False`.
//! Das ist kein Versehen von pygame, sondern der fehlende
//! Fensterverwalter unter dem Dummy-Treiber.
//!
//! Hier gilt dasselbe, solange kein echtes Fenster dahintersteht
//! ([`crate::backend::Backend::hat_fenster`]). Mit minifb gehen alle
//! drei wirklich: Rahmen und Groessenaenderbarkeit ueber einen Umbau des
//! Fensters, `always_on_top` ueber `minifb::Window::topmost`.
//!
//! ### Die Flaeche
//!
//! | Frage | Antwort |
//! |---|---|
//! | `w.get_surface()` zweimal | **dasselbe** Objekt |
//! | `w.size = ..` danach `w.get_surface()` | immer noch dasselbe, nur umgebaut |
//! | Flaggen der Flaeche | `16777216` ([`PREALLOC`]), auch im Vollbild |
//! | `w.get_surface().get_alpha()` | `None` |
//! | `w.flip()` **ohne** vorheriges `get_surface()` | Fehler `the Window has no surface associated with it, did you forget to call Window.get_surface()` |
//!
//! Das ist genau das Bild, das [`crate::Screen`] ohnehin zeichnet - ein
//! geteilter Behaelter, der umgebaut und nicht ersetzt wird.
//!
//! ### Griff: nur wer den Fokus hat, greift
//!
//! `grab_mouse` und `grab_keyboard` sind der **Wunsch** und bleiben
//! stehen; `mouse_grabbed` und `keyboard_grabbed` sagen, was **gilt**.
//! Der Unterschied ist der Eingabefokus, und das ist der Punkt, den man
//! ohne Messung nicht raet - die ganze Tabelle steht bei `haelt_griff`:
//!
//! * Ein einzelnes Fenster greift, sobald es den Wunsch hat.
//! * Bei zwei Fenstern greift nur das **im Fokus**. `a.grab_mouse = True`
//!   bei fokussiertem `b` laesst `a.mouse_grabbed` falsch - und
//!   [`get_grabbed_window`] gibt `None`, nicht `a`.
//! * Ein neues Fenster nimmt dem alten den Fokus und damit den Griff,
//!   ohne selbst zu greifen.
//! * `hide()` nimmt den Griff, `show()` gibt ihn zurueck - der Wunsch
//!   bleibt derweil stehen.
//! * `destroy()` gibt den Griff frei.
//! * `pygame.event.get_grab()` bleibt davon **unberuehrt** - das ist ein
//!   anderer Schalter.
//!
//! Hier stand vorher "wer zuletzt greift, nimmt dem vorigen den Griff
//! weg". Das war aus einer einzigen Messung gewonnen, in der beide
//! Fenster hintereinander griffen, und stimmte in keinem anderen Fall.
//!
//! ### `pygame.window.get_grabbed_window()` bringt pygame zum Absturz
//!
//! **Gemessen und wiederholbar**: der Aufruf beendet den Prozess hart
//! (Windows-Abbruchcode `0xC0000409`), wenn das Video-Teilsystem
//! **nicht** laeuft - also vor `pygame.init()`, nach
//! `pygame.display.quit()` und nach `pygame.quit()`. Mit laufendem
//! Video-Teilsystem ist er harmlos und liefert das gegriffene Fenster
//! oder `None`.
//!
//! Der Absturz haengt also **nicht** am Dummy-Treiber, wie zuerst
//! vermutet, sondern an der fehlenden Pruefung auf das Teilsystem: pygame
//! greift dort auf eine Fensterliste zu, die es noch nicht gibt.
//!
//! [`get_grabbed_window`] hier stuerzt in keinem Fall ab - ohne
//! [`crate::display::init`] gibt es schlicht kein gegriffenes Fenster,
//! also `None`.
//!
//! ## Was es hier ehrlich nicht gibt
//!
//! | Aufruf | ohne Fenster | mit minifb |
//! |---|---|---|
//! | [`Fenster::set_deckkraft`] | nein - **gemessen auch bei pygame nein** | nein, minifb kennt keine Deckkraft |
//! | [`Fenster::blinken`] | nein - **gemessen auch bei pygame nein** | nein |
//! | [`Fenster::modal_fuer`] | nein - **gemessen auch bei pygame nein** | nein |
//! | [`Fenster::zustand_setzen`] | ja, tut nichts - gemessen | nein, minifb hat kein `minimize()` |
//! | [`Fenster::zeigen`] / [`Fenster::verstecken`] | ja - gemessen | nein, minifb kann ein Fenster nicht verstecken |
//! | [`Fenster::nach_vorn`] ohne `nur_eingabe` | ja - gemessen | nein |
//! | [`Fenster::nach_vorn`] mit `nur_eingabe` | nein - **gemessen auch bei pygame nein** | nein |
//! | [`Fenster::set_vollbild`] | ja - gemessen | nein, minifb has no fullscreen mode |
//! | [`Fenster::set_sinnbild`] | ja - gemessen | nein, siehe [`crate::backend::MinifbBackend`] |
//! | `utility` | gemeldet wie bei pygame | nein - minifb kennt kein Hilfsfenster |
//! | `opengl` / `vulkan` | nein - **gemessen auch bei pygame nein** | nein |
//!
//! Die Zeilen, in denen **beide** Spalten nein sagen und pygame es unter
//! dem Dummy-Treiber ebenfalls verweigert, sind keine Luecke dieser
//! Kiste, sondern eine Eigenschaft der Lage. Die anderen sind eine.

use rustgame_core::{Rect, Surface};

use std::sync::Arc;

use parking_lot::RwLock;

use crate::GELEIHT;
use crate::backend::{Backend, Fensterzustand};
use crate::event::{Daten, Event};
use crate::konstanten::*;
use crate::zustand::{mit_zustand, Zustand};
use crate::{Ergebnis, Fehler, Screen};

// ---------------------------------------------------------------------------
//  Wo ein Fenster stehen soll
// ---------------------------------------------------------------------------

/// Wo ein neues Fenster stehen soll - pygames `position`.
///
/// Gemessen: [`Stelle::Mittig`] und [`Stelle::Offen`] liefern unter dem
/// Dummy-Treiber **dasselbe** - beide stellen das Fenster mittig auf die
/// Arbeitsflaeche. Sie bleiben hier trotzdem getrennt, weil sie in
/// pygame zwei verschiedene Zahlen sind
/// ([`WINDOWPOS_CENTERED`], [`WINDOWPOS_UNDEFINED`]) und ein
/// Fensterverwalter sie sehr wohl unterscheiden darf.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Stelle {
    /// Mittig auf der Arbeitsflaeche - `WINDOWPOS_CENTERED`.
    Mittig,
    /// Dem System ueberlassen - `WINDOWPOS_UNDEFINED`. Die Vorgabe.
    #[default]
    Offen,
    /// Genau hier, in Bildschirmkoordinaten.
    Genau((i32, i32)),
}

impl Stelle {
    /// Uebersetzt pygames Zahlenwerte.
    ///
    /// `WINDOWPOS_CENTERED` und `WINDOWPOS_UNDEFINED` kommen dort als
    /// **einzelne** Zahl statt als Paar; wer ein Paar hat, nimmt
    /// [`Stelle::Genau`].
    pub fn aus_zahl(wert: i64) -> Stelle {
        if wert == WINDOWPOS_CENTERED as i64 {
            Stelle::Mittig
        } else if wert == WINDOWPOS_UNDEFINED as i64 {
            Stelle::Offen
        } else {
            Stelle::Genau((wert as i32, wert as i32))
        }
    }
}

// ---------------------------------------------------------------------------
//  Was man sich beim Anlegen wuenschen kann
// ---------------------------------------------------------------------------

/// Die Wuensche fuer ein neues Fenster - pygames Konstruktorwoerter.
///
/// Alle zwanzig, die `help(pygame.Window)` nennt, stehen hier; welche
/// davon wirklich etwas bewirken, sagt die Tabelle im Modulkopf. Wer nur
/// Titel und Groesse braucht, nimmt [`Wunsch::neu`].
#[derive(Debug, Clone, PartialEq)]
pub struct Wunsch {
    /// `title` - Vorgabe `"pygame window"`, gemessen.
    pub titel: String,
    /// `size` - Vorgabe `(640, 480)`, gemessen.
    pub groesse: (u32, u32),
    /// `position`.
    pub stelle: Stelle,
    /// `fullscreen` - das Fenster nimmt die Arbeitsflaeche ein.
    pub vollbild: bool,
    /// `fullscreen_desktop`. Gemessen: unter dem Dummy-Treiber
    /// dasselbe Ergebnis wie `fullscreen`, `(1024, 768)`.
    pub vollbild_arbeitsflaeche: bool,
    /// `opengl`. Gibt es hier nicht - siehe [`Fenster::neu`].
    pub opengl: bool,
    /// `vulkan`. Gibt es hier ebenso wenig.
    pub vulkan: bool,
    /// `hidden` - angelegt, aber nicht gezeigt.
    pub versteckt: bool,
    /// `borderless` - ohne Rahmen ([`NOFRAME`]).
    pub rahmenlos: bool,
    /// `resizable` - der Benutzer darf ziehen ([`RESIZABLE`]).
    pub groessenaenderbar: bool,
    /// `minimized`.
    pub abgelegt: bool,
    /// `maximized`.
    pub aufgezogen: bool,
    /// `mouse_grabbed` - setzt den **Wunsch** nach dem Mausgriff.
    ///
    /// Gemessen: `Window(.., mouse_grabbed=True).grab_mouse` ist danach
    /// wahr. Ob er auch **gilt**, haengt am Fokus - siehe `haelt_griff`.
    pub griff_maus: bool,
    /// `keyboard_grabbed` - dasselbe fuer die Tastatur.
    pub griff_tastatur: bool,
    /// `input_focus`. Gemessen: unter dem Dummy-Treiber ohne Wirkung -
    /// das neue Fenster hat den Fokus ohnehin.
    pub eingabefokus: bool,
    /// `mouse_focus`. Ebenso ohne Wirkung.
    pub mausfokus: bool,
    /// `allow_high_dpi`. Diese Kiste rechnet in Pixeln ohne Skalierung.
    pub hohe_aufloesung: bool,
    /// `mouse_capture`.
    pub maus_fangen: bool,
    /// `always_on_top`.
    ///
    /// Gemessen: das Wort im Konstruktor haelt, auch ohne Fenster - nur
    /// [`Fenster::set_obenauf`] bleibt dann wirkungslos.
    pub obenauf: bool,
    /// `utility` - ein Hilfsfenster ohne Leisteneintrag.
    ///
    /// Wird **angenommen**, wie bei pygame; ob es hinterher gemeldet
    /// wird, entscheidet [`Fenster::hilfsfenster`].
    pub hilfsfenster: bool,
}

impl Default for Wunsch {
    /// Die gemessenen Vorgaben von `pygame.Window()`.
    fn default() -> Wunsch {
        Wunsch {
            titel: "pygame window".into(),
            groesse: (640, 480),
            stelle: Stelle::Offen,
            vollbild: false,
            vollbild_arbeitsflaeche: false,
            opengl: false,
            vulkan: false,
            versteckt: false,
            rahmenlos: false,
            groessenaenderbar: false,
            abgelegt: false,
            aufgezogen: false,
            griff_maus: false,
            griff_tastatur: false,
            eingabefokus: false,
            mausfokus: false,
            hohe_aufloesung: false,
            maus_fangen: false,
            obenauf: false,
            hilfsfenster: false,
        }
    }
}

impl Wunsch {
    /// Titel und Groesse, alles andere auf Vorgabe.
    pub fn neu(titel: &str, groesse: (u32, u32)) -> Wunsch {
        Wunsch {
            titel: titel.into(),
            groesse,
            ..Wunsch::default()
        }
    }
}

// ---------------------------------------------------------------------------
//  Die Ablage
// ---------------------------------------------------------------------------

/// Woran ein Eintrag haengt.
enum Stueck {
    /// Das Fenster des Anzeigemoduls: Hintergrundstueck und Flaeche
    /// liegen in [`Zustand::backend`] und [`Zustand::screen`], damit
    /// `set_mode` und `Window` **dieselbe** Flaeche sehen.
    Anzeige,
    /// Ein eigenes Fenster mit eigenem Hintergrundstueck.
    Eigen {
        /// Das Fenster selbst.
        backend: Box<dyn Backend>,
        /// Die Flaeche, sobald jemand [`Fenster::flaeche`] gerufen hat.
        ///
        /// Gemessen: `Window.flip()` **vor** dem ersten `get_surface()`
        /// ist ein Fehler - pygame legt die Flaeche erst dort an.
        screen: Option<Screen>,
    },
}

/// Ein Fenster, so wie diese Kiste es fuehrt.
struct Eintrag {
    id: u32,
    titel: String,
    /// Die Groesse, die jetzt gilt.
    groesse: (u32, u32),
    /// Die Groesse, auf die [`Fenster::set_vollbild`] mit `false`
    /// zurueckstellt.
    ///
    /// Gemessen: `w.size = (200, 150)` **im Vollbild** aendert `w.size`
    /// nicht (das bleibt `(1024, 768)`), wird aber gemerkt - nach
    /// `set_windowed()` steht das Fenster auf `(200, 150)`.
    fenstergroesse: (u32, u32),
    position: (i32, i32),
    /// `minimum_size` / `maximum_size`. `(0, 0)` heisst "keine Grenze".
    mindest: (u32, u32),
    hoechst: (u32, u32),
    /// `mouse_rect`.
    maus_rechteck: Option<Rect>,
    /// [`NOFRAME`], [`RESIZABLE`], [`FULLSCREEN`], [`HIDDEN`].
    flags: u32,
    obenauf: bool,
    /// Ist das ein Hilfsfenster? - `utility`.
    ///
    /// Beim Anlegen einmal ausgerechnet, nicht bei jedem Lesen: es ist
    /// der Wunsch **und** die Frage, ob ein Stueck ihm widersprechen
    /// kann. Siehe [`Fenster::hilfsfenster`].
    hilfsfenster: bool,
    fokus: bool,
    /// Der **Wunsch** nach dem Griff, nicht sein Erfolg - siehe Modulkopf.
    will_maus: bool,
    will_tastatur: bool,
    /// Hat das Stueck den Griff wirklich genommen?
    ///
    /// Gemessen genuegt der Wunsch nicht: gegriffen ist nur, wer auch den
    /// Eingabefokus hat (siehe Modulkopf). Und das Stueck muss es
    /// koennen - minifb kann es nie, [`crate::backend::Backend::set_grab`]
    /// sagt dort `false`. Hier steht dessen Antwort.
    griff_wirkt: bool,
    stueck: Stueck,
}

/// Alle Fenster, die es gerade gibt.
pub struct Ablage {
    eintraege: Vec<Eintrag>,
    /// Die naechste Nummer. Gemessen: sie zaehlt hoch und wird nach
    /// `destroy()` **nicht** wiederverwendet.
    naechste_id: u32,
}

impl Ablage {
    /// Leer, mit der ersten Nummer auf 1 - gemessen: pygame faengt dort an.
    pub(crate) fn neu() -> Ablage {
        Ablage {
            eintraege: Vec::new(),
            naechste_id: 1,
        }
    }

    /// Raeumt alles ab - `display.quit()`.
    ///
    /// Gemessen: nach `pygame.display.quit()` ist jedes `Window` hinueber.
    /// pygame liest dort allerdings **freigegebenen Speicher** weiter -
    /// `w.size` gibt Zufallszahlen wie `(-1613893632, 1)`, `w.title` die
    /// leere Zeichenkette. Das wird hier nicht nachgebaut: ein Fenster,
    /// das es nicht mehr gibt, meldet [`Fehler::Pygame`] mit
    /// `Invalid window` - derselbe Text, den pygame fuer
    /// `w.get_surface()` nach `destroy()` selbst verwendet.
    pub(crate) fn leeren(&mut self) {
        self.eintraege.clear();
    }

    fn stelle_von(&self, id: u32) -> Ergebnis<usize> {
        self.eintraege
            .iter()
            .position(|e| e.id == id)
            .ok_or(Fehler::Pygame("Invalid window"))
    }
}

// ---------------------------------------------------------------------------
//  Der Griff auf den Zustand
// ---------------------------------------------------------------------------

/// Das Hintergrundstueck eines Eintrags.
///
/// Beim Anzeigefenster ist das [`Zustand::backend`] - so sehen
/// `display::flip` und [`Fenster::flip`] dasselbe Fenster.
///
/// Die beiden Felder werden **vor** dem `match` getrennt ausgeliehen.
/// Das ist kein Schoenheitsfehler: schreibt man statt dessen in einem
/// Arm `z.backend` und im anderen `z.fenster`, leiht jeder Arm `*z`
/// erneut aus, und der Ausleihpruefer lehnt es ab. So sieht er zwei
/// Wege, die sich nicht ueberschneiden.
///
/// Herausgereicht wird die **Schachtel**, nicht das Trait-Objekt darin.
/// Das ist kein Zufall: `&mut` ist ueber seinen Inhalt invariant, und
/// `Box<dyn Backend>` heisst ausgeschrieben `Box<dyn Backend +
/// 'static>` - ein `&mut dyn Backend` mit der kuerzeren Lebensdauer von
/// `z` liesse sich darauf nicht abbilden. Ueber die Schachtel kommt man
/// mit demselben Punkt an dieselben Aufrufe.
fn stueck_mut(z: &mut Zustand, i: usize) -> Option<&mut Box<dyn Backend>> {
    let anzeigestueck = &mut z.backend;
    let eintraege = &mut z.fenster.eintraege;
    match &mut eintraege[i].stueck {
        Stueck::Eigen { backend, .. } => Some(backend),
        Stueck::Anzeige => anzeigestueck.as_mut(),
    }
}

/// Die Flaeche eines Eintrags, ohne sie anzulegen.
fn flaeche_von(z: &Zustand, i: usize) -> Option<Screen> {
    match &z.fenster.eintraege[i].stueck {
        Stueck::Anzeige => z.screen.clone(),
        Stueck::Eigen { screen, .. } => screen.clone(),
    }
}

// ---------------------------------------------------------------------------
//  Das Fenster
// ---------------------------------------------------------------------------

/// Ein Fenster - `pygame.window.Window`.
///
/// Das hier ist ein **Griff**, kein Fenster: es traegt nur die Nummer und
/// schlaegt bei jedem Aufruf in der Ablage nach. Deshalb ist es [`Copy`],
/// und deshalb melden alle Aufrufe nach [`Fenster::zerstoeren`]
/// `Invalid window` statt auf etwas Freigegebenes zu zeigen - anders als
/// echtes pygame, siehe `Ablage::leeren`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fenster {
    id: u32,
}

impl Fenster {
    // -- Anlegen --------------------------------------------------------

    /// Ein Griff, hinter dem **kein** Fenster steht.
    ///
    /// Gebraucht wird er in der Python-Huelle: `pygame.Window` laesst sich
    /// ableiten, und eine Unterklasse muss sich mit `cls.__new__(cls)`
    /// anlegen lassen, **ohne** dabei ein Fenster aufzumachen. Erst
    /// `__init__` macht eines auf.
    ///
    /// Die Nummer 0 vergibt [`Ablage`] nie, also melden alle Aufrufe
    /// darauf `Invalid window` - dieselbe Antwort wie nach
    /// [`Fenster::zerstoeren`], und das ist genau die gewollte.
    pub fn ungueltig() -> Fenster {
        Fenster { id: 0 }
    }

    /// Macht ein Fenster auf - `pygame.window.Window(...)`.
    ///
    /// Gemessen und hier nachgebaut:
    ///
    /// * Eine Breite oder Hoehe kleiner-gleich null ist ein Fehler mit
    ///   dem Wortlaut `width or height should not be less than or equal
    ///   to zero.` - **mit** dem Punkt am Ende; der Setzer
    ///   [`Fenster::set_groesse`] sagt denselben Satz **ohne** ihn.
    /// * Im Vollbild gilt die Arbeitsflaeche, egal was dasteht.
    /// * Das neue Fenster bekommt den Fokus und nimmt ihn allen anderen
    ///   weg - ausser es ist versteckt.
    /// * Es legt dieselben fuenf Ereignisse ab wie
    ///   [`crate::display::set_mode`].
    ///
    /// `opengl` und `vulkan` gibt es hier nicht. Echtes pygame wirft
    /// unter dem Dummy-Treiber woertlich `OpenGL support is either not
    /// configured in SDL or not available in current SDL video driver
    /// (dummy) or platform`; dieser Text wird **nicht** uebernommen, weil
    /// er einen Treibernamen nennt, den es hier nicht gibt. Statt dessen
    /// steht der Grund in [`Fehler::NichtUnterstuetzt`].
    pub fn neu(wunsch: &Wunsch) -> Ergebnis<Fenster> {
        if wunsch.groesse.0 == 0 || wunsch.groesse.1 == 0 {
            return Err(Fehler::Wert(
                "width or height should not be less than or equal to zero.",
            ));
        }
        if wunsch.opengl || wunsch.vulkan {
            // Gemessen: echtes pygame lehnt beides unter
            // `SDL_VIDEODRIVER=dummy` ebenfalls ab, woertlich mit
            // `OpenGL support is either not configured in SDL or not
            // available in current SDL video driver (dummy) or platform`
            // (und ebenso fuer Vulkan). Der Text wird **nicht**
            // uebernommen, weil er einen Treibernamen nennt, den es hier
            // nicht gibt; die Sache selbst - ein Fehler - ist dieselbe.
            return Err(Fehler::NichtUnterstuetzt(
                "OpenGL and Vulkan support is not available: neither minifb nor the windowless backend creates a GL or Vulkan context",
            ));
        }

        // Gemessen: `pygame.Window(...)` faehrt die Anzeige von sich aus
        // hoch - `display.get_init()` ist danach wahr, auch ohne
        // `pygame.init()`.
        crate::display::init()?;

        let vollbild = wunsch.vollbild || wunsch.vollbild_arbeitsflaeche;
        let fenstergroesse = wunsch.groesse;
        let groesse = if vollbild {
            arbeitsflaeche()?
        } else {
            fenstergroesse
        };

        let mut flags = 0;
        if wunsch.rahmenlos {
            flags |= NOFRAME;
        }
        if wunsch.groessenaenderbar {
            flags |= RESIZABLE;
        }
        if vollbild {
            flags |= FULLSCREEN;
        }
        if wunsch.versteckt {
            flags |= HIDDEN;
        }

        // Ausserhalb der Ausleihe: ein Fenster aufzumachen kann dauern.
        let treiber = mit_zustand(|z| z.treiber);
        let mut backend = crate::backend::oeffnen(treiber, groesse, flags, &wunsch.titel)?;

        // `always_on_top` kennt keine pygame-Flagge, also erst hinterher.
        //
        // Der Rueckgabewert wird **verworfen**, und das ist der Punkt:
        // gemessen meldet `Window('T', (64, 48), always_on_top=True)
        // .always_on_top` unter `SDL_VIDEODRIVER=dummy` `True`, obwohl
        // dort kein Fenster obenauf liegen kann - SDL gibt die Flagge
        // zurueck, mit der das Fenster **angelegt** wurde. Genau wie bei
        // `borderless` und `resizable`: das Wort im Konstruktor haelt,
        // nur der Setzer bleibt wirkungslos.
        //
        // Hier stand vorher `wunsch.obenauf && backend.set_obenauf(true)
        // .is_ok()`. Das war die Ursache dafuer, dass
        // `Window(.., always_on_top=True).always_on_top` falsch meldete:
        // das fensterlose Stueck kann `set_obenauf` nicht, also fiel der
        // Wunsch weg - obwohl pygame ihn behaelt.
        if wunsch.obenauf {
            let _ = backend.set_obenauf(true);
        }
        // Gemessen: ein Hilfsfenster laesst pygame unter dem
        // Dummy-Treiber klaglos anlegen, und `w.utility` meldet danach
        // `True`. Siehe [`Fenster::hilfsfenster`] fuer die Frage, wann
        // diese Kiste das wiederholen darf und wann nicht.
        let hilfsfenster = wunsch.hilfsfenster && !backend.hat_fenster();

        let position = stelle_ausrechnen(wunsch.stelle, backend.as_mut());

        mit_zustand(|z| {
            let id = z.fenster.naechste_id;
            z.fenster.naechste_id += 1;
            z.fenster.eintraege.push(Eintrag {
                id,
                titel: wunsch.titel.clone(),
                groesse,
                fenstergroesse,
                position,
                mindest: (0, 0),
                hoechst: (0, 0),
                maus_rechteck: None,
                flags,
                obenauf: wunsch.obenauf,
                hilfsfenster,
                fokus: false,
                // Gemessen: `Window(.., mouse_grabbed=True,
                // keyboard_grabbed=True)` meldet danach `grab_mouse` und
                // `grab_keyboard` als wahr - die beiden Woerter setzen
                // wirklich den Wunsch. Hier wurden sie vorher
                // stillschweigend verschluckt.
                will_maus: wunsch.griff_maus,
                will_tastatur: wunsch.griff_tastatur,
                griff_wirkt: false,
                stueck: Stueck::Eigen {
                    backend,
                    screen: None,
                },
            });
            if !wunsch.versteckt {
                fokus_uebernehmen(z, id);
            }
            // Erst jetzt, denn ob der Griff wirkt, haengt am Fokus.
            let i = z.fenster.stelle_von(id).expect("gerade angelegt");
            griff_nachfuehren(z, i);
            Ok(Fenster { id })
        })
    }

    /// Das Fenster, das `display.set_mode` aufgemacht hat -
    /// `Window.from_display_module()`.
    ///
    /// Gemessen:
    ///
    /// * Ohne vorheriges `set_mode` ist das ein Fehler mit dem Wortlaut
    ///   `display.set_mode has not been called yet.`
    /// * `w.get_surface()` ist **dieselbe** Flaeche, die `set_mode`
    ///   zurueckgegeben hat.
    /// * `w.title = 'X'` aendert `display.get_caption()` mit.
    /// * Wer **zuerst** ein `Window` anlegt und **dann** `set_mode`
    ///   ruft, bekommt zwei Fenster: `set_mode` macht ein eigenes auf,
    ///   und dieser Aufruf liefert dessen Nummer, nicht die des ersten.
    pub fn vom_anzeigemodul() -> Ergebnis<Fenster> {
        mit_zustand(|z| {
            if z.screen.is_none() || z.backend.is_none() {
                return Err(Fehler::Pygame("display.set_mode has not been called yet."));
            }
            anzeigefenster(z).ok_or(Fehler::Pygame("display.set_mode has not been called yet."))
        })
    }

    // -- Die einfachen Auskuenfte ---------------------------------------

    /// Die Nummer - `Window.id`. Nur lesbar, gemessen.
    pub fn id(&self) -> Ergebnis<u32> {
        self.lesen(|e| e.id)
    }

    /// Lebt das Fenster noch? Nach [`Fenster::zerstoeren`] nicht mehr.
    pub fn lebt(&self) -> bool {
        mit_zustand(|z| z.fenster.stelle_von(self.id).is_ok())
    }

    /// Der Titel - `Window.title`.
    pub fn titel(&self) -> Ergebnis<String> {
        self.lesen(|e| e.titel.clone())
    }

    /// Setzt den Titel.
    ///
    /// Beim Anzeigefenster geht er auch in `display.get_caption()` -
    /// gemessen.
    pub fn set_titel(&self, titel: &str) -> Ergebnis<()> {
        self.schreiben(|z, i| {
            z.fenster.eintraege[i].titel = titel.to_string();
            let anzeige = matches!(z.fenster.eintraege[i].stueck, Stueck::Anzeige);
            if anzeige {
                z.caption = (titel.to_string(), titel.to_string());
                z.caption_gesetzt = true;
            }
            if let Some(b) = stueck_mut(z, i) {
                b.set_caption(titel);
            }
            Ok(())
        })
    }

    /// Die Groesse - `Window.size`.
    pub fn groesse(&self) -> Ergebnis<(u32, u32)> {
        self.lesen(|e| e.groesse)
    }

    /// Setzt die Groesse.
    ///
    /// Gemessen, drei Dinge:
    ///
    /// * Null oder weniger ist ein Fehler - `width or height should not
    ///   be less than or equal to zero`, **ohne** Punkt am Ende. Der
    ///   Konstruktor setzt einen; siehe [`Fenster::neu`].
    /// * **Im Vollbild** wirkt der Aufruf nicht auf `size`, wird aber
    ///   gemerkt: nach `set_windowed()` gilt die neue Groesse.
    /// * Er legt ein [`WINDOWSIZECHANGED`] ab, sonst nichts.
    /// * **Die Grenzen gelten.** Nach `maximum_size = (50, 50)` bleibt
    ///   `size = (640, 480)` wirkungslos und das Fenster steht weiter auf
    ///   50x50 - kein Fehler, es passiert schlicht nichts. Ebenso
    ///   andersherum mit `minimum_size`. Das macht SDL selbst, nicht
    ///   pygame, und es ist der ganze Sinn der beiden Grenzen;
    ///   `window_test.test_maximum_size` prueft genau diese Zeile.
    pub fn set_groesse(&self, groesse: (u32, u32)) -> Ergebnis<()> {
        if groesse.0 == 0 || groesse.1 == 0 {
            return Err(Fehler::Wert(
                "width or height should not be less than or equal to zero",
            ));
        }
        self.schreiben(|z, i| {
            z.fenster.eintraege[i].fenstergroesse = groesse;
            if z.fenster.eintraege[i].flags & FULLSCREEN != 0 {
                // Gemessen: im Vollbild bleibt `size` die Arbeitsflaeche.
                return Ok(());
            }
            groesse_setzen(z, i, in_grenzen(&z.fenster.eintraege[i], groesse))
        })
    }

    /// Wo das Fenster steht - `Window.position`.
    ///
    /// Fragt zuerst das Hintergrundstueck; nur wenn das seine Stelle
    /// nicht kennt, kommt der gemerkte Wert. Mit minifb ist das echt
    /// (`Window::get_position()`), ohne Fenster die mittige Rechnung aus
    /// [`crate::backend::dummy`].
    pub fn position(&self) -> Ergebnis<(i32, i32)> {
        self.schreiben(|z, i| {
            let gemerkt = z.fenster.eintraege[i].position;
            Ok(stueck_mut(z, i)
                .and_then(|b| b.fenster_pos().ok())
                .unwrap_or(gemerkt))
        })
    }

    /// Versetzt das Fenster.
    ///
    /// Gemessen: negative Werte sind erlaubt und kommen genauso wieder
    /// heraus; Ereignisse legt der Aufruf unter dem Dummy-Treiber keine
    /// ab.
    pub fn set_position(&self, stelle: Stelle) -> Ergebnis<()> {
        self.schreiben(|z, i| {
            let groesse = z.fenster.eintraege[i].groesse;
            let pos = match stelle {
                Stelle::Genau(p) => p,
                _ => mittig(groesse, z),
            };
            z.fenster.eintraege[i].position = pos;
            if let Some(b) = stueck_mut(z, i) {
                // Ein Stueck, das das Fenster nicht versetzen kann, soll
                // deshalb nicht den ganzen Aufruf verderben - gemessen
                // verpufft er bei pygame ohne Fenster ebenfalls klaglos.
                let _ = b.set_fenster_pos(pos);
            }
            Ok(())
        })
    }

    /// Hat das Fenster den Fokus? - `Window.focused`. Nur lesbar, gemessen.
    pub fn fokussiert(&self) -> Ergebnis<bool> {
        self.lesen(|e| e.fokus)
    }

    /// Ist das Fenster ohne Rahmen? - `Window.borderless`.
    pub fn rahmenlos(&self) -> Ergebnis<bool> {
        self.lesen(|e| e.flags & NOFRAME != 0)
    }

    /// Setzt den Rahmen ab oder wieder auf.
    ///
    /// Wirkt nur, wenn wirklich ein Fenster dahintersteht - siehe den
    /// Absatz "Die drei Merkmale" im Modulkopf. Ohne Fenster bleibt es
    /// beim Wert aus dem Konstruktor, genau wie bei pygame unter dem
    /// Dummy-Treiber; einen Fehler gibt es dort wie hier nicht.
    pub fn set_rahmenlos(&self, an: bool) -> Ergebnis<()> {
        self.flagge_setzen(NOFRAME, an)
    }

    /// Darf der Benutzer ziehen? - `Window.resizable`.
    pub fn groessenaenderbar(&self) -> Ergebnis<bool> {
        self.lesen(|e| e.flags & RESIZABLE != 0)
    }

    /// Setzt das um. Dieselbe Einschraenkung wie [`Fenster::set_rahmenlos`].
    pub fn set_groessenaenderbar(&self, an: bool) -> Ergebnis<()> {
        self.flagge_setzen(RESIZABLE, an)
    }

    /// Liegt das Fenster ueber allen anderen? - `Window.always_on_top`.
    pub fn obenauf(&self) -> Ergebnis<bool> {
        self.lesen(|e| e.obenauf)
    }

    /// Legt das Fenster obenauf oder nimmt es wieder herunter.
    ///
    /// Mit minifb ist das echt (`Window::topmost`). Ohne Fenster bleibt
    /// es stehen - siehe [`Fenster::set_rahmenlos`], derselbe Grund.
    pub fn set_obenauf(&self, an: bool) -> Ergebnis<()> {
        self.schreiben(|z, i| {
            let ging = stueck_mut(z, i).is_some_and(|b| b.set_obenauf(an).is_ok());
            if ging {
                z.fenster.eintraege[i].obenauf = an;
            }
            Ok(())
        })
    }

    /// Ist das ein Hilfsfenster? - `Window.utility`. Nur lesbar.
    ///
    /// Gemessen: `Window('T', (64, 48), utility=True)` **gelingt** unter
    /// `SDL_VIDEODRIVER=dummy`, und `w.utility` meldet danach `True` -
    /// obwohl dort gar kein Fenster in einer Leiste stehen koennte. Es
    /// ist dieselbe Buchfuehrung wie bei `borderless`: SDL gibt die
    /// Flagge zurueck, mit der das Fenster angelegt wurde.
    ///
    /// **Hier gilt deshalb eine Regel, die beides ehrlich haelt:** der
    /// Wunsch wird gemeldet, solange ihm kein Stueck widersprechen kann,
    /// und sonst nicht.
    ///
    /// | Stueck | `Window(.., utility=True).utility` | warum |
    /// |---|---|---|
    /// | ohne Fenster | `true` | es gibt keine Leiste, in der es fehlen koennte - wie bei pygame |
    /// | minifb | **`false`** | das Fenster steht da und ist nachweislich ein gewoehnliches |
    ///
    /// Bei `borderless` und `always_on_top` faellt die Unterscheidung
    /// weg, weil minifb die beiden wirklich kann. `utility` kann es
    /// nicht - `WindowOptions` hat kein Gegenstueck dazu -, und ein
    /// `true` waere dort eine Zusage, die kein Fensterverwalter einloest.
    ///
    /// [`Fenster::neu`] hat den Wunsch vorher **abgewiesen**. Das war zu
    /// streng: pygame legt das Fenster an, und ein Programm, das
    /// `utility=True` schreibt, verliert dadurch sein Fenster statt bloss
    /// die Leisteneigenschaft.
    pub fn hilfsfenster(&self) -> Ergebnis<bool> {
        self.lesen(|e| e.hilfsfenster)
    }

    /// Zeichnet das Fenster mit OpenGL? - `Window.opengl`. Nur lesbar.
    ///
    /// **Immer falsch**: [`Fenster::neu`] laesst `opengl` gar nicht erst
    /// zu. Siehe [`crate::display::gl_get_attribute`] fuer den Grund.
    pub fn opengl(&self) -> Ergebnis<bool> {
        self.lesen(|_| false)
    }

    /// Die Deckkraft zwischen 0.0 und 1.0 - `Window.opacity`.
    ///
    /// **Immer 1.0**, und das ist keine erfundene Zahl, sondern die
    /// wahre: das Fenster ist undurchsichtig, weil es niemand anders
    /// machen kann. Gemessen meldet echtes pygame unter
    /// `SDL_VIDEODRIVER=dummy` dasselbe - und wirft beim Setzen.
    pub fn deckkraft(&self) -> Ergebnis<f32> {
        self.lesen(|_| 1.0)
    }

    /// Stellt die Deckkraft.
    ///
    /// **Geht hier nicht.** minifb kennt keine Fensterdeckkraft, und ohne
    /// Fenster gibt es nichts durchscheinen zu lassen. Gemessen wirft
    /// echtes pygame unter dem Dummy-Treiber an dieser Stelle ebenfalls -
    /// woertlich `That operation is not supported`, was hier der
    /// `Display`-Text von [`Fehler::NichtUnterstuetzt`] ist.
    pub fn set_deckkraft(&self, wert: f32) -> Ergebnis<()> {
        self.schreiben(|z, i| match stueck_mut(z, i) {
            Some(b) => b.set_deckkraft(wert),
            None => Err(Fehler::KeinFenster),
        })
    }

    // -- Kleinste und groesste Groesse ----------------------------------

    /// Die kleinste Groesse - `Window.minimum_size`.
    pub fn mindestgroesse(&self) -> Ergebnis<(u32, u32)> {
        self.lesen(|e| e.mindest)
    }

    /// Setzt die kleinste Groesse.
    ///
    /// Drei gemessene Regeln, die man sich nicht ausdenkt:
    ///
    /// * Ein negativer Wert ist ein Fehler - `minimum width or height
    ///   should not be less than zero`.
    /// * Groesser als die groesste Groesse (sofern gesetzt) ist ein
    ///   Fehler - `minimum width or height should not be greater than
    ///   maximum width or height respectively`.
    /// * **`(0, 0)` setzt nichts zurueck.** Es laeuft klaglos durch und
    ///   laesst den alten Wert stehen - SDL weist die Null ab, und
    ///   pygame reicht das nicht weiter. Wer eine Grenze wieder loswerden
    ///   will, kann es bei pygame gar nicht; hier auch nicht.
    ///
    /// Ist das Fenster kleiner als die neue Grenze, waechst es - ebenso
    /// gemessen.
    pub fn set_mindestgroesse(&self, groesse: (i64, i64)) -> Ergebnis<()> {
        if groesse.0 < 0 || groesse.1 < 0 {
            return Err(Fehler::Wert(
                "minimum width or height should not be less than zero",
            ));
        }
        self.schreiben(|z, i| {
            let (hw, hh) = z.fenster.eintraege[i].hoechst;
            if hw > 0 && hh > 0 && (groesse.0 > hw as i64 || groesse.1 > hh as i64) {
                return Err(Fehler::Wert(
                    "minimum width or height should not be greater than maximum width or height respectively",
                ));
            }
            if groesse.0 == 0 || groesse.1 == 0 {
                // Gemessen: verpufft, der alte Wert bleibt.
                return Ok(());
            }
            let neu = (groesse.0 as u32, groesse.1 as u32);
            z.fenster.eintraege[i].mindest = neu;
            let jetzt = z.fenster.eintraege[i].groesse;
            let gross = (jetzt.0.max(neu.0), jetzt.1.max(neu.1));
            if gross != jetzt {
                groesse_setzen(z, i, gross)?;
            }
            Ok(())
        })
    }

    /// Die groesste Groesse - `Window.maximum_size`.
    pub fn hoechstgroesse(&self) -> Ergebnis<(u32, u32)> {
        self.lesen(|e| e.hoechst)
    }

    /// Setzt die groesste Groesse. Spiegelbild von
    /// [`Fenster::set_mindestgroesse`], mit denselben drei Regeln und den
    /// dazu gemessenen Wortlauten.
    pub fn set_hoechstgroesse(&self, groesse: (i64, i64)) -> Ergebnis<()> {
        if groesse.0 < 0 || groesse.1 < 0 {
            return Err(Fehler::Wert(
                "maximum width or height should not be less than zero",
            ));
        }
        self.schreiben(|z, i| {
            let (mw, mh) = z.fenster.eintraege[i].mindest;
            if mw > 0 && mh > 0 && (groesse.0 < mw as i64 || groesse.1 < mh as i64) {
                return Err(Fehler::Wert(
                    "maximum width or height should not be less than minimum width or height respectively",
                ));
            }
            if groesse.0 == 0 || groesse.1 == 0 {
                return Ok(());
            }
            let neu = (groesse.0 as u32, groesse.1 as u32);
            z.fenster.eintraege[i].hoechst = neu;
            let jetzt = z.fenster.eintraege[i].groesse;
            let klein = (jetzt.0.min(neu.0), jetzt.1.min(neu.1));
            if klein != jetzt {
                groesse_setzen(z, i, klein)?;
            }
            Ok(())
        })
    }

    // -- Mausgehege und Griff -------------------------------------------

    /// Worin der Zeiger bleiben soll - `Window.mouse_rect`.
    pub fn maus_rechteck(&self) -> Ergebnis<Option<Rect>> {
        self.lesen(|e| e.maus_rechteck)
    }

    /// Setzt das Gehege; `None` hebt es auf.
    ///
    /// Gemessen: ein Rechteck mit Breite **oder** Hoehe kleiner-gleich
    /// null kommt als `None` wieder heraus - `Rect(0, 0, 0, 0)` und
    /// `Rect(0, 0, -5, 10)` heben das Gehege also auf, statt ein leeres
    /// zu setzen. Ausserhalb des Fensters ist dagegen erlaubt:
    /// `Rect(100, 100, 10, 10)` bleibt stehen.
    ///
    /// **Der Zeiger wird davon nicht gehalten.** Weder minifb noch das
    /// fensterlose Stueck koennen ihn versetzen (siehe
    /// [`crate::backend::Backend::set_mouse_pos`]), also ist das hier
    /// Buchfuehrung - dieselbe, die pygame unter dem Dummy-Treiber
    /// betreibt: gesetzt, gemerkt, wieder herausgegeben, und kein Pixel
    /// bewegt sich anders.
    pub fn set_maus_rechteck(&self, rechteck: Option<Rect>) -> Ergebnis<()> {
        let sauber = rechteck.filter(|r| r.w > 0 && r.h > 0);
        self.schreiben(|z, i| {
            z.fenster.eintraege[i].maus_rechteck = sauber;
            Ok(())
        })
    }

    /// Der **Wunsch**, die Maus zu halten - `Window.grab_mouse`.
    pub fn will_maus(&self) -> Ergebnis<bool> {
        self.lesen(|e| e.will_maus)
    }

    /// Der **Wunsch**, die Tastatur zu halten - `Window.grab_keyboard`.
    pub fn will_tastatur(&self) -> Ergebnis<bool> {
        self.lesen(|e| e.will_tastatur)
    }

    /// Haelt dieses Fenster die Maus wirklich? - `Window.mouse_grabbed`.
    /// Nur lesbar.
    ///
    /// Der Wunsch allein genuegt nicht - siehe `haelt_griff` und den
    /// Absatz ueber den Griff im Modulkopf.
    pub fn haelt_maus(&self) -> Ergebnis<bool> {
        self.lesen(|e| haelt_griff(e) && e.will_maus)
    }

    /// Haelt dieses Fenster die Tastatur wirklich? -
    /// `Window.keyboard_grabbed`. Nur lesbar.
    pub fn haelt_tastatur(&self) -> Ergebnis<bool> {
        self.lesen(|e| haelt_griff(e) && e.will_tastatur)
    }

    /// Setzt den Mausgriff.
    pub fn set_griff_maus(&self, an: bool) -> Ergebnis<()> {
        self.griff_setzen(an, true)
    }

    /// Setzt den Tastaturgriff.
    pub fn set_griff_tastatur(&self, an: bool) -> Ergebnis<()> {
        self.griff_setzen(an, false)
    }

    // -- Die Flaeche ----------------------------------------------------

    /// Die Zeichenflaeche - `Window.get_surface()`.
    ///
    /// Beim Anzeigefenster ist das **dieselbe** Flaeche, die
    /// [`crate::display::set_mode`] geliefert hat - nicht eine Kopie.
    /// Bei einem eigenen Fenster wird sie beim ersten Aufruf angelegt und
    /// danach immer wieder herausgegeben; ein Groessenwechsel baut sie um
    /// statt sie zu ersetzen. Beides gemessen.
    pub fn flaeche(&self) -> Ergebnis<Screen> {
        self.schreiben(|z, i| {
            if let Some(s) = flaeche_von(z, i) {
                return Ok(s);
            }
            let groesse = z.fenster.eintraege[i].groesse;
            let flags = z.fenster.eintraege[i].flags;
            let neu: Screen = Arc::new(RwLock::new(anzeigeflaeche(groesse, flags)));
            match &mut z.fenster.eintraege[i].stueck {
                Stueck::Eigen { screen, .. } => *screen = Some(Arc::clone(&neu)),
                // Das Anzeigefenster hat seine Flaeche schon, sobald
                // `set_mode` gelaufen ist - hierher kommt es nie.
                Stueck::Anzeige => z.screen = Some(Arc::clone(&neu)),
            }
            Ok(neu)
        })
    }

    /// Zeigt die Flaeche an - `Window.flip()`.
    ///
    /// Gemessen: **ohne** vorheriges [`Fenster::flaeche`] ist das ein
    /// Fehler, woertlich `the Window has no surface associated with it,
    /// did you forget to call Window.get_surface()`. pygame legt die
    /// Flaeche erst dort an, und `flip` findet dann nichts vor.
    pub fn flip(&self) -> Ergebnis<()> {
        self.schreiben(|z, i| {
            let screen = flaeche_von(z, i).ok_or(Fehler::Pygame(
                "the Window has no surface associated with it, \
                 did you forget to call Window.get_surface()",
            ))?;
            let backend = stueck_mut(z, i).ok_or(Fehler::KeinFenster)?;
            let mut s = screen.try_write().expect(GELEIHT);
            let groesse = s.get_size();
            s.mit_pixeln(|daten, _zeilenbreite, _versatz| backend.zeigen(daten, groesse))
        })
    }

    /// Setzt das Fenstersinnbild - `Window.set_icon()`.
    ///
    /// Ohne Fenster geht es (und tut nichts), mit minifb nicht - siehe
    /// [`crate::display::set_icon`], dieselbe Begruendung.
    pub fn set_sinnbild(&self, bild: &Surface) -> Ergebnis<()> {
        self.schreiben(|z, i| match stueck_mut(z, i) {
            Some(b) => b.set_icon(bild),
            None => Err(Fehler::KeinFenster),
        })
    }

    // -- Zeigen, verstecken, nach vorn ----------------------------------

    /// Zeigt das Fenster - `Window.show()`.
    ///
    /// Legt dabei dieselben fuenf Ereignisse ab wie
    /// [`crate::display::set_mode`] - gemessen.
    pub fn zeigen(&self) -> Ergebnis<()> {
        self.sichtbarkeit(true)
    }

    /// Versteckt das Fenster - `Window.hide()`.
    ///
    /// Legt die fuenf **Gegenereignisse** ab: `ACTIVEEVENT(gain=0,
    /// APPMOUSEFOCUS)`, `WINDOWLEAVE`, `ACTIVEEVENT(gain=0,
    /// APPINPUTFOCUS)`, `WINDOWFOCUSLOST`, `WINDOWHIDDEN` - Reihenfolge
    /// und Felder gemessen.
    pub fn verstecken(&self) -> Ergebnis<()> {
        self.sichtbarkeit(false)
    }

    /// Holt das Fenster nach vorn - `Window.focus()`.
    ///
    /// `nur_eingabe` ist pygames `input_only=True`. Gemessen, und der
    /// Unterschied ist echt: ohne das Wort geht der Aufruf unter dem
    /// Dummy-Treiber durch, **mit** ihm wirft pygame `That operation is
    /// not supported`. Mit minifb geht beides nicht - es kann ein
    /// Fenster nicht heben.
    ///
    /// **Den Fokus verschiebt der Aufruf nicht**, und das ist gemessen,
    /// nicht gespart: bei zwei Fenstern laesst `a.focus()` unter dem
    /// Dummy-Treiber `a.focused` auf `False` und `b.focused` auf `True`.
    /// Es gibt dort keinen Fensterverwalter, der etwas heben koennte -
    /// derselbe Fall wie bei [`Fenster::zustand_setzen`] und
    /// [`crate::display::iconify`].
    ///
    /// Hier stand vorher ein `fokus_uebernehmen`. Das fiel nicht auf,
    /// weil die Gegenprobe nur ein einzelnes Fenster kannte, das den
    /// Fokus ohnehin hatte.
    pub fn nach_vorn(&self, nur_eingabe: bool) -> Ergebnis<()> {
        self.schreiben(|z, i| match stueck_mut(z, i) {
            Some(b) => b.nach_vorn(nur_eingabe),
            None => Err(Fehler::KeinFenster),
        })
    }

    /// Laesst das Fenster blinken - `Window.flash()`.
    ///
    /// `art` ist [`FLASH_CANCEL`], [`FLASH_BRIEFLY`] oder
    /// [`FLASH_UNTIL_FOCUSED`]; alles andere ist ein Fehler mit dem
    /// gemessenen Wortlaut `Unsupported window flash operation.` - und
    /// zwar **vor** der Frage, ob das Stueck ueberhaupt blinken kann,
    /// ebenfalls gemessen.
    ///
    /// Blinken selbst gibt es hier nicht: es ist eine Sache des
    /// Fensterverwalters (in der Leiste blinken, das Sinnbild huepfen
    /// lassen), und minifb reicht nichts dorthin durch. Echtes pygame
    /// wirft unter dem Dummy-Treiber fuer **alle drei** Arten ebenfalls.
    pub fn blinken(&self, art: u32) -> Ergebnis<()> {
        if !(art == FLASH_CANCEL || art == FLASH_BRIEFLY || art == FLASH_UNTIL_FOCUSED) {
            return Err(Fehler::Wert("Unsupported window flash operation."));
        }
        self.schreiben(|z, i| match stueck_mut(z, i) {
            Some(b) => b.blinken(art),
            None => Err(Fehler::KeinFenster),
        })
    }

    /// Haengt das Fenster als Kind an ein anderes -
    /// `Window.set_modal_for()`.
    ///
    /// **Geht hier nicht**, und gemessen auch bei pygame unter dem
    /// Dummy-Treiber nicht (`That operation is not supported`). Ein
    /// modales Fenster ist eine Abmachung mit dem Fensterverwalter: er
    /// haelt es ueber dem Elternfenster und laesst dieses keine Eingaben
    /// annehmen. minifb hat dafuer keinen Aufruf.
    ///
    /// `eltern` wird trotzdem geprueft - ein totes Elternfenster meldet
    /// `Invalid window`, bevor der Rest scheitert.
    pub fn modal_fuer(&self, eltern: Fenster) -> Ergebnis<()> {
        eltern.id()?;
        self.schreiben(|z, i| match stueck_mut(z, i) {
            Some(b) => b.modal_fuer(),
            None => Err(Fehler::KeinFenster),
        })
    }

    /// Legt ab, zieht auf oder stellt zurueck - `Window.minimize()`,
    /// `maximize()`, `restore()`.
    ///
    /// Ohne Fenster geht es und tut nichts - gemessen gibt echtes pygame
    /// unter dem Dummy-Treiber `None` zurueck, aendert die Groesse nicht
    /// und legt kein Ereignis ab. Mit minifb ist es ein Fehler: es hat
    /// keinen dieser drei Aufrufe, und an das Fenster des Systems kaeme
    /// man nur ueber unsicheren Code.
    pub fn zustand_setzen(&self, art: Fensterzustand) -> Ergebnis<()> {
        self.schreiben(|z, i| match stueck_mut(z, i) {
            Some(b) => b.fensterzustand(art),
            None => Err(Fehler::KeinFenster),
        })
    }

    /// Laeuft das Fenster im Vollbild?
    pub fn vollbild(&self) -> Ergebnis<bool> {
        self.lesen(|e| e.flags & FULLSCREEN != 0)
    }

    /// Schaltet Vollbild ein oder aus - `Window.set_fullscreen()` und
    /// `Window.set_windowed()`.
    ///
    /// Gemessen, und der Unterschied zu [`crate::display::toggle_fullscreen`]
    /// ist echt: dieser Weg geht unter `SDL_VIDEODRIVER=dummy` durch und
    /// macht die Flaeche `(1024, 768)` gross, waehrend
    /// `display.toggle_fullscreen()` im selben Lauf `That operation is
    /// not supported` wirft. Zwei Wege zum selben Ziel, zwei Antworten.
    ///
    /// Beim Zurueckschalten gilt wieder die gemerkte Fenstergroesse -
    /// auch eine, die waehrend des Vollbilds gesetzt wurde. Die Flaeche
    /// bleibt in beiden Richtungen **dieselbe** und wird nur umgebaut.
    ///
    /// Mit minifb geht es nicht; `arbeitsflaeche` sagt dort ohnehin
    /// nichts, weil minifb keine Bildschirmliste kennt.
    pub fn set_vollbild(&self, an: bool) -> Ergebnis<()> {
        let flaeche = if an { Some(arbeitsflaeche()?) } else { None };
        self.schreiben(|z, i| {
            let ziel = match flaeche {
                Some(g) => g,
                None => z.fenster.eintraege[i].fenstergroesse,
            };
            match stueck_mut(z, i) {
                Some(b) => b.set_vollbild(an, ziel)?,
                None => return Err(Fehler::KeinFenster),
            }
            let e = &mut z.fenster.eintraege[i];
            if an {
                e.flags |= FULLSCREEN;
            } else {
                e.flags &= !FULLSCREEN;
            }
            let anzeige = matches!(e.stueck, Stueck::Anzeige);
            if anzeige {
                // Damit `display.is_fullscreen()` mitzieht - gemessen
                // sagt es nach `Window.set_fullscreen()` `True`.
                z.flags = if an {
                    z.flags | FULLSCREEN
                } else {
                    z.flags & !FULLSCREEN
                };
            }
            groesse_setzen(z, i, ziel)
        })
    }

    /// Macht das Fenster zu - `Window.destroy()`.
    ///
    /// Gemessen: ein zweiter Aufruf ist kein Fehler, und der Griff wird
    /// dabei frei. Was pygame danach noch herausgibt (Zufallszahlen fuer
    /// `size`, die leere Zeichenkette fuer `title`) wird **nicht**
    /// nachgebaut - siehe `Ablage::leeren`.
    ///
    /// War es das Anzeigefenster, ist danach auch `display.get_surface()`
    /// leer und ein neues `set_mode` macht wieder eines auf - ebenfalls
    /// gemessen.
    pub fn zerstoeren(&self) {
        mit_zustand(|z| {
            let Ok(i) = z.fenster.stelle_von(self.id) else {
                return;
            };
            let anzeige = matches!(z.fenster.eintraege[i].stueck, Stueck::Anzeige);
            schliessereignisse(z, self.id);
            // Gemessen: `destroy()` gibt den Griff frei. Das Stueck geht
            // mit dem Eintrag mit, also muss es vorher gefragt werden.
            if let Some(b) = stueck_mut(z, i) {
                b.set_grab(false);
            }
            z.fenster.eintraege.remove(i);
            if anzeige {
                z.backend = None;
                z.screen = None;
                z.flags = 0;
            }
        })
    }

    // -- Innereien ------------------------------------------------------

    /// Liest ein Feld, oder meldet `Invalid window`.
    fn lesen<R>(&self, f: impl FnOnce(&Eintrag) -> R) -> Ergebnis<R> {
        mit_zustand(|z| {
            let i = z.fenster.stelle_von(self.id)?;
            Ok(f(&z.fenster.eintraege[i]))
        })
    }

    /// Fuehrt etwas am Eintrag aus, mit vollem Zugriff auf den Zustand.
    fn schreiben<R>(&self, f: impl FnOnce(&mut Zustand, usize) -> Ergebnis<R>) -> Ergebnis<R> {
        mit_zustand(|z| {
            let i = z.fenster.stelle_von(self.id)?;
            f(z, i)
        })
    }

    /// Setzt eine Fensterflagge - nur, wenn ein Fenster dahintersteht.
    fn flagge_setzen(&self, flagge: u32, an: bool) -> Ergebnis<()> {
        self.schreiben(|z, i| {
            if !stueck_mut(z, i).is_some_and(|b| b.hat_fenster()) {
                // Gemessen: pygame laesst es unter dem Dummy-Treiber
                // ebenso klaglos fallen. Siehe Modulkopf.
                return Ok(());
            }
            let alt = z.fenster.eintraege[i].flags;
            let neu = if an { alt | flagge } else { alt & !flagge };
            if neu == alt {
                return Ok(());
            }
            z.fenster.eintraege[i].flags = neu;
            anzeigeflaggen_nachziehen(z, i);
            let groesse = z.fenster.eintraege[i].groesse;
            match stueck_mut(z, i) {
                Some(b) => b.set_mode(groesse, neu),
                None => Ok(()),
            }
        })
    }

    /// Setzt einen der beiden Griffe.
    ///
    /// Das Modell dahinter steht im Modulkopf und ist gemessen: der
    /// Wunsch bleibt beim Fenster stehen, gehalten wird nur vom Fenster
    /// mit dem **Eingabefokus**.
    fn griff_setzen(&self, an: bool, maus: bool) -> Ergebnis<()> {
        self.schreiben(|z, i| {
            let e = &mut z.fenster.eintraege[i];
            if maus {
                e.will_maus = an;
            } else {
                e.will_tastatur = an;
            }
            griff_nachfuehren(z, i);
            Ok(())
        })
    }

    /// Zeigt oder versteckt und legt die fuenf Ereignisse ab.
    fn sichtbarkeit(&self, an: bool) -> Ergebnis<()> {
        self.schreiben(|z, i| {
            match stueck_mut(z, i) {
                Some(b) => b.fenster_zeigen(an)?,
                None => return Err(Fehler::KeinFenster),
            }
            let id = z.fenster.eintraege[i].id;
            let e = &mut z.fenster.eintraege[i];
            if an {
                e.flags &= !HIDDEN;
            } else {
                e.flags |= HIDDEN;
            }
            // Damit `display.get_active()` mitzieht: es liest
            // [`HIDDEN`] aus `z.flags`.
            anzeigeflaggen_nachziehen(z, i);
            if an {
                fokus_uebernehmen(z, id);
            } else {
                z.fenster.eintraege[i].fokus = false;
                // Gemessen: `hide()` nimmt den Griff, `show()` gibt ihn
                // zurueck - der Wunsch bleibt derweil stehen.
                griff_nachfuehren(z, i);
                schliessereignisse(z, id);
            }
            Ok(())
        })
    }
}

// ---------------------------------------------------------------------------
//  Freie Aufrufe
// ---------------------------------------------------------------------------

/// Welches Fenster den Griff haelt - `pygame.window.get_grabbed_window()`.
///
/// `None`, wenn keines greift. Gemessen: das ist auch vor jedem
/// `set_mode` die Antwort, und nach [`Fenster::zerstoeren`] wieder.
///
/// **Dieser Aufruf stuerzt hier nicht ab.** Echtes pygame tut es: ohne
/// laufendes Video-Teilsystem beendet `pygame.window.get_grabbed_window()`
/// den Prozess hart. Siehe den Modulkopf - dort steht, wann genau, und
/// warum jede Messung an `pygame.window` in einen eigenen Prozess gehoert.
pub fn get_grabbed_window() -> Option<Fenster> {
    mit_zustand(|z| {
        z.fenster
            .eintraege
            .iter()
            .find(|e| haelt_griff(e) && (e.will_maus || e.will_tastatur))
            .map(|e| Fenster { id: e.id })
    })
}

/// Haelt dieses Fenster ueberhaupt einen Griff?
///
/// Zwei Bedingungen, und die erste ist die, die man ohne Messung nicht
/// raet: **nur das Fenster mit dem Eingabefokus greift.** Gemessen an
/// echtem pygame unter `SDL_VIDEODRIVER=dummy`:
///
/// | Lage | `a.grab_mouse` | `a.mouse_grabbed` | `get_grabbed_window()` |
/// |---|---|---|---|
/// | ein Fenster, gegriffen | `True` | `True` | dieses |
/// | zwei Fenster, `a` gegriffen, `b` im Fokus | `True` | **`False`** | **`None`** |
/// | danach `a.focus()` unter dem Dummy-Treiber | `True` | `False` | `None` |
/// | `w.hide()` auf dem gegriffenen Fenster | `True` | `False` | `None` |
/// | danach `w.show()` | `True` | **`True`** | dieses |
///
/// Der Wunsch bleibt also stehen und der Griff kommt von selbst zurueck,
/// sobald das Fenster den Fokus wiederhat. Das ist SDL: es legt den Griff
/// erst an, wenn das Fenster den Eingabefokus hat, und nimmt ihn
/// weg, wenn es ihn verliert.
///
/// **Hier stand vorher ein anderes Modell** - "wer zuletzt greift, nimmt
/// dem vorigen den Griff weg". Das traf die eine Messung, aus der es
/// gewonnen war (zwei Fenster, beide greifen hintereinander), und lag bei
/// jeder anderen daneben: es meldete `a.mouse_grabbed` als wahr, sobald
/// `a` gegriffen hatte, ganz egal wer den Fokus hielt.
///
/// Die zweite Bedingung ist die uebliche dieser Kiste: das Stueck muss es
/// koennen. minifb kann den Zeiger nicht im Fenster halten, also steht
/// dort `false` - genau wie bei [`crate::event::get_grab`].
fn haelt_griff(e: &Eintrag) -> bool {
    e.fokus && e.griff_wirkt
}

/// Fragt das Stueck, ob der Griff jetzt gilt, und schreibt es fort.
///
/// Gerufen nach jeder Aenderung, die den Griff beruehren kann: der Wunsch
/// selbst, ein Fokuswechsel, Zeigen und Verstecken.
fn griff_nachfuehren(z: &mut Zustand, i: usize) {
    let e = &z.fenster.eintraege[i];
    let gewollt = (e.will_maus || e.will_tastatur) && e.fokus;
    let wirkt = stueck_mut(z, i).is_some_and(|b| b.set_grab(gewollt)) && gewollt;
    z.fenster.eintraege[i].griff_wirkt = wirkt;
}

/// Schreibt den Griff **aller** Fenster fort.
///
/// Ein Fokuswechsel betrifft zwei Fenster: das eine verliert den Griff,
/// das andere kann ihn bekommen. Beides gemessen - siehe `haelt_griff`.
fn griff_aller_nachfuehren(z: &mut Zustand) {
    for i in 0..z.fenster.eintraege.len() {
        griff_nachfuehren(z, i);
    }
}

// ---------------------------------------------------------------------------
//  Hilfsstuecke
// ---------------------------------------------------------------------------

/// Die Groesse der Arbeitsflaeche, fuer Vollbild und mittiges Stellen.
fn arbeitsflaeche() -> Ergebnis<(u32, u32)> {
    let flaechen = crate::display::get_desktop_sizes()?;
    flaechen.first().copied().ok_or(Fehler::NichtUnterstuetzt(
        "without a display list no fullscreen size can be determined",
    ))
}

/// Die mittige Stelle auf der Arbeitsflaeche.
///
/// Gemessen: `((1024 - breite) / 2, (768 - hoehe) / 2)`, und zwar auch
/// dann, wenn das Fenster gar nicht passt - `Window('U', (2000, 2000))`
/// steht bei `(-488, -616)`.
///
/// Kennt niemand die Arbeitsflaeche (minifb), bleibt `(0, 0)` - die
/// wirkliche Stelle liefert dann [`Fenster::position`] aus dem Stueck
/// selbst, das sie mit `Window::get_position()` wirklich weiss.
fn mittig(groesse: (u32, u32), z: &Zustand) -> (i32, i32) {
    let desktop = match z.backend.as_ref().and_then(|b| b.desktop_groessen().ok()) {
        Some(v) => v.first().copied(),
        None => match crate::backend::gewaehlter_treiber(z.treiber) {
            crate::zustand::Treiber::Dummy => Some(crate::backend::dummy::DESKTOP),
            _ => None,
        },
    };
    match desktop {
        Some((dw, dh)) => (
            (dw as i32 - groesse.0 as i32) / 2,
            (dh as i32 - groesse.1 as i32) / 2,
        ),
        None => (0, 0),
    }
}

/// Wo ein frisch geoeffnetes Fenster steht.
///
/// Die Groesse steht hier nicht dabei, weil sie nicht gebraucht wird:
/// das Stueck kennt seine Stelle selbst. Das fensterlose stellt sich
/// beim ersten `set_mode` mittig (mit **seiner** Groesse, siehe
/// [`crate::backend::dummy`]), und minifb weiss sie ohnehin.
fn stelle_ausrechnen(stelle: Stelle, backend: &mut dyn Backend) -> (i32, i32) {
    match stelle {
        Stelle::Genau(p) => {
            let _ = backend.set_fenster_pos(p);
            p
        }
        // Gemessen: `WINDOWPOS_CENTERED` und `WINDOWPOS_UNDEFINED`
        // liefern beide die Mitte. Das Stueck weiss es meist selbst -
        // das fensterlose stellt sich beim ersten `set_mode` mittig, und
        // minifb kennt seine wirkliche Stelle.
        _ => backend.fenster_pos().unwrap_or((0, 0)),
    }
}

/// Setzt die Groesse und legt das gemessene Ereignis ab.
///
/// Gemessen: ein Groessenwechsel legt genau **ein** [`WINDOWSIZECHANGED`]
/// ab, mit `x` und `y` als neuer Breite und Hoehe - kein `VIDEORESIZE`,
/// kein `WINDOWRESIZED`. (Beim Vollbildwechsel legt pygame mehr ab; das
/// ist ein anderer Weg und hier nicht nachgebaut, weil der Wechsel selbst
/// mit minifb gar nicht geht.)
/// Eine Groesse, auf die Grenzen dieses Fensters gestutzt.
///
/// `(0, 0)` heisst **keine Grenze** - so legt SDL sie an, und so bleiben
/// sie, bis jemand sie setzt.
fn in_grenzen(e: &Eintrag, groesse: (u32, u32)) -> (u32, u32) {
    let (mut b, mut h) = groesse;
    if e.mindest.0 > 0 && e.mindest.1 > 0 {
        b = b.max(e.mindest.0);
        h = h.max(e.mindest.1);
    }
    if e.hoechst.0 > 0 && e.hoechst.1 > 0 {
        b = b.min(e.hoechst.0);
        h = h.min(e.hoechst.1);
    }
    (b, h)
}

fn groesse_setzen(z: &mut Zustand, i: usize, groesse: (u32, u32)) -> Ergebnis<()> {
    if z.fenster.eintraege[i].groesse == groesse {
        return Ok(());
    }
    z.fenster.eintraege[i].groesse = groesse;
    let flags = z.fenster.eintraege[i].flags;
    match stueck_mut(z, i) {
        Some(b) => b.set_mode(groesse, flags)?,
        None => return Err(Fehler::KeinFenster),
    }
    // Die Flaeche wird **umgebaut**, nicht ersetzt - gemessen bleibt
    // `w.get_surface()` dasselbe Objekt.
    if let Some(s) = flaeche_von(z, i) {
        *s.try_write().expect(GELEIHT) = anzeigeflaeche(groesse, flags);
    }
    z.schlange.ablegen(Event::mit(
        WINDOWSIZECHANGED,
        Daten::Fenster {
            x: groesse.0 as i32,
            y: groesse.1 as i32,
        },
    ));
    Ok(())
}

/// Schreibt die Flaggen des Anzeigefensters nach `Zustand::flags`.
///
/// Nur fuer das Fenster des Anzeigemoduls, und nur, damit `display` und
/// `Window` nicht zwei verschiedene Wahrheiten fuehren:
/// [`crate::display::get_active`] liest [`HIDDEN`] aus `z.flags`,
/// [`crate::display::is_fullscreen`] das [`FULLSCREEN`] ebendort. Ein
/// eigenes Fenster laesst `z.flags` unberuehrt - es hat mit der Anzeige
/// nichts zu tun.
fn anzeigeflaggen_nachziehen(z: &mut Zustand, i: usize) {
    if matches!(z.fenster.eintraege[i].stueck, Stueck::Anzeige) {
        z.flags = z.fenster.eintraege[i].flags;
    }
}

/// Eine frische Fensterflaeche.
///
/// Gemessen: `w.get_surface().get_flags()` ist `16777216`
/// ([`PREALLOC`]), auch im Vollbild, und `get_alpha()` ist `None` -
/// dasselbe Bild wie bei der Anzeigeflaeche von `display.set_mode`.
fn anzeigeflaeche(groesse: (u32, u32), flags: u32) -> Surface {
    let mut s = Surface::with_flags(groesse, PREALLOC | (flags & FULLSCREEN));
    s.fill((0, 0, 0));
    s
}

/// Gibt dem Fenster den Fokus und nimmt ihn allen anderen.
///
/// Gemessen: bei drei Fenstern hat nur das zuletzt angelegte `focused`
/// auf wahr. Legt dazu die fuenf Eroeffnungsereignisse ab.
fn fokus_uebernehmen(z: &mut Zustand, id: u32) {
    for e in z.fenster.eintraege.iter_mut() {
        e.fokus = e.id == id;
    }
    z.eingabe.tast_fokus = true;
    z.eingabe.maus_fokus = true;
    // Der Griff haengt am Fokus - siehe `haelt_griff`. Ein neues
    // Fenster nimmt dem vorigen den Griff weg, ohne selbst zu greifen.
    griff_aller_nachfuehren(z);
    for e in eroeffnungsereignisse() {
        z.schlange.ablegen(e);
    }
}

/// Die fuenf Ereignisse, die ein Fenster beim Aufgehen ablegt.
///
/// Dieselben fuenf wie bei `display.set_mode` - gemessen, Reihenfolge und
/// Felder.
fn eroeffnungsereignisse() -> [Event; 5] {
    [
        Event::mit(
            ACTIVEEVENT,
            Daten::Aktiv {
                gain: 1,
                state: APPMOUSEFOCUS as i32,
            },
        ),
        Event::neu(WINDOWENTER),
        Event::mit(
            ACTIVEEVENT,
            Daten::Aktiv {
                gain: 1,
                state: APPINPUTFOCUS as i32,
            },
        ),
        Event::neu(WINDOWFOCUSGAINED),
        Event::neu(WINDOWSHOWN),
    ]
}

/// Die fuenf Gegenereignisse beim Verstecken oder Zumachen.
///
/// Gemessen an `w.hide()` und `w.destroy()`: dieselbe Folge wie beim
/// Aufgehen, nur mit `gain = 0` und den Gegenstuecken -
/// `WINDOWLEAVE`, `WINDOWFOCUSLOST`, `WINDOWHIDDEN`.
fn schliessereignisse(z: &mut Zustand, id: u32) {
    if let Ok(i) = z.fenster.stelle_von(id) {
        z.fenster.eintraege[i].fokus = false;
    }
    for e in [
        Event::mit(
            ACTIVEEVENT,
            Daten::Aktiv {
                gain: 0,
                state: APPMOUSEFOCUS as i32,
            },
        ),
        Event::neu(WINDOWLEAVE),
        Event::mit(
            ACTIVEEVENT,
            Daten::Aktiv {
                gain: 0,
                state: APPINPUTFOCUS as i32,
            },
        ),
        Event::neu(WINDOWFOCUSLOST),
        Event::neu(WINDOWHIDDEN),
    ] {
        z.schlange.ablegen(e);
    }
}

/// Der Eintrag des Anzeigefensters - legt ihn an, wenn es ihn noch nicht
/// gibt.
///
/// Gerufen wird das aus [`crate::display::set_mode`] und aus
/// [`Fenster::vom_anzeigemodul`]. Beide Wege sollen **denselben**
/// Eintrag treffen, damit `set_mode` und `Window` dieselbe Flaeche sehen.
pub(crate) fn anzeigefenster(z: &mut Zustand) -> Option<Fenster> {
    z.backend.as_ref()?;
    if let Some(e) = z
        .fenster
        .eintraege
        .iter_mut()
        .find(|e| matches!(e.stueck, Stueck::Anzeige))
    {
        let id = e.id;
        // Groesse und Titel koennen sich durch `set_mode` geaendert
        // haben - der Eintrag zieht nach.
        if let Some(s) = z.screen.as_ref() {
            let g = s.try_read().expect(GELEIHT).get_size();
            let i = z.fenster.stelle_von(id).expect("gerade gefunden");
            z.fenster.eintraege[i].groesse = g;
            if z.fenster.eintraege[i].flags & FULLSCREEN == 0 {
                z.fenster.eintraege[i].fenstergroesse = g;
            }
        }
        let i = z.fenster.stelle_von(id).expect("gerade gefunden");
        z.fenster.eintraege[i].titel = z.caption.0.clone();
        z.fenster.eintraege[i].flags = z.flags;
        return Some(Fenster { id });
    }

    let groesse = z.screen.as_ref()?.try_read().expect(GELEIHT).get_size();
    let id = z.fenster.naechste_id;
    z.fenster.naechste_id += 1;
    let position = mittig(groesse, z);
    z.fenster.eintraege.push(Eintrag {
        id,
        titel: z.caption.0.clone(),
        groesse,
        fenstergroesse: groesse,
        position,
        mindest: (0, 0),
        hoechst: (0, 0),
        maus_rechteck: None,
        flags: z.flags,
        obenauf: false,
        hilfsfenster: false,
        fokus: true,
        will_maus: false,
        will_tastatur: false,
        griff_wirkt: false,
        stueck: Stueck::Anzeige,
    });
    for e in z.fenster.eintraege.iter_mut() {
        e.fokus = e.id == id;
    }
    // Das neue Fenster nimmt den anderen den Fokus - und damit den Griff.
    griff_aller_nachfuehren(z);
    Some(Fenster { id })
}

/// Pumpt die Fenster, die ihr eigenes Stueck haben.
///
/// Das Anzeigefenster pumpt [`crate::event`] selbst ueber
/// [`Zustand::backend`]; hierher kommen nur die anderen.
///
/// **Eine Einschraenkung, die man kennen muss:** alle Fenster schreiben
/// in **eine** [`crate::backend::Eingabe`] und in **eine** Schlange, und
/// die Ereignisse dieser Kiste tragen keine Fensternummer (pygame legt
/// bei `WINDOWSIZECHANGED` ein Feld `window` dazu - gemessen). Mit zwei
/// echten minifb-Fenstern ueberschreiben sich deren Fokusstaende
/// gegenseitig. Ohne Fenster - dem Hauptfall dieser Kiste - kann das
/// nicht passieren, weil dort nur ankommt, was jemand selbst einspeist.
pub(crate) fn eigene_pumpen(z: &mut Zustand, heraus: &mut Vec<Event>) {
    for e in z.fenster.eintraege.iter_mut() {
        if let Stueck::Eigen { backend, .. } = &mut e.stueck {
            backend.pumpen(&mut z.eingabe, heraus);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display;
    use crate::zustand::Treiber;

    fn vorbereiten() {
        display::set_treiber(Treiber::Dummy);
        display::init().unwrap();
    }

    #[test]
    fn vorgaben_wie_gemessen() {
        vorbereiten();
        let w = Fenster::neu(&Wunsch::default()).unwrap();
        assert_eq!(w.titel().unwrap(), "pygame window");
        assert_eq!(w.groesse().unwrap(), (640, 480));
        // Gemessen: (1024-640)/2, (768-480)/2
        assert_eq!(w.position().unwrap(), (192, 144));
        assert_eq!(w.id().unwrap(), 1);
        assert!(w.fokussiert().unwrap());
        assert_eq!(w.mindestgroesse().unwrap(), (0, 0));
        assert_eq!(w.hoechstgroesse().unwrap(), (0, 0));
        assert_eq!(w.maus_rechteck().unwrap(), None);
        assert_eq!(w.deckkraft().unwrap(), 1.0);
        assert!(!w.opengl().unwrap());
        assert!(!w.hilfsfenster().unwrap());
        assert!(!w.rahmenlos().unwrap());
        assert!(!w.groessenaenderbar().unwrap());
        assert!(!w.obenauf().unwrap());
    }

    #[test]
    fn groesse_null_ist_ein_wertfehler() {
        vorbereiten();
        // Gemessen: der Konstruktor setzt einen Punkt, der Setzer nicht.
        assert_eq!(
            Fenster::neu(&Wunsch::neu("T", (0, 48))).unwrap_err(),
            Fehler::Wert("width or height should not be less than or equal to zero.")
        );
        let w = Fenster::neu(&Wunsch::neu("T", (64, 48))).unwrap();
        assert_eq!(
            w.set_groesse((0, 48)).unwrap_err(),
            Fehler::Wert("width or height should not be less than or equal to zero")
        );
    }

    #[test]
    fn nummern_zaehlen_hoch_und_werden_nicht_wiederverwendet() {
        vorbereiten();
        let a = Fenster::neu(&Wunsch::neu("A", (64, 48))).unwrap();
        let b = Fenster::neu(&Wunsch::neu("B", (32, 24))).unwrap();
        assert_eq!((a.id().unwrap(), b.id().unwrap()), (1, 2));
        a.zerstoeren();
        let c = Fenster::neu(&Wunsch::neu("C", (16, 12))).unwrap();
        assert_eq!(c.id().unwrap(), 3);
    }

    #[test]
    fn nur_das_letzte_fenster_hat_den_fokus() {
        vorbereiten();
        let a = Fenster::neu(&Wunsch::neu("A", (64, 48))).unwrap();
        let b = Fenster::neu(&Wunsch::neu("B", (32, 24))).unwrap();
        let c = Fenster::neu(&Wunsch::neu("C", (16, 12))).unwrap();
        assert_eq!(
            (
                a.fokussiert().unwrap(),
                b.fokussiert().unwrap(),
                c.fokussiert().unwrap()
            ),
            (false, false, true)
        );
    }

    #[test]
    fn verstecktes_fenster_nimmt_keinen_fokus() {
        vorbereiten();
        let a = Fenster::neu(&Wunsch::neu("A", (64, 48))).unwrap();
        let b = Fenster::neu(&Wunsch {
            versteckt: true,
            ..Wunsch::neu("B", (32, 24))
        })
        .unwrap();
        assert_eq!(
            (a.fokussiert().unwrap(), b.fokussiert().unwrap()),
            (true, false)
        );
        b.zeigen().unwrap();
        assert!(b.fokussiert().unwrap());
        b.verstecken().unwrap();
        assert!(!b.fokussiert().unwrap());
    }

    #[test]
    fn flaeche_bleibt_dieselbe_ueber_den_groessenwechsel() {
        vorbereiten();
        let w = Fenster::neu(&Wunsch::neu("T", (64, 48))).unwrap();
        let a = w.flaeche().unwrap();
        let b = w.flaeche().unwrap();
        assert!(Arc::ptr_eq(&a, &b));
        assert_eq!(a.read().get_size(), (64, 48));
        assert_eq!(a.read().get_flags(), PREALLOC);

        w.set_groesse((100, 80)).unwrap();
        let c = w.flaeche().unwrap();
        assert!(Arc::ptr_eq(&a, &c));
        assert_eq!(a.read().get_size(), (100, 80));
    }

    #[test]
    fn flip_ohne_flaeche_ist_ein_fehler() {
        vorbereiten();
        let w = Fenster::neu(&Wunsch::neu("T", (64, 48))).unwrap();
        assert!(matches!(w.flip(), Err(Fehler::Pygame(_))));
        w.flaeche().unwrap();
        w.flip().unwrap();
    }

    #[test]
    fn zerstoeren_ist_zweimal_erlaubt_und_danach_ist_alles_ungueltig() {
        vorbereiten();
        let w = Fenster::neu(&Wunsch::neu("T", (64, 48))).unwrap();
        w.zerstoeren();
        w.zerstoeren();
        assert!(!w.lebt());
        assert_eq!(w.id().unwrap_err(), Fehler::Pygame("Invalid window"));
        assert_eq!(w.titel().unwrap_err(), Fehler::Pygame("Invalid window"));
        assert_eq!(w.groesse().unwrap_err(), Fehler::Pygame("Invalid window"));
        assert_eq!(w.flaeche().unwrap_err(), Fehler::Pygame("Invalid window"));
    }

    #[test]
    fn vollbild_nimmt_die_arbeitsflaeche_und_kommt_zurueck() {
        vorbereiten();
        let w = Fenster::neu(&Wunsch::neu("T", (64, 48))).unwrap();
        let s = w.flaeche().unwrap();
        w.set_vollbild(true).unwrap();
        assert_eq!(w.groesse().unwrap(), crate::backend::dummy::DESKTOP);
        assert_eq!(s.read().get_size(), crate::backend::dummy::DESKTOP);
        assert!(w.vollbild().unwrap());

        // Gemessen: im Vollbild gesetzt, nach dem Zurueckschalten gueltig.
        w.set_groesse((200, 150)).unwrap();
        assert_eq!(w.groesse().unwrap(), crate::backend::dummy::DESKTOP);
        w.set_vollbild(false).unwrap();
        assert_eq!(w.groesse().unwrap(), (200, 150));
        assert!(Arc::ptr_eq(&s, &w.flaeche().unwrap()));
    }

    #[test]
    fn vollbild_im_konstruktor() {
        vorbereiten();
        let w = Fenster::neu(&Wunsch {
            vollbild: true,
            ..Wunsch::neu("T", (64, 48))
        })
        .unwrap();
        assert_eq!(w.groesse().unwrap(), crate::backend::dummy::DESKTOP);
        w.set_vollbild(false).unwrap();
        assert_eq!(w.groesse().unwrap(), (64, 48));
    }

    #[test]
    fn mindest_und_hoechstgroesse_wie_gemessen() {
        vorbereiten();
        let w = Fenster::neu(&Wunsch {
            groessenaenderbar: true,
            ..Wunsch::neu("T", (64, 48))
        })
        .unwrap();

        assert_eq!(
            w.set_mindestgroesse((-1, 10)).unwrap_err(),
            Fehler::Wert("minimum width or height should not be less than zero")
        );
        assert_eq!(
            w.set_hoechstgroesse((-1, 10)).unwrap_err(),
            Fehler::Wert("maximum width or height should not be less than zero")
        );

        // Gemessen: die Grenze zieht das Fenster mit.
        w.set_mindestgroesse((200, 200)).unwrap();
        assert_eq!(w.mindestgroesse().unwrap(), (200, 200));
        assert_eq!(w.groesse().unwrap(), (200, 200));

        // Gemessen: min groesser als max ist ein Fehler ...
        w.set_hoechstgroesse((300, 300)).unwrap();
        assert!(w.set_mindestgroesse((400, 400)).is_err());
        // ... und max kleiner als min ebenso.
        assert!(w.set_hoechstgroesse((10, 10)).is_err());

        // Gemessen: (0, 0) setzt nichts zurueck.
        w.set_mindestgroesse((0, 0)).unwrap();
        assert_eq!(w.mindestgroesse().unwrap(), (200, 200));
    }

    #[test]
    fn maus_rechteck_wird_bei_leerer_flaeche_zu_nichts() {
        vorbereiten();
        let w = Fenster::neu(&Wunsch::neu("T", (64, 48))).unwrap();
        w.set_maus_rechteck(Some(Rect::new(1, 2, 3, 4))).unwrap();
        assert_eq!(w.maus_rechteck().unwrap(), Some(Rect::new(1, 2, 3, 4)));
        // Gemessen: Breite oder Hoehe null oder negativ -> None.
        w.set_maus_rechteck(Some(Rect::new(0, 0, 0, 0))).unwrap();
        assert_eq!(w.maus_rechteck().unwrap(), None);
        w.set_maus_rechteck(Some(Rect::new(0, 0, -5, 10))).unwrap();
        assert_eq!(w.maus_rechteck().unwrap(), None);
        // Ausserhalb des Fensters ist dagegen erlaubt.
        w.set_maus_rechteck(Some(Rect::new(100, 100, 10, 10)))
            .unwrap();
        assert_eq!(w.maus_rechteck().unwrap(), Some(Rect::new(100, 100, 10, 10)));
    }

    #[test]
    fn nur_das_fenster_im_fokus_greift() {
        vorbereiten();
        // Ein einzelnes Fenster hat den Fokus und greift.
        let a = Fenster::neu(&Wunsch::neu("A", (64, 48))).unwrap();
        a.set_griff_maus(true).unwrap();
        assert!(a.haelt_maus().unwrap());
        assert_eq!(get_grabbed_window().map(|f| f.id().unwrap()), Some(1));

        // Gemessen: ein neues Fenster nimmt den Fokus - und damit den
        // Griff. Der **Wunsch** bleibt stehen.
        let b = Fenster::neu(&Wunsch::neu("B", (32, 24))).unwrap();
        assert!(a.will_maus().unwrap());
        assert!(!a.haelt_maus().unwrap());
        assert!(get_grabbed_window().is_none());

        // Gemessen: greift b, dann greift b - und a weiterhin nicht.
        b.set_griff_tastatur(true).unwrap();
        assert!(b.haelt_tastatur().unwrap());
        assert!(!b.haelt_maus().unwrap());
        assert!(!a.haelt_maus().unwrap());
        assert_eq!(get_grabbed_window().map(|f| f.id().unwrap()), Some(2));

        // Gemessen: destroy gibt frei.
        b.zerstoeren();
        assert!(get_grabbed_window().is_none());
    }

    #[test]
    fn griff_ohne_fokus_wirkt_nicht_und_kommt_mit_dem_fokus_zurueck() {
        vorbereiten();
        let a = Fenster::neu(&Wunsch::neu("A", (64, 48))).unwrap();
        let b = Fenster::neu(&Wunsch::neu("B", (32, 24))).unwrap();
        // b hat den Fokus, also greift a nicht - gemessen `(True, False)`.
        a.set_griff_maus(true).unwrap();
        assert!(a.will_maus().unwrap());
        assert!(!a.haelt_maus().unwrap());
        assert!(get_grabbed_window().is_none());

        // Gemessen: `focus()` verschiebt den Fokus unter dem
        // Dummy-Treiber **nicht**, also aendert es daran nichts.
        a.nach_vorn(false).unwrap();
        assert!(!a.fokussiert().unwrap());
        assert!(b.fokussiert().unwrap());
        assert!(!a.haelt_maus().unwrap());

        // Aber ueber Verstecken und Zeigen kommt der Fokus zu a - und mit
        // ihm der Griff, ohne dass der Wunsch neu gesetzt werden muss.
        a.zeigen().unwrap();
        assert!(a.fokussiert().unwrap());
        assert!(a.haelt_maus().unwrap());
        assert_eq!(get_grabbed_window().map(|f| f.id().unwrap()), Some(1));
    }

    #[test]
    fn verstecken_nimmt_den_griff_und_zeigen_gibt_ihn_zurueck() {
        vorbereiten();
        let w = Fenster::neu(&Wunsch::neu("W", (64, 48))).unwrap();
        w.set_griff_maus(true).unwrap();
        assert!(w.haelt_maus().unwrap());
        w.verstecken().unwrap();
        // Gemessen: Wunsch bleibt, Griff ist weg.
        assert!(w.will_maus().unwrap());
        assert!(!w.haelt_maus().unwrap());
        assert!(get_grabbed_window().is_none());
        w.zeigen().unwrap();
        assert!(w.haelt_maus().unwrap());
    }

    #[test]
    fn griff_aus_dem_konstruktor() {
        vorbereiten();
        // Gemessen: `Window(.., mouse_grabbed=True, keyboard_grabbed=True)`
        // setzt beide Wuensche, und weil das Fenster den Fokus bekommt,
        // gelten sie auch.
        let w = Fenster::neu(&Wunsch {
            griff_maus: true,
            griff_tastatur: true,
            ..Wunsch::neu("W", (64, 48))
        })
        .unwrap();
        assert!(w.will_maus().unwrap());
        assert!(w.will_tastatur().unwrap());
        assert!(w.haelt_maus().unwrap());
        assert!(w.haelt_tastatur().unwrap());

        // Gemessen: mit `hidden=True` bleibt der Wunsch, der Griff nicht.
        let v = Fenster::neu(&Wunsch {
            griff_maus: true,
            versteckt: true,
            ..Wunsch::neu("V", (32, 24))
        })
        .unwrap();
        assert!(!v.fokussiert().unwrap());
        assert!(v.will_maus().unwrap());
        assert!(!v.haelt_maus().unwrap());
    }

    #[test]
    fn was_kein_stueck_kann_meldet_einen_grund() {
        vorbereiten();
        let w = Fenster::neu(&Wunsch::neu("T", (64, 48))).unwrap();
        for f in [
            w.set_deckkraft(0.5),
            w.blinken(FLASH_BRIEFLY),
            w.modal_fuer(w),
            w.nach_vorn(true),
        ] {
            assert!(matches!(f, Err(Fehler::NichtUnterstuetzt(_))), "{f:?}");
        }
        // Eine ungueltige Blinkart wird **vor** dem Stueck geprueft.
        assert_eq!(
            w.blinken(99).unwrap_err(),
            Fehler::Wert("Unsupported window flash operation.")
        );
        // Und das hier geht ohne Fenster - gemessen.
        w.nach_vorn(false).unwrap();
        w.zustand_setzen(Fensterzustand::Abgelegt).unwrap();
        w.zustand_setzen(Fensterzustand::Aufgezogen).unwrap();
        w.zustand_setzen(Fensterzustand::Gewoehnlich).unwrap();
        assert_eq!(w.groesse().unwrap(), (64, 48));
    }

    #[test]
    fn opengl_und_vulkan_werden_abgelehnt() {
        vorbereiten();
        // Gemessen: echtes pygame wirft dort ebenfalls, nur mit einem
        // Text, der den SDL-Treibernamen nennt.
        for wunsch in [
            Wunsch {
                opengl: true,
                ..Wunsch::neu("T", (64, 48))
            },
            Wunsch {
                vulkan: true,
                ..Wunsch::neu("T", (64, 48))
            },
        ] {
            assert!(matches!(
                Fenster::neu(&wunsch),
                Err(Fehler::NichtUnterstuetzt(_))
            ));
        }
    }

    #[test]
    fn hilfsfenster_wird_angelegt_und_ohne_fenster_gemeldet() {
        vorbereiten();
        // Gemessen: pygame legt es an und meldet `utility` als wahr.
        // Vorher wies diese Kiste den Wunsch ab - zu streng, denn das
        // kostete das ganze Fenster statt bloss den Leisteneintrag.
        let w = Fenster::neu(&Wunsch {
            hilfsfenster: true,
            ..Wunsch::neu("T", (64, 48))
        })
        .unwrap();
        assert_eq!(w.groesse().unwrap(), (64, 48));
        assert!(w.hilfsfenster().unwrap());
        // Gemessen: `utility` zieht `borderless` und `resizable` nicht mit.
        assert!(!w.rahmenlos().unwrap());
        assert!(!w.groessenaenderbar().unwrap());

        let v = Fenster::neu(&Wunsch::neu("U", (32, 24))).unwrap();
        assert!(!v.hilfsfenster().unwrap());
    }

    #[test]
    fn always_on_top_aus_dem_konstruktor_haelt() {
        vorbereiten();
        // Gemessen: das Wort im Konstruktor haelt auch ohne Fenster - SDL
        // gibt die Anlegeflagge zurueck. Hier fiel es vorher weg, weil
        // der Wunsch am `set_obenauf` des fensterlosen Stuecks haengte.
        let w = Fenster::neu(&Wunsch {
            obenauf: true,
            ..Wunsch::neu("T", (64, 48))
        })
        .unwrap();
        assert!(w.obenauf().unwrap());
        // Und gemessen: der Setzer bleibt wirkungslos, in beide Richtungen.
        w.set_obenauf(false).unwrap();
        assert!(w.obenauf().unwrap());
    }

    #[test]
    fn rahmen_bleibt_ohne_fenster_stehen() {
        vorbereiten();
        // Gemessen an pygame unter dem Dummy-Treiber: der Konstruktorwert
        // gilt, der Setzer bewirkt nichts und wirft auch nicht.
        let w = Fenster::neu(&Wunsch {
            rahmenlos: true,
            groessenaenderbar: true,
            ..Wunsch::neu("T", (64, 48))
        })
        .unwrap();
        assert!(w.rahmenlos().unwrap());
        assert!(w.groessenaenderbar().unwrap());
        w.set_rahmenlos(false).unwrap();
        w.set_groessenaenderbar(false).unwrap();
        assert!(w.rahmenlos().unwrap());
        assert!(w.groessenaenderbar().unwrap());

        let v = Fenster::neu(&Wunsch::neu("U", (64, 48))).unwrap();
        v.set_obenauf(true).unwrap();
        assert!(!v.obenauf().unwrap());
    }

    #[test]
    fn anzeigefenster_teilt_die_flaeche_mit_set_mode() {
        vorbereiten();
        let schirm = display::set_mode((100, 80), 0).unwrap();
        let w = Fenster::vom_anzeigemodul().unwrap();
        assert!(Arc::ptr_eq(&schirm, &w.flaeche().unwrap()));
        assert_eq!(w.groesse().unwrap(), (100, 80));
        assert_eq!(w.titel().unwrap(), "pygame window");

        // Gemessen: derselbe Eintrag, nicht jedes Mal ein neuer.
        let v = Fenster::vom_anzeigemodul().unwrap();
        assert_eq!(v.id().unwrap(), w.id().unwrap());

        // Gemessen: der Titel geht in beide Richtungen.
        w.set_titel("Anders").unwrap();
        assert_eq!(display::get_caption().0, "Anders");

        // **Anders als pygame**: dort ersetzt ein Groessenwechsel ueber
        // `Window.size` das Flaechenobjekt des Anzeigemoduls und macht
        // das alte (0, 0) gross. Hier bleibt es dasselbe - das ist die
        // Zusage von `Screen`, und `set_mode` haelt sie ebenso.
        w.set_groesse((50, 40)).unwrap();
        assert!(Arc::ptr_eq(&schirm, &display::get_surface().unwrap()));
        assert_eq!(schirm.read().get_size(), (50, 40));
    }

    #[test]
    fn ohne_set_mode_gibt_es_kein_anzeigefenster() {
        vorbereiten();
        assert_eq!(
            Fenster::vom_anzeigemodul().unwrap_err(),
            Fehler::Pygame("display.set_mode has not been called yet.")
        );
    }

    #[test]
    fn eigenes_fenster_zuerst_dann_set_mode_gibt_zwei() {
        vorbereiten();
        // Gemessen: `set_mode` macht ein **eigenes** Fenster auf, und
        // `from_display_module()` liefert dessen Nummer - nicht die des
        // zuerst angelegten.
        let w = Fenster::neu(&Wunsch::neu("W", (64, 48))).unwrap();
        display::set_mode((100, 80), 0).unwrap();
        let d = Fenster::vom_anzeigemodul().unwrap();
        assert_ne!(d.id().unwrap(), w.id().unwrap());
        assert_eq!(d.groesse().unwrap(), (100, 80));
        assert_eq!(w.groesse().unwrap(), (64, 48));
    }

    #[test]
    fn anzeigefenster_zerstoeren_raeumt_das_anzeigemodul() {
        vorbereiten();
        display::set_mode((100, 80), 0).unwrap();
        let w = Fenster::vom_anzeigemodul().unwrap();
        w.zerstoeren();
        assert!(display::get_surface().is_none());
        // Gemessen: ein neues `set_mode` geht danach wieder.
        let s = display::set_mode((60, 40), 0).unwrap();
        assert_eq!(s.read().get_size(), (60, 40));
    }

    #[test]
    fn display_quit_macht_jedes_fenster_ungueltig() {
        vorbereiten();
        let w = Fenster::neu(&Wunsch::neu("T", (64, 48))).unwrap();
        display::quit();
        assert!(!w.lebt());
        // pygame liest hier freigegebenen Speicher; hier gibt es einen
        // Fehler - siehe `Ablage::leeren`.
        assert_eq!(w.groesse().unwrap_err(), Fehler::Pygame("Invalid window"));
    }

    #[test]
    fn groessenwechsel_legt_genau_ein_ereignis_ab() {
        vorbereiten();
        let w = Fenster::neu(&Wunsch::neu("T", (64, 48))).unwrap();
        crate::event::clear(None);
        w.set_groesse((100, 80)).unwrap();
        let alle = crate::event::get();
        assert_eq!(alle.len(), 1);
        assert_eq!(alle[0].typ, WINDOWSIZECHANGED);
        assert!(matches!(alle[0].daten, Daten::Fenster { x: 100, y: 80 }));
    }

    #[test]
    fn anlegen_und_verstecken_legen_die_gemessenen_fuenf_ab() {
        vorbereiten();
        crate::event::clear(None);
        let w = Fenster::neu(&Wunsch::neu("T", (64, 48))).unwrap();
        let auf: Vec<u32> = crate::event::get().iter().map(|e| e.typ).collect();
        assert_eq!(
            auf,
            vec![
                ACTIVEEVENT,
                WINDOWENTER,
                ACTIVEEVENT,
                WINDOWFOCUSGAINED,
                WINDOWSHOWN
            ]
        );
        w.verstecken().unwrap();
        let zu: Vec<u32> = crate::event::get().iter().map(|e| e.typ).collect();
        assert_eq!(
            zu,
            vec![
                ACTIVEEVENT,
                WINDOWLEAVE,
                ACTIVEEVENT,
                WINDOWFOCUSLOST,
                WINDOWHIDDEN
            ]
        );
    }

    #[test]
    fn stelle_aus_zahl_kennt_die_beiden_konstanten() {
        assert_eq!(Stelle::aus_zahl(WINDOWPOS_CENTERED as i64), Stelle::Mittig);
        assert_eq!(Stelle::aus_zahl(WINDOWPOS_UNDEFINED as i64), Stelle::Offen);
        assert_eq!(Stelle::aus_zahl(7), Stelle::Genau((7, 7)));
    }

    #[test]
    fn stelle_genau_kommt_genauso_wieder_heraus() {
        vorbereiten();
        let w = Fenster::neu(&Wunsch {
            stelle: Stelle::Genau((10, 20)),
            ..Wunsch::neu("T", (64, 48))
        })
        .unwrap();
        assert_eq!(w.position().unwrap(), (10, 20));
        // Gemessen: negative Werte sind erlaubt.
        w.set_position(Stelle::Genau((-50, -50))).unwrap();
        assert_eq!(w.position().unwrap(), (-50, -50));
        w.set_position(Stelle::Mittig).unwrap();
        assert_eq!(w.position().unwrap(), (480, 360));
    }

    #[test]
    fn ohne_griff_gibt_es_kein_gegriffenes_fenster() {
        vorbereiten();
        // Gemessen - und der Aufruf, der echtes pygame hier abstuerzen
        // laesst, wenn das Video-Teilsystem nicht laeuft.
        assert!(get_grabbed_window().is_none());
        let _w = Fenster::neu(&Wunsch::neu("T", (64, 48))).unwrap();
        assert!(get_grabbed_window().is_none());
    }
}
