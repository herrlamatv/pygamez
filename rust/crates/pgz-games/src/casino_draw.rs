//! casino_draw.rs
//! ==============
//! Zeichen-Bausteine des Casinos (alles aus pygame-Primitiven) - uebertragen
//! aus `games/casino_draw.py`: Jetons, Roulette-Kessel ([`WheelArt`]), die
//! Symbole des Lama-Slots und eine Walze an gebrochener Position.
//!
//! Die Ausdruecke stehen in derselben Reihenfolge wie in Python
//! (`n * 0.5 + cos(a) * n * 0.48`), damit jede Kommazahl gleich rundet.

use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::{PI, TAU};

use pgz_engine::prelude::*;

use crate::casino_logic as l;

type Rgb = (i32, i32, i32);

/// Rot der Zahlen.
pub const COL_RED: Rgb = (196, 36, 48);
/// Schwarz der Zahlen.
pub const COL_BLACK: Rgb = (26, 28, 34);
/// Gruen der Null.
pub const COL_GREEN: Rgb = (22, 138, 76);
/// Gold.
pub const COL_GOLD: Rgb = (232, 190, 92);
/// Dunkles Gold.
pub const COL_GOLD_D: Rgb = (150, 110, 40);
/// Holz.
pub const COL_WOOD: Rgb = (92, 52, 28);
/// Dunkles Holz.
pub const COL_WOOD_D: Rgb = (58, 32, 18);
/// Filz.
pub const COL_FELT: Rgb = (18, 92, 58);
/// Dunkler Filz.
pub const COL_FELT_D: Rgb = (10, 56, 36);

fn chip_cols(value: i64) -> Option<(Rgb, Rgb)> {
    Some(match value {
        1 => ((236, 236, 242), (40, 90, 190)),
        5 => ((204, 46, 56), (255, 255, 255)),
        25 => ((38, 150, 86), (255, 255, 255)),
        100 => ((30, 32, 40), (255, 222, 120)),
        500 => ((132, 70, 196), (255, 255, 255)),
        _ => return None,
    })
}

/// Farben der Gewinnlinien.
pub const LINE_COLS: [Rgb; 10] = [
    (255, 214, 64),
    (84, 200, 255),
    (255, 110, 150),
    (130, 235, 120),
    (255, 150, 60),
    (190, 130, 255),
    (60, 230, 200),
    (255, 90, 90),
    (160, 200, 60),
    (240, 240, 255),
];

thread_local! {
    static CACHE: RefCell<HashMap<String, Surface>> = RefCell::new(HashMap::new());
}

fn cached(key: String, build: impl FnOnce() -> Surface) -> Surface {
    if let Some(s) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return s;
    }
    let surf = build();
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 220 {
            c.clear();
        }
        c.insert(key, surf.clone());
    });
    surf
}

/// `clear_cache()`
pub fn clear_cache() {
    CACHE.with(|c| c.borrow_mut().clear());
}

/// `fmt_amount(n)` - 1250 -> "1.2k".
pub fn fmt_amount(n: i64) -> String {
    if n >= 10000 {
        return format!("{}k", py::fdiv(n, 1000));
    }
    if n >= 1000 {
        let v = n as f64 / 1000.0;
        let s = format!("{:.1}", v);
        return s.trim_end_matches('0').trim_end_matches('.').to_string() + "k";
    }
    n.to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn kurzform_wie_python() {
        let soll = ["5", "999", "1k", "1.1k", "1.2k", "1.8k", "2.2k", "2.4k", "9.9k", "10k", "10k", "12k"];
        let ist: Vec<String> = [5, 999, 1000, 1050, 1250, 1750, 2250, 2350, 9950, 9999, 10000, 12345].iter().map(|n| super::fmt_amount(*n)).collect();
        assert_eq!(ist, soll);
    }
}

/// `number_color(n)`
pub fn number_color(n: i64) -> Rgb {
    match l::color_of(n) {
        "green" => COL_GREEN,
        "red" => COL_RED,
        _ => COL_BLACK,
    }
}

fn f(v: i32) -> f64 {
    v as f64
}

// ============================================================= Jetons

/// `chip_surface(value, r, label)` - `None` = Wert als Beschriftung.
pub fn chip_surface(value: i64, r: i32, label: Option<&str>) -> Surface {
    let label = label.map(|s| s.to_string()).unwrap_or_else(|| fmt_amount(value));
    let key = format!("chip|{value}|{r}|{label}");
    cached(key, || {
        let (base, txt) = chip_cols(value).or_else(|| chip_cols(chip_for(value))).unwrap_or(((236, 236, 242), (40, 90, 190)));
        let ss = 3;
        let rr_ = r * ss;
        let size = 2 * rr_ + 2 * ss;
        let mut big = pg::surface_alpha(size, size);
        let c = (size / 2, size / 2);
        let edge = ui::mix(base, (0, 0, 0), 0.35);
        draw::circle(&mut big, edge, (c.0, c.1 + ss), rr_, 0);
        draw::circle(&mut big, base, c, rr_, 0);
        let stripe: Rgb = if value != 1 { (250, 250, 250) } else { (40, 90, 190) };
        let rf = f(rr_);
        for i in 0..6 {
            let a = i as f64 * TAU / 6.0;
            let mut pts = Vec::new();
            for da in [-0.17, 0.17] {
                for rad in [rf * 0.99, rf * 0.74] {
                    pts.push((f(c.0) + (a + da).cos() * rad, f(c.1) + (a + da).sin() * rad));
                }
            }
            draw::polygon(&mut big, stripe, &[pts[0], pts[2], pts[3], pts[1]], 0);
        }
        draw::circle(&mut big, ui::mix(base, (255, 255, 255), 0.12), c, (rf * 0.66) as i32, 0);
        draw::circle(&mut big, ui::mix(stripe, base, 0.35), c, (rf * 0.66) as i32, 1.max(ss));
        let mut surf = transform::smoothscale(&big, size / ss, size / ss);
        let fac = if label.chars().count() <= 2 { 0.78 } else { 0.62 };
        let fnt = ui::font(8.max((f(r) * fac) as i32), true, false);
        let img = fnt.render(&label, true, txt);
        let (sw, sh) = (surf.w(), surf.h());
        surf.blr(&img, img.get_rect().center_at((sw / 2, sh / 2)));
        surf
    })
}

/// `_chip_for(amount)` - groesster Jetonwert, der in `amount` passt.
pub fn chip_for(amount: i64) -> i64 {
    let mut best = l::CHIP_VALUES[0];
    for v in l::CHIP_VALUES {
        if amount >= v {
            best = v;
        }
    }
    best
}

/// `draw_chip_stack(surface, center, amount, r, alpha, lift)`
pub fn draw_chip_stack(surface: &mut Surface, center: (f64, f64), amount: i64, r: i32, alpha: i32, lift: i32) {
    let mut rest = amount;
    let mut chips = Vec::new();
    for v in l::CHIP_VALUES.iter().rev() {
        while rest >= *v && chips.len() < 5 {
            chips.push(*v);
            rest -= v;
        }
    }
    if chips.is_empty() {
        chips.push(l::CHIP_VALUES[0]);
    }
    chips.reverse();
    let (cx, cy) = (center.0 as i32, center.1 as i32 - lift);
    let step = 2.max(py::fdiv32(r, 5));
    let n = chips.len();
    for (i, v) in chips.iter().enumerate() {
        let top = i == n - 1;
        let label = if top { fmt_amount(amount) } else { String::new() };
        let mut img = chip_surface(*v, r, Some(&label));
        if alpha < 255 {
            img = img.copy();
            img.set_alpha(Some(alpha.clamp(0, 255) as u8));
        }
        surface.blr(&img, img.get_rect().center_at((cx, cy - i as i32 * step)));
    }
}

// ============================================================= Roulette-Kessel

/// `WheelArt` - Kessel in Durchmesser `d`.
pub struct WheelArt {
    /// Durchmesser.
    pub d: i32,
    /// Radius.
    pub r: f64,
    /// Kugelbahn.
    pub track_r: f64,
    /// Kugel liegt im Fach.
    pub pocket_r: f64,
    /// Rotor-Radius.
    pub rotor_r: f64,
    bowl: Surface,
    rotor: Surface,
}

impl WheelArt {
    /// `WheelArt(d)`
    pub fn new(d: i32) -> WheelArt {
        let r = d as f64 / 2.0;
        let rotor_r = r * 0.78;
        let bowl = Self::build_bowl(d, r);
        let rotor = Self::build_rotor(r, rotor_r);
        WheelArt { d, r, track_r: r * 0.855, pocket_r: r * 0.555, rotor_r, bowl, rotor }
    }

    fn build_bowl(d: i32, r: f64) -> Surface {
        let ss = 2;
        let mut big = pg::surface_alpha(d * ss, d * ss);
        let c = (py::fdiv32(d * ss, 2), py::fdiv32(d * ss, 2));
        let rr = r * ss as f64;
        draw::circle(&mut big, rgba(0, 0, 0, 90), (c.0 + 3 * ss, c.1 + 5 * ss), rr as i32, 0);
        for i in 0..10 {
            let fr = i as f64 / 9.0;
            draw::circle(&mut big, ui::mix(COL_WOOD, COL_WOOD_D, fr), c, (rr * (1.0 - 0.012 * i as f64)) as i32, 0);
        }
        draw::circle(&mut big, COL_GOLD_D, c, (rr * 0.905) as i32, 1.max(ss));
        for i in 0..12 {
            let fr = i as f64 / 11.0;
            draw::circle(&mut big, ui::mix((58, 44, 36), (24, 18, 16), fr), c, (rr * (0.90 - 0.008 * i as f64)) as i32, 0);
        }
        draw::circle(&mut big, (90, 70, 56), c, (rr * 0.80) as i32, 1.max(ss));
        for i in 0..8 {
            let a = i as f64 * TAU / 8.0 + TAU / 16.0;
            let rad = rr * 0.84;
            let (px, py_) = (f(c.0) + a.sin() * rad, f(c.1) - a.cos() * rad);
            let s = rr * 0.035;
            let pts = [
                (px + a.sin() * s * 1.6, py_ - a.cos() * s * 1.6),
                (px + a.cos() * s, py_ + a.sin() * s),
                (px - a.sin() * s * 1.6, py_ + a.cos() * s * 1.6),
                (px - a.cos() * s, py_ - a.sin() * s),
            ];
            draw::polygon(&mut big, COL_GOLD, &pts, 0);
        }
        let mut glow = pg::surface_alpha(big.w(), big.h());
        let rim = (f(c.0) - rr * 0.955, f(c.1) - rr * 0.955, rr * 1.91, rr * 1.91);
        draw::arc(&mut glow, rgba(255, 240, 220, 60), rim, PI * 0.55, PI * 0.95, 2.max((rr * 0.035) as i32));
        big.bl(&glow, (0, 0));
        transform::smoothscale(&big, d, d)
    }

    fn build_rotor(r: f64, rotor_r: f64) -> Surface {
        let ss = 2;
        let size = (rotor_r * 2.0 * ss as f64) as i32 + 2;
        let mut big = pg::surface_alpha(size, size);
        let c = (f(size) / 2.0, f(size) / 2.0);
        let rr = r * ss as f64;
        let n = l::WHEEL_ORDER.len();
        let seg = TAU / n as f64;
        let ring_poly = |a0: f64, a1: f64, r0: f64, r1: f64| -> Vec<(f64, f64)> {
            let steps = 4;
            let mut pts = Vec::new();
            for k in 0..=steps {
                let a = a0 + (a1 - a0) * k as f64 / steps as f64;
                pts.push((c.0 + a.sin() * r1, c.1 - a.cos() * r1));
            }
            for k in (0..=steps).rev() {
                let a = a0 + (a1 - a0) * k as f64 / steps as f64;
                pts.push((c.0 + a.sin() * r0, c.1 - a.cos() * r0));
            }
            pts
        };
        let num_font = ui::font(7.max((r * 0.085) as i32), true, false);
        for (i, num) in l::WHEEL_ORDER.iter().enumerate() {
            let fi = i as f64;
            let (a0, a1) = (fi * seg - seg / 2.0, fi * seg + seg / 2.0);
            let col = number_color(*num);
            draw::polygon(&mut big, col, &ring_poly(a0, a1, rr * 0.62, rr * 0.78), 0);
            draw::polygon(&mut big, ui::mix(col, (0, 0, 0), 0.45), &ring_poly(a0, a1, rr * 0.50, rr * 0.62), 0);
            let img = num_font.render(&num.to_string(), true, (245, 240, 230));
            let img = transform::smoothscale(&img, img.w() * ss, img.h() * ss);
            let img = transform::rotate(&img, -py::degrees(fi * seg));
            let px = c.0 + (fi * seg).sin() * rr * 0.70;
            let py_ = c.1 - (fi * seg).cos() * rr * 0.70;
            big.blr(&img, img.get_rect().center_at((px, py_)));
        }
        for i in 0..n {
            let a = i as f64 * seg - seg / 2.0;
            let p0 = (c.0 + a.sin() * rr * 0.50, c.1 - a.cos() * rr * 0.50);
            let p1 = (c.0 + a.sin() * rr * 0.78, c.1 - a.cos() * rr * 0.78);
            draw::line(&mut big, COL_GOLD, p0, p1, 1.max(ss));
        }
        draw::circle(&mut big, COL_GOLD, c, (rr * 0.78) as i32, 1.max(ss + 1));
        draw::circle(&mut big, COL_GOLD, c, (rr * 0.62) as i32, 1.max(ss));
        draw::circle(&mut big, COL_GOLD_D, c, (rr * 0.50) as i32, 1.max(ss + 1));
        for k in 0..16 {
            let fr = k as f64 / 15.0;
            draw::circle(&mut big, ui::mix((120, 72, 38), (196, 142, 80), fr), c, (rr * (0.49 - 0.018 * k as f64)) as i32, 0);
        }
        for k in 0..4 {
            let a = k as f64 * TAU / 4.0;
            let p1 = (c.0 + a.sin() * rr * 0.34, c.1 - a.cos() * rr * 0.34);
            draw::line(&mut big, COL_GOLD_D, c, p1, 2.max((rr * 0.05) as i32));
            draw::line(&mut big, COL_GOLD, c, p1, 1.max((rr * 0.025) as i32));
            draw::circle(&mut big, COL_GOLD, (p1.0 as i32, p1.1 as i32), 2.max((rr * 0.035) as i32), 0);
        }
        draw::circle(&mut big, COL_GOLD_D, c, (rr * 0.09) as i32, 0);
        draw::circle(&mut big, (255, 236, 170), c, (rr * 0.06) as i32, 0);
        let out = py::fdiv32(size, ss);
        transform::smoothscale(&big, out, out)
    }

    /// `pocket_angle(number)` - Winkel eines Fachs im Rotor.
    pub fn pocket_angle(&self, number: i64) -> f64 {
        let idx = l::WHEEL_ORDER.iter().position(|n| *n == number).unwrap_or(0);
        idx as f64 * TAU / l::WHEEL_ORDER.len() as f64
    }

    /// `draw(surface, center, rotation, ball, trail)`
    pub fn draw(&self, surface: &mut Surface, center: (i32, i32), rotation: f64, ball: Option<(f64, f64)>, trail: &[(f64, f64)]) {
        let (cx, cy) = center;
        surface.blr(&self.bowl, self.bowl.get_rect().center_at((cx, cy)));
        let rot = transform::rotate(&self.rotor, -py::degrees(rotation));
        surface.blr(&rot, rot.get_rect().center_at((cx, cy)));
        let Some((a, r)) = ball else {
            return;
        };
        let br = 3.max((self.r * 0.045) as i32);
        for (i, (ta, tr)) in trail.iter().enumerate() {
            let alpha = 40 + 30 * i as i32;
            let tx = f(cx) + ta.sin() * tr;
            let ty = f(cy) - ta.cos() * tr;
            let dot = ball_dot(br, alpha);
            surface.blr(&dot, dot.get_rect().center_at((tx as i32, ty as i32)));
        }
        let bx = f(cx) + a.sin() * r;
        let by = f(cy) - a.cos() * r;
        draw::circle(surface, (0, 0, 0), (bx as i32 + 1, by as i32 + 2), br, 0);
        draw::circle(surface, (236, 238, 244), (bx as i32, by as i32), br, 0);
        draw::circle(surface, (255, 255, 255), ((bx - f(br) * 0.35) as i32, (by - f(br) * 0.35) as i32), 1.max(br / 3), 0);
    }
}

fn ball_dot(r: i32, alpha: i32) -> Surface {
    cached(format!("ball|{r}|{alpha}"), || {
        let mut s = pg::surface_alpha(2 * r + 2, 2 * r + 2);
        draw::circle(&mut s, rgba(236, 238, 244, alpha), (r + 1, r + 1), r, 0);
        s
    })
}

// ============================================================= Slot-Symbole

/// `symbol_surface(sym, size, blur)`
pub fn symbol_surface(sym: &str, size: i32, blur: bool) -> Surface {
    let key = format!("sym|{sym}|{size}|{blur}");
    cached(key, || {
        if blur {
            let sharp = symbol_surface(sym, size, false);
            let mut out = pg::surface_alpha(size, size);
            for (dy, a) in [(py::fdiv32(-size, 7), 70u8), (py::fdiv32(size, 7), 70), (0, 150)] {
                let mut layer = sharp.copy();
                layer.set_alpha(Some(a));
                out.bl(&layer, (0, dy));
            }
            return out;
        }
        let ss = 2;
        let mut big = pg::surface_alpha(size * ss, size * ss);
        let n = size * ss;
        match sym {
            "cherry" => sym_cherry(&mut big, n),
            "lemon" => sym_lemon(&mut big, n),
            "grape" => sym_grape(&mut big, n),
            "clover" => sym_clover(&mut big, n),
            "bell" => sym_bell(&mut big, n),
            "gem" => sym_gem(&mut big, n),
            "seven" => sym_seven(&mut big, n),
            "lama" => sym_lama(&mut big, n),
            _ => sym_coin(&mut big, n),
        }
        transform::smoothscale(&big, size, size)
    })
}

fn shade(col: Rgb, fv: f64) -> Color {
    ui::mix(col, if fv > 0.0 { (255, 255, 255) } else { (0, 0, 0) }, fv.abs())
}

fn sym_cherry(s: &mut Surface, ni: i32) {
    let n = f(ni);
    let red: Rgb = (214, 30, 52);
    let stem: Rgb = (70, 150, 60);
    draw::lines(s, stem, false, &[(n * 0.33, n * 0.62), (n * 0.46, n * 0.34), (n * 0.58, n * 0.18)], 2.max(ni / 28));
    draw::lines(s, stem, false, &[(n * 0.68, n * 0.66), (n * 0.64, n * 0.38), (n * 0.58, n * 0.18)], 2.max(ni / 28));
    draw::ellipse(s, (60, 170, 70), (n * 0.56, n * 0.12, n * 0.26, n * 0.13), 0);
    for (cx, cy) in [(0.32, 0.68), (0.66, 0.70)] {
        let r = n * 0.17;
        draw::circle(s, shade(red, -0.35), (n * cx + n * 0.015, n * cy + n * 0.02), r, 0);
        draw::circle(s, red, (n * cx, n * cy), r, 0);
        draw::circle(s, shade(red, 0.55), (n * cx - r * 0.35, n * cy - r * 0.35), r * 0.28, 0);
    }
}

fn sym_lemon(s: &mut Surface, ni: i32) {
    let n = f(ni);
    let y: Rgb = (250, 214, 50);
    let pts = [
        (n * 0.14, n * 0.52),
        (n * 0.30, n * 0.30),
        (n * 0.52, n * 0.24),
        (n * 0.74, n * 0.32),
        (n * 0.88, n * 0.50),
        (n * 0.74, n * 0.70),
        (n * 0.52, n * 0.77),
        (n * 0.30, n * 0.70),
    ];
    let sh: Vec<(f64, f64)> = pts.iter().map(|(x, yy)| (x + n * 0.015, yy + n * 0.02)).collect();
    draw::polygon(s, shade(y, -0.3), &sh, 0);
    draw::polygon(s, y, &pts, 0);
    draw::ellipse(s, y, (n * 0.20, n * 0.26, n * 0.62, n * 0.50), 0);
    draw::ellipse(s, shade(y, 0.55), (n * 0.32, n * 0.34, n * 0.22, n * 0.10), 0);
    draw::circle(s, shade(y, -0.35), (n * 0.88, n * 0.50), n * 0.03, 0);
}

fn sym_grape(s: &mut Surface, ni: i32) {
    let n = f(ni);
    let p: Rgb = (136, 58, 176);
    draw::line(s, (90, 140, 60), (n * 0.50, n * 0.12), (n * 0.52, n * 0.30), 2.max(ni / 26));
    draw::ellipse(s, (80, 170, 80), (n * 0.52, n * 0.10, n * 0.28, n * 0.15), 0);
    let rows: [(f64, &[f64]); 5] = [
        (0.30, &[0.32, 0.50, 0.68]),
        (0.47, &[0.40, 0.60]),
        (0.47, &[0.22, 0.78]),
        (0.63, &[0.32, 0.50, 0.68]),
        (0.79, &[0.41, 0.59]),
    ];
    let r = n * 0.10;
    for (cy, xs) in rows {
        for &cx in xs {
            if cy == 0.47 && (cx == 0.22 || cx == 0.78) {
                continue;
            }
            draw::circle(s, shade(p, -0.35), (n * cx + n * 0.01, n * cy + n * 0.015), r, 0);
            draw::circle(s, p, (n * cx, n * cy), r, 0);
            draw::circle(s, shade(p, 0.5), (n * cx - r * 0.35, n * cy - r * 0.35), r * 0.28, 0);
        }
    }
    for cx in [0.41, 0.59] {
        draw::circle(s, p, (n * cx, n * 0.47), r, 0);
        draw::circle(s, shade(p, 0.5), (n * cx - r * 0.35, n * 0.47 - r * 0.35), r * 0.28, 0);
    }
}

fn sym_clover(s: &mut Surface, ni: i32) {
    let n = f(ni);
    let g: Rgb = (46, 176, 84);
    draw::lines(s, shade(g, -0.3), false, &[(n * 0.50, n * 0.52), (n * 0.56, n * 0.76), (n * 0.66, n * 0.88)], 3.max(ni / 18));
    for pass in 0..2 {
        for a in 0..4 {
            let ang = a as f64 * TAU / 4.0 + TAU / 8.0;
            let cx = n * 0.5 + ang.cos() * n * 0.17;
            let cy = n * 0.46 + ang.sin() * n * 0.17;
            for da in [-0.55, 0.55] {
                let px = cx + (ang + da).cos() * n * 0.07;
                let py_ = cy + (ang + da).sin() * n * 0.07;
                if pass == 0 {
                    draw::circle(s, shade(g, -0.25), (px + n * 0.01, py_ + n * 0.015), n * 0.12, 0);
                } else {
                    draw::circle(s, g, (px, py_), n * 0.12, 0);
                }
            }
        }
    }
    draw::circle(s, shade(g, 0.35), (n * 0.5, n * 0.46), n * 0.05, 0);
}

fn sym_bell(s: &mut Surface, ni: i32) {
    let n = f(ni);
    let gold: Rgb = (246, 190, 50);
    let body = [
        (n * 0.50, n * 0.14),
        (n * 0.66, n * 0.22),
        (n * 0.72, n * 0.44),
        (n * 0.78, n * 0.64),
        (n * 0.88, n * 0.74),
        (n * 0.12, n * 0.74),
        (n * 0.22, n * 0.64),
        (n * 0.28, n * 0.44),
        (n * 0.34, n * 0.22),
    ];
    let sh: Vec<(f64, f64)> = body.iter().map(|(x, y)| (x + n * 0.015, y + n * 0.02)).collect();
    draw::polygon(s, shade(gold, -0.35), &sh, 0);
    draw::polygon(s, gold, &body, 0);
    draw::circle(s, gold, (n * 0.50, n * 0.30), n * 0.17, 0);
    draw::rect_r(s, shade(gold, -0.25), (n * 0.10, n * 0.72, n * 0.80, n * 0.07), 0, (n * 0.03) as i32);
    draw::circle(s, shade(gold, -0.45), (n * 0.50, n * 0.84), n * 0.07, 0);
    draw::circle(s, shade(gold, -0.2), (n * 0.50, n * 0.12), n * 0.05, 0);
    draw::ellipse(s, shade(gold, 0.6), (n * 0.34, n * 0.24, n * 0.09, n * 0.30), 0);
}

fn sym_gem(s: &mut Surface, ni: i32) {
    let n = f(ni);
    let c: Rgb = (70, 210, 240);
    let top = n * 0.26;
    let mid = n * 0.42;
    let pts = [(n * 0.28, top), (n * 0.72, top), (n * 0.90, mid), (n * 0.50, n * 0.88), (n * 0.10, mid)];
    let sh: Vec<(f64, f64)> = pts.iter().map(|(x, y)| (x + n * 0.015, y + n * 0.02)).collect();
    draw::polygon(s, shade(c, -0.45), &sh, 0);
    draw::polygon(s, c, &pts, 0);
    draw::polygon(s, shade(c, 0.45), &[(n * 0.28, top), (n * 0.50, top), (n * 0.38, mid), (n * 0.10, mid)], 0);
    draw::polygon(s, shade(c, 0.2), &[(n * 0.50, top), (n * 0.72, top), (n * 0.62, mid), (n * 0.38, mid)], 0);
    draw::polygon(s, shade(c, -0.2), &[(n * 0.62, mid), (n * 0.90, mid), (n * 0.50, n * 0.88)], 0);
    draw::polygon(s, shade(c, 0.15), &[(n * 0.38, mid), (n * 0.62, mid), (n * 0.50, n * 0.88)], 0);
    draw::polygon(s, (255, 255, 255), &pts, 1.max(ni / 60));
    draw::circle(s, (255, 255, 255), (n * 0.30, n * 0.31), n * 0.03, 0);
}

fn sym_seven(s: &mut Surface, ni: i32) {
    let n = f(ni);
    let fnt = ui::font((n * 0.78) as i32, true, false);
    let red: Rgb = (226, 36, 48);
    let img = fnt.render("7", true, red);
    let rect = img.get_rect().center_at((n * 0.5, n * 0.52));
    let edge = fnt.render("7", true, (120, 12, 20));
    let gold = fnt.render("7", true, (255, 214, 90));
    let o = 2.max(ni / 40);
    for (dx, dy) in [(-o, 0), (o, 0), (0, -o), (0, o), (-o, -o), (o, o), (-o, o), (o, -o)] {
        s.blr(&gold, rect.move_(dx, dy));
    }
    s.blr(&edge, rect.move_(o, o + o / 2));
    s.blr(&img, rect);
    let shine = fnt.render("7", true, (255, 150, 150));
    let mut clip = pg::surface_alpha(shine.w(), shine.h());
    clip.bl(&shine, (0, 0));
    let (cw, ch) = (clip.w(), clip.h());
    clip.fill_rf(rgba(255, 255, 255, 0), (0, ch / 2, cw, ch), BLEND_RGBA_MULT);
    clip.set_alpha(Some(90));
    s.blr(&clip, rect.move_(py::fdiv32(-o, 2), py::fdiv32(-o, 2)));
}

fn llama_head(s: &mut Surface, cx: f64, cy: f64, n: f64, wool: Color, dark: Option<Color>) {
    for side in [-1.0, 1.0] {
        let ear = [
            (cx + side * n * 0.10, cy - n * 0.16),
            (cx + side * n * 0.20, cy - n * 0.44),
            (cx + side * n * 0.26, cy - n * 0.14),
        ];
        draw::polygon(s, wool, &ear, 0);
        if let Some(dk) = dark {
            let inner = [
                (cx + side * n * 0.14, cy - n * 0.17),
                (cx + side * n * 0.20, cy - n * 0.36),
                (cx + side * n * 0.23, cy - n * 0.16),
            ];
            draw::polygon(s, dk, &inner, 0);
        }
    }
    draw::ellipse(s, wool, (cx - n * 0.21, cy - n * 0.22, n * 0.42, n * 0.40), 0);
    for k in 0..5 {
        draw::circle(s, wool, (cx - n * 0.14 + k as f64 * n * 0.07, cy - n * 0.22), n * 0.06, 0);
    }
    draw::ellipse(s, wool, (cx - n * 0.15, cy + n * 0.02, n * 0.30, n * 0.24), 0);
}

fn sym_lama(s: &mut Surface, ni: i32) {
    let n = f(ni);
    let wool = rgb(250, 244, 230);
    let shadow = rgb(206, 190, 168);
    let dark = rgb(60, 44, 40);
    let pink = rgb(240, 150, 160);
    let glow: Rgb = (255, 200, 80);
    for k in 0..12 {
        let a = k as f64 * TAU / 12.0;
        let pts = [
            (n * 0.5, n * 0.44),
            (n * 0.5 + (a - 0.12).cos() * n * 0.48, n * 0.44 + (a - 0.12).sin() * n * 0.48),
            (n * 0.5 + (a + 0.12).cos() * n * 0.48, n * 0.44 + (a + 0.12).sin() * n * 0.48),
        ];
        draw::polygon(s, ui::mix(glow, (255, 255, 255), 0.25), &pts, 0);
    }
    draw::circle(s, glow, (n * 0.5, n * 0.44), n * 0.32, 0);
    let (cx, cy) = (n * 0.5, n * 0.44);
    draw::rect_r(s, shadow, (cx - n * 0.13, cy + n * 0.12, n * 0.26, n * 0.30), 0, (n * 0.05) as i32);
    llama_head(s, cx + n * 0.012, cy + n * 0.018, n, shadow, None);
    llama_head(s, cx, cy, n, wool, Some(pink));
    for side in [-1.0, 1.0] {
        let (ex, ey) = (cx + side * n * 0.09, cy - n * 0.03);
        draw::circle(s, dark, (ex, ey), n * 0.035, 0);
        draw::circle(s, (255, 255, 255), (ex - n * 0.01, ey - n * 0.012), n * 0.011, 0);
        draw::circle(s, pink, (cx + side * n * 0.15, cy + n * 0.07), n * 0.03, 0);
    }
    draw::circle(s, dark, (cx - n * 0.035, cy + n * 0.10), n * 0.013, 0);
    draw::circle(s, dark, (cx + n * 0.035, cy + n * 0.10), n * 0.013, 0);
    draw::arc(s, dark, (cx - n * 0.05, cy + n * 0.10, n * 0.10, n * 0.07), PI * 1.1, PI * 1.9, 1.max(ni / 60));
    let band = Rect::new(0, 0, (n * 0.70) as i32, (n * 0.20) as i32).center_at(((n * 0.5) as i32, (n * 0.84) as i32));
    draw::rect_r(s, (150, 30, 90), band.move_(0, ni / 60), 0, (n * 0.06) as i32);
    draw::rect_r(s, (214, 51, 108), band, 0, (n * 0.06) as i32);
    draw::rect_r(s, (255, 214, 90), band, 1.max(ni / 60), (n * 0.06) as i32);
    let fnt = ui::font((n * 0.15) as i32, true, false);
    let img = fnt.render("WILD", true, (255, 244, 220));
    s.blr(&img, img.get_rect().center_at(band.center()));
}

fn sym_coin(s: &mut Surface, ni: i32) {
    let n = f(ni);
    let gold: Rgb = (246, 196, 60);
    let c = (n * 0.5, n * 0.5);
    draw::circle(s, shade(gold, -0.45), (c.0 + n * 0.02, c.1 + n * 0.03), n * 0.40, 0);
    draw::circle(s, shade(gold, -0.15), c, n * 0.40, 0);
    draw::circle(s, gold, c, n * 0.34, 0);
    draw::circle(s, shade(gold, -0.25), c, n * 0.34, 2.max(ni / 45));
    for k in 0..24 {
        let a = k as f64 * TAU / 24.0;
        let p0 = (c.0 + a.cos() * n * 0.37, c.1 + a.sin() * n * 0.37);
        let p1 = (c.0 + a.cos() * n * 0.40, c.1 + a.sin() * n * 0.40);
        draw::line(s, shade(gold, -0.35), p0, p1, 1.max(ni / 80));
    }
    llama_head(s, c.0 + n * 0.01, c.1 + n * 0.05, n * 0.72, shade(gold, -0.30), None);
    llama_head(s, c.0, c.1 + n * 0.04, n * 0.72, shade(gold, 0.30), None);
    draw::circle(s, shade(gold, -0.5), (c.0 - n * 0.06, c.1 + n * 0.02), n * 0.02, 0);
    draw::circle(s, shade(gold, -0.5), (c.0 + n * 0.06, c.1 + n * 0.02), n * 0.02, 0);
    draw::arc(s, (255, 250, 220), (c.0 - n * 0.30, c.1 - n * 0.30, n * 0.60, n * 0.60), PI * 0.55, PI * 0.95, 2.max(ni / 40));
    for (sx, sy, r) in [(0.80, 0.18, 0.06), (0.18, 0.80, 0.04)] {
        let (x, y) = (n * sx, n * sy);
        let pts = [
            (x, y - n * r),
            (x + n * r * 0.3, y - n * r * 0.3),
            (x + n * r, y),
            (x + n * r * 0.3, y + n * r * 0.3),
            (x, y + n * r),
            (x - n * r * 0.3, y + n * r * 0.3),
            (x - n * r, y),
            (x - n * r * 0.3, y - n * r * 0.3),
        ];
        draw::polygon(s, (255, 255, 230), &pts, 0);
    }
}

// ============================================================= Walzen

/// `reel_shade(w, h)` - Zylinder-Schattierung.
pub fn reel_shade(w: i32, h: i32) -> Surface {
    cached(format!("shade|{w}|{h}"), || {
        let mut s = pg::surface_alpha(w, h);
        let band = 4.max(h / 4);
        for y in 0..band {
            let a = (150.0 * py::pow(1.0 - y as f64 / band as f64, 1.6)) as i32;
            s.fill_r(rgba(0, 0, 0, a), (0, y, w, 1));
            s.fill_r(rgba(0, 0, 0, a), (0, h - 1 - y, w, 1));
        }
        s
    })
}

fn dim_cell(w: i32, h: i32) -> Surface {
    cached(format!("dim|{w}|{h}"), || {
        let mut s = pg::surface_alpha(w, h);
        s.fill_c(rgba(0, 0, 0, 120));
        s
    })
}

/// `draw_reel(surface, rect, strip, pos, cell, speed, dim_rows, pop)`
///
/// `dim_rows`/`pop` je Reihe; `None` wie in Python (keine Abdunklung /
/// kein Huepfen). `pop` enthaelt nur Reihen mit Skalierung.
#[allow(clippy::too_many_arguments)]
pub fn draw_reel(
    surface: &mut Surface,
    r: Rect,
    strip: &[&str],
    pos: f64,
    cell: i32,
    speed: f64,
    dim_rows: Option<&[usize]>,
    pop: Option<&[(usize, f64)]>,
) {
    let n = strip.len() as i64;
    let base = pos.floor();
    let frac = pos - base;
    let base = base as i64;
    let blur = speed >= 8.0;
    let old_clip = surface.get_clip();
    let clip = if old_clip.w != 0 && old_clip.h != 0 { r.clip(old_clip) } else { r };
    surface.set_clip(Some(clip));
    let pad = 2.max(cell / 14);
    let size = cell - 2 * pad;
    for i in -1..(l::ROWS as i64 + 1) {
        let sym = strip[py::pmod(base + i, n) as usize];
        let y = r.y + py::round((i as f64 - frac) * cell as f64) as i32;
        let row = if frac < 1e-6 { Some(i) } else { None };
        let scale = match (pop, row) {
            (Some(p), Some(rw)) if !p.is_empty() => p.iter().find(|(k, _)| *k as i64 == rw).map(|e| e.1).unwrap_or(1.0),
            _ => 1.0,
        };
        if scale != 1.0 {
            let sz = 4.max((size as f64 * scale) as i32);
            let img = transform::smoothscale(&symbol_surface(sym, size, false), sz, sz);
            surface.blr(&img, img.get_rect().center_at((r.centerx(), y + cell / 2)));
        } else {
            let img = symbol_surface(sym, size, blur);
            surface.bl(&img, (r.centerx() - size / 2, y + pad));
        }
        if let (Some(d), Some(rw)) = (dim_rows, row) {
            if d.iter().any(|k| *k as i64 == rw) {
                let dim = dim_cell(r.w, cell);
                surface.bl(&dim, (r.x, y));
            }
        }
    }
    let shade = reel_shade(r.w, r.h);
    surface.bl(&shade, (r.x, r.y));
    surface.set_clip(Some(old_clip));
}
