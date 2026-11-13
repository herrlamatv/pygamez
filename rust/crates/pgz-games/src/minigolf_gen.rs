//! minigolf_gen.rs
//! ===============
//! Bahn-Generator fuer Minigolf - uebertragen aus `games/minigolf_gen.py`.
//!
//! Aus einem Seed entstehen reproduzierbare Bahnen (Kurs 7, Bahn 3 sieht
//! ueberall gleich aus). Jede Bahnfamilie legt zuerst den Weg vom Abschlag
//! zum Loch fest und baut die Hindernisse darum herum; `sanitize` entfernt
//! zum Schluss alles, was auf Abschlag oder Loch liegt.
//!
//! Eine Bahn ist ein [`Hole`]: Par, Abschlag, Loch, Groesse und die 15
//! Hindernis-Listen (je Eintrag die Zahlen wie in Python, siehe [`make_hole`]).

use pgz_engine::py;
use serde_json::{json, Map, Value};

/// Kursbreite in Bahn-Einheiten.
pub const CW: f64 = 100.0;
/// Kurshoehe.
pub const CH: f64 = 160.0;
/// Bandenbreite am Rand.
pub const BORDER: f64 = 3.0;
/// Ballradius.
pub const BALL_R: f64 = 1.7;
const X0: f64 = BORDER;
const X1: f64 = CW - BORDER;
const Y0: f64 = BORDER;
const Y1: f64 = CH - BORDER;

/// Bahnen je Runde.
pub const HOLES_PER_ROUND: usize = 9;
/// Tour-Kurse.
pub const TOUR_COURSES: i64 = 38;
/// Classic + Pro.
pub const HANDMADE_HOLES: usize = 18;
/// Alle Bahnen.
pub const TOTAL_HOLES: usize = HANDMADE_HOLES + TOUR_COURSES as usize * HOLES_PER_ROUND;
const MIN_PASS: f64 = 12.0;
const SEED: u64 = 0x9E3779B1;
const GAPS: [f64; 4] = [24.0, 21.0, 18.0, 15.0];

/// Alle Listen-Schluessel einer Bahn in der Reihenfolge von `make_hole`.
pub const HOLE_LISTS: [&str; 15] = [
    "walls", "sand", "water", "slopes", "bumpers", "movers", "mills", "tunnels", "ice", "boosters", "magnets",
    "gates", "sticky", "spinners", "jumps",
];

/// Index eines Listen-Schluessels.
pub fn list_index(key: &str) -> usize {
    HOLE_LISTS.iter().position(|k| *k == key).unwrap_or(0)
}

/// Eine Bahn (Python: das Bahn-Dict).
#[derive(Clone, Debug, PartialEq)]
pub struct Hole {
    /// Par.
    pub par: i64,
    /// Abschlag.
    pub tee: (f64, f64),
    /// Loch.
    pub cup: (f64, f64),
    /// Breite.
    pub w: f64,
    /// Hoehe.
    pub h: f64,
    /// Die 15 Hindernis-Listen (Reihenfolge [`HOLE_LISTS`]).
    pub lists: Vec<Vec<Vec<f64>>>,
    /// Bahnfamilie (nur erzeugte Bahnen).
    pub family: Option<String>,
}

impl Hole {
    /// Liste eines Schluessels.
    pub fn get(&self, key: &str) -> &Vec<Vec<f64>> {
        &self.lists[list_index(key)]
    }

    /// Liste eines Schluessels (veraenderbar).
    pub fn get_mut(&mut self, key: &str) -> &mut Vec<Vec<f64>> {
        &mut self.lists[list_index(key)]
    }

    /// Als JSON-Objekt (Schluesselreihenfolge wie das Python-Dict).
    pub fn to_json(&self) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("par".into(), json!(self.par));
        m.insert("tee".into(), json!([self.tee.0, self.tee.1]));
        m.insert("cup".into(), json!([self.cup.0, self.cup.1]));
        m.insert("w".into(), json!(self.w));
        m.insert("h".into(), json!(self.h));
        for (i, key) in HOLE_LISTS.iter().enumerate() {
            m.insert((*key).into(), json!(self.lists[i]));
        }
        if let Some(f) = &self.family {
            m.insert("family".into(), json!(f));
        }
        m
    }

    /// `normalize(dict)` - aus JSON, fehlende Schluessel ergaenzt.
    pub fn from_json(v: &Value) -> Hole {
        let Some(o) = v.as_object() else {
            return make_hole(3, (CW / 2.0, CH - 18.0), (CW / 2.0, 22.0));
        };
        let num = |v: Option<&Value>| v.and_then(|x| x.as_f64().or_else(|| x.as_bool().map(|b| b as i64 as f64)));
        let pt = |v: Option<&Value>, def: (f64, f64)| -> (f64, f64) {
            match v.and_then(|x| x.as_array()) {
                Some(a) if a.len() >= 2 => (num(a.first()).unwrap_or(def.0), num(a.get(1)).unwrap_or(def.1)),
                _ => def,
            }
        };
        let mut lists = Vec::new();
        for key in HOLE_LISTS {
            let items = o.get(key).and_then(|x| x.as_array()).map(|a| a.iter().map(|it| it.as_array().map(|r| r.iter().map(|z| num(Some(z)).unwrap_or(0.0)).collect()).unwrap_or_default()).collect()).unwrap_or_default();
            lists.push(items);
        }
        Hole {
            par: o.get("par").and_then(|p| p.as_f64()).map(|p| p as i64).unwrap_or(3),
            tee: pt(o.get("tee"), (CW / 2.0, CH - 18.0)),
            cup: pt(o.get("cup"), (CW / 2.0, 22.0)),
            w: num(o.get("w")).unwrap_or(CW),
            h: num(o.get("h")).unwrap_or(CH),
            lists,
            family: o.get("family").and_then(|f| f.as_str()).map(|s| s.to_string()),
        }
    }
}

/// `make_hole(par, tee, cup)` - leere Listen; mit [`Hole::get_mut`] fuellen.
pub fn make_hole(par: i64, tee: (f64, f64), cup: (f64, f64)) -> Hole {
    Hole { par, tee, cup, w: CW, h: CH, lists: vec![Vec::new(); 15], family: None }
}

/// Baustein fuer handgebaute Bahnen: `hole(par, tee, cup, &[("walls", &[..])])`.
pub fn hole_with(par: i64, tee: (f64, f64), cup: (f64, f64), items: &[(&str, &[&[f64]])]) -> Hole {
    let mut h = make_hole(par, tee, cup);
    for (key, its) in items {
        *h.get_mut(key) = its.iter().map(|it| it.to_vec()).collect();
    }
    h
}

fn clampx(x: f64) -> f64 {
    clampx_m(x, 6.0)
}

fn clampx_m(x: f64, margin: f64) -> f64 {
    (X0 + margin).max((X1 - margin).min(x))
}

fn gap_center(side: i64, gap: f64) -> f64 {
    if side == 0 {
        X0 + gap / 2.0
    } else {
        X1 - gap / 2.0
    }
}

fn row(side: i64, gap: f64, y: f64) -> Vec<f64> {
    let width = (X1 - X0) - gap;
    if side == 0 {
        vec![X0 + gap, y, width, 7.0]
    } else {
        vec![X0, y, width, 7.0]
    }
}

fn sand_patch(rng: &mut py::Random, x: f64, y: f64) -> Vec<f64> {
    let w = rng.uniform(16.0, 26.0);
    let h = rng.uniform(12.0, 18.0);
    let x = (X0 + 1.0).max((X1 - 1.0 - w).min(x - w / 2.0));
    let y = (Y0 + 1.0).max((Y1 - 1.0 - h).min(y - h / 2.0));
    vec![x, y, w, h]
}

fn scatter_sand(rng: &mut py::Random, hole: &mut Hole, spots: &[(f64, f64)]) {
    for &(x, y) in spots {
        if rng.random() < 0.65 {
            let p = sand_patch(rng, x, y);
            hole.get_mut("sand").push(p);
        }
    }
}

/// `_sanitize(hole)` - Hindernisse auf Abschlag/Loch weg, beides ins Feld.
pub fn sanitize(hole: &mut Hole) {
    let (cw, ch) = (hole.w, hole.h);
    let (x0, x1, y0, y1) = (BORDER, cw - BORDER, BORDER, ch - BORDER);
    let clamp = |v: f64, lo: f64, hi: f64, margin: f64| (lo + margin).max((hi - margin).min(v));
    hole.tee = (clamp(hole.tee.0, x0, x1, BALL_R + 3.0), clamp(hole.tee.1, y0, y1, BALL_R + 3.0));
    hole.cup = (clamp(hole.cup.0, x0, x1, BALL_R + 5.0), clamp(hole.cup.1, y0, y1, BALL_R + 5.0));
    let pts = [hole.tee, hole.cup];
    let clear = BALL_R + 3.0;
    let rect_free = |r: &[f64]| {
        let (x, y, w, h) = (r[0], r[1], r[2], r[3]);
        pts.iter().all(|(px, py)| !(x - clear < *px && *px < x + w + clear && y - clear < *py && *py < y + h + clear))
    };
    let circle_free = |cx: f64, cy: f64, r: f64| pts.iter().all(|(px, py)| py::hypot(cx - px, cy - py) > r + clear);
    for key in ["walls", "water", "gates", "jumps"] {
        hole.get_mut(key).retain(|r| rect_free(r));
    }
    hole.get_mut("bumpers").retain(|b| circle_free(b[0], b[1], b[2]));
    hole.get_mut("mills").retain(|m| circle_free(m[0], m[1], m[2]));
    hole.get_mut("spinners").retain(|s| circle_free(s[0], s[1], s[2]));
    hole.get_mut("tunnels").retain(|t| circle_free(t[0], t[1], t[4]) && circle_free(t[2], t[3], t[4]));
    hole.get_mut("movers").retain(|m| rect_free(&[m[0], m[1], m[2], m[3]]) && rect_free(&[m[0] + m[4], m[1] + m[5], m[2], m[3]]));
}

// ------------------------------------------------------------ Bahnfamilien

fn f_straight(rng: &mut py::Random, tier: usize) -> Hole {
    let cx = rng.uniform(38.0, 62.0);
    let gap = GAPS[tier] + 4.0;
    let ymid = rng.uniform(66.0, 88.0);
    let half = (X1 - X0 - gap) / 2.0;
    let cup_y = rng.uniform(20.0, 30.0);
    let mut hole = make_hole(if tier < 2 { 2 } else { 3 }, (cx, 142.0), (cx, cup_y));
    *hole.get_mut("walls") = vec![vec![X0, ymid, half, 8.0], vec![X1 - half, ymid, half, 8.0]];
    scatter_sand(rng, &mut hole, &[(X0 + 14.0, ymid + 26.0), (X1 - 14.0, ymid + 26.0)]);
    if tier >= 2 && rng.random() < 0.6 {
        hole.get_mut("bumpers").push(vec![cx, ymid - 22.0, 6.0]);
    }
    sanitize(&mut hole);
    hole
}

fn f_gates(rng: &mut py::Random, tier: usize, idx: usize) -> Hole {
    let n = 1.max(4.min(1 + tier as i64 + if idx >= 6 { 1 } else { 0 }));
    let gap = GAPS[tier];
    let ys: Vec<f64> = if n == 1 { vec![92.0] } else { (0..n).map(|i| 126.0 - i as f64 * (92.0 / (n - 1) as f64)).collect() };
    let side = rng.randint(0, 1);
    let mut walls = Vec::new();
    let mut sides = Vec::new();
    for (i, y) in ys.iter().enumerate() {
        let s = (side + i as i64) % 2;
        sides.push(s);
        walls.push(row(s, gap, *y));
    }
    let tee = (gap_center(sides[0], gap), 146.0);
    let top = ys[ys.len() - 1];
    let cup_x = if tier >= 2 && rng.random() < 0.5 { gap_center(1 - sides[sides.len() - 1], gap) } else { gap_center(sides[sides.len() - 1], gap) };
    let mut hole = make_hole(5.min(2 + n), tee, (cup_x, 16f64.max(top - 16.0)));
    *hole.get_mut("walls") = walls;
    for i in 0..ys.len() - 1 {
        scatter_sand(rng, &mut hole, &[(gap_center(1 - sides[i], gap), ys[i] - 14.0)]);
    }
    sanitize(&mut hole);
    hole
}

fn f_dogleg(rng: &mut py::Random, tier: usize) -> Hole {
    let mut xw = rng.uniform(38.0, 58.0);
    let ytop = rng.uniform(32.0, 44.0);
    let ylen = rng.uniform(72.0, 96.0);
    let flip = rng.random() < 0.5;
    if flip {
        xw = CW - xw - 9.0;
    }
    let mut walls = vec![vec![xw, ytop, 9.0, ylen]];
    if tier >= 2 {
        let by = rng.uniform(96.0, 116.0);
        if flip {
            walls.push(vec![X0, by, rng.uniform(22.0, 30.0), 8.0]);
        } else {
            let w = rng.uniform(22.0, 30.0);
            walls.push(vec![X1 - w, by, w, 8.0]);
        }
    }
    let tee_x = if !flip { xw / 2.0 } else { (xw + 9.0 + X1) / 2.0 };
    let cup_x = if !flip { (xw + 9.0 + X1) / 2.0 } else { xw / 2.0 };
    let mut hole = make_hole(if tier < 2 { 3 } else { 4 }, (clampx(tee_x), 142.0), (clampx(cup_x), 16f64.max(ytop - 14.0)));
    *hole.get_mut("walls") = walls;
    scatter_sand(rng, &mut hole, &[(xw + 4.5, ytop + ylen + 12.0)]);
    if tier >= 1 && rng.random() < 0.5 {
        let off = rng.choice(&[-18.0, 18.0]);
        hole.get_mut("bumpers").push(vec![clampx(cup_x + off), ytop + 18.0, 6.0]);
    }
    sanitize(&mut hole);
    hole
}

fn f_bumpers(rng: &mut py::Random, tier: usize) -> Hole {
    let cx = rng.uniform(40.0, 60.0);
    let k = 3 + tier;
    let mut bumpers: Vec<Vec<f64>> = Vec::new();
    for _ in 0..k * 4 {
        if bumpers.len() >= k {
            break;
        }
        let x = rng.uniform(X0 + 10.0, X1 - 10.0);
        let y = rng.uniform(40.0, 122.0);
        let r = rng.uniform(5.0, 7.0);
        if bumpers.iter().all(|b| py::hypot(x - b[0], y - b[1]) > r + b[2] + 8.0) {
            bumpers.push(vec![x, y, r]);
        }
    }
    let cup_x = clampx(cx + rng.uniform(-14.0, 14.0));
    let cup_y = rng.uniform(18.0, 28.0);
    let mut hole = make_hole(3, (cx, 142.0), (cup_x, cup_y));
    *hole.get_mut("bumpers") = bumpers;
    scatter_sand(rng, &mut hole, &[(cx, 118.0)]);
    sanitize(&mut hole);
    hole
}

fn f_water(rng: &mut py::Random, tier: usize) -> Hole {
    let lane = GAPS[tier] + 8.0;
    let cx = rng.uniform(34.0, 66.0);
    let y0 = rng.uniform(48.0, 66.0);
    let h = rng.uniform(30.0, 46.0);
    let left_w = (cx - lane / 2.0) - X0;
    let right_w = X1 - (cx + lane / 2.0);
    let mut water = Vec::new();
    if left_w > 8.0 {
        water.push(vec![X0, y0, left_w, h]);
    }
    if right_w > 8.0 {
        water.push(vec![cx + lane / 2.0, y0, right_w, h]);
    }
    let mut walls = Vec::new();
    if tier >= 2 {
        let gap = (MIN_PASS + 3.0).max(GAPS[tier]);
        let gx = clampx_m(cx + rng.uniform(-8.0, 8.0), gap / 2.0 + 2.0);
        walls = vec![vec![X0, y0 - 16.0, 2f64.max(gx - gap / 2.0 - X0), 7.0], vec![gx + gap / 2.0, y0 - 16.0, 2f64.max(X1 - gx - gap / 2.0), 7.0]];
    }
    let cup_x = clampx(cx + rng.uniform(-10.0, 10.0));
    let cup_y = rng.uniform(18.0, 30.0);
    let mut hole = make_hole(if tier < 2 { 3 } else { 4 }, (cx, 144.0), (cup_x, cup_y));
    *hole.get_mut("walls") = walls;
    *hole.get_mut("water") = water;
    scatter_sand(rng, &mut hole, &[(cx, y0 + h + 16.0)]);
    sanitize(&mut hole);
    hole
}

fn f_ramp(rng: &mut py::Random, tier: usize) -> Hole {
    let y0 = rng.uniform(40.0, 56.0);
    let h = rng.uniform(44.0, 60.0);
    let x0 = rng.uniform(16.0, 26.0);
    let w = rng.uniform(52.0, 66.0);
    let accel = 20.0 + tier as f64 * 5.0;
    let cx = clampx(x0 + w / 2.0 + rng.uniform(-10.0, 10.0));
    let cup_y = rng.uniform(16.0, 26.0);
    let mut hole = make_hole(if tier < 2 { 3 } else { 4 }, (cx, 146.0), (cx, cup_y));
    *hole.get_mut("slopes") = vec![vec![x0, y0, w, h, 0.0, accel]];
    scatter_sand(rng, &mut hole, &[(x0 + 8.0, y0 + h + 14.0), (x0 + w - 8.0, y0 + h + 14.0)]);
    if tier >= 2 {
        let off = rng.choice(&[-24.0, 24.0]);
        hole.get_mut("bumpers").push(vec![clampx(cx + off), y0 - 12.0, 6.0]);
    }
    sanitize(&mut hole);
    hole
}

fn f_chicane(rng: &mut py::Random, tier: usize) -> Hole {
    let xa = rng.uniform(26.0, 36.0);
    let xb = rng.uniform(60.0, 70.0);
    let ya = rng.uniform(34.0, 46.0);
    let la = rng.uniform(52.0, 66.0);
    let yb = rng.uniform(84.0, 96.0);
    let lb = rng.uniform(48.0, 62.0);
    let cup_y = rng.uniform(16.0, 26.0);
    let mut hole = make_hole(4, (clampx(xa / 2.0), 146.0), (clampx((xb + 8.0 + X1) / 2.0), cup_y));
    *hole.get_mut("walls") = vec![vec![xa, ya, 8.0, la], vec![xb, yb, 8.0, lb]];
    scatter_sand(rng, &mut hole, &[(xa + 22.0, ya + la + 10.0)]);
    if tier >= 3 && rng.random() < 0.5 {
        hole.get_mut("bumpers").push(vec![clampx((xa + xb) / 2.0), (ya + yb) / 2.0, 6.0]);
    }
    sanitize(&mut hole);
    hole
}

fn f_mill(rng: &mut py::Random, tier: usize) -> Hole {
    let w = 42.0 - tier as f64 * 3.0;
    let xl = 50.0 - w / 2.0;
    let xr = 50.0 + w / 2.0;
    let (ytop, ylen) = (22.0, 100.0);
    let arm = w / 2.0 - 4.5;
    let n = if tier < 2 { 1 } else { 2 };
    let mut mills = Vec::new();
    for i in 0..n {
        let y = 104.0 - i as f64 * 44.0;
        let sgn = rng.choice(&[-1.0, 1.0]);
        let speed = sgn * rng.uniform(1.1, 1.9);
        mills.push(vec![50.0, y, arm, 2.0, speed]);
    }
    let mut hole = make_hole(3 + n, (50.0, 146.0), (50.0, 16.0));
    *hole.get_mut("walls") = vec![vec![xl - 8.0, ytop, 8.0, ylen], vec![xr, ytop, 8.0, ylen]];
    *hole.get_mut("mills") = mills;
    scatter_sand(rng, &mut hole, &[(50.0, 130.0)]);
    sanitize(&mut hole);
    hole
}

fn f_island(rng: &mut py::Random, tier: usize) -> Hole {
    let cx = rng.uniform(38.0, 62.0);
    let cy = rng.uniform(30.0, 42.0);
    let (half_w, half_h) = (15.0, 13.0);
    let neck = (MIN_PASS + 2.0).max(GAPS[tier].min(2.0 * half_w - 6.0));
    let top = cy - half_h;
    let bot = cy + half_h;
    let mut water = Vec::new();
    let lw = (cx - half_w) - X0;
    let rw = X1 - (cx + half_w);
    if lw > 8.0 {
        water.push(vec![X0, top, lw, 2.0 * half_h]);
    }
    if rw > 8.0 {
        water.push(vec![cx + half_w, top, rw, 2.0 * half_h]);
    }
    let blw = (cx - neck / 2.0) - X0;
    let brw = X1 - (cx + neck / 2.0);
    if blw > 8.0 {
        water.push(vec![X0, bot, blw, 15.0]);
    }
    if brw > 8.0 {
        water.push(vec![cx + neck / 2.0, bot, brw, 15.0]);
    }
    let mut hole = make_hole(if tier < 2 { 3 } else { 4 }, (clampx(cx), 146.0), (cx, cy));
    *hole.get_mut("water") = water;
    scatter_sand(rng, &mut hole, &[(cx, bot + 34.0)]);
    if tier >= 2 {
        hole.get_mut("slopes").push(vec![cx - 26.0, bot + 22.0, 52.0, 30.0, 0.0, -10.0]);
    }
    sanitize(&mut hole);
    hole
}

fn f_mover(rng: &mut py::Random, tier: usize) -> Hole {
    let bw = 14.0;
    let gap = bw + 2.0 * MIN_PASS + 2.0;
    let gx = clampx_m(rng.uniform(34.0, 66.0), gap / 2.0 + 3.0) - gap / 2.0;
    let y = rng.uniform(78.0, 104.0);
    let left_w = gx - X0;
    let right_w = X1 - (gx + gap);
    let mut walls = Vec::new();
    if left_w > 2.0 {
        walls.push(vec![X0, y, left_w, 8.0]);
    }
    if right_w > 2.0 {
        walls.push(vec![gx + gap, y, right_w, 8.0]);
    }
    let speed = 18.0 + tier as f64 * 4.0;
    let cup_x = clampx(gx + gap / 2.0 + rng.uniform(-16.0, 16.0));
    let cup_y = rng.uniform(18.0, 30.0);
    let mut hole = make_hole(4, (clampx(gx + gap / 2.0), 146.0), (cup_x, cup_y));
    *hole.get_mut("walls") = walls;
    *hole.get_mut("movers") = vec![vec![gx, y, bw, 8.0, gap - bw, 0.0, speed]];
    scatter_sand(rng, &mut hole, &[(gx + gap / 2.0, y + 28.0)]);
    if tier >= 3 && rng.random() < 0.5 {
        let off = rng.choice(&[-20.0, 20.0]);
        hole.get_mut("bumpers").push(vec![clampx(cup_x + off), y - 26.0, 6.0]);
    }
    sanitize(&mut hole);
    hole
}

const POOLS: [&[&str]; 4] = [
    &["gates", "dogleg", "bumpers", "straight", "water", "ramp"],
    &["gates", "dogleg", "bumpers", "water", "ramp", "chicane", "island"],
    &["gates", "dogleg", "bumpers", "water", "ramp", "chicane", "mill", "island", "mover"],
    &["gates", "dogleg", "bumpers", "water", "ramp", "chicane", "mill", "island", "mover"],
];
const FINALE: [&str; 5] = ["mill", "island", "mover", "gates", "chicane"];

/// `tier_of(course)` - Schwierigkeitsstufe 0..3.
pub fn tier_of(course: i64) -> usize {
    0.max(3.min(py::fdiv(course - 1, 10))) as usize
}

fn family_for(course: i64, idx: usize, rng: &mut py::Random, used: &[(String, i64)]) -> &'static str {
    if idx == 0 {
        return "straight";
    }
    let tier = tier_of(course);
    let mut pool: Vec<&'static str> = if idx < HOLES_PER_ROUND - 1 { POOLS[tier].to_vec() } else { FINALE[..3 + tier].to_vec() };
    rng.shuffle(&mut pool);
    for name in &pool {
        let n = used.iter().find(|e| e.0 == *name).map(|e| e.1).unwrap_or(0);
        if n < 2 {
            return name;
        }
    }
    pool[0]
}

/// `generate(course, idx, used)`
pub fn generate(course: i64, idx: usize, used: Option<&mut Vec<(String, i64)>>) -> Hole {
    let course = 1.max(TOUR_COURSES.min(course));
    let idx = idx.min(HOLES_PER_ROUND - 1);
    let mut rng = py::Random::new(SEED + course as u64 * 9176 + idx as u64 * 131);
    let empty = Vec::new();
    let name = family_for(course, idx, &mut rng, used.as_deref().unwrap_or(&empty));
    if let Some(u) = used {
        match u.iter_mut().find(|e| e.0 == name) {
            Some(e) => e.1 += 1,
            None => u.push((name.to_string(), 1)),
        }
    }
    let tier = tier_of(course);
    let mut hole = match name {
        "straight" => f_straight(&mut rng, tier),
        "gates" => f_gates(&mut rng, tier, idx),
        "dogleg" => f_dogleg(&mut rng, tier),
        "bumpers" => f_bumpers(&mut rng, tier),
        "water" => f_water(&mut rng, tier),
        "ramp" => f_ramp(&mut rng, tier),
        "chicane" => f_chicane(&mut rng, tier),
        "mill" => f_mill(&mut rng, tier),
        "island" => f_island(&mut rng, tier),
        _ => f_mover(&mut rng, tier),
    };
    hole.family = Some(name.to_string());
    hole
}

/// `course_holes(course)` - alle neun Bahnen, nach Par sortiert.
pub fn course_holes(course: i64) -> Vec<Hole> {
    let mut used = Vec::new();
    let holes: Vec<Hole> = (0..HOLES_PER_ROUND).map(|i| generate(course, i, Some(&mut used))).collect();
    let first = holes[0].clone();
    let mut rest: Vec<Hole> = holes[1..].to_vec();
    rest.sort_by_key(|h| h.par);
    let mut out = vec![first];
    out.extend(rest);
    out
}

/// `course_par(course)`
pub fn course_par(course: i64) -> i64 {
    course_holes(course).iter().map(|h| h.par).sum()
}
