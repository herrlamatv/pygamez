//! lamamines_draw.rs
//! =================
//! Zeichen-Bausteine von Lama Mines - uebertragen aus
//! `games/lamamines_draw.py` (nur Zeichen-Primitive, keine Bilddateien):
//! Hoehlenwand (Verlauf, Gesteinsbrocken, Koernung, Kristalladern, Vignette),
//! Holzrahmen mit Eisenwinkeln, Steinquader (4 Varianten, Maus darueber),
//! ausgehobenes Feld, Feld der getroffenen Mine, Abdunklung, Edelstein
//! (4 Farben), Seemine (Licht an/aus), fertige Vorderseiten, Lichthof zum
//! additiven Blitten, Funkeln, Auto-Markierung, das Maskottchen (Lama aus
//! [`crate::lamacrossing_draw`] mit Grubenhelm, auf Wunsch russig),
//! Spitzhacke, Lore und die Piktogramme der Knoepfe.
//!
//! Die gecachten Flaechen werden doppelt so gross gezeichnet und weich
//! verkleinert. Die Ausdruecke stehen in derselben Reihenfolge wie in Python,
//! damit jede Kommazahl gleich rundet.

use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::{PI, TAU};

use pgz_engine::prelude::*;

use crate::lamacrossing_draw as ld;

type Rgb = (i32, i32, i32);

// Farben -------------------------------------------------------------------
/// Edelstein-Farben: Tuerkis (Spielfarbe), Rubin, Saphir, Topas.
pub const GEM_COLS: [Rgb; 4] = [(30, 212, 174), (236, 72, 104), (84, 140, 248), (250, 196, 64)];
const STONE: Rgb = (66, 70, 98);
const STONE_HOT: Rgb = (86, 92, 128);
const STONE_D: Rgb = (32, 34, 52);
const STONE_L: Rgb = (112, 118, 156);
const PIT: Rgb = (17, 18, 30);
const PIT_D: Rgb = (9, 9, 17);
const PIT_L: Rgb = (44, 48, 72);
const WOOD: Rgb = (112, 72, 42);
const WOOD_D: Rgb = (66, 40, 24);
const WOOD_L: Rgb = (146, 98, 58);
const IRON: Rgb = (104, 108, 124);
const IRON_D: Rgb = (60, 62, 74);
const IRON_L: Rgb = (168, 172, 188);
const HELMET: Rgb = (248, 192, 44);
const MINE_BODY: Rgb = (48, 46, 58);
const MINE_HORN: Rgb = (92, 90, 104);
const MINE_RED: Rgb = (255, 64, 70);
const MINE_RED_OFF: Rgb = (120, 32, 40);

thread_local! {
    static CACHE: RefCell<HashMap<String, Surface>> = RefCell::new(HashMap::new());
}

/// `_cached(key, build)` - wie Python: ueber 400 Eintraege wird geleert.
fn cached(key: String, build: impl FnOnce() -> Surface) -> Surface {
    if let Some(s) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return s;
    }
    let surf = build();
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 400 {
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

/// `_hash(i)` - Ganzzahl-Streuwert 0..2^32-1 (deterministisch, ohne Zufall/sin).
pub fn hash(i: i64) -> u64 {
    let v = (i as i128 * 2_654_435_761 + 40_503).rem_euclid(4_294_967_296) as u64;
    let v = (v ^ (v >> 15)).wrapping_mul(2_246_822_519) % 4_294_967_296;
    v ^ (v >> 13)
}

/// `_shade(col, f)` - f > 0 Richtung Weiss, f < 0 Richtung Schwarz.
pub fn shade(col: impl Col, f: f64) -> Color {
    let other = if f > 0.0 { (255, 255, 255) } else { (0, 0, 0) };
    ui::mix(col, other, f.abs())
}

/// `_gradient(surface, rect, top, bottom)`
fn gradient(s: &mut Surface, r: (i32, i32, i32, i32), top: Rgb, bottom: Rgb) {
    let (x, y, w, h0) = r;
    let h = 1.max(h0 - 1);
    for yy in 0..h0 {
        s.fill_r(ui::mix(top, bottom, yy as f64 / h as f64), (x, y + yy, w, 1));
    }
}

/// `_smooth(size, draw, ss=2)` - doppelt gross zeichnen, weich verkleinern.
fn smooth(size: i32, draw: impl FnOnce(&mut Surface, i32)) -> Surface {
    let mut big = pg::surface_alpha(size * 2, size * 2);
    draw(&mut big, size * 2);
    transform::smoothscale(&big, size, size)
}

fn ckey(c: Color) -> String {
    format!("{},{},{}", c.r, c.g, c.b)
}

// ============================================================= Hintergrund

/// `_crystal(s, x, y, h, ang, col)` - schlanker Kristall mit Glanzkante.
fn crystal(s: &mut Surface, x: f64, y: f64, h: f64, ang: f64, col: Rgb) {
    let (ca, sa) = (ang.cos(), ang.sin());
    let w = h * 0.26;
    // u entlang der Achse (0 = Fuss, 1 = Spitze), v quer (-1..1)
    let p = |u: f64, v: f64| (x + ca * u * h - sa * v * w, y + sa * u * h + ca * v * w);
    let pts = [p(0.0, -0.8), p(0.78, -1.0), p(1.0, 0.0), p(0.78, 1.0), p(0.0, 0.8)];
    draw::polygon(s, shade(col, -0.45), &pts, 0);
    draw::polygon(s, col, &[p(0.0, -0.8), p(0.78, -1.0), p(1.0, 0.0), p(0.05, 0.0)], 0);
    draw::line(s, shade(col, 0.55), p(0.08, -0.35), p(0.80, -0.55), 1.max((h * 0.05) as i32));
}

/// `background(w, h)` - Hoehlenwand, gecacht je Aufloesung.
pub fn background(w: i32, h: i32) -> Surface {
    cached(format!("bg|{w}|{h}"), || {
        let k = (w as f64 / 800.0).min(h as f64 / 600.0);
        let kk = 0.6f64.max(k);
        let mut s = pg::surface(w, h);
        gradient(&mut s, (0, 0, w, h), (16, 15, 28), (30, 22, 36));
        // grosse, weiche Gesteinsbrocken (etwas heller als die Wand)
        for i in 0..22i64 {
            let v = hash(i * 3 + 1);
            let x = (v % w as u64) as i32;
            let y = ((v >> 11) % h as u64) as i32;
            let r = ((24 + (v >> 5) % 70) as f64 * kk) as i32;
            let f = ((v >> 3) % 100) as f64 / 100.0;
            let col = ui::mix((24, 22, 38), (34, 30, 48), f);
            let rr = (x - r, y - (r as f64 * 0.6) as i32, 2 * r, (r as f64 * 1.2) as i32);
            draw::ellipse(&mut s, col, rr, 0);
            draw::arc(&mut s, ui::mix(col, (60, 54, 80), 0.6), rr, PI * 0.25, PI * 0.85, 1.max((2.0 * k) as i32));
        }
        // feine Koernung
        let grain: [Rgb; 3] = [(40, 36, 56), (12, 11, 20), (28, 26, 42)];
        let (wu, hu) = (w as u64, h as u64);
        for i in 0..(w as i64 * h as i64 / 420) {
            let v = hash(5000 + i);
            s.fill_r(grain[((v >> 24) % 3) as usize], ((v % wu) as i32, ((v / wu) % hu) as i32, 1, 1));
        }
        // Kristalladern mit Lichthof
        let spots: [(f64, f64, usize); 7] = [
            (0.03, 0.30, 0),
            (0.97, 0.22, 1),
            (0.02, 0.78, 2),
            (0.98, 0.70, 0),
            (0.50, 0.985, 1),
            (0.30, 0.99, 2),
            (0.74, 0.99, 0),
        ];
        let cols: [Rgb; 3] = [(30, 200, 168), (150, 96, 230), (70, 150, 240)];
        for (i, &(fx, fy, ci)) in spots.iter().enumerate() {
            let (x, y) = ((w as f64 * fx) as i32, (h as f64 * fy) as i32);
            let col = cols[ci];
            let r = (46.0 * kk) as i32;
            let mut glow = pg::surface_alpha(r * 2, r * 2);
            for (j, a) in [(3, 14), (2, 18), (1, 24)] {
                draw::circle(&mut glow, (col.0, col.1, col.2, a), (r, r), (r * j) / 3, 0);
            }
            s.bl(&glow, (x - r, y - r));
            let base = -PI / 2.0 + (if fx > 0.5 { 0.5 } else { -0.5 }) * (if fy < 0.9 { 1.0 } else { 0.4 });
            for j in 0..3i32 {
                let hh = (18 + 9 * ((i as i32 + j) % 3)) as f64 * kk;
                crystal(&mut s, x as f64 + ((j - 1) * 7) as f64 * k, y as f64, hh, base + (j - 1) as f64 * 0.42, col);
            }
        }
        // Vignette: 10 Ringe von aussen nach innen, jeder etwas heller (Alpha 80 -> 0)
        let mut vig = pg::surface_alpha(w, h);
        let steps = 10;
        let band = 1.max((w.min(h) as f64 * 0.012) as i32);
        for i in 0..steps {
            let a = 80 * (steps - i) * (steps - i) / (steps * steps);
            draw::rect(&mut vig, (0, 0, 0, a), (i * band, i * band, w - 2 * i * band, h - 2 * i * band), band);
        }
        s.bl(&vig, (0, 0));
        s
    })
}

// ============================================================= Spielbrett

/// `board(side, pad)` - Holzrahmen (Breite pad) mit Eisenwinkeln um die Grube.
pub fn board(side: i32, pad: i32) -> Surface {
    cached(format!("board|{side}|{pad}"), || {
        let mut s = pg::surface_alpha(side, side);
        let rad = 4.max((side as f64 * 0.035) as i32);
        draw::rect_r(&mut s, WOOD_D, (0, 0, side, side), 0, rad);
        draw::rect_r(&mut s, WOOD, (1, 1, side - 2, side - 3), 0, rad);
        // Maserung: Bretter oben/unten waagerecht, links/rechts senkrecht
        let lw = 1.max(side / 260);
        for j in 0..3 {
            let f = (j + 1) as f64 / 4.0;
            let y0 = (pad as f64 * f) as i32;
            draw::line(&mut s, WOOD_D, (pad, y0), (side - pad, y0 + (j - 1)), lw);
            draw::line(&mut s, WOOD_D, (pad, side - y0), (side - pad, side - y0 - (j - 1)), lw);
            let x0 = (pad as f64 * f) as i32;
            draw::line(&mut s, WOOD_D, (x0, pad), (x0 + (j - 1), side - pad), lw);
            draw::line(&mut s, WOOD_D, (side - x0, pad), (side - x0 - (j - 1), side - pad), lw);
        }
        for i in 0..6i64 {
            let v = hash(i * 11 + 3);
            let x = pad + (v % 1.max(side - 2 * pad) as u64) as i32;
            let cy = if i % 2 == 0 { pad / 2 } else { side - pad / 2 };
            let knot = Rect::new(0, 0, 3.max(pad / 2), 2.max(pad / 4)).center_at((x, cy));
            draw::ellipse(&mut s, WOOD_D, knot, 1.max(lw));
        }
        draw::line(&mut s, WOOD_L, (rad, 1), (side - rad, 1), 1.max(lw));
        // Fugen an den Ecken (Gehrung)
        for ((ax, ay), (bx, by)) in [
            ((0, 0), (pad, pad)),
            ((side, 0), (side - pad, pad)),
            ((0, side), (pad, side - pad)),
            ((side, side), (side - pad, side - pad)),
        ] {
            draw::line(&mut s, WOOD_D, (ax, ay), (bx, by), 1.max(lw));
        }
        // Grube mit Innenschatten
        let inner = Rect::new(pad, pad, side - 2 * pad, side - 2 * pad);
        let irad = 3.max(rad / 2);
        draw::rect_r(&mut s, PIT_D, inner, 0, irad);
        draw::rect_r(&mut s, (14, 14, 24), inner.inflate(-2, -2).move_(0, 1.max(pad / 5)), 0, irad);
        draw::rect_r(&mut s, (40, 26, 16), inner.inflate(2, 2), 1, irad);
        // Eisenwinkel mit Nieten
        let a = (pad as f64 * 2.1) as i32;
        let t = 3.max((pad as f64 * 0.62) as i32);
        let nr = 1.max((pad as f64 * 0.14) as i32);
        for (cx, cy, sx, sy) in [(0, 0, 1, 1), (side, 0, -1, 1), (0, side, 1, -1), (side, side, -1, -1)] {
            let x0 = if sx > 0 { cx } else { cx - a };
            let y0 = if sy > 0 { cy } else { cy - t };
            draw::rect_r(&mut s, IRON_D, (x0, y0 + sy, a, t), 0, 1.max(t / 3));
            draw::rect_r(&mut s, IRON, (x0, y0, a, t), 0, 1.max(t / 3));
            let x1 = if sx > 0 { cx } else { cx - t };
            let y1 = if sy > 0 { cy } else { cy - a };
            draw::rect_r(&mut s, IRON_D, (x1 + sx, y1, t, a), 0, 1.max(t / 3));
            draw::rect_r(&mut s, IRON, (x1, y1, t, a), 0, 1.max(t / 3));
            for (ox, oy) in [(t / 2, t / 2), (a - t / 2, t / 2), (t / 2, a - t / 2)] {
                let (px, py) = (cx + sx * ox, cy + sy * oy);
                draw::circle(&mut s, IRON_D, (px, py + 1), nr + 1, 0);
                draw::circle(&mut s, IRON_L, (px, py), nr, 0);
            }
        }
        s
    })
}

// ============================================================= Felder

fn tile_rad(size: i32) -> i32 {
    2.max((size as f64 * 0.16) as i32)
}

/// `tile_hidden(size, var, hot)` - Steinquader; var 0..3 = Sprenkel-Variante.
pub fn tile_hidden(size: i32, var: i64, hot: bool) -> Surface {
    let key = format!("hid|{size}|{}|{}", var.rem_euclid(4), hot as i32);
    cached(key, || {
        let sf = size as f64;
        let mut s = pg::surface_alpha(size, size);
        let rad = tile_rad(size);
        let bev = 2.max((sf * 0.09) as i32);
        let body = if hot { STONE_HOT } else { STONE };
        draw::rect_r(&mut s, STONE_D, (0, 0, size, size), 0, rad);
        draw::rect_r(&mut s, body, (0, 0, size, size - bev), 0, rad);
        let face = Rect::new(bev, bev, size - 2 * bev, size - 3 * bev);
        draw::rect_r(&mut s, shade(body, 0.10), face, 0, 1.max(rad - bev / 2));
        let lc = if !hot { STONE_L.c() } else { shade(STONE_L, 0.2) };
        draw::line(&mut s, lc, (rad, 1.max(bev / 2)), (size - rad - 1, 1.max(bev / 2)), 1.max(bev / 2));
        // Sprenkel und ein feiner Riss je Variante
        for j in 0..5i64 {
            let v = hash(var * 31 + j * 7 + 1);
            let x = face.x + 2 + (v % 1.max(face.w - 4) as u64) as i32;
            let y = face.y + 2 + ((v >> 9) % 1.max(face.h - 4) as u64) as i32;
            let r = 1.max((sf * (0.022 + 0.012 * ((v >> 4) % 2) as f64)) as i32);
            let col = if j % 2 == 0 { shade(body, -0.25) } else { shade(body, 0.22) };
            draw::circle(&mut s, col, (x, y), r, 0);
        }
        if var % 2 == 1 {
            // feiner Haarriss von der oberen Kante schraeg nach unten
            let v = hash(var * 97 + 5);
            let (fx, fy, fw, fh) = (face.x as f64, face.y as f64, face.w as f64, face.h as f64);
            let x0 = fx + fw * (0.55 + (v % 25) as f64 / 100.0);
            let pts = [
                (x0, fy),
                (x0 - fw * 0.06, fy + fh * 0.16),
                (x0 + fw * 0.02, fy + fh * 0.27),
                (x0 - fw * 0.08, fy + fh * 0.42),
            ];
            draw::lines(&mut s, shade(body, -0.16), false, &pts, 1);
        }
        s
    })
}

/// `tile_open(size)` - ausgehobenes Feld (Grube mit Innenschatten).
pub fn tile_open(size: i32) -> Surface {
    cached(format!("open|{size}"), || {
        let mut s = pg::surface_alpha(size, size);
        let rad = tile_rad(size);
        let bev = 2.max((size as f64 * 0.09) as i32);
        draw::rect_r(&mut s, PIT_L, (0, 0, size, size), 0, rad);
        draw::rect_r(&mut s, PIT_D, (0, 0, size, size - 1.max(bev / 2)), 0, rad);
        let b3 = 1.max(bev / 3);
        draw::rect_r(&mut s, PIT, (b3, bev, size - 2 * b3, size - bev - 1.max(bev / 2) - 1), 0, 1.max(rad - 1));
        s
    })
}

/// `tile_boom(size)` - Feld der getroffenen Mine (rot gluehend, Brandfleck).
pub fn tile_boom(size: i32) -> Surface {
    cached(format!("boom|{size}"), || {
        let sf = size as f64;
        let mut s = pg::surface_alpha(size, size);
        let rad = tile_rad(size);
        draw::rect_r(&mut s, (110, 20, 28), (0, 0, size, size), 0, rad);
        draw::rect_r(&mut s, (176, 38, 44), (1, 1, size - 2, size - 3), 0, rad);
        let c = size / 2;
        for (i, col) in [(214, 70, 50), (240, 120, 60), (250, 170, 90)].iter().enumerate() {
            draw::circle(&mut s, *col, (c, c), (sf * (0.44 - 0.11 * i as f64)) as i32, 0);
        }
        let burn = Rect::new(0, 0, (sf * 0.64) as i32, (sf * 0.5) as i32).center_at((c, (sf * 0.58) as i32));
        draw::ellipse(&mut s, (70, 22, 20), burn, 0);
        for j in 0..6 {
            let a = TAU * j as f64 / 6.0 + 0.3;
            let x1 = c as f64 + a.cos() * sf * 0.46;
            let y1 = c as f64 + a.sin() * sf * 0.40;
            draw::line(&mut s, (90, 26, 22), (c, (sf * 0.58) as i32), (x1, y1), 1.max(size / 30));
        }
        s
    })
}

/// `dim(size, alpha=150)` - halbdurchsichtige Abdunklung eines Feldes.
pub fn dim(size: i32, alpha: i32) -> Surface {
    cached(format!("dim|{size}|{alpha}"), || {
        let mut s = pg::surface_alpha(size, size);
        draw::rect_r(&mut s, (6, 6, 14, alpha), (0, 0, size, size), 0, tile_rad(size));
        s
    })
}

// ============================================================= Edelstein + Mine

fn draw_gem(s: &mut Surface, n: i32, col: Rgb) {
    let nf = n as f64;
    let (tl, tr) = ((nf * 0.30, nf * 0.17), (nf * 0.70, nf * 0.17));
    let (l, r, b) = ((nf * 0.09, nf * 0.39), (nf * 0.91, nf * 0.39), (nf * 0.50, nf * 0.90));
    let t = (nf * 0.50, nf * 0.17);
    let (g1, g2) = ((nf * 0.35, nf * 0.39), (nf * 0.65, nf * 0.39));
    let outline = [tl, tr, r, b, l];
    // weicher Schatten
    draw::ellipse(s, (0, 0, 0, 70), (nf * 0.22, nf * 0.84, nf * 0.56, nf * 0.10), 0);
    let sh: Vec<(f64, f64)> = outline.iter().map(|&(x, y)| (x + nf * 0.012, y + nf * 0.02)).collect();
    draw::polygon(s, shade(col, -0.55), &sh, 0);
    draw::polygon(s, col, &outline, 0);
    draw::polygon(s, shade(col, 0.50), &[tl, t, g1, l], 0);
    draw::polygon(s, shade(col, 0.22), &[t, tr, r, g2], 0);
    draw::polygon(s, shade(col, 0.70), &[t, g2, g1], 0);
    draw::polygon(s, shade(col, -0.05), &[l, g1, b], 0);
    draw::polygon(s, shade(col, 0.28), &[g1, g2, b], 0);
    draw::polygon(s, shade(col, -0.38), &[g2, r, b], 0);
    let edge = shade(col, 0.82);
    let lw = 1.max(n / 70);
    draw::line(s, edge, l, r, lw);
    draw::lines(s, edge, true, &outline, lw);
    draw::line(s, shade(col, 0.6), g1, b, lw);
    draw::line(s, shade(col, 0.6), g2, b, lw);
    // Glanzpunkte
    draw::circle(s, (255, 255, 255), (nf * 0.33, nf * 0.25), nf * 0.035, 0);
    draw::polygon(s, (255, 255, 255), &[(nf * 0.40, nf * 0.47), (nf * 0.46, nf * 0.47), (nf * 0.50, nf * 0.70)], 0);
}

/// `gem(size, col_idx=0)` - geschliffener Edelstein (gecacht).
pub fn gem(size: i32, col_idx: usize) -> Surface {
    let ci = col_idx % GEM_COLS.len();
    let col = GEM_COLS[ci];
    cached(format!("gem|{size}|{ci}"), || smooth(size, |s, n| draw_gem(s, n, col)))
}

fn draw_mine(s: &mut Surface, n: i32, lit: bool) {
    let nf = n as f64;
    let (cx, cy) = (nf * 0.5, nf * 0.54);
    let rr = nf * 0.25;
    draw::ellipse(s, (0, 0, 0, 80), (nf * 0.20, nf * 0.84, nf * 0.60, nf * 0.10), 0);
    let lw = 2.max((nf * 0.075) as i32);
    for i in 0..8 {
        let a = TAU * i as f64 / 8.0 + PI / 8.0;
        let (bx, by) = (cx + a.cos() * rr * 0.8, cy + a.sin() * rr * 0.8);
        let (tx, ty) = (cx + a.cos() * rr * 1.42, cy + a.sin() * rr * 1.42);
        draw::line(s, shade(MINE_HORN, -0.3), (bx, by), (tx, ty), lw);
        draw::circle(s, MINE_HORN, (tx, ty), nf * 0.058, 0);
        draw::circle(s, shade(MINE_HORN, 0.45), (tx - nf * 0.015, ty - nf * 0.015), nf * 0.022, 0);
    }
    draw::circle(s, shade(MINE_BODY, -0.45), (cx, cy + nf * 0.02), rr, 0);
    draw::circle(s, MINE_BODY, (cx, cy), rr, 0);
    draw::ellipse(s, shade(MINE_BODY, -0.25), (cx - rr, cy - rr * 0.18, 2.0 * rr, rr * 0.36), 1.max((nf * 0.02) as i32));
    draw::ellipse(s, shade(MINE_BODY, 0.38), (cx - rr * 0.70, cy - rr * 0.78, rr * 0.75, rr * 0.55), 0);
    draw::circle(s, (230, 230, 240), (cx - rr * 0.42, cy - rr * 0.55), nf * 0.028, 0);
    let red = if lit { MINE_RED } else { MINE_RED_OFF };
    if lit {
        draw::circle(s, (255, 120, 110, 90), (cx, cy - rr * 0.05), nf * 0.13, 0);
    }
    draw::circle(s, shade(red, -0.4), (cx, cy - rr * 0.05), nf * 0.075, 0);
    draw::circle(s, red, (cx, cy - rr * 0.05), nf * 0.058, 0);
    draw::circle(s, shade(red, 0.6), (cx - nf * 0.018, cy - rr * 0.05 - nf * 0.02), nf * 0.018, 0);
}

/// `mine(size, lit=True)` - Seemine mit Signallicht.
pub fn mine(size: i32, lit: bool) -> Surface {
    cached(format!("mine|{size}|{}", lit as i32), || smooth(size, |s, n| draw_mine(s, n, lit)))
}

/// `face(size, kind, dimmed)` - fertige Vorderseite eines aufgedeckten Feldes.
///
/// kind: "gem" (Edelstein in der Grube), "mine" (Mine, Licht aus) oder
/// "boom" (getroffene Mine auf gluehendem Grund). dimmed = abgedunkelt.
pub fn face(size: i32, kind: &str, dimmed: bool) -> Surface {
    cached(format!("face|{size}|{kind}|{}", dimmed as i32), || {
        let mut s = pg::surface_alpha(size, size);
        if kind == "boom" {
            s.bl(&tile_boom(size), (0, 0));
        } else {
            s.bl(&tile_open(size), (0, 0));
        }
        let g = 4.max((size as f64 * 0.80) as i32);
        let img = if kind == "gem" { gem(g, 0) } else { mine(g, kind == "boom") };
        s.blr(&img, img.get_rect().center_at((size / 2, size / 2)));
        if dimmed {
            s.bl(&dim(size, 150), (0, 0));
        }
        s
    })
}

/// `glow(r, col)` - Lichthof auf schwarzem Grund (additiv blitten).
pub fn glow(r: i32, col: Rgb) -> Surface {
    cached(format!("glow|{r}|{},{},{}", col.0, col.1, col.2), || {
        let mut s = pg::surface(2 * r + 2, 2 * r + 2);
        s.fill_c((0, 0, 0));
        let steps = 8;
        for i in 0..steps {
            let f = (i + 1) as f64 / steps as f64; // 1/8 .. 1 (aussen -> innen)
            let rr = 1.max((r as f64 * (1.0 - i as f64 / steps as f64)) as i32);
            let a = f * f;
            let c = ((col.0 as f64 * a) as i32, (col.1 as f64 * a) as i32, (col.2 as f64 * a) as i32);
            draw::circle(&mut s, c, (r + 1, r + 1), rr, 0);
        }
        s
    })
}

/// `sparkle(s, x, y, r, col)` - vierzackiges Funkeln.
pub fn sparkle(s: &mut Surface, x: f64, y: f64, r: f64, col: Rgb) {
    if r < 1.0 {
        return;
    }
    let q = r * 0.28;
    let pts = [(x, y - r), (x + q, y - q), (x + r, y), (x + q, y + q), (x, y + r), (x - q, y + q), (x - r, y), (x - q, y - q)];
    draw::polygon(s, col, &pts, 0);
    draw::circle(s, (255, 255, 255), (x as i32, y as i32), 1.max((r * 0.22) as i32), 0);
}

/// `draw_mark(s, rect, col, t, idx)` - Auto-Markierung (Rahmen + Raute).
pub fn draw_mark(s: &mut Surface, r: Rect, col: Color, t: f64, idx: i32) {
    let rad = tile_rad(r.w);
    let lw = 2.max(r.w / 18);
    let glow = 0.5 + 0.5 * (t * 3.0 + idx as f64 * 0.7).sin();
    let edge = ui::mix(col, (255, 255, 255), 0.25 * glow);
    draw::rect_r(s, edge, r.inflate(-lw, -lw), lw, rad);
    let (cx, cy) = (r.centerx(), r.centery() - r.h / 20);
    let d = 3.max(r.w / 7);
    let pts = [(cx, cy - d), (cx + d, cy), (cx, cy + d), (cx - d, cy)];
    let sh: Vec<(i32, i32)> = pts.iter().map(|&(x, y)| (x + 1, y + 2)).collect();
    draw::polygon(s, shade(col, -0.5), &sh, 0);
    draw::polygon(s, edge, &pts, 0);
    draw::polygon(s, shade(col, 0.7), &[(cx, cy - d), (cx + d / 2, cy - d / 2), (cx, cy), (cx - d / 2, cy - d / 2)], 0);
}

// ============================================================= Maskottchen

/// `_helmet(s, n)` - Grubenhelm mit Stirnlampe (Koordinaten wie `_draw_llama`).
fn helmet(s: &mut Surface, ni: i32) {
    let n = ni as f64;
    let (cx, by) = (n * 0.705, n * 0.132);
    let (rx, ry) = (n * 0.128, n * 0.118);
    let dome: Vec<(f64, f64)> = (0..17)
        .map(|i| (cx + rx * (PI * i as f64 / 16.0).cos(), by - ry * (PI * i as f64 / 16.0).sin()))
        .collect();
    let dsh: Vec<(f64, f64)> = dome.iter().map(|&(x, y)| (x, y + n * 0.012)).collect();
    draw::polygon(s, shade(HELMET, -0.35), &dsh, 0);
    draw::polygon(s, HELMET, &dome, 0);
    let hi: Vec<(f64, f64)> = (0..9)
        .map(|i| {
            let w = PI * (0.45 + 0.5 * i as f64 / 8.0);
            (cx - rx * 0.15 + rx * 0.62 * w.cos(), by - ry * 0.18 - ry * 0.70 * w.sin())
        })
        .collect();
    draw::lines(s, shade(HELMET, 0.55), false, &hi, 1.max((n * 0.02) as i32));
    // Mittelgrat
    draw::polygon(
        s,
        shade(HELMET, -0.15),
        &[(cx - n * 0.012, by - ry * 0.98), (cx + n * 0.012, by - ry * 0.98), (cx + n * 0.016, by), (cx - n * 0.016, by)],
        0,
    );
    // Krempe
    draw::rect_r(s, shade(HELMET, -0.30), (cx - rx * 1.16, by - n * 0.006, rx * 2.38, n * 0.034), 0, (n * 0.016) as i32);
    draw::rect_r(s, shade(HELMET, -0.05), (cx - rx * 1.16, by - n * 0.010, rx * 2.38, n * 0.024), 0, (n * 0.012) as i32);
    // Stirnlampe
    let (lx, ly) = (cx + rx * 0.80, by - ry * 0.50);
    draw::rect_r(s, (64, 64, 76), (lx - n * 0.040, ly - n * 0.034, n * 0.062, n * 0.068), 0, (n * 0.014) as i32);
    draw::circle(s, (255, 236, 160), (lx + n * 0.020, ly), n * 0.030, 0);
    draw::circle(s, (255, 255, 230), (lx + n * 0.024, ly - n * 0.006), n * 0.013, 0);
}

/// `_soot(s, n)` - Russflecken nach der Explosion.
fn soot(s: &mut Surface, ni: i32) {
    let n = ni as f64;
    for (x, y, r) in [(0.33, 0.50, 0.050), (0.47, 0.46, 0.038), (0.58, 0.55, 0.030), (0.66, 0.30, 0.030), (0.80, 0.22, 0.024), (0.26, 0.56, 0.026)] {
        draw::circle(s, (126, 118, 114), (n * x, n * y), n * r, 0);
        draw::circle(s, (98, 92, 92), (n * x + n * r * 0.2, n * y + n * r * 0.2), n * r * 0.55, 0);
    }
}

/// `miner_llama(size, pose, scarf, soot)` - Maskottchen mit Grubenhelm.
pub fn miner_llama(size: i32, pose: &str, scarf: Color, sooty: bool) -> Surface {
    let key = format!("miner|{size}|{pose}|{}|{}", ckey(scarf), sooty as i32);
    cached(key, || {
        smooth(size, |big, n| {
            ld::draw_llama(big, n, pose, scarf);
            if sooty {
                soot(big, n);
            }
            helmet(big, n);
        })
    })
}

/// `draw_pickaxe(s, x, y, h, k)` - Spitzhacke, Stiel-Fuss bei (x, y), Hoehe h.
pub fn draw_pickaxe(s: &mut Surface, x: f64, y: f64, h: f64, _k: f64) {
    let lw = 2.max((h * 0.07) as i32);
    let top = (x + h * 0.32, y - h);
    draw::line(s, (88, 56, 30), (x, y), top, lw + 1);
    draw::line(s, (150, 104, 60), (x + 1.0, y - 1.0), (top.0 + 1.0, top.1 + 1.0), 1.max(lw / 2));
    // Kopf: gebogenes Eisen quer zum Stiel
    let mut pts = Vec::with_capacity(9);
    for i in 0..9 {
        let u = -1.0 + (2 * i) as f64 / 8.0;
        pts.push((top.0 + u * h * 0.34, top.1 + u * u * h * 0.12 - h * 0.02));
    }
    let sh: Vec<(f64, f64)> = pts.iter().map(|&(px, py)| (px, py + 2.0)).collect();
    draw::lines(s, IRON_D, false, &sh, lw + 1);
    draw::lines(s, IRON_L, false, &pts, lw);
}

/// `draw_cart(s, x, y, w, k, fill, gem_size)` - Lore (Unterkante der Raeder bei
/// y, linke Kante x, Breite w); fill = sichtbare Edelsteine im Wagen (0..7).
pub fn draw_cart(s: &mut Surface, xi: i32, yi: i32, wi: i32, _k: f64, fill: i32, gem_size: i32) {
    let (x, y, w) = (xi as f64, yi as f64, wi as f64);
    let h = w * 0.46;
    let top = y - w * 0.16 - h;
    // Schienen
    draw::line(s, (70, 64, 70), (x - w * 0.15, y), (x + w * 1.15, y), 2.max((w * 0.035) as i32));
    for i in 0..5 {
        let sx = x - w * 0.1 + (i * wi) as f64 * 0.31;
        draw::line(s, (86, 58, 36), (sx, y + 1.0), (sx + w * 0.12, y + 1.0), 2.max((w * 0.05) as i32));
    }
    // Edelstein-Haufen (ragt ueber den Rand; fy in Edelstein-Groessen ueber der Kante)
    let spots: [(f64, f64, usize); 7] =
        [(0.30, 0.10, 1), (0.52, 0.06, 0), (0.74, 0.10, 2), (0.41, -0.34, 3), (0.63, -0.32, 0), (0.52, -0.70, 1), (0.18, -0.08, 2)];
    for &(fx, fy, ci) in spots.iter().take(fill.clamp(0, 7) as usize) {
        let img = gem(gem_size, ci);
        s.blr(&img, img.get_rect().center_at(((x + w * fx) as i32, (top + gem_size as f64 * fy) as i32)));
    }
    // Wagenkasten (Trapez) mit Rand und Nieten
    let body = [(x, top), (x + w, top), (x + w * 0.88, top + h), (x + w * 0.12, top + h)];
    let bsh: Vec<(f64, f64)> = body.iter().map(|&(px, py)| (px, py + 2.0)).collect();
    draw::polygon(s, IRON_D, &bsh, 0);
    draw::polygon(s, IRON, &body, 0);
    draw::polygon(
        s,
        shade(IRON, -0.18),
        &[(x + w * 0.05, top + h * 0.55), (x + w * 0.95, top + h * 0.55), (x + w * 0.88, top + h), (x + w * 0.12, top + h)],
        0,
    );
    draw::rect_r(
        s,
        IRON_L,
        ((x - w * 0.03) as i32, (top - w * 0.02) as i32, (w * 1.06) as i32, 2.max((w * 0.07) as i32)),
        0,
        1.max((w * 0.03) as i32),
    );
    for i in 0..4 {
        let rx = x + w * (0.20 + 0.2 * i as f64);
        draw::circle(s, IRON_L, (rx as i32, (top + h * 0.32) as i32), 1.max((w * 0.025) as i32), 0);
    }
    // Raeder
    for fx in [0.27, 0.73] {
        let (cx, cy) = ((x + w * fx) as i32, (y - w * 0.12) as i32);
        let rr = 3.max((w * 0.12) as i32);
        draw::circle(s, (30, 30, 38), (cx, cy), rr, 0);
        draw::circle(s, (120, 120, 134), (cx, cy), 1.max(rr - 1.max(rr / 3)), 1.max(rr / 4));
        draw::circle(s, (170, 170, 184), (cx, cy), 1.max(rr / 3), 0);
    }
}

// ============================================================= Piktogramme

/// `icon(s, name, rect, col)` - Piktogramm in rect.
pub fn icon(s: &mut Surface, name: &str, r: Rect, col: Color) {
    let (cx, cy) = (r.centerx() as f64, r.centery() as f64);
    let (rx, ry, rr, rb) = (r.x as f64, r.y as f64, r.right() as f64, r.bottom() as f64);
    let u = r.w as f64 / 10.0;
    let lw = 1.max((u * 0.9) as i32);
    match name {
        "auto" => {
            // Kreispfeil (Wiederholen) mit Play-Dreieck in der Mitte
            let rad = u * 3.8;
            let bx = Rect::new(0, 0, (rad * 2.0) as i32, (rad * 2.0) as i32).center_at((r.centerx(), r.centery()));
            draw::arc(s, col, bx, PI * 0.35, PI * 1.95, 1.max(lw));
            let a = PI * 0.35;
            let (ex, ey) = (cx + a.cos() * rad, cy - a.sin() * rad);
            draw::polygon(s, col, &[(ex + u * 1.6, ey - u * 0.2), (ex - u * 0.6, ey - u * 1.3), (ex - u * 0.2, ey + u * 1.2)], 0);
            draw::polygon(s, col, &[(cx - u * 1.0, cy - u * 1.5), (cx + u * 1.6, cy), (cx - u * 1.0, cy + u * 1.5)], 0);
        }
        "stats" => {
            let base = rb - u * 1.6;
            for (i, hh) in [3.0, 5.2, 4.0, 6.8].iter().enumerate() {
                let x = rx + u * (1.4 + i as f64 * 2.0);
                draw::rect(s, col, (x as i32, (base - hh * u) as i32, 1.max((u * 1.3) as i32), (hh * u) as i32), 0);
            }
            draw::line(s, col, (rx + u, base), (rr - u, base), 1.max(if lw > 1 { lw - 1 } else { 1 }));
        }
        "fair" => {
            let pts = [
                (cx, ry + u * 1.0),
                (cx + u * 3.6, ry + u * 2.2),
                (cx + u * 3.2, cy + u * 1.4),
                (cx, rb - u * 0.8),
                (cx - u * 3.2, cy + u * 1.4),
                (cx - u * 3.6, ry + u * 2.2),
            ];
            draw::polygon(s, col, &pts, 1.max(lw));
            draw::lines(s, col, false, &[(cx - u * 1.6, cy), (cx - u * 0.3, cy + u * 1.4), (cx + u * 1.8, cy - u * 1.4)], 1.max(lw));
        }
        "turbo" => {
            draw::polygon(
                s,
                col,
                &[
                    (cx + u * 1.0, ry + u * 0.6),
                    (cx - u * 2.6, cy + u * 0.6),
                    (cx - u * 0.2, cy + u * 0.6),
                    (cx - u * 1.0, rb - u * 0.6),
                    (cx + u * 2.6, cy - u * 0.8),
                    (cx + u * 0.2, cy - u * 0.8),
                ],
                0,
            );
        }
        "dice" => {
            let bx = Rect::new(0, 0, (u * 7.2) as i32, (u * 7.2) as i32).center_at((r.centerx(), r.centery()));
            draw::rect_r(s, col, bx, 1.max(lw), 2.max((u * 1.4) as i32));
            let pr = 1.max((u * 0.75) as i32);
            for (dx, dy) in [(-1, -1), (1, 1), (0, 0), (1, -1), (-1, 1)] {
                draw::circle(s, col, ((cx + dx as f64 * u * 1.9) as i32, (cy + dy as f64 * u * 1.9) as i32), pr, 0);
            }
        }
        "close" => {
            let d = u * 2.6;
            draw::line(s, col, (cx - d, cy - d), (cx + d, cy + d), 2.max(lw));
            draw::line(s, col, (cx - d, cy + d), (cx + d, cy - d), 2.max(lw));
        }
        "check" => {
            draw::lines(s, col, false, &[(cx - u * 3.0, cy), (cx - u * 0.8, cy + u * 2.4), (cx + u * 3.2, cy - u * 2.6)], 2.max((lw as f64 * 1.4) as i32));
        }
        "cross" => {
            let d = u * 2.6;
            let w = 2.max((lw as f64 * 1.4) as i32);
            draw::line(s, col, (cx - d, cy - d), (cx + d, cy + d), w);
            draw::line(s, col, (cx - d, cy + d), (cx + d, cy - d), w);
        }
        "left" | "right" => {
            let sx = if name == "left" { -1.0 } else { 1.0 };
            draw::polygon(
                s,
                col,
                &[(cx + sx * u * 2.0, cy), (cx - sx * u * 1.4, cy - u * 2.6), (cx - sx * u * 1.4, cy + u * 2.6)],
                0,
            );
        }
        "gem" => {
            let img = gem(4.max((r.w as f64 * 0.9) as i32), 0);
            s.blr(&img, img.get_rect().center_at((r.centerx(), r.centery())));
        }
        "mine" => {
            let img = mine(4.max((r.w as f64 * 0.9) as i32), true);
            s.blr(&img, img.get_rect().center_at((r.centerx(), r.centery())));
        }
        _ => {}
    }
}
