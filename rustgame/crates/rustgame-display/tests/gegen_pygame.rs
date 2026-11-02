//! gegen_pygame.rs
//! ===============
//! Was diese Kiste kann, gemessen an echtem pygame-ce 2.5.7.
//!
//! Jede Behauptung hier stammt aus einem Lauf gegen das echte pygame im
//! Projekt-`.venv`, kopflos mit `SDL_VIDEODRIVER=dummy`. Wo eine Messung
//! einer naheliegenden Annahme widerspricht, steht das dabei - das ist
//! der eigentliche Zweck dieser Datei.
//!
//! **Kein Test oeffnet ein Fenster.** Alle laufen auf dem fensterlosen
//! Hintergrundstueck; [`vorbereiten`] schaltet es ein. Weil der Zustand
//! dieser Kiste am Strang haengt und `cargo test` jeden Test in einem
//! eigenen Strang faehrt, stoeren sich die Tests auch nicht.

use rustgame_display::backend::dummy;
use rustgame_display::display::Treiber;
use rustgame_display::konstanten::*;
use rustgame_display::{display, event, key, mouse, tk, Event, Fehler};

/// Fensterloses Stueck an, Anzeige hoch, Schlange leer.
fn vorbereiten(groesse: (u32, u32)) {
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    display::set_mode(groesse, 0).unwrap();
    event::clear(None);
}

// ---------------------------------------------------------------------------
//  display
// ---------------------------------------------------------------------------

#[test]
fn set_mode_liefert_immer_dieselbe_flaeche() {
    // Gemessen: `s1 is s2` ist True - und, entgegen der Erwartung, auch
    // dann noch, wenn die Groesse dazwischen wechselt. pygame baut die
    // vorhandene Flaeche um, statt eine neue zu machen.
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();

    let a = display::set_mode((320, 240), 0).unwrap();
    let b = display::set_mode((320, 240), 0).unwrap();
    assert!(std::sync::Arc::ptr_eq(&a, &b), "gleiche Groesse: gleiche Flaeche");

    let c = display::set_mode((400, 300), 0).unwrap();
    assert!(
        std::sync::Arc::ptr_eq(&a, &c),
        "andere Groesse: immer noch dieselbe Flaeche"
    );
    assert_eq!(a.read().get_size(), (400, 300), "aber umgebaut");
    assert!(std::sync::Arc::ptr_eq(
        &a,
        &display::get_surface().expect("gesetzt")
    ));
}

#[test]
fn get_surface_vor_set_mode_ist_leer() {
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    assert!(display::get_surface().is_none());
}

#[test]
fn get_init_folgt_init_und_quit() {
    display::set_treiber(Treiber::Dummy);
    assert!(!display::get_init());
    display::init().unwrap();
    assert!(display::get_init());
    display::quit();
    assert!(!display::get_init(), "gemessen: nach quit ist get_init falsch");
    assert!(display::get_surface().is_none(), "und get_surface leer");
}

#[test]
fn caption_faengt_bei_pygame_window_an() {
    // Gemessen: ('pygame window', 'pygame window').
    display::set_treiber(Treiber::Dummy);
    assert_eq!(
        display::get_caption(),
        ("pygame window".to_string(), "pygame window".to_string())
    );
    display::set_caption("Titel");
    assert_eq!(
        display::get_caption(),
        ("Titel".to_string(), "Titel".to_string()),
        "gemessen: beide Titel sind immer gleich - set_caption('A','B') \
         ergibt get_caption() == ('A','A')"
    );
}

#[test]
fn flip_ohne_set_mode_ist_ein_fehler() {
    // Gemessen: pygame wirft `Display mode not set`.
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    assert_eq!(display::flip(), Err(Fehler::KeinModus));
    assert_eq!(display::update(), Err(Fehler::KeinModus));
    assert_eq!(display::flip().unwrap_err().to_string(), "Display mode not set");
}

#[test]
fn toggle_fullscreen_ohne_fenster_ist_ein_fehler() {
    // Widerlegt die naheliegende Annahme, pygame liefere hier 0:
    // gemessen wirft es `That operation is not supported`.
    vorbereiten((64, 48));
    let f = display::toggle_fullscreen().unwrap_err();
    assert!(matches!(f, Fehler::NichtUnterstuetzt(_)));
    assert_eq!(f.to_string(), "That operation is not supported");
}

#[test]
fn list_modes_ohne_fenster_liefert_1024_768() {
    // Gemessen unter SDL_VIDEODRIVER=dummy.
    vorbereiten((64, 48));
    assert_eq!(display::list_modes(), vec![(1024, 768)]);
}

#[test]
fn get_window_size_und_info() {
    vorbereiten((320, 240));
    assert_eq!(display::get_window_size(), Ok((320, 240)));

    // Alle Werte an echtem pygame gemessen.
    let i = display::info();
    assert_eq!((i.hw, i.wm, i.video_mem), (0, 1, 0));
    assert_eq!((i.bitsize, i.bytesize), (32, 4));
    assert_eq!(i.masks, (0x00ff_0000, 0x0000_ff00, 0x0000_00ff, 0));
    assert_eq!(i.shifts, (16, 8, 0, 0));
    assert_eq!(i.losses, (0, 0, 0, 8));
    assert_eq!((i.current_w, i.current_h), (320, 240));
    assert_eq!(i.pixel_format, "PIXELFORMAT_RGB888");
}

#[test]
fn der_treiber_heisst_dummy() {
    vorbereiten((8, 8));
    assert_eq!(display::get_driver(), Ok("dummy"));
}

// ---------------------------------------------------------------------------
//  event
// ---------------------------------------------------------------------------

#[test]
fn set_mode_legt_fuenf_ereignisse_ab() {
    // Der Fund, den man ohne Messung nicht ahnt. Gemessene Reihenfolge:
    // ActiveEvent(gain=1,state=1), WindowEnter,
    // ActiveEvent(gain=1,state=2), WindowFocusGained, WindowShown.
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    display::set_mode((64, 48), 0).unwrap();

    let evs = event::get();
    let typen: Vec<EventType> = evs.iter().map(|e| e.typ).collect();
    assert_eq!(
        typen,
        vec![
            ACTIVEEVENT,
            WINDOWENTER,
            ACTIVEEVENT,
            WINDOWFOCUSGAINED,
            WINDOWSHOWN
        ]
    );
    // APPMOUSEFOCUS ist 1, APPINPUTFOCUS ist 2 - gemessen.
    assert_eq!(APPMOUSEFOCUS, 1);
    assert_eq!(APPINPUTFOCUS, 2);
}

#[test]
fn get_auf_leerer_schlange_ist_leer() {
    // Gemessen: eine leere Liste, kein Blockieren.
    vorbereiten((8, 8));
    assert!(event::get().is_empty());
}

#[test]
fn poll_auf_leerer_schlange_liefert_noevent() {
    // Gemessen: ein Ereignis vom Typ 0, nicht None.
    vorbereiten((8, 8));
    assert_eq!(event::poll().typ, NOEVENT);
    assert_eq!(NOEVENT, 0);
    assert_eq!(event::event_name(NOEVENT), "NoEvent");
}

#[test]
fn peek_auf_leerer_schlange_ist_falsch() {
    // Gemessen: peek() liefert einen Wahrheitswert, keine Liste.
    vorbereiten((8, 8));
    assert!(!event::peek(None));
    assert!(!event::peek(Some(&[QUIT])));

    event::post(Event::quit());
    assert!(event::peek(None));
    assert!(event::peek(Some(&[QUIT])));
    assert!(!event::peek(Some(&[KEYDOWN])));
    // Gemessen: peek raeumt nicht ab.
    assert_eq!(event::get().len(), 1);
}

#[test]
fn erst_herein_erst_heraus() {
    vorbereiten((8, 8));
    for i in 0..3 {
        event::post(Event::user(i));
    }
    let codes: Vec<i32> = event::get().iter().filter_map(|e| e.code()).collect();
    assert_eq!(codes, vec![0, 1, 2]);
}

#[test]
fn get_mit_auswahl_laesst_den_rest_liegen() {
    vorbereiten((8, 8));
    event::post(Event::quit());
    event::post(Event::key_down(K_a, 0, "a"));
    event::post(Event::key_up(K_a, 0));

    let geholt: Vec<EventType> = event::get_typen(Some(&[KEYDOWN, KEYUP]))
        .iter()
        .map(|e| e.typ)
        .collect();
    assert_eq!(geholt, vec![KEYDOWN, KEYUP]);
    let rest: Vec<EventType> = event::get().iter().map(|e| e.typ).collect();
    assert_eq!(rest, vec![QUIT]);
}

#[test]
fn get_ausser_holt_alles_andere() {
    vorbereiten((8, 8));
    event::post(Event::quit());
    event::post(Event::key_down(K_a, 0, "a"));

    let geholt: Vec<EventType> = event::get_ausser(&[QUIT]).iter().map(|e| e.typ).collect();
    assert_eq!(geholt, vec![KEYDOWN]);
    assert_eq!(event::get().len(), 1, "QUIT bleibt liegen");
}

#[test]
fn gesperrte_ereignisse_werden_verworfen() {
    // Gemessen: post auf einen gesperrten Typ verschwindet stillschweigend
    // - auch dann, wenn der Typ spaeter wieder freigegeben wird.
    vorbereiten((8, 8));
    event::set_blocked(Some(&[KEYDOWN]));
    assert!(event::get_blocked(KEYDOWN));
    assert!(!event::post(Event::key_down(K_a, 0, "a")));
    event::post(Event::key_up(K_a, 0));

    let typen: Vec<EventType> = event::get().iter().map(|e| e.typ).collect();
    assert_eq!(typen, vec![KEYUP]);

    event::set_allowed(Some(&[KEYDOWN]));
    assert!(!event::get_blocked(KEYDOWN));
    assert!(event::get().is_empty(), "das Verworfene kommt nicht zurueck");
}

#[test]
fn set_blocked_none_sperrt_alles_set_allowed_none_gibt_alles_frei() {
    // Beides gemessen - `None` heisst hier nicht "nichts", sondern "alles".
    vorbereiten((8, 8));
    event::set_blocked(None);
    assert!(event::get_blocked(KEYDOWN));
    assert!(event::get_blocked(QUIT));
    event::post(Event::quit());
    assert!(event::get().is_empty());

    event::set_allowed(None);
    assert!(!event::get_blocked(KEYDOWN));
    event::post(Event::quit());
    assert_eq!(event::get().len(), 1);
}

#[test]
fn set_allowed_nach_set_blocked_none_gibt_nur_das_genannte_frei() {
    vorbereiten((8, 8));
    event::set_blocked(None);
    event::set_allowed(Some(&[QUIT]));
    assert!(!event::get_blocked(QUIT));
    assert!(event::get_blocked(KEYDOWN));
}

#[test]
fn event_name_kennt_die_gemessenen_faelle() {
    assert_eq!(event::event_name(QUIT), "Quit");
    assert_eq!(event::event_name(KEYDOWN), "KeyDown");
    assert_eq!(event::event_name(MOUSEBUTTONDOWN), "MouseButtonDown");
    assert_eq!(event::event_name(WINDOWFOCUSGAINED), "WindowFocusGained");
    // Gemessen: alles ab USEREVENT heisst "UserEvent".
    assert_eq!(event::event_name(USEREVENT), "UserEvent");
    assert_eq!(event::event_name(USEREVENT + 3), "UserEvent");
    // Gemessen: Unbekanntes heisst "Unknown".
    assert_eq!(event::event_name(9999), "Unknown");
}

#[test]
fn custom_type_faengt_bei_userevent_plus_eins_an() {
    // Gemessen: der erste Aufruf liefert 32866, also USEREVENT + 1.
    vorbereiten((8, 8));
    assert_eq!(event::custom_type(), Ok(USEREVENT + 1));
    assert_eq!(event::custom_type(), Ok(USEREVENT + 2));
}

#[test]
fn clear_raeumt_nur_das_genannte_ab() {
    vorbereiten((8, 8));
    event::post(Event::quit());
    event::post(Event::key_down(K_a, 0, "a"));
    event::clear(Some(&[KEYDOWN]));
    let typen: Vec<EventType> = event::get().iter().map(|e| e.typ).collect();
    assert_eq!(typen, vec![QUIT]);
}

#[test]
fn wait_mit_frist_gibt_auf() {
    // Gemessen: wait(50) auf einer leeren Schlange kehrt nach rund 50 ms
    // mit NoEvent zurueck.
    vorbereiten((8, 8));
    let t0 = std::time::Instant::now();
    let e = event::wait(Some(50));
    assert_eq!(e.typ, NOEVENT);
    assert!(
        t0.elapsed().as_millis() >= 45,
        "hat nicht gewartet: {:?}",
        t0.elapsed()
    );
}

#[test]
fn wait_nimmt_was_schon_da_ist() {
    vorbereiten((8, 8));
    event::post(Event::quit());
    assert_eq!(event::wait(Some(1000)).typ, QUIT);
}

#[test]
fn die_felder_haengen_am_richtigen_ereignis() {
    let e = Event::key_down(K_a, KMOD_LSHIFT, "A");
    assert_eq!(e.key(), Some(K_a));
    assert_eq!(e.scancode(), Some(KSCAN_A));
    assert_eq!(e.mods(), KMOD_LSHIFT);
    assert_eq!(e.unicode(), "A");
    assert_eq!(e.pos(), None, "eine Taste hat keine Stelle");

    let m = Event::mouse_button_down((10, 20), 1);
    assert_eq!(m.pos(), Some((10, 20)));
    assert_eq!(m.button(), Some(1));
    assert_eq!(m.key(), None, "ein Klick hat keinen Tastencode");

    let b = Event::mouse_motion((5, 6), (1, 2), (true, false, false));
    assert_eq!(b.pos(), Some((5, 6)));
    assert_eq!(b.rel(), Some((1, 2)));
}

// ---------------------------------------------------------------------------
//  key
// ---------------------------------------------------------------------------

#[test]
fn get_pressed_hat_512_plaetze() {
    // Gemessen: len(pygame.key.get_pressed()) ist 512, Index 0 ist False.
    // Das ist die Zahl der **Tastenplaetze**, nicht der Tastencodes -
    // K_UP allein ist 1073741906.
    vorbereiten((8, 8));
    let t = key::get_pressed();
    assert_eq!(t.len(), 512);
    assert_eq!(NUM_SCANCODES, 512);
    assert!(!t.roh(0), "Platz 0 ist SDL_SCANCODE_UNKNOWN");
    assert!(!t.irgendeine());
}

#[test]
fn get_pressed_laesst_sich_mit_tastencodes_ansprechen() {
    // Gemessen: p[K_UP] geht, obwohl K_UP 1073741906 ist und die Tabelle
    // nur 512 Plaetze hat. pygame rechnet still um; hier tut es
    // key::scancode_von.
    vorbereiten((8, 8));
    let t = key::get_pressed();
    assert!(!t[K_UP]);
    assert!(!t[K_a]);
    assert!(!t[K_SPACE]);
}

#[test]
fn scancode_von_rechnet_beide_faelle_richtig() {
    // Tasten mit gesetzter Maske tragen ihren Platz selbst:
    assert_eq!(key::scancode_von(K_UP), KSCAN_UP);
    assert_eq!(K_UP - KSCAN_UP, SCANCODE_MASK);
    assert_eq!(key::scancode_von(K_F1), KSCAN_F1);
    // Zeichentasten nicht - die kommen aus der ausgelesenen Tabelle:
    assert_eq!(key::scancode_von(K_a), KSCAN_A);
    assert_eq!(KSCAN_A, 4, "gemessen");
    assert_eq!(key::scancode_von(K_SPACE), KSCAN_SPACE);
    assert_eq!(KSCAN_SPACE, 44, "gemessen");
}

#[test]
fn tastennamen_in_beiden_schreibweisen() {
    // Alle vier an echtem pygame gemessen.
    assert_eq!(key::name(K_a), "a");
    assert_eq!(key::name_sdl(K_a), "A");
    assert_eq!(key::name(K_SPACE), "space");
    assert_eq!(key::name(K_RETURN), "return");
    assert_eq!(key::name(K_UP), "up");
    assert_eq!(key::name(K_ESCAPE), "escape");
    // Der Fall, den man aus der pygame-Schreibweise nicht hochrechnen
    // kann: "left shift" wird zu "Left Shift", nicht "Left shift".
    assert_eq!(key::name(K_LSHIFT), "left shift");
    assert_eq!(key::name_sdl(K_LSHIFT), "Left Shift");
}

#[test]
fn key_code_beachtet_bei_langen_namen_die_schreibweise_nicht() {
    // Gemessen: key_code("Up") und key_code("up") ergeben beide 1073741906.
    assert_eq!(key::key_code("Up"), Some(K_UP));
    assert_eq!(key::key_code("up"), Some(K_UP));
    assert_eq!(key::key_code("space"), Some(K_SPACE));
    assert_eq!(key::key_code("return"), Some(K_RETURN));
    assert_eq!(key::key_code("a"), Some(K_a));
    assert_eq!(K_SPACE, 32, "gemessen");
    assert_eq!(K_RETURN, 13, "gemessen");
    assert_eq!(K_a, 97, "gemessen");
}

#[test]
fn zusatztasten_sind_bitmasken() {
    // Gemessen: set_mods(KMOD_SHIFT) danach get_mods() ist 3 - KMOD_SHIFT
    // meint links UND rechts. Ein Vergleich mit == geht deshalb daneben.
    assert_eq!(KMOD_SHIFT, 3);
    assert_eq!(KMOD_LSHIFT, 1);
    assert_eq!(KMOD_RSHIFT, 2);
    assert_eq!(KMOD_SHIFT, KMOD_LSHIFT | KMOD_RSHIFT);

    vorbereiten((8, 8));
    assert_eq!(key::get_mods(), 0);
    key::set_mods(KMOD_SHIFT);
    assert_eq!(key::get_mods(), 3);
    assert!(key::get_mods() & KMOD_SHIFT != 0);
}

#[test]
fn wiederholung_faengt_bei_null_an() {
    // Gemessen: get_repeat() ohne set_repeat ist (0, 0).
    vorbereiten((8, 8));
    assert_eq!(key::get_repeat(), (0, 0));
    key::set_repeat(400, 50);
    assert_eq!(key::get_repeat(), (400, 50));
    key::set_repeat(0, 0);
    assert_eq!(key::get_repeat(), (0, 0));
}

#[test]
fn texteingabe_laesst_sich_schalten() {
    vorbereiten((8, 8));
    key::stop_text_input();
    key::start_text_input();
}

// ---------------------------------------------------------------------------
//  mouse
// ---------------------------------------------------------------------------

#[test]
fn maustasten_kommen_zu_dritt_oder_zu_fuenft() {
    // Gemessen: get_pressed() hat drei Werte, get_pressed(5) fuenf.
    vorbereiten((8, 8));
    assert_eq!(mouse::get_pressed(), (false, false, false));
    assert_eq!(mouse::get_pressed5(), [false; 5]);
}

#[test]
fn set_visible_liefert_den_alten_zustand() {
    // Gemessen und entgegen der Erwartung: set_visible(False) gibt True
    // zurueck, set_visible(True) danach False. Der Rueckgabewert ist der
    // **vorherige** Zustand.
    vorbereiten((8, 8));
    assert!(mouse::get_visible());
    assert!(mouse::set_visible(false), "vorher war er sichtbar");
    assert!(!mouse::set_visible(true), "vorher war er unsichtbar");
    assert!(mouse::get_visible());
}

#[test]
fn set_pos_zaehlt_nicht_als_bewegung() {
    // Gemessen: nach set_pos((100, 50)) ist get_rel() (0, 0) - ein Sprung
    // ist keine Bewegung. Wichtig fuer Programme, die den Zeiger zur
    // Mitte zuruecksetzen, um endlos drehen zu koennen.
    vorbereiten((8, 8));
    assert_eq!(mouse::get_pos(), (0, 0));
    assert_eq!(mouse::get_rel(), (0, 0));
    mouse::set_pos((100, 50));
    assert_eq!(mouse::get_pos(), (100, 50));
    assert_eq!(mouse::get_rel(), (0, 0));
    // Gemessen: das MOUSEMOTION liegt trotzdem in der Schlange.
    let evs = event::get();
    assert_eq!(evs.len(), 1);
    assert_eq!(evs[0].typ, MOUSEMOTION);
    assert_eq!(evs[0].pos(), Some((100, 50)));
    assert_eq!(evs[0].rel(), Some((0, 0)));
}

#[test]
fn get_rel_setzt_sich_zurueck() {
    vorbereiten((8, 8));
    dummy::maus_bewegen((10, 5));
    assert_eq!(mouse::get_rel(), (10, 5));
    assert_eq!(mouse::get_rel(), (0, 0), "zweimal gefragt heisst nichts mehr");
}

// ---------------------------------------------------------------------------
//  Das fensterlose Stueck
// ---------------------------------------------------------------------------

#[test]
fn eingespeiste_tasten_landen_in_schlange_und_zustand() {
    vorbereiten((8, 8));
    assert!(dummy::taste_druecken(K_a, "a"));

    assert!(key::get_pressed()[K_a], "get_pressed sieht sie");
    let evs = event::get();
    assert_eq!(evs.len(), 1);
    assert_eq!(evs[0].typ, KEYDOWN);
    assert_eq!(evs[0].key(), Some(K_a));
    assert_eq!(evs[0].unicode(), "a");

    dummy::taste_loslassen(K_a);
    assert!(!key::get_pressed()[K_a]);
    assert_eq!(event::get()[0].typ, KEYUP);
}

#[test]
fn eingespeiste_zusatztasten_schreiben_die_bitmaske_nach() {
    vorbereiten((8, 8));
    dummy::taste_druecken(K_LSHIFT, "");
    assert_eq!(key::get_mods() & KMOD_SHIFT, KMOD_LSHIFT);
    dummy::taste_loslassen(K_LSHIFT);
    assert_eq!(key::get_mods() & KMOD_SHIFT, 0);
}

#[test]
fn eingespeiste_klicks_landen_in_schlange_und_zustand() {
    vorbereiten((64, 48));
    dummy::maus_klicken((12, 34), 1);
    let typen: Vec<EventType> = event::get().iter().map(|e| e.typ).collect();
    assert_eq!(typen, vec![MOUSEMOTION, MOUSEBUTTONDOWN, MOUSEBUTTONUP]);
    assert_eq!(mouse::get_pos(), (12, 34));
    assert_eq!(mouse::get_pressed(), (false, false, false), "wieder los");

    dummy::maus_druecken(3);
    assert_eq!(mouse::get_pressed(), (false, false, true));
}

#[test]
fn eingespeistes_mausrad() {
    vorbereiten((8, 8));
    dummy::maus_rad(0, -2);
    let evs = event::get();
    assert_eq!(evs[0].typ, MOUSEWHEEL);
}

#[test]
fn eingespeistes_quit() {
    vorbereiten((8, 8));
    dummy::fenster_schliessen();
    assert_eq!(event::poll().typ, QUIT);
}

#[test]
fn eingespeister_fokus() {
    vorbereiten((8, 8));
    assert!(
        key::get_focused().unwrap(),
        "nach set_mode hat das Fenster den Fokus"
    );
    dummy::fokus_setzen(false);
    assert!(!key::get_focused().unwrap());
    let typen: Vec<EventType> = event::get().iter().map(|e| e.typ).collect();
    assert_eq!(typen, vec![ACTIVEEVENT, WINDOWFOCUSLOST]);
    dummy::fokus_setzen(true);
    assert!(key::get_focused().unwrap());

    // **Vor `display.init()` ist es ein Fehler**, nicht `false` -
    // gemessen an echtem pygame, das dort
    // `pygame.error: video system not initialized` gibt. `key_test.py`
    // prueft die Ausnahme ausdruecklich; vorher gab diese Zeile hier
    // stillschweigend `false` zurueck.
    display::quit();
    assert!(key::get_focused().is_err());
}

#[test]
fn eingespeistes_geht_durch_die_sperrliste() {
    // Eingespeiste Ereignisse sollen sich verhalten wie echte - also auch
    // von set_blocked erwischt werden.
    vorbereiten((8, 8));
    event::set_blocked(Some(&[KEYDOWN]));
    dummy::taste_druecken(K_a, "a");
    assert!(event::get().is_empty(), "gesperrt, also weg");
    assert!(
        key::get_pressed()[K_a],
        "der Tastenzustand stimmt trotzdem - wie bei echter Hardware"
    );
}

#[test]
fn flip_wird_geschluckt_aber_gezaehlt() {
    vorbereiten((32, 24));
    assert_eq!(dummy::bildzaehler(), 0);
    display::flip().unwrap();
    display::flip().unwrap();
    assert_eq!(dummy::bildzaehler(), 2);
    display::update().unwrap();
    assert_eq!(dummy::bildzaehler(), 3);
}

#[test]
fn update_mit_rechtecken_zeigt_auch_an() {
    use rustgame_core::Rect;
    vorbereiten((32, 24));
    display::update_rects(&[Some(Rect::new(0, 0, 8, 8))]).unwrap();
    assert_eq!(dummy::bildzaehler(), 1);
    // Gemessen: pygame ueberspringt None-Eintraege klaglos.
    display::update_rects(&[None]).unwrap();
    assert_eq!(dummy::bildzaehler(), 1, "nichts anzuzeigen, nichts gezaehlt");
    display::update_rects(&[None, Some(Rect::new(0, 0, 2, 2))]).unwrap();
    assert_eq!(dummy::bildzaehler(), 2);
    display::update_bereiche(&[Rect::new(0, 0, 4, 4)]).unwrap();
    assert_eq!(dummy::bildzaehler(), 3);
    display::update_bereiche(&[]).unwrap();
    assert_eq!(dummy::bildzaehler(), 3, "leere Liste zeigt nichts an");
}

#[test]
fn set_icon_ohne_fenster_geht_durch() {
    use rustgame_core::Surface;
    // Das fensterlose Stueck nimmt das Sinnbild klaglos an und tut
    // nichts - wie pygame unter dem Dummy-Treiber, wo set_icon `None`
    // zurueckgibt statt zu meckern.
    vorbereiten((8, 8));
    let mut bild = Surface::new((32, 32));
    bild.fill((255, 0, 0));
    assert!(display::set_icon(&bild).is_ok());
}

#[test]
fn set_icon_vor_set_mode_ist_ein_fehler() {
    use rustgame_core::Surface;
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    let bild = Surface::new((8, 8));
    assert_eq!(display::set_icon(&bild), Err(Fehler::KeinModus));
}

#[test]
fn pump_haelt_den_zustand_frisch_ohne_die_schlange_zu_leeren() {
    vorbereiten((8, 8));
    dummy::taste_druecken(K_SPACE, " ");
    event::pump();
    assert!(key::get_pressed()[K_SPACE]);
    assert_eq!(
        event::get().len(),
        1,
        "pump holt nur herein, es raeumt nicht ab"
    );
}

#[test]
fn ppm_ist_der_weg_nach_tkinter() {
    // Genau der Weg, den das umgebende Projekt heute mit pygame geht.
    vorbereiten((4, 3));
    display::get_surface().unwrap().write().fill((10, 20, 30));
    let p = dummy::ppm().expect("set_mode ist gelaufen");
    assert!(p.starts_with(b"P6\n4 3\n255\n"));
    assert_eq!(p.len(), "P6\n4 3\n255\n".len() + 4 * 3 * 3);
    assert_eq!(&p[p.len() - 3..], &[10, 20, 30], "die Farbe steht drin");
}

#[test]
fn ohne_dummy_backend_wird_nichts_eingespeist() {
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    // Noch kein set_mode, also kein Stueck offen.
    assert!(!dummy::taste_druecken(K_a, "a"));
    assert_eq!(dummy::bildzaehler(), 0);
    assert!(dummy::ppm().is_none());
}

// ---------------------------------------------------------------------------
//  Tkinter-Namen
// ---------------------------------------------------------------------------

#[test]
fn tkinter_namen_aus_der_einstellungsdatei() {
    // Genau die Werte, die in settings.py des umgebenden Projekts stehen.
    assert_eq!(tk::key_code("w"), Some(K_w));
    assert_eq!(tk::key_code("space"), Some(K_SPACE));
    assert_eq!(tk::key_code("Up"), Some(K_UP));
    assert_eq!(tk::key_code("Return"), Some(K_RETURN));
    // Die beiden, die man nicht raet:
    assert_eq!(tk::key_code("Prior"), Some(K_PAGEUP));
    assert_eq!(tk::key_code("Next"), Some(K_PAGEDOWN));
}

// ---------------------------------------------------------------------------
//  Die erzeugten Tabellen
// ---------------------------------------------------------------------------

#[test]
fn alle_tabellen_sind_sortiert() {
    // Binaere Suche auf einer unsortierten Tabelle findet manchmal nichts
    // und meldet nie einen Fehler - das ist genau die Art Fehler, die erst
    // Monate spaeter auffaellt.
    for p in KEY_TO_SCANCODE.windows(2) {
        assert!(p[0].0 < p[1].0, "KEY_TO_SCANCODE bei {}", p[0].0);
    }
    for p in KEY_NAMES.windows(2) {
        assert!(p[0].0 < p[1].0, "KEY_NAMES bei {}", p[0].0);
    }
    for p in KEY_CODES.windows(2) {
        assert!(p[0].0 < p[1].0, "KEY_CODES bei {}", p[0].0);
    }
    for p in TK_KEYSYMS.windows(2) {
        assert!(p[0].0 < p[1].0, "TK_KEYSYMS bei {}", p[0].0);
    }
    for p in MINIFB_KEYS.windows(2) {
        assert!(p[0].0 < p[1].0, "MINIFB_KEYS bei {}", p[0].0);
    }
    for p in EVENT_NAMES.windows(2) {
        assert!(p[0].0 < p[1].0, "EVENT_NAMES bei {}", p[0].0);
    }
}

#[test]
fn die_tabellen_haben_die_erwartete_groesse() {
    assert_eq!(MINIFB_KEYS.len(), 107, "minifb kennt 107 Tasten");
    assert_eq!(KEY_NAMES.len(), 134);
    assert!(TK_KEYSYMS.len() > 150);
}

#[test]
fn die_anzeigeflaeche_traegt_prealloc() {
    // Gemessen: get_flags() der Anzeigeflaeche ist 16777216 = PREALLOC -
    // und zwar auch dann, wenn set_mode mit RESIZABLE gerufen wurde. Das
    // Flag verschwindet unterwegs. Nur FULLSCREEN bleibt stehen.
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();

    let s = display::set_mode((16, 16), 0).unwrap();
    assert_eq!(s.read().get_flags(), PREALLOC);
    // Die Anzeigeflaeche hat kein Flaechen-Alpha - gemessen.
    assert_eq!(s.read().get_alpha(), None);

    display::quit();
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    let r = display::set_mode((16, 16), RESIZABLE).unwrap();
    assert_eq!(r.read().get_flags(), PREALLOC, "RESIZABLE verschwindet");

    display::quit();
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    let f = display::set_mode((16, 16), FULLSCREEN).unwrap();
    assert_eq!(f.read().get_flags(), FULLSCREEN | PREALLOC);
}

#[test]
fn fullscreen_ist_das_oberste_bit() {
    // In pygame steht dort -2147483648, weil CPython aus dem obersten
    // SDL-Bit ein Vorzeichen macht. Als u32 ist es 0x80000000.
    assert_eq!(FULLSCREEN, 0x8000_0000);
    assert_eq!(RESIZABLE, 16);
    assert_eq!(PREALLOC, 16_777_216);
}

#[test]
fn die_beiden_anderen_negativen_bleiben_negativ() {
    // Der zweite Fall mit Vorzeichen - und der, bei dem Maskieren falsch
    // waere: `FULLSCREEN` ist ein gesetztes oberstes Bit, diese beiden
    // sind echte -1 aus einer vorzeichenbehafteten SDL-Aufzaehlung.
    // Maskiert waeren sie 4294967295 - ein Achsenindex, den niemand
    // wiedererkennt. Darum sind sie `i32`.
    assert_eq!(CONTROLLER_AXIS_INVALID, -1);
    assert_eq!(CONTROLLER_BUTTON_INVALID, -1);
    assert_eq!(CONTROLLER_BUTTON_A, 0);
    assert_eq!(CONTROLLER_BUTTON_MAX, 21);
    assert_eq!(CONTROLLER_AXIS_MAX, 6);
}

#[test]
fn aus_jeder_neuen_gruppe_ein_gemessener_wert() {
    // Alle aus `pygame.locals` ausgelesen, nicht abgeschrieben - hier
    // steht je Gruppe einer, damit ein verrutschter Block auffaellt.
    assert_eq!(GL_DEPTH_SIZE, 6);
    assert_eq!(GL_SWAP_CONTROL, 0);
    assert_eq!(GL_CONTEXT_PROFILE_CORE, 1);
    assert_eq!(HAT_CENTERED, 0);
    assert_eq!(HAT_UP, 1);
    assert_eq!(HAT_LEFTUP, 9, "das Steuerkreuz ist eine Bitmaske: 8|1");
    assert_eq!(AUDIO_U8, 8);
    assert_eq!(AUDIO_S16SYS, 32784);
    assert_eq!(AUDIO_ALLOW_ANY_CHANGE, 15);
    assert_eq!(BLEND_RGBA_ADD, 6);
    assert_eq!(BLEND_PREMULTIPLIED, 17);
    assert_eq!(BLENDMODE_MUL, 8);
    assert_eq!(BLENDFACTOR_ONE_MINUS_DST_COLOR, 8);
    assert_eq!(BLENDOPERATION_REV_SUBTRACT, 3);
    assert_eq!(DIRECTION_BTT, 3);
    assert_eq!(FLASH_BRIEFLY, 1);
    assert_eq!(FLIP_HORIZONTAL, 1);
    assert_eq!(FONT_CENTER, 1);
    assert_eq!(TEXTUREACCESS_TARGET, 2);
    assert_eq!(SCROLL_ERASE, 4);
    assert_eq!(SCRAP_CLIPBOARD, 0);
    assert_eq!(BIG_ENDIAN, 4321);
    assert_eq!(LIL_ENDIAN, 1234);
    assert_eq!(HWACCEL, 256);
    assert_eq!(IS_CE, 1, "das hier ist pygame-ce, nicht pygame");
    assert_eq!(TIMER_RESOLUTION, 0);
    // Die fuenf Namen, deren Wert in pygame **keine Zahl** ist. Wer sie
    // fuer Zahlen haelt, merkt es erst beim Uebersetzen des erzeugten
    // Rust - oder gar nicht, weil sie im Skript herausfielen.
    assert_eq!(SCRAP_TEXT, "text/plain");
    assert_eq!(NULL_VIDEODRIVER, "dummy");
    // Und die Zeigerformen, die `mouse::set_cursor` braucht.
    assert_eq!(SYSTEM_CURSOR_ARROW, 0);
    assert_eq!(SYSTEM_CURSOR_HAND, 11);
}

// ---------------------------------------------------------------------------
//  display - was das Fenstersystem weiss
// ---------------------------------------------------------------------------

#[test]
fn get_active_folgt_dem_fenster() {
    // Gemessen: False vor set_mode, True danach - und False, wenn
    // set_mode mit HIDDEN gerufen wurde.
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    assert!(!display::get_active(), "ohne Fenster nicht aktiv");
    display::set_mode((64, 48), 0).unwrap();
    assert!(display::get_active());
    display::set_mode((64, 48), HIDDEN).unwrap();
    assert!(!display::get_active(), "HIDDEN heisst nicht sichtbar");
    assert_eq!(HIDDEN, 128, "gemessen");
    display::quit();
    assert!(!display::get_active(), "nach quit wieder nicht");
}

#[test]
fn bildschirmschoner_faengt_bei_falsch_an() {
    // Gemessen: vor init ein Fehler, danach False - pygame haelt den
    // Bildschirmschoner waehrend des Spiels an.
    display::set_treiber(Treiber::Dummy);
    assert_eq!(
        display::get_allow_screensaver(),
        Err(Fehler::NichtInitialisiert)
    );
    display::init().unwrap();
    assert_eq!(display::get_allow_screensaver(), Ok(false));
    display::set_allow_screensaver(true).unwrap();
    assert_eq!(display::get_allow_screensaver(), Ok(true));
    display::set_allow_screensaver(false).unwrap();
    assert_eq!(display::get_allow_screensaver(), Ok(false));
}

#[test]
fn der_schreibtisch_ist_schon_vor_dem_fenster_bekannt() {
    // Gemessen: nach blossem display.init(), noch ohne set_mode, meldet
    // pygame unter dummy get_desktop_sizes() == [(1024, 768)],
    // get_desktop_refresh_rates() == [60] und get_num_displays() == 1.
    display::set_treiber(Treiber::Dummy);
    assert_eq!(display::get_num_displays(), 0, "vor init keiner");
    assert_eq!(
        display::get_desktop_sizes(),
        Err(Fehler::NichtInitialisiert)
    );

    display::init().unwrap();
    assert_eq!(display::get_num_displays(), 1);
    assert_eq!(display::get_desktop_sizes(), Ok(vec![(1024, 768)]));
    assert_eq!(display::get_desktop_refresh_rates(), Ok(vec![60]));

    // Mit Fenster dasselbe.
    display::set_mode((8, 8), 0).unwrap();
    assert_eq!(display::get_desktop_sizes(), Ok(vec![(1024, 768)]));
    assert_eq!(display::list_modes(), vec![(1024, 768)]);

    display::quit();
    assert_eq!(display::get_num_displays(), 0, "nach quit wieder keiner");
}

#[test]
fn die_aktuelle_bildrate_braucht_ein_fenster() {
    // Gemessen: ohne Fenster wirft pygame `No open window`, mit Fenster
    // meldet es unter dummy 60.
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    assert_eq!(display::get_current_refresh_rate(), Err(Fehler::KeinFenster));
    assert_eq!(
        display::get_current_refresh_rate().unwrap_err().to_string(),
        "No open window",
        "ein anderer Wortlaut als bei flip()"
    );
    display::set_mode((8, 8), 0).unwrap();
    assert_eq!(display::get_current_refresh_rate(), Ok(60));
}

#[test]
fn die_fensterstelle_wird_einmal_gesetzt_und_bleibt() {
    // Der Fund, den man ohne Messung nicht ahnt: nach
    // set_mode((320, 240)) sagt get_window_position() (352, 264) - genau
    // ((1024-320)/2, (768-240)/2). Ein zweites set_mode mit **anderer**
    // Groesse aendert daran nichts mehr, das Fenster bleibt liegen.
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    assert_eq!(display::get_window_position(), Err(Fehler::KeinFenster));

    display::set_mode((320, 240), 0).unwrap();
    assert_eq!(display::get_window_position(), Ok((352, 264)));
    display::set_mode((640, 480), 0).unwrap();
    assert_eq!(
        display::get_window_position(),
        Ok((352, 264)),
        "gemessen: das Fenster wird umgebaut, nicht neu gestellt"
    );

    display::set_window_position((10, 20)).unwrap();
    assert_eq!(display::get_window_position(), Ok((10, 20)));
    // WINDOWPOS_CENTERED stellt mittig - fuer die **jetzige** Groesse.
    let mitte = (WINDOWPOS_CENTERED as i32, WINDOWPOS_CENTERED as i32);
    display::set_window_position(mitte).unwrap();
    assert_eq!(display::get_window_position(), Ok(((1024 - 640) / 2, (768 - 480) / 2)));
}

#[test]
fn set_window_position_ohne_fenster_verpufft_klaglos() {
    // Gemessen: pygame gibt None zurueck und stellt das naechste Fenster
    // trotzdem mittig.
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    assert_eq!(display::set_window_position((10, 20)), Ok(()));
    display::set_mode((320, 240), 0).unwrap();
    assert_eq!(display::get_window_position(), Ok((352, 264)));
}

#[test]
fn wm_info_ohne_fenstersystem_ist_leer() {
    // Gemessen: vor init ein Fehler; nach init ein leeres Woerterbuch,
    // und zwar auch dann, wenn schon ein Fenster offen ist.
    display::set_treiber(Treiber::Dummy);
    assert_eq!(display::get_wm_info(), Err(Fehler::NichtInitialisiert));
    display::init().unwrap();
    assert_eq!(display::get_wm_info(), Ok(Vec::new()));
    display::set_mode((8, 8), 0).unwrap();
    assert_eq!(display::get_wm_info(), Ok(Vec::new()));
}

#[test]
fn opengl_gibt_es_hier_nicht() {
    // Gemessen: gl_get_attribute wirft unter dummy woertlich
    // `No dynamic GL support in current SDL video driver (dummy)`.
    vorbereiten((8, 8));
    let f = display::gl_get_attribute(GL_DEPTH_SIZE).unwrap_err();
    assert_eq!(
        f.to_string(),
        "No dynamic GL support in current SDL video driver (dummy)"
    );
    // gl_set_attribute nimmt pygame unter dummy stillschweigend an.
    // Hier nicht: es gibt keinen GL-Kontext, auf den der Wunsch je
    // wirken koennte, und ein Ok() hiesse "wird gemacht".
    assert!(matches!(
        display::gl_set_attribute(GL_DEPTH_SIZE, 24),
        Err(Fehler::NichtUnterstuetzt(_))
    ));
}

#[test]
fn iconify_meldet_erfolg_und_tut_nichts() {
    // Gemessen und entgegen pygames eigener Beschreibung: unter dummy
    // gibt iconify() True zurueck, legt **kein** Ereignis ab, und
    // get_active() bleibt True.
    vorbereiten((8, 8));
    assert_eq!(display::iconify(), Ok(true));
    assert!(event::get().is_empty(), "kein ACTIVEEVENT, kein WINDOWMINIMIZED");
    assert!(display::get_active(), "und aktiv ist es immer noch");
}

#[test]
fn is_fullscreen_und_is_vsync() {
    // Gemessen: ohne Fenster `No open window`; mit gewoehnlichem
    // set_mode beide False; nach set_mode(.., FULLSCREEN) das erste True.
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    assert_eq!(display::is_fullscreen(), Err(Fehler::KeinFenster));
    assert_eq!(display::is_vsync(), Err(Fehler::KeinFenster));

    display::set_mode((64, 48), 0).unwrap();
    assert_eq!(display::is_fullscreen(), Ok(false));
    assert_eq!(
        display::is_vsync(),
        Ok(false),
        "set_mode dieser Kiste nimmt gar kein vsync entgegen"
    );

    display::set_mode((64, 48), FULLSCREEN).unwrap();
    assert_eq!(display::is_fullscreen(), Ok(true));
}

#[test]
fn message_box_gibt_es_hier_nicht() {
    // Gemessen: pygame wirft unter dummy `No message system available`.
    vorbereiten((8, 8));
    let f = display::message_box("Titel", Some("Text"), display::Meldungsart::Info, &["OK"])
        .unwrap_err();
    assert_eq!(f.to_string(), "No message system available");
}

#[test]
fn mode_ok_folgt_den_gemessenen_regeln() {
    // Jede Zeile hier ist ein Aufruf an echtem pygame unter dummy.
    vorbereiten((8, 8));
    assert_eq!(display::mode_ok((640, 480), 0, 0), Ok(24));
    assert_eq!(display::mode_ok((1024, 768), 0, 0), Ok(24), "genau die Flaeche");
    assert_eq!(display::mode_ok((1025, 768), 0, 0), Ok(0), "eine Spalte zu viel");
    assert_eq!(display::mode_ok((1024, 769), 0, 0), Ok(0));
    assert_eq!(display::mode_ok((99999, 99999), 0, 0), Ok(0));
    assert_eq!(display::mode_ok((0, 0), 0, 0), Ok(24));
    assert_eq!(display::mode_ok((-5, -5), 0, 0), Ok(24), "gemessen, ja");

    // Im Vollbild geht nur, was es als Anzeigeart wirklich gibt.
    assert_eq!(display::mode_ok((640, 480), FULLSCREEN, 0), Ok(0));
    assert_eq!(display::mode_ok((800, 600), FULLSCREEN, 0), Ok(0));
    assert_eq!(display::mode_ok((1024, 768), FULLSCREEN, 0), Ok(24));

    // Andere Flaggen aendern nichts.
    assert_eq!(display::mode_ok((640, 480), RESIZABLE, 0), Ok(24));
    assert_eq!(display::mode_ok((640, 480), NOFRAME, 0), Ok(24));
    assert_eq!(display::mode_ok((640, 480), OPENGL, 0), Ok(24));

    // Farbtiefe: 8 bis 32 geht, sonst nicht.
    for t in [8, 12, 15, 16, 24, 32] {
        assert_eq!(display::mode_ok((640, 480), 0, t), Ok(24), "Tiefe {t}");
    }
    for t in [1, 2, 4, 33, 64] {
        assert_eq!(display::mode_ok((640, 480), 0, t), Ok(0), "Tiefe {t}");
    }

    // Und die 24 ist nicht die 32 aus Info() - siehe dort.
    assert_eq!(display::info().bitsize, 32);
}

#[test]
fn gamma_und_farbtafel_gibt_es_hier_nicht() {
    // Gemessen mit offenem Fenster unter dummy: set_gamma und
    // set_gamma_ramp geben beide **False** zurueck - keinen Fehler.
    vorbereiten((8, 8));
    assert!(!display::set_gamma(1.0, 1.0, 1.0));
    assert!(!display::set_gamma(0.5, 0.5, 0.5));
    let rampe = [0u16; 256];
    assert!(!display::set_gamma_ramp(&rampe, &rampe, &rampe));

    // set_palette dagegen wirft - und zwar zwei verschiedene Texte, je
    // nachdem ob schon ein set_mode gelaufen ist.
    assert_eq!(
        display::set_palette(&[(1, 2, 3)]).unwrap_err().to_string(),
        "Display mode is not colormapped"
    );
    display::quit();
    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    assert_eq!(
        display::set_palette(&[]).unwrap_err().to_string(),
        "No display mode is set"
    );
}

// ---------------------------------------------------------------------------
//  event - der Griff
// ---------------------------------------------------------------------------

#[test]
fn der_griff_braucht_ein_fenster() {
    // Gemessen, und beides ueberraschend: vor display.init() ist es ein
    // Fehler; nach init, aber ohne Fenster, laeuft set_grab(True)
    // klaglos durch und bleibt doch wirkungslos - get_grab() sagt
    // weiterhin False. Erst mit Fenster greift es, sogar unter dummy.
    display::set_treiber(Treiber::Dummy);
    assert_eq!(event::get_grab(), Err(Fehler::NichtInitialisiert));
    assert_eq!(event::set_grab(true), Err(Fehler::NichtInitialisiert));

    display::init().unwrap();
    assert_eq!(event::get_grab(), Ok(false));
    assert_eq!(event::set_grab(true), Ok(()), "klaglos");
    assert_eq!(event::get_grab(), Ok(false), "aber wirkungslos");

    display::set_mode((8, 8), 0).unwrap();
    event::set_grab(true).unwrap();
    assert_eq!(event::get_grab(), Ok(true));
    event::set_grab(false).unwrap();
    assert_eq!(event::get_grab(), Ok(false));

    display::quit();
    assert_eq!(event::get_grab(), Err(Fehler::NichtInitialisiert));
}

// ---------------------------------------------------------------------------
//  mouse - Zeigerform, Relativbetrieb, Kanten
// ---------------------------------------------------------------------------

#[test]
fn der_zeiger_faengt_beim_pfeil_an() {
    // Gemessen: get_cursor() ist ganz am Anfang
    // Cursor(type: system, constant: SYSTEM_CURSOR_ARROW) mit data (0,).
    display::set_treiber(Treiber::Dummy);
    assert_eq!(mouse::get_cursor(), Err(Fehler::NichtInitialisiert));
    display::init().unwrap();
    let c = mouse::get_cursor().unwrap();
    assert_eq!(c, mouse::Cursor::System(SYSTEM_CURSOR_ARROW));
    assert_eq!(c.art(), "system");
}

#[test]
fn ohne_fenster_geht_die_eigene_zeigerform_und_die_systemform_nicht() {
    // Gemessen und verkehrt herum, als man denkt: unter dummy wirft
    // set_cursor(SYSTEM_CURSOR_HAND) woertlich
    // `Error while creating system cursor`, waehrend
    // set_cursor(cursors.diamond) klaglos durchgeht und get_cursor().type
    // danach 'bitmap' ist. Der Grund: eine Systemform muss beim
    // Fenstersystem angefordert werden, eine eigene ist nur Speicher.
    vorbereiten((8, 8));

    let f = mouse::set_cursor(mouse::Cursor::System(SYSTEM_CURSOR_HAND)).unwrap_err();
    assert_eq!(f.to_string(), "Error while creating system cursor");
    assert_eq!(
        mouse::get_cursor().unwrap(),
        mouse::Cursor::System(SYSTEM_CURSOR_ARROW),
        "gemessen: nach einem Fehlschlag bleibt die alte Form stehen"
    );

    // Der Rautenzeiger aus pygame.cursors: 16x16, Brennpunkt (7, 7),
    // beide Masken 32 Byte - gemessen.
    let raute = mouse::Cursor::Bitmap {
        groesse: (16, 16),
        hotspot: (7, 7),
        xormasks: vec![0; 32],
        andmasks: vec![0; 32],
    };
    assert_eq!(mouse::set_cursor(raute.clone()), Ok(()));
    assert_eq!(mouse::get_cursor().unwrap().art(), "bitmap");
    assert_eq!(mouse::get_cursor().unwrap(), raute);

    let bild = rustgame_core::Surface::new((8, 8));
    let farbig = mouse::Cursor::farbe((3, 4), bild);
    assert_eq!(mouse::set_cursor(farbig.clone()), Ok(()));
    assert_eq!(mouse::get_cursor().unwrap().art(), "color");
    assert_eq!(mouse::get_cursor().unwrap(), farbig, "dasselbe Bild");

    // set_system_cursor ist nur die alte Schreibweise fuer dasselbe.
    assert!(mouse::set_system_cursor(SYSTEM_CURSOR_ARROW).is_err());
}

#[test]
fn der_relativbetrieb_gibt_es_hier_nicht() {
    // Gemessen: get_relative_mode() antwortet sogar **vor**
    // display.init() klaglos mit False - als einziger Aufruf aus
    // pygame.mouse. set_relative_mode(True) wirft unter dummy
    // `No relative mode implementation available`, set_relative_mode(False)
    // geht durch.
    display::set_treiber(Treiber::Dummy);
    assert!(!mouse::get_relative_mode(), "auch vor init, ohne Fehler");

    vorbereiten((8, 8));
    assert!(!mouse::get_relative_mode());
    let f = mouse::set_relative_mode(true).unwrap_err();
    assert_eq!(f.to_string(), "No relative mode implementation available");
    assert!(!mouse::get_relative_mode());
    assert_eq!(mouse::set_relative_mode(false), Ok(()));
    assert!(!mouse::get_relative_mode());
}

#[test]
fn set_relative_mode_gibt_keinen_alten_zustand_zurueck() {
    // Die naheliegende Vermutung nach `set_visible` - und sie stimmt
    // nicht. Nachgemessen gibt pygame hier `None` zurueck, keinen
    // vorherigen Zustand. Der Rueckgabewert von set_visible ist eine
    // Eigenart genau dieses einen Aufrufs.
    vorbereiten((8, 8));
    assert_eq!(mouse::set_relative_mode(false), Ok(()));
    // Zum Vergleich der Aufruf, der es wirklich so macht:
    assert!(mouse::set_visible(false), "vorher war er sichtbar");
}

#[test]
fn maustasten_melden_ihre_kanten() {
    // Gemessen: get_just_pressed() ist ein Fuenfer-Tupel, und True nur
    // in dem Durchlauf, in dem die Taste heruntergeht.
    vorbereiten((8, 8));
    assert_eq!(mouse::get_just_pressed(), [false; 5]);
    assert_eq!(mouse::get_just_released(), [false; 5]);

    dummy::maus_druecken(1);
    event::get(); // ein Durchlauf
    assert_eq!(mouse::get_just_pressed(), [true, false, false, false, false]);
    assert_eq!(mouse::get_just_released(), [false; 5]);
    assert_eq!(mouse::get_pressed(), (true, false, false));

    event::get(); // der naechste Durchlauf
    assert_eq!(mouse::get_just_pressed(), [false; 5], "nur einen Durchlauf lang");
    assert_eq!(mouse::get_pressed(), (true, false, false), "unten ist sie noch");

    dummy::maus_loslassen(1);
    event::get();
    assert_eq!(mouse::get_just_released(), [true, false, false, false, false]);
    assert_eq!(mouse::get_pressed(), (false, false, false));
}

// ---------------------------------------------------------------------------
//  key - Kanten, ScancodeWrapper, Eingabemethode
// ---------------------------------------------------------------------------

#[test]
fn tasten_melden_ihre_kanten() {
    vorbereiten((8, 8));
    assert!(!key::get_just_pressed()[K_a]);

    dummy::taste_druecken(K_a, "a");
    event::get();
    assert!(key::get_just_pressed()[K_a]);
    assert!(!key::get_just_released()[K_a]);
    assert!(key::get_pressed()[K_a]);

    event::get();
    assert!(!key::get_just_pressed()[K_a], "nur einen Durchlauf lang");
    assert!(key::get_pressed()[K_a], "unten ist sie noch");

    dummy::taste_loslassen(K_a);
    event::get();
    assert!(key::get_just_released()[K_a]);
    assert!(!key::get_pressed()[K_a]);
}

#[test]
fn im_selben_durchlauf_gedrueckt_und_losgelassen() {
    // Gemessen und in pygames eigener Beschreibung: eine Taste kann als
    // "gerade gedrueckt" gelten, obwohl get_pressed() sie als oben
    // meldet - naemlich wenn beides im selben Durchlauf passiert ist.
    vorbereiten((8, 8));
    dummy::taste_tippen(K_SPACE, " ");
    event::get();
    assert!(key::get_just_pressed()[K_SPACE]);
    assert!(key::get_just_released()[K_SPACE]);
    assert!(!key::get_pressed()[K_SPACE], "unten ist sie nicht mehr");
}

#[test]
fn der_scancode_wrapper_wirft_nicht() {
    // Gemessen: get_pressed()[99999] und get_pressed()[-1] sind in
    // pygame beide False, kein IndexError. Negativ gibt es in Rust
    // nicht, aber "viel zu gross" schon.
    vorbereiten((8, 8));
    let t = key::get_pressed();
    assert!(!t[99_999]);
    assert!(!t.roh(99_999));
    assert_eq!(t.len(), NUM_SCANCODES);
    // Und der Kantentabellen-Zwilling ist derselbe Typ.
    let n: key::ScancodeWrapper = key::get_just_pressed();
    assert_eq!(n.len(), 512);
    assert!(!n.irgendeine());
}

#[test]
fn set_text_input_rect_wird_wenigstens_gemerkt() {
    // Gemessen: pygame nimmt das klaglos an und gibt None zurueck, sogar
    // vor display.init(). Zuhoeren tut hier niemand - weder minifb noch
    // das fensterlose Stueck hat eine Eingabemethode -, aber verschwinden
    // soll es auch nicht.
    use rustgame_core::Rect;
    vorbereiten((8, 8));
    assert_eq!(key::get_text_input_rect(), None);
    key::set_text_input_rect(Rect::new(1, 2, 3, 4));
    assert_eq!(key::get_text_input_rect(), Some(Rect::new(1, 2, 3, 4)));
}

// ---------------------------------------------------------------------------
//  Die Naht zum Ton
// ---------------------------------------------------------------------------

/// Ein kurzer Klang, den das stumme Tonstueck in wenigen Millisekunden
/// durchlaeuft.
fn kurzer_klang() -> rustgame_mixer::Sound {
    // 10 ms bei 44100 Hz, zwei Spuren.
    let werte = vec![0i16; 441 * 2];
    rustgame_mixer::Sound::from_samples(&werte, 44100, 2).expect("Klang")
}

#[test]
fn mixer_endereignisse_landen_in_dieser_schlange() {
    // `Channel.set_endevent` ist die einzige Stelle, an der der Ton die
    // Ereignisschlange anfasst. Die Tonkiste kennt diese hier nicht und
    // bietet nur einen Rueckruf an - `display::init()` setzt ihn.
    //
    // Gemessen an echtem pygame (`_pg_push_mixer_event`): das
    // `code`-Feld haengt **nur** an, wenn der Typ im Bereich der
    // Benutzerereignisse liegt, und es ist der Kanalindex.
    use rustgame_mixer::{mixer, treiber::Wahl};

    vorbereiten((8, 8));
    // Ohne das oeffnet die Tonkiste ein echtes Geraet: ihr `cfg!(test)`
    // gilt nur in ihrer eigenen Kiste, nicht in diesem Test.
    mixer::set_treiber(Wahl::Dummy);
    mixer::init(Default::default()).expect("stummer Mixer");

    let kanal = rustgame_mixer::Channel::new(0).expect("Kanal 0");
    kanal.set_endevent(USEREVENT + 5);
    kurzer_klang().play(0, 0, 0).expect("laeuft");

    // Abwarten, bis der Klang durch ist, dann die Tonkiste anstossen -
    // sie fuehrt bei jedem Zugriff nach und schiebt das Ereignis in die
    // Senke.
    std::thread::sleep(std::time::Duration::from_millis(60));
    let _ = kanal.get_busy();

    let evs = event::get();
    assert_eq!(evs.len(), 1, "ein Endereignis, {evs:?}");
    assert_eq!(evs[0].typ, USEREVENT + 5);
    assert_eq!(evs[0].code(), Some(0), "gemessen: das code-Feld ist der Kanal");

    mixer::quit();
    display::quit();
}

#[test]
fn ein_endereignis_ausserhalb_des_nutzerbereichs_traegt_kein_code() {
    // Gemessen: pygame haengt das `code`-Feld nur an, wenn der Typ
    // zwischen USEREVENT und NUMEVENTS liegt. Wer
    // `set_endevent(pygame.QUIT)` schreibt, bekommt ein blankes QUIT.
    use rustgame_mixer::{mixer, treiber::Wahl};

    vorbereiten((8, 8));
    mixer::set_treiber(Wahl::Dummy);
    mixer::init(Default::default()).expect("stummer Mixer");

    let kanal = rustgame_mixer::Channel::new(0).expect("Kanal 0");
    kanal.set_endevent(QUIT);
    kurzer_klang().play(0, 0, 0).expect("laeuft");
    std::thread::sleep(std::time::Duration::from_millis(60));
    let _ = kanal.get_busy();

    let evs = event::get();
    assert_eq!(evs.len(), 1, "{evs:?}");
    assert_eq!(evs[0].typ, QUIT);
    assert_eq!(evs[0].code(), None, "kein code an einem QUIT");

    mixer::quit();
    display::quit();
}

#[test]
fn ohne_laufende_anzeige_legt_der_ton_nichts_ab() {
    // Gemessen: pygame legt gar kein Endereignis ab, wenn das
    // Video-Teilsystem nicht laeuft - die Schlange bleibt leer.
    use rustgame_mixer::{mixer, treiber::Wahl};

    vorbereiten((8, 8));
    mixer::set_treiber(Wahl::Dummy);
    mixer::init(Default::default()).expect("stummer Mixer");
    let kanal = rustgame_mixer::Channel::new(0).expect("Kanal 0");
    kanal.set_endevent(USEREVENT + 6);

    // Anzeige aus - der Rueckruf haengt weiter, wirft aber alles weg.
    display::quit();
    kurzer_klang().play(0, 0, 0).expect("laeuft");
    std::thread::sleep(std::time::Duration::from_millis(60));
    let _ = kanal.get_busy();

    display::set_treiber(Treiber::Dummy);
    display::init().unwrap();
    display::set_mode((8, 8), 0).unwrap();
    event::clear(None);
    assert!(
        event::get().is_empty(),
        "was bei ausgeschalteter Anzeige entstand, ist weg"
    );
    mixer::quit();
}
