//! zeitgeber.rs
//! ============
//! Dass `time.set_timer` wirklich Ereignisse zustellt.
//!
//! ## Warum das eine eigene Datei ist
//!
//! `rustgame_core::time` fuehrt seine Zeitgeber **prozessweit** (ein
//! `static Mutex<Vec<..>>`), waehrend der Zustand von `rustgame-display`
//! am Strang haengt. `cargo test` faehrt die Tests einer Datei
//! nebenlaeufig in einem Prozess - ein Test, der einen Zeitgeber setzt,
//! und ein anderer, der `event::get()` ruft, kaemen sich also in die
//! Quere: der zweite wuerde das Ereignis des ersten abholen.
//!
//! Jede Datei unter `tests/` wird aber zu einem **eigenen Programm**.
//! Diese hier enthaelt darum nur einen einzigen Test, und der ist der
//! einzige im Prozess, der Zeitgeber anfasst.
//!
//! **Kein Fenster.** Wie in `gegen_pygame.rs` wird ausdruecklich
//! `display::set_treiber(Treiber::Dummy)` gerufen - `cfg!(test)` gilt
//! nur innerhalb der Bibliothek, nicht in einem Integrationstest.

use rustgame_core::time;
use rustgame_display::display::Treiber;
use rustgame_display::konstanten::*;
use rustgame_display::{display, event};

/// Ein Zeitgeber legt sein Ereignis in die Schlange - und niemand sonst
/// muss dafuer etwas tun.
///
/// Was hier geprueft wird, Punkt fuer Punkt:
///
/// 1. Vor der Faelligkeit kommt nichts.
/// 2. Nach der Faelligkeit kommt es bei **jedem** Blick in die Schlange -
///    `get`, `poll` und `pump` gehen denselben Weg.
/// 3. Ein Zeitgeber, der mehrere Abstaende verschlafen hat, **holt sie
///    nach**. Das stand hier einmal andersherum ("nur einmal je
///    Durchlauf, das ist SDLs Verhalten") - an echtem pygame
///    nachgemessen stimmt es nicht: ein 20-ms-Zeitgeber, sechsmal im
///    Abstand von 30 ms abgefragt, liefert dort acht Ereignisse, nicht
///    sechs. SDL laesst einen eigenen Strang laufen, der bei jedem
///    Abstand einreiht.
/// 4. `set_blocked` wirkt auch auf Zeitgeber-Ereignisse - sie gehen
///    denselben Weg wie alle anderen.
/// 5. `millis <= 0` haelt ihn an.
#[test]
fn set_timer_stellt_wirklich_zu() {
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    display::set_mode((8, 8), 0).unwrap();
    event::clear(None);
    time::init();

    let typ = USEREVENT + 7;

    // --- 1. Vor der Faelligkeit kommt nichts --------------------------
    time::set_timer(typ, 40, 0);
    assert!(
        event::get().is_empty(),
        "vierzig Millisekunden sind noch nicht um"
    );

    // --- 2. Danach schon ----------------------------------------------
    std::thread::sleep(std::time::Duration::from_millis(60));
    let evs = event::get();
    assert_eq!(evs.len(), 1, "genau eins, {evs:?}");
    assert_eq!(evs[0].typ, typ);
    assert_eq!(
        evs[0].code(),
        None,
        "ein Zeitgeber-Ereignis traegt keine Felder - wie in pygame"
    );

    // Auch `poll` und `pump` sind Durchlaeufe.
    std::thread::sleep(std::time::Duration::from_millis(60));
    assert_eq!(event::poll().typ, typ, "poll stellt ebenfalls zu");
    std::thread::sleep(std::time::Duration::from_millis(60));
    event::pump();
    assert_eq!(event::poll().typ, typ, "und pump auch");

    // --- 3. Verschlafenes wird nachgeholt ------------------------------
    std::thread::sleep(std::time::Duration::from_millis(200)); // fuenf Abstaende
    let evs = event::get();
    assert!(
        evs.len() >= 5,
        "jeder verschlafene Abstand zaehlt, waren nur {}: {evs:?}",
        evs.len()
    );

    // --- 4. Die Sperrliste gilt auch hier -----------------------------
    event::set_blocked(Some(&[typ]));
    std::thread::sleep(std::time::Duration::from_millis(60));
    assert!(event::get().is_empty(), "gesperrt, also weg");
    event::set_allowed(Some(&[typ]));

    // --- 5. millis <= 0 haelt ihn an ----------------------------------
    time::set_timer(typ, 0, 0);
    event::clear(None);
    std::thread::sleep(std::time::Duration::from_millis(60));
    assert!(event::get().is_empty(), "abgeschaltet heisst abgeschaltet");
    assert!(time::laufende_zeitgeber().is_empty());

    // --- Und die Begrenzung auf n Wiederholungen ----------------------
    //
    // Dreissig Millisekunden reichen fuer drei Abstaende, verlangt waren
    // aber nur zwei - also kommen genau zwei und danach nichts mehr.
    time::set_timer(typ, 10, 2);
    std::thread::sleep(std::time::Duration::from_millis(30));
    assert_eq!(event::get().len(), 2, "zweimal war verlangt");
    std::thread::sleep(std::time::Duration::from_millis(30));
    assert!(event::get().is_empty(), "und dann Schluss");
    assert!(time::laufende_zeitgeber().is_empty());

    time::quit();
    display::quit();
}
