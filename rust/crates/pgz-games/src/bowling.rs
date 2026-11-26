//! bowling.rs
//! ==========
//! Bowling - zehn Frames mit voller Strike-/Spare-Wertung, Pin-Physik und
//! perspektivischer Bahn, allein oder zu zweit.
//!
//! Uebertragen aus `games/bowling.py`: vier Regler je Wurf (Position, Ziel,
//! Effet, Kraft), Hook hinter dem Oel, Pins als Kreise mit Masse,
//! Scorecard, Bestwert je Stufe, Replay-Aufnahme (`pgz_core::replay`) und
//! die Wiedergabe-Methoden fuer den Replay-Screen.

use pgz_core::replay::{self, rnd, Recorder};
use pgz_core::store;
use pgz_engine::prelude::*;
use serde_json::{json, Map, Value};

const COL_LANE: Color = Color::rgb(206, 164, 104);
const COL_BOARD: Color = Color::rgb(196, 152, 96);
const COL_GUTTER: Color = Color::rgb(46, 50, 62);
const COL_GUTTER_D: Color = Color::rgb(32, 35, 45);
const COL_FOUL: Color = Color::rgb(208, 72, 72);
const COL_ARROW: Color = Color::rgb(150, 108, 62);
const COL_DECK: Color = Color::rgb(232, 228, 220);
const COL_BACK: Color = Color::rgb(28, 32, 46);
const COL_PIN: Color = Color::rgb(246, 246, 242);
const COL_PIN_D: Color = Color::rgb(198, 198, 192);
const COL_PIN_RED: Color = Color::rgb(216, 60, 60);
const COL_BALL: Color = Color::rgb(58, 74, 176);
const COL_BALL_HI: Color = Color::rgb(120, 140, 236);
const COL_MARK: Color = Color::rgb(250, 224, 120);

const LW: f64 = 41.5;
const GUT: f64 = 9.0;
const PIN_Y: f64 = 720.0;
const ROW_DY: f64 = 10.392;
const PIN_DX: f64 = 6.0;
const BALL_R: f64 = 4.25;
const PIN_R: f64 = 2.4;
const PIN_M: f64 = 3.5;
const BALL_M: f64 = 16.0;
const OIL_END: f64 = 430.0;
const HOOK: f64 = 300.0;
const PIN_FRIC: f64 = 3.4;
const BALL_FRIC: f64 = 0.10;
const REST: f64 = 0.55;
const DOWN_DIST: f64 = 1.6;
const MAX_ROLL_TIME: f64 = 7.0;
const PIT_Y: f64 = 810.0;

const DIFFS: [&str; 3] = ["easy", "normal", "pro"];
const STEPS: [&str; 4] = ["pos", "aim", "spin", "power"];

fn swing(d: &str) -> f64 {
    match d {
        "easy" => 0.72,
        "pro" => 1.45,
        _ => 1.05,
    }
}

fn scatter(d: &str) -> f64 {
    match d {
        "easy" => 0.20,
        "pro" => 0.80,
        _ => 0.45,
    }
}

const PIN_SETUP: [(i64, f64, f64); 10] = [
    (1, 0.0, PIN_Y),
    (2, -PIN_DX, PIN_Y + ROW_DY),
    (3, PIN_DX, PIN_Y + ROW_DY),
    (4, -2.0 * PIN_DX, PIN_Y + 2.0 * ROW_DY),
    (5, 0.0, PIN_Y + 2.0 * ROW_DY),
    (6, 2.0 * PIN_DX, PIN_Y + 2.0 * ROW_DY),
    (7, -3.0 * PIN_DX, PIN_Y + 3.0 * ROW_DY),
    (8, -PIN_DX, PIN_Y + 3.0 * ROW_DY),
    (9, PIN_DX, PIN_Y + 3.0 * ROW_DY),
    (10, 3.0 * PIN_DX, PIN_Y + 3.0 * ROW_DY),
];

#[derive(Clone, Copy)]
struct Pin {
    num: i64,
    x: f64,
    y: f64,
    hx: f64,
    hy: f64,
    vx: f64,
    vy: f64,
    down: bool,
    spin: f64,
}

impl Pin {
    fn new(num: i64, x: f64, y: f64) -> Pin {
        Pin { num, x, y, hx: x, hy: y, vx: 0.0, vy: 0.0, down: false, spin: 0.0 }
    }
    fn moved(&self) -> f64 {
        py::hypot(self.x - self.hx, self.y - self.hy)
    }
}

#[derive(Clone, Copy)]
struct Ball {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    spin: f64,
    gutter: bool,
    roll: f64,
}

/// Offizielle Wertung; je Frame die laufende Summe (`None` = offen).
fn score_frames(rolls: &[i64]) -> Vec<Option<i64>> {
    let mut out = Vec::with_capacity(10);
    let mut total = 0;
    let mut i = 0;
    for _ in 0..10 {
        if i >= rolls.len() {
            out.push(None);
            continue;
        }
        if rolls[i] == 10 {
            if i + 2 < rolls.len() {
                total += 10 + rolls[i + 1] + rolls[i + 2];
                out.push(Some(total));
            } else {
                out.push(None);
            }
            i += 1;
        } else if i + 1 < rolls.len() {
            if rolls[i] + rolls[i + 1] == 10 {
                if i + 2 < rolls.len() {
                    total += 10 + rolls[i + 2];
                    out.push(Some(total));
                } else {
                    out.push(None);
                }
            } else {
                total += rolls[i] + rolls[i + 1];
                out.push(Some(total));
            }
            i += 2;
        } else {
            out.push(None);
            i += 1;
        }
    }
    out
}

fn total_score(rolls: &[i64]) -> i64 {
    score_frames(rolls).into_iter().flatten().last().unwrap_or(0)
}

fn rolls_in_frame(rolls: &[i64], frame: usize) -> usize {
    let mut i = 0;
    for f in 0..10 {
        let start = i;
        if i >= rolls.len() {
            return 0;
        }
        if f == 9 {
            return rolls.len() - start;
        }
        if rolls[i] == 10 {
            i += 1;
        } else {
            i += 2;
        }
        if f == frame {
            return rolls.len().min(i) - start;
        }
    }
    0
}

fn frame_cells(rolls: &[i64]) -> Vec<Vec<String>> {
    let mut cells = vec![Vec::new(); 10];
    let mut i = 0;
    let num = |v: i64| if v == 0 { "-".to_string() } else { v.to_string() };
    for (f, cell) in cells.iter_mut().enumerate() {
        if i >= rolls.len() {
            break;
        }
        if f < 9 {
            if rolls[i] == 10 {
                *cell = vec!["X".into()];
                i += 1;
                continue;
            }
            let a = rolls[i];
            *cell = vec![num(a)];
            i += 1;
            if i < rolls.len() {
                let b = rolls[i];
                cell.push(if a + b == 10 { "/".into() } else { num(b) });
                i += 1;
            }
        } else {
            let mut prev: Option<i64> = None;
            while i < rolls.len() && cell.len() < 3 {
                let v = rolls[i];
                if v == 10 {
                    cell.push("X".into());
                } else if prev.is_some_and(|p| p + v == 10 && p != 10) {
                    cell.push("/".into());
                } else {
                    cell.push(num(v));
                }
                prev = Some(v);
                i += 1;
            }
        }
    }
    cells
}

#[derive(Clone, Copy, PartialEq)]
enum State {
    Setup,
    Play,
    Over,
}

/// `BowlingGame`
pub struct BowlingGame {
    b: GameBase,
    diff: &'static str,
    guide: bool,
    rec: Option<Recorder>,
    replay: Option<Value>,
    replay_request: Option<Value>,
    rep: Option<Value>,
    rep_at: Option<(usize, usize)>,
    rec_pins: Vec<Option<(Value, Value, i64)>>,
    small: Font,
    tiny: Font,
    card: Font,
    huge: Font,
    hud_h: i32,
    card_h: i32,
    cx: f64,
    depth: f64,
    hor: f64,
    ky: f64,
    kx: f64,
    diff_rects: Vec<Rect>,
    guide_rects: Vec<Rect>,
    start_rect: Rect,
    best: Map<String, Value>,
    state: State,
    players: usize,
    rolls: Vec<Vec<i64>>,
    frame: Vec<usize>,
    player: usize,
    winner: Option<usize>,
    strike_run: i64,
    msg: Option<String>,
    msg_t: f64,
    pins: Vec<Pin>,
    step: usize,
    t_osc: f64,
    manual: bool,
    vals: [f64; 4],
    ball: Option<Ball>,
    roll_t: f64,
    pins_before: i64,
    result_key: Option<String>,
}

impl BowlingGame {
    /// `BowlingGame(...)`
    pub fn new(b: GameBase) -> Box<dyn Game> {
        let f = b.font.clone();
        let z = Rect::new(0, 0, 0, 0);
        let mut g = BowlingGame {
            b,
            diff: "normal",
            guide: true,
            rec: None,
            replay: None,
            replay_request: None,
            rep: None,
            rep_at: None,
            rec_pins: Vec::new(),
            small: f.clone(),
            tiny: f.clone(),
            card: f.clone(),
            huge: f,
            hud_h: 40,
            card_h: 34,
            cx: 0.0,
            depth: 340.0,
            hor: 0.0,
            ky: 0.0,
            kx: 0.0,
            diff_rects: Vec::new(),
            guide_rects: Vec::new(),
            start_rect: z,
            best: Map::new(),
            state: State::Setup,
            players: 1,
            rolls: vec![Vec::new()],
            frame: vec![0],
            player: 0,
            winner: None,
            strike_run: 0,
            msg: None,
            msg_t: 0.0,
            pins: Vec::new(),
            step: 0,
            t_osc: 0.0,
            manual: false,
            vals: [0.0, 0.0, 0.0, 0.5],
            ball: None,
            roll_t: 0.0,
            pins_before: 10,
            result_key: None,
        };
        g.reset();
        Box::new(g)
    }

    fn build_fonts(&mut self) {
        let h = self.b.height;
        self.small = ui::f(14.max(h / 32));
        self.tiny = ui::f(11.max(h / 42));
        self.card = ui::font(10.max(h / 44), false, true);
        self.huge = ui::fb(24.max(h / 13));
    }

    fn layout(&mut self) {
        let (w, h) = (self.b.width, self.b.height);
        self.hud_h = 40;
        self.card_h = 34.max(self.card.get_height() * 2 + 12);
        let top = self.hud_h + self.card_h + 6;
        let near_y = (h - 12) as f64;
        let far_y = (top + 18) as f64;
        self.cx = w as f64 / 2.0;
        self.depth = 340.0;
        let k = (PIT_Y + self.depth) / self.depth;
        self.hor = (k * far_y - near_y) / (k - 1.0);
        self.ky = (near_y - self.hor) * self.depth;
        let half = LW / 2.0 + GUT;
        self.kx = (w as f64 * 0.43) * self.depth / half;
    }

    fn load_best() -> Map<String, Value> {
        let data = store::load_section("bowling");
        let mut out = Map::new();
        if let Some(Value::Object(best)) = data.get("best") {
            for (k, v) in best {
                let n = match v {
                    Value::Number(n) => n.as_i64().or_else(|| n.as_f64().map(|f| f as i64)),
                    Value::Bool(b) => Some(*b as i64),
                    Value::String(s) => s.trim().parse::<i64>().ok(),
                    _ => None,
                };
                if let Some(n) = n {
                    out.insert(k.clone(), json!(n));
                }
            }
        }
        out
    }

    fn best_of(&self, d: &str) -> i64 {
        self.best.get(d).and_then(|v| v.as_i64()).unwrap_or(0)
    }

    fn save_best(&mut self, score: i64) {
        if score > self.best_of(self.diff) {
            self.best.insert(self.diff.into(), json!(score));
            let mut sec = Map::new();
            sec.insert("best".into(), Value::Object(self.best.clone()));
            store::save_section("bowling", sec);
        }
    }

    fn new_game(&mut self) {
        self.players = if self.b.multiplayer { 2 } else { 1 };
        self.rolls = vec![Vec::new(); self.players];
        self.frame = vec![0; self.players];
        self.player = 0;
        self.winner = None;
        self.strike_run = 0;
        self.b.score = 0;
        self.b.game_over = false;
        self.msg = None;
        self.msg_t = 0.0;
        self.rec_new();
        self.rack(true);
        self.new_delivery();
    }

    fn rack(&mut self, full: bool) {
        if full {
            self.pins = PIN_SETUP.iter().map(|(n, x, y)| Pin::new(*n, *x, *y)).collect();
        } else {
            self.pins.retain(|p| !p.down);
            for p in self.pins.iter_mut() {
                p.x = p.hx;
                p.y = p.hy;
                p.vx = 0.0;
                p.vy = 0.0;
                p.spin = 0.0;
            }
        }
    }

    fn new_delivery(&mut self) {
        self.step = 0;
        self.t_osc = 0.0;
        self.manual = false;
        self.vals = [0.0, 0.0, 0.0, 0.5];
        self.ball = None;
        self.roll_t = 0.0;
        self.pins_before = self.pins.iter().filter(|p| !p.down).count() as i64;
        self.result_key = None;
    }

    fn build_setup_layout(&mut self) {
        let cx = self.b.width / 2;
        let bw = 370.min(self.b.width - 50);
        let y0 = (self.b.height as f64 * 0.30) as i32;
        let gap = 8;
        let row = |y: i32, n: i32| -> Vec<Rect> {
            let cw = (bw - gap * (n - 1)) as f64 / n as f64;
            (0..n).map(|i| Rect::new((cx as f64 - bw as f64 / 2.0 + i as f64 * (cw + gap as f64)) as i32, y, cw as i32, 42)).collect()
        };
        self.diff_rects = row(y0, 3);
        self.guide_rects = row(y0 + 88, 2);
        self.start_rect = Rect::new(cx - 95, y0 + 156, 190, 46);
    }

    fn handle_setup(&mut self, ev: &InputEvent) {
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            match k {
                "1" | "2" | "3" => {
                    self.diff = DIFFS[k.parse::<usize>().unwrap_or(2) - 1];
                    self.b.set_opt("bowling", "difficulty", self.diff.into());
                    self.b.play_sound("click");
                }
                "g" | "G" => self.toggle_guide(),
                "Return" | "space" => self.start_play(),
                _ => {}
            }
        } else if ev.kind == Kind::MouseDown {
            if let Some(i) = self.diff_rects.iter().position(|r| r.hit(ev.pos)) {
                self.diff = DIFFS[i];
                self.b.set_opt("bowling", "difficulty", self.diff.into());
                self.b.play_sound("click");
                return;
            }
            if let Some(i) = self.guide_rects.iter().position(|r| r.hit(ev.pos)) {
                if self.guide != (i == 0) {
                    self.toggle_guide();
                }
                return;
            }
            if self.start_rect.hit(ev.pos) {
                self.start_play();
            }
        }
    }

    fn toggle_guide(&mut self) {
        self.guide = !self.guide;
        self.b.set_opt("bowling", "guide", self.guide.into());
        self.b.play_sound("select");
    }

    fn start_play(&mut self) {
        self.new_game();
        self.state = State::Play;
        self.b.play_sound("click");
    }

    fn adjust(&mut self, d: f64) {
        self.manual = true;
        let lo: f64 = if STEPS[self.step] == "power" { 0.0 } else { -1.0 };
        self.vals[self.step] = lo.max(1.0f64.min(self.vals[self.step] + d));
        self.b.play_sound("move");
    }

    fn confirm_step(&mut self) {
        self.b.play_sound("click");
        self.step += 1;
        self.manual = false;
        self.t_osc = 0.0;
        if self.step >= STEPS.len() {
            self.throw();
        }
    }

    fn throw(&mut self) {
        let sc = scatter(self.diff);
        let [pos, aim, spin, power] = self.vals;
        let x0 = pos * (LW / 2.0 - BALL_R - 1.0);
        let ang = py::radians(aim * 4.2 + py::uniform(-sc, sc) * 0.6);
        let speed = 240.0 + 210.0 * power;
        let bspin = spin + py::uniform(-sc, sc) * 0.10;
        self.ball = Some(Ball { x: x0, y: 0.0, vx: ang.sin() * speed, vy: ang.cos() * speed, spin: bspin, gutter: false, roll: 0.0 });
        self.roll_t = 0.0;
        if self.rec.is_some() {
            let pins: Vec<Value> = self.pins.iter().map(|p| json!([p.num, rnd(p.x, 1), rnd(p.y, 1)])).collect();
            self.rec_pins = pins.iter().map(|pn| Some((pn[1].clone(), pn[2].clone(), 0))).collect();
            let mut info = Map::new();
            info.insert("player".into(), json!(self.player));
            info.insert("frame".into(), json!(self.frame[self.player]));
            info.insert("pins".into(), Value::Array(pins));
            info.insert("pos".into(), rnd(pos, 3));
            info.insert("aim".into(), rnd(aim, 3));
            info.insert("spin".into(), rnd(spin, 3));
            info.insert("power".into(), rnd(power, 3));
            if let Some(r) = self.rec.as_mut() {
                r.scene(false, info);
            }
        }
        self.b.play_sound("shoot");
        self.b.rumble(80);
    }

    fn step_ball(&mut self, dt: f64) {
        let Some(mut b) = self.ball else { return };
        let speed = py::hypot(b.vx, b.vy) + 1.0;
        let steps = 2.max(24.min((speed * dt / PIN_R) as i64 + 2));
        let h = dt / steps as f64;
        for _ in 0..steps {
            if !b.gutter && b.y > OIL_END {
                let grip = 1.0f64.min((b.y - OIL_END) / 180.0);
                b.vx += b.spin * HOOK * grip * h;
            }
            let f = 0f64.max(1.0 - BALL_FRIC * h);
            b.vx *= f;
            b.vy *= f;
            b.x += b.vx * h;
            b.y += b.vy * h;
            b.roll += b.vy * h / 24.0;
            let limit = LW / 2.0 - BALL_R;
            if !b.gutter && b.x.abs() > limit {
                b.gutter = true;
                b.vx = 0.0;
                b.spin = 0.0;
                b.x = (LW / 2.0 + GUT / 2.0 - BALL_R * 0.4).copysign(b.x);
                self.b.play_sound("hit");
            }
            if !b.gutter {
                for i in 0..self.pins.len() {
                    self.hit_pin(&mut b, i);
                }
            }
        }
        self.ball = Some(b);
    }

    fn hit_pin(&mut self, b: &mut Ball, i: usize) {
        let p = &mut self.pins[i];
        let (dx, dy) = (p.x - b.x, p.y - b.y);
        let d = py::hypot(dx, dy);
        let rad = BALL_R + PIN_R;
        if d >= rad || d < 1e-9 {
            return;
        }
        let (ux, uy) = (dx / d, dy / d);
        let overlap = rad - d;
        p.x += ux * overlap;
        p.y += uy * overlap;
        let (rvx, rvy) = (p.vx - b.vx, p.vy - b.vy);
        let vn = rvx * ux + rvy * uy;
        if vn > 0.0 {
            return;
        }
        let j = -(1.0 + REST) * vn / (1.0 / PIN_M + 1.0 / BALL_M);
        p.vx += j * ux / PIN_M;
        p.vy += j * uy / PIN_M;
        b.vx -= j * ux / BALL_M;
        b.vy -= j * uy / BALL_M;
        p.spin += py::uniform(-3.0, 3.0);
        if !p.down {
            self.b.play_sound("lock");
        }
    }

    fn step_pins(&mut self, dt: f64) {
        let steps = 3;
        let h = dt / steps as f64;
        for _ in 0..steps {
            for p in self.pins.iter_mut() {
                if p.vx == 0.0 && p.vy == 0.0 {
                    continue;
                }
                let f = 0f64.max(1.0 - PIN_FRIC * h);
                p.vx *= f;
                p.vy *= f;
                p.x += p.vx * h;
                p.y += p.vy * h;
                if p.vx.abs() + p.vy.abs() < 1.2 {
                    p.vx = 0.0;
                    p.vy = 0.0;
                }
            }
            let n = self.pins.len();
            for i in 0..n {
                for k in i + 1..n {
                    let (lo, hi) = self.pins.split_at_mut(k);
                    pin_pin(&mut lo[i], &mut hi[0]);
                }
            }
            for p in self.pins.iter_mut() {
                if !p.down && p.moved() > DOWN_DIST {
                    p.down = true;
                }
            }
        }
    }

    fn roll_done(&self) -> bool {
        let Some(b) = self.ball else { return false };
        if b.y > PIT_Y {
            return self.pins.iter().all(|p| p.vx == 0.0 && p.vy == 0.0);
        }
        self.roll_t > MAX_ROLL_TIME
    }

    fn rec_sample(&mut self) -> Option<Vec<Value>> {
        let b = self.ball?;
        let mut out = vec![rnd(b.x, 1), rnd(b.y, 1), rnd(b.roll, 2)];
        if self.rec_pins.len() < self.pins.len() {
            self.rec_pins.resize(self.pins.len(), None);
        }
        for (i, p) in self.pins.iter().enumerate() {
            let state = (rnd(p.x, 1), rnd(p.y, 1), if p.down { 1 } else { 0 });
            if self.rec_pins[i].as_ref() != Some(&state) {
                self.rec_pins[i] = Some(state.clone());
                out.push(json!(i));
                out.push(state.0);
                out.push(state.1);
                out.push(json!(state.2));
            }
        }
        Some(out)
    }

    fn finish_roll(&mut self) {
        let knocked = self.pins.iter().filter(|p| p.down).count() as i64;
        let standing = self.pins_before - knocked;
        self.rolls[self.player].push(knocked);
        let frame = self.frame[self.player];
        let strike = knocked == 10 && self.pins_before == 10;
        let spare = !strike && standing == 0;
        let key = if strike {
            self.strike_run += 1;
            self.b.play_sound("win");
            if self.strike_run >= 3 {
                self.b.ach_event("bowl_turkey", None);
            }
            "bowl.strike"
        } else if spare {
            self.strike_run = 0;
            self.b.play_sound("level");
            "bowl.spare"
        } else {
            self.strike_run = 0;
            self.b.play_sound(if knocked > 0 { "point" } else { "hit" });
            if knocked > 0 {
                "bowl.pins"
            } else {
                "bowl.miss"
            }
        };
        self.result_key = Some(key.into());
        self.msg = Some(tf(key, &[("n", &knocked.to_string())]));
        self.msg_t = 1.8;
        if let Some(mut r) = self.rec.take() {
            let last = self.rec_sample();
            let mut info = Map::new();
            info.insert("knocked".into(), json!(knocked));
            info.insert("result".into(), json!(key));
            r.close(last, info);
            self.rec = Some(r);
        }
        let rolls = self.rolls[self.player].clone();
        let (done, refill) = self.frame_state(frame, &rolls, strike, spare);
        self.ball = None;
        if done {
            self.frame[self.player] += 1;
            if self.frame[self.player] >= 10 {
                self.next_player_or_end();
                return;
            }
            self.rack(true);
            if self.players > 1 {
                self.switch_player();
            }
            self.new_delivery();
        } else {
            self.rack(refill);
            self.new_delivery();
        }
        self.sync_score();
    }

    fn frame_state(&self, frame: usize, rolls: &[i64], strike: bool, _spare: bool) -> (bool, bool) {
        let in_frame = rolls_in_frame(rolls, frame);
        if frame < 9 {
            if strike {
                return (true, true);
            }
            return (in_frame >= 2, false);
        }
        if in_frame == 1 {
            return (false, strike);
        }
        if in_frame == 2 {
            let (first, second) = (rolls[rolls.len() - 2], rolls[rolls.len() - 1]);
            if first == 10 {
                return (false, second == 10);
            }
            if first + second == 10 {
                return (false, true);
            }
            return (true, true);
        }
        (true, true)
    }

    fn switch_player(&mut self) {
        for i in 1..=self.players {
            let nxt = (self.player + i) % self.players;
            if self.frame[nxt] < 10 {
                self.player = nxt;
                return;
            }
        }
    }

    fn next_player_or_end(&mut self) {
        if self.frame.iter().all(|f| *f >= 10) {
            self.end_game();
            return;
        }
        self.switch_player();
        self.rack(true);
        self.new_delivery();
        self.sync_score();
    }

    fn end_game(&mut self) {
        let fin = total_score(&self.rolls[0]);
        self.b.score = fin;
        self.save_best(fin);
        if fin >= 200 {
            self.b.ach_event("bowl_200", Some(fin));
        }
        if self.b.multiplayer {
            let (s0, s1) = (total_score(&self.rolls[0]), total_score(&self.rolls[1]));
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
            self.b.report_result(fin >= 100);
        }
        if let Some(mut r) = self.rec.take() {
            let mut meta = Map::new();
            meta.insert("score".into(), json!(fin));
            self.replay = r.result(&t(&format!("bowl.diff.{}", self.diff)), &tf("bowl.final", &[("n", &fin.to_string())]), meta);
        }
        self.state = State::Over;
        self.b.game_over = true;
        self.b.play_sound("gameover");
    }

    fn restart(&mut self) {
        self.new_game();
        self.state = State::Play;
        self.b.play_sound("click");
    }

    fn step_name(&self) -> &'static str {
        STEPS[self.step.min(STEPS.len() - 1)]
    }

    fn sync_score(&mut self) {
        self.b.score = total_score(&self.rolls[0]);
    }

    fn rec_new(&mut self) {
        self.replay = None;
        self.rec_pins.clear();
        let mut meta = Map::new();
        meta.insert("diff".into(), json!(self.diff));
        meta.insert("players".into(), json!(self.players));
        self.rec = replay::recorder("bowling", shell::replay_enabled(&self.b.settings), meta);
    }

    fn open_replay(&mut self) {
        if self.replay.is_some() {
            self.replay_request = self.replay.clone();
            self.b.play_sound("click");
        }
    }

    fn project(&self, x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        let inv = 1.0 / (y + self.depth);
        let s = self.kx * inv;
        (self.cx + x * s, self.hor + self.ky * inv - z * s, s)
    }

    // ------------------------------------------------- Replay-Wiedergabe

    /// Schaltet diese Instanz auf reine Wiedergabe um.
    pub fn replay_begin(&mut self, rep: &Value) {
        self.rec = None;
        self.replay = None;
        self.replay_request = None;
        self.rep = Some(rep.clone());
        self.rep_at = None;
        let meta = rep.get("meta").cloned().unwrap_or(json!({}));
        if let Some(d) = meta.get("diff").and_then(|v| v.as_str()) {
            if let Some(x) = DIFFS.iter().find(|x| **x == d) {
                self.diff = x;
            }
        }
        let pl = meta.get("players").and_then(|v| v.as_i64()).filter(|n| *n != 0).unwrap_or(1);
        self.players = pl.clamp(1, 2) as usize;
        self.b.multiplayer = self.players > 1;
        self.state = State::Play;
        self.b.game_over = false;
        self.msg = None;
        self.msg_t = 0.0;
        self.manual = true;
        self.step = STEPS.len() - 1;
        self.replay_seek(0, 0);
    }

    /// Setzt Pins, Ball und Scorecard auf Szene `index`, Sample `frame`.
    pub fn replay_seek(&mut self, index: usize, frame: usize) {
        let Some(rep) = self.rep.clone() else { return };
        let Some(scenes) = rep.get("scenes").and_then(|v| v.as_array()).cloned() else { return };
        if scenes.is_empty() {
            return;
        }
        let index = index.min(scenes.len() - 1);
        let sc = &scenes[index];
        let frames = sc.get("f").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let n = 1.max(frames.len());
        let frame = frame.min(n - 1);
        let last = frame >= n - 1;
        let gi = |v: &Value, k: &str, d: i64| v.get(k).and_then(|x| x.as_i64()).unwrap_or(d);
        let gf = |v: &Value, k: &str, d: f64| v.get(k).and_then(|x| x.as_f64()).unwrap_or(d);
        self.rolls = vec![Vec::new(); self.players];
        self.frame = vec![0; self.players];
        let pmax = self.players as i64 - 1;
        let upto = if last { index + 1 } else { index };
        for prev in &scenes[..upto] {
            let p = gi(prev, "player", 0).clamp(0, pmax) as usize;
            if let Some(k) = prev.get("knocked").and_then(|v| v.as_i64()) {
                self.rolls[p].push(k);
            }
        }
        for prev in &scenes[..=index] {
            let p = gi(prev, "player", 0).clamp(0, pmax) as usize;
            self.frame[p] = gi(prev, "frame", 0) as usize;
        }
        self.player = gi(sc, "player", 0).clamp(0, pmax) as usize;
        self.vals = [gf(sc, "pos", 0.0), gf(sc, "aim", 0.0), gf(sc, "spin", 0.0), gf(sc, "power", 0.5)];
        self.result_key = sc.get("result").and_then(|v| v.as_str()).map(String::from);
        let start = match self.rep_at {
            Some((i, f)) if i == index && frame >= f => f + 1,
            _ => {
                self.pins = sc
                    .get("pins")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().map(|p| Pin::new(p[0].as_i64().unwrap_or(0), p[1].as_f64().unwrap_or(0.0), p[2].as_f64().unwrap_or(0.0))).collect())
                    .unwrap_or_default();
                0
            }
        };
        for fr in frames.iter().take(frame + 1).skip(start) {
            let fr = fr.as_array().cloned().unwrap_or_default();
            let mut j = 3;
            while j + 3 < fr.len() {
                let i = fr[j].as_i64().unwrap_or(-1);
                if (0..self.pins.len() as i64).contains(&i) {
                    let p = &mut self.pins[i as usize];
                    p.x = fr[j + 1].as_f64().unwrap_or(p.x);
                    p.y = fr[j + 2].as_f64().unwrap_or(p.y);
                    p.down = fr[j + 3].as_f64().unwrap_or(0.0) != 0.0;
                }
                j += 4;
            }
        }
        let fr = frames.get(frame).and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let at = |k: usize| fr.get(k).and_then(|v| v.as_f64()).unwrap_or(0.0);
        self.ball = Some(Ball { x: at(0), y: at(1), vx: 0.0, vy: 0.0, spin: 0.0, gutter: false, roll: at(2) });
        self.msg = match (last, sc.get("result").and_then(|v| v.as_str())) {
            (true, Some(r)) => Some(tf(r, &[("n", &gi(sc, "knocked", 0).to_string())])),
            _ => None,
        };
        self.rep_at = Some((index, frame));
    }

    /// Zeichnet den aktuellen Replay-Stand (ohne Menue-Overlay).
    pub fn replay_draw(&mut self, aiming: bool) {
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        let ball = self.ball;
        if aiming {
            self.ball = None;
        }
        self.draw_lane(&mut s);
        self.draw_pins(&mut s);
        if self.ball.is_some() {
            self.draw_ball(&mut s);
        } else {
            self.draw_controls(&mut s);
        }
        self.draw_hud(&mut s);
        self.ball = ball;
        self.draw_card(&mut s);
    }

    // ------------------------------------------------- Zeichnen

    fn quad(&self, s: &mut Surface, x0: f64, x1: f64, col: Color) {
        let a = self.project(x0, 0.0, 0.0);
        let b = self.project(x1, 0.0, 0.0);
        let c = self.project(x1, PIT_Y, 0.0);
        let d = self.project(x0, PIT_Y, 0.0);
        draw::polygon(s, col, &[(a.0, a.1), (b.0, b.1), (c.0, c.1), (d.0, d.1)], 0);
    }

    fn draw_lane(&self, s: &mut Surface) {
        let far = self.project(0.0, PIT_Y, 0.0);
        let top = (self.hud_h + self.card_h + 4) as f64;
        draw::rect(s, COL_BACK, (0.0, top, self.b.width as f64, far.1 - (self.hud_h + self.card_h) as f64), 0);
        self.quad(s, -LW / 2.0 - GUT, -LW / 2.0, COL_GUTTER);
        self.quad(s, LW / 2.0, LW / 2.0 + GUT, COL_GUTTER);
        self.quad(s, -LW / 2.0, LW / 2.0, COL_LANE);
        for i in 1..10 {
            let x = -LW / 2.0 + LW * i as f64 / 10.0;
            let a = self.project(x, 0.0, 0.0);
            let b = self.project(x, PIT_Y, 0.0);
            draw::line(s, COL_BOARD, (a.0, a.1), (b.0, b.1), 1);
        }
        let a = self.project(-LW / 2.0, PIN_Y - 26.0, 0.0);
        let b = self.project(LW / 2.0, PIN_Y - 26.0, 0.0);
        let c = self.project(LW / 2.0, PIT_Y, 0.0);
        let d = self.project(-LW / 2.0, PIT_Y, 0.0);
        draw::polygon(s, COL_DECK, &[(a.0, a.1), (b.0, b.1), (c.0, c.1), (d.0, d.1)], 0);
        for i in -3..4 {
            let x = i as f64 * 5.0;
            let y = 180.0 + (i as f64).abs() * 22.0;
            let p0 = self.project(x, y, 0.0);
            let p1 = self.project(x - 1.6, y - 16.0, 0.0);
            let p2 = self.project(x + 1.6, y - 16.0, 0.0);
            draw::polygon(s, COL_ARROW, &[(p0.0, p0.1), (p1.0, p1.1), (p2.0, p2.1)], 0);
        }
        let fa = self.project(-LW / 2.0 - GUT, 0.0, 0.0);
        let fb = self.project(LW / 2.0 + GUT, 0.0, 0.0);
        draw::line(s, COL_FOUL, (fa.0, fa.1), (fb.0, fb.1), 3);
        draw::line(s, COL_GUTTER_D, (fa.0, fa.1 + 3.0), (fb.0, fb.1 + 3.0), 1);
    }

    fn draw_pins(&self, s: &mut Surface) {
        let mut order: Vec<&Pin> = self.pins.iter().collect();
        order.sort_by(|a, b| (-a.y).partial_cmp(&(-b.y)).unwrap_or(std::cmp::Ordering::Equal));
        for p in order {
            if p.down && p.vx == 0.0 && p.vy == 0.0 && p.moved() > 40.0 {
                continue;
            }
            let (px, py_, sc) = self.project(p.x, p.y, 0.0);
            let r = 2.max((PIN_R * sc) as i32);
            if p.down {
                let w = 3.max((PIN_R * 2.6 * sc) as i32);
                let h = 2.max((PIN_R * 1.3 * sc) as i32);
                let (x0, y0) = (px - (w / 2) as f64, py_ - (h / 2) as f64);
                draw::ellipse(s, COL_PIN_D, (x0, y0, w as f64, h as f64), 0);
                draw::ellipse(s, COL_PIN_RED, (x0, y0, w as f64, 1.max(h / 3) as f64), 0);
                continue;
            }
            let rf = r as f64;
            let body = [
                (px - rf, py_),
                (px - rf * 0.75, py_ - rf * 2.2),
                (px - rf * 0.42, py_ - rf * 3.4),
                (px - rf * 0.62, py_ - rf * 4.6),
                (px, py_ - rf * 5.6),
                (px + rf * 0.62, py_ - rf * 4.6),
                (px + rf * 0.42, py_ - rf * 3.4),
                (px + rf * 0.75, py_ - rf * 2.2),
                (px + rf, py_),
            ];
            draw::polygon(s, COL_PIN, &body, 0);
            draw::polygon(s, COL_PIN_D, &body, 1);
            if r >= 3 {
                draw::line(s, COL_PIN_RED, (px - rf * 0.55, py_ - rf * 3.0), (px + rf * 0.55, py_ - rf * 3.0), 1.max(r / 2));
            }
        }
    }

    fn draw_ball(&self, s: &mut Surface) {
        let Some(b) = self.ball else { return };
        let (px, py_, sc) = self.project(b.x, b.y, BALL_R);
        let r = 3.max((BALL_R * sc) as i32);
        let rf = r as f64;
        let sh = self.project(b.x, b.y, 0.0);
        draw::ellipse(s, rgb(120, 96, 60), (sh.0 - rf, sh.1 - rf * 0.4, rf * 2.0, rf * 0.8), 0);
        draw::circle(s, COL_BALL, (px as i32, py_ as i32), r, 0);
        draw::circle(s, COL_BALL_HI, ((px - rf * 0.35) as i32, (py_ - rf * 0.35) as i32), 1.max((rf * 0.32) as i32), 0);
        if r >= 6 {
            for k in 0..3 {
                let a = b.roll + k as f64 * 2.1;
                let hx = px + a.cos() * rf * 0.45;
                let hy = py_ + a.sin() * rf * 0.45 * 0.6;
                draw::circle(s, rgb(18, 22, 60), (hx as i32, hy as i32), 1.max((rf * 0.14) as i32), 0);
            }
        }
    }

    fn draw_controls(&self, s: &mut Surface) {
        let p = ui::pal();
        let [pos, aim, spin, _] = self.vals;
        let x0 = pos * (LW / 2.0 - BALL_R - 1.0);
        let p0 = self.project(x0, 0.0, 0.0);
        let r = 3.max((BALL_R * p0.2) as i32);
        draw::circle(s, COL_BALL, (p0.0 as i32, (p0.1 - r as f64) as i32), r, 0);
        if self.guide && self.step >= 1 {
            let ang = py::radians(aim * 4.2);
            let mut pts = Vec::with_capacity(46);
            let (mut x, mut y) = (x0, 0.0);
            let (mut vx, vy) = (ang.sin(), ang.cos());
            let sp = if self.step >= 2 { spin } else { 0.0 };
            for _ in 0..46 {
                let step = PIN_Y / 46.0;
                if y > OIL_END {
                    vx += sp * 0.055 * 1.0f64.min((y - OIL_END) / 180.0);
                }
                x += vx * step;
                y += vy * step;
                let pr = self.project((-LW).max(LW.min(x)), y, 0.0);
                pts.push((pr.0, pr.1));
            }
            let mut i = 0;
            while i + 1 < pts.len() {
                draw::line(s, COL_MARK, pts[i], pts[i + 1], 2);
                i += 2;
            }
        }
        let name = self.step_name();
        let val = self.vals[self.step.min(STEPS.len() - 1)];
        let bw = 240.min(self.b.width - 60);
        let bx = self.b.width / 2 - bw / 2;
        let by = self.b.height - 26;
        draw::rect_r(s, p.panel, (bx - 6, by - 16, bw + 12, 34), 0, 8);
        draw::rect_r(s, p.btn, (bx, by, bw, 10), 0, 5);
        if name == "power" {
            draw::rect_r(s, self.b.accent, (bx, by, (bw as f64 * val) as i32, 10), 0, 5);
        } else {
            let mid = bx + bw / 2;
            draw::line(s, p.border, (mid, by - 3), (mid, by + 13), 1);
            let mx = mid + (val * bw as f64 / 2.0) as i32;
            draw::rect_r(s, self.b.accent, (mx - 4, by - 3, 8, 16), 0, 3);
        }
        let lbl = self.tiny.render(&t(&format!("bowl.step.{name}")), true, p.text);
        s.blr(&lbl, lbl.get_rect().midbottom_at((self.b.width / 2, by - 4)));
    }

    fn draw_hud(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        draw::rect(s, p.panel, (0, 0, w, self.hud_h), 0);
        draw::line(s, p.border, (0, self.hud_h), (w, self.hud_h), 1);
        let cy = self.hud_h / 2;
        let left = self.tiny.render(&tf("bowl.frame", &[("n", &(self.frame[self.player] + 1).min(10).to_string())]), true, p.text_dim);
        s.blr(&left, left.get_rect().midleft_at((12, cy)));
        let (mid, col) = if let Some(m) = &self.msg {
            (m.clone(), p.gold)
        } else if self.b.multiplayer {
            (self.b.pname(self.player).chars().take(10).collect(), self.b.accent)
        } else if self.ball.is_none() {
            (t(&format!("bowl.step.{}", self.step_name())), p.text_dim)
        } else {
            (t("bowl.rolling"), p.text_dim)
        };
        let m = self.small.render(&mid, true, col);
        s.blr(&m, m.get_rect().center_at((w / 2, cy)));
        let tot = self.small.render(&total_score(&self.rolls[self.player]).to_string(), true, self.b.accent);
        s.blr(&tot, tot.get_rect().midright_at((w - 12, cy)));
    }

    fn draw_card(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let y = self.hud_h + 2;
        let h = self.card_h - 4;
        draw::rect_r(s, p.panel, (4, y, w - 8, h), 0, 6);
        draw::rect_r(s, p.border, (4, y, w - 8, h), 1, 6);
        let rolls = &self.rolls[self.player];
        let sums = score_frames(rolls);
        let cells = frame_cells(rolls);
        let fw = (w - 16) as f64 / 10.0;
        for f in 0..10 {
            let fx = 8.0 + f as f64 * fw;
            if f > 0 {
                draw::line(s, p.border, (fx, (y + 2) as f64), (fx, (y + h - 2) as f64), 1);
            }
            let active = f == self.frame[self.player];
            let marks = cells[f].join(" ");
            let im = self.card.render(&marks, true, if active { p.text } else { p.text_dim });
            s.blr(&im, im.get_rect().midtop_at((fx + fw / 2.0, (y + 3) as f64)));
            if let Some(val) = sums[f] {
                let v = self.card.render(&val.to_string(), true, if active { self.b.accent } else { p.text_dim });
                s.blr(&v, v.get_rect().midbottom_at((fx + fw / 2.0, (y + h - 3) as f64)));
            }
        }
    }

    fn draw_over(&self, s: &mut Surface) {
        let p = ui::pal();
        let w = self.b.width;
        let h = 118;
        let mut ov = pg::surface_alpha(w, h);
        ov.fill_c(rgba(14, 12, 10, 214));
        let y = self.b.height / 2 - h / 2;
        s.bl(&ov, (0, y));
        draw::line(s, self.b.accent, (0, y), (w, y), 1);
        draw::line(s, self.b.accent, (0, y + h - 1), (w, y + h - 1), 1);
        let cx = w / 2;
        let head = if self.b.multiplayer {
            let txt = match self.winner {
                None => t("common.draw"),
                Some(wn) => tf("common.player_wins_name", &[("name", &self.b.pname(wn))]),
            };
            self.huge.render(&txt, true, self.b.accent)
        } else {
            self.huge.render(&t("bowl.game_done"), true, self.b.accent)
        };
        s.blr(&head, head.get_rect().center_at((cx, y + 32)));
        let sub = self.small.render(&tf("bowl.final", &[("n", &total_score(&self.rolls[0]).to_string())]), true, p.text);
        s.blr(&sub, sub.get_rect().center_at((cx, y + 66)));
        let best = self.best_of(self.diff);
        if best != 0 {
            let b = self.tiny.render(&tf("bowl.best", &[("n", &best.to_string())]), true, p.gold);
            s.blr(&b, b.get_rect().center_at((cx, y + 88)));
        }
        let mut hint_txt = t("bowl.new_round");
        if self.replay.is_some() {
            hint_txt += &format!("  \u{b7}  {}", t("bowl.replay_hint"));
        }
        let hint = self.tiny.render(&hint_txt, true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, y + 106)));
    }

    fn btn(&self, s: &mut Surface, rc: Rect, text: &str, on: bool) {
        let p = ui::pal();
        draw::rect_r(s, if on { p.btn_sel } else { p.btn }, rc, 0, 8);
        draw::rect_r(s, if on { self.b.accent } else { p.border }, rc, if on { 2 } else { 1 }, 8);
        let im = self.small.render(text, true, if on { p.text } else { p.text_dim });
        s.blr(&im, im.get_rect().center_at(rc.center()));
    }

    fn draw_setup(&self, s: &mut Surface) {
        let p = ui::pal();
        let (w, h) = (self.b.width, self.b.height);
        let cx = w / 2;
        let title = self.huge.render(&t("bowl.title"), true, self.b.accent);
        s.blr(&title, title.get_rect().center_at((cx, (h as f64 * 0.13) as i32)));
        let sub = self.small.render(&t("bowl.subtitle"), true, p.text_dim);
        s.blr(&sub, sub.get_rect().center_at((cx, (h as f64 * 0.20) as i32)));
        let label = |s: &mut Surface, rects: &[Rect], txt: &str| {
            let im = self.tiny.render(txt, true, p.text_dim);
            s.blr(&im, im.get_rect().midbottom_at((cx, rects[0].y - 4)));
        };
        label(s, &self.diff_rects, &t("bowl.lbl_diff"));
        for (i, rc) in self.diff_rects.iter().enumerate() {
            self.btn(s, *rc, &t(&format!("bowl.diff.{}", DIFFS[i])), self.diff == DIFFS[i]);
        }
        label(s, &self.guide_rects, &t("bowl.lbl_guide"));
        for (i, rc) in self.guide_rects.iter().enumerate() {
            self.btn(s, *rc, &if i == 0 { t("common.on") } else { t("common.off") }, self.guide == (i == 0));
        }
        draw::rect_r(s, p.btn_sel, self.start_rect, 0, 9);
        draw::rect_r(s, self.b.accent, self.start_rect, 2, 9);
        let st = self.b.font.render(&t("common.start"), true, p.text);
        s.blr(&st, st.get_rect().center_at(self.start_rect.center()));
        let best = self.best_of(self.diff);
        if best != 0 {
            let b = self.tiny.render(&tf("bowl.best", &[("n", &best.to_string())]), true, p.gold);
            s.blr(&b, b.get_rect().center_at((cx, self.start_rect.bottom() + 22)));
        }
        let hint = self.tiny.render(&t("bowl.setup_hint"), true, p.text_dim);
        s.blr(&hint, hint.get_rect().center_at((cx, h - 14)));
    }
}

fn pin_pin(a: &mut Pin, b: &mut Pin) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let d = py::hypot(dx, dy);
    let rad = PIN_R * 2.0;
    if d >= rad || d < 1e-9 {
        return;
    }
    let (ux, uy) = (dx / d, dy / d);
    let push = (rad - d) / 2.0;
    a.x -= ux * push;
    a.y -= uy * push;
    b.x += ux * push;
    b.y += uy * push;
    let vn = (b.vx - a.vx) * ux + (b.vy - a.vy) * uy;
    if vn > 0.0 {
        return;
    }
    let j = -(1.0 + REST) * vn / (2.0 / PIN_M);
    a.vx -= j * ux / PIN_M;
    a.vy -= j * uy / PIN_M;
    b.vx += j * ux / PIN_M;
    b.vy += j * uy / PIN_M;
}

impl Game for BowlingGame {
    /// Match-Historie: Dauer zaehlt nicht im Setup (Python-Standard
    /// `getattr(self, "state", None) != "setup"`).
    fn match_active(&self) -> bool {
        !matches!(self.state, State::Setup)
    }
    fn replay_begin(&mut self, rep: &Value) -> bool {
        BowlingGame::replay_begin(self, rep);
        true
    }

    fn replay_seek(&mut self, index: usize, frame: usize) {
        BowlingGame::replay_seek(self, index, frame);
    }

    fn replay_draw(&mut self, aiming: bool, banner: bool) {
        let _ = banner;
        BowlingGame::replay_draw(self, aiming);
    }

    fn replay_hud_h(&self) -> i32 {
        self.hud_h + self.card_h
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
        let d = self.b.opt_str("bowling", "difficulty", "normal");
        self.diff = DIFFS.iter().copied().find(|x| *x == d).unwrap_or("normal");
        self.guide = self.b.opt_bool("bowling", "guide", true);
        self.rec = None;
        self.replay = None;
        self.replay_request = None;
        self.rep = None;
        self.rec_pins.clear();
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
        if ev.kind == Kind::KeyDown && ev.key_in(&["g", "G"]) {
            self.toggle_guide();
            return;
        }
        if self.state == State::Over {
            if ev.kind == Kind::KeyDown {
                if ev.key_in(&["Return", "space"]) {
                    self.restart();
                } else if ev.key_in(&["s", "S"]) {
                    self.state = State::Setup;
                    self.b.game_over = false;
                    self.b.play_sound("click");
                } else if ev.key_in(&["p", "P"]) && self.replay.is_some() {
                    self.open_replay();
                }
            } else if ev.kind == Kind::MouseDown && ev.button == 1 {
                self.restart();
            }
            return;
        }
        if self.state != State::Play || self.ball.is_some() {
            return;
        }
        if ev.kind == Kind::KeyDown {
            let k = ev.key.as_str();
            if k == "Left" || self.b.is_action(k, "left", None) {
                self.adjust(-0.06);
            } else if k == "Right" || self.b.is_action(k, "right", None) {
                self.adjust(0.06);
            } else if matches!(k, "space" | "Return") || self.b.is_action(k, "action", None) {
                self.confirm_step();
            }
        } else if ev.kind == Kind::MouseDown && ev.button == 1 {
            self.confirm_step();
        }
    }

    fn update(&mut self, dt: f64) {
        if self.msg_t > 0.0 {
            self.msg_t -= dt;
            if self.msg_t <= 0.0 {
                self.msg = None;
            }
        }
        if self.state != State::Play {
            return;
        }
        if self.ball.is_none() {
            if !self.manual {
                self.t_osc += dt * swing(self.diff);
                let w = (self.t_osc * 2.4).sin();
                self.vals[self.step] = if STEPS[self.step] == "power" { w.abs() } else { w };
            }
            return;
        }
        self.roll_t += dt;
        self.step_ball(dt);
        self.step_pins(dt);
        if let Some(mut r) = self.rec.take() {
            r.tick(dt, || self.rec_sample());
            self.rec = Some(r);
        }
        if self.roll_done() {
            self.finish_roll();
        }
    }

    fn draw(&mut self) {
        let mut s = self.b.sf();
        ui::draw_background(&mut s, self.b.width, self.b.height);
        if self.state == State::Setup {
            self.draw_setup(&mut s);
            return;
        }
        self.draw_lane(&mut s);
        self.draw_pins(&mut s);
        if self.ball.is_some() {
            self.draw_ball(&mut s);
        } else if self.state == State::Play {
            self.draw_controls(&mut s);
        }
        self.draw_hud(&mut s);
        self.draw_card(&mut s);
        if self.state == State::Over {
            self.draw_over(&mut s);
        }
    }
}

/// Eintrag fuer die Spieleliste.
pub const INFO: GameInfo = GameInfo {
    key: "bowling",
    class_name: "BowlingGame",
    supports_multiplayer: true,
    modes: &[],
    make: BowlingGame::new,
};
