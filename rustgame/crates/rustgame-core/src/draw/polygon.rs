//! polygon.rs
//! ==========
//! `pygame.draw.polygon` - 172 Aufrufe in der Sammlung.
//!
//! Nachgebildet ist der Rechenweg aus `src_c/draw.c` von pygame-ce 2.5.7.
//! Vier Eigenheiten, die man nicht erraet, sondern nachmessen muss:
//!
//! * **`width != 0` ist gar kein Polygon**, sondern ein geschlossener
//!   Linienzug: pygame reicht den Aufruf an `lines` weiter. Damit erbt der
//!   Umriss auch dessen Kerben an den Knicken.
//! * **`width < 0` zeichnet nichts.** Nur `width == 0` fuellt.
//! * **Dreiecke gehen einen eigenen Weg.** Bei genau drei Punkten ruft
//!   pygame `draw_filltri` auf - eine Abtastung mit Fliesskomma-Steigungen -,
//!   bei jeder anderen Punktzahl `draw_fillpoly` mit ganzzahligen
//!   Schnittpunkten. Die beiden Verfahren liefern **nicht** dasselbe Bild;
//!   wer alles durch einen Weg schickt, hat bei Dreiecken Pixelabstand.
//! * **Die Rundung der Schnittpunkte wechselt.** In `draw_fillpoly` wird der
//!   erste Schnittpunkt einer Zeile ab-, der zweite aufgerundet, dann wieder
//!   von vorn - gezaehlt in der Reihenfolge, in der die Kanten *gefunden*
//!   werden, nicht in der sortierten. So schliesst pygame die Flaeche
//!   luecken- und ueberstandsfrei.
//!
//! Gefuellt wird nach der Geraden-Ungeraden-Regel: pro Zeile werden die
//! Schnittpunkte mit allen Kanten gesammelt, sortiert und paarweise
//! ausgemalt. Einspringende Ecken und Selbstueberschneidungen fallen damit
//! von selbst richtig aus.
//!
//! Die Farbe wird roh geschrieben, ohne Alpha-Mischen - wie bei allen
//! `draw`-Funktionen (gemessen).

use crate::color::{Color, IntoColor};
use crate::draw::line::{hlinie_bereich, lines, Bereich};
use crate::rect::Rect;
use crate::surface::Surface;

/// Zeichnet ein Vieleck durch alle Punkte.
///
/// `width == 0` fuellt, `width > 0` zieht den Umriss in dieser Staerke,
/// `width < 0` zeichnet nichts. Zurueck kommt die Umgrenzung der
/// tatsaechlich gesetzten Pixel, auf das Clip-Rechteck beschnitten.
///
/// Weniger als drei Punkte kann nur der Umriss-Weg (als Linienzug);
/// gefuellt wirft pygame dort einen `ValueError`, den es in dieser Signatur
/// nicht gibt - hier bleibt die Flaeche dann unberuehrt.
pub fn polygon(s: &mut Surface, c: impl IntoColor, pts: &[(i32, i32)], width: i32) -> Rect {
    // Alles ausser der Fuellung ist in pygame ein geschlossener Linienzug -
    // auch das negative `width`, das dort dann gar nichts zeichnet.
    if width != 0 {
        return lines(s, c, true, pts, width);
    }
    let Some(&(x0, y0)) = pts.first() else {
        return Rect::new(0, 0, 0, 0);
    };
    if pts.len() < 3 {
        return Rect::new(x0, y0, 0, 0);
    }

    let farbe = c.into_color();
    let mut bereich = Bereich::leer();
    if pts.len() == 3 {
        fuelldreieck(s, &mut bereich, farbe, pts);
    } else {
        fuellpolygon(s, &mut bereich, farbe, pts);
    }
    bereich.zu_rect().unwrap_or(Rect::new(x0, y0, 0, 0))
}

// ---------------------------------------------------------------------------
//  Dreiecke
// ---------------------------------------------------------------------------

/// Gefuelltes Dreieck (pygames `draw_filltri`).
///
/// Die drei Punkte werden nach `y` sortiert, danach laeuft je Zeile eine
/// Spanne zwischen der langen Kante und der jeweils kuerzeren. Gerechnet
/// wird mit `f32`-Steigungen und Abschneiden Richtung Null - die Stufen
/// liegen darum anders als bei [`fuellpolygon`].
///
/// Das `1e-17` im Nenner ist pygames Bremse gegen die Division durch Null
/// bei waagerechten Kanten; der Zaehler ist dann ohnehin Null.
fn fuelldreieck(s: &mut Surface, b: &mut Bereich, farbe: Color, pts: &[(i32, i32)]) {
    let (mut p0, mut p1, mut p2) = (pts[0], pts[1], pts[2]);

    if p1.1 < p0.1 {
        std::mem::swap(&mut p1, &mut p0);
    }
    if p2.1 < p1.1 {
        std::mem::swap(&mut p1, &mut p2);
        if p1.1 < p0.1 {
            std::mem::swap(&mut p1, &mut p0);
        }
    }
    // Entartet: alle drei auf einer Zeile, die ersten beiden uebereinander.
    if p0.1 == p1.1 && p1.1 == p2.1 && p0.0 == p1.0 && p1.0 != p2.0 {
        std::mem::swap(&mut p1, &mut p2);
    }

    let steigung = |von: (i32, i32), nach: (i32, i32)| -> f32 {
        ((nach.0 - von.0) as f64 / ((nach.1 - von.1) as f64 + 1e-17)) as f32
    };
    let d1 = steigung(p0, p2); // die lange Kante
    let d2 = steigung(p0, p1); // oberes Stueck
    let d3 = steigung(p1, p2); // unteres Stueck

    for y in p0.1..=p2.1 {
        let x1 = p0.0 + ((y - p0.1) as f32 * d1) as i32;
        let x2 = if y < p1.1 {
            p0.0 + ((y - p0.1) as f32 * d2) as i32
        } else {
            p1.0 + ((y - p1.1) as f32 * d3) as i32
        };
        hlinie_bereich(s, b, farbe, x1, y, x2);
    }
}

// ---------------------------------------------------------------------------
//  Alle anderen Vielecke
// ---------------------------------------------------------------------------

/// Gefuelltes Vieleck (pygames `draw_fillpoly`), zeilenweises Abtasten.
///
/// Je Zeile werden die Schnittpunkte mit allen nicht-waagerechten Kanten
/// gesammelt, sortiert und paarweise ausgemalt. Die untere Kante zaehlt
/// nicht mit (`y < y2`), sonst gaebe es an jeder Ecke einen Schnittpunkt zu
/// viel; nur die letzte Zeile ist ausgenommen.
fn fuellpolygon(s: &mut Surface, b: &mut Bereich, farbe: Color, pts: &[(i32, i32)]) {
    let n = pts.len();
    let miny = pts.iter().map(|p| p.1).min().unwrap_or(0);
    let maxy = pts.iter().map(|p| p.1).max().unwrap_or(0);

    if miny == maxy {
        // Nur eine Zeile hoch: eine Strecke von links nach rechts.
        let minx = pts.iter().map(|p| p.0).min().unwrap_or(0);
        let maxx = pts.iter().map(|p| p.0).max().unwrap_or(0);
        hlinie_bereich(s, b, farbe, minx, miny, maxx);
        return;
    }

    let mut schnitte = vec![0i32; n];
    for y in miny..=maxy {
        let mut anzahl = 0usize;
        // Der erste Schnittpunkt einer Zeile wird ab-, der zweite
        // aufgerundet, dann wieder von vorn (im Original `n_intersections % 2`).
        let mut abrunden = true;
        for i in 0..n {
            let vorher = if i > 0 { i - 1 } else { n - 1 };
            let (y1, y2, x1, x2);
            if pts[vorher].1 < pts[i].1 {
                y1 = pts[vorher].1;
                y2 = pts[i].1;
                x1 = pts[vorher].0;
                x2 = pts[i].0;
            } else if pts[vorher].1 > pts[i].1 {
                y1 = pts[i].1;
                y2 = pts[vorher].1;
                x1 = pts[i].0;
                x2 = pts[vorher].0;
            } else {
                // Waagerechte Kanten kommen ganz unten dran.
                continue;
            }
            if (y >= y1 && y < y2) || (y == maxy && y2 == maxy) {
                // Erst ganzzahlig multiplizieren, dann durch die Hoehe
                // teilen - die Reihenfolge steht so im Original.
                let p = ((y - y1) * (x2 - x1)) as f32 / (y2 - y1) as f32;
                // Abwechselnd ab- und aufrunden, gezaehlt in der
                // Fundreihenfolge - nicht in der sortierten. Das ist
                // pygames Trick gegen Luecken und Ueberstaende.
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

    // Nachtrag: waagerechte Kanten mitten in der Flaeche liefern keinen
    // Schnittpunkt und fielen sonst durchs Raster.
    for i in 0..n {
        let vorher = if i > 0 { i - 1 } else { n - 1 };
        let y = pts[i].1;
        if miny < y && pts[vorher].1 == y && y < maxy {
            hlinie_bereich(s, b, farbe, pts[i].0, y, pts[vorher].0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::circle::pruefhilfe::{
        gleich_wie_pygame, schwarz as leinwand, zeichenbild as muster,
    };
    use crate::surface::SRCALPHA;

    const WEISS: (u8, u8, u8, u8) = (255, 255, 255, 255);

    const DREIECK: [(i32, i32); 3] = [(2, 18), (10, 2), (18, 18)];

    // ----- die vier Referenzfaelle aus tools/cases.py ---------------------

    #[test]
    fn referenz_polygon_dreieck() {
        let mut s = leinwand(20, 20);
        assert_eq!(polygon(&mut s, WEISS, &DREIECK, 0), Rect::new(2, 2, 17, 17));
        gleich_wie_pygame("polygon_dreieck", &s);
    }

    #[test]
    fn referenz_polygon_umriss() {
        let mut s = leinwand(20, 20);
        assert_eq!(polygon(&mut s, WEISS, &DREIECK, 1), Rect::new(2, 2, 17, 17));
        gleich_wie_pygame("polygon_umriss", &s);
    }

    #[test]
    fn referenz_polygon_viereck() {
        let mut s = leinwand(20, 20);
        let pts = [(3, 3), (16, 5), (14, 17), (5, 15)];
        assert_eq!(polygon(&mut s, WEISS, &pts, 0), Rect::new(3, 3, 14, 15));
        gleich_wie_pygame("polygon_viereck", &s);
    }

    #[test]
    fn referenz_polygon_konkav() {
        let mut s = leinwand(24, 24);
        let pts = [(2, 2), (21, 2), (21, 21), (12, 10), (2, 21)];
        assert_eq!(polygon(&mut s, WEISS, &pts, 0), Rect::new(2, 2, 20, 20));
        gleich_wie_pygame("polygon_konkav", &s);
    }

    // ----- Randfaelle -----------------------------------------------------

    #[test]
    fn das_dreieck_trifft_die_gemessenen_zeilenbreiten() {
        // Gemessen an echtem pygame-ce 2.5.7.
        let mut s = leinwand(20, 20);
        let r = polygon(&mut s, (255, 255, 255), &[(2, 18), (10, 2), (18, 18)], 0);
        assert_eq!(r, Rect::new(2, 2, 17, 17));
        let m = muster(&s);
        assert_eq!(m[2], "..........#.........");
        assert_eq!(m[4], ".........###........");
        assert_eq!(m[18], "..#################.");
        assert_eq!(
            m[19], "....................",
            "die Spitze bleibt unten offen"
        );
    }

    #[test]
    fn das_viereck_geht_einen_anderen_weg_als_das_dreieck() {
        // Vier Punkte -> Schnittpunktverfahren. Gemessen an pygame.
        let mut s = leinwand(20, 20);
        let r = polygon(
            &mut s,
            (255, 255, 255),
            &[(3, 3), (16, 5), (14, 17), (5, 15)],
            0,
        );
        assert_eq!(r, Rect::new(3, 3, 14, 15));
        let m = muster(&s);
        assert_eq!(m[3], "...#................");
        assert_eq!(m[4], "...########.........");
        assert_eq!(m[16], "..........#####.....");
    }

    #[test]
    fn einspringende_ecken_bleiben_ausgespart() {
        // Die Kerbe des konkaven Polygons muss frei bleiben - das prueft
        // die Gerade-Ungerade-Regel.
        let mut s = leinwand(24, 24);
        let r = polygon(
            &mut s,
            (255, 255, 255),
            &[(2, 2), (21, 2), (21, 21), (12, 10), (2, 21)],
            0,
        );
        assert_eq!(r, Rect::new(2, 2, 20, 20));
        let m = muster(&s);
        assert_eq!(m[11], "..####################..", "oberhalb der Kerbe voll");
        assert_eq!(m[13], "..#########...########..");
        assert_eq!(m[20], "..##................##..");
    }

    #[test]
    fn der_umriss_ist_ein_geschlossener_linienzug() {
        let ecken = [(2, 18), (10, 2), (18, 18)];
        let mut a = leinwand(20, 20);
        let ra = polygon(&mut a, (255, 255, 255), &ecken, 1);
        let mut b = leinwand(20, 20);
        let rb = lines(&mut b, (255, 255, 255), true, &ecken, 1);
        assert_eq!(ra, rb);
        assert_eq!(muster(&a), muster(&b), "polygon(width>0) == lines(closed)");
        assert_eq!(a.get_at((10, 10)).r, 0, "die Mitte bleibt leer");
    }

    #[test]
    fn negative_breite_zeichnet_nichts() {
        // Gemessen: nur width == 0 fuellt, width < 0 ist ein Verzicht.
        let mut s = leinwand(12, 12);
        let r = polygon(
            &mut s,
            (255, 255, 255),
            &[(1, 1), (9, 1), (9, 9), (1, 9)],
            -1,
        );
        assert_eq!(r, Rect::new(1, 1, 0, 0));
        assert!(muster(&s).iter().all(|z| !z.contains('#')));

        let r = polygon(
            &mut s,
            (255, 255, 255),
            &[(1, 1), (9, 1), (9, 9), (1, 9)],
            0,
        );
        assert_eq!(r, Rect::new(1, 1, 9, 9));
    }

    #[test]
    fn zu_wenige_punkte() {
        // Gefuellt braucht pygame drei Punkte (sonst ValueError); der
        // Umriss kommt mit zweien aus und zeichnet eine Strecke.
        let mut s = leinwand(12, 12);
        assert_eq!(
            polygon(&mut s, (255, 255, 255), &[(2, 2), (7, 7)], 0),
            Rect::new(2, 2, 0, 0)
        );
        assert!(muster(&s).iter().all(|z| !z.contains('#')));

        let r = polygon(&mut s, (255, 255, 255), &[(2, 2), (7, 7)], 1);
        assert_eq!(r, Rect::new(2, 2, 6, 6));
        assert_eq!(s.get_at((4, 4)).r, 255);

        assert_eq!(
            polygon(&mut s, (255, 255, 255), &[], 0),
            Rect::new(0, 0, 0, 0)
        );
        assert_eq!(
            polygon(&mut s, (255, 255, 255), &[(3, 5)], 0),
            Rect::new(3, 5, 0, 0)
        );
    }

    #[test]
    fn entartete_vielecke_werden_zu_strecken() {
        // Alle Punkte auf einer Zeile: pygame zieht eine Strecke von der
        // kleinsten bis zur groessten x-Koordinate.
        let mut s = leinwand(10, 10);
        let r = polygon(
            &mut s,
            (255, 255, 255),
            &[(2, 4), (8, 4), (5, 4), (3, 4)],
            0,
        );
        assert_eq!(r, Rect::new(2, 4, 7, 1));
        assert_eq!(muster(&s)[4], "..#######.");

        // Alle auf einer Spalte - hier greift das Abtasten ganz normal.
        let mut s = leinwand(10, 10);
        let r = polygon(
            &mut s,
            (255, 255, 255),
            &[(4, 2), (4, 8), (4, 5), (4, 3)],
            0,
        );
        assert_eq!(r, Rect::new(4, 2, 1, 7));
    }

    #[test]
    fn das_clip_rechteck_begrenzt_und_bestimmt_die_rueckgabe() {
        let mut s = leinwand(20, 20);
        s.set_clip(Some(Rect::new(6, 6, 5, 5)));
        let r = polygon(&mut s, (255, 255, 255), &[(2, 18), (10, 2), (18, 18)], 0);
        assert_eq!(r, Rect::new(6, 6, 5, 5));
        assert_eq!(s.get_at((5, 6)).r, 0);
        assert_eq!(s.get_at((10, 10)).r, 255);

        // Ganz ausserhalb: leeres Rechteck am ersten Punkt.
        let mut s = leinwand(20, 20);
        let r = polygon(&mut s, (255, 255, 255), &[(40, 40), (50, 40), (45, 50)], 0);
        assert_eq!(r, Rect::new(40, 40, 0, 0));
    }

    #[test]
    fn die_farbe_wird_ohne_mischen_geschrieben() {
        let mut s = Surface::with_flags((12, 12), SRCALPHA);
        s.fill((50, 50, 50, 255));
        polygon(
            &mut s,
            (255, 0, 0, 128),
            &[(2, 2), (9, 2), (9, 9), (2, 9)],
            0,
        );
        assert_eq!(s.get_at((5, 5)).as_tuple(), (255, 0, 0, 128));
        assert_eq!(s.get_at((0, 0)).as_tuple(), (50, 50, 50, 255));
    }
}
