//! geodash_draw.rs
//! ===============
//! Zeichnen fuer Geometry Dash - Spiel UND Level-Editor benutzen dieselben
//! Funktionen. Uebertragen aus `games/geodash_draw.py`.
//!
//! Die Welt wird in Bloecken gerechnet: `ts` ist die Kachelgroesse in Pixeln,
//! `cam_x`/`cam_y` die Welt-Koordinate der linken unteren Bildschirmecke.
//! Weltachse y zeigt nach OBEN, der Bildschirm nach unten.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::f64::consts::PI;

use pgz_engine::pg::surface_alpha;
use pgz_engine::prelude::*;

use crate::geodash_core::{self as core, Level};

/// Gelb.
pub const COL_YELLOW: (i32, i32, i32) = (255, 214, 40);
/// Pink.
pub const COL_PINK: (i32, i32, i32) = (255, 90, 210);
/// Blau.
pub const COL_BLUE: (i32, i32, i32) = (70, 200, 255);
/// Pads/Orbs je Wert.
pub const COL_PAD: [(i32, i32, i32); 3] = [COL_YELLOW, COL_PINK, COL_BLUE];
/// Portale je Spielform.
pub const COL_MODE: [(i32, i32, i32); 5] = [(90, 255, 110), (255, 110, 220), (255, 120, 50), (255, 190, 40), (70, 170, 255)];
/// Tempo-Portale.
pub const COL_SPEED: [(i32, i32, i32); 4] = [(255, 170, 60), (90, 200, 255), (90, 255, 140), (255, 90, 200)];
/// Muenze.
pub const COL_COIN: (i32, i32, i32) = (255, 205, 70);
/// Haken.
pub const COL_CHECK: (i32, i32, i32) = (90, 255, 140);
/// Spieler Grundfarbe.
pub const COL_P1: (i32, i32, i32) = (185, 242, 58);
/// Spieler Zweitfarbe.
pub const COL_P2: (i32, i32, i32) = (58, 214, 242);
/// Schwierigkeits-Farben.
pub const DIFF_COLORS: [(i32, i32, i32); 6] = [(90, 200, 255), (110, 230, 90), (255, 200, 60), (255, 120, 60), (255, 70, 150), (220, 40, 50)];

const WHITE: (i32, i32, i32) = (255, 255, 255);
const BLACK: (i32, i32, i32) = (0, 0, 0);

/// `COL_GRAV[g]`
pub fn col_grav(g: i64) -> (i32, i32, i32) {
    if g > 0 {
        (70, 200, 255)
    } else {
        (255, 225, 60)
    }
}

fn tup(c: Color) -> (i32, i32, i32) {
    (c.r as i32, c.g as i32, c.b as i32)
}

fn json_rgb(v: &serde_json::Value) -> Color {
    let a = v.as_array();
    let g = |i: usize| a.and_then(|a| a.get(i)).and_then(|x| x.as_i64()).unwrap_or(0);
    rgb(g(0), g(1), g(2))
}

/// `color_at(lv, xb, which)` - Hintergrund- (0) bzw. Bodenfarbe (1) an Welt-x.
pub fn color_at(lv: &Level, xb: f64, which: i64) -> Color {
    let mut col = json_rgb(&lv.data[if which == 0 { "bg" } else { "ground" }]);
    for &i in &lv.triggers {
        let tx = lv.ox[i];
        if tx as f64 > xb {
            break;
        }
        let p = &lv.par[i];
        if p[0] != which {
            continue;
        }
        let f = if p[4] > 0 { (xb - tx as f64) / 1.max(p[4]) as f64 } else { 1.0 };
        col = mix(col, rgb(p[1], p[2], p[3]), f);
    }
    col
}

fn glow_surface(radius: f64, color: (i32, i32, i32), strength: f64) -> Surface {
    let size = 4.max((radius * 2.0) as i32);
    let mut surf = surface_alpha(size, size);
    let c = size as f64 / 2.0;
    let steps = 6.max(py::ffdiv(radius, 2.0) as i32);
    for i in (1..=steps).rev() {
        let r = radius * i as f64 / steps as f64;
        let a = (90.0 * strength * py::pow(1.0 - i as f64 / steps as f64, 1.6)) as i32;
        draw::circle(&mut surf, rgba(color.0, color.1, color.2, a), (c as i32, c as i32), r as i32, 0);
    }
    surf
}

/// Zeichnet Welt, Objekte und Spieler bei gegebener Kachelgroesse.
#[derive(Default)]
pub struct Renderer {
    /// Kachelgroesse.
    pub ts: i32,
    /// Breite.
    pub w: i32,
    /// Hoehe.
    pub h: i32,
    blocks: HashMap<(i64, i64), Surface>,
    spikes: HashMap<(i64, bool), Surface>,
    glows: HashMap<(&'static str, i32, (i32, i32, i32), u64), Surface>,
    shade: Option<Surface>,
    players: HashMap<(i64, i64, i64), Surface>,
}

impl Renderer {
    /// Leerer Renderer.
    pub fn new() -> Self {
        Self::default()
    }

    fn cache_len(&self) -> usize {
        self.blocks.len() + self.spikes.len() + self.glows.len() + self.shade.is_some() as usize + self.players.len()
    }

    /// `resize(w, h, ts)`
    pub fn resize(&mut self, w: i32, h: i32, ts: f64) {
        let ts = 8.max(ts as i32);
        if (w, h, ts) == (self.w, self.h, self.ts) {
            return;
        }
        self.w = w;
        self.h = h;
        self.ts = ts;
        self.blocks.clear();
        self.spikes.clear();
        self.glows.clear();
        self.shade = None;
        self.players.clear();
    }

    /// Bildschirm-x der Welt-x (Bloecke).
    pub fn sx(&self, xb: f64, cam_x: f64) -> f64 {
        (xb - cam_x) * self.ts as f64
    }

    /// Bildschirm-y der Welthoehe yb (Bloecke).
    pub fn sy(&self, yb: f64, cam_y: f64) -> f64 {
        self.h as f64 - (yb - cam_y) * self.ts as f64
    }

    /// Block-Kachel (mask: 1 oben frei, 2 rechts, 4 unten, 8 links; half 1..4).
    pub fn block(&mut self, mask: i64, half: i64) -> &Surface {
        let ts = self.ts;
        self.blocks.entry((mask, half)).or_insert_with(|| {
            let mut surf = surface_alpha(ts, ts);
            let rc = match half {
                1 => rect(0, ts / 2, ts, ts - ts / 2),
                2 => rect(0, 0, ts / 2, ts),
                3 => rect(0, 0, ts, ts / 2),
                4 => rect(ts / 2, 0, ts - ts / 2, ts),
                _ => rect(0, 0, ts, ts),
            };
            for i in 0..rc.h {
                let f = i as f64 / 1.max(rc.h - 1) as f64;
                let c = mix((34, 36, 58), (8, 8, 16), f);
                draw::line(&mut surf, pg::alpha(c, 238), (rc.x, rc.y + i), (rc.right() - 1, rc.y + i), 1);
            }
            let lw = 1.max(ts / 14);
            let line = (250, 252, 255);
            if half != 0 {
                draw::rect(&mut surf, line, rc, lw);
            } else {
                if mask & 1 != 0 {
                    draw::rect(&mut surf, line, (0, 0, ts, lw), 0);
                }
                if mask & 2 != 0 {
                    draw::rect(&mut surf, line, (ts - lw, 0, lw, ts), 0);
                }
                if mask & 4 != 0 {
                    draw::rect(&mut surf, line, (0, ts - lw, ts, lw), 0);
                }
                if mask & 8 != 0 {
                    draw::rect(&mut surf, line, (0, 0, lw, ts), 0);
                }
                let inset = 3.max(ts / 4);
                let inner = rect(inset, inset, ts - 2 * inset, ts - 2 * inset);
                draw::rect(&mut surf, rgba(255, 255, 255, 26), inner, 1.max(lw / 2));
            }
            surf
        })
    }

    fn spike(&mut self, rot: i64, small: bool) -> &Surface {
        let ts = self.ts;
        self.spikes.entry((rot, small)).or_insert_with(|| {
            let tsf = ts as f64;
            let mut surf = surface_alpha(ts, ts);
            let top = if small { tsf * 0.5 } else { tsf * 0.06 };
            let pts = [(tsf * 0.06, (ts - 1) as f64), (tsf * 0.94, (ts - 1) as f64), (tsf * 0.5, top)];
            let lw = 1.max(ts / 16);
            draw::polygon(&mut surf, (12, 12, 22), &pts, 0);
            let glint = [(tsf * 0.5, top + (tsf - top) * 0.28), (tsf * 0.34, (ts - 3) as f64), (tsf * 0.5, (ts - 3) as f64)];
            draw::polygon(&mut surf, (60, 64, 90), &glint, 0);
            draw::polygon(&mut surf, (250, 252, 255), &pts, lw);
            if rot != 0 {
                surf = transform::rotate(&surf, -90.0 * rot as f64);
            }
            surf
        })
    }

    fn glow(&mut self, key: &'static str, radius: f64, color: (i32, i32, i32), strength: f64) -> &Surface {
        self.glows.entry((key, radius as i32, color, strength.to_bits())).or_insert_with(|| glow_surface(radius, color, strength))
    }

    fn shade_surf(&mut self) -> &Surface {
        let (w, h) = (self.w, self.h);
        self.shade.get_or_insert_with(|| {
            let mut surf = surface_alpha(1, 64);
            for i in 0..64 {
                let a = (120.0 * py::pow(1.0 - i as f64 / 63.0, 1.4)) as i32;
                surf.set_at((0, i), rgba(0, 0, 20, a));
            }
            transform::smoothscale(&surf, 1.max(w), 1.max(h))
        })
    }

    /// Hintergrundfarbe mit Parallax-Quadraten und Beat-Puls.
    pub fn draw_backdrop(&mut self, s: &mut Surface, bg: Color, cam_x: f64, cam_y: f64, pulse: f64) {
        let col = mix(bg, WHITE, 0.06 * pulse);
        s.fill_c(mix(col, BLACK, 0.25));
        let ts = self.ts as f64;
        for (layer, (speed, size, alpha)) in [(0.10, 4.6, 0.045), (0.25, 2.2, 0.06)].into_iter().enumerate() {
            let px = size * ts;
            let span = px * 2.6;
            let off = py::fpmod(cam_x * speed * ts, span);
            let n = (self.w as f64 / span) as i64 + 2;
            for i in -1..n {
                let x = (i as f64 * span - off) as i32;
                let k = py::ffdiv(cam_x * speed * ts, span) as i64 + i;
                let yb = (k * 37 + layer as i64 * 11).rem_euclid(7) as f64 / 7.0;
                let y = (self.h as f64 * (0.08 + 0.5 * yb) + cam_y * ts * speed) as i32;
                let c = mix(bg, WHITE, alpha + 0.03 * pulse);
                draw::rect_r(s, c, (x, y, px as i32, px as i32), if layer > 0 { 1.max(self.ts / 12) } else { 0 }, 2.max(self.ts / 6));
            }
        }
        let shade = self.shade_surf().clone();
        s.bl(&shade, (0, 0));
    }

    /// Boden ab Reihe 0 abwaerts, mit Luecken ueber den Gruben.
    pub fn draw_ground(&mut self, s: &mut Surface, lv: Option<&Level>, ground: Color, cam_x: f64, cam_y: f64, pulse: f64) {
        let ts = self.ts;
        let (w, h) = (self.w, self.h);
        let y0 = self.sy(0.0, cam_y) as i32;
        if y0 >= h {
            return;
        }
        let dark = mix(ground, BLACK, 0.35);
        draw::rect(s, dark, (0, y0, w, h - y0), 0);
        let band = 2.max(ts / 2);
        for i in 0..band {
            draw::line(s, mix(ground, dark, i as f64 / band as f64), (0, y0 + i), (w, y0 + i), 1);
        }
        let tile = 4 * ts;
        let off = py::fpmod(cam_x * ts as f64, tile as f64) as i32;
        let fug = mix(dark, WHITE, 0.08);
        for i in -1..(w / tile + 2) {
            let x = i * tile - off;
            draw::line(s, fug, (x, y0 + band), (x, h), 1);
        }
        if let Some(lv) = lv.filter(|lv| !lv.pits.is_empty()) {
            let c0 = cam_x as i64 - 1;
            let c1 = (cam_x + w as f64 / ts as f64) as i64 + 1;
            for c in c0..=c1 {
                if lv.pits.contains(&c) {
                    let x = self.sx(c as f64, cam_x) as i32;
                    draw::rect(s, (4, 2, 8), (x, y0, ts + 1, h - y0), 0);
                    let m = 2.max(ts / 3);
                    for j in 0..m {
                        let a = 1.0 - j as f64 / m as f64;
                        draw::line(s, mix((4, 2, 8), (255, 60, 60), 0.35 * a), (x, h - 1 - j), (x + ts, h - 1 - j), 1);
                    }
                }
            }
        }
        let glow = mix(ground, WHITE, 0.55 + 0.35 * pulse);
        let lw = 2.max(ts / 16);
        draw::rect(s, glow, (0, y0 - lw / 2, w, lw), 0);
    }

    /// Decke des Korridors (Schiff/Ball/UFO/Welle).
    pub fn draw_ceiling(&mut self, s: &mut Surface, ground: Color, ceil_b: f64, cam_y: f64, pulse: f64, alpha: f64) {
        if ceil_b <= 0.0 || alpha <= 0.0 {
            return;
        }
        let ts = self.ts;
        let y1 = self.sy(ceil_b, cam_y) as i32;
        if y1 <= 0 {
            return;
        }
        let dark = mix(ground, BLACK, 0.35);
        if alpha >= 0.99 {
            draw::rect(s, dark, (0, 0, self.w, y1), 0);
        } else {
            let mut ov = surface_alpha(self.w, y1);
            ov.fill_c(pg::alpha(dark, (255.0 * alpha) as i32));
            s.bl(&ov, (0, 0));
        }
        let lw = 2.max(ts / 16);
        let glow = mix(ground, WHITE, 0.55 + 0.35 * pulse);
        let mut line = surface_alpha(self.w, lw);
        line.fill_c(pg::alpha(glow, (255.0 * alpha) as i32));
        s.bl(&line, (0, y1 - lw / 2));
    }

    /// Alle sichtbaren Objekte. `used`: ausgeloeste Objekte (Orbs/Muenzen).
    #[allow(clippy::too_many_arguments)]
    pub fn draw_objects(&mut self, s: &mut Surface, lv: &Level, cam_x: f64, cam_y: f64, t: f64, pulse: f64, used: &BTreeSet<usize>, editor: bool, masks: Option<&HashMap<usize, i64>>) {
        let ts = self.ts;
        let c0 = cam_x.floor() as i64 - 2;
        let c1 = (cam_x + self.w as f64 / ts as f64) as i64 + 2;
        let cols = lv.draw_cols.get_or_init(|| {
            let mut cols: HashMap<i64, Vec<usize>> = HashMap::new();
            for i in 0..lv.n {
                cols.entry(lv.ox[i]).or_default().push(i);
            }
            cols
        });
        let masks = masks.unwrap_or_else(|| block_masks(lv));
        let mut glows = Vec::new();
        let mut portals = Vec::new();
        for c in c0..=c1 {
            let Some(list) = cols.get(&c) else { continue };
            for &i in list {
                let name = core::KINDS[lv.kind[i]].0;
                let x = self.sx(lv.ox[i] as f64, cam_x) as i32;
                let y = self.sy((lv.oy[i] + 1) as f64, cam_y) as i32;
                if y > self.h + 3 * ts || y < -4 * ts {
                    continue;
                }
                let cls = lv.cls[i];
                if name == "block" {
                    let m = masks.get(&i).copied().unwrap_or(15);
                    let b = self.block(m, 0).clone();
                    s.bl(&b, (x, y));
                } else if name == "half" {
                    let b = self.block(15, lv.rot[i] + 1).clone();
                    s.bl(&b, (x, y));
                } else if cls == core::C_HAZARD {
                    let sp = self.spike(lv.rot[i], name == "spike_s").clone();
                    s.bl(&sp, (x, y));
                } else if cls == core::C_PAD {
                    self.pad(s, x, y, lv.val[i], lv.rot[i], t, used.contains(&i));
                } else if cls == core::C_ORB {
                    glows.push((i, x, y));
                } else if cls == core::C_MODE || cls == core::C_GRAV || cls == core::C_SPEED {
                    portals.push((i, x, y));
                } else if cls == core::C_COIN {
                    if used.contains(&i) {
                        continue;
                    }
                    let r = ts as f64 * 0.36;
                    self.coin(s, (x + ts / 2) as f64, (y + ts / 2) as f64, t, r);
                } else if editor && cls == core::C_TRIGGER {
                    self.trigger_icon(s, x, y, &lv.par[i]);
                } else if editor && cls == core::C_PIT {
                    let y0 = self.sy(0.0, cam_y) as i32;
                    draw::rect(s, (255, 70, 70), (x + 2, y0 + 2, ts - 4, 3.max(ts / 6)), 0);
                }
            }
        }
        for (i, x, y) in glows {
            self.orb(s, x, y, lv.val[i], pulse, used.contains(&i));
        }
        for (i, x, y) in portals {
            self.portal(s, x, y, lv.cls[i], lv.val[i], t);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn pad(&mut self, s: &mut Surface, x: i32, y: i32, which: i64, rot: i64, t: f64, used: bool) {
        let ts = self.ts;
        let col = COL_PAD[which as usize];
        let g = self.glow("pad", ts as f64 * 0.7, col, 0.9).clone();
        let cy = if rot == 0 { y + ts - ts / 8 } else { y + ts / 8 };
        s.bl(&g, (x + ts / 2 - g.w() / 2, cy - g.h() / 2));
        let w = (ts as f64 * 0.8) as i32;
        let h = 3.max(ts / 5);
        let r = rect(x + (ts - w) / 2, if rot == 0 { y + ts - h } else { y }, w, h);
        draw::ellipse(s, if used { tup(mix(col, WHITE, 0.5)) } else { col }, r.inflate(0, h), 0);
        draw::rect(s, (12, 12, 22), (r.x, if rot == 0 { y + ts - 2 } else { y }, r.w, 2), 0);
        for k in 0..3 {
            let ph = py::fpmod(t * 1.6 + k as f64 / 3.0, 1.0);
            let px = x as f64 + ts as f64 * (0.25 + 0.25 * k as f64);
            let dy = ph * ts as f64 * 0.9;
            let pyy = if rot == 0 { (y + ts - h) as f64 - dy } else { (y + h) as f64 + dy };
            let rr = 1.max((ts as f64 * 0.05 * (1.0 - ph)) as i32);
            draw::circle(s, mix(col, WHITE, 0.4), (px as i32, pyy as i32), rr, 0);
        }
    }

    fn orb(&mut self, s: &mut Surface, x: i32, y: i32, which: i64, pulse: f64, used: bool) {
        let ts = self.ts;
        let col = COL_PAD[which as usize];
        let (cx, cy) = (x + ts / 2, y + ts / 2);
        let g = self.glow("orb", ts as f64 * 0.95, col, 1.0).clone();
        s.bl(&g, (cx - g.w() / 2, cy - g.h() / 2));
        let r = (ts as f64 * 0.34) as i32;
        draw::circle(s, mix(col, WHITE, 0.25), (cx, cy), r, 0);
        draw::circle(s, WHITE, (cx, cy), r, 1.max(ts / 14));
        draw::circle(s, mix(col, WHITE, 0.7), (cx - py::fdiv32(r, 3), cy - py::fdiv32(r, 3)), 1.max(r / 3), 0);
        let ring = if used { r + (ts as f64 * 0.4) as i32 } else { r + (ts as f64 * (0.12 + 0.18 * pulse)) as i32 };
        draw::circle(s, col, (cx, cy), ring, 1.max(ts / 18));
    }

    fn portal(&mut self, s: &mut Surface, x: i32, y: i32, cls: i64, val: i64, t: f64) {
        let ts = self.ts;
        let tsf = ts as f64;
        let cx = x + ts / 2;
        let cy = y + ts / 2;
        if cls == core::C_SPEED {
            let col = COL_SPEED[val as usize];
            let g = self.glow("spd", tsf * 1.2, col, 0.7).clone();
            s.bl(&g, (cx - g.w() / 2, cy - g.h() / 2));
            let n = val + 1;
            let wv = tsf * 0.34;
            for i in 0..n {
                let ox = cx as f64 + (i as f64 - (n - 1) as f64 / 2.0) * wv * 0.8;
                let cyf = cy as f64;
                let pts = [(ox - wv * 0.5, cyf - tsf * 0.55), (ox + wv * 0.4, cyf), (ox - wv * 0.5, cyf + tsf * 0.55), (ox - wv * 0.05, cyf)];
                draw::polygon(s, col, &pts, 0);
                draw::polygon(s, WHITE, &pts, 1.max(ts / 20));
            }
            return;
        }
        let col = if cls == core::C_MODE { COL_MODE[val as usize] } else { col_grav(val) };
        let g = self.glow("prt", tsf * 1.6, col, 0.8).clone();
        s.bl(&g, (cx - g.w() / 2, cy - g.h() / 2));
        let rc = rect(0, 0, (tsf * 0.9) as i32, (tsf * 3.0) as i32).center_at((cx, cy));
        let wob = 0.08 * (t * 4.0).sin();
        let inner = rc.inflate(-((tsf * (0.35 + wob)) as i32), -((tsf * 0.5) as i32));
        draw::ellipse(s, (10, 10, 20), rc, 0);
        draw::ellipse(s, mix(col, BLACK, 0.45), inner, 0);
        draw::ellipse(s, col, rc, 2.max(ts / 8));
        let d5 = 2.max(ts / 5);
        draw::ellipse(s, WHITE, rc.inflate(-d5, -d5), 1.max(ts / 20));
        let r = 3.max((tsf * 0.18) as i32);
        if cls == core::C_GRAV {
            let d = if val > 0 { -1.0 } else { 1.0 };
            let (cxf, cyf, rf) = (cx as f64, cy as f64, r as f64);
            draw::polygon(s, WHITE, &[(cxf - rf, cyf - d * rf * 0.4), (cxf + rf, cyf - d * rf * 0.4), (cxf, cyf + d * rf)], 0);
        } else {
            mode_glyph(s, cx, cy, r, val);
        }
    }

    fn coin(&mut self, s: &mut Surface, cx: f64, cy: f64, t: f64, r: f64) {
        let g = self.glow("coin", self.ts as f64 * 0.9, COL_COIN, 0.8).clone();
        s.bl(&g, (cx as i32 - g.w() / 2, cy as i32 - g.h() / 2));
        draw_coin(s, cx, cy, r, t, false);
    }

    fn trigger_icon(&mut self, s: &mut Surface, x: i32, y: i32, par: &[i64]) {
        let ts = self.ts;
        let col = rgb(par[1], par[2], par[3]);
        let r = rect(x + ts / 6, y + ts / 6, ts - ts / 3, ts - ts / 3);
        draw::rect_r(s, col, r, 0, 2.max(ts / 6));
        draw::rect_r(s, WHITE, r, 1.max(ts / 16), 2.max(ts / 6));
        let f = ui::font(9.max(ts / 3), true, false);
        let img = f.render(if par[0] == 0 { "BG" } else { "G" }, true, WHITE);
        let ir = img.get_rect().center_at(r.center());
        s.blr(&img, ir);
    }

    /// Spieler an Welt-Position (x, y = linke untere Ecke, in Bloecken).
    #[allow(clippy::too_many_arguments)]
    pub fn draw_player(&mut self, s: &mut Surface, x: f64, y: f64, mode: i64, grav: i64, angle: f64, cam_x: f64, cam_y: f64, trail: Option<&[(f64, f64)]>, alpha: i32) {
        let ts = self.ts;
        let cx = self.sx(x + 0.5, cam_x);
        let cy = self.sy(y + 0.5, cam_y);
        if let Some(trail) = trail.filter(|t| t.len() > 1) {
            let pts: Vec<(f64, f64)> = trail.iter().map(|&(px, pyy)| (self.sx(px + 0.5, cam_x), self.sy(pyy + 0.5, cam_y))).collect();
            if mode == core::WAVE {
                let w = 2.max(ts / 6);
                draw::lines(s, mix(COL_P2, WHITE, 0.2), false, &pts, w + 4);
                draw::lines(s, WHITE, false, &pts, 1.max(w / 2));
            } else {
                let n = pts.len();
                for (i, &(px, pyy)) in pts[..n - 1].iter().enumerate() {
                    let f = i as f64 / n as f64;
                    let rr = 1.max((ts as f64 * 0.14 * f) as i32);
                    draw::circle(s, mix(COL_P2, WHITE, f), (px as i32, pyy as i32), rr, 0);
                }
            }
        }
        let ia = angle as i64;
        let key = (mode, if mode != core::WAVE { py::fdiv(ia, 3) * 3 } else { ia }, grav);
        if !self.players.contains_key(&key) {
            let surf = self.player_surface(mode, angle, grav);
            if self.cache_len() > 900 {
                self.players.clear();
            }
            self.players.insert(key, surf);
        }
        let surf = &self.players[&key];
        if alpha < 255 {
            let mut c = surf.copy();
            c.alpha_set(alpha);
            let r = c.get_rect().center_at((cx as i32, cy as i32));
            s.blr(&c, r);
        } else {
            let r = surf.get_rect().center_at((cx as i32, cy as i32));
            s.blr(surf, r);
        }
    }

    fn player_surface(&self, mode: i64, angle: f64, grav: i64) -> Surface {
        let ts = self.ts;
        let tsf = ts as f64;
        let big = ts * 2;
        let mut surf = surface_alpha(big, big);
        let c = big / 2;
        let cf = c as f64;
        let lw = 1.max(ts / 14);
        let dark = (14, 14, 24);
        if mode == core::CUBE {
            cube_face(&mut surf, c, c, (tsf * 0.96) as i32);
            surf = transform::rotate(&surf, -angle);
        } else if mode == core::BALL {
            let r = (tsf * 0.48) as i32;
            draw::circle(&mut surf, dark, (c, c), r, 0);
            draw::circle(&mut surf, COL_P1, (c, c), r - lw, 0);
            for k in 0..4 {
                let a = py::radians((k * 90) as f64);
                draw::line(&mut surf, dark, (c, c), (cf + a.cos() * r as f64, cf + a.sin() * r as f64), lw + 1);
            }
            draw::circle(&mut surf, COL_P2, (c, c), (r as f64 * 0.45) as i32, 0);
            draw::circle(&mut surf, dark, (c, c), (r as f64 * 0.45) as i32, lw);
            draw::circle(&mut surf, dark, (c, c), r, lw);
            surf = transform::rotate(&surf, -angle);
        } else if mode == core::SHIP {
            cube_face(&mut surf, c + (tsf * 0.02) as i32, c - (tsf * 0.22) as i32, (tsf * 0.5) as i32);
            let hull = [(cf - tsf * 0.55, cf - tsf * 0.02), (cf + tsf * 0.62, cf + tsf * 0.12), (cf + tsf * 0.3, cf + tsf * 0.42), (cf - tsf * 0.5, cf + tsf * 0.42)];
            draw::polygon(&mut surf, COL_P2, &hull, 0);
            draw::polygon(&mut surf, dark, &hull, lw + 1);
            draw::line(&mut surf, COL_P1, (cf - tsf * 0.4, cf + tsf * 0.2), (cf + tsf * 0.35, cf + tsf * 0.24), lw + 1);
            if grav < 0 {
                surf = transform::flip(&surf, false, true);
            }
            surf = transform::rotate(&surf, angle);
        } else if mode == core::UFO {
            let dome = rect(0, 0, (tsf * 0.7) as i32, (tsf * 0.62) as i32).midbottom_at((c, c + (tsf * 0.08) as i32));
            draw::ellipse(&mut surf, rgba(200, 240, 255, 90), dome, 0);
            cube_face(&mut surf, c, c - (tsf * 0.14) as i32, (tsf * 0.42) as i32);
            draw::ellipse(&mut surf, dark, dome, lw);
            let saucer = rect(0, 0, (tsf * 1.1) as i32, (tsf * 0.36) as i32).center_at((c, c + (tsf * 0.18) as i32));
            draw::ellipse(&mut surf, COL_P2, saucer, 0);
            draw::ellipse(&mut surf, dark, saucer, lw + 1);
            for k in [-1.0, 0.0, 1.0] {
                draw::circle(&mut surf, COL_P1, (c + (k * tsf * 0.28) as i32, saucer.centery()), 1.max(ts / 16), 0);
            }
            if grav < 0 {
                surf = transform::flip(&surf, false, true);
            }
        } else {
            let pts = [(cf + tsf * 0.42, cf), (cf - tsf * 0.34, cf - tsf * 0.3), (cf - tsf * 0.16, cf), (cf - tsf * 0.34, cf + tsf * 0.3)];
            draw::polygon(&mut surf, COL_P1, &pts, 0);
            draw::polygon(&mut surf, dark, &pts, lw + 1);
            draw::circle(&mut surf, COL_P2, ((cf - tsf * 0.02) as i32, c), 2.max(ts / 9), 0);
            surf = transform::rotate(&surf, angle);
        }
        surf
    }
}

fn mode_glyph(s: &mut Surface, cx: i32, cy: i32, r: i32, mode: i64) {
    let (cxf, cyf, rf) = (cx as f64, cy as f64, r as f64);
    let w = 1.max(r / 3);
    match mode {
        core::CUBE => {
            draw::rect(s, WHITE, (cx - r, cy - r, 2 * r, 2 * r), w);
        }
        core::SHIP => {
            draw::polygon(s, WHITE, &[(cxf - rf, cyf + rf * 0.6), (cxf + rf, cyf + rf * 0.2), (cxf - rf * 0.4, cyf - rf * 0.8)], w);
        }
        core::BALL => {
            draw::circle(s, WHITE, (cx, cy), r, w);
        }
        core::UFO => {
            draw::ellipse(s, WHITE, (cxf - rf, cyf - rf * 0.2, 2.0 * rf, rf), w);
            draw::arc(s, WHITE, (cxf - rf * 0.5, cyf - rf * 0.8, rf, rf * 1.2), 0.0, PI, w);
        }
        _ => {
            draw::lines(s, WHITE, false, &[(cxf - rf, cyf + rf * 0.5), (cxf - rf * 0.3, cyf - rf * 0.5), (cxf + rf * 0.3, cyf + rf * 0.5), (cxf + rf, cyf - rf * 0.5)], w);
        }
    }
}

/// Das Lama-Wuerfelgesicht (Grundfarbe aussen, Zweitfarbe innen).
pub fn cube_face(surf: &mut Surface, cx: i32, cy: i32, size: i32) {
    let dark = (14, 14, 24);
    let lw = 1.max(size / 14);
    let r = rect(0, 0, size, size).center_at((cx, cy));
    draw::rect(surf, COL_P1, r, 0);
    let d = py::fdiv32(-size, 3);
    let inner = r.inflate(d, d);
    draw::rect(surf, COL_P2, inner, 0);
    draw::rect(surf, dark, inner, lw);
    let eye = 2.max(size / 7);
    draw::rect(surf, dark, (inner.x + inner.w / 4 - eye / 2, inner.y + inner.h / 3 - eye / 2, eye, eye), 0);
    draw::rect(surf, dark, (inner.right() - inner.w / 4 - eye / 2, inner.y + inner.h / 3 - eye / 2, eye, eye), 0);
    draw::rect(surf, dark, (inner.x + inner.w / 4, inner.bottom() - inner.h / 3, inner.w / 2, 1.max(lw)), 0);
    let ear = 2.max(size / 6);
    draw::rect(surf, COL_P2, (r.x + size / 8, r.y + 2, ear, ear), 0);
    draw::rect(surf, COL_P2, (r.right() - size / 8 - ear, r.y + 2, ear, ear), 0);
    draw::rect(surf, dark, r, lw + 1);
}

/// `block_masks(lv)` - Kanten-Masken aller Bloecke.
pub fn block_masks(lv: &Level) -> &HashMap<usize, i64> {
    lv.masks.get_or_init(|| {
        let mut cells = HashSet::new();
        for i in 0..lv.n {
            if core::KINDS[lv.kind[i]].0 == "block" {
                cells.insert((lv.ox[i], lv.oy[i]));
            }
        }
        let mut masks = HashMap::new();
        for i in 0..lv.n {
            if core::KINDS[lv.kind[i]].0 != "block" {
                continue;
            }
            let (x, y) = (lv.ox[i], lv.oy[i]);
            let mut m = 0;
            if !cells.contains(&(x, y + 1)) {
                m |= 1;
            }
            if !cells.contains(&(x + 1, y)) {
                m |= 2;
            }
            if !cells.contains(&(x, y - 1)) && y > 0 {
                m |= 4;
            }
            if !cells.contains(&(x - 1, y)) {
                m |= 8;
            }
            masks.insert(i, m);
        }
        masks
    })
}

/// Goldmuenze mit Dreh-Glanz (auch fuer HUD/Level-Auswahl).
pub fn draw_coin(s: &mut Surface, cx: f64, cy: f64, r: f64, t: f64, dim: bool) {
    let squash = (t * 2.4).cos().abs();
    let w = 2.max((2.0 * r * (0.35 + 0.65 * squash)) as i32);
    let h = 2.max((2.0 * r) as i32);
    let rc = rect(0, 0, w, h).center_at((cx as i32, cy as i32));
    let base = if dim { (120, 110, 90) } else { COL_COIN };
    draw::ellipse(s, mix(base, BLACK, 0.35), rc.move_(1.max(py::ffdiv(r, 6.0) as i32), 0), 0);
    draw::ellipse(s, base, rc, 0);
    draw::ellipse(s, mix(base, WHITE, 0.45), rc.inflate(-(2.max(w / 3)), -(2.max(h / 3))), 1.max(py::ffdiv(r, 5.0) as i32));
    draw::ellipse(s, (60, 40, 10), rc, 1.max(py::ffdiv(r, 6.0) as i32));
}

/// Stern.
pub fn draw_star(s: &mut Surface, cx: f64, cy: f64, r: f64, col: impl Col, outline: impl Col) {
    let mut pts = Vec::with_capacity(10);
    for k in 0..10 {
        let a = -PI / 2.0 + k as f64 * PI / 5.0;
        let rr = if k % 2 == 0 { r } else { r * 0.45 };
        pts.push((cx + a.cos() * rr, cy + a.sin() * rr));
    }
    draw::polygon(s, col, &pts, 0);
    draw::polygon(s, outline, &pts, 1.max(py::ffdiv(r, 6.0) as i32));
}

/// Schwierigkeits-Gesicht (0 leicht .. 5 Daemon).
pub fn draw_face(s: &mut Surface, cx: f64, cy: f64, r: f64, diff: i64) {
    let col = DIFF_COLORS[diff.clamp(0, 5) as usize];
    let dark = (18, 14, 24);
    let lw = 1.max(py::ffdiv(r, 8.0) as i32);
    let (cx, cy, r) = (cx as i32, cy as i32, r as i32);
    let (cxf, cyf, rf) = (cx as f64, cy as f64, r as f64);
    if diff >= 5 {
        for sgn in [-1.0, 1.0] {
            let pts = [(cxf + sgn * rf * 0.55, cyf - rf * 0.55), (cxf + sgn * rf * 1.05, cyf - rf * 1.25), (cxf + sgn * rf * 0.2, cyf - rf * 0.85)];
            draw::polygon(s, (240, 60, 40), &pts, 0);
            draw::polygon(s, dark, &pts, lw);
        }
    }
    draw::circle(s, col, (cx, cy), r, 0);
    draw::circle(s, mix(col, WHITE, 0.35), (cx - r / 3, cy - r / 3), 1.max(r / 4), 0);
    draw::circle(s, dark, (cx, cy), r, lw + 1);
    let ey = cy - r / 5;
    let ex = (rf * 0.38) as i32;
    let er = 1.max(r / 6);
    if diff <= 1 {
        draw::circle(s, dark, (cx - ex, ey), er, 0);
        draw::circle(s, dark, (cx + ex, ey), er, 0);
        draw::arc(s, dark, (cx - r / 2, cy - r / 4, r, (rf * 0.8) as i32), PI * 1.1, PI * 1.9, lw + 1);
    } else if diff == 2 {
        draw::circle(s, dark, (cx - ex, ey), er, 0);
        draw::circle(s, dark, (cx + ex, ey), er, 0);
        draw::line(s, dark, (cx - r / 3, cy + r / 3), (cx + r / 3, cy + r / 3), lw + 1);
    } else if diff == 3 {
        for sgn in [-1, 1] {
            draw::line(s, dark, (cx + sgn * ex - er * 2, ey + er * 2 * sgn), (cx + sgn * ex + er * 2, ey - er * 2 * sgn), lw + 1);
            draw::circle(s, dark, (cx + sgn * ex, ey + er), er, 0);
        }
        draw::arc(s, dark, (cx - r / 3, cy + r / 5, (rf * 0.66) as i32, (rf * 0.6) as i32), 0.15, PI - 0.15, lw + 1);
    } else {
        for sgn in [-1, 1] {
            let c = if diff == 5 { WHITE } else { dark };
            draw::polygon(s, c, &[(cx + sgn * ex - er * 2, ey - er), (cx + sgn * ex + er * 2, ey - er), (cx + sgn * (ex - er * 2 * sgn), ey + er * 2)], 0);
            draw::line(s, dark, (cx + sgn * (ex + er * 3), ey - er * 3), (cx + sgn * (ex - er * 2), ey - er), lw + 1);
        }
        let mouth = rect(cx - r / 2, cy + r / 5, r, 3.max(r / 3));
        draw::rect_r(s, dark, mouth, 0, 1.max(r / 8));
        for k in 1..4 {
            let x = mouth.x + mouth.w * k / 4;
            draw::line(s, WHITE, (x, mouth.y), (x, mouth.bottom() - 1), 1.max(lw / 2));
        }
    }
}
