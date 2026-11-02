//! rustgame-math
//! =============
//! Gegenstueck zu `pygame.math` ([`Vector2`], [`Vector3`] und die freien
//! Funktionen) sowie zu `pygame.mask` ([`Mask`]).
//!
//! ## Warum hier nichts "aufgeraeumt" wurde
//!
//! Vektorrechnung sieht nach Schulmathematik aus, und genau das ist die
//! Falle: pygames Umsetzung hat Sonderfaelle, die mathematisch ueberfluessig
//! sind, aber das Ergebnis im letzten Bit aendern - und die sich in einem
//! Spiel aufsummieren.
//!
//! Zwei Beispiele, beide an echtem pygame-ce 2.5.7 gemessen:
//!
//! ```text
//! Vector2(3,4).rotate(90)            ->  (-4.0, 3.0)                 exakt
//! Vector3(1,2,3).rotate(90, z-Achse) ->  (-2.0, 1.0,                3.0)
//! Vector3(1,2,3).rotate_z(90)        ->  (-2.0, 1.0000000000000002, 3.0)
//! ```
//!
//! Die ersten beiden fangen rechte Winkel ab, der dritte nicht. Wer alle
//! drei ueber `sin`/`cos` baut, bekommt bei den ersten beiden Werte wie
//! `-4.000000000000001` - und ein Objekt, das viermal um 90 Grad gedreht
//! wurde, steht nicht mehr am Ausgangspunkt.
//!
//! Deshalb ist auch hier `src_c/math.c` die Vorlage und wurde uebertragen,
//! nicht nachempfunden.

// **Kein `unsafe` in dieser Kiste, und das prueft der Uebersetzer.**
//
// Hier stand der Satz lange nur in der Beschreibung - und eine Behauptung,
// die niemand nachhaelt, veraltet still. Siehe
// `rustgame-py/src/puffer.rs`: dort liegt das `unsafe` des Projekts, und
// nur dort ist der Lint ausdruecklich abgeschaltet.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod geometry;
pub mod mask;
pub mod vector2;
pub mod vector3;

pub use geometry::{Circle, Line};
pub use mask::Mask;
pub use vector2::{Fehler, Vector2, EPSILON};
pub use vector3::Vector3;

/// Bogenmass in Grad - **so, wie pygame es rechnet**.
///
/// ```c
/// #define RAD2DEG(angle) ((angle) * 180. / M_PI)
/// ```
///
/// Erst mal 180, **dann** durch pi. Rusts `to_degrees()` multipliziert
/// stattdessen mit der vorberechneten Konstante `180/pi` - mathematisch
/// dasselbe, im letzten Bit nicht. Das zieht sich durch jede
/// Winkelangabe: `angle_to`, `as_polar`, `as_spherical`.
#[inline]
pub fn rad2deg(winkel: f64) -> f64 {
    winkel * 180.0 / std::f64::consts::PI
}

/// Grad in Bogenmass - das Gegenstueck, ebenso geklammert.
///
/// ```c
/// #define DEG2RAD(angle) ((angle) * M_PI / 180.)
/// ```
#[inline]
pub fn deg2rad(winkel: f64) -> f64 {
    winkel * std::f64::consts::PI / 180.0
}

/// Begrenzt einen Wert auf `[min, max]` - `pygame.math.clamp`.
pub fn clamp(wert: f64, min: f64, max: f64) -> f64 {
    if wert < min {
        min
    } else if wert > max {
        max
    } else {
        wert
    }
}

/// Geradlinige Mischung zwischen zwei Zahlen.
///
/// ```c
/// static inline double
/// lerp(double a, double b, double v)
/// {
///     return a + (b - a) * v;
/// }
/// ```
///
/// **Ohne Begrenzung** - und `a + (b-a)*v`, nicht `a*(1-v) + b*v`. Hier
/// stand beides einmal andersherum, mit dem Vermerk, pygame begrenze
/// still. Gemessen wurde dabei `pygame.math.lerp`, und das stimmt auch:
/// nur sitzt das Begrenzen in `math_lerp`, also in der **Huelle**, und
/// gilt deshalb nicht fuer `remap`, das dieselbe Zeile ohne Begrenzung
/// benutzt:
///
/// ```text
/// pygame.math.lerp(0, 10, -0.5)                ->  0.0    (begrenzt)
/// pygame.math.remap(0, 10, 0, 100, -1)         -> -10.0   (nicht)
/// ```
///
/// Der zweite Wert war der Beleg: mit begrenztem `lerp` kaeme `0.0`.
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Die Umkehrung von [`lerp`]: wo zwischen `a` und `b` liegt `wert`?
///
/// ```c
/// return (v - a) / (b - a);
/// ```
///
/// Ohne Sonderfall fuer `a == b`: pygame faengt den in `math_invlerp`
/// und `math_remap` ab, **bevor** gerechnet wird, und wirft dort einen
/// `ValueError`. Ein stilles `0.0` an dieser Stelle wuerde genau das
/// verdecken.
pub fn invlerp(a: f64, b: f64, wert: f64) -> f64 {
    (wert - a) / (b - a)
}

/// Rechnet einen Wert aus einem Bereich in einen anderen um.
///
/// `lerp(c, d, invlerp(a, b, v))` - zeilengleich mit `math_remap`, also
/// **ohne** Begrenzung, siehe [`lerp`].
pub fn remap(a_von: f64, a_bis: f64, b_von: f64, b_bis: f64, wert: f64) -> f64 {
    lerp(b_von, b_bis, invlerp(a_von, a_bis, wert))
}

/// Weiche Kurve zwischen zwei Zahlen - `pygame.math.smoothstep`.
///
/// Achtung, die Reihenfolge der Angaben ist in pygame `(a, b, t)`, und
/// `t` wird **vor** der Kurve auf 0..1 begrenzt.
pub fn smoothstep(a: f64, b: f64, t: f64) -> f64 {
    let t = if t <= 0.0 {
        0.0
    } else if t >= 1.0 {
        1.0
    } else {
        t * t * (3.0 - 2.0 * t)
    };
    a * (1.0 - t) + b * t
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nah(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    /// Sollwerte aus echtem pygame-ce 2.5.7.
    #[test]
    fn die_freien_funktionen_treffen_pygame() {
        assert_eq!(clamp(5.0, 1.0, 3.0), 3.0);
        assert_eq!(clamp(0.0, 1.0, 3.0), 1.0);
        assert_eq!(clamp(2.0, 1.0, 3.0), 2.0);

        assert_eq!(lerp(0.0, 10.0, 0.25), 2.5);
        // **Extrapoliert.** Das Begrenzen steht in `math_lerp`, also in
        // der Huelle - hier nicht, sonst stimmte `remap` nicht mehr.
        assert_eq!(lerp(0.0, 10.0, -0.5), -5.0);
        assert_eq!(lerp(0.0, 10.0, 1.5), 15.0);
        assert_eq!(invlerp(0.0, 10.0, 2.5), 0.25);
        assert_eq!(remap(0.0, 10.0, 0.0, 100.0, 2.5), 25.0);
        // Sollwert aus echtem pygame: `remap` begrenzt nicht.
        assert_eq!(remap(0.0, 10.0, 0.0, 100.0, -1.0), -10.0);
        assert!(nah(smoothstep(0.0, 10.0, 0.25), 1.5625));

        // Ausserhalb von 0..1 wird begrenzt, nicht extrapoliert.
        assert_eq!(smoothstep(0.0, 10.0, -1.0), 0.0);
        assert_eq!(smoothstep(0.0, 10.0, 2.0), 10.0);
    }
}
