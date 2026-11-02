//! event.rs
//! ========
//! Gegenstueck zu `pygame.event`: die Ereignisschlange.
//!
//! ## Was an echtem pygame gemessen wurde
//!
//! | Frage | Antwort |
//! |---|---|
//! | `get()` auf leerer Schlange | leere Liste, **kein** Blockieren |
//! | `poll()` auf leerer Schlange | ein Ereignis vom Typ `NOEVENT` (0), nicht `None` |
//! | `peek()` auf leerer Schlange | `False` - ein Wahrheitswert, keine Liste |
//! | Reihenfolge | erst herein, erst heraus |
//! | `post` eines gesperrten Typs | wird stillschweigend verworfen |
//! | `set_blocked(None)` | sperrt **alles** |
//! | `set_allowed(None)` | gibt **alles** frei |
//! | `event_name` eines unbekannten Typs | `"Unknown"` |
//! | `event_name(USEREVENT + 3)` | `"UserEvent"` - alles oberhalb zaehlt dazu |
//! | vor `display.init()` | Fehler `video system not initialized` |
//! | `set_grab(True)` **ohne** Fenster | `None` - und `get_grab()` bleibt `False` |
//! | `set_grab(True)` **mit** Fenster | `get_grab()` ist danach `True`, auch unter `dummy` |
//! | `get_grab()` vor `display.init()` | Fehler `video system not initialized` |
//!
//! Und der Fund, den man nicht raet: **`set_mode` legt von sich aus fuenf
//! Ereignisse in die Schlange**, in dieser Reihenfolge -
//! `ActiveEvent(gain=1, state=1)`, `WindowEnter`,
//! `ActiveEvent(gain=1, state=2)`, `WindowFocusGained`, `WindowShown`.
//! Wer nach dem ersten `set_mode` auf `QUIT` wartet und dabei jedes
//! unbekannte Ereignis als Abbruch wertet, liegt sofort daneben.
//! [`crate::display::set_mode`] legt dieselben fuenf ab.
//!
//! ## Ein Blick in die Schlange ist ein Durchlauf
//!
//! [`get`], [`poll`], [`peek`], [`pump`], [`clear`] und [`wait`] gehen
//! alle durch `durchlauf`, und das ist mehr als nur "das Fenster
//! fragen":
//!
//! 1. Die Kantentabellen werden geleert - [`crate::key::get_just_pressed`]
//!    und [`crate::mouse::get_just_pressed`] fangen von vorn an. Genau
//!    diesen Zeitpunkt nennt pygame: "updated when new events are
//!    processed".
//! 2. **Die faelligen Zeitgeber werden zugestellt.**
//!    [`rustgame_core::time::set_timer`] fuehrt nur Buch; wer die
//!    Ereignisse in die Schlange legt, ist diese Kiste. Ohne das waere
//!    `set_timer` eine Buchhaltung ohne Wirkung - man setzt ihn, und es
//!    passiert nie etwas.
//! 3. Dann erst wird das Fenster abgefragt.
//!
//! Die Zustandsfragen ([`crate::key::get_pressed`],
//! [`crate::mouse::get_pos`]) nehmen den kuerzeren Weg `pumpen`: sie
//! fragen das Fenster, ruehren die Kantentabellen aber nicht an. Sonst
//! wuerde ein `key::get_pressed()` zwischen `event::get()` und
//! `key::get_just_pressed()` den Vermerk loeschen, den der Aufrufer
//! gerade lesen will.

use std::collections::VecDeque;

use crate::konstanten::*;
use crate::zustand::{mit_zustand, Zustand};
use crate::{Ergebnis, Fehler};

/// Die Felder, die an einem Ereignis haengen.
///
/// pygame haengt je Typ andere Felder an ein sonst gleiches Objekt
/// (`e.key`, `e.pos`, `e.button`, ...). In Rust waere ein Struct mit
/// zwanzig meist leeren Feldern die schlechtere Wahl, also steht hier ein
/// Enum - die Feldnamen sind die von pygame, damit nichts uebersetzt
/// werden muss.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Daten {
    /// Keine Felder: `QUIT`, `VIDEOEXPOSE`, ...
    #[default]
    Leer,
    /// `KEYDOWN` / `KEYUP`.
    Taste {
        /// `e.key` - der Tastencode (`K_*`).
        key: Key,
        /// `e.scancode` - der Tastenplatz (`KSCAN_*`).
        scancode: Scancode,
        /// `e.mod` - die gehaltenen Zusatztasten. Heisst hier `mods`,
        /// weil `mod` in Rust ein Schluesselwort ist.
        mods: KeyMod,
        /// `e.unicode` - das erzeugte Zeichen, oft leer.
        ///
        /// Gemessen: bei `KEYUP` hat pygame dieses Feld gar nicht, bei
        /// `KEYDOWN` ist es bei Steuertasten die leere Zeichenkette.
        unicode: String,
    },
    /// `TEXTINPUT`.
    Text {
        /// `e.text` - das eingegebene Stueck Text.
        text: String,
    },
    /// `MOUSEMOTION`.
    MausBewegung {
        /// `e.pos` - wo der Zeiger jetzt ist.
        pos: (i32, i32),
        /// `e.rel` - wie weit er sich seit dem letzten Mal bewegt hat.
        rel: (i32, i32),
        /// `e.buttons` - welche Tasten dabei unten sind (links, mitte, rechts).
        buttons: (bool, bool, bool),
        /// `e.touch` - kam das von einem Finger statt einer Maus?
        touch: bool,
    },
    /// `MOUSEBUTTONDOWN` / `MOUSEBUTTONUP`.
    MausTaste {
        /// `e.pos` - wo geklickt wurde.
        pos: (i32, i32),
        /// `e.button` - 1 links, 2 mitte, 3 rechts, 4/5 Rad.
        button: u8,
        /// `e.touch`.
        touch: bool,
    },
    /// `MOUSEWHEEL`.
    MausRad {
        /// `e.x` - waagerecht, in Rasten.
        x: i32,
        /// `e.y` - senkrecht, in Rasten.
        y: i32,
        /// `e.precise_x` - dasselbe ungerundet.
        precise_x: f32,
        /// `e.precise_y`.
        precise_y: f32,
        /// `e.flipped` - hat das System die Richtung gedreht?
        flipped: bool,
    },
    /// `VIDEORESIZE`.
    Groesse {
        /// `e.size` - die neue Groesse.
        size: (u32, u32),
        /// `e.w`.
        w: i32,
        /// `e.h`.
        h: i32,
    },
    /// `WINDOWRESIZED` / `WINDOWSIZECHANGED` / `WINDOWMOVED`.
    Fenster {
        /// `e.x` - Breite bzw. neue Position.
        x: i32,
        /// `e.y` - Hoehe bzw. neue Position.
        y: i32,
    },
    /// `ACTIVEEVENT`.
    Aktiv {
        /// `e.gain` - 1 gewonnen, 0 verloren.
        gain: i32,
        /// `e.state` - welche Art Fokus (`APPMOUSEFOCUS`, `APPINPUTFOCUS`, ...).
        state: i32,
    },
    /// `AUDIODEVICEADDED` und `AUDIODEVICEREMOVED`.
    ///
    /// Gemessen legt pygame beim Hochfahren **je Tongeraet eines** dieser
    /// Ereignisse in die Schlange - erst die Ausgabe-, dann die
    /// Aufnahmegeraete, jedes mit seiner Nummer:
    ///
    /// ```text
    /// (4352, which=0, iscapture=0)   fuenf Ausgabegeraete
    /// ...
    /// (4352, which=4, iscapture=0)
    /// (4352, which=0, iscapture=1)   ein Aufnahmegeraet
    /// ```
    ///
    /// Die Nummern zaehlen **je Art** von null an - das Mikrofon ist
    /// `which=0` mit `iscapture=1`, nicht `which=5`.
    Tongeraet {
        /// `e.which` - die Nummer des Geraets in seiner Art.
        which: i32,
        /// `e.iscapture` - 1 bei einem Aufnahmegeraet, sonst 0.
        iscapture: i32,
    },
    /// `USEREVENT` und alles darueber.
    Nutzer {
        /// `e.code` - der uebliche Zusatz, den pygame-Programme anhaengen.
        code: i32,
    },
}

/// Ein Ereignis - `pygame.event.Event`.
///
/// `typ` ist pygames `e.type`; die Felder stecken in [`Daten`]. Fuer den
/// haeufigen Fall gibt es Abkuerzungen: [`Event::key`], [`Event::pos`],
/// [`Event::button`].
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    /// `e.type` - siehe [`crate::konstanten`].
    pub typ: EventType,
    /// Die Felder dieses Ereignisses.
    pub daten: Daten,
}

impl Event {
    /// Ein Ereignis ohne Felder.
    pub fn neu(typ: EventType) -> Event {
        Event {
            typ,
            daten: Daten::Leer,
        }
    }

    /// Ein Ereignis mit Feldern.
    pub fn mit(typ: EventType, daten: Daten) -> Event {
        Event { typ, daten }
    }

    /// `QUIT` - das Fenster soll zu.
    pub fn quit() -> Event {
        Event::neu(QUIT)
    }

    /// `KEYDOWN`. Der Tastenplatz wird aus dem Tastencode hergeleitet.
    pub fn key_down(key: Key, mods: KeyMod, unicode: &str) -> Event {
        Event::mit(
            KEYDOWN,
            Daten::Taste {
                key,
                scancode: crate::key::scancode_von(key),
                mods,
                unicode: unicode.to_string(),
            },
        )
    }

    /// `KEYUP`.
    pub fn key_up(key: Key, mods: KeyMod) -> Event {
        Event::mit(
            KEYUP,
            Daten::Taste {
                key,
                scancode: crate::key::scancode_von(key),
                mods,
                unicode: String::new(),
            },
        )
    }

    /// `MOUSEBUTTONDOWN`.
    pub fn mouse_button_down(pos: (i32, i32), button: u8) -> Event {
        Event::mit(
            MOUSEBUTTONDOWN,
            Daten::MausTaste {
                pos,
                button,
                touch: false,
            },
        )
    }

    /// `MOUSEBUTTONUP`.
    pub fn mouse_button_up(pos: (i32, i32), button: u8) -> Event {
        Event::mit(
            MOUSEBUTTONUP,
            Daten::MausTaste {
                pos,
                button,
                touch: false,
            },
        )
    }

    /// `MOUSEMOTION`.
    pub fn mouse_motion(pos: (i32, i32), rel: (i32, i32), buttons: (bool, bool, bool)) -> Event {
        Event::mit(
            MOUSEMOTION,
            Daten::MausBewegung {
                pos,
                rel,
                buttons,
                touch: false,
            },
        )
    }

    /// `USEREVENT` mit `code`.
    pub fn user(code: i32) -> Event {
        Event::mit(USEREVENT, Daten::Nutzer { code })
    }

    /// `e.key`, falls es das an diesem Ereignis gibt.
    pub fn key(&self) -> Option<Key> {
        match &self.daten {
            Daten::Taste { key, .. } => Some(*key),
            _ => None,
        }
    }

    /// `e.scancode`, falls vorhanden.
    pub fn scancode(&self) -> Option<Scancode> {
        match &self.daten {
            Daten::Taste { scancode, .. } => Some(*scancode),
            _ => None,
        }
    }

    /// `e.mod` - die Zusatztasten; 0, wenn das Ereignis keine fuehrt.
    pub fn mods(&self) -> KeyMod {
        match &self.daten {
            Daten::Taste { mods, .. } => *mods,
            _ => 0,
        }
    }

    /// `e.unicode` bzw. `e.text` - leer, wenn es nichts zu lesen gibt.
    pub fn unicode(&self) -> &str {
        match &self.daten {
            Daten::Taste { unicode, .. } => unicode,
            Daten::Text { text } => text,
            _ => "",
        }
    }

    /// `e.pos`, falls es das an diesem Ereignis gibt.
    pub fn pos(&self) -> Option<(i32, i32)> {
        match &self.daten {
            Daten::MausBewegung { pos, .. } | Daten::MausTaste { pos, .. } => Some(*pos),
            _ => None,
        }
    }

    /// `e.rel`, falls vorhanden.
    pub fn rel(&self) -> Option<(i32, i32)> {
        match &self.daten {
            Daten::MausBewegung { rel, .. } => Some(*rel),
            _ => None,
        }
    }

    /// `e.button`, falls vorhanden.
    pub fn button(&self) -> Option<u8> {
        match &self.daten {
            Daten::MausTaste { button, .. } => Some(*button),
            _ => None,
        }
    }

    /// `e.size`, falls vorhanden.
    pub fn size(&self) -> Option<(u32, u32)> {
        match &self.daten {
            Daten::Groesse { size, .. } => Some(*size),
            _ => None,
        }
    }

    /// `e.code`, falls vorhanden.
    pub fn code(&self) -> Option<i32> {
        match &self.daten {
            Daten::Nutzer { code } => Some(*code),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
//  Die Warteschlange
// ---------------------------------------------------------------------------

/// Die Ereignisschlange samt Sperrliste.
#[derive(Debug, Default)]
pub(crate) struct Schlange {
    pub(crate) eintraege: VecDeque<Event>,
    /// Gesperrte Typen. pygame fuehrt eine Tabelle ueber alle 65536
    /// moeglichen Typen; eine Menge ist dasselbe, nur ohne die 64 KiB.
    pub(crate) gesperrt: std::collections::HashSet<EventType>,
    /// `set_blocked(None)` sperrt pauschal alles - das laesst sich mit
    /// einer Menge nicht ausdruecken, also steht es daneben.
    pub(crate) alles_gesperrt: bool,
    /// Naechster freier Typ fuer [`custom_type`].
    pub(crate) naechster_eigener: EventType,
}

impl Schlange {
    pub(crate) fn neu() -> Schlange {
        Schlange {
            eintraege: VecDeque::new(),
            gesperrt: std::collections::HashSet::new(),
            alles_gesperrt: false,
            naechster_eigener: USEREVENT + 1,
        }
    }

    pub(crate) fn erlaubt(&self, typ: EventType) -> bool {
        // **"Alles" ist nicht jede Zahl** - siehe [`sperrbar`].
        if self.alles_gesperrt && sperrbar(typ) {
            return false;
        }
        !self.gesperrt.contains(&typ)
    }

    /// Legt ein Ereignis ab, sofern sein Typ nicht gesperrt ist.
    ///
    /// Gemessen: pygame verwirft ein gesperrtes Ereignis beim `post`
    /// stillschweigend - es taucht spaeter auch dann nicht auf, wenn der
    /// Typ inzwischen wieder freigegeben wurde.
    pub(crate) fn ablegen(&mut self, e: Event) {
        if self.erlaubt(e.typ) {
            self.eintraege.push_back(e);
        }
    }
}

/// Trifft ein Ereignis auf die Auswahl zu?
///
/// `None` heisst "alle", wie bei pygame.
fn passt(e: &Event, auswahl: Option<&[EventType]>) -> bool {
    match auswahl {
        None => true,
        Some(ts) => ts.contains(&e.typ),
    }
}

/// Stoesst den Mischer an, **bevor** der Zustand ausgeliehen wird.
///
/// In pygame laeuft SDLs Tonfaden nebenher und legt seine Endereignisse
/// von selbst in die Schlange. `rustgame-mixer` hat keinen eigenen Faden:
/// dort vergeht die Zeit genau dann, wenn jemand die Kiste anfasst
/// (siehe `zustand::mit` dort - "die Stelle, an der die Zeit vergeht").
/// Ohne diesen Anstoss kaeme `Channel.set_endevent` nie an: gemessen
/// liefert echtes pygame nach dem Ende eines Klangs `USEREVENT+n`, hier
/// blieb die Schlange leer, solange das Programm nur `event.get()` rief.
///
/// **Der Aufruf muss ausserhalb von [`mit_zustand`] stehen.** Die Senke,
/// die `rustgame_py` anmeldet, ruft `event::post` - und das leiht den
/// Zustand selbst aus. Von innen heraus waere das die uebliche
/// `RefCell`-Panik. Deshalb steht hier eine eigene Funktion und keine
/// Zeile in `durchlauf`.
fn ton_anstossen() {
    // `wartend` geht durch `zustand::mit` und fuehrt damit nach; die
    // Ereignisse selbst nimmt die Senke entgegen.
    let _ = rustgame_mixer::ereignis::wartend();
}

/// Holt alle passenden Ereignisse aus der Schlange - `pygame.event.get()`.
///
/// Ohne Auswahl kommt alles, sonst nur die genannten Typen; der Rest
/// bleibt liegen. Auf einer leeren Schlange ist das Ergebnis eine leere
/// Liste - gemessen, `get()` blockiert **nie**.
pub fn get_typen(auswahl: Option<&[EventType]>) -> Vec<Event> {
    ton_anstossen();
    mit_zustand(|z| {
        durchlauf(z);
        let Some(typen) = auswahl else {
            // Ohne Auswahl: alles, in Schlangenfolge.
            return z.schlange.eintraege.drain(..).collect();
        };
        // **Typ fuer Typ, in der Reihenfolge der gegebenen Liste** - nicht
        // in Schlangenfolge. Gemessen an einer Schlange `A B C D A B C D
        // A B C D`:
        //
        // ```text
        // get([A, C])   A A A C C C
        // get([C, A])   C C C A A A     <- die Liste entscheidet
        // get([A, A, C])A A A C C C     <- Doppelte zaehlen einmal
        // ```
        //
        // pygames `pg_event_get` laeuft die Liste durch und fragt fuer
        // jeden Typ einzeln bei SDL nach; hier lief ein Durchgang mit
        // einer Mengenpruefung, und das gab Schlangenfolge. Ein Programm,
        // das `get([QUIT, KEYDOWN])` schreibt und die Tasten erst nach
        // dem Beenden sehen will, bekam sie vorher.
        //
        // Die Doppelten erledigen sich von selbst: der erste Durchgang
        // nimmt alle Ereignisse dieses Typs heraus, der zweite findet
        // keines mehr.
        let mut heraus = Vec::new();
        let mut bleibt = std::mem::take(&mut z.schlange.eintraege);
        for t in typen {
            let mut rest = VecDeque::with_capacity(bleibt.len());
            while let Some(e) = bleibt.pop_front() {
                if e.typ == *t {
                    heraus.push(e);
                } else {
                    rest.push_back(e);
                }
            }
            bleibt = rest;
        }
        z.schlange.eintraege = bleibt;
        heraus
    })
}

/// Holt alles aus der Schlange - `pygame.event.get()`.
pub fn get() -> Vec<Event> {
    get_typen(None)
}

/// Holt alles **ausser** den genannten Typen - `event.get(None, False, ausser)`.
pub fn get_ausser(ausser: &[EventType]) -> Vec<Event> {
    ton_anstossen();
    mit_zustand(|z| {
        durchlauf(z);
        let mut heraus = Vec::new();
        let mut gehalten = Vec::new();
        while let Some(e) = z.schlange.eintraege.pop_front() {
            if ausser.contains(&e.typ) {
                gehalten.push(e);
            } else {
                heraus.push(e);
            }
        }
        // **Was zurueckbleibt, bleibt gruppiert** - und zwar in der
        // Reihenfolge der Ausnahmeliste. Gemessen an derselben Schlange
        // `A B C D A B C D A B C D`:
        //
        // ```text
        // get(exclude=[A, B])   rest A A A B B B
        // get(exclude=[B, A])   rest B B B A A A     <- die Liste wieder
        // ```
        //
        // Das ist kein Zufall, sondern dieselbe Ursache wie bei
        // [`get_typen`]: pygame holt die Ausgenommenen Typ fuer Typ
        // beiseite, nimmt dann den Rest und **legt sie danach wieder
        // hinein**. Hier blieben sie unberuehrt in Schlangenfolge
        // liegen.
        let mut neu = VecDeque::with_capacity(gehalten.len());
        for t in ausser {
            let mut rest = Vec::with_capacity(gehalten.len());
            for e in gehalten.drain(..) {
                if e.typ == *t {
                    neu.push_back(e);
                } else {
                    rest.push(e);
                }
            }
            gehalten = rest;
        }
        // Kann nicht vorkommen - jedes `gehalten` hat einen Typ aus
        // `ausser`. Steht da, damit kein Ereignis verschwinden kann,
        // falls sich das einmal aendert.
        for e in gehalten {
            neu.push_back(e);
        }
        z.schlange.eintraege = neu;
        heraus
    })
}

/// Nimmt das naechste Ereignis - `pygame.event.poll()`.
///
/// Ist nichts da, kommt ein Ereignis vom Typ [`NOEVENT`] heraus, **kein**
/// `None`. Das ist gemessen und der Grund, warum hier kein `Option` steht:
/// `poll().typ == NOEVENT` ist die Frage, die pygame-Programme stellen.
pub fn poll() -> Event {
    ton_anstossen();
    mit_zustand(|z| {
        durchlauf(z);
        z.schlange
            .eintraege
            .pop_front()
            .unwrap_or_else(|| Event::neu(NOEVENT))
    })
}

/// Wartet auf das naechste Ereignis - `pygame.event.wait()`.
///
/// `timeout` in Millisekunden; `None` wartet ohne Ende. Laeuft die Zeit
/// ab, kommt [`NOEVENT`] - gemessen: `wait(50)` auf einer leeren Schlange
/// kehrt nach rund 50 ms mit `NoEvent` zurueck.
///
/// Zwischen zwei Blicken in die Schlange schlaeft der Strang eine
/// Millisekunde. Ohne das wuerde das Warten einen Kern verheizen; mit dem
/// Dummy-Stueck koennte sonst ueberhaupt nichts mehr hereinkommen, weil
/// niemand mehr an die Reihe kaeme.
pub fn wait(timeout: Option<u32>) -> Event {
    let start = std::time::Instant::now();
    loop {
        let e = poll();
        if e.typ != NOEVENT {
            return e;
        }
        if let Some(ms) = timeout {
            if start.elapsed().as_millis() as u32 >= ms {
                return Event::neu(NOEVENT);
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}

/// Liegt etwas Passendes in der Schlange? - `pygame.event.peek()`.
///
/// Gemessen: pygame gibt hier einen Wahrheitswert zurueck, keine Liste,
/// und die Schlange bleibt unberuehrt.
pub fn peek(auswahl: Option<&[EventType]>) -> bool {
    ton_anstossen();
    mit_zustand(|z| {
        durchlauf(z);
        z.schlange.eintraege.iter().any(|e| passt(e, auswahl))
    })
}

/// Leert die Schlange - `pygame.event.clear()`.
///
/// Mit Auswahl fliegen nur die genannten Typen heraus.
pub fn clear(auswahl: Option<&[EventType]>) {
    mit_zustand(|z| {
        durchlauf(z);
        match auswahl {
            None => z.schlange.eintraege.clear(),
            Some(ts) => z.schlange.eintraege.retain(|e| !ts.contains(&e.typ)),
        }
    })
}

/// Legt ein Ereignis ab - `pygame.event.post()`.
///
/// Gemessen: ist der Typ gesperrt, wird das Ereignis stillschweigend
/// verworfen. pygame gibt in dem Fall `False` zurueck, sonst `True` -
/// genau das steht auch hier im Ergebnis.
pub fn post(e: Event) -> bool {
    mit_zustand(|z| {
        let ok = z.schlange.erlaubt(e.typ);
        z.schlange.ablegen(e);
        ok
    })
}

/// Gibt Typen frei - `pygame.event.set_allowed()`.
///
/// `None` gibt **alles** frei; gemessen an echtem pygame.
pub fn set_allowed(typen: Option<&[EventType]>) {
    mit_zustand(|z| match typen {
        None => {
            z.schlange.alles_gesperrt = false;
            z.schlange.gesperrt.clear();
        }
        Some(ts) => {
            if z.schlange.alles_gesperrt {
                // Aus "alles gesperrt" wird "alles gesperrt ausser diesen".
                z.schlange.alles_gesperrt = false;
                z.schlange.gesperrt = alle_typen();
            }
            for t in ts {
                z.schlange.gesperrt.remove(t);
            }
        }
    })
}

/// Sperrt Typen - `pygame.event.set_blocked()`.
///
/// `None` sperrt **alles**; gemessen an echtem pygame. Gesperrte
/// Ereignisse kommen gar nicht erst in die Schlange.
///
/// **Und was schon drinsteht, fliegt hinaus.** Gemessen an einer
/// Schlange mit je zehn `USEREVENT`, `KEYDOWN` und `QUIT`:
///
/// ```text
/// set_blocked(KEYDOWN)   danach 20 Ereignisse   <- die zehn sind weg
/// set_blocked(None)      danach 0
/// set_blocked([])        danach 30
/// set_allowed(...)       danach 30  (jede Form)
/// ```
///
/// Das kommt von SDL: `SDL_EventState(typ, SDL_IGNORE)` ruft
/// `SDL_FlushEvent(typ)` mit. Hier blieben die gesperrten Ereignisse
/// liegen und kamen beim naechsten `get()` heraus - ein Programm, das
/// einen Typ sperrt, bekam ihn danach trotzdem noch einmal. Freigeben
/// raeumt dagegen **nicht** aus, in keiner Form; auch das gemessen.
pub fn set_blocked(typen: Option<&[EventType]>) {
    mit_zustand(|z| match typen {
        None => {
            z.schlange.alles_gesperrt = true;
            z.schlange.gesperrt.clear();
            z.schlange.eintraege.clear();
        }
        Some(ts) => {
            for t in ts {
                z.schlange.gesperrt.insert(*t);
            }
            z.schlange.eintraege.retain(|e| !ts.contains(&e.typ));
        }
    })
}

/// Ist dieser Typ gesperrt? - `pygame.event.get_blocked()`.
pub fn get_blocked(typ: EventType) -> bool {
    mit_zustand(|z| !z.schlange.erlaubt(typ))
}

/// Alle Typen, die pygame kennt - fuer `set_allowed` nach `set_blocked(None)`.
fn alle_typen() -> std::collections::HashSet<EventType> {
    (0..NUMEVENTS).filter(|t| sperrbar(*t)).collect()
}

/// Trifft **"alles sperren"** diesen Typ?
///
/// `set_blocked(None)` sperrt nicht alle 65535 Zahlen, sondern pygames
/// eigene Tabelle. Gemessen, Zahl fuer Zahl durch den ganzen Bereich:
///
/// ```text
/// gesperrt: 32818 Typen
///   0 | 256-263 | 513 | 768-772 | 1024-1027 | 1536-1542 | 1616-1625
///   | 1792-1794 | 2048-2050 | 2304 | 4096-4099 | 4352-4353
///   | 8192-8193 | 32768-65534
/// ```
///
/// Und das ist keine Liste von Hand, sondern genau: **jeder Typ mit
/// Namen**, dazu die zwei Gesten `DOLLARGESTURE` und `DOLLARRECORD`
/// (2048 und 2049 - die kennt pygame, benennt sie aber nicht) und der
/// ganze SDL-Nutzerbereich ab `0x8000`. Nachgerechnet: von den 32818
/// gesperrten haben 32742 einen Namen, die 76 anderen sind die zwei
/// Gesten und 32791..32864, also der Teil des Nutzerbereichs unterhalb
/// von `USEREVENT`.
///
/// Was **nicht** darunterfaellt, antwortet weiter. Auch das gemessen:
///
/// ```text
/// set_blocked(None); post(Event(1))      True   get_blocked(1)   False
/// set_blocked(None); post(Event(256))    False  get_blocked(256) True
/// ```
///
/// Hier galt `(0..NUMEVENTS)`, also **alles** - ein Programm, das sich
/// einen eigenen Typ unter 256 ausdenkt, bekam ihn nach einem
/// `set_blocked(None)` nicht mehr durch.
fn sperrbar(typ: EventType) -> bool {
    /// `SDL_DOLLARGESTURE` und `SDL_DOLLARRECORD` - pygame sperrt sie,
    /// ohne sie zu benennen.
    const GESTEN: [EventType; 2] = [2048, 2049];
    /// `SDL_USEREVENT`, der Anfang des Nutzerbereichs. **Nicht**
    /// pygames `USEREVENT` (32865) - der liegt weiter oben.
    const NUTZER_AB: EventType = 0x8000;
    EVENT_NAMES.binary_search_by_key(&typ, |(t, _)| *t).is_ok()
        || GESTEN.contains(&typ)
        || (NUTZER_AB..NUMEVENTS).contains(&typ)
}

/// Holt neue Ereignisse vom Fenster - `pygame.event.pump()`.
///
/// Wer nur `get()` oder `poll()` benutzt, braucht das nie: die holen von
/// sich aus. Noetig wird es, wenn ein Programm die Schlange gar nicht
/// liest und trotzdem will, dass [`crate::key::get_pressed`] und
/// [`crate::mouse::get_pos`] aktuell bleiben - genau wie bei pygame.
///
/// Zaehlt als voller Durchlauf, stellt also auch die Zeitgeber zu und
/// setzt die Kantentabellen zurueck - siehe `durchlauf`.
pub fn pump() {
    ton_anstossen();
    mit_zustand(durchlauf)
}

// ---------------------------------------------------------------------------
//  Griff
// ---------------------------------------------------------------------------

/// Haelt Maus und Tastatur am Fenster fest - `pygame.event.set_grab()`.
///
/// Gemessen an echtem pygame, und beides ueberraschend:
///
/// * **Ohne Fenster** laeuft der Aufruf klaglos durch, wirkt aber nicht:
///   `set_grab(True)` gibt `None` zurueck, und `get_grab()` sagt danach
///   weiterhin `False`. Es gibt nichts, woran sich etwas festhalten
///   liesse.
/// * **Mit Fenster** wirkt es sogar unter `SDL_VIDEODRIVER=dummy` -
///   `get_grab()` ist danach `True`, obwohl gar kein Fenster auf dem
///   Schirm steht. SDL merkt sich den Wunsch, mehr braucht es nicht.
///
/// Deshalb steht in [`get_grab`] nicht, was verlangt wurde, sondern was
/// wirklich gilt. Bei minifb heisst das: **nie** `true`. minifb hat
/// keinen Aufruf, der den Zeiger im Fenster haelt, und ohne
/// `set_mouse_pos` (das es ebenfalls nicht hat) liesse er sich auch
/// nicht von Hand zurueckholen. Ein `true` waere dort ein Versprechen,
/// das niemand einloest.
///
/// Vor [`crate::display::init`] ist es ein Fehler - gemessen:
/// `video system not initialized`.
pub fn set_grab(an: bool) -> Ergebnis<()> {
    mit_zustand(|z| {
        if !z.init {
            return Err(Fehler::NichtInitialisiert);
        }
        z.griff = match z.backend.as_mut() {
            Some(b) => b.set_grab(an),
            None => false,
        };
        Ok(())
    })
}

/// Haelt das Fenster Maus und Tastatur fest? - `pygame.event.get_grab()`.
///
/// Gibt zurueck, was **wirklich** gilt, nicht was verlangt wurde - siehe
/// [`set_grab`].
pub fn get_grab() -> Ergebnis<bool> {
    mit_zustand(|z| {
        if !z.init {
            return Err(Fehler::NichtInitialisiert);
        }
        Ok(z.griff)
    })
}

/// Der Name eines Ereignistyps - `pygame.event.event_name()`.
///
/// Gemessen: Unbekanntes heisst `"Unknown"`, und **alles** ab
/// [`USEREVENT`] heisst `"UserEvent"` - auch `USEREVENT + 3`.
pub fn event_name(typ: EventType) -> &'static str {
    if let Ok(i) = EVENT_NAMES.binary_search_by_key(&typ, |(t, _)| *t) {
        return EVENT_NAMES[i].1;
    }
    if (USEREVENT..NUMEVENTS).contains(&typ) {
        return "UserEvent";
    }
    "Unknown"
}

/// Ein neuer, freier Ereignistyp - `pygame.event.custom_type()`.
///
/// Gemessen: der erste Aufruf liefert `USEREVENT + 1`.
pub fn custom_type() -> Ergebnis<EventType> {
    mit_zustand(|z| {
        if z.schlange.naechster_eigener >= NUMEVENTS {
            // pygames Wortlaut, gemessen beim 32669. Aufruf:
            // `pygame.error: pygame.event.custom_type made too many event
            // types.` - mit Punkt am Ende. Hier stand
            // `no unused event types left`, was dasselbe sagt und nicht
            // dasselbe ist.
            return Err(Fehler::Pygame(
                "pygame.event.custom_type made too many event types.",
            ));
        }
        let t = z.schlange.naechster_eigener;
        z.schlange.naechster_eigener += 1;
        Ok(t)
    })
}

/// Ein voller Durchlauf - der Weg, den jeder Blick in die Schlange geht.
///
/// Drei Dinge, in dieser Reihenfolge:
///
/// 1. **Kantentabellen leeren.** `get_just_pressed` und `get_just_released`
///    melden nur, was in *diesem* Durchlauf passiert ist; also faengt
///    hier ihre Zaehlung an. pygame nennt genau diesen Zeitpunkt.
/// 2. **Zeitgeber zustellen.** `rustgame_core::time::set_timer` legt nur
///    einen Auftrag an und ueberlaesst das Zustellen ausdruecklich
///    seinem Aufrufer - das ist diese Zeile hier. Ohne sie waere
///    `set_timer` ein Schalter ohne Draht: man setzt ihn, und es kommt
///    nie ein Ereignis.
///
///    Die Ereignisse gehen durch [`Schlange::ablegen`], unterliegen
///    also derselben Sperrliste wie alles andere - `set_blocked` auf den
///    Zeitgeber-Typ wirkt, wie es in pygame auch wirkt. Und sie kommen
///    **vor** dem, was das Fenster in diesem Durchlauf meldet; ein
///    Zeitgeber war ja schon vorher faellig.
/// 3. **Das Fenster fragen** - `pumpen`.
///
/// Wer nur den Tastatur- oder Mauszustand auffrischen will, nimmt
/// `pumpen` unmittelbar: das laesst die Kantentabellen stehen.
pub(crate) fn durchlauf(z: &mut Zustand) {
    z.eingabe.kanten_loeschen();
    for typ in rustgame_core::time::faellige_zeitgeber() {
        z.schlange.ablegen(Event::neu(typ));
    }
    pumpen(z);
}

/// Fragt das Hintergrundstueck ab und legt ab, was es meldet.
///
/// Der kurze Weg: **ohne** Zeitgeber und **ohne** die Kantentabellen
/// anzufassen. Dafuer gibt es `durchlauf`.
pub(crate) fn pumpen(z: &mut Zustand) {
    let mut neu = Vec::new();
    // Getrennte Felder desselben Zustands - Rust laesst beide Ausleihen zu,
    // weil sie sich nicht ueberschneiden.
    if let Some(b) = z.backend.as_mut() {
        b.pumpen(&mut z.eingabe, &mut neu);
    }
    // Und die Fenster, die ihr eigenes Stueck haben - siehe
    // [`crate::fenster::eigene_pumpen`], dort steht auch, was dabei
    // nicht geht.
    crate::fenster::eigene_pumpen(z, &mut neu);
    for e in neu {
        z.schlange.ablegen(e);
    }
}
