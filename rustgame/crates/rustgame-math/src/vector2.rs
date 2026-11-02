//! vector2.rs
//! ==========
//! Gegenstueck zu `pygame.math.Vector2`.
//!
//! Gerechnet wird durchgehend in `f64`, genau wie in pygame (`double`).
//! Wo pygame Sonderfaelle kennt, stehen sie hier auch - die Vorlage ist
//! `src_c/math.c` aus pygame-ce 2.5.7, und sie wurde uebertragen, nicht
//! nachempfunden. Warum das wichtig ist, zeigt [`Vector2::rotate`].

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// Ab wann gilt eine Laenge als null und ein Winkel als rechter Winkel.
///
/// Derselbe Wert wie `pygame.Vector2().epsilon` - nachgemessen: `1e-06`.
pub const EPSILON: f64 = 1e-6;

/// Was schiefgehen kann. pygame wirft hier `ValueError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fehler {
    /// Ein Vektor der Laenge null laesst sich nicht normieren.
    LaengeNull,
    /// `lerp` nimmt nur Werte von 0 bis 1 - **anders als `slerp`** und
    /// anders als die freie Funktion `lerp` des Moduls, die stattdessen
    /// begrenzt. Gemessen, nicht angenommen.
    MischwertAusserhalb,
    /// `slerp` nimmt **-1 bis 1** - der Unterschied zu
    /// [`MischwertAusserhalb`](Self::MischwertAusserhalb) ist echt, und
    /// auch der Wortlaut endet anders (mit Punkt).
    SlerpMischwertAusserhalb,
    /// `slerp` mit einem Vektor der Laenge null. pygame sagt hier
    /// **etwas anderes** als beim Normieren, deshalb ein eigener Fall.
    SlerpNullVektor,
    /// Bei 180 Grad gibt es keinen kuerzeren Bogen - pygame weigert sich,
    /// statt einen der beiden zu raten.
    Slerp180,
    /// `clamp_magnitude(min, max)` mit `min > max`.
    ClampMinUeberMax,
    /// `clamp_magnitude` mit einer negativen Grenze.
    ClampNegativ,
    /// `clamp_magnitude` mit Laenge null und `min > 0` - es gibt keine
    /// Richtung, in die man den Vektor verlaengern koennte.
    ClampNullLaenge,
    /// `project` auf einen Vektor, dessen Laengenquadrat unter `epsilon`
    /// liegt. Die Schranke ist **nicht** null, siehe
    /// [`Vector2::project_mit_eps`].
    ProjektionAufNullvektor,
    /// `reflect` an einer Normalen der Laenge null. pygame sagt hier
    /// **nicht** dasselbe wie beim Normieren, obwohl die Rechnung darauf
    /// hinauslaeuft - siehe [`Vector2::reflect_mit_eps`].
    NormaleNullLaenge,
    /// `scale_to_length` bei Laenge null - wieder ein eigener Wortlaut.
    SkalierenNullLaenge,
}

impl std::fmt::Display for Fehler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Wortgleich mit pygame, damit eine Fehlermeldung aus der
            // Python-Huelle nicht ploetzlich anders lautet.
            Fehler::LaengeNull => write!(f, "Can't normalize Vector of length zero"),
            Fehler::MischwertAusserhalb => write!(f, "Argument 2 must be in range [0, 1]"),
            Fehler::SlerpMischwertAusserhalb => {
                write!(f, "Argument 2 must be in range [-1, 1].")
            }
            Fehler::SlerpNullVektor => write!(f, "can't use slerp with Zero-Vector"),
            Fehler::Slerp180 => write!(f, "SLERP with 180 degrees is undefined."),
            Fehler::ClampMinUeberMax => {
                write!(f, "Argument min_length cannot exceed max_length")
            }
            Fehler::ClampNegativ => {
                write!(f, "Arguments to Vector clamp must be non-negative")
            }
            Fehler::ClampNullLaenge => write!(
                f,
                "Cannot clamp a vector with zero length with a min_length greater than 0"
            ),
            Fehler::ProjektionAufNullvektor => {
                write!(f, "Cannot project onto a vector with zero length")
            }
            Fehler::NormaleNullLaenge => write!(f, "Normal must not be of length zero."),
            Fehler::SkalierenNullLaenge => {
                write!(f, "Cannot scale a vector with zero length")
            }
        }
    }
}

impl std::error::Error for Fehler {}

/// Ein Vektor in der Ebene.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vector2 {
    /// Der Anteil nach rechts.
    pub x: f64,
    /// Der Anteil nach unten - **nach unten**, wie bei pygame.
    pub y: f64,
}

impl Vector2 {
    /// Aus zwei Zahlen.
    pub const fn new(x: f64, y: f64) -> Vector2 {
        Vector2 { x, y }
    }

    /// Der Nullvektor.
    pub const fn null() -> Vector2 {
        Vector2 { x: 0.0, y: 0.0 }
    }

    // ----- Laenge --------------------------------------------------------

    /// Die Laenge.
    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    /// Die Laenge zum Quadrat - ohne Wurzel, zum Vergleichen.
    pub fn length_squared(self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Wie [`length`](Self::length) - pygame hat beide Namen.
    pub fn magnitude(self) -> f64 {
        self.length()
    }

    /// Zweitname von [`Vector2::length_squared`] - pygame hat beide.
    pub fn magnitude_squared(self) -> f64 {
        self.length_squared()
    }

    /// Derselbe Vektor mit Laenge 1.
    ///
    /// Bei Laenge null gibt pygame einen `ValueError` - hier ein
    /// [`Fehler::LaengeNull`], damit die Python-Huelle daraus wieder
    /// dieselbe Ausnahme machen kann.
    pub fn normalize(self) -> Result<Vector2, Fehler> {
        let l = self.length();
        if l == 0.0 {
            return Err(Fehler::LaengeNull);
        }
        Ok(Vector2::new(self.x / l, self.y / l))
    }

    /// Hat der Vektor bereits Laenge 1?
    ///
    /// Verglichen wird das **Quadrat** der Laenge gegen `1 +- epsilon` -
    /// so macht es pygame, und deshalb ist die Grenze nicht symmetrisch
    /// zur Laenge selbst.
    pub fn is_normalized(self) -> bool {
        self.is_normalized_mit_eps(EPSILON)
    }

    /// Wie [`is_normalized`](Self::is_normalized), aber mit eigener
    /// Schranke.
    ///
    /// In pygame ist `epsilon` **je Vektor** einstellbar und wirkt
    /// wirklich - gemessen: `Vector2(0.9, 0).is_normalized()` ist `False`,
    /// mit `v.epsilon = 0.5` aber `True`. Der Kern rechnet mit `Copy`-
    /// Werten und traegt keinen Zustand mit sich; die Schranke kommt
    /// deshalb als Beiwert herein, und die Python-Huelle merkt sich den
    /// Wert je Objekt.
    ///
    /// Gemessen ist auch, dass die Schranke **nicht vererbt** wird: ein
    /// abgeleiteter Vektor (`v.rotate(10)`, `v.copy()`, `v + w`) hat
    /// wieder `1e-06`. Ohne diese Messung haette die Huelle sie
    /// naheliegenderweise weitergereicht - und waere falsch gewesen.
    pub fn is_normalized_mit_eps(self, eps: f64) -> bool {
        (self.length_squared() - 1.0).abs() < eps
    }

    /// Auf eine bestimmte Laenge bringen.
    ///
    /// Kurzform von
    /// [`scale_to_length_mit_eps`](Self::scale_to_length_mit_eps).
    pub fn scale_to_length(self, laenge: f64) -> Result<Vector2, Fehler> {
        self.scale_to_length_mit_eps(laenge, EPSILON)
    }

    /// Dasselbe mit eigener Schranke - `Vector2.epsilon`.
    ///
    /// ```c
    /// if (old_length < self->epsilon) {
    ///     return RAISE(PyExc_ValueError,
    ///                  "Cannot scale a vector with zero length");
    /// }
    /// fraction = new_length / old_length;
    /// for (i = 0; i < self->dim; ++i) {
    ///     self->coords[i] *= fraction;
    /// }
    /// ```
    ///
    /// Hier stand `normalize()? * laenge`. Das ist derselbe Gedanke und
    /// zwei Unterschiede: die Schranke ist `epsilon`, nicht null (und der
    /// Wortlaut ein anderer als beim Normieren), und es wird **einmal**
    /// gerundet statt zweimal - `x * (neu/alt)` gegen `(x/alt) * neu`.
    pub fn scale_to_length_mit_eps(self, laenge: f64, eps: f64) -> Result<Vector2, Fehler> {
        let alt = self.length();
        if alt < eps {
            return Err(Fehler::SkalierenNullLaenge);
        }
        Ok(self * (laenge / alt))
    }

    /// Die Laenge auf einen Bereich begrenzen.
    ///
    /// Mit nur einem Wert ist das die Obergrenze; pygame kennt beide
    /// Formen (`clamp_magnitude(max)` und `clamp_magnitude(min, max)`).
    ///
    /// Zeilengetreu aus `vector_clamp_magnitude_ip` in `src_c/math.c`.
    /// Zwei Dinge daran sind **nicht** beliebig:
    ///
    /// 1. Verglichen wird das **Quadrat** der Laenge gegen `max*max`,
    ///    nicht die Laenge gegen `max`. Beides ist mathematisch dasselbe,
    ///    in Gleitkomma aber nicht.
    /// 2. Skaliert wird mit `faktor = grenze / sqrt(laenge2)` und einer
    ///    **Multiplikation** je Achse - nicht ueber
    ///    [`scale_to_length`](Self::scale_to_length), das erst normiert
    ///    und dann streckt. Das ist eine Rundung mehr:
    ///
    /// ```text
    /// Vector2(3, 4).clamp_magnitude(2)
    ///   pygame / hier        (1.2000000000000002, 1.6)
    ///   ueber normalize()    (1.2,                1.6)
    /// ```
    ///
    /// Die Reihenfolge der Pruefungen ist ebenfalls uebernommen:
    /// `min > max` schlaegt zu, **bevor** eine negative Grenze auffaellt.
    pub fn clamp_magnitude(self, min_length: f64, max_length: f64) -> Result<Vector2, Fehler> {
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
        // Kein `else` - so steht es in der Vorlage.
        if laenge2 < min_length * min_length {
            faktor = min_length / laenge2.sqrt();
        }
        Ok(Vector2::new(self.x * faktor, self.y * faktor))
    }

    // ----- Drehen --------------------------------------------------------

    /// Dreht um `grad` gegen den Uhrzeigersinn.
    ///
    /// Zeilengetreu aus `_vector2_rotate_helper` in `src_c/math.c`. Der
    /// Sonderfall fuer Vielfache von 90 Grad ist **kein** Beiwerk:
    ///
    /// ```text
    /// rotate(90)  auf (3, 4)  ->  (-4.0, 3.0)      exakt
    /// rotate(1e-9) auf (3, 4) ->  (3.0, 4.0)       unveraendert
    /// ```
    ///
    /// Wer schlicht `sin`/`cos` einsetzt, bekommt bei 90 Grad
    /// `(-4.000000000000001, 2.9999999999999996)` - und ein Spiel, das
    /// viermal um 90 Grad dreht, steht danach nicht mehr am Ausgangspunkt.
    pub fn rotate(self, grad: f64) -> Vector2 {
        self.rotate_rad(crate::deg2rad(grad))
    }

    /// Wie [`rotate`](Self::rotate), aber im Bogenmass.
    pub fn rotate_rad(self, winkel: f64) -> Vector2 {
        self.rotate_rad_mit_eps(winkel, EPSILON)
    }

    /// Wie [`rotate_rad`](Self::rotate_rad), aber mit eigener Schranke
    /// fuer "dicht genug an einem rechten Winkel" - siehe
    /// [`is_normalized_mit_eps`](Self::is_normalized_mit_eps).
    pub fn rotate_rad_mit_eps(self, winkel: f64, eps: f64) -> Vector2 {
        use std::f64::consts::{FRAC_PI_2, TAU};
        
        // Erst in den Bereich [0, 2 PI) bringen.
        let mut a = winkel % TAU;
        if a < 0.0 {
            a += TAU;
        }

        // Liegt der Winkel dicht genug an einem rechten Winkel?
        if (a + eps) % FRAC_PI_2 < 2.0 * eps {
            return match ((a + eps) / FRAC_PI_2) as i32 {
                // 0 und 360 Grad sind dasselbe.
                0 | 4 => self,
                1 => Vector2::new(-self.y, self.x),
                2 => Vector2::new(-self.x, -self.y),
                3 => Vector2::new(self.y, -self.x),
                // Kann nicht vorkommen; lieber rechnen als abstuerzen.
                _ => Vector2::new(
                    a.cos() * self.x - a.sin() * self.y,
                    a.sin() * self.x + a.cos() * self.y,
                ),
            };
        }

        let (sin, cos) = (a.sin(), a.cos());
        Vector2::new(cos * self.x - sin * self.y, sin * self.x + cos * self.y)
    }

    /// Der Winkel zu einem anderen Vektor, in Grad.
    pub fn angle_to(self, other: Vector2) -> f64 {
        let a = other.y.atan2(other.x) - self.y.atan2(self.x);
        crate::rad2deg(a)
    }

    /// Der eigene Winkel gegen die x-Achse, in Grad.
    pub fn angle(self) -> f64 {
        crate::rad2deg(self.y.atan2(self.x))
    }

    /// Der eigene Winkel gegen die x-Achse, im **Bogenmass**.
    ///
    /// In pygame eine nur lesbare Eigenschaft (`v.angle_rad = 1` gibt
    /// `AttributeError`). `Vector3` hat sie **nicht** - gemessen.
    pub fn angle_rad(self) -> f64 {
        self.y.atan2(self.x)
    }

    /// `(Laenge, Winkel in Grad)`.
    pub fn as_polar(self) -> (f64, f64) {
        (self.length(), crate::rad2deg(self.y.atan2(self.x)))
    }

    /// Aus Laenge und Winkel in Grad.
    pub fn from_polar(laenge: f64, grad: f64) -> Vector2 {
        let w = crate::deg2rad(grad);
        Vector2::new(laenge * w.cos(), laenge * w.sin())
    }

    // ----- Verhaeltnisse zu anderen Vektoren -----------------------------

    /// Das Skalarprodukt.
    pub fn dot(self, other: Vector2) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Das Kreuzprodukt - in der Ebene eine einzelne Zahl.
    pub fn cross(self, other: Vector2) -> f64 {
        self.x * other.y - self.y * other.x
    }

    /// Der Abstand zu `other`.
    pub fn distance_to(self, other: Vector2) -> f64 {
        (other - self).length()
    }

    /// Der Abstand zum Quadrat - ohne Wurzel.
    pub fn distance_squared_to(self, other: Vector2) -> f64 {
        (other - self).length_squared()
    }

    /// Spiegelt an einer Flaeche mit dieser Normalen.
    ///
    /// Kurzform von [`reflect_mit_eps`](Self::reflect_mit_eps).
    pub fn reflect(self, normal: Vector2) -> Result<Vector2, Fehler> {
        self.reflect_mit_eps(normal, EPSILON)
    }

    /// Dasselbe mit eigener Schranke - `Vector2.epsilon`.
    ///
    /// ```c
    /// norm_length = _scalar_product(norm_coords, norm_coords, dim);
    /// if (norm_length < epsilon) {
    ///     PyErr_SetString(PyExc_ValueError, "Normal must not be of length zero.");
    /// }
    /// if (norm_length != 1) { … /= sqrt(norm_length); }
    /// ```
    ///
    /// Die Schranke steht auf dem **Laengenquadrat**, und normiert wird
    /// nur, wenn das Quadrat nicht schon genau 1 ist. Hier stand vorher
    /// `normal.normalize()?` - gleiche Rechnung, aber mit der Meldung
    /// **des Normierens** (`Can't normalize Vector of length zero`), und
    /// die kommt an dieser Stelle in pygame nicht vor.
    pub fn reflect_mit_eps(self, normal: Vector2, eps: f64) -> Result<Vector2, Fehler> {
        let q = normal.dot(normal);
        if q < eps {
            return Err(Fehler::NormaleNullLaenge);
        }
        let n = if q != 1.0 { normal / q.sqrt() } else { normal };
        Ok(self - n * (2.0 * self.dot(n)))
    }

    /// Der Anteil dieses Vektors in Richtung `other`.
    ///
    /// Kurzform von [`project_mit_eps`](Self::project_mit_eps) mit
    /// pygames Vorgabe.
    pub fn project(self, other: Vector2) -> Result<Vector2, Fehler> {
        self.project_mit_eps(other, EPSILON)
    }

    /// Dasselbe mit eigener Schranke - `Vector2.epsilon`.
    ///
    /// ```c
    /// if (b_dot_b < self->epsilon) {
    ///     PyErr_SetString(PyExc_ValueError,
    ///                     "Cannot project onto a vector with zero length");
    /// ```
    ///
    /// Die Schranke steht auf dem **Laengenquadrat**, nicht auf der
    /// Laenge, und sie ist nicht null: `Vector2(1,2).project((1e-4, 0))`
    /// wirft in pygame, obwohl der Vektor nicht der Nullvektor ist
    /// (`1e-8 < 1e-6`). Nachgemessen; die naheliegende Fassung
    /// `if q == 0.0` stand hier vorher und liess genau diese Faelle
    /// durch.
    pub fn project_mit_eps(self, other: Vector2, eps: f64) -> Result<Vector2, Fehler> {
        let q = other.dot(other);
        if q < eps {
            return Err(Fehler::ProjektionAufNullvektor);
        }
        Ok(other * (self.dot(other) / q))
    }

    /// Geradlinige Mischung.
    ///
    /// **`t` muss zwischen 0 und 1 liegen** - pygame wirft sonst einen
    /// `ValueError`, statt zu extrapolieren. Das ist leicht zu uebersehen,
    /// weil die drei Nachbarn es anders halten: [`smoothstep`](Self::smoothstep)
    /// begrenzt stillschweigend, [`slerp`](Self::slerp) laesst negative
    /// Werte ausdruecklich zu (sie drehen den langen Weg herum), und die
    /// freie Funktion [`crate::lerp`] begrenzt ebenfalls.
    pub fn lerp(self, other: Vector2, t: f64) -> Result<Vector2, Fehler> {
        if !(0.0..=1.0).contains(&t) {
            return Err(Fehler::MischwertAusserhalb);
        }
        Ok(Vector2::new(
            self.x * (1.0 - t) + other.x * t,
            self.y * (1.0 - t) + other.y * t,
        ))
    }

    /// Weiche Mischung (`3t^2 - 2t^3`), `t` wird auf 0..1 begrenzt.
    ///
    /// Uebertragen aus `vector_smoothstep`; das Begrenzen von `t`
    /// **vor** der Kurve ist Teil davon.
    pub fn smoothstep(self, other: Vector2, t: f64) -> Vector2 {
        let t = if t <= 0.0 {
            0.0
        } else if t >= 1.0 {
            1.0
        } else {
            t * t * (3.0 - 2.0 * t)
        };
        Vector2::new(
            self.x * (1.0 - t) + other.x * t,
            self.y * (1.0 - t) + other.y * t,
        )
    }

    /// Mischung ueber den Kreisbogen.
    ///
    /// Zeilengetreu aus `vector_slerp` in `src_c/math.c`. Die erste
    /// Fassung hier war "drehen, normieren, auf Laenge bringen" - das ist
    /// dieselbe Kurve, aber eine andere Rechnung, und sie lag in **elf**
    /// von zwoelf Proben im letzten Bit daneben. pygame rechnet
    /// stattdessen mit drei Faktoren:
    ///
    /// ```text
    /// f0 = ((l2 - l1) * t + l1) / sin(w)
    /// f1 = sin(w * (1 - t)) / l1
    /// f2 = sin(w * t)       / l2
    /// erg = (self * f1 + other * f2) * f0
    /// ```
    ///
    /// Drei Feinheiten, die eine Rekonstruktion verfehlt:
    ///
    /// * Bei `t < 0` wird **immer** `2 PI` abgezogen (der lange Bogen),
    ///   nicht `sign(w) * 2 PI`. Und das geschieht **vor** dem
    ///   Vorzeichenwechsel unten, nicht danach.
    /// * Die Richtung entscheidet `self.x * other.y < self.y * other.x`,
    ///   also das Kreuzprodukt **ausgeschrieben**. Bei drei Achsen nimmt
    ///   pygame dafuer ebenfalls nur die ersten beiden.
    /// * Nahe 0 und nahe 360 Grad wird auf [`lerp`](Self::lerp)
    ///   ausgewichen, weil `1/sin(w)` dort davonlaeuft - und bei 180 Grad
    ///   gibt es einen **Fehler** statt eines geratenen Bogens.
    pub fn slerp(self, other: Vector2, t: f64) -> Result<Vector2, Fehler> {
        self.slerp_mit_eps(other, t, EPSILON)
    }

    /// Wie [`slerp`](Self::slerp), aber mit eigener Schranke - siehe
    /// [`is_normalized_mit_eps`](Self::is_normalized_mit_eps).
    pub fn slerp_mit_eps(self, other: Vector2, t: f64, eps: f64) -> Result<Vector2, Fehler> {
        use std::f64::consts::PI;
                if t.abs() > 1.0 {
            return Err(Fehler::SlerpMischwertAusserhalb);
        }
        let length1 = self.dot(self).sqrt();
        let length2 = other.dot(other).sqrt();
        if length1 < eps || length2 < eps {
            return Err(Fehler::SlerpNullVektor);
        }
        // Begrenzt, damit `acos` kein NaN liefert - die Vorlage schreibt
        // das von Hand aus, `clamp` tut dasselbe. (Es tut auch bei NaN
        // dasselbe: `clamp` gibt NaN zurueck und geraet nur dann in
        // Panik, wenn die **Grenzen** verkehrt herum stehen. Hier stand
        // einmal die Behauptung, `clamp` wuerde bei NaN panisch - das
        // war ungeprueft und falsch.)
        let tmp = (self.dot(other) / (length1 * length2)).clamp(-1.0, 1.0);
        let mut winkel = tmp.acos();
        let mut t = t;
        if t < 0.0 {
            winkel -= 2.0 * PI;
            t = -t;
        }
        if self.x * other.y < self.y * other.x {
            winkel *= -1.0;
        }
        if winkel.abs() < eps || (winkel.abs() - 2.0 * PI).abs() < eps {
            return Ok(Vector2::new(
                self.x * (1.0 - t) + other.x * t,
                self.y * (1.0 - t) + other.y * t,
            ));
        }
        if (winkel.abs() - PI).abs() < eps {
            return Err(Fehler::Slerp180);
        }
        let f0 = ((length2 - length1) * t + length1) / winkel.sin();
        let f1 = (winkel * (1.0 - t)).sin() / length1;
        let f2 = (winkel * t).sin() / length2;
        Ok(Vector2::new(
            (self.x * f1 + other.x * f2) * f0,
            (self.y * f1 + other.y * f2) * f0,
        ))
    }

    /// Hoechstens `max_distance` weit auf `other` zu.
    ///
    /// Uebertragen aus `_vector_move_towards_helper` - samt der beiden
    /// fruehen Ausstiege, die verhindern, dass `frac` zu `NaN` wird.
    pub fn move_towards(self, other: Vector2, max_distance: f64) -> Vector2 {
        if max_distance == 0.0 {
            return self;
        }
        let delta = other - self;
        let dist = delta.length();
        if dist == 0.0 {
            return self;
        }
        if dist <= max_distance {
            return other;
        }
        self + delta * (max_distance / dist)
    }

    /// Die beiden Zahlen als Tupel.
    pub const fn as_tuple(self) -> (f64, f64) {
        (self.x, self.y)
    }

    /// Auf ganze Zahlen gerundet - fuer alles, was Pixel erwartet.
    pub fn as_ipair(self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }
}

// ----- Rechenzeichen ------------------------------------------------------

impl Add for Vector2 {
    type Output = Vector2;
    fn add(self, o: Vector2) -> Vector2 {
        Vector2::new(self.x + o.x, self.y + o.y)
    }
}
impl Sub for Vector2 {
    type Output = Vector2;
    fn sub(self, o: Vector2) -> Vector2 {
        Vector2::new(self.x - o.x, self.y - o.y)
    }
}
impl Mul<f64> for Vector2 {
    type Output = Vector2;
    fn mul(self, s: f64) -> Vector2 {
        Vector2::new(self.x * s, self.y * s)
    }
}
impl Mul<Vector2> for f64 {
    type Output = Vector2;
    fn mul(self, v: Vector2) -> Vector2 {
        v * self
    }
}
impl Div<f64> for Vector2 {
    type Output = Vector2;
    fn div(self, s: f64) -> Vector2 {
        Vector2::new(self.x / s, self.y / s)
    }
}
impl Neg for Vector2 {
    type Output = Vector2;
    fn neg(self) -> Vector2 {
        Vector2::new(-self.x, -self.y)
    }
}
impl AddAssign for Vector2 {
    fn add_assign(&mut self, o: Vector2) {
        *self = *self + o;
    }
}
impl SubAssign for Vector2 {
    fn sub_assign(&mut self, o: Vector2) {
        *self = *self - o;
    }
}
impl MulAssign<f64> for Vector2 {
    fn mul_assign(&mut self, s: f64) {
        *self = *self * s;
    }
}
impl DivAssign<f64> for Vector2 {
    fn div_assign(&mut self, s: f64) {
        *self = *self / s;
    }
}

impl From<(f64, f64)> for Vector2 {
    fn from(p: (f64, f64)) -> Vector2 {
        Vector2::new(p.0, p.1)
    }
}
impl From<(i32, i32)> for Vector2 {
    fn from(p: (i32, i32)) -> Vector2 {
        Vector2::new(p.0 as f64, p.1 as f64)
    }
}
impl From<Vector2> for (f64, f64) {
    fn from(v: Vector2) -> (f64, f64) {
        (v.x, v.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nah(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    /// Alle Sollwerte an echtem pygame-ce 2.5.7 gemessen.
    #[test]
    fn die_grundrechnungen_treffen_pygame() {
        let v = Vector2::new(3.0, 4.0);
        assert_eq!(v.length(), 5.0);
        assert_eq!(v.length_squared(), 25.0);
        assert_eq!(v.normalize().unwrap(), Vector2::new(0.6, 0.8));
        assert!(!v.is_normalized());
        assert_eq!(EPSILON, 1e-6, "pygame meldet epsilon = 1e-06");

        assert!(nah(v.angle_to(Vector2::new(1.0, 0.0)), -53.13010235415598));
        let (l, w) = v.as_polar();
        assert!(nah(l, 5.0) && nah(w, 53.13010235415598));
        let zurueck = Vector2::from_polar(5.0, 53.13010235415598);
        assert!(nah(zurueck.x, 3.0) && nah(zurueck.y, 4.0));

        assert_eq!(v.cross(Vector2::new(1.0, 2.0)), 2.0);
        assert_eq!(v.reflect(Vector2::new(0.0, 1.0)).unwrap(), Vector2::new(3.0, -4.0));
        assert_eq!(
            v.project(Vector2::new(1.0, 0.0)).unwrap(),
            Vector2::new(3.0, 0.0)
        );
        // Die Schranke steht auf dem Laengenquadrat: 1e-8 < 1e-6.
        assert_eq!(
            v.project(Vector2::new(1e-4, 0.0)),
            Err(Fehler::ProjektionAufNullvektor)
        );
    }

    /// Der Sonderfall, den eine naive Umsetzung verfehlt.
    #[test]
    fn rechte_winkel_sind_exakt() {
        let v = Vector2::new(3.0, 4.0);
        // **Exakt**, nicht ungefaehr: sin/cos allein lieferten hier
        // (-4.000000000000001, 2.9999999999999996).
        assert_eq!(v.rotate(90.0), Vector2::new(-4.0, 3.0));
        assert_eq!(v.rotate(180.0), Vector2::new(-3.0, -4.0));
        assert_eq!(v.rotate(270.0), Vector2::new(4.0, -3.0));
        assert_eq!(v.rotate(360.0), v);
        assert_eq!(v.rotate(-90.0), Vector2::new(4.0, -3.0));

        // Und viermal 90 Grad landet wieder am Anfang - genau das geht
        // ohne den Sonderfall verloren.
        let mut w = v;
        for _ in 0..4 {
            w = w.rotate(90.0);
        }
        assert_eq!(w, v);

        // Ein winziger Winkel liegt innerhalb von Epsilon und aendert nichts.
        assert_eq!(v.rotate(1e-9), v);

        // Ein schraeger Winkel rechnet dagegen ganz normal.
        let r = v.rotate(45.0);
        assert!(nah(r.x, -0.7071067811865475), "{}", r.x);
        assert!(nah(r.y, 4.949747468305834), "{}", r.y);
    }

    #[test]
    fn mischen_und_annaehern_treffen_pygame() {
        let v = Vector2::new(3.0, 4.0);
        let z = Vector2::new(10.0, 10.0);

        let l = v.lerp(z, 0.3).unwrap();
        assert!(nah(l.x, 5.1) && nah(l.y, 5.8), "{l:?}");

        let s = v.smoothstep(z, 0.3);
        assert!(nah(s.x, 4.5120000000000005) && nah(s.y, 5.296), "{s:?}");

        let m = v.move_towards(z, 2.0);
        assert!(nah(m.x, 4.518513204730593), "{m:?}");
        assert!(nah(m.y, 5.3015827469119365), "{m:?}");

        // Weiter als die Entfernung: landet genau auf dem Ziel.
        assert_eq!(v.move_towards(z, 1000.0), z);
        // Null Schritt: bleibt stehen.
        assert_eq!(v.move_towards(z, 0.0), v);

        let c = v.clamp_magnitude(0.0, 2.0).unwrap();
        assert!(nah(c.x, 1.2000000000000002) && nah(c.y, 1.6), "{c:?}");
    }

    /// `slerp` und `clamp_magnitude` **bitgenau**, nicht nur ungefaehr.
    ///
    /// Der Nachbar oben prueft mit `nah` auf `1e-12` - fuer die beiden
    /// hier waere das wertlos. Genau diese Abweichungen sind es, die eine
    /// nachgebaute statt uebertragene Rechnung hinterlaesst: die alte
    /// Fassung von `slerp` traf jede Kurve, aber kein einziges letztes
    /// Bit. Deshalb steht hier `assert_eq!` auf dem `f64` selbst.
    ///
    /// Alle Sollwerte aus echtem pygame-ce 2.5.7.
    #[test]
    fn slerp_und_clamp_sind_bitgenau() {
        let ziel = Vector2::new(1.0, 2.0);

        /// Ausgangsvektor, `t`, und was herauskommen muss.
        type Fall = ((f64, f64), f64, (f64, f64));
        let faelle: &[Fall] = &[
            ((3.0, 4.0), 0.0, (3.0, 4.0)),
            ((3.0, 4.0), 0.25, (2.427851032555723, 3.5599391598074)),
            ((3.0, 4.0), 1.0, (1.0, 2.0)),
            ((3.0, 4.0), -0.25, (3.5599391598073984, -2.4278510325557194)),
            ((-1.5, 2.25), 0.0, (-1.5, 2.25)),
            ((-1.5, 2.25), 0.25, (-0.8263171739768262, 2.4516303085293916)),
            ((-1.5, 2.25), 1.0, (1.0, 2.0)),
            ((-1.5, 2.25), -0.25, (-2.45163030852939, -0.8263171739768261)),
            // Sehr lang und sehr schmal - hier trifft `t = 0` den
            // Ausgangsvektor **nicht** mehr, und das ist richtig so.
            (
                (100_000_000.0, 0.1),
                0.0,
                (99999999.99999999, 0.09999999999999999),
            ),
            (
                (100_000_000.0, 0.1),
                0.25,
                (72145377.35616247, 20494989.875320043),
            ),
            (
                (100_000_000.0, 0.1),
                1.0,
                (1.000000001354202, 2.000000002708404),
            ),
            ((0.0, -7.0), 0.0, (0.0, -7.000000000000001)),
            ((0.0, -7.0), 0.25, (3.604978503139796, -4.555085996206537)),
            ((0.0, -7.0), 1.0, (1.0, 2.0)),
        ];
        for &((x, y), t, soll) in faelle {
            let got = Vector2::new(x, y).slerp(ziel, t).unwrap();
            assert_eq!(
                (got.x, got.y),
                soll,
                "slerp von ({x}, {y}) mit t = {t}"
            );
        }

        let v = Vector2::new(3.0, 4.0);
        let c = v.clamp_magnitude(0.0, 2.0).unwrap();
        assert_eq!((c.x, c.y), (1.2000000000000002, 1.6));
        let c = v.clamp_magnitude(10.0, 20.0).unwrap();
        assert_eq!((c.x, c.y), (6.0, 8.0));
        let v = Vector2::new(-1.5, 2.25);
        let c = v.clamp_magnitude(0.0, 2.0).unwrap();
        assert_eq!((c.x, c.y), (-1.1094003924504583, 1.6641005886756872));
        let c = v.clamp_magnitude(10.0, 20.0).unwrap();
        assert_eq!((c.x, c.y), (-5.54700196225229, 8.320502943378436));
    }

    /// Die Fehlerfaelle - samt Wortlaut und samt Reihenfolge.
    ///
    /// Gemessen: `clamp_magnitude(-1)` meldet **nicht** "non-negative",
    /// sondern "min_length cannot exceed max_length". Der Grund ist die
    /// Reihenfolge in der Vorlage: die eine Grenze ist dann `0`, die
    /// andere `-1`, und `0 > -1` schlaegt zuerst zu.
    #[test]
    fn slerp_und_clamp_melden_wie_pygame() {
        let e = Vector2::new(1.0, 0.0);
        assert_eq!(
            e.slerp(Vector2::new(0.0, 0.0), 0.5),
            Err(Fehler::SlerpNullVektor)
        );
        assert_eq!(
            e.slerp(Vector2::new(0.0, 1.0), 2.0),
            Err(Fehler::SlerpMischwertAusserhalb)
        );
        assert_eq!(
            e.slerp(Vector2::new(-1.0, 0.0), 0.5),
            Err(Fehler::Slerp180)
        );
        assert_eq!(e.clamp_magnitude(5.0, 1.0), Err(Fehler::ClampMinUeberMax));
        assert_eq!(e.clamp_magnitude(0.0, -1.0), Err(Fehler::ClampMinUeberMax));
        assert_eq!(
            e.clamp_magnitude(-2.0, -1.0),
            Err(Fehler::ClampNegativ),
            "beide negativ, min <= max - erst hier greift die zweite Pruefung"
        );
        let null = Vector2::new(0.0, 0.0);
        assert_eq!(
            null.clamp_magnitude(1.0, 2.0),
            Err(Fehler::ClampNullLaenge)
        );
        // Ohne Untergrenze ist der Nullvektor dagegen in Ordnung.
        assert_eq!(null.clamp_magnitude(0.0, 2.0), Ok(null));

        assert_eq!(
            Fehler::SlerpMischwertAusserhalb.to_string(),
            "Argument 2 must be in range [-1, 1].",
            "der Punkt am Ende gehoert dazu - `lerp` sagt es ohne"
        );
    }

    /// Gemessen: `lerp` wirft ausserhalb von 0..1, `smoothstep` nicht.
    #[test]
    fn lerp_ist_waehlerisch_smoothstep_nicht() {
        let v = Vector2::new(3.0, 4.0);
        let z = Vector2::new(10.0, 10.0);
        assert_eq!(v.lerp(z, -0.1), Err(Fehler::MischwertAusserhalb));
        assert_eq!(v.lerp(z, 1.1), Err(Fehler::MischwertAusserhalb));
        assert_eq!(v.lerp(z, 0.0).unwrap(), v);
        assert_eq!(v.lerp(z, 1.0).unwrap(), z);
        assert_eq!(
            Fehler::MischwertAusserhalb.to_string(),
            "Argument 2 must be in range [0, 1]"
        );

        // smoothstep begrenzt dagegen stillschweigend.
        assert_eq!(v.smoothstep(z, -0.1), v);
        assert_eq!(v.smoothstep(z, 1.1), z);
    }

    #[test]
    fn ein_nullvektor_laesst_sich_nicht_normieren() {
        assert_eq!(Vector2::null().normalize(), Err(Fehler::LaengeNull));
        // Wortgleich mit pygame.
        assert_eq!(
            Fehler::LaengeNull.to_string(),
            "Can't normalize Vector of length zero"
        );
    }

    #[test]
    fn die_rechenzeichen_tun_das_naheliegende() {
        let a = Vector2::new(1.0, 2.0);
        let b = Vector2::new(3.0, 5.0);
        assert_eq!(a + b, Vector2::new(4.0, 7.0));
        assert_eq!(b - a, Vector2::new(2.0, 3.0));
        assert_eq!(a * 3.0, Vector2::new(3.0, 6.0));
        assert_eq!(3.0 * a, Vector2::new(3.0, 6.0));
        assert_eq!(b / 2.0, Vector2::new(1.5, 2.5));
        assert_eq!(-a, Vector2::new(-1.0, -2.0));
        assert_eq!(a.dot(b), 13.0);

        let mut c = a;
        c += b;
        c -= a;
        c *= 2.0;
        c /= 2.0;
        assert_eq!(c, b);
    }
}
