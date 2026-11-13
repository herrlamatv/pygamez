//! crossyroad_draw.rs
//! ==================
//! Voxel-Renderer fuer Crossy Road - uebertragen aus
//! `games/crossyroad_draw.py`.
//!
//! Schraege Projektion `bild_x = x*tw + y*sk`, `bild_y = -y*th - z*zh`; jedes
//! Modell wird einmal je Kachelgroesse als Sprite gerendert (Quader nach
//! topologischer Sortierung, Schatten eingebacken), jede Boden-Variante
//! einmal als Streifen.

use std::collections::HashMap;

use pgz_engine::prelude::*;

use crate::crossyroad_world::{self as cw, Kind, B};

const MV: i32 = 7;
const SHADOW_ALPHA: i32 = 64;
const DIM: (i32, i32, i32) = (30, 40, 66);

/// Grasfarbe A.
pub const GRASS_A: (i32, i32, i32) = (160, 222, 98);
const GRASS_B: (i32, i32, i32) = (150, 212, 90);
const GRASS_SIDE: (i32, i32, i32) = (112, 160, 64);
const ROAD: (i32, i32, i32) = (86, 90, 104);
const ROAD_SIDE: (i32, i32, i32) = (62, 64, 76);
const ROAD_LINE: (i32, i32, i32) = (226, 228, 222);
const WATER_A: (i32, i32, i32) = (96, 198, 248);
const WATER_B: (i32, i32, i32) = (88, 190, 242);
const WAVE: (i32, i32, i32) = (150, 224, 255);
const RAIL_BED: (i32, i32, i32) = (134, 122, 118);
const RAIL_SIDE: (i32, i32, i32) = (104, 94, 92);
const SLEEPER: (i32, i32, i32) = (116, 84, 62);
const STEEL: (i32, i32, i32) = (198, 202, 214);
const STEEL_SIDE: (i32, i32, i32) = (124, 128, 142);

/// `shade(col, f)`
pub fn shade(col: (i32, i32, i32), f: f64) -> (i32, i32, i32) {
    (255.min((col.0 as f64 * f) as i32), 255.min((col.1 as f64 * f) as i32), 255.min((col.2 as f64 * f) as i32))
}

/// `dim(col, f)`
fn dim(col: (i32, i32, i32), f: f64) -> (i32, i32, i32) {
    (
        (col.0 as f64 + (DIM.0 - col.0) as f64 * f) as i32,
        (col.1 as f64 + (DIM.1 - col.1) as f64 * f) as i32,
        (col.2 as f64 + (DIM.2 - col.2) as f64 * f) as i32,
    )
}

fn rnd(v: f64) -> i32 {
    py::round(v) as i32
}

/// Ein gerendertes Sprite: (Flaeche, Anker x, Anker y).
pub type Sprite = (Surface, i32, i32);

/// Projektion + Caches fuer EINE Kachelgroesse.
pub struct Voxel {
    /// Kachelbreite.
    pub tw: i32,
    /// Kacheltiefe (Bild).
    pub th: i32,
    /// Versatz je Reihe nach rechts.
    pub sk: i32,
    /// Hoehe je Einheit.
    pub zh: i32,
    /// Schatten einbacken?
    pub shadows: bool,
    sprites: HashMap<String, Sprite>,
    strips: HashMap<(Kind, i64, Option<u64>, bool), Sprite>,
    scaled: HashMap<(String, u64, u64), (Surface, f64, f64)>,
}

impl Voxel {
    /// `Voxel(tw, shadows)`
    pub fn new(tw: i32, shadows: bool) -> Voxel {
        let tw = 12.max(tw);
        Voxel {
            tw,
            th: 8.max(rnd(tw as f64 * 0.78)),
            sk: 2.max(rnd(tw as f64 * 0.28)),
            zh: 6.max(rnd(tw as f64 * 0.66)),
            shadows,
            sprites: HashMap::new(),
            strips: HashMap::new(),
            scaled: HashMap::new(),
        }
    }

    /// `proj(x, y, z)`
    pub fn proj(&self, x: f64, y: f64, z: f64) -> (f64, f64) {
        (x * self.tw as f64 + y * self.sk as f64, -y * self.th as f64 - z * self.zh as f64)
    }

    fn depth(&self, b: &B) -> f64 {
        let cx = (b.0 + b.3) * 0.5;
        let cy = (b.1 + b.4) * 0.5;
        let cz = (b.2 + b.5) * 0.5;
        -cx * self.sk as f64 / self.tw as f64 + cy - cz * self.th as f64 / self.zh as f64
    }

    fn rect(&self, b: &B) -> (f64, f64, f64, f64) {
        let (mut minx, mut miny, mut maxx, mut maxy) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
        for x in [b.0, b.3] {
            for y in [b.1, b.4] {
                for z in [b.2, b.5] {
                    let (px, py_) = self.proj(x, y, z);
                    minx = minx.min(px);
                    miny = miny.min(py_);
                    maxx = maxx.max(px);
                    maxy = maxy.max(py_);
                }
            }
        }
        (minx, miny, maxx, maxy)
    }

    /// `order(boxes)` - Zeichenreihenfolge hinten -> vorn.
    pub fn order(&self, boxes: &[B]) -> Vec<usize> {
        let n = boxes.len();
        let eps = 1e-6;
        let rects: Vec<_> = boxes.iter().map(|b| self.rect(b)).collect();
        let depth: Vec<f64> = boxes.iter().map(|b| self.depth(b)).collect();
        let mut deps: Vec<Vec<usize>> = vec![Vec::new(); n];
        for i in 0..n {
            let ra = rects[i];
            for j in i + 1..n {
                let rb = rects[j];
                if ra.2 <= rb.0 || rb.2 <= ra.0 || ra.3 <= rb.1 || rb.3 <= ra.1 {
                    continue;
                }
                let (a, b) = (&boxes[i], &boxes[j]);
                let first = if a.1 >= b.4 - eps {
                    i
                } else if b.1 >= a.4 - eps {
                    j
                } else if a.3 <= b.0 + eps {
                    i
                } else if b.3 <= a.0 + eps {
                    j
                } else if a.5 <= b.2 + eps {
                    i
                } else if b.5 <= a.2 + eps {
                    j
                } else if depth[i] >= depth[j] {
                    i
                } else {
                    j
                };
                let second = if first == i { j } else { i };
                deps[second].push(first);
            }
        }
        let mut done = vec![false; n];
        let mut out = Vec::new();
        while out.len() < n {
            let mut best: Option<usize> = None;
            for k in 0..n {
                if done[k] || !deps[k].iter().all(|d| done[*d]) {
                    continue;
                }
                if best.is_none_or(|b| depth[k] > depth[b]) {
                    best = Some(k);
                }
            }
            if best.is_none() {
                for k in 0..n {
                    if !done[k] && best.is_none_or(|b| depth[k] > depth[b]) {
                        best = Some(k);
                    }
                }
            }
            let b = best.unwrap_or(0);
            done[b] = true;
            out.push(b);
        }
        out
    }

    /// `render(boxes, shadow)` - (Flaeche, ax, ay); (ax, ay) = Bildpunkt von (0,0,0).
    pub fn render(&self, boxes: &[B], shadow: bool) -> Sprite {
        let mut pts: Vec<(f64, f64)> = Vec::new();
        for b in boxes {
            let r = self.rect(b);
            pts.push((r.0, r.1));
            pts.push((r.2, r.3));
        }
        let mut shadow_polys: Vec<[(f64, f64); 4]> = Vec::new();
        if shadow && self.shadows {
            for b in boxes {
                if b.2 > 0.6 {
                    continue;
                }
                let q = [self.proj(b.0 + 0.1, b.1 - 0.02, 0.0), self.proj(b.3 + 0.18, b.1 - 0.02, 0.0), self.proj(b.3 + 0.18, b.4 - 0.1, 0.0), self.proj(b.0 + 0.1, b.4 - 0.1, 0.0)];
                shadow_polys.push(q);
                pts.extend(q);
            }
        }
        let minx = pts.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let maxx = pts.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let miny = pts.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let maxy = pts.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
        let ax = (-minx) as i32 + 2;
        let ay = (-miny) as i32 + 2;
        let w = (maxx - minx) as i32 + 5;
        let h = (maxy - miny) as i32 + 5;
        let mut surf = pg::surface_alpha(w, h);
        if !shadow_polys.is_empty() {
            let mut layer = pg::surface_alpha(w, h);
            for q in &shadow_polys {
                let pp: Vec<(i32, i32)> = q.iter().map(|p| (rnd(ax as f64 + p.0), rnd(ay as f64 + p.1))).collect();
                draw::polygon(&mut layer, rgba(0, 0, 0, 255), &pp, 0);
            }
            layer.fill_f(rgba(255, 255, 255, SHADOW_ALPHA), BLEND_RGBA_MULT);
            surf.bl(&layer, (0, 0));
        }
        for i in self.order(boxes) {
            self.draw_box(&mut surf, &boxes[i], ax, ay);
        }
        (surf, ax, ay)
    }

    fn draw_box(&self, surf: &mut Surface, b: &B, ax: i32, ay: i32) {
        let &(x0, y0, z0, x1, y1, z1, col) = b;
        let p = |x: f64, y: f64, z: f64| {
            let (px, py_) = self.proj(x, y, z);
            (rnd(ax as f64 + px), rnd(ay as f64 + py_))
        };
        let east = shade(col, 0.70);
        let front = shade(col, 0.86);
        let top = col;
        draw::polygon(surf, east, &[p(x1, y0, z0), p(x1, y1, z0), p(x1, y1, z1), p(x1, y0, z1)], 0);
        draw::polygon(surf, front, &[p(x0, y0, z0), p(x1, y0, z0), p(x1, y0, z1), p(x0, y0, z1)], 0);
        draw::polygon(surf, top, &[p(x0, y0, z1), p(x1, y0, z1), p(x1, y1, z1), p(x0, y1, z1)], 0);
    }

    /// `sprite(key, shadow)` - gecacht nur nach dem Schluessel (wie Python).
    pub fn sprite(&mut self, key: &str, shadow: bool) -> Sprite {
        if let Some(s) = self.sprites.get(key) {
            return s.clone();
        }
        let got = self.render(&model_for(key), shadow);
        self.sprites.insert(key.to_string(), got.clone());
        got
    }

    /// `scaled(key, sx, sy)` - Squash & Stretch.
    pub fn scaled(&mut self, key: &str, sx: f64, sy: f64) -> (Surface, f64, f64) {
        let sx = py::round_to(sx, 2);
        let sy = py::round_to(sy, 2);
        if sx == 1.0 && sy == 1.0 {
            let (s, ax, ay) = self.sprite(key, false);
            return (s, ax as f64, ay as f64);
        }
        let ck = (key.to_string(), sx.to_bits(), sy.to_bits());
        if let Some(g) = self.scaled.get(&ck) {
            return g.clone();
        }
        let (surf, ax, ay) = self.sprite(key, false);
        let w = 1.max((surf.w() as f64 * sx) as i32);
        let h = 1.max((surf.h() as f64 * sy) as i32);
        let got = (transform::smoothscale(&surf, w, h), ax as f64 * sx, ay as f64 * sy);
        if self.scaled.len() > 240 {
            self.scaled.clear();
        }
        self.scaled.insert(ck, got.clone());
        got
    }

    /// `strip(kind, parity, lower, line)` - Boden-Streifen einer Reihe.
    pub fn strip(&mut self, kind: Kind, parity: i64, lower: Option<f64>, line: bool) -> Sprite {
        let key = (kind, parity, lower.map(|l| l.to_bits()), line);
        if let Some(s) = self.strips.get(&key) {
            return s.clone();
        }
        let lvl = kind.level();
        let (c0, c1) = (-MV, cw::COLS + MV);
        let left = c0 * self.tw - 2;
        let right = c1 * self.tw + self.sk + 2;
        let top = rnd(-(self.th as f64) - lvl * self.zh as f64);
        let bottom = rnd(-(lower.unwrap_or(lvl)) * self.zh as f64);
        let (ax, ay) = (-left, -top);
        let (w, h) = (right - left, bottom - top + 1);
        let mut surf = pg::surface(w, h);
        let p = |x: f64, y: f64, z: f64| {
            let (px, py_) = self.proj(x, y, z);
            (rnd(ax as f64 + px), rnd(ay as f64 + py_))
        };
        let base = match kind {
            Kind::Grass => {
                if parity != 0 {
                    GRASS_A
                } else {
                    GRASS_B
                }
            }
            Kind::Road => ROAD,
            Kind::River => {
                if parity != 0 {
                    WATER_A
                } else {
                    WATER_B
                }
            }
            Kind::Rail => RAIL_BED,
        };
        let side = match kind {
            Kind::Grass => GRASS_SIDE,
            Kind::Road => ROAD_SIDE,
            Kind::River => WATER_B,
            Kind::Rail => RAIL_SIDE,
        };
        surf.fill_c(dim(base, 0.6));
        for c in c0..c1 {
            let out = c < 0 || c >= cw::COLS;
            let f = if out { 0.42 } else { 0.0 };
            let mut col = if out { dim(base, f) } else { base };
            if kind == Kind::Grass && !out && (c as i64 + parity) % 2 == 0 {
                col = shade(col, 1.025);
            }
            let cf = c as f64;
            draw::polygon(&mut surf, col, &[p(cf, 0.0, lvl), p(cf + 1.0, 0.0, lvl), p(cf + 1.0, 1.0, lvl), p(cf, 1.0, lvl)], 0);
            if let Some(lw) = lower {
                let sc = if out { dim(side, f) } else { side };
                draw::polygon(&mut surf, sc, &[p(cf, 0.0, lw), p(cf + 1.0, 0.0, lw), p(cf + 1.0, 0.0, lvl), p(cf, 0.0, lvl)], 0);
            }
            match kind {
                Kind::River => {
                    let hh = cw::deco_hash(parity * 7 + 3, c as i64);
                    let wx = cf + (hh % 60) as f64 / 100.0;
                    let wy = 0.2 + ((hh >> 8) % 55) as f64 / 100.0;
                    let wc = if out { dim(WAVE, f) } else { WAVE };
                    draw::polygon(&mut surf, wc, &[p(wx, wy, lvl), p(wx + 0.3, wy, lvl), p(wx + 0.3, wy + 0.05, lvl), p(wx, wy + 0.05, lvl)], 0);
                }
                Kind::Road if line => {
                    let lc = if out { dim(ROAD_LINE, f) } else { ROAD_LINE };
                    draw::polygon(&mut surf, lc, &[p(cf + 0.25, 0.93, lvl), p(cf + 0.75, 0.93, lvl), p(cf + 0.75, 1.0, lvl), p(cf + 0.25, 1.0, lvl)], 0);
                }
                Kind::Rail => {
                    let sl = if out { dim(SLEEPER, f) } else { SLEEPER };
                    for sx0 in [0.1, 0.6] {
                        draw::polygon(&mut surf, sl, &[p(cf + sx0, 0.14, lvl), p(cf + sx0 + 0.28, 0.14, lvl), p(cf + sx0 + 0.28, 0.86, lvl), p(cf + sx0, 0.86, lvl)], 0);
                    }
                }
                _ => {}
            }
        }
        if kind == Kind::Rail {
            let (c0f, c1f) = (c0 as f64, c1 as f64);
            for ry in [0.26, 0.66] {
                let z1 = lvl + 0.05;
                draw::polygon(&mut surf, STEEL_SIDE, &[p(c0f, ry, lvl), p(c1f, ry, lvl), p(c1f, ry, z1), p(c0f, ry, z1)], 0);
                draw::polygon(&mut surf, STEEL, &[p(c0f, ry, z1), p(c1f, ry, z1), p(c1f, ry + 0.07, z1), p(c0f, ry + 0.07, z1)], 0);
            }
        }
        let got = (surf, ax, ay);
        self.strips.insert(key, got.clone());
        got
    }
}

/// `model_for(key)` - Quader-Liste zu einem Sprite-Schluessel.
pub fn model_for(key: &str) -> Vec<B> {
    if let Some((cid, facing)) = key.split_once(':') {
        return cw::rotate(&cw::char_model(cid), facing);
    }
    let flip = key.ends_with('<');
    let name = if flip { &key[..key.len() - 1] } else { key };
    let boxes = cw::model(name);
    if flip {
        let length = if name.starts_with("car") {
            cw::CAR_LEN
        } else if name.starts_with("truck") {
            cw::TRUCK_LEN
        } else if name == "engine" || name == "wagon" {
            cw::TRAIN_CAR
        } else {
            1.0
        };
        return cw::mirror_x(&boxes, length);
    }
    boxes
}
