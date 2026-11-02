//! frect.rs
//! ========
//! Rechtecke mit Nachkommastellen - Gegenstueck zu `pygame.FRect`.
//!
//! Dieselben 20 virtuellen Attribute wie [`crate::rect::Rect`], erzeugt
//! aus **denselben** Makros - die sind dafuer typgenerisch gemacht
//! worden. Es gibt hier also keine zweite, leicht abweichende Fassung von
//! `set_center`, die irgendwann auseinanderlaeuft.
//!
//! ## Wo `FRect` sich von `Rect` unterscheidet
//!
//! Nicht nur in den Nachkommastellen. Alles, was in `Rect` ganzzahlig
//! gerechnet wird, rechnet hier durch - und das aendert Ergebnisse:
//!
//! ```text
//! Rect(1, 2, 10, 6).center    -> (6, 5)        ganzzahlig halbiert
//! FRect(1.5, 2.5, 10.25, 6.5).center -> (6.625, 5.75)
//!
//! Rect(28,13,20,22).scale_by(3.7) -> (1, 2, 74, 44)      dreifach abgeschnitten
//! FRect(1.5,2.5,10.25,6.5).scale_by(2) -> (-3.625, -0.75, 20.5, 13.0)
//! ```
//!
//! ## Gespeichert wird `f32`, gelesen wird `f64`
//!
//! Das ist keine Nachlaessigkeit, sondern zweimal gemessen:
//!
//! * **Gespeichert** wird `f32`. `FRect(0.1, …).x` meldet
//!   `0.10000000149011612` - die `f64`-Schreibweise einer `f32`-Null-
//!   Komma-Eins. Wer hier `f64` speichert, weicht ab der siebten Stelle
//!   ab, und `scale_by` zeigt es sofort.
//! * **Abgeleitete Werte** wie `right` oder `center` rechnet pygame
//!   dagegen in `f64` und rundet **nicht** auf `f32` zurueck:
//!
//! ```text
//! FRect(1.951, 0, 4.084, 1).right
//!   pygame     6.0350000858306885      <- f64-Summe
//!   f32-Summe  6.034999847412109       <- ein Bit daneben
//! ```
//!
//! Deshalb geben die Leser hier `f64` zurueck, die Setzer nehmen `f32`.
//! Beide Makros sind dafuer um einen zweiten Typ erweitert worden - und
//! `Rect` benutzt dieselben, dort mit `i32` in beiden Rollen.

use crate::rect::{punkt_attr, zahl_attr, Rect};

/// Ein Rechteck mit Nachkommastellen.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FRect {
    /// Die linke Kante.
    pub x: f32,
    /// Die obere Kante.
    pub y: f32,
    /// Die Breite. Darf negativ sein - siehe [`FRect::normalized`].
    pub w: f32,
    /// Die Hoehe. Dito.
    pub h: f32,
}

impl FRect {
    /// Aus vier Zahlen - wie [`Rect::new`], nur mit Komma.
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> FRect {
        FRect { x, y, w, h }
    }

    /// Aus Ort und Groesse. Siehe [`Rect::from_parts`].
    pub const fn from_parts(pos: (f32, f32), size: (f32, f32)) -> FRect {
        FRect::new(pos.0, pos.1, size.0, size.1)
    }

    // ----- Die einfachen Kanten ----------------------------------------
    //
    // Dieselben Makros wie in `rect.rs`, nur mit `f32` statt `i32` - und
    // **ohne** die Ganzzahlteilung: `s.w / 2.0` statt `s.w / 2`.

    zahl_attr!(f64, f32, left, set_left, with_left, |s| s.x as f64, |m, v| m.x = v);
    zahl_attr!(f64, f32, top, set_top, with_top, |s| s.y as f64, |m, v| m.y = v);
    zahl_attr!(f64, f32, right, set_right, with_right, |s| s.x as f64 + s.w as f64, |m, v| m.x = v - m.w);
    zahl_attr!(f64, f32, bottom, set_bottom, with_bottom, |s| s.y as f64 + s.h as f64, |m, v| m.y = v - m.h);
    zahl_attr!(f64, f32, centerx, set_centerx, with_centerx,
        |s| s.x as f64 + s.w as f64 / 2.0, |m, v| m.x = v - m.w / 2.0);
    zahl_attr!(f64, f32, centery, set_centery, with_centery,
        |s| s.y as f64 + s.h as f64 / 2.0, |m, v| m.y = v - m.h / 2.0);
    zahl_attr!(f64, f32, width, set_width, with_width, |s| s.w as f64, |m, v| m.w = v);
    zahl_attr!(f64, f32, height, set_height, with_height, |s| s.h as f64, |m, v| m.h = v);

    // ----- Die neun Anker ----------------------------------------------
    //
    // **Die Setzer rechnen mit einer Differenz, nicht mit einer
    // Zuweisung.** Im Original steht
    //
    // ```c
    // self->r.x += val1 - (self->r.x + (self->r.w / 2));
    // ```
    //
    // und **nicht** `self->r.x = val1 - self->r.w / 2` - das steht nur
    // beim Einzelwert-Setzer `centerx`. Bei ganzen Zahlen sind beide
    // Formen dasselbe, weshalb es `Rect` nicht betrifft; bei Fliesskomma
    // gehen sie um ein Bit auseinander. Gemessen an
    // `FRect(-38.951, …).center = (1.5, 2.5)`: 10.320499420166016 gegen
    // 10.320500373840332.
    //
    // **Hier rundet pygame, bei den Einzelwerten nicht.** Gemessen an
    // `FRect(-38.5, 0, 2.534, 1)`:
    //
    // ```text
    //   centerx    -37.23300004005432      <- f64, ungerundet
    //   center[0]  -37.233001708984375     <- durch f32 hindurch
    // ```
    //
    // Dieselbe Rechnung, zwei Ergebnisse - weil die Tupel-Leser ihre
    // Werte als `float` durchreichen und die Einzelwert-Leser nicht.
    // `f` bildet genau das ab.


    punkt_attr!(f64, f32, topleft, set_topleft, with_topleft,
        |s| (Self::rund(s.x as f64), Self::rund(s.y as f64)), |m, v| { m.x += v.0 - m.x; m.y += v.1 - m.y });
    punkt_attr!(f64, f32, topright, set_topright, with_topright,
        |s| (Self::rund(s.x as f64 + s.w as f64), Self::rund(s.y as f64)), |m, v| { m.x += v.0 - (m.x + m.w); m.y += v.1 - m.y });
    punkt_attr!(f64, f32, bottomleft, set_bottomleft, with_bottomleft,
        |s| (Self::rund(s.x as f64), Self::rund(s.y as f64 + s.h as f64)), |m, v| { m.x += v.0 - m.x; m.y += v.1 - (m.y + m.h) });
    punkt_attr!(f64, f32, bottomright, set_bottomright, with_bottomright,
        |s| (Self::rund(s.x as f64 + s.w as f64), Self::rund(s.y as f64 + s.h as f64)), |m, v| { m.x += v.0 - (m.x + m.w); m.y += v.1 - (m.y + m.h) });
    punkt_attr!(f64, f32, center, set_center, with_center,
        |s| (Self::rund(s.x as f64 + s.w as f64 / 2.0), Self::rund(s.y as f64 + s.h as f64 / 2.0)),
        |m, v| { m.x += v.0 - (m.x + m.w / 2.0); m.y += v.1 - (m.y + m.h / 2.0) });
    punkt_attr!(f64, f32, midtop, set_midtop, with_midtop,
        |s| (Self::rund(s.x as f64 + s.w as f64 / 2.0), Self::rund(s.y as f64)), |m, v| { m.x += v.0 - (m.x + m.w / 2.0); m.y += v.1 - m.y });
    punkt_attr!(f64, f32, midbottom, set_midbottom, with_midbottom,
        |s| (Self::rund(s.x as f64 + s.w as f64 / 2.0), Self::rund(s.y as f64 + s.h as f64)),
        |m, v| { m.x += v.0 - (m.x + m.w / 2.0); m.y = v.1 - m.h });
    punkt_attr!(f64, f32, midleft, set_midleft, with_midleft,
        |s| (Self::rund(s.x as f64), Self::rund(s.y as f64 + s.h as f64 / 2.0)), |m, v| { m.x += v.0 - m.x; m.y += v.1 - (m.y + m.h / 2.0) });
    punkt_attr!(f64, f32, midright, set_midright, with_midright,
        |s| (Self::rund(s.x as f64 + s.w as f64), Self::rund(s.y as f64 + s.h as f64 / 2.0)),
        |m, v| { m.x = v.0 - m.w; m.y += v.1 - (m.y + m.h / 2.0) });
    punkt_attr!(f64, f32, size, set_size, with_size,
        |s| (Self::rund(s.w as f64), Self::rund(s.h as f64)), |m, v| { m.w += v.0 - m.w; m.h += v.1 - m.h });

    /// Durch `f32` hindurch - siehe die Anmerkung bei den Ankern.
    #[inline]
    fn rund(v: f64) -> f64 {
        v as f32 as f64
    }

    // Die Mitte **relativ zur eigenen Ecke** - `(w/2, h/2)`.
    //
    // Setzen aendert die Groesse, nicht die Lage: `w = 2*v.0`,
    // `h = 2*v.1`. Gemessen:
    // `f = FRect(1,2,75,45); f.relcenter = (697.5, 345.25)` gibt
    // `FRect(1.0, 2.0, 1395.0, 690.5)`. Aus
    // `RectExport_setrelcenter`, `rect_impl.h` Zeile 2851.
    punkt_attr!(f64, f32, relcenter, set_relcenter, with_relcenter,
        |s| (Self::rund(s.w as f64 / 2.0), Self::rund(s.h as f64 / 2.0)),
        |m, v| { m.w = v.0 * 2.0; m.h = v.1 * 2.0 });

    // ----- Verschieben und Groesse aendern ------------------------------

    /// Verschobene Kopie. Siehe [`Rect::move_`].
    ///
    /// Der Unterstrich steht da, weil `move` in Rust ein Schluesselwort
    /// ist; in Python heisst die Methode `move`.
    pub fn move_(&self, dx: f32, dy: f32) -> FRect {
        FRect::new(self.x + dx, self.y + dy, self.w, self.h)
    }

    /// Verschiebt an Ort und Stelle. Siehe [`Rect::move_ip`].
    pub fn move_ip(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }

    /// Um `dx`/`dy` groessere Kopie, **mittig**. Siehe [`Rect::inflate`].
    pub fn inflate(&self, dx: f32, dy: f32) -> FRect {
        FRect::new(
            self.x - dx / 2.0,
            self.y - dy / 2.0,
            self.w + dx,
            self.h + dy,
        )
    }

    /// Wie [`FRect::inflate`], an Ort und Stelle.
    pub fn inflate_ip(&mut self, dx: f32, dy: f32) {
        *self = self.inflate(dx, dy);
    }

    /// Um Faktoren skalierte Kopie, mittig.
    ///
    /// Dieselbe Rechnung wie bei [`Rect::scale_by`](crate::rect::Rect::scale_by),
    /// nur **ohne** die drei Abschneidungen: der Faktor wird betragsmaessig
    /// genommen, ein `fy` von 0 heisst "nimm `fx`", und danach bleibt
    /// alles Fliesskomma.
    pub fn scale_by(&self, fx: f32, fy: f32) -> FRect {
        let fx = fx.abs();
        let fy = fy.abs();
        let fy = if fy > 0.0 { fy } else { fx };
        FRect::new(
            self.x + self.w / 2.0 - self.w * fx / 2.0,
            self.y + self.h / 2.0 - self.h * fy / 2.0,
            self.w * fx,
            self.h * fy,
        )
    }

    /// Wie [`FRect::scale_by`], an Ort und Stelle.
    pub fn scale_by_ip(&mut self, fx: f32, fy: f32) {
        *self = self.scale_by(fx, fy);
    }

    /// Setzt alle vier Zahlen neu. Siehe [`Rect::update`].
    pub fn update(&mut self, x: f32, y: f32, w: f32, h: f32) {
        *self = FRect::new(x, y, w, h);
    }

    /// Eine Kopie.
    ///
    /// Gibt es, weil pygames `Rect.copy()` es gibt - in Rust ist ein
    /// `FRect` ohnehin `Copy`, und `*self` taete dasselbe.
    pub fn copy(&self) -> FRect {
        *self
    }

    /// Dreht negative Breiten/Hoehen um.
    pub fn normalize(&mut self) {
        if self.w < 0.0 {
            self.x += self.w;
            self.w = -self.w;
        }
        if self.h < 0.0 {
            self.y += self.h;
            self.h = -self.h;
        }
    }

    /// Mit nicht-negativer Breite und Hoehe. Siehe [`Rect::normalized`].
    pub fn normalized(&self) -> FRect {
        let mut r = *self;
        r.normalize();
        r
    }

    /// Ist die Flaeche null? Siehe [`Rect::is_empty`].
    pub fn is_empty(&self) -> bool {
        self.w <= 0.0 || self.h <= 0.0
    }

    // ----- Beruehrungen ---------------------------------------------------

    /// Liegt der Punkt darin? Die rechte und untere Kante gehoeren
    /// **nicht** dazu - gemessen: `FRect(0,0,4,3).collidepoint((4, 3))`
    /// ist `false`, `(3.999, 2.999)` dagegen `true`.
    pub fn collidepoint(&self, p: (f32, f32)) -> bool {
        p.0 >= self.x && p.0 < self.x + self.w && p.1 >= self.y && p.1 < self.y + self.h
    }

    /// Beruehren sich die beiden? Siehe [`Rect::colliderect`].
    pub fn colliderect(&self, other: FRect) -> bool {
        let a = self.normalized();
        let b = other.normalized();
        if a.w <= 0.0 || a.h <= 0.0 || b.w <= 0.0 || b.h <= 0.0 {
            return false;
        }
        a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
    }

    /// Liegt `other` ganz darin? Siehe [`Rect::contains`].
    pub fn contains(&self, other: FRect) -> bool {
        self.x <= other.x
            && self.y <= other.y
            && self.x + self.w >= other.x + other.w
            && self.y + self.h >= other.y + other.h
            && self.x + self.w > other.x
            && self.y + self.h > other.y
    }

    /// Der **erste** Treffer in der Liste. Siehe [`Rect::collidelist`].
    pub fn collidelist(&self, others: &[FRect]) -> Option<usize> {
        others.iter().position(|o| self.colliderect(*o))
    }

    /// **Alle** Treffer. Siehe [`Rect::collidelistall`].
    pub fn collidelistall(&self, others: &[FRect]) -> Vec<usize> {
        others
            .iter()
            .enumerate()
            .filter(|(_, o)| self.colliderect(**o))
            .map(|(i, _)| i)
            .collect()
    }

    /// Der erste Treffer als Rechteck statt als Nummer.
    pub fn collideobjects(&self, others: &[FRect]) -> Option<FRect> {
        others.iter().copied().find(|r| self.colliderect(*r))
    }

    /// Alle Treffer als Rechtecke.
    pub fn collideobjectsall(&self, others: &[FRect]) -> Vec<FRect> {
        others
            .iter()
            .copied()
            .filter(|r| self.colliderect(*r))
            .collect()
    }

    // ----- Verknuepfungen -------------------------------------------------

    /// Der gemeinsame Teil. Siehe [`Rect::clip`].
    pub fn clip(&self, other: FRect) -> FRect {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let r = (self.x + self.w).min(other.x + other.w);
        let b = (self.y + self.h).min(other.y + other.h);
        if r <= x || b <= y {
            return FRect::new(self.x, self.y, 0.0, 0.0);
        }
        FRect::new(x, y, r - x, b - y)
    }

    /// Das kleinste Rechteck um beide. Siehe [`Rect::union`].
    pub fn union(&self, other: FRect) -> FRect {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let r = (self.x + self.w).max(other.x + other.w);
        let b = (self.y + self.h).max(other.y + other.h);
        FRect::new(x, y, r - x, b - y)
    }

    /// Wie [`FRect::union`], an Ort und Stelle.
    pub fn union_ip(&mut self, other: FRect) {
        *self = self.union(other);
    }

    /// Das kleinste Rechteck um alle. Siehe [`Rect::unionall`].
    pub fn unionall(&self, others: &[FRect]) -> FRect {
        others.iter().fold(*self, |a, b| a.union(*b))
    }

    /// Wie [`FRect::unionall`], an Ort und Stelle.
    pub fn unionall_ip(&mut self, others: &[FRect]) {
        *self = self.unionall(others);
    }

    /// Nach `other` hineingeschoben. Siehe [`Rect::clamp`].
    pub fn clamp(&self, other: FRect) -> FRect {
        let mut x = self.x;
        let mut y = self.y;
        if self.w >= other.w {
            x = other.x + other.w / 2.0 - self.w / 2.0;
        } else if self.x < other.x {
            x = other.x;
        } else if self.x + self.w > other.x + other.w {
            x = other.x + other.w - self.w;
        }
        if self.h >= other.h {
            y = other.y + other.h / 2.0 - self.h / 2.0;
        } else if self.y < other.y {
            y = other.y;
        } else if self.y + self.h > other.y + other.h {
            y = other.y + other.h - self.h;
        }
        FRect::new(x, y, self.w, self.h)
    }

    /// Wie [`FRect::clamp`], an Ort und Stelle.
    pub fn clamp_ip(&mut self, other: FRect) {
        *self = self.clamp(other);
    }

    /// Passt das Rechteck unter Wahrung des Seitenverhaeltnisses hinein.
    ///
    /// Zeilengetreu aus `RectExport_fit`: pygame bildet die **Verhaeltnisse
    /// andersherum** (`self.w / other.w`), nimmt davon das **groessere**
    /// und **teilt** dadurch. Das ist bei positiven Groessen dasselbe wie
    /// "mit dem kleineren Kehrwert malnehmen", bei negativen aber nicht -
    /// und pygame sichert nichts ab, es rechnet einfach durch. Eine
    /// Fassung mit Absicherung trifft 339 von 2500 Faellen, diese 2500.
    pub fn fit(&self, other: FRect) -> FRect {
        let xratio = self.w / other.w;
        let yratio = self.h / other.h;
        let maxratio = if xratio > yratio { xratio } else { yratio };
        let w = self.w / maxratio;
        let h = self.h / maxratio;
        FRect::new(
            other.x + (other.w - w) / 2.0,
            other.y + (other.h - h) / 2.0,
            w,
            h,
        )
    }

    /// Schneidet eine Strecke am Rechteck zu.
    ///
    /// Auch hier gilt die `- 1`-Kante wie bei `Rect`: gemessen liefern
    /// `Rect(0,0,4,3)` und `FRect(0,0,4,3)` fuer die Strecke
    /// `(0,0)-(10,10)` **beide** den Endpunkt `(2, 2)`.
    pub fn clipline(&self, line: (f32, f32, f32, f32)) -> Option<(f32, f32, f32, f32)> {
        let n = self.normalized();
        if n.w <= 0.0 || n.h <= 0.0 {
            return None;
        }
        let (mut x1, mut y1, mut x2, mut y2) = line;
        let (rx1, ry1) = (n.x, n.y);
        let (rx2, ry2) = (n.x + n.w - 1.0, n.y + n.h - 1.0);

        if x1 >= rx1
            && x1 <= rx2
            && x2 >= rx1
            && x2 <= rx2
            && y1 >= ry1
            && y1 <= ry2
            && y2 >= ry1
            && y2 <= ry2
        {
            return Some((x1, y1, x2, y2));
        }
        if (x1 < rx1 && x2 < rx1)
            || (x1 > rx2 && x2 > rx2)
            || (y1 < ry1 && y2 < ry1)
            || (y1 > ry2 && y2 > ry2)
        {
            return None;
        }
        if y1 == y2 {
            return Some((x1.clamp(rx1, rx2), y1, x2.clamp(rx1, rx2), y2));
        }
        if x1 == x2 {
            return Some((x1, y1.clamp(ry1, ry2), x2, y2.clamp(ry1, ry2)));
        }

        // **Die Lagebestimmung benutzt andere Grenzen als die
        // Beschneidung.** `ComputeOutCodeF` fragt `y >= rect.y + rect.h`,
        // zugeschnitten wird aber auf `rect.y + rect.h - 1`. Bei ganzen
        // Zahlen ist beides dasselbe - `y > y+h-1` und `y >= y+h` fallen
        // zusammen -, bei Fliesskomma nicht: ein Punkt im Band
        // `[y+h-1, y+h)` gilt als **drinnen**, landet aber auf `y+h-1`.
        //
        // Das ist offensichtlich aus SDLs Ganzzahlfassung uebernommen und
        // dort nie aufgefallen. Unveraendert nachgebaut - es sind 32 von
        // 2500 Faellen, in denen es den Ausschlag gibt.
        fn lage(r: &FRect, x: f32, y: f32) -> u8 {
            let mut c = 0u8;
            if y < r.y {
                c |= 1;
            } else if y >= r.y + r.h {
                c |= 2;
            }
            if x < r.x {
                c |= 4;
            } else if x >= r.x + r.w {
                c |= 8;
            }
            c
        }

        let mut c1 = lage(&n, x1, y1);
        let mut c2 = lage(&n, x2, y2);
        while c1 != 0 || c2 != 0 {
            if c1 & c2 != 0 {
                return None;
            }
            let c = if c1 != 0 { c1 } else { c2 };
            // Wie bei `Rect`: **immer** von `(x1, y1)` aus gerechnet.
            let (px, py) = (x1, y1);
            // **In f64 rechnen, Ergebnis durch f32.** pygame bestimmt
            // den Schnittpunkt in doppelter Genauigkeit und legt ihn
            // dann als `float` ab; wer durchgehend in `f32` rechnet,
            // landet an 156 von 2500 Faellen ein Bit daneben.
            // **Zwischenwerte in `f64`, Ergebnis als `f32`.** Im
            // C-Original sind alle Groessen `float`; gemessen trifft
            // diese Fassung aber deutlich besser (32 statt 133
            // Abweichungen von 2500), was dafuer spricht, dass der
            // Uebersetzer den Ausdruck mit erhoehter Genauigkeit
            // auswertet oder zu einem FMA zusammenzieht. Ein Bit
            // Unterschied bleibt in wenigen Faellen - siehe den
            // Modulkopf.
            let (nx, ny);
            if c & 1 != 0 || c & 2 != 0 {
                ny = if c & 1 != 0 { ry1 } else { ry2 };
                nx = (px as f64
                    + ((x2 - x1) as f64 * (ny - py) as f64) / (y2 - y1) as f64)
                    as f32;
            } else {
                nx = if c & 4 != 0 { rx1 } else { rx2 };
                ny = (py as f64
                    + ((y2 - y1) as f64 * (nx - px) as f64) / (x2 - x1) as f64)
                    as f32;
            }
            if c1 != 0 {
                x1 = nx;
                y1 = ny;
                c1 = lage(&n, x1, y1);
            } else {
                x2 = nx;
                y2 = ny;
                c2 = lage(&n, x2, y2);
            }
        }
        Some((x1, y1, x2, y2))
    }
}

// ----- Umwandlung ---------------------------------------------------------

impl From<Rect> for FRect {
    fn from(r: Rect) -> FRect {
        FRect::new(r.x as f32, r.y as f32, r.w as f32, r.h as f32)
    }
}

impl From<FRect> for Rect {
    /// **Abschneiden, nicht runden** - gemessen:
    /// `Rect(FRect(1.5, 2.5, 10.25, 6.5))` ist `(1, 2, 10, 6)`.
    fn from(f: FRect) -> Rect {
        Rect::new(f.x as i32, f.y as i32, f.w as i32, f.h as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nah(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    /// Alle Sollwerte an echtem pygame-ce 2.5.7 gemessen,
    /// Ausgangsrechteck `FRect(1.5, 2.5, 10.25, 6.5)`.
    #[test]
    fn die_anker_rechnen_ohne_abschneiden() {
        let f = FRect::new(1.5, 2.5, 10.25, 6.5);
        assert_eq!(f.center(), (6.625, 5.75));
        assert_eq!(f.centerx(), 6.625);
        assert_eq!(f.right(), 11.75);
        assert_eq!(f.midbottom(), (6.625, 9.0));

        // **In f64 gerechnet.** Die f32-Summe waere hier 6.034999847412109
        // und damit ein Bit daneben; pygame meldet die f64-Summe.
        let g = FRect::new(1.951, 0.0, 4.084, 1.0);
        assert_eq!(g.right(), 6.0350000858306885);

        // Zum Vergleich: `Rect` halbiert ganzzahlig.
        assert_eq!(Rect::new(1, 2, 10, 6).center(), (6, 5));
    }

    #[test]
    fn die_kanten_gehoeren_nicht_dazu() {
        let g = FRect::new(0.0, 0.0, 4.0, 3.0);
        assert!(g.collidepoint((0.0, 0.0)));
        assert!(g.collidepoint((3.999, 2.999)));
        assert!(!g.collidepoint((4.0, 3.0)), "rechte untere Ecke ist draussen");
        assert!(!g.collidepoint((4.0, 0.0)));
        assert!(!g.collidepoint((-0.001, 0.0)));

        assert!(!g.colliderect(FRect::new(4.0, 0.0, 1.0, 1.0)));
        assert!(g.colliderect(FRect::new(3.9, 0.0, 1.0, 1.0)));
        // Ein Rechteck ohne Flaeche beruehrt nichts.
        assert!(!FRect::new(0.0, 0.0, 0.0, 3.0).colliderect(g));
    }

    #[test]
    fn die_verknuepfungen_treffen_pygame() {
        let f = FRect::new(1.5, 2.5, 10.25, 6.5);
        assert_eq!(
            f.clip(FRect::new(0.0, 0.0, 5.0, 5.0)),
            FRect::new(1.5, 2.5, 3.5, 2.5)
        );
        assert_eq!(
            f.union(FRect::new(-3.0, -3.0, 2.0, 2.0)),
            FRect::new(-3.0, -3.0, 14.75, 12.0)
        );
        assert_eq!(f.inflate(1.5, 1.5), FRect::new(0.75, 1.75, 11.75, 8.0));
        assert_eq!(f.scale_by(2.0, 2.0), FRect::new(-3.625, -0.75, 20.5, 13.0));

        let p = f.fit(FRect::new(0.0, 0.0, 20.0, 5.0));
        assert!(nah(p.x, 6.057692) && nah(p.y, 0.0), "{p:?}");
        assert!(nah(p.w, 7.884616) && nah(p.h, 5.0), "{p:?}");
        // Gemessen: pygame sichert nichts ab und liefert auch negative
        // Groessen, wenn das Zielrechteck welche hat.
        let q = FRect::new(-10.021, 36.562, 20.174, -30.504).fit(
            FRect::new(-10.021, 36.562, 20.174, -30.504));
        assert!(nah(q.w, 20.174) && nah(q.h, -30.504), "{q:?}");

        let mut n = FRect::new(5.0, 5.0, -3.0, -4.0);
        n.normalize();
        assert_eq!(n, FRect::new(2.0, 1.0, 3.0, 4.0));

        assert_eq!(
            f.clipline((0.0, 0.0, 20.0, 20.0)),
            Some((2.5, 2.5, 8.0, 8.0))
        );
    }

    /// `FRect` rechnet in `f32` - das ist an `scale_by` ablesbar.
    #[test]
    fn gerechnet_wird_in_einfacher_genauigkeit() {
        let f = FRect::new(1.5, 2.5, 10.25, 6.5);
        let s = f.scale_by(0.33, 0.33);
        // pygame meldet 4.933750152587891 bzw. 3.382500171661377 - das
        // sind f32-Werte, in f64 ausgeschrieben.
        assert_eq!(s.x as f64, 4.933750152587891);
        assert_eq!(s.w as f64, 3.382500171661377);
    }

    #[test]
    fn der_weg_zu_rect_und_zurueck() {
        let f = FRect::new(1.5, 2.5, 10.25, 6.5);
        // Abschneiden, nicht runden.
        assert_eq!(Rect::from(f), Rect::new(1, 2, 10, 6));
        assert_eq!(FRect::from(Rect::new(1, 2, 10, 6)), FRect::new(1.0, 2.0, 10.0, 6.0));
    }

    /// Gemessen an pygame: `FRect(1,2,75,45).relcenter = (697.5, 345.25)`
    /// gibt `FRect(1.0, 2.0, 1395.0, 690.5)`.
    #[test]
    fn relcenter_setzen_aendert_nur_die_groesse() {
        let mut f = FRect::new(1.0, 2.0, 75.0, 45.0);
        f.set_relcenter((697.5, 345.25));
        assert_eq!(f, FRect::new(1.0, 2.0, 1395.0, 690.5));
        assert_eq!(f.relcenter(), (697.5, 345.25));
    }
}
