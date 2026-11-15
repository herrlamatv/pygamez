//! lamacrossing_draw.rs
//! ====================
//! Zeichen-Bausteine von Lama Crossing - uebertragen aus
//! `games/lamacrossing_draw.py`: das Lama (stehen / springen / fallen) mit
//! Schal in der Akzentfarbe, das plattgedrueckte Lama, Himmel und Ferne je
//! Welt bzw. Asphalt von oben (Strasse), Ufer bzw. Gehweg, Fahrbahn-
//! markierungen, Brueckenseile, Kacheln (Planke / Stein / Eisscholle /
//! Zebrastreifen), Risslinien, Hindernisse (Autos + Lieferwagen, Felsbrocken,
//! Treibholz, Eisblock), Absperrung, Reifenspuren und Welt-Piktogramme.
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

/// `_hash(i)` - Ganzzahl-Streuwert 0..2^32-1 (deterministisch).
///
/// Python rechnet unbeschraenkt; beide Produkte werden sofort modulo 2^32
/// genommen. Das erste rechnet in i128 (mit Pythons `%` fuer negative i),
/// das zweite passt in u64 (beide Faktoren < 2^32).
fn hash(i: i64) -> u64 {
    let v = (i as i128 * 2_654_435_761 + 40_503).rem_euclid(4_294_967_296) as u64;
    let v = (v ^ (v >> 15)).wrapping_mul(2_246_822_519) % 4_294_967_296;
    v ^ (v >> 13)
}

/// `_rrect(s, col, x, y, w, h, r)` - abgerundetes Rechteck, jede Zahl per `int()`.
fn rrect(s: &mut Surface, col: impl Col, x: f64, y: f64, w: f64, h: f64, r: f64) {
    draw::rect_r(s, col, (x as i32, y as i32, 1.max(w as i32), 1.max(h as i32)), 0, 0.max(r as i32));
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

/// `flat_llama(size, scarf)` - plattgedruecktes Lama (Pose "fall" gestaucht),
/// gecacht. Flaeche int(size * 1.6) x max(4, int(size * 0.34)) mit dunklem
/// Abdruck darunter.
pub fn flat_llama(size: i32, scarf: Color) -> Surface {
    let key = format!("flat|{size}|{},{},{}", scarf.r, scarf.g, scarf.b);
    cached(key, || {
        let sf = size as f64;
        let (fw, fh) = (4.max((sf * 1.6) as i32), 4.max((sf * 0.34) as i32));
        let squashed = transform::smoothscale(&llama_sprite(size, "fall", scarf), fw, fh);
        let mut flat = pg::surface_alpha(fw, fh);
        let (fwf, fhf) = (fw as f64, fh as f64);
        // dunkler Abdruck unter dem Lama
        draw::ellipse(&mut flat, (24, 20, 26, 50), ((fwf * 0.13) as i32, 0, (fwf * 0.82) as i32, fh), 0);
        draw::ellipse(
            &mut flat,
            (24, 20, 26, 110),
            ((fwf * 0.17) as i32, (fhf * 0.12) as i32, (fwf * 0.74) as i32, 1.max((fhf * 0.80) as i32)),
            0,
        );
        flat.bl(&squashed, (0, 0));
        // Schalzipfel, flach neben dem Hals ausgebreitet
        let dark = ui::mix(scarf, (0, 0, 0), 0.35);
        draw::polygon(
            &mut flat,
            dark,
            &[(fwf * 0.57, fhf * 0.42), (fwf * 0.66, fhf * 0.42), (fwf * 0.56, fhf * 0.98), (fwf * 0.47, fhf * 0.90)],
            0,
        );
        draw::polygon(
            &mut flat,
            scarf,
            &[(fwf * 0.58, fhf * 0.42), (fwf * 0.645, fhf * 0.42), (fwf * 0.555, fhf * 0.86), (fwf * 0.495, fhf * 0.82)],
            0,
        );
        flat
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

const ASPHALT: (Rgb, Rgb, Rgb) = ((52, 54, 62), (66, 68, 76), (44, 46, 54)); // oben, Pfad, unten
const ASPHALT_GRAIN: [Rgb; 3] = [(80, 82, 90), (38, 40, 46), (58, 60, 68)];
const LANE: Rgb = (230, 230, 222);
const ZEBRA: Rgb = (238, 238, 230);
const ZEBRA_D: Rgb = (196, 196, 190);
const CURB: Rgb = (208, 208, 202);
const CURB_L: Rgb = (236, 236, 230);
const CURB_D: Rgb = (160, 160, 156);
const GUTTER: Rgb = (34, 35, 41);
const PAVE: Rgb = (184, 176, 164);
const PAVE_2: Rgb = (172, 164, 152);
const PAVE_D: Rgb = (150, 142, 132);
const HEDGE: Rgb = (58, 128, 62);
const HEDGE_D: Rgb = (38, 92, 46);
const HEDGE_L: Rgb = (92, 162, 82);
const GRASS: Rgb = (104, 170, 88);
const GRASS_D: Rgb = (94, 158, 80);

/// `_road_bg(w, h, path_y)` - Strasse von oben: dunkler Asphalt mit leichtem
/// Verlauf und feiner Koernung.
fn road_bg(w: i32, h: i32, path_y: i32) -> Surface {
    let mut s = pg::surface(w, h);
    let (top, mid, bot) = ASPHALT;
    let py_ = 1.max((h - 1).min(path_y)); // Python: py
    gradient(&mut s, (0, 0, w, py_), top, mid);
    gradient(&mut s, (0, py_, w, h - py_), mid, bot);
    let dot = if w >= 1200 { 2 } else { 1 };
    let (wu, hu) = (w as u64, h as u64);
    for i in 0..py::fdiv(w as i64 * h as i64, 650) {
        let v = hash(i);
        s.fill_r(ASPHALT_GRAIN[((v >> 24) % 3) as usize], ((v % wu) as i32, ((v / wu) % hu) as i32, dot, dot));
    }
    s
}

/// `background(world, w, h, path_y)` - Himmel, Sonne und Untergrund
/// (Strasse: Asphalt von oben).
pub fn background(world: &str, w: i32, h: i32, path_y: i32) -> Surface {
    let key = format!("bg|{world}|{w}|{h}|{path_y}");
    cached(key, || {
        if world == "road" {
            return road_bg(w, h, path_y);
        }
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

/// `far_layer(world, w, h, path_y)` - kachelbare Ferne, Breite = w
/// (Strasse: leer und durchsichtig).
pub fn far_layer(world: &str, w: i32, h: i32, path_y: i32) -> Surface {
    let key = format!("far|{world}|{w}|{h}|{path_y}");
    cached(key, || {
        let mut s = pg::surface_alpha(w, path_y);
        if !matches!(world, "bridge" | "river" | "ice") {
            return s; // Strasse: keine Ferne (bleibt durchsichtig)
        }
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

/// `_band(s, a, b, y0, hgt, col)` - senkrechter Streifen zwischen x=a und
/// x=b (Reihenfolge egal).
fn band(s: &mut Surface, a: f64, b: f64, y0: i32, hgt: i32, col: Rgb) {
    let xa = a.min(b).max(-10.0);
    let xb = a.max(b).min((s.w() + 10) as f64);
    if xb > xa && hgt > 0 {
        s.fill_r(col, (xa as i32, y0, (xb - xa) as i32, hgt));
    }
}

/// `_road_bank(s, lip, side, y, h, k, top)` - Gehweg der Strasse: Bordstein
/// an lip, Pflaster, Hecke, Rasen nach aussen.
///
/// side=-1: alles links von lip, +1: alles rechts davon. Zur Fahrbahn hin
/// kommen nur Rinne und durchgezogene Randlinie dazu.
fn road_bank(s: &mut Surface, lip: i32, side: i32, y: i32, h: i32, k: f64, top: i32) {
    let w = s.w();
    let hh = h - top;
    if hh <= 0 {
        return;
    }
    let sw = (44.max((118.0 * k) as i32) as f64 * 1.15) as i32; // Gehweg inkl. Bordstein
    let cw = 2.max((4.0 * k) as i32); // Bordstein
    let hw = 6.max((18.0 * k) as i32); // Hecke
    let ps = 8.max((22.0 * k) as i32); // Pflasterstein
    let jw = 1.max((1.2 * k) as i32); // Fugen
    let far = if side < 0 { -10 } else { w + 10 };
    let up = 0.max(y - top); // Raster haengt am Pfad (y)
    let span = h - y.min(top); // Hoehe ab der ersten Rasterzeile
    // Rasen mit Maehstreifen und ein paar Bluemchen
    let g0 = lip + side * (sw + hw);
    band(s, g0 as f64, far as f64, top, hh, GRASS);
    let bw = 6.max((30.0 * k) as i32);
    let fs = 10.max((64.0 * k) as i32);
    let m0 = -(py::fdiv32(up, fs) + 1);
    let rows = py::fdiv32(span, fs) + 3;
    let fr = 1.max((2.0 * k) as i32);
    let near = if side < 0 { w + 10 } else { -10 };
    let j0 = 0.max(py::fdiv32((near - g0) * side, bw) - 1); // Streifen vor dem Bild ueberspringen
    for j in j0..j0 + py::fdiv32(w + 20, bw) + 4 {
        let a = g0 + side * j * bw;
        if (a - far) * side > 0 {
            break;
        }
        if py::pmod32(j, 2) == 1 {
            band(s, a as f64, (a + side * bw) as f64, top, hh, GRASS_D);
        }
        for i in 0..rows {
            if py::pmod32(j * 5 + i * 3, 7) != 0 {
                continue;
            }
            let fx = (a as f64 + (side * bw) as f64 * 0.5 + (py::pmod32(j * 13 + i * 7, 9) - 4) as f64 * k) as i32;
            let fy = ((y + (m0 + i) * fs) as f64 + ((py::pmod32(j * 7 + i * 11, 11) - 5) * 2) as f64 * k) as i32;
            if fy - fr * 2 < top || fy + fr * 2 > h {
                continue;
            }
            let petal: Rgb = if py::pmod32(j + i, 2) == 0 { (250, 250, 240) } else { (246, 190, 210) };
            for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                draw::circle(s, petal, (fx + dx * fr, fy + dy * fr), fr, 0);
            }
            draw::circle(s, (246, 196, 64), (fx, fy), 1.max(fr - 1), 0);
        }
    }
    // Hecke: Band + runde Buesche
    band(s, (lip + side * sw) as f64, g0 as f64, top, hh, HEDGE_D);
    let step = 4.max((hw as f64 * 0.75) as i32);
    let rad = hw as f64 * 0.56;
    let hx = lip as f64 + side as f64 * (sw as f64 + hw as f64 * 0.5);
    let m0 = -(py::fdiv32(up, step) + 1);
    for i in 0..py::fdiv32(span, step) + 3 {
        let cy = y + (m0 + i) * step;
        let cx = hx + if py::pmod32(i, 2) == 0 { hw as f64 * 0.08 } else { (-hw) as f64 * 0.08 };
        let cyf = cy as f64;
        if cyf - rad < top as f64 || cyf + rad > h as f64 || cx < (-hw) as f64 || cx > (w + hw) as f64 {
            continue;
        }
        draw::circle(s, HEDGE, (cx as i32, cy), rad as i32, 0);
        draw::circle(s, HEDGE_L, ((cx - rad * 0.28) as i32, (cyf - rad * 0.30) as i32), 1.max((rad * 0.40) as i32), 0);
    }
    // Gehweg: versetzte Pflastersteine
    band(s, (lip + side * cw) as f64, (lip + side * sw) as f64, top, hh, PAVE);
    let nc = 2.max(((sw - cw) as f64 / ps as f64 + 0.5) as i32);
    let colw = (sw - cw) as f64 / nc as f64;
    let m0 = -(py::fdiv32(up, ps) + 2);
    for j in 0..nc {
        let xa = lip as f64 + side as f64 * (cw as f64 + j as f64 * colw);
        let xb = lip as f64 + side as f64 * (cw as f64 + (j + 1) as f64 * colw);
        let (xl, xr) = (xa.min(xb) as i32, xa.max(xb) as i32);
        if xr < -10 || xl > w + 10 {
            continue;
        }
        let off = if py::pmod32(j, 2) == 1 { py::fdiv32(ps, 2) } else { 0 };
        for i in 0..py::fdiv32(span, ps) + 4 {
            let yy = y + (m0 + i) * ps + off;
            if yy >= h {
                break;
            }
            if yy + ps <= top {
                continue;
            }
            if py::pmod32(j * 5 + i * 3, 7) == 0 {
                let y0 = top.max(yy);
                s.fill_r(PAVE_2, (xl, y0, xr - xl, h.min(yy + ps) - y0));
            }
            if yy >= top {
                s.fill_r(PAVE_D, (xl, yy, xr - xl, jw));
            }
        }
        if j > 0 {
            band(s, xa, xa + (side * jw) as f64, top, hh, PAVE_D);
        }
    }
    // Bordstein mit Fugen, Rinne und Randlinie auf der Fahrbahn
    band(s, lip as f64, (lip + side * cw) as f64, top, hh, CURB);
    band(s, lip as f64, (lip + side * 1.max(py::fdiv32(cw, 3))) as f64, top, hh, CURB_L);
    let m0 = -(py::fdiv32(up, ps * 3) + 1);
    for i in 0..py::fdiv32(span, ps * 3) + 3 {
        let yy = y + (m0 + i) * ps * 3;
        if top <= yy && yy < h {
            band(s, lip as f64, (lip + side * cw) as f64, yy, 1, CURB_D);
        }
    }
    band(s, lip as f64, (lip - side * 1.max((2.0 * k) as i32)) as f64, top, hh, GUTTER);
    let ex = lip - side * (9.0 * k) as i32;
    band(s, ex as f64, (ex - side * 2.max((3.0 * k) as i32)) as f64, top, hh, LANE);
}

/// `draw_bank(s, world, x_edge, side, y, h, k, top)` - Start- bzw. Zielufer.
///
/// top: Oberkante (nur Strasse; die uebrigen Welten beginnen kurz ueber y).
#[allow(clippy::too_many_arguments)]
pub fn draw_bank(s: &mut Surface, world: &str, x_edge: f64, side: i32, y: i32, h: i32, k: f64, top: i32) {
    let w = s.w();
    if world == "road" {
        // Rinne und Randlinie reichen bis ~12k auf die Fahrbahn
        let m = (14.0 * k) as i32 + 4;
        if (side < 0 && x_edge < (-m) as f64) || (side >= 0 && x_edge > (w + m) as f64) {
            return;
        }
        road_bank(s, x_edge as i32, if side < 0 { -1 } else { 1 }, y, h, k, top);
        return;
    }
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

/// `draw_lanes(s, xs, top, bottom, k, t)` - gestrichelte Spur-Trennlinien der
/// Strasse (statisch, t ungenutzt). xs: Bild-x der Trennlinien.
pub fn draw_lanes(s: &mut Surface, xs: &[f64], top: i32, bottom: i32, k: f64, _t: f64) {
    let lw = 2.max((3.0 * k) as i32);
    let dash = 6.max((26.0 * k) as i32);
    let gap = 4.max((18.0 * k) as i32);
    let w = s.w();
    for &x in xs {
        if x < (-lw) as f64 || x > (w + lw) as f64 {
            continue;
        }
        let xi = x as i32 - py::fdiv32(lw, 2);
        let mut yy = top;
        while yy < bottom {
            s.fill_r(LANE, (xi, yy, lw, dash.min(bottom - yy)));
            yy += dash + gap;
        }
    }
}

// ============================================================= Kacheln

/// `draw_tile(s, world, cx, y, tw, k, t, idx, alpha)` - ein Schritt.
///
/// Strasse: Zebrastreifen-Stueck von y - tw*0.38 bis y + tw*0.17 (Fuesse bei y).
#[allow(clippy::too_many_arguments)]
pub fn draw_tile(s: &mut Surface, world: &str, cx: f64, y: f64, tw: i32, k: f64, t: f64, idx: i32, alpha: i32) {
    if alpha <= 0 {
        return;
    }
    let twf = tw as f64;
    let fi = idx as f64;
    if alpha < 255 {
        let oy = if world == "road" { (twf * 0.45) as i32 } else { (twf * 0.2) as i32 };
        let mut layer = pg::surface_alpha((twf * 1.4) as i32 + 8, (twf * 0.9) as i32 + 8);
        let lw2 = layer.w() as f64 / 2.0;
        draw_tile(&mut layer, world, lw2, oy as f64, tw, k, t, idx, 255);
        layer.set_alpha(Some(alpha as u8));
        s.bl(&layer, (cx - layer.w() as f64 / 2.0, y - oy as f64));
        return;
    }
    if world == "road" {
        // Zebrastreifen: weisse Balken parallel zum Verkehr (senkrecht)
        let nb = 4;
        let u = twf / (2 * nb - 1) as f64;
        let zh = 4.max((twf * 0.55) as i32);
        let zy = (y - twf * 0.38) as i32;
        let bw = 2.max(u as i32);
        let rad = 1.max((u * 0.25) as i32);
        for j in 0..nb {
            let bx = (cx - twf / 2.0 + (j * 2) as f64 * u) as i32;
            draw::rect_r(s, ZEBRA_D, (bx, zy, bw, zh), 0, rad);
            draw::rect_r(s, ZEBRA, (bx, zy, bw, zh - 1.max((2.0 * k) as i32)), 0, rad);
        }
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

// ============================================================= Hindernisse

const CAR_COLS: [Rgb; 3] = [(216, 66, 58), (60, 124, 214), (242, 190, 50)];
const TRUCK_COL: Rgb = (36, 150, 140);
const GLASS: Rgb = (56, 72, 98);
const GLASS_L: Rgb = (150, 186, 214);
const TIRE: Rgb = (32, 32, 38);
const HEAD: Rgb = (255, 248, 206);
const TAIL: Rgb = (196, 26, 40);

/// `obstacle_len(world, kind, size)` - senkrechte Laenge eines Hindernisses
/// in Pixeln (fuer Treffer-Test/Halten).
pub fn obstacle_len(world: &str, kind: i32, size: i32) -> i32 {
    let sf = size as f64;
    match world {
        "road" => (sf * if kind == 3 { 1.45 } else { 0.86 }) as i32,
        "bridge" => (sf * 0.42) as i32,
        "river" => (sf * 0.30) as i32,
        _ => (sf * 0.48) as i32,
    }
}

/// `_car_shadow(b, x0, y0, w, ln, r)` - weicher Schatten nach rechts unten
/// (3 Stufen, ersetzt Alpha).
fn car_shadow(b: &mut Surface, x0: f64, y0: f64, w: f64, ln: f64, r: f64) {
    let sh = 2.max((w * 0.08) as i32);
    let shf = sh as f64;
    for (i, a) in [(2, 28), (1, 52), (0, 78)] {
        let g = (sh * i) as f64 * 0.5;
        rrect(b, (0, 0, 0, a), x0 + shf - g, y0 + shf * 1.4 - g, w + 2.0 * g, ln + 2.0 * g, r + g);
    }
}

/// `_draw_car(b, cx, cy, w, ln, kind)` - Auto von oben, Front unten (faehrt
/// nach unten); w/ln in Feld-Pixeln.
fn draw_car(b: &mut Surface, cx: f64, cy: f64, w: f64, ln: f64, kind: i32) {
    let col = CAR_COLS[py::pmod32(kind, 3) as usize];
    let dark = ui::mix(col, (0, 0, 0), 0.30);
    let light = ui::mix(col, (255, 255, 255), 0.32);
    let roof = ui::mix(col, (255, 255, 255), 0.14);
    let (x0, y0) = (cx - w / 2.0, cy - ln / 2.0);
    let v = |u: f64| y0 + ln * u;
    car_shadow(b, x0, y0, w, ln, w * 0.30);
    // Raeder (schauen seitlich etwas heraus)
    let (rw, rh) = (w * 0.14, ln * 0.17);
    for u in [0.19, 0.79] {
        for sx in [-1.0, 1.0] {
            rrect(b, TIRE, cx + sx * (w / 2.0 - rw * 0.30) - rw / 2.0, v(u) - rh / 2.0, rw, rh, rw * 0.35);
        }
    }
    // Karosserie: dunkle Flanken, helle Oberseite
    rrect(b, dark, x0, y0, w, ln, w * 0.30);
    rrect(b, col, x0 + w * 0.07, y0 + ln * 0.012, w * 0.86, ln * 0.976, w * 0.26);
    // Glanz auf Motorhaube und Kofferraum
    rrect(b, light, x0 + w * 0.15, v(0.76), w * 0.07, ln * 0.17, w * 0.035);
    rrect(b, light, x0 + w * 0.15, v(0.05), w * 0.07, ln * 0.10, w * 0.035);
    // Heckscheibe, Dach, Seitenfenster, Frontscheibe
    draw::polygon(
        b,
        GLASS,
        &[(cx - w * 0.27, v(0.19)), (cx + w * 0.27, v(0.19)), (cx + w * 0.34, v(0.295)), (cx - w * 0.34, v(0.295))],
        0,
    );
    for sx in [-1.0, 1.0] {
        draw::polygon(
            b,
            GLASS,
            &[
                (cx + sx * w * 0.345, v(0.30)),
                (cx + sx * w * 0.405, v(0.32)),
                (cx + sx * w * 0.405, v(0.57)),
                (cx + sx * w * 0.345, v(0.585)),
            ],
            0,
        );
    }
    rrect(b, roof, cx - w * 0.34, v(0.285), w * 0.68, ln * 0.31, w * 0.10);
    draw::polygon(
        b,
        GLASS,
        &[(cx - w * 0.34, v(0.585)), (cx + w * 0.34, v(0.585)), (cx + w * 0.40, v(0.715)), (cx - w * 0.40, v(0.715))],
        0,
    );
    let lw = 1.max((w * 0.05) as i32);
    draw::line(b, GLASS_L, (cx - w * 0.24, v(0.70)), (cx - w * 0.06, v(0.60)), lw);
    draw::line(b, GLASS_L, (cx - w * 0.04, v(0.70)), (cx + w * 0.04, v(0.645)), lw);
    draw::line(b, GLASS_L, (cx + w * 0.10, v(0.20)), (cx + w * 0.20, v(0.28)), lw);
    match py::pmod32(kind, 3) {
        0 => {
            // Rennstreifen ueber Motorhaube, Dach und Kofferraum
            for (u0, u1) in [(0.015, 0.185), (0.30, 0.575), (0.725, 0.985)] {
                for sx in [-1.0, 1.0] {
                    rrect(b, (248, 246, 240), cx + sx * w * 0.075 - w * 0.035, v(u0), w * 0.07, ln * (u1 - u0), 0.0);
                }
            }
        }
        1 => {
            // Schiebedach
            rrect(b, GLASS, cx - w * 0.17, v(0.35), w * 0.34, ln * 0.13, w * 0.06);
            draw::line(b, GLASS_L, (cx - w * 0.09, v(0.465)), (cx + w * 0.02, v(0.37)), lw);
        }
        _ => {
            // Dachtraeger
            for u in [0.36, 0.51] {
                rrect(b, (66, 66, 74), cx - w * 0.31, v(u), w * 0.62, ln * 0.025, w * 0.02);
            }
            for sx in [-1.0, 1.0] {
                rrect(b, (66, 66, 74), cx + sx * w * 0.24 - w * 0.025, v(0.34), w * 0.05, ln * 0.20, 0.0);
            }
        }
    }
    // Aussenspiegel
    for sx in [-1.0, 1.0] {
        draw::ellipse(
            b,
            dark,
            ((cx + sx * w * 0.50 - w * 0.07) as i32, v(0.585) as i32, (w * 0.14) as i32, 2.max((ln * 0.045) as i32)),
            0,
        );
    }
    // Scheinwerfer vorn (unten), Rueckleuchten hinten (oben)
    for sx in [-1.0, 1.0] {
        rrect(b, HEAD, cx + sx * w * 0.27 - w * 0.10, v(0.962), w * 0.20, ln * 0.026, w * 0.04);
        rrect(b, TAIL, cx + sx * w * 0.28 - w * 0.08, v(0.010), w * 0.16, ln * 0.024, w * 0.03);
    }
}

/// `_draw_truck(b, cx, cy, w, ln)` - Lieferwagen von oben: Kofferaufbau
/// hinten, Fahrerhaus vorn (unten).
fn draw_truck(b: &mut Surface, cx: f64, cy: f64, w: f64, ln: f64) {
    let col = TRUCK_COL;
    let dark = ui::mix(col, (0, 0, 0), 0.30);
    let roof = ui::mix(col, (255, 255, 255), 0.16);
    let (bx, box_d, box_r): (Rgb, Rgb, Rgb) = ((238, 236, 228), (190, 188, 180), (216, 214, 206));
    let (x0, y0) = (cx - w / 2.0, cy - ln / 2.0);
    let v = |u: f64| y0 + ln * u;
    car_shadow(b, x0, y0, w, ln, w * 0.12);
    let (rw, rh) = (w * 0.13, ln * 0.09);
    for u in [0.12, 0.23, 0.86] {
        for sx in [-1.0, 1.0] {
            rrect(b, TIRE, cx + sx * (w / 2.0 - rw * 0.30) - rw / 2.0, v(u) - rh / 2.0, rw, rh, rw * 0.35);
        }
    }
    // Kofferaufbau mit Rippen und Zierstreifen
    rrect(b, box_d, x0, y0, w, ln * 0.71, w * 0.08);
    rrect(b, bx, x0 + w * 0.05, y0 + ln * 0.006, w * 0.90, ln * 0.70, w * 0.06);
    let lw = 1.max((w * 0.025) as i32);
    for i in 1..11 {
        let yy = v(0.065 * i as f64);
        draw::line(b, box_r, (x0 + w * 0.10, yy), (x0 + w * 0.90, yy), lw);
    }
    for sx in [-1.0, 1.0] {
        rrect(b, col, cx + sx * w * 0.36 - w * 0.04, v(0.03), w * 0.08, ln * 0.65, 0.0);
    }
    draw::line(b, box_d, (cx, v(0.006)), (cx, v(0.05)), lw);
    for sx in [-1.0, 1.0] {
        rrect(b, TAIL, cx + sx * w * 0.36 - w * 0.07, v(0.004), w * 0.14, ln * 0.016, w * 0.02);
    }
    // Kupplung + Fahrerhaus
    rrect(b, (54, 54, 62), cx - w * 0.24, v(0.70), w * 0.48, ln * 0.05, 0.0);
    rrect(b, dark, x0 + w * 0.03, v(0.735), w * 0.94, ln * 0.265, w * 0.24);
    rrect(b, col, x0 + w * 0.09, v(0.742), w * 0.82, ln * 0.252, w * 0.20);
    rrect(b, roof, cx - w * 0.33, v(0.752), w * 0.66, ln * 0.12, w * 0.08);
    for sx in [-1.0, 1.0] {
        draw::circle(b, (255, 184, 56), ((cx + sx * w * 0.16) as i32, v(0.77) as i32), 1.max((w * 0.04) as i32), 0);
    }
    draw::polygon(
        b,
        GLASS,
        &[(cx - w * 0.33, v(0.876)), (cx + w * 0.33, v(0.876)), (cx + w * 0.38, v(0.935)), (cx - w * 0.38, v(0.935))],
        0,
    );
    draw::line(b, GLASS_L, (cx - w * 0.22, v(0.93)), (cx - w * 0.10, v(0.88)), 1.max((w * 0.04) as i32));
    for sx in [-1.0, 1.0] {
        draw::ellipse(
            b,
            dark,
            ((cx + sx * w * 0.52 - w * 0.08) as i32, v(0.86) as i32, (w * 0.16) as i32, 2.max((ln * 0.025) as i32)),
            0,
        );
        rrect(b, HEAD, cx + sx * w * 0.28 - w * 0.09, v(0.976), w * 0.18, ln * 0.016, w * 0.03);
    }
}

/// `_car_sprite(size, kind)` - Auto/Lieferwagen als gecachte Flaeche,
/// 2-fach ueberabgetastet.
fn car_sprite(size: i32, kind: i32) -> Surface {
    cached(format!("car|{size}|{kind}"), || {
        let truck = kind == 3;
        let bw = 4.max((size as f64 * if truck { 0.58 } else { 0.52 }) as i32);
        let ln = 4.max(obstacle_len("road", kind, size));
        let m = 3.max((size as f64 * 0.10) as i32);
        let (fw, fh) = (bw + 2 * m, ln + 2 * m);
        let mut big = pg::surface_alpha(fw * 2, fh * 2);
        if truck {
            draw_truck(&mut big, fw as f64, fh as f64, (bw * 2) as f64, (ln * 2) as f64);
        } else {
            draw_car(&mut big, fw as f64, fh as f64, (bw * 2) as f64, (ln * 2) as f64, kind);
        }
        transform::smoothscale(&big, fw, fh)
    })
}

/// `_boulder(s, x, y, size, kind)` - fallender Felsbrocken (Bruecke),
/// Durchmesser = obstacle_len.
fn boulder(s: &mut Surface, x: i32, y: i32, size: i32, kind: i32) {
    let r = obstacle_len("bridge", kind, size) as f64 / 2.0;
    let (xf, yf, kf) = (x as f64, y as f64, kind as f64);
    let ring = |sc: f64, dx: f64, dy: f64| -> Vec<(f64, f64)> {
        (0..9)
            .map(|j| {
                let jf = j as f64;
                let a = TAU * jf / 9.0 + kf * 0.6;
                let rad = r * sc * (0.86 + 0.14 * (jf * 2.4 + kf * 1.9).sin());
                (xf + dx + a.cos() * rad * 1.08, yf + dy + a.sin() * rad)
            })
            .collect()
    };
    draw::polygon(s, (74, 66, 68), &ring(1.0, 0.0, 0.0), 0);
    draw::polygon(s, (132, 124, 120), &ring(0.84, -r * 0.08, -r * 0.10), 0);
    draw::polygon(s, (168, 160, 152), &ring(0.42, -r * 0.28, -r * 0.32), 0);
    let lw = 1.max((r * 0.09) as i32);
    draw::lines(
        s,
        (90, 82, 82),
        false,
        &[(xf + r * 0.12, yf - r * 0.18), (xf + r * 0.30, yf + r * 0.08), (xf + r * 0.20, yf + r * 0.40)],
        lw,
    );
    draw::line(s, (90, 82, 82), (xf - r * 0.42, yf + r * 0.22), (xf - r * 0.12, yf + r * 0.34), lw);
}

/// `_log(s, x, y, size, kind)` - Treibholz (Fluss): liegender Stamm,
/// Stirnseite rechts, Gischt unten.
fn log(s: &mut Surface, x: i32, y: i32, size: i32, kind: i32) {
    let lw = 6.max((size as f64 * 0.80) as i32);
    let lh = 3.max(obstacle_len("river", kind, size));
    let (x0, y0) = ((x as f64 - lw as f64 / 2.0) as i32, (y as f64 - lh as f64 / 2.0) as i32);
    let (bark, bark_d, bark_l): (Rgb, Rgb, Rgb) = ((128, 84, 50), (88, 56, 36), (160, 112, 70));
    let (wood, ring): (Rgb, Rgb) = ((228, 188, 130), (180, 132, 84));
    let (x0f, y0f, lwf, lhf) = (x0 as f64, y0 as f64, lw as f64, lh as f64);
    // Stamm (unten dunkler = rund)
    draw::rect_r(s, bark_d, (x0, y0, lw, lh), 0, py::fdiv32(lh, 2));
    draw::rect_r(s, bark, (x0, y0, lw, 2.max((lhf * 0.70) as i32)), 0, py::fdiv32(lh, 2));
    let t = 1.max((lhf * 0.10) as i32);
    draw::line(s, bark_l, (x0f + lhf * 0.5, y0f + lhf * 0.22), ((x0 + lw) as f64 - lhf * 0.9, y0f + lhf * 0.22), t);
    // Rindenrillen
    for j in 0..3 {
        let yy = y0f + lhf * (0.40 + 0.18 * j as f64);
        let a = x0f + lhf * (0.45 + 0.9 * py::pmod32(j * 2 + kind, 3) as f64);
        let b = a + lwf * (0.30 + 0.06 * py::pmod32(kind + j, 2) as f64);
        draw::line(s, bark_d, (a, yy), (b.min((x0 + lw) as f64 - lhf * 0.8), yy), t);
    }
    // Aststummel oben
    let sx = x0f + lwf * (0.30 + 0.12 * py::pmod32(kind, 3) as f64);
    draw::ellipse(
        s,
        bark_d,
        ((sx - lhf * 0.16) as i32, (y0f - lhf * 0.12) as i32, 2.max((lhf * 0.32) as i32), 2.max((lhf * 0.30) as i32)),
        0,
    );
    draw::ellipse(
        s,
        wood,
        ((sx - lhf * 0.09) as i32, (y0f - lhf * 0.07) as i32, 1.max((lhf * 0.18) as i32), 1.max((lhf * 0.16) as i32)),
        0,
    );
    // Stirnseite mit Jahresringen
    let ew = 2.max((lhf * 0.62) as i32);
    let er = Rect::new(x0 + lw - ew, y0, ew, lh);
    draw::ellipse(s, bark_d, er, 0);
    let d = 2.max((lhf * 0.16) as i32);
    draw::ellipse(s, wood, er.inflate(-d, -d), 0);
    draw::ellipse(s, ring, er.inflate(-((ew as f64 * 0.45) as i32), -((lhf * 0.45) as i32)), 1.max((lhf * 0.06) as i32));
    draw::circle(s, ring, er.center(), 1.max((lhf * 0.06) as i32), 0);
    // kleine Gischt an der Wasserlinie (flache Schaumkronen + Bugwelle)
    let foam: Rgb = (240, 248, 252);
    for j in 0..5 {
        let fw = lwf * (0.16 + 0.04 * py::pmod32(j + kind, 2) as f64);
        let fx = x0f + lwf * (0.02 + 0.20 * j as f64);
        let fh = 2.max((lhf * 0.22) as i32);
        let fy = y0f + lhf * (0.90 + 0.05 * (j as f64 * 1.7 + kind as f64).sin());
        draw::ellipse(s, foam, (fx as i32, fy as i32, 2.max(fw as i32), fh), 0);
    }
    for sx in [-1, 1] {
        let ex = if sx < 0 { x0f - lhf * 0.10 } else { (x0 + lw) as f64 - lhf * 0.12 };
        draw::ellipse(
            s,
            foam,
            (ex as i32, (y0f + lhf * 0.70) as i32, 2.max((lhf * 0.26) as i32), 2.max((lhf * 0.34) as i32)),
            0,
        );
    }
}

/// `_ice_block(s, x, y, size, kind)` - rutschender Eisblock (Eissee): Quader
/// mit Deck- und Seitenflaeche.
fn ice_block(s: &mut Surface, x: i32, y: i32, size: i32, kind: i32) {
    let bh = 4.max(obstacle_len("ice", kind, size));
    let bw = 4.max((size as f64 * 0.62) as i32);
    let (x0, y0) = ((x as f64 - bw as f64 / 2.0) as i32, (y as f64 - bh as f64 / 2.0) as i32);
    let (x1, y1) = (x0 + bw, y0 + bh);
    let th = (bh as f64 * (0.36 + 0.04 * py::pmod32(kind, 2) as f64)) as i32; // Hoehe der Deckflaeche
    let sd = 2.max((bw as f64 * 0.16) as i32); // Versatz der Seitenflaeche
    let snow: Rgb = (236, 246, 252);
    let sp = 2.max((bh as f64 * 0.16) as i32);
    let (bwf, bhf, spf) = (bw as f64, bh as f64, sp as f64);
    // Schneestaub seitlich hinter dem Block
    for (fx, fy) in [(-0.12, -0.20), (1.0, -0.40)] {
        let (rw, rh) = (2.max((spf * 1.6) as i32), 1.max((spf * 0.9) as i32));
        draw::ellipse(
            s,
            snow,
            ((x0 as f64 + bwf * fx - rw as f64 / 2.0) as i32, (y1 as f64 + bhf * fy - rh as f64 / 2.0) as i32, rw, rh),
            0,
        );
    }
    ice_faces(s, x0, y0, x1, y1, th, sd, bw, bh, kind);
    // vorn aufgeschobener Schneewulst + ein paar Flocken
    draw::ellipse(
        s,
        snow,
        (x0 - (bwf * 0.06) as i32, y1 - (spf * 0.7) as i32, bw - sd + (bwf * 0.12) as i32, 2.max((spf * 1.1) as i32)),
        0,
    );
    for (j, (fx, fy)) in [(-0.17, -0.04), (-0.10, -0.34), (1.06, -0.18)].into_iter().enumerate() {
        draw::circle(
            s,
            snow,
            ((x0 as f64 + bwf * fx) as i32, (y1 as f64 + bhf * fy) as i32),
            1.max((spf * (0.30 + 0.10 * py::pmod32(j as i32 + kind, 2) as f64)) as i32),
            0,
        );
    }
}

/// `_ice_faces(...)` - Flaechen, Kanten und Glanz des Eisblocks.
#[allow(clippy::too_many_arguments)]
fn ice_faces(s: &mut Surface, x0: i32, y0: i32, x1: i32, y1: i32, th: i32, sd: i32, bw: i32, bh: i32, kind: i32) {
    let edge: Rgb = (92, 150, 194);
    let front = [(x0, y0 + th), (x1 - sd, y0 + th), (x1 - sd, y1), (x0, y1)];
    let top = [(x0, y0 + th), (x0 + sd, y0), (x1, y0), (x1 - sd, y0 + th)];
    let side = [(x1 - sd, y0 + th), (x1, y0), (x1, y1 - th), (x1 - sd, y1)];
    draw::polygon(s, (178, 222, 244), &front, 0);
    draw::polygon(s, (128, 182, 220), &side, 0);
    draw::polygon(s, (240, 250, 255), &top, 0);
    let bwf = bw as f64;
    let lw = 1.max((bwf * 0.025) as i32);
    for pts in [&front, &top, &side] {
        draw::polygon(s, edge, pts, lw);
    }
    // Glanz + Luftblaeschen + Riss
    let fw = (bw - sd) as f64;
    let (x0f, y1f) = (x0 as f64, y1 as f64);
    let yt = (y0 + th) as f64; // y0 + th
    let dh = (bh - th) as f64; // bh - th
    draw::line(s, (255, 255, 255), (x0f + fw * 0.14, y1f - dh * 0.25), (x0f + fw * 0.38, yt + dh * 0.22), 1.max((bwf * 0.05) as i32));
    draw::line(s, (255, 255, 255), (x0f + fw * 0.34, y1f - dh * 0.20), (x0f + fw * 0.46, yt + dh * 0.45), 1.max((bwf * 0.03) as i32));
    for j in 0..2 {
        let jf = j as f64;
        draw::circle(
            s,
            (220, 242, 252),
            ((x0f + fw * (0.62 + 0.16 * jf)) as i32, (yt + dh * (0.35 + 0.25 * jf)) as i32),
            1.max((bwf * 0.03) as i32),
            0,
        );
    }
    if py::pmod32(kind, 2) == 1 {
        draw::lines(
            s,
            edge,
            false,
            &[(x0f + fw * 0.70, yt), (x0f + fw * 0.62, yt + dh * 0.4), (x0f + fw * 0.72, y1f - dh * 0.15)],
            lw,
        );
    }
    let thf = th as f64;
    draw::line(
        s,
        (255, 255, 255),
        (x0f + sd as f64 * 0.6, y0 as f64 + thf * 0.5),
        ((x0 + sd) as f64 + bwf * 0.25, y0 as f64 + thf * 0.5),
        1.max((bwf * 0.03) as i32),
    );
}

/// `draw_obstacle(s, world, x, y, size, kind, alpha)` - Hindernis mittig bei
/// (x, y); size = fertige Pixelgroesse, kind 0..3.
///
/// Strasse: Auto (kind 0-2 rot/blau/gelb) bzw. Lieferwagen (3) von oben,
/// Front unten; Bruecke: Felsbrocken; Fluss: Treibholz; Eissee: Eisblock.
/// Senkrechte Ausdehnung = obstacle_len(world, kind, size).
pub fn draw_obstacle(s: &mut Surface, world: &str, x: f64, y: f64, size: i32, kind: i32, alpha: i32) {
    if alpha <= 0 || size <= 0 {
        return;
    }
    let kind = py::pmod32(kind, 4);
    if world == "road" {
        let mut img = car_sprite(size, kind);
        if alpha < 255 {
            // Kopie: das Bild teilt seinen Speicher mit dem Zwischenspeicher
            img = img.copy();
            img.set_alpha(Some(alpha as u8));
        }
        let (iw, ih) = (img.w(), img.h());
        s.bl(&img, (x as i32 - py::fdiv32(iw, 2), y as i32 - py::fdiv32(ih, 2)));
        return;
    }
    if alpha < 255 {
        let (lw, lh) = (size + 8, (size as f64 * 0.8) as i32 + 8);
        let mut layer = pg::surface_alpha(lw, lh);
        draw_obstacle(&mut layer, world, py::fdiv32(lw, 2) as f64, py::fdiv32(lh, 2) as f64, size, kind, 255);
        layer.set_alpha(Some(alpha as u8));
        s.bl(&layer, (x as i32 - py::fdiv32(lw, 2), y as i32 - py::fdiv32(lh, 2)));
        return;
    }
    if world == "bridge" {
        boulder(s, x as i32, y as i32, size, kind);
    } else if world == "river" {
        log(s, x as i32, y as i32, size, kind);
    } else {
        ice_block(s, x as i32, y as i32, size, kind);
    }
}

/// `draw_barrier(s, x, y, w, k)` - rot-weisse Absperrung (Balken der Breite
/// w mittig bei x, Hoehe ~8k bei y).
pub fn draw_barrier(s: &mut Surface, x: f64, y: f64, w: i32, k: f64) {
    let w = 8.max(w);
    let bh = 4.max((8.0 * k) as i32);
    let pw = 2.max((4.0 * k) as i32);
    let ph = 4.max((12.0 * k) as i32);
    let (x0, y0) = (x as i32 - py::fdiv32(w, 2), y as i32 - py::fdiv32(bh, 2));
    let (post, post_d): (Rgb, Rgb) = ((84, 86, 96), (48, 50, 58));
    // Pfosten mit Fuessen
    for px in [x0 + pw, x0 + w - 2 * pw] {
        s.fill_r(post, (px, y0, pw, py::fdiv32(bh, 2) + ph));
        let fh = 2.max((3.0 * k) as i32);
        s.fill_r(post_d, (px - pw, y0 + py::fdiv32(bh, 2) + ph - fh, pw * 3, fh));
    }
    let bar = cached(format!("barrier|{w}|{bh}"), || {
        let mut bar = pg::surface(w, bh);
        bar.fill_c((244, 244, 238));
        let st = 3.max((bh as f64 * 1.1) as i32);
        for i in -1..py::fdiv32(w, st) + 2 {
            if py::pmod32(i, 2) == 0 {
                continue;
            }
            let sx = i * st;
            draw::polygon(&mut bar, (214, 44, 46), &[(sx, 0), (sx + st, 0), (sx + st - bh, bh), (sx - bh, bh)], 0);
        }
        draw::rect(&mut bar, (120, 28, 34), (0, 0, w, bh), 1);
        bar
    });
    s.bl(&bar, (x0, y0));
    // kleines Warnlicht links oben
    let r = 2.max((4.0 * k) as i32);
    let (lx, ly) = (x0 + pw + py::fdiv32(pw, 2), y0 - r);
    let glow = cached(format!("glow|{r}"), || glow_surface(r));
    s.bl(&glow, (lx - py::fdiv32(glow.w(), 2), ly - py::fdiv32(glow.h(), 2)));
    s.fill_r(post_d, (lx - py::fdiv32(r, 2), ly, 1.max(r), r + 1));
    draw::circle(s, (255, 168, 40), (lx, ly), r, 0);
    draw::circle(s, (255, 236, 170), (lx - py::fdiv32(r, 3), ly - py::fdiv32(r, 3)), 1.max(py::fdiv32(r, 2)), 0);
}

/// `_glow_surface(r)` - weicher Lichthof fuer das Warnlicht (3 Ringe, Alpha
/// ersetzt).
fn glow_surface(r: i32) -> Surface {
    let mut g = pg::surface_alpha(r * 6, r * 6);
    for (i, a) in [(3, 40), (2, 70), (1, 100)] {
        draw::circle(&mut g, (255, 190, 70, a), (r * 3, r * 3), (r as f64 * (1.0 + i as f64 * 0.6)) as i32, 0);
    }
    g
}

/// `draw_tire_marks(s, x, y, w, k)` - zwei dunkle, halbdurchsichtige
/// Reifenspuren mittig bei (x, y). Laenge ~ w*1.2, Abstand ~ w*0.32,
/// Spurbreite max(2, 5k).
pub fn draw_tire_marks(s: &mut Surface, x: f64, y: f64, w: f64, k: f64) {
    let sw = 2.max((5.0 * k) as i32);
    let ln = 4.max((w * 1.2) as i32);
    let gp = (sw + 1).max((w * 0.32) as i32);
    let tread = 2.max((3.0 * k) as i32);
    let lay = cached(format!("tire|{ln}|{gp}|{sw}|{tread}"), || {
        let mut lay = pg::surface_alpha(gp + sw, ln);
        let half = ln as f64 / 2.0;
        for yy in 0..ln {
            let f = 1.0 - (yy as f64 + 0.5 - half).abs() / half; // 0 an den Enden, 1 Mitte
            let mut a = (30.0 + 110.0 * 1.0f64.min(f * 2.5)) as i32;
            if py::pmod32(py::fdiv32(yy, tread), 2) == 1 {
                a = py::fdiv32(a * 6, 10); // Profilkerben
            }
            for xx in [0, gp] {
                lay.fill_r((22, 20, 26, a), (xx, yy, sw, 1));
            }
        }
        lay
    });
    s.bl(&lay, (x as i32 - py::fdiv32(lay.w(), 2), y as i32 - py::fdiv32(ln, 2)));
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
    } else if world == "road" {
        // Strasse in die Ferne: zulaufende Raender, kuerzer werdende Mittelstriche
        let (ry, rb) = (r.y as f64, r.bottom() as f64);
        for sx in [-1.0, 1.0] {
            draw::line(s, col, (cx + sx * u * 4.4, rb - u * 0.6), (cx + sx * u * 1.2, ry + u * 1.0), lw);
        }
        draw::line(s, col, (cx, rb - u * 0.6), (cx, rb - u * 2.8), lw + 1);
        draw::line(s, col, (cx, cy + u * 0.6), (cx, cy - u * 0.9), lw);
        draw::line(s, col, (cx, ry + u * 2.6), (cx, ry + u * 1.6), 1.max(lw - 1));
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
