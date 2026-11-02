//! circle.rs
//! =========
//! `pygame.draw.circle` - **pixelgenau** nachgebaut.
//!
//! Der Mittelpunkt liegt auf der **Pixelecke**, nicht in der Pixelmitte.
//! `circle(…, (6, 6), 5, …)` beschreibt `x = 1…10`: fuenf Spalten nach
//! links, aber nur vier nach rechts. Eine symmetrische Bresenham-Umsetzung
//! trifft das nicht - deshalb steht ueberall `x0 + x - 1` statt `x0 + x`.
//!
//! pygame kennt drei getrennte Verfahren, und sie sind **nicht** ineinander
//! ueberfuehrbar (gemessen an pygame-ce 2.5.7):
//!
//! | Fall                     | Verfahren                                |
//! |--------------------------|------------------------------------------|
//! | `width == 0` oder `== r` | gefuellt, ganzzahliges Bresenham          |
//! | `width == 1`             | duenner Umriss, dieselbe Punktfolge       |
//! | sonst                    | dickes Bresenham mit zweitem, innerem Lauf|
//!
//! Vor allem gilt: `width == 1` ist **nicht** "gefuellt(r) ohne
//! gefuellt(r-1)". Das wurde gemessen und weicht ab.
//!
//! Dieses Modul beherbergt ausserdem die Bausteine `Bereich`, `hlinie`
//! und `punkt`, die `ellipse` und `arc` mitbenutzen - sie bilden pygames
//! `drawhorzlineclipbounding` und `set_and_check_rect` nach.

use crate::color::{Color, IntoColor};
use crate::draw::aacircle::Quadranten;
use crate::rect::Rect;
use crate::surface::Surface;

// ---------------------------------------------------------------------------
//  Gemeinsame Bausteine (auch von ellipse.rs und arc.rs benutzt)
// ---------------------------------------------------------------------------

/// Mitschrift des tatsaechlich beschriebenen Bereichs.
///
/// pygame fuehrt dafuer ein `int drawn_area[4]`, das mit
/// `{INT_MAX, INT_MAX, INT_MIN, INT_MIN}` beginnt; bleibt es unberuehrt,
/// gibt die Zeichenfunktion ein leeres Rechteck an der Ausgangsstelle
/// zurueck. Genau dieses Verhalten steckt hier drin.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Bereich {
    minx: i32,
    miny: i32,
    maxx: i32,
    maxy: i32,
}

impl Bereich {
    /// Noch nichts gezeichnet.
    pub(crate) fn neu() -> Bereich {
        Bereich {
            minx: i32::MAX,
            miny: i32::MAX,
            maxx: i32::MIN,
            maxy: i32::MIN,
        }
    }

    /// Ein einzelnes beschriebenes Pixel vermerken.
    pub(crate) fn merke(&mut self, x: i32, y: i32) {
        self.minx = self.minx.min(x);
        self.miny = self.miny.min(y);
        self.maxx = self.maxx.max(x);
        self.maxy = self.maxy.max(y);
    }

    /// Setzt den Bereich hart - `arc` rechnet ihn vorab aus, statt ihn
    /// beim Zeichnen mitzufuehren.
    pub(crate) fn setze(&mut self, minx: i32, miny: i32, maxx: i32, maxy: i32) {
        self.minx = minx;
        self.miny = miny;
        self.maxx = maxx;
        self.maxy = maxy;
    }

    /// Das Ergebnisrechteck. `ersatz` ist die Stelle, die pygame meldet,
    /// wenn gar nichts beschrieben wurde.
    pub(crate) fn zu_rect(&self, ersatz: (i32, i32)) -> Rect {
        if self.minx != i32::MAX
            && self.miny != i32::MAX
            && self.maxx != i32::MIN
            && self.maxy != i32::MIN
        {
            Rect::new(
                self.minx,
                self.miny,
                self.maxx - self.minx + 1,
                self.maxy - self.miny + 1,
            )
        } else {
            Rect::new(ersatz.0, ersatz.1, 0, 0)
        }
    }
}

/// Eine waagerechte Linie von `x1` bis `x2` (beide einschliesslich).
///
/// Entspricht pygames `drawhorzlineclipbounding`. Die beiden Feinheiten,
/// an denen Nachbauten scheitern:
///
/// * Ist `x2 < x1`, werden die Enden **vertauscht** statt uebersprungen.
///   Ein scheinbar leerer Aufruf wie `x1 = x0, x2 = x0 - 1` zeichnet
///   deshalb zwei Pixel - der gefuellte Kreis mit Radius 1 lebt davon.
/// * Gezeichnet wird ohne Mischen. pygames Zeichenfunktionen schreiben die
///   Farbe roh in die Flaeche (gemessen: `(255,0,0,128)` auf Blau ergibt
///   `(255,0,0,128)`, kein Mischwert) - deshalb `fill_rect`, nicht `blend`.
pub(crate) fn hlinie(s: &mut Surface, farbe: Color, x1: i32, y: i32, x2: i32, b: &mut Bereich) {
    let (a, e) = if x2 < x1 { (x2, x1) } else { (x1, x2) };
    // `fill_rect` schneidet selbst auf das Clip-Rechteck zu und meldet,
    // was uebrig blieb - genau die Zahlen, die pygame vermerkt.
    let geschrieben = s.fill_rect(farbe, Rect::new(a, y, e - a + 1, 1));
    if !geschrieben.is_empty() {
        b.merke(geschrieben.x, geschrieben.y);
        b.merke(geschrieben.x + geschrieben.w - 1, geschrieben.y);
    }
}

/// Ein einzelnes Pixel; entspricht pygames `set_and_check_rect`.
pub(crate) fn punkt(s: &mut Surface, farbe: Color, x: i32, y: i32, b: &mut Bereich) {
    // Das Clip-Rechteck liegt immer innerhalb der Flaeche, also genuegt
    // dieser eine Test, um zu wissen, ob wirklich geschrieben wurde.
    if s.get_clip().collidepoint((x, y)) {
        s.set_at((x, y), farbe);
        b.merke(x, y);
    }
}

// ---------------------------------------------------------------------------
//  Die drei Verfahren
// ---------------------------------------------------------------------------

/// Gefuellter Kreis (`width == 0` oder `width == radius`).
///
/// Die Abfrage `if f >= 0` vor den beiden waagerechten Linien ist keine
/// Zierde: sie sorgt dafuer, dass eine Zeile nur beim letzten Schritt mit
/// gleichem `y` gezeichnet wird. Ohne sie wuerde mehrfach ueberzeichnet -
/// bei undurchsichtigen Farben unsichtbar, bei Alpha aber messbar.
fn gefuellt(s: &mut Surface, farbe: Color, x0: i32, y0: i32, radius: i32, b: &mut Bereich) {
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

        if f >= 0 {
            hlinie(
                s,
                farbe,
                (x0 - x) as i32,
                (y0 + y - 1) as i32,
                (x0 + x - 1) as i32,
                b,
            );
            hlinie(
                s,
                farbe,
                (x0 - x) as i32,
                (y0 - y) as i32,
                (x0 + x - 1) as i32,
                b,
            );
        }
        hlinie(
            s,
            farbe,
            (x0 - y) as i32,
            (y0 + x - 1) as i32,
            (x0 + y - 1) as i32,
            b,
        );
        hlinie(
            s,
            farbe,
            (x0 - y) as i32,
            (y0 - x) as i32,
            (x0 + y - 1) as i32,
            b,
        );
    }
}

/// Umriss der Staerke eins.
///
/// Dieselbe Punktfolge wie beim gefuellten Kreis, aber achtfach gespiegelt
/// als einzelne Pixel gesetzt.
fn umriss_duenn(s: &mut Surface, farbe: Color, x0: i32, y0: i32, radius: i32, b: &mut Bereich) {
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

        for (px, py) in [
            (x0 + x - 1, y0 + y - 1),
            (x0 - x, y0 + y - 1),
            (x0 + x - 1, y0 - y),
            (x0 - x, y0 - y),
            (x0 + y - 1, y0 + x - 1),
            (x0 + y - 1, y0 - x),
            (x0 - y, y0 + x - 1),
            (x0 - y, y0 - x),
        ] {
            punkt(s, farbe, px as i32, py as i32, b);
        }
    }
}

/// Dicker Umriss (`2 <= width < radius`).
///
/// Zwei Bresenham-Laeufe gleichzeitig: der aeussere auf `radius`, der
/// innere auf `radius - width + 1`. Solange der innere noch nicht
/// angefangen hat (`line == true`), wird durchgezeichnet; danach nur noch
/// die beiden Randstuecke links und rechts der Aussparung.
fn umriss_dick(
    s: &mut Surface,
    farbe: Color,
    x0: i32,
    y0: i32,
    radius: i32,
    dicke: i32,
    b: &mut Bereich,
) {
    let (x0, y0) = (x0 as i64, y0 as i64);
    let mut x: i64 = 0;
    let mut y: i64 = radius as i64;
    let r2: i64 = radius as i64 * radius as i64;
    let r2x2: i64 = 2 * r2;
    let mut d1: f64 = r2 as f64 * (1.25 - radius as f64);
    let mut dx: i64 = 0;
    let mut dy: i64 = r2x2 * y;

    let mut voll = true;
    let r_innen: i64 = radius as i64 - dicke as i64 + 1;
    let mut x_innen: i64 = 0;
    let mut y_innen: i64 = r_innen;
    let ri2: i64 = r_innen * r_innen;
    let ri2x2: i64 = 2 * ri2;
    let mut d1_innen: f64 = ri2 as f64 * (1.25 - r_innen as f64);
    let mut d2_innen: f64 = 0.0;
    let mut dx_innen: i64 = 0;
    let mut dy_innen: i64 = ri2x2 * y_innen;

    // Beide Schleifen zeichnen dasselbe - einmal als Vollzeile, einmal als
    // zwei Randstuecke.
    macro_rules! zeichne {
        () => {
            if voll {
                hlinie(
                    s,
                    farbe,
                    (x0 - x) as i32,
                    (y0 - y) as i32,
                    (x0 + x - 1) as i32,
                    b,
                );
                hlinie(
                    s,
                    farbe,
                    (x0 - x) as i32,
                    (y0 + y - 1) as i32,
                    (x0 + x - 1) as i32,
                    b,
                );
            } else {
                hlinie(
                    s,
                    farbe,
                    (x0 - x) as i32,
                    (y0 - y) as i32,
                    (x0 - x_innen) as i32,
                    b,
                );
                hlinie(
                    s,
                    farbe,
                    (x0 - x) as i32,
                    (y0 + y - 1) as i32,
                    (x0 - x_innen) as i32,
                    b,
                );
                hlinie(
                    s,
                    farbe,
                    (x0 + x_innen - 1) as i32,
                    (y0 - y) as i32,
                    (x0 + x - 1) as i32,
                    b,
                );
                hlinie(
                    s,
                    farbe,
                    (x0 + x_innen - 1) as i32,
                    (y0 + y - 1) as i32,
                    (x0 + x - 1) as i32,
                    b,
                );
            }
        };
    }

    // Erster Abschnitt: solange die x-Schrittweite kleiner ist als die in y.
    while dx < dy {
        while d1 < 0.0 {
            x += 1;
            dx += r2x2;
            d1 += (dx + r2) as f64;
        }
        zeichne!();
        x += 1;
        y -= 1;
        dx += r2x2;
        dy -= r2x2;
        d1 += (dx - dy + r2) as f64;
        if voll && y < r_innen {
            voll = false;
        }
        if !voll {
            while d1_innen < 0.0 {
                x_innen += 1;
                dx_innen += ri2x2;
                d1_innen += (dx_innen + ri2) as f64;
            }
            x_innen += 1;
            y_innen -= 1;
            dx_innen += ri2x2;
            dy_innen -= ri2x2;
            d1_innen += (dx_innen - dy_innen + ri2) as f64;
        }
    }

    // Zweiter Abschnitt: der flachere Teil bis hinunter zu y == 0.
    d1 = r2 as f64
        * ((x as f64 + 0.5) * (x as f64 + 0.5) + (y as f64 - 1.0) * (y as f64 - 1.0) - r2 as f64);
    while y >= 0 {
        zeichne!();
        if d1 > 0.0 {
            y -= 1;
            dy -= r2x2;
            d1 += (r2 - dy) as f64;
        } else {
            y -= 1;
            x += 1;
            dx += r2x2;
            dy -= r2x2;
            d1 += (dx - dy + r2) as f64;
        }
        if voll && y < r_innen {
            voll = false;
        }
        if !voll {
            if dx_innen < dy_innen {
                while d1_innen < 0.0 {
                    x_innen += 1;
                    dx_innen += ri2x2;
                    d1_innen += (dx_innen + ri2) as f64;
                }
                x_innen += 1;
                y_innen -= 1;
                dx_innen += ri2x2;
                dy_innen -= ri2x2;
                d1_innen += (dx_innen - dy_innen + ri2) as f64;
            } else {
                // pygame prueft hier `if (!d2_inner)` - die Null gilt als
                // "noch nicht ausgerechnet".
                if d2_innen == 0.0 {
                    d2_innen = ri2 as f64
                        * ((x_innen as f64 + 0.5) * (x_innen as f64 + 0.5)
                            + (y_innen as f64 - 1.0) * (y_innen as f64 - 1.0)
                            - ri2 as f64);
                }
                if d2_innen > 0.0 {
                    y_innen -= 1;
                    dy_innen -= ri2x2;
                    d2_innen += (ri2 - dy_innen) as f64;
                } else {
                    y_innen -= 1;
                    x_innen += 1;
                    dx_innen += ri2x2;
                    dy_innen -= ri2x2;
                    d2_innen += (dx_innen - dy_innen + ri2) as f64;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
//  Die oeffentliche Funktion
// ---------------------------------------------------------------------------

/// Zeichnet einen Kreis; gibt das beschriebene Rechteck zurueck.
///
/// `width == 0` fuellt, `width < 0` zeichnet **nichts** (gemessen - das ist
/// nicht dasselbe!), `width >= radius` fuellt wieder.
pub fn circle(
    s: &mut Surface,
    c: impl IntoColor,
    center: (i32, i32),
    radius: i32,
    width: i32,
) -> Rect {
    circle_quadranten(s, c, center, radius, width, Quadranten::keine())
}

/// Wie [`circle`], aber nur die angegebenen Viertel.
///
/// Zeilengetreue Uebertragung von `draw_circle_quadrant` aus
/// `src_c/draw.c` (pygame-ce 2.5.7). Hier stand in der Huelle lange ein
/// `NotImplementedError` mit dem Vermerk "draw_top_* fehlt im Kern" - das
/// stimmte, und die Luecke war genau so gross wie diese Funktion.
///
/// Zwei Dinge, die man aus der Signatur nicht sieht:
///
/// * **Kein gesetzter Schalter heisst ganzer Kreis**, nicht "gar nichts".
///   pygames Beiwerte sind `int` mit dem Vorbesetzten `0`, und die Abfrage
///   lautet `top_right == 0 && ... == 0`. `draw_top_right=False` aendert
///   deshalb nichts.
/// * `radius == 1` ist ein ausdruecklicher Sonderfall im Original: vier
///   einzelne Pixel um die Mitte herum, ohne Bresenham.
pub fn circle_quadranten(
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

    // pygames Schnelltest: liegt der Kreis vollstaendig neben dem
    // Clip-Rechteck, wird gar nicht erst gezeichnet.
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

    let farbe = c.into_color();
    let mut b = Bereich::neu();

    if q.keiner() {
        if width == 0 || width == radius {
            gefuellt(s, farbe, posx, posy, radius, &mut b);
        } else if width == 1 {
            umriss_duenn(s, farbe, posx, posy, radius, &mut b);
        } else {
            umriss_dick(s, farbe, posx, posy, radius, width, &mut b);
        }
    } else {
        viertel(s, farbe, posx, posy, radius, width, q, &mut b);
    }

    b.zu_rect((posx, posy))
}

/// pygames `draw_circle_quadrant`.
///
/// Der Aufbau ist zwei Bresenham-Laeufe wie in [`umriss_dick`] - aber die
/// acht Achtel werden hier einzeln geschrieben, jedes mit seiner eigenen
/// Randbedingung. Die Vergleiche wie `(y0 - y1) < (y0 - x)` sehen nach
/// `y1 > x` aus und sind es bei `int` auch; sie stehen trotzdem in dieser
/// Form da, weil sie im Original so stehen und ein Ueberlauf in `x0`/`y0`
/// sonst anders ausginge.
///
/// `dicke == 0` (also `width == 0`) nimmt den zweiten, fuellenden Zweig:
/// je Schritt eine senkrechte Spanne bis zur Mitte.
#[allow(clippy::too_many_arguments)]
fn viertel(
    s: &mut Surface,
    farbe: Color,
    x0: i32,
    y0: i32,
    radius: i32,
    dicke: i32,
    q: Quadranten,
    b: &mut Bereich,
) {
    // Ein Punkt aus `i64`-Rechnung. pygame rechnet in `int` und laeuft bei
    // riesigen Mittelpunkten ueber; hier wird stattdessen breit gerechnet
    // und beim Setzen gekuerzt - was ausserhalb des Clip-Rechtecks landet,
    // zeichnet ohnehin nichts.
    fn p(s: &mut Surface, farbe: Color, x: i64, y: i64, b: &mut Bereich) {
        punkt(s, farbe, x as i32, y as i32, b);
    }

    let (x0, y0) = (x0 as i64, y0 as i64);
    let radius = radius as i64;
    let mut dicke = dicke as i64;

    if radius == 1 {
        if q.top_right {
            p(s, farbe, x0, y0 - 1, b);
        }
        if q.top_left {
            p(s, farbe, x0 - 1, y0 - 1, b);
        }
        if q.bottom_left {
            p(s, farbe, x0 - 1, y0, b);
        }
        if q.bottom_right {
            p(s, farbe, x0, y0, b);
        }
        return;
    }

    let mut f = 1 - radius;
    let mut ddf_x: i64 = 0;
    let mut ddf_y = -2 * radius;
    let mut x: i64 = 0;
    let mut y = radius;
    let mut i_y = radius - dicke;
    let mut i_f = 1 - i_y;
    let mut i_ddf_x: i64 = 0;
    let mut i_ddf_y = -2 * i_y;

    if dicke != 0 {
        while x < y {
            if f >= 0 {
                y -= 1;
                ddf_y += 2;
                f += ddf_y;
            }
            if i_f >= 0 {
                i_y -= 1;
                i_ddf_y += 2;
                i_f += i_ddf_y;
            }
            x += 1;
            ddf_x += 2;
            f += ddf_x + 1;

            i_ddf_x += 2;
            i_f += i_ddf_x + 1;

            // Ab dem zweiten Schritt richtet sich die Dicke nach dem
            // Abstand der beiden Laeufe - so bleibt der Ring gleich breit.
            if dicke > 1 {
                dicke = y - i_y;
            }

            /* Die Zahlen bezeichnen die Achtel im Bogenmass:
               [n - 1 * pi / 4, n * pi / 4]. */
            if q.top_right {
                for i in 0..dicke {
                    let y1 = y - i;
                    if (y0 - y1) < (y0 - x) {
                        p(s, farbe, x0 + x - 1, y0 - y1, b); /* 2 */
                    }
                    if (x0 + y1 - 1) >= (x0 + x - 1) {
                        p(s, farbe, x0 + y1 - 1, y0 - x, b); /* 1 */
                    }
                }
            }
            if q.top_left {
                for i in 0..dicke {
                    let y1 = y - i;
                    if (y0 - y1) <= (y0 - x) {
                        p(s, farbe, x0 - x, y0 - y1, b); /* 3 */
                    }
                    if (x0 - y1) < (x0 - x) {
                        p(s, farbe, x0 - y1, y0 - x, b); /* 4 */
                    }
                }
            }
            if q.bottom_left {
                for i in 0..dicke {
                    let y1 = y - i;
                    if (x0 - y1) <= (x0 - x) {
                        p(s, farbe, x0 - y1, y0 + x - 1, b); /* 5 */
                    }
                    if (y0 + y1 - 1) > (y0 + x - 1) {
                        p(s, farbe, x0 - x, y0 + y1 - 1, b); /* 6 */
                    }
                }
            }
            if q.bottom_right {
                for i in 0..dicke {
                    let y1 = y - i;
                    if (y0 + y1 - 1) >= (y0 + x - 1) {
                        p(s, farbe, x0 + x - 1, y0 + y1 - 1, b); /* 7 */
                    }
                    if (x0 + y1 - 1) > (x0 + x - 1) {
                        p(s, farbe, x0 + y1 - 1, y0 + x - 1, b); /* 8 */
                    }
                }
            }
        }
    } else {
        while x < y {
            if f >= 0 {
                y -= 1;
                ddf_y += 2;
                f += ddf_y;
            }
            x += 1;
            ddf_x += 2;
            f += ddf_x + 1;
            if q.top_right {
                for y1 in (y0 - x)..=y0 {
                    p(s, farbe, x0 + y - 1, y1, b); /* 1 */
                }
                for y1 in (y0 - y)..=y0 {
                    p(s, farbe, x0 + x - 1, y1, b); /* 2 */
                }
            }
            if q.top_left {
                for y1 in (y0 - x)..=y0 {
                    p(s, farbe, x0 - y, y1, b); /* 4 */
                }
                for y1 in (y0 - y)..=y0 {
                    p(s, farbe, x0 - x, y1, b); /* 3 */
                }
            }
            if q.bottom_left {
                for y1 in y0..(y0 + x) {
                    p(s, farbe, x0 - y, y1, b); /* 4 */
                }
                for y1 in y0..(y0 + y) {
                    p(s, farbe, x0 - x, y1, b); /* 3 */
                }
            }
            if q.bottom_right {
                for y1 in y0..(y0 + x) {
                    p(s, farbe, x0 + y - 1, y1, b); /* 1 */
                }
                for y1 in y0..(y0 + y) {
                    p(s, farbe, x0 + x - 1, y1, b); /* 2 */
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
//  Tests
// ---------------------------------------------------------------------------

/// Gemeinsame Pruefhilfe fuer circle, ellipse und arc.
#[cfg(test)]
pub(crate) mod pruefhilfe {
    use crate::surface::Surface;
    use std::path::PathBuf;

    /// Wo die mit echtem pygame erzeugten Referenzbilder liegen.
    fn golden_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("tests")
            .join("golden")
    }

    /// Laedt ein Referenzbild als RGBA-Bytes.
    fn lade(name: &str) -> Option<(u32, u32, Vec<u8>)> {
        let pfad = golden_dir().join(format!("{name}.png"));
        let datei = std::fs::File::open(pfad).ok()?;
        let mut leser = png::Decoder::new(datei).read_info().ok()?;
        let mut puffer = vec![0; leser.output_buffer_size()];
        let info = leser.next_frame(&mut puffer).ok()?;
        puffer.truncate(info.buffer_size());
        if info.bit_depth != png::BitDepth::Eight {
            return None;
        }
        // pygame legt je nach Inhalt Graustufen, RGB oder RGBA ab -
        // hier wird alles auf RGBA vereinheitlicht.
        let kanaele = match info.color_type {
            png::ColorType::Rgba => 4usize,
            png::ColorType::Rgb => 3,
            png::ColorType::GrayscaleAlpha => 2,
            png::ColorType::Grayscale => 1,
            png::ColorType::Indexed => return None,
        };
        let anzahl = (info.width as usize) * (info.height as usize);
        let mut daten = Vec::with_capacity(anzahl * 4);
        for i in 0..anzahl {
            let p = &puffer[i * kanaele..i * kanaele + kanaele];
            daten.extend_from_slice(&match kanaele {
                4 => [p[0], p[1], p[2], p[3]],
                3 => [p[0], p[1], p[2], 255],
                2 => [p[0], p[0], p[0], p[1]],
                _ => [p[0], p[0], p[0], 255],
            });
        }
        Some((info.width, info.height, daten))
    }

    /// Vergleicht eine Flaeche **byteweise** mit dem Referenzbild aus
    /// echtem pygame. Toleranz null.
    pub(crate) fn gleich_wie_pygame(name: &str, s: &Surface) {
        let Some((w, h, soll)) = lade(name) else {
            panic!(
                "Referenzbild {name}.png fehlt oder ist unlesbar. \
                 Erst erzeugen: .venv/Scripts/python.exe rustgame/tools/gen_golden.py"
            );
        };
        assert_eq!(
            (w, h),
            s.get_size(),
            "{name}: Groesse weicht vom Referenzbild ab"
        );
        let ist = s.to_rgba_bytes();
        let mut abweichungen = Vec::new();
        for i in 0..(w * h) as usize {
            let a = &soll[i * 4..i * 4 + 4];
            let e = &ist[i * 4..i * 4 + 4];
            if a != e && abweichungen.len() < 10 {
                abweichungen.push(format!(
                    "({},{}) erwartet {:?} bekommen {:?}",
                    i as u32 % w,
                    i as u32 / w,
                    a,
                    e
                ));
            }
        }
        assert!(
            abweichungen.is_empty(),
            "{name}: weicht von pygame ab:\n  {}",
            abweichungen.join("\n  ")
        );
    }

    /// Eine schwarze, undurchsichtige Flaeche - der Ausgangszustand aller
    /// Zeichenfaelle in `tools/cases.py`.
    pub(crate) fn schwarz(w: u32, h: u32) -> Surface {
        let mut s = Surface::with_flags((w, h), 0);
        s.fill((0u8, 0u8, 0u8, 255u8));
        s
    }

    /// Die Form als Textbild - `#` fuer gesetzt, `.` fuer Hintergrund.
    pub(crate) fn zeichenbild(s: &Surface) -> Vec<String> {
        let (w, h) = s.get_size();
        (0..h as i32)
            .map(|y| {
                (0..w as i32)
                    .map(|x| if s.get_at((x, y)).r > 0 { '#' } else { '.' })
                    .collect()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::pruefhilfe::{gleich_wie_pygame, schwarz, zeichenbild};
    use super::*;
    use crate::surface::SRCALPHA;

    const WEISS: (u8, u8, u8, u8) = (255, 255, 255, 255);

    // ----- die sechs Referenzfaelle aus tools/cases.py --------------------

    #[test]
    fn referenz_circle_gefuellt() {
        let mut s = schwarz(13, 13);
        assert_eq!(circle(&mut s, WEISS, (6, 6), 5, 0), Rect::new(1, 1, 10, 10));
        gleich_wie_pygame("circle_gefuellt", &s);
    }

    #[test]
    fn referenz_circle_umriss_1() {
        let mut s = schwarz(13, 13);
        assert_eq!(circle(&mut s, WEISS, (6, 6), 5, 1), Rect::new(1, 1, 10, 10));
        gleich_wie_pygame("circle_umriss_1", &s);
    }

    #[test]
    fn referenz_circle_umriss_2() {
        let mut s = schwarz(21, 21);
        assert_eq!(
            circle(&mut s, WEISS, (10, 10), 9, 2),
            Rect::new(1, 1, 18, 18)
        );
        gleich_wie_pygame("circle_umriss_2", &s);
    }

    #[test]
    fn referenz_circle_radius_0() {
        let mut s = schwarz(9, 9);
        assert_eq!(circle(&mut s, WEISS, (4, 4), 0, 0), Rect::new(4, 4, 0, 0));
        gleich_wie_pygame("circle_radius_0", &s);
    }

    #[test]
    fn referenz_circle_radius_1() {
        let mut s = schwarz(9, 9);
        assert_eq!(circle(&mut s, WEISS, (4, 4), 1, 0), Rect::new(3, 3, 2, 2));
        gleich_wie_pygame("circle_radius_1", &s);
    }

    #[test]
    fn referenz_circle_am_rand() {
        let mut s = schwarz(16, 16);
        assert_eq!(circle(&mut s, WEISS, (0, 0), 6, 0), Rect::new(0, 0, 6, 6));
        gleich_wie_pygame("circle_am_rand", &s);
    }

    // ----- Randfaelle ------------------------------------------------------

    #[test]
    fn radius_kleiner_eins_zeichnet_nichts() {
        let mut s = schwarz(9, 9);
        assert_eq!(circle(&mut s, WEISS, (4, 4), 0, 0), Rect::new(4, 4, 0, 0));
        assert_eq!(circle(&mut s, WEISS, (4, 4), -3, 0), Rect::new(4, 4, 0, 0));
        assert!(
            zeichenbild(&s).iter().all(|z| !z.contains('#')),
            "bei radius < 1 darf kein Pixel gesetzt sein"
        );
    }

    #[test]
    fn negative_breite_zeichnet_nichts() {
        // Gemessen an pygame-ce 2.5.7: width < 0 fuellt NICHT, es zeichnet
        // gar nichts. Nur width == 0 fuellt.
        let mut s = schwarz(11, 11);
        assert_eq!(circle(&mut s, WEISS, (5, 5), 4, -1), Rect::new(5, 5, 0, 0));
        assert!(zeichenbild(&s).iter().all(|z| !z.contains('#')));
    }

    #[test]
    fn zu_grosse_breite_fuellt() {
        let mut a = schwarz(16, 16);
        let mut e = schwarz(16, 16);
        let ra = circle(&mut a, WEISS, (8, 8), 5, 99);
        let re = circle(&mut e, WEISS, (8, 8), 5, 0);
        assert_eq!(ra, re);
        assert_eq!(zeichenbild(&a), zeichenbild(&e), "width > radius fuellt");
    }

    #[test]
    fn mittelpunkt_liegt_auf_der_pixelecke() {
        // Fuenf Spalten nach links, vier nach rechts - die Falle.
        let mut s = schwarz(13, 13);
        let r = circle(&mut s, WEISS, (6, 6), 5, 0);
        assert_eq!(r, Rect::new(1, 1, 10, 10));
        assert_eq!(s.get_at((1, 6)).r, 255, "x0 - radius gehoert dazu");
        assert_eq!(s.get_at((10, 6)).r, 255, "x0 + radius - 1 gehoert dazu");
        assert_eq!(s.get_at((11, 6)).r, 0, "x0 + radius gehoert nicht dazu");
    }

    #[test]
    fn clip_begrenzt_und_beschneidet_das_rueckgaberechteck() {
        let mut s = schwarz(20, 20);
        s.set_clip(Some(Rect::new(8, 8, 4, 4)));
        let r = circle(&mut s, WEISS, (10, 10), 8, 0);
        assert_eq!(r, Rect::new(8, 8, 4, 4));
        assert_eq!(s.get_at((7, 10)).r, 0, "ausserhalb des Clips bleibt leer");
        assert_eq!(s.get_at((9, 9)).r, 255);
    }

    #[test]
    fn weit_ausserhalb_liefert_leeres_rechteck() {
        let mut s = schwarz(16, 16);
        assert_eq!(
            circle(&mut s, WEISS, (200, 4), 3, 0),
            Rect::new(200, 4, 0, 0)
        );
        assert!(zeichenbild(&s).iter().all(|z| !z.contains('#')));
    }

    #[test]
    fn umriss_eins_ist_nicht_die_differenz_zweier_scheiben() {
        // Gemessen: der duenne Umriss benutzt ein eigenes Verfahren.
        // Radius 5 hat oben genau vier Pixel nebeneinander, nicht zwei
        // Paare mit Luecke.
        let mut s = schwarz(13, 13);
        circle(&mut s, WEISS, (6, 6), 5, 1);
        assert_eq!(zeichenbild(&s)[1], "....####.....");
        assert_eq!(zeichenbild(&s)[4], ".#........#..");
    }

    #[test]
    fn farbe_wird_roh_geschrieben_nicht_gemischt() {
        // pygames Zeichenfunktionen mischen nicht: (255,0,0,128) auf blau
        // ergibt genau (255,0,0,128). Gemessen an pygame-ce 2.5.7.
        let mut s = Surface::with_flags((9, 9), SRCALPHA);
        s.fill((0u8, 0u8, 255u8, 255u8));
        circle(&mut s, (255u8, 0u8, 0u8, 128u8), (4, 4), 3, 0);
        assert_eq!(s.get_at((4, 4)).as_tuple(), (255, 0, 0, 128));
    }

    #[test]
    fn gefuellter_kreis_ist_waagerecht_zusammenhaengend() {
        // Kein Loch in irgendeiner Zeile - eine typische Folge falsch
        // gesetzter Bresenham-Schritte.
        let mut s = schwarz(40, 40);
        circle(&mut s, WEISS, (20, 20), 17, 0);
        for zeile in zeichenbild(&s) {
            let inhalt = zeile.trim_matches('.');
            assert!(
                !inhalt.contains('.'),
                "Loch in einer Zeile des gefuellten Kreises: {zeile}"
            );
        }
    }

    // ----- die Viertelkreise -----------------------------------------------
    //
    // Alle Muster hier sind mit echtem pygame-ce 2.5.7 erzeugt
    // (`draw.circle(s, rot, (6,6), 4, ...)` auf einer 12x12-Flaeche) und
    // Zeichen fuer Zeichen uebernommen, nicht nachgerechnet.

    /// Hilfe: nur die genannten Viertel setzen.
    fn q(tr: bool, tl: bool, bl: bool, br: bool) -> Quadranten {
        Quadranten {
            top_right: tr,
            top_left: tl,
            bottom_left: bl,
            bottom_right: br,
        }
    }

    #[test]
    fn viertel_oben_rechts_duenn() {
        let mut s = schwarz(12, 12);
        let r = circle_quadranten(&mut s, WEISS, (6, 6), 4, 1, q(true, false, false, false));
        assert_eq!(r, Rect::new(6, 2, 4, 4));
        assert_eq!(
            zeichenbild(&s),
            vec![
                "............",
                "............",
                "......#.....",
                ".......##...",
                "........#...",
                ".........#..",
                "............",
                "............",
                "............",
                "............",
                "............",
                "............",
            ]
        );
    }

    /// `width == 0` nimmt im Original den **zweiten** Zweig von
    /// `draw_circle_quadrant`: je Schritt eine senkrechte Spanne bis zur
    /// Mitte. Das Ergebnisrechteck ist dabei um eine Zeile hoeher als beim
    /// duennen Viertel - gemessen `Rect(6, 2, 4, 5)`.
    #[test]
    fn viertel_oben_rechts_gefuellt() {
        let mut s = schwarz(12, 12);
        let r = circle_quadranten(&mut s, WEISS, (6, 6), 4, 0, q(true, false, false, false));
        assert_eq!(r, Rect::new(6, 2, 4, 5));
        assert_eq!(
            zeichenbild(&s),
            vec![
                "............",
                "............",
                "......#.....",
                "......###...",
                "......###...",
                "......####..",
                "......####..",
                "............",
                "............",
                "............",
                "............",
                "............",
            ]
        );
    }

    #[test]
    fn zwei_viertel_dicke_zwei() {
        let mut s = schwarz(12, 12);
        let r = circle_quadranten(&mut s, WEISS, (6, 6), 4, 2, q(false, true, false, true));
        assert_eq!(r, Rect::new(2, 2, 8, 8));
        assert_eq!(
            zeichenbild(&s),
            vec![
                "............",
                "............",
                ".....#......",
                "...###......",
                "...##.......",
                "..##........",
                "........##..",
                ".......##...",
                "......###...",
                "......#.....",
                "............",
                "............",
            ]
        );
    }

    #[test]
    fn alle_vier_viertel_duenn() {
        let mut s = schwarz(12, 12);
        let r = circle_quadranten(&mut s, WEISS, (6, 6), 4, 1, q(true, true, true, true));
        assert_eq!(r, Rect::new(2, 2, 8, 8));
        assert_eq!(
            zeichenbild(&s),
            vec![
                "............",
                "............",
                ".....##.....",
                "...##..##...",
                "...#....#...",
                "..#......#..",
                "..#......#..",
                "...#....#...",
                "...##..##...",
                ".....##.....",
                "............",
                "............",
            ]
        );
    }

    /// `radius == 1` ist im Original ein eigener Zweig: vier einzelne
    /// Pixel um die Mitte, ohne Bresenham. Gemessen auf 6x6 mit Mitte
    /// `(3, 3)`: oben rechts ist `(3, 2)`, unten links `(2, 3)`.
    #[test]
    fn radius_eins_ist_ein_sonderfall() {
        let mut s = schwarz(6, 6);
        let r = circle_quadranten(&mut s, WEISS, (3, 3), 1, 1, q(true, false, true, false));
        assert_eq!(r, Rect::new(2, 2, 2, 2));
        assert_eq!(
            zeichenbild(&s),
            vec!["......", "......", "...#..", "..#...", "......", "......"]
        );
    }

    /// **Kein gesetzter Schalter heisst ganzer Kreis.** Das ist der Punkt,
    /// an dem die Huelle vorher einen `NotImplementedError` geworfen hat:
    /// `draw_top_right=False` ist in pygame gar keine Auswahl, weil die
    /// Abfrage dort `== 0` lautet. Gemessen: das Bild ist Byte fuer Byte
    /// dasselbe wie ohne jeden Schalter.
    #[test]
    fn kein_schalter_ist_der_ganze_kreis() {
        let mut mit = schwarz(12, 12);
        circle_quadranten(&mut mit, WEISS, (6, 6), 4, 1, q(false, false, false, false));
        let mut ohne = schwarz(12, 12);
        circle(&mut ohne, WEISS, (6, 6), 4, 1);
        assert_eq!(zeichenbild(&mit), zeichenbild(&ohne));
    }
}
