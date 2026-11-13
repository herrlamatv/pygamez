//! minigolf_draw.rs
//! ================
//! Zeichnen des Minigolf-Platzes (Spiel UND Bahn-Editor) - uebertragen aus
//! `games/minigolf_draw.py`.

use std::f64::consts::PI;

use pgz_engine::prelude::*;

use crate::minigolf_gen::{Hole, BORDER, CH, CW};

type Rgb = (i32, i32, i32);

/// Gruen.
pub const COL_GREEN: Rgb = (46, 132, 74);
const COL_GREEN_D: Rgb = (38, 112, 62);
const COL_FRINGE: Rgb = (62, 152, 88);
/// Bande.
pub const COL_WALL: Rgb = (122, 84, 52);
/// Bande hell.
pub const COL_WALL_HI: Rgb = (156, 112, 72);
/// Sand.
pub const COL_SAND: Rgb = (214, 190, 132);
const COL_SAND_D: Rgb = (190, 164, 108);
/// Wasser.
pub const COL_WATER: Rgb = (52, 122, 196);
const COL_WATER_D: Rgb = (36, 92, 158);
/// Rampe.
pub const COL_SLOPE: Rgb = (60, 150, 92);
/// Gummipuffer.
pub const COL_BUMPER: Rgb = (222, 84, 108);
const COL_BUMPER_HI: Rgb = (246, 140, 160);
/// Muehle.
pub const COL_MILL: Rgb = (186, 190, 198);
const COL_MILL_D: Rgb = (128, 134, 146);
/// Ball.
pub const COL_BALL: Rgb = (248, 248, 244);
const COL_CUP: Rgb = (16, 22, 18);
const COL_FLAG: Rgb = (226, 72, 72);
/// Ziellinie.
pub const COL_AIM: Rgb = (245, 245, 210);
/// Staerke-Sperre.
pub const COL_LOCK: Rgb = (248, 208, 96);
/// Rohr.
pub const COL_TUNNEL: Rgb = (156, 108, 210);
const COL_TUNNEL_D: Rgb = (108, 68, 156);
/// Eis.
pub const COL_ICE: Rgb = (176, 222, 240);
const COL_ICE_D: Rgb = (132, 186, 214);
/// Schub.
pub const COL_BOOST: Rgb = (250, 176, 60);
const COL_BOOST_D: Rgb = (206, 132, 26);
/// Magnet (zieht).
pub const COL_MAGNET: Rgb = (236, 96, 96);
const COL_MAGNET_R: Rgb = (96, 148, 236);
/// Einbahn-Tor.
pub const COL_GATE: Rgb = (120, 200, 150);
const COL_GATE_D: Rgb = (70, 150, 104);
/// Klebefeld.
pub const COL_STICKY: Rgb = (128, 104, 66);
const COL_STICKY_D: Rgb = (96, 76, 46);
/// Drehscheibe.
pub const COL_SPIN: Rgb = (214, 200, 120);
const COL_SPIN_D: Rgb = (168, 152, 82);
/// Sprungrampe.
pub const COL_JUMP: Rgb = (240, 232, 120);
const COL_JUMP_D: Rgb = (196, 186, 62);

/// Zeichenreihenfolge: Flaechen zuerst, Aufbauten zuletzt.
pub const DRAW_ORDER: [&str; 15] = [
    "slopes", "ice", "sticky", "sand", "boosters", "water", "magnets", "spinners", "tunnels", "walls", "gates", "movers",
    "bumpers", "jumps", "mills",
];

/// Nullpunkt und Massstab: Bahn-Einheiten <-> Bildschirm-Pixel.
#[derive(Clone, Copy, Debug)]
pub struct View {
    /// Nullpunkt x.
    pub ox: f64,
    /// Nullpunkt y.
    pub oy: f64,
    /// Massstab.
    pub scale: f64,
}

impl View {
    /// `project(x, y)`
    pub fn project(&self, x: f64, y: f64) -> (f64, f64) {
        (self.ox + x * self.scale, self.oy + y * self.scale)
    }

    /// `unproject(sx, sy)`
    pub fn unproject(&self, sx: f64, sy: f64) -> (f64, f64) {
        ((sx - self.ox) / self.scale, (sy - self.oy) / self.scale)
    }

    /// `rect_px(r)`
    pub fn rect_px(&self, r: &[f64]) -> Rect {
        let (px, py_) = self.project(r[0], r[1]);
        Rect::new(px as i32, py_ as i32, 1.max((r[2] * self.scale) as i32), 1.max((r[3] * self.scale) as i32))
    }

    /// `board(cw, ch)`
    pub fn board(&self, cw: f64, ch: f64) -> Rect {
        Rect::new(self.ox as i32, self.oy as i32, (cw * self.scale) as i32, (ch * self.scale) as i32)
    }

    /// `board()` mit Standardmass.
    pub fn board_std(&self) -> Rect {
        self.board(CW, CH)
    }
}

fn tsec() -> f64 {
    pg::ticks() as f64 / 1000.0
}

/// Python-`range(start, stop, step)` fuer positive Schritte.
fn prange(start: i32, stop: i32, step: i32) -> impl Iterator<Item = i32> {
    let step = step.max(1);
    (0..).map(move |i| start + i * step).take_while(move |v| *v < stop)
}

/// `draw_ground(s, hole, view)` - Gruen, Maehstreifen, Banden.
pub fn draw_ground(s: &mut Surface, hole: &Hole, view: &View) -> Rect {
    let board = view.board(hole.w, hole.h);
    draw::rect_r(s, COL_FRINGE, board.inflate(10, 10), 0, 10);
    draw::rect_r(s, COL_GREEN, board, 0, 6);
    let stripe = 6.max((10.0 * view.scale) as i32);
    for i in prange(0, board.h, stripe * 2) {
        draw::rect(s, COL_GREEN_D, (board.x, board.y + i, board.w, stripe.min(board.h - i)), 0);
    }
    let b = 2.max((BORDER * view.scale) as i32);
    for r in [
        Rect::new(board.x, board.y, board.w, b),
        Rect::new(board.x, board.bottom() - b, board.w, b),
        Rect::new(board.x, board.y, b, board.h),
        Rect::new(board.right() - b, board.y, b, board.h),
    ] {
        draw::rect(s, COL_WALL, r, 0);
        draw::rect(s, COL_WALL_HI, r, 1);
    }
    board
}

fn draw_slope(s: &mut Surface, r: &[f64], ax: f64, ay: f64, view: &View) {
    let rc = view.rect_px(r);
    draw::rect_r(s, COL_SLOPE, rc, 0, 5);
    let ang = ay.atan2(ax);
    let step = 16.max((14.0 * view.scale) as i32);
    let col = ui::mix(COL_SLOPE, (255, 255, 255), 0.22);
    let (dx, dy) = (ang.cos() * 6.0, ang.sin() * 6.0);
    for yy in prange(rc.y + step / 2, rc.bottom() - 2, step) {
        for xx in prange(rc.x + step / 2, rc.right() - 2, step) {
            let (xf, yf) = (xx as f64, yy as f64);
            draw::line(s, col, (xf - dx, yf - dy), (xf + dx, yf + dy), 2);
            draw::circle(s, col, ((xf + dx) as i32, (yf + dy) as i32), 2, 0);
        }
    }
}

fn draw_sand(s: &mut Surface, r: &[f64], view: &View) {
    let rc = view.rect_px(r);
    draw::rect_r(s, COL_SAND, rc, 0, 6);
    draw::rect_r(s, COL_SAND_D, rc, 2, 6);
}

fn draw_water(s: &mut Surface, r: &[f64], view: &View) {
    let rc = view.rect_px(r);
    draw::rect_r(s, COL_WATER, rc, 0, 7);
    let t = tsec();
    for i in 0..3 {
        let yy = rc.y + (rc.h as f64 * (0.25 + 0.25 * i as f64)) as i32 + ((t * 1.4 + i as f64).sin() * 3.0) as i32;
        if rc.y < yy && yy < rc.bottom() {
            draw::line(s, COL_WATER_D, (rc.x + 6, yy), (rc.right() - 6, yy), 2);
        }
    }
    draw::rect_r(s, COL_WATER_D, rc, 2, 7);
}

/// `draw_wall(s, rc, mover)`
pub fn draw_wall(s: &mut Surface, rc: Rect, mover: bool) {
    draw::rect_r(s, COL_WALL, rc, 0, 3);
    draw::rect_r(s, COL_WALL_HI, rc, 2, 3);
    if mover {
        draw::line(s, COL_WALL_HI, (rc.x + 4, rc.centery()), (rc.right() - 4, rc.centery()), 1);
    }
}

fn draw_bumper(s: &mut Surface, b: &[f64], view: &View) {
    let (px, py_) = view.project(b[0], b[1]);
    let rr = 3.max((b[2] * view.scale) as i32);
    draw::circle(s, COL_BUMPER, (px as i32, py_ as i32), rr, 0);
    draw::circle(s, COL_BUMPER_HI, (px as i32, py_ as i32), 2.max(rr - 3), 2);
}

fn draw_mill(s: &mut Surface, mill: &[f64], view: &View, mill_a: f64) {
    let (x, y, length, arms, speed) = (mill[0], mill[1], mill[2], mill[3], mill[4]);
    let (px, py_) = view.project(x, y);
    let base = mill_a * speed;
    let w = 3.max((3.0 * view.scale) as i32);
    let n = 1.max(arms as i64);
    for i in 0..n {
        let a = base + i as f64 * (2.0 * PI / n as f64);
        let (ex, ey) = view.project(x + a.cos() * length, y + a.sin() * length);
        draw::line(s, COL_MILL, (px, py_), (ex, ey), w);
        draw::line(s, COL_MILL_D, (px, py_), (ex, ey), 1);
    }
    draw::circle(s, COL_MILL_D, (px as i32, py_ as i32), 3.max((2.4 * view.scale) as i32), 0);
}

fn draw_tunnel(s: &mut Surface, tun: &[f64], view: &View) {
    let (ax, ay) = view.project(tun[0], tun[1]);
    let (bx, by) = view.project(tun[2], tun[3]);
    let rr = 4.max((tun[4] * view.scale) as i32);
    let (seg, gap) = (9.0, 7.0);
    let (dx, dy) = (bx - ax, by - ay);
    let dist = py::hypot(dx, dy);
    if dist > 1.0 {
        let (ux, uy) = (dx / dist, dy / dist);
        let mut pos = rr as f64;
        while pos < dist - rr as f64 {
            let e = (pos + seg).min(dist - rr as f64);
            draw::line(s, COL_TUNNEL_D, (ax + ux * pos, ay + uy * pos), (ax + ux * e, ay + uy * e), 2);
            pos = e + gap;
        }
    }
    let puls = ui::pulse(2.2, 0.55, 1.0);
    for (px, py_) in [(ax, ay), (bx, by)] {
        draw::circle(s, COL_TUNNEL_D, (px as i32, py_ as i32), rr, 0);
        draw::circle(s, ui::mix(COL_TUNNEL, (255, 255, 255), 0.25 * puls), (px as i32, py_ as i32), 2.max(rr - 3), 0);
        draw::circle(s, COL_TUNNEL_D, (px as i32, py_ as i32), 1.max(rr / 3), 0);
    }
}

fn draw_ice(s: &mut Surface, r: &[f64], view: &View) {
    let rc = view.rect_px(r);
    draw::rect_r(s, COL_ICE, rc, 0, 6);
    for i in 0..3 {
        let yy = rc.y + py::fdiv32(rc.h * (i + 1), 4);
        draw::line(s, COL_ICE_D, (rc.x + 5, yy), (rc.right() - 5, yy - py::fdiv32(rc.h, 8)), 1);
    }
    draw::rect_r(s, COL_ICE_D, rc, 2, 6);
}

fn draw_booster(s: &mut Surface, bo: &[f64], view: &View) {
    let rc = view.rect_px(&bo[..4]);
    draw::rect_r(s, COL_BOOST_D, rc, 0, 5);
    draw::rect_r(s, COL_BOOST, rc.inflate(-4, -4), 0, 4);
    let ang = bo[5].atan2(bo[4]);
    let (ux, uy) = (ang.cos(), ang.sin());
    let step = 14.max((12.0 * view.scale) as i32);
    for k in 0..2 {
        let off = (k as f64 - 0.5) * step as f64 * 0.7;
        for yy in prange(rc.y + step / 2, rc.bottom() - 2, step) {
            for xx in prange(rc.x + step / 2, rc.right() - 2, step) {
                let (cx, cy) = (xx as f64 - uy * off, yy as f64 + ux * off);
                let (tipx, tipy) = (cx + ux * 6.0, cy + uy * 6.0);
                draw::polygon(
                    s,
                    COL_BOOST_D,
                    &[(tipx, tipy), (cx - ux * 4.0 - uy * 4.0, cy - uy * 4.0 + ux * 4.0), (cx - ux * 4.0 + uy * 4.0, cy - uy * 4.0 - ux * 4.0)],
                    0,
                );
            }
        }
    }
}

fn draw_magnet(s: &mut Surface, mag: &[f64], view: &View) {
    let (px, py_) = view.project(mag[0], mag[1]);
    let rr = 5.max((mag[2] * view.scale) as i32);
    let col = if mag[3] >= 0.0 { COL_MAGNET } else { COL_MAGNET_R };
    let puls = ui::pulse(1.6, 0.35, 0.9);
    for i in 0..3 {
        let rad = (rr as f64 * (0.4 + 0.3 * i as f64)) as i32;
        let mut ring = pg::surface_alpha(rad * 2 + 4, rad * 2 + 4);
        let a = (150.0 * puls * (1.0 - i as f64 * 0.25)) as i32;
        draw::circle(&mut ring, rgba(col.0, col.1, col.2, a), (rad + 2, rad + 2), rad, 2);
        s.bl(&ring, (px - rad as f64 - 2.0, py_ - rad as f64 - 2.0));
    }
    draw::circle(s, col, (px as i32, py_ as i32), 3.max(rr / 4), 0);
}

fn draw_gate(s: &mut Surface, gt: &[f64], view: &View) {
    let rc = view.rect_px(&gt[..4]);
    let mut face = pg::surface_alpha(rc.w, rc.h);
    face.fill_c(rgba(COL_GATE.0, COL_GATE.1, COL_GATE.2, 120));
    s.bl(&face, (rc.x, rc.y));
    draw::rect_r(s, COL_GATE_D, rc, 2, 3);
    let ang = gt[5].atan2(gt[4]);
    let (ux, uy) = (ang.cos(), ang.sin());
    let (cx, cy) = (rc.centerx() as f64, rc.centery() as f64);
    let ln = 6.max((rc.w.min(rc.h) as f64 * 0.35) as i32) as f64;
    let tip = (cx + ux * ln, cy + uy * ln);
    draw::line(s, COL_GATE_D, (cx - ux * ln, cy - uy * ln), tip, 2);
    draw::polygon(s, COL_GATE_D, &[tip, (tip.0 - ux * 6.0 - uy * 5.0, tip.1 - uy * 6.0 + ux * 5.0), (tip.0 - ux * 6.0 + uy * 5.0, tip.1 - uy * 6.0 - ux * 5.0)], 0);
}

fn draw_sticky(s: &mut Surface, r: &[f64], view: &View) {
    let rc = view.rect_px(r);
    draw::rect_r(s, COL_STICKY, rc, 0, 6);
    let step = 12.max((11.0 * view.scale) as i32);
    for yy in prange(rc.y + step / 2, rc.bottom() - 2, step) {
        for xx in prange(rc.x + step / 2, rc.right() - 2, step) {
            draw::circle(s, COL_STICKY_D, (xx, yy), 3, 0);
        }
    }
    draw::rect_r(s, COL_STICKY_D, rc, 2, 6);
}

fn draw_spinner(s: &mut Surface, sp: &[f64], view: &View, mill_a: f64) {
    let (px, py_) = view.project(sp[0], sp[1]);
    let rr = 5.max((sp[2] * view.scale) as i32);
    draw::circle(s, COL_SPIN_D, (px as i32, py_ as i32), rr, 0);
    draw::circle(s, COL_SPIN, (px as i32, py_ as i32), 2.max(rr - 3), 0);
    let base = mill_a * sp[3];
    for i in 0..4 {
        let a = base + i as f64 * (PI / 2.0);
        draw::line(s, COL_SPIN_D, (px, py_), (px + a.cos() * rr as f64, py_ + a.sin() * rr as f64), 2);
    }
    draw::circle(s, COL_SPIN_D, (px as i32, py_ as i32), 2.max(rr / 5), 0);
}

fn draw_jump(s: &mut Surface, jp: &[f64], view: &View) {
    let rc = view.rect_px(&jp[..4]);
    draw::rect_r(s, COL_JUMP_D, rc, 0, 4);
    draw::rect_r(s, COL_JUMP, rc.inflate(-4, -4), 0, 3);
    let ang = jp[5].atan2(jp[4]);
    let (ux, uy) = (ang.cos(), ang.sin());
    let (cx, cy) = (rc.centerx() as f64, rc.centery() as f64);
    let lni = 5.max((rc.w.min(rc.h) as f64 * 0.42) as i32);
    let ln = lni as f64;
    for k in [-1, 0, 1] {
        let off = (k * 4.max(lni / 2)) as f64;
        draw::line(s, COL_JUMP_D, (cx - uy * ln + ux * off, cy + ux * ln + uy * off), (cx + uy * ln + ux * off, cy - ux * ln + uy * off), 2);
    }
    let tip = (cx + ux * ln, cy + uy * ln);
    draw::polygon(s, COL_JUMP_D, &[tip, (tip.0 - ux * 7.0 - uy * 5.0, tip.1 - uy * 7.0 + ux * 5.0), (tip.0 - ux * 7.0 + uy * 5.0, tip.1 - uy * 7.0 - ux * 5.0)], 0);
}

/// `draw_cup(s, cup, view, cup_r)` - Loch mit Fahne.
pub fn draw_cup(s: &mut Surface, cup: (f64, f64), view: &View, cup_r: f64) {
    let (px, py_) = view.project(cup.0, cup.1);
    let r = 3.max((cup_r * view.scale) as i32);
    draw::circle(s, (24, 60, 36), (px as i32, py_ as i32), r + 2, 0);
    draw::circle(s, COL_CUP, (px as i32, py_ as i32), r, 0);
    let top = py_ - 16.max((18.0 * view.scale) as i32) as f64;
    draw::line(s, (238, 238, 232), (px, py_), (px, top), 2);
    let wave = (pg::ticks() as f64 / 260.0).sin() * 2.0;
    draw::polygon(s, COL_FLAG, &[(px, top), (px + 14.0, top + 5.0 + wave), (px, top + 11.0)], 0);
}

/// `draw_tee(s, tee, view)` - Abschlagpunkt.
pub fn draw_tee(s: &mut Surface, tee: (f64, f64), view: &View) {
    let (px, py_) = view.project(tee.0, tee.1);
    let r = 3.max((2.6 * view.scale) as i32);
    draw::circle(s, ui::mix(COL_GREEN, (255, 255, 255), 0.45), (px as i32, py_ as i32), r, 2);
    draw::circle(s, ui::mix(COL_GREEN, (255, 255, 255), 0.7), (px as i32, py_ as i32), 1.max(r / 3), 0);
}

/// `mover_rect(m, move_t)` - Rechteck eines Wanderblocks + Geschwindigkeit.
pub fn mover_rect(m: &[f64], move_t: f64) -> ([f64; 4], (f64, f64)) {
    let (x, y, w, h, dx, dy, speed) = (m[0], m[1], m[2], m[3], m[4], m[5], m[6]);
    if speed <= 0.0 || (dx == 0.0 && dy == 0.0) {
        return ([x, y, w, h], (0.0, 0.0));
    }
    let span = py::hypot(dx, dy);
    let period = 2.0 * span / speed;
    let ph = py::fpmod(move_t, period) / period;
    let f = if ph < 0.5 { ph * 2.0 } else { (1.0 - ph) * 2.0 };
    let sign = if ph < 0.5 { 1.0 } else { -1.0 };
    ([x + dx * f, y + dy * f, w, h], (dx / span * speed * sign, dy / span * speed * sign))
}

/// `draw_obstacles(s, hole, view, mill_a, move_t)`
pub fn draw_obstacles(s: &mut Surface, hole: &Hole, view: &View, mill_a: f64, move_t: f64) {
    for key in DRAW_ORDER {
        for it in hole.get(key) {
            match key {
                "slopes" => draw_slope(s, &it[..4], it[4], it[5], view),
                "sand" => draw_sand(s, it, view),
                "water" => draw_water(s, it, view),
                "walls" => draw_wall(s, view.rect_px(it), false),
                "movers" => draw_wall(s, view.rect_px(&mover_rect(it, move_t).0), true),
                "bumpers" => draw_bumper(s, it, view),
                "mills" => draw_mill(s, it, view, mill_a),
                "tunnels" => draw_tunnel(s, it, view),
                "ice" => draw_ice(s, it, view),
                "boosters" => draw_booster(s, it, view),
                "magnets" => draw_magnet(s, it, view),
                "gates" => draw_gate(s, it, view),
                "sticky" => draw_sticky(s, it, view),
                "spinners" => draw_spinner(s, it, view, mill_a),
                _ => draw_jump(s, it, view),
            }
        }
    }
}

/// `draw_course(s, hole, view, mill_a, move_t, cup_r, tee)`
pub fn draw_course(s: &mut Surface, hole: &Hole, view: &View, mill_a: f64, move_t: f64, cup_r: f64, tee: bool) {
    draw_ground(s, hole, view);
    draw_cup(s, hole.cup, view, cup_r);
    if tee {
        draw_tee(s, hole.tee, view);
    }
    draw_obstacles(s, hole, view, mill_a, move_t);
}
