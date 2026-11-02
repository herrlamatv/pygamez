//! vector3.rs
//! ==========
//! Gegenstueck zu `pygame.math.Vector3`.
//!
//! Vorlage ist `src_c/math.c` aus pygame-ce 2.5.7. Der Unterschied zu
//! [`crate::vector2::Vector2`] ist nicht nur die dritte Zahl - hier gibt es
//! eine Eigenheit, die man nicht erfindet, sondern nur abliest: das
//! **allgemeine** `rotate` faengt rechte Winkel ab, die
//! **achsenparallelen** `rotate_x/y/z` tun das **nicht**. Gemessen:
//!
//! ```text
//! (1,2,3).rotate(90, (0,0,1))  ->  (-2.0, 1.0,                3.0)
//! (1,2,3).rotate_z(90)         ->  (-2.0, 1.0000000000000002, 3.0)
//! ```
//!
//! Dieselbe Drehung, zwei verschiedene Ergebnisse. Wer beide ueber
//! denselben Weg baut, liegt bei einem davon daneben.

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::vector2::{Fehler, EPSILON};

/// Ein Vektor im Raum.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector3 {
    /// Der Anteil nach rechts.
    pub x: f64,
    /// Der Anteil nach unten.
    pub y: f64,
    /// Der Anteil in die Tiefe.
    pub z: f64,
}

impl Vector3 {
    /// Aus drei Zahlen.
    pub const fn new(x: f64, y: f64, z: f64) -> Vector3 {
        Vector3 { x, y, z }
    }

    /// Der Nullvektor.
    pub const fn null() -> Vector3 {
        Vector3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    // ----- Laenge --------------------------------------------------------

    /// Die Laenge.
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Die Laenge zum Quadrat - ohne Wurzel, zum Vergleichen.
    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Zweitname von [`Vector3::length`] - pygame hat beide.
    pub fn magnitude(self) -> f64 {
        self.length()
    }

    /// Zweitname von [`Vector3::length_squared`].
    pub fn magnitude_squared(self) -> f64 {
        self.length_squared()
    }

    /// Derselbe Vektor mit Laenge 1.
    /// 
    /// `Err` beim Nullvektor - pygame sagt dort
    /// `Can't normalize Vector of length Zero`.
    pub fn normalize(self) -> Result<Vector3, Fehler> {
        let l = self.length();
        if l == 0.0 {
            return Err(Fehler::LaengeNull);
        }
        Ok(Vector3::new(self.x / l, self.y / l, self.z / l))
    }

    /// Ist die Laenge 1? Mit derselben Toleranz, die pygame nimmt.
    pub fn is_normalized(self) -> bool {
        self.is_normalized_mit_eps(EPSILON)
    }

    /// Wie [`is_normalized`](Self::is_normalized) mit eigener Schranke -
    /// siehe [`Vector2::is_normalized_mit_eps`](crate::Vector2::is_normalized_mit_eps).
    pub fn is_normalized_mit_eps(self, eps: f64) -> bool {
        (self.length_squared() - 1.0).abs() < eps
    }

    /// Auf eine bestimmte Laenge bringen - siehe
    /// [`Vector2::scale_to_length_mit_eps`](crate::Vector2::scale_to_length_mit_eps).
    pub fn scale_to_length(self, laenge: f64) -> Result<Vector3, Fehler> {
        self.scale_to_length_mit_eps(laenge, EPSILON)
    }

    /// Dasselbe mit eigener Schranke - `Vector3.epsilon`.
    pub fn scale_to_length_mit_eps(self, laenge: f64, eps: f64) -> Result<Vector3, Fehler> {
        let alt = self.length();
        if alt < eps {
            return Err(Fehler::SkalierenNullLaenge);
        }
        Ok(self * (laenge / alt))
    }

    /// Die Laenge auf einen Bereich begrenzen.
    ///
    /// Wort fuer Wort dieselbe Rechnung wie
    /// [`Vector2::clamp_magnitude`](crate::Vector2::clamp_magnitude) -
    /// dort steht auch, warum es **nicht** ueber
    /// [`scale_to_length`](Self::scale_to_length) geht.
    pub fn clamp_magnitude(self, min_length: f64, max_length: f64) -> Result<Vector3, Fehler> {
        if min_length > max_length {
            return Err(Fehler::ClampMinUeberMax);
        }
        if max_length < 0.0 || min_length < 0.0 {
            return Err(Fehler::ClampNegativ);
        }
        let laenge2 = self.dot(self);
        if laenge2 == 0.0 && min_length > 0.0 {
            return Err(Fehler::ClampNullLaenge);
        }
        let mut faktor = 1.0;
        if laenge2 > max_length * max_length {
            faktor = max_length / laenge2.sqrt();
        }
        if laenge2 < min_length * min_length {
            faktor = min_length / laenge2.sqrt();
        }
        Ok(Vector3::new(
            self.x * faktor,
            self.y * faktor,
            self.z * faktor,
        ))
    }

    // ----- Drehen --------------------------------------------------------

    /// Dreht um `grad` um eine beliebige Achse.
    ///
    /// Zeilengetreu aus `_vector3_rotate_helper`: erst den Winkel in
    /// `[0, 2 PI)` bringen, dann die Achse normieren (aber **nur**, wenn
    /// ihre Laenge merklich von 1 abweicht), dann die Sonderfaelle fuer
    /// 0, 90, 180 und 270 Grad - jeder mit einer **eigenen**, vereinfachten
    /// Matrix, nicht mit der allgemeinen.
    pub fn rotate(self, grad: f64, achse: Vector3) -> Result<Vector3, Fehler> {
        self.rotate_rad(crate::deg2rad(grad), achse)
    }

    /// Wie [`rotate`](Self::rotate), aber im Bogenmass.
    pub fn rotate_rad(self, winkel: f64, achse: Vector3) -> Result<Vector3, Fehler> {
        self.rotate_rad_mit_eps(winkel, achse, EPSILON)
    }

    /// Wie [`rotate_rad`](Self::rotate_rad) mit eigener Schranke - die
    /// hier an **zwei** Stellen wirkt: ob die Achse als schon normiert
    /// gilt, und ob der Winkel als rechter Winkel gilt.
    pub fn rotate_rad_mit_eps(
        self,
        winkel: f64,
        achse: Vector3,
        eps: f64,
    ) -> Result<Vector3, Fehler> {
        use std::f64::consts::{FRAC_PI_2, TAU};

        let mut a = winkel % TAU;
        if a < 0.0 {
            a += TAU;
        }

        let laenge2 = achse.length_squared();
        if laenge2 < eps {
            return Err(Fehler::LaengeNull);
        }
        // pygame normiert nur, wenn es noetig ist - und das ist kein
        // Sparen, sondern bestimmt das letzte Bit des Ergebnisses.
        let n = if (laenge2 - 1.0).abs() > eps {
            achse * (1.0 / laenge2.sqrt())
        } else {
            achse
        };
        let (ax, ay, az) = (n.x, n.y, n.z);
        let (sx, sy, sz) = (self.x, self.y, self.z);

        if (a + eps) % FRAC_PI_2 < 2.0 * eps {
            return Ok(match ((a + eps) / FRAC_PI_2) as i32 {
                0 | 4 => self,
                1 => Vector3::new(
                    sx * (ax * ax) + sy * (ax * ay - az) + sz * (ax * az + ay),
                    sx * (ax * ay + az) + sy * (ay * ay) + sz * (ay * az - ax),
                    sx * (ax * az - ay) + sy * (ay * az + ax) + sz * (az * az),
                ),
                2 => Vector3::new(
                    sx * (-1.0 + ax * ax * 2.0) + sy * (ax * ay * 2.0) + sz * (ax * az * 2.0),
                    sx * (ax * ay * 2.0) + sy * (-1.0 + ay * ay * 2.0) + sz * (ay * az * 2.0),
                    sx * (ax * az * 2.0) + sy * (ay * az * 2.0) + sz * (-1.0 + az * az * 2.0),
                ),
                3 => Vector3::new(
                    sx * (ax * ax) + sy * (ax * ay + az) + sz * (ax * az - ay),
                    sx * (ax * ay - az) + sy * (ay * ay) + sz * (ay * az + ax),
                    sx * (ax * az + ay) + sy * (ay * az - ax) + sz * (az * az),
                ),
                _ => self.rotate_allgemein(a, n),
            });
        }
        Ok(self.rotate_allgemein(a, n))
    }

    /// Die allgemeine Drehmatrix (Rodrigues), ausgeschrieben wie im
    /// C-Original - Klammer fuer Klammer, damit die Rundung stimmt.
    fn rotate_allgemein(self, a: f64, n: Vector3) -> Vector3 {
        let (sin, cos) = (a.sin(), a.cos());
        let rest = 1.0 - cos;
        let (ax, ay, az) = (n.x, n.y, n.z);
        let (sx, sy, sz) = (self.x, self.y, self.z);
        Vector3::new(
            sx * (cos + ax * ax * rest)
                + sy * (ax * ay * rest - az * sin)
                + sz * (ax * az * rest + ay * sin),
            sx * (ax * ay * rest + az * sin)
                + sy * (cos + ay * ay * rest)
                + sz * (ay * az * rest - ax * sin),
            sx * (ax * az * rest - ay * sin)
                + sy * (ay * az * rest + ax * sin)
                + sz * (cos + az * az * rest),
        )
    }

    /// Dreht um die x-Achse.
    ///
    /// **Ohne** Sonderfall fuer rechte Winkel - anders als
    /// [`rotate`](Self::rotate). Das ist nicht vergessen, sondern
    /// abgelesen: `vector3_rotate_x_rad` in `math.c` ruft schlicht `sin`
    /// und `cos`. Deshalb liefert `rotate_z(90)` auf `(1,2,3)` in pygame
    /// `1.0000000000000002` statt `1.0`, und hier ebenso.
    pub fn rotate_x(self, grad: f64) -> Vector3 {
        self.rotate_x_rad(crate::deg2rad(grad))
    }

    /// Um die x-Achse gedreht, im Bogenmass.
    pub fn rotate_x_rad(self, a: f64) -> Vector3 {
        let (sin, cos) = (a.sin(), a.cos());
        Vector3::new(
            self.x,
            self.y * cos - self.z * sin,
            self.y * sin + self.z * cos,
        )
    }

    /// Dreht um die y-Achse. Siehe [`rotate_x`](Self::rotate_x).
    pub fn rotate_y(self, grad: f64) -> Vector3 {
        self.rotate_y_rad(crate::deg2rad(grad))
    }

    /// Um die y-Achse gedreht, im Bogenmass.
    pub fn rotate_y_rad(self, a: f64) -> Vector3 {
        let (sin, cos) = (a.sin(), a.cos());
        Vector3::new(
            self.x * cos + self.z * sin,
            self.y,
            -self.x * sin + self.z * cos,
        )
    }

    /// Dreht um die z-Achse. Siehe [`rotate_x`](Self::rotate_x).
    pub fn rotate_z(self, grad: f64) -> Vector3 {
        self.rotate_z_rad(crate::deg2rad(grad))
    }

    /// Um die z-Achse gedreht, im Bogenmass.
    pub fn rotate_z_rad(self, a: f64) -> Vector3 {
        let (sin, cos) = (a.sin(), a.cos());
        Vector3::new(
            self.x * cos - self.y * sin,
            self.x * sin + self.y * cos,
            self.z,
        )
    }

    /// `(Laenge, Neigung, Drehwinkel)` in Grad.
    pub fn as_spherical(self) -> (f64, f64, f64) {
        let r = self.length();
        if r == 0.0 {
            return (0.0, 0.0, 0.0);
        }
        (
            r,
            crate::rad2deg((self.z / r).acos()),
            crate::rad2deg(self.y.atan2(self.x)),
        )
    }

    /// Aus `(Laenge, Neigung, Drehwinkel)` in Grad.
    pub fn from_spherical(r: f64, theta: f64, phi: f64) -> Vector3 {
        let (t, p) = (crate::deg2rad(theta), crate::deg2rad(phi));
        Vector3::new(
            r * t.sin() * p.cos(),
            r * t.sin() * p.sin(),
            r * t.cos(),
        )
    }

    // ----- Verhaeltnisse zu anderen Vektoren -----------------------------

    /// Das Skalarprodukt.
    pub fn dot(self, o: Vector3) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    /// Das Kreuzprodukt - der Vektor senkrecht auf beiden.
    pub fn cross(self, o: Vector3) -> Vector3 {
        Vector3::new(
            self.y * o.z - self.z * o.y,
            self.z * o.x - self.x * o.z,
            self.x * o.y - self.y * o.x,
        )
    }

    /// Der Abstand zu `o`.
    pub fn distance_to(self, o: Vector3) -> f64 {
        (o - self).length()
    }

    /// Der Abstand zum Quadrat - ohne Wurzel.
    pub fn distance_squared_to(self, o: Vector3) -> f64 {
        (o - self).length_squared()
    }

    /// Der Winkel zu einem anderen Vektor, in Grad.
    pub fn angle_to(self, o: Vector3) -> f64 {
        let n = (self.length_squared() * o.length_squared()).sqrt();
        if n == 0.0 {
            return 0.0;
        }
        crate::rad2deg((self.dot(o) / n).clamp(-1.0, 1.0).acos())
    }

    /// Spiegelt an einer Flaeche mit dieser Normalen - siehe
    /// [`Vector2::reflect_mit_eps`](crate::Vector2::reflect_mit_eps).
    pub fn reflect(self, normal: Vector3) -> Result<Vector3, Fehler> {
        self.reflect_mit_eps(normal, EPSILON)
    }

    /// Dasselbe mit eigener Schranke - `Vector3.epsilon`.
    pub fn reflect_mit_eps(self, normal: Vector3, eps: f64) -> Result<Vector3, Fehler> {
        let q = normal.dot(normal);
        if q < eps {
            return Err(Fehler::NormaleNullLaenge);
        }
        let n = if q != 1.0 { normal / q.sqrt() } else { normal };
        Ok(self - n * (2.0 * self.dot(n)))
    }

    /// Der Anteil dieses Vektors in Richtung `o` - siehe
    /// [`Vector2::project_mit_eps`](crate::Vector2::project_mit_eps).
    pub fn project(self, o: Vector3) -> Result<Vector3, Fehler> {
        self.project_mit_eps(o, EPSILON)
    }

    /// Dasselbe mit eigener Schranke - `Vector3.epsilon`.
    pub fn project_mit_eps(self, o: Vector3, eps: f64) -> Result<Vector3, Fehler> {
        let q = o.dot(o);
        if q < eps {
            return Err(Fehler::ProjektionAufNullvektor);
        }
        Ok(o * (self.dot(o) / q))
    }

    /// Geradlinige Mischung - **`t` muss zwischen 0 und 1 liegen**, sonst
    /// `ValueError` wie in pygame. Siehe [`crate::vector2::Vector2::lerp`].
    pub fn lerp(self, o: Vector3, t: f64) -> Result<Vector3, Fehler> {
        if !(0.0..=1.0).contains(&t) {
            return Err(Fehler::MischwertAusserhalb);
        }
        Ok(Vector3::new(
            self.x * (1.0 - t) + o.x * t,
            self.y * (1.0 - t) + o.y * t,
            self.z * (1.0 - t) + o.z * t,
        ))
    }

    /// Mischung ueber den Kreisbogen.
    ///
    /// Dieselbe Vorlage wie
    /// [`Vector2::slerp`](crate::Vector2::slerp) - `vector_slerp` in
    /// `src_c/math.c` ist fuer beide Dimensionen **eine** Funktion.
    /// Daher auch die Stelle, die hier eigenartig aussieht und trotzdem
    /// richtig ist: ueber die Drehrichtung entscheidet
    /// `self.x * o.y < self.y * o.x` - nur die ersten **zwei** Achsen,
    /// obwohl der Vektor drei hat. Das ist kein Versehen der Uebertragung,
    /// sondern steht so in der Vorlage.
    pub fn slerp(self, o: Vector3, t: f64) -> Result<Vector3, Fehler> {
        self.slerp_mit_eps(o, t, EPSILON)
    }

    /// Wie [`slerp`](Self::slerp) mit eigener Schranke.
    pub fn slerp_mit_eps(self, o: Vector3, t: f64, eps: f64) -> Result<Vector3, Fehler> {
        use std::f64::consts::PI;
        if t.abs() > 1.0 {
            return Err(Fehler::SlerpMischwertAusserhalb);
        }
        let length1 = self.dot(self).sqrt();
        let length2 = o.dot(o).sqrt();
        if length1 < eps || length2 < eps {
            return Err(Fehler::SlerpNullVektor);
        }
        // Begrenzt, damit `acos` kein NaN liefert - siehe
        // [`Vector2::slerp`](crate::Vector2::slerp).
        let tmp = (self.dot(o) / (length1 * length2)).clamp(-1.0, 1.0);
        let mut winkel = tmp.acos();
        let mut t = t;
        if t < 0.0 {
            winkel -= 2.0 * PI;
            t = -t;
        }
        if self.x * o.y < self.y * o.x {
            winkel *= -1.0;
        }
        if winkel.abs() < eps || (winkel.abs() - 2.0 * PI).abs() < eps {
            return Ok(Vector3::new(
                self.x * (1.0 - t) + o.x * t,
                self.y * (1.0 - t) + o.y * t,
                self.z * (1.0 - t) + o.z * t,
            ));
        }
        if (winkel.abs() - PI).abs() < eps {
            return Err(Fehler::Slerp180);
        }
        let f0 = ((length2 - length1) * t + length1) / winkel.sin();
        let f1 = (winkel * (1.0 - t)).sin() / length1;
        let f2 = (winkel * t).sin() / length2;
        Ok(Vector3::new(
            (self.x * f1 + o.x * f2) * f0,
            (self.y * f1 + o.y * f2) * f0,
            (self.z * f1 + o.z * f2) * f0,
        ))
    }

    /// Weiche Mischung - `t` wird **vor** der Kurve begrenzt, wie in pygame.
    pub fn smoothstep(self, o: Vector3, t: f64) -> Vector3 {
        let t = if t <= 0.0 {
            0.0
        } else if t >= 1.0 {
            1.0
        } else {
            t * t * (3.0 - 2.0 * t)
        };
        // Nach dem Begrenzen liegt `t` sicher im erlaubten Bereich.
        Vector3::new(
            self.x * (1.0 - t) + o.x * t,
            self.y * (1.0 - t) + o.y * t,
            self.z * (1.0 - t) + o.z * t,
        )
    }

    /// Hoechstens `max_distance` weit auf `o` zu.
    pub fn move_towards(self, o: Vector3, max_distance: f64) -> Vector3 {
        if max_distance == 0.0 {
            return self;
        }
        let delta = o - self;
        let dist = delta.length();
        if dist == 0.0 {
            return self;
        }
        if dist <= max_distance {
            return o;
        }
        self + delta * (max_distance / dist)
    }

    /// Die drei Zahlen als Tupel.
    pub const fn as_tuple(self) -> (f64, f64, f64) {
        (self.x, self.y, self.z)
    }
}

// ----- Rechenzeichen ------------------------------------------------------

impl Add for Vector3 {
    type Output = Vector3;
    fn add(self, o: Vector3) -> Vector3 {
        Vector3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}
impl Sub for Vector3 {
    type Output = Vector3;
    fn sub(self, o: Vector3) -> Vector3 {
        Vector3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}
impl Mul<f64> for Vector3 {
    type Output = Vector3;
    fn mul(self, s: f64) -> Vector3 {
        Vector3::new(self.x * s, self.y * s, self.z * s)
    }
}
impl Mul<Vector3> for f64 {
    type Output = Vector3;
    fn mul(self, v: Vector3) -> Vector3 {
        v * self
    }
}
impl Div<f64> for Vector3 {
    type Output = Vector3;
    fn div(self, s: f64) -> Vector3 {
        Vector3::new(self.x / s, self.y / s, self.z / s)
    }
}
impl Neg for Vector3 {
    type Output = Vector3;
    fn neg(self) -> Vector3 {
        Vector3::new(-self.x, -self.y, -self.z)
    }
}
impl AddAssign for Vector3 {
    fn add_assign(&mut self, o: Vector3) {
        *self = *self + o;
    }
}
impl SubAssign for Vector3 {
    fn sub_assign(&mut self, o: Vector3) {
        *self = *self - o;
    }
}
impl MulAssign<f64> for Vector3 {
    fn mul_assign(&mut self, s: f64) {
        *self = *self * s;
    }
}
impl DivAssign<f64> for Vector3 {
    fn div_assign(&mut self, s: f64) {
        *self = *self / s;
    }
}

impl From<(f64, f64, f64)> for Vector3 {
    fn from(p: (f64, f64, f64)) -> Vector3 {
        Vector3::new(p.0, p.1, p.2)
    }
}
impl From<Vector3> for (f64, f64, f64) {
    fn from(v: Vector3) -> (f64, f64, f64) {
        (v.x, v.y, v.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nah(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    /// Sollwerte aus echtem pygame-ce 2.5.7, `Vector3(1, 2, 3)`.
    #[test]
    fn die_grundrechnungen_treffen_pygame() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        assert!(nah(v.length(), 3.7416573867739413));
        let n = v.normalize().unwrap();
        assert!(nah(n.x, 0.2672612419124244), "{n:?}");
        assert!(nah(n.y, 0.5345224838248488), "{n:?}");
        assert!(nah(n.z, 0.8017837257372732), "{n:?}");

        let o = Vector3::new(4.0, 5.0, 6.0);
        assert_eq!(v.dot(o), 32.0);
        assert_eq!(v.cross(o), Vector3::new(-3.0, 6.0, -3.0));
        assert!(nah(v.angle_to(o), 12.933154491899135), "{}", v.angle_to(o));

        assert_eq!(
            v.reflect(Vector3::new(0.0, 1.0, 0.0)).unwrap(),
            Vector3::new(1.0, -2.0, 3.0)
        );
        assert_eq!(
            v.project(Vector3::new(1.0, 0.0, 0.0)).unwrap(),
            Vector3::new(1.0, 0.0, 0.0)
        );

        let (r, t, p) = v.as_spherical();
        assert!(nah(r, 3.7416573867739413));
        assert!(nah(t, 36.69922520048988), "{t}");
        assert!(nah(p, 63.43494882292201), "{p}");
    }

    /// Der Unterschied, der diese Datei interessant macht.
    #[test]
    fn allgemeines_drehen_faengt_rechte_winkel_ab_achsendrehen_nicht() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        let z = Vector3::new(0.0, 0.0, 1.0);

        // Allgemein: exakt.
        assert_eq!(v.rotate(90.0, z).unwrap(), Vector3::new(-2.0, 1.0, 3.0));

        // Achsenparallel: **nicht** exakt - und genau so misst es pygame.
        let rz = v.rotate_z(90.0);
        assert_eq!(rz.x, -2.0);
        assert_eq!(rz.y, 1.0000000000000002, "pygame liefert hier auch nicht 1.0");
        assert_eq!(rz.z, 3.0);

        // rotate_x trifft die 1.0 zufaellig doch - auch das gemessen.
        assert_eq!(v.rotate_x(90.0), Vector3::new(1.0, -3.0, 2.0));

        let ry = v.rotate_y(90.0);
        assert_eq!(ry.x, 3.0);
        assert_eq!(ry.y, 2.0);
        assert!(nah(ry.z, -0.9999999999999998), "{}", ry.z);
    }

    #[test]
    fn eine_achse_der_laenge_null_geht_nicht() {
        let v = Vector3::new(1.0, 2.0, 3.0);
        assert_eq!(v.rotate(90.0, Vector3::null()), Err(Fehler::LaengeNull));
        assert_eq!(Vector3::null().normalize(), Err(Fehler::LaengeNull));
    }

    /// `slerp` bei drei Achsen - bitgenau, Sollwerte aus pygame-ce 2.5.7.
    ///
    /// Der Fall `(-1.5, 2.25, 0.5)` mit `t = 0` ist der interessante:
    /// Das Ergebnis ist **nicht** der Ausgangsvektor, sondern
    /// `-1.4999999999999998`. Wer `t == 0` abkuerzt und den Vektor
    /// durchreicht, ist hier schon daneben.
    // `1.414213562373095` ist ein **Messwert** aus pygame, der zufaellig
    // die Wurzel aus 2 ist - `clamp_magnitude((3,4,5), 2)` liefert nun
    // einmal genau das. Die Konstante `SQRT_2` waere hier keine
    // Verbesserung, sondern eine Behauptung ueber die Herkunft der Zahl.
    #[allow(clippy::approx_constant)]
    #[test]
    fn slerp_und_clamp_sind_bitgenau() {
        let ziel = Vector3::new(1.0, 2.0, 3.0);
        /// Ausgangsvektor, `t`, und was herauskommen muss.
        type Fall = ((f64, f64, f64), f64, (f64, f64, f64));
        let faelle: &[Fall] = &[
            ((3.0, 4.0, 5.0), 0.0, (3.0, 4.0, 5.0)),
            (
                (3.0, 4.0, 5.0),
                0.25,
                (2.4092928413711507, 3.4917969225486374, 4.574301003726124),
            ),
            ((3.0, 4.0, 5.0), 1.0, (1.0, 2.0, 3.0)),
            (
                (3.0, 4.0, 5.0),
                -0.5,
                (-1.8774523082275394, -2.9870025668231284, -4.096552825418717),
            ),
            (
                (-1.5, 2.25, 0.5),
                0.0,
                (-1.4999999999999998, 2.25, 0.5),
            ),
            (
                (-1.5, 2.25, 0.5),
                0.25,
                (-1.1065122732569608, 2.520154188024378, 1.1882524512179748),
            ),
            (
                (-1.5, 2.25, 0.5),
                1.0,
                (0.9999999999999999, 1.9999999999999998, 3.0),
            ),
            (
                (-1.5, 2.25, 0.5),
                -0.5,
                (0.5325716327925177, -2.5896092020636163, -1.8830017374783057),
            ),
        ];
        for &((x, y, z), t, soll) in faelle {
            let got = Vector3::new(x, y, z).slerp(ziel, t).unwrap();
            assert_eq!(
                (got.x, got.y, got.z),
                soll,
                "slerp von ({x}, {y}, {z}) mit t = {t}"
            );
        }

        let v = Vector3::new(3.0, 4.0, 5.0);
        let c = v.clamp_magnitude(0.0, 2.0).unwrap();
        assert_eq!(
            (c.x, c.y, c.z),
            (0.848528137423857, 1.131370849898476, 1.414213562373095)
        );
        let c = v.clamp_magnitude(10.0, 20.0).unwrap();
        assert_eq!(
            (c.x, c.y, c.z),
            (4.242640687119285, 5.65685424949238, 7.071067811865475)
        );
        let v = Vector3::new(-1.5, 2.25, 0.5);
        let c = v.clamp_magnitude(0.0, 2.0).unwrap();
        assert_eq!(
            (c.x, c.y, c.z),
            (-1.0909090909090908, 1.6363636363636365, 0.36363636363636365)
        );
    }
}
