//! `pygame.geometry` - `Circle` und `Line`.
//!
//! Zwei Formen, die pygame-ce seit 2.5 mitbringt und die bis heute in
//! keiner Zahl dieses Projekts auftauchten: `geometry_test.py` hat **145**
//! Tests, und der Laeufer kannte die Datei nicht.
//!
//! Alles hier ist reine Rechnerei - kein Fenster, kein Betriebssystem.
//! Deshalb steht es im Kern und nicht in der Huelle.
//!
//! **Gemessen an echtem pygame-ce 2.5.7**, und zwei Dinge sind nicht
//! erratbar:
//!
//! ```text
//! Circle(1, 2, 3).top      -> (1.0, -1.0)     ein **Punkt**, kein Skalar
//! Circle(2.5, 3.5, 4.2).as_rect() -> Rect(-1, 0, 8, 8)
//! ```
//!
//! Die zweite Zeile: `as_rect` rechnet `(x - r, y - r, d, d)` und
//! schneidet **zur Null hin** ab, so wie `Rect` es mit Fliesskomma immer
//! tut - aus `-1.7` wird `-1`, nicht `-2`.

use crate::vector2::Vector2;

/// Ein Kreis - `pygame.geometry.Circle`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    /// Mittelpunkt, waagerecht.
    pub x: f64,
    /// Mittelpunkt, senkrecht.
    pub y: f64,
    /// Radius.
    pub r: f64,
}

impl Circle {
    /// Ein Kreis aus Mittelpunkt und Radius.
    pub fn new(x: f64, y: f64, r: f64) -> Circle {
        Circle { x, y, r }
    }

    /// Der Durchmesser - `c.d` und `c.diameter`.
    pub fn diameter(&self) -> f64 {
        self.r * 2.0
    }

    /// Das Quadrat des Radius - `c.r_sqr`.
    ///
    /// Gibt es, weil jeder Abstandsvergleich es braucht und die Wurzel
    /// dabei nur Genauigkeit kostet.
    pub fn r_sqr(&self) -> f64 {
        self.r * self.r
    }

    /// Die Flaeche - `pi * r * r`.
    ///
    /// **Die Reihenfolge ist nicht beliebig.** Hier stand
    /// `PI * self.r_sqr()`, also `PI * (r * r)`; pygames `circle.c`
    /// schreibt `M_PI * circle.r * circle.r`, und das rechnet
    /// `(PI * r) * r`. Gemessen an `c.area = 2**31-1` und dann `c.area`:
    ///
    /// ```text
    /// pygame     2147483646.9999998
    /// vorher     2147483646.9999995
    /// ```
    ///
    /// Drei Einheiten im letzten Bit, und sie kamen allein aus der
    /// Klammer. Gefunden hat das die Probe `Felder zuweisen`, weil sie den
    /// Wert nach der Zuweisung **zurueckliest** statt nur zu pruefen, dass
    /// es keinen Fehler gab.
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.r * self.r
    }

    /// Der Umfang - `2 * pi * r`.
    pub fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.r
    }

    /// Der Mittelpunkt.
    pub fn center(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    /// Der oberste **Punkt** - nicht die oberste Kante.
    pub fn top(&self) -> (f64, f64) {
        (self.x, self.y - self.r)
    }

    /// Der unterste Punkt.
    pub fn bottom(&self) -> (f64, f64) {
        (self.x, self.y + self.r)
    }

    /// Der linke Punkt.
    pub fn left(&self) -> (f64, f64) {
        (self.x - self.r, self.y)
    }

    /// Der rechte Punkt.
    pub fn right(&self) -> (f64, f64) {
        (self.x + self.r, self.y)
    }

    /// Liegt der Punkt im Kreis? Der Rand zaehlt dazu.
    pub fn collidepoint(&self, px: f64, py: f64) -> bool {
        let dx = px - self.x;
        let dy = py - self.y;
        dx * dx + dy * dy <= self.r_sqr()
    }

    /// Beruehren oder schneiden sich die beiden Kreise?
    pub fn collidecircle(&self, other: &Circle) -> bool {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let summe = self.r + other.r;
        dx * dx + dy * dy <= summe * summe
    }

    /// Schneidet der Kreis das achsenparallele Rechteck?
    ///
    /// Der naechste Punkt des Rechtecks wird auf den Kreis bezogen -
    /// das ist die uebliche Loesung und auch pygames.
    pub fn colliderect(&self, rx: f64, ry: f64, rw: f64, rh: f64) -> bool {
        let nx = self.x.clamp(rx, rx + rw);
        let ny = self.y.clamp(ry, ry + rh);
        let dx = self.x - nx;
        let dy = self.y - ny;
        dx * dx + dy * dy <= self.r_sqr()
    }

    /// Liegt der andere Kreis **ganz** in diesem?
    pub fn contains_circle(&self, other: &Circle) -> bool {
        if other.r > self.r {
            return false;
        }
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let d = (dx * dx + dy * dy).sqrt();
        d + other.r <= self.r
    }

    /// Liegt das Rechteck ganz im Kreis? Alle vier Ecken muessen hinein.
    pub fn contains_rect(&self, rx: f64, ry: f64, rw: f64, rh: f64) -> bool {
        [
            (rx, ry),
            (rx + rw, ry),
            (rx, ry + rh),
            (rx + rw, ry + rh),
        ]
        .iter()
        .all(|(px, py)| self.collidepoint(*px, *py))
    }

    /// Verschoben.
    pub fn move_by(&self, dx: f64, dy: f64) -> Circle {
        Circle::new(self.x + dx, self.y + dy, self.r)
    }

    /// Um einen Punkt gedreht - der Winkel in Grad.
    ///
    /// Gemessen: `Circle(0, 0, 1).rotate(90, (5, 0))` gibt den
    /// Mittelpunkt `(5, -5)`. Das ist die uebliche Drehmatrix
    /// `(x cos - y sin, x sin + y cos)`, angewandt auf den Abstand zum
    /// Drehpunkt - auf dem Bildschirm sieht das wegen der nach unten
    /// wachsenden y-Achse wie eine Drehung im Uhrzeigersinn aus.
    pub fn rotate(&self, grad: f64, um: (f64, f64)) -> Circle {
        let b = grad.to_radians();
        let (s, c) = b.sin_cos();
        let dx = self.x - um.0;
        let dy = self.y - um.1;
        Circle::new(um.0 + dx * c - dy * s, um.1 + dx * s + dy * c, self.r)
    }

    /// Die zwei Schnittpunkte mit einem anderen Kreis.
    ///
    /// Leer, wenn sie sich nicht schneiden, wenn einer im anderen liegt
    /// oder wenn beide gleich sind. Bei Beruehrung kommt **ein** Punkt.
    pub fn intersect_circle(&self, other: &Circle) -> Vec<(f64, f64)> {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let d2 = dx * dx + dy * dy;
        let d = d2.sqrt();
        if d == 0.0 && self.r == other.r {
            return Vec::new();
        }
        if d > self.r + other.r || d < (self.r - other.r).abs() {
            return Vec::new();
        }
        let a = (self.r_sqr() - other.r_sqr() + d2) / (2.0 * d);
        let h2 = self.r_sqr() - a * a;
        let h = if h2 < 0.0 { 0.0 } else { h2.sqrt() };
        let mx = self.x + a * dx / d;
        let my = self.y + a * dy / d;
        if h == 0.0 {
            return vec![(mx, my)];
        }
        vec![
            (mx + h * dy / d, my - h * dx / d),
            (mx - h * dy / d, my + h * dx / d),
        ]
    }
}

/// Eine Strecke - `pygame.geometry.Line`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    /// Anfang, waagerecht.
    pub ax: f64,
    /// Anfang, senkrecht.
    pub ay: f64,
    /// Ende, waagerecht.
    pub bx: f64,
    /// Ende, senkrecht.
    pub by: f64,
}

impl Line {
    /// Eine Strecke aus zwei Punkten.
    pub fn new(ax: f64, ay: f64, bx: f64, by: f64) -> Line {
        Line { ax, ay, bx, by }
    }

    /// Die Laenge.
    pub fn length(&self) -> f64 {
        let dx = self.bx - self.ax;
        let dy = self.by - self.ay;
        (dx * dx + dy * dy).sqrt()
    }

    /// Verschoben.
    pub fn move_by(&self, dx: f64, dy: f64) -> Line {
        Line::new(self.ax + dx, self.ay + dy, self.bx + dx, self.by + dy)
    }

    /// Anfang und Ende vertauscht.
    pub fn flip_ab(&self) -> Line {
        Line::new(self.bx, self.by, self.ax, self.ay)
    }

    /// Um den Faktor gestreckt, mit einem Ursprung zwischen 0 und 1.
    ///
    /// `origin` sagt, welcher Punkt der Strecke stehen bleibt: `0.0` der
    /// Anfang, `1.0` das Ende, `0.5` die Mitte.
    pub fn scale(&self, faktor: f64, ursprung: f64) -> Line {
        let ox = self.ax + (self.bx - self.ax) * ursprung;
        let oy = self.ay + (self.by - self.ay) * ursprung;
        Line::new(
            ox + (self.ax - ox) * faktor,
            oy + (self.ay - oy) * faktor,
            ox + (self.bx - ox) * faktor,
            oy + (self.by - oy) * faktor,
        )
    }

    /// Der Punkt auf der Strecke, der einem gegebenen am naechsten liegt.
    ///
    /// Mit `clamp` bleibt er zwischen Anfang und Ende; ohne darf er auf
    /// der verlaengerten Geraden liegen.
    pub fn project(&self, px: f64, py: f64, begrenzen: bool) -> (f64, f64) {
        let dx = self.bx - self.ax;
        let dy = self.by - self.ay;
        let laenge2 = dx * dx + dy * dy;
        if laenge2 == 0.0 {
            return (self.ax, self.ay);
        }
        let mut t = ((px - self.ax) * dx + (py - self.ay) * dy) / laenge2;
        if begrenzen {
            t = t.clamp(0.0, 1.0);
        }
        (self.ax + t * dx, self.ay + t * dy)
    }

    /// Anfang und Ende als Vektoren - fuer alles, was damit weiterrechnet.
    pub fn punkte(&self) -> (Vector2, Vector2) {
        (
            Vector2::new(self.ax, self.ay),
            Vector2::new(self.bx, self.by),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Die Werte stammen aus echtem pygame-ce 2.5.7.
    #[test]
    fn kreis_masse_wie_pygame() {
        let c = Circle::new(1.0, 2.0, 3.0);
        assert_eq!(c.diameter(), 6.0);
        assert_eq!(c.r_sqr(), 9.0);
        assert!((c.area() - 28.274_333_882_308_138).abs() < 1e-12);
        assert!((c.circumference() - 18.849_555_921_538_76).abs() < 1e-12);
        // **Punkte**, keine Skalare.
        assert_eq!(c.top(), (1.0, -1.0));
        assert_eq!(c.bottom(), (1.0, 5.0));
        assert_eq!(c.left(), (-2.0, 2.0));
        assert_eq!(c.right(), (4.0, 2.0));
    }

    /// Der Rand zaehlt dazu - `(3, 4)` liegt genau auf `r = 5`.
    #[test]
    fn der_rand_gehoert_dazu() {
        let c = Circle::new(0.0, 0.0, 5.0);
        assert!(c.collidepoint(3.0, 4.0));
        assert!(!c.collidepoint(4.0, 4.0));
        assert!(c.collidecircle(&Circle::new(10.0, 0.0, 5.0)), "Beruehrung");
        assert!(!c.collidecircle(&Circle::new(11.0, 0.0, 5.0)));
    }

    /// Gemessen: `Circle(0,0,5).colliderect(Rect(4,4,2,2))` ist **falsch**.
    #[test]
    fn rechteck_beruehrung_wie_pygame() {
        let c = Circle::new(0.0, 0.0, 5.0);
        assert!(!c.colliderect(4.0, 4.0, 2.0, 2.0));
        assert!(c.colliderect(0.0, 0.0, 2.0, 2.0));
    }

    /// Gemessen: `Circle(0,0,1).rotate(90, (5,0))` gibt `(5, -5)`.
    #[test]
    fn drehen_wie_pygame() {
        let c = Circle::new(0.0, 0.0, 1.0).rotate(90.0, (5.0, 0.0));
        assert!((c.x - 5.0).abs() < 1e-9, "{c:?}");
        assert!((c.y + 5.0).abs() < 1e-9, "{c:?}");
    }

    /// Gemessen: die Schnittpunkte von `(0,0,5)` und `(8,0,5)`.
    #[test]
    fn schnittpunkte_wie_pygame() {
        let s = Circle::new(0.0, 0.0, 5.0).intersect_circle(&Circle::new(8.0, 0.0, 5.0));
        assert_eq!(s.len(), 2);
        let mut y: Vec<f64> = s.iter().map(|p| p.1).collect();
        y.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!((s[0].0 - 4.0).abs() < 1e-9, "{s:?}");
        assert!((y[0] + 3.0).abs() < 1e-9 && (y[1] - 3.0).abs() < 1e-9, "{s:?}");
    }

    #[test]
    fn strecke_masse() {
        let l = Line::new(0.0, 0.0, 3.0, 4.0);
        assert_eq!(l.length(), 5.0);
        assert_eq!(l.flip_ab(), Line::new(3.0, 4.0, 0.0, 0.0));
        // Gemessen: der Fusspunkt von (5,5) auf der Waagerechten.
        assert_eq!(Line::new(0.0, 0.0, 10.0, 0.0).project(5.0, 5.0, true), (5.0, 0.0));
    }
}
