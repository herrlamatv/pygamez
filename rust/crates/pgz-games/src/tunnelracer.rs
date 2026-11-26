//! tunnelracer.rs
//! ==============
//! Tunnel Racer - Neon-3D-Roehrenflug.
//!
//! Uebertragen aus `games/tunnelracer.py`: Modi ENDLOS und LEVEL (30 feste,
//! seed-generierte Strecken), kurvige Quadrat-Roehre in Software-3D
//! (Kamerabasis, Projektion, Near-Clip, Nebel), Hindernisse und Muenzen,
//! Tasten- oder Maussteuerung, einstellbare Bewegungsunschaerfe.

use pgz_core::store;
use pgz_engine::prelude::*;
use serde_json::{Map, Value};

const NEAR: f64 = 0.12;
const FOV_MUL: f64 = 1.04;
const H: f64 = 3.0;
const SHIP_AHEAD: f64 = 4.5;
const SHIP_W: f64 = 0.45;
const SHIP_H: f64 = 0.35;
const VIEW_DIST: f64 = 120.0;
const LEVELS: i32 = 30;

type C3 = (i32, i32, i32);
type V3 = (f64, f64, f64);
const COL_BG_TOP: C3 = (8, 6, 24);
const COL_BG_BOT: C3 = (24, 10, 48);
const COL_RING: C3 = (90, 240, 255);
const COL_RING_HI: C3 = (255, 80, 220);
const COL_RAIL: C3 = (150, 90, 255);
const COL_OBST: C3 = (200, 40, 120);
const COL_OBST_EDGE: C3 = (255, 120, 190);
const COL_COIN: C3 = (255, 210, 80);

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Ready,
    Play,
    Crash,
    Finish,
}

struct LevelDef {
    length: f64,
    base: f64,
    gap: f64,
    ring_p: f64,
    amp: f64,
}

fn level_def(i: i32) -> LevelDef {
    let fi = i as f64;
    LevelDef {
        length: (900 + i * 140) as f64,
        base: 55.0f64.min(24.0 + fi * 1.1),
        gap: 11.0f64.max(24.0 - fi * 0.45),
        ring_p: 0.35f64.min(0.08 + fi * 0.01),
        amp: 2.4f64.min(0.5 + fi * 0.07),
    }
}

#[derive(Clone, Copy)]
struct Obst {
    z: f64,
    cx: f64,
    cy: f64,
    hw: f64,
    hh: f64,
}

#[derive(Clone, Copy)]
struct Coin {
    z: f64,
    cx: f64,
    cy: f64,
    taken: bool,
}

/// `TunnelRacerGame`
pub struct TunnelRacerGame {
    b: GameBase,
    control: &'static str,
    blur: f64,
    invert: bool,
    cursor: i32,
    small: Font,
    tiny: Font,
    big: Font,
    huge: Font,
    lv_font: Font,
    sky: Option<((i32, i32), Surface)>,
    prev_frame: Option<Surface>,
    capture: bool,
    solved: Vec<i32>,
    level: i32,
    state: State,
    ctrl_rect: Rect,
    blur_minus: Rect,
    blur_plus: Rect,
    blur_box: Rect,
    lv_cell: i32,
    lv_x: i32,
    lv_y: i32,
    start_rect: Rect,
    length: Option<f64>,
    base_speed: f64,
    gap: f64,
    ring_p: f64,
    amp: f64,
    rng: py::Random,
    z: f64,
    px: f64,
    py: f64,
    vx: f64,
    vy: f64,
    cam_x: f64,
    cam_y: f64,
    coins: i64,
    run_score: i64,
    elapsed: f64,
    obstacles: Vec<Obst>,
    coin_list: Vec<Coin>,
    next_spawn: f64,
    ready_t: f64,
    flash_t: f64,
    keys: [bool; 4],
    time_bonus: i64,
    coin_bonus: i64,
    // Kamera fuer das aktuelle Bild
    scx: f64,
    scy: f64,
    f: f64,
    cam: V3,
    basis: (V3, V3, V3),
}

impl TunnelRacerGame {
    /// `TunnelRacerGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = TunnelRacerGame {
            b,
            control: "keys",
            blur: 0.35,
            invert: false,
            cursor: 1,
            small: f.clone(),
            tiny: f.clone(),
            big: f.clone(),
            huge: f.clone(),
            lv_font: f,
            sky: None,
            prev_frame: None,
            capture: false,
            solved: Vec::new(),
            level: 1,
            state: State::Setup,
            ctrl_rect: z,
            blur_minus: z,
            blur_plus: z,
            blur_box: z,
            lv_cell: 20,
            lv_x: 0,
            lv_y: 0,
            start_rect: z,
            length: None,
            base_speed: 26.0,
            gap: 26.0,
            ring_p: 0.10,
            amp: 0.0,
            rng: py::Random::new(0),
            z: 0.0,
            px: 0.0,
            py: 0.0,
            vx: 0.0,
            vy: 0.0,
            cam_x: 0.0,
            cam_y: 0.0,
            coins: 0,
            run_score: 0,
            elapsed: 0.0,
            obstacles: Vec::new(),
            coin_list: Vec::new(),
            next_spawn: 40.0,
            ready_t: 0.0,
            flash_t: 0.0,
            keys: [false; 4],
            time_bonus: 0,
            coin_bonus: 0,
            scx: 0.0,
            scy: 0.0,
            f: 1.0,
            cam: (0.0, 0.0, 0.0),
            basis: ((1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0)),
        };
        g.reset();
        Box::new(g)
    }

    fn levels(&self) -> bool {
        self.b.mode == "levels"
    }

    fn make_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::f(13.max(h / 30));
        self.tiny = ui::f(11.max(h / 36));
        self.big = ui::fb(16.max(h / 21));
        self.huge = ui::fb(26.max(h / 11));
    }

    fn dim(&self, s: &mut Surface, c: C3, alpha: i32) {
        let mut surf = pg::surface(self.b.width, self.b.height);
        surf.fill_c(rgb(c.0, c.1, c.2));
        surf.set_alpha(Some(alpha.clamp(0, 255) as u8));
        s.bl(&surf, (0, 0));
    }

    fn save_setting(&mut self, key: &str, value: Value) {
        self.b.set_opt("tunnel", key, value);
    }

    fn load_solved(&mut self) {
        let data = store::load_section("tunnel");
        let mut out: Vec<i32> = Vec::new();
        if let Some(Value::Array(lst)) = data.get("solved") {
            for v in lst {
                let n = match v {
                    Value::Number(n) if n.is_i64() || n.is_u64() => n.as_i64(),
                    Value::Bool(b) => Some(*b as i64),
                    _ => None,
                };
                if let Some(n) = n.filter(|n| (1..=LEVELS as i64).contains(n)) {
                    if !out.contains(&(n as i32)) {
                        out.push(n as i32);
                    }
                }
            }
        }
        out.sort();
        self.solved = out;
    }

    fn mark_solved(&mut self, n: i32) {
        if !self.solved.contains(&n) {
            self.solved.push(n);
            self.solved.sort();
            let mut sec = Map::new();
            sec.insert("solved".into(), Value::from(self.solved.clone()));
            store::save_section("tunnel", sec);
        }
    }

    fn build_setup_layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let y0 = (h as f64 * 0.24) as i32;
        let rx = cx + 24;
        self.ctrl_rect = Rect::new(rx - 60, y0, 220, 40);
        self.blur_minus = Rect::new(rx - 60, y0 + 52, 44, 40);
        self.blur_plus = Rect::new(rx + 116, y0 + 52, 44, 40);
        self.blur_box = Rect::new(rx - 8, y0 + 52, 116, 40);
        if self.levels() {
            let top = y0 + 108;
            let avail_h = h - top - 96;
            let cell = 20.max(py::fdiv32(w - 80, 10).min(py::fdiv32(avail_h, 3)));
            self.lv_cell = cell;
            self.lv_x = cx - cell * 5;
            self.lv_y = top;
            self.lv_font = ui::f(10.max(py::fdiv32(cell * 2, 5)));
            self.start_rect = Rect::new(cx - 95, top + 3 * cell + 14, 190, 46);
        } else {
            self.start_rect = Rect::new(cx - 95, y0 + 124, 190, 46);
        }
    }

    fn level_at(&self, pos: (i32, i32)) -> Option<i32> {
        let c = py::fdiv32(pos.0 - self.lv_x, self.lv_cell);
        let r = py::fdiv32(pos.1 - self.lv_y, self.lv_cell);
        if (0..10).contains(&c) && (0..3).contains(&r) {
            Some(r * 10 + c + 1)
        } else {
            None
        }
    }

    fn cycle_blur(&mut self) {
        self.blur = if self.blur >= 0.79 { 0.0 } else { py::round_to(self.blur + 0.1, 1) };
        self.save_setting("blur", self.blur.into());
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        let lv = self.levels();
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if matches!(k, "c" | "C") {
                self.toggle_control();
            } else if matches!(k, "i" | "I") {
                self.toggle_invert();
            } else if matches!(k, "b" | "B") {
                self.cycle_blur();
                self.b.play_sound("select");
            } else if lv && matches!(k, "Left" | "a" | "A") {
                self.cursor = py::pmod32(self.cursor - 2, LEVELS) + 1;
                self.b.play_sound("move");
            } else if lv && matches!(k, "Right" | "d" | "D") {
                self.cursor = py::pmod32(self.cursor, LEVELS) + 1;
                self.b.play_sound("move");
            } else if lv && matches!(k, "Up" | "w" | "W") {
                self.cursor = py::pmod32(self.cursor - 11, LEVELS) + 1;
                self.b.play_sound("move");
            } else if lv && matches!(k, "Down" | "s" | "S") {
                self.cursor = py::pmod32(self.cursor + 9, LEVELS) + 1;
                self.b.play_sound("move");
            } else if matches!(k, "Return" | "space") {
                self.start_run(if lv { Some(self.cursor) } else { None });
            }
        } else if ev.kind == Kind::MouseDown {
            if self.ctrl_rect.hit(ev.pos) {
                self.toggle_control();
                return;
            }
            if self.blur_minus.hit(ev.pos) {
                self.blur = py::round_to(0f64.max(self.blur - 0.1), 1);
                self.save_setting("blur", self.blur.into());
                self.b.play_sound("select");
                return;
            }
            if self.blur_plus.hit(ev.pos) {
                self.blur = py::round_to(0.8f64.min(self.blur + 0.1), 1);
                self.save_setting("blur", self.blur.into());
                self.b.play_sound("select");
                return;
            }
            if lv {
                if let Some(l) = self.level_at(ev.pos) {
                    self.start_run(Some(l));
                    return;
                }
            }
            if self.start_rect.hit(ev.pos) {
                self.start_run(if lv { Some(self.cursor) } else { None });
            }
        }
    }

    fn toggle_invert(&mut self) {
        self.invert = !self.invert;
        self.save_setting("mouse_invert", self.invert.into());
        self.b.play_sound("select");
    }

    fn toggle_control(&mut self) {
        self.control = if self.control == "keys" { "mouse" } else { "keys" };
        self.save_setting("control", self.control.into());
        self.b.play_sound("select");
    }

    fn start_run(&mut self, level: Option<i32>) {
        self.level = level.unwrap_or(1);
        if self.levels() {
            self.cursor = self.level;
            self.save_setting("last_level", (self.level as i64).into());
            let d = level_def(self.level);
            self.length = Some(d.length);
            self.base_speed = d.base;
            self.gap = d.gap;
            self.ring_p = d.ring_p;
            self.amp = d.amp;
            self.rng = py::Random::new((7700 + self.level) as u64);
        } else {
            self.length = None;
            self.base_speed = 26.0;
            self.gap = 26.0;
            self.ring_p = 0.10;
            self.amp = 0.0;
            self.rng = py::Random::unseeded();
        }
        self.z = 0.0;
        self.px = 0.0;
        self.py = 0.0;
        self.vx = 0.0;
        self.vy = 0.0;
        self.cam_x = 0.0;
        self.cam_y = 0.0;
        self.coins = 0;
        self.run_score = 0;
        self.elapsed = 0.0;
        self.obstacles.clear();
        self.coin_list.clear();
        self.next_spawn = 40.0;
        self.ready_t = 1.2;
        self.flash_t = 0.0;
        self.prev_frame = None;
        self.keys = [false; 4];
        self.state = State::Ready;
        self.capture = false;
        self.b.play_sound("level");
    }

    fn amp_now(&self) -> f64 {
        if self.levels() {
            self.amp
        } else {
            2.2f64.min(self.z / 1500.0 * 2.2)
        }
    }

    fn center(&self, z: f64) -> (f64, f64) {
        let a1 = self.amp_now();
        let a2 = 0.4 * a1;
        (a1 * (z * 0.021).sin() + a2 * (z * 0.043 + 1.7).sin(), 0.5 * a1 * (z * 0.017 + 0.9).sin())
    }

    fn gap_now(&self) -> f64 {
        if self.levels() {
            self.gap
        } else {
            26.0 - (26.0 - 11.0) * 1.0f64.min(self.z / 6000.0)
        }
    }

    fn gen_ahead(&mut self) {
        let horizon = self.z + 160.0;
        while self.next_spawn < horizon {
            if let Some(len) = self.length {
                if self.next_spawn > len - 30.0 {
                    break;
                }
            }
            let zs = self.next_spawn;
            let kind_roll = self.rng.random();
            let ring_p = if self.levels() { self.ring_p } else { 0.35f64.min(0.08 + self.z / 20000.0) };
            let free_cell;
            if kind_roll < ring_p {
                let hx = self.rng.uniform(-H + 1.0, H - 1.0);
                let hy = self.rng.uniform(-H + 1.0, H - 1.0);
                let hole = 1.4;
                self.obstacles.extend([
                    Obst { z: zs, cx: 0.0, cy: (hy - hole / 2.0 - H) / 2.0 + 0.0, hw: H, hh: (hy - hole / 2.0 + H) / 2.0 },
                    Obst { z: zs, cx: 0.0, cy: (hy + hole / 2.0 + H) / 2.0, hw: H, hh: (H - hy - hole / 2.0) / 2.0 },
                    Obst { z: zs, cx: (hx - hole / 2.0 - H) / 2.0, cy: hy, hw: (hx - hole / 2.0 + H) / 2.0, hh: hole / 2.0 },
                    Obst { z: zs, cx: (hx + hole / 2.0 + H) / 2.0, cy: hy, hw: (H - hx - hole / 2.0) / 2.0, hh: hole / 2.0 },
                ]);
                free_cell = (hx, hy);
            } else if kind_roll < ring_p + 0.30 {
                let top = self.rng.random() < 0.5;
                let cy = if top { -H + 0.45 * H } else { H - 0.45 * H };
                self.obstacles.push(Obst { z: zs, cx: 0.0, cy, hw: H, hh: 0.45 * H });
                free_cell = (0.0, if top { H * 0.5 } else { -H * 0.5 });
            } else if kind_roll < ring_p + 0.55 {
                let left = self.rng.random() < 0.5;
                let cx = if left { -H / 2.0 } else { H / 2.0 };
                self.obstacles.push(Obst { z: zs, cx, cy: 0.0, hw: H / 2.0, hh: H });
                free_cell = (if left { H / 2.0 } else { -H / 2.0 }, 0.0);
            } else {
                let gx = self.rng.randint(-1, 1);
                let gy = self.rng.randint(-1, 1);
                self.obstacles.push(Obst { z: zs, cx: gx as f64 * 2.0, cy: gy as f64 * 2.0, hw: 0.6, hh: 0.6 });
                free_cell = ((py::pmod(gx + 2, 3) - 1) as f64 * 2.0, gy as f64 * 2.0);
            }
            if self.rng.random() < 0.6 {
                let n = self.rng.randint(3, 5);
                for i in 0..n {
                    self.coin_list.push(Coin { z: zs + 6.0 + i as f64 * 4.0, cx: free_cell.0, cy: free_cell.1, taken: false });
                }
            }
            self.next_spawn += self.gap_now() * self.rng.uniform(0.8, 1.2);
        }
        let lim = self.z - 10.0;
        self.obstacles.retain(|o| o.z > lim);
        self.coin_list.retain(|c| c.z > lim);
    }

    fn crash(&mut self) {
        self.b.play_sound("explode");
        self.b.rumble(220);
        self.flash_t = 0.15;
        self.capture = false;
        self.state = State::Crash;
        if !self.levels() {
            self.b.game_over = true;
        }
    }

    fn finish(&mut self) {
        self.capture = false;
        let len = self.length.unwrap_or(1.0);
        let par = len / self.base_speed * 1.25;
        self.time_bonus = 0.max(((par - self.elapsed) * 20.0) as i64);
        self.coin_bonus = self.coins * 50;
        self.run_score = 1000 + self.coin_bonus + self.time_bonus;
        self.b.score += self.run_score;
        self.mark_solved(self.level);
        self.state = State::Finish;
        self.b.play_sound("win");
    }

    fn tangent(&self, z: f64) -> (f64, f64) {
        let c0 = self.center(z);
        let c1 = self.center(z + 8.0);
        ((c1.0 - c0.0).atan2(8.0), (c1.1 - c0.1).atan2(8.0))
    }

    fn basis_at(&self, z: f64) -> (V3, V3, V3) {
        let (yaw, pitch) = self.tangent(z);
        let cp = pitch.cos();
        let f = (yaw.sin() * cp, pitch.sin(), yaw.cos() * cp);
        let mut rl = py::hypot(f.0, f.2);
        if rl == 0.0 {
            rl = 1.0;
        }
        let r = (f.2 / rl, 0.0, -f.0 / rl);
        let u = (f.1 * r.2 - f.2 * r.1, f.2 * r.0 - f.0 * r.2, f.0 * r.1 - f.1 * r.0);
        (r, u, f)
    }

    fn world(&self, z: f64, ox: f64, oy: f64) -> V3 {
        let (cx, cy) = self.center(z);
        (cx + ox, cy + oy, z)
    }

    fn to_cam(&self, p: V3) -> V3 {
        let (r, u, f) = self.basis;
        let dx = p.0 - self.cam.0;
        let dy = p.1 - self.cam.1;
        let dz = p.2 - self.cam.2;
        (dx * r.0 + dy * r.1 + dz * r.2, dx * u.0 + dy * u.1 + dz * u.2, dx * f.0 + dy * f.1 + dz * f.2)
    }

    fn proj(&self, c: V3) -> (f64, f64) {
        let k = self.f / c.2;
        (self.scx + c.0 * k, self.scy - c.1 * k)
    }

    fn clip_seg(a: V3, b: V3) -> Option<(V3, V3)> {
        let (da, db) = (a.2 - NEAR, b.2 - NEAR);
        if da < 0.0 && db < 0.0 {
            return None;
        }
        if da < 0.0 || db < 0.0 {
            let tt = da / (da - db);
            let m = (a.0 + (b.0 - a.0) * tt, a.1 + (b.1 - a.1) * tt, NEAR);
            return Some(if da < 0.0 { (m, b) } else { (a, m) });
        }
        Some((a, b))
    }

    fn fog(col: C3, depth: f64) -> Color {
        let f = 1.0f64.min(0f64.max(depth / VIEW_DIST));
        let m = |c: i32, bg: i32| (c as f64 + (bg - c) as f64 * f) as i32;
        rgb(m(col.0, COL_BG_BOT.0), m(col.1, COL_BG_BOT.1), m(col.2, COL_BG_BOT.2))
    }

    fn ipt(p: (f64, f64)) -> (i32, i32) {
        (p.0 as i32, p.1 as i32)
    }

    fn draw_sky(&mut self, s: &mut Surface) {
        let (w, h) = (self.b.width, self.b.height);
        if self.sky.as_ref().map(|c| c.0) != Some((w, h)) {
            let mut surf = pg::surface(w, h);
            for y in 0..h {
                let f = y as f64 / 1.max(h - 1) as f64;
                let m = |a: i32, b: i32| (a as f64 + (b - a) as f64 * f) as i32;
                surf.fill_r(rgb(m(COL_BG_TOP.0, COL_BG_BOT.0), m(COL_BG_TOP.1, COL_BG_BOT.1), m(COL_BG_TOP.2, COL_BG_BOT.2)), (0, y, w, 1));
            }
            self.sky = Some(((w, h), surf));
        }
        if let Some((_, surf)) = &self.sky {
            s.bl(surf, (0, 0));
        }
    }

    fn ring_pts(&self, z: f64) -> [V3; 4] {
        [(-H, -H), (H, -H), (H, H), (-H, H)].map(|(ox, oy)| self.to_cam(self.world(z, ox, oy)))
    }

    fn draw_tunnel(&self, s: &mut Surface) {
        let z0 = (self.z / 6.0).floor() * 6.0;
        let rings: Vec<(f64, [V3; 4])> = (0..(VIEW_DIST / 6.0) as i32 + 1)
            .map(|i| {
                let z = z0 + i as f64 * 6.0;
                (z, self.ring_pts(z))
            })
            .collect();
        for pair in rings.windows(2) {
            let (a, b) = (&pair[0].1, &pair[1].1);
            for k in 0..4 {
                if let Some(seg) = Self::clip_seg(a[k], b[k]) {
                    let depth = (seg.0 .2 + seg.1 .2) / 2.0;
                    draw::line(s, Self::fog(COL_RAIL, depth), Self::ipt(self.proj(seg.0)), Self::ipt(self.proj(seg.1)), 1);
                }
            }
        }
        for (z, pts) in &rings {
            let hi = py::pmod((z / 6.0) as i64, 4) == 0;
            let col = if hi { COL_RING_HI } else { COL_RING };
            for k in 0..4 {
                if let Some(seg) = Self::clip_seg(pts[k], pts[(k + 1) % 4]) {
                    let depth = (seg.0 .2 + seg.1 .2) / 2.0;
                    draw::line(s, Self::fog(col, depth), Self::ipt(self.proj(seg.0)), Self::ipt(self.proj(seg.1)), if hi { 2 } else { 1 });
                }
            }
        }
    }

    fn draw_obstacles(&self, s: &mut Surface) {
        let mut items: Vec<(f64, Vec<(i32, i32)>)> = Vec::new();
        for o in &self.obstacles {
            let rel = o.z - self.z;
            if !(NEAR..=VIEW_DIST).contains(&rel) {
                continue;
            }
            let corners: Vec<V3> =
                [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)].iter().map(|(sx, sy)| self.to_cam(self.world(o.z, o.cx + sx * o.hw, o.cy + sy * o.hh))).collect();
            if corners.iter().all(|c| c.2 <= NEAR) {
                continue;
            }
            let depth = corners.iter().map(|c| c.2).sum::<f64>() / 4.0;
            let pts = corners.iter().map(|c| Self::ipt(self.proj(*c))).collect();
            items.push((depth, pts));
        }
        items.sort_by(|a, b| (-a.0).partial_cmp(&(-b.0)).unwrap_or(std::cmp::Ordering::Equal));
        for (depth, pts) in &items {
            draw::polygon(s, Self::fog(COL_OBST, *depth), pts, 0);
            draw::polygon(s, Self::fog(COL_OBST_EDGE, *depth), pts, 2);
        }
    }

    fn draw_coins(&self, s: &mut Surface) {
        let tick = pg::ticks() as f64 / 1000.0;
        for c in &self.coin_list {
            if c.taken {
                continue;
            }
            let rel = c.z - self.z;
            if !(NEAR..=VIEW_DIST).contains(&rel) {
                continue;
            }
            let cam = self.to_cam(self.world(c.z, c.cx, c.cy));
            if cam.2 < NEAR {
                continue;
            }
            let k = self.f / cam.2;
            let x = (self.scx + cam.0 * k) as i32;
            let y = (self.scy - cam.1 * k) as i32;
            let r = 2.max((k * 0.35 * (1.0 + 0.15 * (tick * 4.0).sin())) as i32);
            draw::circle(s, Self::fog(COL_COIN, cam.2), (x, y), r, 0);
            draw::circle(s, Self::fog((255, 240, 180), cam.2), (x, y), r, 1.max(r / 4));
        }
    }

    fn draw_ship(&self, s: &mut Surface) {
        let z = self.z + SHIP_AHEAD;
        let pts3 = [
            self.world(z, self.px, self.py - SHIP_H),
            self.world(z - 0.9, self.px - SHIP_W, self.py + SHIP_H),
            self.world(z - 0.5, self.px, self.py + SHIP_H * 0.4),
            self.world(z - 0.9, self.px + SHIP_W, self.py + SHIP_H),
        ];
        let cams: Vec<V3> = pts3.iter().map(|p| self.to_cam(*p)).collect();
        if cams.iter().any(|c| c.2 <= NEAR) {
            return;
        }
        let ipts: Vec<(i32, i32)> = cams.iter().map(|c| Self::ipt(self.proj(*c))).collect();
        draw::polygon(s, self.b.accent, &ipts, 0);
        draw::polygon(s, rgb(240, 250, 255), &ipts, 2);
    }

    fn apply_blur(&mut self, s: &mut Surface) {
        if self.blur <= 0.01 {
            self.prev_frame = None;
            return;
        }
        let same = self.prev_frame.as_ref().is_some_and(|p| p.get_size() == s.get_size());
        if !same {
            self.prev_frame = Some(s.copy());
            return;
        }
        if let Some(prev) = self.prev_frame.as_mut() {
            prev.set_alpha(Some((self.blur * 230.0) as i32 as u8));
            s.bl(prev, (0, 0));
        }
        self.prev_frame = Some(s.copy());
    }

    fn speed_endless(&self) -> f64 {
        75.0f64.min(26.0 + 10.0 * (self.z / 1000.0))
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let img = self.big.render(&tf("common.points", &[("score", &self.b.score.to_string())]), true, self.b.accent);
        s.bl(&img, (14, 10));
        let mut right = vec![tf("tun.coins", &[("n", &self.coins.to_string())])];
        let len = self.length.unwrap_or(1.0);
        if !self.levels() {
            let v = self.speed_endless();
            right.push(tf("tun.distance", &[("n", &(self.z as i64).to_string())]));
            right.push(tf("tun.speed", &[("v", &((v * 3.6) as i64).to_string())]));
        } else {
            right.insert(0, tf("tun.level", &[("n", &self.level.to_string())]));
            let pct = (100.0 * self.z / 1.0f64.max(len)) as i64;
            right.push(format!("{}%", pct.min(100)));
        }
        let mut y = 12;
        for line in &right {
            let img = self.small.render(line, true, p.text_dim);
            s.blr(&img, img.get_rect().topright_at((w - 14, y)));
            y += 22;
        }
        if self.levels() && self.state == State::Play {
            let frac = 1.0f64.min(self.z / 1.0f64.max(len));
            draw::rect(s, p.panel_light, (0, h - 6, w, 6), 0);
            draw::rect(s, self.b.accent, (0, h - 6, (w as f64 * frac) as i32, 6), 0);
        }
    }

    fn draw_crash(&self, s: &mut Surface) {
        let p = ui::pal();
        self.dim(s, (10, 6, 20), 175);
        let (cx, cy) = (self.b.width / 2, self.b.height / 2);
        let head = self.huge.render(&t("tun.crash"), true, p.red);
        let sc = self.b.font.render(&tf("common.points", &[("score", &self.b.score.to_string())]), true, p.text);
        let key = if !self.levels() { "common.enter_restart" } else { "tun.retry" };
        let hint = self.small.render(&t(key), true, p.text_dim);
        let top = cy - 50 - head.h() / 2 - 22;
        let bottom = cy + 40 + hint.h() / 2 + 22;
        let pw = (self.b.width - 40).min(380.max(head.w() + 80).max(hint.w() + 60).max(sc.w() + 60));
        let panel = Rect::new(cx - py::fdiv32(pw, 2), top, pw, bottom - top);
        draw::rect_r(s, p.panel, panel, 0, 14);
        draw::rect_r(s, p.border_light, panel, 1, 14);
        s.blr(&head, head.get_rect().center_at((cx, cy - 50)));
        s.blr(&sc, sc.get_rect().center_at((cx, cy + 2)));
        s.blr(&hint, hint.get_rect().center_at((cx, cy + 40)));
    }

    fn draw_finish(&self, s: &mut Surface) {
        let p = ui::pal();
        self.dim(s, (6, 12, 24), 185);
        let (cx, cy) = (self.b.width / 2, self.b.height / 2);
        let head = self.huge.render(&tf("tun.level_done", &[("n", &self.level.to_string())]), true, self.b.accent);
        let lines = [
            tf("tun.base", &[("n", "1000")]),
            tf("tun.coin_bonus", &[("n", &self.coin_bonus.to_string())]),
            tf("tun.time_bonus", &[("n", &self.time_bonus.to_string())]),
            tf("common.points", &[("score", &self.b.score.to_string())]),
        ];
        let imgs: Vec<Surface> = lines.iter().map(|l| self.b.font.render(l, true, p.text)).collect();
        let hint = self.small.render(&t("tun.next"), true, p.text_dim);
        let top = cy - 70 - head.h() / 2 - 22;
        let bottom = cy - 20 + 30 * imgs.len() as i32 + 12 + hint.h() / 2 + 22;
        let widest = imgs.iter().map(|i| i.w()).max().unwrap_or(0);
        let pw = (self.b.width - 40).min(400.max(head.w() + 80).max(hint.w() + 60).max(widest + 60));
        let panel = Rect::new(cx - py::fdiv32(pw, 2), top, pw, bottom - top);
        draw::rect_r(s, p.panel, panel, 0, 14);
        draw::rect_r(s, p.border_light, panel, 1, 14);
        s.blr(&head, head.get_rect().center_at((cx, cy - 70)));
        let mut y = cy - 20;
        for img in &imgs {
            s.blr(img, img.get_rect().center_at((cx, y)));
            y += 30;
        }
        s.blr(&hint, hint.get_rect().center_at((cx, y + 12)));
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        ui::draw_background(s, w, h);
        let cx = w / 2;
        let title = self.huge.render("TUNNEL RACER", true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.10) as i32)));
        let mode_lbl = t(&format!("tun.mode.{}", self.b.mode));
        let sub = self.small.render(&format!("{mode_lbl}   -   {}", t("tun.subtitle")), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, (h as f64 * 0.17) as i32)));
        let lbl = self.small.render(&t("tun.control"), true, p.text_dim);
        s.blr(&lbl, lbl.get_rect().midright_at((self.ctrl_rect.x - 16, self.ctrl_rect.centery())));
        draw::rect_r(s, p.btn_sel, self.ctrl_rect, 0, 8);
        draw::rect_r(s, p.border, self.ctrl_rect, 1, 8);
        let img = self.small.render(&format!("{}  [C]", t(&format!("tun.control.{}", self.control))), true, p.text);
        s.blr(&img, img.get_rect().center_at(self.ctrl_rect.center()));
        let lbl = self.small.render(&t("tun.blur"), true, p.text_dim);
        s.blr(&lbl, lbl.get_rect().midright_at((self.blur_minus.x - 16, self.blur_minus.centery())));
        for (r, sym) in [(self.blur_minus, "-"), (self.blur_plus, "+")] {
            draw::rect_r(s, p.btn, r, 0, 8);
            draw::rect_r(s, p.border, r, 1, 8);
            let img = self.big.render(sym, true, p.text);
            s.blr(&img, img.get_rect().center_at(r.center()));
        }
        let blur_lbl = if self.blur <= 0.0 { t("common.off") } else { format!("{}%", (self.blur * 100.0) as i64) };
        let img = self.big.render(&blur_lbl, true, self.b.accent);
        s.blr(&img, img.get_rect().center_at(self.blur_box.center()));
        let mdir = if self.invert { t("common.dir_inverted") } else { t("common.dir_normal") };
        let hint = self.small.render(&tf("tun.mousedir", &[("dir", &mdir)]), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, h - 18)));
        if self.levels() {
            let done_fill = ui::mix(p.btn, self.b.accent, 0.25);
            let done_text = ui::mix(self.b.accent, p.text, 0.35);
            for n in 1..=LEVELS {
                let i = n - 1;
                let x = self.lv_x + (i % 10) * self.lv_cell;
                let y = self.lv_y + (i / 10) * self.lv_cell;
                let cell = Rect::new(x + 1, y + 1, self.lv_cell - 2, self.lv_cell - 2);
                let done = self.solved.contains(&n);
                draw::rect_r(s, if done { done_fill } else { p.btn }, cell, 0, 4);
                if n == self.cursor {
                    draw::rect_r(s, self.b.accent, cell, 2, 4);
                }
                let num = self.lv_font.render(&n.to_string(), true, if done { done_text } else { p.text_dim });
                s.blr(&num, num.get_rect().center_at(cell.center()));
            }
            let prog = self.small.render(&tf("tun.progress", &[("n", &self.solved.len().to_string()), ("m", &LEVELS.to_string())]), true, p.text_dim);
            s.blr(&prog, prog.get_rect().center_at((cx, self.lv_y + 3 * self.lv_cell + 40)));
        }
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 10);
        draw::rect_r(s, self.b.accent, self.start_rect, 2, 10);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
    }

    fn key_acts(&self, k: &str) -> Vec<usize> {
        ["up", "down", "left", "right"]
            .iter()
            .enumerate()
            .filter(|(_, act)| {
                let cap = match **act {
                    "up" => "Up",
                    "down" => "Down",
                    "left" => "Left",
                    _ => "Right",
                };
                self.b.is_action(k, act, None) || k == cap
            })
            .map(|(i, _)| i)
            .collect()
    }
}

impl Game for TunnelRacerGame {
    /// Match-Historie: Dauer zaehlt nicht im Setup (Python-Standard
    /// `getattr(self, "state", None) != "setup"`).
    fn match_active(&self) -> bool {
        !matches!(self.state, State::Setup)
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
        let c = self.b.opt_str("tunnel", "control", "keys");
        self.control = if c == "mouse" { "mouse" } else { "keys" };
        self.blur = self.b.opt_float_py("tunnel", "blur", 0.35).map(|v| v.clamp(0.0, 0.8)).unwrap_or(0.35);
        self.invert = self.b.opt_bool("tunnel", "mouse_invert", false);
        self.cursor = self.b.opt_int_py("tunnel", "last_level", 1).map(|v| v.clamp(1, LEVELS as i64) as i32).unwrap_or(1);
        self.make_fonts();
        self.sky = None;
        self.prev_frame = None;
        self.capture = false;
        self.load_solved();
        self.level = 1;
        self.build_setup_layout();
        self.state = State::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.make_fonts();
        self.sky = None;
        self.prev_frame = None;
        self.build_setup_layout();
    }

    fn capture_mouse(&self) -> bool {
        self.capture
    }

    fn handle_event(&mut self, ev: &InputEvent) {
        match self.state {
            State::Setup => {
                self.handle_setup(ev);
                return;
            }
            State::Crash => {
                if ev.kind == Kind::KeyDown {
                    if !self.levels() {
                        if ev.key_in(&["Return", "space"]) {
                            self.b.game_over = false;
                            self.reset();
                        }
                    } else if ev.key_in(&["r", "R", "Return", "space"]) {
                        self.start_run(Some(self.level));
                    } else if ev.key_in(&["s", "S"]) {
                        self.state = State::Setup;
                        self.capture = false;
                        self.build_setup_layout();
                    }
                }
                return;
            }
            State::Finish => {
                if ev.kind == Kind::KeyDown {
                    if ev.key_in(&["Return", "space"]) {
                        self.start_run(Some(LEVELS.min(self.level + 1)));
                    } else if ev.key_in(&["s", "S"]) {
                        self.state = State::Setup;
                        self.build_setup_layout();
                    }
                }
                return;
            }
            _ => {}
        }
        match ev.kind {
            Kind::KeyDown => {
                let k = ev.key.as_str();
                for i in self.key_acts(k) {
                    self.keys[i] = true;
                }
                if matches!(k, "b" | "B") {
                    self.cycle_blur();
                }
                if matches!(k, "i" | "I") {
                    self.toggle_invert();
                }
            }
            Kind::KeyUp => {
                for i in self.key_acts(&ev.key) {
                    self.keys[i] = false;
                }
            }
            Kind::MouseRel if self.control == "mouse" => {
                let hx = if self.invert { -1.0 } else { 1.0 };
                let hy = if self.invert { 1.0 } else { -1.0 };
                self.px += ev.rel.0 as f64 * 0.011 * H * hx;
                self.py += ev.rel.1 as f64 * 0.011 * H * hy;
            }
            _ => {}
        }
    }

    fn update(&mut self, dt: f64) {
        if self.state == State::Ready {
            self.ready_t -= dt;
            if self.ready_t <= 0.0 {
                self.state = State::Play;
                self.capture = self.control == "mouse";
            }
            return;
        }
        if self.state != State::Play || self.b.game_over {
            return;
        }
        self.elapsed += dt;
        let v = if !self.levels() {
            let v = self.speed_endless();
            if v >= 75.0 {
                self.b.ach_event("tunnel_far", None);
            }
            v
        } else {
            let progress = self.z / 1.0f64.max(self.length.unwrap_or(1.0));
            self.base_speed * (1.0 + 0.15 * progress)
        };
        let z_prev = self.z;
        self.z += v * dt;
        if self.control == "keys" {
            let on = |i: usize| if self.keys[i] { 1.0 } else { 0.0 };
            let ax = (on(3) - on(2)) * 14.0;
            let ay = (on(1) - on(0)) * 14.0;
            self.vx += ax * dt;
            self.vy += ay * dt;
            self.vx -= self.vx * 1.0f64.min(6.0 * dt);
            self.vy -= self.vy * 1.0f64.min(6.0 * dt);
            self.vx = (-7.0f64).max(7.0f64.min(self.vx));
            self.vy = (-7.0f64).max(7.0f64.min(self.vy));
            self.px += self.vx * dt;
            self.py += self.vy * dt;
        }
        self.px = (-(H - SHIP_W)).max((H - SHIP_W).min(self.px));
        self.py = (-(H - SHIP_H)).max((H - SHIP_H).min(self.py));
        let k = 1.0f64.min(6.0 * dt);
        self.cam_x += (self.px * 0.55 - self.cam_x) * k;
        self.cam_y += (self.py * 0.55 - self.cam_y) * k;
        self.gen_ahead();
        let z_lo = z_prev + SHIP_AHEAD - 0.8;
        let z_hi = self.z + SHIP_AHEAD + 0.8;
        let crash = self
            .obstacles
            .iter()
            .any(|o| z_lo < o.z && o.z < z_hi && (self.px - o.cx).abs() < SHIP_W + o.hw && (self.py - o.cy).abs() < SHIP_H + o.hh);
        if crash {
            self.crash();
            return;
        }
        for i in 0..self.coin_list.len() {
            let c = self.coin_list[i];
            if !c.taken && z_lo < c.z && c.z < z_hi && py::hypot(self.px - c.cx, self.py - c.cy) < 0.75 {
                self.coin_list[i].taken = true;
                self.coins += 1;
                self.b.play_sound("point");
            }
        }
        if !self.levels() {
            self.b.score = (self.z * 10.0 + (self.coins * 50) as f64) as i64;
        } else if self.z >= self.length.unwrap_or(f64::INFINITY) {
            self.finish();
        }
        if self.flash_t > 0.0 {
            self.flash_t -= dt;
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        let (w, h) = (self.b.width, self.b.height);
        self.scx = w as f64 / 2.0;
        self.scy = h as f64 / 2.0;
        self.f = h as f64 * FOV_MUL;
        if self.state == State::Setup {
            self.draw_setup(&mut s);
            return;
        }
        let (cx, cy) = self.center(self.z);
        self.cam = (cx + self.cam_x, cy + self.cam_y, self.z);
        self.basis = self.basis_at(self.z);
        self.draw_sky(&mut s);
        self.draw_tunnel(&mut s);
        self.draw_obstacles(&mut s);
        self.draw_coins(&mut s);
        self.draw_ship(&mut s);
        self.apply_blur(&mut s);
        if self.flash_t > 0.0 {
            self.dim(&mut s, (255, 255, 255), (200.0 * self.flash_t / 0.15) as i32);
        }
        self.draw_hud(&mut s);
        match self.state {
            State::Ready => {
                let img = self.huge.render(&t("tun.start_hint"), true, self.b.accent);
                s.blr(&img, img.get_rect().center_at((w / 2, (h as f64 * 0.42) as i32)));
            }
            State::Crash => self.draw_crash(&mut s),
            State::Finish => self.draw_finish(&mut s),
            _ => {}
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "tunnel",
    class_name: "TunnelRacerGame",
    supports_multiplayer: false,
    modes: &[("endless", "tun.mode.endless"), ("levels", "tun.mode.levels")],
    make: TunnelRacerGame::new,
};
