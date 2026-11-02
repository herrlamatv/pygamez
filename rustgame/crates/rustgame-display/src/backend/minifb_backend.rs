//! backend/minifb_backend.rs
//! =========================
//! Das Hintergrundstueck mit echtem Fenster.
//!
//! ## minifb kennt keine Ereignisse
//!
//! minifb meldet **Zustaende**: welche Tasten unten sind, wo der Zeiger
//! steht, ob das Fenster noch offen ist. pygame meldet **Ereignisse**:
//! diese Taste ging gerade herunter, der Zeiger ist um so viel gewandert.
//! Die Umrechnung ist eine Kantenerkennung und steht in [`MinifbBackend::pumpen`]:
//! einmal je Bild wird der Stand von eben mit dem von jetzt verglichen,
//! und jeder Unterschied wird ein Ereignis.
//!
//! Fuer Tasten nimmt es minifb sogar ab - `get_keys_pressed` und
//! `get_keys_released` liefern genau die Kanten. Fuer Maus, Groesse und
//! Fokus steht der Vergleich hier.
//!
//! ## Der Pixelpuffer geht ohne Umkopieren durch
//!
//! minifb will `&[u32]` in `0RGB`, `rustgame-core` fuehrt `ARGB8888`. Die
//! unteren 24 Bit sind in beiden dasselbe, und das oberste Byte wirft
//! minifb weg (unter Windows landet der Puffer in `StretchDIBits` mit
//! `BI_RGB`, das den Alphakanal nicht anfasst). Der Puffer aus
//! `Surface::mit_pixeln` geht also unveraendert weiter - kein Kopieren,
//! keine zweite Zeile Speicher je Bild.
//!
//! ## Was minifb nicht kann
//!
//! Vollbild und Fenstersinnbild aus einer Flaeche. Beides meldet sich als
//! [`crate::Fehler::NichtUnterstuetzt`] statt still danebenzugehen -
//! genau wie echtes pygame es unter dem Dummy-Treiber tut.

use minifb::{Key, KeyRepeat, MouseButton, MouseMode, Window, WindowOptions};
use rustgame_core::Surface;
use std::cell::RefCell;
use std::rc::Rc;

use crate::backend::{Backend, Eingabe};
use crate::event::{Daten, Event};
use crate::konstanten::{self, *};
use crate::mouse::Cursor;
use crate::{Ergebnis, Fehler};

/// Sammelt die Zeichen, die minifb im Rueckruf meldet.
///
/// Der einzige Rueckruf in dieser Kiste - minifb bietet Text nur so an.
/// Er schreibt in eine geteilte Liste, die [`MinifbBackend::pumpen`]
/// gleich wieder leert; nach aussen bleibt alles abfragend.
#[derive(Default)]
struct Zeichensammler {
    zeichen: Rc<RefCell<Vec<char>>>,
}

impl minifb::InputCallback for Zeichensammler {
    fn add_char(&mut self, uni_char: u32) {
        if let Some(c) = char::from_u32(uni_char) {
            self.zeichen.borrow_mut().push(c);
        }
    }
}

/// Ein echtes Fenster.
pub struct MinifbBackend {
    fenster: Window,
    groesse: (u32, u32),
    /// Titel und Flaggen, um das Fenster bei einem Groessenwechsel
    /// genauso wieder aufbauen zu koennen.
    titel: String,
    flags: u32,
    /// Zeichen aus dem minifb-Rueckruf, noch nicht abgeholt.
    zeichen: Rc<RefCell<Vec<char>>>,
    /// Der Stand von eben, fuer die Kantenerkennung.
    letzte_maus: Option<(i32, i32)>,
    letzte_tasten: [bool; 3],
    letzte_groesse: (u32, u32),
    letzter_fokus: bool,
    /// Hat das Fenster schon einmal ein Bild gesehen? Vorher darf
    /// `is_active()` nicht als Fokusverlust gelten.
    lief_schon: bool,
    /// Liegt das Fenster ueber allen anderen? - [`Backend::set_obenauf`].
    ///
    /// Steht hier und nicht in `flags`, weil es keine pygame-Flagge dazu
    /// gibt: `always_on_top` ist bei pygame ein Wort im Konstruktor und
    /// eine Eigenschaft, aber keine `set_mode`-Flagge. Gebraucht wird es
    /// trotzdem hier, damit es einen Umbau ueberlebt - siehe
    /// [`MinifbBackend::set_mode`], das bei jedem Groessenwechsel ein
    /// neues Fenster aufmacht.
    obenauf: bool,
}

impl MinifbBackend {
    /// Oeffnet ein Fenster.
    pub fn neu(groesse: (u32, u32), flags: u32, titel: &str) -> Ergebnis<MinifbBackend> {
        let (w, h) = (groesse.0.max(1) as usize, groesse.1.max(1) as usize);
        let opts = WindowOptions {
            resize: flags & konstanten::RESIZABLE != 0,
            borderless: flags & konstanten::NOFRAME != 0,
            scale_mode: minifb::ScaleMode::UpperLeft,
            ..WindowOptions::default()
        };
        let mut fenster =
            Window::new(titel, w, h, opts).map_err(|e| Fehler::Fenster(e.to_string()))?;

        // Keine Bremse: pygame taktet mit `time.Clock`, nicht im Fenster.
        // Ohne das wartet `update_with_buffer` von sich aus und das
        // Programm laeuft doppelt gebremst.
        fenster.set_target_fps(0);

        let sammler = Zeichensammler::default();
        let zeichen = Rc::clone(&sammler.zeichen);
        fenster.set_input_callback(Box::new(sammler));

        Ok(MinifbBackend {
            fenster,
            groesse: (w as u32, h as u32),
            titel: titel.to_string(),
            flags,
            zeichen,
            letzte_maus: None,
            letzte_tasten: [false; 3],
            letzte_groesse: (w as u32, h as u32),
            letzter_fokus: true,
            lief_schon: false,
            // Frisch immer falsch: es gibt keine pygame-Flagge fuer
            // "obenauf", also kann `flags` es gar nicht verlangen. Wer es
            // will, ruft danach [`Backend::set_obenauf`] - so macht es
            // auch [`crate::fenster::Fenster::neu`].
            obenauf: false,
        })
    }
}

impl Backend for MinifbBackend {
    fn name(&self) -> &'static str {
        "minifb"
    }

    /// Baut das Fenster um.
    ///
    /// minifb kann ein offenes Fenster nicht in der Groesse aendern, also
    /// wird ein neues aufgemacht und das alte fallen gelassen. Das
    /// flackert einmal - die Alternative waere schlimmer: ein Puffer, der
    /// nicht zum Fenster passt, zeigt mit `ScaleMode::UpperLeft` nur
    /// einen Ausschnitt, und der Rest des Bildes fehlt stillschweigend.
    ///
    /// Bei gleicher Groesse **und** gleichen Flaggen passiert nichts.
    ///
    /// Die Flaggen stehen hier mit in der Bedingung, weil sie in
    /// [`MinifbBackend::neu`] die `WindowOptions` bestimmen: ueber diesen
    /// Weg lassen sich `borderless` ([`crate::konstanten::NOFRAME`]) und
    /// `resizable` ([`crate::konstanten::RESIZABLE`]) eines offenen
    /// Fensters nachtraeglich umstellen, was minifb sonst nicht anbietet.
    fn set_mode(&mut self, groesse: (u32, u32), flags: u32) -> Ergebnis<()> {
        if groesse == self.groesse && flags == self.flags {
            return Ok(());
        }
        let titel = self.titel.clone();
        let obenauf = self.obenauf;
        let mut neu = MinifbBackend::neu(groesse, flags, &titel)?;
        // Der Umbau macht ein frisches Fenster - was nicht in den Flaggen
        // steht, muss von Hand mit hinueber.
        if obenauf {
            neu.fenster.topmost(true);
            neu.obenauf = true;
        }
        // Erst wenn das neue Fenster steht, wird das alte aufgegeben.
        *self = neu;
        Ok(())
    }

    fn set_caption(&mut self, titel: &str) {
        self.titel = titel.to_string();
        self.fenster.set_title(titel);
    }

    /// minifb nimmt ein Sinnbild nur als Dateipfad (Windows) oder als
    /// Rohzeiger auf einen ARGB-Puffer (X11), dessen Lebensdauer es nicht
    /// uebernimmt. Aus einer Flaeche laesst sich daraus nichts Sicheres
    /// bauen, also sagt es ehrlich nein, statt still nichts zu tun.
    fn set_icon(&mut self, _bild: &Surface) -> Ergebnis<()> {
        Err(Fehler::NichtUnterstuetzt(
            "minifb cannot set a window icon from a surface",
        ))
    }

    fn zeigen(&mut self, daten: &[u32], groesse: (u32, u32)) -> Ergebnis<()> {
        let (w, h) = (groesse.0 as usize, groesse.1 as usize);
        // Hier geht der Puffer unveraendert durch: ARGB8888 und minifbs
        // 0RGB unterscheiden sich nur im obersten Byte, das minifb wegwirft.
        self.fenster
            .update_with_buffer(&daten[..w * h], w, h)
            .map_err(|e| Fehler::Fenster(e.to_string()))?;
        self.lief_schon = true;
        Ok(())
    }

    fn pumpen(&mut self, eingabe: &mut Eingabe, heraus: &mut Vec<Event>) {
        if !self.fenster.is_open() {
            heraus.push(Event::quit());
            return;
        }
        // Ohne Bild hat minifb keinen Anlass, die Warteschlange des
        // Betriebssystems zu leeren - `update()` holt sie nach.
        if !self.lief_schon {
            self.fenster.update();
        }

        // --- Tasten: minifb liefert die Kanten selbst -------------------
        for k in self.fenster.get_keys_released() {
            let code = key_code(k);
            eingabe.taste_setzen(code, false);
            heraus.push(Event::key_up(code, eingabe.mods));
        }
        let gedrueckt = self.fenster.get_keys_pressed(KeyRepeat::Yes);
        // Erst die Zusatztasten setzen, damit ein gleichzeitiges
        // Shift+A schon das Shift im `mod` des A-Ereignisses fuehrt.
        for k in &gedrueckt {
            eingabe.taste_setzen(key_code(*k), true);
        }
        let zeichen: Vec<char> = self.zeichen.borrow_mut().drain(..).collect();
        let mut naechstes = zeichen.iter();
        for k in &gedrueckt {
            let code = key_code(*k);
            // Nur Tasten, die ein Zeichen erzeugen, bekommen eins - fuer
            // Pfeile und F-Tasten ist `unicode` bei pygame leer.
            let u = if erzeugt_zeichen(*k) {
                naechstes.next().copied().map(String::from).unwrap_or_default()
            } else {
                String::new()
            };
            heraus.push(Event::key_down(code, eingabe.mods, &u));
        }
        // Was uebrig blieb, ging an keine Taste - trotzdem Text.
        let rest: String = naechstes.collect();
        if !rest.is_empty() && eingabe.text_eingabe {
            heraus.push(Event::mit(TEXTINPUT, Daten::Text { text: rest }));
        }

        // --- Maus ------------------------------------------------------
        let tasten = [
            self.fenster.get_mouse_down(MouseButton::Left),
            self.fenster.get_mouse_down(MouseButton::Middle),
            self.fenster.get_mouse_down(MouseButton::Right),
        ];
        if let Some((x, y)) = self.fenster.get_mouse_pos(MouseMode::Pass) {
            let jetzt = (x as i32, y as i32);
            let alt = self.letzte_maus.unwrap_or(jetzt);
            if self.letzte_maus.is_none() || jetzt != alt {
                let rel = (jetzt.0 - alt.0, jetzt.1 - alt.1);
                eingabe.maus_pos = jetzt;
                eingabe.maus_rel.0 += rel.0;
                eingabe.maus_rel.1 += rel.1;
                heraus.push(Event::mouse_motion(
                    jetzt,
                    rel,
                    (tasten[0], tasten[1], tasten[2]),
                ));
            }
            self.letzte_maus = Some(jetzt);
        }
        for (i, jetzt) in tasten.iter().enumerate() {
            if *jetzt != self.letzte_tasten[i] {
                let knopf = i as u8 + 1;
                eingabe.maustaste_setzen(knopf, *jetzt);
                heraus.push(if *jetzt {
                    Event::mouse_button_down(eingabe.maus_pos, knopf)
                } else {
                    Event::mouse_button_up(eingabe.maus_pos, knopf)
                });
            }
        }
        self.letzte_tasten = tasten;

        if let Some((dx, dy)) = self.fenster.get_scroll_wheel() {
            if dx != 0.0 || dy != 0.0 {
                heraus.push(Event::mit(
                    MOUSEWHEEL,
                    Daten::MausRad {
                        x: dx.round() as i32,
                        y: dy.round() as i32,
                        precise_x: dx,
                        precise_y: dy,
                        flipped: false,
                    },
                ));
            }
        }

        // --- Groesse ---------------------------------------------------
        let (w, h) = self.fenster.get_size();
        let neu = (w as u32, h as u32);
        if neu != self.letzte_groesse {
            self.letzte_groesse = neu;
            heraus.push(Event::mit(
                VIDEORESIZE,
                Daten::Groesse {
                    size: neu,
                    w: neu.0 as i32,
                    h: neu.1 as i32,
                },
            ));
            heraus.push(Event::mit(
                WINDOWRESIZED,
                Daten::Fenster {
                    x: neu.0 as i32,
                    y: neu.1 as i32,
                },
            ));
        }

        // --- Fokus -----------------------------------------------------
        let fokus = self.fenster.is_active();
        if fokus != self.letzter_fokus {
            self.letzter_fokus = fokus;
            eingabe.tast_fokus = fokus;
            eingabe.maus_fokus = fokus;
            // Dieselbe Folge, die echtes pygame meldet - gemessen.
            heraus.push(Event::mit(
                ACTIVEEVENT,
                Daten::Aktiv {
                    gain: i32::from(fokus),
                    state: APPINPUTFOCUS as i32,
                },
            ));
            heraus.push(Event::neu(if fokus {
                WINDOWFOCUSGAINED
            } else {
                WINDOWFOCUSLOST
            }));
        }
    }

    fn window_size(&self) -> (u32, u32) {
        let (w, h) = self.fenster.get_size();
        (w as u32, h as u32)
    }

    /// minifb sagt nicht, welche Vollbild-Groessen es gibt. Was bleibt,
    /// ist die jetzige Fenstergroesse - besser als eine erfundene Liste.
    fn list_modes(&self) -> Vec<(u32, u32)> {
        vec![self.window_size()]
    }

    /// minifb kann den Zeiger nicht versetzen; das Ergebnis waere sonst
    /// ein Zeiger, der im Programm woanders steht als auf dem Schirm.
    fn set_mouse_pos(&mut self, _pos: (i32, i32)) {}

    fn set_mouse_visible(&mut self, sichtbar: bool) {
        self.fenster.set_cursor_visibility(sichtbar);
    }

    fn toggle_fullscreen(&mut self) -> Ergebnis<()> {
        Err(Fehler::NichtUnterstuetzt("minifb has no fullscreen mode"))
    }

    /// Das kann minifb wirklich: `Window::topmost()`.
    ///
    /// **Hier stand einmal, minifb koenne das nicht.** Der Vermerk war
    /// veraltet: `minifb::Window::topmost(bool)` gibt es in 0.27, und
    /// unter Windows ist es ein echtes `SetWindowPos(.., HWND_TOPMOST,
    /// ..)` - nachgesehen im Quelltext der Kiste, nicht vermutet.
    ///
    /// Gemessen an echtem pygame: unter `SDL_VIDEODRIVER=dummy` bleibt
    /// `always_on_top` nach dem Anlegen unveraenderlich. Mit minifb geht
    /// hier also **mehr** als bei pygame unter dem Dummy-Treiber - das
    /// ist der seltene Fall, in dem diese Kiste nicht hinterherhinkt.
    fn set_obenauf(&mut self, an: bool) -> Ergebnis<()> {
        self.fenster.topmost(an);
        self.obenauf = an;
        Ok(())
    }

    fn set_repeat(&mut self, verzoegerung: i32, abstand: i32) {
        // pygame rechnet in Millisekunden, minifb in Sekunden.
        // 0 heisst bei pygame "keine Wiederholung"; minifb kann das nicht
        // abschalten, also wird die Verzoegerung so gross, dass es im
        // Spielbetrieb nie dazu kommt.
        if verzoegerung <= 0 {
            self.fenster.set_key_repeat_delay(f32::MAX);
        } else {
            self.fenster
                .set_key_repeat_delay(verzoegerung as f32 / 1000.0);
            if abstand > 0 {
                self.fenster.set_key_repeat_rate(abstand as f32 / 1000.0);
            }
        }
    }

    fn ist_offen(&self) -> bool {
        self.fenster.is_open()
    }

    /// Das kann minifb wirklich: `Window::get_position()`.
    fn fenster_pos(&self) -> Ergebnis<(i32, i32)> {
        let (x, y) = self.fenster.get_position();
        Ok((x as i32, y as i32))
    }

    /// Und das auch: `Window::set_position()`.
    ///
    /// [`crate::konstanten::WINDOWPOS_CENTERED`] kann minifb nicht
    /// ausrechnen - es weiss nicht, wie gross die Arbeitsflaeche ist.
    /// Der Wert wuerde als riesige Zahl durchgereicht und das Fenster
    /// verschwaende; darum wird er hier abgefangen.
    fn set_fenster_pos(&mut self, pos: (i32, i32)) -> Ergebnis<()> {
        let zentriert = konstanten::WINDOWPOS_CENTERED as i32;
        if pos.0 == zentriert || pos.1 == zentriert {
            return Err(Fehler::NichtUnterstuetzt(
                "minifb kennt die Groesse der Arbeitsflaeche nicht, \
                 kann also nicht mittig stellen",
            ));
        }
        self.fenster.set_position(pos.0 as isize, pos.1 as isize);
        Ok(())
    }

    /// minifb hat keine Bildschirmliste - `Window` kennt nur sich selbst.
    ///
    /// Eine Zahl zu erfinden waere hier besonders teuer: wer
    /// `get_desktop_sizes()` fragt, will meist ein Fenster einpassen, und
    /// eine falsche Groesse schiebt es aus dem Bild.
    fn desktop_groessen(&self) -> Ergebnis<Vec<(u32, u32)>> {
        Err(Fehler::NichtUnterstuetzt(
            "minifb cannot list the displays",
        ))
    }

    /// minifb fragt die Bildwiederholrate nirgends ab.
    ///
    /// `set_target_fps` setzt nur eine eigene Bremse und sagt nichts
    /// darueber, wie oft der Bildschirm wirklich aufbaut. 60
    /// zurueckzugeben waere geraten - auf einem 144-Hz-Schirm falsch,
    /// und zwar auf die Art, die man erst im Spiel merkt.
    fn bildwiederholraten(&self) -> Ergebnis<Vec<u32>> {
        Err(Fehler::NichtUnterstuetzt(
            "minifb does not query the refresh rate",
        ))
    }

    fn bildwiederholrate(&self) -> Ergebnis<u32> {
        Err(Fehler::NichtUnterstuetzt(
            "minifb does not query the refresh rate",
        ))
    }

    /// minifb kann ein Fenster nicht als Sinnbild ablegen - es gibt
    /// weder `minimize()` noch einen Weg an das Fenster des Systems, der
    /// ohne unsicheren Code auskaeme.
    fn iconify(&mut self) -> Ergebnis<bool> {
        Err(Fehler::NichtUnterstuetzt(
            "minifb cannot resize the window",
        ))
    }

    /// Hier ist es genau andersherum als beim fensterlosen Stueck:
    /// minifb kennt **acht feste Formen** (`CursorStyle`) und keine
    /// eigenen.
    ///
    /// Die Zuordnung steht in `cursor_style` weiter unten; von pygames zwoelf
    /// `SYSTEM_CURSOR_*` lassen sich neun abbilden. Die drei uebrigen
    /// (`WAIT`, `WAITARROW`, `NO`) hat minifb nicht - dafuer eine
    /// aehnliche Form zu nehmen waere eine stille Luege, also sagen sie
    /// nein.
    ///
    /// Eine eigene Pixelform kann minifb gar nicht: es gibt keinen
    /// Aufruf, der ihm eine Bitmap oder eine Flaeche als Zeiger gaebe.
    fn set_cursor(&mut self, zeiger: &Cursor) -> Ergebnis<()> {
        match zeiger {
            Cursor::System(k) => match cursor_style(*k) {
                Some(s) => {
                    self.fenster.set_cursor_style(s);
                    Ok(())
                }
                None => Err(Fehler::NichtUnterstuetzt(
                    "minifb does not know this system cursor",
                )),
            },
            _ => Err(Fehler::NichtUnterstuetzt(
                "minifb does not accept a custom cursor",
            )),
        }
    }

    /// minifb kann den Zeiger nicht im Fenster festhalten - es gibt
    /// keinen Aufruf dafuer, und ohne [`Backend::set_mouse_pos`] liesse
    /// er sich auch nicht von Hand zurueckholen.
    ///
    /// `false` heisst hier: es hat **nicht** gegriffen. `get_grab()`
    /// meldet darum weiterhin `false`, statt einen Griff zu behaupten,
    /// den es nicht gibt.
    fn set_grab(&mut self, _an: bool) -> bool {
        false
    }

    /// Das ist genau der Schluessel, den auch SDL dort ablegt: die
    /// Fensternummer des Systems.
    ///
    /// `Window::get_window_handle()` gibt sie als Zeiger; hier steht sie
    /// als Zahl, wie in pygames `get_wm_info()["window"]`.
    fn wm_info(&self) -> Vec<(&'static str, u64)> {
        vec![("window", self.fenster.get_window_handle() as usize as u64)]
    }
}

/// pygames `SYSTEM_CURSOR_*` -> minifbs `CursorStyle`.
///
/// Neun der zwoelf lassen sich abbilden. `SYSTEM_CURSOR_WAIT`,
/// `SYSTEM_CURSOR_WAITARROW` und `SYSTEM_CURSOR_NO` haben bei minifb
/// kein Gegenstueck - dafuer den Pfeil zu nehmen hiesse, eine
/// Sanduhr zu versprechen und einen Pfeil zu zeigen.
fn cursor_style(k: crate::Key) -> Option<minifb::CursorStyle> {
    use minifb::CursorStyle as C;
    Some(match k {
        konstanten::SYSTEM_CURSOR_ARROW => C::Arrow,
        konstanten::SYSTEM_CURSOR_IBEAM => C::Ibeam,
        konstanten::SYSTEM_CURSOR_CROSSHAIR => C::Crosshair,
        konstanten::SYSTEM_CURSOR_HAND => C::OpenHand,
        konstanten::SYSTEM_CURSOR_SIZEWE => C::ResizeLeftRight,
        konstanten::SYSTEM_CURSOR_SIZENS => C::ResizeUpDown,
        konstanten::SYSTEM_CURSOR_SIZEALL => C::ResizeAll,
        // Die beiden Diagonalen kennt minifb nicht einzeln; `ResizeAll`
        // ist die naechstliegende Form, die wirklich "Groesse aendern"
        // heisst - und nicht bloss irgendein Pfeil.
        konstanten::SYSTEM_CURSOR_SIZENWSE => C::ResizeAll,
        konstanten::SYSTEM_CURSOR_SIZENESW => C::ResizeAll,
        _ => return None,
    })
}

/// Erzeugt diese Taste ein Zeichen?
///
/// Pfeile, F-Tasten und Zusatztasten tun es nicht - bei pygame ist deren
/// `unicode` leer.
fn erzeugt_zeichen(k: Key) -> bool {
    !matches!(
        k,
        Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::Home
            | Key::End
            | Key::PageUp
            | Key::PageDown
            | Key::Insert
            | Key::Delete
            | Key::LeftShift
            | Key::RightShift
            | Key::LeftCtrl
            | Key::RightCtrl
            | Key::LeftAlt
            | Key::RightAlt
            | Key::LeftSuper
            | Key::RightSuper
            | Key::CapsLock
            | Key::NumLock
            | Key::ScrollLock
            | Key::Pause
            | Key::Menu
            | Key::Unknown
            | Key::F1
            | Key::F2
            | Key::F3
            | Key::F4
            | Key::F5
            | Key::F6
            | Key::F7
            | Key::F8
            | Key::F9
            | Key::F10
            | Key::F11
            | Key::F12
            | Key::F13
            | Key::F14
            | Key::F15
    )
}

/// minifb-Taste -> pygame-Tastencode.
///
/// Die Zahlen stehen nicht hier, sondern in [`crate::konstanten`] - sie
/// sind aus echtem pygame ausgelesen. Dieses `match` ist nur die schnelle
/// Fassung davon; der Test `minifb_zuordnung_stimmt_mit_pygame_ueberein`
/// prueft Taste fuer Taste, dass beide dasselbe sagen.
pub(crate) fn key_code(k: Key) -> crate::Key {
    match k {
        Key::Key0 => K_0,
        Key::Key1 => K_1,
        Key::Key2 => K_2,
        Key::Key3 => K_3,
        Key::Key4 => K_4,
        Key::Key5 => K_5,
        Key::Key6 => K_6,
        Key::Key7 => K_7,
        Key::Key8 => K_8,
        Key::Key9 => K_9,
        Key::A => K_a,
        Key::B => K_b,
        Key::C => K_c,
        Key::D => K_d,
        Key::E => K_e,
        Key::F => K_f,
        Key::G => K_g,
        Key::H => K_h,
        Key::I => K_i,
        Key::J => K_j,
        Key::K => K_k,
        Key::L => K_l,
        Key::M => K_m,
        Key::N => K_n,
        Key::O => K_o,
        Key::P => K_p,
        Key::Q => K_q,
        Key::R => K_r,
        Key::S => K_s,
        Key::T => K_t,
        Key::U => K_u,
        Key::V => K_v,
        Key::W => K_w,
        Key::X => K_x,
        Key::Y => K_y,
        Key::Z => K_z,
        Key::F1 => K_F1,
        Key::F2 => K_F2,
        Key::F3 => K_F3,
        Key::F4 => K_F4,
        Key::F5 => K_F5,
        Key::F6 => K_F6,
        Key::F7 => K_F7,
        Key::F8 => K_F8,
        Key::F9 => K_F9,
        Key::F10 => K_F10,
        Key::F11 => K_F11,
        Key::F12 => K_F12,
        Key::F13 => K_F13,
        Key::F14 => K_F14,
        Key::F15 => K_F15,
        Key::Down => K_DOWN,
        Key::Left => K_LEFT,
        Key::Right => K_RIGHT,
        Key::Up => K_UP,
        Key::Apostrophe => K_QUOTE,
        Key::Backquote => K_BACKQUOTE,
        Key::Backslash => K_BACKSLASH,
        Key::Comma => K_COMMA,
        Key::Equal => K_EQUALS,
        Key::LeftBracket => K_LEFTBRACKET,
        Key::Minus => K_MINUS,
        Key::Period => K_PERIOD,
        Key::RightBracket => K_RIGHTBRACKET,
        Key::Semicolon => K_SEMICOLON,
        Key::Slash => K_SLASH,
        Key::Backspace => K_BACKSPACE,
        Key::Delete => K_DELETE,
        Key::End => K_END,
        // minifb sagt "Enter", pygame sagt "Return" - dieselbe Taste.
        Key::Enter => K_RETURN,
        Key::Escape => K_ESCAPE,
        Key::Home => K_HOME,
        Key::Insert => K_INSERT,
        Key::Menu => K_MENU,
        Key::PageDown => K_PAGEDOWN,
        Key::PageUp => K_PAGEUP,
        Key::Pause => K_PAUSE,
        Key::Space => K_SPACE,
        Key::Tab => K_TAB,
        Key::NumLock => K_NUMLOCK,
        Key::CapsLock => K_CAPSLOCK,
        Key::ScrollLock => K_SCROLLLOCK,
        Key::LeftShift => K_LSHIFT,
        Key::RightShift => K_RSHIFT,
        Key::LeftCtrl => K_LCTRL,
        Key::RightCtrl => K_RCTRL,
        Key::NumPad0 => K_KP0,
        Key::NumPad1 => K_KP1,
        Key::NumPad2 => K_KP2,
        Key::NumPad3 => K_KP3,
        Key::NumPad4 => K_KP4,
        Key::NumPad5 => K_KP5,
        Key::NumPad6 => K_KP6,
        Key::NumPad7 => K_KP7,
        Key::NumPad8 => K_KP8,
        Key::NumPad9 => K_KP9,
        Key::NumPadDot => K_KP_PERIOD,
        Key::NumPadSlash => K_KP_DIVIDE,
        Key::NumPadAsterisk => K_KP_MULTIPLY,
        Key::NumPadMinus => K_KP_MINUS,
        Key::NumPadPlus => K_KP_PLUS,
        Key::NumPadEnter => K_KP_ENTER,
        Key::LeftAlt => K_LALT,
        Key::RightAlt => K_RALT,
        Key::LeftSuper => K_LGUI,
        Key::RightSuper => K_RGUI,
        _ => K_UNKNOWN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Jede minifb-Taste muss denselben Code liefern wie die Tabelle, die
    /// aus echtem pygame ausgelesen wurde.
    ///
    /// Der Test braucht kein Fenster - `minifb::Key` ist nur ein Enum.
    #[test]
    fn minifb_zuordnung_stimmt_mit_pygame_ueberein() {
        // Alle Varianten, `Count` ausgenommen - das ist keine Taste.
        let alle: Vec<Key> = (0..107u8)
            .filter_map(|i| ALLE_TASTEN.get(i as usize).copied())
            .collect();
        assert_eq!(alle.len(), MINIFB_KEYS.len(), "Tastenzahl weicht ab");
        for k in alle {
            let name = format!("{k:?}");
            let erwartet = MINIFB_KEYS
                .binary_search_by_key(&name.as_str(), |(n, _)| *n)
                .map(|i| MINIFB_KEYS[i].1)
                .unwrap_or_else(|_| panic!("{name} fehlt in MINIFB_KEYS"));
            assert_eq!(
                key_code(k),
                erwartet,
                "{name}: match sagt {}, pygame sagt {erwartet}",
                key_code(k)
            );
        }
    }

    /// Alle 107 Tasten, die minifb kennt (ohne `Count`).
    static ALLE_TASTEN: &[Key] = &[
        Key::Key0, Key::Key1, Key::Key2, Key::Key3, Key::Key4, Key::Key5,
        Key::Key6, Key::Key7, Key::Key8, Key::Key9,
        Key::A, Key::B, Key::C, Key::D, Key::E, Key::F, Key::G, Key::H,
        Key::I, Key::J, Key::K, Key::L, Key::M, Key::N, Key::O, Key::P,
        Key::Q, Key::R, Key::S, Key::T, Key::U, Key::V, Key::W, Key::X,
        Key::Y, Key::Z,
        Key::F1, Key::F2, Key::F3, Key::F4, Key::F5, Key::F6, Key::F7,
        Key::F8, Key::F9, Key::F10, Key::F11, Key::F12, Key::F13,
        Key::F14, Key::F15,
        Key::Down, Key::Left, Key::Right, Key::Up,
        Key::Apostrophe, Key::Backquote, Key::Backslash, Key::Comma,
        Key::Equal, Key::LeftBracket, Key::Minus, Key::Period,
        Key::RightBracket, Key::Semicolon, Key::Slash, Key::Backspace,
        Key::Delete, Key::End, Key::Enter, Key::Escape, Key::Home,
        Key::Insert, Key::Menu, Key::PageDown, Key::PageUp, Key::Pause,
        Key::Space, Key::Tab, Key::NumLock, Key::CapsLock,
        Key::ScrollLock, Key::LeftShift, Key::RightShift, Key::LeftCtrl,
        Key::RightCtrl,
        Key::NumPad0, Key::NumPad1, Key::NumPad2, Key::NumPad3,
        Key::NumPad4, Key::NumPad5, Key::NumPad6, Key::NumPad7,
        Key::NumPad8, Key::NumPad9, Key::NumPadDot, Key::NumPadSlash,
        Key::NumPadAsterisk, Key::NumPadMinus, Key::NumPadPlus,
        Key::NumPadEnter,
        Key::LeftAlt, Key::RightAlt, Key::LeftSuper, Key::RightSuper,
        Key::Unknown,
    ];
}
