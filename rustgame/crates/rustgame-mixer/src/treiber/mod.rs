//! treiber
//! =======
//! Was ein Treiber koennen muss, damit `mixer`, `sound`, `channel` und
//! `music` darauf aufsetzen koennen - und die zwei Stuecke, die das tun.
//!
//! | Stueck | Schall? | wofuer |
//! |---|---|---|
//! | [`RodioTreiber`] | ja | der Normalfall am Schreibtisch |
//! | [`StummerTreiber`] | nein | kopflos, eingebettet, getestet |
//!
//! ## Warum der Trait so schmal ist
//!
//! Das Muster stammt von `rustgame-display`, aber die Arbeitsteilung ist
//! hier eine andere. Dort fragt das Stueck das Fenster nach Ereignissen -
//! es ist also Quelle von Zustand. Hier ist es das **nicht**: der Treiber
//! wird nie gefragt, nur beauftragt. Wie lange etwas laeuft, ob ein Kanal
//! belegt ist, was in der Schlange steht - das rechnet `crate::zustand`
//! aus der Uhr aus, fuer beide Stuecke gleich.
//!
//! Das ist der ganze Trick hinter der Bedingung "muss ohne Tonausgabe
//! laufen": es gibt gar keinen zweiten Satz Regeln, den man
//! auseinanderlaufen lassen koennte. `get_busy()` und `music.get_pos()`
//! liefern unter [`StummerTreiber`] dieselben Zahlen wie unter
//! [`RodioTreiber`], weil sie aus derselben Rechnung kommen.
//!
//! ## Welches Stueck kommt
//!
//! [`oeffnen`] entscheidet:
//!
//! * `RUSTGAME_AUDIODRIVER=dummy` - der stille Treiber,
//! * in den Modultests dieser Kiste (`cfg!(test)`) der stille Treiber,
//! * sonst rodio, und wenn kein Tongeraet da ist, ein sauberer
//!   [`crate::Fehler::KeinGeraet`].
//!
//! **Achtung bei Integrationstests:** `cfg!(test)` hilft dort nicht. Ein
//! Test unter `tests/` bindet die Bibliothek als gewoehnliche
//! Abhaengigkeit ein, und die ist ohne `cfg(test)` uebersetzt - der
//! Testlauf wuerde also die Lautsprecher anwerfen. Wer dort testet,
//! schaltet mit [`crate::mixer::set_treiber`] ausdruecklich um; so macht
//! es `tests/gegen_pygame.rs`, und so macht es `rustgame-display` mit
//! seinem fensterlosen Hintergrundstueck auch.

use crate::Ergebnis;

mod rodio_treiber;
mod stumm;

pub use rodio_treiber::RodioTreiber;
pub use stumm::StummerTreiber;

/// Wohin ein Klang geht.
///
/// pygame trennt Mischkanaele und Musik streng: die Musik laeuft an den
/// Kanaelen vorbei, hat eine eigene Lautstaerke und eine eigene
/// Warteschlange. Das bildet diese Aufzaehlung ab.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stimme {
    /// Einer der Mischkanaele, gezaehlt ab null.
    Kanal(usize),
    /// Die Musikspur.
    Musik,
}

/// Was gespielt werden soll.
///
/// Alles hier ist bereits ausgerechnet - der Treiber muss nichts mehr
/// ableiten, insbesondere nichts ueber die Zeit.
#[derive(Debug)]
pub struct Auftrag<'a> {
    /// Die Abtastwerte, verschraenkt.
    pub werte: &'a [i16],
    /// Wie viele Spuren verschraenkt sind.
    pub spuren: u16,
    /// Abtastrate der Werte.
    pub frequenz: u32,
    /// Wie oft das Stueck laufen soll; `-1` heisst endlos.
    ///
    /// **Schon umgerechnet.** `Sound.play(loops=1)` spielt zweimal,
    /// `music.play(loops=1)` einmal - diese beiden Zaehlweisen sind in
    /// [`crate::sound`] und [`crate::music`] aufgeloest, hier steht die
    /// schlichte Anzahl.
    pub durchlaeufe: i32,
    /// Wo im Stueck begonnen wird, in Millisekunden.
    pub ab_ms: u64,
    /// Nach so vielen Millisekunden ist Schluss - aus `maxtime`.
    ///
    /// `crate::zustand` wuerde es ohnehin abbrechen, sobald jemand
    /// wieder etwas fragt. Der Treiber bekommt es trotzdem, damit ein
    /// Klang nicht ueber seine Zeit hinaus zu hoeren ist, nur weil das
    /// Spiel gerade nichts abfragt.
    pub hoechstdauer_ms: Option<u64>,
    /// Wie lange eingeblendet wird, in Millisekunden; `0` heisst gar nicht.
    pub einblenden_ms: u64,
    /// Die Lautstaerke von 0.0 bis 1.0.
    pub lautstaerke: f32,
}

/// Was ein Treiber koennen muss.
pub trait Treiber {
    /// Der Name des Treibers - Gegenstueck zu
    /// `pygame.mixer.get_driver()`.
    fn name(&self) -> &str;

    /// Faengt an zu spielen und wirft weg, was an der Stimme noch lief.
    fn spielen(&mut self, stimme: Stimme, auftrag: Auftrag<'_>);

    /// Bricht ab.
    fn stoppen(&mut self, stimme: Stimme);

    /// Haelt an, ohne die Stelle zu verlieren.
    fn pausieren(&mut self, stimme: Stimme);

    /// Macht weiter.
    fn fortsetzen(&mut self, stimme: Stimme);

    /// Stellt die Lautstaerke ein, 0.0 bis 1.0.
    ///
    /// Das ist auch der Weg, auf dem das Ausblenden hoerbar wird:
    /// `crate::zustand::nachfuehren` ruft es waehrend eines `fadeout`
    /// immer wieder mit fallenden Werten. Ein eigener Strang dafuer waere
    /// Aufwand fuer nichts - wer `get_busy()` nicht abfragt, hoert das
    /// Ausblenden ohnehin nicht zu Ende.
    fn lautstaerke(&mut self, stimme: Stimme, wert: f32);

    /// Wie viele Kanaele es gibt - der Treiber darf aufraeumen.
    fn kanaele_setzen(&mut self, anzahl: usize);

    /// Macht dieses Stueck ueberhaupt Schall?
    ///
    /// Nur [`StummerTreiber`] sagt hier nein.
    fn ist_stumm(&self) -> bool {
        false
    }
}

/// Welches Stueck geoeffnet werden soll.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Wahl {
    /// Aus `RUSTGAME_AUDIODRIVER` lesen; unter `cargo test` immer
    /// [`Wahl::Dummy`].
    #[default]
    Automatisch,
    /// Echter Ton ueber rodio.
    Rodio,
    /// Kein Ton - alles bleibt Buchhaltung.
    Dummy,
}

/// Oeffnet das Stueck, das zur Wahl passt.
///
/// Gibt [`crate::Fehler::KeinGeraet`] zurueck, wenn rodio kein Tongeraet
/// findet. Das ist der saubere Fehler, den `mixer.init()` auf einem
/// Rechner ohne Soundkarte liefern soll - **keine Panik, kein stiller
/// Rueckfall auf Stille.** Wer trotzdem weiterarbeiten will, nimmt
/// ausdruecklich [`Wahl::Dummy`].
pub fn oeffnen(wahl: Wahl) -> Ergebnis<Box<dyn Treiber>> {
    let gewaehlt = match wahl {
        Wahl::Dummy => Wahl::Dummy,
        Wahl::Rodio => Wahl::Rodio,
        Wahl::Automatisch => {
            if cfg!(test) {
                Wahl::Dummy
            } else {
                match std::env::var("RUSTGAME_AUDIODRIVER") {
                    Ok(v) if v.eq_ignore_ascii_case("dummy") => Wahl::Dummy,
                    _ => Wahl::Rodio,
                }
            }
        }
    };

    match gewaehlt {
        Wahl::Dummy => Ok(Box::new(StummerTreiber::neu())),
        _ => Ok(Box::new(RodioTreiber::neu()?)),
    }
}
