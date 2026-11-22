//! lamacrash_world.rs
//! ==================
//! Die Hoehenwelten von Lama Crash - uebertragen aus
//! `games/lamacrash_world.py` (nur Primitive, keine Bilddateien):
//!
//! ```text
//!     0 Wiese (ab x1)  . 1 Wolken (ab x1,5) . 2 Gewitter (ab x3)
//!     3 Nordlicht (ab x10) . 4 Weltall (ab x100) . 5 Mond (nur die Landung)
//! ```
//!
//! Himmelsverlauf, senkrecht kachelbare Ebenen mit Parallaxe (Haufen-,
//! Feder- und Schichtwolken, Regenvorhaenge, Sterne, Milchstrasse), der
//! Bodenstreifen (Weide, Zaun, Stall, Lamas, Startplatz) und belebte
//! Kleinigkeiten (Voegel, Flugzeuge, Regen, Blitze, Polarlicht, Jets mit
//! Kondensstreifen, Satelliten, Sternschnuppen, Funkeln, Erdkruemmung,
//! Planeten, wachsender Mond) sowie die Mondlandung.
//!
//! Zufall nur ueber `hash`, keine Zeit ausser `t`. Pythons `//` und `%`
//! auf Ganzzahlen stehen als `fd`/`md` (Abrunden auch bei negativen
//! Zahlen), `//` auf Kommazahlen als `floordiv` (CPythons `float_floor_div`).
//! Speicher wie in Python: Weltflaechen gehoeren zur Gruppe (Welt, w, h),
//! hoechstens zwei Gruppen bleiben; kleine Teile werden ab 300 Eintraegen
//! gemeinsam verworfen.

use std::cell::RefCell;
use std::collections::HashMap;
use std::f64::consts::{PI, TAU};

use pgz_engine::prelude::*;

use crate::lamacrash_logic as lg;
use crate::lamacrossing_draw as lcd;

/// Tempo der Ebenen (fern .. nah) relativ zur Steighoehe.
pub const PARALLAX: [f64; 3] = [0.22, 0.5, 1.0];
/// Weltnamen.
pub const NAMES: [&str; 6] = ["meadow", "clouds", "storm", "aurora", "space", "moon"];

const GROUPS: usize = 2; // hoechstens so viele Welt-Gruppen: aktuelle + naechste Welt
const SMALL_MAX: usize = 300; // kleine Teile (Voegel, Flugzeuge, Lichter): ab hier leeren

const RISE: i64 = 22; // Steigtempo: h * RISE / 10000 Pixel je Takt
const EASE: i64 = 60; // Anfahren: Tempo waechst in den ersten EASE Takten linear
const GROUND: i64 = 30; // Bodenstreifen in % der Buehnenhoehe
const PAD_X: i64 = 15; // Startplatz-Mitte in % der Buehnenbreite
const PAD_W: i64 = 17; // Deckbreite in % der Buehnenhoehe
const TOP: i64 = 35; // Ding 0 einer Hoehenreihe steht bei Hoehe 0 auf 35 % von h
const SAT_P: f64 = 0.3; // Parallaxe der Satelliten
const PLANET_P: f64 = 0.16; // Parallaxe der fernen Planeten
const EARTH_P: f64 = 0.6; // so schnell sinkt die Erdkruemmung (x rise_px)
const FLASH: f64 = 1.9; // Sekunden je Blitz-Fenster
const SHOOT: f64 = 2.6; // Sekunden je Sternschnuppen-Fenster

type Rgb = (i32, i32, i32);
type C4 = [i64; 4];
type Stop = (i64, C4);
/// Huegelreihe: (Grundlinie Promille, Hoehe Promille, Farbe, Wellen-Phasen).
type Hill = (i64, i64, Rgb, (f64, f64, f64));
/// Mondberge: (Hoehe Promille, Grundlinie Promille, Farbe, Kante, Phasen).
type MoonHill = (i64, i64, Rgb, Rgb, (f64, f64, f64));

const fn o(r: i64, g: i64, b: i64) -> C4 {
    [r, g, b, 255]
}

/// Himmel je Welt: Verlauf ((Promille von oben, Farbe), ...).
const SKY: [&[Stop]; 6] = [
    &[(0, o(52, 128, 222)), (560, o(110, 180, 242)), (1000, o(196, 228, 250))],
    &[(0, o(28, 88, 196)), (600, o(84, 150, 232)), (1000, o(162, 206, 244))],
    &[(0, o(20, 22, 36)), (520, o(42, 46, 64)), (1000, o(70, 76, 94))],
    &[(0, o(4, 5, 20)), (450, o(10, 14, 46)), (850, o(17, 19, 60)), (1000, o(28, 22, 70))],
    &[(0, o(0, 0, 3)), (700, o(2, 3, 12)), (1000, o(6, 8, 26))],
    &[(0, o(0, 0, 2)), (1000, o(8, 8, 18))],
];

/// Wolken-Ebene: (Anzahl bei w = h, Breite Promille von h min/max, Hoehe %
/// der Breite, Palette (Licht, Koerper, Schatten), Deckkraft, nur Seiten).
type CloudSpec = (i64, i64, i64, i64, [Rgb; 3], i64, bool);

fn clouds_spec(world: i64, depth: i64) -> Option<CloudSpec> {
    Some(match (world, depth) {
        (0, 1) => (3, 200, 320, 44, [(255, 255, 255), (236, 243, 252), (188, 207, 234)], 225, false),
        (0, 2) => (2, 340, 480, 42, [(255, 255, 255), (240, 246, 254), (180, 201, 232)], 255, true),
        (1, 0) => (9, 110, 210, 44, [(234, 243, 254), (210, 227, 248), (174, 199, 236)], 205, false),
        (1, 1) => (5, 240, 380, 44, [(255, 255, 255), (230, 240, 252), (170, 195, 230)], 240, false),
        (1, 2) => (2, 520, 760, 42, [(255, 255, 255), (238, 245, 254), (164, 189, 226)], 255, true),
        (2, 0) => (4, 460, 760, 34, [(86, 92, 116), (62, 66, 88), (44, 46, 64)], 225, false),
        (2, 1) => (3, 340, 540, 38, [(98, 104, 130), (54, 58, 78), (34, 36, 50)], 240, false),
        (2, 2) => (2, 600, 900, 36, [(74, 78, 102), (40, 42, 58), (24, 25, 36)], 250, true),
        _ => return None,
    })
}

/// Sternen-Ebene: (Pixel je Stern, Deckkraft min/max, Promille Glanzsterne,
/// Promille Doppelpixel).
fn stars_spec(world: i64, depth: i64) -> Option<(i64, i64, i64, i64, i64)> {
    Some(match (world, depth) {
        (3, 0) => (700, 30, 140, 0, 40),
        (3, 1) => (5200, 90, 230, 60, 200),
        (3, 2) => (60000, 200, 255, 1000, 0),
        (4, 0) => (420, 40, 170, 0, 30),
        (4, 1) => (3400, 110, 255, 80, 250),
        (4, 2) => (24000, 210, 255, 1000, 0),
        _ => return None,
    })
}

const STAR_COLS: [Rgb; 4] = [(238, 242, 255), (255, 238, 216), (206, 222, 255), (255, 250, 244)];

/// ferne Planeten: (Start-Takt ab Weltall-Beginn, x Promille von w, Radius
/// Promille von h, Art).
const PLANETS: [(i64, i64, i64, i64); 3] = [(520, 150, 58, 0), (2500, 860, 32, 1), (4500, 230, 44, 2)];

/// Polarlicht-Boegen je Art: (Unterkante % von h, Bogenhoehe Promille,
/// Scheitel Promille von w, Welle 1: Hoehe Promille von h, Laenge x w, Tempo;
/// Welle 2 ebenso; Wandern x w/s).
type Arc = (i64, i64, i64, i64, f64, f64, i64, f64, f64, f64);
const ARCS: [Arc; 2] = [(68, 100, 400, 40, 0.9, 0.55, 18, 0.33, 0.9, 0.012), (47, 70, 560, 30, 1.3, 0.40, 16, 0.40, 0.7, -0.008)];

// ============================================================= Cache

type Group = (i64, i64, i64);

struct WCache {
    /// Weltflaechen je Gruppe (Welt, w, h).
    world: HashMap<Group, HashMap<String, Surface>>,
    /// Welt-Gruppen, zuletzt benutzte hinten.
    groups: Vec<Group>,
    /// kleine Teile (alle anderen Schluessel).
    small: HashMap<String, Surface>,
    /// `_cache[("buf",)]` - buehnengrosse Zwischenflaeche.
    buf: Option<Surface>,
}

thread_local! {
    static WC: RefCell<WCache> = RefCell::new(WCache {
        world: HashMap::new(),
        groups: Vec::new(),
        small: HashMap::new(),
        buf: None,
    });
}

/// Gruppenteil von `_cached`: die Gruppe nach hinten holen bzw. neu
/// eintragen (dabei die am laengsten unbenutzte samt Flaechen verwerfen).
fn touch_group(c: &mut WCache, g: Group) {
    if c.groups.last() != Some(&g) {
        if let Some(pos) = c.groups.iter().position(|x| *x == g) {
            c.groups.remove(pos);
        } else {
            while c.groups.len() >= GROUPS {
                let old = c.groups.remove(0);
                c.world.remove(&old);
            }
        }
        c.groups.push(g);
    }
}

/// `_cached(_wkey(world, w, h, rest), build)` - Weltflaeche.
fn cached_w(world: i64, w: i64, h: i64, rest: &str, build: impl FnOnce() -> Surface) -> Surface {
    let g = (world, w, h);
    let hit = WC.with(|c| {
        let mut c = c.borrow_mut();
        touch_group(&mut c, g);
        c.world.get(&g).and_then(|m| m.get(rest)).cloned()
    });
    if let Some(s) = hit {
        return s;
    }
    let surf = build();
    WC.with(|c| {
        c.borrow_mut().world.entry(g).or_default().insert(rest.to_string(), surf.clone());
    });
    surf
}

/// `key in _cache` fuer eine Weltflaeche.
fn has_w(world: i64, w: i64, h: i64, rest: &str) -> bool {
    WC.with(|c| c.borrow().world.get(&(world, w, h)).is_some_and(|m| m.contains_key(rest)))
}

/// `_cache[key] = img` fuer eine Weltflaeche (ohne Gruppenpflege).
fn put_w(world: i64, w: i64, h: i64, rest: &str, img: Surface) {
    WC.with(|c| {
        c.borrow_mut().world.entry((world, w, h)).or_default().insert(rest.to_string(), img);
    });
}

/// `_cached(key, build)` - kleines Teil (ab SMALL_MAX Eintraegen leeren).
fn cached_s(key: String, build: impl FnOnce() -> Surface) -> Surface {
    if let Some(s) = WC.with(|c| c.borrow().small.get(&key).cloned()) {
        return s;
    }
    WC.with(|c| {
        let mut c = c.borrow_mut();
        if c.small.len() >= SMALL_MAX {
            c.small.clear();
        }
    });
    let surf = build();
    WC.with(|c| c.borrow_mut().small.insert(key, surf.clone()));
    surf
}

/// `clear_cache()`
pub fn clear_cache() {
    WC.with(|c| {
        let mut c = c.borrow_mut();
        c.world.clear();
        c.groups.clear();
        c.small.clear();
        c.buf = None;
    });
}

/// `name_key(world)` - Text-Schluessel des Weltnamens (lcx.world_<name>).
pub fn name_key(world: i32) -> String {
    format!("lcx.world_{}", NAMES[world_of(world as i64) as usize])
}

// ============================================================= Helfer

/// `_world(world)` - Weltnummer auf 0..5 begrenzt.
fn world_of(world: i64) -> i64 {
    0.max(5.min(world))
}

/// Pythons `a // b` fuer ganze Zahlen.
fn fd(a: i64, b: i64) -> i64 {
    py::fdiv(a, b)
}

/// Pythons `a % b` fuer ganze Zahlen.
fn md(a: i64, b: i64) -> i64 {
    py::pmod(a, b)
}

/// Pythons `a // b` fuer Kommazahlen (CPythons `_float_div_mod`: ueber
/// fmod, danach auf die naechste ganze Zahl geschnappt).
fn floordiv(a: f64, b: f64) -> f64 {
    let m = a % b;
    let mut div = (a - m) / b;
    if m != 0.0 && ((b < 0.0) != (m < 0.0)) {
        div -= 1.0;
    }
    if div != 0.0 {
        let mut fl = div.floor();
        if div - fl > 0.5 {
            fl += 1.0;
        }
        fl
    } else {
        0.0f64.copysign(a / b)
    }
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

/// `_lerp(c1, c2, num, den)` - ganzzahlige Farbmischung (abgerundet).
fn lerp(c1: C4, c2: C4, num: i64, den: i64) -> C4 {
    let den = 1.max(den);
    [
        c1[0] + fd((c2[0] - c1[0]) * num, den),
        c1[1] + fd((c2[1] - c1[1]) * num, den),
        c1[2] + fd((c2[2] - c1[2]) * num, den),
        c1[3] + fd((c2[3] - c1[3]) * num, den),
    ]
}

fn c4(c: C4) -> (i32, i32, i32, i32) {
    (c[0] as i32, c[1] as i32, c[2] as i32, c[3] as i32)
}

fn c3(c: C4) -> Rgb {
    (c[0] as i32, c[1] as i32, c[2] as i32)
}

fn rgba(c: Rgb, a: i64) -> (i32, i32, i32, i32) {
    (c.0, c.1, c.2, a as i32)
}

fn ck(c: Rgb) -> String {
    format!("{},{},{}", c.0, c.1, c.2)
}

/// `_hash(i)` - Ganzzahl-Streuwert 0..2^32-1 (deterministisch).
fn hash(i: i64) -> i64 {
    let v = (i as i128 * 2_654_435_761 + 40_503).rem_euclid(4_294_967_296) as u64;
    let v = (v ^ (v >> 15)).wrapping_mul(2_246_822_519) % 4_294_967_296;
    (v ^ (v >> 13)) as i64
}

/// `_wrap(v, p)` - v modulo p im Bereich 0 .. p, auch fuer negative v.
fn wrap(v: f64, p: f64) -> f64 {
    v - p * (v / p).floor()
}

/// `_wave(x, w, sd)` - weiche Wellenlinie -1..1 ueber die Breite w.
fn wave(x: i64, w: i64, sd: (f64, f64, f64)) -> f64 {
    let a = TAU * x as f64 / 1.max(w) as f64;
    0.55 * (a * 2.0 + sd.0).sin() + 0.30 * (a * 5.0 + sd.1).sin() + 0.15 * (a * 11.0 + sd.2).sin()
}

/// `_grad(s, y0, hh, w, stops)` - senkrechter Verlauf ueber hh Zeilen ab y0.
fn grad(s: &mut Surface, y0: i64, hh: i64, w: i64, stops: &[Stop]) {
    let den = 1.max(hh - 1);
    let mut k = 0usize;
    for y in 0..hh {
        let p = fd(y * 1000, den);
        while k + 2 < stops.len() && p > stops[k + 1].0 {
            k += 1;
        }
        let (p0, cc0) = stops[k];
        let (p1, cc1) = stops[k + 1];
        s.fill_r(c4(lerp(cc0, cc1, p - p0, p1 - p0)), (0, y0 + y, w, 1));
    }
}

/// `_glow_new(r, col, amax)` - weicher Lichthof 2r x 2r (nicht gecacht).
fn glow_new(r: i64, col: Rgb, amax: i64) -> Surface {
    let r = 1.max(r);
    let n = 4.max(40.min(fd(r, 2)));
    let mut s = pg::surface_alpha(2 * n, 2 * n);
    for i in (1..=n).rev() {
        let q = n - i + 1;
        draw::circle(&mut s, rgba(col, fd(amax * q * q, n * n)), (n, n), i, 0);
    }
    transform::smoothscale(&s, 2 * r, 2 * r)
}

/// `_disc(r, col)` - glatte Scheibe (2r+2)^2 (4-fach ueberabgetastet).
fn disc(r: i64, col: Rgb) -> Surface {
    let r = 1.max(r);
    cached_s(format!("disc|{r}|{}", ck(col)), || {
        let ss = 4;
        let n = (2 * r + 2) * ss;
        let mut s = pg::surface_alpha(n, n);
        draw::circle(&mut s, col, (fd(n, 2), fd(n, 2)), r * ss, 0);
        transform::smoothscale(&s, 2 * r + 2, 2 * r + 2)
    })
}

/// `_sparkle(r, col)` - Glanzstern (2r+1)^2: weicher Kern mit vier Strahlen.
fn sparkle(r: i64, col: Rgb) -> Surface {
    let r = 1.max(r);
    cached_s(format!("spark|{r}|{}", ck(col)), || {
        let ss = 4;
        let n = (2 * r + 1) * ss;
        let c = fd(n, 2);
        let mut s = pg::surface_alpha(n, n);
        let g = glow_new(2.max(fd(r * ss * 3, 4)), col, 110);
        s.bl(&g, (c - fd(g.w() as i64, 2), c - fd(g.h() as i64, 2)));
        let (rr, th) = (r * ss, 1.max(fd(ss * 3, 4)));
        draw::polygon(&mut s, rgba(col, 190), &[(c - rr, c), (c, c - th), (c + rr, c), (c, c + th)], 0);
        draw::polygon(&mut s, rgba(col, 190), &[(c, c - rr), (c + th, c), (c, c + rr), (c - th, c)], 0);
        draw::circle(&mut s, (255, 255, 255, 255), (c, c), 2.max(ss), 0);
        transform::smoothscale(&s, 2 * r + 1, 2 * r + 1)
    })
}

/// `_light(r, col)` - Positionslicht: Lichthof (2r)^2 mit hellem Kern.
fn light(r: i64, col: Rgb) -> Surface {
    let r = 2.max(r);
    cached_s(format!("light|{r}|{}", ck(col)), || {
        let mut s = glow_new(r, col, 150);
        let core = disc(1.max(fd(r, 4)), (255.min(col.0 + 60), 255.min(col.1 + 60), 255.min(col.2 + 60)));
        s.bl(&core, (r - fd(core.w() as i64, 2), r - fd(core.h() as i64, 2)));
        s
    })
}

/// `_blit_a(s, img, pos, a)` - Blit mit Deckkraft a (0..255).
fn blit_a(s: &mut Surface, img: &Surface, pos: (i64, i64), a: i64) {
    if a <= 0 {
        return;
    }
    if a < 255 {
        let mut im = img.clone();
        im.alpha_set(a);
        s.bl(&im, pos);
        im.alpha_set(255);
    } else {
        s.bl(img, pos);
    }
}

/// `_wblit(dst, img, x, y)` - Blit mit senkrechtem Umlauf.
fn wblit(dst: &mut Surface, img: &Surface, x: i64, y: i64) {
    let h = dst.h() as i64;
    dst.bl(img, (x, y));
    if y < 0 {
        dst.bl(img, (x, y + h));
    }
    if y + img.h() as i64 > h {
        dst.bl(img, (x, y - h));
    }
}

/// `_mask(size, c, r)` - weisse Kreis-Maske (BLEND_RGBA_MIN).
fn circle_mask(size: i64, c: (i64, i64), r: i64) -> Surface {
    let mut m = pg::surface_alpha(size, size);
    draw::circle(&mut m, (255, 255, 255, 255), c, r, 0);
    m
}

/// `_shade(size, lx, ly, rad, col, amax, lit, steps)` - Kugel-Schatten.
#[allow(clippy::too_many_arguments)]
fn shade(size: i64, lx: i64, ly: i64, rad: i64, col: Rgb, amax: i64, lit: i64, steps: i64) -> Surface {
    let mut sh = pg::surface_alpha(size, size);
    sh.fill_c(rgba(col, amax));
    let i0 = fd(steps * lit, 1000);
    for i in (1..=steps).rev() {
        let q = 0.max(i - i0);
        draw::circle(&mut sh, rgba(col, fd(amax * q * q, (steps - i0) * (steps - i0))), (lx, ly), 1.max(fd(rad * i, steps)), 0);
    }
    sh
}

// ============================================================= Steigen

/// `rise_px(alt, h)` - Steighoehe in Pixeln zur Hoehe alt (gebrochener
/// Takt): sanftes Anfahren, danach h * RISE / 10000 Pixel je Takt.
pub fn rise_px(alt: f64, h: i32) -> f64 {
    rise(alt, h as i64)
}

fn rise(alt: f64, h: i64) -> f64 {
    let a = fmax(0.0, alt);
    let d = if a < EASE as f64 { a * a / (2 * EASE) as f64 } else { a - EASE as f64 / 2.0 };
    d * h as f64 * RISE as f64 / 10000.0
}

/// `ground_h(h)` - Hoehe des Bodenstreifens (30 % der Buehnenhoehe).
pub fn ground_h(h: i32) -> i32 {
    gh_of(h as i64) as i32
}

fn gh_of(h: i64) -> i64 {
    8.max(fd(h * GROUND, 100))
}

/// `_pad_top(h)` - Deck-Oberkante (Korbboden) im Bodenstreifen.
fn pad_top(h: i64) -> i64 {
    let gh = gh_of(h);
    fd(gh, 3).max(gh - 16.max(fd(h * 5, 100)))
}

// ============================================================= Himmel

/// `_sun(s, w, h, world)` - Sonne mit weichem Lichthof oben links.
fn sun(s: &mut Surface, w: i64, h: i64, world: i64) {
    let (cx, cy) = (fd(w * 16, 100), fd(h * 17, 100));
    let rg = 8.max(fd(h * 36, 100));
    let halo = glow_new(rg, (255, 244, 212), if world == 0 { 120 } else { 100 });
    s.bl(&halo, (cx - rg, cy - rg));
    let r = 3.max(fd(h * 40, 1000));
    let inner = glow_new(r * 3, (255, 250, 232), 210);
    s.bl(&inner, (cx - r * 3, cy - r * 3));
    let d = disc(r, (255, 252, 238));
    s.bl(&d, (cx - r - 1, cy - r - 1));
}

/// `sky(world, w, h)` - Himmelsverlauf der Welt (opak), Wiese/Wolken mit
/// Sonne.
pub fn sky(world: i32, w: i32, h: i32) -> Surface {
    sky_of(world_of(world as i64), 1.max(w as i64), 1.max(h as i64))
}

fn sky_of(world: i64, w: i64, h: i64) -> Surface {
    cached_w(world, w, h, "sky", || {
        let mut s = pg::surface(w, h);
        grad(&mut s, 0, h, w, SKY[world as usize]);
        if world <= 1 {
            sun(&mut s, w, h, world);
        }
        s
    })
}

// ============================================================= Wolken

/// `_cloud(cw, ch, seed, pal, storm)` - Haufenwolke cw x ch.
fn cloud(cw: i64, ch: i64, seed: i64, pal: [Rgb; 3], storm: bool) -> Surface {
    let ss = 2;
    let (ww, hh) = (4.max(cw * ss), 4.max(ch * ss));
    let mut big = pg::surface_alpha(ww, hh);
    let [lgt, body, shd] = pal;
    let base = fd(hh * 82, 100);
    let n = if storm { 7 + hash(seed) % 5 } else { 4 + hash(seed) % 4 };
    let mut puffs: Vec<(i64, i64, i64)> = Vec::new();
    for k in 0..n {
        let v = hash(seed * 16 + k + 1);
        let u = fd((2 * k + 1) * 1000, 2 * n) + v % 80 - 40;
        let bell = 0.max(1000 - (u - 500).abs() * 2);
        let r = if storm {
            2.max(fd(hh * (150 + fd(bell * 170, 1000) + (v >> 9) % 70), 1000))
        } else {
            2.max(fd(hh * (170 + fd(bell * 250, 1000) + (v >> 9) % 60), 1000))
        };
        let x = fd(ww * 14, 100) + fd(fd(ww * 72, 100) * u, 1000);
        puffs.push((x, base - fd(r * 45, 100), r));
    }
    if storm {
        for j in 0..(2 + hash(seed + 5) % 2) {
            let v = hash(seed * 16 + 40 + j);
            let r = 2.max(fd(hh * (200 + (v >> 9) % 80), 1000));
            puffs.push((fd(ww * (32 + v % 36), 100), (r + 1).max(base - fd(hh * (40 + (v >> 20) % 12), 100)), r));
        }
    }
    let (lx, ly): (i64, i64) = if storm { (0, -20) } else { (-10, -17) };
    for &(x, y, r) in &puffs {
        draw::circle(&mut big, shd, (x, y), r, 0);
    }
    let (x0, x1) = (puffs[0].0, puffs[n as usize - 1].0);
    let rb = 2.max(puffs[0].2.min(puffs[n as usize - 1].2));
    draw::rect_r(&mut big, shd, (x0 - fd(rb, 2), base - rb, x1 - x0 + rb, rb), 0, fd(rb, 2) as i32);
    for &(x, y, r) in &puffs {
        draw::circle(&mut big, lgt, (x + fd(r * lx, 100), y + fd(r * ly, 100)), fd(r * 84, 100), 0);
    }
    for &(x, y, r) in &puffs {
        draw::circle(&mut big, body, (x + fd(r * (lx + 8), 100), y + fd(r * (ly + 9), 100)), fd(r * 80, 100), 0);
    }
    big.fill_r((0, 0, 0, 0), (0, base, ww, hh - base));
    transform::smoothscale(&big, 1.max(cw), 1.max(ch))
}

/// `_rain_shaft(sw, sh, seed, col)` - Regenvorhang unter einer Gewitterwolke.
fn rain_shaft(sw: i64, sh: i64, seed: i64, col: Rgb) -> Surface {
    let mut s = pg::surface_alpha(2.max(sw), 2.max(sh));
    for k in 0..2.max(fd(sw, 3)) {
        let v = hash(seed * 64 + k);
        let x = fd(sw * k, 1.max(fd(sw, 3))) + v % 3;
        draw::line(
            &mut s,
            rgba(col, 40 + (v >> 8) % 90),
            (x, 0),
            (x - fd(sh * 18, 100), sh),
            (1 + (v >> 16) % 2) as i32,
        );
    }
    let mut fade = pg::surface_alpha(1, 2.max(sh));
    grad(&mut fade, 0, 2.max(sh), 1, &[(0, [255, 255, 255, 255]), (500, [255, 255, 255, 150]), (1000, [255, 255, 255, 0])]);
    let (fw, fh) = (s.w(), s.h());
    s.bl_flags(&transform::scale(&fade, fw, fh), (0, 0), BLEND_RGBA_MULT);
    s
}

/// `_stratus_on(s, seed, w, h, col, cnt, alpha)` - Schichtwolken-Baender.
#[allow(clippy::too_many_arguments)]
fn stratus_on(s: &mut Surface, seed: i64, w: i64, h: i64, col: Rgb, cnt: i64, alpha: i64) {
    let n = 1.max(cnt);
    for k in 0..n {
        let v = hash(seed + 71 * k + 3);
        let bh = 4.max(fd(h * (60 + v % 50), 1000));
        let yc = fd(k * h, n) + fd(((v >> 8) % 1000) * h, n * 1000);
        let mut band = pg::surface_alpha(w + bh * 4, bh * 3);
        let m = 4.max(fd(w, 1.max(bh * 2)));
        for j in 0..m + 3 {
            let u = hash(seed + 900 + k * 97 + j);
            let ew = fd(bh * (30 + u % 30), 10);
            let eh = fd(bh * (12 + (u >> 8) % 12), 10);
            let ex = fd((w + bh * 4) * j, m + 2) - fd(ew, 2);
            draw::ellipse(&mut band, col, (ex, fd(bh * 3, 2) - fd(eh, 2) + fd(((u >> 16) % 5 - 2) * bh, 8), ew, eh), 0);
        }
        band.alpha_set(alpha);
        wblit(s, &band, -bh * 2, yc - fd(bh * 3, 2));
    }
}

/// `_wisp(ww, wh, seed, col)` - Federwolke (2-fach ueberabgetastet).
fn wisp(ww: i64, wh: i64, seed: i64, col: Rgb) -> Surface {
    let ss = 2;
    let (bw, bh) = (4.max(ww * ss), 4.max(wh * ss));
    let mut big = pg::surface_alpha(bw, bh);
    let n = 5 + hash(seed) % 3;
    for k in 0..n {
        let v = hash(seed * 8 + k + 1);
        let lw = fd(bw * (34 + v % 30), 100);
        let lh = ss.max(fd(bh * (16 + (v >> 8) % 14), 100));
        let x = fd((bw - lw) * k, n - 1);
        let mut y = fd((bh - lh) * k, n - 1) + (v >> 16) % 1.max(fd(bh, 6)) - fd(bh, 12);
        y = 0.max((bh - lh).min(y));
        let a = 110 + (v >> 20) % 110;
        draw::ellipse(&mut big, rgba(col, fd(a, 2)), (x, y, lw, lh), 0);
        draw::ellipse(&mut big, rgba(col, a), (x + fd(lw, 6), y + fd(lh, 4), fd(lw * 2, 3), 1.max(fd(lh, 2))), 0);
    }
    transform::smoothscale(&big, 1.max(ww), 1.max(wh))
}

/// `_clouds_on(s, seed, w, h, spec, storm)` - Wolken einer Ebene.
fn clouds_on(s: &mut Surface, seed: i64, w: i64, h: i64, spec: CloudSpec, storm: bool) {
    let (cnt, wmin, wmax, hr, pal, alpha, sides) = spec;
    let n = 1.max(fd(cnt * w + fd(h, 2), h));
    for k in 0..n {
        let v = hash(seed + 31 * k);
        let cw = 8.max(fd(h * (wmin + v % (wmax - wmin + 1)), 1000));
        let ch = 4.max(fd(cw * hr, 100));
        let y = fd(k * h, n) + fd(((v >> 10) % 1000) * h, n * 1000);
        let fx = if sides {
            let f = (v >> 20) % 300 - 100;
            if k % 2 != 0 {
                1000 - f
            } else {
                f
            }
        } else {
            fd(md(k * 618 + (v >> 20) % 200, 1000) * 12, 10) - 100
        };
        let mut img = cloud(cw, ch, v % 100000, pal, storm);
        img.alpha_set(alpha);
        let x = fd(w * fx, 1000) - fd(cw, 2);
        if storm && (v >> 28) % 3 != 0 {
            // Regenvorhang unter der Wolke (haengt mit ihr in der Ebene)
            let (sw, sh) = (fd(cw * 55, 100), fd(ch * 16, 10));
            let shaft = rain_shaft(sw, sh, v % 100000, pal[0]);
            wblit(s, &shaft, x + fd(cw * 26, 100), y - fd(ch, 2) + fd(ch * 74, 100));
        }
        wblit(s, &img, x, y - fd(ch, 2));
    }
}

/// `_wisps_on(s, seed, w, h, col, cnt, alpha)` - Federwolken einer Ebene.
#[allow(clippy::too_many_arguments)]
fn wisps_on(s: &mut Surface, seed: i64, w: i64, h: i64, col: Rgb, cnt: i64, alpha: i64) {
    let n = 1.max(fd(cnt * w + fd(h, 2), h));
    for k in 0..n {
        let v = hash(seed + 53 * k + 7);
        let ww = 12.max(fd(h * (420 + v % 360), 1000));
        let wh = 4.max(fd(ww * (7 + (v >> 10) % 5), 100));
        let y = fd(k * h, n) + fd(((v >> 14) % 1000) * h, n * 1000);
        let x = fd(w * md(k * 618 + (v >> 24) % 150, 1000), 1000);
        let mut img = wisp(ww, wh, v % 100000, col);
        img.alpha_set(alpha);
        wblit(s, &img, x - fd(ww, 2), y - fd(wh, 2));
    }
}

// ============================================================= Sterne

/// `_stars_on(s, seed, n, w, h, amin, amax, big, two)` - n Sterne.
#[allow(clippy::too_many_arguments)]
fn stars_on(s: &mut Surface, seed: i64, n: i64, w: i64, h: i64, amin: i64, amax: i64, big: i64, two: i64) {
    let rs = 2.max(fd(h, 190));
    for i in 0..n {
        let v = hash(seed + 2 * i);
        let u = hash(seed + 2 * i + 1);
        let (x, y) = (v % w, u % h);
        let col = STAR_COLS[((v >> 20) % 4) as usize];
        let a = amin + (u >> 12) % (amax - amin + 1);
        let p = (u >> 22) % 1000;
        if p < big {
            let mut spr = sparkle(rs + (v >> 26) % 2, col);
            spr.alpha_set(a);
            let (sw, sh) = (spr.w() as i64, spr.h() as i64);
            wblit(s, &spr, x - fd(sw, 2), y - fd(sh, 2));
            spr.alpha_set(255);
        } else if p < big + two {
            s.fill_r(rgba(col, fd(a * 3, 4)), (x, y, 2, 2));
        } else {
            s.fill_r(rgba(col, a), (x, y, 1, 1));
        }
    }
}

/// `_milky(s, seed, w, h)` - Milchstrasse und zwei Nebel.
fn milky(s: &mut Surface, seed: i64, w: i64, h: i64) {
    let q = 4;
    let mut lo = pg::surface_alpha(fd(w, q) + 2, fd(h, q) + 2);
    let (y0, y1, th) = (fd(h * 62, 100), fd(h * 36, 100), fd(h * 10, 100));
    let n = 6.max(fd(w, 36));
    let cols: [Rgb; 3] = [(150, 140, 230), (116, 158, 232), (190, 150, 222)];
    for k in 0..n {
        let v = hash(seed + 7 * k);
        let x = fd(w * k, n - 1) + fd(((v % 41) - 20) * w, 1000);
        let yc = y0 + fd((y1 - y0) * x, w) + fd(((v >> 8) % 61 - 30) * th, 100);
        let r = 2.max(fd(fd(th * [60, 80, 100, 120][((v >> 16) % 4) as usize], 100), q));
        let g = glow_new(r, cols[((v >> 24) % 3) as usize], 22 + (v >> 4) % 14);
        lo.bl(&g, (fd(x, q) - r, fd(yc, q) - r));
    }
    let nebel: [(i64, i64, Rgb, Rgb); 2] = [
        (fd(w * 22, 100), fd(h * 25, 100), (200, 90, 190), (120, 90, 220)),
        (fd(w * 80, 100), fd(h * 72, 100), (70, 160, 210), (90, 120, 220)),
    ];
    for (j, &(cx, cy, col, col2)) in nebel.iter().enumerate() {
        for k in 0..16 {
            let v = hash(seed + 900 + j as i64 * 50 + k);
            let r = 2.max(fd(fd(h * (60 + v % 60), 1000), q));
            let dx = fd(fd(((v >> 8) % 240 - 120) * h, 1000), q);
            let dy = fd(fd(((v >> 16) % 120 - 60) * h, 1000), q);
            let g = glow_new(r, if k % 3 != 0 { col } else { col2 }, 9 + (v >> 24) % 8);
            lo.bl(&g, (fd(cx, q) + dx - r, fd(cy, q) + dy - r));
        }
    }
    s.bl(&transform::smoothscale(&lo, (fd(w, q) + 2) * q, (fd(h, q) + 2) * q), (0, 0));
    let m = fd(w * th, 70);
    for i in 0..m {
        let v = hash(seed + 5000 + 2 * i);
        let u = hash(seed + 5001 + 2 * i);
        let x = v % w;
        let d = fd(((u % 1000) + ((u >> 10) % 1000) - 1000) * th, 1000);
        let y = y0 + fd((y1 - y0) * x, w) + d;
        s.fill_r((226, 230, 255, (40 + (u >> 20) % 130) as i32), (x, y, 1, 1));
    }
}

// ============================================================= Ebenen

fn build_layer(world: i64, depth: i64, w: i64, h: i64) -> Surface {
    let mut s = pg::surface_alpha(w, h);
    let seed = 1000 * (world * 3 + depth) + 17;
    if world == 2 && depth == 0 {
        stratus_on(&mut s, seed + 300, w, h, (50, 54, 74), 3, 150);
    }
    if let Some(spec) = clouds_spec(world, depth) {
        clouds_on(&mut s, seed, w, h, spec, world == 2);
    }
    if world == 0 && depth == 0 {
        wisps_on(&mut s, seed, w, h, (255, 255, 255), 3, 150);
    }
    if world == 4 && depth == 0 {
        milky(&mut s, seed + 400, w, h);
    }
    if world == 3 && depth == 2 {
        wisps_on(&mut s, seed, w, h, (120, 150, 210), 2, 56);
    }
    if let Some((per, amin, amax, big, two)) = stars_spec(world, depth) {
        stars_on(&mut s, seed + 100, 1.max(fd(w * h, per)), w, h, amin, amax, big, two);
    }
    s
}

/// `layer(world, depth, w, h)` - kachelbare Ebene der Welt (depth 0 fern ..
/// 2 nah), w x h, SRCALPHA; Mond leer.
pub fn layer(world: i32, depth: i32, w: i32, h: i32) -> Surface {
    let depth = 0.max(2.min(depth as i64));
    layer_of(world_of(world as i64), depth, 1.max(w as i64), 1.max(h as i64))
}

fn layer_of(world: i64, depth: i64, w: i64, h: i64) -> Surface {
    cached_w(world, w, h, &format!("layer|{depth}"), || build_layer(world, depth, w, h))
}

/// `_tile(s, rect, img, off)` - kachelbare Ebene senkrecht um off verschoben.
fn tile(s: &mut Surface, rect: Rect, img: &Surface, off: f64) {
    let lh = img.h() as i64;
    let y = md(off as i64, lh);
    if y != 0 {
        s.bl_area(img, (rect.x, rect.y), (0, lh - y, rect.w, y));
    }
    s.bl_area(img, (rect.x as i64, rect.y as i64 + y), (0, 0, rect.w as i64, lh - y));
}

// ============================================================= Boden

/// `_barn(s, x0, yb, bw, ss)` - Lama-Stall: rote Scheune von vorn.
fn barn(s: &mut Surface, x0: i64, yb: i64, bw: i64, _ss: i64) {
    let bh = fd(bw * 50, 100);
    let rf = fd(bw * 42, 100);
    let (yw, yk, yr) = (yb - bh, yb - bh - fd(rf * 55, 100), yb - bh - rf);
    let (d, dy) = (fd(bw * 20, 100), fd(bw * 6, 100));
    let (red, red_d, red_s): (Rgb, Rgb, Rgb) = ((190, 60, 50), (158, 46, 40), (138, 40, 36));
    let (trim, roof, roof_d): (Rgb, Rgb, Rgb) = ((246, 242, 232), (96, 70, 64), (70, 50, 48));
    let lw = 1.max(fd(bw, 55));
    let lwi = lw as i32;
    let (xk0, xm, xk1) = (x0 + fd(bw * 14, 100), x0 + fd(bw, 2), x0 + fd(bw * 86, 100));
    let gable = |x: i64| -> i64 {
        if x <= xk0 {
            return yw + fd((yk - yw) * (x - x0), 1.max(xk0 - x0));
        }
        if x <= xm {
            return yk + fd((yr - yk) * (x - xk0), 1.max(xm - xk0));
        }
        if x <= xk1 {
            return yr + fd((yk - yr) * (x - xm), 1.max(xk1 - xm));
        }
        yk + fd((yw - yk) * (x - xk1), 1.max(x0 + bw - xk1))
    };
    draw::ellipse(s, (40, 96, 40, 90), (x0 - fd(bw, 10), yb - fd(bw, 16), fd(bw * 13, 10) + d, fd(bw, 8)), 0);
    // Seitenwand und Dach nach hinten rechts
    draw::polygon(s, red_s, &[(x0 + bw, yb), (x0 + bw, yw), (x0 + bw + d, yw - dy), (x0 + bw + d, yb - dy)], 0);
    draw::polygon(s, roof_d, &[(xk1, yk), (x0 + bw, yw), (x0 + bw + d, yw - dy), (xk1 + d, yk - dy)], 0);
    draw::polygon(s, roof, &[(xm, yr), (xk1, yk), (xk1 + d, yk - dy), (xm + d, yr - dy)], 0);
    let (wx, wy, ws) = (x0 + bw + fd(d * 3, 10), yw + fd(bh * 3, 10), 2.max(fd(d * 4, 10)));
    draw::rect(s, (60, 36, 32), (wx, wy - fd(dy, 2), ws, ws), 0);
    draw::rect(s, trim, (wx, wy - fd(dy, 2), ws, ws), lwi);
    // Giebelwand
    draw::polygon(s, red, &[(x0, yb), (x0, yw), (xk0, yk), (xm, yr), (xk1, yk), (x0 + bw, yw), (x0 + bw, yb)], 0);
    for k in 1..12 {
        let x = x0 + fd(bw * k, 12);
        draw::line(s, red_d, (x, gable(x) + lw * 2), (x, yb), lwi);
    }
    // Dachkante + Zierleiste, Ecken
    let edge = [
        (x0 - fd(bw * 5, 100), yw + lw * 3),
        (xk0 - lw, yk - lw * 2),
        (xm, yr - lw * 3),
        (xk1 + lw, yk - lw * 2),
        (x0 + fd(bw * 105, 100), yw + lw * 3),
    ];
    draw::lines(s, roof, false, &edge, 2.max(fd(bw, 14)) as i32);
    draw::lines(s, trim, false, &[(x0, yw), (xk0, yk), (xm, yr), (xk1, yk), (x0 + bw, yw)], lwi * 2);
    draw::line(s, trim, (x0 + lw, yw), (x0 + lw, yb), lwi * 2);
    draw::line(s, trim, (x0 + bw - lw, yw), (x0 + bw - lw, yb), lwi * 2);
    // Tor mit weissem X
    let (dw, dh) = (fd(bw * 40, 100), fd(bh * 72, 100));
    let dx = x0 + fd(bw - dw, 2);
    draw::rect(s, red_d, (dx, yb - dh, dw, dh), 0);
    for (a, b) in [
        ((dx, yb - dh), (dx + fd(dw, 2), yb)),
        ((dx + fd(dw, 2), yb - dh), (dx, yb)),
        ((dx + fd(dw, 2), yb - dh), (dx + dw, yb)),
        ((dx + dw, yb - dh), (dx + fd(dw, 2), yb)),
    ] {
        draw::line(s, trim, a, b, lwi * 2);
    }
    draw::line(s, trim, (dx + fd(dw, 2), yb - dh), (dx + fd(dw, 2), yb), lwi * 2);
    draw::rect(s, trim, (dx, yb - dh, dw, dh), lwi * 2);
    // Heuluke mit Heu
    let hs = fd(bw * 22, 100);
    let (hx, hy) = (x0 + fd(bw - hs, 2), yw - fd(rf * 40, 100));
    draw::rect(s, (66, 38, 30), (hx, hy, hs, hs), 0);
    draw::polygon(
        s,
        (236, 200, 92),
        &[
            (hx + lw, hy + hs),
            (hx + fd(hs, 5), hy + fd(hs * 55, 100)),
            (hx + fd(hs * 2, 5), hy + fd(hs * 75, 100)),
            (hx + fd(hs * 3, 5), hy + fd(hs * 50, 100)),
            (hx + fd(hs * 4, 5), hy + fd(hs * 72, 100)),
            (hx + hs - lw, hy + hs),
        ],
        0,
    );
    draw::rect(s, trim, (hx, hy, hs, hs), lwi * 2);
    // Wetterfahne
    draw::line(s, (70, 64, 70), (xm, yr - lw * 3), (xm, yr - fd(rf * 40, 100)), lwi);
    draw::polygon(
        s,
        (70, 64, 70),
        &[
            (xm - fd(bw, 14), yr - fd(rf * 34, 100)),
            (xm + fd(bw, 10), yr - fd(rf * 36, 100)),
            (xm + fd(bw, 14), yr - fd(rf * 30, 100)),
        ],
        0,
    );
}

/// `_bale(s, cx, cy, r, ss)` - Rundballen Heu.
fn bale(s: &mut Surface, cx: i64, cy: i64, r: i64, ss: i64) {
    draw::ellipse(s, (40, 96, 40, 90), (cx - fd(r * 11, 10), cy + fd(r * 6, 10), fd(r * 22, 10), fd(r, 2)), 0);
    draw::circle(s, (204, 164, 74), (cx, cy), r, 0);
    draw::circle(s, (234, 200, 106), (cx - fd(r, 10), cy - fd(r, 10)), fd(r * 86, 100), 0);
    for k in [64, 42, 20] {
        draw::circle(s, (198, 158, 70), (cx - fd(r, 10), cy - fd(r, 10)), 1.max(fd(r * k, 100)), 1.max(ss) as i32);
    }
}

/// `_tree(s, x, yb, size, ss)` - Laubbaum: Stamm und runde Krone.
fn tree(s: &mut Surface, x: i64, yb: i64, size: i64, ss: i64) {
    let tw = ss.max(fd(size, 9));
    draw::ellipse(s, (40, 96, 40, 90), (x - fd(size, 2), yb - fd(size, 18), size, fd(size, 9)), 0);
    draw::rect(s, (112, 80, 54), (x - fd(tw, 2), yb - fd(size * 55, 100), tw, fd(size * 55, 100)), 0);
    let (shd, body, lgt): (Rgb, Rgb, Rgb) = ((52, 118, 60), (72, 148, 70), (112, 182, 92));
    for (dx, dy, r, col) in [
        (-20, -60, 28, shd),
        (20, -58, 28, shd),
        (0, -80, 32, shd),
        (-14, -66, 24, body),
        (14, -64, 23, body),
        (-2, -84, 26, body),
        (-12, -88, 13, lgt),
        (-22, -70, 10, lgt),
    ] {
        draw::circle(s, col, (x + fd(size * dx, 100), yb + fd(size * dy, 100)), 1.max(fd(size * r, 100)), 0);
    }
}

/// `_windsock(s, x, yb, hh, ss)` - Windsack (rot-weiss) am Mast.
fn windsock(s: &mut Surface, x: i64, yb: i64, hh: i64, ss: i64) {
    let top = yb - hh;
    draw::line(s, (196, 198, 206), (x, yb), (x, top), ss.max(fd(hh, 45)) as i32);
    let ln = fd(hh * 38, 100);
    let (h0, h1) = (fd(hh * 15, 100), fd(hh * 8, 100));
    for k in 0..4 {
        let (xa, xb) = (x + fd(ln * k, 4), x + fd(ln * (k + 1), 4));
        let (ha, hb) = (h0 + fd((h1 - h0) * k, 4), h0 + fd((h1 - h0) * (k + 1), 4));
        let (da, db) = (fd(hh * 2 * k, 100), fd(hh * 2 * (k + 1), 100));
        let col: Rgb = if k % 2 == 0 { (232, 52, 76) } else { (250, 248, 240) };
        draw::polygon(
            s,
            col,
            &[
                (xa, top + da - fd(ha, 2) + fd(h0, 2)),
                (xb, top + db - fd(hb, 2) + fd(h0, 2)),
                (xb, top + db + fd(hb, 2) + fd(h0, 2)),
                (xa, top + da + fd(ha, 2) + fd(h0, 2)),
            ],
            0,
        );
    }
    draw::circle(s, (230, 230, 236), (x, top), ss.max(fd(hh, 28)), 0);
}

/// `_pad(s, cx, yd, pw, G, ss)` - Startplatz: rundes Holzpodest mit Treppe,
/// rotem Startkreis, Wimpelkette und zwei Faehnchen.
fn pad(s: &mut Surface, cx: i64, yd: i64, pw: i64, g: i64, ss: i64) {
    let eb = 2.max(fd(pw * 11, 100));
    let th = (ss * 2).max(fd(g * 97, 100) - yd - eb);
    let yg = yd + th + eb;
    let (wl, wm, wd): (Rgb, Rgb, Rgb) = ((220, 178, 120), (174, 124, 74), (136, 94, 56));
    let lw = 1.max(ss);
    let lwi = lw as i32;
    draw::ellipse(
        s,
        (36, 86, 36, 110),
        (cx - fd(pw * 64, 100), yd + th - fd(eb, 2), fd(pw * 132, 100), eb * 2 + fd(g * 3, 100)),
        0,
    );
    // Treppe rechts (zwei Stufen)
    for k in 0..2 {
        let sx = cx + fd(pw * 38, 100) + fd(k * pw * 11, 100);
        let sy = yd + fd(th * (k + 1), 3);
        draw::rect(s, wd, (sx, sy, fd(pw * 17, 100), yg - sy), 0);
        draw::rect(s, wm, (sx, sy, fd(pw * 15, 100), yg - sy), 0);
        draw::rect(s, wl, (sx, sy, fd(pw * 15, 100), ss.max(fd(g * 15, 1000))), 0);
    }
    // Zylinder: Unterkante, Seitenflaeche mit Brettern, Deck
    draw::ellipse(s, wd, (cx - fd(pw, 2), yd + th - eb, pw, 2 * eb), 0);
    draw::rect(s, wm, (cx - fd(pw, 2), yd, pw, th), 0);
    for k in 1..8 {
        let x = cx - fd(pw, 2) + fd(pw * k, 8);
        let q = ((x - cx) * 2) as f64 / pw as f64;
        let e = (eb as f64 * fmax(0.0, 1.0 - q * q).sqrt()) as i64;
        draw::line(s, wd, (x, yd + e), (x, yd + th + e), lwi);
    }
    draw::ellipse(s, wl, (cx - fd(pw, 2), yd - eb, pw, 2 * eb), 0);
    draw::ellipse(s, wd, (cx - fd(pw, 2), yd - eb, pw, 2 * eb), lwi);
    for k in 1..5 {
        let y = yd - eb + fd(2 * eb * k, 5);
        let q = (y - yd) as f64 / eb as f64;
        let hw = (pw as f64 / 2.0 * fmax(0.0, 1.0 - q * q).sqrt()) as i64;
        draw::line(s, (198, 154, 100), (cx - hw + ss * 2, y), (cx + hw - ss * 2, y), lwi);
    }
    draw::ellipse(
        s,
        (232, 52, 76),
        (cx - fd(pw * 30, 100), yd - fd(eb * 56, 100), fd(pw * 60, 100), fd(eb * 112, 100)),
        ss.max(fd(pw, 36)) as i32,
    );
    // Wimpelkette an der Vorderkante
    let cols: [Rgb; 4] = [(232, 52, 76), (250, 206, 70), (250, 248, 240), (70, 140, 220)];
    let tw = (ss * 2).max(fd(pw * 9, 100));
    for k in 0..9 {
        let x = cx - fd(pw * 44, 100) + fd(pw * 88 * k, 800);
        let q = ((x - cx) * 2) as f64 / pw as f64;
        let y = yd + (eb as f64 * fmax(0.0, 1.0 - q * q).sqrt()) as i64 + lw;
        draw::polygon(s, cols[(k % 4) as usize], &[(x - fd(tw, 2), y), (x + fd(tw, 2), y), (x, y + fd(tw * 11, 10))], 0);
    }
    // zwei Faehnchen am hinteren Rand
    for (fx, col) in [(-44, (232, 52, 76)), (44, (250, 206, 70))] {
        let x = cx + fd(pw * fx, 100);
        let top = yd - fd(g * 30, 100);
        draw::line(s, (236, 236, 240), (x, yd - fd(eb, 3)), (x, top), ss.max(fd(pw, 50)) as i32);
        let fw = fd(pw * 16, 100);
        draw::polygon(s, col, &[(x, top), (x + fw, top + fd(fw * 3, 10)), (x, top + fd(fw * 6, 10))], 0);
    }
}

/// `_meadow(s, W, G, ss, h)` - alles auf dem Bodenstreifen.
fn meadow(s: &mut Surface, ww: i64, g: i64, ss: i64, h: i64) {
    let yy = |p: i64| -> i64 { fd(g * p, 1000) };
    let step = 4.max(fd(ww, 140));
    // Huegel: dunstig fern, gruener davor; Baumgruppen auf dem vorderen Kamm
    let hills: [Hill; 2] = [
        (440, 250, (158, 200, 192), (0.6, 1.9, 0.3)),
        (500, 160, (120, 182, 126), (2.2, 0.4, 1.3)),
    ];
    for (base, amp, col, sd) in hills {
        let mut pts: Vec<(i64, i64)> = vec![(0, g)];
        let mut x = 0;
        while x < ww + step {
            pts.push((x, yy(base) - (yy(amp) as f64 * (0.5 + 0.5 * wave(x, ww, sd))) as i64));
            x += step;
        }
        pts.push((ww + step, g));
        draw::polygon(s, col, &pts, 0);
    }
    let n_trees = 4.max(fd(ww, 1.max(yy(120))));
    for k in 0..n_trees {
        let v = hash(71000 + k);
        let x = fd(k * ww, n_trees) + v % 1.max(fd(ww, n_trees));
        let y = yy(500) - (yy(160) as f64 * (0.5 + 0.5 * wave(x, ww, (2.2, 0.4, 1.3)))) as i64 + yy(10);
        for j in 0..(1 + (v >> 8) % 3) {
            let r = ss.max(fd(g * (22 + (v >> (12 + j)) % 14), 1000));
            let tx = x + fd(j * r * 15, 10);
            draw::circle(s, (70, 136, 82), (tx, y - r), r, 0);
            draw::circle(s, (92, 160, 96), (tx - fd(r, 4), y - r - fd(r, 4)), fd(r * 6, 10), 0);
        }
    }
    // Wiese: Verlauf, oben leicht gewellt, mit hellem Saum
    let mut top: Vec<(i64, i64)> = Vec::new();
    let mut x = 0;
    while x < ww + step {
        top.push((x, yy(505) - (yy(26) as f64 * (0.5 + 0.5 * wave(x, ww, (1.0, 2.6, 0.2)))) as i64));
        x += step;
    }
    let mut grass = pg::surface_alpha(ww, g);
    let y0 = yy(470);
    grad(&mut grass, y0, g - y0, ww, &[(0, [146, 208, 104, 255]), (450, [110, 184, 82, 255]), (1000, [82, 154, 62, 255])]);
    let mut mask = pg::surface_alpha(ww, g);
    let mut mpts: Vec<(i64, i64)> = vec![(0, g)];
    mpts.extend(top.iter().copied());
    mpts.push((ww + step, g));
    draw::polygon(&mut mask, (255, 255, 255, 255), &mpts, 0);
    grass.bl_flags(&mask, (0, 0), BLEND_RGBA_MIN);
    s.bl(&grass, (0, 0));
    draw::lines(s, (176, 226, 128), false, &top, 1.max(ss) as i32);
    // Grasbueschel und Blumen (vorn groesser)
    let gy0 = yy(540);
    let n = fd(ww * (g - gy0), ss * ss * 230);
    let flowers: [Rgb; 4] = [(250, 250, 240), (250, 220, 80), (240, 150, 180), (190, 160, 240)];
    for i in 0..n {
        let v = hash(72000 + 2 * i);
        let u = hash(72001 + 2 * i);
        let x = v % ww;
        let y = gy0 + fd((u % 1000) * (g - gy0), 1000);
        let sz = ss * (2 + fd((y - gy0) * 5, 1.max(g - gy0)));
        if (v >> 20) % 7 == 0 {
            draw::circle(s, flowers[((u >> 12) % 4) as usize], (x, y - fd(sz, 2)), 1.max(fd(sz, 3)), 0);
        } else {
            let col: Rgb = if (u >> 14) & 1 != 0 { (78, 146, 58) } else { (100, 170, 72) };
            for dx in [fd(-sz, 2), 0, fd(sz, 2)] {
                draw::line(s, col, (x, y), (x + dx, y - sz), 1.max(fd(ss, 2)) as i32);
            }
        }
    }
    let pw = fd(h * PAD_W, 100) * ss;
    let (cx, yd) = (fd(fd(ww, ss) * PAD_X, 100) * ss, pad_top(h) * ss);
    let free = cx + fd(ww, 10); // bis hier bleibt es um den Startplatz frei
    let bw = fd(ww * 18, 100).min(fd(g * 62, 100));
    let bx = fd(ww * 71, 100);
    // Trampelpfad vom Startplatz zum Stalltor
    let (p0, p1) = (cx + fd(pw * 60, 100), bx + fd(bw, 2));
    let mut top_pts: Vec<(i64, i64)> = Vec::with_capacity(18);
    let mut bot_pts: Vec<(i64, i64)> = Vec::with_capacity(9);
    for k in 0..9 {
        let x = p0 + fd((p1 - p0) * k, 8);
        let y = yy(935) - fd((yy(935) - yy(872)) * k * k, 64);
        let half = yy(34) - fd((yy(34) - yy(14)) * k, 8);
        top_pts.push((x, y - half));
        bot_pts.push((x, y + half));
    }
    let mut path = top_pts.clone();
    path.extend(bot_pts.iter().rev());
    draw::polygon(s, (190, 162, 112), &path, 0);
    let rim: Vec<(i64, i64)> = top_pts.iter().map(|&(x, y)| (x, y + ss)).collect();
    draw::lines(s, (210, 184, 134), false, &rim, 1.max(ss) as i32);
    // Zaun quer ueber die Weide, erst rechts vom Startplatz
    let rh = ss.max(yy(20));
    for yr in [yy(588), yy(650)] {
        s.fill_r((190, 142, 92), (free, yr, ww - free, rh));
        s.fill_r((140, 100, 62), (free, yr + rh, ww - free, 1.max(ss)));
    }
    let sp = 8.max(yy(150));
    let pwid = ss.max(yy(26));
    let mut x = free;
    while x < ww {
        draw::rect(s, (160, 114, 72), (x, yy(552), pwid, yy(702) - yy(552)), 0);
        draw::rect(
            s,
            (122, 86, 54),
            (x + fd(pwid * 6, 10), yy(552), pwid - fd(pwid * 6, 10), yy(702) - yy(552)),
            0,
        );
        draw::polygon(s, (176, 128, 82), &[(x, yy(552)), (x + fd(pwid, 2), yy(534)), (x + pwid, yy(552))], 0);
        x += sp;
    }
    // Buesche, kleiner Baum in der Mitte, Windsack
    for (fx, fy, fr) in [(47, 880, 44), (51, 905, 32), (89, 935, 40), (64, 915, 28)] {
        let (ux, uy, ur) = (fd(ww * fx, 100), yy(fy), ss.max(fd(g * fr, 1000)));
        draw::circle(s, (58, 124, 62), (ux, uy - ur), ur, 0);
        draw::circle(s, (84, 156, 74), (ux - fd(ur, 4), uy - ur - fd(ur, 4)), fd(ur * 7, 10), 0);
    }
    tree(s, fd(ww * 55, 100), yy(810), fd(g * 40, 100), ss);
    windsock(s, fd(ww * 31, 100), yy(900), fd(g * 50, 100), ss);
    // Stall rechts (71 .. 90 %), Heuballen davor, grosser Baum ganz rechts
    let r = (ss * 2).max(fd(g * 6, 100));
    bale(s, bx - fd(r * 16, 10), yy(900), r, ss);
    bale(s, bx - fd(r * 38, 10), yy(925), fd(r * 9, 10), ss);
    barn(s, bx, yy(875), bw, ss);
    tree(s, fd(ww * 95, 100), yy(860), fd(g * 60, 100), ss);
    // Startplatz und Lamas (sehen dem Start zu)
    pad(s, cx, yd, pw, g, ss);
    let n = 8.max(fd(g * 24, 100));
    let lamas: [(i64, i64, i64, Rgb, bool); 3] = [
        (36, 925, 100, (60, 150, 220), true),
        (43, 960, 86, (250, 196, 60), true),
        (60, 945, 92, (150, 90, 200), false),
    ];
    for (fx, fy, k, scarf, flip) in lamas {
        let m = 8.max(fd(n * k, 100));
        let mut spr = lcd::llama_sprite(m as i32, "stand", scarf.c());
        if flip {
            spr = transform::flip(&spr, true, false);
        }
        let (x, y) = (fd(ww * fx, 100), yy(fy));
        draw::ellipse(s, (36, 86, 36, 100), (x - fd(m * 3, 10), y - fd(m, 16), fd(m * 6, 10), fd(m, 8)), 0);
        s.bl(&spr, (x - fd(m, 2), y - fd(m * 94, 100)));
    }
}

/// `ground(w, h)` - Bodenstreifen w x ground_h(h) (SRCALPHA, 2-fach
/// ueberabgetastet): Huegel mit Baeumen, Weide, Zaun, Stall, Heuballen,
/// Windsack, drei Lamas und links der Startplatz.
pub fn ground(w: i32, h: i32) -> Surface {
    ground_of(1.max(w as i64), 1.max(h as i64))
}

fn ground_of(w: i64, h: i64) -> Surface {
    cached_w(0, w, h, "ground", || {
        let gh = gh_of(h);
        let ss = 2;
        let mut big = pg::surface_alpha(w * ss, gh * ss);
        meadow(&mut big, w * ss, gh * ss, ss, h);
        transform::smoothscale(&big, w, gh)
    })
}

// ============================================================= kleine Figuren

/// `_bird(n, frame, col, flip)` - Vogel n x n (Fluegelschlag frame 0..3).
fn bird(n: i64, frame: i64, col: Rgb, flip: bool) -> Surface {
    let frame = md(frame, 4);
    cached_s(format!("bird|{n}|{frame}|{}|{flip}", ck(col)), || {
        let ss = 4;
        let nn = n * ss;
        let nf = nn as f64;
        let mut s = pg::surface_alpha(nn, nn);
        let tip = [0.10, 0.40, 0.74, 0.40][frame as usize];
        let elb = [0.30, 0.44, 0.60, 0.46][frame as usize];
        let (cx, cy) = (nf * 0.52, nf * 0.54);
        let lw = ss.max(fd(nn, 9)) as i32;
        draw::lines(&mut s, col, false, &[(nf * 0.03, nf * tip), (nf * 0.28, nf * elb), (cx, cy)], lw);
        draw::lines(&mut s, col, false, &[(cx, cy), (nf * 0.76, nf * elb), (nf * 0.97, nf * tip)], lw);
        draw::ellipse(&mut s, col, (nf * 0.38, nf * 0.48, nf * 0.30, nf * 0.13), 0);
        draw::circle(&mut s, col, (nf * 0.70, nf * 0.52), nf * 0.055, 0);
        let img = transform::smoothscale(&s, n, n);
        if flip {
            transform::flip(&img, true, false)
        } else {
            img
        }
    })
}

/// `_plane(n, flip)` - Verkehrsflugzeug von der Seite, n lang.
fn plane(n: i64, flip: bool) -> Surface {
    cached_s(format!("plane|{n}|{flip}"), || {
        let ss = 3;
        let nn = n * ss;
        let hh = fd(nn * 38, 100);
        let mut s = pg::surface_alpha(nn, hh);
        let (body, belly, dark): (Rgb, Rgb, Rgb) = ((170, 176, 194), (126, 132, 152), (84, 90, 108));
        let (fy, fh) = (fd(hh * 44, 100), fd(hh * 26, 100));
        let q = |a: i64, b: i64| fd(a * b, 100);
        draw::polygon(&mut s, dark, &[(q(nn, 1), q(hh, 52)), (q(nn, 16), q(hh, 52)), (q(nn, 19), q(hh, 60)), (0, q(hh, 62))], 0);
        draw::polygon(
            &mut s,
            (184, 56, 74),
            &[(q(nn, 4), q(hh, 52)), (0, q(hh, 4)), (q(nn, 9), q(hh, 4)), (q(nn, 25), q(hh, 50))],
            0,
        );
        draw::rect_r(&mut s, body, (q(nn, 4), fy, q(nn, 86), fh), 0, fd(fh, 2) as i32);
        draw::ellipse(&mut s, body, (q(nn, 80), fy, q(nn, 19), fh), 0);
        draw::rect_r(&mut s, belly, (q(nn, 10), fy + q(fh, 62), q(nn, 82), q(fh, 38)), 0, fd(fh, 3) as i32);
        draw::polygon(
            &mut s,
            (40, 52, 82),
            &[(q(nn, 90), fy + q(fh, 22)), (q(nn, 95), fy + q(fh, 30)), (q(nn, 95), fy + q(fh, 44)), (q(nn, 89), fy + q(fh, 44))],
            0,
        );
        for k in 0..17 {
            draw::circle(&mut s, (255, 226, 150), (q(nn, 26 + fd(k * 37, 10)), fy + q(fh, 38)), 1.max(fd(ss * 2, 3)), 0);
        }
        draw::polygon(&mut s, dark, &[(q(nn, 40), fy + q(fh, 70)), (q(nn, 58), fy + q(fh, 70)), (q(nn, 44), hh - 1), (q(nn, 36), hh - 1)], 0);
        draw::ellipse(&mut s, (70, 74, 92), (q(nn, 45), fy + fh, q(nn, 13), q(hh, 15)), 0);
        let img = transform::smoothscale(&s, n, 2.max(fd(hh, ss)));
        if flip {
            transform::flip(&img, true, false)
        } else {
            img
        }
    })
}

/// `_jet(n, flip)` - schlanker Jet als dunkle Silhouette, n lang.
fn jet(n: i64, flip: bool) -> Surface {
    cached_s(format!("jet|{n}|{flip}"), || {
        let ss = 3;
        let nn = n * ss;
        let hh = fd(nn * 34, 100);
        let mut s = pg::surface_alpha(nn, hh);
        let (col, top): (Rgb, Rgb) = ((92, 102, 136), (176, 196, 224));
        let (fy, fh) = (fd(hh * 46, 100), fd(hh * 18, 100));
        let q = |a: i64, b: i64| fd(a * b, 100);
        draw::polygon(&mut s, col, &[(q(nn, 4), q(hh, 50)), (0, q(hh, 6)), (q(nn, 10), q(hh, 6)), (q(nn, 26), q(hh, 48))], 0);
        draw::rect_r(&mut s, col, (q(nn, 4), fy, q(nn, 80), fh), 0, fd(fh, 2) as i32);
        draw::polygon(&mut s, col, &[(q(nn, 82), fy), (nn, fy + q(fh, 60)), (q(nn, 82), fy + fh)], 0);
        draw::polygon(
            &mut s,
            col,
            &[(q(nn, 42), fy + q(fh, 60)), (q(nn, 60), fy + q(fh, 60)), (q(nn, 38), hh - 1), (q(nn, 30), hh - 1)],
            0,
        );
        draw::line(&mut s, top, (q(nn, 10), fy + fd(ss, 2)), (q(nn, 84), fy + fd(ss, 2)), 1.max(fd(ss, 2)) as i32);
        let img = transform::smoothscale(&s, n, 2.max(fd(hh, ss)));
        if flip {
            transform::flip(&img, true, false)
        } else {
            img
        }
    })
}

/// `_contrail(n, hgt, flip)` - Kondensstreifen n x hgt.
fn contrail(n: i64, hgt: i64, flip: bool) -> Surface {
    cached_s(format!("trail|{n}|{hgt}|{flip}"), || {
        let ss = 2;
        let (nn, hh) = (n * ss, 2.max(hgt * ss));
        let mut s = pg::surface_alpha(nn, hh);
        let segs = 48;
        for k in 0..segs {
            let f = k as f64 / (segs - 1) as f64;
            let th = 1.max((hh as f64 * (1.0 - 0.7 * f)) as i64);
            let a = (8.0 + 190.0 * f * f) as i32;
            let x0 = fd(nn * k, segs);
            s.fill_r((206, 222, 246, a), (x0, fd(hh - th, 2), fd(nn, segs) + 1, th));
        }
        let img = transform::smoothscale(&s, n, 1.max(hgt));
        if flip {
            transform::flip(&img, true, false)
        } else {
            img
        }
    })
}

/// `_satellite(n)` - Satellit (goldener Koerper, zwei Solarfluegel,
/// Schuessel), schraeg gedreht.
fn satellite(n: i64) -> Surface {
    cached_s(format!("sat|{n}"), || {
        let ss = 3;
        let nn = n * ss;
        let mut s = pg::surface_alpha(nn, nn);
        let c = fd(nn, 2);
        let (pw, ph) = (fd(nn * 30, 100), fd(nn * 13, 100));
        let (bw, bh) = (fd(nn * 18, 100), fd(nn * 22, 100));
        let lw = 1.max(fd(ss, 2)) as i32;
        draw::line(&mut s, (150, 154, 166), (c - fd(bw, 2) - fd(pw, 2), c), (c + fd(bw, 2) + fd(pw, 2), c), 1.max(ss) as i32);
        for side in [-1, 1] {
            let x0 = if side > 0 { c + fd(bw, 2) + fd(nn * 4, 100) } else { c - fd(bw, 2) - fd(nn * 4, 100) - pw };
            draw::rect(&mut s, (34, 58, 132), (x0, c - fd(ph, 2), pw, ph), 0);
            for k in 1..4 {
                let xx = x0 + fd(pw * k, 4);
                draw::line(&mut s, (88, 120, 200), (xx, c - fd(ph, 2)), (xx, c + fd(ph, 2)), lw);
            }
            draw::line(&mut s, (88, 120, 200), (x0, c), (x0 + pw, c), lw);
            draw::rect(&mut s, (130, 150, 200), (x0, c - fd(ph, 2), pw, ph), lw);
        }
        draw::rect_r(&mut s, (196, 156, 70), (c - fd(bw, 2), c - fd(bh, 2), bw, bh), 0, ss as i32);
        draw::rect_r(&mut s, (238, 206, 118), (c - fd(bw, 2), c - fd(bh, 2), bw, fd(bh, 2)), 0, ss as i32);
        draw::line(&mut s, (200, 202, 210), (c, c - fd(bh, 2)), (c, c - bh), lw);
        draw::ellipse(&mut s, (226, 228, 236), (c - fd(bw, 2), c - bh - fd(bh, 3), bw, fd(bh, 2)), 0);
        let img = transform::rotate(&s, 24.0);
        let (iw, ih) = (img.w() as i64, img.h() as i64);
        transform::smoothscale(&img, 1.max(fd(iw, ss)), 1.max(fd(ih, ss)))
    })
}

/// `_bolt(w, h, var)` - Blitz (Variante 0..3): gezackter Hauptast mit zwei
/// Seitenaesten, violetter Schein um einen weissen Kern.
fn bolt(w: i64, h: i64, var: i64) -> Surface {
    let var = md(var, 4);
    cached_w(2, w, h, &format!("bolt|{var}"), || {
        let (bw, bh) = (8.max(fd(h * 30, 100)), 16.max(fd(h * 55, 100)));
        let ss = 2;
        let (bbw, bbh) = (bw * ss, bh * ss);
        let mut s = pg::surface_alpha(bbw, bbh);
        let mut pts: Vec<(i64, i64)> = vec![(fd(bbw, 2), 0)];
        let (mut x, mut y, mut k) = (fd(bbw, 2), 0i64, 0i64);
        while y < fd(bbh * 90, 100) {
            let v = hash(21000 + var * 100 + k);
            y += fd(bbh * (5 + v % 6), 100);
            x = fd(bbw, 6).max(fd(bbw * 5, 6).min(x + fd(bbw * ((v >> 8) % 25 - 12), 100)));
            pts.push((x, y));
            k += 1;
        }
        let mut branches: Vec<Vec<(i64, i64)>> = Vec::new();
        for (j, &start) in [2usize, 4].iter().enumerate() {
            if start >= pts.len() {
                continue;
            }
            let (mut bx, mut by) = pts[start];
            let side = if hash(21500 + var * 10 + j as i64) & 1 != 0 { 1 } else { -1 };
            let mut br = vec![(bx, by)];
            for m in 0..4 {
                let v = hash(21600 + var * 40 + j as i64 * 8 + m);
                bx = 0.max((bbw - 1).min(bx + fd(side * bbw * (6 + v % 8), 100)));
                by += fd(bbh * (4 + (v >> 8) % 4), 100);
                br.push((bx, by));
            }
            branches.push(br);
        }
        let g = 2.max(fd(bbh, 70));
        for (col, wid) in [((150, 140, 255, 50), g * 3), ((186, 178, 255, 110), fd(g * 3, 2)), ((255, 255, 255, 255), ss.max(fd(g, 2)))] {
            draw::lines(&mut s, col, false, &pts, wid as i32);
            for br in &branches {
                draw::lines(&mut s, col, false, br, 1.max(fd(wid * 2, 3)) as i32);
            }
        }
        transform::smoothscale(&s, bw, bh)
    })
}

/// `_bolt_glow(w, h)` - Lichtschein in der Wolke um den Blitz.
fn bolt_glow(w: i64, h: i64) -> Surface {
    cached_w(2, w, h, "boltglow", || glow_new(8.max(fd(h * 34, 100)), (160, 160, 236), 120))
}

// ============================================================= Polarlicht

/// `_ribbon_h(h, kind)` - (Strahlenhoehe, Schein darunter).
fn ribbon_h(h: i64, kind: i64) -> (i64, i64) {
    (8.max(fd(h * if kind == 0 { 44 } else { 38 }, 100)), 4.max(fd(h * 12, 100)))
}

/// `_slice_w(w)` - Streifenbreite des gewellt gezeichneten Polarlichts.
fn slice_w(w: i64) -> i64 {
    if w >= 400 {
        8
    } else {
        4
    }
}

/// `_ribbon(w, h, kind)` - Polarlicht-Vorhang (w + Streifen) x (rh + gh):
/// senkrechte Strahlen mit hellem Saum, darunter ein schwacher Schein.
fn ribbon(w: i64, h: i64, kind: i64) -> Surface {
    cached_w(3, w, h, &format!("ribbon|{kind}"), || {
        let (rh, gh) = ribbon_h(h, kind);
        let sl = slice_w(w);
        let mut s = pg::surface_alpha(w + sl, rh + gh);
        let mut glow = pg::surface_alpha(1, gh);
        let gstops: [Stop; 3] = if kind == 0 {
            [(0, [120, 250, 176, 120]), (220, [90, 226, 160, 52]), (1000, [50, 160, 130, 0])]
        } else {
            [(0, [226, 128, 230, 100]), (220, [180, 96, 214, 40]), (1000, [120, 60, 190, 0])]
        };
        grad(&mut glow, 0, gh, 1, &gstops);
        let mut prof = pg::surface_alpha(1, rh);
        if kind == 0 {
            grad(
                &mut prof,
                0,
                rh,
                1,
                &[
                    (0, [170, 80, 210, 0]),
                    (380, [150, 110, 220, 40]),
                    (700, [70, 230, 150, 140]),
                    (880, [140, 255, 186, 225]),
                    (950, [215, 255, 230, 240]),
                    (1000, [130, 252, 180, 150]),
                ],
            );
        } else {
            grad(
                &mut prof,
                0,
                rh,
                1,
                &[
                    (0, [110, 50, 200, 0]),
                    (450, [150, 70, 210, 50]),
                    (800, [210, 100, 222, 135]),
                    (930, [250, 160, 236, 200]),
                    (1000, [232, 134, 232, 120]),
                ],
            );
        }
        let rw = 2.max(fd(w, 200));
        let mut glow = transform::smoothscale(&glow, rw, gh);
        let kf = kind as f64;
        let p = (kf * 1.7 + 0.3, kf * 0.9 + 2.1, kf * 2.3 + 1.1, kf * 1.3 + 0.7);
        let mut x = 0;
        while x < w {
            let v = hash(51000 + kind * 4000 + fd(x, rw));
            let a = TAU * x as f64 / w as f64;
            let mut i = 0.5 + 0.40 * (a * 2.0 + p.0).sin() + 0.18 * (a * 7.0 + p.1).sin() + 0.10 * (a * 17.0 + p.2).sin();
            i = fmax(0.0, fmin(1.0, i)) * (0.74 + (v % 26) as f64 / 100.0);
            let hf = 0.56 + 0.30 * (0.5 + 0.5 * (a * 5.0 + p.3).sin()) + ((v >> 8) % 14) as f64 / 100.0;
            let hh = 2.max((rh as f64 * fmin(1.0, hf)) as i64);
            let jit = (v >> 16) % 1.max(fd(h, 90));
            glow.alpha_set((255.0 * i) as i64);
            s.bl(&glow, (x, rh - jit));
            let mut ray = transform::smoothscale(&prof, rw, hh);
            ray.alpha_set(0.max(255.min((12.0 + 243.0 * i) as i64)));
            s.bl(&ray, (x, rh - hh - jit));
            x += rw;
        }
        let part = s.subsurface(Rect::new(0, 0, sl as i32, (rh + gh) as i32)).copy();
        s.bl(&part, (w, 0));
        s
    })
}

/// `_aurora(s, rect, t)` - zwei wogende Polarlicht-Boegen, in senkrechten
/// Streifen versetzt gezeichnet und wandernd.
fn aurora(s: &mut Surface, rect: Rect, t: f64) {
    let (w, h) = (rect.w as i64, rect.h as i64);
    let sl = slice_w(w);
    for kind in [1i64, 0] {
        let mut rib = ribbon(w, h, kind);
        let (rh, th) = (ribbon_h(h, kind).0, rib.h() as i64);
        let top = fd(rh * 11, 100);
        let (base, arc, peak, a1, l1, o1, a2, l2, o2, drift) = ARCS[kind as usize];
        if kind == 1 {
            rib.alpha_set((195.0 + 60.0 * (t * 0.8 + 2.4).sin()) as i64);
        }
        let by = rect.y as i64 + fd(h * base, 100) - rh + top;
        let dx = t * w as f64 * drift;
        let (aa1, aa2, bb) = ((h * a1) as f64 / 1000.0, (h * a2) as f64 / 1000.0, (h * arc) as f64 / 1000.0);
        let (px, span) = ((w * peak) as f64 / 1000.0, w as f64 * 0.62);
        let mut x = 0;
        while x < w {
            let xf = x as f64;
            let u = (xf - px) / span;
            let yo = bb * u * u - bb
                + aa1 * (TAU * xf / (w as f64 * l1) + t * o1).sin()
                + aa2 * (TAU * xf / (w as f64 * l2) - t * o2 + kf(kind)).sin();
            let sx = wrap(xf + dx, w as f64) as i64;
            s.bl_area(&rib, (rect.x as i64 + x, by + yo as i64), (sx, top, sl, th - top));
            x += sl;
        }
        if kind == 1 {
            rib.alpha_set(255);
        }
    }
}

fn kf(k: i64) -> f64 {
    k as f64
}

// ============================================================= Weltall

fn moon_r1(h: i64) -> i64 {
    6.max(fd(h * 34, 100))
}

/// `_moon_disc(d, seed)` - Mondscheibe d x d: Meere, Krater, Kugelschatten.
fn moon_disc(d: i64, seed: i64) -> Surface {
    let ss = 2;
    let dd = d * ss;
    let (c, r) = (fd(dd, 2), fd(dd, 2) - ss);
    let (cf, rf) = (c as f64, r as f64);
    let mut big = pg::surface_alpha(dd, dd);
    draw::circle(&mut big, (214, 214, 220), (c, c), r, 0);
    for k in 0..7 {
        let v = hash(seed + k);
        let ang = (v % 628) as f64 / 100.0;
        let rr = (r * ((v >> 10) % 55)) as f64 / 100.0;
        let (mx, my) = (cf + ang.cos() * rr, cf + ang.sin() * rr);
        for j in 0..4 {
            let u = hash(seed + 100 + k * 8 + j);
            draw::circle(
                &mut big,
                (170, 172, 188),
                (mx + (r * ((u % 31) - 15)) as f64 / 100.0, my + (r * (((u >> 8) % 31) - 15)) as f64 / 100.0),
                (r * (8 + (u >> 16) % 14)) as f64 / 100.0,
                0,
            );
        }
    }
    for k in 0..64 {
        let v = hash(seed + 500 + k);
        let ang = (v % 628) as f64 / 100.0;
        let rr = rf * 0.92 * (((v >> 10) % 1000) as f64 / 1000.0).sqrt();
        let extra = if k < 3 {
            8
        } else if k < 10 {
            3
        } else {
            0
        };
        let cr = rf * (1.5 + ((v >> 20) % 5) as f64 + extra as f64) / 100.0;
        let (x, y) = (cf + ang.cos() * rr, cf + ang.sin() * rr);
        draw::circle(&mut big, (234, 234, 240), (x - cr * 0.16, y - cr * 0.16), cr, 0);
        draw::circle(&mut big, (158, 160, 176), (x + cr * 0.06, y + cr * 0.06), cr * 0.86, 0);
        draw::circle(&mut big, (196, 198, 208), (x + cr * 0.24, y + cr * 0.24), cr * 0.6, 0);
    }
    let sh = shade(dd, (cf - rf * 0.38) as i64, (cf - rf * 0.38) as i64, (rf * 1.56) as i64, (16, 18, 38), 190, 200, 40);
    big.bl(&sh, (0, 0));
    big.bl_flags(&circle_mask(dd, (c, c), r), (0, 0), BLEND_RGBA_MIN);
    transform::smoothscale(&big, d, d)
}

/// `_moon_master(w, h)` - Mond mit Lichthof in der groessten Groesse.
fn moon_master(w: i64, h: i64) -> Surface {
    cached_w(4, w, h, "moon", || {
        let r = moon_r1(h);
        let m = fd(r * 27, 10);
        let mut s = pg::surface_alpha(m, m);
        s.bl(&glow_new(fd(m, 2), (176, 190, 236), 64), (0, 0));
        let d = moon_disc(2 * r, 61000);
        s.bl(&d, (fd(m, 2) - r, fd(m, 2) - r));
        s
    })
}

/// `_moon_geom(w, h, alt)` - (cx, cy, r) des Monds.
fn moon_geom(w: i64, h: i64, alt: f64) -> (f64, f64, f64) {
    let (t4, tm) = (lg::world_ticks()[4], lg::MOON_TICK);
    let f = fmax(0.0, fmin(1.0, (alt - t4 as f64) / 1.max(tm - t4) as f64));
    let (r0, r1) = (fmax(3.0, h as f64 * 0.018), moon_r1(h) as f64);
    let r = r0 * py::pow(r1 / r0, py::pow(f, 1.4));
    let k = (r - r0) / fmax(1.0, r1 - r0);
    (w as f64 * (0.80 - 0.30 * k), h as f64 * (0.20 - 0.17 * k), r)
}

/// `_moon(s, rect, alt)` - wachsender Mond; nur die zuletzt gebrauchte
/// Groesse bleibt gecacht.
fn moon(s: &mut Surface, rect: Rect, alt: f64) {
    let (w, h) = (rect.w as i64, rect.h as i64);
    let (cx, cy, r) = moon_geom(w, h, alt);
    let rr = moon_r1(h);
    let master = moon_master(w, h);
    let mw = master.w() as i64;
    let size = 4.max(fd((mw as f64 * r / rr as f64) as i64, 2) * 2);
    let mut img = cached_w(4, w, h, "moon_now", || transform::smoothscale(&master, size, size));
    if img.w() as i64 != size {
        img = if size == mw { master.clone() } else { transform::smoothscale(&master, size, size) };
        put_w(4, w, h, "moon_now", img.clone());
    }
    s.bl(&img, (rect.x as i64 + cx as i64 - fd(size, 2), rect.y as i64 + cy as i64 - fd(size, 2)));
}

/// `_earth(w, h)` - Erdkruemmung (w x 56 % h): Nachtseite mit Stadtlichtern,
/// blauer Saum ueber der Kante, Sonnenaufgang rechts.
fn earth(w: i64, h: i64) -> Surface {
    cached_w(4, w, h, "earth", || {
        let eh = 8.max(fd(h * 56, 100));
        let mut s = pg::surface_alpha(w, eh);
        let g = 4.max(fd(h * 12, 100));
        let rr = fd(w.max(h) * 13, 10);
        let (cx, cy) = (fd(w, 2), g + rr);
        let limb = |x: i64| -> i64 { (cy as f64 - (0.max(rr * rr - (x - cx) * (x - cx)) as f64).sqrt()) as i64 };
        let mut band = pg::surface_alpha(2, g);
        grad(&mut band, 0, g, 2, &[(0, [40, 90, 220, 0]), (640, [70, 140, 255, 56]), (1000, [156, 214, 255, 200])]);
        let mut x = 0;
        while x < w {
            s.bl(&band, (x, limb(x) - g));
            x += 2;
        }
        draw::circle(&mut s, (8, 16, 40), (cx, cy), rr, 0);
        let sb = 3.max(fd(h * 9, 100));
        let mut inner = pg::surface_alpha(2, sb);
        grad(&mut inner, 0, sb, 2, &[(0, [60, 124, 214, 255]), (300, [26, 56, 116, 200]), (1000, [8, 16, 40, 0])]);
        let mut x = 0;
        while x < w {
            s.bl(&inner, (x, limb(x)));
            x += 2;
        }
        let dot = if h >= 600 { 2 } else { 1 };
        for k in 0..6.max(fd(w, 40)) {
            let v = hash(66000 + k);
            let x = v % w;
            let y = limb(x) + fd(sb * 6, 10) + (v >> 12) % 1.max(eh - limb(x) - fd(sb * 6, 10));
            for j in 0..(4 + (v >> 24) % 10) {
                let u = hash(66500 + k * 16 + j);
                let px = x + fd((u % 41 - 20) * h, 1000);
                let pyy = y + fd(((u >> 8) % 21 - 10) * h, 1000);
                if pyy > limb(px) + fd(sb, 2) {
                    s.fill_r((255, 206, 120, (120 + (u >> 16) % 120) as i32), (px, pyy, dot, dot));
                }
            }
        }
        let st = 2.max(fd(w, 200));
        let mut pts: Vec<(i64, i64)> = Vec::new();
        let mut x = 0;
        while x < w + 1 {
            pts.push((x, limb(x)));
            x += st;
        }
        draw::lines(&mut s, (176, 224, 255), false, &pts, 1.max(fd(h, 320)) as i32);
        let sx = fd(w * 86, 100);
        let gl = glow_new(4.max(fd(h * 22, 100)), (255, 196, 120), 150);
        s.bl(&gl, (sx - fd(gl.w() as i64, 2), limb(sx) - fd(gl.h() as i64, 2)));
        let sp = sparkle(3.max(fd(h, 60)), (255, 236, 200));
        s.bl(&sp, (sx - fd(sp.w() as i64, 2), limb(sx) - fd(sp.h() as i64, 2)));
        s
    })
}

/// `_earth_draw(s, rect, alt)` - Erde zu Beginn des Weltalls.
fn earth_draw(s: &mut Surface, rect: Rect, alt: f64) {
    let (w, h) = (rect.w as i64, rect.h as i64);
    let t4 = lg::world_ticks()[4];
    let drop = fmax(-h as f64 * 0.08, (rise(alt, h) - rise(t4 as f64, h)) * EARTH_P);
    let y = rect.y as i64 + fd(h * 58, 100) - 4.max(fd(h * 12, 100)) + drop as i64;
    if y < rect.bottom() as i64 {
        s.bl(&earth(w, h), (rect.x as i64, y));
    }
}

/// `_planet(w, h, kind)` - ferner Planet: 0 Ringplanet, 1 roter Planet,
/// 2 Eisriese; Licht von oben links (3-fach ueberabgetastet).
fn planet(w: i64, h: i64, kind: i64) -> Surface {
    cached_w(4, w, h, &format!("planet|{kind}"), || {
        let r = 3.max(fd(h * PLANETS[kind as usize].2, 1000));
        let ss = 3;
        let rr = r * ss;
        let size = if kind == 0 { rr * 5 } else { rr * 2 + 4 * ss };
        let c = fd(size, 2);
        let mut dsk = pg::surface_alpha(size, size);
        if kind == 0 {
            let bands: [Rgb; 5] = [(228, 198, 144), (206, 166, 114), (238, 216, 170), (192, 150, 102), (224, 192, 138)];
            for k in 0..11 {
                dsk.fill_r(bands[(k % 5) as usize], (0, c - rr + fd(2 * rr * k, 11), size, fd(2 * rr, 11) + 1));
            }
        } else if kind == 1 {
            dsk.fill_c((198, 102, 70));
            for k in 0..6 {
                let v = hash(67000 + k);
                draw::circle(
                    &mut dsk,
                    (154, 72, 52),
                    (c + fd(rr * ((v % 120) - 60), 100), c + fd(rr * (((v >> 8) % 120) - 60), 100)),
                    fd(rr * (12 + (v >> 16) % 20), 100),
                    0,
                );
            }
            draw::ellipse(&mut dsk, (244, 236, 228), (c - fd(rr * 45, 100), c - rr - fd(rr, 10), fd(rr * 90, 100), fd(rr * 36, 100)), 0);
        } else {
            dsk.fill_c((128, 200, 220));
            for k in 0..5 {
                dsk.fill_r((112, 184, 210), (0, c - rr + fd(2 * rr * (2 * k + 1), 10), size, 1.max(fd(rr, 9))));
            }
        }
        dsk.bl(&shade(size, c - fd(rr * 4, 10), c - fd(rr * 4, 10), fd(rr * 16, 10), (8, 8, 24), 220, 0, 40), (0, 0));
        dsk.bl_flags(&circle_mask(size, (c, c), rr), (0, 0), BLEND_RGBA_MIN);
        let mut s = pg::surface_alpha(size, size);
        if kind == 0 {
            let mut ring = pg::surface_alpha(size, size);
            draw::ellipse(
                &mut ring,
                (214, 194, 150, 210),
                (c - fd(rr * 23, 10), c - fd(rr * 55, 100), fd(rr * 46, 10), fd(rr * 110, 100)),
                ss.max(fd(rr, 5)) as i32,
            );
            draw::ellipse(
                &mut ring,
                (176, 156, 118, 160),
                (c - fd(rr * 18, 10), c - fd(rr * 42, 100), fd(rr * 36, 10), fd(rr * 84, 100)),
                ss.max(fd(rr, 9)) as i32,
            );
            let mut back = ring.copy();
            back.fill_r((0, 0, 0, 0), (0, c, size, size - c));
            ring.fill_r((0, 0, 0, 0), (0, 0, size, c));
            s.bl(&back, (0, 0));
            s.bl(&dsk, (0, 0));
            s.bl(&ring, (0, 0));
            s = transform::rotate(&s, 16.0);
        } else {
            s.bl(&dsk, (0, 0));
        }
        let (sw, sh) = (s.w() as i64, s.h() as i64);
        transform::smoothscale(&s, 1.max(fd(sw, ss)), 1.max(fd(sh, ss)))
    })
}

/// `_planets(s, rect, rise)` - ferne Planeten ziehen langsam durchs Bild.
fn planets(s: &mut Surface, rect: Rect, rise_v: f64) {
    let (w, h) = (rect.w as i64, rect.h as i64);
    let t4 = lg::world_ticks()[4];
    for (kind, &(start, fx, fr, _k)) in PLANETS.iter().enumerate() {
        let r = 3.max(fd(h * fr, 1000));
        let cy = (rect.y as i64 - r * 2) as f64 + (rise_v - rise((t4 + start) as f64, h)) * PLANET_P;
        if cy < (rect.y as i64 - r * 3) as f64 || cy > (rect.bottom() as i64 + r * 3) as f64 {
            continue;
        }
        let img = planet(w, h, kind as i64);
        s.bl(&img, (rect.x as i64 + fd(w * fx, 1000) - fd(img.w() as i64, 2), cy as i64 - fd(img.h() as i64, 2)));
    }
}

// ============================================================= belebte Details

/// `_slots(off, h, gap)` - Plaetze i >= 0 einer Hoehenreihe, die gerade
/// (mit Rand) sichtbar sein koennen.
fn slots(off: f64, h: i64, gap: i64) -> std::ops::Range<i64> {
    let top = fd(h * TOP, 100) as f64 + off;
    let g = gap as f64;
    let lo = 0.max(floordiv(top - (2 * h) as f64, g) as i64);
    let hi = floordiv(top + h as f64, g) as i64 + 2;
    lo..hi
}

/// `_cross(rect, v, d, sp, t, m)` - x eines Dings, das waagerecht durchs
/// Bild zieht (d = +1 nach rechts, -1 nach links).
fn cross(rect: Rect, v: i64, d: i64, sp: f64, t: f64, m: i64) -> f64 {
    let w = rect.w as i64;
    let pp = w + fd(w, 2) + 2 * m;
    let u = wrap(((v >> 4) % pp) as f64 + sp * t, pp as f64);
    if d > 0 {
        (rect.x as i64 - m) as f64 + u
    } else {
        (rect.right() as i64 + m) as f64 - u
    }
}

/// Zeile `y` einer Hoehenreihe: `rect.y + int(h * TOP // 100 + off - i * gap
/// - (v >> 8) % max(1, gap // 3))`.
fn row_y(rect: Rect, h: i64, off: f64, i: i64, gap: i64, v: i64) -> i64 {
    rect.y as i64 + ((fd(h * TOP, 100) as f64 + off - (i * gap) as f64) - ((v >> 8) % 1.max(fd(gap, 3))) as f64) as i64
}

/// `_flocks(s, rect, world, t, rise)` - Vogelschwaerme.
fn flocks(s: &mut Surface, rect: Rect, world: i64, t: f64, rise_v: f64) {
    let h = rect.h as i64;
    let n = 5.max(fd(h * if world == 0 { 18 } else { 24 }, 1000));
    let gap = fd(h * if world == 0 { 50 } else { 56 }, 100);
    let off = rise_v * PARALLAX[1];
    let col: Rgb = (46, 52, 74);
    let margin = n * 8;
    for i in slots(off, h, gap) {
        let v = hash(9000 + world * 500 + i);
        if v % 100 >= 80 {
            continue;
        }
        let y = row_y(rect, h, off, i, gap, v);
        if y < rect.y as i64 - margin || y > rect.bottom() as i64 + margin {
            continue;
        }
        let d = if (v >> 7) & 1 != 0 { 1 } else { -1 };
        let x = cross(rect, v, d, (h * (34 + (v >> 9) % 30)) as f64 / 1000.0, t, margin);
        let nb = 3 + (v >> 20) % if world == 0 { 3 } else { 5 };
        let rate = if world == 0 { 7.0 } else { 4.2 };
        for k in 0..nb {
            let (bx, by) = if world == 0 {
                let u = hash(9400 + i * 8 + k);
                (x - fd(d * k * n * 13, 10) as f64 + fd((u % 9 - 4) * n, 6) as f64, y + fd(((u >> 8) % 13 - 6) * n, 6))
            } else {
                let (rank, side) = (fd(k + 1, 2), if k % 2 != 0 { 1 } else { -1 });
                (x - fd(d * rank * n * 14, 10) as f64, y + fd(side * rank * n * 6, 10))
            };
            let fr = md((t * rate + k as f64 * 1.37 + i as f64) as i64, 4);
            s.bl(&bird(n, fr, col, d < 0), (bx as i64 - fd(n, 2), by - fd(n, 2)));
        }
    }
}

/// `_lights(s, x, y, img_w, img_h, d, t, ph, r, spots)` - Blinklichter
/// eines Flugzeugs.
#[allow(clippy::too_many_arguments)]
fn lights(s: &mut Surface, x: i64, y: i64, img_w: i64, img_h: i64, d: i64, t: f64, ph: f64, r: i64, spots: [(i64, i64); 3]) {
    let at = |p: (i64, i64)| -> (i64, i64) {
        let px = if d > 0 { p.0 } else { 1000 - p.0 };
        (x + fd(img_w * px, 1000), y + fd(img_h * p.1, 1000))
    };
    let nav: Rgb = if d > 0 { (255, 70, 70) } else { (80, 255, 120) };
    let q = wrap(t * 1.1 + ph, 1.0);
    let st = wrap(t * 0.8 + ph * 0.7, 1.0);
    let mut items: Vec<((i64, i64), Rgb, i64)> = vec![(spots[2], nav, r)];
    if q < 0.16 {
        items.push((spots[0], (255, 60, 60), r + 1));
    }
    if st < 0.05 || (0.11 < st && st < 0.16) {
        items.push((spots[1], (255, 255, 255), r + 2));
    }
    for (p, col, rr) in items {
        let (lx, ly) = at(p);
        s.bl(&light(rr, col), (lx - 2.max(rr), ly - 2.max(rr)));
    }
}

/// `_planes(s, rect, t, rise)` - kleine Verkehrsflugzeuge.
fn planes(s: &mut Surface, rect: Rect, t: f64, rise_v: f64) {
    let (w, h) = (rect.w as i64, rect.h as i64);
    let n = 14.max(fd(h * 9, 100));
    let gap = fd(h * 60, 100);
    let off = rise_v * PARALLAX[1];
    let r = 2.max(fd(n, 16));
    for i in slots(off, h, gap) {
        let v = hash(12000 + i);
        if v % 100 >= 72 {
            continue;
        }
        let y = row_y(rect, h, off, i, gap, v);
        if y < rect.y as i64 - n || y > rect.bottom() as i64 + n {
            continue;
        }
        let d = if (v >> 6) & 1 != 0 { 1 } else { -1 };
        let x = cross(rect, v, d, (w * (55 + (v >> 16) % 35)) as f64 / 1000.0, t, n);
        let img = plane(n, d < 0);
        let ih = img.h() as i64;
        let (ix, iy) = (x as i64 - fd(n, 2), y - fd(ih, 2));
        s.bl(&img, (ix, iy));
        lights(s, ix, iy, n, ih, d, t, (v >> 24) as f64 / 256.0, r, [(520, 420), (20, 60), (370, 980)]);
    }
}

/// `_jets(s, rect, t, rise)` - Jets mit Kondensstreifen unter dem Polarlicht.
fn jets(s: &mut Surface, rect: Rect, t: f64, rise_v: f64) {
    let (w, h) = (rect.w as i64, rect.h as i64);
    let n = 12.max(fd(h * 6, 100));
    let gap = fd(h * 65, 100);
    let off = rise_v * PARALLAX[1];
    let cl = 16.max(fd(w * 45, 100));
    let ch = 2.max(fd(h * 12, 1000));
    let r = 2.max(fd(n, 14));
    for i in slots(off, h, gap) {
        let v = hash(14000 + i);
        if v % 100 >= 75 {
            continue;
        }
        let y = row_y(rect, h, off, i, gap, v);
        if y < rect.y as i64 - n || y > rect.bottom() as i64 + n {
            continue;
        }
        let d = if (v >> 6) & 1 != 0 { 1 } else { -1 };
        let x = cross(rect, v, d, (w * (100 + (v >> 16) % 50)) as f64 / 1000.0, t, n + cl);
        let img = jet(n, d < 0);
        let ih = img.h() as i64;
        let (ix, iy) = (x as i64 - fd(n, 2), y - fd(ih, 2));
        let tx = if d > 0 { ix - cl + fd(n, 10) } else { ix + n - fd(n, 10) };
        s.bl(&contrail(cl, ch, d < 0), (tx, y + fd(ih, 10) - fd(ch, 2)));
        s.bl(&img, (ix, iy));
        lights(s, ix, iy, n, ih, d, t, (v >> 24) as f64 / 256.0, r, [(540, 440), (30, 80), (340, 980)]);
    }
}

/// `_flash(t)` - (Staerke 0..1, x Promille, y Promille, Variante) des Blitzes
/// zur Zeit t oder None.
fn flash(t: f64) -> Option<(f64, i64, i64, i64)> {
    let tt = fmax(0.0, t);
    let k = floordiv(tt, FLASH) as i64;
    let v = hash(20011 + k % 1000000);
    if v % 100 >= 60 {
        return None;
    }
    let d = tt - k as f64 * FLASH - ((v >> 8) % 1000) as f64 * (FLASH - 0.5) / 1000.0;
    if d < 0.0 || d >= 0.45 {
        return None;
    }
    let f = if d < 0.05 {
        1.0
    } else if d < 0.09 {
        0.3
    } else if d < 0.16 {
        0.9
    } else {
        0.9 * (0.45 - d) / 0.29
    };
    Some((f, 150 + (v >> 12) % 700, 40 + (v >> 22) % 260, (v >> 4) % 4))
}

/// `_lightning(s, rect, t, front)` - Blitz: hinten Schein + Blitzast in den
/// Wolken, vorn heller Aufschein ueber die ganze Buehne (additiv).
fn lightning(s: &mut Surface, rect: Rect, t: f64, front: bool) {
    let Some((f, fx, fy, var)) = flash(t) else {
        return;
    };
    let (w, h) = (rect.w as i64, rect.h as i64);
    if front {
        let g = (34.0 * f) as i32;
        if g > 0 {
            s.fill_rf((g, g, g + g / 3), rect, BLEND_RGB_ADD);
        }
        return;
    }
    let (x, y) = (rect.x as i64 + fd(w * fx, 1000), rect.y as i64 + fd(h * fy, 1000));
    let gl = bolt_glow(w, h);
    blit_a(s, &gl, (x - fd(gl.w() as i64, 2), y - fd(gl.h() as i64, 2)), (255.0 * f) as i64);
    if f > 0.5 {
        let b = bolt(w, h, var);
        blit_a(s, &b, (x - fd(b.w() as i64, 2), y), 255.min((300.0 * f) as i64));
    }
}

/// `_rain(s, rect, t, rise)` - schraeger Regen.
fn rain(s: &mut Surface, rect: Rect, t: f64, rise_v: f64) {
    let (w, h) = (rect.w as i64, rect.h as i64);
    let n = fd(w * h, 4200);
    let ln = 5.max(fd(h * 5, 100));
    let thick = if h >= 600 { 2 } else { 1 };
    for i in 0..n {
        let v = hash(30011 + i);
        let near = v % 3 == 0;
        let sp = (h * (130 + (v >> 8) % 70)) as f64 / 100.0 * if near { 1.4 } else { 1.0 };
        let y = wrap((((v >> 4) % 1000) * (h + ln)) as f64 / 1000.0 + sp * t + rise_v * 0.6, (h + ln) as f64) - ln as f64;
        let ll = fd(ln * if near { 13 } else { 8 }, 10);
        let x = ((v >> 14) % (w + fd(w, 3)) - fd(w, 8)) as f64 - y * 0.22;
        let col: Rgb = if near { (136, 146, 176) } else { (92, 100, 128) };
        let (x0, y0) = (rect.x as i64 + x as i64, rect.y as i64 + y as i64);
        draw::line(s, col, (x0, y0), (x0 - fd(ll * 22, 100), y0 + ll), if near { thick } else { 1 });
    }
}

/// `_twinkle(s, rect, world, t, rise)` - funkelnde Sterne.
fn twinkle(s: &mut Surface, rect: Rect, world: i64, t: f64, rise_v: f64) {
    let (w, h) = (rect.w as i64, rect.h as i64);
    let n = 6.max(fd(w * h, 24000));
    let off = rise_v * PARALLAX[1];
    let r = 2.max(fd(h, 150));
    for i in 0..n {
        let v = hash(40000 + world * 1000 + i);
        let u = hash(40500 + world * 1000 + i);
        let x = v % w;
        let y = if world == 5 {
            let y = u % 1.max(fd(h * 50, 100));
            if (x - fd(w * 76, 100)).abs() < fd(h * 16, 100) && (y - fd(h * 22, 100)).abs() < fd(h * 16, 100) {
                continue;
            }
            y
        } else {
            md(((u % h) as f64 + off) as i64, h)
        };
        let b = 0.5 + 0.5 * (t * (1.4 + ((v >> 20) % 30) as f64 / 10.0) + ((v >> 8) % 628) as f64 / 100.0).sin();
        let al = (255.0 * b * b) as i64;
        if al < 12 {
            continue;
        }
        let spr = sparkle(r + (u >> 24) % 2, STAR_COLS[((u >> 12) % 4) as usize]);
        let (sw, sh) = (spr.w() as i64, spr.h() as i64);
        blit_a(s, &spr, (rect.x as i64 + x - fd(sw, 2), rect.y as i64 + y - fd(sh, 2)), al);
    }
}

/// `_shooting(s, rect, t)` - Sternschnuppe: in jedem SHOOT-Fenster mit 55 %
/// eine, 0,75 s lang.
fn shooting(s: &mut Surface, rect: Rect, t: f64) {
    let tt = fmax(0.0, t);
    let k = floordiv(tt, SHOOT) as i64;
    let v = hash(41017 + k % 1000000);
    if v % 100 >= 55 {
        return;
    }
    let dur = 0.75;
    let d = tt - k as f64 * SHOOT - ((v >> 8) % 1000) as f64 * (SHOOT - dur) / 1000.0;
    if !(0.0 <= d && d < dur) {
        return;
    }
    let p = d / dur;
    let (w, h) = (rect.w as i64, rect.h as i64);
    let x0 = rect.x as i64 + fd(w * (100 + (v >> 12) % 800), 1000);
    let y0 = rect.y as i64 + fd(h * (40 + (v >> 20) % 350), 1000);
    let dx = if (v >> 3) & 1 != 0 { 1.0 } else { -1.0 };
    let ang = py::radians((22 + (v >> 28) % 16) as f64);
    let (ux, uy) = (dx * ang.cos(), ang.sin());
    let hf = h as f64;
    let (hx, hy) = (x0 as f64 + ux * hf * 0.45 * p, y0 as f64 + uy * hf * 0.45 * p);
    let tail = hf * 0.22 * fmin(1.0, p * 4.0);
    let env = (PI * p).sin();
    let bg: C4 = o(4, 6, 20);
    let wid = if h >= 400 { 2 } else { 1 };
    for j in 0..5 {
        let c = lerp(bg, o(236, 242, 255), (env * (1000 - j * 190) as f64) as i64, 1000);
        let (f0, f1) = (j as f64 / 5.0, (j + 1) as f64 / 5.0);
        draw::line(
            s,
            c3(c),
            ((hx - ux * tail * f0) as i64, (hy - uy * tail * f0) as i64),
            ((hx - ux * tail * f1) as i64, (hy - uy * tail * f1) as i64),
            if j < 3 { wid } else { 1 },
        );
    }
    let hd = light(3.max(fd(h, 110)), (220, 230, 255));
    blit_a(s, &hd, (hx as i64 - fd(hd.w() as i64, 2), hy as i64 - fd(hd.h() as i64, 2)), (255.0 * env) as i64);
}

/// `_satellites(s, rect, t, rise)` - Satelliten mit rotem Blinklicht.
fn satellites(s: &mut Surface, rect: Rect, t: f64, rise_v: f64) {
    let (w, h) = (rect.w as i64, rect.h as i64);
    let n = 10.max(fd(h * 6, 100));
    let gap = fd(h * 70, 100);
    let off = rise_v * SAT_P;
    for i in slots(off, h, gap) {
        let v = hash(43013 + i);
        if v % 100 >= 80 {
            continue;
        }
        let d = if (v >> 6) & 1 != 0 { 1 } else { -1 };
        let x = cross(rect, v, d, (w * (22 + (v >> 16) % 20)) as f64 / 1000.0, t, 2 * n) as i64;
        let slope = ((v >> 24) % 40 - 20) as f64 / 100.0;
        let y = rect.y as i64
            + ((fd(h * TOP, 100) as f64 + off - (i * gap) as f64) - ((v >> 8) % 1.max(fd(gap, 3))) as f64
                + (x - rect.centerx() as i64) as f64 * slope) as i64;
        if y < rect.y as i64 - n || y > rect.bottom() as i64 + n {
            continue;
        }
        let img = satellite(n);
        s.bl(&img, (x - fd(img.w() as i64, 2), y - fd(img.h() as i64, 2)));
        if wrap(t + (v >> 20) as f64 / 64.0, 1.3) < 0.14 {
            let lt = light(2.max(fd(n, 7)), (255, 70, 70));
            s.bl(&lt, (x - fd(lt.w() as i64, 2), y - fd(lt.h() as i64, 2)));
        }
    }
}

/// `_details(s, rect, world, phase, alt, t, rise)` - Belebtes je Welt;
/// phase 0 = hinter der mittleren Ebene, 1 = hinter der nahen, 2 = vorn.
#[allow(clippy::too_many_arguments)]
fn details(s: &mut Surface, rect: Rect, world: i64, phase: i64, alt: f64, t: f64, rise_v: f64) {
    if world == 0 || world == 1 {
        if phase == 1 {
            flocks(s, rect, world, t, rise_v);
        }
    } else if world == 2 {
        if phase == 0 {
            lightning(s, rect, t, false);
        } else if phase == 1 {
            planes(s, rect, t, rise_v);
        } else {
            rain(s, rect, t, rise_v);
            lightning(s, rect, t, true);
        }
    } else if world == 3 {
        if phase == 0 {
            aurora(s, rect, t);
        } else if phase == 1 {
            jets(s, rect, t, rise_v);
        } else {
            twinkle(s, rect, 3, t, rise_v);
        }
    } else if world == 4 && phase == 2 {
        twinkle(s, rect, 4, t, rise_v);
        shooting(s, rect, t);
        planets(s, rect, rise_v);
        moon(s, rect, alt);
        earth_draw(s, rect, alt);
        satellites(s, rect, t, rise_v);
    }
}

// ============================================================= Mond-Landung

/// `_marble(r)` - die Erde vom Mond aus (3-fach ueberabgetastet).
fn marble(r: i64) -> Surface {
    let ss = 3;
    let rr = r * ss;
    let rf = rr as f64;
    let size = rr * 2 + rr;
    let c = fd(size, 2);
    let cf = c as f64;
    let mut s = pg::surface_alpha(size, size);
    s.bl(&glow_new(c, (110, 170, 255), 80), (0, 0));
    let mut dsk = pg::surface_alpha(size, size);
    draw::circle(&mut dsk, (34, 92, 198), (c, c), rr, 0);
    for k in 0..5 {
        let v = hash(64000 + k);
        let (ang, rad) = ((v % 628) as f64 / 100.0, (rr * (10 + (v >> 10) % 60)) as f64 / 100.0);
        let (mx, my) = (cf + ang.cos() * rad, cf + ang.sin() * rad);
        for j in 0..10 {
            let u = hash(64100 + k * 16 + j);
            let col: Rgb = if (u >> 24) % 4 == 0 { (172, 152, 100) } else { (86, 152, 80) };
            draw::circle(
                &mut dsk,
                col,
                (mx + (rr * ((u % 51) - 25)) as f64 / 100.0, my + (rr * (((u >> 8) % 41) - 20)) as f64 / 100.0),
                (rr * (4 + (u >> 16) % 9)) as f64 / 100.0,
                0,
            );
        }
    }
    for k in 0..22 {
        let v = hash(64500 + k);
        let (ang, rad) = ((v % 628) as f64 / 100.0, (rr * ((v >> 10) % 92)) as f64 / 100.0);
        let (cw, ch) = ((rr * (14 + (v >> 18) % 24)) as f64 / 100.0, (rr * (3 + (v >> 24) % 4)) as f64 / 100.0);
        draw::ellipse(
            &mut dsk,
            (246, 248, 255, 200),
            (cf + ang.cos() * rad - cw / 2.0, cf + ang.sin() * rad - ch / 2.0, cw, ch),
            0,
        );
    }
    dsk.bl(&shade(size, (cf - rf * 0.60) as i64, (cf - rf * 0.25) as i64, (rf * 1.30) as i64, (4, 8, 26), 248, 560, 40), (0, 0));
    dsk.bl_flags(&circle_mask(size, (c, c), rr), (0, 0), BLEND_RGBA_MIN);
    s.bl(&dsk, (0, 0));
    draw::arc(&mut s, (170, 214, 255, 190), (c - rr, c - rr, 2 * rr, 2 * rr), 1.2, 4.3, 1.max(ss) as i32);
    transform::smoothscale(&s, fd(size, ss), fd(size, ss))
}

/// `_crater(s, x, y, cw, ch)` - Krater in Aufsicht schraeg.
fn crater(s: &mut Surface, x: i64, y: i64, cw: i64, ch: i64) {
    draw::ellipse(s, (172, 172, 180), (x - fd(cw, 2), y - fd(ch, 2) - 1.max(fd(ch, 8)), cw, ch), 0);
    draw::ellipse(s, (84, 84, 94), (x - fd(cw * 42, 100), y - fd(ch * 36, 100), fd(cw * 84, 100), fd(ch * 76, 100)), 0);
    draw::ellipse(s, (124, 124, 134), (x - fd(cw * 4, 100), y - fd(ch * 20, 100), fd(cw * 44, 100), fd(ch * 52, 100)), 0);
}

/// `moon_scene(w, h)` - Mondlandung: schwarzer Himmel mit Sternen, die Erde
/// oben rechts, ferne Berge, graue Kraterlandschaft mit Landeplatz.
pub fn moon_scene(w: i32, h: i32) -> Surface {
    moon_scene_of(1.max(w as i64), 1.max(h as i64))
}

fn moon_scene_of(w: i64, h: i64) -> Surface {
    cached_w(5, w, h, "scene", || {
        let mut s = pg::surface(w, h);
        grad(&mut s, 0, h, w, SKY[5]);
        let bg = SKY[5][0].1;
        for i in 0..1.max(fd(w * h, 1100)) {
            let v = hash(81000 + 2 * i);
            let u = hash(81001 + 2 * i);
            let sc = STAR_COLS[((v >> 20) % 4) as usize];
            let col = lerp(bg, o(sc.0 as i64, sc.1 as i64, sc.2 as i64), 40 + (u >> 12) % 216, 255);
            let big = 1 + i64::from((u >> 24) % 9 == 0);
            s.fill_r(c3(col), (v % w, u % 1.max(fd(h * 62, 100)), big, big));
        }
        let re = 4.max(fd(h * 10, 100));
        let e = marble(re);
        s.bl(&e, (fd(w * 76, 100) - fd(e.w() as i64, 2), fd(h * 22, 100) - fd(e.h() as i64, 2)));
        let yh = fd(h * 60, 100);
        // ferne, runde Mondberge in zwei Reihen (Licht von links: helle Kante)
        let step = 3.max(fd(w, 160));
        let hills: [MoonHill; 2] = [
            (70, 10, (96, 96, 108), (140, 140, 152), (0.4, 2.0, 1.1)),
            (44, 22, (118, 118, 130), (170, 170, 180), (2.6, 0.8, 0.2)),
        ];
        for (amp, base, col, rim, sd) in hills {
            let mut pts: Vec<(i64, i64)> = vec![(0, yh + fd(h, 20))];
            let mut x = 0;
            while x < w + step {
                let q = 0.5 + 0.5 * wave(x, w, sd);
                pts.push((x, yh + fd(h * base, 1000) - ((h * amp) as f64 / 1000.0 * q * q) as i64));
                x += step;
            }
            pts.push((w + step, yh + fd(h, 20)));
            draw::polygon(&mut s, col, &pts, 0);
            draw::lines(&mut s, rim, false, &pts[1..pts.len() - 1], 1.max(fd(h, 320)) as i32);
        }
        // Boden: Verlauf, leicht gewoelbt
        let rs = w * 2;
        let gh = h - yh + fd(h, 20);
        let mut surf = pg::surface_alpha(w, gh);
        grad(&mut surf, 0, gh, w, &[(0, [160, 160, 168, 255]), (400, [134, 134, 144, 255]), (1000, [96, 96, 106, 255])]);
        let mut mk = pg::surface_alpha(w, gh);
        draw::circle(&mut mk, (255, 255, 255, 255), (fd(w, 2), rs + fd(h, 20)), rs, 0);
        surf.bl_flags(&mk, (0, 0), BLEND_RGBA_MIN);
        s.bl(&surf, (0, yh - fd(h, 20)));
        // dunklere Meere (flach in der Perspektive)
        for i in 0..5 {
            let v = hash(83500 + i);
            let z = 0.15 + 0.7 * (v % 1000) as f64 / 1000.0;
            let (mw, mh) = ((w as f64 * (0.25 + 0.3 * z)) as i64, 2.max((h as f64 * 0.04 * (0.5 + z)) as i64));
            let mx = (v >> 10) % w;
            let my = yh + ((h - yh) as f64 * z * z) as i64 + fd(h, 40);
            draw::ellipse(&mut s, (124, 124, 134), (mx - fd(mw, 2), my - fd(mh, 2), mw, mh), 0);
        }
        // Staub, Krater (hinten klein und flach, vorn gross), Steine
        let dust: [Rgb; 3] = [(150, 150, 158), (118, 118, 128), (170, 170, 178)];
        for i in 0..fd(w * (h - yh), 260) {
            let v = hash(83000 + i);
            let (x, y) = (v % w, yh + (v >> 12) % 1.max(h - yh));
            let dv = (v >> 24) % 3;
            s.fill_r(dust[dv as usize], (x, y, 1, 1));
        }
        for i in 0..26 {
            let v = hash(84000 + i);
            let z = (v % 1000) as f64 / 1000.0;
            let y = yh + ((h - yh) as f64 * (0.06 + 0.9 * z * z)) as i64;
            let x = (v >> 10) & 0xFFFF;
            let x = fd(x * w, 65536);
            let cw = (w as f64 * (0.03 + 0.13 * z) * (0.6 + ((v >> 26) % 8) as f64 / 10.0)) as i64;
            let ch = 2.max((cw as f64 * (0.16 + 0.22 * z)) as i64);
            if (x - fd(w, 2)).abs() < fd(w * 16, 100) + fd(cw, 2) && y > fd(h * 70, 100) {
                continue;
            }
            crater(&mut s, x, y, cw, ch);
        }
        for i in 0..40 {
            let v = hash(85000 + i);
            let z = (v % 1000) as f64 / 1000.0;
            let y = yh + ((h - yh) as f64 * (0.05 + 0.92 * z)) as i64;
            let x = (v >> 10) % w;
            let rr = 1.max((h as f64 * (0.002 + 0.012 * z)) as i64);
            if (x - fd(w, 2)).abs() < fd(w * 12, 100) && y > fd(h * 72, 100) {
                continue;
            }
            draw::ellipse(&mut s, (66, 66, 76), (x - rr, y - fd(rr, 2), rr * 2, rr), 0);
            draw::ellipse(&mut s, (176, 176, 184), (x - fd(rr * 7, 10), y - fd(rr * 8, 10), fd(rr * 12, 10), fd(rr * 7, 10)), 0);
        }
        s
    })
}

// ============================================================= Zeichnen

/// `_prebuild(world, w, h)` - eine noch fehlende Flaeche von world bauen
/// (hoechstens eine je Bild; Reihenfolge wie `_assets`).
fn prebuild(world: i64, w: i64, h: i64) {
    if !has_w(world, w, h, "sky") {
        sky_of(world, w, h);
        return;
    }
    for d in 0..3 {
        if !has_w(world, w, h, &format!("layer|{d}")) {
            layer_of(world, d, w, h);
            return;
        }
    }
    if world == 0 {
        if !has_w(0, w, h, "ground") {
            ground_of(w, h);
        }
    } else if world == 2 {
        if !has_w(2, w, h, "boltglow") {
            bolt_glow(w, h);
            return;
        }
        for v in 0..4 {
            if !has_w(2, w, h, &format!("bolt|{v}")) {
                bolt(w, h, v);
                return;
            }
        }
    } else if world == 3 {
        for k in 0..2 {
            if !has_w(3, w, h, &format!("ribbon|{k}")) {
                ribbon(w, h, k);
                return;
            }
        }
    } else if world == 4 {
        if !has_w(4, w, h, "earth") {
            earth(w, h);
            return;
        }
        if !has_w(4, w, h, "moon") {
            moon_master(w, h);
            return;
        }
        for k in 0..3 {
            if !has_w(4, w, h, &format!("planet|{k}")) {
                planet(w, h, k);
                return;
            }
        }
    }
}

/// `_keep(world, w, h)` - nur world und world + 1 (in dieser Groesse)
/// behalten.
fn keep(world: i64, w: i64, h: i64) {
    WC.with(|c| {
        let mut c = c.borrow_mut();
        let gs = c.groups.clone();
        for g in gs {
            if g != (world, w, h) && g != (world + 1, w, h) {
                c.groups.retain(|x| *x != g);
                c.world.remove(&g);
            }
        }
    });
}

/// `_buffer(w, h)` - buehnengrosse Zwischenflaeche fuer die Ueberblendung.
fn buffer(w: i64, h: i64) -> Surface {
    WC.with(|c| {
        let mut c = c.borrow_mut();
        let ok = c.buf.as_ref().is_some_and(|b| b.w() as i64 == w && b.h() as i64 == h);
        if !ok {
            c.buf = Some(pg::surface(w, h));
        }
        c.buf.clone().unwrap_or_else(|| pg::surface(w, h))
    })
}

/// `_scene(s, rect, world, alt, t, rise, with_ground)` - eine Welt deckend.
#[allow(clippy::too_many_arguments)]
fn scene(s: &mut Surface, rect: Rect, world: i64, alt: f64, t: f64, rise_v: f64, with_ground: bool) {
    let (w, h) = (rect.w as i64, rect.h as i64);
    s.bl(&sky_of(world, w, h), (rect.x, rect.y));
    for depth in 0..3 {
        tile(s, rect, &layer_of(world, depth, w, h), rise_v * PARALLAX[depth as usize]);
        if depth == 2 && with_ground {
            let gy = rect.bottom() as i64 - gh_of(h) + rise_v as i64;
            if gy < rect.bottom() as i64 {
                s.bl(&ground_of(w, h), (rect.x as i64, gy));
            }
        }
        details(s, rect, world, depth, alt, t, rise_v);
    }
}

/// `draw(s, rect, world, share, alt, t)` - ganze Buehne in rect (auf rect
/// beschnitten): die Welt mit Himmel, Ebenen (Parallaxe), Details und Boden;
/// bei share > 0 wird die naechste Welt mit share/255 darueber geblendet.
/// alt = Hoehe als gebrochener Takt, t = Animationszeit in Sekunden.
pub fn draw(s: &mut Surface, rect: impl IntoRect, world: i32, share: i32, alt: f64, t: f64) {
    let rect = rect.rect();
    let (w, h) = (rect.w as i64, rect.h as i64);
    if w < 4 || h < 4 {
        return;
    }
    let world = world_of(world as i64);
    keep(world, w, h);
    let clip = s.get_clip();
    s.set_clip(Some(rect.clip(clip)));
    if world >= 5 {
        s.bl(&moon_scene_of(w, h), (rect.x, rect.y));
        twinkle(s, rect, 5, t, 0.0);
    } else {
        let share = if world < 4 { 0.max(255.min(share)) } else { 0 };
        let rise_v = rise(alt, h);
        scene(s, rect, world, alt, t, rise_v, true);
        if share != 0 {
            let mut buf = buffer(w, h);
            let br = buf.get_rect();
            scene(&mut buf, br, world + 1, alt, t, rise_v, false);
            buf.alpha_set(share);
            s.bl(&buf, (rect.x, rect.y));
            buf.set_alpha(None);
        } else if world < 4 {
            prebuild(world + 1, w, h);
        }
    }
    s.set_clip(Some(clip));
}
