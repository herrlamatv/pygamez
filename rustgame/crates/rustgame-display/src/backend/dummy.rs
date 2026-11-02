//! backend/dummy.rs
//! ================
//! Das Hintergrundstueck **ohne Fenster** - und der eigentliche Betriebsfall.
//!
//! Das umgebende Projekt (pygamez) faehrt pygame genau so: die Oberflaeche
//! ist Tkinter, pygame laeuft mit `SDL_VIDEODRIVER=dummy` und
//! `set_mode((1, 1))` daneben, das fertige Bild geht als PPM ins
//! Tkinter-Fenster, und die Tastendruecke kommen von dort zurueck. Damit
//! rustgame pygame dort ersetzen kann, muss dieses Stueck genau das
//! koennen:
//!
//! * **Bild abholen** - [`ppm`] liefert die Anzeigeflaeche als PPM (P6),
//!   das Format, das Tkinters `PhotoImage` unmittelbar frisst.
//! * **Ereignisse einspeisen** - [`taste_druecken`], [`maus_bewegen`],
//!   [`einspeisen`] und die uebrigen legen Ereignisse so ab, als kaemen
//!   sie von einem echten Fenster: sie schreiben also auch
//!   [`crate::key::get_pressed`] und [`crate::mouse::get_pos`] nach.
//! * **`flip` schlucken** - es gibt nichts anzuzeigen, also passiert
//!   nichts. Gezaehlt wird trotzdem, siehe [`bildzaehler`].
//!
//! Und weil es das kann, laeuft die gesamte Testsuite dieser Kiste ohne
//! Bildschirm. Eingeschaltet wird es mit `RUSTGAME_VIDEODRIVER=dummy`
//! oder mit [`crate::display::set_treiber`].

use rustgame_core::Surface;

use crate::backend::{Backend, Eingabe};
use crate::event::{Daten, Event};
use crate::konstanten::*;
use crate::mouse::Cursor;
use crate::zustand::mit_zustand;
use crate::{Ergebnis, Fehler};

/// Die Arbeitsflaeche, die SDLs Dummy-Treiber vorgibt.
///
/// Gemessen an echtem pygame unter `SDL_VIDEODRIVER=dummy`:
/// `list_modes()`, `get_desktop_sizes()` und die Vollbildgroesse sagen
/// alle `(1024, 768)`. Die Zahl ist also nicht ausgedacht, sondern die,
/// die das umgebende Projekt heute schon sieht.
pub const DESKTOP: (u32, u32) = (1024, 768);

/// Die Bildwiederholrate, die derselbe Treiber meldet.
///
/// Gemessen: `get_desktop_refresh_rates()` ist `[60]` und
/// `get_current_refresh_rate()` ist `60`.
pub const BILDRATE: u32 = 60;

/// Ein Fenster, das keines ist.
pub struct DummyBackend {
    groesse: (u32, u32),
    /// Was beim naechsten `pumpen` herauskommt - die eingespeisten
    /// Ereignisse. Sie warten hier statt direkt in der Schlange zu landen,
    /// damit sie denselben Weg nehmen wie echte: erst `pumpen`, dann
    /// Sperrliste, dann Schlange.
    puffer: Vec<Event>,
    /// Wie oft `flip`/`update` gerufen wurde.
    bilder: u64,
    /// Wo das Fenster steht.
    ///
    /// `None`, bis das erste `set_mode` gelaufen ist - dann wird es
    /// **einmal** auf der Arbeitsflaeche mittig gesetzt und bleibt dort.
    /// Gemessen: nach `set_mode((320, 240))` sagt
    /// `get_window_position()` `(352, 264)`, und das ist genau
    /// `((1024-320)/2, (768-240)/2)`. Ein zweites `set_mode` mit einer
    /// **anderen** Groesse aendert daran nichts mehr - das Fenster wird
    /// nur umgebaut, nicht neu gestellt. Auch das ist gemessen.
    pos: Option<(i32, i32)>,
    /// Haelt das "Fenster" Maus und Tastatur fest?
    griff: bool,
}

impl Default for DummyBackend {
    fn default() -> Self {
        Self::neu()
    }
}

impl DummyBackend {
    /// Ein frisches Stueck ohne Fenster.
    pub fn neu() -> DummyBackend {
        DummyBackend {
            groesse: (0, 0),
            puffer: Vec::new(),
            bilder: 0,
            pos: None,
            griff: false,
        }
    }

    /// Legt ein Ereignis in den Puffer.
    pub fn einspeisen(&mut self, e: Event) {
        self.puffer.push(e);
    }

    /// Wie oft `flip`/`update` gerufen wurde.
    pub fn bilder(&self) -> u64 {
        self.bilder
    }
}

impl Backend for DummyBackend {
    fn name(&self) -> &'static str {
        "dummy"
    }

    fn set_mode(&mut self, groesse: (u32, u32), _flags: u32) -> Ergebnis<()> {
        self.groesse = groesse;
        if self.pos.is_none() {
            // Nur beim allerersten Mal - siehe das Feld `pos` weiter oben.
            self.pos = Some((
                (DESKTOP.0 as i32 - groesse.0 as i32) / 2,
                (DESKTOP.1 as i32 - groesse.1 as i32) / 2,
            ));
        }
        Ok(())
    }

    fn set_caption(&mut self, _titel: &str) {}

    fn set_icon(&mut self, _bild: &Surface) -> Ergebnis<()> {
        Ok(())
    }

    /// Schluckt das Bild und zaehlt es.
    fn zeigen(&mut self, _daten: &[u32], _groesse: (u32, u32)) -> Ergebnis<()> {
        self.bilder += 1;
        Ok(())
    }

    fn pumpen(&mut self, eingabe: &mut Eingabe, heraus: &mut Vec<Event>) {
        for e in self.puffer.drain(..) {
            nachfuehren(eingabe, &e);
            heraus.push(e);
        }
    }

    fn window_size(&self) -> (u32, u32) {
        self.groesse
    }

    /// Gemessen: unter `SDL_VIDEODRIVER=dummy` meldet `list_modes()` genau
    /// eine Groesse, `(1024, 768)`.
    fn list_modes(&self) -> Vec<(u32, u32)> {
        vec![DESKTOP]
    }

    /// Gemessen: `get_desktop_sizes()` ist `[(1024, 768)]`.
    fn desktop_groessen(&self) -> Ergebnis<Vec<(u32, u32)>> {
        Ok(vec![DESKTOP])
    }

    /// Gemessen: `get_desktop_refresh_rates()` ist `[60]`.
    fn bildwiederholraten(&self) -> Ergebnis<Vec<u32>> {
        Ok(vec![BILDRATE])
    }

    /// Gemessen: `get_current_refresh_rate()` ist `60`.
    fn bildwiederholrate(&self) -> Ergebnis<u32> {
        Ok(BILDRATE)
    }

    /// Gemessen: mittig auf der Arbeitsflaeche, einmal beim Oeffnen
    /// festgelegt - siehe das Feld `pos` weiter oben.
    fn fenster_pos(&self) -> Ergebnis<(i32, i32)> {
        self.pos.ok_or(crate::Fehler::KeinFenster)
    }

    /// Gemessen: `set_window_position((10, 20))` wirkt sofort, und
    /// `get_window_position()` sagt danach `(10, 20)`. Mit
    /// [`crate::konstanten::WINDOWPOS_CENTERED`] geht es zurueck in die
    /// Mitte.
    fn set_fenster_pos(&mut self, pos: (i32, i32)) -> Ergebnis<()> {
        let mittig = (
            (DESKTOP.0 as i32 - self.groesse.0 as i32) / 2,
            (DESKTOP.1 as i32 - self.groesse.1 as i32) / 2,
        );
        let zentriert = crate::konstanten::WINDOWPOS_CENTERED as i32;
        self.pos = Some((
            if pos.0 == zentriert { mittig.0 } else { pos.0 },
            if pos.1 == zentriert { mittig.1 } else { pos.1 },
        ));
        Ok(())
    }

    /// Gemessen und ueberraschend: `iconify()` gibt unter dem
    /// Dummy-Treiber **`True`** zurueck - und tut dabei nichts. Kein
    /// `ACTIVEEVENT`, kein `WINDOWMINIMIZED`, und `get_active()` steht
    /// danach immer noch auf `True`. Wer sich auf pygames eigene
    /// Beschreibung verlaesst ("The event queue should receive an
    /// ACTIVEEVENT"), wartet hier vergebens.
    fn iconify(&mut self) -> Ergebnis<bool> {
        Ok(true)
    }

    /// Gemessen: ohne Fenster nimmt pygame eine Bitmap- oder Farbform
    /// klaglos an (`set_cursor(cursors.diamond)` gibt `None` zurueck und
    /// `get_cursor().type` ist danach `'bitmap'`), wirft aber bei einer
    /// Systemform `Error while creating system cursor`.
    ///
    /// Das sieht verkehrt herum aus - die Systemform ist doch die
    /// einfachere - hat aber einen Grund: SDL muss fuer eine Systemform
    /// beim Fenstersystem eine anfordern, und das gibt es unter dem
    /// Dummy-Treiber nicht. Eine eigene Form ist dagegen nur ein
    /// Stueck Speicher.
    fn set_cursor(&mut self, zeiger: &Cursor) -> Ergebnis<()> {
        match zeiger {
            Cursor::System(_) => Err(crate::Fehler::Pygame(
                "Error while creating system cursor",
            )),
            _ => Ok(()),
        }
    }

    /// Gemessen: `set_grab(True)` **mit** Fenster wirkt auch unter dem
    /// Dummy-Treiber - `get_grab()` sagt danach `True`. Ohne Fenster
    /// bleibt es bei `False`; um das kuemmert sich
    /// [`crate::event::set_grab`], denn ohne Fenster gibt es dieses
    /// Stueck gar nicht.
    fn set_grab(&mut self, an: bool) -> bool {
        self.griff = an;
        an
    }

    /// Gemessen: `get_wm_info()` ist unter dem Dummy-Treiber ein leeres
    /// Woerterbuch - es gibt kein Fenstersystem, ueber das es etwas zu
    /// sagen gaebe.
    fn wm_info(&self) -> Vec<(&'static str, u64)> {
        Vec::new()
    }

    fn set_mouse_pos(&mut self, _pos: (i32, i32)) {}

    fn set_mouse_visible(&mut self, _sichtbar: bool) {}

    /// Gemessen: echtes pygame wirft unter dem Dummy-Treiber woertlich
    /// `That operation is not supported`. Kein stiller Fehlschlag, kein 0 -
    /// ein Fehler.
    fn toggle_fullscreen(&mut self) -> Ergebnis<()> {
        Err(crate::Fehler::NichtUnterstuetzt(
            "toggle_fullscreen without a window",
        ))
    }

    fn set_repeat(&mut self, _verzoegerung: i32, _abstand: i32) {}

    fn ist_offen(&self) -> bool {
        true
    }

    /// Nein - und genau deshalb gibt es dieses Stueck.
    ///
    /// Siehe [`Backend::hat_fenster`]: davon haengt ab, dass
    /// `borderless`, `resizable` und `always_on_top` sich hier nach dem
    /// Anlegen nicht mehr aendern lassen - gemessen macht es echtes
    /// pygame unter `SDL_VIDEODRIVER=dummy` genauso.
    fn hat_fenster(&self) -> bool {
        false
    }

    /// Zeigt oder versteckt das "Fenster".
    ///
    /// Ohne Fenster ist das reine Buchfuehrung - aber sie stimmt:
    /// gemessen legt echtes pygame unter dem Dummy-Treiber bei
    /// `Window.hide()` fuenf Ereignisse ab und meldet `focused` danach
    /// als `False`. Die Ereignisse legt [`crate::fenster`] ab, hier wird
    /// der Wunsch nur angenommen.
    fn fenster_zeigen(&mut self, _an: bool) -> Ergebnis<()> {
        Ok(())
    }

    /// Holt das "Fenster" nach vorn.
    ///
    /// Gemessen, und der Grund fuer die Unterscheidung: unter
    /// `SDL_VIDEODRIVER=dummy` geht `w.focus()` klaglos durch, aber
    /// `w.focus(input_only=True)` wirft woertlich `That operation is not
    /// supported`. SDL verlangt fuer den blossen Eingabefokus eine
    /// Fensterverwaltung, die der Dummy-Treiber nicht hat.
    fn nach_vorn(&mut self, nur_eingabe: bool) -> Ergebnis<()> {
        if nur_eingabe {
            return Err(Fehler::NichtUnterstuetzt(
                "input focus alone requires a window manager",
            ));
        }
        Ok(())
    }

    /// Legt ab, zieht auf, stellt zurueck - und tut dabei nichts.
    ///
    /// Gemessen: echtes pygame gibt unter dem Dummy-Treiber bei
    /// `minimize()`, `maximize()` und `restore()` `None` zurueck, aendert
    /// die Groesse nicht und legt **kein** Ereignis ab. Dasselbe wie bei
    /// [`Backend::iconify`], das hier aus demselben Grund `true` meldet.
    fn fensterzustand(&mut self, _art: super::Fensterzustand) -> Ergebnis<()> {
        Ok(())
    }

    /// Schaltet Vollbild - hier ein Groessenwechsel und sonst nichts.
    ///
    /// Gemessen, und die Ueberraschung: `Window.set_fullscreen()` geht
    /// unter `SDL_VIDEODRIVER=dummy` durch und macht die Flaeche
    /// `(1024, 768)` gross, waehrend `display.toggle_fullscreen()` im
    /// selben Lauf `That operation is not supported` wirft. Zwei Wege zum
    /// selben Ziel, zwei verschiedene Antworten - beides gemessen, und
    /// beides steht hier so.
    fn set_vollbild(&mut self, _an: bool, groesse: (u32, u32)) -> Ergebnis<()> {
        self.groesse = groesse;
        Ok(())
    }

    fn dummy_mut(&mut self) -> Option<&mut DummyBackend> {
        Some(self)
    }
}

/// Schreibt Tastatur- und Mauszustand nach, wie es ein echtes Fenster taete.
fn nachfuehren(eingabe: &mut Eingabe, e: &Event) {
    match (&e.typ, &e.daten) {
        (&KEYDOWN, Daten::Taste { key, .. }) => eingabe.taste_setzen(*key, true),
        (&KEYUP, Daten::Taste { key, .. }) => eingabe.taste_setzen(*key, false),
        (&MOUSEMOTION, Daten::MausBewegung { pos, rel, .. }) => {
            eingabe.maus_pos = *pos;
            eingabe.maus_rel.0 += rel.0;
            eingabe.maus_rel.1 += rel.1;
        }
        (&MOUSEBUTTONDOWN, Daten::MausTaste { pos, button, .. }) => {
            eingabe.maus_pos = *pos;
            eingabe.maustaste_setzen(*button, true);
        }
        (&MOUSEBUTTONUP, Daten::MausTaste { pos, button, .. }) => {
            eingabe.maus_pos = *pos;
            eingabe.maustaste_setzen(*button, false);
        }
        (&WINDOWFOCUSGAINED, _) => eingabe.tast_fokus = true,
        (&WINDOWFOCUSLOST, _) => eingabe.tast_fokus = false,
        (&WINDOWENTER, _) => eingabe.maus_fokus = true,
        (&WINDOWLEAVE, _) => eingabe.maus_fokus = false,
        _ => {}
    }
}

// ---------------------------------------------------------------------------
//  Einspeisen von aussen
// ---------------------------------------------------------------------------

/// Ruft `f` mit dem offenen Dummy-Stueck - oder gibt `false`, wenn gar
/// keines offen ist.
fn mit_dummy(f: impl FnOnce(&mut DummyBackend)) -> bool {
    mit_zustand(|z| match z.backend.as_mut().and_then(|b| b.dummy_mut()) {
        Some(d) => {
            f(d);
            true
        }
        None => false,
    })
}

/// Speist ein fertiges Ereignis ein, als kaeme es vom Fenster.
///
/// Anders als [`crate::event::post`] laeuft es denselben Weg wie ein
/// echtes: beim naechsten Pumpen schreibt es den Tastatur- und
/// Mauszustand nach, bevor es in der Schlange landet.
///
/// Gibt `false` zurueck, wenn gerade kein fensterloses Stueck offen ist.
pub fn einspeisen(e: Event) -> bool {
    mit_dummy(|d| d.einspeisen(e))
}

/// Drueckt eine Taste: `KEYDOWN`, und [`crate::key::get_pressed`] sieht sie.
///
/// `unicode` ist das erzeugte Zeichen - fuer Steuertasten die leere
/// Zeichenkette, genau wie bei pygame.
pub fn taste_druecken(key: Key, unicode: &str) -> bool {
    let mods = mit_zustand(|z| z.eingabe.mods);
    einspeisen(Event::key_down(key, mods, unicode))
}

/// Laesst eine Taste los: `KEYUP`.
pub fn taste_loslassen(key: Key) -> bool {
    let mods = mit_zustand(|z| z.eingabe.mods);
    einspeisen(Event::key_up(key, mods))
}

/// Tippt eine Taste: erst herunter, dann herauf.
pub fn taste_tippen(key: Key, unicode: &str) -> bool {
    taste_druecken(key, unicode) && taste_loslassen(key)
}

/// Wo der Zeiger steht, **einschliesslich** dessen, was noch im Puffer
/// wartet.
///
/// Noetig, weil `maus_klicken` drei Ereignisse hintereinander einspeist:
/// die Bewegung ist noch nicht gepumpt, wenn der Klick entsteht. Wuerde
/// der Klick den gepumpten Stand nehmen, traege er die alte Stelle - und
/// beim Pumpen wuerde er die neue gleich wieder ueberschreiben.
fn zeigerstelle() -> (i32, i32) {
    mit_zustand(|z| {
        if let Some(d) = z.backend.as_mut().and_then(|b| b.dummy_mut()) {
            if let Some(p) = d.puffer.iter().rev().find_map(|e| e.pos()) {
                return p;
            }
        }
        z.eingabe.maus_pos
    })
}

/// Bewegt den Zeiger nach `pos`: `MOUSEMOTION` mit passendem `rel`.
pub fn maus_bewegen(pos: (i32, i32)) -> bool {
    let alt = zeigerstelle();
    let tasten = mit_zustand(|z| z.eingabe.maus_tasten);
    einspeisen(Event::mouse_motion(
        pos,
        (pos.0 - alt.0, pos.1 - alt.1),
        (tasten[0], tasten[1], tasten[2]),
    ))
}

/// Drueckt eine Maustaste an der jetzigen Zeigerstelle.
///
/// `button` zaehlt wie bei pygame: 1 links, 2 mitte, 3 rechts.
pub fn maus_druecken(button: u8) -> bool {
    let pos = zeigerstelle();
    einspeisen(Event::mouse_button_down(pos, button))
}

/// Laesst eine Maustaste los.
pub fn maus_loslassen(button: u8) -> bool {
    let pos = zeigerstelle();
    einspeisen(Event::mouse_button_up(pos, button))
}

/// Klickt: herunter und herauf an `pos`.
pub fn maus_klicken(pos: (i32, i32), button: u8) -> bool {
    maus_bewegen(pos) && maus_druecken(button) && maus_loslassen(button)
}

/// Dreht das Mausrad - `MOUSEWHEEL`.
pub fn maus_rad(x: i32, y: i32) -> bool {
    einspeisen(Event::mit(
        MOUSEWHEEL,
        Daten::MausRad {
            x,
            y,
            precise_x: x as f32,
            precise_y: y as f32,
            flipped: false,
        },
    ))
}

/// Schliesst das Fenster - `QUIT`.
pub fn fenster_schliessen() -> bool {
    einspeisen(Event::quit())
}

/// Gibt oder nimmt den Tastaturfokus.
///
/// Legt dieselben Ereignisse ab wie ein echtes Fenster:
/// `ACTIVEEVENT` und `WINDOWFOCUSGAINED`/`WINDOWFOCUSLOST`.
pub fn fokus_setzen(fokus: bool) -> bool {
    let gain = if fokus { 1 } else { 0 };
    einspeisen(Event::mit(
        ACTIVEEVENT,
        Daten::Aktiv {
            gain,
            state: APPINPUTFOCUS as i32,
        },
    )) && einspeisen(Event::neu(if fokus {
        WINDOWFOCUSGAINED
    } else {
        WINDOWFOCUSLOST
    }))
}

/// Wie oft `flip`/`update` gerufen wurde, seit das Stueck offen ist.
///
/// Damit laesst sich pruefen, dass ein Bild wirklich abgeschickt wurde,
/// obwohl es nirgends erscheint.
pub fn bildzaehler() -> u64 {
    mit_zustand(|z| {
        z.backend
            .as_mut()
            .and_then(|b| b.dummy_mut())
            .map_or(0, |d| d.bilder())
    })
}

/// Die Anzeigeflaeche als PPM (P6, 8 Bit je Kanal).
///
/// Das ist der Weg, auf dem das umgebende Projekt sein Bild nach Tkinter
/// bringt: `PhotoImage(data=...)` liest PPM unmittelbar. Gibt `None`,
/// wenn noch kein `set_mode` gelaufen ist.
///
/// Der Alphakanal faellt weg - PPM kennt keinen, und die Anzeigeflaeche
/// hat gemessen ohnehin keinen (`get_alpha()` ist `None`, jeder Pixel
/// kommt mit A=255 heraus).
pub fn ppm() -> Option<Vec<u8>> {
    let screen = mit_zustand(|z| z.screen.clone())?;
    let s = screen.read();
    let (w, h) = s.get_size();
    let mut heraus = format!("P6\n{w} {h}\n255\n").into_bytes();
    heraus.extend_from_slice(&s.to_rgb_bytes());
    Some(heraus)
}
