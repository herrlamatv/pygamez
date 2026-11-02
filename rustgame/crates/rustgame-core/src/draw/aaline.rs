//! aaline.rs
//! =========
//! `pygame.draw.aaline` und `pygame.draw.aalines` - die **erste** Stelle im
//! ganzen `draw`-Modul, die wirklich *mischt*.
//!
//! Alle anderen Zeichenfunktionen rastern binaer und schreiben die Farbe
//! roh. Hier entstehen Zwischenwerte, und die Formel dafuer ist **keine**
//! der vier aus [`crate::blend_formula`] - die gelten fuer den Blit.
//! `draw.c` rechnet in Gleitkomma:
//!
//! ```c
//! /* src_c/draw.c: get_antialiased_color */
//! color_part[0] = (Uint8)(brightness * color_part[0] +
//!                         (1 - brightness) * background_color[0]);
//! ```
//!
//! Vier Dinge daran sind gemessen und wichtig:
//!
//! * Es ist `float`-Arithmetik mit **Abschneiden** zur Null hin, nicht
//!   Runden. Beleg: `aaline` von `(1,1)` nach `(10,6)`, Rot auf Schwarz,
//!   Pixel `(2,2)` hat `brightness = 0.5` und wird `127`, nicht `128`.
//! * **Der Alphakanal wird mitgerechnet.** Auf einer `SRCALPHA`-Flaeche
//!   (Hintergrund durchsichtig) ergibt Rot mit `brightness = 0.5` den Wert
//!   `(127, 0, 0, 127)` - Alpha wandert genauso mit wie Rot.
//! * Der Hintergrund kommt ueber `PG_GetRGBA`. Ohne Alphakanal liefert das
//!   **immer 255** im vierten Kanal, gleichgueltig was im ungenutzten
//!   vierten Byte steht - deshalb hier [`Surface::unmap_rgb`] und nicht der
//!   rohe Wert.
//! * Liegt das Pixel ausserhalb des Clip-Rechtecks, gibt `get_antialiased_color`
//!   die Farbe **unveraendert** zurueck; geschrieben wird es dann ohnehin nicht.
//!
//! ## Was ein gutgemeinter Nachbau verfehlen wuerde
//!
//! * **`width < 1` zeichnet trotzdem.** Bei `line` heisst `width < 1`
//!   "nichts zeichnen"; `aaline` behandelt jedes `width <= 1` als `1` -
//!   auch `0` und `-5`. Im C steht dazu der Kommentar *"this helps compat
//!   with the old blend argument"*. Gemessen: `aaline(..., width=-3)`
//!   liefert dieselben Pixel wie `width=1`.
//! * **Die dicke `aaline` beschneidet senkrecht gar nicht.** In
//!   `draw_aaline_width` steht die senkrechte Beschneidung in einem
//!   `if (from_x < clip_left + 1)`, das drei Zeilen darueber bereits
//!   unmoeglich gemacht wurde (`from_x = clip_left + 1`). Toter Code im
//!   Original - hier absichtlich mit uebernommen, weil er sonst Pixel
//!   verschoebe.
//! * **`draw_aaline_width` setzt ein Pixel an einer anderen Stelle, als es
//!   die Mischfarbe geholt hat**: `get_antialiased_color(x - width, y)`,
//!   danach `set_and_check_rect(x - width + extra_width - 1, y)`. Bei
//!   gerader Staerke sind das zwei verschiedene Pixel. Ebenfalls uebernommen.
//! * **`aalines` zeichnet die Strecke bei genau zwei Punkten zweimal** -
//!   einmal aus dem "erste Linie"-Block, einmal aus dem "letzte
//!   Linie"-Block. Bei gemischten Pixeln ist das sichtbar: das zweite Mal
//!   mischt auf das Ergebnis des ersten.
//! * **`aalines` vergleicht bei der ersten Linie `ylist[2]` mit `pts[1]`**
//!   statt mit `pts[3]` - ein Tippfehler im Original, der die Entscheidung
//!   ueber das Extrapixel verschiebt.

use crate::color::{Color, IntoColor};
use crate::draw::line::{spanne, Bereich};
use crate::rect::Rect;
use crate::surface::Surface;

// ---------------------------------------------------------------------------
//  Die Mischformel
// ---------------------------------------------------------------------------

/// pygames `get_antialiased_color`: die Farbe, die an `(x, y)` entstehen
/// soll, wenn `farbe` dort mit `helligkeit` ueber den vorhandenen Inhalt
/// gelegt wird.
///
/// `farbe` und der Rueckgabewert sind **abgebildete** Pixelwerte
/// (`Surface::abbilden`), keine `Color`.
///
/// Die Umwandlung `(Uint8)` im Original ist ein C-Cast von `float` auf
/// `Uint8`: erst zur Null hin abschneiden, dann das unterste Byte nehmen.
/// Das ist kein Haarspalten - `helligkeit` kann an den Endpunkten ueber 1
/// liegen (`brightness * x_gap` mit `x_gap` bis 2), und dann laeuft die
/// Rechnung ueber 255 hinaus. Darum hier `as i32 as u8` und nicht das
/// saettigende `as u8`.
pub(crate) fn aa_farbe(s: &Surface, x: i32, y: i32, farbe: u32, helligkeit: f32) -> u32 {
    let clip = s.get_clip();
    if x < clip.x || x >= clip.x + clip.w || y < clip.y || y >= clip.y + clip.h {
        return farbe;
    }
    let vorn = s.unmap_rgb(farbe);
    let hinten = s.unmap_rgb(s.get_at_mapped((x, y)));
    let misch = |v: u8, h: u8| -> u8 {
        (helligkeit * v as f32 + (1.0 - helligkeit) * h as f32) as i32 as u8
    };
    s.abbilden(Color::rgba(
        misch(vorn.r, hinten.r),
        misch(vorn.g, hinten.g),
        misch(vorn.b, hinten.b),
        misch(vorn.a, hinten.a),
    ))
}

// ---------------------------------------------------------------------------
//  Kleinste Bausteine - alle auf **abgebildeten** Farben
// ---------------------------------------------------------------------------

/// pygames `set_and_check_rect`, aber mit einem fertigen Pixelwert.
fn setze_roh(s: &mut Surface, b: &mut Bereich, x: i32, y: i32, farbe: u32) {
    if s.get_clip().collidepoint((x, y)) {
        s.set_at_ungeprueft((x, y), farbe);
        b.punkt(x, y);
    }
}

/// pygames `drawhorzlineclipbounding` mit rohem Pixelwert.
///
/// Wie beim Nachbarn in `line.rs`: ist `x2 < x1`, werden die Enden
/// **vertauscht** statt den Aufruf zu verwerfen.
fn hlinie_roh(s: &mut Surface, b: &mut Bereich, farbe: u32, x1: i32, y: i32, x2: i32) {
    if let Some((a, e)) = spanne(s.get_clip(), x1, y, x2) {
        b.strecke(a, y, e, y);
        for x in a..=e {
            s.set_at_ungeprueft((x, y), farbe);
        }
    }
}

/// pygames `drawvertlineclipbounding` mit rohem Pixelwert.
fn vlinie_roh(s: &mut Surface, b: &mut Bereich, farbe: u32, y1: i32, x: i32, y2: i32) {
    let clip = s.get_clip();
    if x < clip.x || x >= clip.x + clip.w {
        return;
    }
    let (mut a, mut e) = if y2 < y1 { (y2, y1) } else { (y1, y2) };
    a = a.max(clip.y);
    e = e.min(clip.y + clip.h - 1);
    if e < clip.y || a >= clip.y + clip.h {
        return;
    }
    b.strecke(x, a, x, e);
    for y in a..=e {
        s.set_at_ungeprueft((x, y), farbe);
    }
}

/// `(int)f` aus C: zur Null hin abschneiden.
#[inline]
fn ganz(f: f32) -> i32 {
    f as i32
}

// ---------------------------------------------------------------------------
//  draw_aaline - Xiaolin Wu, Zeile fuer Zeile aus draw.c
// ---------------------------------------------------------------------------

/// pygames `draw_aaline`.
///
/// `ohne_erstes_ende` / `ohne_zweites_ende` lassen die gesonderte
/// Endpunktbehandlung aus (`aalines` schaltet sie an den Knicken ab, damit
/// die Naht nicht doppelt gemischt wird), `extra_pixel` holt sie trotzdem
/// zurueck - das braucht `aalines` dort, wo eine steile auf eine flache
/// Teilstrecke trifft.
#[allow(clippy::too_many_arguments)]
pub(crate) fn aa_strecke(
    s: &mut Surface,
    b: &mut Bereich,
    farbe: u32,
    von_x: f32,
    von_y: f32,
    nach_x: f32,
    nach_y: f32,
    ohne_erstes_ende: bool,
    ohne_zweites_ende: bool,
    extra_pixel: bool,
) {
    let (mut von_x, mut von_y, mut nach_x, mut nach_y) = (von_x, von_y, nach_x, nach_y);
    let mut dx = nach_x - von_x;
    let mut dy = nach_y - von_y;

    // Eine Strecke der Laenge null wird ein **voll** deckendes Pixel.
    // `floor(f + 0.5)` ist hier echtes Abrunden, nicht `(int)`.
    if (dx as f64).abs() < 0.0001 && (dy as f64).abs() < 0.0001 {
        let x = (von_x + 0.5).floor() as i32;
        let y = (von_y + 0.5).floor() as i32;
        let p = aa_farbe(s, x, y, farbe, 1.0);
        setze_roh(s, b, x, y, p);
        return;
    }

    // "To draw correctly the pixels at the border of the clipping area when
    //  the line crosses it, we need to clip it one pixel wider in all four
    //  directions" - der Kommentar steht so im Original.
    let clip = s.get_clip();
    let mut clip_links = clip.x as f32 - 1.0;
    let mut clip_rechts = clip_links + clip.w as f32 + 1.0;
    let mut clip_oben = clip.y as f32 - 1.0;
    let mut clip_unten = clip_oben + clip.h as f32 + 1.0;

    // Steile Linien werden an der Hauptdiagonale gespiegelt gerechnet und
    // beim Schreiben zurueckgetauscht. Das Clip-Rechteck muss mit.
    let steil = (dx as f64).abs() < (dy as f64).abs();
    if steil {
        std::mem::swap(&mut von_x, &mut von_y);
        std::mem::swap(&mut nach_x, &mut nach_y);
        std::mem::swap(&mut dx, &mut dy);
        std::mem::swap(&mut clip_links, &mut clip_oben);
        std::mem::swap(&mut clip_rechts, &mut clip_unten);
    }
    if dx < 0.0 {
        std::mem::swap(&mut von_x, &mut nach_x);
        std::mem::swap(&mut von_y, &mut nach_y);
        dx = -dx;
        dy = -dy;
    }

    if nach_x <= clip_links || von_x >= clip_rechts {
        return;
    }

    // Durch null kann hier nicht geteilt werden: war `dx` null, war es
    // entweder ein Punkt (oben abgefangen) oder es wurde mit `dy` getauscht.
    let gradient = dy / dx;

    if von_x < clip_links {
        von_y += gradient * (clip_links - von_x);
        von_x = clip_links;
    }
    if nach_x > clip_rechts {
        nach_y += gradient * (clip_rechts - nach_x);
        nach_x = clip_rechts;
    }

    if gradient > 0.0 {
        // `von_` ist das obere Ende.
        if nach_y <= clip_oben || von_y >= clip_unten {
            return;
        }
        if von_y < clip_oben {
            von_x += (clip_oben - von_y) / gradient;
            von_y = clip_oben;
        }
        if nach_y > clip_unten {
            nach_x += (clip_unten - nach_y) / gradient;
            nach_y = clip_unten;
        }
    } else {
        // `nach_` ist das obere Ende.
        if von_y <= clip_oben || nach_y >= clip_unten {
            return;
        }
        if nach_y < clip_oben {
            nach_x += (clip_oben - nach_y) / gradient;
            nach_y = clip_oben;
        }
        if von_y > clip_unten {
            von_x += (clip_unten - von_y) / gradient;
            von_y = clip_unten;
        }
    }

    // Der Trick des Originals: beide Enden um ein Pixel nach unten schieben,
    // damit `y` nie negativ wird. Dann tut `(int)y` dasselbe wie `floor(y)`,
    // und geschrieben wird eine Zeile hoeher.
    von_y += 1.0;
    nach_y += 1.0;

    // Beide werden unten gesetzt: entweder vom Endpunktblock oder von der
    // Abschaltung danach. Im Original stehen sie hier uninitialisiert - das
    // ist dort ungefaehrlich, weil sich die beiden Zweige ergaenzen.
    let mut x_start = 0i32;
    let mut schnitt_y = 0.0f32;

    // --- erster Endpunkt ---------------------------------------------------
    // Die Linie ist keine mathematische Linie der Dicke null; dasselbe gilt
    // fuer ihre Enden. Sie sind ein Pixel hoch und ein Pixel breit.
    if !ohne_erstes_ende || extra_pixel {
        x_start = ganz(von_x);
        let y_ende = von_y + gradient * (x_start as f32 - von_x);
        schnitt_y = y_ende;
        if nach_x > clip_links + 1.0 {
            let x_lueck = 1.0 + x_start as f32 - von_x;
            let mut helligkeit = y_ende - ganz(y_ende) as f32;
            let (mut px, mut py) = if steil {
                (ganz(y_ende), x_start)
            } else {
                (x_start, ganz(y_ende))
            };
            if (ganz(y_ende) as f32) < y_ende {
                let p = aa_farbe(s, px, py, farbe, helligkeit * x_lueck);
                setze_roh(s, b, px, py, p);
            }
            if steil {
                px -= 1;
            } else {
                py -= 1;
            }
            helligkeit = 1.0 - helligkeit;
            let p = aa_farbe(s, px, py, farbe, helligkeit * x_lueck);
            setze_roh(s, b, px, py, p);
            schnitt_y += gradient;
            x_start += 1;
        }
    }
    // Damit die Hauptschleife den ersten Endpunkt sicher ueberspringt.
    if ohne_erstes_ende {
        x_start = von_x.ceil() as i32;
        schnitt_y = von_y + gradient * (x_start as f32 - von_x);
    }

    // --- zweiter Endpunkt --------------------------------------------------
    let x_ende = nach_x.ceil() as i32;
    if (!ohne_zweites_ende || extra_pixel) && von_x < clip_rechts - 1.0 {
        let y_ende = nach_y + gradient * (x_ende as f32 - nach_x);
        let x_lueck = 1.0 - x_ende as f32 + nach_x;
        let mut helligkeit = y_ende - ganz(y_ende) as f32;
        let (mut px, mut py) = if steil {
            (ganz(y_ende), x_ende)
        } else {
            (x_ende, ganz(y_ende))
        };
        if (ganz(y_ende) as f32) < y_ende {
            let p = aa_farbe(s, px, py, farbe, helligkeit * x_lueck);
            setze_roh(s, b, px, py, p);
        }
        if steil {
            px -= 1;
        } else {
            py -= 1;
        }
        helligkeit = 1.0 - helligkeit;
        let p = aa_farbe(s, px, py, farbe, helligkeit * x_lueck);
        setze_roh(s, b, px, py, p);
    }

    // --- Hauptschleife -----------------------------------------------------
    let mut x = x_start;
    while x < x_ende {
        let y = ganz(schnitt_y);
        let mut helligkeit = 1.0 - schnitt_y + y as f32;
        let (ax, ay) = if steil { (y - 1, x) } else { (x, y - 1) };
        let p = aa_farbe(s, ax, ay, farbe, helligkeit);
        setze_roh(s, b, ax, ay, p);
        if (y as f32) < schnitt_y {
            helligkeit = 1.0 - helligkeit;
            let (bx, by) = if steil { (y, x) } else { (x, y) };
            let p = aa_farbe(s, bx, by, farbe, helligkeit);
            setze_roh(s, b, bx, by, p);
        }
        schnitt_y += gradient;
        x += 1;
    }
}

// ---------------------------------------------------------------------------
//  draw_aaline_width - die dicke Fassung
// ---------------------------------------------------------------------------

/// pygames `draw_aaline_width` (nur fuer `width > 1`).
///
/// Aufbau wie bei `line`: ein geglaetteter Rand oben und unten, dazwischen
/// eine **achsenparallele** Spanne quer zur laengeren Achse. Bei gerader
/// Staerke sitzt die Linie nicht mittig (`extra` im Original
/// `extra_width`).
#[allow(clippy::too_many_arguments)]
fn aa_strecke_breit(
    s: &mut Surface,
    b: &mut Bereich,
    farbe: u32,
    von_x: f32,
    von_y: f32,
    nach_x: f32,
    nach_y: f32,
    breite: i32,
) {
    let (mut von_x, mut von_y, mut nach_x, mut nach_y) = (von_x, von_y, nach_x, nach_y);
    let extra = 1 - (breite % 2);
    let halbe = breite / 2;

    let mut dx = nach_x - von_x;
    let mut dy = nach_y - von_y;
    let steil = (dx as f64).abs() < (dy as f64).abs();

    // Einzelner Punkt. Die Abfrage `dx != 0 && dy != 0` danach ist **nicht**
    // ueberfluessig: sind beide Enden exakt gleich, bleibt es bei einem
    // einzigen Pixel - eine dicke `aaline` der Laenge null ist ein Punkt,
    // eine dicke `line` der Laenge null dagegen eine Spanne.
    if (dx as f64).abs() < 0.0001 && (dy as f64).abs() < 0.0001 {
        let x = (von_x + 0.5).floor() as i32;
        let y = (von_y + 0.5).floor() as i32;
        let p = aa_farbe(s, x, y, farbe, 1.0);
        setze_roh(s, b, x, y, p);
        if dx != 0.0 && dy != 0.0 {
            if steil {
                hlinie_roh(s, b, farbe, x - halbe + extra, y, x + halbe - 1);
            } else {
                vlinie_roh(s, b, farbe, y - halbe + extra, x, y + halbe - 1);
            }
        }
        return;
    }

    let clip = s.get_clip();
    let mut clip_links = clip.x as f32 - 1.0;
    let mut clip_rechts = clip_links + clip.w as f32 + 1.0;
    let mut clip_oben = clip.y as f32 - 1.0;
    let mut clip_unten = clip_oben + clip.h as f32 + 1.0;

    if steil {
        std::mem::swap(&mut von_x, &mut von_y);
        std::mem::swap(&mut nach_x, &mut nach_y);
        std::mem::swap(&mut dx, &mut dy);
        std::mem::swap(&mut clip_links, &mut clip_oben);
        std::mem::swap(&mut clip_rechts, &mut clip_unten);
    }
    if dx < 0.0 {
        std::mem::swap(&mut von_x, &mut nach_x);
        std::mem::swap(&mut von_y, &mut nach_y);
        dx = -dx;
        dy = -dy;
    }

    if nach_x <= clip_links || von_x >= clip_rechts {
        return;
    }

    let gradient = dy / dx;

    if von_x < clip_links + 1.0 {
        von_y += gradient * (clip_links + 1.0 - von_x);
        von_x = clip_links + 1.0;
    }
    if nach_x > clip_rechts - 1.0 {
        nach_y += gradient * (clip_rechts - 1.0 - nach_x);
        nach_x = clip_rechts - 1.0;
    }

    // **Toter Code im Original.** Die beiden Bedingungen wurden drei Zeilen
    // darueber gerade unmoeglich gemacht: `von_x` steht jetzt exakt auf
    // `clip_links + 1`, `nach_x` exakt auf `clip_rechts - 1`. Die senkrechte
    // Beschneidung der dicken `aaline` laeuft deshalb **nie**. Bleibt hier
    // stehen, weil ein "reparierter" Nachbau andere Pixel ergaebe.
    if gradient > 0.0 {
        if von_x < clip_links + 1.0 {
            if nach_y <= clip_oben || von_y >= clip_unten {
                return;
            }
            if von_y < clip_oben - halbe as f32 {
                von_x += (clip_oben - halbe as f32 - von_y) / gradient;
                von_y = clip_oben - halbe as f32;
            }
            if nach_y > clip_unten + halbe as f32 {
                nach_x += (clip_unten + halbe as f32 - nach_y) / gradient;
                nach_y = clip_unten + halbe as f32;
            }
        }
    } else if nach_x > clip_rechts - 1.0 {
        if von_y <= clip_oben || nach_y >= clip_unten {
            return;
        }
        if nach_y < clip_oben - halbe as f32 {
            nach_x += (clip_oben - halbe as f32 - nach_y) / gradient;
            nach_y = clip_oben - halbe as f32;
        }
        if von_y > clip_unten + halbe as f32 {
            von_x += (clip_unten + halbe as f32 - von_y) / gradient;
            von_y = clip_unten + halbe as f32;
        }
    }

    von_y += 1.0;
    nach_y += 1.0;

    // --- erster Endpunkt ---------------------------------------------------
    let mut x_start = ganz(von_x);
    let y_ende_1 = von_y + gradient * (x_start as f32 - von_x);
    let mut schnitt_y = y_ende_1;
    if nach_x > clip_links + 1.0 {
        let mut helligkeit = y_ende_1 - ganz(y_ende_1) as f32;
        let (px, py) = if steil {
            (ganz(y_ende_1), x_start)
        } else {
            (x_start, ganz(y_ende_1))
        };
        if (ganz(y_ende_1) as f32) < y_ende_1 {
            if steil {
                let p = aa_farbe(s, px + halbe, py, farbe, helligkeit);
                setze_roh(s, b, px + halbe, py, p);
            } else {
                let p = aa_farbe(s, px, py + halbe, farbe, helligkeit);
                setze_roh(s, b, px, py + halbe, p);
            }
        }
        helligkeit = 1.0 - helligkeit;
        if steil {
            // **Kein Tippfehler von mir, sondern einer im Original:** die
            // Mischfarbe wird bei `px - halbe` geholt, geschrieben wird sie
            // bei `px - halbe + extra - 1`. Bei gerader Staerke sind das
            // zwei verschiedene Pixel, und die Mischung passt dann nicht
            // zum Untergrund, auf den sie landet.
            let p = aa_farbe(s, px - halbe, py, farbe, helligkeit);
            setze_roh(s, b, px - halbe + extra - 1, py, p);
            hlinie_roh(s, b, farbe, px - halbe + extra, py, px + halbe - 1);
        } else {
            let p = aa_farbe(s, px, py - halbe + extra - 1, farbe, helligkeit);
            setze_roh(s, b, px, py - halbe + extra - 1, p);
            vlinie_roh(s, b, farbe, py - halbe + extra, px, py + halbe - 1);
        }
        schnitt_y += gradient;
        x_start += 1;
    }

    // --- zweiter Endpunkt --------------------------------------------------
    let x_ende = nach_x.ceil() as i32;
    if von_x < clip_rechts - 1.0 {
        let y_ende_2 = nach_y + gradient * (x_ende as f32 - nach_x);
        let mut helligkeit = y_ende_2 - ganz(y_ende_2) as f32;
        let (px, py) = if steil {
            (ganz(y_ende_2), x_ende)
        } else {
            (x_ende, ganz(y_ende_2))
        };
        if (ganz(y_ende_2) as f32) < y_ende_2 {
            if steil {
                let p = aa_farbe(s, px + halbe, py, farbe, helligkeit);
                setze_roh(s, b, px + halbe, py, p);
            } else {
                let p = aa_farbe(s, px, py + halbe, farbe, helligkeit);
                setze_roh(s, b, px, py + halbe, p);
            }
        }
        helligkeit = 1.0 - helligkeit;
        if steil {
            let p = aa_farbe(s, px - halbe + extra - 1, py, farbe, helligkeit);
            setze_roh(s, b, px - halbe + extra - 1, py, p);
            // Hier faengt die Spanne bei `px - halbe` an, beim ersten
            // Endpunkt dagegen bei `px - halbe + extra`. Auch das ist im
            // Original so und macht die beiden Enden unsymmetrisch.
            hlinie_roh(s, b, farbe, px - halbe, py, px + halbe - 1);
        } else {
            let p = aa_farbe(s, px, py - halbe + extra - 1, farbe, helligkeit);
            setze_roh(s, b, px, py - halbe + extra - 1, p);
            vlinie_roh(s, b, farbe, py - halbe + extra, px, py + halbe - 1);
        }
    }

    // --- Hauptschleife -----------------------------------------------------
    let mut x = x_start;
    while x < x_ende {
        let y = ganz(schnitt_y);
        let mut helligkeit = 1.0 - schnitt_y + y as f32;
        if steil {
            let p = aa_farbe(s, y - halbe + extra - 1, x, farbe, helligkeit);
            setze_roh(s, b, y - halbe + extra - 1, x, p);
            if (y as f32) < schnitt_y {
                helligkeit = 1.0 - helligkeit;
                let p = aa_farbe(s, y + halbe, x, farbe, helligkeit);
                setze_roh(s, b, y + halbe, x, p);
            }
            hlinie_roh(s, b, farbe, y - halbe + extra, x, y + halbe - 1);
        } else {
            let p = aa_farbe(s, x, y - halbe + extra - 1, farbe, helligkeit);
            setze_roh(s, b, x, y - halbe + extra - 1, p);
            if (y as f32) < schnitt_y {
                helligkeit = 1.0 - helligkeit;
                let p = aa_farbe(s, x, y + halbe, farbe, helligkeit);
                setze_roh(s, b, x, y + halbe, p);
            }
            vlinie_roh(s, b, farbe, y - halbe + extra, x, y + halbe - 1);
        }
        schnitt_y += gradient;
        x += 1;
    }
}

// ---------------------------------------------------------------------------
//  Die oeffentlichen Funktionen
// ---------------------------------------------------------------------------

/// Zeichnet eine kantengeglaettete Strecke; gibt das beschriebene Rechteck
/// zurueck.
///
/// Anders als bei [`line`](fn@crate::draw::line) zeichnet `width < 1` hier
/// **trotzdem**: pygame behandelt jedes `width <= 1` als `1`. Gemessen an
/// echtem pygame-ce 2.5.7 - `width = 0` und `width = -3` liefern Pixel fuer
/// Pixel dasselbe Bild wie `width = 1`.
///
/// Wurde gar nichts geschrieben, kommt `Rect((int)start.0, (int)start.1, 0, 0)`
/// zurueck - mit Abschneiden zur Null hin, nicht mit Abrunden.
pub fn aaline(
    s: &mut Surface,
    c: impl IntoColor,
    start: (f32, f32),
    end: (f32, f32),
    width: i32,
) -> Rect {
    let farbe = s.abbilden(c);
    let mut b = Bereich::leer();
    if width > 1 {
        aa_strecke_breit(s, &mut b, farbe, start.0, start.1, end.0, end.1, width);
    } else {
        aa_strecke(
            s, &mut b, farbe, start.0, start.1, end.0, end.1, false, false, false,
        );
    }
    b.zu_rect()
        .unwrap_or(Rect::new(ganz(start.0), ganz(start.1), 0, 0))
}

/// Ob eine Teilstrecke "steil" ist - `fabs(dx) < fabs(dy)`.
#[inline]
fn steil_von(x1: f32, y1: f32, x2: f32, y2: f32) -> bool {
    ((x2 - x1) as f64).abs() < ((y2 - y1) as f64).abs()
}

/// Ob beide Enden auf ganzen Zahlen liegen. Nur dann behandelt `aalines`
/// die Nahtstelle gesondert (`roundf(v) == v` im Original).
#[inline]
fn enden_abschalten(x: f32, y: f32) -> bool {
    !(x.round() == x && y.round() == y)
}

/// Kantengeglaetteter Linienzug durch alle Punkte; `closed` haengt den
/// Rueckweg an.
///
/// Drei Eigenheiten, die aus `draw.c` stammen und hier absichtlich
/// mitwandern:
///
/// * **Bei genau zwei Punkten wird die Strecke zweimal gezeichnet.** Der
///   Block fuer die "erste Linie" und der fuer die "letzte Linie" meinen
///   dieselbe Strecke. Bei gemischten Pixeln ist das sichtbar - das zweite
///   Mal mischt auf das Ergebnis des ersten und macht die Linie dichter.
/// * **Fuer die erste Linie vergleicht das Original `ylist[2]` mit
///   `pts[1]`** statt mit `pts[3]`, benutzt also den *Anfangs*punkt, wo der
///   Endpunkt gemeint war.
/// * **Bei genau zwei Punkten liest es dabei ueber das Feldende hinaus.**
///   Das ist nicht nur in C unbestimmt, sondern auch in der Messung: ein
///   und derselbe Aufruf liefert je nach Python-Speicherlage zwei
///   verschiedene Bilder. pygame ist dort also nicht mit sich selbst
///   reproduzierbar; hier faellt die Entscheidung fest auf "kein
///   Extrapixel" - Einzelheiten am Fundort im Quelltext unten.
pub fn aalines(s: &mut Surface, c: impl IntoColor, closed: bool, pts: &[(f32, f32)]) -> Rect {
    let Some(&(x0, y0)) = pts.first() else {
        return Rect::new(0, 0, 0, 0);
    };
    let n = pts.len();
    if n < 2 {
        return Rect::new(ganz(x0), ganz(y0), 0, 0);
    }
    let farbe = s.abbilden(c);
    let mut b = Bereich::leer();

    // --- erste Linie -------------------------------------------------------
    let mut vorher = (pts[0], pts[1]);
    let steil_vorher = steil_von(pts[0].0, pts[0].1, pts[1].0, pts[1].1);
    // `extra_px = steep_prev > steep_curr` - beim ersten Teilstueck ein
    // echtes Groesser, ab dem zweiten ein Ungleich. Und `steep_curr`
    // vergleicht `xlist[2]` mit `pts[2]`, aber `ylist[2]` mit **`pts[1]`**.
    let mut extra = if n > 2 {
        // `steep_curr = fabs(xlist[2] - pts[2]) < fabs(ylist[2] - pts[1])`.
        // `pts[2]` ist `xlist[1]`, `pts[1]` aber `ylist[0]` - der
        // *Anfangs*punkt, wo der Endpunkt gemeint war. Der Tippfehler
        // bleibt.
        let steil_jetzt =
            ((pts[2].0 - pts[1].0) as f64).abs() < ((pts[2].1 - pts[0].1) as f64).abs();
        steil_vorher && !steil_jetzt
    } else {
        // **Hier liest pygame ueber das Feldende hinaus.** `xlist` und
        // `ylist` liegen in *einem* Block (`xlist = buf`,
        // `ylist = buf + n`); bei `n == 2` ist `ylist[2]` der Eintrag
        // `buf[4]` - einer hinter dem Block. In C unbestimmt, und
        // **gemessen auch wirklich unbestimmt**: derselbe Aufruf
        //
        //   aalines(s, farbe, False, [(19.8082447, 9.3954296),
        //                             (20.0210876, 5.5279446)])
        //
        // liefert an (20,10) je nach Python-Speicherlage 74 **oder** 125.
        // pygame ist hier also nicht einmal mit sich selbst
        // reproduzierbar. Gewaehlt ist der weitaus haeufigere Ausgang
        // "kein Extrapixel": in einem Abtastlauf ueber 32.040 Faelle
        // (20.758 davon unterscheidbar) war er 20.758-mal richtig und
        // 0-mal falsch; in 2.200 zufaelligen Zwei-Punkt-Faellen 2.188-mal
        // richtig und 12-mal falsch. Er ist zugleich der einzige, der zur
        // Absicht passt - zwei Punkte haben keinen Knick.
        false
    };
    let mut abschalten = enden_abschalten(pts[1].0, pts[1].1);
    aa_strecke(
        s,
        &mut b,
        farbe,
        pts[0].0,
        pts[0].1,
        pts[1].0,
        pts[1].1,
        closed && abschalten,
        abschalten,
        extra,
    );

    // --- mittlere Linien ---------------------------------------------------
    for i in 2..n.saturating_sub(1) {
        let (a, e) = (pts[i - 1], pts[i]);
        let sv = steil_von(vorher.0 .0, vorher.0 .1, vorher.1 .0, vorher.1 .1);
        let sj = steil_von(a.0, a.1, e.0, e.1);
        extra = sv != sj;
        abschalten = enden_abschalten(e.0, e.1);
        vorher = (a, e);
        aa_strecke(
            s, &mut b, farbe, a.0, a.1, e.0, e.1, abschalten, abschalten, extra,
        );
    }

    // --- letzte Linie ------------------------------------------------------
    let (a, e) = (pts[n - 2], pts[n - 1]);
    let sv = steil_von(vorher.0 .0, vorher.0 .1, vorher.1 .0, vorher.1 .1);
    let sj = steil_von(a.0, a.1, e.0, e.1);
    extra = sv != sj;
    abschalten = enden_abschalten(e.0, e.1);
    vorher = (a, e);
    // Offen: der **letzte** Endpunkt wird gezeichnet. Geschlossen: er
    // faellt weg, weil der Rueckweg dort ansetzt.
    let zweites_ende_aus = closed && abschalten;
    aa_strecke(
        s,
        &mut b,
        farbe,
        a.0,
        a.1,
        e.0,
        e.1,
        abschalten,
        zweites_ende_aus,
        extra,
    );

    // --- Rueckweg ----------------------------------------------------------
    if closed && n > 2 {
        let (a, e) = (pts[n - 1], pts[0]);
        let sv = steil_von(vorher.0 .0, vorher.0 .1, vorher.1 .0, vorher.1 .1);
        let sj = steil_von(a.0, a.1, e.0, e.1);
        extra = sv != sj;
        abschalten = enden_abschalten(e.0, e.1);
        aa_strecke(
            s, &mut b, farbe, a.0, a.1, e.0, e.1, abschalten, abschalten, extra,
        );
    }

    b.zu_rect().unwrap_or(Rect::new(ganz(x0), ganz(y0), 0, 0))
}

// ---------------------------------------------------------------------------
//  Tests
// ---------------------------------------------------------------------------

/// Hilfen fuer die geglaetteten Zeichenfunktionen.
///
/// Anders als beim harten Zeichnen genuegt hier kein Schwarz-Weiss-Bild -
/// es kommt gerade auf die Zwischenwerte an. Darum eine Hexziffer je
/// Pixel: die **obere Haelfte** des gewaehlten Kanals. Das ist genau die
/// Darstellung, in der die Messungen gegen echtes pygame abgelegt sind.
#[cfg(test)]
pub(crate) mod pruefhilfe {
    use crate::surface::Surface;

    /// Kanal 0 = Rot, 1 = Gruen, 2 = Blau, 3 = Alpha.
    pub(crate) fn hexbild(s: &Surface, kanal: usize) -> Vec<String> {
        let (w, h) = s.get_size();
        (0..h as i32)
            .map(|y| {
                (0..w as i32)
                    .map(|x| {
                        let c = s.get_at((x, y));
                        let v = [c.r, c.g, c.b, c.a][kanal];
                        char::from_digit((v >> 4) as u32, 16)
                            .unwrap()
                            .to_ascii_uppercase()
                    })
                    .collect()
            })
            .collect()
    }

    /// Vergleicht mit einem an echtem pygame gemessenen Hexbild.
    pub(crate) fn gleich(s: &Surface, kanal: usize, soll: &[&str]) {
        let ist = hexbild(s, kanal);
        assert_eq!(
            ist,
            soll.iter().map(|z| z.to_string()).collect::<Vec<_>>(),
            "\nist:\n  {}\nsoll:\n  {}",
            ist.join("\n  "),
            soll.join("\n  ")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::pruefhilfe::gleich;
    use super::*;
    use crate::draw::circle::pruefhilfe::schwarz;
    use crate::surface::SRCALPHA;

    const ROT: (u8, u8, u8, u8) = (255, 0, 0, 255);

    /// Die Messung aus dem Auftrag, Pixel fuer Pixel.
    #[test]
    fn die_gemessene_schraege_linie() {
        let mut s = schwarz(12, 8);
        let r = aaline(&mut s, ROT, (1.0, 1.0), (10.0, 6.0), 1);
        assert_eq!(r, Rect::new(1, 1, 10, 6));
        gleich(
            &s,
            0,
            &[
                "000000000000",
                "0F7000000000",
                "008E50000000",
                "0001AC300000",
                "000003CA1000",
                "00000005E800",
                "0000000007F0",
                "000000000000",
            ],
        );
    }

    /// Die dicke Fassung geht einen ganz anderen Weg (`draw_aaline_width`):
    /// geglaettete Raender, dazwischen harte Spannen.
    #[test]
    fn die_dicke_linie_ist_innen_hart() {
        let mut s = schwarz(14, 10);
        let r = aaline(&mut s, ROT, (1.0, 1.0), (12.0, 8.0), 4);
        assert_eq!(r, Rect::new(1, 0, 12, 10));
        gleich(
            &s,
            0,
            &[
                "0F500000000000",
                "0FFB1000000000",
                "0FFFF700000000",
                "0FFFFFD2000000",
                "00AFFFFF800000",
                "0004EFFFFE4000",
                "000008FFFFFA00",
                "0000002DFFFFF0",
                "000000007FFFF0",
                "0000000001BFF0",
            ],
        );
    }

    /// **Die Mischformel.** Halbe Helligkeit auf Weiss ergibt 127, nicht
    /// 128: `(Uint8)(0.5 * 0 + 0.5 * 255)` schneidet ab.
    #[test]
    fn gemischt_wird_abgeschnitten_nicht_gerundet() {
        let mut s = Surface::with_flags((8, 4), 0);
        s.fill((255u8, 255u8, 255u8));
        aaline(&mut s, (0u8, 0u8, 0u8), (0.0, 0.5), (7.0, 0.5), 1);
        for x in 0..8 {
            assert_eq!(s.get_at((x, 0)).r, 127, "Zeile 0, x={x}");
            assert_eq!(s.get_at((x, 1)).r, 127, "Zeile 1, x={x}");
        }
    }

    /// **Der Alphakanal mischt mit.** Auf einer durchsichtigen Flaeche
    /// wandert er genauso wie Rot - und die Farbe darf selbst Alpha haben.
    #[test]
    fn das_alpha_mischt_mit() {
        let mut s = Surface::with_flags((8, 4), SRCALPHA);
        s.fill((0u8, 0u8, 0u8, 0u8));
        aaline(&mut s, ROT, (0.0, 0.5), (7.0, 0.5), 1);
        let c = s.get_at((1, 0));
        assert_eq!((c.r, c.g, c.b, c.a), (127, 0, 0, 127));

        let mut s = Surface::with_flags((8, 4), SRCALPHA);
        s.fill((0u8, 0u8, 0u8, 0u8));
        aaline(&mut s, (255u8, 0u8, 0u8, 128u8), (0.0, 0.5), (7.0, 0.5), 1);
        let c = s.get_at((1, 0));
        assert_eq!(
            (c.r, c.g, c.b, c.a),
            (127, 0, 0, 64),
            "0.5 * 128 abgeschnitten"
        );
    }

    /// **Der Unterschied zu `line`:** dort zeichnet `width < 1` nichts,
    /// hier wird jede Staerke unter 2 wie 1 behandelt.
    #[test]
    fn breite_unter_eins_zeichnet_trotzdem() {
        let mut vergleich = schwarz(12, 8);
        let r0 = aaline(&mut vergleich, ROT, (1.0, 1.0), (10.0, 6.0), 1);
        for breite in [-7, -1, 0] {
            let mut s = schwarz(12, 8);
            let r = aaline(&mut s, ROT, (1.0, 1.0), (10.0, 6.0), breite);
            assert_eq!(r, r0, "Breite {breite}");
            for y in 0..8 {
                for x in 0..12 {
                    assert_eq!(
                        s.get_at_mapped((x, y)),
                        vergleich.get_at_mapped((x, y)),
                        "Breite {breite} an ({x},{y})"
                    );
                }
            }
        }
    }

    /// Eine Strecke der Laenge null wird ein voll deckendes Pixel; die
    /// Stelle wird **abgerundet** (`floor(v + 0.5)`), nicht abgeschnitten.
    #[test]
    fn ein_punkt_deckt_voll() {
        let mut s = schwarz(8, 8);
        let r = aaline(&mut s, ROT, (3.4, 3.6), (3.4, 3.6), 1);
        assert_eq!(r, Rect::new(3, 4, 1, 1));
        let c = s.get_at((3, 4));
        assert_eq!((c.r, c.g, c.b, c.a), (255, 0, 0, 255));
    }

    /// Wurde nichts geschrieben, meldet pygame `Rect(start, 0, 0)` - mit
    /// `(int)`, also zur Null hin abgeschnitten. `-30.7` wird `-30`.
    #[test]
    fn nichts_gezeichnet_meldet_den_startpunkt() {
        let mut s = schwarz(8, 8);
        let r = aaline(&mut s, ROT, (-30.7, -30.2), (-20.0, -25.0), 1);
        assert_eq!(r, Rect::new(-30, -30, 0, 0));
    }

    /// Ein geschlossener Linienzug; die Naht wird nicht doppelt gemischt,
    /// weil `aalines` an den Knicken die Endpunkte abschaltet.
    #[test]
    fn der_geschlossene_linienzug_stimmt() {
        let mut s = schwarz(14, 12);
        let r = aalines(&mut s, ROT, true, &[(2.0, 2.0), (11.0, 4.0), (6.0, 10.0)]);
        assert_eq!(r, Rect::new(2, 2, 10, 9));
        gleich(
            &s,
            0,
            &[
                "00000000000000",
                "00000000000000",
                "00FC8510000000",
                "00797AEEA73000",
                "000F000158CF00",
                "0007700000D200",
                "0000F0000A5000",
                "00007700770000",
                "00000F05A00000",
                "0000079D000000",
                "000000F0000000",
                "00000000000000",
            ],
        );
    }

    /// **Zwei Punkte sind nicht dasselbe wie `aaline`.** pygame zeichnet
    /// die Strecke zweimal - einmal als "erste", einmal als "letzte"
    /// Linie -, und das zweite Mal mischt auf das erste. Gemessen.
    #[test]
    fn zwei_punkte_werden_doppelt_gezeichnet() {
        let mut s = schwarz(12, 8);
        let r = aalines(&mut s, ROT, false, &[(1.0, 1.0), (10.0, 6.0)]);
        assert_eq!(r, Rect::new(1, 1, 10, 6));
        gleich(
            &s,
            0,
            &[
                "000000000000",
                "0FB000000000",
                "00CF80000000",
                "0003EF600000",
                "000006FE3000",
                "00000008FC00",
                "000000000BF0",
                "000000000000",
            ],
        );

        // Zum Vergleich dieselbe Strecke mit `aaline` - deutlich blasser.
        let mut einzeln = schwarz(12, 8);
        aaline(&mut einzeln, ROT, (1.0, 1.0), (10.0, 6.0), 1);
        assert_ne!(einzeln.get_at((2, 2)).r, s.get_at((2, 2)).r);
    }

    /// Das Clip-Rechteck aendert die Pixelwahl nicht, nur was geschrieben
    /// wird - und das Rueckgaberechteck.
    #[test]
    fn das_clip_rechteck_beschneidet_nur() {
        let mut ganz = schwarz(16, 12);
        aaline(&mut ganz, ROT, (0.0, 0.0), (15.0, 11.0), 1);

        let mut beschnitten = schwarz(16, 12);
        beschnitten.set_clip(Some(Rect::new(3, 2, 7, 6)));
        let r = aaline(&mut beschnitten, ROT, (0.0, 0.0), (15.0, 11.0), 1);
        beschnitten.set_clip(None);
        assert_eq!(r, Rect::new(3, 2, 7, 6));
        for y in 2..8 {
            for x in 3..10 {
                assert_eq!(
                    beschnitten.get_at_mapped((x, y)),
                    ganz.get_at_mapped((x, y)),
                    "({x},{y})"
                );
            }
        }
        assert_eq!(beschnitten.get_at((2, 2)).r, 0, "ausserhalb bleibt leer");
    }
}
