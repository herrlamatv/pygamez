//! aacircle.rs
//! ===========
//! `pygame.draw.aacircle` - der Kreis mit geglaetteter Kante.
//!
//! Verfahren ist **Xiaolin Wu** (`draw_circle_xiaolinwu` in `src_c/draw.c`,
//! angelehnt an <https://cgg.mff.cuni.cz/~pepca/ref/WU.pdf>), nicht
//! Bresenham wie beim harten [`circle`](fn@crate::draw::circle). Gemischt wird
//! mit derselben Formel wie bei [`aaline`](fn@crate::draw::aaline) - siehe dort.
//!
//! ## Der Kern in drei Zeilen
//!
//! ```c
//! double height = sqrt(pow_layer_r - pow(x, 2));
//! double opacity = 255.0 * (ceil(height) - height);
//! if (opacity < prev_opacity) { --y; }
//! ```
//!
//! Das `--y` haengt also an der **Deckung**, nicht an einer Fehlerschranke:
//! sinkt die Deckung gegenueber dem vorigen Schritt, ist die Kante um eine
//! Zeile gerutscht. Deshalb ist `y` keine Funktion von `x` allein, und
//! deshalb laesst sich der Kreis nicht Zeile fuer Zeile nachrechnen.
//!
//! ## Vier Wege fuer einen Aufruf
//!
//! `aacircle` entscheidet an `width` zwischen vier verschiedenen
//! Rechnungen - das ist keine Optimierung, die Ergebnisse unterscheiden
//! sich:
//!
//! | `width` | was passiert |
//! |---|---|
//! | `0` oder `= radius` | **gefuellte** Scheibe mit `radius - 1` (Bresenham, hart) **und** darueber ein Xiaolin-Wu-Ring der Dicke 2 |
//! | `1` | `draw_circle_xiaolinwu_thin` - eine einzige Spur |
//! | sonst | `draw_circle_xiaolinwu` mit `thickness = width` |
//! | `< 0` | zeichnet **nichts** |
//!
//! Die erste Zeile ist die ueberraschende: ein *gefuellter* `aacircle` ist
//! eine harte Scheibe plus ein zwei Pixel breiter weicher Rand, und der
//! Rand mischt dabei auf die gerade gefuellte Scheibe. Gemessen:
//! `aacircle((6,6), 4, 0)` hat innen genau die Farbe der Scheibe.
//!
//! Mit Quadrantenschaltern (`aacircle_quadranten`) faellt die Sonderbehandlung
//! fuer `width == 0` weg - dort wird statt der Scheibe ein Xiaolin-Wu-Ring
//! der Dicke `radius` gezeichnet.
//!
//! **"Mit Schaltern" heisst: mindestens einer ist gesetzt.** pygame fuehrt
//! die vier als `int` mit dem Vorbesetzten `0` und fragt
//! `top_right == 0 && top_left == 0 && ...`. `draw_top_right=False` ist
//! also *kein* Schalter im Sinne dieser Abfrage, und vier mal `True` ist
//! nicht der Normalfall, sondern der Sonderweg. Beides gemessen - siehe
//! [`Quadranten::alle`].

use crate::color::IntoColor;
use crate::draw::aaline::aa_farbe;
use crate::draw::line::{spanne, Bereich};
use crate::rect::Rect;
use crate::surface::Surface;

// ---------------------------------------------------------------------------
//  Bausteine
// ---------------------------------------------------------------------------

/// pygames `set_and_check_rect` mit fertigem Pixelwert.
fn setze_roh(s: &mut Surface, b: &mut Bereich, x: i32, y: i32, farbe: u32) {
    if s.get_clip().collidepoint((x, y)) {
        s.set_at_ungeprueft((x, y), farbe);
        b.punkt(x, y);
    }
}

/// pygames `drawhorzlineclipbounding` mit fertigem Pixelwert - vertauscht
/// die Enden, statt einen scheinbar leeren Aufruf zu verwerfen.
fn hlinie_roh(s: &mut Surface, b: &mut Bereich, farbe: u32, x1: i32, y: i32, x2: i32) {
    if let Some((a, e)) = spanne(s.get_clip(), x1, y, x2) {
        b.strecke(a, y, e, y);
        for x in a..=e {
            s.set_at_ungeprueft((x, y), farbe);
        }
    }
}

/// pygames `draw_circle_filled`.
///
/// Steht hier ein zweites Mal (das harte [`circle`](fn@crate::draw::circle)
/// hat dieselbe Schleife), weil `aacircle` sie mit `radius - 1` aufruft und
/// die dortige Fassung nicht oeffentlich ist. `xmax` faengt den Ueberlauf
/// ab, den ein weit ausserhalb liegender Mittelpunkt sonst ausloeste -
/// hier mit `i64` geloest statt mit pygames `INT_MAX`-Rechnung.
fn scheibe(s: &mut Surface, b: &mut Bereich, farbe: u32, x0: i32, y0: i32, radius: i32) {
    let (x0, y0) = (x0 as i64, y0 as i64);
    let mut f: i64 = 1 - radius as i64;
    let mut ddf_x: i64 = 0;
    let mut ddf_y: i64 = -2 * radius as i64;
    let mut x: i64 = 0;
    let mut y: i64 = radius as i64;

    while x < y {
        if f >= 0 {
            y -= 1;
            ddf_y += 2;
            f += ddf_y;
        }
        x += 1;
        ddf_x += 2;
        f += ddf_x + 1;

        // Nur zeichnen, wenn `y` gleich kleiner wird - sonst wuerde
        // dieselbe Zeile mehrfach beschrieben. Bei einer deckenden Farbe
        // faellt das nicht auf, hier schon.
        if f >= 0 {
            hlinie_roh(
                s,
                b,
                farbe,
                (x0 - x) as i32,
                (y0 + y - 1) as i32,
                (x0 + x - 1) as i32,
            );
            hlinie_roh(
                s,
                b,
                farbe,
                (x0 - x) as i32,
                (y0 - y) as i32,
                (x0 + x - 1) as i32,
            );
        }
        hlinie_roh(
            s,
            b,
            farbe,
            (x0 - y) as i32,
            (y0 + x - 1) as i32,
            (x0 + y - 1) as i32,
        );
        hlinie_roh(
            s,
            b,
            farbe,
            (x0 - y) as i32,
            (y0 - x) as i32,
            (x0 + y - 1) as i32,
        );
    }
}

/// Welche Viertel gezeichnet werden: oben rechts, oben links, unten links,
/// unten rechts - in genau dieser Reihenfolge, wie in pygames Signatur.
#[derive(Clone, Copy, Debug)]
pub struct Quadranten {
    /// Oben rechts zeichnen?
    pub top_right: bool,
    /// Oben links zeichnen?
    pub top_left: bool,
    /// Unten links zeichnen?
    pub bottom_left: bool,
    /// Unten rechts zeichnen?
    pub bottom_right: bool,
}

impl Quadranten {
    /// Alle vier **ausdruecklich** angefordert.
    ///
    /// Nicht dasselbe wie [`keine`](Self::keine)! In pygames Signatur sind
    /// die vier Schalter `int`-Beiwerte mit dem Vorbesetzten `0`, und die
    /// Abfrage lautet `top_right == 0 && top_left == 0 && ...`. Wer alle
    /// vier auf `True` setzt, landet deshalb in einem **anderen** Zweig als
    /// wer keinen setzt - gemessen an pygame-ce 2.5.7:
    ///
    /// ```text
    /// aacircle(s, rot, (6,6), 4, 0)                      -> Zeile 5: DE FF FF FE FF FE FF FF DE
    /// aacircle(s, rot, (6,6), 4, 0, True, True, True, True) -> Zeile 5: DE FF FF FF FF FF FF FF DE
    /// ```
    ///
    /// Die beiden `FE` sind der Unterschied: ohne Schalter fuellt pygame
    /// erst eine harte Scheibe mit `radius - 1` und mischt dann einen
    /// Xiaolin-Wu-Ring der Dicke 2 darueber; mit Schaltern zeichnet es
    /// einen Ring der Dicke `radius`, der innen voll deckt.
    pub fn alle() -> Quadranten {
        Quadranten {
            top_right: true,
            top_left: true,
            bottom_left: true,
            bottom_right: true,
        }
    }

    /// Kein Schalter gesetzt - pygames Vorbesetzung, also der **ganze**
    /// Kreis auf dem Sonderweg (siehe [`alle`](Self::alle)).
    pub fn keine() -> Quadranten {
        Quadranten {
            top_right: false,
            top_left: false,
            bottom_left: false,
            bottom_right: false,
        }
    }

    /// Ist kein einziger gesetzt, nimmt pygame einen anderen Weg - deshalb
    /// muss das abfragbar sein.
    pub(crate) fn keiner(&self) -> bool {
        !self.top_right && !self.top_left && !self.bottom_left && !self.bottom_right
    }
}

/// pygames `draw_eight_symetric_pixels`.
///
/// `deckung` kommt in der Skala **0..255** herein und wird gleich zu Beginn
/// durch 255 geteilt - genau wie im Original (`opacity = opacity / 255.0f`).
/// Diese Reihenfolge ist wichtig: der Aufrufer rechnet mit
/// `255.0f - (float)opacity`, wobei `opacity` ein `double` ist. Erst wird
/// also in `double` gerechnet, dann auf `float` gekuerzt, dann geteilt.
#[allow(clippy::too_many_arguments)]
fn acht_punkte(
    s: &mut Surface,
    b: &mut Bereich,
    farbe: u32,
    x0: i32,
    y0: i32,
    x: i32,
    y: i32,
    deckung: f32,
    q: Quadranten,
) {
    let deckung = deckung / 255.0;
    let male = |s: &mut Surface, b: &mut Bereich, px: i32, py: i32| {
        let p = aa_farbe(s, px, py, farbe, deckung);
        setze_roh(s, b, px, py, p);
    };
    if q.top_right {
        male(s, b, x0 + x, y0 - y);
        male(s, b, x0 + y, y0 - x);
    }
    if q.top_left {
        male(s, b, x0 - x, y0 - y);
        male(s, b, x0 - y, y0 - x);
    }
    if q.bottom_left {
        male(s, b, x0 - x, y0 + y);
        male(s, b, x0 - y, y0 + x);
    }
    if q.bottom_right {
        male(s, b, x0 + x, y0 + y);
        male(s, b, x0 + y, y0 + x);
    }
}

/// pygames `draw_circle_xiaolinwu` - ein Ring aus `thickness` Lagen.
///
/// Die innerste und die aeusserste Lage werden geglaettet, alle dazwischen
/// voll gedeckt. Beachte, welche Lage **welche** Seite glaettet: innen wird
/// die Spur `y` voll und `y - 1` mit `opacity` gezeichnet, aussen genau
/// andersherum.
#[allow(clippy::too_many_arguments)]
fn ring(
    s: &mut Surface,
    b: &mut Bereich,
    farbe: u32,
    x0: i32,
    y0: i32,
    radius: i32,
    dicke: i32,
    q: Quadranten,
) {
    for lage in (radius - dicke)..=radius {
        let mut x: i32 = 0;
        let mut y: i32 = lage;
        let lage_r2 = (lage as f64).powi(2);
        let mut vorige_deckung = 0.0f64;
        while x < y {
            let hoehe = (lage_r2 - (x as f64).powi(2)).sqrt();
            let deckung = 255.0 * (hoehe.ceil() - hoehe);
            // Sinkt die Deckung, ist die Kante eine Zeile weitergerutscht.
            if deckung < vorige_deckung {
                y -= 1;
            }
            vorige_deckung = deckung;
            if lage == radius - dicke {
                acht_punkte(s, b, farbe, x0, y0, x, y, 255.0, q);
                acht_punkte(s, b, farbe, x0, y0, x, y - 1, deckung as f32, q);
            } else if lage == radius {
                acht_punkte(s, b, farbe, x0, y0, x, y, 255.0 - deckung as f32, q);
                acht_punkte(s, b, farbe, x0, y0, x, y - 1, 255.0, q);
            } else {
                acht_punkte(s, b, farbe, x0, y0, x, y, 255.0, q);
                acht_punkte(s, b, farbe, x0, y0, x, y - 1, 255.0, q);
            }
            x += 1;
        }
    }
}

/// pygames `draw_circle_xiaolinwu_thin` - die eine Spur fuer `width == 1`.
///
/// **Nicht** dasselbe wie [`ring`] mit `dicke = 1`: dort liefen zwei Lagen
/// (`radius - 1` und `radius`) und damit die doppelte Pixelzahl.
#[allow(clippy::too_many_arguments)]
fn ring_duenn(
    s: &mut Surface,
    b: &mut Bereich,
    farbe: u32,
    x0: i32,
    y0: i32,
    radius: i32,
    q: Quadranten,
) {
    let mut x: i32 = 0;
    let mut y: i32 = radius;
    let r2 = (radius as f64).powi(2);
    let mut vorige_deckung = 0.0f64;
    while x < y {
        let hoehe = (r2 - (x as f64).powi(2)).sqrt();
        let deckung = 255.0 * (hoehe.ceil() - hoehe);
        if deckung < vorige_deckung {
            y -= 1;
        }
        vorige_deckung = deckung;
        acht_punkte(s, b, farbe, x0, y0, x, y, 255.0 - deckung as f32, q);
        acht_punkte(s, b, farbe, x0, y0, x, y - 1, deckung as f32, q);
        x += 1;
    }
}

// ---------------------------------------------------------------------------
//  Die oeffentlichen Funktionen
// ---------------------------------------------------------------------------

/// Zeichnet einen kantengeglaetteten Kreis; gibt das beschriebene Rechteck
/// zurueck.
///
/// `width == 0` fuellt, `width < 0` zeichnet **nichts** (genau wie bei
/// [`circle`](crate::draw::circle::circle) - und anders als bei
/// [`aaline`](crate::draw::aaline::aaline), wo `width < 1` trotzdem
/// zeichnet), `width > radius` wird auf `radius` gekappt und fuellt damit
/// ebenfalls. `radius < 1` zeichnet nichts.
pub fn aacircle(
    s: &mut Surface,
    c: impl IntoColor,
    center: (i32, i32),
    radius: i32,
    width: i32,
) -> Rect {
    // **`keine()`, nicht `alle()`.** Hier stand lange `Quadranten::alle()`,
    // und das war ein echter Fehler: pygames vier Schalter sind `int` mit
    // dem Vorbesetzten `0`, und der ganze Kreis ist der Fall "**kein**
    // Schalter gesetzt". Mit `alle()` landete der Normalaufruf im
    // Quadrantenzweig und zeichnete einen Ring der Dicke `radius` statt
    // Scheibe-plus-Rand. Sichtbar waren es zwei Pixel je Zeile (`FE`
    // statt `FF`) - zu wenig, um in `gleich(..)` aufzufallen, wo nur das
    // obere Halbbyte verglichen wird, aber `draw_test.py` sieht es.
    aacircle_quadranten(s, c, center, radius, width, Quadranten::keine())
}

/// Wie [`aacircle`], aber nur die angegebenen Viertel.
///
/// Sobald **ein** Schalter gesetzt ist, geht pygame einen anderen Weg:
/// `width == 0` fuellt dann nicht mit einer harten Scheibe, sondern
/// zeichnet einen Xiaolin-Wu-Ring der Dicke `radius`. Der ist innen nicht
/// ganz dicht - aber genau das tut pygame.
pub fn aacircle_quadranten(
    s: &mut Surface,
    c: impl IntoColor,
    center: (i32, i32),
    radius: i32,
    width: i32,
    q: Quadranten,
) -> Rect {
    let (posx, posy) = center;

    if radius < 1 || width < 0 {
        return Rect::new(posx, posy, 0, 0);
    }
    let width = width.min(radius);

    // pygames Schnelltest. Er rechnet mit `int` und kann dabei ueberlaufen;
    // hier in `i64`, damit ein Mittelpunkt bei `i32::MIN` nicht knallt.
    let clip = s.get_clip();
    let (px, py) = (posx as i64, posy as i64);
    let r = radius as i64;
    if px > clip.x as i64 + clip.w as i64 + r
        || px < clip.x as i64 - r
        || py > clip.y as i64 + clip.h as i64 + r
        || py < clip.y as i64 - r
    {
        return Rect::new(posx, posy, 0, 0);
    }

    let farbe = s.abbilden(c);
    let mut b = Bereich::leer();

    if q.keiner() {
        if width == 0 || width == radius {
            // Harte Scheibe mit `radius - 1`, darueber ein weicher Ring der
            // Dicke 2. Die Reihenfolge ist wesentlich: der Ring mischt auf
            // die frisch gefuellte Scheibe.
            scheibe(s, &mut b, farbe, posx, posy, radius - 1);
            ring(s, &mut b, farbe, posx, posy, radius, 2, Quadranten::alle());
        } else if width == 1 {
            ring_duenn(s, &mut b, farbe, posx, posy, radius, Quadranten::alle());
        } else {
            ring(
                s,
                &mut b,
                farbe,
                posx,
                posy,
                radius,
                width,
                Quadranten::alle(),
            );
        }
    } else if width == 0 || width == radius {
        ring(s, &mut b, farbe, posx, posy, radius, radius, q);
    } else if width == 1 {
        ring_duenn(s, &mut b, farbe, posx, posy, radius, q);
    } else {
        ring(s, &mut b, farbe, posx, posy, radius, width, q);
    }

    b.zu_rect().unwrap_or(Rect::new(posx, posy, 0, 0))
}

// ---------------------------------------------------------------------------
//  Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::aaline::pruefhilfe::gleich;
    use crate::draw::circle::pruefhilfe::schwarz;
    use crate::surface::SRCALPHA;

    const ROT: (u8, u8, u8, u8) = (255, 0, 0, 255);

    /// Die Messung aus dem Auftrag.
    #[test]
    fn der_gemessene_duenne_ring() {
        let mut s = schwarz(12, 12);
        let r = aacircle(&mut s, ROT, (6, 6), 4, 1);
        assert_eq!(r, Rect::new(2, 2, 9, 9));
        gleich(
            &s,
            0,
            &[
                "000000000000",
                "000000000000",
                "00007DFD7000",
                "000DB202BD00",
                "007B00000B70",
                "00D2000002D0",
                "00F0000000F0",
                "00D2000002D0",
                "007B00000B70",
                "000DB202BD00",
                "00007DFD7000",
                "000000000000",
            ],
        );
    }

    /// **`width == 0` ist eine harte Scheibe plus weicher Rand.** Innen
    /// steht genau die Farbe, aussen dieselbe Kante wie beim Ring der
    /// Staerke 1 - das ist kein Zufall, sondern dieselbe Xiaolin-Wu-Spur.
    #[test]
    fn der_gefuellte_kreis_ist_scheibe_plus_rand() {
        let mut s = schwarz(12, 12);
        let r = aacircle(&mut s, ROT, (6, 6), 4, 0);
        assert_eq!(r, Rect::new(2, 2, 9, 9));
        gleich(
            &s,
            0,
            &[
                "000000000000",
                "000000000000",
                "00007DFD7000",
                "000DFFFFFD00",
                "007FFFFFFF70",
                "00DFFFFFFFD0",
                "00FFFFFFFFF0",
                "00DFFFFFFFD0",
                "007FFFFFFF70",
                "000DFFFFFD00",
                "00007DFD7000",
                "000000000000",
            ],
        );
        // Die aeussere Kante ist Pixel fuer Pixel die des duennen Rings.
        let mut duenn = schwarz(12, 12);
        aacircle(&mut duenn, ROT, (6, 6), 4, 1);
        for x in 0..12 {
            assert_eq!(s.get_at((x, 2)).r, duenn.get_at((x, 2)).r, "Zeile 2, x={x}");
        }

        // **Auf das ganze Byte geprueft, nicht nur auf das obere Halbbyte.**
        // `gleich(..)` vergleicht nur `wert >> 4`; genau dort hat sich ein
        // Fehler versteckt, bei dem `aacircle` in den Quadrantenzweig lief
        // und einen Ring der Dicke `radius` zeichnete. Der Unterschied sind
        // diese beiden `254` - gemessen an pygame-ce 2.5.7, Zeile 5:
        // `00 00 DE FF FF FE FF FE FF FF DE 00`.
        let zeile5: Vec<u8> = (0..12).map(|x| s.get_at((x, 5)).r).collect();
        assert_eq!(
            zeile5,
            vec![0, 0, 0xDE, 0xFF, 0xFF, 0xFE, 0xFF, 0xFE, 0xFF, 0xFF, 0xDE, 0],
        );
    }

    /// Mit **allen vier** Schaltern auf wahr geht pygame den anderen Weg -
    /// und deckt innen voll. Gemessen, dieselbe Zeile 5:
    /// `00 00 DE FF FF FF FF FF FF FF DE 00`.
    #[test]
    fn alle_vier_schalter_sind_nicht_der_normalfall() {
        let mut s = schwarz(12, 12);
        let r = aacircle_quadranten(&mut s, ROT, (6, 6), 4, 0, Quadranten::alle());
        assert_eq!(r, Rect::new(2, 2, 9, 9));
        let zeile5: Vec<u8> = (0..12).map(|x| s.get_at((x, 5)).r).collect();
        assert_eq!(
            zeile5,
            vec![0, 0, 0xDE, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xDE, 0],
        );
    }

    /// Mit Quadrantenschaltern bleibt nur ein Viertel stehen - und das
    /// Rueckgaberechteck schrumpft mit.
    #[test]
    fn nur_ein_viertel() {
        let mut s = schwarz(12, 12);
        let r = aacircle_quadranten(
            &mut s,
            ROT,
            (6, 6),
            4,
            1,
            Quadranten {
                top_right: true,
                top_left: false,
                bottom_left: false,
                bottom_right: false,
            },
        );
        assert_eq!(r, Rect::new(6, 2, 5, 5));
        gleich(
            &s,
            0,
            &[
                "000000000000",
                "000000000000",
                "000000FD7000",
                "00000002BD00",
                "000000000B70",
                "0000000002D0",
                "0000000000F0",
                "000000000000",
                "000000000000",
                "000000000000",
                "000000000000",
                "000000000000",
            ],
        );
    }

    /// **Anders als bei `aaline`:** `width < 0` zeichnet hier nichts.
    /// Genau wie beim harten [`circle`](crate::draw::circle::circle).
    #[test]
    fn negative_breite_zeichnet_nichts() {
        let mut s = schwarz(12, 12);
        let r = aacircle(&mut s, ROT, (6, 6), 4, -1);
        assert_eq!(r, Rect::new(6, 6, 0, 0));
        for y in 0..12 {
            for x in 0..12 {
                assert_eq!(s.get_at((x, y)).r, 0, "({x},{y})");
            }
        }
    }

    /// `radius < 1` zeichnet nichts, ein weit entfernter Mittelpunkt
    /// ebenfalls - beides faellt in pygames Schnelltests heraus.
    #[test]
    fn radius_null_und_weit_draussen() {
        let mut s = schwarz(8, 8);
        assert_eq!(aacircle(&mut s, ROT, (4, 4), 0, 1), Rect::new(4, 4, 0, 0));
        assert_eq!(aacircle(&mut s, ROT, (4, 4), -2, 1), Rect::new(4, 4, 0, 0));
        assert_eq!(
            aacircle(&mut s, ROT, (900, 900), 3, 1),
            Rect::new(900, 900, 0, 0)
        );
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(s.get_at((x, y)).r, 0);
            }
        }
    }

    /// Auch hier mischt der Alphakanal mit: auf durchsichtigem Grund
    /// bekommt jedes Randpixel sein Alpha aus der Deckung.
    #[test]
    fn auf_durchsichtigem_grund_wandert_das_alpha_mit() {
        let mut s = Surface::with_flags((12, 12), SRCALPHA);
        s.fill((0u8, 0u8, 0u8, 0u8));
        aacircle(&mut s, ROT, (6, 6), 4, 1);
        for y in 0..12 {
            for x in 0..12 {
                let c = s.get_at((x, y));
                assert_eq!(c.r, c.a, "({x},{y}): Rot und Alpha laufen gleich");
                assert_eq!((c.g, c.b), (0, 0));
            }
        }
    }
}
