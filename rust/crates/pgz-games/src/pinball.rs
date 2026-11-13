//! pinball.rs
//! ==========
//! Pinball - Flipperautomat mit drei Tischen, Multiball, Drop-Targets,
//! Rollover-Bahnen, Bonus-Multiplikator, Ball-Save, Nudge und Tilt.
//!
//! Uebertragen aus `games/pinball.py`, samt Replay-Aufnahme je Kugel und den
//! Wiedergabe-Methoden (`replay_begin`/`replay_seek`/`replay_draw`).

use pgz_core::replay::{self, rnd, Delta, Recorder};
use pgz_core::store;
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

const COL_TABLE: Color = Color::rgb(24, 26, 40);
const COL_TABLE_D: Color = Color::rgb(18, 19, 30);
const COL_WALL: Color = Color::rgb(128, 136, 168);
const COL_WALL_HI: Color = Color::rgb(176, 186, 220);
const COL_BALL: Color = Color::rgb(226, 230, 240);
const COL_BALL_D: Color = Color::rgb(140, 146, 164);
const COL_FLIP: Color = Color::rgb(238, 196, 74);
const COL_FLIP_D: Color = Color::rgb(176, 138, 40);
const COL_BUMP: Color = Color::rgb(86, 196, 236);
const COL_BUMP_HI: Color = Color::rgb(206, 244, 255);
const COL_SLING: Color = Color::rgb(236, 108, 96);
const COL_TARGET: Color = Color::rgb(150, 236, 140);
const COL_TARGET_OFF: Color = Color::rgb(58, 82, 60);
const COL_LANE: Color = Color::rgb(208, 150, 240);
const COL_SAUCER: Color = Color::rgb(250, 214, 120);
const COL_PLUNGER: Color = Color::rgb(222, 96, 96);

const TW: f64 = 100.0;
const TH: f64 = 170.0;
const BR: f64 = 2.2;
const WALL_R: f64 = 1.0;
const FLIP_R: f64 = 1.6;
const GRAV: f64 = 118.0;
const DRAG: f64 = 0.14;
const WALL_E: f64 = 0.52;
const SLING_KICK: f64 = 118.0;
const BUMP_KICK: f64 = 96.0;
const MAX_SPEED: f64 = 320.0;
const FLIP_SPEED: f64 = 17.0;
const FLIP_LEN: f64 = 19.0;
const DRAIN_Y: f64 = 162.0;
const LANE_X: f64 = 91.0;
const BALL_SAVE: f64 = 6.0;
const TILT_LIMIT: i32 = 3;
const LANE_LETTERS: [&str; 4] = ["L", "A", "M", "A"];

const TABLES: [&str; 3] = ["classic", "space", "lama"];
const BALL_COUNTS: [i64; 2] = [3, 5];
const REC_STATE: i64 = 10;

/// Einbahn-Klappe am Ausgang der Schussbahn.
const GATE: [f64; 4] = [86.0, 52.0, 77.0, 43.0];

type Seg = [f64; 4];

#[derive(Clone)]
struct Ball {
    id: u32,
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    held: f64,
    st: f64,
    sx: f64,
    sy: f64,
}

struct Table {
    walls: Vec<Seg>,
    bumpers: Vec<[f64; 3]>,
    slings: Vec<Seg>,
    targets: Vec<[f64; 4]>,
    lanes: Vec<[f64; 3]>,
    saucer: [f64; 3],
}

/// `_arc` - Kreisbogen als Streckenzug (Winkel in Grad).
fn arc(cx: f64, cy: f64, r: f64, a0: f64, a1: f64, n: usize) -> Vec<Seg> {
    let pts: Vec<(f64, f64)> = (0..=n)
        .map(|i| {
            let a = py::radians(a0 + (a1 - a0) * i as f64 / n as f64);
            (cx + a.cos() * r, cy + a.sin() * r)
        })
        .collect();
    (0..pts.len() - 1).map(|i| [pts[i].0, pts[i].1, pts[i + 1].0, pts[i + 1].1]).collect()
}

fn base_walls() -> Vec<Seg> {
    let mut walls = arc(50.0, 52.0, 46.0, 180.0, 360.0, 22);
    walls.extend([
        [4.0, 52.0, 4.0, 110.0],
        [4.0, 110.0, 24.0, 140.0],
        [86.0, 52.0, 86.0, 110.0],
        [86.0, 110.0, 76.0, 140.0],
        [86.0, 110.0, 86.0, 152.0],
        [96.0, 40.0, 96.0, 152.0],
        [86.0, 152.0, 96.0, 152.0],
    ]);
    walls
}

fn table_data(key: &str) -> Table {
    let sl = vec![[24.0, 116.0, 38.0, 134.0], [76.0, 116.0, 62.0, 134.0]];
    let (bumpers, slings, targets, lanes, saucer, extra): (Vec<[f64; 3]>, Vec<Seg>, Vec<[f64; 4]>, Vec<[f64; 3]>, [f64; 3], Vec<Seg>) = match key {
        "space" => (
            vec![[50.0, 38.0, 5.5], [34.0, 58.0, 5.5], [66.0, 58.0, 5.5], [50.0, 76.0, 5.5]],
            sl,
            vec![[11.0, 62.0, 7.0, 6.0], [11.0, 72.0, 7.0, 6.0], [11.0, 82.0, 7.0, 6.0], [79.0, 62.0, 7.0, 6.0], [79.0, 72.0, 7.0, 6.0], [79.0, 82.0, 7.0, 6.0]],
            vec![[30.0, 28.0, 3.0], [43.0, 22.0, 3.0], [58.0, 22.0, 3.0], [71.0, 28.0, 3.0]],
            [50.0, 100.0, 4.2],
            vec![[20.0, 96.0, 32.0, 104.0], [80.0, 96.0, 68.0, 104.0]],
        ),
        "lama" => (
            vec![[30.0, 50.0, 6.5], [70.0, 50.0, 6.5]],
            vec![[24.0, 116.0, 38.0, 134.0], [76.0, 116.0, 62.0, 134.0], [34.0, 92.0, 44.0, 104.0], [66.0, 92.0, 56.0, 104.0]],
            vec![[24.0, 30.0, 7.0, 6.0], [36.0, 24.0, 7.0, 6.0], [48.0, 22.0, 7.0, 6.0], [60.0, 24.0, 7.0, 6.0], [72.0, 30.0, 7.0, 6.0], [46.0, 62.0, 8.0, 6.0]],
            vec![[16.0, 60.0, 3.0], [16.0, 72.0, 3.0], [84.0, 60.0, 3.0], [84.0, 72.0, 3.0]],
            [50.0, 84.0, 4.6],
            vec![[16.0, 96.0, 26.0, 104.0], [84.0, 96.0, 74.0, 104.0]],
        ),
        _ => (
            vec![[38.0, 62.0, 6.0], [64.0, 62.0, 6.0], [51.0, 42.0, 6.0]],
            sl,
            vec![[12.0, 66.0, 7.0, 6.0], [12.0, 76.0, 7.0, 6.0], [12.0, 86.0, 7.0, 6.0], [12.0, 96.0, 7.0, 6.0]],
            vec![[32.0, 26.0, 3.0], [44.0, 22.0, 3.0], [57.0, 22.0, 3.0], [69.0, 26.0, 3.0]],
            [50.0, 88.0, 4.2],
            vec![[24.0, 104.0, 34.0, 96.0], [76.0, 104.0, 66.0, 96.0]],
        ),
    };
    let mut walls = base_walls();
    walls.extend(extra);
    Table { walls, bumpers, slings, targets, lanes, saucer }
}

fn closest_on_seg(x1: f64, y1: f64, x2: f64, y2: f64, px: f64, py_: f64) -> (f64, f64) {
    let (dx, dy) = (x2 - x1, y2 - y1);
    let l2 = dx * dx + dy * dy;
    if l2 < 1e-9 {
        return (x1, y1);
    }
    let f = 0f64.max(1f64.min(((px - x1) * dx + (py_ - y1) * dy) / l2));
    (x1 + f * dx, y1 + f * dy)
}

/// `_hit_seg` - Kreis gegen Strecke; true bei Treffer.
fn hit_seg(b: &mut Ball, seg: &Seg, radius: f64, rest: f64, kick: f64, vel: (f64, f64)) -> bool {
    let (px, py_) = closest_on_seg(seg[0], seg[1], seg[2], seg[3], b.x, b.y);
    let (mut dx, mut dy) = (b.x - px, b.y - py_);
    let mut d = py::hypot(dx, dy);
    let rad = BR + radius;
    if d >= rad {
        return false;
    }
    if d < 1e-9 {
        dx = 0.0;
        dy = -1.0;
        d = 1.0;
    }
    let (ux, uy) = (dx / d, dy / d);
    b.x = px + ux * rad;
    b.y = py_ + uy * rad;
    let (mut rvx, mut rvy) = (b.vx - vel.0, b.vy - vel.1);
    let dot = rvx * ux + rvy * uy;
    if dot < 0.0 {
        rvx -= (1.0 + rest) * dot * ux;
        rvy -= (1.0 + rest) * dot * uy;
    }
    b.vx = rvx + vel.0 + ux * kick;
    b.vy = rvy + vel.1 + uy * kick;
    true
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
    Over,
}

/// `PinballGame`
pub struct PinballGame {
    b: GameBase,
    table_key: &'static str,
    ball_count: i64,
    rec: Option<Recorder>,
    replay: Option<Value>,
    replay_request: Option<Value>,
    rep: Option<Value>,
    rep_at: Option<(usize, usize)>,
    rep_state: Option<Vec<f64>>,
    rec_open: bool,
    rec_delta: Delta,
    rec_start: i64,
    small: Font,
    tiny: Font,
    num: Font,
    huge: Font,
    hud_h: i32,
    scale: f64,
    ox: f64,
    oy: f64,
    side_x: i32,
    table_rects: Vec<Rect>,
    ball_rects: Vec<Rect>,
    start_rect: Rect,
    best: Vec<(String, i64)>,
    state: State,
    table: Table,
    players: usize,
    scores: Vec<i64>,
    balls_left: Vec<i64>,
    player: usize,
    winner: Option<usize>,
    msg: Option<String>,
    msg_t: f64,
    balls: Vec<Ball>,
    next_id: u32,
    phase_launch: bool,
    plunger: f64,
    charging: bool,
    flip_l: f64,
    flip_r: f64,
    flip_l_up: bool,
    flip_r_up: bool,
    omega_l: f64,
    omega_r: f64,
    mult: i64,
    locks: i64,
    multiball: bool,
    jackpot: bool,
    save_t: f64,
    tilt: bool,
    nudges: i32,
    nudge_t: f64,
    shake: f64,
    targets_hit: Vec<bool>,
    lanes_hit: Vec<bool>,
    bank_clears: i64,
    flash: Vec<(String, f64)>,
}

impl PinballGame {
    /// `PinballGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = PinballGame {
            b,
            table_key: "classic",
            ball_count: 3,
            rec: None,
            replay: None,
            replay_request: None,
            rep: None,
            rep_at: None,
            rep_state: None,
            rec_open: false,
            rec_delta: Delta::new(),
            rec_start: 0,
            small: f.clone(),
            tiny: f.clone(),
            num: f.clone(),
            huge: f,
            hud_h: 44,
            scale: 1.0,
            ox: 0.0,
            oy: 0.0,
            side_x: 0,
            table_rects: Vec::new(),
            ball_rects: Vec::new(),
            start_rect: z,
            best: Vec::new(),
            state: State::Setup,
            table: table_data("classic"),
            players: 1,
            scores: vec![0],
            balls_left: vec![3],
            player: 0,
            winner: None,
            msg: None,
            msg_t: 0.0,
            balls: Vec::new(),
            next_id: 0,
            phase_launch: true,
            plunger: 0.0,
            charging: false,
            flip_l: 0.0,
            flip_r: 0.0,
            flip_l_up: false,
            flip_r_up: false,
            omega_l: 0.0,
            omega_r: 0.0,
            mult: 1,
            locks: 0,
            multiball: false,
            jackpot: false,
            save_t: 0.0,
            tilt: false,
            nudges: 0,
            nudge_t: 0.0,
            shake: 0.0,
            targets_hit: Vec::new(),
            lanes_hit: Vec::new(),
            bank_clears: 0,
            flash: Vec::new(),
        };
        g.reset();
        Box::new(g)
    }

    fn build_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::f(14.max(h / 32));
        self.tiny = ui::f(11.max(h / 42));
        self.num = ui::fb(16.max(h / 26));
        self.huge = ui::fb(24.max(h / 13));
    }

    fn layout(&mut self) {
        self.hud_h = 44;
        let avail_h = (self.b.height - self.hud_h - 10) as f64;
        self.scale = 1.1f64.max(((self.b.width - 150) as f64 / TW).min(avail_h / TH));
        self.ox = self.b.width as f64 / 2.0 - TW * self.scale / 2.0;
        self.oy = self.hud_h as f64 + (avail_h - TH * self.scale) / 2.0 + 5.0;
        self.side_x = (self.ox + TW * self.scale) as i32 + 8;
    }

    fn best_of(&self, key: &str) -> i64 {
        self.best.iter().find(|(k, _)| k == key).map(|(_, v)| *v).unwrap_or(0)
    }

    fn load_best() -> Vec<(String, i64)> {
        let data = store::load_section("pinball");
        let mut out = Vec::new();
        if let Some(Value::Object(best)) = data.get("best") {
            for (k, v) in best {
                let n = match v {
                    Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f.trunc() as i64)),
                    Value::Bool(b) => Some(*b as i64),
                    Value::String(s) => s.trim().parse::<i64>().ok(),
                    _ => None,
                };
                if let Some(n) = n {
                    out.push((k.clone(), n));
                }
            }
        }
        out
    }

    fn save_best(&mut self, score: i64) {
        if score > self.best_of(self.table_key) {
            match self.best.iter_mut().find(|(k, _)| k == self.table_key) {
                Some(e) => e.1 = score,
                None => self.best.push((self.table_key.to_string(), score)),
            }
            let mut m = Map::new();
            for (k, v) in &self.best {
                m.insert(k.clone(), json!(v));
            }
            let mut sec = Map::new();
            sec.insert("best".into(), Value::Object(m));
            store::save_section("pinball", sec);
        }
    }

    fn save_setting(&mut self, key: &str, value: Value) {
        self.b.set_opt("pinball", key, value);
    }

    // ----------------------------------------------- Partie / Ball

    fn new_game(&mut self) {
        self.table = table_data(self.table_key);
        self.players = if self.b.multiplayer { 2 } else { 1 };
        self.scores = vec![0; self.players];
        self.balls_left = vec![self.ball_count; self.players];
        self.player = 0;
        self.winner = None;
        self.b.score = 0;
        self.b.game_over = false;
        self.b.new_round_result();
        self.msg = None;
        self.msg_t = 0.0;
        self.rec_new();
        self.new_ball();
    }

    fn make_ball(&mut self, x: f64, y: f64, vx: f64, vy: f64) -> Ball {
        self.next_id += 1;
        Ball { id: self.next_id, x, y, vx, vy, held: 0.0, st: 0.0, sx: x, sy: y }
    }

    fn new_ball(&mut self) {
        self.rec_open = false;
        self.balls = vec![self.make_ball(LANE_X, 146.0, 0.0, 0.0)];
        self.phase_launch = true;
        self.plunger = 0.0;
        self.charging = false;
        self.flip_l = 0.0;
        self.flip_r = 0.0;
        self.flip_l_up = false;
        self.flip_r_up = false;
        self.omega_l = 0.0;
        self.omega_r = 0.0;
        self.mult = 1;
        self.locks = 0;
        self.multiball = false;
        self.jackpot = false;
        self.save_t = 0.0;
        self.tilt = false;
        self.nudges = 0;
        self.nudge_t = 0.0;
        self.shake = 0.0;
        self.targets_hit = vec![false; self.table.targets.len()];
        self.lanes_hit = vec![false; self.table.lanes.len()];
        self.bank_clears = 0;
        self.flash.clear();
    }

    // ----------------------------------------------- Setup-Screen

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let bw = 370.min(self.b.width - 50);
        let y0 = (self.b.height as f64 * 0.30) as i32;
        let gap = 8.0;
        let row = |y: i32, n: i32| -> Vec<Rect> {
            let cw = (bw as f64 - gap * (n - 1) as f64) / n as f64;
            (0..n).map(|i| Rect::new((cx as f64 - bw as f64 / 2.0 + i as f64 * (cw + gap)) as i32, y, cw as i32, 42)).collect()
        };
        self.table_rects = row(y0, 3);
        self.ball_rects = row(y0 + 88, 2);
        self.start_rect = Rect::new(cx - 95, y0 + 156, 190, 46);
    }

    fn set_table(&mut self, i: usize) {
        self.table_key = TABLES[i];
        self.save_setting("table", json!(self.table_key));
        self.b.play_sound("click");
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            match ev.key.as_str() {
                k @ ("1" | "2" | "3") => self.set_table(k.parse::<usize>().unwrap_or(1) - 1),
                "b" | "B" => self.cycle_balls(),
                "Return" | "space" => self.start_play(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.table_rects.iter().position(|r| r.hit(ev.pos)) {
                self.set_table(i);
                return;
            }
            if let Some(i) = self.ball_rects.iter().position(|r| r.hit(ev.pos)) {
                self.ball_count = BALL_COUNTS[i];
                self.save_setting("balls", json!(self.ball_count));
                self.b.play_sound("select");
                return;
            }
            if self.start_rect.hit(ev.pos) {
                self.start_play();
            }
        }
    }

    fn cycle_balls(&mut self) {
        let i = (BALL_COUNTS.iter().position(|c| *c == self.ball_count).unwrap_or(0) + 1) % BALL_COUNTS.len();
        self.ball_count = BALL_COUNTS[i];
        self.save_setting("balls", json!(self.ball_count));
        self.b.play_sound("select");
    }

    fn start_play(&mut self) {
        self.new_game();
        self.state = State::Play;
        self.b.play_sound("click");
    }

    // ----------------------------------------------- Eingabe

    fn launch(&mut self) {
        self.charging = false;
        if !self.phase_launch || self.balls.is_empty() {
            return;
        }
        let pl = 0.10f64.max(self.plunger);
        self.balls[0].vy = -(95.0 + 175.0 * pl);
        self.balls[0].vx = -py::uniform(0.0, 4.0);
        self.rec_scene();
        self.phase_launch = false;
        self.save_t = BALL_SAVE;
        self.plunger = 0.0;
        self.b.play_sound("shoot");
        self.b.rumble(70);
    }

    fn nudge(&mut self) {
        if self.tilt || self.phase_launch {
            return;
        }
        self.nudges += 1;
        self.nudge_t = 4.0;
        self.shake = 1.0;
        for b in self.balls.iter_mut() {
            b.vx += py::uniform(-26.0, 26.0);
            b.vy -= 22.0;
        }
        self.b.play_sound("hit");
        if self.nudges > TILT_LIMIT {
            self.tilt = true;
            self.say(t("pin.tilt"), 2.6);
            self.b.play_sound("gameover");
        }
    }

    // ----------------------------------------------- Update / Physik

    fn unstick(b: &mut Ball, dt: f64) {
        if py::hypot(b.x - b.sx, b.y - b.sy) > 1.0 {
            b.sx = b.x;
            b.sy = b.y;
            b.st = 0.0;
            return;
        }
        b.st += dt;
        if b.st > 3.0 {
            b.vx += py::uniform(-45.0, 45.0);
            b.vy -= 25.0;
            b.sx = b.x;
            b.sy = b.y;
            b.st = 0.0;
        }
    }

    fn back_in_lane(&self) -> bool {
        if self.phase_launch || self.balls.len() != 1 {
            return false;
        }
        let b = &self.balls[0];
        b.x > 86.0 && b.y > 138.0 && b.vy.abs() < 26.0 && b.vx.abs() < 26.0
    }

    fn has_ball(&self, id: u32) -> bool {
        self.balls.iter().any(|b| b.id == id)
    }

    fn write_back(&mut self, b: &Ball) {
        if let Some(x) = self.balls.iter_mut().find(|x| x.id == b.id) {
            *x = b.clone();
        }
    }

    fn step_ball(&mut self, b: &mut Ball, dt: f64) {
        let speed = py::hypot(b.vx, b.vy);
        let steps = 2.max(26.min((speed * dt / BR) as i64 + 2));
        let h = dt / steps as f64;
        for _ in 0..steps {
            b.vy += GRAV * h;
            let f = 0f64.max(1.0 - DRAG * h);
            b.vx *= f;
            b.vy *= f;
            let sp = py::hypot(b.vx, b.vy);
            if sp > MAX_SPEED {
                b.vx *= MAX_SPEED / sp;
                b.vy *= MAX_SPEED / sp;
            }
            b.x += b.vx * h;
            b.y += b.vy * h;
            self.collide_walls(b);
            self.collide_flippers(b);
            self.collide_bumpers(b);
            self.collide_slings(b);
            self.collide_targets(b);
            self.check_lanes(b);
            if self.check_saucer(b) {
                return;
            }
            if b.y > DRAIN_Y {
                self.drain(b);
                return;
            }
        }
    }

    fn collide_walls(&mut self, b: &mut Ball) {
        for i in 0..self.table.walls.len() {
            let seg = self.table.walls[i];
            if hit_seg(b, &seg, WALL_R, WALL_E, 0.0, (0.0, 0.0)) && b.vx.abs() + b.vy.abs() > 90.0 {
                self.b.play_sound("bounce");
            }
        }
        if b.x < 84.0 {
            hit_seg(b, &GATE, WALL_R, WALL_E, 0.0, (0.0, 0.0));
        }
    }

    fn flipper_seg(&self, left: bool) -> Seg {
        let (pivot, a) = if left {
            let (rest, up) = (py::radians(26.0), py::radians(-30.0));
            ((24.0, 140.0), rest + (up - rest) * self.flip_l)
        } else {
            let (rest, up) = (py::radians(154.0), py::radians(210.0));
            ((76.0, 140.0), rest + (up - rest) * self.flip_r)
        };
        [pivot.0, pivot.1, pivot.0 + a.cos() * FLIP_LEN, pivot.1 + a.sin() * FLIP_LEN]
    }

    fn collide_flippers(&mut self, b: &mut Ball) {
        for left in [true, false] {
            let seg = self.flipper_seg(left);
            let (px, py_) = closest_on_seg(seg[0], seg[1], seg[2], seg[3], b.x, b.y);
            let omega = if left { self.omega_l } else { self.omega_r };
            let sign = if left { -1.0 } else { 1.0 };
            let (rx, ry) = (px - seg[0], py_ - seg[1]);
            let w = omega * sign * 3.1;
            let vel = (-ry * w, rx * w);
            if hit_seg(b, &seg, FLIP_R, 0.45, 0.0, vel) {
                self.b.play_sound("bounce");
            }
        }
    }

    fn set_flash(&mut self, key: String, v: f64) {
        match self.flash.iter_mut().find(|(k, _)| *k == key) {
            Some(e) => e.1 = v,
            None => self.flash.push((key, v)),
        }
    }

    fn is_hot(&self, key: &str) -> bool {
        self.flash.iter().any(|(k, _)| k == key)
    }

    fn collide_bumpers(&mut self, b: &mut Ball) {
        for i in 0..self.table.bumpers.len() {
            let [x, y, r] = self.table.bumpers[i];
            let (dx, dy) = (b.x - x, b.y - y);
            let d = py::hypot(dx, dy);
            let rad = r + BR;
            if d >= rad || d < 1e-9 {
                continue;
            }
            let (ux, uy) = (dx / d, dy / d);
            b.x = x + ux * rad;
            b.y = y + uy * rad;
            let dot = b.vx * ux + b.vy * uy;
            if dot < 0.0 {
                b.vx -= 1.5 * dot * ux;
                b.vy -= 1.5 * dot * uy;
            }
            b.vx += ux * BUMP_KICK;
            b.vy += uy * BUMP_KICK;
            self.set_flash(format!("b{i}"), 0.22);
            let jp = self.jackpot;
            self.award(if jp { 2500 } else { 100 }, if jp { Some(t("pin.jackpot")) } else { None });
            self.b.play_sound(if jp { "point" } else { "bounce" });
        }
    }

    fn collide_slings(&mut self, b: &mut Ball) {
        for i in 0..self.table.slings.len() {
            let seg = self.table.slings[i];
            if hit_seg(b, &seg, WALL_R * 1.6, 0.35, SLING_KICK, (0.0, 0.0)) {
                self.set_flash(format!("s{i}"), 0.18);
                self.award(50, None);
                self.b.play_sound("bounce");
            }
        }
    }

    fn collide_targets(&mut self, b: &mut Ball) {
        for i in 0..self.table.targets.len() {
            if self.targets_hit[i] {
                continue;
            }
            let [x, y, w, h] = self.table.targets[i];
            let nx = x.max(b.x.min(x + w));
            let ny = y.max(b.y.min(y + h));
            let (dx, dy) = (b.x - nx, b.y - ny);
            if dx * dx + dy * dy >= BR * BR {
                continue;
            }
            let mut d = py::hypot(dx, dy);
            if d == 0.0 {
                d = 1.0;
            }
            let (ux, uy) = (dx / d, dy / d);
            b.x = nx + ux * BR;
            b.y = ny + uy * BR;
            let dot = b.vx * ux + b.vy * uy;
            if dot < 0.0 {
                b.vx -= 1.4 * dot * ux;
                b.vy -= 1.4 * dot * uy;
            }
            self.targets_hit[i] = true;
            self.award(250, None);
            self.b.play_sound("lock");
            if self.targets_hit.iter().all(|h| *h) {
                self.targets_hit = vec![false; self.targets_hit.len()];
                self.bank_clears += 1;
                self.mult = 5.min(self.mult + 1);
                let m = tf("pin.bank", &[("n", &self.mult.to_string())]);
                self.award(2500, Some(m));
                self.b.play_sound("powerup");
            }
        }
    }

    fn check_lanes(&mut self, b: &Ball) {
        for i in 0..self.table.lanes.len() {
            if self.lanes_hit[i] {
                continue;
            }
            let [x, y, r] = self.table.lanes[i];
            if py::pow(b.x - x, 2.0) + py::pow(b.y - y, 2.0) < py::pow(r + BR, 2.0) {
                self.lanes_hit[i] = true;
                self.award(200, None);
                self.b.play_sound("eat");
                if self.lanes_hit.iter().all(|h| *h) {
                    self.lanes_hit = vec![false; self.lanes_hit.len()];
                    self.mult = 5.min(self.mult + 1);
                    let m = tf("pin.lanes", &[("n", &self.mult.to_string())]);
                    self.award(1000, Some(m));
                    self.b.play_sound("level");
                }
            }
        }
    }

    fn check_saucer(&mut self, b: &mut Ball) -> bool {
        let [x, y, r] = self.table.saucer;
        if py::pow(b.x - x, 2.0) + py::pow(b.y - y, 2.0) > r * r {
            return false;
        }
        if py::hypot(b.vx, b.vy) > 110.0 {
            return false;
        }
        b.x = x;
        b.y = y;
        b.vx = 0.0;
        b.vy = 0.0;
        b.held = 1.1;
        if self.multiball {
            self.award(5000, Some(t("pin.jackpot")));
            self.b.play_sound("win");
            return true;
        }
        self.locks += 1;
        if self.locks >= 3 {
            self.start_multiball(b);
        } else {
            let m = tf("pin.lock", &[("n", &self.locks.to_string())]);
            self.award(1000, Some(m));
            self.b.play_sound("powerup");
        }
        true
    }

    fn start_multiball(&mut self, b: &mut Ball) {
        self.multiball = true;
        self.jackpot = true;
        self.locks = 0;
        b.held = 1.4;
        self.award(5000, Some(t("pin.multiball")));
        self.b.ach_event("pin_multiball", None);
        self.b.play_sound("win");
    }

    fn eject(&mut self, b: &mut Ball) {
        b.vx = py::uniform(-40.0, 40.0);
        b.vy = -150.0;
        b.held = 0.0;
        if self.multiball && self.balls.len() < 3 {
            for _ in 0..3 - self.balls.len() {
                let x = b.x + py::uniform(-4.0, 4.0);
                let vx = py::uniform(-60.0, 60.0);
                let nb = self.make_ball(x, b.y, vx, -130.0);
                self.balls.push(nb);
            }
            self.save_t = self.save_t.max(4.0);
        }
        self.b.play_sound("shoot");
    }

    // ----------------------------------------------- Ballverlust

    fn drain(&mut self, b: &Ball) {
        if let Some(i) = self.balls.iter().position(|x| x.id == b.id) {
            self.balls.remove(i);
        }
        if !self.balls.is_empty() {
            if self.balls.len() == 1 {
                self.multiball = false;
                self.jackpot = false;
            }
            self.b.play_sound("hit");
            return;
        }
        if self.save_t > 0.0 && !self.tilt {
            self.balls = vec![self.make_ball(LANE_X, 146.0, 0.0, 0.0)];
            self.phase_launch = true;
            self.plunger = 0.0;
            self.say(t("pin.saved"), 1.8);
            self.b.play_sound("powerup");
            return;
        }
        self.b.play_sound("gameover");
    }

    fn ball_lost(&mut self) {
        let bonus = 250 * self.mult;
        self.scores[self.player] += bonus;
        self.sync_score();
        self.balls_left[self.player] -= 1;
        let last = self.balls_left.iter().copied().max().unwrap_or(0) <= 0;
        self.rec_close(bonus, last);
        if last {
            self.end_game();
            return;
        }
        for i in 1..=self.players {
            let nxt = (self.player + i) % self.players;
            if self.balls_left[nxt] > 0 {
                self.player = nxt;
                break;
            }
        }
        self.new_ball();
        self.say(tf("pin.bonus", &[("n", &bonus.to_string())]), 2.2);
    }

    fn end_game(&mut self) {
        self.save_best(self.scores[0]);
        if self.scores[0] >= 50000 {
            self.b.ach_event("pin_high", Some(self.scores[0]));
        }
        if self.b.multiplayer {
            let (s0, s1) = (self.scores[0], self.scores[1]);
            self.winner = if s0 > s1 {
                Some(0)
            } else if s1 > s0 {
                Some(1)
            } else {
                None
            };
            if let Some(w) = self.winner {
                self.b.report_duel(w);
            }
        } else {
            let won = self.scores[0] >= self.best_of(self.table_key);
            self.b.report_result(won);
        }
        self.sync_score();
        self.state = State::Over;
        self.b.game_over = true;
        self.rec_finish();
        self.b.play_sound("gameover");
    }

    fn restart(&mut self) {
        self.new_game();
        self.state = State::Play;
        self.b.play_sound("click");
    }

    // ----------------------------------------------- Replay-Aufnahme

    fn rec_new(&mut self) {
        self.replay = None;
        self.rec_open = false;
        self.rec_delta.reset();
        let mut meta = Map::new();
        meta.insert("table".into(), json!(self.table_key));
        meta.insert("balls".into(), json!(self.ball_count));
        meta.insert("players".into(), json!(if self.b.multiplayer { 2 } else { 1 }));
        self.rec = replay::recorder("pinball", shell::replay_enabled(&self.b.settings), meta);
    }

    fn rec_scene(&mut self) {
        if self.rec.is_none() || self.rec_open {
            return;
        }
        self.rec_open = true;
        self.rec_delta.reset();
        self.rec_start = self.scores[self.player];
        let mut info = Map::new();
        info.insert("pl".into(), json!(self.player));
        info.insert("ball".into(), json!(self.ball_count - self.balls_left[self.player] + 1));
        info.insert("total".into(), json!(self.ball_count));
        if let Some(r) = self.rec.as_mut() {
            r.scene(false, info);
        }
    }

    fn rec_state(&self) -> Vec<i64> {
        let flags = (self.tilt as i64) | ((self.multiball as i64) << 1) | ((self.phase_launch as i64) << 2) | ((self.jackpot as i64) << 3);
        let tmask: i64 = self.targets_hit.iter().enumerate().filter(|(_, on)| **on).map(|(i, _)| 1i64 << i).sum();
        let lmask: i64 = self.lanes_hit.iter().enumerate().filter(|(_, on)| **on).map(|(i, _)| 1i64 << i).sum();
        vec![
            self.scores[0],
            *self.scores.last().unwrap_or(&0),
            self.mult,
            tmask,
            lmask,
            flags,
            self.locks,
            self.bank_clears,
            (self.save_t * 10.0) as i64,
            (self.plunger * 100.0) as i64,
        ]
    }

    fn rec_sample(&mut self) -> Option<Vec<Value>> {
        let mut out = vec![json!(self.balls.len())];
        for b in &self.balls {
            out.push(rnd(b.x, 1));
            out.push(rnd(b.y, 1));
        }
        out.push(rnd(self.flip_l, 2));
        out.push(rnd(self.flip_r, 2));
        let st: Vec<Value> = self.rec_state().into_iter().map(|v| json!(v)).collect();
        if self.rec_delta.push(0, &st) {
            out.push(json!(REC_STATE));
            out.extend(st);
        } else {
            out.push(json!(0));
        }
        Some(out)
    }

    fn rec_close(&mut self, bonus: i64, last: bool) {
        if self.rec.is_none() || !self.rec_open {
            return;
        }
        self.rec_open = false;
        let sample = self.rec_sample();
        let mut info = Map::new();
        info.insert("bonus".into(), json!(bonus));
        info.insert("final".into(), json!(last));
        info.insert("pts".into(), json!(self.scores[self.player] - self.rec_start));
        if let Some(r) = self.rec.as_mut() {
            r.close(sample, info);
        }
    }

    fn rec_tick(&mut self, dt: f64) {
        if self.rec_open {
            if let Some(mut r) = self.rec.take() {
                r.tick(dt, || self.rec_sample());
                self.rec = Some(r);
            }
        }
    }

    fn rec_finish(&mut self) {
        let Some(mut r) = self.rec.take() else {
            return;
        };
        let sub = if self.b.multiplayer {
            match self.winner {
                None => t("common.draw"),
                Some(w) => tf("common.player_wins_name", &[("name", &self.b.pname(w))]),
            }
        } else {
            tf("common.points", &[("score", &self.scores[0].to_string())])
        };
        let mut meta = Map::new();
        meta.insert("score".into(), json!(self.scores[0]));
        meta.insert("scores".into(), json!(self.scores));
        meta.insert("winner".into(), json!(self.winner.map(|w| w as i64).unwrap_or(-1)));
        self.replay = r.result(&t(&format!("pin.table.{}", self.table_key)), &sub, meta);
    }

    fn open_replay(&mut self) {
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
        if let Some(k) = meta.get("table").and_then(|v| v.as_str()).and_then(|k| TABLES.iter().find(|t| **t == k)) {
            self.table_key = k;
        }
        let balls = meta.get("balls").and_then(|v| v.as_i64()).filter(|v| *v != 0).unwrap_or(3);
        if BALL_COUNTS.contains(&balls) {
            self.ball_count = balls;
        }
        let pl = meta.get("players").and_then(|v| v.as_i64()).filter(|v| *v != 0).unwrap_or(1);
        self.players = pl.clamp(1, 2) as usize;
        self.b.multiplayer = self.players > 1;
        self.table = table_data(self.table_key);
        self.scores = vec![0; self.players];
        self.balls_left = vec![self.ball_count; self.players];
        let win = meta.get("winner").and_then(|v| v.as_i64()).unwrap_or(-1);
        self.winner = if win >= 0 { Some(win as usize) } else { None };
        self.state = State::Play;
        self.b.game_over = false;
        self.msg = None;
        self.msg_t = 0.0;
        self.shake = 0.0;
        self.flash.clear();
        self.layout();
        self.replay_seek(0, 0);
    }

    /// Setzt Baelle, Flipper und Tisch auf Szene `index`, Sample `frame`.
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
        let pl = sc.get("pl").and_then(|v| v.as_i64()).unwrap_or(0);
        self.player = (self.players as i64 - 1).min(0.max(pl)) as usize;
        let ball = sc.get("ball").and_then(|v| v.as_i64()).unwrap_or(1);
        for i in 0..self.players {
            self.balls_left[i] = 0.max(self.ball_count - ball + 1);
        }
        let start = match self.rep_at {
            Some((i, f)) if i == index && frame >= f => f + 1,
            _ => 0,
        };
        if start == 0 {
            self.rep_state = None;
        }
        for fr in frames.iter().take(frame + 1).skip(start) {
            self.rep_apply(fr);
        }
        self.msg = None;
        if last {
            if sc.get("final").and_then(|v| v.as_bool()).unwrap_or(false) {
                self.msg = Some(t("common.game_over"));
            } else if let Some(bonus) = sc.get("bonus").and_then(|v| v.as_i64()).filter(|b| *b != 0) {
                self.msg = Some(tf("pin.bonus", &[("n", &bonus.to_string())]));
            }
        }
        self.rep_at = Some((index, frame));
    }

    fn rep_apply(&mut self, fr: &Value) {
        let Some(fr) = fr.as_array() else {
            return;
        };
        let num = |i: usize| fr.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0);
        let nb = num(0) as usize;
        let balls: Vec<(f64, f64)> = (0..nb).map(|i| (num(1 + 2 * i), num(2 + 2 * i))).collect();
        self.balls = balls.into_iter().map(|(x, y)| self.make_ball(x, y, 0.0, 0.0)).collect();
        let p = 1 + 2 * nb;
        self.flip_l = num(p);
        self.flip_r = num(p + 1);
        if num(p + 2) as i64 != 0 {
            self.rep_state = Some((0..REC_STATE as usize).map(|k| num(p + 3 + k)).collect());
        }
        let Some(st) = self.rep_state.clone() else {
            return;
        };
        if st.is_empty() {
            return;
        }
        self.scores[0] = st[0] as i64;
        let li = self.scores.len() - 1;
        self.scores[li] = st[1] as i64;
        self.b.score = self.scores[0];
        self.mult = st[2] as i64;
        let (tmask, lmask, flags) = (st[3] as i64, st[4] as i64, st[5] as i64);
        self.targets_hit = (0..self.table.targets.len()).map(|i| (tmask >> i) & 1 != 0).collect();
        self.lanes_hit = (0..self.table.lanes.len()).map(|i| (lmask >> i) & 1 != 0).collect();
        self.tilt = flags & 1 != 0;
        self.multiball = flags & 2 != 0;
        self.phase_launch = flags & 4 != 0;
        self.jackpot = flags & 8 != 0;
        self.locks = st[6] as i64;
        self.bank_clears = st[7] as i64;
        self.save_t = st[8] / 10.0;
        self.plunger = st[9] / 100.0;
    }

    /// Zeichnet den aktuellen Replay-Stand (ohne Menue-Overlay).
    pub fn replay_draw(&mut self, _aiming: bool, banner: bool) {
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        self.draw_table(&mut s);
        self.draw_hud(&mut s);
        self.draw_side(&mut s);
        if banner {
            self.draw_over(&mut s);
        }
    }

    // ----------------------------------------------- Hilfsfunktionen

    fn award(&mut self, points: i64, message: Option<String>) {
        self.scores[self.player] += points * self.mult;
        self.sync_score();
        if let Some(m) = message {
            if !m.is_empty() {
                self.say(m, 1.8);
            }
        }
    }

    fn sync_score(&mut self) {
        self.b.score = self.scores[0];
    }

    fn say(&mut self, text: String, secs: f64) {
        self.msg = Some(text);
        self.msg_t = secs;
    }

    // ----------------------------------------------- Zeichnen

    fn project(&self, x: f64, y: f64) -> (f64, f64) {
        let mut sh = 0.0;
        if self.shake > 0.0 {
            sh = (pg::ticks() as f64 / 22.0).sin() * 4.0 * self.shake;
        }
        (self.ox + x * self.scale + sh, self.oy + y * self.scale)
    }

    fn draw_table(&self, s: &mut Surface) {
        let board = Rect::new(self.ox as i32, self.oy as i32, (TW * self.scale) as i32, (TH * self.scale) as i32);
        draw::rect_r(s, COL_TABLE, board, 0, 8);
        draw::rect_r(s, COL_TABLE_D, board, 2, 8);
        let mut glow = pg::surface_alpha(board.w, board.h / 2);
        let gr = glow.get_rect();
        draw::ellipse(&mut glow, rgba(70, 90, 150, 46), gr, 0);
        s.bl(&glow, (board.x, board.y));

        let ww = 2.max((WALL_R * 2.0 * self.scale) as i32);
        for seg in &self.table.walls {
            let (a, b) = (self.project(seg[0], seg[1]), self.project(seg[2], seg[3]));
            draw::line(s, COL_WALL, a, b, ww);
            draw::line(s, COL_WALL_HI, a, b, 1);
        }
        let (ga, gb) = (self.project(GATE[0], GATE[1]), self.project(GATE[2], GATE[3]));
        draw::line(s, COL_WALL_HI, ga, gb, 1);

        let [sx, sy, sr] = self.table.saucer;
        let (px, py_) = self.project(sx, sy);
        let rr = 3.max((sr * self.scale) as i32);
        draw::circle(s, rgb(12, 14, 22), (px as i32, py_ as i32), rr, 0);
        draw::circle(s, COL_SAUCER, (px as i32, py_ as i32), rr, 2);

        for (i, l) in self.table.lanes.iter().enumerate() {
            let (px, py_) = self.project(l[0], l[1]);
            let rr = 3.max((l[2] * self.scale) as i32);
            let on = self.lanes_hit.get(i).copied().unwrap_or(false);
            draw::circle(s, if on { COL_LANE } else { rgb(58, 44, 72) }, (px as i32, py_ as i32), rr, 0);
            draw::circle(s, COL_LANE, (px as i32, py_ as i32), rr, 1);
            let ch = LANE_LETTERS[i % LANE_LETTERS.len()];
            let img = self.tiny.render(ch, true, if on { rgb(20, 16, 28) } else { COL_LANE });
            s.blr(&img, img.get_rect().center_at((px, py_)));
        }

        for (i, r) in self.table.targets.iter().enumerate() {
            let (px, py_) = self.project(r[0], r[1]);
            let rc = Rect::new(px as i32, py_ as i32, 2.max((r[2] * self.scale) as i32), 2.max((r[3] * self.scale) as i32));
            let col = if self.targets_hit.get(i).copied().unwrap_or(false) { COL_TARGET_OFF } else { COL_TARGET };
            draw::rect_r(s, col, rc, 0, 2);
            draw::rect_r(s, ui::mix(col, (255, 255, 255), 0.3), rc, 1, 2);
        }

        let sw = 3.max((WALL_R * 3.4 * self.scale) as i32);
        for (i, seg) in self.table.slings.iter().enumerate() {
            let (a, b) = (self.project(seg[0], seg[1]), self.project(seg[2], seg[3]));
            let hot = self.is_hot(&format!("s{i}"));
            draw::line(s, if hot { rgb(255, 236, 200) } else { COL_SLING }, a, b, sw);
        }

        for (i, bm) in self.table.bumpers.iter().enumerate() {
            let (px, py_) = self.project(bm[0], bm[1]);
            let rr = 4.max((bm[2] * self.scale) as i32);
            let hot = self.is_hot(&format!("b{i}"));
            let c = (px as i32, py_ as i32);
            draw::circle(s, COL_BUMP, c, rr, 0);
            draw::circle(s, if hot { rgb(255, 255, 255) } else { COL_BUMP_HI }, c, 2.max((rr as f64 * 0.55) as i32), 0);
            draw::circle(s, COL_BUMP_HI, c, rr, 2);
        }

        self.draw_flippers(s);
        self.draw_plunger(s);
        for b in &self.balls {
            self.draw_ball(s, b);
        }
    }

    fn draw_flippers(&self, s: &mut Surface) {
        for left in [true, false] {
            let seg = self.flipper_seg(left);
            let (a, b) = (self.project(seg[0], seg[1]), self.project(seg[2], seg[3]));
            let w = 4.max((FLIP_R * 2.0 * self.scale) as i32);
            let col = if self.tilt { COL_FLIP_D } else { COL_FLIP };
            draw::line(s, col, a, b, w);
            draw::circle(s, COL_FLIP_D, (a.0 as i32, a.1 as i32), 2.max(w / 2), 0);
            draw::circle(s, ui::mix(col, (255, 255, 255), 0.4), (b.0 as i32, b.1 as i32), 2.max(w / 3), 0);
        }
    }

    fn draw_plunger(&self, s: &mut Surface) {
        if !self.phase_launch {
            return;
        }
        let (x0, y0) = self.project(LANE_X - 3.4, 150.0);
        let (x1, _) = self.project(LANE_X + 3.4, 150.0);
        let h = 6.max((16.0 * self.scale * (0.25 + self.plunger)) as i32);
        draw::rect_r(s, COL_PLUNGER, Rect::new(x0 as i32, y0 as i32, (x1 - x0) as i32, h), 0, 3);
    }

    fn draw_ball(&self, s: &mut Surface, b: &Ball) {
        let (px, py_) = self.project(b.x, b.y);
        let r = 2.max((BR * self.scale) as i32);
        draw::circle(s, rgb(8, 10, 16), ((px + 2.0) as i32, (py_ + 2.0) as i32), r, 0);
        draw::circle(s, COL_BALL, (px as i32, py_ as i32), r, 0);
        draw::circle(s, COL_BALL_D, (px as i32, py_ as i32), r, 1);
        if r >= 4 {
            let o = (r / 3) as f64;
            draw::circle(s, rgb(255, 255, 255), ((px - o) as i32, (py_ - o) as i32), 1.max(r / 3), 0);
        }
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        draw::rect(s, p.panel, Rect::new(0, 0, w, self.hud_h), 0);
        draw::line(s, p.border, (0, self.hud_h), (w, self.hud_h), 1);
        let cy = self.hud_h / 2;
        let img = self.num.render(&self.scores[self.player].to_string(), true, self.b.accent);
        s.blr(&img, img.get_rect().midleft_at((12, cy)));
        let (mid, col) = if let Some(m) = &self.msg {
            (m.clone(), p.gold)
        } else if self.tilt {
            (t("pin.tilt"), p.red)
        } else if self.phase_launch {
            (t("pin.launch"), p.text_dim)
        } else if self.b.multiplayer {
            (self.b.pname(self.player).chars().take(10).collect(), p.text_dim)
        } else {
            (tf("pin.mult", &[("n", &self.mult.to_string())]), p.text_dim)
        };
        let m = self.small.render(&mid, true, col);
        s.blr(&m, m.get_rect().center_at((w / 2, cy)));
        let balls = self.tiny.render(&tf("pin.ball", &[("n", &self.balls_left[self.player].to_string())]), true, p.text_dim);
        s.blr(&balls, balls.get_rect().midright_at((w - 12, cy)));
    }

    fn draw_side(&self, s: &mut Surface) {
        let p = ui::pal();
        let x = self.side_x;
        let w = self.b.width - x - 6;
        if w < 70 {
            return;
        }
        let y = self.hud_h + 10;
        let rect = Rect::new(x, y, w, 116);
        ui::draw_panel_ex(s, rect, None, None, Some(8), false, None);
        let mut cy = y + 10;
        let mut rows = vec![
            (t("pin.lbl_mult"), format!("x{}", self.mult), self.b.accent),
            (t("pin.lbl_lock"), format!("{}/3", self.locks), COL_SAUCER),
            (t("pin.lbl_bank"), self.bank_clears.to_string(), COL_TARGET),
        ];
        if self.save_t > 0.0 {
            rows.push((t("pin.lbl_save"), format!("{:.0}s", self.save_t), p.green));
        } else if self.multiball {
            rows.push((t("pin.lbl_mode"), t("pin.multiball_short"), p.gold));
        }
        let th = self.tiny.get_height();
        for (label, value, col) in rows {
            s.bl(&self.tiny.render(&label, true, p.text_dim), (x + 8, cy));
            let v = self.tiny.render(&value, true, col);
            s.blr(&v, v.get_rect().topright_at((x + w - 8, cy)));
            cy += th + 8;
        }
        if self.b.multiplayer {
            cy += 4;
            draw::line(s, p.border, (x + 6, cy), (x + w - 6, cy), 1);
            cy += 6;
            for i in 0..self.players {
                let col = if i == self.player { self.b.accent } else { p.text_dim };
                let name: String = self.b.pname(i).chars().take(6).collect();
                s.bl(&self.tiny.render(&name, true, col), (x + 8, cy));
                let v = self.tiny.render(&self.scores[i].to_string(), true, col);
                s.blr(&v, v.get_rect().topright_at((x + w - 8, cy)));
                cy += th + 4;
            }
        }
    }

    fn draw_over(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let h = 118;
        let mut ov = pg::surface_alpha(w, h);
        ov.fill_c(rgba(10, 12, 20, 214));
        let y = self.b.height / 2 - h / 2;
        s.bl(&ov, (0, y));
        draw::line(s, self.b.accent, (0, y), (w, y), 1);
        draw::line(s, self.b.accent, (0, y + h - 1), (w, y + h - 1), 1);
        let cx = w / 2;
        let head_txt = if self.b.multiplayer {
            match self.winner {
                None => t("common.draw"),
                Some(wn) => tf("common.player_wins_name", &[("name", &self.b.pname(wn).chars().take(10).collect::<String>())]),
            }
        } else {
            t("common.game_over")
        };
        let head = self.huge.render(&head_txt, true, self.b.accent);
        s.blr(&head, head.get_rect().center_at((cx, y + 32)));
        let sub = self.small.render(&tf("common.points", &[("score", &self.scores[0].to_string())]), true, p.text);
        s.blr(&sub, sub.get_rect().center_at((cx, y + 66)));
        let best = self.best_of(self.table_key);
        if best != 0 {
            let b = self.tiny.render(&tf("pin.best", &[("n", &best.to_string())]), true, p.gold);
            s.blr(&b, b.get_rect().center_at((cx, y + 88)));
        }
        let mut hint_txt = t("pin.new_round");
        if self.replay.is_some() && self.rep.is_none() {
            hint_txt += &format!("  ·  {}", t("pin.replay_hint"));
        }
        let hint = self.tiny.render(&hint_txt, true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, y + 106)));
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let cx = self.b.width / 2;
        let h = self.b.height;
        let title = self.huge.render(&t("pin.title"), true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.13) as i32)));
        let sub = self.small.render(&t("pin.subtitle"), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, (h as f64 * 0.20) as i32)));
        let label = |s: &mut Surface, rects: &[Rect], txt: &str| {
            let im = self.tiny.render(txt, true, p.text_dim);
            s.blr(&im, im.get_rect().midbottom_at((cx, rects[0].y - 4)));
        };
        label(s, &self.table_rects, &t("pin.lbl_table"));
        for (i, rc) in self.table_rects.iter().enumerate() {
            self.btn(s, *rc, &t(&format!("pin.table.{}", TABLES[i])), self.table_key == TABLES[i]);
        }
        label(s, &self.ball_rects, &t("pin.lbl_balls"));
        for (i, rc) in self.ball_rects.iter().enumerate() {
            self.btn(s, *rc, &BALL_COUNTS[i].to_string(), self.ball_count == BALL_COUNTS[i]);
        }
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 9);
        draw::rect_r(s, self.b.accent, self.start_rect, 2, 9);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        let best = self.best_of(self.table_key);
        if best != 0 {
            let b = self.tiny.render(&tf("pin.best", &[("n", &best.to_string())]), true, p.gold);
            s.blr(&b, b.get_rect().center_at((cx, self.start_rect.bottom() + 22)));
        }
        let hint = self.tiny.render(&t("pin.setup_hint"), true, p.text_dim);
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

impl Game for PinballGame {
    fn replay_begin(&mut self, rep: &Value) -> bool {
        PinballGame::replay_begin(self, rep);
        true
    }

    fn replay_seek(&mut self, index: usize, frame: usize) {
        PinballGame::replay_seek(self, index, frame);
    }

    fn replay_draw(&mut self, aiming: bool, banner: bool) {
        PinballGame::replay_draw(self, aiming, banner);
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
        let tk = self.b.opt_str("pinball", "table", "classic");
        self.table_key = TABLES.iter().find(|t| **t == tk).copied().unwrap_or("classic");
        self.ball_count = self.b.opt_int_py("pinball", "balls", 3).unwrap_or(3);
        if !BALL_COUNTS.contains(&self.ball_count) {
            self.ball_count = 3;
        }
        self.rec = None;
        self.replay = None;
        self.replay_request = None;
        self.rep = None;
        self.rep_at = None;
        self.rep_state = None;
        self.rec_open = false;
        self.rec_delta = Delta::new();
        self.rec_start = 0;
        self.build_fonts();
        self.layout();
        self.build_setup_layout();
        self.best = Self::load_best();
        self.new_game();
        self.state = State::Setup;
    }

    fn on_surface_changed(&mut self) {
        self.build_fonts();
        self.layout();
        self.build_setup_layout();
    }

    fn take_replay_request(&mut self) -> Option<Value> {
        self.replay_request.take()
    }

    fn handle_event(&mut self, ev: &InputEvent) {
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
        let k = ev.key.as_str();
        if ev.kind == Kind::KeyDown {
            if k == "Shift_L" || self.b.is_action(k, "left", None) || k == "Left" {
                self.flip_l_up = true;
            }
            if k == "Shift_R" || self.b.is_action(k, "right", None) || k == "Right" {
                self.flip_r_up = true;
            }
            if self.b.is_action(k, "up", None) || k == "Up" {
                self.nudge();
            }
            if (self.b.is_action(k, "action", None) || k == "space" || k == "Return") && self.phase_launch {
                self.charging = true;
            }
        } else if ev.kind == Kind::KeyUp {
            if k == "Shift_L" || self.b.is_action(k, "left", None) || k == "Left" {
                self.flip_l_up = false;
            }
            if k == "Shift_R" || self.b.is_action(k, "right", None) || k == "Right" {
                self.flip_r_up = false;
            }
            if (self.b.is_action(k, "action", None) || k == "space" || k == "Return") && self.charging {
                self.launch();
            }
        } else if ev.kind == Kind::MouseDown && ev.button == 1 {
            if self.phase_launch {
                self.charging = true;
            } else if ev.pos.0 < self.b.width / 2 {
                self.flip_l_up = true;
            } else {
                self.flip_r_up = true;
            }
        } else if ev.kind == Kind::MouseUp && ev.button == 1 {
            if self.charging {
                self.launch();
            }
            self.flip_l_up = false;
            self.flip_r_up = false;
        }
    }

    fn update(&mut self, dt: f64) {
        if self.msg_t > 0.0 {
            self.msg_t -= dt;
            if self.msg_t <= 0.0 {
                self.msg = None;
            }
        }
        for e in self.flash.iter_mut() {
            e.1 -= dt;
        }
        self.flash.retain(|e| e.1 > 0.0);
        if self.shake > 0.0 {
            self.shake = 0f64.max(self.shake - dt * 3.2);
        }
        if self.nudge_t > 0.0 {
            self.nudge_t -= dt;
            if self.nudge_t <= 0.0 {
                self.nudges = 0;
            }
        }
        if self.state != State::Play {
            return;
        }
        for left in [true, false] {
            let cur = if left { self.flip_l } else { self.flip_r };
            let up = (if left { self.flip_l_up } else { self.flip_r_up }) && !self.tilt;
            let target = if up { 1.0 } else { 0.0 };
            let step = FLIP_SPEED * dt / 2.6;
            let new = if target > cur { target.min(cur + step) } else { target.max(cur - step) };
            let omega = (new - cur) / dt.max(1e-4) * 0.95;
            if left {
                self.omega_l = omega;
                self.flip_l = new;
            } else {
                self.omega_r = omega;
                self.flip_r = new;
            }
        }
        if self.phase_launch {
            if self.charging {
                self.plunger = 1f64.min(self.plunger + dt * 0.85);
            }
            self.rec_tick(dt);
            return;
        }
        if self.save_t > 0.0 {
            self.save_t = 0f64.max(self.save_t - dt);
        }
        let ids: Vec<u32> = self.balls.iter().map(|b| b.id).collect();
        for id in ids {
            let Some(mut b) = self.balls.iter().find(|b| b.id == id).cloned() else {
                continue;
            };
            if b.held > 0.0 {
                b.held -= dt;
                if b.held <= 0.0 {
                    self.write_back(&b);
                    self.eject(&mut b);
                }
                self.write_back(&b);
                continue;
            }
            self.step_ball(&mut b, dt);
            Self::unstick(&mut b, dt);
            if self.has_ball(id) {
                self.write_back(&b);
            }
        }
        self.rec_tick(dt);
        if self.balls.is_empty() {
            self.ball_lost();
        } else if self.back_in_lane() {
            let b = &mut self.balls[0];
            b.x = LANE_X;
            b.y = 146.0;
            b.vx = 0.0;
            b.vy = 0.0;
            self.phase_launch = true;
            self.plunger = 0.0;
            self.charging = false;
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        if self.state == State::Setup {
            self.draw_setup(&mut s);
            return;
        }
        self.draw_table(&mut s);
        self.draw_hud(&mut s);
        self.draw_side(&mut s);
        if self.state == State::Over {
            self.draw_over(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "pinball",
    class_name: "PinballGame",
    supports_multiplayer: true,
    modes: &[],
    make: PinballGame::new,
};
