//! ereignis.rs
//! ===========
//! Wo die Endereignisse landen - die Naht zu `rustgame-display`.
//!
//! ## Das Problem
//!
//! `Channel.set_endevent` und `music.set_endevent` sind die einzige
//! Stelle, an der `pygame.mixer` die Ereignisschlange anfasst. Die liegt
//! aber in `rustgame-display`, und diese Kiste haengt nicht daran - eine
//! Abhaengigkeit `mixer -> display` waere verkehrt herum: Ton braucht
//! kein Fenster.
//!
//! ## Die Loesung: zwei Wege, beide offen
//!
//! Der Mixer **sammelt** die faelligen Ereignisse selbst. Abgeholt werden
//! sie wahlweise
//!
//! 1. **geholt** - [`abholen`] leert die Sammlung. `rustgame-display`
//!    kann das am Anfang von `event::get()` und `event::pump()` tun und
//!    jedes [`Endereignis`] mit `event::post` in die Schlange legen. Das
//!    ist der Weg, der ohne jede Anmeldung funktioniert und den auch ein
//!    Programm ohne Fenster nehmen kann.
//! 2. **geschoben** - [`set_senke`] meldet einen Rueckruf an, der jedes
//!    Ereignis sofort bekommt, sobald es entsteht. Damit kann
//!    `rustgame-display` beim Anlauf einmal
//!    `rustgame_mixer::ereignis::set_senke(|e| { event::post(...); })`
//!    setzen und muss danach nichts mehr abfragen.
//!
//! Solange eine Senke angemeldet ist, sammelt der Mixer **nicht** mehr -
//! sonst kaeme jedes Ereignis zweimal an.
//!
//! ## Was in einem Ereignis steht
//!
//! Gemessen an echtem pygame:
//!
//! | woher | `type` | Felder |
//! |---|---|---|
//! | `Channel.set_endevent(USEREVENT)` | `USEREVENT` | `code` = Kanalindex |
//! | `music.set_endevent(USEREVENT+1)` | `USEREVENT+1` | **keine** |
//!
//! Das `code`-Feld gibt es nur, wenn der Ereignistyp im Bereich der
//! Benutzerereignisse liegt (`USEREVENT` bis `NUMEVENTS`); pygame prueft
//! das in `_pg_push_mixer_event`. Und: pygame legt das Ereignis
//! **gar nicht** ab, wenn das Video-Teilsystem nicht laeuft - gemessen,
//! die Schlange bleibt dann leer. Diese Kiste sammelt trotzdem, weil sie
//! vom Video-Teilsystem nichts weiss; wer das nachbilden will, fragt auf
//! seiner Seite und verwirft.

use std::cell::RefCell;
use std::rc::Rc;

use crate::zustand;

/// Ein faelliges Endereignis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Endereignis {
    /// Der Ereignistyp, wie er bei `set_endevent` uebergeben wurde.
    pub typ: u32,
    /// Der Kanalindex - pygames `code`-Feld. Bei der Musik `None`.
    pub code: Option<usize>,
}

/// Der Rueckruf, den [`set_senke`] ablegt.
///
/// Ein `Rc` und kein `Box`, damit [`an_senke`] ihn herausziehen kann,
/// bevor es ihn ruft - sonst koennte ein Rueckruf, der selbst wieder
/// `set_senke` aufruft, die uebliche `RefCell`-Panik ausloesen.
type Senke = Rc<dyn Fn(Endereignis)>;

thread_local! {
    /// Wohin die Ereignisse sofort gehen sollen, falls jemand das will.
    static SENKE: RefCell<Option<Senke>> = const { RefCell::new(None) };
}

/// Meldet einen Rueckruf an, der jedes Endereignis sofort bekommt.
///
/// Gedacht fuer `rustgame-display`: einmal beim Anlauf setzen, fertig.
/// Solange einer gesetzt ist, sammelt [`abholen`] nichts mehr.
///
/// Der Rueckruf darf zurueck in den Mixer greifen - er laeuft erst,
/// nachdem der Mixerzustand wieder freigegeben ist.
pub fn set_senke(f: impl Fn(Endereignis) + 'static) {
    SENKE.with(|s| *s.borrow_mut() = Some(Rc::new(f)));
}

/// Nimmt den Rueckruf wieder weg.
pub fn senke_loeschen() {
    SENKE.with(|s| *s.borrow_mut() = None);
}

/// Ist ein Rueckruf angemeldet?
pub(crate) fn senke_gesetzt() -> bool {
    SENKE.with(|s| s.borrow().is_some())
}

/// Gibt ein Ereignis an den Rueckruf weiter.
pub(crate) fn an_senke(e: Endereignis) {
    // Erst den Rc herausziehen, dann die Ausleihe fallen lassen - sonst
    // koennte ein Rueckruf, der `set_senke` aufruft, die uebliche
    // RefCell-Panik ausloesen.
    let f = SENKE.with(|s| s.borrow().clone());
    if let Some(f) = f {
        f(e);
    }
}

/// Holt alle faelligen Endereignisse und leert die Sammlung.
///
/// Der Aufruf fuehrt den Mixer vorher nach, es kommt also wirklich alles
/// heraus, was bis jetzt faellig geworden ist.
///
/// Liefert immer eine leere Liste, solange eine [`set_senke`] angemeldet
/// ist - dann gehen die Ereignisse dorthin.
pub fn abholen() -> Vec<Endereignis> {
    zustand::mit(|z| std::mem::take(&mut z.ereignisse))
}

/// Wie viele Ereignisse gerade warten, ohne sie zu nehmen.
pub fn wartend() -> usize {
    zustand::mit(|z| z.ereignisse.len())
}
