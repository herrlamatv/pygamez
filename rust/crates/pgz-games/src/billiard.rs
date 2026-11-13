//! billiard.rs
//! ===========
//! Billard / Pool - 8-Ball, 9-Ball und Uebungsmodus, gegen die KI oder zu
//! zweit, in drei Ansichten (2D, 3D, frei drehbar).
//!
//! Uebertragen aus `games/billiard.py`: Physik in Teilschritten, Regeln,
//! KI mit Geisterkugel-Zielen, Replay-Aufnahme je Stoss (+ Wiedergabe).

use std::f64::consts::{PI, TAU};

use pgz_core::replay::{self, rnd, Delta, Recorder};
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

const COL_CLOTH: Color = Color::rgb(26, 112, 72);
const COL_CLOTH_D: Color = Color::rgb(20, 92, 58);
const COL_RAIL: Color = Color::rgb(74, 48, 30);
const COL_RAIL_HI: Color = Color::rgb(104, 70, 44);
const COL_POCKET: Color = Color::rgb(10, 12, 10);
const COL_CUE: Color = Color::rgb(245, 245, 240);
const COL_AIM: Color = Color::rgb(240, 240, 200);
const COL_STICK: Color = Color::rgb(208, 168, 96);
const COL_STICK_D: Color = Color::rgb(150, 116, 60);

fn ball_color(n: i64) -> (i32, i32, i32) {
    match n {
        1 | 9 => (232, 190, 40),
        2 | 10 => (40, 84, 180),
        3 | 11 => (200, 48, 48),
        4 | 12 => (120, 60, 160),
        5 | 13 => (224, 120, 40),
        6 | 14 => (34, 130, 80),
        7 | 15 => (150, 44, 52),
        8 => (26, 26, 30),
        _ => (200, 200, 200),
    }
}

const HW: f64 = 127.0;
const HH: f64 = 63.5;
const BR: f64 = 3.1;
const POCKET_R: f64 = 6.4;
const FRICTION: f64 = 0.62;
const WALL_E: f64 = 0.90;
const BALL_E: f64 = 0.94;
const STOP_EPS: f64 = 2.0;
const MAX_SPEED: f64 = 470.0;
const MAX_SHOT_TIME: f64 = 12.0;
const POCKETS: [(f64, f64); 6] = [(-HW, -HH), (0.0, -HH), (HW, -HH), (-HW, HH), (0.0, HH), (HW, HH)];
const VARIANTS: [&str; 3] = ["8ball", "9ball", "practice"];
const VIEWS: [&str; 3] = ["2d", "3d", "free"];
const DIFFS: [&str; 3] = ["easy", "medium", "hard"];
const AI_ACC: [f64; 3] = [0.80, 0.91, 0.985];

fn sq(x: f64) -> f64 {
    py::pow(x, 2.0)
}

#[derive(Clone)]
struct Ball {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    num: i64,
    potted: bool,
}

impl Ball {
    fn new(x: f64, y: f64, num: i64) -> Ball {
        Ball { x, y, vx: 0.0, vy: 0.0, num, potted: false }
    }
    fn speed2(&self) -> f64 {
        self.vx * self.vx + self.vy * self.vy
    }
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
    Over,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Aim,
    Place,
    Rolling,
}

type V3 = (f64, f64, f64);

/// `BilliardGame`
pub struct BilliardGame {
    b: GameBase,
    variant: &'static str,
    view: &'static str,
    diff: usize,
    rec: Option<Recorder>,
    replay: Option<Value>,
    replay_request: Option<Value>,
    rep: Option<Value>,
    rep_at: Option<(usize, usize)>,
    rec_delta: Delta,
    rec_res: Option<String>,
    small: Font,
    tiny: Font,
    huge: Font,
    wins: [i64; 2],
    cam_yaw: f64,
    cam_yaw_t: f64,
    dragging_cam: bool,
    last_mx: Option<i32>,
    var_rects: Vec<Rect>,
    view_rects: Vec<Rect>,
    diff_rects: Vec<Rect>,
    start_rect: Rect,
    hud_h: i32,
    cx: f64,
    cy: f64,
    scale2d: f64,
    focal: f64,
    cam_r: f64,
    cam_el: f64,
    cy3d: f64,
    basis: (V3, V3, V3, V3),
    balls: Vec<Ball>,
    group: [Option<&'static str>; 2],
    current: usize,
    ball_in_hand: bool,
    break_done: bool,
    winner: Option<usize>,
    phase: Phase,
    aim: f64,
    power: f64,
    charging: bool,
    shot_time: f64,
    first_hit: Option<i64>,
    cue_potted: bool,
    potted_shot: Vec<i64>,
    potted_all: Vec<i64>,
    msg: Option<String>,
    msg_t: f64,
    ai_delay: f64,
    state: State,
}

impl BilliardGame {
    /// `BilliardGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let o = (0.0, 0.0, 0.0);
        let mut g = BilliardGame {
            b,
            variant: "8ball",
            view: "2d",
            diff: 1,
            rec: None,
            replay: None,
            replay_request: None,
            rep: None,
            rep_at: None,
            rec_delta: Delta::new(),
            rec_res: None,
            small: f.clone(),
            tiny: f.clone(),
            huge: f,
            wins: [0, 0],
            cam_yaw: 0.0,
            cam_yaw_t: 0.0,
            dragging_cam: false,
            last_mx: None,
            var_rects: Vec::new(),
            view_rects: Vec::new(),
            diff_rects: Vec::new(),
            start_rect: z,
            hud_h: 46,
            cx: 0.0,
            cy: 0.0,
            scale2d: 1.0,
            focal: 1.0,
            cam_r: 1.0,
            cam_el: 0.0,
            cy3d: 0.0,
            basis: (o, o, o, o),
            balls: Vec::new(),
            group: [None, None],
            current: 0,
            ball_in_hand: false,
            break_done: false,
            winner: None,
            phase: Phase::Aim,
            aim: 0.0,
            power: 0.35,
            charging: false,
            shot_time: 0.0,
            first_hit: None,
            cue_potted: false,
            potted_shot: Vec::new(),
            potted_all: Vec::new(),
            msg: None,
            msg_t: 0.0,
            ai_delay: 0.8,
            state: State::Setup,
        };
        g.reset();
        Box::new(g)
    }

    fn build_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::f(14.max(h / 32));
        self.tiny = ui::f(11.max(h / 40));
        self.huge = ui::fb(26.max(h / 12));
    }

    // ----------------------------------------------- Kamera / Projektion

    fn setup_camera(&mut self) {
        let (w, h) = (self.b.width as f64, self.b.height as f64);
        self.hud_h = 46;
        let hud = self.hud_h as f64;
        self.cx = w / 2.0;
        self.cy = hud + (h - hud) * 0.5;
        self.scale2d = ((w - 46.0) / (2.0 * HW)).min((h - hud - 40.0) / (2.0 * HH));
        self.focal = w * 0.92;
        self.cam_r = HW * 2.35;
        self.cam_el = py::radians(42.0);
        self.cy3d = hud + (h - hud) * 0.54;
        self.basis = self.cam_basis();
    }

    fn cam_basis(&self) -> (V3, V3, V3, V3) {
        let yaw = self.cam_yaw;
        let (ce, se) = (self.cam_el.cos(), self.cam_el.sin());
        let (cyw, syw) = (yaw.cos(), yaw.sin());
        let cam = (self.cam_r * ce * syw, -self.cam_r * ce * cyw, self.cam_r * se);
        let (mut fx, mut fy, mut fz) = (-cam.0, -cam.1, -cam.2);
        let mut fl = (fx * fx + fy * fy + fz * fz).sqrt();
        if fl == 0.0 {
            fl = 1.0;
        }
        fx /= fl;
        fy /= fl;
        fz /= fl;
        let (mut rx, mut ry, mut rz) = (fy * 1.0 - fz * 0.0, fz * 0.0 - fx * 1.0, fx * 0.0 - fy * 0.0);
        let mut rl = (rx * rx + ry * ry + rz * rz).sqrt();
        if rl == 0.0 {
            rl = 1.0;
        }
        rx /= rl;
        ry /= rl;
        rz /= rl;
        let u = (ry * fz - rz * fy, rz * fx - rx * fz, rx * fy - ry * fx);
        (cam, (fx, fy, fz), (rx, ry, rz), u)
    }

    /// (x,y) Tisch-Koordinaten -> (sx, sy, scale, depth).
    fn project(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64, f64) {
        if self.view == "2d" {
            return (self.cx + x * self.scale2d, self.cy + y * self.scale2d, self.scale2d, y);
        }
        let (cam, f, r, u) = self.basis;
        let (relx, rely, relz) = (x - cam.0, y - cam.1, z - cam.2);
        let mut zc = relx * f.0 + rely * f.1 + relz * f.2;
        if zc < 1.0 {
            zc = 1.0;
        }
        let xc = relx * r.0 + rely * r.1 + relz * r.2;
        let yc = relx * u.0 + rely * u.1 + relz * u.2;
        (self.cx + self.focal * xc / zc, self.cy3d - self.focal * yc / zc, self.focal / zc, zc)
    }

    fn unproject(&self, sx: f64, sy: f64) -> (f64, f64) {
        if self.view == "2d" {
            return ((sx - self.cx) / self.scale2d, (sy - self.cy) / self.scale2d);
        }
        let (cam, f, r, u) = self.basis;
        let dxc = (sx - self.cx) / self.focal;
        let dyc = -(sy - self.cy3d) / self.focal;
        let dx = dxc * r.0 + dyc * u.0 + f.0;
        let dy = dxc * r.1 + dyc * u.1 + f.1;
        let dz = dxc * r.2 + dyc * u.2 + f.2;
        if dz.abs() < 1e-6 {
            return (0.0, 0.0);
        }
        let s = -cam.2 / dz;
        (cam.0 + s * dx, cam.1 + s * dy)
    }

    // ----------------------------------------------- Rack

    fn new_rack(&mut self) {
        self.balls = vec![Ball::new(-HW * 0.5, 0.0, 0)];
        let fx = HW * 0.45;
        let sp = 2.0 * BR + 0.25;
        let dx = sp * 0.87;
        if self.variant == "9ball" {
            let mut order = vec![1i64];
            order.extend(py::sample(&[2i64, 3, 4, 5, 6, 7, 8], 7));
            order.push(9);
            let lay = [(0.0, 0.0), (1.0, -0.5), (1.0, 0.5), (2.0, -1.0), (2.0, 0.0), (2.0, 1.0), (3.0, -0.5), (3.0, 0.5), (4.0, 0.0)];
            for (&(row, off), &num) in lay.iter().zip(&order) {
                self.balls.push(Ball::new(fx + row * dx, off * sp, num));
            }
        } else {
            let seq: Vec<i64> = if self.variant == "8ball" {
                let mut others: Vec<i64> = (1..16).filter(|n| *n != 8).collect();
                py::shuffle(&mut others);
                let mut seq = Vec::new();
                let mut idx = 0;
                for row in 0..5 {
                    for k in 0..=row {
                        if row == 2 && k == 1 {
                            seq.push(8);
                        } else {
                            seq.push(others[idx]);
                            idx += 1;
                        }
                    }
                }
                seq
            } else {
                let mut seq: Vec<i64> = (1..16).collect();
                py::shuffle(&mut seq);
                seq
            };
            let mut i = 0;
            for row in 0..5 {
                for k in 0..=row {
                    let x = fx + row as f64 * dx;
                    let y = (k as f64 - row as f64 / 2.0) * sp;
                    self.balls.push(Ball::new(x, y, seq[i]));
                    i += 1;
                }
            }
        }
        self.group = [None, None];
        self.current = 0;
        self.ball_in_hand = false;
        self.break_done = false;
        self.winner = None;
        self.phase = Phase::Aim;
        self.aim = 0.0;
        self.power = 0.35;
        self.charging = false;
        self.shot_time = 0.0;
        self.first_hit = None;
        self.cue_potted = false;
        self.potted_shot.clear();
        self.potted_all.clear();
        self.msg = None;
        self.msg_t = 0.0;
        self.ai_delay = 0.8;
    }

    // ----------------------------------------------- Setup

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let bw = 380.min(self.b.width - 50);
        let y0 = (self.b.height as f64 * 0.26) as i32;
        let gap = 8.0;
        let row = |y: i32, n: i32| -> Vec<Rect> {
            let cw = (bw as f64 - gap * (n - 1) as f64) / n as f64;
            (0..n).map(|i| Rect::new((cx as f64 - bw as f64 / 2.0 + i as f64 * (cw + gap)) as i32, y, cw as i32, 42)).collect()
        };
        self.var_rects = row(y0, 3);
        self.view_rects = row(y0 + 84, 3);
        self.diff_rects = row(y0 + 168, 3);
        self.start_rect = Rect::new(cx - 95, y0 + 228, 190, 46);
    }

    fn save_setting(&mut self, key: &str, value: Value) {
        self.b.set_opt("billiard", key, value);
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                k @ ("1" | "2" | "3") => {
                    self.variant = VARIANTS[k.parse::<usize>().unwrap_or(1) - 1];
                    self.save_setting("variant", json!(self.variant));
                    self.b.play_sound("click");
                }
                "v" | "V" => self.cycle_view(),
                "d" | "D" => {
                    self.diff = (self.diff + 1) % 3;
                    self.save_setting("difficulty", json!(self.diff));
                    self.b.play_sound("select");
                }
                "Return" | "space" => self.start_play(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.var_rects.iter().position(|r| r.hit(ev.pos)) {
                self.variant = VARIANTS[i];
                self.save_setting("variant", json!(self.variant));
                self.b.play_sound("click");
                return;
            }
            if let Some(i) = self.view_rects.iter().position(|r| r.hit(ev.pos)) {
                self.view = VIEWS[i];
                self.cam_yaw = 0.0;
                self.cam_yaw_t = 0.0;
                self.save_setting("view", json!(self.view));
                self.b.play_sound("select");
                return;
            }
            if let Some(i) = self.diff_rects.iter().position(|r| r.hit(ev.pos)) {
                self.diff = i;
                self.save_setting("difficulty", json!(i));
                self.b.play_sound("select");
                return;
            }
            if self.start_rect.hit(ev.pos) {
                self.start_play();
            }
        }
    }

    fn cycle_view(&mut self) {
        let i = VIEWS.iter().position(|v| *v == self.view).unwrap_or(0);
        self.view = VIEWS[(i + 1) % 3];
        self.dragging_cam = false;
        if self.view != "free" {
            self.cam_yaw_t = 0.0;
        }
        self.save_setting("view", json!(self.view));
        self.b.play_sound("select");
    }

    fn start_play(&mut self) {
        self.setup_camera();
        self.new_rack();
        self.rec_new();
        self.state = State::Play;
        self.b.play_sound("click");
    }

    // ----------------------------------------------- Eingabe

    fn human_turn(&self) -> bool {
        self.rep.is_some() || self.b.multiplayer || self.current == 0
    }

    fn handle_aim(&mut self, ev: &InputEvent) {
        match ev.kind {
            Kind::MouseMove => {
                let (mx, my) = self.unproject(ev.pos.0 as f64, ev.pos.1 as f64);
                self.aim = (my - self.balls[0].y).atan2(mx - self.balls[0].x);
            }
            Kind::MouseDown if ev.button == 1 => {
                self.charging = true;
                self.power = 0.05;
            }
            Kind::MouseUp if ev.button == 1 => {
                if self.charging {
                    self.charging = false;
                    self.strike();
                }
            }
            Kind::KeyDown => match ev.key.as_str() {
                "Left" | "a" | "A" => self.aim -= py::radians(2.0),
                "Right" | "d" | "D" => self.aim += py::radians(2.0),
                "Up" | "w" | "W" => self.power = 1f64.min(self.power + 0.05),
                "Down" | "s" | "S" => self.power = 0.05f64.max(self.power - 0.05),
                "space" | "Return" => self.strike(),
                _ => {}
            },
            _ => {}
        }
    }

    fn handle_place(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::MouseMove || ev.kind == Kind::MouseDown {
            let (mx, my) = self.unproject(ev.pos.0 as f64, ev.pos.1 as f64);
            let mx = (-HW + BR).max((HW - BR).min(mx));
            let my = (-HH + BR).max((HH - BR).min(my));
            if self.place_free(mx, my) {
                self.balls[0].x = mx;
                self.balls[0].y = my;
            }
            if ev.kind == Kind::MouseDown && ev.button == 1 && self.place_free(self.balls[0].x, self.balls[0].y) {
                self.ball_in_hand = false;
                self.phase = Phase::Aim;
                self.b.play_sound("click");
            }
        }
    }

    fn place_free(&self, x: f64, y: f64) -> bool {
        for b in &self.balls[1..] {
            if b.potted {
                continue;
            }
            if sq(b.x - x) + sq(b.y - y) < sq(2.0 * BR) {
                return false;
            }
        }
        true
    }

    // ----------------------------------------------- Stoss / Physik

    fn strike(&mut self) {
        let sp = MAX_SPEED * self.power;
        self.balls[0].vx = self.aim.cos() * sp;
        self.balls[0].vy = self.aim.sin() * sp;
        self.rec_scene();
        self.phase = Phase::Rolling;
        self.shot_time = 0.0;
        self.first_hit = None;
        self.cue_potted = false;
        self.potted_shot.clear();
        self.b.play_sound("shoot");
        self.b.rumble(60);
    }

    fn all_stopped(&self) -> bool {
        self.balls.iter().all(|b| b.potted || b.speed2() <= STOP_EPS * STOP_EPS)
    }

    fn physics(&mut self, dt: f64) {
        let mut vmax = 0f64;
        for b in &self.balls {
            if !b.potted {
                vmax = vmax.max(b.speed2());
            }
        }
        let vmax = vmax.sqrt();
        let steps = 3.max(20.min((vmax * dt / BR) as i64 + 2));
        let h = dt / steps as f64;
        let fr = 0f64.max(1.0 - FRICTION * h);
        for _ in 0..steps {
            for b in self.balls.iter_mut() {
                if b.potted {
                    continue;
                }
                b.x += b.vx * h;
                b.y += b.vy * h;
                b.vx *= fr;
                b.vy *= fr;
                if b.speed2() < STOP_EPS * STOP_EPS {
                    b.vx = 0.0;
                    b.vy = 0.0;
                }
            }
            self.walls();
            self.collisions();
            self.pockets();
        }
    }

    fn walls(&mut self) {
        for b in self.balls.iter_mut() {
            if b.potted {
                continue;
            }
            if b.x < -HW + BR {
                b.x = -HW + BR;
                b.vx = -b.vx * WALL_E;
            } else if b.x > HW - BR {
                b.x = HW - BR;
                b.vx = -b.vx * WALL_E;
            }
            if b.y < -HH + BR {
                b.y = -HH + BR;
                b.vy = -b.vy * WALL_E;
            } else if b.y > HH - BR {
                b.y = HH - BR;
                b.vy = -b.vy * WALL_E;
            }
        }
    }

    fn collisions(&mut self) {
        let idx: Vec<usize> = (0..self.balls.len()).filter(|&i| !self.balls[i].potted).collect();
        let n = idx.len();
        for ii in 0..n {
            for jj in ii + 1..n {
                let (ai, ci) = (idx[ii], idx[jj]);
                let (a, c) = (self.balls[ai].clone(), self.balls[ci].clone());
                let (mut dx, mut dy) = (c.x - a.x, c.y - a.y);
                let mut d2 = dx * dx + dy * dy;
                if d2 >= sq(2.0 * BR) || d2 <= 1e-9 {
                    if d2 <= 1e-9 {
                        dx = 0.01;
                        dy = 0.0;
                        d2 = 0.0001;
                    } else {
                        continue;
                    }
                }
                let dist = d2.sqrt();
                let (nx, ny) = (dx / dist, dy / dist);
                let overlap = 2.0 * BR - dist;
                {
                    let a = &mut self.balls[ai];
                    a.x -= nx * overlap / 2.0;
                    a.y -= ny * overlap / 2.0;
                }
                {
                    let c = &mut self.balls[ci];
                    c.x += nx * overlap / 2.0;
                    c.y += ny * overlap / 2.0;
                }
                let (a, c) = (&self.balls[ai], &self.balls[ci]);
                let rvx = a.vx - c.vx;
                let rvy = a.vy - c.vy;
                let vn = rvx * nx + rvy * ny;
                if vn > 0.0 {
                    let jimp = (1.0 + BALL_E) * vn / 2.0;
                    let (an, cn) = (a.num, c.num);
                    {
                        let a = &mut self.balls[ai];
                        a.vx -= jimp * nx;
                        a.vy -= jimp * ny;
                    }
                    {
                        let c = &mut self.balls[ci];
                        c.vx += jimp * nx;
                        c.vy += jimp * ny;
                    }
                    if self.first_hit.is_none() && (an == 0 || cn == 0) {
                        self.first_hit = Some(if an == 0 { cn } else { an });
                        self.b.play_sound("bounce");
                    }
                }
            }
        }
    }

    fn pockets(&mut self) {
        for i in 0..self.balls.len() {
            if self.balls[i].potted {
                continue;
            }
            for &(px, py_) in &POCKETS {
                let b = &self.balls[i];
                if sq(b.x - px) + sq(b.y - py_) <= sq(POCKET_R) {
                    let b = &mut self.balls[i];
                    b.potted = true;
                    b.vx = 0.0;
                    b.vy = 0.0;
                    let num = b.num;
                    if num == 0 {
                        self.cue_potted = true;
                    } else {
                        self.potted_shot.push(num);
                        self.potted_all.push(num);
                    }
                    self.b.play_sound("point");
                    break;
                }
            }
        }
    }

    // ----------------------------------------------- Zugaufloesung

    fn object_balls_left(&self, lo: i64, hi: i64) -> Vec<i64> {
        self.balls.iter().filter(|b| !b.potted && b.num != 0 && lo <= b.num && b.num <= hi).map(|b| b.num).collect()
    }

    fn resolve_shot(&mut self) {
        if !self.break_done && !self.potted_shot.is_empty() {
            self.b.ach_event("billiard_break", None);
        }
        self.break_done = true;
        if let Some(mut r) = self.rec.take() {
            let s = self.rec_sample();
            r.close(Some(s), Map::new());
            self.rec = Some(r);
        }
        self.rec_res = None;
        match self.variant {
            "practice" => self.resolve_practice(),
            "9ball" => self.resolve_9ball(),
            _ => self.resolve_8ball(),
        }
        self.rec_result();
        if self.cue_potted && self.state == State::Play {
            self.balls[0].potted = false;
            self.balls[0].vx = 0.0;
            self.balls[0].vy = 0.0;
            if !self.ball_in_hand {
                self.respot_cue();
            }
        }
        if self.state == State::Play {
            if self.ball_in_hand && !self.human_turn() {
                self.ai_place();
            }
            if self.ball_in_hand && self.human_turn() {
                self.phase = Phase::Place;
            } else {
                self.phase = Phase::Aim;
                self.power = 0.35;
                if !self.b.multiplayer && self.current == 1 {
                    self.ai_delay = 0.7;
                }
            }
        }
    }

    fn respot_cue(&mut self) {
        for x in [-HW * 0.5, -HW * 0.6, -HW * 0.4, -HW * 0.7, 0.0] {
            if self.place_free(x, 0.0) {
                self.balls[0].x = x;
                self.balls[0].y = 0.0;
                return;
            }
        }
        self.balls[0].x = -HW * 0.5;
        self.balls[0].y = 0.0;
    }

    fn foul(&mut self, key: &str) {
        self.rec_res = Some(key.to_string());
        self.msg = Some(t(key));
        self.msg_t = 2.2;
        self.ball_in_hand = true;
        self.current = 1 - self.current;
        self.b.play_sound("hit");
    }

    fn resolve_practice(&mut self) {
        self.b.score = self.potted_all.len() as i64;
        if self.cue_potted {
            self.ball_in_hand = false;
        }
        if self.object_balls_left(1, 15).is_empty() {
            let sc = self.b.score;
            self.new_rack();
            self.state = State::Play;
            self.b.score = sc;
            self.msg = Some(t("bil.reracked"));
            self.msg_t = 2.0;
        }
    }

    fn group_range(g: &str) -> (i64, i64) {
        if g == "solid" {
            (1, 7)
        } else {
            (9, 15)
        }
    }

    fn group_cleared(&self, player: usize) -> bool {
        let Some(g) = self.group[player] else {
            return false;
        };
        let (lo, hi) = Self::group_range(g);
        self.object_balls_left(lo, hi).is_empty()
    }

    fn resolve_8ball(&mut self) {
        let cur = self.current;
        let opp = 1 - cur;
        let potted = self.potted_shot.clone();
        let mut foul = self.cue_potted || self.first_hit.is_none();
        if let (Some(g), Some(fh)) = (self.group[cur], self.first_hit) {
            let (lo, hi) = Self::group_range(g);
            if !self.group_cleared(cur) {
                if !(lo..=hi).contains(&fh) {
                    foul = true;
                }
            } else if fh != 8 {
                foul = true;
            }
        }
        if potted.contains(&8) {
            let cleared = self.group_cleared(cur);
            self.winner = Some(if self.group[cur].is_none() || !cleared || self.cue_potted { opp } else { cur });
            self.end();
            return;
        }
        let obj: Vec<i64> = potted.iter().copied().filter(|p| *p != 8).collect();
        if self.group[cur].is_none() && !obj.is_empty() && !foul {
            if obj[0] <= 7 {
                self.group[cur] = Some("solid");
                self.group[opp] = Some("stripe");
            } else {
                self.group[cur] = Some("stripe");
                self.group[opp] = Some("solid");
            }
        }
        let made_own = match self.group[cur] {
            Some(g) => {
                let (lo, hi) = Self::group_range(g);
                obj.iter().any(|p| (lo..=hi).contains(p))
            }
            None => !obj.is_empty(),
        };
        if foul {
            self.foul("bil.foul");
        } else if !made_own {
            self.current = opp;
        }
    }

    fn resolve_9ball(&mut self) {
        let cur = self.current;
        let opp = 1 - cur;
        let left_before = self.lowest_before();
        let mut foul = self.cue_potted || self.first_hit.is_none();
        if let (Some(fh), Some(lb)) = (self.first_hit, left_before) {
            if fh != lb {
                foul = true;
            }
        }
        if self.potted_shot.contains(&9) {
            if !foul {
                self.winner = Some(cur);
                self.end();
                return;
            }
            for b in self.balls.iter_mut() {
                if b.num == 9 {
                    b.potted = true;
                }
            }
            self.respot_9();
        }
        if foul {
            self.foul("bil.foul");
        } else if self.potted_shot.is_empty() {
            self.current = opp;
        }
    }

    fn lowest_before(&self) -> Option<i64> {
        self.balls.iter().filter(|b| b.num != 0 && (!b.potted || self.potted_shot.contains(&b.num))).map(|b| b.num).min()
    }

    fn respot_9(&mut self) {
        for i in 0..self.balls.len() {
            if self.balls[i].num == 9 {
                self.balls[i].potted = false;
                self.balls[i].x = HW * 0.45;
                self.balls[i].y = 0.0;
                let (x, y) = (self.balls[i].x, self.balls[i].y);
                if !self.place_free(x, y) {
                    self.balls[i].x = HW * 0.55;
                }
                if let Some(k) = self.potted_all.iter().position(|n| *n == 9) {
                    self.potted_all.remove(k);
                }
            }
        }
    }

    fn ai_place(&mut self) {
        let target = self.ai_target_ball().map(|i| (self.balls[i].x, self.balls[i].y));
        let mut placed = false;
        if let Some((tx, ty)) = target {
            for off in [BR * 5.0, BR * 8.0, BR * 12.0] {
                let x = tx - HW * 0.02 - off;
                if -HW + BR < x && x < HW - BR && self.place_free(x, ty) {
                    self.balls[0].x = x;
                    self.balls[0].y = ty;
                    placed = true;
                    break;
                }
            }
        }
        if !placed {
            self.respot_cue();
        }
        self.ball_in_hand = false;
    }

    fn end(&mut self) {
        self.state = State::Over;
        if let Some(w) = self.winner {
            self.wins[w] += 1;
            if !self.b.multiplayer {
                if w == 0 {
                    self.b.score = self.wins[0];
                    self.b.play_sound("win");
                    self.b.report_result(true);
                } else {
                    self.b.play_sound("gameover");
                    self.b.report_result(false);
                }
            } else {
                self.b.report_duel(w);
                self.b.play_sound("win");
            }
        }
        self.b.game_over = true;
    }

    fn restart(&mut self) {
        self.b.game_over = false;
        self.new_rack();
        self.rec_new();
        self.state = State::Play;
        self.b.play_sound("click");
    }

    // ----------------------------------------------- Replay-Aufnahme

    fn rec_new(&mut self) {
        self.replay = None;
        self.rec_res = None;
        self.rec_delta.reset();
        let mut meta = Map::new();
        meta.insert("variant".into(), json!(self.variant));
        meta.insert("view".into(), json!(self.view));
        meta.insert("diff".into(), json!(self.diff));
        meta.insert("players".into(), json!(if self.b.multiplayer { 2 } else { 1 }));
        self.rec = replay::recorder("billiard", shell::replay_enabled(&self.b.settings), meta);
    }

    fn rec_scene(&mut self) {
        if self.rec.is_none() {
            return;
        }
        self.rec_delta.reset();
        let mut info = Map::new();
        info.insert("pl".into(), json!(self.current));
        info.insert("aim".into(), rnd(self.aim, 4));
        info.insert("pw".into(), rnd(self.power, 3));
        info.insert("grp".into(), json!(self.group.iter().map(|g| g.map(|s| s.to_string())).collect::<Vec<_>>()));
        info.insert("pall".into(), json!(self.potted_all));
        let balls: Vec<Value> = self.balls.iter().map(|b| json!([b.num, rnd(b.x, 1), rnd(b.y, 1), if b.potted { 1 } else { 0 }])).collect();
        info.insert("balls".into(), Value::Array(balls));
        if let Some(r) = self.rec.as_mut() {
            r.scene(false, info);
        }
    }

    fn rec_sample(&mut self) -> Vec<Value> {
        let mut out = Vec::new();
        for i in 0..self.balls.len() {
            let b = &self.balls[i];
            let st = [rnd(b.x, 1), rnd(b.y, 1), json!(if b.potted { 1 } else { 0 })];
            if self.rec_delta.push(i as i64, &st) {
                out.push(json!(i));
                out.extend(st);
            }
        }
        out
    }

    fn rec_result(&mut self) {
        if self.rec.is_none() {
            return;
        }
        let res = match (&self.rec_res, self.winner) {
            (Some(r), _) => r.clone(),
            (None, Some(w)) => if w == 0 { "bil.win_you".into() } else { "bil.win_ai".into() },
            (None, None) => if self.potted_shot.is_empty() { "bil.res_miss".into() } else { "bil.res_pot".into() },
        };
        let mut info = Map::new();
        info.insert("res".into(), json!(res));
        info.insert("pot".into(), json!(self.potted_shot));
        info.insert("nxt".into(), json!(self.current));
        info.insert("final".into(), json!(self.winner.is_some()));
        info.insert("win".into(), json!(self.winner.map(|w| w as i64).unwrap_or(-1)));
        if let Some(r) = self.rec.as_mut() {
            r.set_last(info);
        }
        if self.state == State::Over {
            self.rec_finish();
        }
    }

    fn rec_finish(&mut self) {
        let Some(mut r) = self.rec.take() else {
            return;
        };
        let sub = if self.variant == "practice" {
            tf("bil.potted", &[("n", &self.potted_all.len().to_string())])
        } else if let Some(w) = self.winner {
            if self.b.multiplayer {
                tf("common.player_wins_name", &[("name", &self.b.pname(w).chars().take(10).collect::<String>())])
            } else if w == 0 {
                t("bil.win_you")
            } else {
                t("bil.win_ai")
            }
        } else {
            t(&format!("bil.var.{}", self.variant))
        };
        let mut meta = Map::new();
        meta.insert("winner".into(), json!(self.winner.map(|w| w as i64).unwrap_or(-1)));
        let title = format!("{}  ·  {}", t(&format!("bil.var.{}", self.variant)), t(&format!("bil.view.{}", self.view)));
        self.replay = r.result(&title, &sub, meta);
    }

    fn rec_snapshot(&mut self) {
        let title = format!("{}  ·  {}", t("bil.var.practice"), t(&format!("bil.view.{}", self.view)));
        let sub = tf("bil.potted", &[("n", &self.potted_all.len().to_string())]);
        if let Some(r) = self.rec.as_mut() {
            if !r.has_scenes() {
                return;
            }
            self.replay = r.result(&title, &sub, Map::new());
        }
    }

    fn open_replay(&mut self) {
        if self.state == State::Play {
            self.rec_snapshot();
        }
        if self.replay.is_some() {
            self.replay_request = self.replay.clone();
            self.b.play_sound("click");
        }
    }

    // ----------------------------------------------- Replay-Wiedergabe

    /// Schaltet diese Instanz auf reine Wiedergabe um.
    pub fn replay_begin(&mut self, rep: &Value) {
        self.rec = None;
        self.replay = None;
        self.replay_request = None;
        self.rep = Some(rep.clone());
        self.rep_at = None;
        let meta = rep.get("meta").cloned().unwrap_or(json!({}));
        if let Some(v) = meta.get("variant").and_then(|v| v.as_str()).and_then(|v| VARIANTS.iter().find(|x| **x == v)) {
            self.variant = v;
        }
        if let Some(v) = meta.get("view").and_then(|v| v.as_str()).and_then(|v| VIEWS.iter().find(|x| **x == v)) {
            self.view = v;
        }
        self.diff = meta.get("diff").and_then(|v| v.as_i64()).unwrap_or(0).clamp(0, 2) as usize;
        self.b.multiplayer = meta.get("players").and_then(|v| v.as_i64()).filter(|v| *v != 0).unwrap_or(1) > 1;
        self.cam_yaw = 0.0;
        self.cam_yaw_t = 0.0;
        self.setup_camera();
        self.state = State::Play;
        self.b.game_over = false;
        self.msg = None;
        self.msg_t = 0.0;
        self.winner = None;
        self.wins = [0, 0];
        self.replay_seek(0, 0);
    }

    /// Setzt Kugeln, Zielen und HUD auf Szene `index`, Sample `frame`.
    pub fn replay_seek(&mut self, index: usize, frame: usize) {
        let Some(rep) = self.rep.clone() else {
            return;
        };
        let scenes = rep.get("scenes").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        if scenes.is_empty() {
            return;
        }
        let index = index.min(scenes.len() - 1);
        let sc = &scenes[index];
        let frames = sc.get("f").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let n = frames.len().max(1);
        let frame = frame.min(n - 1);
        let last = frame >= n - 1;
        self.current = sc.get("pl").and_then(|v| v.as_i64()).unwrap_or(0).clamp(0, 1) as usize;
        self.aim = sc.get("aim").and_then(|v| v.as_f64()).unwrap_or(0.0);
        self.power = sc.get("pw").and_then(|v| v.as_f64()).unwrap_or(0.35);
        let grp = sc.get("grp").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let g = |i: usize| -> Option<&'static str> {
            match grp.get(i).and_then(|v| v.as_str()) {
                Some("solid") => Some("solid"),
                Some("stripe") => Some("stripe"),
                _ => None,
            }
        };
        self.group = [g(0), g(1)];
        self.potted_all = sc.get("pall").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|v| v.as_i64()).collect()).unwrap_or_default();
        self.ball_in_hand = false;
        self.phase = Phase::Rolling;
        self.winner = None;
        let start = match self.rep_at {
            Some((i, f)) if i == index && frame >= f => f + 1,
            _ => {
                let src = sc.get("balls").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                self.balls = src
                    .iter()
                    .map(|b| {
                        let num = |k: usize| b.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let mut ball = Ball::new(num(1), num(2), num(0) as i64);
                        ball.potted = num(3) != 0.0;
                        ball
                    })
                    .collect();
                if self.balls.is_empty() {
                    self.balls.push(Ball::new(0.0, 0.0, 0));
                }
                0
            }
        };
        for fr in frames.iter().take(frame + 1).skip(start) {
            let Some(fr) = fr.as_array() else {
                continue;
            };
            let num = |k: usize| fr.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let mut j = 0;
            while j + 3 < fr.len() {
                let i = num(j) as i64;
                if i >= 0 && (i as usize) < self.balls.len() {
                    let b = &mut self.balls[i as usize];
                    b.x = num(j + 1);
                    b.y = num(j + 2);
                    b.potted = num(j + 3) != 0.0;
                }
                j += 4;
            }
        }
        if last {
            if let Some(pot) = sc.get("pot").and_then(|v| v.as_array()) {
                self.potted_all.extend(pot.iter().filter_map(|v| v.as_i64()));
            }
            if let Some(res) = sc.get("res").and_then(|v| v.as_str()).filter(|s| !s.is_empty()) {
                self.msg = Some(t(res));
            }
            if sc.get("final").and_then(|v| v.as_bool()).unwrap_or(false) {
                let win = sc.get("win").and_then(|v| v.as_i64()).unwrap_or(-1);
                self.winner = if win >= 0 { Some(win as usize) } else { None };
            }
        } else {
            self.msg = None;
        }
        self.rep_at = Some((index, frame));
    }

    /// Zeichnet den aktuellen Replay-Stand (ohne Menue-Overlay).
    pub fn replay_draw(&mut self, aiming: bool, banner: bool) {
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        if self.view != "2d" {
            self.basis = self.cam_basis();
        }
        self.draw_table(&mut s);
        if aiming {
            self.phase = Phase::Aim;
            self.draw_balls(&mut s);
            self.draw_aim(&mut s);
            self.phase = Phase::Rolling;
        } else {
            self.draw_balls(&mut s);
        }
        self.draw_hud(&mut s);
        if banner && self.winner.is_some() {
            self.draw_over(&mut s);
        }
    }

    // ----------------------------------------------- KI

    fn ai_legal_targets(&self) -> Vec<usize> {
        let sel = |f: &dyn Fn(&Ball) -> bool| -> Vec<usize> { (0..self.balls.len()).filter(|&i| !self.balls[i].potted && f(&self.balls[i])).collect() };
        if self.variant == "9ball" {
            let low = self.lowest_before();
            return sel(&|b| Some(b.num) == low);
        }
        if self.variant == "practice" {
            return sel(&|b| b.num != 0);
        }
        match self.group[self.current] {
            None => sel(&|b| b.num != 0 && b.num != 8),
            Some(_) if self.group_cleared(self.current) => sel(&|b| b.num == 8),
            Some(g) => {
                let (lo, hi) = Self::group_range(g);
                sel(&|b| lo <= b.num && b.num <= hi)
            }
        }
    }

    fn ai_target_ball(&self) -> Option<usize> {
        let ts = self.ai_legal_targets();
        let cue = &self.balls[0];
        let mut best: Option<(f64, usize)> = None;
        for i in ts {
            let b = &self.balls[i];
            let k = sq(b.x - cue.x) + sq(b.y - cue.y);
            if best.is_none_or(|bb| k < bb.0) {
                best = Some((k, i));
            }
        }
        best.map(|b| b.1)
    }

    fn ai_shoot(&mut self) {
        let mut best: Option<(f64, f64, f64)> = None;
        let (cx, cy) = (self.balls[0].x, self.balls[0].y);
        for ti in self.ai_legal_targets() {
            let tb = self.balls[ti].clone();
            for &(px, py_) in &POCKETS {
                let (dpx, dpy) = (px - tb.x, py_ - tb.y);
                let mut dl = py::hypot(dpx, dpy);
                if dl == 0.0 {
                    dl = 1.0;
                }
                let ghost_x = tb.x - dpx / dl * 2.0 * BR;
                let ghost_y = tb.y - dpy / dl * 2.0 * BR;
                let aim = (ghost_y - cy).atan2(ghost_x - cx);
                let a1 = (tb.y - cy).atan2(tb.x - cx);
                let cut = (py::fpmod(aim - a1 + PI, 2.0 * PI) - PI).abs();
                if cut > py::radians(80.0) {
                    continue;
                }
                if self.blocked(cx, cy, ghost_x, ghost_y, ti) {
                    continue;
                }
                let dist = py::hypot(ghost_x - cx, ghost_y - cy) + py::hypot(dpx, dpy);
                let score = cut * 60.0 + dist * 0.2;
                if best.is_none_or(|b| score < b.0) {
                    best = Some((score, aim, dist));
                }
            }
        }
        match best {
            None => match self.ai_target_ball() {
                None => {
                    self.aim = py::uniform(0.0, TAU);
                    self.power = 0.4;
                }
                Some(ti) => {
                    let tb = &self.balls[ti];
                    self.aim = (tb.y - cy).atan2(tb.x - cx);
                    self.power = 0.5;
                }
            },
            Some((_, aim, dist)) => {
                let acc = AI_ACC[self.diff];
                self.aim = aim + py::uniform(-1.0, 1.0) * (1.0 - acc) * 0.28;
                self.power = 0.3f64.max(0.92f64.min(0.32 + dist / (2.0 * HW) * 0.6));
            }
        }
        self.strike();
    }

    fn blocked(&self, x0: f64, y0: f64, x1: f64, y1: f64, ignore: usize) -> bool {
        let (dx, dy) = (x1 - x0, y1 - y0);
        let seg2 = dx * dx + dy * dy;
        if seg2 < 1e-6 {
            return false;
        }
        for (i, b) in self.balls.iter().enumerate() {
            if i == 0 || i == ignore || b.potted {
                continue;
            }
            let t_ = ((b.x - x0) * dx + (b.y - y0) * dy) / seg2;
            if t_ <= 0.02 || t_ >= 0.98 {
                continue;
            }
            let cx = x0 + t_ * dx;
            let cy = y0 + t_ * dy;
            if sq(b.x - cx) + sq(b.y - cy) < sq(2.0 * BR) {
                return true;
            }
        }
        false
    }

    // ----------------------------------------------- Zeichnen

    fn corners(e: f64) -> [(f64, f64); 4] {
        [(-HW - e, -HH - e), (HW + e, -HH - e), (HW + e, HH + e), (-HW - e, HH + e)]
    }

    fn draw_table(&self, s: &mut Surface) {
        let rail: Vec<(f64, f64)> = Self::corners(BR * 2.4).iter().map(|&(x, y)| {
            let p = self.project(x, y, 0.0);
            (p.0, p.1)
        }).collect();
        let bed: Vec<(f64, f64)> = Self::corners(0.0).iter().map(|&(x, y)| {
            let p = self.project(x, y, 0.0);
            (p.0, p.1)
        }).collect();
        draw::polygon(s, COL_RAIL, &rail, 0);
        draw::polygon(s, COL_RAIL_HI, &rail, 3);
        draw::polygon(s, COL_CLOTH, &bed, 0);
        draw::polygon(s, COL_CLOTH_D, &bed, 2);
        for &(px, py_) in &POCKETS {
            let (sx, sy, sc, _) = self.project(px, py_, 0.0);
            draw::circle(s, COL_POCKET, (sx as i32, sy as i32), 4.max((POCKET_R * sc) as i32), 0);
        }
    }

    fn draw_balls(&self, s: &mut Surface) {
        let mut order: Vec<&Ball> = self.balls.iter().filter(|b| !b.potted).collect();
        order.sort_by(|a, b| {
            let (da, db) = (self.project(a.x, a.y, 0.0).3, self.project(b.x, b.y, 0.0).3);
            db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
        });
        let z = if self.view != "2d" { BR } else { 0.0 };
        for b in order {
            let (sxs, sys_, scs, _) = self.project(b.x, b.y, 0.0);
            if self.view != "2d" {
                let mut sh = pg::surface_alpha((BR * scs * 2.2) as i32, (BR * scs * 1.4) as i32);
                let r = sh.get_rect();
                draw::ellipse(&mut sh, rgba(0, 0, 0, 90), r, 0);
                s.bl(&sh, (sxs - BR * scs * 1.1, sys_ - BR * scs * 0.7));
            }
            let (sx, sy, sc, _) = self.project(b.x, b.y, z);
            let r = 3.max((BR * sc) as i32);
            self.draw_ball(s, b, sx as i32, sy as i32, r);
        }
        if self.phase == Phase::Place && self.human_turn() {
            let (sx, sy, sc, _) = self.project(self.balls[0].x, self.balls[0].y, z);
            let k = 0.5 + 0.5 * (pg::ticks() as f64 / 150.0).sin();
            let g = (120.0 + 120.0 * k) as i32;
            draw::circle(s, rgb(g, g, g), (sx as i32, sy as i32), (BR * sc) as i32 + 3, 2);
        }
    }

    fn draw_ball(&self, s: &mut Surface, b: &Ball, x: i32, y: i32, r: i32) {
        if b.num == 0 {
            draw::circle(s, COL_CUE, (x, y), r, 0);
        } else if (9..=15).contains(&b.num) {
            let base = ball_color(b.num);
            let d = 2 * r;
            let mut surf = pg::surface_alpha(d, d);
            draw::circle(&mut surf, rgb(245, 245, 240), (r, r), r, 0);
            let band_h = 2.max(r);
            draw::rect(&mut surf, base, Rect::new(0, r - band_h / 2, d, band_h), 0);
            let mut mask = pg::surface_alpha(d, d);
            draw::circle(&mut mask, rgba(255, 255, 255, 255), (r, r), r, 0);
            surf.bl_flags(&mask, (0, 0), BLEND_RGBA_MULT);
            s.bl(&surf, (x - r, y - r));
        } else {
            draw::circle(s, ball_color(b.num), (x, y), r, 0);
        }
        draw::circle(s, rgb(255, 255, 255), (x - r / 3, y - r / 3), 1.max(r / 4), 0);
        if b.num != 0 && r >= 7 {
            let col = if b.num != 8 { rgb(20, 20, 20) } else { rgb(235, 235, 235) };
            let img = self.tiny.render(&b.num.to_string(), true, col);
            s.blr(&img, img.get_rect().center_at((x, y)));
        }
        draw::circle(s, rgb(10, 14, 10), (x, y), r, 1);
    }

    fn predict(&self) -> (f64, f64, Option<usize>) {
        let (ox, oy) = (self.aim.cos(), self.aim.sin());
        let cue = &self.balls[0];
        let mut best_t = 1e9;
        let mut hit = None;
        for (i, b) in self.balls.iter().enumerate() {
            if i == 0 || b.potted {
                continue;
            }
            let (relx, rely) = (b.x - cue.x, b.y - cue.y);
            let proj = relx * ox + rely * oy;
            if proj <= 0.0 {
                continue;
            }
            let perp2 = sq(relx - proj * ox) + sq(rely - proj * oy);
            if perp2 <= sq(2.0 * BR) {
                let back = 0f64.max(sq(2.0 * BR) - perp2).sqrt();
                let tc = proj - back;
                if 0.0 < tc && tc < best_t {
                    best_t = tc;
                    hit = Some(i);
                }
            }
        }
        if hit.is_none() {
            let tx = if ox > 0.0 {
                ((HW - BR) - cue.x) / ox
            } else if ox < 0.0 {
                ((-HW + BR) - cue.x) / ox
            } else {
                1e9
            };
            let ty = if oy > 0.0 {
                ((HH - BR) - cue.y) / oy
            } else if oy < 0.0 {
                ((-HH + BR) - cue.y) / oy
            } else {
                1e9
            };
            best_t = 0f64.max(tx.min(ty));
        }
        (cue.x + ox * best_t, cue.y + oy * best_t, hit)
    }

    fn draw_aim(&self, s: &mut Surface) {
        if self.phase == Phase::Place {
            return;
        }
        let (cxp, cyp, hit) = self.predict();
        let cue = &self.balls[0];
        let cs = self.project(cue.x, cue.y, 0.0);
        let ce = self.project(cxp, cyp, 0.0);
        draw::line(s, COL_AIM, (cs.0, cs.1), (ce.0, ce.1), 2);
        if let Some(hi) = hit {
            let (gx, gy, gsc, _) = self.project(cxp, cyp, 0.0);
            draw::circle(s, COL_AIM, (gx as i32, gy as i32), 3.max((BR * gsc) as i32), 1);
            let hb = &self.balls[hi];
            let (tdx, tdy) = (hb.x - cxp, hb.y - cyp);
            let mut tl = py::hypot(tdx, tdy);
            if tl == 0.0 {
                tl = 1.0;
            }
            let ex = hb.x + tdx / tl * 6.0 * BR;
            let ey = hb.y + tdy / tl * 6.0 * BR;
            let hs = self.project(hb.x, hb.y, 0.0);
            let he = self.project(ex, ey, 0.0);
            draw::line(s, rgb(200, 220, 255), (hs.0, hs.1), (he.0, he.1), 2);
        }
        let (ox, oy) = (self.aim.cos(), self.aim.sin());
        let pull = 4.0 + self.power * 16.0;
        let bx = cue.x - ox * (BR + pull);
        let by = cue.y - oy * (BR + pull);
        let ex = cue.x - ox * (BR + pull + 70.0);
        let ey = cue.y - oy * (BR + pull + 70.0);
        let bs2 = self.project(bx, by, 0.0);
        let es2 = self.project(ex, ey, 0.0);
        draw::line(s, COL_STICK, (bs2.0, bs2.1), (es2.0, es2.1), 5);
        draw::line(s, COL_STICK_D, (bs2.0, bs2.1), (es2.0, es2.1), 1);
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        draw::rect(s, p.panel, Rect::new(0, 0, w, self.hud_h), 0);
        draw::line(s, p.border, (0, self.hud_h), (w, self.hud_h), 1);
        let cy = self.hud_h / 2;
        let vt = self.tiny.render(&format!("{}  ·  {}", t(&format!("bil.view.{}", self.view)), t(&format!("bil.var.{}", self.variant))), true, p.text_dim);
        s.blr(&vt, vt.get_rect().midleft_at((12, cy)));
        let (mw, mh) = (90, 10);
        let mx = w - mw - 14;
        draw::rect_r(s, p.btn, Rect::new(mx, cy - mh / 2, mw, mh), 0, 4);
        draw::rect_r(s, self.b.accent, Rect::new(mx, cy - mh / 2, (mw as f64 * self.power) as i32, mh), 0, 4);
        let mid = if let Some(m) = self.msg.as_ref().filter(|m| !m.is_empty()) {
            m.clone()
        } else if self.variant == "practice" {
            tf("bil.potted", &[("n", &self.potted_all.len().to_string())])
        } else if self.phase == Phase::Place {
            t("bil.place_cue")
        } else if !self.b.multiplayer && self.current == 1 {
            t("bil.ai_turn")
        } else {
            let mut who = if self.b.multiplayer { self.b.pname(self.current).chars().take(10).collect() } else { t("bil.you") };
            if let Some(g) = self.group[self.current] {
                who += &format!(" ({})", t(&format!("bil.grp.{g}")));
            }
            who
        };
        let img = self.small.render(&mid, true, self.b.accent);
        s.blr(&img, img.get_rect().center_at((w / 2, cy)));
    }

    fn draw_over(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let mut ov = pg::surface_alpha(w, 100);
        ov.fill_c(rgba(8, 12, 10, 210));
        let y = self.b.height / 2 - 50;
        s.bl(&ov, (0, y));
        draw::line(s, self.b.accent, (0, y), (w, y), 1);
        draw::line(s, self.b.accent, (0, y + 99), (w, y + 99), 1);
        let cx = w / 2;
        let head = if self.variant == "practice" {
            self.huge.render(&tf("bil.potted", &[("n", &self.potted_all.len().to_string())]), true, self.b.accent)
        } else if self.b.multiplayer {
            let name: String = self.winner.map(|wn| self.b.pname(wn)).unwrap_or_default().chars().take(10).collect();
            self.huge.render(&tf("common.player_wins_name", &[("name", &name)]), true, self.b.accent)
        } else {
            let won = self.winner == Some(0);
            self.huge.render(&t(if won { "bil.win_you" } else { "bil.win_ai" }), true, if won { self.b.accent } else { p.text_dim })
        };
        s.blr(&head, head.get_rect().center_at((cx, y + 36)));
        let mut hint_txt = t("bil.new_round");
        if self.replay.is_some() && self.rep.is_none() {
            hint_txt += &format!("  ·  {}", t("bil.replay_hint"));
        }
        let hint = self.tiny.render(&hint_txt, true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, y + 76)));
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let cx = self.b.width / 2;
        let h = self.b.height;
        let title = self.huge.render(&t("bil.title"), true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.12) as i32)));
        let sub = self.small.render(&t("bil.subtitle"), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, (h as f64 * 0.185) as i32)));
        let label = |s: &mut Surface, rects: &[Rect], txt: &str| {
            let im = self.tiny.render(txt, true, p.text_dim);
            s.blr(&im, im.get_rect().midbottom_at((cx, rects[0].y - 4)));
        };
        label(s, &self.var_rects, &t("bil.lbl_variant"));
        for (i, rc) in self.var_rects.iter().enumerate() {
            self.btn(s, *rc, &t(&format!("bil.var.{}", VARIANTS[i])), self.variant == VARIANTS[i]);
        }
        label(s, &self.view_rects, &t("bil.lbl_view"));
        for (i, rc) in self.view_rects.iter().enumerate() {
            self.btn(s, *rc, &t(&format!("bil.view.{}", VIEWS[i])), self.view == VIEWS[i]);
        }
        label(s, &self.diff_rects, &t("bil.lbl_diff"));
        for (i, rc) in self.diff_rects.iter().enumerate() {
            self.btn(s, *rc, &t(&format!("bil.diff.{}", DIFFS[i])), self.diff == i);
        }
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 9);
        draw::rect_r(s, self.b.accent, self.start_rect, 2, 9);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        let hint = self.tiny.render(&t("bil.setup_hint"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, h - 14)));
    }

    fn btn(&self, s: &mut Surface, rc: Rect, text: &str, on: bool) {
        let p = ui::pal();
        draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 8);
        if on {
            draw::rect_r(s, self.b.accent, rc, 2, 8);
        } else {
            draw::rect_r(s, p.border, rc, 1, 8);
        }
        let im = self.small.render(text, true, if on { p.text } else { p.text_dim });
        s.blr(&im, im.get_rect().center_at(rc.center()));
    }
}

impl Game for BilliardGame {
    fn replay_begin(&mut self, rep: &Value) -> bool {
        BilliardGame::replay_begin(self, rep);
        true
    }

    fn replay_seek(&mut self, index: usize, frame: usize) {
        BilliardGame::replay_seek(self, index, frame);
    }

    fn replay_draw(&mut self, aiming: bool, banner: bool) {
        BilliardGame::replay_draw(self, aiming, banner);
    }

    fn replay_hud_h(&self) -> i32 {
        self.hud_h
    }

    fn base(&self) -> &GameBase {
        &self.b
    }
    fn base_mut(&mut self) -> &mut GameBase {
        &mut self.b
    }

    fn reset(&mut self) {
        self.b.score = 0;
        self.b.game_over = false;
        let v = self.b.opt_str("billiard", "variant", "8ball");
        self.variant = VARIANTS.iter().find(|x| **x == v).copied().unwrap_or("8ball");
        let v = self.b.opt_str("billiard", "view", "2d");
        self.view = VIEWS.iter().find(|x| **x == v).copied().unwrap_or("2d");
        self.diff = self.b.opt_int_py("billiard", "difficulty", 1).unwrap_or(1).clamp(0, 2) as usize;
        self.rec = None;
        self.replay = None;
        self.replay_request = None;
        self.rep = None;
        self.rep_at = None;
        self.rec_delta = Delta::new();
        self.build_fonts();
        self.wins = [0, 0];
        self.cam_yaw = 0.0;
        self.cam_yaw_t = 0.0;
        self.dragging_cam = false;
        self.build_setup_layout();
        self.setup_camera();
        self.new_rack();
        self.state = State::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.build_fonts();
        self.build_setup_layout();
        self.setup_camera();
    }

    fn wants_right_click(&self) -> bool {
        true
    }

    fn take_replay_request(&mut self) -> Option<Value> {
        self.replay_request.take()
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown && ev.key_in(&["v", "V"]) && self.state == State::Play {
            self.cycle_view();
            return;
        }
        if self.state == State::Setup {
            self.handle_setup(ev);
            return;
        }
        if self.state == State::Over {
            if ev.kind == Kind::KeyDown {
                match ev.key.as_str() {
                    "Return" | "space" => self.restart(),
                    "p" | "P" if self.replay.is_some() => self.open_replay(),
                    "s" | "S" => {
                        self.state = State::Setup;
                        self.b.game_over = false;
                        self.b.play_sound("click");
                    }
                    _ => {}
                }
            } else if ev.kind == Kind::MouseDown && ev.button == 1 {
                self.restart();
            }
            return;
        }
        if self.state != State::Play {
            return;
        }
        if ev.kind == Kind::KeyDown && ev.key_in(&["p", "P"]) && self.variant == "practice" && self.phase == Phase::Aim && self.rec.as_ref().is_some_and(|r| r.has_scenes()) {
            self.open_replay();
            return;
        }
        if self.view == "free" {
            if ev.kind == Kind::MouseDown && ev.button == 3 {
                self.dragging_cam = true;
                self.last_mx = Some(ev.pos.0);
                return;
            }
            if ev.kind == Kind::MouseUp && ev.button == 3 {
                self.dragging_cam = false;
                return;
            }
            if self.dragging_cam {
                if ev.kind == Kind::MouseMove {
                    let dx = ev.pos.0 - self.last_mx.unwrap_or(ev.pos.0);
                    self.last_mx = Some(ev.pos.0);
                    self.cam_yaw_t += dx as f64 * 0.006;
                    return;
                }
                if matches!(ev.kind, Kind::MouseDown | Kind::MouseUp | Kind::KeyDown) {
                    self.dragging_cam = false;
                }
            }
        } else if self.dragging_cam {
            self.dragging_cam = false;
        }
        if !self.human_turn() {
            return;
        }
        if self.phase == Phase::Place {
            self.handle_place(ev);
            return;
        }
        if self.phase != Phase::Aim {
            return;
        }
        self.handle_aim(ev);
    }

    fn update(&mut self, dt: f64) {
        if self.msg_t > 0.0 {
            self.msg_t -= dt;
            if self.msg_t <= 0.0 {
                self.msg = None;
            }
        }
        self.cam_yaw += (self.cam_yaw_t - self.cam_yaw) * 1f64.min(dt * 8.0);
        if self.view != "2d" {
            self.basis = self.cam_basis();
        }
        if self.state != State::Play {
            return;
        }
        match self.phase {
            Phase::Rolling => {
                self.physics(dt);
                self.shot_time += dt;
                if let Some(mut r) = self.rec.take() {
                    r.tick(dt, || Some(self.rec_sample()));
                    self.rec = Some(r);
                }
                if self.all_stopped() || self.shot_time > MAX_SHOT_TIME {
                    self.resolve_shot();
                }
            }
            Phase::Aim => {
                if self.charging {
                    self.power = 1f64.min(self.power + dt * 0.85);
                }
                if !self.b.multiplayer && self.current == 1 {
                    self.ai_delay -= dt;
                    if self.ai_delay <= 0.0 {
                        self.ai_shoot();
                    }
                }
            }
            Phase::Place => {}
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        if self.state == State::Setup {
            self.draw_setup(&mut s);
            return;
        }
        if self.view != "2d" {
            self.basis = self.cam_basis();
        }
        self.draw_table(&mut s);
        self.draw_balls(&mut s);
        if matches!(self.phase, Phase::Aim | Phase::Place) && self.human_turn() {
            self.draw_aim(&mut s);
        }
        self.draw_hud(&mut s);
        if self.state == State::Over {
            self.draw_over(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "billiard",
    class_name: "BilliardGame",
    supports_multiplayer: true,
    modes: &[],
    make: BilliardGame::new,
};
