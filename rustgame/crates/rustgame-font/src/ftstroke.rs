//! ftstroke.rs
//! ===========
//! FreeTypes **Konturzieher** (`FT_Stroker`), uebertragen aus FreeType
//! 2.11.1 (`src/base/ftstroke.c`), samt der CORDIC-Winkelrechnung, auf der
//! er steht (`src/base/fttrigon.c`).
//!
//! ## Wofuer
//!
//! `font.outline = n` ist SDL_ttfs `TTF_SetFontOutline`: jedes Zeichen wird
//! durch seinen **Rand** ersetzt, und den zieht `FT_Glyph_Stroke` mit
//! Radius `n * 64`, runden Enden und runden Ecken um den gehinteten Umriss.
//! Vorher stand hier ein Weiten des fertigen Bildes um `n` Pixel - gemessen
//! traf das **0 von 240** Zeichenbildern: der Rand ist eine neue Kontur mit
//! Kreisboegen, keine Pixelmorphologie.
//!
//! ## Was uebertragen ist
//!
//! | C-Quelle | hier |
//! |---|---|
//! | `fttrigon.c`: `ft_trig_downscale` (64-Bit-Fassung), `ft_trig_prenorm`, `ft_trig_pseudo_rotate`, `ft_trig_pseudo_polarize`, `FT_Tan`, `FT_Atan2`, `FT_Vector_Unit`, `FT_Vector_Rotate`, `FT_Vector_Length`, `FT_Vector_From_Polar`, `FT_Angle_Diff` | die `fn`s unter "Winkel" |
//! | `ftstroke.c`: `ft_conic_*`, `ft_cubic_*`, `ft_stroke_border_*`, `ft_stroker_*`, `FT_Stroker_LineTo/ConicTo/CubicTo/BeginSubPath/EndSubPath/ParseOutline/Export` | [`Zieher`] |
//! | `FT_Glyph_Stroke` | [`konturieren`] |
//!
//! Die Winkel sind 16.16-Grad (`FT_Angle`), alle Laengen 26.6. Ein `long`
//! hat unter Windows 32 Bit; die Werte hier bleiben weit darunter.

use crate::fixed::{div_fix, mul_div, mul_fix};
use crate::ftgrays::{Art, Umriss};

// ----- Winkel (`fttrigon.c`) -------------------------------------------------

/// `FT_ANGLE_PI`.
const PI: i64 = 180 << 16;
/// `FT_ANGLE_2PI`.
const PI2X: i64 = 360 << 16;
/// `FT_ANGLE_PI2`.
const PI_2: i64 = 90 << 16;
/// `FT_ANGLE_PI4`.
const PI_4: i64 = 45 << 16;

/// `FT_TRIG_SCALE`.
const TRIG_SCALE: u64 = 0xDBD9_5B16;
/// `FT_TRIG_SAFE_MSB`.
const TRIG_SAFE_MSB: i32 = 29;
/// `FT_TRIG_MAX_ITERS`.
const TRIG_MAX_ITERS: i32 = 23;

/// `ft_trig_arctan_table`.
const ARCTAN: [i64; 22] = [
    1740967, 919879, 466945, 234379, 117304, 58666, 29335, 14668, 7334, 3667, 1833, 917, 458, 229,
    115, 57, 29, 14, 7, 4, 2, 1,
];

/// `FT_MSB` - Stelle des hoechsten gesetzten Bits.
fn msb(z: u32) -> i32 {
    if z == 0 {
        0
    } else {
        31 - z.leading_zeros() as i32
    }
}

/// `ft_trig_downscale`, die Fassung mit `FT_INT64`.
fn downscale(val: i64) -> i64 {
    let s = val < 0;
    let v = val.unsigned_abs();
    let v = ((v * TRIG_SCALE + 0x4000_0000) >> 32) as i64;
    if s {
        -v
    } else {
        v
    }
}

/// `ft_trig_prenorm`.
fn prenorm(v: &mut (i64, i64)) -> i32 {
    let (x, y) = *v;
    let shift = msb((x.unsigned_abs() | y.unsigned_abs()) as u32);
    if shift <= TRIG_SAFE_MSB {
        let s = TRIG_SAFE_MSB - shift;
        v.0 = ((x as u32) << s) as i32 as i64;
        v.1 = ((y as u32) << s) as i32 as i64;
        s
    } else {
        let s = shift - TRIG_SAFE_MSB;
        v.0 = x >> s;
        v.1 = y >> s;
        -s
    }
}

/// `ft_trig_pseudo_rotate`.
fn pseudo_rotate(v: &mut (i64, i64), mut theta: i64) {
    let (mut x, mut y) = *v;
    while theta < -PI_4 {
        let t = y;
        y = -x;
        x = t;
        theta += PI_2;
    }
    while theta > PI_4 {
        let t = -y;
        y = x;
        x = t;
        theta -= PI_2;
    }
    let mut b: i64 = 1;
    for i in 1..TRIG_MAX_ITERS {
        let a = ARCTAN[(i - 1) as usize];
        if theta < 0 {
            let t = x + ((y + b) >> i);
            y -= (x + b) >> i;
            x = t;
            theta += a;
        } else {
            let t = x - ((y + b) >> i);
            y += (x + b) >> i;
            x = t;
            theta -= a;
        }
        b <<= 1;
    }
    *v = (x, y);
}

/// `ft_trig_pseudo_polarize`: gibt `(Laenge unskaliert, Winkel)`.
fn pseudo_polarize(v: &mut (i64, i64)) {
    let (mut x, mut y) = *v;
    let mut theta;
    if y > x {
        if y > -x {
            theta = PI_2;
            let t = y;
            y = -x;
            x = t;
        } else {
            theta = if y > 0 { PI } else { -PI };
            x = -x;
            y = -y;
        }
    } else if y < -x {
        theta = -PI_2;
        let t = -y;
        y = x;
        x = t;
    } else {
        theta = 0;
    }
    let mut b: i64 = 1;
    for i in 1..TRIG_MAX_ITERS {
        let a = ARCTAN[(i - 1) as usize];
        if y > 0 {
            let t = x + ((y + b) >> i);
            y -= (x + b) >> i;
            x = t;
            theta += a;
        } else {
            let t = x - ((y + b) >> i);
            y += (x + b) >> i;
            x = t;
            theta -= a;
        }
        b <<= 1;
    }
    // `FT_PAD_ROUND( theta, 16 )`
    theta = if theta >= 0 {
        (theta + 8) & !15
    } else {
        -((-theta + 8) & !15)
    };
    *v = (x, theta);
}

/// `FT_Tan`.
fn tan(angle: i64) -> i64 {
    let mut v = (1i64 << 24, 0i64);
    pseudo_rotate(&mut v, angle);
    div_fix(v.1, v.0)
}

/// `FT_Atan2`.
fn atan2(dx: i64, dy: i64) -> i64 {
    if dx == 0 && dy == 0 {
        return 0;
    }
    let mut v = (dx, dy);
    prenorm(&mut v);
    pseudo_polarize(&mut v);
    v.1
}

/// `FT_Vector_Unit`.
fn unit(angle: i64) -> (i64, i64) {
    let mut v = ((TRIG_SCALE >> 8) as i64, 0i64);
    pseudo_rotate(&mut v, angle);
    ((v.0 + 0x80) >> 8, (v.1 + 0x80) >> 8)
}

/// `FT_Cos`.
fn cos(angle: i64) -> i64 {
    unit(angle).0
}

/// `FT_Sin`.
fn sin(angle: i64) -> i64 {
    unit(angle).1
}

/// `FT_Vector_Rotate`.
fn rotate(vec: (i64, i64), angle: i64) -> (i64, i64) {
    if angle == 0 {
        return vec;
    }
    let mut v = vec;
    if v.0 == 0 && v.1 == 0 {
        return vec;
    }
    let shift = prenorm(&mut v);
    pseudo_rotate(&mut v, angle);
    v.0 = downscale(v.0);
    v.1 = downscale(v.1);
    if shift > 0 {
        let half = 1i64 << (shift - 1);
        (
            (v.0 + half - i64::from(v.0 < 0)) >> shift,
            (v.1 + half - i64::from(v.1 < 0)) >> shift,
        )
    } else {
        let s = -shift;
        (((v.0 as u32) << s) as i32 as i64, ((v.1 as u32) << s) as i32 as i64)
    }
}

/// `FT_Vector_Length`.
fn length(vec: (i64, i64)) -> i64 {
    let mut v = vec;
    if v.0 == 0 {
        return v.1.abs();
    } else if v.1 == 0 {
        return v.0.abs();
    }
    let shift = prenorm(&mut v);
    pseudo_polarize(&mut v);
    v.0 = downscale(v.0);
    if shift > 0 {
        return (v.0 + (1i64 << (shift - 1))) >> shift;
    }
    ((v.0 as u32) << -shift) as i64
}

/// `FT_Vector_Unit` fuer andere Module (`pygame.freetype`s Drehung).
pub(crate) fn einheit(angle: i64) -> (i64, i64) {
    unit(angle)
}

/// `FT_Vector_Rotate` fuer andere Module.
pub(crate) fn drehen(v: (i64, i64), angle: i64) -> (i64, i64) {
    rotate(v, angle)
}

/// `FT_Vector_Length` fuer andere Module (`pygame.freetype`s
/// Schreibmarke braucht es beim Unterschneiden).
pub(crate) fn laenge(v: (i64, i64)) -> i64 {
    length(v)
}

/// `FT_Vector_From_Polar`.
fn from_polar(length: i64, angle: i64) -> (i64, i64) {
    rotate((length, 0), angle)
}

/// `FT_Angle_Diff`.
fn angle_diff(a1: i64, a2: i64) -> i64 {
    let mut d = a2 - a1;
    while d <= -PI {
        d += PI2X;
    }
    while d > PI {
        d -= PI2X;
    }
    d
}

// ----- Konturzieher (`ftstroke.c`) -------------------------------------------

/// `FT_SMALL_CONIC_THRESHOLD`.
const SMALL_CONIC: i64 = PI / 6;
/// `FT_SMALL_CUBIC_THRESHOLD`.
const SMALL_CUBIC: i64 = PI / 8;
/// `FT_ARC_CUBIC_ANGLE`.
const ARC_CUBIC_ANGLE: i64 = PI / 2;

/// `FT_IS_SMALL`.
#[inline]
fn klein(x: i64) -> bool {
    x > -2 && x < 2
}

/// `ft_angle_mean`.
fn angle_mean(a1: i64, a2: i64) -> i64 {
    a1 + angle_diff(a1, a2) / 2
}

const TAG_ON: u8 = 1;
const TAG_CUBIC: u8 = 2;
const TAG_BEGIN: u8 = 4;
const TAG_END: u8 = 8;
const TAG_BEGIN_END: u8 = TAG_BEGIN | TAG_END;

type V = (i64, i64);

fn conic_split(b: &mut [V], i: usize) {
    b[i + 4].0 = b[i + 2].0;
    let a = b[i].0 + b[i + 1].0;
    let c = b[i + 1].0 + b[i + 2].0;
    b[i + 3].0 = c >> 1;
    b[i + 2].0 = (a + c) >> 2;
    b[i + 1].0 = a >> 1;
    b[i + 4].1 = b[i + 2].1;
    let a = b[i].1 + b[i + 1].1;
    let c = b[i + 1].1 + b[i + 2].1;
    b[i + 3].1 = c >> 1;
    b[i + 2].1 = (a + c) >> 2;
    b[i + 1].1 = a >> 1;
}

fn cubic_split(b: &mut [V], i: usize) {
    b[i + 6].0 = b[i + 3].0;
    let mut a = b[i].0 + b[i + 1].0;
    let bb = b[i + 1].0 + b[i + 2].0;
    let mut c = b[i + 2].0 + b[i + 3].0;
    b[i + 5].0 = c >> 1;
    c += bb;
    b[i + 4].0 = c >> 2;
    b[i + 1].0 = a >> 1;
    a += bb;
    b[i + 2].0 = a >> 2;
    b[i + 3].0 = (a + c) >> 3;
    b[i + 6].1 = b[i + 3].1;
    let mut a = b[i].1 + b[i + 1].1;
    let bb = b[i + 1].1 + b[i + 2].1;
    let mut c = b[i + 2].1 + b[i + 3].1;
    b[i + 5].1 = c >> 1;
    c += bb;
    b[i + 4].1 = c >> 2;
    b[i + 1].1 = a >> 1;
    a += bb;
    b[i + 2].1 = a >> 2;
    b[i + 3].1 = (a + c) >> 3;
}

/// `ft_conic_is_small_enough`.
fn conic_klein_genug(b: &[V], i: usize, ein: &mut i64, aus: &mut i64) -> bool {
    let d1 = (b[i + 1].0 - b[i + 2].0, b[i + 1].1 - b[i + 2].1);
    let d2 = (b[i].0 - b[i + 1].0, b[i].1 - b[i + 1].1);
    let c1 = klein(d1.0) && klein(d1.1);
    let c2 = klein(d2.0) && klein(d2.1);
    if c1 {
        if !c2 {
            let a = atan2(d2.0, d2.1);
            *ein = a;
            *aus = a;
        }
    } else if c2 {
        let a = atan2(d1.0, d1.1);
        *ein = a;
        *aus = a;
    } else {
        *ein = atan2(d1.0, d1.1);
        *aus = atan2(d2.0, d2.1);
    }
    angle_diff(*ein, *aus).abs() < SMALL_CONIC
}

/// `ft_cubic_is_small_enough`.
fn cubic_klein_genug(b: &[V], i: usize, ein: &mut i64, mitte: &mut i64, aus: &mut i64) -> bool {
    let d1 = (b[i + 2].0 - b[i + 3].0, b[i + 2].1 - b[i + 3].1);
    let d2 = (b[i + 1].0 - b[i + 2].0, b[i + 1].1 - b[i + 2].1);
    let d3 = (b[i].0 - b[i + 1].0, b[i].1 - b[i + 1].1);
    let c1 = klein(d1.0) && klein(d1.1);
    let c2 = klein(d2.0) && klein(d2.1);
    let c3 = klein(d3.0) && klein(d3.1);
    if c1 {
        if c2 {
            if !c3 {
                let a = atan2(d3.0, d3.1);
                (*ein, *mitte, *aus) = (a, a, a);
            }
        } else if c3 {
            let a = atan2(d2.0, d2.1);
            (*ein, *mitte, *aus) = (a, a, a);
        } else {
            let a = atan2(d2.0, d2.1);
            *ein = a;
            *mitte = a;
            *aus = atan2(d3.0, d3.1);
        }
    } else if c2 {
        if c3 {
            let a = atan2(d1.0, d1.1);
            (*ein, *mitte, *aus) = (a, a, a);
        } else {
            *ein = atan2(d1.0, d1.1);
            *aus = atan2(d3.0, d3.1);
            *mitte = angle_mean(*ein, *aus);
        }
    } else if c3 {
        *ein = atan2(d1.0, d1.1);
        let a = atan2(d2.0, d2.1);
        *mitte = a;
        *aus = a;
    } else {
        *ein = atan2(d1.0, d1.1);
        *mitte = atan2(d2.0, d2.1);
        *aus = atan2(d3.0, d3.1);
    }
    let t1 = angle_diff(*ein, *mitte).abs();
    let t2 = angle_diff(*mitte, *aus).abs();
    t1 < SMALL_CUBIC && t2 < SMALL_CUBIC
}

/// `FT_StrokeBorderRec`.
#[derive(Debug, Clone, Default)]
struct Rand {
    punkte: Vec<V>,
    tags: Vec<u8>,
    movable: bool,
    start: i64,
    valid: bool,
}

impl Rand {
    fn neu() -> Rand {
        Rand { start: -1, ..Rand::default() }
    }

    /// `ft_stroke_border_close`.
    fn close(&mut self, reverse: bool) {
        let start = self.start as usize;
        let mut count = self.punkte.len();
        if count <= start + 1 {
            self.punkte.truncate(start);
            self.tags.truncate(start);
        } else {
            count -= 1;
            self.punkte[start] = self.punkte[count];
            self.tags[start] = self.tags[count];
            self.punkte.truncate(count);
            self.tags.truncate(count);
            if reverse && count > start + 1 {
                self.punkte[start + 1..count].reverse();
                self.tags[start + 1..count].reverse();
            }
            self.tags[start] |= TAG_BEGIN;
            self.tags[count - 1] |= TAG_END;
        }
        self.start = -1;
        self.movable = false;
    }

    /// `ft_stroke_border_lineto`.
    fn lineto(&mut self, to: V, movable: bool) {
        if self.movable {
            let n = self.punkte.len();
            self.punkte[n - 1] = to;
        } else {
            let n = self.punkte.len();
            if n as i64 > self.start
                && klein(self.punkte[n - 1].0 - to.0)
                && klein(self.punkte[n - 1].1 - to.1)
            {
                return;
            }
            self.punkte.push(to);
            self.tags.push(TAG_ON);
        }
        self.movable = movable;
    }

    /// `ft_stroke_border_conicto`.
    fn conicto(&mut self, control: V, to: V) {
        self.punkte.push(control);
        self.punkte.push(to);
        self.tags.push(0);
        self.tags.push(TAG_ON);
        self.movable = false;
    }

    /// `ft_stroke_border_cubicto`.
    fn cubicto(&mut self, c1: V, c2: V, to: V) {
        self.punkte.extend([c1, c2, to]);
        self.tags.extend([TAG_CUBIC, TAG_CUBIC, TAG_ON]);
        self.movable = false;
    }

    /// `ft_stroke_border_arcto`.
    fn arcto(&mut self, center: V, radius: i64, angle_start: i64, angle_diff: i64) {
        let mut arcs: i64 = 1;
        while angle_diff > ARC_CUBIC_ANGLE * arcs || -angle_diff > ARC_CUBIC_ANGLE * arcs {
            arcs += 1;
        }
        let mut coef = tan(angle_diff / (4 * arcs));
        coef += coef / 3;
        let mut a0 = from_polar(radius, angle_start);
        let mut a1 = (mul_fix(-a0.1, coef), mul_fix(a0.0, coef));
        a0.0 += center.0;
        a0.1 += center.1;
        a1.0 += a0.0;
        a1.1 += a0.1;
        for i in 1..=arcs {
            let mut a3 = from_polar(radius, angle_start + i * angle_diff / arcs);
            let mut a2 = (mul_fix(a3.1, coef), mul_fix(-a3.0, coef));
            a3.0 += center.0;
            a3.1 += center.1;
            a2.0 += a3.0;
            a2.1 += a3.1;
            self.cubicto(a1, a2, a3);
            a1 = (a3.0 - a2.0 + a3.0, a3.1 - a2.1 + a3.1);
        }
    }

    /// `ft_stroke_border_moveto`.
    fn moveto(&mut self, to: V) {
        if self.start >= 0 {
            self.close(false);
        }
        self.start = self.punkte.len() as i64;
        self.movable = false;
        self.lineto(to, false);
    }

    /// `ft_stroke_border_get_counts` - setzt `valid`.
    fn zaehlen(&mut self) -> (usize, usize) {
        let mut konturen = 0;
        let mut drin = false;
        for &t in &self.tags {
            if t & TAG_BEGIN != 0 {
                if drin {
                    return (0, 0);
                }
                drin = true;
            } else if !drin {
                return (0, 0);
            }
            if t & TAG_END != 0 {
                drin = false;
                konturen += 1;
            }
        }
        if drin {
            return (0, 0);
        }
        self.valid = true;
        (self.punkte.len(), konturen)
    }

    /// `ft_stroke_border_export`.
    fn export(&self, u: &mut Umriss) {
        let basis = u.punkte.len();
        u.punkte.extend_from_slice(&self.punkte);
        for &t in &self.tags {
            u.arten.push(if t & TAG_ON != 0 {
                Art::Auf
            } else if t & TAG_CUBIC != 0 {
                Art::Kubisch
            } else {
                Art::Quadratisch
            });
        }
        for (i, &t) in self.tags.iter().enumerate() {
            if t & TAG_END != 0 {
                u.enden.push(basis + i);
            }
        }
    }
}

/// `FT_StrokerRec` mit `FT_STROKER_LINECAP_ROUND` und
/// `FT_STROKER_LINEJOIN_ROUND` - das Einzige, was SDL_ttf benutzt. Die
/// anderen Enden und Ecken sind trotzdem uebertragen, damit der Weg durch
/// `ft_stroker_outside` derselbe bleibt.
struct Zieher {
    angle_in: i64,
    angle_out: i64,
    center: V,
    line_length: i64,
    first_point: bool,
    subpath_open: bool,
    subpath_angle: i64,
    subpath_start: V,
    subpath_line_length: i64,
    handle_wide_strokes: bool,
    line_cap: Kappe,
    line_join: Ecke,
    line_join_saved: Ecke,
    miter_limit: i64,
    radius: i64,
    raender: [Rand; 2],
}

/// `FT_Stroker_LineCap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum Kappe {
    Butt,
    Round,
    Square,
}

/// `FT_Stroker_LineJoin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum Ecke {
    Round,
    Bevel,
    MiterVariable,
    MiterFixed,
}

/// `FT_SIDE_TO_ROTATE`.
#[inline]
fn side_to_rotate(s: usize) -> i64 {
    PI_2 - s as i64 * PI
}

impl Zieher {
    /// `FT_Stroker_New` + `FT_Stroker_Set`.
    fn neu(radius: i64, line_cap: Kappe, line_join: Ecke, miter_limit: i64) -> Zieher {
        Zieher {
            angle_in: 0,
            angle_out: 0,
            center: (0, 0),
            line_length: 0,
            first_point: false,
            subpath_open: false,
            subpath_angle: 0,
            subpath_start: (0, 0),
            subpath_line_length: 0,
            handle_wide_strokes: false,
            line_cap,
            line_join,
            line_join_saved: line_join,
            miter_limit: miter_limit.max(0x10000),
            radius,
            raender: [Rand::neu(), Rand::neu()],
        }
    }

    /// `ft_stroker_arcto`.
    fn arcto(&mut self, side: usize) {
        let rotate = side_to_rotate(side);
        let mut total = angle_diff(self.angle_in, self.angle_out);
        if total == PI {
            total = -rotate * 2;
        }
        let (center, radius, start) = (self.center, self.radius, self.angle_in + rotate);
        let r = &mut self.raender[side];
        r.arcto(center, radius, start, total);
        r.movable = false;
    }

    /// `ft_stroker_cap`.
    fn cap(&mut self, angle: i64, side: usize) {
        if self.line_cap == Kappe::Round {
            self.angle_in = angle;
            self.angle_out = angle + PI;
            self.arcto(side);
        } else {
            let mut middle = from_polar(self.radius, angle);
            let mut delta = if side != 0 { (middle.1, -middle.0) } else { (-middle.1, middle.0) };
            if self.line_cap == Kappe::Square {
                middle.0 += self.center.0;
                middle.1 += self.center.1;
            } else {
                middle = self.center;
            }
            delta.0 += middle.0;
            delta.1 += middle.1;
            self.raender[side].lineto(delta, false);
            delta = (middle.0 - delta.0 + middle.0, middle.1 - delta.1 + middle.1);
            self.raender[side].lineto(delta, false);
        }
    }

    /// `ft_stroker_inside`.
    fn inside(&mut self, side: usize, line_length: i64) {
        let rotate = side_to_rotate(side);
        let theta = angle_diff(self.angle_in, self.angle_out) / 2;
        let mut sigma = (0, 0);
        let intersect = if !self.raender[side].movable
            || line_length == 0
            || !(-0x59C000..=0x59C000).contains(&theta)
        {
            false
        } else {
            sigma = unit(theta);
            let min_length = mul_div(self.radius, sigma.1, sigma.0).abs();
            min_length != 0 && self.line_length >= min_length && line_length >= min_length
        };
        let delta;
        if !intersect {
            let d = from_polar(self.radius, self.angle_out + rotate);
            delta = (d.0 + self.center.0, d.1 + self.center.1);
            self.raender[side].movable = false;
        } else {
            let phi = self.angle_in + theta + rotate;
            let length = div_fix(self.radius, sigma.0);
            let d = from_polar(length, phi);
            delta = (d.0 + self.center.0, d.1 + self.center.1);
        }
        self.raender[side].lineto(delta, false);
    }

    /// `ft_stroker_outside`.
    fn outside(&mut self, side: usize, line_length: i64) {
        if self.line_join == Ecke::Round {
            self.arcto(side);
            return;
        }
        let radius = self.radius;
        let mut sigma = (0, 0);
        let mut phi = 0;
        let rotate = side_to_rotate(side);
        let mut bevel = self.line_join == Ecke::Bevel;
        let fixed_bevel = self.line_join != Ecke::MiterVariable;
        if !bevel {
            let mut theta = angle_diff(self.angle_in, self.angle_out) / 2;
            if theta == PI_2 {
                theta = -rotate;
            }
            phi = self.angle_in + theta + rotate;
            sigma = from_polar(self.miter_limit, theta);
            if sigma.0 < 0x10000 && (fixed_bevel || theta.abs() > 57) {
                bevel = true;
            }
        }
        if bevel {
            if fixed_bevel {
                let d = from_polar(radius, self.angle_out + rotate);
                let delta = (d.0 + self.center.0, d.1 + self.center.1);
                self.raender[side].movable = false;
                self.raender[side].lineto(delta, false);
            } else {
                let mut middle = from_polar(mul_fix(radius, self.miter_limit), phi);
                let coef = div_fix(0x10000 - sigma.0, sigma.1);
                let mut delta = (mul_fix(middle.1, coef), mul_fix(-middle.0, coef));
                middle.0 += self.center.0;
                middle.1 += self.center.1;
                delta.0 += middle.0;
                delta.1 += middle.1;
                self.raender[side].lineto(delta, false);
                delta = (middle.0 - delta.0 + middle.0, middle.1 - delta.1 + middle.1);
                self.raender[side].lineto(delta, false);
                if line_length == 0 {
                    let d = from_polar(radius, self.angle_out + rotate);
                    self.raender[side].lineto((d.0 + self.center.0, d.1 + self.center.1), false);
                }
            }
        } else {
            let length = mul_div(self.radius, self.miter_limit, sigma.0);
            let d = from_polar(length, phi);
            self.raender[side].lineto((d.0 + self.center.0, d.1 + self.center.1), false);
            if line_length == 0 {
                let d = from_polar(self.radius, self.angle_out + rotate);
                self.raender[side].lineto((d.0 + self.center.0, d.1 + self.center.1), false);
            }
        }
    }

    /// `ft_stroker_process_corner`.
    fn process_corner(&mut self, line_length: i64) {
        let turn = angle_diff(self.angle_in, self.angle_out);
        if turn == 0 {
            return;
        }
        let inside_side = usize::from(turn < 0);
        self.inside(inside_side, line_length);
        self.outside(1 - inside_side, line_length);
    }

    /// `ft_stroker_subpath_start`.
    fn subpath_start(&mut self, start_angle: i64, line_length: i64) {
        let delta = from_polar(self.radius, start_angle + PI_2);
        let p = (self.center.0 + delta.0, self.center.1 + delta.1);
        self.raender[0].moveto(p);
        let p = (self.center.0 - delta.0, self.center.1 - delta.1);
        self.raender[1].moveto(p);
        self.subpath_angle = start_angle;
        self.first_point = false;
        self.subpath_line_length = line_length;
    }

    /// `FT_Stroker_LineTo`.
    fn line_to(&mut self, to: V) {
        let mut delta = (to.0 - self.center.0, to.1 - self.center.1);
        if delta.0 == 0 && delta.1 == 0 {
            return;
        }
        let line_length = length(delta);
        let angle = atan2(delta.0, delta.1);
        delta = from_polar(self.radius, angle + PI_2);
        if self.first_point {
            self.subpath_start(angle, line_length);
        } else {
            self.angle_out = angle;
            self.process_corner(line_length);
        }
        for side in [1usize, 0] {
            // C laeuft `border = borders, side = 1` und dann `border++,
            // side--`: Rand 0 bekommt den positiven Versatz, Rand 1 den
            // negativen.
            let rand = 1 - side;
            let p = (to.0 + delta.0, to.1 + delta.1);
            self.raender[rand].lineto(p, true);
            delta = (-delta.0, -delta.1);
        }
        self.angle_in = angle;
        self.center = to;
        self.line_length = line_length;
    }

    /// `FT_Stroker_ConicTo`.
    fn conic_to(&mut self, control: V, to: V) {
        if klein(self.center.0 - control.0)
            && klein(self.center.1 - control.1)
            && klein(control.0 - to.0)
            && klein(control.1 - to.1)
        {
            self.center = to;
            return;
        }
        let mut b = [(0i64, 0i64); 34];
        let limit = 30isize;
        let mut arc: isize = 0;
        let mut first_arc = true;
        b[0] = to;
        b[1] = control;
        b[2] = self.center;
        while arc >= 0 {
            let a = arc as usize;
            let mut angle_in = self.angle_in;
            let mut angle_out = self.angle_in;
            if arc < limit && !conic_klein_genug(&b, a, &mut angle_in, &mut angle_out) {
                if self.first_point {
                    self.angle_in = angle_in;
                }
                conic_split(&mut b, a);
                arc += 2;
                continue;
            }
            if first_arc {
                first_arc = false;
                if self.first_point {
                    self.subpath_start(angle_in, 0);
                } else {
                    self.angle_out = angle_in;
                    self.process_corner(0);
                }
            } else if angle_diff(self.angle_in, angle_in).abs() > SMALL_CONIC / 4 {
                self.center = b[a + 2];
                self.angle_out = angle_in;
                self.line_join = Ecke::Round;
                self.process_corner(0);
                self.line_join = self.line_join_saved;
            }
            {
                let theta = angle_diff(angle_in, angle_out) / 2;
                let phi = angle_in + theta;
                let length = div_fix(self.radius, cos(theta));
                let alpha0 = if self.handle_wide_strokes {
                    atan2(b[a].0 - b[a + 2].0, b[a].1 - b[a + 2].1)
                } else {
                    0
                };
                for side in 0..2usize {
                    let rotate = side_to_rotate(side);
                    let c = from_polar(length, phi + rotate);
                    let ctrl = (c.0 + b[a + 1].0, c.1 + b[a + 1].1);
                    let e = from_polar(self.radius, angle_out + rotate);
                    let end = (e.0 + b[a].0, e.1 + b[a].1);
                    if self.handle_wide_strokes {
                        let rand = &self.raender[side];
                        let start = rand.punkte[rand.punkte.len() - 1];
                        let alpha1 = atan2(end.0 - start.0, end.1 - start.1);
                        if angle_diff(alpha0, alpha1).abs() > PI / 2 {
                            let beta = atan2(b[a + 2].0 - start.0, b[a + 2].1 - start.1);
                            let gamma = atan2(b[a].0 - end.0, b[a].1 - end.1);
                            let blen = length_of(end, start);
                            let sin_a = sin(alpha1 - gamma).abs();
                            let sin_b = sin(beta - gamma).abs();
                            let alen = mul_div(blen, sin_a, sin_b);
                            let d = from_polar(alen, beta);
                            let delta = (d.0 + start.0, d.1 + start.1);
                            let rand = &mut self.raender[side];
                            rand.movable = false;
                            rand.lineto(delta, false);
                            rand.lineto(end, false);
                            rand.conicto(ctrl, start);
                            rand.lineto(end, false);
                            continue;
                        }
                    }
                    self.raender[side].conicto(ctrl, end);
                }
            }
            arc -= 2;
            self.angle_in = angle_out;
        }
        self.center = to;
        self.line_length = 0;
    }

    /// `FT_Stroker_CubicTo`.
    fn cubic_to(&mut self, c1: V, c2: V, to: V) {
        if klein(self.center.0 - c1.0)
            && klein(self.center.1 - c1.1)
            && klein(c1.0 - c2.0)
            && klein(c1.1 - c2.1)
            && klein(c2.0 - to.0)
            && klein(c2.1 - to.1)
        {
            self.center = to;
            return;
        }
        let mut b = [(0i64, 0i64); 37];
        let limit = 32isize;
        let mut arc: isize = 0;
        let mut first_arc = true;
        b[0] = to;
        b[1] = c2;
        b[2] = c1;
        b[3] = self.center;
        while arc >= 0 {
            let a = arc as usize;
            let (mut ai, mut am, mut ao) = (self.angle_in, self.angle_in, self.angle_in);
            if arc < limit && !cubic_klein_genug(&b, a, &mut ai, &mut am, &mut ao) {
                if self.first_point {
                    self.angle_in = ai;
                }
                cubic_split(&mut b, a);
                arc += 3;
                continue;
            }
            if first_arc {
                first_arc = false;
                if self.first_point {
                    self.subpath_start(ai, 0);
                } else {
                    self.angle_out = ai;
                    self.process_corner(0);
                }
            } else if angle_diff(self.angle_in, ai).abs() > SMALL_CUBIC / 4 {
                self.center = b[a + 3];
                self.angle_out = ai;
                self.line_join = Ecke::Round;
                self.process_corner(0);
                self.line_join = self.line_join_saved;
            }
            {
                let theta1 = angle_diff(ai, am) / 2;
                let theta2 = angle_diff(am, ao) / 2;
                let phi1 = angle_mean(ai, am);
                let phi2 = angle_mean(am, ao);
                let length1 = div_fix(self.radius, cos(theta1));
                let length2 = div_fix(self.radius, cos(theta2));
                let alpha0 = if self.handle_wide_strokes {
                    atan2(b[a].0 - b[a + 3].0, b[a].1 - b[a + 3].1)
                } else {
                    0
                };
                for side in 0..2usize {
                    let rotate = side_to_rotate(side);
                    let p = from_polar(length1, phi1 + rotate);
                    let ctrl1 = (p.0 + b[a + 2].0, p.1 + b[a + 2].1);
                    let p = from_polar(length2, phi2 + rotate);
                    let ctrl2 = (p.0 + b[a + 1].0, p.1 + b[a + 1].1);
                    let p = from_polar(self.radius, ao + rotate);
                    let end = (p.0 + b[a].0, p.1 + b[a].1);
                    if self.handle_wide_strokes {
                        let rand = &self.raender[side];
                        let start = rand.punkte[rand.punkte.len() - 1];
                        let alpha1 = atan2(end.0 - start.0, end.1 - start.1);
                        if angle_diff(alpha0, alpha1).abs() > PI / 2 {
                            let beta = atan2(b[a + 3].0 - start.0, b[a + 3].1 - start.1);
                            let gamma = atan2(b[a].0 - end.0, b[a].1 - end.1);
                            let blen = length_of(end, start);
                            let sin_a = sin(alpha1 - gamma).abs();
                            let sin_b = sin(beta - gamma).abs();
                            let alen = mul_div(blen, sin_a, sin_b);
                            let d = from_polar(alen, beta);
                            let delta = (d.0 + start.0, d.1 + start.1);
                            let rand = &mut self.raender[side];
                            rand.movable = false;
                            rand.lineto(delta, false);
                            rand.lineto(end, false);
                            rand.cubicto(ctrl2, ctrl1, start);
                            rand.lineto(end, false);
                            continue;
                        }
                    }
                    self.raender[side].cubicto(ctrl1, ctrl2, end);
                }
            }
            arc -= 3;
            self.angle_in = ao;
        }
        self.center = to;
        self.line_length = 0;
    }

    /// `FT_Stroker_BeginSubPath`.
    fn begin_subpath(&mut self, to: V, open: bool) {
        self.first_point = true;
        self.center = to;
        self.subpath_open = open;
        self.handle_wide_strokes = self.line_join != Ecke::Round
            || (self.subpath_open && self.line_cap == Kappe::Butt);
        self.subpath_start = to;
        self.angle_in = 0;
    }

    /// `ft_stroker_add_reverse_left`.
    fn add_reverse_left(&mut self, open: bool) {
        let [right, left] = &mut self.raender;
        let start = left.start as usize;
        let neu = left.punkte.len() as i64 - left.start;
        if neu > 0 {
            for i in (start..left.punkte.len()).rev() {
                let mut t = left.tags[i];
                if open {
                    t &= !TAG_BEGIN_END;
                } else {
                    let tt = t & TAG_BEGIN_END;
                    if tt == TAG_BEGIN || tt == TAG_END {
                        t ^= TAG_BEGIN_END;
                    }
                }
                right.punkte.push(left.punkte[i]);
                right.tags.push(t);
            }
            left.punkte.truncate(start);
            left.tags.truncate(start);
            right.movable = false;
            left.movable = false;
        }
    }

    /// `FT_Stroker_EndSubPath`.
    fn end_subpath(&mut self) {
        if self.subpath_open {
            self.cap(self.angle_in, 0);
            self.add_reverse_left(true);
            self.center = self.subpath_start;
            self.cap(self.subpath_angle + PI, 0);
            self.raender[0].close(false);
        } else {
            if !klein(self.center.0 - self.subpath_start.0) || !klein(self.center.1 - self.subpath_start.1) {
                let s = self.subpath_start;
                self.line_to(s);
            }
            self.angle_out = self.subpath_angle;
            self.process_corner(self.subpath_line_length);
            self.raender[0].close(false);
            self.raender[1].close(true);
        }
    }

    /// `FT_Stroker_ParseOutline` mit `opened = FALSE`. `None` bei einem
    /// ungueltigen Umriss.
    fn parse_outline(&mut self, u: &Umriss) -> Option<()> {
        for r in &mut self.raender {
            r.punkte.clear();
            r.tags.clear();
            r.start = -1;
            r.valid = false;
        }
        let mut first = 0usize;
        for &last in &u.enden {
            if last <= first {
                first = last + 1;
                continue;
            }
            let mut v_start = u.punkte[first];
            let v_last = u.punkte[last];
            let mut point = first as isize;
            let mut limit = last as isize;
            let tag = u.arten[first];
            if tag == Art::Kubisch {
                return None;
            }
            if tag == Art::Quadratisch {
                if u.arten[last] == Art::Auf {
                    v_start = v_last;
                    limit -= 1;
                } else {
                    v_start = ((v_start.0 + v_last.0) / 2, (v_start.1 + v_last.1) / 2);
                }
                point -= 1;
            }
            self.begin_subpath(v_start, false);
            'kontur: while point < limit {
                point += 1;
                match u.arten[point as usize] {
                    Art::Auf => {
                        self.line_to(u.punkte[point as usize]);
                    }
                    Art::Quadratisch => {
                        let mut v_control = u.punkte[point as usize];
                        loop {
                            if point < limit {
                                point += 1;
                                let vec = u.punkte[point as usize];
                                match u.arten[point as usize] {
                                    Art::Auf => {
                                        self.conic_to(v_control, vec);
                                        continue 'kontur;
                                    }
                                    Art::Quadratisch => {}
                                    Art::Kubisch => return None,
                                }
                                let mitte = ((v_control.0 + vec.0) / 2, (v_control.1 + vec.1) / 2);
                                self.conic_to(v_control, mitte);
                                v_control = vec;
                                continue;
                            }
                            self.conic_to(v_control, v_start);
                            break 'kontur;
                        }
                    }
                    Art::Kubisch => {
                        if point + 1 > limit || u.arten[(point + 1) as usize] != Art::Kubisch {
                            return None;
                        }
                        point += 2;
                        let v1 = u.punkte[(point - 2) as usize];
                        let v2 = u.punkte[(point - 1) as usize];
                        if point <= limit {
                            self.cubic_to(v1, v2, u.punkte[point as usize]);
                            continue;
                        }
                        self.cubic_to(v1, v2, v_start);
                        break 'kontur;
                    }
                }
            }
            if !self.first_point {
                self.end_subpath();
            }
            first = last + 1;
        }
        Some(())
    }
}

/// `FT_Vector_Length` der Strecke `a - b`.
fn length_of(a: V, b: V) -> i64 {
    length((a.0 - b.0, a.1 - b.1))
}

/// `FT_Glyph_Stroke` mit dem Zieher, den SDL_ttf einrichtet:
/// `FT_Stroker_Set(stroker, outline * 64, ROUND, ROUND, 0)`.
///
/// Der neue Umriss traegt **keine** Flaggen des alten mehr
/// (`FT_Outline_New`): kein `FT_OUTLINE_HIGH_PRECISION`, keine
/// Dropout-Flaggen, keine Scan-Marken, kein `FT_OUTLINE_OVERLAP`.
pub(crate) fn konturieren(u: &Umriss, radius: i64) -> Option<Umriss> {
    let mut z = Zieher::neu(radius, Kappe::Round, Ecke::Round, 0);
    z.parse_outline(u)?;
    let mut aus = Umriss::default();
    // `FT_Stroker_GetCounts` setzt `valid`; `FT_Stroker_Export` gibt erst
    // `FT_STROKER_BORDER_LEFT` (0), dann `FT_STROKER_BORDER_RIGHT` (1) aus.
    let _ = z.raender[0].zaehlen();
    let _ = z.raender[1].zaehlen();
    for r in &z.raender {
        if r.valid {
            r.export(&mut aus);
        }
    }
    Some(aus)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winkel_wie_freetype() {
        // FT_Atan2(1, 1) = 45 Grad, FT_Atan2(0, -1) = -90 Grad (16.16).
        assert_eq!(atan2(1 << 16, 1 << 16), 45 << 16);
        assert_eq!(atan2(0, -100), -(90 << 16));
        assert_eq!(length((3 << 10, 4 << 10)), 5 << 10);
        assert_eq!(from_polar(64, 0), (64, 0));
    }

    #[test]
    fn quadrat_bekommt_rand_innen_und_aussen() {
        let q = Umriss {
            punkte: vec![(0, 0), (0, 640), (640, 640), (640, 0)],
            arten: vec![Art::Auf; 4],
            enden: vec![3],
            ueberlappt: false,
        };
        let k = konturieren(&q, 64).unwrap();
        assert_eq!(k.enden.len(), 2);
        let (x0, y0, x1, y1) = k.kontrollkasten();
        assert!(x0 <= -64 && y0 <= -64 && x1 >= 704 && y1 >= 704);
    }

    fn umriss_aus(punkte: &str, enden: &str) -> Umriss {
        let mut u = Umriss::default();
        if punkte != "-" {
            for p in punkte.split(' ') {
                let w: Vec<i64> = p.split(',').map(|v| v.parse().unwrap()).collect();
                u.punkte.push((w[0], w[1]));
                u.arten.push(match w[2] & 3 {
                    1 => Art::Auf,
                    0 => Art::Quadratisch,
                    _ => Art::Kubisch,
                });
            }
        }
        if enden != "-" {
            u.enden = enden.split(' ').map(|v| v.parse().unwrap()).collect();
        }
        u
    }

    /// Gegen FreeTypes eigenes `FT_Glyph_Stroke`: `RUSTGAME_KONTUR_ORAKEL`
    /// zeigt auf die Datei aus `ft_orakel_kontur.py` (pygames
    /// `freetype.dll`). Verglichen wird der Rand **Punkt fuer Punkt**,
    /// samt Art und Konturenden.
    #[test]
    #[ignore]
    fn gegen_freetype() {
        let Ok(pfad) = std::env::var("RUSTGAME_KONTUR_ORAKEL") else {
            return;
        };
        let text = std::fs::read_to_string(pfad).unwrap();
        let (mut gleich, mut anders) = (0usize, Vec::new());
        for zeile in text.lines() {
            let t: Vec<&str> = zeile.split(" | ").collect();
            let radius: i64 = t[0].split(' ').nth(3).unwrap().parse().unwrap();
            let ein = umriss_aus(t[1], t[2]);
            let soll = umriss_aus(t[3], t[4]);
            let ist = konturieren(&ein, radius);
            let ok = match &ist {
                Some(k) => k.punkte == soll.punkte && k.arten == soll.arten && k.enden == soll.enden,
                None => t[3] == "FEHLER",
            };
            if ok {
                gleich += 1;
            } else {
                if anders.is_empty() {
                    if let Some(k) = &ist {
                        let n = k.punkte.iter().zip(&soll.punkte).position(|(a, b)| a != b);
                        eprintln!("erste Abweichung bei Punkt {:?} von {}/{}", n, k.punkte.len(), soll.punkte.len());
                        if let Some(n) = n {
                            eprintln!("  ist  {:?}", &k.punkte[n.saturating_sub(2)..(n + 3).min(k.punkte.len())]);
                            eprintln!("  soll {:?}", &soll.punkte[n.saturating_sub(2)..(n + 3).min(soll.punkte.len())]);
                        }
                    }
                }
                anders.push(t[0].to_string());
            }
        }
        eprintln!("gleich {gleich}, anders {}", anders.len());
        for a in anders.iter().take(20) {
            eprintln!("  {a}");
        }
        assert!(anders.is_empty());
    }
}
