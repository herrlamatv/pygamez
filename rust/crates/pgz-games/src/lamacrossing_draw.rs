//! lamacrossing_draw.rs
//! ====================
//! Zeichen-Bausteine von Lama Crossing - uebertragen aus
//! `games/lamacrossing_draw.py`: das Lama (stehen / springen / fallen) mit
//! Schal in der Akzentfarbe, Himmel und Ferne je Welt, Ufer, Brueckenseile,
//! Kacheln (Planke / Stein / Eisscholle), Risslinien und Welt-Piktogramme.
//!
//! Die Ausdruecke stehen in derselben Reihenfolge wie in Python, damit jede
//! Kommazahl gleich rundet.

use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::{PI, TAU};

use pgz_engine::prelude::*;

type Rgb = (i32, i32, i32);

const WOOL: Rgb = (250, 244, 230);
const WOOL_D: Rgb = (214, 200, 178);
const DARK: Rgb = (60, 44, 40);
const HOOF: Rgb = (82, 60, 52);
const PINK: Rgb = (240, 150, 160);

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
        if c.len() > 120 {
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

/// `_gradient(surface, rect, top, bottom)` - senkrechter Farbverlauf.
fn gradient(s: &mut Surface, r: (i32, i32, i32, i32), top: Rgb, bottom: Rgb) {
    let (x, y, w, h0) = r;
    let h = 1.max(h0 - 1);
    for yy in 0..h0 {
        s.fill_r(ui::mix(top, bottom, yy as f64 / h as f64), (x, y + yy, w, 1));
    }
}

// ============================================================= Lama

fn leg(s: &mut Surface, a: (f64, f64), b: (f64, f64), w: i32, col: Rgb) {
    draw::line(s, col, a, b, w);
    draw::circle(s, HOOF, b, w as f64 * 0.62, 0);
}

type Leg = ((f64, f64), (f64, f64));

fn draw_llama(s: &mut Surface, ni: i32, pose: &str, scarf: Color) {
    let n = ni as f64;
    let lw = 2.max((n * 0.055) as i32);
    let hip = (n * 0.34, n * 0.62);
    let sh = (n * 0.62, n * 0.62);
    let legs: [Leg; 4] = if pose == "hop" {
        [((-0.02, 0.0), (-0.12, 0.22)), ((0.04, 0.0), (-0.04, 0.24)), ((-0.02, 0.0), (0.10, 0.20)), ((0.04, 0.0), (0.16, 0.18))]
    } else if pose == "fall" {
        [((-0.02, 0.0), (-0.14, -0.10)), ((0.04, 0.0), (-0.08, 0.16)), ((-0.02, 0.0), (0.14, -0.08)), ((0.04, 0.0), (0.12, 0.16))]
    } else {
        [((-0.02, 0.0), (-0.03, 0.31)), ((0.04, 0.0), (0.05, 0.31)), ((-0.02, 0.0), (-0.01, 0.31)), ((0.04, 0.0), (0.06, 0.31))]
    };
    let leg_at = |s: &mut Surface, i: usize, col: Rgb| {
        let base = if i < 2 { hip } else { sh };
        let ((ax, ay), (bx, by)) = legs[i];
        leg(s, (base.0 + ax * n, base.1 + ay * n), (base.0 + bx * n, base.1 + by * n), lw, col);
    };
    // hintere (ferne) Beine dunkler, zuerst
    leg_at(s, 1, WOOL_D);
    leg_at(s, 3, WOOL_D);
    // Schwanz + Rumpf
    draw::circle(s, WOOL_D, (n * 0.215, n * 0.47), n * 0.055, 0);
    draw::ellipse(s, WOOL_D, (n * 0.22, n * 0.44, n * 0.54, n * 0.27), 0);
    draw::ellipse(s, WOOL, (n * 0.22, n * 0.42, n * 0.54, n * 0.27), 0);
    for i in 0..6 {
        draw::circle(s, WOOL, (n * (0.27 + i as f64 * 0.085), n * 0.44), n * 0.055, 0);
    }
    // Hals + Kopf
    draw::rect_r(s, WOOL, (n * 0.60, n * 0.17, n * 0.12, n * 0.38), 0, (n * 0.05) as i32);
    draw::polygon(s, WOOL, &[(n * 0.63, n * 0.12), (n * 0.655, n * 0.01), (n * 0.69, n * 0.11)], 0);
    draw::polygon(s, PINK, &[(n * 0.645, n * 0.11), (n * 0.66, n * 0.04), (n * 0.68, n * 0.11)], 0);
    draw::ellipse(s, WOOL, (n * 0.59, n * 0.09, n * 0.24, n * 0.17), 0);
    draw::ellipse(s, WOOL, (n * 0.72, n * 0.14, n * 0.17, n * 0.12), 0);
    for i in 0..3 {
        draw::circle(s, WOOL, (n * (0.62 + i as f64 * 0.045), n * 0.10), n * 0.035, 0);
    }
    // Gesicht
    let eye = (n * 0.765, n * 0.155);
    if pose == "fall" {
        let d = n * 0.022;
        draw::line(s, DARK, (eye.0 - d, eye.1 - d), (eye.0 + d, eye.1 + d), 1.max(lw / 2));
        draw::line(s, DARK, (eye.0 - d, eye.1 + d), (eye.0 + d, eye.1 - d), 1.max(lw / 2));
        draw::circle(s, DARK, (n * 0.86, n * 0.225), n * 0.022, 0);
    } else {
        draw::circle(s, DARK, eye, n * 0.024, 0);
        draw::circle(s, (255, 255, 255), (eye.0 - n * 0.007, eye.1 - n * 0.008), n * 0.008, 0);
        draw::arc(s, DARK, (n * 0.80, n * 0.19, n * 0.07, n * 0.05), PI * 1.15, PI * 1.85, 1.max(lw / 3));
    }
    draw::circle(s, PINK, (n * 0.79, n * 0.205), n * 0.022, 0);
    draw::circle(s, DARK, (n * 0.88, n * 0.18), n * 0.012, 0);
    // vordere Beine
    leg_at(s, 0, WOOL);
    leg_at(s, 2, WOOL);
    // Schal in der Akzentfarbe (Knoten + flatterndes Ende)
    let dark_scarf = ui::mix(scarf, (0, 0, 0), 0.35);
    draw::rect_r(s, scarf, (n * 0.585, n * 0.33, n * 0.15, n * 0.07), 0, (n * 0.03) as i32);
    let end = if pose == "hop" {
        [(n * 0.61, n * 0.36), (n * 0.66, n * 0.37), (n * 0.48, n * 0.42), (n * 0.46, n * 0.37)]
    } else {
        [(n * 0.61, n * 0.37), (n * 0.66, n * 0.38), (n * 0.56, n * 0.52), (n * 0.50, n * 0.49)]
    };
    draw::polygon(s, dark_scarf, &end, 0);
    draw::line(s, dark_scarf, (n * 0.60, n * 0.355), (n * 0.73, n * 0.355), 1.max(lw / 3));
}

/// `llama_sprite(size, pose, scarf)` - gecacht, Fuesse an der Unterkante Mitte.
pub fn llama_sprite(size: i32, pose: &str, scarf: Color) -> Surface {
    let key = format!("llama|{size}|{pose}|{},{},{}", scarf.r, scarf.g, scarf.b);
    cached(key, || {
        let mut big = pg::surface_alpha(size * 2, size * 2);
        draw_llama(&mut big, size * 2, pose, scarf);
        transform::smoothscale(&big, size, size)
    })
}

// ============================================================= Welten

fn sky(world: &str) -> (Rgb, Rgb) {
    match world {
        "bridge" => ((58, 82, 140), (238, 170, 120)),
        "river" => ((120, 186, 236), (222, 238, 250)),
        _ => ((150, 182, 214), (232, 240, 248)),
    }
}

fn far(world: &str) -> (Rgb, Rgb) {
    match world {
        "bridge" => ((176, 104, 84), (132, 74, 66)),
        "river" => ((70, 128, 84), (46, 98, 66)),
        _ => ((214, 228, 242), (168, 192, 220)),
    }
}

/// `_ridge(x, w, h, seed)` - weiche, periodische Huegellinie.
fn ridge(x: i32, w: i32, h: f64, seed: f64) -> f64 {
    let a = TAU * x as f64 / w as f64;
    h * (0.5 + 0.22 * (a * 2.0 + seed).sin() + 0.16 * (a * 5.0 + seed * 2.3).sin() + 0.08 * (a * 11.0 + seed * 0.7).sin())
}

/// `background(world, w, h, path_y)` - Himmel, Sonne und Untergrund.
pub fn background(world: &str, w: i32, h: i32, path_y: i32) -> Surface {
    let key = format!("bg|{world}|{w}|{h}|{path_y}");
    cached(key, || {
        let mut s = pg::surface(w, h);
        let (top, bot) = sky(world);
        gradient(&mut s, (0, 0, w, path_y), top, bot);
        if world == "bridge" {
            let sun = ((w as f64 * 0.78) as i32, (path_y as f64 * 0.42) as i32);
            for i in (1..=5).rev() {
                let r = (path_y as f64 * (0.06 + 0.035 * i as f64)) as i32;
                let mut glow = pg::surface_alpha(r * 2, r * 2);
                draw::circle(&mut glow, (255, 214, 150, 26), (r, r), r, 0);
                s.bl(&glow, (sun.0 - r, sun.1 - r));
            }
            draw::circle(&mut s, (255, 226, 170), sun, (path_y as f64 * 0.07) as i32, 0);
            gradient(&mut s, (0, path_y, w, h - path_y), (96, 54, 48), (22, 14, 22));
            // Felsnadeln in der Tiefe der Schlucht
            let (wf, hf, pf) = (w as f64, h as f64, path_y as f64);
            for i in 0..7 {
                let fi = i as f64;
                let x = wf * (0.07 + 0.145 * fi) + (fi * 2.3).sin() * wf * 0.03;
                let top = pf + (hf - pf) * (0.30 + 0.30 * (fi * 1.7).sin().abs());
                let bw = wf * (0.035 + 0.015 * (fi * 1.3).cos().abs());
                draw::polygon(
                    &mut s,
                    (66, 38, 40),
                    &[(x - bw, hf), (x - bw * 0.45, top + bw * 0.4), (x, top), (x + bw * 0.5, top + bw * 0.5), (x + bw, hf)],
                    0,
                );
                draw::line(&mut s, (112, 66, 58), (x - bw * 0.45, top + bw * 0.4), (x, top), 2);
            }
        } else if world == "river" {
            let d = (h as f64 * 0.10) as i32;
            gradient(&mut s, (0, path_y - d, w, h - path_y + d), (78, 156, 200), (26, 78, 124));
        } else {
            let d = (h as f64 * 0.08) as i32;
            gradient(&mut s, (0, path_y - d, w, h - path_y + d), (34, 78, 108), (10, 30, 52));
        }
        s
    })
}

/// `far_layer(world, w, h, path_y)` - kachelbare Ferne, Breite = w.
pub fn far_layer(world: &str, w: i32, h: i32, path_y: i32) -> Surface {
    let key = format!("far|{world}|{w}|{h}|{path_y}");
    cached(key, || {
        let mut s = pg::surface_alpha(w, path_y);
        let (c1, c2) = far(world);
        let base = path_y
            - if world == "river" {
                (h as f64 * 0.10) as i32
            } else if world == "ice" {
                (h as f64 * 0.08) as i32
            } else {
                0
            };
        for (layer, (col, amp, seed)) in [(c1, 0.42, 1.0), (c2, 0.26, 2.6)].into_iter().enumerate() {
            let mut pts: Vec<(f64, f64)> = vec![(0.0, base as f64)];
            let mut i = 0;
            while i < w + 1 {
                pts.push((i as f64, base as f64 - ridge(i, w, path_y as f64 * amp, seed)));
                i += 4;
            }
            pts.push((w as f64, base as f64));
            if world == "ice" && layer == 0 {
                // Schneekante: weisse Grate, die Flanke etwas tiefer
                draw::polygon(&mut s, (252, 253, 255), &pts, 0);
                pts = pts.iter().map(|&(x, y)| (x, y + if 0.0 < x && x < w as f64 { 5.0 } else { 0.0 })).collect();
            }
            draw::polygon(&mut s, col, &pts, 0);
        }
        if world == "river" {
            draw::rect(&mut s, (96, 160, 92), (0, base - 4, w, 6), 0);
        }
        s
    })
}

// ============================================================= Ufer + Seile

/// `draw_bank(s, world, x_edge, side, y, h, k)` - Start- bzw. Zielufer.
#[allow(clippy::too_many_arguments)]
pub fn draw_bank(s: &mut Surface, world: &str, x_edge: f64, side: i32, y: i32, h: i32, k: f64) {
    let w = s.w();
    let (x0, x1);
    if side < 0 {
        if x_edge < 0.0 {
            return;
        }
        x0 = -10;
        x1 = x_edge as i32;
    } else {
        if x_edge > w as f64 {
            return;
        }
        x0 = x_edge as i32;
        x1 = w + 10;
    }
    if world == "bridge" {
        let (rock, rock_d, top): (Rgb, Rgb, Rgb) = ((150, 92, 66), (104, 62, 48), (112, 168, 84));
        let lip = x_edge as i32;
        let pts = if side < 0 {
            [(x0, y), (lip, y), (lip - (18.0 * k) as i32, h), (x0, h)]
        } else {
            [(lip, y), (x1, y), (x1, h), (lip + (18.0 * k) as i32, h)]
        };
        draw::polygon(s, rock, &pts, 0);
        for i in 1..6 {
            let yy = y + ((h - y) as f64 * i as f64 / 6.0) as i32;
            let off = (18.0 * k * i as f64 / 6.0) as i32;
            let a = if side < 0 { (x0, yy) } else { (lip + off, yy) };
            let b = if side < 0 { (lip - off, yy) } else { (x1, yy) };
            draw::line(s, rock_d, a, b, 1.max((2.0 * k) as i32));
        }
        draw::rect(s, top, (x0, y - (6.0 * k) as i32, x1 - x0, (8.0 * k) as i32), 0);
    } else if world == "river" {
        let lip = x_edge as i32;
        let sand = (22.0 * k) as i32;
        draw::rect(s, (88, 156, 74), (x0, y - (8.0 * k) as i32, x1 - x0, h), 0);
        for i in 1..5 {
            let yy = y + ((h - y) as f64 * i as f64 / 5.0) as i32;
            draw::line(s, (78, 142, 66), (x0, yy), (x1, yy), 1.max((2.0 * k) as i32));
        }
        let sand_pts = if side < 0 {
            [(lip - sand, y - (8.0 * k) as i32), (lip, y - (4.0 * k) as i32), (lip + (6.0 * k) as i32, h), (lip - sand, h)]
        } else {
            [(lip + sand, y - (8.0 * k) as i32), (lip, y - (4.0 * k) as i32), (lip - (6.0 * k) as i32, h), (lip + sand, h)]
        };
        draw::polygon(s, (222, 200, 146), &sand_pts, 0);
        draw::line(s, (250, 252, 255), (lip, y - (4.0 * k) as i32), (lip + side * -((6.0 * k) as i32), h), 1.max((3.0 * k) as i32));
        draw::rect(s, (110, 184, 92), (x0, y - (10.0 * k) as i32, x1 - x0, (4.0 * k) as i32), 0);
    } else {
        draw::rect(s, (236, 244, 250), (x0, y - (4.0 * k) as i32, x1 - x0, h), 0);
        draw::rect(s, (196, 214, 232), (x0, y + (10.0 * k) as i32, x1 - x0, (4.0 * k) as i32), 0);
        draw::line(s, (255, 255, 255), (x0, y - (4.0 * k) as i32), (x1, y - (4.0 * k) as i32), 1.max((2.0 * k) as i32));
    }
}

/// `draw_ropes(s, points, k)` - Laufseil und Handlauf der Haengebruecke.
pub fn draw_ropes(s: &mut Surface, points: &[(f64, f64)], k: f64) {
    let rope: Rgb = (120, 82, 48);
    let lw = 1.max((2.0 * k) as i32);
    let rail = (46.0 * k) as i32 as f64;
    let w = s.w() as f64;
    for i in 0..points.len().saturating_sub(1) {
        let ((xa, ya), (xb, yb)) = (points[i], points[i + 1]);
        if xb < -40.0 || xa > w + 40.0 {
            continue;
        }
        let (mx, my) = ((xa + xb) / 2.0, (ya + yb) / 2.0 + 5.0 * k);
        draw::lines(s, rope, false, &[(xa, ya), (mx, my), (xb, yb)], lw);
        draw::lines(s, rope, false, &[(xa, ya - rail), (mx, my - rail + 2.0 * k), (xb, yb - rail)], lw);
        draw::line(s, rope, (xb, yb), (xb, yb - rail), 1.max(lw - 1));
    }
}

// ============================================================= Kacheln

/// `draw_tile(s, world, cx, y, tw, k, t, idx, alpha)` - ein Schritt.
#[allow(clippy::too_many_arguments)]
pub fn draw_tile(s: &mut Surface, world: &str, cx: f64, y: f64, tw: i32, k: f64, t: f64, idx: i32, alpha: i32) {
    if alpha <= 0 {
        return;
    }
    let twf = tw as f64;
    let fi = idx as f64;
    if alpha < 255 {
        let mut layer = pg::surface_alpha((twf * 1.4) as i32 + 8, (twf * 0.9) as i32 + 8);
        let lw2 = layer.w() as f64 / 2.0;
        draw_tile(&mut layer, world, lw2, (twf * 0.2) as i32 as f64, tw, k, t, idx, 255);
        layer.set_alpha(Some(alpha as u8));
        s.bl(&layer, (cx - layer.w() as f64 / 2.0, y - (twf * 0.2) as i32 as f64));
        return;
    }
    if world == "bridge" {
        let th = 5.max((13.0 * k) as i32);
        let r = Rect::new(0, 0, tw, th).midtop_at((cx as i32, y as i32));
        let rad = 1.max((3.0 * k) as i32);
        draw::rect_r(s, (96, 60, 34), r.move_(0, 1.max((3.0 * k) as i32)), 0, rad);
        draw::rect_r(s, (168, 112, 66), r, 0, rad);
        for j in [0.3, 0.68] {
            let yy = r.y + (r.h as f64 * j) as i32;
            draw::line(s, (132, 84, 48), (r.x + 4, yy), (r.right() - 4 - (6.0 * j * k) as i32, yy), 1);
        }
        for xx in [r.x + (5.0 * k) as i32, r.right() - (5.0 * k) as i32] {
            draw::circle(s, (70, 58, 54), (xx, r.y + r.h / 2), 1.max((1.6 * k) as i32), 0);
        }
    } else if world == "river" {
        let bob = (t * 2.2 + fi * 1.3).sin() * 1.5 * k;
        let h = (twf * 0.40) as i32;
        let hf = h as f64;
        let r = Rect::new(0, 0, tw, h).midtop_at((cx as i32, (y - hf * 0.22 + bob) as i32));
        let ring = Rect::new(0, 0, (twf * 1.25) as i32, (hf * 0.62) as i32).center_at((cx as i32, (y + hf * 0.42 + bob) as i32));
        draw::ellipse(s, (170, 214, 236), ring, 1.max((2.0 * k) as i32));
        draw::ellipse(s, (82, 88, 96), r.move_(0, (4.0 * k) as i32), 0);
        draw::ellipse(s, (128, 134, 142), r, 0);
        let top = r.inflate(-((twf * 0.28) as i32), -((hf * 0.45) as i32)).move_(-((twf * 0.05) as i32), -((hf * 0.12) as i32));
        draw::ellipse(s, (170, 176, 184), top, 0);
        draw::ellipse(
            s,
            (96, 140, 92),
            (r.x + (twf * 0.62) as i32, r.y + (hf * 0.10) as i32, (twf * 0.16) as i32, (hf * 0.20) as i32),
            0,
        );
    } else {
        let h = (twf * 0.34) as i32;
        let hf = h as f64;
        let side = 3.max((7.0 * k) as i32) as f64;
        let mut pts = Vec::with_capacity(7);
        for j in 0..7 {
            let jf = j as f64;
            let a = TAU * jf / 7.0 + fi * 0.7;
            let rad = 0.5 + 0.06 * (fi * 2.1 + jf * 1.7).sin();
            pts.push((cx + a.cos() * twf * 0.80 * rad * 1.1, y + hf * 0.5 + a.sin() * hf * rad));
        }
        let low: Vec<(f64, f64)> = pts.iter().map(|&(x, yy)| (x, yy + side + 3.0 * k)).collect();
        draw::polygon(s, (24, 52, 76), &low, 0);
        let mid: Vec<(f64, f64)> = pts.iter().map(|&(x, yy)| (x, yy + side)).collect();
        draw::polygon(s, (164, 202, 226), &mid, 0);
        draw::polygon(s, (228, 242, 250), &pts, 0);
        draw::line(s, (255, 255, 255), (cx - twf * 0.22, y + hf * 0.28), (cx - twf * 0.02, y + hf * 0.18), 1.max((2.0 * k) as i32));
    }
}

/// `crack_lines(cx, y, tw, idx, f)` - Risslinien einer Eisscholle.
pub fn crack_lines(cx: f64, y: f64, tw: i32, idx: i32, f: f64) -> Vec<Vec<(f64, f64)>> {
    let twf = tw as f64;
    let fi = idx as f64;
    let mut lines = Vec::new();
    for j in 0..4 {
        let jf = j as f64;
        let a = TAU * jf / 4.0 + fi * 0.9 + 0.4;
        let mut pts = vec![(cx, y + twf * 0.15)];
        for step in 1..5 {
            let sf = step as f64;
            if sf / 4.0 > f {
                break;
            }
            let r = twf * 0.16 * sf;
            let wob = (fi * 3.1 + jf * 2.0 + sf * 1.9).sin() * 0.45;
            pts.push((cx + (a + wob).cos() * r, y + twf * 0.15 + (a + wob).sin() * r * 0.42));
        }
        if pts.len() > 1 {
            lines.push(pts);
        }
    }
    lines
}

/// `world_icon(s, world, rect, col)` - Piktogramm der Welt.
pub fn world_icon(s: &mut Surface, world: &str, r: Rect, col: Color) {
    let cx = r.centerx() as f64;
    let cy = r.centery() as f64;
    let u = r.w as f64 / 10.0;
    let lw = 1.max((u * 0.8) as i32);
    let (rx, rr) = (r.x as f64, r.right() as f64);
    if world == "bridge" {
        draw::lines(s, col, false, &[(rx + u, cy - u * 2.0), (cx, cy - u * 0.8), (rr - u, cy - u * 2.0)], lw);
        draw::lines(s, col, false, &[(rx + u, cy + u), (cx, cy + u * 2.2), (rr - u, cy + u)], lw);
        for i in -2..3 {
            let fi = i as f64;
            let x = cx + fi * u * 1.6;
            let dy = u * (2.2 - 0.3 * fi * fi) * 0.5;
            draw::line(s, col, (x, cy + u + dy - u * 0.6), (x, cy + u + dy + u * 0.4), lw);
        }
    } else if world == "river" {
        for i in 0..3 {
            let yy = cy + u * (1.0 + i as f64 * 1.3);
            draw::lines(
                s,
                col,
                false,
                &[(rx + u, yy), (cx - u * 1.5, yy - u * 0.6), (cx + u * 1.5, yy + u * 0.3), (rr - u, yy - u * 0.4)],
                1.max(lw - 1),
            );
        }
        draw::ellipse(s, col, (cx - u * 2.4, cy - u * 2.2, u * 4.8, u * 2.6), 0);
    } else {
        draw::polygon(
            s,
            col,
            &[
                (rx + u, cy),
                (cx - u * 1.5, cy - u * 2.0),
                (cx + u * 2.5, cy - u * 1.6),
                (rr - u, cy + u * 0.4),
                (cx + u * 0.5, cy + u * 1.8),
                (cx - u * 2.6, cy + u * 1.4),
            ],
            lw,
        );
        draw::lines(
            s,
            col,
            false,
            &[(cx - u * 0.5, cy - u * 1.6), (cx + u * 0.2, cy - u * 0.2), (cx - u * 0.6, cy + u * 0.6), (cx + u * 0.4, cy + u * 1.6)],
            1.max(lw - 1),
        );
    }
}
