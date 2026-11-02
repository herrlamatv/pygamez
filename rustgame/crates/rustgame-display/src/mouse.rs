//! mouse.rs
//! ========
//! Gegenstueck zu `pygame.mouse`.
//!
//! ## Was an echtem pygame gemessen wurde
//!
//! | Frage | Antwort |
//! |---|---|
//! | `get_pressed()` | drei Werte - links, mitte, rechts |
//! | `get_pressed(5)` | fuenf - die beiden Seitentasten dazu |
//! | `set_visible(False)` | gibt den **vorherigen** Zustand zurueck, nicht den neuen |
//! | `set_pos((100, 50))` danach `get_rel()` | `(0, 0)` - ein Sprung zaehlt nicht als Bewegung |
//! | `set_pos` | legt ein `MOUSEMOTION` mit `rel=(0, 0)` ab |
//! | `get_cursor()` ganz am Anfang | `Cursor(type: system, constant: SYSTEM_CURSOR_ARROW)`, `data == (0,)` |
//! | `set_cursor(SYSTEM_CURSOR_HAND)` unter `dummy` | Fehler `Error while creating system cursor` |
//! | `set_cursor(cursors.diamond)` unter `dummy` | geht - danach ist `get_cursor().type` `'bitmap'` |
//! | `set_cursor((3, 4), surf)` unter `dummy` | geht ebenfalls - `'color'` |
//! | `set_cursor` nach einem Fehlschlag | die **alte** Form bleibt stehen |
//! | `get_relative_mode()` vor `display.init()` | `False`, **kein** Fehler |
//! | `set_relative_mode(True)` unter `dummy` | Fehler `No relative mode implementation available` |
//! | `set_relative_mode(False)` | `None` - und `get_relative_mode()` bleibt `False` |
//! | `get_just_pressed()` | ein Fuenfer-Tupel, wie `get_pressed(5)` |
//!
//! Der zweite Punkt ist der, an dem man ohne Messung vorbeilaeuft:
//! `set_visible` liefert **den alten** Zustand. Wer das Ergebnis als "ist
//! jetzt sichtbar" liest, baut sich einen Zeiger, der genau verkehrt
//! herum blinkt.
//!
//! **Und der Punkt, an dem dieselbe Vermutung danebengeht:**
//! [`set_relative_mode`] und [`set_cursor`] machen es *nicht* so.
//! Nachgemessen geben beide in pygame `None` zurueck - keinen vorherigen
//! Zustand, keinen Erfolgswert. Der Rueckgabewert von `set_visible` ist
//! eine Eigenart genau dieses einen Aufrufs, keine Hausregel.

use std::rc::Rc;

use rustgame_core::Surface;

use crate::event::{Daten, Event};
use crate::konstanten::*;
use crate::zustand::mit_zustand;
use crate::{Ergebnis, Fehler};

/// Wo der Zeiger steht - `pygame.mouse.get_pos()`.
pub fn get_pos() -> (i32, i32) {
    mit_zustand(|z| {
        crate::event::pumpen(z);
        z.eingabe.maus_pos
    })
}

/// Welche Maustasten unten sind - `pygame.mouse.get_pressed()`.
///
/// Drei Werte: links, mitte, rechts. Fuer fuenf siehe [`get_pressed5`].
pub fn get_pressed() -> (bool, bool, bool) {
    let t = get_pressed5();
    (t[0], t[1], t[2])
}

/// Alle fuenf Maustasten - `pygame.mouse.get_pressed(5)`.
pub fn get_pressed5() -> [bool; 5] {
    mit_zustand(|z| {
        crate::event::pumpen(z);
        z.eingabe.maus_tasten
    })
}

/// Versetzt den Zeiger - `pygame.mouse.set_pos()`.
///
/// Gemessen: pygame legt dabei ein `MOUSEMOTION` mit `rel=(0, 0)` ab -
/// ein Sprung ist keine Bewegung, und [`get_rel`] meldet danach
/// entsprechend nichts. Das ist wichtig fuer Programme, die den Zeiger
/// zur Fenstermitte zuruecksetzen, um endlos drehen zu koennen: wuerde
/// der Sprung mitzaehlen, hoebe er die Drehung genau wieder auf.
///
/// minifb kann den Zeiger nicht wirklich versetzen; dort bleibt es beim
/// gemerkten Stand, damit Programm und Schirm nicht auseinanderlaufen.
pub fn set_pos(pos: (i32, i32)) {
    mit_zustand(|z| {
        z.eingabe.maus_pos = pos;
        if let Some(b) = z.backend.as_mut() {
            b.set_mouse_pos(pos);
        }
        let tasten = z.eingabe.maus_tasten;
        z.schlange.ablegen(Event::mit(
            MOUSEMOTION,
            Daten::MausBewegung {
                pos,
                rel: (0, 0),
                buttons: (tasten[0], tasten[1], tasten[2]),
                touch: false,
            },
        ));
    })
}

/// Wie weit sich der Zeiger seit dem letzten Aufruf bewegt hat -
/// `pygame.mouse.get_rel()`.
///
/// Der Zaehler wird dabei zurueckgesetzt, wie bei pygame: zweimal
/// hintereinander gefragt heisst beim zweiten Mal `(0, 0)`.
pub fn get_rel() -> (i32, i32) {
    mit_zustand(|z| {
        crate::event::pumpen(z);
        let r = z.eingabe.maus_rel;
        z.eingabe.maus_rel = (0, 0);
        r
    })
}

/// Zeigt oder versteckt den Zeiger - `pygame.mouse.set_visible()`.
///
/// Gibt den **vorherigen** Zustand zurueck, nicht den neuen - gemessen an
/// echtem pygame: `set_visible(False)` liefert `True`, `set_visible(True)`
/// danach `False`.
pub fn set_visible(sichtbar: bool) -> bool {
    mit_zustand(|z| {
        let vorher = z.eingabe.maus_sichtbar;
        z.eingabe.maus_sichtbar = sichtbar;
        if let Some(b) = z.backend.as_mut() {
            b.set_mouse_visible(sichtbar);
        }
        vorher
    })
}

/// Ist der Zeiger sichtbar? - `pygame.mouse.get_visible()`.
pub fn get_visible() -> bool {
    mit_zustand(|z| z.eingabe.maus_sichtbar)
}

/// Ist der Zeiger ueber dem Fenster? - `pygame.mouse.get_focused()`.
pub fn get_focused() -> bool {
    mit_zustand(|z| {
        crate::event::pumpen(z);
        z.eingabe.maus_fokus
    })
}

// ---------------------------------------------------------------------------
//  Die Zeigerform
// ---------------------------------------------------------------------------

/// Eine Zeigerform - `pygame.cursors.Cursor`.
///
/// pygame kennt drei Arten, und `Cursor.type` sagt welche: `"system"`,
/// `"bitmap"`, `"color"`. Hier sind es drei Varianten, damit der
/// Uebersetzer die Fallunterscheidung erzwingt statt eines
/// `data`-Tupels, dessen Inhalt von `type` abhaengt.
///
/// Gemessen: ganz am Anfang steht `Cursor(type: system, constant:
/// SYSTEM_CURSOR_ARROW)` und `cursor.data` ist `(0,)`.
#[derive(Debug, Clone)]
pub enum Cursor {
    /// Eine der zwoelf Systemformen - `SYSTEM_CURSOR_*` aus
    /// [`crate::konstanten`].
    System(u32),
    /// Eine Schwarzweissform aus zwei Bitmasken, wie `pygame.cursors`
    /// sie mitbringt (`cursors.diamond`, `cursors.broken_x`).
    ///
    /// Gemessen an `cursors.diamond`: `size` ist `(16, 16)`, `hotspot`
    /// `(7, 7)`, und beide Masken sind 32 Byte lang - also
    /// `breite / 8 * hoehe`.
    Bitmap {
        /// Breite und Hoehe in Pixeln; die Breite ist ein Vielfaches von 8.
        groesse: (u32, u32),
        /// Der Punkt, der "zeigt".
        hotspot: (i32, i32),
        /// Welche Pixel gesetzt werden.
        xormasks: Vec<u8>,
        /// Welche Pixel ueberhaupt gezeichnet werden.
        andmasks: Vec<u8>,
    },
    /// Eine Form aus einer farbigen Flaeche.
    ///
    /// Die Flaeche haengt am `Rc`, damit [`Cursor`] billig zu klonen ist
    /// und zwei Zeiger sich vergleichen lassen - siehe die
    /// `PartialEq`-Umsetzung.
    Farbe {
        /// Der Punkt, der "zeigt".
        hotspot: (i32, i32),
        /// Das Bild.
        bild: Rc<Surface>,
    },
}

impl Cursor {
    /// Die Art, wie `pygame.cursors.Cursor.type` sie nennt.
    pub fn art(&self) -> &'static str {
        match self {
            Cursor::System(_) => "system",
            Cursor::Bitmap { .. } => "bitmap",
            Cursor::Farbe { .. } => "color",
        }
    }

    /// Eine Form aus einer Flaeche.
    pub fn farbe(hotspot: (i32, i32), bild: Surface) -> Cursor {
        Cursor::Farbe {
            hotspot,
            bild: Rc::new(bild),
        }
    }
}

impl Default for Cursor {
    /// Gemessen: der Pfeil, `SYSTEM_CURSOR_ARROW` (0).
    fn default() -> Cursor {
        Cursor::System(SYSTEM_CURSOR_ARROW)
    }
}

/// Zwei Zeiger sind gleich, wenn sie dieselbe Form meinen.
///
/// Bei [`Cursor::Farbe`] heisst das **dasselbe Bild**, nicht "ein Bild
/// mit denselben Pixeln": `rustgame_core::Surface` kennt keinen
/// Gleichheitsvergleich, und zwei zufaellig gleich aussehende Bilder
/// gleichzusetzen waere eine Behauptung, die diese Kiste nicht pruefen
/// kann. Verglichen wird darum die Herkunft (`Rc::ptr_eq`).
impl PartialEq for Cursor {
    fn eq(&self, other: &Cursor) -> bool {
        match (self, other) {
            (Cursor::System(a), Cursor::System(b)) => a == b,
            (
                Cursor::Bitmap {
                    groesse: g1,
                    hotspot: h1,
                    xormasks: x1,
                    andmasks: a1,
                },
                Cursor::Bitmap {
                    groesse: g2,
                    hotspot: h2,
                    xormasks: x2,
                    andmasks: a2,
                },
            ) => g1 == g2 && h1 == h2 && x1 == x2 && a1 == a2,
            (
                Cursor::Farbe {
                    hotspot: h1,
                    bild: b1,
                },
                Cursor::Farbe {
                    hotspot: h2,
                    bild: b2,
                },
            ) => h1 == h2 && Rc::ptr_eq(b1, b2),
            _ => false,
        }
    }
}

/// Die jetzige Zeigerform - `pygame.mouse.get_cursor()`.
///
/// Gemessen: vor `display.init()` wirft pygame hier
/// `video system not initialized`; danach steht dort der Pfeil.
pub fn get_cursor() -> Ergebnis<Cursor> {
    mit_zustand(|z| {
        if !z.init {
            return Err(Fehler::NichtInitialisiert);
        }
        Ok(z.eingabe.zeiger.clone())
    })
}

/// Setzt die Zeigerform - `pygame.mouse.set_cursor()`.
///
/// Der Rueckgabewert ist **kein** vorheriger Zustand: gemessen gibt
/// pygame hier `None` zurueck, anders als bei [`set_visible`].
///
/// Was welches Hintergrundstueck kann:
///
/// * **Ohne Fenster** ist es wie bei echtem pygame unter
///   `SDL_VIDEODRIVER=dummy`: [`Cursor::Bitmap`] und [`Cursor::Farbe`]
///   werden klaglos entgegengenommen (es gibt ja nichts zu zeichnen),
///   eine [`Cursor::System`] dagegen wirft
///   `Error while creating system cursor` - gemessen, beides.
/// * **Mit minifb** ist es andersherum: minifb kennt acht feste Formen
///   (`CursorStyle`), also geht [`Cursor::System`] fuer die Konstanten,
///   die sich darauf abbilden lassen. Eigene Bilder kann minifb nicht -
///   es gibt keinen Weg, ihm eine Pixelform zu uebergeben.
///
/// Gemessen und hier genauso: **schlaegt es fehl, bleibt die alte Form
/// stehen.** Nur ein erfolgreicher Aufruf aendert [`get_cursor`].
pub fn set_cursor(zeiger: Cursor) -> Ergebnis<()> {
    mit_zustand(|z| {
        if !z.init {
            return Err(Fehler::NichtInitialisiert);
        }
        if let Some(b) = z.backend.as_mut() {
            b.set_cursor(&zeiger)?;
        }
        z.eingabe.zeiger = zeiger;
        Ok(())
    })
}

/// Setzt eine der zwoelf Systemformen - `pygame.mouse.set_system_cursor()`.
///
/// Gleichbedeutend mit `set_cursor(Cursor::System(k))`. In pygame ist
/// dieser Aufruf seit 2.2 veraltet; er steht hier, weil vorhandene
/// Programme ihn benutzen.
pub fn set_system_cursor(konstante: u32) -> Ergebnis<()> {
    set_cursor(Cursor::System(konstante))
}

/// Laeuft die Maus im Relativbetrieb? - `pygame.mouse.get_relative_mode()`.
///
/// Gemessen: das ist der einzige Aufruf aus diesem Modul, der auch vor
/// `display.init()` klaglos antwortet - mit `False`.
pub fn get_relative_mode() -> bool {
    mit_zustand(|z| z.eingabe.relativ)
}

/// Schaltet den Relativbetrieb - `pygame.mouse.set_relative_mode()`.
///
/// Im Relativbetrieb verschwindet der Zeiger, bleibt im Fenster gefangen
/// und meldet trotzdem weiter Bewegung, auch am Rand. Das ist das, was
/// ein Spiel mit freier Blickrichtung braucht.
///
/// **Hier gibt es das nicht, und das ist kein stilles Nichtstun:**
/// `set_relative_mode(true)` meldet [`Fehler::Pygame`] mit genau dem
/// Text, den echtes pygame unter `SDL_VIDEODRIVER=dummy` wirft -
/// `No relative mode implementation available`. Der Grund ist derselbe
/// wie dort: es gibt keinen Unterbau dafuer. minifb kann den Zeiger
/// weder fangen noch versetzen (siehe [`set_pos`]), ohne beides waere
/// der Relativbetrieb eine Kamera, die am Fensterrand haengenbleibt.
///
/// `set_relative_mode(false)` geht durch - gemessen gibt pygame dort
/// `None` zurueck, und `get_relative_mode()` bleibt `False`.
pub fn set_relative_mode(an: bool) -> Ergebnis<()> {
    if an {
        return Err(Fehler::Pygame("No relative mode implementation available"));
    }
    mit_zustand(|z| {
        z.eingabe.relativ = false;
        Ok(())
    })
}

/// Welche Maustasten in **diesem** Durchlauf heruntergegangen sind -
/// `pygame.mouse.get_just_pressed()`.
///
/// Fuenf Werte, wie [`get_pressed5`] - aber `true` nur in dem Durchlauf,
/// in dem die Taste heruntergeht. Zurueckgesetzt wird beim naechsten
/// Blick in die Ereignisschlange, siehe `event::durchlauf`.
///
/// **Dieser Aufruf pumpt nicht** - und das ist Absicht: er liest den
/// Vermerk des letzten Durchlaufs, und zu pumpen hiesse, ihn vorher zu
/// loeschen. pygame macht es genauso. Die Reihenfolge ist also erst
/// `event::get()`, dann fragen.
///
/// Wer die Reihenfolge mehrerer Klicks braucht, nimmt
/// `MOUSEBUTTONDOWN` aus der Schlange - hier steht nur, *ob* geklickt
/// wurde, nicht wie oft.
pub fn get_just_pressed() -> [bool; 5] {
    mit_zustand(|z| z.eingabe.maus_neu)
}

/// Welche Maustasten in diesem Durchlauf heraufgekommen sind -
/// `pygame.mouse.get_just_released()`.
///
/// Das Gegenstueck zu [`get_just_pressed`], mit denselben Regeln.
pub fn get_just_released() -> [bool; 5] {
    mit_zustand(|z| z.eingabe.maus_weg)
}
