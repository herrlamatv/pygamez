//! joystick.rs
//! ===========
//! `pygame.joystick`.
//!
//! **rustgame hat keinen Spielhebel-Treiber**, und das ist hier kein
//! Mangel, den eine Notloesung kaschieren muesste: `minifb` kennt keine
//! Gamepads, und das fensterlose Stueck erst recht nicht. Es gibt also
//! nie einen - `get_count()` ist **0**.
//!
//! Und genau deshalb laesst sich dieses Modul **vollstaendig
//! wahrheitsgetreu** anbieten. Was pygame in diesem Zustand tut, ist
//! gemessen:
//!
//! | Aufruf | pygame-ce 2.5.7 |
//! |---|---|
//! | `get_count()` nach `init()` | `0` |
//! | `get_count()` nach `quit()` | `pygame.error: joystick system not initialized` |
//! | `get_init()` vor `pygame.init()` | `False` |
//! | `Joystick(0)` | `pygame.error: Invalid joystick device number` |
//!
//! Ein Programm, das Hebel sucht, findet hier keine und kommt damit
//! zurecht - genauso wie auf einem Rechner ohne angeschlossenes Geraet.
//! Der einzige Aufruf, der etwas anderes braeuchte, ist `Joystick(n)`,
//! und der scheitert mit **pygames eigenem Wortlaut**.
//!
//! Kommt eines Tages ein Treiber dazu, wandert die Zaehlung hierher und
//! die Huelle bleibt, wie sie ist.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::{Ergebnis, Fehler};

/// Ob `joystick.init()` gelaufen ist.
///
/// Eigener Schalter, nicht der von [`crate::display`]: gemessen laesst
/// sich `joystick.quit()` einzeln aufrufen, ohne dass die Anzeige davon
/// etwas merkt.
static BEREIT: AtomicBool = AtomicBool::new(false);

/// `pygame.joystick.init()`.
pub fn init() {
    BEREIT.store(true, Ordering::SeqCst);
}

/// `pygame.joystick.quit()`.
pub fn quit() {
    BEREIT.store(false, Ordering::SeqCst);
}

/// `pygame.joystick.get_init()`.
pub fn get_init() -> bool {
    BEREIT.load(Ordering::SeqCst)
}

/// `pygame.joystick.get_count()`.
///
/// Immer `0` - siehe den Modulkopf. Vor `init()` ist es allerdings ein
/// **Fehler**, keine Null; gemessen.
pub fn get_count() -> Ergebnis<usize> {
    if !get_init() {
        return Err(Fehler::Pygame("joystick system not initialized"));
    }
    Ok(0)
}

/// `pygame.joystick.Joystick(n)` - scheitert immer.
///
/// Der Wortlaut ist der von echtem pygame, wenn die Nummer zu gross ist,
/// und da es hier **nie** einen Hebel gibt, ist jede Nummer zu gross.
pub fn oeffnen(_nummer: usize) -> Ergebnis<()> {
    if !get_init() {
        return Err(Fehler::Pygame("joystick system not initialized"));
    }
    Err(Fehler::Pygame("Invalid joystick device number"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sollwerte an echtem pygame-ce 2.5.7 gemessen.
    #[test]
    fn ohne_hebel_verhaelt_es_sich_wie_pygame() {
        quit();
        assert!(!get_init());
        assert!(get_count().is_err(), "vor init ist es ein Fehler, keine Null");
        init();
        assert!(get_init());
        assert_eq!(get_count().unwrap(), 0);
        assert!(oeffnen(0).is_err());
        quit();
        assert!(!get_init());
    }
}
