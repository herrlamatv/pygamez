//! ellipse.rs
//! ==========
//! `pygame.draw.ellipse` - **pixelgenau** nachgebaut.
//!
//! Die Ellipse fuellt das uebergebene Rechteck aus. Ihr Mittelpunkt liegt
//! bei `(x + w/2, y + h/2)`, und weil das bei **geraden** Kanten-
//! laengen wieder eine Pixelecke ist, verschieben `x_offset` und
//! `y_offset` die rechte und untere Haelfte um ein Pixel nach innen -
//! `(w + 1) % 2` ist eins, wenn `w` gerade ist. Dadurch bleibt die Form
//! genau `w` breit und `h` hoch.
//!
//! Zwei Sonderfaelle kommen vor dem eigentlichen Verfahren (gemessen an
//! pygame-ce 2.5.7):
//!
//! * `w == 1` - eine senkrechte Linie ueber die volle Hoehe,
//! * `h == 1` - eine waagerechte Linie ueber die volle Breite.
//!
//! Ein Umriss entsteht wie beim dicken Kreis aus zwei gleichzeitig
//! laufenden Rasterungen; die innere bekommt `width - 1` als Dicke.

use crate::color::{Color, IntoColor};
use crate::rect::Rect;
use crate::surface::Surface;

use super::circle::{hlinie, punkt, Bereich};

/// Senkrechte Linie - pygames `draw_line` fuer den Fall `x1 == x2`.
fn senkrechte(s: &mut Surface, farbe: Color, x: i32, y1: i32, y2: i32, b: &mut Bereich) {
    let schritt = if y1 < y2 { 1 } else { -1 };
    let anzahl = (y1 - y2).abs();
    for i in 0..=anzahl {
        punkt(s, farbe, x, y1 + schritt * i, b);
    }
}

/// Gefuellte Ellipse im Rechteck `(x0, y0, breite, hoehe)`.
fn gefuellt(
    s: &mut Surface,
    farbe: Color,
    x0: i32,
    y0: i32,
    breite: i32,
    hoehe: i32,
    b: &mut Bereich,
) {
    if breite == 1 {
        senkrechte(s, farbe, x0, y0, y0 + hoehe - 1, b);
        return;
    }
    if hoehe == 1 {
        hlinie(s, farbe, x0, y0, x0 + breite - 1, b);
        return;
    }

    let mx = (x0 + breite / 2) as i64;
    let my = (y0 + hoehe / 2) as i64;
    let x_offset = ((breite + 1) % 2) as i64;
    let y_offset = ((hoehe + 1) % 2) as i64;
    let w = (breite / 2) as i64;
    let h = (hoehe / 2) as i64;

    let mut x: i64 = 0;
    let mut y: i64 = h;
    let mut d1: f64 = (h * h - w * w * h) as f64 + 0.25 * (w as f64) * (w as f64);
    let mut dx: i64 = 0;
    let mut dy: i64 = 2 * w * w * y;

    // Erster Abschnitt: die Flanke, auf der x schneller waechst als y faellt.
    while dx < dy {
        hlinie(
            s,
            farbe,
            (mx - x) as i32,
            (my - y) as i32,
            (mx + x - x_offset) as i32,
            b,
        );
        hlinie(
            s,
            farbe,
            (mx - x) as i32,
            (my + y - y_offset) as i32,
            (mx + x - x_offset) as i32,
            b,
        );
        if d1 < 0.0 {
            x += 1;
            dx += 2 * h * h;
            d1 += (dx + h * h) as f64;
        } else {
            x += 1;
            y -= 1;
            dx += 2 * h * h;
            dy -= 2 * w * w;
            d1 += (dx - dy + h * h) as f64;
        }
    }

    // Zweiter Abschnitt bis hinunter zu y == 0.
    let mut d2: f64 = (h * h) as f64 * ((x as f64 + 0.5) * (x as f64 + 0.5))
        + (w * w) as f64 * ((y - 1) * (y - 1)) as f64
        - (w * w * h * h) as f64;
    while y >= 0 {
        hlinie(
            s,
            farbe,
            (mx - x) as i32,
            (my - y) as i32,
            (mx + x - x_offset) as i32,
            b,
        );
        hlinie(
            s,
            farbe,
            (mx - x) as i32,
            (my + y - y_offset) as i32,
            (mx + x - x_offset) as i32,
            b,
        );
        if d2 > 0.0 {
            y -= 1;
            dy -= 2 * w * w;
            d2 += (w * w - dy) as f64;
        } else {
            y -= 1;
            x += 1;
            dx += 2 * h * h;
            dy -= 2 * w * w;
            d2 += (dx - dy + w * w) as f64;
        }
    }
}

/// Ellipsenring der Staerke `dicke + 1`.
#[allow(clippy::too_many_arguments)]
fn umriss(
    s: &mut Surface,
    farbe: Color,
    x0: i32,
    y0: i32,
    breite: i32,
    hoehe: i32,
    dicke: i32,
    b: &mut Bereich,
) {
    let mx = (x0 + breite / 2) as i64;
    let my = (y0 + hoehe / 2) as i64;
    let x_offset = ((breite + 1) % 2) as i64;
    let y_offset = ((hoehe + 1) % 2) as i64;
    let w = (breite / 2) as i64;
    let h = (hoehe / 2) as i64;
    let t = dicke as i64;
    // Die Halbachsen der inneren Ellipse.
    let wi = w - t;
    let hi = h - t;

    let mut voll = true;
    let mut x: i64 = 0;
    let mut y: i64 = h;
    let mut x_innen: i64 = 0;
    let mut y_innen: i64 = hi;

    let mut d1: f64 = (h * h - w * w * h) as f64 + 0.25 * (w as f64) * (w as f64);
    let mut d1_innen: f64 = (hi * hi - wi * wi * hi) as f64 + 0.25 * (wi as f64) * (wi as f64);
    let mut d2_innen: f64 = 0.0;
    let mut dx: i64 = 0;
    let mut dy: i64 = 2 * w * w * y;
    let mut dx_innen: i64 = 0;
    let mut dy_innen: i64 = 2 * wi * wi * y_innen;

    macro_rules! zeichne {
        () => {
            if voll {
                hlinie(
                    s,
                    farbe,
                    (mx - x) as i32,
                    (my - y) as i32,
                    (mx + x - x_offset) as i32,
                    b,
                );
                hlinie(
                    s,
                    farbe,
                    (mx - x) as i32,
                    (my + y - y_offset) as i32,
                    (mx + x - x_offset) as i32,
                    b,
                );
            } else {
                hlinie(
                    s,
                    farbe,
                    (mx - x) as i32,
                    (my - y) as i32,
                    (mx - x_innen) as i32,
                    b,
                );
                hlinie(
                    s,
                    farbe,
                    (mx - x) as i32,
                    (my + y - y_offset) as i32,
                    (mx - x_innen) as i32,
                    b,
                );
                hlinie(
                    s,
                    farbe,
                    (mx + x - x_offset) as i32,
                    (my - y) as i32,
                    (mx + x_innen - x_offset) as i32,
                    b,
                );
                hlinie(
                    s,
                    farbe,
                    (mx + x - x_offset) as i32,
                    (my + y - y_offset) as i32,
                    (mx + x_innen - x_offset) as i32,
                    b,
                );
            }
        };
    }

    // Ein Schritt der inneren Rasterung im ersten Abschnitt.
    macro_rules! innen_schritt_1 {
        () => {
            while d1_innen < 0.0 {
                x_innen += 1;
                dx_innen += 2 * hi * hi;
                d1_innen += (dx_innen + hi * hi) as f64;
            }
            x_innen += 1;
            y_innen -= 1;
            dx_innen += 2 * hi * hi;
            dy_innen -= 2 * wi * wi;
            d1_innen += (dx_innen - dy_innen + hi * hi) as f64;
        };
    }

    while dx < dy {
        zeichne!();
        if d1 < 0.0 {
            x += 1;
            dx += 2 * h * h;
            d1 += (dx + h * h) as f64;
        } else {
            x += 1;
            y -= 1;
            dx += 2 * h * h;
            dy -= 2 * w * w;
            d1 += (dx - dy + h * h) as f64;
            if voll && y < hi {
                voll = false;
            }
            if !voll && dx_innen < dy_innen {
                innen_schritt_1!();
            }
        }
    }

    let mut d2: f64 = (h * h) as f64 * ((x as f64 + 0.5) * (x as f64 + 0.5))
        + (w * w) as f64 * ((y - 1) * (y - 1)) as f64
        - (w * w * h * h) as f64;
    while y >= 0 {
        zeichne!();
        if d2 > 0.0 {
            y -= 1;
            dy -= 2 * w * w;
            d2 += (w * w - dy) as f64;
        } else {
            y -= 1;
            x += 1;
            dx += 2 * h * h;
            dy -= 2 * w * w;
            d2 += (dx - dy + w * w) as f64;
        }
        if voll && y < hi {
            voll = false;
        }
        if !voll {
            if dx_innen < dy_innen {
                innen_schritt_1!();
            } else if y_innen >= 0 {
                // pygame prueft `if (d2_inner == 0)` - die Null bedeutet
                // "noch nicht ausgerechnet".
                if d2_innen == 0.0 {
                    d2_innen = (hi * hi) as f64 * ((x_innen as f64 + 0.5) * (x_innen as f64 + 0.5))
                        + (wi * wi) as f64 * ((y_innen - 1) * (y_innen - 1)) as f64
                        - (wi * wi * hi * hi) as f64;
                }
                if d2_innen > 0.0 {
                    y_innen -= 1;
                    dy_innen -= 2 * wi * wi;
                    d2_innen += (wi * wi - dy_innen) as f64;
                } else {
                    y_innen -= 1;
                    x_innen += 1;
                    dx_innen += 2 * hi * hi;
                    dy_innen -= 2 * wi * wi;
                    d2_innen += (dx_innen - dy_innen + wi * wi) as f64;
                }
            }
        }
    }
}

/// Zeichnet eine Ellipse ins Rechteck `r`; gibt das beschriebene
/// Rechteck zurueck.
///
/// `width == 0` fuellt, `width < 0` zeichnet **nichts**, ein `width`, das
/// mindestens die halbe Kantenlaenge erreicht, fuellt ebenfalls.
pub fn ellipse(s: &mut Surface, c: impl IntoColor, r: Rect, width: i32) -> Rect {
    if width < 0 {
        return Rect::new(r.x, r.y, 0, 0);
    }
    let farbe = c.into_color();
    let mut b = Bereich::neu();

    // "Halbe Kantenlaenge, aufgerundet" - bei ungerader Laenge eins mehr.
    let halb = (r.w / 2 + r.w % 2).min(r.h / 2 + r.h % 2);
    if width == 0 || width >= halb {
        gefuellt(s, farbe, r.x, r.y, r.w, r.h, &mut b);
    } else {
        umriss(s, farbe, r.x, r.y, r.w, r.h, width - 1, &mut b);
    }

    b.zu_rect((r.x, r.y))
}

#[cfg(test)]
mod tests {
    use super::super::circle::pruefhilfe::{gleich_wie_pygame, schwarz, zeichenbild};
    use super::*;
    use crate::surface::SRCALPHA;

    const WEISS: (u8, u8, u8, u8) = (255, 255, 255, 255);

    // ----- die drei Referenzfaelle aus tools/cases.py ---------------------

    #[test]
    fn referenz_ellipse_gefuellt() {
        let mut s = schwarz(20, 14);
        let r = ellipse(&mut s, WEISS, Rect::new(2, 2, 16, 10), 0);
        assert_eq!(r, Rect::new(2, 2, 16, 10));
        gleich_wie_pygame("ellipse_gefuellt", &s);
    }

    #[test]
    fn referenz_ellipse_umriss() {
        let mut s = schwarz(20, 14);
        let r = ellipse(&mut s, WEISS, Rect::new(2, 2, 16, 10), 1);
        assert_eq!(r, Rect::new(2, 2, 16, 10));
        gleich_wie_pygame("ellipse_umriss", &s);
    }

    #[test]
    fn referenz_ellipse_flach() {
        let mut s = schwarz(20, 6);
        let r = ellipse(&mut s, WEISS, Rect::new(1, 1, 18, 4), 0);
        assert_eq!(r, Rect::new(1, 1, 18, 4));
        gleich_wie_pygame("ellipse_flach", &s);
    }

    // ----- Randfaelle ------------------------------------------------------

    #[test]
    fn negative_breite_zeichnet_nichts() {
        let mut s = schwarz(12, 12);
        assert_eq!(
            ellipse(&mut s, WEISS, Rect::new(1, 2, 9, 7), -1),
            Rect::new(1, 2, 0, 0)
        );
        assert!(zeichenbild(&s).iter().all(|z| !z.contains('#')));
    }

    #[test]
    fn leeres_rechteck_ergibt_vier_pixel() {
        // Ueberraschend, aber gemessen: ein Rechteck ohne Ausdehnung
        // zeichnet den Zwei-mal-zwei-Block links oberhalb seiner Ecke.
        // Das faellt aus der Regel "x0 - x bis x0 + x - 1", deren Enden
        // sich bei x == 0 kreuzen und dann vertauscht werden.
        let mut s = schwarz(12, 12);
        assert_eq!(
            ellipse(&mut s, WEISS, Rect::new(3, 4, 0, 0), 0),
            Rect::new(2, 3, 2, 2)
        );
        assert_eq!(s.get_at((2, 3)).r, 255);
        assert_eq!(s.get_at((3, 4)).r, 255);
        assert_eq!(s.get_at((4, 4)).r, 0);
    }

    #[test]
    fn ein_pixel_breit_wird_zur_senkrechten() {
        let mut s = schwarz(8, 10);
        let r = ellipse(&mut s, WEISS, Rect::new(3, 2, 1, 6), 0);
        assert_eq!(r, Rect::new(3, 2, 1, 6));
        for y in 2..8 {
            assert_eq!(s.get_at((3, y)).r, 255, "Zeile {y} fehlt");
        }
        assert_eq!(s.get_at((4, 4)).r, 0);
    }

    #[test]
    fn ein_pixel_hoch_wird_zur_waagerechten() {
        let mut s = schwarz(10, 8);
        let r = ellipse(&mut s, WEISS, Rect::new(2, 3, 6, 1), 0);
        assert_eq!(r, Rect::new(2, 3, 6, 1));
        assert_eq!(zeichenbild(&s)[3], "..######..");
    }

    #[test]
    fn zu_grosse_breite_fuellt() {
        let mut a = schwarz(20, 14);
        let mut e = schwarz(20, 14);
        let ra = ellipse(&mut a, WEISS, Rect::new(2, 2, 16, 10), 40);
        let re = ellipse(&mut e, WEISS, Rect::new(2, 2, 16, 10), 0);
        assert_eq!(ra, re);
        assert_eq!(zeichenbild(&a), zeichenbild(&e));
    }

    #[test]
    fn form_fuellt_das_rechteck_genau_aus() {
        // Die Ellipse beruehrt alle vier Kanten und ragt nirgends heraus -
        // daran scheitert eine Umsetzung ohne x_offset/y_offset.
        let mut s = schwarz(24, 18);
        let r = ellipse(&mut s, WEISS, Rect::new(3, 2, 18, 14), 0);
        assert_eq!(r, Rect::new(3, 2, 18, 14));
    }

    #[test]
    fn ungerade_kantenlaengen_bleiben_genau() {
        let mut s = schwarz(24, 18);
        let r = ellipse(&mut s, WEISS, Rect::new(3, 2, 17, 13), 0);
        assert_eq!(r, Rect::new(3, 2, 17, 13));
    }

    #[test]
    fn clip_begrenzt_und_beschneidet_das_rueckgaberechteck() {
        let mut s = schwarz(24, 18);
        s.set_clip(Some(Rect::new(6, 6, 5, 4)));
        let r = ellipse(&mut s, WEISS, Rect::new(3, 2, 18, 14), 0);
        assert_eq!(r, Rect::new(6, 6, 5, 4));
        assert_eq!(s.get_at((5, 7)).r, 0, "ausserhalb des Clips bleibt leer");
        assert_eq!(s.get_at((7, 7)).r, 255);
    }

    #[test]
    fn farbe_wird_roh_geschrieben_nicht_gemischt() {
        let mut s = Surface::with_flags((12, 10), SRCALPHA);
        s.fill((0u8, 0u8, 255u8, 255u8));
        ellipse(&mut s, (255u8, 0u8, 0u8, 128u8), Rect::new(1, 1, 9, 7), 0);
        assert_eq!(s.get_at((5, 4)).as_tuple(), (255, 0, 0, 128));
    }

    #[test]
    fn umriss_laesst_die_mitte_frei() {
        let mut s = schwarz(20, 14);
        ellipse(&mut s, WEISS, Rect::new(2, 2, 16, 10), 2);
        assert_eq!(s.get_at((9, 6)).r, 0, "die Mitte bleibt leer");
        assert_eq!(s.get_at((2, 6)).r, 255, "der linke Rand ist gesetzt");
    }
}
