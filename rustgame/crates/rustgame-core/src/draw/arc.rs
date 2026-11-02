//! arc.rs
//! ======
//! `pygame.draw.arc` - **pixelgenau** nachgebaut.
//!
//! Anders als Kreis und Ellipse wird der Bogen **nicht** gerastert,
//! sondern gesiebt: pygame laeuft ueber jedes Pixel im umschliessenden
//! Rechteck und fragt dreierlei ab -
//!
//! 1. liegt es innerhalb der aeusseren Ellipse (`x²/a² + y²/b² <= 1`),
//! 2. liegt es ausserhalb der inneren (dieselbe Rechnung mit `a - width`,
//!    `b - width`),
//! 3. zeigt es in den erlaubten Winkelbereich?
//!
//! Punkt drei laeuft ueber ein Skalarprodukt: der Punkt muss mit der
//! Halbierenden des Winkelbereichs mindestens so gut uebereinstimmen wie
//! die Randstrahlen. Deshalb kommt hier kein `atan2` vor.
//!
//! Winkel kommen im **Bogenmass**, gegen den Uhrzeigersinn, 0 = rechts.
//! Liegt `stop` vor `start`, wird `2π` aufgeschlagen.
//!
//! Das zurueckgegebene Rechteck ist **nicht** aus den gesetzten Pixeln
//! zusammengetragen, sondern der vorab berechnete Suchbereich, der
//! anschliessend von allen vier Seiten so weit eingeschnuert wird, bis in
//! der jeweiligen Randzeile bzw. -spalte wirklich ein Pixel liegt.

use crate::color::{Color, IntoColor};
use crate::rect::Rect;
use crate::surface::Surface;

use super::circle::Bereich;

const PI: f64 = std::f64::consts::PI;

/// Alles, was die beiden Siebe brauchen - einmal ausgerechnet.
#[derive(Debug, Clone, Copy)]
struct Sieb {
    /// `1 / a²` der aeusseren Ellipse (waagerecht bzw. senkrecht).
    inv_a: f64,
    inv_b: f64,
    /// dasselbe fuer die innere Ellipse; `inf`, wenn sie zu nichts wird.
    inv_ai: f64,
    inv_bi: f64,
    /// kleinstes erlaubtes Skalarprodukt mit der Winkelhalbierenden.
    min_skalar: f64,
    xm: f64,
    ym: f64,
}

/// `1 / v`, wobei `1 / 0` bewusst `inf` ergibt - wie in C.
///
/// Damit faellt der Test auf die innere Ellipse fuer einen Vollkreis von
/// selbst weg: `x² * inf` ist fuer `x != 0` unendlich (also nie `< 1`),
/// und fuer `x == 0` `NaN`, was den Vergleich ebenfalls nicht besteht.
fn kehrwert(v: i64) -> f64 {
    1.0 / v as f64
}

impl Sieb {
    /// Gehoert `(x, y)` - relativ zur Mitte - zum Bogen?
    fn trifft(&self, x: i32, y: i32) -> bool {
        let (xf, yf) = (x as f64, y as f64);
        let q = (x as i64 * x as i64) as f64;
        let r = (y as i64 * y as i64) as f64;
        if q * self.inv_a + r * self.inv_b > 1.0 {
            return false;
        }
        if q * self.inv_ai + r * self.inv_bi < 1.0 {
            return false;
        }
        xf * self.xm + yf * self.ym >= self.min_skalar * (q + r).sqrt()
    }
}

/// Der Suchbereich, in dem ueberhaupt Pixel liegen koennen - relativ zur
/// Mitte, bereits auf das Clip-Rechteck beschnitten.
///
/// Gibt `None`, wenn nichts uebrig bleibt.
#[allow(clippy::too_many_arguments)]
fn suchbereich(
    clip: Rect,
    sieb: &Sieb,
    start: f64,
    stop: f64,
    a: i32,
    b: i32,
    ai: i32,
    bi: i32,
    mitte: (i32, i32),
) -> Option<(i32, i32, i32, i32)> {
    let (x_start, y_start) = (start.cos(), -start.sin());
    let (x_stop, y_stop) = (stop.cos(), -stop.sin());

    // Die vier Eckpunkte der beiden Randstrahlen, kaufmaennisch gerundet.
    let runde = |v: f64| v as i32;
    let xsi = runde(x_start * ai as f64 + 0.5);
    let ysi = runde(y_start * bi as f64 + 0.5);
    let xei = runde(x_stop * ai as f64 + 0.5);
    let yei = runde(y_stop * bi as f64 + 0.5);
    let xsa = runde(x_start * a as f64 + 0.5);
    let ysa = runde(y_start * b as f64 + 0.5);
    let xea = runde(x_stop * a as f64 + 0.5);
    let yea = runde(y_stop * b as f64 + 0.5);

    // Reicht der Bogen ueber die jeweilige Scheitelstelle hinaus, gilt die
    // volle Halbachse; sonst genuegt der aeusserste Randstrahl.
    let mut minx = if -sieb.xm < sieb.min_skalar {
        xsi.min(xei).min(xsa.min(xea))
    } else {
        -a
    };
    minx = minx.max(clip.x - mitte.0);

    let mut miny = if -sieb.ym < sieb.min_skalar {
        ysi.min(yei).min(ysa.min(yea))
    } else {
        -b
    };
    miny = miny.max(clip.y - mitte.1);

    let mut maxx = if sieb.xm < sieb.min_skalar {
        xsi.max(xei).max(xsa.max(xea))
    } else {
        a
    };
    maxx = maxx.min(clip.x + clip.w - mitte.0 - 1);

    let mut maxy = if sieb.ym < sieb.min_skalar {
        ysi.max(yei).max(ysa.max(yea))
    } else {
        b
    };
    maxy = maxy.min(clip.y + clip.h - mitte.1 - 1);

    if minx >= maxx || miny >= maxy {
        return None;
    }

    // Von aussen nach innen zusammenziehen, bis die Randzeile bzw.
    // -spalte wirklich ein Pixel des Bogens enthaelt.
    while !(minx..=maxx).any(|x| sieb.trifft(x, miny)) {
        miny += 1;
        if miny >= maxy {
            return None;
        }
    }
    while !(minx..=maxx).any(|x| sieb.trifft(x, maxy)) {
        maxy -= 1;
        if maxy <= miny {
            return None;
        }
    }
    while !(miny..=maxy).any(|y| sieb.trifft(minx, y)) {
        minx += 1;
        if minx >= maxx {
            return None;
        }
    }
    while !(miny..=maxy).any(|y| sieb.trifft(maxx, y)) {
        maxx -= 1;
        if minx >= maxx {
            return None;
        }
    }
    Some((minx, miny, maxx, maxy))
}

/// Der eigentliche Bogen; `a`/`b` sind die Halbachsen, `mitte` die
/// Pixelecke in der Mitte des Rechtecks.
#[allow(clippy::too_many_arguments)]
fn zeichne_bogen(
    s: &mut Surface,
    farbe: Color,
    mitte: (i32, i32),
    a: i32,
    b: i32,
    width: i32,
    start: f64,
    stop: f64,
    bereich: &mut Bereich,
) {
    if width <= 0 {
        return;
    }
    let mut stop = stop;
    if stop < start {
        stop += 2.0 * PI;
    }
    if stop <= start {
        return;
    }

    let mitte_winkel = 0.5 * (start + stop);
    let halbe_spanne = mitte_winkel - start;
    let ai = a - width;
    let bi = b - width;

    let sieb = Sieb {
        inv_a: kehrwert(a as i64 * a as i64),
        inv_b: kehrwert(b as i64 * b as i64),
        inv_ai: kehrwert(ai as i64 * ai as i64),
        inv_bi: kehrwert(bi as i64 * bi as i64),
        // Ueber einen Halbkreis hinaus gibt es keine Winkelschranke mehr.
        min_skalar: if halbe_spanne < PI {
            halbe_spanne.cos()
        } else {
            -1.0
        },
        xm: mitte_winkel.cos(),
        ym: -mitte_winkel.sin(),
    };

    let Some((minx, miny, maxx, maxy)) =
        suchbereich(s.get_clip(), &sieb, start, stop, a, b, ai, bi, mitte)
    else {
        return;
    };
    if minx >= maxx || miny >= maxy {
        return;
    }

    // Nur ein Viertel wird durchlaufen; die drei anderen entstehen durch
    // Spiegeln - jedes mit eigener Winkelprobe.
    let hoechstes_y = maxy.max(-miny);
    for y in 0..=hoechstes_y {
        let y_positiv = y >= miny && y <= maxy;
        let y_negativ = -y >= miny && -y <= maxy;
        let y2 = (y as i64 * y as i64) as f64;

        let x_aussen = (a as f64 * (1.0 - y2 * sieb.inv_b).sqrt()) as i32;
        let x_innen = if y < bi {
            (ai as f64 * (1.0 - y2 * sieb.inv_bi).sqrt()) as i32
        } else {
            0
        };

        let y_dot = y as f64 * sieb.ym;
        for x in x_innen..=x_aussen {
            let x_positiv = x >= minx && x <= maxx;
            let x_negativ = -x >= minx && -x <= maxx;
            if !(x_positiv || x_negativ) {
                continue;
            }
            let schranke = sieb.min_skalar * ((x as i64 * x as i64) as f64 + y2).sqrt();
            let x_dot = x as f64 * sieb.xm;

            if y_positiv && x_positiv && x_dot + y_dot >= schranke {
                s.set_at((mitte.0 + x, mitte.1 + y), farbe);
            }
            if y_positiv && x_negativ && -x_dot + y_dot >= schranke {
                s.set_at((mitte.0 - x, mitte.1 + y), farbe);
            }
            if y_negativ && x_positiv && x_dot - y_dot >= schranke {
                s.set_at((mitte.0 + x, mitte.1 - y), farbe);
            }
            if y_negativ && x_negativ && -x_dot - y_dot >= schranke {
                s.set_at((mitte.0 - x, mitte.1 - y), farbe);
            }
        }
    }

    bereich.setze(
        minx + mitte.0,
        miny + mitte.1,
        maxx + mitte.0,
        maxy + mitte.1,
    );
}

/// Zeichnet einen Ellipsenbogen im Rechteck `r`; gibt das beschriebene
/// Rechteck zurueck.
///
/// `start`/`stop` sind Winkel im **Bogenmass**, gegen den Uhrzeigersinn,
/// 0 = rechts. `width` ist die Strichstaerke nach innen; `width < 0`
/// zeichnet nichts.
///
/// **Die Winkel sind `f64`, nicht `f32`** - pygame liest sie mit dem
/// Format `"O!OOdd|i"`, also als `double`, und `draw.arc` ist die einzige
/// Funktion in `draw.c`, die das tut. Hier stand `f32`, und damit lag der
/// Bogen in **1887 von 4116** Faellen daneben (Groessen 5x3 bis 30x18,
/// Breiten 1 bis 3, vierzehn Winkel je Ende): `math.pi` als `f32` ist
/// `3.1415927`, also **groesser** als pi, und genau an den Achsen, wo ein
/// Bogen beginnt oder endet, entscheidet das letzte Bit, ob das
/// Randpixel noch dazugehoert. Nachgewiesen in beide Richtungen: pygame
/// mit **auf `f32` gerundeten** Winkeln traf die alte Fassung in 4116 von
/// 4116 Faellen - der Algorithmus war richtig, nur die Zahl nicht.
/// Sichtbar war es zuerst am Laecheln des Minesweeper-Smileys
/// (`draw.arc(.., math.pi, math.tau, 2)`): pygame setzt das linke Ende
/// eine Zeile hoeher, die alte Fassung das rechte.
pub fn arc(s: &mut Surface, c: impl IntoColor, r: Rect, start: f64, stop: f64, width: i32) -> Rect {
    if width < 0 {
        return Rect::new(r.x, r.y, 0, 0);
    }
    let mut width = width;
    // pygames zwei Klammerungen: erst gross-, dann kleinrechnen. Das sieht
    // umstaendlich aus, ist aber genau die gemessene Reihenfolge.
    if width > r.w / 2 || width > r.h / 2 {
        width = (r.w / 2).max(r.h / 2);
    }
    width = width.min(r.w.min(r.h) / 2);

    let mut stop = stop;
    if stop < start {
        stop += 2.0 * PI;
    }

    let farbe = c.into_color();
    let mut b = Bereich::neu();
    zeichne_bogen(
        s,
        farbe,
        (r.x + r.w / 2, r.y + r.h / 2),
        r.w / 2,
        r.h / 2,
        width,
        start,
        stop,
        &mut b,
    );
    b.zu_rect((r.x, r.y))
}

// Die Referenzfaelle aus `tools/cases.py` rechnen mit **genau** 3.14159
// bzw. 6.28 - nicht mit pi und tau. Die Referenzbilder sind mit diesen
// Zahlen entstanden; sie durch die genauen Konstanten zu ersetzen waere
// ein anderer Test.
#[cfg(test)]
#[allow(clippy::approx_constant)]
mod tests {
    use super::super::circle::pruefhilfe::{gleich_wie_pygame, schwarz, zeichenbild};
    use super::*;
    use crate::surface::SRCALPHA;

    const WEISS: (u8, u8, u8, u8) = (255, 255, 255, 255);

    // ----- die beiden Referenzfaelle aus tools/cases.py -------------------

    #[test]
    fn referenz_arc_halb() {
        let mut s = schwarz(20, 20);
        let r = arc(&mut s, WEISS, Rect::new(2, 2, 16, 16), 0.0, 3.14159, 1);
        gleich_wie_pygame("arc_halb", &s);
        // Gemessen an pygame-ce 2.5.7: das Rechteck ist nicht symmetrisch,
        // weil der Bogen rechts unterhalb der Mitte endet.
        assert_eq!(r, Rect::new(3, 2, 16, 9));
    }

    #[test]
    fn referenz_arc_dick() {
        let mut s = schwarz(24, 24);
        let r = arc(&mut s, WEISS, Rect::new(2, 2, 20, 20), 0.5, 2.5, 3);
        gleich_wie_pygame("arc_dick", &s);
        assert!(!r.is_empty(), "der dicke Bogen beschreibt Pixel");
    }

    // ----- Randfaelle ------------------------------------------------------

    #[test]
    fn negative_breite_zeichnet_nichts() {
        let mut s = schwarz(16, 16);
        assert_eq!(
            arc(&mut s, WEISS, Rect::new(1, 1, 12, 12), 0.0, 3.0, -1),
            Rect::new(1, 1, 0, 0)
        );
        assert!(zeichenbild(&s).iter().all(|z| !z.contains('#')));
    }

    #[test]
    fn breite_null_zeichnet_nichts() {
        // Anders als bei Kreis und Ellipse fuellt width == 0 hier nicht -
        // pygames draw_arc steigt bei `width <= 0` sofort aus.
        let mut s = schwarz(16, 16);
        assert_eq!(
            arc(&mut s, WEISS, Rect::new(1, 1, 12, 12), 0.0, 3.0, 0),
            Rect::new(1, 1, 0, 0)
        );
        assert!(zeichenbild(&s).iter().all(|z| !z.contains('#')));
    }

    #[test]
    fn gleiche_winkel_zeichnen_nichts() {
        let mut s = schwarz(16, 16);
        assert_eq!(
            arc(&mut s, WEISS, Rect::new(1, 1, 12, 12), 1.0, 1.0, 2),
            Rect::new(1, 1, 0, 0)
        );
        assert!(zeichenbild(&s).iter().all(|z| !z.contains('#')));
    }

    #[test]
    fn rueckwaerts_laufende_winkel_gehen_ueber_null() {
        // stop < start bekommt 2*pi aufgeschlagen - der Bogen laeuft dann
        // ueber die Null hinweg und liegt rechts.
        let mut s = schwarz(20, 20);
        let r = arc(&mut s, WEISS, Rect::new(2, 2, 16, 16), 2.0, 1.0, 2);
        assert!(!r.is_empty());
        assert!(
            zeichenbild(&s).iter().any(|z| z.contains('#')),
            "ueber die Null laufende Boegen zeichnen etwas"
        );
    }

    #[test]
    fn zu_flaches_rechteck_zeichnet_nichts() {
        let mut s = schwarz(16, 16);
        assert_eq!(
            arc(&mut s, WEISS, Rect::new(2, 2, 12, 1), 0.0, 3.0, 1),
            Rect::new(2, 2, 0, 0)
        );
        assert!(zeichenbild(&s).iter().all(|z| !z.contains('#')));
    }

    #[test]
    fn halber_bogen_liegt_oben() {
        // 0 bis pi laeuft gegen den Uhrzeigersinn - in Bildkoordinaten
        // also nach oben.
        let mut s = schwarz(20, 20);
        let r = arc(&mut s, WEISS, Rect::new(2, 2, 16, 16), 0.0, 3.14159, 1);
        assert_eq!(r.y, 2, "der Bogen reicht bis an den oberen Rand");
        assert!(r.h <= 9, "und nicht in die untere Haelfte: {r:?}");
        assert_eq!(s.get_at((10, 2)).r, 255, "oben in der Mitte gesetzt");
        assert_eq!(s.get_at((10, 17)).r, 0, "unten nichts");
    }

    #[test]
    fn clip_begrenzt_und_beschneidet_das_rueckgaberechteck() {
        let mut s = schwarz(20, 20);
        s.set_clip(Some(Rect::new(2, 2, 8, 8)));
        let r = arc(&mut s, WEISS, Rect::new(2, 2, 16, 16), 0.0, 3.14159, 2);
        assert!(Rect::new(2, 2, 8, 8).contains(r), "{r:?} liegt im Clip");
        assert_eq!(s.get_at((13, 3)).r, 0, "ausserhalb des Clips bleibt leer");
    }

    #[test]
    fn farbe_wird_roh_geschrieben_nicht_gemischt() {
        let mut s = Surface::with_flags((20, 20), SRCALPHA);
        s.fill((0u8, 0u8, 255u8, 255u8));
        arc(
            &mut s,
            (255u8, 0u8, 0u8, 128u8),
            Rect::new(2, 2, 16, 16),
            0.0,
            3.14159,
            1,
        );
        assert_eq!(s.get_at((10, 2)).as_tuple(), (255, 0, 0, 128));
    }

    #[test]
    fn voller_kreis_schliesst_sich() {
        let mut s = schwarz(20, 20);
        let r = arc(&mut s, WEISS, Rect::new(2, 2, 16, 16), 0.0, 6.28, 1);
        // Der Ring reicht von Mitte minus acht bis Mitte plus acht - also
        // siebzehn Pixel, nicht sechzehn. Gemessen an pygame-ce 2.5.7.
        assert_eq!(r, Rect::new(2, 2, 17, 17), "ein voller Ring reicht ringsum");
        assert_eq!(s.get_at((10, 2)).r, 255, "oben");
        assert_eq!(s.get_at((10, 18)).r, 255, "unten");
    }
}
