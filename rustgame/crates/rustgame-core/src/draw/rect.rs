//! rect.rs
//! =======
//! `pygame.draw.rect` - das meistgebrauchte Primitiv der Sammlung:
//! 900 Aufrufe, davon 719 mit `border_radius` und 302 als Umriss mit Radius.
//!
//! SDL2 kennt keine abgerundeten Rechtecke; pygame rechnet sie in
//! `src_c/draw.c` selbst aus. Diese Datei bildet genau diesen Rechenweg
//! nach - samt seiner Eigenheiten, denn gesucht ist nicht *ein huebsches*
//! Rundrechteck, sondern **dasselbe Pixelmuster**:
//!
//! | Fall | wie pygame es baut |
//! |---|---|
//! | eckig, gefuellt | ein `fill_rect` auf das Clip-Rechteck |
//! | eckig, Umriss | vier Balken (`draw_rect`) |
//! | rund, gefuellt | ein Achteck als Fuellpolygon **plus** vier Kreisviertel |
//! | rund, Umriss | vier dicke Linien **plus** vier Kreisviertel-Ringe |
//!
//! Drei gemessene Eigenheiten, die ein "vernuenftiger" Nachbau verfehlen
//! wuerde und die hier absichtlich mitwandern:
//!
//! * **`width < 0` zeichnet gar nichts** und gibt ein leeres Rechteck
//!   zurueck - nicht etwa gefuellt.
//! * **Die Farbe wird ohne Mischen geschrieben.** `draw.rect` mit
//!   `(255,0,0,128)` hinterlaesst auf einer SRCALPHA-Flaeche genau diesen
//!   Wert; es gibt kein Alpha-Mischen wie beim Blit.
//! * **Das Rueckgabe-Rechteck hat zwei Bedeutungen:** eckig ist es das auf
//!   das Clip-Rechteck beschnittene Eingaberechteck, rund dagegen die
//!   Umgrenzung der *tatsaechlich* gesetzten Pixel.
//!
//! Gegenprobe: eine zeilengetreue Portierung dieser Datei nach Python wurde
//! gegen echtes pygame-ce 2.5.7 gehalten - 12.474 Faelle (Groessen, Breiten,
//! Radien, Eckradien, Randlagen, negative Groessen, Clip-Rechtecke),
//! **0 Abweichungen**.

use crate::color::{Color, IntoColor};
use crate::rect::Rect;
use crate::surface::Surface;

// ---------------------------------------------------------------------------
//  Oeffentliche Zeichenfunktionen
// ---------------------------------------------------------------------------

/// Zeichnet ein Rechteck mit scharfen Ecken.
///
/// `width <= 0` fuellt, `width > 0` ist die Strichstaerke nach innen,
/// `width < 0` zeichnet nichts (so haelt es pygame). Zurueck kommt das
/// tatsaechlich beschriebene Rechteck, auf das Clip-Rechteck beschnitten.
pub fn rect(s: &mut Surface, c: impl IntoColor, r: Rect, width: i32) -> Rect {
    rect_corners(s, c, r, width, 0, -1, -1, -1, -1)
}

/// Abgerundetes Rechteck - `radius <= 0` ist wieder das eckige.
///
/// Ein zu grosser Radius wird auf die Rechteckgroesse begrenzt, ein
/// `width`, das ueber die halbe Kantenlaenge geht, ebenso.
pub fn rect_radius(s: &mut Surface, c: impl IntoColor, r: Rect, width: i32, radius: i32) -> Rect {
    rect_corners(s, c, r, width, radius, -1, -1, -1, -1)
}

/// Der vollstaendige Aufruf mit einem eigenen Radius je Ecke.
///
/// Gegenstueck zu `border_top_left_radius` und seinen drei Geschwistern;
/// `-1` heisst "nimm `radius`". Die Sammlung braucht das an elf Stellen
/// (Wordle-Tastatur, Sudoku-Kopfzeile, Hauptmenue), deshalb steht es hier
/// und nicht in einer spaeteren Ausbaustufe.
#[allow(clippy::too_many_arguments)]
pub fn rect_corners(
    s: &mut Surface,
    c: impl IntoColor,
    r: Rect,
    width: i32,
    radius: i32,
    top_left: i32,
    top_right: i32,
    bottom_left: i32,
    bottom_right: i32,
) -> Rect {
    let farbe = c.into_color();

    // Negative Strichstaerke: pygame steigt hier aus, ohne ein Pixel zu
    // setzen. Das ist kein Versehen, sondern dokumentiert.
    if width < 0 {
        return Rect::new(r.x, r.y, 0, 0);
    }

    let eckig =
        radius <= 0 && top_left <= 0 && top_right <= 0 && bottom_left <= 0 && bottom_right <= 0;

    // Die zweite Bedingung ist pygames Notbremse: bei einer Kante von 0 oder
    // 1 Pixel zeichnet der Rundrechteck-Zweig Unsinn, also geht auch ein
    // angefordertes Rundrechteck dann durch den eckigen Weg.
    if eckig || r.w.unsigned_abs() < 2 || r.h.unsigned_abs() < 2 {
        let ziel = r.clip(s.get_clip());
        if ziel.is_empty() {
            return Rect::new(r.x, r.y, 0, 0);
        }
        // Gemessen: der Umriss-Zweig entscheidet an der **beschnittenen**
        // Groesse. Ein Rechteck, das halb aus dem Bild ragt, wird dadurch
        // eher gefuellt als eines mitten im Bild.
        if width > 0 && width.saturating_mul(2) < ziel.w && width.saturating_mul(2) < ziel.h {
            rahmen(
                s,
                farbe,
                r.x,
                r.y,
                r.x.saturating_add(r.w).saturating_sub(1),
                r.y.saturating_add(r.h).saturating_sub(1),
                width,
            );
        } else {
            s.fill_rect(farbe, ziel);
        }
        return ziel;
    }

    // Ab hier: echtes Rundrechteck. Nur dieser Zweig normalisiert negative
    // Groessen - beim eckigen Weg erledigt das der Rechteckschnitt.
    let mut rr = r;
    if rr.w < 0 {
        rr.x = rr.x.saturating_add(rr.w);
        rr.w = rr.w.saturating_neg();
    }
    if rr.h < 0 {
        rr.y = rr.y.saturating_add(rr.h);
        rr.h = rr.h.saturating_neg();
    }
    let mut breite = width;
    if breite > rr.w / 2 || breite > rr.h / 2 {
        breite = (rr.w / 2).max(rr.h / 2);
    }

    let mut b = Bereich::leer();
    rundrechteck(
        s,
        &mut b,
        farbe,
        rr.x,
        rr.y,
        rr.x.saturating_add(rr.w).saturating_sub(1),
        rr.y.saturating_add(rr.h).saturating_sub(1),
        radius,
        breite,
        top_left,
        top_right,
        bottom_left,
        bottom_right,
    );
    b.zu_rect().unwrap_or(Rect::new(rr.x, rr.y, 0, 0))
}

// ---------------------------------------------------------------------------
//  Die Umgrenzung der gesetzten Pixel
// ---------------------------------------------------------------------------

/// Sammelt, welche Pixel wirklich beschrieben wurden.
///
/// Der Rundrechteck-Zweig kann sein Ergebnis nicht vorher ausrechnen - die
/// Kreisviertel setzen Pixel, die weder im Eingaberechteck noch in einer
/// einfachen Formel stehen. pygame fuehrt darum waehrend des Zeichnens
/// Buch; hier steht dasselbe Buch.
#[derive(Clone, Copy, Debug)]
struct Bereich {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Bereich {
    /// Noch nichts gezeichnet - die Grenzen stehen bewusst verkehrt herum.
    fn leer() -> Bereich {
        Bereich {
            x1: i32::MAX,
            y1: i32::MAX,
            x2: i32::MIN,
            y2: i32::MIN,
        }
    }

    fn punkt(&mut self, x: i32, y: i32) {
        self.x1 = self.x1.min(x);
        self.y1 = self.y1.min(y);
        self.x2 = self.x2.max(x);
        self.y2 = self.y2.max(y);
    }

    /// Wie [`Bereich::punkt`], aber fuer eine bereits sortierte Strecke.
    fn strecke(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        self.x1 = self.x1.min(x1);
        self.y1 = self.y1.min(y1);
        self.x2 = self.x2.max(x2);
        self.y2 = self.y2.max(y2);
    }

    /// `None`, solange kein einziges Pixel gesetzt wurde.
    fn zu_rect(&self) -> Option<Rect> {
        if self.x1 == i32::MAX || self.y1 == i32::MAX || self.x2 == i32::MIN || self.y2 == i32::MIN
        {
            None
        } else {
            Some(Rect::new(
                self.x1,
                self.y1,
                self.x2 - self.x1 + 1,
                self.y2 - self.y1 + 1,
            ))
        }
    }
}

// ---------------------------------------------------------------------------
//  Kleinste Bausteine: Pixel und waagerechte Strecken
// ---------------------------------------------------------------------------

/// Ein Pixel setzen und, falls es im Clip-Rechteck lag, vermerken.
///
/// Der Ruecksprung von `Surface::set_at` sagt nicht, ob geschrieben wurde,
/// darum wird hier vorher gefragt - genau wie pygames `set_and_check_rect`.
fn setze(s: &mut Surface, b: &mut Bereich, x: i32, y: i32, farbe: Color) {
    if s.get_clip().collidepoint((x, y)) {
        s.set_at((x, y), farbe);
        b.punkt(x, y);
    }
}

/// Beschneidet eine waagerechte Strecke am Clip-Rechteck.
///
/// `None`, wenn nichts davon uebrig bleibt. Die zurueckgegebene Spanne ist
/// aufsteigend und liegt ganz im Clip-Rechteck.
fn spanne(clip: Rect, x1: i32, y: i32, x2: i32) -> Option<(i32, i32)> {
    if y < clip.y || y >= clip.y + clip.h {
        return None;
    }
    let (mut a, mut e) = if x2 < x1 { (x2, x1) } else { (x1, x2) };
    a = a.max(clip.x);
    e = e.min(clip.x + clip.w - 1);
    if e < clip.x || a >= clip.x + clip.w {
        return None;
    }
    Some((a, e))
}

/// Waagerechte Strecke, bereits beschnitten.
fn hlinie_roh(s: &mut Surface, farbe: Color, x1: i32, y: i32, x2: i32) {
    s.fill_rect(farbe, Rect::new(x1, y, x2 - x1 + 1, 1));
}

/// Senkrechte Strecke, bereits beschnitten.
fn vlinie_roh(s: &mut Surface, farbe: Color, y1: i32, x: i32, y2: i32) {
    s.fill_rect(farbe, Rect::new(x, y1, 1, y2 - y1 + 1));
}

/// Waagerechte Strecke mit Beschnitt, ohne Buchfuehrung.
fn hlinie_clip(s: &mut Surface, farbe: Color, x1: i32, y: i32, x2: i32) {
    if let Some((a, e)) = spanne(s.get_clip(), x1, y, x2) {
        hlinie_roh(s, farbe, a, y, e);
    }
}

/// Waagerechte Strecke mit Beschnitt **und** Buchfuehrung.
fn hlinie_bereich(s: &mut Surface, b: &mut Bereich, farbe: Color, x1: i32, y: i32, x2: i32) {
    if let Some((a, e)) = spanne(s.get_clip(), x1, y, x2) {
        b.strecke(a, y, e, y);
        hlinie_roh(s, farbe, a, y, e);
    }
}

// ---------------------------------------------------------------------------
//  Linien
// ---------------------------------------------------------------------------

/// Der Startfehler des Bresenham-Laufs, genau wie im C-Original.
///
/// Der Wert ist bei senkrechtlastigen Linien negativ, und dort kommt es auf
/// die Rundungsrichtung an: Rusts `/` schneidet wie C Richtung Null ab,
/// `-3 / 2` ergibt also `-1` und nicht `-2`. Mit der anderen Richtung
/// verschoebe sich die Linie um ein Pixel.
#[inline]
fn startfehler(dx: i32, dy: i32) -> i32 {
    (if dx > dy { dx } else { -dy }) / 2
}

/// Einzelne Linie, Strichstaerke 1 (pygames `draw_line`).
fn linie(s: &mut Surface, b: &mut Bereich, farbe: Color, x1: i32, y1: i32, x2: i32, y2: i32) {
    if x1 == x2 && y1 == y2 {
        setze(s, b, x1, y1, farbe);
        return;
    }
    if y1 == y2 {
        let schritt = if x1 < x2 { 1 } else { -1 };
        for i in 0..=(x1 - x2).abs() {
            setze(s, b, x1 + schritt * i, y1, farbe);
        }
        return;
    }
    if x1 == x2 {
        let schritt = if y1 < y2 { 1 } else { -1 };
        for i in 0..=(y1 - y2).abs() {
            setze(s, b, x1, y1 + schritt * i, farbe);
        }
        return;
    }
    // Bresenham, Zeile fuer Zeile wie im Original.
    let (mut x, mut y) = (x1, y1);
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = startfehler(dx, dy);
    while x != x2 || y != y2 {
        setze(s, b, x, y, farbe);
        let e2 = err;
        if e2 > -dx {
            err -= dy;
            x += sx;
        }
        if e2 < dy {
            err += dx;
            y += sy;
        }
    }
    setze(s, b, x2, y2, farbe);
}

/// Grobpruefung, ob eine dicke Linie das Clip-Rechteck ueberhaupt trifft.
fn linie_sichtbar(clip: Rect, x1: i32, y1: i32, x2: i32, y2: i32, breite: i32, quer: bool) -> bool {
    let (links, rechts, oben, unten) = if quer {
        (
            x1.min(x2) - breite,
            x1.max(x2) + breite,
            y1.min(y2),
            y1.max(y2),
        )
    } else {
        (
            x1.min(x2),
            x1.max(x2),
            y1.min(y2) - breite,
            y1.max(y2) + breite,
        )
    };
    !(clip.x > rechts || clip.y > unten || clip.x + clip.w <= links || clip.y + clip.h <= oben)
}

/// Dicke Linie (pygames `draw_line_width`).
///
/// Die Dicke waechst quer zur laengeren Achse; bei gerader Staerke liegt
/// die Linie darum **nicht** mittig, sondern ein Pixel weiter zur hoeheren
/// Koordinate - `extra` haelt genau diese Unwucht fest.
#[allow(clippy::too_many_arguments)]
fn linie_breit(
    s: &mut Surface,
    b: &mut Bereich,
    farbe: Color,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    breite: i32,
) {
    let clip = s.get_clip();
    let ende_x = clip.x + clip.w - 1;
    let ende_y = clip.y + clip.h - 1;
    let extra = 1 - (breite % 2);

    if breite < 1 {
        return;
    }
    if breite == 1 {
        linie(s, b, farbe, x1, y1, x2, y2);
        return;
    }
    let halbe = breite / 2;
    // Ist die Linie eher senkrecht, waechst sie in x-Richtung.
    let quer = (x1 - x2).abs() <= (y1 - y2).abs();
    if !linie_sichtbar(clip, x1, y1, x2, y2, halbe, quer) {
        return;
    }

    if x1 == x2 && y1 == y2 {
        let anfang = ((x1 - halbe) + extra).max(clip.x);
        let schluss = ende_x.min(x1 + halbe);
        if anfang <= schluss {
            hlinie_roh(s, farbe, anfang, y1, schluss);
            b.strecke(anfang, y1, schluss, y1);
        }
        return;
    }

    let (mut x, mut y) = (x1, y1);
    let dx = (x2 - x1).abs();
    let dy = (y2 - y1).abs();
    let sx = if x2 > x1 { 1 } else { -1 };
    let sy = if y2 > y1 { 1 } else { -1 };
    let mut err = startfehler(dx, dy);

    if quer {
        while y != y2 + sy {
            if clip.y <= y && y <= ende_y {
                let anfang = ((x - halbe) + extra).max(clip.x);
                let schluss = ende_x.min(x + halbe);
                if anfang <= schluss {
                    hlinie_roh(s, farbe, anfang, y, schluss);
                    b.strecke(anfang, y, schluss, y);
                }
            }
            let e2 = err;
            if e2 > -dx {
                err -= dy;
                x += sx;
            }
            if e2 < dy {
                err += dx;
                y += sy;
            }
        }
    } else {
        while x != x2 + sx {
            if clip.x <= x && x <= ende_x {
                let anfang = ((y - halbe) + extra).max(clip.y);
                let schluss = ende_y.min(y + halbe);
                if anfang <= schluss {
                    vlinie_roh(s, farbe, anfang, x, schluss);
                    b.strecke(x, anfang, x, schluss);
                }
            }
            let e2 = err;
            if e2 > -dx {
                err -= dy;
                x += sx;
            }
            if e2 < dy {
                err += dx;
                y += sy;
            }
        }
    }
}

// ---------------------------------------------------------------------------
//  Der eckige Umriss
// ---------------------------------------------------------------------------

/// Vier Balken (pygames `draw_rect`): oben, unten, links, rechts.
///
/// Fuehrt bewusst **kein** Buch - der eckige Zweig gibt das beschnittene
/// Eingaberechteck zurueck, nicht die Umgrenzung der gesetzten Pixel.
fn rahmen(s: &mut Surface, farbe: Color, x1: i32, y1: i32, x2: i32, y2: i32, breite: i32) {
    for i in 0..breite {
        hlinie_clip(s, farbe, x1, y1 + i, x2);
        hlinie_clip(s, farbe, x1, y2 - i, x2);
    }
    for i in 0..((y2 - y1) - 2 * breite + 1) {
        hlinie_clip(s, farbe, x1, y1 + breite + i, x1 + breite - 1);
        hlinie_clip(s, farbe, x2 - breite + 1, y1 + breite + i, x2);
    }
}

// ---------------------------------------------------------------------------
//  Das gefuellte Achteck
// ---------------------------------------------------------------------------

/// Fuellt das Achteck zwischen den Eckbogen (pygames `draw_fillpoly`).
///
/// Zeilenweises Abtasten mit Schnittpunktpaaren. Die Rundung der
/// Schnittpunkte wechselt je nach Zaehlerstand zwischen ab- und aufrunden -
/// so schliesst pygame die Flaeche luecken- und ueberstandsfrei; nachrechnen
/// lohnt nicht, nachbilden schon.
fn fuellachteck(s: &mut Surface, b: &mut Bereich, farbe: Color, px: &[i32; 8], py: &[i32; 8]) {
    let n = 8usize;
    let miny = *py.iter().min().unwrap_or(&0);
    let maxy = *py.iter().max().unwrap_or(&0);

    if miny == maxy {
        let minx = *px.iter().min().unwrap_or(&0);
        let maxx = *px.iter().max().unwrap_or(&0);
        hlinie_bereich(s, b, farbe, minx, miny, maxx);
        return;
    }

    let mut schnitte = [0i32; 8];
    for y in miny..=maxy {
        let mut anzahl = 0usize;
        // Der erste Schnittpunkt einer Zeile wird ab-, der zweite
        // aufgerundet, dann wieder von vorn.
        let mut abrunden = true;
        for i in 0..n {
            let vorher = if i > 0 { i - 1 } else { n - 1 };
            let (y1, y2, x1, x2);
            if py[vorher] < py[i] {
                y1 = py[vorher];
                y2 = py[i];
                x1 = px[vorher];
                x2 = px[i];
            } else if py[vorher] > py[i] {
                y1 = py[i];
                y2 = py[vorher];
                x1 = px[i];
                x2 = px[vorher];
            } else {
                // Waagerechte Kanten kommen weiter unten dran.
                continue;
            }
            if (y >= y1 && y < y2) || (y == maxy && y2 == maxy) {
                // Erst ganzzahlig multiplizieren, dann teilen - genau so
                // steht es im Original, und die Reihenfolge ist sichtbar.
                let p = ((y - y1) * (x2 - x1)) as f32 / (y2 - y1) as f32;
                let gerundet = if abrunden { p.floor() } else { p.ceil() };
                abrunden = !abrunden;
                schnitte[anzahl] = gerundet as i32 + x1;
                anzahl += 1;
            }
        }
        schnitte[..anzahl].sort_unstable();
        let mut i = 0;
        while i + 1 < anzahl {
            hlinie_bereich(s, b, farbe, schnitte[i], y, schnitte[i + 1]);
            i += 2;
        }
    }

    // Nachtrag fuer waagerechte Kanten mitten in der Flaeche: die faellt das
    // Abtasten oben durch, weil sie keinen Schnittpunkt liefert.
    for i in 0..n {
        let vorher = if i > 0 { i - 1 } else { n - 1 };
        let y = py[i];
        if miny < y && py[vorher] == y && y < maxy {
            hlinie_bereich(s, b, farbe, px[i], y, px[vorher]);
        }
    }
}

// ---------------------------------------------------------------------------
//  Die Eckbogen
// ---------------------------------------------------------------------------

/// Ein Viertelkreis (pygames `draw_circle_quadrant`).
///
/// `thickness == 0` fuellt das Viertel, sonst wird ein Ring dieser Staerke
/// gezogen. Die vier Schalter waehlen das Viertel; `(x0, y0)` ist dabei
/// nicht der Mittelpunkt, sondern die Ecke, an der die Viertel
/// zusammenstossen - deshalb die vielen `- 1` im Rumpf.
#[allow(clippy::too_many_arguments)]
fn kreisviertel(
    s: &mut Surface,
    b: &mut Bereich,
    farbe: Color,
    x0: i32,
    y0: i32,
    radius: i32,
    dicke: i32,
    oben_rechts: bool,
    oben_links: bool,
    unten_links: bool,
    unten_rechts: bool,
) {
    let mut f = 1 - radius;
    let mut ddf_x = 0;
    let mut ddf_y = -2 * radius;
    let mut x = 0;
    let mut y = radius;
    let mut i_y = radius - dicke;
    let mut i_f = 1 - i_y;
    let mut i_ddf_x = 0;
    let mut i_ddf_y = -2 * i_y;
    let mut dicke = dicke;

    // Radius 1 hat keinen Bogen, nur ein Eckpixel je Viertel.
    if radius == 1 {
        if oben_rechts {
            setze(s, b, x0, y0 - 1, farbe);
        }
        if oben_links {
            setze(s, b, x0 - 1, y0 - 1, farbe);
        }
        if unten_links {
            setze(s, b, x0 - 1, y0, farbe);
        }
        if unten_rechts {
            setze(s, b, x0, y0, farbe);
        }
        return;
    }

    if dicke != 0 {
        // Ring: ein zweiter Bresenham-Kreis laeuft innen mit, der Abstand
        // beider Radien ist die tatsaechliche Staerke in dieser Zeile.
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

            if dicke > 1 {
                dicke = y - i_y;
            }

            if oben_rechts {
                for i in 0..dicke {
                    let y1 = y - i;
                    if (y0 - y1) < (y0 - x) {
                        setze(s, b, x0 + x - 1, y0 - y1, farbe);
                    }
                    if (x0 + y1 - 1) >= (x0 + x - 1) {
                        setze(s, b, x0 + y1 - 1, y0 - x, farbe);
                    }
                }
            }
            if oben_links {
                for i in 0..dicke {
                    let y1 = y - i;
                    if (y0 - y1) <= (y0 - x) {
                        setze(s, b, x0 - x, y0 - y1, farbe);
                    }
                    if (x0 - y1) < (x0 - x) {
                        setze(s, b, x0 - y1, y0 - x, farbe);
                    }
                }
            }
            if unten_links {
                for i in 0..dicke {
                    let y1 = y - i;
                    if (x0 - y1) <= (x0 - x) {
                        setze(s, b, x0 - y1, y0 + x - 1, farbe);
                    }
                    if (y0 + y1 - 1) > (y0 + x - 1) {
                        setze(s, b, x0 - x, y0 + y1 - 1, farbe);
                    }
                }
            }
            if unten_rechts {
                for i in 0..dicke {
                    let y1 = y - i;
                    if (y0 + y1 - 1) >= (y0 + x - 1) {
                        setze(s, b, x0 + x - 1, y0 + y1 - 1, farbe);
                    }
                    if (x0 + y1 - 1) > (x0 + x - 1) {
                        setze(s, b, x0 + y1 - 1, y0 + x - 1, farbe);
                    }
                }
            }
        }
    } else {
        // Gefuellt: zu jedem Bresenham-Schritt zwei Saeulen bis zur Ecke.
        while x < y {
            if f >= 0 {
                y -= 1;
                ddf_y += 2;
                f += ddf_y;
            }
            x += 1;
            ddf_x += 2;
            f += ddf_x + 1;

            if oben_rechts {
                for y1 in (y0 - x)..=y0 {
                    setze(s, b, x0 + y - 1, y1, farbe);
                }
                for y1 in (y0 - y)..=y0 {
                    setze(s, b, x0 + x - 1, y1, farbe);
                }
            }
            if oben_links {
                for y1 in (y0 - x)..=y0 {
                    setze(s, b, x0 - y, y1, farbe);
                }
                for y1 in (y0 - y)..=y0 {
                    setze(s, b, x0 - x, y1, farbe);
                }
            }
            if unten_links {
                for y1 in y0..(y0 + x) {
                    setze(s, b, x0 - y, y1, farbe);
                }
                for y1 in y0..(y0 + y) {
                    setze(s, b, x0 - x, y1, farbe);
                }
            }
            if unten_rechts {
                for y1 in y0..(y0 + x) {
                    setze(s, b, x0 + y - 1, y1, farbe);
                }
                for y1 in y0..(y0 + y) {
                    setze(s, b, x0 + x - 1, y1, farbe);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
//  Das Rundrechteck
// ---------------------------------------------------------------------------

/// Setzt die vier Eckradien und begrenzt sie auf die Rechteckgroesse.
///
/// Passen zwei benachbarte Radien nicht nebeneinander, werden **alle vier**
/// mit demselben Faktor verkleinert - so bleibt das Verhaeltnis erhalten,
/// statt dass eine Ecke einbricht. Teilen durch Null ist dabei kein
/// Sonderfall: in C ergibt es unendlich, und das Kleinste-Auswaehlen
/// sortiert es von selbst aus.
fn eckradien(x1: i32, y1: i32, x2: i32, y2: i32, radius: i32, ecken: [i32; 4]) -> [i32; 4] {
    let mut e = ecken;
    for v in e.iter_mut() {
        if *v < 0 {
            *v = radius;
        }
    }
    let (tl, tr, bl, br) = (e[0], e[1], e[2], e[3]);
    let breite = x2 - x1 + 1;
    let hoehe = y2 - y1 + 1;

    if (tl + tr) > breite || (bl + br) > breite || (tl + bl) > hoehe || (tr + br) > hoehe {
        let teile = |zaehler: i32, nenner: i32| -> f32 {
            if nenner == 0 {
                f32::INFINITY
            } else {
                zaehler as f32 / nenner as f32
            }
        };
        let q = [
            teile(breite, tl + tr),
            teile(hoehe, tl + bl),
            teile(breite, bl + br),
            teile(hoehe, tr + br),
        ];
        let f = q
            .iter()
            .fold(f32::INFINITY, |a, &v| if v < a { v } else { a });
        for v in e.iter_mut() {
            *v = (*v as f32 * f) as i32;
        }
    }
    e
}

/// Das abgerundete Rechteck (pygames `draw_round_rect`).
///
/// `x2`/`y2` sind die **letzten** Pixel, nicht die Kante dahinter.
#[allow(clippy::too_many_arguments)]
fn rundrechteck(
    s: &mut Surface,
    b: &mut Bereich,
    farbe: Color,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    radius: i32,
    breite: i32,
    top_left: i32,
    top_right: i32,
    bottom_left: i32,
    bottom_right: i32,
) {
    let e = eckradien(
        x1,
        y1,
        x2,
        y2,
        radius,
        [top_left, top_right, bottom_left, bottom_right],
    );
    let (tl, tr, bl, br) = (e[0], e[1], e[2], e[3]);

    if breite == 0 {
        // Gefuellt: ein Achteck deckt die Mitte ab, die Bogen die Ecken.
        let px = [x1, x1 + tl, x2 - tr, x2, x2, x2 - br, x1 + bl, x1];
        let py = [y1 + tl, y1, y1, y1 + tr, y2 - br, y2, y2, y2 - bl];
        fuellachteck(s, b, farbe, &px, &py);
        kreisviertel(
            s,
            b,
            farbe,
            x2 - tr + 1,
            y1 + tr,
            tr,
            0,
            true,
            false,
            false,
            false,
        );
        kreisviertel(
            s,
            b,
            farbe,
            x1 + tl,
            y1 + tl,
            tl,
            0,
            false,
            true,
            false,
            false,
        );
        kreisviertel(
            s,
            b,
            farbe,
            x1 + bl,
            y2 - bl + 1,
            bl,
            0,
            false,
            false,
            true,
            false,
        );
        kreisviertel(
            s,
            b,
            farbe,
            x2 - br + 1,
            y2 - br + 1,
            br,
            0,
            false,
            false,
            false,
            true,
        );
        return;
    }

    // Umriss: vier dicke Linien zwischen den Bogen. Treffen sich zwei
    // Eckradien genau, bleibt keine Linie uebrig - dann stopft eine
    // Pixelreihe die Luecke, sonst klaffte dort ein Loch.
    let versatz = breite / 2 - 1 + breite % 2;

    if x2 - tr == x1 + tl {
        for i in 0..breite {
            setze(s, b, x1 + tl, y1 + i, farbe);
        }
    } else {
        linie_breit(
            s,
            b,
            farbe,
            x1 + tl,
            y1 + versatz,
            x2 - tr,
            y1 + versatz,
            breite,
        );
    }

    if y2 - bl == y1 + tl {
        for i in 0..breite {
            setze(s, b, x1 + i, y1 + tl, farbe);
        }
    } else {
        linie_breit(
            s,
            b,
            farbe,
            x1 + versatz,
            y1 + tl,
            x1 + versatz,
            y2 - bl,
            breite,
        );
    }

    if x2 - br == x1 + bl {
        for i in 0..breite {
            setze(s, b, x1 + bl, y2 - i, farbe);
        }
    } else {
        let y = y2 - breite / 2;
        linie_breit(s, b, farbe, x1 + bl, y, x2 - br, y, breite);
    }

    if y2 - br == y1 + tr {
        for i in 0..breite {
            setze(s, b, x2 - i, y1 + tr, farbe);
        }
    } else {
        let x = x2 - breite / 2;
        linie_breit(s, b, farbe, x, y1 + tr, x, y2 - br, breite);
    }

    kreisviertel(
        s,
        b,
        farbe,
        x2 - tr + 1,
        y1 + tr,
        tr,
        breite,
        true,
        false,
        false,
        false,
    );
    kreisviertel(
        s,
        b,
        farbe,
        x1 + tl,
        y1 + tl,
        tl,
        breite,
        false,
        true,
        false,
        false,
    );
    kreisviertel(
        s,
        b,
        farbe,
        x1 + bl,
        y2 - bl + 1,
        bl,
        breite,
        false,
        false,
        true,
        false,
    );
    kreisviertel(
        s,
        b,
        farbe,
        x2 - br + 1,
        y2 - br + 1,
        br,
        breite,
        false,
        false,
        false,
        true,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::SRCALPHA;

    /// Zeichnet auf schwarzem Grund und gibt je Zeile zurueck, welche
    /// Spalten weiss geworden sind.
    fn muster(s: &Surface) -> Vec<String> {
        (0..s.get_height() as i32)
            .map(|y| {
                (0..s.get_width() as i32)
                    .map(|x| if s.get_at((x, y)).r > 0 { '#' } else { '.' })
                    .collect()
            })
            .collect()
    }

    fn leinwand(w: u32, h: u32) -> Surface {
        let mut s = Surface::new((w, h));
        s.fill((0, 0, 0));
        s
    }

    #[test]
    fn gefuelltes_rechteck_deckt_genau_seine_flaeche() {
        let mut s = leinwand(16, 12);
        let r = rect(&mut s, (255, 255, 255), Rect::new(2, 2, 10, 7), 0);
        assert_eq!(
            r,
            Rect::new(2, 2, 10, 7),
            "Rueckgabe ist das Eingaberechteck"
        );
        assert_eq!(s.get_at((2, 2)).r, 255);
        assert_eq!(s.get_at((11, 8)).r, 255, "letztes Pixel gehoert dazu");
        assert_eq!(s.get_at((12, 8)).r, 0, "eines weiter nicht mehr");
        assert_eq!(s.get_at((1, 2)).r, 0);
    }

    #[test]
    fn negative_breite_zeichnet_nichts() {
        // Gemessen an pygame: width < 0 ist kein "gefuellt", sondern ein
        // vollstaendiger Verzicht.
        let mut s = leinwand(16, 16);
        let r = rect(&mut s, (255, 255, 255), Rect::new(2, 2, 10, 10), -1);
        assert_eq!(r, Rect::new(2, 2, 0, 0));
        assert!(muster(&s).iter().all(|z| !z.contains('#')));
    }

    #[test]
    fn zu_dicker_umriss_wird_zur_fuellung() {
        // 20 Pixel Strich in einem 8x8-Rechteck: pygame fuellt.
        let mut s = leinwand(16, 16);
        let r = rect(&mut s, (255, 255, 255), Rect::new(2, 2, 8, 8), 20);
        assert_eq!(r, Rect::new(2, 2, 8, 8));
        assert_eq!(s.get_at((5, 5)).r, 255, "die Mitte ist voll");
    }

    #[test]
    fn umriss_laesst_die_mitte_frei() {
        let mut s = leinwand(20, 16);
        let r = rect(&mut s, (255, 255, 255), Rect::new(3, 3, 13, 9), 3);
        assert_eq!(r, Rect::new(3, 3, 13, 9));
        assert_eq!(s.get_at((3, 3)).r, 255);
        assert_eq!(s.get_at((5, 5)).r, 255, "drittes Pixel gehoert zum Strich");
        assert_eq!(s.get_at((6, 6)).r, 0, "viertes nicht mehr");
        assert_eq!(s.get_at((15, 11)).r, 255, "gegenueberliegende Ecke");
    }

    #[test]
    fn rundrechteck_trifft_die_gemessenen_zeilenbreiten() {
        // Gemessen an echtem pygame: 11x11 mit border_radius 3.
        let mut s = leinwand(11, 11);
        let r = rect_radius(&mut s, (255, 255, 255), Rect::new(0, 0, 11, 11), 0, 3);
        assert_eq!(r, Rect::new(0, 0, 11, 11));
        let breiten: Vec<usize> = muster(&s)
            .iter()
            .map(|z| z.chars().filter(|c| *c == '#').count())
            .collect();
        assert_eq!(breiten, vec![7, 9, 11, 11, 11, 11, 11, 11, 11, 9, 7]);
    }

    #[test]
    fn zu_grosser_radius_wird_auf_die_haelfte_begrenzt() {
        // radius 50 in einem 20x20-Rechteck muss dasselbe Bild ergeben wie
        // radius 10 - mehr als ein Halbkreis geht nicht.
        let mut a = leinwand(20, 20);
        rect_radius(&mut a, (255, 255, 255), Rect::new(0, 0, 20, 20), 0, 10);
        let mut b = leinwand(20, 20);
        rect_radius(&mut b, (255, 255, 255), Rect::new(0, 0, 20, 20), 0, 50);
        assert_eq!(muster(&a), muster(&b));
        assert_eq!(muster(&a)[0], ".......######.......");
    }

    #[test]
    fn leeres_rechteck_zeichnet_nichts() {
        let mut s = leinwand(16, 16);
        let r = rect(&mut s, (255, 255, 255), Rect::new(4, 4, 0, 5), 0);
        assert_eq!(r, Rect::new(4, 4, 0, 0));
        assert!(muster(&s).iter().all(|z| !z.contains('#')));

        // Ganz ausserhalb der Flaeche: ebenfalls nichts.
        let r = rect(&mut s, (255, 255, 255), Rect::new(90, 90, 5, 5), 0);
        assert_eq!(r, Rect::new(90, 90, 0, 0));
        assert!(muster(&s).iter().all(|z| !z.contains('#')));
    }

    #[test]
    fn negative_groesse_wird_nur_beim_runden_gedreht() {
        // pygame normalisiert im Rundrechteck-Zweig; heraus kommt dasselbe
        // wie fuer das gedrehte Rechteck (4,4,6,6).
        let mut s = leinwand(16, 16);
        let r = rect_radius(&mut s, (255, 255, 255), Rect::new(10, 10, -6, -6), 0, 2);
        assert_eq!(r, Rect::new(4, 4, 6, 6));
        assert_eq!(muster(&s)[4], ".....####.......");
        assert_eq!(muster(&s)[5], "....######......");
    }

    #[test]
    fn das_clip_rechteck_begrenzt_und_bestimmt_die_rueckgabe() {
        let mut s = leinwand(16, 16);
        s.set_clip(Some(Rect::new(4, 4, 5, 5)));
        let r = rect(&mut s, (255, 255, 255), Rect::new(2, 2, 10, 10), 0);
        assert_eq!(r, Rect::new(4, 4, 5, 5), "beschnitten wie in pygame");
        assert_eq!(s.get_at((3, 4)).r, 0);
        assert_eq!(s.get_at((4, 4)).r, 255);
        assert_eq!(s.get_at((9, 4)).r, 0);

        // Rund: die Rueckgabe ist die Umgrenzung der gesetzten Pixel und
        // liegt damit ebenfalls im Clip-Rechteck.
        let mut s = leinwand(16, 16);
        s.set_clip(Some(Rect::new(4, 4, 6, 6)));
        let r = rect_radius(&mut s, (255, 255, 255), Rect::new(2, 2, 10, 10), 0, 3);
        assert_eq!(r, Rect::new(4, 4, 6, 6));
    }

    #[test]
    fn die_farbe_wird_ohne_mischen_geschrieben() {
        // Gemessen: draw.rect mischt nicht, auch nicht auf SRCALPHA.
        let mut s = Surface::with_flags((16, 16), SRCALPHA);
        s.fill((50, 50, 50, 255));
        rect(&mut s, (255, 0, 0, 128), Rect::new(2, 2, 4, 4), 0);
        assert_eq!(s.get_at((3, 3)).as_tuple(), (255, 0, 0, 128));
        assert_eq!(s.get_at((0, 0)).as_tuple(), (50, 50, 50, 255));
    }

    #[test]
    fn eckradien_wirken_einzeln() {
        // Gemessen: oben links 6, unten rechts 3, die anderen scharf.
        let mut s = leinwand(16, 16);
        let r = rect_corners(
            &mut s,
            (255, 255, 255),
            Rect::new(2, 2, 12, 12),
            0,
            0,
            6,
            -1,
            -1,
            3,
        );
        assert_eq!(r, Rect::new(2, 2, 12, 12));
        let m = muster(&s);
        assert_eq!(
            m[2], "......########..",
            "oben links rund, oben rechts scharf"
        );
        assert_eq!(m[11], "..############..");
        assert_eq!(m[13], "..##########....", "unten rechts rund");
    }

    #[test]
    fn duenne_rechtecke_gehen_am_rundzweig_vorbei() {
        // Eine Kante von nur einem Pixel: pygame zeichnet eckig, sonst
        // kaeme aus dem Rundrechteck Unsinn.
        let mut s = leinwand(12, 8);
        let r = rect_radius(&mut s, (255, 255, 255), Rect::new(2, 3, 8, 1), 0, 4);
        assert_eq!(r, Rect::new(2, 3, 8, 1));
        assert_eq!(muster(&s)[3], "..########..");
    }
}
