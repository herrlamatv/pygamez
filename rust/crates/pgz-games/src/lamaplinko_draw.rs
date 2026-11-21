//! lamaplinko_draw.rs
//! ==================
//! Zeichen-Bausteine von Lama Plinko - uebertragen aus
//! `games/lamaplinko_draw.py` (nur Primitive, keine Bilddateien, alles
//! Teure gecacht): Spielhallen-Hintergrund, Trapez-Platte, Stifte (+
//! Leuchten), Wollknaeuel in 12 Drehstufen, Spurpunkte, Faecher und Schilder
//! in der Multiplikator-Farbe, Einwurf-Trichter, das nickende Lama (aus
//! `lamacrossing_draw`), Piktogramme und die Mini-Vorschau eines Kugelwegs.
//!
//! Alle Formen haengen nur von Groessen und ganzzahligen Werten ab; die
//! Ausdruecke stehen in derselben Reihenfolge wie in Python, damit jede
//! Kommazahl gleich rundet. Farbverlaeufe rechnen ganzzahlig (`lerp_i`, mit
//! Pythons `//`).

use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::PI;

use pgz_engine::prelude::*;

use crate::lamacrossing_draw as lcd;

/// RGB als ganze Zahlen (wie die Python-Tupel).
pub type Rgb = (i32, i32, i32);

/// Gold des Wollknaeuels.
pub const GOLD: Rgb = (236, 184, 64);
/// helles Gold.
pub const GOLD_L: Rgb = (255, 228, 140);
/// dunkles Gold.
pub const GOLD_D: Rgb = (168, 112, 30);
/// sehr dunkles Gold.
pub const GOLD_DD: Rgb = (110, 68, 18);
const PEG: Rgb = (226, 230, 246);
const PEG_D: Rgb = (120, 124, 150);

/// Fach-Farben: (Multiplikator x100, Farbe) - dazwischen linear (ganzzahlig).
const SLOT_STOPS: [(i64, Rgb); 10] = [
    (0, (40, 96, 196)),
    (20, (52, 120, 222)),
    (50, (40, 170, 206)),
    (100, (70, 196, 140)),
    (150, (150, 210, 84)),
    (200, (236, 206, 60)),
    (500, (248, 154, 44)),
    (2000, (240, 86, 52)),
    (10000, (226, 40, 86)),
    (100000, (200, 50, 226)),
];

/// Drehpunkt (Hufe) relativ zur Sprite-Groesse.
pub const LLAMA_PIVOT: (f64, f64) = (0.42, 0.94);
/// Schnauzenspitze relativ zur Sprite-Groesse (Blick nach rechts).
pub const LLAMA_MOUTH: (f64, f64) = (0.895, 0.225);

thread_local! {
    static CACHE: RefCell<HashMap<String, Surface>> = RefCell::new(HashMap::new());
}

/// `_cached(key, build)` - wie Python (ab 400 Eintraegen alles leeren).
fn cached(key: String, build: impl FnOnce() -> Surface) -> Surface {
    if let Some(s) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return s;
    }
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 400 {
            c.clear();
        }
    });
    let surf = build();
    CACHE.with(|c| c.borrow_mut().insert(key, surf.clone()));
    surf
}

/// `clear_cache()`
pub fn clear_cache() {
    CACHE.with(|c| c.borrow_mut().clear());
}

/// `_ri(v)` - kaufmaennisch runden (x.5 nach oben).
pub fn ri(v: f64) -> i32 {
    (v + 0.5).floor() as i32
}

/// Farbe (u8) als ganzzahliges Tupel.
pub fn rgb_of(c: Color) -> Rgb {
    (c.r as i32, c.g as i32, c.b as i32)
}

/// `ui.mix(...)` als ganzzahliges Tupel.
fn mixt(c1: impl Col, c2: impl Col, f: f64) -> Rgb {
    rgb_of(ui::mix(c1, c2, f))
}

/// `_lerp_i(c1, c2, num, den)` - `a + (b - a) * num // den` (Pythons `//`).
fn lerp_i(c1: Rgb, c2: Rgb, num: i64, den: i64) -> Rgb {
    let den = 1.max(den);
    let f = |a: i32, b: i32| (a as i64 + py::fdiv((b - a) as i64 * num, den)) as i32;
    (f(c1.0, c2.0), f(c1.1, c2.1), f(c1.2, c2.2))
}

/// `_gradient(surface, rect, top, bottom)` - senkrechter Verlauf (ganzzahlig).
fn gradient(s: &mut Surface, r: (i32, i32, i32, i32), top: Rgb, bottom: Rgb) {
    let (x, y, w, rh) = r;
    let h = 1.max(rh - 1);
    for yy in 0..rh {
        s.fill_r(lerp_i(top, bottom, yy as i64, h as i64), (x, y + yy, w, 1));
    }
}

/// `_circle_mask(size, c, r)`
fn circle_mask(size: i32, c: (i32, i32), r: i32) -> Surface {
    let mut mask = pg::surface_alpha(size, size);
    mask.fill_c((255, 255, 255, 0));
    draw::circle(&mut mask, (255, 255, 255, 255), c, r, 0);
    mask
}

fn key_rgb(c: Color) -> String {
    format!("{},{},{}", c.r, c.g, c.b)
}

// ============================================================= Hintergrund

/// `background(w, h, glow_center, glow_r, accent)` - Verlauf, Lichtschein
/// hinter dem Brett, Punktraster, Vignette.
pub fn background(w: i32, h: i32, glow_center: (i32, i32), glow_r: i32, accent: Color) -> Surface {
    let key = format!("bg|{w}|{h}|{}|{}|{glow_r}|{}", glow_center.0, glow_center.1, key_rgb(accent));
    cached(key, || {
        let mut s = pg::surface(w, h);
        gradient(&mut s, (0, 0, w, h), (26, 16, 38), (8, 6, 16));
        // weicher Lichtschein: konzentrische Kreise von aussen nach innen
        let (gx, gy) = glow_center;
        let mut glow = pg::surface_alpha(w, h);
        let steps = 40;
        let ac = rgb_of(accent);
        let rgb3 = (ac.0, ac.1 / 2 + 40, ac.2 / 3);
        for i in (1..=steps).rev() {
            let a = (steps - i + 1) * 58 / steps;
            draw::circle(&mut glow, (rgb3.0, rgb3.1, rgb3.2, a), (gx, gy), 2.max(glow_r * i / steps), 0);
        }
        s.bl(&glow, (0, 0));
        // feines Punktraster
        let step = 14.max(w.min(h) / 26);
        let dot = (40, 30, 56);
        let big = if w >= 1200 { 2 } else { 1 };
        let mut yy = step / 2;
        while yy < h {
            let mut xx = step / 2 + (yy / step % 2) * (step / 2);
            while xx < w {
                s.fill_r(dot, (xx, yy, big, big));
                xx += step;
            }
            yy += step;
        }
        // dunkle Raender (Vignette)
        let mut vig = pg::surface_alpha(w, h);
        let band = 8.max(w.min(h) / 7);
        for i in 0..band {
            let a = (band - i) * 90 / band;
            draw::rect(&mut vig, (0, 0, 0, a / 3), (i, i, w - 2 * i, h - 2 * i), 1);
        }
        s.bl(&vig, (0, 0));
        s
    })
}

/// `board_plate(points, w, h, accent)` - Trapez-Platte unter den Stiften.
pub fn board_plate(points: &[(f64, f64); 4], w: i32, h: i32, accent: Color) -> Surface {
    let pts: Vec<(i32, i32)> = points.iter().map(|&(x, y)| (x as i32, y as i32)).collect();
    let key = format!("plate|{:?}|{w}|{h}|{}", pts, key_rgb(accent));
    cached(key, || {
        let ac = rgb_of(accent);
        let mut s = pg::surface_alpha(w, h);
        // Verlauf (oben warm, unten dunkel), mit dem Trapez ausgestanzt
        let top = (ac.0 / 4 + 22, ac.1 / 6 + 12, ac.2 / 6 + 26);
        let bot = (ac.0 / 9 + 14, ac.1 / 12 + 8, ac.2 / 10 + 22);
        let y0 = pts.iter().map(|p| p.1).min().unwrap_or(0);
        let y1 = pts.iter().map(|p| p.1).max().unwrap_or(0);
        let hh = 1.max(y1 - y0);
        for y in y0..=y1 {
            let c = lerp_i(top, bot, (y - y0) as i64, hh as i64);
            s.fill_r((c.0, c.1, c.2, 150), (0, y, w, 1));
        }
        let mut mask = pg::surface_alpha(w, h);
        draw::polygon(&mut mask, (255, 255, 255, 255), &pts, 0);
        s.bl_flags(&mask, (0, 0), BLEND_RGBA_MIN);
        draw::polygon(&mut s, pg::alpha(ui::mix(accent, (255, 255, 255), 0.25), 80), &pts, 2);
        // Lichtkante oben + feiner Schimmer an den Schraegen
        draw::line(&mut s, pg::alpha(ui::mix(accent, (255, 255, 255), 0.55), 140), pts[0], pts[1], 2);
        draw::line(
            &mut s,
            pg::alpha(ui::mix(accent, (255, 255, 255), 0.4), 50),
            (pts[0].0 + 3, pts[0].1 + 2),
            (pts[3].0 + 3, pts[3].1 - 2),
            1,
        );
        draw::line(
            &mut s,
            pg::alpha(ui::mix(accent, (255, 255, 255), 0.4), 50),
            (pts[1].0 - 3, pts[1].1 + 2),
            (pts[2].0 - 3, pts[2].1 - 2),
            1,
        );
        s
    })
}

// ============================================================= Stifte

/// `peg_sprite(r)` - Schatten, Koerper, Glanzpunkt (4-fach ueberabgetastet).
pub fn peg_sprite(r: i32) -> Surface {
    cached(format!("peg|{r}"), || {
        let ss = 4;
        let size = (2 * r + 4) * ss;
        let mut big = pg::surface_alpha(size, size);
        let c = (size / 2, size / 2);
        let rr = r * ss;
        draw::circle(&mut big, (0, 0, 0, 110), (c.0 + ss, c.1 + 2 * ss), rr, 0);
        draw::circle(&mut big, PEG_D, c, rr, 0);
        draw::circle(&mut big, PEG, (c.0 - ss / 2, c.1 - ss / 2), rr - ss, 0);
        draw::circle(&mut big, (255, 255, 255), (c.0 - rr / 3, c.1 - rr / 3), ss.max(rr / 3), 0);
        transform::smoothscale(&big, size / ss, size / ss)
    })
}

/// `peg_layer(w, h, pegs, r)` - alle Stifte auf einer Flaeche.
pub fn peg_layer(w: i32, h: i32, pegs: &[(i32, i32)], r: i32) -> Surface {
    let key = format!("pegs|{w}|{h}|{r}|{:?}", pegs);
    cached(key, || {
        let mut s = pg::surface_alpha(w, h);
        let spr = peg_sprite(r);
        let off = spr.w() / 2;
        for &(x, y) in pegs {
            s.bl(&spr, (x - off, y - off));
        }
        s
    })
}

/// `peg_glow(r, level)` - Leuchten eines getroffenen Stifts (1..8).
pub fn peg_glow(r: i32, level: i32) -> Surface {
    cached(format!("glow|{r}|{level}"), || {
        let rr = (r as f64 * 3.2) as i32 + 2;
        let mut s = pg::surface_alpha(2 * rr + 2, 2 * rr + 2);
        let c = (rr + 1, rr + 1);
        for i in (1..=6).rev() {
            let ri_ = 1.max(rr * i / 6);
            let a = (7 - i) * 10 * level / 8;
            draw::circle(&mut s, (255, 214, 120, a), c, ri_, 0);
        }
        draw::circle(&mut s, (255, 250, 220, 40 + 26 * level), c, r + 1, 0);
        s
    })
}

// ============================================================= Kugel

/// `yarn_ball(r, step)` - goldenes Wollknaeuel mit Radius r in Drehstufe
/// step (0..11).
pub fn yarn_ball(r: i32, step: i64) -> Surface {
    let step = py::pmod(step, 12);
    cached(format!("yarn|{r}|{step}"), || {
        let ss = 4;
        let size = (2 * r + 2) * ss;
        let rr = r * ss;
        let rf = rr as f64;
        let c = (size / 2, size / 2);
        let (c0, c1) = (c.0 as f64, c.1 as f64);
        let mut big = pg::surface_alpha(size, size);
        draw::circle(&mut big, GOLD_D, c, rr, 0);
        draw::circle(&mut big, GOLD, (c.0 - rr / 8, c.1 - rr / 8), rr - rr / 7, 0);
        // Wollfaeden: gebogene Linien quer ueber das Knaeuel, gedreht je Stufe
        let ang = step as f64 * PI / 6.0;
        let (ca, sa) = (ang.cos(), ang.sin());
        let lw = ss.max(rr / 6);
        let lagen: [(f64, Rgb); 5] = [(-0.52, GOLD_DD), (-0.12, GOLD_D), (0.30, GOLD_DD), (0.66, GOLD_D), (0.05, GOLD_L)];
        for (i, &(off, col)) in lagen.iter().enumerate() {
            let half = 0f64.max(1.0 - off * off).sqrt() * 0.98;
            let mut pts: Vec<(f64, f64)> = Vec::with_capacity(9);
            for j in 0..9 {
                let u = -half + 2.0 * half * j as f64 / 8.0;
                let v = off + 0.22 * if half > 0.0 { 1.0 - py::pow(u / half, 2.0) } else { 0.0 };
                let (x, y) = (u * rf, v * rf);
                pts.push((c0 + x * ca - y * sa, c1 + x * sa + y * ca));
            }
            let w = if i < 4 { lw } else { ss.max(lw * 2 / 3) };
            draw::lines(&mut big, col, false, &pts, w);
        }
        // zweite Fadenlage quer dazu
        let ang2 = ang + PI / 2.4;
        let (ca, sa) = (ang2.cos(), ang2.sin());
        for off in [-0.34f64, 0.40] {
            let half = 0f64.max(1.0 - off * off).sqrt() * 0.98;
            let mut pts: Vec<(f64, f64)> = Vec::with_capacity(9);
            for j in 0..9 {
                let u = -half + 2.0 * half * j as f64 / 8.0;
                let v = off - 0.18 * (1.0 - py::pow(u / half, 2.0));
                let (x, y) = (u * rf, v * rf);
                pts.push((c0 + x * ca - y * sa, c1 + x * sa + y * ca));
            }
            draw::lines(&mut big, GOLD_D, false, &pts, ss.max(lw * 3 / 4));
        }
        // Licht oben links, Schatten unten rechts (fest, dreht nicht mit)
        let mut shade = pg::surface_alpha(size, size);
        draw::circle(&mut shade, (90, 50, 10, 90), (c.0 + rr / 3, c.1 + rr / 3), rr, 0);
        draw::circle(&mut shade, (0, 0, 0, 0), (c.0 - rr / 6, c.1 - rr / 6), rr * 4 / 5, 0);
        big.bl(&shade, (0, 0));
        draw::circle(&mut big, (255, 246, 210), (c.0 - rr * 2 / 5, c.1 - rr * 2 / 5), ss.max(rr / 5), 0);
        big.bl_flags(&circle_mask(size, c, rr), (0, 0), BLEND_RGBA_MIN);
        draw::circle(&mut big, (70, 40, 10), c, rr, ss.max(rr / 9));
        transform::smoothscale(&big, size / ss, size / ss)
    })
}

/// `trail_dot(r, alpha)` - Leuchtpunkt der Kugelspur (alpha in 16er-Stufen).
pub fn trail_dot(r: i32, alpha: i32) -> Surface {
    let alpha = 16.max(255.min(alpha / 16 * 16));
    cached(format!("trail|{r}|{alpha}"), || {
        let rr = 1.max(r);
        let mut s = pg::surface_alpha(2 * rr + 2, 2 * rr + 2);
        draw::circle(&mut s, (255, 196, 80, alpha / 2), (rr + 1, rr + 1), rr, 0);
        draw::circle(&mut s, (255, 230, 150, alpha), (rr + 1, rr + 1), 1.max(rr * 3 / 5), 0);
        s
    })
}

// ============================================================= Faecher

/// `slot_color(m100)` - kuehl (unter x1) ueber Gelb und Orange bis Magenta.
pub fn slot_color(m100: i64) -> Rgb {
    let m100 = 0.max(m100);
    for w in SLOT_STOPS.windows(2) {
        let ((a, ca), (b, cb)) = (w[0], w[1]);
        if m100 <= b {
            return lerp_i(ca, cb, m100 - a, b - a);
        }
    }
    SLOT_STOPS[SLOT_STOPS.len() - 1].1
}

/// `_text_dark(col)` - dunkle Schrift auf hellen Faechern?
fn text_dark(col: Rgb) -> bool {
    (col.0 * 299 + col.1 * 587 + col.2 * 114) / 1000 > 150
}

/// `fit_font(text, max_w, size, bold, min_size=7)` - groesste Schrift ab size
/// abwaerts, in der text in max_w passt.
pub fn fit_font(text: &str, max_w: i32, size: i32, bold: bool) -> Font {
    let mut size = size;
    while size > 7 && ui::font(size, bold, false).width(text) > max_w {
        size -= 1;
    }
    ui::font(size, bold, false)
}

/// `slot_surface(w, h, m100, label, font_size, hot)` - ein Fach.
pub fn slot_surface(w: i32, h: i32, m100: i64, label: &str, font_size: i32, hot: bool) -> Surface {
    let key = format!("slot|{w}|{h}|{m100}|{label}|{font_size}|{hot}");
    cached(key, || {
        let mut col = slot_color(m100);
        if hot {
            col = mixt(col, (255, 255, 255), 0.28);
        }
        let mut s = pg::surface_alpha(w, h + 3);
        let rad = 2.max(w.min(h) / 4);
        let dark = mixt(col, (0, 0, 0), 0.45);
        draw::rect_r(&mut s, dark, (0, 3, w, h), 0, rad);
        let mut body = pg::surface_alpha(w, h);
        gradient(&mut body, (0, 0, w, h), mixt(col, (255, 255, 255), 0.30), mixt(col, (0, 0, 0), 0.12));
        let mut mask = pg::surface_alpha(w, h);
        draw::rect_r(&mut mask, (255, 255, 255, 255), (0, 0, w, h), 0, rad);
        body.bl_flags(&mask, (0, 0), BLEND_RGBA_MIN);
        s.bl(&body, (0, 0));
        draw::rect(&mut s, mixt(col, (255, 255, 255), 0.55), (rad, 1, 1.max(w - 2 * rad), 1), 0);
        if hot {
            draw::rect_r(&mut s, (255, 255, 255), (0, 0, w, h), 1, rad);
        }
        let fg: Rgb = if text_dark(col) { (34, 22, 12) } else { (255, 255, 255) };
        let fnt = fit_font(label, w - 2, font_size, true);
        let mut img = fnt.render(label, true, fg);
        if img.w() > w - 1 {
            // winzige Faecher: Text weich auf die Fachbreite verkleinern
            let f = (w - 1) as f64 / img.w() as f64;
            img = transform::smoothscale(&img, 1.max((img.w() as f64 * f) as i32), 1.max((img.h() as f64 * f) as i32));
        }
        s.blr(&img, img.get_rect().center_at((w / 2, h / 2)));
        s
    })
}

/// `pill(w, h, m100, label, font_size)` - Schild in der Fach-Farbe.
pub fn pill(w: i32, h: i32, m100: i64, label: &str, font_size: i32) -> Surface {
    let key = format!("pill|{w}|{h}|{m100}|{label}|{font_size}");
    cached(key, || {
        let col = slot_color(m100);
        let mut s = pg::surface_alpha(w, h);
        draw::rect_r(&mut s, mixt(col, (0, 0, 0), 0.40), (0, 1, w, h - 1), 0, h / 2);
        draw::rect_r(&mut s, col, (0, 0, w, h - 2), 0, h / 2);
        draw::rect(&mut s, mixt(col, (255, 255, 255), 0.45), (h / 2, 1, 1.max(w - h), 1), 0);
        let fg: Rgb = if text_dark(col) { (34, 22, 12) } else { (255, 255, 255) };
        let fnt = fit_font(label, w - 6, font_size, true);
        let img = fnt.render(label, true, fg);
        s.blr(&img, img.get_rect().center_at((w / 2, (h - 2) / 2)));
        s
    })
}

// ============================================================= Trichter + Lama

/// `funnel(w, h, accent)` - Einwurf-Trichter, Spitze unten in der Mitte.
pub fn funnel(w: i32, h: i32, accent: Color) -> Surface {
    let key = format!("funnel|{w}|{h}|{}", key_rgb(accent));
    cached(key, || {
        let ss = 3;
        let (bw, bh) = (w * ss, h * ss);
        let mut big = pg::surface_alpha(bw, bh);
        let mid = bw / 2;
        let neck = (ss * 4).max(bw * 2 / 9);
        let rim_h = (ss * 2).max(bh / 6);
        let cone = bh * 13 / 20;
        // Trichter-Koerper: Verlauf in Akzent-Dunkel, mit der Form ausgestanzt
        let body = [
            (0, rim_h / 2),
            (bw, rim_h / 2),
            (mid + neck, cone),
            (mid + neck, bh),
            (mid - neck, bh),
            (mid - neck, cone),
        ];
        let mut grad = pg::surface_alpha(bw, bh);
        gradient(&mut grad, (0, 0, bw, bh), mixt(accent, (40, 20, 30), 0.35), mixt(accent, (10, 6, 16), 0.75));
        let mut mask = pg::surface_alpha(bw, bh);
        draw::polygon(&mut mask, (255, 255, 255, 255), &body, 0);
        grad.bl_flags(&mask, (0, 0), BLEND_RGBA_MIN);
        big.bl(&grad, (0, 0));
        // dunkle Oeffnung + Glanzstreifen links
        let inner = [(bw / 7, rim_h), (bw - bw / 7, rim_h), (mid + neck * 2 / 3, cone), (mid - neck * 2 / 3, cone)];
        draw::polygon(&mut big, (18, 10, 24, 220), &inner, 0);
        draw::line(
            &mut big,
            pg::alpha(ui::mix(accent, (255, 255, 255), 0.5), 160),
            (bw / 9, rim_h + ss),
            (mid - neck + ss, cone),
            ss.max(bw / 30),
        );
        // Ring am Hals und goldener Rand oben
        draw::rect_r(&mut big, GOLD_D, (mid - neck - ss, cone, 2 * neck + 2 * ss, (ss * 2).max(bh / 9)), 0, ss);
        draw::rect_r(&mut big, GOLD_D, (0, 0, bw, rim_h), 0, rim_h / 2);
        draw::rect_r(&mut big, GOLD, (ss, 0, bw - 2 * ss, rim_h - ss), 0, rim_h / 2);
        draw::line(&mut big, GOLD_L, (rim_h, ss), (bw - rim_h, ss), 1.max(ss));
        transform::smoothscale(&big, w, h)
    })
}

/// `llama_bow(n, deg, scarf)` - Lama-Sprite um deg Grad gedreht (negativ =
/// nach vorn gebeugt).
pub fn llama_bow(n: i32, deg: i32, scarf: Color) -> Surface {
    let key = format!("llama|{n}|{deg}|{}", key_rgb(scarf));
    cached(key, || {
        if deg == 0 {
            return lcd::llama_sprite(n, "stand", scarf);
        }
        // doppelt so gross zeichnen, drehen, weich verkleinern (glatte Kanten)
        let mut big = pg::surface_alpha(n * 2, n * 2);
        lcd::draw_llama(&mut big, n * 2, "stand", scarf);
        let rot = transform::rotate(&big, deg as f64);
        transform::smoothscale(&rot, rot.w() / 2, rot.h() / 2)
    })
}

/// `llama_blit_pos(img, n, deg, foot_x, foot_y)` - linke obere Ecke, sodass
/// die Hufe (LLAMA_PIVOT) bei (foot_x, foot_y) stehen bleiben.
pub fn llama_blit_pos(img: &Surface, n: i32, deg: i32, foot_x: f64, foot_y: f64) -> (i32, i32) {
    let a = py::radians(deg as f64);
    let nf = n as f64;
    let vx = (LLAMA_PIVOT.0 - 0.5) * nf;
    let vy = (LLAMA_PIVOT.1 - 0.5) * nf;
    let rx = vx * a.cos() + vy * a.sin();
    let ry = -vx * a.sin() + vy * a.cos();
    let (rw, rh) = (img.w() as f64, img.h() as f64);
    (ri(foot_x - rx - rw / 2.0), ri(foot_y - ry - rh / 2.0))
}

/// `mouth_pos(n, deg, foot_x, foot_y)` - Bildschirmposition des Mauls.
pub fn mouth_pos(n: i32, deg: i32, foot_x: f64, foot_y: f64) -> (f64, f64) {
    let a = py::radians(deg as f64);
    let nf = n as f64;
    let vx = (LLAMA_MOUTH.0 - LLAMA_PIVOT.0) * nf;
    let vy = (LLAMA_MOUTH.1 - LLAMA_PIVOT.1) * nf;
    (foot_x + vx * a.cos() + vy * a.sin(), foot_y - vx * a.sin() + vy * a.cos())
}

// ============================================================= Piktogramme

/// `icon(name, size, col)` - auto, stats, fair, turbo, close, left, right,
/// check, cross.
pub fn icon(name: &str, size: i32, col: impl Col) -> Surface {
    let col = col.c();
    let key = format!("icon|{name}|{size}|{},{},{},{}", col.r, col.g, col.b, col.a);
    cached(key, || {
        let ss = 4;
        let n = size * ss;
        let nf = n as f64;
        let mut s = pg::surface_alpha(n, n);
        let lw = ss.max(n / 9);
        match name {
            "auto" => {
                // Kreispfeil mit Spitze + Play-Dreieck in der Mitte
                let (c, r) = ((n / 2, n / 2), n * 2 / 5);
                draw::arc(&mut s, col, (c.0 - r, c.1 - r, 2 * r, 2 * r), 0.55, 5.75, lw);
                let tip = (c.0 as f64 + r as f64 * 0.55f64.cos(), c.1 as f64 - r as f64 * 0.55f64.sin());
                draw::polygon(
                    &mut s,
                    col,
                    &[
                        (tip.0 - nf * 0.16, tip.1 - nf * 0.02),
                        (tip.0 + nf * 0.10, tip.1 - nf * 0.06),
                        (tip.0 - nf * 0.02, tip.1 + nf * 0.18),
                    ],
                    0,
                );
                draw::polygon(&mut s, col, &[(nf * 0.42, nf * 0.34), (nf * 0.66, nf * 0.5), (nf * 0.42, nf * 0.66)], 0);
            }
            "stats" => {
                let bw = n / 5;
                for (i, hh) in [0.38f64, 0.62, 0.86].iter().enumerate() {
                    let x = nf * 0.14 + i as f64 * (bw as f64 + nf * 0.09);
                    draw::rect_r(&mut s, col, (x as i32, (nf * (0.92 - hh)) as i32, bw, (nf * hh) as i32), 0, 1.max(ss));
                }
            }
            "fair" => {
                let pts = [
                    (nf * 0.5, nf * 0.06),
                    (nf * 0.88, nf * 0.2),
                    (nf * 0.82, nf * 0.6),
                    (nf * 0.5, nf * 0.95),
                    (nf * 0.18, nf * 0.6),
                    (nf * 0.12, nf * 0.2),
                ];
                draw::polygon(&mut s, col, &pts, lw);
                draw::lines(&mut s, col, false, &[(nf * 0.32, nf * 0.5), (nf * 0.46, nf * 0.64), (nf * 0.7, nf * 0.36)], lw);
            }
            "turbo" => {
                draw::polygon(
                    &mut s,
                    col,
                    &[
                        (nf * 0.58, nf * 0.04),
                        (nf * 0.2, nf * 0.56),
                        (nf * 0.48, nf * 0.56),
                        (nf * 0.38, nf * 0.96),
                        (nf * 0.82, nf * 0.4),
                        (nf * 0.53, nf * 0.4),
                    ],
                    0,
                );
            }
            "close" => {
                draw::line(&mut s, col, (nf * 0.22, nf * 0.22), (nf * 0.78, nf * 0.78), lw);
                draw::line(&mut s, col, (nf * 0.78, nf * 0.22), (nf * 0.22, nf * 0.78), lw);
            }
            "left" | "right" => {
                let mut pts = vec![(nf * 0.66, nf * 0.18), (nf * 0.30, nf * 0.5), (nf * 0.66, nf * 0.82)];
                if name == "right" {
                    pts = pts.iter().map(|&(x, y)| (nf - x, y)).collect();
                }
                draw::polygon(&mut s, col, &pts, 0);
            }
            "check" => {
                draw::lines(&mut s, col, false, &[(nf * 0.16, nf * 0.52), (nf * 0.4, nf * 0.76), (nf * 0.84, nf * 0.26)], ss.max(n / 7));
            }
            "cross" => {
                let w2 = ss.max(n / 7);
                draw::line(&mut s, col, (nf * 0.2, nf * 0.2), (nf * 0.8, nf * 0.8), w2);
                draw::line(&mut s, col, (nf * 0.8, nf * 0.2), (nf * 0.2, nf * 0.8), w2);
            }
            _ => {}
        }
        transform::smoothscale(&s, size, size)
    })
}

// ============================================================= Fairness-Vorschau

/// `path_preview(rows, path, w, h, m100)` - Mini-Brett mit Stiften und dem
/// Weg der Kugel (`m100 = None` -> Gold).
pub fn path_preview(rows: i32, path: &[i32], w: i32, h: i32, m100: Option<i64>) -> Surface {
    let key = format!("preview|{rows}|{:?}|{w}|{h}|{:?}", path, m100);
    cached(key, || {
        let ss = 2;
        let (bw, bh) = ((w * ss) as f64, (h * ss) as f64);
        let mut big = pg::surface_alpha(w * ss, h * ss);
        let n = 1.max(rows);
        let nf = n as f64;
        let g1 = bw / (nf + 2.4);
        let g2 = bh / (nf + 1.6);
        let gap = if g2 < g1 { g2 } else { g1 };
        let cx = bw / 2.0;
        let top = gap * 0.7;
        let dot = 1.max((gap * 0.12) as i32);
        for r in 0..n {
            for j in 0..r + 3 {
                let x = cx + (j as f64 - (r + 2) as f64 / 2.0) * gap;
                draw::circle(&mut big, (150, 156, 190), (x as i32, (top + r as f64 * gap) as i32), dot, 0);
            }
        }
        let mut pts: Vec<(f64, f64)> = vec![(cx, top - gap * 0.6)];
        let mut k: i32 = 0;
        for r in 0..n {
            pts.push((cx + (k as f64 - r as f64 / 2.0) * gap, top + r as f64 * gap - gap * 0.3));
            if (r as usize) < path.len() {
                k += path[r as usize];
            }
        }
        let sy = top + nf * gap - gap * 0.2;
        let sx = cx + (k as f64 - nf / 2.0) * gap;
        pts.push((sx, sy));
        let col = match m100 {
            Some(m) => slot_color(m),
            None => GOLD,
        };
        let bx = Rect::new(0, 0, 2.max((gap * 0.9) as i32), 2.max((gap * 0.7) as i32)).center_at((sx as i32, (sy + gap * 0.2) as i32));
        draw::rect_r(&mut big, col, bx, 0, 1.max(bx.h / 3));
        let ipts: Vec<(i32, i32)> = pts.iter().map(|&(x, y)| (x as i32, y as i32)).collect();
        draw::lines(&mut big, GOLD_L, false, &ipts, 2.max((gap * 0.16) as i32));
        draw::circle(&mut big, GOLD, (sx as i32, sy as i32), 2.max((gap * 0.28) as i32), 0);
        transform::smoothscale(&big, w, h)
    })
}
