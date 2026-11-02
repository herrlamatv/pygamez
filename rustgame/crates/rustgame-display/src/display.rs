//! display.rs
//! ==========
//! Gegenstueck zu `pygame.display`: Fenster, Anzeigeflaeche, Bildwechsel.
//!
//! ## Was an echtem pygame gemessen wurde
//!
//! | Frage | Antwort |
//! |---|---|
//! | `set_mode` zweimal, gleiche Groesse | **dasselbe** Objekt |
//! | `set_mode` zweimal, andere Groesse | **immer noch dasselbe** Objekt, nur umgebaut |
//! | `get_surface()` vor `set_mode` | `None` |
//! | `get_caption()` ohne `set_caption` | `('pygame window', 'pygame window')` |
//! | `set_caption("A", "B")` danach `get_caption()` | `('A', 'A')` - der zweite Titel geht verloren |
//! | `flip()` ohne `set_mode` | Fehler `Display mode not set` |
//! | `toggle_fullscreen()` unter `dummy` | Fehler `That operation is not supported` |
//! | `list_modes()` unter `dummy` | genau ein Eintrag, `(1024, 768)` |
//! | Flaggen der Anzeigeflaeche | `PREALLOC` (16777216), auch bei `RESIZABLE` |
//! | `set_mode(.., FULLSCREEN)` | `FULLSCREEN | PREALLOC`; `RESIZABLE` verschwindet |
//! | Anzeigeflaeche und Alpha | keins - `get_alpha()` ist `None`, jeder Pixel kommt mit A=255 |
//! | `get_desktop_sizes()` nach blossem `init()` | `[(1024, 768)]` - **ohne** Fenster |
//! | `get_desktop_refresh_rates()` | `[60]` |
//! | `get_num_displays()` | `0` vor `init()`, `1` danach - kein Fehler |
//! | `get_window_position()` nach `set_mode((320, 240))` | `(352, 264)` - mittig |
//! | dasselbe nach `set_mode((640, 480))` | **immer noch** `(352, 264)` |
//! | `get_window_position()` ohne Fenster | Fehler `No open window` |
//! | `get_active()` | `False` vor `set_mode`, `True` danach, `False` mit `HIDDEN` |
//! | `iconify()` unter `dummy` | `True` - und es passiert nichts |
//! | `get_wm_info()` unter `dummy` | `{}` |
//! | `gl_get_attribute(..)` | Fehler `No dynamic GL support in current SDL video driver (dummy)` |
//! | `gl_set_attribute(..)` | `None` - pygame nimmt es still an |
//! | `message_box(..)` | Fehler `No message system available` |
//! | `set_gamma(..)`, `set_gamma_ramp(..)` **mit** Fenster | `False` |
//! | `set_palette(..)` | Fehler `Display mode is not colormapped` |
//! | `mode_ok((640, 480))` | `24` - obwohl `Info().bitsize` 32 sagt |
//! | `mode_ok((1025, 768))` | `0` - eine Spalte breiter als der Schreibtisch |
//! | `mode_ok((640, 480), FULLSCREEN)` | `0`; nur `(1024, 768)` geht |
//!
//! Der zweite Punkt hat das Bild dieser Kiste bestimmt: weil `set_mode`
//! die Flaeche umbaut statt sie zu ersetzen, ist [`crate::Screen`] ein
//! geteilter Behaelter und keine nackte `Surface`.
//!
//! Und der Punkt, den man ohne Messung nicht ahnt: **`set_mode` legt fuenf
//! Ereignisse in die Schlange** - siehe [`crate::event`].
//!
//! ## Zwei Wortlaute, die man nicht zusammenwerfen darf
//!
//! `flip()` ohne `set_mode` sagt `Display mode not set`
//! ([`crate::Fehler::KeinModus`]), `get_window_position()` ohne Fenster
//! dagegen `No open window` ([`crate::Fehler::KeinFenster`]). Beides
//! gemessen. Ein Programm, das auf den Text prueft, merkt den
//! Unterschied.

use rustgame_core::{Rect, Surface};

use std::sync::Arc;

use parking_lot::RwLock;

use crate::GELEIHT;
use crate::event::{Daten, Event};
use crate::konstanten::*;
use crate::zustand::mit_zustand;
use crate::{Ergebnis, Fehler, Screen};

pub use crate::zustand::Treiber;

/// Waehlt das Hintergrundstueck fuer das naechste [`set_mode`].
///
/// Die uebliche Wahl geht ueber `RUSTGAME_VIDEODRIVER=dummy`; das hier
/// ist der Weg, wenn ein Programm es selbst entscheiden will - und der
/// Weg, den die Tests gehen, weil eine Umgebungsvariable in einem
/// nebenlaeufigen Testlauf allen anderen dazwischenfunken wuerde.
pub fn set_treiber(t: Treiber) {
    mit_zustand(|z| z.treiber = t)
}

/// Startet die Anzeige - `pygame.display.init()`.
///
/// Ein Fenster gibt es danach noch nicht; das kommt erst mit [`set_mode`].
///
/// Nebenbei wird hier die Naht zum Ton geknuepft - siehe
/// `mixer_senke_anmelden` weiter unten.
pub fn init() -> Ergebnis<()> {
    mixer_senke_anmelden();
    mit_zustand(|z| {
        z.init = true;
        Ok(())
    })
}

thread_local! {
    /// Steht die Naht zum Ton schon? Ein zweites `init()` soll nicht
    /// jedes Mal einen neuen Rueckruf anlegen.
    static SENKE_STEHT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Meldet bei `rustgame-mixer` an, wohin dessen Endereignisse gehoeren.
///
/// `Channel.set_endevent` und `music.set_endevent` sind die einzige
/// Stelle, an der `pygame.mixer` die Ereignisschlange anfasst. Die
/// Tonkiste kennt diese hier nicht (und soll es auch nicht - Ton braucht
/// kein Fenster), also sammelt sie ihre faelligen Ereignisse selbst und
/// bietet unter `ereignis::set_senke` einen Rueckruf an. Gesetzt wird er
/// genau einmal, hier.
///
/// **Was dabei an echtem pygame nachgesehen wurde** - beides steht so in
/// `_pg_push_mixer_event`:
///
/// * Das `code`-Feld haengt **nur** an, wenn der Ereignistyp im Bereich
///   der Benutzerereignisse liegt (`USEREVENT` bis `NUMEVENTS`). Wer
///   `Channel.set_endevent(pygame.QUIT)` schreibt, bekommt ein blankes
///   `QUIT` ohne `code` - und nicht ein `QUIT`, an dem ploetzlich eine
///   Kanalnummer klebt.
/// * pygame legt **gar nichts** ab, wenn das Video-Teilsystem nicht
///   laeuft. Nach `display.quit()` bleibt die Schlange leer, auch wenn
///   weiter Toene zu Ende gehen. Darum die Pruefung auf `z.init`.
///
/// Die Tonkiste sammelt nicht mehr, solange ein Rueckruf haengt - sonst
/// kaeme jedes Ereignis zweimal an. Wer `rustgame_mixer::ereignis::abholen()`
/// selbst aufrufen will, muss also vorher `senke_loeschen()` sagen.
fn mixer_senke_anmelden() {
    if SENKE_STEHT.with(|s| s.replace(true)) {
        return;
    }
    rustgame_mixer::ereignis::set_senke(|e| {
        mit_zustand(|z| {
            // Kein Video-Teilsystem, kein Ereignis - gemessen.
            if !z.init {
                return;
            }
            let daten = match e.code {
                Some(kanal) if (USEREVENT..NUMEVENTS).contains(&e.typ) => Daten::Nutzer {
                    code: kanal as i32,
                },
                _ => Daten::Leer,
            };
            z.schlange.ablegen(Event::mit(e.typ, daten));
        })
    });
}

/// Faehrt die Anzeige herunter - `pygame.display.quit()`.
///
/// Gemessen: danach ist `get_init()` falsch, `get_surface()` leer,
/// `get_num_displays()` wieder `0`, `get_active()` falsch, und
/// `get_grab()` wirft wieder `video system not initialized`.
pub fn quit() {
    mit_zustand(|z| {
        z.backend = None;
        z.screen = None;
        z.init = false;
        z.flags = 0;
        z.schlange.eintraege.clear();
        // **Und die Sperrliste.** Gemessen an echtem pygame mit nur
        // `display.init()` (ohne `pygame.init()`, das eine zweite
        // Zaehlung auf dem Videosystem haelt):
        //
        // ```python
        // display.init(); event.set_blocked(USEREVENT)
        // event.get_blocked(USEREVENT)              # True
        // display.quit(); display.init()
        // event.get_blocked(USEREVENT)              # False
        // ```
        //
        // SDL setzt beim Herunterfahren des Videosystems seine
        // Ereigniszustaende zurueck, und pygame fuehrt keine eigene
        // Tabelle darueber.
        //
        // Das war **elf** Tests in `event_test.py` wert, und keiner davon
        // sah danach aus: `EventModuleTest` sperrt in
        // `test_get_blocked__event_sequence` sieben Typen und nimmt es nie
        // zurueck; sein `tearDown` ruft nur `display.quit()`. Alle spaeter
        // laufenden Tests, die `USEREVENT` oder `KEYDOWN` abschicken,
        // bekamen hier nichts zurueck - `post` gab `False` und `poll` die
        // Null. Einzeln gefahren war jeder dieser Tests gruen.
        //
        // **Vorsicht bei der Gegenmessung:** ist `pygame.init()` aktiv,
        // raeumt `display.quit(); display.init()` die Liste **nicht** -
        // dann steht das Videosystem noch. Meine erste Messung lief genau
        // in diese Falle und sagte "pygame raeumt auch nicht", woraufhin
        // ich zwei Stunden an der falschen Stelle gesucht habe.
        z.schlange.gesperrt.clear();
        z.schlange.alles_gesperrt = false;
        // **Und die eigenen Ereignistypen sind wieder frei.**
        //
        // Gemessen: `custom_type()` zweimal, dann `pygame.quit()` und
        // `pygame.init()` - danach gibt `custom_type()` **wieder die
        // erste** Nummer. `event_test.test_custom_type__reset` heisst
        // ausdruecklich so ("Ensure custom events get 'deregistered' by
        // quit()").
        //
        // Ohne diese Zeile zaehlt der Zaehler ueber die ganze Sitzung
        // weiter. Das ist die stillste Sorte Abweichung: ein Programm
        // laeuft trotzdem, bekommt aber andere Zahlen als pygame - und wer
        // sie irgendwo abgelegt hat, greift daneben.
        z.schlange.naechster_eigener = crate::konstanten::USEREVENT + 1;
        // **Und der Fenstertitel steht wieder auf der Vorgabe.**
        //
        // Gemessen: `set_caption("test")`, dann `quit()` und `init()` -
        // pygame sagt danach wieder `('pygame window', 'pygame window')`.
        // Das Fenster ist ja fort, und mit ihm sein Titel; was bleibt, ist
        // der Name, den SDL einem neuen Fenster gibt.
        z.caption = ("pygame window".into(), "pygame window".into());
        z.caption_gesetzt = false;
        z.eingabe = Default::default();
        z.griff = false;
        z.bildschirmschoner = false;
        // Gemessen: nach `display.quit()` ist auch jedes `Window`
        // hinueber - siehe [`crate::fenster::Ablage::leeren`].
        z.fenster.leeren();
    })
}

/// Laeuft die Anzeige? - `pygame.display.get_init()`.
pub fn get_init() -> bool {
    mit_zustand(|z| z.init)
}

/// Welche Groesse die Anzeigeflaeche wirklich bekommt.
///
/// Drei Regeln, alle an echtem pygame unter `SDL_VIDEODRIVER=dummy`
/// gemessen:
///
/// | Aufruf | Ergebnis |
/// |---|---|
/// | `set_mode((64, 48), FULLSCREEN)` | `(1024, 768)` |
/// | `set_mode((1024, 768), FULLSCREEN)` | `(1024, 768)` |
/// | `set_mode((2000, 2000), FULLSCREEN)` | `(1024, 768)` |
/// | `set_mode((0, 0))` | `(1024, 768)` |
/// | `set_mode((0, 48))` | `(1, 48)` |
/// | `set_mode((64, 0))` | `(64, 1)` |
/// | `set_mode((2000, 2000))` | `(2000, 2000)` |
///
/// Im **Vollbild** gilt also immer die Arbeitsflaeche, egal was
/// dasteht - auch eine Groesse, die [`mode_ok`] ablehnt. Und `(0, 0)`
/// heisst "so gross wie die Arbeitsflaeche", waehrend eine **einzelne**
/// Null nur auf 1 aufgerundet wird. Der Unterschied sieht nach einer
/// Spitzfindigkeit aus und ist keiner: `(0, 0)` ist die uebliche
/// Schreibweise fuer "nimm den ganzen Bildschirm".
///
/// Hier stand vorher schlicht `groesse.0.max(1)` fuer beide Achsen.
/// Das traf `(0, 48)` und `(64, 0)` richtig und die beiden haeufigen
/// Faelle daneben.
fn gewuenschte_groesse(groesse: (u32, u32), flags: u32) -> Ergebnis<(u32, u32)> {
    if flags & crate::konstanten::FULLSCREEN != 0 || groesse == (0, 0) {
        let desktop = get_desktop_sizes()?;
        if let Some(g) = desktop.first().copied() {
            return Ok(g);
        }
    }
    Ok((groesse.0.max(1), groesse.1.max(1)))
}

/// Oeffnet das Fenster und liefert die Anzeigeflaeche - `pygame.display.set_mode()`.
///
/// `flags` nimmt die Werte aus [`crate::konstanten`]; ausgewertet werden
/// [`RESIZABLE`] und [`NOFRAME`], der Rest wird gemerkt und von
/// [`Info`] weitergereicht.
///
/// Beim zweiten Aufruf kommt **dieselbe** Flaeche zurueck, auch bei einer
/// anderen Groesse - das ist an echtem pygame gemessen und nicht bloss
/// bequem: Programme halten den Rueckgabewert fest und erwarten, dass er
/// nach einem Groessenwechsel weiterhin das Fenster zeigt.
///
/// Nebenbei legt der Aufruf fuenf Ereignisse ab, genau wie pygame:
/// `ACTIVEEVENT(gain=1, state=APPMOUSEFOCUS)`, `WINDOWENTER`,
/// `ACTIVEEVENT(gain=1, state=APPINPUTFOCUS)`, `WINDOWFOCUSGAINED`,
/// `WINDOWSHOWN`.
pub fn set_mode(groesse: (u32, u32), flags: u32) -> Ergebnis<Screen> {
    let (w, h) = gewuenschte_groesse(groesse, flags)?;
    let titel = mit_zustand(|z| z.caption.0.clone());
    let treiber = mit_zustand(|z| z.treiber);

    // Das Stueck wird ausserhalb der Ausleihe geoeffnet - ein Fenster
    // aufzumachen kann dauern, und solange darf der Zustand nicht
    // festhaengen.
    let neues = mit_zustand(|z| z.backend.is_none());
    let gebaut = if neues {
        Some(crate::backend::oeffnen(treiber, (w, h), flags, &titel)?)
    } else {
        None
    };

    mit_zustand(|z| {
        z.init = true;
        z.flags = flags;
        if let Some(b) = gebaut {
            z.backend = Some(b);
        } else if let Some(b) = z.backend.as_mut() {
            b.set_mode((w, h), flags)?;
        }

        match &z.screen {
            // Gemessen: dieselbe Flaeche wird umgebaut, nicht ersetzt.
            Some(s) if s.try_read().expect(GELEIHT).get_size() != (w, h) => {
                *s.try_write().expect(GELEIHT) = neue_anzeigeflaeche((w, h), flags);
            }
            Some(_) => {}
            None => {
                z.screen = Some(Arc::new(RwLock::new(neue_anzeigeflaeche((w, h), flags))));
            }
        }

        // Die fuenf Ereignisse, die echtes pygame von sich aus ablegt.
        for e in eroeffnungsereignisse() {
            z.schlange.ablegen(e);
        }
        z.eingabe.tast_fokus = true;
        z.eingabe.maus_fokus = true;

        // Damit `Window.from_display_module()` dieses Fenster findet -
        // und damit es, gemessen, seine eigene Nummer in derselben
        // Reihe bekommt wie ein von Hand angelegtes.
        crate::fenster::anzeigefenster(z);

        Ok(z.screen.clone().expect("gerade gesetzt"))
    })
}

/// Eine frische Anzeigeflaeche.
///
/// Ohne `SRCALPHA`: gemessen hat die Anzeigeflaeche von pygame keinen
/// Alphakanal (`get_alpha()` ist `None`), und jeder gelesene Pixel kommt
/// mit A=255 heraus.
///
/// Die Flaggen sind ebenfalls gemessen: `get_flags()` liefert bei pygame
/// [`PREALLOC`] (16777216) - auch dann, wenn `set_mode` mit
/// [`RESIZABLE`] gerufen wurde; das Flag verschwindet. Nur
/// [`FULLSCREEN`] bleibt stehen (`FULLSCREEN | PREALLOC`).
/// `PREALLOC` heisst bei SDL "der Pixelspeicher gehoert nicht dieser
/// Flaeche" - fuer die Anzeigeflaeche stimmt das, sie zeigt ins Fenster.
fn neue_anzeigeflaeche(groesse: (u32, u32), flags: u32) -> Surface {
    let flaechenflaggen = PREALLOC | (flags & FULLSCREEN);
    let mut s = Surface::with_flags(groesse, flaechenflaggen);
    s.fill((0, 0, 0));
    s
}

/// Die fuenf Ereignisse, die `set_mode` bei echtem pygame ablegt.
///
/// Reihenfolge und Felder sind gemessen, nicht geraten.
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

/// Die Anzeigeflaeche - `pygame.display.get_surface()`.
///
/// `None`, solange kein [`set_mode`] gelaufen ist.
pub fn get_surface() -> Option<Screen> {
    mit_zustand(|z| z.screen.clone())
}

/// Zeigt die ganze Flaeche an - `pygame.display.flip()`.
///
/// Wer waehrend des Aufrufs noch eine Leihe auf die Flaeche haelt,
/// bekommt eine Panik - Zeichnen und Anzeigen gehoeren in getrennte
/// Bloecke. Siehe [`crate::GELEIHT`].
pub fn flip() -> Ergebnis<()> {
    mit_zustand(|z| {
        let screen = z.screen.clone().ok_or(Fehler::KeinModus)?;
        let backend = z.backend.as_mut().ok_or(Fehler::KeinModus)?;
        let mut s = screen.try_write().expect(GELEIHT);
        let groesse = s.get_size();
        // `mit_pixeln` reicht den ganzen Puffer am Stueck durch; fuer die
        // Anzeigeflaeche ist der Versatz null und die Zeilenbreite gleich
        // der Breite, also geht er unveraendert ins Fenster.
        s.mit_pixeln(|daten, _zeilenbreite, _versatz| backend.zeigen(daten, groesse))
    })
}

/// Zeigt die ganze Flaeche an - `pygame.display.update()` ohne Argument.
///
/// Gleichbedeutend mit [`flip`].
pub fn update() -> Ergebnis<()> {
    flip()
}

/// Zeigt Ausschnitte an - `pygame.display.update(rects)`.
///
/// pygame kann damit Arbeit sparen, weil SDL nur die genannten Kacheln
/// hochlaedt. Weder minifb noch das fensterlose Stueck koennen das: minifb
/// kennt nur ganze Bilder. Die Rechtecke werden deshalb zusammengefasst
/// und das ganze Bild geschickt - richtig sieht es in jedem Fall aus, nur
/// nicht schneller.
///
/// Eine leere Liste zeigt nichts an; `None`-Eintraege werden
/// uebersprungen, wie bei pygame.
pub fn update_rects(rects: &[Option<Rect>]) -> Ergebnis<()> {
    update_ausschnitte(rects.iter().all(|r| r.is_none()))
}

/// Zeigt Ausschnitte an - dieselbe Sache wie [`update_rects`], nur ohne
/// die `Option`-Huelle, wenn ohnehin alle Rechtecke da sind.
pub fn update_bereiche(rects: &[Rect]) -> Ergebnis<()> {
    update_ausschnitte(rects.is_empty())
}

fn update_ausschnitte(nichts_zu_tun: bool) -> Ergebnis<()> {
    if nichts_zu_tun {
        // Nichts zu tun - aber die Pruefung auf `set_mode` bleibt, damit
        // der Fehler derselbe ist wie bei pygame.
        return mit_zustand(|z| {
            if z.screen.is_none() || z.backend.is_none() {
                Err(Fehler::KeinModus)
            } else {
                Ok(())
            }
        });
    }
    flip()
}

/// Setzt den Fenstertitel - `pygame.display.set_caption()`.
///
/// Gemessen: pygame nimmt zwar einen zweiten Titel fuers Sinnbild
/// entgegen, gibt ihn aber nie wieder heraus - `set_caption("A", "B")`
/// gefolgt von `get_caption()` liefert `('A', 'A')`. Deshalb gibt es hier
/// nur einen.
pub fn set_caption(titel: &str) {
    mit_zustand(|z| {
        z.caption = (titel.to_string(), titel.to_string());
        z.caption_gesetzt = true;
        if let Some(b) = z.backend.as_mut() {
            b.set_caption(titel);
        }
    })
}

/// Hat jemand den Titel ausdruecklich gesetzt?
///
/// Siehe das Feld `caption_gesetzt` im Zustand - kein Doku-Verweis, denn
/// der Zustand ist modulprivat, und rustdoc warnt zu Recht.
///
/// `display.get_caption()` ist leer, solange kein Fenster offen ist und
/// niemand einen Titel gesetzt hat. Gemessen an pygame:
///
/// ```text
/// frisch eingefuehrt      ()
/// nach set_caption('x')   ('x', 'x')
/// nach init() ohne Modus  ()
/// nach set_mode           ('pygame window', 'pygame window')
/// nach quit()             ()
/// ```
pub fn caption_gesetzt() -> bool {
    mit_zustand(|z| z.caption_gesetzt)
}

/// Fenstertitel und Sinnbildtitel - `pygame.display.get_caption()`.
///
/// Beide sind immer gleich; siehe [`set_caption`].
pub fn get_caption() -> (String, String) {
    mit_zustand(|z| z.caption.clone())
}

/// Setzt das Fenstersinnbild - `pygame.display.set_icon()`.
///
/// Das fensterlose Stueck nimmt es entgegen und tut nichts. minifb meldet
/// [`Fehler::NichtUnterstuetzt`]: es nimmt ein Sinnbild nur als Dateipfad
/// oder als Rohzeiger, dessen Lebensdauer es nicht uebernimmt - daraus
/// laesst sich aus einer Flaeche nichts Sicheres bauen.
pub fn set_icon(bild: &Surface) -> Ergebnis<()> {
    mit_zustand(|z| match z.backend.as_mut() {
        Some(b) => b.set_icon(bild),
        None => Err(Fehler::KeinModus),
    })
}

/// Welche Vollbild-Groessen es gibt - `pygame.display.list_modes()`.
///
/// Gemessen: unter `SDL_VIDEODRIVER=dummy` meldet pygame genau eine,
/// `(1024, 768)`.
pub fn list_modes() -> Vec<(u32, u32)> {
    // **Auch ohne Fenster** - dieselbe Ueberlegung wie bei
    // [`get_desktop_sizes`] und [`get_driver`]: ohne offenes Stueck
    // entscheidet, wer bei einem [`set_mode`] geoeffnet **wuerde**.
    //
    // Hier stand `None => Vec::new()`, also "kein Fenster, keine Modi".
    // Das klingt vorsichtig und ist falsch: gemessen gibt echtes pygame
    // nach blossem `display.init()` schon `[(1024, 768)]`. Vier Tests in
    // `display_test.py` hingen daran, und zwar mit
    // `IndexError: list index out of range` auf `modes[0]` - eine
    // Meldung, die von einer leeren Liste erzaehlt und nicht davon,
    // **warum** sie leer war. `DisplayModuleTest.setUp` ruft naemlich nur
    // `display.init()`, kein `set_mode`.
    ohne_fenster_beantworten(
        |b| Ok(b.list_modes()),
        || Ok(vec![crate::backend::dummy::DESKTOP]),
    )
    .unwrap_or_default()
}

/// Die Groesse des Fensters - `pygame.display.get_window_size()`.
pub fn get_window_size() -> Ergebnis<(u32, u32)> {
    mit_zustand(|z| match z.screen.as_ref() {
        Some(s) => Ok(s.try_read().expect(GELEIHT).get_size()),
        None => Err(Fehler::KeinModus),
    })
}

/// Der Name des Treibers - `pygame.display.get_driver()`.
///
/// Gemessen: echtes pygame antwortet schon nach blossem [`init`], **ohne**
/// Fenster - unter `SDL_VIDEODRIVER=dummy` mit `'dummy'`. In `display.c`
/// steht dort nur `VIDEO_INIT_CHECK()` und dann
/// `SDL_GetCurrentVideoDriver()`; der Treiber wird bei `SDL_VideoInit`
/// gewaehlt, nicht beim Fensteroeffnen.
///
/// Hier stand vorher `z.backend.as_ref()`, also **kein Fenster, kein
/// Name** - und weil `event_test.py` und `display_test.py` in ihrer
/// **neunten** Zeile `pygame.display.get_driver()` nach blossem `init()`
/// rufen, brach das Laden beider Dateien ab; keiner ihrer Tests wurde
/// ueberhaupt sichtbar. Ohne Fenster entscheidet also, wer bei einem
/// [`set_mode`] gleich geoeffnet **wuerde** - dieselbe Ueberlegung wie
/// bei [`get_desktop_sizes`].
pub fn get_driver() -> Ergebnis<&'static str> {
    mit_zustand(|z| {
        if !z.init {
            return Err(Fehler::NichtInitialisiert);
        }
        Ok(match z.backend.as_ref() {
            Some(b) => b.name(),
            None => match crate::backend::gewaehlter_treiber(z.treiber) {
                Treiber::Dummy => "dummy",
                _ => "minifb",
            },
        })
    })
}

/// Schaltet zwischen Fenster und Vollbild - `pygame.display.toggle_fullscreen()`.
///
/// Gemessen: echtes pygame gibt hier unter dem Dummy-Treiber keinen
/// Misserfolgswert zurueck, sondern wirft `That operation is not
/// supported`. Deshalb steht hier ein `Result` und keine 0 - ein stiller
/// Fehlschlag waere der Fehler, den man erst im Vollbild merkt.
/// minifb kann es ebenfalls nicht.
pub fn toggle_fullscreen() -> Ergebnis<()> {
    mit_zustand(|z| {
        let b = z.backend.as_mut().ok_or(Fehler::KeinModus)?;
        b.toggle_fullscreen()?;
        z.flags ^= FULLSCREEN;
        Ok(())
    })
}

/// Was `pygame.display.Info()` erzaehlt.
///
/// Alle Werte sind an echtem pygame unter dem Dummy-Treiber gemessen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Info {
    /// `hw` - Flaeche im Grafikspeicher? Gemessen: 0.
    pub hw: i32,
    /// `wm` - gibt es einen Fensterverwalter? Gemessen: 1.
    pub wm: i32,
    /// `video_mem` - freier Grafikspeicher in Megabyte. Gemessen: 0.
    pub video_mem: i32,
    /// `bitsize` - Bits je Pixel. Gemessen: 32.
    pub bitsize: u8,
    /// `bytesize` - Bytes je Pixel. Gemessen: 4.
    pub bytesize: u8,
    /// `masks` - wo Rot, Gruen, Blau und Alpha im Pixel sitzen.
    ///
    /// Gemessen: `(0x00ff0000, 0x0000ff00, 0x000000ff, 0x00000000)` -
    /// genau das ARGB8888, in dem `rustgame-core` rechnet, nur ohne
    /// Alphamaske, weil die Anzeigeflaeche keins fuehrt.
    pub masks: (u32, u32, u32, u32),
    /// `shifts` - gemessen `(16, 8, 0, 0)`.
    pub shifts: (u8, u8, u8, u8),
    /// `losses` - gemessen `(0, 0, 0, 8)`.
    pub losses: (u8, u8, u8, u8),
    /// `current_w` - Breite der Anzeigeflaeche.
    pub current_w: u32,
    /// `current_h` - Hoehe der Anzeigeflaeche.
    pub current_h: u32,
    /// `pixel_format` - gemessen `"PIXELFORMAT_RGB888"`.
    pub pixel_format: &'static str,
}

// ---------------------------------------------------------------------------
//  Was das Fenstersystem weiss
// ---------------------------------------------------------------------------
//
// Ab hier wird gefragt, nicht gerechnet: wie viele Bildschirme es gibt,
// wo das Fenster steht, wie schnell der Schirm aufbaut. Das fensterlose
// Stueck kann darauf antworten, weil echtes pygame unter
// `SDL_VIDEODRIVER=dummy` eine feste Antwort gibt und die hier gemessen
// steht. minifb kann einiges davon nicht - dann steht hier ein
// [`Fehler::NichtUnterstuetzt`] samt Grund und keine erfundene Zahl.

/// Ist die Anzeige sichtbar? - `pygame.display.get_active()`.
///
/// Gemessen: `False` vor jedem [`set_mode`], `True` danach - und
/// `False`, wenn `set_mode` mit [`HIDDEN`] gerufen wurde. Nach
/// [`quit`] wieder `False`.
///
/// Das hat **nichts** mit dem Eingabefokus zu tun; dafuer gibt es
/// [`crate::key::get_focused`]. Ein Fenster, das hinter einem anderen
/// liegt, ist weiterhin "aktiv".
pub fn get_active() -> bool {
    mit_zustand(|z| z.backend.is_some() && z.flags & HIDDEN == 0)
}

/// Darf der Bildschirmschoner laufen? - `pygame.display.get_allow_screensaver()`.
///
/// Gemessen: die Vorgabe ist `False` - pygame haelt den
/// Bildschirmschoner waehrend des Spiels an. Vor [`init`] ist es ein
/// Fehler.
pub fn get_allow_screensaver() -> Ergebnis<bool> {
    mit_zustand(|z| {
        if !z.init {
            return Err(Fehler::NichtInitialisiert);
        }
        Ok(z.bildschirmschoner)
    })
}

/// Erlaubt oder verbietet den Bildschirmschoner -
/// `pygame.display.set_allow_screensaver()`.
///
/// Gemessen: der Wert wird gemerkt und von [`get_allow_screensaver`]
/// wieder herausgegeben, auch unter dem Dummy-Treiber.
///
/// Ob wirklich ein Bildschirmschoner angehalten wird, sagt auch pygame
/// nicht - seine eigene Beschreibung warnt ausdruecklich, dass der
/// Aufruf "silently appear to work" kann, weil SDL keinen Weg bietet,
/// das nachzufragen. Hier gilt dasselbe, nur ehrlicher gesagt: weder
/// minifb noch das fensterlose Stueck reden mit der
/// Energieverwaltung. Gemerkt wird es trotzdem, damit ein Programm
/// seinen eigenen Schalter wiederfindet.
pub fn set_allow_screensaver(an: bool) -> Ergebnis<()> {
    mit_zustand(|z| {
        if !z.init {
            return Err(Fehler::NichtInitialisiert);
        }
        z.bildschirmschoner = an;
        Ok(())
    })
}

/// Wie oft der Bildschirm aufbaut, auf dem das Fenster steht -
/// `pygame.display.get_current_refresh_rate()`.
///
/// Gemessen: unter `SDL_VIDEODRIVER=dummy` ist das `60`; ohne Fenster
/// wirft pygame `No open window`.
///
/// Mit minifb gibt es die Zahl nicht: minifb fragt den Bildschirm nie.
/// `set_target_fps` setzt nur seine eigene Bremse und weiss nichts
/// darueber, wie schnell der Schirm wirklich ist. Hier `60`
/// zurueckzugeben waere auf einem 144-Hz-Schirm falsch - und zwar auf
/// die Art, die man erst merkt, wenn das Spiel ruckelt.
pub fn get_current_refresh_rate() -> Ergebnis<u32> {
    mit_zustand(|z| match z.backend.as_ref() {
        Some(b) => b.bildwiederholrate(),
        None if z.init => Err(Fehler::KeinFenster),
        None => Err(Fehler::NichtInitialisiert),
    })
}

/// Die Bildwiederholraten aller Arbeitsflaechen -
/// `pygame.display.get_desktop_refresh_rates()`.
///
/// Gemessen: unter dem Dummy-Treiber `[60]`, und zwar schon nach
/// blossem [`init`], noch ohne Fenster. Fuer minifb siehe
/// [`get_current_refresh_rate`] - dieselbe Begruendung.
pub fn get_desktop_refresh_rates() -> Ergebnis<Vec<u32>> {
    ohne_fenster_beantworten(
        |b| b.bildwiederholraten(),
        || Ok(vec![crate::backend::dummy::BILDRATE]),
    )
}

/// Die Groessen aller Arbeitsflaechen - `pygame.display.get_desktop_sizes()`.
///
/// Gemessen: unter dem Dummy-Treiber `[(1024, 768)]`, ebenfalls schon
/// ohne Fenster. Vor [`init`] wirft pygame `video system not
/// initialized`.
///
/// Das ist der Aufruf, mit dem pygame selbst [`list_modes`] ersetzt
/// haben will - er sagt, wie gross der Schreibtisch *jetzt* ist, nicht
/// was die Hardware koennte. Mit minifb gibt es ihn nicht: minifb kennt
/// nur sein eigenes Fenster und hat keine Bildschirmliste. Eine Zahl zu
/// erfinden waere hier besonders teuer, denn wer danach fragt, will
/// meist ein Fenster einpassen.
pub fn get_desktop_sizes() -> Ergebnis<Vec<(u32, u32)>> {
    ohne_fenster_beantworten(
        |b| b.desktop_groessen(),
        || Ok(vec![crate::backend::dummy::DESKTOP]),
    )
}

/// Beantwortet eine Frage, die pygame auch **ohne** Fenster beantwortet.
///
/// Mit offenem Stueck fragt sie dieses. Ohne Stueck gibt es niemanden zu
/// fragen - dann entscheidet, wer bei einem [`set_mode`] gleich
/// geoeffnet **wuerde**: beim fensterlosen Stueck ist die Antwort
/// bekannt und gemessen, bei minifb nicht.
fn ohne_fenster_beantworten<T>(
    vom_stueck: impl FnOnce(&dyn crate::backend::Backend) -> Ergebnis<T>,
    ohne_stueck: impl FnOnce() -> Ergebnis<T>,
) -> Ergebnis<T> {
    mit_zustand(|z| {
        if !z.init {
            return Err(Fehler::NichtInitialisiert);
        }
        match z.backend.as_ref() {
            Some(b) => vom_stueck(b.as_ref()),
            None => match crate::backend::gewaehlter_treiber(z.treiber) {
                Treiber::Dummy => ohne_stueck(),
                _ => Err(Fehler::NichtUnterstuetzt(
                    "minifb cannot query the displays",
                )),
            },
        }
    })
}

/// Wie viele Bildschirme es gibt - `pygame.display.get_num_displays()`.
///
/// Gemessen: `0` vor [`init`] und `1` danach - **ohne** Fehler, auch
/// vor dem ersten [`set_mode`]. Das ist der eine Fall, in dem eine
/// Antwort ohne Fenstersystem trotzdem stimmt: es gibt hier genau ein
/// Fenster, also genau einen Bildschirm, auf dem es liegt.
pub fn get_num_displays() -> usize {
    mit_zustand(|z| usize::from(z.init))
}

/// Wo das Fenster auf dem Schirm steht - `pygame.display.get_window_position()`.
///
/// Gemessen unter `SDL_VIDEODRIVER=dummy`, und der Punkt, den man ohne
/// Messung nicht raet: das Fenster wird **einmal** mittig auf der
/// Arbeitsflaeche gesetzt und bleibt dann liegen. Nach
/// `set_mode((320, 240))` steht dort `(352, 264)`, und das ist genau
/// `((1024-320)/2, (768-240)/2)`. Ein zweites `set_mode` mit einer
/// anderen Groesse aendert die Stelle **nicht** mehr - auch nicht,
/// wenn das Fenster dadurch gar nicht mehr mittig ist.
///
/// Ohne Fenster wirft pygame `No open window` - ein anderer Text als
/// das `Display mode not set` von [`flip`]; siehe [`Fehler::KeinFenster`].
///
/// Mit minifb ist das echt: `Window::get_position()`.
pub fn get_window_position() -> Ergebnis<(i32, i32)> {
    mit_zustand(|z| match z.backend.as_ref() {
        Some(b) => b.fenster_pos(),
        None if z.init => Err(Fehler::KeinFenster),
        None => Err(Fehler::NichtInitialisiert),
    })
}

/// Versetzt das Fenster - `pygame.display.set_window_position()`.
///
/// [`WINDOWPOS_CENTERED`] fuer eine oder beide Achsen stellt mittig.
///
/// Gemessen: **ohne** Fenster ist der Aufruf kein Fehler, er verpufft
/// nur - pygame gibt `None` zurueck, und das naechste `set_mode` stellt
/// das Fenster trotzdem mittig. Mit Fenster wirkt er sofort.
///
/// Mit minifb ist das echt: `Window::set_position()`. Nur
/// [`WINDOWPOS_CENTERED`] kann minifb nicht - es weiss nicht, wie gross
/// die Arbeitsflaeche ist.
pub fn set_window_position(pos: (i32, i32)) -> Ergebnis<()> {
    mit_zustand(|z| {
        if !z.init {
            return Err(Fehler::NichtInitialisiert);
        }
        match z.backend.as_mut() {
            Some(b) => b.set_fenster_pos(pos),
            // Gemessen: ohne Fenster verpufft es klaglos.
            None => Ok(()),
        }
    })
}

/// Was das Fenstersystem ueber sich erzaehlt - `pygame.display.get_wm_info()`.
///
/// pygame liefert ein Woerterbuch mit Schluesseln, die das System sich
/// selbst ausdenkt; ueblich ist `"window"` mit der Fensternummer.
///
/// Gemessen: unter `SDL_VIDEODRIVER=dummy` ist es **leer** - es gibt
/// kein Fenstersystem, ueber das es etwas zu sagen gaebe; und zwar auch
/// dann, wenn schon ein Fenster offen ist. Vor [`init`] wirft pygame
/// dagegen `video system not initialized`, ein Fenster braucht es aber
/// nicht.
///
/// Mit minifb steht genau der eine uebliche Schluessel drin, `"window"`,
/// mit der Fensternummer des Systems als Zahl - dasselbe, was SDL dort
/// ablegt.
pub fn get_wm_info() -> Ergebnis<Vec<(&'static str, u64)>> {
    mit_zustand(|z| {
        if !z.init {
            return Err(Fehler::NichtInitialisiert);
        }
        Ok(match z.backend.as_ref() {
            Some(b) => b.wm_info(),
            None => Vec::new(),
        })
    })
}

// ---------------------------------------------------------------------------
//  OpenGL - gibt es hier nicht
// ---------------------------------------------------------------------------

/// Fragt ein OpenGL-Merkmal ab - `pygame.display.gl_get_attribute()`.
///
/// **Diese Kiste hat kein OpenGL, und das steht hier statt einer Zahl.**
/// Nicht weil es muehsam waere, sondern weil es keinen Kontext gibt,
/// ueber den man etwas sagen koennte: `minifb` schiebt einen
/// Pixelpuffer ins Fenster und macht nie einen GL-Kontext auf, und das
/// fensterlose Stueck hat ueberhaupt kein Fenster. Jede Antwort waere
/// erfunden - und ein Programm, das `gl_get_attribute(GL_DEPTH_SIZE)`
/// fragt, will danach zeichnen.
///
/// Gemessen: echtes pygame wirft unter `SDL_VIDEODRIVER=dummy`
/// woertlich `No dynamic GL support in current SDL video driver
/// (dummy)`. Genau der Text steht auch hier.
pub fn gl_get_attribute(_merkmal: u32) -> Ergebnis<i32> {
    Err(Fehler::Pygame(
        "No dynamic GL support in current SDL video driver (dummy)",
    ))
}

/// Wuenscht sich ein OpenGL-Merkmal fuer das naechste [`set_mode`] -
/// `pygame.display.gl_set_attribute()`.
///
/// Gemessen: echtes pygame nimmt das unter dem Dummy-Treiber
/// stillschweigend an und gibt `None` zurueck - der Wunsch landet in
/// SDL und wartet auf einen GL-Kontext, der nie kommt.
///
/// Hier wird er **nicht** stillschweigend angenommen. Der Grund steht
/// bei [`gl_get_attribute`]: es gibt keinen Kontext, auf den der Wunsch
/// je wirken koennte, und ein `Ok(())` hiesse "gemerkt, wird gemacht".
/// Ein Programm, das `set_mode(.., OPENGL)` plant, soll es an dieser
/// Stelle erfahren und nicht erst beim ersten leeren Bild.
pub fn gl_set_attribute(_merkmal: u32, _wert: i32) -> Ergebnis<()> {
    Err(Fehler::NichtUnterstuetzt(
        "OpenGL support is not available: neither minifb nor the windowless backend creates a GL context",
    ))
}

// ---------------------------------------------------------------------------
//  Fensterzustand
// ---------------------------------------------------------------------------

/// Legt das Fenster als Sinnbild ab - `pygame.display.iconify()`.
///
/// Gemessen, und ueberraschend: unter `SDL_VIDEODRIVER=dummy` gibt
/// pygame hier **`True`** zurueck und tut dabei nichts. Kein
/// `ACTIVEEVENT`, kein `WINDOWMINIMIZED`, und [`get_active`] steht
/// danach immer noch auf `true` - obwohl pygames eigene Beschreibung
/// beides verspricht. Wer darauf wartet, wartet ewig.
///
/// Das fensterlose Stueck macht es genauso. minifb kann es nicht: es
/// hat kein `minimize()`, und an das Fenster des Systems kaeme man nur
/// ueber unsicheren Code.
pub fn iconify() -> Ergebnis<bool> {
    mit_zustand(|z| match z.backend.as_mut() {
        Some(b) => b.iconify(),
        None if z.init => Err(Fehler::KeinFenster),
        None => Err(Fehler::NichtInitialisiert),
    })
}

/// Laeuft die Anzeige im Vollbild? - `pygame.display.is_fullscreen()`.
///
/// Gemessen: `False` nach einem gewoehnlichen [`set_mode`], `True`
/// nach einem mit [`FULLSCREEN`] - und dann ist die Fenstergroesse die
/// der Arbeitsflaeche, also `(1024, 768)`. Ohne Fenster wirft pygame
/// `No open window`.
///
/// Der Wert kommt aus den Flaggen, mit denen `set_mode` gerufen wurde,
/// und aus [`toggle_fullscreen`] - **nicht** vom Fenstersystem. Das
/// genuegt, weil hier niemand sonst das Vollbild umschalten kann; ein
/// Fenster, das der Benutzer selbst per Tastenkuerzel aufzieht, wuerde
/// davon nichts merken.
pub fn is_fullscreen() -> Ergebnis<bool> {
    mit_zustand(|z| {
        if z.backend.is_none() {
            return Err(if z.init {
                Fehler::KeinFenster
            } else {
                Fehler::NichtInitialisiert
            });
        }
        Ok(z.flags & FULLSCREEN != 0)
    })
}

/// Wartet [`flip`] auf den Bildaufbau? - `pygame.display.is_vsync()`.
///
/// Gemessen: `False` nach einem gewoehnlichen [`set_mode`], `True`
/// nach `set_mode(.., vsync=1)`. Ohne Fenster wirft pygame
/// `No open window`.
///
/// Hier ist es **immer** `false`, und zwar nicht aus Bequemlichkeit:
/// [`set_mode`] dieser Kiste nimmt gar kein `vsync` entgegen, weil
/// minifb es nicht anbietet. minifbs `set_target_fps` ist eine eigene
/// Bremse, kein Warten auf den Bildaufbau - der Unterschied ist genau
/// der, den man sehen kann (Bildrisse bleiben). `true`
/// zurueckzugeben waere darum eine Zusage, die kein Pixel einloest.
pub fn is_vsync() -> Ergebnis<bool> {
    mit_zustand(|z| {
        if z.backend.is_none() {
            return Err(if z.init {
                Fehler::KeinFenster
            } else {
                Fehler::NichtInitialisiert
            });
        }
        Ok(false)
    })
}

/// Welche Art Meldung [`message_box`] zeigen soll.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Meldungsart {
    /// `"info"` - die Vorgabe.
    #[default]
    Info,
    /// `"warn"`.
    Warnung,
    /// `"error"`.
    Fehler,
}

/// Zeigt ein Meldungsfenster des Systems - `pygame.display.message_box()`.
///
/// Gibt zurueck, welcher Knopf gedrueckt wurde (der Index in `knoepfe`).
///
/// **Gibt es hier nicht**, und das ist gemessen: echtes pygame wirft
/// unter `SDL_VIDEODRIVER=dummy` woertlich `No message system
/// available`. Genau der Text steht auch hier, und aus demselben Grund:
/// ein Meldungsfenster baut nicht das Programm, sondern das
/// Fenstersystem. minifb hat keinen Aufruf dafuer - es kann ein Fenster
/// aufmachen und einen Pixelpuffer hineinschieben, mehr nicht.
///
/// Ein eigenes Fenster mit zwei gezeichneten Knoepfen waere kein
/// Ersatz: pygames `message_box` haelt den Strang an, bis geklickt
/// wurde, und funktioniert auch **vor** `pygame.init()` - beides liesse
/// sich so nicht nachbauen.
pub fn message_box(
    _titel: &str,
    _nachricht: Option<&str>,
    _art: Meldungsart,
    _knoepfe: &[&str],
) -> Ergebnis<usize> {
    Err(Fehler::Pygame("No message system available"))
}

/// Taugt diese Anzeigeart? - `pygame.display.mode_ok()`.
///
/// Gibt die Farbtiefe zurueck, die am besten passt, oder `0`, wenn die
/// Art nicht geht. Dieselben Argumente wie [`set_mode`]; die
/// Bildschirmnummer ist hier immer 0, weil es nur einen gibt (siehe
/// [`get_num_displays`]).
///
/// Alle Regeln an echtem pygame unter `SDL_VIDEODRIVER=dummy` gemessen:
///
/// | Aufruf | Antwort |
/// |---|---|
/// | `mode_ok((640, 480))` | `24` |
/// | `mode_ok((1024, 768))` | `24` - genau die Arbeitsflaeche |
/// | `mode_ok((1025, 768))` | `0` - eine Spalte zu breit |
/// | `mode_ok((0, 0))`, `mode_ok((-5, -5))` | `24` |
/// | `mode_ok((640, 480), FULLSCREEN)` | `0` |
/// | `mode_ok((1024, 768), FULLSCREEN)` | `24` |
/// | `mode_ok(.., 0, 8)` bis `(.., 0, 32)` | `24` |
/// | `mode_ok(.., 0, 4)` oder `(.., 0, 33)` | `0` |
///
/// Zwei Dinge daran sind der Rede wert. Erstens: **`24`, obwohl
/// [`Info`] `bitsize = 32` meldet.** Das ist kein Widerspruch - `24`
/// ist die Farbtiefe der Arbeitsflaeche (RGB888, drei Kanaele), `32`
/// die Breite eines Pixels im Speicher. Zweitens: im Vollbild geht
/// **nur** eine Groesse, die es als Anzeigeart wirklich gibt; im
/// Fenster dagegen jede, die auf die Arbeitsflaeche passt.
pub fn mode_ok(groesse: (i32, i32), flags: u32, tiefe: u32) -> Ergebnis<u32> {
    let desktop = get_desktop_sizes()?;
    let (dw, dh) = desktop.first().copied().unwrap_or((0, 0));

    // Gemessen: eine Tiefe unter 8 oder ueber 32 gibt es nicht; 0 heisst
    // "such dir was aus".
    if tiefe != 0 && !(8..=32).contains(&tiefe) {
        return Ok(0);
    }
    if flags & FULLSCREEN != 0 {
        // Im Vollbild muss die Groesse eine der aufgelisteten sein.
        let geht = desktop
            .iter()
            .any(|(w, h)| *w as i32 == groesse.0 && *h as i32 == groesse.1);
        return Ok(if geht { TIEFE_DER_ARBEITSFLAECHE } else { 0 });
    }
    if groesse.0 > dw as i32 || groesse.1 > dh as i32 {
        return Ok(0);
    }
    Ok(TIEFE_DER_ARBEITSFLAECHE)
}

/// Die Farbtiefe, die [`mode_ok`] meldet.
///
/// Gemessen: `24`. Das ist die Tiefe der Arbeitsflaeche (RGB888), nicht
/// die Breite eines Pixels im Speicher - die meldet [`Info::bitsize`]
/// mit `32`.
const TIEFE_DER_ARBEITSFLAECHE: u32 = 24;

// ---------------------------------------------------------------------------
//  Gamma und Farbtafel - beides gibt es hier nicht
// ---------------------------------------------------------------------------

/// Verstellt die Helligkeitskurve - `pygame.display.set_gamma()`.
///
/// Gibt zurueck, ob es geklappt hat. **Hier immer `false`**, und das
/// ist genau das, was echtes pygame mit offenem Fenster unter
/// `SDL_VIDEODRIVER=dummy` auch sagt - gemessen. Der Grund ist
/// derselbe: die Helligkeitskurve gehoert der Grafikkarte, und weder
/// SDLs Dummy-Treiber noch minifb reden mit ihr.
///
/// `false` und kein Fehler, weil pygame es genauso haelt: "if the
/// function succeeds it will return True". In pygame ist der Aufruf
/// seit 2.1.4 ueberdies als veraltet gekennzeichnet und verschwindet
/// mit SDL3.
///
/// Nebenbei bemerkt und hier **nicht** nachgebaut: ruft man
/// `pygame.display.set_gamma()` unter dem Dummy-Treiber **vor**
/// `set_mode`, gibt es `True` zurueck - und das darauf folgende
/// `set_mode` scheitert dann mit `That operation is not supported`. Das
/// ist ein Fehler in pygame/SDL, kein Verhalten, das man nachahmt.
pub fn set_gamma(_rot: f32, _gruen: f32, _blau: f32) -> bool {
    false
}

/// Verstellt die Helligkeitskurve punktweise -
/// `pygame.display.set_gamma_ramp()`.
///
/// Drei Tabellen zu je 256 Werten zwischen 0 und 0xffff. Gibt zurueck,
/// ob es geklappt hat - **hier immer `false`**, aus demselben Grund wie
/// bei [`set_gamma`], und gemessen mit demselben Ergebnis.
pub fn set_gamma_ramp(_rot: &[u16; 256], _gruen: &[u16; 256], _blau: &[u16; 256]) -> bool {
    false
}

/// Setzt die Farbtafel der Anzeige - `pygame.display.set_palette()`.
///
/// Eine leere Liste stellt die Tafel des Systems wieder her, genau wie
/// `set_palette()` ohne Argument in pygame.
///
/// **Das geht nur bei einer Anzeige mit Farbtafel**, also einer mit 8
/// Bit je Pixel. Diese Kiste rechnet durchgehend in ARGB8888 - siehe
/// [`Info::masks`] -, hat also keine. Gemessen wirft echtes pygame
/// dort woertlich `Display mode is not colormapped`; genau der Text
/// steht auch hier. Ohne [`set_mode`] wirft es vorher `No display mode
/// is set`.
pub fn set_palette(_farben: &[(u8, u8, u8)]) -> Ergebnis<()> {
    mit_zustand(|z| {
        if z.screen.is_none() {
            // Gemessen: pygame sagt hier "No display mode is set" - nicht
            // ganz derselbe Wortlaut wie bei `flip()`, aber dieselbe
            // Sache; siehe `Fehler::KeinModus`.
            return Err(Fehler::Pygame("No display mode is set"));
        }
        Err(Fehler::Pygame("Display mode is not colormapped"))
    })
}

/// Auskunft ueber die Anzeige - `pygame.display.Info()`.
pub fn info() -> Info {
    let (w, h) = mit_zustand(|z| {
        z.screen
            .as_ref()
            .map_or((0, 0), |s| s.try_read().expect(GELEIHT).get_size())
    });
    Info {
        hw: 0,
        wm: 1,
        video_mem: 0,
        bitsize: 32,
        bytesize: 4,
        masks: (0x00ff_0000, 0x0000_ff00, 0x0000_00ff, 0x0000_0000),
        shifts: (16, 8, 0, 0),
        losses: (0, 0, 0, 8),
        current_w: w,
        current_h: h,
        pixel_format: "PIXELFORMAT_RGB888",
    }
}
