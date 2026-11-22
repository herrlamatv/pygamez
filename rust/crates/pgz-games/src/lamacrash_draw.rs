//! lamacrash_draw.rs
//! =================
//! Zeichen-Bausteine von Lama Crash - uebertragen aus
//! `games/lamacrash_draw.py` (nur Primitive, keine Bilddateien): Ballon in
//! 12 Aufblas-Stufen (Huelle mit Bahnen, Naehten, Falten, Glanz, Schuerze,
//! Brenner-Gestell und Seilen), Korb (Rueckwand / Vorderwand), Brennerflamme
//! in 6 Flacker-Stufen, Fallschirm (+ Oeffnungsstufen), Geldsack,
//! Gummifetzen, Knall-Stern, Verlaufs-Pillen, Piktogramme, Hilfslinien und
//! Kurve, Lichthof, Mini-Kurve fuers Fairness-Fenster, Mondfahne und die
//! zusammengesetzten Figuren (Ballon mit Korb und Lamas, Lama am Schirm,
//! Geldsack am Schirm, purzelndes Lama).
//!
//! Alles Teure wird einmal 2- bis 4-fach ueberabgetastet gezeichnet, weich
//! verkleinert und gecacht. Die Ausdruecke stehen in derselben Reihenfolge
//! wie in Python, damit jede Kommazahl gleich rundet; Farben mischen
//! ganzzahlig (`lerp_i` mit Pythons `//`), Streuung nur ueber `hash`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::{PI, TAU};
use std::thread::LocalKey;

use pgz_engine::prelude::*;

use crate::lamacrash_logic as lg;
use crate::lamacrossing_draw as lcd;

/// RGB als ganze Zahlen (wie die Python-Tupel).
pub type Rgb = (i32, i32, i32);
type P = (f64, f64);

/// Aufblas-Stufen: ab diesen Multiplikatoren (x100) wird der Ballon praller.
pub const INFL_AT: [i64; 12] = [100, 110, 125, 150, 200, 300, 500, 1000, 3000, 10000, 100000, 1000000];
/// Hoechste Aufblas-Stufe.
pub const INFL_MAX: i32 = INFL_AT.len() as i32 - 1;

/// Gummi der Huelle.
pub const RUBBER: Rgb = (232, 52, 76);
/// Gummi dunkel.
pub const RUBBER_D: Rgb = (150, 26, 46);
/// Gummi sehr dunkel (Umriss).
pub const RUBBER_DD: Rgb = (84, 14, 30);
/// Gummi hell (pralle Huelle).
pub const RUBBER_L: Rgb = (255, 226, 232);
/// Korbgeflecht.
pub const WICKER: Rgb = (176, 122, 62);
/// Korbgeflecht dunkel.
pub const WICKER_D: Rgb = (112, 72, 34);
/// Korbgeflecht sehr dunkel.
pub const WICKER_DD: Rgb = (70, 42, 20);
/// Korbgeflecht hell.
pub const WICKER_L: Rgb = (222, 176, 108);
/// Leder (Randwulst, Pfosten).
pub const LEATHER: Rgb = (98, 56, 34);
/// Leder hell.
pub const LEATHER_L: Rgb = (160, 104, 64);
/// Seile.
pub const ROPE: Rgb = (90, 70, 56);
/// Metall (Brenner).
pub const METAL: Rgb = (138, 144, 162);
/// Metall hell.
pub const METAL_L: Rgb = (216, 222, 234);
/// Metall dunkel.
pub const METAL_D: Rgb = (62, 66, 80);
/// Gold (Muenzen).
pub const GOLD: Rgb = (232, 190, 92);
/// Gold hell.
pub const GOLD_L: Rgb = (255, 236, 160);
/// Gold dunkel.
pub const GOLD_D: Rgb = (160, 112, 34);
/// Jute (Geldsack).
pub const BURLAP: Rgb = (196, 150, 88);
/// Jute hell.
pub const BURLAP_L: Rgb = (230, 196, 134);
/// Jute dunkel.
pub const BURLAP_D: Rgb = (128, 88, 46);
/// Weiss.
pub const WHITE: Rgb = (255, 255, 255);
/// Schwarz.
pub const BLACK: Rgb = (0, 0, 0);

/// Sichtbare Bahnen der Huelle (Vorderseite).
pub const GORES: i32 = 8;
/// Lamas im Korb: diese Sprite-Zeile (in %) liegt auf der vorderen Korbkante.
pub const RIDER_RIM: i32 = 48;
/// Saum des Fallschirms in Prozent der Hoehe (Leinen beginnen hier).
pub const CHUTE_HEM: i32 = 38;
/// Flackern der Brennerflamme: (Hoehe %, Spitze seitlich %, Breite %, Zunge -1/0/1).
pub const FLAME_STEPS: [(i32, i32, i32, i32); 6] =
    [(100, 0, 100, 0), (93, -6, 97, 1), (97, 5, 103, -1), (89, -3, 95, 1), (100, 7, 99, -1), (95, -5, 104, 0)];
/// Knall-Stern: Groesse (%) je Schritt.
pub const POP_SCALE: [i32; 6] = [52, 84, 100, 100, 97, 93];
/// Knall-Stern: Deckkraft je Schritt.
pub const POP_ALPHA: [i32; 6] = [255, 255, 255, 215, 140, 64];

// ============================================================= Grundlagen

type Store = RefCell<HashMap<String, Surface>>;

thread_local! {
    /// `_cache` - grosse Teile: Ballon, Korb, Sprites.
    static CACHE: Store = RefCell::new(HashMap::new());
    /// `_small` - Texte und Pillen (wechseln oft, eigener kleiner Speicher).
    static SMALL: Store = RefCell::new(HashMap::new());
    /// Die Fetzen-Tupel von `shreds` (in Python ebenfalls in `_cache`).
    static TUPLES: RefCell<HashMap<String, Vec<Surface>>> = RefCell::new(HashMap::new());
    /// `_cache[("work",)]` - Arbeitsflaeche der Kurve (waechst nur).
    static WORK: RefCell<Option<Surface>> = const { RefCell::new(None) };
}

/// `_cached(key, build, store)` - Flaeche aus dem Speicher holen oder einmal
/// bauen (ab 400 Eintraegen wird der Speicher geleert).
fn cached_in(store: &'static LocalKey<Store>, key: String, build: impl FnOnce() -> Surface) -> Surface {
    if let Some(s) = store.with(|c| c.borrow().get(&key).cloned()) {
        return s;
    }
    store.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 400 {
            c.clear();
        }
    });
    let surf = build();
    store.with(|c| c.borrow_mut().insert(key, surf.clone()));
    surf
}

fn cached(key: String, build: impl FnOnce() -> Surface) -> Surface {
    cached_in(&CACHE, key, build)
}

fn cached_small(key: String, build: impl FnOnce() -> Surface) -> Surface {
    cached_in(&SMALL, key, build)
}

/// `clear_cache()`
pub fn clear_cache() {
    CACHE.with(|c| c.borrow_mut().clear());
    SMALL.with(|c| c.borrow_mut().clear());
    TUPLES.with(|c| c.borrow_mut().clear());
    WORK.with(|c| *c.borrow_mut() = None);
}

/// Pythons `max(a, b)` fuer Kommazahlen (bei Gleichstand das erste).
fn fmax(a: f64, b: f64) -> f64 {
    if b > a {
        b
    } else {
        a
    }
}

/// Pythons `min(a, b)` fuer Kommazahlen (bei Gleichstand das erste).
fn fmin(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}

/// Pythons `a // b` fuer `i32`.
fn fd(a: i32, b: i32) -> i32 {
    py::fdiv32(a, b)
}

/// `_ri(v)` - kaufmaennisch runden (x.5 nach oben).
fn ri(v: f64) -> i32 {
    (v + 0.5).floor() as i32
}

/// `_lerp_i(c1, c2, num, den)` - `a + (b - a) * num // den` (Pythons `//`).
fn lerp_i(c1: Rgb, c2: Rgb, num: i64, den: i64) -> Rgb {
    let den = 1.max(den);
    let f = |a: i32, b: i32| (a as i64 + py::fdiv((b - a) as i64 * num, den)) as i32;
    (f(c1.0, c2.0), f(c1.1, c2.1), f(c1.2, c2.2))
}

/// `_rgb(col)` - nur die drei Farbkanaele.
fn rgb_of(col: impl Col) -> Rgb {
    let c = col.c();
    (c.r as i32, c.g as i32, c.b as i32)
}

/// `_rgba(col, a)` - Farbe mit Deckkraft `max(0, min(255, int(a)))`.
fn rgba_of(col: Rgb, a: f64) -> (i32, i32, i32, i32) {
    (col.0, col.1, col.2, 0.max(255.min(a as i32)))
}

/// `_hash(i)` - Ganzzahl-Streuwert 0..2^32-1 (deterministisch).
fn hash(i: i64) -> i64 {
    let v = (i as i128 * 2_654_435_761 + 40_503).rem_euclid(4_294_967_296) as u64;
    let v = (v ^ (v >> 15)).wrapping_mul(2_246_822_519) % 4_294_967_296;
    (v ^ (v >> 13)) as i64
}

/// `_layer(w, h)` - leere, durchsichtige Flaeche.
fn layer(w: i32, h: i32) -> Surface {
    pg::surface_alpha(1.max(w), 1.max(h))
}

/// `_mask(w, h, pts)` - Stanzmaske fuer BLEND_RGBA_MIN.
fn mask(w: i32, h: i32, pts: &[P]) -> Surface {
    let mut m = layer(w, h);
    m.fill_c((255, 255, 255, 0));
    draw::polygon(&mut m, (255, 255, 255, 255), pts, 0);
    m
}

/// `_stroke(s, col, pts, lw)` - dicke Linie mit runden Gelenken und Enden.
fn stroke(s: &mut Surface, col: impl Col, pts: &[P], lw: i32) {
    let lw = 1.max(lw);
    draw::lines(s, col, false, pts, lw);
    if lw > 2 {
        for &p in pts {
            draw::circle(s, col, p, lw as f64 / 2.0, 0);
        }
    }
}

/// `_radial(s, col, c, r_in, r_out, a_in, a_out, steps, power)` - stufiger
/// Radialverlauf von aussen nach innen (Alpha ersetzen).
#[allow(clippy::too_many_arguments)]
fn radial(s: &mut Surface, col: Rgb, c: P, r_in: f64, r_out: f64, a_in: i32, a_out: i32, steps: i32, power: f64) {
    for i in (0..=steps).rev() {
        let f = i as f64 / steps as f64;
        let a = a_in as f64 + (a_out - a_in) as f64 * py::pow(f, power);
        draw::circle(s, rgba_of(col, a), c, 1.max((r_in + (r_out - r_in) * f) as i32), 0);
    }
}

/// `_ellipse_pts(cx, cy, rx, ry, rot, n)` - gedrehte Ellipse als Punktliste.
fn ellipse_pts(cx: f64, cy: f64, rx: f64, ry: f64, rot: f64, n: i32) -> Vec<P> {
    let (ca, sa) = (rot.cos(), rot.sin());
    let mut pts = Vec::with_capacity(n as usize);
    for i in 0..n {
        let a = TAU * i as f64 / n as f64;
        let (x, y) = (rx * a.cos(), ry * a.sin());
        pts.push((cx + x * ca - y * sa, cy + x * sa + y * ca));
    }
    pts
}

/// `_arc_band(cx, cy, r_out, thick, a0, a1, n)` - Sichel entlang eines
/// Kreisbogens, in der Mitte thick dick, an den Enden spitz.
fn arc_band(cx: f64, cy: f64, r_out: f64, thick: f64, a0: f64, a1: f64, n: i32) -> Vec<P> {
    let mut outer = Vec::with_capacity(n as usize + 1);
    let mut inner = Vec::with_capacity(n as usize + 1);
    for i in 0..=n {
        let f = i as f64 / n as f64;
        let a = a0 + (a1 - a0) * f;
        let t = thick * (PI * f).sin();
        let (ca, sa) = (a.cos(), a.sin());
        outer.push((cx + r_out * ca, cy + r_out * sa));
        inner.push((cx + (r_out - t) * ca, cy + (r_out - t) * sa));
    }
    outer.extend(inner.iter().rev());
    outer
}

/// `_sparkle(s, col, x, y, r)` - vierzackiges Funkeln.
fn sparkle(s: &mut Surface, col: impl Col, x: f64, y: f64, r: f64) {
    let q = r * 0.26;
    draw::polygon(
        s,
        col,
        &[(x, y - r), (x + q, y - q), (x + r, y), (x + q, y + q), (x, y + r), (x - q, y + q), (x - r, y), (x - q, y - q)],
        0,
    );
}

/// `_burst(cx, cy, r_out, r_in, spikes, rot, seed, jit)` - gezackter
/// Comic-Stern, Zackenlaengen ueber `hash` leicht unregelmaessig.
#[allow(clippy::too_many_arguments)]
fn burst(cx: f64, cy: f64, r_out: f64, r_in: f64, spikes: i32, rot: f64, seed: i64, jit: f64) -> Vec<P> {
    let mut pts = Vec::with_capacity(spikes as usize * 2);
    for j in 0..spikes * 2 {
        let a = rot + PI * j as f64 / spikes as f64;
        let rr = if j % 2 == 0 {
            r_out * (1.0 - jit * (hash(seed + j as i64) % 100) as f64 / 100.0)
        } else {
            r_in * (1.0 - 0.12 * (hash(seed + j as i64 * 7) % 100) as f64 / 100.0)
        };
        pts.push((cx + a.cos() * rr, cy + a.sin() * rr));
    }
    pts
}

/// `_smooth(big, w, h)` - weich verkleinern.
fn smooth(big: &Surface, w: i32, h: i32) -> Surface {
    transform::smoothscale(big, 1.max(w), 1.max(h))
}

fn ck(c: Rgb) -> String {
    format!("{},{},{}", c.0, c.1, c.2)
}

// ============================================================= Groessen

/// `infl_level(m100)` - Aufblas-Stufe 0 .. INFL_MAX zum Multiplikator.
pub fn infl_level(m100: i64) -> i32 {
    let mut lvl = 0;
    for (i, &lo) in INFL_AT.iter().enumerate() {
        if m100 >= lo {
            lvl = i as i32;
        }
    }
    lvl
}

/// `basket_w(base)` - Korbbreite zur Grundbreite des Ballons.
pub fn basket_w(base: i32) -> i32 {
    8.max(fd(base * 9, 20))
}

/// `llama_size(base)` - Groesse der Lamas im Korb (Lama-Sprite n x n).
pub fn llama_size(base: i32) -> i32 {
    8.max(fd(basket_w(base) * 3, 4))
}

/// `balloon_size(base, level)` - (w, h) der Ballon-Flaeche: 4 % breiter je
/// Aufblas-Stufe, die Huelle wird dabei runder, die Seilzone bleibt.
pub fn balloon_size(base: i32, level: i32) -> (i32, i32) {
    let level = 0.max(INFL_MAX.min(level));
    let w = 12.max(fd(base * (100 + 4 * level), 100));
    (w, fd(w * (118 - level), 100) + fd(base * 9, 25))
}

/// `_basket_dims(w)` - (Hoehe, Randwulst, Tiefe der Vorderkante).
fn basket_dims(w: i32) -> (i32, i32, i32) {
    let h = 4.max(fd(w * 2, 3));
    let t = 2.max(fd(h * 14, 100));
    let e = (t * 2).max(fd(h * 30, 100));
    (h, t, e)
}

/// `_rig(base)` - Masse unter der Huelle in Pixeln ueber dem Korbrand:
/// (Schuerzen-Unterkante, Schuerzenhoehe, halbe Halsbreite, halbe
/// Mundbreite, Brennerrahmen, Brennerhoehe, halbe Brennerbreite,
/// Flammenbreite).
fn rig(base: i32) -> (i32, i32, i32, i32, i32, i32, i32, i32) {
    let bw = basket_w(base);
    let n = llama_size(base);
    let (_bh, t, e) = basket_dims(bw);
    let ears = fd(n * RIDER_RIM, 100) - (e - fd(t, 2));
    let frame = 3.max(ears + 2.max(fd(base, 30)));
    let burner_h = 3.max(fd(base, 20));
    let burner_hw = 2.max(fd(bw * 11, 100));
    let mouth = (frame + burner_h + 4).max(fd(base * 9, 25));
    let skirt = 2.max(fd(base * 7, 100));
    let neck_hw = 2.max(fd(bw * 34, 100));
    let mouth_hw = 3.max(fd(bw * 40, 100));
    let flame_w = 4.max(fd(bw * 9, 25));
    (mouth, skirt, neck_hw, mouth_hw, frame, burner_h, burner_hw, flame_w)
}

/// `_rider_foot(rim_y, base)` - y der Lama-Fuesse im Korb.
fn rider_foot(rim_y: i32, base: i32) -> i32 {
    let n = llama_size(base);
    let (_bh, t, e) = basket_dims(basket_w(base));
    rim_y + e - fd(t, 2) + n - fd(n * RIDER_RIM, 100)
}

// ============================================================= Ballon

/// `_profile(R, L, m, level)` - halbe Huellenbreite als Liste (dy, r) vom
/// Pol ueber den Aequator bis zum Hals.
fn profile(r: f64, l: f64, m: f64, level: i32) -> Vec<P> {
    let f = 0.max(INFL_MAX.min(level)) as f64 / INFL_MAX as f64;
    let bulge = 0.92 * py::pow(f, 1.15);
    let p = 1.32 - 0.36 * f;
    let mut out = Vec::with_capacity(41);
    for i in 0..18 {
        let a = PI / 2.0 * i as f64 / 18.0;
        out.push((-r * a.cos(), r * a.sin()));
    }
    for i in 0..23 {
        let u = i as f64 / 22.0;
        let g0 = py::pow(fmax(0.0, (u * PI / 2.0).cos()), p);
        let g1 = fmax(0.0, 1.0 - u * u).sqrt();
        out.push((l * u, m + (r - m) * (g0 + (g1 - g0) * bulge)));
    }
    out
}

/// `_gore_bounds()` - Bahnkanten als Laengengrade -pi/2 .. pi/2, um eine
/// halbe Bahn gedreht.
fn gore_bounds() -> Vec<f64> {
    let mut out = vec![-PI / 2.0];
    for k in 0..GORES {
        out.push(-PI / 2.0 + (k as f64 + 0.5) * PI / GORES as f64);
    }
    out.push(PI / 2.0);
    out
}

/// `_stripe_col(stripe)` - Bahnfarbe mit Kontrast zu RUBBER.
fn stripe_col(stripe: Rgb) -> Rgb {
    let c = stripe;
    if (c.0 - RUBBER.0).abs() + (c.1 - RUBBER.1).abs() + (c.2 - RUBBER.2).abs() < 150 {
        return lerp_i(c, (255, 246, 232), 82, 100);
    }
    c
}

/// `balloon(base, level, stripe)` - Huelle + Schuerze + Brenner-Gestell +
/// Seile (gecacht), Flaeche `balloon_size`.
pub fn balloon(base: i32, level: i32, stripe: impl Col) -> Surface {
    let level = 0.max(INFL_MAX.min(level));
    let stripe = stripe_col(rgb_of(stripe));
    cached(format!("balloon|{base}|{level}|{}", ck(stripe)), || build_balloon(base, level, stripe))
}

fn build_balloon(base: i32, level: i32, stripe: Rgb) -> Surface {
    let (w, h) = balloon_size(base, level);
    let (mouth, skirt, neck_hw, mouth_hw, frame, burner_h, burner_hw, _fw) = rig(base);
    let bw = basket_w(base);
    let ss = 3;
    let (ww, hh) = (w * ss, h * ss);
    let cx = ww as f64 / 2.0;
    let r = ((w - 2) * ss) as f64 / 2.0;
    let y_eq = ss as f64 + r;
    let y_mouth = (h - mouth) * ss;
    let y_neck = y_mouth - skirt * ss;
    let l = fmax(r * 0.5, y_neck as f64 - y_eq);
    let m = neck_hw * ss;
    let mb = mouth_hw * ss;
    let (mf, mbf) = (m as f64, mb as f64);
    let prof = profile(r, l, mf, level);
    let right: Vec<P> = prof.iter().map(|&(dy, rr)| (cx + rr, y_eq + dy)).collect();
    let left: Vec<P> = prof.iter().map(|&(dy, rr)| (cx - rr, y_eq + dy)).collect();
    let mut sil = right.clone();
    sil.extend(left.iter().rev());
    let lift = (level * 2) as i64;
    let red = lerp_i(RUBBER, RUBBER_L, lift, 100);
    let st = lerp_i(stripe, WHITE, lift, 100);
    let outline = lerp_i(RUBBER_DD, BLACK, 25, 100);
    let bounds = gore_bounds();

    // --- Huelle: Bahnen
    let mut env = layer(ww, hh);
    for j in 0..bounds.len() - 1 {
        let (s0, s1) = (bounds[j].sin(), bounds[j + 1].sin());
        let mut poly: Vec<P> = prof.iter().map(|&(dy, rr)| (cx + rr * s0, y_eq + dy)).collect();
        poly.extend(prof.iter().rev().map(|&(dy, rr)| (cx + rr * s1, y_eq + dy)));
        draw::polygon(&mut env, if j % 2 == 0 { red } else { st }, &poly, 0);
    }
    // Naehte (bei praller Huelle glaenzen sie gespannt)
    let mut seam = layer(ww, hh);
    let sw = 2.max(fd(ss * 2, 3));
    for &b in &bounds[1..bounds.len() - 1] {
        let sb = b.sin();
        let line: Vec<P> = prof[2..].iter().map(|&(dy, rr)| (cx + rr * sb, y_eq + dy)).collect();
        draw::lines(&mut seam, (70, 10, 28, 84), false, &line, sw);
        if level >= 3 {
            let shine: Vec<P> = line[3..].iter().map(|&(x, y)| (x - sw as f64, y)).collect();
            draw::lines(&mut seam, (255, 240, 244, 150.min(14 + level * 12)), false, &shine, 1.max(fd(sw, 2)));
        }
    }
    // Falten der noch schlaffen Huelle (nur Stufe 0..2)
    for j in 0..(3 - 3.min(level)) {
        let phi = [-0.42, 0.18, 0.62][j as usize];
        let mut pts: Vec<P> = Vec::new();
        for (i, &(dy, rr)) in prof.iter().enumerate() {
            if dy >= l * 0.66 {
                let wob = 0.05 * (i as f64 * 1.7 + j as f64).sin();
                pts.push((cx + rr * (phi + wob).sin(), y_eq + dy));
            }
        }
        if pts.len() >= 2 {
            draw::lines(&mut seam, (60, 8, 24, 70), false, &pts, ss);
        }
    }
    env.bl(&seam, (0, 0));
    // Schatten: Licht von links oben, rechts unten dunkel
    let mut shade = layer(ww, hh);
    radial(&mut shade, (58, 6, 24), (cx - r * 0.36, y_eq - r * 0.42), r * 0.10, r * 2.40, 0, 196 - level * 6, 30, 1.45);
    env.bl(&shade, (0, 0));
    // Gegenlicht am rechten/unteren Rand (Gummi spiegelt)
    let mut bounce = layer(ww, hh);
    draw::polygon(&mut bounce, (255, 150, 160, 40 + level * 4), &sil, 0);
    let inner_sil: Vec<P> = sil.iter().map(|&(x, y)| (x - r * 0.075, y - r * 0.05)).collect();
    draw::polygon(&mut bounce, (0, 0, 0, 0), &inner_sil, 0);
    env.bl(&bounce, (0, 0));
    // Glanz
    let mut hl = layer(ww, hh);
    let amax = 54 + level * 9;
    for i in (1..=10).rev() {
        let f = i as f64 / 10.0;
        let a = amax as f64 * py::pow((11 - i) as f64 / 10.0, 1.6);
        let pts = ellipse_pts(cx - r * 0.40, y_eq - r * 0.46, r * 0.52 * f, r * 0.40 * f, -0.62, 28);
        draw::polygon(&mut hl, (255, 240, 244, a as i32), &pts, 0);
    }
    env.bl(&hl, (0, 0));
    let mut hl = layer(ww, hh); // scharfe Glanzformen darueber
    let lv = level as f64;
    draw::polygon(
        &mut hl,
        (255, 255, 255, 168 + level * 7),
        &arc_band(cx, y_eq, r * 0.88, r * (0.085 + 0.005 * lv), PI * 1.10, PI * (1.34 + 0.008 * lv), 16),
        0,
    );
    let pa = PI * (1.40 + 0.006 * lv);
    let dot = (cx + r * 0.82 * pa.cos(), y_eq + r * 0.82 * pa.sin());
    draw::circle(&mut hl, (255, 255, 255, 190 + level * 5), dot, r * (0.030 + 0.0015 * lv), 0);
    if level >= 4 {
        // gespannte Haut: Randlicht links unten am Bauch
        let k = level - 3;
        let kf = k as f64;
        let mut rim: Vec<P> = Vec::new();
        let mut inner: Vec<P> = Vec::new();
        let sel: Vec<P> = prof.iter().copied().filter(|&(dy, _)| 0.0 <= dy && dy <= l * (0.40 + 0.03 * kf)).collect();
        for (i, &(dy, rr)) in sel.iter().enumerate() {
            let t = (PI * i as f64 / 1.max(sel.len() as i32 - 1) as f64).sin();
            rim.push((cx - rr * 0.955, y_eq + dy));
            inner.push((cx - rr * (0.955 - (0.03 + 0.004 * kf) * t), y_eq + dy));
        }
        if rim.len() >= 2 {
            let mut poly = rim.clone();
            poly.extend(inner.iter().rev());
            draw::polygon(&mut hl, (255, 236, 240, 50 + 14 * k), &poly, 0);
        }
    }
    if level >= 6 {
        // Spannungs-Streifen neben dem Glanz
        for j in 0..(1 + fd(level - 6, 2)) {
            let jf = j as f64;
            draw::polygon(
                &mut hl,
                (255, 255, 255, 70 + 10 * j),
                &arc_band(cx, y_eq, r * (0.74 - 0.07 * jf), r * 0.022, PI * (1.17 + 0.02 * jf), PI * (1.31 - 0.01 * jf), 10),
                0,
            );
        }
    }
    if level >= 8 {
        // zweiter Lichtreflex rechts oben
        draw::polygon(
            &mut hl,
            (255, 250, 252, 60 + 12 * (level - 8)),
            &arc_band(cx, y_eq, r * 0.86, r * 0.05, PI * 1.66, PI * 1.80, 10),
            0,
        );
    }
    if level >= 10 {
        sparkle(&mut hl, (255, 255, 255, 230), dot.0 + r * 0.09, dot.1 + r * 0.05, r * 0.085);
    }
    env.bl(&hl, (0, 0));
    env.bl_flags(&mask(ww, hh, &sil), (0, 0), BLEND_RGBA_MIN);
    let olw = 2.max(fd(ss * 4, 3));
    draw::lines(&mut env, outline, false, &right, olw);
    draw::lines(&mut env, outline, false, &left, olw);
    let mut big = env; // Schuerze, Seile, Brenner kommen dazu

    // --- Schuerze (Bahnen setzen sich dunkler fort) + Ring am Hals
    let yn = y_neck - ss;
    let (ynf, ymf) = (yn as f64, y_mouth as f64);
    let skx = |y: f64, sb: f64| -> f64 {
        let t = (y - ynf) / fmax(1.0, (y_mouth - yn) as f64);
        cx + (mf + (mb - m) as f64 * t) * sb
    };
    for j in 0..bounds.len() - 1 {
        let (s0, s1) = (bounds[j].sin(), bounds[j + 1].sin());
        let mid = (s0 + s1) / 2.0;
        let col = lerp_i(if j % 2 == 0 { red } else { st }, (40, 8, 20), 28 + (24.0 * (mid + 1.0)) as i64, 100);
        draw::polygon(
            &mut big,
            col,
            &[(skx(ynf, s0), ynf), (skx(ymf, s0), ymf), (skx(ymf, s1), ymf), (skx(ynf, s1), ynf)],
            0,
        );
    }
    draw::line(&mut big, outline, (cx - mf, ynf), (cx - mbf, ymf), olw);
    draw::line(&mut big, outline, (cx + mf, ynf), (cx + mbf, ymf), olw);
    let ring = (ss * 2).max(fd(skirt * ss, 3));
    draw::rect_r(
        &mut big,
        outline,
        ((cx - mf - ss as f64) as i32, yn - fd(ring, 2), 2 * m + 2 * ss, ring),
        0,
        fd(ring, 2),
    );
    let yr = (yn - fd(ring, 4)) as f64;
    draw::line(&mut big, lerp_i(red, WHITE, 30, 100), (cx - mf * 0.75, yr), (cx - mf * 0.15, yr), 1.max(fd(ss, 2)));
    // Mundoeffnung (dunkel) mit hellem Saum vorn
    let ry = fmax(ss as f64 * 1.5, mbf * 0.2);
    draw::ellipse(&mut big, (34, 10, 18), (cx - mbf, ymf - ry, 2 * mb, 2.0 * ry), 0);
    let hem: Vec<P> = (0..13)
        .map(|i| (cx + mbf * (PI * i as f64 / 12.0).cos(), ymf + ry * (PI * i as f64 / 12.0).sin()))
        .collect();
    stroke(&mut big, lerp_i(red, (40, 8, 20), 20, 100), &hem, ss);

    // --- Seile, Brennerrahmen, Brenner
    let rlw = ss.max(fd(ss * base, 100));
    let yb = hh + ss;
    let ybf = yb as f64;
    let (cl, cr) = (cx - (bw * ss) as f64 / 2.0, cx + (bw * ss) as f64 / 2.0);
    let il = (cx - mbf * 0.40, ymf + ry * 0.7);
    let ir = (cx + mbf * 0.40, ymf + ry * 0.7);
    let rope_d = lerp_i(ROPE, BLACK, 35, 100);
    draw::line(&mut big, rope_d, il, (cl, ybf), rlw);
    draw::line(&mut big, rope_d, ir, (cr, ybf), rlw);
    let yf = hh - frame * ss;
    let yff = yf as f64;
    let tf = (yff - il.1) / fmax(1.0, ybf - il.1);
    let xl = il.0 + (cl - il.0) * tf;
    let xr = ir.0 + (cr - ir.0) * tf;
    let bar = ss.max(fd(ss * base, 75));
    let barf = bar as f64;
    draw::rect_r(&mut big, METAL_D, (xl as i32, (yff - barf / 2.0) as i32, (xr - xl) as i32, bar), 0, 1.max(fd(bar, 2)));
    draw::line(&mut big, METAL, (xl + barf, yff - barf / 4.0), (xr - barf, yff - barf / 4.0), 1.max(fd(bar, 3)));
    let bhw = burner_hw * ss;
    let bt = yf - burner_h * ss;
    let body = Rect::new((cx - bhw as f64) as i32, bt, 2 * bhw, yf - bt + fd(bar, 2));
    draw::rect_r(&mut big, METAL_D, body, 0, ss * 2);
    draw::rect_r(&mut big, METAL, (body.x + ss, body.y + ss, fd(body.w * 11, 20), body.h - ss * 2), 0, ss);
    draw::line(
        &mut big,
        METAL_L,
        (body.x + ss * 2, body.y + ss * 2),
        (body.x + ss * 2, body.bottom() - ss * 2),
        1.max(fd(ss * 2, 3)),
    );
    for fy in [0.38, 0.68] {
        let yy = body.y as f64 + body.h as f64 * fy;
        draw::line(&mut big, METAL_D, ((body.x + ss) as f64, yy), ((body.right() - ss) as f64, yy), 1.max(fd(ss * 2, 3)));
    }
    draw::ellipse(&mut big, (28, 28, 36), (body.x + ss, body.y - ss, body.w - 2 * ss, ss * 2), 0);
    draw::line(&mut big, ROPE, (cx - mbf, ymf), (cl, ybf), rlw);
    draw::line(&mut big, ROPE, (cx + mbf, ymf), (cr, ybf), rlw);
    smooth(&big, w, h)
}

// ============================================================= Korb

/// `basket(w, front)` - Korb w x (w*2//3): front=false Rueckwand +
/// Innenraum, front=true Vorderwand mit Flechtmuster, Randwulst, Pfosten.
pub fn basket(w: i32, front: bool) -> Surface {
    let w = 8.max(w);
    cached(format!("basket|{w}|{front}"), || {
        let (bh, t, e) = basket_dims(w);
        let ss = 3;
        let mut big = layer(w * ss, bh * ss);
        if front {
            basket_front(&mut big, w * ss, bh * ss, t * ss, e * ss, ss);
        } else {
            basket_back(&mut big, w * ss, bh * ss, t * ss, e * ss, ss);
        }
        smooth(&big, w, bh)
    })
}

/// `_basket_back(s, W, H, T, E, ss)` - Innenseite der Rueckwand + Randwulst.
fn basket_back(s: &mut Surface, ww: i32, hh: i32, tt: i32, ee: i32, ss: i32) {
    let (x0, x1) = ((tt as f64 * 0.5) as i32, (ww as f64 - tt as f64 * 0.5) as i32);
    let (y0, y1) = ((tt as f64 * 0.5) as i32, hh.min(ee + tt));
    for y in y0..y1 {
        s.fill_r(lerp_i((100, 66, 36), (44, 26, 12), (y - y0) as i64, (y1 - y0) as i64), (x0, y, x1 - x0, 1));
    }
    let step = (ss * 3).max(tt);
    let mut y = y0 + fd(step, 2);
    while y < y1 {
        draw::line(s, (36, 22, 10), (x0, y), (x1, y), 1.max(fd(ss, 2)));
        let off = py::pmod32(fd(y, step), 2) * step;
        let mut x = x0 + off;
        while x < x1 {
            let yy = y - fd(step, 2) + ss;
            draw::line(s, (128, 88, 50), (x + ss, yy), (x + step - ss, yy), 1.max(fd(ss, 2)));
            x += step * 2;
        }
        y += step;
    }
    let rim = lerp_i(LEATHER, BLACK, 28, 100);
    draw::rect_r(s, rim, (0, 0, ww, tt), 0, fd(tt, 2));
    draw::line(s, LEATHER, (fd(tt, 2), fd(tt, 3)), (ww - fd(tt, 2), fd(tt, 3)), 1.max(fd(ss, 2)));
}

/// `_basket_front(s, W, H, T, E, ss)` - Vorderwand: Ziegel-Flechtmuster,
/// Schatten rechts, Bodenleiste, Rand vorn.
fn basket_front(s: &mut Surface, ww: i32, hh: i32, tt: i32, ee: i32, ss: i32) {
    let inset = fd(ww * 5, 100);
    let rad = (ss * 2).max(fd(hh - ee, 4));
    let mut wall: Vec<P> = vec![(0.0, ee as f64), (ww as f64, ee as f64)];
    for i in 0..7 {
        let a = PI / 2.0 * i as f64 / 6.0;
        wall.push(((ww - inset - rad) as f64 + rad as f64 * a.cos(), (hh - rad) as f64 + rad as f64 * a.sin()));
    }
    for i in 0..7 {
        let a = PI / 2.0 + PI / 2.0 * i as f64 / 6.0;
        wall.push(((inset + rad) as f64 + rad as f64 * a.cos(), (hh - rad) as f64 + rad as f64 * a.sin()));
    }
    let mut lay = layer(ww, hh);
    lay.fill_c(WICKER_DD);
    // Geflecht: Reihen versetzter, gewoelbter Staebe
    let top = (ee + fd(tt, 2)).min(hh - ss * 2);
    let rows = 1.max(ri((hh - top) as f64 / (ss as f64 * 5.5)));
    let rh = fmax(ss as f64, (hh - top) as f64 / rows as f64);
    let sw = rh * 1.9;
    let gap = 1.max(fd(ss * 2, 3));
    let gapf = gap as f64;
    for r in 0..rows {
        let y = top as f64 + r as f64 * rh;
        let off = (r % 2) as f64 * sw / 2.0;
        let mut x = -sw + off;
        while x < ww as f64 + sw {
            let bx = (x + gapf, y + gapf, sw - (2 * gap) as f64, rh - (2 * gap) as f64);
            draw::ellipse(&mut lay, WICKER, bx, 0);
            draw::ellipse(
                &mut lay,
                WICKER_L,
                (x + (gap * 2) as f64, y + gapf * 1.5, sw - (4 * gap) as f64, (rh - (2 * gap) as f64) * 0.45),
                0,
            );
            x += sw;
        }
    }
    // Schatten: rechts und unten dunkler, links Lichtkante
    let mut sh = layer(ww, hh);
    let bands = 12;
    for i in 0..bands {
        let x = fd(ww * (40 + fd(60 * i, bands)), 100);
        sh.fill_r((30, 14, 4, 8 + 6 * i), (x, 0, ww - x, hh));
    }
    for i in 0..6 {
        let y = (hh as f64 - ((hh - ee) * (6 - i)) as f64 / 14.0) as i32;
        sh.fill_r((30, 14, 4, 10 + 6 * i), (0, y, ww, hh - y));
    }
    sh.fill_r((20, 10, 2, 90), (0, ee + fd(tt, 2), ww, ss.max(fd(tt, 2)))); // Schatten unter dem Wulst
    lay.bl(&sh, (0, 0));
    draw::line(&mut lay, WICKER_L, (ss, ee), (inset + ss, hh - rad), 1.max(ss));
    let yl = (hh as f64 - tt as f64 * 0.8) as i32;
    draw::rect(&mut lay, WICKER_DD, (0, yl, ww, (tt as f64 * 0.8) as i32 + 1), 0);
    draw::line(&mut lay, WICKER_D, (0, yl), (ww, yl), 1.max(fd(ss, 2)));
    lay.bl_flags(&mask(ww, hh, &wall), (0, 0), BLEND_RGBA_MIN);
    s.bl(&lay, (0, 0));
    let mut edge: Vec<P> = wall[1..].to_vec();
    edge.push(wall[0]);
    draw::lines(s, WICKER_DD, false, &edge, 1.max(fd(ss * 2, 3)));
    // Eckpfosten (verbinden hinteren und vorderen Rand) mit Seil-Ringen
    let pw = (tt as f64 * 0.9) as i32;
    for x in [0, ww - pw] {
        draw::rect_r(s, LEATHER, (x, 0, pw, ee + fd(tt, 2)), 0, 1.max(fd(pw, 3)));
        draw::line(s, LEATHER_L, (x + fd(pw, 3), ss), (x + fd(pw, 3), ee), 1.max(fd(ss, 2)));
        let c = (x as f64 + pw as f64 / 2.0, tt as f64 * 0.45);
        draw::circle(s, METAL_D, c, tt as f64 * 0.36, 0);
        draw::circle(s, METAL_L, c, tt as f64 * 0.36, 1.max(fd(ss * 2, 3)));
    }
    // vorderer Randwulst
    draw::rect_r(s, lerp_i(LEATHER, BLACK, 30, 100), (0, ee - fd(tt, 2) + fd(ss, 2), ww, tt), 0, fd(tt, 2));
    draw::rect_r(s, LEATHER, (0, ee - fd(tt, 2), ww, tt - fd(ss, 2)), 0, fd(tt, 2));
    draw::line(s, LEATHER_L, (fd(tt, 2), ee - fd(tt, 4)), (ww - fd(tt, 2), ee - fd(tt, 4)), 1.max(fd(ss * 2, 3)));
    // Naehte am Wulst
    let step = (ss * 4).max(tt * 2);
    let mut x = tt;
    while x < ww - tt {
        draw::line(
            s,
            lerp_i(LEATHER, BLACK, 40, 100),
            (x, ee - fd(tt, 2) + ss),
            (x - ss, ee + fd(tt, 2) - ss),
            1.max(fd(ss, 2)),
        );
        x += step;
    }
}

// ============================================================= Brenner

/// `_flame_pts(cx, by, hw, fh, sway, n)` - Flammenumriss.
fn flame_pts(cx: f64, by: f64, hw: f64, fh: f64, sway: f64, n: i32) -> Vec<P> {
    let mut right: Vec<P> = Vec::with_capacity(2 * n as usize + 2);
    let mut left: Vec<P> = Vec::with_capacity(n as usize + 1);
    for i in 0..=n {
        let u = i as f64 / n as f64;
        let wdt = hw * fmin(1.0, u * 5.0 + 0.04).sqrt() * py::pow(1.0 - u, 1.15);
        let x = cx + sway * u * u;
        let y = by - fh * u;
        right.push((x + wdt, y));
        left.push((x - wdt, y));
    }
    right.extend(left.iter().rev());
    right
}

/// `burner_flame(w, step)` - Duese + Flamme w x w*3//2 (step 0..5), Anker
/// Unterkante Mitte.
pub fn burner_flame(w: i32, step: i32) -> Surface {
    let w = 4.max(w);
    let step = py::pmod32(step, 6);
    cached(format!("flame|{w}|{step}"), || {
        let fh_img = fd(w * 3, 2);
        let ss = 4;
        let (ww, hh) = (w * ss, fh_img * ss);
        let mut big = layer(ww, hh);
        let cx = ww as f64 / 2.0;
        let (hp, sway_p, wp, tongue) = FLAME_STEPS[step as usize];
        let noz = (ss * 2).max(fd(hh * 9, 100));
        let by = hh - noz;
        let byf = by as f64;
        let fh = ((by - ss) * hp) as f64 / 100.0;
        let hw = ww as f64 * 0.36 * wp as f64 / 100.0;
        let sway = (ww * sway_p) as f64 / 100.0;
        draw::polygon(&mut big, (255, 140, 40, 64), &flame_pts(cx, byf, hw * 1.32, fh, sway, 14), 0);
        if tongue != 0 {
            let tg = tongue as f64;
            draw::polygon(
                &mut big,
                (255, 150, 44, 220),
                &flame_pts(cx + tg * hw * 0.55, byf - fh * 0.10, hw * 0.34, fh * 0.42, (tongue * ww) as f64 * 0.10, 10),
                0,
            );
        }
        draw::polygon(&mut big, (255, 118, 30, 238), &flame_pts(cx, byf, hw, fh * 0.94, sway, 14), 0);
        draw::polygon(&mut big, (255, 192, 58, 250), &flame_pts(cx, byf, hw * 0.70, fh * 0.74, sway * 0.8, 14), 0);
        draw::polygon(&mut big, (255, 242, 176), &flame_pts(cx, byf, hw * 0.42, fh * 0.50, sway * 0.5, 14), 0);
        draw::polygon(&mut big, (96, 162, 255, 236), &flame_pts(cx, byf, hw * 0.32, fh * 0.22, 0.0, 8), 0);
        draw::polygon(&mut big, (220, 238, 255), &flame_pts(cx, byf, hw * 0.14, fh * 0.12, 0.0, 6), 0);
        // Duese
        let nw = ww as f64 * 0.40;
        draw::rect_r(&mut big, METAL_D, ((cx - nw / 2.0) as i32, by, nw as i32, noz), 0, ss);
        draw::rect_r(
            &mut big,
            METAL,
            ((cx - nw / 2.0 + ss as f64) as i32, by + fd(ss, 2), (nw * 0.45) as i32, noz - ss),
            0,
            fd(ss, 2),
        );
        draw::ellipse(&mut big, (24, 24, 30), (cx - nw / 2.0 + ss as f64, by - ss, nw - (2 * ss) as f64, ss * 2), 0);
        smooth(&big, w, fh_img)
    })
}

// ============================================================= Fallschirm

/// `parachute(w, col)` - Fallschirm w x w: Kuppel mit Bahnen in col und
/// Weiss, gewellter Saum, Leinen laufen unten in der Mitte zusammen.
pub fn parachute(w: i32, col: impl Col) -> Surface {
    let w = 6.max(w);
    let col = rgb_of(col);
    cached(format!("chute|{w}|{}", ck(col)), || {
        let ss = 3;
        let nn = w * ss;
        let nf = nn as f64;
        let mut big = layer(nn, nn);
        let cx = nf / 2.0;
        let rx = nf * 0.47;
        let top = nf * 0.05;
        let hem = (nn * CHUTE_HEM) as f64 / 100.0;
        let ry = hem - top;
        let conv = (cx, nf - ss as f64 * 0.5);
        let np_ = 6;
        let bounds: Vec<f64> = (0..=np_).map(|k| -PI / 2.0 + k as f64 * PI / np_ as f64).collect();
        let white: Rgb = (250, 248, 242);
        let dark = lerp_i(col, BLACK, 45, 100);
        let mer = |sb: f64, th: f64| -> P { (cx + rx * sb * th.sin(), hem - ry * th.cos()) };
        let mut sil_top: Vec<P> = Vec::new();
        for j in 0..np_ as usize {
            let (s0, s1) = (bounds[j].sin(), bounds[j + 1].sin());
            let (x0, x1) = (cx + rx * s0, cx + rx * s1);
            let scal = rx * (s1 - s0) * 0.16;
            let mut pts: Vec<P> = (0..11).map(|i| mer(s0, PI / 2.0 * i as f64 / 10.0)).collect();
            let edge: Vec<P> =
                (1..8).map(|i| (x0 + (x1 - x0) * i as f64 / 8.0, hem - scal * (PI * i as f64 / 8.0).sin())).collect();
            pts.extend(edge.iter().copied());
            pts.extend((0..11).rev().map(|i| mer(s1, PI / 2.0 * i as f64 / 10.0)));
            let mid = (s0 + s1) / 2.0;
            let base_col = if j % 2 == 0 { col } else { white };
            draw::polygon(&mut big, lerp_i(base_col, BLACK, (8.0 + 16.0 * (mid + 1.0)) as i64, 100), &pts, 0);
            sil_top.push((x0, hem));
            sil_top.extend(edge.iter().copied());
        }
        sil_top.push((cx + rx, hem));
        // Glanz links oben, Bahnkanten
        let mut hl = layer(nn, nn);
        let band = arc_band(0.0, 0.0, 0.84, 0.16, PI * 1.14, PI * 1.40, 12);
        let shine: Vec<P> = band.iter().map(|&(x, y)| (cx + rx * x, hem + ry * y)).collect();
        draw::polygon(&mut hl, (255, 255, 255, 96), &shine, 0);
        big.bl(&hl, (0, 0));
        for &b in &bounds[1..bounds.len() - 1] {
            let sb = b.sin();
            let pts: Vec<P> = (1..11).map(|i| mer(sb, PI / 2.0 * i as f64 / 10.0)).collect();
            draw::lines(&mut big, lerp_i(dark, BLACK, 20, 100), false, &pts, 1.max(fd(ss, 2)));
        }
        let mut outline: Vec<P> = (0..11).rev().map(|i| mer(-1.0, PI / 2.0 * i as f64 / 10.0)).collect();
        outline.extend((1..11).map(|i| mer(1.0, PI / 2.0 * i as f64 / 10.0)));
        draw::lines(&mut big, dark, false, &outline, 2.max(ss));
        draw::lines(&mut big, dark, false, &sil_top, 1.max(fd(ss * 2, 3)));
        // Leinen
        let llw = 2.max(fd(ss * w, 70));
        for &b in &bounds {
            draw::line(&mut big, (74, 70, 82), (cx + rx * b.sin(), hem), conv, llw);
        }
        draw::circle(&mut big, (74, 70, 82), conv, llw, 0);
        smooth(&big, w, w)
    })
}

/// `_chute_stage(cw, col, k)` - Schirm in Oeffnungsstufe k (0..8).
fn chute_stage(cw: i32, col: Rgb, k: i32) -> Surface {
    if k >= 8 {
        return parachute(cw, col);
    }
    cached(format!("chute_k|{cw}|{}|{k}", ck(col)), || {
        let full = parachute(cw, col);
        let hem = fd(cw * CHUTE_HEM, 100);
        let f = k as f64 / 8.0;
        let sw = 2.max((cw as f64 * (0.16 + 0.80 * f)) as i32);
        let sh = 2.max(((hem + 1) as f64 * (0.82 + 0.18 * f)) as i32);
        let can = smooth(&full.subsurface(Rect::new(0, 0, cw, hem + 1)), sw, sh);
        let mut out = layer(cw, cw);
        let x0 = fd(cw - sw, 2);
        out.bl(&can, (x0, hem + 1 - sh));
        let conv = (cw as f64 / 2.0, (cw - 1) as f64);
        for fx in [0.06, 0.36, 0.64, 0.94] {
            draw::aaline(&mut out, (74, 70, 82), (x0 as f64 + sw as f64 * fx, hem as f64), conv);
        }
        out
    })
}

// ============================================================= Geldsack

/// `money_sack(n, level)` - Geldsack n x n (Anker Mitte): Jute mit Zipfel,
/// Kordel und goldener Muenze; level 0..2 = voller.
pub fn money_sack(n: i32, level: i32) -> Surface {
    let n = 6.max(n);
    let level = 0.max(2.min(level));
    cached(format!("sack|{n}|{level}"), || {
        let ss = 3;
        let nn = n * ss;
        let nf = nn as f64;
        let lv = level as f64;
        let mut big = layer(nn, nn);
        let cx = nf / 2.0;
        let brx = nf * (0.31 + 0.045 * lv);
        let bry = nf * (0.25 + 0.025 * lv);
        let bcy = nf * 0.95 - bry;
        let neck_y = nf * 0.34;
        let nhw = nf * 0.085;
        let mut right: Vec<P> = Vec::with_capacity(21);
        for i in 0..10 {
            let t = i as f64 / 10.0;
            right.push((
                cx + nhw + (brx - nhw) * py::pow((t * PI / 2.0).sin(), 1.3),
                neck_y + (bcy - neck_y) * t,
            ));
        }
        for i in 0..11 {
            let a = PI / 2.0 * i as f64 / 10.0;
            right.push((cx + brx * a.cos(), bcy + bry * a.sin()));
        }
        let mut body = right.clone();
        body.extend(right.iter().rev().map(|&(x, y)| (2.0 * cx - x, y)));
        // Muenzen oben (nur voller Sack), dahinter der Zipfel
        if level >= 2 {
            for (dx, dy, rot) in [(-0.07, 0.15, -0.3), (0.06, 0.12, 0.25), (0.0, 0.08, 0.0)] {
                let pts = ellipse_pts(cx + nf * dx, nf * dy, nf * 0.075, nf * 0.05, rot, 16);
                let low: Vec<P> = pts.iter().map(|&(x, y)| (x, y + ss as f64)).collect();
                draw::polygon(&mut big, GOLD_D, &low, 0);
                draw::polygon(&mut big, GOLD, &pts, 0);
                draw::line(
                    &mut big,
                    GOLD_L,
                    (cx + nf * (dx - 0.04), nf * (dy - 0.015)),
                    (cx + nf * (dx + 0.02), nf * (dy - 0.03)),
                    1.max(ss),
                );
            }
        }
        let tuft: [P; 8] = [
            (cx - nhw, neck_y),
            (cx - nf * 0.17, nf * 0.13),
            (cx - nf * 0.09, nf * 0.19),
            (cx - nf * 0.04, nf * 0.10),
            (cx + nf * 0.03, nf * 0.18),
            (cx + nf * 0.10, nf * 0.11),
            (cx + nf * 0.16, nf * 0.17),
            (cx + nhw, neck_y),
        ];
        draw::polygon(&mut big, BURLAP_D, &tuft, 0);
        let tuft2: Vec<P> = tuft.iter().map(|&(x, y)| (x + (cx - x) * 0.18, y + (neck_y - y) * 0.15)).collect();
        draw::polygon(&mut big, BURLAP, &tuft2, 0);
        // Koerper mit Schatten und Jute-Muster
        let mut lay = layer(nn, nn);
        draw::polygon(&mut lay, BURLAP, &body, 0);
        let mut tex = layer(nn, nn);
        let step = (ss * 2).max(fd(nn, 14));
        let mut i = -nn;
        while i < nn {
            draw::line(&mut tex, (110, 74, 38, 46), (i, 0), (i + nn, nn), 1.max(fd(ss, 2)));
            draw::line(&mut tex, (250, 220, 170, 30), (i + nn, 0), (i, nn), 1.max(fd(ss, 2)));
            i += step;
        }
        lay.bl(&tex, (0, 0));
        let mut sh = layer(nn, nn);
        radial(&mut sh, (80, 46, 16), (cx - nf * 0.14, bcy - bry * 0.55), nf * 0.08, nf * 0.80, 0, 150, 16, 1.4);
        draw::polygon(
            &mut sh,
            (255, 236, 196, 70),
            &arc_band(cx, bcy, brx * 0.86, brx * 0.14, PI * 1.05, PI * 1.35, 10),
            0,
        );
        lay.bl(&sh, (0, 0));
        lay.bl_flags(&mask(nn, nn, &body), (0, 0), BLEND_RGBA_MIN);
        big.bl(&lay, (0, 0));
        draw::lines(&mut big, lerp_i(BURLAP_D, BLACK, 30, 100), true, &body, 2.max(ss));
        // Kordel mit Schleife
        let cord: Rgb = (124, 82, 46);
        draw::rect_r(
            &mut big,
            lerp_i(cord, BLACK, 35, 100),
            ((cx - nhw * 1.45) as i32, (neck_y - nf * 0.035) as i32, (nhw * 2.9) as i32, (nf * 0.07) as i32),
            0,
            (nf * 0.03) as i32,
        );
        draw::rect_r(
            &mut big,
            cord,
            ((cx - nhw * 1.35) as i32, (neck_y - nf * 0.03) as i32, (nhw * 2.7) as i32, (nf * 0.05) as i32),
            0,
            (nf * 0.025) as i32,
        );
        let lw = ss.max(fd(nn, 30));
        draw::ellipse(&mut big, cord, (cx + nhw * 1.1, neck_y - nf * 0.09, nf * 0.10, nf * 0.08), lw);
        stroke(
            &mut big,
            cord,
            &[(cx + nhw * 1.2, neck_y), (cx + nhw * 1.9, neck_y + nf * 0.07), (cx + nhw * 1.7, neck_y + nf * 0.12)],
            lw,
        );
        // goldene Muenze als Wappen
        let r = nf * (0.13 + 0.012 * lv);
        let mc = (cx, bcy - bry * 0.08);
        draw::circle(&mut big, lerp_i(GOLD_D, BLACK, 25, 100), (mc.0, mc.1 + ss as f64), r, 0);
        draw::circle(&mut big, GOLD_D, mc, r, 0);
        draw::circle(&mut big, GOLD, mc, r * 0.84, 0);
        draw::circle(&mut big, lerp_i(GOLD, GOLD_D, 1, 3), mc, r * 0.64, 1.max(fd(ss, 2) + 1));
        let mut star: Vec<P> = Vec::with_capacity(10);
        for j in 0..10 {
            let a = -PI / 2.0 + PI * j as f64 / 5.0;
            let rr = r * if j % 2 == 0 { 0.46 } else { 0.19 };
            star.push((mc.0 + a.cos() * rr, mc.1 + a.sin() * rr));
        }
        draw::polygon(&mut big, GOLD_D, &star, 0);
        draw::polygon(&mut big, GOLD_L, &arc_band(mc.0, mc.1, r * 0.80, r * 0.16, PI * 1.05, PI * 1.45, 8), 0);
        if level >= 2 {
            sparkle(&mut big, WHITE, cx + nf * 0.20, nf * 0.08, nf * 0.07);
        }
        smooth(&big, n, n)
    })
}

// ============================================================= Platzen

/// `_shred_poly(S, v, flap)` - Umriss eines Gummifetzens in einer Flaeche
/// S x S (Streuwert v): gebogener Streifen oder gezackter Lappen.
fn shred_poly(s_: i32, v: i64, flap: bool) -> Vec<P> {
    let sf = s_ as f64;
    let c = sf / 2.0;
    let rot = ((v >> 5) % 360) as f64 * PI / 180.0;
    let (ca, sa) = (rot.cos(), rot.sin());
    let mut pts: Vec<P> = Vec::new();
    if flap {
        let k = 11;
        for j in 0..k {
            let a = TAU * j as f64 / k as f64;
            let mut rr = c * (0.62 + ((v >> (j * 3 % 27)) % 34) as f64 / 100.0);
            if j % 4 == 1 {
                rr *= 0.62; // Riss-Kerbe
            }
            pts.push((a.cos() * rr, a.sin() * rr * 0.74));
        }
    } else {
        let ln = sf * (0.84 + (v % 12) as f64 / 100.0);
        let th = sf * (0.20 + ((v >> 4) % 12) as f64 / 100.0);
        let bend = ((v >> 9) % 100) as f64 / 100.0 * 1.2 - 0.6;
        let mut top: Vec<P> = Vec::with_capacity(9);
        let mut bot: Vec<P> = Vec::with_capacity(9);
        for i in 0..9i64 {
            let t = i as f64 / 8.0 - 0.5;
            let (x, y) = (t * ln, bend * (t * t - 0.125) * ln);
            let d = 2.0 * bend * t;
            let nl = py::hypot(1.0, d);
            let hw = th * (0.62 + 0.38 * (t * PI).cos()) * (0.86 + 0.14 * ((v >> (i + 2)) % 2) as f64) / 2.0;
            top.push((x + d / nl * hw, y - hw / nl));
            bot.push((x - d / nl * hw, y + hw / nl));
        }
        // ausgefranste Enden (Zickzack)
        let tear = |a: P, b: P, sign: f64, seed: i64| -> Vec<P> {
            let mut out = Vec::with_capacity(3);
            for j in 1..4i64 {
                let f = j as f64 / 4.0;
                let jag = th * (if j % 2 != 0 { 0.30 } else { -0.12 }) * (0.7 + 0.3 * ((seed >> j) % 2) as f64);
                out.push((a.0 + (b.0 - a.0) * f + sign * jag, a.1 + (b.1 - a.1) * f));
            }
            out
        };
        pts.extend(top.iter().copied());
        pts.extend(tear(top[8], bot[8], 1.0, v >> 3));
        pts.extend(bot.iter().rev().copied());
        pts.extend(tear(bot[0], top[0], -1.0, v >> 7));
    }
    pts.iter().map(|&(x, y)| (c + x * ca - y * sa, c + x * sa + y * ca)).collect()
}

/// `shreds(w, stripe)` - 8 Gummifetzen (abwechselnd RUBBER / stripe wie die
/// Huelle): gebogene Streifen und gezackte Lappen, Groesse ~ w/5 .. w/3.
pub fn shreds(w: i32, stripe: impl Col) -> Vec<Surface> {
    let w = 8.max(w);
    let stripe = stripe_col(rgb_of(stripe));
    let key = format!("shreds|{w}|{}", ck(stripe));
    if let Some(v) = TUPLES.with(|c| c.borrow().get(&key).cloned()) {
        return v;
    }
    let mut out = Vec::with_capacity(8);
    for i in 0..8i64 {
        let v = hash(i * 7 + 3);
        let sz = 5.max((w as i64 * (20 + v % 13) / 100) as i32);
        let ss = 3;
        let s_ = sz * ss;
        let sf = s_ as f64;
        let mut big = layer(s_, s_);
        let col = if i % 2 == 0 { RUBBER } else { stripe };
        let pts = shred_poly(s_, v, i % 3 == 2);
        draw::polygon(&mut big, lerp_i(col, BLACK, 50, 100), &pts, 0);
        let c = sf / 2.0;
        let inner: Vec<P> = pts.iter().map(|&(x, y)| (c + (x - c) * 0.86 - sf * 0.02, c + (y - c) * 0.80 - sf * 0.03)).collect();
        draw::polygon(&mut big, col, &inner, 0);
        // Glanz: helle Linie nahe der Oberkante
        let hi = inner.len() / 3;
        let gl: Vec<P> = if hi > 1 {
            inner[1..hi].iter().map(|&(x, y)| (c + (x - c) * 0.55 - sf * 0.03, c + (y - c) * 0.45 - sf * 0.06)).collect()
        } else {
            Vec::new()
        };
        if gl.len() >= 2 {
            stroke(&mut big, lerp_i(col, WHITE, 60, 100), &gl, ss.max(fd(s_, 14)));
        }
        out.push(smooth(&big, sz, sz));
    }
    TUPLES.with(|c| {
        let mut c = c.borrow_mut();
        if c.len() > 400 {
            c.clear();
        }
        c.insert(key, out.clone());
    });
    out
}

/// `pop_star(r, step)` - Comic-Knall 2r x 2r (ohne Text); step 0..5: waechst
/// bis Schritt 2, dann wird er hohl und verblasst.
pub fn pop_star(r: i32, step: i32) -> Surface {
    let r = 4.max(r);
    let step = py::pmod32(step, 6);
    cached(format!("pop|{r}|{step}"), || {
        let ss = 3;
        let s_ = 2 * r * ss;
        let mut big = layer(s_, s_);
        let c = s_ as f64 / 2.0;
        let sc = POP_SCALE[step as usize] as f64 / 100.0 * 0.86;
        let a = POP_ALPHA[step as usize];
        let rot = 0.12 * step as f64;
        let rr = c * sc;
        // dunkler Rand verblasst schneller als der Stern
        draw::polygon(&mut big, (120, 28, 18, fd(a * a, 255)), &burst(c, c, rr * 1.06, rr * 0.58, 14, rot, 11, 0.22), 0);
        draw::polygon(&mut big, (255, 112, 36, a), &burst(c, c, rr, rr * 0.54, 14, rot, 11, 0.22), 0);
        draw::polygon(&mut big, (255, 212, 62, a), &burst(c, c, rr * 0.74, rr * 0.42, 14, rot + 0.11, 23, 0.22), 0);
        draw::polygon(&mut big, (255, 252, 230, a), &burst(c, c, rr * 0.44, rr * 0.32, 10, rot + 0.2, 37, 0.15), 0);
        if step >= 4 {
            // hohl werden: Mitte ausstanzen
            let (o, i) = if step == 4 { (0.30, 0.24) } else { (0.56, 0.44) };
            draw::polygon(&mut big, (0, 0, 0, 0), &burst(c, c, rr * o, rr * i, 10, rot, 41, 0.1), 0);
        }
        if (1..=3).contains(&step) {
            // Speed-Linien aussen herum
            let lw = ss.max(fd(s_, 60));
            for j in 0..10 {
                let ang = TAU * j as f64 / 10.0 + 0.3 + step as f64 * 0.05;
                let r0 = c * if step < 3 { 0.90 } else { 0.93 };
                let r1 = c * 0.99;
                draw::line(
                    &mut big,
                    (255, 236, 150, a),
                    (c + ang.cos() * r0, c + ang.sin() * r0),
                    (c + ang.cos() * r1, c + ang.sin() * r1),
                    lw,
                );
            }
        }
        smooth(&big, 2 * r, 2 * r)
    })
}

// ============================================================= Verlauf (Pillen)

/// `pill_color(m100)` - (Hintergrund, Schrift) einer Verlaufs-Pille: rot
/// unter x2, gruen ab x2, gold ab x10, violett ab x100.
pub fn pill_color(m100: i64) -> (Rgb, Rgb) {
    if m100 >= 10000 {
        return ((142, 70, 222), (255, 255, 255));
    }
    if m100 >= 1000 {
        return ((222, 168, 48), (46, 28, 6));
    }
    if m100 >= 200 {
        return ((36, 148, 100), (240, 255, 246));
    }
    ((168, 44, 60), (255, 236, 238))
}

/// `_fit_font(text, max_w, size, min_size=7)` - groesste fette Schrift (ab
/// size abwaerts), in der text in max_w passt.
fn fit_font(text: &str, max_w: i32, size: i32) -> Font {
    let mut size = size;
    while size > 7 && ui::font(size, true, false).width(text) > max_w {
        size -= 1;
    }
    ui::font(size, true, false)
}

/// `pill(w, h, m100, label, font_size)` - Verlaufs-Pille w x h: Verlauf,
/// Unterkante, Glanzlinie, Beschriftung (gecacht).
pub fn pill(w: i32, h: i32, m100: i64, label: &str, font_size: i32) -> Surface {
    let (w, h) = (6.max(w), 6.max(h));
    cached_small(format!("pill|{w}|{h}|{m100}|{label}|{font_size}"), || {
        let (bg, fg) = pill_color(m100);
        let ss = 2;
        let (ww, hh) = (w * ss, h * ss);
        let mut big = layer(ww, hh);
        let rad = fd(hh, 2);
        draw::rect_r(&mut big, lerp_i(bg, BLACK, 50, 100), (0, ss, ww, hh - ss), 0, rad);
        let mut body = layer(ww, hh - ss);
        let hb = hh - ss;
        let (top, bot) = (lerp_i(bg, WHITE, 28, 100), lerp_i(bg, BLACK, 10, 100));
        for y in 0..hb {
            body.fill_r(lerp_i(top, bot, y as i64, (hb - 1) as i64), (0, y, ww, 1));
        }
        let mut mk = layer(ww, hb);
        mk.fill_c((255, 255, 255, 0));
        draw::rect_r(&mut mk, (255, 255, 255, 255), (0, 0, ww, hb), 0, rad);
        body.bl_flags(&mk, (0, 0), BLEND_RGBA_MIN);
        big.bl(&body, (0, 0));
        draw::line(
            &mut big,
            lerp_i(bg, WHITE, 52, 100),
            (rad as f64 * 0.8, ss as f64 * 1.5),
            (ww as f64 - rad as f64 * 0.8, ss as f64 * 1.5),
            ss,
        );
        if m100 >= 1000 {
            draw::rect_r(&mut big, lerp_i(bg, WHITE, 60, 100), (0, 0, ww, hh - ss), 1.max(fd(ss, 2)), rad);
        }
        let mut img = smooth(&big, w, h);
        let fnt = fit_font(label, w - 4.max(fd(h, 2)), font_size);
        let txt = fnt.render(label, true, fg);
        img.blr(&txt, txt.get_rect().center_at((fd(w, 2), fd(h - 1, 2))));
        img
    })
}

// ============================================================= Piktogramme

/// `_note(s, col, hx, hy, n, stem_top, lw)` - Notenkopf mit Hals; liefert x
/// des Halses.
fn note(s: &mut Surface, col: Rgb, hx: f64, hy: f64, n: f64, stem_top: f64, lw: i32) -> f64 {
    draw::polygon(s, col, &ellipse_pts(hx, hy, n * 0.145, n * 0.105, -0.42, 20), 0);
    let sx = hx + n * 0.118;
    draw::line(s, col, (sx, hy - n * 0.02), (sx, stem_top), 1.max((lw as f64 * 0.8) as i32));
    sx
}

/// `_mini_balloon(cx, top, R)` - Umriss eines kleinen Ballons + y und halbe
/// Breite des Munds.
fn mini_balloon(cx: f64, top: f64, r: f64) -> (Vec<P>, f64, f64) {
    let yeq = top + r;
    let l = r * 1.15;
    let m = r * 0.26;
    let mut right: Vec<P> = Vec::with_capacity(17);
    for i in 0..9 {
        let a = PI / 2.0 * i as f64 / 8.0;
        right.push((cx + r * a.sin(), yeq - r * a.cos()));
    }
    for i in 1..9 {
        let u = i as f64 / 8.0;
        right.push((cx + m + (r - m) * py::pow((u * PI / 2.0).cos(), 1.2), yeq + l * u));
    }
    let mut sil = right.clone();
    sil.extend(right.iter().rev().map(|&(x, y)| (2.0 * cx - x, y)));
    (sil, yeq + l, m)
}

/// `_icon_balloon(s, col, cx, top, R, n, lw, ss)` - kleiner Ballon mit
/// ausgespartem Glanz, Seilen und Korb.
#[allow(clippy::too_many_arguments)]
fn icon_balloon(s: &mut Surface, col: Rgb, cx: f64, top: f64, r: f64, _n: f64, lw: i32, ss: i32) -> (Vec<P>, f64) {
    let (sil, ym, m) = mini_balloon(cx, top, r);
    draw::polygon(s, col, &sil, 0);
    draw::polygon(s, (0, 0, 0, 0), &arc_band(cx, top + r, r * 0.76, r * 0.22, PI * 1.10, PI * 1.40, 10), 0);
    let bw = r * 0.70;
    let by = ym + r * 0.34;
    let rl = ss.max(fd(lw, 2));
    draw::line(s, col, (cx - m, ym), (cx - bw / 2.0, by), rl);
    draw::line(s, col, (cx + m, ym), (cx + bw / 2.0, by), rl);
    draw::rect_r(s, col, ((cx - bw / 2.0) as i32, by as i32, bw as i32, (r * 0.46) as i32), 0, 1.max(ss * 2));
    (sil, by + r * 0.46)
}

/// `icon(name, size, col)` - Piktogramm size x size (4-fach
/// ueberabgetastet): auto, stats, fair, turbo, music, mute, close, left,
/// right, check, cross, half, ff, play, balloon, two.
pub fn icon(name: &str, size: i32, col: impl Col) -> Surface {
    let size = 4.max(size);
    let col = rgb_of(col);
    cached_small(format!("icon|{name}|{size}|{}", ck(col)), || {
        let ss = 4;
        let n = size * ss;
        let nf = n as f64;
        let mut s = layer(n, n);
        let lw = ss.max(fd(n, 9));
        match name {
            "auto" => {
                // Kreispfeil mit Spitze + Play-Dreieck in der Mitte
                let (c, r) = ((fd(n, 2), fd(n, 2)), fd(n * 2, 5));
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
                let bw = fd(n, 5);
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
            "music" => {
                // zwei Achtelnoten mit Balken
                let x1 = note(&mut s, col, nf * 0.30, nf * 0.75, nf, nf * 0.22, lw);
                let x2 = note(&mut s, col, nf * 0.71, nf * 0.66, nf, nf * 0.13, lw);
                let hl = ss.max((lw as f64 * 0.8) as i32) as f64 / 2.0;
                let bt = nf * 0.13;
                draw::polygon(
                    &mut s,
                    col,
                    &[(x1 - hl, nf * 0.22), (x2 + hl, nf * 0.13), (x2 + hl, nf * 0.13 + bt), (x1 - hl, nf * 0.22 + bt)],
                    0,
                );
            }
            "mute" => {
                // Achtelnote, schraeg durchgestrichen
                let x1 = note(&mut s, col, nf * 0.40, nf * 0.72, nf, nf * 0.15, lw);
                stroke(
                    &mut s,
                    col,
                    &[(x1, nf * 0.15), (x1 + nf * 0.13, nf * 0.25), (x1 + nf * 0.19, nf * 0.38), (x1 + nf * 0.14, nf * 0.50)],
                    ss.max((lw as f64 * 0.8) as i32),
                );
                let (a, b) = ((nf * 0.15, nf * 0.13), (nf * 0.86, nf * 0.87));
                stroke(&mut s, (0, 0, 0, 0), &[a, b], (lw as f64 * 2.4) as i32);
                stroke(&mut s, col, &[a, b], lw);
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
                draw::lines(
                    &mut s,
                    col,
                    false,
                    &[(nf * 0.16, nf * 0.52), (nf * 0.4, nf * 0.76), (nf * 0.84, nf * 0.26)],
                    ss.max(fd(n, 7)),
                );
            }
            "cross" => {
                let w2 = ss.max(fd(n, 7));
                draw::line(&mut s, col, (nf * 0.2, nf * 0.2), (nf * 0.8, nf * 0.8), w2);
                draw::line(&mut s, col, (nf * 0.8, nf * 0.2), (nf * 0.2, nf * 0.8), w2);
            }
            "half" => {
                // "1/2" aus Strichen: 1, Bruchstrich, 2
                let lw2 = ss.max((lw as f64 * 0.9) as i32);
                stroke(&mut s, col, &[(nf * 0.16, nf * 0.22), (nf * 0.30, nf * 0.11), (nf * 0.30, nf * 0.50)], lw2);
                stroke(&mut s, col, &[(nf * 0.74, nf * 0.10), (nf * 0.27, nf * 0.90)], lw2);
                let (c2x, c2y, r2) = (nf * 0.69, nf * 0.63, nf * 0.13);
                let mut pts: Vec<P> = (0..11)
                    .map(|i| PI * (1.06 + 1.12 * i as f64 / 10.0))
                    .map(|a| (c2x + r2 * a.cos(), c2y + r2 * a.sin()))
                    .collect();
                pts.push((nf * 0.55, nf * 0.90));
                pts.push((nf * 0.85, nf * 0.90));
                stroke(&mut s, col, &pts, lw2);
            }
            "ff" => {
                draw::polygon(&mut s, col, &[(nf * 0.10, nf * 0.20), (nf * 0.50, nf * 0.50), (nf * 0.10, nf * 0.80)], 0);
                draw::polygon(&mut s, col, &[(nf * 0.48, nf * 0.20), (nf * 0.88, nf * 0.50), (nf * 0.48, nf * 0.80)], 0);
            }
            "play" => {
                draw::polygon(&mut s, col, &[(nf * 0.26, nf * 0.14), (nf * 0.84, nf * 0.50), (nf * 0.26, nf * 0.86)], 0);
            }
            "balloon" => {
                icon_balloon(&mut s, col, nf * 0.5, nf * 0.05, nf * 0.31, nf, lw, ss);
            }
            "two" => {
                // zwei Ballone: hinten links kleiner, vorn rechts groesser (mit Luecke)
                icon_balloon(&mut s, col, nf * 0.33, nf * 0.04, nf * 0.21, nf, lw, ss);
                let sil = mini_balloon(nf * 0.66, nf * 0.16, nf * 0.245).0;
                draw::polygon(&mut s, (0, 0, 0, 0), &sil, 0);
                draw::lines(&mut s, (0, 0, 0, 0), true, &sil, ss.max(lw));
                icon_balloon(&mut s, col, nf * 0.66, nf * 0.16, nf * 0.245, nf, lw, ss);
            }
            _ => {}
        }
        smooth(&s, size, size)
    })
}

// ============================================================= Kurve

/// `_label(text, size)` - Beschriftung mit weichem Schatten (gecacht).
fn label_img(text: &str, size: i32) -> Surface {
    cached_small(format!("label|{text}|{size}"), || {
        let f = ui::font(size, true, false);
        let fg = f.render(text, true, (238, 240, 252));
        let mut sh = f.render(text, true, (0, 0, 0)).copy();
        sh.alpha_set(150);
        let mut out = layer(fg.w() + 1, fg.h() + 1);
        out.bl(&sh, (1, 1));
        out.bl(&fg, (0, 0));
        out
    })
}

/// `_grid_strip(length, vertical)` - Hilfslinie (1 px, gestrichelt,
/// halbdurchsichtig) als gecachter Streifen.
fn grid_strip(length: i32, vertical: bool) -> Surface {
    cached_small(format!("gstrip|{length}|{vertical}"), || {
        let mut s = if vertical { layer(1, length) } else { layer(length, 1) };
        let mut i = 0;
        while i < length {
            if vertical {
                s.fill_r((255, 255, 255, 44), (0, i, 1, 5));
            } else {
                s.fill_r((255, 255, 255, 44), (i, 0, 5, 1));
            }
            i += 8;
        }
        s
    })
}

/// `draw_grid(s, rect, ylines, xlines, font_size)` - Hilfslinien:
/// waagerecht bei ylines (Text links ueber der Linie), senkrecht bei xlines
/// (Text unten mittig); ein leerer Text zeichnet nur die Linie.
pub fn draw_grid<A: AsRef<str>, B: AsRef<str>>(
    s: &mut Surface,
    rect: impl IntoRect,
    ylines: &[(i32, A)],
    xlines: &[(i32, B)],
    font_size: i32,
) {
    let rect = rect.rect();
    if rect.w <= 0 || rect.h <= 0 {
        return;
    }
    let pad = 2.max(fd(font_size, 3));
    if !ylines.is_empty() {
        let hl = grid_strip(rect.w, false);
        for (y, label) in ylines {
            let y = *y;
            s.bl(&hl, (rect.x, y));
            let label = label.as_ref();
            if !label.is_empty() {
                let img = label_img(label, font_size);
                s.bl(&img, (rect.x + pad, y - img.h()));
            }
        }
    }
    for (x, label) in xlines {
        let x = *x;
        let label = label.as_ref();
        let img = if label.is_empty() { None } else { Some(label_img(label, font_size)) };
        let bottom = rect.bottom() - img.as_ref().map_or(0, |i| i.h() + pad);
        if bottom > rect.y {
            s.bl_area(&grid_strip(rect.h, true), (x, rect.y), (0, 0, 1, bottom - rect.y));
        }
        if let Some(img) = img {
            s.bl(&img, (x - fd(img.w(), 2), rect.bottom() - img.h() - 1));
        }
    }
}

/// `_ribbon(p, hw)` - Band der halben Breite hw um die Punktfolge p.
fn ribbon(p: &[(i32, i32)], hw: f64) -> Vec<P> {
    let n = p.len();
    let mut left: Vec<P> = Vec::with_capacity(2 * n);
    let mut right: Vec<P> = Vec::with_capacity(n);
    for i in 0..n {
        let (xa, ya) = p[i.saturating_sub(1)];
        let (xb, yb) = p[(n - 1).min(i + 1)];
        let (dx, dy) = (xb - xa, yb - ya);
        let mut d = py::hypot(dx as f64, dy as f64);
        if d == 0.0 {
            d = 1.0;
        }
        let (nx, ny) = ((-dy) as f64 / d * hw, dx as f64 / d * hw);
        let (x, y) = (p[i].0 as f64, p[i].1 as f64);
        left.push((x + nx, y + ny));
        right.push((x - nx, y - ny));
    }
    left.extend(right.iter().rev());
    left
}

/// `_work(w, h)` - wiederverwendete Arbeitsflaeche fuer die Kurvenflaeche
/// (waechst nur).
fn work(w: i32, h: i32) -> Surface {
    WORK.with(|c| {
        let mut c = c.borrow_mut();
        let need = match c.as_ref() {
            None => true,
            Some(cur) => cur.w() < w || cur.h() < h,
        };
        if need {
            let (ow, oh) = c.as_ref().map_or((0, 0), |cur| (cur.w(), cur.h()));
            *c = Some(layer(fd(w.max(ow) + 63, 64) * 64, fd(h.max(oh) + 63, 64) * 64));
        }
        c.as_ref().cloned().unwrap_or_else(|| layer(w, h))
    })
}

/// `draw_curve(s, pts, col, base_y, width)` - Multiplikator-Kurve:
/// Verlaufsflaeche bis base_y (oben dichter), breites Leuchten, kraeftige
/// Linie mit Glanz und hellem Kopf (pts = absolute Punkte, x steigend).
pub fn draw_curve(s: &mut Surface, pts: &[(f64, f64)], col: impl Col, base_y: f64, width: i32) {
    if pts.len() < 2 {
        return;
    }
    let col = rgb_of(col);
    let width = 1.max(width);
    let by = ri(base_y);
    let step = 1.max(pts.len().div_ceil(240)); // hoechstens ~240 Stuetzpunkte
    let mut sel: Vec<(f64, f64)> = pts.iter().step_by(step).copied().collect();
    if !(pts.len() - 1).is_multiple_of(step) {
        sel.push(pts[pts.len() - 1]);
    }
    let p: Vec<(i32, i32)> = sel.iter().map(|&(x, y)| (ri(x), ri(y))).collect();
    let (sw, sh) = (s.w(), s.h());
    let g = width * 2 + 3;
    let xmin = p.iter().map(|q| q.0).min().unwrap_or(0);
    let xmax = p.iter().map(|q| q.0).max().unwrap_or(0);
    let ymin = p.iter().map(|q| q.1).min().unwrap_or(0);
    let ymax = p.iter().map(|q| q.1).max().unwrap_or(0);
    let (left, right) = (0.max(xmin - g), sw.min(xmax + g + 1));
    let (top, bottom) = (0.max(ymin.min(by) - g), sh.min(ymax.max(by) + 1));
    let light = lerp_i(col, WHITE, 40, 100);
    if right > left && bottom > top {
        let (bw, bh) = (right - left, bottom - top);
        let mut wk = work(bw, bh);
        wk.fill_r((0, 0, 0, 0), (0, 0, bw, bh));
        let loc: Vec<(i32, i32)> = p.iter().map(|&(x, y)| (x - left, y - top)).collect();
        if by > ymin {
            let mut poly = loc.clone();
            poly.push((loc[loc.len() - 1].0, by - top));
            poly.push((loc[0].0, by - top));
            draw::polygon(&mut wk, (col.0, col.1, col.2, 255), &poly, 0);
            // Verlauf: Alpha bandweise multiplizieren (oben ~70, unten 0)
            let y0 = 0.max(ymin - top);
            let span = 1.max(by - top - y0);
            let bands = 24;
            for k in 0..bands {
                let ya = y0 + fd(span * k, bands);
                let yb = if k < bands - 1 { y0 + fd(span * (k + 1), bands) } else { bh };
                let a = fd(70 * (bands - k) * (bands - k), bands * bands);
                wk.fill_rf((255, 255, 255, a), (0, ya, bw, yb - ya), BLEND_RGBA_MULT);
            }
        }
        let st = 1.max(loc.len().div_ceil(96)); // Leuchten: hoechstens ~96 Punkte
        let mut gl: Vec<(i32, i32)> = loc.iter().step_by(st).copied().collect();
        if !(loc.len() - 1).is_multiple_of(st) {
            gl.push(loc[loc.len() - 1]);
        }
        for (hw, a) in [(width as f64 * 2.0, 70), (width as f64 * 1.0 + 1.0, 120)] {
            let c = (light.0, light.1, light.2, a);
            draw::polygon(&mut wk, c, &ribbon(&gl, hw), 0);
            draw::circle(&mut wk, c, gl[0], hw, 0);
            draw::circle(&mut wk, c, gl[gl.len() - 1], hw, 0);
        }
        s.bl_area(&wk, (left, top), (0, 0, bw, bh));
    }
    draw::lines(s, col, false, &p, width);
    if width >= 3 {
        let r = width as f64 / 2.0;
        for &q in &p {
            draw::circle(s, col, q, r, 0);
        }
    }
    if width >= 3 {
        let shine: Vec<(i32, i32)> = p.iter().map(|&(x, y)| (x, y - fd(width, 4))).collect();
        draw::lines(s, light, false, &shine, 1.max(fd(width, 3)));
    }
    let end = p[p.len() - 1];
    draw::circle(s, col, end, width as f64 * 0.95 + 1.0, 0);
    draw::circle(s, lerp_i(col, WHITE, 75, 100), end, fmax(1.0, width as f64 * 0.5), 0);
}

/// `glow(r, col)` - Lichthof mit Radius r auf schwarzem Grund - zum
/// additiven Blitten (`BLEND_RGB_ADD`).
pub fn glow(r: i32, col: impl Col) -> Surface {
    let r = 1.max(r);
    let col = rgb_of(col);
    cached(format!("glow|{r}|{}", ck(col)), || {
        let mut s = pg::surface(2 * r + 2, 2 * r + 2);
        s.fill_c((0, 0, 0));
        let steps = 12;
        for i in 0..steps {
            let f = (i + 1) as f64 / steps as f64;
            let a = f * f;
            draw::circle(
                &mut s,
                ((col.0 as f64 * a) as i32, (col.1 as f64 * a) as i32, (col.2 as f64 * a) as i32),
                (r + 1, r + 1),
                1.max((r as f64 * (1.0 - i as f64 / steps as f64)) as i32),
                0,
            );
        }
        s
    })
}

/// `curve_preview(w, h, crash100, marks)` - Mini-Kurve w x h fuers
/// Fairness-Fenster: von x1 bis zum Platzpunkt (Knall-Stern bzw. goldener
/// Stern bei der Mondlandung), gruene Punkte bei den Ausstiegen marks (m100).
pub fn curve_preview(w: i32, h: i32, crash100: i64, marks: &[i64]) -> Surface {
    let (w, h) = (16.max(w), 12.max(h));
    let crash100 = 100.max(lg::MAX_MULT100.min(crash100));
    let key = format!("preview|{w}|{h}|{crash100}|{:?}", marks);
    cached_small(key, || {
        let ss = 2;
        let (ww, hh) = (w * ss, h * ss);
        let mut big = layer(ww, hh);
        let pad = (ss * 2).max(fd(ww.min(hh), 12));
        let (x0, y0, x1, y1) = (pad, pad + ss, ww - pad * 2, hh - pad);
        let moon = crash100 >= lg::MAX_MULT100;
        let tt = 1.max(lg::pop_tick(crash100));
        let xs = tt.max(lg::TPS * 4);
        let top = crash100.max(150);
        let px = |t: i64| -> i64 { x0 as i64 + py::fdiv((x1 - x0) as i64 * t, xs) };
        let pyy = |m: i64| -> i64 { y1 as i64 - py::fdiv((y1 - y0) as i64 * (m.min(crash100) - 100), top - 100) };
        let ax = (255, 255, 255, 70);
        draw::line(&mut big, ax, (x0, y0 - fd(pad, 2)), (x0, y1), ss);
        draw::line(&mut big, ax, (x0, y1), (ww - fd(pad, 2), y1), ss);
        let yc = pyy(crash100);
        let mut x = x0 + ss * 2;
        while x < x1 {
            draw::line(&mut big, (255, 255, 255, 46), (x as i64, yc), (x1.min(x + ss * 3) as i64, yc), 1.max(fd(ss, 2)));
            x += ss * 6;
        }
        let k = 8.max(fd(ww, 3 * ss)) as i64;
        let pts: Vec<(i64, i64)> =
            (0..=k).map(|i| (px(py::fdiv(tt * i, k)), pyy(lg::mult_at(py::fdiv(tt * i, k))))).collect();
        let line = if moon { GOLD } else { (255, 96, 112) };
        let mut area = pts.clone();
        area.push((pts[pts.len() - 1].0, y1 as i64));
        area.push((pts[0].0, y1 as i64));
        draw::polygon(&mut big, (line.0, line.1, line.2, 56), &area, 0);
        draw::lines(&mut big, line, false, &pts, ss.max(fd(hh, 40)));
        let rm = (ss * 2).max(fd(hh, 22));
        for &mv in marks {
            if lg::MIN_CASH <= mv && mv <= crash100 {
                let c = (px(lg::tick_at(mv)), pyy(mv));
                draw::circle(&mut big, (12, 52, 30), c, rm + ss, 0);
                draw::circle(&mut big, (64, 214, 122), c, rm, 0);
                draw::circle(&mut big, (214, 255, 230), (c.0 - fd(rm, 3) as i64, c.1 - fd(rm, 3) as i64), 1.max(fd(rm, 3)), 0);
            }
        }
        let (ex, ey) = (pts[pts.len() - 1].0 as f64, pts[pts.len() - 1].1 as f64);
        let rs = (ss * 4).max(fd(ww.min(hh), 9)) as f64;
        if moon {
            draw::polygon(&mut big, GOLD_D, &burst(ex, ey, rs * 1.1, rs * 0.5, 5, -PI / 2.0, 5, 0.0), 0);
            draw::polygon(&mut big, GOLD, &burst(ex, ey, rs * 0.9, rs * 0.4, 5, -PI / 2.0, 5, 0.0), 0);
        } else {
            draw::polygon(&mut big, (120, 28, 18), &burst(ex, ey, rs * 1.08, rs * 0.56, 9, 0.2, 3, 0.22), 0);
            draw::polygon(&mut big, (255, 112, 36), &burst(ex, ey, rs, rs * 0.52, 9, 0.2, 3, 0.22), 0);
            draw::polygon(&mut big, (255, 226, 90), &burst(ex, ey, rs * 0.62, rs * 0.36, 9, 0.35, 9, 0.22), 0);
            draw::circle(&mut big, (255, 252, 230), (ex, ey), rs * 0.26, 0);
        }
        smooth(&big, w, h)
    })
}

// ============================================================= Mondlandung

/// `flag(n, col)` - Fahne n x n: Stange mit Querstab, Tuch in col mit
/// Falten und weissem Lama im Abzeichen, Staubhuegel am Fuss.
pub fn flag(n: i32, col: impl Col) -> Surface {
    let n = 8.max(n);
    let col = rgb_of(col);
    cached(format!("flag|{n}|{}", ck(col)), || {
        let ss = 3;
        let nn = n * ss;
        let nf = nn as f64;
        let mut big = layer(nn, nn);
        let px = nf * 0.20;
        let pw = ss.max(fd(nn, 22));
        let pwf = pw as f64;
        let foot = nf * 0.94;
        draw::ellipse(&mut big, (92, 92, 104), (px - nf * 0.18, foot - nf * 0.05, nf * 0.36, nf * 0.10), 0);
        draw::ellipse(&mut big, (150, 150, 162), (px - nf * 0.15, foot - nf * 0.065, nf * 0.28, nf * 0.07), 0);
        draw::rect(&mut big, (112, 116, 130), ((px - pwf / 2.0) as i32, (nf * 0.06) as i32, pw, (foot - nf * 0.06) as i32), 0);
        draw::rect(
            &mut big,
            (232, 234, 242),
            ((px - pwf / 2.0) as i32, (nf * 0.06) as i32, 1.max(fd(pw, 3)), (foot - nf * 0.06) as i32),
            0,
        );
        let (fx0, fx1) = (px + pwf / 2.0, nf * 0.93);
        let fy0 = nf * 0.10;
        let fh = nf * 0.40;
        let mut top_e: Vec<P> = Vec::with_capacity(17);
        let mut bot_e: Vec<P> = Vec::with_capacity(17);
        for i in 0..17 {
            let f = i as f64 / 16.0;
            let x = fx0 + (fx1 - fx0) * f;
            let wave = (f * PI * 3.0).sin() * nf * 0.02 * f;
            top_e.push((x, fy0 + ss as f64 + wave * 0.25));
            bot_e.push((x, fy0 + fh + wave - nf * 0.03 * f));
        }
        let mut cloth = top_e.clone();
        cloth.extend(bot_e.iter().rev());
        let mut lay = layer(nn, nn);
        draw::polygon(&mut lay, col, &cloth, 0);
        let mut folds = layer(nn, nn);
        let mut x = fx0 as i32;
        while x < fx1 as i32 + 1 {
            let f = (x as f64 - fx0) / fmax(1.0, fx1 - fx0);
            let a = (46.0 * (f * PI * 3.0).cos() * fmin(1.0, f * 4.0)) as i32;
            folds.fill_r(if a > 0 { (255, 255, 255, a) } else { (0, 0, 0, -a) }, (x, 0, ss, nn));
            x += ss;
        }
        lay.bl(&folds, (0, 0));
        // Abzeichen: dunklerer Kreis mit weissem Lama
        let bc = (fx0 + (fx1 - fx0) * 0.5, fy0 + fh * 0.50);
        let br = fh * 0.34;
        draw::circle(&mut lay, lerp_i(col, BLACK, 30, 100), bc, br, 0);
        let e = 8.max((br * 1.6) as i32);
        let mut lam = layer(e, e);
        lcd::draw_llama(&mut lam, e, "stand", WHITE.c());
        lam.fill_f((255, 255, 255, 0), BLEND_RGBA_MAX);
        lay.bl(&lam, ((bc.0 - e as f64 * 0.55) as i32, (bc.1 - e as f64 * 0.52) as i32));
        lay.bl_flags(&mask(nn, nn, &cloth), (0, 0), BLEND_RGBA_MIN);
        big.bl(&lay, (0, 0));
        draw::lines(&mut big, lerp_i(col, BLACK, 45, 100), false, &bot_e, 1.max(fd(ss * 2, 3)));
        draw::line(&mut big, (206, 208, 220), (px, fy0), (fx1 + ss as f64, fy0), ss.max(fd(pw * 2, 3)));
        draw::circle(&mut big, (244, 244, 250), (px, nf * 0.06), pwf * 0.9, 0);
        smooth(&big, n, n)
    })
}

// ============================================================= Zusammengesetztes

/// `draw_rig(s, cx, rim_y, base, level, stripe, riders, flame)` - ganzer
/// Ballon ueber dem Korbrand (cx, rim_y): Huelle + Seile, Flamme, Rueckwand,
/// Lamas, Vorderwand. riders: je Platz (Halstuch, Pose) oder None (schon
/// abgesprungen) - 1 Eintrag = Lama mittig, 2 = Plaetze links/rechts;
/// flame 0..5 oder negativ (aus). Gibt das Rechteck der Huelle (mit
/// Schuerze, ohne Seile) zurueck.
#[allow(clippy::too_many_arguments)]
pub fn draw_rig(
    s: &mut Surface,
    cx: f64,
    rim_y: f64,
    base: i32,
    level: i32,
    stripe: impl Col,
    riders: &[Option<(Color, &str)>],
    flame: i32,
) -> Rect {
    let (cx, rim_y) = (cx as i32, rim_y as i32);
    let img = balloon(base, level, stripe);
    let (w, h) = (img.w(), img.h());
    let (x0, y0) = (cx - fd(w, 2), rim_y + 1 - h);
    s.bl(&img, (x0, y0));
    let (mouth, _sk, _nh, mouth_hw, frame, burner_h, _bhw, flame_w) = rig(base);
    if flame >= 0 {
        let g = glow(3.max(fd(mouth_hw * 5, 4)), if flame % 2 == 0 { (84, 46, 12) } else { (70, 38, 10) });
        s.bl_flags(&g, (cx - fd(g.w(), 2), rim_y - mouth - fd(g.h(), 2)), BLEND_RGB_ADD);
        let fl = burner_flame(flame_w, flame);
        s.bl(&fl, (cx - fd(fl.w(), 2), rim_y - frame - burner_h + 1 - fl.h()));
    }
    let bw = basket_w(base);
    let bx = cx - fd(bw, 2);
    s.bl(&basket(bw, false), (bx, rim_y));
    let n = llama_size(base);
    let foot = rider_foot(rim_y, base);
    let single = riders.len() == 1;
    for (i, rider) in riders.iter().take(2).enumerate() {
        let Some((scarf, pose)) = rider else {
            continue;
        };
        let dx = if single {
            0
        } else if i == 0 {
            fd(-bw, 4)
        } else {
            fd(bw, 4)
        };
        let spr = lcd::llama_sprite(n, pose, rgb_of(*scarf).c());
        s.bl(&spr, (cx + dx - fd(n, 2), foot - n));
    }
    s.bl(&basket(bw, true), (bx, rim_y));
    Rect::new(x0, y0, w, h - mouth)
}

/// `_open_stage(open_f)` - Oeffnungsstufe 0..8 des Schirms.
fn open_stage(open_f: f64) -> i32 {
    0.max(8.min((open_f * 8.0 + 1e-6) as i32))
}

/// `_jumper_img(n, scarf, chute, k)` - Lama mit Gurt am Schirm (Stufe k):
/// Flaeche und Lage der Sprite-Ecke.
fn jumper_img(n: i32, scarf: Rgb, chute: Rgb, k: i32) -> (Surface, i32, i32) {
    let cw = 6.max(fd(n * 3, 2));
    let ww = cw.max(n) + 2;
    let hx = fd(ww, 2);
    let lx = hx - fd(n * 49, 100);
    let ly = cw - 1 - fd(n * 40, 100);
    let img = cached(format!("jumper|{n}|{}|{}|{k}", ck(scarf), ck(chute)), || {
        let mut out = layer(ww, ly + n + 1);
        let can = chute_stage(cw, chute, k);
        out.bl(&can, (hx - fd(cw, 2), 0));
        let spr = lcd::llama_sprite(n, if k < 4 { "hop" } else { "stand" }, scarf.c());
        out.bl(&spr, (lx, ly));
        let strap = 1.max(fd(n, 16));
        draw::line(&mut out, (54, 48, 60), (hx, ly + fd(n * 40, 100)), (hx - fd(n * 3, 100), ly + fd(n * 68, 100)), strap);
        draw::circle(&mut out, METAL_L, (hx, ly + fd(n * 41, 100)), 1.max(strap), 0);
        out
    });
    (img, lx, ly)
}

/// `draw_jumper(s, cx, cy, n, scarf, chute, open_f)` - Lama am Fallschirm;
/// (cx, cy) = Mitte des Lama-Sprites, open_f 0..1 = Schirm geht auf.
pub fn draw_jumper(s: &mut Surface, cx: f64, cy: f64, n: i32, scarf: impl Col, chute: impl Col, open_f: f64) {
    let n = 4.max(n);
    let (img, lx, ly) = jumper_img(n, rgb_of(scarf), rgb_of(chute), open_stage(open_f));
    s.bl(&img, (cx as i32 - lx - fd(n, 2), cy as i32 - ly - fd(n, 2)));
}

/// `draw_sack(s, cx, cy, n, chute, open_f)` - Geldsack am Mini-Schirm;
/// (cx, cy) = Mitte des Sacks, open_f 0..1.
pub fn draw_sack(s: &mut Surface, cx: f64, cy: f64, n: i32, chute: impl Col, open_f: f64) {
    let n = 6.max(n);
    let chute = rgb_of(chute);
    let k = open_stage(open_f);
    let cw = 6.max(fd(n * 13, 10));
    let ww = cw.max(n) + 2;
    let hx = fd(ww, 2);
    let hook = fd(n * 10, 100);
    let (sx, sy) = (hx - fd(n, 2), cw - 1 - hook);
    let img = cached(format!("sackchute|{n}|{}|{k}", ck(chute)), || {
        let mut out = layer(ww, sy + n + 1);
        out.bl(&chute_stage(cw, chute, k), (hx - fd(cw, 2), 0));
        out.bl(&money_sack(n, 1), (sx, sy));
        out
    });
    s.bl(&img, (cx as i32 - sx - fd(n, 2), cy as i32 - sy - fd(n, 2)));
}

/// `draw_faller(s, cx, cy, n, scarf, step)` - purzelndes Lama (Pose "fall"),
/// in 8 Stufen zu 45 Grad gedreht, Mitte (cx, cy).
pub fn draw_faller(s: &mut Surface, cx: f64, cy: f64, n: i32, scarf: impl Col, step: i32) {
    let n = 4.max(n);
    let step = py::pmod32(step, 8);
    let scarf = rgb_of(scarf);
    let img = cached(format!("faller|{n}|{}|{step}", ck(scarf)), || {
        let mut big = layer(n * 2, n * 2);
        lcd::draw_llama(&mut big, n * 2, "fall", scarf.c());
        if step != 0 {
            big = transform::rotate(&big, (-45 * step) as f64);
        }
        smooth(&big, fd(big.w(), 2), fd(big.h(), 2))
    });
    s.bl(&img, (cx as i32 - fd(img.w(), 2), cy as i32 - fd(img.h(), 2)));
}
