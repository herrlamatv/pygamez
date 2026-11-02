//! backend
//! =======
//! Was ein Fenster koennen muss, damit `display`, `event`, `key` und
//! `mouse` darauf aufsetzen koennen - und die zwei Stuecke, die das tun.
//!
//! | Stueck | Fenster? | wofuer |
//! |---|---|---|
//! | [`MinifbBackend`] | ja | der Normalfall auf dem Schreibtisch |
//! | [`DummyBackend`] | nein | eingebettet, kopflos, getestet |
//!
//! ## Warum das fensterlose Stueck zuerst kam
//!
//! Das umgebende Projekt faehrt pygame mit `SDL_VIDEODRIVER=dummy` und
//! `set_mode((1, 1))` und schiebt das fertige Bild als PPM in ein
//! Tkinter-Fenster. rustgame muss das genauso koennen, sonst ist es kein
//! Ersatz. Und weil es das kann, laeuft die gesamte Testsuite ohne
//! Bildschirm - was ohne dieses Stueck schlicht nicht ginge.
//!
//! Siehe [`dummy`] fuer die Werkzeuge, die dazugehoeren: Ereignisse
//! einspeisen, das Bild als PPM abholen, die Bildzaehler lesen.

use rustgame_core::Surface;

use crate::event::Event;
use crate::mouse::Cursor;
use crate::{Ergebnis, Fehler};

pub mod dummy;
#[cfg(not(target_arch = "wasm32"))]
mod minifb_backend;

pub use crate::zustand::{Eingabe, Treiber};
pub use dummy::DummyBackend;
#[cfg(not(target_arch = "wasm32"))]
pub use minifb_backend::MinifbBackend;

/// Was ein Fenster koennen muss.
///
/// Der Trait ist bewusst schmal: alles, was sich ohne Fenster rechnen
/// laesst - Sperrlisten, Tastennamen, die Schlange selbst - steht in den
/// Modulen darueber und kommt hier gar nicht vor.
pub trait Backend {
    /// Der Name des Treibers, wie `pygame.display.get_driver()` ihn nennt.
    fn name(&self) -> &'static str;

    /// Baut das Fenster auf die neue Groesse um.
    fn set_mode(&mut self, groesse: (u32, u32), flags: u32) -> Ergebnis<()>;

    /// Setzt den Fenstertitel.
    fn set_caption(&mut self, titel: &str);

    /// Setzt das Fenstersinnbild.
    ///
    /// Gibt [`crate::Fehler::NichtUnterstuetzt`] zurueck, wenn das Stueck
    /// das nicht kann - siehe [`MinifbBackend::set_icon`].
    fn set_icon(&mut self, bild: &Surface) -> Ergebnis<()>;

    /// Zeigt den Pixelpuffer an.
    ///
    /// `daten` kommt aus `Surface::mit_pixeln` und ist **ARGB8888**, ein
    /// `u32` je Pixel, Zeile fuer Zeile ohne Luecke.
    fn zeigen(&mut self, daten: &[u32], groesse: (u32, u32)) -> Ergebnis<()>;

    /// Fragt das Fenster ab und meldet, was sich geaendert hat.
    ///
    /// Das ist die Stelle, an der aus minifbs Zustaenden Ereignisse
    /// werden: `eingabe` ist der Stand von eben und wird fortgeschrieben,
    /// `heraus` nimmt die Ereignisse auf, die dabei entstehen.
    fn pumpen(&mut self, eingabe: &mut Eingabe, heraus: &mut Vec<Event>);

    /// Die Groesse des Fensters (nicht der Zeichenflaeche).
    fn window_size(&self) -> (u32, u32);

    /// Welche Vollbild-Groessen es gibt - `pygame.display.list_modes()`.
    fn list_modes(&self) -> Vec<(u32, u32)>;

    /// Setzt den Zeiger.
    fn set_mouse_pos(&mut self, pos: (i32, i32));

    /// Zeigt oder versteckt den Zeiger.
    fn set_mouse_visible(&mut self, sichtbar: bool);

    /// Schaltet zwischen Fenster und Vollbild.
    fn toggle_fullscreen(&mut self) -> Ergebnis<()>;

    /// Setzt die Tastenwiederholung (Verzoegerung, Abstand) in Millisekunden.
    fn set_repeat(&mut self, verzoegerung: i32, abstand: i32);

    /// Steht das Fenster noch offen?
    fn ist_offen(&self) -> bool;

    /// Ist das hier das fensterlose Stueck?
    ///
    /// Nur [`DummyBackend`] sagt hier ja. So kommen die Einspeis-Werkzeuge
    /// aus [`dummy`] an ihren Puffer, ohne dass jemand `dyn Any`
    /// herunterreichen muss.
    fn dummy_mut(&mut self) -> Option<&mut DummyBackend> {
        None
    }

    // -- Ab hier: was nicht jedes Stueck koennen muss --------------------
    //
    // Diese Aufrufe haben eine Vorgabe, die ehrlich nein sagt. Ein Stueck,
    // das es kann, schreibt sie um; eines, das es nicht kann, laesst sie
    // stehen und meldet [`Fehler::NichtUnterstuetzt`] mit dem Grund. Das
    // ist die Regel dieser Kiste: lieber ein Fehler mit Begruendung als
    // eine erfundene Zahl.

    /// Wo das Fenster auf dem Schirm steht - `display.get_window_position()`.
    fn fenster_pos(&self) -> Ergebnis<(i32, i32)> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend does not know its window position",
        ))
    }

    /// Versetzt das Fenster - `display.set_window_position()`.
    fn set_fenster_pos(&mut self, _pos: (i32, i32)) -> Ergebnis<()> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend cannot move the window",
        ))
    }

    /// Die Groessen der Arbeitsflaechen - `display.get_desktop_sizes()`.
    fn desktop_groessen(&self) -> Ergebnis<Vec<(u32, u32)>> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend cannot query the displays",
        ))
    }

    /// Die Bildwiederholraten der Arbeitsflaechen -
    /// `display.get_desktop_refresh_rates()`.
    fn bildwiederholraten(&self) -> Ergebnis<Vec<u32>> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend cannot query the refresh rate",
        ))
    }

    /// Die Bildwiederholrate des Bildschirms, auf dem das Fenster steht -
    /// `display.get_current_refresh_rate()`.
    fn bildwiederholrate(&self) -> Ergebnis<u32> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend cannot query the refresh rate",
        ))
    }

    /// Legt das Fenster als Sinnbild ab - `display.iconify()`.
    fn iconify(&mut self) -> Ergebnis<bool> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend cannot resize the window",
        ))
    }

    /// Setzt die Zeigerform - `mouse.set_cursor()`.
    fn set_cursor(&mut self, _zeiger: &Cursor) -> Ergebnis<()> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend cannot change the cursor",
        ))
    }

    /// Haelt Maus und Tastatur am Fenster fest - `event.set_grab()`.
    ///
    /// Gibt zurueck, ob es **wirklich** gegriffen hat. Ein Stueck, das
    /// es nicht kann, gibt `false` und luegt damit niemanden an -
    /// [`crate::event::get_grab`] meldet dann weiterhin `false`, genau
    /// wie echtes pygame es tut, wenn kein Fenster offen ist.
    fn set_grab(&mut self, _an: bool) -> bool {
        false
    }

    /// Was `display.get_wm_info()` erzaehlt.
    ///
    /// Gemessen: unter `SDL_VIDEODRIVER=dummy` ist das ein **leeres**
    /// Woerterbuch. Auf den meisten anderen Treibern steht dort ein
    /// Schluessel `"window"` mit der Fensternummer des Systems.
    fn wm_info(&self) -> Vec<(&'static str, u64)> {
        Vec::new()
    }

    // -- Ab hier: was `pygame.window.Window` zusaetzlich verlangt --------
    //
    // Dieselbe Regel wie oben, nur fuer die Aufrufe, die es erst mit
    // [`crate::fenster::Fenster`] gibt. Wer hier nichts umschreibt,
    // sagt ehrlich nein.

    /// Steht hinter diesem Stueck wirklich ein Fenster auf dem Schirm?
    ///
    /// Nur [`DummyBackend`] sagt hier nein. Das entscheidet ueber die
    /// drei Merkmale, die ein **Rahmen** braucht, um etwas zu bedeuten -
    /// `borderless`, `resizable`, `always_on_top`. Gemessen an echtem
    /// pygame unter `SDL_VIDEODRIVER=dummy`: dort bleiben genau diese
    /// drei nach dem Anlegen **unveraenderlich**. `w.borderless = True`
    /// wirft keinen Fehler und bewirkt nichts; `w.borderless` sagt
    /// hinterher weiter, was im Konstruktor stand. Das ist kein Zufall,
    /// sondern der fehlende Fensterverwalter - und das fensterlose
    /// Stueck hier ist in derselben Lage.
    fn hat_fenster(&self) -> bool {
        true
    }

    /// Legt das Fenster ueber alle anderen - `Window.always_on_top`.
    fn set_obenauf(&mut self, _an: bool) -> Ergebnis<()> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend cannot keep the window on top",
        ))
    }

    /// Stellt die Deckkraft zwischen 0.0 und 1.0 - `Window.opacity`.
    fn set_deckkraft(&mut self, _wert: f32) -> Ergebnis<()> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend cannot set window opacity",
        ))
    }

    /// Zeigt oder versteckt das Fenster - `Window.show()` / `Window.hide()`.
    fn fenster_zeigen(&mut self, _an: bool) -> Ergebnis<()> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend cannot show or hide a window",
        ))
    }

    /// Holt das Fenster nach vorn - `Window.focus()`.
    ///
    /// `nur_eingabe` ist pygames `input_only=True`: nur den Eingabefokus
    /// nehmen, ohne das Fenster zu heben.
    fn nach_vorn(&mut self, _nur_eingabe: bool) -> Ergebnis<()> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend cannot raise a window",
        ))
    }

    /// Laesst das Fenster blinken - `Window.flash()`.
    ///
    /// `art` ist eine der drei `FLASH_*`-Konstanten; geprueft wird sie
    /// schon in [`crate::fenster`], hier kommt nur noch eine gueltige an.
    fn blinken(&mut self, _art: u32) -> Ergebnis<()> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend cannot flash a window",
        ))
    }

    /// Haengt das Fenster als Kind an ein anderes - `Window.set_modal_for()`.
    fn modal_fuer(&mut self) -> Ergebnis<()> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend has no parent/child relation between windows",
        ))
    }

    /// Legt ab, zieht auf oder stellt zurueck - `minimize`, `maximize`,
    /// `restore`.
    fn fensterzustand(&mut self, _art: Fensterzustand) -> Ergebnis<()> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend has no window manager that could minimize a window",
        ))
    }

    /// Schaltet Vollbild - `Window.set_fullscreen()` / `set_windowed()`.
    ///
    /// `groesse` ist die Groesse, die danach gelten soll: im Vollbild die
    /// Arbeitsflaeche, sonst die gemerkte Fenstergroesse.
    fn set_vollbild(&mut self, _an: bool, _groesse: (u32, u32)) -> Ergebnis<()> {
        Err(Fehler::NichtUnterstuetzt(
            "this backend has no fullscreen mode",
        ))
    }
}

/// Was [`Backend::fensterzustand`] tun soll.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fensterzustand {
    /// `Window.minimize()`.
    Abgelegt,
    /// `Window.maximize()`.
    Aufgezogen,
    /// `Window.restore()`.
    Gewoehnlich,
}

/// Oeffnet das Stueck, das zum gewaehlten Treiber passt.
///
/// `Treiber::Automatisch` liest `RUSTGAME_VIDEODRIVER`; steht dort
/// `dummy`, kommt das fensterlose Stueck.
///
/// **Achtung, `cfg!(test)` allein reicht nicht.** Es ist nur innerhalb
/// dieser Kiste wahr. Ein **Integrationstest** (`tests/*.rs`) bindet die
/// Bibliothek als gewoehnliche Abhaengigkeit ein, und dort ist es
/// `false` - ein Test ohne weiteres Zutun wuerde also ein echtes Fenster
/// aufreissen. Genau darueber ist der Kollege bei `rustgame-mixer`
/// gestolpert, dessen Tonausgabe nach demselben Muster gebaut ist: sein
/// erster Testlauf hat das echte Geraet geoeffnet.
///
/// Deshalb faellt die Entscheidung hier **nicht** an `cfg!(test)` allein,
/// sondern zusaetzlich an `RUSTGAME_VIDEODRIVER`, das die Testdatei
/// setzen kann - und die Testdatei ruft ohnehin
/// `display::set_treiber(Treiber::Dummy)`. Wer einen neuen Integrations-
/// test schreibt, muss das eine oder das andere tun.
pub(crate) fn oeffnen(
    treiber: Treiber,
    groesse: (u32, u32),
    flags: u32,
    titel: &str,
) -> Ergebnis<Box<dyn Backend>> {
    match gewaehlter_treiber(treiber) {
        Treiber::Dummy => {
            let mut b = DummyBackend::neu();
            b.set_mode(groesse, flags)?;
            Ok(Box::new(b))
        }
        _ => oeffnen_fenster(groesse, flags, titel),
    }
}

/// Welches Stueck [`oeffnen`] nehmen wuerde - ohne es zu oeffnen.
///
/// Gibt nie [`Treiber::Automatisch`] zurueck: die Wahl faellt hier.
/// Gebraucht wird das fuer die Fragen, die echtes pygame **vor** dem
/// ersten `set_mode` beantwortet - `get_desktop_sizes()` zum Beispiel
/// liefert nach blossem `display.init()` schon `[(1024, 768)]`. Ohne
/// offenes Stueck gibt es niemanden, den man fragen koennte; also
/// entscheidet die Antwort daran, wer gleich geoeffnet wuerde.
pub(crate) fn gewaehlter_treiber(treiber: Treiber) -> Treiber {
    match treiber {
        Treiber::Dummy => Treiber::Dummy,
        Treiber::Minifb => Treiber::Minifb,
        Treiber::Automatisch => {
            if cfg!(test) {
                Treiber::Dummy
            } else {
                match std::env::var("RUSTGAME_VIDEODRIVER") {
                    Ok(v) if v.eq_ignore_ascii_case("dummy") => Treiber::Dummy,
                    _ => Treiber::Minifb,
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn oeffnen_fenster(groesse: (u32, u32), flags: u32, titel: &str) -> Ergebnis<Box<dyn Backend>> {
    Ok(Box::new(MinifbBackend::neu(groesse, flags, titel)?))
}

/// Ohne Betriebssystem gibt es kein Fenster - dann bleibt nur das
/// fensterlose Stueck.
#[cfg(target_arch = "wasm32")]
fn oeffnen_fenster(groesse: (u32, u32), flags: u32, _titel: &str) -> Ergebnis<Box<dyn Backend>> {
    let mut b = DummyBackend::neu();
    b.set_mode(groesse, flags)?;
    Ok(Box::new(b))
}
